//! Seeded financial invariants and concurrent reuse of immutable quotes.
//!
//! Normal-domain properties exercise relationships rather than reproducing
//! scalar fixture formulas. Every failure prints its deterministic case seed.
//! Elementary calls use the conditioned E3/E4 policy; exact assertions are
//! limited to declared input/context copies and explicit identity paths.

mod support;

use sabiquant::calendar::{Calendar, JointRule, TargetCalendar};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, ThirtyE360Isda,
};
use sabiquant::rates::{
    AccrualConvention, Compounding, CompoundingMethod, InterestRate, RateError,
};
use sabiquant::time::{Date, Frequency};
use std::error::Error;
use std::sync::Arc;
use support::float::{Tolerance, assert_close};
use support::prng;

const UNIT_ROUNDOFF: f64 = f64::EPSILON / 2.0;

fn all_compoundings() -> Result<Vec<Compounding>, RateError> {
    let mut result = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_some() {
            result.extend([
                Compounding::compounded(frequency)?,
                Compounding::simple_then_compounded(frequency)?,
                Compounding::compounded_then_simple(frequency)?,
            ]);
        }
    }
    Ok(result)
}

// Initial E3 k=8 per evaluation. Comparing two independently evaluated factors
// combines both budgets; basic product/addition roundings are stated separately.
fn factor_tolerance(left: f64, right: f64, evaluations: f64) -> Tolerance {
    let conditioning = 1.0 + left.ln().abs().max(right.ln().abs());
    Tolerance::Mixed {
        absolute: f64::from_bits(1),
        relative: 8.0 * UNIT_ROUNDOFF * evaluations * conditioning,
    }
}

#[test]
fn twenty_thousand_replayable_cases_obey_financial_relationships_in_normal_domains()
-> Result<(), RateError> {
    let methods = all_compoundings()?;
    let mut visited = [false; 32];
    prng::check(
        "rates normal-domain financial invariants",
        0x5AB1_0004_2026_1011,
        20_000,
        |rng| {
            let index = usize::try_from(rng.below(32)).map_err(|error| error.to_string())?;
            visited[index] = true;
            let method = methods[index];
            let target = *rng.choose(&methods);
            let ordinary_rate = -0.2 + 0.4 * rng.next_f64();
            let rate = match rng.below(8) {
                0 => 0.0,
                1 => -0.0,
                2 => 1e-20,
                3 => -1e-20,
                _ => ordinary_rate,
            };
            let time = 1e-6 + rng.next_f64();
            let later = time + rng.next_f64();
            let context = *rng.choose(&DayCount::ALL);
            let quote =
                InterestRate::new(rate, context, method).map_err(|error| error.to_string())?;
            let factor = quote
                .compound_factor(time)
                .map_err(|error| error.to_string())?;
            let discount = quote
                .discount_factor(time)
                .map_err(|error| error.to_string())?;
            let future_factor = quote
                .compound_factor(later)
                .map_err(|error| error.to_string())?;
            if !(factor > 0.0 && discount > 0.0 && factor.is_finite() && discount.is_finite()) {
                return Err("nonpositive or nonfinite factor in normal domain".to_owned());
            }
            if (rate > 0.0 && factor < 1.0) || (rate < 0.0 && factor > 1.0) {
                return Err(format!(
                    "rounded factor violates rate sign: r={rate:?},C={factor:?}"
                ));
            }
            let time_monotonic = if rate >= 0.0 {
                future_factor >= factor
            } else {
                future_factor <= factor
            };
            if !time_monotonic {
                factor_tolerance(future_factor, factor, 2.0).check(future_factor, factor)?;
            }
            let higher_rate = rate + rng.next_f64() * 0.2;
            let higher = InterestRate::new(higher_rate, context, method)
                .map_err(|error| error.to_string())?;
            let higher_factor = higher
                .compound_factor(time)
                .map_err(|error| error.to_string())?;
            if higher_factor < factor {
                factor_tolerance(higher_factor, factor, 2.0).check(higher_factor, factor)?;
            }
            let reciprocal_bound =
                2.0 * UNIT_ROUNDOFF + 16.0 * UNIT_ROUNDOFF * (1.0 + factor.ln().abs());
            Tolerance::Mixed {
                absolute: reciprocal_bound,
                relative: 0.0,
            }
            .check(factor * discount, 1.0)?;
            let converted = quote
                .equivalent_rate(target, time)
                .map_err(|error| error.to_string())?;
            if converted.accrual() != quote.accrual() {
                return Err("equivalence discarded accrual context".to_owned());
            }
            let converted_factor = converted
                .compound_factor(time)
                .map_err(|error| error.to_string())?;
            factor_tolerance(converted_factor, factor, 2.0).check(converted_factor, factor)?;
            let round_trip = converted
                .equivalent_rate(method, time)
                .map_err(|error| error.to_string())?;
            let rate_bound = 16.0 * UNIT_ROUNDOFF * (rate.abs() + 1.0 / time);
            Tolerance::Mixed {
                absolute: rate_bound,
                relative: 0.0,
            }
            .check(round_trip.rate(), rate)?;
            if matches!(
                method.method(),
                CompoundingMethod::Compounded | CompoundingMethod::Continuous
            ) {
                let first = time / 2.0;
                let second = time - first;
                let composed = quote
                    .compound_factor(first)
                    .map_err(|error| error.to_string())?
                    * quote
                        .compound_factor(second)
                        .map_err(|error| error.to_string())?;
                // Three elementary evaluations plus one multiplication.
                let bound = 25.0 * UNIT_ROUNDOFF * (1.0 + factor.ln().abs());
                Tolerance::Mixed {
                    absolute: f64::from_bits(1),
                    relative: bound,
                }
                .check(composed, factor)?;
                if matches!(
                    target.method(),
                    CompoundingMethod::Compounded | CompoundingMethod::Continuous
                ) {
                    let converted_later = quote
                        .equivalent_rate(target, later)
                        .map_err(|error| error.to_string())?;
                    Tolerance::Mixed {
                        absolute: (16.0 * UNIT_ROUNDOFF * converted.rate().abs())
                            .max(f64::from_bits(1)),
                        relative: 0.0,
                    }
                    .check(converted_later.rate(), converted.rate())?;
                }
            }
            let first_amount = (rng.next_f64() - 0.5) * 2_000_000.0;
            let second_amount = if rng.below(2) == 0 {
                -first_amount + (rng.next_f64() - 0.5)
            } else {
                (rng.next_f64() - 0.5) * 2_000_000.0
            };
            let first_pv = quote
                .present_value(first_amount, time)
                .map_err(|error| error.to_string())?;
            let second_pv = quote
                .present_value(second_amount, time)
                .map_err(|error| error.to_string())?;
            let combined = quote
                .present_value(first_amount + second_amount, time)
                .map_err(|error| error.to_string())?;
            // Net-cash-flow cancellation is conditioned on gross components,
            // rather than the possibly tiny difference of their present values.
            let linearity_bound = (16.0 * UNIT_ROUNDOFF * (1.0 + factor.ln().abs())
                + 4.0 * UNIT_ROUNDOFF)
                * discount
                * (first_amount.abs() + second_amount.abs());
            Tolerance::Mixed {
                absolute: linearity_bound,
                relative: 0.0,
            }
            .check(combined, first_pv + second_pv)?;
            let positive = quote
                .present_value(100.0, time)
                .map_err(|error| error.to_string())?;
            if (rate >= 0.0 && positive > 100.0) || (rate <= 0.0 && positive < 100.0) {
                return Err(format!(
                    "PV violates signed growth direction: r={rate:?},PV={positive:?}"
                ));
            }
            if quote
                .compound_factor(0.0)
                .map_err(|error| error.to_string())?
                .to_bits()
                != 1.0_f64.to_bits()
            {
                return Err("zero-horizon factor identity failed".to_owned());
            }
            if quote
                .present_value(-0.0, time)
                .map_err(|error| error.to_string())?
                .to_bits()
                != 0
            {
                return Err("cash-flow signed zero was not canonical".to_owned());
            }
            Ok(())
        },
    );
    assert!(visited.into_iter().all(|value| value));
    Ok(())
}

#[test]
fn semigroup_is_not_asserted_for_simple_or_hybrid_growth() -> Result<(), RateError> {
    let simple = InterestRate::new(0.125, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    assert_eq!(simple.compound_factor(2.0)?.to_bits(), 1.25_f64.to_bits());
    assert_eq!(
        (simple.compound_factor(1.0)? * simple.compound_factor(1.0)?).to_bits(),
        1.265_625_f64.to_bits()
    );
    for compounding in [
        Compounding::simple_then_compounded(Frequency::Annual)?,
        Compounding::compounded_then_simple(Frequency::Annual)?,
    ] {
        let quote = InterestRate::new(0.125, DayCount::Actual365Fixed, compounding)?;
        let combined = quote.compound_factor(2.0)?;
        let components = quote.compound_factor(0.5)? * quote.compound_factor(1.5)?;
        assert!((combined - components).abs() > 0.001);
    }
    Ok(())
}

#[derive(Debug)]
struct Observation {
    values: [f64; 4],
    zero_factor: f64,
    same_date_amount: f64,
    accrual: AccrualConvention,
}

fn observe(
    quote: &InterestRate,
    index: usize,
    start: Date,
    end: Date,
) -> Result<Observation, RateError> {
    let scale = u32::try_from(index % 8).map(f64::from).unwrap_or(0.0);
    let time = 0.1 + scale / 8.0;
    let cloned = quote.clone();
    Ok(Observation {
        values: [
            quote.compound_factor(time)?,
            cloned.discount_factor(time)?,
            quote.compound_factor_between(start, end)?,
            cloned.present_value_between(-123.0, start, end)?,
        ],
        zero_factor: cloned.compound_factor(0.0)?,
        same_date_amount: quote.present_value_between(-123.0, start, start)?,
        accrual: cloned.accrual().clone(),
    })
}

#[test]
fn concurrent_clone_and_reverse_order_reuse_preserves_all_owned_contexts()
-> Result<(), Box<dyn Error>> {
    let start = Date::from_ymd(2026, 1, 1)?;
    let end = Date::from_ymd(2026, 1, 8)?;
    let period = DeterminationPeriod::new(start, Date::from_ymd(2026, 7, 1)?)?;
    let guarded = TargetCalendar::new()
        .without_holiday(Date::from_ymd(2026, 1, 1)?)
        .joint_with(vec![Calendar::weekends_only()], JointRule::JoinHolidays)?;
    let mut contexts: Vec<AccrualConvention> = DayCount::ALL.into_iter().map(Into::into).collect();
    contexts.extend([
        ThirtyE360Isda::new(end).into(),
        ActualActualIcma::new(vec![period], Frequency::Semiannual)?.into(),
        Business252::with_target(guarded).into(),
    ]);
    let mut quotes = Vec::new();
    for context in contexts {
        for method in all_compoundings()? {
            quotes.push(InterestRate::new(
                if quotes.len() % 2 == 0 { -0.005 } else { 0.05 },
                context.clone(),
                method,
            )?);
        }
    }
    assert_eq!(quotes.len(), 256);
    let baseline: Vec<_> = quotes
        .iter()
        .enumerate()
        .map(|(index, quote)| observe(quote, index, start, end))
        .collect::<Result<_, _>>()?;
    let shared = Arc::new(quotes);
    let mut workers = Vec::new();
    for worker in 0..8 {
        let quotes = Arc::clone(&shared);
        #[expect(
            clippy::disallowed_methods,
            reason = "test harness intentionally shares immutable quotes across threads to check state independence"
        )]
        let task = std::thread::spawn(move || -> Result<Vec<(usize, Observation)>, RateError> {
            let mut observations = Vec::new();
            for round in 0..4 {
                for offset in 0..quotes.len() {
                    let index = if (worker + round) % 2 == 0 {
                        offset
                    } else {
                        quotes.len() - 1 - offset
                    };
                    observations.push((index, observe(&quotes[index], index, start, end)?));
                }
            }
            Ok(observations)
        });
        workers.push(task);
    }
    for worker in workers {
        let observations = worker
            .join()
            .map_err(|_| "immutable-quote worker panicked")??;
        for (index, actual) in observations {
            let expected = &baseline[index];
            assert_eq!(actual.accrual, expected.accrual);
            for (actual, expected) in actual.values.into_iter().zip(expected.values) {
                let absolute =
                    16.0 * UNIT_ROUNDOFF * expected.abs() * (1.0 + expected.abs().ln().abs());
                assert_close(
                    actual,
                    expected,
                    Tolerance::Mixed {
                        absolute,
                        relative: 0.0,
                    },
                );
            }
            assert_eq!(actual.zero_factor.to_bits(), 1.0_f64.to_bits());
            assert_eq!(actual.same_date_amount.to_bits(), (-123.0_f64).to_bits());
        }
    }
    Ok(())
}
