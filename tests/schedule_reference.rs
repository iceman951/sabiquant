//! Scoped public-reference comparisons and exact declared schedule distinctions.

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth, TargetCalendar};
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, PeriodRegularity, ScheduleBuilder, ScheduleCalendar,
    ScheduleError, StubPolicy,
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
fn supported_reference_inputs_have_only_the_four_declared_distinctions() {
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
    let mut explicit_shared = 0;
    let mut roll_regularity = 0;
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
                assert!(matches!(rule_text, "OldCDS" | "CDS" | "CDS2015"));
                deferred += 1;
                continue;
            }
        };
        let first_text = row.text("first_coupon_date").unwrap();
        let penultimate_text = row.text("penultimate_coupon_date").unwrap();
        let explicit = if first_text.is_empty() && penultimate_text.is_empty() {
            None
        } else {
            assert_eq!(row.text("group").unwrap(), "stubs");
            Some(
                ExplicitStubDates::new(
                    (!first_text.is_empty()).then(|| first_text.parse().unwrap()),
                    (!penultimate_text.is_empty()).then(|| penultimate_text.parse().unwrap()),
                )
                .unwrap(),
            )
        };
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
        } else if let DateGenerationRule::Roll(roll) = rule {
            builder.build_roll(
                Period::new(
                    row.parse("tenor_length").unwrap(),
                    row.parse("tenor_unit").unwrap(),
                ),
                roll,
                400,
            )
        } else if let Some(explicit) = explicit {
            builder.build_explicit(
                Period::new(
                    row.parse("tenor_length").unwrap(),
                    row.parse("tenor_unit").unwrap(),
                ),
                rule,
                explicit,
                if row.parse::<bool>("end_of_month").unwrap() {
                    EndOfMonth::Preserve
                } else {
                    EndOfMonth::Ignore
                },
                400,
            )
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
        } else if rule_text == "ThirdWednesday" {
            assert_eq!(schedule.adjusted_dates(), reference, "{}", row.id());
            assert_eq!(
                flags,
                [
                    PeriodRegularity::Regular,
                    PeriodRegularity::Regular,
                    PeriodRegularity::Stub
                ]
            );
            assert_eq!(
                schedule.regularity(),
                [
                    PeriodRegularity::Stub,
                    PeriodRegularity::Regular,
                    PeriodRegularity::Stub
                ]
            );
            assert!(schedule.roll_provenance().is_some());
            roll_regularity += 1;
        } else {
            assert_eq!(schedule.adjusted_dates(), reference, "{}", row.id());
            assert_eq!(schedule.regularity(), flags, "{}", row.id());
            assert_eq!(schedule.explicit_stub_dates(), explicit);
            if explicit.is_some() {
                assert_eq!(schedule.stub_policy(), None);
                explicit_shared += 1;
            }
            shared += 1;
        }
    }
    assert_eq!(
        (
            shared,
            eom_distinctions,
            collisions,
            zero_metadata,
            deferred,
            roll_regularity
        ),
        (33, 4, 4, 1, 3, 1)
    );
    assert_eq!(explicit_shared, 5);
}

#[test]
fn explicit_probe_values_errors_and_named_lattice_differences_are_all_classified() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/explicit_probes.fixture",
    ))
    .unwrap();
    assert_eq!(fixture.record_count(), 29);
    assert_eq!(fixture.header("quantlib-module-files"), Some("3"));
    assert_eq!(
        fixture.header("quantlib-module-manifest-sha256"),
        Some("e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d")
    );
    let mut shared = 0;
    let mut reversed = 0;
    let mut off_lattice = 0;
    let mut range_rejections = 0;
    let mut reference_range_errors = 0;
    let mut clamp_flags = 0;
    let mut business_eom = 0;
    for row in fixture.records() {
        let json = row.text("public_observation").unwrap();
        assert!(json.contains("\"runtime_version\":\"1.43\""));
        let start: Date = row.parse("start").unwrap();
        let end: Date = row.parse("end").unwrap();
        let rule: DateGenerationRule = row.parse("rule").unwrap();
        let first_text = row.text("first_coupon_date").unwrap();
        let penultimate_text = row.text("penultimate_coupon_date").unwrap();
        let first = (!first_text.is_empty()).then(|| first_text.parse::<Date>().unwrap());
        let penultimate =
            (!penultimate_text.is_empty()).then(|| penultimate_text.parse::<Date>().unwrap());
        if row.text("group").unwrap() == "reversed-roles" {
            assert_eq!(row.text("status").unwrap(), "value");
            let reference = reference_dates(&row);
            assert_eq!(reference.len(), 4);
            assert_eq!(
                reference,
                [start, first.unwrap(), penultimate.unwrap(), end]
            );
            assert!(reference.windows(2).any(|pair| pair[0] > pair[1]));
            assert!(json.contains("\"strictly_increasing\":false"));
            let error = ExplicitStubDates::new(first, penultimate).unwrap_err();
            assert!(matches!(error, ScheduleError::ReversedExplicitStubDates {
                first: rejected_first, penultimate: rejected_penultimate, ..
            } if Some(rejected_first) == first && Some(rejected_penultimate) == penultimate));
            reversed += 1;
            continue;
        }
        let explicit = ExplicitStubDates::new(first, penultimate).unwrap();
        let calendar = match row.text("calendar").unwrap() {
            "Null" => Calendar::null(),
            "Weekends" => Calendar::weekends_only(),
            other => panic!("unclassified explicit probe calendar {other}"),
        };
        let builder = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(calendar),
            row.parse("convention").unwrap(),
            row.parse("termination").unwrap(),
        )
        .unwrap();
        let result = builder.build_explicit(
            Period::new(
                row.parse("tenor_length").unwrap(),
                row.parse("tenor_unit").unwrap(),
            ),
            rule,
            explicit,
            if row.parse::<bool>("end_of_month").unwrap() {
                EndOfMonth::Preserve
            } else {
                EndOfMonth::Ignore
            },
            400,
        );
        if row.text("group").unwrap() == "relative-endpoint-domain" {
            let error = result.unwrap_err();
            let (role, date) = first.map_or_else(
                || ("penultimate", penultimate.unwrap()),
                |date| ("first", date),
            );
            assert!(matches!(error, ScheduleError::ExplicitStubOutsideRange {
                role: rejected_role, date: rejected_date, start: rejected_start, end: rejected_end, ..
            } if rejected_role == role && rejected_date == date && rejected_start == start && rejected_end == end));
            if row.text("status").unwrap() == "constructor-error" {
                assert!(json.contains("\"error_type\":\"RuntimeError\""));
                assert!(!json.contains("\"dates\":"));
                reference_range_errors += 1;
            } else {
                assert_eq!(row.text("status").unwrap(), "value");
                let reference = reference_dates(&row);
                let expected = if row.id().ends_with("FIRST-AT-START")
                    || row.id().ends_with("PENULTIMATE-AT-END")
                {
                    dates("2026-01-15;2026-04-15;2026-07-15;2026-10-15")
                } else {
                    dates("2026-01-15;2026-10-15")
                };
                assert_eq!(reference, expected, "{}", row.id());
            }
            range_rejections += 1;
            continue;
        }
        assert_eq!(row.text("status").unwrap(), "value");
        let reference = reference_dates(&row);
        let flags = reference_regularity(&row);
        assert!(reference.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(reference.len(), flags.len() + 1);
        let irregular = match row.id() {
            "EXPLICIT-OFFLATTICE-FORWARD" => Some(("2026-02-17", "2026-09-18", "2026-08-17")),
            "EXPLICIT-OFFLATTICE-BACKWARD" => Some(("2026-09-18", "2026-02-17", "2026-03-18")),
            "EXPLICIT-CALENDAR-EOM-FORWARD-IGNORE" => {
                Some(("2026-02-28", "2026-08-31", "2026-08-28"))
            }
            _ => None,
        };
        if let Some((expected_anchor, expected_endpoint, expected_last)) = irregular {
            let error = result.unwrap_err();
            assert!(matches!(error, ScheduleError::IrregularExplicitMiddle {
                rule: rejected_rule, anchor, endpoint, regular_boundary, ..
            } if rejected_rule == rule && anchor.to_string() == expected_anchor
                && endpoint.to_string() == expected_endpoint && regular_boundary.to_string() == expected_last));
            assert!(flags.contains(&PeriodRegularity::Stub));
            off_lattice += 1;
            continue;
        }
        let schedule = result.unwrap_or_else(|error| panic!("{}: {error}", row.id()));
        assert_eq!(schedule.explicit_stub_dates(), Some(explicit));
        assert_eq!(schedule.stub_policy(), None);
        assert_eq!(
            schedule.lattice_anchor(),
            if rule == DateGenerationRule::Forward {
                Some(first.unwrap_or(start))
            } else {
                Some(penultimate.unwrap_or(end))
            }
        );
        if row.id() == "EXPLICIT-CLAMP-FORWARD" {
            assert_eq!(schedule.adjusted_dates(), reference);
            assert_eq!(
                flags,
                [
                    PeriodRegularity::Regular,
                    PeriodRegularity::Regular,
                    PeriodRegularity::Regular,
                    PeriodRegularity::Stub
                ]
            );
            assert_eq!(
                schedule.regularity(),
                [
                    PeriodRegularity::Stub,
                    PeriodRegularity::Regular,
                    PeriodRegularity::Regular,
                    PeriodRegularity::Stub
                ]
            );
            clamp_flags += 1;
        } else if row.id() == "EXPLICIT-BUSINESS-EOM-FORWARD-PRESERVE"
            || row.id() == "EXPLICIT-BUSINESS-EOM-BACKWARD-PRESERVE"
        {
            let expected = if rule == DateGenerationRule::Forward {
                "2026-01-15;2026-01-30;2026-03-02;2026-03-30;2026-04-30;2026-05-15"
            } else {
                "2026-01-15;2026-01-29;2026-03-02;2026-03-30;2026-04-29;2026-05-29;2026-06-15"
            };
            assert_eq!(schedule.adjusted_dates(), dates(expected), "{}", row.id());
            assert_ne!(schedule.adjusted_dates(), reference);
            assert_eq!(schedule.regularity(), flags);
            business_eom += 1;
        } else {
            assert_eq!(schedule.adjusted_dates(), reference, "{}", row.id());
            assert_eq!(schedule.regularity(), flags, "{}", row.id());
            shared += 1;
        }
    }
    assert_eq!(
        (
            shared,
            reversed,
            off_lattice,
            range_rejections,
            reference_range_errors,
            clamp_flags,
            business_eom
        ),
        (13, 2, 3, 8, 4, 1, 2)
    );
}
