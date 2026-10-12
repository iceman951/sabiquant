#!/usr/bin/env python3
"""Independent bounded civil-lattice schedule oracle; no Rust/QuantLib outputs."""

import argparse
import calendar
import datetime as dt
import hashlib
from pathlib import Path
import platform

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/oracles.fixture"
REVISION = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()


def month_index(date):
    return (date.year - 1) * 12 + date.month - 1


def civil(month, day, preserve):
    year, zero_month = divmod(month, 12)
    if not 0 <= year < 9999:
        return None
    last = calendar.monthrange(year + 1, zero_month + 1)[1]
    return dt.date(year + 1, zero_month + 1, last if preserve else min(day, last))


def oracle(start, end, length, unit, rule, stub, eom):
    """Select all in-span dates congruent to the directional civil seed.

    This enumerates the whole bounded civil span instead of multiplying a
    production recurrence counter or iteratively clamping prior dates.
    """
    anchor = start if rule == "Forward" else end
    opposite = end if rule == "Forward" else start
    if unit in ("Days", "Weeks"):
        step = length * (7 if unit == "Weeks" else 1)
        interior = [dt.date.fromordinal(day)
                    for day in range(start.toordinal() + 1, end.toordinal())
                    if (day - anchor.toordinal()) % step == 0]
        exact = (opposite.toordinal() - anchor.toordinal()) % step == 0
    else:
        step = length * (12 if unit == "Years" else 1)
        seed = month_index(anchor)
        preserve = eom == "Preserve" and anchor.day == calendar.monthrange(anchor.year, anchor.month)[1]
        interior = []
        for month in range(month_index(start), month_index(end) + 1):
            if (month - seed) % step == 0:
                candidate = civil(month, anchor.day, preserve)
                if start < candidate < end:
                    interior.append(candidate)
        exact = ((month_index(opposite) - seed) % step == 0
                 and civil(month_index(opposite), anchor.day, preserve) == opposite)
    if not exact:
        if stub == "None" or stub == "Long" and not interior:
            return None
        if stub == "Long":
            del interior[-1 if rule == "Forward" else 0]
    dates = [start] + interior + [end]
    flags = ["Regular"] * (len(dates) - 1)
    if not exact:
        flags[-1 if rule == "Forward" else 0] = "Stub"
    return dates, flags


def definitions():
    for year in [1, 99, 100, 400, 1900, 2000, 2100, 9998]:
        for month in range(1, 13):
            for day in sorted({28, calendar.monthrange(year, month)[1]}):
                start = dt.date(year, month, day)
                for horizon in [3, 7, 13]:
                    end = civil(month_index(start) + horizon,
                                15 if horizon == 7 else day,
                                horizon == 13)
                    if end is None:
                        continue
                    for length, unit in [(1, "Months"), (2, "Months"), (3, "Months"), (1, "Years")]:
                        for rule in ["Forward", "Backward"]:
                            for stub in ["None", "Short", "Long"]:
                                for eom in ["Ignore", "Preserve"]:
                                    yield start, end, length, unit, rule, stub, eom
    starts = [dt.date.fromisoformat(text) for text in [
        "0001-01-01", "0099-12-28", "0100-02-27", "0400-02-28",
        "1900-02-27", "1999-12-28", "2000-02-27", "2024-01-31",
        "2024-02-29", "2100-02-27", "2400-02-28", "9999-08-01",
    ]]
    for start in starts:
        for span in [1, 7, 31, 97]:
            end = start + dt.timedelta(days=span)
            for length, unit in [(1, "Days"), (3, "Days"), (7, "Days"), (1, "Weeks"), (2, "Weeks")]:
                for rule in ["Forward", "Backward"]:
                    for stub in ["None", "Short", "Long"]:
                        yield start, end, length, unit, rule, stub, "Ignore"


def render(generated_at):
    header = ["schema: 1", "fixture-set: SCHEDULE-CIVIL-ORACLE", "domain: schedule",
        "description: anchored civil lattices; month ends, centuries, both directions, exact/short/long stubs; null calendar",
        "reference: oracle:Python datetime/calendar", "reference-version: Python " + platform.python_version(),
        "reference-source: bounded span enumeration and congruence to original seed; Gregorian month lengths; independently selected boundary dates and regularity; no Rust/QuantLib outputs",
        "tolerance: TOL-E0 exact civil dates and raw interval regularity",
        "reproduction: generator-sha256=" + REVISION + "; python3 validation/generator/schedule_oracles.py",
        "generated-at: " + generated_at, "platform: " + platform.system() + " " + platform.machine(),
        "scope: recurrent Forward/Backward only; owned null calendar, Unadjusted conventions; specialized rules and explicit coupon dates excluded",
    ]
    rows = []
    for values in definitions():
        expected = oracle(*values)
        if expected is None:
            continue
        dates, flags = expected
        start, end, length, unit, rule, stub, eom = values
        assert 2 <= len(dates) <= 98
        assert len(flags) == len(dates) - 1
        assert all(left < right for left, right in zip(dates, dates[1:]))
        rows.append("\t".join([f"SCHEDULE-CIVIL-{len(rows):06d}", start.isoformat(), end.isoformat(),
            str(length), unit, rule, stub, eom, str(len(dates)),
            ";".join(date.isoformat() for date in dates), ";".join(flags)]))
    return "\n".join(header) + "\n\nid\tstart\tend\tlength\tunit\trule\tstub\teom\tlimit\tdates\tregularity\n" + "\n".join(rows) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        previous = DEST.read_text()
        date = next(line.split(": ", 1)[1] for line in previous.splitlines() if line.startswith("generated-at: "))
        if render(date) != previous:
            raise SystemExit("schedule civil oracle stale; regenerate and review")
        print("schedule civil oracle reproducible")
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        data = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.write_text(data)
        print("schedule civil oracle written:", len(data.splitlines()) - 14, "records")


if __name__ == "__main__":
    main()
