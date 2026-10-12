#!/usr/bin/env python3
"""Independent bounded civil-span oracle for named explicit coupon lattices."""

import argparse
import calendar
import datetime as dt
import hashlib
from pathlib import Path
import platform

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/explicit_oracles.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
MINIMUM, MAXIMUM = dt.date.min, dt.date.max


def month_index(date):
    return 12 * (date.year - 1) + date.month - 1


def calendar_shift(anchor, steps, length, unit, eom):
    """Construct bounded INPUT endpoints, never a recurrent expected-date loop."""
    if unit in ("Days", "Weeks"):
        serial = anchor.toordinal() + steps * length * (7 if unit == "Weeks" else 1)
        return dt.date.fromordinal(serial) if 1 <= serial <= MAXIMUM.toordinal() else None
    month = month_index(anchor) + steps * length * (12 if unit == "Years" else 1)
    year, zero_month = divmod(month, 12)
    if not 0 <= year < 9999:
        return None
    last = calendar.monthrange(year + 1, zero_month + 1)[1]
    seed_last = calendar.monthrange(anchor.year, anchor.month)[1]
    day = last if eom == "Preserve" and anchor.day == seed_last else min(anchor.day, last)
    return dt.date(year + 1, zero_month + 1, day)


def membership(candidate, anchor, length, unit, eom):
    """Return the named anchor coordinate, or absence; no candidate recurrence."""
    if unit in ("Days", "Weeks"):
        distance = candidate.toordinal() - anchor.toordinal()
        step = length * (7 if unit == "Weeks" else 1)
        return distance // step if distance % step == 0 else None
    distance = month_index(candidate) - month_index(anchor)
    step = length * (12 if unit == "Years" else 1)
    if distance % step:
        return None
    last = calendar.monthrange(candidate.year, candidate.month)[1]
    seed_last = calendar.monthrange(anchor.year, anchor.month)[1]
    day = last if eom == "Preserve" and anchor.day == seed_last else min(anchor.day, last)
    return distance // step if candidate.day == day else None


def oracle(case):
    start, end, first, penultimate = (case[key] for key in ("start", "end", "first", "penultimate"))
    regular_start, regular_end = first or start, penultimate or end
    anchor = regular_start if case["rule"] == "Forward" else regular_end
    endpoint = regular_end if case["rule"] == "Forward" else regular_start
    length, unit, eom = (case[key] for key in ("length", "unit", "eom"))
    # Enumerate the ENTIRE bounded civil middle; selection is independent of
    # production recurrence multipliers and its early termination/resource path.
    if unit in ("Days", "Weeks"):
        candidates = (dt.date.fromordinal(day) for day in range(regular_start.toordinal(), regular_end.toordinal() + 1))
    else:
        candidates = []
        for index in range(month_index(regular_start), month_index(regular_end) + 1):
            year, zero_month = divmod(index, 12)
            last = calendar.monthrange(year + 1, zero_month + 1)[1]
            seed_last = calendar.monthrange(anchor.year, anchor.month)[1]
            day = last if eom == "Preserve" and anchor.day == seed_last else min(anchor.day, last)
            candidates.append(dt.date(year + 1, zero_month + 1, day))
    middle = [date for date in candidates if regular_start <= date <= regular_end
              and membership(date, anchor, length, unit, eom) is not None]
    if membership(endpoint, anchor, length, unit, eom) is None:
        assert middle  # The directional seed is always present.
        last = middle[-1] if case["rule"] == "Forward" else middle[0]
        return "schedule.irregular_explicit_middle", [], [], anchor, endpoint, last
    assert middle[0] == regular_start and middle[-1] == regular_end
    dates = ([start] if first else []) + middle + ([end] if penultimate else [])
    flags = []
    for left, right in zip(dates, dates[1:]):
        left_coordinate = membership(left, anchor, length, unit, eom)
        right_coordinate = membership(right, anchor, length, unit, eom)
        flags.append("Regular" if left_coordinate is not None and right_coordinate is not None
                     and right_coordinate == left_coordinate + 1 else "Stub")
    assert 2 <= len(dates) <= 6 and len(flags) == len(dates) - 1
    assert all(left < right for left, right in zip(dates, dates[1:]))
    return "value", dates, flags, anchor, endpoint, None


def definitions():
    for year in [1, 99, 100, 400, 1900, 2000, 2100, 9999]:
        for month in [1, 2, 3, 6, 8, 12]:
            for day in sorted({1, 28, calendar.monthrange(year, month)[1]}):
                anchor = dt.date(year, month, day)
                for unit in ["Days", "Weeks", "Months", "Years"]:
                    for length in [1, 2, 3]:
                        for rule in ["Forward", "Backward"]:
                            for eom in ["Ignore", "Preserve"] if unit in ("Months", "Years") else ["Ignore"]:
                                for role_index, role in enumerate(["first-only", "penultimate-only", "both", "equal"]):
                                    low, high = (0, 2) if rule == "Forward" else (-2, 0)
                                    if role == "equal":
                                        low = high = 0
                                    middle_start = calendar_shift(anchor, low, length, unit, eom)
                                    middle_end = calendar_shift(anchor, high, length, unit, eom)
                                    prior = calendar_shift(anchor, low - 1, length, unit, eom)
                                    following = calendar_shift(anchor, high + 1, length, unit, eom)
                                    if any(value is None for value in [middle_start, middle_end, prior, following]):
                                        continue
                                    profile = (year + month + day + length + role_index) % 3
                                    if profile == 0:
                                        outer_start, outer_end, label = prior, following, "one-step"
                                    elif profile == 1:
                                        before = max(1, (middle_start - prior).days // 2)
                                        after = max(1, (following - middle_end).days // 2)
                                        outer_start = middle_start - dt.timedelta(days=before)
                                        outer_end = middle_end + dt.timedelta(days=after)
                                        label = "half-step-or-one-day"
                                    else:
                                        outer_start = calendar_shift(anchor, low - 2, length, unit, eom)
                                        outer_end = calendar_shift(anchor, high + 2, length, unit, eom)
                                        label = "multi-tenor"
                                        if outer_start is None or outer_end is None:
                                            continue
                                    first = middle_start if role in ("first-only", "both", "equal") else None
                                    penultimate = middle_end if role in ("penultimate-only", "both", "equal") else None
                                    start = outer_start if first else middle_start
                                    end = outer_end if penultimate else middle_end
                                    base = dict(start=start, end=end, first=first, penultimate=penultimate,
                                                length=length, unit=unit, rule=rule, eom=eom,
                                                role=role, profile=label, variant="exact-middle")
                                    yield base
                                    if role != "equal" and (year + month + day + length + role_index) % 3 == 0:
                                        shifted = dict(base, variant="perturbed-middle")
                                        key = "penultimate" if rule == "Forward" and penultimate else "end" if rule == "Forward" else "first" if first else "start"
                                        shifted[key] += dt.timedelta(days=-1 if rule == "Forward" else 1)
                                        # Tenor 1D necessarily accepts any civil endpoint; retain that
                                        # meaningful control rather than fabricating an off-lattice error.
                                        if shifted["start"] < shifted["end"] and (shifted["first"] is None or shifted["penultimate"] is None or shifted["first"] <= shifted["penultimate"]):
                                            yield shifted
    for start in [MINIMUM, MAXIMUM - dt.timedelta(days=2)]:
        coupon, end = start + dt.timedelta(days=1), start + dt.timedelta(days=2)
        for unit in ["Days", "Weeks", "Months", "Years"]:
            for length in [1, 2, 3]:
                for rule in ["Forward", "Backward"]:
                    for eom in ["Ignore", "Preserve"] if unit in ("Months", "Years") else ["Ignore"]:
                        yield dict(start=start, end=end, first=coupon, penultimate=coupon,
                                   length=length, unit=unit, rule=rule, eom=eom, role="equal",
                                   profile="civil-range-edge", variant="exact-middle")


def render(generated_at):
    header = ["schema: 1", "fixture-set: SCHEDULE-EXPLICIT-CIVIL-ORACLE", "domain: schedule",
        "description: named directional explicit coupon lattices; all units, role combinations, EOM, centuries and civil bounds",
        "reference: oracle:Python datetime/calendar", "reference-version: Python " + platform.python_version(),
        "reference-source: whole bounded civil-middle enumeration; exact anchor-coordinate membership and adjacent-coordinate interval flags; no Rust/QuantLib outputs",
        "tolerance: TOL-E0 exact contractual dates, interval regularity and irregular-middle error context",
        "reproduction: generator-sha256=" + REVISION + "; python3 validation/generator/schedule_explicit_oracles.py",
        "generated-at: " + generated_at, "platform: " + platform.system() + " " + platform.machine(),
        "scope: null calendar and Unadjusted conventions; positive tenors; strictly interior ordered/equal explicit roles; no automatic ICMA or specialized rules",
    ]
    rows = []
    for case in definitions():
        assert case["start"] < case["end"]
        assert (case["end"] - case["start"]).days <= 18 * 366
        assert case["first"] is None or case["start"] < case["first"] < case["end"]
        assert case["penultimate"] is None or case["start"] < case["penultimate"] < case["end"]
        assert case["first"] is None or case["penultimate"] is None or case["first"] <= case["penultimate"]
        status, dates, flags, anchor, endpoint, last = oracle(case)
        limit = len(dates) if status == "value" else 256
        values = [f"SCHEDULE-EXPLICIT-{len(rows):06d}", case["role"], case["profile"], case["variant"],
                  str(case["start"]), str(case["end"]), str(case["length"]), case["unit"], case["rule"], case["eom"],
                  str(case["first"]) if case["first"] else "", str(case["penultimate"]) if case["penultimate"] else "",
                  str(limit), status, ";".join(str(value) for value in dates), ";".join(flags), str(anchor), str(endpoint), str(last) if last else ""]
        rows.append("\t".join(values))
    columns = "id\troles\touter_profile\tvariant\tstart\tend\tlength\tunit\trule\teom\tfirst\tpenultimate\tlimit\tstatus\tdates\tregularity\tanchor\tendpoint\tlast_regular"
    return "\n".join(header) + "\n\n" + columns + "\n" + "\n".join(rows) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        date = next(line.split(": ", 1)[1] for line in previous.splitlines() if line.startswith("generated-at: "))
        if render(date) != previous:
            raise SystemExit("explicit civil oracle stale; regenerate and review")
        print("explicit civil oracle reproducible")
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.write_text(data)
        print("explicit civil oracle written:", len(data.splitlines()) - 14, "records")


if __name__ == "__main__":
    main()
