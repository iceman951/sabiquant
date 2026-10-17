#!/usr/bin/env python3
"""Independent contextual bond corpus: exact dates/Fraction and Decimal factors.

No Rust or QuantLib production/output is imported. Raw-rational financial
targets and enclosures for the documented staged binary64 calculation differ.
"""
import argparse
import calendar
import collections
from datetime import date, timedelta
from decimal import Decimal, localcontext
from fractions import Fraction as F
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / 'validation/fixtures/bonds/contextual-oracles.fixture'
PRECISIONS = (160, 240)
SIGN, INF = 1 << 63, 0x7ff0000000000000
U, ETA = F(1, 1 << 53), F(1, 1 << 1074)
FREQUENCIES = (1, 2, 3, 4, 6, 12, 13, 26, 52, 365)
COMPS = ('Simple', 'Continuous') + tuple(f'{method}:{f}' for method in ('Compounded', 'SimpleThenCompounded', 'CompoundedThenSimple') for f in FREQUENCIES)
CONTRACTS = ('docs/M8_BOND_CORE_CONTRACT.md', 'docs/M3_ICMA_CONTRACT.md', 'docs/M3_US_AFB_CONTRACT.md', 'docs/M3_TARGET_CONTRACT.md', 'docs/M3_DAYCOUNT_EXTENSIONS.md', 'docs/M4_DATE_CONTRACT.md', 'docs/M7_CASHFLOW_CONTRACT.md')
COLUMNS = ('id profile issue raw_dates pay_dates redemption_date redemption_bits ex_terms face_bits coupon_rate_bits coupon_context yield_context settlement inclusion comp yield_bits work_cap sched_tenor sched_rule sched_first sched_penultimate sched_eom sched_bdc sched_term_bdc sched_holidays sched_business sched_max expected error_index error_phase error_kind error_date error_max error_used error_required entitlements redemption_entitled work full_queries plan_queries accrued_lo_bits accrued_hi_bits dirty_lo_bits dirty_hi_bits clean_lo_bits clean_hi_bits raw_ideal_dirty_bits raw_ideal_accrued_bits raw_ideal_clean_bits raw_ideal_dirty_error_bits raw_ideal_accrued_error_bits raw_ideal_clean_error_bits').split()


def sha(b): return hashlib.sha256(b).hexdigest()
def fnv(b):
    h = 0xcbf29ce484222325
    for x in b: h = ((h ^ x) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{h:016x}'
def hx(b): return f'0x{b:016x}'
def input_bits(x): return struct.unpack('>Q', struct.pack('>d', x))[0]
def binary(b):
    s, e, m = bool(b & SIGN), (b >> 52) & 2047, b & ((1 << 52) - 1)
    assert e != 2047
    if e: m += 1 << 52; p = e - 1075
    else: p = -1074
    x = F(m << p) if p >= 0 else F(m, 1 << -p)
    return -x if s else x
def pow2(p): return F(1 << p) if p >= 0 else F(1, 1 << -p)
def rn(x):
    if not x: return 0
    neg, a = x < 0, abs(x)
    top = a.numerator.bit_length() - a.denominator.bit_length()
    if a < pow2(top): top -= 1
    unit = max(-1074, top - 52); z = a / pow2(unit)
    q, r = divmod(z.numerator, z.denominator)
    if 2*r > z.denominator or (2*r == z.denominator and q % 2): q += 1
    assert q and q.bit_length()-1+unit <= 1023
    top = q.bit_length()-1+unit
    if top < -1022: bits = q << (unit + 1074)
    else:
        k = q.bit_length()-53
        m = q >> k if k >= 0 else q << -k
        bits = ((top+1023) << 52) | (m-(1 << 52))
    return bits | (SIGN if neg else 0)
def rounded(x): return binary(rn(x))
def next_bits(b, direction):
    if not b: return 1 if direction > 0 else SIGN | 1
    if b == SIGN: return 1 if direction > 0 else SIGN | 1
    return b + (direction if not b & SIGN else -direction)
def outward(x, direction):
    b = rn(F(x)) if x else 0
    if (direction < 0 and binary(b) > x) or (direction > 0 and binary(b) < x): b = next_bits(b, direction)
    return binary(b)
def rational(x): return f'{x.numerator}/{x.denominator}'
def dec(x): return Decimal(x.numerator) / Decimal(x.denominator)
def dates(xs): return ','.join(str(x) for x in xs) or '-'
def parse_dates(s): return [] if s == '-' else [date.fromisoformat(x) for x in s.split(',')]


class Coverage(Exception):
    def __init__(self, kind, rejected): self.kind, self.rejected = kind, rejected


@lru_cache(None)
def fraction(ctx, start, end):
    assert start <= end
    periods = []
    if ctx.startswith('ICMA:'):
        _, f, bs = ctx.split(':'); bs = parse_dates(bs)
        if start < bs[0] or end > bs[-1]: raise Coverage('icma.out_of_coverage', None)
        periods = [F((min(end, b)-max(start, a)).days, int(f)*(b-a).days) for a, b in zip(bs, bs[1:]) if max(start, a) < min(end, b)]
        exact = sum(periods, F(0)); tol = 4 if exact else 0
    elif ctx == 'A365F': exact, tol = F((end-start).days, 365), 0
    elif ctx.startswith('B252:'):
        spec = ctx.split(':'); tol = 0
        if spec[1] == 'T':
            for d in (start, end):
                if d < date(1999, 1, 4): raise Coverage('calendar.outside_validity', d)
                if not (d <= date(2002, 12, 31) or date(2026, 1, 1) <= d <= date(2028, 12, 31)): raise Coverage('calendar.unverified_coverage', d)
            if start <= date(2002, 12, 31) and end >= date(2026, 1, 1): raise Coverage('calendar.unverified_coverage', date(2003, 1, 1))
            mask, holidays, business = 96, set(), set()
            easter = {2026: (date(2026, 4, 3), date(2026, 4, 6)), 2027: (date(2027, 3, 26), date(2027, 3, 29)), 2028: (date(2028, 4, 14), date(2028, 4, 17))}
            for year in range(start.year, end.year+1):
                holidays.update(date(year, m, d) for m, d in ((1, 1), (5, 1), (12, 25), (12, 26)))
                if year in easter: holidays.update(easter[year])
        else:
            _, _, mask, hs, bs = spec; mask = int(mask)
            holidays, business = set(parse_dates(hs)), set(parse_dates(bs))
        count = 0
        for i in range((end-start).days):
            d = start+timedelta(days=i)
            count += d in business or (d not in holidays and not mask & (1 << d.weekday()))
        exact = F(count, 252)
    else:
        # The adopted signed-forward contract defines equal synthetic dates
        # as zero BEFORE ISDA's final-February termination adjustment. ICMA
        # and guarded Business/252 reach this point only in their own branches
        # above, so their mandatory coverage checks are never bypassed.
        if ctx.startswith(('ISDA:', 'US:')) and start == end:
            return F(0), 0, (F(0),)
        d1, d2 = start.day, end.day
        f1 = start.month == 2 and d1 == calendar.monthrange(start.year, 2)[1]
        f2 = end.month == 2 and d2 == calendar.monthrange(end.year, 2)[1]
        if ctx.startswith('ISDA:'):
            term = date.fromisoformat(ctx.split(':')[1])
            if d1 == 31 or f1: d1 = 30
            if d2 == 31 or (f2 and end != term): d2 = 30
        elif ctx.startswith('US:'):
            flag = ctx == 'US:1'
            if flag and f1 and f2: d2 = 30
            if flag and f1: d1 = 30
            if d2 == 31 and d1 in (30, 31): d2 = 30
            if d1 == 31: d1 = 30
        else: raise AssertionError(ctx)
        exact, tol = F(360*(end.year-start.year)+30*(end.month-start.month)+d2-d1, 360), 0
    assert exact >= 0, (ctx, start, end, exact)
    if not periods: periods = [exact]
    return exact, tol, tuple(periods)


def work(ctx, start, end):
    if ctx.startswith('B252:'): return 1+(end-start).days
    if ctx.startswith('ICMA:'):
        bs = parse_dates(ctx.split(':')[2])
        return 1+sum(max(start, a) < min(end, b) for a, b in zip(bs, bs[1:]))
    return 1


def possibilities(exact, tol):
    b = rn(exact)
    return tuple(binary(k) for k in range(b-tol, b+tol+1)) if exact else (F(0),)


def query_cell(role, start, end, exact, tol, parts, cost, products=None, status='ok'):
    return '~'.join((role, str(start), str(end), rational(exact) if exact is not None else '-', hx(rn(exact)) if exact is not None else '-', str(tol) if exact is not None else '-', str(cost), '+'.join(map(rational, parts)) if parts else '-', ','.join(f'{hx(rn(y))}={hx(rn(a))}' for y, a in products) if products else '-', status))


@lru_cache(None)
def product_map(face, rate, exact, tol):
    result = []
    for y in possibilities(exact, tol):
        product = face*rate*y
        # These bounded corpus dyadics terminate within160 digits. Check the
        # actual product value as well as its rounded bits at two precisions.
        for precision in PRECISIONS:
            with localcontext() as c:
                c.prec = precision
                independent = dec(face)*dec(rate)*dec(y)
                assert F(independent) == product
                assert rn(F(independent)) == rn(product)
        result.append((y, rounded(product)))
    return tuple(result)
def span(xs):
    xs = tuple(xs)
    return min(xs), max(xs)
def add_spans(xs): return rounded(sum((x[0] for x in xs), F(0))), rounded(sum((x[1] for x in xs), F(0)))
def subtract(a, b): return rounded(a[0]-b[1]), rounded(a[1]-b[0])


@lru_cache(None)
def factor(rate, time, comp, precision):
    with localcontext() as c:
        c.prec = precision
        method, _, f = comp.partition(':'); f = int(f) if f else 0
        if f: assert rate > -f
        if method in ('SimpleThenCompounded', 'CompoundedThenSimple'):
            simple = time <= rounded(F(1, f)) if method == 'SimpleThenCompounded' else time > rounded(F(1, f))
            method = 'Simple' if simple else 'Compounded'
        if method == 'Simple':
            compound = 1+rate*time; assert compound > 0
            d = rounded(1/rounded(compound))
            ideal = Decimal(1)/dec(compound)
            return d, d, ideal
        log = dec(rate)*dec(time) if method == 'Continuous' else Decimal(f)*dec(time)*(1+dec(rate)/f).ln()
        ideal = (-log).exp()
        if rate == 0 or time == 0: return F(1), F(1), ideal
        allowance = 8*dec(U)*(1+abs(log))*ideal+dec(ETA)
        return outward(F(ideal-allowance), -1), outward(F(ideal+allowance), 1), ideal


def ideal_discount(rate, exact_time, comp, precision):
    with localcontext() as c:
        c.prec = precision
        method, _, f = comp.partition(':'); f = int(f) if f else 0
        if method in ('SimpleThenCompounded', 'CompoundedThenSimple'):
            # Raw ideal retains the declared operational rounded switch, while
            # all fixed-branch formula arguments remain exact raw rationals.
            simple = exact_time <= rounded(F(1, f)) if method == 'SimpleThenCompounded' else exact_time > rounded(F(1, f))
            method = 'Simple' if simple else 'Compounded'
        if method == 'Simple': return Decimal(1)/(1+dec(rate)*dec(exact_time))
        log = dec(rate)*dec(exact_time) if method == 'Continuous' else Decimal(f)*dec(exact_time)*(1+dec(rate)/f).ln()
        return (-log).exp()


def active_simple(time, comp):
    method, _, frequency = comp.partition(':')
    if method == 'Simple': return True
    if method == 'SimpleThenCompounded': return time <= rounded(F(1, int(frequency)))
    if method == 'CompoundedThenSimple': return time > rounded(F(1, int(frequency)))
    return False


def evaluate(row, precision):
    raw, pays = parse_dates(row['raw_dates']), parse_dates(row['pay_dates'])
    settlement = date.fromisoformat(row['settlement']); face = binary(int(row['face_bits'], 16)); rate = binary(int(row['coupon_rate_bits'], 16)); yr = binary(int(row['yield_bits'], 16))
    full, full_ideal, full_queries = [], [], []
    for i, (a, b) in enumerate(zip(raw, raw[1:])):
        x, tol, parts = fraction(row['coupon_context'], a, b)
        pm = product_map(face, rate, x, tol); full.append(span(a for _, a in pm)); full_ideal.append(face*rate*x)
        full_queries.append(query_cell(f'C{i}', a, b, x, tol, parts, 0, pm))
    plan_queries, used, accrual, raw_accrual, flows, ents = [], 0, [], F(0), [], []
    def query(role, a, b, ctx, phase, index, product=False):
        nonlocal used
        cost = work(ctx, a, b); maximum = int(row['work_cap'])
        if used+cost > maximum:
            plan_queries.append(query_cell(role, a, b, None, None, (), cost, status='work'))
            raise Stop('work', index, phase, 'bond.accrual_work_limit_exceeded', '-', maximum, used, used+cost)
        used += cost
        try: x, tol, parts = fraction(ctx, a, b)
        except Coverage as e:
            plan_queries.append(query_cell(role, a, b, None, None, (), cost, status='coverage'))
            raise Stop('icma' if e.kind.startswith('icma.') else 'calendar', index, phase, e.kind, str(e.rejected) if e.rejected else '-', '-', '-', '-') from None
        pm = product_map(face, rate, x, tol) if product else None
        plan_queries.append(query_cell(role, a, b, x, tol, parts, cost, pm))
        return x, tol, pm
    def eligible(d): return d > settlement or (d == settlement and row['inclusion'] == 'Include')
    try:
        query('A', settlement, settlement, row['yield_context'], 'discount anchor', 'none')
        for i, (a, b, p, ex) in enumerate(zip(raw, raw[1:], pays[1:], row['ex_terms'].split(';'))):
            if not eligible(p): ents.append('C'); continue
            boundary, cutoff = ex.split(':'); cutoff = date.fromisoformat(cutoff)
            detached = settlement >= cutoff if boundary == 'OnOrAfter' else settlement > cutoff
            ents.append('X' if detached else 'B')
            if settlement <= a: earned, raw_earned = (F(0), F(0)), F(0)
            elif settlement >= b: earned, raw_earned = full[i], full_ideal[i]
            else:
                x, tol, pm = query(f'P{i}', a, settlement, row['coupon_context'], 'partial accrual', str(i), True)
                earned, raw_earned = span(v for _, v in pm), face*rate*x
            accrual.append(subtract(earned, full[i]) if detached else earned)
            raw_accrual += raw_earned-full_ideal[i] if detached else raw_earned
            if not detached:
                x, tol, _ = query(f'H{i}', settlement, p, row['yield_context'], 'discount horizon', str(i))
                flows.append((full[i], possibilities(x, tol), full_ideal[i], x))
        redemption_entitled = eligible(date.fromisoformat(row['redemption_date']))
        if redemption_entitled:
            amount = binary(int(row['redemption_bits'], 16)); p = date.fromisoformat(row['redemption_date'])
            x, tol, _ = query('R2', settlement, p, row['yield_context'], 'redemption discount horizon', '2')
            flows.append(((amount, amount), possibilities(x, tol), amount, x))
        with localcontext() as c:
            c.prec = precision
            pvs, ideal_dirty = [], Decimal(0)
            raw_by_time = collections.defaultdict(F)
            for amount, times, raw_amount, raw_time in flows:
                ds = [factor(yr, t, row['comp'], precision) for t in times]
                dlo, dhi = min(x[0] for x in ds), max(x[1] for x in ds)
                low, high = span(a*d for a in amount for d in (dlo, dhi))
                pvs.append((rounded(low), rounded(high)))
                raw_by_time[raw_time] += raw_amount
            # The ideal is an exact mathematical sum, with equal raw horizons
            # combined before elementary evaluation to retain cancellations.
            for raw_time, raw_amount in raw_by_time.items():
                ideal_dirty += dec(raw_amount)*ideal_discount(yr, raw_time, row['comp'], precision)
            accrued, dirty = add_spans(accrual), add_spans(pvs); clean = subtract(dirty, accrued)
            ideals = (ideal_dirty, dec(raw_accrual), ideal_dirty-dec(raw_accrual))
            assert rn(F(ideals[1])) == rn(raw_accrual)
            if all(active_simple(t, row['comp']) for t in raw_by_time):
                rational_dirty = sum((a/(1+yr*t) for t, a in raw_by_time.items()), F(0))
                assert rn(F(ideals[0])) == rn(rational_dirty)
                assert rn(F(ideals[2])) == rn(rational_dirty-raw_accrual)
            numeric = [hx(rn(x)) for pair in (accrued, dirty, clean) for x in pair]+[hx(rn(F(x))) if x else hx(0) for x in ideals]
            # Distance to the SERIALIZED nearest-binary64 raw-ideal anchor,
            # not a claim that the raw ideal equals a staged binary result.
            for ideal, bounds in zip(ideals, (dirty, accrued, clean)):
                center = rounded(F(ideal)) if ideal else F(0)
                error = max(abs(endpoint-center) for endpoint in bounds)
                numeric.append(hx(rn(outward(error, 1))) if error else hx(0))
        result = ['ok']+['-']*7+[''.join(ents), str(int(redemption_entitled)), str(used), ';'.join(full_queries), ';'.join(plan_queries)]+numeric
    except Stop as e:
        result = list(map(str, e.args))+['-', '-', str(used), ';'.join(full_queries), ';'.join(plan_queries)]+['-']*12
    return result


class Stop(Exception): pass


def self_checks():
    count = 0
    for year in (1900, 2000, 2028):
        d = date(year, 2, calendar.monthrange(year, 2)[1])
        for ctx in (f'ISDA:{d}', 'US:0', 'US:1'):
            assert fraction(ctx, d, d) == (F(0), 0, (F(0),))
            count += 1
    for ctx, d, kind in (
        ('B252:T', date(2025, 12, 31), 'calendar.unverified_coverage'),
        ('B252:T', date(1998, 12, 31), 'calendar.outside_validity'),
        ('ICMA:2:2027-08-31,2028-02-29', date(2027, 8, 30), 'icma.out_of_coverage'),
    ):
        try: fraction(ctx, d, d)
        except Coverage as e: assert e.kind == kind
        else: raise AssertionError('equal dates must not waive coverage')
        count += 1
    return count


def context_icma(boundaries, f=2): return f'ICMA:{f}:{",".join(boundaries)}'


def profiles():
    base = ['2027-08-31', '2028-02-29', '2028-08-31']
    refs = ['2027-02-28', '2027-08-31', '2028-02-29', '2028-08-31', '2029-02-28']
    icma = context_icma(refs)
    custom = 'B252:C:48:2027-09-06,2028-03-01:2027-09-03,2028-03-04'
    custom2 = 'B252:C:65:2028-03-01:2028-03-05'
    out = []
    for name, raw in (('REGULAR', base), ('SHORT-FIRST', ['2027-11-15']+base[1:]), ('LONG-FIRST', ['2027-04-15']+base[1:]), ('SHORT-FINAL', base[:2]+['2028-05-15']), ('LONG-FINAL', base[:2]+['2029-01-15'])):
        out.append(('ICMA-'+name, raw, icma, icma, 0, 0))
    out += [
        ('ISDA-FINAL-FEB', ['2027-02-28', '2027-08-31', '2028-02-29'], 'ISDA:2028-02-29', 'ISDA:2028-02-29', 0, 0),
        ('ISDA-INTERMEDIATE-FEB', base, 'ISDA:2028-08-31', 'ISDA:2028-08-31', 0, 0),
        ('US-EOM-OFF', ['2027-02-28', '2027-08-31', '2028-02-29'], 'US:0', 'US:0', 0, 0),
        ('US-EOM-ON', ['2027-02-28', '2027-08-31', '2028-02-29'], 'US:1', 'US:1', 0, 0),
        ('B252-FRI-SAT', base, custom, custom, 0, 0),
        ('B252-SUN-MON', base, custom2, custom2, 0, 0),
        ('TARGET-VERIFIED', base, 'B252:T', 'B252:T', 0, 0),
        ('ICMA-ISDA-DELAYED', base, icma, 'ISDA:2028-08-31', 3, 0),
        ('US-B252-EARLY', base, 'US:1', custom, -2, 0),
        ('ISDA-ICMA-REOPENING', base, 'ISDA:2028-08-31', icma, 2, 14),
        ('B252-US-DELAYED', base, custom, 'US:0', 4, 0),
    ]
    return out


def make_rows():
    rows = []
    for pi, (profile, raw_s, cc, yc, lag, reopen) in enumerate(profiles()):
        raw = list(map(date.fromisoformat, raw_s)); pays = [raw[0], raw[1]+timedelta(days=lag), raw[2]+timedelta(days=2 if lag else 0)]
        issue = raw[0]+timedelta(days=reopen)
        cutoff = pays[1]-timedelta(days=5)
        settlements = sorted({issue, cutoff-timedelta(days=1), cutoff, cutoff+timedelta(days=1), raw[1], pays[1], raw[1]+timedelta(days=1), pays[2]-timedelta(days=5), pays[2]-timedelta(days=1), pays[2], pays[2]+timedelta(days=1)})
        holidays = set()
        for a, p in zip(raw[1:], pays[1:]):
            if p > a: holidays.update(a+timedelta(days=k) for k in range((p-a).days))
            elif p < a: holidays.update(p+timedelta(days=k) for k in range(1, (a-p).days+1))
        for si, settlement in enumerate(settlements):
            for inclusion in ('Include', 'Exclude'):
                for ci, comp in enumerate(COMPS):
                    face = (100.0, 1000.0)[ci % 2]
                    row = dict.fromkeys(COLUMNS, '-')
                    row.update(profile=profile, issue=str(issue), raw_dates=dates(raw), pay_dates=dates(pays), redemption_date=str(pays[2]), redemption_bits=hx(input_bits(face*1.01)), ex_terms=f'{"After" if pi%2 else "OnOrAfter"}:{cutoff};OnOrAfter:{pays[2]-timedelta(days=5)}', face_bits=hx(input_bits(face)), coupon_rate_bits=hx(input_bits((0.05, -0.07, -1.0)[(ci+si)%3])), coupon_context=cc, yield_context=yc, settlement=str(settlement), inclusion=inclusion, comp=comp, yield_bits=hx(input_bits(0.04 if (ci+si)%2 else -0.02)), work_cap='1000000', sched_tenor='6M', sched_rule='Forward', sched_first=str(raw[1]), sched_penultimate=str(raw[1]), sched_eom='Ignore', sched_bdc='Following' if lag>0 else 'Preceding' if lag<0 else 'Unadjusted', sched_term_bdc='Following' if lag else 'Unadjusted', sched_holidays=dates(sorted(holidays)), sched_business='-', sched_max='3')
                    rows.append(row)
        # Cap one below complete work, after a nonzero mid-period query.
        for ci, comp in enumerate(COMPS):
            r = next(r.copy() for r in rows if r['profile']==profile and r['comp']==comp and r['settlement']==str(cutoff-timedelta(days=1)) and r['inclusion']=='Include')
            full = evaluate(r, 160); assert full[0] == 'ok'
            r['profile'] += '-WORK-CAP'; r['work_cap'] = str(int(full[10])-1); rows.append(r)
    # Yield-only guards: constructor coupons deliberately use independent A365F.
    failures = [
        ('TARGET-PROJECTED-ANCHOR', ['2025-12-31', '2026-06-30', '2026-12-31'], 'B252:T', '2025-12-31'),
        ('TARGET-GAP-HORIZON', ['2002-12-31', '2026-01-02', '2026-07-02'], 'B252:T', '2002-12-31'),
        ('TARGET-PRE-INCEPTION', ['1998-12-31', '1999-06-30', '1999-12-31'], 'B252:T', '1998-12-31'),
        ('TARGET-POST-MATURITY-ANCHOR', ['2027-08-31', '2028-02-29', '2028-08-31'], 'B252:T', '2029-01-01'),
        ('ICMA-ANCHOR-COVERAGE', base_raw := ['2027-08-31', '2028-02-29', '2028-08-31'], context_icma(['2027-09-01', '2028-02-29', '2028-08-31']), '2027-08-31'),
        ('ICMA-HORIZON-COVERAGE', base_raw, context_icma(['2027-08-31', '2028-02-29', '2028-08-30']), '2027-08-31'),
    ]
    template = rows[0]
    for profile, raw, ctx, settlement in failures:
        for ci, comp in enumerate(COMPS):
            r = template.copy(); r.update(profile=profile, issue=raw[0], raw_dates=','.join(raw), pay_dates=','.join(raw), redemption_date=raw[2], coupon_context='A365F', yield_context=ctx, settlement=settlement, comp=comp, sched_first=raw[1], sched_penultimate=raw[1], sched_bdc='Unadjusted', sched_term_bdc='Unadjusted', sched_holidays='-', ex_terms=f'After:{date.fromisoformat(raw[1])-timedelta(days=5)};OnOrAfter:{date.fromisoformat(raw[2])-timedelta(days=5)}', yield_bits=hx(input_bits(-0.02 if ci%2 else 0.04)))
            rows.append(r)
    for i, row in enumerate(rows, 1): row['id'] = f'BCTX-{i:06}'
    return rows


def generate():
    controls = self_checks()
    rows = make_rows(); counts = collections.Counter(); records = []; profiles_count = collections.Counter(); config_count = collections.Counter()
    for row in rows:
        results = [evaluate(row, p) for p in PRECISIONS]
        assert results[0] == results[1], (row['id'], results)
        result = results[1]; assert len(result) == 25, (len(result), result)
        for name, value in zip(COLUMNS[-25:], result): row[name] = value
        counts[row['expected']] += 1; profiles_count[row['profile']] += 1; config_count[row['comp']] += 1
        records.append('\t'.join(row[k] for k in COLUMNS)+'\n')
    keys = [tuple(row[k] for k in COLUMNS[2:-25]) for row in rows]
    assert len(set(keys)) == len(keys), 'duplicate input tuple'
    assert len(rows) == 11_136
    assert counts == {'ok': 10_432, 'work': 512, 'icma': 64, 'calendar': 128}
    assert set(config_count.values()) == {348} and len(config_count) == 32
    ledger_counts, yf_counts, product_entries = collections.Counter(), collections.Counter(), 0
    for row in rows:
        for name in ('full_queries', 'plan_queries'):
            for cell in row[name].split(';'):
                fields = cell.split('~'); assert len(fields) == 10
                ledger_counts[fields[0][0]+'-'+fields[-1]] += 1
                if fields[-1] == 'ok': yf_counts[fields[5]] += 1
                if fields[8] != '-': product_entries += len(fields[8].split(','))
    body = '\t'.join(COLUMNS)+'\n'+''.join(records)
    requirements = '\n'.join(line for line in (ROOT/'docs/REQUIREMENTS.md').read_text().splitlines() if line.startswith('| FR-BOND-'))+'\n'
    source = Path(__file__).read_bytes()
    header = [('schema','1'),('fixture-set','M8-BOND-CONTEXTUAL-ORACLE'),('domain','bonds'),('description','Independent contextual bond raw-rational and staged-binary enclosures across all32 compounding configurations'),('reference','oracle:Fraction-civil-DCC+ICMA-four-ULP-enumeration+Decimal160-240'),('reference-version','Python>=3.9 stdlib'),('reference-source','Adopted local bond/daycount/calendar/numerical contracts; no Rust or QuantLib output'),('tolerance','Exact discrete/product-map stages; ICMA YF<=4ULP; propagated outward PV enclosures conditional on measured M4 E3 factor criterion'),('reproduction','python3 -I validation/generator/bond_contextual_oracles.py --check'),('generated-at','2026-10-17T05:09:03Z'),('platform','platform-independent stdlib integer/Fraction/Decimal'),('generator-sha256',sha(source)),('generator-fnv64',fnv(source)),('requirements-rows-sha256',sha(requirements.encode())),('requirements-rows-fnv64',fnv(requirements.encode())),('record-count',str(len(rows))),('unique-input-count',str(len(set(keys)))),('profile-counts',';'.join(f'{k}={v}' for k,v in sorted(profiles_count.items()))),('configuration-counts',';'.join(f'{k}={v}' for k,v in sorted(config_count.items()))),('expectation-counts',';'.join(f'{k}={v}' for k,v in sorted(counts.items()))),('body-sha256',sha(body.encode())),('body-fnv64',fnv(body.encode()))]
    header += [('ledger-query-counts',';'.join(f'{k}={v}' for k,v in sorted(ledger_counts.items()))),('yf-profile-counts',';'.join(f'{k}={v}' for k,v in sorted(yf_counts.items()))),('product-map-entry-count',str(product_entries)),('precision-agreement-count',str(len(rows))),('equal-date-guard-control-count',str(controls))]
    for file in CONTRACTS:
        b=(ROOT/file).read_bytes(); name=Path(file).stem.lower().replace('_','-')
        header += [(name+'-sha256',sha(b)),(name+'-fnv64',fnv(b))]
    return (''.join(f'{k}: {v}\n' for k,v in header)+'\n'+body).encode(), dict(counts)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args()
    result, counts=generate()
    if args.check: assert DEST.read_bytes()==result, 'fixture differs'
    else: DEST.parent.mkdir(parents=True,exist_ok=True);DEST.write_bytes(result)
    print(json.dumps(counts,sort_keys=True));print(f'bytes={len(result)} sha256={sha(result)} check={args.check}')
if __name__=='__main__':main()
