//! FR-YR-10/14: valid public construction with normalized-only overflow.
//!
//! Independent exact-binary control: a settlement coupon of -1 cancels a
//! redemption PV of +1. The tiny remaining coupon makes dirty price nonzero,
//! while a finite time-weighted amount divided by it overflows. Other getters
//! remain independently usable. No private report or extended state is forged.

use sabiquant::bonds::{Bond, BondError, BondPricingPlan, BondSettlement, BondYieldConvention};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{ActualActualIcma, DayCount, DeterminationPeriod};
use sabiquant::error::{Error as RootError, NonFinite, ValueError};
use sabiquant::rates::Compounding;
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, ScheduleBuilder, ScheduleCalendar,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::error::Error;
use std::ops::Bound;

#[test]
fn normalized_overflow_does_not_block_other_raw_or_normalized_getters() {
    let issue = Date::from_ymd(2023, 1, 1).expect("civil issue");
    let settlement = Date::from_ymd(2024, 1, 1).expect("civil settlement");
    let maturity = Date::from_ymd(2024, 1, 2).expect("civil maturity");
    let schedule = ScheduleBuilder::new(
        issue,
        maturity,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )
    .expect("unadjusted schedule definition")
    .build_explicit(
        Period::new(6, TimeUnit::Months),
        DateGenerationRule::Forward,
        ExplicitStubDates::new(Some(settlement), Some(settlement)).expect("one shared interior"),
        EndOfMonth::Ignore,
        4,
    )
    .expect("long first and short final coupon");
    let large = f64::from_bits(2046_u64 << 52); // 2^1023
    let bond = Bond::fixed_rate(
        issue,
        schedule,
        100.0,
        FixedCouponRate::new(-0.01, DayCount::Thirty360BondBasis).expect("signed coupon rate"),
        vec![None, None],
        SimpleCashFlow::new(maturity, large).expect("positive finite redemption"),
        32,
    )
    .expect("valid signed fixed-rate bond");
    // Deliberately independent coupon/yield contexts. ICMA gives exactly t=1
    // over the single civil day; the retained settlement coupon has t=0.
    let yield_context = ActualActualIcma::new(
        vec![DeterminationPeriod::new(settlement, maturity).expect("one-day determination")],
        Frequency::Annual,
    )
    .expect("explicit complete yield coverage");
    let convention = BondYieldConvention::new(yield_context, Compounding::SIMPLE);
    let plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(settlement, PaymentDateInclusion::Include),
        &convention,
        32,
    )
    .expect("prepared valid public plan");
    assert_eq!(plan.entitled_flow_count(), 3);

    // Exact Fraction derivation (no elementary-function allowance):
    // a=RN(100*binary64(-.01)/360), v=RN(a*2^-1023).
    // FMA(large,1,1)=large; red PV=1; dirty=-1+v+1=v.
    // Extended moment=1+v, derivative=-(1+v)*2^-1023,
    // second derivative=(1+v)*2^-2045. All per-term power shifts are exact.
    // One final RN of the direct ratios gives the pins below.
    let risk = plan
        .risk(large)
        .expect("only final views admit binary64 range");
    assert_eq!(risk.dirty_amount().to_bits(), 0x8000_05b0_5b05_b05b);
    assert_eq!(
        risk.time_weighted_amount()
            .expect("raw moment finite")
            .to_bits(),
        1.0_f64.to_bits()
    );
    assert_eq!(
        risk.price_derivative()
            .expect("raw first derivative finite")
            .to_bits(),
        0x8008_0000_0000_0000
    );
    assert_eq!(
        risk.modified_duration()
            .expect("direct derivative ratio finite")
            .to_bits(),
        0xc076_8000_0000_0017
    );
    assert_eq!(
        risk.convexity()
            .expect("direct second derivative ratio finite")
            .to_bits(),
        0x8096_8000_0000_0017
    );

    let overflow = risk
        .macaulay_duration()
        .expect_err("finite numerator/tiny dirty overflows");
    assert!(matches!(
        &overflow,
        BondError::Value {
            index: None,
            phase: "macaulay duration",
            cause: ValueError::ResultNotFinite {
                quantity: "macaulay duration",
                value: NonFinite::NegativeInfinity,
                ..
            },
            ..
        }
    ));
    let root = RootError::from(overflow.clone());
    assert_eq!(root.kind(), "bond.value");
    let source = root.source().expect("typed bond source");
    assert_eq!(source.downcast_ref::<BondError>(), Some(&overflow));
    assert!(source.source().expect("numeric cause").is::<ValueError>());
    assert!(source.source().expect("numeric cause").source().is_none());

    let underflow = risk
        .price_second_derivative()
        .expect_err("positive raw second derivative underflows");
    assert!(matches!(
        &underflow,
        BondError::Value {
            index: None,
            phase: "price second derivative",
            cause: ValueError::ResultOutOfDomain { .. },
            ..
        }
    ));
    if let BondError::Value {
        index: None,
        phase: "price second derivative",
        cause:
            ValueError::ResultOutOfDomain {
                quantity,
                value,
                domain,
                ..
            },
        ..
    } = &underflow
    {
        assert_eq!(*quantity, "price second derivative");
        assert_eq!(value.to_bits(), 0);
        assert_eq!(domain.lower(), Bound::Excluded(0.0));
        assert_eq!(domain.upper(), Bound::Unbounded);
    }
    // Failing two independent views cannot consume or mutate the retained sum.
    assert_eq!(
        risk.macaulay_duration().expect_err("same overflow"),
        overflow
    );
    assert_eq!(
        risk.price_second_derivative().expect_err("same underflow"),
        underflow
    );
    assert_eq!(
        risk.time_weighted_amount().expect("still usable").to_bits(),
        1.0_f64.to_bits()
    );
}
