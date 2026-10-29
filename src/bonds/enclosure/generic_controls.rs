//! Actual Generic selector, admission, progress and final-point controls.
//!
//! FR-ENCL-03/04/05/08/09/10/11/13/15: these call the producer and its actual
//! branch selector. HYBRID144/H and adopted exact-point laws supply expectations.

use super::*;
use crate::bonds::{Bond, BondSettlement, BondYieldConvention};
use crate::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use crate::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use crate::daycount::{ActualActualIcma, DayCount, DeterminationPeriod};
use crate::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
use crate::time::{Date, Frequency, Period, TimeUnit};
use std::collections::BTreeSet;
use std::error::Error;

#[path = "generic_hybrid_cases.rs"]
mod hybrid_cases;

type Check<T = ()> = std::result::Result<T, Box<dyn Error>>;
const MIN_WORK: u64 = 53_680;
const AMPLE: u64 = 1_000_000_000_000;

fn budget(maximum: u64) -> Check<EnclosureBudget> {
    Ok(EnclosureBudget::new(64, maximum)?)
}

fn compounding(row: &hybrid_cases::Case) -> Check<Compounding> {
    Ok(match row.method {
        "Simple" => Compounding::SIMPLE,
        "Continuous" => Compounding::CONTINUOUS,
        method => {
            let frequency: Frequency = row.quote_frequency.parse()?;
            match method {
                "Compounded" => Compounding::compounded(frequency)?,
                "SimpleThenCompounded" => Compounding::simple_then_compounded(frequency)?,
                "CompoundedThenSimple" => Compounding::compounded_then_simple(frequency)?,
                _ => return Err("unknown frozen compounding method".into()),
            }
        }
    })
}

fn zero_bond() -> Check<Bond> {
    let start: Date = "2026-01-01".parse()?;
    let end: Date = "2027-01-01".parse()?;
    Ok(Bond::zero_coupon(
        start,
        end,
        1000.0,
        SimpleCashFlow::new(end, 1001.0)?,
    )?)
}

fn scheduled_zero(end: &str, cap: u32) -> Check<Bond> {
    let start: Date = "2026-01-01".parse()?;
    let end: Date = end.parse()?;
    let schedule = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )?
    .build_recurrent(
        Period::new(6, TimeUnit::Months),
        DateGenerationRule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        cap,
    )?;
    let coupons = schedule.contractual_dates().len() - 1;
    Ok(Bond::fixed_rate(
        start,
        schedule,
        1000.0,
        FixedCouponRate::new(0.0, DayCount::Actual365Fixed)?,
        vec![None; coupons],
        SimpleCashFlow::new(end, 1001.0)?,
        cap,
    )?)
}

fn single_coupon(face: f64, coupon_amount: f64, redemption: f64) -> Check<Bond> {
    let start: Date = "2026-01-01".parse()?;
    let end: Date = "2027-01-01".parse()?;
    let schedule = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )?
    .build_zero(2)?;
    Ok(Bond::fixed_rate(
        start,
        schedule,
        face,
        FixedCouponRate::new(coupon_amount / face, DayCount::Actual365Fixed)?,
        vec![None],
        SimpleCashFlow::new(end, redemption)?,
        2,
    )?)
}

fn plan<'s>(bond: &'s Bond, convention: &'s BondYieldConvention) -> Check<BondPricingPlan<'s>> {
    Ok(BondPricingPlan::new(
        bond,
        BondSettlement::new("2026-01-01".parse()?, PaymentDateInclusion::Include),
        convention,
        u64::MAX,
    )?)
}

#[test]
fn actual_selector_consumes_all_frozen_hybrid_horizons() -> Check {
    // EP-G02/G03/G04: the actual selector's independent expected branch is the
    // frozen row, not price(y0), which equals100 under every branch.
    // Exact projection of the existing immutable FS1 rows. Unit controls must
    // also run from the published package, which intentionally omits validation/.
    assert_eq!(
        hybrid_cases::SOURCE_SHA256,
        "b74caa0280208a67858f04596decd7cb31c51b83b3b1e40ce933e65419794628"
    );
    assert_eq!(hybrid_cases::CASES.len(), 144);
    let mut configurations = BTreeSet::new();
    for row in hybrid_cases::CASES {
        let method = compounding(row)?;
        let mut periods = Vec::new();
        for &(start, end) in row.periods {
            periods.push(DeterminationPeriod::new(start.parse()?, end.parse()?)?);
        }
        let icma = ActualActualIcma::new(periods, row.accrual_frequency.parse()?)?;
        let convention = BondYieldConvention::new(icma, method);
        let payment: Date = row.payment.parse()?;
        let bond = Bond::zero_coupon(
            row.issue.parse()?,
            payment,
            row.face_literal.parse()?,
            SimpleCashFlow::new(payment, row.redemption_literal.parse()?)?,
        )?;
        let prepared = BondPricingPlan::new(
            &bond,
            BondSettlement::new(row.settlement.parse()?, PaymentDateInclusion::Exclude),
            &convention,
            u64::MAX,
        )?;
        assert_eq!(prepared.discount_flows.len(), 1);
        let horizon = prepared.discount_flows[0].horizon;
        assert_eq!(horizon.to_bits(), row.expected_horizon_bits, "{}", row.id);
        let actual = selected_branch(method, horizon);
        let expected = match row.expected_branch {
            "Simple" => ActiveBranch::Simple,
            "Continuous" => ActiveBranch::Continuous,
            "Periodic" => ActiveBranch::Periodic(row.m.ok_or("missing frozen periodic count")?),
            _ => return Err("unknown frozen active branch".into()),
        };
        assert_eq!(actual, expected, "{}", row.id);
        assert_eq!(actual, method.active_branch(horizon));
        configurations.insert(format!("{}:{}", row.method, row.quote_frequency));
    }
    assert_eq!(configurations.len(), 32);
    Ok(())
}

#[test]
fn global_errors_keep_original_bits_and_charge_only_entry_terminal() -> Check {
    // FR-ENCL-03/11/13: includes empty/time0 and selected-Simple hybrid inputs.
    let bond = zero_bond()?;
    for method in [
        Compounding::compounded(Frequency::Semiannual)?,
        Compounding::simple_then_compounded(Frequency::Semiannual)?,
        Compounding::compounded_then_simple(Frequency::Semiannual)?,
    ] {
        let convention = BondYieldConvention::new(DayCount::Actual365Fixed, method);
        for (settlement, inclusion) in [
            ("2026-01-01", PaymentDateInclusion::Include),
            ("2027-01-01", PaymentDateInclusion::Include),
            ("2027-01-02", PaymentDateInclusion::Exclude),
        ] {
            let prepared = BondPricingPlan::new(
                &bond,
                BondSettlement::new(settlement.parse()?, inclusion),
                &convention,
                u64::MAX,
            )?;
            for quote in [
                0x7ff0_0000_0000_0000,
                0xfff0_0000_0000_0000,
                0xfff8_0000_0000_00a5,
                (-2.0_f64).to_bits(),
            ] {
                let error = prepared
                    .enclose_price_per_100(
                        f64::from_bits(quote),
                        BondPriceKind::Dirty,
                        budget(MIN_WORK)?,
                    )
                    .err()
                    .ok_or("required global error")?;
                assert_eq!(error.context().work_used(), 1088);
                assert_eq!(error.context().yield_bits(), quote);
                assert_eq!(error.context().stage(), EnclosureStage::QuoteAdmission);
                assert_eq!(error.context().admission_checked_flows(), 0);
                assert_eq!(error.context().reduction_completed_flows(), 0);
                assert_eq!(error.context().next_original_index(), None);
                assert!(!error.context().admission_complete());
                if quote == (-2.0_f64).to_bits() {
                    assert_eq!(error.kind(), "enclosure.yield_not_above_floor");
                    assert_eq!(
                        error.domain_cause(),
                        Some(EnclosureDomainCause::YieldNotAboveFloor {
                            payments_per_year: 2
                        })
                    );
                } else {
                    assert_eq!(error.kind(), "enclosure.non_finite_yield");
                    assert!(
                        error
                            .source()
                            .and_then(|cause| cause.downcast_ref::<ValueError>())
                            .is_some()
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn exact_simple_rejection_counts_completed_check_even_when_last() -> Check {
    // Q0000011 and Q0003488: zero source coefficients still require admission.
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    for (bond, expected_checked, expected_index) in [
        (zero_bond()?, 1, 0),
        (scheduled_zero("2027-01-01", 3)?, 2, 1),
    ] {
        let prepared = plan(&bond, &convention)?;
        let error = prepared
            .enclose_price_per_100(-1.0, BondPriceKind::Dirty, budget(AMPLE)?)
            .err()
            .ok_or("required exact Simple rejection")?;
        assert_eq!(error.kind(), "enclosure.nonpositive_simple_denominator");
        assert_eq!(
            error.domain_cause(),
            Some(EnclosureDomainCause::NonpositiveSimpleDenominator {
                horizon_bits: 1.0_f64.to_bits()
            })
        );
        assert_eq!(error.context().stage(), EnclosureStage::FlowAdmission);
        assert_eq!(error.context().admission_checked_flows(), expected_checked);
        assert_eq!(error.context().next_original_index(), Some(expected_index));
        assert_eq!(error.context().reduction_completed_flows(), 0);
        assert!(!error.context().admission_complete());
    }
    Ok(())
}

#[test]
fn admission_stop_is_error_but_admitted_stop_is_unresolved() -> Check {
    // EP-X03/X06: actual many-flow plan, with no synthetic Work/plan mutation.
    let bond = scheduled_zero("2038-01-01", 25)?;
    let simple = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    let prepared = plan(&bond, &simple)?;
    let error = prepared
        .enclose_price_per_100(-0.0, BondPriceKind::Dirty, budget(MIN_WORK)?)
        .err()
        .ok_or("admission must exhaust this budget")?;
    let context = error.context();
    assert_eq!(error.kind(), "enclosure.admission_incomplete");
    assert_eq!(context.stage(), EnclosureStage::FlowAdmission);
    assert!(!context.admission_complete());
    assert_eq!(context.yield_bits(), (-0.0_f64).to_bits());
    assert_eq!(context.reduction_completed_flows(), 0);
    assert!(context.admission_checked_flows() < prepared.discount_flows.len());
    assert_eq!(
        context.next_original_index(),
        Some(prepared.discount_flows[context.admission_checked_flows()].index)
    );
    assert!(context.work_used() <= MIN_WORK);

    let continuous = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::CONTINUOUS);
    let prepared = plan(&bond, &continuous)?;
    let EnclosureOutcome::Unresolved(report) =
        prepared.enclose_price_per_100(-0.0, BondPriceKind::Dirty, budget(MIN_WORK)?)?
    else {
        return Err("required post-admission unresolved".into());
    };
    let context = report.context();
    assert_eq!(
        report.reason(),
        EnclosureUnresolvedReason::WorkLimitAfterAdmission
    );
    assert!(context.admission_complete());
    assert_eq!(
        context.admission_checked_flows(),
        prepared.discount_flows.len()
    );
    assert_eq!(context.yield_bits(), 0);
    assert!(!report.complete_extended_arithmetic());
    assert!(context.reduction_completed_flows() < prepared.discount_flows.len());
    if let Some(index) = context.next_original_index() {
        assert_eq!(
            index,
            prepared.discount_flows[context.reduction_completed_flows()].index
        );
    } else {
        assert_eq!(context.reduction_completed_flows(), 0);
    }
    assert!(context.work_used() <= MIN_WORK);
    Ok(())
}

#[test]
fn actual_target_completion_precedes_codec_and_export_cutoffs() -> Check {
    // EP-N08/X05: locate real current-byte cutpoints, never freeze guessed U.
    let bond = scheduled_zero("2038-01-01", 25)?;
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::CONTINUOUS);
    let prepared = plan(&bond, &convention)?;
    let EnclosureOutcome::Finite(full) =
        prepared.enclose_price_per_100(0.0, BondPriceKind::Dirty, budget(AMPLE)?)?
    else {
        return Err("required complete zero-yield target".into());
    };
    let total = full.context().work_used();
    assert!(total > MIN_WORK);
    let mut before = MIN_WORK;
    let mut after = total;
    while before + 1 < after {
        let middle = before + (after - before) / 2;
        let complete =
            match prepared.enclose_price_per_100(0.0, BondPriceKind::Dirty, budget(middle)?)? {
                EnclosureOutcome::Finite(_) => true,
                EnclosureOutcome::Unresolved(report) => report.complete_extended_arithmetic(),
            };
        if complete {
            after = middle;
        } else {
            before = middle;
        }
    }
    let EnclosureOutcome::Unresolved(unfinished) =
        prepared.enclose_price_per_100(0.0, BondPriceKind::Dirty, budget(before)?)?
    else {
        return Err("required unfinished-target stop".into());
    };
    assert!(!unfinished.complete_extended_arithmetic());
    for maximum in [after, after + 1, total - 1] {
        let EnclosureOutcome::Unresolved(report) =
            prepared.enclose_price_per_100(0.0, BondPriceKind::Dirty, budget(maximum)?)?
        else {
            return Err("required codec/export stop".into());
        };
        assert!(report.complete_extended_arithmetic());
        assert_eq!(report.context().stage(), EnclosureStage::Export);
        assert_eq!(
            report.context().admission_checked_flows(),
            prepared.discount_flows.len()
        );
        assert_eq!(
            report.context().reduction_completed_flows(),
            prepared.discount_flows.len()
        );
        assert_eq!(report.context().next_original_index(), None);
        assert!(report.context().work_used() <= maximum);
    }
    for maximum in [total, total + 1] {
        let EnclosureOutcome::Finite(report) =
            prepared.enclose_price_per_100(0.0, BondPriceKind::Dirty, budget(maximum)?)?
        else {
            return Err("exact completion budget must suffice".into());
        };
        assert_eq!(report.context().work_used(), total);
        assert_eq!(report.encoding(), EnclosureEncoding::ExactDecimalPoint);
    }
    Ok(())
}

#[test]
fn frozen_hand_points_prefer_decimal_and_preserve_signed_targets() -> Check {
    // Existing H G0=100.1 and GNEG=-100.5; no new expected price table.
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    for (bond, expected) in [
        (zero_bond()?, "100.1"),
        (single_coupon(1000.0, -2000.0, 995.0)?, "-100.5"),
    ] {
        let prepared = plan(&bond, &convention)?;
        for precision in [64, 128, 256, 512, 1024] {
            let EnclosureOutcome::Finite(report) = prepared.enclose_price_per_100(
                -0.0,
                BondPriceKind::Dirty,
                EnclosureBudget::new(precision, AMPLE)?,
            )?
            else {
                return Err("required exact hand point".into());
            };
            assert_eq!(report.decimal_point(), Some(expected.parse()?));
            assert_eq!(report.encoding(), EnclosureEncoding::ExactDecimalPoint);
            assert_eq!(
                report.proof_kind(),
                EnclosureProofKind::ExactRationalIdentity
            );
            assert_eq!(report.context().yield_bits(), 0);
            assert!(std::ptr::eq(
                report.origin().generic_plan().ok_or("missing origin")?,
                &prepared
            ));
        }
    }
    Ok(())
}

#[test]
fn actual_nonzero_final_singletons_obey_adopted_fit_laws() -> Check {
    // Adopted contract laws: exact half discount, same-date amount sum. Assert
    // actual retained bits before applying these exact mathematical identities.
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    let power101 = f64::from_bits(u64::from(1023_u32 + 101) << 52);
    for (coupon, redemption, decimal) in [
        (power101, 2.0, true),
        (2.0, f64::from_bits(0x3cb0_0000_0000_0000), false),
    ] {
        let bond = single_coupon(100.0, coupon, redemption)?;
        let prepared = plan(&bond, &convention)?;
        assert_eq!(prepared.discount_flows.len(), 2);
        assert_eq!(
            prepared.discount_flows[0].amount.to_bits(),
            coupon.to_bits()
        );
        assert_eq!(
            prepared.discount_flows[1].amount.to_bits(),
            redemption.to_bits()
        );
        assert!(
            prepared
                .discount_flows
                .iter()
                .all(|flow| flow.horizon.to_bits() == 1.0_f64.to_bits())
        );
        let EnclosureOutcome::Finite(report) = prepared.enclose_price_per_100(
            1.0,
            BondPriceKind::Dirty,
            EnclosureBudget::new(128, AMPLE)?,
        )?
        else {
            return Err("required controlled complete singleton".into());
        };
        assert_eq!(report.proof_kind(), EnclosureProofKind::IntervalCollapse);
        if decimal {
            assert_eq!(report.encoding(), EnclosureEncoding::ExactDecimalPoint);
            assert_eq!(
                report.decimal_point(),
                Some(DecimalAmount::new((1_i128 << 100) + 1, 0)?)
            );
        } else {
            assert_eq!(report.encoding(), EnclosureEncoding::OutwardBinary64Bounds);
            let (lower, upper) = report
                .settlement_evidence()
                .binary64_bounds()
                .ok_or("missing strict binary bounds")?;
            assert_eq!(lower.to_bits(), 1.0_f64.to_bits());
            assert_eq!(upper.to_bits(), 1.0_f64.to_bits() + 1);
        }
    }
    Ok(())
}

#[test]
fn actual_binary_point_outside_decimal_coefficient_range_stays_exact() -> Check {
    // Existing exact-point law2^127: exceeds positive i128, fits binary64.
    let date: Date = "2027-01-01".parse()?;
    let power127 = f64::from_bits(u64::from(1023_u32 + 127) << 52);
    let bond = Bond::zero_coupon(
        "2026-01-01".parse()?,
        date,
        100.0,
        SimpleCashFlow::new(date, power127)?,
    )?;
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::CONTINUOUS);
    let prepared = plan(&bond, &convention)?;
    let EnclosureOutcome::Finite(report) =
        prepared.enclose_price_per_100(0.0, BondPriceKind::Dirty, budget(AMPLE)?)?
    else {
        return Err("required exact binary point".into());
    };
    assert_eq!(report.encoding(), EnclosureEncoding::ExactBinary64Point);
    assert_eq!(
        report.proof_kind(),
        EnclosureProofKind::ExactRationalIdentity
    );
    let (lower, upper) = report
        .settlement_evidence()
        .binary64_bounds()
        .ok_or("missing binary point")?;
    assert_eq!(
        (lower.to_bits(), upper.to_bits()),
        (power127.to_bits(), power127.to_bits())
    );
    Ok(())
}

#[test]
fn scoped_threads_keep_actual_budgets_and_work_independent() -> Check {
    // EP-A05: one immutable plan, independent actual low/high calls and repeat.
    let bond = scheduled_zero("2038-01-01", 25)?;
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::CONTINUOUS);
    let prepared = plan(&bond, &convention)?;
    let low = budget(MIN_WORK)?;
    let high = budget(AMPLE)?;
    let (first, second) = std::thread::scope(|scope| -> Check<_> {
        let a = scope.spawn(|| prepared.enclose_price_per_100(-0.0, BondPriceKind::Dirty, low));
        let b = scope.spawn(|| prepared.enclose_price_per_100(-0.0, BondPriceKind::Dirty, high));
        Ok((
            a.join().map_err(|_| "low worker panicked")??,
            b.join().map_err(|_| "high worker panicked")??,
        ))
    })?;
    let EnclosureOutcome::Unresolved(first) = first else {
        return Err("low budget must stop".into());
    };
    let EnclosureOutcome::Finite(second) = second else {
        return Err("high budget must finish".into());
    };
    assert_eq!(first.context().budget(), low);
    assert_eq!(second.context().budget(), high);
    let EnclosureOutcome::Unresolved(repeated) =
        prepared.enclose_price_per_100(-0.0, BondPriceKind::Dirty, low)?
    else {
        return Err("repeat must preserve work stop".into());
    };
    assert_eq!(first.context(), repeated.context());
    assert_eq!(first.reason(), repeated.reason());
    assert_eq!(
        first.complete_extended_arithmetic(),
        repeated.complete_extended_arithmetic()
    );
    Ok(())
}
