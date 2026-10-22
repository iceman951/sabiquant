//! Post-production independent Fraction seed-midpoint controls, FR-UST-10/11/13/14/15/25.
//! Expected bits are independently reconstructed with Fraction, directed p256
//! reciprocal endpoints, extended53 terms and direct IEEE extraction. This is
//! a post-production supplement, not a change to a frozen pre-production row.
//! The full-period q is an odd-lower midpoint; rounding it down changes four
//! getters. The half-period q differs from delta/2 and detects false coupling.
//! Derivation and counterfactuals: M8_US_TREASURY_NUMERIC_IMPLEMENTATION_REVIEW.md.
use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    TreasuryDailyBasis, TreasuryPricingPlan, TreasurySourcePolicy, TreasuryTieRule, UsTreasury,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::time::Date;

#[test]
fn independent_full_and_half_odd_midpoint_observations() -> Result<(), Box<dyn std::error::Error>> {
    let issue = Date::from_ymd(2024, 12, 7)?;
    let maturity = Date::from_ymd(2025, 6, 7)?;
    let treasury = UsTreasury::new(
        issue,
        issue,
        maturity,
        DecimalAmount::new(0, 0)?,
        maturity,
        vec![issue, maturity],
        EndOfMonth::Ignore,
        TreasurySourcePolicy::new(
            TreasuryDailyBasis::ExactRatio,
            TreasuryTieRule::NearestTiesEven,
        ),
        UsTreasury::MAX_QUASI_DATES,
    )?;
    for (settlement, dirty_bits, expected) in [
        (
            issue,
            0x4058_ffff_ffff_fffe,
            [
                0xc048_ffff_ffff_fffb,
                0x4048_ffff_ffff_fff8,
                0x4048_ffff_ffff_fffe,
                0x3fdf_ffff_ffff_fffc,
                0x3fe0_0000_0000_0000,
                0x3fdf_ffff_ffff_fff8,
            ],
        ),
        (
            Date::from_ymd(2025, 3, 8)?,
            0x4058_ffff_ffff_ffff,
            [
                0xc038_ffff_ffff_fffd,
                0x4028_ffff_ffff_fffc,
                0x4038_ffff_ffff_ffff,
                0x3fcf_ffff_ffff_fffd,
                0x3fd0_0000_0000_0000,
                0x3fbf_ffff_ffff_fffc,
            ],
        ),
    ] {
        let plan = TreasuryPricingPlan::new(&treasury, settlement, 1)?;
        let risk = plan.risk(f64::from_bits(0x3cc8_0000_0000_0000))?;
        assert_eq!(risk.dirty_per_100().to_bits(), dirty_bits);
        let observed = [
            risk.price_derivative()?.to_bits(),
            risk.price_second_derivative()?.to_bits(),
            risk.time_weighted_amount()?.to_bits(),
            risk.modified_duration()?.to_bits(),
            risk.macaulay_duration()?.to_bits(),
            risk.convexity()?.to_bits(),
        ];
        assert_eq!(observed, expected, "settlement {settlement}");
    }
    Ok(())
}
