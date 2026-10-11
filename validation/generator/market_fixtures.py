#!/usr/bin/env python3
"""Attested public QuantLib market/Easter and contextual day-count fixtures."""

import argparse
import calendar
import datetime as dt
from fractions import Fraction
import hashlib
import pathlib
import platform
import zipfile

from daycount_fixtures import samples

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/market"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
VALID_FROM = dt.date(1999, 1, 4)
MAX = dt.date(2199, 12, 31)


def qdate(ql, date):
    return ql.Date(date.day, date.month, date.year)


def gauss(year):
    # Independent Gauss computus, not production's Oudin calculation.
    a, b, c = year % 19, year % 4, year % 7
    century = year // 100
    p, q = (13 + 8 * century) // 25, century // 4
    m, n = (15 - p + century - q) % 30, (4 + century - q) % 7
    d = (19 * a + m) % 30
    e = (2 * b + 4 * c + 6 * d + n) % 7
    result = dt.date(year, 3, 22) + dt.timedelta(days=d + e)
    if d == 29 and e == 6 or d == 28 and e == 6 and a > 10:
        result -= dt.timedelta(days=7)
    return result


def target_business(date):
    if date.weekday() >= 5:
        return False
    fixed = {(1, 1), (12, 25)}
    if date.year >= 2000:
        fixed |= {(5, 1), (12, 26)}
        easter = gauss(date.year)
        if date in (easter - dt.timedelta(days=2), easter + dt.timedelta(days=1)):
            return False
    if date.year in (1999, 2001):
        fixed.add((12, 31))
    return (date.month, date.day) not in fixed


def coverage(date):
    return "Verified" if date.year <= 2002 or 2026 <= date.year <= 2028 else "Projected"


def adjusted_days(start, end, first, last):
    return 360 * (end.year - start.year) + 30 * (end.month - start.month) + last - first


def bond_direct(start, end):
    first = min(start.day, 30)
    last = 30 if end.day == 31 and first == 30 else end.day
    return adjusted_days(start, end, first, last)


def isda_direct(start, end, termination):
    def february_end(date):
        return date.month == 2 and date.day == calendar.monthrange(date.year, 2)[1]
    first = 30 if start.day == 31 or february_end(start) else start.day
    last = 30 if end.day == 31 or february_end(end) and end != termination else end.day
    return adjusted_days(start, end, first, last)


def signed_forward(function, start, end):
    if start == end:
        return 0
    return -function(end, start) if start > end else function(start, end)


def write_fixture(name, columns, rows, ql, args, description):
    metadata = {
        "schema": "1", "fixture-set": name.upper(), "domain": "market",
        "description": description, "reference": "QuantLib", "reference-version": ql.__version__,
        "reference-source": "Attested wheel public API; independent datetime/Gauss/Fraction oracle; ECB source facts attributed in M3_TARGET_CONTRACT.md",
        "tolerance": "Exact dates/counts; E1 correctly rounded Fraction quotient (0 ULP)",
        "input-samples-generator-sha256": hashlib.sha256((ROOT / "validation/generator/daycount_fixtures.py").read_bytes()).hexdigest(),
        "reproduction": "generator-sha256=" + REVISION + "; python3 validation/generator/market_fixtures.py --quantlib-wheel " + args.quantlib_wheel.name,
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
        "quantlib-wheel": args.quantlib_wheel.name,
        "quantlib-wheel-sha256": hashlib.sha256(args.quantlib_wheel.read_bytes()).hexdigest(),
        "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995; wheel source commit not independently attested",
    }
    body = "\t".join(columns) + "\n" + "".join("\t".join(map(str, row)) + "\n" for row in rows)
    path = DEST / (name + ".fixture")
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
    print(action, path.relative_to(ROOT), "records=" + str(len(rows)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=pathlib.Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    import QuantLib as ql
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

    uk = ql.UnitedKingdom(ql.UnitedKingdom.Settlement)
    easter_rows = []
    for year in range(1901, 2200):
        fridays = [dt.date(year, 3, 20) + dt.timedelta(days=n) for n in range(35)]
        closed = [day for day in fridays if day.weekday() == 4 and uk.isHoliday(qdate(ql, day))]
        if len(closed) != 1:
            raise ValueError("Good Friday oracle is not unique: " + str((year, closed)))
        reference = closed[0] + dt.timedelta(days=2)
        expected = gauss(year)
        if reference != expected:
            raise ValueError("Easter discrepancy: " + str((year, reference, expected)))
        easter_rows.append(("EASTER-%04d" % year, year, reference.isoformat(), expected.isoformat()))
    write_fixture("easter", ("id", "year", "reference_date", "oracle_date"), easter_rows, ql, args,
                  "299 Easter Sundays recovered from unique UK Settlement Good Friday closures; independently checked with Gauss computus")

    target = ql.TARGET()
    target_rows, prefixes = [], {"WeekendsOnly": [0], "TARGET": [0]}
    for ordinal in range(VALID_FROM.toordinal(), MAX.toordinal() + 1):
        date = dt.date.fromordinal(ordinal)
        expected = target_business(date)
        actual = target.isBusinessDay(qdate(ql, date))
        if actual != expected:
            raise ValueError("TARGET discrepancy: " + date.isoformat())
        target_rows.append(("TARGET-" + date.isoformat(), date.isoformat(), int(actual), int(expected), coverage(date)))
        prefixes["WeekendsOnly"].append(prefixes["WeekendsOnly"][-1] + int(date.weekday() < 5))
        prefixes["TARGET"].append(prefixes["TARGET"][-1] + int(expected))
    write_fixture("target-membership", ("id", "date", "reference_business", "oracle_business", "coverage"), target_rows, ql, args,
                  "All TARGET modeled dates 1999-01-04 through 2199-12-31; verified/projection coverage is independent of differential agreement")

    pairs = set(samples())
    pairs.update((a, b) for a, b in [
        (dt.date(2023, 1, 15), dt.date(2023, 3, 31)),
        (dt.date(2023, 3, 31), dt.date(2023, 1, 15)),
        (dt.date(2024, 1, 6), dt.date(2024, 1, 8)),
        (dt.date(2024, 1, 8), dt.date(2024, 1, 6)),
    ])
    rows, differences = [], {}
    for index, (start, end) in enumerate(sorted(pairs)):
        qs, qe = qdate(ql, start), qdate(ql, end)
        termination = end if end.month == 2 else dt.date(2199, 12, 31)
        counters = [
            ("30/360 (Bond Basis)", "", "", ql.Thirty360(ql.Thirty360.BondBasis),
             lambda a, b: bond_direct(a, b), "D9-BOND-SIGNED"),
            ("30E/360 (ISDA)", "", termination.isoformat(), ql.Thirty360(ql.Thirty360.ISDA, qdate(ql, termination)),
             lambda a, b: isda_direct(a, b, termination), "D9-30E-ISDA-SIGNED"),
        ]
        for name, market, term, counter, direct, label in counters:
            expected = signed_forward(direct, start, end)
            actual = counter.dayCount(qs, qe)
            reference_fraction = counter.yearFraction(qs, qe)
            if actual != direct(start, end) or reference_fraction != float(Fraction(actual, 360)):
                raise ValueError("unexpected 30/360 reference formula: " + str((name, start, end)))
            discrepancy = ""
            if actual != expected:
                if start > end:
                    discrepancy = label
                elif name == "30E/360 (ISDA)" and start == end == termination and start.month == 2:
                    discrepancy = "D9-30E-ISDA-ZERO"
                else:
                    raise ValueError("unclassified 30/360 difference")
            rows.append(("EXT-%06d-%d" % (index, len(rows)), name, market, term, start.isoformat(), end.isoformat(),
                         actual, expected, repr(reference_fraction), repr(float(Fraction(expected, 360))), discrepancy))
            differences[discrepancy] = differences.get(discrepancy, 0) + 1
        if min(start, end) < VALID_FROM:
            continue
        for market, cal in (("WeekendsOnly", ql.WeekendsOnly()), ("TARGET", target)):
            prefix = prefixes[market]
            def ascending(a, b):
                return prefix[b.toordinal() - VALID_FROM.toordinal()] - prefix[a.toordinal() - VALID_FROM.toordinal()]
            expected = signed_forward(ascending, start, end)
            direct_expected = ascending(start, end) if start <= end else -(
                prefix[start.toordinal() - VALID_FROM.toordinal() + 1] - prefix[end.toordinal() - VALID_FROM.toordinal() + 1])
            counter = ql.Business252(cal)
            actual = counter.dayCount(qs, qe)
            reference_fraction = counter.yearFraction(qs, qe)
            if actual != direct_expected or reference_fraction != float(Fraction(actual, 252)):
                raise ValueError("unexpected Business/252 reference formula: " + str((market, start, end)))
            discrepancy = "D9-B252-SIGNED" if actual != expected and start > end else ""
            if actual != expected and not discrepancy:
                raise ValueError("unclassified Business/252 difference")
            rows.append(("EXT-%06d-%d" % (index, len(rows)), "Business/252", market, "", start.isoformat(), end.isoformat(),
                         actual, expected, repr(reference_fraction), repr(float(Fraction(expected, 252))), discrepancy))
            differences[discrepancy] = differences.get(discrepancy, 0) + 1
    write_fixture("extended-daycounts", ("id", "convention", "calendar", "termination", "start", "end", "reference_days", "oracle_days", "reference_fraction", "oracle_fraction", "discrepancy"),
                  rows, ql, args, "Bond Basis, termination-explicit 30E/360 ISDA and explicit-calendar Business/252; signed-input/zero-accrual differences narrowly classified")
    print("Classified differences:", differences)


if __name__ == "__main__":
    main()
