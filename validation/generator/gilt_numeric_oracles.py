#!/usr/bin/env python3
"""Independent exact conventional-gilt numerical primitives; no Rust financial reads.

This additive fixture is authored from the frozen local gilt contract before
production. Accepted primitive inputs are explicitly not a native libm profile
or a claim that a public yield produces every supplied discount/log/PV.
"""
from __future__ import annotations

import argparse
import calendar
from collections import Counter
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal, ROUND_HALF_EVEN, localcontext
from fractions import Fraction as F
from functools import lru_cache
import hashlib
from itertools import product
import json
from pathlib import Path
import platform
import random
import struct

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'validation/fixtures/bonds/gilt-numeric-oracle.fixture'
CONTRACT = ROOT / 'docs/M8_GILT_CONTRACT.md'
CONTRACT_SHA = '03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292'
SIGN = 1 << 63
INF = 0x7ff0000000000000
MAX = INF - 1
ONE = 0x3ff0000000000000
LIMIT = (1 << 127) - 1
BASIS = -8192
WORDS = 243
COLS = ('''id op family reachability source c_coef c_scale r s j redeem dates
ai_branch t s2 y_bits g_bits discount_bits pv_bits delta_bits measure recipes
bundle_count dirty_bits target_bits scale exact_n exact_d pow2 negative
significand exponent basis_bit_lengths basis_fnv64 status expected_bits
alternate_bits normalized_status normalized_bits amount_status amount_bits
p1_status p1_bits p2_status p2_bits moment_status moment_bits modified_status
modified_bits macaulay_status macaulay_bits convexity_status convexity_bits detail''').split()
MEASURES = ('p1', 'p2', 'moment', 'modified', 'macaulay', 'convexity')
STATS: Counter[str] = Counter()


def p2(e: int) -> F:
    return F(1 << e) if e >= 0 else F(1, 1 << -e)


def p10(e: int) -> F:
    return F(10 ** e) if e >= 0 else F(1, 10 ** -e)


def hx(b: int) -> str:
    return f'{b:016x}'


def bits(x: float) -> int:
    return struct.unpack('>Q', struct.pack('>d', x))[0]


@lru_cache(maxsize=16384)
def binary(b: int) -> F:
    """Exact decoding, independent of the machine's decimal formatting."""
    negative = bool(b & SIGN)
    field, tail = (b >> 52) & 2047, b & ((1 << 52) - 1)
    assert field != 2047
    significand = tail + ((1 << 52) if field else 0)
    value = F(significand) * p2(field - 1075 if field else -1074)
    return -value if negative else value


def decompose(b: int) -> tuple[int, int]:
    assert 0 <= b < INF
    field, tail = (b >> 52) & 2047, b & ((1 << 52) - 1)
    return (tail + ((1 << 52) if field else 0), field - 1075 if field else -1074)


ZERO_MID = p2(-1075)
OVER_MID = p2(1024) - p2(970)


def ieee_cell(magnitude: int) -> tuple[F, F | None]:
    if magnitude == 0:
        return F(0), ZERO_MID
    if magnitude == INF:
        return OVER_MID, None
    value = binary(magnitude)
    lower = (binary(magnitude - 1) + value) / 2
    upper = OVER_MID if magnitude == MAX else (value + binary(magnitude + 1)) / 2
    return lower, upper


def certify_cell(value: F, lower: F, upper: F | None, even: bool) -> bool:
    """Exact adjacent-representable Voronoi certificate, including parity."""
    assert value >= lower and (upper is None or value <= upper)
    on_boundary = value == lower or value == upper
    if on_boundary:
        assert even
    return on_boundary


def ieee(value: F, canonical_zero: bool = False) -> tuple[str, int]:
    """CPython Fraction conversion + independent exact cell/parity certificate.

    This intentionally does not implement the contract's exponent-binade,
    quotient/remainder, carry and IEEE-encoding algorithm.
    """
    if not value:
        return 'ok', 0
    negative = value < 0
    magnitude = abs(value)
    if magnitude <= ZERO_MID:
        result = 0
    elif magnitude >= OVER_MID:
        result = INF
    else:
        result = bits(float(magnitude))
        assert 0 < result <= MAX
    lower, upper = ieee_cell(result)
    certify_cell(magnitude, lower, upper, result & 1 == 0)
    if result == INF:
        return ('overflow-' if negative else 'overflow+'), result | (SIGN if negative else 0)
    if result == 0:
        return ('underflow-' if negative else 'underflow+'), 0 if canonical_zero else (SIGN if negative else 0)
    return 'ok', result | (SIGN if negative else 0)


def floor_log2(value: F) -> int:
    assert value > 0
    guess = value.numerator.bit_length() - value.denominator.bit_length()
    if value < p2(guess):
        guess -= 1
    assert p2(guess) <= value < p2(guess + 1)
    return guess


def extended(value: F) -> tuple[bool, int, int]:
    if not value:
        return False, 0, 0
    magnitude = abs(value)
    exponent = floor_log2(magnitude) - 52
    significand = round(magnitude / p2(exponent))
    if significand == 1 << 53:
        significand //= 2
        exponent += 1
    assert 1 << 52 <= significand < 1 << 53
    return value < 0, significand, exponent


def extended_value(parts: tuple[bool, int, int]) -> F:
    negative, significand, exponent = parts
    return (-1 if negative else 1) * significand * p2(exponent)


def extended_cell(parts: tuple[bool, int, int]) -> tuple[F, F]:
    _, significand, exponent = parts
    assert significand
    value = significand * p2(exponent)
    previous = value - p2(exponent - int(significand == 1 << 52))
    following = value + p2(exponent)
    return (previous + value) / 2, (value + following) / 2


def certify_decimal(value: F, extended_parts: tuple[bool, int, int] | None = None) -> None:
    """Two precision division checks; exact midpoint ties never judged by epsilon.

    Every non-tie included scalar decision has a gap exceeding twice the
    decimal half-ULP radius at each precision. Exact integer/Fraction cells,
    rather than dual-precision agreement, are rounding authority.
    """
    if not value:
        STATS['decimal_exact_zero'] += 1
        return
    magnitude = abs(value)
    if extended_parts is not None:
        lower, upper = extended_cell(extended_parts)
        even = extended_parts[1] & 1 == 0
    else:
        _, encoded = ieee(magnitude)
        lower, upper = ieee_cell(encoded)
        even = encoded & 1 == 0
    tie = certify_cell(magnitude, lower, upper, even)
    for precision in (160, 240):
        with localcontext() as ctx:
            ctx.prec = precision
            ctx.rounding = ROUND_HALF_EVEN
            observed = Decimal(magnitude.numerator) / Decimal(magnitude.denominator)
        radius = p10(observed.adjusted() - precision + 1) / 2
        observed_exact = F(observed)
        assert abs(observed_exact - magnitude) <= radius
        if not tie:
            distances = [magnitude - lower]
            if upper is not None:
                distances.append(upper - magnitude)
            assert 2 * radius < min(distances), (value, precision)
        STATS[f'decimal_divisions_{precision}'] += 1
    STATS['decimal_exact_tie' if tie else 'decimal_non_tie_separated'] += 1


def row(op: str, family: str, **fields: object) -> dict[str, str]:
    result = dict.fromkeys(COLS, '-')
    result.update(op=op, family=family, reachability='private-primitive')
    for name, value in fields.items():
        assert name in result, name
        result[name] = str(value)
    return result


def coupon(coefficient: int, scale: int) -> F:
    assert 0 <= coefficient <= LIMIT and 0 <= scale <= 38
    return F(coefficient, 10 ** scale)


def source_amount(kind: str, coefficient: int, scale: int, r: int, s: int, redeem: bool) -> F:
    c = coupon(coefficient, scale)
    assert 1 <= r <= s <= 184
    if kind == 'Regular':
        dividend = c / 2
    else:
        assert kind in ('Short', 'Long') and r < s
        raw = c * (r if kind == 'Short' else s + r) / (2 * s)
        units = raw * 10 ** 6
        dividend = F((2 * units.numerator + units.denominator) // (2 * units.denominator), 10 ** 6)
    return dividend + (100 if redeem else 0)


def source_dates(kind: str) -> str:
    issue = '1998-12-07' if kind == 'Regular' else '1999-05-07'
    first = '1999-12-07' if kind == 'Long' else '1999-06-07'
    return '|'.join(('1998-12-07', issue, '1999-06-07', first))


def checked_pv(kind: str, coefficient: int, scale: int, r: int, s: int,
               redeem: bool, discount: int, family: str, alternate: str = '-') -> dict[str, str]:
    assert 0 < discount <= MAX
    source = source_amount(kind, coefficient, scale, r, s, redeem)
    exact = source * binary(discount)
    status, encoded = ieee(exact)
    certify_decimal(exact)
    assert source.numerator.bit_length() <= 148 and source.denominator.bit_length() <= 128
    dates = source_dates(kind)
    return row('PV', family, source=kind, c_coef=coefficient, c_scale=scale,
               r=s if kind == 'Regular' else r, s=s, redeem=int(redeem), dates=dates, discount_bits=hx(discount),
               exact_n=source.numerator, exact_d=source.denominator,
               status=status, expected_bits=hx(encoded), alternate_bits=alternate,
               detail='source-coefficient-times-admitted-discount;not-a-public-libm-observation')


def pv_rows():
    discounts = (1, 2, 3, (1 << 52) - 1, 1 << 52, ONE - 1, ONE, ONE + 1,
                 bits(.5), bits(2.), MAX)
    for scale, coefficient, kind, redeem, discount in product(
            range(39), (1, 17, LIMIT), ('Regular', 'Short', 'Long'), (False, True), discounts):
        yield checked_pv(kind, coefficient, scale, 31, 182, redeem, discount, 'FULL-DECIMAL-SCALES')
    for kind, redeem, discount in product(('Regular', 'Short', 'Long'), (False, True), discounts):
        yield checked_pv(kind, 0, 0, 31, 182, redeem, discount, 'ZERO-DIVIDEND-REDEMPTION')
    # Integer/decimal source controls deliberately straddle IEEE tie/carry cells.
    for coefficient, redeem, discount in product((1, 3, 5, 7, (1 << 54) - 1,
                                                (1 << 54) - 201), (False, True), discounts):
        yield checked_pv('Regular', coefficient, 0, 31, 182, redeem, discount, 'TIES-CARRY')
    # Exact IEEE overflow midpoint and subnormal/normal crossover cells.
    boundaries = (((1 << 54) - 1, bits(2.**971) - 1),
                  ((1 << 54) - 1, bits(2.**971)),
                  ((1 << 54) - 1, bits(2.**971) + 1),
                  ((1 << 53) - 1, 1), ((1 << 53) - 3, 1))
    for coefficient, discount in boundaries:
        yield checked_pv('Regular', coefficient, 0, 31, 182, False, discount, 'IEEE-RANGE-MIDPOINTS')
    # Finite canonical source C=1+1e-38 gives exact (100.5+5e-39)*MIN_SUB.
    coefficient = 10 ** 38 + 1
    source = source_amount('Regular', coefficient, 38, 31, 182, True)
    staged = extended_value(extended(source * binary(1)))
    _, wrong = ieee(staged)
    correct = checked_pv('Regular', coefficient, 38, 31, 182, True, 1,
                         'SUBNORMAL-DOUBLE-ROUND-DISCRIMINATOR', hx(wrong))
    assert correct['expected_bits'] != correct['alternate_bits']
    yield correct
    # Exact decimal source versus rounding the coefficient before multiplication.
    for kind, coefficient, scale, redeem in (('Regular', 2, 6, False), ('Long', 6, 0, True)):
        source = source_amount(kind, coefficient, scale, 31, 182, redeem)
        source_status, rounded_source = ieee(source)
        assert source_status == 'ok'
        _, wrong = ieee(binary(rounded_source) * binary(ONE - 1))
        correct = checked_pv(kind, coefficient, scale, 31, 182, redeem, ONE - 1,
                             'EARLY-COEFFICIENT-DISCRIMINATOR', hx(wrong))
        assert correct['expected_bits'] != correct['alternate_bits']
        yield correct
    # Same-date grouping precedes one PV seed, not two independently rounded seeds.
    dividend = source_amount('Long', 6, 0, 31, 182, False)
    _, dividend_bits = ieee(dividend * binary(ONE - 1))
    _, principal_bits = ieee(F(100) * binary(ONE - 1))
    _, wrong = ieee(binary(dividend_bits) + binary(principal_bits))
    correct = checked_pv('Long', 6, 0, 31, 182, True, ONE - 1,
                         'GROUPED-REDEMPTION-DISCRIMINATOR', hx(wrong))
    assert correct['expected_bits'] != correct['alternate_bits']
    yield correct


def add_months(value: date, months: int) -> date:
    serial = value.year * 12 + value.month - 1 + months
    year, month0 = divmod(serial, 12)
    return date(year, month0 + 1, min(value.day, calendar.monthrange(year, month0 + 1)[1]))


PERIODS = ((date(2023, 1, 1), date(2023, 7, 1)),
           (date(2024, 1, 1), date(2024, 7, 1)),
           (date(2023, 4, 1), date(2023, 10, 1)),
           (date(2023, 3, 1), date(2023, 9, 1)))
assert tuple((b - a).days for a, b in PERIODS) == (181, 182, 183, 184)


def coordinate(previous: date, following: date, r: int, j: int) -> tuple[int, int, str]:
    s = (following - previous).days
    assert 1 <= r <= s <= 184 and 0 <= j < 20000
    settlement = following - timedelta(days=r)
    payment = add_months(following, 6 * j)
    assert previous <= settlement < following <= payment
    assert add_months(payment, -6 * j) == following
    return r + j * s, s, '|'.join(map(date.isoformat, (previous, settlement, following, payment)))


def z_record(g: int, previous: date, following: date, r: int, j: int, family: str):
    assert (g & ~SIGN) < INF
    L, s, dates = coordinate(previous, following, r, j)
    exact = -binary(g) * F(L, 2 * s)
    status, result = ieee(exact, canonical_zero=True)
    assert status in ('ok', 'underflow+', 'underflow-')
    certify_decimal(exact)
    sig, exponent = decompose(g & ~SIGN)
    if exact:
        numer, denom = abs(sig * L), 2 * s
    else:
        numer, denom = 0, 1
    return row('Z', family, r=r, s=s, j=j, dates=dates, g_bits=hx(g),
               y_bits=hx(g) if abs(binary(g)) < p2(-52) else '-',
               exact_n=numer, exact_d=denom, pow2=exponent, negative=int(exact < 0),
               status='ok', expected_bits=hx(result),
               detail='direct-signed-exponent-RN;internal-rounded-zero-is-allowed')


def z_rows():
    previous, following = PERIODS[0]
    for field, tail, negative in product(range(1034), (1, (1 << 52) - 1), (False, True)):
        encoded = (field << 52) | tail | (SIGN if negative else 0)
        observed = binary(encoded)
        if -64 <= observed <= 1400:
            yield z_record(encoded, previous, following, 181, 0, 'LOG-SIGNIFICAND-BINADE-CENSUS')
    anchors = (0, SIGN, 1, SIGN | 1, 3, SIGN | 3, (1 << 52) - 1, (1 << 52),
               bits(2.**-53), bits(-2.**-53), bits(1.), bits(-1.), bits(1400.), bits(-64.))
    for previous, following in PERIODS:
        s = (following - previous).days
        for r in range(1, s + 1):
            for g, j in product(anchors, (0, 1, 17, 40)):
                yield z_record(g, previous, following, r, j, 'ALL-CIVIL-OFFSETS-EXACT-COORDINATES')
    for g in anchors:
        yield z_record(g, date(1, 1, 1), date(1, 7, 1), 181, 19996, 'MAXIMUM-DATE-SPAN-AMPLIFICATION')
    # Exact189/181 differs from RN(8/181)+1; choose represented g=2.
    exact = z_record(bits(2.), *PERIODS[0], 8, 1, 'NO-EARLY-QUASI-RATIO-ROUNDING')
    _, partial = ieee(F(8, 181))
    _, lambda_wrong = ieee(binary(partial) + 1)
    _, wrong = ieee(-binary(bits(2.)) * binary(lambda_wrong) / 2)
    assert exact['expected_bits'] != hx(wrong)
    exact['alternate_bits'] = hx(wrong)
    yield exact


def ai_value(branch: str, coefficient: int, scale: int, r: int, s: int, t: int, s2: int) -> F:
    c = coupon(coefficient, scale)
    assert 0 <= t and 1 <= r <= s <= 184 and 1 <= s2 <= 184
    assert r < s or branch.startswith('Regular')
    if branch == 'RegularCum':
        return c * F(t, 2 * s)
    if branch == 'RegularEx':
        return c * F(t - s, 2 * s)
    if branch == 'ShortCum' or branch == 'LongFirst':
        return c * F(t, 2 * s)
    if branch == 'ShortEx':
        return c * F(t - r, 2 * s)
    if branch == 'LongSecondCum':
        return c * (F(r, s) + F(t, s2)) / 2
    assert branch == 'LongSecondEx'
    return c * F(t - s2, 2 * s2)


def ai_rows():
    # Original source dates fix r=31,s=182,s2=183, independently of price.
    for branch in ('RegularCum', 'RegularEx', 'ShortCum', 'ShortEx',
                   'LongFirst', 'LongSecondCum', 'LongSecondEx'):
        duration = 31 if branch.startswith('Short') or branch == 'LongFirst' else (183 if branch.startswith('LongSecond') else 182)
        times = sorted({0, 1, duration // 2, duration - 1})
        if branch.endswith('Ex'):
            times.remove(0)  # Compatible cutoff cannot be strictly before the interval start.
        for scale, coefficient, t in product(range(39), (1, 17, LIMIT), times):
            source_r = 182 if branch.startswith('Regular') else 31
            exact = ai_value(branch, coefficient, scale, source_r, 182, t, 183)
            status, result = ieee(exact)
            assert status == 'ok'
            certify_decimal(exact)
            assert abs(exact) < p2(127)
            if exact:
                assert abs(exact) > p2(-143)
            yield row('AI', 'SEVEN-SOURCE-BRANCHES-ALL-SCALES', source='SourceAccrued',
                      c_coef=coefficient, c_scale=scale, r=source_r, s=182, s2=183, t=t,
                      dates=source_dates('Regular' if branch.startswith('Regular') else ('Short' if branch.startswith('Short') else 'Long')),
                      ai_branch=branch, exact_n=exact.numerator, exact_d=exact.denominator,
                      status=status, expected_bits=hx(result),
                      detail='source-raw-rational;no-published-dividend-subtraction')
    for branch in ('RegularCum', 'RegularEx', 'ShortCum', 'ShortEx', 'LongFirst', 'LongSecondCum', 'LongSecondEx'):
        source_r = 182 if branch.startswith('Regular') else 31
        exact = ai_value(branch, 0, 38, source_r, 182, 1, 183)
        assert not exact
        certify_decimal(exact)
        yield row('AI', 'CANONICAL-SOURCE-ZERO', source='SourceAccrued', c_coef=0,
                  c_scale=38, r=source_r, s=182, t=1, s2=183, ai_branch=branch,
                  dates=source_dates('Regular' if branch.startswith('Regular') else ('Short' if branch.startswith('Short') else 'Long')),
                  exact_n=0, exact_d=1, status='ok', expected_bits=hx(0),
                  detail='exact-zero-canonical-positive')


def delta(y: int) -> int:
    assert -2 < binary(y)
    status, result = ieee(F(2) + binary(y))
    assert status == 'ok' and 0 < result <= MAX
    return result


def exact_term(measure: str, pv: int, L: int, s: int, d: int) -> tuple[F, int, int, int, bool]:
    assert 0 <= pv <= MAX and 1 <= L < (1 << 22) and 1 <= s <= 184 and 0 < d <= MAX
    vp, ve = decompose(pv)
    dp, de = decompose(d)
    if measure == 'p1':
        numerator, denominator, shift, negative = vp * L, s * dp, ve - de, True
    elif measure == 'p2':
        numerator, denominator, shift, negative = vp * L * (L + s), s * s * dp * dp, ve - 2 * de, False
    else:
        assert measure == 'moment'
        numerator, denominator, shift, negative = vp * L, 2 * s, ve, False
    exact = F(numerator, denominator) * p2(shift) * (-1 if negative else 1)
    assert numerator.bit_length() <= 97 and denominator.bit_length() <= 122
    return exact, numerator, denominator, shift, negative


def term_record(measure: str, pv: int, y: int, previous: date, following: date,
                r: int, j: int, family: str):
    L, s, dates = coordinate(previous, following, r, j)
    d = delta(y)
    exact, numerator, denominator, shift, _ = exact_term(measure, pv, L, s, d)
    parts = extended(exact)
    if exact:
        lower, upper = extended_cell(parts)
        certify_cell(abs(exact), lower, upper, parts[1] & 1 == 0)
        certify_decimal(exact, parts)
    else:
        certify_decimal(exact)
    extended_observation = extended_value(parts)
    status, result = ieee(extended_observation)
    if pv:
        normalized = (-extended_observation if measure == 'p1' else extended_observation) / binary(pv)
        nstatus, nbits = ieee(normalized)
    else:
        nstatus, nbits = 'not-applicable', None
    return row('TERM', family, source='Regular', c_coef=2 if pv else 0, c_scale=0, redeem=0,
               r=r, s=s, j=j, dates=dates, y_bits=hx(y), discount_bits=hx(pv if pv else ONE),
               pv_bits=hx(pv), delta_bits=hx(d), measure=measure,
               exact_n=numerator, exact_d=denominator, pow2=shift,
               negative=int(parts[0]), significand=parts[1], exponent=parts[2],
               status=status, expected_bits=hx(result),
               normalized_status=nstatus, normalized_bits=hx(nbits) if nbits is not None else '-',
               detail=('couponC2-sourceA1-accepted-discount-equals-PV' if pv else 'couponC0-sourceA0-accepted-discount1') + ';normalized-scalar-control-not-a-risk-report')


def term_rows():
    # All finite positive exponent fields, bottom/top significands, three measures.
    for field, tail, measure in product(range(2047), (1, (1 << 52) - 1), ('p1', 'p2', 'moment')):
        pv = (field << 52) | tail
        yield term_record(measure, pv, 0, *PERIODS[0], 181, 0, 'PV-ALL-EXPONENT-FIELDS')
    ys = (bits(-2.) - 1, bits(-1.), SIGN | 1, 0, 1, bits(2.**-51),
          bits(2.), bits(1e170), MAX)
    pvs = (1, 3, (1 << 52) - 1, 1 << 52, ONE - 1, ONE, ONE + 1, MAX)
    for y, pv, measure in product(ys, pvs, ('p1', 'p2', 'moment')):
        for previous, following, r, j in ((PERIODS[2][0], PERIODS[2][1], 1, 0),
                                          (PERIODS[2][0], PERIODS[2][1], 8, 1),
                                          (date(1, 1, 1), date(1, 7, 1), 181, 19996)):
            yield term_record(measure, pv, y, previous, following, r, j, 'EXACT-DELTA-WIDTH-AND-RANGE')
    rng = random.Random(0x47494c544e554d)
    for _ in range(512):
        previous, following = PERIODS[rng.randrange(4)]
        s = (following - previous).days
        pv = rng.randrange(1, MAX + 1)
        y = ys[rng.randrange(len(ys))]
        yield term_record(('p1', 'p2', 'moment')[rng.randrange(3)], pv, y, previous,
                          following, rng.randrange(1, s + 1), rng.randrange(41),
                          'FIXED-SEED-QUOTIENT-REMAINDERS')
    for measure in ('p1', 'p2', 'moment'):
        yield term_record(measure, 0, 0, *PERIODS[0], 181, 0, 'ZERO-SOURCE-COEFFICIENT')
    # All three measures can have multiplier3/2, including normalized53 ties
    # and a carry to the following binade, with exact represented PV inputs.
    for measure, j in (('p1', 2), ('p2', 1), ('moment', 2)):
        for pv in (ONE + 1, ONE + 3, bits(4 / 3)):
            yield term_record(measure, pv, 0, *PERIODS[0], 181, j,
                              'EXTENDED-SIGNIFICAND-TIES-AND-CARRY')
    # Risk deliberately retains53 bits before getter IEEE admission. Unlike
    # source PV, replacing those two stages by direct IEEE is the wrong rule.
    pv = 0x000fffffffffed31
    control = term_record('p2', pv, 0, *PERIODS[2], 1, 0,
                          'RETAINED-53-THEN-IEEE-DISCRIMINATOR')
    direct = ieee(exact_term('p2', pv, 1, 183, delta(0))[0])[1]
    control['alternate_bits'] = hx(direct)
    assert control['expected_bits'] != control['alternate_bits']
    yield control


def rle(values: list[int]) -> str:
    groups: list[tuple[int, int]] = []
    for value in values:
        if groups and groups[-1][0] == value:
            groups[-1] = (value, groups[-1][1] + 1)
        else:
            groups.append((value, 1))
    return ';'.join(f'{hx(value)}*{count}' for value, count in groups)


def fnv(data: bytes) -> str:
    state = 0xcbf29ce484222325
    for byte in data:
        state = ((state ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{state:016x}'


def reduction_record(coefficient: int, scale: int, y: int, previous: date,
                     following: date, r: int, discounts: list[int], family: str):
    count = len(discounts)
    _, s, dates = coordinate(previous, following, r, count - 1)
    assert 1 <= count <= 19997
    d = delta(y)
    pvs = []
    for j, discount in enumerate(discounts):
        assert 0 < discount <= MAX
        source = source_amount('Regular', coefficient, scale, 31, 182, j == count - 1)
        status, pv = ieee(source * binary(discount))
        assert status == 'ok', (family, j, status)
        pvs.append(pv)
    status, dirty = ieee(sum(map(binary, pvs), F(0)))
    assert status == 'ok' and dirty > 0
    retained: dict[str, F] = {}
    lengths, hashes = [], []
    for measure in ('p1', 'p2', 'moment'):
        components = [extended_value(extended(exact_term(measure, pv, r + j * s, s, d)[0]))
                      for j, pv in enumerate(pvs)]
        exact_sum = sum(components, F(0))
        coefficient_on_basis = abs(exact_sum) / p2(BASIS)
        assert coefficient_on_basis.denominator == 1
        integer = coefficient_on_basis.numerator
        assert integer.bit_length() <= 9365
        retained[measure] = exact_sum
        lengths.append(integer.bit_length())
        hashes.append(fnv(integer.to_bytes(WORDS * 8, 'little')))
    observed = dict(retained)
    observed.update(modified=-retained['p1'] / binary(dirty),
                    macaulay=retained['moment'] / binary(dirty),
                    convexity=retained['p2'] / binary(dirty))
    out = row('REDUCE', family, source='Regular', c_coef=coefficient, c_scale=scale,
              r=r, s=s, dates=dates, redeem=1, y_bits=hx(y), delta_bits=hx(d),
              bundle_count=count, recipes=rle(discounts), dirty_bits=hx(dirty),
              basis_bit_lengths=','.join(map(str, lengths)), basis_fnv64=','.join(hashes),
              status='ok', detail='rle-admitted-discounts;actual-same-period-coordinates;par100-final;no-public-libm-claim')
    for name in MEASURES:
        status, encoded = ieee(observed[name])
        out[f'{name}_status'], out[f'{name}_bits'] = status, hx(encoded)
    assert out['modified_status'] == out['macaulay_status'] == 'ok'
    assert out['convexity_status'] != 'overflow+'
    return out


def reduction_rows():
    ys = (bits(-2.) - 1, bits(-1.), 0, 1, bits(2.), bits(1e170), MAX)
    for count, y, discount in product((1, 2, 3, 19, 101), ys,
                                     (ONE - 1, ONE, ONE + 1, bits(2.**-500), bits(2.**500))):
        # Positive sourceC2 and par100 make every generated dirty representable.
        yield reduction_record(2, 0, y, *PERIODS[2], 1, [discount] * count,
                               'POSITIVE-SOURCE-RAW-NORMALIZED-INDEPENDENCE')
    for count, y in product((1, 2, 19, 101), ys):
        discounts = [bits(2.**(-500 + (j % 3) * 500)) for j in range(count)]
        yield reduction_record(LIMIT, 0, y, *PERIODS[3], 1, discounts,
                               'SOURCE-COEFFICIENT-AND-PV-EXPONENT-MIX')
    for y in (bits(-2.) - 1, 0, MAX):
        yield reduction_record(0, 0, y, date(1, 1, 1), date(1, 7, 1), 181,
                               [ONE] * 19997, 'FULL-DATE-SPAN-ZERO-COUPONS-POSITIVE-REDEMPTION')
    # Large last-period source amount rescues raw P'' while normalized view underflows.
    yield reduction_record(LIMIT, 0, bits(1e170), *PERIODS[3], 1,
                           [bits(2.**-8)], 'NORMALIZED-ONLY-CONVEXITY-UNDERFLOW')
    # Finite dirty with raw overflow; direct retained normalization remains finite.
    yield reduction_record(0, 0, 0, *PERIODS[0], 181, [bits(2.**1015)] * 19,
                           'RAW-MOMENT-OVERFLOW-NORMALIZED-FINITE')


def residual_rows():
    pairs = []
    for field in range(2047):
        positive = (field << 52) | 1
        pairs.extend(((positive, positive), (positive + 1, positive),
                      (positive, positive + 1)))
    anchors = (1, 2, 3, (1 << 52) - 1, 1 << 52, ONE - 1, ONE, ONE + 1,
               bits(2.**970) - 1, bits(2.**970), bits(2.**970) + 1, MAX - 1, MAX)
    pairs.extend(product(anchors, repeat=2))
    seen = set()
    for dirty, target in pairs:
        if (dirty, target) in seen:
            continue
        seen.add((dirty, target))
        assert 0 < dirty <= MAX and 0 < target <= MAX
        exact = binary(dirty) - binary(target)
        scale = int(binary(target) >= p2(970))
        status, result = ieee(exact * p2(-scale))
        ast, amount = ieee(exact)
        assert status == ast == 'ok'
        assert (result == 0) == (dirty == target)
        if exact:
            assert bool(result & SIGN) == (exact < 0)
        yield row('RESIDUAL', 'POSITIVE-TARGET-ALL-BINADES', dirty_bits=hx(dirty),
                  target_bits=hx(target), scale=scale, status=status, expected_bits=hx(result),
                  amount_status=ast, amount_bits=hx(amount),
                  detail='native-per100-positive-D-and-T;fixed-target-scale;no-amount-overflow')


def audit_serialized_rows(rows: list[dict[str, str]]) -> None:
    """Check emitted input/provenance fields, not only in-memory arithmetic.

    These checks catch a valid arithmetic result attached to the wrong source
    role, a zero source with an inadmissible factor, or a mislabelled date/index.
    They do not manufacture a claim that libm realizes a primitive observation.
    """
    expected_counts = {'PV': 7929, 'Z': 45020, 'AI': 2932, 'TERM': 13455,
                       'REDUCE': 208, 'RESIDUAL': 6305}
    assert dict(Counter(r['op'] for r in rows)) == expected_counts
    for record in rows:
        operation = record['op']
        if operation in ('PV', 'AI') and record['dates'] != '-':
            q0, issue, q1, first = map(date.fromisoformat, record['dates'].split('|'))
            assert (q1 - q0).days == int(record['s'])
            assert q0 <= issue < q1 and first in (q1, add_months(q1, 6))
            role = record['source'] if operation == 'PV' else (
                'Regular' if record['ai_branch'].startswith('Regular') else
                'Short' if record['ai_branch'].startswith('Short') else 'Long')
            assert (issue == q0) == (role == 'Regular')
            assert (first > q1) == (role == 'Long')
            assert (q1 - issue).days == int(record['r'])
            if operation == 'AI':
                t = int(record['t'])
                branch = record['ai_branch']
                if branch.startswith('Regular'):
                    settlement = q0 + timedelta(days=t)
                    assert q0 <= settlement < q1
                elif branch.startswith('Short') or branch == 'LongFirst':
                    settlement = issue + timedelta(days=t)
                    assert issue <= settlement < q1
                else:
                    settlement = q1 + timedelta(days=t)
                    assert q1 <= settlement < first
                    assert (first - q1).days == int(record['s2'])
                if branch.endswith('Ex'):
                    assert t > 0  # A compatible cutoff at the interval start exists.
        if operation in ('Z', 'TERM', 'REDUCE'):
            previous, settlement, following, payment = map(date.fromisoformat, record['dates'].split('|'))
            assert (following - previous).days == int(record['s'])
            assert (following - settlement).days == int(record['r'])
            assert previous <= settlement < following <= payment
            j = int(record['bundle_count']) - 1 if operation == 'REDUCE' else int(record['j'])
            assert add_months(payment, -6 * j) == following
        if operation == 'TERM':
            discount = int(record['discount_bits'], 16)
            pv = int(record['pv_bits'], 16)
            assert discount > 0
            c = coupon(int(record['c_coef']), int(record['c_scale']))
            status, reconstructed = ieee(c * binary(discount) / 2)
            assert status == 'ok' and reconstructed == pv
            assert int(record['delta_bits'], 16) == delta(int(record['y_bits'], 16))
            if not pv:
                assert record['normalized_status'] == 'not-applicable'
                assert record['normalized_bits'] == '-'
        if record['alternate_bits'] != '-':
            assert record['alternate_bits'] != record['expected_bits']
        assert record['reachability'] == 'private-primitive'
    assert {int(r['c_scale']) for r in rows if r['op'] == 'PV'} == set(range(39))
    assert {int(r['s']) for r in rows if r['op'] == 'Z'} == {181, 182, 183, 184}
    assert {r['ai_branch'] for r in rows if r['op'] == 'AI'} == {
        'RegularCum', 'RegularEx', 'ShortCum', 'ShortEx', 'LongFirst',
        'LongSecondCum', 'LongSecondEx'}
    assert STATS['decimal_exact_tie'] > 0 and STATS['decimal_non_tie_separated'] > 0


def fixture_bytes(rows: list[dict[str, str]], generated_at: str, platform_text: str) -> bytes:
    source = Path(__file__).read_bytes()
    contract = CONTRACT.read_bytes()
    assert hashlib.sha256(contract).hexdigest() == CONTRACT_SHA, 'frozen gilt contract changed'
    requirements = ''.join(line + '\n' for line in (ROOT / 'docs/REQUIREMENTS.md').read_text().splitlines()
                           if line.startswith('| FR-GILT-')).encode()
    assert len(requirements.splitlines()) == 24
    records = ''.join('\t'.join(record[name] for name in COLS) + '\n' for record in rows).encode()
    body = ('\t'.join(COLS) + '\n').encode() + records
    headers = [
        'schema: 1', 'fixture-set: M8-GILT-NUMERIC-ORACLE', 'domain: bonds',
        'description: Independent exact source, exponent, IEEE, extended risk and residual primitive oracles',
        'generator: validation/generator/gilt_numeric_oracles.py',
        'reference: exact Fraction integers and adjacent-representable cell/parity certificates; no Rust financial reads',
        'reference-version: Python standard library',
        'reference-source: docs/M8_GILT_CONTRACT.md',
        f'generated-at: {generated_at}',
        f'platform: {platform_text}',
        'reproduction: python3 -I validation/generator/gilt_numeric_oracles.py --check',
        'tolerance: TOL-E0; exact staged primitive controls only; no libm/ideal-price/runtime acceptance',
        f'generator-sha256: {hashlib.sha256(source).hexdigest()}',
        f'generator-fnv64: {fnv(source)}',
        f'contract-m8-gilt-contract-sha256: {CONTRACT_SHA}',
        f'contract-m8-gilt-contract-fnv64: {fnv(contract)}',
        f'requirements-sha256: {hashlib.sha256(requirements).hexdigest()}',
        f'requirements-fnv64: {fnv(requirements)}',
        f'record-count: {len(rows)}', f'column-count: {len(COLS)}',
        f'body-sha256: {hashlib.sha256(body).hexdigest()}', f'body-fnv64: {fnv(body)}',
        f'records-sha256: {hashlib.sha256(records).hexdigest()}',
        'operation-counts: ' + json.dumps(dict(sorted(Counter(r['op'] for r in rows).items())), separators=(',', ':')),
        'family-counts: ' + json.dumps(dict(sorted(Counter(r['family'] for r in rows).items())), separators=(',', ':')),
        'status-counts: ' + json.dumps(dict(sorted(Counter(r['status'] for r in rows).items())), separators=(',', ':')),
        'reachability-counts: ' + json.dumps(dict(sorted(Counter(r['reachability'] for r in rows).items())), separators=(',', ':')),
        'crosscheck-counts: ' + json.dumps(dict(sorted(STATS.items())), separators=(',', ':')),
        'decimal-precisions: 160,240; exact ties use parity authority; all nonties separated by twice decimal half-ULP',
        'risk-basis: 2^-8192; exactly243-u64-word1944-byte-little-endian absolute magnitudes; p1-negative-others-positive',
        'source-date-encoding: dates=q_previous|settlement|q_next|payment for Z/TERM/REDUCE; PV/AI dates=Q0|I|Q1|F',
        'reduction-recipe-encoding: semicolon runs admitted_discount_bits*count; couponC/2 each, par100 at final, localj=0..count-1',
        'primitive-reachability: accepted log/discount/PV inputs do not assert a public yield/libm observation; no forged GiltRisk report',
    ]
    return ('\n'.join(headers) + '\n\n').encode() + body


def generate(generated_at: str, platform_text: str) -> bytes:
    STATS.clear()
    records = []
    for producer in (pv_rows, z_rows, ai_rows, term_rows, reduction_rows, residual_rows):
        group = list(producer())
        print(json.dumps({'operation': group[0]['op'], 'rows': len(group)}, sort_keys=True), flush=True)
        records.extend(group)
    for index, record in enumerate(records, 1):
        record['id'] = f'GNUM-{index:06d}'
        assert set(record) == set(COLS)
        assert all('\t' not in value and '\n' not in value and value for value in record.values())
    assert len({r['id'] for r in records}) == len(records)
    assert {r['op'] for r in records} == {'PV', 'Z', 'AI', 'TERM', 'REDUCE', 'RESIDUAL'}
    audit_serialized_rows(records)
    return fixture_bytes(records, generated_at, platform_text)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='compare exact bytes without rewriting the fixture')
    args = parser.parse_args()
    generated_at = datetime.now(timezone.utc).isoformat(timespec='seconds').replace('+00:00', 'Z')
    platform_text = f'{platform.system()} {platform.machine()}; Python {platform.python_version()}'
    if args.check and OUT.exists():
        metadata = dict(line.split(': ', 1) for line in OUT.read_text().split('\n\n', 1)[0].splitlines())
        generated_at, platform_text = metadata['generated-at'], metadata['platform']
    data = generate(generated_at, platform_text)
    if args.check:
        if not OUT.exists() or OUT.read_bytes() != data:
            raise SystemExit(f'NOT REPRODUCIBLE: {OUT.relative_to(ROOT)}')
    else:
        if OUT.exists() and OUT.read_bytes() != data:
            raise SystemExit('refusing to overwrite an existing frozen fixture; write a new additive version')
        OUT.write_bytes(data)
    print(json.dumps({'path': str(OUT.relative_to(ROOT)), 'bytes': len(data),
                      'sha256': hashlib.sha256(data).hexdigest(), 'check': args.check}, sort_keys=True), flush=True)


if __name__ == '__main__':
    main()
