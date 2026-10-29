//! Bounded call bookkeeping and controlled final evidence construction.

use super::arithmetic::{arena, codec, fold, scalar, source};
use super::*;

pub(super) struct Call {
    pub(super) context: EnclosureContext,
    pub(super) work: scalar::Work,
    pub(super) complete: bool,
}

pub(super) struct Finite {
    evidence: SettlementEvidence,
    encoding: EnclosureEncoding,
    proof: EnclosureProofKind,
}

pub(super) enum Stop {
    Work,
    NonFinite(ValueError),
    Domain(EnclosureDomainCause),
    Allocation(std::collections::TryReserveError),
    Invariant(Operation),
    Endpoint,
}

pub(super) type Result<T> = std::result::Result<T, Stop>;

pub(super) fn debit(work: &mut scalar::Work, units: u64) -> Result<()> {
    let next = work.used.checked_add(units).ok_or(Stop::Work)?;
    if next > work.max {
        return Err(Stop::Work);
    }
    work.used = next;
    Ok(())
}

pub(super) fn source_result<T>(result: std::result::Result<T, source::Error>) -> Result<T> {
    result.map_err(|error| match error {
        source::Error::WorkLimit => Stop::Work,
        _ => Stop::Invariant(Operation::Source),
    })
}

pub(super) fn fold_result<T>(result: std::result::Result<T, fold::Error>) -> Result<T> {
    result.map_err(|error| match error {
        fold::Error::WorkLimit => Stop::Work,
        _ => Stop::Invariant(Operation::ExactFold),
    })
}

pub(super) fn arena_result<T>(
    result: std::result::Result<T, arena::Error>,
    operation: Operation,
) -> Result<T> {
    result.map_err(|error| match error {
        arena::Error::Scalar(scalar::Error::WorkLimit) => Stop::Work,
        arena::Error::Reserve { cause, .. } => Stop::Allocation(cause),
        _ => Stop::Invariant(operation),
    })
}

pub(super) fn kernel_result<T>(
    result: std::result::Result<T, arena::kernel::KernelError>,
    operation: Operation,
) -> Result<T> {
    result.map_err(|error| match error {
        arena::kernel::KernelError::Arena(arena::Error::Scalar(scalar::Error::WorkLimit)) => {
            Stop::Work
        }
        _ => Stop::Invariant(operation),
    })
}

impl Call {
    pub(super) fn new(yield_rate: f64, basis: EnclosureBasis, budget: EnclosureBudget) -> Self {
        Self {
            context: EnclosureContext {
                budget,
                work_used: 0,
                admission_checked_flows: 0,
                reduction_completed_flows: 0,
                next_original_index: None,
                yield_bits: yield_rate.to_bits(),
                basis,
                stage: EnclosureStage::QuoteAdmission,
                admission_complete: false,
            },
            work: scalar::Work {
                used: 64,
                max: budget.max_work - 1_024,
            },
            complete: false,
        }
    }

    pub(super) fn p(&self) -> usize {
        usize::from(self.context.budget.precision_bits)
    }

    pub(super) fn charge(&mut self, units: u64) -> Result<()> {
        debit(&mut self.work, units)
    }

    pub(super) fn allocate(&mut self) -> Result<arena::SlotArena> {
        self.context.stage = EnclosureStage::Initialization;
        self.context.next_original_index = None;
        self.charge(44_032)?;
        arena_result(arena::SlotArena::try_new(), Operation::Initialization)
    }

    /// The raw target is complete before this method, including all unit shifts.
    pub(super) fn encode_raw(
        &mut self,
        frame: &mut scalar::Frame,
        seed: source::Seed,
        proof: EnclosureProofKind,
    ) -> Result<Option<Finite>> {
        let encoding = codec::encode(seed.negative, seed.exponent, frame, &mut self.work).map_err(
            |error| match error {
                codec::Error::WorkLimit => Stop::Work,
                _ => Stop::Invariant(Operation::PointEncoding),
            },
        )?;
        let (evidence, encoding) = match encoding {
            codec::Encoding::Decimal { coefficient, scale } => {
                let scale =
                    u8::try_from(scale).map_err(|_| Stop::Invariant(Operation::PointEncoding))?;
                let amount = DecimalAmount::new(coefficient, scale)
                    .map_err(|_| Stop::Invariant(Operation::PointEncoding))?;
                let evidence = SettlementEvidence::declared_decimal_interval(amount, amount)
                    .map_err(|_| Stop::Invariant(Operation::PointEncoding))?;
                (evidence, EnclosureEncoding::ExactDecimalPoint)
            }
            codec::Encoding::Binary64 { bits } => {
                let point = f64::from_bits(bits);
                let evidence = SettlementEvidence::declared_binary64_interval(point, point)
                    .map_err(|_| Stop::Invariant(Operation::PointEncoding))?;
                (evidence, EnclosureEncoding::ExactBinary64Point)
            }
            codec::Encoding::NoFittingPoint => return Ok(None),
        };
        Ok(Some(Finite {
            evidence,
            encoding,
            proof,
        }))
    }

    pub(super) fn export(
        &mut self,
        slots: &arena::SlotArena,
        frame: &mut scalar::Frame,
        final_slot: usize,
        proof: EnclosureProofKind,
    ) -> Result<Finite> {
        let (lower, upper) = arena_result(
            slots.export(self.p(), final_slot, &mut self.work),
            Operation::Export,
        )?;
        let (Some(lower), Some(upper)) = (lower, upper) else {
            return Err(Stop::Endpoint);
        };
        if lower == upper {
            let seed = source_result(source::binary(lower, frame, &mut self.work))?;
            let proof = if proof == EnclosureProofKind::ExactRationalIdentity {
                proof
            } else {
                EnclosureProofKind::IntervalCollapse
            };
            return self
                .encode_raw(frame, seed, proof)?
                .ok_or(Stop::Invariant(Operation::PointEncoding));
        }
        let evidence = SettlementEvidence::declared_binary64_interval(
            f64::from_bits(lower),
            f64::from_bits(upper),
        )
        .map_err(|_| Stop::Invariant(Operation::Export))?;
        Ok(Finite {
            evidence,
            encoding: EnclosureEncoding::OutwardBinary64Bounds,
            proof,
        })
    }

    pub(super) fn finish<'p, 's: 'p>(
        mut self,
        origin: EnclosureOrigin<'p, 's>,
        result: Result<Finite>,
    ) -> EnclosureResult<'p, 's> {
        // Work.max reserved this exact terminal tariff on entry, so the sum
        // cannot overflow, even for a u64::MAX caller budget.
        self.context.work_used = self.work.used + 1_024;
        match result {
            Ok(finite) => {
                self.canonical_quote();
                self.context.stage = EnclosureStage::Completion;
                self.context.next_original_index = None;
                Ok(EnclosureOutcome::Finite(PriceEnclosure {
                    origin,
                    context: self.context,
                    evidence: finite.evidence,
                    encoding: finite.encoding,
                    proof_kind: finite.proof,
                }))
            }
            Err(Stop::Work) if self.context.admission_complete => {
                Ok(self.unresolved(origin, EnclosureUnresolvedReason::WorkLimitAfterAdmission))
            }
            Err(Stop::Endpoint) if self.complete => {
                Ok(self.unresolved(origin, EnclosureUnresolvedReason::FiniteEndpointUnavailable))
            }
            Err(stop) => {
                let context = self.context;
                let kind = match stop {
                    Stop::Work => CallFailure::AdmissionIncomplete { context },
                    Stop::NonFinite(cause) => CallFailure::NonFiniteYield { context, cause },
                    Stop::Domain(cause) => CallFailure::Domain { context, cause },
                    Stop::Allocation(cause) => CallFailure::Allocation {
                        context,
                        requested_bytes: arena::SLOT_COUNT * std::mem::size_of::<arena::Interval>(),
                        cause,
                    },
                    Stop::Invariant(operation) => CallFailure::Invariant { context, operation },
                    Stop::Endpoint => CallFailure::Invariant {
                        context,
                        operation: Operation::Export,
                    },
                };
                Err(EnclosureError { kind })
            }
        }
    }

    fn canonical_quote(&mut self) {
        if self.context.yield_bits << 1 == 0 {
            self.context.yield_bits = 0;
        }
    }

    fn unresolved<'p, 's: 'p>(
        mut self,
        origin: EnclosureOrigin<'p, 's>,
        reason: EnclosureUnresolvedReason,
    ) -> EnclosureOutcome<'p, 's> {
        self.canonical_quote();
        EnclosureOutcome::Unresolved(PriceEnclosureUnresolved {
            origin,
            context: self.context,
            reason,
            complete_extended_arithmetic: self.complete,
        })
    }
}
