#!/usr/bin/env python3
"""Attested public QuantLib 1.43 core-calendar fixtures, without market data."""

import argparse
import calendar
import datetime as dt
import hashlib
import pathlib
import platform
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/calendar"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
UNITS = ("Days", "Weeks", "Months", "Years")
CONVENTIONS = ("Unadjusted", "Following", "ModifiedFollowing", "Preceding",
               "ModifiedPreceding", "HalfMonthModifiedFollowing", "Nearest")
HOLIDAYS = tuple(dt.date.fromisoformat(text) for text in (
    "1999-12-31", "2000-01-03", "2000-02-29", "2000-03-01", "2000-06-15",
    "2000-07-03", "2000-07-15", "2000-07-17", "2000-12-25", "2000-12-26",
    "2024-02-29", "2024-03-29", "2024-04-01", "2024-07-03", "2024-08-30",
    "2024-09-02", "2100-02-26", "2100-03-01", "2100-12-31",
))
ALT_HOLIDAYS = tuple(dt.date.fromisoformat(text) for text in (
    "2000-01-03", "2000-02-28", "2000-07-03", "2024-03-28", "2024-07-03",
    "2024-09-03", "2100-02-25", "2100-03-01",
))


class OracleCalendar:
    """Membership from explicit sets; independent of QuantLib and Rust code."""
    def __init__(self, weekends=(), holidays=(), business_overrides=(), members=(), rule=None):
        self.weekends = set(weekends)
        self.holidays = set(holidays)
        self.overrides = set(business_overrides)
        self.members, self.rule = members, rule

    def business(self, date):
        if self.members:
            values = [member.business(date) for member in self.members]
            return all(values) if self.rule == "JoinHolidays" else any(values)
        return date in self.overrides or (date.isoweekday() not in self.weekends and date not in self.holidays)

    def weekend(self, weekday):
        if self.members:
            values = [member.weekend(weekday) for member in self.members]
            return any(values) if self.rule == "JoinHolidays" else all(values)
        return weekday in self.weekends

    def seek(self, date, direction):
        for offset in range(367):
            candidate = date + dt.timedelta(days=direction * offset)
            if self.business(candidate):
                return candidate
        raise ValueError("oracle search limit")

    def adjust(self, date, convention):
        if convention == "Unadjusted" or self.business(date):
            return date
        if convention in ("Following", "ModifiedFollowing", "HalfMonthModifiedFollowing"):
            result = self.seek(date, 1)
            if convention != "Following" and (result.month != date.month or
                    (convention == "HalfMonthModifiedFollowing" and date.day <= 15 < result.day)):
                result = self.seek(date, -1)
            return result
        if convention in ("Preceding", "ModifiedPreceding"):
            result = self.seek(date, -1)
            return self.seek(date, 1) if convention == "ModifiedPreceding" and result.month != date.month else result
        before, after = self.seek(date, -1), self.seek(date, 1)
        return before if (date - before).days < (after - date).days else after

    def month_end(self, date):
        return self.seek(date.replace(day=calendar.monthrange(date.year, date.month)[1]), -1)

    def count(self, start, end, include_start, include_end):
        if start > end:
            return -self.count(end, start, include_end, include_start)
        if start == end:
            return int(include_start and include_end and self.business(start))
        return sum(self.business(date) for date in dates_between(start, end)
                   if (date != start or include_start) and (date != end or include_end))

    def advance(self, date, unit, count, convention, end_of_month):
        if count == 0:
            return self.adjust(date, convention)
        if unit == "Days":
            direction, remaining, result = (1 if count > 0 else -1), abs(count), date
            while remaining:
                result += dt.timedelta(days=direction)
                remaining -= self.business(result)
            return result
        if unit == "Weeks":
            return self.adjust(date + dt.timedelta(days=7 * count), convention)
        target = (date.year - 1) * 12 + date.month - 1 + count * (12 if unit == "Years" else 1)
        year, month = divmod(target, 12)
        year, month = year + 1, month + 1
        result = dt.date(year, month, min(date.day, calendar.monthrange(year, month)[1]))
        if end_of_month:
            if convention == "Unadjusted" and date.day == calendar.monthrange(date.year, date.month)[1]:
                return result.replace(day=calendar.monthrange(year, month)[1])
            if convention != "Unadjusted" and date >= self.month_end(date):
                return self.month_end(result)
        return self.adjust(result, convention)


def dates_between(start, end):
    for ordinal in range(start.toordinal(), end.toordinal() + 1):
        yield dt.date.fromordinal(ordinal)


def scenarios(ql):
    def bespoke(name, weekends, holidays=(), overrides=()):
        value = ql.BespokeCalendar("SabiQuant-fixture-" + name)
        for weekday in weekends:
            value.addWeekend(weekday % 7 + 1)  # ISO -> QuantLib Sunday=1.
        for holiday in holidays:
            value.addHoliday(qdate(ql, holiday))
        for date in overrides:
            value.removeHoliday(qdate(ql, date))
        return value, OracleCalendar(weekends, holidays, overrides)

    null, weekends = ql.NullCalendar(), ql.WeekendsOnly()
    null.resetAddedAndRemovedHolidays()
    weekends.resetAddedAndRemovedHolidays()
    custom, custom_oracle = bespoke("custom", (6, 7), HOLIDAYS)
    alternative, alternative_oracle = bespoke("alternative", (5, 6), ALT_HOLIDAYS)
    overrides = (dt.date(2024, 2, 24), dt.date(2024, 7, 3))
    overridden, overridden_oracle = bespoke("overridden", (6, 7), HOLIDAYS, overrides)
    return (
        ("null", null, OracleCalendar()),
        ("weekends", weekends, OracleCalendar((6, 7))),
        ("custom", custom, custom_oracle),
        ("alternative", alternative, alternative_oracle),
        ("overridden", overridden, overridden_oracle),
        ("joint_holidays", ql.JointCalendar(custom, alternative, ql.JoinHolidays),
         OracleCalendar(members=(custom_oracle, alternative_oracle), rule="JoinHolidays")),
        ("joint_business", ql.JointCalendar(custom, alternative, ql.JoinBusinessDays),
         OracleCalendar(members=(custom_oracle, alternative_oracle), rule="JoinBusinessDays")),
    )


def qdate(ql, date):
    return ql.Date(date.day, date.month, date.year)


def iso(date):
    return dt.date(date.year(), int(date.month()), date.dayOfMonth()).isoformat()


def dense_dates():
    ranges = (
        ("1999-12-20", "2000-03-10"), ("2000-06-10", "2000-07-20"),
        ("2000-12-20", "2001-01-10"), ("2099-12-20", "2100-03-10"),
        ("2024-02-20", "2024-04-05"), ("2024-06-10", "2024-06-20"),
        ("2024-06-28", "2024-07-10"), ("2024-08-25", "2024-09-06"),
    )
    return sorted({date for start, end in ranges for date in dates_between(dt.date.fromisoformat(start), dt.date.fromisoformat(end))})


def run(ql, write):
    contexts = scenarios(ql)

    def membership():
        for name, reference, oracle in contexts:
            for year in (1999, 2000, 2001, 2002, 2099, 2100, 2101, 2024):
                for date in dates_between(dt.date(year, 1, 1), dt.date(year, 12, 31)):
                    business = reference.isBusinessDay(qdate(ql, date))
                    weekend = reference.isWeekend(date.isoweekday() % 7 + 1)
                    if (business, weekend) != (oracle.business(date), oracle.weekend(date.isoweekday())):
                        raise ValueError("unexpected calendar membership discrepancy")
                    yield ("MEM-" + name + "-" + date.isoformat(), name, date, int(business), int(weekend), int(reference.isHoliday(qdate(ql, date))))
    write("core-membership", ("id", "calendar", "date", "business", "weekend", "holiday"), membership())

    def adjustments():
        for name, reference, oracle in contexts:
            for date in dense_dates():
                for convention in CONVENTIONS:
                    result = iso(reference.adjust(qdate(ql, date), getattr(ql, convention)))
                    expected = oracle.adjust(date, convention).isoformat()
                    if result != expected:
                        raise ValueError("unexpected adjustment discrepancy " + repr((name, date, convention, result, expected)))
                    yield ("ADJ-" + name + "-" + date.isoformat() + "-" + convention, name, date, convention, result)
    write("core-adjustments", ("id", "calendar", "date", "convention", "result"), adjustments())

    def advancement():
        starts = tuple(dt.date.fromisoformat(text) for text in (
            "1999-12-31", "2000-01-31", "2000-02-29", "2000-06-15",
            "2024-02-24", "2024-02-29", "2024-03-28", "2024-03-31",
            "2024-06-15", "2024-07-03", "2024-08-31", "2100-02-28",
        ))
        case = 0
        for name, reference, oracle in contexts:
            for date in starts:
                for unit in UNITS:
                    for count in (-2, -1, 0, 1, 2):
                        for convention in CONVENTIONS:
                            for eom in (False, True):
                                case += 1
                                result = iso(reference.advance(qdate(ql, date), count, getattr(ql, unit), getattr(ql, convention), eom))
                                expected = oracle.advance(date, unit, count, convention, eom).isoformat()
                                if result != expected:
                                    raise ValueError("unexpected advancement discrepancy " + repr((name, date, unit, count, convention, eom, result, expected)))
                                yield ("ADV-" + str(case).zfill(6), name, date, unit, count, convention, int(eom), result)
    write("core-advancement", ("id", "calendar", "date", "unit", "count", "convention", "end_of_month", "result"), advancement())

    def counts():
        case = 0
        for name, reference, oracle in contexts:
            for first in (dt.date(2000, 2, 24), dt.date(2024, 6, 29), dt.date(2100, 2, 23)):
                dates = tuple(first + dt.timedelta(days=n) for n in range(12))
                for start in dates:
                    for end in dates:
                        for include_start in (False, True):
                            for include_end in (False, True):
                                case += 1
                                result = reference.businessDaysBetween(qdate(ql, start), qdate(ql, end), include_start, include_end)
                                if result != oracle.count(start, end, include_start, include_end):
                                    raise ValueError("unexpected inclusion/counting discrepancy")
                                yield ("COUNT-" + str(case).zfill(6), name, start, end, int(include_start), int(include_end), result)
    write("core-counts", ("id", "calendar", "start", "end", "include_start", "include_end", "result"), counts())

    def month_edges():
        for name, reference, oracle in contexts:
            for date in dense_dates():
                result = iso(reference.endOfMonth(qdate(ql, date)))
                is_end = reference.isEndOfMonth(qdate(ql, date))
                if result != oracle.month_end(date).isoformat() or is_end != (date >= oracle.month_end(date)):
                    raise ValueError("unexpected business month-end discrepancy")
                yield ("EOM-" + name + "-" + date.isoformat(), name, date, result, int(is_end))
    write("core-month-edges", ("id", "calendar", "date", "end_of_month", "is_end_of_month"), month_edges())

    def lists():
        case = 0
        for name, reference, oracle in contexts:
            for start, end in ((dt.date(2000, 2, 25), dt.date(2000, 3, 5)), (dt.date(2024, 2, 20), dt.date(2024, 3, 5)),
                               (dt.date(2024, 6, 29), dt.date(2024, 7, 8)), (dt.date(2100, 2, 23), dt.date(2100, 3, 5))):
                for include_weekends in (False, True):
                    case += 1
                    holidays = [iso(date) for date in reference.holidayList(qdate(ql, start), qdate(ql, end), include_weekends)]
                    business = [iso(date) for date in reference.businessDayList(qdate(ql, start), qdate(ql, end))]
                    expected_holidays = [date.isoformat() for date in dates_between(start, end) if not oracle.business(date) and
                                         (include_weekends or not oracle.weekend(date.isoweekday()))]
                    expected_business = [date.isoformat() for date in dates_between(start, end) if oracle.business(date)]
                    if (holidays, business) != (expected_holidays, expected_business):
                        raise ValueError("unexpected inclusive list discrepancy")
                    yield ("LIST-" + str(case).zfill(4), name, start, end, int(include_weekends), ",".join(holidays), ",".join(business))
    write("core-lists", ("id", "calendar", "start", "end", "include_weekends", "holidays", "business_days"), lists())

    def masks():
        for mask in range(128):
            reference = ql.BespokeCalendar("SabiQuant-mask-" + str(mask))
            for weekday in range(1, 8):
                if mask & (1 << (weekday - 1)):
                    reference.addWeekend(weekday % 7 + 1)
            for offset in range(14):
                date = dt.date(2000, 3, 1) + dt.timedelta(days=offset)
                result = reference.isBusinessDay(qdate(ql, date))
                if result != (not mask & (1 << (date.isoweekday() - 1))):
                    raise ValueError("unexpected weekend-mask discrepancy")
                yield ("MASK-" + str(mask).zfill(3) + "-" + str(offset).zfill(2), mask, date, int(result))
    write("weekend-masks", ("id", "weekend_mask", "date", "business"), masks())
    def bounded_divergences():
        first, origin = dt.date(2024, 2, 1), dt.date(2024, 2, 15)
        empty = ql.BespokeCalendar("SabiQuant-empty-February")
        for offset in range(29):
            empty.addHoliday(qdate(ql, first + dt.timedelta(days=offset)))
        for name, code, expected in (
                ("ModifiedFollowing", ql.ModifiedFollowing, dt.date(2024, 1, 31)),
                ("ModifiedPreceding", ql.ModifiedPreceding, dt.date(2024, 3, 1)),
                ("HalfMonthModifiedFollowing", ql.HalfMonthModifiedFollowing, dt.date(2024, 1, 31))):
            actual = iso(empty.adjust(qdate(ql, origin), code))
            if actual != expected.isoformat():
                raise ValueError("unexpected empty-month reference behavior")
            yield ("EMPTY-MONTH-" + name, origin, name, actual,
                   "calendar.no_business_day_in_month", "D9-CALENDAR-EMPTY-MONTH")
        origin = dt.date(2024, 1, 15)
        for direction, name, code in ((1, "Following", ql.Following), (-1, "Preceding", ql.Preceding)):
            reference = ql.BespokeCalendar("SabiQuant-bounded-" + name)
            for offset in range(367):
                reference.addHoliday(qdate(ql, origin + dt.timedelta(days=direction * offset)))
            actual = iso(reference.adjust(qdate(ql, origin), code))
            expected = (origin + dt.timedelta(days=direction * 367)).isoformat()
            if actual != expected:
                raise ValueError("unexpected finite long-search reference behavior")
            yield ("LONG-CLOSURE-" + name, origin, name, actual,
                   "calendar.search_limit", "D9-CALENDAR-SEARCH-BOUND")
    write("bounded-divergences", ("id", "date", "convention", "reference_result", "expected_error", "discrepancy"), bounded_divergences())
    for _, reference, _ in contexts:
        reference.resetAddedAndRemovedHolidays()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=pathlib.Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    import QuantLib as ql
    if ql.__version__ != "1.43":
        parser.error("expected installed QuantLib 1.43")
    if not args.quantlib_wheel.name.lower().startswith("quantlib-1.43-") or args.quantlib_wheel.suffix != ".whl":
        parser.error("expected downloaded QuantLib 1.43 wheel")
    wheel_bytes = args.quantlib_wheel.read_bytes()
    installed_root = pathlib.Path(ql.__file__).resolve().parent.parent
    with zipfile.ZipFile(args.quantlib_wheel) as wheel:
        members = [name for name in wheel.namelist() if name.startswith("QuantLib/") and not name.endswith("/")]
        if not members:
            parser.error("provided wheel has no QuantLib module")
        for name in members:
            if hashlib.sha256((installed_root / name).read_bytes()).digest() != hashlib.sha256(wheel.read(name)).digest():
                parser.error("installed module differs from wheel: " + name)

    def write(name, columns, records):
        metadata = {
            "schema": "1", "fixture-set": "CALENDAR-" + name.upper(), "domain": "calendar",
            "description": ("Five finite explicit empty-month/search-bound divergences" if name == "bounded-divergences" else "Synthetic core-calendar scenarios; all results independently checked; no intentional divergences"),
            "reference": "QuantLib", "reference-version": "1.43 (reported by installed Python wheel)",
            "reference-source": "QuantLib-Python exact attested wheel; Python datetime/set oracle",
            "tolerance": "TOL-E1", "reproduction": "generator-sha256=" + REVISION + "; python3 validation/generator/calendar_fixtures.py --quantlib-wheel " + args.quantlib_wheel.name,
            "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
            "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
            "quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": hashlib.sha256(wheel_bytes).hexdigest(),
            "quantlib-source-baseline": "commit 6b57206e04598f092efee66e3b367efc84771995 from docs/QUANTLIB_REFERENCE.md; wheel source commit not independently attested",
        }
        lines = ["\t".join(columns)]
        for record in records:
            fields = [str(value) for value in record]
            if len(fields) != len(columns) or any("\t" in value or "\n" in value for value in fields):
                raise ValueError("malformed generator record")
            lines.append("\t".join(fields))
        body = "\n".join(lines) + "\n"
        path = DEST / (name + ".fixture")
        if args.check:
            old_header, old_body = path.read_text().split("\n\n", 1)
            old_metadata = dict(line.split(": ", 1) for line in old_header.splitlines())
            stable_keys = metadata.keys() - {"generated-at", "platform", "python-version"}
            if old_body != body or any(old_metadata.get(key) != metadata[key] for key in stable_keys):
                raise SystemExit("fixture differs: " + str(path))
            print("verified", path.relative_to(ROOT), "records=" + str(len(lines) - 1))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            output = "\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n" + body
            path.write_bytes(output.encode("utf-8"))
            print("generated", path.relative_to(ROOT), "records=" + str(len(lines) - 1))
    run(ql, write)


if __name__ == "__main__":
    main()
