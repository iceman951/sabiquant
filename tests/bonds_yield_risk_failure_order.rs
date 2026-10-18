//! Public quote, factor, branch and present-value failure-order controls for
//! FR-YR-12 and FR-YR-14.
//!
//! Derived from the frozen generic yield/risk contract (sections 4 to 6 and 9)
//! and the existing M4 quote, factor and present-value policies. Every plan
//! uses valid public bond construction. Expectations are exact dyadic values
//! or bit-exact results of the contract's staged Simple operations
//! (`q=RN_fma(1+y*t)`, then the rounded reciprocal and present value).
//! Periodic and Continuous values depend on elementary functions, so they are
//! compared only by order or inequality. No report field or private state is
//! fabricated.

#![expect(
    clippy::expect_used,
    reason = "failure-order controls require the stated valid public construction/results"
)]

use sabiquant::bonds::{
    Bond, BondError, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement,
    BondYieldConvention, BondYieldError,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{ActualActualIcma, DayCount, DeterminationPeriod};
use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{EvaluationStage, RootBracket, SolverOptions};
use sabiquant::rates::{AccrualConvention, Compounding, CompoundingMethod, RateError};
use sabiquant::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::ops::Bound;

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid civil control date")
}

fn power(exponent: u64) -> f64 {
    f64::from_bits((1023 + exponent) << 52)
}

fn zero_coupon(issue: Date, maturity: Date, face: f64, redemption: f64) -> Bond {
    Bond::zero_coupon(
        issue,
        maturity,
        face,
        SimpleCashFlow::new(maturity, redemption).expect("positive redemption"),
    )
    .expect("valid public zero-coupon bond")
}

fn forward_schedule(issue: Date, maturity: Date, tenor: Period) -> sabiquant::schedule::Schedule {
    ScheduleBuilder::new(
        issue,
        maturity,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )
    .expect("schedule definition")
    .build_recurrent(
        tenor,
        DateGenerationRule::Forward,
        StubPolicy::Short,
        EndOfMonth::Ignore,
        8,
    )
    .expect("forward coupon intervals")
}

fn icma(boundaries: &[Date], frequency: Frequency) -> ActualActualIcma {
    let periods = boundaries
        .windows(2)
        .map(|pair| DeterminationPeriod::new(pair[0], pair[1]).expect("ordered period"))
        .collect();
    ActualActualIcma::new(periods, frequency).expect("contiguous ICMA periods")
}

fn plan<'a>(
    bond: &'a Bond,
    settlement: Date,
    inclusion: PaymentDateInclusion,
    convention: &'a BondYieldConvention,
) -> BondPricingPlan<'a> {
    BondPricingPlan::new(
        bond,
        BondSettlement::new(settlement, inclusion),
        convention,
        8,
    )
    .expect("prepared public plan")
}

fn options() -> SolverOptions {
    SolverOptions::new(1e-12, 0.0, 16).expect("explicit positive tolerance/cap")
}

fn dirty(value: f64) -> BondPrice {
    BondPrice::new(value, BondPriceKind::Dirty).expect("finite signed per100 target")
}

/// All 30 periodic and hybrid configurations with their event count `m`.
fn periodic_configurations() -> Vec<(Compounding, f64)> {
    let mut configurations = Vec::new();
    for frequency in Frequency::ALL {
        let Some(count) = frequency.events_per_year() else {
            continue;
        };
        for compounding in [
            Compounding::compounded(frequency),
            Compounding::simple_then_compounded(frequency),
            Compounding::compounded_then_simple(frequency),
        ] {
            configurations.push((compounding.expect("periodic frequency"), f64::from(count)));
        }
    }
    assert_eq!(configurations.len(), 30);
    configurations
}

/// The contracted fixed-time branch: hybrids compare `t` with `RN(1/m)`.
fn simple_branch(compounding: Compounding, count: f64, time: f64) -> bool {
    let cutoff = 1.0 / count;
    match compounding.method() {
        CompoundingMethod::Simple => true,
        CompoundingMethod::SimpleThenCompounded => time <= cutoff,
        CompoundingMethod::CompoundedThenSimple => time > cutoff,
        _ => false,
    }
}

/// One flow's staged Simple present value: fused factor, rounded reciprocal,
/// rounded product.
fn simple_dirty(amount: f64, rate: f64, time: f64) -> f64 {
    amount * (1.0 / rate.mul_add(time, 1.0))
}

fn assert_quote_domain(error: &BondError, rate: f64, count: f64) {
    assert!(
        matches!(error, BondError::Rate {
            index: None,
            phase: "yield quote",
            cause: RateError::Value(ValueError::OutOfDomain {
                parameter: "rate", value, domain, ..
            }),
            ..
        } if value.to_bits() == rate.to_bits()
            && domain.lower() == Bound::Excluded(-count)
            && domain.upper() == Bound::Unbounded),
        "{error:?}"
    );
}

fn assert_lower_callback(failure: &BondYieldError, rate: f64, expected: &BondError) {
    assert!(
        matches!(failure, BondYieldError::Callback {
            stage: EvaluationStage::LowerEndpoint, point, evaluations, cause, ..
        } if point.to_bits() == rate.to_bits()
            && evaluations.function() == 1
            && evaluations.derivative() == 0
            && cause == expected),
        "{failure:?}"
    );
}

fn assert_factor_failure(error: &BondError, index: usize, factor: f64) {
    assert!(
        matches!(error, BondError::Rate {
            index: Some(observed),
            phase: "discount factor",
            cause: RateError::Value(ValueError::ResultOutOfDomain {
                quantity: "compound factor", value, domain, ..
            }),
            ..
        } if *observed == index
            && value.to_bits() == factor.to_bits()
            && domain.lower() == Bound::Excluded(0.0)
            && domain.upper() == Bound::Unbounded),
        "{error:?}"
    );
}

/// FR-YR-12: every periodic or hybrid quote must exceed `-m`, in `risk` and in
/// the one lower inversion callback. This holds for a paid plan with no flow,
/// a time-zero plan, a one-day horizon `t=RN(1/365)` that is Simple-active for
/// every `SimpleThenCompounded` frequency, and a two-year horizon that is
/// Simple-active for every `CompoundedThenSimple` frequency.
#[test]
fn periodic_and_hybrid_quotes_must_exceed_minus_m_before_any_flow_work() {
    let issue = date(2024, 1, 1);
    let next_day = date(2024, 1, 2);
    let short = zero_coupon(issue, next_day, 100.0, 100.0);
    let long = zero_coupon(issue, date(2026, 1, 1), 100.0, 100.0);
    let (mut simple_active, mut periodic) = (0, 0);
    for (compounding, count) in periodic_configurations() {
        let convention = BondYieldConvention::new(DayCount::Actual365Fixed, compounding);
        let horizon = |payment| {
            convention
                .accrual()
                .year_fraction(issue, payment)
                .expect("scalar horizon")
        };
        let paid = plan(
            &short,
            date(2024, 2, 1),
            PaymentDateInclusion::Exclude,
            &convention,
        );
        let time_zero = plan(&short, next_day, PaymentDateInclusion::Include, &convention);
        let one_day = plan(&short, issue, PaymentDateInclusion::Exclude, &convention);
        let two_year = plan(&long, issue, PaymentDateInclusion::Exclude, &convention);
        let plans = [
            (&paid, None),
            (&time_zero, None),
            (&one_day, Some(horizon(next_day))),
            (&two_year, Some(horizon(date(2026, 1, 1)))),
        ];
        let counts: Vec<usize> = plans
            .iter()
            .map(|(plan, _)| plan.entitled_flow_count())
            .collect();
        assert_eq!(counts, [0, 1, 1, 1]);
        for (index, (plan, time)) in plans.into_iter().enumerate() {
            for rate in [-count, (-count).next_down(), -2.0 * count, f64::MIN] {
                let error = plan.risk(rate).expect_err("quote at or below -m");
                assert_quote_domain(&error, rate, count);
                let bracket = RootBracket::new(rate, 1.0).expect("finite increasing bracket");
                let failure = plan
                    .yield_from_price(dirty(100.0), bracket, options())
                    .expect_err("the lower quote fails before any constant comparison");
                assert_lower_callback(&failure, rate, &error);
            }

            // An admitted quote: exact staged Simple values, otherwise a
            // discount above one.
            let rate = -0.25;
            let observed = plan
                .risk(rate)
                .expect("quote inside the periodic domain")
                .dirty_amount();
            match time {
                None => {
                    let expected = if index == 0 { 0.0_f64 } else { 100.0_f64 };
                    assert_eq!(observed.to_bits(), expected.to_bits());
                }
                Some(time) if simple_branch(compounding, count, time) => {
                    simple_active += 1;
                    assert_eq!(
                        observed.to_bits(),
                        simple_dirty(100.0, rate, time).to_bits()
                    );
                }
                Some(_) => {
                    periodic += 1;
                    assert!(observed > 100.0, "{observed:?}");
                }
            }
        }
    }
    // Ten one-day STC and ten two-year CTS flows are Simple-active; the ten
    // Compounded configurations, one-day CTS and two-year STC are periodic.
    assert_eq!((simple_active, periodic), (20, 40));
}

/// FR-YR-12: Simple and Continuous quotes have no `-m` floor. A paid or
/// time-zero plan accepts an extreme negative quote, while a positive-time
/// flow fails at its original index, not at its traversal position.
#[test]
fn simple_and_continuous_quotes_fail_only_at_an_indexed_positive_time_factor() {
    let issue = date(2024, 1, 1);
    let next_day = date(2024, 1, 2);
    let short = zero_coupon(issue, next_day, 100.0, 100.0);
    let maturity = date(2025, 1, 1);
    let semiannual = Bond::fixed_rate(
        issue,
        forward_schedule(issue, maturity, Period::new(6, TimeUnit::Months)),
        100.0,
        FixedCouponRate::new(0.05, DayCount::Thirty360BondBasis).expect("finite coupon"),
        vec![None, None],
        SimpleCashFlow::new(maturity, 100.0).expect("positive redemption"),
        8,
    )
    .expect("valid public fixed-rate bond");
    for compounding in [Compounding::SIMPLE, Compounding::CONTINUOUS] {
        let convention = BondYieldConvention::new(DayCount::Actual365Fixed, compounding);
        let paid = plan(
            &short,
            date(2024, 2, 1),
            PaymentDateInclusion::Exclude,
            &convention,
        );
        let time_zero = plan(&short, next_day, PaymentDateInclusion::Include, &convention);
        let one_day = plan(&short, issue, PaymentDateInclusion::Exclude, &convention);
        // The first coupon is paid, so the first entitled flow is coupon 1.
        let late_settlement = date(2024, 8, 1);
        let late = plan(
            &semiannual,
            late_settlement,
            PaymentDateInclusion::Exclude,
            &convention,
        );
        assert_eq!(late.entitled_flow_count(), 2);
        for rate in [-1.0e300, f64::MIN] {
            let bracket = RootBracket::new(rate, 1.0).expect("finite increasing bracket");
            assert_eq!(
                paid.risk(rate).expect("no factor").dirty_amount().to_bits(),
                0.0_f64.to_bits()
            );
            assert_eq!(
                time_zero
                    .risk(rate)
                    .expect("time-zero factor is one")
                    .dirty_amount()
                    .to_bits(),
                100.0_f64.to_bits()
            );
            assert!(matches!(
                time_zero.yield_from_price(dirty(100.0), bracket, options()),
                Err(BondYieldError::NoYieldSensitivity {
                    observed_lower_dirty_amount: 100.0,
                    constant_amount: Some(100.0),
                    evaluations,
                    ..
                }) if evaluations.function() == 1
            ));

            for (plan, index, payment, settlement) in [
                (&one_day, 0, next_day, issue),
                (&late, 1, maturity, late_settlement),
            ] {
                // Simple keeps the fused `1+y*t`; Continuous underflows to zero.
                let factor = if compounding == Compounding::SIMPLE {
                    let time = convention
                        .accrual()
                        .year_fraction(settlement, payment)
                        .expect("scalar horizon");
                    rate.mul_add(time, 1.0)
                } else {
                    0.0
                };
                let error = plan
                    .risk(rate)
                    .expect_err("first positive-time factor fails");
                assert_factor_failure(&error, index, factor);
                let failure = plan
                    .yield_from_price(dirty(100.0), bracket, options())
                    .expect_err("lower factor fails");
                assert_lower_callback(&failure, rate, &error);
            }
        }
    }
}

/// FR-YR-12: hybrid branch selection compares the prepared binary time with
/// the rounded cutoff `RN(1/m)`, inclusively for `SimpleThenCompounded` and
/// strictly for `CompoundedThenSimple`. Public Actual/Actual ICMA horizons
/// are compensated sums of per-period quotients, so they can equal the cutoff
/// or one of its binary neighbours. At `y=next_up(-m)` the staged Simple value
/// needs no elementary function and differs from the periodic value, which
/// identifies the selected branch.
#[test]
fn rounded_cutoffs_and_adjacent_times_select_the_contracted_branch() {
    let monthly = icma(
        &[date(2024, 7, 1), date(2024, 8, 1), date(2024, 9, 1)],
        Frequency::Monthly,
    );
    let quarterly = icma(
        &[date(2024, 1, 1), date(2024, 4, 1), date(2024, 7, 1)],
        Frequency::Quarterly,
    );
    // Accrual, issue, settlement, payment, yield frequency, binary steps from
    // `RN(1/m)` to the prepared time, then the expected Simple selections of
    // `SimpleThenCompounded` and `CompoundedThenSimple`.
    let cases = [
        (
            AccrualConvention::from(DayCount::Actual365Fixed),
            date(2024, 1, 1),
            date(2024, 1, 1),
            date(2024, 1, 2),
            Frequency::Daily,
            0,
            (true, false),
        ),
        (
            AccrualConvention::from(monthly.clone()),
            date(2024, 7, 1),
            date(2024, 7, 1),
            date(2024, 8, 1),
            Frequency::Monthly,
            0,
            (true, false),
        ),
        // 3/372 + 28/372 is exactly 1/12, but its rounded sum is next_up.
        (
            AccrualConvention::from(monthly),
            date(2024, 7, 1),
            date(2024, 7, 29),
            date(2024, 8, 29),
            Frequency::Monthly,
            1,
            (false, true),
        ),
        // 26/364 + 2/364 is exactly 1/13, but its rounded sum is next_down.
        (
            AccrualConvention::from(quarterly),
            date(2024, 1, 1),
            date(2024, 3, 6),
            date(2024, 4, 3),
            Frequency::EveryFourthWeek,
            -1,
            (true, false),
        ),
    ];
    for (accrual, issue, settlement, payment, frequency, step, (stc, cts)) in cases {
        let count = f64::from(frequency.events_per_year().expect("periodic frequency"));
        let cutoff = 1.0 / count;
        let time = accrual
            .year_fraction(settlement, payment)
            .expect("covered horizon");
        let expected_time = match step {
            0 => cutoff,
            1 => cutoff.next_up(),
            _ => cutoff.next_down(),
        };
        assert_eq!(time.to_bits(), expected_time.to_bits(), "{frequency:?}");

        let bond = zero_coupon(issue, payment, 100.0, 100.0);
        let rate = (-count).next_up();
        let staged_simple = simple_dirty(100.0, rate, time);
        assert!(staged_simple.is_finite() && staged_simple > 100.0);
        for (compounding, simple) in [
            (Compounding::compounded(frequency), false),
            (Compounding::simple_then_compounded(frequency), stc),
            (Compounding::compounded_then_simple(frequency), cts),
        ] {
            let compounding = compounding.expect("periodic frequency");
            assert_eq!(simple, simple_branch(compounding, count, time));
            let convention = BondYieldConvention::new(accrual.clone(), compounding);
            let plan = plan(
                &bond,
                settlement,
                PaymentDateInclusion::Exclude,
                &convention,
            );
            let observed = plan
                .risk(rate)
                .expect("admitted quote and positive factor")
                .dirty_amount();
            if simple {
                assert_eq!(
                    observed.to_bits(),
                    staged_simple.to_bits(),
                    "{compounding:?}"
                );
            } else {
                assert_ne!(
                    observed.to_bits(),
                    staged_simple.to_bits(),
                    "{compounding:?}"
                );
                assert!(observed > 100.0, "{compounding:?}: {observed:?}");
            }
        }
    }
}

/// FR-YR-12: at an exactly representable cutoff `t=1/m` the hybrid stays on
/// Simple. The smallest admitted quote `next_up(-m)` leaves the exact factor
/// `2^-53`, so the dirty price and all six risk views are exact.
#[test]
fn smallest_hybrid_quote_at_an_exact_simple_cutoff_has_the_exact_factor() {
    let issue = date(2024, 1, 1);
    for (frequency, count, maturity) in [
        (Frequency::Annual, 1.0, date(2025, 1, 1)),
        (Frequency::Semiannual, 2.0, date(2024, 7, 1)),
        (Frequency::Quarterly, 4.0, date(2024, 4, 1)),
    ] {
        let bond = zero_coupon(issue, maturity, 100.0, 100.0);
        let convention = BondYieldConvention::new(
            DayCount::Thirty360BondBasis,
            Compounding::simple_then_compounded(frequency).expect("periodic hybrid"),
        );
        let plan = plan(&bond, issue, PaymentDateInclusion::Exclude, &convention);
        assert_quote_domain(&plan.risk(-count).expect_err("quote -m"), -count, count);

        let risk = plan
            .risk((-count).next_up())
            .expect("smallest admitted quote");
        let square = count * count;
        let exact = [
            (risk.dirty_amount(), 100.0 * power(53)),
            (
                risk.price_derivative()
                    .expect("finite raw first derivative"),
                -100.0 * power(106) / count,
            ),
            (
                risk.price_second_derivative()
                    .expect("finite raw second derivative"),
                200.0 * power(159) / square,
            ),
            (
                risk.time_weighted_amount().expect("finite raw moment"),
                100.0 * power(53) / count,
            ),
            (
                risk.modified_duration().expect("finite modified duration"),
                power(53) / count,
            ),
            (
                risk.macaulay_duration().expect("finite Macaulay duration"),
                1.0 / count,
            ),
            (
                risk.convexity().expect("finite convexity"),
                power(107) / square,
            ),
        ];
        for (observed, expected) in exact {
            assert_eq!(observed.to_bits(), expected.to_bits(), "m={count}");
        }
    }
}

/// FR-YR-12: the quote domain precedes an active Simple factor check. A zero
/// or negative Simple factor then fails at its original index, for a hybrid
/// and for plain Simple alike, and inversion reports the identical cause.
#[test]
fn quote_validation_precedes_active_simple_factor_failures() {
    let issue = date(2024, 1, 1);
    // Thirty360 Bond Basis gives the exact horizon `t=2`, Simple after `1/m=1`.
    let bond = zero_coupon(issue, date(2026, 1, 1), 100.0, 100.0);
    let hybrid = BondYieldConvention::new(
        DayCount::Thirty360BondBasis,
        Compounding::compounded_then_simple(Frequency::Annual).expect("periodic hybrid"),
    );
    let simple = BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::SIMPLE);
    let hybrid_plan = plan(&bond, issue, PaymentDateInclusion::Exclude, &hybrid);
    let simple_plan = plan(&bond, issue, PaymentDateInclusion::Exclude, &simple);

    // `1-1*2` would also be an invalid factor; the quote fails first.
    let quote = hybrid_plan.risk(-1.0).expect_err("quote -m");
    assert_quote_domain(&quote, -1.0, 1.0);
    let failure = hybrid_plan
        .yield_from_price(
            dirty(100.0),
            RootBracket::new(-1.0, 1.0).expect("valid bracket"),
            options(),
        )
        .expect_err("lower quote fails");
    assert_lower_callback(&failure, -1.0, &quote);
    for (rate, factor) in [(-0.75, -0.5), (-0.5, 0.0)] {
        for plan in [&hybrid_plan, &simple_plan] {
            let error = plan
                .risk(rate)
                .expect_err("nonpositive active Simple factor");
            assert_factor_failure(&error, 0, factor);
            let bracket = RootBracket::new(rate, 1.0).expect("finite increasing bracket");
            let failure = plan
                .yield_from_price(dirty(100.0), bracket, options())
                .expect_err("lower factor fails");
            assert_lower_callback(&failure, rate, &error);
        }
    }
    // `next_up(-0.5)*2` is `-(1-2^-53)`, leaving the exact factor `2^-53`.
    for plan in [&hybrid_plan, &simple_plan] {
        let risk = plan
            .risk((-0.5_f64).next_up())
            .expect("smallest positive Simple factor");
        assert_eq!(risk.dirty_amount().to_bits(), (100.0 * power(53)).to_bits());
    }
}

/// FR-YR-12/14: opposite equal flows certify mathematical constancy, yet an
/// individual present value that overflows, or an unresolved factor range,
/// fails at its own index; exact cancellation never rescues it.
#[test]
fn invalid_individual_present_values_are_not_rescued_by_exact_cancellation() {
    let issue = date(2024, 1, 1);
    let maturity = date(2025, 1, 1);
    // Face 2^7, rate -2^1016 and year fraction 1 give the exact coupon -2^1023.
    let bond = Bond::fixed_rate(
        issue,
        forward_schedule(issue, maturity, Period::new(100, TimeUnit::Years)),
        128.0,
        FixedCouponRate::new(-power(1016), DayCount::Thirty360BondBasis)
            .expect("finite signed coupon"),
        vec![None],
        SimpleCashFlow::new(maturity, power(1023)).expect("positive redemption"),
        4,
    )
    .expect("valid public fixed-rate bond");
    let convention =
        BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::CONTINUOUS);
    let plan = plan(&bond, issue, PaymentDateInclusion::Exclude, &convention);
    assert_eq!(plan.entitled_flow_count(), 2);

    let constant = plan
        .yield_from_price(
            dirty(0.0),
            RootBracket::new(0.0, 1.0).expect("valid bracket"),
            options(),
        )
        .expect_err("opposite equal same-time flows are mathematically constant");
    assert!(matches!(
        constant,
        BondYieldError::NoYieldSensitivity {
            observed_lower_dirty_amount: 0.0,
            constant_amount: Some(0.0),
            evaluations,
            ..
        } if evaluations.function() == 1
    ));

    // A growing discount `e` overflows the first, negative, PV on its own.
    let overflow = plan.risk(-1.0).expect_err("individual PV overflow");
    assert!(
        matches!(
            &overflow,
            BondError::Rate {
                index: Some(0),
                phase: "discount factor",
                cause: RateError::Value(ValueError::ResultNotFinite {
                    quantity: "present value",
                    value: NonFinite::NegativeInfinity,
                    ..
                }),
                ..
            }
        ),
        "{overflow:?}"
    );
    let failure = plan
        .yield_from_price(
            dirty(0.0),
            RootBracket::new(-1.0, 1.0).expect("valid bracket"),
            options(),
        )
        .expect_err("the lower callback fails before the constant comparison");
    assert_lower_callback(&failure, -1.0, &overflow);

    // A Continuous exponent exactly at `ln(MAX)` is an unresolved range.
    let boundary = f64::MAX.ln();
    let unresolved = plan.risk(boundary).expect_err("unresolved factor range");
    assert!(
        matches!(&unresolved, BondError::Rate {
            index: Some(0),
            phase: "discount factor",
            cause: RateError::NumericalBoundary {
                quantity: "compound factor", logarithm, ..
            },
            ..
        } if logarithm.to_bits() == boundary.to_bits()),
        "{unresolved:?}"
    );
}

/// FR-YR-12/14: target conversion precedes every yield quote error, and a
/// valid target then reports the invalid lower quote from its callback.
#[test]
fn target_resolution_precedes_an_invalid_lower_quote() {
    let issue = date(2024, 1, 1);
    let bond = zero_coupon(issue, date(2025, 1, 1), f64::MAX, 100.0);
    let convention = BondYieldConvention::new(
        DayCount::Thirty360BondBasis,
        Compounding::compounded(Frequency::Annual).expect("periodic frequency"),
    );
    let plan = plan(&bond, issue, PaymentDateInclusion::Exclude, &convention);
    let bracket = RootBracket::new(-1.0, 1.0).expect("lower violates y>-1");

    // 200 per 100 of a MAX face overflows its single-rounded amount.
    let target = plan
        .yield_from_price(dirty(200.0), bracket, options())
        .expect_err("target conversion overflow");
    assert!(
        matches!(
            target,
            BondYieldError::Target {
                phase: "target conversion",
                ..
            }
        ),
        "{target:?}"
    );

    let quote = plan.risk(-1.0).expect_err("quote -m");
    assert_quote_domain(&quote, -1.0, 1.0);
    let failure = plan
        .yield_from_price(dirty(50.0), bracket, options())
        .expect_err("finite target, invalid lower quote");
    assert_lower_callback(&failure, -1.0, &quote);
}
