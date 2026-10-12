#!/usr/bin/env python3
"""Attested public QuantLib rate APIs with independently checked rounding stages."""

import argparse
import collections
import datetime as dt
from decimal import Decimal as D, localcontext
from fractions import Fraction
import hashlib
import math
import pathlib
import platform
import struct
import sys
import zipfile

import rate_oracles as oracle

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/rates"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
ORACLE_REVISION = hashlib.sha256(pathlib.Path(oracle.__file__).read_bytes()).hexdigest()
UNIT = 2.0 ** -53


def config(ql, method):
    return getattr(ql, method[0]), getattr(ql, method[1]) if method[1] else ql.NoFrequency


def cases(operation):
    yield from oracle.cases(operation)
    if operation == "implied":
        for method in oracle.METHODS:
            for factor in (0.0, -oracle.MIN_SUB, -1.0):
                for time in (0.0, 1.0):
                    yield operation, method, 0.0, time, factor, method
            for time in (-oracle.MIN_SUB, -1.0):
                yield operation, method, 0.0, time, 1.0, method
            if method[2]:
                threshold = 1.0 / method[2]
                for time in (math.nextafter(threshold, 0.0), threshold, math.nextafter(threshold, math.inf)):
                    for factor in (0.5, math.nextafter(1.0, 0.0), 1.0, math.nextafter(1.0, math.inf), 2.0):
                        yield operation, method, 0.0, time, factor, method
    elif operation == "equivalent":
        for source in oracle.METHODS:
            for target in oracle.METHODS:
                for time in (-oracle.MIN_SUB, -1.0):
                    yield operation, source, 0.05, time, 1.0, target
                counts = sorted({count for count in (source[2], target[2]) if count})
                for count in counts:
                    threshold = 1.0 / count
                    for time in (math.nextafter(threshold, 0.0), threshold, math.nextafter(threshold, math.inf)):
                        for rate in (-0.005, 0.05):
                            yield operation, source, rate, time, 1.0, target
    else:
        for method in oracle.METHODS:
            for time in (-oracle.MIN_SUB, -1.0):
                yield operation, method, 0.05, time, 1.0, method
        for method in oracle.METHODS:
            for rate, boundary in ((0.05, math.log(oracle.MAX)),
                                   (1.0, math.log(oracle.MAX)),
                                   (-0.05, math.log(oracle.MIN_SUB) - math.log(2.0)),
                                   (-0.05, -math.log(oracle.MAX))):
                annual = rate if method[0] == "Continuous" else method[2] * math.log1p(rate/method[2]) if method[2] else rate
                center = boundary / annual
                if oracle.phase(method[0], method[2], center) == "Simple":
                    continue
                center_bits = int.from_bytes(struct.pack(">d", center), "big")
                for offset in (0, -1, 1, -4, 4, -8, 8, -16, 16, -32, 32, -64, 64):
                    time = struct.unpack(">d", (center_bits + offset).to_bytes(8, "big"))[0]
                    yield operation, method, rate, time, 1.0, method
    for method in oracle.METHODS:
        for invalid in (math.nan, math.inf, -math.inf):
            for time in (0.0, 1.0):
                yield operation, method, 0.0 if operation == "implied" else invalid, time, invalid if operation == "implied" else 1.0, method
            yield operation, method, 0.0 if operation == "implied" else 0.05, invalid, 1.1 if operation == "implied" else 1.0, method


def expected(case, precision):
    operation, _, rate, time, factor, _ = case
    if not math.isfinite(time) or not math.isfinite(factor if operation == "implied" else rate):
        return "error", "rate.value", ""
    return oracle.expected(case, precision)


def guard_information(case, precision):
    """Separate true arithmetic from the computed-logarithm guard policy.

    The 6..10-S annulus is a measured local status tier, not a formal libm
    bound and not a change to the E3 value tolerance or the 8-S guard.
    """
    operation, source, rate, time, _, _ = case
    empty = ("", "0.0", "0.0", "0.0", "", "0.0", "", "")
    if operation not in ("compound", "discount") or not math.isfinite(rate) or not math.isfinite(time) or time <= 0 or rate == 0:
        return empty
    if source[2] and rate <= -source[2] or oracle.phase(source[0], source[2], time) == "Simple":
        return empty
    with localcontext() as context:
        context.prec = precision
        logarithm = oracle.log_growth(rate, time, source)
        factor = float(oracle.factor_from_log(logarithm))
        valid = math.isfinite(factor) and factor > 0
        value = factor
        if operation == "discount" and valid:
            try:
                value = float(Fraction(1) / Fraction.from_float(factor))
            except OverflowError:
                value = math.inf
            valid = math.isfinite(value) and value > 0
        candidates = [(logarithm, D.from_float(oracle.MAX).ln()),
                      (logarithm, D.from_float(oracle.MIN_SUB).ln() - D(2).ln())]
        if operation == "discount" and math.isfinite(factor) and factor > 0:
            candidates.append((-D.from_float(factor).ln(), D.from_float(oracle.MAX).ln()))
        scored = [(abs(argument - boundary) / (D(2) ** -53 * (1 + abs(argument))), argument, boundary)
                  for argument, boundary in candidates]
        score, argument, boundary = min(scored, key=lambda item: abs(item[0] - 8))
        classification = "D5-RANGE-GUARD-EDGE" if D(6) <= score <= D(10) and not any(item[0] < 6 for item in scored) else ""
        return (classification, repr(float(score)), repr(float(argument)), repr(float(boundary)), ",".join(repr(float(item[0])) for item in scored), repr(factor),
                "value" if valid else "error", repr(value) if valid else "")


def invoke(function):
    try:
        value = float(function())
        return ("value" if math.isfinite(value) else "nonfinite"), repr(value), ""
    except RuntimeError as error:
        return "raises", "", " ".join(str(error).split())


def equal(left, right):
    return left == right or math.isnan(left) and math.isnan(right)


def rounded_exp(growth):
    if growth > 1000:
        return math.inf
    if growth < -1000:
        return 0.0
    return float(growth.exp())


def decimal_power(base, exponent, precision):
    """Correctly rounded model of pow's already-rounded binary64 arguments."""
    if exponent == 0.0 or base == 1.0:
        return 1.0
    if base == 0.0:
        return 0.0 if exponent > 0 else math.inf
    if math.isinf(exponent):
        return math.inf if base > 1 else 0.0
    if math.isinf(base):
        return math.inf if exponent > 0 else 0.0
    with localcontext() as context:
        context.prec = precision
        return rounded_exp(D.from_float(base).ln() * D.from_float(exponent))


def power(base, exponent):
    try:
        return math.pow(base, exponent)
    except OverflowError:
        return math.inf


def divide(numerator, denominator):
    if denominator == 0.0:
        return math.copysign(math.inf, numerator) if numerator else math.nan
    return numerator / denominator


def rate_bound(operation, expected, time, method):
    if expected == 0.0:
        return 0.0
    kind = oracle.phase(method[0], method[2], time)
    extra = 0.0
    if kind == "Compounded":
        count = method[2]
        scaled = expected / count
        if abs(scaled) < sys.float_info.epsilon:
            extra = UNIT * abs(expected)
        else:
            argument = math.log((count + expected) / count) if scaled <= -0.5 else math.log1p(scaled)
            extra = (UNIT * abs(argument)) * (count + expected)
    elif operation == "equivalent" and kind == "Simple":
        product = expected * time
        if product <= -1.0:
            extra = 0.0
        elif abs(product) < sys.float_info.epsilon:
            extra = UNIT * abs(expected)
        else:
            argument = math.log(expected) + math.log(time) if math.isinf(product) else math.log1p(product)
            extra = (UNIT * abs(argument)) * abs(expected) + (UNIT * abs(argument)) / time
    return 8.0 * (UNIT * abs(expected) + divide(UNIT, time) + extra)


def factor_matches(actual, expected):
    if not math.isfinite(expected) or expected <= 0:
        return equal(actual, expected)
    if not math.isfinite(actual):
        return False
    absolute = oracle.MIN_SUB if expected < sys.float_info.min else 0.0
    relative = 8.0 * UNIT * (1.0 + abs(math.log(expected)))
    return abs(actual - expected) <= absolute + abs(expected) * relative


def fits(operation, actual, expected, time, method):
    if operation in ("compound", "discount"):
        return factor_matches(actual, expected)
    if not math.isfinite(actual):
        return False
    return abs(actual - expected) <= rate_bound(operation, expected, time, method)


def source_model(source, rate, time):
    """Model public factors without using a QuantLib result as the oracle."""
    kind = oracle.phase(source[0], source[2], time)
    if kind == "Compounded":
        base, exponent = 1.0 + rate / source[2], source[2] * time
        low, high = decimal_power(base, exponent, 120), decimal_power(base, exponent, 200)
        if not equal(low, high):
            raise ArithmeticError("unstable rounded-base model")
        return base, exponent, high
    with localcontext() as context:
        context.prec = 200
        if kind == "Simple":
            # This wheel retains the fused residual; do not assume that every
            # public C++ build rounds the product separately.
            value = 1 + Fraction.from_float(rate) * Fraction.from_float(time)
            try:
                binary = float(value)
            except OverflowError:
                binary = math.inf if value > 0 else -math.inf
            return 0.0, 0.0, binary
        exponent = rate * time
        return 0.0, exponent, rounded_exp(D.from_float(exponent)) if math.isfinite(exponent) else (math.inf if exponent > 0 else 0.0)


def inverse_model(factor, time, target):
    """Expose pow rounding and the subtraction that loses small increments.

    Python's public libm pow is evidence of the binary64 rounding stage,
    separately checked against a two-precision Decimal power. It is never
    the oracle for the project implementation.
    """
    if factor == 1.0:
        return 0.0, 1.0, 1.0, 0.0
    kind = oracle.phase(target[0], target[2], time)
    if kind == "Compounded":
        exponent = divide(1.0, target[2] * time)
        low, high = decimal_power(factor, exponent, 120), decimal_power(factor, exponent, 200)
        if not equal(low, high):
            raise ArithmeticError("unstable inverse power model")
        observed_power = power(factor, exponent)
        if not factor_matches(observed_power, high):
            raise ArithmeticError("libm power differs from independent Decimal model")
        return exponent, observed_power, high, (observed_power - 1.0) * target[2]
    if kind == "Simple":
        return 0.0, 0.0, 0.0, divide(factor - 1.0, time)
    return 0.0, 0.0, 0.0, divide(math.log(factor), time)


def violates_rate(value, time, target):
    if not math.isfinite(value):
        return "D9-NONFINITE-RATE"
    if target[2] and value <= -target[2]:
        return "D9-INFERRED-BASE"
    if time > 0 and oracle.phase(target[0], target[2], time) == "Simple":
        with localcontext() as context:
            context.prec = 200
            if float(1 + D.from_float(value) * D.from_float(time)) <= 0:
                return "D9-INFERRED-SIMPLE"
    return ""


def classify(case, expected, reference, source_factor):
    operation, source, rate, time, factor, target = case
    behavior, _, value = expected
    ref_behavior, ref_value, _ = reference
    if ref_behavior == "raises":
        if behavior == "error":
            return ""
        if operation == "equivalent" and (not math.isfinite(source_factor) or source_factor <= 0):
            return "D5-INTERMEDIATE-FACTOR"
        raise ArithmeticError("unclassified reference exception: " + str((case, reference)))
    actual = float(ref_value)
    if behavior == "error":
        if not math.isfinite(rate) and operation != "implied":
            return "D9-QUOTE-NONFINITE"
        if not math.isfinite(time):
            return "D9-TIME-NONFINITE"
        if operation == "implied" and not math.isfinite(factor):
            return "D9-FACTOR-INPUT"
        if expected[1] == "rate.numerical_boundary":
            return "D9-NUMERICAL-BOUNDARY"
        if source[2] and operation != "implied" and rate <= -source[2]:
            return "D9-QUOTE-BASE"
        if operation in ("compound", "discount"):
            if not math.isfinite(actual) or actual <= 0:
                return "D9-FACTOR-RESULT"
            # An erased rounded base can return 1 when exact growth overflows.
            if oracle.phase(source[0], source[2], time) == "Compounded" and 1.0 + rate/source[2] == 1.0 and rate != 0:
                return "D5-BASE-ERASURE"
            if oracle.phase(source[0], source[2], time) == "Compounded":
                return "D5-ROUNDED-BASE-RANGE"
        else:
            invalid = violates_rate(actual, time, target if operation == "equivalent" else source)
            if invalid:
                return invalid
            if operation == "equivalent":
                if not math.isfinite(source_factor) or source_factor <= 0:
                    return "D5-INTERMEDIATE-FACTOR"
                if source_factor == 1.0 and rate != 0 and time > 0:
                    return "D5-FACTOR-ERASURE"
            elif actual == 0.0 and factor != 1.0 and time > 0:
                return "D9-RATE-UNDERFLOW"
        raise ArithmeticError("unclassified accepted error case: " + str((case, expected, reference)))
    expected_value = float(value)
    if operation == "equivalent" and (not math.isfinite(source_factor) or source_factor <= 0):
        return "D5-INTERMEDIATE-FACTOR"
    selector = target if operation == "equivalent" else source
    lost = expected_value != 0.0 and actual == 0.0
    if not lost and fits(operation, actual, expected_value, time, selector):
        return ""
    if operation in ("compound", "discount") and oracle.phase(source[0], source[2], time) == "Compounded":
        return "D5-ROUNDED-BASE"
    if operation == "implied" and oracle.phase(source[0], source[2], time) == "Compounded":
        return "D5-POW-SUBTRACTION"
    if operation == "equivalent":
        if source_factor == 1.0 and rate != 0.0 and time > 0:
            return "D5-FACTOR-ERASURE"
        if oracle.phase(target[0], target[2], time) == "Compounded":
            return "D5-POW-SUBTRACTION"
        if oracle.phase(source[0], source[2], time) == "Compounded":
            return "D5-ROUNDED-BASE-ROUNDTRIP"
        return "D5-FACTOR-ROUNDTRIP"
    raise ArithmeticError("unclassified numeric difference: " + str((case, expected, reference)))


def attest(ql, args, parser):
    if ql.__version__ != "1.43" or not args.quantlib_wheel.name.lower().startswith("quantlib-1.43-"):
        parser.error("runtime and wheel must be QuantLib 1.43")
    installed = pathlib.Path(ql.__file__).resolve().parent.parent
    with zipfile.ZipFile(args.quantlib_wheel) as wheel:
        members = [name for name in wheel.namelist() if name.startswith("QuantLib/") and not name.endswith("/")]
        if not members:
            parser.error("wheel contains no QuantLib module")
        for name in members:
            if hashlib.sha256((installed / name).read_bytes()).digest() != hashlib.sha256(wheel.read(name)).digest():
                parser.error("installed runtime differs from wheel: " + name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=pathlib.Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    import QuantLib as ql
    attest(ql, args, parser)
    columns = ("id", "operation", "method", "frequency", "rate", "time", "factor", "target_method", "target_frequency",
               "behavior", "error_kind", "expected", "tolerance", "reference_behavior", "reference_value", "reference_error",
               "source_factor", "rounded_base", "rounded_exponent", "source_model", "inverse_exponent", "inverse_power", "inverse_power_model",
               "inverse_model", "reference_implied", "discrepancy", "guard_classification", "guard_edge_score", "guard_argument", "guard_boundary", "guard_scores",
               "mathematical_factor", "mathematical_behavior", "mathematical_expected")
    for operation in ("compound", "discount", "implied", "equivalent"):
        lines, counts, guard_counts = ["\t".join(columns)], collections.Counter(), collections.Counter()
        for index, case in enumerate(cases(operation)):
            _, source, rate, time, factor, target = case
            low, high = expected(case, 120), expected(case, 200)
            if low != high:
                raise ArithmeticError("unstable project oracle: " + str((case, low, high)))
            behavior, error, expected_value = high
            guard_low, guard_high = guard_information(case, 120), guard_information(case, 200)
            if guard_low[0] != guard_high[0] or guard_low[-2:] != guard_high[-2:] or guard_high[0] and guard_low != guard_high:
                raise ArithmeticError("unstable independent guard/math information: " + str((case, guard_low, guard_high)))
            if guard_high[0]:
                guard_counts[guard_high[-2]] += 1
            base = exponent = model = source_factor = inverse_exponent = inverse_power = inverse_power_model = inverse = 0.0
            reference_implied = ""
            if operation == "implied":
                reference = invoke(lambda: ql.InterestRate.impliedRate(factor, ql.Actual365Fixed(), *config(ql, source), time).rate())
                if math.isfinite(time) and math.isfinite(factor) and time > 0 and factor > 0:
                    inverse_exponent, inverse_power, inverse_power_model, inverse = inverse_model(factor, time, source)
            else:
                quote = ql.InterestRate(rate, ql.Actual365Fixed(), *config(ql, source))
                reference_factor = invoke(lambda: quote.compoundFactor(time))
                if reference_factor[0] != "raises":
                    source_factor = float(reference_factor[1])
                if math.isfinite(rate) and math.isfinite(time) and (not source[2] or rate > -source[2]):
                    base, exponent, model = source_model(source, rate, time)
                    if reference_factor[0] != "raises" and not factor_matches(source_factor, model):
                        raise ArithmeticError("reference compound differs from rounded-input model: " + str((case, reference_factor, model)))
                reference = reference_factor if operation == "compound" else invoke(lambda: quote.discountFactor(time)) if operation == "discount" else invoke(lambda: quote.equivalentRate(*config(ql, target), time).rate())
                if operation == "equivalent" and math.isfinite(rate) and math.isfinite(time) and source_factor > 0 and time > 0:
                    ref_inverse = invoke(lambda: ql.InterestRate.impliedRate(source_factor, ql.Actual365Fixed(), *config(ql, target), time).rate())
                    if ref_inverse[:2] != reference[:2]:
                        raise ArithmeticError("equivalent does not match public compound-to-implied pipeline")
                    reference_implied = ref_inverse[1]
                    inverse_exponent, inverse_power, inverse_power_model, inverse = inverse_model(source_factor, time, target)
            if operation in ("implied", "equivalent") and math.isfinite(time) and math.isfinite(factor if operation == "implied" else rate) and reference[0] != "raises" and time > 0 and (operation == "implied" and factor > 0 or operation == "equivalent" and source_factor > 0):
                if not equal(float(reference[1]), inverse):
                    raise ArithmeticError("reference inferred rate differs from independently checked binary rounding pipeline: " + str((case, reference, inverse)))
            discrepancy = classify(case, high, reference, source_factor)
            counts[discrepancy or "agreement"] += 1
            row = ("RATE-QL-%s-%06d" % (operation.upper(), index), operation, source[0], source[1], repr(rate), repr(time), repr(factor), target[0], target[1],
                   behavior, error, expected_value, "TOL-E3" if operation in ("compound", "discount") else "TOL-E4", *reference,
                   repr(source_factor), repr(base), repr(exponent), repr(model), repr(inverse_exponent), repr(inverse_power), repr(inverse_power_model), repr(inverse), reference_implied, discrepancy, *guard_high)
            lines.append("\t".join(row))
        metadata = {
            "schema": "1", "fixture-set": "RATE-QUANTLIB-" + operation.upper(), "domain": "rates",
            "description": "All 32 configurations, hybrid threshold neighbors, tiny/base-boundary/negative/zero/extreme binary64 inputs; explicit rounding-pipeline classifications",
            "reference": "QuantLib", "reference-version": ql.__version__,
            "reference-source": "Attested wheel public InterestRate APIs; independently checked Decimal equations and rounded-input power stages; no Rust outputs",
            "tolerance": "Unchanged TOL-E3/E4 k=8; E4 includes derived exponential argument conditioning; D5 source rounding never expands the project tolerance",
            "reproduction": "generator-sha256=" + REVISION + "; python3 validation/generator/rate_quantlib_fixtures.py --quantlib-wheel " + args.quantlib_wheel.name,
            "oracle-generator-sha256": ORACLE_REVISION,
            "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
            "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
            "quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": hashlib.sha256(args.quantlib_wheel.read_bytes()).hexdigest(),
            "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995; wheel source commit not independently attested",
            "classification-counts": "; ".join(key + "=" + str(value) for key, value in sorted(counts.items())),
            "guard-edge-counts": "; ".join(key + "=" + str(value) for key, value in sorted(guard_counts.items())) or "none",
        }
        body = "\n".join(lines) + "\n"
        path = DEST / (operation + "-quantlib.fixture")
        if args.check:
            header, old_body = path.read_text().split("\n\n", 1)
            old = dict(line.split(": ", 1) for line in header.splitlines())
            stable = metadata.keys() - {"generated-at", "platform", "python-version"}
            if old_body != body or any(old.get(key) != metadata[key] for key in stable):
                raise SystemExit("fixture differs: " + str(path))
            action = "verified"
        else:
            DEST.mkdir(parents=True, exist_ok=True)
            path.write_bytes(("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n" + body).encode())
            action = "generated"
        print(action, path.relative_to(ROOT), "records=" + str(len(lines) - 1), dict(counts), "guard-edge=" + str(dict(guard_counts)), flush=True)


if __name__ == "__main__":
    main()
