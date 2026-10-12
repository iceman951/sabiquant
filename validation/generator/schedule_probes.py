#!/usr/bin/env python3
"""Bounded, attested public QuantLib 1.43 schedule observations, not oracles."""

import argparse
import calendar as gregorian_calendar
import datetime as dt
import hashlib
import importlib.metadata
import importlib.util
import json
import os
import pathlib
import platform
import subprocess
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/probes.fixture"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
WHEEL_SHA256 = "85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c"
WORKER_TIMEOUT = 10
MAX_OUTPUT_DATES = 400
RULES = ("Forward", "Backward", "Zero", "ThirdWednesday",
         "ThirdWednesdayInclusive", "Twentieth", "TwentiethIMM", "OldCDS",
         "CDS", "CDS2015")


def scenarios():
    def case(name, group, start, end, rule="Forward", length=1, unit="Months",
             calendar="Null", convention="Unadjusted", termination=None,
             eom=False, first="", penultimate=""):
        return dict(id=name, group=group, start=start, end=end, rule=rule,
                    length=length, unit=unit, calendar=calendar,
                    convention=convention, termination=termination or convention,
                    eom=eom, first=first, penultimate=penultimate)

    result = [case("RULE-" + rule.upper(), "ten-rules", "2026-02-06",
                   "2026-11-17", rule=rule, length=3) for rule in RULES]
    for rule in ("Forward", "Backward"):
        result.append(case("ANCHOR-JAN31-" + rule.upper(), "anchored-monthly",
                           "2026-01-31", "2026-05-31", rule=rule))
    result.extend((
        case("STUB-AUTO-FINAL", "stubs", "2026-01-15", "2026-10-31", length=3),
        case("STUB-AUTO-FIRST", "stubs", "2026-01-15", "2026-10-31",
             rule="Backward", length=3),
        case("STUB-EXPLICIT-SHORT-FIRST", "stubs", "2026-01-15", "2026-10-31",
             rule="Backward", length=3, first="2026-01-31"),
        case("STUB-EXPLICIT-LONG-FIRST", "stubs", "2026-01-15", "2026-10-31",
             rule="Backward", length=3, first="2026-04-30"),
        case("STUB-EXPLICIT-SHORT-FINAL", "stubs", "2026-01-31", "2026-11-15",
             length=3, penultimate="2026-10-31"),
        case("STUB-EXPLICIT-LONG-FINAL", "stubs", "2026-01-31", "2026-11-15",
             length=3, penultimate="2026-07-31"),
        case("STUB-EXPLICIT-BOTH", "stubs", "2026-01-15", "2026-11-15",
             length=3, first="2026-01-31", penultimate="2026-10-31"),
    ))
    for rule, endpoint, choices in (
            ("Forward", "start", ("2026-01-30", "2026-01-31")),
            ("Backward", "end", ("2026-05-29", "2026-05-31"))):
        for seed in choices:
            for eom in (False, True):
                for convention in ("Unadjusted", "Following"):
                    name = "EOM-" + rule.upper() + "-" + seed + "-" + (
                        "PRESERVE" if eom else "IGNORE") + "-" + convention.upper()
                    result.append(case(name, "calendar-business-eom",
                                       seed if endpoint == "start" else "2026-01-01",
                                       seed if endpoint == "end" else "2026-06-30",
                                       rule=rule, calendar="Weekends", eom=eom,
                                       convention=convention))
    for convention in ("Unadjusted", "Following", "Preceding", "ModifiedFollowing"):
        result.append(case("COLLISION-DAILY-" + convention.upper(), "collisions",
                           "2026-01-09", "2026-01-13", unit="Days",
                           calendar="Weekends", convention=convention))
    for termination in ("Unadjusted", "Following", "Preceding", "ModifiedFollowing"):
        result.append(case("TERMINATION-" + termination.upper(), "endpoints",
                           "2026-01-31", "2026-05-31", calendar="Weekends",
                           convention="Following", termination=termination))
    result.append(case("TARGET-EASTER-DAILY", "target",
                       "2026-04-02", "2026-04-07", unit="Days",
                       calendar="TARGET", convention="Following"))
    for length, unit in ((7, "Days"), (1, "Weeks")):
        result.append(case("CALENDAR-RECURRENCE-" + unit.upper(), "recurrence-units",
                           "2026-01-09", "2026-02-06", length=length, unit=unit,
                           calendar="Weekends", convention="Unadjusted"))
    assert len(result) == 46 and len({case["id"] for case in result}) == 46
    return result


def attest(wheel_path):
    """Attest every shipped module file before importing the native runtime."""
    wheel_bytes = wheel_path.read_bytes()
    if not wheel_path.name.lower().startswith("quantlib-1.43-") or wheel_path.suffix != ".whl":
        raise ValueError("provide the previously attested QuantLib 1.43 wheel")
    if hashlib.sha256(wheel_bytes).hexdigest() != WHEEL_SHA256:
        raise ValueError("wheel digest differs from the established local reference")
    spec = importlib.util.find_spec("QuantLib")
    if spec is None or spec.origin is None or importlib.metadata.version("QuantLib") != "1.43":
        raise ValueError("installed distribution must be QuantLib 1.43")
    installed_root = pathlib.Path(spec.origin).resolve().parent.parent
    manifest = []
    with zipfile.ZipFile(wheel_path) as wheel:
        members = sorted(name for name in wheel.namelist()
                         if name.startswith("QuantLib/") and not name.endswith("/"))
        if not members:
            raise ValueError("wheel has no QuantLib module files")
        for name in members:
            expected = hashlib.sha256(wheel.read(name)).hexdigest()
            installed = installed_root / name
            if not installed.is_file() or hashlib.sha256(installed.read_bytes()).hexdigest() != expected:
                raise ValueError("installed module differs from wheel: " + name)
            manifest.append(name + "=" + expected)
    return hashlib.sha256("\n".join(manifest).encode()).hexdigest(), len(manifest)


def validate_inputs(case):
    start, end = dt.date.fromisoformat(case["start"]), dt.date.fromisoformat(case["end"])
    if start.year != 2026 or end.year != 2026 or not 0 < (end-start).days <= 366:
        raise ValueError("probe range must be positive, bounded and inside 2026")
    if not 1 <= case["length"] <= 7 or case["unit"] not in ("Days", "Weeks", "Months"):
        raise ValueError("unexpected probe tenor")
    if case["rule"] not in RULES or case["calendar"] not in ("Null", "Weekends", "TARGET"):
        raise ValueError("unsupported probe rule/calendar")
    stubs = [dt.date.fromisoformat(case[name]) for name in ("first", "penultimate") if case[name]]
    if any(not start < date < end for date in stubs) or len(stubs) == 2 and stubs[0] >= stubs[1]:
        raise ValueError("invalid explicit probe stub dates")


def worker(case, wheel_path):
    validate_inputs(case)
    manifest, files = attest(wheel_path)
    import QuantLib as ql
    if ql.__version__ != "1.43":
        raise ValueError("public runtime version differs from attested distribution")

    def date(text):
        if not text:
            return ql.Date()  # Public API absence sentinel; no effective date is absent.
        value = dt.date.fromisoformat(text)
        return ql.Date(value.day, value.month, value.year)

    def iso(value):
        return "%04d-%02d-%02d" % (value.year(), value.month(), value.dayOfMonth())

    def calendar_eom(text):
        value = dt.date.fromisoformat(text)
        return value.day == gregorian_calendar.monthrange(value.year, value.month)[1]

    calendar = {"Null": ql.NullCalendar, "Weekends": ql.WeekendsOnly,
                "TARGET": ql.TARGET}[case["calendar"]]()
    rule = getattr(ql.DateGeneration, case["rule"])
    try:
        schedule = ql.Schedule(date(case["start"]), date(case["end"]),
                               ql.Period(case["length"], getattr(ql, case["unit"])),
                               calendar, getattr(ql, case["convention"]),
                               getattr(ql, case["termination"]), rule, case["eom"],
                               date(case["first"]), date(case["penultimate"]))
    except RuntimeError as error:
        return dict(status="constructor-error", error_type=type(error).__name__,
                    error_message=str(error), runtime_version=ql.__version__,
                    module_manifest=manifest, module_files=files)
    dates = [iso(value) for value in schedule.dates()]
    if not 2 <= len(dates) <= MAX_OUTPUT_DATES:
        raise ValueError("public output exceeds the probe boundary-count guard")
    if any(not "2025-01-01" <= value <= "2027-12-31" for value in dates):
        raise ValueError("public output leaves the probe date envelope")
    units = {getattr(ql, name): name for name in ("Days", "Weeks", "Months", "Years")}
    rules = {getattr(ql.DateGeneration, name): name for name in RULES}
    conventions = {getattr(ql, name): name for name in (
        "Unadjusted", "Following", "ModifiedFollowing", "Preceding",
        "ModifiedPreceding", "HalfMonthModifiedFollowing", "Nearest")}
    adjustment_control = []
    if case["group"] in ("collisions", "target"):
        start, end = dt.date.fromisoformat(case["start"]), dt.date.fromisoformat(case["end"])
        for offset in range((end-start).days+1):
            original = str(start + dt.timedelta(days=offset))
            adjusted = calendar.adjust(date(original), getattr(ql, case["convention"]))
            adjustment_control.append(dict(original=original, adjusted=iso(adjusted)))
    advance_control = []
    if case["group"] == "calendar-business-eom":
        seed = case["start"] if case["rule"] == "Forward" else case["end"]
        direction = 1 if case["rule"] == "Forward" else -1
        for count in (1, 2, 4):
            shifted = calendar.advance(date(seed), ql.Period(direction*count, ql.Months),
                                       getattr(ql, case["convention"]), case["eom"])
            advance_control.append(dict(seed=seed, signed_months=direction*count,
                                        result=iso(shifted)))
    return dict(status="value", dates=dates, boundary_count=len(dates),
                regularity=[bool(schedule.isRegular(index)) for index in range(1, len(dates))]
                if schedule.hasIsRegular() else None,
                has_regularity=bool(schedule.hasIsRegular()),
                start_date=iso(schedule.startDate()), end_date=iso(schedule.endDate()),
                tenor_length=schedule.tenor().length(), tenor_unit=units[schedule.tenor().units()],
                rule=rules[schedule.rule()], eom=bool(schedule.endOfMonth()),
                convention=conventions[schedule.businessDayConvention()],
                termination=conventions[schedule.terminationDateBusinessDayConvention()],
                calendar_name=schedule.calendar().name(),
                start_calendar_eom=calendar_eom(case["start"]),
                start_business_eom=bool(calendar.isEndOfMonth(date(case["start"]))),
                end_calendar_eom=calendar_eom(case["end"]),
                end_business_eom=bool(calendar.isEndOfMonth(date(case["end"]))),
                strictly_increasing=all(first < last for first, last in zip(dates, dates[1:])),
                daily_adjustment_control=adjustment_control,
                calendar_advance_control=advance_control,
                runtime_version=ql.__version__, module_manifest=manifest, module_files=files)


def collect(wheel_path, manifest, files):
    records = []
    environment = dict(os.environ)
    environment.pop("PYTHONPATH", None)
    environment.pop("PYTHONHOME", None)
    for case in scenarios():
        validate_inputs(case)
        try:
            process = subprocess.run(
                [sys.executable, "-I", str(pathlib.Path(__file__).resolve()),
                 "--worker", case["id"], "--quantlib-wheel", str(wheel_path.resolve())],
                capture_output=True, text=True, timeout=WORKER_TIMEOUT, env=environment,
                check=False)
        except subprocess.TimeoutExpired as error:
            raise SystemExit("bounded probe timed out; no fixture written: " + case["id"]) from error
        if process.returncode or process.stderr:
            raise SystemExit("probe worker failed: " + case["id"] + ": " + process.stderr)
        observation = json.loads(process.stdout)
        if observation["runtime_version"] != "1.43" or observation["module_manifest"] != manifest or observation["module_files"] != files:
            raise ValueError("worker runtime/module attestation changed: " + case["id"])
        records.append((case, observation))
    return records


def compact(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=pathlib.Path, required=True)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--worker", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        selected = next((case for case in scenarios() if case["id"] == args.worker), None)
        if selected is None:
            parser.error("unknown bounded worker case")
        print(compact(worker(selected, args.quantlib_wheel)))
        return
    manifest, files = attest(args.quantlib_wheel)
    records = collect(args.quantlib_wheel, manifest, files)
    metadata = {
        "schema": "1", "fixture-set": "M7-SCHEDULE-REFERENCE-PROBES",
        "domain": "schedule.reference-probes",
        "description": "46 bounded public constructor/accessor observations; reference-only, no SabiQuant parity or financial oracle",
        "reference": "QuantLib", "reference-version": "1.43 (reported by attested worker runtime)",
        "reference-source": "Attested wheel public Schedule constructor/dates/regularity/endpoints/metadata and Calendar EOM/adjust/advance controls; no schedule.cpp derivation",
        "tolerance": "Exact recorded reference text; no SabiQuant expectation or tolerance claim",
        "reproduction": "generator-sha256=" + REVISION + "; python3 validation/generator/schedule_probes.py --quantlib-wheel " + args.quantlib_wheel.name,
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(),
        "python-version": platform.python_version(),
        "quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": WHEEL_SHA256,
        "quantlib-module-files": str(files), "quantlib-module-manifest-sha256": manifest,
        "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995 from docs/QUANTLIB_REFERENCE.md; wheel source commit not independently attested",
        "execution-bound": "One isolated subprocess per case; 10 second timeout; 2026 positive input spans <=366 days; output 2..400 dates within 2025..2027; timeout/process failure aborts without writing",
        "scope": "M7 preparation only; excluded from M6 oracle/compatibility/acceptance evidence",
    }
    columns = ("id", "group", "start", "end", "tenor_length", "tenor_unit", "rule", "calendar", "convention", "termination", "end_of_month", "first_coupon_date", "penultimate_coupon_date", "status", "public_observation")
    lines = ["\t".join(columns)]
    for case, observation in records:
        values = (case["id"], case["group"], case["start"], case["end"],
                  str(case["length"]), case["unit"], case["rule"], case["calendar"],
                  case["convention"], case["termination"], str(case["eom"]).lower(),
                  case["first"], case["penultimate"], observation["status"], compact(observation))
        lines.append("\t".join(values))
    body = "\n".join(lines) + "\n"
    if args.check:
        old_header, _ = DEST.read_text().split("\n\n", 1)
        old = dict(line.split(": ", 1) for line in old_header.splitlines())
        for key in ("generated-at", "platform", "python-version"):
            metadata[key] = old[key]
    output = ("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n" + body).encode()
    if args.check:
        if DEST.read_bytes() != output:
            raise SystemExit("reference probe fixture differs: " + str(DEST))
        action = "verified"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(output)
        action = "generated"
    counts = {status: sum(observation["status"] == status for _, observation in records)
              for status in sorted({observation["status"] for _, observation in records})}
    print(action, DEST.relative_to(ROOT), "records=" + str(len(records)), compact(counts))


if __name__ == "__main__":
    main()
