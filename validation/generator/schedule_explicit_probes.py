#!/usr/bin/env python3
"""Attested, isolated public QuantLib explicit-coupon observations, not oracles."""

import argparse
import calendar as gregorian
import datetime as dt
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/explicit_probes.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
HELPER = Path(__file__).with_name("schedule_probes.py")
HELPER_REVISION = hashlib.sha256(HELPER.read_bytes()).hexdigest()
SPEC = importlib.util.spec_from_file_location("frozen_schedule_attestation", HELPER)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load frozen schedule attestation helper")
REFERENCE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REFERENCE)
TIMEOUT = 10


def scenarios():
    def case(name, group, rule="Forward", start="2026-01-15", end="2026-11-15",
             length=3, first="", penultimate="", eom=False, calendar="Null",
             convention="Unadjusted"):
        return dict(id=name, group=group, start=start, end=end, length=length,
                    unit="Months", rule=rule, calendar=calendar,
                    convention=convention, termination=convention, eom=eom,
                    first=first, penultimate=penultimate)

    result = []
    for old in REFERENCE.scenarios():
        if old["id"].startswith("STUB-EXPLICIT-"):
            existing = dict(old)
            existing["id"] = "EXPLICIT-EXISTING-" + old["id"]
            existing["group"] = "existing-five-coupon-cases"
            result.append(existing)
    for rule in ("Forward", "Backward"):
        result.append(case("EXPLICIT-EQUAL-" + rule.upper(), "equal-roles", rule,
                           first="2026-07-15", penultimate="2026-07-15"))
        result.append(case("EXPLICIT-REVERSED-" + rule.upper(), "reversed-roles", rule,
                           first="2026-07-15", penultimate="2026-04-15"))
        result.append(case("EXPLICIT-CLAMP-" + rule.upper(), "clamp-lattice-regularity", rule,
                           start="2026-01-31", end="2026-05-31", length=1,
                           first="2026-02-28", penultimate="2026-04-28"))
        result.append(case("EXPLICIT-OFFLATTICE-" + rule.upper(), "off-lattice-middle", rule,
                           first="2026-02-17", penultimate="2026-09-18"))
        for eom in (False, True):
            suffix = rule.upper() + ("-PRESERVE" if eom else "-IGNORE")
            result.append(case("EXPLICIT-CALENDAR-EOM-" + suffix, "explicit-calendar-eom-anchor", rule,
                               end="2026-09-15", length=2,
                               first="2026-02-28", penultimate="2026-08-31", eom=eom))
            result.append(case("EXPLICIT-BUSINESS-EOM-" + suffix, "explicit-business-only-eom-anchor", rule,
                               end="2026-05-15" if rule == "Forward" else "2026-06-15", length=1,
                               first="2026-01-30" if rule == "Forward" else "2026-01-29",
                               penultimate="2026-04-30" if rule == "Forward" else "2026-05-29",
                               eom=eom, calendar="Weekends", convention="Following"))
    for role, value, label in [
        ("first", "2026-01-15", "AT-START"),
        ("first", "2026-10-15", "AT-END"),
        ("first", "2026-01-01", "BEFORE-START"),
        ("first", "2026-11-01", "AFTER-END"),
        ("penultimate", "2026-01-15", "AT-START"),
        ("penultimate", "2026-10-15", "AT-END"),
        ("penultimate", "2026-01-01", "BEFORE-START"),
        ("penultimate", "2026-11-01", "AFTER-END"),
    ]:
        result.append(case("EXPLICIT-RANGE-" + role.upper() + "-" + label,
                           "relative-endpoint-domain", end="2026-10-15", **{role: value}))
    assert len(result) == 29 and len({entry["id"] for entry in result}) == 29
    return result


def validate(case):
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))
    if start.year != 2026 or end.year != 2026 or not 0 < (end - start).days <= 366:
        raise ValueError("probe start/end must be positive bounded 2026 spans")
    if case["unit"] != "Months" or not 1 <= case["length"] <= 3:
        raise ValueError("bounded positive monthly tenor required")
    if case["rule"] not in ("Forward", "Backward") or case["calendar"] not in ("Null", "Weekends"):
        raise ValueError("unexpected probe rule/calendar")
    coupons = [dt.date.fromisoformat(case[key]) for key in ("first", "penultimate") if case[key]]
    if not coupons or any(value.year != 2026 for value in coupons):
        raise ValueError("at least one explicit 2026 coupon required")
    if case["group"] != "relative-endpoint-domain" and any(not start < value < end for value in coupons):
        raise ValueError("only named range-rejection probes may use noninterior coupons")
    # Equal/reversed roles and relative-endpoint rejection cases deliberately
    # reach the bounded public constructor; outputs are retained, never repaired.


def worker(case, wheel):
    validate(case)
    manifest, files = REFERENCE.attest(wheel)
    import QuantLib as ql
    if ql.__version__ != "1.43":
        raise ValueError("public runtime version differs from attested wheel")

    def date(text):
        if not text:
            return ql.Date()
        value = dt.date.fromisoformat(text)
        return ql.Date(value.day, value.month, value.year)

    def iso(value):
        return "%04d-%02d-%02d" % (value.year(), value.month(), value.dayOfMonth())

    def calendar_eom(text):
        value = dt.date.fromisoformat(text)
        return value.day == gregorian.monthrange(value.year, value.month)[1]

    calendar = ql.NullCalendar() if case["calendar"] == "Null" else ql.WeekendsOnly()
    common = dict(runtime_version=ql.__version__, module_manifest=manifest, module_files=files)
    try:
        schedule = ql.Schedule(date(case["start"]), date(case["end"]),
                               ql.Period(case["length"], ql.Months), calendar,
                               getattr(ql, case["convention"]), getattr(ql, case["termination"]),
                               getattr(ql.DateGeneration, case["rule"]), case["eom"],
                               date(case["first"]), date(case["penultimate"]))
    except RuntimeError as error:
        return dict(common, status="constructor-error", error_type=type(error).__name__,
                    error_message=str(error))
    dates = [iso(value) for value in schedule.dates()]
    if not 2 <= len(dates) <= 400 or any(not "2025-01-01" <= value <= "2027-12-31" for value in dates):
        raise ValueError("public output exceeds bounded date/count envelope")
    roles = {}
    for key in ("first", "penultimate"):
        if case[key]:
            roles[key] = dict(date=case[key], calendar_eom=calendar_eom(case[key]),
                              business_eom=bool(calendar.isEndOfMonth(date(case[key]))))
    seed = (case["first"] or case["start"]) if case["rule"] == "Forward" else (case["penultimate"] or case["end"])
    return dict(common, status="value", dates=dates, boundary_count=len(dates),
                strictly_increasing=all(left < right for left, right in zip(dates, dates[1:])),
                has_regularity=bool(schedule.hasIsRegular()),
                regularity=[bool(schedule.isRegular(i)) for i in range(1, len(dates))]
                if schedule.hasIsRegular() else None,
                start_date=iso(schedule.startDate()), end_date=iso(schedule.endDate()),
                tenor_length=schedule.tenor().length(), eom=bool(schedule.endOfMonth()),
                calendar_name=schedule.calendar().name(), explicit_input_roles=roles,
                local_contract_seed=dict(date=seed, calendar_eom=calendar_eom(seed),
                                         business_eom=bool(calendar.isEndOfMonth(date(seed)))))


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
            raise SystemExit("probe timed out; no fixture written: " + case["id"]) from error
        if process.returncode or process.stderr:
            raise SystemExit("probe worker failed: " + case["id"] + ": " + process.stderr)
        observation = json.loads(process.stdout)
        if observation["runtime_version"] != "1.43" or observation["module_manifest"] != manifest or observation["module_files"] != files:
            raise ValueError("runtime/module attestation changed: " + case["id"])
        records.append((case, observation))
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
            parser.error("unknown bounded probe")
        print(REFERENCE.compact(worker(selected, args.quantlib_wheel)))
        return
    manifest, files = REFERENCE.attest(args.quantlib_wheel)
    records = collect(args.quantlib_wheel, manifest, files)
    metadata = {
        "schema": "1", "fixture-set": "M7-EXPLICIT-COUPON-REFERENCE-PROBES",
        "domain": "schedule.reference-probes",
        "description": "29 bounded explicit coupon constructor/accessor observations; preserves unordered values/errors; no financial oracle",
        "reference": "QuantLib", "reference-version": "1.43 (attested public worker runtime)",
        "reference-source": "Public Schedule constructor/dates/regularity/endpoints and Calendar business-EOM; independent local-contract seed annotation; no schedule.cpp read or translation",
        "tolerance": "Exact recorded reference text; local validity/regularity differences require explicit classification",
        "reproduction": "generator-sha256=" + REVISION + "; attestation-helper-sha256=" + HELPER_REVISION + "; python3 validation/generator/schedule_explicit_probes.py --quantlib-wheel " + args.quantlib_wheel.name,
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
        "quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": REFERENCE.WHEEL_SHA256,
        "quantlib-module-files": str(files), "quantlib-module-manifest-sha256": manifest,
        "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995 from docs/QUANTLIB_REFERENCE.md; wheel source commit not independently attested",
        "execution-bound": "One isolated -I subprocess per case; 10 second timeout; positive <=366-day 2026 input spans and 2026 coupons; 2..400 output dates within 2025..2027; timeout/process failure aborts before writing",
        "scope": "Explicit coupon local tranche reference observations only; stricter local domains/named lattice are separate; no M7 acceptance",
    }
    columns = ("id", "group", "start", "end", "tenor_length", "tenor_unit", "rule", "calendar", "convention", "termination", "end_of_month", "first_coupon_date", "penultimate_coupon_date", "status", "public_observation")
    lines = ["\t".join(columns)]
    for case, observation in records:
        values = (case["id"], case["group"], case["start"], case["end"], str(case["length"]), case["unit"], case["rule"], case["calendar"], case["convention"], case["termination"], str(case["eom"]).lower(), case["first"], case["penultimate"], observation["status"], REFERENCE.compact(observation))
        lines.append("\t".join(values))
    if args.check:
        old_header, _ = DEST.read_text().split("\n\n", 1)
        old = dict(line.split(": ", 1) for line in old_header.splitlines())
        for key in ("generated-at", "platform", "python-version"):
            metadata[key] = old[key]
    output = ("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n" + "\n".join(lines) + "\n").encode()
    if args.check:
        if output != DEST.read_bytes():
            raise SystemExit("explicit coupon fixture differs")
        action = "verified"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(output)
        action = "generated"
    statuses = {status: sum(value["status"] == status for _, value in records)
                for status in sorted({value["status"] for _, value in records})}
    unordered = sum(value.get("strictly_increasing") is False for _, value in records)
    print(action, DEST.relative_to(ROOT), "records=" + str(len(records)),
          REFERENCE.compact(statuses), "unordered=" + str(unordered))


if __name__ == "__main__":
    main()
