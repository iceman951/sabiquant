//! Independent daily-prefix checks of arithmetic calendar counting.

mod support;

use sabiquant::calendar::{Calendar, JointRule};
use sabiquant::time::{Date, Weekday};
use support::prng;

#[test]
fn every_weekend_mask_and_endpoint_choice_matches_daily_full_range_totals() {
    let mut weekday_totals = [0_i32; 7];
    let mut date = Date::MIN;
    loop {
        weekday_totals[usize::try_from(date.weekday().number_from_monday() - 1).unwrap()] += 1;
        if date == Date::MAX {
            break;
        }
        date = date.checked_add_days(1).unwrap();
    }
    assert_eq!(weekday_totals.iter().sum::<i32>(), 3_652_059);
    for mask in 0_u32..128 {
        let calendar = Calendar::custom(
            Weekday::ALL
                .into_iter()
                .filter(|weekday| mask & (1 << (weekday.number_from_monday() - 1)) != 0),
            [],
        );
        let full: i32 = weekday_totals
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) == 0)
            .map(|(_, count)| count)
            .sum();
        for include_start in [false, true] {
            for include_end in [false, true] {
                let expected = full
                    - i32::from(!include_start && mask & 1 == 0)
                    - i32::from(
                        !include_end
                            && mask & (1 << (Date::MAX.weekday().number_from_monday() - 1)) == 0,
                    );
                assert_eq!(
                    calendar.business_days_between(
                        Date::MIN,
                        Date::MAX,
                        include_start,
                        include_end
                    ),
                    expected
                );
                assert_eq!(
                    calendar.business_days_between(
                        Date::MAX,
                        Date::MIN,
                        include_end,
                        include_start
                    ),
                    -expected
                );
            }
        }
        let changed = calendar.with_holiday(Date::MIN).without_holiday(Date::MAX);
        let expected = full - i32::from(mask & 1 == 0)
            + i32::from(mask & (1 << (Date::MAX.weekday().number_from_monday() - 1)) != 0);
        assert_eq!(
            changed.business_days_between(Date::MIN, Date::MAX, true, true),
            expected
        );
    }
}

#[test]
fn nested_joints_and_overlapping_overrides_match_independent_daily_prefixes() {
    let origin = Date::from_ymd(1999, 1, 1).unwrap();
    let dates: Vec<_> = (0..=1461)
        .map(|offset| origin.checked_add_days(offset).unwrap())
        .collect();
    let a = Calendar::custom(
        [Weekday::Monday, Weekday::Sunday],
        dates.iter().step_by(17).copied(),
    )
    .without_holiday(dates[34]);
    let b = Calendar::weekends_only()
        .with_holiday(dates[34])
        .without_holiday(dates[9]);
    let closed = Calendar::custom(Weekday::ALL, [])
        .without_holiday(dates[34])
        .without_holiday(dates[400]);
    for outer_rule in JointRule::ALL {
        for inner_rule in JointRule::ALL {
            let inner = Calendar::joint(vec![a.clone(), b.clone()], inner_rule).unwrap();
            let calendar = Calendar::joint(vec![inner, closed.clone()], outer_rule)
                .unwrap()
                .with_holiday(dates[34])
                .without_holiday(dates[35]);
            let mut prefix = vec![0];
            for &date in &dates {
                prefix.push(prefix.last().unwrap() + i32::from(calendar.is_business_day(date)));
            }
            prng::check(
                "nested-joint count versus daily prefixes",
                0x4341_4c5f_434e_5433,
                16_000,
                |rng| {
                    let first = usize::try_from(rng.range_i64(0, 1461)).unwrap();
                    let last = usize::try_from(rng.range_i64(0, 1461)).unwrap();
                    for include_first in [false, true] {
                        for include_last in [false, true] {
                            let (lo, hi, include_lo, include_hi, sign) = if first <= last {
                                (first, last, include_first, include_last, 1)
                            } else {
                                (last, first, include_last, include_first, -1)
                            };
                            let expected = if lo == hi {
                                i32::from(include_lo && include_hi) * (prefix[lo + 1] - prefix[lo])
                            } else {
                                let start = lo + usize::from(!include_lo);
                                let end = hi + usize::from(include_hi);
                                sign * (prefix[end] - prefix[start])
                            };
                            let actual = calendar.business_days_between(
                                dates[first],
                                dates[last],
                                include_first,
                                include_last,
                            );
                            if actual != expected {
                                return Err(format!(
                                    "{outer_rule}/{inner_rule}: {first}..{last}, flags {include_first}/{include_last}: {actual} != {expected}"
                                ));
                            }
                        }
                    }
                    Ok(())
                },
            );
        }
    }
}
