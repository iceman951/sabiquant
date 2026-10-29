//! Actual retained-source conventional Gilt mathematical price producer.

use super::arithmetic::{arena, fold, scalar, source};
use super::engine::{
    self, Call, Finite, Stop, arena_result, fold_result, kernel_result, source_result,
};
use super::*;
use crate::bonds::GiltFirstPeriod;
use crate::bonds::gilt::{ExactSource, Scalar, SourceRational};

const SMALL_BLOCK: u64 = 16 * 5 + 32;

impl<'s> GiltPricingPlan<'s> {
    /// Proves the retained conventional Gilt mathematical price per100.
    ///
    /// Uses exact selected source dividends, bundled par, original quasi
    /// coordinates and raw signed source accrued interest. No calendar query,
    /// observed binary64 valuation or rounded accrued getter is a premise.
    /// The controlled evidence borrows this immutable originating plan.
    ///
    /// # Errors
    /// Returns [`EnclosureError`] for nonfinite yield, the global `yield > -2`
    /// domain, incomplete admission, allocation failure or an internal capacity
    /// invariant. Fully admitted work stops and unavailable finite endpoints
    /// return [`EnclosureOutcome::Unresolved`] without usable partial evidence.
    pub fn enclose_price_per_100<'p>(
        &'p self,
        yield_rate: f64,
        basis: BondPriceKind,
        budget: EnclosureBudget,
    ) -> EnclosureResult<'p, 's> {
        let mut call = Call::new(yield_rate, EnclosureBasis::Gilt(basis), budget);
        let result = run(&mut call, self, yield_rate, basis);
        call.finish(EnclosureOrigin::gilt(self), result)
    }
}

fn run(
    call: &mut Call,
    plan: &GiltPricingPlan<'_>,
    yield_rate: f64,
    basis: BondPriceKind,
) -> engine::Result<Finite> {
    crate::numeric::finite("yield_rate", yield_rate).map_err(Stop::NonFinite)?;
    if yield_rate <= -2.0 {
        return Err(Stop::Domain(EnclosureDomainCause::YieldNotAboveFloor {
            payments_per_year: 2,
        }));
    }
    call.context.stage = EnclosureStage::Initialization;
    call.charge(8_560)?;
    let mut frame = scalar::ZERO_FRAME;
    for bundle in &plan.bundles {
        call.context.stage = EnclosureStage::FlowAdmission;
        call.context.next_original_index = Some(bundle.index);
        call.charge(16)?;
        // Valid prepared geometry and the global floor establish each positive
        // factor domain, including zero-valued retained source dividends.
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

fn power10(call: &mut Call, scale: u8) -> engine::Result<Scalar> {
    if scale > 38 {
        return Err(Stop::Invariant(Operation::Source));
    }
    call.charge(64 + u64::from(scale) * SMALL_BLOCK)?;
    Scalar::pow10(u16::from(scale)).ok_or(Stop::Invariant(Operation::Source))
}

fn small(call: &mut Call, value: u128) -> engine::Result<Scalar> {
    call.charge(SMALL_BLOCK)?;
    Ok(Scalar::from_u128(value))
}

fn multiply_small(call: &mut Call, value: Scalar, multiplier: u32) -> engine::Result<Scalar> {
    call.charge(SMALL_BLOCK)?;
    value
        .checked_mul_small(multiplier)
        .ok_or(Stop::Invariant(Operation::Source))
}

/// Interpret only the retained AI recipe, using bounded prepaid five-word work.
fn accrued(call: &mut Call, plan: &GiltPricingPlan<'_>) -> engine::Result<SourceRational> {
    call.charge(4_096)?;
    let coupon = plan.gilt().coupon_per_100();
    let (numerator, denominator, negative) = match plan.accrued_source {
        ExactSource::Unrounded {
            numerator,
            denominator,
        } => {
            let coefficient = small(call, coupon.coefficient().unsigned_abs())?;
            let n = multiply_small(call, coefficient, numerator.unsigned_abs())?;
            let decimal = power10(call, coupon.scale())?;
            let d = multiply_small(call, decimal, denominator)?;
            (n, d, (coupon.coefficient() < 0) != (numerator < 0))
        }
        ExactSource::Published { micro } => {
            call.charge(SMALL_BLOCK)?;
            (micro, power10(call, 6)?, false)
        }
    };
    call.charge(SMALL_BLOCK)?;
    Ok(SourceRational {
        numerator,
        denominator,
        negative: negative && !numerator.is_zero(),
    })
}

fn exact(
    call: &mut Call,
    plan: &GiltPricingPlan<'_>,
    basis: BondPriceKind,
    frame: &mut scalar::Frame,
) -> engine::Result<Finite> {
    call.context.stage = EnclosureStage::Source;
    // Fixed semantic roles reproduce the sealed source graph: regular, odd
    // first publication when applicable, and the plan's original AI recipe.
    // Equal numeric denominators are not otherwise deduplicated.
    call.charge(4_096)?;
    let decimal = power10(call, plan.gilt().coupon_per_100().scale())?;
    let regular = multiply_small(call, decimal, 2)?;
    call.charge(SMALL_BLOCK)?;
    let odd = plan.gilt().first_period() != GiltFirstPeriod::Regular;
    let first = if odd {
        call.charge(4_096)?;
        small(call, 1_000_000)?
    } else {
        call.charge(SMALL_BLOCK)?;
        regular
    };
    let ai = accrued(call, plan)?;
    call.charge(4_096)?;
    let roles = if odd { 3 } else { 2 };
    let ai_role = usize::from(roles - 1);
    let terms = u64::try_from(plan.bundles.len())
        .ok()
        .and_then(|n| n.checked_add(u64::from(basis == BondPriceKind::Clean)))
        .ok_or(Stop::Invariant(Operation::Progress))?;
    let mut sum = fold_result(fold::Session::named(frame, &mut call.work, roles, terms))?;
    sum = fold_result(sum.role(regular.limbs()))?;
    if odd {
        sum = fold_result(sum.role(first.limbs()))?;
    }
    sum = fold_result(sum.role(ai.denominator.limbs()))?;
    for bundle in &plan.bundles {
        call.context.stage = EnclosureStage::Source;
        call.context.next_original_index = Some(bundle.index);
        sum = fold_result(sum.debit(4_096))?;
        let role = usize::from(odd && bundle.index == plan.gilt().first_dividend_quasi_index());
        call.context.stage = EnclosureStage::Reduction;
        sum = fold_result(sum.rational(
            Some(role),
            bundle.amount.numerator.limbs(),
            bundle.amount.denominator.limbs(),
            bundle.amount.negative,
        ))?;
        sum = fold_result(sum.debit(8))?;
        call.context.reduction_completed_flows += 1;
    }
    call.context.next_original_index = None;
    call.context.stage = EnclosureStage::BasisAndUnit;
    sum = fold_result(sum.debit(64))?;
    if basis == BondPriceKind::Clean {
        sum = fold_result(sum.debit(4_096))?;
        sum = fold_result(sum.rational(
            Some(ai_role),
            ai.numerator.limbs(),
            ai.denominator.limbs(),
            !ai.negative,
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

fn normalize(
    call: &mut Call,
    slots: &mut arena::SlotArena,
    frame: &mut scalar::Frame,
    output: usize,
    seed: source::Seed,
) -> engine::Result<()> {
    arena_result(
        slots.normalize_staged(
            output,
            call.p(),
            seed.negative,
            seed.exponent < 0,
            frame,
            &mut call.work,
        ),
        Operation::Normalize,
    )
}

fn directed(
    call: &mut Call,
    plan: &GiltPricingPlan<'_>,
    yield_bits: u64,
    basis: BondPriceKind,
    frame: &mut scalar::Frame,
) -> engine::Result<Finite> {
    let p = call.p();
    let mut slots = call.allocate()?;
    arena_result(
        slots.initialize_unit(p, frame, &mut call.work),
        Operation::Initialization,
    )?;
    call.context.stage = EnclosureStage::Source;
    call.charge(4_096)?;
    source_result(source::ratio_affine(
        yield_bits,
        1,
        2,
        frame,
        &mut call.work,
    ))?;
    call.context.stage = EnclosureStage::Kernel;
    kernel_result(
        arena::kernel::logarithm(p, &mut slots, frame, 1, &mut call.work),
        Operation::Log,
    )?;
    let mut current = 0;
    let mut next = 6;
    for bundle in &plan.bundles {
        call.context.next_original_index = Some(bundle.index);
        call.context.stage = EnclosureStage::Source;
        call.charge(4_096)?;
        let coordinate = source_result(source::source(
            &[u64::from(bundle.l)],
            &[u64::from(plan.quasi_period_days())],
            true,
            frame,
            &mut call.work,
        ))?;
        normalize(call, &mut slots, frame, 3, coordinate)?;
        call.context.stage = EnclosureStage::Kernel;
        arena_result(
            slots.multiply(p, 1, 3, 4, frame, &mut call.work),
            Operation::Multiply,
        )?;
        kernel_result(
            arena::kernel::exponential_interval(p, &mut slots, frame, 4, 5, &mut call.work),
            Operation::Exp,
        )?;
        call.context.stage = EnclosureStage::Source;
        call.charge(4_096)?;
        let amount = source_result(source::source(
            bundle.amount.numerator.limbs(),
            bundle.amount.denominator.limbs(),
            bundle.amount.negative,
            frame,
            &mut call.work,
        ))?;
        normalize(call, &mut slots, frame, 3, amount)?;
        call.context.stage = EnclosureStage::Reduction;
        arena_result(
            slots.multiply(p, 3, 5, 4, frame, &mut call.work),
            Operation::Multiply,
        )?;
        arena_result(
            slots.add(p, current, 4, next, frame, &mut call.work),
            Operation::Add,
        )?;
        call.charge(8)?;
        call.context.reduction_completed_flows += 1;
        std::mem::swap(&mut current, &mut next);
    }
    call.context.next_original_index = None;
    call.context.stage = EnclosureStage::BasisAndUnit;
    call.charge(64)?;
    if basis == BondPriceKind::Clean {
        let ai = accrued(call, plan)?;
        let seed = source_result(source::source(
            ai.numerator.limbs(),
            ai.denominator.limbs(),
            !ai.negative,
            frame,
            &mut call.work,
        ))?;
        normalize(call, &mut slots, frame, 3, seed)?;
        arena_result(
            slots.add(p, current, 3, next, frame, &mut call.work),
            Operation::Add,
        )?;
        std::mem::swap(&mut current, &mut next);
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

#[cfg(test)]
#[path = "gilt_controls.rs"]
mod controls;
