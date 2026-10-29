//! Actual prepared-bit Generic mathematical price producer.

use super::arithmetic::{arena, fold, scalar, source};
use super::engine::{
    self, Call, Finite, Stop, arena_result, fold_result, kernel_result, source_result,
};
use super::*;
use crate::rates::{ActiveBranch, Compounding};

impl<'s> BondPricingPlan<'s> {
    /// Proves a mathematical price per100 from this plan's exact retained bits.
    ///
    /// Uses the prepared amounts, direct horizons, accrued amount and face;
    /// exact admission precedes bounded arithmetic. No date/calendar query or
    /// observed valuation is performed. The returned borrow identifies this
    /// immutable plan; successful evidence is DECLARED at the cash boundary.
    ///
    /// # Errors
    /// Returns [`EnclosureError`] for a nonfinite quote, exact domain rejection,
    /// incomplete admission, allocation failure or internal capacity invariant.
    /// Fully admitted work stops and unavailable finite endpoints are explicit
    /// [`EnclosureOutcome::Unresolved`] values, never partial price evidence.
    pub fn enclose_price_per_100<'p>(
        &'p self,
        yield_rate: f64,
        basis: BondPriceKind,
        budget: EnclosureBudget,
    ) -> EnclosureResult<'p, 's> {
        let mut call = Call::new(yield_rate, EnclosureBasis::Generic(basis), budget);
        let result = run(&mut call, self, yield_rate, basis);
        call.finish(EnclosureOrigin::generic(self), result)
    }
}

fn run(
    call: &mut Call,
    plan: &BondPricingPlan<'_>,
    yield_rate: f64,
    basis: BondPriceKind,
) -> engine::Result<Finite> {
    crate::numeric::finite("yield_rate", yield_rate).map_err(Stop::NonFinite)?;
    let compounding = plan.yield_convention.compounding();
    let frequency = compounding.frequency().and_then(|f| f.events_per_year());
    if let Some(count) = frequency
        && yield_rate <= -f64::from(count)
    {
        return Err(Stop::Domain(EnclosureDomainCause::YieldNotAboveFloor {
            payments_per_year: u16::try_from(count)
                .map_err(|_| Stop::Invariant(Operation::Source))?,
        }));
    }
    call.context.stage = EnclosureStage::Initialization;
    call.charge(8_560)?;
    // The sole numerical frame is initialized here after finite/global checks.
    let mut frame = scalar::ZERO_FRAME;
    let mut all_unit = true;
    let mut all_zero = true;
    for flow in &plan.discount_flows {
        call.context.stage = EnclosureStage::FlowAdmission;
        call.context.next_original_index = Some(flow.index);
        call.charge(16)?;
        all_unit &= flow.horizon.to_bits() << 1 == 0;
        all_zero &= flow.amount.to_bits() << 1 == 0;
        if matches!(
            selected_branch(compounding, flow.horizon),
            ActiveBranch::Simple
        ) {
            call.charge(4_096)?;
            let seed = source_result(source::simple(
                yield_rate.to_bits(),
                flow.horizon.to_bits(),
                &mut frame,
                &mut call.work,
            ))?;
            call.charge(64)?;
            let positive = !seed.negative && frame.raw0.n.iter().any(|&word| word != 0);
            call.context.admission_checked_flows += 1;
            if !positive {
                return Err(Stop::Domain(
                    EnclosureDomainCause::NonpositiveSimpleDenominator {
                        horizon_bits: flow.horizon.to_bits(),
                    },
                ));
            }
        } else {
            call.context.admission_checked_flows += 1;
        }
    }
    call.context.admission_complete = true;
    call.context.next_original_index = None;
    if yield_rate.to_bits() << 1 == 0 || all_unit || all_zero {
        exact(call, plan, basis, &mut frame)
    } else {
        directed(
            call,
            plan,
            yield_rate.to_bits(),
            basis,
            compounding,
            &mut frame,
        )
    }
}

// This is the selector the producer actually uses. Its binary64 switch is the
// existing prepared model's RN(1/m), not an exact rational replacement.
fn selected_branch(compounding: Compounding, horizon: f64) -> ActiveBranch {
    compounding.active_branch(horizon)
}

fn exact(
    call: &mut Call,
    plan: &BondPricingPlan<'_>,
    basis: BondPriceKind,
    frame: &mut scalar::Frame,
) -> engine::Result<Finite> {
    call.context.stage = EnclosureStage::Source;
    call.charge(4_096)?;
    let face = source_result(source::binary(
        plan.bond.face.to_bits(),
        frame,
        &mut call.work,
    ))?;
    let face_significand = frame.raw0.n[0];
    let terms = u64::try_from(plan.discount_flows.len())
        .ok()
        .and_then(|n| n.checked_add(u64::from(basis == BondPriceKind::Clean)))
        .ok_or(Stop::Invariant(Operation::Progress))?;
    let mut sum = fold_result(fold::Session::generic(frame, &mut call.work, terms))?;
    for flow in &plan.discount_flows {
        call.context.next_original_index = Some(flow.index);
        call.context.stage = EnclosureStage::Source;
        sum = fold_result(sum.debit(4_096))?;
        call.context.stage = EnclosureStage::Reduction;
        sum = fold_result(sum.binary(flow.amount.to_bits(), false))?;
        sum = fold_result(sum.debit(8))?;
        call.context.reduction_completed_flows += 1;
    }
    call.context.next_original_index = None;
    call.context.stage = EnclosureStage::BasisAndUnit;
    sum = fold_result(sum.debit(64))?;
    if basis == BondPriceKind::Clean {
        sum = fold_result(sum.debit(4_096))?;
        sum = fold_result(sum.binary(plan.accrued_amount.to_bits(), true))?;
    }
    let staged = fold_result(sum.finish_generic(face_significand, -1074 - face.exponent))?;
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
    // Raw0/E0 is the complete fold result, preserved by the point codec. Stage
    // it before any unit initialization could overwrite that exact target.
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

fn binary(
    call: &mut Call,
    slots: &mut arena::SlotArena,
    frame: &mut scalar::Frame,
    output: usize,
    bits: u64,
    negate: bool,
) -> engine::Result<()> {
    let mut seed = source_result(source::binary(bits, frame, &mut call.work))?;
    seed.negative ^= negate;
    normalize(call, slots, frame, output, seed)
}

fn directed(
    call: &mut Call,
    plan: &BondPricingPlan<'_>,
    yield_bits: u64,
    basis: BondPriceKind,
    compounding: Compounding,
    frame: &mut scalar::Frame,
) -> engine::Result<Finite> {
    call.context.stage = EnclosureStage::Initialization;
    let mut slots = call.allocate()?;
    let p = call.p();
    arena_result(
        slots.initialize_unit(p, frame, &mut call.work),
        Operation::Initialization,
    )?;
    // G0/G6 alternate as the complete signed accumulator; G1 retains ln(base).
    let mut current = 0;
    let mut next = 6;
    let mut log_ready = false;
    for flow in &plan.discount_flows {
        call.context.next_original_index = Some(flow.index);
        call.context.stage = EnclosureStage::Source;
        // Prepay the actual cached coordinate lookup and branch selection.
        call.charge(4_096)?;
        let branch = selected_branch(compounding, flow.horizon);
        if let ActiveBranch::Periodic(count) = branch
            && !log_ready
        {
            call.context.next_original_index = None;
            call.context.stage = EnclosureStage::Source;
            call.charge(4_096)?;
            source_result(source::ratio_affine(
                yield_bits,
                1,
                count,
                frame,
                &mut call.work,
            ))?;
            call.context.stage = EnclosureStage::Kernel;
            kernel_result(
                arena::kernel::logarithm(p, &mut slots, frame, 1, &mut call.work),
                Operation::Log,
            )?;
            log_ready = true;
            call.context.next_original_index = Some(flow.index);
        }
        call.context.stage = EnclosureStage::Source;
        match branch {
            ActiveBranch::Simple => {
                let seed = source_result(source::simple(
                    yield_bits,
                    flow.horizon.to_bits(),
                    frame,
                    &mut call.work,
                ))?;
                normalize(call, &mut slots, frame, 3, seed)?;
                call.context.stage = EnclosureStage::Kernel;
                arena_result(
                    slots.reciprocal(p, 3, 5, frame, &mut call.work),
                    Operation::Divide,
                )?;
            }
            ActiveBranch::Continuous => {
                let mut seed = source_result(source::product(
                    yield_bits,
                    flow.horizon.to_bits(),
                    frame,
                    &mut call.work,
                ))?;
                seed.negative = !seed.negative;
                normalize(call, &mut slots, frame, 3, seed)?;
                call.context.stage = EnclosureStage::Kernel;
                kernel_result(
                    arena::kernel::exponential_interval(p, &mut slots, frame, 3, 5, &mut call.work),
                    Operation::Exp,
                )?;
            }
            ActiveBranch::Periodic(count) => {
                let mut seed = source_result(source::product(
                    f64::from(count).to_bits(),
                    flow.horizon.to_bits(),
                    frame,
                    &mut call.work,
                ))?;
                seed.negative = !seed.negative;
                normalize(call, &mut slots, frame, 3, seed)?;
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
        }
        call.context.stage = EnclosureStage::Source;
        call.charge(4_096)?;
        binary(call, &mut slots, frame, 3, flow.amount.to_bits(), false)?;
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
        call.charge(4_096)?;
        binary(
            call,
            &mut slots,
            frame,
            3,
            plan.accrued_amount.to_bits(),
            true,
        )?;
        arena_result(
            slots.add(p, current, 3, next, frame, &mut call.work),
            Operation::Add,
        )?;
        std::mem::swap(&mut current, &mut next);
    }
    call.charge(4_096)?;
    let scale = source_result(source::per_100_face(
        plan.bond.face.to_bits(),
        frame,
        &mut call.work,
    ))?;
    normalize(call, &mut slots, frame, 5, scale)?;
    arena_result(
        slots.multiply(p, current, 5, 2, frame, &mut call.work),
        Operation::Multiply,
    )?;
    call.complete = true;
    call.context.stage = EnclosureStage::Export;
    call.charge(256)?;
    let (singleton, seed) = arena_result(
        slots.stage_final_point(p, 2, frame, &mut call.work),
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
    call.export(&slots, frame, 2, proof)
}

#[cfg(test)]
#[path = "generic_controls.rs"]
mod controls;
