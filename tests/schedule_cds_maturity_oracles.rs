//! Exact independent candidate-enumeration oracles for quoted CDS maturities.

mod support;

use sabiquant::schedule::{
    CdsMaturityConvention, CdsMaturityOutcome, CdsMaturityResolution, ScheduleError,
    resolve_cds_maturity,
};
use sabiquant::time::{Date, DateError, Period, TimeUnit};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

const TENORS: [(i32, TimeUnit); 12] = [
    (0, TimeUnit::Months),
    (3, TimeUnit::Months),
    (6, TimeUnit::Months),
    (9, TimeUnit::Months),
    (12, TimeUnit::Months),
    (15, TimeUnit::Months),
    (18, TimeUnit::Months),
    (21, TimeUnit::Months),
    (60, TimeUnit::Months),
    (0, TimeUnit::Years),
    (1, TimeUnit::Years),
    (5, TimeUnit::Years),
];

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |value, byte| {
        (value ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn ordinal(date: Date) -> i64 {
    let prior = i64::from(date.year() - 1);
    let month_total: u32 = (1..date.month())
        .map(|month| month_days(date.year(), month))
        .sum();
    365 * prior + prior / 4 - prior / 100 + prior / 400 + i64::from(month_total + date.day() - 1)
}

fn stored_tenor(actual: Period, expected: Period) {
    assert_eq!(actual.length(), expected.length());
    assert_eq!(actual.unit(), expected.unit());
}

fn metadata(
    resolution: CdsMaturityResolution,
    trade: Date,
    tenor: Period,
    convention: CdsMaturityConvention,
) {
    assert_eq!(resolution.trade_date(), trade);
    stored_tenor(resolution.quoted_tenor(), tenor);
    assert_eq!(resolution.convention(), convention);
}

fn formatted_counts(counts: &BTreeMap<String, usize>) -> String {
    counts
        .iter()
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>()
        .join(";")
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports measured exact oracle distributions"
)]
fn every_gregorian_cycle_day_both_regimes_stored_tenors_and_exact_failure_causes()
-> Result<(), Box<dyn Error>> {
    let text = include_str!("../validation/fixtures/schedule/cds-maturity-oracle.fixture");
    let (_, body) = text.split_once("\n\n").ok_or("missing fixture separator")?;
    assert_eq!(fnv64(body.as_bytes()), 0x5cf8_2c86_8d25_43d0);
    // These non-cryptographic integrity checks detect changed retained inputs;
    // SHA-256 provenance is separately recorded in the header and report.
    assert_eq!(
        fnv64(include_bytes!(
            "../validation/generator/cds_maturity_oracles.py"
        )),
        0x0e01_a8aa_f65a_c12d
    );
    assert_eq!(
        fnv64(include_bytes!("../docs/M7_CDS_MATURITY_DESIGN.md")),
        0xb351_6ad6_507e_682f
    );
    assert_eq!(
        fnv64(include_bytes!("../docs/M7_CDS_CONTRACT.md")),
        0xbf60_b7d6_7079_e618
    );
    let fixture = Fixture::parse(text)?;
    assert_eq!(fixture.record_count(), 148_688);
    for (key, value) in [
        ("fixture-set", "CDS-MATURITY-CIVIL-ORACLE"),
        (
            "reference",
            "oracle:Python datetime and abstract quarter-candidate enumeration",
        ),
        (
            "tolerance",
            "TOL-E0 exact outcomes, dates, stored inputs and typed error causes",
        ),
        ("record-count", "148688"),
        ("regime-comparison-count", "297376"),
        ("unique-input-count", "148512"),
        (
            "historical-design-sha256",
            "4c221a428544d0fbd54736505331689c2bcf33405b3973a666ab94c90b64a2d8",
        ),
        (
            "adopted-contract-sha256",
            "312df1e25a22c2f2575c30fd30853d315258ac3844ac0ab386d42a1db9c5aad5",
        ),
        (
            "body-sha256",
            "83dfee265e35923f8a836a46b2e9063b789ec0705889de2be24d3ec3a28b7d1f",
        ),
        ("body-fnv64", "5cf82c868d2543d0"),
    ] {
        assert_eq!(fixture.header(key), Some(value), "header {key}");
    }
    assert_eq!(
        fixture.header("reproduction"),
        Some(
            "generator-sha256=a1705d627d34facca566cdf0c1e0d9d6ba6c5c9c51f7dc19cb070e8b24d774f0; python3 validation/generator/cds_maturity_oracles.py"
        )
    );
    let mut profiles = BTreeMap::<String, usize>::new();
    let mut statuses = [BTreeMap::<String, usize>::new(), BTreeMap::new()];
    let mut unique_inputs = BTreeSet::new();
    let mut boundary_inputs = BTreeSet::new();
    let mut cycle_days = 0;
    let mut aliases = 0;
    let cycle_first = ordinal(Date::from_ymd(2000, 1, 1)?);
    for row in fixture.records() {
        let trade: Date = row.parse("trade")?;
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let profile = row.text("profile")?;
        *profiles.entry(profile.to_owned()).or_default() += 1;
        let key = (trade, tenor.length(), row.text("unit")?.to_owned());
        unique_inputs.insert(key.clone());
        if profile == "daily-cycle" {
            assert_eq!(ordinal(trade), cycle_first + i64::try_from(cycle_days)?);
            let (length, unit) = TENORS[cycle_days % TENORS.len()];
            stored_tenor(tenor, Period::new(length, unit));
            cycle_days += 1;
        } else if profile == "quarter-boundaries" {
            assert!(boundary_inputs.insert(key));
        }
        for (regime_index, prefix, convention) in [
            (0, "quarterly", CdsMaturityConvention::Quarterly2009),
            (1, "semiannual", CdsMaturityConvention::Semiannual2015),
        ] {
            let status = row.text(&format!("{prefix}_status"))?;
            let expected_date = row.parse_optional::<Date>(&format!("{prefix}_date"))?;
            let expected_offset = row.parse_optional::<i64>(&format!("{prefix}_error_months"))?;
            *statuses[regime_index].entry(status.to_owned()).or_default() += 1;
            let result = resolve_cds_maturity(trade, tenor, convention);
            let resolution = match status {
                "invalid_tenor" => {
                    let error = result.unwrap_err();
                    assert_eq!(error.kind(), "schedule.invalid_cds_tenor", "{}", row.id());
                    let ScheduleError::InvalidCdsTenor {
                        trade_date,
                        tenor: actual,
                        convention: actual_convention,
                        ..
                    } = error
                    else {
                        panic!("{} wrong typed domain error: {error:?}", row.id());
                    };
                    assert_eq!(trade_date, trade);
                    stored_tenor(actual, tenor);
                    assert_eq!(actual_convention, convention);
                    assert!(expected_date.is_none() && expected_offset.is_none());
                    continue;
                }
                "date_error" => {
                    let error = result.unwrap_err();
                    assert_eq!(error.kind(), "schedule.cds_maturity_date", "{}", row.id());
                    assert!(error.source().is_some());
                    let ScheduleError::CdsMaturityDate {
                        trade_date,
                        tenor: actual,
                        convention: actual_convention,
                        cause,
                        ..
                    } = error
                    else {
                        panic!("{} wrong typed date error: {error:?}", row.id());
                    };
                    assert_eq!(trade_date, trade);
                    stored_tenor(actual, tenor);
                    assert_eq!(actual_convention, convention);
                    let DateError::OutOfRange {
                        date,
                        operation,
                        count,
                        ..
                    } = cause
                    else {
                        panic!("{} wrong date cause: {cause:?}", row.id());
                    };
                    assert_eq!(date, trade);
                    assert_eq!(operation, "add months");
                    assert_eq!(Some(count), expected_offset, "{} {convention}", row.id());
                    assert!(expected_date.is_none());
                    continue;
                }
                "date" | "matured" => result.unwrap_or_else(|error| {
                    panic!("{} {convention} unexpected failure: {error:?}", row.id())
                }),
                other => panic!("{} unknown status {other}", row.id()),
            };
            metadata(resolution, trade, tenor, convention);
            assert!(expected_offset.is_none());
            if status == "date" {
                let date = expected_date.ok_or("date status without date")?;
                assert_eq!(
                    resolution.outcome(),
                    CdsMaturityOutcome::Date(date),
                    "{} {convention}",
                    row.id()
                );
                assert!(date > trade);
                assert_eq!(date.day(), 20);
                assert!([3, 6, 9, 12].contains(&date.month()));
            } else {
                assert_eq!(
                    resolution.outcome(),
                    CdsMaturityOutcome::Matured,
                    "{}",
                    row.id()
                );
                assert_eq!(convention, CdsMaturityConvention::Semiannual2015);
                assert_eq!(tenor.length(), 0);
                assert!(expected_date.is_none());
            }
            if profile == "quarter-boundaries" {
                let alias = match tenor.unit() {
                    TimeUnit::Years => Some(Period::new(tenor.length() * 12, TimeUnit::Months)),
                    TimeUnit::Months if tenor.length() % 12 == 0 => {
                        Some(Period::new(tenor.length() / 12, TimeUnit::Years))
                    }
                    _ => None,
                };
                if let Some(alias) = alias {
                    let aliased = resolve_cds_maturity(trade, alias, convention)?;
                    metadata(aliased, trade, alias, convention);
                    assert_eq!(resolution.outcome(), aliased.outcome());
                    assert_ne!(
                        resolution, aliased,
                        "stored aliases preserve definition identity"
                    );
                    aliases += 1;
                }
            }
        }
    }
    assert_eq!(cycle_days, 146_097);
    assert_eq!(unique_inputs.len(), 148_512);
    assert_eq!(boundary_inputs.len(), 2_304);
    assert_eq!(aliases, 2_154);
    for year in [
        1, 4, 99, 100, 400, 1900, 2000, 2001, 2015, 2016, 2099, 2100, 2399, 2400, 9998, 9999,
    ] {
        for month in [3, 6, 9, 12] {
            for day in [19, 20, 21] {
                for (length, unit) in TENORS {
                    assert!(boundary_inputs.contains(&(
                        Date::from_ymd(year, month, day)?,
                        length,
                        unit.to_string()
                    )));
                }
            }
        }
    }
    let expected_profiles = "civil-extremes=120;daily-cycle=146097;huge-valid-tenor=8;long-civil-edge=15;quarter-boundaries=2304;tenor-domain=144";
    let expected_statuses = [
        "date=148328;date_error=256;invalid_tenor=104",
        "date=135981;date_error=226;invalid_tenor=104;matured=12377",
    ];
    assert_eq!(formatted_counts(&profiles), expected_profiles);
    assert_eq!(fixture.header("profile-counts"), Some(expected_profiles));
    for (index, prefix) in ["quarterly", "semiannual"].into_iter().enumerate() {
        assert_eq!(formatted_counts(&statuses[index]), expected_statuses[index]);
        assert_eq!(
            fixture.header(&format!("{prefix}-status-counts")),
            Some(expected_statuses[index])
        );
    }
    println!(
        "CDS maturity oracle: records={} comparisons={} cycle_days={cycle_days} unique_inputs={} statuses={statuses:?} stored_alias_checks={aliases}",
        fixture.record_count(),
        2 * fixture.record_count(),
        unique_inputs.len()
    );
    Ok(())
}
