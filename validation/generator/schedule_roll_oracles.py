#!/usr/bin/env python3
"""Independent bounded civil-enumeration oracle for four explicit roll rules."""

import argparse
import calendar
from collections import Counter
import datetime as dt
from functools import lru_cache
import hashlib
from pathlib import Path
import platform


ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/roll-oracle.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
RULES = ("ThirdWednesday", "ThirdWednesdayInclusive", "Twentieth", "TwentiethIMM")
TENORS = ((1, "Months"), (2, "Months"), (3, "Months"), (4, "Months"),
          (6, "Months"), (12, "Months"), (18, "Months"), (1, "Years"), (2, "Years"))
QUARTER_TENORS = ((3, "Months"), (6, "Months"), (12, "Months"),
                  (18, "Months"), (1, "Years"), (2, "Years"))
COLUMNS = ("id profile start end length unit rule status limit anchor_year anchor_month "
           "resolved_start resolved_end dates regularity error_role error_input").split()


def month_index(date):
    return 12 * (date.year - 1) + date.month - 1


def civil(index, day):
    year, month = divmod(index, 12)
    return dt.date(year + 1, month + 1, day) if 0 <= year < 9999 else None


@lru_cache(maxsize=None)
def roll_point(index, rule):
    first = civil(index, 1)
    if first is None:
        return None
    if rule.startswith("ThirdWednesday"):
        # Enumerate the actual Gregorian month, including all prior Wednesdays.
        # This does not use the common weekday-offset +14 implementation.
        last = calendar.monthrange(first.year, first.month)[1]
        wednesdays = [dt.date(first.year, first.month, day) for day in range(1, last + 1)
                      if dt.date(first.year, first.month, day).weekday() == 2]
        selected = wednesdays[2]
        assert 15 <= selected.day <= 21 and len([x for x in wednesdays if x < selected]) == 2
        return selected
    return dt.date(first.year, first.month, 20)


def eligible_month(index, rule):
    return rule != "TwentiethIMM" or (index + 1) % 3 == 0


def on_or_after(date, rule):
    # Enumerate a bounded civil-month window; no production stride recurrence.
    for index in range(month_index(date), month_index(date) + 5):
        if not eligible_month(index, rule):
            continue
        candidate = roll_point(index, rule)
        if candidate is None:
            return None
        if candidate >= date:
            return candidate
    raise AssertionError("eligible roll must occur within four following months")


def oracle(case):
    start, end, length, unit, rule = (case[key] for key in ("start", "end", "length", "unit", "rule"))
    row = dict(profile=case["profile"], start=str(start), end=str(end),
               length=str(length), unit=unit, rule=rule, limit="128")
    stride = length * (12 if unit == "Years" else 1)
    if length <= 0 or unit not in ("Months", "Years") or rule == "TwentiethIMM" and stride % 3:
        return dict(row, status="schedule.invalid_roll_tenor")
    if rule == "ThirdWednesday":
        anchor = roll_point(month_index(start), rule)
        resolved_start, resolved_end = start, end
    elif rule == "ThirdWednesdayInclusive":
        anchor = roll_point(month_index(start), rule)
        resolved_start, resolved_end = anchor, roll_point(month_index(end), rule)
    else:
        anchor = on_or_after(start, rule)
        if anchor is None:
            return dict(row, status="schedule.roll_date", error_role="anchor", error_input=str(start))
        resolved_start, resolved_end = start, on_or_after(end, rule)
        if resolved_end is None:
            return dict(row, status="schedule.roll_date", error_role="end", error_input=str(end))
    row.update(anchor_year=str(anchor.year), anchor_month=str(anchor.month),
               resolved_start=str(resolved_start), resolved_end=str(resolved_end))
    if resolved_start >= resolved_end:
        return dict(row, status="schedule.resolved_roll_range")
    seed = month_index(anchor)
    # Enumerate EVERY in-range calendar month and select by congruence. This
    # neither increments a roll step nor terminates on a production candidate.
    interior = []
    for index in range(month_index(resolved_start), month_index(resolved_end) + 1):
        distance = index - seed
        if distance < 0 or distance % stride or not eligible_month(index, rule):
            continue
        if rule.startswith("ThirdWednesday") and distance == 0:
            continue
        candidate = roll_point(index, rule)
        if resolved_start < candidate < resolved_end:
            interior.append(candidate)
    dates = [resolved_start] + interior + [resolved_end]

    def member(date):
        index = month_index(date)
        distance = index - seed
        return (distance // stride if distance >= 0 and distance % stride == 0
                and eligible_month(index, rule) and date == roll_point(index, rule) else None)

    flags = []
    for left, right in zip(dates, dates[1:]):
        a, b = member(left), member(right)
        flags.append("Regular" if a is not None and b is not None and b == a + 1 else "Stub")
    assert 2 <= len(dates) <= 128 and len(flags) == len(dates) - 1
    assert all(left < right for left, right in zip(dates, dates[1:]))
    return dict(row, status="value", limit=str(len(dates)),
                dates=";".join(map(str, dates)), regularity=";".join(flags))


def definition(start, end, length, unit, rule, profile):
    assert start is not None and end is not None and start < end
    return dict(start=start, end=end, length=length, unit=unit, rule=rule, profile=profile)


def definitions():
    # Each of all 4,800 months is evaluated under EVERY rule, with six
    # independently specified endpoint/tenor patterns. Strides rotate through
    # all supported samples rather than repeating one redundant full product.
    for year in range(2000, 2400):
        for month in range(1, 13):
            index = 12 * (year - 1) + month - 1
            for rule in RULES:
                point = roll_point(index, rule)
                samples = QUARTER_TENORS if rule == "TwentiethIMM" else TENORS
                rotating = samples[(index - 23988) % len(samples)]
                exact_stride = rotating[0] * (12 if rotating[1] == "Years" else 1)
                # Both sides of a roll; inclusive resolves this range to zero.
                yield definition(point-dt.timedelta(days=1), point+dt.timedelta(days=1),
                                 3 if rule == "TwentiethIMM" else 1, "Months", rule, "cycle-same-month")
                yield definition(point, roll_point(index+3, rule), 3, "Months", rule, "cycle-on-roll")
                yield definition(point+dt.timedelta(days=1), roll_point(index+7, rule)-dt.timedelta(days=1),
                                 *rotating, rule, "cycle-after-before")
                end_index = index+13
                end_first = civil(end_index, 1)
                last = calendar.monthrange(end_first.year, end_first.month)[1]
                yield definition(civil(index, 1), civil(end_index, last), 1, "Years", rule, "cycle-first-last")
                last = calendar.monthrange(year, month)[1]
                yield definition(civil(index, last), civil(index+25, 1), 12, "Months", rule, "cycle-last-first")
                yield definition(point-dt.timedelta(days=1), roll_point(index+exact_stride, rule),
                                 *rotating, rule, "cycle-leading-exact")
    for year in (1, 99, 100, 400, 1900, 2000, 2100, 9998, 9999):
        for month in (1, 2, 3, 6, 9, 12):
            index = 12 * (year-1) + month-1
            for rule in RULES:
                point = roll_point(index, rule)
                for length, unit in QUARTER_TENORS if rule == "TwentiethIMM" else TENORS:
                    for horizon in (0, 1, 3, 11):
                        final = roll_point(index+horizon, rule)
                        if final is None:
                            continue
                        end = final+dt.timedelta(days=1)
                        yield definition(point-dt.timedelta(days=1), end, length, unit, rule, "century-extreme")
    for rule in RULES:
        for length, unit in ((2**31-1,"Months"),(2**31-1,"Years")):
            for start,end in ((dt.date.min,dt.date(1,12,31)),
                              (dt.date(2026,2,6),dt.date(2026,11,17)),
                              (dt.date(9999,1,1),dt.date.max)):
                yield definition(start,end,length,unit,rule,"huge-stored-tenor")
        for start,end in ((dt.date(9999,12,1),dt.date.max),
                          (dt.date(9999,12,19),dt.date(9999,12,21)),
                          (dt.date(9999,12,21),dt.date.max),
                          (dt.date.min,dt.date(1,1,2))):
            yield definition(start,end,3,"Months",rule,"required-endpoint-extreme")
        for unit in ("Days","Weeks","Months","Years"):
            for length in (-2**31,-1,0,1,2,4):
                yield definition(dt.date(2026,2,6),dt.date(2026,11,17),length,unit,rule,"tenor-domain")


def render(generated_at):
    rows = [oracle(case) for case in definitions()]
    statuses = Counter(row["status"] for row in rows)
    profiles = Counter(row["profile"] for row in rows)
    flags = Counter(flag for row in rows if row["status"] == "value" for flag in row["regularity"].split(";"))
    counts = Counter(row["rule"] for row in rows)
    for index,row in enumerate(rows):
        row["id"] = f"SCHEDULE-ROLL-{index:06}"
    format_counts = lambda counter: ";".join(k+"="+str(v) for k,v in sorted(counter.items()))
    headers = {
        "schema":"1", "fixture-set":"SCHEDULE-ROLL-CIVIL-ORACLE", "domain":"schedule",
        "description":"Four explicit civil rolls; Gregorian400-year months, endpoint projections, true adjacent-roll flags, stored strides and exact final caps",
        "reference":"oracle:Python datetime/calendar", "reference-version":"Python "+platform.python_version(),
        "reference-source":"M7_SPECIAL_RULES_RESEARCH adopted local civil contract; full month/day enumeration and in-span congruence membership; no Rust/QuantLib outputs",
        "tolerance":"TOL-E0 exact dates, provenance, interval regularity and typed statuses",
        "reproduction":"generator-sha256="+REVISION+"; python3 validation/generator/schedule_roll_oracles.py",
        "generated-at":generated_at, "platform":platform.system()+" "+platform.machine(),
        "scope":"Owned NullCalendar and two Unadjusted conventions; no CDS families, adjusted market/calendar claims or automatic ICMA",
        "cycle":"2000..2399 all4800 months; each rule six endpoint/stride profiles",
        "record-count":str(len(rows)), "status-counts":format_counts(statuses),
        "profile-counts":format_counts(profiles), "rule-counts":format_counts(counts), "flag-counts":format_counts(flags),
    }
    body = "\t".join(COLUMNS)+"\n"+"\n".join("\t".join(str(row.get(k,"")) for k in COLUMNS) for row in rows)+"\n"
    return "\n".join(k+": "+v for k,v in headers.items())+"\n\n"+body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check",action="store_true")
    args = parser.parse_args()
    if args.check:
        existing = DEST.read_text()
        timestamp = next(line.split(": ",1)[1] for line in existing.splitlines() if line.startswith("generated-at: "))
        data = render(timestamp)
        # Runtime platform is evidence, not a condition on portable exact data.
        old_header,old_body = existing.split("\n\n",1)
        new_header,new_body = data.split("\n\n",1)
        old = dict(line.split(": ",1) for line in old_header.splitlines())
        new = dict(line.split(": ",1) for line in new_header.splitlines())
        if old_body != new_body or any(old.get(k)!=v for k,v in new.items() if k!="platform"):
            raise SystemExit("roll civil oracle stale; regenerate and review")
        action = "verified"
    else:
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.parent.mkdir(parents=True,exist_ok=True)
        DEST.write_text(data)
        action = "generated"
    print(action,DEST.relative_to(ROOT),next(x for x in data.splitlines() if x.startswith("record-count: ")),flush=True)


if __name__ == "__main__":
    main()
