#!/usr/bin/env python3
"""Independent quarter-candidate enumeration for explicit CDS maturity regimes."""

import argparse
from collections import Counter
import datetime as dt
from functools import lru_cache
import hashlib
from pathlib import Path
import platform


ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/schedule/cds-maturity-oracle.fixture"
DESIGN = ROOT / "docs/M7_CDS_MATURITY_DESIGN.md"
CONTRACT = ROOT / "docs/M7_CDS_CONTRACT.md"
TENORS = tuple((n, "Months") for n in (0, 3, 6, 9, 12, 15, 18, 21, 60)) + (
    (0, "Years"), (1, "Years"), (5, "Years"))
BOUNDARY_YEARS = (1, 4, 99, 100, 400, 1900, 2000, 2001, 2015, 2016,
                  2099, 2100, 2399, 2400, 9998, 9999)
COLUMNS = ("id profile trade length unit quarterly_status quarterly_date "
           "quarterly_error_months semiannual_status semiannual_date "
           "semiannual_error_months").split()


def abstract(date):
    return date.year, date.month, date.day


@lru_cache(maxsize=None)
def quarter_candidates(year):
    # Tuple dates deliberately permit the unused year-zero anchor and a
    # required year-10000 maturity. Python datetime is constructed only for
    # the final representable outcome, never for a historical anchor.
    return tuple((y, month, 20) for y in (year - 1, year, year + 1)
                 for month in (3, 6, 9, 12))


def first_quarter_after(point):
    return min(candidate for candidate in quarter_candidates(point[0]) if candidate > point)


def quoted_target(base, months):
    # Remove whole years from the quote, then enumerate and index the remaining
    # quarter points. This is not production's cutoff/remainder anchor formula,
    # nor a repeated Date month-addition algorithm. Large quotes remain bounded.
    whole_years, remainder = divmod(months, 12)
    shifted = (base[0] + whole_years, base[1], 20)
    candidates = [candidate for candidate in quarter_candidates(shifted[0])
                  if candidate >= shifted]
    return candidates[remainder // 3]


def outcome(trade, length, unit, regime):
    if length < 0 or unit not in ("Months", "Years") or (unit == "Months" and length % 3):
        return "invalid_tenor", "", ""
    months = length * (12 if unit == "Years" else 1)
    trade_tuple = abstract(trade)
    if regime == "quarterly":
        base = first_quarter_after(trade_tuple)
    else:
        anchor = max(candidate for candidate in quarter_candidates(trade.year)
                     if candidate[1] in (3, 9) and candidate <= trade_tuple)
        base = first_quarter_after(anchor)
        if months == 0 and base <= trade_tuple:
            return "matured", "", ""
    year, month, day = quoted_target(base, months)
    if not 1 <= year <= 9999:
        displacement = (year - trade.year) * 12 + month - trade.month
        return "date_error", "", str(displacement)
    result = dt.date(year, month, day)
    assert result > trade and result.day == 20 and result.month in (3, 6, 9, 12)
    return "date", result.isoformat(), ""


def definitions():
    start = dt.date(2000, 1, 1)
    for offset in range((dt.date(2400, 1, 1) - start).days):
        yield "daily-cycle", start + dt.timedelta(days=offset), *TENORS[offset % len(TENORS)]
    for year in BOUNDARY_YEARS:
        for month in (3, 6, 9, 12):
            for day in (19, 20, 21):
                for length, unit in TENORS:
                    yield "quarter-boundaries", dt.date(year, month, day), length, unit
    extremes = (dt.date.min, dt.date(1, 1, 31), dt.date(1, 2, 28),
                dt.date(1, 3, 19), dt.date(1, 3, 20), dt.date(9999, 6, 20),
                dt.date(9999, 9, 19), dt.date(9999, 12, 19),
                dt.date(9999, 12, 20), dt.date.max)
    for trade in extremes:
        for length, unit in TENORS:
            yield "civil-extremes", trade, length, unit
    for trade in (dt.date.min, dt.date(2016, 6, 20), dt.date(9999, 12, 20), dt.date.max):
        for unit in ("Days", "Weeks", "Months", "Years"):
            for length in (-2**31, -1, 0, 1, 2, 3, 4, 2**31 - 2, 2**31 - 1):
                yield "tenor-domain", trade, length, unit
    for trade in (dt.date.min, dt.date(2000, 2, 29), dt.date(2100, 2, 28), dt.date.max):
        for length, unit in ((2**31 - 2, "Months"), (2**31 - 1, "Years")):
            yield "huge-valid-tenor", trade, length, unit
    # Valid long-tenor edges distinguish a required final date from an invalid
    # or unnecessary intermediate anchor, including exactly reaching year9999.
    for trade in (dt.date.min, dt.date(1, 3, 19), dt.date(1, 3, 20)):
        for length, unit in ((119_976, "Months"), (119_979, "Months"),
                             (119_988, "Months"), (9_998, "Years"), (9_999, "Years")):
            yield "long-civil-edge", trade, length, unit


def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def render(stamp):
    rows = []
    for serial, (profile, trade, length, unit) in enumerate(definitions()):
        row = dict(id=f"CDS-MATURITY-{serial:06d}", profile=profile,
                   trade=trade.isoformat(), length=length, unit=unit)
        for regime in ("quarterly", "semiannual"):
            status, date, error = outcome(trade, length, unit, regime)
            row.update({regime + "_status": status, regime + "_date": date,
                        regime + "_error_months": error})
        rows.append(row)
    format_counts = lambda counts: ";".join(k + "=" + str(v) for k, v in sorted(counts.items()))
    body = "\t".join(COLUMNS) + "\n" + "\n".join(
        "\t".join(str(row[column]) for column in COLUMNS) for row in rows) + "\n"
    body_bytes = body.encode("utf-8")
    header = {
        "schema": "1", "fixture-set": "CDS-MATURITY-CIVIL-ORACLE", "domain": "schedule.cds-maturity",
        "description": "Both explicit maturity regimes for every day of Gregorian400-year cycle; rotating stored tenors, exact roll boundaries and civil/tenor extremes",
        "reference": "oracle:Python datetime and abstract quarter-candidate enumeration",
        "reference-version": "Python " + platform.python_version(),
        "reference-source": "surrounding-year quarter tuples; min/max candidate selection; quoted-quarter indexing; no Rust or QuantLib outputs",
        "tolerance": "TOL-E0 exact outcomes, dates, stored inputs and typed error causes",
        "reproduction": "generator-sha256=" + hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
                        + "; python3 validation/generator/cds_maturity_oracles.py",
        "historical-design-sha256": hashlib.sha256(DESIGN.read_bytes()).hexdigest(),
        "adopted-contract-sha256": hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),
        "generated-at": stamp, "platform": platform.system() + " " + platform.machine(),
        "scope": "Unadjusted explicit quoted-maturity mathematics only; no calendar, premium-leg, protection-date or historical market acceptance",
        "cycle": "2000-01-01..2399-12-31 all146097 days; both regimes each day; twelve rotating stored tenors, not a full tenor Cartesian product",
        "record-count": str(len(rows)), "regime-comparison-count": str(2 * len(rows)),
        "unique-input-count": str(len({(row["trade"], row["length"], row["unit"]) for row in rows})),
        "profile-counts": format_counts(Counter(row["profile"] for row in rows)),
        "quarterly-status-counts": format_counts(Counter(row["quarterly_status"] for row in rows)),
        "semiannual-status-counts": format_counts(Counter(row["semiannual_status"] for row in rows)),
        "body-sha256": hashlib.sha256(body_bytes).hexdigest(), "body-fnv64": fnv64(body_bytes),
    }
    return "\n".join(key + ": " + value for key, value in header.items()) + "\n\n" + body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        old_header_text, old_body = DEST.read_text().split("\n\n", 1)
        old_header = dict(line.split(": ", 1) for line in old_header_text.splitlines())
        text = render(old_header["generated-at"])
        new_header_text, new_body = text.split("\n\n", 1)
        new_header = dict(line.split(": ", 1) for line in new_header_text.splitlines())
        # Environment labels describe collection. Exact arithmetic and source
        # pins remain portable across Python/platform versions.
        ignored = {"platform", "reference-version"}
        if old_body != new_body or ({k: v for k, v in old_header.items() if k not in ignored}
                                   != {k: v for k, v in new_header.items() if k not in ignored}):
            raise SystemExit("CDS maturity oracle stale; regenerate and review")
        action = "verified"
    else:
        text = render(dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_text(text)
        action = "generated"
    print(action, DEST.relative_to(ROOT), next(line for line in text.splitlines()
                                             if line.startswith("record-count: ")))


if __name__ == "__main__":
    main()
