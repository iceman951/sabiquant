#!/usr/bin/env python3
"""Independent phase-A decimal oracle; never reads Rust or reference outputs.

Fraction and Python integers are authoritative. Decimal checks separately form
the exact input expression and perform a single quotient rounding, at both
384 and 512 significant digits. See docs/M8_AMOUNT_ORACLES.md for the precision
proof, corpus scope and field/error encoding. No non-stdlib imports are used.
"""

import argparse
import datetime as dt
import platform
from collections import Counter
from decimal import (
    Context, Decimal, ROUND_CEILING, ROUND_DOWN, ROUND_FLOOR, ROUND_HALF_EVEN,
    ROUND_HALF_UP, ROUND_UP,
)
from fractions import Fraction
import hashlib
import itertools
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/amounts/oracles.fixture"
CONTRACT = ROOT / "docs/M8_AMOUNT_CONTRACT.md"
CONTRACT_SHA = "954571eb64be97e71c68877c8e5c705f5ca9312a2109d6b54f6a1883aead7df1"
REQUIREMENTS = ROOT / "docs/REQUIREMENTS.md"
MIN, MAX, UMAX = -(2**127), 2**127 - 1, 2**128 - 1
MODES = (
    "TowardZero", "AwayFromZero", "TowardPositiveInfinity",
    "TowardNegativeInfinity", "NearestTiesEven", "NearestTiesAwayFromZero", "Exact",
)
DECIMAL_MODES = dict(zip(MODES[:-1], (
    ROUND_DOWN, ROUND_UP, ROUND_CEILING, ROUND_FLOOR, ROUND_HALF_EVEN, ROUND_HALF_UP,
)))
CONTEXTS = (Context(prec=384), Context(prec=512))
COLUMNS = (
    "id profile op lhs rhs numerator denominator quantum mode texthex terms "
    "expected context output"
).split()
GRAMMAR = re.compile(rb"[+-]?[0-9]+(?:\.[0-9]+)?\Z")
OP_FIELDS = {
    "new": ("lhs",), "quantum": ("lhs",), "parse": ("text",), "mode": ("text",),
    "add": ("lhs", "rhs"), "sub": ("lhs", "rhs"), "mul": ("lhs", "rhs"),
    "cmp": ("lhs", "rhs"), "neg": ("lhs",), "sum": ("terms",),
    "multiple": ("lhs", "quantum"), "quantize": ("lhs", "quantum", "mode"),
    "ratio": ("numerator", "denominator", "quantum", "mode"),
    "multiply_ratio": ("lhs", "numerator", "denominator", "quantum", "mode"),
    "multiply_divide": ("lhs", "rhs", "denominator", "quantum", "mode"),
}
COUNTS = Counter()
PROFILES = Counter()
OPS = Counter()
EXPECTATIONS = Counter()
ERROR_OPERATIONS = Counter()
WIDTHS = Counter()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    value = 14695981039346656037
    for byte in data:
        value = ((value ^ byte) * 1099511628211) & (2**64 - 1)
    return "%016x" % value


def hx(text):
    encoded = text.encode("utf-8").hex()
    return encoded if encoded else "_"


def cell(value):
    return "-" if value is None else "%d:%d" % value


def rational(value):
    return Fraction(value[0], 10**value[1])


def decimal_pair(value):
    """Minimal exact finite decimal, derived from reduced denominator factors."""
    if not value:
        return 0, 0
    denominator = value.denominator
    twos = fives = 0
    while denominator % 2 == 0:
        twos += 1
        denominator //= 2
    while denominator % 5 == 0:
        fives += 1
        denominator //= 5
    assert denominator == 1
    scale = max(twos, fives)
    return value.numerator * 2**(scale - twos) * 5**(scale - fives), scale


def display(pair):
    coefficient, scale = pair
    digits = str(abs(coefficient)).rjust(scale + 1, "0")
    text = (digits[:-scale] + "." + digits[-scale:]) if scale else digits
    return ("-" if coefficient < 0 else "") + text


def failure(kind, **context):
    return "amount." + kind, ",".join("%s=%s" % item for item in context.items()), "-"


def admit(value, operation):
    coefficient, scale = decimal_pair(value)
    if scale > 38:
        return failure("result_scale", operation=operation, scale=scale, maximum=38)
    if not MIN <= coefficient <= MAX:
        return failure("out_of_range", operation=operation, negative=int(coefficient < 0))
    canonical = coefficient, scale
    text = display(canonical)
    for ctx in CONTEXTS:
        number = ctx.scaleb(Decimal(coefficient), Decimal(-scale))
        assert format(number, "f") == text
    return "ok", "text=" + hx(text), cell(canonical)


def rounded_units(units, mode):
    """Integer/Fraction authority; signed modes never use floating arithmetic."""
    negative = units < 0
    magnitude = abs(units)
    k, r = divmod(magnitude.numerator, magnitude.denominator)
    d = magnitude.denominator
    if r and mode == "Exact":
        return None
    increment = bool(r) and (
        mode == "AwayFromZero"
        or (mode == "TowardPositiveInfinity" and not negative)
        or (mode == "TowardNegativeInfinity" and negative)
        or (mode == "NearestTiesEven" and (2*r > d or (2*r == d and k % 2)))
        or (mode == "NearestTiesAwayFromZero" and 2*r >= d)
    )
    result = k + int(increment)
    return -result if negative else result


def verify_rounding(units, rounded, mode):
    if rounded is None:
        assert mode == "Exact" and units.denominator != 1
        COUNTS["exact-inexact-checks"] += 1
        return
    difference = Fraction(rounded) - units
    assert abs(difference) < 1 or difference == 0
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
        assert units.denominator == 1 and difference == 0
    assert rounded_units(Fraction(rounded), mode) == rounded
    if mode in ("TowardZero", "AwayFromZero", "NearestTiesEven", "NearestTiesAwayFromZero", "Exact"):
        assert rounded_units(-units, mode) == -rounded
    if mode == "TowardPositiveInfinity":
        assert rounded_units(-units, "TowardNegativeInfinity") == -rounded
    COUNTS["rounding-inequality-idempotence-symmetry-checks"] += 1
    k, r = divmod(abs(units).numerator, units.denominator)
    if r and 2*r == units.denominator:
        COUNTS["exact-half-quantum-inputs"] += 1
        COUNTS["half-quantum-odd" if k % 2 else "half-quantum-even"] += 1
    if units and rounded == 0:
        COUNTS["nonzero-rounded-zero"] += 1


def dec_input(value, ctx):
    return ctx.scaleb(Decimal(value[0]), Decimal(-value[1]))


def decimal_expression(op, lhs, rhs, numerator, denominator, quantum, ctx):
    """Independently form exact Decimal operands before its one division.

    All products/powers have at most154 digits, so these operations are exact
    at either precision. Crucially, no repeated division approaches a tie
    from the wrong side: form the entire numerator and denominator first.
    """
    if op == "quantize":
        a = ctx.scaleb(Decimal(lhs[0]), Decimal(quantum[1]))
        b = ctx.multiply(Decimal(quantum[0]), ctx.power(Decimal(10), Decimal(lhs[1])))
    elif op == "ratio":
        a = ctx.scaleb(Decimal(numerator), Decimal(quantum[1]))
        b = ctx.multiply(Decimal(denominator), Decimal(quantum[0]))
    elif op == "multiply_ratio":
        a = ctx.scaleb(ctx.multiply(Decimal(lhs[0]), Decimal(numerator)), Decimal(quantum[1]))
        b = ctx.multiply(ctx.multiply(Decimal(denominator), Decimal(quantum[0])),
                         ctx.power(Decimal(10), Decimal(lhs[1])))
    else:
        assert op == "multiply_divide"
        a = ctx.scaleb(ctx.multiply(Decimal(lhs[0]), Decimal(rhs[0])), Decimal(quantum[1]))
        b = ctx.multiply(ctx.multiply(Decimal(denominator), Decimal(quantum[0])),
                         ctx.power(Decimal(10), Decimal(lhs[1] + rhs[1])))
    return ctx.divide(a, b)


def quantization(op, lhs, rhs, numerator, denominator, quantum, mode):
    operation = "ratio" if op == "ratio" else op
    if op != "quantize" and denominator == 0:
        return failure("zero_denominator", operation=operation)
    if op == "quantize":
        value = rational(lhs)
        a, b = abs(lhs[0])*10**quantum[1], quantum[0]*10**lhs[1]
    elif op == "ratio":
        value = Fraction(numerator, denominator)
        a, b = abs(numerator)*10**quantum[1], denominator*quantum[0]
    elif op == "multiply_ratio":
        value = rational(lhs)*Fraction(numerator, denominator)
        a = abs(lhs[0])*abs(numerator)*10**quantum[1]
        b = denominator*quantum[0]*10**lhs[1]
    else:
        value = rational(lhs)*rational(rhs)/denominator
        a = abs(lhs[0])*abs(rhs[0])*10**quantum[1]
        b = denominator*quantum[0]*10**(lhs[1] + rhs[1])
    WIDTHS[op + "-numerator"] = max(WIDTHS[op + "-numerator"], a.bit_length())
    WIDTHS[op + "-denominator"] = max(WIDTHS[op + "-denominator"], b.bit_length())
    units = value/rational(quantum)
    k = rounded_units(units, mode)
    verify_rounding(units, k, mode)
    for ctx in CONTEXTS:
        decimal_units = decimal_expression(op, lhs, rhs, numerator, denominator, quantum, ctx)
        integral = decimal_units.to_integral_value(rounding=DECIMAL_MODES.get(mode, ROUND_DOWN), context=ctx)
        decimal_k = None if mode == "Exact" and integral != decimal_units else int(integral)
        assert decimal_k == k, (op, lhs, rhs, numerator, denominator, quantum, mode, ctx.prec)
        if k is not None:
            decimal_result = ctx.multiply(integral, dec_input(quantum, ctx))
            assert Fraction(decimal_result) == k*rational(quantum)
        COUNTS["decimal-quotient-cross-checks"] += 1
    if k is None:
        return failure("inexact", operation=operation)
    return admit(k*rational(quantum), operation)


def parse(text):
    data = text.encode("utf-8")
    if len(data) > 128:
        return failure("text_too_long", length=len(data), maximum=128)
    index = int(bool(data) and data[0] in (43, 45))
    if index == len(data) or not 48 <= data[index] <= 57:
        assert GRAMMAR.fullmatch(data) is None
        return failure("invalid_syntax", index=index, reason=hx("missing integer digits"))
    while index < len(data) and 48 <= data[index] <= 57:
        index += 1
    scale = 0
    if index < len(data) and data[index] == 46:
        index += 1
        if index == len(data) or not 48 <= data[index] <= 57:
            assert GRAMMAR.fullmatch(data) is None
            return failure("invalid_syntax", index=index, reason=hx("missing fractional digits"))
        start = index
        while index < len(data) and 48 <= data[index] <= 57:
            index += 1
        scale = index-start
    if index != len(data):
        assert GRAMMAR.fullmatch(data) is None
        return failure("invalid_syntax", index=index, reason=hx("unexpected byte"))
    assert GRAMMAR.fullmatch(data) is not None
    if scale > 38:
        return failure("invalid_scale", scale=scale, maximum=38)
    value = Fraction(int(data.replace(b".", b"")), 10**scale)
    result = admit(value, "parse")
    for ctx in CONTEXTS:
        assert Fraction(ctx.create_decimal(text)) == value
        COUNTS["decimal-exact-cross-checks"] += 1
    return result


def expected(op, lhs=None, rhs=None, numerator=None, denominator=None,
             quantum=None, mode=None, text=None, terms=()):
    if op == "parse":
        return parse(text)
    if op == "mode":
        if text in MODES:
            return "mode", "text=" + hx(text), text
        return "identifier.unknown", "concept=" + hx("amount rounding") + ",text=" + hx(text), "-"
    if op in ("new", "quantum"):
        if lhs[1] > 38:
            return failure("invalid_scale", scale=lhs[1], maximum=38)
        if op == "quantum" and lhs[0] <= 0:
            return failure("invalid_quantum", coefficient=lhs[0], scale=lhs[1])
        return admit(rational(lhs), "new")
    if op in ("quantize", "ratio", "multiply_ratio", "multiply_divide"):
        return quantization(op, lhs, rhs, numerator, denominator, quantum, mode)
    if op in ("cmp", "multiple"):
        if op == "cmp":
            a, b = rational(lhs), rational(rhs)
            return "cmp", "-", str((a > b)-(a < b))
        return "bool", "-", str(int((rational(lhs)/rational(quantum)).denominator == 1))
    operation = {"sub": "subtract", "neg": "negate", "mul": "multiply"}.get(op, op)
    if op == "add":
        value = rational(lhs)+rational(rhs)
    elif op == "sub":
        value = rational(lhs)-rational(rhs)
    elif op == "neg":
        value = -rational(lhs)
    elif op == "mul":
        value = rational(lhs)*rational(rhs)
    else:
        assert op == "sum"
        value = sum((rational(item) for item in terms), Fraction())
    for ctx in CONTEXTS:
        if op == "add":
            check = ctx.add(dec_input(lhs, ctx), dec_input(rhs, ctx))
        elif op == "sub":
            check = ctx.subtract(dec_input(lhs, ctx), dec_input(rhs, ctx))
        elif op == "neg":
            check = ctx.minus(dec_input(lhs, ctx))
        elif op == "mul":
            check = ctx.multiply(dec_input(lhs, ctx), dec_input(rhs, ctx))
        else:
            check = Decimal(0)
            for item in terms:
                check = ctx.add(check, dec_input(item, ctx))
        assert Fraction(check) == value
        COUNTS["decimal-exact-cross-checks"] += 1
    return admit(value, operation)


def cases():
    def case(profile, op, **arguments):
        return profile, op, arguments

    coefficients = {MIN, MAX, 0, 1, -1, 9, -9, 10, -10, 11, -11}
    for bit in range(128):
        for delta in (-1, 0, 1):
            for sign in (-1, 1):
                value = sign*(2**bit+delta)
                if MIN <= value <= MAX:
                    coefficients.add(value)
    coefficients = sorted(coefficients)
    for scale in range(39):
        for coefficient in coefficients:
            yield case("SCALE-BITS", "new", lhs=(coefficient, scale))
        for coefficient in (1, 2, 5, 10, 25, MAX, 2**64-1, 2**64+1):
            yield case("QUANTUM-VALID", "quantum", lhs=(coefficient, scale))
        for coefficient in (0, -1, -10, MIN):
            yield case("QUANTUM-SIGN", "quantum", lhs=(coefficient, scale))
        for coefficient in (MIN, MAX, 0):
            yield case("NEGATE-LIMITS", "neg", lhs=(coefficient, scale))
    for scale in range(39, 256):
        for coefficient in (0, 1, MIN, MAX):
            yield case("SCALE-INVALID", "new", lhs=(coefficient, scale))
            yield case("QUANTUM-SCALE-FIRST", "quantum", lhs=(coefficient, scale))

    # Full short UTF-8 grammar alphabet, not just accepted-number examples.
    alphabet = ("0", "1", "+", "-", ".", "a", " ", "é")
    for length in range(6):
        for letters in itertools.product(alphabet, repeat=length):
            yield case("TEXT-SHORT", "parse", text="".join(letters))
    for scale in range(39):
        for coefficient in (MIN, MIN+1, MAX-1, MAX, -1, 0, 1, 10**37+1):
            text = display(decimal_pair(rational((coefficient, scale))))
            for variant in (text, "+"+text if not text.startswith("-") else text,
                            "-000"+text[1:] if text.startswith("-") else "000"+text):
                yield case("TEXT-CANONICAL", "parse", text=variant)
        for sign in ("", "-", "+"):
            yield case("TEXT-ZERO-PRECISION", "parse", text=sign+"0."+"0"*(scale+1))
    for integer in (str(MAX), str(MIN), str(MAX+1), str(MIN-1)):
        for zeros in range(39):
            suffix = "."+"0"*zeros if zeros else ""
            yield case("TEXT-RANGE-RESCUE", "parse", text=integer+suffix)
    for length in (127, 128, 129, 130):
        for text in ("0"*length, "0"*(length-1)+"a", "0."+"0"*(length-2),
                     "1"*length, "é"*(length//2)+("0" if length%2 else "")):
            yield case("TEXT-CAP", "parse", text=text)
    for text in ("1\x00", "\x001", "\n1", "1\r", "١", "１", "−1", "1e2", "NaN", "inf",
                 "-é", "1.é", "1.2é", "1."+"0"*39+"x", "9"*80+"."+"0"*39):
        yield case("TEXT-SPECIAL", "parse", text=text)
    for mode in MODES:
        for text in (mode, mode.lower(), " "+mode, mode+" ", mode+"\x00", mode+","+mode):
            yield case("MODE-REGISTRY", "mode", text=text)
    for text in ("", "EXACT", "Nearest", "é", "Exact"*100):
        yield case("MODE-UNKNOWN", "mode", text=text)

    small = [(c, s) for c in range(-8, 9) for s in range(4)]
    for lhs in small:
        yield case("EXACT-SMALL-NEG", "neg", lhs=lhs)
        for rhs in small:
            for op in ("add", "sub", "mul", "cmp"):
                yield case("EXACT-SMALL", op, lhs=lhs, rhs=rhs)
    for a in range(39):
        for b in range(39):
            bit = (3*a+5*b)%127
            pairs = (
                ((MAX, a), (MIN, b)), ((MIN, a), (MAX, b)),
                ((2**bit-1, a), (-(2**bit+1), b)),
                (((MAX+3)//10, a), (-3, b)),
            )
            for lhs, rhs in pairs:
                for op in ("add", "sub", "mul", "cmp"):
                    yield case("EXACT-ALL-SCALE-PAIRS", op, lhs=lhs, rhs=rhs)
    for index, coefficient in enumerate(coefficients):
        for scale in range(39):
            rhs = (coefficients[(index*13+scale*17)%len(coefficients)], (index+scale*7)%39)
            yield case("EXACT-BIT-ROTATION", ("add", "sub", "mul", "cmp", "neg")[index%5],
                       lhs=(coefficient, scale), rhs=rhs)

    for scale in range(39):
        groups = (
            (), ((MIN, scale),), ((MAX, scale), (MAX, scale)),
            ((MAX, scale), (MAX, scale), (-MAX, scale), (-MAX, scale), (1, 38)),
            ((MIN, scale), (MAX, scale), (1, scale)),
            ((MIN, scale), (MIN, scale), (MAX, scale), (MAX, scale), (2, scale)),
            ((0, scale), (0, 38)),
        )
        for group in groups:
            for sequence in (group, tuple(reversed(group)), group[1:]+group[:1]):
                yield case("SUM-PERMUTATIONS", "sum", terms=sequence)
    for count in (63, 64, 65, 255, 256, 257, 4097):
        group = ((MAX, 0),)*count + ((-MAX, 0),)*count + ((1, 38),)
        yield case("SUM-CARRY-CANCELLATION", "sum", terms=group)
        yield case("SUM-CARRY-RANGE", "sum", terms=((MIN, 38),)*count)

    quanta = [(q, s) for q in (1, 2, 3, 5, 25) for s in range(4)]
    for lhs in [(c, s) for c in range(-12, 13) for s in range(4)]:
        for quantum in quanta:
            yield case("MULTIPLE-SMALL", "multiple", lhs=lhs, quantum=quantum)
            for mode in MODES:
                yield case("QUANTIZE-SMALL", "quantize", lhs=lhs, quantum=quantum, mode=mode)

    special_q = ((1, 0), (1, 38), (5, 2), (25, 3), (MAX, 0), (MAX, 38), (10**37+1, 37))
    quotients = (0, 1, 2, 3, 7, 10, 2**31-1, 2**63, 2**100-1)
    for quantum in special_q:
        for k in quotients:
            for den in (2, 3, 10, 2**64+1):
                for delta in (-1, 0, 1):
                    num = (2*k+1)*den+delta
                    if num > MAX:
                        continue
                    for sign in (-1, 1):
                        for mode in MODES:
                            yield case("ALGEBRAIC-HALF-RATIO", "multiply_ratio", lhs=quantum,
                                       numerator=sign*num, denominator=2*den, quantum=quantum, mode=mode)
                            yield case("ALGEBRAIC-HALF-FUSED", "multiply_divide", lhs=quantum,
                                       rhs=(sign*num, 0), denominator=2*den, quantum=quantum, mode=mode)
    denominators = tuple(range(18)) + (2**127, UMAX)
    for num in range(-64, 65):
        for index, den in enumerate(denominators):
            quantum = special_q[(num+index)%len(special_q)]
            for mode in MODES:
                yield case("RATIO-SMALL-AND-UMAX", "ratio", numerator=num, denominator=den,
                           quantum=quantum, mode=mode)
    for num in (MIN, MAX, -1, 0, 1):
        for den in (0, 1, 2**127, UMAX):
            for quantum in special_q:
                for mode in MODES:
                    yield case("RATIO-LIMITS-COPRIME", "ratio", numerator=num, denominator=den,
                               quantum=quantum, mode=mode)

    for a in range(39):
        for b in range(39):
            # Include simultaneous MIN/MIN at scale38 and qMAXscale38,
            # divisorUMAX: exact381/508-bit extremal operands.
            for mode in MODES:
                yield case("FUSED-ALL-SCALE-PAIRS", "multiply_divide", lhs=(MIN, a), rhs=(MIN, b),
                           denominator=UMAX, quantum=(MAX, 38), mode=mode)
    for mode in MODES:
        for op in ("quantize", "ratio", "multiply_ratio", "multiply_divide"):
            yield case("HUGE-QUOTIENT-RESCUE", op, lhs=(MAX, 0), rhs=(1, 0),
                       numerator=MAX if op == "ratio" else 1, denominator=1,
                       quantum=(1, 38), mode=mode)
        yield case("FUSED-PRICE-DISCRIMINATOR", "multiply_divide", lhs=(MAX, 0),
                   rhs=(10**37+1, 37), denominator=100, quantum=(1, 0), mode=mode)
        for op in ("ratio", "multiply_ratio", "multiply_divide"):
            for sign in (-1, 1):
                yield case("ZERO-DENOMINATOR-FIRST", op, lhs=(0, 38), rhs=(0, 38),
                           numerator=sign*MAX, denominator=0, quantum=(MAX, 38), mode=mode)
        yield case("INEXACT-BEFORE-RANGE", "multiply_divide", lhs=(MAX, 0), rhs=(MAX, 0),
                   denominator=3, quantum=(1, 0), mode=mode)

    state = 0x6D385F414D545F31
    def word():
        nonlocal state
        state = (6364136223846793005*state+1442695040888963407) & (2**128-1)
        return state

    def amount():
        value = word()
        coefficient = value if value <= MAX else value-2**128
        return coefficient, word()%39

    for index in range(8192):
        lhs, rhs = amount(), amount()
        op = ("add", "sub", "mul", "cmp", "neg")[index%5]
        yield case("WIDE-EXACT", op, lhs=lhs, rhs=rhs)
    for index in range(2048):
        group = tuple(amount() for _ in range(index%17))
        if index%3 == 0:
            group = group + tuple((-c if c != MIN else MAX, s) for c, s in group)
        yield case("WIDE-SUM", "sum", terms=group)
    for index in range(8192):
        lhs, rhs = amount(), amount()
        numerator = word()
        numerator = numerator if numerator <= MAX else numerator-2**128
        denominator = (0, 1, 2, 3, 100, 2**64-1, 2**127, UMAX, word())[index%9]
        quantum = ((word()%MAX)+1, word()%39)
        op = ("quantize", "ratio", "multiply_ratio", "multiply_divide")[index%4]
        for mode in MODES:
            yield case("WIDE-QUANTIZED", op, lhs=lhs, rhs=rhs, numerator=numerator,
                       denominator=denominator, quantum=quantum, mode=mode)


def monotonicity():
    values = sorted({Fraction(c, 10**s) for c in range(-12, 13) for s in range(4)})
    for quantum in ((1, 0), (5, 2), (25, 3), (1, 38), (MAX, 38)):
        for mode in MODES:
            previous = None
            for value in values:
                current = rounded_units(value/rational(quantum), mode)
                if current is not None:
                    assert previous is None or previous <= current
                    previous = current
                    COUNTS["ordered-monotonicity-checks"] += 1


def generate(generated_at, platform_text):
    assert sha(CONTRACT.read_bytes()) == CONTRACT_SHA, "frozen contract changed"
    records = []
    distinct = set()
    for index, (profile, op, arguments) in enumerate(cases(), 1):
        arguments = {key: value for key, value in arguments.items() if key in OP_FIELDS[op]}
        outcome, context, output = expected(op, **arguments)
        lhs, rhs = arguments.get("lhs"), arguments.get("rhs")
        quantum, terms = arguments.get("quantum"), arguments.get("terms", ())
        row = (
            "AMT-%06d" % index, profile, op, cell(lhs), cell(rhs),
            str(arguments["numerator"]) if "numerator" in arguments else "-",
            str(arguments["denominator"]) if "denominator" in arguments else "-",
            cell(quantum), arguments.get("mode", "-"),
            hx(arguments["text"]) if "text" in arguments else "-",
            ";".join(cell(value) for value in terms) if terms else "-",
            outcome, context, output,
        )
        assert len(row) == len(COLUMNS) and all("\t" not in item and "\n" not in item for item in row)
        records.append("\t".join(row))
        distinct.add(row[2:11])
        PROFILES[profile] += 1
        OPS[op] += 1
        EXPECTATIONS[outcome] += 1
        if outcome.startswith("amount.") and "operation=" in context:
            ERROR_OPERATIONS[context.split(",")[0].split("=", 1)[1]] += 1
    monotonicity()
    assert WIDTHS["multiply_divide-numerator"] == 381
    assert WIDTHS["multiply_divide-denominator"] == 508
    assert set(EXPECTATIONS) >= {
        "ok", "bool", "cmp", "mode", "identifier.unknown", "amount.text_too_long",
        "amount.invalid_syntax", "amount.invalid_scale", "amount.invalid_quantum",
        "amount.zero_denominator", "amount.inexact", "amount.result_scale", "amount.out_of_range",
    }
    # TESTING_STRATEGY §7.3: one column line precedes the records.
    body = ("\t".join(COLUMNS)+"\n"+"\n".join(records)+"\n").encode("utf-8")
    own = Path(__file__).read_bytes()
    contract = CONTRACT.read_bytes()
    requirements = "\n".join(line for line in REQUIREMENTS.read_text().splitlines()
                              if re.match(r"\| FR-AMT-\d+ \|", line)).encode("utf-8")
    assert len(requirements.decode().splitlines()) == 14
    def census(counter):
        return ";".join("%s=%d" % item for item in sorted(counter.items()))
    headers = [
        "schema: 1", "fixture-set: EXACT-DECIMAL-AMOUNT-PHASE-A-ORACLE", "domain: amounts",
        "description: Independent exact Fraction/integer authority and two-precision Decimal cross-checks",
        "reference: Python stdlib integer/Fraction/Decimal; no Rust or QuantLib output",
        "reference-version: Python >=3.9; Decimal precisions384,512",
        "reference-source: Frozen docs/M8_AMOUNT_CONTRACT.md; Python integer/Fraction quotient-remainder authority with separately formulated Decimal cross-checks",
        "reproduction: python3 -I validation/generator/amount_oracles.py --check",
        "generated-at: " + generated_at, "platform: " + platform_text,
        "generator-sha256: " + sha(own), "generator-fnv64: " + fnv(own),
        "adopted-contract-sha256: " + sha(contract), "adopted-contract-fnv64: " + fnv(contract),
        "requirements-rows-sha256: " + sha(requirements),
        "tolerance: TOL-E0; exact canonical amounts, comparisons, predicates and structured errors; no epsilon",
        "encoding: tab-separated; decimal=coefficient:scale; -=unused/emptyterms; texthex=UTF8hex,_=emptytext; terms=semicolon-separateddecimals; context=comma-separatedkey=value; reason/concept/text context values=UTF8hex",
        "expected-encoding: ok output=canonicaldecimal context=text=canonicaldisplayhex; bool/cmp output=integer context=-; mode output=canonicalidentifier; errors output=- and context=allstructuredfields",
        "source-scope: Frozen local phaseA contract; no phaseB floats or market settlement certification",
        "record-count: " + str(len(records)), "unique-input-count: " + str(len(distinct)),
        "profile-counts: " + census(PROFILES), "operation-counts: " + census(OPS),
        "expectation-counts: " + census(EXPECTATIONS), "error-operation-counts: " + census(ERROR_OPERATIONS),
        "cross-check-counts: " + census(COUNTS), "maximum-width-bits: " + census(WIDTHS),
        "body-sha256: " + sha(body), "body-fnv64: " + fnv(body),
    ]
    data = ("\n".join(headers)+"\n\n").encode()+body
    summary = {
        "records": len(records), "unique_inputs": len(distinct), "bytes": len(data),
        "sha256": sha(data), "body_sha256": sha(body), "body_fnv64": fnv(body),
        "generator_sha256": sha(own), "contract_sha256": sha(contract),
        "requirements_rows_sha256": sha(requirements), "profiles": dict(sorted(PROFILES.items())),
        "operations": dict(sorted(OPS.items())), "expectations": dict(sorted(EXPECTATIONS.items())),
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
            raise SystemExit("amount fixture differs from independent regeneration")
        summary["check"] = "identical"
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(data)
        summary["written"] = str(args.output.relative_to(ROOT)) if args.output.is_relative_to(ROOT) else str(args.output)
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
