//! Scoped public-reference comparisons and exact declared schedule distinctions.

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth, TargetCalendar};
use sabiquant::schedule::{
    DateGenerationRule, PeriodRegularity, ScheduleBuilder, ScheduleCalendar, ScheduleError,
    StubPolicy,
};
use sabiquant::time::{Date, Period};
use support::fixture::{Fixture, Record};

// The frozen probe generator writes compact deterministic JSON. These helpers
// decode only its two flat array fields, not general JSON or production data.
// Strict date/bool parsing and one-key checks reject an unexpected shape.
#[expect(
    clippy::unwrap_used,
    reason = "test-only decoder rejects a malformed frozen reference field"
)]
fn array<'a>(row: &Record<'a>, key: &str) -> &'a str {
    let json = row.text("public_observation").unwrap();
    let marker = format!("\"{key}\":[");
    assert_eq!(json.matches(&marker).count(), 1, "{} {key}", row.id());
    json.split_once(&marker)
        .unwrap()
        .1
        .split_once(']')
        .unwrap()
        .0
}

#[expect(
    clippy::unwrap_used,
    reason = "test-only decoder requires exact quoted ISO reference dates"
)]
fn reference_dates(row: &Record<'_>) -> Vec<Date> {
    array(row, "dates")
        .split(',')
        .map(|field| {
            field
                .strip_prefix('"')
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
    reason = "test-only decoder rejects an unexpected reference boolean"
)]
fn reference_regularity(row: &Record<'_>) -> Vec<PeriodRegularity> {
    array(row, "regularity")
        .split(',')
        .map(|field| match field {
            "true" => PeriodRegularity::Regular,
            "false" => PeriodRegularity::Stub,
            other => panic!("unexpected reference regularity {other}"),
        })
        .collect()
}

#[expect(
    clippy::unwrap_used,
    reason = "hand-audited literal schedule expectations use valid ISO dates"
)]
fn dates(text: &str) -> Vec<Date> {
    text.split(';').map(|date| date.parse().unwrap()).collect()
}

#[test]
fn supported_reference_inputs_have_only_the_three_declared_distinctions() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/probes.fixture",
    ))
    .unwrap();
    assert_eq!(fixture.record_count(), 46);
    let mut shared = 0;
    let mut eom_distinctions = 0;
    let mut collisions = 0;
    let mut zero_metadata = 0;
    let mut deferred = 0;
    for row in fixture.records() {
        let json = row.text("public_observation").unwrap();
        assert!(json.contains("\"runtime_version\":\"1.43\""));
        assert_eq!(row.text("status").unwrap(), "value");
        let reference = reference_dates(&row);
        let flags = reference_regularity(&row);
        assert!(reference.len() >= 2 && reference.len() <= 400);
        assert!(reference.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(flags.len() + 1, reference.len());
        let rule_text = row.text("rule").unwrap();
        let rule: DateGenerationRule = match rule_text.parse() {
            Ok(rule) => rule,
            Err(error) => {
                assert_eq!(error.kind(), "identifier.unknown");
                assert!(matches!(
                    rule_text,
                    "ThirdWednesday"
                        | "ThirdWednesdayInclusive"
                        | "Twentieth"
                        | "TwentiethIMM"
                        | "OldCDS"
                        | "CDS"
                        | "CDS2015"
                ));
                deferred += 1;
                continue;
            }
        };
        if !row.text("first_coupon_date").unwrap().is_empty()
            || !row.text("penultimate_coupon_date").unwrap().is_empty()
        {
            assert_eq!(row.text("group").unwrap(), "stubs");
            deferred += 1;
            continue;
        }
        let calendar = match row.text("calendar").unwrap() {
            "Null" => ScheduleCalendar::Plain(Calendar::null()),
            "Weekends" => ScheduleCalendar::Plain(Calendar::weekends_only()),
            "TARGET" => ScheduleCalendar::Target(TargetCalendar::new()),
            other => panic!("unclassified reference calendar {other}"),
        };
        let builder = ScheduleBuilder::new(
            row.parse("start").unwrap(),
            row.parse("end").unwrap(),
            calendar,
            row.parse::<BusinessDayConvention>("convention").unwrap(),
            row.parse::<BusinessDayConvention>("termination").unwrap(),
        )
        .unwrap();
        let result = if rule == DateGenerationRule::Zero {
            builder.build_zero(400)
        } else {
            builder.build_recurrent(
                Period::new(
                    row.parse("tenor_length").unwrap(),
                    row.parse("tenor_unit").unwrap(),
                ),
                rule,
                StubPolicy::Short,
                if row.parse::<bool>("end_of_month").unwrap() {
                    EndOfMonth::Preserve
                } else {
                    EndOfMonth::Ignore
                },
                400,
            )
        };
        if matches!(
            row.id(),
            "COLLISION-DAILY-FOLLOWING"
                | "COLLISION-DAILY-PRECEDING"
                | "COLLISION-DAILY-MODIFIEDFOLLOWING"
                | "TARGET-EASTER-DAILY"
        ) {
            let error = result.unwrap_err();
            let ScheduleError::AdjustedOrder {
                index,
                previous_contractual,
                contractual,
                previous_adjusted,
                adjusted,
                ..
            } = error
            else {
                panic!("{} expected contextual collision, got {error}", row.id());
            };
            assert_eq!(previous_adjusted, adjusted, "{}", row.id());
            assert!(previous_contractual < contractual);
            let (expected_index, expected_raw, expected_adjusted) = match row.id() {
                "COLLISION-DAILY-PRECEDING" => (1, "2026-01-10", "2026-01-09"),
                "TARGET-EASTER-DAILY" => (2, "2026-04-04", "2026-04-07"),
                _ => (2, "2026-01-11", "2026-01-12"),
            };
            assert_eq!(index, expected_index, "{}", row.id());
            assert_eq!(contractual.to_string(), expected_raw);
            assert_eq!(adjusted.to_string(), expected_adjusted);
            assert!(
                reference.len()
                    < if row.id() == "TARGET-EASTER-DAILY" {
                        6
                    } else {
                        5
                    }
            );
            collisions += 1;
            continue;
        }
        let schedule = result.unwrap_or_else(|error| panic!("{}: {error}", row.id()));
        let distinct_eom = match row.id() {
            "EOM-FORWARD-2026-01-30-PRESERVE-UNADJUSTED" => {
                Some("2026-01-30;2026-02-28;2026-03-30;2026-04-30;2026-05-30;2026-06-30")
            }
            "EOM-FORWARD-2026-01-30-PRESERVE-FOLLOWING" => {
                Some("2026-01-30;2026-03-02;2026-03-30;2026-04-30;2026-06-01;2026-06-30")
            }
            "EOM-BACKWARD-2026-05-29-PRESERVE-UNADJUSTED" => {
                Some("2026-01-01;2026-01-29;2026-02-28;2026-03-29;2026-04-29;2026-05-29")
            }
            "EOM-BACKWARD-2026-05-29-PRESERVE-FOLLOWING" => {
                Some("2026-01-01;2026-01-29;2026-03-02;2026-03-30;2026-04-29;2026-05-29")
            }
            _ => None,
        };
        if let Some(expected) = distinct_eom {
            assert_eq!(schedule.adjusted_dates(), dates(expected), "{}", row.id());
            assert_ne!(schedule.adjusted_dates(), reference, "{}", row.id());
            assert_eq!(schedule.regularity(), flags, "{}", row.id());
            eom_distinctions += 1;
        } else if rule == DateGenerationRule::Zero {
            assert_eq!(schedule.adjusted_dates(), reference);
            assert_eq!(flags, [PeriodRegularity::Regular]);
            assert_eq!(schedule.regularity(), [PeriodRegularity::NonPeriodic]);
            assert_eq!(schedule.tenor(), None);
            assert_eq!(schedule.stub_policy(), None);
            assert_eq!(schedule.end_of_month(), None);
            assert!(json.contains("\"tenor_length\":0,\"tenor_unit\":\"Years\""));
            zero_metadata += 1;
        } else {
            assert_eq!(schedule.adjusted_dates(), reference, "{}", row.id());
            assert_eq!(schedule.regularity(), flags, "{}", row.id());
            shared += 1;
        }
    }
    assert_eq!(
        (
            shared,
            eom_distinctions,
            collisions,
            zero_metadata,
            deferred
        ),
        (25, 4, 4, 1, 12)
    );
}
