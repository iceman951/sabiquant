#!/usr/bin/env python3
"""One-round exact Fraction oracles for three supplied binary64 factors.

Expected results use Fraction.from_float for each original operand followed
by exactly one Python rational-to-binary64 conversion. No Rust result, limb
arithmetic, logarithm, exponent scaling algorithm or intermediate floating
product supplies the truth. These are arithmetic observations, not claims
that arbitrary factors are financially valid coupon inputs.
"""

import argparse
from collections import Counter
import datetime as dt
from fractions import Fraction
import hashlib
import itertools
import math
from pathlib import Path
import platform
import random
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/cashflows/product-oracle.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
MIN_SUB = float.fromhex("0x0.0000000000001p-1022")
MIN_NORMAL = sys.float_info.min
MAX = sys.float_info.max
SEED = 0x5AB17008


def bits(value):
    return struct.unpack(">Q", struct.pack(">d", value))[0]


def from_bits(value):
    return struct.unpack(">d", struct.pack(">Q", value))[0]


def rounded(value):
    try:
        return float(value)
    except OverflowError:
        return -math.inf if value < 0 else math.inf


def signs(values):
    for mask in range(8):
        yield tuple(-value if mask & (1 << index) else value
                    for index, value in enumerate(values))


def triples():
    # Exact zero bypasses domain rejection and must be canonical positive zero
    # regardless of operand signs or whether ordinary multiplication overflows.
    for zero in [0.0, -0.0]:
        for values in [(MAX, MAX), (MIN_SUB, MIN_SUB), (MAX, MIN_SUB)]:
            for signed in signs((zero, *values)):
                for permutation in itertools.permutations(signed):
                    yield "exact-zero", permutation

    hand = [
        ("rescued-overflow", (MAX, 2.0, 0.5)),
        ("rescued-underflow", (MIN_SUB, 0.5, 2.0)),
        ("rescued-underflow", (MIN_NORMAL, MIN_NORMAL, MAX)),
        ("normal-midpoint", (1.0 + 2.0**-27, 1.0 - 2.0**-27, 1.0)),
        ("normal-midpoint", (math.nextafter(1.0, math.inf), 1.5, 1.0)),
        ("normal-midpoint", (1.0 + 2.0**-51, 1.25, 1.0)),
        ("normal-carry", (math.nextafter(2.0, 0.0), 1.0 + 2.0**-52, 1.0)),
        ("normal-subnormal-midpoint", (MIN_SUB, float(2**53 - 1), 0.5)),
        ("huge-discarded-shift", (MIN_SUB, MIN_SUB, MIN_SUB)),
        ("maximum-significands", (MAX, MAX, MAX)),
        ("three-limb-sticky", (math.nextafter(1.0, math.inf),) * 3),
    ]
    for factor in [0.25, 0.5, 0.75, 1.5, 2.5, 3.5, 4.5]:
        hand.append(("subnormal-midpoint-neighbors", (MIN_SUB, factor, 1.0)))
    midpoint_factor = float((2**54 - 1) // 3)
    for neighbor in [-2, -1, 0, 1, 2]:
        hand.append(("overflow-midpoint-neighbors",
                     (3.0, midpoint_factor + neighbor, math.ldexp(1.0, 970))))
    for family, values in hand:
        for signed in signs(values):
            for permutation in itertools.permutations(signed):
                yield family, permutation

    # Every finite exponent field, several significand shapes, exact identity
    # scales, intermediate rescue in both orders and mantissa midpoint rounding.
    for exponent in range(2047):
        for fraction in [0, 1, 1 << 51, (1 << 52) - 1]:
            value = from_bits((exponent << 52) | fraction)
            for second, third in [(1.0, 1.0), (2.0, 0.5), (0.5, 2.0), (1.5, 1.0)]:
                for sign in [-1.0, 1.0]:
                    yield "all-exponent-fields", (sign * value, second, third)

    rng = random.Random(SEED)
    # Balance individually large/small scales near the FINAL boundaries. The
    # expected result still comes exclusively from the original exact inputs.
    for index in range(8000):
        target = [-1076, -1075, -1074, -1023, -1022, -1, 0, 1023, 1024][index % 9]
        first = rng.randrange(-1074, 1024)
        second = rng.randrange(-1074, 1024)
        third = target - first - second
        if not -1074 <= third <= 1023:
            continue
        values = tuple(math.ldexp(1.0, exponent)
                       for exponent in (first, second, third))
        yield "balanced-power-boundaries", values
        yield "balanced-power-boundaries", (-values[0], values[1], values[2])

    for _ in range(20000):
        values = []
        for _ in range(3):
            raw = rng.getrandbits(64)
            if (raw >> 52) & 0x7ff == 0x7ff:
                raw ^= 1 << 52
            values.append(from_bits(raw))
        yield "seeded-finite-bits", tuple(values)

    # Every subnormal magnitude near zero, the normal boundary and alternating
    # integer parity; all signs with nontrivial three-factor significands.
    for magnitude in list(range(1, 257)) + [(1 << 52) - offset for offset in range(1, 33)]:
        value = from_bits(magnitude)
        for scale in [0.5, 1.5, 2.5, math.nextafter(1.0, math.inf)]:
            for signed in signs((value, scale, math.nextafter(1.0, 0.0))):
                yield "subnormal-parity-sticky", signed


def render(generated_at):
    header = [
        "schema: 1",
        "fixture-set: CASHFLOW-PRODUCT-ORACLE",
        "domain: cashflows",
        "description: exact product of three supplied finite binary64 inputs, rounded once; arithmetic scope only",
        "reference: oracle:Python Fraction.from_float",
        "reference-version: Python " + platform.python_version(),
        "reference-source: https://docs.python.org/3.9/library/fractions.html; exact original input rationals with one nearest-ties-even float conversion",
        "tolerance: TOL-E0; every accepted result has exactly matching binary64 bits; exact zero canonical +0; nonzero zero/infinity rejected",
        "reproduction: generator-sha256=" + REVISION + "; python3 validation/generator/cashflow_product_oracles.py",
        "generated-at: " + generated_at,
        "platform: " + platform.system() + " " + platform.machine(),
        "random-seed: 0x5ab17008; Python random.Random; all 2047 finite exponent fields plus bounded seeded bit patterns",
    ]
    rows = []
    counts = Counter()
    for index, (family, values) in enumerate(triples()):
        assert all(math.isfinite(value) for value in values)
        exact = Fraction(1)
        for value in values:
            exact *= Fraction.from_float(value)
        result = rounded(exact)
        if exact == 0:
            behavior, kind, result = "zero", "", 0.0
        elif not math.isfinite(result):
            behavior, kind = "overflow", "value.result_not_finite"
        elif result == 0.0:
            behavior, kind = "underflow", "value.result_out_of_domain"
        else:
            behavior, kind = "value", ""
        sign = "negative" if exact < 0 else "positive"
        counts[family] += 1
        counts[behavior] += 1
        rows.append("\t".join([
            f"PRODUCT-{index:06d}", family,
            *(f"0x{bits(value):016x}" for value in values),
            behavior, kind, f"0x{bits(result):016x}", sign,
        ]))
    text = "\n".join(header) + "\n\nid\tfamily\ta\tb\tc\tbehavior\terror_kind\texpected\texact_sign\n"
    return text + "\n".join(rows) + "\n", counts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        generated_at = next(line.split(": ", 1)[1] for line in previous.splitlines()
                            if line.startswith("generated-at: "))
        text, counts = render(generated_at)
        if text != previous:
            raise SystemExit("cashflow product oracle is stale; regenerate and review")
        label = "reproducible"
    else:
        text, counts = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_text(text)
        label = "written"
    print("cashflow product oracle", label + ":", sum(counts[key] for key in ["value", "zero", "underflow", "overflow"]), "rows")
    print("families/status:", dict(sorted(counts.items())))
    print("generator sha256:", REVISION)
    print("fixture sha256:", hashlib.sha256(text.encode()).hexdigest())


if __name__ == "__main__":
    main()
