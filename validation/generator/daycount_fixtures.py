#!/usr/bin/env python3
"""Actual QuantLib 1.43 core day counts versus an independent Fraction oracle."""

import argparse
import calendar
import datetime as dt
from fractions import Fraction
import hashlib
import math
import pathlib
import platform
import struct
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/daycount/core-daycounts.fixture"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
MIN = dt.date(1901, 1, 1)
MAX = dt.date(2199, 12, 31)


def samples():
    pairs = {(MIN, MAX), (MAX, MIN), (MIN, MIN), (MAX, MAX)}
    for year in (1901, 1999, 2000, 2003, 2004, 2023, 2024, 2099, 2100, 2199):
        for center in (dt.date(year, 3, 1), dt.date(year, 12, 31)):
            window = [center + dt.timedelta(days=n) for n in range(-9, 10)]
            window = [date for date in window if MIN <= date <= MAX]
            pairs.update((start, end) for start in window for end in window)
    for year in range(1901, 2200):
        for month in range(1, 13):
            last = dt.date(year, month, calendar.monthrange(year, month)[1])
            pairs.add((last - dt.timedelta(days=2), last))
            if last < MAX:
                pairs.add((last, last + dt.timedelta(days=1)))
    # Explicit SplitMix64 bit sequence avoids random-module version changes.
    state, mask = 0x4443435F4D335F31, (1 << 64) - 1
    ordinals = []
    for _ in range(8192):
        state = (state + 0x9E3779B97F4A7C15) & mask
        value = state
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & mask
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & mask
        value ^= value >> 31
        ordinals.append(MIN.toordinal() + value % ((MAX - MIN).days + 1))
    pairs.update((dt.date.fromordinal(a), dt.date.fromordinal(b))
                 for a, b in zip(ordinals[::2], ordinals[1::2]))
    return sorted(pairs)


def weighted_prefixes():
    # Independent daily classification, not the Rust endpoint split formula.
    prefix = {}
    common, leap = 0, 0
    date = MIN
    while date <= MAX:
        prefix[date] = (common, leap)
        if calendar.isleap(date.year):
            leap += 1
        else:
            common += 1
        date += dt.timedelta(days=1)
    return prefix


def exact_values(name, start, end, prefix):
    count = (end - start).days
    if name == "Actual/Actual (ISDA)":
        common_s, leap_s = prefix[start]
        common_e, leap_e = prefix[end]
        value = Fraction(common_e - common_s, 365) + Fraction(leap_e - leap_s, 366)
    elif name == "30E/360":
        def point(date):
            return date.year * 360 + (date.month - 1) * 30 + min(date.day, 30)
        count = point(end) - point(start)
        value = Fraction(count, 360)
    else:
        value = Fraction(count, 360 if name == "Actual/360" else 365)
    return count, value


def ulps(left, right):
    def key(value):
        bits = struct.unpack(">q", struct.pack(">d", value))[0]
        return -(1 << 63) - bits if bits < 0 else bits
    return abs(key(left) - key(right))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=pathlib.Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    import QuantLib as ql
    if ql.__version__ != "1.43" or not args.quantlib_wheel.name.lower().startswith("quantlib-1.43-"):
        parser.error("reference runtime and wheel must both be QuantLib 1.43")
    installed = pathlib.Path(ql.__file__).resolve().parent.parent
    with zipfile.ZipFile(args.quantlib_wheel) as wheel:
        members = [name for name in wheel.namelist() if name.startswith("QuantLib/") and not name.endswith("/")]
        if not members:
            parser.error("wheel contains no QuantLib module")
        for name in members:
            if hashlib.sha256((installed / name).read_bytes()).digest() != hashlib.sha256(wheel.read(name)).digest():
                parser.error("installed runtime does not match wheel: " + name)
    counters = (
        ("Actual/360", ql.Actual360(), 0),
        ("Actual/365 (Fixed)", ql.Actual365Fixed(), 0),
        ("Actual/Actual (ISDA)", ql.ActualActual(ql.ActualActual.ISDA), 8),
        ("30E/360", ql.Thirty360(ql.Thirty360.European), 0),
    )
    prefix = weighted_prefixes()
    columns = ("id", "convention", "start", "end", "reference_days", "oracle_days",
               "reference_fraction", "oracle_fraction", "oracle_numerator", "oracle_denominator", "tolerance",
               "discrepancy", "reference_error", "reference_behavior")
    lines = ["\t".join(columns)]
    max_distance = {name: 0 for name, _, _ in counters}
    discrepancies = {}
    for index, (start, end) in enumerate(samples()):
        qstart = ql.Date(start.day, start.month, start.year)
        qend = ql.Date(end.day, end.month, end.year)
        for counter_index, (name, counter, tolerance) in enumerate(counters):
            expected_count, exact = exact_values(name, start, end, prefix)
            reference_count = counter.dayCount(qstart, qend)
            expected_fraction = float(exact)
            discrepancy, error, behavior = "", "", "value"
            try:
                reference_fraction = counter.yearFraction(qstart, qend)
            except RuntimeError as exception:
                # Confirmed bounded-reference intermediate, not a general escape.
                if name != "Actual/Actual (ISDA)" or start == end or min(start.year, end.year) != 2199 or "2200" not in str(exception):
                    raise
                reference_fraction = None
                discrepancy = "D8-ISDA-MAX-YEAR-INTERMEDIATE"
                behavior = "raises"
                error = str(exception).replace("\n", "\\n").replace("\t", "\\t")
            if reference_fraction is not None:
                if not math.isfinite(reference_fraction):
                    raise ValueError("nonfinite reference result: " + str((name, start, end)))
                distance = ulps(reference_fraction, expected_fraction)
                if distance > tolerance:
                    # Same-year QL combines -1 and two quantities in [0,1].
                    # Four roundings <=4u plus one oracle rounding <=u: 5u.
                    # 3*epsilon=6u is a conservative absolute bound, NOT an
                    # expansion of SabiQuant's four-ULP oracle contract.
                    if name != "Actual/Actual (ISDA)" or start.year != end.year or abs(reference_fraction - expected_fraction) > 3 * sys.float_info.epsilon:
                        raise ValueError("unexpected reference discrepancy: " + str((name, start, end, distance)))
                    discrepancy = "D5-ISDA-SAME-YEAR-CANCELLATION"
                max_distance[name] = max(max_distance[name], distance)
            if reference_count != expected_count:
                raise ValueError("unexpected reference discrepancy: " + str((name, start, end, reference_count, expected_count, distance)))
            if discrepancy:
                discrepancies[discrepancy] = discrepancies.get(discrepancy, 0) + 1
            row = ("DCC-%06d-%d" % (index, counter_index), name, start.isoformat(), end.isoformat(),
                   reference_count, expected_count, "" if reference_fraction is None else repr(reference_fraction), repr(expected_fraction),
                   exact.numerator, exact.denominator, "TOL-E2" if tolerance else "TOL-E1", discrepancy, error, behavior)
            lines.append("\t".join(map(str, row)))
    metadata = {
        "schema": "1", "fixture-set": "CORE-DAYCOUNTS", "domain": "daycount",
        "description": "Four core conventions; weighted daily Fraction oracle; explicit same-year conditioning and maximum-year reference issue",
        "reference": "QuantLib", "reference-version": ql.__version__,
        "reference-source": "Attested QuantLib wheel public API; independent Python datetime/Fraction oracle",
        "tolerance": "TOL-E1 (0 ULP); TOL-E2 (reference 8 ULP, Rust oracle 4 ULP)",
        "reproduction": "generator-sha256=" + REVISION + "; python3 validation/generator/daycount_fixtures.py --quantlib-wheel " + args.quantlib_wheel.name,
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
        "quantlib-wheel": args.quantlib_wheel.name,
        "quantlib-wheel-sha256": hashlib.sha256(args.quantlib_wheel.read_bytes()).hexdigest(),
        "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995; wheel source commit not independently attested",
    }
    body = "\n".join(lines) + "\n"
    if args.check:
        old_header, old_body = DEST.read_text().split("\n\n", 1)
        old = dict(line.split(": ", 1) for line in old_header.splitlines())
        stable = metadata.keys() - {"generated-at", "platform", "python-version"}
        if old_body != body or any(old.get(key) != metadata[key] for key in stable):
            raise SystemExit("fixture differs: " + str(DEST))
        action = "verified"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n" + body).encode("utf-8"))
        action = "generated"
    print(action, DEST.relative_to(ROOT), "records=" + str(len(lines) - 1))
    print("Maximum reference/oracle ULP distances:", max_distance)
    print("Classified reference discrepancies:", discrepancies)


if __name__ == "__main__":
    main()
