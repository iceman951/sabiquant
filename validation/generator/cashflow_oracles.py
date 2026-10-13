#!/usr/bin/env python3
"""Independent civil/Fraction and two-precision Decimal cash-flow analytics."""

import argparse
from collections import Counter
import datetime as dt
from decimal import Decimal as D, localcontext
from fractions import Fraction as F
from functools import lru_cache
import hashlib
import math
import pathlib
import platform
import random
import struct
import sys

import rate_oracles as rates

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/cashflows/analytics-oracle.fixture"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
RATE_REVISION = hashlib.sha256(pathlib.Path(rates.__file__).read_bytes()).hexdigest()
MIN_SUB = math.ulp(0.0)
MAX = sys.float_info.max
U = F(1, 2**53)
PRECS = (120, 200)
CONTEXTS = {
    "A360": "scalar:Actual360",
    "A365": "scalar:Actual365Fixed",
    "ISDA": "scalar:ActualActualIsda",
    "30E": "scalar:ThirtyE360",
    "BOND": "scalar:Thirty360BondBasis",
    "30E-TERM": "termination:2024-02-29",
    "ICMA-M": "icma:Monthly:" + "/".join(
        ["2023-12-01"] + [f"2024-{m:02}-01" for m in range(1, 13)] + ["2025-01-01"]),
    "ICMA-A": "icma:Annual:2023-01-01/2024-01-01/2025-01-01",
    "ICMA-D": "icma:Daily:" + "/".join(
        (dt.date(2024, 1, 1) + dt.timedelta(days=i)).isoformat() for i in range(61)),
    "B252-N": "business:0:",
    "B252-W": "business:96:",
    "B252-C": "business:96:2023-12-25/2024-01-01/2024-02-29/2024-12-25",
    "B252-T": "target:VerifiedOnly",
}
COUNTS = dict(rates.FREQUENCIES)
TARGET_2026 = {dt.date(2026, 1, 1), dt.date(2026, 4, 3), dt.date(2026, 4, 6),
               dt.date(2026, 5, 1), dt.date(2026, 12, 25), dt.date(2026, 12, 26)}
COLUMNS = ("id operation context start end payment notional coupon_rate as_of inclusion "
           "flows flow_products query_flow_products discount_context method frequency discount_rate work_budget "
           "alpha_n alpha_d alpha alpha_products query_mode query_n query_d query_alpha query_products "
           "expected allowance gross behavior error_kind cause_kind error_index error_phase "
           "work_used work_required tolerance").split()


class ModelError(Exception):
    def __init__(self, kind):
        self.kind = kind


def date(text):
    return dt.date.fromisoformat(text)


def bits(value):
    return struct.unpack(">Q", struct.pack(">d", value))[0]


def binary(word):
    return struct.unpack(">d", struct.pack(">Q", word))[0]


def rounded(value):
    try:
        return float(value)
    except OverflowError:
        return math.inf if value > 0 else -math.inf


def decimal(value):
    return D(value.numerator) / D(value.denominator) if isinstance(value, F) else D.from_float(value)


def reference_dates(context):
    descriptor = CONTEXTS[context]
    return [date(x) for x in descriptor.split(":")[2].split("/")] if descriptor.startswith("icma:") else []


def charge(context, start, end):
    if start == end:
        return 1
    if context.startswith("ICMA"):
        low, high = sorted((start, end))
        refs = reference_dates(context)
        return 1 + sum(left < high and right > low for left, right in zip(refs, refs[1:]))
    return 1 + abs((end - start).days) if context.startswith("B252") else 1


@lru_cache(maxsize=None)
def yf(context, start, end):
    """Exact convention formula from civil dates; no Rust/reference outputs."""
    if start > end:
        return -yf(context, end, start)
    days = (end - start).days
    if context.startswith("ICMA"):
        refs = reference_dates(context)
        if start < refs[0] or end > refs[-1]:
            raise ModelError("rate.icma")
        count = COUNTS[CONTEXTS[context].split(":")[1]]
        return sum((F((min(end, right) - max(start, left)).days, count * (right-left).days)
                    for left, right in zip(refs, refs[1:]) if left < end and right > start), F())
    if context == "A360":
        return F(days, 360)
    if context == "A365":
        return F(days, 365)
    if context == "ISDA":
        total = F()
        cursor = start
        while cursor < end:
            boundary = min(end, dt.date(cursor.year + 1, 1, 1)) if cursor.year < 9999 else end
            length = 366 if cursor.year % 4 == 0 and (cursor.year % 100 != 0 or cursor.year % 400 == 0) else 365
            total += F((boundary-cursor).days, length)
            cursor = boundary
        return total
    if context in ("30E", "BOND", "30E-TERM"):
        a, b = start.day, end.day
        if context == "30E":
            a, b = min(a, 30), min(b, 30)
        elif context == "BOND":
            a = min(a, 30)
            b = 30 if b == 31 and a == 30 else b
        else:
            last_feb = lambda x: x.month == 2 and (x + dt.timedelta(days=1)).month == 3
            a = 30 if a == 31 or last_feb(start) else a
            b = 30 if b == 31 or last_feb(end) and end != date("2024-02-29") else b
        return F(360*(end.year-start.year)+30*(end.month-start.month)+b-a, 360)
    if context == "B252-T":
        if not date("2026-01-01") <= start <= date("2027-01-01") or not date("2026-01-01") <= end <= date("2027-01-01"):
            raise ModelError("rate.day_count")
        mask, holidays = 96, TARGET_2026
    else:
        _, mask, text = CONTEXTS[context].split(":")
        mask = int(mask)
        holidays = {date(x) for x in text.split("/") if x}
    count = sum(not (mask & (1 << (start+dt.timedelta(days=i)).weekday()))
                and start+dt.timedelta(days=i) not in holidays for i in range(days))
    return F(count, 252)


def alpha_candidates(context, exact):
    center = rounded(exact)
    radius = 4 if context == "ISDA" or context.startswith("ICMA") else 0
    if center == 0:
        return [0.0]
    return [binary(word) for word in range(bits(center)-radius, bits(center)+radius+1)]


def product(notional, rate, alpha):
    exact = F.from_float(notional) * F.from_float(rate) * F.from_float(alpha)
    output = rounded(exact)
    if not math.isfinite(output) or exact and not output:
        return None
    return 0.0 if not output else output


def products(context, exact, notional, rate):
    entries=[]
    for a in alpha_candidates(context,exact):
        real=F.from_float(notional)*F.from_float(rate)*F.from_float(a)
        p=rounded(real)
        result="overflow" if not math.isfinite(p) else "underflow" if real and not p else f"{bits(0.0 if not p else p):016x}"
        entries.append(f"{bits(a):016x}="+result)
    return ";".join(entries)


def alpha_fields(prefix, context, exact, notional, rate):
    stem = "alpha" if not prefix else "query"
    return {stem+"_n":str(exact.numerator),stem+"_d":str(exact.denominator),
            ("alpha" if not prefix else "query_alpha"):repr(rounded(exact)),
            stem+"_products":products(context, exact, notional, rate)}


def eligible(payment, cutoff, inclusion):
    return payment > cutoff or payment == cutoff and inclusion == "Include"


def windows(context):
    if context == "B252-T":
        return [(date(a),date(b)) for a,b in [("2026-01-01","2026-01-02"),("2026-04-02","2026-04-07"),
                ("2026-04-03","2026-04-06"),("2026-12-01","2027-01-01")]]
    if context == "ICMA-D":
        return [(date(a),date(b)) for a,b in [("2024-01-01","2024-01-02"),("2024-01-15","2024-02-15"),
                ("2024-02-28","2024-03-01")]]
    if context.startswith("ICMA"):
        return [(date(a),date(b)) for a,b in [("2024-01-01","2024-01-31"),("2024-01-15","2024-07-01"),
                ("2024-02-29","2024-10-15"),("2023-12-15","2024-03-15")]
                if date(a) >= reference_dates(context)[0]]
    return [(date(a),date(b)) for a,b in [("2024-01-30","2024-01-31"),("2024-02-28","2024-02-29"),
            ("2024-02-29","2024-03-31"),("2024-01-01","2024-07-01"),
            ("2023-07-15","2024-07-15"),("1900-02-28","1900-03-01"),
            ("2000-02-28","2000-03-01"),("2099-12-31","2100-03-01")]]


def coupon_rows():
    for context in CONTEXTS:
        for start,end in windows(context):
            exact = yf(context,start,end)
            payment = end+dt.timedelta(days=7)
            for notional in [-MAX,-1000.0,-MIN_SUB,-0.0,0.0,MIN_SUB,1000.0,MAX]:
                for rate in [-2.0,-0.05,-MIN_SUB,-0.0,0.0,MIN_SUB,0.05,2.0,MAX]:
                    row = dict(operation="coupon",context=context,start=str(start),end=str(end),payment=str(payment),
                               notional=repr(notional),coupon_rate=repr(rate),behavior="product-map",tolerance="TOL-E0-PRODUCT")
                    row.update(alpha_fields(False,context,exact,notional,rate))
                    yield row
    # Exact elementary alpha anchors expose recoverable intermediate range
    # and ties without confusing the property with upstream YF uncertainty.
    for end in [date("2023-07-01"),date("2024-01-01"),date("2025-01-01")]:
        start = date("2023-01-01")
        for notional,rate in [(MAX,2.0),(MIN_SUB,0.5),(MIN_SUB,math.nextafter(0.5,1.0)),
                              (MAX,math.nextafter(1.0,0.0)),(-MAX,-2.0)]:
            exact = yf("30E",start,end)
            row = dict(operation="coupon",context="30E",start=str(start),end=str(end),payment=str(end),
                       notional=repr(notional),coupon_rate=repr(rate),behavior="product-map",tolerance="TOL-E0-PRODUCT")
            row.update(alpha_fields(False,"30E",exact,notional,rate))
            yield row
    # Balanced exponent inputs preserve a finite true output while covering
    # every normal/subnormal power-of-two bin and rich 53-bit significands.
    randomizer=random.Random(0x5AB1CF2026)
    start,end=date("2024-01-01"),date("2024-01-02")
    for exponent in range(-1074,1024):
        n=math.ldexp(1+randomizer.randrange(2**52)/2**52,exponent)
        r=math.ldexp(1+randomizer.randrange(2**52)/2**52,min(1023,-exponent))
        for sign in [-1.0,1.0]:
            exact=yf("A365",start,end)
            row=dict(operation="coupon",context="A365",start=str(start),end=str(end),payment=str(end),
                     notional=repr(n),coupon_rate=repr(sign*r),behavior="product-map",tolerance="TOL-E0-PRODUCT")
            row.update(alpha_fields(False,"A365",exact,n,sign*r))
            yield row
    # Exact overflow midpoint factors: (2^27−1)(2^27+1)2^970.
    a=math.ldexp(2**27-1,500); b=math.ldexp(2**27+1,470)
    start,end=date("2023-01-01"),date("2024-01-01")
    for n in [math.nextafter(a,0.0),a,math.nextafter(a,math.inf)]:
        for sign in [-1.0,1.0]:
            row=dict(operation="coupon",context="30E",start=str(start),end=str(end),payment=str(end),
                     notional=repr(n),coupon_rate=repr(sign*b),behavior="product-map",tolerance="TOL-E0-PRODUCT")
            row.update(alpha_fields(False,"30E",F(1),n,sign*b))
            yield row


def accrued_rows():
    for context in CONTEXTS:
        for start,end in windows(context)[:4]:
            full = yf(context,start,end)
            for payment in [start+(end-start)//2,end,end+dt.timedelta(days=7)]:
                instants = sorted({start-dt.timedelta(days=1),start,start+(end-start)//2,end,
                                   payment,payment+dt.timedelta(days=1)})
                for as_of in instants:
                    for inclusion in ["Include","Exclude"]:
                        mode = "zero" if not eligible(payment,as_of,inclusion) or as_of<=start else "full" if as_of>=end else "partial"
                        exact = F() if mode=="zero" else full if mode=="full" else yf(context,start,as_of)
                        for notional,rate in [(1e6,0.037),(-1e6,0.037),(1e6,-0.037),(0.0,0.037)]:
                            row = dict(operation="accrued",context=context,start=str(start),end=str(end),payment=str(payment),
                                       notional=repr(notional),coupon_rate=repr(rate),as_of=str(as_of),inclusion=inclusion,
                                       query_mode=mode,behavior="product-map",tolerance="TOL-E0-PRODUCT")
                            row.update(alpha_fields(False,context,full,notional,rate))
                            row.update(alpha_fields(True,context,exact,notional,rate))
                            yield row


def encode(flows):
    return ";".join("@".join(str(x) if not isinstance(x,float) else repr(x) for x in flow) for flow in flows)


def pay(flow):
    return flow[3] if flow[0]=="C" else flow[1]


def maps_for_flows(flows,as_of=None,inclusion="Include"):
    output=[]
    for i,flow in enumerate(flows):
        if flow[0]!="C":
            continue
        _,start,end,payment,n,r,context=flow
        exact=yf(context,start,end) if as_of is None else F() if not eligible(payment,as_of,inclusion) or as_of<=start else yf(context,start,min(end,as_of))
        output.append(str(i)+"@"+products(context,exact,n,r))
    return "|".join(output)


def flow_amount(flow):
    if flow[0]=="S":
        return F.from_float(flow[2]),F()
    _,start,end,payment,notional,rate,context = flow
    exact = yf(context,start,end)
    ideal = F.from_float(notional)*F.from_float(rate)*exact
    outputs = [product(notional,rate,a) for a in alpha_candidates(context,exact)]
    if any(x is None for x in outputs):
        raise ArithmeticError("invalid eager coupon in a valuation scenario")
    return ideal,max(abs(F.from_float(x)-ideal) for x in outputs)


def discount(rate,method,time,precision):
    with localcontext() as c:
        c.prec=precision
        phase=rates.phase(method[0],method[2],float(time))
        r,t=decimal(rate),decimal(time)
        if phase=="Simple":
            compound=1+r*t
            if compound<=0:
                raise ModelError("rate.value")
            return 1/compound,compound.ln()
        growth=r*t if phase=="Continuous" else D(method[2])*t*rates.log1p(r/D(method[2]))
        return (-growth).exp(),growth


def pv_error(amount,amount_error,context,start,end,rate,method,precision):
    exact_time=yf(context,start,end)
    with localcontext() as c:
        c.prec=precision
        ideal,growth=discount(rate,method,exact_time,precision)
        allowance=D(0)
        for time in alpha_candidates(context,exact_time):
            d,g=discount(rate,method,F.from_float(time),precision)
            elementary=D(0) if rate==0 or time==0 else D(8)*decimal(U)*(1+abs(g))*abs(d)+decimal(MIN_SUB)
            allowance=max(allowance,abs(d-ideal)+elementary)
        a,ae=decimal(amount),decimal(amount_error)
        product_round=D(0) if rate==0 and not amount_error else decimal(U)*(abs(a)+ae)*(abs(ideal)+allowance)+decimal(F.from_float(MIN_SUB)/2)
        error=abs(ideal)*ae+abs(a)*allowance+ae*allowance+product_round
        return a*ideal,error,abs(a*ideal)


def first_error(flows,context,valuation,rate,method,inclusion,budget):
    used=0
    queries=[(None,valuation,"valuation anchor")]+[(i,flow[3] if flow[0]=="C" else flow[1],"discount horizon")
             for i,flow in enumerate(flows) if eligible(flow[3] if flow[0]=="C" else flow[1],valuation,inclusion)]
    for index,payment,phase in queries:
        required=used+charge(context,valuation,payment)
        if required>budget:
            return dict(behavior="error",error_kind="cashflow.accrual_work_limit_exceeded",error_index="" if index is None else str(index),
                        error_phase=phase,work_used=str(used),work_required=str(required))
        used=required
        try:
            exact=yf(context,valuation,payment)
        except ModelError as error:
            return dict(behavior="error",error_kind="cashflow.rate",cause_kind=error.kind,error_index="" if index is None else str(index),error_phase=phase)
        if index is None:
            continue
        time=rounded(exact)
        case=("discount",method,rate,time,1.0,method)
        answers=[rates.expected(case,p) for p in PRECS]
        if answers[0]!=answers[1]:
            raise ArithmeticError("two-precision factor status disagreement")
        if answers[0][0]=="error":
            return dict(behavior="error",error_kind="cashflow.rate",cause_kind=answers[0][1],error_index=str(index),error_phase=phase)
        amount,_=flow_amount(flows[index])
        if flows[index][0]=="C":
            amount=F.from_float(product(flows[index][4],flows[index][5],rounded(yf(flows[index][6],flows[index][1],flows[index][2]))))
        term=amount*F.from_float(float(answers[0][2]))
        result=rounded(term)
        if not math.isfinite(result) or term and not result:
            return dict(behavior="error",error_kind="cashflow.rate",cause_kind="rate.value",error_index=str(index),error_phase=phase)
    return None


def valuation_row(flows,context,valuation,rate,method,inclusion,budget):
    row=dict(operation="npv",flows=encode(flows),discount_context=context,as_of=str(valuation),
             discount_rate=repr(rate),method=method[0],frequency=method[1],inclusion=inclusion,work_budget=str(budget),
             tolerance="TOL-E5-CASHFLOW-PROPAGATION")
    row["flow_products"]=maps_for_flows(flows)
    error=first_error(flows,context,valuation,rate,method,inclusion,budget)
    if error:
        row.update(error)
        return row
    selected=[flow for flow in flows if eligible(flow[3] if flow[0]=="C" else flow[1],valuation,inclusion)]
    if not selected or rate==0 and all(flow[0]=="S" for flow in selected):
        exact=sum((F.from_float(flow[2]) for flow in selected),F())
        expected=rounded(exact)
        if not math.isfinite(expected):
            row.update(behavior="error",error_kind="cashflow.sum",cause_kind="sum.arithmetic",error_phase="NPV reduction")
        else:
            row.update(behavior="value",expected=repr(0.0 if not expected else expected),allowance="0.0",
                       gross=repr(rounded(sum((abs(F.from_float(flow[2])) for flow in selected),F()))),tolerance="TOL-E0-SUM")
        return row
    answers=[]
    for precision in PRECS:
        with localcontext() as c:
            c.prec=precision
            value,error,gross=D(0),D(0),D(0)
            for flow in selected:
                amount,ae=flow_amount(flow)
                payment=flow[3] if flow[0]=="C" else flow[1]
                term,bound,magnitude=pv_error(amount,ae,context,valuation,payment,rate,method,precision)
                value+=term; error+=bound; gross+=magnitude
            error+=decimal(U)*(abs(value)+error)+decimal(F.from_float(MIN_SUB)/2)
            answers.append((rounded(value),rounded(error),rounded(gross)))
    if answers[0]!=answers[1]:
        raise ArithmeticError("two-precision NPV result/bound disagreement: "+str(answers))
    expected,allowance,gross=answers[0]
    if not math.isfinite(expected) or not math.isfinite(allowance):
        raise ArithmeticError("unexpected saturated nominal valuation fixture")
    row.update(behavior="value",expected=repr(0.0 if not expected else expected),
               allowance=repr(math.nextafter(allowance,math.inf)),gross=repr(gross))
    return row


def valuation_rows():
    for context in CONTEXTS:
        valuation=date("2026-01-01") if context=="B252-T" else date("2024-01-01")
        last=date("2027-01-01") if context=="B252-T" else date("2024-03-01") if context=="ICMA-D" else date("2025-01-01")
        quarter=valuation+(last-valuation)//4
        next_day=valuation+dt.timedelta(days=1)
        patterns=[[],[("S",valuation,100.0),("S",valuation,-40.0),("S",next_day,1.0)],
                  [("S",quarter,1000.0),("S",last,-400.0)],
                  [("S",last,1e16),("S",last,1.0),("S",last,-1e16)],
                  [("S",quarter,0.0),("S",last,-1e-100),("S",last,1e-100)]]
        for method in rates.METHODS:
            values=[-0.05,-1e-20,0.0,MIN_SUB,0.05,1.0]
            for rate in values:
                for flows in patterns:
                    for inclusion in ["Include","Exclude"]:
                        yield valuation_row(flows,context,valuation,rate,method,inclusion,2**64-1)
    # Integer/exact-factor reductions require bitwise answers even with
    # unrepresentable gross magnitude; no cancellation tolerance can mask them.
    v=date("2024-01-01"); p=date("2025-01-01")
    for values in [[MAX,MAX,-MAX],[MAX,MAX,-MAX,-MAX,MIN_SUB],[MAX,MAX],
                   [1e16,1.0,-1e16],[-0.0,0.0],[-MAX,-MAX,MAX]]:
        yield valuation_row([("S",p,x) for x in values],"A365",v,0.0,rates.METHODS[0],"Include",2**64-1)
    # Every periodic/hybrid switch is reached or bracketed by CIVIL horizons;
    # the six month-based counts and Daily reach the exact rounded threshold.
    for method in rates.METHODS:
        count=method[2]
        if not count:
            continue
        if count in (1,2,3,4,6,12):
            v=date("2024-01-01")
            months=12//count
            year=2024+months//12
            month=months%12+1
            switch=dt.date(year,month,1)
            context="30E"
        else:
            v=date("2024-01-01")
            switch=v+dt.timedelta(days=365//count)
            context="A365"
        for rate in [-0.05,0.05,math.nextafter(-float(count),math.inf)]:
            for payment in [switch-dt.timedelta(days=1),switch,switch+dt.timedelta(days=1)]:
                yield valuation_row([("S",payment,1.0)],context,v,rate,method,"Include",2**64-1)
    # Dedicated strict factor/domain errors and conditional work precedence.
    v=date("2023-01-01"); p=date("2024-01-01")
    for method,rate in [(rates.METHODS[0],-1.0),(rates.METHODS[1],710.0),(rates.METHODS[1],-750.0),
                        (rates.METHODS[1],-745.0),(rates.METHODS[1],math.log(MAX))]:
        for amount in [0.0,MIN_SUB,1.0,-1.0,MAX]:
            yield valuation_row([("S",p,amount)],"A365",v,rate,method,"Include",2**64-1)
    for context in ["A365","ICMA-M","ICMA-D","B252-W","B252-C","B252-T"]:
        v=date("2026-01-01") if context=="B252-T" else date("2024-01-01")
        payments=[v,v+dt.timedelta(days=1),v+dt.timedelta(days=7),v+dt.timedelta(days=28)]
        flows=[("S",p,float(i+1)) for i,p in enumerate(payments)]
        for inclusion in ["Include","Exclude"]:
            total=1+sum(charge(context,v,p) for p in payments if eligible(p,v,inclusion))
            for budget in sorted({0,1,total-1,total,total+1}):
                yield valuation_row(flows,context,v,0.05,rates.METHODS[0],inclusion,budget)
    # The same-date anchor guard must run even when no cash flow is eligible.
    for context,v in [("ICMA-M",date("2023-11-30")),("ICMA-A",date("2022-12-31")),
                      ("ICMA-D",date("2024-03-02"))]:
        for budget in [0,1]:
            yield valuation_row([],context,v,0.0,rates.METHODS[0],"Exclude",budget)
    # Coupon accrual convention is independent of discount accrual/compounding.
    for context in CONTEXTS:
        start,end=windows(context)[0]
        payment=end+dt.timedelta(days=7)
        flows=[("S",start,-50.0),("C",start,end,payment,1e6,0.037,context),
               ("C",start,end,payment,-3e5,0.019,context),
               ("C",start,end,payment,2e5,-0.041,context),("S",payment,25.0)]
        for discount_context in ["A365","B252-W"]:
            for method in rates.METHODS:
                for rate in [-0.05,0.0,0.05]:
                    for inclusion in ["Include","Exclude"]:
                        yield valuation_row(flows,discount_context,start,rate,method,inclusion,2**64-1)


def accrued_leg_row(flows,as_of,inclusion,budget):
    row=dict(operation="leg-accrued",flows=encode(flows),as_of=str(as_of),inclusion=inclusion,
             work_budget=str(budget),tolerance="TOL-E5-CASHFLOW-PROPAGATION")
    row["flow_products"]=maps_for_flows(flows)
    row["query_flow_products"]=maps_for_flows(flows,as_of,inclusion)
    used=0; total=F(); allowance=F(); gross=F()
    for i,flow in enumerate(flows):
        if flow[0]=="S" or not eligible(pay(flow),as_of,inclusion):
            continue
        _,start,end,payment,notional,rate,context=flow
        if as_of<=start:
            continue
        if as_of<end:
            required=used+charge(context,start,as_of)
            if required>budget:
                row.update(behavior="error",error_kind="cashflow.accrual_work_limit_exceeded",error_index=str(i),
                           error_phase="partial accrual",work_used=str(used),work_required=str(required))
                return row
            used=required
        exact=yf(context,start,min(end,as_of))
        ideal=F.from_float(notional)*F.from_float(rate)*exact
        candidates=[product(notional,rate,a) for a in alpha_candidates(context,exact)]
        if any(x is None for x in candidates):
            raise ArithmeticError("unexpected partial-product domain ambiguity")
        error=max(abs(F.from_float(x)-ideal) for x in candidates)
        total+=ideal; allowance+=error; gross+=abs(ideal)
    allowance+=U*(abs(total)+allowance)+F.from_float(MIN_SUB)/2
    bound=rounded(allowance)
    if F.from_float(bound)<allowance:
        bound=math.nextafter(bound,math.inf)
    row.update(behavior="value",expected=repr(0.0 if not total else rounded(total)),
               allowance=repr(bound),gross=repr(rounded(gross)))
    return row


def accrued_leg_rows():
    for context in CONTEXTS:
        (start,end),(other_start,other_end)=windows(context)[:2]
        early=start+(end-start)//2
        flows=[("C",start,end,early,1e6,0.037,context),
               ("C",start,end,end+dt.timedelta(days=7),-1e6,0.037,context),
               ("C",other_start,other_end,other_end+dt.timedelta(days=7),2e6,0.019,context),
               ("S",end+dt.timedelta(days=7),500.0)]
        flows.sort(key=pay)
        for as_of in sorted({start-dt.timedelta(days=1),start,early,end,end+dt.timedelta(days=7),
                             other_start+(other_end-other_start)//2,other_end}):
            for inclusion in ["Include","Exclude"]:
                total=sum(charge(flow[6],flow[1],as_of) for flow in flows if flow[0]=="C" and
                          eligible(pay(flow),as_of,inclusion) and flow[1]<as_of<flow[2])
                for budget in sorted({0,max(0,total-1),total,total+1}):
                    yield accrued_leg_row(flows,as_of,inclusion,budget)
        cancelled=[("C",start,end,end+dt.timedelta(days=7),1e6,0.037,context),
                   ("C",start,end,end+dt.timedelta(days=7),-1e6,0.037,context)]
        for as_of in [start,start+(end-start)//2,end,end+dt.timedelta(days=7)]:
            for inclusion in ["Include","Exclude"]:
                row=accrued_leg_row(cancelled,as_of,inclusion,2**64-1)
                row.update(expected="0.0",allowance="0.0",tolerance="TOL-E0-SUM")
                yield row


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check",action="store_true")
    args=parser.parse_args()
    rows=list(coupon_rows())+list(accrued_rows())+list(valuation_rows())+list(accrued_leg_rows())
    lines=["\t".join(COLUMNS)]
    counts=Counter()
    profiles=Counter()
    classifications=Counter()
    for index,row in enumerate(rows):
        row["id"]=f"CF-ANALYTICS-{index:06}"
        counts[row["operation"]]+=1
        profiles[row["tolerance"]]+=1
        if row["behavior"]=="product-map":
            outcomes={value if value in ("underflow","overflow") else "value"
                      for entry in row["alpha_products"].split(";") for _,value in [entry.split("=")]}
            classifications["product-"+"-or-".join(sorted(outcomes))]+=1
        else:
            classifications[row["error_kind"] if row["behavior"]=="error" else "aggregate-value"]+=1
        lines.append("\t".join(str(row.get(column,"")) for column in COLUMNS))
    metadata={"schema":"1","fixture-set":"CASHFLOW-ANALYTICS-ORACLE","domain":"cashflows",
        "description":"Independent civil daycounts, Fraction binary64 coupon products and two-precision Decimal direct flat NPV; explicit contextual work",
        "reference":"oracle:Python Fraction/Decimal","reference-version":platform.python_version()+"; Decimal120/200 digits",
        "reference-source":"M7_CASHFLOW_CONTRACT formulas; civil datetime/Fraction and independent Decimal discount; ECB TARGET-marked2026 closures",
        "tolerance":"TOL-E0-PRODUCT/SUM; existing TOL-E1/E2 alpha; TOL-E5-CASHFLOW-PROPAGATION from unchanged E3k8 plus amount/time/rounding",
        "reproduction":"generator-sha256="+REVISION+"; rate-helper-sha256="+RATE_REVISION+"; python3 validation/generator/cashflow_oracles.py",
        "generated-at":dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00","Z"),
        "platform":platform.system()+" "+platform.machine(),
        "target-source":"https://www.ecb.europa.eu/ecb/contacts/working-hours/html/index.en.html; TARGET-marked2026 only; queried days are2026",
        "record-count":str(len(rows)),"operation-counts":";".join(k+"="+str(v) for k,v in sorted(counts.items())),
        "profile-counts":";".join(k+"="+str(v) for k,v in sorted(profiles.items())),
        "classification-counts":";".join(k+"="+str(v) for k,v in sorted(classifications.items()))}
    metadata.update({"context-"+key.lower():value for key,value in CONTEXTS.items()})
    body="\n".join(lines)+"\n"
    if args.check:
        header,oldbody=DEST.read_text().split("\n\n",1)
        old=dict(line.split(": ",1) for line in header.splitlines())
        if oldbody!=body or any(old.get(k)!=v for k,v in metadata.items() if k not in ("generated-at","platform")):
            raise SystemExit("fixture differs: "+str(DEST))
        action="verified"
    else:
        DEST.parent.mkdir(parents=True,exist_ok=True)
        DEST.write_bytes(("\n".join(k+": "+v for k,v in metadata.items())+"\n\n"+body).encode())
        action="generated"
    print(action,DEST.relative_to(ROOT),len(rows),counts,flush=True)


if __name__=="__main__":
    main()
