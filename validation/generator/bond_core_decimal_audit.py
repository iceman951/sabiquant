#!/usr/bin/env python3
"""Independent, additive audit of the frozen local M8 bond-core corpus.

Only the contract, input cells and published mathematical policy are inputs to
the model. Frozen expectations are inspected after constructing each answer.
No SabiQuant production code or previous generator calculation is imported.
"""

import argparse
import calendar
import collections
import csv
import datetime
from decimal import Decimal, ROUND_HALF_EVEN, localcontext
from fractions import Fraction
import hashlib
import io
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "validation/fixtures/bonds/bond-core-oracles.fixture"
CONTRACT = ROOT / "docs/M8_BOND_CORE_CONTRACT.md"
DEST = ROOT / "validation/fixtures/bonds/bond-core-decimal-audit.fixture"
PRECISIONS = (160, 240)
QUOTE_PRECISIONS = (256, 512)
SOURCE_SHA256 = "5a5dc4f4ec573d189b8b7f87aeaf335b5a253f1ad983633612dfbe2ae8ae7817"
CONTRACT_SHA256 = "009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c"
SOURCE_GENERATOR_SHA256 = "afe27c0ce1a362492a98202c2882271cabbc3b1d510bb49dd3d5215460055b18"
SIGN = 1 << 63
MASK = SIGN - 1
INF = 0x7ff0000000000000
U = Fraction(1, 1 << 53)
MIN_SUB = Fraction(1, 1 << 1074)
INPUT_NAMES = (
    "id", "profile", "op", "face_bits", "value_bits", "spec", "settlement",
    "inclusion", "yield_dcc", "yield_comp", "yield_bits", "work_cap",
)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def bits_value(bits):
    """Decode a finite bit pattern exactly, without binary floating arithmetic."""
    magnitude = bits & MASK
    exponent = magnitude >> 52
    mantissa = magnitude & ((1 << 52) - 1)
    if exponent == 0x7ff:
        return None
    if exponent:
        mantissa += 1 << 52
        power = exponent - 1023 - 52
    else:
        power = -1074
    value = Fraction(mantissa << power) if power >= 0 else Fraction(mantissa, 1 << -power)
    return -value if bits & SIGN else value


def hx(bits):
    return f"0x{bits:016x}"


def parse_bits(cell):
    assert len(cell) == 18 and cell.startswith("0x")
    return int(cell[2:], 16)


def power_two(exponent):
    return Fraction(1 << exponent) if exponent >= 0 else Fraction(1, 1 << -exponent)


def unit_for(value):
    """Select the target binary grid using an exact integer binade comparison."""
    n, d = value.numerator, value.denominator
    top = n.bit_length() - d.bit_length()
    if value < power_two(top):
        top -= 1
    return max(top - 52, -1074)


def encode_quantum_integer(integer, unit, negative):
    if integer == 0:
        return "underflow", SIGN if negative else 0
    top = integer.bit_length() - 1 + unit
    if top > 1023:
        return "overflow", INF | (SIGN if negative else 0)
    if top < -1022:
        magnitude = integer << (unit + 1074)
    else:
        shift = integer.bit_length() - 53
        if shift > 0:
            assert integer % (1 << shift) == 0
            significand = integer >> shift
        else:
            significand = integer << -shift
        magnitude = ((top + 1023) << 52) | (significand - (1 << 52))
    return "ok", magnitude | (SIGN if negative else 0)


def rounded(value):
    """Round an exact Fraction once, nearest-even, including IEEE range edges."""
    if value == 0:
        return "ok", 0, False
    negative = value < 0
    value = abs(value)
    unit = unit_for(value)
    scaled = value / power_two(unit)
    q, r = divmod(scaled.numerator, scaled.denominator)
    twice = 2 * r
    tie = twice == scaled.denominator
    if twice > scaled.denominator or (tie and q % 2):
        q += 1
    status, result = encode_quantum_integer(q, unit, negative)
    return status, result, tie


def decimal_rounded(value, precision):
    """Separately decide the rounded integer VALUE with Decimal, not its sign.

    Scaling to a binary target unit avoids double rounding at exact binary
    midpoints. For a nontrivial quote rounding decision, the reduced operands
    fit 108 bits, so 256 decimal digits exceed both the terminating midpoint
    expansion and the off-midpoint rational separation by a wide margin.
    """
    if value == 0:
        return "ok", 0
    negative = value < 0
    value = abs(value)
    unit = unit_for(value)
    scaled = value / power_two(unit)
    if Fraction(1, 4) <= scaled <= 1 << 53:
        assert max(scaled.numerator.bit_length(), scaled.denominator.bit_length()) <= precision
    with localcontext() as ctx:
        ctx.prec = precision
        quotient = Decimal(scaled.numerator) / Decimal(scaled.denominator)
        q = int(quotient.to_integral_value(rounding=ROUND_HALF_EVEN))
    return encode_quantum_integer(q, unit, negative)


def rounding_self_checks():
    """Independent hand anchors include both parity directions and range ties."""
    cases = (
        (Fraction(0), "ok", 0),
        (MIN_SUB / 2, "underflow", 0),
        (MIN_SUB / 2 + power_two(-1200), "ok", 1),
        (MIN_SUB * Fraction(3, 2), "ok", 2),
        (MIN_SUB * Fraction(5, 2), "ok", 2),
        (1 + power_two(-53), "ok", 0x3ff0000000000000),
        (1 + 3 * power_two(-53), "ok", 0x3ff0000000000002),
        (2 - power_two(-53), "ok", 0x4000000000000000),
        (bits_value(0x7fefffffffffffff), "ok", 0x7fefffffffffffff),
        (bits_value(0x7fefffffffffffff) + power_two(970), "overflow", INF),
    )
    for exact, status, bits in cases:
        for sign in (1, -1):
            signed_bits = bits | (SIGN if sign < 0 and exact else 0)
            assert rounded(sign * exact)[:2] == (status, signed_bits)
            for precision in QUOTE_PRECISIONS:
                assert decimal_rounded(sign * exact, precision) == (status, signed_bits)
    return 2 * len(cases)


def finite_fraction(cell):
    value = bits_value(parse_bits(cell))
    assert value is not None
    return value


def round_fraction(value):
    status, bits, _ = rounded(value)
    assert status == "ok", (status, value)
    return bits_value(bits)


def nonfinite_class(bits):
    if bits & ((1 << 52) - 1):
        return "NaN"
    return "NegativeInfinity" if bits & SIGN else "PositiveInfinity"


def quote_answer(row):
    face_bits, value_bits = parse_bits(row["face_bits"]), parse_bits(row["value_bits"])
    face, value = bits_value(face_bits), bits_value(value_bits)
    if face is None:
        return "bond.value", f"phase=bond face,kind=value.not_finite,class={nonfinite_class(face_bits)}", "-", None
    if face <= 0:
        return "bond.value", "phase=bond face,kind=value.out_of_domain", "-", None
    forward = row["op"] == "per_100_from_amount"
    input_phase = "price per 100" if forward else "bond price"
    if value is None:
        return "bond.value", f"phase={input_phase},kind=value.not_finite,class={nonfinite_class(value_bits)}", "-", None
    exact = 100 * value / face if forward else face * value / 100
    status, result, tie = rounded(exact)
    scaled = abs(exact) / power_two(unit_for(abs(exact))) if exact else Fraction(0)
    width = max(scaled.numerator.bit_length(), scaled.denominator.bit_length()) if scaled >= Fraction(1, 4) else 0
    assert width <= 108
    parity = (scaled.numerator // scaled.denominator) % 2
    for precision in QUOTE_PRECISIONS:
        assert decimal_rounded(exact, precision) == (status, result), (row["id"], precision)
    if status == "ok":
        return "ok", "-", hx(result), (status, tie, result, width, parity)
    phase = "price per 100" if forward else "bond amount"
    kind = "value.result_not_finite" if status == "overflow" else "value.result_out_of_domain"
    classification = f",class={nonfinite_class(result)}" if status == "overflow" else ""
    return "bond.value", f"phase={phase},kind={kind}{classification}", "-", (status, tie, result, width, parity)


def date(cell):
    return datetime.date.fromisoformat(cell)


def yf(start, end, convention):
    """Six selected explicit civil-date definitions, then one binary64 rounding."""
    if end < start:
        return -yf(end, start, convention)
    days = (end - start).days
    if convention == "Act365F":
        exact = Fraction(days, 365)
    elif convention == "Act360":
        exact = Fraction(days, 360)
    elif convention == "B252_WEEKENDS":
        count = sum((start + datetime.timedelta(days=i)).weekday() < 5 for i in range(days))
        exact = Fraction(count, 252)
    else:
        d1, d2 = start.day, end.day
        if convention == "30E360":
            d1, d2 = min(d1, 30), min(d2, 30)
        elif convention == "30360BB":
            d1 = min(d1, 30)
            if d1 == 30 and d2 == 31:
                d2 = 30
        elif convention == "30360US_EOM":
            feb1 = start.month == 2 and start.day == calendar.monthrange(start.year, 2)[1]
            feb2 = end.month == 2 and end.day == calendar.monthrange(end.year, 2)[1]
            if feb1 and feb2:
                d2 = 30
            if feb1:
                d1 = 30
            if d2 == 31 and d1 in (30, 31):
                d2 = 30
            if d1 == 31:
                d1 = 30
        else:
            raise AssertionError(convention)
        exact = Fraction(360 * (end.year - start.year) + 30 * (end.month - start.month) + d2 - d1, 360)
    return round_fraction(exact)


class PreparationFailure(Exception):
    def __init__(self, kind, context):
        super().__init__(kind, context)
        self.kind, self.context = kind, context


def prepare(row):
    kind, issue, maturity, redemption_date, redemption_bits, coupon_dcc, coupons_cell = row["spec"].split("|")
    issue, maturity, redemption_date = date(issue), date(maturity), date(redemption_date)
    settlement = date(row["settlement"])
    if settlement < issue:
        raise PreparationFailure("bond.settlement_before_issue", f"settlement_date={settlement},issue_date={issue}")
    face, rate, redemption = finite_fraction(row["face_bits"]), finite_fraction(row["value_bits"]), finite_fraction(redemption_bits)
    maximum, used = int(row["work_cap"]), 0

    def query(start, end, convention, index, phase):
        nonlocal used
        cost = 1 + abs((end - start).days) if convention == "B252_WEEKENDS" else 1
        if used + cost > maximum:
            raise PreparationFailure("bond.accrual_work_limit_exceeded", f"index={index},phase={phase},maximum={maximum},used={used},required_at_least={used + cost}")
        used += cost
        return yf(start, end, convention)

    def eligible(payment):
        return payment > settlement or (payment == settlement and row["inclusion"] == "Include")

    query(settlement, settlement, row["yield_dcc"], "none", "discount anchor")
    entitlements, accrued_terms, flows = [], [], []
    coupons = [] if coupons_cell == "-" else coupons_cell.split(";")
    for index, cell in enumerate(coupons):
        start, end, payment, ex = cell.split("@")
        start, end, payment = date(start), date(end), date(payment)
        full = round_fraction(face * rate * yf(start, end, coupon_dcc))
        if not eligible(payment):
            entitlements.append("C")
            continue
        detached = False
        if ex != "-":
            boundary, cutoff = ex.split(":")
            cutoff = date(cutoff)
            detached = settlement >= cutoff if boundary == "OnOrAfter" else settlement > cutoff
        entitlements.append("X" if detached else "B")
        if settlement <= start:
            earned = Fraction(0)
        elif settlement >= end:
            earned = full
        else:
            fraction = query(start, settlement, coupon_dcc, str(index), "partial accrual")
            earned = round_fraction(face * rate * fraction)
        accrued_terms.append(round_fraction(earned - full) if detached else earned)
        if not detached:
            horizon = query(settlement, payment, row["yield_dcc"], str(index), "discount horizon")
            flows.append((full, horizon))
    redemption_entitled = eligible(redemption_date)
    if redemption_entitled:
        horizon = query(settlement, redemption_date, row["yield_dcc"], str(len(coupons)), "redemption discount horizon")
        flows.append((redemption, horizon))
    accrued = round_fraction(sum(accrued_terms, Fraction(0)))
    return {
        "entitlements": "".join(entitlements) or "-", "redemption_entitled": int(redemption_entitled),
        "flow_count": len(flows), "work": used, "accrued": accrued, "flows": flows,
    }


def dec(value):
    return Decimal(value.numerator) / Decimal(value.denominator)


def discount(amount, horizon, rate, compounding):
    """Ideal fixed-input factors plus independently rounded operational stages."""
    method, _, frequency = compounding.partition(":")
    frequency = int(frequency) if frequency else 0
    if frequency:
        assert rate > -frequency
    active = method
    if method in ("SimpleThenCompounded", "CompoundedThenSimple"):
        threshold = round_fraction(Fraction(1, frequency))
        use_simple = horizon <= threshold if method == "SimpleThenCompounded" else horizon > threshold
        active = "Simple" if use_simple else "Compounded"
    t, r = dec(horizon), dec(rate)
    if active == "Simple":
        compound_exact = 1 + rate * horizon
        assert compound_exact > 0
        ideal = Decimal(1) / dec(compound_exact)
        compound_stored = round_fraction(compound_exact)
        discount_stored = round_fraction(1 / compound_stored)
        # Exact fused-factor error, plus one correctly rounded reciprocal.
        factor_bound = abs(dec(1 / compound_stored) - ideal) + dec(U) * abs(dec(1 / compound_stored)) + dec(MIN_SUB) / 2
        log_compound = dec(compound_exact).ln()
    else:
        log_compound = r * t if active == "Continuous" else Decimal(frequency) * t * (1 + r / frequency).ln()
        ideal = (-log_compound).exp()
        compound_stored = round_fraction(Fraction(log_compound.exp()))
        discount_stored = round_fraction(1 / compound_stored)
        # The existing measured M4 E3 criterion; not a libm precision theorem.
        factor_bound = 8 * dec(U) * (1 + abs(log_compound)) * abs(ideal) + dec(MIN_SUB)
    pv_ideal = dec(amount) * ideal
    pv_staged = round_fraction(amount * discount_stored)
    if amount == 0 or rate == 0 or horizon == 0:
        factor_bound = Decimal(0)
    pv_bound = abs(dec(amount)) * factor_bound
    if amount and rate and horizon:
        pv_bound += dec(U) * abs(dec(amount)) * (abs(ideal) + factor_bound) + dec(MIN_SUB) / 2
    return pv_ideal, pv_staged, pv_bound, abs(pv_ideal), active


def valuation(row, prepared, precision):
    with localcontext() as ctx:
        ctx.prec = precision
        rate = finite_fraction(row["yield_bits"])
        terms = [discount(a, t, rate, row["yield_comp"]) for a, t in prepared["flows"]]
        ideal_dirty = sum((t[0] for t in terms), Decimal(0))
        ideal_clean = ideal_dirty - dec(prepared["accrued"])
        gross = sum((t[3] for t in terms), Decimal(0))
        bound_terms = sum((t[2] for t in terms), Decimal(0))
        dirty_bound = bound_terms + dec(U) * (abs(ideal_dirty) + bound_terms) + dec(MIN_SUB) / 2
        clean_bound = dirty_bound + dec(U) * (abs(ideal_clean) + dirty_bound) + dec(MIN_SUB) / 2
        staged_dirty = round_fraction(sum((t[1] for t in terms), Fraction(0)))
        staged_clean = round_fraction(staged_dirty - prepared["accrued"])
        outputs = [rounded(Fraction(x))[1] for x in (ideal_dirty, ideal_clean, staged_dirty, staged_clean)]
        return {
            "bits": outputs, "ideal_dirty": +ideal_dirty, "ideal_clean": +ideal_clean,
            "dirty_bound": +dirty_bound, "clean_bound": +clean_bound, "gross": +gross,
            "active": tuple(t[4] for t in terms),
        }


def cancellation_case():
    """Staged PV cancellation is intentionally distinct from an ideal net PV."""
    return dict(zip(INPUT_NAMES, (
        "AUDIT-CANCELLATION-001", "SIGNED-CANCELLATION", "bond_plan", hx(0x4059000000000000),
        hx(0xbff0000000000000),
        "FixedRate|2026-01-01|2027-01-01|2027-01-01|0x4059000000000001|Act365F|2026-01-01@2027-01-01@2027-01-01@-",
        "2026-01-01", "Exclude", "Act365F", "Simple", hx(0x3fd0000000000000), "10",
    )))


def load_source():
    source = SOURCE.read_bytes()
    assert sha(source) == SOURCE_SHA256
    assert sha(CONTRACT.read_bytes()) == CONTRACT_SHA256
    assert sha((ROOT / "validation/generator/bond_core_oracles.py").read_bytes()) == SOURCE_GENERATOR_SHA256
    header, body = source.decode("ascii").split("\n\n", 1)
    rows = list(csv.DictReader(io.StringIO(body), delimiter="\t"))
    metadata = dict(line.split(": ", 1) for line in header.splitlines())
    assert sha(body.encode()) == metadata["body-sha256"]
    assert len(rows) == int(metadata["record-count"])
    assert list(rows[0]) == list(INPUT_NAMES) + ["expected", "context", "output"]
    assert [row["id"] for row in rows] == [f"BOND-{i:06}" for i in range(1, len(rows) + 1)]
    assert len({tuple(row[key] for key in INPUT_NAMES[2:]) for row in rows}) == int(metadata["unique-input-count"])
    for column, metadata_name in (("profile", "profile-counts"), ("op", "operation-counts"), ("expected", "expectation-counts")):
        declared = {key: int(value) for key, value in (item.split("=") for item in metadata[metadata_name].split(";"))}
        assert dict(collections.Counter(row[column] for row in rows)) == declared
    return source, rows


def same_sign_two_ulp_radius(bits):
    """Exact maximum amount error admitted by the existing Rust consumer.

    That consumer allows at most two integer-bit steps with the same sign.
    Across a binade boundary the two spacings need not be equal.
    """
    sign, magnitude = bits & SIGN, bits & MASK
    original = bits_value(bits)
    neighbors = [bits_value(sign | m) for m in (max(0, magnitude - 2), min(INF - 1, magnitude + 2))]
    return max(abs(neighbor - original) for neighbor in neighbors)


def audit():
    source, rows = load_source()
    counters = collections.Counter()
    counters["rounding_hand_self_checks"] = rounding_self_checks()
    profiles = collections.Counter()
    quote_hash = hashlib.sha256()
    records = []
    max_scores = {"dirty": Decimal(0), "clean": Decimal(0)}
    max_ulp = {"dirty": 0, "clean": 0}
    maxima = {}
    classifications = collections.Counter()
    for original in rows + [cancellation_case()]:
        row = {key: original[key] for key in INPUT_NAMES}
        profiles[row["profile"]] += 1
        if row["op"] != "bond_plan":
            expected, context, output, detail = quote_answer(row)
            # First access to the frozen numerical expectations follows the
            # complete independent Fraction + two-precision Decimal answer.
            assert (expected, context, output) == (original["expected"], original["context"], original["output"]), row["id"]
            counters["quote_rows"] += 1
            counters[f"quote_status_{expected}"] += 1
            if detail is not None:
                status, tie, bits, width, parity = detail
                counters["decimal_quote_checks"] += len(QUOTE_PRECISIONS)
                counters["quote_max_nontrivial_scaled_operand_bits"] = max(counters["quote_max_nontrivial_scaled_operand_bits"], width)
                counters[f"quote_range_{status}"] += 1
                counters["quote_exact_midpoint_rows"] += tie
                if tie:
                    counters[f"quote_midpoint_floor_parity_{'odd' if parity else 'even'}"] += 1
                counters["quote_subnormal_outputs"] += status == "ok" and 0 < (bits & MASK) < (1 << 52)
            quote_hash.update((row["id"] + "\t" + expected + "\t" + context + "\t" + output + "\n").encode())
            continue
        counters["plan_rows"] += 1
        try:
            prepared = prepare(row)
        except PreparationFailure as failure:
            assert failure.kind == original["expected"], (row["id"], failure.kind, original["expected"])
            assert failure.context == original["context"], (row["id"], failure.context, original["context"])
            counters[f"plan_status_{failure.kind}"] += 1
            records.append([row[name] for name in INPUT_NAMES] + [failure.kind, failure.context] + ["-"] * 14)
            continue
        values = [valuation(row, prepared, precision) for precision in PRECISIONS]
        low, high = values
        assert low["bits"] == high["bits"], (row["id"], low["bits"], high["bits"])
        for key in ("ideal_dirty", "ideal_clean", "gross", "dirty_bound", "clean_bound"):
            # Relative agreement is tested against gross, preserving zero and
            # heavily cancelled results without an inappropriate relative test.
            assert abs(low[key] - high[key]) <= max(high["gross"], Decimal(1)) * Decimal("1e-145"), (row["id"], key)
        counters["decimal_plan_precision_agreements"] += 1
        for active in high["active"]:
            counters[f"discount_active_{active}"] += 1
        accrued_bits = rounded(prepared["accrued"])[1]
        fields = [prepared["entitlements"], str(prepared["redemption_entitled"]), str(prepared["flow_count"]), str(prepared["work"]), hx(accrued_bits)]
        linked_bounds = []
        if "output" in original:
            assert original["expected"] == "ok", row["id"]
            frozen = original["output"].split(":")
            assert frozen[:4] == fields[:4], (row["id"], frozen[:4], fields[:4])
            assert frozen[5] == fields[4], (row["id"], frozen[5], fields[4])
            frozen_dirty, frozen_clean = parse_bits(frozen[4]), parse_bits(frozen[6])
            assert len(frozen) == 10
            with localcontext() as ctx:
                ctx.prec = max(PRECISIONS)
                for name, frozen_bits, ideal_index, staged_index in (("dirty", frozen_dirty, 0, 2), ("clean", frozen_clean, 1, 3)):
                    error = abs(dec(bits_value(frozen_bits)) - high[f"ideal_{name}"])
                    score = error / high[f"{name}_bound"]
                    assert score <= 1, (row["id"], name, str(score))
                    if score >= max_scores[name]:
                        max_scores[name] = score
                        maxima[f"{name}_conditioned_score_id"] = row["id"]
                    distance = abs(frozen_bits - high["bits"][ideal_index])
                    classifications[f"{name}_frozen_vs_ideal_ulp_{distance}"] += 1
                    if distance >= max_ulp[name]:
                        max_ulp[name] = distance
                        maxima[f"{name}_ideal_ulp_id"] = row["id"]
                    classifications[f"{name}_frozen_vs_independent_staged_{'same' if frozen_bits == high['bits'][staged_index] else 'different'}"] += 1
                    linked_bounds.append(high[f"{name}_bound"] + dec(same_sign_two_ulp_radius(frozen_bits)))
            # Per-100 outputs are exact conversions of the supplied rounded
            # amounts, not conversions of ideal unrounded Decimal net prices.
            for amount_index, quote_index in ((4, 7), (5, 8), (6, 9)):
                exact = 100 * bits_value(parse_bits(frozen[amount_index])) / finite_fraction(row["face_bits"])
                assert hx(rounded(exact)[1]) == frozen[quote_index], (row["id"], quote_index)
                for precision in QUOTE_PRECISIONS:
                    assert decimal_rounded(exact, precision) == rounded(exact)[:2]
                counters["decimal_valuation_quote_checks"] += len(QUOTE_PRECISIONS)
            counters["frozen_plan_successes"] += 1
        else:
            # exact values: ideal net = 0.8 * 2^-46; staged net = 2^-46.
            assert high["bits"][2] == rounded(Fraction(1, 1 << 46))[1]
            assert high["bits"][0] == rounded(Fraction(4, 5 << 46))[1]
            error = abs(dec(bits_value(high["bits"][2])) - high["ideal_dirty"])
            assert error <= high["dirty_bound"]
            counters["supplementary_signed_cancellation"] += 1
            linked_bounds = [high["dirty_bound"], high["clean_bound"]]
        counters["plan_status_ok"] += 1
        with localcontext() as ctx:
            ctx.prec = 35
            # Round bounds upward for serialization. The last printed decimal
            # digit is enlarged rather than risking an understated threshold.
            rendered = [format(+high[key] * Decimal("1.000000000000000000000001"), ".25E") for key in ("dirty_bound", "clean_bound", "gross")]
            rendered += [format(+value * Decimal("1.000000000000000000000001"), ".25E") for value in linked_bounds]
        records.append([row[name] for name in INPUT_NAMES] + ["ok", "-"] + fields + [hx(b) for b in high["bits"]] + rendered)
    assert counters["quote_rows"] == 16_656
    assert counters["plan_rows"] == 1_497
    assert counters["frozen_plan_successes"] == 1_187
    assert counters["decimal_quote_checks"] == 33_048
    assert counters["decimal_valuation_quote_checks"] == 7_122
    counters["unclassified_frozen_mismatches"] = 0
    summary = {
        "counts": dict(sorted(counters.items())), "profiles": dict(sorted(profiles.items())),
        "classifications": dict(sorted(classifications.items())),
        "maxima_row_ids": maxima,
        "max_frozen_ideal_ulp_distance": max_ulp,
        "max_frozen_conditioned_error_score": {k: format(v, ".20E") for k, v in max_scores.items()},
        "quote_answer_sha256": quote_hash.hexdigest(),
    }
    columns = INPUT_NAMES + ("expected", "context", "entitlements", "redemption_entitled", "flow_count", "work", "accrued_bits", "ideal_dirty_bits", "ideal_clean_bits", "staged_dirty_bits", "staged_clean_bits", "dirty_bound", "clean_bound", "gross", "linked_dirty_bound", "linked_clean_bound")
    body = "\t".join(columns) + "\n" + "".join("\t".join(record) + "\n" for record in records)
    header = {
        "schema": "1", "fixture-set": "M8-BOND-CORE-DECIMAL-AUDIT", "domain": "bonds",
        "description": "Additive two-precision Decimal direct-horizon and exact Fraction quote audit; independent ideal and staged models",
        "reference": "oracle:independent-Fraction-bitfield+Decimal-160-240-pricing+Decimal-256-512-quote-values",
        "reference-version": "Python >=3.9 standard library only", "reference-source": "Frozen local bond core contract; frozen corpus INPUT cells only",
        "tolerance": "TOL-E0 exact quote/accrual/entitlement/error checks; ideal pricing uses gross-conditioned measured M4 E3 factor plus rounding propagation; linked bounds add the existing same-sign two-ULP consumer radius",
        "reproduction": "python3 -I validation/generator/bond_core_decimal_audit.py --check",
        "generated-at": "2026-10-16T08:44:25Z", "platform": "platform-independent integer/Decimal arithmetic",
        "generator-sha256": sha(Path(__file__).read_bytes()), "adopted-contract-sha256": sha(CONTRACT.read_bytes()),
        "source-fixture-sha256": sha(source), "source-generator-sha256": sha((ROOT / "validation/generator/bond_core_oracles.py").read_bytes()),
        "record-count": str(len(records)), "body-sha256": sha(body.encode()),
        "audit-summary": json.dumps(summary, sort_keys=True, separators=(",", ":")),
    }
    return ("".join(f"{key}: {value}\n" for key, value in header.items()) + "\n" + body).encode(), summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Recompute all checks and require byte-identical additive fixture")
    args = parser.parse_args()
    result, summary = audit()
    if args.check:
        assert DEST.read_bytes() == result, f"fixture differs: {DEST}"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(result)
    print(json.dumps(summary, indent=2, sort_keys=True))
    print(f"fixture_sha256={sha(result)} bytes={len(result)} check={args.check}")


if __name__ == "__main__":
    main()
