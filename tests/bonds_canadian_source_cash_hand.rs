//! Independently frozen public source/cash hands, before Canadian production.
//! FR-CAD-09,15..32,61..71. Rational derivations live in hands-rational.json.
//! Executed integration copy of validation/hand/bonds_canadian_source_cash_hand.rs:
//! explicit imports replace `use sabiquant::*` and one unfulfillable test-fn
//! `clippy::panic` expectation is dropped; every input and assertion is unchanged.
use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    CanadianBond, CanadianCashDecision, CanadianCashInstruction, CanadianCashReceipt,
    CanadianCouponSchedule, CanadianError, CanadianFrequency, CanadianPeriodKind,
    CanadianPricingPlan, CanadianSourcePolicy, CanadianYieldConvention, CanadianYieldSelection,
    CouponEntitlement,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::settlement::{
    CashAggregation, SettlementEvidence, SettlementGuarantee, SettlementRounding,
};
use sabiquant::time::Date;
use std::collections::HashSet;
type Checked<T = ()> = Result<T, Box<dyn std::error::Error>>;
macro_rules! d {
    ($y:literal,$m:literal,$day:literal) => {
        Date::from_ymd($y, $m, $day)?
    };
}
fn a(c: i128, s: u8) -> Checked<DecimalAmount> {
    Ok(DecimalAmount::new(c, s)?)
}
fn rounding(c: i128, s: u8, m: AmountRounding) -> Checked<SettlementRounding> {
    Ok(SettlementRounding::new(AmountQuantum::new(c, s)?, m))
}
fn semi(c: i128, s: u8) -> Checked<CanadianBond> {
    let schedule = CanadianCouponSchedule::new(
        d!(2025, 3, 1),
        d!(2026, 3, 1),
        CanadianFrequency::Semiannual,
        vec![d!(2025, 3, 1), d!(2025, 9, 1), d!(2026, 3, 1)],
        vec![d!(2025, 9, 1), d!(2026, 3, 1)],
        EndOfMonth::Ignore,
        3,
    )?;
    Ok(CanadianBond::new(
        d!(2025, 3, 1),
        a(c, s)?,
        schedule,
        d!(2026, 3, 1),
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?)
}
fn determined(i: &CanadianCashInstruction) -> Checked<CanadianCashReceipt> {
    match i.evaluate()? {
        CanadianCashDecision::Determined(r) => Ok(r),
        other => Err(format!("expected Determined, got {other:?}").into()),
    }
}
#[expect(
    clippy::panic,
    reason = "A hand oracle must fail if the structured cash error differs."
)]
fn check_inexact(result: Result<CanadianCashDecision, CanadianError>, stage: &'static str) {
    match result {
        Err(CanadianError::CashArithmetic {
            stage: actual,
            endpoint: "point",
            cause:
                AmountError::Inexact {
                    operation: "quantize",
                    ..
                },
            ..
        }) => assert_eq!(actual, stage),
        other => panic!("expected point Inexact at {stage}, got {other:?}"),
    }
}
#[test]
fn dirty_all_signed_modes_and_no_source_quantizer() -> Checked {
    let b = semi(0, 0)?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 3, 2),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    let modes = [
        AmountRounding::TowardZero,
        AmountRounding::AwayFromZero,
        AmountRounding::TowardPositiveInfinity,
        AmountRounding::TowardNegativeInfinity,
        AmountRounding::NearestTiesEven,
        AmountRounding::NearestTiesAwayFromZero,
    ];
    for (j, m) in modes.into_iter().enumerate() {
        for (sign, expected) in [
            (1, [2, 3, 3, 2, 2, 3][j]),
            (-1, [-2, -3, -2, -3, -2, -3][j]),
        ] {
            let i = p.dirty_cash(
                a(1, 0)?,
                SettlementEvidence::exact_decimal(a(sign * 25, 1)?),
                rounding(1, 2, m)?,
            )?;
            let r = determined(&i)?;
            assert_eq!(r.amount(), a(expected, 2)?);
            assert_eq!(
                r.price_evidence_guarantee(),
                Some(SettlementGuarantee::ForSuppliedDecimals)
            );
            assert_eq!(i.recipe_identifier(), "SettlementDirtyPricePer100");
            assert_eq!(i.source_coupon_index(), None);
            assert_eq!(i.source_interval(), None);
            assert_eq!(i.source_quasi_endpoints(), None);
            assert_eq!(
                i.source_value_per_100(AmountQuantum::new(1, 38)?, AmountRounding::Exact)?,
                None
            );
        }
    }
    let exact = p.dirty_cash(
        a(1, 0)?,
        SettlementEvidence::exact_decimal(a(25, 1)?),
        rounding(1, 2, AmountRounding::Exact)?,
    )?;
    check_inexact(exact.evaluate(), "cash total");
    Ok(())
}
#[test]
#[expect(
    clippy::unreachable,
    reason = "This loop contains only the three explicitly frozen Canadian frequency variants."
)]
fn regular_coupons_every_frequency_and_terminal_excludes_principal() -> Checked {
    for f in [
        CanadianFrequency::Semiannual,
        CanadianFrequency::Quarterly,
        CanadianFrequency::Monthly,
    ] {
        let (start, end, q, coupons, n) = match f {
            CanadianFrequency::Semiannual => (
                d!(2025, 3, 1),
                d!(2026, 3, 1),
                vec![d!(2025, 3, 1), d!(2025, 9, 1), d!(2026, 3, 1)],
                vec![d!(2025, 9, 1), d!(2026, 3, 1)],
                1,
            ),
            CanadianFrequency::Quarterly => (
                d!(2025, 3, 1),
                d!(2025, 9, 1),
                vec![d!(2025, 3, 1), d!(2025, 6, 1), d!(2025, 9, 1)],
                vec![d!(2025, 6, 1), d!(2025, 9, 1)],
                2,
            ),
            CanadianFrequency::Monthly => (
                d!(2025, 1, 1),
                d!(2025, 3, 1),
                vec![d!(2025, 1, 1), d!(2025, 2, 1), d!(2025, 3, 1)],
                vec![d!(2025, 2, 1), d!(2025, 3, 1)],
                6,
            ),
            _ => unreachable!(),
        };
        let s = CanadianCouponSchedule::new(start, end, f, q, coupons, EndOfMonth::Ignore, 3)?;
        let b = CanadianBond::new(
            start,
            a(5, 0)?,
            s,
            end,
            CanadianSourcePolicy::CanonicalIiac13LocalV1,
        )?;
        let i = b.coupon_cash(
            1,
            a(n, 0)?,
            rounding(1, 2, AmountRounding::NearestTiesEven)?,
        )?;
        assert_eq!(determined(&i)?.amount(), a(2, 2)?);
        assert_eq!(i.recipe_identifier(), "PayableCouponPer100");
        assert_eq!(i.source_coupon_index(), Some(1));
        assert_eq!(i.settlement_date(), None);
        check_inexact(
            b.coupon_cash(1, a(n, 0)?, rounding(1, 2, AmountRounding::Exact)?)?
                .evaluate(),
            "cash coupon",
        );
    }
    let z = semi(0, 0)?;
    let i = z.coupon_cash(1, a(100, 0)?, rounding(1, 38, AmountRounding::Exact)?)?;
    assert_eq!(determined(&i)?.amount(), a(0, 0)?);
    assert!(i.source_interval().is_some());
    assert_eq!(i.source_full_period_count(), Some(1));
    assert_eq!(i.source_day_ratios(), None);
    Ok(())
}
#[test]
fn late184_drop_completed_period_and_exact_view_failure() -> Checked {
    let b = semi(5, 0)?;
    let q = AmountQuantum::new(1, 12)?;
    let p182 = CanadianPricingPlan::new(
        &b,
        d!(2025, 8, 30),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    let p183 = CanadianPricingPlan::new(
        &b,
        d!(2025, 8, 31),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    assert_eq!(
        p182.settlement_accrued_at_quantum(q, AmountRounding::NearestTiesEven)?,
        a(2493150684932, 12)?
    );
    assert_eq!(
        p183.settlement_accrued_at_quantum(q, AmountRounding::NearestTiesEven)?,
        a(2486301369863, 12)?
    );
    assert_eq!(
        b.payable_coupon_per_100(0, AmountQuantum::new(1, 1)?, AmountRounding::Exact)?,
        a(25, 1)?
    );
    assert!(matches!(
        p183.settlement_accrued_at_quantum(AmountQuantum::new(1, 0)?, AmountRounding::Exact),
        Err(CanadianError::CashArithmetic {
            stage: "source settlement accrued",
            endpoint: "point",
            cause: AmountError::Inexact {
                operation: "quantize",
                ..
            },
            ..
        })
    ));
    let i = p183.accrued_cash(a(100, 0)?, rounding(1, 2, AmountRounding::NearestTiesEven)?)?;
    assert_eq!(determined(&i)?.amount(), a(249, 2)?);
    Ok(())
}
#[test]
fn monthly_composite_single_fragment_and_ai_policy() -> Checked {
    let s = CanadianCouponSchedule::new(
        d!(2025, 1, 2),
        d!(2025, 1, 20),
        CanadianFrequency::Monthly,
        vec![d!(2025, 1, 1), d!(2025, 2, 1)],
        vec![d!(2025, 1, 20)],
        EndOfMonth::Ignore,
        2,
    )?;
    assert_eq!(s.first_period(), CanadianPeriodKind::Composite);
    assert_eq!(s.last_period(), CanadianPeriodKind::Composite);
    let b = CanadianBond::new(
        d!(2025, 1, 2),
        a(5, 0)?,
        s,
        d!(2025, 1, 22),
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?;
    let q = AmountQuantum::new(1, 12)?;
    assert_eq!(
        b.valuation_coupon_per_100(0, q, AmountRounding::NearestTiesEven)?,
        a(241935483871, 12)?
    );
    assert_eq!(
        b.payable_coupon_per_100(0, q, AmountRounding::NearestTiesEven)?,
        a(246575342466, 12)?
    );
    let i = b.coupon_cash(
        0,
        a(100, 0)?,
        rounding(1, 2, AmountRounding::NearestTiesEven)?,
    )?;
    assert_eq!(i.source_full_period_count(), Some(0));
    assert_eq!(i.source_day_ratios(), Some(((18, 31), None)));
    assert_eq!(i.source_payment_date(), Some(d!(2025, 1, 22)));
    assert_eq!(
        i.source_quasi_endpoints(),
        Some((
            (d!(2025, 1, 1), d!(2025, 2, 1)),
            (d!(2025, 1, 1), d!(2025, 2, 1))
        ))
    );
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 1, 19),
        CanadianYieldSelection::LastCouponMoneyMarket,
        1,
    )?;
    assert_eq!(
        p.valuation_accrued_at_quantum(q, AmountRounding::NearestTiesEven)?,
        p.settlement_accrued_at_quantum(q, AmountRounding::NearestTiesEven)?
    );
    assert_eq!(
        p.settlement_accrued_at_quantum(q, AmountRounding::NearestTiesEven)?,
        a(228494623656, 12)?
    );
    Ok(())
}
#[test]
fn exact_ai_tie_components_versus_total_and_recipe_metadata() -> Checked {
    let b = semi(9125, 3)?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 3, 2),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    let r = rounding(1, 2, AmountRounding::NearestTiesEven)?;
    let x = SettlementEvidence::exact_decimal(a(25, 3)?);
    let t = p.clean_accrued_cash(a(100, 0)?, x, CashAggregation::RoundTotal, r)?;
    let c = p.clean_accrued_cash(a(100, 0)?, x, CashAggregation::RoundComponentsThenSum, r)?;
    assert_eq!(determined(&t)?.amount(), a(5, 2)?);
    assert_eq!(determined(&c)?.amount(), a(4, 2)?);
    assert_ne!(t, c);
    assert_eq!(
        t.recipe_identifier(),
        "CleanPriceAndSettlementAccruedPer100"
    );
    let ai = p.accrued_cash(a(100, 0)?, r)?;
    assert_eq!(ai.recipe_identifier(), "SettlementAccruedPer100");
    assert_eq!(determined(&ai)?.amount(), a(2, 2)?);
    assert_eq!(determined(&ai)?.price_evidence_guarantee(), None);
    assert_eq!(ai.source_interval(), Some((d!(2025, 3, 1), d!(2025, 3, 2))));
    assert_eq!(ai.source_day_ratios(), Some(((1, 184), None)));
    assert_eq!(
        ai.source_value_per_100(AmountQuantum::new(1, 3)?, AmountRounding::Exact)?,
        Some(a(25, 3)?)
    );
    let bad = p.clean_accrued_cash(
        a(1, 0)?,
        SettlementEvidence::exact_decimal(a(-25, 3)?),
        CashAggregation::RoundComponentsThenSum,
        rounding(1, 2, AmountRounding::Exact)?,
    )?;
    check_inexact(bad.evaluate(), "cash capital");
    Ok(())
}
#[test]
fn continuous_exact_equal_bounds_and_endpoint_ambiguity_with_holes() -> Checked {
    let b = semi(0, 0)?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 3, 2),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket),
        2,
    )?;
    let i = p.dirty_cash(
        a(100, 0)?,
        SettlementEvidence::declared_decimal_interval(a(0, 0)?, a(1, 0)?)?,
        rounding(1, 0, AmountRounding::Exact)?,
    )?;
    assert!(matches!(
        i.evaluate()?,
        CanadianCashDecision::ExactStageUnresolved {
            stage: "cash total",
            ..
        }
    ));
    let equal = SettlementEvidence::declared_decimal_interval(a(25, 1)?, a(25, 1)?)?;
    let j = p.dirty_cash(
        a(1, 0)?,
        equal,
        rounding(1, 2, AmountRounding::NearestTiesEven)?,
    )?;
    assert_eq!(determined(&j)?.amount(), a(2, 2)?);
    assert_eq!(
        determined(&j)?.price_evidence_guarantee(),
        Some(SettlementGuarantee::UniqueForDeclaredEnclosure)
    );
    let hole = p.dirty_cash(
        a(100, 0)?,
        SettlementEvidence::declared_decimal_interval(a(i128::MAX - 1, 0)?, a(i128::MAX, 0)?)?,
        rounding(1, 38, AmountRounding::NearestTiesEven)?,
    )?;
    match hole.evaluate()? {
        CanadianCashDecision::QuantumAmbiguous {
            lower_amount,
            upper_amount,
            ..
        } => {
            assert_eq!(lower_amount, a(i128::MAX - 1, 0)?);
            assert_eq!(upper_amount, a(i128::MAX, 0)?);
        }
        other => panic!("expected endpoint ambiguity, got {other:?}"),
    }
    let collapse = p.dirty_cash(
        a(100, 0)?,
        SettlementEvidence::declared_decimal_interval(a(-1, 3)?, a(1, 3)?)?,
        rounding(1, 2, AmountRounding::TowardZero)?,
    )?;
    assert_eq!(determined(&collapse)?.amount(), a(0, 0)?);
    Ok(())
}
fn wide_bond() -> Checked<CanadianBond> {
    let s = CanadianCouponSchedule::new(
        d!(2024, 1, 1),
        d!(2027, 1, 1),
        CanadianFrequency::Semiannual,
        vec![
            d!(2024, 1, 1),
            d!(2024, 7, 1),
            d!(2025, 1, 1),
            d!(2025, 7, 1),
            d!(2026, 1, 1),
            d!(2026, 7, 1),
            d!(2027, 1, 1),
        ],
        vec![d!(2027, 1, 1)],
        EndOfMonth::Ignore,
        7,
    )?;
    Ok(CanadianBond::new(
        d!(2024, 1, 1),
        a(1_i128 << 126, 0)?,
        s,
        d!(2027, 1, 1),
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?)
}
#[test]
fn source_view_range_is_independent_of_wide_net_cancellation() -> Checked {
    let b = wide_bond()?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2026, 1, 1),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        1,
    )?;
    assert!(matches!(
        p.settlement_accrued_at_quantum(AmountQuantum::new(1, 0)?, AmountRounding::Exact),
        Err(CanadianError::CashArithmetic {
            stage: "source settlement accrued",
            endpoint: "point",
            cause: AmountError::OutOfRange {
                operation: "quantize",
                ..
            },
            ..
        })
    ));
    for (n, m) in [
        (100, AmountRounding::Exact),
        (i128::MAX, AmountRounding::NearestTiesEven),
    ] {
        let x = SettlementEvidence::supplied_binary64(-2.0_f64.powi(127))?;
        let i = p.clean_accrued_cash(
            a(n, 0)?,
            x,
            CashAggregation::RoundComponentsThenSum,
            rounding(1, 0, m)?,
        )?;
        assert_eq!(determined(&i)?.amount(), a(0, 0)?);
        assert_eq!(i.source_full_period_count(), Some(4));
        assert_eq!(i.source_day_ratios(), None);
    }
    let bad = p.dirty_cash(
        a(100, 0)?,
        SettlementEvidence::declared_binary64_interval(2.0_f64.powi(127), 2.0_f64.powi(128))?,
        rounding(1, 0, AmountRounding::NearestTiesEven)?,
    )?;
    assert!(matches!(
        bad.evaluate(),
        Err(CanadianError::CashArithmetic {
            stage: "cash total",
            endpoint: "lower",
            cause: AmountError::OutOfRange {
                operation: "quantize",
                ..
            },
            ..
        })
    ));
    // Capital collapses to a point before net admission: endpoint must be point.
    let net = p.clean_accrued_cash(
        a(100, 0)?,
        SettlementEvidence::declared_decimal_interval(a(-1, 3)?, a(1, 3)?)?,
        CashAggregation::RoundComponentsThenSum,
        rounding(1, 0, AmountRounding::TowardZero)?,
    )?;
    assert!(matches!(
        net.evaluate(),
        Err(CanadianError::CashArithmetic {
            stage: "cash net",
            endpoint: "point",
            cause: AmountError::OutOfRange {
                operation: "sum",
                ..
            },
            ..
        })
    ));
    Ok(())
}
#[test]
fn ceased_original_ordinal_empty_ai_and_factory_precedence() -> Checked {
    let b = semi(0, 0)?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 9, 1),
        CanadianYieldSelection::LastCouponMoneyMarket,
        1,
    )?;
    assert_eq!(p.bundle_coupon_index(0), Some(1));
    assert_eq!(p.coupon_entitlement(0), Some(CouponEntitlement::Ceased));
    assert_eq!(
        p.coupon_entitlement(1),
        Some(CouponEntitlement::BuyerEntitled)
    );
    assert_eq!(p.coupon_entitlement(2), None);
    let i = p.accrued_cash(a(1, 0)?, rounding(1, 38, AmountRounding::Exact)?)?;
    assert_eq!(i.source_coupon_index(), Some(1));
    assert_eq!(i.source_interval(), Some((d!(2025, 9, 1), d!(2025, 9, 1))));
    assert_eq!(i.source_day_ratios(), None);
    assert_eq!(i.source_full_period_count(), Some(0));
    assert_eq!(
        i.source_quasi_endpoints(),
        Some((
            (d!(2025, 9, 1), d!(2026, 3, 1)),
            (d!(2025, 9, 1), d!(2026, 3, 1))
        ))
    );
    assert!(matches!(
        b.coupon_cash(99, a(0, 0)?, rounding(1, 2, AmountRounding::Exact)?),
        Err(CanadianError::InvalidCouponIndex {
            index: 99,
            count: 2,
            ..
        })
    ));
    assert!(matches!(
        p.accrued_cash(a(0, 0)?, rounding(1, 2, AmountRounding::Exact)?),
        Err(CanadianError::NonPositiveNominal { .. })
    ));
    Ok(())
}
fn identity_bond(extra: bool, c: i128, s: u8) -> Checked<CanadianBond> {
    let mut actual = vec![d!(2024, 7, 1), d!(2025, 1, 1)];
    if extra {
        actual.push(d!(2025, 7, 1));
    }
    actual.push(d!(2027, 1, 1));
    let schedule = CanadianCouponSchedule::new(
        d!(2024, 1, 1),
        d!(2027, 1, 1),
        CanadianFrequency::Semiannual,
        vec![
            d!(2024, 1, 1),
            d!(2024, 7, 1),
            d!(2025, 1, 1),
            d!(2025, 7, 1),
            d!(2026, 1, 1),
            d!(2026, 7, 1),
            d!(2027, 1, 1),
        ],
        actual,
        EndOfMonth::Ignore,
        7,
    )?;
    Ok(CanadianBond::new(
        d!(2024, 1, 1),
        a(c, s)?,
        schedule,
        d!(2027, 1, 1),
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?)
}
#[test]
fn complete_identity_retains_l_and_evidence_provenance() -> Checked {
    let b1 = identity_bond(false, 5, 0)?;
    let b2 = identity_bond(true, 5, 0)?;
    let b_alias = identity_bond(false, 50, 1)?;
    assert_eq!(b1, b_alias);
    assert_ne!(b1, b2);
    assert_eq!(b1.source_version(), "CA-IIAC13-fixed-2026-10-23-local-v1");
    let r = rounding(1, 2, AmountRounding::Exact)?;
    let i1 = b1.coupon_cash(0, a(100, 0)?, r)?;
    let i2 = b2.coupon_cash(0, a(100, 0)?, r)?;
    assert_eq!(determined(&i1)?.amount(), determined(&i2)?.amount());
    assert_ne!(i1, i2);
    assert_eq!(i1.last_coupon_start_date(), d!(2025, 1, 1));
    assert_eq!(i2.last_coupon_start_date(), d!(2025, 7, 1));
    let mut set = HashSet::new();
    set.insert(i1);
    set.insert(i2);
    set.insert(i1);
    assert_eq!(set.len(), 2);
    let p = CanadianPricingPlan::new(
        &b1,
        d!(2024, 2, 1),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        3,
    )?;
    let point = p.dirty_cash(a(100, 0)?, SettlementEvidence::exact_decimal(a(25, 1)?), r)?;
    let binary = p.dirty_cash(a(100, 0)?, SettlementEvidence::supplied_binary64(2.5)?, r)?;
    let declared = p.dirty_cash(
        a(100, 0)?,
        SettlementEvidence::declared_decimal_interval(a(25, 1)?, a(25, 1)?)?,
        r,
    )?;
    assert_eq!(determined(&point)?.amount(), determined(&binary)?.amount());
    let p2 = CanadianPricingPlan::new(
        &b2,
        d!(2024, 2, 1),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        4,
    )?;
    let other_l = p2.dirty_cash(a(100, 0)?, SettlementEvidence::exact_decimal(a(25, 1)?), r)?;
    assert_eq!(determined(&point)?.amount(), determined(&other_l)?.amount());
    assert_ne!(point, other_l);
    assert_eq!(other_l.last_coupon_start_date(), d!(2025, 7, 1));
    assert_eq!(
        determined(&point)?.amount(),
        determined(&declared)?.amount()
    );
    assert_ne!(point, binary);
    assert_ne!(point, declared);
    assert_ne!(binary, declared);
    assert_eq!(
        determined(&binary)?.price_evidence_guarantee(),
        Some(SettlementGuarantee::ForSuppliedBinary64)
    );
    assert_eq!(
        determined(&declared)?.price_evidence_guarantee(),
        Some(SettlementGuarantee::UniqueForDeclaredEnclosure)
    );
    Ok(())
}

#[test]
fn four_factories_seven_modes_and_both_invoice_aggregations() -> Checked {
    let b = semi(9125, 3)?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 3, 2),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    let cb = semi(5, 0)?;
    let modes = [
        AmountRounding::TowardZero,
        AmountRounding::AwayFromZero,
        AmountRounding::TowardPositiveInfinity,
        AmountRounding::TowardNegativeInfinity,
        AmountRounding::NearestTiesEven,
        AmountRounding::NearestTiesAwayFromZero,
    ];
    for (j, m) in modes.into_iter().enumerate() {
        let r = rounding(1, 2, m)?;
        let expected = a([2, 3, 3, 2, 2, 3][j], 2)?;
        assert_eq!(
            determined(&cb.coupon_cash(0, a(1, 0)?, r)?)?.amount(),
            expected
        );
        assert_eq!(
            determined(&p.accrued_cash(a(100, 0)?, r)?)?.amount(),
            expected
        );
        assert_eq!(
            determined(&p.dirty_cash(
                a(1, 0)?,
                SettlementEvidence::exact_decimal(a(25, 1)?),
                r
            )?)?
            .amount(),
            expected
        );
        for aggregation in [
            CashAggregation::RoundTotal,
            CashAggregation::RoundComponentsThenSum,
        ] {
            let i = p.clean_accrued_cash(
                a(100, 0)?,
                SettlementEvidence::exact_decimal(a(25, 3)?),
                aggregation,
                r,
            )?;
            let amount = if aggregation == CashAggregation::RoundTotal {
                a(5, 2)?
            } else {
                a([4, 6, 6, 4, 4, 6][j], 2)?
            };
            assert_eq!(determined(&i)?.amount(), amount);
        }
    }
    let exact = rounding(1, 2, AmountRounding::Exact)?;
    check_inexact(
        cb.coupon_cash(0, a(1, 0)?, exact)?.evaluate(),
        "cash coupon",
    );
    check_inexact(
        p.accrued_cash(a(100, 0)?, exact)?.evaluate(),
        "cash accrued",
    );
    let total = p.clean_accrued_cash(
        a(100, 0)?,
        SettlementEvidence::exact_decimal(a(25, 3)?),
        CashAggregation::RoundTotal,
        exact,
    )?;
    assert_eq!(determined(&total)?.amount(), a(5, 2)?);
    check_inexact(
        p.clean_accrued_cash(
            a(100, 0)?,
            SettlementEvidence::exact_decimal(a(25, 3)?),
            CashAggregation::RoundComponentsThenSum,
            exact,
        )?
        .evaluate(),
        "cash capital",
    );
    Ok(())
}
#[test]
fn all39_nominal_scales_keep_exact_source_before_quantum() -> Checked {
    let b = semi(9125, 3)?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 3, 2),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    for s in 0..=38 {
        let i = p.accrued_cash(a(1, s)?, rounding(1, 38, AmountRounding::NearestTiesEven)?)?;
        let expected = if s <= 33 {
            a(25, s + 5)?
        } else if s == 34 {
            a(2, 38)?
        } else {
            a(0, 0)?
        };
        assert_eq!(determined(&i)?.amount(), expected);
        assert_eq!(
            determined(
                &p.accrued_cash(a(1, s)?, rounding(1, 2, AmountRounding::NearestTiesEven)?)?
            )?
            .amount(),
            a(0, 0)?
        );
    }
    Ok(())
}
#[test]
fn long_first_source_has_full_cells_and_skipped_q_does_not_reset_ai() -> Checked {
    let schedule = CanadianCouponSchedule::new(
        d!(2024, 2, 1),
        d!(2026, 1, 1),
        CanadianFrequency::Semiannual,
        vec![
            d!(2024, 1, 1),
            d!(2024, 7, 1),
            d!(2025, 1, 1),
            d!(2025, 7, 1),
            d!(2026, 1, 1),
        ],
        vec![d!(2025, 7, 1), d!(2026, 1, 1)],
        EndOfMonth::Ignore,
        5,
    )?;
    let b = CanadianBond::new(
        d!(2024, 8, 1),
        a(5, 0)?,
        schedule,
        d!(2026, 1, 3),
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?;
    let q = AmountQuantum::new(1, 12)?;
    assert_eq!(
        b.valuation_coupon_per_100(0, q, AmountRounding::NearestTiesEven)?,
        a(7074175824176, 12)?
    );
    assert_eq!(
        b.payable_coupon_per_100(0, q, AmountRounding::NearestTiesEven)?,
        a(7068493150685, 12)?
    );
    let i = b.coupon_cash(
        0,
        a(100, 0)?,
        rounding(1, 2, AmountRounding::NearestTiesEven)?,
    )?;
    assert_eq!(i.source_full_period_count(), Some(2));
    assert_eq!(i.source_day_ratios(), Some(((151, 182), None)));
    let p = CanadianPricingPlan::new(
        &b,
        d!(2024, 8, 1),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    let ai = p.accrued_cash(a(100, 0)?, rounding(1, 2, AmountRounding::NearestTiesEven)?)?;
    assert_eq!(ai.source_interval(), Some((d!(2024, 2, 1), d!(2024, 8, 1))));
    assert_eq!(ai.source_full_period_count(), Some(0));
    assert_eq!(ai.source_day_ratios(), Some(((151, 182), Some((31, 184)))));
    assert_eq!(
        p.settlement_accrued_at_quantum(q, AmountRounding::NearestTiesEven)?,
        a(2493150684932, 12)?
    );
    let reset = CanadianPricingPlan::new(
        &b,
        d!(2025, 7, 1),
        CanadianYieldSelection::LastCouponMoneyMarket,
        1,
    )?;
    assert_eq!(reset.bundle_coupon_index(0), Some(1));
    let empty = reset.accrued_cash(a(100, 0)?, rounding(1, 2, AmountRounding::Exact)?)?;
    assert_eq!(empty.source_payment_date(), Some(d!(2026, 1, 3)));
    assert_eq!(
        empty.source_interval(),
        Some((d!(2025, 7, 1), d!(2025, 7, 1)))
    );
    Ok(())
}
#[test]
fn direct_anchor_clamp_is_not_recursive_and_eom_identity_survives() -> Checked {
    let q = vec![
        d!(2025, 2, 28),
        d!(2025, 3, 31),
        d!(2025, 4, 30),
        d!(2025, 5, 31),
    ];
    let actual = vec![d!(2025, 3, 31), d!(2025, 4, 30), d!(2025, 5, 31)];
    let ignored = CanadianCouponSchedule::new(
        d!(2025, 2, 28),
        d!(2025, 5, 31),
        CanadianFrequency::Monthly,
        q.clone(),
        actual.clone(),
        EndOfMonth::Ignore,
        4,
    )?;
    let preserved = CanadianCouponSchedule::new(
        d!(2025, 2, 28),
        d!(2025, 5, 31),
        CanadianFrequency::Monthly,
        q,
        actual,
        EndOfMonth::Preserve,
        4,
    )?;
    assert_ne!(ignored, preserved);
    assert!(
        matches!(CanadianCouponSchedule::new(d!(2025,2,28),d!(2025,5,31),CanadianFrequency::Monthly,
      vec![d!(2025,2,28),d!(2025,3,28),d!(2025,4,28),d!(2025,5,31)],vec![d!(2025,3,31),d!(2025,4,30),d!(2025,5,31)],EndOfMonth::Ignore,4),
      Err(CanadianError::QuasiDateMismatch{index:1,supplied,expected,..}) if supplied==d!(2025,3,28) && expected==d!(2025,3,31))
    );
    Ok(())
}
#[test]
fn binary_price_evidence_and_exact_source_ai_do_not_gain_false_cash_equality() -> Checked {
    let b = semi(9125, 3)?;
    let p = CanadianPricingPlan::new(
        &b,
        d!(2025, 3, 2),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        2,
    )?;
    let r = rounding(1, 38, AmountRounding::NearestTiesEven)?;
    let dirty = p.dirty_cash(
        a(100, 0)?,
        SettlementEvidence::supplied_binary64(p.settlement_accrued_per_100())?,
        r,
    )?;
    let invoice = p.clean_accrued_cash(
        a(100, 0)?,
        SettlementEvidence::supplied_binary64(0.0)?,
        CashAggregation::RoundTotal,
        r,
    )?;
    assert_eq!(determined(&invoice)?.amount(), a(25, 3)?);
    assert_eq!(
        determined(&dirty)?.amount(),
        a(2500000000000000138777878078144567553, 38)?
    );
    assert_ne!(determined(&dirty)?.amount(), determined(&invoice)?.amount());
    assert_eq!(
        determined(&dirty)?.price_evidence_guarantee(),
        Some(SettlementGuarantee::ForSuppliedBinary64)
    );
    assert_eq!(
        determined(&invoice)?.price_evidence_guarantee(),
        Some(SettlementGuarantee::ForSuppliedBinary64)
    );
    Ok(())
}
