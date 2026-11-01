//! Actual retained-source Canadian Compound and MoneyMarket price producer.

use super::arithmetic::{arena, fold, scalar, source};
use super::engine::{
    self, Call, Finite, Stop, arena_result, fold_result, kernel_result, source_result,
};
use super::*;
use crate::bonds::canadian::source::{Coordinate, Source};
use crate::bonds::canadian::wide;
use crate::bonds::{CanadianFrequency, CanadianYieldConvention};
use std::cmp::Ordering;

const SMALL_BLOCK: u64 = 16 * 5 + 32;
const ONE: [u64; 5] = [1, 0, 0, 0, 0];

impl<'s> CanadianPricingPlan<'s> {
    /// Proves the retained Canadian source price per100 in the selected basis.
    ///
    /// Uses the prepared active equation, exact selected coupons, original
    /// quasi coordinates, true-terminal day counts and separate exact A/AI.
    /// Exact admission completes before pricing; no observed valuation, date
    /// query or rounded source getter is a premise. ModelDirty remains distinct
    /// from the settlement all-in basis. Evidence retains this plan's borrow.
    ///
    /// # Errors
    /// Returns [`EnclosureError`] for nonfinite yield, exact mathematical domain
    /// rejection, incomplete admission, allocation or a capacity invariant.
    /// After complete admission, work stops and unavailable finite endpoints
    /// return [`EnclosureOutcome::Unresolved`] without partial price evidence.
    pub fn enclose_price_per_100<'p>(
        &'p self,
        yield_rate: f64,
        basis: CanadianPriceBasis,
        budget: EnclosureBudget,
    ) -> EnclosureResult<'p, 's> {
        let mut call = Call::new(yield_rate, EnclosureBasis::Canadian(basis), budget);
        let result = run(&mut call, self, yield_rate, basis);
        call.finish(EnclosureOrigin::canadian(self), result)
    }
}

fn run(
    call: &mut Call,
    plan: &CanadianPricingPlan<'_>,
    yield_rate: f64,
    basis: CanadianPriceBasis,
) -> engine::Result<Finite> {
    crate::numeric::finite("yield_rate", yield_rate).map_err(Stop::NonFinite)?;
    let convention = plan.yield_convention();
    let frequency = plan.bond().schedule().frequency().payments_per_year();
    if convention == CanadianYieldConvention::Compound && yield_rate <= -f64::from(frequency) {
        return Err(Stop::Domain(EnclosureDomainCause::YieldNotAboveFloor {
            payments_per_year: u16::try_from(frequency)
                .map_err(|_| Stop::Invariant(Operation::Source))?,
        }));
    }
    call.context.stage = EnclosureStage::Initialization;
    call.charge(8_560)?;
    let mut frame = scalar::ZERO_FRAME;
    if convention == CanadianYieldConvention::MoneyMarket {
        call.context.stage = EnclosureStage::QuoteAdmission;
        call.charge(4_096)?;
        let days = plan.cached_days_to_true_maturity();
        let seed = source_result(source::ratio_affine(
            yield_rate.to_bits(),
            days,
            365,
            &mut frame,
            &mut call.work,
        ))?;
        call.charge(64)?;
        if seed.negative || frame.raw0.n.iter().all(|word| *word == 0) {
            return Err(Stop::Domain(
                EnclosureDomainCause::NonpositiveMoneyMarketDenominator {
                    days_to_true_maturity: days,
                },
            ));
        }
    }
    for bundle in &plan.bundles {
        call.context.stage = EnclosureStage::FlowAdmission;
        call.context.next_original_index = Some(bundle.index);
        call.charge(16)?;
        // Prepared geometry gives 0<=rho<=tau; positive exact d proves every
        // MM numerator positive. Compound has the global positive-base proof.
        call.context.admission_checked_flows += 1;
    }
    call.context.admission_complete = true;
    call.context.next_original_index = None;
    if yield_rate.to_bits() << 1 == 0 {
        exact(call, plan, basis, &mut frame)
    } else {
        directed(call, plan, yield_rate.to_bits(), basis, &mut frame)
    }
}

struct Roles<'a> {
    denominators: [&'a [u64; 5]; 5],
    count: usize,
    last: usize,
    a: usize,
    ai: usize,
    coupons: usize,
}

impl<'a> Roles<'a> {
    fn push(&mut self, denominator: &'a [u64; 5]) -> engine::Result<usize> {
        let index = self.count;
        *self
            .denominators
            .get_mut(index)
            .ok_or(Stop::Invariant(Operation::Source))? = denominator;
        self.count = index
            .checked_add(1)
            .ok_or(Stop::Invariant(Operation::Progress))?;
        Ok(index)
    }
}

fn roles<'a>(call: &mut Call, plan: &'a CanadianPricingPlan<'_>) -> engine::Result<Roles<'a>> {
    call.charge(4_096)?;
    let coupons = plan.bond().schedule().coupon_count();
    let monthly = plan.bond().schedule().frequency() == CanadianFrequency::Monthly;
    let mut result = Roles {
        denominators: [&ONE; 5],
        count: 0,
        last: 1,
        a: 0,
        ai: 0,
        coupons,
    };
    call.charge(4_096)?;
    result.push(&plan.bond().regular.denominator)?;
    call.charge(4_096)?;
    let first = plan
        .bundles
        .first()
        .filter(|b| b.index == 0)
        .map_or(&ONE, |b| &b.amount.denominator);
    result.push(first)?;
    if coupons != 1 {
        call.charge(4_096)?;
        let last = plan
            .bundles
            .last()
            .ok_or(Stop::Invariant(Operation::Source))?;
        if last.index.checked_add(1) != Some(coupons) {
            return Err(Stop::Invariant(Operation::Source));
        }
        result.last = result.push(&last.amount.denominator)?;
    }
    call.charge(4_096)?;
    result.a = result.push(&plan.valuation_ai.denominator)?;
    result.ai = if monthly {
        result.a
    } else {
        call.charge(4_096)?;
        result.push(&plan.settlement_ai.denominator)?
    };
    Ok(result)
}

fn bundle_role(index: usize, coupons: usize, last: usize) -> usize {
    if index == 0 {
        1
    } else if index.checked_add(1) == Some(coupons) {
        last
    } else {
        0
    }
}

fn shifts(convention: CanadianYieldConvention, basis: CanadianPriceBasis) -> (bool, bool, bool) {
    match (convention, basis) {
        (_, CanadianPriceBasis::ModelDirty) => (false, false, false),
        (CanadianYieldConvention::Compound, CanadianPriceBasis::Clean) => (true, false, false),
        (CanadianYieldConvention::Compound, CanadianPriceBasis::SettlementDirty) => {
            (true, true, false)
        }
        (CanadianYieldConvention::MoneyMarket, CanadianPriceBasis::Clean) => (false, true, true),
        (CanadianYieldConvention::MoneyMarket, CanadianPriceBasis::SettlementDirty) => {
            (false, false, false)
        }
    }
}

fn exact(
    call: &mut Call,
    plan: &CanadianPricingPlan<'_>,
    basis: CanadianPriceBasis,
    frame: &mut scalar::Frame,
) -> engine::Result<Finite> {
    call.context.stage = EnclosureStage::Source;
    let roles = roles(call, plan)?;
    let (subtract_a, include_ai, subtract_ai) = shifts(plan.yield_convention(), basis);
    let extra = 1 + u64::from(subtract_a) + u64::from(include_ai);
    let terms = u64::try_from(plan.bundles.len())
        .ok()
        .and_then(|n| n.checked_add(extra))
        .ok_or(Stop::Invariant(Operation::Progress))?;
    let role_count = u8::try_from(roles.count).map_err(|_| Stop::Invariant(Operation::Progress))?;
    let mut sum = fold_result(fold::Session::named(
        frame,
        &mut call.work,
        role_count,
        terms,
    ))?;
    for denominator in roles.denominators.iter().take(roles.count) {
        sum = fold_result(sum.role(*denominator))?;
    }
    for bundle in &plan.bundles {
        call.context.stage = EnclosureStage::Source;
        call.context.next_original_index = Some(bundle.index);
        sum = fold_result(sum.debit(4_096))?;
        sum = fold_result(sum.debit(SMALL_BLOCK))?;
        let mut numerator = [0; 5];
        if !wide::copy_into(&bundle.amount.numerator, &mut numerator) {
            return Err(Stop::Invariant(Operation::Source));
        }
        let terminal = bundle.index.checked_add(1) == Some(roles.coupons);
        if terminal {
            sum = fold_result(sum.debit(SMALL_BLOCK))?;
            let mut par = [0; 5];
            if !wide::copy_into(&bundle.amount.denominator, &mut par) {
                return Err(Stop::Invariant(Operation::Source));
            }
            sum = fold_result(sum.debit(SMALL_BLOCK))?;
            if !wide::mul_small_assign(&mut par, 100) {
                return Err(Stop::Invariant(Operation::Source));
            }
            sum = fold_result(sum.debit(SMALL_BLOCK))?;
            if wide::cmp(&numerator, &par) == Ordering::Less {
                return Err(Stop::Invariant(Operation::Source));
            }
            sum = fold_result(sum.debit(SMALL_BLOCK))?;
            if !wide::sub_assign(&mut numerator, &par) {
                return Err(Stop::Invariant(Operation::Source));
            }
        }
        let role = bundle_role(bundle.index, roles.coupons, roles.last);
        sum = fold_result(sum.debit(SMALL_BLOCK))?;
        if &bundle.amount.denominator != roles.denominators[role] {
            return Err(Stop::Invariant(Operation::Source));
        }
        call.context.stage = EnclosureStage::Reduction;
        sum = fold_result(sum.rational(Some(role), &numerator, &bundle.amount.denominator, false))?;
        if terminal {
            call.context.stage = EnclosureStage::Source;
            sum = fold_result(sum.debit(4_096))?;
            call.context.stage = EnclosureStage::Reduction;
            sum = fold_result(sum.rational(None, &[100], &[1], false))?;
        }
        sum = fold_result(sum.debit(8))?;
        call.context.reduction_completed_flows += 1;
    }
    call.context.next_original_index = None;
    call.context.stage = EnclosureStage::BasisAndUnit;
    sum = fold_result(sum.debit(64))?;
    if subtract_a {
        sum = fold_result(sum.debit(4_096))?;
        sum = fold_result(sum.rational(
            Some(roles.a),
            &plan.valuation_ai.numerator,
            &plan.valuation_ai.denominator,
            true,
        ))?;
    }
    if include_ai {
        sum = fold_result(sum.debit(4_096))?;
        sum = fold_result(sum.rational(
            Some(roles.ai),
            &plan.settlement_ai.numerator,
            &plan.settlement_ai.denominator,
            subtract_ai,
        ))?;
    }
    let staged = fold_result(sum.finish_named())?;
    call.complete = true;
    call.context.stage = EnclosureStage::Export;
    call.charge(256)?;
    let seed = source::Seed {
        negative: staged.negative,
        exponent: staged.exponent,
    };
    if let Some(finite) = call.encode_raw(frame, seed, EnclosureProofKind::ExactRationalIdentity)? {
        return Ok(finite);
    }
    let mut slots = call.allocate()?;
    call.context.stage = EnclosureStage::Export;
    normalize(call, &mut slots, frame, 0, seed)?;
    call.export(&slots, frame, 0, EnclosureProofKind::ExactRationalIdentity)
}

fn coupon(call: &mut Call, amount: &Source, terminal: bool) -> engine::Result<[u64; 5]> {
    call.charge(SMALL_BLOCK)?;
    let mut numerator = [0; 5];
    if !wide::copy_into(&amount.numerator, &mut numerator) {
        return Err(Stop::Invariant(Operation::Source));
    }
    if terminal {
        call.charge(SMALL_BLOCK)?;
        let mut par = [0; 5];
        if !wide::copy_into(&amount.denominator, &mut par) {
            return Err(Stop::Invariant(Operation::Source));
        }
        call.charge(SMALL_BLOCK)?;
        if !wide::mul_small_assign(&mut par, 100) {
            return Err(Stop::Invariant(Operation::Source));
        }
        call.charge(SMALL_BLOCK)?;
        if wide::cmp(&numerator, &par) == Ordering::Less {
            return Err(Stop::Invariant(Operation::Source));
        }
        call.charge(SMALL_BLOCK)?;
        if !wide::sub_assign(&mut numerator, &par) {
            return Err(Stop::Invariant(Operation::Source));
        }
    }
    Ok(numerator)
}

fn normalize(
    call: &mut Call,
    slots: &mut arena::SlotArena,
    frame: &mut scalar::Frame,
    out: usize,
    seed: source::Seed,
) -> engine::Result<()> {
    arena_result(
        slots.normalize_staged(
            out,
            call.p(),
            seed.negative,
            seed.exponent < 0,
            frame,
            &mut call.work,
        ),
        Operation::Normalize,
    )
}

fn coefficient(
    call: &mut Call,
    slots: &mut arena::SlotArena,
    frame: &mut scalar::Frame,
    numerator: &[u64],
    denominator: &[u64],
    negative: bool,
) -> engine::Result<()> {
    let seed = source_result(source::source(
        numerator,
        denominator,
        negative,
        frame,
        &mut call.work,
    ))?;
    normalize(call, slots, frame, 3, seed)
}

fn add_term(
    call: &mut Call,
    slots: &mut arena::SlotArena,
    frame: &mut scalar::Frame,
    current: &mut usize,
    next: &mut usize,
) -> engine::Result<()> {
    let p = call.p();
    arena_result(
        slots.multiply(p, 3, 5, 4, frame, &mut call.work),
        Operation::Multiply,
    )?;
    arena_result(
        slots.add(p, *current, 4, *next, frame, &mut call.work),
        Operation::Add,
    )?;
    std::mem::swap(current, next);
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "one borrowed Frame and explicit retained coordinates avoid copied numerical storage"
)]
fn factor(
    call: &mut Call,
    slots: &mut arena::SlotArena,
    frame: &mut scalar::Frame,
    plan: &CanadianPricingPlan<'_>,
    yield_bits: u64,
    lambda: Coordinate,
    rho_days: u32,
    log_ready: &mut bool,
) -> engine::Result<()> {
    let p = call.p();
    if plan.yield_convention() == CanadianYieldConvention::MoneyMarket {
        let seed = source_result(source::ratio_affine(
            yield_bits,
            rho_days,
            365,
            frame,
            &mut call.work,
        ))?;
        normalize(call, slots, frame, 3, seed)?;
        call.context.stage = EnclosureStage::Kernel;
        arena_result(
            slots.multiply(p, 3, 7, 5, frame, &mut call.work),
            Operation::Multiply,
        )?;
    } else {
        if !*log_ready {
            let index = call.context.next_original_index;
            call.context.next_original_index = None;
            call.context.stage = EnclosureStage::Source;
            call.charge(4_096)?;
            source_result(source::ratio_affine(
                yield_bits,
                1,
                plan.bond().schedule().frequency().payments_per_year(),
                frame,
                &mut call.work,
            ))?;
            call.context.stage = EnclosureStage::Kernel;
            kernel_result(
                arena::kernel::logarithm(p, slots, frame, 1, &mut call.work),
                Operation::Log,
            )?;
            *log_ready = true;
            call.context.next_original_index = index;
        }
        call.context.stage = EnclosureStage::Source;
        let seed = source_result(source::source(
            &[lambda.numerator],
            &[u64::from(lambda.denominator)],
            true,
            frame,
            &mut call.work,
        ))?;
        normalize(call, slots, frame, 3, seed)?;
        call.context.stage = EnclosureStage::Kernel;
        arena_result(
            slots.multiply(p, 1, 3, 4, frame, &mut call.work),
            Operation::Multiply,
        )?;
        kernel_result(
            arena::kernel::exponential_interval(p, slots, frame, 4, 5, &mut call.work),
            Operation::Exp,
        )?;
    }
    Ok(())
}

fn directed(
    call: &mut Call,
    plan: &CanadianPricingPlan<'_>,
    yield_bits: u64,
    basis: CanadianPriceBasis,
    frame: &mut scalar::Frame,
) -> engine::Result<Finite> {
    let p = call.p();
    let convention = plan.yield_convention();
    let mut slots = call.allocate()?;
    if convention == CanadianYieldConvention::MoneyMarket {
        // Global d survived metadata-only admission. Preserve it in the arena
        // before initialize_unit releases/clobbers the exact raw staging fields.
        normalize(
            call,
            &mut slots,
            frame,
            2,
            source::Seed {
                negative: false,
                exponent: 0,
            },
        )?;
    }
    arena_result(
        slots.initialize_unit(p, frame, &mut call.work),
        Operation::Initialization,
    )?;
    if convention == CanadianYieldConvention::MoneyMarket {
        call.context.stage = EnclosureStage::Kernel;
        arena_result(
            slots.reciprocal(p, 2, 7, frame, &mut call.work),
            Operation::Divide,
        )?;
    }
    call.context.stage = EnclosureStage::Source;
    call.charge(4_096)?;
    let coupons = plan.bond().schedule().coupon_count();
    let mut current = 0;
    let mut next = 6;
    let mut log_ready = false;
    for bundle in &plan.bundles {
        call.context.next_original_index = Some(bundle.index);
        call.context.stage = EnclosureStage::Source;
        call.charge(4_096)?;
        let terminal = bundle.index.checked_add(1) == Some(coupons);
        let numerator = coupon(call, &bundle.amount, terminal)?;
        call.charge(SMALL_BLOCK)?;
        let zero_coupon = numerator.iter().all(|word| *word == 0);
        if !zero_coupon || terminal {
            call.charge(4_096)?;
            factor(
                call,
                &mut slots,
                frame,
                plan,
                yield_bits,
                bundle.lambda,
                bundle.days_payment_to_true,
                &mut log_ready,
            )?;
            if !zero_coupon {
                call.context.stage = EnclosureStage::Source;
                coefficient(
                    call,
                    &mut slots,
                    frame,
                    &numerator,
                    &bundle.amount.denominator,
                    false,
                )?;
                call.context.stage = EnclosureStage::Reduction;
                add_term(call, &mut slots, frame, &mut current, &mut next)?;
            }
            if terminal {
                call.context.stage = EnclosureStage::Source;
                call.charge(4_096)?;
                coefficient(call, &mut slots, frame, &[100], &[1], false)?;
                call.context.stage = EnclosureStage::Reduction;
                add_term(call, &mut slots, frame, &mut current, &mut next)?;
            }
        }
        call.context.stage = EnclosureStage::Reduction;
        call.charge(8)?;
        call.context.reduction_completed_flows += 1;
    }
    call.context.next_original_index = None;
    call.context.stage = EnclosureStage::BasisAndUnit;
    call.charge(64)?;
    let (subtract_a, include_ai, subtract_ai) = shifts(convention, basis);
    for (include, amount, negative) in [
        (subtract_a, &plan.valuation_ai, true),
        (include_ai, &plan.settlement_ai, subtract_ai),
    ] {
        if include {
            call.charge(4_096)?;
            coefficient(
                call,
                &mut slots,
                frame,
                &amount.numerator,
                &amount.denominator,
                negative,
            )?;
            arena_result(
                slots.add(p, current, 3, next, frame, &mut call.work),
                Operation::Add,
            )?;
            std::mem::swap(&mut current, &mut next);
        }
    }
    call.complete = true;
    call.context.stage = EnclosureStage::Export;
    call.charge(256)?;
    let (singleton, seed) = arena_result(
        slots.stage_final_point(p, current, frame, &mut call.work),
        Operation::PointEncoding,
    )?;
    let proof = if singleton {
        EnclosureProofKind::IntervalCollapse
    } else {
        EnclosureProofKind::DirectedArithmetic
    };
    if let Some(seed) = seed
        && let Some(finite) = call.encode_raw(frame, seed, proof)?
    {
        return Ok(finite);
    }
    call.export(&slots, frame, current, proof)
}
