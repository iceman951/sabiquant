#!/usr/bin/env python3
"""Bounded attested QuantLib OldCDS observations, not financial oracles.

Only the public Schedule constructor and accessors are called, one isolated
worker per case. A separate, independent Python model of the adopted local
contract (docs/M7_OLD_CDS_CONTRACT.md) annotates every row: it scans calendar
days for quarterly twentieths, selects stride points by month-coordinate
congruence and adjusts with its own weekday/TARGET business-day model. It reads
no QuantLib implementation and no Rust output. Every returned vector, flag and
error is retained unchanged; differences are classified, never repaired.
"""

import argparse
from collections import Counter
import datetime as dt
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/old-cds-probes.fixture"
FROZEN = ROOT / "validation/fixtures/schedule/roll-probes.fixture"
CONTRACT = ROOT / "docs/M7_OLD_CDS_CONTRACT.md"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
HELPER = Path(__file__).with_name("schedule_probes.py")
HELPER_REVISION = hashlib.sha256(HELPER.read_bytes()).hexdigest()
CONTRACT_REVISION = hashlib.sha256(CONTRACT.read_bytes()).hexdigest()
SPEC = importlib.util.spec_from_file_location("frozen_old_cds_attestation", HELPER)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load frozen schedule attestation helper")
REFERENCE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REFERENCE)
CONVENTIONS = ("Unadjusted", "Following", "ModifiedFollowing", "Preceding",
               "ModifiedPreceding", "HalfMonthModifiedFollowing", "Nearest")
UNITS = ("Days", "Weeks", "Months", "Years")
DOMAIN_CONTROLS = ((0, "Months"), (-1, "Months"), (1, "Days"), (1, "Weeks"),
                   (5, "Days"), (2, "Weeks"), (0, "Years"))
TIMEOUT = 10
MAX_DATES = 100
MAX_SPAN_DAYS = 1096
INPUT_ENVELOPE = (dt.date(2025, 1, 1), dt.date(2029, 12, 31))
OUTPUT_ENVELOPE = ("2025-01-01", "2030-12-31")
# The 551 OLD-TENOR cases reproduce a temporary, uncommitted probe run whose
# JSON had this digest; --compare-temporary-json re-checks it field by field.
TEMPORARY_ROWS = 551
TEMPORARY_SHA256 = "e83ca664c0d537d511cc9958ca0e7a06534579fb932db5b99258e2846999e35d"
QUARTER_MONTHS = (3, 6, 9, 12)
ONE_DAY = dt.timedelta(days=1)
CLASSES = ("exact", "flags-only", "T1-offquarter-exact-end", "T2-adjusted-end-coincidence",
           "S1-unadjusted-first", "S1+T2", "domain-rejected", "local-AdjustedOrder",
           "outside-options", "UNCLASSIFIED")


def scenarios():
    result, serial = [], Counter()

    def add(prefix, group, start, end, length, unit="Months", calendar="Null",
            convention="Unadjusted", termination="Unadjusted"):
        result.append(dict(id="%s-%04d" % (prefix, serial[prefix]), group=group, start=start,
                           end=end, length=length, unit=unit, calendar=calendar,
                           convention=convention, termination=termination))
        serial[prefix] += 1

    # OLD-TENOR: the temporary 551-case probe, same ids, inputs and order.
    for start in ("2026-02-06", "2026-02-18", "2026-02-19", "2026-03-19", "2026-03-20", "2026-03-21"):
        for end in ("2026-06-19", "2026-06-20", "2026-06-21", "2026-08-19", "2026-08-20",
                    "2026-08-21", "2026-11-19", "2026-11-20", "2026-11-21"):
            for length in (1, 2, 3, 4, 6, 12):
                add("OLD-TENOR", "civil-stop-and-first-quarter", start, end, length)
    for start in ("2026-02-06", "2026-03-20", "2026-03-21"):
        for end in ("2027-06-19", "2027-06-20", "2027-06-21", "2028-08-19", "2028-08-20", "2028-08-21"):
            for length, unit in ((6, "Months"), (12, "Months"), (18, "Months"), (24, "Months"),
                                 (1, "Years"), (2, "Years")):
                add("OLD-TENOR", "multi-year-stride", start, end, length, unit)
    for start in ("2026-02-07", "2026-03-21"):
        for convention in CONVENTIONS:
            for termination in ("Unadjusted", "Preceding"):
                for length in (1, 3):
                    add("OLD-TENOR", "holiday-start", start, "2026-11-21", length, calendar="Weekends",
                        convention=convention, termination=termination)
    for end in ("2026-11-19", "2026-11-20", "2026-11-21"):
        for convention in ("Unadjusted", "Following", "Preceding"):
            for termination in CONVENTIONS:
                add("OLD-TENOR", "adjusted-stop", "2026-02-06", end, 1, calendar="Weekends",
                    convention=convention, termination=termination)
    assert serial["OLD-TENOR"] == TEMPORARY_ROWS

    # OLD-EXTRA: leap-year cutoff, 2028-02-18/19/20/21 are 31/30/29/28 days and
    # 2028-03-19/20 are 1/0 days before 2028-03-20.
    for start in ("2028-02-18", "2028-02-19", "2028-02-20", "2028-02-21", "2028-03-19", "2028-03-20"):
        for end in ("2028-03-20", "2028-04-30", "2028-12-31"):
            if end > start:
                for length in (1, 3, 4, 12):
                    add("OLD-EXTRA", "leap-year-cutoff", start, end, length)
    for start in ("2026-11-20", "2026-11-21", "2026-12-19", "2026-12-20", "2026-12-21", "2026-12-31"):
        for end in ("2027-01-19", "2027-03-20", "2027-03-21", "2027-12-31"):
            for length in (1, 3, 12):
                add("OLD-EXTRA", "year-end-crossing", start, end, length)
    for start, end in (("2026-04-01", "2026-04-02"), ("2026-04-01", "2026-06-19"),
                       ("2026-05-21", "2026-06-19"), ("2026-05-22", "2026-06-19"),
                       ("2026-06-19", "2026-06-20"), ("2026-06-20", "2026-06-21"),
                       ("2026-06-21", "2026-06-22"), ("2026-07-20", "2026-08-20"),
                       ("2026-08-20", "2026-08-21"), ("2026-08-21", "2026-09-20")):
        for length in (1, 2, 3, 4, 6, 12):
            add("OLD-EXTRA", "very-short-span", start, end, length)
    for quarter in ("2026-03-20", "2026-06-20", "2026-09-20", "2026-12-20"):
        for distance in (0, 1, 29, 30, 31):
            start = str(dt.date.fromisoformat(quarter) - dt.timedelta(days=distance))
            for length in (1, 2, 4, 6, 12):
                add("OLD-EXTRA", "quarter-distance-strides", start, "2027-06-30", length)
    # Holiday terminals: 2026-06-21 is the Sunday after Saturday 2026-06-20.
    for convention in CONVENTIONS:
        for termination in CONVENTIONS:
            add("OLD-EXTRA", "holiday-terminal-49-pairs", "2026-02-06", "2026-06-21", 1,
                calendar="Weekends", convention=convention, termination=termination)
    for convention in ("Unadjusted", "Following", "Preceding"):
        for termination in CONVENTIONS:
            add("OLD-EXTRA", "holiday-terminal-quarterly", "2026-02-06", "2026-06-21", 3,
                calendar="Weekends", convention=convention, termination=termination)
    for end in ("2026-06-20", "2026-09-19", "2026-11-22", "2026-12-19"):
        for convention in ("Unadjusted", "Following", "Preceding"):
            for termination in CONVENTIONS:
                add("OLD-EXTRA", "holiday-terminal-ends", "2026-02-06", end, 1,
                    calendar="Weekends", convention=convention, termination=termination)
    for start in ("2026-02-08", "2026-03-22"):
        for convention in CONVENTIONS:
            for termination in ("Unadjusted", "Preceding"):
                for length in (1, 3):
                    add("OLD-EXTRA", "holiday-start-sunday", start, "2026-11-21", length,
                        calendar="Weekends", convention=convention, termination=termination)
    for start in ("2026-06-20", "2026-09-20"):
        for convention in CONVENTIONS:
            for length in (1, 3):
                add("OLD-EXTRA", "holiday-start-on-quarter", start, "2027-03-31", length,
                    calendar="Weekends", convention=convention)
    # Requested ends on hand-listed non-quarter 2M/4M stride points, and the next day.
    for start, length, ends in (
            ("2026-02-06", 2, ("2026-05-20", "2026-07-20", "2026-11-20", "2027-01-20", "2027-05-20")),
            ("2026-02-06", 4, ("2026-07-20", "2026-11-20")),
            ("2026-03-20", 2, ("2026-08-20", "2026-10-20", "2027-02-20", "2027-04-20")),
            ("2026-03-20", 4, ("2026-10-20", "2027-02-20")),
            ("2026-02-19", 2, ("2026-08-20", "2026-10-20", "2027-02-20", "2027-04-20")),
            ("2026-02-19", 4, ("2026-10-20", "2027-02-20"))):
        for end in ends:
            for shift in (0, 1):
                add("OLD-EXTRA", "offquarter-stride-end", start,
                    str(dt.date.fromisoformat(end) + dt.timedelta(days=shift)), length)
    # Requested ends on the stride lattice, but before the recurrence anchor.
    for start, end, length in (("2026-02-19", "2026-04-20", 2), ("2026-03-20", "2026-04-20", 2),
                               ("2026-03-20", "2026-05-20", 1), ("2026-01-05", "2026-01-20", 2),
                               ("2026-02-19", "2026-05-20", 1), ("2026-02-19", "2026-02-20", 1)):
        add("OLD-EXTRA", "lattice-end-before-anchor", start, end, length)
    for start, end in (("2026-02-06", "2026-03-31"), ("2026-03-20", "2026-04-30")):
        for length, unit in DOMAIN_CONTROLS:
            add("OLD-EXTRA", "tenor-domain", start, end, length, unit)
    for start in ("2026-02-06", "2026-03-20"):
        for end in ("2027-06-19", "2028-11-21"):
            for length, unit in ((5, "Months"), (7, "Months"), (9, "Months"), (36, "Months"),
                                 (3, "Years"), (120, "Months"), (10, "Years")):
                add("OLD-EXTRA", "odd-and-huge-stride", start, end, length, unit)
    for start, end in (("2026-06-19", "2026-06-20"), ("2026-09-19", "2026-09-20")):
        for convention in ("Unadjusted", "Following", "Preceding"):
            for termination in ("Unadjusted", "Following", "Preceding", "Nearest"):
                add("OLD-EXTRA", "adjusted-collision", start, end, 3, calendar="Weekends",
                    convention=convention, termination=termination)
    assert len(result) == 1199 and len({case["id"] for case in result}) == len(result)
    return result


def validate(case):
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))
    if not INPUT_ENVELOPE[0] <= start < end <= INPUT_ENVELOPE[1] or (end-start).days > MAX_SPAN_DAYS:
        raise ValueError("positive bounded input span inside 2025..2029 required")
    if case["group"] == "tenor-domain":
        if (case["length"], case["unit"]) not in DOMAIN_CONTROLS:
            raise ValueError("unexpected tenor-domain control")
    elif not (case["unit"] == "Months" and 1 <= case["length"] <= 120
              or case["unit"] == "Years" and 1 <= case["length"] <= 10):
        raise ValueError("positive bounded Months/Years tenor required outside tenor-domain")
    if case["calendar"] not in ("Null", "Weekends"):
        raise ValueError("unsupported calendar")
    if case["convention"] not in CONVENTIONS or case["termination"] not in CONVENTIONS:
        raise ValueError("unsupported convention")


def worker(case, wheel):
    validate(case)
    manifest, files = REFERENCE.attest(wheel)
    import QuantLib as ql
    if ql.__version__ != "1.43":
        raise ValueError("runtime differs from attested wheel")

    def date(text):
        value = dt.date.fromisoformat(text)
        return ql.Date(value.day, value.month, value.year)

    def iso(value):
        return "%04d-%02d-%02d" % (value.year(), value.month(), value.dayOfMonth())

    calendar = {"Null": ql.NullCalendar, "Weekends": ql.WeekendsOnly}[case["calendar"]]()
    common = dict(runtime_version=ql.__version__, module_manifest=manifest, module_files=files)
    try:
        schedule = ql.Schedule(date(case["start"]), date(case["end"]),
                               ql.Period(case["length"], getattr(ql, case["unit"])), calendar,
                               getattr(ql, case["convention"]), getattr(ql, case["termination"]),
                               ql.DateGeneration.OldCDS, False)
    except RuntimeError as error:
        return dict(common, status="constructor-error", error_type=type(error).__name__,
                    error_message=str(error))
    dates = [iso(value) for value in schedule.dates()]
    if len(dates) > MAX_DATES or any(not OUTPUT_ENVELOPE[0] <= value <= OUTPUT_ENVELOPE[1]
                                     for value in dates):
        raise ValueError("public schedule leaves bounded output envelope")
    has_regularity = bool(schedule.hasIsRegular())
    observation = dict(common, status="value", dates=dates, boundary_count=len(dates),
                       strictly_increasing=all(a < b for a, b in zip(dates, dates[1:])),
                       has_regularity=has_regularity,
                       regularity=[bool(schedule.isRegular(i)) for i in range(1, len(dates))]
                       if has_regularity else None, accessor_errors={})
    for name, accessor in (("start_date", schedule.startDate), ("end_date", schedule.endDate)):
        try:
            observation[name] = iso(accessor())
        except RuntimeError as error:
            observation["accessor_errors"][name] = dict(error_type=type(error).__name__,
                                                        error_message=str(error))
    conventions = {getattr(ql, name): name for name in CONVENTIONS}
    observation.update(tenor_length=schedule.tenor().length(),
                       tenor_unit={getattr(ql, name): name for name in UNITS}[schedule.tenor().units()],
                       returned_rule={getattr(ql.DateGeneration, name): name
                                      for name in REFERENCE.RULES}[schedule.rule()],
                       returned_rule_value=int(schedule.rule()), eom=bool(schedule.endOfMonth()),
                       calendar_name=schedule.calendar().name(),
                       convention=conventions[schedule.businessDayConvention()],
                       termination=conventions[schedule.terminationDateBusinessDayConvention()])
    return observation


# ---- Independent local-contract model (docs/M7_OLD_CDS_CONTRACT.md) ----

def month_coordinate(value):
    return (value.year-1)*12 + value.month-1


def twentieth(coordinate):
    year, month = divmod(coordinate, 12)
    return dt.date(year+1, month+1, 20)


def quarter_on_or_after(value):
    """Q(x): scan forward day by day; consecutive quarter twentieths are <= 92 days apart."""
    probe = value
    for _ in range(93):
        if probe.day == 20 and probe.month in QUARTER_MONTHS:
            return probe
        probe += ONE_DAY
    raise AssertionError("no quarterly twentieth within 92 days")


def easter_sunday(year):
    """Anonymous Gregorian computus (Meeus/Jones/Butcher)."""
    golden = year % 19
    century, rest = divmod(year, 100)
    leap_century, century_rest = divmod(century, 4)
    correction = (century - (century+8)//25 + 1)//3
    epact = (19*golden + century - leap_century - correction + 15) % 30
    leap_rest, rest_rest = divmod(rest, 4)
    weekday = (32 + 2*century_rest + 2*leap_rest - epact - rest_rest) % 7
    shift = (golden + 11*epact + 22*weekday)//451
    month, day = divmod(epact + weekday - 7*shift + 114, 31)
    return dt.date(year, month, day+1)


def business(value, calendar):
    if calendar == "Null":
        return True
    if value.weekday() >= 5:
        return False
    if calendar == "Weekends":
        return True
    # TARGET only for the frozen 2026 control: verified local coverage 2026..2028.
    if calendar != "TARGET" or not 2026 <= value.year <= 2028:
        raise AssertionError("model calendar outside its declared coverage")
    easter = easter_sunday(value.year)
    closed = (dt.date(value.year, 1, 1), easter - 2*ONE_DAY, easter + ONE_DAY,
              dt.date(value.year, 5, 1), dt.date(value.year, 12, 25), dt.date(value.year, 12, 26))
    return value not in closed


def nearest_business(value, calendar, step):
    for _ in range(367):
        if business(value, calendar):
            return value
        value += step
    raise AssertionError("bounded business-day search exhausted")


def adjust(value, convention, calendar):
    """FS-BDC conventions of docs/REQUIREMENTS.md section 8."""
    if convention == "Unadjusted" or business(value, calendar):
        return value
    after = nearest_business(value, calendar, ONE_DAY)
    before = nearest_business(value, calendar, -ONE_DAY)
    same_month = lambda other: (other.year, other.month) == (value.year, value.month)
    if convention == "Following":
        return after
    if convention == "Preceding":
        return before
    if convention == "ModifiedFollowing":
        return after if same_month(after) else before
    if convention == "ModifiedPreceding":
        return before if same_month(before) else after
    if convention == "HalfMonthModifiedFollowing":
        return before if not same_month(after) or value.day <= 15 < after.day else after
    if convention == "Nearest":
        return after if after-value <= value-before else before
    raise AssertionError("unknown convention " + convention)


def local_contract(case):
    """The adopted local OldCDS definition, before any comparison."""
    if case["unit"] not in ("Months", "Years") or case["length"] <= 0:
        return dict(status="InvalidOldCdsTenor", stable_kind="schedule.invalid_old_cds_tenor")
    stride = case["length"] * (12 if case["unit"] == "Years" else 1)
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))
    initial, resolved = quarter_on_or_after(start), quarter_on_or_after(end)
    distance = (initial-start).days
    anchor = twentieth(month_coordinate(initial)+3) if initial < resolved and distance < 30 else initial
    base = month_coordinate(anchor)
    interior, month = [], base
    while twentieth(month) < end:  # every twentieth from B before E; keep congruent months
        if (month-base) % stride == 0:
            interior.append(twentieth(month))
        month += 1
    raw = [start, *interior, resolved]
    limit = month_coordinate(end) - (1 if end.day <= 20 else 0)
    closed_count = (0 if base > limit else (limit-base)//stride + 1) + 2
    on_lattice = lambda value: value.day == 20 and (month_coordinate(value)-base) % stride == 0
    flags = [on_lattice(a) and on_lattice(b) and month_coordinate(b)-month_coordinate(a) == stride
             for a, b in zip(raw, raw[1:])]
    if not (closed_count == len(raw) and start < anchor <= resolved and (anchor < end or anchor == resolved)
            and all(a < b for a, b in zip(raw, raw[1:]))):
        raise AssertionError("local model violates a stated contract consequence: " + case["id"])
    adjusted = [adjust(value, case["convention"], case["calendar"]) for value in raw[:-1]]
    adjusted.append(adjust(resolved, case["termination"], case["calendar"]))
    violation = next((i for i in range(len(adjusted)-1) if adjusted[i] >= adjusted[i+1]), None)
    model = dict(status="value" if violation is None else "AdjustedOrder", stride_months=stride,
                 distance_days=distance, initial_quarter=initial, recurrence_anchor=anchor,
                 resolved_end=resolved,
                 omitted_quarter=initial if anchor != initial and initial != start else None,
                 raw=raw, adjusted=adjusted, regularity=flags, boundary_count=len(raw))
    if violation is not None:
        model["first_order_violation_index"] = violation
    return model


def position(index, intervals):
    if intervals == 1:
        return "sole"
    return "first" if index == 0 else "last" if index == intervals-1 else "interior"


def classify(case, observation, model):
    """Classify one reference row under the local contract; returns (class, comparison)."""
    if case.get("eom") or case.get("first") or case.get("penultimate"):
        return "outside-options", None
    if model["status"] == "InvalidOldCdsTenor":
        return "domain-rejected", None
    if model["status"] == "AdjustedOrder":
        start = dt.date.fromisoformat(case["start"])
        return "local-AdjustedOrder", dict(
            s1_start_moved_by_ordinary_convention=adjust(start, case["convention"], case["calendar"]) != start,
            reference_first_date_is_unadjusted_start=observation["dates"][0] == case["start"]
            if observation["status"] == "value" else None)
    if observation["status"] != "value":
        return "UNCLASSIFIED", dict(unclassified_reason="reference constructor error for a local value")
    if (observation["returned_rule"], observation["tenor_length"], observation["tenor_unit"],
            observation["eom"], observation["has_regularity"], observation["strictly_increasing"]) != (
            "OldCDS", case["length"], case["unit"], False, True, True):
        return "UNCLASSIFIED", dict(unclassified_reason="reference metadata or order differs")
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))
    convention, termination, calendar = case["convention"], case["termination"], case["calendar"]
    local = [value.isoformat() for value in model["adjusted"]]
    flags, reference, reference_flags = model["regularity"], observation["dates"], observation["regularity"]
    base, stride = month_coordinate(model["recurrence_anchor"]), model["stride_months"]
    adjusted_end = adjust(end, termination, calendar)
    raw = model["raw"]
    last_point = raw[-2] if len(raw) >= 3 else None
    # T2 compares adjust(E) with the last raw stride point P, both under the
    # termination convention. P under the ordinary convention is recorded only
    # as a diagnostic of the other possible reading of the contract's wording.
    comparison = dict(
        dates_equal=reference == local, regularity_equal=reference_flags == flags,
        s1_start_moved_by_ordinary_convention=adjust(start, convention, calendar) != start,
        t1_end_is_offquarter_stride_point=(end.day == 20 and end.month not in QUARTER_MONTHS
                                           and month_coordinate(end) >= base
                                           and (month_coordinate(end)-base) % stride == 0),
        t2_last_stride_point_coincidence=dict(
            termination=last_point is not None and adjust(last_point, termination, calendar) == adjusted_end,
            ordinary_diagnostic=last_point is not None
            and adjust(last_point, convention, calendar) == adjusted_end))
    if reference == local:
        differences = [dict(index=i, position=position(i, len(flags)),
                            reference="Regular" if reference_flags[i] else "Stub",
                            local="Regular" if flags[i] else "Stub")
                       for i in range(len(flags)) if reference_flags[i] != flags[i]]
        comparison["flag_differences"] = differences
        return ("flags-only" if differences else "exact"), comparison
    s1 = comparison["s1_start_moved_by_ordinary_convention"]
    t1 = comparison["t1_end_is_offquarter_stride_point"]
    t2 = comparison["t2_last_stride_point_coincidence"]["termination"]
    first, final = [start.isoformat()], [adjusted_end.isoformat()]
    projections = {}
    if t1:
        projections["T1-offquarter-exact-end"] = (local[:-1] + final, flags)
    if t2:
        projections["T2-adjusted-end-coincidence"] = (local[:-2] + final, flags[:-1])
    if s1:
        projections["S1-unadjusted-first"] = (first + local[1:], flags)
    if s1 and t2:
        projections["S1+T2"] = (first + local[1:-2] + final, flags[:-1])
    unlisted = {"S1+T1": first + local[1:-1] + final} if s1 and t1 else {}
    matched = [name for name, (dates, _) in projections.items() if dates == reference]
    comparison["matched_distinctions"] = matched
    if len(matched) == 1:
        dates, projected_flags = projections[matched[0]]
        comparison["projection"] = dates
        comparison["projected_flag_difference_indexes"] = [
            i for i, (a, b) in enumerate(zip(reference_flags, projected_flags)) if a != b]
        return matched[0], comparison
    unlisted_matches = [name for name, dates in unlisted.items() if dates == reference]
    comparison["unclassified_reason"] = (
        "ambiguous contract distinctions" if matched else
        "reference equals unlisted composition " + unlisted_matches[0] if unlisted_matches else
        "no adopted contract distinction reproduces the reference vector")
    return "UNCLASSIFIED", comparison


def annotate(case, observation):
    model = local_contract(case)
    classification, comparison = classify(case, observation, model)
    local = {key: (value.isoformat() if isinstance(value, dt.date) else
                   [item.isoformat() if isinstance(item, dt.date) else item for item in value]
                   if isinstance(value, list) else value)
             for key, value in model.items()}
    annotation = dict(classification=classification, local=local, comparison=comparison)
    return classification, local, annotation


def self_check():
    """Independent sanity anchors for the model; no reference output is used."""
    for year, easter in ((2026, dt.date(2026, 4, 5)), (2027, dt.date(2027, 3, 28)),
                         (2028, dt.date(2028, 4, 16))):  # M3_TARGET_CONTRACT dated anchors
        if easter_sunday(year) != easter:
            raise AssertionError("computus disagrees with dated TARGET anchors")
    examples = {dt.date(2024, 8, 31): ("2024-09-02", "2024-08-30", "2024-08-30", "2024-08-30",
                                       "2024-08-30", "2024-08-30"),
                dt.date(2024, 6, 15): ("2024-06-17", "2024-06-17", "2024-06-14", "2024-06-14",
                                       "2024-06-14", "2024-06-14")}
    for day, expected in examples.items():  # REQUIREMENTS FS-BDC weekends-only table
        if tuple(adjust(day, name, "Weekends").isoformat() for name in CONVENTIONS[1:]) != expected:
            raise AssertionError("business-day model disagrees with FS-BDC examples")
    maximum = local_contract(dict(id="maximum", start="0001-01-01", end="9999-12-20", length=1,
                                  unit="Months", calendar="Null", convention="Unadjusted",
                                  termination="Unadjusted"))
    if maximum["boundary_count"] != 119_987:  # contract's attainable civil maximum
        raise AssertionError("model disagrees with the contract's 119,987-boundary maximum")


def frozen_classification():
    """Read-only classification of the 98 frozen OldCDS rows in roll-probes.fixture."""
    body = FROZEN.read_text().split("\n\n", 1)[1].splitlines()
    columns = body[0].split("\t")
    groups = {name: [] for name in CLASSES}
    for line in body[1:]:
        row = dict(zip(columns, line.split("\t")))
        if row["rule"] != "OldCDS":
            continue
        case = dict(id=row["id"], start=row["start"], end=row["end"], length=int(row["tenor_length"]),
                    unit=row["tenor_unit"], calendar=row["calendar"], convention=row["convention"],
                    termination=row["termination"], eom=row["end_of_month"] == "true",
                    first=row["first_coupon_date"], penultimate=row["penultimate_coupon_date"])
        classification, _, _ = annotate(case, json.loads(row["public_observation"]))
        groups[classification].append(row["id"])
    if sum(map(len, groups.values())) != 98:
        raise ValueError("frozen roll fixture no longer has 98 OldCDS rows")
    return groups


def collect(wheel, manifest, files):
    records = []
    environment = dict(os.environ)
    environment.pop("PYTHONPATH", None)
    environment.pop("PYTHONHOME", None)
    for case in scenarios():
        validate(case)
        try:
            process = subprocess.run([sys.executable, "-I", str(Path(__file__).resolve()),
                                      "--worker", case["id"], "--quantlib-wheel", str(wheel.resolve())],
                                     capture_output=True, text=True, timeout=TIMEOUT,
                                     env=environment, check=False)
        except subprocess.TimeoutExpired as error:
            raise SystemExit("bounded worker timed out; no fixture written: " + case["id"]) from error
        if process.returncode or process.stderr:
            raise SystemExit("probe worker failed; no fixture written: " + case["id"] + ": " + process.stderr)
        observation = json.loads(process.stdout)
        if (observation["runtime_version"], observation["module_manifest"], observation["module_files"]) != (
                "1.43", manifest, files):
            raise ValueError("worker runtime/module attestation changed: " + case["id"])
        records.append((case, observation) + annotate(case, observation))
        if len(records) % 100 == 0:
            print("observed", len(records), "bounded OldCDS cases", flush=True)
    return records


def compare_temporary(path, records):
    """Field-by-field identity of OLD-TENOR rows with the temporary probe JSON."""
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != TEMPORARY_SHA256:
        raise SystemExit("temporary probe JSON digest differs: " + str(path))
    rows = json.loads(data)["rows"]
    if len(rows) != TEMPORARY_ROWS:
        raise SystemExit("temporary probe JSON does not hold 551 rows")
    for row, (case, observation, _, _, _) in zip(rows, records):
        old, mine = row["case"], row["observation"]
        same = old == {key: case[key] for key in old} and set(old) == set(case) and (
            mine["status"], mine["runtime"], mine["manifest"], mine["module_files"]) == (
            observation["status"], observation["runtime_version"], observation["module_manifest"],
            observation["module_files"])
        if same and mine["status"] == "value":
            same = (mine["dates"], mine["regularity"], mine["strictly_increasing"], mine["returned_rule"],
                    mine["tenor_length"], mine["tenor_unit"], mine["eom"]) == (
                observation["dates"], observation["regularity"], observation["strictly_increasing"],
                observation["returned_rule_value"], observation["tenor_length"],
                observation["tenor_unit"], observation["eom"])
        elif same:
            same = mine.get("message") == observation.get("error_message")
        if not same:
            raise SystemExit("temporary observation differs: " + case["id"])
    print("temporary-json", "rows=" + str(len(rows)), "identical", TEMPORARY_SHA256)


def fnv64(data):
    """FNV-1a 64-bit over the body bytes, as in cds_maturity_oracles.py."""
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--compare-temporary-json", type=Path,
                        help="also require OLD-TENOR rows to equal the temporary 551-row probe JSON")
    parser.add_argument("--worker", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        selected = next((case for case in scenarios() if case["id"] == args.worker), None)
        if selected is None:
            parser.error("unknown bounded worker case")
        print(REFERENCE.compact(worker(selected, args.quantlib_wheel)))
        return
    self_check()
    manifest, files = REFERENCE.attest(args.quantlib_wheel)
    records = collect(args.quantlib_wheel, manifest, files)
    if args.compare_temporary_json:
        compare_temporary(args.compare_temporary_json, records)
    frozen = frozen_classification()
    join = lambda values: ";".join(values)
    counts = lambda values: ";".join(key + "=" + str(value) for key, value in sorted(Counter(values).items()))
    flag_text = lambda flags: join("Regular" if flag else "Stub" for flag in flags)
    columns = ("id", "group", "start", "end", "tenor_length", "tenor_unit", "rule", "calendar",
               "convention", "termination", "end_of_month", "status", "reference_dates",
               "reference_regularity", "classification", "flag_differences", "local_status",
               "local_raw_dates", "local_dates", "local_regularity", "initial_quarter",
               "recurrence_anchor", "resolved_end", "omitted_quarter", "boundary_count",
               "public_observation", "adopted_local_contract_annotation")
    lines = ["\t".join(columns)]
    for case, observation, classification, local, annotation in records:
        value = observation["status"] == "value"
        defined = local["status"] != "InvalidOldCdsTenor"
        differences = (annotation["comparison"] or {}).get("flag_differences", [])
        values = (case["id"], case["group"], case["start"], case["end"], str(case["length"]), case["unit"],
                  "OldCDS", case["calendar"], case["convention"], case["termination"], "false",
                  observation["status"], join(observation["dates"]) if value else "",
                  flag_text(observation["regularity"]) if value and observation["has_regularity"] else "",
                  classification,
                  join(str(item["index"]) + ":" + item["position"] for item in differences),
                  local["status"], join(local["raw"]) if defined else "",
                  join(local["adjusted"]) if local["status"] == "value" else "",
                  flag_text(local["regularity"]) if local["status"] == "value" else "",
                  local.get("initial_quarter", ""), local.get("recurrence_anchor", ""),
                  local.get("resolved_end", ""), local.get("omitted_quarter") or "",
                  str(local["boundary_count"]) if defined else "",
                  REFERENCE.compact(observation), REFERENCE.compact(annotation))
        lines.append("\t".join(values))
    body = ("\n".join(lines) + "\n").encode()
    tenor = [classification for case, _, classification, _, _ in records if case["id"].startswith("OLD-TENOR")]
    extra = [classification for case, _, classification, _, _ in records if case["id"].startswith("OLD-EXTRA")]
    metadata = {
        "schema": "1", "fixture-set": "M7-OLD-CDS-REFERENCE-PROBES",
        "domain": "schedule.reference-probes",
        "description": str(len(records)) + " bounded attested OldCDS Schedule observations (" + str(len(tenor))
                       + " reproduced OLD-TENOR, " + str(len(extra)) + " OLD-EXTRA); values/errors retained; independent local-contract classification, not financial oracles",
        "reference": "QuantLib", "reference-version": "1.43 (attested public worker runtime)",
        "reference-source": "Public Schedule constructor/accessors with DateGeneration.OldCDS, EOM false and no explicit roles; no C++ implementation read or translation",
        "tolerance": "Exact recorded runtime text; TOL-E0 dates and flags in the separate local-contract classification; no tolerance and no SabiQuant parity claim",
        "reproduction": "generator-sha256=" + REVISION + "; attestation-helper-sha256=" + HELPER_REVISION + "; python3 validation/generator/schedule_old_cds_probes.py --quantlib-wheel " + args.quantlib_wheel.name,
        "contract-source-sha256": CONTRACT_REVISION,
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
        "quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": REFERENCE.WHEEL_SHA256,
        "quantlib-module-files": str(files), "quantlib-module-manifest-sha256": manifest,
        "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995 from docs/QUANTLIB_REFERENCE.md; wheel source commit not independently attested",
        "execution-bound": "One isolated -I subprocess per case, 10s timeout; positive <=1096-day inputs inside 2025..2029; 1..120M/1..10Y tenors plus seven named domain controls; 0..100 output dates in 2025..2030; timeout/worker failure aborts before writing",
        "scope": "Engineering classification against the adopted local OldCDS contract; observations are software behavior, not financial truth; no historical CDS, legal-date, owner or full-M7 acceptance",
        "classification-source": "Independent Python model in this generator; class predicates in docs/M7_OLD_CDS_REFERENCE_PROBES.md; T2 adjusts E and the last raw stride point under the termination convention; any other difference is UNCLASSIFIED",
        "record-count": str(len(records)),
        "group-counts": counts(case["group"] for case, *_ in records),
        "status-counts": counts(observation["status"] for _, observation, *_ in records),
        "old-tenor-classification-counts": counts(tenor),
        "old-extra-classification-counts": counts(extra),
        "temporary-observations": "OLD-TENOR-0000..0550 reproduce the uncommitted 551-row temporary probe JSON sha256=" + TEMPORARY_SHA256 + " field for field (checked with --compare-temporary-json)",
        "frozen-roll-probes-sha256": hashlib.sha256(FROZEN.read_bytes()).hexdigest(),
        "frozen-oldcds-classification-counts": ";".join(name + "=" + str(len(ids)) for name, ids in frozen.items() if ids),
        "frozen-oldcds-flags-only-ids": join(frozen["flags-only"]) or "none",
        "frozen-oldcds-domain-rejected-ids": join(frozen["domain-rejected"]) or "none",
        "frozen-oldcds-outside-options-ids": join(frozen["outside-options"]) or "none",
        "body-sha256": hashlib.sha256(body).hexdigest(),
        "body-fnv64": fnv64(body),
    }
    if args.check:
        old_header, _ = DEST.read_text().split("\n\n", 1)
        old = dict(line.split(": ", 1) for line in old_header.splitlines())
        for key in ("generated-at", "platform", "python-version"):
            metadata[key] = old[key]
    output = ("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n").encode() + body
    if args.check:
        if DEST.read_bytes() != output:
            raise SystemExit("reference probe fixture differs: " + str(DEST))
        label = "verified"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(output)
        label = "generated"
    print(label, DEST.relative_to(ROOT), "rows=" + str(len(records)))
    print("statuses", metadata["status-counts"])
    print("old-tenor", metadata["old-tenor-classification-counts"])
    print("old-extra", metadata["old-extra-classification-counts"])
    print("frozen-oldcds", metadata["frozen-oldcds-classification-counts"])
    for case, _, classification, _, annotation in records:
        if classification == "UNCLASSIFIED":
            print("UNCLASSIFIED", case["id"], annotation["comparison"]["unclassified_reason"])
    print("generator-sha256", REVISION)
    print("fixture-sha256", hashlib.sha256(output).hexdigest())


if __name__ == "__main__":
    main()
