#!/usr/bin/env python3
"""Explicit-period ICMA exact fractions and public QuantLib 1.43 components."""

import argparse
import calendar
import datetime as dt
import fractions
import hashlib
import math
import pathlib
import platform
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/daycount"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
FREQUENCIES = {
    "Annual": 1, "Semiannual": 2, "EveryFourthMonth": 3, "Quarterly": 4,
    "Bimonthly": 6, "Monthly": 12, "EveryFourthWeek": 13, "Biweekly": 26,
    "Weekly": 52, "Daily": 365,
}
SOURCE = "https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf"


def d(text):
    return dt.date.fromisoformat(text)


def qdate(ql, date):
    return ql.Date(date.day, date.month, date.year)


def exact_fraction(start, end, boundaries, frequency):
    if start > end:
        return -exact_fraction(end, start, boundaries, frequency)
    return sum((fractions.Fraction((min(end, last) - max(start, first)).days,
                                   FREQUENCIES[frequency] * (last - first).days)
                for first, last in zip(boundaries, boundaries[1:]) if max(start, first) < min(end, last)), fractions.Fraction())


def scenarios():
    shapes = (
        ("REGULAR", "Semiannual", ("2003-11-01", "2004-05-01"), "2003-11-01", "2004-05-01"),
        ("SHORT-FIRST", "Annual", ("1998-07-01", "1999-07-01"), "1999-02-01", "1999-07-01"),
        ("LONG-FIRST", "Semiannual", ("2002-07-15", "2003-01-15", "2003-07-15"), "2002-08-15", "2003-07-15"),
        ("SHORT-FINAL", "Semiannual", ("1999-12-31", "2000-06-30"), "1999-12-31", "2000-02-29"),
        ("LONG-FINAL", "Semiannual", ("2000-01-01", "2000-07-01", "2001-01-01"), "2000-01-01", "2000-08-15"),
        ("MONTH-EOM", "Monthly", ("2024-01-31", "2024-02-29", "2024-03-31", "2024-04-30"), "2024-02-01", "2024-04-29"),
    )
    for name, frequency, bounds, start, end in shapes:
        for reversed_dates in (False, True):
            yield name + ("-REVERSED" if reversed_dates else ""), frequency, tuple(map(d, bounds)), d(end if reversed_dates else start), d(start if reversed_dates else end), True
    # All frequency values are tested against the exact declared formula.
    # The reference API cannot express a separate weekly/daily frequency:
    # its explicit-date overload infers frequency from period length. Those
    # rows are therefore oracle-only, not relabeled as compatible.
    for frequency in FREQUENCIES:
        yield "FREQUENCY-" + frequency, frequency, (d("2024-01-01"), d("2025-01-01")), d("2024-02-29"), d("2024-09-17"), frequency == "Annual"
    for frequency in ("Annual", "Daily"):
        yield "FULL-RANGE-" + frequency, frequency, (dt.date.min, dt.date.max), dt.date.min, dt.date.max, False
        yield "FULL-RANGE-SINGLE-DAY-" + frequency, frequency, (dt.date.min, dt.date.max), dt.date.min, dt.date.min + dt.timedelta(days=1), False
    # Arbitrary explicit positive segments stress summation, not market
    # schedule semantics. Boundaries are serialized as inputs, never inferred
    # by SabiQuant or a QuantLib schedule constructor.
    boundaries, state = [d("1950-01-01")], 0x49434D41
    for _ in range(1024):
        state = (1664525 * state + 1013904223) & 0xFFFFFFFF
        boundaries.append(boundaries[-1] + dt.timedelta(days=1 + state % 31))
    for frequency in ("Annual", "Monthly", "Daily"):
        yield "IRREGULAR-1024-" + frequency, frequency, tuple(boundaries), boundaries[0] + dt.timedelta(days=1), boundaries[-1] - dt.timedelta(days=1), False
    # Calendar-month/year examples for every reference-supported frequency.
    for frequency, months in (("Annual", 12), ("Semiannual", 6), ("EveryFourthMonth", 4),
                              ("Quarterly", 3), ("Bimonthly", 2), ("Monthly", 1)):
        for year in (1999, 2000, 2099, 2100):
            boundaries = []
            for index in range(13):
                year_index, month_index = divmod(year * 12 + index * months, 12)
                boundaries.append(dt.date(year_index, month_index + 1, 1))
            for first, last in ((0, 1), (0, 12), (3, 11)):
                start, end = boundaries[first] + dt.timedelta(days=1), boundaries[last] - dt.timedelta(days=1)
                yield "GRID-" + frequency + "-" + str(year) + "-" + str(first) + "-" + str(last), frequency, tuple(boundaries), start, end, True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=pathlib.Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    import QuantLib as ql
    if ql.__version__ != "1.43":
        parser.error("installed QuantLib must be 1.43")
    if not args.quantlib_wheel.name.lower().startswith("quantlib-1.43-"):
        parser.error("provide the exact QuantLib 1.43 wheel")
    wheel_bytes = args.quantlib_wheel.read_bytes()
    installed = pathlib.Path(ql.__file__).resolve().parent.parent
    with zipfile.ZipFile(args.quantlib_wheel) as wheel:
        members = [name for name in wheel.namelist() if name.startswith("QuantLib/") and not name.endswith("/")]
        if not members:
            parser.error("wheel has no QuantLib module")
        for name in members:
            if hashlib.sha256((installed / name).read_bytes()).digest() != hashlib.sha256(wheel.read(name)).digest():
                parser.error("installed module differs from wheel: " + name)
    counter = ql.ActualActual(ql.ActualActual.ISMA)
    oracle, reference = [], []
    for name, frequency, boundaries, start, end, comparable in scenarios():
        exact = exact_fraction(start, end, boundaries, frequency)
        base = ("ICMA-" + name, start, end, frequency, ",".join(map(str, boundaries)), repr(float(exact)), exact.numerator, exact.denominator)
        oracle.append(base)
        if comparable:
            lower, upper, components = min(start, end), max(start, end), []
            for first, last in zip(boundaries, boundaries[1:]):
                # This guard verifies the reference's interpreted frequency,
                # rather than silently treating inferred input as equivalent.
                full = counter.yearFraction(qdate(ql, first), qdate(ql, last), qdate(ql, first), qdate(ql, last))
                if full != 1 / FREQUENCIES[frequency]:
                    raise ValueError("reference does not interpret the explicit period with the declared frequency")
                overlap_start, overlap_end = max(lower, first), min(upper, last)
                if overlap_start < overlap_end:
                    value = counter.yearFraction(qdate(ql, overlap_start), qdate(ql, overlap_end), qdate(ql, first), qdate(ql, last))
                    if not math.isfinite(value):
                        raise ValueError("nonfinite reference component")
                    components.append(value)
            result = math.fsum(components) * (-1 if start > end else 1)
            reference.append(base + (repr(result), ",".join(map(repr, components)), len(components)))
    coverage = []
    for name, zero in (("BEFORE", d("2023-12-31")), ("AFTER", d("2024-07-02"))):
        start, end = d("2024-01-01"), d("2024-07-01")
        value = counter.yearFraction(qdate(ql, zero), qdate(ql, zero), qdate(ql, start), qdate(ql, end))
        if value != 0.0:
            raise ValueError("pinned zero-outside-coverage reference behavior changed")
        coverage.append(("ICMA-ZERO-" + name, zero, zero, "Semiannual", str(start) + "," + str(end), repr(value), "icma.out_of_coverage", "D9-ICMA-ZERO-COVERAGE"))

    def write(name, columns, records, quantlib):
        metadata = {
            "schema": "1", "fixture-set": name.upper(), "domain": "day-count",
            "description": "Explicit ICMA determination boundaries; exact fractions and documented public reference components",
            "reference": "QuantLib" if quantlib else "oracle:Python fractions",
            "reference-version": "1.43 (reported by attested Python wheel)" if quantlib else "Python " + platform.python_version() + " exact Fraction",
            "reference-source": "QuantLib public ActualActual(ISMA).yearFraction explicit reference dates" if quantlib else "Python datetime/Fraction; primary ICMA Appendix A5 March 2022 Version 2",
            "tolerance": "TOL-E2", "reproduction": "generator-sha256=" + REVISION + "; python3 validation/generator/icma_fixtures.py --quantlib-wheel " + args.quantlib_wheel.name,
            "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
            "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
            "primary-source": SOURCE,
        }
        if quantlib:
            metadata.update({"quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": hashlib.sha256(wheel_bytes).hexdigest(),
                             "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995 from docs/QUANTLIB_REFERENCE.md; wheel commit not independently attested"})
        lines = ["\t".join(columns)] + ["\t".join(map(str, record)) for record in records]
        body = "\n".join(lines) + "\n"
        path = DEST / (name + ".fixture")
        if args.check:
            old_header, old_body = path.read_text().split("\n\n", 1)
            old = dict(line.split(": ", 1) for line in old_header.splitlines())
            stable = metadata.keys() - {"generated-at", "platform", "python-version"}
            if body != old_body or any(old.get(key) != metadata[key] for key in stable):
                raise SystemExit("fixture differs: " + str(path))
            print("verified", path.relative_to(ROOT), "records=" + str(len(records)))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n" + body).encode())
            print("generated", path.relative_to(ROOT), "records=" + str(len(records)))
    columns = ("id", "start", "end", "frequency", "boundaries", "expected", "exact_numerator", "exact_denominator")
    write("icma-oracle", columns, oracle, False)
    write("icma-quantlib", columns + ("reference_result", "reference_components", "component_count"), reference, True)
    write("icma-strict-coverage", ("id", "start", "end", "frequency", "boundaries", "reference_result", "expected_error", "discrepancy"), coverage, True)


if __name__ == "__main__":
    main()
