#!/usr/bin/env python3
"""Independent logarithmic range-policy probes and exact rational anchors."""

import argparse
import datetime as dt
from decimal import Decimal as D, localcontext
from fractions import Fraction
import hashlib
import importlib.util
import math
import pathlib
import platform
import struct
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/rates/boundaries-oracle.fixture"
DEPENDENCY = pathlib.Path(__file__).with_name("rate_oracles.py")
SPEC = importlib.util.spec_from_file_location("rate_oracles", DEPENDENCY)
ORACLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ORACLE)
# Deliberately separate clear inner/outer samples from the uncertain edge of
# the declared band. A Rust test independently scans the actual binary64
# continuous exponent at every adjacent value through the band edge.
OFFSETS = (0, -1, 1, -2, 2, -32, 32, -64, 64, -128, 128,
           -512, 512, -4096, 4096)


def neighbor(value, offset):
    """Select a positive finite binary64 neighbor without accumulated rounding."""
    bits = struct.unpack(">Q", struct.pack(">d", value))[0] + offset
    if bits <= 0 or bits >= 0x7ff0000000000000:
        return None
    return struct.unpack(">d", struct.pack(">Q", bits))[0]


def probes():
    """Yield (case, category, boundary, offset, optional exact compound factor)."""
    for method in ORACLE.METHODS:
        if method[0] == "Simple":
            continue
        rates = [-0.005, -0.05, -0.5, 0.005, 0.05, 1.0, 10.0, 100.0,
                 ORACLE.MAX, -ORACLE.MAX]
        if method[2]:
            rates.append(math.nextafter(-float(method[2]), math.inf))
        for rate in sorted(set(rates)):
            if method[2] and rate <= -method[2]:
                continue
            # Pick centers with Decimal annual growth independently of Rust's
            # operation order. Hybrid branch selection is evaluated later at
            # each actual binary64 horizon, including threshold crossings.
            logarithmic = ("Continuous", "", 0) if method[0] == "Continuous" else (
                "Compounded", method[1], method[2])
            with localcontext() as context:
                context.prec = 200
                growth = ORACLE.log_growth(rate, 1.0, logarithmic)
                targets = (("compound-overflow", D.from_float(ORACLE.MAX).ln()),) if rate > 0 else (
                    ("reciprocal-overflow", -D.from_float(ORACLE.MAX).ln()),
                    ("compound-zero", D.from_float(ORACLE.MIN_SUB).ln()-D(2).ln()))
                centers = [(name, float(target/growth)) for name, target in targets]
            for boundary, center in centers:
                if not math.isfinite(center) or center <= 0:
                    continue
                for offset in OFFSETS:
                    time = neighbor(center, offset)
                    if time is None:
                        continue
                    for operation in ("compound", "discount"):
                        case = operation, method, rate, time, 1.0, method
                        yield case, "log-neighbor", boundary, offset, None

    # Every periodic frequency and both hybrid switch directions, including
    # enormous positive rates and the closest valid negative periodic rate.
    for method in ORACLE.METHODS:
        if not method[2]:
            continue
        threshold = 1.0/method[2]
        for rate in [-0.05, 0.05, ORACLE.MAX,
                     math.nextafter(-float(method[2]), math.inf)]:
            for offset in (-1, 0, 1):
                time = neighbor(threshold, offset)
                for operation in ("compound", "discount"):
                    yield (operation, method, rate, time, 1.0, method), "horizon-switch", "1/frequency", offset, None

    # Simple paths use fused arithmetic and intentionally have no logarithmic
    # uncertainty policy, even when their result is very close to MAX.
    simple = ORACLE.METHODS[0]
    for rate in (-0.005, -0.05, -0.5, -1.0, -10.0, -ORACLE.MAX,
                 1.0, 2.0, 100.0, ORACLE.MAX):
        center = -1.0/rate if rate < 0 else ORACLE.MAX/rate
        if not math.isfinite(center) or center <= 0:
            continue
        for offset in OFFSETS:
            time = neighbor(center, offset)
            if time is None:
                continue
            exact = Fraction(1)+Fraction.from_float(rate)*Fraction.from_float(time)
            for operation in ("compound", "discount"):
                yield (operation, simple, rate, time, 1.0, simple), "simple-range", "fused-zero" if rate < 0 else "fused-overflow", offset, exact
    fused_rate, fused_time = -1.0-math.ulp(1.0), 1.0-math.ulp(1.0)
    exact = Fraction(1)+Fraction.from_float(fused_rate)*Fraction.from_float(fused_time)
    assert exact == Fraction(1, 2**104)
    for operation in ("compound", "discount"):
        yield (operation, simple, fused_rate, fused_time, 1.0, simple), "simple-fused", "2^-104", 0, exact

    # Integer powers avoid Decimal/libm ambiguity at overflow and the exact
    # halfway-to-zero tie. Fraction conversion rounds 2^-1075 to even zero.
    for method in ORACLE.METHODS:
        if method[:2] not in (("Compounded", "Annual"),
                              ("SimpleThenCompounded", "Annual")):
            continue
        for power in (-1076, -1075, -1074, -1025, -1024, -1023,
                      1022, 1023, 1024, 1025):
            rate, time = (-0.5, float(-power)) if power < 0 else (1.0, float(power))
            exact = Fraction(2**power) if power >= 0 else Fraction(1, 2**-power)
            for operation in ("compound", "discount"):
                yield (operation, method, rate, time, 1.0, method), "exact-power2", str(power), 0, exact


def cases():
    """Public compact API for optional separately attested reference probes."""
    for case, _, _, _, _ in probes():
        yield case


def rational_result(exact, operation):
    """Round C once, then one reciprocal, matching the declared factor policy."""
    try:
        factor = float(exact)
    except OverflowError:
        return "error", ""
    if not math.isfinite(factor) or factor <= 0:
        return "error", ""
    if operation == "discount":
        try:
            factor = float(1/Fraction.from_float(factor))
        except OverflowError:
            return "error", ""
        if not math.isfinite(factor) or factor <= 0:
            return "error", ""
    return "value", repr(factor)


def short_decimal(value):
    with localcontext() as context:
        context.prec = 25
        return str(+value)


def records():
    for index, (case, category, boundary, offset, exact) in enumerate(probes()):
        operation, method, rate, time, _, _ = case
        phase = ORACLE.phase(method[0], method[2], time)
        if phase == "Simple" and exact is not None:
            # A finite Decimal precision can straddle an exact binary64 FMA
            # midpoint. Fraction, rather than either precision, resolves it.
            behavior, value = rational_result(exact, operation)
            error = "rate.value" if behavior == "error" else ""
        else:
            low, high = ORACLE.expected(case, 120), ORACLE.expected(case, 200)
            if low != high:
                raise ArithmeticError("unstable policy oracle: " + repr((case, low, high)))
            behavior, error, value = high
        logarithm = ""
        if phase != "Simple":
            with localcontext() as context:
                context.prec = 200
                logarithm = short_decimal(ORACLE.log_growth(rate, time, method))
        if exact is not None:
            mathematical_behavior, mathematical_value = rational_result(exact, operation)
            origin = "Python Fraction exact binary64 arithmetic"
            numerator, denominator = str(exact.numerator), str(exact.denominator)
            # Fraction is a separate check for the exponential power anchors.
            if error != "rate.numerical_boundary" and (behavior, value) != (mathematical_behavior, mathematical_value):
                raise ArithmeticError("Decimal/Fraction disagreement: " + repr(case))
        elif error == "rate.numerical_boundary":
            # No claim about mathematical binary64 rounding inside the band:
            # two agreeing precisions alone do not resolve an exact midpoint.
            mathematical_behavior, mathematical_value = "not-classified", ""
            origin, numerator, denominator = "Decimal 120/200 policy", "", ""
        else:
            mathematical_behavior, mathematical_value = behavior, value
            origin, numerator, denominator = "Decimal 120/200 stable output", "", ""
        yield (
            "RATE-BOUNDARY-%06d" % index, operation, method[0], method[1],
            repr(rate), repr(time), category, boundary, str(offset), phase,
            logarithm, behavior, error, value, "TOL-E3", origin,
            mathematical_behavior, mathematical_value, numerator, denominator)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    columns = ("id", "operation", "method", "frequency", "rate", "time",
               "category", "boundary", "offset", "phase", "logarithm",
               "behavior", "error_kind", "expected", "tolerance", "origin",
               "mathematical_behavior", "mathematical_expected",
               "exact_compound_numerator", "exact_compound_denominator")
    lines, counts = ["\t".join(columns)], {}
    for row in records():
        lines.append("\t".join(row))
        key = row[12] or row[11]
        counts[key] = counts.get(key, 0)+1
    metadata = {
        "schema": "1", "fixture-set": "RATE-BOUNDARIES-ORACLE", "domain": "rates",
        "description": "FS-RATE/FS-DISC logarithmic range policy; adjacent horizons; exact rational ties and fused Simple exceptions",
        "reference": "oracle:Python Decimal and Fraction",
        "reference-version": platform.python_version()+" Decimal 120/200 digits and exact Fraction",
        "reference-source": "FS-RATE equations and declared 8u logarithmic boundary policy; no Rust or QuantLib outputs",
        "tolerance": "Exact policy statuses on documented grid; finite factors TOL-E3 k=8 plus one minimum subnormal absolute unit",
        "reproduction": "generator-sha256="+hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()+"; rate-oracles-sha256="+hashlib.sha256(DEPENDENCY.read_bytes()).hexdigest()+"; python3 validation/generator/rate_boundaries.py",
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system()+" "+platform.machine(),
        "boundary-policy": "8*2^-53*(1+abs(log)); exp C around ln(MAX) and ln(MIN_SUB)-ln(2); inverse C around ln(MAX); Simple exempt",
        "grid": "Positive finite horizon offsets 0,+/-1,+/-2,+/-32,+/-64,+/-128,+/-512,+/-4096 binary64 neighbors; switch offsets -1,0,+1",
        "precision-limitation": "Inside a declared numerical-boundary band, mathematical rounding is unclassified unless an exact Fraction anchor is supplied",
    }
    body = "\n".join(lines)+"\n"
    if args.check:
        header, old_body = DEST.read_text().split("\n\n", 1)
        old = dict(line.split(": ", 1) for line in header.splitlines())
        stable = metadata.keys()-{"generated-at", "platform"}
        if old_body != body or any(old.get(key) != metadata[key] for key in stable):
            raise SystemExit("fixture differs: " + str(DEST))
        action = "verified"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(("\n".join(key+": "+value for key, value in metadata.items())+"\n\n"+body).encode())
        action = "generated"
    print(action, DEST.relative_to(ROOT), "records="+str(len(lines)-1), counts)


if __name__ == "__main__":
    main()
