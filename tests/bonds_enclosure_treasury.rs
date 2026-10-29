//! Actual Treasury enclosure consumers (FR-ENCL-02/03/05..11/16..18).
//!
//! Uses the complete frozen S/P/Q/H and single B2 Treasury partitions.
//! Observed valuations and native elementary functions supply no price premise.

#[allow(
    dead_code,
    reason = "the shared enclosure support also serves other model consumers"
)]
#[path = "support/enclosure.rs"]
mod enclosure;
mod support;

use enclosure::{Flat, ORDINARY_WORK, PRECISIONS, Reference, TestResult, bits, price_kind};
use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondPriceKind, CouponEntitlement, EnclosureBasis, EnclosureBudget, EnclosureContext,
    EnclosureDomainCause, EnclosureOrigin, EnclosureOutcome, EnclosureStage,
    EnclosureUnresolvedReason, PriceEnclosure, TreasuryDailyBasis, TreasuryFirstPeriod,
    TreasuryPricingPlan, TreasurySourcePolicy, TreasuryTieRule, UsTreasury,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::time::Date;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

const SOURCE_VERSION: &str = "US-Treasury-31CFR356-AppB-fixed-2026-10-16-local-v1";

fn fixture(name: &str) -> TestResult<Fixture> {
    Ok(Fixture::load(Fixture::repository_path(&format!(
        "validation/fixtures/bonds/enclosure-{name}.fixture"
    )))?)
}

#[expect(
    clippy::print_stderr,
    reason = "durable actual financial-consumer checkpoints, not library I/O"
)]
fn checkpoint(family: &str, count: usize) {
    eprintln!("Treasury enclosure {family}: {count} checked");
}

fn source_index(id: &str) -> TestResult<usize> {
    id.strip_prefix("ENC-SOURCE-")
        .ok_or("source ID prefix")?
        .parse::<usize>()?
        .checked_sub(9261)
        .ok_or_else(|| "source is outside the frozen Treasury partition".into())
}

fn role_text(role: TreasuryFirstPeriod) -> TestResult<&'static str> {
    match role {
        TreasuryFirstPeriod::Regular => Ok("Regular"),
        TreasuryFirstPeriod::Short => Ok("Short"),
        TreasuryFirstPeriod::Long => Ok("Long"),
        _ => Err("unknown first-coupon role".into()),
    }
}

fn dates(input: &Flat<'_>, field: &str) -> TestResult<Vec<Date>> {
    (0..input.count(field)?)
        .map(|i| Ok(input.string(&format!("{field}[{i}]"))?.parse()?))
        .collect()
}

fn policy(daily: &str, tie: &str) -> TestResult<TreasurySourcePolicy> {
    let daily = match daily {
        "ExactRatio" => TreasuryDailyBasis::ExactRatio,
        "NineDecimal" => TreasuryDailyBasis::NineDecimal,
        _ => return Err("unknown Treasury daily basis".into()),
    };
    let tie = match tie {
        "NearestTiesEven" => TreasuryTieRule::NearestTiesEven,
        "NearestTiesAwayFromZero" => TreasuryTieRule::NearestTiesAwayFromZero,
        _ => return Err("unknown Treasury publication tie".into()),
    };
    Ok(TreasurySourcePolicy::new(daily, tie))
}

#[derive(Debug)]
struct TreasurySource {
    id: String,
    treasury: UsTreasury,
    settlement: Date,
    bundle_cap: usize,
    next: usize,
    r: u32,
    s: u32,
    indices: Vec<usize>,
    payments: Vec<Date>,
    family: String,
    shape: String,
    useful: bool,
}

impl TreasurySource {
    fn from_source(id: &str, family: &str, literal: &str, summary: &str) -> TestResult<Self> {
        let input = Flat::parse(literal)?;
        let expected = Flat::parse(summary)?;
        let coupon = DecimalAmount::new(
            input.string("coupon_coefficient")?.parse()?,
            u8::try_from(input.integer("coupon_scale")?)?,
        )?;
        let (tag, preserve) = input.node("preserve_eom")?;
        assert_eq!(tag, "b");
        let end_of_month = match preserve {
            "0" => EndOfMonth::Ignore,
            "1" => EndOfMonth::Preserve,
            _ => return Err("unknown EOM boolean".into()),
        };
        let quasi = dates(&input, "quasi")?;
        let dated = input.string("dated")?.parse()?;
        let issue = input.string("issue")?.parse()?;
        let maturity = input.string("maturity")?.parse()?;
        let first = input.string("first_coupon")?.parse()?;
        let source_policy = policy(input.string("daily_basis")?, input.string("tie")?)?;
        let treasury = UsTreasury::new(
            dated,
            issue,
            maturity,
            coupon,
            first,
            quasi.clone(),
            end_of_month,
            source_policy,
            input.integer("construction.date_cap")?,
        )?;
        assert_eq!(treasury.original_dated_date(), dated);
        assert_eq!(treasury.original_issue_date(), issue);
        assert_eq!(treasury.maturity_date(), maturity);
        assert_eq!(treasury.first_coupon_date(), first);
        assert_eq!(treasury.coupon_per_100(), coupon);
        assert_eq!(treasury.quasi_dates(), quasi);
        assert_eq!(treasury.end_of_month(), end_of_month);
        assert_eq!(treasury.source_policy(), source_policy);
        assert_eq!(treasury.source_version(), SOURCE_VERSION);
        let first_role = family.split(',').next().ok_or("missing first role")?;
        assert_eq!(role_text(treasury.first_period())?, first_role);
        assert_eq!(expected.string("index_kind")?, "OriginalQuasiDateIndex");
        let count = expected.integer("prepared_bundle_count")?;
        assert_eq!(expected.count("prepared_bundle_indices")?, count);
        assert_eq!(expected.count("prepared_payment_dates")?, count);
        let indices = (0..count)
            .map(|i| expected.integer(&format!("prepared_bundle_indices[{i}]")))
            .collect::<TestResult<Vec<_>>>()?;
        let payments = (0..count)
            .map(|i| {
                Ok(expected
                    .string(&format!("prepared_payment_dates[{i}]"))?
                    .parse()?)
            })
            .collect::<TestResult<Vec<_>>>()?;
        let pieces = expected.count("pieces")?;
        assert!((1..=2).contains(&pieces));
        for i in 0..pieces {
            assert_eq!(expected.count(&format!("pieces[{i}]"))?, 2);
            let t = expected.integer(&format!("pieces[{i}][0]"))?;
            let s = expected.integer(&format!("pieces[{i}][1]"))?;
            assert!(t <= s && s > 0 && s <= 184);
        }
        // The six unchanged coupon inputs divide into C<=6 and MAXscale0.
        // Small selected raw coupons are <C, a par bundle <=C+100 and
        // published AI <=C+1e-6, hence all are below1024. For MAXscale0,
        // every strict-future plan retains a positive coupon with day fraction
        // >=1/184, so its raw coefficient >=C/368 >1024. No rounded getter
        // decides this exact source-based width selector.
        let useful = coupon <= DecimalAmount::new(6, 0)?;
        if !useful {
            assert_eq!(coupon, DecimalAmount::new(i128::MAX, 0)?);
            assert!(coupon.coefficient().unsigned_abs() > 1024 * 368);
        }
        Ok(Self {
            id: id.to_owned(),
            treasury,
            settlement: input.string("settlement")?.parse()?,
            bundle_cap: input.integer("construction.bundle_cap")?,
            next: expected.integer("next_quasi")?,
            r: u32::try_from(expected.integer("r")?)?,
            s: u32::try_from(expected.integer("s")?)?,
            indices,
            payments,
            family: family.to_owned(),
            shape: format!("{first_role}:{pieces}piece"),
            useful,
        })
    }

    fn plan(&self) -> TestResult<TreasuryPricingPlan<'_>> {
        let plan = TreasuryPricingPlan::new(&self.treasury, self.settlement, self.bundle_cap)?;
        assert!(std::ptr::eq(plan.treasury(), &self.treasury));
        assert_eq!(plan.source_version(), SOURCE_VERSION);
        assert_eq!(plan.source_policy(), self.treasury.source_policy());
        assert_eq!(plan.settlement_date(), self.settlement);
        assert_eq!(plan.next_quasi_index(), self.next, "{}", self.id);
        assert_eq!(plan.remaining_days(), self.r, "{}", self.id);
        assert_eq!(plan.quasi_period_days(), self.s, "{}", self.id);
        assert_eq!(
            plan.remaining_quasi_periods(),
            u32::try_from(self.treasury.quasi_dates().len() - 1 - self.next)?
        );
        assert_eq!(plan.bundle_count(), self.indices.len());
        for (at, (&index, payment)) in self.indices.iter().zip(&self.payments).enumerate() {
            assert_eq!(plan.bundle_quasi_index(at), Some(index), "{}", self.id);
            assert_eq!(self.treasury.quasi_dates().get(index), Some(payment));
        }
        assert_eq!(plan.bundle_quasi_index(self.indices.len()), None);
        if !self.useful {
            assert_eq!(
                plan.coupon_entitlement(self.treasury.coupon_count() - 1),
                Some(CouponEntitlement::BuyerEntitled),
            );
        }
        Ok(plan)
    }
}

fn load_sources() -> TestResult<Vec<TreasurySource>> {
    let table = fixture("sources")?;
    let mut sources = Vec::new();
    let mut families = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "Treasury" {
            continue;
        }
        assert_eq!(source_index(row.id())?, sources.len());
        let source = TreasurySource::from_source(
            row.id(),
            row.text("families")?,
            row.text("input")?,
            row.text("source_summary")?,
        )?;
        source.plan()?;
        families.insert(source.family.clone());
        sources.push(source);
    }
    assert_eq!(sources.len(), 2160);
    assert_eq!(families.len(), 32);
    checkpoint("S", sources.len());
    Ok(sources)
}

fn check_origin(origin: EnclosureOrigin<'_, '_>, plan: &TreasuryPricingPlan<'_>) {
    assert!(
        origin
            .treasury_plan()
            .is_some_and(|p| std::ptr::eq(p, plan))
    );
    assert!(
        origin.generic_plan().is_none()
            && origin.gilt_plan().is_none()
            && origin.canadian_plan().is_none()
    );
}

fn check_context(
    context: EnclosureContext,
    budget: EnclosureBudget,
    quote: u64,
    basis: BondPriceKind,
    count: usize,
    successful: bool,
) {
    assert_eq!(context.budget(), budget);
    assert!(context.work_used() <= budget.max_work());
    assert_eq!(context.basis(), EnclosureBasis::Treasury(basis));
    assert_eq!(
        context.yield_bits(),
        if successful && quote << 1 == 0 {
            0
        } else {
            quote
        }
    );
    assert!(context.admission_checked_flows() <= count);
    assert!(context.reduction_completed_flows() <= count);
    if context.admission_complete() {
        assert_eq!(context.admission_checked_flows(), count);
    }
}

fn finite_complete(
    report: &PriceEnclosure<'_, '_>,
    plan: &TreasuryPricingPlan<'_>,
    budget: EnclosureBudget,
    quote: u64,
    basis: BondPriceKind,
) {
    check_origin(report.origin(), plan);
    check_context(
        report.context(),
        budget,
        quote,
        basis,
        plan.bundle_count(),
        true,
    );
    assert!(report.context().admission_complete());
    assert_eq!(
        report.context().reduction_completed_flows(),
        plan.bundle_count()
    );
    assert_eq!(report.context().next_original_index(), None);
    assert_eq!(report.context().stage(), EnclosureStage::Completion);
}

fn ordinary(sources: &[TreasurySource]) -> TestResult {
    let table = fixture("prices-v2")?;
    let mut previous = None;
    let mut plan = None;
    let mut count = 0;
    let mut sampled = BTreeSet::new();
    let mut bases = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "Treasury" {
            continue;
        }
        let index = source_index(row.text("source_id")?)?;
        let source = sources.get(index).ok_or("missing Treasury price source")?;
        assert_eq!(source.id, row.text("source_id")?);
        if previous != Some(index) {
            plan = Some(source.plan()?);
            previous = Some(index);
        }
        let plan = plan.as_ref().ok_or("missing prepared Treasury plan")?;
        let basis = price_kind(row.text("basis")?)?;
        bases.insert(row.text("basis")?);
        let quote = bits(row.text("yield_bits")?)?;
        let reference = Reference::price(
            row.text("outward_lower_bits")?,
            row.text("outward_upper_bits")?,
            row.text("point_encoding")?,
            row.text("point_value")?,
        )?;
        let mut precisions = vec![64];
        if quote == 0x3fa0_0000_0000_0000
            && sampled.insert(format!("{}:{}", source.shape, row.text("basis")?))
        {
            precisions.extend(PRECISIONS.into_iter().filter(|p| *p != 64));
        }
        for p in precisions {
            let budget = EnclosureBudget::new(p, ORDINARY_WORK)?;
            let output = plan
                .enclose_price_per_100(f64::from_bits(quote), basis, budget)
                .map_err(|e| format!("{} p{p}: {e}", row.id()))?;
            let EnclosureOutcome::Finite(report) = output else {
                return Err(format!(
                    "{} p{p}: ordinary target must be Finite: {output:?}",
                    row.id()
                )
                .into());
            };
            finite_complete(&report, plan, budget, quote, basis);
            reference.compare(&report, source.useful, &format!("{} p{p}", row.id()))?;
        }
        count += 1;
        if count % 256 == 0 {
            checkpoint("P", count);
        }
    }
    assert_eq!(count, 9024);
    assert_eq!(sampled.len(), 8);
    assert_eq!(bases, BTreeSet::from(["Clean", "Dirty"]));
    checkpoint("P", count);
    Ok(())
}

fn admission(sources: &[TreasurySource]) -> TestResult {
    let table = fixture("admission")?;
    // Small frozen Treasury graphs complete every exact admission within the
    // minimum budget; subsequent pricing may legitimately remain unresolved.
    let budget = EnclosureBudget::new(64, 53_680)?;
    let mut count = 0;
    let mut predicates = BTreeMap::new();
    let mut previous = None;
    let mut plan = None;
    for row in table.records() {
        if row.text("model")? != "Treasury" {
            continue;
        }
        let index = source_index(row.text("source_id")?)?;
        if previous != Some(index) {
            plan = Some(sources.get(index).ok_or("missing Q source")?.plan()?);
            previous = Some(index);
        }
        let plan = plan.as_ref().ok_or("missing Q plan")?;
        let quote = bits(row.text("yield_bits")?)?;
        let predicate = row.text("predicate")?;
        *predicates.entry(predicate).or_insert(0_usize) += 1;
        for basis in [BondPriceKind::Dirty, BondPriceKind::Clean] {
            let result = plan.enclose_price_per_100(f64::from_bits(quote), basis, budget);
            if predicate == "MathematicallyAdmitted" {
                match result.map_err(|e| format!("{}: admitted quote rejected: {e}", row.id()))? {
                    EnclosureOutcome::Finite(report) => {
                        finite_complete(&report, plan, budget, quote, basis);
                        assert_eq!(
                            report.context().yield_bits(),
                            bits(row.text("canonical_quote_bits")?)?
                        );
                    }
                    EnclosureOutcome::Unresolved(report) => {
                        check_origin(report.origin(), plan);
                        check_context(
                            report.context(),
                            budget,
                            quote,
                            basis,
                            plan.bundle_count(),
                            true,
                        );
                        assert!(report.context().admission_complete());
                        assert_eq!(
                            report.context().yield_bits(),
                            bits(row.text("canonical_quote_bits")?)?
                        );
                        if report.reason() == EnclosureUnresolvedReason::FiniteEndpointUnavailable {
                            assert!(report.complete_extended_arithmetic());
                        }
                    }
                    _ => return Err("unknown admitted Treasury outcome".into()),
                }
            } else {
                let error = result
                    .err()
                    .ok_or_else(|| format!("{}: invalid quote accepted", row.id()))?;
                let context = error.context();
                check_context(context, budget, quote, basis, plan.bundle_count(), false);
                assert!(!context.admission_complete());
                assert_eq!(context.admission_checked_flows(), 0);
                assert_eq!(context.reduction_completed_flows(), 0);
                assert_eq!(context.next_original_index(), None);
                assert_eq!(context.stage(), EnclosureStage::QuoteAdmission);
                assert_eq!(context.work_used(), 1_088);
                assert!(error.source().is_some());
                match predicate {
                    "NonFiniteYield" => {
                        assert_eq!(error.kind(), "enclosure.non_finite_yield");
                        assert!(error.domain_cause().is_none());
                        assert_eq!(
                            context.yield_bits(),
                            bits(row.text("attempted_quote_bits")?)?
                        );
                    }
                    "YieldNotAboveFloor" => {
                        assert_eq!(error.kind(), "enclosure.yield_not_above_floor");
                        assert_eq!(
                            error.domain_cause(),
                            Some(EnclosureDomainCause::YieldNotAboveFloor {
                                payments_per_year: row.parse("frequency")?
                            })
                        );
                    }
                    _ => return Err(format!("unexpected Treasury predicate {predicate}").into()),
                }
            }
        }
        count += 1;
    }
    assert_eq!(count, 2040);
    assert_eq!(predicates.len(), 3);
    checkpoint("Q", count);
    Ok(())
}

fn hand_treasury(input: &Flat<'_>) -> TestResult<UsTreasury> {
    let eom = match input.string("eom")? {
        "Ignore" => EndOfMonth::Ignore,
        "Preserve" => EndOfMonth::Preserve,
        _ => return Err("unknown hand EOM".into()),
    };
    Ok(UsTreasury::new(
        input.string("dated")?.parse()?,
        input.string("issue")?.parse()?,
        input.string("maturity")?.parse()?,
        input.string("coupon")?.parse()?,
        input.string("first")?.parse()?,
        dates(input, "quasi")?,
        eom,
        policy(input.string("daily_basis")?, input.string("tie")?)?,
        input.integer("max_dates")?,
    )?)
}

fn hands() -> TestResult {
    let table = fixture("hands")?;
    let mut count = 0;
    let mut contexts = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "treasury" {
            continue;
        }
        let input = Flat::parse(row.text("input")?)?;
        let expected = Flat::parse(row.text("metadata_expected")?)?;
        let treasury = hand_treasury(&input)?;
        let plan = TreasuryPricingPlan::new(
            &treasury,
            input.string("settlement")?.parse()?,
            input.integer("max_bundles")?,
        )?;
        assert_eq!(plan.bundle_count(), expected.integer("bundles")?);
        assert_eq!(
            plan.remaining_days(),
            u32::try_from(expected.integer("r")?)?
        );
        assert_eq!(
            plan.quasi_period_days(),
            u32::try_from(expected.integer("s")?)?
        );
        assert_eq!(
            plan.remaining_quasi_periods(),
            u32::try_from(expected.integer("n")?)?
        );
        assert_eq!(
            role_text(treasury.first_period())?,
            expected.string("first_role")?
        );
        assert_eq!(plan.source_policy(), treasury.source_policy());
        let quote = bits(input.string("y_bits")?)?;
        let basis = price_kind(row.text("basis")?)?;
        let reference = Reference::hand(
            row.text("outward_lower_bits")?,
            row.text("outward_upper_bits")?,
            row.text("reference_kind")?,
            row.text("canonical_decimal_point")?,
        )?;
        for p in PRECISIONS {
            let budget = EnclosureBudget::new(p, ORDINARY_WORK)?;
            let output = plan.enclose_price_per_100(f64::from_bits(quote), basis, budget)?;
            let EnclosureOutcome::Finite(report) = output else {
                return Err(format!("{} p{p}: required finite hand result", row.id()).into());
            };
            finite_complete(&report, &plan, budget, quote, basis);
            reference.compare(&report, true, &format!("{} p{p}", row.id()))?;
        }
        contexts.insert(row.text("context_id")?);
        count += 1;
    }
    assert_eq!(count, 12);
    assert_eq!(
        contexts,
        BTreeSet::from([
            "TREG0",
            "TREGPREFIX",
            "TTIE_EE",
            "TTIE_EA",
            "TTIE_NE",
            "TTIE_NA"
        ])
    );
    checkpoint("H", count);
    Ok(())
}

fn boundaries(sources: &[TreasurySource]) -> TestResult {
    let table = fixture("boundary")?;
    let mut count = 0;
    let mut finite = 0;
    for row in table.records() {
        if row.text("model")? != "Treasury" {
            continue;
        }
        assert_eq!(row.text("context_id")?, "B2-REUSE-TREASURY");
        assert_eq!(row.text("admission_case_id")?, "ENC-ADMISSION-0014035");
        let source = sources
            .get(source_index(row.text("source_plan_id")?)?)
            .ok_or("B2 source")?;
        assert_eq!(source.id, "ENC-SOURCE-009261");
        let plan = source.plan()?;
        let quote = bits(row.text("yield_bits")?)?;
        let basis = price_kind(row.text("basis")?)?;
        let budget = EnclosureBudget::new(64, ORDINARY_WORK)?;
        let reference = Reference::boundary(
            row.text("lower_bits")?,
            row.text("upper_bits")?,
            row.text("point_kind")?,
            row.text("point_coefficient")?,
            row.text("point_scale")?,
            row.text("point_bits")?,
        )?;
        match plan.enclose_price_per_100(f64::from_bits(quote), basis, budget)? {
            EnclosureOutcome::Finite(report) => {
                finite_complete(&report, &plan, budget, quote, basis);
                reference.compare(&report, false, row.id())?;
                finite += 1;
            }
            EnclosureOutcome::Unresolved(report) => {
                check_origin(report.origin(), &plan);
                check_context(
                    report.context(),
                    budget,
                    quote,
                    basis,
                    plan.bundle_count(),
                    true,
                );
                assert!(report.context().admission_complete());
                if report.reason() == EnclosureUnresolvedReason::FiniteEndpointUnavailable {
                    assert!(report.complete_extended_arithmetic());
                }
                // Honest stress-case unresolved is not counted as a finite
                // target comparison. The same source/Q literals remain fixed.
            }
            _ => return Err("unknown B2 Treasury outcome".into()),
        }
        count += 1;
    }
    assert_eq!(count, 2);
    checkpoint("B2", count);
    checkpoint("B2 finite", finite);
    Ok(())
}

fn repeated_and_budget_threshold(sources: &[TreasurySource]) -> TestResult {
    let source = sources.first().ok_or("missing representative source")?;
    let plan = source.plan()?;
    for quote in [0_u64, 0x3fa0_0000_0000_0000] {
        for basis in [BondPriceKind::Dirty, BondPriceKind::Clean] {
            let ample = EnclosureBudget::new(64, ORDINARY_WORK)?;
            let EnclosureOutcome::Finite(first) =
                plan.enclose_price_per_100(f64::from_bits(quote), basis, ample)?
            else {
                return Err("representative ordinary price unresolved".into());
            };
            let EnclosureOutcome::Finite(second) =
                plan.enclose_price_per_100(f64::from_bits(quote), basis, ample)?
            else {
                return Err("repeat ordinary price unresolved".into());
            };
            finite_complete(&first, &plan, ample, quote, basis);
            assert_eq!(first.context(), second.context());
            assert_eq!(first.encoding(), second.encoding());
            assert_eq!(first.proof_kind(), second.proof_kind());
            assert_eq!(first.settlement_evidence(), second.settlement_evidence());
            let used = first.context().work_used();
            if used > 53_680 {
                let below = EnclosureBudget::new(64, used - 1)?;
                let output = plan.enclose_price_per_100(f64::from_bits(quote), basis, below)?;
                let EnclosureOutcome::Unresolved(report) = output else {
                    return Err("one unit below completed work must stop".into());
                };
                check_origin(report.origin(), &plan);
                check_context(
                    report.context(),
                    below,
                    quote,
                    basis,
                    plan.bundle_count(),
                    true,
                );
                assert!(report.context().admission_complete());
                assert_eq!(
                    report.reason(),
                    EnclosureUnresolvedReason::WorkLimitAfterAdmission
                );
                assert!(report.complete_extended_arithmetic());
                for maximum in [used, used + 1] {
                    let exact = EnclosureBudget::new(64, maximum)?;
                    let output = plan.enclose_price_per_100(f64::from_bits(quote), basis, exact)?;
                    let EnclosureOutcome::Finite(report) = output else {
                        return Err("observed completion budget did not complete".into());
                    };
                    assert_eq!(report.context().work_used(), used);
                    assert_eq!(report.settlement_evidence(), first.settlement_evidence());
                }
            }
        }
    }
    Ok(())
}

#[test]
fn actual_treasury_enclosures_consume_frozen_financial_partitions() -> TestResult {
    let sources = load_sources()?;
    // Cheap source/domain/hand/boundary checks run before the full ordinary
    // matrix. Only one large fixture table is retained at a time.
    admission(&sources)?;
    hands()?;
    boundaries(&sources)?;
    repeated_and_budget_threshold(&sources)?;
    ordinary(&sources)?;
    Ok(())
}
