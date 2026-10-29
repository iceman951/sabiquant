//! Actual retained-source fixed-principal Treasury mathematical price producer.

use super::arithmetic::{arena, fold, scalar, source};
use super::engine::{
    self, Call, Finite, Stop, arena_result, fold_result, kernel_result, source_result,
};
use super::*;
use crate::bonds::TreasuryFirstPeriod;
use crate::bonds::treasury::Scalar;

const SMALL_BLOCK: u64 = 16 * 5 + 32;

impl<'s> TreasuryPricingPlan<'s> {
    /// Proves the retained fixed-principal Treasury mathematical price per100.
    ///
    /// Uses exact raw analytical coupons, bundled par, original integer j and
    /// exact q/b coordinates, with the selected published source AI. No
    /// calendar query, observed valuation or rounded getter is a premise.
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
        let mut call = Call::new(yield_rate, EnclosureBasis::Treasury(basis), budget);
        let result = run(&mut call, self, yield_rate, basis);
        call.finish(EnclosureOrigin::treasury(self), result)
    }
}

fn run(
    call: &mut Call,
    plan: &TreasuryPricingPlan<'_>,
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

fn multiply_small(call: &mut Call, value: Scalar, multiplier: u32) -> engine::Result<Scalar> {
    call.charge(SMALL_BLOCK)?;
    value
        .checked_mul_small(multiplier)
        .ok_or(Stop::Invariant(Operation::Source))
}

fn million(call: &mut Call) -> engine::Result<Scalar> {
    call.charge(SMALL_BLOCK)?;
    Ok(Scalar::from_u128(1_000_000))
}

fn bundle_role(plan: &TreasuryPricingPlan<'_>, index: usize, odd: bool) -> usize {
    usize::from(odd && index == plan.treasury().first_coupon_quasi_index())
}

fn exact(
    call: &mut Call,
    plan: &TreasuryPricingPlan<'_>,
    basis: BondPriceKind,
    frame: &mut scalar::Frame,
) -> engine::Result<Finite> {
    call.context.stage = EnclosureStage::Source;
    // Same sealed semantic source graph: regular, nonregular raw first (or
    // structural regular alias), and selected published AI. Raw AI is unused.
    call.charge(4_096)?;
    let decimal = power10(call, plan.treasury().coupon_per_100().scale())?;
    let regular = multiply_small(call, decimal, 2)?;
    call.charge(SMALL_BLOCK)?;
    let odd = plan.treasury().first_period() != TreasuryFirstPeriod::Regular;
    let first = if odd {
        call.charge(4_096)?;
        multiply_small(call, regular, plan.treasury().s0)?
    } else {
        call.charge(SMALL_BLOCK)?;
        regular
    };
    call.charge(4_096)?;
    let ai_denominator = million(call)?;
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
    sum = fold_result(sum.role(ai_denominator.limbs()))?;
    for bundle in &plan.bundles {
        call.context.stage = EnclosureStage::Source;
        call.context.next_original_index = Some(bundle.index);
        sum = fold_result(sum.debit(4_096))?;
        let role = bundle_role(plan, bundle.index, odd);
        call.context.stage = EnclosureStage::Reduction;
        sum = fold_result(sum.rational(
            Some(role),
            bundle.amount.numerator.limbs(),
            bundle.amount.denominator.limbs(),
            false,
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
            plan.accrued_source.published_count.limbs(),
            ai_denominator.limbs(),
            true,
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
    plan: &TreasuryPricingPlan<'_>,
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
    let denominator = plan
        .quasi_period_days()
        .checked_mul(2)
        .ok_or(Stop::Invariant(Operation::Source))?;
    let q = source_result(source::ratio_affine(
        yield_bits,
        plan.remaining_days(),
        denominator,
        frame,
        &mut call.work,
    ))?;
    normalize(call, &mut slots, frame, 2, q)?;
    call.context.stage = EnclosureStage::Kernel;
    // Prepared 0<r<=s and the global floor prove exact q>0. No native q gate.
    arena_result(
        slots.reciprocal(p, 2, 7, frame, &mut call.work),
        Operation::Divide,
    )?;
    let mut log_ready = false;
    let mut current = 0;
    let mut next = 6;
    for bundle in &plan.bundles {
        call.context.next_original_index = Some(bundle.index);
        call.context.stage = EnclosureStage::Source;
        call.charge(4_096)?;
        if bundle.j == 0 {
            // The actual original integer coordinate is zero: b^0=1 exactly.
            arena_result(
                slots.copy_interval(p, arena::UNIT, 5, &mut call.work),
                Operation::Normalize,
            )?;
        } else {
            if !log_ready {
                call.context.next_original_index = None;
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
                log_ready = true;
                call.context.next_original_index = Some(bundle.index);
            }
            call.context.stage = EnclosureStage::Source;
            let coordinate = source_result(source::source(
                &[u64::from(bundle.j)],
                &[1],
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
        }
        call.context.stage = EnclosureStage::Kernel;
        arena_result(
            slots.multiply(p, 5, 7, 2, frame, &mut call.work),
            Operation::Multiply,
        )?;
        call.context.stage = EnclosureStage::Source;
        call.charge(4_096)?;
        let amount = source_result(source::source(
            bundle.amount.numerator.limbs(),
            bundle.amount.denominator.limbs(),
            false,
            frame,
            &mut call.work,
        ))?;
        normalize(call, &mut slots, frame, 3, amount)?;
        call.context.stage = EnclosureStage::Reduction;
        arena_result(
            slots.multiply(p, 3, 2, 4, frame, &mut call.work),
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
        call.charge(4_096)?;
        let ai = source_result(source::source(
            plan.accrued_source.published_count.limbs(),
            &[1_000_000],
            true,
            frame,
            &mut call.work,
        ))?;
        normalize(call, &mut slots, frame, 3, ai)?;
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
#[path = "treasury_controls.rs"]
mod controls;
