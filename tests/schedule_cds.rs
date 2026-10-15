//! Independent modern CDS payment/accrual dates, bounded resources and guards.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, EndpointInclusion,
    JointRule, TargetCalendar, TargetPolicy,
};
use sabiquant::error::Error as RootError;
use sabiquant::schedule::{
    CdsCouponDates, CdsDatePlan, CdsMaturityConvention as Regime, DateGenerationRule as Rule,
    ScheduleBuilder, ScheduleCalendar, ScheduleError, StubPolicy,
};
use sabiquant::time::{Date, DateError, Period, TimeUnit, Weekday};
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

const REGIMES: [Regime; 2] = [Regime::Quarterly2009, Regime::Semiannual2015];

#[expect(
    clippy::expect_used,
    reason = "literal test dates are hand-audited ISO dates"
)]
fn d(text: &str) -> Date {
    text.parse().expect("valid literal date")
}

#[expect(
    clippy::panic,
    reason = "an explicitly rejected request must not return a value"
)]
fn rejected<T: Debug>(result: Result<T, ScheduleError>) -> ScheduleError {
    match result {
        Ok(value) => panic!("expected rejection, got {value:?}"),
        Err(error) => error,
    }
}

fn build(
    trade: Date,
    maturity: Date,
    calendar: Calendar,
    regime: Regime,
    cap: u32,
) -> Result<CdsDatePlan, ScheduleError> {
    CdsDatePlan::standard_following(
        trade,
        maturity,
        ScheduleCalendar::Plain(calendar),
        regime,
        cap,
    )
}

fn guarded(
    trade: Date,
    maturity: Date,
    calendar: TargetCalendar,
    cap: u32,
) -> Result<CdsDatePlan, ScheduleError> {
    CdsDatePlan::standard_following(
        trade,
        maturity,
        ScheduleCalendar::Target(calendar),
        Regime::Quarterly2009,
        cap,
    )
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// An independent Gregorian ordinal also supplies weekdays: 0001-01-01 was
// Monday. Expected selection never calls a schedule or calendar adjuster.
fn ordinal(date: Date) -> i64 {
    let (year, month, day) = date.to_ymd();
    let y = i64::from(year - 1);
    let before: u32 = (1..month).map(|m| month_days(year, m)).sum();
    365 * y + y / 4 - y / 100 + y / 400 + i64::from(before + day - 1)
}

fn weekend_payment(raw: Date) -> Result<Date, DateError> {
    raw.checked_add_days(match ordinal(raw).rem_euclid(7) {
        5 => 2,
        6 => 1,
        _ => 0,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExpectedCoupon {
    raw_start: Date,
    raw_end: Date,
    payment: Date,
    accrual_start: Date,
    accrual_end: Date,
    inclusion: EndpointInclusion,
    days: u32,
}

fn assert_coupon(actual: &CdsCouponDates, expected: &ExpectedCoupon) {
    assert_eq!(actual.unadjusted_start(), expected.raw_start);
    assert_eq!(actual.unadjusted_end(), expected.raw_end);
    assert_eq!(actual.payment_date(), expected.payment);
    assert_eq!(actual.accrual_start(), expected.accrual_start);
    assert_eq!(actual.accrual_end(), expected.accrual_end);
    assert_eq!(actual.accrual_end_inclusion(), expected.inclusion);
    assert_eq!(actual.accrual_day_count(), expected.days);
}

fn calendar_oracle(
    trade: Date,
    maturity: Date,
    weekends: bool,
) -> Result<Vec<ExpectedCoupon>, Box<dyn Error>> {
    // Enumerate civil quarter dates over a deliberately wider range, adjust
    // using independent weekday arithmetic, then select globally by the
    // strict inequality. This does not reproduce cutoff-quarter branching.
    let mut grid = Vec::new();
    for year in (trade.year() - 1)..=maturity.year() {
        for month in [3, 6, 9, 12] {
            let raw = Date::from_ymd(year, month, 20)?;
            if raw <= maturity {
                let payment = if weekends { weekend_payment(raw)? } else { raw };
                grid.push((raw, payment));
            }
        }
    }
    let cutoff_ordinal = ordinal(trade) + 1;
    let first = grid
        .iter()
        .position(|(_, payment)| ordinal(*payment) > cutoff_ordinal)
        .ok_or("oracle requires an eligible coupon")?;
    assert!(first > 0, "oracle includes the prior civil year");
    let mut result = Vec::new();
    for index in first..grid.len() {
        let (raw_end, payment) = grid[index];
        let (raw_start, accrual_start) = grid[index - 1];
        let final_coupon = raw_end == maturity;
        let accrual_end = if final_coupon { maturity } else { payment };
        result.push(ExpectedCoupon {
            raw_start,
            raw_end,
            payment,
            accrual_start,
            accrual_end,
            inclusion: if final_coupon {
                EndpointInclusion::Include
            } else {
                EndpointInclusion::Exclude
            },
            days: u32::try_from(
                ordinal(accrual_end) - ordinal(accrual_start) + i64::from(final_coupon),
            )?,
        });
    }
    Ok(result)
}

fn closed_range(start: Date, end: Date) -> Result<Calendar, DateError> {
    let dates = (0..=end.days_since(start))
        .map(|offset| start.checked_add_days(i64::from(offset)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Calendar::custom([], dates))
}

#[test]
fn primary_2009_payment_selection_and_actual_accrual_roles_are_exact() -> Result<(), Box<dyn Error>>
{
    // Standard CDS Examples' March 18/19 and June 20/21 controls distinguish
    // strict selection after unadjusted T+1 from selection by raw twentieth.
    let raw = [
        "2008-12-20",
        "2009-03-20",
        "2009-06-20",
        "2009-09-20",
        "2009-12-20",
    ]
    .map(d);
    let paid = [
        "2008-12-22",
        "2009-03-20",
        "2009-06-22",
        "2009-09-21",
        "2009-12-21",
    ]
    .map(d);
    let exact_days = [88, 94, 91, 91];
    for (trade, cutoff, first) in [
        ("2009-03-18", "2009-03-19", 1_usize),
        ("2009-03-19", "2009-03-20", 2),
        ("2009-06-20", "2009-06-21", 2),
        ("2009-06-21", "2009-06-22", 3),
    ] {
        for regime in REGIMES {
            let plan = build(d(trade), raw[4], Calendar::weekends_only(), regime, 4)?;
            assert_eq!(plan.trade_date(), d(trade));
            assert_eq!(plan.payment_selection_cutoff(), d(cutoff));
            assert_eq!(plan.unadjusted_maturity(), raw[4]);
            assert_eq!(plan.maturity_convention(), regime);
            assert_eq!(plan.coupon_dates().len(), 5 - first);
            for (offset, actual) in plan.coupon_dates().iter().enumerate() {
                let i = first + offset;
                assert_coupon(
                    actual,
                    &ExpectedCoupon {
                        raw_start: raw[i - 1],
                        raw_end: raw[i],
                        payment: paid[i],
                        accrual_start: paid[i - 1],
                        accrual_end: if i == 4 { raw[4] } else { paid[i] },
                        inclusion: if i == 4 {
                            EndpointInclusion::Include
                        } else {
                            EndpointInclusion::Exclude
                        },
                        days: exact_days[i - 1],
                    },
                );
            }
        }
    }
    Ok(())
}

#[test]
fn independent_400_year_quarter_oracle_checks_dates_inclusion_and_every_exact_cap()
-> Result<(), Box<dyn Error>> {
    let mut cases = 0;
    for year in 1600..2000 {
        for month in [3, 6, 9, 12] {
            let roll = Date::from_ymd(year, month, 20)?;
            let maturity = Date::from_ymd(year + 1, month, 20)?;
            for delta in [-1, 0, 1] {
                let trade = roll.checked_add_days(delta)?;
                for weekends in [false, true] {
                    let calendar = if weekends {
                        Calendar::weekends_only()
                    } else {
                        Calendar::null()
                    };
                    let expected = calendar_oracle(trade, maturity, weekends)?;
                    let count = u32::try_from(expected.len())?;
                    for regime in REGIMES {
                        let plan = build(trade, maturity, calendar.clone(), regime, count)?;
                        assert_eq!(plan.coupon_dates().len(), expected.len());
                        for (actual, wanted) in plan.coupon_dates().iter().zip(&expected) {
                            assert_coupon(actual, wanted);
                        }
                        assert_eq!(
                            plan,
                            build(trade, maturity, calendar.clone(), regime, count + 1)?
                        );
                        assert!(matches!(
                            build(trade, maturity, calendar.clone(), regime, count - 1),
                            Err(ScheduleError::CdsCouponLimitExceeded { maximum, required, .. })
                            if maximum == count - 1 && required == count
                        ));
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 19_200);
    Ok(())
}

#[test]
fn supplied_off_the_run_maturity_is_retained_under_both_explicit_regimes()
-> Result<(), Box<dyn Error>> {
    for maturity in ["2026-03-20", "2026-09-20"] {
        let old = build(
            d("2026-01-03"),
            d(maturity),
            Calendar::weekends_only(),
            REGIMES[0],
            4,
        )?;
        let new = build(
            d("2026-01-03"),
            d(maturity),
            Calendar::weekends_only(),
            REGIMES[1],
            4,
        )?;
        assert_eq!(old.unadjusted_maturity(), d(maturity));
        assert_eq!(new.unadjusted_maturity(), d(maturity));
        assert_eq!(old.coupon_dates(), new.coupon_dates());
        assert_ne!(old, new);
        assert_eq!(old.rule(), Rule::Cds);
        assert_eq!(new.rule(), Rule::Cds2015);
    }
    Ok(())
}

#[test]
fn strict_equality_can_exhaust_the_plan_while_holiday_maturity_still_has_a_payment()
-> Result<(), Box<dyn Error>> {
    for regime in REGIMES {
        let error = rejected(build(
            d("2026-03-19"),
            d("2026-03-20"),
            Calendar::null(),
            regime,
            1,
        ));
        assert!(matches!(error, ScheduleError::NoEligibleCdsPayment {
            trade_date, payment_selection_cutoff, unadjusted_maturity, final_payment, ..
        } if trade_date == d("2026-03-19") && payment_selection_cutoff == d("2026-03-20")
            && unadjusted_maturity == d("2026-03-20") && final_payment == d("2026-03-20")));
        assert_eq!(error.kind(), "schedule.no_eligible_cds_payment");
        assert!(error.source().is_none());
        let plan = build(
            d("2026-06-19"),
            d("2026-06-20"),
            Calendar::weekends_only(),
            regime,
            1,
        )?;
        assert_eq!(plan.coupon_dates().len(), 1);
        assert_coupon(
            &plan.coupon_dates()[0],
            &ExpectedCoupon {
                raw_start: d("2026-03-20"),
                raw_end: d("2026-06-20"),
                payment: d("2026-06-22"),
                accrual_start: d("2026-03-20"),
                accrual_end: d("2026-06-20"),
                inclusion: EndpointInclusion::Include,
                days: 93,
            },
        );
    }
    Ok(())
}

#[test]
fn closures_can_retain_a_raw_coupon_before_trade_but_cannot_merge_two_coupons()
-> Result<(), Box<dyn Error>> {
    let shifted = build(
        d("2026-03-22"),
        d("2026-06-20"),
        closed_range(d("2026-03-20"), d("2026-03-25"))?,
        REGIMES[0],
        2,
    )?;
    assert_eq!(shifted.payment_selection_cutoff(), d("2026-03-23"));
    assert_eq!(shifted.coupon_dates()[0].unadjusted_end(), d("2026-03-20"));
    assert_eq!(shifted.coupon_dates()[0].payment_date(), d("2026-03-26"));
    assert_eq!(shifted.coupon_dates()[0].accrual_start(), d("2025-12-20"));
    for (trade, first_closed, last_closed, index, previous, current, paid) in [
        (
            "2026-03-22",
            "2025-12-20",
            "2026-03-25",
            0_usize,
            "2025-12-20",
            "2026-03-20",
            "2026-03-26",
        ),
        (
            "2026-04-01",
            "2026-06-20",
            "2026-09-25",
            1,
            "2026-06-20",
            "2026-09-20",
            "2026-09-26",
        ),
    ] {
        let error = rejected(build(
            d(trade),
            d("2026-12-20"),
            closed_range(d(first_closed), d(last_closed))?,
            REGIMES[0],
            4,
        ));
        assert!(matches!(error, ScheduleError::CdsAdjustedOrder {
            index: got_index, previous_contractual, contractual, previous_adjusted, adjusted, ..
        } if got_index == index && previous_contractual == d(previous) && contractual == d(current)
            && previous_adjusted == d(paid) && adjusted == d(paid)));
        assert_eq!(error.kind(), "schedule.cds_adjusted_order");
    }
    Ok(())
}

#[test]
fn calendar_search_bound_is_preserved_without_clipping_final_payment_to_maturity()
-> Result<(), Box<dyn Error>> {
    let maturity = d("2026-03-20");
    let accepted = build(
        d("2026-03-18"),
        maturity,
        closed_range(maturity, maturity.checked_add_days(364)?)?,
        REGIMES[0],
        1,
    )?;
    assert_eq!(accepted.coupon_dates()[0].payment_date(), d("2027-03-20"));
    assert_eq!(accepted.coupon_dates()[0].accrual_end(), maturity);
    assert_eq!(accepted.coupon_dates()[0].accrual_day_count(), 91);
    let error = rejected(build(
        d("2026-03-18"),
        maturity,
        closed_range(maturity, maturity.checked_add_days(365)?)?,
        REGIMES[0],
        1,
    ));
    assert!(matches!(&error, ScheduleError::CdsCalendar {
        phase: "coupon adjustment", index: Some(0),
        cause: CalendarError::SearchLimit { start, direction: 1, rejected: 366, .. }, ..
    } if *start == maturity));
    assert_eq!(error.kind(), "schedule.cds_calendar");
    assert!(
        error
            .source()
            .is_some_and(|cause| cause.is::<CalendarError>())
    );
    let error = rejected(build(
        d("2026-04-01"),
        d("2026-06-20"),
        Calendar::custom(Weekday::ALL, []),
        REGIMES[0],
        1,
    ));
    assert!(matches!(
        error,
        ScheduleError::CdsCalendar {
            phase: "cutoff quarter adjustment",
            cause: CalendarError::NoBusinessWeekday,
            ..
        }
    ));
    Ok(())
}

#[test]
fn direct_constructor_domain_and_budget_precedence_are_explicit() -> Result<(), Box<dyn Error>> {
    assert_eq!(CdsDatePlan::MIN_COUPONS, 1);
    assert_eq!(CdsDatePlan::MAX_COUPONS, 39_996);
    for cap in [0, 39_997, u32::MAX] {
        let error = rejected(build(
            d("2026-12-31"),
            d("2026-01-01"),
            Calendar::custom(Weekday::ALL, []),
            REGIMES[0],
            cap,
        ));
        assert!(matches!(error, ScheduleError::InvalidCdsCouponLimit {
            requested, minimum: 1, maximum: 39_996, ..
        } if requested == cap));
        assert_eq!(error.kind(), "schedule.invalid_cds_coupon_limit");
    }
    for (trade, maturity) in [("2026-03-20", "2026-03-20"), ("2026-03-21", "2026-03-20")] {
        assert!(
            matches!(build(d(trade), d(maturity), Calendar::null(), REGIMES[0], 1),
            Err(ScheduleError::InvalidRange { start, end, .. }) if start == d(trade) && end == d(maturity))
        );
    }
    for maturity in ["2026-02-20", "2026-03-19", "2026-03-21", "9999-12-31"] {
        for convention in REGIMES {
            let error = rejected(build(
                d("2026-01-01"),
                d(maturity),
                Calendar::null(),
                convention,
                1,
            ));
            assert!(
                matches!(error, ScheduleError::InvalidCdsMaturity { maturity: got, convention: got_convention, .. }
                if got == d(maturity) && got_convention == convention)
            );
            assert_eq!(error.kind(), "schedule.invalid_cds_maturity");
        }
    }
    let calendar = Calendar::null().with_holiday(d("0001-03-20"));
    let error = rejected(build(
        d("0001-03-19"),
        d("0001-06-20"),
        calendar.clone(),
        REGIMES[0],
        1,
    ));
    assert!(matches!(
        error,
        ScheduleError::CdsCouponLimitExceeded {
            maximum: 1,
            required: 2,
            ..
        }
    ));
    let error = rejected(build(
        d("0001-03-19"),
        d("0001-06-20"),
        calendar,
        REGIMES[0],
        2,
    ));
    // The adopted contract uses the original trade plus the required signed
    // quarter-month displacement for Date causes. The role distinguishes
    // this required precursor from the cutoff quarter or a future coupon.
    assert!(
        matches!(&error, ScheduleError::CdsDate {
        trade_date, unadjusted_maturity, role: "initial accrual precursor", input,
        cause: DateError::OutOfRange { date, operation: "add months", count: -3, .. }, ..
    } if *trade_date == d("0001-03-19") && *unadjusted_maturity == d("0001-06-20")
        && *input == d("0001-03-19") && *date == d("0001-03-19")),
        "{error:?}"
    );
    assert_eq!(error.kind(), "schedule.cds_date");
    assert!(error.source().is_some_and(|cause| cause.is::<DateError>()));
    Ok(())
}

#[test]
fn required_civil_precursors_and_final_adjustment_keep_their_typed_root_causes()
-> Result<(), Box<dyn Error>> {
    let error = rejected(build(
        Date::MIN,
        d("0001-03-20"),
        Calendar::null(),
        REGIMES[0],
        1,
    ));
    assert!(matches!(&error, ScheduleError::CdsDate {
        trade_date, unadjusted_maturity, role: "cutoff quarter", ..
    } if *trade_date == Date::MIN && *unadjusted_maturity == d("0001-03-20")));
    let root = RootError::from(error);
    assert!(
        root.source()
            .is_some_and(|cause| cause.is::<ScheduleError>())
    );
    assert!(
        root.source()
            .and_then(|cause| cause.source())
            .is_some_and(|cause| cause.is::<DateError>())
    );
    let error = rejected(build(
        d("9999-09-21"),
        d("9999-12-20"),
        closed_range(d("9999-12-20"), Date::MAX)?,
        REGIMES[0],
        1,
    ));
    assert!(matches!(
        &error,
        ScheduleError::CdsCalendar {
            phase: "coupon adjustment",
            index: Some(0),
            cause: CalendarError::Date(_),
            ..
        }
    ));
    let root = RootError::from(error);
    assert!(
        root.source()
            .and_then(|cause| cause.source())
            .is_some_and(|cause| cause.is::<CalendarError>())
    );
    assert!(
        root.source()
            .and_then(|cause| cause.source())
            .and_then(|cause| cause.source())
            .is_some_and(|cause| cause.is::<DateError>())
    );
    Ok(())
}

#[test]
fn full_civil_plan_has_exact_quarter_count_and_does_not_request_a_terminal_successor()
-> Result<(), Box<dyn Error>> {
    let trade = d("0001-03-19");
    let maturity = d("9999-12-20");
    let plan = build(trade, maturity, Calendar::null(), REGIMES[0], 39_995)?;
    assert_eq!(plan.coupon_dates().len(), 39_995);
    assert_eq!(plan.coupon_dates()[0].unadjusted_start(), d("0001-03-20"));
    assert_eq!(plan.coupon_dates()[0].unadjusted_end(), d("0001-06-20"));
    let mut index = 0;
    let mut previous = d("0001-03-20");
    let mut actual_days = 0_u64;
    for year in 1..=9999 {
        for month in [3, 6, 9, 12] {
            if year == 1 && month == 3 {
                continue;
            }
            let raw = Date::from_ymd(year, month, 20)?;
            let final_coupon = raw == maturity;
            let expected_days =
                u32::try_from(ordinal(raw) - ordinal(previous) + i64::from(final_coupon))?;
            assert_coupon(
                &plan.coupon_dates()[index],
                &ExpectedCoupon {
                    raw_start: previous,
                    raw_end: raw,
                    payment: raw,
                    accrual_start: previous,
                    accrual_end: raw,
                    days: expected_days,
                    inclusion: if final_coupon {
                        EndpointInclusion::Include
                    } else {
                        EndpointInclusion::Exclude
                    },
                },
            );
            actual_days += u64::from(expected_days);
            previous = raw;
            index += 1;
        }
    }
    assert_eq!(index, plan.coupon_dates().len());
    assert_eq!(
        actual_days,
        u64::try_from(ordinal(maturity) - ordinal(d("0001-03-20")) + 1)?
    );
    assert_eq!(
        plan,
        build(trade, maturity, Calendar::null(), REGIMES[0], 39_996)?
    );
    assert!(matches!(
        build(trade, maturity, Calendar::null(), REGIMES[0], 39_994),
        Err(ScheduleError::CdsCouponLimitExceeded {
            maximum: 39_994,
            required: 39_995,
            ..
        })
    ));
    Ok(())
}

#[test]
fn target_original_and_cutoff_extension_guards_survive_overrides_and_every_joint_rule()
-> Result<(), Box<dyn Error>> {
    let strict = TargetCalendar::new();
    let mut calendars = vec![strict.clone(), strict.without_holiday(d("2025-12-20"))];
    for joint in JointRule::ALL {
        calendars.push(strict.joint_with(vec![Calendar::null()], joint)?);
    }
    for calendar in calendars {
        let error = rejected(guarded(
            d("2026-01-01"),
            d("2026-12-20"),
            calendar.clone(),
            4,
        ));
        assert!(matches!(&error, ScheduleError::CdsCalendar {
            phase: "cutoff quarter coverage", index: None, start, end,
            cause: CalendarError::UnverifiedCoverage { date, .. }, ..
        } if *start == d("2025-12-20") && *end == d("2026-12-20") && *date == d("2025-12-20")));
        let error = rejected(guarded(d("2002-12-20"), d("2026-03-20"), calendar, 100));
        assert!(matches!(&error, ScheduleError::CdsCalendar {
            phase: "original coverage", index: None,
            cause: CalendarError::UnverifiedCoverage { date, .. }, ..
        } if *date == d("2003-01-01")));
    }
    for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
        let target = TargetCalendar::with_policy(policy).without_holiday(d("1999-01-03"));
        let error = rejected(guarded(d("1999-01-03"), d("1999-03-20"), target.clone(), 1));
        assert!(matches!(&error, ScheduleError::CdsCalendar {
            phase: "original coverage", cause: CalendarError::OutsideValidity { date, .. }, ..
        } if *date == d("1999-01-03")));
        let error = rejected(guarded(d("1999-01-04"), d("1999-03-20"), target, 1));
        assert!(matches!(&error, ScheduleError::CdsCalendar {
            phase: "cutoff quarter coverage", cause: CalendarError::OutsideValidity { date, .. }, ..
        } if *date == d("1998-12-20")));
    }
    Ok(())
}

#[test]
fn target_precursor_cap_and_final_candidate_errors_have_distinct_contexts()
-> Result<(), Box<dyn Error>> {
    let target = TargetCalendar::new().with_holiday(d("2026-03-20"));
    let error = rejected(guarded(d("2026-03-19"), d("2026-06-20"), target.clone(), 1));
    assert!(matches!(
        error,
        ScheduleError::CdsCouponLimitExceeded {
            maximum: 1,
            required: 2,
            ..
        }
    ));
    let error = rejected(guarded(d("2026-03-19"), d("2026-06-20"), target, 2));
    assert!(matches!(&error, ScheduleError::CdsCalendar {
        phase: "precursor coverage", index: None, start, end,
        cause: CalendarError::UnverifiedCoverage { date, .. }, ..
    } if *start == d("2025-12-20") && *end == d("2026-06-20") && *date == d("2025-12-20")));
    let mut strict = TargetCalendar::new();
    let mut projected = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    for day in 20..=31 {
        let holiday = Date::from_ymd(2028, 12, day)?;
        strict = strict.with_holiday(holiday);
        projected = projected.with_holiday(holiday);
    }
    let error = rejected(guarded(d("2028-09-21"), d("2028-12-20"), strict, 1));
    assert!(matches!(&error, ScheduleError::CdsCalendar {
        phase: "coupon adjustment", index: Some(0),
        cause: CalendarError::UnverifiedCoverage { date, .. }, ..
    } if *date == d("2029-01-01")));
    let plan = guarded(d("2028-09-21"), d("2028-12-20"), projected, 1)?;
    assert_eq!(plan.coupon_dates()[0].payment_date(), d("2029-01-02"));
    assert_eq!(plan.coupon_dates()[0].accrual_end(), d("2028-12-20"));
    Ok(())
}

#[test]
fn explicit_target_projection_and_both_verified_intervals_have_positive_controls()
-> Result<(), Box<dyn Error>> {
    for (trade, maturity) in [("1999-04-01", "2002-12-20"), ("2026-04-01", "2028-12-20")] {
        let plan = guarded(d(trade), d(maturity), TargetCalendar::new(), 20)?;
        assert_eq!(plan.unadjusted_maturity(), d(maturity));
        assert!(
            plan.coupon_dates()
                .iter()
                .all(|coupon| coupon.payment_date() >= coupon.unadjusted_end())
        );
    }
    let projected = guarded(
        d("2026-01-01"),
        d("2026-12-20"),
        TargetCalendar::with_policy(TargetPolicy::AllowProjection),
        4,
    )?;
    assert_eq!(
        projected.coupon_dates()[0].unadjusted_start(),
        d("2025-12-20")
    );
    assert_eq!(projected.coupon_dates()[0].accrual_start(), d("2025-12-22"));
    let crossing = guarded(
        d("2002-12-20"),
        d("2026-03-20"),
        TargetCalendar::with_policy(TargetPolicy::AllowProjection),
        100,
    )?;
    assert_eq!(
        crossing.coupon_dates()[0].unadjusted_start(),
        d("2002-12-20")
    );
    assert_eq!(crossing.coupon_dates().len(), 93);
    Ok(())
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "integration tests exercise immutable sharing; the library starts no threads"
)]
fn immutable_definition_identity_and_thread_reuse_keep_unsimplified_context()
-> Result<(), Box<dyn Error>> {
    fn immutable<T: Clone + Debug + Eq + Hash + Send + Sync + 'static>() {}
    fn copy_value<T: Copy>() {}
    immutable::<CdsDatePlan>();
    immutable::<CdsCouponDates>();
    copy_value::<CdsCouponDates>();
    let calendar = Calendar::null();
    let plan = build(
        d("2026-04-01"),
        d("2026-12-20"),
        calendar.clone(),
        REGIMES[0],
        3,
    )?;
    assert_eq!(plan.calendar(), &ScheduleCalendar::Plain(calendar.clone()));
    let same = build(
        d("2026-04-01"),
        d("2026-12-20"),
        calendar.clone(),
        REGIMES[0],
        4,
    )?;
    assert_eq!(plan, same);
    assert_eq!(hash(&plan), hash(&same));
    let changed_trade = build(
        d("2026-04-02"),
        d("2026-12-20"),
        calendar.clone(),
        REGIMES[0],
        3,
    )?;
    let changed_regime = build(
        d("2026-04-01"),
        d("2026-12-20"),
        calendar.clone(),
        REGIMES[1],
        3,
    )?;
    let changed_calendar = build(
        d("2026-04-01"),
        d("2026-12-20"),
        calendar.with_holiday(d("2028-05-01")),
        REGIMES[0],
        3,
    )?;
    for distinct in [&changed_trade, &changed_regime, &changed_calendar] {
        assert_eq!(plan.coupon_dates(), distinct.coupon_dates());
        assert_ne!(&plan, distinct);
    }
    let values: HashSet<_> = [
        plan.clone(),
        same,
        changed_trade,
        changed_regime,
        changed_calendar,
    ]
    .into_iter()
    .collect();
    assert_eq!(values.len(), 4);
    let shared = Arc::new(plan.clone());
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let shared = Arc::clone(&shared);
            std::thread::spawn(move || (hash(shared.as_ref()), shared.coupon_dates().to_vec()))
        })
        .collect();
    for worker in workers {
        let (worker_hash, coupons) = worker
            .join()
            .map_err(|_| "CDS immutable query thread failed")?;
        assert_eq!(worker_hash, hash(&plan));
        assert_eq!(coupons, plan.coupon_dates());
    }
    Ok(())
}

#[test]
fn modern_labels_are_explicit_and_ordinary_builders_do_not_accept_cds_plans()
-> Result<(), Box<dyn Error>> {
    let ordinary = ScheduleBuilder::new(
        d("2026-01-01"),
        d("2026-12-20"),
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?;
    for (rule, text) in [(Rule::Cds, "CDS"), (Rule::Cds2015, "CDS2015")] {
        assert_eq!(text.parse::<Rule>()?, rule);
        assert_eq!(rule.canonical_identifier(), text);
        assert_eq!(rule.to_string(), text);
        for wrong in [format!(" {text}"), format!("{text} "), text.to_lowercase()] {
            assert!(wrong.parse::<Rule>().is_err());
        }
        assert!(
            matches!(ordinary.build_recurrent(Period::new(3, TimeUnit::Months), rule,
            StubPolicy::Short, EndOfMonth::Ignore, 4), Err(ScheduleError::InvalidRule { rule: actual, .. }) if actual == rule)
        );
        assert!(matches!(
            ordinary.build_recurrent(
                Period::new(3, TimeUnit::Months),
                rule,
                StubPolicy::Short,
                EndOfMonth::Ignore,
                0
            ),
            Err(ScheduleError::InvalidBoundaryLimit { requested: 0, .. })
        ));
    }
    // OldCDS has its own automatic builder; it is not a modern CDS plan label.
    assert_eq!("OldCDS".parse::<Rule>()?, Rule::OldCds);
    Ok(())
}

#[test]
fn quoted_resolution_and_explicit_date_plan_are_separately_fallible_operations()
-> Result<(), Box<dyn Error>> {
    use sabiquant::schedule::{CdsMaturityOutcome, resolve_cds_maturity};
    let trade = d("2015-12-22");
    for (regime, expected_maturity, expected_count) in [
        (Regime::Quarterly2009, d("2021-03-20"), 21_usize),
        (Regime::Semiannual2015, d("2020-12-20"), 20),
    ] {
        let resolution = resolve_cds_maturity(trade, Period::new(5, TimeUnit::Years), regime)?;
        assert_eq!(
            resolution.outcome(),
            CdsMaturityOutcome::Date(expected_maturity)
        );
        let plan = build(
            trade,
            expected_maturity,
            Calendar::weekends_only(),
            regime,
            u32::try_from(expected_count)?,
        )?;
        assert_eq!(plan.coupon_dates().len(), expected_count);
        assert_eq!(plan.coupon_dates()[0].unadjusted_start(), d("2015-12-20"));
        assert_eq!(plan.coupon_dates()[0].accrual_start(), d("2015-12-21"));
        assert_eq!(plan.coupon_dates()[0].payment_date(), d("2016-03-21"));
        assert_eq!(
            plan.coupon_dates()
                .last()
                .ok_or("nonempty plan")?
                .accrual_end(),
            expected_maturity
        );
    }
    // A matured quote has no Date to feed into the plan. A valid future
    // zero-month maturity can independently have no eligible selected payment.
    let trade = d("2026-06-20");
    assert_eq!(
        resolve_cds_maturity(
            trade,
            Period::new(0, TimeUnit::Months),
            Regime::Semiannual2015
        )?
        .outcome(),
        CdsMaturityOutcome::Matured
    );
    let trade = d("2026-03-19");
    let resolution = resolve_cds_maturity(
        trade,
        Period::new(0, TimeUnit::Years),
        Regime::Quarterly2009,
    )?;
    assert_eq!(
        resolution.outcome(),
        CdsMaturityOutcome::Date(d("2026-03-20"))
    );
    assert!(matches!(
        build(
            trade,
            d("2026-03-20"),
            Calendar::null(),
            Regime::Quarterly2009,
            1
        ),
        Err(ScheduleError::NoEligibleCdsPayment { .. })
    ));
    Ok(())
}

#[test]
fn a_required_precursor_search_failure_is_not_misclassified_as_a_collision()
-> Result<(), Box<dyn Error>> {
    let trade = d("2026-03-20");
    let maturity = d("2026-06-20");
    let precursor = d("2025-12-20");
    let calendar = closed_range(precursor, d("2026-12-21"))?;
    // The cutoff quarter has a reachable Following business day, whereas
    // its required earlier precursor exceeds the 366-rejection search cap.
    // This calendar error precedes any eventual payment collision.
    for regime in REGIMES {
        let error = rejected(build(trade, maturity, calendar.clone(), regime, 2));
        assert!(matches!(&error, ScheduleError::CdsCalendar {
            trade_date, unadjusted_maturity, phase: "precursor adjustment", index: None,
            start, end, cause: CalendarError::SearchLimit {
                start: cause_start, direction: 1, rejected: 366, .. }, ..
        } if *trade_date == trade && *unadjusted_maturity == maturity
            && *start == precursor && *end == precursor && *cause_start == precursor));
        let root = RootError::from(error);
        assert_eq!(root.kind(), "schedule.cds_calendar");
        assert!(
            root.source()
                .and_then(|cause| cause.source())
                .is_some_and(|cause| cause.is::<CalendarError>())
        );
    }
    Ok(())
}

#[test]
fn a_later_payment_search_failure_retains_its_index_after_a_valid_first_coupon()
-> Result<(), Box<dyn Error>> {
    let trade = d("2026-04-01");
    let maturity = d("2026-09-20");
    let calendar = closed_range(maturity, maturity.checked_add_days(365)?)?;
    for regime in REGIMES {
        let error = rejected(build(trade, maturity, calendar.clone(), regime, 2));
        assert!(matches!(&error, ScheduleError::CdsCalendar {
            trade_date, unadjusted_maturity, phase: "coupon adjustment", index: Some(1),
            start, end, cause: CalendarError::SearchLimit {
                start: cause_start, direction: 1, rejected: 366, .. }, ..
        } if *trade_date == trade && *unadjusted_maturity == maturity
            && *start == maturity && *end == maturity && *cause_start == maturity));
        let root = RootError::from(error);
        assert_eq!(root.kind(), "schedule.cds_calendar");
        assert!(
            root.source()
                .and_then(|cause| cause.source())
                .is_some_and(|cause| cause.is::<CalendarError>())
        );
    }
    Ok(())
}
