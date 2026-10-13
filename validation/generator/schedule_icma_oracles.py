#!/usr/bin/env python3
"""Independent civil-coordinate hull and exact Fraction generated-ICMA oracle."""

import argparse
import calendar
import datetime as dt
from fractions import Fraction
import hashlib
import importlib.util
from pathlib import Path
import platform

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/icma_oracles.fixture"
HELPER = Path(__file__).with_name("schedule_explicit_oracles.py")
SPEC = importlib.util.spec_from_file_location("civil_membership", HELPER)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("missing independent civil membership helper")
CIVIL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CIVIL)
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
FREQUENCIES = [("Annual", 1, "Years", 1), ("Semiannual", 6, "Months", 2),
    ("EveryFourthMonth", 4, "Months", 3), ("Quarterly", 3, "Months", 4),
    ("Bimonthly", 2, "Months", 6), ("Monthly", 1, "Months", 12),
    ("EveryFourthWeek", 4, "Weeks", 13), ("Biweekly", 2, "Weeks", 26),
    ("Weekly", 1, "Weeks", 52), ("Daily", 1, "Days", 365)]


def definitions():
    for year in [1, 99, 100, 400, 1900, 2000, 2100, 9995]:
        for month in [1, 2, 8, 12]:
            for day in sorted({1, 28, calendar.monthrange(year, month)[1]}):
                seed = dt.date(year, month, day)
                for frequency, length, unit, events in FREQUENCIES:
                    for rule in ["Forward", "Backward"]:
                        for eom in ["Ignore", "Preserve"] if unit in ("Months", "Years") else ["Ignore"]:
                            # Named aliases exercise stored representation separately.
                            stored_length, stored_unit = length, unit
                            if (year + month + day) % 2:
                                if unit == "Years":
                                    stored_length, stored_unit = 12 * length, "Months"
                                elif unit == "Weeks":
                                    stored_length, stored_unit = 7 * length, "Days"
                            shifts = {i: CIVIL.calendar_shift(seed, i, length, unit, eom) for i in range(-5, 6)}
                            if any(value is None for value in shifts.values()):
                                continue
                            for mode in ["None", "Short", "Long", "first", "penultimate", "both", "equal"]:
                                if mode in ("None", "Short", "Long"):
                                    start, end = (seed, shifts[3]) if rule == "Forward" else (shifts[-3], seed)
                                    if mode != "None":
                                        if rule == "Forward": end += dt.timedelta(days=1)
                                        else: start -= dt.timedelta(days=1)
                                    first = penultimate = None
                                    kind, stub = "automatic", mode
                                else:
                                    low, high = (0, 2) if rule == "Forward" else (-2, 0)
                                    if mode == "equal": low = high = 0
                                    first = shifts[low] if mode in ("first", "both", "equal") else None
                                    penultimate = shifts[high] if mode in ("penultimate", "both", "equal") else None
                                    start = shifts[low - 2] + dt.timedelta(days=1) if first else shifts[low]
                                    end = shifts[high + 2] - dt.timedelta(days=1) if penultimate else shifts[high]
                                    kind, stub = "explicit", ""
                                assert start < end
                                assert first is None or start < first < end
                                assert penultimate is None or start < penultimate < end
                                yield dict(start=start, end=end, first=first, penultimate=penultimate,
                                    kind=kind, stub=stub, role=mode, rule=rule, eom=eom, anchor=seed,
                                    length=stored_length, unit=stored_unit, frequency=frequency, events=events,
                                    envelope_start=shifts[-5], envelope_end=shifts[5])


def hull(case):
    anchor, length, unit, eom = (case[key] for key in ("anchor", "length", "unit", "eom"))
    lower, upper = case["envelope_start"], case["envelope_end"]
    # Select congruent points across a bounded civil envelope, not a signed
    # step/extension loop or any Rust-produced schedule/reference dates.
    if unit in ("Days", "Weeks"):
        candidates = (dt.date.fromordinal(i) for i in range(lower.toordinal(), upper.toordinal() + 1))
    else:
        points = []
        seed_last = calendar.monthrange(anchor.year, anchor.month)[1]
        for index in range(CIVIL.month_index(lower), CIVIL.month_index(upper) + 1):
            y, m = divmod(index, 12)
            last = calendar.monthrange(y + 1, m + 1)[1]
            day = last if eom == "Preserve" and anchor.day == seed_last else min(anchor.day, last)
            points.append(dt.date(y + 1, m + 1, day))
        candidates = points
    selected = [value for value in candidates if CIVIL.membership(value, anchor, length, unit, eom) is not None]
    first = max(value for value in selected if value <= case["start"])
    last = min(value for value in selected if value >= case["end"])
    result = [value for value in selected if first <= value <= last]
    assert 2 <= len(result) <= 9 and all(a < b for a, b in zip(result, result[1:]))
    assert result[0] <= case["start"] < case["end"] <= result[-1]
    return result


def fraction(start, end, dates, events):
    low, high = min(start, end), max(start, end)
    total = Fraction(0)
    for first, last in zip(dates, dates[1:]):
        overlap = max(0, (min(high, last) - max(low, first)).days)
        total += Fraction(overlap, events * (last - first).days)
    return -total if start > end else total


def render(generated_at):
    header = ["schema: 1", "fixture-set: SCHEDULE-GENERATED-ICMA-ORACLE", "domain: schedule.icma",
        "description: minimal contractual lattice hull and exact determination-period accrual",
        "reference: oracle:Python datetime/calendar/Fraction", "reference-version: Python " + platform.python_version(),
        "reference-source: civil envelope membership selection; minimal bracketing hull; exact independent overlap Fractions; no Rust or QuantLib outputs",
        "tolerance: TOL-E0 exact reference dates/context; TOL-E2 4 ULP existing ICMA accrual profile",
        "reproduction: generator-sha256=" + REVISION + "; civil-helper-sha256=" + hashlib.sha256(HELPER.read_bytes()).hexdigest() + "; python3 validation/generator/schedule_icma_oracles.py",
        "generated-at: " + generated_at, "platform: " + platform.system() + " " + platform.machine(),
        "scope: all ten explicit named periodic frequencies; automatic and explicit coupon shapes; null calendar Unadjusted; no inferred frequency or adjusted accrual",
    ]
    columns = "id\tcontext\tkind\trole\tstart\tend\tlength\tunit\trule\teom\tstub\tfirst\tpenultimate\tfrequency\tevents\tlimit\tanchor\treferences\tquery_start\tquery_end\tnumerator\tdenominator\tfraction"
    rows = []
    for context, case in enumerate(definitions()):
        dates = hull(case)
        extra_query = [(case["end"], case["start"]), (dates[0], dates[-1]),
            (case["start"], case["start"]),
            (case["start"] + dt.timedelta(days=1), case["end"])][context % 4]
        for start, end in [(case["start"], case["end"]), extra_query]:
            exact = fraction(start, end, dates, case["events"])
            assert abs(exact.numerator) < 2**127 and exact.denominator < 2**127
            values = [f"SCHEDULE-ICMA-{len(rows):06d}", str(context), case["kind"], case["role"], str(case["start"]), str(case["end"]),
                str(case["length"]), case["unit"], case["rule"], case["eom"], case["stub"],
                str(case["first"]) if case["first"] else "", str(case["penultimate"]) if case["penultimate"] else "",
                case["frequency"], str(case["events"]), str(len(dates) - 1), str(case["anchor"]),
                ";".join(str(value) for value in dates), str(start), str(end), str(exact.numerator), str(exact.denominator), repr(float(exact))]
            rows.append("\t".join(values))
    return "\n".join(header) + "\n\n" + columns + "\n" + "\n".join(rows) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        stamp = next(line.split(": ", 1)[1] for line in previous.splitlines() if line.startswith("generated-at: "))
        if render(stamp) != previous:
            raise SystemExit("generated ICMA oracle stale")
        print("generated ICMA oracle reproducible")
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        result = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.write_text(result)
        print("generated ICMA rows:", len(result.splitlines()) - 14)


if __name__ == "__main__":
    main()
