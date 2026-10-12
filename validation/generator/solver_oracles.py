#!/usr/bin/env python3
"""Exact Fraction validation of the pinned binary64 solver tolerance test.

Every arithmetic operation is independently rounded from an exact rational;
no Rust or QuantLib results are read. The real configured allowance is also
retained exactly, so operational acceptance and rounding bounds are distinct.
This validates the stopping contract, not mathematical callback accuracy.
"""

import argparse
import datetime as dt
from fractions import Fraction as F
import hashlib
import itertools
import math
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/math/solver-tolerance-oracle.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
ETA = float.fromhex("0x0.0000000000001p-1022")
MAXIMUM = float.fromhex("0x1.fffffffffffffp+1023")
UNIT = F(1, 2**53)


def rounded(value):
    """Python Fraction's correctly rounded binary64 conversion, including range."""
    try:
        return float(value)
    except OverflowError:
        return math.inf if value > 0 else -math.inf


def encoded(value):
    return "0x%016x" % struct.unpack(">Q", struct.pack(">d", value))[0]


def operational(lower, upper, absolute, relative):
    a, b, absolute, relative = map(F, (lower, upper, absolute, relative))
    magnitude = max(abs(a), abs(b))
    true_width = b-a
    true_allowance = absolute+relative*magnitude
    width = rounded(true_width)
    if math.isfinite(width):
        product = rounded(relative*magnitude)
        tolerance = rounded(absolute+F(product))
        branch, multiplier, additive = "full", 4, 2
    else:
        width = rounded(F(rounded(b/2))-F(rounded(a/2)))
        product = rounded(relative*F(rounded(magnitude/2)))
        tolerance = rounded(F(rounded(absolute/2))+F(product))
        branch, multiplier, additive = "overflow-half", 6, 4
    accepted = width <= tolerance
    # This rational check never rounds the mathematical width or allowance.
    bound = (1+multiplier*UNIT)*true_allowance+additive*F(ETA)
    if accepted and true_width > bound:
        raise ArithmeticError("claimed conservative rounding allowance violated")
    return branch, accepted, true_width <= true_allowance, width, tolerance


def cases():
    positive = [ETA, 2*ETA, 3*ETA, 4*ETA, 5*ETA, 8*ETA,
                float.fromhex("0x0.fffffffffffffp-1022"),
                float.fromhex("0x1p-1022"),
                math.nextafter(float.fromhex("0x1p-1022"), math.inf),
                2.0**-52, 1.0, math.nextafter(1.0, math.inf), 2.0,
                2.0**53, 2.0**500, 2.0**1023,
                math.nextafter(MAXIMUM, -math.inf), MAXIMUM]
    endpoints = sorted([-x for x in positive]+[0.0]+positive)
    absolutes = [ETA, 4*ETA, 2.0**-1022, 2.0**-52, 1.0, 2.0**500, MAXIMUM/2, MAXIMUM]
    relatives = [0.0, ETA, 2.0**-53, 0.125, 0.3, 0.5, 0.6, 0.75, math.nextafter(1.0, -math.inf)]
    seen = set()

    def emit(a, b, absolute, relative):
        key = tuple(encoded(x) for x in (a, b, absolute, relative))
        if key in seen:
            return None
        seen.add(key)
        return a, b, absolute, relative

    pairs = list(itertools.combinations(endpoints, 2))
    # Cartesian endpoints include opposite extremes, all signed subnormal
    # widths, the subnormal/normal transition and equal-magnitude signs.
    for a, b in pairs:
        for absolute, relative in itertools.product(absolutes, relatives):
            row = emit(a, b, absolute, relative)
            if row is not None:
                yield row
    # Target the exact acceptance surface, then perturb absolute tolerance
    # by adjacent representable values in both directions.
    for a, b in pairs:
        for relative in relatives:
            needed = F(b)-F(a)-F(relative)*max(abs(F(a)), abs(F(b)))
            if needed <= 0 or needed > F(MAXIMUM):
                continue
            pivot = rounded(needed)
            candidates = [pivot]
            down, up = pivot, pivot
            for _ in range(2):
                down = math.nextafter(down, -math.inf)
                up = math.nextafter(up, math.inf)
                candidates.extend((down, up))
            for absolute in candidates:
                if absolute > 0 and math.isfinite(absolute):
                    row = emit(a, b, absolute, relative)
                    if row is not None:
                        yield row
    # Arbitrary exponent patterns, with adjacent coordinate brackets rather
    # than only the fixed grid above. Seed and order are stable.
    state = 0x534F4C5645520001
    for _ in range(2048):
        state = (state*6364136223846793005+1442695040888963407) & ((1 << 64)-1)
        bits = state & 0xFFEFFFFFFFFFFFFF
        a = struct.unpack(">d", struct.pack(">Q", bits))[0]
        b = math.nextafter(a, math.inf)
        if not math.isfinite(b):
            continue
        for relative in [0.0, 2.0**-53, math.nextafter(1.0, -math.inf)]:
            for absolute in [ETA, abs(b-a), 2*abs(b-a)]:
                if absolute > 0 and math.isfinite(absolute):
                    row = emit(a, b, absolute, relative)
                    if row is not None:
                        yield row
    # Arbitrary non-dyadic-scale significands and relative coefficients near
    # the real acceptance surface exercise independently rounded products.
    for _ in range(2048):
        values = []
        for _ in range(3):
            state = (state*6364136223846793005+1442695040888963407) & ((1 << 64)-1)
            values.append(state)
        a = struct.unpack(">d", struct.pack(">Q", values[0] & 0xFFEFFFFFFFFFFFFF))[0]
        b = struct.unpack(">d", struct.pack(">Q", values[1] & 0xFFEFFFFFFFFFFFFF))[0]
        a, b = sorted((a, b))
        if a == b:
            continue
        relative = (values[2] >> 11)/2.0**53
        needed = F(b)-F(a)-F(relative)*max(abs(F(a)), abs(F(b)))
        if needed <= 0 or needed > F(MAXIMUM):
            continue
        pivot = rounded(needed)
        for absolute in [math.nextafter(pivot, -math.inf), pivot, math.nextafter(pivot, math.inf)]:
            if absolute > 0 and math.isfinite(absolute):
                row = emit(a, b, absolute, relative)
                if row is not None:
                    yield row


def render(generated_at):
    rows = []
    counts = {"full": 0, "overflow-half": 0, "accepted": 0, "rejected": 0,
              "operational-only-accept": 0, "operational-only-reject": 0}
    for index, (a, b, absolute, relative) in enumerate(cases()):
        branch, accepted, exact_accept, width, tolerance = operational(a, b, absolute, relative)
        counts[branch] += 1
        counts["accepted" if accepted else "rejected"] += 1
        if accepted != exact_accept:
            counts["operational-only-accept" if accepted else "operational-only-reject"] += 1
        adjacent = math.nextafter(a, math.inf) == b
        status = "accepted" if accepted else ("precision" if adjacent else "budget")
        rows.append("\t".join(["SOLVER-TOL-%06d" % index, *(encoded(x) for x in (a, b, absolute, relative)),
            branch, status, "yes" if exact_accept else "no", encoded(width), encoded(tolerance)]))
    header = [
        "schema: 1", "fixture-set: MATH-SOLVER-TOLERANCE-ORACLE", "domain: math",
        "description: exact rational operational tolerance, real configured allowance and conservative rounding bounds; fixed Cartesian and boundary-adjacent grids",
        "reference: oracle:Python Fraction",
        "reference-version: Python " + platform.python_version() + "; exact rational arithmetic with correctly rounded binary64 conversion",
        "reference-source: pinned full-width and overflow-half solver contract; independently rounded exact operations; no Rust or QuantLib outputs",
        "tolerance: TOL-E0 status and rounded intermediates; TOL-E5-SOLVER-BRACKET accepted real width <= (1+4u)T+2MIN_SUB or (1+6u)T+4MIN_SUB; no changed solver tolerance",
        "reproduction: generator-sha256="+REVISION+"; python3 validation/generator/solver_oracles.py",
        "generated-at: "+generated_at,
        "platform: "+platform.system()+" "+platform.machine(),
        "counts: "+"; ".join(key+"="+str(value) for key, value in sorted(counts.items())),
        "callback: -1 at lower, +1 at upper; two shared calls only; adjacent rejection is precision exhaustion, other rejection is budget exhaustion",
    ]
    return "\n".join(header)+"\n\nid\tlower\tupper\tabsolute\trelative\tbranch\tstatus\texact_accept\toperational_width\toperational_tolerance\n"+"\n".join(rows)+"\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        generated_at = next(line.split(": ", 1)[1] for line in previous.splitlines() if line.startswith("generated-at: "))
        if render(generated_at) != previous:
            raise SystemExit("solver tolerance oracle stale; regenerate and review")
        print("solver tolerance oracle reproducible")
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.write_text(data)
        print("solver tolerance oracle written:", len(data.splitlines())-15, "records")


if __name__ == "__main__":
    main()
