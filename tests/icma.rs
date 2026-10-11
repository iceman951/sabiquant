//! ICMA explicit-period validation: FS-DCC §9.5, TS-DCC-01/03/07/08.
//!
//! A daily integer-weight prefix oracle over independently enumerated civil
//! months checks all ordered endpoint pairs in both mandated date windows.
//! Numerical stress cases exercise the maximum number of positive intervals.

#![expect(
    clippy::expect_used,
    reason = "test helpers require valid explicit fixture/scenario inputs"
)]

mod support;

use sabiquant::daycount::{ActualActualIcma, DeterminationPeriod, IcmaError};
use sabiquant::time::{Date, Frequency};
use std::error::Error as StdError;
use support::fixture::Fixture;
use support::float::ulp_distance;

fn date(text: &str) -> Date {
    text.parse().expect("valid test date")
}

fn period(start: &str, end: &str) -> DeterminationPeriod {
    DeterminationPeriod::new(date(start), date(end)).expect("positive explicit period")
}

fn convention(boundaries: &[&str], frequency: Frequency) -> ActualActualIcma {
    ActualActualIcma::new(
        boundaries
            .windows(2)
            .map(|dates| period(dates[0], dates[1]))
            .collect(),
        frequency,
    )
    .expect("ordered adjacent positive periods")
}

fn assert_ulps(actual: f64, expected: f64, limit: u64) {
    assert!(actual.is_finite());
    assert!(
        ulp_distance(actual, expected).expect("finite expected value") <= limit,
        "actual={actual:.17}, expected={expected:.17}, ulps={:?}",
        ulp_distance(actual, expected)
    );
}

fn error_contract(error: &IcmaError, kind: &str) {
    assert_eq!(error.kind(), kind);
    assert!(!error.to_string().is_empty());
    assert!(error.source().is_none());
    assert_eq!(&error.clone(), error);
    let wrapped: sabiquant::error::Error = error.clone().into();
    assert_eq!(wrapped.kind(), kind);
    assert_eq!(wrapped.to_string(), error.to_string());
    assert_eq!(
        wrapped
            .source()
            .and_then(|source| source.downcast_ref::<IcmaError>()),
        Some(error)
    );
}

#[test]
fn explicit_regular_and_all_stub_shapes_follow_primary_icma_fraction_definition() {
    let cases = [
        (
            vec!["2003-11-01", "2004-05-01"],
            Frequency::Semiannual,
            "2003-11-01",
            "2004-05-01",
            1.0 / 2.0,
        ),
        (
            vec!["1998-07-01", "1999-07-01"],
            Frequency::Annual,
            "1999-02-01",
            "1999-07-01",
            150.0 / 365.0,
        ),
        (
            vec!["2002-07-15", "2003-01-15", "2003-07-15"],
            Frequency::Semiannual,
            "2002-08-15",
            "2003-07-15",
            337.0 / 368.0,
        ),
        (
            vec!["1999-12-31", "2000-06-30"],
            Frequency::Semiannual,
            "1999-12-31",
            "2000-02-29",
            60.0 / 364.0,
        ),
        (
            vec!["2000-01-01", "2000-07-01", "2001-01-01"],
            Frequency::Semiannual,
            "2000-01-01",
            "2000-08-15",
            229.0 / 368.0,
        ),
    ];
    for (boundaries, frequency, start, end, expected) in cases {
        let value = convention(&boundaries, frequency);
        let actual = value.year_fraction(date(start), date(end)).unwrap();
        assert_ulps(actual, expected, 4);
        assert_eq!(
            value
                .year_fraction(date(end), date(start))
                .unwrap()
                .to_bits(),
            (-actual).to_bits()
        );
        assert_eq!(
            value.day_count(date(start), date(end)),
            date(end).days_since(date(start))
        );
        assert_eq!(value.periods().len(), boundaries.len() - 1);
        assert_eq!(value.frequency(), frequency);
        assert_eq!(value.coverage_start(), date(boundaries[0]));
        assert_eq!(value.coverage_end(), date(boundaries[boundaries.len() - 1]));
    }
}

#[test]
fn every_periodic_frequency_is_explicit_and_once_is_rejected() {
    let reference = period("2024-01-01", "2025-01-01");
    for frequency in Frequency::ALL {
        match frequency.events_per_year() {
            Some(events) => {
                let value = ActualActualIcma::new(vec![reference], frequency).unwrap();
                assert_eq!(
                    value
                        .year_fraction(reference.start(), reference.end())
                        .unwrap()
                        .to_bits(),
                    (1.0 / f64::from(events)).to_bits()
                );
            }
            None => assert_eq!(
                ActualActualIcma::new(vec![reference], frequency)
                    .unwrap_err()
                    .kind(),
                "icma.invalid_frequency"
            ),
        }
    }
    // No date-based frequency guessing: the declared divisor is honored.
    let fortnight = period("2024-01-01", "2024-01-15");
    let declared = ActualActualIcma::new(vec![fortnight], Frequency::Quarterly).unwrap();
    assert_eq!(
        declared
            .year_fraction(fortnight.start(), fortnight.end())
            .unwrap()
            .to_bits(),
        0.25_f64.to_bits()
    );
}

#[test]
fn all_constructor_error_kinds_retain_dates_indices_and_frequency() {
    for (start, end) in [("2024-01-01", "2024-01-01"), ("2024-02-01", "2024-01-01")] {
        let error = DeterminationPeriod::new(date(start), date(end)).unwrap_err();
        error_contract(&error, "icma.invalid_period");
        assert!(
            matches!(error, IcmaError::InvalidPeriod { start: rejected_start, end: rejected_end, .. } if rejected_start == date(start) && rejected_end == date(end))
        );
    }
    error_contract(
        &ActualActualIcma::new(Vec::new(), Frequency::Annual).unwrap_err(),
        "icma.empty_periods",
    );
    let frequency = ActualActualIcma::new(Vec::new(), Frequency::Once).unwrap_err();
    error_contract(&frequency, "icma.invalid_frequency");
    assert!(matches!(
        frequency,
        IcmaError::InvalidFrequency {
            frequency: Frequency::Once,
            ..
        }
    ));
    let first = period("2024-01-01", "2024-02-01");
    for second in [
        period("2024-02-02", "2024-03-01"),
        period("2024-01-31", "2024-03-01"),
        first,
        period("2023-12-01", "2024-01-01"),
    ] {
        let error = ActualActualIcma::new(vec![first, second], Frequency::Monthly).unwrap_err();
        error_contract(&error, "icma.non_contiguous_periods");
        assert!(
            matches!(error, IcmaError::NonContiguousPeriods { index: 1, previous_end, start, .. } if previous_end == first.end() && start == second.start())
        );
    }
}

#[test]
fn coverage_is_validated_before_zero_or_reversed_calculation() {
    let value = convention(
        &["2024-01-01", "2024-02-01", "2024-03-01"],
        Frequency::Monthly,
    );
    for zero in ["2024-01-01", "2024-01-15", "2024-02-01", "2024-03-01"] {
        assert_eq!(
            value
                .year_fraction(date(zero), date(zero))
                .unwrap()
                .to_bits(),
            0.0_f64.to_bits()
        );
    }
    for (start, end) in [
        ("2023-12-31", "2023-12-31"),
        ("2024-03-02", "2024-03-02"),
        ("2023-12-31", "2024-02-01"),
        ("2024-02-01", "2023-12-31"),
        ("2024-02-01", "2024-03-02"),
        ("2024-03-02", "2024-02-01"),
    ] {
        let error = value.year_fraction(date(start), date(end)).unwrap_err();
        error_contract(&error, "icma.out_of_coverage");
        assert!(
            matches!(error, IcmaError::OutOfCoverage { start: rejected_start, end: rejected_end, coverage_start, coverage_end, .. } if rejected_start == date(start) && rejected_end == date(end) && coverage_start == value.coverage_start() && coverage_end == value.coverage_end())
        );
    }
    assert_eq!(value.day_count(Date::MIN, Date::MAX), 3_652_058);
    assert_eq!(value.day_count(Date::MAX, Date::MIN), -3_652_058);
}

fn month_length(year: i32, month: u32) -> u32 {
    match month {
        2 => 28 + u32::from(year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)),
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records measured oracle error for reproducible validation with --nocapture"
)]
fn ts_dcc_07_all_3338660_window_pairs_match_independent_integer_weight_oracle() {
    // LCM(28,29,30,31) allows each daily monthly-ICMA weight to be
    // represented as an exact integer, with no production-style divisions.
    const WEIGHT_DENOMINATOR: i32 = 377_580;
    let mut total = 0_u64;
    let mut maximum_ulp = 0;
    for (first_year, last_year) in [(1999, 2002), (2099, 2101)] {
        let mut boundaries = Vec::new();
        let mut dates = Vec::new();
        let mut prefix = vec![0_i32];
        let mut weight = 0;
        for year in first_year..=last_year {
            for month in 1..=12 {
                boundaries.push(Date::from_ymd(year, month, 1).unwrap());
                let length = month_length(year, month);
                for day in 1..=length {
                    dates.push(Date::from_ymd(year, month, day).unwrap());
                    weight += WEIGHT_DENOMINATOR / i32::try_from(length).unwrap();
                    prefix.push(weight);
                }
            }
        }
        let last = Date::from_ymd(last_year + 1, 1, 1).unwrap();
        boundaries.push(last);
        dates.push(last);
        let periods = boundaries
            .windows(2)
            .map(|pair| DeterminationPeriod::new(pair[0], pair[1]).unwrap())
            .collect();
        let value = ActualActualIcma::new(periods, Frequency::Monthly).unwrap();
        for (start_index, &start) in dates.iter().enumerate() {
            for (end_index, &end) in dates.iter().enumerate() {
                let numerator = prefix[end_index] - prefix[start_index];
                let expected = f64::from(numerator) / f64::from(12 * WEIGHT_DENOMINATOR);
                let actual = value.year_fraction(start, end).unwrap();
                let ulps = ulp_distance(actual, expected).unwrap();
                maximum_ulp = maximum_ulp.max(ulps);
                assert!(
                    ulps <= 4,
                    "{start}..{end}: {actual} vs {expected}, {ulps} ULP"
                );
                total += 1;
            }
        }
    }
    assert_eq!(total, 3_338_660);
    assert!(maximum_ulp <= 4);
    println!("ICMA exhaustive monthly oracle: cases={total}, maximum_ulp={maximum_ulp}");
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records maximum-domain numerical stress evidence with --nocapture"
)]
fn maximum_3652058_daily_segments_keep_compensated_sum_within_e2() {
    let periods = (0..3_652_058)
        .map(|offset| {
            let start = Date::MIN.checked_add_days(offset).unwrap();
            DeterminationPeriod::new(start, start.checked_add_days(1).unwrap()).unwrap()
        })
        .collect();
    let value = ActualActualIcma::new(periods, Frequency::Daily).unwrap();
    let actual = value.year_fraction(Date::MIN, Date::MAX).unwrap();
    let expected = 3_652_058.0 / 365.0;
    assert_ulps(actual, expected, 4);
    println!(
        "ICMA 3652058 daily segments: ulps={}",
        ulp_distance(actual, expected).unwrap()
    );
    assert_eq!(
        value.year_fraction(Date::MAX, Date::MIN).unwrap().to_bits(),
        (-actual).to_bits()
    );
    // Late clipped queries exercise binary search, not rescanning all periods.
    assert_ulps(
        value
            .year_fraction(Date::MAX.checked_sub_days(7).unwrap(), Date::MAX)
            .unwrap(),
        7.0 / 365.0,
        4,
    );
}

#[test]
fn full_range_single_period_and_extreme_frequency_denominators_are_exact() {
    let reference = DeterminationPeriod::new(Date::MIN, Date::MAX).unwrap();
    for frequency in Frequency::ALL {
        if let Some(events) = frequency.events_per_year() {
            let value = ActualActualIcma::new(vec![reference], frequency).unwrap();
            assert_eq!(
                value.year_fraction(Date::MIN, Date::MAX).unwrap().to_bits(),
                (1.0 / f64::from(events)).to_bits()
            );
            let end = Date::MIN.checked_add_days(1).unwrap();
            assert_eq!(
                value.year_fraction(Date::MIN, end).unwrap().to_bits(),
                (1.0 / (f64::from(events) * 3_652_058.0)).to_bits()
            );
        }
    }
}

#[test]
fn public_icma_values_and_errors_are_immutable_standard_thread_safe_types() {
    fn value<T: Clone + std::fmt::Debug + PartialEq + Send + Sync + 'static>() {}
    fn error<T: StdError + Clone + PartialEq + Send + Sync + 'static>() {}
    value::<DeterminationPeriod>();
    value::<ActualActualIcma>();
    error::<IcmaError>();
    let error = DeterminationPeriod::new(Date::MAX, Date::MIN).unwrap_err();
    assert!(error.source().is_none());
    assert!(!error.to_string().is_empty());
    assert_eq!(error.clone(), error);
    let wrapped: sabiquant::error::Error = error.into();
    assert_eq!(wrapped.kind(), "icma.invalid_period");
    assert!(wrapped.source().is_some());
}

fn icma_fixture(name: &str) -> Fixture {
    Fixture::load(Fixture::repository_path(&format!(
        "validation/fixtures/daycount/{name}.fixture"
    )))
    .expect("committed ICMA schema-1 fixture")
}

fn fixture_convention(boundaries: &str, frequency: Frequency) -> ActualActualIcma {
    let dates = boundaries.split(',').map(date).collect::<Vec<_>>();
    let periods = dates
        .windows(2)
        .map(|pair| {
            DeterminationPeriod::new(pair[0], pair[1]).expect("positive fixture reference period")
        })
        .collect();
    ActualActualIcma::new(periods, frequency).expect("valid explicit fixture convention")
}

// Reconstruct the fixture's exact rational from contractual inputs without
// performing floating-point division or reusing compensated summation.
// Fractions are reduced before cross-products; every intermediate operation
// is checked so a changed fixture cannot silently overflow the test oracle.
fn exact_fixture_fraction(value: &ActualActualIcma, start: Date, end: Date) -> (i128, i128) {
    fn gcd(mut left: i128, mut right: i128) -> i128 {
        assert!(left >= 0 && right > 0);
        while right != 0 {
            (left, right) = (right, left % right);
        }
        left
    }

    let lower = start.min(end);
    let upper = start.max(end);
    let frequency = i128::from(
        value
            .frequency()
            .events_per_year()
            .expect("fixture declares a periodic frequency"),
    );
    let mut numerator = 0_i128;
    let mut denominator = 1_i128;
    for &period in value.periods() {
        let first = lower.max(period.start());
        let last = upper.min(period.end());
        if first >= last {
            continue;
        }
        let term_numerator = i128::from(last.days_since(first));
        let term_denominator = frequency
            .checked_mul(i128::from(period.end().days_since(period.start())))
            .expect("exact fixture term denominator fits i128");
        let reduction = gcd(term_numerator, term_denominator);
        let term_numerator = term_numerator / reduction;
        let term_denominator = term_denominator / reduction;
        let common = gcd(denominator, term_denominator);
        let combined = numerator
            .checked_mul(term_denominator / common)
            .and_then(|left| {
                term_numerator
                    .checked_mul(denominator / common)
                    .and_then(|right| left.checked_add(right))
            })
            .expect("pre-reduced exact fixture numerator fits i128");
        // For two reduced fractions, any remaining common divisor of the
        // combined numerator and denominator divides the old common factor.
        // Divide it out before multiplying the denominator's factors.
        let final_reduction = gcd(combined, common);
        numerator = combined / final_reduction;
        denominator = (denominator / common)
            .checked_mul(term_denominator / final_reduction)
            .expect("pre-reduced exact fixture denominator fits i128");
        assert!(denominator > 0);
    }
    if start > end {
        numerator = numerator
            .checked_neg()
            .expect("exact fixture numerator can be negated");
    }
    (numerator, denominator)
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records independently generated Fraction fixture error with --nocapture"
)]
fn python_fraction_oracle_covers_all_frequencies_stubs_long_and_extreme_references() {
    let data = icma_fixture("icma-oracle");
    assert_eq!(data.record_count(), 101);
    let mut maximum_ulp = 0;
    for record in data.records() {
        let value = fixture_convention(
            record.text("boundaries").unwrap(),
            record.parse("frequency").unwrap(),
        );
        let start: Date = record.parse("start").unwrap();
        let end: Date = record.parse("end").unwrap();
        let actual = value.year_fraction(start, end).unwrap();
        let expected = record.real("expected").unwrap();
        let ulps = ulp_distance(actual, expected).unwrap();
        assert!(ulps <= 4, "{}: {ulps} ULP", record.id());
        maximum_ulp = maximum_ulp.max(ulps);
        assert_eq!(
            exact_fixture_fraction(&value, start, end),
            (
                record.parse::<i128>("exact_numerator").unwrap(),
                record.parse::<i128>("exact_denominator").unwrap(),
            ),
            "{}: rational metadata must follow the supplied accrual/reference inputs",
            record.id()
        );
    }
    println!("ICMA Python Fraction fixtures: cases=101, maximum_ulp={maximum_ulp}");
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records actual explicit-period QuantLib component errors with --nocapture"
)]
fn actual_quantlib_143_explicit_reference_calls_match_components_and_totals() {
    let data = icma_fixture("icma-quantlib");
    assert_eq!(data.record_count(), 85);
    let mut maximum_ulp = 0;
    for record in data.records() {
        let frequency: Frequency = record.parse("frequency").unwrap();
        let value = fixture_convention(record.text("boundaries").unwrap(), frequency);
        let start: Date = record.parse("start").unwrap();
        let end: Date = record.parse("end").unwrap();
        assert_eq!(
            exact_fixture_fraction(&value, start, end),
            (
                record.parse::<i128>("exact_numerator").unwrap(),
                record.parse::<i128>("exact_denominator").unwrap(),
            ),
            "{}: rational metadata must follow the supplied accrual/reference inputs",
            record.id()
        );
        let actual = value.year_fraction(start, end).unwrap();
        assert_ulps(actual, record.real("expected").unwrap(), 4);
        let reference = record.real("reference_result").unwrap();
        let ulps = ulp_distance(actual, reference).unwrap();
        assert!(ulps <= 8, "{}: {ulps} ULP", record.id());
        maximum_ulp = maximum_ulp.max(ulps);
        let components = record
            .text("reference_components")
            .unwrap()
            .split(',')
            .map(|text| text.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            components.len(),
            record.parse::<usize>("component_count").unwrap()
        );
        let mut index = 0;
        for &period in value.periods() {
            let first = start.min(end).max(period.start());
            let last = start.max(end).min(period.end());
            if first < last {
                let single = ActualActualIcma::new(vec![period], frequency).unwrap();
                assert_ulps(
                    single.year_fraction(first, last).unwrap(),
                    components[index],
                    8,
                );
                index += 1;
            }
        }
        assert_eq!(index, components.len());
    }
    println!("ICMA public QuantLib fixtures: cases=85, maximum_ulp={maximum_ulp}");
}

#[test]
fn exact_zero_outside_coverage_divergences_retain_reference_value_and_strict_error() {
    let data = icma_fixture("icma-strict-coverage");
    assert_eq!(data.record_count(), 2);
    let mut observed = Vec::new();
    for record in data.records() {
        assert_eq!(record.text("discrepancy").unwrap(), "D9-ICMA-ZERO-COVERAGE");
        assert_eq!(record.text("boundaries").unwrap(), "2024-01-01,2024-07-01");
        assert_eq!(
            record.parse::<Frequency>("frequency").unwrap(),
            Frequency::Semiannual
        );
        assert_eq!(
            record.real("reference_result").unwrap().to_bits(),
            0.0_f64.to_bits()
        );
        let start: Date = record.parse("start").unwrap();
        let end: Date = record.parse("end").unwrap();
        assert_eq!(start, end);
        let value = fixture_convention(record.text("boundaries").unwrap(), Frequency::Semiannual);
        let error = value.year_fraction(start, end).unwrap_err();
        assert_eq!(error.kind(), record.text("expected_error").unwrap());
        assert_eq!(error.kind(), "icma.out_of_coverage");
        observed.push((record.id().to_owned(), start));
    }
    assert_eq!(
        observed,
        vec![
            ("ICMA-ZERO-BEFORE".to_owned(), date("2023-12-31")),
            ("ICMA-ZERO-AFTER".to_owned(), date("2024-07-02"))
        ]
    );
}
