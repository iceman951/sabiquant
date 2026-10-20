#!/usr/bin/env python3
"""Research-only Treasury outward-power design check (stdlib only).

This adopts no Treasury contract, API, fixture or production code, and reads
no QuantLib. It models a candidate libm-free factor kernel for the Treasury
candidate term V_j = A_j / (q * b^j), with b = 1 + y/2, q = 1 + (r/s)(y/2)
from one represented y and one source (r, s). Values are p-bit significands
(p = 192 or 256) with an unbounded integer exponent; every endpoint is rounded
outward. Each enclosure carries certified rounding weights (c, o): c weighted
inexact conversions and o weighted outward operations. With u = 2^(1-p),
hi/lo <= (1+u)^c * ((1+u)/(1-u))^o <= (1-u)^-N <= 1/(1 - N*u), N = c + 2*o,
and the last form is checked exactly with p-bit integers.

Containment of the exact value is proved for every cell by induction: each
conversion and each outward product or quotient is checked exactly against its
exact operand result, and products/quotients of positive intervals are
monotone. Independently, every enclosure is compared end to end with an exact
integer rational when the exact power fits a bit budget. Larger cells get only
a directed (floor/ceiling) Decimal cross-check at two precisions, which is not
part of the proof. Binary64 round-to-nearest-even decisions are counted
honestly: decided, unresolved, near (within the certified width of a rounding
boundary), exact ties and exact point enclosures. A degenerate (exact point)
enclosure resolves by the tie rule; an enclosure strictly straddling a midpoint
cannot. Merely touching a midpoint can resolve. No uniform unique rounding is
promised.

It also re-checks, for coupled (delta_hat, q_hat) derived from one (y, r, s),
the five-word widths of the candidate derivative ratios against a model of the
shift steps of gilt's private `extended_ratio` (src/bonds/gilt/numeric.rs at
fc3f629), and the tie-rule qualification of two clean-price recipes.

Source inputs are transcribed from 31 CFR 356 Appendix B Section II A-G and
verified as anchor strings in the pinned eCFR XML snapshot.
"""
from __future__ import annotations

import argparse
import datetime as dt
from decimal import Context, Decimal, MAX_EMAX, MIN_EMIN, ROUND_CEILING, ROUND_FLOOR
from fractions import Fraction as F
import hashlib
import json
import math
from pathlib import Path
import platform
import random
import sys

SOURCE_XML_SHA = '35fa6cb3febd8807c06b08e73d1f311a472b31f127796c6ef369237915b6699d'
PRECISIONS = (192, 256)
J_CAP = 19999
EXACT_BIT_BUDGET = 1 << 22
DECIMAL_DIGITS = (400, 600)
VARIANTS = ('forward', 'reciprocal_exact_seed', 'reciprocal_rounded_seed')
WIDTH_LIMIT = 320
SEED = 20261020

# name, C, i, r, s, n, (r', s''), printed dirty (6 dp), printed P, anchors.
CASES = (
    ('A', '8.75', '.0884', 184, 184, 59, None, '99.057893', '99.057893',
     ('C = 8.75.', 'i = .0884.', 'r = 184 (May 15 to November 15, 1990)',
      's = 184 (May 15 to November 15, 1990)', 'n = 59 (', 'P = 99.057893.')),
    ('B', '8.50', '.0859', 181, 183, 3, None, '99.838183', '99.838183',
     ('C = 8.50.', 'i = .0859.', 'n = 3.', 'r = 181 (April 2 to September 30, 1990)',
      's = 183 (March 31 to September 30, 1990)', 'P = 99.838183.')),
    ('C', '8.50', '.0853', 75, 181, 10, None, '99.805118', '99.805118',
     ('i = .0853.', 'n = 10.', 'r = 75 (March 1 to May 15, 1990',
      's = 181 (November 15, 1989, to May 15, 1990)', 'P = 99.805118.')),
    ('D', '9.50', '.0954', 167, 181, 19, None, '100.098321', '99.730918',
     ('C = 9.50.', 'i = .0954.', 'n = 19.', 'r = 167 (November 29, 1985, to May 15, 1986)',
      's = 181 (November 15, 1985, to May 15, 1986)', '100.098321', 'P = 99.730918.')),
    ('E', '10.75', '.1047', 103, 184, 39, (44, 181), '105.887384', '102.214586',
     ('C = 10.75.', 'i = .1047.', 'n = 39.', 'r = 103 (November 4, 1985, to February 15, 1986)',
      's = 184 (August 15, 1985, to February 15, 1986)', '= 44 (July 2 to August 15, 1985)',
      '= 181 (February 15 to August 15, 1985)', '105.887384', 'P = 102.214586.')),
    ('F', '10.50', '.1053', 92, 184, 15, (183, None), '102.373541', '99.777074',
     ('C = 10.50.', 'i = .1053.', 'n = 15.', 'r = 92 (August 15, 1983, to November 15, 1983)',
      's = 184 (May 15, 1983, to November 15, 1983)', '= 183 (May 16, 1983, to November 15, 1983)',
      '102.373541', 'P = 99.777074.')),
    ('G', '9.75', '.0979', 30, 183, 12, (61, None), '100.563865', '99.738045',
     ('C = 9.75.', 'i = .0979.', 'n = 12.', 'r = 30 (November 15, 1988, to December 15, 1988)',
      's = 183 (June 15, 1988, to December 15, 1988)', '= 61 (October 15, 1988, to December 15, 1988)',
      '100.563865', 'P = 99.738045.')),
)

EDGE_Y = (
    ('MIN_SUB', 5e-324),
    ('MIN_NORMAL', 2.2250738585072014e-308),
    ('2^-60', 2.0 ** -60),
    ('next_up(-2)', -2.0 + 2.0 ** -52),
    ('-1.5', -1.5),
    ('-1', -1.0),
    ('-0.5', -0.5),
    ('RN(.0884)', 0.0884),
    ('RN(.1053)', 0.1053),
    ('1', 1.0),
    ('2', 2.0),
    ('1e300', 1e300),
    ('MAX', sys.float_info.max),
)
EDGE_RS = ((1, 181), (92, 184), (183, 184), (184, 184))
EDGE_J = (0, 1, 2, 3, 59, 60, 61, 255, 1000, J_CAP)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def json_ready(value):
    """Encode infinite diagnostics explicitly; reject NaN and unknown types."""
    if isinstance(value, float):
        if math.isnan(value):
            raise ValueError('NaN is not an admissible report diagnostic')
        if math.isinf(value):
            return 'Infinity' if value > 0 else '-Infinity'
        return value
    if isinstance(value, Decimal):
        if value.is_nan():
            raise ValueError('NaN is not an admissible report diagnostic')
        if value.is_infinite():
            return 'Infinity' if value > 0 else '-Infinity'
        return str(value)
    if isinstance(value, F):
        return str(value)
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (tuple, list)):
        return [json_ready(item) for item in value]
    if value is None or isinstance(value, (str, int, bool)):
        return value
    raise TypeError(f'unsupported report value: {type(value).__name__}')


def reject_json_constant(token):
    """Refuse Python's permissive extension for bare NaN/Infinity tokens."""
    raise ValueError(f'nonstandard JSON constant: {token}')


def strict_json_text(value, **kwargs):
    """Serialize strict JSON and independently parse its final text strictly."""
    serialized = json.dumps(json_ready(value), allow_nan=False, **kwargs)
    json.loads(serialized, parse_constant=reject_json_constant)
    return serialized


def strict_json_controls():
    """Report-format controls; no arithmetic or financial inputs are changed."""
    converted = json.loads(strict_json_text({'negative': -math.inf, 'positive': math.inf,
                                             'finite': 1.25, 'nested': [0, -math.inf]}),
                           parse_constant=reject_json_constant)
    rows = [{'name': 'nested infinities are explicit strings; finite values preserved',
             'ok': converted == {'negative': '-Infinity', 'positive': 'Infinity',
                                  'finite': 1.25, 'nested': [0, '-Infinity']}}]
    for name, value in (('float NaN rejected', math.nan), ('Decimal NaN rejected', Decimal('NaN'))):
        try:
            strict_json_text({'value': value})
        except ValueError:
            rejected = True
        else:
            rejected = False
        rows.append({'name': name, 'ok': rejected})
    for token in ('NaN', 'Infinity', '-Infinity'):
        try:
            json.loads('{"value":' + token + '}', parse_constant=reject_json_constant)
        except ValueError:
            rejected = True
        else:
            rejected = False
        rows.append({'name': f'bare {token} rejected by parser', 'ok': rejected})
    for name, value in (('NaN', math.nan), ('Infinity', math.inf), ('-Infinity', -math.inf)):
        try:
            json.dumps({'value': value}, allow_nan=False)
        except ValueError:
            rejected = True
        else:
            rejected = False
        rows.append({'name': f'allow_nan=False rejects bare {name}', 'ok': rejected})
    rows.append({'name': 'explicit infinity strings accepted by strict parser',
                 'ok': json.loads('["-Infinity","Infinity"]', parse_constant=reject_json_constant)
                        == ['-Infinity', 'Infinity']})
    return {'scope': 'strict JSON format controls, independent of numerical arithmetic',
            'checks': rows, 'ok': all(row['ok'] for row in rows)}


def exact(value):
    """Exact Fraction of a binary64 or a decimal string."""
    return F(value)


# ---------------------------------------------------------------------------
# p-bit outward arithmetic. An endpoint is (m, e) with 2^(p-1) <= m < 2^p and
# value m * 2^e. An enclosure is [lo, hi] plus certified weights (c, o).
# ---------------------------------------------------------------------------

def round_int(n, e, p, up):
    """Round the positive n * 2^e to p bits, down or up; returns (m, e)."""
    bl = n.bit_length()
    if bl <= p:
        return n << (p - bl), e - (p - bl)
    sh = bl - p
    m = n >> sh
    if up and n & ((1 << sh) - 1):
        m += 1
        if m >> p:
            m >>= 1
            sh += 1
    return m, e + sh


def round_ratio(n, d, e, p, up):
    """Round the positive (n/d) * 2^e to p bits, down or up; returns (m, e)."""
    s = p + 1 - (n.bit_length() - d.bit_length())
    q, r = divmod(n << s, d) if s >= 0 else divmod(n, d << -s)
    sh = q.bit_length() - p
    m = q >> sh
    if up and (r or q & ((1 << sh) - 1)):
        m += 1
        if m >> p:
            m >>= 1
            sh += 1
    return m, e - s + sh


class Enc:
    __slots__ = ('lo', 'hi', 'c', 'o')

    def __init__(self, lo, hi, c, o):
        self.lo, self.hi, self.c, self.o = lo, hi, c, o

    def exact(self):
        return self.lo == self.hi


PER_OP = {'conversions': 0, 'products': 0, 'quotients': 0, 'failures': []}


def le_points(x, y):
    """Exact x <= y for points (m, e)."""
    a, b = aligned(x, y)
    return a <= b


def le_scaled(point, n, d):
    """Exact point <= n/d."""
    m, e = point
    return (m << e) * d <= n if e >= 0 else m * d <= n << -e


def ge_scaled(point, n, d):
    m, e = point
    return (m << e) * d >= n if e >= 0 else m * d >= n << -e


def convert(n, d, p):
    lo = round_ratio(n, d, 0, p, False)
    hi = round_ratio(n, d, 0, p, True)
    PER_OP['conversions'] += 1
    if not (le_scaled(lo, n, d) and ge_scaled(hi, n, d)):
        PER_OP['failures'].append(['conversion', n.bit_length(), d.bit_length(), p])
    return Enc(lo, hi, 0 if lo == hi else 1, 0)


def mul(a, b, p):
    lo = round_int(a.lo[0] * b.lo[0], a.lo[1] + b.lo[1], p, False)
    hi = round_int(a.hi[0] * b.hi[0], a.hi[1] + b.hi[1], p, True)
    PER_OP['products'] += 1
    exact_lo = (a.lo[0] * b.lo[0], a.lo[1] + b.lo[1])
    exact_hi = (a.hi[0] * b.hi[0], a.hi[1] + b.hi[1])
    if not (le_points(lo, exact_lo) and le_points(exact_hi, hi)):
        PER_OP['failures'].append(['product', p])
    return Enc(lo, hi, a.c + b.c, a.o + b.o + 1)


def div(a, b, p):
    lo = round_ratio(a.lo[0], b.hi[0], a.lo[1] - b.hi[1], p, False)
    hi = round_ratio(a.hi[0], b.lo[0], a.hi[1] - b.lo[1], p, True)
    PER_OP['quotients'] += 1
    # lo * b.hi <= a.lo and a.hi <= hi * b.lo, as exact p-bit products.
    lo_times = (lo[0] * b.hi[0], lo[1] + b.hi[1])
    hi_times = (hi[0] * b.lo[0], hi[1] + b.lo[1])
    if not (le_points(lo_times, a.lo) and le_points(a.hi, hi_times)):
        PER_OP['failures'].append(['quotient', p])
    return Enc(lo, hi, a.c + b.c, a.o + b.o + 1)


def one(p):
    unit = (1 << (p - 1), 1 - p)
    return Enc(unit, unit, 0, 0)


def power(base, j, p):
    """Left-to-right binary exponentiation; rounding weights sum to j - 1."""
    if j == 0:
        return one(p)
    acc = base
    for bit in bin(j)[3:]:
        acc = mul(acc, acc, p)
        if bit == '1':
            acc = mul(acc, base, p)
    return acc


def term(variant, b, q, amount, j, p):
    """Enclosure of amount / (q * b^j); `amount` None means a factor cell."""
    a_enc = None if amount is None else convert(amount.numerator, amount.denominator, p)
    if variant == 'forward':
        q_enc = convert(q.numerator, q.denominator, p)
        qb = mul(q_enc, power(convert(b.numerator, b.denominator, p), j, p), p) if j else q_enc
        return div(one(p) if a_enc is None else a_enc, qb, p)
    if variant == 'reciprocal_exact_seed':
        seed = convert(b.denominator, b.numerator, p)
    else:
        seed = div(one(p), convert(b.numerator, b.denominator, p), p)
    inv_q = convert(q.denominator, q.numerator, p)
    value = mul(inv_q, power(seed, j, p), p) if j else inv_q
    return value if a_enc is None else mul(a_enc, value, p)


def expected_weights(variant, j, c_seed, c_q, c_a, factor):
    """Closed-form certified weights (c, o) for each variant, asserted per cell.

    forward: b^j has j - 1 weighted operations, then q * b^j and A / (q b^j).
    reciprocal_exact_seed: (1/b)^j from the exact 1/b, then times 1/q and A.
    reciprocal_rounded_seed: 1 / RD/RU(b) is a weight-j operation, so the
    power carries 2j - 1 operations; it is outside the 3j-type budget.
    """
    a_op = 0 if factor else 1
    c = j * c_seed + c_q + c_a
    if variant == 'forward':
        return c, (j + 1) if j else 1
    if variant == 'reciprocal_exact_seed':
        return c, (j if j else 0) + a_op
    return c, (2 * j if j else 0) + a_op


def aligned(x, y):
    e0 = min(x[1], y[1])
    return x[0] << (x[1] - e0), y[0] << (y[1] - e0)


def bound_check(enc, p):
    """Certify hi/lo <= 1/(1 - N u), N = c + 2o; returns (ok, width / (N u)).

    (1+u) <= 1/(1-u) and gamma <= (1-u)^-2 give (1-u)^-N, and Bernoulli
    (1-u)^N >= 1 - N u. The check hi * (2^(p-1) - N) <= lo * 2^(p-1) is exact.
    """
    hi, lo = aligned(enc.hi, enc.lo)
    weight = enc.c + 2 * enc.o
    if weight == 0:
        return hi == lo, 0.0
    half = 1 << (p - 1)
    if weight >= half:
        return False, math.inf
    return hi * (half - weight) <= lo * half, ((hi - lo) << (p - 1)) / (weight * lo)


# ---------------------------------------------------------------------------
# Exact binary64 round-to-nearest-even classification.
# ---------------------------------------------------------------------------

def rn64_ratio(n, d):
    """RNE of positive n/d; (value, actual binary64 boundary tie, exact value).

    Above the overflow boundary there are no further binary64 boundaries.
    Hypothetical midpoints in larger binades must not be tagged as ties.
    """
    e = n.bit_length() - d.bit_length()
    if e > 1100:
        return math.inf, False, False
    if e < -1200:
        return 0.0, False, False
    if (n << max(0, -e)) < (d << max(0, e)):
        e -= 1
    if e >= 1024:
        return math.inf, False, False
    if e < -1075:
        return 0.0, False, False
    qe = max(e - 52, -1074)
    k, r = divmod(n, d << qe) if qe >= 0 else divmod(n << -qe, d)
    tie = 2 * r == (d << qe if qe >= 0 else d)
    den = d << qe if qe >= 0 else d
    if 2 * r > den or (tie and k & 1):
        k += 1
    if k == 0:
        return 0.0, tie, False
    try:
        return math.ldexp(k, qe), tie, r == 0
    except OverflowError:
        return math.inf, tie, False


def rn64_point(m, e):
    binade = e + m.bit_length() - 1
    if binade > 1100:
        return math.inf
    if binade < -1200:
        return 0.0
    return rn64_ratio(m << e, 1)[0] if e >= 0 else rn64_ratio(m, 1 << -e)[0]


def rn64_decimal(x):
    if x.adjusted() < -330:
        return 0.0
    if x.adjusted() > 330:
        return math.inf
    fx = F(x)
    return rn64_ratio(fx.numerator, fx.denominator)[0]


# ---------------------------------------------------------------------------
# References: exact integer ratios (no gcd on huge values) or directed Decimal.
# ---------------------------------------------------------------------------

POWER_CACHE = {}


def exact_power(b, j):
    key = (b, j)
    if key not in POWER_CACHE:
        POWER_CACHE[key] = (b.numerator ** j, b.denominator ** j)
    return POWER_CACHE[key]


def exact_term(b, q, amount, j):
    """(n, d) of amount / (q * b^j), unnormalized."""
    bn, bd = exact_power(b, j)
    a = amount if amount is not None else F(1)
    return a.numerator * q.denominator * bd, a.denominator * q.numerator * bn


def contains_exact(enc, n, d):
    lo_m, lo_e = enc.lo
    hi_m, hi_e = enc.hi
    lo_ok = (lo_m << lo_e) * d <= n if lo_e >= 0 else lo_m * d <= n << -lo_e
    hi_ok = n <= (hi_m << hi_e) * d if hi_e >= 0 else n << -hi_e <= hi_m * d
    return lo_ok and hi_ok


def contexts(digits):
    return (Context(prec=digits, rounding=ROUND_FLOOR, Emax=MAX_EMAX, Emin=MIN_EMIN),
            Context(prec=digits, rounding=ROUND_CEILING, Emax=MAX_EMAX, Emin=MIN_EMIN))


def dconv(x, ctx):
    lo, hi = ctx
    return lo.divide(Decimal(x.numerator), Decimal(x.denominator)), hi.divide(
        Decimal(x.numerator), Decimal(x.denominator))


def dmul(a, b, ctx):
    return ctx[0].multiply(a[0], b[0]), ctx[1].multiply(a[1], b[1])


def ddiv(a, b, ctx):
    return ctx[0].divide(a[0], b[1]), ctx[1].divide(a[1], b[0])


def dpow(base, j):
    ctx = base[2]
    acc = (Decimal(1), Decimal(1))
    if j:
        acc = base[:2]
        for bit in bin(j)[3:]:
            acc = dmul(acc, acc, ctx)
            if bit == '1':
                acc = dmul(acc, base[:2], ctx)
    return acc


def decimal_term(b, q, amount, j, digits):
    ctx = contexts(digits)
    bj = dpow(dconv(b, ctx) + (ctx,), j)
    qb = dmul(dconv(q, ctx), bj, ctx)
    a = dconv(amount if amount is not None else F(1), ctx)
    return ddiv(a, qb, ctx)


POW2_CACHE = {}


def decimal_point(m, e, digits):
    """Directed Decimal enclosure of m * 2^e."""
    ctx = contexts(digits)
    key = (e, digits)
    if key not in POW2_CACHE:
        base = (Decimal(2), Decimal(2)) if e >= 0 else (Decimal('0.5'), Decimal('0.5'))
        POW2_CACHE[key] = dpow(base + (ctx,), abs(e))
    lo, hi = POW2_CACHE[key]
    return ctx[0].multiply(Decimal(m), lo), ctx[1].multiply(Decimal(m), hi)


def decimal_relation(enc, ref, digits):
    """'strict' if the binary interval visibly contains the Decimal one,
    'overlap' if they only intersect, 'disjoint' otherwise (a red flag)."""
    lo_lo, lo_up = decimal_point(enc.lo[0], enc.lo[1], digits)
    hi_down, hi_up = decimal_point(enc.hi[0], enc.hi[1], digits)
    if lo_up <= ref[0] and ref[1] <= hi_down:
        return 'strict'
    if lo_lo <= ref[1] and ref[0] <= hi_up:
        return 'overlap'
    return 'disjoint'


def midpoint_distance_log2(n, d):
    """log2 distance to the nearest actual binary64 RNE boundary, relative to x.

    Both adjacent boundaries matter. Immediately below a power of two above
    MIN_NORMAL, the spacing is half the current binade's spacing. Zero and
    overflow each have only their one nearest positive boundary. Integer gap
    comparisons avoid normalizing the potentially large reference rational.
    """
    def gap(point):
        m, be = point
        if be >= 0:
            return abs(n - (m << be) * d), n
        return abs((n << -be) - m * d), n << -be

    overflow = ((1 << 54) - 1, 970)
    if n >= (overflow[0] << overflow[1]) * d:
        num, den = gap(overflow)
        return -math.inf if num == 0 else math.log2(num) - math.log2(den)
    e = n.bit_length() - d.bit_length()
    if (n << max(0, -e)) < (d << max(0, e)):
        e -= 1
    qe = max(e - 52, -1074)
    num, den = (n, d << qe) if qe >= 0 else (n << -qe, d)
    k = num // den
    candidates = [(2 * k + 1, qe - 1)]
    if k:
        candidates.append((4 * k - 1, qe - 2) if k == 1 << 52 and qe > -1074
                          else (2 * k - 1, qe - 1))
    gaps = [gap(point) for point in candidates]
    best_n, best_d = gaps[0]
    for gn, gd in gaps[1:]:
        if gn * best_d < best_n * gd:
            best_n, best_d = gn, gd
    return -math.inf if best_n == 0 else math.log2(best_n) - math.log2(best_d)


def width_bound(enc, p):
    """Certified relative width hi/lo - 1 <= N u / (1 - N u)."""
    nu = F(enc.c + 2 * enc.o, 1 << (p - 1))
    return nu / (1 - nu)


def boundary_controls():
    """Exact mathematical controls, not source examples or financial states."""
    tiny = F(1, 1 << 1074)
    overflow = F((1 << 54) - 1) * (1 << 970)
    mid = 1 + F(1, 1 << 53)
    mid_odd = 1 + F(3, 1 << 53)
    rn_rows = (
        ('zero midpoint below', tiny / 2 - tiny / (1 << 20), 0.0, False, False),
        ('zero midpoint', tiny / 2, 0.0, True, False),
        ('zero midpoint above', tiny / 2 + tiny / (1 << 20), float(tiny), False, False),
        ('MIN_SUB', tiny, float(tiny), False, True),
        ('subnormal odd tie', 3 * tiny / 2, float(2 * tiny), True, False),
        ('normal even tie', mid, 1.0, True, False),
        ('normal odd tie', mid_odd, 1 + 2.0 ** -51, True, False),
        ('overflow midpoint below', overflow - 1, sys.float_info.max, False, False),
        ('overflow midpoint', overflow, math.inf, True, False),
        ('overflow midpoint above', overflow + 1, math.inf, False, False),
        ('above-overflow hypothetical tie', F((1 << 53) + 1) * (1 << 971),
         math.inf, False, False),
    )
    rne = [{'name': name, 'ok': rn64_ratio(x.numerator, x.denominator) == (want, tie, representable),
            'rounded': str(rn64_ratio(x.numerator, x.denominator)[0]),
            'exact_tie': rn64_ratio(x.numerator, x.denominator)[1]}
           for name, x, want, tie, representable in rn_rows]
    # The exact expected boundary is specified independently for each control.
    distance_rows = (
        ('power of two', F(1), 1 - F(1, 1 << 54)),
        ('above binade edge', 1 + F(1, 1 << 60), 1 - F(1, 1 << 54)),
        ('below binade edge', 1 - F(1, 1 << 60), 1 - F(1, 1 << 54)),
        ('MIN_NORMAL uniform boundary', F(1, 1 << 1022), F(1, 1 << 1022) - tiny / 2),
        ('MIN_SUB uniform boundary', tiny, tiny / 2),
        ('underflow exact boundary', tiny / 2, tiny / 2),
        ('below underflow boundary', tiny / 4, tiny / 2),
        ('overflow exact boundary', overflow, overflow),
        ('above overflow boundary', overflow + (1 << 971), overflow),
    )
    distances = []
    for name, x, nearest in distance_rows:
        delta = abs(x - nearest) / x
        want = -math.inf if delta == 0 else math.log2(delta)
        actual = midpoint_distance_log2(x.numerator, x.denominator)
        distances.append({'name': name, 'log2': actual,
                          'ok': actual == want or abs(actual - want) < 1e-12})
    intervals = []
    for name, left, right, decided in (
        ('exact midpoint point', mid, mid, True),
        ('touching tie-even upper endpoint', F(1), mid, True),
        ('strict straddle', F(1), 1 + F(1, 1 << 52), False),
        ('touching tie-even lower endpoint', mid_odd, 1 + F(1, 1 << 51), True),
    ):
        lo = convert(left.numerator, left.denominator, 192).lo
        hi = convert(right.numerator, right.denominator, 192).hi
        actual = rn64_point(*lo) == rn64_point(*hi)
        intervals.append({'name': name, 'decided': actual, 'ok': actual == decided})
    return {'scope': 'exact mathematical helper controls, not financial states',
            'rne': rne, 'nearest_boundary': distances, 'midpoint_intervals': intervals,
            'ok': all(row['ok'] for rows in (rne, distances, intervals) for row in rows)}


# ---------------------------------------------------------------------------
# Cells.
# ---------------------------------------------------------------------------

def classify(enc, ref, p, stats, cell):
    """Count containment, bound, RNE decision and boundary proximity."""
    if ref[0] == 'exact':
        n, d = ref[1]
        stats['end_to_end_exact'] += 1
        if not contains_exact(enc, n, d):
            stats['end_to_end_failures'].append(cell)
    else:
        stats['decimal_crosscheck'] += 1
        relations = [decimal_relation(enc, r, digits) for digits, r in ref[1]]
        if any(rel == 'disjoint' for rel in relations):
            stats['decimal_crosscheck_disjoint'].append(cell)
        elif any(rel == 'overlap' for rel in relations):
            stats['decimal_crosscheck_overlap_only'].append(cell)
    ok, ratio = bound_check(enc, p)
    if ratio > stats['max_width_over_linear_bound']:
        stats['max_width_over_linear_bound'] = ratio
        stats['max_width_cell'] = [cell, enc.c, enc.o]
    if not ok:
        stats['bound_failures'].append(cell)
    lo_rn = rn64_point(*enc.lo)
    hi_rn = rn64_point(*enc.hi)
    if enc.exact():
        stats['exact_enclosures'] += 1
    if lo_rn != hi_rn:
        stats['unresolved'] += 1
        stats['unresolved_cells'].append(cell)
    else:
        stats['decided'] += 1
        kind = 'overflow' if lo_rn == math.inf else 'zero' if lo_rn == 0.0 else (
            'subnormal' if lo_rn < 2.2250738585072014e-308 else 'normal')
        stats['decided_kinds'][kind] = stats['decided_kinds'].get(kind, 0) + 1
    width = width_bound(enc, p)
    if ref[0] == 'exact':
        n, d = ref[1]
        _, tie, representable = rn64_ratio(n, d)
        stats['exact_ties'] += tie
        if tie and not enc.exact():
            stats['inexact_tie_cells'].append(cell)
        stats['exactly_representable'] += representable
        distance = midpoint_distance_log2(n, d)
        if distance is not None and not enc.exact():
            margin = distance - math.log2(width)
            if margin < stats['min_midpoint_margin_bits']:
                stats['min_midpoint_margin_bits'] = margin
                stats['min_midpoint_cell'] = [cell, round(distance, 3), round(math.log2(width), 3)]
        near_lo = rn64_ratio(n * (width.denominator - width.numerator), d * width.denominator)[0]
        near_hi = rn64_ratio(n * (width.denominator + width.numerator), d * width.denominator)[0]
    else:
        digits, (rlo, rhi) = ref[1][-1]
        ctx = contexts(digits)
        w = ctx[1].divide(Decimal(width.numerator), Decimal(width.denominator))
        near_lo = rn64_decimal(ctx[0].multiply(rlo, ctx[0].subtract(Decimal(1), w)))
        near_hi = rn64_decimal(ctx[1].multiply(rhi, ctx[1].add(Decimal(1), w)))
    if near_lo != near_hi:
        stats['near_boundary'] += 1
        stats['near_cells'].append(cell)


def new_stats():
    return {
        'cells': 0, 'end_to_end_exact': 0, 'decimal_crosscheck': 0,
        'end_to_end_failures': [], 'decimal_crosscheck_overlap_only': [],
        'decimal_crosscheck_disjoint': [], 'bound_failures': [], 'weight_failures': [],
        'max_width_over_linear_bound': 0.0, 'max_width_cell': None,
        'min_midpoint_margin_bits': math.inf, 'min_midpoint_cell': None,
        'exact_enclosures': 0, 'decided': 0, 'decided_kinds': {}, 'unresolved': 0,
        'unresolved_cells': [], 'near_boundary': 0, 'near_cells': [], 'exact_ties': 0,
        'inexact_tie_cells': [], 'exactly_representable': 0,
    }


def reference(b, q, amount, j, decimal_refs):
    bits = max(b.numerator.bit_length(), b.denominator.bit_length()) * j
    if bits <= EXACT_BIT_BUDGET:
        return ('exact', exact_term(b, q, amount, j))
    key = (b, q, amount, j)
    if key not in decimal_refs:
        decimal_refs[key] = [(digits, decimal_term(b, q, amount, j, digits)) for digits in DECIMAL_DIGITS]
    return ('decimal', decimal_refs[key])


def run_cells(label, rows, stats, decimal_refs):
    """rows: (cell_id, y, r, s, amount or None for a factor cell, j)."""
    for cell_id, y, r, s, amount, j in rows:
        b = 1 + y / 2
        q = (2 * s + r * y) / (2 * s)
        ref = reference(b, q, amount, j, decimal_refs)
        for p in PRECISIONS:
            cb = convert(b.numerator, b.denominator, p).c
            cq = convert(q.numerator, q.denominator, p).c
            ca = 0 if amount is None else convert(amount.numerator, amount.denominator, p).c
            for variant in VARIANTS:
                enc = term(variant, b, q, amount, j, p)
                cell = f'{label}:{cell_id}:p{p}:{variant}'
                stats['cells'] += 1
                cqi = convert(q.denominator, q.numerator, p).c
                if variant == 'forward':
                    want = expected_weights(variant, j, cb, cq, ca, amount is None)
                elif variant == 'reciprocal_exact_seed':
                    ci = convert(b.denominator, b.numerator, p).c
                    want = expected_weights(variant, j, ci, cqi, ca, amount is None)
                else:
                    want = expected_weights(variant, j, cb, cqi, ca, amount is None)
                if (enc.c, enc.o) != want:
                    stats['weight_failures'].append([cell, [enc.c, enc.o], list(want)])
                classify(enc, ref, p, stats, cell)


def source_bundles(case):
    name, c, _, r, s, n, extra = case[:7]
    half = F(c) / 2
    amounts = {j: half for j in range(1, n + 1)}
    amounts[n] += 100
    if name in ('A', 'B'):
        amounts[0] = half * F(r, s)
    elif name == 'C':
        amounts[1] += half * F(r, s)
    elif name == 'D':
        amounts[0] = half
    elif name == 'E':
        amounts[0] = F(extra[0], extra[1]) * half + half
    elif name == 'F':
        amounts[0] = F(extra[0], s) * half
    elif name == 'G':
        amounts[1] += F(extra[0], s) * half
    return amounts


def source_ai(case):
    name, c, _, r, s, _, extra = case[:7]
    half = F(c) / 2
    if name == 'D':
        return F(s - r, s) * half
    if name == 'E':
        return F(extra[0], extra[1]) * half + F(s - r, s) * half
    if name in ('F', 'G'):
        return F(extra[0] - r, s) * half
    return F(0)


def rn6_half_up(x):
    scaled = x * 10 ** 6
    k = math.floor(scaled + F(1, 2))
    return F(k, 10 ** 6), scaled - math.floor(scaled) == F(1, 2)


def add_enc(a, b, p):
    """Outward sum of positive enclosures (sum weights are not tracked)."""
    lo_a, lo_b = aligned(a.lo, b.lo)
    hi_a, hi_b = aligned(a.hi, b.hi)
    lo = round_int(lo_a + lo_b, min(a.lo[1], b.lo[1]), p, False)
    hi = round_int(hi_a + hi_b, min(a.hi[1], b.hi[1]), p, True)
    return Enc(lo, hi, None, None)


def to_fraction(point):
    m, e = point
    return F(m << e) if e >= 0 else F(m, 1 << -e)


def source_section(xml_text, stats, decimal_refs):
    report = []
    rows = []
    for case in CASES:
        name, _, i, r, s, n, _, dirty_txt, p_txt, anchors = case
        missing = [a for a in anchors if a not in xml_text]
        amounts = source_bundles(case)
        y_dec = exact(i)
        y_bin = exact(float(i))
        entry = {'case': name, 'missing_anchors': missing, 'bundles': len(amounts)}
        for y_label, y in (('decimal', y_dec), ('binary64', y_bin)):
            b = 1 + y / 2
            q = (2 * s + r * y) / (2 * s)
            dirty = sum((a / (q * b ** j) for j, a in amounts.items()), F(0))
            rn_dirty, tie_dirty = rn6_half_up(dirty)
            rn_ai, tie_ai = rn6_half_up(source_ai(case))
            entry[y_label] = {
                'rn6_dirty': str(rn_dirty), 'printed_dirty': dirty_txt,
                'dirty_matches': rn_dirty == F(dirty_txt), 'tie': tie_dirty or tie_ai,
                'r3_clean': str(rn_dirty - rn_ai), 'printed_p': p_txt,
                'r3_matches': rn_dirty - rn_ai == F(p_txt),
            }
            for p in PRECISIONS:
                total = None
                for j, a in sorted(amounts.items()):
                    enc = term('forward', b, q, a, j, p)
                    total = enc if total is None else add_enc(total, enc, p)
                lo, hi = to_fraction(total.lo), to_fraction(total.hi)
                entry[y_label][f'p{p}_dirty_contains_exact'] = lo <= dirty <= hi
                entry[y_label][f'p{p}_dirty_rn6_decided'] = (
                    rn6_half_up(lo)[0] == rn6_half_up(hi)[0] == rn_dirty)
                entry[y_label][f'p{p}_dirty_rel_width_log2'] = round(
                    math.log2((hi - lo) / lo) if hi != lo else -math.inf, 3)
            for j, a in sorted(amounts.items()):
                rows.append((f'{name}:{y_label}:j{j}', y, r, s, a, j))
        report.append(entry)
    run_cells('source', rows, stats, decimal_refs)
    return report


def edge_section(stats, decimal_refs):
    rows = []
    for y_label, y in EDGE_Y:
        for r, s in EDGE_RS:
            for j in EDGE_J:
                rows.append((f'y={y_label}:r{r}/s{s}:j{j}', exact(y), r, s, None, j))
    run_cells('edge', rows, stats, decimal_refs)
    return len(rows)


def variant_table():
    """Observed and certified widths per variant at the j cap, r/s = 92/184."""
    rows = []
    for y_label, y in (('MIN_SUB', 5e-324), ('RN(.0884)', 0.0884), ('-1.9', -1.9), ('1e300', 1e300)):
        fy = exact(y)
        b = 1 + fy / 2
        q = (2 * 184 + 92 * fy) / (2 * 184)
        for p in PRECISIONS:
            for variant in VARIANTS:
                enc = term(variant, b, q, None, J_CAP, p)
                hi, lo = aligned(enc.hi, enc.lo)
                rows.append({'y': y_label, 'p': p, 'variant': variant, 'c': enc.c, 'o': enc.o,
                             'N': enc.c + 2 * enc.o,
                             'observed_log2': round(math.log2((hi - lo) / lo), 3),
                             'certified_log2': round(math.log2(float(width_bound(enc, p))), 3)})
    return rows


def counterexample_section():
    """y = MIN_SUB: b = 1 + 2^-1075, outward base [1, 1 + 2^(1-p)]."""
    out = {}
    y = exact(5e-324)
    b = 1 + y / 2
    for p in PRECISIONS:
        u = F(2) ** (1 - p)
        base = convert(b.numerator, b.denominator, p)
        assert to_fraction(base.lo) == 1 and to_fraction(base.hi) == 1 + u
        bj = power(base, J_CAP, p)
        hi, lo = aligned(bj.hi, bj.lo)
        lower_ok = (hi - lo) << (p - 1) >= J_CAP * lo
        seed = convert(b.denominator, b.numerator, p)
        assert to_fraction(seed.lo) == 1 - u / 2 and to_fraction(seed.hi) == 1
        sj = power(seed, J_CAP, p)
        shi, slo = aligned(sj.hi, sj.lo)
        seed_lower_ok = (shi - slo) << p >= J_CAP * slo
        # Width of [1, (1+u)^j] if every later product were exact; 2^(1-p) and
        # 1 + 2^(1-p) are exact at 300 digits, the power is display-only.
        ctx = Context(prec=300)
        exact_products = ctx.subtract(ctx.power(ctx.add(Decimal(1), ctx.power(Decimal(2), 1 - p)), J_CAP), 1)
        out[f'p{p}'] = {
            'base_interval': f'[1, 1+2^{1 - p}]',
            'helper_weights_c_o': [bj.c, bj.o],
            'helper_rel_width_log2': round(math.log2((hi - lo) / lo), 4),
            'helper_width_at_least_j_2^(1-p)': lower_ok,
            'exact_products_width_log2': round(float(exact_products.ln(ctx) / Decimal(2).ln(ctx)), 4),
            'j_2^(1-p)_log2': round(math.log2(J_CAP) + 1 - p, 4),
            'certified_upper_log2': round(math.log2(float(width_bound(bj, p))), 4),
            'reciprocal_seed_interval': f'[1-2^{-p}, 1]',
            'reciprocal_rel_width_log2': round(math.log2((shi - slo) / slo), 4),
            'reciprocal_width_at_least_j_2^-p': seed_lower_ok,
            'width_over_2^-185': round(((hi - lo) / lo) / 2.0 ** -185, 3) if p == 192 else None,
        }
    return out


# ---------------------------------------------------------------------------
# Coupled five-word widths for the candidate derivative ratios.
# ---------------------------------------------------------------------------

def decode(value):
    """(significand, exponent) with value = sig * 2^exp and 53-bit sig."""
    m, e = math.frexp(value)
    sig = int(m * (1 << 53))
    return sig, e - 53


def rn64_float(n, d):
    return rn64_ratio(n, d)[0]


def extended_ratio_peak(num, den):
    """Model the shift steps of gilt's extended_ratio; returns peak bits."""
    nb, db = num.bit_length(), den.bit_length()
    k0 = nb - db
    peaks = [nb, db]
    if k0 >= 0:
        t = den << k0
        at_least = num >= t
    else:
        t = num << -k0
        at_least = t >= den
    peaks.append(t.bit_length())
    k = k0 if at_least else k0 - 1
    shift = 52 - k
    a, b = (num << shift, den) if shift >= 0 else (num, den << -shift)
    peaks += [a.bit_length(), b.bit_length()]
    quotient = a // b
    assert (1 << 52) <= quotient < (1 << 53)
    return max(peaks)


def coupled_y():
    rng = random.Random(SEED)
    ys = []
    for _ in range(5000):
        ys.append(-2 * rng.random())
        ys.append(4 * rng.random())
        ys.append(10 ** rng.uniform(-300, 308))
        ys.append(-2 + 10 ** rng.uniform(-16, 0))
    ys += [-2 + k * 2.0 ** -52 for k in range(1, 200)]
    for edge in (1.0, -1.0, 2.0, 5e-324, 2.2250738585072014e-308, 1e300, sys.float_info.max):
        for v in (edge, math.nextafter(edge, math.inf), math.nextafter(edge, -math.inf)):
            ys.append(v)
    ys += [0.0, -0.0]
    return [y for y in ys if y > -2 and math.isfinite(y)]


def coupled_rs():
    pairs = []
    for s in range(181, 185):
        for r in sorted({1, 2, 3, s // 2, (s + 1) // 2, s - 2, s - 1, s}):
            pairs.append((r, s))
    return pairs


def coupled_exponent_premise():
    """Exact constants used by the analytic proof, distinct from sampling.

    delta=2+y >= 2^-52 and q>=2^-53 are positive normal and round finitely.
    For -2<y<0, q>=delta/2, so RNE monotonicity and exact normal scaling give
    q_hat>=delta_hat/2: the binade difference is <=1. For y>=-1 it is >=0;
    for y<-1, q<1-1/368<1-2^-54, so q_hat<1 and the difference is >=-51.
    For y>=0, 2<=delta/q<=368 and normal RNE relative error <=2^-53 gives
    1<delta_hat/q_hat<512. Thus the binade difference is in [0,9].
    The 53-bit normalized exponent difference equals that binade difference.
    """
    constants = {
        '368*(2^53+1)<512*(2^53-1)': 368 * ((1 << 53) + 1) < 512 * ((1 << 53) - 1),
        '2*(2^53-1)>2^53+1': 2 * ((1 << 53) - 1) > (1 << 53) + 1,
        '1/368>2^-54': F(1, 368) > F(1, 1 << 54),
        'MAX+2<overflow_midpoint': F(sys.float_info.max) + 2 < F((1 << 54) - 1) * (1 << 970),
        '1+MAX/2<MAX': 1 + F(sys.float_info.max) / 2 < F(sys.float_info.max),
        'next_up(-2)+2=2^-52': F(math.nextafter(-2.0, math.inf)) + 2 == F(1, 1 << 52),
    }
    return {'scope': 'analytic proof above and in report; constants checked exactly; not sampled',
            'proven_range': [-51, 9], 'negative_y_range': [-51, 1],
            'nonnegative_y_range': [0, 9], 'exact_constant_checks': constants,
            'ok': all(constants.values())}


def coupled_section():
    ys = coupled_y()
    pairs = coupled_rs()
    pv = (1 << 53) - 1
    j = J_CAP
    out = {'y_count': len(ys), 'rs_pairs': len(pairs), 'j': j, 'pv_significand': pv}
    diff = [10 ** 9, -10 ** 9]
    peaks = {'second_num_bits': 0, 'second_den_bits': 0, 'second_peak': 0,
             'first_num_bits': 0, 'first_den_bits': 0, 'first_peak': 0,
             'moment_peak': 0}
    worst = None
    term_bits = [0, 0, 0]
    for y in ys:
        fy = exact(y)
        delta = rn64_float((2 + fy).numerator, (2 + fy).denominator)
        dsig, dexp = decode(delta)
        for r, s in pairs:
            qx = (2 * s + r * fy) / (2 * s)
            qhat = rn64_float(qx.numerator, qx.denominator)
            qsig, qexp = decode(qhat)
            d_e = dexp - qexp
            diff = [min(diff[0], d_e), max(diff[1], d_e)]
            t1 = 2 * j * (j + 1) * s * s * qsig * qsig
            t2 = 2 * j * r * s * dsig * qsig
            t3 = r * r * dsig * dsig
            term_bits = [max(term_bits[0], t1.bit_length()), max(term_bits[1], t2.bit_length()),
                         max(term_bits[2], t3.bit_length())]
            m2 = 2 * min(dexp, qexp)
            num2 = pv * ((t1 << (2 * qexp - m2)) + (t2 << (dexp + qexp - m2)) + (t3 << (2 * dexp - m2)))
            den2 = 2 * s * s * dsig * dsig * qsig * qsig
            m1 = min(dexp, qexp)
            num1 = pv * (((2 * s * j * qsig) << (qexp - m1)) + ((r * dsig) << (dexp - m1)))
            den1 = 2 * s * dsig * qsig
            p2 = extended_ratio_peak(num2, den2)
            p1 = extended_ratio_peak(num1, den1)
            pm = extended_ratio_peak(pv * (r + j * s), 2 * s)
            if num2.bit_length() > peaks['second_num_bits']:
                worst = {'y': repr(y), 'r': r, 's': s, 'delta_hat': repr(delta), 'q_hat': repr(qhat)}
            peaks['second_num_bits'] = max(peaks['second_num_bits'], num2.bit_length())
            peaks['second_den_bits'] = max(peaks['second_den_bits'], den2.bit_length())
            peaks['second_peak'] = max(peaks['second_peak'], p2)
            peaks['first_num_bits'] = max(peaks['first_num_bits'], num1.bit_length())
            peaks['first_den_bits'] = max(peaks['first_den_bits'], den1.bit_length())
            peaks['first_peak'] = max(peaks['first_peak'], p1)
            peaks['moment_peak'] = max(peaks['moment_peak'], pm)
    out['e_delta_minus_e_q'] = diff
    out['unaligned_term_bits'] = term_bits
    out.update(peaks)
    out['second_worst_input'] = worst
    premise = coupled_exponent_premise()
    # Preserve the conservative [-52,9] width budget. The analytic premise
    # proves the stronger [-51,9]; sampling is independent confirmation only.
    analytic = {
        't1_lt_2^152': (2 * j * (j + 1) * 184 * 184 * ((1 << 53) - 1) ** 2).bit_length() <= 152,
        't2_lt_2^138': (2 * j * 184 * 184 * ((1 << 53) - 1) ** 2).bit_length() <= 138,
        't3_lt_2^122': (184 * 184 * ((1 << 53) - 1) ** 2).bit_length() <= 122,
        'max_alignment_bits': 2 * 52,
        'alignment_range_used': [-52, 9],
        'negative_difference_sum_bits_le': max(152 + 104, 138 + 52, 122) + 2,
        'positive_difference_sum_bits_le': max(152, 138 + 9, 122 + 18) + 2,
        'aligned_sum_bits_le': max(152 + 104, 138 + 52, 122) + 2,
        'numerator_bits_le': max(152 + 104, 138 + 52, 122) + 2 + 53,
        'denominator_bits_le': (2 * 184 * 184 * ((1 << 53) - 1) ** 4).bit_length(),
        'extended_ratio_admits': 'num bits <= 320 and den bits <= 267',
    }
    analytic['holds'] = (analytic['t1_lt_2^152'] and analytic['t2_lt_2^138'] and analytic['t3_lt_2^122']
                         and analytic['numerator_bits_le'] <= 312 and analytic['denominator_bits_le'] <= 267)
    out['analytic'] = analytic
    out['analytic_exponent_premise'] = premise
    out['ok'] = (premise['ok'] and -51 <= diff[0] and diff[1] <= 9 and peaks['second_num_bits'] <= 312
                 and peaks['second_den_bits'] <= 229 and max(peaks['second_peak'], peaks['first_peak'],
                                                             peaks['moment_peak']) <= WIDTH_LIMIT
                 and analytic['holds'])
    return out


# ---------------------------------------------------------------------------
# Clean-recipe tie-rule qualification: RN6(D - a) versus RN6(D) - a.
# ---------------------------------------------------------------------------

def round_grid(x, rule):
    """Round x (Fraction, units of 1e-6 already applied) to an integer."""
    fl = math.floor(x)
    frac = x - fl
    if frac != F(1, 2):
        return fl + (frac > F(1, 2))
    if rule == 'ties_toward_positive':
        return fl + 1
    if rule == 'ties_away_from_zero':
        return fl + 1 if x > 0 else fl
    return fl + (fl & 1)


def tie_rule_section():
    rng = random.Random(SEED + 1)
    rules = ('ties_toward_positive', 'ties_away_from_zero', 'ties_to_even')
    sets = {'non_tie_any_sign': [], 'tie_both_nonnegative': [], 'tie_crossing_zero': []}
    for _ in range(20000):
        a = rng.randrange(0, 10 ** 8)
        x = F(rng.randrange(-10 ** 14, 10 ** 15), 10 ** 6) + F(1, 3)
        sets['non_tie_any_sign'].append((x, a))
    for _ in range(10000):
        k = rng.randrange(0, 10 ** 8)
        a = rng.randrange(0, k + 1)
        sets['tie_both_nonnegative'].append((F(2 * k + 1, 2), a))
        k = rng.randrange(0, 10 ** 8)
        a = rng.randrange(k + 1, k + 10 ** 8)
        sets['tie_crossing_zero'].append((F(2 * k + 1, 2), a))
    table = {}
    for rule in rules:
        table[rule] = {name: sum(round_grid(x - a, rule) != round_grid(x, rule) - a for x, a in rows)
                       for name, rows in sets.items()}
    expected = (all(v == 0 for v in table['ties_toward_positive'].values())
                and table['ties_away_from_zero']['non_tie_any_sign'] == 0
                and table['ties_away_from_zero']['tie_both_nonnegative'] == 0
                and table['ties_away_from_zero']['tie_crossing_zero'] == len(sets['tie_crossing_zero'])
                and table['ties_to_even']['non_tie_any_sign'] == 0
                and table['ties_to_even']['tie_both_nonnegative'] > 0)
    return {'cases': {k: len(v) for k, v in sets.items()}, 'differences': table, 'expected': expected}


def summarize(stats, limit=20):
    out = dict(stats)
    for key in ('end_to_end_failures', 'decimal_crosscheck_overlap_only', 'decimal_crosscheck_disjoint',
                'bound_failures', 'weight_failures', 'unresolved_cells', 'near_cells', 'inexact_tie_cells'):
        out[key + '_count'] = len(stats[key])
        out[key] = stats[key][:limit]
    out['max_width_over_linear_bound'] = round(stats['max_width_over_linear_bound'], 6)
    return out


def source_observations_ok(entries):
    """Require every claimed source observation for both exact yield forms."""
    if [entry['case'] for entry in entries] != [case[0] for case in CASES]:
        return False
    return all(not entry['missing_anchors'] and all(
        entry[y]['dirty_matches'] and entry[y]['r3_matches'] and not entry[y]['tie']
        and all(entry[y][f'p{p}_dirty_contains_exact']
                and entry[y][f'p{p}_dirty_rn6_decided'] for p in PRECISIONS)
        for y in ('decimal', 'binary64')) for entry in entries)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--source-xml', required=True, type=Path,
                        help='pinned eCFR Part 356 XML snapshot (2026-10-16)')
    parser.add_argument('--output', required=True, type=Path, help='new JSON file; never overwritten')
    args = parser.parse_args()
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    xml_bytes = args.source_xml.read_bytes()
    if sha(xml_bytes) != SOURCE_XML_SHA:
        raise SystemExit('source XML hash mismatch')
    xml_text = xml_bytes.decode('utf-8')
    decimal_refs = {}

    json_controls = strict_json_controls()
    controls = boundary_controls()
    source_stats = new_stats()
    source = source_section(xml_text, source_stats, decimal_refs)
    edge_stats = new_stats()
    edge_rows = edge_section(edge_stats, decimal_refs)
    counter = counterexample_section()
    variants = variant_table()
    coupled = coupled_section()
    ties = tie_rule_section()

    source_ok = source_observations_ok(source)
    cells_ok = all(not s[k] for s in (source_stats, edge_stats)
                   for k in ('end_to_end_failures', 'decimal_crosscheck_disjoint', 'bound_failures',
                             'weight_failures'))
    per_op_ok = not PER_OP['failures']
    counter_ok = all(v['helper_width_at_least_j_2^(1-p)'] and v['reciprocal_width_at_least_j_2^-p']
                     for v in counter.values())
    result = 'PASS' if (json_controls['ok'] and controls['ok'] and source_ok and cells_ok
                        and per_op_ok and counter_ok and coupled['ok']
                        and ties['expected']) else 'FAIL'
    evidence = {
        'scope': 'research-only Treasury outward-power design check; no contract, API, fixture or production',
        'result': result,
        'started_utc': started,
        'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
        'environment': {'python': sys.version, 'implementation': platform.python_implementation(),
                        'platform': platform.platform(), 'machine': platform.machine()},
        'script_sha256': sha(Path(__file__).read_bytes()),
        'source_xml': {'path': str(args.source_xml), 'sha256': SOURCE_XML_SHA},
        'parameters': {'precisions': PRECISIONS, 'j_cap': J_CAP, 'exact_bit_budget': EXACT_BIT_BUDGET,
                       'decimal_digits': DECIMAL_DIGITS, 'variants': VARIANTS, 'seed': SEED,
                       'edge_y': [label for label, _ in EDGE_Y], 'edge_rs': EDGE_RS, 'edge_j': EDGE_J},
        'gates': {'strict_json_controls': json_controls['ok'],
                  'exact_boundary_controls': controls['ok'],
                  'source': source_ok, 'cells_end_to_end_bound_weights': cells_ok,
                  'per_operation_induction': per_op_ok, 'counterexample': counter_ok,
                  'coupled_widths': coupled['ok'], 'tie_rules': ties['expected']},
        'per_operation_checks': {k: (v if k != 'failures' else v[:20]) for k, v in PER_OP.items()},
        'per_operation_failure_count': len(PER_OP['failures']),
        'strict_json_controls': json_controls,
        'exact_boundary_controls': controls,
        'source_cases': source,
        'source_cells': summarize(source_stats),
        'edge_factor_rows': edge_rows,
        'edge_cells': summarize(edge_stats),
        'min_sub_counterexample': counter,
        'variant_widths_at_j_cap': variants,
        'coupled_widths': coupled,
        'clean_recipe_tie_rules': ties,
    }
    serialized = strict_json_text(evidence, indent=2)
    with args.output.open('x') as stream:
        stream.write(serialized)
    json.loads(args.output.read_text(), parse_constant=reject_json_constant)
    brief = {k: evidence[k] for k in ('result', 'gates')}
    brief['source_cells'] = {k: evidence['source_cells'][k] for k in (
        'cells', 'decided', 'unresolved', 'near_boundary', 'exact_ties', 'exact_enclosures')}
    brief['edge_cells'] = {k: evidence['edge_cells'][k] for k in (
        'cells', 'decided', 'unresolved', 'near_boundary', 'exact_ties', 'exact_enclosures',
        'decimal_crosscheck', 'decided_kinds')}
    brief['per_operation_checks'] = {k: v for k, v in PER_OP.items() if k != 'failures'}
    print(strict_json_text(brief))
    return 0 if result == 'PASS' else 1


if __name__ == '__main__':
    raise SystemExit(main())
