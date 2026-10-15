//! Independent CDS calendar masks, primary coupon table and explicit overrides.

use sabiquant::calendar::{Calendar, CalendarError, EndpointInclusion, JointRule};
use sabiquant::schedule::{
    CdsDatePlan, CdsMaturityConvention as Regime, ScheduleCalendar, ScheduleError,
};
use sabiquant::time::{Date, DateError, Weekday};
use std::collections::BTreeMap;
use std::error::Error;

const REGIMES: [Regime; 2] = [Regime::Quarterly2009, Regime::Semiannual2015];
const WEEKDAYS: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

#[expect(clippy::expect_used, reason = "hand-audited literal test dates")]
fn d(text: &str) -> Date {
    text.parse().expect("valid literal date")
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn ordinal(date: Date) -> i64 {
    let (year, month, day) = date.to_ymd();
    let y = i64::from(year - 1);
    let before: u32 = (1..month).map(|m| month_days(year, m)).sum();
    365 * y + y / 4 - y / 100 + y / 400 + i64::from(before + day - 1)
}

/// Independent model: a bit excludes its weekday, beginning with Monday.
/// Explicit true/false dates override that rule; no production calendar query
/// or adjustment contributes to the expected results.
#[derive(Clone, Debug)]
struct Model {
    closed_weekdays: u8,
    overrides: BTreeMap<Date, bool>,
}

impl Model {
    fn mask(closed_weekdays: u8) -> Self {
        Self {
            closed_weekdays,
            overrides: BTreeMap::new(),
        }
    }

    fn business(&self, date: Date) -> bool {
        self.overrides.get(&date).copied().unwrap_or_else(|| {
            let weekday = ordinal(date).rem_euclid(7);
            self.closed_weekdays & (1_u8 << weekday) == 0
        })
    }

    fn close(mut self, start: Date, end: Date) -> Result<Self, DateError> {
        for offset in 0..=end.days_since(start) {
            self.overrides
                .insert(start.checked_add_days(i64::from(offset))?, false);
        }
        Ok(self)
    }

    fn open(mut self, date: Date) -> Self {
        self.overrides.insert(date, true);
        self
    }

    fn joint(&self, other: &Self, rule: JointRule) -> Self {
        let intersection = rule == JointRule::JoinHolidays;
        let mut joined = Self::mask(if intersection {
            self.closed_weekdays | other.closed_weekdays
        } else {
            self.closed_weekdays & other.closed_weekdays
        });
        for date in self.overrides.keys().chain(other.overrides.keys()) {
            let business = if intersection {
                self.business(*date) && other.business(*date)
            } else {
                self.business(*date) || other.business(*date)
            };
            joined.overrides.insert(*date, business);
        }
        joined
    }

    fn calendar(&self) -> Calendar {
        let excluded = WEEKDAYS
            .into_iter()
            .enumerate()
            .filter_map(|(bit, weekday)| {
                (self.closed_weekdays & (1_u8 << bit) != 0).then_some(weekday)
            });
        let mut calendar = Calendar::custom(excluded, []);
        for (date, business) in &self.overrides {
            calendar = if *business {
                calendar.without_holiday(*date)
            } else {
                calendar.with_holiday(*date)
            };
        }
        calendar
    }

    fn following(&self, raw: Date) -> Result<Date, Box<dyn Error>> {
        // Explicit candidate enumeration, not Calendar::adjust and not a
        // weekday-offset shortcut. The finite closure models remain well
        // inside this bound; all-seven-closed masks are classified first.
        let mut candidate = raw;
        for _ in 0..366 {
            if self.business(candidate) {
                return Ok(candidate);
            }
            candidate = candidate.checked_add_days(1)?;
        }
        Err("independent calendar oracle exhausted its candidate bound".into())
    }
}

#[derive(Debug)]
struct Coupon {
    raw_start: Date,
    raw_end: Date,
    payment: Date,
    start: Date,
    end: Date,
    include_end: bool,
    days: u32,
}

#[derive(Debug)]
enum Outcome {
    Plan(Vec<Coupon>),
    NoEligible {
        final_payment: Date,
    },
    NoBusinessWeekday,
    Collision {
        previous: Date,
        current: Date,
        previous_paid: Date,
        current_paid: Date,
    },
}

fn expected(model: &Model, trade: Date, maturity: Date) -> Result<Outcome, Box<dyn Error>> {
    if model.closed_weekdays == 127 && !model.overrides.values().any(|open| *open) {
        return Ok(Outcome::NoBusinessWeekday);
    }
    // Enumerate a wide civil-quarter grid, independently adjust every raw
    // point, and select by strict inequality across the whole grid. This
    // deliberately does not reproduce the implementation's cutoff-quarter
    // branch or its separately selected precursor.
    let mut grid = Vec::new();
    for year in (trade.year() - 1)..=maturity.year() {
        for month in [3, 6, 9, 12] {
            let raw = Date::from_ymd(year, month, 20)?;
            if raw <= maturity {
                grid.push((raw, model.following(raw)?));
            }
        }
    }
    let cutoff = ordinal(trade) + 1;
    let Some(first) = grid.iter().position(|(_, paid)| ordinal(*paid) > cutoff) else {
        return Ok(Outcome::NoEligible {
            final_payment: grid.last().ok_or("oracle grid must include maturity")?.1,
        });
    };
    assert!(first > 0, "the oracle includes the preceding civil year");
    let mut coupons = Vec::new();
    for index in first..grid.len() {
        let (raw_start, start) = grid[index - 1];
        let (raw_end, payment) = grid[index];
        if start >= payment {
            return Ok(Outcome::Collision {
                previous: raw_start,
                current: raw_end,
                previous_paid: start,
                current_paid: payment,
            });
        }
        let include_end = raw_end == maturity;
        let end = if include_end { maturity } else { payment };
        coupons.push(Coupon {
            raw_start,
            raw_end,
            payment,
            start,
            end,
            include_end,
            days: u32::try_from(ordinal(end) - ordinal(start) + i64::from(include_end))?,
        });
    }
    Ok(Outcome::Plan(coupons))
}

// Returns a compact category for exact request-count assertions:
// plan, no eligible payment, no business weekday, adjusted collision.
fn compare(
    outcome: &Outcome,
    result: Result<CdsDatePlan, ScheduleError>,
    trade: Date,
    maturity: Date,
    expected_collision_index: Option<usize>,
) -> Result<usize, Box<dyn Error>> {
    match outcome {
        Outcome::Plan(coupons) => {
            let plan = result?;
            assert_eq!(plan.coupon_dates().len(), coupons.len());
            assert_eq!(plan.trade_date(), trade);
            assert_eq!(plan.payment_selection_cutoff(), trade.checked_add_days(1)?);
            assert_eq!(plan.unadjusted_maturity(), maturity);
            for (actual, wanted) in plan.coupon_dates().iter().zip(coupons) {
                assert_eq!(actual.unadjusted_start(), wanted.raw_start);
                assert_eq!(actual.unadjusted_end(), wanted.raw_end);
                assert_eq!(actual.payment_date(), wanted.payment);
                assert_eq!(actual.accrual_start(), wanted.start);
                assert_eq!(actual.accrual_end(), wanted.end);
                assert_eq!(actual.accrual_day_count(), wanted.days);
                assert_eq!(
                    actual.accrual_end_inclusion(),
                    if wanted.include_end {
                        EndpointInclusion::Include
                    } else {
                        EndpointInclusion::Exclude
                    }
                );
            }
            Ok(0)
        }
        Outcome::NoEligible { final_payment } => {
            assert!(
                matches!(&result, Err(ScheduleError::NoEligibleCdsPayment {
                trade_date, payment_selection_cutoff, unadjusted_maturity, final_payment: paid, ..
            }) if *trade_date == trade && ordinal(*payment_selection_cutoff) == ordinal(trade) + 1
                && *unadjusted_maturity == maturity && paid == final_payment),
                "{result:?}"
            );
            Ok(1)
        }
        Outcome::NoBusinessWeekday => {
            assert!(
                matches!(
                    &result,
                    Err(ScheduleError::CdsCalendar {
                        phase: "cutoff quarter adjustment",
                        cause: CalendarError::NoBusinessWeekday,
                        ..
                    })
                ),
                "{result:?}"
            );
            Ok(2)
        }
        Outcome::Collision {
            previous,
            current,
            previous_paid,
            current_paid,
        } => {
            assert!(
                matches!(&result, Err(ScheduleError::CdsAdjustedOrder {
                index, previous_contractual, contractual, previous_adjusted, adjusted, ..
            }) if previous_contractual == previous && contractual == current
                && previous_adjusted == previous_paid && adjusted == current_paid
                && expected_collision_index.is_none_or(|wanted| wanted == *index)),
                "{result:?}"
            );
            Ok(3)
        }
    }
}

#[test]
fn full_primary_five_coupon_table_distinguishes_ordinary_and_final_accrual()
-> Result<(), Box<dyn Error>> {
    // Standard CDS Examples: trade 2009-02-20, maturity 2010-03-20.
    // The preceding 2008-12-20 adjusts to December 22. December 21, 2009
    // is an ordinary exclusive accrual end; only March 20, 2010 is included.
    let raw = [
        "2008-12-20",
        "2009-03-20",
        "2009-06-20",
        "2009-09-20",
        "2009-12-20",
        "2010-03-20",
    ]
    .map(d);
    let paid = [
        "2008-12-22",
        "2009-03-20",
        "2009-06-22",
        "2009-09-21",
        "2009-12-21",
        "2010-03-22",
    ]
    .map(d);
    let day_counts = [88_u32, 94, 91, 91, 90];
    for regime in REGIMES {
        let plan = CdsDatePlan::standard_following(
            d("2009-02-20"),
            d("2010-03-20"),
            ScheduleCalendar::Plain(Calendar::weekends_only()),
            regime,
            5,
        )?;
        assert_eq!(plan.coupon_dates().len(), 5);
        assert_eq!(plan.payment_selection_cutoff(), d("2009-02-21"));
        for (index, coupon) in plan.coupon_dates().iter().enumerate() {
            assert_eq!(coupon.unadjusted_start(), raw[index]);
            assert_eq!(coupon.unadjusted_end(), raw[index + 1]);
            assert_eq!(coupon.payment_date(), paid[index + 1]);
            assert_eq!(coupon.accrual_start(), paid[index]);
            let end = if index == 4 { raw[5] } else { paid[index + 1] };
            assert_eq!(coupon.accrual_end(), end);
            assert_eq!(
                coupon.accrual_end_inclusion(),
                if index == 4 {
                    EndpointInclusion::Include
                } else {
                    EndpointInclusion::Exclude
                }
            );
            assert_eq!(coupon.accrual_day_count(), day_counts[index]);
            assert_eq!(
                i64::from(day_counts[index]),
                ordinal(end) - ordinal(paid[index]) + i64::from(index == 4)
            );
        }
        assert_eq!(
            plan.coupon_dates()
                .iter()
                .map(|coupon| coupon.accrual_day_count())
                .sum::<u32>(),
            454
        );
    }
    Ok(())
}

#[test]
fn all_128_weekday_masks_have_exact_independent_quarter_and_cutoff_outcomes()
-> Result<(), Box<dyn Error>> {
    let anchors = [
        (1899, 12),
        (1900, 3),
        (1900, 6),
        (1900, 9),
        (1999, 12),
        (2000, 3),
        (2000, 6),
        (2000, 9),
        (2099, 12),
        (2100, 3),
        (2100, 6),
        (2100, 9),
        (2026, 3),
        (2026, 6),
        (2026, 9),
        (2026, 12),
    ];
    let mut outcomes = [0_u32; 4];
    for mask in 0_u8..=127 {
        let model = Model::mask(mask);
        let calendar = model.calendar();
        for (year, month) in anchors {
            let roll = Date::from_ymd(year, month, 20)?;
            let longer_maturity = Date::from_ymd(year + 1, month, 20)?;
            let requests = [
                (roll.checked_add_days(-1)?, longer_maturity),
                (roll, longer_maturity),
                (roll.checked_add_days(1)?, longer_maturity),
                (roll.checked_add_days(-1)?, roll),
            ];
            for (trade, maturity) in requests {
                let oracle = expected(&model, trade, maturity)?;
                for regime in REGIMES {
                    let actual = CdsDatePlan::standard_following(
                        trade,
                        maturity,
                        ScheduleCalendar::Plain(calendar.clone()),
                        regime,
                        8,
                    );
                    outcomes[compare(&oracle, actual, trade, maturity, None)?] += 1;
                }
            }
        }
    }
    // For each maturity weekday, exactly 64 masks leave it business; the
    // other 64 contain 63 usable calendars and the all-seven-closed mask.
    assert_eq!(outcomes, [14_208, 2_048, 128, 0]);
    assert_eq!(outcomes.iter().sum::<u32>(), 16_384);
    Ok(())
}

#[test]
fn explicit_holidays_business_overrides_and_joint_truth_tables_preserve_collision_roles()
-> Result<(), Box<dyn Error>> {
    let shifted = Model::mask(0).close(d("2026-03-20"), d("2026-03-25"))?;
    let initial_collision = Model::mask(0).close(d("2025-12-20"), d("2026-03-25"))?;
    let later_collision = Model::mask(0).close(d("2026-06-20"), d("2026-09-25"))?;
    let weekends = Model::mask(96);
    let null = Model::mask(0);
    let mut cases = vec![
        (
            shifted.clone(),
            shifted.calendar(),
            d("2026-03-22"),
            d("2026-12-20"),
            None,
        ),
        (
            shifted.clone().open(d("2026-03-23")),
            shifted.calendar().without_holiday(d("2026-03-23")),
            d("2026-03-22"),
            d("2026-12-20"),
            None,
        ),
        (
            initial_collision.clone(),
            initial_collision.calendar(),
            d("2026-03-22"),
            d("2026-12-20"),
            Some(0),
        ),
        (
            initial_collision.clone().open(d("2025-12-25")),
            initial_collision
                .calendar()
                .without_holiday(d("2025-12-25")),
            d("2026-03-22"),
            d("2026-12-20"),
            None,
        ),
        (
            later_collision.clone(),
            later_collision.calendar(),
            d("2026-04-01"),
            d("2026-12-20"),
            Some(1),
        ),
        (
            later_collision.clone().open(d("2026-06-21")),
            later_collision.calendar().without_holiday(d("2026-06-21")),
            d("2026-04-01"),
            d("2026-12-20"),
            None,
        ),
        (
            weekends.clone(),
            weekends.calendar(),
            d("2026-06-19"),
            d("2026-06-20"),
            None,
        ),
        (
            weekends.clone().open(d("2026-06-20")),
            weekends.calendar().without_holiday(d("2026-06-20")),
            d("2026-06-19"),
            d("2026-06-20"),
            None,
        ),
    ];
    for rule in [JointRule::JoinHolidays, JointRule::JoinBusinessDays] {
        cases.push((
            initial_collision.joint(&null, rule),
            Calendar::joint(vec![initial_collision.calendar(), null.calendar()], rule)?,
            d("2026-03-22"),
            d("2026-12-20"),
            (rule == JointRule::JoinHolidays).then_some(0),
        ));
        cases.push((
            later_collision.joint(&weekends, rule),
            Calendar::joint(vec![later_collision.calendar(), weekends.calendar()], rule)?,
            d("2026-04-01"),
            d("2026-12-20"),
            (rule == JointRule::JoinHolidays).then_some(1),
        ));
    }
    assert_eq!(cases.len(), 12);
    let mut outcomes = [0_u32; 4];
    for (model, calendar, trade, maturity, collision_index) in cases {
        let oracle = expected(&model, trade, maturity)?;
        for regime in REGIMES {
            let actual = CdsDatePlan::standard_following(
                trade,
                maturity,
                ScheduleCalendar::Plain(calendar.clone()),
                regime,
                8,
            );
            outcomes[compare(&oracle, actual, trade, maturity, collision_index)?] += 1;
        }
    }
    assert_eq!(outcomes, [14, 2, 0, 8]);
    Ok(())
}
