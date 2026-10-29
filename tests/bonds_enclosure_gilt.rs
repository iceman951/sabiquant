//! Actual Gilt enclosure consumers (FR-ENCL-02/03/05..11/16..18).
//!
//! All expectations are the frozen S/P/Q/H and single B2 inputs. No observed
//! valuation or native elementary result is used as a price oracle.

#[allow(
    dead_code,
    reason = "the shared enclosure support also serves Generic and later named consumers"
)]
#[path = "support/enclosure.rs"]
mod enclosure;
mod support;

use enclosure::{Flat, ORDINARY_WORK, PRECISIONS, Reference, TestResult, bits, price_kind};
use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondPriceKind, ConventionalGilt, CouponEntitlement, EnclosureBasis, EnclosureBudget,
    EnclosureContext, EnclosureDomainCause, EnclosureOrigin, EnclosureOutcome, EnclosureStage,
    EnclosureUnresolvedReason, GiltFirstPeriod, GiltPricingPlan, PriceEnclosure,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::time::Date;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

const SOURCE_VERSION: &str = "UK-DMO-Conventional-v4-2024-12-18";

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
    eprintln!("Gilt enclosure {family}: {count} checked");
}

fn source_index(id: &str) -> TestResult<usize> {
    let suffix = id.strip_prefix("ENC-SOURCE-").ok_or("source ID prefix")?;
    suffix
        .parse::<usize>()?
        .checked_sub(8961)
        .ok_or_else(|| "source is outside the frozen Gilt partition".into())
}

fn role_text(role: GiltFirstPeriod) -> TestResult<&'static str> {
    match role {
        GiltFirstPeriod::Regular => Ok("Regular"),
        GiltFirstPeriod::Short => Ok("Short"),
        GiltFirstPeriod::Long => Ok("Long"),
        _ => Err("unknown first-dividend role".into()),
    }
}

fn dates(input: &Flat<'_>, field: &str) -> TestResult<Vec<Date>> {
    (0..input.count(field)?)
        .map(|i| Ok(input.string(&format!("{field}[{i}]"))?.parse()?))
        .collect()
}

#[derive(Debug)]
struct GiltSource {
    id: String,
    gilt: ConventionalGilt,
    settlement: Date,
    bundle_cap: usize,
    next: usize,
    r: u32,
    s: u32,
    indices: Vec<usize>,
    payments: Vec<Date>,
    first_role: String,
    family: String,
    useful: bool,
    accrued_n: String,
    accrued_d: String,
}

impl GiltSource {
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
        let ex = dates(&input, "ex_cutoffs")?;
        let issue = input.string("issue")?.parse()?;
        let maturity = input.string("maturity")?.parse()?;
        let first = input.string("first_coupon")?.parse()?;
        let gilt = ConventionalGilt::new(
            issue,
            maturity,
            coupon,
            first,
            quasi.clone(),
            ex.clone(),
            end_of_month,
            input.integer("construction.date_cap")?,
        )?;
        assert_eq!(gilt.issue_date(), issue);
        assert_eq!(gilt.maturity_date(), maturity);
        assert_eq!(gilt.first_dividend_date(), first);
        assert_eq!(gilt.coupon_per_100(), coupon);
        assert_eq!(gilt.quasi_dates(), quasi);
        assert_eq!(gilt.ex_dividend_dates(), ex);
        assert_eq!(gilt.end_of_month(), end_of_month);
        assert_eq!(gilt.source_version(), SOURCE_VERSION);
        let first_role = family.split(',').next().ok_or("missing source role")?;
        assert_eq!(role_text(gilt.first_period())?, first_role);
        match first_role {
            "Regular" => assert_eq!(
                expected.string("first_dividend_publication")?,
                "NoPublicationExactHalfCoupon"
            ),
            "Short" | "Long" => assert_ne!(
                expected.string("first_dividend_publication")?,
                "NoPublicationExactHalfCoupon"
            ),
            _ => return Err("unexpected frozen first role".into()),
        }
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
        // The frozen six coupon literals divide into the small family (C<=6)
        // and MAX at scale0. Adopted Gilt geometry bounds a selected dividend
        // by C plus its publication unit and a terminal bundle by C+100.
        // The large-family exclusion is justified again from actual retained
        // entitlement or the exact frozen AI ratio in plan(), not a rounded AI.
        let useful = coupon <= DecimalAmount::new(6, 0)?;
        if !useful {
            assert_eq!(coupon, DecimalAmount::new(i128::MAX, 0)?);
        }
        Ok(Self {
            id: id.to_owned(),
            gilt,
            settlement: input.string("settlement")?.parse()?,
            bundle_cap: input.integer("construction.bundle_cap")?,
            next: expected.integer("next_quasi")?,
            r: u32::try_from(expected.integer("r")?)?,
            s: u32::try_from(expected.integer("s")?)?,
            indices,
            payments,
            first_role: first_role.to_owned(),
            family: family.to_owned(),
            useful,
            accrued_n: expected.string("source_accrued.n")?.to_owned(),
            accrued_d: expected.string("source_accrued.d")?.to_owned(),
        })
    }

    fn plan(&self) -> TestResult<GiltPricingPlan<'_>> {
        let plan = GiltPricingPlan::new(&self.gilt, self.settlement, self.bundle_cap)?;
        assert!(std::ptr::eq(plan.gilt(), &self.gilt));
        assert_eq!(plan.source_version(), SOURCE_VERSION);
        assert_eq!(plan.settlement_date(), self.settlement);
        assert_eq!(plan.next_quasi_index(), self.next, "{}", self.id);
        assert_eq!(plan.remaining_days(), self.r, "{}", self.id);
        assert_eq!(plan.quasi_period_days(), self.s, "{}", self.id);
        assert_eq!(plan.bundle_count(), self.indices.len());
        for (at, (&index, payment)) in self.indices.iter().zip(&self.payments).enumerate() {
            assert_eq!(plan.bundle_quasi_index(at), Some(index), "{}", self.id);
            assert_eq!(self.gilt.quasi_dates().get(index), Some(payment));
        }
        assert_eq!(plan.bundle_quasi_index(self.indices.len()), None);
        if !self.useful {
            if self.accrued_n != "0" {
                // For MAX scale0 the unreduced day denominator is <=2*184^2,
                // so this exact source-selector comparison fits u128. Compare
                // decimal magnitudes without rounding the large numerator.
                let denominator: u128 = self.accrued_d.parse()?;
                let limit = denominator.checked_mul(1024).ok_or("AI bound overflow")?;
                let magnitude = self
                    .accrued_n
                    .trim_start_matches('-')
                    .trim_start_matches('0');
                let bound = limit.to_string();
                assert!(
                    magnitude.len() > bound.len()
                        || (magnitude.len() == bound.len() && magnitude > bound.as_str()),
                    "{}: large-coupon width exclusion needs |AI|>1024",
                    self.id
                );
            } else {
                // A retained buyer dividend of MAX coupon is >1024 even for
                // the smallest positive admitted first-period day fraction.
                let retained = (0..self.gilt.dividend_count()).any(|i| {
                    plan.dividend_entitlement(i) == Some(CouponEntitlement::BuyerEntitled)
                        && self
                            .indices
                            .contains(&(self.gilt.first_dividend_quasi_index() + i))
                });
                assert!(
                    retained,
                    "{}: unproved large-coupon width exclusion",
                    self.id
                );
            }
        }
        Ok(plan)
    }
}

fn load_sources() -> TestResult<Vec<GiltSource>> {
    let table = fixture("sources")?;
    let mut sources = Vec::new();
    let mut families = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "Gilt" {
            continue;
        }
        assert_eq!(source_index(row.id())?, sources.len());
        let source = GiltSource::from_source(
            row.id(),
            row.text("families")?,
            row.text("input")?,
            row.text("source_summary")?,
        )?;
        source.plan()?; // Every one of S300 has actual public preparation.
        families.insert(source.family.clone());
        sources.push(source);
    }
    assert_eq!(sources.len(), 300);
    assert_eq!(families.len(), 22);
    checkpoint("S", sources.len());
    Ok(sources)
}

fn check_origin(origin: EnclosureOrigin<'_, '_>, plan: &GiltPricingPlan<'_>) {
    assert!(origin.gilt_plan().is_some_and(|p| std::ptr::eq(p, plan)));
    assert!(
        origin.generic_plan().is_none()
            && origin.treasury_plan().is_none()
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
    assert_eq!(context.basis(), EnclosureBasis::Gilt(basis));
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
    plan: &GiltPricingPlan<'_>,
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

fn ordinary(sources: &[GiltSource]) -> TestResult {
    let table = fixture("prices-v2")?;
    let mut previous = None;
    let mut plan = None;
    let mut count = 0;
    let mut sampled = BTreeSet::new();
    let mut bases = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "Gilt" {
            continue;
        }
        let index = source_index(row.text("source_id")?)?;
        let source = sources.get(index).ok_or("missing Gilt price source")?;
        assert_eq!(source.id, row.text("source_id")?);
        if previous != Some(index) {
            plan = Some(source.plan()?);
            previous = Some(index);
        }
        let plan = plan.as_ref().ok_or("missing prepared Gilt plan")?;
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
            && sampled.insert(format!("{}:{}", source.first_role, row.text("basis")?))
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
    assert_eq!(count, 1464);
    assert_eq!(sampled.len(), 6);
    assert_eq!(bases, BTreeSet::from(["Clean", "Dirty"]));
    checkpoint("P", count);
    Ok(())
}

fn admission(sources: &[GiltSource]) -> TestResult {
    let table = fixture("admission")?;
    // Small frozen Gilt graphs complete every exact admission within the
    // minimum budget; subsequent pricing may legitimately remain unresolved.
    let budget = EnclosureBudget::new(64, 53_680)?;
    let mut count = 0;
    let mut predicates = BTreeMap::new();
    let mut previous = None;
    let mut plan = None;
    for row in table.records() {
        if row.text("model")? != "Gilt" {
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
                    _ => return Err("unknown admitted Gilt outcome".into()),
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
                    _ => return Err(format!("unexpected Gilt predicate {predicate}").into()),
                }
            }
        }
        count += 1;
    }
    assert_eq!(count, 870);
    assert_eq!(predicates.len(), 3);
    checkpoint("Q", count);
    Ok(())
}

fn hand_gilt(input: &Flat<'_>) -> TestResult<ConventionalGilt> {
    let eom = match input.string("eom")? {
        "Ignore" => EndOfMonth::Ignore,
        "Preserve" => EndOfMonth::Preserve,
        _ => return Err("unknown hand EOM".into()),
    };
    Ok(ConventionalGilt::new(
        input.string("issue")?.parse()?,
        input.string("maturity")?.parse()?,
        input.string("coupon")?.parse()?,
        input.string("first")?.parse()?,
        dates(input, "quasi")?,
        dates(input, "ex_dates")?,
        eom,
        input.integer("max_dates")?,
    )?)
}

fn hands() -> TestResult {
    let table = fixture("hands")?;
    let mut count = 0;
    let mut contexts = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "gilt" {
            continue;
        }
        let input = Flat::parse(row.text("input")?)?;
        let expected = Flat::parse(row.text("metadata_expected")?)?;
        let gilt = hand_gilt(&input)?;
        let plan = GiltPricingPlan::new(
            &gilt,
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
            role_text(gilt.first_period())?,
            expected.string("first_role")?
        );
        if let Ok(entitlement) = expected.string("entitlement") {
            let state = match entitlement {
                "BuyerEntitled" => CouponEntitlement::BuyerEntitled,
                "ExCouponDetached" => CouponEntitlement::ExCouponDetached,
                _ => return Err("unknown hand entitlement".into()),
            };
            assert_eq!(plan.dividend_entitlement(0), Some(state));
        }
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
    assert_eq!(count, 8);
    assert_eq!(
        contexts,
        BTreeSet::from(["LREG0", "LREGHALF", "LLONGCUM", "LLONGEX"])
    );
    checkpoint("H", count);
    Ok(())
}

fn boundaries(sources: &[GiltSource]) -> TestResult {
    let table = fixture("boundary")?;
    let mut count = 0;
    let mut finite = 0;
    for row in table.records() {
        if row.text("model")? != "Gilt" {
            continue;
        }
        assert_eq!(row.text("context_id")?, "B2-REUSE-GILT");
        assert_eq!(row.text("admission_case_id")?, "ENC-ADMISSION-0013165");
        let source = sources
            .get(source_index(row.text("source_plan_id")?)?)
            .ok_or("B2 source")?;
        assert_eq!(source.id, "ENC-SOURCE-008961");
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
            _ => return Err("unknown B2 Gilt outcome".into()),
        }
        count += 1;
    }
    assert_eq!(count, 2);
    checkpoint("B2", count);
    checkpoint("B2 finite", finite);
    Ok(())
}

fn repeated_and_budget_threshold(sources: &[GiltSource]) -> TestResult {
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
fn actual_gilt_enclosures_consume_frozen_financial_partitions() -> TestResult {
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
