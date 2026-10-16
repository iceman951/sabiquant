#!/usr/bin/env python3
"""Independent phase-B binary64 boundary oracle; never reads Rust or reference outputs.

Every binary64 input is a 64-bit pattern decoded with integer shifts into the
exact rational (-1)^sign * M * 2^e. Python integers and Fraction values are the
only authority: exact incoming conversion removes trailing zero bits, rounded
incoming conversion takes the integer quotient/remainder of the contract's A/B,
and outgoing rounding selects exponent, 53-bit significand and tie with exact
integer comparisons. Python float objects appear only inside assertion
cross-checks (struct/float.hex/as_integer_ratio decomposition, exact
Decimal.from_float expansion, correctly rounded int/int true division and
decimal-string parsing); none of them decides an expectation. Rounded incoming
quotients are separately checked with reduced Fraction values, one Decimal
division at 768 and 1024 significant digits and, for power-of-ten quanta, a
division-free Decimal.quantize of the exact expansion. See
docs/M8_BINARY_BOUNDARY_ORACLES.md for method, corpus and precision proof.
No non-stdlib imports are used.
"""

import argparse
from collections import Counter
import datetime as dt
from decimal import (
    Context, Decimal, ROUND_CEILING, ROUND_DOWN, ROUND_FLOOR, ROUND_HALF_EVEN,
    ROUND_HALF_UP, ROUND_UP,
)
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import platform
import re
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/amounts/binary-boundary-oracles.fixture"
CONTRACT = ROOT / "docs/M8_BINARY_BOUNDARY_CONTRACT.md"
CONTRACT_SHA = "7f3c24a089e17115b9d85527797c79617a4e7e599f8bb4474aed5a7be2ccec31"
AMOUNT_CONTRACT = ROOT / "docs/M8_AMOUNT_CONTRACT.md"
AMOUNT_CONTRACT_SHA = "954571eb64be97e71c68877c8e5c705f5ca9312a2109d6b54f6a1883aead7df1"
REQUIREMENTS = ROOT / "docs/REQUIREMENTS.md"

MIN, MAX = -(2**127), 2**127 - 1
MAX_SCALE = 38
SIGN = 1 << 63
FRACTION_MASK = 2**52 - 1
POSITIVE_INFINITY = 0x7FF << 52
F64_MAX = POSITIVE_INFINITY - 1
MODES = (
    "TowardZero", "AwayFromZero", "TowardPositiveInfinity",
    "TowardNegativeInfinity", "NearestTiesEven", "NearestTiesAwayFromZero", "Exact",
)
DECIMAL_MODES = dict(zip(MODES[:-1], (
    ROUND_DOWN, ROUND_UP, ROUND_CEILING, ROUND_FLOOR, ROUND_HALF_EVEN, ROUND_HALF_UP,
)))
# A/B < 2^1152 < 10^347 and a non-tie remainder is at least 1/(2B) > 10^-363:
# both precisions exceed the 710 digits that separation requires.
CONTEXTS = (Context(prec=768), Context(prec=1024))
COLUMNS = "id profile op bits amount quantum mode expected context output".split()
OP_FIELDS = {
    "from_exact": ("bits",),
    "from_rounded": ("bits", "quantum", "mode"),
    "to_exact": ("amount",),
    "to_nearest": ("amount",),
}
OPERATION = {
    "from_exact": "from_binary64_exact",
    "from_rounded": "from_binary64_rounded",
    "to_exact": "to_binary64_exact",
}
HEX_FLOAT = re.compile(r"(-?)0x([01])\.([0-9a-f]+)p([+-][0-9]+)\Z")

COUNTS = Counter()
PROFILES = Counter()
OPS = Counter()
EXPECTATIONS = Counter()
ERROR_OPERATIONS = Counter()
WIDTHS = Counter()
OUTGOING = []
ROUNDED = {}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    value = 14695981039346656037
    for byte in data:
        value = ((value ^ byte) * 1099511628211) & (2**64 - 1)
    return "%016x" % value


def hexbits(bits):
    return "0x%016x" % bits


def cell(pair):
    return "-" if pair is None else "%d:%d" % pair


def widen(key, value):
    WIDTHS[key] = max(WIDTHS[key], value.bit_length())


def canonical(coefficient, scale):
    if coefficient == 0:
        return 0, 0
    while scale > 0 and coefficient % 10 == 0:
        coefficient //= 10
        scale -= 1
    return coefficient, scale


def failure(kind, *context):
    return "amount." + kind, ",".join("%s=%s" % item for item in context), "-"


# ---------------------------------------------------------------------------
# Binary64 bit fields: the only route from an input pattern to its value.

def decode(bits):
    """Return (kind, sign, M, e) read from the IEEE 754 fields alone."""
    assert 0 <= bits < 2**64
    sign = bits >> 63
    field = (bits >> 52) & 0x7FF
    fraction = bits & FRACTION_MASK
    if field == 0x7FF:
        return ("nan" if fraction else "inf"), sign, None, None
    if field == 0:
        significand, exponent = fraction, -1074
    else:
        significand, exponent = fraction | 2**52, field - 1075
    assert significand < 2**53 and -1074 <= exponent <= 971
    return "finite", sign, significand, exponent


def magnitude(significand, exponent):
    if exponent >= 0:
        return Fraction(significand << exponent)
    return Fraction(significand, 1 << -exponent)


def exact_value(bits):
    kind, sign, significand, exponent = decode(bits)
    assert kind == "finite"
    value = magnitude(significand, exponent)
    return -value if sign else value


def as_float(bits):
    """Assertion-only view of a pattern as a Python float."""
    return struct.unpack("<d", struct.pack("<Q", bits))[0]


def float_bits(value):
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def parse_float_hex(text):
    match = HEX_FLOAT.match(text)
    assert match, text
    sign, lead, digits, exponent = match.groups()
    value = Fraction(int(lead + digits, 16)) * Fraction(2) ** (int(exponent) - 4 * len(digits))
    return -value if sign else value


def check_decode(bits):
    """Cross-check the field decoder against three independent CPython views."""
    kind, sign, significand, exponent = decode(bits)
    number = as_float(bits)
    if kind == "nan":
        assert math.isnan(number)
        COUNTS["nonfinite-class-checks"] += 1
        return
    if kind == "inf":
        assert math.isinf(number) and (math.copysign(1.0, number) < 0) == bool(sign)
        COUNTS["nonfinite-class-checks"] += 1
        return
    value = exact_value(bits)
    assert Fraction(*number.as_integer_ratio()) == value
    assert parse_float_hex(number.hex()) == value
    assert (math.copysign(1.0, number) < 0) == bool(sign)
    assert float_bits(number) == bits
    COUNTS["decode-struct-hex-ratio-checks"] += 1


def round_magnitude(numerator, denominator, how):
    """Round positive numerator/denominator to binary64 with integers only.

    Returns positive bits and (class, rounded up, carried, shift, shifted
    numerator, divisor). `how` is nearest (ties to even significand), down or up.
    """
    assert numerator > 0 and denominator > 0 and how in ("nearest", "down", "up")
    exponent = numerator.bit_length() - denominator.bit_length()
    if exponent >= 0:
        below = numerator < denominator << exponent
    else:
        below = numerator << -exponent < denominator
    exponent -= below
    # Now 2^exponent <= value < 2^(exponent+1), decided by exact comparison.
    unit = max(exponent, -1022) - 52
    if unit >= 0:
        shifted, divisor = numerator, denominator << unit
    else:
        shifted, divisor = numerator << -unit, denominator
    significand, remainder = divmod(shifted, divisor)
    twice = 2 * remainder
    if remainder == 0:
        position = "exact"
    elif twice < divisor:
        position = "below-half"
    elif twice == divisor:
        position = "half"
    else:
        position = "above-half"
    if how == "nearest":
        up = twice > divisor or (twice == divisor and significand & 1 == 1)
    else:
        up = how == "up" and remainder > 0
    significand += up
    if exponent < -1022:
        bits = significand                      # subnormal; 2^52 is the minimum normal
        carried = up and significand == 2**52
    else:
        bits = ((exponent + 1022) << 52) + significand  # 2^53 carries into the exponent
        carried = up and significand == 2**53
    assert 0 < bits < POSITIVE_INFINITY
    return bits, (position, up, carried, -unit, shifted, divisor)


def nearest_bits(numerator, denominator):
    bits, info = round_magnitude(numerator, denominator, "nearest")
    # CPython's int/int true division is correctly rounded (half-even).
    assert float_bits(numerator / denominator) == bits
    COUNTS["true-division-checks"] += 1
    return bits, info


def bracket(numerator, denominator):
    """The binary64 values just below and above a positive rational."""
    low, _ = round_magnitude(numerator, denominator, "down")
    high, _ = round_magnitude(numerator, denominator, "up")
    near, _ = nearest_bits(numerator, denominator)
    value = Fraction(numerator, denominator)
    assert near in (low, high)
    if low == high:
        assert exact_value(low) == value
    else:
        assert high == low + 1 and exact_value(low) < value < exact_value(high)
    COUNTS["bracket-checks"] += 1
    return low, high


def neighbourhood(numerator, denominator):
    """A representable value with both neighbours, or both bracketing values
    with their outer neighbours (bit increments, positive patterns)."""
    low, high = bracket(numerator, denominator)
    patterns = [low - 1, low, low + 1] if low == high else [low - 1, low, high, high + 1]
    return [bits for bits in patterns if 0 < bits < POSITIVE_INFINITY]


def exact_bits(numerator, denominator=1):
    bits, (position, _, _, _, _, _) = round_magnitude(numerator, denominator, "nearest")
    assert position == "exact"
    return bits


def verify_nearest(value, bits):
    """Definition check: no neighbour is closer; a tie has an even significand."""
    target = abs(value)
    pattern = bits & ~SIGN
    distance = abs(exact_value(pattern) - target)
    for neighbour in (pattern - 1, pattern + 1):
        if 0 <= neighbour < POSITIVE_INFINITY:
            other = abs(exact_value(neighbour) - target)
            assert distance <= other
            if distance == other:
                assert pattern & 1 == 0
    COUNTS["nearest-neighbour-distance-checks"] += 1


# ---------------------------------------------------------------------------
# Phase A quantization table, restated over reduced Fraction units.

def rounded_units(units, mode):
    negative = units < 0
    size = -units if negative else units
    count, remainder = divmod(size.numerator, size.denominator)
    whole = size.denominator
    if remainder and mode == "Exact":
        return None
    increment = bool(remainder) and (
        mode == "AwayFromZero"
        or (mode == "TowardPositiveInfinity" and not negative)
        or (mode == "TowardNegativeInfinity" and negative)
        or (mode == "NearestTiesEven" and (2 * remainder > whole or (2 * remainder == whole and count % 2)))
        or (mode == "NearestTiesAwayFromZero" and 2 * remainder >= whole)
    )
    count += increment
    return -count if negative else count


def verify_rounding(units, rounded, mode):
    if rounded is None:
        assert mode == "Exact" and units.denominator != 1
        return
    difference = Fraction(rounded) - units
    assert abs(difference) < 1
    if mode.startswith("Nearest"):
        assert abs(difference) <= Fraction(1, 2)
    elif mode == "TowardPositiveInfinity":
        assert difference >= 0
    elif mode == "TowardNegativeInfinity":
        assert difference <= 0
    elif mode == "TowardZero":
        assert abs(rounded) <= abs(units)
    elif mode == "AwayFromZero":
        assert abs(rounded) >= abs(units)
    else:
        assert difference == 0
    assert rounded_units(Fraction(rounded), mode) == rounded
    mirror = {"TowardPositiveInfinity": "TowardNegativeInfinity",
              "TowardNegativeInfinity": "TowardPositiveInfinity"}.get(mode, mode)
    assert rounded_units(-units, mirror) == -rounded
    COUNTS["rounding-inequality-idempotence-symmetry-checks"] += 1


# ---------------------------------------------------------------------------
# The four conversions.

def non_finite(operation, kind, sign):
    value = "NaN" if kind == "nan" else ("NegativeInfinity" if sign else "PositiveInfinity")
    return failure("non_finite", ("operation", operation), ("class", value))


def from_exact(bits):
    operation = OPERATION["from_exact"]
    check_decode(bits)
    kind, sign, significand, exponent = decode(bits)
    if kind != "finite":
        return non_finite(operation, kind, sign)
    if significand == 0:
        return "ok", "-", cell((0, 0))
    while significand % 2 == 0:
        significand //= 2
        exponent += 1
    if exponent >= 0:
        coefficient, scale = significand << exponent, 0
    else:
        scale = -exponent
        coefficient = significand * 5**scale
        if scale <= MAX_SCALE:
            assert coefficient < 2**142
            widen("exact-fractional-coefficient", coefficient)
    # Separately formulated: reduced-denominator factoring, then CPython's
    # exact float-to-Decimal expansion.
    value = abs(exact_value(bits))
    twos = value.denominator.bit_length() - 1
    assert value.denominator == 1 << twos
    assert (value.numerator * 5**twos, twos) == (coefficient, scale)
    digit_sign, digits, power = Decimal(as_float(bits)).as_tuple()
    assert digit_sign == sign
    expansion = int("".join(map(str, digits)))
    expansion = canonical(expansion * 10**power, 0) if power >= 0 else canonical(expansion, -power)
    assert expansion == (coefficient, scale)
    COUNTS["decimal-expansion-checks"] += 1
    if scale > MAX_SCALE:
        assert scale <= 1074
        return failure("result_scale", ("operation", operation), ("scale", scale), ("maximum", MAX_SCALE))
    signed = -coefficient if sign else coefficient
    if not MIN <= signed <= MAX:
        return failure("out_of_range", ("operation", operation), ("negative", sign))
    assert signed % 10 != 0 or scale == 0
    return "ok", "-", cell((signed, scale))


def from_rounded(bits, quantum, mode):
    operation = OPERATION["from_rounded"]
    check_decode(bits)
    kind, sign, significand, exponent = decode(bits)
    if kind != "finite":
        return non_finite(operation, kind, sign)
    if significand == 0:
        COUNTS["rounded-signed-zero"] += 1
        return "ok", "-", cell((0, 0))
    big_q, scale = quantum
    assert big_q > 0 and canonical(big_q, scale) == quantum
    if exponent >= 0:
        a, b = (significand << exponent) * 10**scale, big_q
        assert a < 2**1151 and b < 2**127
        widen("rounded-nonnegative-exponent-A", a)
        widen("rounded-nonnegative-exponent-B", b)
    else:
        a, b = significand * 10**scale, big_q << -exponent
        assert a < 2**180 and b < 2**1201
        widen("rounded-negative-exponent-A", a)
        widen("rounded-negative-exponent-B", b)
    count, remainder = divmod(a, b)
    if remainder == 0:
        COUNTS["rounded-exact-multiple"] += 1
    elif 2 * remainder < b:
        COUNTS["rounded-below-half"] += 1
    elif 2 * remainder == b:
        COUNTS["rounded-exact-half-odd-count" if count % 2 else "rounded-exact-half-even-count"] += 1
    else:
        COUNTS["rounded-above-half"] += 1
    if remainder and mode == "Exact":
        rounded = None
    else:
        increment = bool(remainder) and (
            mode == "AwayFromZero"
            or (mode == "TowardPositiveInfinity" and not sign)
            or (mode == "TowardNegativeInfinity" and sign)
            or (mode == "NearestTiesEven" and (2 * remainder > b or (2 * remainder == b and count % 2)))
            or (mode == "NearestTiesAwayFromZero" and 2 * remainder >= b)
        )
        rounded = count + increment
    signed_rounded = None if rounded is None else (-rounded if sign else rounded)

    # Cross-check 1: reduced Fraction units under the restated phase A table.
    units = exact_value(bits) / Fraction(big_q, 10**scale)
    assert rounded_units(units, mode) == signed_rounded
    verify_rounding(units, signed_rounded, mode)
    COUNTS["fraction-quotient-cross-checks"] += 1
    # Cross-check 2: exact Decimal operands, then exactly one division.
    numerator = Decimal(as_float(bits))
    for ctx in CONTEXTS:
        denominator = ctx.scaleb(Decimal(big_q), Decimal(-scale))
        quotient = ctx.divide(numerator, denominator)
        if mode == "Exact":
            integral = quotient.to_integral_value(rounding=ROUND_DOWN, context=ctx)
            decimal_rounded = int(integral) if integral == quotient else None
        else:
            integral = quotient.to_integral_value(rounding=DECIMAL_MODES[mode], context=ctx)
            decimal_rounded = int(integral)
        assert decimal_rounded == signed_rounded, (hexbits(bits), quantum, mode, ctx.prec)
        COUNTS["decimal-quotient-cross-checks"] += 1
    # Cross-check 3, power-of-ten quanta 10^(m-s) only: division-free
    # Decimal.quantize of the exact expansion (at most 347 result digits).
    power = len(str(big_q)) - 1
    if big_q == 10**power:
        ctx = CONTEXTS[-1]
        rounding = ROUND_DOWN if mode == "Exact" else DECIMAL_MODES[mode]
        quantized = numerator.quantize(Decimal(1).scaleb(power - scale), rounding=rounding, context=ctx)
        if mode == "Exact" and quantized != numerator:
            quantized_count = None
        else:
            quantized_count = int(quantized.scaleb(scale - power, context=ctx))
        assert quantized_count == signed_rounded, (hexbits(bits), quantum, mode)
        COUNTS["decimal-quantize-cross-checks"] += 1

    if rounded is None:
        COUNTS["rounded-exact-inexact"] += 1
        truncated = canonical(-count * big_q if sign else count * big_q, scale)[0]
        if not MIN <= truncated <= MAX:
            COUNTS["rounded-inexact-before-range"] += 1
        return failure("inexact", ("operation", operation))
    product = rounded * big_q
    assert product <= a + big_q < 2**1152
    widen("rounded-count-times-quantum", product)
    if count == 0 and rounded == 0:
        COUNTS["rounded-nonzero-to-zero"] += 1
    if count == 0 and rounded == 1:
        COUNTS["rounded-tiny-to-one-quantum"] += 1
    signed = -product if sign else product
    coefficient, result_scale = canonical(signed, scale)
    if not MIN <= coefficient <= MAX:
        COUNTS["rounded-out-of-range-after-rounding"] += 1
        return failure("out_of_range", ("operation", operation), ("negative", sign))
    if not MIN <= signed <= MAX:
        COUNTS["rounded-normalization-rescue"] += 1
    ROUNDED.setdefault((quantum, mode), []).append(
        (exact_value(bits), Fraction(coefficient, 10**result_scale)))
    return "ok", "-", cell((coefficient, result_scale))


def to_nearest(pair, census=True):
    coefficient, scale = pair
    assert canonical(coefficient, scale) == pair and MIN <= coefficient <= MAX and scale <= MAX_SCALE
    if coefficient == 0:
        return 0
    size = abs(coefficient)
    bits, (position, up, carried, shift, shifted, divisor) = nearest_bits(size, 10**scale)
    assert 1 <= bits >> 52 <= 2046          # always normal and finite
    assert shift <= 180 and divisor < 2**201
    WIDTHS["outgoing-numerator-shift"] = max(WIDTHS["outgoing-numerator-shift"], shift)
    widen("outgoing-shifted-numerator", shifted)
    widen("outgoing-shifted-denominator", divisor)
    if census:
        if position == "half":
            position = "half-rounded-up" if up else "half-rounded-down"
        COUNTS["nearest-" + position] += 1
        if carried:
            COUNTS["nearest-carry-into-next-binade"] += 1
            if position.startswith("half"):
                COUNTS["nearest-tie-carry-into-next-binade"] += 1
    if coefficient < 0:
        bits |= SIGN
    # Independent decimal-string conversion (CPython's correctly rounded strtod).
    assert float_bits(float("%de-%d" % (coefficient, scale))) == bits
    COUNTS["decimal-string-parse-checks"] += 1
    verify_nearest(Fraction(coefficient, 10**scale), bits)
    return bits


def to_exact(pair):
    bits = to_nearest(pair, census=False)
    value = Fraction(pair[0], 10**pair[1])
    exact = exact_value(bits) == value
    # Representability criterion: power-of-two reduced denominator and an odd
    # numerator part of at most 53 bits (the exponent is always normal here).
    numerator, denominator = abs(value.numerator), value.denominator
    odd = numerator >> ((numerator & -numerator).bit_length() - 1) if numerator else 0
    assert exact == (denominator & (denominator - 1) == 0 and odd.bit_length() <= 53)
    COUNTS["representability-criterion-checks"] += 1
    if not exact:
        return failure("inexact", ("operation", OPERATION["to_exact"]))
    return "ok", "-", hexbits(bits)


def expected(op, bits=None, amount=None, quantum=None, mode=None):
    if op == "from_exact":
        return from_exact(bits)
    if op == "from_rounded":
        return from_rounded(bits, quantum, mode)
    if op == "to_exact":
        return to_exact(amount)
    assert op == "to_nearest"
    bits = to_nearest(amount)
    OUTGOING.append((Fraction(amount[0], 10**amount[1]), bits))
    return "ok", "-", hexbits(bits)


# ---------------------------------------------------------------------------
# Corpus.

POWER_QUANTA = tuple((1, s) for s in range(39)) + tuple((10**m, 0) for m in range(1, 39))
SPECIAL_QUANTA = (
    (5, 2), (25, 2), (5, 1), (25, 3), (125, 3), (5, 3), (2, 0), (3, 0), (7, 0), (3, 1),
    (7, 3), (11, 2), (13, 5), (999, 3), (9, 38), (2, 38), (5**38, 38), (5**20, 20),
    (2**126, 38), (6361, 0), (2**53 - 1, 0), (2**64 + 1, 20), (123456789, 9),
    (10**37 + 1, 37), (MAX, 0), (MAX, 19), (MAX, 38),
)
ALL_QUANTA = POWER_QUANTA + SPECIAL_QUANTA
EDGE_CYCLE = (
    (MAX, 38), (1, 38), (1, 2), (1, 0), (5, 2), (MAX, 0), (7, 3), (5**38, 38), (10**38, 0),
    (3, 0), (2**64 + 1, 20), (9, 38), (1, 19), (25, 3), (123456789, 9), (10**37 + 1, 37), (2, 0),
)
NONFINITE_QUANTA = ((1, 0), (1, 38), (5, 2), (MAX, 0))
SUBNORMAL_QUANTA = ((1, 38), (1, 0), (MAX, 38))
BOUNDARY_QUANTA = ((1, 0), (1, 38), (2, 0), (MAX, 0))
HALF_QUANTA = (
    (1, 2), (5, 2), (25, 2), (1, 1), (5, 1), (1, 0), (2, 0), (3, 0), (7, 0),
    (1, 3), (5, 3), (25, 3), (125, 3), (7, 3), (999, 3), (11, 2), (3, 1), (13, 5),
    (1, 4), (1, 6), (1, 10), (1, 15), (1, 16), (1, 17), (1, 20), (1, 30), (1, 36), (1, 37), (1, 38),
    (2, 38), (9, 38), (5**38, 38), (5**20, 20), (123456789, 9), (2**64 + 1, 20), (10**37 + 1, 37),
    (10, 0), (100, 0), (10**3, 0), (10**9, 0), (10**15, 0), (10**16, 0), (10**22, 0), (10**23, 0),
    (10**30, 0), (10**37, 0), (10**38, 0), (6361, 0), (2**53 - 1, 0), (MAX, 0), (MAX, 19), (MAX, 38),
)
TINY_QUANTA = (
    (1, 38), (2, 38), (9, 38), (5**38, 38), (1, 37), (1, 30), (1, 20), (1, 10), (1, 2), (5, 2),
    (1, 1), (1, 0), (7, 3), (3, 0), (10**38, 0), (2**64 + 1, 20), (123456789, 9), (MAX, 0), (MAX, 38),
)
HUGE_QUANTA = (
    (1, 0), (2, 0), (3, 0), (7, 0), (6361, 0), (2**53 - 1, 0), (10**38, 0), (MAX, 0),
    (1, 2), (5, 2), (7, 3), (1, 38), (9, 38), (5**38, 38), (2**64 + 1, 20), (MAX, 38),
)
DISCRIMINATOR_QUANTA = ((1, 2), (1, 1), (5, 2), (1, 3), (1, 0), (1, 17), (1, 38))
DISCRIMINATOR_TEXT = (
    "0.1", "2.675", "1.005", "1.015", "1.025", "1.045", "1.255", "2.665", "2.685",
    "0.125", "0.375", "0.05", "0.15", "0.3", "0.7", "0.5", "1.5", "2.5",
)
OUTGOING_DISCRIMINATORS = (
    (1, 1), (2675, 3), (1005, 3), (1015, 3), (1025, 3), (1255, 3), (2665, 3), (125, 3),
    (375, 3), (5, 2), (15, 2), (3, 1), (7, 1), (5, 1), (15, 1), (25, 1),
)
EXACT_SCALES = tuple(range(46)) + (52, 53, 54, 55, 63, 64, 65, 100, 126, 127, 128, 500,
                                   1021, 1022, 1023, 1024, 1073, 1074)
TRANSITIONS = (
    2**52 - 1, 2**52, 2**52 + 1, 2**53 - 3, 2**53 - 2, 2**53 - 1, 2**53, 2**53 + 1,
    2**53 + 2, 2**53 + 3, 2**53 + 4, 2**53 + 5, 2**54 - 5, 2**54 - 4, 2**54 - 3, 2**54 - 2,
    2**54 - 1, 2**54, 2**54 + 1, 2**54 + 2, 2**54 + 3, 2**54 + 4, 2**54 + 6, 2**55 - 3,
    2**55 - 2, 2**55 - 1,
)
for _quantum in ALL_QUANTA + EDGE_CYCLE + HALF_QUANTA + TINY_QUANTA + HUGE_QUANTA:
    assert 0 < _quantum[0] <= MAX and _quantum[1] <= MAX_SCALE and canonical(*_quantum) == _quantum


class SplitMix64:
    """Deterministic corpus selection only; never an expectation source."""

    def __init__(self, seed):
        self.state = seed

    def word(self):
        self.state = (self.state + 0x9E3779B97F4A7C15) & (2**64 - 1)
        value = self.state
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & (2**64 - 1)
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & (2**64 - 1)
        return value ^ (value >> 31)

    def bits(self, count):
        value = 0
        for _ in range((count + 63) // 64):
            value = (value << 64) | self.word()
        return value & ((1 << count) - 1)

    def below(self, bound):
        assert bound > 0
        return self.bits(bound.bit_length() + 64) % bound


def unique(values):
    seen = set()
    for value in values:
        if value not in seen:
            seen.add(value)
            yield value


def signed_patterns(patterns):
    for sign in (0, SIGN):
        for bits in patterns:
            yield bits | sign


def nonfinite_patterns():
    payloads = [1 << position for position in range(52)] + [
        2**52 - 1, 2**51 - 1, 2**51 + 1, 0x5555555555555, 0xAAAAAAAAAAAAA,
        0x00DEADBEEFCAF, 0x8DEADBEEFCAFE,
    ]
    for sign in (0, SIGN):
        yield sign | POSITIVE_INFINITY
        for payload in unique(payloads):
            assert 0 < payload <= FRACTION_MASK
            yield sign | POSITIVE_INFINITY | payload


def odd_fit(limit):
    """Largest odd significand below 2^53 whose value stays at most limit."""
    value = min(2**53 - 1, limit)
    return value if value % 2 else value - 1


def fitted_quantum(bits):
    kind, _, significand, exponent = decode(bits)
    if kind != "finite" or significand == 0:
        return 1, 0
    integer = significand << exponent if exponent >= 0 else significand >> -exponent
    digits = len(str(integer)) if integer else 0
    return 1, max(0, MAX_SCALE - digits)


def half_quantum_counts(quantum, rng):
    big_q, scale = quantum
    largest = MAX // big_q
    five = 5**scale // math.gcd(big_q, 5**scale)
    counts = {0, 1, 2, 3, (five - 1) // 2, (3 * five - 1) // 2, 2**52 - 1, 2**52, 2**53 + 1,
              largest - 1, largest}
    counts.add(4 + rng.below(max(1, min(largest, 2**60) - 4)))
    return sorted(k for k in counts if 0 <= k <= largest)


def amount_from_dyadic(odd, shift):
    """Canonical decimal pair of odd*2^shift."""
    if shift >= 0:
        return odd << shift, 0
    return canonical(odd * 5**-shift, -shift)


def random_amount(rng):
    length = 1 + rng.below(128)
    size = (1 << (length - 1)) | rng.bits(length - 1)
    negative = rng.below(2)
    if length == 128:
        size, negative = 2**127, 1
    return canonical(-size if negative else size, rng.below(39))


def random_dyadic(rng):
    while True:
        length = 1 + rng.below(53)
        odd = (1 << (length - 1)) | rng.bits(length - 1) | 1
        if rng.below(2):
            coefficient, scale = amount_from_dyadic(odd, -rng.below(39))
        else:
            coefficient, scale = amount_from_dyadic(odd, rng.below(128 - length))
        if coefficient <= MAX:
            return (-coefficient if rng.below(2) else coefficient), scale


def cases():
    rng = SplitMix64(0x4D385F42494E3634)

    def case(profile, op, **arguments):
        return profile, op, arguments

    def incoming(profile, bits, quanta, modes=MODES):
        yield case(profile, "from_exact", bits=bits)
        for quantum in quanta:
            for mode in modes:
                yield case(profile, "from_rounded", bits=bits, quantum=quantum, mode=mode)

    def outgoing(profile, pair):
        yield case(profile, "to_nearest", amount=pair)
        yield case(profile, "to_exact", amount=pair)

    # Both zeros under every quantum and mode; nonfinite classes fail first.
    for bits in (0, SIGN):
        yield from incoming("IN-ZERO", bits, ALL_QUANTA)
    for bits in nonfinite_patterns():
        yield from incoming("IN-NONFINITE", bits, NONFINITE_QUANTA)

    # Every exponent bin from subnormal (field 0) through e=971 (field 2046):
    # minimum/maximum fraction and both bit-increment neighbours, plus +/-inf
    # above f64::MAX. Rounded rows rotate modes and quanta deterministically.
    edges = []
    for sign in (0, SIGN):
        for field in range(2047):
            for fraction in (0, 1, FRACTION_MASK - 1, FRACTION_MASK):
                edges.append(sign | (field << 52) | fraction)
        edges.append(sign | POSITIVE_INFINITY)
    for index, bits in enumerate(edges):
        yield case("IN-BIN-EDGE", "from_exact", bits=bits)
        yield case("IN-BIN-EDGE", "from_rounded", bits=bits,
                   quantum=EDGE_CYCLE[index % len(EDGE_CYCLE)], mode=MODES[index % 7])
        yield case("IN-BIN-EDGE", "from_rounded", bits=bits,
                   quantum=fitted_quantum(bits), mode=MODES[(index + 3) % 7])

    # Every subnormal leading-bit position with in-position neighbours.
    subnormals = []
    for position in range(52):
        lead = 1 << position
        subnormals.extend((lead - 1, lead, lead + 1, 2 * lead - 2, 2 * lead - 1))
    subnormals = [bits for bits in unique(subnormals) if 0 < bits <= FRACTION_MASK]
    for bits in signed_patterns(subnormals):
        yield from incoming("IN-SUBNORMAL-LEAD", bits, SUBNORMAL_QUANTA)

    for bits in signed_patterns((1, FRACTION_MASK, 2**52, F64_MAX)):
        yield from incoming("IN-EXTREMES", bits, ALL_QUANTA)

    # Exact scale boundaries: odd M * 2^-t, including 37/38/39 and the
    # largest odd significands whose coefficient M*5^t fits.
    for scale in EXACT_SCALES:
        limit = MAX // 5**scale
        candidates = [1, 3, 2**52 + 1, 2**53 - 3, 2**53 - 1]
        if limit:
            fit = odd_fit(limit)
            candidates += [fit - 2, fit, fit + 2]
        values = [m for m in unique(candidates) if 1 <= m < 2**53 and m % 2]
        for bits in signed_patterns([exact_bits(m, 2**scale) for m in sorted(values)]):
            yield from incoming("IN-EXACT-SCALE", bits, ((1, 38),))

    # Signed i128 coefficient boundaries for integers and scales 32..38.
    boundary = []
    for shift in range(74, 128):
        room = 127 - shift
        for odd in (2**room - 1, 2**room + 1):
            if 1 <= odd < 2**53 and odd % 2:
                boundary.append(exact_bits(odd << shift))
    for centre in (2**126, 2**127, 2**128):
        middle = exact_bits(centre)
        boundary.extend(middle + delta for delta in (-3, -2, -1, 0, 1, 2, 3))
    for scale in range(32, 39):
        fit = odd_fit(MAX // 5**scale)
        boundary.extend(exact_bits(odd, 2**scale) for odd in (fit - 2, fit, fit + 2))
    for bits in signed_patterns(list(unique(boundary))):
        yield from incoming("IN-I128-BOUNDARY", bits, BOUNDARY_QUANTA)

    # Half-quantum points (k + 1/2) q and their binary64 neighbourhoods.
    for quantum in HALF_QUANTA:
        big_q, scale = quantum
        for count in half_quantum_counts(quantum, rng):
            patterns = neighbourhood((2 * count + 1) * big_q, 2 * 10**scale)
            for bits in signed_patterns(patterns):
                for mode in MODES:
                    yield case("IN-HALF-QUANTUM", "from_rounded", bits=bits, quantum=quantum, mode=mode)

    # Tiny inputs: subnormal extremes, half a quantum and one quantum.
    for quantum in TINY_QUANTA:
        big_q, scale = quantum
        patterns = [1, FRACTION_MASK, 2**52]
        patterns += neighbourhood(big_q, 2 * 10**scale) + neighbourhood(big_q, 10**scale)
        for bits in signed_patterns(list(unique(patterns))):
            for mode in MODES:
                yield case("IN-TINY", "from_rounded", bits=bits, quantum=quantum, mode=mode)

    # Huge inputs: range is classified after rounding; Exact checks the
    # remainder before range.
    huge = [F64_MAX, F64_MAX - 1] + [exact_bits(2**power) for power in
                                     (1023, 1000, 971, 970, 600, 256, 200, 192, 129, 128)]
    huge += list(bracket(10**38, 1)) + list(bracket(10**308, 1)) + list(bracket(3 * MAX, 2))
    for bits in signed_patterns(list(unique(huge))):
        yield from incoming("IN-HUGE", bits, HUGE_QUANTA)

    # Discriminators: binary64 0.1 needs scale 55; binary64 2.675 lies below
    # the decimal cent tie. Nearest binary64 of short decimal texts.
    discriminators = []
    for text in DISCRIMINATOR_TEXT:
        whole, fraction = text.split(".")
        bits, _ = nearest_bits(int(whole + fraction), 10**len(fraction))
        assert float_bits(float(text)) == bits
        discriminators.append(bits)
        if text in ("0.1", "2.675"):
            discriminators.extend(bits + delta for delta in (-2, -1, 1, 2))
    for bits in signed_patterns(list(unique(discriminators))):
        yield from incoming("IN-DISCRIMINATOR", bits, DISCRIMINATOR_QUANTA)

    # Outgoing.
    yield from outgoing("OUT-DISCRIMINATOR", (0, 0))
    for pair in OUTGOING_DISCRIMINATORS:
        yield from outgoing("OUT-DISCRIMINATOR", pair)
        yield from outgoing("OUT-DISCRIMINATOR", (-pair[0], pair[1]))

    for scale in range(39):
        coefficients = [MIN, MIN + 1, MIN + 2, MAX - 2, MAX - 1, MAX, 1, -1, 2, -2, 3, -3,
                        5**scale, -5**scale, 3 * 5**scale, -3 * 5**scale,
                        10**scale - 1, -(10**scale - 1), 10**scale + 1, -(10**scale + 1)]
        for power in range(128):
            for delta in (-1, 0, 1):
                size = 2**power + delta
                first = 1 if (power + delta) % 2 == 0 else -1
                for sign in (first, -first):
                    if MIN <= sign * size <= MAX:
                        coefficients.append(sign * size)
                        break
        for coefficient in unique(coefficients):
            if coefficient and MIN <= coefficient <= MAX and canonical(coefficient, scale) == (coefficient, scale):
                yield from outgoing("OUT-SCALE-COEFFICIENT", (coefficient, scale))

    for odd in TRANSITIONS:
        for shift in range(-38, 74):
            coefficient, scale = amount_from_dyadic(odd, shift)
            for signed in (coefficient, -coefficient):
                if MIN <= signed <= MAX:
                    yield from outgoing("OUT-SIGNIFICAND", (signed, scale))

    # Midpoints (2m+1)*2^(E-53) of every binade from 2^-126 to 2^126: the exact
    # tie and its coefficient neighbours when decimal, else the decimal values
    # bracketing it at the finest scale that fits.
    for exponent in range(-126, 127):
        for significand in (2**52, 2**52 + 1, 2**52 + rng.below(2**52), 2**53 - 2, 2**53 - 1):
            numerator = (2 * significand + 1) << max(exponent - 53, 0)
            denominator = 1 << max(53 - exponent, 0)
            scale = MAX_SCALE
            while -(-numerator * 10**scale // denominator) > MAX:
                scale -= 1
            assert scale >= 0
            low, remainder = divmod(numerator * 10**scale, denominator)
            candidates = (low - 1, low, low + 1) if remainder == 0 else (low, low + 1)
            pairs = unique(canonical(c, scale) for c in candidates if 1 <= c <= MAX)
            for coefficient, pair_scale in pairs:
                yield from outgoing("OUT-MIDPOINT", (coefficient, pair_scale))
                yield from outgoing("OUT-MIDPOINT", (-coefficient, pair_scale))

    for _ in range(6000):
        yield from outgoing("OUT-RANDOM", random_amount(rng))
    for _ in range(2000):
        yield from outgoing("OUT-RANDOM-DYADIC", random_dyadic(rng))


def monotonicity():
    """Exact-order checks over the generated expectations themselves."""
    ordered = sorted(OUTGOING, key=lambda item: item[0])
    for (_, left_bits), (_, right_bits) in zip(ordered, ordered[1:]):
        assert exact_value(left_bits) <= exact_value(right_bits)
        COUNTS["ordered-monotonicity-checks"] += 1
    for key in sorted(ROUNDED, key=lambda item: (item[0], MODES.index(item[1]))):
        ordered = sorted(ROUNDED[key], key=lambda item: item[0])
        for (left, left_result), (right, right_result) in zip(ordered, ordered[1:]):
            assert left_result <= right_result
            COUNTS["ordered-monotonicity-checks"] += 1


def generate(generated_at, platform_text):
    assert sha(CONTRACT.read_bytes()) == CONTRACT_SHA, "frozen phase B contract changed"
    assert sha(AMOUNT_CONTRACT.read_bytes()) == AMOUNT_CONTRACT_SHA, "frozen phase A contract changed"
    for counter in (COUNTS, PROFILES, OPS, EXPECTATIONS, ERROR_OPERATIONS, WIDTHS):
        counter.clear()
    OUTGOING.clear()
    ROUNDED.clear()
    records = []
    distinct = set()

    def emit(profile, op, arguments):
        assert set(arguments) == set(OP_FIELDS[op])
        outcome, context, output = expected(op, **arguments)
        bits = arguments.get("bits")
        row = (
            "BIN-%06d" % (len(records) + 1), profile, op,
            "-" if bits is None else hexbits(bits), cell(arguments.get("amount")),
            cell(arguments.get("quantum")), arguments.get("mode", "-"), outcome, context, output,
        )
        assert len(row) == len(COLUMNS) and all(row) and all("\t" not in item and "\n" not in item for item in row)
        records.append("\t".join(row))
        distinct.add(row[2:7])
        PROFILES[profile] += 1
        OPS[op] += 1
        EXPECTATIONS[outcome] += 1
        if outcome.startswith("amount."):
            ERROR_OPERATIONS[context.split(",")[0].split("=", 1)[1]] += 1
        return outcome, output

    for profile, op, arguments in cases():
        outcome, output = emit(profile, op, arguments)
        # Round trips in both exact directions, emitted beside their source.
        if op == "from_exact" and outcome == "ok":
            coefficient, scale = map(int, output.split(":"))
            back = 0 if coefficient == 0 else arguments["bits"]
            assert emit("ROUNDTRIP-INCOMING", "to_exact", {"amount": (coefficient, scale)}) == ("ok", hexbits(back))
            COUNTS["round-trip-checks"] += 1
        if op == "to_exact" and outcome == "ok":
            bits = int(output, 16)
            assert emit("ROUNDTRIP-OUTGOING", "from_exact", {"bits": bits}) == ("ok", cell(arguments["amount"]))
            COUNTS["round-trip-checks"] += 1
    monotonicity()
    assert WIDTHS["rounded-nonnegative-exponent-A"] == 1151
    assert WIDTHS["rounded-negative-exponent-B"] == 1201
    assert set(EXPECTATIONS) == {
        "ok", "amount.non_finite", "amount.result_scale", "amount.out_of_range", "amount.inexact",
    }
    assert set(ERROR_OPERATIONS) == set(OPERATION.values())
    # TESTING_STRATEGY §7.3: one column line precedes the records.
    body = ("\t".join(COLUMNS) + "\n" + "\n".join(records) + "\n").encode("utf-8")
    own = Path(__file__).read_bytes()
    contract = CONTRACT.read_bytes()
    amount_contract = AMOUNT_CONTRACT.read_bytes()
    requirements = "\n".join(line for line in REQUIREMENTS.read_text(encoding="utf-8").splitlines()
                             if re.match(r"\| FR-BIN-\d+ \|", line)).encode("utf-8")
    assert len(requirements.decode().splitlines()) == 8

    def census(counter):
        return ";".join("%s=%d" % item for item in sorted(counter.items()))

    headers = [
        "schema: 1", "fixture-set: EXACT-DECIMAL-AMOUNT-PHASE-B-BINARY64-ORACLE", "domain: amounts",
        "description: Independent binary64 bit-field, integer and Fraction authority for the four named exact-amount conversions",
        "reference: oracle:struct-bit-fields+integer-quotient-remainder+exact-comparison-rounding; no Rust or QuantLib output",
        "reference-version: Python >=3.9 stdlib struct/fractions/decimal; Decimal cross-check precisions768,1024",
        "reference-source: Frozen docs/M8_BINARY_BOUNDARY_CONTRACT.md over docs/M8_AMOUNT_CONTRACT.md; float objects appear only in assertion cross-checks",
        "reproduction: python3 -I validation/generator/binary_boundary_oracles.py --check",
        "generated-at: " + generated_at, "platform: " + platform_text,
        "generator-sha256: " + sha(own), "generator-fnv64: " + fnv(own),
        "adopted-contract-sha256: " + sha(contract), "adopted-contract-fnv64: " + fnv(contract),
        "amount-contract-sha256: " + sha(amount_contract), "amount-contract-fnv64: " + fnv(amount_contract),
        "requirements-rows-sha256: " + sha(requirements),
        "tolerance: TOL-E0; exact canonical amounts, binary64 bit patterns and structured errors; no epsilon",
        "encoding: tab-separated; bits=0x+16 lowercase hex IEEE754 binary64 pattern (sign and NaN payload kept); amount/quantum=canonical coefficient:scale; mode=canonical AmountRounding identifier; -=unused; context=comma-separated key=value",
        "operation-map: from_exact=DecimalAmount::from_binary64_exact(f64::from_bits(bits)); from_rounded=DecimalAmount::from_binary64_rounded(f64::from_bits(bits),AmountQuantum::new(quantum),mode); to_exact=DecimalAmount::new(amount).to_binary64_exact(); to_nearest=DecimalAmount::new(amount).to_binary64_nearest()",
        "expected-encoding: ok output=canonical coefficient:scale (from_exact/from_rounded) or 0x bits (to_exact/to_nearest) and context=-; errors output=- and context=all structured fields; non_finite context class=NaN|PositiveInfinity|NegativeInfinity is the variant's NonFinite value",
        "source-scope: Frozen local phaseB contract; no shortest-decimal reading, phaseC settlement adapters or bond certification",
        "record-count: " + str(len(records)), "unique-input-count: " + str(len(distinct)),
        "profile-counts: " + census(PROFILES), "operation-counts: " + census(OPS),
        "expectation-counts: " + census(EXPECTATIONS), "error-operation-counts: " + census(ERROR_OPERATIONS),
        "cross-check-counts: " + census(COUNTS), "maximum-width-bits: " + census(WIDTHS),
        "body-sha256: " + sha(body), "body-fnv64: " + fnv(body),
    ]
    data = ("\n".join(headers) + "\n\n").encode("utf-8") + body
    summary = {
        "records": len(records), "unique_inputs": len(distinct), "bytes": len(data),
        "sha256": sha(data), "body_sha256": sha(body), "body_fnv64": fnv(body),
        "generator_sha256": sha(own), "contract_sha256": sha(contract),
        "amount_contract_sha256": sha(amount_contract), "requirements_rows_sha256": sha(requirements),
        "profiles": dict(sorted(PROFILES.items())), "operations": dict(sorted(OPS.items())),
        "expectations": dict(sorted(EXPECTATIONS.items())),
        "error_operations": dict(sorted(ERROR_OPERATIONS.items())),
        "cross_checks": dict(sorted(COUNTS.items())), "width_bits": dict(sorted(WIDTHS.items())),
    }
    return data, summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Recompute and compare the existing fixture byte-for-byte")
    parser.add_argument("--output", type=Path, default=DEST)
    args = parser.parse_args()
    generated_at = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    platform_text = platform.system() + " " + platform.machine()
    if args.check and args.output.is_file():
        # Like the other generators, --check reuses the recorded run metadata.
        header = args.output.read_text(encoding="utf-8").split("\n\n", 1)[0]
        recorded = dict(line.split(": ", 1) for line in header.splitlines() if ": " in line)
        generated_at = recorded.get("generated-at", generated_at)
        platform_text = recorded.get("platform", platform_text)
    data, summary = generate(generated_at, platform_text)
    if args.check:
        if not args.output.is_file() or args.output.read_bytes() != data:
            raise SystemExit("binary boundary fixture differs from independent regeneration")
        summary["check"] = "identical"
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(data)
        output = args.output.resolve()
        summary["written"] = str(output.relative_to(ROOT)) if output.is_relative_to(ROOT) else str(output)
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
