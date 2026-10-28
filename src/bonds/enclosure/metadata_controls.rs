//! FR-ENCL-10/11/15 metadata, progress and typed error contract checks.
use super::*;

fn context(
    stage: EnclosureStage,
    checked: usize,
    reduced: usize,
    next: Option<usize>,
) -> EnclosureContext {
    EnclosureContext {
        budget: EnclosureBudget::new(128, 100_000).expect("explicit test configuration"),
        work_used: 900,
        admission_checked_flows: checked,
        reduction_completed_flows: reduced,
        next_original_index: next,
        yield_bits: 0x7ff8_0000_0000_1234,
        basis: EnclosureBasis::Generic(BondPriceKind::Clean),
        stage,
        admission_complete: checked == 5,
    }
}

#[test]
fn admission_and_reduction_counts_are_distinct() {
    for (stage, checked, reduced, next) in [
        (EnclosureStage::QuoteAdmission, 0, 0, None),
        (EnclosureStage::FlowAdmission, 3, 0, Some(7)),
        (EnclosureStage::Source, 5, 2, Some(9)),
        (EnclosureStage::Kernel, 5, 2, Some(9)),
        (EnclosureStage::Reduction, 5, 2, Some(9)),
        (EnclosureStage::BasisAndUnit, 5, 5, None),
        (EnclosureStage::Export, 5, 5, None),
    ] {
        let c = context(stage, checked, reduced, next);
        assert_eq!(c.admission_checked_flows(), checked);
        assert_eq!(c.reduction_completed_flows(), reduced);
        assert_eq!(c.next_original_index(), next);
        assert_eq!(c.admission_complete(), checked == 5);
        assert_eq!(c.stage(), stage);
    }
}

#[test]
fn compact_errors_retain_context_and_typed_causes() {
    let c = context(EnclosureStage::FlowAdmission, 3, 0, Some(7));
    let crate::rates::RateError::Value(value_cause) =
        crate::rates::decimal_from_percent(f64::NAN).expect_err("nonfinite source")
    else {
        panic!("source shape changed")
    };
    let reserve_cause = Vec::<u8>::new()
        .try_reserve(usize::MAX)
        .expect_err("capacity overflow, no allocation");
    let cases = [
        (
            CallFailure::NonFiniteYield {
                context: c,
                cause: value_cause,
            },
            "enclosure.non_finite_yield",
            true,
        ),
        (
            CallFailure::Domain {
                context: c,
                cause: EnclosureDomainCause::YieldNotAboveFloor {
                    payments_per_year: 365,
                },
            },
            "enclosure.yield_not_above_floor",
            true,
        ),
        (
            CallFailure::Domain {
                context: c,
                cause: EnclosureDomainCause::NonpositiveSimpleDenominator {
                    horizon_bits: 1.25f64.to_bits(),
                },
            },
            "enclosure.nonpositive_simple_denominator",
            true,
        ),
        (
            CallFailure::Domain {
                context: c,
                cause: EnclosureDomainCause::NonpositiveMoneyMarketDenominator {
                    days_to_true_maturity: 366,
                },
            },
            "enclosure.nonpositive_money_market_denominator",
            true,
        ),
        (
            CallFailure::AdmissionIncomplete { context: c },
            "enclosure.admission_incomplete",
            false,
        ),
        (
            CallFailure::Allocation {
                context: c,
                requested_bytes: 19_200,
                cause: reserve_cause,
            },
            "enclosure.allocation_failed",
            true,
        ),
        (
            CallFailure::Invariant {
                context: c,
                operation: Operation::Divide,
            },
            "enclosure.internal_capacity",
            false,
        ),
    ];
    for (kind, label, has_source) in cases {
        let e = EnclosureError { kind };
        assert_eq!(e.context(), c);
        assert_eq!(e.context().yield_bits(), 0x7ff8_0000_0000_1234);
        assert_eq!(e.kind(), label);
        assert_eq!(e.source().is_some(), has_source);
        assert_eq!(e.clone().context(), c);
        assert!(e.to_string().contains(label));
        if label == "enclosure.allocation_failed" {
            assert_eq!(e.requested_bytes(), Some(19_200));
        } else {
            assert_eq!(e.requested_bytes(), None);
        }
        if label == "enclosure.internal_capacity" {
            assert_eq!(e.operation(), Some("divide"));
        } else {
            assert_eq!(e.operation(), None);
        }
    }
}

#[test]
fn every_private_arithmetic_tag_is_compact_and_inspectable() {
    let c = context(EnclosureStage::Kernel, 5, 2, Some(9));
    for (operation, label) in [
        (Operation::Source, "source"),
        (Operation::ExactFold, "exact_fold"),
        (Operation::PointEncoding, "point_encoding"),
        (Operation::Initialization, "initialization"),
        (Operation::Progress, "progress"),
        (Operation::Normalize, "normalize"),
        (Operation::Add, "add"),
        (Operation::Multiply, "multiply"),
        (Operation::Divide, "divide"),
        (Operation::Log, "log"),
        (Operation::Exp, "exp"),
        (Operation::Export, "export"),
    ] {
        let e = EnclosureError {
            kind: CallFailure::Invariant {
                context: c,
                operation,
            },
        };
        assert_eq!(e.operation(), Some(label));
        assert_eq!(e.context().next_original_index(), Some(9));
    }
}

// FR-ENCL-11/15 finalization controls: actual opaque error constructors only.
// This creates no finite/unresolved report and no financial result.
pub(super) fn error_samples() -> Vec<(crate::error::Error, &'static str)> {
    let mut c = context(EnclosureStage::QuoteAdmission, 0, 0, None);
    c.work_used = 1088;
    let reserve_cause = Vec::<u8>::new()
        .try_reserve(usize::MAX)
        .expect_err("bounded capacity failure");
    let value_cause = crate::numeric::finite("yield_rate", f64::NAN).expect_err("nonfinite quote");
    let errors = [
        CallFailure::NonFiniteYield {
            context: c,
            cause: value_cause,
        },
        CallFailure::Domain {
            context: c,
            cause: EnclosureDomainCause::YieldNotAboveFloor {
                payments_per_year: 365,
            },
        },
        CallFailure::Domain {
            context: c,
            cause: EnclosureDomainCause::NonpositiveSimpleDenominator {
                horizon_bits: 1.25f64.to_bits(),
            },
        },
        CallFailure::Domain {
            context: c,
            cause: EnclosureDomainCause::NonpositiveMoneyMarketDenominator {
                days_to_true_maturity: 366,
            },
        },
        CallFailure::AdmissionIncomplete { context: c },
        CallFailure::Allocation {
            context: c,
            requested_bytes: 19200,
            cause: reserve_cause,
        },
        CallFailure::Invariant {
            context: c,
            operation: Operation::PointEncoding,
        },
    ];
    let mut samples: Vec<_> = errors
        .into_iter()
        .map(|kind| {
            let e = EnclosureError { kind };
            let label = e.kind();
            (e.into(), label)
        })
        .collect();
    for (p, work) in [(96, 100000), (64, 53679)] {
        let e = EnclosureBudget::new(p, work).expect_err("exact final budget boundary");
        let label = e.kind();
        samples.push((e.into(), label));
    }
    samples
}

#[test]
fn root_wrappers_and_error_equality_preserve_final_context() {
    fn error_traits<T: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    error_traits::<EnclosureError>();
    error_traits::<EnclosureBudgetError>();
    for (root, label) in error_samples() {
        assert_eq!(root.kind(), label);
        assert_eq!(root, root.clone());
        let source = root.source().expect("actual root immediate source");
        match &root {
            crate::error::Error::Enclosure(inner) => {
                assert_eq!(source.downcast_ref::<EnclosureError>(), Some(inner))
            }
            crate::error::Error::EnclosureBudget(inner) => {
                assert_eq!(source.downcast_ref::<EnclosureBudgetError>(), Some(inner))
            }
            _ => panic!("only new wrapper samples"),
        }
        assert_eq!(root.to_string(), source.to_string());
    }
    let c = context(EnclosureStage::Initialization, 5, 0, None);
    let allocation = |context, requested_bytes| EnclosureError {
        kind: CallFailure::Allocation {
            context,
            requested_bytes,
            cause: Vec::<u8>::new()
                .try_reserve(usize::MAX)
                .expect_err("no allocation"),
        },
    };
    let a = allocation(c, 19200);
    let b = allocation(c, 19200);
    assert_eq!(a, b);
    assert_ne!(a, allocation(c, 19201));
    let mut changed = c;
    changed.work_used += 1;
    assert_ne!(a, allocation(changed, 19200));
    assert!(
        a.source()
            .expect("reserve source")
            .is::<std::collections::TryReserveError>()
    );
}
