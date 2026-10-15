//! Public roll definitions, guards, errors, resources and coupon composition.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, JointRule, TargetCalendar,
    TargetPolicy,
};
use sabiquant::cashflows::{FixedCouponRate, Leg, PaymentDateInclusion};
use sabiquant::daycount::DayCount;
use sabiquant::error::{Error as RootError, IdentifierError};
use sabiquant::rates::{Compounding, InterestRate};
use sabiquant::schedule::{
    DateGenerationRule as Generation, ExplicitStubDates, PeriodRegularity as Regularity,
    RollProvenance, ScheduleBuilder, ScheduleCalendar, ScheduleError, SpecialRollRule as Roll,
    StubPolicy,
};
use sabiquant::time::{Date, DateError, Frequency, Month, Period, TimeUnit, Weekday};
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[expect(
    clippy::expect_used,
    reason = "literal dates are hand-checked valid ISO dates"
)]
fn d(text: &str) -> Date {
    text.parse().expect("valid literal date")
}

#[expect(
    clippy::panic,
    reason = "a rejected public request must not return a value"
)]
fn rejected<T: Debug>(result: Result<T, ScheduleError>) -> ScheduleError {
    match result {
        Ok(value) => panic!("expected rejection, got {value:?}"),
        Err(error) => error,
    }
}

fn plain(start: Date, end: Date) -> Result<ScheduleBuilder, ScheduleError> {
    ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )
}

fn months(length: i32) -> Period {
    Period::new(length, TimeUnit::Months)
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

fn ordinal(date: Date) -> i64 {
    let (year, month, day) = date.to_ymd();
    let y = i64::from(year - 1);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let lengths = [
        31_u32,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let before: u32 = lengths.into_iter().take((month - 1) as usize).sum();
    365 * y + y / 4 - y / 100 + y / 400 + i64::from(before + day - 1)
}

#[test]
fn canonical_roll_labels_and_truthful_generation_metadata_are_closed() -> Result<(), Box<dyn Error>>
{
    fn choices<T: Copy + Debug + Eq + Hash + Send + Sync + 'static>() {}
    choices::<Roll>();
    choices::<RollProvenance>();
    assert_eq!(Roll::ALL.len(), 4);
    assert_eq!(Generation::ALL.len(), 10);
    let names = [
        "ThirdWednesday",
        "ThirdWednesdayInclusive",
        "Twentieth",
        "TwentiethIMM",
    ];
    for (rule, name) in Roll::ALL.into_iter().zip(names) {
        assert_eq!(rule.canonical_identifier(), name);
        assert_eq!(rule.to_string(), name);
        assert_eq!(name.parse::<Roll>()?, rule);
        assert_eq!(name.parse::<Generation>()?, Generation::Roll(rule));
        assert_eq!(Generation::Roll(rule).to_string(), name);
        for text in [
            format!(" {name}"),
            format!("{name} "),
            name.to_lowercase(),
            format!("Roll({name})"),
        ] {
            assert!(
                matches!(text.parse::<Roll>(), Err(IdentifierError::Unknown { text: t, .. }) if t==text)
            );
            assert!(text.parse::<Generation>().is_err());
        }
    }
    for text in ["IMM", "CDS", "CDS2015", "OldCDS", "Forward", ""] {
        assert!(text.parse::<Roll>().is_err());
    }
    // OldCDS is a distinct automatic generation label, never a civil roll.
    assert_eq!("OldCDS".parse::<Generation>()?, Generation::OldCds);
    let set: HashSet<_> = Generation::ALL.into_iter().collect();
    assert_eq!(set.len(), 10);
    Ok(())
}

#[test]
fn declared_hand_vectors_separate_requested_contractual_and_lattice_flags()
-> Result<(), Box<dyn Error>> {
    let start = d("2026-02-06");
    let end = d("2026-11-17");
    for (rule, raw, flags, anchor) in [
        (
            Roll::ThirdWednesday,
            vec![start, d("2026-05-20"), d("2026-08-19"), end],
            vec![Regularity::Stub, Regularity::Regular, Regularity::Stub],
            Month::February,
        ),
        (
            Roll::ThirdWednesdayInclusive,
            vec![
                d("2026-02-18"),
                d("2026-05-20"),
                d("2026-08-19"),
                d("2026-11-18"),
            ],
            vec![Regularity::Regular; 3],
            Month::February,
        ),
        (
            Roll::Twentieth,
            vec![
                start,
                d("2026-02-20"),
                d("2026-05-20"),
                d("2026-08-20"),
                d("2026-11-20"),
            ],
            vec![
                Regularity::Stub,
                Regularity::Regular,
                Regularity::Regular,
                Regularity::Regular,
            ],
            Month::February,
        ),
        (
            Roll::TwentiethIMM,
            vec![
                start,
                d("2026-03-20"),
                d("2026-06-20"),
                d("2026-09-20"),
                d("2026-12-20"),
            ],
            vec![
                Regularity::Stub,
                Regularity::Regular,
                Regularity::Regular,
                Regularity::Regular,
            ],
            Month::March,
        ),
    ] {
        let cap = u32::try_from(raw.len())?;
        let schedule = plain(start, end)?.build_roll(months(3), rule, cap)?;
        assert_eq!(schedule.contractual_dates(), raw);
        assert_eq!(schedule.adjusted_dates(), raw);
        assert_eq!(schedule.regularity(), flags);
        assert_eq!(schedule.generation_rule(), Generation::Roll(rule));
        assert_eq!(schedule.tenor(), Some(months(3)));
        assert_eq!(schedule.stub_policy(), None);
        assert_eq!(schedule.end_of_month(), None);
        assert_eq!(schedule.explicit_stub_dates(), None);
        assert_eq!(schedule.lattice_anchor(), None);
        let p = schedule
            .roll_provenance()
            .ok_or("missing roll provenance")?;
        assert_eq!(p.requested_start(), start);
        assert_eq!(p.requested_end(), end);
        assert_eq!(p.resolved_start(), raw[0]);
        assert_eq!(p.resolved_end(), raw[raw.len() - 1]);
        assert_eq!(p.rule(), rule);
        assert_eq!(p.tenor().length(), 3);
        assert_eq!(p.tenor().unit(), TimeUnit::Months);
        assert_eq!(p.anchor_year(), 2026);
        assert_eq!(p.anchor_month(), anchor);
        assert_eq!(
            schedule,
            plain(start, end)?.build_roll(months(3), rule, cap + 10)?
        );
        assert!(matches!(plain(start,end)?.build_roll(months(3),rule,cap-1),
            Err(ScheduleError::BoundaryLimitExceeded { maximum,required_at_least,.. })
            if maximum==cap-1 && required_at_least==cap));
    }
    Ok(())
}

#[test]
fn month_rolls_are_not_repeated_clamped_dates_or_implicit_front_rolls() -> Result<(), Box<dyn Error>>
{
    let b = plain(d("2026-01-01"), d("2026-04-17"))?;
    let s = b.build_roll(months(1), Roll::ThirdWednesday, 5)?;
    assert_eq!(
        s.contractual_dates(),
        [
            "2026-01-01",
            "2026-02-18",
            "2026-03-18",
            "2026-04-15",
            "2026-04-17"
        ]
        .map(d)
    );
    assert_eq!(
        s.regularity(),
        [
            Regularity::Stub,
            Regularity::Regular,
            Regularity::Regular,
            Regularity::Stub
        ]
    );
    let same_month =
        plain(d("2026-01-01"), d("2026-01-31"))?.build_roll(months(1), Roll::ThirdWednesday, 2)?;
    assert_eq!(
        same_month.contractual_dates(),
        [d("2026-01-01"), d("2026-01-31")]
    );
    assert_eq!(same_month.regularity(), [Regularity::Stub]);
    let exact =
        plain(d("2026-01-21"), d("2026-04-15"))?.build_roll(months(1), Roll::ThirdWednesday, 4)?;
    assert_eq!(
        exact.contractual_dates(),
        ["2026-01-21", "2026-02-18", "2026-03-18", "2026-04-15"].map(d)
    );
    assert_eq!(exact.regularity(), [Regularity::Regular; 3]);
    Ok(())
}

#[test]
fn inclusive_same_month_collapse_retains_both_ranges_and_original_tenor()
-> Result<(), Box<dyn Error>> {
    for (start, end) in [
        (d("2026-01-01"), d("2026-01-02")),
        (d("2026-01-22"), d("2026-01-31")),
    ] {
        let error = rejected(plain(start, end)?.build_roll(
            Period::new(1, TimeUnit::Years),
            Roll::ThirdWednesdayInclusive,
            2,
        ));
        assert_eq!(error.kind(), "schedule.resolved_roll_range");
        assert!(
            matches!(error,ScheduleError::ResolvedRollRange { rule:Roll::ThirdWednesdayInclusive,tenor,
            requested_start:a,requested_end:b,resolved_start:c,resolved_end:e,.. }
            if a==start && b==end && c==d("2026-01-21") && e==c && tenor.length()==1 && tenor.unit()==TimeUnit::Years)
        );
        assert!(error.source().is_none());
    }
    assert!(matches!(
        plain(d("2026-01-02"), d("2026-01-01")),
        Err(ScheduleError::InvalidRange { .. })
    ));
    Ok(())
}

#[test]
fn twentieth_endpoint_projection_is_independent_of_stride_and_day_twenty_is_inclusive()
-> Result<(), Box<dyn Error>> {
    for (rule, tenor, start, end, raw, flags, anchor) in [
        (
            Roll::Twentieth,
            months(3),
            d("2026-02-06"),
            d("2026-04-01"),
            vec![d("2026-02-06"), d("2026-02-20"), d("2026-04-20")],
            vec![Regularity::Stub; 2],
            Month::February,
        ),
        (
            Roll::Twentieth,
            months(3),
            d("2026-02-21"),
            d("2026-05-19"),
            vec![d("2026-02-21"), d("2026-03-20"), d("2026-05-20")],
            vec![Regularity::Stub; 2],
            Month::March,
        ),
        (
            Roll::TwentiethIMM,
            months(6),
            d("2026-03-20"),
            d("2026-06-19"),
            vec![d("2026-03-20"), d("2026-06-20")],
            vec![Regularity::Stub],
            Month::March,
        ),
        (
            Roll::TwentiethIMM,
            months(6),
            d("2026-03-21"),
            d("2026-12-20"),
            vec![d("2026-03-21"), d("2026-06-20"), d("2026-12-20")],
            vec![Regularity::Stub, Regularity::Regular],
            Month::June,
        ),
    ] {
        let s = plain(start, end)?.build_roll(tenor, rule, u32::try_from(raw.len())?)?;
        assert_eq!(s.contractual_dates(), raw);
        assert_eq!(s.regularity(), flags);
        assert_eq!(
            s.roll_provenance().ok_or("provenance")?.anchor_month(),
            anchor
        );
    }
    Ok(())
}

#[test]
fn roll_domain_and_old_builder_rejection_order_precede_context_and_roles()
-> Result<(), Box<dyn Error>> {
    let b = ScheduleBuilder::new(
        d("2025-12-30"),
        d("2026-01-01"),
        ScheduleCalendar::Target(TargetCalendar::new()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?;
    let roles = ExplicitStubDates::new(Some(Date::MAX), None)?;
    for rule in Roll::ALL {
        for unit in [
            TimeUnit::Days,
            TimeUnit::Weeks,
            TimeUnit::Months,
            TimeUnit::Years,
        ] {
            for length in [0, -1, i32::MIN] {
                let p = Period::new(length, unit);
                assert!(
                    matches!(b.build_roll(p,rule,2),Err(ScheduleError::InvalidRollTenor { rule:r,tenor,.. })
                    if r==rule && tenor.length()==length && tenor.unit()==unit)
                );
            }
        }
        for unit in [TimeUnit::Days, TimeUnit::Weeks] {
            assert!(matches!(
                b.build_roll(Period::new(1, unit), rule, 2),
                Err(ScheduleError::InvalidRollTenor { .. })
            ));
        }
        let selected = Generation::Roll(rule);
        assert!(
            matches!(b.build_recurrent(months(0),selected,StubPolicy::Long,EndOfMonth::Preserve,2),
            Err(ScheduleError::InvalidRule { rule:r,.. }) if r==selected)
        );
        assert!(
            matches!(b.build_explicit(months(0),selected,roles,EndOfMonth::Preserve,2),
            Err(ScheduleError::InvalidRule { rule:r,.. }) if r==selected)
        );
        for cap in [0, 1, ScheduleBuilder::MAX_BOUNDARIES + 1, u32::MAX] {
            assert!(
                matches!(b.build_roll(months(0),rule,cap),Err(ScheduleError::InvalidBoundaryLimit {requested,..}) if requested==cap)
            );
            assert!(matches!(
                b.build_recurrent(
                    months(0),
                    selected,
                    StubPolicy::None,
                    EndOfMonth::Ignore,
                    cap
                ),
                Err(ScheduleError::InvalidBoundaryLimit { .. })
            ));
            assert!(matches!(
                b.build_explicit(months(0), selected, roles, EndOfMonth::Ignore, cap),
                Err(ScheduleError::InvalidBoundaryLimit { .. })
            ));
        }
    }
    for length in [1, 2, 4, 5, i32::MAX] {
        assert!(matches!(
            b.build_roll(months(length), Roll::TwentiethIMM, 2),
            Err(ScheduleError::InvalidRollTenor { .. })
        ));
    }
    Ok(())
}

#[test]
fn rolled_icma_validates_cap_then_explicit_frequency() -> Result<(), Box<dyn Error>> {
    for rule in Roll::ALL {
        let s = plain(d("2026-02-06"), d("2026-11-17"))?.build_roll(months(3), rule, 5)?;
        for frequency in Frequency::ALL {
            match frequency {
                Frequency::Once => assert!(matches!(
                    s.build_icma_context(frequency, 8),
                    Err(ScheduleError::InvalidIcmaFrequency { .. })
                )),
                Frequency::Quarterly => {
                    assert!(s.build_icma_context(frequency, 8).is_ok());
                    assert!(matches!(
                        s.build_icma_context(frequency, 1),
                        Err(ScheduleError::IcmaPeriodLimitExceeded { .. })
                    ));
                }
                _ => assert!(matches!(
                    s.build_icma_context(frequency, 8),
                    Err(ScheduleError::IcmaFrequencyMismatch { .. })
                )),
            }
            assert!(matches!(
                s.build_icma_context(frequency, 0),
                Err(ScheduleError::InvalidIcmaPeriodLimit { .. })
            ));
        }
    }
    let b = plain(d("2026-01-01"), d("2027-01-01"))?;
    assert!(b.build_zero(2)?.roll_provenance().is_none());
    assert!(
        b.build_recurrent(
            months(12),
            Generation::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            2
        )?
        .roll_provenance()
        .is_none()
    );
    assert!(
        b.build_explicit(
            months(6),
            Generation::Forward,
            ExplicitStubDates::new(Some(d("2026-07-01")), None)?,
            EndOfMonth::Ignore,
            3
        )?
        .roll_provenance()
        .is_none()
    );
    Ok(())
}

#[test]
fn required_civil_overflow_retains_role_input_actual_signed_date_cause_and_root_source()
-> Result<(), Box<dyn Error>> {
    for (rule, tenor, count) in [
        (Roll::Twentieth, months(1), 1),
        (Roll::TwentiethIMM, months(3), 3),
    ] {
        for (start, role, input) in [
            (d("9999-12-30"), "anchor", d("9999-12-30")),
            (d("9999-12-01"), "end", Date::MAX),
        ] {
            let error = rejected(plain(start, Date::MAX)?.build_roll(tenor, rule, 2));
            assert_eq!(error.kind(), "schedule.roll_date");
            assert!(
                matches!(&error,ScheduleError::RollDate { rule:r,tenor:p,role:q,input:i,
                cause:DateError::OutOfRange { date,operation:"add months",count:n,.. },.. }
                if *r==rule && p.length()==tenor.length() && *q==role && *i==input && *date==input && *n==count)
            );
            let root = RootError::from(error.clone());
            assert_eq!(
                root.source()
                    .and_then(|e| e.downcast_ref::<ScheduleError>()),
                Some(&error)
            );
            assert!(
                root.source()
                    .and_then(|e| e.source())
                    .is_some_and(|e| e.is::<DateError>())
            );
        }
    }
    Ok(())
}

#[test]
fn unused_huge_next_months_stop_without_false_endpoint_clamping() -> Result<(), Box<dyn Error>> {
    for rule in Roll::ALL {
        let end = if matches!(rule, Roll::Twentieth | Roll::TwentiethIMM) {
            d("9999-12-20")
        } else {
            Date::MAX
        };
        let expected_count = if matches!(rule, Roll::Twentieth | Roll::TwentiethIMM) {
            3
        } else {
            2
        };
        let s = plain(Date::MIN, end)?.build_roll(
            Period::new(i32::MAX, TimeUnit::Years),
            rule,
            expected_count,
        )?;
        assert_eq!(s.contractual_dates().len(), expected_count as usize);
        assert!(s.regularity().iter().all(|r| *r == Regularity::Stub));
        let p = s.roll_provenance().ok_or("provenance")?;
        assert_eq!(p.tenor().length(), i32::MAX);
        assert_eq!(p.tenor().unit(), TimeUnit::Years);
        assert_eq!(p.requested_start(), Date::MIN);
        assert_eq!(p.requested_end(), end);
        assert_eq!(
            p.resolved_end(),
            if rule == Roll::ThirdWednesdayInclusive {
                d("9999-12-15")
            } else {
                end
            }
        );
        assert!(matches!(
            plain(Date::MIN, end)?.build_roll(
                Period::new(i32::MAX, TimeUnit::Years),
                rule,
                expected_count - 1
            ),
            Err(ScheduleError::InvalidBoundaryLimit { .. }
                | ScheduleError::BoundaryLimitExceeded { .. })
        ));
    }
    Ok(())
}

#[test]
fn requested_ranges_and_stored_aliases_remain_distinct_hashable_definitions()
-> Result<(), Box<dyn Error>> {
    let a = plain(d("2026-02-01"), d("2026-11-01"))?.build_roll(
        months(3),
        Roll::ThirdWednesdayInclusive,
        4,
    )?;
    let b = plain(d("2026-02-02"), d("2026-11-02"))?.build_roll(
        months(3),
        Roll::ThirdWednesdayInclusive,
        4,
    )?;
    assert_eq!(a.contractual_dates(), b.contractual_dates());
    assert_eq!(a.adjusted_dates(), b.adjusted_dates());
    assert_ne!(a, b);
    assert_ne!(a.roll_provenance(), b.roll_provenance());
    let same = plain(d("2026-02-01"), d("2026-11-01"))?.build_roll(
        months(3),
        Roll::ThirdWednesdayInclusive,
        100,
    )?;
    assert_eq!(a, same);
    assert_eq!(hash(&a), hash(&same));
    assert_eq!(HashSet::from([a.clone(), b.clone(), same]).len(), 2);
    assert_eq!(
        HashSet::from([
            *a.roll_provenance().ok_or("p")?,
            *b.roll_provenance().ok_or("p")?
        ])
        .len(),
        2
    );
    for rule in Roll::ALL {
        let b = plain(d("2026-03-20"), d("2028-03-20"))?;
        let monthly = b.build_roll(months(12), rule, 5)?;
        let yearly = b.build_roll(Period::new(1, TimeUnit::Years), rule, 5)?;
        assert_eq!(months(12), Period::new(1, TimeUnit::Years));
        assert_eq!(monthly.contractual_dates(), yearly.contractual_dates());
        assert_ne!(monthly, yearly);
        assert_ne!(monthly.roll_provenance(), yearly.roll_provenance());
        assert_eq!(HashSet::from([monthly, yearly]).len(), 2);
    }
    Ok(())
}

fn adjusted_literal(convention: Bdc, index: usize) -> Date {
    let dates = match index {
        0 => [
            "2026-01-31",
            "2026-02-02",
            "2026-01-30",
            "2026-01-30",
            "2026-01-30",
            "2026-01-30",
            "2026-01-30",
        ],
        1 => [
            "2026-02-20",
            "2026-02-23",
            "2026-02-23",
            "2026-02-19",
            "2026-02-19",
            "2026-02-23",
            "2026-02-19",
        ],
        2 => [
            "2026-03-20",
            "2026-03-23",
            "2026-03-23",
            "2026-03-19",
            "2026-03-19",
            "2026-03-23",
            "2026-03-19",
        ],
        3 => [
            "2026-04-20",
            "2026-04-21",
            "2026-04-21",
            "2026-04-17",
            "2026-04-17",
            "2026-04-21",
            "2026-04-21",
        ],
        // Nearest explicitly favors Following on an equal-distance tie.
        _ => [
            "2026-05-20",
            "2026-05-21",
            "2026-05-21",
            "2026-05-19",
            "2026-05-19",
            "2026-05-21",
            "2026-05-21",
        ],
    };
    let slot = match convention {
        Bdc::Unadjusted => 0,
        Bdc::Following => 1,
        Bdc::ModifiedFollowing => 2,
        Bdc::Preceding => 3,
        Bdc::ModifiedPreceding => 4,
        Bdc::HalfMonthModifiedFollowing => 5,
        Bdc::Nearest => 6,
    };
    d(dates[slot])
}

#[test]
fn all_49_adjustment_pairs_use_ordinary_then_terminal_rules_without_changing_raw_flags()
-> Result<(), Box<dyn Error>> {
    let raw = [
        "2026-01-31",
        "2026-02-20",
        "2026-03-20",
        "2026-04-20",
        "2026-05-20",
    ]
    .map(d);
    let calendar = Calendar::custom(
        [Weekday::Saturday, Weekday::Sunday],
        raw[1..].iter().copied(),
    );
    let mut count = 0;
    for ordinary in Bdc::ALL {
        for terminal in Bdc::ALL {
            let s = ScheduleBuilder::new(
                raw[0],
                d("2026-05-01"),
                ScheduleCalendar::Plain(calendar.clone()),
                ordinary,
                terminal,
            )?
            .build_roll(months(1), Roll::Twentieth, 5)?;
            assert_eq!(s.contractual_dates(), raw);
            assert_eq!(
                s.regularity(),
                [
                    Regularity::Stub,
                    Regularity::Regular,
                    Regularity::Regular,
                    Regularity::Regular
                ]
            );
            let expected: Vec<_> = (0..5)
                .map(|i| adjusted_literal(if i == 4 { terminal } else { ordinary }, i))
                .collect();
            assert_eq!(s.adjusted_dates(), expected, "{ordinary:?}/{terminal:?}");
            assert_eq!(s.convention(), ordinary);
            assert_eq!(s.termination_convention(), terminal);
            assert_eq!(s.calendar(), &ScheduleCalendar::Plain(calendar.clone()));
            assert_eq!(s.roll_provenance().ok_or("p")?.resolved_end(), raw[4]);
            count += 1;
        }
    }
    assert_eq!(count, 49);
    Ok(())
}

#[test]
fn adjustment_collisions_and_reversals_retain_raw_and_adjusted_boundaries()
-> Result<(), Box<dyn Error>> {
    for (start, holidays, previous, current) in [
        (
            d("2026-01-17"),
            vec![d("2026-01-20")],
            d("2026-01-19"),
            d("2026-01-19"),
        ),
        (
            d("2026-01-18"),
            vec![d("2026-01-19"), d("2026-01-20")],
            d("2026-01-21"),
            d("2026-01-16"),
        ),
    ] {
        let error = rejected(
            ScheduleBuilder::new(
                start,
                d("2026-01-19"),
                ScheduleCalendar::Plain(Calendar::custom(
                    [Weekday::Saturday, Weekday::Sunday],
                    holidays,
                )),
                Bdc::Following,
                Bdc::Preceding,
            )?
            .build_roll(months(1), Roll::Twentieth, 2),
        );
        assert!(
            matches!(error,ScheduleError::AdjustedOrder { index:1,previous_contractual,contractual,
            previous_adjusted,adjusted,.. } if previous_contractual==start && contractual==d("2026-01-20")
            && previous_adjusted==previous && adjusted==current)
        );
    }
    let closed = Calendar::custom(Weekday::ALL, []);
    let s = ScheduleBuilder::new(
        d("2026-02-06"),
        d("2026-11-17"),
        ScheduleCalendar::Plain(closed),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_roll(months(3), Roll::TwentiethIMM, 5)?;
    assert_eq!(s.adjusted_dates(), s.contractual_dates());
    let error = rejected(
        ScheduleBuilder::new(
            d("2026-02-06"),
            d("2026-11-17"),
            ScheduleCalendar::Plain(Calendar::custom(Weekday::ALL, [])),
            Bdc::Following,
            Bdc::Unadjusted,
        )?
        .build_roll(months(3), Roll::TwentiethIMM, 5),
    );
    assert!(matches!(
        error,
        ScheduleError::Calendar {
            index: Some(0),
            phase: "adjustment",
            cause: CalendarError::NoBusinessWeekday,
            ..
        }
    ));
    Ok(())
}

#[test]
fn calendar_search_month_and_date_failures_keep_original_boundary_and_typed_sources()
-> Result<(), Box<dyn Error>> {
    let closed_month = Calendar::custom(
        [Weekday::Saturday, Weekday::Sunday],
        (1..=28)
            .map(|day| Date::from_ymd(2026, 2, day))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let error = rejected(
        ScheduleBuilder::new(
            d("2026-02-01"),
            d("2026-03-01"),
            ScheduleCalendar::Plain(closed_month),
            Bdc::ModifiedFollowing,
            Bdc::Unadjusted,
        )?
        .build_roll(months(1), Roll::ThirdWednesdayInclusive, 2),
    );
    assert!(
        matches!(error,ScheduleError::Calendar { index:Some(0),phase:"adjustment",
        cause:CalendarError::NoBusinessDayInMonth {date,..},.. } if date==d("2026-02-18"))
    );
    let start = d("2026-01-01");
    let holidays = (0..366)
        .map(|n| start.checked_add_days(n))
        .collect::<Result<Vec<_>, _>>()?;
    let error = rejected(
        ScheduleBuilder::new(
            start,
            d("2026-03-20"),
            ScheduleCalendar::Plain(Calendar::custom([], holidays)),
            Bdc::Following,
            Bdc::Unadjusted,
        )?
        .build_roll(months(12), Roll::Twentieth, 3),
    );
    assert!(matches!(error,ScheduleError::Calendar { index:Some(0),
        cause:CalendarError::SearchLimit {start:s,direction:1,rejected:366,..},.. } if s==start));
    for (start, end, ordinary, terminal, index, holiday) in [
        (
            Date::MIN,
            Date::MIN.checked_add_days(1)?,
            Bdc::Preceding,
            Bdc::Unadjusted,
            0,
            Date::MIN,
        ),
        (
            Date::MAX.checked_sub_days(1)?,
            Date::MAX,
            Bdc::Unadjusted,
            Bdc::Following,
            1,
            Date::MAX,
        ),
    ] {
        let error = rejected(
            ScheduleBuilder::new(
                start,
                end,
                ScheduleCalendar::Plain(Calendar::null().with_holiday(holiday)),
                ordinary,
                terminal,
            )?
            .build_roll(
                Period::new(i32::MAX, TimeUnit::Years),
                Roll::ThirdWednesday,
                2,
            ),
        );
        assert!(
            matches!(&error,ScheduleError::Calendar {index:Some(i),phase:"adjustment",cause:CalendarError::Date(_),..} if *i==index)
        );
        let root = RootError::from(error);
        assert!(
            root.source()
                .and_then(|e| e.source())
                .is_some_and(|e| e.is::<CalendarError>())
        );
        assert!(
            root.source()
                .and_then(|e| e.source())
                .and_then(|e| e.source())
                .is_some_and(|e| e.is::<DateError>())
        );
    }
    Ok(())
}

#[test]
fn target_original_span_checks_survive_projection_unadjusted_overrides_and_joints()
-> Result<(), Box<dyn Error>> {
    let strict = TargetCalendar::new();
    let mut calendars = vec![strict.clone(), strict.without_holiday(d("2003-01-01"))];
    for joint in JointRule::ALL {
        calendars.push(strict.joint_with(vec![Calendar::null()], joint)?);
    }
    for calendar in calendars {
        for rule in Roll::ALL {
            let error = rejected(
                ScheduleBuilder::new(
                    d("2002-12-31"),
                    d("2026-01-01"),
                    ScheduleCalendar::Target(calendar.clone()),
                    Bdc::Unadjusted,
                    Bdc::Unadjusted,
                )?
                .build_roll(Period::new(i32::MAX, TimeUnit::Years), rule, 2),
            );
            assert!(
                matches!(error,ScheduleError::Calendar { index:None,phase:"coverage",
                cause:CalendarError::UnverifiedCoverage {date,..},.. } if date==d("2003-01-01"))
            );
        }
    }
    for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
        let error = rejected(
            ScheduleBuilder::new(
                d("1999-01-03"),
                d("1999-02-01"),
                ScheduleCalendar::Target(
                    TargetCalendar::with_policy(policy).without_holiday(d("1999-01-03")),
                ),
                Bdc::Unadjusted,
                Bdc::Unadjusted,
            )?
            .build_roll(months(3), Roll::TwentiethIMM, 2),
        );
        assert!(
            matches!(error,ScheduleError::Calendar {index:None,phase:"coverage",
            cause:CalendarError::OutsideValidity {date,..},..} if date==d("1999-01-03"))
        );
    }
    let projected = ScheduleBuilder::new(
        d("2002-12-31"),
        d("2026-01-01"),
        ScheduleCalendar::Target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_roll(
        Period::new(i32::MAX, TimeUnit::Years),
        Roll::ThirdWednesday,
        2,
    )?;
    assert_eq!(
        projected.contractual_dates(),
        [d("2002-12-31"), d("2026-01-01")]
    );
    Ok(())
}

#[test]
fn target_resolved_union_and_adjustment_candidates_have_distinct_failure_phases()
-> Result<(), Box<dyn Error>> {
    for (rule, tenor, terminal) in [
        (Roll::Twentieth, months(1), d("2029-01-20")),
        (Roll::TwentiethIMM, months(3), d("2029-03-20")),
    ] {
        let calendar = TargetCalendar::new()
            .without_holiday(terminal)
            .joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)?;
        let error = rejected(
            ScheduleBuilder::new(
                d("2028-12-01"),
                Date::from_ymd(2028, 12, 31)?,
                ScheduleCalendar::Target(calendar),
                Bdc::Unadjusted,
                Bdc::Unadjusted,
            )?
            .build_roll(tenor, rule, 2),
        );
        assert!(
            matches!(error,ScheduleError::Calendar {index:None,phase:"roll coverage",
            cause:CalendarError::UnverifiedCoverage {date,..},..} if date==terminal)
        );
        let s = ScheduleBuilder::new(
            d("2028-12-01"),
            d("2028-12-31"),
            ScheduleCalendar::Target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )?
        .build_roll(tenor, rule, 3)?;
        assert_eq!(
            s.roll_provenance().ok_or("p")?.requested_end(),
            d("2028-12-31")
        );
        assert_eq!(s.roll_provenance().ok_or("p")?.resolved_end(), terminal);
    }
    let closed = (20..=31)
        .map(|day| Date::from_ymd(2028, 12, day))
        .collect::<Result<Vec<_>, _>>()?;
    let mut strict = TargetCalendar::new();
    for holiday in closed {
        strict = strict.with_holiday(holiday);
    }
    let error = rejected(
        ScheduleBuilder::new(
            d("2028-12-01"),
            d("2028-12-20"),
            ScheduleCalendar::Target(strict),
            Bdc::Unadjusted,
            Bdc::Following,
        )?
        .build_roll(months(1), Roll::Twentieth, 2),
    );
    assert!(
        matches!(error,ScheduleError::Calendar {index:Some(1),phase:"adjustment",
        cause:CalendarError::UnverifiedCoverage {date,..},..} if date==d("2029-01-01"))
    );
    Ok(())
}

#[test]
fn target_union_keeps_requested_dates_even_when_inclusive_projection_shrinks_them()
-> Result<(), Box<dyn Error>> {
    // Requested end enters an unverified year, although the projected end in
    // that same month could precede the requested day. Original guard wins.
    let error = rejected(
        ScheduleBuilder::new(
            d("2028-12-20"),
            d("2029-01-31"),
            ScheduleCalendar::Target(TargetCalendar::new()),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )?
        .build_roll(months(1), Roll::ThirdWednesdayInclusive, 2),
    );
    assert!(
        matches!(error,ScheduleError::Calendar {index:None,phase:"coverage",
        cause:CalendarError::UnverifiedCoverage {date,..},..} if date==d("2029-01-31"))
    );
    let s = ScheduleBuilder::new(
        d("1999-01-04"),
        d("1999-03-31"),
        ScheduleCalendar::Target(TargetCalendar::new()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_roll(months(1), Roll::ThirdWednesdayInclusive, 3)?;
    assert_eq!(
        s.contractual_dates(),
        ["1999-01-20", "1999-02-17", "1999-03-17"].map(d)
    );
    assert_eq!(
        s.roll_provenance().ok_or("p")?.requested_start(),
        d("1999-01-04")
    );
    assert_eq!(
        s.roll_provenance().ok_or("p")?.requested_end(),
        d("1999-03-31")
    );
    Ok(())
}

#[test]
fn every_insufficient_interior_or_terminal_cap_precedes_business_day_adjustment()
-> Result<(), Box<dyn Error>> {
    // These counts follow the literal hand vectors above. A closed calendar
    // makes it observable that generation rejects insufficient caps before
    // trying to adjust even the first date.
    for (rule, full_count) in Roll::ALL.into_iter().zip([4_u32, 4, 5, 5]) {
        let builder = ScheduleBuilder::new(
            d("2026-02-06"),
            d("2026-11-17"),
            ScheduleCalendar::Plain(Calendar::custom(Weekday::ALL, [])),
            Bdc::Following,
            Bdc::Following,
        )?;
        for cap in 2..full_count {
            let error = rejected(builder.build_roll(months(3), rule, cap));
            assert_eq!(error.kind(), "schedule.boundary_limit_exceeded");
            assert!(matches!(error,
                ScheduleError::BoundaryLimitExceeded { maximum, required_at_least, .. }
                if maximum == cap && required_at_least == cap + 1));
            assert!(error.source().is_none());
        }
        assert!(matches!(
            builder.build_roll(months(3), rule, full_count),
            Err(ScheduleError::Calendar {
                index: Some(0),
                phase: "adjustment",
                cause: CalendarError::NoBusinessWeekday,
                ..
            })
        ));
    }
    Ok(())
}

#[test]
fn full_supported_civil_range_is_bounded_at_the_exact_real_boundary_count()
-> Result<(), Box<dyn Error>> {
    for (rule, tenor, end, count, stubs) in [
        (
            Roll::ThirdWednesday,
            months(1),
            Date::MAX,
            119_989_u32,
            2_usize,
        ),
        (
            Roll::ThirdWednesdayInclusive,
            months(1),
            Date::MAX,
            119_988,
            0,
        ),
        (Roll::Twentieth, months(1), d("9999-12-20"), 119_989, 1),
        (Roll::TwentiethIMM, months(3), d("9999-12-20"), 39_997, 1),
    ] {
        let b = plain(Date::MIN, end)?;
        let s = b.build_roll(tenor, rule, count)?;
        assert_eq!(s.contractual_dates().len(), count as usize);
        assert_eq!(s.adjusted_dates(), s.contractual_dates());
        assert_eq!(s.regularity().len(), count as usize - 1);
        assert_eq!(
            s.regularity()
                .iter()
                .filter(|x| **x == Regularity::Stub)
                .count(),
            stubs
        );
        assert!(s.regularity().iter().all(|x| *x != Regularity::NonPeriodic));
        assert!(s.contractual_dates().windows(2).all(|p| p[0] < p[1]));
        assert_eq!(s.roll_provenance().ok_or("p")?.requested_start(), Date::MIN);
        assert_eq!(s.roll_provenance().ok_or("p")?.requested_end(), end);
        let first = if rule == Roll::ThirdWednesdayInclusive {
            d("0001-01-17")
        } else {
            Date::MIN
        };
        let last = if rule == Roll::ThirdWednesdayInclusive {
            d("9999-12-15")
        } else {
            end
        };
        assert_eq!(s.contractual_dates()[0], first);
        assert_eq!(s.contractual_dates()[count as usize - 1], last);
        for date in &s.contractual_dates()[1..count as usize - 1] {
            if matches!(rule, Roll::ThirdWednesday | Roll::ThirdWednesdayInclusive) {
                assert!((15..=21).contains(&date.day()));
                assert_eq!(ordinal(*date) % 7, 2); // Independent civil ordinal: Monday=0.
            } else {
                assert_eq!(date.day(), 20);
                if rule == Roll::TwentiethIMM {
                    assert_eq!(date.month() % 3, 0);
                }
            }
        }
        assert!(
            matches!(b.build_roll(tenor,rule,count-1),Err(ScheduleError::BoundaryLimitExceeded {maximum,required_at_least,..})
            if maximum==count-1 && required_at_least==count)
        );
    }
    Ok(())
}

#[test]
fn every_roll_composes_with_explicit_scalar_coupons_and_direct_zero_rate_valuation()
-> Result<(), Box<dyn Error>> {
    let expected_bits = [
        0x3fe8_e60c_9fe9_8e61_u64,
        0x3fe7_ef2a_c897_7ef3,
        0x3fe9_2961_7d8b_9296,
        0x3feb_cab2_25df_bcac,
    ];
    // These four constants independently sum supplied rounded day/365 terms
    // with Python Fraction, then round once. N=8 and r=1/8 make N*r exactly1.
    for (rule, bits) in Roll::ALL.into_iter().zip(expected_bits) {
        let s = ScheduleBuilder::new(
            d("2026-02-06"),
            d("2026-11-17"),
            ScheduleCalendar::Plain(Calendar::weekends_only()),
            Bdc::Following,
            Bdc::ModifiedFollowing,
        )?
        .build_roll(months(3), rule, 5)?;
        let quote = FixedCouponRate::new(0.125, DayCount::Actual365Fixed)?;
        let count = u32::try_from(s.contractual_dates().len() - 1)?;
        let leg = Leg::fixed_rate(&s, 8.0, &quote, count)?;
        for (index, flow) in leg.flows().iter().enumerate() {
            let sabiquant::cashflows::CashFlow::FixedRateCoupon(c) = flow else {
                return Err("non-coupon factory value".into());
            };
            let raw = s.contractual_dates();
            let days = u32::try_from(ordinal(raw[index + 1]) - ordinal(raw[index]))?;
            let alpha = f64::from(days) / 365.0;
            assert_eq!(c.start(), raw[index]);
            assert_eq!(c.end(), raw[index + 1]);
            assert_eq!(c.payment_date(), s.adjusted_dates()[index + 1]);
            assert_eq!(c.accrual_fraction().to_bits(), alpha.to_bits());
            assert_eq!(c.amount().to_bits(), alpha.to_bits());
        }
        let discount = InterestRate::new(0.0, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
        assert_eq!(
            leg.net_present_value(
                d("2026-02-06"),
                &discount,
                PaymentDateInclusion::Exclude,
                u64::from(count) + 1
            )?
            .to_bits(),
            bits
        );
        let end = *s.contractual_dates().last().ok_or("empty schedule")?;
        assert_eq!(
            leg.accrued_amount(end, PaymentDateInclusion::Include, 0)?
                .to_bits(),
            leg.flows().last().ok_or("empty leg")?.amount().to_bits()
        );
    }
    Ok(())
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "integration tests exercise immutable sharing; the library starts no threads"
)]
fn cloned_owned_builders_and_provenance_reuse_concurrently_without_reanchoring()
-> Result<(), Box<dyn Error>> {
    let b = Arc::new(ScheduleBuilder::new(
        d("2026-02-06"),
        d("2026-11-17"),
        ScheduleCalendar::Plain(Calendar::weekends_only()),
        Bdc::Following,
        Bdc::Following,
    )?);
    let baseline: Vec<_> = Roll::ALL
        .into_iter()
        .map(|r| b.build_roll(months(3), r, 5))
        .collect::<Result<_, _>>()?;
    let expected = Arc::new(baseline);
    let mut threads = Vec::new();
    for _ in 0..8 {
        let b = Arc::clone(&b);
        let expected = Arc::clone(&expected);
        threads.push(std::thread::spawn(move || {
            let mut cases = 0;
            for _ in 0..16 {
                for (r, reference) in Roll::ALL.into_iter().zip(expected.iter()) {
                    let s = b
                        .build_roll(months(3), r, 5)
                        .expect("valid threaded schedule");
                    assert_eq!(&s, reference);
                    assert_eq!(hash(&s), hash(reference));
                    assert_eq!(s.roll_provenance(), reference.roll_provenance());
                    assert_eq!(s.generation_rule(), Generation::Roll(r));
                    cases += 1;
                }
            }
            cases
        }));
    }
    let total: usize = threads
        .into_iter()
        .map(|t| t.join().expect("thread succeeds"))
        .sum();
    assert_eq!(total, 512);
    assert_eq!(b.start(), d("2026-02-06"));
    assert_eq!(b.end(), d("2026-11-17"));
    Ok(())
}
