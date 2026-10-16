#!/usr/bin/env python3
"""Independent integer/rational oracle for explicit EOM-qualified US 30/360.

Only the adopted contract, Python Gregorian dates, endpoint coordinates and
Fraction supply expected values. No SabiQuant source or reference runtime is
read. The primary coordinate/table oracle is independently cross-checked
against the contract's literal ordered rules, for every emitted row.
"""

import argparse
import calendar
from collections import Counter
import datetime as dt
from fractions import Fraction
import hashlib
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/daycount/us-30360-oracle.fixture"
CONTRACT = ROOT / "docs/M3_US_AFB_CONTRACT.md"
SOURCE_REVIEW = ROOT / "docs/M3_THIRTY360_US_SOURCE_REVIEW.md"
DAY = dt.timedelta(days=1)
MIN, MAX = dt.date.min, dt.date.max
CYCLE_START, CYCLE_DAYS = dt.date(2000, 1, 1), 146097
COLUMNS = "id profile anchor start end eom days numerator denominator fraction".split()
IGNORED = {"reference-version", "platform"}


def last(year, month):
    return dt.date(year, month, calendar.monthrange(year, month)[1])


def feb_end(date):
    return date.month == 2 and date.day == calendar.monthrange(date.year, 2)[1]


def endpoint_oracle(start, end, eom):
    """Map endpoints to exact synthetic month coordinates plus a day table.

    This does not mutate the source dates or execute the four ordered rules.
    The right-hand day is selected from the original February predicates and
    the already determined left coordinate's day class.
    """
    left, right = sorted((start, end))
    left_day = 30 if (eom and feb_end(left)) else min(left.day, 30)
    right_table = {
        "both-february": 30,
        "end-31-after-day-30": 30,
        "ordinary": right.day,
    }
    if eom and feb_end(left) and feb_end(right):
        right_class = "both-february"
    elif right.day == 31 and left_day == 30:
        right_class = "end-31-after-day-30"
    else:
        right_class = "ordinary"
    a = 360 * left.year + 30 * (left.month - 1) + left_day
    b = 360 * right.year + 30 * (right.month - 1) + right_table[right_class]
    forward = b - a
    assert 0 <= forward <= 3599640
    return -forward if start > end else forward


def literal_rules(start, end, eom):
    """Cross-check only: the adopted ordered four-rule specification."""
    left, right = sorted((start, end))
    d1, d2 = left.day, right.day
    original_first_feb, original_last_feb = feb_end(left), feb_end(right)
    if eom and original_first_feb and original_last_feb:
        d2 = 30
    if eom and original_first_feb:
        d1 = 30
    if d2 == 31 and d1 in (30, 31):
        d2 = 30
    if d1 == 31:
        d1 = 30
    count = 360 * (right.year - left.year) + 30 * (right.month - left.month) + d2 - d1
    return -count if start > end else count


def anniversary(date, year):
    return dt.date(year, date.month, min(date.day, calendar.monthrange(year, date.month)[1]))


def definitions():
    # All 146097 dates are anchors under both flags. Each date receives ONE
    # endpoint profile (serial mod 12); this is deliberately not Cartesian.
    for serial in range(CYCLE_DAYS):
        anchor = CYCLE_START + serial * DAY
        profile = serial % 12
        month_last = last(anchor.year, anchor.month)
        pairs = (
            (anchor, anchor),
            (anchor, anchor + DAY),
            (anchor, anchor - DAY),
            (anchor, month_last),
            (anchor, month_last + DAY),
            (anchor, anchor.replace(day=1)),
            (anchor, last(anchor.year, 2)),
            (anchor, dt.date(anchor.year, 3, 31)),
            (anchor, anniversary(anchor, anchor.year + 1)),
            (anchor, anniversary(anchor, anchor.year - 1)),
            (MIN, anchor),
            (anchor, MAX),
        )
        yield "C%02d" % profile, anchor, *pairs[profile]

    # Five documented classes for EVERY year 0001..9999, both flags.
    # Other-month coverage rotates one month per year; the hand suite owns
    # the separate all-month-end Cartesian coverage.
    for year in range(1, 10000):
        feb = last(year, 2)
        direction = -1 if year % 2 else 1
        other = last(year, 1 + (year - 1) % 12)
        neighbor = other + DAY if other < MAX else other - DAY
        adjacent = last(year + 1, 2) if year < 9999 else last(year - 1, 2)
        pairs = (
            ("Y-FSTART", feb, feb, dt.date(year, 3, 31)),
            ("Y-FEND", feb, dt.date(year, 1, 31), feb),
            ("Y-BOTHF", feb, feb, adjacent),
            ("Y-MEDGE", other, other, neighbor),
            ("Y-FMINUS1", feb, feb - DAY, dt.date(year, 3, 31)),
        )
        for profile, anchor, start, end in pairs:
            start, end = sorted((start, end))
            yield profile, anchor, *((end, start) if direction < 0 else (start, end))

    # Cartesian civil-limit controls, including month/day transitions. Every
    # endpoint is representable; no DateMIN-1 or DateMAX+1 construction.
    edge = (
        MIN, MIN + DAY, dt.date(1, 1, 30), dt.date(1, 1, 31),
        dt.date(1, 2, 27), dt.date(1, 2, 28), dt.date(1, 3, 1),
        dt.date(1, 3, 30), dt.date(1, 3, 31), dt.date(9999, 1, 31),
        dt.date(9999, 2, 27), dt.date(9999, 2, 28), dt.date(9999, 3, 1),
        dt.date(9999, 3, 30), dt.date(9999, 3, 31),
        dt.date(9999, 12, 30), MAX - DAY, MAX,
    )
    # MAX-1 equals December30 above; retain it only once in this explicit set.
    edge = sorted(set(edge))
    assert len(edge) == 17
    for start in edge:
        for end in edge:
            yield "LIMIT", start, start, end

    # +/-1 neighborhoods around each discriminating endpoint, including
    # common/leap/century years and both orientations. Separate profiles
    # remain visible even when an input also occurs in the larger corpus.
    centers = (
        ("2023-02-28", "2023-03-31"),
        ("2024-02-29", "2024-03-31"),
        ("2023-02-28", "2024-02-29"),
        ("2024-02-29", "2025-02-28"),
        ("2023-01-31", "2023-02-28"),
        ("2024-02-28", "2024-03-31"),
        ("1900-02-28", "1900-03-31"),
        ("2000-02-29", "2000-03-31"),
        ("2100-02-28", "2100-03-31"),
        ("2400-02-29", "2400-03-31"),
        ("2023-01-30", "2023-01-31"),
    )
    for start_text, end_text in centers:
        center_start, center_end = dt.date.fromisoformat(start_text), dt.date.fromisoformat(end_text)
        for start_delta in (-1, 0, 1):
            for end_delta in (-1, 0, 1):
                start, end = center_start + start_delta * DAY, center_end + end_delta * DAY
                yield "EDGE-F", center_start, start, end
                yield "EDGE-R", center_start, end, start


def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def counts(counter):
    return ";".join(f"{key}={value}" for key, value in sorted(counter.items()))


def render(stamp):
    rows = ["\t".join(COLUMNS)]
    profiles, directions, zero_classes, flags = (Counter() for _ in range(4))
    inputs, differing = set(), 0
    cycle_anchors = [set(), set()]
    years = {name: set() for name in ("Y-FSTART", "Y-FEND", "Y-BOTHF", "Y-MEDGE", "Y-FMINUS1")}
    for serial, (profile, anchor, start, end) in enumerate(definitions()):
        pair_days = []
        for eom in (False, True):
            days = endpoint_oracle(start, end, eom)
            assert days == literal_rules(start, end, eom)
            fraction = Fraction(days, 360)
            bits = struct.unpack(">Q", struct.pack(">d", float(fraction)))[0]
            assert days != 0 or bits == 0
            row = ("US-%06d-%d" % (serial, eom), profile, anchor.isoformat(),
                   start.isoformat(), end.isoformat(), int(eom), days,
                   fraction.numerator, fraction.denominator, "0x%016x" % bits)
            rows.append("\t".join(map(str, row)))
            profiles[profile] += 1
            direction = "equal" if start == end else "forward" if start < end else "reverse"
            directions[direction] += 1
            zero_classes["equal" if start == end else "synthetic-" + direction if days == 0 else "nonzero"] += 1
            flags[str(int(eom))] += 1
            inputs.add((start, end, eom))
            pair_days.append(days)
            if profile.startswith("C"):
                cycle_anchors[int(eom)].add(anchor)
        differing += pair_days[0] != pair_days[1]
        if profile in years:
            years[profile].add(anchor.year)
    assert all(len(anchors) == CYCLE_DAYS for anchors in cycle_anchors)
    assert all(len(values) == 9999 for values in years.values())
    assert all(zero_classes[key] > 0 for key in ("equal", "synthetic-forward", "synthetic-reverse"))
    body = ("\n".join(rows) + "\n").encode("utf-8")
    header = {
        "schema": "1", "fixture-set": "US-30360-INTEGER-RATIONAL-ORACLE", "domain": "daycount",
        "description": "Explicit US EOM off/on; all Gregorian-cycle anchors with rotating endpoint profiles, five classes per civil year, limits and +/-1 neighborhoods",
        "reference": "oracle:Python datetime/calendar/Fraction",
        "reference-version": "Python " + platform.python_version(),
        "reference-source": "Adopted M3_US_AFB_CONTRACT; synthetic endpoint coordinate/table oracle; literal ordered rules are cross-check only; no Rust or QuantLib output",
        "tolerance": "TOL-E0 integer days; TOL-E1 0 ULP from correctly rounded signed days/360; every zero is positive",
        "reproduction": "generator-sha256=" + hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
                        + "; python3 -I validation/generator/daycount_us_oracles.py",
        "adopted-contract-sha256": hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),
        "source-review-sha256": hashlib.sha256(SOURCE_REVIEW.read_bytes()).hexdigest(),
        "adopted-contract-fnv64": fnv64(CONTRACT.read_bytes()),
        "source-review-fnv64": fnv64(SOURCE_REVIEW.read_bytes()),
        "generator-fnv64": fnv64(Path(__file__).read_bytes()),
        "generated-at": stamp, "platform": platform.system() + " " + platform.machine(),
        "scope": "Local explicit EOM-qualified contract from reviewed ROC-v4 A001 FIX-column snapshot; full SIA/SIFMA book unread; no universal ISO/FpML/name equivalence or financial acceptance",
        "cycle": "2000-01-01..2399-12-31 all146097 anchors under both flags; endpoint profile=zero-based day index mod12; rotating profiles, not Cartesian",
        "year-classes": "Five classes per year0001..9999 under both flags; odd years reverse endpoints; Y-MEDGE rotates one month per year, not all month pairs",
        "profile-encoding": "C00 equal; C01 nextday; C02 prevday; C03 monthend; C04 monthend+1; C05 monthstart; C06 Febend; C07 Mar31; C08 nextyear anniversary; C09 prevyear anniversary; C10 MIN-to-anchor; C11 anchor-to-MAX; Y-FSTART Febend-Mar31; Y-FEND Jan31-Febend; Y-BOTHF adjacent-Febends; Y-MEDGE rotating-monthend neighbor; Y-FMINUS1 Febend-1-Mar31; LIMIT 17x17 endpoints; EDGE-F/EDGE-R 11 center-pairs x3x3 neighborhoods",
        "encoding": "anchor is coverage input, not a financial reference date; eom 0/1; days exact signed synthetic count; numerator/denominator reduced exact days/360; fraction correctly rounded binary64 bits; paired rows differ only by eom",
        "record-count": str(len(rows) - 1), "unique-input-count": str(len(inputs)),
        "profile-counts": counts(profiles), "direction-counts": counts(directions),
        "zero-class-counts": counts(zero_classes), "eom-counts": counts(flags),
        "eom-different-pair-count": str(differing),
        "oracle-cross-check": "every emitted integer agrees with independently written literal four-rule evaluation; disagreements=0",
        "body-sha256": hashlib.sha256(body).hexdigest(), "body-fnv64": fnv64(body),
    }
    return ("\n".join(key + ": " + value for key, value in header.items()) + "\n\n").encode("utf-8") + body


def masked(data):
    header, body = data.decode("utf-8").split("\n\n", 1)
    return ([line.split(": ", 1)[0] + ": *" if line.split(": ", 1)[0] in IGNORED else line
             for line in header.split("\n")], body)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        old = DEST.read_bytes()
        stamp = next(line.split(": ", 1)[1] for line in old.decode("utf-8").split("\n")
                     if line.startswith("generated-at: "))
        data = render(stamp)
        if masked(data) != masked(old):
            raise SystemExit("US day-count oracle stale; regenerate and review")
        action = "verified"
    else:
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(data)
        action = "generated"
    header = data.decode("utf-8").split("\n\n", 1)[0]
    print(action, DEST.relative_to(ROOT))
    for line in header.split("\n"):
        if line.startswith(("record-count:", "unique-input-count:", "profile-counts:", "direction-counts:", "zero-class-counts:", "eom-different-pair-count:", "body-sha256:")):
            print(line)


if __name__ == "__main__":
    main()
