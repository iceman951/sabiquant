//! Self-contained actual Gilt admission, source-role, budget and point controls.
//!
//! FR-ENCL-03/04/05/08/09/10/11/13/15. Constants are existing source-contract
//! identities and the frozen tiny-coupon family, not observed native prices.

use super::*;
use crate::bonds::ConventionalGilt;
use crate::calendar::EndOfMonth;
use crate::time::Date;
use std::error::Error;

type Check<T = ()> = std::result::Result<T, Box<dyn Error>>;
const AMPLE: u64 = 1_000_000_000_000;
const MIN_WORK: u64 = 53_680;

fn budget(maximum: u64) -> Check<EnclosureBudget> {
    Ok(EnclosureBudget::new(64, maximum)?)
}

fn regular(coupon: DecimalAmount, periods: usize) -> Check<ConventionalGilt> {
    let start: Date = "2000-01-01".parse()?;
    let mut quasi = Vec::new();
    for n in 0..=periods {
        quasi.push(start.checked_add_months(6 * i64::try_from(n)?)?);
    }
    let maturity = *quasi.last().ok_or("missing maturity")?;
    let mut cutoffs = Vec::new();
    for date in &quasi[1..] {
        cutoffs.push(date.checked_sub_days(7)?);
    }
    Ok(ConventionalGilt::new(
        start,
        maturity,
        coupon,
        quasi[1],
        quasi,
        cutoffs,
        EndOfMonth::Ignore,
        ConventionalGilt::MAX_QUASI_DATES,
    )?)
}

fn prepared(gilt: &ConventionalGilt) -> Check<GiltPricingPlan<'_>> {
    Ok(GiltPricingPlan::new(
        gilt,
        gilt.issue_date(),
        GiltPricingPlan::MAX_BUNDLES,
    )?)
}

#[test]
fn global_errors_precede_frame_and_preserve_every_attempted_bit() -> Check {
    let gilt = regular(DecimalAmount::new(0, 0)?, 2)?;
    let plan = prepared(&gilt)?;
    for bits in [
        0x7ff8_0000_0000_0042,
        0xfff8_0000_0000_0019,
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
        (-2.0_f64).to_bits(),
        (-3.0_f64).to_bits(),
    ] {
        let error = plan
            .enclose_price_per_100(
                f64::from_bits(bits),
                BondPriceKind::Clean,
                budget(MIN_WORK)?,
            )
            .err()
            .ok_or("expected global rejection")?;
        let c = error.context();
        assert_eq!(c.yield_bits(), bits);
        assert_eq!(c.stage(), EnclosureStage::QuoteAdmission);
        assert_eq!(c.work_used(), 1_088);
        assert_eq!(c.admission_checked_flows(), 0);
        assert_eq!(c.reduction_completed_flows(), 0);
        assert_eq!(c.next_original_index(), None);
        assert!(!c.admission_complete());
        if f64::from_bits(bits).is_finite() {
            assert_eq!(
                error.domain_cause(),
                Some(EnclosureDomainCause::YieldNotAboveFloor {
                    payments_per_year: 2
                })
            );
        } else {
            assert_eq!(error.kind(), "enclosure.non_finite_yield");
            assert!(error.source().is_some());
        }
    }
    Ok(())
}

#[test]
fn actual_original_flow_admission_and_after_admission_stops_are_distinct() -> Check {
    let large = regular(DecimalAmount::new(0, 0)?, 3_000)?;
    let plan = prepared(&large)?;
    let error = plan
        .enclose_price_per_100(-0.0, BondPriceKind::Dirty, budget(MIN_WORK)?)
        .err()
        .ok_or("many-flow exact admission must exhaust")?;
    let c = error.context();
    assert_eq!(error.kind(), "enclosure.admission_incomplete");
    assert_eq!(c.stage(), EnclosureStage::FlowAdmission);
    assert_eq!(c.yield_bits(), (-0.0_f64).to_bits());
    assert!(!c.admission_complete());
    assert!(c.admission_checked_flows() < plan.bundle_count());
    assert_eq!(c.reduction_completed_flows(), 0);
    assert_eq!(
        c.next_original_index(),
        plan.bundle_quasi_index(c.admission_checked_flows())
    );
    assert!(c.work_used() <= MIN_WORK);

    let small = regular(DecimalAmount::new(0, 0)?, 1)?;
    let plan = prepared(&small)?;
    let EnclosureOutcome::Unresolved(report) =
        plan.enclose_price_per_100(0.03125, BondPriceKind::Dirty, budget(MIN_WORK)?)?
    else {
        return Err("arena fee must stop fully admitted minimum-budget call".into());
    };
    assert_eq!(
        report.reason(),
        EnclosureUnresolvedReason::WorkLimitAfterAdmission
    );
    assert!(report.context().admission_complete());
    assert_eq!(report.context().admission_checked_flows(), 1);
    assert_eq!(report.context().reduction_completed_flows(), 0);
    assert_eq!(report.context().stage(), EnclosureStage::Initialization);
    assert_eq!(report.context().next_original_index(), None);
    assert!(!report.complete_extended_arithmetic());
    Ok(())
}

#[test]
fn exact_zero_yield_uses_all_bundles_and_preferred_points() -> Check {
    for coupon in [DecimalAmount::new(6, 0)?, DecimalAmount::new(1, 38)?] {
        let gilt = regular(coupon, 2)?;
        let plan = prepared(&gilt)?;
        for basis in BondPriceKind::ALL {
            for p in [64, 128, 256, 512, 1024] {
                let EnclosureOutcome::Finite(report) =
                    plan.enclose_price_per_100(-0.0, basis, EnclosureBudget::new(p, AMPLE)?)?
                else {
                    return Err("mandatory whole zero-yield fold must finish".into());
                };
                assert!(std::ptr::eq(
                    report.origin().gilt_plan().ok_or("origin missing")?,
                    &plan
                ));
                assert!(report.origin().generic_plan().is_none());
                assert_eq!(report.context().yield_bits(), 0);
                assert_eq!(report.context().basis(), EnclosureBasis::Gilt(basis));
                assert_eq!(report.context().admission_checked_flows(), 2);
                assert_eq!(report.context().reduction_completed_flows(), 2);
                assert_eq!(
                    report.proof_kind(),
                    EnclosureProofKind::ExactRationalIdentity
                );
                assert!(report.settlement_evidence().is_declared_interval());
                if coupon.scale() == 0 {
                    assert_eq!(report.decimal_point(), Some(DecimalAmount::new(106, 0)?));
                    assert_eq!(report.encoding(), EnclosureEncoding::ExactDecimalPoint);
                } else {
                    // Existing tiny-coupon family:100+10^-38 fits neither a
                    // positive i128 decimal coefficient nor binary64 exactly.
                    assert_eq!(report.encoding(), EnclosureEncoding::OutwardBinary64Bounds);
                    let (lo, hi) = report
                        .settlement_evidence()
                        .binary64_bounds()
                        .ok_or("bounds missing")?;
                    assert_eq!(
                        (lo.to_bits(), hi.to_bits()),
                        (100.0_f64.to_bits(), 100.0_f64.to_bits() + 1)
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn actual_long_ex_recipe_remains_raw_and_first_role_stays_published() -> Check {
    // Existing conventional-gilt source example: Long first publication is
    // 3.510989, while the ex-period AI is raw -6/366 = -1/61.
    let maturity: Date = "2028-12-07".parse()?;
    let mut quasi = Vec::new();
    for n in 0..=60 {
        quasi.push(maturity.checked_add_months(6 * (i64::from(n) - 60))?);
    }
    let mut cutoffs = Vec::new();
    for date in &quasi[2..] {
        cutoffs.push(date.checked_sub_days(10)?);
    }
    let gilt = ConventionalGilt::new(
        "1999-05-07".parse()?,
        maturity,
        DecimalAmount::new(6, 0)?,
        "1999-12-07".parse()?,
        quasi,
        cutoffs,
        EndOfMonth::Ignore,
        61,
    )?;
    let plan = GiltPricingPlan::new(&gilt, "1999-12-06".parse()?, 60)?;
    assert_eq!(gilt.first_period(), GiltFirstPeriod::Long);
    assert_eq!(gilt.first_dividend_quasi_index(), 2);
    assert_eq!(plan.bundle_quasi_index(0), Some(3));
    let mut call = Call::new(
        0.0,
        EnclosureBasis::Gilt(BondPriceKind::Clean),
        budget(AMPLE)?,
    );
    let ai = accrued(&mut call, &plan).map_err(|_| "raw source staging failed")?;
    assert!(ai.negative);
    assert_eq!(ai.numerator.to_u128(), Some(6));
    assert_eq!(ai.denominator.to_u128(), Some(366));
    for bundle in &plan.bundles {
        // The first odd coupon is detached here; remaining actual retained
        // bundle denominators belong to the regular role2, including par.
        assert_eq!(bundle.amount.denominator.to_u128(), Some(2));
    }
    let EnclosureOutcome::Finite(report) =
        plan.enclose_price_per_100(0.0, BondPriceKind::Clean, budget(AMPLE)?)?
    else {
        return Err("whole Long clean fold must finish".into());
    };
    assert_eq!(
        report.context().reduction_completed_flows(),
        plan.bundle_count()
    );
    assert_eq!(
        report.proof_kind(),
        EnclosureProofKind::ExactRationalIdentity
    );
    Ok(())
}

#[test]
fn actual_target_completion_survives_point_and_export_work_stops() -> Check {
    let gilt = regular(DecimalAmount::new(1, 38)?, 2)?;
    let plan = prepared(&gilt)?;
    let EnclosureOutcome::Finite(full) =
        plan.enclose_price_per_100(0.0, BondPriceKind::Clean, budget(AMPLE)?)?
    else {
        return Err("required full result".into());
    };
    let total = full.context().work_used();
    assert!(total > MIN_WORK);
    let mut before = MIN_WORK;
    let mut after = total;
    while before + 1 < after {
        let middle = before + (after - before) / 2;
        let complete =
            match plan.enclose_price_per_100(0.0, BondPriceKind::Clean, budget(middle)?)? {
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
        plan.enclose_price_per_100(0.0, BondPriceKind::Clean, budget(before)?)?
    else {
        return Err("required pre-target stop".into());
    };
    assert!(!unfinished.complete_extended_arithmetic());
    for maximum in [after, after + 1, total - 1] {
        let EnclosureOutcome::Unresolved(report) =
            plan.enclose_price_per_100(-0.0, BondPriceKind::Clean, budget(maximum)?)?
        else {
            return Err("required post-target work stop".into());
        };
        assert!(report.complete_extended_arithmetic());
        assert_eq!(report.context().yield_bits(), 0);
        assert_eq!(report.context().admission_checked_flows(), 2);
        assert_eq!(report.context().reduction_completed_flows(), 2);
        assert_eq!(report.context().next_original_index(), None);
        assert!(matches!(
            report.context().stage(),
            EnclosureStage::Export | EnclosureStage::Initialization
        ));
        assert!(report.context().work_used() <= maximum);
    }
    Ok(())
}

#[test]
fn scoped_calls_keep_budgets_and_work_independent() -> Check {
    let gilt = regular(DecimalAmount::new(6, 0)?, 2)?;
    let plan = prepared(&gilt)?;
    let expected = plan.enclose_price_per_100(0.0, BondPriceKind::Dirty, budget(AMPLE)?)?;
    let EnclosureOutcome::Finite(expected) = expected else {
        return Err("expected finite result".into());
    };
    std::thread::scope(|scope| -> Check {
        let mut handles = Vec::new();
        for _ in 0..4 {
            handles.push(scope.spawn(|| {
                plan.enclose_price_per_100(
                    0.0,
                    BondPriceKind::Dirty,
                    EnclosureBudget::new(64, AMPLE).map_err(crate::error::Error::from)?,
                )
                .map_err(crate::error::Error::from)
            }));
        }
        for handle in handles {
            let output = handle.join().map_err(|_| "worker panicked")??;
            let EnclosureOutcome::Finite(report) = output else {
                return Err("independent call unresolved".into());
            };
            assert_eq!(report.context(), expected.context());
            assert_eq!(report.settlement_evidence(), expected.settlement_evidence());
        }
        Ok(())
    })
}
