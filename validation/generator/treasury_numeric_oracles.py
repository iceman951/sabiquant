#!/usr/bin/env python3
"""Independent exact US Treasury numerical primitive oracles (stdlib only).

Authored from the adopted contract pair (docs/M8_US_TREASURY_CONTRACT.md N1-N8
and docs/M8_US_TREASURY_CASH_CONTRACT.md T3) before any Treasury Rust. No Rust,
production output or QuantLib source is read. Every expected value is the
contract's prescribed observation, computed with exact Python integers and
Fractions, and certified by an independent route:

* p256 directed endpoints by an exact operand-product/quotient check;
* extended53 and IEEE binary64 results by an exact adjacent-cell/parity
  certificate, plus CPython's correctly rounded Fraction->float for IEEE and a
  dual-precision Decimal (160/240 digits) cross-check where separated.

Rows are labelled by reachability:
* dated-financial: built from a validated date lattice (main E step 5), shape,
  D<=I<F, I<=S<M and a valid DecimalAmount coupon;
* coordinate-primitive: a valid coupled (y, r, s, j) numeric coordinate inside
  the contract domain, without asserting one particular dated instrument;
* primitive-superset: rounding-primitive inputs that need not be reachable
  (constructed endpoints, PV/factor pairs, j above the Date-range maximum,
  arbitrary dirty/target doubles). They make no financial completeness claim.

The comparison for every row is exact staging (TOL-E0; TOL-E1 for the AI
binary views), never an ideal-price certificate.
"""
from __future__ import annotations

import argparse
import calendar
from collections import Counter
from datetime import date, datetime, timedelta, timezone
from decimal import Context, Decimal
from fractions import Fraction as F
import hashlib
import json
from pathlib import Path
import platform
import random
import struct

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'validation/fixtures/bonds/treasury-numeric-oracle.fixture'
MAIN = ROOT / 'docs/M8_US_TREASURY_CONTRACT.md'
CASH = ROOT / 'docs/M8_US_TREASURY_CASH_CONTRACT.md'
REQUIREMENTS = ROOT / 'docs/REQUIREMENTS.md'
MAIN_SHA = '50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93'
CASH_SHA = '25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585'

P = 256
B1074 = 1 << 1074
SIGN = 1 << 63
INF_BITS = 0x7ff0000000000000
MAX_BITS = INF_BITS - 1
I128_MAX = (1 << 127) - 1
BASIS = 8192
WORDS = 243
J_CAP = 19999
DATE_J_MAX = 19996
EXACT_BIT_BUDGET = 1 << 20

INPUT_COLS = ('op dates eom y_bits r s j c_coef c_scale basis tie '
              'amount_num amount_den low_point factor_hat pv_bits measure dirty_bits '
              'target_kind target_bits ai_bits').split()
OUTPUT_COLS = ('remaining_periods status expected_bits alternate_bits tie_flag carry_flag b_interval q_interval '
               'delta_bits qhat_bits seed_bits factor_lo factor_hi weights width_ok relation '
               'straddle containment tree_alt peak_bits ext_value num_bits den_bits e_diff '
               'bundle_count positive_count first_fail dirty_status dirty_bits_out sum_bits '
               'sum_fnv64 p1_status p1_bits p2_status p2_bits moment_status moment_bits '
               'modified_status modified_bits macaulay_status macaulay_bits convexity_status '
               'convexity_bits pieces raw_ai published_count raw_ai_bits published_bits '
               'scale residual_bits amount_bits dirty_target_bits').split()
# Classification metadata cannot make otherwise duplicate actual inputs unique.
COLS = ['id', 'family', 'profile', 'case', 'reachability', 'shape'] + INPUT_COLS + OUTPUT_COLS
STATS: Counter = Counter()


# ---------------------------------------------------------------- utilities

def fnv(data: bytes) -> str:
    state = 0xcbf29ce484222325
    for byte in data:
        state = ((state ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{state:016x}'


def p2(e: int) -> F:
    return F(1 << e) if e >= 0 else F(1, 1 << -e)


def hb(bits: int) -> str:
    return f'0x{bits:016x}'


def float_bits(x: float) -> int:
    return struct.unpack('>Q', struct.pack('>d', x))[0]


def bits_float(b: int) -> float:
    return struct.unpack('>d', struct.pack('>Q', b))[0]


def decode(b: int) -> tuple[int, int, int]:
    """(sign, significand, exponent) of a finite binary64 bit pattern."""
    field, tail = (b >> 52) & 2047, b & ((1 << 52) - 1)
    assert field != 2047
    if field == 0:
        return (b >> 63), tail, -1074
    return (b >> 63), tail | (1 << 52), field - 1075


def binary_value(b: int) -> F:
    sign, m, e = decode(b)
    value = F(m) * p2(e)
    return -value if sign else value


def point(m: int, e: int) -> F:
    return F(m) * p2(e)


def ptxt(m: int, e: int, width: int) -> str:
    return f'{m:0{width}x}:{e}'


def le(a: tuple[int, int], b: tuple[int, int]) -> bool:
    """Exact a <= b for points (m, e)."""
    e0 = min(a[1], b[1])
    return (a[0] << (a[1] - e0)) <= (b[0] << (b[1] - e0))


def floor_log2(n: int, d: int) -> tuple[int, int]:
    """k = floor(log2(n/d)) by bit counts and one aligned comparison; peak bits."""
    k = n.bit_length() - d.bit_length()
    if k >= 0:
        shifted = d << k
        below = n < shifted
        peak = shifted.bit_length()
    else:
        shifted = n << -k
        below = shifted < d
        peak = shifted.bit_length()
    return (k - 1 if below else k), max(peak, n.bit_length(), d.bit_length())


# ------------------------------------------------- N1 p256 directed rounding

def norm_p256(n: int, d: int, e: int, up: bool) -> tuple[int, int, int]:
    """Contract N1 point normalization of positive (n/d)*2^e; (m, exponent, peak)."""
    assert n > 0 and d > 0
    k, peak = floor_log2(n, d)
    unit = k - (P - 1)
    if unit < 0:
        num, den = n << -unit, d
    else:
        num, den = n, d << unit
    peak = max(peak, num.bit_length(), den.bit_length())
    f, rem = divmod(num, den)
    assert (1 << (P - 1)) <= f < (1 << P)
    if up and rem:
        f += 1
        if f == 1 << P:
            f, unit = 1 << (P - 1), unit + 1
    peak = max(peak, f.bit_length())
    # Independent adjacent-grid certificate for n/d, before the outer 2^e.
    # The predecessor of a binade's first significand has HALF the new unit.
    lhs = n << max(0, -unit)
    rhs_scale = d << max(0, unit)
    if up:
        assert f * rhs_scale >= lhs
        if f == 1 << (P - 1):
            assert (2 * f - 1) * rhs_scale < 2 * lhs
        else:
            assert (f - 1) * rhs_scale < lhs
    else:
        assert f * rhs_scale <= lhs < (f + 1) * rhs_scale
    STATS['p256_directed_certificates'] += 1
    return f, e + unit, peak


class Interval:
    """Positive p256 interval; peak is the endpoint-operation temporary bits,
    seed_peak the 34-word seed-conversion temporary bits."""
    __slots__ = ('lo', 'hi', 'c', 'o', 'peak', 'seed_peak')

    def __init__(self, lo, hi, c, o, peak, seed_peak=0):
        self.lo, self.hi, self.c, self.o, self.peak, self.seed_peak = lo, hi, c, o, peak, seed_peak


ONE = Interval((1 << (P - 1), -(P - 1)), (1 << (P - 1), -(P - 1)), 0, 0, 0)


def convert(n: int, d: int, e: int) -> Interval:
    lo = norm_p256(n, d, e, False)
    hi = norm_p256(n, d, e, True)
    exact = lo[:2] == hi[:2]
    return Interval(lo[:2], hi[:2], 0 if exact else 1, 0, 0, max(lo[2], hi[2]))


def multiply(a: Interval, b: Interval) -> Interval:
    lo = norm_p256(a.lo[0] * b.lo[0], 1, a.lo[1] + b.lo[1], False)
    hi = norm_p256(a.hi[0] * b.hi[0], 1, a.hi[1] + b.hi[1], True)
    # Per-operation containment: exact low/high products lie inside.
    assert le(lo[:2], (a.lo[0] * b.lo[0], a.lo[1] + b.lo[1]))
    assert le((a.hi[0] * b.hi[0], a.hi[1] + b.hi[1]), hi[:2])
    STATS['per_operation_products'] += 1
    return Interval(lo[:2], hi[:2], a.c + b.c, a.o + b.o + 1, max(a.peak, b.peak, lo[2], hi[2]))


def reciprocal(q: Interval) -> Interval:
    """Divide exact 1 (p256 point 2^255 * 2^-255) by the interval."""
    one_m, one_e = 1 << (P - 1), -(P - 1)
    lo = norm_p256(one_m, q.hi[0], one_e - q.hi[1], False)
    hi = norm_p256(one_m, q.lo[0], one_e - q.lo[1], True)
    assert le((lo[0] * q.hi[0], lo[1] + q.hi[1]), (one_m, one_e))
    assert le((one_m, one_e), (hi[0] * q.lo[0], hi[1] + q.lo[1]))
    STATS['per_operation_quotients'] += 1
    return Interval(lo[:2], hi[:2], q.c, q.o + 1, max(q.peak, lo[2], hi[2]))


def l2r_power(base: Interval, j: int) -> Interval:
    if j == 0:
        return ONE
    acc = base
    for bit in bin(j)[3:]:
        acc = multiply(acc, acc)
        if bit == '1':
            acc = multiply(acc, base)
    return acc


def cumulative_power(base: Interval, j: int) -> Interval:
    acc = ONE if j == 0 else base
    for _ in range(max(0, j - 1)):
        acc = multiply(acc, base)
    return acc


# --------------------------------------------------------- N2 exact seeds

def seed_numerators(y_bits: int, r: int, s: int) -> tuple[int, int, int]:
    """Exact Y = y*2^1074 and the b/q/delta numerators (denominators 2, 2s, 1)."""
    sign, m, e = decode(y_bits)
    big_y = (m << (e + 1074)) * (-1 if sign else 1)
    nb = 2 * B1074 + big_y
    nq = 2 * s * B1074 + r * big_y
    assert nb > 0 and nq > 0
    assert nb < 1 << 2107 and nq < 1 << 2107
    assert nb >= 1 << 1022 and nq >= 1 << 1022
    return nb, nq, big_y


def seeds(y_bits: int, r: int, s: int):
    nb, nq, _ = seed_numerators(y_bits, r, s)
    b = convert(nb, 2, -1074)
    q = convert(nq, 2 * s, -1074)
    return nb, nq, b, q


def valid_yield(y_bits: int) -> bool:
    if (y_bits >> 52) & 2047 == 2047:
        return False
    return binary_value(y_bits) > -2


def canonical_yield(y_bits: int) -> int:
    return 0 if y_bits == SIGN else y_bits


# ------------------------------------------------ extended53 and IEEE RN

def rne53_point(m: int, e: int) -> tuple[int, int, bool, bool]:
    """Unbounded-exponent RNE53 of a p256 point: discard 203 bits."""
    drop = m.bit_length() - 53
    assert drop == 203
    k, rem = m >> drop, m & ((1 << drop) - 1)
    half = 1 << (drop - 1)
    tie = rem == half
    if rem > half or (tie and k & 1):
        k += 1
    carry = k == 1 << 53
    if carry:
        k, drop = 1 << 52, drop + 1
    return k, e + drop, tie, carry


def rne53_ratio(n: int, d: int, e: int) -> tuple[int, int, bool, bool, int]:
    """Unbounded-exponent RNE53 of positive (n/d)*2^e; (sig, exp, tie, carry, peak)."""
    k, peak = floor_log2(n, d)
    unit = k - 52
    num, den = (n << -unit, d) if unit < 0 else (n, d << unit)
    peak = max(peak, num.bit_length(), den.bit_length())
    f, rem = divmod(num, den)
    assert (1 << 52) <= f < (1 << 53)
    complement = den - rem
    tie = rem == complement
    if rem > complement or (tie and f & 1):
        f += 1
    carry = f == 1 << 53
    if carry:
        f, unit = 1 << 52, unit + 1
    return f, e + unit, tie, carry, peak


def certify_ext53(value: F, sig: int, exp: int, tie: bool) -> None:
    """Adjacent extended53 cell certificate with parity."""
    assert (1 << 52) <= sig < (1 << 53)
    result = point(sig, exp)
    below = point(sig - 1, exp) if sig > 1 << 52 else point((1 << 53) - 1, exp - 1)
    above = point(sig + 1, exp)
    lower, upper = (below + result) / 2, (result + above) / 2
    assert lower <= value <= upper
    boundary = value in (lower, upper)
    assert tie == boundary
    if boundary:
        assert sig % 2 == 0
    STATS['ext53_cell_certificates'] += 1


def ieee_direct(n: int, d: int, e: int) -> tuple[str, int, bool, int]:
    """Contract N3 direct IEEE RN of positive (n/d)*2^e; (status, bits, tie, peak)."""
    k, peak = floor_log2(n, d)
    big_k = k + e
    if big_k > 1023:
        return 'overflow+', INF_BITS, False, peak
    if big_k < -1075:
        return 'underflow+', 0, False, peak
    unit = max(big_k - 52, -1074)
    shift = e - unit
    num, den = (n << shift, d) if shift >= 0 else (n, d << -shift)
    peak = max(peak, num.bit_length(), den.bit_length())
    f, rem = divmod(num, den)
    complement = den - rem
    tie = rem == complement
    if rem > complement or (tie and f & 1):
        f += 1
    if f == 1 << 53:
        f, unit = 1 << 52, unit + 1
    peak = max(peak, f.bit_length())
    if f == 0:
        return 'underflow+', 0, tie, peak
    if unit + 52 > 1023 and f >= 1 << 52:
        return 'overflow+', INF_BITS, tie, peak
    if f < 1 << 52:
        assert unit == -1074
        bits = f
    else:
        bits = ((unit + 52 + 1023) << 52) | (f - (1 << 52))
    assert bits < INF_BITS
    return 'ok', bits, tie, peak


OVER_MID = p2(1024) - p2(970)
ZERO_MID = p2(-1075)


def certify_ieee(value: F, status: str, bits: int) -> None:
    """CPython Fraction->float route plus exact adjacent-cell/parity certificate."""
    assert value > 0
    if status == 'overflow+':
        assert value >= OVER_MID
        STATS['ieee_overflow_certificates'] += 1
        return
    if status == 'underflow+':
        assert value <= ZERO_MID
        STATS['ieee_zero_certificates'] += 1
        return
    assert float_bits(float(value)) == bits
    result = binary_value(bits)
    lower = (binary_value(bits - 1) + result) / 2 if bits > 1 else ZERO_MID
    upper = OVER_MID if bits == MAX_BITS else (result + binary_value(bits + 1)) / 2
    assert lower <= value <= upper
    if value in (lower, upper):
        assert bits % 2 == 0
        STATS['ieee_exact_ties'] += 1
    STATS['ieee_cell_certificates'] += 1
    decimal_crosscheck(value, bits, lower, upper)


def decimal_crosscheck(value: F, bits: int, lower: F, upper: F) -> None:
    """Dual-precision Decimal route; asserted when separated from both midpoints."""
    margin = min(value - lower, upper - value)
    for digits in (160, 240):
        ctx = Context(prec=digits)
        approx = ctx.divide(Decimal(value.numerator), Decimal(value.denominator))
        error_bound = abs(F(approx)) * F(1, 10 ** (digits - 2))
        if margin > error_bound:
            assert float_bits(float(approx)) == bits
            STATS[f'decimal_separated_{digits}'] += 1
        else:
            STATS[f'decimal_tie_or_near_{digits}'] += 1


def ieee_signed(value: F, negative_label: bool = False) -> tuple[str, int]:
    """Signed IEEE RN with canonical +0 for exact zero (no tie bookkeeping)."""
    if value == 0:
        return 'ok', 0
    magnitude = abs(value)
    status, bits, _, _ = ieee_direct(magnitude.numerator, magnitude.denominator, 0)
    certify_ieee(magnitude, status, bits)
    negative = value < 0
    if negative:
        bits |= SIGN
        status = status.replace('+', '-')
    return status, bits


# ------------------------------------------------- dated instrument lattice

def month_end(value: date) -> bool:
    return value.day == calendar.monthrange(value.year, value.month)[1]


def lattice_date(maturity: date, back_months: int, eom: str) -> date:
    total = maturity.year * 12 + maturity.month - 1 - back_months
    year, month0 = divmod(total, 12)
    last = calendar.monthrange(year, month0 + 1)[1]
    day = last if eom == 'Preserve' and month_end(maturity) else min(maturity.day, last)
    return date(year, month0 + 1, day)


def build_lattice(maturity: date, length: int, eom: str) -> list[date]:
    return [lattice_date(maturity, 6 * (length - 1 - i), eom) for i in range(length)]


class Instrument:
    """Validated dated instrument (main E) with exact raw amounts."""

    def __init__(self, maturity, length, eom, shape, d_off, i_off, s_off, coef, scale):
        q = build_lattice(maturity, length, eom)
        assert all(a < b for a, b in zip(q, q[1:]))
        assert all(181 <= (b - a).days <= 184 for a, b in zip(q, q[1:]))
        if shape == 'Long':
            assert length >= 3
        if shape == 'Regular':
            d_date = q[0]
        else:
            span = (q[1] - q[0]).days
            d_date = q[0] + timedelta(days=1 + d_off % (span - 1))
        f_index = 2 if shape == 'Long' else 1
        assert q[0] <= d_date < q[1]
        f_date = q[f_index]
        i_date = d_date + timedelta(days=i_off % (f_date - d_date).days)
        assert d_date <= i_date < f_date and i_date < q[-1]
        settle = i_date + timedelta(days=s_off % (q[-1] - i_date).days)
        assert i_date <= settle < q[-1]
        self.q, self.eom, self.shape, self.coef, self.scale = q, eom, shape, coef, scale
        self.d, self.i, self.f, self.s_date, self.f_index = d_date, i_date, f_date, settle, f_index
        self.k = next(idx for idx, x in enumerate(q) if x > settle)
        self.r = (q[self.k] - settle).days
        self.s = (q[self.k] - q[self.k - 1]).days
        self.n = length - 1 - self.k
        self.s0 = (q[1] - q[0]).days
        self.d0 = (q[1] - d_date).days
        assert 1 <= self.r <= self.s

    def dates(self) -> str:
        return '|'.join(x.isoformat() for x in (self.q[0], self.d, self.i, self.f, self.s_date, self.q[-1]))

    def unreduced(self, index: int) -> tuple[int, int]:
        """Main E/N3 unreduced raw coefficient N_A/D_A for absolute index."""
        two = 2 * 10 ** self.scale
        if index == self.f_index and self.shape == 'Short':
            n_a, d_a = self.coef * self.d0, two * self.s0
        elif index == self.f_index and self.shape == 'Long':
            n_a, d_a = self.coef * (self.d0 + self.s0), two * self.s0
        else:
            n_a, d_a = self.coef, two
        if index == len(self.q) - 1:
            n_a += 100 * d_a
        return n_a, d_a

    def bundles(self) -> list[tuple[int, int, int, int]]:
        """(absolute index, j, N_A, D_A) for every future actual coupon bundle."""
        out = []
        for index in range(max(self.k, self.f_index), len(self.q)):
            n_a, d_a = self.unreduced(index)
            out.append((index, index - self.k, n_a, d_a))
        return out


def random_instrument(rng: random.Random, lengths, coefs) -> Instrument | None:
    maturity = date(rng.randint(2027, 2060), rng.randint(1, 12), rng.randint(1, 28))
    if rng.random() < 0.3:
        maturity = maturity.replace(day=calendar.monthrange(maturity.year, maturity.month)[1])
    length = rng.choice(lengths)
    shape = rng.choice(['Regular', 'Short', 'Long'])
    if shape == 'Long' and length < 3:
        return None
    coef, scale = rng.choice(coefs)
    return Instrument(maturity, length, rng.choice(['Preserve', 'Ignore']), shape,
                      rng.randint(0, 999), rng.randint(0, 999), rng.randint(0, 10 ** 6), coef, scale)


COEFS = [(0, 0), (1, 38), (23, 7), (5, 0), (125, 3), (I128_MAX, 38), (I128_MAX, 0)]


# ---------------------------------------------------------------- rows

def blank() -> dict:
    return {name: '-' for name in COLS}


def emit(rows: list, family: str, profile: str, case: str, **fields) -> None:
    record = blank()
    record.update(family=family, profile=profile, case=case)
    for key, value in fields.items():
        assert key in record, key
        record[key] = str(value)
    rows.append(record)


def special_yields() -> list[tuple[str, int]]:
    out = [('zero', 0), ('minus-zero', SIGN), ('min-sub', 1), ('max-sub', (1 << 52) - 1),
           ('min-normal', 1 << 52), ('two-pow-minus60', float_bits(2.0 ** -60)),
           ('one', float_bits(1.0)), ('two', float_bits(2.0)), ('minus-one', float_bits(-1.0)),
           ('minus-one-half', float_bits(-1.5)), ('minus-one-point-nine', float_bits(-1.9)),
           ('next-up-minus-two', float_bits(-2.0) - 1), ('minus-two-plus-2ulp', float_bits(-2.0) - 2),
           ('rate-0884', float_bits(0.0884)), ('rate-1053', float_bits(0.1053)),
           ('ten', float_bits(10.0)), ('one-e300', float_bits(1e300)), ('max', MAX_BITS),
           ('half-max', float_bits(2.0 ** 1023)), ('minus-min-sub', SIGN | 1)]
    return out


RS = [(1, 181), (90, 181), (181, 181), (1, 182), (91, 182), (182, 182), (1, 183), (182, 183),
      (1, 184), (92, 184), (183, 184), (184, 184)]


def seed_rows(rows: list) -> None:
    for label, y in special_yields():
        for r, s in RS:
            yc = canonical_yield(y)
            nb, nq, b, q = seeds(yc, r, s)
            delta_status, delta_bits = ieee_signed(F(nb) * p2(-1074))
            q_status, qhat_bits = ieee_signed(F(nq, 2 * s) * p2(-1074))
            assert delta_status == 'ok' and q_status == 'ok'
            assert max(b.seed_peak, q.seed_peak) <= 2107
            assert binary_value(delta_bits) >= p2(-52) and binary_value(qhat_bits) >= p2(-53)
            emit(rows, 'UST-21', 'TOL-E0', f'seed:{label}', op='SEED', reachability='coordinate-primitive',
                 y_bits=hb(y), r=r, s=s, status='ok',
                 b_interval=f'{ptxt(*b.lo, 64)}|{ptxt(*b.hi, 64)}',
                 q_interval=f'{ptxt(*q.lo, 64)}|{ptxt(*q.hi, 64)}',
                 delta_bits=hb(delta_bits), qhat_bits=hb(qhat_bits),
                 seed_bits=f'{nb.bit_length()},{nq.bit_length()}', peak_bits=max(b.seed_peak, q.seed_peak))
    for label, y in (('minus-two', float_bits(-2.0)), ('minus-three', float_bits(-3.0)),
                     ('plus-inf', INF_BITS), ('minus-inf', SIGN | INF_BITS), ('nan', 0x7ff8000000000000)):
        assert not valid_yield(y)
        emit(rows, 'UST-29', 'TOL-E0', f'invalid-yield:{label}', op='SEED', reachability='primitive-superset',
             y_bits=hb(y), r=1, s=182, status='invalid-yield')


def ideal_factor(nb: int, nq: int, s: int, j: int):
    """Exact ideal factor as unnormalized integers (n, d) when feasible, else None."""
    b = F(nb, 2 << 1074)
    q = F(nq, (2 * s) << 1074)
    if j * max(b.numerator.bit_length(), b.denominator.bit_length()) > EXACT_BIT_BUDGET:
        return None
    return q.denominator * b.denominator ** j, q.numerator * b.numerator ** j


def cmp_point_ratio(m: int, e: int, n: int, d: int) -> int:
    """Sign of m*2^e - n/d using integers only."""
    if e >= 0:
        lhs, rhs = (m << e) * d, n
    else:
        lhs, rhs = m * d, n << -e
    return (lhs > rhs) - (lhs < rhs)


def straddle_class(lo, hi) -> str:
    """Extended53 midpoint relation of [lo, hi]: exact-point/inside/endpoint-touch/strict."""
    if lo == hi:
        return 'exact-point'
    m, e = lo
    # At most one extended53 midpoint can occur in this narrow p256 interval.
    # Positive binade spacing never decreases above lo; require < one unit.
    limit = (m + (1 << 203), e)
    assert le(hi, limit) and not le(limit, hi)
    unit = e + 203
    n, frac = m >> 203, m & ((1 << 203) - 1)
    if frac <= 1 << 202:
        mid = (2 * n + 1, unit - 1)
    elif n + 1 < 1 << 53:
        mid = (2 * n + 3, unit - 1)
    else:
        mid = ((1 << 53) + 1, unit)
    if (le(mid, lo) and le(lo, mid)) or (le(mid, hi) and le(hi, mid)):
        return 'endpoint-touch'
    return 'strict-straddle' if le(mid, hi) else 'inside'


def factor_observation(y: int, r: int, s: int, j: int, alt: bool = False):
    nb, nq, b, q = seeds(y, r, s)
    power = l2r_power(b, j)
    big_q = q if j == 0 else multiply(q, power)
    fac = reciprocal(big_q)
    sig, exp, tie, carry = rne53_point(*fac.lo)
    # A power-of-two rescaling preserves both cell membership and tie parity.
    # Certify the 256-bit integer, without constructing a 20M-bit denominator.
    certify_ext53(F(fac.lo[0]), sig, exp - fac.lo[1], tie)
    width = fac.c + 2 * fac.o
    e0 = min(fac.lo[1], fac.hi[1])
    hi_al, lo_al = fac.hi[0] << (fac.hi[1] - e0), fac.lo[0] << (fac.lo[1] - e0)
    width_ok = hi_al * ((1 << 255) - width) <= lo_al * (1 << 255)
    assert width_ok
    alt_text = '-'
    if alt:
        cum = cumulative_power(b, j)
        alt_q = q if j == 0 else multiply(q, cum)
        alt_fac = reciprocal(alt_q)
        alt_sig = rne53_point(*alt_fac.lo)[:2]
        alt_text = 'same' if alt_sig == (sig, exp) and alt_fac.lo == fac.lo else (
            'same-observation' if alt_sig == (sig, exp) else 'differs')
    return nb, nq, fac, (sig, exp, tie, carry), width, alt_text


def factor_rows(rows: list) -> None:
    ys = [y for y in special_yields() if valid_yield(canonical_yield(y[1]))]
    js = [0, 1, 2, 3, 4, 7, 8, 15, 16, 59, 60, 61, 255, 256, 1000, 1023, 1024, 4095, 16384, 19996, 19997, 19999]
    rs = [(1, 181), (92, 184), (183, 184), (184, 184), (1, 182)]
    for label, y in ys:
        yc = canonical_yield(y)
        for r, s in rs:
            for j in js:
                if y == SIGN and (r, s) != (1, 181):
                    continue
                alt = j in (3, 7, 59, 61, 255, 1000) and (r, s) == (92, 184)
                nb, nq, fac, ext, width, alt_text = factor_observation(yc, r, s, j, alt)
                sig, exp, tie, carry = ext
                ideal = ideal_factor(nb, nq, s, j)
                relation, containment = 'unknown', 'per-operation'
                if ideal is not None:
                    n, d = ideal
                    assert cmp_point_ratio(*fac.lo, n, d) <= 0 <= cmp_point_ratio(*fac.hi, n, d)
                    sign = cmp_point_ratio(sig, exp, n, d)
                    relation = 'above' if sign > 0 else 'below' if sign < 0 else 'equal'
                    containment = 'exact-and-per-operation'
                    STATS['factor_exact_containment'] += 1
                family = 'UST-25' if j in (0, 1, 2, 4, 8, 16, 256, 1024, 16384) else (
                    'UST-23' if label == 'min-sub' else 'UST-26' if label in ('max', 'next-up-minus-two', 'half-max') else 'UST-27')
                if alt:
                    family = 'UST-24'
                reach = 'primitive-superset' if j > DATE_J_MAX else 'coordinate-primitive'
                emit(rows, family, 'TOL-E0', f'factor:{label}:r{r}s{s}:j{j}', op='FACTOR', reachability=reach,
                     y_bits=hb(y), r=r, s=s, j=j, status='ok',
                     factor_lo=ptxt(*fac.lo, 64), factor_hi=ptxt(*fac.hi, 64),
                     weights=f'{fac.c},{fac.o},{width}', width_ok='yes', relation=relation,
                     straddle=straddle_class(fac.lo, fac.hi), containment=containment, tree_alt=alt_text,
                     peak_bits=fac.peak, ext_value=ptxt(sig, exp, 14),
                     tie_flag='yes' if tie else 'no', carry_flag='yes' if carry else 'no')
                assert fac.peak <= 513


def ext53_rows(rows: list) -> None:
    """Constructed p256 low endpoints for extended53 tie/carry boundaries."""
    base = (1 << 255) | (0x1234567 << 210)
    cases = [
        ('tie-even-down', ((base >> 203) & ~1) << 203 | (1 << 202)),
        ('tie-odd-up', (((base >> 203) | 1) << 203) | (1 << 202)),
        ('below-half', ((base >> 203) << 203) | ((1 << 202) - 1)),
        ('above-half', ((base >> 203) << 203) | ((1 << 202) + 1)),
        ('exact-grid', (base >> 203) << 203),
        ('carry-all-ones-up', ((1 << 256) - 1)),
        ('carry-tie-odd', (((1 << 53) - 1) << 203) | (1 << 202)),
        ('no-carry-tie-even', (((1 << 53) - 2) << 203) | (1 << 202)),
        ('min-significand', 1 << 255),
        ('max-minus-half', (((1 << 53) - 1) << 203) | ((1 << 202) - 1)),
    ]
    for exponent in (-20460255, -255, 0, 1059745):
        for label, m in cases:
            assert m.bit_length() == 256
            sig, exp, tie, carry = rne53_point(m, exponent)
            certify_ext53(F(m), sig, exp - exponent, tie)
            emit(rows, 'UST-27', 'TOL-E0', f'ext53:{label}:e{exponent}', op='EXT53', reachability='primitive-superset',
                 low_point=ptxt(m, exponent, 64), status='ok', ext_value=ptxt(sig, exp, 14),
                 tie_flag='yes' if tie else 'no', carry_flag='yes' if carry else 'no')


def pv_record(rows, family, reach, case, n_a, d_a, sig, exp, **extra) -> tuple[str, int]:
    status, bits, tie, peak = ieee_direct(n_a * sig, d_a, exp)
    value = F(n_a * sig, d_a) * p2(exp)
    certify_ieee(value, status, bits)
    reduced = F(n_a, d_a)
    status2, bits2, _, _ = ieee_direct(reduced.numerator * sig, reduced.denominator, exp)
    assert (status2, bits2) == (status, bits)
    assert peak <= 206
    alternate = '-'
    if status == 'ok':
        ext = rne53_ratio(n_a * sig, d_a, exp)
        twice = ieee_direct(ext[0], 1, ext[1])
        if twice[1] != bits:
            alternate = hb(twice[1])
            STATS['pv_double_rounding_discriminators'] += 1
    emit(rows, family, 'TOL-E0', case, op='PV', reachability=reach,
         amount_num=n_a, amount_den=d_a, factor_hat=ptxt(sig, exp, 14), status=status,
         expected_bits=hb(bits), alternate_bits=alternate, tie_flag='yes' if tie else 'no',
         peak_bits=peak, **extra)
    return status, bits


def pv_rows(rows: list) -> None:
    rng = random.Random(20261020)
    produced = 0
    while produced < 260:
        inst = random_instrument(rng, [2, 3, 4, 5, 8, 21], COEFS)
        if inst is None:
            continue
        band = rng.random()
        y = float_bits(-2 + 2.0 ** rng.uniform(-52, 1)) if band < 0.25 else (
            float_bits(2.0 ** rng.uniform(-40, 8)) if band < 0.65 else float_bits(2.0 ** rng.uniform(100, 1023.9)))
        if not valid_yield(y):
            continue
        bundles = inst.bundles()
        chosen = {0, len(bundles) - 1, min(1, len(bundles) - 1)}
        for position in sorted(chosen):
            index, j, n_a, d_a = bundles[position]
            if n_a == 0:
                continue
            _, _, _, ext, _, _ = factor_observation(y, inst.r, inst.s, j)
            pv_record(rows, 'UST-31', 'dated-financial', f'pv:dated:{produced}:{index}', n_a, d_a, ext[0], ext[1],
                      dates=inst.dates(), eom=inst.eom, shape=inst.shape, y_bits=hb(y), r=inst.r, s=inst.s,
                      j=j, c_coef=inst.coef, c_scale=inst.scale)
        produced += 1
    # Dated witnesses: C=0 capital-only, alpha=1, principal at j=1.
    capital = Instrument(date(2025, 7, 15), 4, 'Preserve', 'Regular', 0, 0, 182, 0, 0)
    assert capital.s_date == date(2024, 7, 15) and capital.r == capital.s == 184 and capital.n == 1
    for case, y_hex, family in (('double-rounding-real', '0x1.bdb8cdadbe029p+515', 'UST-32'),
                                ('exact-tie-real', '0x1.9999999999b3cp-5', 'UST-32'),
                                ('accepted-subnormal-real', '0x1.c48c6001f0abfp+541', 'UST-33')):
        y = float_bits(float.fromhex(y_hex))
        index, j, n_a, d_a = capital.bundles()[-1]
        _, _, _, ext, _, _ = factor_observation(y, capital.r, capital.s, j)
        pv_record(rows, family, 'dated-financial', f'pv:{case}', n_a, d_a, ext[0], ext[1],
                  dates=capital.dates(), eom=capital.eom, shape=capital.shape, y_bits=hb(y),
                  r=capital.r, s=capital.s, j=j, c_coef=0, c_scale=0)
    # Tiny factor rescued by source scaling: C = i128::MAX, j=1, huge y.
    rescue = Instrument(date(2025, 7, 15), 3, 'Preserve', 'Regular', 0, 0, 0, I128_MAX, 0)
    y = float_bits(2.0 ** 549.5)
    index, j, n_a, d_a = rescue.bundles()[-1]
    _, _, _, ext, _, _ = factor_observation(y, rescue.r, rescue.s, j)
    assert ext[1] + 52 < -1075
    pv_record(rows, 'UST-34', 'dated-financial', 'pv:tiny-factor-rescued', n_a, d_a, ext[0], ext[1],
              dates=rescue.dates(), eom=rescue.eom, shape=rescue.shape, y_bits=hb(y), r=rescue.r,
              s=rescue.s, j=j, c_coef=I128_MAX, c_scale=0)
    # Primitive supersets: subnormal boundary, overflow boundary and ties.
    for label, n_a, d_a, sig, exp in (
            ('half-min-sub-exact-tie-to-zero', 1, 1, 1 << 52, -1127),
            ('just-above-half-min-sub', 1, 1, (1 << 52) + 1, -1127),
            ('max-sub-to-min-normal-tie', (1 << 53) - 1, 1, 1 << 52, -1127),
            ('subnormal-tie-to-even', 3, 1, 1 << 52, -1127),
            ('exact-max', (1 << 54) - 2, 1, 1 << 52, 918),
            ('overflow-midpoint', (1 << 54) - 1, 1, 1 << 52, 918),
            ('normal-tie-rounds-up', 3, 1, (1 << 52) | 1, -2),
            ('normal-tie-stays', 3, 1, (1 << 52) | 3, -2),
            ('direct-vs-double-generic', (1 << 108) - (1 << 55) - 1, 1, 1 << 52, -1182)):
        pv_record(rows, 'UST-32' if 'tie' in label or 'double' in label else 'UST-33',
                  'primitive-superset', f'pv:primitive:{label}', n_a, d_a, sig, exp)


def ai_pieces(inst: Instrument) -> list[tuple[int, int]]:
    k, f = inst.k, inst.f_index
    q, settle = inst.q, inst.s_date
    if k > f or (inst.shape == 'Regular' and k == f):
        return [((settle - q[k - 1]).days, (q[k] - q[k - 1]).days)]
    if inst.shape == 'Short' and k == 1:
        return [((settle - inst.d).days, inst.s0)]
    if inst.shape == 'Long' and k == 1:
        return [((settle - inst.d).days, inst.s0)]
    assert inst.shape == 'Long' and k == 2
    return [(inst.d0, inst.s0), ((settle - q[1]).days, (q[2] - q[1]).days)]


def round_count(value: F, places: int, tie: str) -> int:
    scaled = value * 10 ** places
    fl = scaled.numerator // scaled.denominator
    frac = scaled - fl
    if frac > F(1, 2) or (frac == F(1, 2) and (tie == 'NearestTiesAwayFromZero' or fl % 2)):
        fl += 1
    if frac == F(1, 2):
        STATS[f'source_tie_{places}'] += 1
    return fl


def ai_rows(rows: list) -> None:
    rng = random.Random(20261021)
    made = 0
    coefs = COEFS + [(23, 7), (368, 6)]
    while made < 300:
        inst = random_instrument(rng, [2, 3, 4, 5, 21], coefs)
        if inst is None:
            continue
        if rng.random() < 0.35:
            # Move settlement to a structurally interesting date for this instrument.
            options = [inst.d, inst.q[1], inst.f, inst.q[1] + timedelta(days=1)]
            target = rng.choice(options)
            if inst.i <= target < inst.q[-1]:
                inst = Instrument(inst.q[-1], len(inst.q), inst.eom, inst.shape,
                                  (inst.d - inst.q[0]).days - 1, (inst.i - inst.d).days,
                                  (target - inst.i).days, inst.coef, inst.scale)
        pieces = ai_pieces(inst)
        c_total = F(inst.coef, 10 ** inst.scale)
        raw = sum((c_total / 2) * F(t, s) for t, s in pieces)
        for basis in ('ExactRatio', 'NineDecimal'):
            for tie in ('NearestTiesEven', 'NearestTiesAwayFromZero'):
                total = F(0)
                for t, s in pieces:
                    daily = 5 * c_total / s
                    if basis == 'NineDecimal':
                        daily = F(round_count(daily, 9, tie), 10 ** 9)
                    total += daily * t
                count = round_count(total, 5, tie)
                published = F(count, 10 ** 6)
                raw_status, raw_bits = ieee_signed(raw)
                pub_status, pub_bits = ieee_signed(published)
                assert raw_status == pub_status == 'ok'
                emit(rows, 'UST-19' if len(pieces) == 2 else 'UST-20', 'TOL-E1', f'ai:{made}:{basis}:{tie}',
                     op='AICACHE', reachability='dated-financial', dates=inst.dates(), eom=inst.eom,
                     shape=inst.shape, r=inst.r, s=inst.s, c_coef=inst.coef, c_scale=inst.scale,
                     basis=basis, tie=tie, status='ok',
                     pieces=';'.join(f'{t}/{s}' for t, s in pieces),
                     raw_ai=f'{raw.numerator}/{raw.denominator}', published_count=count,
                     raw_ai_bits=hb(raw_bits), published_bits=hb(pub_bits))
        made += 1


def prefix_observations(y: int, r: int, s: int) -> tuple[int, int]:
    nb, nq, _ = seed_numerators(y, r, s)
    _, delta_bits = ieee_signed(F(nb) * p2(-1074))
    _, qhat_bits = ieee_signed(F(nq, 2 * s) * p2(-1074))
    return delta_bits, qhat_bits


def term_value(measure: str, pv: F, j: int, r: int, s: int, dhat: F, qhat: F) -> F:
    if measure == 'p1':
        return pv * (2 * s * j * qhat + r * dhat) / (2 * s * dhat * qhat)
    if measure == 'p2':
        return pv * (2 * j * (j + 1) * s * s * qhat * qhat + 2 * j * r * s * dhat * qhat
                     + r * r * dhat * dhat) / (2 * s * s * dhat * dhat * qhat * qhat)
    return pv * (r + j * s) / (2 * s)


def term_integers(measure: str, pv_bits: int, j: int, r: int, s: int, delta_bits: int, qhat_bits: int):
    """Contract N5 aligned integer numerator/denominator and their exponent."""
    _, p, pe = decode(pv_bits)
    _, d, ed = decode(delta_bits)
    _, qs, eq = decode(qhat_bits)
    assert d >= 1 << 52 and qs >= 1 << 52
    if measure == 'p2':
        m2 = 2 * min(ed, eq)
        num = p * (((2 * j * (j + 1) * s * s * qs * qs) << (2 * eq - m2))
                   + ((2 * j * r * s * d * qs) << (ed + eq - m2))
                   + ((r * r * d * d) << (2 * ed - m2)))
        return num, 2 * s * s * d * d * qs * qs, pe + m2 - 2 * ed - 2 * eq, ed - eq
    if measure == 'p1':
        m1 = min(ed, eq)
        num = p * (((2 * s * j * qs) << (eq - m1)) + ((r * d) << (ed - m1)))
        return num, 2 * s * d * qs, pe + m1 - ed - eq, ed - eq
    return p * (r + j * s), 2 * s, pe, ed - eq


def term_record(rows, family, reach, case, pv_bits, y, r, s, j, measure, **extra):
    delta_bits, qhat_bits = prefix_observations(y, r, s)
    pv = binary_value(pv_bits)
    value = term_value(measure, pv, j, r, s, binary_value(delta_bits), binary_value(qhat_bits))
    num, den, exponent, e_diff = term_integers(measure, pv_bits, j, r, s, delta_bits, qhat_bits)
    assert F(num, den) * p2(exponent) == value
    assert -52 <= e_diff <= 9
    assert num.bit_length() <= 311 and den.bit_length() <= 229
    sig, exp, tie, carry, peak = rne53_ratio(num, den, exponent)
    certify_ext53(value, sig, exp, tie)
    assert peak <= 311
    emit(rows, family, 'TOL-E0', case, op='TERM', reachability=reach, y_bits=hb(y), r=r, s=s, j=j,
         pv_bits=hb(pv_bits), measure=measure, status='ok', ext_value=ptxt(sig, exp, 14),
         num_bits=num.bit_length(), den_bits=den.bit_length(), e_diff=e_diff, peak_bits=peak,
         delta_bits=hb(delta_bits), qhat_bits=hb(qhat_bits),
         tie_flag='yes' if tie else 'no', carry_flag='yes' if carry else 'no', **extra)
    return sig, exp


def term_rows(rows: list) -> None:
    rng = random.Random(20261022)
    made = 0
    while made < 240:
        inst = random_instrument(rng, [2, 3, 5, 21], COEFS)
        if inst is None:
            continue
        y = float_bits(2.0 ** rng.uniform(-30, 6)) if rng.random() < 0.6 else float_bits(-2 + 2.0 ** rng.uniform(-52, 1))
        bundles = inst.bundles()
        index, j, n_a, d_a = bundles[-1]
        _, _, _, ext, _, _ = factor_observation(y, inst.r, inst.s, j)
        status, bits, _, _ = ieee_direct(n_a * ext[0], d_a, ext[1])
        if status != 'ok':
            continue
        for measure in ('p1', 'p2', 'moment'):
            term_record(rows, 'UST-41', 'dated-financial', f'term:dated:{made}:{measure}', bits, y,
                        inst.r, inst.s, j, measure, dates=inst.dates(), eom=inst.eom, shape=inst.shape,
                        c_coef=inst.coef, c_scale=inst.scale)
        made += 1
    # Coupled extremes over valid coordinates with primitive PV significands.
    for y_label, y in (('next-up-minus-two', float_bits(-2.0) - 1), ('max', MAX_BITS), ('min-sub', 1),
                       ('one', float_bits(1.0)), ('minus-one-point-nine', float_bits(-1.9))):
        for r, s in ((1, 181), (181, 181), (92, 184), (184, 184)):
            for j in (0, 1, J_CAP):
                for pv_label, pv_bits in (('max-sig', float_bits(1.0) | ((1 << 52) - 1)), ('min-sub', 1),
                                          ('max', MAX_BITS)):
                    for measure in ('p1', 'p2', 'moment'):
                        term_record(rows, 'UST-42', 'primitive-superset',
                                    f'term:extreme:{y_label}:r{r}s{s}:j{j}:{pv_label}:{measure}',
                                    pv_bits, y, r, s, j, measure)


def evaluate_plan(inst: Instrument, y: int):
    """Full prescribed evaluation and risk of a dated plan."""
    delta_bits, qhat_bits = prefix_observations(y, inst.r, inst.s)
    dhat, qhat = binary_value(delta_bits), binary_value(qhat_bits)
    pvs, sums = [], {'p1': 0, 'p2': 0, 'moment': 0}
    for index, j, n_a, d_a in inst.bundles():
        _, _, _, ext, _, _ = factor_observation(y, inst.r, inst.s, j)
        if n_a == 0:
            pvs.append(0)
            continue
        status, bits, _, _ = ieee_direct(n_a * ext[0], d_a, ext[1])
        certify_ieee(F(n_a * ext[0], d_a) * p2(ext[1]), status, bits)
        if status != 'ok':
            return {'status': 'pv-' + status, 'first_fail': index}
        pvs.append(bits)
        for measure in sums:
            value = term_value(measure, binary_value(bits), j, inst.r, inst.s, dhat, qhat)
            num, den, exponent, _ = term_integers(measure, bits, j, inst.r, inst.s, delta_bits, qhat_bits)
            sig, exp, tie, _, _ = rne53_ratio(num, den, exponent)
            certify_ext53(value, sig, exp, tie)
            assert exp + BASIS >= 0
            sums[measure] += sig << (exp + BASIS)
    total = sum((binary_value(b) for b in pvs), F(0))
    status, dirty = ieee_signed(total)
    result = {'status': 'ok' if status == 'ok' else 'sum-' + status.rstrip('+'), 'pvs': pvs,
              'positive': sum(1 for b in pvs if b), 'sums': sums}
    if status != 'ok':
        return result
    result['dirty'] = dirty
    d_value = binary_value(dirty)
    for measure, total_int in sums.items():
        assert total_int.bit_length() <= 9367
        value = F(total_int) * p2(-BASIS)
        signed = -value if measure == 'p1' else value
        result[measure] = ieee_signed(signed)
        normalized = value / d_value
        name = {'p1': 'modified', 'p2': 'convexity', 'moment': 'macaulay'}[measure]
        result[name] = ieee_signed(normalized)
        # Contract N5 normalization width after early classification.
        if result[name][0] == 'ok':
            _, dm, de = decode(dirty)
            k, _ = floor_log2(total_int, dm)
            big_k = k - BASIS - de
            unit = max(big_k - 52, -1074)
            shift = BASIS + de + unit
            assert 6044 <= shift <= 10134
            assert (dm << shift).bit_length() <= 10187
    return result


def plan_record(rows, family, case, inst: Instrument, y: int) -> dict:
    out = evaluate_plan(inst, y)
    fields = dict(op='REDUCE', reachability='dated-financial', dates=inst.dates(), eom=inst.eom,
                  shape=inst.shape, y_bits=hb(y), r=inst.r, s=inst.s, remaining_periods=inst.n, c_coef=inst.coef,
                  c_scale=inst.scale, bundle_count=len(inst.bundles()), status=out['status'])
    if out['status'].startswith('pv-'):
        fields['first_fail'] = out['first_fail']
    else:
        fields['positive_count'] = out['positive']
        fields['sum_bits'] = ','.join(str(out['sums'][m].bit_length()) for m in ('p1', 'p2', 'moment'))
        fields['sum_fnv64'] = ','.join(fnv(out['sums'][m].to_bytes(WORDS * 8, 'little')) for m in ('p1', 'p2', 'moment'))
        if 'dirty' in out:
            fields['dirty_status'] = 'ok'
            fields['dirty_bits_out'] = hb(out['dirty'])
            for name in ('p1', 'p2', 'moment', 'modified', 'macaulay', 'convexity'):
                status, bits = out[name]
                fields[f'{name}_status'] = status
                fields[f'{name}_bits'] = hb(bits)
        else:
            fields['dirty_status'] = out['status']
    emit(rows, family, 'TOL-E0', case, **fields)
    return out


def reduce_rows(rows: list) -> None:
    witnesses = [
        ('raw-overflow-10y', 'UST-43', Instrument(date(2035, 7, 15), 21, 'Preserve', 'Regular', 0, 0, 1, 5, 0),
         float_bits(-2.0) - 1),
        ('raw-underflow-two-bundle', 'UST-44', Instrument(date(2026, 7, 15), 3, 'Preserve', 'Regular', 0, 0, 183, 1, 38),
         float_bits(2.0 ** 545.5)),
        ('subnormal-convexity', 'UST-47', Instrument(date(2026, 7, 15), 3, 'Preserve', 'Regular', 0, 0, 183, 1, 38),
         float_bits(1.2345 * 2.0 ** 530)),
        ('single-term-subnormal', 'UST-48', Instrument(date(2025, 7, 15), 4, 'Preserve', 'Regular', 0, 0, 182, 0, 0),
         float_bits(float.fromhex('0x1.c48c6001f0abfp+541'))),
        ('n0-max-yield-modified', 'UST-46', Instrument(date(2026, 1, 15), 2, 'Preserve', 'Regular', 0, 0, 183, 5, 0),
         MAX_BITS),
        ('supremum-overshoot', 'UST-57', Instrument(date(2026, 4, 15), 5, 'Preserve', 'Regular', 0, 0, 729, 5, 0),
         float_bits(-2.0) - 1),
        ('sum-overflow', 'UST-36', Instrument(date(2534, 7, 15), 1018, 'Preserve', 'Regular', 0, 0, 74, 100, 0),
         float_bits(-1.0)),
        ('first-failed-pv-overflow', 'UST-35', Instrument(date(2040, 7, 15), 41, 'Preserve', 'Regular', 0, 0, 1, 5, 0),
         float_bits(-2.0) - 1),
        ('first-failed-pv-underflow', 'UST-35', Instrument(date(2030, 7, 15), 11, 'Preserve', 'Regular', 0, 0, 1, 5, 0),
         float_bits(2.0 ** 400)),
    ]
    for case, family, inst, y in witnesses:
        out = plan_record(rows, family, f'reduce:{case}', inst, y)
        STATS[f'witness_{case}_{out["status"]}'] += 1
    rng = random.Random(20261023)
    made = 0
    while made < 220:
        inst = random_instrument(rng, [2, 3, 4, 5, 8, 21], COEFS)
        if inst is None:
            continue
        band = rng.random()
        y = float_bits(-2 + 2.0 ** rng.uniform(-52, 1)) if band < 0.3 else (
            float_bits(2.0 ** rng.uniform(-40, 8)) if band < 0.6 else float_bits(2.0 ** rng.uniform(100, 1023.9)))
        if not valid_yield(y):
            continue
        plan_record(rows, 'UST-49', f'reduce:dated:{made}', inst, y)
        made += 1


def residual_rows(rows: list) -> None:
    cases = [
        ('equal', 1.0, 'dirty', 1.0, 0.0),
        ('near-equal-up', 1.0, 'dirty', float.fromhex('0x1.0000000000001p+0'), 0.0),
        ('near-equal-down', float.fromhex('0x1.0000000000001p+0'), 'dirty', 1.0, 0.0),
        ('large-target-scale', 2.0 ** 1000, 'dirty', 2.0 ** 1001, 0.0),
        ('scale-threshold', 2.0 ** 1000, 'dirty', 2.0 ** 970, 0.0),
        ('below-threshold', 2.0 ** 1000, 'dirty', float.fromhex('0x1.fffffffffffffp+969'), 0.0),
        ('max-dirty-small-target', bits_float(MAX_BITS), 'dirty', 1.0, 0.0),
        ('subnormal-difference', 5e-324 * 3, 'dirty', 5e-324, 0.0),
        ('subnormal-negative', 5e-324, 'dirty', 5e-324 * 4, 0.0),
        ('clean-shift-positive', 99.5, 'clean', 99.0, 0.367403),
        ('negative-clean-resolving-positive', 1.0, 'clean', -0.25, 0.5),
        ('nonpositive-clean', 1.0, 'clean', -0.5, 0.5),
        ('nonpositive-dirty-zero', 1.0, 'dirty', 0.0, 0.0),
        ('nonpositive-dirty-negative', 1.0, 'dirty', -3.0, 0.0),
        ('huge-target-tiny-dirty', 5e-324, 'dirty', bits_float(MAX_BITS), 0.0),
        ('min-sub-clean-sum', 2.0, 'clean', 5e-324, 1e-6),
    ]
    for label, dirty, kind, target, ai in cases:
        dirty_bits, target_bits, ai_bits = float_bits(dirty), float_bits(target), float_bits(ai)
        t_exact = binary_value(target_bits) + (binary_value(ai_bits) if kind == 'clean' else 0)
        payload_status, payload = ieee_signed(t_exact)
        assert payload_status == 'ok'
        resolved = binary_value(payload)
        fields = dict(op='RESIDUAL', reachability='primitive-superset', dirty_bits=hb(dirty_bits),
                      target_kind=kind, target_bits=hb(target_bits), ai_bits=hb(ai_bits) if kind == 'clean' else '-',
                      dirty_target_bits=hb(payload))
        if resolved <= 0:
            fields['status'] = 'nonpositive-target'
            emit(rows, 'UST-51', 'TOL-E0', f'residual:{label}', **fields)
            continue
        scale = 0 if resolved < p2(970) else 1
        d_exact = binary_value(dirty_bits)
        residual_status, residual = ieee_signed((d_exact - resolved) * p2(-scale))
        amount_status, amount = ieee_signed(d_exact - resolved)
        assert residual_status == 'ok' and amount_status == 'ok'
        assert (residual == 0) == (d_exact == resolved)
        emit(rows, 'UST-55', 'TOL-E0', f'residual:{label}', status='ok', scale=scale,
             residual_bits=hb(residual), amount_bits=hb(amount), **fields)


# --------------------------------------------------------------- assembly

def audit(rows: list) -> None:
    for record in rows:
        assert set(record) == set(COLS)
        assert all(value and '\t' not in value and '\n' not in value for value in record.values())
        if record['reachability'] == 'dated-financial':
            q0, d, i, f, s, m = map(date.fromisoformat, record['dates'].split('|'))
            assert q0 <= d < f and d <= i < f and i <= s < m and i < m
            coef = int(record['c_coef'])
            assert 0 <= coef <= I128_MAX and 0 <= int(record['c_scale']) <= 38
        if record['r'] != '-':
            r, s = int(record['r']), int(record['s'])
            assert 181 <= s <= 184 and 1 <= r <= s
        if record['j'] != '-' and record['op'] in ('FACTOR', 'TERM', 'PV'):
            assert 0 <= int(record['j']) <= J_CAP
            if int(record['j']) > DATE_J_MAX:
                assert record['reachability'] == 'primitive-superset'


def fixture_bytes(rows: list, generated_at: str, platform_text: str) -> bytes:
    source = Path(__file__).read_bytes()
    main, cash = MAIN.read_bytes(), CASH.read_bytes()
    assert hashlib.sha256(main).hexdigest() == MAIN_SHA, 'adopted main contract changed'
    assert hashlib.sha256(cash).hexdigest() == CASH_SHA, 'adopted cash contract changed'
    requirement_rows = [line for line in REQUIREMENTS.read_text(encoding='utf-8').splitlines() if line.startswith('| FR-UST-')]
    assert len(requirement_rows) == 26
    requirements = ''.join(line + '\n' for line in requirement_rows).encode()
    records = ''.join('\t'.join(record[name] for name in COLS) + '\n' for record in rows).encode()
    body = ('\t'.join(COLS) + '\n').encode() + records
    inputs = ''.join('\t'.join(record[name] for name in INPUT_COLS) + '\n' for record in rows).encode()
    unique_inputs = len({'\t'.join(record[name] for name in INPUT_COLS) for record in rows})
    counts = lambda key: json.dumps(dict(sorted(Counter(r[key] for r in rows).items())), separators=(',', ':'))
    headers = [
        'schema: 1', 'fixture-set: M8-US-TREASURY-NUMERIC-ORACLE', 'domain: bonds',
        'description: Independent prescribed N1-N8 p256 factor, direct IEEE PV, AI cache, extended53 risk term, reduction and residual primitive observations',
        'generator: validation/generator/treasury_numeric_oracles.py',
        'reference: exact Python integers/Fraction per adopted contract; adjacent-cell/parity, operand-product and dual-Decimal certificates; no Rust/production/QuantLib reads',
        'reference-version: Python standard library',
        'reference-source: docs/M8_US_TREASURY_CONTRACT.md; docs/M8_US_TREASURY_CASH_CONTRACT.md',
        f'generated-at: {generated_at}',
        f'platform: {platform_text}',
        'reproduction: python3 -I validation/generator/treasury_numeric_oracles.py --check',
        'tolerance: TOL-E0 exact prescribed staging (TOL-E1 for AI binary views); not an ideal-price/risk certificate',
        f'generator-sha256: {hashlib.sha256(source).hexdigest()}', f'generator-fnv64: {fnv(source)}',
        f'adopted-contract-sha256: {MAIN_SHA}', f'adopted-contract-fnv64: {fnv(main)}',
        f'cash-contract-sha256: {CASH_SHA}', f'cash-contract-fnv64: {fnv(cash)}',
        f'requirements-rows-sha256: {hashlib.sha256(requirements).hexdigest()}',
        f'requirements-rows-fnv64: {fnv(requirements)}',
        f'record-count: {len(rows)}', f'column-count: {len(COLS)}',
        f'body-sha256: {hashlib.sha256(body).hexdigest()}', f'body-fnv64: {fnv(body)}',
        f'records-sha256: {hashlib.sha256(records).hexdigest()}', f'records-fnv64: {fnv(records)}',
        'input-columns: ' + ','.join(INPUT_COLS),
        f'unique-input-count: {unique_inputs}',
        f'inputs-sha256: {hashlib.sha256(inputs).hexdigest()}', f'inputs-fnv64: {fnv(inputs)}',
        'profile-counts: ' + counts('profile'),
        'operation-counts: ' + counts('op'),
        'family-counts: ' + counts('family'),
        'reachability-counts: ' + counts('reachability'),
        'status-counts: ' + counts('status'),
        'crosscheck-counts: ' + json.dumps(dict(sorted(STATS.items())), separators=(',', ':')),
        'encoding: REDUCE remaining_periods=n with j unused; classification reachability/shape excluded from input fingerprint; binary64 0x+16hex; p256 and extended53 points significand_hex:exponent; dates Q0|D|I|F|S|M; weights c,o,N; sums 243-u64-word little-endian FNV64 on basis 2^-8192 (p1,p2,moment)',
        'reachability-meaning: dated-financial validated lattice/shape/dates/coupon; coordinate-primitive valid coupled y/r/s/j; primitive-superset need not be reachable',
    ]
    return ('\n'.join(headers) + '\n\n').encode() + body


def generate(generated_at: str, platform_text: str) -> bytes:
    STATS.clear()
    rows: list = []
    for producer in (seed_rows, factor_rows, ext53_rows, pv_rows, ai_rows, term_rows, reduce_rows, residual_rows):
        before = len(rows)
        producer(rows)
        print(json.dumps({'producer': producer.__name__, 'rows': len(rows) - before}), flush=True)
    for index, record in enumerate(rows, 1):
        record['id'] = f'UTNUM-{index:06d}'
    audit(rows)
    unique = len({'\t'.join(r[name] for name in INPUT_COLS) for r in rows})
    assert unique == len(rows), f'duplicate inputs: {len(rows) - unique}'
    return fixture_bytes(rows, generated_at, platform_text)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='compare exact bytes without rewriting the fixture')
    args = parser.parse_args()
    generated_at = datetime.now(timezone.utc).isoformat(timespec='seconds').replace('+00:00', 'Z')
    platform_text = f'{platform.system()} {platform.machine()}; Python {platform.python_version()}'
    if args.check:
        if not OUT.exists():
            raise SystemExit(f'NOT REPRODUCIBLE: {OUT.relative_to(ROOT)} is missing')
        metadata = dict(line.split(': ', 1) for line in OUT.read_text(encoding='utf-8').split('\n\n', 1)[0].splitlines())
        generated_at, platform_text = metadata['generated-at'], metadata['platform']
    if not args.check and OUT.exists():
        raise SystemExit('refusing to overwrite an existing fixture; --check is read-only')
    data = generate(generated_at, platform_text)
    if args.check:
        if OUT.read_bytes() != data:
            raise SystemExit(f'NOT REPRODUCIBLE: {OUT.relative_to(ROOT)}')
    else:
        if OUT.exists():
            raise SystemExit('refusing to overwrite an existing fixture; use --check or write a new additive version')
        with OUT.open('xb') as stream:
            stream.write(data)
    print(json.dumps({'path': str(OUT.relative_to(ROOT)), 'bytes': len(data),
                      'sha256': hashlib.sha256(data).hexdigest(), 'check': args.check}, sort_keys=True), flush=True)


if __name__ == '__main__':
    main()
