#!/usr/bin/env python3
"""Independent civil-month and exact-Fraction oracle for OldCDS ICMA references.

Expected values come only from Python datetime/Fraction and the frozen docs
M7_OLD_CDS_CONTRACT.md, M7_OLD_CDS_ICMA_DESIGN.md and
M7_ICMA_REFERENCE_CONTRACT.md. No SabiQuant production source, Rust output or
QuantLib behaviour is consulted. This file is self-contained and does not
import the raw-rule generator; its raw rule is a second implementation:

* Q(x) is the least abstract (year, month, 20) quarter tuple of x.year and
  x.year + 1 on or after x; B = A + 3 months on the month coordinate.
* Raw interior points walk EVERY civil day in (S, E) and keep day-20 dates at
  or after B whose month distance from B is a stride multiple.
* The reference hull enumerates every month coordinate in an envelope one
  stride beyond raw S..R, keeps those congruent to B, and selects the greatest
  day-20 tuple <= S and least >= R (abstract years 0 and 10000 allowed).
* Accruals are integer day overlaps / (events * determination-period days).
"""

import argparse
from collections import Counter
import datetime as dt
from fractions import Fraction
import hashlib
from pathlib import Path
import platform


ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/old-cds-icma-oracle.fixture"
CONTRACT = ROOT / "docs/M7_OLD_CDS_CONTRACT.md"
DESIGN = ROOT / "docs/M7_OLD_CDS_ICMA_DESIGN.md"
ICMA_CONTRACT = ROOT / "docs/M7_ICMA_REFERENCE_CONTRACT.md"
MIN_PERIODS, MAX_PERIODS = 1, 3_652_058
I32_MAX = 2**31 - 1
ONE_DAY = dt.timedelta(days=1)
CYCLE_START, CYCLE_DAYS, CYCLE_STEP = dt.date(2000, 1, 1), 146_097, 23
FREQUENCIES = ("Once", "Annual", "Semiannual", "EveryFourthMonth", "Quarterly", "Bimonthly",
               "Monthly", "EveryFourthWeek", "Biweekly", "Weekly", "Daily")
EVENTS = dict(Annual=1, Semiannual=2, EveryFourthMonth=3, Quarterly=4, Bimonthly=6, Monthly=12,
              EveryFourthWeek=13, Biweekly=26, Weekly=52, Daily=365)
# Month periods of the monthly-family frequencies; day/week periods never
# normalize equal to a Months/Years tenor.
FREQUENCY_MONTHS = dict(Annual=12, Semiannual=6, EveryFourthMonth=4, Quarterly=3, Bimonthly=2,
                        Monthly=1)
PAIRS = ((1, "Months", "Monthly"), (2, "Months", "Bimonthly"), (3, "Months", "Quarterly"),
         (4, "Months", "EveryFourthMonth"), (6, "Months", "Semiannual"),
         (12, "Months", "Annual"), (1, "Years", "Annual"))
DOMAIN_TENORS = tuple((n, "Months") for n in (1, 2, 3, 4, 5, 6, 9, 12, 18, 24, I32_MAX)) + tuple(
    (n, "Years") for n in (1, 2, 3, I32_MAX))
COLUMNS = ("id profile start end length unit frequency events limit status initial_quarter "
           "recurrence_anchor resolved_end omitted_quarter raw_dates references periods "
           "lower_steps raw_fractions raw_floats span_fraction span_float required error_index "
           "error_anchor error_operation error_count").split()
IGNORED_HEADER_KEYS = ("platform", "reference-version")


def iso(text):
    return dt.date.fromisoformat(text)


def coordinate(date):
    return 12 * date.year + date.month - 1


def tuple20(month_coordinate):
    year, month0 = divmod(month_coordinate, 12)
    return year, month0 + 1, 20


def civil(month_coordinate):
    return 1 <= month_coordinate // 12 <= 9999


def date20(month_coordinate):
    assert civil(month_coordinate)
    return dt.date(*tuple20(month_coordinate))


def as_tuple(date):
    return date.year, date.month, date.day


def quarter_tuple(x):
    # Least abstract quarter twentieth on/after x among two civil years.
    return min(candidate for candidate in ((year, month, 20) for year in (x.year, x.year + 1)
                                           for month in (3, 6, 9, 12)) if candidate >= as_tuple(x))


def raw_schedule(start, end, length, unit):
    assert start < end and unit in ("Months", "Years") and length > 0
    stride = length * (12 if unit == "Years" else 1)
    resolved = quarter_tuple(end)
    assert resolved[0] <= 9999, "ICMA rows require a successfully built raw schedule"
    initial, resolved_end = dt.date(*quarter_tuple(start)), dt.date(*resolved)
    distance = (initial - start).days
    anchor = date20(coordinate(initial) + 3) if initial < resolved_end and distance < 30 else initial
    b = coordinate(anchor)
    interior, day = [], start + ONE_DAY
    while day < end:  # every civil day strictly between S and E
        if day.day == 20 and day >= anchor and (coordinate(day) - b) % stride == 0:
            interior.append(day)
        day += ONE_DAY
    dates = [start] + interior + [resolved_end]
    assert start < anchor <= resolved_end and (interior[:1] == [anchor] or anchor == resolved_end)
    assert all(left < right for left, right in zip(dates, dates[1:]))
    omitted = initial if anchor != initial and initial != start else None
    return dict(stride=stride, initial=initial, anchor=anchor, resolved_end=resolved_end,
                omitted=omitted, dates=dates)


def hull(start, resolved_end, anchor, stride):
    # Every month coordinate one stride beyond the raw span, congruence-selected.
    b = coordinate(anchor)
    window = [c for c in range(coordinate(start) - stride, coordinate(resolved_end) + stride + 1)
              if (c - b) % stride == 0]
    lower = max(c for c in window if tuple20(c) <= as_tuple(start))
    upper = min(c for c in window if tuple20(c) >= as_tuple(resolved_end))
    return lower, upper, [c for c in window if lower <= c <= upper]


def accrual(first, last, references, events):
    assert references[0] <= first <= last <= references[-1]
    total = Fraction(0)
    for left, right in zip(references, references[1:]):
        overlap = (min(last, right) - max(first, left)).days
        if overlap > 0:
            total += Fraction(overlap, events * (right - left).days)
    return total


def text_fraction(value):
    return f"{value.numerator}/{value.denominator}"


def evaluate(start, end, length, unit, frequency, limit):
    """Raw OldCDS build (max boundaries) then build_icma_context(frequency, limit)."""
    raw = raw_schedule(start, end, length, unit)
    stride, anchor, resolved_end = raw["stride"], raw["anchor"], raw["resolved_end"]
    row = dict(start=start.isoformat(), end=end.isoformat(), length=str(length), unit=unit,
               frequency=frequency, events=str(EVENTS.get(frequency, "")), limit=str(limit),
               initial_quarter=raw["initial"].isoformat(), recurrence_anchor=anchor.isoformat(),
               resolved_end=resolved_end.isoformat(),
               omitted_quarter="" if raw["omitted"] is None else raw["omitted"].isoformat(),
               raw_dates=";".join(date.isoformat() for date in raw["dates"]))
    # Existing precedence: cap, periodic source (always), frequency, match.
    if not MIN_PERIODS <= limit <= MAX_PERIODS:
        return dict(row, status="schedule.invalid_icma_period_limit"), raw
    if frequency == "Once":
        return dict(row, status="schedule.invalid_icma_frequency"), raw
    if FREQUENCY_MONTHS.get(frequency) != stride:
        return dict(row, status="schedule.icma_frequency_mismatch"), raw
    lower, upper, coordinates = hull(raw["dates"][0], resolved_end, anchor, stride)
    b = coordinate(anchor)
    lower_steps, upper_steps = (b - lower) // stride, (upper - b) // stride
    reserve = 1 if anchor < resolved_end else 0
    assert lower_steps >= 1 and reserve == (upper_steps > 0)
    # Documented bounded-generation order: lower dates first, each preceded by
    # the cap lower bound k + reserve; then upper dates, each preceded by the
    # running total + 1. Generation position: B = 0, lower 1..n, upper next.
    plan = [(k, -k, k + reserve) for k in range(1, lower_steps + 1)]
    plan += [(lower_steps + k, k, lower_steps + k) for k in range(1, upper_steps + 1)]
    raw["plan"] = plan
    for index, step, needed in plan:
        if needed > limit:
            return dict(row, status="schedule.icma_period_limit_exceeded", required=str(needed)), raw
        if not civil(b + step * stride):
            return dict(row, status="schedule.date", error_index=str(index),
                        error_anchor=anchor.isoformat(),
                        error_operation="add years" if unit == "Years" else "add months",
                        error_count=str(step * length)), raw
    references = [date20(c) for c in coordinates]
    assert len(references) - 1 == lower_steps + upper_steps
    # Minimality and membership invariants of the design.
    assert references[0] <= raw["dates"][0] < references[1]
    assert references[-2] < resolved_end <= references[-1] and anchor in references
    if raw["omitted"] is not None:
        on_lattice = (coordinate(raw["omitted"]) - b) % stride == 0
        assert (raw["omitted"] in references) == on_lattice
    events = EVENTS[frequency]
    pieces = [accrual(left, right, references, events)
              for left, right in zip(raw["dates"], raw["dates"][1:])]
    span = accrual(raw["dates"][0], resolved_end, references, events)
    assert sum(pieces, Fraction(0)) == span and all(piece > 0 for piece in pieces)
    raw.update(references=references, pieces=pieces, span=span)
    return dict(row, status="value", references=";".join(x.isoformat() for x in references),
                periods=str(len(references) - 1), lower_steps=str(lower_steps),
                raw_fractions=";".join(map(text_fraction, pieces)),
                raw_floats=";".join(repr(float(piece)) for piece in pieces),
                span_fraction=text_fraction(span), span_float=repr(float(span))), raw


def next_quarter_after(x):
    return dt.date(*quarter_tuple(x + ONE_DAY))


def shift_months(date, count):
    # Input construction only: month shift with day clamped to month end.
    year, month0 = divmod(coordinate(date) + count, 12)
    first_next = dt.date(year + (month0 == 11), (month0 + 1) % 12 + 1, 1)
    return dt.date(year, month0 + 1, min(date.day, (first_next - ONE_DAY).day))


def next_twentieth(date):
    day = date + ONE_DAY
    while day.day != 20:
        day += ONE_DAY
    return day


END_KINDS = (
    ("plus1d", lambda s: s + ONE_DAY),
    ("plus29d", lambda s: s + dt.timedelta(days=29)),
    ("plus30d", lambda s: s + dt.timedelta(days=30)),
    ("quarter-minus1d", lambda s: next_quarter_after(s + ONE_DAY) - ONE_DAY),
    ("next-quarter", next_quarter_after),
    ("quarter-plus1d", lambda s: next_quarter_after(s) + ONE_DAY),
    ("next-twentieth", next_twentieth),
    ("plus5m", lambda s: shift_months(s, 5)),
    ("plus13m", lambda s: shift_months(s, 13)),
    ("plus3y", lambda s: shift_months(s, 36)),
)


def F(text):
    return Fraction(text)


def dates(*texts):
    return [iso(text) for text in texts]


# Every hand number of docs/M7_OLD_CDS_ICMA_DESIGN.md, asserted exactly.
LOWER_TABLE = {
    "Monthly": (dates("2026-01-20", "2026-02-20", "2026-03-20", "2026-04-20", "2026-05-20",
                      "2026-06-20"), 5, F("125/372")),
    "Bimonthly": (dates("2025-12-20", "2026-02-20", "2026-04-20", "2026-06-20"), 3, F("125/372")),
    "Quarterly": (dates("2025-12-20", "2026-03-20", "2026-06-20"), 2, F("119/360")),
    "EveryFourthMonth": (dates("2025-10-20", "2026-02-20", "2026-06-20"), 2, F("124/369")),
    "Semiannual": (dates("2025-12-20", "2026-06-20"), 1, F("121/364")),
    "Annual": (dates("2025-06-20", "2026-06-20"), 1, F("121/365")),
}
ADDITIONAL_TABLE = (
    (("2026-02-06", "2026-05-01", 1, "Months", "Monthly"),
     dict(anchor="2026-03-20", resolved="2026-06-20", pieces=("15/124", "1/12", "1/6"),
          span="23/62", references=("2026-01-20", "2026-02-20", "2026-03-20", "2026-04-20",
                                    "2026-05-20", "2026-06-20"), omitted=None,
          reference_only=("2026-01-20", "2026-02-20", "2026-05-20"))),
    (("2026-03-20", "2026-08-01", 6, "Months", "Semiannual"),
     dict(anchor="2026-06-20", resolved="2026-09-20", pieces=("23/91", "46/183"),
          span="8395/16653", references=("2025-12-20", "2026-06-20", "2026-12-20"), omitted=None,
          reference_only=("2025-12-20", "2026-12-20"))),
    (("2024-02-20", "2024-06-01", 3, "Months", "Quarterly"),
     dict(anchor="2024-06-20", resolved="2024-06-20", pieces=("30/91",), span="30/91",
          references=("2023-12-20", "2024-03-20", "2024-06-20"), omitted="2024-03-20",
          reference_only=("2023-12-20", "2024-03-20"))),
)
# (inputs, cap, expected status, required or (index, anchor, operation, count))
CIVIL_TABLE = (
    (("0001-02-01", "0001-04-01", 1, "Years", "Annual"), 1, "exceeded", 2),
    (("0001-02-01", "0001-04-01", 1, "Years", "Annual"), 2, "date", (1, "0001-03-20", "add years", -1)),
    (("0001-02-01", "0001-04-01", 12, "Months", "Annual"), 1, "exceeded", 2),
    (("0001-02-01", "0001-04-01", 12, "Months", "Annual"), 2, "date", (1, "0001-03-20", "add months", -12)),
    (("0001-01-01", "0001-04-01", 1, "Months", "Monthly"), 3, "exceeded", 4),
    (("0001-01-01", "0001-04-01", 1, "Months", "Monthly"), 4, "date", (3, "0001-03-20", "add months", -3)),
    (("9999-02-01", "9999-04-01", 1, "Years", "Annual"), 1, "exceeded", 2),
    (("9999-02-01", "9999-04-01", 1, "Years", "Annual"), 2, "date", (2, "9999-03-20", "add years", 1)),
    (("9999-02-01", "9999-04-01", 12, "Months", "Annual"), 1, "exceeded", 2),
    (("9999-02-01", "9999-04-01", 12, "Months", "Annual"), 2, "date", (2, "9999-03-20", "add months", 12)),
    (("9999-02-01", "9999-10-01", 4, "Months", "EveryFourthMonth"), 3, "exceeded", 4),
    (("9999-02-01", "9999-10-01", 4, "Months", "EveryFourthMonth"), 4, "date", (4, "9999-03-20", "add months", 12)),
    (("9999-12-01", "9999-12-02", 1, "Years", "Annual"), 1, "value", ("9998-12-20", "9999-12-20")),
)


def hand_definitions():
    # Shared S=2026-02-19, E=2026-06-01: omitted A=Mar20, B=R=Jun20.
    for length, unit, frequency in PAIRS:
        start, end = iso("2026-02-19"), iso("2026-06-01")
        references, periods, span = LOWER_TABLE[frequency]

        def check(row, raw, references=references, periods=periods, span=span, frequency=frequency):
            assert row["status"] == "value" and raw["dates"] == dates("2026-02-19", "2026-06-20")
            assert raw["omitted"] == iso("2026-03-20") and raw["anchor"] == raw["resolved_end"]
            assert raw["references"] == references and int(row["periods"]) == periods
            assert raw["span"] == span and raw["pieces"] == [span]
            # A is a reference boundary for 1M/3M only.
            assert (iso("2026-03-20") in references) == (frequency in ("Monthly", "Quarterly"))
        yield "hand-lower-steps", start, end, length, unit, frequency, None, check
    assert F("1/372") + F("4/12") == F("125/372") and F("29/360") + F("1/4") == F("119/360")
    assert F("1/369") + F("1/3") == F("124/369") and F("29/364") + F("1/4") == F("30/91")
    for (start, end, length, unit, frequency), expected in ADDITIONAL_TABLE:
        def check(row, raw, expected=expected):
            assert row["status"] == "value"
            assert raw["anchor"] == iso(expected["anchor"])
            assert raw["resolved_end"] == iso(expected["resolved"])
            assert raw["pieces"] == [F(x) for x in expected["pieces"]]
            assert raw["span"] == F(expected["span"])
            assert raw["references"] == dates(*expected["references"])
            assert raw["omitted"] == (expected["omitted"] and iso(expected["omitted"]))
            # Required reference boundaries that are not raw coupon boundaries.
            assert [x for x in raw["references"] if x not in raw["dates"]] == \
                dates(*expected["reference_only"])
        yield "hand-raw-fractions", iso(start), iso(end), length, unit, frequency, None, check

    def check_exact(row, raw):
        # Mar20..Jun20 holds three Monthly periods: 1/4; 1000 * 1/8 * 1/4 = 31.25.
        assert raw["anchor"] == iso("2026-06-20") and raw["omitted"] is None
        assert raw["pieces"][0] == F("1/4") and F("1000") * F("1/8") * raw["pieces"][0] == F("125/4")
        assert sum(1 for x in raw["references"] if x <= iso("2026-06-20")) == 4
    yield "hand-exact-quarter", iso("2026-03-20"), iso("2026-08-01"), 1, "Months", "Monthly", None, check_exact

    def check_raw_4m(row, raw):
        assert raw["dates"] == dates("9999-02-01", "9999-03-20", "9999-07-20", "9999-12-20")
    for (start, end, length, unit, frequency), cap, status, detail in CIVIL_TABLE:
        def check(row, raw, status=status, detail=detail):
            assert raw["anchor"] == raw["initial"]
            if status == "exceeded":
                assert row["status"] == "schedule.icma_period_limit_exceeded"
                assert row["required"] == str(detail)
            elif status == "date":
                assert row["status"] == "schedule.date"
                assert (row["error_index"], row["error_anchor"], row["error_operation"],
                        row["error_count"]) == tuple(map(str, detail))
            else:
                assert row["status"] == "value" and raw["references"] == dates(*detail)
                assert raw["initial"] == raw["anchor"] == raw["resolved_end"] == iso("9999-12-20")
            if row["frequency"] == "EveryFourthMonth":
                check_raw_4m(row, raw)
        yield "hand-civil-caps", iso(start), iso(end), length, unit, frequency, cap, check
    for (start, end, length, unit, frequency), _, status, _ in CIVIL_TABLE:
        if status == "date":  # The same required Date cause at the maximum cap.
            yield "hand-civil-caps", iso(start), iso(end), length, unit, frequency, MAX_PERIODS, None


def cycle_definitions():
    for offset in range(0, CYCLE_DAYS, CYCLE_STEP):
        serial = offset // CYCLE_STEP
        start = CYCLE_START + dt.timedelta(days=offset)
        name, make_end = END_KINDS[serial % len(END_KINDS)]
        length, unit, frequency = PAIRS[serial % len(PAIRS)]
        yield "cycle-" + name, start, make_end(start), length, unit, frequency, None, None


def distance_definitions():
    for year in (2001, 2024, 2100):
        for month in (3, 6, 9, 12):
            quarter = dt.date(year, month, 20)
            following = next_quarter_after(quarter)
            for distance in (0, 1, 29, 30, 31, 60):
                start = quarter - dt.timedelta(days=distance)
                for end in (quarter + ONE_DAY, following, following + ONE_DAY,
                            dt.date(year + 1, month, 20)):
                    for length, unit, frequency in PAIRS:
                        yield "distance", start, end, length, unit, frequency, None, None


def civil_definitions():
    early = ("0001-01-01", "0001-01-21", "0001-02-01", "0001-02-19", "0001-02-20",
             "0001-03-19", "0001-03-20", "0001-03-21", "0001-04-01")
    for start in map(iso, early):
        for end in (x for x in map(iso, ("0001-03-21", "0001-04-01", "0001-06-20", "0001-12-31"))
                    if x > start):
            for length, unit, frequency in PAIRS:
                yield "civil-early", start, end, length, unit, frequency, None, None
    late = ("9998-12-31", "9999-02-01", "9999-06-21", "9999-09-19", "9999-09-20",
            "9999-10-01", "9999-11-21", "9999-12-01", "9999-12-19")
    for start in map(iso, late):
        for end in (x for x in map(iso, ("9999-04-01", "9999-10-01", "9999-12-02", "9999-12-20"))
                    if x > start):
            for length, unit, frequency in PAIRS:
                yield "civil-late", start, end, length, unit, frequency, None, None


def domain_definitions():
    for start, end in ((iso("2026-02-06"), iso("2026-11-17")), (iso("2026-02-19"), iso("2026-06-01"))):
        for length, unit in DOMAIN_TENORS:
            for frequency in FREQUENCIES:
                yield "frequency-domain", start, end, length, unit, frequency, None, None


def precedence_definitions():
    common = (iso("2026-02-06"), iso("2026-11-17"))
    early = (iso("0001-01-01"), iso("0001-04-01"))
    for cases in (
            (common, 3, "Months", "Quarterly", 0), (common, 3, "Months", "Once", 0),
            (common, 5, "Months", "Quarterly", MAX_PERIODS + 1),
            (common, I32_MAX, "Years", "Daily", 2**32 - 1),
            (common, 3, "Months", "Once", 1), (common, 3, "Months", "Monthly", 1),
            (common, 1, "Months", "Weekly", 1), (common, 12, "Months", "Annual", MAX_PERIODS),
            (common, 1, "Years", "Annual", MAX_PERIODS), (early, 1, "Months", "Monthly", 1),
            (early, 1, "Months", "Monthly", 2), (early, 1, "Months", "Monthly", 5),
            (early, 1, "Months", "Monthly", MAX_PERIODS)):
        (start, end), length, unit, frequency, limit = cases
        yield "precedence", start, end, length, unit, frequency, limit, None


def base_definitions():
    yield from hand_definitions()
    yield from cycle_definitions()
    yield from distance_definitions()
    yield from civil_definitions()
    yield from domain_definitions()
    yield from precedence_definitions()


def records():
    """Yield (profile, row, raw); cap None means the base-row cap policy."""
    for profile, start, end, length, unit, frequency, limit, check in base_definitions():
        if limit is None:
            # Base policy: exact period count for contexts, maximum otherwise.
            row, raw = evaluate(start, end, length, unit, frequency, MAX_PERIODS)
            if row["status"] == "value":
                limit = int(row["periods"])
            elif row["status"] == "schedule.date" and profile.startswith("civil"):
                # Cap insufficiency precedes the required Date cause.
                needed = next(n for i, _, n in raw["plan"] if str(i) == row["error_index"])
                if needed > MIN_PERIODS:
                    cap_row, cap_raw = evaluate(start, end, length, unit, frequency, needed - 1)
                    assert cap_row["status"] == "schedule.icma_period_limit_exceeded"
                    assert cap_row["required"] == str(needed)
                    yield profile + "-cap", cap_row, cap_raw
                limit = MAX_PERIODS
            else:
                limit = MAX_PERIODS
        row, raw = evaluate(start, end, length, unit, frequency, limit)
        if check is not None:
            check(row, raw)
        yield profile, row, raw
        if profile.startswith("cycle-") and row["status"] == "value" and \
                ((start - CYCLE_START).days // CYCLE_STEP) % 8 == 0:
            periods = int(row["periods"])
            for cap in (periods - 1, periods + 1):
                if MIN_PERIODS <= cap <= MAX_PERIODS:
                    cap_row, cap_raw = evaluate(start, end, length, unit, frequency, cap)
                    assert cap_row["status"] == ("value" if cap > periods else
                                                 "schedule.icma_period_limit_exceeded")
                    assert cap > periods or cap_row["required"] == str(periods)
                    yield "caps", cap_row, cap_raw


def lower_step_census():
    """Maximum lower steps over every cycle start with a late requested end."""
    maxima = {stride: (0, None) for stride in (1, 2, 3, 4, 6, 12)}
    for offset in range(CYCLE_DAYS):
        start = CYCLE_START + dt.timedelta(days=offset)
        initial = dt.date(*quarter_tuple(start))
        distance = (initial - start).days  # E = S + 400 days makes A < R.
        anchor = date20(coordinate(initial) + 3) if distance < 30 else initial
        b = coordinate(anchor)
        for stride, (best, _) in maxima.items():
            lower = max(c for c in range(coordinate(start) - stride, coordinate(start) + 1)
                        if (c - b) % stride == 0 and tuple20(c) <= as_tuple(start))
            steps = (b - lower) // stride
            if steps > best:
                maxima[stride] = (steps, start)
    assert {k: v[0] for k, v in maxima.items()} == {1: 5, 2: 3, 3: 2, 4: 2, 6: 1, 12: 1}
    assert maxima[1][1] == iso("2001-02-19")
    return maxima


def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def render(stamp):
    census = lower_step_census()
    lines = ["\t".join(COLUMNS)]
    statuses, profiles, frequencies, steps, periods = (Counter() for _ in range(5))
    inputs, pieces, hand = set(), 0, 0
    for serial, (profile, row, raw) in enumerate(records()):
        row.update(id=f"OLDCDS-ICMA-{serial:06d}", profile=profile)
        lines.append("\t".join(row.get(column, "") for column in COLUMNS))
        statuses[row["status"]] += 1
        profiles[profile] += 1
        frequencies[row["frequency"]] += 1
        hand += profile.startswith("hand-")
        inputs.add(tuple(row[k] for k in ("start", "end", "length", "unit", "frequency", "limit")))
        if row["status"] == "value":
            steps[f"{FREQUENCY_MONTHS[row['frequency']]}M:{row['lower_steps']}"] += 1
            periods[row["periods"]] += 1
            pieces += len(raw["pieces"])
    body = "\n".join(lines) + "\n"
    body_bytes = body.encode("utf-8")
    counts = lambda counter, key=str: ";".join(f"{k}={v}" for k, v in sorted(counter.items(), key=lambda x: key(x[0])))
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    header = {
        "schema": "1", "fixture-set": "SCHEDULE-OLDCDS-ICMA-ORACLE", "domain": "schedule.icma",
        "description": "OldCDS generated ICMA references; minimal B-anchored stride hull over raw S..R, exact Fraction raw-interval and whole-span accruals, caps, civil causes and frequency domain",
        "reference": "oracle:Python datetime/Fraction",
        "reference-version": "Python " + platform.python_version(),
        "reference-source": "abstract quarter tuples; civil-day walk for raw interior points; civil-month envelope congruence selection of the minimal bracketing hull; integer-overlap Fractions over determination periods; documented cap/Date generation order; no Rust or QuantLib outputs",
        "tolerance": "TOL-E0 exact dates, hulls, counts, statuses, causes and Fraction numerators/denominators; TOL-E2 4 ULP existing ICMA accrual profile for binary64 values",
        "reproduction": "generator-sha256=" + sha(Path(__file__))
                        + "; python3 -I validation/generator/schedule_old_cds_icma_oracles.py",
        "adopted-contract-sha256": sha(CONTRACT),
        "icma-design-sha256": sha(DESIGN),
        "icma-reference-contract-sha256": sha(ICMA_CONTRACT),
        "generated-at": stamp, "platform": platform.system() + " " + platform.machine(),
        "scope": "NullCalendar Unadjusted raw OldCDS schedules built with max_boundaries 3652059; explicit frequencies only; no coverage, TARGET, adjustment, coupon amount or CDS premium claim",
        "cycle": f"2000-01-01..2399-12-31 every {CYCLE_STEP}rd start ({len(range(0, CYCLE_DAYS, CYCLE_STEP))} rows); seven matching tenor/frequency pairs (mod 7) and ten relative ends (mod 10) rotate, not a Cartesian product",
        "cap-model": "rows without an explicit cap control use the exact period count for contexts and 3652058 otherwise; caps, civil-*-cap, hand-civil-caps and precedence rows state explicit caps; lower dates are generated first, each after checking k + reserve <= cap (reserve = 1 iff B < R), then upper dates after checking running periods + 1 <= cap; required = first failing check; error_index = generation position with B = 0, lower 1..n, upper following",
        "encoding": "raw provenance and raw_dates on every row (raw build always succeeds); references, periods, lower_steps, fractions and floats only on value rows; omitted_quarter empty means None; fractions are reduced num/den per raw interval ';'-joined and for S..R; floats are correctly rounded binary64 shortest repr of the same Fractions; required only for schedule.icma_period_limit_exceeded; error_index/anchor/operation/count only for schedule.date (count in stored tenor units, signed); events empty for Once",
        "lower-step-census": "all 146097 starts 2000..2399 with E = S + 400 days: max lower steps " + ";".join(f"{k}M={v[0]} first {v[1]}" for k, v in sorted(census.items())),
        "hand-control-count": f"{hand} rows asserted against every table number of M7_OLD_CDS_ICMA_DESIGN",
        "record-count": str(len(lines) - 1),
        "unique-input-count": str(len(inputs)),
        "status-counts": counts(statuses),
        "profile-counts": counts(profiles),
        "frequency-counts": counts(frequencies),
        "value-period-counts": counts(periods, int),
        "value-lower-step-counts": counts(steps),
        "raw-fraction-count": str(pieces),
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
            raise SystemExit("OldCDS ICMA oracle stale; regenerate and review")
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
