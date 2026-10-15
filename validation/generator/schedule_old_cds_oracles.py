#!/usr/bin/env python3
"""Independent civil-enumeration oracle for the local OldCDS raw schedule rule.

Expected values come only from Python's proleptic Gregorian datetime.date and
the frozen docs/M7_OLD_CDS_CONTRACT.md text. No SabiQuant production source,
Rust output or QuantLib behaviour is consulted.

* Q(x) scans civil days forward from x until a March/June/September/December
  twentieth; the three-month anchor shift is the next such scanned day.
* Interior points enumerate EVERY day-20 civil date in [B, E) and keep those
  whose month distance from B is a stride multiple. The count is len().
* Regularity is explicit lattice membership with Python % semantics.

The contract's closed-form count, closed-form quarter selection and literal
k-step recurrence are evaluated only as assertions that must agree exactly.
"""

import argparse
import calendar
from collections import Counter
import datetime as dt
from functools import lru_cache
import hashlib
from pathlib import Path
import platform


ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/old-cds-oracle.fixture"
CONTRACT = ROOT / "docs/M7_OLD_CDS_CONTRACT.md"
MIN_BOUNDARIES, MAX_BOUNDARIES = 2, 3_652_059
I32_MIN, I32_MAX, U32_MAX = -2**31, 2**31 - 1, 2**32 - 1
CHECKSUM_MODULUS = 2**61 - 1
FULL_LIST_LIMIT = 1024
CIVIL_MAXIMUM = 119_987
ONE_DAY = dt.timedelta(days=1)
QUARTER_MONTHS = (3, 6, 9, 12)
CYCLE_START, CYCLE_DAYS = dt.date(2000, 1, 1), 146_097
CYCLE_TENORS = tuple((n, "Months") for n in (1, 2, 3, 4, 5, 6, 9, 12, 18, 24)) + tuple(
    (n, "Years") for n in (1, 2, 3))
SUPPORTED = tuple((n, "Months") for n in (1, 2, 3, 4, 6, 12)) + ((1, "Years"),)
COLUMNS = ("id profile start end length unit limit status initial_quarter recurrence_anchor "
           "resolved_end omitted_quarter count dates regularity error_role error_input "
           "required").split()
IGNORED_HEADER_KEYS = ("platform", "reference-version")


def iso(text):
    return dt.date.fromisoformat(text)


def month_coordinate(date):
    return 12 * date.year + date.month - 1


def twentieth(coordinate):
    year, month0 = divmod(coordinate, 12)
    return dt.date(year, month0 + 1, 20)


def quarter_on_or_after(x):
    # Civil day scan only: no month arithmetic, quarter table or formula.
    current = x
    while not (current.day == 20 and current.month in QUARTER_MONTHS):
        if current == dt.date.max:
            return None
        current += ONE_DAY
    return current


def closed_form_quarter(x):
    # Contract-style shortcut used ONLY to cross-check the day scan.
    if x.month % 3 == 0 and x.day <= 20:
        return x.year, x.month, 20
    month = x.month + 3 - x.month % 3
    return (x.year + 1, month - 12, 20) if month > 12 else (x.year, month, 20)


def interior_points(anchor, end, stride):
    # Enumerate EVERY day-20 civil date in [anchor, end); keep stride
    # multiples by month distance. No k-step recurrence or count formula.
    points = []
    first = month_coordinate(anchor)
    for coordinate in range(first, month_coordinate(end) + 1):
        candidate = twentieth(coordinate)
        if anchor <= candidate < end and (coordinate - first) % stride == 0:
            points.append(candidate)
    return points


def literal_recurrence(anchor, end, stride):
    # The contract's literal "B + k*m strictly before E" (cross-check only).
    points, base, limit = [], month_coordinate(anchor), month_coordinate(end)
    k = 0
    while base + k * stride <= limit and twentieth(base + k * stride) < end:
        points.append(twentieth(base + k * stride))
        k += 1
    return points


def closed_form_count(anchor, end, stride):
    # Contract validation step 6 (cross-check only).
    b = month_coordinate(anchor)
    last = month_coordinate(end) - (1 if end.day <= 20 else 0)
    return 2 + (0 if b > last else (last - b) // stride + 1)


def on_lattice(date, anchor, stride):
    return date.day == 20 and (month_coordinate(date) - month_coordinate(anchor)) % stride == 0


@lru_cache(maxsize=256)
def evaluate(start, end, length, unit):
    """Cap-independent build_old_cds outcome, in contract precedence order."""
    assert dt.date.min <= start < end <= dt.date.max
    if unit not in ("Months", "Years") or length <= 0:
        return dict(status="schedule.invalid_old_cds_tenor")
    stride = length * (12 if unit == "Years" else 1)
    resolved_end = quarter_on_or_after(end)
    expected = closed_form_quarter(end)
    assert (resolved_end is None) == (expected[0] > 9999)
    if resolved_end is None:
        return dict(status="schedule.old_cds_date", error_role="terminal quarter",
                    error_input=end.isoformat())
    assert (resolved_end.year, resolved_end.month, resolved_end.day) == expected
    initial = quarter_on_or_after(start)
    assert initial is not None and initial <= resolved_end
    s_quarter = closed_form_quarter(start)
    assert (initial.year, initial.month, initial.day) == s_quarter
    distance = (initial - start).days
    if initial < resolved_end and distance < 30:
        anchor = quarter_on_or_after(initial + ONE_DAY)
        assert month_coordinate(anchor) - month_coordinate(initial) == 3
    else:
        anchor = initial
    interior = interior_points(anchor, end, stride)
    dates = [start] + interior + [resolved_end]
    omitted = initial if anchor != initial and initial != start else None
    # Contract consequences and cross-checks, all exact.
    assert start < anchor <= resolved_end and end <= resolved_end
    assert anchor >= end and anchor == resolved_end or interior and interior[0] == anchor
    assert all(left < right for left, right in zip(dates, dates[1:]))
    assert all(point.day == 20 and point < end for point in interior)
    assert interior == literal_recurrence(anchor, end, stride)
    assert len(dates) == closed_form_count(anchor, end, stride)
    assert distance != 0 or (initial == start and anchor != start and omitted is None)
    assert distance not in (30, 31) or anchor == initial
    assert initial != resolved_end or (anchor == initial and omitted is None)
    assert (omitted is not None) == (initial < resolved_end and 0 < distance < 30)
    assert omitted is None or omitted not in dates
    flags = tuple("R" if on_lattice(left, anchor, stride) and on_lattice(right, anchor, stride)
                  and month_coordinate(right) - month_coordinate(left) == stride else "S"
                  for left, right in zip(dates, dates[1:]))
    return dict(status="value", initial=initial, anchor=anchor, resolved_end=resolved_end,
                omitted=omitted, dates=tuple(dates), flags=flags, distance=distance)


def checksum(dates):
    total = 0
    for index, date in enumerate(dates):
        total = (total + (index + 1) * date.toordinal()) % CHECKSUM_MODULUS
    return total


def run_length(flags):
    runs = []
    for flag in flags:
        if runs and runs[-1][0] == flag:
            runs[-1][1] += 1
        else:
            runs.append([flag, 1])
    return ";".join(flag + str(count) for flag, count in runs)


def shift_months(date, count):
    # Input construction only: civil month shift with day clamped to month end.
    year, month0 = divmod(month_coordinate(date) + count, 12)
    return dt.date(year, month0 + 1, min(date.day, calendar.monthrange(year, month0 + 1)[1]))


def next_twentieth(date):
    # Input construction only: first day-20 date strictly after the day.
    current = date + ONE_DAY
    while current.day != 20:
        current += ONE_DAY
    return current


END_KINDS = (
    ("plus1d", lambda s: s + ONE_DAY),
    ("plus29d", lambda s: s + dt.timedelta(days=29)),
    ("plus30d", lambda s: s + dt.timedelta(days=30)),
    ("quarter-minus1d", lambda s: quarter_on_or_after(s + 2 * ONE_DAY) - ONE_DAY),
    ("next-quarter", lambda s: quarter_on_or_after(s + ONE_DAY)),
    ("quarter-plus1d", lambda s: quarter_on_or_after(s + ONE_DAY) + ONE_DAY),
    ("next-twentieth", next_twentieth),
    ("plus5m", lambda s: shift_months(s, 5)),
    ("plus13m", lambda s: shift_months(s, 13)),
    ("plus3y", lambda s: shift_months(s, 36)),
)


def unique(values):
    seen, kept = set(), []
    for value in values:
        if value not in seen:
            seen.add(value)
            kept.append(value)
    return kept


def cycle_definitions():
    # Every start day exactly once; end kind and tenor rotate independently
    # (coprime moduli 10 and 13), not a Cartesian product.
    for offset in range(CYCLE_DAYS):
        start = CYCLE_START + dt.timedelta(days=offset)
        name, make_end = END_KINDS[offset % len(END_KINDS)]
        length, unit = CYCLE_TENORS[offset % len(CYCLE_TENORS)]
        yield "cycle-" + name, start, make_end(start), length, unit, MAX_BOUNDARIES


def distance_definitions():
    for year in (2000, 2001, 2024, 2100, 2399):
        for month in QUARTER_MONTHS:
            quarter = dt.date(year, month, 20)
            following = quarter_on_or_after(quarter + ONE_DAY)
            for distance in (0, 1, 2, 28, 29, 30, 31, 60):
                start = quarter - dt.timedelta(days=distance)
                ends = [("distance-quarter-plus1d", quarter + ONE_DAY)]
                if distance:
                    ends.append(("distance-on-quarter", quarter))
                ends += [("distance-next-quarter", following),
                         ("distance-plus1y", dt.date(year + 1, month, 20))]
                for profile, end in ends:
                    for length, unit in SUPPORTED:
                        yield profile, start, end, length, unit, MAX_BOUNDARIES


def terminal_definitions():
    for year in (2026, 2028, 2100):
        for month in range(1, 13):
            for day_of_month in (1, 19, 20, 21, 28):
                start = dt.date(year, month, day_of_month)
                initial = quarter_on_or_after(start)
                distance = (initial - start).days
                following = quarter_on_or_after(initial + ONE_DAY)
                # Anchor for any requested end after A (input placement only).
                late = following if distance < 30 else initial
                for stride in (1, 2, 4, 5):
                    off_quarter, on_quarter, k = [], None, 1
                    while len(off_quarter) < 2 or on_quarter is None:
                        point = twentieth(month_coordinate(late) + k * stride)
                        if point.month not in QUARTER_MONTHS and len(off_quarter) < 2:
                            off_quarter.append(point)
                        elif point.month in QUARTER_MONTHS and on_quarter is None:
                            on_quarter = point
                        k += 1
                    for point in off_quarter + [on_quarter]:
                        for end in (point - ONE_DAY, point, point + ONE_DAY):
                            yield "terminal-stride", start, end, stride, "Months", MAX_BOUNDARIES
                for stride in (2, 4, 5):
                    quarter = quarter_on_or_after(late + ONE_DAY)
                    while (month_coordinate(quarter) - month_coordinate(late)) % stride == 0:
                        quarter = quarter_on_or_after(quarter + ONE_DAY)
                    for end in (quarter - ONE_DAY, quarter, quarter + ONE_DAY):
                        yield "terminal-quarter", start, end, stride, "Months", MAX_BOUNDARIES
                if distance:
                    for end in unique(x for x in (start + ONE_DAY, initial - ONE_DAY, initial)
                                      if x > start):
                        for stride in (1, 2, 3, 12):
                            yield "terminal-short", start, end, stride, "Months", MAX_BOUNDARIES
                if distance < 30:
                    for end in (initial + ONE_DAY, following - ONE_DAY, following):
                        for length, unit in ((1, "Months"), (3, "Months"), (6, "Months"),
                                             (1, "Years")):
                            yield "terminal-b-eq-r", start, end, length, unit, MAX_BOUNDARIES


def domain_definitions():
    for start, end in ((iso("2026-02-06"), iso("2026-11-17")),
                       (dt.date.min, iso("0001-01-02")),
                       (iso("9999-12-21"), dt.date.max)):
        for unit in ("Days", "Weeks", "Months", "Years"):
            for length in (I32_MIN, -1, 0, 1, 7, I32_MAX):
                yield "tenor-domain", start, end, length, unit, MAX_BOUNDARIES


def civil_definitions():
    early = ("0001-01-01", "0001-01-02", "0001-02-18", "0001-02-19", "0001-02-20",
             "0001-03-19", "0001-03-20", "0001-03-21")
    for start in map(iso, early):
        ends = unique(x for x in (start + ONE_DAY, iso("0001-03-20"), iso("0001-03-21"),
                                  iso("0001-06-20"), iso("0001-12-31"), iso("0002-03-20"))
                      if x > start)
        for end in ends:
            for length, unit in SUPPORTED:
                yield "civil-early", start, end, length, unit, MAX_BOUNDARIES
    late = ("9998-12-20", "9999-01-01", "9999-09-20", "9999-09-21", "9999-11-19",
            "9999-11-20", "9999-11-21", "9999-12-01", "9999-12-19")
    for start in map(iso, late):
        for end in (x for x in (iso("9999-12-19"), iso("9999-12-20")) if x > start):
            for length, unit in SUPPORTED:
                yield "civil-late", start, end, length, unit, MAX_BOUNDARIES
    for start in map(iso, ("0001-01-01", "9999-01-01", "9999-12-19", "9999-12-20",
                           "9999-12-21", "9999-12-30")):
        for end in (x for x in (iso("9999-12-21"), dt.date.max) if x > start):
            for length, unit in ((1, "Months"), (3, "Months"), (1, "Years"), (I32_MAX, "Months")):
                yield "civil-terminal-overflow", start, end, length, unit, MAX_BOUNDARIES
    huge_spans = (("0001-01-01", "9999-12-20"), ("0001-01-01", "0001-01-02"),
                  ("0001-03-20", "0001-03-21"), ("2026-02-06", "2026-11-17"),
                  ("2026-02-19", "2026-06-01"), ("2026-03-20", "2026-03-21"),
                  ("9999-01-01", "9999-12-20"), ("9999-12-19", "9999-12-20"))
    for start, end in huge_spans:
        for unit in ("Months", "Years"):
            yield "civil-huge-tenor", iso(start), iso(end), I32_MAX, unit, MAX_BOUNDARIES
    for start in map(iso, ("0001-01-01", "0001-03-01", "0001-03-20")):
        for length, unit in CYCLE_TENORS:
            yield "civil-full-range", start, iso("9999-12-20"), length, unit, MAX_BOUNDARIES
    yield "civil-full-range", dt.date.min, iso("9999-12-19"), 1, "Months", MAX_BOUNDARIES


def cap_definitions():
    bases = [x for offset, x in enumerate(cycle_definitions()) if offset % 797 == 0]
    bases += [x for x in distance_definitions() if x[1].year == 2024 and x[3:5] == (1, "Months")]
    bases += [("", dt.date.min, iso("9999-12-20"), length, unit, 0) for length, unit in CYCLE_TENORS]
    bases += [("", dt.date.min, iso("0001-01-02"), 1, "Months", 0),
              ("", iso("2026-02-19"), iso("2026-06-01"), 3, "Months", 0)]
    for _, start, end, length, unit, _ in bases:
        outcome = evaluate(start, end, length, unit)
        assert outcome["status"] == "value"
        count = len(outcome["dates"])
        for limit in unique(x for x in (count - 1, count, count + 1, MIN_BOUNDARIES)
                            if MIN_BOUNDARIES <= x <= MAX_BOUNDARIES):
            yield "caps", start, end, length, unit, limit


def precedence_definitions():
    cases = ((iso("2026-02-06"), iso("2026-11-17"), 3, "Months"),
             (iso("2026-02-06"), iso("2026-11-17"), 0, "Months"),
             (dt.date.min, dt.date.max, 1, "Months"),
             (iso("9999-12-21"), dt.date.max, 1, "Weeks"))
    for limit in (0, 1, MAX_BOUNDARIES + 1, U32_MAX):
        for start, end, length, unit in cases:
            yield "precedence", start, end, length, unit, limit
    yield "precedence", iso("9999-12-21"), dt.date.max, -1, "Months", MIN_BOUNDARIES
    yield "precedence", iso("9999-12-21"), dt.date.max, I32_MIN, "Years", MIN_BOUNDARIES
    yield "precedence", dt.date.min, dt.date.max, 1, "Months", MIN_BOUNDARIES
    yield "precedence", dt.date.min, iso("9999-12-21"), I32_MAX, "Years", MIN_BOUNDARIES
    yield "precedence", dt.date.min, iso("0001-01-02"), 1, "Months", MIN_BOUNDARIES
    yield "precedence", dt.date.min, iso("9999-12-20"), 1, "Months", MAX_BOUNDARIES


def definitions():
    yield from cycle_definitions()
    yield from distance_definitions()
    yield from terminal_definitions()
    yield from domain_definitions()
    yield from civil_definitions()
    yield from cap_definitions()
    yield from precedence_definitions()


def oracle_row(profile, start, end, length, unit, limit):
    row = dict(profile=profile, start=start.isoformat(), end=end.isoformat(),
               length=str(length), unit=unit, limit=str(limit))
    # Validation order 1: the boundary cap precedes every other check.
    if not MIN_BOUNDARIES <= limit <= MAX_BOUNDARIES:
        return dict(row, status="schedule.invalid_boundary_limit"), None
    outcome = evaluate(start, end, length, unit)
    if outcome["status"] != "value":
        return dict(row, status=outcome["status"], error_role=outcome.get("error_role", ""),
                    error_input=outcome.get("error_input", "")), outcome
    dates, flags = outcome["dates"], outcome["flags"]
    if len(dates) > limit:
        return dict(row, status="schedule.boundary_limit_exceeded", required=str(len(dates))), outcome
    if len(dates) <= FULL_LIST_LIMIT:
        encoded_dates = ";".join(date.isoformat() for date in dates)
        encoded_flags = ";".join(flags)
    else:
        encoded_dates = "checksum:" + str(checksum(dates))
        encoded_flags = "rle:" + run_length(flags)
    omitted = outcome["omitted"]
    return dict(row, status="value", initial_quarter=outcome["initial"].isoformat(),
                recurrence_anchor=outcome["anchor"].isoformat(),
                resolved_end=outcome["resolved_end"].isoformat(),
                omitted_quarter="" if omitted is None else omitted.isoformat(),
                count=str(len(dates)), dates=encoded_dates, regularity=encoded_flags), outcome


def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def anchor_class(outcome):
    if outcome["initial"] == outcome["resolved_end"]:
        return "a-equals-r"
    if outcome["distance"] == 0:
        return "distance0-shift"
    if outcome["distance"] < 30:
        return "distance1to29-shift-omits-a"
    return "distance30plus-retains-a"


def render(stamp):
    evaluate.cache_clear()
    lines = ["\t".join(COLUMNS)]
    statuses, profiles, flags, omitted, anchors, shapes = (Counter() for _ in range(6))
    inputs, checksum_rows, cross_checked, maximum = set(), 0, 0, (0, None)
    for serial, (profile, start, end, length, unit, limit) in enumerate(definitions()):
        row, outcome = oracle_row(profile, start, end, length, unit, limit)
        row["id"] = f"SCHEDULE-OLDCDS-{serial:06d}"
        lines.append("\t".join(row.get(column, "") for column in COLUMNS))
        statuses[row["status"]] += 1
        profiles[profile] += 1
        inputs.add((start, end, length, unit, limit))
        # Every row reaching the count step asserted enumeration == closed form.
        cross_checked += outcome is not None and outcome["status"] == "value"
        if row["status"] != "value":
            continue
        count = len(outcome["dates"])
        flags.update(outcome["flags"])
        omitted["some" if outcome["omitted"] else "none"] += 1
        anchors[anchor_class(outcome)] += 1
        shapes["b-equals-r" if outcome["anchor"] == outcome["resolved_end"] else "b-before-r"] += 1
        checksum_rows += count > FULL_LIST_LIMIT
        if count > maximum[0]:
            maximum = (count, f"{start}..{end} {length}{unit}")
    assert maximum == (CIVIL_MAXIMUM, f"{dt.date.min}..9999-12-20 1Months")
    body = "\n".join(lines) + "\n"
    body_bytes = body.encode("utf-8")
    counts = lambda counter: ";".join(f"{key}={value}" for key, value in sorted(counter.items()))
    header = {
        "schema": "1", "fixture-set": "SCHEDULE-OLDCDS-CIVIL-ORACLE", "domain": "schedule",
        "description": "Local OldCDS raw rule; every start day of the Gregorian 400-year cycle, distance and terminal controls, stored-tenor domain, civil extremes and exact final caps",
        "reference": "oracle:Python datetime civil-day and twentieth enumeration",
        "reference-version": "Python " + platform.python_version(),
        "reference-source": "M7_OLD_CDS_CONTRACT adopted local definition; Q(x) by forward civil-day scan; interior = every day-20 date in [B,E) with month-distance congruence; count = len(); flags by explicit lattice membership; no Rust or QuantLib outputs",
        "tolerance": "TOL-E0 exact dates, provenance, counts, interval regularity, typed statuses and error roles",
        "reproduction": "generator-sha256=" + hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
                        + "; python3 -I validation/generator/schedule_old_cds_oracles.py",
        "adopted-contract-sha256": hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),
        "generated-at": stamp, "platform": platform.system() + " " + platform.machine(),
        "scope": "NullCalendar with Unadjusted ordinary and termination conventions, so raw equals adjusted; no calendar guard, adjustment, ICMA, coupon or historical CDS premium claim",
        "cycle": "2000-01-01..2399-12-31 all 146097 starts once; end kind = day index mod 10 and stored tenor = day index mod 13 (period 130), not a Cartesian product; limit 3652059",
        "encoding": "limit = max_boundaries passed; status = value or stable ScheduleError kind; provenance, count, dates and regularity only on value rows; empty omitted_quarter on a value row means None; dates = ';'-joined raw boundaries S..R, or checksum:<n> when count > 1024; regularity = R (Regular) or S (Stub) per raw interval ';'-joined, or rle:<flag><run>;... on checksum rows; error_role/error_input only for schedule.old_cds_date; required only for schedule.boundary_limit_exceeded (= exact final count)",
        "checksum-formula": "n = sum over i=0..count-1 of (i+1)*ordinal(date_i) mod 2305843009213693951 (2^61-1), dates in schedule order from S to R; ordinal(0001-01-01) = 1 as Python date.toordinal, i.e. Rust days since 0001-01-01 plus one",
        "record-count": str(len(lines) - 1),
        "unique-input-count": str(len(inputs)),
        "status-counts": counts(statuses),
        "profile-counts": counts(profiles),
        "flag-counts": counts(flags),
        "omitted-quarter-counts": counts(omitted),
        "anchor-class-counts": counts(anchors),
        "terminal-shape-counts": counts(shapes),
        "checksum-row-count": str(checksum_rows),
        "closed-form-count-check": f"rows={cross_checked}; disagreements=0; every value and boundary_limit_exceeded row asserts len(enumerated dates) == contract step-6 count, literal B+k*m recurrence and closed-form quarter selection",
        "civil-maximum-count": f"{CIVIL_MAXIMUM} at {maximum[1]}",
        "body-sha256": hashlib.sha256(body_bytes).hexdigest(), "body-fnv64": fnv64(body_bytes),
    }
    return "\n".join(key + ": " + value for key, value in header.items()) + "\n\n" + body


def masked(text):
    header, body = text.split("\n\n", 1)
    lines = [line.split(": ", 1)[0] + ": *" if line.split(": ", 1)[0] in IGNORED_HEADER_KEYS
             else line for line in header.split("\n")]
    return lines, body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_bytes().decode("utf-8")
        stamp = next(line.split(": ", 1)[1] for line in previous.split("\n")
                     if line.startswith("generated-at: "))
        text = render(stamp)
        # Byte-exact except the recorded Python/platform labels; the original
        # timestamp is retained. Exact arithmetic is portable.
        if masked(previous) != masked(text):
            raise SystemExit("OldCDS civil oracle stale; regenerate and review")
        action = "verified"
    else:
        text = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(text.encode("utf-8"))
        action = "generated"
    print(action, DEST.relative_to(ROOT), next(line for line in text.split("\n")
                                             if line.startswith("record-count: ")))


if __name__ == "__main__":
    main()
