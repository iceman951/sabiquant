#!/usr/bin/env python3
"""Additive, production-unread Fraction/Decimal bond yield/risk evidence.

Only this generator's two NEW fixtures are written. Earlier generators,
fixtures and contracts are inputs/pins and are never rewritten. Integer and
Fraction calculations are the primary exact kernel oracle; Decimal financial
references are independently evaluated at 160 and 240 digits.
"""
from __future__ import annotations

import argparse
from collections import Counter
from decimal import Decimal, localcontext
from datetime import date, timedelta
from fractions import Fraction as F
from functools import lru_cache
import calendar
import hashlib
from itertools import product
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'validation/fixtures/bonds/yield-risk-extended.fixture'
PUBLIC_OUT = ROOT / 'validation/fixtures/bonds/yield-risk-extended-public.fixture'
SIGN = 1 << 63
MAX = 0x7fefffffffffffff
ONE = 0x3ff0000000000000
MIN = 1
T970 = (970 + 1023) << 52
FREQUENCIES = (1, 2, 3, 4, 6, 12, 13, 26, 52, 365)
COMPS = ('Simple', 'Continuous') + tuple(
    f'{method}:{m}' for method in ('Compounded', 'SimpleThenCompounded', 'CompoundedThenSimple')
    for m in FREQUENCIES)
COLS = ('id op family comp y_bits t_bits amount_bits dirty_bits target_bits '
        'numerators denominators pow2 negative recipes scale status sign '
        'significand exponent expected_bits error_kind error_phase error_index detail').split()
MEASURES = ('p1', 'p2', 'moment', 'modified', 'macaulay', 'convexity')
PUBLIC_COLS = ('id family kind issue raw_dates pay_dates settlement inclusion coupon_context '
               'yield_context face_bits rate_bits redemption_date redemption_bits ex_terms comp '
               'yield_bits status error_kind error_phase error_index flows dirty_lo dirty_hi').split()
PUBLIC_COLS += [f'{m}_{s}' for m in MEASURES for s in ('status', 'lo', 'hi')]
PUBLIC_COLS += [f'ideal_{m}' for m in ('dirty', *MEASURES)]
PUBLIC_COLS += [f'ideal_{m}_error' for m in ('dirty', *MEASURES)]
CONTRACTS = ('docs/M8_BOND_YIELD_RISK_CONTRACT.md', 'docs/M8_BOND_CORE_CONTRACT.md',
             'docs/M4_DATE_CONTRACT.md', 'docs/M3_ICMA_CONTRACT.md',
             'docs/M3_US_AFB_CONTRACT.md', 'docs/M3_DAYCOUNT_EXTENSIONS.md',
             'docs/M3_TARGET_CONTRACT.md',
             'docs/M6_SOLVER_RESEARCH.md', 'docs/NUMERICAL_CORRECTNESS.md')


def power(e):
    return F(1 << e) if e >= 0 else F(1, 1 << -e)


def hx(b):
    return f'{b:016x}'


def bits(x):
    return struct.unpack('>Q', struct.pack('>d', x))[0]


def binary(b):
    sign = -1 if b & SIGN else 1
    field, sig = (b >> 52) & 2047, b & ((1 << 52) - 1)
    assert field != 2047
    if field:
        sig |= 1 << 52
    return sign * F(sig) * power(field - 1075 if field else -1074)


def binade(x):
    assert x > 0
    e = x.numerator.bit_length() - x.denominator.bit_length()
    return e - (x < power(e))


def nearest_integer(x):
    q, r = divmod(x.numerator, x.denominator)
    return q + (2 * r > x.denominator or (2 * r == x.denominator and q & 1))


def admission(x):
    """Exact ordinary IEEE RN-even, with signed nonzero range failures."""
    if not x:
        return 'ok', 0
    neg = x < 0
    x = abs(x)
    top = binade(x)
    unit = max(-1074, top - 52)
    q = nearest_integer(x / power(unit))
    if q == 0:
        return ('underflow-' if neg else 'underflow+'), None
    if q == 1 << 53:
        q >>= 1
        unit += 1
    if unit + 52 > 1023:
        return ('overflow-' if neg else 'overflow+'), None
    if unit == -1074 and q < 1 << 52:
        result = q
    else:
        result = ((unit + 52 + 1023) << 52) | (q - (1 << 52))
    return 'ok', result | (SIGN if neg else 0)


def rn(x):
    status, b = admission(x)
    assert status == 'ok', (status, x)
    return b


def extended(x):
    """RN-even to53 normalized bits, without binary64 exponent admission."""
    assert x
    e = binade(abs(x)) - 52
    q = nearest_integer(abs(x) / power(e))
    if q == 1 << 53:
        q >>= 1
        e += 1
    assert 1 << 52 <= q < 1 << 53
    return x < 0, q, e


def recipe_value(rec):
    neg, shift, nums, dens = rec
    assert 1 <= len(nums) <= 5 and len(dens) <= 2
    val = power(shift)
    for b in nums:
        assert binary(b) > 0
        val *= binary(b)
    for b in dens:
        assert binary(b) > 0
        val /= binary(b)
    return -val if neg else val


def rounded_recipe(rec):
    neg, q, e = extended(recipe_value(rec))
    return (-1 if neg else 1) * q * power(e)


def encode_recipe(rec):
    neg, shift, nums, dens = rec
    return ','.join((str(int(neg)), str(shift), ':'.join(map(hx, nums)),
                     ':'.join(map(hx, dens)) or '-'))


def record(op, family, **cells):
    row = dict.fromkeys(COLS, '-')
    row.update(op=op, family=family, **{k: str(v) for k, v in cells.items()})
    return row


def residual_rows():
    # Every finite exponent field, signed adjacent/equal/opposite controls.
    pairs = []
    for exp in range(2047):
        v = (exp << 52) | 1
        pairs.extend(((v, v), (v + 1, v), (v - 1, v),
                      (v, v | SIGN), (v | SIGN, v), (v + 1 | SIGN, v | SIGN)))
    anchors = (0, 1, 2, 3, (1 << 52) - 1, 1 << 52,
               ONE - 1, ONE, ONE + 1, T970 - 1, T970, T970 + 1, MAX - 1, MAX)
    for d, t in product(anchors, repeat=2):
        for ds, ts in ((0, 0), (SIGN, 0), (0, SIGN), (SIGN, SIGN)):
            pairs.append((d | ds if d else 0, t | ts if t else 0))
    seen = set()
    for d, t in pairs:
        if (d, t) in seen:
            continue
        seen.add((d, t))
        df, tf = binary(d), binary(t)
        scale = int(abs(tf) >= power(970))
        st, result = admission((df - tf) * power(-scale))
        assert st == 'ok'
        assert (result == 0) == (df == tf)
        if df != tf:
            assert bool(result & SIGN) == (df < tf)
        ast, amount = admission(df - tf)
        yield record('residual', 'ALL-EXPONENT-FIELDS' if len(seen) <= 12282 else 'EXTREME-CROSS',
                     dirty_bits=hx(d), target_bits=hx(t), scale=scale, status=st,
                     expected_bits=hx(result), detail=f'amount:{ast}:{hx(amount) if amount is not None else "-"}')


def term_recipes():
    # Every numerator/denominator arity with all extreme products, not staged floats.
    for n in range(1, 6):
        for k in range(3):
            for ns in product((MIN, MAX), repeat=n):
                for ds in product((MIN, MAX), repeat=k):
                    for negative, shift in product((False, True), (0, 1)):
                        yield 'FINITE-WIDTH-EXTREMES', (negative, shift, ns, ds)
    # Products exactly at a normalized53-bit tie. Multiplying odd53-bit
    # significands by1.5 produces half-unit remainders at the next binade.
    tie_values = (ONE, ONE + 1, ONE + 2, ONE + 3,
                  bits(1.5), bits(1.5) + 1, bits(2.0) - 1)
    for a, b, den, negative in product(tie_values, tie_values, (ONE, ONE + 1, bits(3.0)), (False, True)):
        yield 'TIES-CARRY-QUOTIENT', (negative, 0, (a, b), (den,))
    # (1+2u)/(1+u)^2 is strictly below1, but within half of the last
    # normalized unit. RN therefore carries2^53 and raises the binade.
    for negative, shift in product((False, True), (0, 1)):
        yield 'NORMALIZATION-CARRY', (negative, shift, (ONE + 2,), (ONE + 1, ONE + 1))
    # Each finite exponent field enters as a valid scalar factor, either in
    # numerator or denominator, exposing all decoding/normalization binades.
    for field in range(2047):
        a = (field << 52) | ((1 << 51) + 1)
        yield 'SCALAR-EXPONENT-CENSUS', (bool(field & 1), 0, (a,), ())
        yield 'SCALAR-EXPONENT-CENSUS', (not field & 1, 1, (ONE,), (a,))


def term_rows():
    seen = set()
    for family, rec in term_recipes():
        if rec in seen:
            continue
        seen.add(rec)
        neg, q, e = extended(recipe_value(rec))
        assert -7470 <= e <= 7216
        yield record('term', family, numerators=':'.join(map(hx, rec[2])),
                     denominators=':'.join(map(hx, rec[3])) or '-', pow2=rec[1],
                     negative=int(rec[0]), status='ok', sign=int(neg),
                     significand=hx(q), exponent=e)


def sum_rows():
    low = (False, 0, (MIN,) * 5, (MAX,) * 2)
    high = (False, 0, (MAX,) * 5, (MIN,) * 2)
    ordinary = (False, 0, (bits(1.0),), ())
    cases = [('EMPTY', (), 0), ('EMPTY-FINITE-DIVISOR', (), ONE)]
    # Overflowing unsigned prefixes, exact cancellation and a tiny retained
    # tail span practically the full -7470..7216 exponent proof.
    for n in (1, 2, 3, 17, 64, 257):
        positive = (high,) * n
        negative = tuple((True, *r[1:]) for r in positive)
        for tail in ((), (low,), ((True, *low[1:]),), (ordinary,)):
            for divisor in (MIN, ONE, MAX, MAX | SIGN, 0):
                cases.append(('FULL-WIDTH-CANCELLATION', positive + tail + negative, divisor))
    # Direct IEEE midpoint and sign tests; every entry derives from factors.
    for a, t, negative in product((MIN, 2, (1 << 52) - 1, MAX),
                                   (bits(.25), bits(.5), ONE, bits(2.0)), (False, True)):
        for divisor in (MIN, ONE, MAX, ONE | SIGN, 0):
            cases.append(('RAW-NORMALIZED-INDEPENDENCE', ((negative, 0, (a, t), ()),), divisor))
    for family, recipes, dirty in cases:
        total = sum(map(rounded_recipe, recipes), F(0))
        coeff = total / power(-8192)
        assert coeff.denominator == 1 and abs(coeff.numerator).bit_length() <= 243 * 64
        status, b = admission(total)
        if not binary(dirty):
            nst, nb = 'zero_dirty', None
        else:
            nst, nb = admission(total / binary(dirty))
        yield record('sum', family, recipes=';'.join(map(encode_recipe, recipes)) or '-',
                     dirty_bits=hx(dirty), status=status, sign=int(total < 0),
                     significand=f'{abs(coeff.numerator):x}', exponent=-8192,
                     expected_bits=hx(b) if b is not None else '-',
                     error_kind='bond.value' if status != 'ok' else '-',
                     error_phase='time weighted amount' if status != 'ok' else '-',
                     detail=f'normalized:{nst}:{hx(nb) if nb is not None else "-"}')


def branch(comp, t):
    if comp in ('Simple', 'Continuous'):
        return comp, 0
    method, m = comp.split(':')
    m = int(m)
    threshold = binary(rn(F(1, m)))
    simple = ((method == 'SimpleThenCompounded' and t <= threshold)
              or (method == 'CompoundedThenSimple' and t > threshold))
    return ('Simple', m) if simple else ('Periodic', m)


def dec(x):
    return Decimal(x.numerator) / Decimal(x.denominator)


@lru_cache(None)
def ideal_flow(comp, y, a, t, precision):
    """Exact rational Simple, Decimal fixed-branch real model otherwise.

    Fraction inputs remove cancellation in the periodic base before Decimal
    conversion. Every Decimal operation, including summation by its caller,
    is inside the requested local precision. The explicit uncertainty applies
    only to these bounded corpus evaluations, never arbitrary libm calls.
    """
    br, m = branch(comp, t)
    if not t:
        return (a, F(0), F(0), F(0)), F(0)
    if br == 'Simple':
        q = 1 + y * t
        assert q > 0
        pv = a / q
        return (pv, -pv * t / q, 2 * pv * t * t / (q * q), pv * t), F(0)
    if not y:
        pv = a
        if br == 'Continuous':
            return (pv, -pv * t, pv * t * t, pv * t), F(0)
        base = F(1)
        return (pv, -pv * t, pv * t * (t + F(1, m)), pv * t), F(0)
    with localcontext() as ctx:
        ctx.prec = precision
        td, ad, yd = dec(t), dec(a), dec(y)
        if br == 'Continuous':
            logarithm = -yd * td
            discount = logarithm.exp()
            p = ad * discount
            values = (p, -p * td, p * td * td, p * td)
        else:
            base = 1 + y / m
            assert base > 0
            bd = dec(base)
            logarithm = -Decimal(m) * td * bd.ln()
            discount = logarithm.exp()
            p = ad * discount
            values = (p, -p * td / bd,
                      p * td * dec(t + F(1, m)) / (bd * bd), p * td)
        assert abs(logarithm) < 1000
        # At most24 rounded operations at precision p; |log(discount)|<1000.
        # Absolute/relative primitive error propagation is below10^(-p+16)
        # for this positive-base corpus. The wider10^(-p+24) envelope also
        # dominates conversion, derivative products and the final divisions.
        uncertainty = F(Decimal(10) ** (-precision + 24))
        return tuple(F(v) for v in values), uncertainty


@lru_cache(None)
def discount_interval(comp, y, t, precision):
    br, _ = branch(comp, t)
    if not t or not y:
        return F(1), F(1)
    if br == 'Simple':
        q = binary(rn(1 + y * t))
        assert q > 0
        d = binary(rn(1 / q))
        return d, d
    values, eps = ideal_flow(comp, y, F(1), t, precision)
    d = values[0]
    with localcontext() as ctx:
        ctx.prec = precision
        logabs = abs(dec(d).ln())
        allowance = abs(d) * F(8, 1 << 53) * (1 + F(logabs))
    # E3 is the existing measured comparison criterion, not a platform theorem.
    allowance += power(-1074) + max(F(1), abs(d)) * eps
    assert d > allowance
    return d - allowance, d + allowance


def contributions(comp, y, t, pv):
    if not t or not pv:
        return F(0), F(0), F(0)
    br, m = branch(comp, t)
    moment = pv * t
    if br == 'Simple':
        q = binary(rn(1 + y * t))
        exact = (-pv * t / q, 2 * pv * t * t / (q * q), moment)
    elif br == 'Continuous':
        exact = (-pv * t, pv * t * t, moment)
    else:
        delta = binary(rn(F(m) + y))
        w = binary(rn(t + binary(rn(F(1, m)))))
        exact = (-pv * t * m / delta, pv * t * w * m * m / (delta * delta), moment)
    result = []
    for v in exact:
        neg, q, e = extended(v)
        result.append((-1 if neg else 1) * q * power(e))
    return tuple(result)


def staged_flow_interval(comp, y, amounts, times, precision):
    candidates = []
    for t in times:
        dlo, dhi = discount_interval(comp, y, t, precision)
        pv_endpoints = [binary(rn(a * d)) for a in amounts for d in (dlo, dhi)]
        p_lo, p_hi = min(pv_endpoints), max(pv_endpoints)
        lo_terms = contributions(comp, y, t, p_lo)
        hi_terms = contributions(comp, y, t, p_hi)
        candidates.append(((p_lo, p_hi),) + tuple((min(l, h), max(l, h))
                                                for l, h in zip(lo_terms, hi_terms)))
    return tuple((min(c[k][0] for c in candidates), max(c[k][1] for c in candidates))
                 for k in range(4))


def admitted_interval(interval):
    results = [admission(v) for v in interval]
    assert results[0][0] == results[1][0], ('ambiguous range interval', interval, results)
    status = results[0][0]
    return status, results[0][1], results[1][1]


def flow_rows(precision):
    for comp in COMPS:
        m = int(comp.split(':')[1]) if ':' in comp else 1
        cutoff = rn(F(1, m))
        times = (cutoff - 1, cutoff, cutoff + 1, rn(F(2, m)))
        yields = (bits(.0375), bits(-.125), bits(-float(m)) - 1)
        for yb, tb, ab in product(yields, times, (bits(100.0), bits(-100.0))):
            y, t, a = binary(yb), binary(tb), binary(ab)
            br, fm = branch(comp, t)
            if br == 'Simple' and 1 + y * t <= 0:
                continue
            if comp == 'Continuous' and y * t < -600:
                continue
            intervals = staged_flow_interval(comp, y, (a,), (t,), precision)
            encoded = []
            for name, interval in zip(('pv', 'd1', 'd2', 'moment'), intervals):
                st, lo, hi = admitted_interval(interval)
                assert st == 'ok'
                encoded.append(f'{name}:{hx(lo)}:{hx(hi)}')
            label = f'Periodic:{fm}' if br == 'Periodic' else br
            yield record('flow', 'ROUNDED-HYBRID-AND-NEAR-M', comp=comp, y_bits=hx(yb),
                         t_bits=hx(tb), amount_bits=hx(ab), status='ok',
                         detail=';'.join((f'branch:{label}', *encoded)))
    for comp in COMPS:
        for tb, ab in ((0, bits(-100.0)), (bits(.5), 0)):
            t, a, y = binary(tb), binary(ab), F(1, 16)
            br, m = branch(comp, t)
            intervals = staged_flow_interval(comp, y, (a,), (t,), precision)
            label = f'Periodic:{m}' if br == 'Periodic' else br
            encoded = [f'{name}:{hx(rn(bounds[0]))}:{hx(rn(bounds[1]))}'
                       for name, bounds in zip(('pv', 'd1', 'd2', 'moment'), intervals)]
            yield record('flow', 'ZERO-TIME-OR-AMOUNT', comp=comp, y_bits=hx(rn(y)),
                         t_bits=hx(tb), amount_bits=hx(ab), status='ok',
                         detail=';'.join((f'branch:{label}', *encoded)))


def dates(text):
    return () if text == '-' else tuple(date.fromisoformat(x) for x in text.split(','))


def last_feb(d):
    return d.month == 2 and d.day == calendar.monthrange(d.year, 2)[1]


def custom_business(d, mask, holidays, business):
    return d in business or (d not in holidays and not (mask & (1 << d.weekday())))


def target_business(d):
    assert 2026 <= d.year <= 2028
    easter = {2026: ((4, 3), (4, 6)), 2027: ((3, 26), (3, 29)),
              2028: ((4, 14), (4, 17))}
    holidays = {(1, 1), (5, 1), (12, 25), (12, 26), *easter[d.year]}
    return d.weekday() < 5 and (d.month, d.day) not in holidays


@lru_cache(None)
def day_fraction(context, start, end):
    """Original chronological civil/rational formulas; no Rust calls."""
    assert start <= end
    if context.startswith('ICMA:'):
        _, frequency, refs = context.split(':')
        refs = dates(refs)
        assert refs[0] <= start <= end <= refs[-1]
        raw = sum((F((min(end, b) - max(start, a)).days,
                     int(frequency) * (b - a).days)
                   for a, b in zip(refs, refs[1:]) if max(start, a) < min(end, b)), F(0))
        return raw, 4 if raw else 0
    if context.startswith('B252:'):
        spec = context.split(':')
        if spec[1] == 'T':
            assert date(2026, 1, 1) <= start <= end <= date(2028, 12, 31)
            predicate = target_business
        else:
            if spec[1] == 'C':
                _, _, mask, hs, bs = spec
                rule = None
            else:
                _, _, rule, mask, hs, bs = spec
            mask, hs, bs = int(mask), set(dates(hs)), set(dates(bs))
            def predicate(d):
                plain = custom_business(d, mask, hs, bs)
                if rule is None:
                    return plain
                return (plain and target_business(d)) if rule == 'H' else (plain or target_business(d))
        return F(sum(predicate(start + timedelta(days=i))
                     for i in range((end - start).days)), 252), 0
    if start == end:
        return F(0), 0
    days = (end - start).days
    if context == 'A360':
        return F(days, 360), 0
    if context == 'A365F':
        return F(days, 365), 0
    if context == 'AAISDA':
        total = F(0)
        current = start
        while current < end:
            stop = min(end, date(current.year + 1, 1, 1))
            total += F((stop - current).days, 366 if calendar.isleap(current.year) else 365)
            current = stop
        return total, 4
    if context == 'AFB':
        remaining, years = end, 0
        while True:
            year = remaining.year - 1
            if year < start.year:
                break
            day = remaining.day
            if remaining.month == 2 and day in (28, 29):
                day = calendar.monthrange(year, 2)[1]
            candidate = date(year, remaining.month, day)
            if candidate < start:
                break
            remaining, years = candidate, years + 1
        leap = any(calendar.isleap(y) and start <= date(y, 2, 29) < remaining
                   for y in range(start.year, remaining.year + 1))
        return F(years) + F((remaining - start).days, 366 if leap else 365), 4
    d1, d2 = start.day, end.day
    if context == '30360':
        if d1 == 31:
            d1 = 30
        if d2 == 31 and d1 == 30:
            d2 = 30
    elif context == '30E360':
        d1, d2 = min(d1, 30), min(d2, 30)
    elif context.startswith('ISDA:'):
        termination = date.fromisoformat(context.split(':')[1])
        if d1 == 31 or last_feb(start):
            d1 = 30
        if d2 == 31 or (last_feb(end) and end != termination):
            d2 = 30
    elif context.startswith('US:'):
        eom = context == 'US:1'
        if eom and last_feb(start) and last_feb(end):
            d2 = 30
        if eom and last_feb(start):
            d1 = 30
        if d2 == 31 and d1 in (30, 31):
            d2 = 30
        if d1 == 31:
            d1 = 30
    else:
        raise AssertionError(context)
    return F(360 * (end.year - start.year) + 30 * (end.month - start.month) + d2 - d1, 360), 0


def fraction_candidates(context, start, end):
    raw, radius = day_fraction(context, start, end)
    center = rn(raw)
    return raw, tuple(binary(b) for b in range(max(0, center - radius), center + radius + 1))


def public_profiles():
    ordinary = ('2027-02-28', '2027-08-31', '2028-02-29')
    refs = '2026-08-31,2027-02-28,2027-08-31,2028-02-29,2028-08-31'
    contexts = ('A360', 'A365F', 'AAISDA', 'AFB', '30360', '30E360',
                'ISDA:2028-02-29', 'ISDA:2028-03-01', 'US:0', 'US:1',
                f'ICMA:2:{refs}', 'B252:C:48:2027-09-06,2028-02-28:2027-09-05',
                'B252:T', 'B252:J:H:65:2027-09-06:2027-09-05',
                'B252:J:B:48:2027-09-06:2027-09-05')
    for context in contexts:
        yield dict(family='CONTEXT-ZERO', kind='Zero', raw_dates=(ordinary[0], ordinary[-1]),
                   settlement='2027-04-15', coupon_context=context, yield_context=context,
                   rate=F(0), redemption=F(100), ex_terms='-')
        yield dict(family='CONTEXT-SIGNED-COUPONS', kind='Explicit', raw_dates=ordinary,
                   settlement='2027-04-15', coupon_context=context, yield_context=context,
                   rate=F(-3, 8), redemption=F(100), ex_terms='-;-')
    yield dict(family='ICMA-SHORT-STUB', kind='Explicit',
               raw_dates=('2027-04-15', ordinary[1], ordinary[2]), settlement='2027-05-20',
               coupon_context=f'ICMA:2:{refs}', yield_context=f'ICMA:2:{refs}',
               rate=F(1, 16), redemption=F(100), ex_terms='-;-')
    yield dict(family='ICMA-LONG-STUB-REOPENING', kind='Explicit',
               raw_dates=('2026-11-15', ordinary[1], ordinary[2]), issue='2026-09-01', settlement='2026-10-01',
               coupon_context=f'ICMA:2:{refs}', yield_context=f'ICMA:2:{refs}',
               rate=F(1, 16), redemption=F(100), ex_terms='-;-')
    yield dict(family='EX-DETACHED', kind='Explicit', raw_dates=ordinary, settlement='2027-08-20',
               coupon_context='US:1', yield_context='B252:T', rate=F(1, 16),
               redemption=F(100), ex_terms='OnOrAfter:2027-08-20;-')
    yield dict(family='PAYMENT-DATE-INCLUDED', kind='Explicit', raw_dates=ordinary,
               settlement=ordinary[1], inclusion='Include', coupon_context='A365F',
               yield_context='AAISDA', rate=F(-1, 8), redemption=F(100), ex_terms='-;-')


def prepared_flows(profile):
    raw = tuple(date.fromisoformat(d) for d in profile['raw_dates'])
    settlement = date.fromisoformat(profile['settlement'])
    included = profile.get('inclusion', 'Exclude') == 'Include'
    face, rate = F(100), binary(rn(profile['rate']))
    result = []
    if profile['kind'] == 'Explicit':
        terms = profile['ex_terms'].split(';')
        for index, (start, end) in enumerate(zip(raw, raw[1:])):
            if end < settlement or (end == settlement and not included):
                continue
            term = terms[index]
            if term != '-':
                policy, cutoff = term.split(':')
                cutoff = date.fromisoformat(cutoff)
                if settlement > cutoff or (policy == 'OnOrAfter' and settlement == cutoff):
                    continue
            raw_yf, yfs = fraction_candidates(profile['coupon_context'], start, end)
            amounts = tuple(binary(rn(face * rate * yf)) for yf in yfs)
            raw_time, times = fraction_candidates(profile['yield_context'], settlement, end)
            result.append((min(amounts), max(amounts), times, face * rate * raw_yf, raw_time))
    redemption_date = date.fromisoformat(profile.get('redemption_date', profile['raw_dates'][-1]))
    if redemption_date > settlement or (redemption_date == settlement and included):
        raw_time, times = fraction_candidates(profile['yield_context'], settlement, redemption_date)
        redemption = binary(rn(profile['redemption']))
        result.append((redemption, redemption, times, redemption, raw_time))
    return result


def up_bits(x):
    assert x >= 0
    st, b = admission(x)
    assert st == 'ok'
    if binary(b) < x:
        b += 1
    assert b <= MAX
    return b


def public_row(profile, comp, yb, precision):
    flows = prepared_flows(profile)
    y = binary(yb)
    gross = [staged_flow_interval(comp, y, (alo, ahi), times, precision)
             for alo, ahi, times, _, _ in flows]
    sums = tuple((sum((v[k][0] for v in gross), F(0)),
                  sum((v[k][1] for v in gross), F(0))) for k in range(4))
    dst, dlo, dhi = admitted_interval(sums[0])
    assert dst == 'ok'
    dirty_interval = binary(dlo), binary(dhi)
    staged = dict(zip(('dirty', 'p1', 'p2', 'moment'), sums))
    for name, source, neg in (('modified', 'p1', -1), ('macaulay', 'moment', 1),
                              ('convexity', 'p2', 1)):
        if dirty_interval == (F(0), F(0)):
            staged[name] = None
        else:
            assert not dirty_interval[0] <= 0 <= dirty_interval[1], ('uninformative ratio', profile, comp)
            candidates = [neg * n / d for n in staged[source] for d in dirty_interval]
            staged[name] = min(candidates), max(candidates)
    ideal_observations = [ideal_flow(comp, y, a, t, precision) for _, _, _, a, t in flows]
    ideals = [v for v, _ in ideal_observations]
    ideal_values = [sum((v[k] for v in ideals), F(0)) for k in range(4)]
    ideal_errors = [sum((max(F(1), abs(v[k])) * eps for v, eps in ideal_observations), F(0))
                    for k in range(4)]
    ideal_intervals = [(v - e, v + e) for v, e in zip(ideal_values, ideal_errors)]
    ip, i1, i2, im = ideal_values
    ideal_values += [-i1 / ip, im / ip, i2 / ip] if ip else [None] * 3
    for numerator, sign in ((1, -1), (3, 1), (2, 1)):
        if not ip:
            ideal_intervals.append(None)
        else:
            assert not ideal_intervals[0][0] <= 0 <= ideal_intervals[0][1]
            ratios = [sign * n / d for n in ideal_intervals[numerator] for d in ideal_intervals[0]]
            ideal_intervals.append((min(ratios), max(ratios)))
    # Precision is justified by a finite-domain uncertainty calculation in
    # addition to whole-record160/240 agreement: the enclosure must determine
    # the same IEEE rounding or signed range outcome at both edges.
    for value, interval in zip(ideal_values, ideal_intervals):
        if value is not None:
            assert admission(interval[0]) == admission(interval[1]) == admission(value), (
                'precision does not decide ideal rounding', profile, comp, value, interval)
    ideal = dict(zip(('dirty', 'p1', 'p2', 'moment', 'modified', 'macaulay', 'convexity'), ideal_values))
    row = dict.fromkeys(PUBLIC_COLS, '-')
    raw_dates = ','.join(profile['raw_dates'])
    row.update(family=profile['family'], kind=profile['kind'], issue=profile.get('issue', profile['raw_dates'][0]),
               raw_dates=raw_dates, pay_dates=raw_dates, settlement=profile['settlement'],
               inclusion=profile.get('inclusion', 'Exclude'), coupon_context=profile['coupon_context'],
               yield_context=profile['yield_context'], face_bits=hx(bits(100.0)),
               rate_bits=hx(rn(profile['rate'])), redemption_date=profile.get('redemption_date', profile['raw_dates'][-1]),
               redemption_bits=hx(rn(profile['redemption'])), ex_terms=profile['ex_terms'],
               comp=comp, yield_bits=hx(yb), status='ok', dirty_lo=hx(dlo), dirty_hi=hx(dhi),
               flows=';'.join(':'.join(hx(rn(v)) for v in (alo, ahi, min(times), max(times)))
                              for alo, ahi, times, _, _ in flows) or '-')
    for name in MEASURES:
        if staged[name] is None:
            row[f'{name}_status'] = 'zero_dirty'
        else:
            status, lo, hi = admitted_interval(staged[name])
            row[f'{name}_status'] = status
            if status == 'ok':
                row[f'{name}_lo'], row[f'{name}_hi'] = hx(lo), hx(hi)
    for name, value in ideal.items():
        if value is None:
            row[f'ideal_{name}'] = 'zero_dirty'
            continue
        status, b = admission(value)
        row[f'ideal_{name}'] = hx(b) if b is not None else status
        if b is not None and staged[name] is not None:
            st, lo, hi = admitted_interval(staged[name])
            if st == 'ok':
                error = max(abs(binary(lo) - binary(b)), abs(binary(hi) - binary(b)))
                row[f'ideal_{name}_error'] = hx(up_bits(error))
    return row


def public_rows(precision):
    for profile in public_profiles():
        for comp, yb in product(COMPS, (bits(.0375), bits(-.0625))):
            yield public_row(profile, comp, yb, precision)
    # All exact supplied-value controls. No elementary uncertainty is injected
    # into zero-yield or Simple identity paths.
    controls = (
        ('SIGNED-CANCELLATION-DISCRIMINATOR', F(-1), binary(bits(100.0) + 1), bits(.25), 'Simple', '2025-01-01'),
        ('EXACT-ZERO-DIRTY', F(-1), F(100), 0, 'Compounded:1', '2025-01-01'),
        ('NEGATIVE-DIRTY', F(-2), F(100), 0, 'Continuous', '2025-01-01'),
        ('MAX-RAW-OVERFLOW-RESCUE', F(0), binary(MAX), 0, 'Continuous', '2026-01-01'),
        ('MIN-RAW-UNDERFLOW-RESCUE', F(0), binary(MIN), 0, 'Continuous', '2024-07-01'),
    )
    for family, rate, redemption, yb, comp, maturity in controls:
        raw = ('2024-01-01', maturity)
        profile = dict(family=family, kind='Zero' if rate == 0 else 'Explicit',
                       raw_dates=raw, settlement=raw[0], coupon_context='30360',
                       yield_context='30360', rate=rate, redemption=redemption,
                       ex_terms='-' if rate == 0 else '-')
        yield public_row(profile, comp, yb, precision)


def fnv(data):
    value = 0xcbf29ce484222325
    for b in data:
        value = ((value ^ b) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{value:016x}'


def fixture_bytes(rows, columns, extra):
    records = ''.join('\t'.join(row[k] for k in columns) + '\n' for row in rows).encode()
    body = ('\t'.join(columns) + '\n').encode() + records
    source = Path(__file__).read_bytes()
    census = dict(sorted(Counter(r['op'] if 'op' in r else r['family'] for r in rows).items()))
    headers = [
        'schema: 1', 'fixture-set: M8-BOND-YIELD-RISK-EXTENDED', 'domain: bonds',
        'description: Independent exact wide-kernel and conditioned financial evidence; additive corpus',
        'generator: validation/generator/bond_yield_risk_extended_oracles.py',
        'reference: independent exact integer/Fraction and two-precision Decimal; no production reads',
        'reference-version: Python standard library',
        'reference-source: docs/M8_BOND_YIELD_RISK_CONTRACT.md',
        'generated-at: 2026-10-18T00:00:00Z',
        'platform: platform-independent Python standard-library integers/Fraction/Decimal',
        'reproduction: python3 -I validation/generator/bond_yield_risk_extended_oracles.py --check',
        'tolerance: exact kernels TOL-E0; financial enclosures conditional on measured M4 E3; no net-ULP promise',
        f'generator-sha256: {hashlib.sha256(source).hexdigest()}',
        f'generator-fnv64: {fnv(source)}', f'record-count: {len(rows)}',
        f'body-sha256: {hashlib.sha256(body).hexdigest()}', f'body-fnv64: {fnv(body)}',
        'census: ' + json.dumps(census, sort_keys=True, separators=(',', ':')),
        'family-counts: ' + ';'.join(f'{k}={v}' for k, v in sorted(Counter(r['family'] for r in rows).items())),
        'compounding-counts: ' + ';'.join(f'{k}={v}' for k, v in sorted(Counter(r['comp'] for r in rows if r['comp'] != '-').items())),
        f'records-sha256: {hashlib.sha256(records).hexdigest()}',
    ]
    for name in CONTRACTS:
        data = (ROOT / name).read_bytes()
        key = Path(name).stem.lower().replace('_', '-')
        headers.append(f'contract-{key}-sha256: {hashlib.sha256(data).hexdigest()}')
        headers.append(f'contract-{key}-fnv64: {fnv(data)}')
    req = ''.join(line + '\n' for line in (ROOT / 'docs/REQUIREMENTS.md').read_text().splitlines()
                  if line.startswith('| FR-YR-')).encode()
    headers += [f'requirements-fnv64: {fnv(req)}', f'requirements-sha256: {hashlib.sha256(req).hexdigest()}']
    headers.extend(extra)
    return ('\n'.join(headers) + '\n\n').encode() + body


def generate_kernel():
    rows = list(residual_rows()) + list(term_rows()) + list(sum_rows())
    low, high = list(flow_rows(160)), list(flow_rows(240))
    assert low == high, 'flow160/240 serialization mismatch'
    rows += low
    for i, row in enumerate(rows, 1):
        row['id'] = f'BYRX-{i:06d}'
    return fixture_bytes(rows, COLS, ['exact-basis: 2^-8192', 'decimal-precisions: 160,240'])


def generate_public():
    low, high = list(public_rows(160)), list(public_rows(240))
    assert low == high, 'public160/240 serialization mismatch'
    for i, row in enumerate(low, 1):
        row['id'] = f'BYRPX-{i:06d}'
    return fixture_bytes(low, PUBLIC_COLS, ['decimal-precisions: 160,240',
                         'schedule-metadata: Null/Unadjusted/Unadjusted; explicit6M/Forward/Ignore; first=penultimate interior if present; cap3',
                         'compounding-count: 32', 'public-column-count: 56'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--kernel-only', action='store_true')
    args = parser.parse_args()
    generated = {OUT: generate_kernel()}
    if not args.kernel_only:
        generated[PUBLIC_OUT] = generate_public()
    for path, data in generated.items():
        if args.check:
            if not path.exists() or path.read_bytes() != data:
                raise SystemExit(f'NOT REPRODUCIBLE: {path.relative_to(ROOT)}')
        else:
            path.write_bytes(data)
        print(json.dumps({'path': str(path.relative_to(ROOT)), 'bytes': len(data),
                          'sha256': hashlib.sha256(data).hexdigest(), 'fnv64': fnv(data),
                          'check': args.check}, sort_keys=True), flush=True)


if __name__ == '__main__':
    main()
