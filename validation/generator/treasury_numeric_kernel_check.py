#!/usr/bin/env python3
"""Attested exact-copy private Treasury kernel check (stdlib only).

Appends test-only observation to two modules in a create-new scratch crate.
Every frozen primitive row reaches actual production helpers; no repository
Rust, frozen fixture, oracle generator, public API or package is modified.
Certificate-only proof columns are not reported as runtime instrumentation.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
MODULE = Path('src/bonds/treasury/numeric.rs')
WIDE_MODULE = Path('src/amounts/uint.rs')
FIXTURE = Path('validation/fixtures/bonds/treasury-numeric-oracle.fixture')
GENERATOR = Path('validation/generator/treasury_numeric_oracles.py')
CONTRACT = Path('docs/M8_US_TREASURY_CONTRACT.md')
CASH_CONTRACT = Path('docs/M8_US_TREASURY_CASH_CONTRACT.md')
PINS = {'validation/fixtures/bonds/treasury-numeric-oracle.fixture': 'b24208d7150c65a0ad3b5714f588cfbbd3b6481a511ff086fa94ade132464adc', 'validation/generator/treasury_numeric_oracles.py': '4915101db4c1040702e820570cbe117ceddc72fd55413a7aedd91397256ede54', 'docs/M8_US_TREASURY_CONTRACT.md': '50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93', 'docs/M8_US_TREASURY_CASH_CONTRACT.md': '25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585'}
TEST_NAME = 'bonds::treasury::numeric::validation_treasury_kernel::all_frozen_numeric_rows_match_actual_helpers'
SCHEMA = ['id', 'family', 'profile', 'case', 'reachability', 'shape', 'op', 'dates', 'eom', 'y_bits', 'r', 's', 'j', 'c_coef', 'c_scale', 'basis', 'tie', 'amount_num', 'amount_den', 'low_point', 'factor_hat', 'pv_bits', 'measure', 'dirty_bits', 'target_kind', 'target_bits', 'ai_bits', 'remaining_periods', 'status', 'expected_bits', 'alternate_bits', 'tie_flag', 'carry_flag', 'b_interval', 'q_interval', 'delta_bits', 'qhat_bits', 'seed_bits', 'factor_lo', 'factor_hi', 'weights', 'width_ok', 'relation', 'straddle', 'containment', 'tree_alt', 'peak_bits', 'ext_value', 'num_bits', 'den_bits', 'e_diff', 'bundle_count', 'positive_count', 'first_fail', 'dirty_status', 'dirty_bits_out', 'sum_bits', 'sum_fnv64', 'p1_status', 'p1_bits', 'p2_status', 'p2_bits', 'moment_status', 'moment_bits', 'modified_status', 'modified_bits', 'macaulay_status', 'macaulay_bits', 'convexity_status', 'convexity_bits', 'pieces', 'raw_ai', 'published_count', 'raw_ai_bits', 'published_bits', 'scale', 'residual_bits', 'amount_bits', 'dirty_target_bits']
WIDE_ACCESSOR = r'''
// BEGIN TEMPORARY TREASURY WORD OBSERVATION (TEST ONLY)
#[cfg(test)]
mod validation_treasury_observation {
    use super::Wide;
    impl<const N: usize> Wide<N> {
        /// Borrows actual retained limbs for supplemental primitive validation.
        pub(crate) fn validation_treasury_words(&self) -> &[u64; N] { &self.0 }
    }
}
// END TEMPORARY TREASURY WORD OBSERVATION
'''
ADAPTER = r'''
// BEGIN TEMPORARY TREASURY KERNEL ADAPTER (TEST ONLY)
#[cfg(test)]
mod validation_treasury_kernel {
use super::*;
use super::super::{UsTreasury, TreasuryDailyBasis, TreasurySourcePolicy, TreasuryTieRule};
use crate::amounts::DecimalAmount;
use crate::calendar::EndOfMonth;
use crate::error::{NonFinite, ValueError};
use crate::support::fixture::{Fixture, Record};
use std::collections::BTreeMap;

type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;
const PHASES: [&str; 3] = ["price derivative", "price second derivative", "time weighted amount"];
const COLUMNS: [&str; 79] = ["id", "family", "profile", "case", "reachability", "shape", "op", "dates", "eom", "y_bits", "r", "s", "j", "c_coef", "c_scale", "basis", "tie", "amount_num", "amount_den", "low_point", "factor_hat", "pv_bits", "measure", "dirty_bits", "target_kind", "target_bits", "ai_bits", "remaining_periods", "status", "expected_bits", "alternate_bits", "tie_flag", "carry_flag", "b_interval", "q_interval", "delta_bits", "qhat_bits", "seed_bits", "factor_lo", "factor_hi", "weights", "width_ok", "relation", "straddle", "containment", "tree_alt", "peak_bits", "ext_value", "num_bits", "den_bits", "e_diff", "bundle_count", "positive_count", "first_fail", "dirty_status", "dirty_bits_out", "sum_bits", "sum_fnv64", "p1_status", "p1_bits", "p2_status", "p2_bits", "moment_status", "moment_bits", "modified_status", "modified_bits", "macaulay_status", "macaulay_bits", "convexity_status", "convexity_bits", "pieces", "raw_ai", "published_count", "raw_ai_bits", "published_bits", "scale", "residual_bits", "amount_bits", "dirty_target_bits"];
struct Row<'a>(Record<'a>);
impl<'a> Row<'a> {
    fn text(&self, name: &str) -> Check<&'a str> { Ok(self.0.text(name)?) }
    fn id(&self) -> &'a str { self.0.id() }
    fn integer<T: std::str::FromStr>(&self, name: &str) -> Check<T>
    where T::Err: std::error::Error + 'static { Ok(self.text(name)?.parse()?) }
    fn real(&self, name: &str) -> Check<f64> { Ok(self.0.real_bits(name)?) }
}
fn bits(text: &str) -> Check<u64> {
    let text = text.strip_prefix("0x").ok_or("bits prefix")?;
    assert_eq!(text.len(), 16);
    Ok(u64::from_str_radix(text, 16)?)
}
fn wide<const N: usize>(text: &str, radix: u32) -> Check<Wide<N>> {
    assert!(!text.is_empty());
    let mut out = Wide::ZERO;
    for digit in text.chars() {
        let value = digit.to_digit(radix).ok_or("invalid integer digit")?;
        out = out.checked_mul_small(radix).and_then(|value0| value0.checked_add(Wide::from_u128(u128::from(value))))
            .ok_or("integer exceeds fixed width")?;
    }
    Ok(out)
}
fn point(text: &str) -> Check<Endpoint> {
    let (sig, exp) = text.split_once(':').ok_or("point syntax")?;
    assert_eq!(sig.len(), 64);
    let significand = wide(sig, 16)?;
    assert_eq!(significand.bits(), 256);
    Ok(Endpoint { significand, exponent: exp.parse()? })
}
fn extended(text: &str) -> Check<Extended> {
    let (sig, exp) = text.split_once(':').ok_or("extended syntax")?;
    let significand = u64::from_str_radix(sig, 16)?;
    assert!((1_u64 << 52..1_u64 << 53).contains(&significand));
    Ok(Extended { significand, exponent: exp.parse()? })
}
fn interval(text: &str) -> Check<Endpoints> {
    let (low, high) = text.split_once('|').ok_or("interval syntax")?;
    Ok(Endpoints { low: point(low)?, high: point(high)? })
}
fn expected_bits(row: &Row<'_>, name: &str, actual: f64) -> Check {
    assert_eq!(actual.to_bits(), bits(row.text(name)?)?, "{} {name}", row.id());
    Ok(())
}
fn source(row: &Row<'_>) -> Check<SourceRational> {
    let numerator = wide(row.text("amount_num")?, 10)?;
    let denominator = wide(row.text("amount_den")?, 10)?;
    assert!(!denominator.is_zero());
    Ok(SourceRational { numerator, denominator })
}
fn compare_ratio(actual: SourceRational, expected: SourceRational, id: &str) -> Check {
    let first = actual.numerator.checked_mul(expected.denominator).ok_or("source comparison width")?;
    let second = expected.numerator.checked_mul(actual.denominator).ok_or("source comparison width")?;
    assert_eq!(first, second, "{id} exact rational");
    Ok(())
}
fn policy(row: &Row<'_>) -> Check<TreasurySourcePolicy> {
    let basis = match row.text("basis")? { "-" | "ExactRatio" => TreasuryDailyBasis::ExactRatio,
        "NineDecimal" => TreasuryDailyBasis::NineDecimal, _ => return Err("unknown daily basis".into()) };
    let tie = match row.text("tie")? { "-" | "NearestTiesEven" => TreasuryTieRule::NearestTiesEven,
        "NearestTiesAwayFromZero" => TreasuryTieRule::NearestTiesAwayFromZero, _ => return Err("unknown tie rule".into()) };
    Ok(TreasurySourcePolicy::new(basis, tie))
}
fn instrument(row: &Row<'_>) -> Check<(UsTreasury, Date)> {
    assert_eq!(row.text("reachability")?, "dated-financial");
    let values = row.text("dates")?.split('|').map(str::parse::<Date>).collect::<Result<Vec<_>, _>>()?;
    let [q0, dated, issue, first, settlement, maturity]: [Date; 6] = values.try_into().map_err(|_| "six dates required")?;
    let eom = match row.text("eom")? { "Ignore" => EndOfMonth::Ignore, "Preserve" => EndOfMonth::Preserve,
        _ => return Err("unknown EOM".into()) };
    let months = 12 * (maturity.year() - q0.year()) + i32::try_from(maturity.month())? - i32::try_from(q0.month())?;
    assert!(months >= 6 && months % 6 == 0);
    let count = usize::try_from(months / 6)?;
    assert!(count < 20_000);
    let mut cycle = Vec::new();
    for index in (0..=count).rev() {
        let date = maturity.checked_add_months(-6 * i64::try_from(index)?)?;
        cycle.push(if eom == EndOfMonth::Preserve && maturity.is_end_of_month() { date.last_day_of_month() } else { date });
    }
    assert_eq!(cycle[0], q0);
    let coupon = DecimalAmount::new(row.integer("c_coef")?, row.integer("c_scale")?)?;
    let treasury = UsTreasury::new(dated, issue, maturity, coupon, first, cycle, eom, policy(row)?, 20_000)?;
    assert_eq!(format!("{:?}", treasury.first_period()), row.text("shape")?, "{} shape", row.id());
    Ok((treasury, settlement))
}
fn coordinate(row: &Row<'_>, plan: &TreasuryPricingPlan<'_>) -> Check {
    assert_eq!(plan.r, row.integer("r")?, "{} r", row.id());
    assert_eq!(plan.s, row.integer("s")?, "{} s", row.id());
    Ok(())
}
fn check_value(result: Result<f64, TreasuryError>, status: &str, expected: u64,
               phase: &'static str, quantity: &'static str, index: Option<usize>, id: &str) -> Check {
    match (status, result) {
        ("ok", Ok(value)) => assert_eq!(value.to_bits(), expected, "{id} {phase}"),
        ("overflow+" | "overflow-" | "underflow+" | "underflow-", Err(error)) => {
            assert_eq!(error.kind(), "treasury.value", "{id} kind");
            assert!(error.source().is_some());
            match error {
                TreasuryError::Value { index: actual_index, phase: actual_phase, cause, .. } => {
                    assert_eq!(actual_index, index, "{id} error index");
                    assert_eq!(actual_phase, phase, "{id} error phase");
                    let negative = status.ends_with('-');
                    if status.starts_with("overflow") {
                        assert_eq!(expected, INFINITY | if negative { SIGN } else { 0 });
                        let value = if negative { NonFinite::NegativeInfinity } else { NonFinite::PositiveInfinity };
                        assert_eq!(cause, ValueError::ResultNotFinite { quantity, value }, "{id} cause");
                    } else {
                        assert_eq!(expected, if negative { SIGN } else { 0 });
                        match cause {
                            ValueError::ResultOutOfDomain { quantity: actual_quantity, value, domain, .. } => {
                                assert_eq!(actual_quantity, quantity);
                                assert_eq!(value.to_bits(), expected, "{id} signed underflow");
                                assert_eq!(domain, if negative { Interval::less_than(0.0) } else { Interval::greater_than(0.0) });
                            }
                            other => return Err(format!("{id} wrong underflow {other:?}").into()),
                        }
                    }
                }
                other => return Err(format!("{id} wrong outer error {other:?}").into()),
            }
        }
        (status, actual) => return Err(format!("{id} {phase}: expected {status}/{expected:016x}, got {actual:?}").into()),
    }
    Ok(())
}
fn endpoints(seeded: &Seeds, j: u32, index: usize) -> Check<Endpoints> {
    let denominator = if j == 0 { seeded.q } else {
        let mut power = seeded.b;
        for bit in (0..31 - j.leading_zeros()).rev() {
            power = product(power, power, index)?;
            if j & (1 << bit) != 0 { power = product(power, seeded.b, index)?; }
        }
        product(seeded.q, power, index)?
    };
    Ok(Endpoints { low: reciprocal_endpoint(denominator.high, false, index)?,
        high: reciprocal_endpoint(denominator.low, true, index)? })
}
fn factor_flags(row: &Row<'_>, low: Endpoint, observed: Extended) -> Check {
    let divisor = Wide::<4>::from_u128(1).checked_shl(203).ok_or("discard divisor")?;
    let (q, rem) = low.significand.div_rem(divisor).ok_or("discard division")?;
    let complement = divisor.checked_sub(rem).ok_or("discard remainder")?;
    assert_eq!(row.text("tie_flag")?, if rem == complement { "yes" } else { "no" });
    let carry = observed.exponent != low.exponent + 203;
    assert_eq!(row.text("carry_flag")?, if carry { "yes" } else { "no" });
    assert!(q.bits() == 53);
    Ok(())
}
fn seed_row(row: &Row<'_>) -> Check {
    let y = row.real("y_bits")?;
    if row.text("status")? == "invalid-yield" {
        let expected = validation::within("rate", y, Interval::greater_than(-2.0));
        let error = quote(y).err().ok_or("invalid yield admitted")?;
        assert_eq!(error, TreasuryError::Value { index: None, phase: "yield quote", cause: expected.err().ok_or("invalid-yield fixture admitted")? });
        return Ok(());
    }
    let y = quote(y)?;
    let r = row.integer("r")?;
    let s = row.integer("s")?;
    let actual = seeds(y, r, s)?;
    assert_eq!(actual.b, interval(row.text("b_interval")?)?, "{} b", row.id());
    assert_eq!(actual.q, interval(row.text("q_interval")?)?, "{} q", row.id());
    let (delta, q) = risk_prefix(y, r, s)?;
    expected_bits(row, "delta_bits", delta)?;
    expected_bits(row, "qhat_bits", q)?;
    let (nb, nq) = prefix_numerators(y, r, s, "discount seeds")?;
    assert_eq!(format!("{},{}", nb.bits(), nq.bits()), row.text("seed_bits")?);
    Ok(())
}
fn factor_row(row: &Row<'_>) -> Check {
    let y = quote(row.real("y_bits")?)?;
    let j = row.integer("j")?;
    let seeded = seeds(y, row.integer("r")?, row.integer("s")?)?;
    let actual = endpoints(&seeded, j, 2)?;
    assert_eq!(actual.low, point(row.text("factor_lo")?)?, "{} low", row.id());
    assert_eq!(actual.high, point(row.text("factor_hi")?)?, "{} high", row.id());
    let observed = discount(&seeded, j, 2)?;
    assert_eq!(observed, extended(row.text("ext_value")?)?, "{} factor", row.id());
    factor_flags(row, actual.low, observed)
}
fn ext_row(row: &Row<'_>) -> Check {
    let low = point(row.text("low_point")?)?;
    let actual = factor_observation(low, 3)?;
    assert_eq!(actual, extended(row.text("ext_value")?)?, "{} ext53", row.id());
    factor_flags(row, low, actual)
}
fn pv_row(row: &Row<'_>) -> Check {
    let amount = source(row)?;
    let factor = extended(row.text("factor_hat")?)?;
    let index = if row.text("reachability")? == "dated-financial" {
        let (treasury, settlement) = instrument(row)?;
        let plan = TreasuryPricingPlan::new(&treasury, settlement, 20_000)?;
        coordinate(row, &plan)?;
        let j: u32 = row.integer("j")?;
        let bundle = plan.bundles.iter().find(|bundle| bundle.j == j).ok_or("dated PV bundle missing")?;
        compare_ratio(bundle.amount, amount, row.id())?;
        let seeded = seeds(quote(row.real("y_bits")?)?, plan.r, plan.s)?;
        assert_eq!(discount(&seeded, j, bundle.index)?, factor, "{} dated factor", row.id());
        bundle.index
    } else { 3 };
    check_value(source_pv(amount, factor, index), row.text("status")?, bits(row.text("expected_bits")?)?,
        "source present value", "treasury present value", Some(index), row.id())?;
    if row.text("alternate_bits")? != "-" { assert_ne!(row.text("alternate_bits")?, row.text("expected_bits")?); }
    Ok(())
}
fn ai_row(row: &Row<'_>) -> Check {
    let (treasury, settlement) = instrument(row)?;
    let plan = TreasuryPricingPlan::new(&treasury, settlement, 20_000)?;
    coordinate(row, &plan)?;
    let raw = row.text("raw_ai")?.split_once('/').ok_or("raw AI syntax")?;
    compare_ratio(plan.accrued_source.raw, SourceRational { numerator: wide(raw.0, 10)?, denominator: wide(raw.1, 10)? }, row.id())?;
    assert_eq!(plan.accrued_source.published_count, wide(row.text("published_count")?, 10)?, "{} published count", row.id());
    expected_bits(row, "raw_ai_bits", plan.raw_accrued_per_100())?;
    expected_bits(row, "published_bits", plan.accrued_per_100())?;
    let (first, second) = plan.accrued_source.pieces;
    let mut pieces = format!("{}/{}", first.0, first.1);
    if let Some((days, denominator)) = second { pieces.push_str(&format!(";{days}/{denominator}")); }
    assert_eq!(pieces, row.text("pieces")?, "{} pieces", row.id());
    Ok(())
}
fn term_row(row: &Row<'_>) -> Check {
    let y = quote(row.real("y_bits")?)?;
    let r = row.integer("r")?;
    let s = row.integer("s")?;
    let j = row.integer("j")?;
    let pv = row.real("pv_bits")?;
    let (delta, q) = risk_prefix(y, r, s)?;
    expected_bits(row, "delta_bits", delta)?;
    expected_bits(row, "qhat_bits", q)?;
    let index = if row.text("reachability")? == "dated-financial" {
        let (treasury, settlement) = instrument(row)?;
        let plan = TreasuryPricingPlan::new(&treasury, settlement, 20_000)?;
        coordinate(row, &plan)?;
        let bundle = plan.bundles.iter().find(|bundle| bundle.j == j).ok_or("dated TERM bundle missing")?;
        expected_bits(row, "pv_bits", observe(bundle, &seeds(y, r, s)?)?)?;
        bundle.index
    } else { 5 };
    let terms = risk_terms(pv, delta, q, j, r, s, index)?;
    let term_index = match row.text("measure")? { "p1" => 0, "p2" => 1, "moment" => 2, _ => return Err("unknown measure".into()) };
    assert_eq!(terms[term_index], extended(row.text("ext_value")?)?, "{} complete retained ratio", row.id());
    Ok(())
}
fn fnv_words(value: &RiskWide) -> String {
    let mut state = 0xcbf2_9ce4_8422_2325_u64;
    for word in value.validation_treasury_words() {
        for byte in word.to_le_bytes() { state = (state ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3); }
    }
    format!("{state:016x}")
}
fn reduction_row(row: &Row<'_>) -> Check {
    let (treasury, settlement) = instrument(row)?;
    let plan = TreasuryPricingPlan::new(&treasury, settlement, 20_000)?;
    coordinate(row, &plan)?;
    assert_eq!(plan.n, row.integer("remaining_periods")?);
    assert_eq!(plan.bundles.len(), row.integer("bundle_count")?);
    let y = quote(row.real("y_bits")?)?;
    let actual = plan.risk(y);
    if let Some(status) = row.text("status")?.strip_prefix("pv-") {
        let index = row.integer("first_fail")?;
        let expected = if status.starts_with("overflow") { INFINITY } else { 0 };
        check_value(actual.map(|risk| risk.dirty), status, expected, "source present value", "treasury present value", Some(index), row.id())?;
        return Ok(());
    }
    if row.text("status")? == "sum-overflow" {
        let error = actual.err().ok_or("sum overflow admitted")?;
        assert!(matches!(error, TreasuryError::Sum { phase: "dirty price reduction", .. }));
        assert_eq!(row.text("dirty_status")?, "sum-overflow");
        // The public report is correctly withheld on dirty overflow. Observe
        // the actual helpers separately to compare its frozen retained sums.
        let seeded = seeds(y, plan.r, plan.s)?;
        let (delta, q) = risk_prefix(y, plan.r, plan.s)?;
        let mut sums = [RiskWide::ZERO; 3];
        let mut positive = 0;
        for bundle in &plan.bundles {
            let pv = observe(bundle, &seeded)?;
            positive += usize::from(pv.to_bits() != 0);
            let terms = risk_terms(pv, delta, q, bundle.j, plan.r, plan.s, bundle.index)?;
            for ((sum, term), phase) in sums.iter_mut().zip(terms).zip(PHASES) {
                accumulate(sum, term, bundle.index, phase)?;
            }
        }
        assert_eq!(positive, row.integer("positive_count")?);
        assert_eq!(sums.iter().map(|value| value.bits().to_string()).collect::<Vec<_>>().join(","), row.text("sum_bits")?);
        assert_eq!(sums.iter().map(fnv_words).collect::<Vec<_>>().join(","), row.text("sum_fnv64")?);
        return Ok(());
    }
    let risk = actual?;
    assert_eq!(row.text("dirty_status")?, "ok");
    let sums = [&risk.first, &risk.second, &risk.moment];
    assert_eq!(sums.iter().map(|value| value.bits().to_string()).collect::<Vec<_>>().join(","), row.text("sum_bits")?, "{} sum bit lengths", row.id());
    assert_eq!(sums.iter().map(|value| fnv_words(value)).collect::<Vec<_>>().join(","), row.text("sum_fnv64")?, "{} exact accumulator limbs", row.id());
    expected_bits(row, "dirty_bits_out", risk.dirty_per_100())?;
    assert_eq!(risk.metadata, plan.metadata());
    let seeded = seeds(y, plan.r, plan.s)?;
    let positive = plan.bundles.iter().map(|bundle| observe(bundle, &seeded)).collect::<Result<Vec<_>, _>>()?.into_iter().filter(|pv| pv.to_bits() != 0).count();
    assert_eq!(positive, row.integer("positive_count")?);
    let results = [risk.price_derivative(), risk.price_second_derivative(), risk.time_weighted_amount(),
        risk.modified_duration(), risk.macaulay_duration(), risk.convexity()];
    let names = ["p1", "p2", "moment", "modified", "macaulay", "convexity"];
    let phases = [PHASES[0], PHASES[1], PHASES[2], "modified duration", "macaulay duration", "convexity"];
    for ((result, name), phase) in results.into_iter().zip(names).zip(phases) {
        check_value(result, row.text(&format!("{name}_status"))?, bits(row.text(&format!("{name}_bits"))?)?, phase, phase, None, row.id())?;
    }
    Ok(())
}
fn residual_row(row: &Row<'_>) -> Check {
    let dirty = row.real("dirty_bits")?;
    let target = row.real("target_bits")?;
    let resolved = match row.text("target_kind")? {
        "dirty" => zero(target),
        "clean" => zero(exact_sum(&[target, row.real("ai_bits")?])?),
        _ => return Err("unknown target kind".into()),
    };
    expected_bits(row, "dirty_target_bits", resolved)?;
    if row.text("status")? == "nonpositive-target" {
        assert!(resolved <= 0.0);
        // A real source plan with AI=.5 covers the clean cancellation row.
        let q0: Date = "2025-03-01".parse()?;
        let q1: Date = "2025-09-01".parse()?;
        let settlement: Date = "2025-06-01".parse()?;
        let treasury = UsTreasury::new(q0, q0, q1, DecimalAmount::new(2, 0)?, q1, vec![q0, q1], EndOfMonth::Ignore,
            TreasurySourcePolicy::new(TreasuryDailyBasis::ExactRatio, TreasuryTieRule::NearestTiesEven), 2)?;
        let plan = TreasuryPricingPlan::new(&treasury, settlement, 1)?;
        let target_quote = BondPrice { value_per_100_face: target, kind: if row.text("target_kind")? == "dirty" { BondPriceKind::Dirty } else { BondPriceKind::Clean } };
        let bracket = RootBracket::new(-3.0, -2.5)?;
        let options = SolverOptions::new(1e-12, 1e-12, 8)?;
        match plan.yield_from_price(target_quote, bracket, options) {
            Err(TreasuryYieldError::NonPositiveTarget { dirty_target_per_100, evaluations, source_policy, .. }) => {
                assert_eq!(dirty_target_per_100.to_bits(), resolved.to_bits());
                assert_eq!(evaluations.function(), 0);
                assert_eq!(evaluations.derivative(), 0);
                assert_eq!(source_policy, treasury.source_policy());
            }
            other => return Err(format!("{} nonpositive target precedence: {other:?}", row.id()).into()),
        }
        return Ok(());
    }
    assert!(resolved > 0.0);
    let scale = u8::from(resolved >= f64::from_bits(SCALE_THRESHOLD));
    assert_eq!(scale, row.integer("scale")?);
    expected_bits(row, "residual_bits", residual(dirty, resolved, scale)?)?;
    expected_bits(row, "amount_bits", amount_residual(dirty, resolved)?)?;
    Ok(())
}
fn bump(map: &mut BTreeMap<String, usize>, key: &str) { *map.entry(key.to_owned()).or_default() += 1; }
fn census(map: &BTreeMap<String, usize>) -> String {
    format!("{{{}}}", map.iter().map(|(key, value)| format!("\"{key}\":{value}")).collect::<Vec<_>>().join(","))
}
// These controls exercise shared solver transport, not a Treasury financial
// state. The valid financial interior-failure case is separately proof-excluded.
fn synthetic_fallible_transport_controls() -> Check {
    let bracket = RootBracket::new(0.0, 1.0)?;
    let options = SolverOptions::new(1e-12, 0.0, 3)?;
    let expected = [0.0_f64.to_bits(), 1.0_f64.to_bits(), 0.5_f64.to_bits()];
    let marker = "synthetic generic third-call rejection";
    let mut calls = Vec::new();
    let failed = brent_with_payload(
        |coordinate| {
            calls.push(coordinate.to_bits());
            if calls.len() == 3 { Err(marker) }
            else { Ok((coordinate - 0.5, calls.len())) }
        }, bracket, options,
    );
    match failed {
        Err(FallibleSolverError::Callback { stage, point, evaluations, cause }) => {
            assert_eq!(stage, EvaluationStage::TrialPoint);
            assert_eq!(point.to_bits(), 0.5_f64.to_bits());
            assert_eq!(evaluations.function(), 3);
            assert_eq!(evaluations.derivative(), 0);
            assert_eq!(cause, marker);
        }
        _ => return Err("synthetic TrialPoint failure was not transported".into()),
    }
    assert_eq!(calls.as_slice(), &expected);

    let mut calls = Vec::new();
    let success = brent_with_payload(
        |coordinate| {
            calls.push(coordinate.to_bits());
            Ok::<_, &str>((coordinate - 0.5, (coordinate.to_bits(), calls.len())))
        }, bracket, options,
    );
    match success {
        Ok((report, payload)) => {
            assert_eq!(report.point().to_bits(), 0.5_f64.to_bits());
            assert_eq!(report.residual().to_bits(), 0.0_f64.to_bits());
            assert_eq!(report.reason(), crate::math::TerminationReason::FunctionZero);
            assert_eq!(report.evaluations().function(), 3);
            assert_eq!(report.evaluations().derivative(), 0);
            assert_eq!(report.iterations(), 1);
            assert_eq!(payload, (0.5_f64.to_bits(), 3));
            // Reading report/payload above must not invoke a final callback.
            assert_eq!(calls.as_slice(), &expected);
        }
        _ => return Err("synthetic cached-payload success failed".into()),
    }

    let mut calls = Vec::new();
    let limited = brent_with_payload(
        |coordinate| {
            calls.push(coordinate.to_bits());
            Ok::<_, &str>((coordinate - 0.5, calls.len()))
        }, bracket, SolverOptions::new(1e-12, 0.0, 2)?,
    );
    match limited {
        Err(FallibleSolverError::Solver(SolverError::EvaluationLimit { evaluations, .. })) => {
            assert_eq!(evaluations.function(), 2);
            assert_eq!(evaluations.derivative(), 0);
        }
        _ => return Err("synthetic cap2 did not stop before the third invocation".into()),
    }
    assert_eq!(calls.as_slice(), &expected[..2]);
    Ok(())
}
// Positive normalized primitive supersets, independently derived after coverage
// review. These are not new financial rows or claims of dated reachability.
fn positive_normalization_controls() -> Check {
    let leading = Wide::<4>::from_u128(1).checked_shl(255).ok_or("leading bit")?;
    let first = leading.checked_add(Wide::from_u128(1)).ok_or("first operand")?;
    let last = Wide::<4>::from_u128(u128::MAX).checked_shl(128)
        .and_then(|v| v.checked_add(Wide::from_u128(u128::MAX))).ok_or("all ones")?;
    let second = last.checked_sub(Wide::from_u128(1)).ok_or("second operand")?;
    // (2^255+1)(2^256-2) = 2^511-2. Directed upward rounding must
    // carry into a new binade; downward retains the last old-binade point.
    let a = Endpoint { significand: first, exponent: -255 };
    let b = Endpoint { significand: second, exponent: -255 };
    assert_eq!(endpoint_product(a, b, false, 0)?, Endpoint { significand: last, exponent: -255 });
    assert_eq!(endpoint_product(a, b, true, 0)?, Endpoint { significand: leading, exponent: -254 });

    // 1+3*2^-53 is an exact midpoint with an odd lower significand.
    let halfway = Wide::<4>::from_u128((1_u128 << 52) + 1).checked_shl(203)
        .and_then(|v| Wide::<4>::from_u128(1).checked_shl(202).and_then(|h| v.checked_add(h)))
        .ok_or("odd half endpoint")?;
    assert_eq!(factor_observation(Endpoint { significand: halfway, exponent: -255 }, 0)?,
        Extended { significand: (1_u64 << 52) + 2, exponent: -52 });
    Ok(())
}
#[test]
fn all_frozen_numeric_rows_match_actual_helpers() -> Check {
    let fixture = Fixture::load(Fixture::repository_path("validation/fixtures/bonds/treasury-numeric-oracle.fixture"))?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    let mut operations = BTreeMap::new();
    let mut statuses = BTreeMap::new();
    let mut families = BTreeMap::new();
    let mut profiles = BTreeMap::new();
    let mut reachability = BTreeMap::new();
    let mut visited = 0;
    for record in fixture.records() {
        let row = Row(record);
        assert_eq!(row.id(), format!("UTNUM-{:06}", visited + 1));
        let op = row.text("op")?;
        let outcome = match op {
            "SEED" => seed_row(&row), "FACTOR" => factor_row(&row), "EXT53" => ext_row(&row), "PV" => pv_row(&row),
            "AICACHE" => ai_row(&row), "TERM" => term_row(&row), "REDUCE" => reduction_row(&row), "RESIDUAL" => residual_row(&row),
            _ => Err("unknown operation; no row skipped".into()),
        };
        outcome.map_err(|error| format!("{} {op}: {error}", row.id()))?;
        bump(&mut operations, op); bump(&mut statuses, row.text("status")?); bump(&mut families, row.text("family")?);
        bump(&mut profiles, row.text("profile")?); bump(&mut reachability, row.text("reachability")?);
        visited += 1;
    }
    assert_eq!(visited, 5_593);
    assert_eq!(fixture.record_count(), visited);
    for (name, actual) in [("operation-counts", operations), ("status-counts", statuses), ("family-counts", families),
        ("profile-counts", profiles), ("reachability-counts", reachability)] {
        let expected = fixture.header(name).ok_or("missing census")?.replace(" ", "");
        assert_eq!(expected, census(&actual), "{name}");
    }
    synthetic_fallible_transport_controls()?;
    positive_normalization_controls()?;
    Ok(())
}
}
// END TEMPORARY TREASURY KERNEL ADAPTER
'''

def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    state = 0xcbf29ce484222325
    for byte in data:
        state = ((state ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{state:016x}'


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode()


def snapshot():
    names = [Path(name) for name in ('Cargo.toml', 'Cargo.lock', 'README.md', 'CHANGELOG.md',
                                     'clippy.toml', 'LICENSE-MIT', 'LICENSE-APACHE')]
    for directory in (ROOT / 'src', ROOT / 'tests/support'):
        names.extend(path.relative_to(ROOT) for path in directory.rglob('*') if path.is_file())
    result = {}
    for relative in sorted(set(names)):
        path = ROOT / relative
        if path.is_symlink() or not path.is_file():
            raise ValueError('snapshot requires regular nonsymlink files: ' + str(relative))
        result[str(relative)] = path.read_bytes()
    return result


def manifest(files):
    return [{'path': name, 'bytes': len(data), 'sha256': sha(data)} for name, data in sorted(files.items())]


def fixture_info(data):
    header, blank, body = data.partition(b'\n\n')
    if not blank:
        raise ValueError('fixture separator missing')
    fields = {}
    for line in header.decode().splitlines():
        key, value = line.split(': ', 1)
        if not re.fullmatch('[a-z][a-z0-9-]*', key) or key in fields or not value or value.strip() != value:
            raise ValueError('invalid fixture metadata')
        fields[key] = value
    columns, _, records = body.partition(b'\n')
    if columns.decode().split('\t') != SCHEMA or fields['schema'] != '1':
        raise ValueError('frozen numeric schema mismatch')
    if int(fields['record-count']) != 5593 or len(records.splitlines()) != 5593:
        raise ValueError('frozen numeric record census mismatch')
    for key, actual in (('body-sha256', sha(body)), ('records-sha256', sha(records)), ('body-fnv64', fnv(body)), ('records-fnv64', fnv(records))):
        if fields[key] != actual:
            raise ValueError('fixture integrity mismatch: ' + key)
    return fields, {'bytes': len(data), 'sha256': sha(data), 'body_sha256': sha(body),
                    'records_sha256': sha(records), 'record_count': 5593}


def run(argv, cwd, environment, output, label, evidence):
    process = subprocess.run(argv, cwd=cwd, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    streams = {}
    for name, data in (('stdout', process.stdout), ('stderr', process.stderr)):
        path = output / f'{label}.{name}'
        path.write_bytes(data)
        streams[name] = {'path': str(path), 'bytes': len(data), 'sha256': sha(data)}
    evidence.setdefault('commands', []).append(dict(label=label, argv=argv, cwd=str(cwd),
                                                    exit_code=process.returncode, **streams))
    if process.returncode:
        raise RuntimeError(f'{label} failed with exit{process.returncode}; inspect {output / (label + ".stderr")}')
    return process.stdout


def unchanged(original, evidence):
    current = snapshot()
    changed = sorted(name for name in set(original) | set(current) if original.get(name) != current.get(name))
    evidence['original_files_unchanged'] = not changed
    evidence['original_changed_paths'] = changed
    return not changed




def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='create-new evidence directory outside the repository')
    parser.add_argument('--cargo', default='cargo')
    parser.add_argument('--rustc', default='rustc')
    args = parser.parse_args()
    pinned = {}
    for name, digest in PINS.items():
        data = (ROOT / name).read_bytes()
        if sha(data) != digest:
            parser.error('frozen input changed: ' + name)
        pinned[name] = data
    fixture_header, fixture = fixture_info(pinned[str(FIXTURE)])
    original = snapshot()
    runner = Path(__file__).read_bytes()
    destination = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix='sabiquant-treasury-kernel-'))
    if destination == ROOT or ROOT in destination.parents:
        parser.error('scratch evidence must be outside the repository')
    if args.output:
        destination.mkdir(parents=True, exist_ok=False)
    crate = destination / 'crate'
    evidence = {'schema': 1, 'started_at': dt.datetime.now(dt.timezone.utc).isoformat(),
                'python': sys.version, 'platform': platform.platform(), 'output': str(destination),
                'crate': str(crate), 'fixture': fixture, 'fixture_header': fixture_header,
                'runner': {'path': str(Path(__file__).resolve()), 'bytes': len(runner), 'sha256': sha(runner)},
                'adapter': {'embedded': True, 'bytes': len(ADAPTER.encode()), 'sha256': sha(ADAPTER.encode())},
                'proof_metadata_not_runtime_counters': ['weights', 'width_ok', 'relation', 'straddle', 'containment', 'tree_alt', 'peak_bits', 'num_bits', 'den_bits', 'e_diff'],
                'word_accessor': {'cfg_test_only': True, 'bytes': len(WIDE_ACCESSOR.encode()), 'sha256': sha(WIDE_ACCESSOR.encode())},
                'pinned_inputs': manifest(pinned), 'source_manifest': manifest(original),
                'source_manifest_sha256': sha(canonical(manifest(original))), 'test_name': TEST_NAME,
                'scope': '5593 frozen primitive rows, all eight partitions, plus3 explicitly synthetic shared-Brent transport controls and2 postproduction positive normalized primitive-superset controls; supplemental tests separate from standard suite. No dated financial reachability is asserted for the2 added primitive controls.',
                'positive_primitive_control_names': ['directed-product-next-binade-carry', 'extended53-odd-lower-midpoint'],
                'synthetic_transport_control_names': ['third-trial-fallible-error-context', 'cached-payload-success-no-final-callback', 'cap2-before-third-invocation'],
                'result': 'incomplete'}
    failure = None
    try:
        cargo, rustc = shutil.which(args.cargo), shutil.which(args.rustc)
        if not cargo or not rustc:
            raise ValueError('matching cargo/rustc toolchain required')
        cargo, rustc = str(Path(cargo).absolute()), str(Path(rustc).absolute())
        environment = os.environ.copy()
        removed = []
        for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'LLVM_PROFILE_FILE', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_TARGET'):
            if key in environment:
                del environment[key]
                removed.append(key)
        environment.update(CARGO_TARGET_DIR=str(destination / 'target'), CARGO_INCREMENTAL='0',
                           RUSTC=rustc, RUSTFLAGS='-D warnings', RUST_BACKTRACE='1')
        evidence['environment'] = {'removed_keys': removed, 'RUSTC': rustc, 'RUSTFLAGS': '-D warnings',
                                   'CARGO_INCREMENTAL': '0', 'CARGO_TARGET_DIR': environment['CARGO_TARGET_DIR']}
        toolchain = {}
        for label, executable, flags in (('rustc', rustc, ['-Vv']), ('cargo', cargo, ['-V'])):
            version = run([executable, *flags], ROOT, environment, destination, label + '-version', evidence)
            toolchain[label] = {'path': executable, 'sha256': sha(Path(executable).read_bytes()), 'version': version.decode()}
        evidence['toolchain'] = toolchain
        for name, data in original.items():
            path = crate / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            if path.read_bytes() != data:
                raise RuntimeError('copy mismatch: ' + name)
        copied = {name: (crate / name).read_bytes() for name in original}
        evidence['copied_source_manifest_before_injection'] = manifest(copied)
        assert copied == original
        injected = {}
        evidence['injections'] = []
        for relative, addition in ((MODULE, ADAPTER.encode()), (WIDE_MODULE, WIDE_ACCESSOR.encode())):
            source = original[str(relative)]
            if b'validation_treasury_kernel' in source or b'validation_treasury_words' in source:
                raise ValueError('original already contains supplemental validation injection')
            content = source + addition
            (crate / relative).write_bytes(content)
            assert (crate / relative).read_bytes()[:len(source)] == source
            injected[str(relative)] = content
            evidence['injections'].append({'path': str(relative), 'original_prefix_bytes': len(source),
                                            'original_prefix_sha256': sha(source), 'appended_bytes': len(addition),
                                            'injected_sha256': sha(content), 'cfg_test_only': True})
        installed = crate / FIXTURE
        installed.parent.mkdir(parents=True, exist_ok=True)
        installed.write_bytes(pinned[str(FIXTURE)])
        assert installed.read_bytes() == pinned[str(FIXTURE)]
        metadata = json.loads(run([cargo, 'metadata', '--offline', '--locked', '--no-deps', '--format-version=1'],
                                  crate, environment, destination, 'metadata', evidence))
        assert len(metadata['packages']) == 1 and not metadata['packages'][0]['dependencies']
        listing = run([cargo, 'package', '--list', '--allow-dirty', '--offline', '--locked'],
                      crate, environment, destination, 'package-list', evidence).decode().splitlines()
        assert not any(name.startswith('validation/') for name in listing)
        evidence['package_excludes_validation_artifacts'] = True
        for label, release in (('kernel-debug', []), ('kernel-release', ['--release'])):
            stdout = run([cargo, 'test', '--offline', '--locked', *release, '--lib', TEST_NAME,
                          '--', '--exact', '--test-threads=1', '--nocapture'], crate, environment,
                         destination, label, evidence).decode()
            if not re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;', stdout):
                raise RuntimeError('exactly one supplemental test did not execute: ' + label)
        run([cargo, 'clippy', '--offline', '--locked', '--lib', '--tests', '--', '-D', 'warnings'],
            crate, environment, destination, 'copied-clippy', evidence)
        evidence['all_fixture_rows_checked_per_test'] = 5593
        evidence['synthetic_transport_controls_checked_per_test'] = 3
        evidence['supplemental_test_executions'] = 2
        evidence['postproduction_positive_primitive_controls_per_test'] = 2
        changed = sorted(name for name, data in original.items() if (crate / name).read_bytes() != data)
        if changed != sorted(injected) or any((crate / name).read_bytes() != data for name, data in injected.items()):
            raise RuntimeError('unexpected copied source changes')
        evidence['copied_source_changes'] = changed
        if installed.read_bytes() != pinned[str(FIXTURE)]:
            raise RuntimeError('copied frozen fixture changed during check')
        evidence['copied_fixture_unchanged'] = True
        if any((ROOT / name).read_bytes() != data for name, data in pinned.items()) or Path(__file__).read_bytes() != runner:
            raise RuntimeError('frozen inputs or runner changed during check')
        if not unchanged(original, evidence):
            raise RuntimeError('original source snapshot changed during run')
        evidence['result'] = 'passed'
    except Exception as error:
        failure = error
        evidence['result'] = 'failed'
        evidence['failure'] = {'type': type(error).__name__, 'message': str(error)}
        unchanged(original, evidence)
    evidence['finished_at'] = dt.datetime.now(dt.timezone.utc).isoformat()
    output = destination / 'evidence.json'
    output.write_text(json.dumps(evidence, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'result': evidence['result'], 'evidence': str(output), 'evidence_sha256': sha(output.read_bytes()),
                      'source_manifest_sha256': evidence['source_manifest_sha256'], 'fixture_sha256': fixture['sha256']}, sort_keys=True), flush=True)
    if failure:
        raise SystemExit(str(failure))


if __name__ == '__main__':
    main()
