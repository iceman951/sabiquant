//! Exact classifications of bounded public-reference roll observations.
//!
//! The civil annotation is an independent adopted-contract model. Reference
//! vectors/flags stay untouched, including unsupported options and unordered
//! outcomes. A matching vector never implies a matching financial product.

mod support;

use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar, TargetCalendar};
use sabiquant::schedule::{
    DateGenerationRule, PeriodRegularity, ScheduleBuilder, ScheduleCalendar, ScheduleError,
    SpecialRollRule,
};
use sabiquant::time::{Date, Period};
use std::collections::BTreeMap;
use std::error::Error;
use support::fixture::{Fixture, Record};

const CONTRACT_SHA: &str = "1787bedf20c07fd45dd2077c2ed6eeda3641c402111440e1807eaae6d347cd68";
const RESEARCH_SHA: &str = "5c7b5c325d7a9edb069a6c32eb33a9814b3267d1b6b79206ec73bed8ac1a8582";

fn load() -> Result<Fixture, Box<dyn Error>> {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/roll-probes.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 538);
    assert_eq!(fixture.header("contract-source-sha256"), Some(CONTRACT_SHA));
    assert_eq!(fixture.header("research-source-sha256"), Some(RESEARCH_SHA));
    assert_eq!(fixture.header("quantlib-module-files"), Some("3"));
    assert_eq!(
        fixture.header("quantlib-wheel-sha256"),
        Some("85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c")
    );
    assert_eq!(
        fixture.header("quantlib-module-manifest-sha256"),
        Some("e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d")
    );
    assert!(
        fixture
            .header("quantlib-source-baseline")
            .is_some_and(|text| text.contains("not independently attested"))
    );
    Ok(fixture)
}

// This decoder handles only frozen compact flat string/array fields. It is
// intentionally not a JSON parser or a production-input parser. One-key
// checks and exact ISO/bool parsing reject unexpected fixture structure.
#[expect(
    clippy::unwrap_used,
    reason = "test-only strict frozen string decoder rejects malformed evidence"
)]
fn field<'a>(json: &'a str, key: &str) -> &'a str {
    let marker = format!("\"{key}\":\"");
    assert_eq!(json.matches(&marker).count(), 1, "{key}");
    json.split_once(&marker)
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
}

#[expect(
    clippy::unwrap_used,
    reason = "test-only strict frozen array decoder rejects malformed evidence"
)]
fn array<'a>(json: &'a str, key: &str) -> &'a str {
    let marker = format!("\"{key}\":[");
    assert_eq!(json.matches(&marker).count(), 1, "{key}");
    json.split_once(&marker)
        .unwrap()
        .1
        .split_once(']')
        .unwrap()
        .0
}

#[expect(
    clippy::unwrap_used,
    reason = "test-only exact quoted ISO array decoder rejects malformed evidence"
)]
fn dates(json: &str) -> Vec<Date> {
    array(json, "dates")
        .split(',')
        .map(|part| {
            part.strip_prefix('"')
                .unwrap()
                .strip_suffix('"')
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect()
}

#[expect(
    clippy::panic,
    reason = "test-only frozen-field decoder rejects malformed evidence"
)]
fn flags(json: &str) -> Vec<PeriodRegularity> {
    array(json, "regularity")
        .split(',')
        .map(|part| match part {
            "true" => PeriodRegularity::Regular,
            "false" => PeriodRegularity::Stub,
            other => panic!("unexpected frozen boolean {other}"),
        })
        .collect()
}

fn builder(row: &Record<'_>) -> Result<ScheduleBuilder, Box<dyn Error>> {
    let calendar = match row.text("calendar")? {
        "Null" => ScheduleCalendar::Plain(Calendar::null()),
        "Weekends" => ScheduleCalendar::Plain(Calendar::weekends_only()),
        "TARGET" => ScheduleCalendar::Target(TargetCalendar::new()),
        other => return Err(format!("unclassified reference calendar {other}").into()),
    };
    Ok(ScheduleBuilder::new(
        row.parse("start")?,
        row.parse("end")?,
        calendar,
        row.parse("convention")?,
        row.parse("termination")?,
    )?)
}

fn tenor(row: &Record<'_>) -> Result<Period, Box<dyn Error>> {
    Ok(Period::new(
        row.parse("tenor_length")?,
        row.parse("tenor_unit")?,
    ))
}

fn increment(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_owned()).or_default() += 1;
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports exact preserved reference categories"
)]
fn every_observation_preserves_status_order_options_and_reference_only_maturity_controls()
-> Result<(), Box<dyn Error>> {
    let fixture = load()?;
    let mut statuses = BTreeMap::new();
    let mut groups = BTreeMap::new();
    let mut nonstrict = Vec::new();
    let mut maturity_calls = 0;
    let mut unsupported_family = 0;
    let mut absent_options = 0;
    let mut zero_metadata = 0;
    for row in fixture.records() {
        let observation = row.text("public_observation")?;
        let annotation = row.text("adopted_local_contract_annotation")?;
        assert_eq!(field(observation, "runtime_version"), "1.43");
        assert_eq!(
            field(observation, "module_manifest"),
            fixture.header("quantlib-module-manifest-sha256").unwrap()
        );
        let status = row.text("status")?;
        // A nested maturity error may also have a status field, so use the
        // row-level state plus the exact frozen root token here.
        assert!(observation.contains(&format!("\"status\":\"{status}\"")));
        increment(&mut statuses, status);
        increment(&mut groups, row.text("group")?);
        if status == "value" {
            let returned = dates(observation);
            let regularity = flags(observation);
            assert!((2..=400).contains(&returned.len()));
            assert_eq!(regularity.len() + 1, returned.len());
            if field(observation, "returned_rule") == "Zero" {
                assert_eq!(row.parse::<i32>("tenor_length")?, 0);
                assert_eq!(row.text("tenor_unit")?, "Months");
                assert!(observation.contains("\"tenor_length\":0,\"tenor_unit\":\"Years\""));
                assert_eq!(
                    returned,
                    [row.parse::<Date>("start")?, row.parse::<Date>("end")?]
                );
                assert_eq!(regularity, [PeriodRegularity::Regular]);
                zero_metadata += 1;
            } else {
                assert_eq!(field(observation, "returned_rule"), row.text("rule")?);
            }
            assert!(
                returned
                    .iter()
                    .all(|date| (2025..=2028).contains(&date.year()))
            );
            let ordered = returned.windows(2).all(|pair| pair[0] < pair[1]);
            assert!(observation.contains(if ordered {
                "\"strictly_increasing\":true"
            } else {
                "\"strictly_increasing\":false"
            }));
            if !ordered {
                nonstrict.push(row.id().to_owned());
            }
        } else {
            assert_eq!(status, "constructor-error");
            assert!(observation.contains("\"error_type\":\"RuntimeError\""));
            assert!(observation.contains("\"error_message\":\""));
        }
        maturity_calls += observation.matches("\"quoted_months\":").count();
        match row.text("rule")?.parse::<SpecialRollRule>() {
            Err(error) => {
                assert_eq!(error.kind(), "identifier.unknown");
                assert!(matches!(row.text("rule")?, "OldCDS" | "CDS" | "CDS2015"));
                assert_eq!(field(annotation, "status"), "outside-four-rule-contract");
                unsupported_family += 1;
            }
            Ok(_)
                if row.parse::<bool>("end_of_month")?
                    || !row.text("first_coupon_date")?.is_empty()
                    || !row.text("penultimate_coupon_date")?.is_empty() =>
            {
                // build_roll deliberately has no EOM or explicit-role parameter.
                assert_eq!(field(annotation, "status"), "outside-local-options");
                absent_options += 1;
            }
            Ok(_) => {}
        }
    }
    assert_eq!(
        statuses,
        BTreeMap::from([("constructor-error".into(), 72), ("value".into(), 466)])
    );
    assert_eq!(
        (
            unsupported_family,
            absent_options,
            maturity_calls,
            zero_metadata
        ),
        (254, 36, 144, 7)
    );
    nonstrict.sort();
    assert_eq!(
        nonstrict,
        [
            "ROLL-COUPON-THIRDWEDNESDAY-REVERSED-WED",
            "ROLL-UNIT-THIRDWEDNESDAY-1Days",
            "ROLL-UNIT-THIRDWEDNESDAY-1Weeks",
            "ROLL-UNIT-THIRDWEDNESDAYINCLUSIVE-1Days",
            "ROLL-UNIT-THIRDWEDNESDAYINCLUSIVE-1Weeks",
        ]
    );
    assert_eq!(
        groups,
        BTreeMap::from([
            ("before-on-after-roll".into(), 189),
            ("eom-option".into(), 28),
            ("explicit-role-options".into(), 35),
            ("fixed-end-versus-quoted-maturity".into(), 36),
            ("legacy-first-roll-distance".into(), 40),
            ("ordinary-terminal-options".into(), 98),
            ("original-common-control".into(), 7),
            ("same-month-and-year-end".into(), 42),
            ("target-calendar-control".into(), 7),
            ("tenor-domains".into(), 56),
        ])
    );
    println!(
        "roll reference census: 538 observations; {statuses:?}; 5 non-strict; 7 zero-rule rewrites; 144 maturity controls; 254 family/36 option exclusions"
    );
    Ok(())
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports exact civil/reference distinctions without tolerance"
)]
fn independent_civil_annotations_and_domain_errors_match_the_adopted_public_api()
-> Result<(), Box<dyn Error>> {
    let fixture = load()?;
    let mut categories = BTreeMap::new();
    let mut rejected = BTreeMap::new();
    let mut date_differences = Vec::new();
    let mut matched = 0;
    for row in fixture.records() {
        let annotation = row.text("adopted_local_contract_annotation")?;
        let state = field(annotation, "status");
        if matches!(
            state,
            "outside-four-rule-contract" | "outside-local-options"
        ) {
            continue;
        }
        let rule: SpecialRollRule = row.parse("rule")?;
        let period = tenor(&row)?;
        let builder = builder(&row)?;
        let result = builder.build_roll(period, rule, 400);
        if state != "local-value" {
            let error = result.unwrap_err();
            let expected = match state {
                "local-incompatible-tenor" => "schedule.invalid_roll_tenor",
                "local-resolved-range-rejection" => "schedule.resolved_roll_range",
                other => return Err(format!("unclassified local state {other}").into()),
            };
            assert_eq!(error.kind(), expected, "{}", row.id());
            increment(&mut rejected, expected);
            continue;
        }
        let schedule = result?;
        let expected_dates = dates(annotation);
        let expected_flags = flags(annotation);
        assert_eq!(schedule.contractual_dates(), expected_dates, "{}", row.id());
        assert_eq!(schedule.adjusted_dates(), expected_dates, "{}", row.id());
        assert_eq!(schedule.regularity(), expected_flags, "{}", row.id());
        assert_eq!(schedule.generation_rule(), DateGenerationRule::Roll(rule));
        assert_eq!(schedule.stub_policy(), None);
        assert_eq!(schedule.end_of_month(), None);
        assert_eq!(schedule.explicit_stub_dates(), None);
        let provenance = schedule.roll_provenance().unwrap();
        assert_eq!(provenance.requested_start(), row.parse::<Date>("start")?);
        assert_eq!(provenance.requested_end(), row.parse::<Date>("end")?);
        assert_eq!(provenance.resolved_start(), expected_dates[0]);
        assert_eq!(provenance.resolved_end(), *expected_dates.last().unwrap());
        assert_eq!(provenance.rule(), rule);
        assert_eq!(provenance.tenor().length(), period.length());
        assert_eq!(provenance.tenor().unit(), period.unit());
        let reference = row.text("public_observation")?;
        assert_eq!(row.text("status")?, "value");
        let reference_dates = dates(reference);
        let reference_flags = flags(reference);
        let classification = if expected_dates != reference_dates {
            date_differences.push(row.id().to_owned());
            "date-vector-difference"
        } else if expected_flags != reference_flags {
            "regularity-difference-only"
        } else {
            "dates-and-flags-agree"
        };
        assert_eq!(
            field(annotation, "comparison"),
            classification,
            "{}",
            row.id()
        );
        increment(&mut categories, classification);
        let exact_cap = u32::try_from(expected_dates.len())?;
        assert_eq!(builder.build_roll(period, rule, exact_cap)?, schedule);
        assert_eq!(builder.build_roll(period, rule, exact_cap + 1)?, schedule);
        if exact_cap > 2 {
            assert!(matches!(builder.build_roll(period, rule, exact_cap-1),
                Err(ScheduleError::BoundaryLimitExceeded { maximum, .. }) if maximum == exact_cap-1));
        }
        matched += 1;
    }
    assert_eq!(matched, 163);
    assert_eq!(
        categories,
        BTreeMap::from([
            ("date-vector-difference".into(), 6),
            ("dates-and-flags-agree".into(), 79),
            ("regularity-difference-only".into(), 78),
        ])
    );
    assert_eq!(
        rejected,
        BTreeMap::from([
            ("schedule.invalid_roll_tenor".into(), 19),
            ("schedule.resolved_roll_range".into(), 6),
        ])
    );
    assert!(
        date_differences
            .iter()
            .all(|id| id.starts_with("ROLL-EDGE-THIRDWEDNESDAY-2026-03-")
                && id.contains("2026-09-17"))
    );
    println!(
        "roll civil reference: {matched} values {categories:?}; typed rejections {rejected:?}"
    );
    Ok(())
}

#[expect(
    clippy::unwrap_used,
    reason = "hand-audited literal contractual expectations are valid ISO dates"
)]
fn literal_dates(text: &str) -> Vec<Date> {
    text.split(';')
        .map(|value| value.parse().unwrap())
        .collect()
}

fn calendar_raw(rule: SpecialRollRule, target: bool) -> (Vec<Date>, Vec<PeriodRegularity>) {
    use PeriodRegularity::{Regular as R, Stub as S};
    use SpecialRollRule::{
        ThirdWednesday as W, ThirdWednesdayInclusive as I, Twentieth as T, TwentiethIMM as Q,
    };
    let (text, flags) = match (rule, target) {
        (W, false) => (
            "2026-02-06;2026-05-20;2026-08-19;2026-11-18;2026-11-21",
            vec![S, R, R, S],
        ),
        (W, true) => ("2026-02-06;2026-05-20;2026-08-19;2026-11-17", vec![S, R, S]),
        (I, _) => ("2026-02-18;2026-05-20;2026-08-19;2026-11-18", vec![R, R, R]),
        (T, false) => (
            "2026-02-06;2026-02-20;2026-05-20;2026-08-20;2026-11-20;2026-12-20",
            vec![S, R, R, R, S],
        ),
        (T, true) => (
            "2026-02-06;2026-02-20;2026-05-20;2026-08-20;2026-11-20",
            vec![S, R, R, R],
        ),
        (Q, _) => (
            "2026-02-06;2026-03-20;2026-06-20;2026-09-20;2026-12-20",
            vec![S, R, R, R],
        ),
    };
    (literal_dates(text), flags)
}

#[expect(
    clippy::unwrap_used,
    reason = "bounded 2026 neighborhoods contain a weekday within two days"
)]
fn weekend_adjust(date: Date, convention: Bdc) -> Date {
    let is_weekday = |day: Date| day.weekday().number_from_monday() <= 5;
    let seek = |direction: i64| {
        (0_i64..=2)
            .map(|offset| date.checked_add_days(offset * direction).unwrap())
            .find(|&day| is_weekday(day))
            .unwrap()
    };
    let following = seek(1);
    let preceding = seek(-1);
    match convention {
        Bdc::Unadjusted => date,
        Bdc::Following => following,
        Bdc::ModifiedFollowing if following.month() != date.month() => preceding,
        Bdc::HalfMonthModifiedFollowing
            if following.month() != date.month() || date.day() <= 15 && following.day() > 15 =>
        {
            preceding
        }
        Bdc::ModifiedFollowing | Bdc::HalfMonthModifiedFollowing => following,
        Bdc::Preceding => preceding,
        Bdc::ModifiedPreceding if preceding.month() != date.month() => following,
        Bdc::ModifiedPreceding => preceding,
        Bdc::Nearest if following.days_since(date) <= date.days_since(preceding) => following,
        Bdc::Nearest => preceding,
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports independently pinned calendar classifications"
)]
fn all_eligible_calendar_options_preserve_raw_lattice_and_classify_reference_adjustment()
-> Result<(), Box<dyn Error>> {
    let fixture = load()?;
    let mut categories = BTreeMap::new();
    let mut counts = (0, 0);
    let closures =
        literal_dates("2026-01-01;2026-04-03;2026-04-06;2026-05-01;2026-12-25;2026-12-26");
    for row in fixture.records() {
        if !matches!(
            row.text("group")?,
            "ordinary-terminal-options" | "target-calendar-control"
        ) {
            continue;
        }
        let Ok(rule) = row.parse::<SpecialRollRule>("rule") else {
            continue;
        };
        let target = row.text("calendar")? == "TARGET";
        let (expected_raw, expected_flags) = calendar_raw(rule, target);
        let ordinary: Bdc = row.parse("convention")?;
        let terminal: Bdc = row.parse("termination")?;
        // These fixed node/candidate neighborhoods do not intersect any of
        // the six 2026 TARGET closures, so its independent weekday result is
        // sufficient here. Broader TARGET policy is covered by market tests.
        let expected_adjusted: Vec<_> = expected_raw
            .iter()
            .enumerate()
            .map(|(index, &date)| {
                let convention = if index + 1 == expected_raw.len() {
                    terminal
                } else {
                    ordinary
                };
                for offset in -2..=2 {
                    assert!(!closures.contains(&date.checked_add_days(offset).unwrap()));
                }
                weekend_adjust(date, convention)
            })
            .collect();
        let schedule = builder(&row)?.build_roll(tenor(&row)?, rule, 400)?;
        assert_eq!(schedule.contractual_dates(), expected_raw, "{}", row.id());
        assert_eq!(schedule.adjusted_dates(), expected_adjusted, "{}", row.id());
        assert_eq!(schedule.regularity(), expected_flags, "{}", row.id());
        let reference = row.text("public_observation")?;
        assert_eq!(row.text("status")?, "value");
        let category = if expected_adjusted != dates(reference) {
            assert_eq!(rule, SpecialRollRule::Twentieth, "{}", row.id());
            assert_eq!(terminal, Bdc::Preceding, "{}", row.id());
            assert!(!target);
            assert_eq!(
                dates(reference),
                expected_adjusted[..expected_adjusted.len() - 1],
                "{}",
                row.id()
            );
            assert_eq!(
                flags(reference),
                expected_flags[..expected_flags.len() - 1],
                "{}",
                row.id()
            );
            assert_eq!(
                *schedule.contractual_dates().last().unwrap(),
                "2026-12-20".parse::<Date>()?
            );
            assert_eq!(
                *schedule.adjusted_dates().last().unwrap(),
                "2026-12-18".parse::<Date>()?
            );
            "date-vector-difference"
        } else if expected_flags != flags(reference) {
            "regularity-difference-only"
        } else {
            "dates-and-flags-agree"
        };
        increment(&mut categories, category);
        if target {
            counts.1 += 1;
        } else {
            counts.0 += 1;
        }
    }
    assert_eq!(counts, (56, 4));
    println!("roll calendar reference: {counts:?}; {categories:?}");
    // Independent raw/adjusted values above are mandatory even where flags
    // follow a different reference definition. Pin the measured classes too.
    assert_eq!(
        categories,
        BTreeMap::from([
            ("date-vector-difference".into(), 7),
            ("dates-and-flags-agree".into(), 31),
            ("regularity-difference-only".into(), 22),
        ])
    );
    Ok(())
}

#[test]
fn legacy_cutoff_and_quoted_maturity_remain_distinct_reference_only_observations()
-> Result<(), Box<dyn Error>> {
    let fixture = load()?;
    let mut cutoff = BTreeMap::new();
    let mut maturity_rows = 0;
    let mut december_control = 0;
    for row in fixture.records() {
        if row.text("group")? == "legacy-first-roll-distance" {
            assert_eq!(row.text("status")?, "value");
            let rule = row.text("rule")?;
            let prefix = format!("ROLL-CUTOFF-{}-", rule.to_uppercase());
            let suffix = row.id().strip_prefix(&prefix).unwrap();
            let (roll, distance) = suffix.rsplit_once('-').unwrap();
            let roll: Date = roll.parse()?;
            let distance: i64 = distance.parse()?;
            assert_eq!(
                row.parse::<Date>("start")?,
                roll.checked_add_days(-distance)?
            );
            assert!(
                cutoff
                    .insert(
                        (roll, distance, rule.to_owned()),
                        dates(row.text("public_observation")?)
                    )
                    .is_none()
            );
        }
        if row.text("group")? == "fixed-end-versus-quoted-maturity" {
            let controls = array(row.text("public_observation")?, "maturity_controls");
            for months in [0, 3, 6, 12] {
                assert_eq!(
                    controls
                        .matches(&format!("\"quoted_months\":{months},"))
                        .count(),
                    1
                );
            }
            if row.id() == "ROLL-MATURITY-CDS2015-2026-12-21" {
                // Public quoted maturity is a separate operation, not the
                // constructor's supplied 2026-12-31 end or projected endpoint.
                assert_eq!(
                    controls,
                    "{\"date\":null,\"quoted_months\":0,\"status\":\"null-date\"},{\"date\":\"2027-03-20\",\"quoted_months\":3,\"status\":\"value\"},{\"date\":\"2027-06-20\",\"quoted_months\":6,\"status\":\"value\"},{\"date\":\"2027-12-20\",\"quoted_months\":12,\"status\":\"value\"}"
                );
                assert_eq!(row.parse::<Date>("end")?, "2026-12-31".parse::<Date>()?);
                december_control += 1;
            }
            maturity_rows += 1;
        }
    }
    assert_eq!((cutoff.len(), maturity_rows, december_control), (40, 36, 1));
    for month in [3, 6, 9, 12] {
        let roll = Date::from_ymd(2026, month, 20)?;
        for distance in [0, 1, 29, 30, 31] {
            let old = &cutoff[&(roll, distance, "OldCDS".to_owned())];
            let quarter = &cutoff[&(roll, distance, "TwentiethIMM".to_owned())];
            assert_eq!(old[0], quarter[0]);
            let mut expected = quarter.clone();
            if matches!(distance, 1 | 29) {
                assert_eq!(expected[1], roll);
                expected.remove(1);
            }
            assert_eq!(*old, expected, "roll {roll}, distance {distance}");
        }
    }
    // This pins observed cutoff behavior for four 2026 quarters only. It
    // neither supplies a market contract nor admits OldCDS to the local API.
    Ok(())
}
