#!/usr/bin/env python3
"""Exact Fraction rounding oracles for independent binary64 sum validation.

No Rust output, limb accumulator, compensated algorithm or reference-library
result is used to compute expected sums. Python Fraction retains each supplied
float exactly and Python's rational-to-float conversion rounds once.
"""

import argparse
import datetime as dt
from fractions import Fraction
import hashlib
import math
from pathlib import Path
import platform
import random
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/math/sum-oracle.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
MIN_SUB = float.fromhex("0x0.0000000000001p-1022")
MAX = sys.float_info.max
UNIT = Fraction(1, 2**53)
# Public diagnostic counterexamples: exact prefix rounding reaches infinity
# before later cancellation leaves a finite final sum. This proof computes
# rational prefixes, not the Rust/Neumaier correction loop.
PREFIX_PROOFS = {3: (1, 1), 5: (1, -1), 8: (1, 1),
                 102294: (1, -1), 102299: (1, 1),
                 105302: (2097, 1), 105305: (53, 1)}


def bits(value):
    return struct.unpack(">Q", struct.pack(">d", value))[0]


def from_bits(value):
    return struct.unpack(">d", struct.pack(">Q", value))[0]


def round_rational(value):
    try:
        return float(value)
    except OverflowError:
        return -math.inf if value < 0 else math.inf


def vectors():
    yield "empty", []
    yield "signed-zero", [-0.0, 0.0, -0.0]
    yield "cancellation", [1e16, 1.0, -1e16, 1.0]
    yield "intermediate-overflow", [MAX, MAX, -MAX, -MAX, MIN_SUB]
    yield "minimum-normal", [sys.float_info.min, -MIN_SUB]
    half_overflow_ulp = math.ldexp(1.0, 970)
    for sign in [-1, 1]:
        for delta in [-MIN_SUB, 0.0, MIN_SUB]:
            yield "overflow-tie", [sign*MAX, sign*half_overflow_ulp, sign*delta]
    # Every exponent, including subnormal mantissas; boundary pairs exercise
    # ties both above/below even and odd significands. Each input is binary64.
    for exponent in range(2047):
        for fraction in [0, 1, (1 << 51) - 1, 1 << 51, (1 << 52) - 1]:
            value = from_bits((exponent << 52) | fraction)
            quantum = math.ulp(value)
            half = quantum / 2
            for sign in [-1, 1]:
                yield "singleton", [sign*value]
                yield "cancel-small", [sign*value, MIN_SUB, -sign*value]
                if half:
                    yield "round-even-tie", [sign*value, sign*half]
                    yield "round-tie-below", [sign*value, sign*half, -sign*MIN_SUB]
                    yield "round-tie-above", [sign*value, sign*half, sign*MIN_SUB]
    rng = random.Random(0x5AB16006)
    for index in range(3000):
        count = 2 + index % 63
        values = []
        for _ in range(count):
            raw = rng.getrandbits(64)
            if (raw >> 52) & 0x7ff == 0x7ff:
                raw ^= 1 << 52
            values.append(from_bits(raw))
        if index % 3 == 0:
            values = values + [-v for v in values] + [MIN_SUB]
        elif index % 3 == 1:
            values += [-values[0], -values[-1]]
        rng.shuffle(values)
        yield "seeded-vector", values
    # Long carry/borrow chains and sums whose final sign differs from the
    # sign of the largest element. Powers cover every binary64 bit position.
    powers = [math.ldexp(1.0, exponent) for exponent in range(-1074, 1024)]
    for order in [powers, list(reversed(powers))]:
        yield "all-powers", order
        yield "all-powers-cancel", order + [-x for x in order] + [MIN_SUB]
        yield "all-powers-borrow", [MAX] + [-x for x in order]


def render(generated_at):
    header = [
        "schema: 1",
        "fixture-set: MATH-SUM-ORACLE",
        "domain: math",
        "description: exact rational sums of binary64 inputs; all exponents, midpoint ties, carry/borrow, overflow and seeded cancellation",
        "reference: oracle:Python Fraction",
        "reference-version: Python " + platform.python_version(),
        "reference-source: https://docs.python.org/3.9/library/fractions.html; exact sum of supplied binary64 rationals, rounded once",
        "tolerance: TOL-E0 exact_sum 0 ULP; TOL-E5-SUM compensated_sum empirical conditioned absolute budget, not a universal certificate",
        "reproduction: generator-sha256=" + REVISION + "; python3 validation/generator/sum_oracles.py",
        "generated-at: " + generated_at,
        "platform: " + platform.system() + " " + platform.machine(),
        "random-seed: 0x5ab16006; Python random.Random; finite binary64 bit patterns",
        "compensated-prefix-proof: seven named finite-final-sum cases; independently exact Fraction prefixes prove first overflow index/sign; consumer checks quantity and context per row",
    ]
    rows = []
    proved = set()
    for index, (family, values) in enumerate(vectors()):
        fractions = [Fraction.from_float(value) for value in values]
        exact = sum(fractions, Fraction(0))
        expected = round_rational(exact)
        if index in PREFIX_PROOFS:
            assert math.isfinite(expected), "prefix counterexample must have finite final truth"
            prefix = Fraction(0)
            for position, value in enumerate(fractions):
                prefix += value
                partial = round_rational(prefix)
                if not math.isfinite(partial):
                    actual = (position, -1 if partial < 0 else 1)
                    assert actual == PREFIX_PROOFS[index], (index, actual)
                    proved.add(index)
                    break
            else:
                raise ArithmeticError("missing independently overflowing prefix")
        total_absolute = sum(map(abs, fractions), Fraction(0))
        # Original conservative calibration scale. The exact-sum assertion
        # never uses this budget. It is independently evaluated as a rational
        # to avoid saturated intermediate arithmetic or vanished n*u^2 terms.
        budget = 8*UNIT*abs(exact) + 8*len(values)*UNIT*UNIT*total_absolute
        budget = max(MIN_SUB, round_rational(budget))
        rows.append("\t".join([
            f"SUM-{index:06d}", family,
            ";".join(f"{bits(value):016x}" for value in values),
            "value" if math.isfinite(expected) else "error",
            "" if math.isfinite(expected) else "sum.arithmetic",
            f"0x{bits(expected if math.isfinite(expected) else 0.0):016x}",
            repr(budget),
            "" if math.isfinite(expected) else ("negative" if expected < 0 else "positive"),
        ]))
    assert proved == set(PREFIX_PROOFS)
    return "\n".join(header) + "\n\nid\tfamily\tinput_bits\tbehavior\terror_kind\texpected\tcompensated_absolute_budget\toverflow_sign\n" + "\n".join(rows) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        generated_at = next(line.split(": ", 1)[1] for line in previous.splitlines()
                            if line.startswith("generated-at: "))
        if render(generated_at) != previous:
            raise SystemExit("sum oracle is stale; regenerate and review")
        print("sum oracle reproducible:", len(list(vectors())), "records")
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_text(render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")))
        print("sum oracle written:", len(list(vectors())), "records")


if __name__ == "__main__":
    main()
