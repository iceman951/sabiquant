#!/usr/bin/env python3
"""Independent civil-month membership and exact Fraction roll-ICMA oracle."""

import argparse
import calendar
from collections import Counter
import datetime as dt
from fractions import Fraction
from functools import lru_cache
import hashlib
from pathlib import Path
import platform

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/roll-icma-oracle.fixture"
DESIGN = ROOT / "docs/M7_ROLL_ICMA_DESIGN.md"
CONTRACT = ROOT / "docs/M7_ROLL_ICMA_CONTRACT.md"
RULES = ("ThirdWednesday", "ThirdWednesdayInclusive", "Twentieth", "TwentiethIMM")
FREQUENCIES = (("Monthly", 1, "Months", 12), ("Bimonthly", 2, "Months", 6),
               ("Quarterly", 3, "Months", 4), ("EveryFourthMonth", 4, "Months", 3),
               ("Semiannual", 6, "Months", 2), ("Annual", 12, "Months", 1),
               ("Annual", 1, "Years", 1))
ALL_FREQUENCIES = ("Once", "Annual", "Semiannual", "EveryFourthMonth", "Quarterly",
                   "Bimonthly", "Monthly", "EveryFourthWeek", "Biweekly", "Weekly", "Daily")
FREQUENCY_STRIDES = {frequency: length * (12 if unit == "Years" else 1)
                     for frequency, length, unit, _ in FREQUENCIES}
COLUMNS = ("id profile start end length unit rule frequency events status anchor "
           "resolved_start resolved_end limit references error_offset error_index queries").split()


def month_index(date):
    return 12 * (date.year - 1) + date.month - 1


def civil(index, day):
    year, month = divmod(index, 12)
    return dt.date(year + 1, month + 1, day) if 0 <= year < 9999 else None


@lru_cache(maxsize=None)
def point(index, rule):
    first = civil(index, 1)
    if first is None:
        return None
    if rule.startswith("ThirdWednesday"):
        # Count actual Wednesdays across the whole civil month, independently
        # of a weekday-offset/14-day production implementation.
        days = calendar.monthrange(first.year, first.month)[1]
        wednesdays = [civil(index, day) for day in range(1, days + 1)
                      if civil(index, day).weekday() == 2]
        return wednesdays[2]
    return civil(index, 20)


def projected(date, rule):
    for index in range(month_index(date), month_index(date) + 4):
        if rule == "TwentiethIMM" and (index + 1) % 3:
            continue
        candidate = point(index, rule)
        if candidate is None or candidate >= date:
            return candidate
    raise AssertionError("eligible twentieth missing from four-month window")


def definition(start, end, rule, frequency, length, unit, events, profile):
    assert start is not None and end is not None and start < end
    return dict(start=start, end=end, rule=rule, frequency=frequency, length=length,
                unit=unit, events=events, profile=profile)


def definitions():
    for index in range(23988, 28788):  # Every month of Gregorian years 2000..2399.
        for rule_index, rule in enumerate(RULES):
            allowed = [x for x in FREQUENCIES if rule != "TwentiethIMM"
                       or x[1] * (12 if x[2] == "Years" else 1) % 3 == 0]
            roll = point(index, rule)
            for profile in range(2):
                frequency, length, unit, events = allowed[(index + rule_index + profile) % len(allowed)]
                stride = length * (12 if unit == "Years" else 1)
                if profile == 0:
                    # The nine before/on/after endpoint pairs rotate. The
                    # terminal month sometimes misses the tenor congruence.
                    start = roll + dt.timedelta(days=(index % 3) - 1)
                    horizon = stride * 2 + (index // 9) % 3
                    end = point(index + horizon, rule) + dt.timedelta(days=(index // 3) % 3 - 1)
                    name = "cycle-endpoint-pairs"
                else:
                    # Same-month Wednesday and twentieth starts, including
                    # ranges entirely before/after/crossing q(0). Inclusive
                    # necessarily needs a later requested endpoint month.
                    offsets = ((-3, -1), (-1, 1), (1, 3))[(index // 3) % 3]
                    start = roll + dt.timedelta(days=offsets[0])
                    end = point(index + (1 if rule == "ThirdWednesdayInclusive" else 0), rule)
                    end += dt.timedelta(days=offsets[1])
                    name = "cycle-short-hull"
                yield definition(start, end, rule, frequency, length, unit, events, name)
    for year in (1, 99, 100, 400, 1900, 2000, 2100, 9998, 9999):
        for month in (1, 2, 3, 6, 9, 12):
            index = 12 * (year - 1) + month - 1
            for rule in RULES:
                for frequency, length, unit, events in FREQUENCIES:
                    if rule == "TwentiethIMM" and length * (12 if unit == "Years" else 1) % 3:
                        continue
                    for before, after in ((-1, 1), (0, 0), (1, -1)):
                        start = point(index, rule) + dt.timedelta(days=before)
                        end_point = point(index + 1, rule)
                        if end_point is not None:
                            end = end_point + dt.timedelta(days=after)
                            # Keep source roll construction representable so
                            # civil failures belong to the ICMA extension.
                            if rule == "TwentiethIMM" and projected(end, rule) is None:
                                continue
                            yield definition(start, end, rule, frequency, length, unit, events,
                                             "century-civil-controls")
    for rule in RULES:
        for frequency, length, unit, events in FREQUENCIES:
            if rule == "TwentiethIMM" and length * (12 if unit == "Years" else 1) % 3:
                continue
            for start, end in ((dt.date.min, dt.date(1, 2, 1)),
                               (dt.date(9999, 10, 1), dt.date(9999, 12, 20))):
                yield definition(start, end, rule, frequency, length, unit, events, "civil-limits")
        if rule == "ThirdWednesday":
            for start, end in ((dt.date(1, 1, 16), dt.date(1, 1, 18)),
                               (dt.date(9999, 12, 22), dt.date.max)):
                yield definition(start, end, rule, "Monthly", 1, "Months", 12, "civil-cross-anchor")
        for length, unit in ((3, "Months"), (12, "Months"), (1, "Years"),
                             (2, "Years"), (2**31 - 1, "Years")):
            for frequency in ALL_FREQUENCIES:
                events = next((x[3] for x in FREQUENCIES if x[0] == frequency), 0)
                yield definition(dt.date(2026, 2, 6), dt.date(2026, 11, 17), rule,
                                 frequency, length, unit, events, "frequency-domain")


def exact_fraction(start, end, references, events):
    low, high = min(start, end), max(start, end)
    answer = sum((Fraction(max(0, (min(high, right) - max(low, left)).days),
                           events * (right - left).days)
                  for left, right in zip(references, references[1:])), Fraction(0))
    return -answer if start > end else answer


def oracle(case, serial):
    row = dict(case)
    start, end, rule = case["start"], case["end"], case["rule"]
    stride = case["length"] * (12 if case["unit"] == "Years" else 1)
    if rule.startswith("ThirdWednesday"):
        anchor = point(month_index(start), rule)
        raw_start, raw_end = ((anchor, point(month_index(end), rule))
                              if rule.endswith("Inclusive") else (start, end))
    else:
        anchor = projected(start, rule)
        raw_start, raw_end = start, projected(end, rule)
    assert anchor is not None and raw_end is not None and raw_start < raw_end
    row.update(anchor=anchor, resolved_start=raw_start, resolved_end=raw_end, limit=32)
    frequency = case["frequency"]
    if frequency == "Once":
        return dict(row, status="schedule.invalid_icma_frequency")
    if FREQUENCY_STRIDES.get(frequency) != stride:
        return dict(row, status="schedule.icma_frequency_mismatch")
    anchor_index = month_index(anchor)
    # Enumerate EVERY month across the civil endpoint envelope plus one whole
    # stride each side, selecting by congruence. This is neither a recurrence
    # from q(0) nor production's lower/upper extension loop.
    indices = [i for i in range(month_index(raw_start) - stride,
                               month_index(raw_end) + stride + 1)
               if (i - anchor_index) % stride == 0]
    lower = max(i for i in indices if i < 0 or (point(i, rule) is not None and point(i, rule) <= raw_start))
    upper = min(i for i in indices if i > 119987 or (point(i, rule) is not None and point(i, rule) >= raw_end))
    if lower < 0:
        return dict(row, status="schedule.date", error_offset=lower-anchor_index,
                    error_index=1)
    if upper > 119987:
        return dict(row, status="schedule.date", error_offset=upper-anchor_index,
                    error_index=(anchor_index-lower)//stride+(upper-anchor_index)//stride)
    references = [point(i, rule) for i in indices if lower <= i <= upper]
    assert 2 <= len(references) <= 8
    assert references[0] <= raw_start < raw_end <= references[-1]
    assert references[1] > raw_start and references[-2] < raw_end
    middle = raw_start + (raw_end - raw_start) // 2
    extra = ((raw_start, raw_start), (references[0], references[-1]),
             (middle, raw_end), (references[0] + dt.timedelta(days=1), references[-1] - dt.timedelta(days=1)))[serial % 4]
    queries = []
    for first, last in ((raw_start, raw_end), (raw_end, raw_start), extra):
        fraction = exact_fraction(first, last, references, case["events"])
        assert abs(fraction.numerator) < 2**127 and fraction.denominator < 2**127
        queries.append(",".join(map(str, (first, last, fraction.numerator, fraction.denominator, repr(float(fraction))))))
    return dict(row, status="value", limit=len(references)-1,
                references=";".join(map(str, references)), queries=";".join(queries))


def render(stamp):
    rows = [oracle(case, serial) for serial, case in enumerate(definitions())]
    counts = lambda field: ";".join(k+"="+str(v) for k, v in sorted(Counter(str(x[field]) for x in rows).items()))
    header = {
        "schema": "1", "fixture-set": "SCHEDULE-ROLL-ICMA-ORACLE", "domain": "schedule.icma",
        "description": "Minimal same-roll reference hull and exact integer-overlap Fraction accrual",
        "reference": "oracle:Python datetime/calendar/Fraction", "reference-version": "Python "+platform.python_version(),
        "reference-source": "full civil-month envelope enumeration and congruence selection; actual Wednesday counting; minimal bracketing hull; no Rust or QuantLib outputs",
        "tolerance": "TOL-E0 exact dates/provenance/errors; TOL-E2 4 ULP existing ICMA accrual profile",
        "reproduction": "generator-sha256="+hashlib.sha256(Path(__file__).read_bytes()).hexdigest()+"; python3 validation/generator/schedule_roll_icma_oracles.py",
        "historical-design-sha256": hashlib.sha256(DESIGN.read_bytes()).hexdigest(),
        "adopted-contract-sha256": hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),
        "generated-at": stamp, "platform": platform.system()+" "+platform.machine(),
        "scope": "NullCalendar Unadjusted; explicit matching month frequencies and stored annual aliases; no inferred frequencies, adjusted-calendar or CDS-product claim",
        "cycle": "2000..2399 all4800 months x4rules x2endpointprofiles; rotating endpoint pairs and frequencies, not their Cartesian product",
        "record-count": str(len(rows)), "status-counts": counts("status"), "profile-counts": counts("profile"),
        "rule-counts": counts("rule"), "frequency-counts": counts("frequency"),
    }
    body = "\t".join(COLUMNS)+"\n"
    for serial, row in enumerate(rows):
        row["id"] = f"ROLL-ICMA-{serial:06d}"
        body += "\t".join(str(row.get(column, "")) for column in COLUMNS)+"\n"
    return "\n".join(k+": "+v for k, v in header.items())+"\n\n"+body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        stamp = next(line.split(": ", 1)[1] for line in previous.splitlines() if line.startswith("generated-at: "))
        data = render(stamp)
        old_header, old_body = previous.split("\n\n", 1)
        new_header, new_body = data.split("\n\n", 1)
        old = dict(line.split(": ", 1) for line in old_header.splitlines())
        new = dict(line.split(": ", 1) for line in new_header.splitlines())
        if old_body != new_body or any(old.get(k) != v for k, v in new.items() if k != "platform"):
            raise SystemExit("roll ICMA oracle stale; regenerate and review")
        action = "verified"
    else:
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_text(data)
        action = "generated"
    print(action, DEST.relative_to(ROOT), next(x for x in data.splitlines() if x.startswith("record-count: ")))


if __name__ == "__main__":
    main()
