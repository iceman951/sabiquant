#!/usr/bin/env python3
"""Attested exact-copy private conventional-gilt kernel check (stdlib only).

Adds test-only observation code to two copied modules in a create-new scratch
crate. No original Rust file, frozen oracle, public API or package is changed.
The oracle was authored before any gilt production; this adapter now reads
private interfaces mechanically and compares every frozen primitive row.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from fractions import Fraction as F
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import struct
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
MODULE = Path('src/bonds/gilt/numeric.rs')
WIDE_MODULE = Path('src/amounts/uint.rs')
FIXTURE = Path('validation/fixtures/bonds/gilt-numeric-oracle.fixture')
GENERATOR = Path('validation/generator/gilt_numeric_oracles.py')
CONTRACT = Path('docs/M8_GILT_CONTRACT.md')
PINS = {
    str(FIXTURE): '40a8bb169b9876906e60465805b49b95c49ac6289d0720608819505120f36915',
    str(GENERATOR): 'be197c8408a5d3d0a394701925468d42a2eeca03b08d32f74d9ef37bbc9e7160',
    str(CONTRACT): '03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292',
}
TEST_NAME = 'bonds::gilt::numeric::validation_gilt_kernel::all_frozen_numeric_rows_match_actual_helpers'
GUARD_TEST = 'bonds::gilt::numeric::validation_gilt_kernel::postproduction_guard_matches_exact_reference'
GUARD_FIXTURE = Path('validation/fixtures/bonds/gilt-guard-adapter.fixture')
SCHEMA = '''id op family reachability source c_coef c_scale r s j redeem dates ai_branch t s2
 y_bits g_bits discount_bits pv_bits delta_bits measure recipes bundle_count dirty_bits target_bits
 scale exact_n exact_d pow2 negative significand exponent basis_bit_lengths basis_fnv64 status
 expected_bits alternate_bits normalized_status normalized_bits amount_status amount_bits p1_status
 p1_bits p2_status p2_bits moment_status moment_bits modified_status modified_bits macaulay_status
 macaulay_bits convexity_status convexity_bits detail'''.split()
WIDE_ACCESSOR = r'''
// BEGIN TEMPORARY GILT WORD OBSERVATION (TEST ONLY)
#[cfg(test)]
mod validation_gilt_observation {
    use super::Wide;
    impl<const N: usize> Wide<N> {
        /// Borrows the actual stored limbs for supplemental primitive validation.
        pub(crate) fn validation_gilt_words(&self) -> &[u64; N] {
            &self.0
        }
    }
}
// END TEMPORARY GILT WORD OBSERVATION
'''
ADAPTER = r'''
// BEGIN TEMPORARY GILT KERNEL ADAPTER (TEST ONLY)
#[cfg(test)]
mod validation_gilt_kernel {
use super::*;
use super::super::{ExactSource, REGULAR_DIVIDEND, publish_first_dividend, source_rational};
use crate::amounts::DecimalAmount;
use crate::error::{NonFinite, ValueError};
use crate::support::fixture::{Fixture, Record};
use std::collections::{BTreeMap, BTreeSet};

type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;
const COLUMNS: [&str; 54] = [
    "id", "op", "family", "reachability", "source", "c_coef", "c_scale", "r", "s", "j",
    "redeem", "dates", "ai_branch", "t", "s2", "y_bits", "g_bits", "discount_bits",
    "pv_bits", "delta_bits", "measure", "recipes", "bundle_count", "dirty_bits",
    "target_bits", "scale", "exact_n", "exact_d", "pow2", "negative", "significand",
    "exponent", "basis_bit_lengths", "basis_fnv64", "status", "expected_bits",
    "alternate_bits", "normalized_status", "normalized_bits", "amount_status", "amount_bits",
    "p1_status", "p1_bits", "p2_status", "p2_bits", "moment_status", "moment_bits",
    "modified_status", "modified_bits", "macaulay_status", "macaulay_bits",
    "convexity_status", "convexity_bits", "detail",
];
const PHASES: [&str; 3] = ["price derivative", "price second derivative", "time weighted amount"];
const NORMAL_PHASES: [&str; 3] = ["modified duration", "convexity", "macaulay duration"];

struct Seen<'a> {
    record: Record<'a>,
}
impl<'a> Seen<'a> {
    fn get(&self, name: &'static str) -> Check<&'a str> {
        Ok(self.record.text(name)?)
    }
    fn id(&self) -> &'a str { self.record.id() }
    fn integer<T: std::str::FromStr>(&self, name: &'static str) -> Check<T>
    where T::Err: std::error::Error + 'static {
        Ok(self.get(name)?.parse()?)
    }
    fn finish(&self) -> Check {
        let common = ["id", "op", "family", "reachability", "status", "detail"];
        let active: &[&str] = match self.record.text("op")? {
            "PV" => &["source", "c_coef", "c_scale", "r", "s", "redeem", "dates", "discount_bits", "exact_n", "exact_d", "expected_bits", "alternate_bits"],
            "Z" => &["r", "s", "j", "dates", "g_bits", "y_bits", "exact_n", "exact_d", "pow2", "negative", "expected_bits", "alternate_bits"],
            "AI" => &["source", "c_coef", "c_scale", "r", "s", "dates", "ai_branch", "t", "s2", "exact_n", "exact_d", "expected_bits"],
            "TERM" => &["source", "c_coef", "c_scale", "redeem", "r", "s", "j", "dates", "y_bits", "delta_bits", "discount_bits", "pv_bits", "measure", "exact_n", "exact_d", "pow2", "negative", "significand", "exponent", "expected_bits", "normalized_status", "normalized_bits", "alternate_bits"],
            "REDUCE" => &["source", "c_coef", "c_scale", "redeem", "r", "s", "dates", "y_bits", "delta_bits", "bundle_count", "recipes", "dirty_bits", "basis_bit_lengths", "basis_fnv64", "p1_status", "p1_bits", "p2_status", "p2_bits", "moment_status", "moment_bits", "modified_status", "modified_bits", "macaulay_status", "macaulay_bits", "convexity_status", "convexity_bits"],
            "RESIDUAL" => &["dirty_bits", "target_bits", "scale", "expected_bits", "amount_status", "amount_bits"],
            _ => return Err("unknown operation active columns".into()),
        };
        for name in COLUMNS {
            if !common.contains(&name) && !active.contains(&name) {
                assert_eq!(self.record.text(name)?, "-", "{} unused {name}", self.id());
            }
        }
        Ok(())
    }
}
fn hex(text: &str) -> Check<u64> {
    assert_eq!(text.len(), 16, "hex width");
    assert!(text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
    Ok(u64::from_str_radix(text, 16)?)
}
fn number(row: &Seen<'_>, name: &'static str) -> Check<f64> {
    let value = f64::from_bits(hex(row.get(name)?)?);
    assert!(value.is_finite(), "{} finite {name}", row.id());
    Ok(value)
}
fn flag(text: &str) -> Check<bool> {
    match text { "0" => Ok(false), "1" => Ok(true), _ => Err("invalid flag".into()) }
}
fn magnitude(text: &str) -> Check<Scalar> {
    assert!(!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()));
    let mut value = Scalar::ZERO;
    for digit in text.bytes() {
        value = value.checked_mul_small(10)
            .and_then(|v| v.checked_add(Scalar::from_u128(u128::from(digit - b'0'))))
            .ok_or("oracle primitive scalar outside five words")?;
    }
    Ok(value)
}
fn exact_input(row: &Seen<'_>) -> Check<SourceRational> {
    let text = row.get("exact_n")?;
    let (negative, text) = text.strip_prefix('-').map_or((false, text), |v| (true, v));
    let numerator = magnitude(text)?;
    let denominator = magnitude(row.get("exact_d")?)?;
    assert!(!denominator.is_zero());
    Ok(SourceRational { numerator, denominator, negative: negative && !numerator.is_zero() })
}
fn amount(row: &Seen<'_>) -> Check<DecimalAmount> {
    let coefficient: i128 = row.integer("c_coef")?;
    let scale: u8 = row.integer("c_scale")?;
    assert!(coefficient >= 0 && scale <= 38);
    Ok(DecimalAmount::new(coefficient, scale)?)
}
fn dates(row: &Seen<'_>) -> Check<[Date; 4]> {
    let values: Result<Vec<_>, _> = row.get("dates")?.split('|').map(str::parse::<Date>).collect();
    Ok(values?.try_into().map_err(|_| "four source/coordinate dates required")?)
}
fn source_dates(row: &Seen<'_>, role: &str, r: u32, s: u32) -> Check {
    let [q0, issue, q1, first] = dates(row)?;
    assert!(q0 <= issue && issue < q1);
    assert_eq!(q1.days_since(q0), i32::try_from(s)?);
    assert_eq!(q1.days_since(issue), i32::try_from(r)?);
    assert_eq!(issue == q0, role == "Regular");
    assert_eq!(first > q1, role == "Long");
    assert!(first >= q1);
    Ok(())
}
fn coordinate(row: &Seen<'_>, j: u32) -> Check<(u32, u32, usize)> {
    let r: u32 = row.integer("r")?;
    let s: u32 = row.integer("s")?;
    assert!(1 <= r && r <= s && s <= 184 && j < 20_000);
    let [previous, settlement, next, payment] = dates(row)?;
    assert!(previous <= settlement && settlement < next && next <= payment);
    assert_eq!(next.days_since(previous), i32::try_from(s)?);
    assert_eq!(next.days_since(settlement), i32::try_from(r)?);
    let months = 12 * (payment.year() - next.year()) + i32::try_from(payment.month())? - i32::try_from(next.month())?;
    assert_eq!(months, i32::try_from(6 * j)?);
    assert_eq!(next.day(), payment.day());
    let l = r.checked_add(j.checked_mul(s).ok_or("coordinate multiplication")?).ok_or("coordinate addition")?;
    assert!(l < 1 << 22 && l + s < 1 << 22);
    Ok((l, s, usize::try_from(j)? + 1))
}
fn compare_rational(actual: SourceRational, expected: SourceRational, id: &str) -> Check {
    assert_eq!(actual.negative, expected.negative, "{id} rational sign");
    let first = actual.numerator.checked_mul(expected.denominator).ok_or("source rational crossproduct")?;
    let second = expected.numerator.checked_mul(actual.denominator).ok_or("source rational crossproduct")?;
    assert_eq!(first, second, "{id} exact source coefficient");
    Ok(())
}
fn check_value(result: Result<f64, GiltError>, status: &str, expected: u64,
               phase: &'static str, quantity: &'static str, index: Option<usize>, id: &str) -> Check {
    match (status, result) {
        ("ok", Ok(value)) => assert_eq!(value.to_bits(), expected, "{id} {phase}"),
        ("overflow+" | "overflow-" | "underflow+" | "underflow-", Err(error)) => {
            assert_eq!(error.kind(), "gilt.value", "{id} kind");
            assert!(error.source().is_some(), "{id} immediate source");
            match error {
                GiltError::Value { index: actual_index, phase: actual_phase, cause, .. } => {
                    assert_eq!(actual_index, index, "{id} error index");
                    assert_eq!(actual_phase, phase, "{id} error phase");
                    let negative = status.ends_with('-');
                    if status.starts_with("overflow") {
                        assert_eq!(expected, INFINITY | if negative { SIGN } else { 0 });
                        let class = if negative { NonFinite::NegativeInfinity } else { NonFinite::PositiveInfinity };
                        assert_eq!(cause, ValueError::ResultNotFinite { quantity, value: class }, "{id} overflow cause");
                    } else {
                        assert_eq!(expected, if negative { SIGN } else { 0 });
                        match cause {
                            ValueError::ResultOutOfDomain { quantity: actual_quantity, value, domain, .. } => {
                                assert_eq!(actual_quantity, quantity, "{id} quantity");
                                assert_eq!(value.to_bits(), expected, "{id} signed underflow");
                                assert_eq!(domain, if negative { Interval::less_than(0.0) } else { Interval::greater_than(0.0) }, "{id} strict domain");
                            }
                            other => return Err(format!("{id} unexpected underflow cause {other:?}").into()),
                        }
                    }
                }
                other => return Err(format!("{id} wrong outer error {other:?}").into()),
            }
        }
        (expected_status, actual) => return Err(format!("{id} {phase}: expected {expected_status}/{expected:016x}, got {actual:?}").into()),
    }
    Ok(())
}
fn alternate(row: &Seen<'_>) -> Check {
    let alternative = row.get("alternate_bits")?;
    if alternative != "-" {
        assert_ne!(hex(alternative)?, hex(row.get("expected_bits")?)?, "{} discriminator", row.id());
    }
    Ok(())
}
fn pv(row: &Seen<'_>) -> Check {
    let coupon = amount(row)?;
    let role = row.get("source")?;
    let r: u32 = row.integer("r")?;
    let s: u32 = row.integer("s")?;
    source_dates(row, role, r, s)?;
    let exact = match role {
        "Regular" => REGULAR_DIVIDEND,
        "Short" | "Long" => ExactSource::Published { micro: publish_first_dividend(coupon, if role == "Short" { r } else { s + r }, s)? },
        _ => return Err("unknown source role".into()),
    };
    let index = if role == "Long" { 2 } else { 1 };
    let mut rational = source_rational(coupon, exact, Some(0), "source dividend")?;
    if flag(row.get("redeem")?)? { rational = rational.with_par(Some(index), "pricing bundles")?; }
    compare_rational(rational, exact_input(row)?, row.id())?;
    let factor = number(row, "discount_bits")?;
    assert!(factor > 0.0);
    check_value(source_pv(rational, factor, index), row.get("status")?, hex(row.get("expected_bits")?)?,
                "source present value", "gilt present value", Some(index), row.id())?;
    alternate(row)
}
fn z(row: &Seen<'_>) -> Check {
    let j: u32 = row.integer("j")?;
    let (l, s, index) = coordinate(row, j)?;
    let g = number(row, "g_bits")?;
    let supplied_y = row.get("y_bits")?;
    if supplied_y != "-" {
        assert_eq!(hex(supplied_y)?, g.to_bits());
        assert!(g.abs() < TINY);
    }
    let exact = exact_input(row)?;
    let exponent: i32 = row.integer("pow2")?;
    let negative = flag(row.get("negative")?)?;
    if g.to_bits() & !SIGN == 0 {
        assert!(exact.numerator.is_zero());
        assert_eq!(exact.denominator, Scalar::from_u128(1));
        assert_eq!(exponent, -1074);
        assert!(!negative);
    } else {
        let (sig, power) = decode(g, Some(index), "discount factor")?;
        assert_eq!(exponent, power);
        assert_eq!(exact.numerator, Scalar::from_u128(u128::from(sig)).checked_mul_small(l).ok_or("z metadata product")?);
        assert_eq!(exact.denominator, Scalar::from_u128(2 * u128::from(s)));
        assert_eq!(negative, !g.is_sign_negative());
    }
    assert_eq!(row.get("status")?, "ok");
    assert_eq!(quasi_exponent(g, l, s, index)?.to_bits(), hex(row.get("expected_bits")?)?, "{} exponent", row.id());
    alternate(row)
}
fn ai(row: &Seen<'_>) -> Check {
    assert_eq!(row.get("source")?, "SourceAccrued");
    let _coupon = amount(row)?;
    let branch = row.get("ai_branch")?;
    let r: u32 = row.integer("r")?;
    let s: u32 = row.integer("s")?;
    let s2: u32 = row.integer("s2")?;
    let t: u32 = row.integer("t")?;
    let role = match branch {
        "RegularCum" | "RegularEx" => "Regular",
        "ShortCum" | "ShortEx" => "Short",
        "LongFirst" | "LongSecondCum" | "LongSecondEx" => "Long",
        _ => return Err("unknown AI provenance branch".into()),
    };
    source_dates(row, role, r, s)?;
    assert!((1..=184).contains(&s2));
    let limit = if role == "Regular" { s } else if branch.starts_with("Short") || branch == "LongFirst" { r } else { s2 };
    assert!(t < limit);
    if branch.ends_with("Ex") { assert!(t > 0); }
    let exact = exact_input(row)?;
    assert!(exact.numerator.bits() <= 144 && exact.denominator.bits() <= 144);
    assert_eq!(row.get("status")?, "ok");
    let actual = source_binary(exact, None, "source accrued", "round source accrued")?;
    assert_eq!(actual.to_bits(), hex(row.get("expected_bits")?)?, "{} AI primitive admission", row.id());
    Ok(())
}
fn term(row: &Seen<'_>) -> Check {
    let coupon = amount(row)?;
    assert_eq!(row.get("source")?, "Regular");
    assert!(!flag(row.get("redeem")?)?);
    let j: u32 = row.integer("j")?;
    let (l, s, index) = coordinate(row, j)?;
    let y = quote(number(row, "y_bits")?)?;
    let delta = number(row, "delta_bits")?;
    assert_eq!((2.0 + y).to_bits(), delta.to_bits());
    let accepted_factor = number(row, "discount_bits")?;
    assert!(accepted_factor > 0.0);
    let source = source_rational(coupon, REGULAR_DIVIDEND, Some(index), "pricing bundles")?;
    let pv = source_pv(source, accepted_factor, index)?;
    assert_eq!(pv.to_bits(), hex(row.get("pv_bits")?)?, "{} source-derived PV", row.id());
    let selected = match row.get("measure")? { "p1" => 0, "p2" => 1, "moment" => 2, _ => return Err("unknown risk measure".into()) };
    let actual = risk_terms(pv, delta, l, s, index)?[selected];
    assert_eq!(actual.significand, row.integer::<u64>("significand")?, "{} significand", row.id());
    assert_eq!(actual.exponent, row.integer::<i32>("exponent")?, "{} exponent", row.id());
    let expected_recipe = exact_input(row)?;
    let power: i32 = row.integer("pow2")?;
    let from_recipe = extended_ratio(expected_recipe.numerator, expected_recipe.denominator, power, index, PHASES[selected])?;
    assert_eq!(actual, from_recipe, "{} exact input recipe", row.id());
    let negative = flag(row.get("negative")?)?;
    assert_eq!(negative, selected == 0 && actual.significand != 0);
    let mut sum = RiskWide::ZERO;
    accumulate(&mut sum, actual, index, PHASES[selected])?;
    check_value(raw(sum, negative, PHASES[selected]), row.get("status")?, hex(row.get("expected_bits")?)?,
                PHASES[selected], PHASES[selected], None, row.id())?;
    let normal_status = row.get("normalized_status")?;
    let normal_bits = row.get("normalized_bits")?;
    if pv.to_bits() & !SIGN == 0 {
        assert_eq!(normal_status, "not-applicable");
        assert_eq!(normal_bits, "-");
    } else {
        check_value(normalized(sum, pv, y, NORMAL_PHASES[selected]), normal_status, hex(normal_bits)?,
                    NORMAL_PHASES[selected], NORMAL_PHASES[selected], None, row.id())?;
    }
    alternate(row)
}
fn words_hash(value: &RiskWide) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for word in value.validation_gilt_words() {
        for byte in word.to_le_bytes() { hash = (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3); }
    }
    hash
}
fn reduce(row: &Seen<'_>) -> Check {
    let coupon = amount(row)?;
    assert_eq!(row.get("source")?, "Regular");
    assert!(flag(row.get("redeem")?)?);
    let count: usize = row.integer("bundle_count")?;
    assert!((1..=19_997).contains(&count));
    let (_, s, _) = coordinate(row, u32::try_from(count - 1)?)?;
    let r: u32 = row.integer("r")?;
    let y = quote(number(row, "y_bits")?)?;
    let delta = number(row, "delta_bits")?;
    assert_eq!((2.0 + y).to_bits(), delta.to_bits());
    let mut factors = Vec::with_capacity(count);
    for run in row.get("recipes")?.split(';') {
        let (value, repetitions) = run.split_once('*').ok_or("bad reduction run")?;
        let factor = f64::from_bits(hex(value)?);
        assert!(factor.is_finite() && factor > 0.0);
        let repetitions: usize = repetitions.parse()?;
        assert!(repetitions > 0 && repetitions <= count - factors.len());
        factors.extend(std::iter::repeat_n(factor, repetitions));
    }
    assert_eq!(factors.len(), count);
    let mut pvs = Vec::with_capacity(count);
    let mut sums = [RiskWide::ZERO; 3];
    for (j, factor) in factors.into_iter().enumerate() {
        let index = j + 1;
        let mut source = source_rational(coupon, REGULAR_DIVIDEND, Some(index), "pricing bundles")?;
        if j + 1 == count { source = source.with_par(Some(index), "pricing bundles")?; }
        let pv = source_pv(source, factor, index)?;
        let l = r + u32::try_from(j)? * s;
        let terms = risk_terms(pv, delta, l, s, index)?;
        for (measure, term) in terms.into_iter().enumerate() { accumulate(&mut sums[measure], term, index, PHASES[measure])?; }
        pvs.push(pv);
    }
    let dirty = exact_sum(&pvs)?;
    assert_eq!(dirty.to_bits(), hex(row.get("dirty_bits")?)?, "{} dirty reduction", row.id());
    assert!(dirty > 0.0);
    let lengths: Result<Vec<u32>, _> = row.get("basis_bit_lengths")?.split(',').map(str::parse).collect();
    let hashes: Check<Vec<u64>> = row.get("basis_fnv64")?.split(',').map(hex).collect();
    let lengths = lengths?;
    let hashes = hashes?;
    assert_eq!(lengths.len(), 3);
    assert_eq!(hashes.len(), 3);
    for (measure, sum) in sums.iter().enumerate() {
        assert!(!sum.is_zero());
        assert_eq!(sum.bits(), lengths[measure], "{} basis length", row.id());
        assert_eq!(words_hash(sum), hashes[measure], "{} actual stored words", row.id());
    }
    for (index, status_name, bits_name) in [(0, "p1_status", "p1_bits"), (1, "p2_status", "p2_bits"), (2, "moment_status", "moment_bits")] {
        check_value(raw(sums[index], index == 0, PHASES[index]), row.get(status_name)?, hex(row.get(bits_name)?)?, PHASES[index], PHASES[index], None, row.id())?;
    }
    for (index, status_name, bits_name) in [(0, "modified_status", "modified_bits"), (1, "convexity_status", "convexity_bits"), (2, "macaulay_status", "macaulay_bits")] {
        check_value(normalized(sums[index], dirty, y, NORMAL_PHASES[index]), row.get(status_name)?, hex(row.get(bits_name)?)?, NORMAL_PHASES[index], NORMAL_PHASES[index], None, row.id())?;
    }
    assert_eq!(row.get("status")?, "ok");
    Ok(())
}
fn residual_row(row: &Seen<'_>) -> Check {
    let dirty = number(row, "dirty_bits")?;
    let target = number(row, "target_bits")?;
    assert!(dirty > 0.0 && target > 0.0);
    let scale: u8 = row.integer("scale")?;
    assert_eq!(scale, u8::from(target >= f64::from_bits(1993_u64 << 52)));
    assert_eq!(row.get("status")?, "ok");
    assert_eq!(residual(dirty, target, scale)?.to_bits(), hex(row.get("expected_bits")?)?, "{} residual", row.id());
    check_value(amount_residual(dirty, target), row.get("amount_status")?, hex(row.get("amount_bits")?)?,
                "yield amount residual", "yield amount residual", None, row.id())?;
    Ok(())
}
fn census_text(values: &BTreeMap<String, usize>) -> String {
    format!("{{{}}}", values.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect::<Vec<_>>().join(","))
}
fn bump(values: &mut BTreeMap<String, usize>, key: &str) { *values.entry(key.to_owned()).or_default() += 1; }
#[test]
fn all_frozen_numeric_rows_match_actual_helpers() -> Check {
    let fixture = Fixture::load(Fixture::repository_path("validation/fixtures/bonds/gilt-numeric-oracle.fixture"))?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    let mut operations = BTreeMap::new();
    let mut families = BTreeMap::new();
    let mut statuses = BTreeMap::new();
    let mut reachability = BTreeMap::new();
    let mut visited = 0;
    for record in fixture.records() {
        let row = Seen { record };
        assert_eq!(row.get("id")?, format!("GNUM-{:06}", visited + 1));
        assert_eq!(row.get("reachability")?, "private-primitive");
        assert!(!row.get("detail")?.is_empty());
        let operation = row.get("op")?;
        let outcome = match operation {
            "PV" => pv(&row), "Z" => z(&row), "AI" => ai(&row), "TERM" => term(&row),
            "REDUCE" => reduce(&row), "RESIDUAL" => residual_row(&row),
            _ => Err("unknown operation; no row skipped".into()),
        };
        outcome.map_err(|error| format!("{} {operation}: {error}", row.id()))?;
        bump(&mut operations, operation);
        bump(&mut families, row.get("family")?);
        bump(&mut statuses, row.get("status")?);
        bump(&mut reachability, row.get("reachability")?);
        row.finish()?;
        visited += 1;
    }
    assert_eq!(visited, 75_849);
    assert_eq!(visited, fixture.record_count());
    assert_eq!(fixture.header("record-count"), Some("75849"));
    for (name, actual) in [("operation-counts", &operations), ("family-counts", &families), ("status-counts", &statuses), ("reachability-counts", &reachability)] {
        assert_eq!(fixture.header(name), Some(census_text(actual).as_str()), "{name}");
    }
    Ok(())
}
#[test]
fn postproduction_guard_matches_exact_reference() -> Check {
    let fixture = Fixture::load(Fixture::repository_path("validation/fixtures/bonds/gilt-guard-adapter.fixture"))?;
    let columns = ["id", "z_bits", "a_bits", "guard_bits", "h_lower_bits", "z_lower_bits", "status", "boundary", "edge"];
    assert_eq!(fixture.columns(), &columns.map(str::to_owned));
    let mut status_counts = BTreeMap::new();
    let mut boundary_counts = BTreeMap::new();
    let mut edges = BTreeSet::new();
    let mut equalities = 0;
    let mut visited = 0;
    for record in fixture.records() {
        assert_eq!(record.id(), format!("GGUARD-{:04}", visited + 1));
        let z = f64::from_bits(hex(record.text("z_bits")?)?);
        assert!(z.is_finite());
        let a = z.abs() * GUARD_SCALE;
        let radius = ((a + CONSTANT_RADIUS).next_up() + GUARD_SCALE).next_up();
        assert_eq!(a.to_bits(), hex(record.text("a_bits")?)?, "{} scaled allowance", record.id());
        assert_eq!(radius.to_bits(), hex(record.text("guard_bits")?)?, "{} outward radius", record.id());
        let mut distances = [0.0; 2];
        for (slot, boundary) in distances.iter_mut().zip([LOG_OVERFLOW, LOG_HALF_MINIMUM]) {
            let d = (z - boundary).abs();
            *slot = if d.to_bits() & !SIGN == 0 { 0.0 } else { d.next_down().max(0.0) };
            equalities += usize::from(slot.to_bits() == radius.to_bits());
        }
        assert_eq!(distances[0].to_bits(), hex(record.text("h_lower_bits")?)?);
        assert_eq!(distances[1].to_bits(), hex(record.text("z_lower_bits")?)?);
        let expected = record.text("status")?;
        let boundary = record.text("boundary")?;
        let edge = record.text("edge")?;
        assert!(["ordinary", "constant-H", "constant-Z", "inside-low-H", "inside-high-H", "outside-low-H", "outside-high-H", "inside-low-Z", "inside-high-Z", "outside-low-Z", "outside-high-Z"].contains(&edge));
        if edge != "ordinary" { edges.insert(edge.to_owned()); }
        match expected {
            "pass" => {
                assert_eq!(boundary, "none");
                assert!(distances.iter().all(|d| *d > radius));
                assert!(guard(z, 17).is_ok(), "{} pass", record.id());
            }
            "boundary" => {
                if boundary == "H" { assert!(distances[0] <= radius); }
                else { assert_eq!(boundary, "Z"); assert!(distances[0] > radius && distances[1] <= radius); }
                match guard(z, 17) {
                    Err(GiltError::Rate { index: Some(17), phase: "discount factor", cause: RateError::NumericalBoundary { quantity: "discount factor", logarithm, .. }, .. }) => {
                        assert_eq!(logarithm.to_bits(), z.to_bits(), "{} exact boundary payload", record.id());
                    }
                    other => return Err(format!("{} expected precise guard error, got {other:?}", record.id()).into()),
                }
            }
            _ => return Err("unknown guard reference status".into()),
        }
        bump(&mut status_counts, expected);
        bump(&mut boundary_counts, boundary);
        visited += 1;
    }
    assert_eq!(visited, fixture.record_count());
    assert_eq!(fixture.header("record-count"), Some(visited.to_string().as_str()));
    assert_eq!(fixture.header("status-counts"), Some(census_text(&status_counts).as_str()));
    assert_eq!(fixture.header("boundary-counts"), Some(census_text(&boundary_counts).as_str()));
    assert_eq!(fixture.header("comparison-equalities"), Some(equalities.to_string().as_str()));
    assert_eq!(edges.len(), 10); // Two constants and both adjacent inside/outside edges at both bands.
    Ok(())
}
}
// END TEMPORARY GILT KERNEL ADAPTER
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
    if int(fields['record-count']) != 75849 or len(records.splitlines()) != 75849:
        raise ValueError('frozen numeric record census mismatch')
    for key, actual in (('body-sha256', sha(body)), ('records-sha256', sha(records)), ('body-fnv64', fnv(body))):
        if fields[key] != actual:
            raise ValueError('fixture integrity mismatch: ' + key)
    return fields, {'bytes': len(data), 'sha256': sha(data), 'body_sha256': sha(body),
                    'records_sha256': sha(records), 'record_count': 75849}


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



def guard_reference(timestamp):
    """New postproduction oracle: exact IEEE cells, not observed guard outputs.

    The full frozen 75,849-row body is untouched. This temporary extra corpus
    checks finite pure guard inputs; MAX/subnormal/zero probes need not occur
    as production quasi exponents. The reference encodes the contract's exact
    rounded additions and adjacent-bit operations with Fraction arithmetic.
    """
    sign, infinity, maximum = 1 << 63, 0x7ff0000000000000, 0x7fefffffffffffff
    mask = (1 << 64) - 1
    def power(exponent):
        return F(1 << exponent) if exponent >= 0 else F(1, 1 << -exponent)
    def represented(encoded):
        field, tail = (encoded >> 52) & 2047, encoded & ((1 << 52) - 1)
        assert field != 2047
        value = F(tail + ((1 << 52) if field else 0)) * power(field - 1075 if field else -1074)
        return -value if encoded & sign else value
    overflow_midpoint, zero_midpoint = power(1024) - power(970), power(-1075)
    def nearest(value):
        if not value:
            return 0
        magnitude = abs(value)
        if magnitude <= zero_midpoint:
            encoded = 0
        elif magnitude >= overflow_midpoint:
            encoded = infinity
        else:
            encoded = struct.unpack('>Q', struct.pack('>d', float(magnitude)))[0]
        lower = F(0) if encoded == 0 else (overflow_midpoint if encoded == infinity else (represented(encoded - 1) + represented(encoded)) / 2)
        upper = None if encoded == infinity else (zero_midpoint if encoded == 0 else (overflow_midpoint if encoded == maximum else (represented(encoded) + represented(encoded + 1)) / 2))
        assert magnitude >= lower and (upper is None or magnitude <= upper)
        if magnitude == lower or magnitude == upper:
            assert encoded & 1 == 0
        return encoded | (sign if value < 0 and encoded else 0)
    def ordered(encoded):
        return (~encoded & mask) if encoded & sign else encoded | sign
    def unorder(encoded):
        return encoded & ~sign if encoded & sign else ~encoded & mask
    boundaries = (('H', 0x40862e42fefa39ef), ('Z', 0xc0874910d52d3052))
    samples = {0, sign, 1, sign | 1, (1 << 52), sign | (1 << 52), maximum,
               sign | maximum, 0x3ff0000000000000, 0xbff0000000000000,
               (1047 << 52), sign | (1047 << 52), (961 << 52), sign | (961 << 52)}
    for _, encoded in boundaries:
        center = ordered(encoded)
        samples.update(unorder(center + offset) for offset in range(-64, 65))
    data = []
    equality_count = 0
    for encoded in sorted(samples, key=ordered):
        value = represented(encoded)
        a_bits = nearest(abs(value) * power(-50))
        first_up = nearest(represented(a_bits) + power(-44)) + 1
        radius_bits = nearest(represented(first_up) + power(-50)) + 1
        assert radius_bits < infinity
        distances = []
        owner = 'none'
        for name, boundary in boundaries:
            distance_bits = nearest(abs(value - represented(boundary)))
            assert 0 <= distance_bits < infinity
            lower_bits = distance_bits - 1 if distance_bits else 0
            distances.append(lower_bits)
            equality_count += lower_bits == radius_bits
            if owner == 'none' and represented(lower_bits) <= represented(radius_bits):
                owner = name
        edge = next(('constant-' + name for name, b in boundaries if encoded == b), 'ordinary')
        data.append(dict(z_bits=f'{encoded:016x}', a_bits=f'{a_bits:016x}', guard_bits=f'{radius_bits:016x}',
                         h_lower_bits=f'{distances[0]:016x}', z_lower_bits=f'{distances[1]:016x}',
                         status='pass' if owner == 'none' else 'boundary', boundary=owner, edge=edge))
    by_order = {ordered(int(record['z_bits'], 16)): record for record in data}
    for name, _ in boundaries:
        rejected = [ordered(int(record['z_bits'], 16)) for record in data if record['boundary'] == name]
        low, high = min(rejected), max(rejected)
        for key, label in ((low, 'inside-low-'), (high, 'inside-high-'), (low - 1, 'outside-low-'), (high + 1, 'outside-high-')):
            assert key in by_order and by_order[key]['edge'] == 'ordinary'
            by_order[key]['edge'] = label + name
        assert by_order[low - 1]['status'] == by_order[high + 1]['status'] == 'pass'
        assert all(by_order[key]['boundary'] == name for key in range(low, high + 1))
    columns = 'id z_bits a_bits guard_bits h_lower_bits z_lower_bits status boundary edge'.split()
    for index, record in enumerate(data, 1):
        record['id'] = f'GGUARD-{index:04d}'
    counts = {key: dict(sorted(__import__('collections').Counter(record[key] for record in data).items())) for key in ('status', 'boundary', 'edge')}
    records = ''.join('\t'.join(record[key] for key in columns) + '\n' for record in data).encode()
    body = ('\t'.join(columns) + '\n').encode() + records
    header = ['schema: 1', 'fixture-set: M8-GILT-POSTPRODUCTION-GUARD', 'domain: bonds',
              'description: Additive postproduction exact guard arithmetic and actual-helper checks; no frozen rows changed',
              'reference: independent exact Fraction IEEE midpoint/parity certificates and adjacent bits',
              'reference-version: Python standard library', 'reference-source: docs/M8_GILT_CONTRACT.md N3',
              'tolerance: TOL-E0; pure guard inputs, no public yield/libm reachability or financial enclosure claim',
              'reproduction: python3 -I validation/generator/gilt_numeric_kernel_check.py --output NEW_DIRECTORY',
              'generated-at: ' + timestamp, 'platform: ' + platform.platform() + '; Python ' + platform.python_version(),
              'record-count: ' + str(len(data)), 'body-sha256: ' + sha(body), 'body-fnv64: ' + fnv(body),
              'records-sha256: ' + sha(records), 'status-counts: ' + json.dumps(counts['status'], separators=(',', ':')),
              'boundary-counts: ' + json.dumps(counts['boundary'], separators=(',', ':')),
              'comparison-equalities: ' + str(equality_count)]
    return ('\n'.join(header) + '\n\n').encode() + body, {'record_count': len(data), 'body_sha256': sha(body),
             'records_sha256': sha(records), 'counts': counts, 'comparison_equalities': equality_count,
             'scope': 'postproduction exact arithmetic oracle; two bands plus adjacent edges, no claim of blind production independence'}

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
    destination = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix='sabiquant-gilt-kernel-'))
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
                'word_accessor': {'cfg_test_only': True, 'bytes': len(WIDE_ACCESSOR.encode()), 'sha256': sha(WIDE_ACCESSOR.encode())},
                'pinned_inputs': manifest(pinned), 'source_manifest': manifest(original),
                'source_manifest_sha256': sha(canonical(manifest(original))), 'test_name': TEST_NAME,
                'scope': '75849 frozen primitive rows plus additive postproduction guard; supplemental tests separate from standard suite',
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
        guard_data, guard_info = guard_reference(evidence['started_at'])
        evidence['postproduction_guard'] = dict(guard_info, bytes=len(guard_data), sha256=sha(guard_data))
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
            if b'validation_gilt_kernel' in source or b'validation_gilt_words' in source:
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
        installed_guard = crate / GUARD_FIXTURE
        installed_guard.write_bytes(guard_data)
        assert installed_guard.read_bytes() == guard_data
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
        for label, release in (('guard-debug', []), ('guard-release', ['--release'])):
            stdout = run([cargo, 'test', '--offline', '--locked', *release, '--lib', GUARD_TEST,
                          '--', '--exact', '--test-threads=1', '--nocapture'], crate, environment,
                         destination, label, evidence).decode()
            if not re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;', stdout):
                raise RuntimeError('exactly one guard test did not execute: ' + label)
        run([cargo, 'clippy', '--offline', '--locked', '--lib', '--tests', '--', '-D', 'warnings'],
            crate, environment, destination, 'copied-clippy', evidence)
        evidence['all_fixture_rows_checked_per_test'] = 75849
        evidence['supplemental_test_executions'] = 4
        evidence['guard_rows_checked_per_test'] = guard_info['record_count']
        changed = sorted(name for name, data in original.items() if (crate / name).read_bytes() != data)
        if changed != sorted(injected) or any((crate / name).read_bytes() != data for name, data in injected.items()):
            raise RuntimeError('unexpected copied source changes')
        evidence['copied_source_changes'] = changed
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
