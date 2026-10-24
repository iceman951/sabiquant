//! Post-production supplement: FR-CAD-53/54/58 residual-scale boundary.
//! Authored after the semantic mutation run showed that no frozen hand,
//! private test or corpus distinguished a 2^969 from the adopted 2^970
//! threshold. This is not a frozen blind hand. Uses public APIs only.

#![expect(
    clippy::expect_used,
    reason = "explicit valid supplemental test inputs"
)]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    CanadianBond, CanadianCouponSchedule, CanadianFrequency, CanadianPrice, CanadianPriceBasis,
    CanadianPricingPlan, CanadianSourcePolicy, CanadianYieldConvention, CanadianYieldSelection,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::math::{RootBracket, SolverOptions};
use sabiquant::time::Date;

const TWO_969: f64 = f64::from_bits(1992_u64 << 52);
const TWO_970: f64 = f64::from_bits(1993_u64 << 52);
const TWO_971: f64 = f64::from_bits(1994_u64 << 52);

/// A twenty-year semiannual zero-coupon bond settled at its accrual start, so
/// the compound dirty price is 100(1+y/2)^-40 and strictly decreasing in y.
fn forty_period_zero() -> CanadianBond {
    let start = Date::from_ymd(2000, 1, 1).expect("valid civil date");
    let quasi: Vec<_> = (0..=40)
        .map(|i| {
            start
                .checked_add_months(6 * i)
                .expect("valid semiannual date")
        })
        .collect();
    let schedule = CanadianCouponSchedule::new(
        start,
        quasi[40],
        CanadianFrequency::Semiannual,
        quasi.clone(),
        quasi[1..].to_vec(),
        EndOfMonth::Ignore,
        41,
    )
    .expect("regular direct-anchored cycle");
    CanadianBond::new(
        start,
        DecimalAmount::new(0, 0).expect("zero coupon"),
        schedule,
        quasi[40],
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )
    .expect("valid zero-coupon instrument")
}

/// Bisects public compound evaluations for a yield whose observed model dirty
/// price lies in `[low, high)`; the range itself is asserted by the caller.
fn yield_with_dirty_in(plan: &CanadianPricingPlan<'_>, low: f64, high: f64) -> Option<(f64, f64)> {
    // 100*2^(25*40) overflows nothing and exceeds 2^971; 100*2^(21*40) < 2^969.
    let mut near = -2.0 + f64::from_bits(997_u64 << 52); // -2 + 2^-26
    let mut far = -2.0 + f64::from_bits(1001_u64 << 52); // -2 + 2^-22
    for _ in 0..200 {
        let y = near + (far - near) / 2.0;
        let dirty = plan
            .evaluate(y)
            .expect("finite admitted compound quote")
            .model_dirty_price()
            .value();
        if dirty >= high {
            near = y;
        } else if dirty < low {
            far = y;
        } else {
            return Some((y, dirty));
        }
    }
    None
}

/// FR-CAD-53/54/58: scaling starts at 2^970, not one binade earlier or later.
/// The bracket's lower endpoint is the exact root, so one cached callback
/// terminates with a zero amount residual in both scale regimes.
#[test]
fn residual_scale_switches_exactly_at_two_pow_970() {
    let bond = forty_period_zero();
    let plan = CanadianPricingPlan::new(
        &bond,
        bond.schedule().accrual_start(),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        40,
    )
    .expect("forty future bundles");
    for (low, high, scale) in [(TWO_969, TWO_970, 0), (TWO_970, TWO_971, 1)] {
        let (y, dirty) = yield_with_dirty_in(&plan, low, high)
            .expect("a represented yield maps into the binade");
        assert!(low <= dirty && dirty < high);
        let target = CanadianPrice::new(dirty, CanadianPriceBasis::ModelDirty).expect("finite");
        let report = plan
            .yield_from_price(
                target,
                RootBracket::new(y, 0.0).expect("finite ordered bracket"),
                SolverOptions::new(1e-12, 1e-12, 1).expect("one callback"),
            )
            .expect("exact lower-endpoint root");
        assert_eq!(report.residual_scale_exponent(), scale);
        assert_eq!(
            report.model_dirty_target_per_100().to_bits(),
            dirty.to_bits()
        );
        assert_eq!(report.yield_rate().to_bits(), y.to_bits());
        assert_eq!(report.solve_report().evaluations().function(), 1);
        assert_eq!(report.solve_report().residual().to_bits(), 0);
        assert_eq!(
            report
                .amount_residual()
                .expect("finite difference")
                .to_bits(),
            0
        );
    }
}
