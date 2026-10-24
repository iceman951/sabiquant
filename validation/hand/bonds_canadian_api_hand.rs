//! Blind Canadian public API/metadata/trait hands, FR-CAD-10/18/21/52/70/71.
//!
//! Authored from the adopted complete contract before Canadian production.
//! This is a future public test body: rustfmt parsing is not typechecking/runtime.
//! Prospective144 public types/237 domain variants are NOT implemented counts.

use sabiquant::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    CanadianBond, CanadianCashDecision, CanadianCashInstruction, CanadianCashReceipt,
    CanadianCouponSchedule, CanadianError, CanadianFrequency, CanadianPeriodKind, CanadianPrice,
    CanadianPriceBasis, CanadianPricingPlan, CanadianRisk, CanadianSourcePolicy, CanadianValuation,
    CanadianYieldConvention, CanadianYieldError, CanadianYieldReport, CanadianYieldSelection,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::Error;
use sabiquant::math::{RootBracket, SolverOptions};
use sabiquant::settlement::{SettlementEvidence, SettlementRounding};
use sabiquant::time::Date;
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error as StdError;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

type Checked<T = ()> = Result<T, Box<dyn StdError>>;
const VERSION: &str = "CA-IIAC13-fixed-2026-10-23-local-v1";

fn common<T: Debug + Clone + PartialEq + Send + Sync>() {}
fn copy<T: Copy>() {}
fn eq_hash<T: Eq + Hash>() {}
fn error_traits<T: StdError + Clone + PartialEq + Send + Sync + 'static>() {}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

fn clone_debug<T: Debug + Clone + PartialEq>(value: &T) {
    assert_eq!(&value.clone(), value);
    assert!(!format!("{value:?}").is_empty());
}

fn equal_hash<T: Debug + Eq + Hash>(left: &T, right: &T) {
    assert_eq!(left, right);
    assert_eq!(hash_of(left), hash_of(right));
}

#[expect(
    clippy::expect_used,
    reason = "A frozen hand must reject a missing typed source."
)]
fn immediate<'a, E: StdError + 'static>(outer: &'a dyn StdError) -> &'a E {
    outer
        .source()
        .expect("immediate source required")
        .downcast_ref::<E>()
        .expect("exact immediate source type")
}

fn same_source<E: StdError + 'static>(outer: &dyn StdError, stored: &E) {
    assert!(std::ptr::eq(immediate::<E>(outer), stored));
}

#[expect(
    clippy::expect_used,
    reason = "The test explicitly invokes invalid public inputs."
)]
fn invalid<T: Debug, E>(value: Result<T, E>) -> E {
    value.expect_err("explicit invalid input must fail")
}

#[expect(
    clippy::panic,
    reason = "A hand must fail when a structured error branch changes."
)]
fn wrong(error: impl Debug) -> ! {
    panic!("wrong structured branch: {error:?}")
}

fn day(y: i32, m: u32, d: u32) -> Checked<Date> {
    Ok(Date::from_ymd(y, m, d)?)
}

fn instrument() -> Checked<CanadianBond> {
    let start = day(2025, 3, 1)?;
    let maturity = day(2025, 9, 1)?;
    let schedule = CanadianCouponSchedule::new(
        start,
        maturity,
        CanadianFrequency::Semiannual,
        vec![start, maturity],
        vec![maturity],
        EndOfMonth::Ignore,
        2,
    )?;
    Ok(CanadianBond::new(
        start,
        DecimalAmount::new(0, 0)?,
        schedule,
        day(2025, 9, 2)?,
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?)
}

fn prepared<'a>(
    bond: &'a CanadianBond,
    selection: CanadianYieldSelection,
) -> Checked<CanadianPricingPlan<'a>> {
    Ok(CanadianPricingPlan::new(
        bond,
        day(2025, 3, 2)?,
        selection,
        1,
    )?)
}

// This signature requires the original bond lifetime, not the shorter &plan lifetime.
fn original_bond<'a>(plan: &CanadianPricingPlan<'a>) -> &'a CanadianBond {
    plan.bond()
}

#[test]
fn complete_eighteen_type_positive_trait_matrix() {
    macro_rules! all {
        ($($ty:ty),+ $(,)?) => { $(common::<$ty>();)+ };
    }
    all!(
        CanadianBond,
        CanadianCouponSchedule,
        CanadianFrequency,
        CanadianPeriodKind,
        CanadianSourcePolicy,
        CanadianYieldConvention,
        CanadianYieldSelection,
        CanadianPriceBasis,
        CanadianPrice,
        CanadianPricingPlan<'static>,
        CanadianValuation,
        CanadianRisk,
        CanadianYieldReport,
        CanadianError,
        CanadianYieldError,
        CanadianCashInstruction,
        CanadianCashDecision,
        CanadianCashReceipt,
    );
    macro_rules! copies {
        ($($ty:ty),+ $(,)?) => { $(copy::<$ty>();)+ };
    }
    copies!(
        CanadianFrequency,
        CanadianPeriodKind,
        CanadianSourcePolicy,
        CanadianYieldConvention,
        CanadianYieldSelection,
        CanadianPriceBasis,
        CanadianPrice,
        CanadianValuation,
        CanadianYieldReport,
        CanadianCashInstruction,
        CanadianCashDecision,
        CanadianCashReceipt,
    );
    macro_rules! hashed {
        ($($ty:ty),+ $(,)?) => { $(eq_hash::<$ty>();)+ };
    }
    hashed!(
        CanadianBond,
        CanadianCouponSchedule,
        CanadianFrequency,
        CanadianPeriodKind,
        CanadianSourcePolicy,
        CanadianYieldConvention,
        CanadianYieldSelection,
        CanadianPriceBasis,
        CanadianPrice,
        CanadianCashInstruction,
        CanadianCashDecision,
        CanadianCashReceipt,
    );
    error_traits::<CanadianError>();
    error_traits::<CanadianYieldError>();
}

#[test]
fn exact_metadata_all_arrays_and_resource_constants() {
    assert_eq!(
        CanadianFrequency::ALL,
        [
            CanadianFrequency::Semiannual,
            CanadianFrequency::Quarterly,
            CanadianFrequency::Monthly,
        ]
    );
    for (frequency, payments, months) in [
        (CanadianFrequency::Semiannual, 2_u32, 6_u32),
        (CanadianFrequency::Quarterly, 4, 3),
        (CanadianFrequency::Monthly, 12, 1),
    ] {
        assert_eq!(frequency.payments_per_year(), payments);
        assert_eq!(frequency.months(), months);
        clone_debug(&frequency);
    }
    assert_eq!(
        CanadianPeriodKind::ALL,
        [
            CanadianPeriodKind::Regular,
            CanadianPeriodKind::Short,
            CanadianPeriodKind::Long,
            CanadianPeriodKind::Composite,
        ]
    );
    assert_eq!(
        CanadianSourcePolicy::ALL,
        [CanadianSourcePolicy::CanonicalIiac13LocalV1]
    );
    assert_eq!(
        CanadianYieldConvention::ALL,
        [
            CanadianYieldConvention::Compound,
            CanadianYieldConvention::MoneyMarket,
        ]
    );
    assert_eq!(
        CanadianPriceBasis::ALL,
        [
            CanadianPriceBasis::Clean,
            CanadianPriceBasis::ModelDirty,
            CanadianPriceBasis::SettlementDirty,
        ]
    );
    // Selection has no specified ALL. Do not invent a metadata parser/default.
    assert_ne!(
        CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket),
        CanadianYieldSelection::LastCouponMoneyMarket
    );
    clone_debug(&CanadianYieldSelection::LastCouponMoneyMarket);
    assert_eq!(CanadianCouponSchedule::MAX_QUASI_DATES, 20_000);
    assert_eq!(CanadianCouponSchedule::MAX_COUPONS, 20_000);
    assert_eq!(CanadianPricingPlan::MAX_BUNDLES, 20_000);
}

#[test]
fn finite_price_identity_hash_and_basis_are_canonical() -> Checked {
    let mut zeroes = HashSet::new();
    let patterns = [
        0,
        1_u64 << 63,
        1,
        (1_u64 << 63) | 1,
        0x3ff0000000000000,
        0xbff0000000000000,
        0x7fefffffffffffff,
        0xffefffffffffffff,
    ];
    for basis in CanadianPriceBasis::ALL {
        let positive = CanadianPrice::new(0.0, basis)?;
        let negative = CanadianPrice::new(-0.0, basis)?;
        equal_hash(&positive, &negative);
        assert_eq!(positive.value().to_bits(), 0);
        assert_eq!(negative.value().to_bits(), 0);
        zeroes.insert(positive);
        zeroes.insert(negative);
        for bits in patterns {
            let price = CanadianPrice::new(f64::from_bits(bits), basis)?;
            assert_eq!(price.basis(), basis);
            assert_eq!(
                price.value().to_bits(),
                if bits == 1_u64 << 63 { 0 } else { bits }
            );
            clone_debug(&price);
            equal_hash(&price, &price);
        }
    }
    assert_eq!(zeroes.len(), 3);
    for left in CanadianPriceBasis::ALL {
        for right in CanadianPriceBasis::ALL {
            assert_eq!(
                CanadianPrice::new(1.0, left)? == CanadianPrice::new(1.0, right)?,
                left == right
            );
        }
    }
    // Hash collisions for unequal values are legal; no distinct-hash assertion.
    Ok(())
}

macro_rules! snapshot {
    ($value:expr, $selection:expr, $convention:expr, $plan:expr) => {{
        let value = &$value;
        assert_eq!(value.source_version(), VERSION);
        assert_eq!(
            value.source_policy(),
            CanadianSourcePolicy::CanonicalIiac13LocalV1
        );
        assert_eq!(value.accrual_start(), day(2025, 3, 1)?);
        assert_eq!(value.original_issue_date(), day(2025, 3, 1)?);
        assert_eq!(value.stated_maturity_date(), day(2025, 9, 1)?);
        assert_eq!(value.true_maturity_date(), day(2025, 9, 2)?);
        assert_eq!(value.first_coupon_date(), day(2025, 9, 1)?);
        assert_eq!(value.last_coupon_start_date(), day(2025, 3, 1)?);
        assert_eq!(value.first_period(), CanadianPeriodKind::Regular);
        assert_eq!(value.last_period(), CanadianPeriodKind::Regular);
        assert_eq!(value.frequency(), CanadianFrequency::Semiannual);
        assert_eq!(value.coupon_per_100(), DecimalAmount::new(0, 0)?);
        assert_eq!(value.end_of_month(), EndOfMonth::Ignore);
        assert_eq!(value.direct_cycle_anchor(), day(2025, 9, 1)?);
        assert_eq!(value.settlement_date(), day(2025, 3, 2)?);
        assert_eq!(value.yield_selection(), $selection);
        assert_eq!(value.yield_convention(), $convention);
        assert_eq!(value.yield_rate().to_bits(), 0);
        assert_eq!(
            value.valuation_accrued_per_100().to_bits(),
            $plan.valuation_accrued_per_100().to_bits()
        );
        assert_eq!(
            value.settlement_accrued_per_100().to_bits(),
            $plan.settlement_accrued_per_100().to_bits()
        );
        assert_eq!(
            value.quote_accrued_per_100().to_bits(),
            $plan.quote_accrued_per_100().to_bits()
        );
    }};
}

#[test]
fn complete_snapshot_metadata_and_original_bond_lifetime() -> Checked {
    let bond = instrument()?;
    assert_eq!(bond.source_version(), VERSION);
    clone_debug(&bond);
    clone_debug(bond.schedule());
    equal_hash(&bond, &bond.clone());
    for (selection, convention) in [
        (
            CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
            CanadianYieldConvention::Compound,
        ),
        (
            CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket),
            CanadianYieldConvention::MoneyMarket,
        ),
        (
            CanadianYieldSelection::LastCouponMoneyMarket,
            CanadianYieldConvention::MoneyMarket,
        ),
    ] {
        let plan = prepared(&bond, selection)?;
        assert!(std::ptr::eq(original_bond(&plan), &bond));
        assert_eq!(plan.source_version(), VERSION);
        assert_eq!(plan.yield_selection(), selection);
        assert_eq!(plan.yield_convention(), convention);
        clone_debug(&plan);
        let value = plan.evaluate(0.0)?;
        let risk = plan.risk(0.0)?;
        // Construct a report from the cached lower observation, not new ideal math.
        let report = plan.yield_from_price(
            value.model_dirty_price(),
            RootBracket::new(0.0, 0.25)?,
            SolverOptions::new(1e-12, 0.0, 1)?,
        )?;
        snapshot!(value, selection, convention, plan);
        snapshot!(risk, selection, convention, plan);
        snapshot!(report, selection, convention, plan);
        clone_debug(&value);
        clone_debug(&risk);
        clone_debug(&report);
        assert_eq!(
            report.model_dirty_per_100().to_bits(),
            value.model_dirty_per_100().to_bits()
        );
    }
    Ok(())
}

#[test]
fn source_schedule_identity_ignores_caps_and_caller_spare_capacity() -> Checked {
    let first = day(2025, 3, 1)?;
    let last = day(2025, 9, 1)?;
    let normal = CanadianCouponSchedule::new(
        first,
        last,
        CanadianFrequency::Semiannual,
        vec![first, last],
        vec![last],
        EndOfMonth::Ignore,
        2,
    )?;
    let mut q = Vec::with_capacity(64);
    q.extend([first, last]);
    let mut coupons = Vec::with_capacity(64);
    coupons.push(last);
    let roomy = CanadianCouponSchedule::new(
        first,
        last,
        CanadianFrequency::Semiannual,
        q,
        coupons,
        EndOfMonth::Ignore,
        CanadianCouponSchedule::MAX_QUASI_DATES,
    )?;
    equal_hash(&normal, &roomy);
    assert_eq!(roomy.quasi_dates(), &[first, last]);
    assert_eq!(roomy.coupon_dates(), &[last]);
    assert_eq!(roomy.coupon_start_date(0), Some(first));
    assert_eq!(roomy.coupon_start_date(1), None);
    assert_eq!(
        roomy.coupon_period_kind(0),
        Some(CanadianPeriodKind::Regular)
    );
    assert_eq!(roomy.coupon_period_kind(1), None);
    Ok(())
}

#[test]
fn risk_and_cash_snapshots_survive_original_owners() -> Checked {
    let (risk, instruction) = {
        let bond = instrument()?;
        let plan = prepared(
            &bond,
            CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        )?;
        (
            plan.risk(0.0)?,
            plan.dirty_cash(
                DecimalAmount::new(1, 0)?,
                SettlementEvidence::exact_decimal(DecimalAmount::new(1, 0)?),
                SettlementRounding::new(AmountQuantum::new(1, 2)?, AmountRounding::Exact),
            )?,
        )
    };
    clone_debug(&risk);
    clone_debug(&instruction);
    assert_eq!(risk.source_version(), VERSION);
    assert_eq!(instruction.source_version(), VERSION);
    assert_eq!(instruction.last_coupon_start_date(), day(2025, 3, 1)?);
    let decision = instruction.evaluate()?;
    clone_debug(&decision);
    equal_hash(&instruction, &instruction);
    equal_hash(&decision, &decision);
    match decision {
        CanadianCashDecision::Determined(receipt) => {
            clone_debug(&receipt);
            equal_hash(&receipt, &receipt);
            assert_eq!(receipt.instruction(), &instruction);
            assert_eq!(receipt.instruction().source_version(), VERSION);
        }
        other => wrong(other),
    }
    Ok(())
}

#[test]
fn real_public_error_paths_preserve_each_immediate_typed_source() -> Checked {
    let direct = invalid(CanadianPrice::new(f64::NAN, CanadianPriceBasis::Clean));
    clone_debug(&direct);
    let kind = direct.kind();
    let display = direct.to_string();
    let wrapped = Error::from(direct);
    assert_eq!(wrapped.kind(), kind);
    assert_eq!(wrapped.to_string(), display);
    let held = immediate::<CanadianError>(&wrapped);
    match held {
        CanadianError::Value { cause, .. } => same_source(held, cause),
        other => wrong(other),
    }
    let first = day(2025, 3, 1)?;
    let last = day(2025, 9, 1)?;
    let cap = invalid(CanadianCouponSchedule::new(
        first,
        last,
        CanadianFrequency::Semiannual,
        vec![first, last],
        vec![last],
        EndOfMonth::Ignore,
        1,
    ));
    assert!(matches!(cap, CanadianError::InvalidDateCap { cap: 1, .. }));
    assert!(cap.source().is_none());
    let date_error = invalid(CanadianCouponSchedule::new(
        day(1, 1, 1)?,
        day(1, 5, 1)?,
        CanadianFrequency::Semiannual,
        vec![day(1, 1, 1)?, day(1, 5, 1)?],
        vec![day(1, 5, 1)?],
        EndOfMonth::Ignore,
        2,
    ));
    match &date_error {
        CanadianError::Date {
            index: Some(0),
            phase: "quasi cycle",
            cause,
            ..
        } => same_source(&date_error, cause),
        other => wrong(other),
    }
    let bond = instrument()?;
    let plan = prepared(
        &bond,
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
    )?;
    let instruction = plan.dirty_cash(
        DecimalAmount::new(1, 0)?,
        SettlementEvidence::exact_decimal(DecimalAmount::new(1, 0)?),
        SettlementRounding::new(AmountQuantum::new(1, 0)?, AmountRounding::Exact),
    )?;
    let cash = invalid(instruction.evaluate());
    match &cash {
        CanadianError::CashArithmetic {
            stage: "cash total",
            endpoint: "point",
            cause,
            ..
        } => same_source(&cash, cause),
        other => wrong(other),
    }
    let target = plan.evaluate(0.0)?.model_dirty_price();
    let callback = invalid(plan.yield_from_price(
        target,
        RootBracket::new(-2.0, 0.0)?,
        SolverOptions::new(1e-12, 0.0, 1)?,
    ));
    clone_debug(&callback);
    let kind = callback.kind();
    let display = callback.to_string();
    let wrapped = Error::from(callback);
    assert_eq!(wrapped.kind(), kind);
    assert_eq!(wrapped.to_string(), display);
    let held = immediate::<CanadianYieldError>(&wrapped);
    match held {
        CanadianYieldError::Callback { cause, .. } => {
            same_source(held, cause);
            match cause {
                CanadianError::Value { cause: value, .. } => same_source(cause, value),
                other => wrong(other),
            }
        }
        other => wrong(other),
    }
    let nonpositive = invalid(plan.yield_from_price(
        CanadianPrice::new(0.0, CanadianPriceBasis::ModelDirty)?,
        RootBracket::new(0.0, 0.25)?,
        SolverOptions::new(1e-12, 0.0, 1)?,
    ));
    assert!(matches!(
        nonpositive,
        CanadianYieldError::NonPositiveTarget { .. }
    ));
    assert!(nonpositive.source().is_none());
    let mm = prepared(
        &bond,
        CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket),
    )?;
    let solver = invalid(mm.yield_from_price(
        CanadianPrice::new(f64::MAX, CanadianPriceBasis::ModelDirty)?,
        RootBracket::new(0.0, 0.25)?,
        SolverOptions::new(1e-12, 0.0, 2)?,
    ));
    match &solver {
        CanadianYieldError::Solver(cause) => same_source(&solver, cause),
        other => wrong(other),
    }
    Ok(())
}
