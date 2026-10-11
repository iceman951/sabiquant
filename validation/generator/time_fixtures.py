#!/usr/bin/env python3
"""Generate schema-1 M2 fixtures from independent public interfaces.

The oracle uses Python's datetime/calendar standard library, not the Rust
conversion algorithm. QuantLib is imported only for the explicitly requested
differential suite, and must be exactly version 1.43. No source is translated.
"""

import argparse
import calendar
import datetime as dt
import hashlib
import pathlib
import platform
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation" / "fixtures" / "time"
ANCHOR_YEARS = (
    1, 4, 99, 100, 399, 400, 1582, 1600, 1700, 1800, 1899, 1900,
    1901, 1999, 2000, 2001, 2099, 2100, 2199, 2399, 2400, 9996, 9999,
)
UNIT_NAMES = ("Days", "Weeks", "Months", "Years")
FREQUENCY_NAMES = (
    "Annual", "Semiannual", "EveryFourthMonth", "Quarterly", "Bimonthly",
    "Monthly", "EveryFourthWeek", "Biweekly", "Weekly", "Daily", "Once",
)
FREQUENCY_PERIODS = (
    (1, "Years"), (6, "Months"), (4, "Months"), (3, "Months"),
    (2, "Months"), (1, "Months"), (4, "Weeks"), (2, "Weeks"),
    (1, "Weeks"), (1, "Days"), None,
)
GENERATOR_REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
WHEEL_METADATA = {}


def sample_ordinals(count, lower=1, upper=dt.date.max.toordinal()):
    """Fixed SplitMix64 sequence; independent of random-module versions."""
    state = 0x534142495155414E
    mask = (1 << 64) - 1
    for _ in range(count):
        state = (state + 0x9E3779B97F4A7C15) & mask
        value = state
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & mask
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & mask
        value ^= value >> 31
        yield lower + value % (upper - lower + 1)


def fixture(name, reference, columns, records, description, check, version=None):
    """Write exact integer/text data with truthful environment provenance."""
    source = "Python datetime/calendar standard library"
    if reference == "QuantLib":
        source = "QuantLib-Python 1.43 wheel from PyPI; datetime for stated D9 cases"
    headers = {
        "schema": "1",
        "fixture-set": name,
        "domain": "date" if "DATE" in name else "period",
        "description": description,
        "reference": reference,
        "reference-version": version or "Python " + platform.python_version() + " (exact integers)",
        "reference-source": source,
        "tolerance": "TOL-E1",
        "reproduction": "generator-sha256=" + GENERATOR_REVISION + "; python3 validation/generator/time_fixtures.py --reference " + ("quantlib --quantlib-wheel " + WHEEL_METADATA.get("quantlib-wheel", "") if reference == "QuantLib" else "oracle"),
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(),
        "python-version": platform.python_version(),
    }
    if reference == "QuantLib":
        headers.update(WHEEL_METADATA)
        headers["quantlib-source-baseline"] = "commit 6b57206e04598f092efee66e3b367efc84771995 from docs/QUANTLIB_REFERENCE.md; wheel source commit not independently attested"
    body = "\t".join(columns) + "\n"
    for record in records:
        fields = [str(field) for field in record]
        if len(fields) != len(columns) or any("\t" in field or "\n" in field for field in fields):
            raise ValueError("malformed generator record: " + repr(record))
        body += "\t".join(fields) + "\n"
    output = "\n".join(key + ": " + value for key, value in headers.items()) + "\n\n" + body
    path = DEST / (name.lower() + ".fixture")
    if check:
        # Metadata describes the generation that actually wrote the file.
        # Environmental differences do not invalidate exact integer data.
        old_header, old_body = path.read_text(encoding="utf-8").split("\n\n", 1)
        old_metadata = dict(line.split(": ", 1) for line in old_header.splitlines())
        stable_keys = ("schema", "fixture-set", "domain", "description", "reference", "tolerance", "reproduction")
        if reference == "QuantLib":
            stable_keys += ("quantlib-wheel", "quantlib-wheel-sha256", "quantlib-source-baseline")
        if old_body != body or any(old_metadata.get(key) != headers[key] for key in stable_keys):
            raise SystemExit("fixture differs: " + str(path.relative_to(ROOT)))
        print("verified", path.relative_to(ROOT), "records=" + str(body.count("\n") - 1))
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(output.encode("utf-8"))
        print("generated", path.relative_to(ROOT), "records=" + str(body.count("\n") - 1))


def oracle_dates():
    dates = {dt.date.fromordinal(ordinal) for ordinal in sample_ordinals(1024)}
    for year in range(1, 10000):
        dates.update((dt.date(year, 2, 28), dt.date(year, 3, 1)))
    for year in ANCHOR_YEARS:
        first = dt.date(year, 1, 1).toordinal()
        last = dt.date(year, 12, 31).toordinal()
        dates.update(dt.date.fromordinal(ordinal) for ordinal in range(first, last + 1))
    for date in sorted(dates):
        month_length = calendar.monthrange(date.year, date.month)[1]
        yield (
            "DATE-ORACLE-" + date.isoformat(), date.isoformat(), date.year,
            date.month, date.day, date.isoweekday(), date.timetuple().tm_yday,
            date.toordinal(), month_length, 366 if calendar.isleap(date.year) else 365,
            date.replace(day=1).isoformat(), date.replace(day=month_length).isoformat(),
            int(date.day == month_length),
        )


def shifted(date, unit, count):
    """Independent month clamping using datetime constructors and calendar."""
    try:
        if unit in ("Days", "Weeks"):
            return date + dt.timedelta(days=count * (7 if unit == "Weeks" else 1))
        target = (date.year - 1) * 12 + date.month - 1 + count * (12 if unit == "Years" else 1)
        year, month = divmod(target, 12)
        year, month = year + 1, month + 1
        if not 1 <= year <= 9999:
            return None
        return dt.date(year, month, min(date.day, calendar.monthrange(year, month)[1]))
    except (OverflowError, ValueError):
        return None


def arithmetic_dates(quantlib=False):
    years = (1901, 1999, 2000, 2001, 2099, 2100, 2199) if quantlib else ANCHOR_YEARS
    dates = set()
    for year in years:
        for month, day in ((1, 1), (1, 31), (2, 28), (3, 31), (12, 31)):
            dates.add(dt.date(year, month, day))
        if calendar.isleap(year):
            dates.add(dt.date(year, 2, 29))
    lower, upper = (dt.date(1901, 1, 1).toordinal(), dt.date(2199, 12, 31).toordinal()) if quantlib else (1, dt.date.max.toordinal())
    dates.update(dt.date.fromordinal(ordinal) for ordinal in sample_ordinals(64, lower, upper))
    return sorted(dates)


def oracle_arithmetic():
    counts = (-(1 << 63), -9999, -400, -13, -12, -7, -1, 0, 1, 7, 12, 13, 400, 9999, (1 << 63) - 1)
    case = 0
    for date in arithmetic_dates():
        for unit in UNIT_NAMES:
            for count in counts:
                case += 1
                result = shifted(date, unit, count)
                yield ("DATE-ARITH-" + str(case).zfill(6), date.isoformat(), unit, count,
                       result.isoformat() if result else "", "" if result else "date.out_of_range")


def normalized(length, unit):
    if length == 0:
        return 0, "Days"
    divisor, target = {"Days": (7, "Weeks"), "Months": (12, "Years")}.get(unit, (1, unit))
    return (length // divisor, target) if length % divisor == 0 else (length, unit)


def expected_frequency(length, unit):
    if length <= 0:
        return ""
    period = normalized(length, unit)
    for name, candidate in zip(FREQUENCY_NAMES, FREQUENCY_PERIODS):
        if candidate and normalized(*candidate) == period:
            return name
    return ""


def canonical_period(length, unit):
    if length == 0:
        return "day", 0
    family, factor = {"Days": ("day", 1), "Weeks": ("day", 7), "Months": ("month", 1), "Years": ("month", 12)}[unit]
    return family, length * factor


def expected_comparison(left, right):
    left_family, left_value = canonical_period(*left)
    right_family, right_value = canonical_period(*right)
    if left_family != right_family:
        return "incomparable"
    return "less" if left_value < right_value else "greater" if left_value > right_value else "equal"


def quantlib_fixtures(check, wheel_path):
    try:
        import QuantLib as ql
    except ImportError as error:
        raise SystemExit("QuantLib 1.43 is required only for --reference quantlib/all; install it in an external venv") from error
    if ql.__version__ != "1.43":
        raise SystemExit("expected QuantLib 1.43, found " + ql.__version__)
    # Attest the installed module bytes, including the native binary, against
    # the exact wheel whose digest is recorded. This does not inspect or use
    # QuantLib implementation source to derive financial algorithms.
    installed_root = pathlib.Path(ql.__file__).resolve().parent.parent
    with zipfile.ZipFile(wheel_path) as wheel:
        members = [member for member in wheel.namelist() if member.startswith("QuantLib/") and not member.endswith("/")]
        if not members:
            raise SystemExit("provided wheel has no QuantLib module files")
        for member in members:
            installed = installed_root / member
            if not installed.is_file() or hashlib.sha256(installed.read_bytes()).digest() != hashlib.sha256(wheel.read(member)).digest():
                raise SystemExit("installed QuantLib differs from provided wheel: " + member)
    version = "1.43 (reported by installed Python wheel)"
    units = {name: getattr(ql, name) for name in UNIT_NAMES}
    unit_names = {value: name for name, value in units.items()}
    frequency_names = {getattr(ql, name): name for name in FREQUENCY_NAMES}

    def date_records():
        # Every day in QuantLib's full supported range, independently enumerated.
        first, last = dt.date(1901, 1, 1).toordinal(), dt.date(2199, 12, 31).toordinal()
        for ordinal in range(first, last + 1):
            date = dt.date.fromordinal(ordinal)
            reference = ql.Date(date.day, date.month, date.year)
            end = ql.Date.endOfMonth(reference)
            yield ("DATE-QL-" + date.isoformat(), date.isoformat(),
                   (reference.weekday() + 5) % 7 + 1, reference.dayOfYear(),
                   end.dayOfMonth(), int(ql.Date.isEndOfMonth(reference)))

    fixture("TIME-DATE-QUANTLIB", "QuantLib", ("id", "date", "weekday", "day_of_year", "month_length", "is_end_of_month"),
            date_records(), "Public Date accessors for all 109208 days in QuantLib range 1901-2199", check, version)

    def arithmetic_records():
        case = 0
        for date in arithmetic_dates(quantlib=True):
            reference = ql.Date(date.day, date.month, date.year)
            for unit in UNIT_NAMES:
                for count in (-3600, -400, -13, -12, -7, -1, 0, 1, 7, 12, 13, 400, 3600):
                    case += 1
                    expected = shifted(date, unit, count)
                    try:
                        result = reference + ql.Period(count, units[unit])
                        ql_result = dt.date(result.year(), int(result.month()), result.dayOfMonth()).isoformat()
                        behavior = "value"
                    except RuntimeError:
                        ql_result, behavior = "", "raises"
                    result = expected.isoformat() if expected else ""
                    discrepancy = "D9-DATE-RANGE" if behavior == "raises" and expected else ""
                    if discrepancy and 1901 <= expected.year <= 2199:
                        raise ValueError("QuantLib unexpectedly rejected an in-range date shift")
                    if behavior == "value" and ql_result != result:
                        raise ValueError("unexpected date discrepancy " + repr((date, unit, count, ql_result, result)))
                    yield ("DATE-QL-ARITH-" + str(case).zfill(6), date.isoformat(), unit, count,
                           ql_result, result, behavior, "" if expected else "date.out_of_range", discrepancy)

    fixture("TIME-DATE-QUANTLIB-ARITHMETIC", "QuantLib",
            ("id", "date", "unit", "count", "reference_result", "expected_result", "reference_behavior", "expected_error", "discrepancy"),
            arithmetic_records(), "Checked date shifts with independent datetime resolution of wider-range D9 cases", check, version)

    def period_records():
        allowed_frequency_differences = {
            (-12, "Months"): ("Annual", "", 1),
            (-6, "Months"): ("Semiannual", "", 2),
            (-4, "Weeks"): ("EveryFourthWeek", "", 13),
            (-4, "Months"): ("EveryFourthMonth", "", 3),
            (-3, "Months"): ("Quarterly", "", 4),
            (-2, "Weeks"): ("Biweekly", "", 26),
            (-2, "Months"): ("Bimonthly", "", 6),
            (-1, "Days"): ("Daily", "", 365),
            (-1, "Weeks"): ("Weekly", "", 52),
            (-1, "Months"): ("Monthly", "", 12),
            (-1, "Years"): ("Annual", "", 1),
            (0, "Years"): ("Once", "", 0),
            (7, "Days"): ("", "Weekly", 999),
            (14, "Days"): ("", "Biweekly", 999),
            (28, "Days"): ("", "EveryFourthWeek", 999),
        }
        observed_differences = set()
        for length in range(-120, 121):
            for unit in UNIT_NAMES:
                reference = ql.Period(length, units[unit])
                norm = reference.normalized()
                frequency_code = reference.frequency()
                ql_frequency = frequency_names.get(frequency_code, "")
                expected_length, expected_unit = normalized(length, unit)
                expected = expected_frequency(length, unit)
                differences = []
                if (norm.length(), unit_names[norm.units()]) != (expected_length, expected_unit):
                    raise ValueError("unexpected period normalization discrepancy")
                if ql_frequency != expected:
                    observed = (ql_frequency, expected, frequency_code)
                    if allowed_frequency_differences.get((length, unit)) != observed:
                        raise ValueError("unexpected period-to-frequency discrepancy")
                    observed_differences.add((length, unit))
                    differences.append("D9-PERIOD-FREQUENCY")
                yield ("PER-QL-" + unit + "-" + str(length), length, unit,
                       norm.length(), unit_names[norm.units()], frequency_code, ql_frequency,
                       expected_length, expected_unit, expected, ",".join(differences))
        if observed_differences != set(allowed_frequency_differences):
            raise ValueError("the pinned reference frequency discrepancy set changed")

    fixture("TIME-PERIOD-QUANTLIB", "QuantLib",
            ("id", "length", "unit", "reference_normalized_length", "reference_normalized_unit", "reference_frequency_code", "reference_frequency", "normalized_length", "normalized_unit", "frequency", "discrepancy"),
            period_records(), "Period normalization and frequency mappings over -120..120 and all four units", check, version)

    def frequency_records():
        for name in FREQUENCY_NAMES:
            code = getattr(ql, name)
            reference = ql.Period(code)
            candidate = FREQUENCY_PERIODS[FREQUENCY_NAMES.index(name)]
            observed = (reference.length(), unit_names[reference.units()])
            if candidate is not None and observed != candidate:
                raise ValueError("QuantLib frequency-to-period disagrees with specified row: " + name)
            if candidate is None and (observed != (0, "Years") or code != 0):
                raise ValueError("QuantLib Once sentinel differs from documented D9 case")
            yield ("FREQ-QL-" + name, name, code, reference.length(), unit_names[reference.units()],
                   candidate[0] if candidate else "", candidate[1] if candidate else "", "D9-ONCE-PERIOD" if candidate is None else "")

    fixture("TIME-PERIOD-QUANTLIB-FREQUENCIES", "QuantLib",
            ("id", "frequency", "reference_events_per_year", "reference_length", "reference_unit", "length", "unit", "discrepancy"),
            frequency_records(), "Every named frequency; Once has no SabiQuant period and QL's zero-year sentinel is documented", check, version)

    def comparison_records():
        lengths = (-24, -14, -12, -7, -2, -1, 0, 1, 2, 5, 7, 12, 14, 24, 30, 31)
        periods = [(length, unit) for length in lengths for unit in UNIT_NAMES]
        case = 0
        for left in periods:
            for right in periods:
                case += 1
                a, b = ql.Period(left[0], units[left[1]]), ql.Period(right[0], units[right[1]])
                try:
                    observed = "equal" if a == b else "less" if a < b else "greater"
                except RuntimeError:
                    observed = "raises"
                expected = expected_comparison(left, right)
                discrepancy = "D9-PERIOD-ORDER" if observed != expected else ""
                if canonical_period(*left)[0] == canonical_period(*right)[0] and observed != expected:
                    raise ValueError("unexpected same-family comparison discrepancy")
                # QuantLib's ambiguous equality throws; compare independently in Rust.
                yield ("PER-QL-CMP-" + str(case).zfill(5), *left, *right, observed, expected, discrepancy)

    fixture("TIME-PERIOD-QUANTLIB-COMPARISONS", "QuantLib",
            ("id", "left_length", "left_unit", "right_length", "right_unit", "reference_comparison", "comparison", "discrepancy"),
            comparison_records(), "4096 ordered period pairs; strict family ordering records every D9 divergence", check, version)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", choices=("oracle", "quantlib", "all"), default="oracle")
    parser.add_argument("--check", action="store_true", help="Compare generated exact data and stable metadata without rewriting files")
    parser.add_argument("--quantlib-wheel", type=pathlib.Path, help="Exact downloaded QuantLib 1.43 wheel, hashed into reference provenance")
    args = parser.parse_args()
    if args.reference in ("quantlib", "all"):
        if args.quantlib_wheel is None or not args.quantlib_wheel.name.lower().startswith("quantlib-1.43-") or args.quantlib_wheel.suffix != ".whl":
            parser.error("--quantlib-wheel must name the downloaded QuantLib 1.43 wheel")
        WHEEL_METADATA.update({"quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": hashlib.sha256(args.quantlib_wheel.read_bytes()).hexdigest()})
    if args.reference in ("oracle", "all"):
        fixture("TIME-DATE-ORACLE", "oracle:Python datetime/calendar",
                ("id", "date", "year", "month", "day", "weekday", "day_of_year", "ordinal", "month_length", "year_length", "first_of_month", "last_of_month", "is_end_of_month"),
                oracle_dates(), "Every year's leap boundary, every day of 23 boundary years, and 1024 deterministic ordinals", args.check)
        fixture("TIME-DATE-ORACLE-ARITHMETIC", "oracle:Python datetime/calendar",
                ("id", "date", "unit", "count", "expected_result", "expected_error"),
                oracle_arithmetic(), "Month/year clamping and day/week shifts including signed-64-bit extremes", args.check)
    if args.reference in ("quantlib", "all"):
        quantlib_fixtures(args.check, args.quantlib_wheel)


if __name__ == "__main__":
    main()
