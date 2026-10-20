#!/usr/bin/env python3
"""Independent US Treasury source/date/cash oracle corpus; Python stdlib only.

Authored from the adopted local Treasury main and cash contracts (pinned by
SHA-256 below, commit a1b66f9) before any Treasury Rust exists. No Rust
implementation, QuantLib implementation or observed library output is read
or imported. Every expectation is derived from the contract text:

Initial producer: Claude peer sabiquant-a2. Codex completed/reviewed this
provisional generator after its separate blind 38-test source/cash hand
suite was frozen; those hand expectations were not changed after this read.
See docs/M8_US_TREASURY_SOURCE_CASH_ORACLES.md for correction provenance.

* route A, authoritative: Fraction arithmetic of main E and cash T3/T5 with
  integer quotient/remainder rounding (remainder compared with d - r) and
  pure Gregorian month lengths;
* route B, corroborating: each row's serialized input text is decoded again,
  the direct cycle is rebuilt with the calendar module, accrued pieces are
  re-derived as overlaps of [start, S] with quasi periods, and every value,
  publication count, view and cash decision is recomputed from main E's
  X/(10^dscale*s_part) forms and cash T9's integer ratios by exact Decimal
  integer divmod (sign-magnitude, remainder compared with den - r) under
  contexts of 1536 and 2048 digits with Inexact and Rounded trapped, so no
  boundary decision depends on precision; binary64 views are rounded from
  integers without float(). Both contexts must equal route A on every
  compared column of every row.

Rows are public-API operations: instrument construction, plan preparation,
the two instrument coupon views, cash factory plus evaluate, and the
instruction source view. Outputs are create-new only; --check regenerates
with the frozen file's own generated-at and platform for byte equality.
"""

import argparse
import calendar
from collections import Counter
from dataclasses import dataclass, replace
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal, Inexact, localcontext, Rounded
from fractions import Fraction as F
import hashlib
import json
from math import gcd
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/bonds/treasury-source-cash-oracle.fixture"
MAIN = ROOT / "docs/M8_US_TREASURY_CONTRACT.md"
CASH = ROOT / "docs/M8_US_TREASURY_CASH_CONTRACT.md"
REQUIREMENTS = ROOT / "docs/REQUIREMENTS.md"
TEST_DESIGN = ROOT / "docs/M8_US_TREASURY_TEST_DESIGN.md"
SOURCE_CHECKER = ROOT / "validation/generator/treasury_source_example_check.py"
CONTRACT_COMMIT = "a1b66f9f90459e2b5b47013149f51251adb2d359"
MAIN_SHA = "50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93"
CASH_SHA = "25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585"
REQUIREMENT_ROWS_SHA = "722e090018bff9bbe25a2574f778766c06068c832dee6db9fe90041e1bcafeb3"
TEST_DESIGN_SHA = "5e91934b538d03297659674cc9f10f51f91b1d35dd21437472a7901d157e0629"
SOURCE_XML_SHA = "35fa6cb3febd8807c06b08e73d1f311a472b31f127796c6ef369237915b6699d"
SOURCE_CHECKER_SHA = "04f268d313b82d1c77421903adc855ebb4ada5f3c25beeaf6db22dd4ef718e8d"
SOURCE_VERSION = "US-Treasury-31CFR356-AppB-fixed-2026-10-16-local-v1"
I128_MIN, I128_MAX = -(2 ** 127), 2 ** 127 - 1
MAX_DATES = MAX_BUNDLES = 20000
R0 = 2 ** 1074 * 5 ** 38
PRECISIONS = (1536, 2048)
MODES = ("TowardZero", "AwayFromZero", "TowardPositiveInfinity", "TowardNegativeInfinity",
         "NearestTiesEven", "NearestTiesAwayFromZero", "Exact")
BASES = ("ExactRatio", "NineDecimal")
TIES = ("NearestTiesEven", "NearestTiesAwayFromZero")
POLICIES = tuple((b, t) for b in BASES for t in TIES)
RECIPE_ID = {"coupon": "SourceCouponPer1000", "accrued": "PublishedAccruedPer1000",
             "invoice": "CleanPricePer100AndPublishedAccruedPer1000", "dirty": "DirtyPricePer100"}
AGGREGATIONS = ("RoundTotal", "RoundComponentsThenSum")
INPUT_COLUMNS = ("op dated issue maturity coupon first quasi eom date_cap basis tie settlement "
                 "bundle_cap coupon_index recipe nominal evidence aggregation quantum mode "
                 "view_quantum view_mode").split()
OUTPUT_COLUMNS = ("expected context first_period first_quasi coupon_count raw_first source_first "
                  "next_quasi r s n bundles entitlements raw_ai raw_ai_bits pub_ai pub_ai_bits "
                  "pieces recipe_id src_index src_date settle_pair day_ratios dated_pair "
                  "source_value lower upper guarantee").split()
ANNOTATION_COLUMNS = ["identity_group"]
COLUMNS = ["id", "profile"] + INPUT_COLUMNS + OUTPUT_COLUMNS + ANNOTATION_COLUMNS
ERROR_KINDS = tuple("treasury." + k for k in (
    "invalid_date_cap too_many_quasi_dates too_few_quasi_dates negative_coupon dated_after_issue "
    "issue_not_before_maturity maturity_boundary quasi_date_order quasi_coverage "
    "quasi_date_mismatch date first_coupon invalid_bundle_cap too_many_bundles "
    "settlement_before_issue settlement_at_or_after_maturity invalid_coupon_index "
    "nonpositive_nominal cash_arithmetic").split())
STAGES = ("raw coupon", "source coupon", "source accrued", "cash coupon", "cash accrued",
          "cash capital", "cash total", "cash net")
PROFILE_FAMILIES = {
    "CONSTRUCTOR-ORDER": "UST-04,UST-09", "DIRECT-CYCLE-DATE-ERROR": "UST-04,UST-06,UST-08",
    "DIRECT-CYCLE-EOM": "UST-06,UST-07", "TABLE1-DAY-COUNTS": "UST-07",
    "DATE-EXTREMES": "UST-08,UST-10,UST-11", "ROLE-SHAPES": "UST-05,UST-09",
    "PLAN-LIFECYCLE": "UST-10,UST-11,UST-12,UST-19", "PLAN-ORDER": "UST-11",
    "SOURCE-EXAMPLES": "UST-01,UST-38", "SOURCE-ANCHORS": "UST-01,UST-17,UST-59,UST-61",
    "TABLE2-DAILY": "UST-13", "GENERAL-C-DAILY-TIES": "UST-14", "STAGING-GRID": "UST-15,UST-20",
    "TOTAL-FIRST-AI": "UST-20", "COUPON-VIEWS": "UST-17,UST-18", "COUPON-CASH": "UST-59,UST-60",
    "INSTRUCTION-VIEW-IDENTITY": "UST-18,UST-70", "COUPON-39-SCALES": "UST-16,UST-68",
    "CANONICAL-EQUIVALENCE": "UST-16,UST-70", "VIEW-RANGE": "UST-16,UST-18",
    "ZERO-COUPON": "UST-12,UST-16,UST-60", "TINY-C-DAILY-ZERO": "UST-20",
    "CASH-PRECEDENCE": "UST-60,UST-72", "FRACTIONAL-NOMINAL": "UST-61",
    "AI-VIEW-PUBLISHED": "UST-61,UST-63", "CASH-COUPON-GRID": "UST-59,UST-68",
    "CASH-ACCRUED-GRID": "UST-61,UST-68", "CASH-39-SCALES": "UST-68",
    "PRICE-EVIDENCE": "UST-62,UST-63,UST-64,UST-69", "PRICE-MODE-SWEEP": "UST-64,UST-65,UST-68",
    "EVIDENCE-39-SCALES": "UST-68", "UNIVERSAL-EXACT": "UST-65",
    "CAPITAL-BEFORE-ACCRUED": "UST-66", "WIDE-CANCELLATION": "UST-66",
    "COLLAPSE-LABELS": "UST-67", "SIGNED-I128-LIMITS": "UST-68,UST-69",
    "IDENTITY": "UST-03,UST-70", "ZERO-AI-PRICE": "UST-19,UST-62,UST-63",
}
WIDTH_BOUNDS = {  # cash T9 / main E operand upper bounds (bits); asserted per operand
    "daily-k": 160, "publication-count": 147, "source-x": 161, "coupon-cash-num": 382,
    "coupon-cash-den": 397, "accrued-cash-num": 374, "accrued-cash-den": 280,
    "coupon-view-num": 265, "coupon-view-den": 265, "ai-view-num": 254, "ai-view-den": 254,
    "price-endpoint": 2187, "invoice-signed-sum": 2190, "price-num": 2433, "price-den": 1426}
WIDTHS = Counter()
CHECKS = Counter()
STAGING = Counter()
# Route B never performs a finite-precision rational division. All Decimal
# arithmetic is integer-exact, including canonicalization and range tests.
# A binary64's exact Decimal coefficient is at most (2^53-1)*5^1074;
# multiplying by the integer R0 has at most the sum of their decimal digits.
# Every other Decimal operand is bounded by the cash T9 integer widths, plus
# one decimal digit for an increment. These bounds do not rely on samples.
DECIMAL_CAPACITY = {
    "binary-coefficient-digits": len(str((2 ** 53 - 1) * 5 ** 1074)),
    "r0-digits": len(str(R0)),
    "integer-operation-digits": len(str(2 ** max(WIDTH_BOUNDS.values()) - 1)) + 1,
}
DECIMAL_CAPACITY["binary-r0-product-digits"] = (
    DECIMAL_CAPACITY["binary-coefficient-digits"] + DECIMAL_CAPACITY["r0-digits"])
DECIMAL_CAPACITY["required-precision"] = max(DECIMAL_CAPACITY.values())
assert min(PRECISIONS) >= DECIMAL_CAPACITY["required-precision"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    h = 14695981039346656037
    for b in data:
        h = ((h ^ b) * 1099511628211) & (2 ** 64 - 1)
    return "%016x" % h


def hx(text):
    return "x:" + text.encode().hex()


def canon(coef, scale=0):
    if coef == 0:
        return 0, 0
    while scale and coef % 10 == 0:
        coef //= 10
        scale -= 1
    return coef, scale


def cell(pair):
    return "%d:%d" % pair


def ratio_text(num, den):
    g = gcd(num, den)
    return "%d/%d" % (num // g, den // g)


def fcell(x):
    x = F(x)
    return "%d/%d" % (x.numerator, x.denominator)


class Failure(Exception):
    def __init__(self, kind, **context):
        super().__init__(kind)
        self.kind, self.context = kind, context


def render(value):
    if value is None:
        return "none"
    if isinstance(value, date):
        return value.isoformat()
    return str(value)


def context_cell(context):
    return ",".join("%s=%s" % (k, render(v)) for k, v in sorted(context.items())) or "-"


def inexact(stage, endpoint="point"):
    return Failure("treasury.cash_arithmetic", cause="amount.inexact", endpoint=endpoint,
                   operation="quantize", stage=hx(stage))


def admit(pair, stage, endpoint, operation):
    coef, scale = canon(*pair)
    if not I128_MIN <= coef <= I128_MAX:
        raise Failure("treasury.cash_arithmetic", cause="amount.out_of_range", endpoint=endpoint,
                      negative=str(coef < 0).lower(), operation=operation, stage=hx(stage))
    return cell((coef, scale))


def finish(lower, upper, point, stage, operation):
    """Gilt C5 admission: lower (or point) first, then upper for a nonpoint image."""
    low = admit(lower, stage, "point" if point else "lower", operation)
    if point:
        return "determined", low, low
    high = admit(upper, stage, "upper", operation)
    return ("determined", low, high) if low == high else ("ambiguous", low, high)


# --------------------------------------------------------------------------
# Route A: dates (direct maturity-anchored cycle, main E5) and exact Fraction values.

class DateRange(Exception):
    pass


def month_length(year, month):
    """Pure Gregorian month length; never constructs a date outside 0001..9999."""
    if month == 2:
        return 29 if year % 4 == 0 and (year % 100 != 0 or year % 400 == 0) else 28
    return 30 if month in (4, 6, 9, 11) else 31


def is_month_end(d):
    return d.day == month_length(d.year, d.month)


def add_months(anchor, months, month_end):
    year, month = divmod(anchor.year * 12 + anchor.month - 1 + months, 12)
    month += 1
    if not 1 <= year <= 9999:
        raise DateRange()
    last = month_length(year, month)
    return date(year, month, last if month_end else min(anchor.day, last))


def direct_cycle(maturity, length, eom):
    preserve = eom == "Preserve" and is_month_end(maturity)
    return tuple(add_months(maturity, -6 * (length - 1 - i), preserve) for i in range(length))


def days(a, b):
    return (b - a).days


def round_count(x, mode):
    """Signed seven-mode count of an exact rational; Exact gives None when inexact."""
    x = F(x)
    d = x.denominator
    q, r = divmod(x.numerator, d)
    if r == 0:
        return q
    if mode == "Exact":
        return None
    if mode == "TowardNegativeInfinity":
        return q
    if mode == "TowardPositiveInfinity":
        return q + 1
    if mode == "TowardZero":
        return q + 1 if x < 0 else q
    if mode == "AwayFromZero":
        return q if x < 0 else q + 1
    complement = d - r
    if r < complement:
        return q
    if r > complement:
        return q + 1
    if mode == "NearestTiesEven":
        return q if q % 2 == 0 else q + 1
    return q + 1 if x > 0 else q


def is_tie(x):
    x = F(x)
    return 2 * (x.numerator % x.denominator) == x.denominator


def publish(x, places, tie):
    """Nonnegative nearest publication under the retained tie rule (never Exact)."""
    assert x >= 0
    return F(round_count(x * 10 ** places, tie), 10 ** places)


def canonical_value(v):
    v = F(v)
    scale = 0
    while (v * 10 ** scale).denominator != 1:
        scale += 1
        assert scale <= 80
    return canon(int(v * 10 ** scale), scale)


@dataclass(frozen=True)
class Definition:
    dated: date
    issue: date
    maturity: date
    coupon: tuple
    first: date
    quasi: tuple
    eom: str
    cap: int
    basis: str
    tie: str


@dataclass(frozen=True)
class Model:
    role: str
    f: int
    count: int
    s0: int
    d0: int
    c: F


@dataclass(frozen=True)
class Plan:
    k: int
    r: int
    s: int
    n: int
    first_bundle: int
    ceased: int
    pieces: tuple
    raw_ai: F
    pub_1000: F


def validate(defn):
    q, n = defn.quasi, len(defn.quasi)
    if not 2 <= defn.cap <= MAX_DATES:
        raise Failure("treasury.invalid_date_cap", cap=defn.cap, minimum=2, maximum=MAX_DATES)
    if n > defn.cap:
        raise Failure("treasury.too_many_quasi_dates", count=n, cap=defn.cap)
    if n < 2:
        raise Failure("treasury.too_few_quasi_dates", count=n, minimum=2)
    c = F(defn.coupon[0], 10 ** defn.coupon[1])
    if c < 0:
        raise Failure("treasury.negative_coupon", coupon_per_100=cell(canon(*defn.coupon)))
    if defn.dated > defn.issue:
        raise Failure("treasury.dated_after_issue", original_dated_date=defn.dated,
                      original_issue_date=defn.issue)
    if defn.issue >= defn.maturity:
        raise Failure("treasury.issue_not_before_maturity", original_issue_date=defn.issue,
                      maturity_date=defn.maturity)
    if q[-1] != defn.maturity:
        raise Failure("treasury.maturity_boundary", maturity_date=defn.maturity,
                      last_quasi_date=q[-1])
    for i in range(1, n):
        if q[i] <= q[i - 1]:
            raise Failure("treasury.quasi_date_order", index=i, previous=q[i - 1], date=q[i])
    if not q[0] <= defn.dated < q[1]:
        raise Failure("treasury.quasi_coverage", original_dated_date=defn.dated,
                      first_quasi_date=q[0], next_quasi_date=q[1])
    preserve = defn.eom == "Preserve" and is_month_end(defn.maturity)
    for i in range(n):
        try:
            expected = add_months(defn.maturity, -6 * (n - 1 - i), preserve)
        except DateRange:
            # Outer error only: main E5 does not pin the nested DateError primitive.
            raise Failure("treasury.date", cause="DateError", index=i, phase=hx("quasi cycle"))
        if q[i] != expected:
            raise Failure("treasury.quasi_date_mismatch", index=i, supplied=q[i], expected=expected,
                          maturity_date=defn.maturity, end_of_month=defn.eom)
    if defn.issue >= defn.first or not (
            (defn.dated == q[0] and defn.first == q[1])
            or (q[0] < defn.dated < q[1] and defn.first == q[1])
            or (q[0] < defn.dated < q[1] and n > 2 and defn.first == q[2])):
        raise Failure("treasury.first_coupon", original_dated_date=defn.dated,
                      original_issue_date=defn.issue, first_coupon_date=defn.first,
                      next_quasi_date=q[1], second_quasi_date=q[2] if n > 2 else None)
    role, f = (("Regular", 1) if defn.dated == q[0] else
               ("Short", 1) if defn.first == q[1] else ("Long", 2))
    return Model(role, f, n - f, days(q[0], q[1]), days(defn.dated, q[1]), c)


def daily(defn, model, s):
    """Selected daily amount per $1,000 for half-year s (cash T3)."""
    ratio = 5 * model.c / s
    return ratio if defn.basis == "ExactRatio" else publish(ratio, 9, defn.tie)


def raw_coupon(model, i):
    half = model.c / 2
    if i == 0 and model.role == "Short":
        return half * F(model.d0, model.s0)
    if i == 0 and model.role == "Long":
        return half * (F(model.d0, model.s0) + 1)
    return half


def source_coupon_1000(defn, model, i):
    if i == 0 and model.role == "Short":
        return daily(defn, model, model.s0) * model.d0
    if i == 0 and model.role == "Long":
        return daily(defn, model, model.s0) * model.d0 + 5 * model.c
    return 5 * model.c


def prepare(defn, model, settlement, cap):
    q = defn.quasi
    if cap > MAX_BUNDLES:
        raise Failure("treasury.invalid_bundle_cap", cap=cap, maximum=MAX_BUNDLES)
    if settlement < defn.issue:
        raise Failure("treasury.settlement_before_issue", settlement_date=settlement,
                      original_issue_date=defn.issue)
    if settlement >= defn.maturity:
        raise Failure("treasury.settlement_at_or_after_maturity", settlement_date=settlement,
                      maturity_date=defn.maturity)
    k = next(i for i, d in enumerate(q) if d > settlement)
    first_bundle = max(k, model.f)
    count = len(q) - first_bundle
    if count > cap:
        raise Failure("treasury.too_many_bundles", count=count, cap=cap)
    f = model.f
    if k > f or (model.role == "Regular" and k == f):
        pieces = ((days(q[k - 1], settlement), days(q[k - 1], q[k])),)
    elif model.role == "Short" or k < f:
        pieces = ((days(defn.dated, settlement), model.s0),)
    else:
        pieces = ((model.d0, model.s0), (days(q[1], settlement), days(q[1], q[2])))
    raw = sum((model.c / 2 * F(t, s) for t, s in pieces), F(0))
    staged = sum((daily(defn, model, s) * t for t, s in pieces), F(0))
    return Plan(k, days(settlement, q[k]), days(q[k - 1], q[k]), len(q) - 1 - k, first_bundle,
                max(0, k - f), pieces, raw, publish(staged, 5, defn.tie))


def binary_bits(x):
    return "%016x" % struct.unpack(">Q", struct.pack(">d", float(F(x))))[0]


def decode_evidence_a(text):
    """Route A: binary64 decoded from the bit fields; decimals as exact fractions."""
    kind, *parts = text.split(":")
    if kind in ("d", "di"):
        ends = [F(int(parts[i]), 10 ** int(parts[i + 1])) for i in range(0, len(parts), 2)]
    else:
        ends = []
        for h in parts:
            bits = int(h, 16)
            sign = -1 if bits >> 63 else 1
            exponent, mantissa = (bits >> 52) & 0x7FF, bits & ((1 << 52) - 1)
            assert exponent != 0x7FF
            significand = mantissa if exponent == 0 else mantissa | (1 << 52)
            ends.append(sign * F(significand) * F(2) ** (max(exponent, 1) - 1075))
    assert ends[0] <= ends[-1]
    return (ends[0], ends[0]) if len(ends) == 1 else (ends[0], ends[1])


def guarantee(evidence):
    return {"d": "ForSuppliedDecimals", "b": "ForSuppliedBinary64",
            "di": "UniqueForDeclaredEnclosure", "bi": "UniqueForDeclaredEnclosure"}[
        evidence.split(":")[0]]


def quantize_a(value, quantum, mode, stage):
    count = round_count(F(value) / quantum, mode)
    if count is None:
        raise inexact(stage)
    return canonical_value(count * quantum)


def image_a(lo, hi, quantum, mode, stage):
    if lo == hi:
        pair = quantize_a(lo, quantum, mode, stage)
        return pair, pair, True
    if mode == "Exact":
        return None
    return (canonical_value(round_count(lo / quantum, mode) * quantum),
            canonical_value(round_count(hi / quantum, mode) * quantum), False)


def pair_value(pair):
    return F(pair[0], 10 ** pair[1])


def cash_a(recipe, nominal, quantum, mode, source_1000, evidence, aggregation):
    if recipe in ("coupon", "accrued"):
        stage = "cash coupon" if recipe == "coupon" else "cash accrued"
        pair = quantize_a(nominal * source_1000 / 1000, quantum, mode, stage)
        return finish(pair, pair, True, stage, "quantize")
    lo, hi = decode_evidence_a(evidence)
    accrued = nominal * source_1000 / 1000
    if recipe == "dirty" or aggregation == "RoundTotal":
        extra = 0 if recipe == "dirty" else accrued
        image = image_a(nominal * lo / 100 + extra, nominal * hi / 100 + extra, quantum, mode,
                        "cash total")
        if image is None:
            return "exact_unresolved", "cash total"
        return finish(image[0], image[1], image[2], "cash total", "quantize")
    capital = image_a(nominal * lo / 100, nominal * hi / 100, quantum, mode, "cash capital")
    if capital is None:
        return "exact_unresolved", "cash capital"
    acc = pair_value(quantize_a(accrued, quantum, mode, "cash accrued"))
    net_lo, net_hi = pair_value(capital[0]) + acc, pair_value(capital[1]) + acc
    return finish(canonical_value(net_lo), canonical_value(net_hi), net_lo == net_hi, "cash net",
                  "sum")


def route_a(inp):
    """Authoritative evaluation; returns (columns, state for route B)."""
    defn, op = inp["defn"], inp["op"]
    out, state = {}, {"reached": None}
    try:
        model = validate(defn)
        state.update(model=model, reached="instrument")
        out.update(first_period=model.role, first_quasi=str(model.f), coupon_count=str(model.count),
                   raw_first=fcell(raw_coupon(model, 0)),
                   source_first=fcell(source_coupon_1000(defn, model, 0) / 10))
        plan = None
        if inp["settlement"] is not None:
            plan = prepare(defn, model, inp["settlement"], inp["bundle_cap"])
            state.update(plan=plan, reached="plan")
            out.update(next_quasi=str(plan.k), r=str(plan.r), s=str(plan.s), n=str(plan.n),
                       bundles="%d..%d" % (plan.first_bundle, len(defn.quasi) - 1),
                       entitlements="%d/%d" % (plan.ceased, model.count),
                       raw_ai=fcell(plan.raw_ai), raw_ai_bits=binary_bits(plan.raw_ai),
                       pub_ai=fcell(plan.pub_1000 / 10), pub_ai_bits=binary_bits(plan.pub_1000 / 10),
                       pieces=",".join("%d/%d" % p for p in plan.pieces))
        if op in ("instrument", "plan"):
            out["expected"] = "ok"
            return out, state
        index = inp["coupon_index"]
        if op in ("raw_coupon_view", "source_coupon_view") or inp["recipe"] == "coupon":
            if not 0 <= index < model.count:
                raise Failure("treasury.invalid_coupon_index", index=index, count=model.count)
        if op in ("raw_coupon_view", "source_coupon_view"):
            state["reached"] = "arith"
            stage = "raw coupon" if op == "raw_coupon_view" else "source coupon"
            value = (raw_coupon(model, index) if op == "raw_coupon_view"
                     else source_coupon_1000(defn, model, index) / 10)
            pair = quantize_a(value, pair_value(inp["view_quantum"]), inp["view_mode"], stage)
            result = finish(pair, pair, True, stage, "quantize")
            out.update(expected="some", lower=result[1], upper=result[2])
            return out, state
        recipe, nominal = inp["recipe"], pair_value(inp["nominal"])
        if nominal <= 0:
            raise Failure("treasury.nonpositive_nominal", nominal=cell(canon(*inp["nominal"])))
        q = defn.quasi
        out.update(recipe_id=RECIPE_ID[recipe], dated_pair="%s,%s" % (q[0], q[1]))
        if recipe == "coupon":
            source = source_coupon_1000(defn, model, index)
            out.update(src_index=str(index), src_date=q[model.f + index].isoformat(),
                       settle_pair="none", source_value=fcell(source / 10),
                       day_ratios=("%d/%d,none" % (model.d0, model.s0)
                                   if index == 0 and model.role != "Regular" else "none"))
        else:
            source = plan.pub_1000
            out["settle_pair"] = "%s,%s" % (q[plan.k - 1], q[plan.k])
            if recipe == "dirty":
                out.update(src_index="none", src_date="none", day_ratios="none",
                           source_value="none")
            else:
                ratios = ["%d/%d" % p for p in plan.pieces] + ["none"]
                out.update(src_index=str(plan.ceased), src_date=q[plan.first_bundle].isoformat(),
                           day_ratios=",".join(ratios[:2]), source_value=fcell(source / 10))
        state["reached"] = "arith"
        if op == "instruction_view":
            if recipe == "dirty":
                out["expected"] = "none"
                return out, state
            stage = "source coupon" if recipe == "coupon" else "source accrued"
            pair = quantize_a(source / 10, pair_value(inp["view_quantum"]), inp["view_mode"], stage)
            result = finish(pair, pair, True, stage, "quantize")
            out.update(expected="some", lower=result[1], upper=result[2])
            return out, state
        decision = cash_a(recipe, nominal, pair_value(inp["quantum"]), inp["mode"], source,
                          inp["evidence"], inp["aggregation"])
        if decision[0] == "exact_unresolved":
            out.update(expected="exact_unresolved", context=context_cell({"stage": hx(decision[1])}))
        else:
            out.update(expected=decision[0], lower=decision[1], upper=decision[2])
            if decision[0] == "determined":
                out["guarantee"] = (guarantee(inp["evidence"]) if recipe in ("invoice", "dirty")
                                    else "none")
        return out, state
    except Failure as error:
        out.update(expected=error.kind, context=context_cell(error.context))
        return out, state


# --------------------------------------------------------------------------
# Route B: separately decoded text, calendar-module cycle, overlap pieces and
# main E / cash T9 integer forms with Decimal division.

def widen(name, value):
    bits = abs(value).bit_length()
    WIDTHS[name] = max(WIDTHS[name], bits)
    assert bits <= WIDTH_BOUNDS[name], (name, bits)


def b_count(num, den, mode, path=None):
    """Quantum count of num/den from exact Decimal integer division.

    Sign-magnitude, truncating Decimal divmod with Inexact/Rounded trapped, so
    every boundary decision compares the exact remainder r with den - r and no
    result depends on the context precision (which bounds capacity only).
    Exact gives None when the ratio is not an integer.
    """
    assert den > 0
    if path:
        widen(path + "-num", num)
        widen(path + "-den", den)
    negative = num < 0
    quotient, remainder = divmod(Decimal(abs(num)), Decimal(den))
    if remainder == 0:
        magnitude = quotient
    elif mode == "Exact":
        return None
    else:
        complement = Decimal(den) - remainder
        up = {"TowardZero": False, "AwayFromZero": True,
              "TowardPositiveInfinity": not negative, "TowardNegativeInfinity": negative,
              }.get(mode)
        if up is None:
            if remainder != complement:
                up = remainder > complement
            elif mode == "NearestTiesEven":
                up = quotient % 2 == 1
            else:
                up = True
        magnitude = quotient + 1 if up else quotient
    return -int(magnitude) if negative else int(magnitude)


def b_binary64(num, den):
    """Binary64 bits of the nonnegative ratio num/den, round-to-nearest-even, from
    integer arithmetic only (independent of float() and Fraction)."""
    assert num >= 0 and den > 0
    if num == 0:
        return "%016x" % 0
    exponent = num.bit_length() - den.bit_length()
    while True:
        shift = 52 - max(exponent, -1022)
        scaled_num, scaled_den = (num << shift, den) if shift >= 0 else (num, den << -shift)
        mantissa, remainder = divmod(scaled_num, scaled_den)
        if exponent > -1022 and mantissa < 1 << 52:
            exponent -= 1
        elif mantissa >= 1 << 53:
            exponent += 1
        else:
            break
    complement = scaled_den - remainder
    if remainder > complement or (remainder == complement and remainder and mantissa % 2):
        mantissa += 1
    if mantissa == 1 << 53:
        mantissa, exponent = 1 << 52, exponent + 1
    assert exponent <= 1023
    if mantissa < 1 << 52:
        return "%016x" % mantissa
    return "%016x" % (((max(exponent, -1022) + 1023) << 52) | (mantissa - (1 << 52)))


def b_pair(text):
    coef, scale = text.split(":")
    return b_canonical(int(coef), int(scale))


def b_canonical(coefficient, scale):
    """Independent Decimal-integer trailing-zero reduction; no route A canon call."""
    negative = coefficient < 0
    magnitude = Decimal(abs(coefficient))
    if magnitude == 0:
        return 0, 0
    while scale:
        quotient, remainder = divmod(magnitude, Decimal(10))
        if remainder:
            break
        magnitude, scale = quotient, scale - 1
    return (-int(magnitude) if negative else int(magnitude)), scale


def b_admit(pair, stage, endpoint, operation):
    coefficient, scale = b_canonical(*pair)
    if Decimal(coefficient) < Decimal(I128_MIN) or Decimal(coefficient) > Decimal(I128_MAX):
        raise Failure("treasury.cash_arithmetic", cause="amount.out_of_range", endpoint=endpoint,
                      negative=str(coefficient < 0).lower(), operation=operation, stage=hx(stage))
    return "%d:%d" % (coefficient, scale)


def b_finish(lower, upper, point, stage, operation):
    low = b_admit(lower, stage, "point" if point else "lower", operation)
    if point:
        return "determined", low, low
    high = b_admit(upper, stage, "upper", operation)
    return ("determined", low, high) if low == high else ("ambiguous", low, high)


def b_scaled(num, den, exponent):
    """num*10^E/den as an unreduced integer ratio, never dividing early."""
    return num * 10 ** max(exponent, 0), den * 10 ** max(-exponent, 0)


def b_cycle(maturity, length, eom):
    last = calendar.monthrange(maturity.year, maturity.month)[1]
    month_end = eom == "Preserve" and maturity.day == last
    out = []
    for i in range(length):
        year, month0 = divmod(maturity.year * 12 + maturity.month - 1 - 6 * (length - 1 - i), 12)
        size = calendar.monthrange(year, month0 + 1)[1]
        out.append(date(year, month0 + 1, size if month_end else min(maturity.day, size)))
    return out


def b_days(a, b):
    return b.toordinal() - a.toordinal()


def b_daily_k(ccoef, cscale, s, tie):
    num, den = 5 * ccoef * 10 ** max(9 - cscale, 0), s * 10 ** max(cscale - 9, 0)
    k = b_count(num, den, tie)
    widen("daily-k", k)
    return k


def b_form(role, index, basis, tie, ccoef, cscale, d0, s0):
    """Main E single normative table: X, dscale, s_part."""
    if index > 0 or role == "Regular":
        form = (5 * ccoef, cscale, 1)
    elif basis == "ExactRatio":
        form = (5 * ccoef * (d0 if role == "Short" else d0 + s0), cscale, s0)
    else:
        k = b_daily_k(ccoef, cscale, s0, tie)
        if role == "Short":
            form = (k * d0, 9, 1)
        else:
            m = max(9, cscale)
            form = (k * d0 * 10 ** (m - 9) + 5 * ccoef * 10 ** (m - cscale), m, 1)
    widen("source-x", form[0])
    return form


def b_quantize(num, den, mode, quantum, stage, path):
    count = b_count(num, den, mode, path)
    if count is None:
        raise inexact(stage)
    return count * quantum[0], quantum[1]


def b_evidence(text):
    """Route B: endpoints decoded onto R0 via Decimal(float) and integer scaling."""
    kind, *parts = text.split(":")
    ends = []
    if kind in ("d", "di"):
        for i in range(0, len(parts), 2):
            ends.append(int(parts[i]) * (R0 // 10 ** int(parts[i + 1])))
    else:
        for h in parts:
            value = Decimal(struct.unpack(">d", bytes.fromhex(h))[0]) * R0
            assert value == value.to_integral_value()
            ends.append(int(value))
    for b in ends:
        widen("price-endpoint", b)
    return (ends[0], ends[0]) if len(ends) == 1 else (ends[0], ends[1])


def b_image(lo, hi, nominal, quantum, mode, stage, extra, exponent, invoice=False):
    """Closed image counts of R0 endpoint integers: (lo, hi, point), or None if unresolved."""
    n = nominal[0]
    def count(b):
        total = 10 * b + extra if invoice else b
        if invoice:
            widen("invoice-signed-sum", total)
        num, den = b_scaled(n * total, quantum[0] * R0, exponent)
        return b_count(num, den, mode, "price")
    if lo == hi:
        value = count(lo)
        if value is None:
            raise inexact(stage)
        return value, value, True
    if mode == "Exact":
        return None
    return count(lo), count(hi), False


def route_b(t, state, precision):
    out = {}
    if state.get("model") is None:
        return out
    with localcontext() as context:
        context.prec = precision
        context.Emin, context.Emax = -999999, 999999
        context.traps[Inexact] = context.traps[Rounded] = True
        dated, maturity = date.fromisoformat(t["dated"]), date.fromisoformat(t["maturity"])
        first = date.fromisoformat(t["first"])
        quasi = [date.fromisoformat(d) for d in t["quasi"].split(",")]
        assert b_cycle(maturity, len(quasi), t["eom"]) == quasi
        CHECKS["route-b-calendar-cycle"] += 1
        ccoef, cscale = b_pair(t["coupon"])
        basis, tie = t["basis"], t["tie"]
        role = "Regular" if dated == quasi[0] else "Short" if first == quasi[1] else "Long"
        f = 2 if role == "Long" else 1
        assert role == state["model"].role
        d0, s0 = b_days(dated, quasi[1]), b_days(quasi[0], quasi[1])
        x, dscale, s_part = b_form(role, 0, "ExactRatio", tie, ccoef, cscale, d0, s0)
        out["raw_first"] = ratio_text(x, 10 ** (dscale + 1) * s_part)
        x, dscale, s_part = b_form(role, 0, basis, tie, ccoef, cscale, d0, s0)
        out["source_first"] = ratio_text(x, 10 ** (dscale + 1) * s_part)
        plan_count = None
        if state.get("plan") is not None:
            settle = date.fromisoformat(t["settlement"])
            paid = [d for d in quasi[f:] if d <= settle]
            start = paid[-1] if paid else dated
            pieces = [(b_days(max(start, a), min(settle, b)), b_days(a, b))
                      for a, b in zip(quasi, quasi[1:]) if b > start and a < settle]
            pieces = [p for p in pieces if p[0] > 0]
            assert sorted(pieces) == sorted(p for p in state["plan"].pieces if p[0] > 0)
            CHECKS["route-b-overlap-pieces"] += 1
            product = 1
            for _, s in pieces:
                product *= s
            weighted = sum(tt * (product // s) for tt, s in pieces)
            raw_num, raw_den = ccoef * weighted, 2 * 10 ** cscale * product
            out["raw_ai"] = ratio_text(raw_num, raw_den)
            out["raw_ai_bits"] = b_binary64(raw_num, raw_den)
            if basis == "ExactRatio":
                num, den = b_scaled(5 * ccoef * weighted, product, 5 - cscale)
                plan_count = b_count(num, den, tie)
            else:
                plan_count = b_count(sum(b_daily_k(ccoef, cscale, s, tie) * tt
                                         for tt, s in pieces), 10 ** 4, tie)
            widen("publication-count", plan_count)
            out["pub_ai"] = ratio_text(plan_count, 10 ** 6)
            out["pub_ai_bits"] = b_binary64(plan_count, 10 ** 6)
        if state["reached"] != "arith":
            return out
        op, recipe = t["op"], t["recipe"]
        try:
            if op in ("raw_coupon_view", "source_coupon_view"):
                use = "ExactRatio" if op == "raw_coupon_view" else basis
                x, dscale, s_part = b_form(role, int(t["coupon_index"]), use, tie, ccoef, cscale,
                                           d0, s0)
                quantum = b_pair(t["view_quantum"])
                num, den = b_scaled(x, s_part * quantum[0], quantum[1] - 1 - dscale)
                stage = "raw coupon" if op == "raw_coupon_view" else "source coupon"
                value = b_admit(b_quantize(num, den, t["view_mode"], quantum, stage, "coupon-view"),
                                stage, "point", "quantize")
                out.update(expected="some", lower=value, upper=value)
                return out
            if recipe == "coupon":
                x, dscale, s_part = b_form(role, int(t["coupon_index"]), basis, tie, ccoef, cscale,
                                           d0, s0)
                out["source_value"] = ratio_text(x, 10 ** (dscale + 1) * s_part)
            elif recipe == "dirty":
                out["source_value"] = "none"
            else:
                out["source_value"] = ratio_text(plan_count, 10 ** 6)
            if op == "instruction_view":
                if recipe == "dirty":
                    out["expected"] = "none"
                    return out
                vq = b_pair(t["view_quantum"])
                if recipe == "coupon":
                    num, den = b_scaled(x, s_part * vq[0], vq[1] - 1 - dscale)
                    stage, path = "source coupon", "coupon-view"
                else:
                    num, den = b_scaled(plan_count, vq[0], vq[1] - 6)
                    stage, path = "source accrued", "ai-view"
                value = b_admit(b_quantize(num, den, t["view_mode"], vq, stage, path), stage,
                                "point", "quantize")
                out.update(expected="some", lower=value, upper=value)
                return out
            nominal, quantum, mode = b_pair(t["nominal"]), b_pair(t["quantum"]), t["mode"]
            n, nscale = nominal
            q, qscale = quantum
            if recipe == "coupon":
                num, den = b_scaled(n * x, s_part * q, qscale - nscale - 3 - dscale)
                value = b_admit(b_quantize(num, den, mode, quantum, "cash coupon", "coupon-cash"),
                                "cash coupon", "point", "quantize")
                decision = ("determined", value, value)
            elif recipe == "accrued":
                num, den = b_scaled(n * plan_count, q, qscale - nscale - 8)
                value = b_admit(b_quantize(num, den, mode, quantum, "cash accrued", "accrued-cash"),
                                "cash accrued", "point", "quantize")
                decision = ("determined", value, value)
            else:
                lo, hi = b_evidence(t["evidence"])
                if recipe == "dirty" or t["aggregation"] == "RoundTotal":
                    stage = "cash total"
                    extra = 0 if recipe == "dirty" else plan_count * (R0 // 10 ** 5)
                    exponent = qscale - nscale - (2 if recipe == "dirty" else 3)
                    image = b_image(lo, hi, nominal, quantum, mode, stage, extra, exponent,
                                    recipe == "invoice")
                    if image is None:
                        decision = ("exact_unresolved", stage)
                    else:
                        decision = b_finish((image[0] * q, qscale), (image[1] * q, qscale), image[2],
                                            stage, "quantize")
                else:
                    image = b_image(lo, hi, nominal, quantum, mode, "cash capital", 0,
                                    qscale - nscale - 2)
                    if image is None:
                        decision = ("exact_unresolved", "cash capital")
                    else:
                        num, den = b_scaled(n * plan_count, q, qscale - nscale - 8)
                        accrued = b_count(num, den, mode, "accrued-cash")
                        if accrued is None:
                            raise inexact("cash accrued")
                        low, high = image[0] + accrued, image[1] + accrued
                        decision = b_finish((low * q, qscale), (high * q, qscale), low == high,
                                            "cash net", "sum")
            if decision[0] == "exact_unresolved":
                out.update(expected="exact_unresolved",
                           context=context_cell({"stage": hx(decision[1])}))
            else:
                out.update(expected=decision[0], lower=decision[1], upper=decision[2])
        except Failure as error:
            out.update(expected=error.kind, context=context_cell(error.context))
        return out


# --------------------------------------------------------------------------
# Corpus assembly.

def opt_text(value):
    if value is None:
        return "-"
    if isinstance(value, tuple):
        return cell(value)
    return render(value)


class Corpus:
    def __init__(self):
        self.rows, self.inputs, self.seen = [], [], set()
        self.counters = {name: Counter() for name in (
            "profile", "op", "expected", "policy", "recipe", "cash-mode", "view-mode", "evidence",
            "stage", "endpoint", "cause", "operation", "guarantee", "first-period", "ai-pieces",
            "aggregation", "eom")}
        self.scales = {k: set() for k in ("coupon", "nominal", "quantum", "view-quantum",
                                          "decimal-evidence")}

    def add(self, profile, defn, op="plan", settlement=None, bundle_cap=MAX_BUNDLES,
            coupon_index=None, recipe=None, nominal=None, evidence=None, aggregation=None,
            quantum=None, mode=None, view_quantum=None, view_mode=None, group="-"):
        assert profile in PROFILE_FAMILIES, profile
        inp = dict(defn=defn, op=op, settlement=settlement, bundle_cap=bundle_cap,
                   coupon_index=coupon_index, recipe=recipe, nominal=nominal, evidence=evidence,
                   aggregation=aggregation, quantum=quantum, mode=mode, view_quantum=view_quantum,
                   view_mode=view_mode)
        text = dict(op=op, dated=render(defn.dated), issue=render(defn.issue),
                    maturity=render(defn.maturity), coupon=cell(defn.coupon),
                    first=render(defn.first),
                    quasi=",".join(d.isoformat() for d in defn.quasi) or "empty", eom=defn.eom,
                    date_cap=str(defn.cap), basis=defn.basis, tie=defn.tie,
                    settlement=opt_text(settlement),
                    bundle_cap=str(bundle_cap) if settlement is not None else "-",
                    coupon_index=opt_text(coupon_index), recipe=opt_text(recipe),
                    nominal=opt_text(nominal), evidence=opt_text(evidence),
                    aggregation=opt_text(aggregation), quantum=opt_text(quantum),
                    mode=opt_text(mode), view_quantum=opt_text(view_quantum),
                    view_mode=opt_text(view_mode))
        key = tuple(text[k] for k in INPUT_COLUMNS)
        if key in self.seen:
            CHECKS["duplicate-input-skipped"] += 1
            return None
        self.seen.add(key)
        out, state = route_a(inp)
        for precision in PRECISIONS:
            other = route_b(text, state, precision)
            for column, value in other.items():
                assert out.get(column, "-") == value, (profile, column, out.get(column), value, key)
            CHECKS["route-b-decimal%d-rows" % precision] += 1
            CHECKS["route-b-decimal%d-cells" % precision] += len(other)
        row = dict.fromkeys(COLUMNS, "-")
        row.update(text)
        row.update(out)
        row.update(id="TSC-%06d" % (len(self.rows) + 1), profile=profile, identity_group=group)
        self.count(row, defn, inp)
        values = [row[k] for k in COLUMNS]
        assert all(v and "\t" not in v and "\n" not in v for v in values), row
        self.rows.append("\t".join(values) + "\n")
        self.inputs.append("\t".join(key) + "\n")
        return row

    def count(self, row, defn, inp):
        c = self.counters
        c["profile"][row["profile"]] += 1
        c["op"][row["op"]] += 1
        c["expected"][row["expected"]] += 1
        c["policy"]["%s/%s" % (row["basis"], row["tie"])] += 1
        c["eom"][row["eom"]] += 1
        for name, column in (("recipe", "recipe"), ("cash-mode", "mode"),
                             ("view-mode", "view_mode"), ("aggregation", "aggregation"),
                             ("guarantee", "guarantee"), ("first-period", "first_period")):
            if row[column] != "-":
                c[name][row[column]] += 1
        if row["evidence"] != "-":
            kind, *parts = row["evidence"].split(":")
            c["evidence"][kind] += 1
            if kind in ("d", "di"):
                self.scales["decimal-evidence"].update(int(s) for s in parts[1::2])
        if row["context"] != "-":
            context = dict(kv.split("=", 1) for kv in row["context"].split(","))
            if "stage" in context:
                c["stage"][bytes.fromhex(context["stage"][2:]).decode()] += 1
            if row["expected"] == "treasury.cash_arithmetic":
                for key in ("endpoint", "cause", "operation"):
                    c[key][context[key]] += 1
        if row["pieces"] != "-":
            c["ai-pieces"]["%d-piece" % (row["pieces"].count(",") + 1)] += 1
        self.scales["coupon"].add(defn.coupon[1])
        for name, key in (("nominal", "nominal"), ("quantum", "quantum"),
                          ("view-quantum", "view_quantum")):
            if inp[key] is not None:
                self.scales[name].add(inp[key][1])


# --------------------------------------------------------------------------
# Instrument catalogue. Every Q is the explicit direct cycle unless a row
# deliberately supplies a malformed vector.

D = date.fromisoformat


def instrument(maturity, length, dated, issue, first_index, coupon=(6, 0), eom="Ignore",
               basis="NineDecimal", tie="NearestTiesEven", cap=MAX_DATES):
    quasi = direct_cycle(maturity, length, eom)
    return Definition(dated, issue, maturity, coupon, quasi[first_index], quasi, eom, cap, basis,
                      tie)


def with_policy(defn, basis, tie):
    return replace(defn, basis=basis, tie=tie)


def base_shapes():
    m, ml = D("2026-02-15"), D("2025-08-15")
    return {
        "regular-d-eq-i": instrument(m, 5, D("2024-02-15"), D("2024-02-15"), 1),
        "regular-d-lt-i": instrument(m, 5, D("2024-02-15"), D("2024-02-29"), 1),
        "regular-single": instrument(D("2024-08-15"), 2, D("2024-02-15"), D("2024-02-15"), 1),
        "short-d-eq-i": instrument(m, 5, D("2024-03-01"), D("2024-03-01"), 1),
        "short-d-lt-i": instrument(m, 5, D("2024-03-01"), D("2024-03-10"), 1),
        "long-d-eq-i": instrument(ml, 5, D("2023-12-01"), D("2023-12-01"), 2),
        "long-d-lt-i": instrument(ml, 5, D("2023-12-01"), D("2024-01-10"), 2),
        "long-i-after-q1": instrument(ml, 5, D("2023-12-01"), D("2024-03-01"), 2),
        "long-single": instrument(D("2024-08-15"), 3, D("2023-09-01"), D("2023-09-01"), 2),
        "eom-preserve-apr30": instrument(D("2026-04-30"), 5, D("2024-04-30"), D("2024-04-30"), 1,
                                         eom="Preserve"),
        "eom-preserve-feb29": instrument(D("2028-02-29"), 4, D("2026-08-31"), D("2026-08-31"), 1,
                                         eom="Preserve"),
        "eom-ignore-apr30": instrument(D("2026-04-30"), 5, D("2024-04-30"), D("2024-04-30"), 1),
        "eom-preserve-long": instrument(D("2026-03-31"), 5, D("2024-05-15"), D("2024-05-15"), 2,
                                        eom="Preserve"),
    }


def settlements_for(defn):
    """Lifecycle settlements: I, D, every quasi boundary and its neighbours, M-1."""
    points = {defn.issue, defn.issue + timedelta(days=1), defn.maturity - timedelta(days=1),
              defn.dated}
    for d in defn.quasi:
        for delta in (-1, 0, 1):
            points.add(d + timedelta(days=delta))
    return sorted(p for p in points if defn.issue <= p < defn.maturity)


def cycle_from(maturity, first_quasi, eom="Ignore"):
    length = 1
    while direct_cycle(maturity, length, eom)[0] > first_quasi:
        length += 1
    assert direct_cycle(maturity, length, eom)[0] == first_quasi, (maturity, first_quasi)
    return length


def source_examples():
    """Appendix B II.A-G and I.A/I.D statement instruments; maturities absent from the
    source statement are labelled local in the oracle document."""
    def make(maturity, q0, dated, issue, first_index, coupon, eom="Ignore"):
        return instrument(maturity, cycle_from(maturity, q0, eom), dated, issue, first_index,
                          coupon=coupon, eom=eom)
    return {
        "II.A": (make(D("2020-05-15"), D("1990-05-15"), D("1990-05-15"), D("1990-05-15"), 1, (875, 2)), D("1990-05-15")),
        "II.B": (make(D("1992-03-31"), D("1990-03-31"), D("1990-04-02"), D("1990-04-02"), 1, (85, 1), "Preserve"), D("1990-04-02")),
        "II.C": (make(D("1995-05-15"), D("1989-11-15"), D("1990-03-01"), D("1990-03-01"), 2, (85, 1)), D("1990-03-01")),
        "II.D": (make(D("1995-11-15"), D("1985-11-15"), D("1985-11-15"), D("1985-11-29"), 1, (95, 1)), D("1985-11-29")),
        "II.E": (make(D("2005-08-15"), D("1985-02-15"), D("1985-07-02"), D("1985-07-02"), 2, (1075, 2)), D("1985-11-04")),
        "II.F": (make(D("1991-05-15"), D("1983-05-15"), D("1983-05-16"), D("1983-05-16"), 1, (105, 1)), D("1983-08-15")),
        "II.G": (make(D("1994-12-15"), D("1988-06-15"), D("1988-10-15"), D("1988-10-15"), 2, (975, 2)), D("1988-11-15")),
        "I.A.3": (make(D("1992-06-30"), D("1990-06-30"), D("1990-07-02"), D("1990-07-02"), 1, (8375, 3), "Preserve"), None),
        "I.A.4": (make(D("1996-02-15"), D("1990-08-15"), D("1990-12-03"), D("1990-12-03"), 2, (7875, 3)), None),
        "I.D(1)": (make(D("2005-05-15"), D("2000-05-15"), D("2000-05-15"), D("2000-05-15"), 1, (675, 2)), D("2000-08-15")),
    }


def grid_instrument(s, coupon, basis, tie, one_day_short=False):
    q0, q1 = {181: (D("2023-02-15"), D("2023-08-15")), 182: (D("2024-02-15"), D("2024-08-15")),
              183: (D("2024-04-15"), D("2024-10-15")), 184: (D("2024-05-15"), D("2024-11-15"))}[s]
    assert days(q0, q1) == s
    dated = q1 - timedelta(days=1) if one_day_short else q0
    return Definition(dated, dated, q1, coupon, q1, (q0, q1), "Ignore", MAX_DATES, basis, tie)


QUANTA = ((1, 0), (1, 2), (5, 2), (1, 6), (1, 9), (1, 10), (1, 38), (I128_MAX, 0), (I128_MAX, 38),
          (3, 7), (7, 3), (25, 4), (1, 5), (1, 1), (10 ** 38, 0), (2, 0))
NOMINALS = ((1, 0), (1000, 0), (20000, 0), (499600, 0), (1, 38), (I128_MAX, 0), (I128_MAX, 38),
            (12345, 2), (1, 3), (999, 0), (150000, 0), (7, 1))
EVIDENCE = (
    "d:0:0", "d:100:0", "d:-11:1", "d:9973125:5", "d:%d:0" % I128_MIN, "d:%d:0" % I128_MAX,
    "d:1:38", "d:-1:38", "di:-1006:3:-1004:3", "di:99:0:101:0", "di:5:1:5:1", "di:-1:0:1:0",
    "b:0000000000000000", "b:8000000000000000", "b:0000000000000001", "b:7fefffffffffffff",
    "b:ffefffffffffffff", "b:3ff0000000000000", "b:4058e00000000000", "b:4630000000000000",
    "bi:4630000000000000:4630000000000001", "bi:8000000000000000:0000000000000000",
    "bi:0000000000000001:0000000000000002", "bi:ffefffffffffffff:7fefffffffffffff",
    "bi:4058e00000000000:4058e00000000000", "b:c7b0000000000000", "b:0010000000000000",
    "b:000fffffffffffff")


class Rotor:
    def __init__(self):
        self.i = 0

    def pick(self, values):
        self.i += 1
        return values[self.i % len(values)]


def expect(row, expected):
    assert row is not None and row["expected"] == expected, (
        row and row["expected"], expected, row and row["context"])
    return row


def populate(c):
    rot = Rotor()
    shapes = base_shapes()
    for defn in shapes.values():
        expect(c.add("ROLE-SHAPES", defn, op="instrument"), "ok")

    # Constructor order and mixed-invalid precedence (main E1-E6).
    good = shapes["regular-d-eq-i"]
    q = good.quasi
    def mut(**changes):
        return replace(good, **changes)
    order = [
        (mut(cap=0, coupon=(-1, 0), quasi=()), "invalid_date_cap"),
        (mut(cap=1, coupon=(-1, 0), quasi=()), "invalid_date_cap"),
        (mut(cap=20001, coupon=(-1, 0), quasi=()), "invalid_date_cap"),
        (mut(cap=4, coupon=(-1, 0)), "too_many_quasi_dates"),
        (mut(quasi=(q[-1],), coupon=(-1, 0)), "too_few_quasi_dates"),
        (mut(quasi=(), coupon=(-1, 0), cap=2), "too_few_quasi_dates"),
        (mut(coupon=(-1, 0), dated=D("2024-03-01"), issue=D("2024-02-20")), "negative_coupon"),
        (mut(coupon=(-1, 38)), "negative_coupon"),
        (mut(coupon=(-10, 1)), "negative_coupon"),
        (mut(coupon=(0, 7)), "ok"),
        (mut(dated=D("2024-03-01"), issue=D("2024-02-20"), maturity=D("2023-01-01")),
         "dated_after_issue"),
        (mut(issue=good.maturity), "issue_not_before_maturity"),
        (mut(issue=good.maturity + timedelta(days=1), maturity=D("2026-02-14")),
         "issue_not_before_maturity"),
        (mut(maturity=D("2026-02-16")), "maturity_boundary"),
        (mut(quasi=q[:2] + (q[1],) + q[3:]), "quasi_date_order"),
        (mut(quasi=q[:3] + (q[2] - timedelta(days=1),) + q[4:]), "quasi_date_order"),
        (mut(quasi=(q[1], q[0]) + q[2:], dated=q[1], issue=q[1]), "quasi_date_order"),
        (mut(dated=q[0] - timedelta(days=1), issue=q[0]), "quasi_coverage"),
        (mut(dated=q[1], issue=q[1]), "quasi_coverage"),
        (mut(quasi=(q[0] - timedelta(days=184),) + q), "quasi_coverage"),
        (mut(quasi=(q[0] + timedelta(days=1),) + q[1:], dated=q[0] + timedelta(days=1),
             issue=q[0] + timedelta(days=1)), "quasi_date_mismatch"),
        (mut(quasi=q[:2] + (q[2] + timedelta(days=1),) + q[3:]), "quasi_date_mismatch"),
        (mut(quasi=q[:2] + (q[2] + timedelta(days=1),) + q[3:], issue=q[1]),
         "quasi_date_mismatch"),
        (mut(issue=q[1]), "first_coupon"),
        (mut(issue=q[1] + timedelta(days=3)), "first_coupon"),
        (mut(first=q[2]), "first_coupon"),
        (mut(first=q[1] + timedelta(days=1)), "first_coupon"),
        (mut(first=q[0]), "first_coupon"),
        (mut(first=q[4]), "first_coupon"),
        (mut(dated=D("2024-03-01"), issue=D("2024-03-01"), first=q[3]), "first_coupon"),
        (mut(dated=D("2024-03-01"), issue=D("2024-08-15"), first=q[2]), "ok"),
        (mut(dated=D("2024-03-01"), issue=D("2024-08-15"), first=q[1]), "first_coupon"),
        (mut(quasi=(D("2025-08-15"), D("2026-02-15")), dated=D("2025-08-20"),
             issue=D("2025-08-20"), first=D("2026-08-15")), "first_coupon"),
        (instrument(D("2026-02-15"), 3, D("2025-02-15"), D("2025-02-15"), 1, cap=2),
         "too_many_quasi_dates"),
        (instrument(D("2026-02-15"), 3, D("2025-02-15"), D("2025-02-15"), 1, cap=3), "ok"),
    ]
    for defn, outcome in order:
        expect(c.add("CONSTRUCTOR-ORDER", defn, op="instrument"),
               outcome if outcome == "ok" else "treasury." + outcome)

    # Direct-cycle date arithmetic failure precedes mismatch (main E5).
    for eom, quasi, maturity, first in (
            ("Ignore", (D("0001-01-15"), D("0001-04-15"), D("0001-07-15")), D("0001-07-15"),
             D("0001-04-15")),
            ("Preserve", (D("0001-01-31"), D("0001-03-31"), D("0001-06-30")), D("0001-06-30"),
             D("0001-03-31")),
            ("Ignore", (D("0001-01-31"), D("0001-03-31"), D("0001-06-30")), D("0001-06-30"),
             D("0001-03-31"))):
        defn = Definition(quasi[0], quasi[0], maturity, (6, 0), first, quasi, eom, MAX_DATES,
                          "NineDecimal", "NearestTiesEven")
        expect(c.add("DIRECT-CYCLE-DATE-ERROR", defn, op="instrument"), "treasury.date")

    def custom(quasi, eom, first_index=1, coupon=(4, 0)):
        return Definition(quasi[0], quasi[0], quasi[-1], coupon, quasi[first_index], quasi, eom,
                          MAX_DATES, "NineDecimal", "NearestTiesEven")
    expect(c.add("DIRECT-CYCLE-DATE-ERROR", custom(direct_cycle(D("0002-01-01"), 3, "Ignore"),
                                                   "Ignore"), op="instrument"), "ok")

    # Month-end policy (UST-06/07): official Treasury Q needs Preserve; Ignore rejects it.
    official = (D("2024-04-30"), D("2024-10-31"), D("2025-04-30"), D("2025-10-31"), D("2026-04-30"))
    recursive = (D("2023-08-29"), D("2024-02-29"), D("2024-08-31"))
    monthend = (D("2023-08-31"), D("2024-02-29"), D("2024-08-31"))
    clamped = (D("2024-03-30"), D("2024-09-30"), D("2025-03-30"), D("2025-09-30"))
    for quasi, preserve, ignore in ((official, "ok", "treasury.quasi_date_mismatch"),
                                    (recursive, "treasury.quasi_date_mismatch",
                                     "treasury.quasi_date_mismatch"),
                                    (monthend, "ok", "ok"),
                                    (clamped, "treasury.quasi_date_mismatch", "ok")):
        expect(c.add("DIRECT-CYCLE-EOM", custom(quasi, "Preserve"), op="instrument"), preserve)
        expect(c.add("DIRECT-CYCLE-EOM", custom(quasi, "Ignore"), op="instrument"), ignore)
    ignore_official = direct_cycle(D("2026-04-30"), 5, "Ignore")
    expect(c.add("DIRECT-CYCLE-EOM", custom(ignore_official, "Ignore"), op="instrument"), "ok")
    expect(c.add("DIRECT-CYCLE-EOM", custom(ignore_official, "Preserve"), op="instrument"),
           "treasury.quasi_date_mismatch")

    # Table 1: 12 half-year rows x (1st/15th, last day) x (regular, leap) = 48 cells.
    names = calendar.month_abbr
    for first_month in range(1, 13):
        last_month = (first_month + 5) % 12 + 1
        for leap in (False, True):
            february = 2024 if leap else 2023
            start_year = february - 1 if first_month >= 8 else february
            end_year = start_year + (1 if last_month < first_month else 0)
            label = "table1:%s-%s:%s" % (names[first_month], names[last_month],
                                         "leap" if leap else "regular")
            for day in (1, 15):
                quasi = (date(start_year, first_month, day), date(end_year, last_month, day))
                expect(c.add("TABLE1-DAY-COUNTS", custom(quasi, "Ignore"), settlement=quasi[0],
                             group=label + ":day%d" % day), "ok")
            quasi = (date(start_year, first_month, month_length(start_year, first_month)),
                     date(end_year, last_month, month_length(end_year, last_month)))
            expect(c.add("TABLE1-DAY-COUNTS", custom(quasi, "Preserve"), settlement=quasi[0],
                         group=label + ":last-day"), "ok")

    # Date extremes, full supported span and bundle caps (UST-08/10/11).
    late = custom(direct_cycle(D("9999-12-31"), 3, "Preserve"), "Preserve")
    expect(c.add("DATE-EXTREMES", late, op="instrument"), "ok")
    expect(c.add("DATE-EXTREMES", late, settlement=D("9999-12-30")), "ok")
    expect(c.add("DATE-EXTREMES", custom((D("0001-01-01"), D("0001-07-01")), "Ignore"),
                 settlement=D("0001-01-01")), "ok")
    span = direct_cycle(D("9999-07-01"), 19998, "Ignore")
    assert span[0] == D("0001-01-01")
    full = custom(span, "Ignore")
    expect(c.add("DATE-EXTREMES", full, op="instrument"), "ok")
    expect(c.add("DATE-EXTREMES", replace(full, cap=19998), op="instrument"), "ok")
    expect(c.add("DATE-EXTREMES", replace(full, cap=19997), op="instrument"),
           "treasury.too_many_quasi_dates")
    expect(c.add("DATE-EXTREMES", full, settlement=span[0], bundle_cap=19997), "ok")
    expect(c.add("DATE-EXTREMES", full, settlement=span[0], bundle_cap=19996),
           "treasury.too_many_bundles")
    expect(c.add("DATE-EXTREMES", full, settlement=D("9999-06-30"), bundle_cap=1), "ok")
    expect(c.add("DATE-EXTREMES", full, op="cash", coupon_index=19996, recipe="coupon",
                 nominal=(1000, 0), quantum=(1, 2), mode="Exact"), "determined")
    expect(c.add("DATE-EXTREMES", full, op="cash", coupon_index=19997, recipe="coupon",
                 nominal=(1000, 0), quantum=(1, 2), mode="Exact"),
           "treasury.invalid_coupon_index")
    sixty = custom(direct_cycle(D("2054-05-15"), 61, "Ignore"), "Ignore")
    settle = sixty.quasi[30] + timedelta(days=7)
    expect(c.add("DATE-EXTREMES", sixty, settlement=settle, bundle_cap=30), "ok")
    expect(c.add("DATE-EXTREMES", sixty, settlement=settle, bundle_cap=29),
           "treasury.too_many_bundles")

    # Every shape and policy: full lifecycle, D=I/D<I, skipped long Q1, reset at F.
    for base in shapes.values():
        for basis, tie in POLICIES:
            defn = with_policy(base, basis, tie)
            c.add("ROLE-SHAPES", defn, op="instrument")
            for s in settlements_for(defn):
                expect(c.add("PLAN-LIFECYCLE", defn, settlement=s), "ok")

    # Plan failure order (main E plan step; UST-11).
    defn = shapes["long-d-lt-i"]
    for settle, cap, outcome in (
            (defn.issue - timedelta(days=9), 20001, "invalid_bundle_cap"),
            (defn.maturity, 20001, "invalid_bundle_cap"),
            (defn.issue - timedelta(days=1), 0, "settlement_before_issue"),
            (defn.maturity, 0, "settlement_at_or_after_maturity"),
            (defn.maturity + timedelta(days=1), 0, "settlement_at_or_after_maturity"),
            (defn.issue, 0, "too_many_bundles"), (defn.issue, 2, "too_many_bundles"),
            (defn.issue, 3, "ok"), (defn.maturity - timedelta(days=1), 1, "ok"),
            (defn.maturity - timedelta(days=1), 0, "too_many_bundles")):
        expect(c.add("PLAN-ORDER", defn, settlement=settle, bundle_cap=cap),
               outcome if outcome == "ok" else "treasury." + outcome)

    # Appendix B statements and printed anchors under every policy.
    examples = source_examples()
    for name, (defn, settle) in examples.items():
        for basis, tie in POLICIES:
            d = with_policy(defn, basis, tie)
            expect(c.add("SOURCE-EXAMPLES", d, op="instrument", group="source:" + name), "ok")
            if settle is not None:
                expect(c.add("SOURCE-EXAMPLES", d, settlement=settle, group="source:" + name), "ok")
                c.add("SOURCE-EXAMPLES", d, op="instruction_view", settlement=settle,
                      recipe="accrued", nominal=(1000, 0), quantum=(1, 2), mode="NearestTiesEven",
                      view_quantum=(1, 6), view_mode="Exact", group="source:" + name)
    for basis, tie in POLICIES:
        ia3 = with_policy(examples["I.A.3"][0], basis, tie)
        for quantum, mode in (((1, 2), "NearestTiesEven"), ((1, 2), "NearestTiesAwayFromZero"),
                              ((1, 8), "Exact"), ((1, 2), "Exact"), ((1, 9), "Exact")):
            c.add("SOURCE-ANCHORS", ia3, op="cash", coupon_index=0, recipe="coupon",
                  nominal=(20000, 0), quantum=quantum, mode=mode, group="anchor:I.A.3")
        for view in ("source_coupon_view", "raw_coupon_view"):
            c.add("SOURCE-ANCHORS", ia3, op=view, coupon_index=0, view_quantum=(1, 10),
                  view_mode="Exact", group="anchor:I.A.3")
        ia4 = with_policy(examples["I.A.4"][0], basis, tie)
        for quantum, mode in (((1, 2), "NearestTiesEven"), ((1, 8), "Exact"),
                              ((1, 2), "TowardZero")):
            c.add("SOURCE-ANCHORS", ia4, op="cash", coupon_index=0, recipe="coupon",
                  nominal=(7000, 0), quantum=quantum, mode=mode, group="anchor:I.A.4")
        c.add("SOURCE-ANCHORS", ia4, op="source_coupon_view", coupon_index=0,
              view_quantum=(1, 10), view_mode="Exact", group="anchor:I.A.4")
        id1 = with_policy(examples["I.D(1)"][0], basis, tie)
        for quantum, mode in (((1, 2), "Exact"), ((1, 2), "NearestTiesEven")):
            c.add("SOURCE-ANCHORS", id1, op="cash", settlement=examples["I.D(1)"][1],
                  recipe="accrued", nominal=(150000, 0), quantum=quantum, mode=mode,
                  group="anchor:I.D(1)")
        id2 = with_policy(examples["II.E"][0], basis, tie)
        for quantum, mode in (((1, 2), "NearestTiesEven"), ((1, 5), "Exact"),
                              ((1, 2), "TowardNegativeInfinity")):
            c.add("SOURCE-ANCHORS", id2, op="cash", settlement=examples["II.E"][1],
                  recipe="accrued", nominal=(11000, 0), quantum=quantum, mode=mode,
                  group="anchor:I.D(2)")

    # Table 2: all 640 daily cells via a one-day short first coupon (UST-13).
    for eighths in range(1, 161):
        coupon = canonical_value(F(eighths, 8))
        for s in (184, 183, 182, 181):
            for basis, tie in POLICIES:
                c.add("TABLE2-DAILY", grid_instrument(s, coupon, basis, tie, True),
                      op="source_coupon_view", coupon_index=0, view_quantum=(1, 10),
                      view_mode="Exact", group="table2:%d/8:%d" % (eighths, s))
    # General-C nine-place daily ties at each half-year length (UST-14).
    for s, coupon in ((181, (181, 10)), (182, (91, 9)), (183, (183, 10)), (184, (23, 7))):
        assert is_tie(5 * F(coupon[0], 10 ** coupon[1]) / s * 10 ** 9)
        for basis, tie in POLICIES:
            defn = grid_instrument(s, coupon, basis, tie, True)
            c.add("GENERAL-C-DAILY-TIES", defn, op="source_coupon_view", coupon_index=0,
                  view_quantum=(1, 10), view_mode="Exact")
            c.add("GENERAL-C-DAILY-TIES", defn, op="cash", coupon_index=0, recipe="coupon",
                  nominal=(10 ** 9, 0), quantum=(1, 9), mode="Exact")
            longer = grid_instrument(s, coupon, basis, tie)
            settle = longer.quasi[0] + timedelta(days=1)
            c.add("GENERAL-C-DAILY-TIES", longer, settlement=settle)
            c.add("GENERAL-C-DAILY-TIES", longer, op="instruction_view", settlement=settle,
                  recipe="accrued", nominal=(1, 0), quantum=(1, 2), mode="Exact",
                  view_quantum=(1, 6), view_mode="Exact")

    # Staging grid: every publicly observable discriminating cell plus one per (C, s).
    for eighths, s, t in staging_cells():
        coupon = canonical_value(F(eighths, 8))
        for basis, tie in POLICIES:
            defn = grid_instrument(s, coupon, basis, tie)
            c.add("STAGING-GRID", defn, op="instruction_view",
                  settlement=defn.quasi[0] + timedelta(days=t), recipe="accrued",
                  nominal=(1000, 0), quantum=(1, 2), mode="NearestTiesEven",
                  view_quantum=(1, 6), view_mode="Exact", group="grid:%d/8:%d:%d" % (eighths, s, t))
    for defn, settle in total_first_cells():
        for basis, tie in POLICIES:
            c.add("TOTAL-FIRST-AI", with_policy(defn, basis, tie), op="instruction_view",
                  settlement=settle, recipe="accrued", nominal=(1000, 0), quantum=(1, 2),
                  mode="NearestTiesEven", view_quantum=(1, 6), view_mode="Exact")

    # Coupon views, coupon cash and the instruction-view identity on every shape.
    for name, base in shapes.items():
        for basis, tie in POLICIES:
            defn = with_policy(base, basis, tie)
            count = validate(defn).count
            for index in sorted({0, 1, count - 1, count, count + 5}):
                for view in ("raw_coupon_view", "source_coupon_view"):
                    c.add("COUPON-VIEWS", defn, op=view, coupon_index=index,
                          view_quantum=rot.pick(QUANTA), view_mode=rot.pick(MODES))
                    c.add("COUPON-VIEWS", defn, op=view, coupon_index=index,
                          view_quantum=(1, 10), view_mode="Exact")
                c.add("COUPON-CASH", defn, op="cash", coupon_index=index, recipe="coupon",
                      nominal=rot.pick(NOMINALS), quantum=rot.pick(QUANTA), mode=rot.pick(MODES))
                for vq, vm in (((1, 10), "Exact"), (rot.pick(QUANTA), rot.pick(MODES))):
                    group = "view-identity:%s:%s/%s:%d:%s:%s" % (name, basis, tie, index,
                                                                 cell(vq), vm)
                    c.add("INSTRUCTION-VIEW-IDENTITY", defn, op="instruction_view",
                          coupon_index=index, recipe="coupon", nominal=(1000, 0), quantum=(1, 2),
                          mode="NearestTiesEven", view_quantum=vq, view_mode=vm, group=group)
                    c.add("INSTRUCTION-VIEW-IDENTITY", defn, op="source_coupon_view",
                          coupon_index=index, view_quantum=vq, view_mode=vm, group=group)

    # All 39 coupon scales, maximum coefficients and widest caches (UST-16/68).
    for scale in range(39):
        for coef in (1, 7, 123456789, I128_MAX):
            for shape in ("regular-d-eq-i", "short-d-lt-i", "long-i-after-q1", "long-single"):
                basis, tie = POLICIES[(scale + coef) % 4]
                defn = replace(with_policy(shapes[shape], basis, tie), coupon=(coef, scale))
                for view in ("raw_coupon_view", "source_coupon_view"):
                    for vq in ((1, 38), (1, scale), rot.pick(QUANTA)):
                        c.add("COUPON-39-SCALES", defn, op=view, coupon_index=0, view_quantum=vq,
                              view_mode=rot.pick(MODES))
                c.add("COUPON-39-SCALES", defn, op="cash", coupon_index=0, recipe="coupon",
                      nominal=rot.pick(NOMINALS), quantum=rot.pick(QUANTA), mode=rot.pick(MODES))
                late = (defn.quasi[-1] - timedelta(days=1) if shape == "long-single"
                        else defn.issue + timedelta(days=40))
                c.add("COUPON-39-SCALES", defn, op="cash", settlement=late, recipe="accrued",
                      nominal=rot.pick(NOMINALS), quantum=rot.pick(QUANTA), mode=rot.pick(MODES))
                c.add("COUPON-39-SCALES", defn, op="instruction_view", settlement=late,
                      recipe="accrued", nominal=(1, 0), quantum=(1, 2), mode="NearestTiesEven",
                      view_quantum=(1, 38), view_mode=rot.pick(MODES))
    for basis, tie in POLICIES:
        for coupon in ((6, 0), (60, 1), (6000, 3), (600000000000, 11)):
            c.add("CANONICAL-EQUIVALENCE", replace(with_policy(shapes["short-d-lt-i"], basis, tie),
                                                   coupon=coupon),
                  op="cash", coupon_index=0, recipe="coupon", nominal=(1000000, 3),
                  quantum=(100, 4), mode="Exact", group="canonical:%s/%s" % (basis, tie))
        for coupon in ((0, 0), (0, 38)):
            c.add("CANONICAL-EQUIVALENCE", replace(with_policy(shapes["long-d-lt-i"], basis, tie),
                                                   coupon=coupon),
                  op="instrument", group="canonical-zero:%s/%s" % (basis, tie))
        widest = replace(with_policy(shapes["long-single"], basis, tie), coupon=(I128_MAX, 0))
        for view in ("raw_coupon_view", "source_coupon_view"):
            for quantum in ((10 ** 38, 0), (I128_MAX, 0), (2, 0)):
                for mode in ("AwayFromZero", "TowardZero", "Exact"):
                    c.add("VIEW-RANGE", widest, op=view, coupon_index=0, view_quantum=quantum,
                          view_mode=mode)
        for quantum in ((10 ** 38, 0), (I128_MAX, 0)):
            c.add("VIEW-RANGE", widest, op="instruction_view",
                  settlement=widest.quasi[-1] - timedelta(days=1), recipe="accrued",
                  nominal=(1, 0), quantum=(1, 0), mode="TowardZero", view_quantum=quantum,
                  view_mode="AwayFromZero")
    zero = replace(shapes["long-d-lt-i"], coupon=(0, 0))
    for basis, tie in POLICIES:
        z = with_policy(zero, basis, tie)
        c.add("ZERO-COUPON", z, op="instrument")
        for s in settlements_for(z)[:8]:
            c.add("ZERO-COUPON", z, settlement=s)
        for index in (0, 2, 3):
            for nominal in ((0, 0), (1, 0)):
                c.add("ZERO-COUPON", z, op="cash", coupon_index=index, recipe="coupon",
                      nominal=nominal, quantum=(1, 2), mode="Exact")
        for nominal in ((-5, 1), (5, 1)):
            c.add("ZERO-COUPON", z, op="cash", settlement=z.issue, recipe="accrued",
                  nominal=nominal, quantum=(1, 2), mode="Exact")
    short = shapes["short-d-eq-i"]
    for basis, tie in POLICIES:
        t = replace(with_policy(shapes["regular-d-eq-i"], basis, tie), coupon=(1, 12))
        c.add("TINY-C-DAILY-ZERO", t, settlement=t.issue + timedelta(days=30))
        c.add("TINY-C-DAILY-ZERO", t, op="instruction_view", settlement=t.issue + timedelta(days=30),
              recipe="accrued", nominal=(1, 0), quantum=(1, 2), mode="NearestTiesEven",
              view_quantum=(1, 6), view_mode="Exact")
        tiny_short = replace(with_policy(short, basis, tie), coupon=(1, 12))
        for view in ("raw_coupon_view", "source_coupon_view"):
            c.add("TINY-C-DAILY-ZERO", tiny_short, op=view, coupon_index=0, view_quantum=(1, 38),
                  view_mode="Exact")

    # Cash factory precedence (cash T6): index before nominal; nominal before any zero shortcut.
    count = validate(short).count
    for nominal in ((0, 0), (-1, 0), (-1, 38), (I128_MIN, 0), (0, 38)):
        c.add("CASH-PRECEDENCE", short, op="cash", coupon_index=count, recipe="coupon",
              nominal=nominal, quantum=(1, 2), mode="Exact")
        c.add("CASH-PRECEDENCE", short, op="cash", coupon_index=0, recipe="coupon",
              nominal=nominal, quantum=(1, 2), mode="Exact")
        for recipe, ev, agg in (("accrued", None, None), ("dirty", "d:100:0", None),
                                ("invoice", "d:100:0", "RoundTotal")):
            for settle in (short.issue + timedelta(days=10), short.issue - timedelta(days=1)):
                c.add("CASH-PRECEDENCE", short, op="cash", settlement=settle, recipe=recipe,
                      nominal=nominal, evidence=ev, aggregation=agg, quantum=(1, 2), mode="Exact")
    c.add("CASH-PRECEDENCE", replace(short, coupon=(-1, 0)), op="cash", coupon_index=count,
          recipe="coupon", nominal=(0, 0), quantum=(1, 2), mode="Exact")

    # Fractional nominal discriminator (cash T1, UST-61): no second five-place stage.
    fractional = Definition(D("2025-07-15"), D("2025-07-15"), D("2026-01-15"), (368, 6),
                            D("2026-01-15"), (D("2025-07-15"), D("2026-01-15")), "Ignore",
                            MAX_DATES, "NineDecimal", "NearestTiesEven")
    for basis, tie in POLICIES:
        d = with_policy(fractional, basis, tie)
        for mode in MODES:
            row = c.add("FRACTIONAL-NOMINAL", d, op="cash", settlement=D("2025-07-16"),
                        recipe="accrued", nominal=(499600, 0), quantum=(1, 2), mode=mode,
                        group="discriminator:0.004996")
            if mode == "NearestTiesAwayFromZero":
                assert row["lower"] == "0:0" and row["source_value"] == "1/1000000", row
                CHECKS["fractional-nominal-cent0-not-0.01"] += 1
        for nominal in ((1, 0), (500, 0), (1001, 0), (12345678, 3), (1, 38), (4996, 1)):
            for quantum, mode in (((1, 8), "Exact"), ((1, 2), "NearestTiesAwayFromZero")):
                c.add("FRACTIONAL-NOMINAL", d, op="cash", settlement=D("2025-07-16"),
                      recipe="accrued", nominal=nominal, quantum=quantum, mode=mode)

    # Instruction views: published AI, never raw; dirty returns Ok(None) (UST-61/63).
    for basis, tie in POLICIES:
        d = with_policy(short, basis, tie)
        s = d.issue + timedelta(days=17)
        for vq, vm in (((1, 6), "Exact"), ((1, 7), "Exact"), ((1, 38), "Exact"), ((1, 5), "Exact"),
                       ((1, 5), "NearestTiesEven"), ((1, 0), "AwayFromZero")):
            c.add("AI-VIEW-PUBLISHED", d, op="instruction_view", settlement=s, recipe="accrued",
                  nominal=(1, 0), quantum=(1, 2), mode="Exact", view_quantum=vq, view_mode=vm)
            c.add("AI-VIEW-PUBLISHED", d, op="instruction_view", settlement=s, recipe="invoice",
                  evidence="d:99:0", aggregation="RoundTotal", nominal=(1, 0), quantum=(1, 2),
                  mode="Exact", view_quantum=vq, view_mode=vm)
            c.add("AI-VIEW-PUBLISHED", d, op="instruction_view", settlement=s, recipe="dirty",
                  evidence="d:99:0", nominal=(1, 0), quantum=(1, 2), mode="Exact",
                  view_quantum=vq, view_mode=vm)

    # Coupon and accrued cash grids: nominals, quanta and every mode.
    for name in ("regular-d-lt-i", "short-d-lt-i", "long-d-eq-i", "long-i-after-q1",
                 "eom-preserve-apr30"):
        for basis, tie in POLICIES:
            d = with_policy(shapes[name], basis, tie)
            count = validate(d).count
            for nominal in NOMINALS:
                for mode in MODES:
                    c.add("CASH-COUPON-GRID", d, op="cash",
                          coupon_index=rot.pick((0, 1, count - 1)), recipe="coupon",
                          nominal=nominal, quantum=rot.pick(QUANTA), mode=mode)
            for s in settlements_for(d)[::2]:
                for mode in MODES:
                    c.add("CASH-ACCRUED-GRID", d, op="cash", settlement=s, recipe="accrued",
                          nominal=rot.pick(NOMINALS), quantum=rot.pick(QUANTA), mode=mode)
    for scale in range(39):
        d = with_policy(shapes["long-d-eq-i"], *POLICIES[scale % 4])
        for nominal in ((1, scale), (I128_MAX, scale), (499600, scale)):
            for quantum in ((1, scale), (I128_MAX, scale)):
                c.add("CASH-39-SCALES", d, op="cash", coupon_index=0, recipe="coupon",
                      nominal=nominal, quantum=quantum, mode=MODES[scale % 7])
                c.add("CASH-39-SCALES", d, op="cash", settlement=d.issue + timedelta(days=75),
                      recipe="accrued", nominal=nominal, quantum=quantum,
                      mode=MODES[(scale + 3) % 7])

    # Zero accrued is an actual source state, not a switch to a different price
    # denominator. Explicit nonzero-price controls prevent truthiness of AI=0
    # from dropping the factor10 of the invoice's shared per1000 integer form.
    for name in ("regular-d-eq-i", "short-d-eq-i", "long-d-eq-i"):
        for basis, tie in POLICIES:
            base = with_policy(shapes[name], basis, tie)
            for coupon in (base.coupon, (0, 0)):
                d = replace(base, coupon=coupon)
                settlements = {d.issue, d.first}
                if coupon == (0, 0):
                    settlements.add(d.issue + timedelta(days=37))
                for settle in sorted(settlements):
                    assert settle < d.maturity
                    for evidence in ("d:100:0", "d:-125:2", "d:0:0"):
                        for recipe, agg in (("dirty", None), ("invoice", "RoundTotal"),
                                            ("invoice", "RoundComponentsThenSum")):
                            for nominal, quantum in (((1000, 0), (1, 2)), ((1, 0), (5, 2))):
                                for mode in MODES:
                                    row = c.add("ZERO-AI-PRICE", d, op="cash", settlement=settle,
                                                recipe=recipe, nominal=nominal, evidence=evidence,
                                                aggregation=agg, quantum=quantum, mode=mode)
                                    if row is not None:
                                        assert row["pub_ai"] == "0/1"
                                        if evidence in ("d:100:0", "d:0:0"):
                                            expected = cell(nominal) if evidence == "d:100:0" else "0:0"
                                            assert row["expected"] == "determined" and row["lower"] == expected
                                        elif nominal == (1000, 0):
                                            assert row["lower"] == "-125:1"
                                        CHECKS["zero-ai-price-control-rows"] += 1

    # Price recipes: every evidence kind, both aggregations, signed and extreme endpoints.
    contexts = [(shapes["regular-d-lt-i"], D("2024-05-01")), (shapes["long-d-lt-i"], D("2024-03-20")),
                (shapes["long-d-eq-i"], D("2024-02-15")), (short, D("2024-08-14"))]
    for defn, settle in contexts:
        for basis, tie in POLICIES:
            d = with_policy(defn, basis, tie)
            for evidence in EVIDENCE:
                nominal, quantum, mode = rot.pick(NOMINALS), rot.pick(QUANTA), rot.pick(MODES)
                c.add("PRICE-EVIDENCE", d, op="cash", settlement=settle, recipe="dirty",
                      nominal=nominal, evidence=evidence, quantum=quantum, mode=mode)
                for agg in AGGREGATIONS:
                    c.add("PRICE-EVIDENCE", d, op="cash", settlement=settle, recipe="invoice",
                          nominal=nominal, evidence=evidence, aggregation=agg, quantum=quantum,
                          mode=mode)
    sweep = with_policy(shapes["long-d-lt-i"], "NineDecimal", "NearestTiesAwayFromZero")
    for evidence in EVIDENCE:
        for mode in MODES:
            for nominal, quantum in (((1000, 0), (1, 2)), ((12345, 2), (5, 2))):
                c.add("PRICE-MODE-SWEEP", sweep, op="cash", settlement=D("2024-03-20"),
                      recipe="dirty", nominal=nominal, evidence=evidence, quantum=quantum,
                      mode=mode)
                for agg in AGGREGATIONS:
                    c.add("PRICE-MODE-SWEEP", sweep, op="cash", settlement=D("2024-03-20"),
                          recipe="invoice", nominal=nominal, evidence=evidence, aggregation=agg,
                          quantum=quantum, mode=mode)
    for scale in range(39):
        for evidence in ("d:1:%d" % scale, "d:%d:%d" % (I128_MAX, scale),
                         "d:%d:%d" % (I128_MIN, scale),
                         "di:%d:%d:%d:%d" % (-I128_MAX, scale, I128_MAX, scale)):
            for recipe, agg in (("dirty", None), ("invoice", "RoundTotal"),
                                ("invoice", "RoundComponentsThenSum")):
                c.add("EVIDENCE-39-SCALES", sweep, op="cash", settlement=D("2024-03-20"),
                      recipe=recipe, nominal=rot.pick(NOMINALS), evidence=evidence,
                      aggregation=agg, quantum=rot.pick(QUANTA), mode=rot.pick(MODES))

    # Universal Exact: point Inexact versus continuous unresolved (UST-65).
    d = shapes["regular-d-lt-i"]
    adjacent = "bi:4630000000000000:4630000000000001"
    for evidence in (adjacent, "di:1:0:2:0", "b:4630000000000000", "di:2:0:2:0"):
        c.add("UNIVERSAL-EXACT", d, op="cash", settlement=D("2024-05-01"), recipe="dirty",
              nominal=(100, 0), evidence=evidence, quantum=(1, 0), mode="Exact")
        for agg in AGGREGATIONS:
            c.add("UNIVERSAL-EXACT", d, op="cash", settlement=D("2024-05-01"), recipe="invoice",
                  nominal=(100, 0), evidence=evidence, aggregation=agg, quantum=(1, 0),
                  mode="Exact")
    for mode in MODES:
        c.add("UNIVERSAL-EXACT", d, op="cash", settlement=D("2024-05-01"), recipe="dirty",
              nominal=(100, 0), evidence=adjacent, quantum=(2 ** 48, 0), mode=mode)
    # Capital Exact before point accrued (UST-66).
    for evidence in ("d:1234567:5", "di:1234567:5:1234567:5", "di:1:0:2:0", "b:3ff0000000000000",
                     "d:1:0"):
        for nominal in ((1, 0), (100, 0)):
            c.add("CAPITAL-BEFORE-ACCRUED", d, op="cash", settlement=D("2024-05-01"),
                  recipe="invoice", nominal=nominal, evidence=evidence,
                  aggregation="RoundComponentsThenSum", quantum=(1, 2), mode="Exact")
    # Wide components outside i128 cancelling to a representable net (UST-66).
    wide = Definition(D("2024-05-15"), D("2024-05-15"), D("2024-11-15"), (2 ** 126, 0),
                      D("2024-11-15"), (D("2024-05-15"), D("2024-11-15")), "Ignore", MAX_DATES,
                      "ExactRatio", "NearestTiesEven")
    for basis, tie in POLICIES:
        w = with_policy(wide, basis, tie)
        for nominal in ((1000, 0), (10000, 0)):
            for agg in AGGREGATIONS:
                for mode in ("Exact", "NearestTiesEven", "TowardZero"):
                    c.add("WIDE-CANCELLATION", w, op="cash", settlement=D("2024-08-15"),
                          recipe="invoice", nominal=nominal, evidence="b:c7b0000000000000",
                          aggregation=agg, quantum=(1, 0), mode=mode)
            c.add("WIDE-CANCELLATION", w, op="cash", settlement=D("2024-08-15"), recipe="accrued",
                  nominal=nominal, quantum=(1, 0), mode="Exact")
        c.add("WIDE-CANCELLATION", w, op="instruction_view", settlement=D("2024-08-15"),
              recipe="accrued", nominal=(1000, 0), quantum=(1, 0), mode="Exact",
              view_quantum=(1, 6), view_mode="Exact")
    # Endpoint labels: collapsed nonpoint image still fails at lower; net collapse at point.
    collapse = "di:%d:1:%d:1" % (I128_MAX - 1, I128_MAX)
    for mode in ("TowardZero", "AwayFromZero", "NearestTiesEven"):
        c.add("COLLAPSE-LABELS", d, op="cash", settlement=D("2024-05-01"), recipe="dirty",
              nominal=(2000, 0), evidence=collapse, quantum=(10, 0), mode=mode)
        for agg in AGGREGATIONS:
            c.add("COLLAPSE-LABELS", d, op="cash", settlement=D("2024-05-01"), recipe="invoice",
                  nominal=(2000, 0), evidence=collapse, aggregation=agg, quantum=(10, 0),
                  mode=mode)
        c.add("COLLAPSE-LABELS", d, op="cash", settlement=D("2024-05-01"), recipe="dirty",
              nominal=(100, 0), evidence="di:99:0:101:0", quantum=(1000, 0), mode=mode)
    # Signed i128 final limits including i128::MIN (UST-68/69).
    for evidence, nominal in (("d:%d:0" % I128_MIN, (100, 0)), ("d:%d:0" % I128_MAX, (100, 0)),
                              ("d:%d:0" % I128_MIN, (101, 0)), ("d:%d:0" % I128_MAX, (101, 0)),
                              ("d:%d:0" % I128_MIN, (100, 2)),
                              ("d:%d:0" % (I128_MIN + 1), (100, 0)),
                              ("di:%d:0:%d:0" % (I128_MAX - 1, I128_MAX), (100, 0)),
                              ("di:%d:0:%d:0" % (I128_MAX - 1, I128_MAX), (101, 0)),
                              ("di:%d:0:%d:0" % (I128_MIN, I128_MIN + 1), (101, 0))):
        for mode in ("Exact", "TowardZero", "AwayFromZero"):
            c.add("SIGNED-I128-LIMITS", d, op="cash", settlement=D("2024-05-01"), recipe="dirty",
                  nominal=nominal, evidence=evidence, quantum=(1, 0), mode=mode)
    # Identity groups: equal cash, distinct provenance (cash T8, UST-03/70).
    regular = shapes["regular-d-eq-i"]
    for basis, tie in POLICIES:
        g = with_policy(regular, basis, tie)
        c.add("IDENTITY", g, op="cash", coupon_index=1, recipe="coupon", nominal=(1000, 0),
              quantum=(1, 2), mode="NearestTiesEven", group="distinct:policy-only")
        c.add("IDENTITY", g, op="instrument", group="distinct:instrument-policy-only")
    for evidence in ("d:100:0", "b:4059000000000000", "di:100:0:100:0",
                     "bi:4059000000000000:4059000000000000"):
        c.add("IDENTITY", regular, op="cash", settlement=D("2024-05-01"), recipe="dirty",
              nominal=(1000, 0), evidence=evidence, quantum=(1, 2), mode="NearestTiesEven",
              group="distinct:evidence-provenance")
    for agg in AGGREGATIONS:
        c.add("IDENTITY", regular, op="cash", settlement=D("2024-05-01"), recipe="invoice",
              nominal=(1000, 0), evidence="d:100:0", aggregation=agg, quantum=(1, 2),
              mode="NearestTiesEven", group="distinct:aggregation")
    for recipe in ("accrued", "invoice", "dirty"):
        c.add("IDENTITY", regular, op="cash", settlement=D("2024-02-15"), recipe=recipe,
              nominal=(1000, 0), evidence=None if recipe == "accrued" else "d:0:0",
              aggregation="RoundTotal" if recipe == "invoice" else None, quantum=(1, 2),
              mode="NearestTiesEven", group="distinct:recipe-zero-cash")
    for name in ("regular-d-eq-i", "regular-d-lt-i"):
        c.add("IDENTITY", shapes[name], op="cash", coupon_index=1, recipe="coupon",
              nominal=(1000, 0), quantum=(1, 2), mode="NearestTiesEven",
              group="distinct:issue-date-only")


def staging_cells():
    """Full 116,800-cell census asserted against cash T11, and the public cells to emit."""
    census, chosen = Counter(), set()
    for eighths in range(1, 161):
        c = F(eighths, 8)
        for s in (184, 183, 182, 181):
            exact_daily = 5 * c / s
            daily9 = {tie: publish(exact_daily, 9, tie) for tie in TIES}
            assert daily9[TIES[0]] == daily9[TIES[1]] and not is_tie(exact_daily * 10 ** 9)
            for t in range(1, s + 1):
                census["cells"] += 1
                raw, staged = exact_daily * t, daily9[TIES[0]] * t
                raw5 = {tie: publish(raw, 5, tie) for tie in TIES}
                staged5 = {tie: publish(staged, 5, tie) for tie in TIES}
                raw_tie, staged_tie = is_tie(raw * 10 ** 5), is_tie(staged * 10 ** 5)
                census["raw-ties"] += raw_tie
                census["staged-ties"] += staged_tie
                census["raw-rule-sensitive"] += raw5[TIES[0]] != raw5[TIES[1]]
                census["staged-rule-sensitive"] += staged5[TIES[0]] != staged5[TIES[1]]
                changes = [raw5[tie] != staged5[tie] for tie in TIES]
                census["changes-half-even"] += changes[0]
                census["changes-half-away"] += changes[1]
                census["changes-either"] += any(changes)
                discriminating = any(changes) or raw_tie or staged_tie
                if t == s:
                    census["t-eq-s-not-single-piece-observable"] += 1
                    census["t-eq-s-discriminating"] += discriminating
                elif discriminating or t == s // 2:
                    chosen.add((eighths, s, t))
    expected = {"cells": 116800, "raw-ties": 584, "staged-ties": 288, "raw-rule-sensitive": 292,
                "staged-rule-sensitive": 144, "changes-half-even": 270, "changes-half-away": 270,
                "changes-either": 350}
    for key, value in expected.items():
        assert census[key] == value, (key, census[key])
    census["emitted-public-cells"] = len(chosen)
    STAGING.update(census)
    return sorted(chosen)


def total_first_cells():
    """Two-piece long-first AI where total-first RN5 differs from per-piece RN5 sums."""
    q = (D("1985-02-15"), D("1985-08-15"), D("1986-02-15"), D("1986-08-15"))
    found = []
    for eighths in range(1, 161, 3):
        c = F(eighths, 8)
        for back in range(1, 181, 7):
            for forward in range(1, 184, 11):
                a, b = 5 * c / 181 * back, 5 * c / 184 * forward
                if publish(a + b, 5, TIES[0]) != publish(a, 5, TIES[0]) + publish(b, 5, TIES[0]):
                    dated = q[1] - timedelta(days=back)
                    defn = Definition(dated, dated, q[-1], canonical_value(c), q[2], q, "Ignore",
                                      MAX_DATES, "ExactRatio", "NearestTiesEven")
                    found.append((defn, q[1] + timedelta(days=forward)))
                    if len(found) == 24:
                        return found
    return found


def json_text(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def find(corpus, **want):
    for line in corpus.rows:
        row = dict(zip(COLUMNS, line.rstrip("\n").split("\t")))
        if all(row[k] == v for k, v in want.items()):
            yield row


def verify_anchors(corpus):
    """Contract T3 printed anchors and the source-examples AI table, as assertions."""
    expected = [  # (group, op, nominal, quantum, mode, basis filter, lower)
        ("anchor:I.A.3", "source_coupon_view", None, None, None, "NineDecimal", "41419837004:10"),
        ("anchor:I.A.4", "source_coupon_view", None, None, None, "NineDecimal", "5521059781:9"),
        ("anchor:I.A.3", "cash", "20000:0", "1:8", "Exact", "NineDecimal", "82839674008:8"),
        ("anchor:I.A.4", "cash", "7000:0", "1:8", "Exact", "NineDecimal", "38647418467:8"),
        ("anchor:I.A.3", "cash", "20000:0", "1:2", "NearestTiesEven", None, "8284:1"),
        ("anchor:I.A.4", "cash", "7000:0", "1:2", "NearestTiesEven", None, "38647:2"),
        ("anchor:I.D(1)", "cash", "150000:0", "1:2", "Exact", None, "253125:2"),
        ("anchor:I.D(2)", "cash", "11000:0", "1:5", "Exact", "NineDecimal", "40400778:5"),
        ("anchor:I.D(2)", "cash", "11000:0", "1:2", "NearestTiesEven", None, "40401:2"),
    ]
    for group, op, nominal, quantum, mode, basis, value in expected:
        rows = [r for r in find(corpus, identity_group=group, op=op)
                if (nominal is None or (r["nominal"], r["quantum"], r["mode"]) == (nominal, quantum, mode))
                and (basis is None or r["basis"] == basis)]
        assert rows and all(r["lower"] == value for r in rows), (
            group, op, value, [r["lower"] for r in rows])
        CHECKS["contract-t3-anchor-rows"] += len(rows)
    rows = list(find(corpus, identity_group="anchor:I.A.3", op="source_coupon_view",
                     basis="ExactRatio"))
    assert rows and all(r["expected"] == "treasury.cash_arithmetic"
                        and r["source_first"] == "6097/1472" for r in rows), rows
    CHECKS["contract-t3-exactratio-30485/736-inexact-rows"] += len(rows)
    # Root diagnostic of the provisional generator: exact five-place tie 23437.5.
    rows = [r for r in find(corpus, identity_group="grid:3/8:184:23") if r["basis"] == "ExactRatio"]
    assert len(rows) == 2 and all(r["pub_ai"] == "11719/500000" and r["lower"] == "23438:6"
                                  for r in rows), rows
    CHECKS["root-diagnostic-exact-tie-23437.5-rows"] += len(rows)
    published = {"source:I.D(1)": "27/16", "source:II.D": "367403/1000000",
                 "source:II.E": "1836399/500000", "source:II.F": "2596467/1000000",
                 "source:II.G": "41291/50000"}
    for group, value in published.items():
        rows = [r for r in find(corpus, identity_group=group, op="plan")
                if r["basis"] == "NineDecimal"]
        assert len(rows) == 2 and all(r["pub_ai"] == value for r in rows), (group, value, rows)
        CHECKS["source-example-published-ai-rows"] += len(rows)


def generate(generated_at, platform_text):
    main, cash = MAIN.read_bytes(), CASH.read_bytes()
    assert sha(main) == MAIN_SHA, "adopted main contract changed"
    assert sha(cash) == CASH_SHA, "adopted cash contract changed"
    assert sha(TEST_DESIGN.read_bytes()) == TEST_DESIGN_SHA, "test design changed"
    assert sha(SOURCE_CHECKER.read_bytes()) == SOURCE_CHECKER_SHA, "parent source checker changed"
    rows = [line for line in REQUIREMENTS.read_text(encoding="utf-8").splitlines()
            if line.startswith("| FR-UST-")]
    requirements = ("\n".join(rows) + "\n").encode()
    assert len(rows) == 26 and sha(requirements) == REQUIREMENT_ROWS_SHA, "FR-UST rows changed"
    for counter in (CHECKS, STAGING, WIDTHS):
        counter.clear()
    corpus = Corpus()
    populate(corpus)
    verify_anchors(corpus)
    counters = corpus.counters
    assert set(counters["cash-mode"]) == set(MODES) and set(counters["view-mode"]) == set(MODES)
    assert all(scales == set(range(39)) for scales in corpus.scales.values()), corpus.scales
    assert set(counters["evidence"]) == {"d", "di", "b", "bi"}
    assert set(counters["policy"]) == {"%s/%s" % p for p in POLICIES}
    assert set(counters["recipe"]) == set(RECIPE_ID)
    assert set(counters["aggregation"]) == set(AGGREGATIONS)
    assert set(counters["first-period"]) == {"Regular", "Short", "Long"}
    assert set(ERROR_KINDS) <= set(counters["expected"]), set(ERROR_KINDS) - set(counters["expected"])
    assert {"ok", "some", "none", "determined", "ambiguous",
            "exact_unresolved"} <= set(counters["expected"])
    assert set(STAGES) <= set(counters["stage"]), set(STAGES) - set(counters["stage"])
    assert set(counters["endpoint"]) == {"point", "lower", "upper"}, counters["endpoint"]
    assert set(counters["cause"]) == {"amount.inexact", "amount.out_of_range"}
    assert set(counters["operation"]) == {"quantize", "sum"}, counters["operation"]
    assert set(counters["guarantee"]) == {"none", "ForSuppliedDecimals", "ForSuppliedBinary64",
                                          "UniqueForDeclaredEnclosure"}
    assert len(corpus.seen) == len(corpus.rows) == len(set(corpus.inputs))
    source = Path(__file__).read_bytes()
    records = "".join(corpus.rows).encode()
    inputs = "".join(corpus.inputs).encode()
    body = "\t".join(COLUMNS).encode() + b"\n" + records
    headers = [
        ("schema", "1"), ("fixture-set", "US-TREASURY-SOURCE-DATE-CASH-ORACLE"),
        ("domain", "bonds"),
        ("description", "Independent exact US Treasury instrument dates, plan accrual, source coupon/accrued publication, views and all four cash recipes under all four source policies"),
        ("reference", "Python stdlib Fraction with integer quotient/remainder rounding (route A, authoritative); separately decoded route B recomputes every value column from main E/cash T9 integer forms by exact Decimal integer divmod with Inexact and Rounded trapped"),
        ("reference-version", "Python>=3.9 stdlib; Decimal contexts 1536,2048 digits (capacity only, all route B operations exact)"),
        ("reference-source", "Adopted local Treasury main contract A-F and cash contract T1-T12 at commit a1b66f9; FR-UST-01..26; no Treasury Rust or QuantLib implementation read"),
        ("tolerance", "TOL-E0 exact: canonical amounts, reduced rationals, binary64 bits, structured decisions and errors"),
        ("reproduction", "python3 -I validation/generator/treasury_source_cash_oracles.py --check"),
        ("generated-at", generated_at), ("platform", platform_text),
        ("generator-sha256", sha(source)), ("generator-fnv64", fnv(source)),
        ("adopted-contract-sha256", sha(main)), ("adopted-contract-fnv64", fnv(main)),
        ("cash-contract-sha256", sha(cash)), ("cash-contract-fnv64", fnv(cash)),
        ("requirements-rows-sha256", sha(requirements)),
        ("requirements-rows-fnv64", fnv(requirements)),
        ("contract-commit", CONTRACT_COMMIT), ("test-design-sha256", TEST_DESIGN_SHA),
        ("parent-source-xml-sha256", SOURCE_XML_SHA),
        ("parent-source-checker-sha256", SOURCE_CHECKER_SHA),
        ("source-version", SOURCE_VERSION),
        ("encoding", "TSV; decimal=coef:scale (inputs may be noncanonical, outputs canonical); rational=N/D reduced; dates ISO, lists comma-separated, empty Q=empty; -=not applicable; none=Option None; evidence=d:C:S,di:C:S:C:S,b:16hex,bi:16hex:16hex; context=sorted key=value with strings x:UTF-8hex; bundles=first..last absolute Q index; entitlements=ceased/coupon_count; pieces,day_ratios=t/s; bits=binary64 hex"),
        ("column-count", str(len(COLUMNS))), ("record-count", str(len(corpus.rows))),
        ("input-columns", ",".join(INPUT_COLUMNS)),
        ("output-columns", ",".join(OUTPUT_COLUMNS)),
        ("annotation-columns", ",".join(ANNOTATION_COLUMNS)),
        ("unique-input-count", str(len(corpus.seen))),
        ("inputs-sha256", sha(inputs)), ("inputs-fnv64", fnv(inputs)),
        ("profile-counts", json_text(counters["profile"])),
        ("profile-families", json_text(PROFILE_FAMILIES)),
        ("operation-counts", json_text(counters["op"])),
        ("expectation-counts", json_text(counters["expected"])),
        ("policy-counts", json_text(counters["policy"])),
        ("first-period-counts", json_text(counters["first-period"])),
        ("eom-counts", json_text(counters["eom"])),
        ("ai-piece-counts", json_text(counters["ai-pieces"])),
        ("recipe-counts", json_text(counters["recipe"])),
        ("cash-mode-counts", json_text(counters["cash-mode"])),
        ("view-mode-counts", json_text(counters["view-mode"])),
        ("aggregation-counts", json_text(counters["aggregation"])),
        ("evidence-kind-counts", json_text(counters["evidence"])),
        ("stage-counts", json_text(counters["stage"])),
        ("endpoint-counts", json_text(counters["endpoint"])),
        ("cause-counts", json_text(counters["cause"])),
        ("operation-label-counts", json_text(counters["operation"])),
        ("guarantee-counts", json_text(counters["guarantee"])),
        ("scale-census", json_text({k: "0..38" for k in corpus.scales})),
        ("staging-grid-census", json_text(STAGING)),
        ("width-maxima-bits", json_text(WIDTHS)),
        ("width-bounds-bits", json_text(WIDTH_BOUNDS)),
        ("decimal-capacity-digits", json_text(DECIMAL_CAPACITY)),
        ("cross-check-counts", json_text(CHECKS)),
        ("records-sha256", sha(records)), ("records-fnv64", fnv(records)),
        ("body-sha256", sha(body)), ("body-fnv64", fnv(body)),
    ]
    for key, value in headers:
        assert value and value.strip() == value and "\n" not in value and "\t" not in value, key
    data = "".join("%s: %s\n" % pair for pair in headers).encode() + b"\n" + body
    return data, dict(headers)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="regenerate and compare every byte")
    parser.add_argument("--output", type=Path, default=DEST)
    args = parser.parse_args()
    if args.check:
        if not args.output.is_file():
            raise SystemExit("no frozen fixture to check")
        header = args.output.read_text(encoding="utf-8").split("\n\n", 1)[0]
        metadata = dict(line.split(": ", 1) for line in header.splitlines())
        timestamp, platform_text = metadata["generated-at"], metadata["platform"]
    else:
        if args.output.exists():
            raise SystemExit("refusing to overwrite existing evidence; use --check or a new --output")
        timestamp = datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
        platform_text = "%s; Python %s" % (platform.platform(), platform.python_version())
    data, summary = generate(timestamp, platform_text)
    if args.check:
        if args.output.read_bytes() != data:
            raise SystemExit("treasury source/cash fixture differs from independent regeneration")
        summary["check"] = "identical"
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open("xb") as stream:
            stream.write(data)
        summary["written"] = str(args.output)
    keep = ("record-count", "unique-input-count", "generator-sha256", "records-sha256",
            "body-sha256", "inputs-sha256", "check", "written", "generated-at", "platform")
    print(json.dumps({k: summary[k] for k in keep if k in summary}, indent=2, sort_keys=True))
    for key in ("expectation-counts", "width-maxima-bits", "cross-check-counts",
                "staging-grid-census"):
        print(key + ": " + summary[key])


if __name__ == "__main__":
    main()
