//! Civil-envelope minimal references and exact rational accrual expectations.

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar};
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, ScheduleBuilder, ScheduleCalendar, ScheduleError,
};
use sabiquant::time::{Date, Frequency, Period};
use std::error::Error;
use support::fixture::Fixture;
use support::float::ulp_distance;

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

// Exact integer overlaps with pre-reduced checked products; no f64 division
// or compensated sum is shared with the production ICMA computation.
fn exact_fraction(
    dates: &[Date],
    events: u32,
    start: Date,
    end: Date,
) -> Result<(i128, i128), Box<dyn Error>> {
    let low = start.min(end);
    let high = start.max(end);
    let (mut n, mut d) = (0_i128, 1_i128);
    for pair in dates.windows(2) {
        let a = low.max(pair[0]);
        let b = high.min(pair[1]);
        if a >= b {
            continue;
        }
        let mut tn = i128::from(b.days_since(a));
        let mut td = i128::from(events)
            .checked_mul(i128::from(pair[1].days_since(pair[0])))
            .ok_or("term overflow")?;
        let g = gcd(tn, td);
        tn /= g;
        td /= g;
        let g = gcd(d, td);
        let combined = n
            .checked_mul(td / g)
            .and_then(|left| {
                tn.checked_mul(d / g)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or("numerator overflow")?;
        let reduce = gcd(combined, g);
        n = combined / reduce;
        d = (d / g)
            .checked_mul(td / reduce)
            .ok_or("denominator overflow")?;
    }
    Ok((
        if start > end {
            n.checked_neg().ok_or("negation overflow")?
        } else {
            n
        },
        d,
    ))
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records measured oracle accuracy with --nocapture"
)]
fn independent_civil_fraction_oracle_checks_every_minimal_period_and_query()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/icma_oracles.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 36_092);
    assert_eq!(
        fixture.header("reference"),
        Some("oracle:Python datetime/calendar/Fraction")
    );
    assert!(
        fixture
            .header("reference-source")
            .unwrap()
            .contains("minimal bracketing hull")
    );
    assert!(
        fixture
            .header("tolerance")
            .unwrap()
            .contains("TOL-E2 4 ULP")
    );
    let mut maximum_ulp = 0;
    let mut zero = 0;
    let mut reversed = 0;
    let mut all_periods = 0;
    for row in fixture.records() {
        let start: Date = row.parse("start")?;
        let end: Date = row.parse("end")?;
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let rule: DateGenerationRule = row.parse("rule")?;
        let eom = row.parse("eom")?;
        let b = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?;
        let schedule = if row.text("kind")? == "automatic" {
            b.build_recurrent(tenor, rule, row.parse("stub")?, eom, 16)?
        } else {
            assert_eq!(row.text("kind")?, "explicit");
            let first = match row.text("first")? {
                "" => None,
                text => Some(text.parse()?),
            };
            let penultimate = match row.text("penultimate")? {
                "" => None,
                text => Some(text.parse()?),
            };
            b.build_explicit(
                tenor,
                rule,
                ExplicitStubDates::new(first, penultimate)?,
                eom,
                16,
            )?
        };
        assert_eq!(
            schedule.lattice_anchor(),
            Some(row.parse::<Date>("anchor")?),
            "{}",
            row.id()
        );
        let frequency: Frequency = row.parse("frequency")?;
        let events: u32 = row.parse("events")?;
        assert_eq!(frequency.events_per_year(), Some(events));
        let limit: u32 = row.parse("limit")?;
        let context = schedule
            .build_icma_context(frequency, limit)
            .unwrap_or_else(|error| panic!("{}: {error}", row.id()));
        let dates: Vec<Date> = row
            .text("references")?
            .split(';')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        assert_eq!(dates.len() - 1, usize::try_from(limit)?);
        assert_eq!(context.periods().len(), usize::try_from(limit)?);
        for (period, pair) in context.periods().iter().zip(dates.windows(2)) {
            assert_eq!(
                (period.start(), period.end()),
                (pair[0], pair[1]),
                "{}",
                row.id()
            );
        }
        assert_eq!(
            (context.coverage_start(), context.coverage_end()),
            (dates[0], dates[dates.len() - 1])
        );
        assert_eq!(context.frequency(), frequency);
        assert_eq!(context, schedule.build_icma_context(frequency, limit + 10)?);
        if limit > 1 {
            assert!(
                matches!(schedule.build_icma_context(frequency,limit-1),
                Err(ScheduleError::IcmaPeriodLimitExceeded{maximum,required_at_least,..})
                if maximum==limit-1 && required_at_least==limit),
                "{} cap",
                row.id()
            );
        }
        let a: Date = row.parse("query_start")?;
        let z: Date = row.parse("query_end")?;
        let expected_exact = (
            row.parse::<i128>("numerator")?,
            row.parse::<i128>("denominator")?,
        );
        assert_eq!(
            exact_fraction(&dates, events, a, z)?,
            expected_exact,
            "{} rational",
            row.id()
        );
        let expected: f64 = row.parse("fraction")?;
        let actual = context.year_fraction(a, z)?;
        assert!(actual.is_finite());
        let distance = ulp_distance(actual, expected).unwrap();
        assert!(
            distance <= 4,
            "{} actual {actual} expected {expected} ULP {distance}",
            row.id()
        );
        maximum_ulp = maximum_ulp.max(distance);
        if a == z {
            assert_eq!(actual.to_bits(), 0.0_f64.to_bits());
            zero += 1;
        }
        if a > z {
            reversed += 1;
        }
        all_periods += context.periods().len();
    }
    assert!(maximum_ulp <= 4);
    assert_eq!((zero, reversed, all_periods), (4_511, 4_512, 147_388));
    println!(
        "generated ICMA oracle: 36,092 rows; max {maximum_ulp} ULP; 147,388 period comparisons"
    );
    Ok(())
}
