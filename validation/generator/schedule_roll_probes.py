#!/usr/bin/env python3
"""Bounded attested QuantLib special-roll observations, not financial oracles.

Only the public constructors/accessors and cdsMaturity function are called.
The separate civil comparison implements the adopted local four-rule
civil contract, not QuantLib's generation implementation or Rust output.
Raw reference values, flags, collapsed/unordered outputs and errors survive.
"""

import argparse
import calendar as gregorian
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
DEST = ROOT / "validation/fixtures/schedule/roll-probes.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
HELPER = Path(__file__).with_name("schedule_probes.py")
HELPER_REVISION = hashlib.sha256(HELPER.read_bytes()).hexdigest()
RESEARCH = ROOT / "docs/M7_SPECIAL_RULES_RESEARCH.md"
RESEARCH_REVISION = hashlib.sha256(RESEARCH.read_bytes()).hexdigest()
CONTRACT = ROOT / "docs/M7_ROLL_CONTRACT.md"
CONTRACT_REVISION = hashlib.sha256(CONTRACT.read_bytes()).hexdigest()
SPEC = importlib.util.spec_from_file_location("frozen_roll_attestation", HELPER)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load frozen schedule attestation helper")
REFERENCE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REFERENCE)
RULES = ("ThirdWednesday", "ThirdWednesdayInclusive", "Twentieth",
         "TwentiethIMM", "OldCDS", "CDS", "CDS2015")
CONVENTIONS = ("Unadjusted", "Following", "ModifiedFollowing", "Preceding",
               "ModifiedPreceding", "HalfMonthModifiedFollowing", "Nearest")
TIMEOUT = 10
MAX_DATES = 400


def scenarios():
    def case(name, group, rule, start="2026-02-06", end="2026-11-17",
             length=3, unit="Months", calendar="Null", convention="Unadjusted",
             termination="Unadjusted", eom=False, first="", penultimate="",
             maturity_controls=False):
        return dict(id=name, group=group, start=start, end=end, length=length,
                    unit=unit, rule=rule, calendar=calendar, convention=convention,
                    termination=termination, eom=eom, first=first,
                    penultimate=penultimate, maturity_controls=maturity_controls)

    result = [case("ROLL-BASE-" + rule.upper(), "original-common-control", rule)
              for rule in RULES]
    for rule in RULES:
        starts = ("2026-03-17", "2026-03-18", "2026-03-19") if rule.startswith("Third") else (
            "2026-03-19", "2026-03-20", "2026-03-21")
        ends = ("2026-09-15", "2026-09-16", "2026-09-17") if rule.startswith("Third") else (
            "2026-11-19", "2026-11-20", "2026-11-21")
        for length in (3, 6, 12):
            for start in starts:
                for end in ends:
                    result.append(case("ROLL-EDGE-" + rule.upper() + "-" + start + "-" + end + "-" + str(length),
                                       "before-on-after-roll", rule, start, end, length))
        for start, end in (("2026-03-01", "2026-03-30"),
                           ("2026-03-17", "2026-03-19"),
                           ("2026-03-19", "2026-03-20"),
                           ("2026-03-20", "2026-03-21"),
                           ("2026-03-21", "2026-03-31"),
                           ("2026-12-30", "2026-12-31")):
            result.append(case("ROLL-SHORT-" + rule.upper() + "-" + start + "-" + end,
                               "same-month-and-year-end", rule, start, end))
        for length, unit in ((1, "Days"), (1, "Weeks"), (1, "Years"),
                             (1, "Months"), (2, "Months"), (4, "Months"),
                             (0, "Months"), (-1, "Months")):
            result.append(case("ROLL-UNIT-" + rule.upper() + "-" + str(length) + unit,
                               "tenor-domains", rule, length=length, unit=unit))
        for calendar in ("Null", "Weekends"):
            for convention in ("Unadjusted", "Following"):
                result.append(case("ROLL-EOM-" + rule.upper() + "-" + calendar + "-" + convention,
                                   "eom-option", rule, eom=True, calendar=calendar,
                                   convention=convention, termination=convention))
        for first, penultimate, label in (
                ("2026-03-18", "", "FIRST-WED"),
                ("2026-03-19", "", "FIRST-OFF-WED"),
                ("", "2026-09-16", "PENULT-WED"),
                ("2026-03-18", "2026-09-16", "BOTH-WED"),
                ("2026-09-16", "2026-03-18", "REVERSED-WED")):
            result.append(case("ROLL-COUPON-" + rule.upper() + "-" + label,
                               "explicit-role-options", rule, first=first,
                               penultimate=penultimate))
        for convention in CONVENTIONS:
            for termination in ("Unadjusted", "Preceding"):
                result.append(case("ROLL-ADJUST-" + rule.upper() + "-" + convention + "-" + termination,
                                   "ordinary-terminal-options", rule,
                                   start="2026-02-06", end="2026-11-21",
                                   calendar="Weekends", convention=convention,
                                   termination=termination))
    for roll in (dt.date(2026, 3, 20), dt.date(2026, 6, 20),
                 dt.date(2026, 9, 20), dt.date(2026, 12, 20)):
        for distance in (0, 1, 29, 30, 31):
            for rule in ("OldCDS", "TwentiethIMM"):
                start = str(roll - dt.timedelta(days=distance))
                result.append(case("ROLL-CUTOFF-" + rule.upper() + "-" + str(roll) + "-" + str(distance),
                                   "legacy-first-roll-distance", rule, start=start,
                                   end="2026-12-31"))
    for month in (3, 6, 9, 12):
        for day in (19, 20, 21):
            for rule in ("OldCDS", "CDS", "CDS2015"):
                result.append(case("ROLL-MATURITY-" + rule.upper() + "-2026-" + f"{month:02d}-{day:02d}",
                                   "fixed-end-versus-quoted-maturity", rule,
                                   start=f"2026-{month:02d}-{day:02d}", end="2026-12-31",
                                   maturity_controls=True))
    for rule in RULES:
        result.append(case("ROLL-TARGET-" + rule.upper(), "target-calendar-control", rule,
                           calendar="TARGET", convention="Following", termination="Following"))
    assert len(result) == 538 and len({entry["id"] for entry in result}) == 538
    return result


def validate(case):
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))
    if start.year != 2026 or end.year != 2026 or not 0 < (end-start).days <= 366:
        raise ValueError("positive bounded 2026 input span required")
    if not -1 <= case["length"] <= 12 or case["unit"] not in ("Days", "Weeks", "Months", "Years"):
        raise ValueError("unsupported bounded tenor")
    if case["length"] <= 0 and case["group"] != "tenor-domains":
        raise ValueError("nonpositive tenor only in named rejection/control group")
    if case["rule"] not in RULES or case["calendar"] not in ("Null", "Weekends", "TARGET"):
        raise ValueError("unsupported rule/calendar")
    if case["convention"] not in CONVENTIONS or case["termination"] not in CONVENTIONS:
        raise ValueError("unsupported convention")
    for name in ("first", "penultimate"):
        if case[name] and not start < dt.date.fromisoformat(case[name]) < end:
            raise ValueError("coupon control must remain strictly inside input span")


def worker(case, wheel):
    validate(case)
    manifest, files = REFERENCE.attest(wheel)
    import QuantLib as ql
    if ql.__version__ != "1.43":
        raise ValueError("runtime differs from attested wheel")

    def date(text):
        if not text:
            return ql.Date()
        value = dt.date.fromisoformat(text)
        return ql.Date(value.day, value.month, value.year)

    def iso(value):
        return "%04d-%02d-%02d" % (value.year(), value.month(), value.dayOfMonth())

    calendar = {"Null": ql.NullCalendar, "Weekends": ql.WeekendsOnly,
                "TARGET": ql.TARGET}[case["calendar"]]()
    rule = getattr(ql.DateGeneration, case["rule"])
    common = dict(runtime_version=ql.__version__, module_manifest=manifest, module_files=files)
    maturity = []
    if case["maturity_controls"]:
        for months in (0, 3, 6, 12):
            try:
                value = ql.cdsMaturity(date(case["start"]), ql.Period(months, ql.Months), rule)
            except RuntimeError as error:
                maturity.append(dict(quoted_months=months, status="error",
                                     error_type=type(error).__name__, error_message=str(error)))
            else:
                null = value == ql.Date()
                rendered = None if null else iso(value)
                if rendered is not None and not "2025-01-01" <= rendered <= "2028-12-31":
                    raise ValueError("public maturity leaves bounded envelope")
                maturity.append(dict(quoted_months=months, status="null-date" if null else "value",
                                     date=rendered))
    try:
        schedule = ql.Schedule(date(case["start"]), date(case["end"]),
                               ql.Period(case["length"], getattr(ql, case["unit"])), calendar,
                               getattr(ql, case["convention"]), getattr(ql, case["termination"]),
                               rule, case["eom"], date(case["first"]), date(case["penultimate"]))
    except RuntimeError as error:
        return dict(common, status="constructor-error", error_type=type(error).__name__,
                    error_message=str(error), maturity_controls=maturity)
    dates = [iso(value) for value in schedule.dates()]
    if len(dates) > MAX_DATES or any(not "2025-01-01" <= value <= "2028-12-31" for value in dates):
        raise ValueError("public schedule leaves bounded output envelope")
    observation = dict(common, status="value", dates=dates, boundary_count=len(dates),
                       strictly_increasing=all(a < b for a, b in zip(dates, dates[1:])),
                       has_regularity=bool(schedule.hasIsRegular()),
                       regularity=[bool(schedule.isRegular(i)) for i in range(1, len(dates))]
                       if schedule.hasIsRegular() else None,
                       input_start_business_eom=bool(calendar.isEndOfMonth(date(case["start"]))),
                       input_end_business_eom=bool(calendar.isEndOfMonth(date(case["end"]))),
                       maturity_controls=maturity, accessor_errors={})
    for name, accessor in (("start_date", schedule.startDate), ("end_date", schedule.endDate)):
        try:
            observation[name] = iso(accessor())
        except RuntimeError as error:
            observation["accessor_errors"][name] = dict(error_type=type(error).__name__, error_message=str(error))
    observation.update(tenor_length=schedule.tenor().length(),
                       tenor_unit={getattr(ql, name): name for name in ("Days", "Weeks", "Months", "Years")}[schedule.tenor().units()],
                       returned_rule={getattr(ql.DateGeneration, name): name for name in REFERENCE.RULES}[schedule.rule()],
                       eom=bool(schedule.endOfMonth()), calendar_name=schedule.calendar().name(),
                       convention={getattr(ql, name): name for name in CONVENTIONS}[schedule.businessDayConvention()],
                       termination={getattr(ql, name): name for name in CONVENTIONS}[schedule.terminationDateBusinessDayConvention()])
    return observation


def local_contract(case, observation):
    """Independently select points for the adopted local civil contract."""
    if case["rule"] not in RULES[:4]:
        return dict(status="outside-four-rule-contract")
    if case["eom"] or case["first"] or case["penultimate"] or case["calendar"] != "Null" or (
            case["convention"], case["termination"]) != ("Unadjusted", "Unadjusted"):
        return dict(status="outside-local-options")
    stride = case["length"] * (12 if case["unit"] == "Years" else 1)
    if case["unit"] not in ("Months", "Years") or stride <= 0 or (
            case["rule"] == "TwentiethIMM" and stride % 3):
        return dict(status="local-incompatible-tenor", reference_status=observation["status"])
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))

    def coordinate(value):
        return (value.year-1)*12 + value.month-1

    def point(month):
        year, local = divmod(month, 12)
        if case["rule"].startswith("Third"):
            # Enumerate the seven possible days independently; no nth-weekday
            # implementation or reference output supplies the expected point.
            return next(dt.date(year+1, local+1, day) for day in range(15, 22)
                        if dt.date(year+1, local+1, day).weekday() == 2)
        return dt.date(year+1, local+1, 20)

    quarter = case["rule"] == "TwentiethIMM"
    months = range(coordinate(start), coordinate(end)+13)
    eligible = [month for month in months if not quarter or month % 3 == 2]
    if case["rule"].startswith("Third"):
        anchor = coordinate(start)
        first = point(anchor) if case["rule"] == "ThirdWednesdayInclusive" else start
        last = point(coordinate(end)) if case["rule"] == "ThirdWednesdayInclusive" else end
        if first >= last:
            return dict(status="local-resolved-range-rejection", reference_status=observation["status"])
        middle = [point(month) for month in eligible if month > anchor and (month-anchor) % stride == 0
                  and first < point(month) < last]
        dates = [first, *middle, last]
    else:
        anchor = next(month for month in eligible if point(month) >= start)
        last = next(point(month) for month in eligible if point(month) >= end)
        middle = [point(month) for month in eligible if (month-anchor) % stride == 0
                  and start < point(month) < last]
        dates = [start, *middle, last]
    flags = [point(coordinate(a)) == a and point(coordinate(b)) == b
             and coordinate(b)-coordinate(a) == stride
             and (not quarter or coordinate(a) % 3 == coordinate(b) % 3 == 2)
             for a, b in zip(dates, dates[1:])]
    rendered = list(map(str, dates))
    if observation["status"] != "value":
        comparison = "reference-rejected-local-value"
    elif observation["dates"] != rendered:
        comparison = "date-vector-difference"
    elif observation["regularity"] != flags:
        comparison = "regularity-difference-only"
    else:
        comparison = "dates-and-flags-agree"
    return dict(status="local-value", dates=rendered, regularity=flags,
                comparison=comparison)


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
        records.append((case, observation, local_contract(case, observation)))
        if len(records) % 100 == 0:
            print("observed", len(records), "bounded special-rule cases", flush=True)
    return records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--worker", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        selected = next((case for case in scenarios() if case["id"] == args.worker), None)
        if selected is None:
            parser.error("unknown bounded worker case")
        print(REFERENCE.compact(worker(selected, args.quantlib_wheel)))
        return
    manifest, files = REFERENCE.attest(args.quantlib_wheel)
    records = collect(args.quantlib_wheel, manifest, files)
    metadata = {
        "schema": "1", "fixture-set": "M7-SPECIAL-ROLL-REFERENCE-PROBES",
        "domain": "schedule.reference-probes",
        "description": str(len(records)) + " bounded attested special-roll observations; preserves errors/unordered/collapsed vectors; adopted local civil annotations are not financial oracles",
        "reference": "QuantLib", "reference-version": "1.43 (attested public worker runtime)",
        "reference-source": "Public Schedule constructor/accessors plus cdsMaturity on bounded explicit inputs; no C++ implementation read or translation",
        "tolerance": "Exact recorded runtime text; adopted local four-rule comparison separately annotated; no SabiQuant parity or acceptance claim",
        "reproduction": "generator-sha256=" + REVISION + "; attestation-helper-sha256=" + HELPER_REVISION + "; python3 validation/generator/schedule_roll_probes.py --quantlib-wheel " + args.quantlib_wheel.name,
        "research-source-sha256": RESEARCH_REVISION,
        "contract-source-sha256": CONTRACT_REVISION,
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
        "quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": REFERENCE.WHEEL_SHA256,
        "quantlib-module-files": str(files), "quantlib-module-manifest-sha256": manifest,
        "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995 from docs/QUANTLIB_REFERENCE.md; wheel source commit not independently attested",
        "execution-bound": "One isolated -I subprocess per case, 10s timeout; positive <=366-day 2026 ranges; bounded -1..12 tenors, 2026 roles; 0..400 output dates and maturity controls in 2025..2028; timeout/worker failure aborts before writing",
        "scope": "Public-reference engineering comparisons under the adopted local civil contract; no financial authority, contract mutation, universal reference parity or full M7 acceptance",
    }
    columns = ("id", "group", "start", "end", "tenor_length", "tenor_unit", "rule", "calendar", "convention", "termination", "end_of_month", "first_coupon_date", "penultimate_coupon_date", "status", "public_observation", "adopted_local_contract_annotation")
    rows = ["\t".join(columns)]
    for case, observation, annotation in records:
        values = (case["id"], case["group"], case["start"], case["end"], str(case["length"]), case["unit"], case["rule"], case["calendar"], case["convention"], case["termination"], str(case["eom"]).lower(), case["first"], case["penultimate"], observation["status"], REFERENCE.compact(observation), REFERENCE.compact(annotation))
        rows.append("\t".join(values))
    if args.check:
        old_header, _ = DEST.read_text().split("\n\n", 1)
        old = dict(line.split(": ", 1) for line in old_header.splitlines())
        for key in ("generated-at", "platform", "python-version"):
            metadata[key] = old[key]
    output = ("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n" + "\n".join(rows) + "\n").encode()
    if args.check:
        if DEST.read_bytes() != output:
            raise SystemExit("reference probe fixture differs: " + str(DEST))
        label = "verified"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(output)
        label = "generated"
    print(label, DEST.relative_to(ROOT), "rows=" + str(len(records)))
    print("statuses", dict(Counter(value["status"] for _, value, _ in records)))
    print("local-contract", dict(Counter(value.get("comparison", value["status"]) for _, _, value in records)))
    print("generator-sha256", REVISION)
    print("fixture-sha256", hashlib.sha256(output).hexdigest())


if __name__ == "__main__":
    main()
