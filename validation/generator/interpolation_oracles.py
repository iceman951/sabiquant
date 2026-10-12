#!/usr/bin/env python3
"""Independent rational spline constraints and two-precision log interpolation.

Natural splines are solved as a dense system of polynomial constraints, not
as the implementation's tridiagonal second-derivative system. Hermite pieces
are solved from endpoint values/derivatives as independent exact rationals.
"""

import argparse
import datetime as dt
from decimal import Decimal, localcontext
from fractions import Fraction as F
import hashlib
import itertools
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/math/interpolation-oracle.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
UNIT = F(1, 2**53)
MIN_SUB = float.fromhex("0x0.0000000000001p-1022")


def solve(matrix, right):
    rows = [list(map(F, row)) + [F(value)] for row, value in zip(matrix, right)]
    size = len(rows)
    for col in range(size):
        pivot = next(i for i in range(col, size) if rows[i][col])
        rows[col], rows[pivot] = rows[pivot], rows[col]
        divisor = rows[col][col]
        rows[col] = [x/divisor for x in rows[col]]
        for i in range(size):
            if i != col and rows[i][col]:
                factor = rows[i][col]
                rows[i] = [x-factor*y for x, y in zip(rows[i], rows[col])]
    return [row[-1] for row in rows]


def natural_pieces(xs, ys):
    intervals = len(xs)-1
    widths = [b-a for a, b in zip(xs, xs[1:])]
    matrix, right = [], []

    def constraint(terms, value):
        row = [F(0)]*(4*intervals)
        for index, coefficient in terms:
            row[index] = coefficient
        matrix.append(row)
        right.append(value)

    for i in range(intervals):
        constraint([(4*i, 1)], ys[i])
        constraint([(4*i+k, 1) for k in range(4)], ys[i+1])
    for i in range(intervals-1):
        constraint([(4*i+k, F(k)/widths[i]) for k in range(1, 4)]
                   + [(4*(i+1)+1, -1/widths[i+1])], 0)
        constraint([(4*i+2, 2/widths[i]**2), (4*i+3, 6/widths[i]**2),
                    (4*(i+1)+2, -2/widths[i+1]**2)], 0)
    constraint([(2, 1)], 0)
    constraint([(4*(intervals-1)+2, 2), (4*(intervals-1)+3, 6)], 0)
    coefficients = solve(matrix, right)
    return [coefficients[4*i:4*i+4] for i in range(intervals)]


def same_sign(a, b):
    return a > 0 and b > 0 or a < 0 and b < 0


def endpoint(h0, h1, d0, d1):
    result = ((2*h0+h1)*d0-h0*d1)/(h0+h1)
    if not same_sign(result, d0):
        return F(0)
    if not same_sign(d0, d1) and abs(result) > 3*abs(d0):
        return 3*d0
    return result


def monotone_pieces(xs, ys):
    widths = [b-a for a, b in zip(xs, xs[1:])]
    secants = [(b-a)/h for a, b, h in zip(ys, ys[1:], widths)]
    if len(xs) == 2:
        slopes = secants*2
    else:
        slopes = [endpoint(widths[0], widths[1], secants[0], secants[1])]
        for i in range(1, len(xs)-1):
            left, right = secants[i-1:i+1]
            if not same_sign(left, right):
                slopes.append(F(0))
            else:
                w1, w2 = 2*widths[i]+widths[i-1], widths[i]+2*widths[i-1]
                slopes.append((w1+w2)/(w1/left+w2/right))
        slopes.append(endpoint(widths[-1], widths[-2], secants[-1], secants[-2]))
    return [solve([[1, 0, 0, 0], [1, 1, 1, 1], [0, 1, 0, 0], [0, 1, 2, 3]],
                  [ys[i], ys[i+1], widths[i]*slopes[i], widths[i]*slopes[i+1]])
            for i in range(len(xs)-1)]


def log_value(y0, y1, t, precision):
    with localcontext() as context:
        context.prec = precision
        left = Decimal(y0.numerator)/Decimal(y0.denominator)
        right = Decimal(y1.numerator)/Decimal(y1.denominator)
        weight = Decimal(t.numerator)/Decimal(t.denominator)
        logarithm = (1-weight)*left.ln()+weight*right.ln()
        expected = logarithm.exp()
        scale = 8*(Decimal(2)**-53)*expected*(1+abs(logarithm)
                    + abs(1-weight)*abs(left.ln())+abs(weight)*abs(right.ln()))
        return float(expected), float(scale)


def cases():
    for size in range(2, 6):
        grids = [list(range(size)), [i*i for i in range(size)],
                 [i*(i+1)//2 for i in range(size)],
                 [i*2+(i % 2) for i in range(size)]]
        for grid in grids:
            xs = list(map(F, grid))
            queries = xs[:]
            for a, b in zip(xs, xs[1:]):
                queries.extend(a+(b-a)*F(k, 4) for k in range(1, 4))
            queries += [xs[0]-(xs[1]-xs[0])/2, xs[-1]+(xs[-1]-xs[-2])/2]
            for raw_ys in itertools.product([-2, 0, 3], repeat=size):
                ys = list(map(F, raw_ys))
                pieces = {
                    "Linear": [[ys[i], ys[i+1]-ys[i], F(0), F(0)] for i in range(size-1)],
                    "NaturalCubic": natural_pieces(xs, ys),
                    "MonotoneCubicHermite": monotone_pieces(xs, ys),
                }
                for method in ["Linear", "LogLinear", "NaturalCubic", "MonotoneCubicHermite"]:
                    values = [y+4 for y in ys] if method == "LogLinear" else ys
                    for query in queries:
                        interval = min(size-2, max(0, sum(x <= query for x in xs)-1))
                        t = (query-xs[interval])/(xs[interval+1]-xs[interval])
                        exact_node = query in xs
                        if exact_node:
                            expected = float(values[xs.index(query)])
                            budget = 0.0
                        elif method == "LogLinear":
                            low, budget = log_value(values[interval], values[interval+1], t, 120)
                            expected, _ = log_value(values[interval], values[interval+1], t, 200)
                            if low != expected:
                                raise ArithmeticError("unresolved two-precision log interpolation rounding")
                        else:
                            terms = [coefficient*t**k for k, coefficient in enumerate(pieces[method][interval])]
                            truth = sum(terms, F(0))
                            expected = float(truth)
                            # Original calibration allowance for normalized
                            # polynomial evaluation and coefficient preparation.
                            # It is not a certificate for all grids/conditioning.
                            scale = sum(map(abs, terms), F(0))+max(map(abs, values))
                            budget = max(MIN_SUB, float(32*UNIT*scale))
                        yield method, xs, values, query, expected, budget, exact_node


def encoded(value):
    return f"0x{struct.unpack('>Q', struct.pack('>d', value))[0]:016x}"


def render(generated_at):
    header = [
        "schema: 1", "fixture-set: MATH-INTERPOLATION-ORACLE", "domain: math",
        "description: exhaustive small node ordinates on four ordered grids; knots, interior and extrapolation; exact rational spline constraints",
        "reference: oracle:Python Fraction and Decimal",
        "reference-version: Python " + platform.python_version() + "; Decimal 120/200 digits",
        "reference-source: project M6 interpolation equations; dense polynomial constraints independent of Rust's tridiagonal construction; weighted harmonic Hermite; no Rust or QuantLib outputs",
        "tolerance: TOL-E0 exact knots; TOL-E5-INTERPOLATION rational polynomial absolute conditioned scale 32u; TOL-E3 log interpolation k=8; empirically calibrated, not universal certificates",
        "reproduction: generator-sha256=" + REVISION + "; python3 validation/generator/interpolation_oracles.py",
        "generated-at: " + generated_at,
        "platform: " + platform.system() + " " + platform.machine(),
        "extrapolation: ExtendBoundarySegment; nodes length2..5; ordinate Cartesian product[-2,0,3], LogLinear shifted+4",
    ]
    rows = []
    for index, (method, xs, ys, query, expected, budget, knot) in enumerate(cases()):
        rows.append("\t".join([f"INTERPOLATION-{index:06d}", method,
            ";".join(repr(float(x))+","+repr(float(y)) for x, y in zip(xs, ys)),
            repr(float(query)), encoded(expected), repr(budget), "yes" if knot else "no"]))
    return "\n".join(header) + "\n\nid\tmethod\tnodes\tquery\texpected\tabsolute_budget\texact_knot\n" + "\n".join(rows) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        date = next(line.split(": ", 1)[1] for line in previous.splitlines() if line.startswith("generated-at: "))
        if render(date) != previous:
            raise SystemExit("interpolation oracle stale; regenerate and review")
        print("interpolation oracle reproducible")
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.write_text(data)
        print("interpolation oracle written:", len(data.splitlines())-14, "records")


if __name__ == "__main__":
    main()
