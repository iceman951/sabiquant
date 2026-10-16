#!/usr/bin/env python3
"""Independent repeated-year/date-membership oracle for ISDA-clarified AFB.

The adopted contract and primary-source report are the only specification
inputs. This generator never imports SabiQuant, reference libraries or another
day-count implementation. It actually removes years one at a time, rather
than duplicating the constant-time production anniversary selector. Python
datetime differences and exact Fraction arithmetic supply the expected data.
"""

import argparse
from collections import Counter
import datetime as dt
from fractions import Fraction
import hashlib
from pathlib import Path
import platform
import struct


ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/daycount/afb-oracle.fixture"
CONTRACT = ROOT / "docs/M3_US_AFB_CONTRACT.md"
SOURCE = ROOT / "docs/M3_AFB_SOURCE_REVIEW.md"
MIN, MAX = dt.date.min, dt.date.max
DAY = dt.timedelta(days=1)
CYCLE_START, CYCLE_DAYS = dt.date(2000, 1, 1), 146_097
DURATIONS = (0, 1, 2, 28, 59, 60, 90, 180, 364, 365, 366, 367, 730, 1096, 1461, 2922)
SEED, MASK = 0xAFB1999C1A41F1ED, (1 << 64) - 1
COLUMNS = ("id profile start end actual_days whole_years residual_end stub_days denominator "
           "fraction_numerator fraction_denominator oracle_fraction").split()
IGNORED_HEADER_KEYS = ("platform", "reference-version")


def february_end(year):
    return dt.date(year, 3, 1) - DAY


def previous_year(end):
    """An actual single-year move; callers never request a year before MIN."""
    assert end.year > 1
    year = end.year - 1
    if end.month == 2 and end.day in (28, 29):
        # Try the original day, clamp an impossible 29, then apply promotion.
        try:
            candidate = dt.date(year, 2, end.day)
        except ValueError:
            candidate = dt.date(year, 2, 28)
        if candidate.day == 28:
            try:
                candidate = dt.date(year, 2, 29)
            except ValueError:
                pass
        return candidate
    return dt.date(year, end.month, end.day)


def evaluate(start, end):
    assert MIN <= start <= end <= MAX
    remaining, years = end, 0
    while remaining.year > 1:
        candidate = previous_year(remaining)
        if candidate < start:
            break
        assert candidate < remaining
        remaining, years = candidate, years + 1
    # Membership is independently asked of actual dates in each residual year.
    # It is not guessed from the full interval or from end.year being leap.
    leap_days = []
    for year in range(start.year, remaining.year + 1):
        try:
            leap_day = dt.date(year, 2, 29)
        except ValueError:
            continue
        if start <= leap_day < remaining:
            leap_days.append(leap_day)
    assert len(leap_days) <= 1
    denominator = 366 if leap_days else 365
    days = (remaining - start).days
    fraction = years + Fraction(days, denominator)
    assert 0 <= years <= 9998 and 0 <= days <= 366
    return dict(actual_days=(end - start).days, whole_years=years,
                residual_end=remaining.isoformat(), stub_days=days,
                denominator=denominator, fraction_numerator=fraction.numerator,
                fraction_denominator=fraction.denominator,
                oracle_fraction="0x" + struct.pack(">d", float(fraction)).hex())


def iso(text):
    return dt.date.fromisoformat(text)


PRIMARY = (
    ("2003-11-01", "2004-05-01", Fraction(182, 366)),
    ("1999-02-01", "1999-07-01", Fraction(150, 365)),
    ("1999-07-01", "2000-07-01", Fraction(1)),
    ("2002-08-15", "2003-07-15", Fraction(334, 365)),
    ("2000-01-30", "2000-06-30", Fraction(152, 366)),
)
BOUNDARIES = (
    ("2024-02-28", "2025-02-28", 1, "2024-02-29", 1, 365),
    ("2023-02-28", "2024-02-28", 1, "2023-02-28", 0, 365),
    ("2024-02-29", "2025-02-28", 1, "2024-02-29", 0, 365),
    ("2023-02-28", "2024-02-29", 1, "2023-02-28", 0, 365),
    ("2024-02-28", "2024-02-29", 0, "2024-02-29", 1, 365),
    ("2024-02-29", "2024-03-01", 0, "2024-03-01", 1, 366),
    ("2023-03-01", "2024-02-29", 0, "2024-02-29", 365, 365),
    ("2024-02-28", "2028-02-28", 4, "2024-02-29", 1, 365),
    ("1994-02-10", "1997-06-30", 3, "1994-06-30", 140, 365),
    ("2023-12-31", "2024-02-29", 0, "2024-02-29", 60, 365),
    ("2023-12-31", "2024-03-01", 0, "2024-03-01", 61, 366),
    ("0001-01-01", "9999-12-31", 9998, "0001-12-31", 364, 365),
)


def definitions():
    for first, last, expected in PRIMARY:
        start, end = iso(first), iso(last)
        row = evaluate(start, end)
        assert Fraction(row["fraction_numerator"], row["fraction_denominator"]) == expected
        yield "primary", start, end
    for first, last, years, residual, days, denominator in BOUNDARIES:
        start, end = iso(first), iso(last)
        row = evaluate(start, end)
        assert tuple(row[key] for key in ("whole_years", "residual_end", "stub_days",
                                          "denominator")) == (years, residual, days, denominator)
        yield "hand-boundary", start, end
    for offset in range(CYCLE_DAYS):
        start = CYCLE_START + dt.timedelta(days=offset)
        duration = DURATIONS[offset % len(DURATIONS)]
        yield "cycle-plus" + str(duration), start, start + dt.timedelta(days=duration)
    for year in range(1, 10000):
        first, last, march = dt.date(year, 2, 1), february_end(year), dt.date(year, 3, 1)
        yield "all-years-feb28-march1", dt.date(year, 2, 28), march
        yield "all-years-feb1-feblast", first, last
        yield "all-years-feblast-march1", last, march
        yield "all-years-jan1-march1", dt.date(year, 1, 1), march
        yield "all-years-equal-feblast", last, last
    for year in range(2, 10000):
        previous = year - 1
        feb28, last = dt.date(previous, 2, 28), february_end(previous)
        ends = (
            (feb28, dt.date(year, 2, 28)), (last, dt.date(year, 2, 28)),
            (feb28, february_end(year)), (dt.date(previous, 3, 1), dt.date(year, 2, 28)),
            (dt.date(previous, 3, 1), february_end(year)),
            (dt.date(previous, 1, 1), dt.date(year, 12, 31)),
            (feb28, dt.date(year, 3, 1)), (last, dt.date(year, 3, 1)),
        )
        start, end = ends[year % len(ends)]
        yield "all-years-anniversary" + str(year % len(ends)), start, end
    for year in (1, 4, 100, 400, 1600, 1700, 1800, 1899, 1900, 1901, 1999, 2000,
                 2001, 2099, 2100, 2101, 2399, 2400, 2401, 9995, 9996, 9997, 9998, 9999):
        dates = []
        for candidate_year in range(max(1, year - 2), min(9999, year + 2) + 1):
            for month, day in ((2, 27), (2, 28), (2, 29), (3, 1), (3, 2)):
                try:
                    dates.append(dt.date(candidate_year, month, day))
                except ValueError:
                    pass
        dates.sort()
        for index, start in enumerate(dates):
            for end in dates[index:]:
                yield "century-feb-cartesian", start, end
    state = SEED
    for _ in range(4096):
        state = (6364136223846793005 * state + 1442695040888963407) & MASK
        first = state % MAX.toordinal() + 1
        state = (6364136223846793005 * state + 1442695040888963407) & MASK
        last = state % MAX.toordinal() + 1
        start, end = sorted((dt.date.fromordinal(first), dt.date.fromordinal(last)))
        yield "wide-deterministic", start, end
    endpoints = (MIN, MIN + DAY, dt.date(1, 2, 28), dt.date(1, 3, 1),
                 dt.date(1, 12, 31), dt.date(2, 1, 1), dt.date(4, 2, 28),
                 dt.date(4, 2, 29), dt.date(4, 3, 1), dt.date(9996, 2, 28),
                 dt.date(9996, 2, 29), dt.date(9996, 3, 1), dt.date(9999, 1, 1),
                 dt.date(9999, 2, 28), dt.date(9999, 3, 1), MAX - DAY, MAX)
    for index, start in enumerate(endpoints):
        for end in endpoints[index:]:
            yield "civil-cartesian", start, end


def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & MASK
    return f"{value:016x}"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def counts(counter):
    return ";".join(str(key) + "=" + str(value) for key, value in sorted(counter.items()))


def render(stamp):
    lines = ["\t".join(COLUMNS)]
    profiles, denominators, years = Counter(), Counter(), Counter()
    inputs, year_steps, maximum_stub = set(), 0, 0
    for serial, (profile, start, end) in enumerate(definitions()):
        row = evaluate(start, end)
        row.update(id=f"AFB-{serial:06d}", profile=profile,
                   start=start.isoformat(), end=end.isoformat())
        lines.append("\t".join(str(row[column]) for column in COLUMNS))
        profiles[profile] += 1
        denominators[row["denominator"]] += 1
        years["zero" if row["whole_years"] == 0 else "positive"] += 1
        year_steps += row["whole_years"]
        maximum_stub = max(maximum_stub, row["stub_days"])
        inputs.add((start, end))
    body = ("\n".join(lines) + "\n").encode("utf-8")
    header = {
        "schema": "1", "fixture-set": "DAYCOUNT-AFB-ORACLE", "domain": "daycount.afb",
        "description": "ISDA-clarified AFB independent repeated-year rollback, residual leap-day membership and exact Fraction oracle",
        "reference": "oracle:Python datetime/Fraction",
        "reference-version": "Python " + platform.python_version(),
        "reference-source": "each year actually rolled backward with February clamp/promotion; residual dates tested for February 29 membership; no constant-time production selector, Rust output or reference-library implementation",
        "tolerance": "TOL-E0 discrete decomposition and actual days; TOL-E2 maximum 4 ULP from correctly rounded exact rational; positive zero bit identity",
        "reproduction": "generator-sha256=" + sha(Path(__file__))
                        + "; python3 -I validation/generator/daycount_afb_oracles.py",
        "adopted-contract-sha256": sha(CONTRACT),
        "adopted-contract-fnv64": fnv64(CONTRACT.read_bytes()),
        "source-review-sha256": sha(SOURCE), "source-review-fnv64": fnv64(SOURCE.read_bytes()),
        "generator-fnv64": fnv64(Path(__file__).read_bytes()),
        "generated-at": stamp, "platform": platform.system() + " " + platform.machine(),
        "cycle": "146097 daily starts 2000-01-01..2399-12-31; durations "
                 + ",".join(map(str, DURATIONS)) + " days rotate by offset modulo 16, not a Cartesian product",
        "all-years": "five same-year February/equal profiles per civil year 1..9999; one of eight rotating adjacent-year anniversary profiles per ending year 2..9999, not their Cartesian product",
        "wide": "4096 chronological ordinal pairs from the full MIN..MAX domain; LCG64 seed "
                + f"0x{SEED:016x}" + "; no oracle scan cap, sampling is explicit",
        "encoding": "forward/equal inputs; initial residual [S,E*) includes S and excludes residual_end E*; Fraction numerator/denominator reduced with positive denominator; oracle_fraction is 0x followed by exactly 16 hex IEEE-754 binary64 bits rounded from Fraction; consumer also checks every reversal and equal start",
        "scope": "stateless explicit AFB selected interpretation only; original 1994 AFB text and full milestone/financial acceptance not asserted",
        "record-count": str(len(lines) - 1), "unique-input-count": str(len(inputs)),
        "profile-counts": counts(profiles), "denominator-counts": counts(denominators),
        "whole-year-counts": counts(years), "accepted-year-steps": str(year_steps),
        "maximum-stub-days": str(maximum_stub),
        "body-sha256": hashlib.sha256(body).hexdigest(), "body-fnv64": fnv64(body),
    }
    return ("\n".join(key + ": " + value for key, value in header.items()) + "\n\n").encode() + body


def masked(text):
    header, body = text.split(b"\n\n", 1)
    lines = [line.split(b": ", 1)[0] + b": *"
             if line.split(b": ", 1)[0].decode() in IGNORED_HEADER_KEYS else line
             for line in header.split(b"\n")]
    return lines, body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_bytes()
        stamp = next(line.split(b": ", 1)[1].decode() for line in previous.split(b"\n")
                     if line.startswith(b"generated-at: "))
        actual = render(stamp)
        if masked(previous) != masked(actual):
            raise SystemExit("AFB oracle stale; regenerate and review")
        action = "verified"
    else:
        actual = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(actual)
        action = "generated"
    header = actual.split(b"\n\n", 1)[0].decode()
    print(action, DEST.relative_to(ROOT))
    for line in header.splitlines():
        if line.startswith(("record-count:", "unique-input-count:", "accepted-year-steps:",
                            "body-sha256:")):
            print(line)


if __name__ == "__main__":
    main()
