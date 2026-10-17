#!/usr/bin/env python3
"""Independent frozen phase-C settlement oracle (Python stdlib only).

The authority is integer/Fraction mathematics from the frozen contract, not
Rust output. A separate Decimal rational expression, decoded from original
inputs, checks every visited rounding quotient at 1536 and 2048 digits. Small
staged enclosures also use independent quantizer-cell and Cartesian enumeration.
See docs/M8_SETTLEMENT_ORACLES.md for schema, proofs and honest scope.
"""

import argparse
from collections import Counter
from dataclasses import dataclass
import datetime as dt
from decimal import (
    Context, Decimal, ROUND_CEILING, ROUND_DOWN, ROUND_FLOOR, ROUND_HALF_EVEN,
    ROUND_HALF_UP, ROUND_UP,
)
from fractions import Fraction as F
import hashlib
import itertools
import json
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/settlement/oracles.fixture"
CONTRACT = ROOT / "docs/M8_SETTLEMENT_CONTRACT.md"
CONTRACT_SHA = "47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba"
REQUIREMENTS_SHA = "f500bb6926d8f38f789817e342397719543d7ffd8f73dc1f114b640da770f637"
MIN, MAX = -(2**127), 2**127 - 1
D0 = 2**1074 * 5**38
MODES = (
    "TowardZero", "AwayFromZero", "TowardPositiveInfinity",
    "TowardNegativeInfinity", "NearestTiesEven", "NearestTiesAwayFromZero", "Exact",
)
DECIMAL_MODES = dict(zip(MODES[:-1], (
    ROUND_DOWN, ROUND_UP, ROUND_CEILING, ROUND_FLOOR, ROUND_HALF_EVEN, ROUND_HALF_UP,
)))
CONTEXTS = (Context(prec=1536), Context(prec=2048))
RECIPES = ("DirtyPer100", "CleanAccruedPer100", "StagedPrice100Accrued1000", "CouponPer1000")
AGGREGATIONS = ("RoundTotal", "RoundComponentsThenSum")
GUARANTEES = ("ForSuppliedDecimals", "ForSuppliedBinary64", "UniqueForDeclaredEnclosure")
COLUMNS = ("id profile op recipe nominal primary secondary aggregation price_mode "
           "accrued_mode quantum cash_mode expected context lower upper guarantee").split()
COUNTS, WIDTHS = Counter(), Counter()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    value = 14695981039346656037
    for byte in data:
        value = ((value ^ byte) * 1099511628211) & (2**64 - 1)
    return "%016x" % value


def hx(text):
    return text.encode("utf-8").hex() or "_"


def canon(c, s):
    if not c:
        return 0, 0
    while s and c % 10 == 0:
        c //= 10
        s -= 1
    return c, s


def cell(pair):
    return "%d:%d" % pair


def rat(pair):
    return F(pair[0], 10**pair[1])


def pair(value):
    """Minimal finite decimal; final cash denominators have only factors2,5."""
    if not value:
        return 0, 0
    denominator = value.denominator
    a = b = 0
    while denominator % 2 == 0:
        a += 1
        denominator //= 2
    while denominator % 5 == 0:
        b += 1
        denominator //= 5
    assert denominator == 1
    s = max(a, b)
    return value.numerator * 2**(s-a) * 5**(s-b), s


def display(value):
    c, s = value
    digits = str(abs(c)).rjust(s+1, "0")
    return ("-" if c < 0 else "") + ((digits[:-s] + "." + digits[-s:]) if s else digits)


def bits(value):
    return struct.unpack(">Q", struct.pack(">d", value))[0]


def ieee(value):
    return struct.unpack(">d", struct.pack(">Q", value))[0]


def d(c, s=0):
    return "d:" + cell((c, s))


def di(c, s, e, t=None):
    return "di:%d:%d:%d:%d" % (c, s, e, s if t is None else t)


def b(value):
    return "b:%016x" % value


def bi(lo, hi):
    return "bi:%016x:%016x" % (lo, hi)


def width(key, integer):
    WIDTHS[key] = max(WIDTHS[key], abs(integer).bit_length())


@dataclass(frozen=True)
class Point:
    value: F
    # Independent exact Decimal integer numerator/denominator per precision.
    decimals: tuple

    def scaled(self, nominal, divisor):
        n, a = nominal
        den = 10**a * divisor
        decimals = tuple((ctx.multiply(x, Decimal(n)), ctx.multiply(y, Decimal(den)))
                         for ctx, (x, y) in zip(CONTEXTS, self.decimals))
        return checked(Point(self.value * F(n, den), decimals))

    def plus(self, other):
        decimals = tuple((ctx.add(ctx.multiply(x, v), ctx.multiply(u, y)), ctx.multiply(y, v))
                         for ctx, (x, y), (u, v) in zip(CONTEXTS, self.decimals, other.decimals))
        return checked(Point(self.value + other.value, decimals))


def checked(point):
    for x, y in point.decimals:
        assert x == x.to_integral_value() and y == y.to_integral_value() and y > 0
        assert F(int(x), int(y)) == point.value
        width("decimal-unreduced-numerator", int(x))
        width("decimal-unreduced-denominator", int(y))
        # This also verifies exactness of every pre-division Decimal operation.
        assert len(x.as_tuple().digits) <= 1536 and len(y.as_tuple().digits) <= 1536
    return point


def input_point(kind, raw):
    if kind == "d":
        c, s = raw
        value = rat(raw)
        u = c * 2**(1074-s) * 5**(38-s)
    else:
        # Fraction authority uses Python's exact float ratio. The Decimal path
        # separately decodes sign, exponent and fraction directly from IEEE bits.
        value = F.from_float(ieee(raw))
        sign = -1 if raw >> 63 else 1
        exponent, mantissa = (raw >> 52) & 2047, raw & (2**52 - 1)
        if exponent:
            mantissa += 2**52
            shift = exponent - 1
        else:
            shift = 0
        u = sign * mantissa * 2**shift * 5**38
    return checked(Point(value, tuple((Decimal(u), Decimal(D0)) for _ in CONTEXTS)))


@dataclass(frozen=True)
class Image:
    lower: Point
    upper: Point
    continuous: bool = False
    steps: tuple = ()

    @property
    def point(self):
        return self.lower.value == self.upper.value

    def scaled(self, nominal, divisor):
        factor = rat(nominal) / divisor
        assert factor > 0
        return Image(self.lower.scaled(nominal, divisor), self.upper.scaled(nominal, divisor),
                     self.continuous, tuple(step * factor for step in self.steps))

    def plus(self, other):
        return Image(self.lower.plus(other.lower), self.upper.plus(other.upper),
                     self.continuous or other.continuous, self.steps + other.steps)


class Stop(Exception):
    def __init__(self, expected, context):
        super().__init__(expected)
        self.expected, self.context = expected, context


def stop(kind, **context):
    raise Stop(kind, ",".join("%s=%s" % item for item in context.items()))


def arithmetic(stage, endpoint, cause, operation="quantize", **extra):
    stop("settlement.arithmetic", stage=stage, endpoint=endpoint,
         cause="amount." + cause, operation=operation, **extra)


def decode(text):
    cells = text.split(":")
    kind, interval = cells[0][0], cells[0].endswith("i")
    if kind == "d":
        raw = [canon(int(cells[1]), int(cells[2]))]
        if interval:
            raw.append(canon(int(cells[3]), int(cells[4])))
        else:
            raw.append(raw[0])
    else:
        raw = [int(x, 16) for x in cells[1:]]
        for i, value in enumerate(raw):
            if ((value >> 52) & 2047) == 2047:
                classification = ("NaN" if value & (2**52-1) else
                                  "NegativeInfinity" if value >> 63 else "PositiveInfinity")
                stop("settlement.non_finite", endpoint=("lower", "upper")[i] if interval else "value",
                     value=classification)
        raw = [0 if value & (2**63-1) == 0 else value for value in raw]
        if not interval:
            raw.append(raw[0])
    points = [input_point(kind, value) for value in raw]
    if points[0].value > points[1].value:
        if kind == "d":
            stop("settlement.decimal_interval_order", lower=cell(raw[0]), upper=cell(raw[1]))
        stop("settlement.binary_interval_order", lower_bits="%016x" % raw[0], upper_bits="%016x" % raw[1])
    return Image(*points, points[0].value != points[1].value), kind, interval, tuple(raw)


def units_round(value, mode):
    sign = -1 if value < 0 else 1
    magnitude = abs(value)
    k, rem = divmod(magnitude.numerator, magnitude.denominator)
    den = magnitude.denominator
    if rem and mode == "Exact":
        return None
    increment = bool(rem) and (
        mode == "AwayFromZero" or (mode == "TowardPositiveInfinity" and sign > 0)
        or (mode == "TowardNegativeInfinity" and sign < 0)
        or (mode == "NearestTiesEven" and (2*rem > den or (2*rem == den and k % 2)))
        or (mode == "NearestTiesAwayFromZero" and 2*rem >= den))
    return sign * (k + increment)


def round_point(point, quantum, mode):
    q, s = quantum
    value = point.value * 10**s / q
    answer = units_round(value, mode)
    width("reduced-quantizer-numerator", value.numerator)
    width("reduced-quantizer-denominator", value.denominator)
    assert abs(value.numerator) < 2**2446 and value.denominator < 2**1427
    if abs(value).denominator == 2:
        COUNTS["algebraic-half-quantum-rounds"] += 1
    decimals = []
    for ctx, (x, y) in zip(CONTEXTS, point.decimals):
        numerator, denominator = ctx.multiply(x, Decimal(10**s)), ctx.multiply(y, Decimal(q))
        width("decimal-quotient-numerator", int(numerator))
        width("decimal-quotient-denominator", int(denominator))
        assert F(int(numerator), int(denominator)) == value
        quotient = ctx.divide(numerator, denominator)  # Exactly one division.
        rounded = quotient.to_integral_value(rounding=DECIMAL_MODES.get(mode, ROUND_DOWN), context=ctx)
        got = None if mode == "Exact" and rounded != quotient else int(rounded)
        assert got == answer, (point.value, quantum, mode, ctx.prec, got, answer)
        COUNTS["decimal-quotient-cross-checks"] += 1
        if answer is not None:
            decimals.append((ctx.multiply(rounded, Decimal(q)), Decimal(10**s)))
    if answer is None:
        COUNTS["exact-inexact-rounds"] += 1
        return None
    result = checked(Point(F(answer*q, 10**s), tuple(decimals)))
    # Independent invariants for every exact rational quantization result.
    out, src = result.value, point.value
    assert out / rat(quantum) == answer
    if mode == "TowardZero":
        assert abs(out) <= abs(src) and abs(src-out) < rat(quantum)
    elif mode == "AwayFromZero":
        assert abs(out) >= abs(src) and abs(src-out) < rat(quantum)
    elif mode == "TowardPositiveInfinity":
        assert out >= src and out-src < rat(quantum)
    elif mode == "TowardNegativeInfinity":
        assert out <= src and src-out < rat(quantum)
    elif mode.startswith("Nearest"):
        assert abs(out-src) <= rat(quantum)/2
    else:
        assert out == src
    if src and not out:
        COUNTS["nonzero-rounded-zero"] += 1
    assert units_round(out / rat(quantum), mode) == answer
    COUNTS["rounding-inequality-idempotence-checks"] += 1
    return result


def quantize(image, quantum, mode, stage, source=False):
    if source:
        for endpoint in (image.lower, image.upper):
            integer = endpoint.value * D0
            assert integer.denominator == 1
            numerator, denominator = integer.numerator * 10**quantum[1], D0 * quantum[0]
            assert abs(numerator) < 2**2314 and denominator < 2**1290
            width("common-basis-source-numerator", numerator)
            width("common-basis-source-denominator", denominator)
    if mode == "Exact":
        if not image.point:
            if image.continuous or any((step / rat(quantum)).denominator != 1 for step in image.steps):
                stop("exact_unresolved", stage=stage)
            if (image.lower.value / rat(quantum)).denominator != 1:
                stop("exact_unresolved", stage=stage)
        lower = round_point(image.lower, quantum, mode)
        if lower is None:
            arithmetic(stage, "point", "inexact")
        upper = lower if image.point else round_point(image.upper, quantum, mode)
        assert upper is not None
        return Image(lower, upper, False, image.steps)
    lower = round_point(image.lower, quantum, mode)
    upper = lower if image.point else round_point(image.upper, quantum, mode)
    assert lower.value <= upper.value
    if not image.point:
        COUNTS["monotone-endpoint-checks"] += 1
    return Image(lower, upper, False, (rat(quantum),) if source and lower.value != upper.value else ())


def admit(value, stage, endpoint, operation):
    c, s = pair(value)
    assert s <= 38
    if not MIN <= c <= MAX:
        arithmetic(stage, endpoint, "out_of_range", operation, negative=int(c < 0))
    for ctx in CONTEXTS:
        assert format(ctx.scaleb(Decimal(c), Decimal(-s)), "f") == display((c, s))
    return c, s


def cash_width(images, weights, nominal, divisor, quantum, mode):
    """Observe the contract's common-basis integers without Decimal reduction."""
    for which in ("lower", "upper"):
        common = sum(getattr(image, which).value * D0 * weight
                     for image, weight in zip(images, weights))
        assert common.denominator == 1
        numerator = common.numerator * nominal[0] * 10**quantum[1]
        denominator = D0 * 10**nominal[1] * divisor * quantum[0]
        assert abs(numerator) < 2**2446 and denominator < 2**1427
        width("common-basis-cash-numerator", numerator)
        width("common-basis-cash-denominator", denominator)
        count = units_round(F(numerator, denominator), mode)
        if count is not None:
            product = count * quantum[0]
            assert abs(product) < 2**2447
            width("common-basis-rounded-count-times-quantum", product)


def outcome(image, ispoint, stage, operation, guarantee):
    lo = admit(image.lower.value, stage, "point" if ispoint else "lower", operation)
    hi = lo if ispoint else admit(image.upper.value, stage, "upper", operation)
    if lo == hi:
        return "determined", "text=" + hx(display(lo)), cell(lo), "-", guarantee
    assert rat(lo) < rat(hi)
    return "ambiguous", "-", cell(lo), cell(hi), "-"


def evaluate(recipe, nominal, primary, secondary, aggregation, price_mode, accrued_mode, quantum, mode):
    # Corpus evaluation arguments are valid values; constructor-order errors
    # have their own rows, so the oracle invents no external call precedence.
    first = decode(primary)
    second = decode(secondary) if secondary != "-" else None
    if nominal[0] <= 0:
        stop("settlement.nonpositive_nominal", nominal=cell(canon(*nominal)))
    evidences = (first,) if second is None else (first, second)
    guarantee = (GUARANTEES[2] if any(e[2] for e in evidences) else
                 GUARANTEES[1] if any(e[1] == "b" for e in evidences) else GUARANTEES[0])
    p, a = first[0], second[0] if second else None
    if recipe == RECIPES[2]:
        p = quantize(p, (1, 6), price_mode, "source price", source=True)
        a = quantize(a, (1, 5), accrued_mode, "source accrued", source=True)
    capital = p.scaled(nominal, 1000 if recipe == RECIPES[3] else 100)
    if a is None:
        stage = "cash coupon" if recipe == RECIPES[3] else "cash total"
        cash_width((p,), (1,), nominal, 1000 if recipe == RECIPES[3] else 100, quantum, mode)
        cash = quantize(capital, quantum, mode, stage)
        return outcome(cash, capital.point, stage, "quantize", guarantee)
    accrued = a.scaled(nominal, 1000 if recipe == RECIPES[2] else 100)
    if aggregation == AGGREGATIONS[0]:
        cash_width((p, a), (10, 1) if recipe == RECIPES[2] else (1, 1), nominal,
                   1000 if recipe == RECIPES[2] else 100, quantum, mode)
        total = capital.plus(accrued)
        cash = quantize(total, quantum, mode, "cash total")
        return outcome(cash, total.point, "cash total", "quantize", guarantee)
    cash_width((p,), (1,), nominal, 100, quantum, mode)
    c = quantize(capital, quantum, mode, "cash capital")
    cash_width((a,), (1,), nominal, 1000 if recipe == RECIPES[2] else 100, quantum, mode)
    a = quantize(accrued, quantum, mode, "cash accrued")
    net = c.plus(a)
    return outcome(net, net.point, "cash net", "sum", guarantee)


def enum_round(value, quantum, mode):
    """Independent nearby-candidate selection, with no quotient/remainder rule."""
    q = rat(quantum)
    units = value / q
    floor = units.numerator // units.denominator
    candidates = [(k*q, k) for k in range(floor-1, floor+3)]
    if mode == "Exact":
        return next((v for v, _ in candidates if v == value), None)
    if mode == "TowardPositiveInfinity":
        return min(v for v, _ in candidates if v >= value)
    if mode == "TowardNegativeInfinity":
        return max(v for v, _ in candidates if v <= value)
    if mode == "TowardZero":
        return max(v for v, _ in candidates if 0 <= v <= value) if value >= 0 else min(v for v, _ in candidates if value <= v <= 0)
    if mode == "AwayFromZero":
        return min(v for v, _ in candidates if v >= value and v >= 0) if value >= 0 else max(v for v, _ in candidates if v <= value and v <= 0)
    distance = min(abs(v-value) for v, _ in candidates)
    tied = [(v, k) for v, k in candidates if abs(v-value) == distance]
    if mode == "NearestTiesEven":
        return next((v for v, k in tied if k % 2 == 0), tied[0][0])
    return max((v for v, _ in tied), key=abs)


def cell_samples(lo, hi, quantum):
    """Every threshold and open cell, including directed zero and tie owners."""
    q = rat(quantum)
    left = (lo/q).numerator // (lo/q).denominator
    right = (hi/q).numerator // (hi/q).denominator
    assert right-left <= 12
    events = {lo, hi}
    for k in range(left-1, right+2):
        for value in (k*q, (F(k)+F(1, 2))*q):
            if lo <= value <= hi:
                events.add(value)
    ordered = sorted(events)
    return ordered + [(x+y)/2 for x, y in zip(ordered, ordered[1:])]


def enumerate_staged(args, expected):
    """Independent finite source cells, then full admissible Cartesian sets."""
    recipe, nominal, primary, secondary, aggregation, pm, am, q, mode = args
    assert recipe == RECIPES[2]
    sources = []
    try:
        for text, quantum, smode, stage in ((primary, (1, 6), pm, "source price"),
                                          (secondary, (1, 5), am, "source accrued")):
            image = decode(text)[0]
            lo, hi = image.lower.value, image.upper.value
            samples = cell_samples(lo, hi, quantum)
            values = {enum_round(v, quantum, smode) for v in samples}
            COUNTS["enumerated-source-cell-samples"] += len(samples)
            if None in values:
                if lo == hi:
                    arithmetic(stage, "point", "inexact")
                stop("exact_unresolved", stage=stage)
            sources.append(values)
        cset = {rat(nominal)*v/100 for v in sources[0]}
        aset = {rat(nominal)*v/1000 for v in sources[1]}
        def finite_round(values, stage):
            outputs = {enum_round(v, q, mode) for v in values}
            COUNTS["enumerated-cash-inputs"] += len(values)
            if None in outputs:
                if len(values) == 1:
                    arithmetic(stage, "point", "inexact")
                stop("exact_unresolved", stage=stage)
            return outputs
        if aggregation == AGGREGATIONS[0]:
            inputs = {x+y for x in cset for y in aset}
            outputs = finite_round(inputs, "cash total")
            stage, operation, ispoint = "cash total", "quantize", len(inputs) == 1
        else:
            cs = finite_round(cset, "cash capital")
            ac = finite_round(aset, "cash accrued")
            outputs = {x+y for x in cs for y in ac}
            stage, operation, ispoint = "cash net", "sum", len(outputs) == 1
        guarantee = (GUARANTEES[2] if primary.startswith(("di:", "bi:")) or secondary.startswith(("di:", "bi:"))
                     else GUARANTEES[1] if primary.startswith("b:") or secondary.startswith("b:") else GUARANTEES[0])
        lo = admit(min(outputs), stage, "point" if ispoint else "lower", operation)
        hi = lo if ispoint else admit(max(outputs), stage, "upper", operation)
        actual = (("determined", "text=" + hx(display(lo)), cell(lo), "-", guarantee) if lo == hi
                  else ("ambiguous", "-", cell(lo), cell(hi), "-"))
    except Stop as error:
        actual = error.expected, error.context, "-", "-", "-"
    assert actual == expected, (args, actual, expected)
    COUNTS["independent-enumerated-instructions"] += 1


class Corpus:
    def __init__(self):
        self.records = []
        self.profiles, self.ops, self.expected = Counter(), Counter(), Counter()
        self.recipes, self.stages, self.guarantees = Counter(), Counter(), Counter()
        self.causes, self.endpoints, self.amount_operations = Counter(), Counter(), Counter()
        self.cash_modes = Counter()
        self.unique = set()

    def append(self, profile, op, inputs, answer):
        fields = ["SET-%06d" % (len(self.records)+1), profile, op] + list(inputs) + list(answer)
        assert len(fields) == len(COLUMNS)
        assert all("\t" not in field and "\n" not in field and field for field in fields)
        self.records.append("\t".join(fields) + "\n")
        self.profiles[profile] += 1
        self.ops[op] += 1
        self.expected[answer[0]] += 1
        self.unique.add(tuple([op] + list(inputs)))
        if op == "evaluate":
            self.recipes[inputs[0]] += 1
            self.cash_modes[inputs[-1]] += 1
        if answer[-1] != "-":
            self.guarantees[answer[-1]] += 1
        if answer[1].startswith("stage="):
            self.stages[answer[1].split(",", 1)[0][6:]] += 1
        context = dict(part.split("=", 1) for part in answer[1].split(",")) if answer[1] != "-" else {}
        for key, census in (("cause", self.causes), ("endpoint", self.endpoints), ("operation", self.amount_operations)):
            if key in context:
                census[context[key]] += 1

    def evaluate(self, profile, recipe=RECIPES[0], nominal=(100, 0), primary="d:1:0", secondary=None,
                 aggregation=AGGREGATIONS[0], price_mode=MODES[4], accrued_mode=MODES[4],
                 quantum=(1, 2), mode=MODES[4], enumerate_small=False):
        nominal, quantum = canon(*nominal), canon(*quantum)
        assert 0 < quantum[0] <= MAX and 0 <= quantum[1] <= 38
        if recipe in (RECIPES[0], RECIPES[3]):
            secondary, aggregation = "-", "-"
        else:
            secondary = secondary or d(0)
        if recipe != RECIPES[2]:
            price_mode, accrued_mode = "-", "-"
        args = (recipe, nominal, primary, secondary, aggregation, price_mode, accrued_mode, quantum, mode)
        try:
            answer = evaluate(*args)
        except Stop as error:
            answer = error.expected, error.context, "-", "-", "-"
        if enumerate_small:
            enumerate_staged(args, answer)
        inputs = (recipe, cell(nominal), primary, secondary, aggregation, price_mode, accrued_mode, cell(quantum), mode)
        self.append(profile, "evaluate", inputs, answer)

    def evidence(self, profile, text):
        try:
            _, kind, declared, raw = decode(text)
            outputs = tuple(cell(x) if kind == "d" else "%016x" % x for x in raw)
            answer = "evidence", "kind=%s,declared=%d" % ("decimal" if kind == "d" else "binary64", declared), *outputs, "-"
        except Stop as error:
            answer = error.expected, error.context, "-", "-", "-"
        self.append(profile, "evidence", ("-", "-", text, "-", "-", "-", "-", "-", "-"), answer)

    def identifier(self, op, text):
        registry = AGGREGATIONS if op == "aggregation" else GUARANTEES
        concept = "cash aggregation" if op == "aggregation" else "settlement guarantee"
        answer = (("identifier", "-", text, "-", "-") if text in registry else
                  ("identifier.unknown", "concept=%s,text=%s" % (hx(concept), hx(text)), "-", "-", "-"))
        self.append("IDENTIFIERS", op, ("-", "-", "x:" + hx(text), "-", "-", "-", "-", "-", "-"), answer)


def populate(c):
    for op, registry in (("aggregation", AGGREGATIONS), ("guarantee", GUARANTEES)):
        for name in registry:
            for text in (name, name.lower(), name.upper(), " " + name, name + " ", name + "\0"):
                c.identifier(op, text)
        for text in ("", "\t", "\n", "Unknown", "é", "Exact", "Round Total"):
            c.identifier(op, text)

    for scale in range(39):
        for coefficient in (MIN, MIN+1, -100, -10, -1, 0, 1, 10, 100, MAX-1, MAX):
            c.evidence("DECIMAL-EVIDENCE-39-SCALES", d(coefficient, scale))
            c.evidence("DECIMAL-DECLARED-POINT", di(coefficient, scale, coefficient))
        for lo, hi in ((MIN, MAX), (-1, 1), (0, 1), (1, 0), (MAX, MIN)):
            c.evidence("DECIMAL-INTERVAL-ORDER", di(lo, scale, hi))
        for recipe in RECIPES:
            for mode in MODES:
                for sign in (-1, 1):
                    c.evaluate("ALL-SCALES-SIGNED-LIMITS", recipe, (MAX, scale), d(sign*MAX, scale),
                               d(sign*MIN, scale) if sign > 0 else d(MAX, scale),
                               aggregation=AGGREGATIONS[scale % 2], quantum=(1+(scale%7), scale), mode=mode)
                    c.evaluate("ALL-SCALES-TINY-NOMINAL", recipe, (1, scale), d(sign, scale),
                               d(-sign, scale), aggregation=AGGREGATIONS[scale % 2], quantum=(5, min(scale+1, 38)), mode=mode)

    finite_special = [0, 2**63, 1, 2**63+1, 2**52-1, 2**52, 2**52+1,
                      bits(0.1), bits(-0.1), bits(1.0), bits(-1.0), 0x7fefffffffffffff, 0xffefffffffffffff]
    nonfinite = [0x7ff0000000000000, 0xfff0000000000000, 0x7ff0000000000001,
                 0x7ff8000000000000, 0xfff0000000000001, 0xfff8000000000001]
    for raw in finite_special + nonfinite:
        c.evidence("BINARY-EVIDENCE-SPECIAL", b(raw))
        for other in finite_special + nonfinite:
            c.evidence("BINARY-INTERVAL-ORDER-FINITE-FIRST", bi(raw, other))
    for exponent in range(2047):
        for mantissa in (0, 1, 2**52-1):
            if exponent == 0 and mantissa == 0:
                pass
            for sign in (0, 2**63):
                raw = sign | (exponent << 52) | mantissa
                c.evidence("BINARY-ALL-EXPONENT-BOUNDARIES", b(raw))
                for i, mode in enumerate(MODES):
                    recipe = RECIPES[(exponent + mantissa + i) % 4]
                    nominal = ((1, 38) if exponent > 1150 else (MAX, 0) if exponent < 900 else (137, 0))
                    quantum = ((1, 38), (5, 2), (MAX, 38), (1, 0))[(exponent+i) % 4]
                    c.evaluate("BINARY-ALL-EXPONENT-RECIPES", recipe, nominal, b(raw), d((-1)**i*17, 3),
                               AGGREGATIONS[i % 2], MODES[(exponent+i)%7], MODES[(exponent+2*i)%7], quantum, mode)
    for shift in range(52):
        for offset in (-1, 0, 1):
            mantissa = (1 << shift) + offset
            for sign in (0, 2**63):
                raw = sign | mantissa
                c.evidence("SUBNORMAL-LEADING-BITS", b(raw))
                c.evaluate("SUBNORMAL-INTERVAL-CONTINUOUS", primary=bi(raw, raw+1) if sign == 0 else bi(raw+1, raw),
                           nominal=(MAX, 0), quantum=(1, 38), mode=MODES[(shift+offset)%7])

    # Algebraic half cash quanta, with +/-one decimal unit neighbors. Exact
    # expected ties are constructed as rationals, never guessed from float text.
    for scale in range(37):
        for coefficient in (1, 3, 5, 25):
            q = coefficient, scale
            for k in (-7, -2, -1, 0, 1, 2, 7):
                for offset in (-1, 0, 1):
                    source = (2*k+1)*coefficient*5 + offset
                    for mode in MODES:
                        c.evaluate("ALGEBRAIC-CASH-TIES", nominal=(100, 0), primary=d(source, scale+1), quantum=q, mode=mode)

    # Full source-mode pairs, finite source threshold neighborhoods, both cash
    # aggregations, all final modes. Exhaustive independent source-cell sets.
    for lo, hi in ((-15, -5), (-5, 5), (0, 0), (0, 20), (5, 15), (15, 25), (-25, 25)):
        for pm, am in itertools.product(MODES, repeat=2):
            for aggregation in AGGREGATIONS:
                for mode in MODES:
                    c.evaluate("ENUMERATED-SOURCE-CELLS", RECIPES[2], (50, 0), di(lo, 7, hi), di(-hi, 6, -lo),
                               aggregation, pm, am, (1, 6), mode, enumerate_small=True)

    for nominal in ((1, 0), (50, 0), (100, 0), (200, 0), (1, 38), (MAX, 38)):
        for q in ((1, 0), (1, 6), (5, 7), (1, 38)):
            for pm in MODES:
                for mode in MODES:
                    c.evaluate("EXACT-DISCRETE-BASE-STEP", RECIPES[2], nominal, di(0, 6, 2), d(0),
                               AGGREGATIONS[(q[1]+nominal[0])%2], pm, MODES[6], q, mode, enumerate_small=True)
                    c.evaluate("EXACT-DISCRETE-TWO-STEPS", RECIPES[2], nominal, di(-1, 6, 1), di(0, 5, 2),
                               AGGREGATIONS[(q[1]+nominal[0]+1)%2], pm, MODES[0], q, mode, enumerate_small=True)

    # Evidence provenance mixes with identical numerical points and narrow,
    # genuinely continuous intervals. They remain distinct fixture inputs.
    evidence = (d(1), b(bits(1.0)), di(1, 0, 1), bi(bits(1.0), bits(1.0)),
                di(9999999, 7, 10000001), bi(bits(1.0), bits(1.0)+1))
    for recipe in RECIPES:
        for primary, secondary in itertools.product(evidence, repeat=2):
            if recipe in (RECIPES[0], RECIPES[3]) and secondary != evidence[0]:
                continue
            for mode in MODES:
                for agg in (AGGREGATIONS if recipe in (RECIPES[1], RECIPES[2]) else (AGGREGATIONS[0],)):
                    c.evaluate("PROVENANCE-POINT-CONTINUOUS", recipe, (100, 0), primary, secondary, agg,
                               MODES[4], MODES[4], (1, 2), mode)

    specials = [
        # Component Exact cannot use an exact combined sum to rescue halves.
        ((100, 0), d(5, 1), d(5, 1), (1, 0)),
        ((100, 0), d(MAX), d(MIN), (1, 0)),
        ((1000, 0), d(MAX), d(MIN), (1, 0)),
        ((MAX, 0), d(MAX), d(-MAX), (1, 0)),
        # Source/rounded components can exceed DecimalAmount, with a safe net.
        ((1, 38), b(bits(1e39)), b(bits(-1e39)), (1, 2)),
        ((100, 0), b(bits(1e39)), b(bits(-1e39)), (1, 2)),
        ((MAX, 0), d(10**37+1, 37), d(0), (1, 0)),
        # Distinct representable endpoints, unrepresentable decimal interior.
        ((100, 0), di(1, 0, 2), d(0), (1, 38)),
        ((100, 0), di(MAX-1, 0, MAX), d(0), (1, 0)),
        ((100, 0), di(MIN, 0, MIN+1), d(0), (1, 0)),
        ((200, 0), di(MAX//2, 0, MAX//2+1), d(0), (1, 0)),
        ((200, 0), di(MIN//2-1, 0, MIN//2), d(0), (1, 0)),
        # Source price precedes source accrued; collapse changes later pointness.
        ((100, 0), di(1, 7, 2), di(1, 6, 2), (1, 0)),
        ((100, 0), di(49, 8, 499), d(0), (1, 0)),
        ((100, 0), di(151, 8, 159), d(0), (1, 0)),
        # Same nonpoint quote: direct cash failure labels lower; staged source
        # collapse makes the later cash input a point, retaining provenance.
        ((MAX, 0), di(10**38+1, 36, 10**38+2), d(0), (10, 0)),
        ((MAX, 0), di(99999999, 8, 100000001), d(0), (1, 38)),
    ]
    for recipe in RECIPES:
        for nominal, primary, secondary, q in specials:
            for mode in MODES:
                for agg in (AGGREGATIONS if recipe in (RECIPES[1], RECIPES[2]) else (AGGREGATIONS[0],)):
                    for smode in (MODES if recipe == RECIPES[2] else (MODES[4],)):
                        c.evaluate("STAGE-PRECEDENCE-RANGE-RESCUE", recipe, nominal, primary, secondary,
                                   agg, smode, MODES[6], q, mode)
    for nominal in ((0, 0), (-1, 0), (MIN, 0), (-1, 38)):
        for recipe, mode in itertools.product(RECIPES, MODES):
            c.evaluate("NONPOSITIVE-NOMINAL", recipe, nominal, di(0, 0, 1), d(1, 7), mode=mode,
                       price_mode=MODES[6], accrued_mode=MODES[6])

    for primary, secondary in ((b(0x7fefffffffffffff), b(1)),
                               (b(0xffefffffffffffff), b(2**63+1)),
                               (b(0x7fefffffffffffff), d(1, 38)),
                               (b(1), d(1, 38)),
                               (bi(0x7feffffffffffffe, 0x7fefffffffffffff), b(1))):
        for recipe in RECIPES:
            for nominal in ((MAX, 0), (1, 38), (MAX, 38)):
                for quantum in ((1, 38), (MAX, 0), (MAX, 38)):
                    for mode in MODES:
                        c.evaluate("FULL-WIDTH-COMMON-BASIS", recipe, nominal, primary, secondary,
                                   AGGREGATIONS[(nominal[1]+quantum[1])%2], MODES[4], MODES[4], quantum, mode)

    # Deterministic wide values rotate independent dimensions, rather than
    # repeating a full product of mostly equivalent scale/mode combinations.
    state = 0x534554544c454d454e54
    def next_int():
        nonlocal state
        state = (state * 6364136223846793005 + 1442695040888963407) & (2**128-1)
        return state
    for i in range(16000):
        scale, ns, qs = i % 39, (i*17+3) % 39, (i*23+1) % 39
        x = next_int() % MAX * (-1 if i & 1 else 1)
        y = next_int() % MAX * (-1 if i & 2 else 1)
        nominal = next_int() % MAX + 1, ns
        primary, secondary = d(x, scale), d(y, (scale+11) % 39)
        if i % 5 == 0:
            primary = di(min(x, y), scale, max(x, y))
        if i % 11 == 0:
            raw = next_int() & (2**64-1)
            raw = raw & ~(2047 << 52) | ((raw >> 52) % 2047 << 52)
            primary = b(raw)
        q = next_int() % MAX + 1, qs
        c.evaluate("WIDE-ROTATING-INDEPENDENT", RECIPES[i % 4], nominal, primary, secondary,
                   AGGREGATIONS[(i//4)%2], MODES[(i//8)%7], MODES[(i//56)%7], q, MODES[(i//392)%7])


def counter_text(counter):
    return ";".join("%s=%d" % item for item in sorted(counter.items()))


def generate(generated_at, platform_text):
    contract = CONTRACT.read_bytes()
    assert sha(contract) == CONTRACT_SHA, "frozen settlement contract changed"
    requirements = "\n".join(line for line in (ROOT / "docs/REQUIREMENTS.md").read_text().splitlines()
                             if line.startswith("| FR-SETTLE-")).encode()
    assert sha(requirements) == REQUIREMENTS_SHA, "frozen settlement requirements changed"
    assert len(requirements.splitlines()) == 16
    # Algebraic loose upper bounds, independently checked with Python integers.
    # These are proofs, not forged public values or observed corpus maxima.
    assert D0 < 2**1163
    source_bound = 2**1025 * D0
    assert source_bound < 2**2188
    weighted_bound = 11 * source_bound
    assert weighted_bound < 2**2192
    assert weighted_bound * 2**127 * 10**38 < 2**2446
    assert D0 * 10**38 * 1000 * 2**127 < 2**1427
    assert 2**2446 + 2**127 < 2**2447
    # Independent unreduced Decimal expressions scale both components before
    # cross-multiplication. 3750 bits = at most1129 decimal digits, below1536.
    assert len(str(2**3750)) < 1536
    COUNTS.clear()
    WIDTHS.clear()
    corpus = Corpus()
    populate(corpus)
    source = Path(__file__).read_bytes()
    records = "".join(corpus.records).encode()
    body = "\t".join(COLUMNS).encode() + b"\n" + records
    headers = [
        ("schema", "1"), ("fixture-set", "EXACT-SETTLEMENT-PHASE-C-ORACLE"), ("domain", "settlement"),
        ("description", "Independent exact settlement recipes, closed images, stages and provenance"),
        ("reference", "Python stdlib integer/Fraction/Decimal; no Rust or QuantLib output"),
        ("reference-version", "Python >=3.9; Decimal precisions1536,2048"),
        ("reference-source", "Frozen settlement contract and FR-SETTLE-01..16; independent exact rational expressions"),
        ("reproduction", "python3 -I validation/generator/settlement_oracles.py --check"),
        ("generated-at", generated_at), ("platform", platform_text),
        ("generator-sha256", sha(source)), ("generator-fnv64", fnv(source)),
        ("adopted-contract-sha256", sha(contract)), ("adopted-contract-fnv64", fnv(contract)),
        ("requirements-rows-sha256", sha(requirements)), ("requirements-rows-fnv64", fnv(requirements)),
        ("tolerance", "TOL-E0; exact canonical amounts, bits, identifier outputs and structured errors; no epsilon"),
        ("encoding", "tab-separated; decimal=C:S; evidence=d:C:S,b:16hex,di:C:S:C:S,bi:16hex:16hex; -=unused; x:UTF8hex,_=empty; context=comma-separatedkey=value"),
        ("expected-encoding", "determined/ambiguous/exact_unresolved/evidence/identifier/stableerror; lower,upper are op-specific; determined guarantee retained; errors use all structured fields"),
        ("source-scope", "Conditional declared-enclosure or supplied-input arithmetic; no ideal-price or named-market certificate"),
        ("record-count", str(len(corpus.records))), ("unique-input-count", str(len(corpus.unique))),
        ("profile-counts", counter_text(corpus.profiles)), ("operation-counts", counter_text(corpus.ops)),
        ("expectation-counts", counter_text(corpus.expected)), ("recipe-counts", counter_text(corpus.recipes)),
        ("stage-context-counts", counter_text(corpus.stages)), ("guarantee-counts", counter_text(corpus.guarantees)),
        ("arithmetic-cause-counts", counter_text(corpus.causes)), ("error-endpoint-counts", counter_text(corpus.endpoints)),
        ("amount-operation-counts", counter_text(corpus.amount_operations)), ("cash-mode-counts", counter_text(corpus.cash_modes)),
        ("cross-check-counts", counter_text(COUNTS)), ("maximum-width-bits", counter_text(WIDTHS)),
        ("records-sha256", sha(records)), ("records-fnv64", fnv(records)),
        ("body-sha256", sha(body)), ("body-fnv64", fnv(body)),
    ]
    data = "".join("%s: %s\n" % item for item in headers).encode() + b"\n" + body
    return data, dict(headers)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Regenerate and compare every byte")
    parser.add_argument("--output", type=Path, default=DEST)
    args = parser.parse_args()
    generated_at = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    platform_text = platform.system() + " " + platform.machine()
    if args.check and args.output.is_file():
        header = args.output.read_text().split("\n\n", 1)[0]
        recorded = dict(line.split(": ", 1) for line in header.splitlines())
        generated_at, platform_text = recorded["generated-at"], recorded["platform"]
    data, summary = generate(generated_at, platform_text)
    if args.check:
        if not args.output.is_file() or args.output.read_bytes() != data:
            raise SystemExit("settlement fixture differs from independent regeneration")
        summary["check"] = "identical"
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(data)
        summary["written"] = str(args.output)
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
