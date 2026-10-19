#!/usr/bin/env python3
"""Independent conventional-gilt source/date/cash oracle; Python stdlib only.

Written from the frozen contract before gilt production. No Rust financial
implementation, QuantLib implementation or observed library result is read.
Fraction is authoritative; separately decoded Decimal rational expressions at
1536/2048 digits check source values and every visited cash rounding decision.
"""

import argparse
from bisect import bisect_right
from calendar import monthrange
from collections import Counter
from dataclasses import dataclass, replace
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal, ROUND_FLOOR, localcontext
from fractions import Fraction as F
import hashlib
import json
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/bonds/gilt-source-cash-oracle.fixture"
CONTRACT = ROOT / "docs/M8_GILT_CONTRACT.md"
CONTRACT_SHA = "03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292"
REQ_SHA = "2eb23647be550fa45c4fceaa252c9db1f8833190a6f337c2fafd1f6ec9e423d2"
MIN, MAX = -(2**127), 2**127 - 1
PRECISIONS = (1536, 2048)
MODES = ("TowardZero", "AwayFromZero", "TowardPositiveInfinity",
         "TowardNegativeInfinity", "NearestTiesEven", "NearestTiesAwayFromZero", "Exact")
RECIPES = ("PublishedDividendPer100", "SourceAccruedPer100",
           "DirtyPricePer100", "CleanPriceAndSourceAccruedPer100")
AGGS = ("RoundTotal", "RoundComponentsThenSum")
COLUMNS = ("id profile op issue maturity coupon first quasi ex_dates eom date_cap "
           "settlement bundle_cap dividend_index recipe nominal evidence aggregation "
           "quantum mode expected context first_period first_quasi dividends source_values "
           "ai ai_bits next_quasi r s n bundles entitlements source_value lower upper guarantee").split()
CHECKS, WIDTHS = Counter(), Counter()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    h = 14695981039346656037
    for b in data:
        h = ((h ^ b) * 1099511628211) & (2**64 - 1)
    return "%016x" % h


def canon(c, s=0):
    if c == 0:
        return 0, 0
    while s and c % 10 == 0:
        c //= 10
        s -= 1
    return c, s


def cell(p):
    return "%d:%d" % p


def amount(p):
    return F(p[0], 10**p[1])


def fcell(x):
    return "%d/%d" % (x.numerator, x.denominator)


def amount_pair(x):
    d = x.denominator
    twos = fives = 0
    while d % 2 == 0:
        d //= 2
        twos += 1
    while d % 5 == 0:
        d //= 5
        fives += 1
    assert d == 1
    s = max(twos, fives)
    return canon(x.numerator * 2**(s-twos) * 5**(s-fives), s)


def bits(x):
    return struct.unpack(">Q", struct.pack(">d", x))[0]


def binary(b):
    return struct.unpack(">d", struct.pack(">Q", b))[0]


def dc(c, s=0):
    return "d:" + cell(canon(c, s))


def di(c, s, e, t=None):
    return "di:%d:%d:%d:%d" % (*canon(c, s), *canon(e, s if t is None else t))


def bc(b):
    return "b:%016x" % b


def bi(lo, hi):
    return "bi:%016x:%016x" % (lo, hi)


def hx(s):
    return "x:" + s.encode().hex()


class Failure(Exception):
    def __init__(self, kind, **context):
        self.kind, self.context = kind, context


def context_cell(context):
    return ",".join("%s=%s" % (k, v) for k, v in sorted(context.items())) or "-"


def add_months(anchor, displacement, eom):
    coordinate = anchor.year * 12 + anchor.month - 1 + displacement
    y, m0 = divmod(coordinate, 12)
    if not 1 <= y <= 9999:
        raise Failure("gilt.date", index="PENDING", phase=hx("quasi cycle"),
                      cause_type="DateError", reference_cause="date.out_of_range",
                      reference_date=anchor.isoformat(),
                      reference_operation=hx("add months"), reference_count=displacement)
    m = m0 + 1
    last = monthrange(y, m)[1]
    preserve = eom == "Preserve" and anchor.day == monthrange(anchor.year, anchor.month)[1]
    return date(y, m, last if preserve else min(anchor.day, last))


@dataclass(frozen=True)
class Definition:
    issue: date
    maturity: date
    coupon: tuple
    first: date
    quasi: tuple
    ex: tuple
    eom: str = "Ignore"
    cap: int = 20000


def build(maturity=date(2025, 12, 7), periods=4, role="Regular", coupon=(6, 0),
          eom="Ignore", issue_offset=31, cutoff_days=7):
    q = tuple(add_months(maturity, -6*k, eom) for k in range(periods, -1, -1))
    issue = q[0] if role == "Regular" else q[1] - timedelta(days=issue_offset)
    fidx = 2 if role == "Long" else 1
    ex = tuple(max(q[i-1] if i > fidx or role == "Long" else issue,
                   q[i]-timedelta(days=cutoff_days)) for i in range(fidx, len(q)))
    return Definition(issue, maturity, canon(*coupon), q[fidx], q, ex, eom)


def round_count(x, mode):
    """Exact independent integer quantum-count decision, before public admission."""
    sign = -1 if x < 0 else 1
    k, r = divmod(abs(x.numerator), x.denominator)
    if r and mode == "Exact":
        return None
    inc = r and (mode == "AwayFromZero" or
                 mode == "TowardPositiveInfinity" and sign > 0 or
                 mode == "TowardNegativeInfinity" and sign < 0 or
                 mode == "NearestTiesEven" and (2*r > x.denominator or
                                               2*r == x.denominator and k % 2 == 1) or
                 mode == "NearestTiesAwayFromZero" and 2*r >= x.denominator)
    return sign * (k + bool(inc))


@dataclass
class DR:
    """Unreduced Decimal-integer rational; separate decoder/formula path."""
    n: Decimal
    d: Decimal

    def plus(self, other):
        return DR(self.n*other.d + other.n*self.d, self.d*other.d)

    def times(self, other):
        return DR(self.n*other.n, self.d*other.d)

    def divide(self, other):
        assert other.n != 0
        return DR(self.n*other.d, self.d*other.n)


def dr(n, d=1):
    return DR(Decimal(n), Decimal(d))


def decimal_pair(p):
    return dr(p[0], 10**p[1])


def decimal_round(x, q, mode):
    """Decimal division/floor followed by exact Decimal remainder comparison."""
    v = x.divide(decimal_pair(q))
    assert v.d > 0
    n, d = abs(v.n), v.d
    WIDTHS["decimal-integer-digits"] = max(WIDTHS["decimal-integer-digits"],
                                          len(str(int(n))), len(str(int(d))))
    assert n == n.to_integral_value() and d == d.to_integral_value()
    # See report: operand digit sum <1536 gives floor/tie separation margin.
    assert len(str(int(n))) + len(str(int(d))) + 4 < min(PRECISIONS)
    quotient = (n/d).to_integral_value(rounding=ROUND_FLOOR)
    remainder = n - quotient*d
    assert 0 <= remainder < d
    negative = v.n < 0
    if remainder and mode == "Exact":
        return None
    inc = bool(remainder) and (
        mode == "AwayFromZero" or
        mode == "TowardPositiveInfinity" and not negative or
        mode == "TowardNegativeInfinity" and negative or
        mode == "NearestTiesEven" and (remainder > d-remainder or
                                      remainder == d-remainder and quotient % 2 != 0) or
        mode == "NearestTiesAwayFromZero" and remainder >= d-remainder)
    count = int(quotient) + int(inc)
    return -count if negative else count


def source_decimal(defn, fidx, ordinal=None, settlement=None):
    """Decode original coupon/day inputs; do not convert Fraction source values."""
    c, a = defn.coupon
    q = defn.quasi
    s1, r1 = (q[1]-q[0]).days, (q[1]-defn.issue).days
    if ordinal is not None:
        if ordinal or defn.issue == q[0]:
            return dr(c, 2*10**a)
        multiplier = r1 if fidx == 1 else s1+r1
        raw = dr(c*multiplier, 2*s1*10**a)
        count = decimal_round(raw, (1, 6), "NearestTiesAwayFromZero")
        return dr(count, 10**6)
    k = bisect_right(q, settlement)-1
    nxt = next(i for i in range(fidx, len(q)) if q[i] > settlement)
    ex = settlement > defn.ex[nxt-fidx]
    if settlement < defn.first and defn.issue != q[0]:
        if fidx == 1:
            elapsed = (settlement-defn.issue).days
            return dr(c*(elapsed-r1 if ex else elapsed), 2*10**a*s1)
        if settlement < q[1]:
            return dr(c*(settlement-defn.issue).days, 2*10**a*s1)
        s2, r2 = (q[2]-q[1]).days, (settlement-q[1]).days
        if ex:
            return dr(c*(r2-s2), 2*10**a*s2)
        return dr(c*(r1*s2+r2*s1), 2*10**a*s1*s2)
    s, t = (q[k+1]-q[k]).days, (settlement-q[k]).days
    return dr(c*(t-s if ex else t), 2*10**a*s)


def validate(defn):
    q = defn.quasi
    if not 2 <= defn.cap <= 20000:
        raise Failure("gilt.invalid_date_cap", cap=defn.cap, minimum=2, maximum=20000)
    if len(q) > defn.cap:
        raise Failure("gilt.too_many_quasi_dates", count=len(q), cap=defn.cap)
    if len(q) < 2:
        raise Failure("gilt.too_few_quasi_dates", count=len(q), minimum=2)
    if defn.coupon[0] < 0:
        raise Failure("gilt.negative_coupon", coupon_per_100=cell(defn.coupon))
    if defn.issue >= defn.maturity:
        raise Failure("gilt.issue_not_before_maturity", issue_date=defn.issue,
                      maturity_date=defn.maturity)
    if q[-1] != defn.maturity:
        raise Failure("gilt.maturity_boundary", maturity_date=defn.maturity,
                      last_quasi_date=q[-1])
    for i in range(1, len(q)):
        if q[i] <= q[i-1]:
            raise Failure("gilt.quasi_date_order", index=i, previous=q[i-1], date=q[i])
    if not q[0] <= defn.issue < q[1]:
        raise Failure("gilt.quasi_coverage", issue_date=defn.issue,
                      first_quasi_date=q[0], next_quasi_date=q[1])
    for i, actual in enumerate(q):
        try:
            expected = add_months(defn.maturity, -6*(len(q)-1-i), defn.eom)
        except Failure as error:
            error.context["index"] = i
            raise
        if actual != expected:
            raise Failure("gilt.quasi_date_mismatch", index=i, supplied=actual,
                          expected=expected, maturity_date=defn.maturity,
                          end_of_month=defn.eom)
    if defn.issue == q[0] and defn.first == q[1]:
        role, fidx = "Regular", 1
    elif q[0] < defn.issue and defn.first == q[1]:
        role, fidx = "Short", 1
    elif q[0] < defn.issue and len(q) > 2 and defn.first == q[2]:
        role, fidx = "Long", 2
    else:
        raise Failure("gilt.first_dividend", issue_date=defn.issue,
                      first_dividend_date=defn.first, next_quasi_date=q[1],
                      second_quasi_date=q[2] if len(q) > 2 else "None")
    if len(defn.ex) != len(q)-fidx:
        raise Failure("gilt.ex_date_count", count=len(defn.ex), expected=len(q)-fidx)
    for i, ex in enumerate(defn.ex):
        qi = fidx+i
        lower = defn.issue if qi == 1 else q[qi-1]
        if not lower <= ex <= q[qi]:
            raise Failure("gilt.ex_date_window", dividend_index=i, cutoff=ex,
                          lower=lower, dividend_date=q[qi])
    half = amount(defn.coupon)/2
    s1, r1 = (q[1]-q[0]).days, (q[1]-defn.issue).days
    first = half
    if role != "Regular":
        raw = half * (F(r1, s1) + (role == "Long"))
        first = F(round_count(raw*10**6, "NearestTiesAwayFromZero"), 10**6)
    values = (first,) + (half,)*(len(q)-fidx-1)
    for precision in PRECISIONS:
        with localcontext() as ctx:
            ctx.prec = precision
            for i, value in enumerate(values):
                d = source_decimal(defn, fidx, ordinal=i)
                assert d.n*value.denominator == d.d*value.numerator
                CHECKS["source-Decimal-%d" % precision] += 1
    return role, fidx, values


def prepare(defn, model, settlement, cap):
    role, fidx, values = model
    if cap > 20000:
        raise Failure("gilt.invalid_bundle_cap", cap=cap, maximum=20000)
    if settlement < defn.issue:
        raise Failure("gilt.settlement_before_issue", settlement_date=settlement,
                      issue_date=defn.issue)
    if settlement >= defn.maturity:
        raise Failure("gilt.settlement_at_or_after_maturity", settlement_date=settlement,
                      maturity_date=defn.maturity)
    q = defn.quasi
    k = bisect_right(q, settlement)-1
    r, s, n = (q[k+1]-settlement).days, (q[k+1]-q[k]).days, len(q)-k-2
    entitlements, bundles = [], []
    for i, value in enumerate(values):
        qi = fidx+i
        entitlement = ("Ceased" if q[qi] <= settlement else
                       "ExCouponDetached" if settlement > defn.ex[i] else "BuyerEntitled")
        entitlements.append(entitlement)
        coefficient = value if entitlement == "BuyerEntitled" else F(0)
        if qi == len(q)-1:
            coefficient += 100
        if entitlement == "BuyerEntitled" or qi == len(q)-1:
            bundles.append((qi, coefficient, r+(qi-k-1)*s))
    if len(bundles) > cap:
        raise Failure("gilt.too_many_bundles", count=len(bundles), cap=cap)
    nxt = next(i for i in range(fidx, len(q)) if q[i] > settlement)
    ex = settlement > defn.ex[nxt-fidx]
    half = amount(defn.coupon)/2
    if settlement < defn.first and role != "Regular":
        s1, r1 = (q[1]-q[0]).days, (q[1]-defn.issue).days
        if role == "Short":
            t = (settlement-defn.issue).days
            ai = half * F(t-r1 if ex else t, s1)
            branch = "short-ex" if ex else "short-cum"
        elif settlement < q[1]:
            ai = half * F((settlement-defn.issue).days, s1)
            branch = "long-first"
        else:
            t, s2 = (settlement-q[1]).days, (q[2]-q[1]).days
            ai = half * (F(t-s2, s2) if ex else F(r1, s1)+F(t, s2))
            branch = "long-second-ex" if ex else "long-second-cum"
    else:
        t = (settlement-q[k]).days
        ai = half * F(t-s if ex else t, s)
        branch = "regular-ex" if ex else "regular-cum"
    for precision in PRECISIONS:
        with localcontext() as ctx:
            ctx.prec = precision
            d = source_decimal(defn, fidx, settlement=settlement)
            assert d.n*ai.denominator == d.d*ai.numerator
            observed = float(d.n/d.d)
            assert bits(observed if observed else 0.0) == bits(float(ai))
            CHECKS["AI-Decimal-binary-%d" % precision] += 1
    return dict(ai=ai, branch=branch, k=k, r=r, s=s, n=n, bundles=bundles,
                entitlements=entitlements, next_actual=nxt-fidx, ex=ex)


def decode_evidence(text):
    cells = text.split(":")
    kind = cells.pop(0)
    if kind in ("d", "di"):
        endpoints = [amount((int(cells[i]), int(cells[i+1]))) for i in range(0, len(cells), 2)]
    else:
        endpoints = [F(binary(int(v, 16))) for v in cells]
    lo, hi = endpoints[0], endpoints[-1]
    assert lo <= hi
    return lo, hi, ("UniqueForDeclaredEnclosure" if kind in ("di", "bi") else
                    "ForSuppliedBinary64" if kind == "b" else "ForSuppliedDecimals")


def decimal_evidence(text):
    cells = text.split(":")
    kind = cells.pop(0)
    if kind in ("d", "di"):
        points = [decimal_pair((int(cells[i]), int(cells[i+1])))
                  for i in range(0, len(cells), 2)]
    else:
        # Separate exact decoder through Decimal.from_float, rather than F(bits).
        points = []
        for v in cells:
            d = Decimal.from_float(binary(int(v, 16)))
            # Canonical integer ratio avoids gratuitous 10^1074 factors for
            # MIN_SUB; this is Decimal's independent exact endpoint decoder.
            points.append(dr(*d.as_integer_ratio()))
    return points[0], points[-1]


def arithmetic_failure(stage, endpoint, cause, negative=None):
    context = dict(stage=hx(stage), endpoint=endpoint, cause="amount."+cause,
                   operation="sum" if stage == "cash net" else "quantize")
    if negative is not None:
        context["negative"] = str(negative).lower()
    raise Failure("gilt.cash_arithmetic", **context)


def quantize(x, dp, q, mode, stage, endpoint):
    qvalue = amount(q)
    count = round_count(x/qvalue, mode)
    for precision in PRECISIONS:
        with localcontext() as ctx:
            ctx.prec = precision
            value = dp(precision)
            assert value.n*x.denominator == value.d*x.numerator
            actual = decimal_round(value, q, mode)
            assert actual == count, (stage, endpoint, precision, actual, count)
            CHECKS["cash-quantizer-Decimal-%d" % precision] += 1
    WIDTHS["Fraction-ratio-numerator-bits"] = max(WIDTHS["Fraction-ratio-numerator-bits"],
                                                abs((x/qvalue).numerator).bit_length())
    WIDTHS["Fraction-ratio-denominator-bits"] = max(WIDTHS["Fraction-ratio-denominator-bits"],
                                                  (x/qvalue).denominator.bit_length())
    if count is None:
        arithmetic_failure(stage, endpoint, "inexact")
    return count*qvalue


def admit(value, stage, endpoint):
    c, s = amount_pair(value)
    assert s <= 38
    for precision in PRECISIONS:
        with localcontext() as ctx:
            ctx.prec = precision
            exact = (Decimal(value.numerator)/Decimal(value.denominator)).normalize()
            sign, digits, exponent = exact.as_tuple()
            coefficient = int("".join(map(str, digits))) * (-1 if sign else 1)
            actual = (coefficient*10**exponent, 0) if exponent >= 0 else (coefficient, -exponent)
            assert canon(*actual) == (c, s)
            CHECKS["final-normalization-Decimal-%d" % precision] += 1
    if not MIN <= c <= MAX:
        arithmetic_failure(stage, endpoint, "out_of_range", c < 0)
    return cell((c, s))


def evaluate(defn, model, plan, recipe, ordinal, nominal, evidence, aggregation, q, mode,
             view=False):
    role, fidx, values = model
    if recipe == RECIPES[0] and not 0 <= ordinal < len(values):
        raise Failure("gilt.invalid_dividend_index", index=ordinal, count=len(values))
    if nominal[0] <= 0:
        raise Failure("gilt.nonpositive_nominal", nominal=cell(nominal))
    source = values[ordinal] if recipe == RECIPES[0] else plan["ai"]
    if recipe == RECIPES[2]:
        source = None
    def dsource(_precision):
        return source_decimal(defn, fidx, ordinal=ordinal if recipe == RECIPES[0] else None,
                              settlement=plan["settlement"] if recipe != RECIPES[0] else None)
    if view:
        if source is None:
            return "none", "-", "-", "-", None
        stage = "source dividend" if recipe == RECIPES[0] else "source accrued"
        result = quantize(source, dsource, q, mode, stage, "point")
        value = admit(result, stage, "point")
        return "some", value, value, "-", source
    n = amount(nominal)/100
    guarantee = "-"
    if recipe in RECIPES[:2]:
        lo = hi = n*source
        def lowdp(p):
            return dsource(p).times(decimal_pair(nominal)).divide(dr(100))
        hidp = lowdp
        stage = "cash coupon" if recipe == RECIPES[0] else "cash accrued"
    else:
        plo, phi, guarantee = decode_evidence(evidence)
        # Independently exercise the selected unreduced40word common basis
        # using valid public input endpoints. These are observed operand widths,
        # separate from the smaller reduced Fraction numerator/denominator.
        qdates = defn.quasi
        dayd = ((qdates[1]-qdates[0]).days *
                (qdates[plan["k"]+1]-qdates[plan["k"]]).days)
        common = (2**1074 * 5**38) * (dayd if recipe == RECIPES[3] else 1)
        for price in (plo, phi):
            coefficient = price*common
            if recipe == RECIPES[3] and aggregation == AGGS[0]:
                coefficient += source*common
            assert coefficient.denominator == 1
            numerator = abs(coefficient.numerator)*nominal[0]*10**q[1]
            denominator = common*10**nominal[1]*100*q[0]
            WIDTHS["common-basis-cash-numerator-bits"] = max(
                WIDTHS["common-basis-cash-numerator-bits"], numerator.bit_length())
            WIDTHS["common-basis-cash-denominator-bits"] = max(
                WIDTHS["common-basis-cash-denominator-bits"], denominator.bit_length())
            assert numerator < 2**2458 and denominator < 2**1440
            CHECKS["valid40word-common-endpoints"] += 1
        lo, hi = n*plo, n*phi
        def lowdp(_p):
            return decimal_evidence(evidence)[0].times(decimal_pair(nominal)).divide(dr(100))
        def hidp(_p):
            return decimal_evidence(evidence)[1].times(decimal_pair(nominal)).divide(dr(100))
        stage = "cash total"
        if recipe == RECIPES[3] and aggregation == AGGS[0]:
            lo, hi = lo+n*source, hi+n*source
            def lowdp(p):
                return decimal_evidence(evidence)[0].plus(dsource(p)).times(
                    decimal_pair(nominal)).divide(dr(100))
            def hidp(p):
                return decimal_evidence(evidence)[1].plus(dsource(p)).times(
                    decimal_pair(nominal)).divide(dr(100))
        elif recipe == RECIPES[3]:
            stage = "cash capital"
    point = lo == hi
    if mode == "Exact" and not point:
        # Construct a real admissible nonmultiple, independently at both
        # Decimal precisions. Endpoints alone cannot establish Exact-definedness.
        qvalue = amount(q)
        offset = F(0) if (lo/qvalue).denominator != 1 else min((hi-lo)/2, qvalue/2)
        witness = lo+offset
        assert lo <= witness <= hi and (witness/qvalue).denominator != 1
        for precision in PRECISIONS:
            with localcontext() as ctx:
                ctx.prec = precision
                dl, dh = lowdp(precision), hidp(precision)
                assert dl.n*lo.denominator == dl.d*lo.numerator
                assert dh.n*hi.denominator == dh.d*hi.numerator
                if (lo/qvalue).denominator == 1:
                    halfwidth = dh.plus(DR(-dl.n, dl.d)).divide(dr(2))
                    halfquantum = decimal_pair(q).divide(dr(2))
                    smaller = (halfwidth if halfwidth.n*halfquantum.d <= halfquantum.n*halfwidth.d
                               else halfquantum)
                    dw = dl.plus(smaller)
                else:
                    dw = dl
                assert decimal_round(dw, q, "Exact") is None
                CHECKS["continuous-Exact-witness-Decimal-%d" % precision] += 1
        CHECKS["continuous-Exact-structural"] += 1
        return "exact_unresolved", "-", "-", stage, source
    lower = quantize(lo, lowdp, q, mode, stage, "point" if point else "lower")
    upper = lower if point else quantize(hi, hidp, q, mode, stage, "upper")
    if recipe == RECIPES[3] and aggregation == AGGS[1]:
        accrued = quantize(n*source,
                           lambda p: dsource(p).times(decimal_pair(nominal)).divide(dr(100)),
                           q, mode, "cash accrued", "point")
        lower, upper = lower+accrued, upper+accrued
        stage, point = "cash net", lower == upper
    lower_cell = admit(lower, stage, "point" if point else "lower")
    upper_cell = lower_cell if point else admit(upper, stage, "upper")
    return ("determined" if lower == upper else "ambiguous", lower_cell, upper_cell,
            guarantee if lower == upper else "-", source)


class Corpus:
    def __init__(self):
        self.rows, self.seen = [], set()
        self.profiles, self.ops, self.results = Counter(), Counter(), Counter()
        self.roles, self.branches, self.modes, self.recipes = Counter(), Counter(), Counter(), Counter()
        self.causes, self.stages, self.endpoints, self.guarantees = Counter(), Counter(), Counter(), Counter()
        self.scales = {"coupon": set(), "nominal": set(), "quantum": set()}
        self.evidence_kinds = Counter()

    def add(self, profile, defn, op="plan", settlement=None, bundle_cap=20000,
            ordinal=0, recipe="-", nominal=(100, 0), evidence="-", aggregation="-",
            quantum=(1, 2), mode="NearestTiesEven"):
        nominal, quantum = canon(*nominal), canon(*quantum)
        key = (defn, op, settlement, bundle_cap, ordinal, recipe, nominal,
               evidence, aggregation, quantum, mode)
        if key in self.seen:
            CHECKS["deduplicated-inputs"] += 1
            return
        self.seen.add(key)
        row = dict.fromkeys(COLUMNS, "-")
        row.update(id="GSC-%06d" % (len(self.rows)+1), profile=profile, op=op,
                   issue=defn.issue.isoformat(), maturity=defn.maturity.isoformat(),
                   coupon=cell(defn.coupon), first=defn.first.isoformat(),
                   quasi=",".join(d.isoformat() for d in defn.quasi) or "empty",
                   ex_dates=",".join(d.isoformat() for d in defn.ex) or "empty",
                   eom=defn.eom, date_cap=str(defn.cap))
        if settlement is not None:
            row.update(settlement=settlement.isoformat(), bundle_cap=str(bundle_cap))
        if op in ("cash", "instruction_view", "dividend_view"):
            row.update(dividend_index=str(ordinal) if recipe == RECIPES[0] else "-",
                       recipe=recipe, nominal=cell(nominal) if op != "dividend_view" else "-",
                       evidence=evidence, aggregation=aggregation,
                       quantum=cell(quantum), mode=mode)
        try:
            model = validate(defn)
            role, fidx, values = model
            row.update(first_period=role, first_quasi=str(fidx),
                       dividends=",".join(d.isoformat() for d in defn.quasi[fidx:]),
                       source_values=",".join(map(fcell, values)))
            self.scales["coupon"].add(defn.coupon[1])
            WIDTHS["source-dividend-numerator-bits"] = max(
                WIDTHS["source-dividend-numerator-bits"], max(v.numerator.bit_length() for v in values))
            if role != "Regular":
                micro = values[0]*10**6
                assert micro.denominator == 1
                WIDTHS["published-micro-coefficient-bits"] = max(
                    WIDTHS["published-micro-coefficient-bits"], micro.numerator.bit_length())
            self.roles[role] += 1
            plan = None
            if settlement is not None:
                plan = prepare(defn, model, settlement, bundle_cap)
                plan["settlement"] = settlement
                row.update(ai=fcell(plan["ai"]), ai_bits="%016x" % bits(float(plan["ai"])),
                           next_quasi=str(plan["k"]+1), r=str(plan["r"]), s=str(plan["s"]),
                           n=str(plan["n"]),
                           bundles=",".join("%d@%s@%d/%d" % (i, fcell(v), l, plan["s"])
                                            for i, v, l in plan["bundles"]),
                           entitlements=",".join(plan["entitlements"]))
                self.branches[plan["branch"]] += 1
            if op in ("instrument", "plan"):
                row["expected"] = "ok"
            else:
                if op == "dividend_view":
                    if not 0 <= ordinal < len(values):
                        raise Failure("gilt.invalid_dividend_index", index=ordinal, count=len(values))
                    value = values[ordinal]
                    result = quantize(value, lambda _: source_decimal(defn, fidx, ordinal=ordinal),
                                      quantum, mode, "source dividend", "point")
                    final = admit(result, "source dividend", "point")
                    row.update(expected="some", source_value=fcell(value), lower=final, upper=final)
                else:
                    expected, lo, hi, detail, source = evaluate(
                        defn, model, plan, recipe, ordinal, nominal, evidence, aggregation,
                        quantum, mode, view=op == "instruction_view")
                    row.update(expected=expected, lower=lo, upper=hi,
                               source_value=fcell(source) if source is not None else "-")
                    if expected == "exact_unresolved":
                        row["context"] = context_cell(dict(stage=hx(detail)))
                    else:
                        row["guarantee"] = detail
        except Failure as error:
            row.update(expected=error.kind, context=context_cell(error.context))
            if "cause" in error.context:
                self.causes[error.context["cause"]] += 1
            if "stage" in error.context:
                self.stages[error.context["stage"]] += 1
            if "endpoint" in error.context:
                self.endpoints[error.context["endpoint"]] += 1
        self.profiles[profile] += 1
        self.ops[op] += 1
        self.results[row["expected"]] += 1
        if row["mode"] != "-":
            self.modes[row["mode"]] += 1
            self.scales["quantum"].add(quantum[1])
        if row["nominal"] != "-" and nominal[0] > 0:
            self.scales["nominal"].add(nominal[1])
        if evidence != "-":
            self.evidence_kinds[evidence.split(":")[0]] += 1
        if row["guarantee"] != "-":
            self.guarantees[row["guarantee"]] += 1
        if recipe != "-":
            self.recipes[recipe] += 1
        assert all("\t" not in str(v) and "\n" not in str(v) and str(v) for v in row.values())
        self.rows.append("\t".join(str(row[k]) for k in COLUMNS)+"\n")


def populate(c):
    regular = build()
    short = build(role="Short")
    long = build(role="Long")
    definitions = (regular, short, long)
    # Boundary trajectories exercise all seven AI branches and original indices.
    for d in definitions:
        points = {d.issue, d.issue+timedelta(days=1), d.maturity-timedelta(days=1)}
        for q in d.quasi:
            points.update(q+timedelta(days=t) for t in (-1, 0, 1)
                          if date.min <= q+timedelta(days=t) <= date.max)
        for ex in d.ex:
            points.update(ex+timedelta(days=t) for t in (-1, 0, 1))
        for s in sorted(p for p in points if d.issue <= p < d.maturity):
            c.add("LIFECYCLE-AND-STRICT-AFTER", d, settlement=s)
        for i in range(len(d.ex)):
            c.add("DIVIDEND-PUBLICATION", d, op="dividend_view", recipe=RECIPES[0],
                  ordinal=i, quantum=(1, 6), mode="Exact")

    # Historical long-first inputs, with later M so actual F is a valid reset.
    historical = build(date(2000, 12, 7), 4, "Long", issue_offset=31)
    historical = replace(historical, ex=(date(1999, 11, 26),)+historical.ex[1:])
    for offset in range(1, 215):
        c.add("DMO-LONG-DAILY-SOURCE", historical,
              settlement=historical.issue+timedelta(days=offset))
    # Gregorian cycle coverage: two anchor months, both EOM policies and two
    # days per month across400years; roles rotate instead of a redundant product.
    for year in range(2000, 2400):
        for month in (2, 8):
            for eom in ("Ignore", "Preserve"):
                for day in (monthrange(year, month)[1]-1, monthrange(year, month)[1]):
                    role = ("Regular", "Short", "Long")[(year+month+day+(eom=="Preserve")) % 3]
                    d = build(date(year, month, day), 4, role, eom=eom)
                    s = d.quasi[1] + timedelta(days=(d.quasi[2]-d.quasi[1]).days//2)
                    c.add("GREGORIAN-400Y-DIRECT-EOM", d, settlement=s)
    # Supported civil endpoints and direct-versus-recursive February controls.
    for m in (date(2, 2, 28), date(9999, 12, 31), date(2024, 2, 29), date(2025, 2, 28)):
        for eom in ("Ignore", "Preserve"):
            d = build(m, 2, "Short", eom=eom)
            c.add("CIVIL-ENDS-AND-FEBRUARY", d, settlement=d.issue)

    # Constructor and preparation errors, including deliberate mixed faults.
    for cap in (0, 1, 20001, 2):
        c.add("CONSTRUCTOR-FAILURE-ORDER", replace(regular, cap=cap), op="instrument")
    for q in ((), regular.quasi[:1], regular.quasi[:2]):
        c.add("CONSTRUCTOR-FAILURE-ORDER", replace(regular, quasi=q), op="instrument")
    malformed = [
        replace(regular, coupon=(-1, 0), issue=regular.maturity),
        replace(regular, issue=regular.maturity),
        replace(regular, issue=regular.maturity+timedelta(days=1)),
        replace(regular, quasi=regular.quasi[:-1]+(regular.maturity-timedelta(days=1),)),
        replace(regular, quasi=regular.quasi[:2]+(regular.quasi[1],)+regular.quasi[3:]),
        replace(regular, issue=regular.quasi[0]-timedelta(days=1)),
        replace(regular, issue=regular.quasi[1]),
        replace(regular, quasi=(regular.quasi[0]+timedelta(days=1),)+regular.quasi[1:],
                issue=regular.quasi[0]+timedelta(days=1)),
        replace(regular, first=regular.quasi[2]),
        replace(short, first=short.quasi[3]),
        replace(long, ex=long.ex[:-1]),
        replace(short, ex=(short.issue-timedelta(days=1),)+short.ex[1:]),
        replace(long, ex=(long.quasi[1]-timedelta(days=1),)+long.ex[1:]),
        replace(regular, ex=(regular.first+timedelta(days=1),)+regular.ex[1:]),
        replace(regular, ex=regular.ex[:1]+(regular.quasi[1]-timedelta(days=1),)+regular.ex[2:]),
    ]
    for d in malformed:
        c.add("CONSTRUCTOR-FAILURE-ORDER", d, op="instrument")
    # Date arithmetic fails only after earlier shape/coverage checks succeed.
    d = Definition(date(1, 1, 1), date(1, 9, 7), (6, 0), date(1, 3, 7),
                   (date(1, 1, 1), date(1, 3, 7), date(1, 9, 7)),
                   (date(1, 3, 1), date(1, 9, 1)))
    c.add("DIRECT-CYCLE-DATE-ERROR", d, op="instrument")
    # Recursively clamped schedule is a valid ascending input, deliberately wrong.
    m = date(2025, 8, 31)
    direct = build(m, 4)
    recursive = [m]
    for _ in range(4):
        recursive.append(add_months(recursive[-1], -6, "Ignore"))
    q = tuple(reversed(recursive))
    c.add("RECURSIVE-CLAMP-REJECTED", replace(direct, quasi=q, issue=q[0]), op="instrument")
    for d in definitions:
        for s, cap in ((d.issue, 0), (d.issue, 1), (d.issue, 20001),
                       (d.issue-timedelta(days=1), 0), (d.maturity, 0),
                       (d.maturity+timedelta(days=1), 20000)):
            c.add("PLAN-FAILURE-ORDER", d, settlement=s, bundle_cap=cap)
        needed = len(prepare(d, validate(d), d.issue, 20000)["bundles"])
        for cap in (needed-1, needed, needed+1):
            c.add("EXACT-BUNDLE-CAP", d, settlement=d.issue, bundle_cap=cap)

    # All39 coupon/nominal/quantum scales and full coefficients, rotating roles.
    for scale in range(39):
        for coefficient in (0, 1, 3, MAX):
            d = build(role=("Regular", "Short", "Long")[(scale+coefficient)%3],
                      coupon=(coefficient, scale))
            s = d.first-timedelta(days=1)
            c.add("COUPON-39-SCALES-FULL-RANGE", d, settlement=s)
            for mode in MODES:
                c.add("SOURCE-VIEWS-39-SCALES", d, op="dividend_view", recipe=RECIPES[0],
                      quantum=(1, (scale+13)%39), mode=mode)
                recipe = RECIPES[(scale+coefficient)%4]
                c.add("CASH-39-SCALES-FULL-RANGE", d, op="cash", settlement=s,
                      recipe=recipe, nominal=(MAX if coefficient == MAX else 1+scale, (17*scale)%39),
                      evidence=dc((-1 if scale%2 else 1)*(10**min(scale, 37)+1), scale)
                      if recipe in RECIPES[2:] else "-",
                      aggregation=AGGS[scale%2] if recipe == RECIPES[3] else "-",
                      quantum=(MAX if coefficient == MAX else 1+2*(scale%5), (23*scale)%39), mode=mode)

    # Exact half-up first publication ties; source regular odd scale38 stays raw.
    for role in ("Short", "Long"):
        d0 = build(date(2025, 12, 7), 2, role, issue_offset=91)
        s1, r1 = (d0.quasi[1]-d0.quasi[0]).days, (d0.quasi[1]-d0.issue).days
        p = r1 + (s1 if role == "Long" else 0)
        for k in (0, 1, 2, 4, 7, 9, 10, 999999):
            # C chosen so raw dividend is (k+.5)micro whenever representable.
            candidate = F((2*k+1)*s1, p*10**6)
            try:
                pair = amount_pair(candidate)
            except AssertionError:
                continue
            if pair[1] <= 38 and pair[0] <= MAX:
                d = replace(d0, coupon=pair)
                for mode in MODES:
                    c.add("PUBLICATION-HALFUP-TIES", d, op="cash", recipe=RECIPES[0],
                          nominal=(100000000, 0), quantum=(1, 2), mode=mode)
    tiny_short = build(role="Short", coupon=(1, 38), issue_offset=1)
    c.add("PUBLISHED-ZERO-ACTUAL-COUPON", tiny_short, settlement=tiny_short.issue)
    for coefficient in (1, MAX):
        d = build(coupon=(coefficient, 38))
        for mode in MODES:
            c.add("REGULAR-SCALE39-SOURCE", d, op="dividend_view", recipe=RECIPES[0],
                  quantum=(1, 38), mode=mode)
    # Full source coefficients specifically include a147bit published micro
    # coefficient and a canonical regular half at scale39. Tiny N rescues cash
    # even when a separately requested public micro view cannot be admitted.
    for role in ("Regular", "Short", "Long"):
        for scale in (0, 38):
            for offset in (1, 181):
                d = build(date(2025, 12, 7), 2, role, coupon=(MAX, scale), issue_offset=offset)
                c.add("WIDE-SOURCE-PUBLICATION-AND-REGULAR", d, settlement=d.issue)
                for mode in MODES:
                    c.add("WIDE-SOURCE-PUBLICATION-AND-REGULAR", d, op="dividend_view",
                          recipe=RECIPES[0], quantum=(1, 6), mode=mode)
                    c.add("WIDE-SOURCE-VIEW-RANGE-CASH-RESCUE", d, op="cash",
                          recipe=RECIPES[0], nominal=(1, 38), quantum=(1, 2), mode=mode)

    # Cash ties and interval thresholds using a genuine fixed source AI.
    zero = build(coupon=(0, 0))
    for evidence in (dc(1005, 3), bc(bits(1.005)), di(1005, 3, 1005),
                     bi(bits(1.005), bits(1.005)), dc(-1005, 3), bc(bits(-1.005)),
                     di(-1006, 3, -1004), di(1004, 3, 1006), di(1000, 3, 1004),
                     di(-1, 4, 1), bc(0), bc(1 << 63), bi((1 << 63)+1, 1),
                     bi(1 << 63, 0), bi(0xffefffffffffffff, 0x7fefffffffffffff)):
        for mode in MODES:
            for recipe in RECIPES[2:]:
                for agg in (AGGS if recipe == RECIPES[3] else ("-",)):
                    c.add("EVIDENCE-TIES-THRESHOLD-PROVENANCE", zero, op="cash", settlement=zero.issue,
                          recipe=recipe, evidence=evidence, aggregation=agg, mode=mode)
    for sign in (1, -1):
        for k in (0, 1, 2, 3):
            for delta in (-1, 0, 1):
                for q in ((1, 2), (5, 2), (25, 3), (7, 0)):
                    value = sign*(F(2*k+1, 2)*amount(q)+F(delta, 10**12))
                    p = amount_pair(value)
                    for mode in MODES:
                        c.add("SIGNED-ARBITRARY-QUANTUM-PARITY", zero, op="cash", settlement=zero.issue,
                              recipe=RECIPES[2], evidence=dc(*p), quantum=q, mode=mode)

    # Component staging, exact failure priority and actual-source wide rescue.
    midpoint = build(date(2025, 6, 7), 1, coupon=(2, 0))
    mid = midpoint.issue + timedelta(days=91)
    for exmode in (False, True):
        d = replace(midpoint, ex=(midpoint.issue if exmode else midpoint.first,))
        for evidence in (dc(5, 1), dc(-5, 1), di(0, 0, 1), dc(100005, 3)):
            for agg in AGGS:
                for mode in MODES:
                    c.add("AGGREGATION-EXACT-PRECEDENCE", d, op="cash", settlement=mid,
                          recipe=RECIPES[3], evidence=evidence, aggregation=agg,
                          quantum=(1, 0), mode=mode)
    huge = replace(midpoint, coupon=(2**126, 0), ex=(midpoint.issue,))
    for agg in AGGS:
        for mode in MODES:
            c.add("VALID-SOURCE-WIDE-COMPONENT-CANCELLATION", huge, op="cash", settlement=mid,
                  recipe=RECIPES[3], evidence=bc(bits(float(2**124))), aggregation=agg,
                  nominal=(1000, 0), quantum=(1, 0), mode=mode)
    for nominal in (799, 800, 801):
        for mode in MODES:
            c.add("SOURCE-AI-NEGATIVE-I128-BOUNDARY", huge, op="cash", settlement=mid,
                  recipe=RECIPES[1], nominal=(nominal, 0), quantum=(1, 0), mode=mode)
    for mode in MODES:
        c.add("SOURCE-AI-NONDECIMAL-SCALING-RESCUE", historical, op="cash",
              settlement=date(1999, 12, 6), recipe=RECIPES[1], nominal=(6100, 0),
              quantum=(1, 0), mode=mode)
        c.add("SOURCE-VIEW-INEXACT-CASH-SUCCESS", historical, op="instruction_view",
              settlement=date(1999, 12, 6), recipe=RECIPES[1], nominal=(6100, 0),
              quantum=(1, 0), mode=mode)
    for recipe in RECIPES:
        for mode in MODES:
            c.add("ALL-SOURCE-INSTRUCTION-VIEWS", long, op="instruction_view",
                  settlement=long.first-timedelta(days=1), recipe=recipe,
                  evidence=dc(1) if recipe in RECIPES[2:] else "-",
                  aggregation=AGGS[0] if recipe == RECIPES[3] else "-", mode=mode)
    for ordinal in (len(regular.ex), 2**32-1):
        c.add("DIVIDEND-INDEX-BEFORE-NOMINAL", regular, op="cash", recipe=RECIPES[0],
              ordinal=ordinal, nominal=(0, 0))
        c.add("DIVIDEND-VIEW-INVALID-INDEX", regular, op="dividend_view", recipe=RECIPES[0],
              ordinal=ordinal)
    for nominal in ((0, 0), (-1, 0), (MIN, 0), (-1, 38)):
        for recipe in RECIPES:
            c.add("NOMINAL-BEFORE-ZERO-SHORTCUT", zero, op="cash",
                  settlement=zero.issue if recipe != RECIPES[0] else None,
                  recipe=recipe, nominal=nominal,
                  evidence=dc(0) if recipe in RECIPES[2:] else "-",
                  aggregation=AGGS[0] if recipe == RECIPES[3] else "-", mode="Exact")

    # Exact continuous binary interval has multiple endpoints, undefined middle.
    b100 = bits(float(2**100))
    for ev in (bi(b100, b100+1), bi(b100, b100), bc(b100)):
        for recipe in RECIPES[2:]:
            for agg in (AGGS if recipe == RECIPES[3] else ("-",)):
                c.add("ADJACENT-BINARY-CONTINUOUS-EXACT", zero, op="cash", settlement=zero.issue,
                      recipe=recipe, evidence=ev, aggregation=agg, quantum=(2**48, 0), mode="Exact")
    # Full finite binary and i128 limits: range labels, normalization and signed limits.
    endpoints = (bc(1), bc((1 << 63)+1), bc(0x7fefffffffffffff), bc(0xffefffffffffffff),
                 bi(0x7feffffffffffffe, 0x7fefffffffffffff),
                 bi(0xffefffffffffffff, 0xffeffffffffffffe),
                 dc(MAX), dc(MIN), di(MAX-1, 0, MAX), di(MIN, 0, MIN+1))
    for i, ev in enumerate(endpoints):
        for nominal in ((1, 38), (100, 0), (MAX, 0), (MAX, 38)):
            for q in ((1, 38), (1, 0), (MAX, 0), (MAX, 38)):
                for mode in MODES:
                    recipe = RECIPES[2+i%2]
                    c.add("FINITE-BINARY-SIGNED-LIMITS-WIDE", historical, op="cash",
                          settlement=date(1999, 12, 6), recipe=recipe, nominal=nominal,
                          evidence=ev, aggregation=AGGS[(i+nominal[1])%2] if recipe == RECIPES[3] else "-",
                          quantum=q, mode=mode)
    for ev in (dc(MAX), dc(MIN), bc(bits(float(2**127))), bc(bits(-float(2**127))),
               bc(bits(float(2**127))-1), bc(bits(-float(2**127))+1)):
        for q in ((1, 0), (1, 38), (10, 0), (MAX, 38)):
            for mode in MODES:
                c.add("EXPLICIT-SIGNED-I128-ADMISSION", zero, op="cash", settlement=zero.issue,
                      recipe=RECIPES[2], evidence=ev, nominal=(100, 0), quantum=q, mode=mode)
    # Both extrema are representable although interior count2 has an i128
    # precision hole. QuantumAmbiguous certifies only the retained extrema.
    for mode in MODES:
        c.add("REPRESENTABLE-EXTREMA-INTERIOR-PRECISION-HOLE", zero, op="cash",
              settlement=zero.issue, recipe=RECIPES[2], evidence=di(MAX, 38, MAX, 37),
              quantum=(MAX, 38), mode=mode)
    # Nonpoint quantizer collapse outside range reports lower; component net point.
    for ev in (di(10**38+1, 36, 10**38+2), di(-10**38-2, 36, -10**38-1)):
        for recipe in RECIPES[2:]:
            for agg in (AGGS if recipe == RECIPES[3] else ("-",)):
                for mode in MODES:
                    c.add("STAGE-POINTNESS-AFTER-COLLAPSE", zero, op="cash", settlement=zero.issue,
                          recipe=recipe, evidence=ev, nominal=(MAX, 0), aggregation=agg,
                          quantum=(10, 0), mode=mode)

    # Deterministic independent rotating dimensions; no arbitrary full product.
    state = 0x47494c54534f5552434543415348
    def next_int():
        nonlocal state
        state = (state*6364136223846793005+1442695040888963407) & (2**128-1)
        return state
    for i in range(1800):
        role = ("Regular", "Short", "Long")[i%3]
        d = build(role=role, coupon=(next_int()%MAX, i%39), issue_offset=1+i%170)
        s = d.issue+timedelta(days=next_int()%(d.maturity-d.issue).days)
        recipe = RECIPES[i%4]
        coefficient = next_int()%MAX * (-1 if i%2 else 1)
        ev = dc(coefficient, (i*11)%39)
        if i%5 == 0:
            ev = di(min(coefficient, -coefficient), (i*11)%39, max(coefficient, -coefficient))
        if i%11 == 0:
            raw = next_int() & (2**64-1)
            raw = (raw & ~(2047 << 52)) | (((raw >> 52)%2047) << 52)
            ev = bc(raw)
        c.add("ROTATING-EXACT-SOURCE-CASH", d, op="cash", settlement=s,
              recipe=recipe, nominal=(1+next_int()%MAX, (i*17)%39),
              evidence=ev if recipe in RECIPES[2:] else "-",
              aggregation=AGGS[(i//4)%2] if recipe == RECIPES[3] else "-",
              quantum=(1+next_int()%MAX, (i*23)%39), mode=MODES[(i//8)%7])


def counter_text(counter):
    return ";".join("%s=%d" % item for item in sorted(counter.items()))


def generate(generated_at, platform_text):
    contract = CONTRACT.read_bytes()
    assert sha(contract) == CONTRACT_SHA, "frozen gilt contract changed"
    req = "\n".join(line for line in (ROOT / "docs/REQUIREMENTS.md").read_text().splitlines()
                    if line.startswith("| FR-GILT-")).encode()
    assert len(req.splitlines()) == 24 and sha(req) == REQ_SHA, "frozen FR-GILT rows changed"
    CHECKS.clear()
    WIDTHS.clear()
    corpus = Corpus()
    populate(corpus)
    assert set(corpus.branches) == {"short-cum", "short-ex", "long-first", "long-second-cum",
                                    "long-second-ex", "regular-cum", "regular-ex"}
    assert set(corpus.modes) == set(MODES)
    assert all(scales == set(range(39)) for scales in corpus.scales.values())
    assert set(corpus.evidence_kinds) == {"d", "di", "b", "bi"}
    assert WIDTHS["published-micro-coefficient-bits"] == 147
    source = Path(__file__).read_bytes()
    records = "".join(corpus.rows).encode()
    body = "\t".join(COLUMNS).encode()+b"\n"+records
    headers = [
        ("schema", "1"), ("fixture-set", "CONVENTIONAL-GILT-SOURCE-DATE-CASH-ORACLE"),
        ("domain", "bonds"),
        ("description", "Independent exact conventional-gilt dates, source publication/accrual and cash decisions"),
        ("reference", "Python stdlib integer/Fraction with separately decoded unreduced Decimal rational expressions"),
        ("reference-version", "Python >=3.9; Decimal1536,2048"),
        ("reference-source", "Frozen conventional-gilt contract S,A-J,C1-C8; FR-GILT-01..06,09,15..22; no Rust financial or QuantLib implementation read"),
        ("tolerance", "TOL-E0 exact canonical amounts, rational metadata, binary AI bits and structured decisions/errors"),
        ("reproduction", "python3 -I validation/generator/gilt_source_cash_oracles.py --check"),
        ("generated-at", generated_at), ("platform", platform_text),
        ("generator-sha256", sha(source)), ("generator-fnv64", fnv(source)),
        ("adopted-contract-sha256", sha(contract)), ("adopted-contract-fnv64", fnv(contract)),
        ("requirements-rows-sha256", sha(req)), ("requirements-rows-fnv64", fnv(req)),
        ("source-model", "UK-DMO-Conventional-v4-2024-12-18"),
        ("encoding", "TSV; decimal=C:S; rational=N/D; dates=ISO; empty=empty; -=unused; evidence=d:C:S,di:C:S:C:S,b:16hex,bi:16hex:16hex; context=key=value,x:UTF8hex"),
        ("record-count", str(len(corpus.rows))), ("unique-input-count", str(len(corpus.seen))),
        ("profile-counts", counter_text(corpus.profiles)), ("operation-counts", counter_text(corpus.ops)),
        ("expectation-counts", counter_text(corpus.results)), ("first-period-counts", counter_text(corpus.roles)),
        ("ai-branch-counts", counter_text(corpus.branches)), ("cash-mode-counts", counter_text(corpus.modes)),
        ("recipe-counts", counter_text(corpus.recipes)), ("cross-check-counts", counter_text(CHECKS)),
        ("nested-cause-counts", counter_text(corpus.causes)),
        ("arithmetic-stage-counts", counter_text(corpus.stages)),
        ("arithmetic-endpoint-counts", counter_text(corpus.endpoints)),
        ("price-guarantee-counts", counter_text(corpus.guarantees)),
        ("evidence-kind-counts", counter_text(corpus.evidence_kinds)),
        ("scale-census", ";".join(k+"="+",".join(map(str, sorted(v)))
                                  for k, v in sorted(corpus.scales.items()))),
        ("maximum-observed-widths", counter_text(WIDTHS)),
        ("records-sha256", sha(records)), ("records-fnv64", fnv(records)),
        ("body-sha256", sha(body)), ("body-fnv64", fnv(body)),
    ]
    data = "".join("%s: %s\n" % pair for pair in headers).encode()+b"\n"+body
    return data, dict(headers)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Regenerate and compare every byte")
    parser.add_argument("--output", type=Path, default=DEST)
    args = parser.parse_args()
    timestamp = datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    platform_text = platform.system()+" "+platform.machine()+"; Python "+platform.python_version()
    if args.check and args.output.is_file():
        header = args.output.read_text().split("\n\n", 1)[0]
        metadata = dict(line.split(": ", 1) for line in header.splitlines())
        timestamp, platform_text = metadata["generated-at"], metadata["platform"]
    data, summary = generate(timestamp, platform_text)
    if args.check:
        if not args.output.is_file() or args.output.read_bytes() != data:
            raise SystemExit("gilt source/cash fixture differs from independent regeneration")
        summary["check"] = "identical"
    else:
        if args.output.exists():
            raise SystemExit("refusing to overwrite existing evidence; use --check or a new --output")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(data)
        summary["written"] = str(args.output)
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
