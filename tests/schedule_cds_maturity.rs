//! Independent quoted CDS maturity tables, civil arithmetic and frozen observations.

mod support;

use sabiquant::schedule::{
    CdsMaturityConvention, CdsMaturityOutcome, CdsMaturityResolution, ScheduleError,
    resolve_cds_maturity,
};
use sabiquant::time::{Date, DateError, Period, TimeUnit};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use support::fixture::Fixture;

type TestResult = Result<(), Box<dyn Error>>;
use CdsMaturityConvention::{Quarterly2009, Semiannual2015};
use CdsMaturityOutcome::{Date as Maturity, Matured};

fn months(length: i32) -> Period {
    Period::new(length, TimeUnit::Months)
}

fn years(length: i32) -> Period {
    Period::new(length, TimeUnit::Years)
}

// Select actual civil candidate tuples by ordering them. This oracle does not
// use the production modulo/coordinate selection or an inferred regime.
fn enumerated_outcome(
    trade: Date,
    quoted_months: i32,
    convention: CdsMaturityConvention,
) -> Result<CdsMaturityOutcome, Box<dyn Error>> {
    let (year, month, day) = trade.to_ymd();
    let candidates = (year - 1..=year + 1)
        .flat_map(|candidate_year| [3, 6, 9, 12].map(|m| (candidate_year, m, 20)));
    let (base_year, base_month, _) = match convention {
        Quarterly2009 => candidates
            .filter(|candidate| *candidate > (year, month, day))
            .min()
            .ok_or("no following quarter in independent candidates")?,
        Semiannual2015 => {
            let (anchor_year, anchor_month, _) = candidates
                .filter(|candidate| {
                    matches!(candidate.1, 3 | 9) && *candidate <= (year, month, day)
                })
                .max()
                .ok_or("no preceding semiannual roll in independent candidates")?;
            (anchor_year, anchor_month + 3, 20)
        }
    };
    if quoted_months == 0 && (base_year, base_month, 20) <= (year, month, day) {
        return Ok(Matured);
    }
    let shifted_month = i64::from(base_month) - 1 + i64::from(quoted_months);
    Ok(Maturity(Date::from_ymd(
        base_year + i32::try_from(shifted_month / 12)?,
        u32::try_from(shifted_month % 12 + 1)?,
        20,
    )?))
}

#[test]
fn closed_conventions_round_trip_exact_identifiers() -> TestResult {
    assert_eq!(CdsMaturityConvention::ALL, [Quarterly2009, Semiannual2015]);
    for (convention, text) in [
        (Quarterly2009, "Quarterly2009"),
        (Semiannual2015, "Semiannual2015"),
    ] {
        assert_eq!(convention.canonical_identifier(), text);
        assert_eq!(convention.to_string(), text);
        assert_eq!(text.parse::<CdsMaturityConvention>()?, convention);
    }
    for text in [
        "",
        "CDS",
        "CDS2015",
        "OldCDS",
        "2009",
        "2015",
        "quarterly2009",
        "semiannual2015",
        " Quarterly2009",
        "Semiannual2015 ",
        "Quarterly2009\n",
    ] {
        assert!(text.parse::<CdsMaturityConvention>().is_err(), "{text:?}");
    }
    // Both enums are intentionally closed: these exhaustive matches compile.
    fn convention_tag(value: CdsMaturityConvention) -> u8 {
        match value {
            Quarterly2009 => 0,
            Semiannual2015 => 1,
        }
    }
    fn outcome_tag(value: CdsMaturityOutcome) -> u8 {
        match value {
            Maturity(_) => 0,
            Matured => 1,
        }
    }
    assert_eq!(convention_tag(Quarterly2009), 0);
    assert_eq!(convention_tag(Semiannual2015), 1);
    assert_eq!(outcome_tag(Maturity(Date::MIN)), 0);
    assert_eq!(outcome_tag(Matured), 1);
    Ok(())
}

#[test]
fn final_primary_tables_cover_every_quarter_tenor_and_all_six_windows() -> TestResult {
    // Final December 10, 2015 ISDA FAQ Q11: the primary search index exposes
    // all 21 tenor rows and SIX windows. Direct PDF retrieval failed. The
    // cdsmodel.com/news.html December 20 table directly corroborates nine
    // tenors in the first four windows. These literal expected dates are
    // transcribed numeric facts, not calculated by the resolver or oracle.
    // Paired windows have identical positive-quote columns; only 0M matures.
    let columns = [
        "2015-12-20 2016-03-20 2016-06-20 2016-09-20 2016-12-20 \
         2017-03-20 2017-06-20 2017-09-20 2017-12-20 \
         2018-03-20 2018-06-20 2018-09-20 2018-12-20 \
         2019-03-20 2019-06-20 2019-09-20 2019-12-20 \
         2020-03-20 2020-06-20 2020-09-20 2020-12-20",
        "2016-06-20 2016-09-20 2016-12-20 2017-03-20 2017-06-20 \
         2017-09-20 2017-12-20 2018-03-20 2018-06-20 \
         2018-09-20 2018-12-20 2019-03-20 2019-06-20 \
         2019-09-20 2019-12-20 2020-03-20 2020-06-20 \
         2020-09-20 2020-12-20 2021-03-20 2021-06-20",
        "2016-12-20 2017-03-20 2017-06-20 2017-09-20 2017-12-20 \
         2018-03-20 2018-06-20 2018-09-20 2018-12-20 \
         2019-03-20 2019-06-20 2019-09-20 2019-12-20 \
         2020-03-20 2020-06-20 2020-09-20 2020-12-20 \
         2021-03-20 2021-06-20 2021-09-20 2021-12-20",
    ];
    let windows = [
        ("2015-09-20", "2015-12-19", 0_usize, false),
        ("2015-12-20", "2016-03-19", 0, true),
        ("2016-03-20", "2016-06-19", 1, false),
        ("2016-06-20", "2016-09-19", 1, true),
        ("2016-09-20", "2016-12-19", 2, false),
        ("2016-12-20", "2017-03-19", 2, true),
    ];
    let mut comparisons = 0;
    for (start, end, column, zero_matured) in windows {
        let expected: Vec<Date> = columns[column]
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        assert_eq!(expected.len(), 21);
        for trade in [start.parse()?, end.parse()?] {
            for (index, date) in expected.iter().enumerate() {
                let quote = 3 * i32::try_from(index)?;
                let outcome = if quote == 0 && zero_matured {
                    Matured
                } else {
                    Maturity(*date)
                };
                assert_eq!(
                    resolve_cds_maturity(trade, months(quote), Semiannual2015)?.outcome(),
                    outcome,
                    "{trade}, {quote}M"
                );
                if quote % 12 == 0 {
                    assert_eq!(
                        resolve_cds_maturity(trade, years(quote / 12), Semiannual2015)?.outcome(),
                        outcome
                    );
                }
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 252);
    // Q2's equality example requires strictly AFTER trade for Quarterly2009;
    // the December22 old/new pair is also published by the model news page.
    for (trade, convention, expected) in [
        ("2015-06-20", Quarterly2009, "2020-09-20"),
        ("2015-12-22", Quarterly2009, "2021-03-20"),
        ("2015-12-22", Semiannual2015, "2020-12-20"),
    ] {
        assert_eq!(
            resolve_cds_maturity(trade.parse()?, years(5), convention)?.outcome(),
            Maturity(expected.parse()?)
        );
    }
    Ok(())
}

#[test]
fn every_quarter_boundary_and_full_gregorian_cycle_match_candidate_enumeration() -> TestResult {
    for year in [1, 1900, 2000, 2015, 2016, 2100, 9997] {
        for month in [3, 6, 9, 12] {
            for day in [19, 20, 21] {
                let trade = Date::from_ymd(year, month, day)?;
                for convention in CdsMaturityConvention::ALL {
                    for quote in [0, 3, 6, 9, 12] {
                        assert_eq!(
                            resolve_cds_maturity(trade, months(quote), convention)?.outcome(),
                            enumerated_outcome(trade, quote, convention)?,
                            "{trade}, {quote}M, {convention}"
                        );
                    }
                }
            }
        }
    }
    let mut trade = Date::from_ymd(2000, 1, 1)?;
    let end = Date::from_ymd(2400, 1, 1)?;
    let mut dates = 0;
    let mut comparisons = 0;
    while trade < end {
        for convention in CdsMaturityConvention::ALL {
            for quote in [0, 3, 6, 9, 12, 60] {
                let actual = resolve_cds_maturity(trade, months(quote), convention)?;
                assert_eq!(
                    actual.outcome(),
                    enumerated_outcome(trade, quote, convention)?
                );
                match actual.outcome() {
                    Maturity(date) => assert!(date > trade),
                    Matured => {
                        assert_eq!(convention, Semiannual2015);
                        assert_eq!(quote, 0);
                    }
                }
                comparisons += 1;
            }
        }
        dates += 1;
        trade = trade.checked_add_days(1)?;
    }
    assert_eq!(dates, 146_097);
    assert_eq!(comparisons, 1_753_164);
    Ok(())
}

fn assert_date_error(
    trade: Date,
    quote: Period,
    convention: CdsMaturityConvention,
    displacement: i64,
) -> TestResult {
    let error = resolve_cds_maturity(trade, quote, convention)
        .err()
        .ok_or("expected required maturity range error")?;
    let expected_cause = trade
        .checked_add_months(displacement)
        .err()
        .ok_or("independent final month must be out of range")?;
    match &error {
        ScheduleError::CdsMaturityDate {
            trade_date,
            tenor,
            convention: found,
            cause,
            ..
        } => {
            assert_eq!(*trade_date, trade);
            assert_eq!(tenor.length(), quote.length());
            assert_eq!(tenor.unit(), quote.unit());
            assert_eq!(*found, convention);
            assert_eq!(*cause, expected_cause);
        }
        other => return Err(format!("unexpected error: {other:?}").into()),
    }
    assert_eq!(error.kind(), "schedule.cds_maturity_date");
    assert_eq!(
        error
            .source()
            .and_then(|source| source.downcast_ref::<DateError>()),
        Some(&expected_cause)
    );
    assert!(!error.to_string().is_empty());
    Ok(())
}

#[test]
fn civil_limits_construct_only_the_required_final_date_and_keep_its_cause() -> TestResult {
    for (trade, quote, convention, expected) in [
        ("0001-01-01", 0, Semiannual2015, None),
        ("0001-01-01", 3, Semiannual2015, Some("0001-03-20")),
        ("0001-01-01", 6, Semiannual2015, Some("0001-06-20")),
        ("0001-01-01", 9, Semiannual2015, Some("0001-09-20")),
        ("0001-01-01", 12, Semiannual2015, Some("0001-12-20")),
        ("0001-01-01", 60, Semiannual2015, Some("0005-12-20")),
        ("0001-03-19", 3, Semiannual2015, Some("0001-03-20")),
        ("0001-01-01", 0, Quarterly2009, Some("0001-03-20")),
        ("9999-06-20", 6, Semiannual2015, Some("9999-12-20")),
        ("9999-12-19", 0, Quarterly2009, Some("9999-12-20")),
        ("9999-12-19", 0, Semiannual2015, Some("9999-12-20")),
        ("9999-12-20", 0, Semiannual2015, None),
        ("9999-12-31", 0, Semiannual2015, None),
    ] {
        let expected = match expected {
            Some(text) => Maturity(text.parse()?),
            None => Matured,
        };
        assert_eq!(
            resolve_cds_maturity(trade.parse()?, months(quote), convention)?.outcome(),
            expected
        );
    }
    assert_date_error(Date::MAX, months(0), Quarterly2009, 3)?;
    assert_date_error("9999-12-20".parse()?, months(0), Quarterly2009, 3)?;
    assert_date_error("9999-06-20".parse()?, months(6), Quarterly2009, 9)?;
    assert_date_error(Date::MAX, months(3), Semiannual2015, 3)?;
    for convention in CdsMaturityConvention::ALL {
        let base_displacement = match convention {
            Quarterly2009 => 2,
            Semiannual2015 => -1,
        };
        for (quote, quoted_months) in [
            (years(i32::MAX), i64::from(i32::MAX) * 12),
            (months(i32::MAX - 1), i64::from(i32::MAX - 1)),
        ] {
            assert_date_error(
                "2026-01-31".parse()?,
                quote,
                convention,
                quoted_months + base_displacement,
            )?;
        }
    }
    // A large valid quote can exactly reach the final representable quarter.
    for convention in CdsMaturityConvention::ALL {
        assert_eq!(
            resolve_cds_maturity(Date::MIN, months(119_985), convention)?.outcome(),
            Maturity(match convention {
                Quarterly2009 => "9999-12-20".parse()?,
                Semiannual2015 => "9999-09-20".parse()?,
            })
        );
    }
    assert_eq!(
        resolve_cds_maturity(Date::MIN, years(9999), Semiannual2015)?.outcome(),
        Maturity("9999-12-20".parse()?)
    );
    assert_date_error(Date::MIN, years(9999), Quarterly2009, 119_990)?;
    Ok(())
}

#[test]
fn original_tenor_domain_precedes_matured_and_range_shortcuts() -> TestResult {
    let rejected = [
        Period::new(0, TimeUnit::Days),
        Period::new(0, TimeUnit::Weeks),
        Period::new(3, TimeUnit::Days),
        Period::new(13, TimeUnit::Weeks),
        months(1),
        months(2),
        months(4),
        months(5),
        months(7),
        months(i32::MAX),
        months(-3),
        years(-1),
        months(i32::MIN),
        years(i32::MIN),
    ];
    for trade in [Date::MIN, "2026-06-20".parse()?, Date::MAX] {
        for convention in CdsMaturityConvention::ALL {
            for quote in rejected {
                let error = resolve_cds_maturity(trade, quote, convention).unwrap_err();
                match &error {
                    ScheduleError::InvalidCdsTenor {
                        trade_date,
                        tenor,
                        convention: found,
                        ..
                    } => {
                        assert_eq!(*trade_date, trade);
                        assert_eq!(tenor.length(), quote.length());
                        assert_eq!(tenor.unit(), quote.unit());
                        assert_eq!(*found, convention);
                    }
                    other => return Err(format!("unexpected error: {other:?}").into()),
                }
                assert_eq!(error.kind(), "schedule.invalid_cds_tenor");
                assert!(error.source().is_none());
                assert!(!error.to_string().is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn stored_aliases_trade_and_regime_remain_distinct_immutable_definitions() -> TestResult {
    let trade: Date = "2026-01-31".parse()?;
    for convention in CdsMaturityConvention::ALL {
        for year_count in [0, 1, 5] {
            let as_years = resolve_cds_maturity(trade, years(year_count), convention)?;
            let as_months = resolve_cds_maturity(trade, months(12 * year_count), convention)?;
            assert_eq!(as_years.outcome(), as_months.outcome());
            assert_eq!(as_years.quoted_tenor(), as_months.quoted_tenor());
            assert_ne!(as_years, as_months);
            assert_eq!(as_years.quoted_tenor().length(), year_count);
            assert_eq!(as_years.quoted_tenor().unit(), TimeUnit::Years);
            assert_eq!(as_months.quoted_tenor().length(), 12 * year_count);
            assert_eq!(as_months.quoted_tenor().unit(), TimeUnit::Months);
            assert_eq!(as_years.trade_date(), trade);
            assert_eq!(as_years.convention(), convention);
            assert_eq!(HashSet::from([as_years, as_months, as_years]).len(), 2);
        }
    }
    let a = resolve_cds_maturity("2026-03-20".parse()?, months(12), Quarterly2009)?;
    let b = resolve_cds_maturity("2026-03-21".parse()?, months(12), Quarterly2009)?;
    let c = resolve_cds_maturity("2026-03-20".parse()?, months(12), Semiannual2015)?;
    assert_eq!(a.outcome(), b.outcome());
    assert_eq!(a.outcome(), c.outcome());
    assert_eq!(HashSet::from([a, b, c]).len(), 3);
    let matured = resolve_cds_maturity(Date::MIN, months(0), Semiannual2015)?;
    assert_eq!(matured.trade_date(), Date::MIN);
    assert_eq!(matured.outcome(), Matured);
    assert_eq!(matured.quoted_tenor().unit(), TimeUnit::Months);
    // Leap/day-end inputs retain their actual trade while maturity is day 20.
    for text in ["1900-02-28", "2000-02-29", "2100-02-28", "2026-01-31"] {
        let trade = text.parse()?;
        for convention in CdsMaturityConvention::ALL {
            for quote in [15, 18, 21, 24, 120] {
                assert_eq!(
                    resolve_cds_maturity(trade, months(quote), convention)?.outcome(),
                    enumerated_outcome(trade, quote, convention)?
                );
            }
        }
    }
    Ok(())
}

// Strict decoder for the frozen compact fixture fields, not a general JSON
// parser. Unexpected evidence shape is a test error rather than a repaired row.
fn json_string<'a>(json: &'a str, key: &str) -> Result<&'a str, Box<dyn Error>> {
    let marker = format!("\"{key}\":\"");
    if json.matches(&marker).count() != 1 {
        return Err(format!("expected exactly one string field {key}").into());
    }
    Ok(json
        .split_once(&marker)
        .ok_or("missing field")?
        .1
        .split_once('"')
        .ok_or("unterminated frozen string")?
        .0)
}

#[test]
fn existing_attested_maturity_observations_match_all_96_modern_calls() -> TestResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/roll-probes.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 538);
    assert_eq!(fixture.header("generated-at"), Some("2026-10-13T17:48:07Z"));
    assert_eq!(
        fixture.header("quantlib-wheel-sha256"),
        Some("85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c")
    );
    assert_eq!(
        fixture.header("quantlib-module-manifest-sha256"),
        Some("e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d")
    );
    let mut modern_counts = [0, 0];
    let mut old_values = 0;
    let mut old_errors = 0;
    for row in fixture.records() {
        let observation = row.text("public_observation")?;
        let Some((_, controls)) = observation.split_once("\"maturity_controls\":[") else {
            return Err("missing maturity_controls field".into());
        };
        let controls = controls.split_once(']').ok_or("unterminated controls")?.0;
        if controls.is_empty() {
            continue;
        }
        assert_eq!(json_string(observation, "runtime_version")?, "1.43");
        let trade = row.parse("start")?;
        for control in controls.split("},{") {
            let (_, quote) = control
                .split_once("\"quoted_months\":")
                .ok_or("missing quoted_months")?;
            let quote: i32 = quote
                .split_once(',')
                .ok_or("missing quoted_months end")?
                .0
                .parse()?;
            let status = json_string(control, "status")?;
            let (convention, index) = match row.text("rule")? {
                "CDS" => (Quarterly2009, 0),
                "CDS2015" => (Semiannual2015, 1),
                "OldCDS" => {
                    if quote == 0 {
                        assert_eq!(status, "error");
                        assert_eq!(json_string(control, "error_type")?, "RuntimeError");
                        old_errors += 1;
                    } else {
                        assert_eq!(status, "value");
                        let _: Date = json_string(control, "date")?.parse()?;
                        old_values += 1;
                    }
                    continue;
                }
                other => return Err(format!("unexpected maturity family {other}").into()),
            };
            let expected = match status {
                "null-date" => {
                    assert!(control.contains("\"date\":null"));
                    Matured
                }
                "value" => Maturity(json_string(control, "date")?.parse()?),
                other => return Err(format!("unexpected modern maturity status {other}").into()),
            };
            assert_eq!(
                resolve_cds_maturity(trade, months(quote), convention)?.outcome(),
                expected,
                "{}, {quote}M",
                row.text("id")?
            );
            modern_counts[index] += 1;
        }
    }
    assert_eq!(modern_counts, [48, 48]);
    assert_eq!(old_values, 36);
    assert_eq!(old_errors, 12);
    assert_eq!(
        modern_counts.iter().sum::<usize>() + old_values + old_errors,
        144
    );
    Ok(())
}

#[test]
fn values_are_copy_send_sync_and_thread_results_are_deterministic() -> TestResult {
    fn value_traits<T: Copy + Clone + Debug + Eq + Hash + Send + Sync + 'static>() {}
    value_traits::<CdsMaturityConvention>();
    value_traits::<CdsMaturityOutcome>();
    value_traits::<CdsMaturityResolution>();
    let inputs = [
        (Date::MIN, months(0), Semiannual2015),
        (Date::MIN, months(3), Semiannual2015),
        ("2026-06-20".parse()?, years(5), Quarterly2009),
        ("2026-06-20".parse()?, years(5), Semiannual2015),
        (Date::MAX, months(0), Semiannual2015),
    ];
    let expected =
        inputs.map(|(trade, quote, convention)| resolve_cds_maturity(trade, quote, convention));
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    inputs.map(|(trade, quote, convention)| {
                        resolve_cds_maturity(trade, quote, convention)
                    })
                })
            })
            .collect();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
    Ok(())
}
