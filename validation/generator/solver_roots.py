#!/usr/bin/env python3
"""Independent 120/200-digit polynomial roots across binary64 coordinate scales."""

import argparse
import datetime as dt
from decimal import Decimal as D, localcontext
import hashlib
import math
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/math/solver-roots-oracle.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()


def encoded(value):
    return f"0x{struct.unpack('>Q', struct.pack('>d', value))[0]:016x}"


def cases():
    roots = {}
    for degree, coefficients in [(2, [1.01, 1.25, 2.0, 2.5, 3.0, 3.9]),
                                  (3, [1.01, 1.25, 2.0, 2.5, 3.0, 4.0, 7.0, 7.9])]:
        for coefficient in coefficients:
            for precision in [120, 200]:
                with localcontext() as context:
                    context.prec = precision
                    value = D.from_float(coefficient)
                    roots[degree, coefficient, precision] = value.sqrt() if degree == 2 else (value.ln()/D(3)).exp()
            for exponent in range(-1000, 1001, 8):
                scale = math.ldexp(1.0, exponent)
                absolute = scale*1e-13
                for translation in [-4.0, 0.0, 4.0]:
                    shift = translation*scale
                    for direction in [-1, 1]:
                        expected = []
                        for precision in [120, 200]:
                            with localcontext() as context:
                                context.prec = precision
                                truth = D.from_float(shift)+D(direction)*D.from_float(scale)*roots[degree, coefficient, precision]
                                expected.append(float(truth))
                        if expected[0] != expected[1]:
                            raise ArithmeticError("unresolved precision agreement for polynomial root")
                        lower = shift+(scale if direction == 1 else -2*scale)
                        upper = shift+(2*scale if direction == 1 else -scale)
                        initial = shift+direction*1.5*scale
                        yield degree, coefficient, scale, shift, direction, lower, upper, initial, absolute, expected[1]


def render(generated_at):
    header = ["schema: 1", "fixture-set: MATH-SOLVER-ROOTS", "domain: math",
        "description: square/cube polynomial roots; 251 binary scales, translated and negative intervals; stable rounding at two precisions",
        "reference: oracle:Python Decimal", "reference-version: Python "+platform.python_version()+"; Decimal 120/200 digits",
        "reference-source: independently solved z^degree=coefficient; Decimal sqrt or exp(ln(a)/3); affine transform of exact supplied binary64 inputs; no Rust/QuantLib outputs",
        "tolerance: TOL-E5-SOLVER-ROOT explicit positional tolerance plus callback/rounded-reference conditioning; no residual-only acceptance",
        "reproduction: generator-sha256="+REVISION+"; python3 validation/generator/solver_roots.py",
        "generated-at: "+generated_at, "platform: "+platform.system()+" "+platform.machine(),
        "callback: z=direction*(x-shift)/scale; repeated basic products; finite analytic derivative; callback cap300",
    ]
    rows = []
    for index, values in enumerate(cases()):
        degree, coefficient, scale, shift, direction, lower, upper, initial, absolute, expected = values
        rows.append("\t".join([f"SOLVER-ROOT-{index:06d}", str(degree), repr(coefficient), repr(scale), repr(shift), str(direction),
            repr(lower), repr(upper), repr(initial), repr(absolute), encoded(expected)]))
    return "\n".join(header)+"\n\nid\tdegree\tcoefficient\tscale\tshift\tdirection\tlower\tupper\tinitial\tabsolute\texpected\n"+"\n".join(rows)+"\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        date = next(line.split(": ", 1)[1] for line in previous.splitlines() if line.startswith("generated-at: "))
        if render(date) != previous:
            raise SystemExit("solver roots oracle stale; regenerate and review")
        print("solver roots oracle reproducible")
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.write_text(data)
        print("solver roots oracle written:", len(data.splitlines())-14, "records")


if __name__ == "__main__":
    main()
