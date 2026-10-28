//! Public enclosure configuration, error and opaque-report contracts.
//!
//! FR-ENCL-04/10/11/15: these checks use the public API and construct only
//! budgets, their real validation errors and public domain causes. They create
//! no price evidence.

use sabiquant::bonds::{
    EnclosureBasis, EnclosureBudget, EnclosureBudgetError, EnclosureContext, EnclosureDomainCause,
    EnclosureEncoding, EnclosureError, EnclosureOrigin, EnclosureOutcome, EnclosureProofKind,
    EnclosureResult, EnclosureStage, EnclosureUnresolvedReason, PriceEnclosure,
    PriceEnclosureUnresolved,
};
use sabiquant::error::Error;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::hash::Hash;

const MINIMUM_WORK: u64 = 53_680;
const PRECISIONS: [u16; 5] = [64, 128, 256, 512, 1024];

fn small_value<T: Clone + Copy + Debug + PartialEq + Eq + Hash + Send + Sync>() {}
fn opaque_report<T: Clone + Debug + Send + Sync>() {}
fn owned_error<T: StdError + Clone + PartialEq + Send + Sync + 'static>() {}

// If a forbidden trait becomes implemented, both marker implementations apply
// and the inferred marker type becomes ambiguous. This is an ordinary compile
// assertion in this public test target, with no synthetic report construction.
macro_rules! assert_not_impl {
    ($ty:ty, $bound:path) => {
        const _: fn() = || {
            trait Unambiguous<Marker> {
                fn marker() {}
            }
            impl<T: ?Sized> Unambiguous<()> for T {}
            struct IfImplemented;
            impl<T: ?Sized + $bound> Unambiguous<IfImplemented> for T {}
            let _ = <$ty as Unambiguous<_>>::marker;
        };
    };
}

assert_not_impl!(PriceEnclosure<'static, 'static>, Copy);
assert_not_impl!(PriceEnclosure<'static, 'static>, PartialEq);
assert_not_impl!(PriceEnclosure<'static, 'static>, Eq);
assert_not_impl!(PriceEnclosure<'static, 'static>, Hash);
assert_not_impl!(PriceEnclosureUnresolved<'static, 'static>, Copy);
assert_not_impl!(PriceEnclosureUnresolved<'static, 'static>, PartialEq);
assert_not_impl!(PriceEnclosureUnresolved<'static, 'static>, Eq);
assert_not_impl!(PriceEnclosureUnresolved<'static, 'static>, Hash);
assert_not_impl!(EnclosureOutcome<'static, 'static>, Copy);
assert_not_impl!(EnclosureOutcome<'static, 'static>, PartialEq);
assert_not_impl!(EnclosureOutcome<'static, 'static>, Eq);
assert_not_impl!(EnclosureOutcome<'static, 'static>, Hash);
assert_not_impl!(EnclosureOrigin<'static, 'static>, PartialEq);
assert_not_impl!(EnclosureOrigin<'static, 'static>, Eq);
assert_not_impl!(EnclosureOrigin<'static, 'static>, Hash);

#[test]
fn only_the_five_documented_precisions_are_supported() {
    // FR-ENCL-04: exhaust the complete input type, including both endpoints.
    for precision in u16::MIN..=u16::MAX {
        match EnclosureBudget::new(precision, MINIMUM_WORK) {
            Ok(budget) => {
                assert!(PRECISIONS.contains(&precision));
                assert_eq!(budget.precision_bits(), precision);
                assert_eq!(budget.max_work(), MINIMUM_WORK);
            }
            Err(error) => {
                assert!(!PRECISIONS.contains(&precision));
                assert_eq!(error.kind(), "enclosure_budget.precision");
                assert_eq!(error.precision_bits(), precision);
                assert_eq!(error.max_work(), MINIMUM_WORK);
                assert_eq!(error.work_used(), 0);
                assert!(error.source().is_none());
            }
        }
    }
}

#[test]
fn work_minimum_is_inclusive_for_every_supported_precision() -> Result<(), Box<dyn StdError>> {
    // FR-ENCL-04/11: boundary validation preserves original attempted fields.
    for precision in PRECISIONS {
        for maximum in [0, 1, MINIMUM_WORK - 2, MINIMUM_WORK - 1] {
            let error = invalid_budget(precision, maximum);
            assert_eq!(error.kind(), "enclosure_budget.minimum_work");
            assert_eq!(error.precision_bits(), precision);
            assert_eq!(error.max_work(), maximum);
            assert_eq!(error.work_used(), 0);
        }
        for maximum in [MINIMUM_WORK, MINIMUM_WORK + 1, u64::MAX] {
            let budget = EnclosureBudget::new(precision, maximum)?;
            assert_eq!(budget.precision_bits(), precision);
            assert_eq!(budget.max_work(), maximum);
        }
    }
    Ok(())
}

#[test]
fn unsupported_precision_precedes_insufficient_work() {
    // FR-ENCL-04/11: two invalid fields must not change the failure order.
    for precision in [
        0,
        1,
        63,
        65,
        127,
        129,
        255,
        257,
        511,
        513,
        1023,
        1025,
        u16::MAX,
    ] {
        for maximum in [0, MINIMUM_WORK - 1, u64::MAX] {
            let error = invalid_budget(precision, maximum);
            assert_eq!(error.kind(), "enclosure_budget.precision");
            assert_eq!(error.precision_bits(), precision);
            assert_eq!(error.max_work(), maximum);
            assert_eq!(error.work_used(), 0);
        }
    }
}

#[test]
fn budget_errors_convert_to_root_with_their_typed_immediate_source() -> Result<(), Box<dyn StdError>>
{
    // FR-ENCL-11: use real public failures rather than constructing error arms.
    for (precision, maximum) in [(63, u64::MAX), (64, MINIMUM_WORK - 1), (0, 0)] {
        let budget_error = invalid_budget(precision, maximum);
        let root: Error = budget_error.into();
        assert_eq!(root.kind(), budget_error.kind());
        assert_eq!(root.to_string(), budget_error.to_string());
        assert_eq!(root.clone(), root);

        let Error::EnclosureBudget(stored) = &root else {
            return Err("budget validation must retain its dedicated root wrapper".into());
        };
        assert_eq!(*stored, budget_error);
        let source = root.source().ok_or("root wrapper must retain a source")?;
        let typed = source
            .downcast_ref::<EnclosureBudgetError>()
            .ok_or("immediate source must have the public budget error type")?;
        assert!(std::ptr::eq(typed, stored));
        assert_eq!(typed.precision_bits(), precision);
        assert_eq!(typed.max_work(), maximum);
        assert!(typed.source().is_none());
    }
    Ok(())
}

#[test]
fn domain_causes_keep_stable_kinds_original_fields_and_error_traits()
-> Result<(), Box<dyn StdError>> {
    // FR-ENCL-10/11: these are public cause values, not fabricated call errors.
    fn cause_traits<T: StdError + Clone + Eq>() {}
    cause_traits::<EnclosureDomainCause>();

    let causes = [
        (
            EnclosureDomainCause::YieldNotAboveFloor {
                payments_per_year: 12,
            },
            "enclosure.yield_not_above_floor",
        ),
        (
            EnclosureDomainCause::NonpositiveSimpleDenominator {
                horizon_bits: 0x3ff0_0000_0000_0001,
            },
            "enclosure.nonpositive_simple_denominator",
        ),
        (
            EnclosureDomainCause::NonpositiveMoneyMarketDenominator {
                days_to_true_maturity: 1461,
            },
            "enclosure.nonpositive_money_market_denominator",
        ),
    ];
    for (cause, kind) in causes {
        assert_eq!(cause.kind(), kind);
        assert!(cause.source().is_none());
        match cause {
            EnclosureDomainCause::YieldNotAboveFloor { payments_per_year } => {
                assert_eq!(payments_per_year, 12);
            }
            EnclosureDomainCause::NonpositiveSimpleDenominator { horizon_bits } => {
                assert_eq!(horizon_bits, 0x3ff0_0000_0000_0001);
            }
            EnclosureDomainCause::NonpositiveMoneyMarketDenominator {
                days_to_true_maturity,
            } => assert_eq!(days_to_true_maturity, 1461),
            _ => return Err("unexpected public domain cause".into()),
        }
    }
    Ok(())
}

#[test]
fn all_public_enclosure_shapes_have_the_documented_traits() {
    // FR-ENCL-10: all fourteen nominal types and the result alias are named
    // through the public re-exports; opaque reports need no fabricated value.
    small_value::<EnclosureBudget>();
    small_value::<EnclosureBudgetError>();
    small_value::<EnclosureBasis>();
    small_value::<EnclosureStage>();
    small_value::<EnclosureContext>();
    small_value::<EnclosureEncoding>();
    small_value::<EnclosureProofKind>();
    small_value::<EnclosureUnresolvedReason>();
    small_value::<EnclosureDomainCause>();
    owned_error::<EnclosureBudgetError>();
    owned_error::<EnclosureError>();
    opaque_report::<EnclosureOrigin<'static, 'static>>();
    opaque_report::<PriceEnclosure<'static, 'static>>();
    opaque_report::<PriceEnclosureUnresolved<'static, 'static>>();
    opaque_report::<EnclosureOutcome<'static, 'static>>();
    opaque_report::<EnclosureResult<'static, 'static>>();

    fn origin_is_copy<T: Copy>() {}
    origin_is_copy::<EnclosureOrigin<'static, 'static>>();

    // Type-check the actual result alias and the second root From conversion
    // without synthesizing an EnclosureError or a proved price report.
    fn result_shape<'p, 's: 'p>(
        value: EnclosureResult<'p, 's>,
    ) -> Result<EnclosureOutcome<'p, 's>, EnclosureError> {
        value
    }
    let _: fn(
        EnclosureResult<'static, 'static>,
    ) -> Result<EnclosureOutcome<'static, 'static>, EnclosureError> = result_shape;
    let _: fn(EnclosureError) -> Error = Into::into;
}

#[test]
fn call_error_stays_within_the_native_compact_layout_checkpoint() {
    // FR-ENCL-15: this measures the current native layout, not a stable ABI,
    // RootError size or a complete financial producer's storage/stack bound.
    assert!(
        std::mem::size_of::<EnclosureError>() <= 160,
        "native EnclosureError must stay within the 160-byte checkpoint"
    );
}

#[expect(
    clippy::expect_used,
    reason = "The public test deliberately supplies invalid budget inputs."
)]
fn invalid_budget(precision: u16, maximum: u64) -> EnclosureBudgetError {
    EnclosureBudget::new(precision, maximum).expect_err("invalid budget must be rejected")
}
