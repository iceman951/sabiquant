//! Self-contained actual Treasury admission, source-role, budget and point controls.
//!
//! FR-ENCL-03/04/05/08/09/10/11/13/15. Constants are existing source-contract
//! identities and the frozen tiny-coupon family, not observed native prices.

use super::*;
use crate::bonds::{TreasuryDailyBasis, TreasurySourcePolicy, TreasuryTieRule, UsTreasury};
use crate::calendar::EndOfMonth;
use crate::time::Date;
use std::error::Error;

type Check<T = ()> = std::result::Result<T, Box<dyn Error>>;
const AMPLE: u64 = 1_000_000_000_000;
const MIN_WORK: u64 = 53_680;

fn budget(maximum: u64) -> Check<EnclosureBudget> {
    Ok(EnclosureBudget::new(64, maximum)?)
}

fn regular(coupon: DecimalAmount, periods: usize) -> Check<UsTreasury> {
    let start: Date = "2000-01-01".parse()?;
    let mut quasi = Vec::new();
    for n in 0..=periods {
        quasi.push(start.checked_add_months(6 * i64::try_from(n)?)?);
    }
    let maturity = *quasi.last().ok_or("missing maturity")?;
    Ok(UsTreasury::new(
        start,
        start,
        maturity,
        coupon,
        quasi[1],
        quasi,
        EndOfMonth::Ignore,
        TreasurySourcePolicy::new(
            TreasuryDailyBasis::ExactRatio,
            TreasuryTieRule::NearestTiesEven,
        ),
        UsTreasury::MAX_QUASI_DATES,
    )?)
}

fn prepared(treasury: &UsTreasury) -> Check<TreasuryPricingPlan<'_>> {
    Ok(TreasuryPricingPlan::new(
        treasury,
        treasury.original_issue_date(),
        TreasuryPricingPlan::MAX_BUNDLES,
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
                    report.origin().treasury_plan().ok_or("origin missing")?,
                    &plan
                ));
                assert!(report.origin().generic_plan().is_none());
                assert_eq!(report.context().yield_bits(), 0);
                assert_eq!(report.context().basis(), EnclosureBasis::Treasury(basis));
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
fn selected_publication_controls_clean_without_changing_raw_dirty_target() -> Check {
    // Existing source tie example: rawAI=5e-7; published per100 count is0/1.
    let issue: Date = "2025-07-15".parse()?;
    let maturity: Date = "2026-01-15".parse()?;
    for daily in TreasuryDailyBasis::ALL {
        for tie in TreasuryTieRule::ALL {
            let treasury = UsTreasury::new(
                issue,
                issue,
                maturity,
                DecimalAmount::new(23, 7)?,
                maturity,
                vec![issue, maturity],
                EndOfMonth::Ignore,
                TreasurySourcePolicy::new(daily, tie),
                2,
            )?;
            let plan = TreasuryPricingPlan::new(&treasury, "2025-10-03".parse()?, 1)?;
            let expected_count = u128::from(tie == TreasuryTieRule::NearestTiesAwayFromZero);
            assert_eq!(
                plan.accrued_source.published_count.to_u128(),
                Some(expected_count)
            );
            for p in [64, 128, 256, 512, 1024] {
                for basis in BondPriceKind::ALL {
                    let EnclosureOutcome::Finite(report) =
                        plan.enclose_price_per_100(0.0, basis, EnclosureBudget::new(p, AMPLE)?)?
                    else {
                        return Err("required exact published-AI result".into());
                    };
                    let coefficient = 10_000_000_115_i128
                        - if basis == BondPriceKind::Clean && expected_count == 1 {
                            100
                        } else {
                            0
                        };
                    assert_eq!(
                        report.decimal_point(),
                        Some(DecimalAmount::new(coefficient, 8)?)
                    );
                    assert_eq!(
                        report.proof_kind(),
                        EnclosureProofKind::ExactRationalIdentity
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn long_first_retains_original_j_and_exact_first_role() -> Check {
    // Existing long-first source example: A0=4.5 at original j1, A1=103 at j2.
    let dated: Date = "2025-03-08".parse()?;
    let treasury = UsTreasury::new(
        dated,
        dated,
        "2026-06-07".parse()?,
        DecimalAmount::new(6, 0)?,
        "2025-12-07".parse()?,
        vec![
            "2024-12-07".parse()?,
            "2025-06-07".parse()?,
            "2025-12-07".parse()?,
            "2026-06-07".parse()?,
        ],
        EndOfMonth::Ignore,
        TreasurySourcePolicy::new(
            TreasuryDailyBasis::ExactRatio,
            TreasuryTieRule::NearestTiesEven,
        ),
        4,
    )?;
    let plan = TreasuryPricingPlan::new(&treasury, dated, 2)?;
    assert_eq!(treasury.first_period(), TreasuryFirstPeriod::Long);
    assert_eq!(
        (plan.next_quasi_index(), treasury.first_coupon_quasi_index()),
        (1, 2)
    );
    assert_eq!(
        plan.bundles
            .iter()
            .map(|b| (b.index, b.j))
            .collect::<Vec<_>>(),
        [(2, 1), (3, 2)]
    );
    assert_eq!(plan.bundles[0].amount.denominator.to_u128(), Some(364));
    assert_eq!(plan.bundles[1].amount.denominator.to_u128(), Some(2));
    assert_eq!(bundle_role(&plan, 2, true), 1);
    assert_eq!(bundle_role(&plan, 3, true), 0);
    for basis in BondPriceKind::ALL {
        let EnclosureOutcome::Finite(report) =
            plan.enclose_price_per_100(0.0, basis, budget(AMPLE)?)?
        else {
            return Err("required exact long-first target".into());
        };
        assert_eq!(report.decimal_point(), Some(DecimalAmount::new(1075, 1)?));
        assert_eq!(report.context().reduction_completed_flows(), 2);
    }
    Ok(())
}

#[test]
fn exact_q_admission_does_not_use_a_native_affine_overflow_gate() -> Check {
    let treasury = regular(DecimalAmount::new(0, 0)?, 1)?;
    let plan = prepared(&treasury)?;
    assert_eq!(plan.bundles[0].j, 0);
    // This is only a trigger check, never a price or containment premise.
    assert!((f64::MAX * f64::from(plan.remaining_days())).is_infinite());
    let EnclosureOutcome::Finite(report) =
        plan.enclose_price_per_100(f64::MAX, BondPriceKind::Dirty, budget(AMPLE)?)?
    else {
        return Err("j0 exact-q MAX case must stay finite".into());
    };
    let (lo, hi) = report
        .settlement_evidence()
        .binary64_bounds()
        .ok_or("bounds missing")?;
    assert!(lo > 0.0 && hi.is_finite());
    assert!(report.context().admission_complete());
    assert_eq!(report.context().reduction_completed_flows(), 1);
    Ok(())
}

#[test]
fn actual_target_completion_survives_point_and_export_work_stops() -> Check {
    // Twelve mandatory source fees alone exceed the minimum residual budget,
    // so the left search endpoint genuinely precedes the complete target.
    let gilt = regular(DecimalAmount::new(1, 38)?, 12)?;
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
        assert_eq!(
            report.context().admission_checked_flows(),
            plan.bundle_count()
        );
        assert_eq!(
            report.context().reduction_completed_flows(),
            plan.bundle_count()
        );
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
