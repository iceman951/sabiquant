//! Actual Generic financial enclosure consumers (FR-ENCL-01..11/16..18).
//!
//! Consumes the frozen Generic partitions and independent hands/boundaries.
//! No observed valuation or native elementary result is used as an oracle.

#[path = "support/enclosure.rs"]
mod enclosure;
mod support;

use enclosure::{
    Flat, GenericSource, ORDINARY_WORK, PRECISIONS, Reference, TestResult, bits, check_context,
    check_origin, compounding, finite_complete, frequency, price_kind,
};
use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    Bond, BondPriceKind, BondPricingPlan, BondSettlement, BondYieldConvention, EnclosureBudget,
    EnclosureDomainCause, EnclosureOutcome, EnclosureStage, EnclosureUnresolvedReason,
};
use sabiquant::cashflows::{PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{ActualActualIcma, DeterminationPeriod};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

fn fixture(name: &str) -> TestResult<Fixture> {
    Ok(Fixture::load(Fixture::repository_path(&format!(
        "validation/fixtures/bonds/enclosure-{name}.fixture"
    )))?)
}
fn source_index(id: &str) -> TestResult<usize> {
    let suffix = id.strip_prefix("ENC-SOURCE-").ok_or("source ID prefix")?;
    let n: usize = suffix.parse()?;
    n.checked_sub(1).ok_or_else(|| "zero source ID".into())
}
#[expect(
    clippy::print_stderr,
    reason = "actual financial consumer checkpoints, not library I/O"
)]
fn checkpoint(family: &str, count: usize) {
    eprintln!("Generic enclosure {family}: {count} checked");
}

fn load_sources() -> TestResult<Vec<GenericSource>> {
    let table = fixture("sources")?;
    let mut result = Vec::new();
    let mut configurations = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "Generic" {
            continue;
        }
        assert_eq!(source_index(row.id())?, result.len());
        let s =
            GenericSource::from_source(row.id(), row.text("input")?, row.text("source_summary")?)?;
        configurations.insert(s.configuration.clone());
        result.push(s);
    }
    assert_eq!((result.len(), configurations.len()), (8960, 32));
    Ok(result)
}

fn ordinary(sources: &[GenericSource]) -> TestResult {
    let table = fixture("prices-v2")?;
    let mut plan = None;
    let mut previous = None;
    let mut count = 0;
    let mut used = vec![false; sources.len()];
    let mut sampled = BTreeSet::new();
    let mut first_nonzero = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "Generic" {
            continue;
        }
        let index = source_index(row.text("source_id")?)?;
        let source = sources.get(index).ok_or("Generic source missing")?;
        assert_eq!(source.id, row.text("source_id")?);
        if previous != Some(index) {
            plan = Some(source.plan()?);
            previous = Some(index);
        }
        let plan = plan.as_ref().ok_or("prepared plan missing")?;
        let kind = price_kind(row.text("basis")?)?;
        let quote = bits(row.text("yield_bits")?)?;
        let reference = Reference::price(
            row.text("outward_lower_bits")?,
            row.text("outward_upper_bits")?,
            row.text("point_encoding")?,
            row.text("point_value")?,
        )?;
        let mut precisions = vec![64];
        if quote == 0x3fa0_0000_0000_0000
            && sampled.insert(format!("{}:{}", source.configuration, row.text("basis")?))
        {
            precisions.extend(PRECISIONS.into_iter().filter(|p| *p != 64));
        }
        for p in precisions {
            let budget = EnclosureBudget::new(p, ORDINARY_WORK)?;
            let output = plan
                .enclose_price_per_100(f64::from_bits(quote), kind, budget)
                .map_err(|e| format!("{} p{p}: {e}", row.id()))?;
            let EnclosureOutcome::Finite(report) = output else {
                return Err(format!(
                    "{} p{p}: ordinary target must be Finite: {output:?}",
                    row.id()
                )
                .into());
            };
            finite_complete(&report, plan, budget, quote, kind, source.flows.len());
            reference.compare(&report, source.useful, &format!("{} p{p}", row.id()))?;
        }
        used[index] = true;
        count += 1;
        let method = source
            .configuration
            .split(':')
            .next()
            .ok_or("missing method")?;
        if quote != 0 && first_nonzero.insert(method.to_owned()) {
            checkpoint(&format!("P first nonzero {method} {}", row.id()), count);
        }
        if count % 4096 == 0 {
            checkpoint("P", count);
        }
    }
    assert_eq!(count, 38400);
    assert_eq!(sampled.len(), 64);
    assert!(used.into_iter().all(|v| v));
    checkpoint("P", count);
    Ok(())
}

fn admission(sources: &[GenericSource]) -> TestResult {
    let table = fixture("admission")?;
    // Explicit test allowance: enough for the tiny frozen admission graphs,
    // bounded so valid extreme quotes need not finish expensive price math.
    let budget = EnclosureBudget::new(64, 1_000_000)?;
    let mut plan = None;
    let mut previous = None;
    let mut count = 0;
    let mut predicates = BTreeMap::new();
    for row in table.records() {
        if row.text("model")? != "Generic" {
            continue;
        }
        let index = source_index(row.text("source_id")?)?;
        let source = sources.get(index).ok_or("Q source missing")?;
        if previous != Some(index) {
            plan = Some(source.plan()?);
            previous = Some(index);
        }
        let plan = plan.as_ref().ok_or("Q plan missing")?;
        let quote = bits(row.text("yield_bits")?)?;
        let predicate = row.text("predicate")?;
        *predicates.entry(predicate).or_insert(0_usize) += 1;
        let output =
            plan.enclose_price_per_100(f64::from_bits(quote), BondPriceKind::Dirty, budget);
        if predicate == "MathematicallyAdmitted" {
            match output
                .map_err(|e| format!("{}: valid mathematical quote rejected: {e}", row.id()))?
            {
                EnclosureOutcome::Finite(report) => finite_complete(
                    &report,
                    plan,
                    budget,
                    quote,
                    BondPriceKind::Dirty,
                    source.flows.len(),
                ),
                EnclosureOutcome::Unresolved(report) => {
                    check_origin(report.origin(), plan);
                    check_context(
                        report.context(),
                        budget,
                        quote,
                        BondPriceKind::Dirty,
                        source.flows.len(),
                        true,
                    );
                    assert!(report.context().admission_complete(), "{}", row.id());
                }
                _ => return Err("unknown admitted outcome".into()),
            }
        } else {
            let error = output
                .err()
                .ok_or_else(|| format!("{}: invalid quote accepted", row.id()))?;
            let context = error.context();
            check_context(
                context,
                budget,
                quote,
                BondPriceKind::Dirty,
                source.flows.len(),
                false,
            );
            assert!(!context.admission_complete());
            assert_eq!(context.reduction_completed_flows(), 0);
            match predicate {
                "NonFiniteYield" => {
                    assert_eq!(error.kind(), "enclosure.non_finite_yield", "{}", row.id());
                    assert!(error.domain_cause().is_none());
                    assert!(error.source().is_some());
                    assert_eq!(context.stage(), EnclosureStage::QuoteAdmission);
                    assert_eq!(context.admission_checked_flows(), 0);
                    assert_eq!(context.next_original_index(), None);
                }
                "YieldNotAboveFloor" => {
                    assert_eq!(
                        error.kind(),
                        "enclosure.yield_not_above_floor",
                        "{}",
                        row.id()
                    );
                    assert_eq!(
                        error.domain_cause(),
                        Some(EnclosureDomainCause::YieldNotAboveFloor {
                            payments_per_year: row.parse("frequency")?
                        })
                    );
                    assert_eq!(context.stage(), EnclosureStage::QuoteAdmission);
                    assert_eq!(context.admission_checked_flows(), 0);
                    assert_eq!(context.next_original_index(), None);
                }
                "NonPositiveSimpleDenominator" => {
                    assert_eq!(
                        error.kind(),
                        "enclosure.nonpositive_simple_denominator",
                        "{}",
                        row.id()
                    );
                    let original: usize = row.parse("original_flow_index")?;
                    let position = source
                        .flows
                        .iter()
                        .position(|f| f.index == original)
                        .ok_or("original Q index missing")?;
                    assert_eq!(
                        error.domain_cause(),
                        Some(EnclosureDomainCause::NonpositiveSimpleDenominator {
                            horizon_bits: source.flows[position].horizon_bits
                        })
                    );
                    assert_eq!(context.stage(), EnclosureStage::FlowAdmission);
                    assert_eq!(context.next_original_index(), Some(original));
                    assert_eq!(context.admission_checked_flows(), position + 1);
                }
                _ => return Err(format!("unexpected Generic Q predicate {predicate}").into()),
            }
        }
        count += 1;
        if count % 4096 == 0 {
            checkpoint("Q", count);
        }
    }
    assert_eq!(count, 13161);
    assert_eq!(predicates.len(), 4);
    checkpoint("Q", count);
    Ok(())
}

fn boundaries() -> TestResult {
    let table = fixture("boundary")?;
    let mut count = 0;
    for row in table.records() {
        if row.text("model")? != "Generic" {
            continue;
        }
        let source = GenericSource::from_boundary(
            row.text("context_id")?,
            row.text("input")?,
            row.text("compounding")?,
            row.text("accrued_bits")?,
            row.text("flows")?,
        )?;
        let plan = source.plan()?;
        let kind = price_kind(row.text("basis")?)?;
        let quote = bits(row.text("yield_bits")?)?;
        let reference = Reference::boundary(
            row.text("lower_bits")?,
            row.text("upper_bits")?,
            row.text("point_kind")?,
            row.text("point_coefficient")?,
            row.text("point_scale")?,
            row.text("point_bits")?,
        )?;
        let budget = EnclosureBudget::new(64, ORDINARY_WORK)?;
        match plan
            .enclose_price_per_100(f64::from_bits(quote), kind, budget)
            .map_err(|e| format!("{}: {e}", row.id()))?
        {
            EnclosureOutcome::Finite(report) => {
                finite_complete(&report, &plan, budget, quote, kind, source.flows.len());
                reference.compare(&report, false, row.id())?;
            }
            EnclosureOutcome::Unresolved(report) => {
                check_origin(report.origin(), &plan);
                check_context(
                    report.context(),
                    budget,
                    quote,
                    kind,
                    source.flows.len(),
                    true,
                );
                assert!(report.context().admission_complete());
                if reference.upper.is_none() || reference.lower.is_none() {
                    assert_eq!(
                        report.reason(),
                        EnclosureUnresolvedReason::FiniteEndpointUnavailable,
                        "{}",
                        row.id()
                    );
                    assert!(report.complete_extended_arithmetic());
                }
                // A stress-case unresolved report is not a passing finite
                // comparison; it remains an explicitly allowed outcome here.
            }
            _ => return Err("unknown boundary outcome".into()),
        }
        count += 1;
    }
    assert_eq!(count, 28);
    checkpoint("B2", count);
    Ok(())
}

fn hands() -> TestResult {
    let table = fixture("hands")?;
    let mut count = 0;
    for row in table.records() {
        if row.text("model")? != "generic" {
            continue;
        }
        let source = GenericSource::from_hand(
            row.text("context_id")?,
            row.text("input")?,
            row.text("source_exact")?,
        )?;
        let plan = source.plan()?;
        let literals = Flat::parse(row.text("input")?)?;
        let quote = bits(literals.string("y_bits")?)?;
        let kind = price_kind(row.text("basis")?)?;
        let reference = Reference::hand(
            row.text("outward_lower_bits")?,
            row.text("outward_upper_bits")?,
            row.text("reference_kind")?,
            row.text("canonical_decimal_point")?,
        )?;
        for p in PRECISIONS {
            let budget = EnclosureBudget::new(p, ORDINARY_WORK)?;
            let output = plan
                .enclose_price_per_100(f64::from_bits(quote), kind, budget)
                .map_err(|e| format!("{} p{p}: {e}", row.id()))?;
            let EnclosureOutcome::Finite(report) = output else {
                return Err(format!("{}: ordinary hand unresolved", row.id()).into());
            };
            finite_complete(&report, &plan, budget, quote, kind, source.flows.len());
            reference.compare(&report, true, row.id())?;
        }
        count += 1;
    }
    assert_eq!(count, 8);
    checkpoint("H", count);
    Ok(())
}

fn hybrid() -> TestResult {
    let table = fixture("hybrid")?;
    let mut count = 0;
    let mut configurations = BTreeSet::new();
    for row in table.records() {
        let periods = Flat::parse(row.text("periods")?)?;
        let mut definitions = Vec::new();
        for i in 0..periods.count("$")? {
            definitions.push(DeterminationPeriod::new(
                periods.string(&format!("[{i}][0]"))?.parse()?,
                periods.string(&format!("[{i}][1]"))?.parse()?,
            )?);
        }
        let icma = ActualActualIcma::new(
            definitions,
            frequency(row.parse("accrual_events_per_year")?)?,
        )?;
        let quote_count = if row.text("m")? == "-" {
            None
        } else {
            Some(row.parse("m")?)
        };
        let comp = compounding(row.text("method")?, quote_count)?;
        let convention = BondYieldConvention::new(icma, comp);
        let settlement = BondSettlement::new(
            row.text("settlement")?.parse()?,
            PaymentDateInclusion::Include,
        );
        let payment = row.text("payment")?.parse()?;
        let bond = Bond::zero_coupon(
            row.text("issue")?.parse()?,
            payment,
            row.parse("face_literal")?,
            SimpleCashFlow::new(payment, row.parse("redemption_literal")?)?,
        )?;
        let plan = BondPricingPlan::new(
            &bond,
            settlement,
            &convention,
            row.parse("expected_accrual_work")?,
        )?;
        assert_eq!(
            plan.accrual_work_used(),
            row.parse::<u64>("expected_accrual_work")?
        );
        assert_eq!(
            convention
                .accrual()
                .year_fraction(settlement.date(), payment)?
                .to_bits(),
            bits(row.text("expected_horizon_bits")?)?
        );
        let budget = EnclosureBudget::new(64, ORDINARY_WORK)?;
        let output = plan.enclose_price_per_100(
            row.parse("trial_yield_literal")?,
            BondPriceKind::Dirty,
            budget,
        )?;
        let EnclosureOutcome::Finite(report) = output else {
            return Err(format!("{}: zero-yield ICMA unresolved", row.id()).into());
        };
        finite_complete(&report, &plan, budget, 0, BondPriceKind::Dirty, 1);
        assert_eq!(report.decimal_point(), Some(DecimalAmount::new(100, 0)?));
        configurations.insert((row.text("method")?, row.text("quote_frequency")?));
        count += 1;
        // Zero-yield equality is NOT branch-selection evidence. The actual
        // selector is checked against these same rows in generic_controls.
    }
    assert_eq!((count, configurations.len()), (144, 32));
    checkpoint("HYBRID", count);
    Ok(())
}

fn repeated_and_budget_threshold(sources: &[GenericSource]) -> TestResult {
    let source = sources
        .iter()
        .find(|s| {
            s.configuration == "Continuous:-"
                && s.flows.len() == 1
                && s.flows[0].horizon_bits == 1.0_f64.to_bits()
        })
        .ok_or("ordinary budget representative")?;
    let plan = source.plan()?;
    let quote = 0x3fa0_0000_0000_0000;
    let high = EnclosureBudget::new(64, ORDINARY_WORK)?;
    let first = plan.enclose_price_per_100(f64::from_bits(quote), BondPriceKind::Dirty, high)?;
    let EnclosureOutcome::Finite(first) = first else {
        return Err("budget representative unresolved".into());
    };
    let used = first.context().work_used();
    let again = plan.enclose_price_per_100(f64::from_bits(quote), BondPriceKind::Dirty, high)?;
    let EnclosureOutcome::Finite(again) = again else {
        return Err("repeat unexpectedly unresolved".into());
    };
    assert_eq!(first.context(), again.context());
    assert_eq!(first.settlement_evidence(), again.settlement_evidence());
    assert_eq!(first.proof_kind(), again.proof_kind());
    if used > 53_680 {
        let below = EnclosureBudget::new(64, used - 1)?;
        let output =
            plan.enclose_price_per_100(f64::from_bits(quote), BondPriceKind::Dirty, below)?;
        let EnclosureOutcome::Unresolved(report) = output else {
            return Err("one-short completion budget unexpectedly finite".into());
        };
        assert_eq!(
            report.reason(),
            EnclosureUnresolvedReason::WorkLimitAfterAdmission
        );
        assert!(report.context().admission_complete());
        assert!(report.context().work_used() <= below.max_work());
        for maximum in [used, used + 1] {
            let budget = EnclosureBudget::new(64, maximum)?;
            let output =
                plan.enclose_price_per_100(f64::from_bits(quote), BondPriceKind::Dirty, budget)?;
            let EnclosureOutcome::Finite(report) = output else {
                return Err("exact completion budget did not finish".into());
            };
            assert_eq!(report.context().work_used(), used);
            assert_eq!(report.settlement_evidence(), first.settlement_evidence());
        }
    }
    Ok(())
}

#[test]
fn actual_generic_enclosures_consume_frozen_financial_partitions() -> TestResult {
    let sources = load_sources()?;
    ordinary(&sources)?;
    admission(&sources)?;
    repeated_and_budget_threshold(&sources)?;
    drop(sources); // No simultaneous large S/P/Q tables.
    boundaries()?;
    hands()?;
    hybrid()?;
    Ok(())
}
