//! Independent CDS closure regressions retained from the implementation review.
//!
//! The expected values enumerate civil quarters and search an explicit holiday
//! set day by day. They do not call Calendar::adjust, use the production month
//! coordinate formula, or construct private state. This matrix is deliberately
//! limited to plain calendars; TARGET, cap errors and search exhaustion have
//! separate tests.

use sabiquant::calendar::{Calendar, EndpointInclusion};
use sabiquant::schedule::{CdsDatePlan, CdsMaturityConvention, ScheduleCalendar, ScheduleError};
use sabiquant::time::{Date, Weekday};
use std::collections::BTreeSet;
use std::error::Error;

#[test]
fn independent_closures_preserve_all_coupon_roles_and_error_contexts() -> Result<(), Box<dyn Error>>
{
    let quarters: Vec<_> = (2024..=2027)
        .flat_map(|year| [3, 6, 9, 12].map(move |month| Date::from_ymd(year, month, 20).unwrap()))
        .collect();
    let trades: Vec<Date> = [
        "2026-01-01",
        "2026-03-18",
        "2026-03-19",
        "2026-03-20",
        "2026-03-21",
        "2026-06-18",
        "2026-06-19",
        "2026-06-20",
        "2026-06-21",
        "2026-09-18",
        "2026-09-19",
        "2026-09-20",
        "2026-12-18",
        "2026-12-19",
    ]
    .map(|text| text.parse().unwrap())
    .to_vec();
    let maturities: Vec<Date> = [
        "2026-03-20",
        "2026-06-20",
        "2026-09-20",
        "2026-12-20",
        "2027-03-20",
    ]
    .map(|text| text.parse().unwrap())
    .to_vec();
    let mut success = 0;
    let mut collisions = 0;
    let mut no_payment = 0;
    for weekends in [false, true] {
        for closure_start in ["2025-12-01", "2026-03-18", "2026-06-18", "2026-09-18"] {
            let closure_start: Date = closure_start.parse()?;
            for length in [0, 1, 2, 3, 89, 90, 91, 92, 93, 180, 275] {
                let holidays: BTreeSet<_> = (0..length)
                    .map(|day| closure_start.checked_add_days(day).unwrap())
                    .collect();
                let calendar = Calendar::custom(
                    if weekends {
                        vec![Weekday::Saturday, Weekday::Sunday]
                    } else {
                        vec![]
                    },
                    holidays.iter().copied(),
                );
                // Direct daily set membership, independent of Calendar::adjust.
                // Every selected closure here has a business day within this
                // finite range; the separate exhaustion tests own longer gaps.
                let following = |raw: Date| {
                    (0..366)
                        .map(|days| raw.checked_add_days(days).unwrap())
                        .find(|date| {
                            !holidays.contains(date)
                                && (!weekends
                                    || !matches!(
                                        date.weekday(),
                                        Weekday::Saturday | Weekday::Sunday
                                    ))
                        })
                        .unwrap()
                };
                let adjusted: Vec<_> = quarters.iter().copied().map(following).collect();
                for &trade in &trades {
                    let cutoff = trade.checked_add_days(1)?;
                    for &maturity in &maturities {
                        if trade >= maturity {
                            continue;
                        }
                        let result = CdsDatePlan::standard_following(
                            trade,
                            maturity,
                            ScheduleCalendar::Plain(calendar.clone()),
                            CdsMaturityConvention::Quarterly2009,
                            CdsDatePlan::MAX_COUPONS,
                        );
                        // Enumerate the whole finite quarterly set, selecting
                        // by payment inequality without cutoff-quarter math.
                        let selected: Vec<_> = quarters
                            .iter()
                            .enumerate()
                            .filter(|(i, raw)| **raw <= maturity && adjusted[*i] > cutoff)
                            .map(|(i, _)| i)
                            .collect();
                        if selected.is_empty() {
                            let error = result.unwrap_err();
                            assert_eq!(error.kind(), "schedule.no_eligible_cds_payment");
                            assert!(error.source().is_none());
                            match error {
                                ScheduleError::NoEligibleCdsPayment {
                                    trade_date,
                                    payment_selection_cutoff,
                                    unadjusted_maturity,
                                    final_payment,
                                    ..
                                } => {
                                    assert_eq!(trade_date, trade);
                                    assert_eq!(payment_selection_cutoff, cutoff);
                                    assert_eq!(unadjusted_maturity, maturity);
                                    assert_eq!(final_payment, following(maturity));
                                    assert!(final_payment <= cutoff);
                                }
                                other => {
                                    return Err(
                                        format!("unexpected no-payment error: {other:?}").into()
                                    );
                                }
                            }
                            no_payment += 1;
                            continue;
                        }
                        let first = selected[0];
                        let last = *selected.last().unwrap();
                        if (first..=last).any(|i| adjusted[i - 1] >= adjusted[i]) {
                            let error = result.unwrap_err();
                            assert_eq!(error.kind(), "schedule.cds_adjusted_order");
                            assert!(error.source().is_none());
                            // The contract's active first raw quarter is the
                            // last eligible past quarter, or first future one.
                            // Enumerate those date relations to locate the
                            // required pair; do not infer context from errors.
                            let active_first = selected
                                .iter()
                                .copied()
                                .rfind(|&i| quarters[i] <= cutoff)
                                .unwrap_or(first);
                            let collision = (active_first..=last)
                                .find(|&i| adjusted[i - 1] >= adjusted[i])
                                .unwrap();
                            match error {
                                ScheduleError::CdsAdjustedOrder {
                                    index,
                                    previous_contractual,
                                    contractual,
                                    previous_adjusted,
                                    adjusted: current_adjusted,
                                    ..
                                } => {
                                    assert_eq!(index, collision - active_first);
                                    assert_eq!(previous_contractual, quarters[collision - 1]);
                                    assert_eq!(contractual, quarters[collision]);
                                    assert_eq!(previous_adjusted, adjusted[collision - 1]);
                                    assert_eq!(current_adjusted, adjusted[collision]);
                                    assert!(contractual <= maturity);
                                }
                                other => {
                                    return Err(
                                        format!("unexpected collision error: {other:?}").into()
                                    );
                                }
                            }
                            collisions += 1;
                            continue;
                        }
                        let plan = result?;
                        assert_eq!(plan.trade_date(), trade);
                        assert_eq!(plan.payment_selection_cutoff(), cutoff);
                        assert_eq!(plan.unadjusted_maturity(), maturity);
                        assert_eq!(plan.coupon_dates().len(), selected.len());
                        for (coupon, i) in plan.coupon_dates().iter().zip(selected) {
                            let final_coupon = quarters[i] == maturity;
                            let end = if final_coupon { maturity } else { adjusted[i] };
                            assert_eq!(coupon.unadjusted_start(), quarters[i - 1]);
                            assert_eq!(coupon.unadjusted_end(), quarters[i]);
                            assert_eq!(coupon.payment_date(), adjusted[i]);
                            assert_eq!(coupon.accrual_start(), adjusted[i - 1]);
                            assert_eq!(coupon.accrual_end(), end);
                            assert_eq!(
                                coupon.accrual_end_inclusion(),
                                if final_coupon {
                                    EndpointInclusion::Include
                                } else {
                                    EndpointInclusion::Exclude
                                }
                            );
                            assert_eq!(
                                coupon.accrual_day_count(),
                                u32::try_from(end.days_since(adjusted[i - 1]))?
                                    + u32::from(final_coupon)
                            );
                        }
                        success += 1;
                    }
                }
            }
        }
    }
    assert_eq!((success, collisions, no_payment), (3_490, 647, 175));
    assert_eq!(success + collisions + no_payment, 4_312);
    Ok(())
}
