//! Post-production coverage supplement: FR-CAD-13/15/36/42/52/61/69/71.
//! Authored after the first Canadian coverage run showed these valid public
//! paths unexecuted. This is not a frozen blind hand. Expectations use
//! independent exact arithmetic on a three-coupon semiannual bond with a
//! 90-day short last period in a 181-day quasi cell, and on a forty-period
//! zero-coupon bond whose compound factors are exact powers of two.

#![expect(
    clippy::expect_used,
    reason = "explicit valid supplemental test inputs"
)]

use sabiquant::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    CanadianBond, CanadianCashDecision, CanadianCashInstruction, CanadianCouponSchedule,
    CanadianError, CanadianFrequency, CanadianPeriodKind, CanadianPricingPlan,
    CanadianSourcePolicy, CanadianYieldConvention, CanadianYieldSelection,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::ValueError;
use sabiquant::rates::RateError;
use sabiquant::settlement::SettlementRounding;
use sabiquant::time::Date;

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid civil date")
}

fn amount(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("valid exact amount")
}

/// Coupon 5 per100, coupons 2000-07-01, 2001-01-01 and maturity 2001-04-01.
fn short_last() -> CanadianBond {
    let schedule = CanadianCouponSchedule::new(
        date(2000, 1, 1),
        date(2001, 4, 1),
        CanadianFrequency::Semiannual,
        vec![
            date(2000, 1, 1),
            date(2000, 7, 1),
            date(2001, 1, 1),
            date(2001, 7, 1),
        ],
        vec![date(2000, 7, 1), date(2001, 1, 1), date(2001, 4, 1)],
        EndOfMonth::Ignore,
        4,
    )
    .expect("regular first, interior and short last periods");
    CanadianBond::new(
        date(2000, 1, 1),
        amount(5, 0),
        schedule,
        date(2001, 4, 1),
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )
    .expect("valid instrument")
}

#[expect(clippy::panic, reason = "a different cash decision fails the control")]
fn cash(instruction: &CanadianCashInstruction) -> DecimalAmount {
    match instruction.evaluate().expect("admitted cash") {
        CanadianCashDecision::Determined(receipt) => receipt.amount(),
        other => panic!("expected a determined receipt, got {other:?}"),
    }
}

/// FR-CAD-13/15: interior and last ordinals report their own period kinds.
#[test]
fn schedule_inspectors_distinguish_interior_and_short_last_periods() {
    let bond = short_last();
    let schedule = bond.schedule();
    assert_eq!(schedule.last_period(), CanadianPeriodKind::Short);
    let kinds: Vec<_> = (0..4).map(|i| schedule.coupon_period_kind(i)).collect();
    assert_eq!(
        kinds,
        [
            Some(CanadianPeriodKind::Regular),
            Some(CanadianPeriodKind::Regular),
            Some(CanadianPeriodKind::Short),
            None,
        ]
    );
    assert_eq!(schedule.coupon_start_date(1), Some(date(2000, 7, 1)));
    assert_eq!(schedule.coupon_start_date(2), Some(date(2001, 1, 1)));
}

/// FR-CAD-61/69: an interior coupon is the exact regular C/f; the short last
/// payable coupon is C*90/365 = 90/73 per100. Nominal 1,000,000 gives 25,000
/// and 900,000/73 = 12,328.767..., which nearest/even publishes as 12,328.77.
#[test]
fn interior_and_short_last_coupon_cash_keep_exact_sources() {
    let bond = short_last();
    let nominal = amount(1_000_000, 0);
    let cents = AmountQuantum::new(1, 2).expect("cent quantum");
    let interior = bond
        .coupon_cash(
            1,
            nominal,
            SettlementRounding::new(cents, AmountRounding::Exact),
        )
        .expect("interior coupon cash");
    assert_eq!(cash(&interior), amount(25_000, 0));
    assert_eq!(interior.source_coupon_index(), Some(1));
    assert_eq!(
        interior.source_interval(),
        Some((date(2000, 7, 1), date(2001, 1, 1)))
    );
    assert_eq!(interior.source_full_period_count(), Some(1));
    assert_eq!(interior.source_day_ratios(), None);
    let cell = (date(2000, 7, 1), date(2001, 1, 1));
    assert_eq!(interior.source_quasi_endpoints(), Some((cell, cell)));

    let last = bond
        .coupon_cash(
            2,
            nominal,
            SettlementRounding::new(cents, AmountRounding::NearestTiesEven),
        )
        .expect("short last coupon cash");
    assert_eq!(cash(&last), amount(1_232_877, 2));
    assert_eq!(
        last.source_interval(),
        Some((date(2001, 1, 1), date(2001, 4, 1)))
    );
    assert_eq!(last.source_full_period_count(), Some(0));
    assert_eq!(last.source_day_ratios(), Some(((90, 181), None)));
    assert_eq!(
        bond.payable_coupon_per_100(
            2,
            AmountQuantum::new(1, 6).expect("micro quantum"),
            AmountRounding::NearestTiesEven,
        ),
        Ok(amount(1_232_877, 6))
    );
}

/// FR-CAD-42/52: plan policy and the risk report's three price bridges equal
/// the valuation's own one-shot observations for both conventions.
#[test]
fn plan_policy_and_risk_price_bridges_match_the_valuation() {
    let bond = short_last();
    for convention in [
        CanadianYieldConvention::Compound,
        CanadianYieldConvention::MoneyMarket,
    ] {
        let plan = CanadianPricingPlan::new(
            &bond,
            date(2000, 3, 1),
            CanadianYieldSelection::Explicit(convention),
            3,
        )
        .expect("three future bundles");
        assert_eq!(
            plan.source_policy(),
            CanadianSourcePolicy::CanonicalIiac13LocalV1
        );
        let valuation = plan.evaluate(0.05).expect("ordinary quote");
        let risk = plan.risk(0.05).expect("ordinary risk");
        assert_eq!(risk.model_dirty_price(), valuation.model_dirty_price());
        assert_eq!(risk.clean_price(), valuation.clean_price());
        assert_eq!(
            risk.settlement_dirty_price(),
            valuation.settlement_dirty_price()
        );
    }
}

/// FR-CAD-13/71: out-of-range ordinals are typed misses, not clamped.
#[test]
fn out_of_range_coupon_ordinals_are_reported_without_clamping() {
    let bond = short_last();
    assert_eq!(bond.coupon_payment_date(2), Some(date(2001, 4, 1)));
    assert_eq!(bond.coupon_payment_date(3), None);
    assert!(matches!(
        bond.valuation_coupon_per_100(
            3,
            AmountQuantum::new(1, 6).expect("micro quantum"),
            AmountRounding::Exact,
        ),
        Err(CanadianError::InvalidCouponIndex {
            index: 3,
            count: 3,
            ..
        })
    ));
}

/// Twenty-year semiannual zero coupon settled at its accrual start; ordinal k
/// is discounted over k+1 whole periods by (1+y/2)^-(k+1).
fn forty_period_zero() -> CanadianBond {
    let start = date(2000, 1, 1);
    let quasi: Vec<_> = (0..=40)
        .map(|i| {
            start
                .checked_add_months(6 * i)
                .expect("valid semiannual date")
        })
        .collect();
    let schedule = CanadianCouponSchedule::new(
        start,
        quasi[40],
        CanadianFrequency::Semiannual,
        quasi.clone(),
        quasi[1..].to_vec(),
        EndOfMonth::Ignore,
        41,
    )
    .expect("regular direct-anchored cycle");
    CanadianBond::new(
        start,
        amount(0, 0),
        schedule,
        quasi[40],
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )
    .expect("valid zero-coupon instrument")
}

/// FR-CAD-36/71: evaluate and risk stop at the first failing original ordinal.
/// With 1+y/2 = 2^-k, ordinal j has factor 2^(k(j+1)), which overflows exactly
/// when k(j+1) >= 1024: k=26 first fails at j=39 and k=28 at j=36. At k=25.5
/// every factor is finite but 100*2^1020 overflows the present value at j=39.
/// At y=10^300 the second short-last bundle's factor underflows to zero.
#[test]
fn factor_and_present_value_failures_name_the_first_failing_ordinal() {
    let zero = forty_period_zero();
    let compound = CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound);
    let plan = CanadianPricingPlan::new(&zero, date(2000, 1, 1), compound, 40)
        .expect("forty future bundles");
    for (exponent, ordinal) in [(26, 39), (28, 36)] {
        let y = -2.0 + 2.0 * f64::from_bits((1023 - exponent) << 52);
        for result in [plan.evaluate(y).map(|_| ()), plan.risk(y).map(|_| ())] {
            assert!(
                matches!(
                    result,
                    Err(CanadianError::Rate {
                        index: Some(index),
                        phase: "discount factor",
                        cause: RateError::Value(ValueError::ResultNotFinite { .. }),
                        ..
                    }) if index == ordinal
                ),
                "k={exponent}: {result:?}"
            );
        }
    }
    let y = -2.0 + 2.0 * (-25.5_f64).exp2();
    for result in [plan.evaluate(y).map(|_| ()), plan.risk(y).map(|_| ())] {
        assert!(
            matches!(
                result,
                Err(CanadianError::Value {
                    index: Some(39),
                    phase: "source present value",
                    cause: ValueError::ResultNotFinite { .. },
                    ..
                })
            ),
            "{result:?}"
        );
    }
    let bond = short_last();
    let plan = CanadianPricingPlan::new(&bond, date(2000, 3, 1), compound, 3)
        .expect("three future bundles");
    for result in [
        plan.evaluate(1e300).map(|_| ()),
        plan.risk(1e300).map(|_| ()),
    ] {
        assert!(
            matches!(
                result,
                Err(CanadianError::Rate {
                    index: Some(1),
                    phase: "discount factor",
                    cause: RateError::Value(ValueError::ResultOutOfDomain { .. }),
                    ..
                })
            ),
            "{result:?}"
        );
    }
}
