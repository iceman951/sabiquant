//! Actual Canadian enclosure consumers (FR-ENCL-01..11/16..18).
//!
//! Complete frozen Canadian S/P/Q/H and single-B2 partitions. Observed
//! valuations, risks and native elementary functions supply no price premise.

#[allow(dead_code, reason = "shared support also serves other model consumers")]
#[path = "support/enclosure.rs"]
mod enclosure;
mod support;

use enclosure::{Flat, ORDINARY_WORK, PRECISIONS, Reference, TestResult, bits};
use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    CanadianBond, CanadianCouponSchedule, CanadianFrequency, CanadianPeriodKind,
    CanadianPriceBasis, CanadianPricingPlan, CanadianSourcePolicy, CanadianYieldConvention,
    CanadianYieldSelection, CouponEntitlement, EnclosureBasis, EnclosureBudget, EnclosureContext,
    EnclosureDomainCause, EnclosureOrigin, EnclosureOutcome, EnclosureStage,
    EnclosureUnresolvedReason, PriceEnclosure,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::time::Date;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

const SOURCE_VERSION: &str = "CA-IIAC13-fixed-2026-10-23-local-v1";

fn fixture(name: &str) -> TestResult<Fixture> {
    Ok(Fixture::load(Fixture::repository_path(&format!(
        "validation/fixtures/bonds/enclosure-{name}.fixture"
    )))?)
}

#[expect(
    clippy::print_stderr,
    reason = "durable consumer checkpoints, not library I/O"
)]
fn checkpoint(family: &str, count: usize) {
    eprintln!("Canadian enclosure {family}: {count} checked");
}

fn source_index(id: &str) -> TestResult<usize> {
    id.strip_prefix("ENC-SOURCE-")
        .ok_or("source ID prefix")?
        .parse::<usize>()?
        .checked_sub(11_421)
        .ok_or_else(|| "source outside frozen Canadian partition".into())
}

fn frequency(count: u32) -> TestResult<CanadianFrequency> {
    CanadianFrequency::ALL
        .into_iter()
        .find(|f| f.payments_per_year() == count)
        .ok_or_else(|| format!("unknown Canadian frequency {count}").into())
}

fn named_frequency(name: &str) -> TestResult<CanadianFrequency> {
    match name {
        "Semiannual" => Ok(CanadianFrequency::Semiannual),
        "Quarterly" => Ok(CanadianFrequency::Quarterly),
        "Monthly" => Ok(CanadianFrequency::Monthly),
        _ => Err("unknown Canadian frequency name".into()),
    }
}

fn selection(name: &str) -> TestResult<CanadianYieldSelection> {
    match name {
        "Compound" => Ok(CanadianYieldSelection::Explicit(
            CanadianYieldConvention::Compound,
        )),
        "MoneyMarket" => Ok(CanadianYieldSelection::Explicit(
            CanadianYieldConvention::MoneyMarket,
        )),
        "LastCouponMoneyMarket" => Ok(CanadianYieldSelection::LastCouponMoneyMarket),
        _ => Err(format!("unknown Canadian selection {name}").into()),
    }
}

fn convention(name: &str) -> TestResult<CanadianYieldConvention> {
    match name {
        "Compound" => Ok(CanadianYieldConvention::Compound),
        "MoneyMarket" => Ok(CanadianYieldConvention::MoneyMarket),
        _ => Err("unknown active Canadian convention".into()),
    }
}

fn basis(name: &str) -> TestResult<CanadianPriceBasis> {
    match name {
        "Clean" => Ok(CanadianPriceBasis::Clean),
        "ModelDirty" => Ok(CanadianPriceBasis::ModelDirty),
        "SettlementDirty" => Ok(CanadianPriceBasis::SettlementDirty),
        _ => Err(format!("unknown Canadian basis {name}").into()),
    }
}

fn period_name(kind: CanadianPeriodKind) -> TestResult<&'static str> {
    match kind {
        CanadianPeriodKind::Regular => Ok("Regular"),
        CanadianPeriodKind::Short => Ok("Short"),
        CanadianPeriodKind::Long => Ok("Long"),
        CanadianPeriodKind::Composite => Ok("Composite"),
        _ => Err("unknown Canadian period kind".into()),
    }
}

fn dates(input: &Flat<'_>, field: &str) -> TestResult<Vec<Date>> {
    (0..input.count(field)?)
        .map(|i| Ok(input.string(&format!("{field}[{i}]"))?.parse()?))
        .collect()
}

#[derive(Debug)]
struct CanadianSource {
    id: String,
    bond: CanadianBond,
    settlement: Date,
    selection: CanadianYieldSelection,
    bundle_cap: usize,
    active: CanadianYieldConvention,
    indices: Vec<usize>,
    payments: Vec<Date>,
    tau_days: u32,
    family: String,
    geometry: String,
    selection_text: String,
    ai_equal: bool,
    useful: bool,
}

impl CanadianSource {
    fn from_source(id: &str, family: &str, literal: &str, summary: &str) -> TestResult<Self> {
        let input = Flat::parse(literal)?;
        let expected = Flat::parse(summary)?;
        let coupon = DecimalAmount::new(
            input.string("coupon_coefficient")?.parse()?,
            u8::try_from(input.integer("coupon_scale")?)?,
        )?;
        let f = frequency(u32::try_from(input.integer("frequency")?)?)?;
        let (tag, preserve) = input.node("preserve_eom")?;
        assert_eq!(tag, "b");
        let eom = match preserve {
            "0" => EndOfMonth::Ignore,
            "1" => EndOfMonth::Preserve,
            _ => return Err("unknown EOM boolean".into()),
        };
        assert_eq!(input.string("source_policy")?, "CanonicalIiac13LocalV1");
        let dated = input.string("dated")?.parse()?;
        let maturity = input.string("maturity")?.parse()?;
        let quasi = dates(&input, "quasi")?;
        let coupons = dates(&input, "coupons")?;
        let schedule = CanadianCouponSchedule::new(
            dated,
            maturity,
            f,
            quasi.clone(),
            coupons.clone(),
            eom,
            input.integer("construction.date_cap")?,
        )?;
        let issue = input.string("issue")?.parse()?;
        let true_maturity = input.string("true_maturity")?.parse()?;
        let bond = CanadianBond::new(
            issue,
            coupon,
            schedule,
            true_maturity,
            CanadianSourcePolicy::CanonicalIiac13LocalV1,
        )?;
        assert_eq!(bond.original_issue_date(), issue);
        assert_eq!(bond.coupon_per_100(), coupon);
        assert_eq!(bond.true_maturity_date(), true_maturity);
        assert_eq!(bond.source_version(), SOURCE_VERSION);
        assert_eq!(bond.schedule().accrual_start(), dated);
        assert_eq!(bond.schedule().stated_maturity(), maturity);
        assert_eq!(bond.schedule().frequency(), f);
        assert_eq!(bond.schedule().quasi_dates(), quasi);
        assert_eq!(bond.schedule().coupon_dates(), coupons);
        assert_eq!(bond.schedule().end_of_month(), eom);
        assert_eq!(
            expected.string("index_kind")?,
            "OriginalActualCouponOrdinal"
        );
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
        let active = convention(expected.string("active_convention")?)?;
        for name in ["Clean", "ModelDirty", "SettlementDirty"] {
            assert_eq!(expected.count(&format!("basis_shifts_A_AI.{name}"))?, 2);
            let mut actual = [0_i8; 2];
            for (i, value) in actual.iter_mut().enumerate() {
                let (tag, text) = expected.node(&format!("basis_shifts_A_AI.{name}[{i}]"))?;
                assert_eq!(tag, "i");
                *value = text.parse()?;
            }
            let wanted = match (active, name) {
                (_, "ModelDirty") => [0, 0],
                (CanadianYieldConvention::Compound, "Clean") => [-1, 0],
                (CanadianYieldConvention::Compound, "SettlementDirty") => [-1, 1],
                (CanadianYieldConvention::MoneyMarket, "Clean") => [0, -1],
                (CanadianYieldConvention::MoneyMarket, "SettlementDirty") => [0, 0],
                _ => return Err("unknown active convention/basis".into()),
            };
            assert_eq!(actual, wanted);
        }
        let ai_equal = expected.string("valuation_accrued.n")?
            == expected.string("settlement_accrued.n")?
            && expected.string("valuation_accrued.d")?
                == expected.string("settlement_accrued.d")?;
        if f == CanadianFrequency::Monthly {
            assert!(ai_equal, "monthly source AI=A");
        }
        // The unchanged corpus has four C<=6 inputs and MAXscale0. At most8
        // complete quasi cells bound every V/P/A/AI by C*8/f<=24; adding par
        // leaves every source coefficient below124. MAXscale0 has a positive
        // future coupon >=C/(f*365)>1024, so it is outside the exact subset.
        assert!((2..=9).contains(&quasi.len()));
        let useful = coupon <= DecimalAmount::new(6, 0)?;
        if !useful {
            assert_eq!(coupon, DecimalAmount::new(i128::MAX, 0)?);
            assert!(coupon.coefficient().unsigned_abs() > 1024 * 12 * 365);
        }
        Ok(Self {
            id: id.to_owned(),
            bond,
            settlement: input.string("settlement")?.parse()?,
            selection: selection(input.string("selection")?)?,
            bundle_cap: input.integer("construction.bundle_cap")?,
            active,
            indices,
            payments,
            tau_days: u32::try_from(expected.integer("maturity_horizon_days")?)?,
            family: family.to_owned(),
            geometry: family
                .split(',')
                .next()
                .ok_or("missing geometry")?
                .to_owned(),
            selection_text: input.string("selection")?.to_owned(),
            ai_equal,
            useful,
        })
    }

    fn plan(&self) -> TestResult<CanadianPricingPlan<'_>> {
        let plan =
            CanadianPricingPlan::new(&self.bond, self.settlement, self.selection, self.bundle_cap)?;
        assert!(std::ptr::eq(plan.bond(), &self.bond));
        assert_eq!(plan.source_version(), SOURCE_VERSION);
        assert_eq!(plan.source_policy(), self.bond.source_policy());
        assert_eq!(plan.settlement_date(), self.settlement);
        assert_eq!(plan.yield_selection(), self.selection);
        assert_eq!(plan.yield_convention(), self.active);
        assert_eq!(plan.bundle_count(), self.indices.len());
        assert_eq!(
            self.bond
                .true_maturity_date()
                .days_since(self.settlement)
                .unsigned_abs(),
            self.tau_days
        );
        if self.selection == CanadianYieldSelection::LastCouponMoneyMarket {
            assert_eq!(
                self.active,
                if self.indices.len() == 1 {
                    CanadianYieldConvention::MoneyMarket
                } else {
                    CanadianYieldConvention::Compound
                }
            );
        }
        for (at, (&index, payment)) in self.indices.iter().zip(&self.payments).enumerate() {
            assert_eq!(plan.bundle_coupon_index(at), Some(index), "{}", self.id);
            assert_eq!(self.bond.coupon_payment_date(index), Some(*payment));
        }
        for (i, date) in self.bond.schedule().coupon_dates().iter().enumerate() {
            assert_eq!(
                plan.coupon_entitlement(i),
                Some(if *date <= self.settlement {
                    CouponEntitlement::Ceased
                } else {
                    CouponEntitlement::BuyerEntitled
                })
            );
        }
        assert_eq!(plan.bundle_coupon_index(self.indices.len()), None);
        assert_eq!(
            plan.coupon_entitlement(self.bond.schedule().coupon_count()),
            None
        );
        Ok(plan)
    }

    fn categories(&self) -> [String; 7] {
        [
            format!("geometry:{}", self.geometry),
            format!(
                "frequency:{}",
                self.bond.schedule().frequency().payments_per_year()
            ),
            format!("convention:{:?}", self.active),
            format!("selection:{}", self.selection_text),
            format!("eom:{:?}", self.bond.schedule().end_of_month()),
            format!(
                "terminal:{}",
                self.bond.true_maturity_date() == self.bond.schedule().stated_maturity()
            ),
            format!("AI_equals_A:{}", self.ai_equal),
        ]
    }
}

fn load_sources() -> TestResult<Vec<CanadianSource>> {
    let table = fixture("sources")?;
    let mut sources = Vec::new();
    let mut families = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "Canadian" {
            continue;
        }
        assert_eq!(source_index(row.id())?, sources.len());
        let source = CanadianSource::from_source(
            row.id(),
            row.text("families")?,
            row.text("input")?,
            row.text("source_summary")?,
        )?;
        source.plan()?;
        families.insert(source.family.clone());
        sources.push(source);
    }
    assert_eq!(sources.len(), 19_500);
    assert_eq!(families.len(), 504);
    checkpoint("S", sources.len());
    Ok(sources)
}

fn check_origin(origin: EnclosureOrigin<'_, '_>, plan: &CanadianPricingPlan<'_>) {
    assert!(
        origin
            .canadian_plan()
            .is_some_and(|p| std::ptr::eq(p, plan))
    );
    assert!(
        origin.generic_plan().is_none()
            && origin.gilt_plan().is_none()
            && origin.treasury_plan().is_none()
    );
}

fn check_context(
    context: EnclosureContext,
    budget: EnclosureBudget,
    quote: u64,
    basis: CanadianPriceBasis,
    count: usize,
    successful: bool,
) {
    assert_eq!(context.budget(), budget);
    assert!(context.work_used() <= budget.max_work());
    assert_eq!(context.basis(), EnclosureBasis::Canadian(basis));
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
    plan: &CanadianPricingPlan<'_>,
    budget: EnclosureBudget,
    quote: u64,
    basis: CanadianPriceBasis,
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

fn admission(sources: &[CanadianSource]) -> TestResult {
    let table = fixture("admission")?;
    // Existing graphs have at most8 original bundles. The configuration
    // minimum completes global exact MM admission and all metadata checks;
    // subsequent source/price work may legitimately stop without evidence.
    let budget = EnclosureBudget::new(64, 53_680)?;
    let mut previous = None;
    let mut plan = None;
    let mut count = 0;
    let mut predicates = BTreeMap::new();
    for row in table.records() {
        if row.text("model")? != "Canadian" {
            continue;
        }
        let index = source_index(row.text("source_id")?)?;
        let source = sources.get(index).ok_or("missing Q source")?;
        if previous != Some(index) {
            plan = Some(source.plan()?);
            previous = Some(index);
        }
        let plan = plan.as_ref().ok_or("missing prepared Q plan")?;
        let quote = bits(row.text("yield_bits")?)?;
        let predicate = row.text("predicate")?;
        *predicates.entry(predicate).or_insert(0_usize) += 1;
        for kind in CanadianPriceBasis::ALL {
            let result = plan.enclose_price_per_100(f64::from_bits(quote), kind, budget);
            if predicate == "MathematicallyAdmitted" {
                match result.map_err(|e| format!("{}: admitted quote rejected: {e}", row.id()))? {
                    EnclosureOutcome::Finite(report) => {
                        finite_complete(&report, plan, budget, quote, kind);
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
                            kind,
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
                    _ => return Err("unknown Canadian admitted outcome".into()),
                }
            } else {
                let error = result
                    .err()
                    .ok_or_else(|| format!("{}: invalid quote accepted", row.id()))?;
                let context = error.context();
                check_context(context, budget, quote, kind, plan.bundle_count(), false);
                assert!(!context.admission_complete());
                assert_eq!(context.admission_checked_flows(), 0);
                assert_eq!(context.reduction_completed_flows(), 0);
                assert_eq!(context.next_original_index(), None);
                assert_eq!(context.stage(), EnclosureStage::QuoteAdmission);
                assert!(error.source().is_some());
                match predicate {
                    "NonFiniteYield" => {
                        assert_eq!(error.kind(), "enclosure.non_finite_yield");
                        assert_eq!(context.work_used(), 1_088);
                        assert_eq!(
                            context.yield_bits(),
                            bits(row.text("attempted_quote_bits")?)?
                        );
                        assert!(error.domain_cause().is_none());
                    }
                    "YieldNotAboveFloor" => {
                        assert_eq!(source.active, CanadianYieldConvention::Compound);
                        assert_eq!(error.kind(), "enclosure.yield_not_above_floor");
                        assert_eq!(context.work_used(), 1_088);
                        assert_eq!(
                            error.domain_cause(),
                            Some(EnclosureDomainCause::YieldNotAboveFloor {
                                payments_per_year: row.parse("frequency")?
                            })
                        );
                    }
                    "NonPositiveMoneyMarketDenominator" => {
                        assert_eq!(source.active, CanadianYieldConvention::MoneyMarket);
                        assert_eq!(
                            error.kind(),
                            "enclosure.nonpositive_money_market_denominator"
                        );
                        assert!(context.work_used() > 1_088);
                        assert_eq!(
                            error.domain_cause(),
                            Some(EnclosureDomainCause::NonpositiveMoneyMarketDenominator {
                                days_to_true_maturity: row.parse("tau_days")?
                            })
                        );
                        assert_eq!(source.tau_days, row.parse("tau_days")?);
                    }
                    _ => return Err(format!("unknown Canadian predicate {predicate}").into()),
                }
            }
        }
        count += 1;
        if count % 16_384 == 0 {
            checkpoint("Q", count);
        }
    }
    assert_eq!(count, 103_560);
    assert_eq!(predicates.len(), 4);
    checkpoint("Q", count);
    Ok(())
}

fn hand_bond(input: &Flat<'_>) -> TestResult<CanadianBond> {
    assert_eq!(input.string("source_policy")?, "CanonicalIiac13LocalV1");
    let eom = match input.string("eom")? {
        "Ignore" => EndOfMonth::Ignore,
        "Preserve" => EndOfMonth::Preserve,
        _ => return Err("unknown hand EOM".into()),
    };
    let schedule = CanadianCouponSchedule::new(
        input.string("accrual_start")?.parse()?,
        input.string("maturity")?.parse()?,
        named_frequency(input.string("frequency")?)?,
        dates(input, "quasi")?,
        dates(input, "coupons")?,
        eom,
        input.integer("max_dates")?,
    )?;
    Ok(CanadianBond::new(
        input.string("issue")?.parse()?,
        input.string("coupon")?.parse()?,
        schedule,
        input.string("true_maturity")?.parse()?,
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?)
}

fn hands() -> TestResult {
    let table = fixture("hands")?;
    let mut count = 0;
    let mut contexts = BTreeSet::new();
    for row in table.records() {
        if row.text("model")? != "canadian" {
            continue;
        }
        let input = Flat::parse(row.text("input")?)?;
        let expected = Flat::parse(row.text("metadata_expected")?)?;
        let bond = hand_bond(&input)?;
        let plan = CanadianPricingPlan::new(
            &bond,
            input.string("settlement")?.parse()?,
            selection(input.string("selection")?)?,
            input.integer("max_bundles")?,
        )?;
        assert_eq!(plan.bundle_count(), expected.integer("bundles")?);
        assert_eq!(
            period_name(bond.schedule().first_period())?,
            expected.string("first_role")?
        );
        assert_eq!(
            period_name(bond.schedule().last_period())?,
            expected.string("last_role")?
        );
        assert_eq!(
            plan.source_policy(),
            CanadianSourcePolicy::CanonicalIiac13LocalV1
        );
        let quote = bits(input.string("y_bits")?)?;
        let kind = basis(row.text("basis")?)?;
        let reference = Reference::hand(
            row.text("outward_lower_bits")?,
            row.text("outward_upper_bits")?,
            row.text("reference_kind")?,
            row.text("canonical_decimal_point")?,
        )?;
        for p in PRECISIONS {
            let budget = EnclosureBudget::new(p, ORDINARY_WORK)?;
            let output = plan.enclose_price_per_100(f64::from_bits(quote), kind, budget)?;
            let EnclosureOutcome::Finite(report) = output else {
                return Err(format!("{} p{p}: required finite hand", row.id()).into());
            };
            finite_complete(&report, &plan, budget, quote, kind);
            reference.compare(&report, true, &format!("{} p{p}", row.id()))?;
        }
        contexts.insert(row.text("context_id")?);
        count += 1;
    }
    assert_eq!(count, 15);
    assert_eq!(
        contexts,
        BTreeSet::from(["CREG0", "CREGHALF", "CLATE182", "CLATE183", "CMMNEG64"])
    );
    checkpoint("H", count);
    Ok(())
}

fn boundaries(sources: &[CanadianSource]) -> TestResult {
    let table = fixture("boundary")?;
    let mut count = 0;
    for row in table.records() {
        if row.text("model")? != "Canadian" {
            continue;
        }
        assert_eq!(row.text("context_id")?, "B2-REUSE-CANADIAN");
        assert_eq!(row.text("admission_case_id")?, "ENC-ADMISSION-0016090");
        let source = sources
            .get(source_index(row.text("source_plan_id")?)?)
            .ok_or("missing B2 source")?;
        assert_eq!(source.id, "ENC-SOURCE-011422");
        assert_eq!(source.active, CanadianYieldConvention::MoneyMarket);
        let plan = source.plan()?;
        let quote = bits(row.text("yield_bits")?)?;
        let kind = basis(row.text("basis")?)?;
        let reference = Reference::boundary(
            row.text("lower_bits")?,
            row.text("upper_bits")?,
            row.text("point_kind")?,
            row.text("point_coefficient")?,
            row.text("point_scale")?,
            row.text("point_bits")?,
        )?;
        let budget = EnclosureBudget::new(64, ORDINARY_WORK)?;
        let output = plan.enclose_price_per_100(f64::from_bits(quote), kind, budget)?;
        let EnclosureOutcome::Finite(report) = output else {
            return Err(format!("{}: required finite exact-affine MAX case", row.id()).into());
        };
        finite_complete(&report, &plan, budget, quote, kind);
        reference.compare(&report, false, row.id())?;
        count += 1;
    }
    assert_eq!(count, 3);
    checkpoint("B2 finite", count);
    Ok(())
}

fn repeated_and_budget_threshold(sources: &[CanadianSource]) -> TestResult {
    let mut chosen = BTreeSet::new();
    for source in sources {
        let key = match (source.selection, source.active) {
            (CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound), _) => "Compound",
            (CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket), _) => {
                "MoneyMarket"
            }
            (
                CanadianYieldSelection::LastCouponMoneyMarket,
                CanadianYieldConvention::MoneyMarket,
            ) => "LastCouponMoneyMarket",
            _ => continue,
        };
        if !chosen.insert(key) {
            continue;
        }
        let plan = source.plan()?;
        for quote in [0_u64, 0x3fa0_0000_0000_0000] {
            for kind in CanadianPriceBasis::ALL {
                let ample = EnclosureBudget::new(64, ORDINARY_WORK)?;
                let EnclosureOutcome::Finite(first) =
                    plan.enclose_price_per_100(f64::from_bits(quote), kind, ample)?
                else {
                    return Err("representative Canadian price unresolved".into());
                };
                let EnclosureOutcome::Finite(second) =
                    plan.enclose_price_per_100(f64::from_bits(quote), kind, ample)?
                else {
                    return Err("repeat Canadian price unresolved".into());
                };
                finite_complete(&first, &plan, ample, quote, kind);
                assert_eq!(first.context(), second.context());
                assert_eq!(first.encoding(), second.encoding());
                assert_eq!(first.proof_kind(), second.proof_kind());
                assert_eq!(first.settlement_evidence(), second.settlement_evidence());
                let used = first.context().work_used();
                if used > 53_680 {
                    let below = EnclosureBudget::new(64, used - 1)?;
                    let output = plan.enclose_price_per_100(f64::from_bits(quote), kind, below)?;
                    let EnclosureOutcome::Unresolved(report) = output else {
                        return Err("one unit below completion did not stop".into());
                    };
                    check_origin(report.origin(), &plan);
                    check_context(
                        report.context(),
                        below,
                        quote,
                        kind,
                        plan.bundle_count(),
                        true,
                    );
                    assert_eq!(
                        report.reason(),
                        EnclosureUnresolvedReason::WorkLimitAfterAdmission
                    );
                    assert!(report.complete_extended_arithmetic());
                    for max in [used, used + 1] {
                        let exact = EnclosureBudget::new(64, max)?;
                        let EnclosureOutcome::Finite(report) =
                            plan.enclose_price_per_100(f64::from_bits(quote), kind, exact)?
                        else {
                            return Err("observed completion budget failed".into());
                        };
                        assert_eq!(report.context().work_used(), used);
                        assert_eq!(report.settlement_evidence(), first.settlement_evidence());
                    }
                }
            }
        }
    }
    assert_eq!(chosen.len(), 3);
    Ok(())
}

fn ordinary(sources: &[CanadianSource]) -> TestResult {
    let table = fixture("prices-v2")?;
    let mut previous = None;
    let mut plan = None;
    let mut count = 0;
    let mut covered = BTreeSet::new();
    let mut bases = BTreeSet::new();
    let mut sampled_rows = 0;
    for row in table.records() {
        if row.text("model")? != "Canadian" {
            continue;
        }
        let index = source_index(row.text("source_id")?)?;
        let source = sources.get(index).ok_or("missing P source")?;
        assert_eq!(source.id, row.text("source_id")?);
        if previous != Some(index) {
            plan = Some(source.plan()?);
            previous = Some(index);
        }
        let plan = plan.as_ref().ok_or("missing P plan")?;
        let basis_name = row.text("basis")?;
        let kind = basis(basis_name)?;
        bases.insert(basis_name);
        let quote = bits(row.text("yield_bits")?)?;
        let reference = Reference::price(
            row.text("outward_lower_bits")?,
            row.text("outward_upper_bits")?,
            row.text("point_encoding")?,
            row.text("point_value")?,
        )?;
        let keys: Vec<_> = source
            .categories()
            .into_iter()
            .map(|k| format!("{k}:{basis_name}"))
            .collect();
        let mut precisions = vec![64];
        // Orthogonal, predeclared representatives cover every geometry class,
        // f, active convention, selection, EOM, true-T and A/AI relationship
        // with EACH basis. The full ordinary Cartesian partition still runs p64.
        if quote == 0x3fa0_0000_0000_0000
            && source.bond.coupon_per_100().coefficient() != 0
            && keys.iter().any(|key| !covered.contains(key))
        {
            covered.extend(keys);
            precisions.extend(PRECISIONS.into_iter().filter(|p| *p != 64));
            sampled_rows += 1;
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
            finite_complete(&report, plan, budget, quote, kind);
            reference.compare(&report, source.useful, &format!("{} p{p}", row.id()))?;
        }
        count += 1;
        if count % 4096 == 0 {
            checkpoint("P", count);
        }
    }
    assert_eq!(count, 124_560);
    assert_eq!(
        bases,
        BTreeSet::from(["Clean", "ModelDirty", "SettlementDirty"])
    );
    for (prefix, wanted) in [
        ("geometry:", 42),
        ("frequency:", 9),
        ("convention:", 6),
        ("selection:", 9),
        ("eom:", 6),
        ("terminal:", 6),
        ("AI_equals_A:", 6),
    ] {
        assert_eq!(
            covered.iter().filter(|key| key.starts_with(prefix)).count(),
            wanted,
            "{prefix}"
        );
    }
    assert!(sampled_rows >= 42);
    checkpoint("P", count);
    checkpoint("all-precision representative rows", sampled_rows);
    Ok(())
}

#[test]
fn actual_canadian_enclosures_consume_frozen_financial_partitions() -> TestResult {
    let sources = load_sources()?;
    admission(&sources)?;
    hands()?;
    boundaries(&sources)?;
    repeated_and_budget_threshold(&sources)?;
    ordinary(&sources)?;
    Ok(())
}
