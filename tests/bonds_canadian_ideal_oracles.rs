//! FR-CAD-09/23/25/28/33..60/72: frozen independent ideal price/risk/inverse
//! references through the complete public Canadian API. Conditional budgets
//! are compared as exact rationals; no blanket ULP allowance is added.
#[path = "support/treasury.rs"]
mod arithmetic;
#[path = "support/canadian_ideal.rs"]
mod canadian_support;
mod support;

use arithmetic::{Rat, bits, bump};
use canadian_support::*;
use sabiquant::bonds::{
    BondYieldSensitivity, CanadianError, CanadianPrice, CanadianPriceBasis,
    CanadianYieldConvention, CanadianYieldError,
};
use sabiquant::error::ValueError;
use sabiquant::math::{RootBracket, SolverError, SolverOptions, TerminationReason};
use std::collections::BTreeMap;
use std::ops::Bound;
use support::fixture::Record;

const METRICS: [(&str, &str); 9] = [
    ("model_dirty", "model dirty price"),
    ("clean", "clean price"),
    ("settlement_dirty", "settlement dirty price"),
    ("price_derivative", "price derivative"),
    ("price_second_derivative", "price second derivative"),
    ("time_weighted_amount", "time weighted amount"),
    ("modified_duration", "modified duration"),
    ("macaulay_duration", "macaulay duration"),
    ("convexity", "convexity"),
];

macro_rules! metadata {
    ($value:expr,$plan:expr,$bond:expr,$yield:expr) => {{
        let value = &$value;
        let plan = &$plan;
        let bond = &$bond;
        let schedule = bond.schedule();
        assert_eq!(value.source_version(), SOURCE);
        assert_eq!(value.source_policy(), bond.source_policy());
        assert_eq!(value.accrual_start(), schedule.accrual_start());
        assert_eq!(value.original_issue_date(), bond.original_issue_date());
        assert_eq!(value.stated_maturity_date(), schedule.stated_maturity());
        assert_eq!(value.true_maturity_date(), bond.true_maturity_date());
        assert_eq!(value.first_coupon_date(), bond.first_coupon_date());
        assert_eq!(
            value.last_coupon_start_date(),
            bond.last_coupon_start_date()
        );
        assert_eq!(value.first_period(), schedule.first_period());
        assert_eq!(value.last_period(), schedule.last_period());
        assert_eq!(value.frequency(), schedule.frequency());
        assert_eq!(value.coupon_per_100(), bond.coupon_per_100());
        assert_eq!(value.end_of_month(), schedule.end_of_month());
        assert_eq!(value.direct_cycle_anchor(), schedule.direct_cycle_anchor());
        assert_eq!(value.settlement_date(), plan.settlement_date());
        assert_eq!(value.yield_selection(), plan.yield_selection());
        assert_eq!(value.yield_convention(), plan.yield_convention());
        assert_eq!(
            value.valuation_accrued_per_100().to_bits(),
            plan.valuation_accrued_per_100().to_bits()
        );
        assert_eq!(
            value.settlement_accrued_per_100().to_bits(),
            plan.settlement_accrued_per_100().to_bits()
        );
        assert_eq!(
            value.quote_accrued_per_100().to_bits(),
            plan.quote_accrued_per_100().to_bits()
        );
        let y: f64 = $yield;
        assert_eq!(
            value.yield_rate().to_bits(),
            if y.to_bits() & 0x7fff_ffff_ffff_ffff == 0 {
                0
            } else {
                y.to_bits()
            }
        );
    }};
}

fn range(id: &str, phase: &str, class: &str, result: &Result<f64, CanadianError>) {
    let negative = class == "underflow_negative_zero";
    let valid = match result {
        Err(CanadianError::Value {
            index: None,
            phase: actual,
            cause:
                ValueError::ResultOutOfDomain {
                    quantity,
                    value,
                    domain,
                    ..
                },
            ..
        }) => {
            *actual == phase
                && *quantity == phase
                && value.to_bits() == if negative { 1_u64 << 63 } else { 0 }
                && if negative {
                    domain.lower() == Bound::Unbounded && domain.upper() == Bound::Excluded(0.0)
                } else {
                    domain.lower() == Bound::Excluded(0.0) && domain.upper() == Bound::Unbounded
                }
        }
        _ => false,
    };
    assert!(valid, "{id}: expected {phase}/{class}, got {result:?}");
}

/// All881 valuations and each of their nine conditional quantity references.
/// Exact interval endpoints/allowances are never converted to rounded f64 first.
#[test]
fn ideal_three_prices_and_six_independent_risks() -> Checked {
    let fixture = fixture(false)?;
    let mut profiles = BTreeMap::new();
    let mut classes = BTreeMap::new();
    let mut ordinary_regimes = BTreeMap::new();
    let mut values = 0;
    let mut ranges = 0;
    for (position, row) in fixture.records().enumerate() {
        let id = row.id();
        assert_eq!(id, format!("CAD-IDEAL-{:04}", position + 1));
        assert_eq!(row.text("operation")?, "evaluate_and_six_risk_getters");
        assert_eq!(row.text("observation_class")?, "value_under_profile");
        bump(&mut profiles, row.text("profile")?);
        let bond = instrument(&row)?;
        let plan = plan(&row, &bond)?;
        assert_eq!(
            plan.valuation_accrued_per_100().to_bits(),
            Rat::parse(row.text("source_a")?)?.binary_round()?.to_bits(),
            "{id} A"
        );
        assert_eq!(
            plan.settlement_accrued_per_100().to_bits(),
            Rat::parse(row.text("source_ai")?)?
                .binary_round()?
                .to_bits(),
            "{id} AI"
        );
        let ordinals = row
            .text("original_ordinals")?
            .split(';')
            .map(str::parse)
            .collect::<Result<Vec<usize>, _>>()?;
        assert_eq!(plan.bundle_count(), ordinals.len(), "{id}");
        for (i, ordinal) in ordinals.into_iter().enumerate() {
            assert_eq!(plan.bundle_coupon_index(i), Some(ordinal), "{id}");
        }
        assert_eq!(plan.bundle_coupon_index(plan.bundle_count()), None);
        let y = bits(row.text("yield_bits")?)?;
        let valuation = plan
            .evaluate(y)
            .map_err(|e| format!("{id} value-under-profile rejected {e:?}"))?;
        let risk = plan
            .risk(y)
            .map_err(|e| format!("{id} risk traversal rejected {e:?}"))?;
        metadata!(valuation, plan, bond, y);
        metadata!(risk, plan, bond, y);
        assert_eq!(
            risk.model_dirty_per_100().to_bits(),
            valuation.model_dirty_per_100().to_bits(),
            "{id}"
        );
        assert_eq!(
            valuation.model_dirty_price().basis(),
            CanadianPriceBasis::ModelDirty
        );
        assert_eq!(valuation.clean_price()?.basis(), CanadianPriceBasis::Clean);
        assert_eq!(
            valuation.settlement_dirty_price()?.basis(),
            CanadianPriceBasis::SettlementDirty
        );
        let outcomes = [
            Ok(valuation.model_dirty_per_100()),
            valuation.clean_price().map(CanadianPrice::value),
            valuation.settlement_dirty_price().map(CanadianPrice::value),
            risk.price_derivative(),
            risk.price_second_derivative(),
            risk.time_weighted_amount(),
            risk.modified_duration(),
            risk.macaulay_duration(),
            risk.convexity(),
        ];
        for ((prefix, quantity), outcome) in METRICS.into_iter().zip(outcomes) {
            let class = row.text(&format!("{prefix}_class"))?;
            bump(&mut classes, class);
            match class {
                "value" => {
                    let value = outcome
                        .map_err(|e| format!("{id} {quantity} unexpectedly rejected {e:?}"))?;
                    assert!(
                        within_interval(
                            value,
                            row.text(&format!("{prefix}_lo"))?,
                            row.text(&format!("{prefix}_hi"))?,
                            row.text(&format!("{prefix}_error"))?
                        )?,
                        "{id} {quantity}: {value:e} outside frozen conditional interval"
                    );
                    assert_eq!(
                        row.text(&format!("{prefix}_informative"))?,
                        "true",
                        "{id} {quantity}"
                    );
                    values += 1;
                }
                "underflow_negative_zero" | "underflow_positive_zero" => {
                    range(id, quantity, class, &outcome);
                    ranges += 1;
                }
                other => return Err(format!("{id} unknown frozen class {other}").into()),
            }
        }
        if row.text("profile")? == "ordinary" {
            let key = [
                row.text("frequency")?,
                row.text("eom")?,
                row.text("geometry")?,
                row.text("selection")?,
            ]
            .join("/");
            *ordinary_regimes.entry(key).or_insert(0_usize) += 1;
        }
    }
    assert_eq!(profiles.get("ordinary"), Some(&648));
    assert_eq!(ordinary_regimes.len(), 216);
    assert!(ordinary_regimes.values().all(|count| *count == 3));
    assert_eq!(values, 7890);
    assert_eq!(ranges, 39);
    assert_eq!(values + ranges, 881 * 9);
    assert_eq!(classes.get("underflow_negative_zero"), Some(&10));
    assert_eq!(classes.get("underflow_positive_zero"), Some(&29));
    Ok(())
}

fn target_basis(text: &str) -> Checked<CanadianPriceBasis> {
    Ok(match text {
        "Clean" => CanadianPriceBasis::Clean,
        "ModelDirty" => CanadianPriceBasis::ModelDirty,
        "SettlementDirty" => CanadianPriceBasis::SettlementDirty,
        _ => return Err("target basis".into()),
    })
}

fn check_cached_report(
    row: &Record<'_>,
    report: sabiquant::bonds::CanadianYieldReport,
    target: CanadianPrice,
    bracket: RootBracket,
    options: SolverOptions,
) -> Checked {
    let id = row.id();
    let solve = report.solve_report();
    let y = report.yield_rate();
    assert_eq!(report.target(), target);
    assert_eq!(report.initial_bracket(), bracket);
    assert_eq!(report.options(), options);
    assert_eq!(
        report.model_dirty_target_per_100().to_bits(),
        bits(row.text("cached_model_target_bits")?)?.to_bits(),
        "{id}"
    );
    assert_eq!(
        report.sensitivity(),
        BondYieldSensitivity::MathematicallyStrictlyDecreasing
    );
    assert!(
        y.is_finite() && bracket.lower() <= y && y <= bracket.upper(),
        "{id}"
    );
    assert_eq!(y.to_bits(), solve.point().to_bits());
    assert_eq!(solve.evaluations().derivative(), 0);
    assert!(
        solve.evaluations().function() >= 1
            && solve.evaluations().function() <= options.max_evaluations()
    );
    assert!(matches!(
        solve.reason(),
        TerminationReason::FunctionZero | TerminationReason::BracketTolerance
    ));
    assert!(
        bracket.lower() <= solve.bracket().lower() && solve.bracket().upper() <= bracket.upper()
    );
    assert!(solve.bracket().lower() <= y && y <= solve.bracket().upper());
    let difference = Rat::binary(report.model_dirty_per_100())
        .sub(&Rat::binary(report.model_dirty_target_per_100()));
    assert_eq!(
        report.amount_residual()?.to_bits(),
        difference.binary_round()?.to_bits(),
        "{id} cached amount residual"
    );
    let scale = u8::from(report.model_dirty_target_per_100().to_bits() >= 1993_u64 << 52);
    assert_eq!(report.residual_scale_exponent(), scale);
    assert_eq!(
        solve.residual().to_bits(),
        difference
            .divide_pow2(u32::from(scale))
            .binary_round()?
            .to_bits(),
        "{id} cached callback residual"
    );
    assert_eq!(
        report.model_dirty_price().basis(),
        CanadianPriceBasis::ModelDirty
    );
    let clean =
        Rat::binary(report.model_dirty_per_100()).sub(&Rat::binary(report.quote_accrued_per_100()));
    assert_eq!(
        report.clean_price()?.value().to_bits(),
        clean.binary_round()?.to_bits(),
        "{id} one-shot clean"
    );
    let settlement = if report.yield_convention() == CanadianYieldConvention::MoneyMarket {
        Rat::binary(report.model_dirty_per_100())
    } else {
        add(&clean, &Rat::binary(report.settlement_accrued_per_100()))?
    };
    assert_eq!(
        report.settlement_dirty_price()?.value().to_bits(),
        settlement.binary_round()?.to_bits(),
        "{id} one-shot settlement bridge"
    );
    Ok(())
}

/// All349 target-selected references.345 coordinate comparisons require the
/// exact cached residual predicate; the remaining four have no ideal-root claim.
#[test]
fn target_selected_inverses_use_exact_cached_residual_condition() -> Checked {
    let fixture = fixture(true)?;
    let mut informative = 0;
    let mut outside = 0;
    let mut observed = 0;
    let mut equations = BTreeMap::new();
    let mut profiles = BTreeMap::new();
    for (position, row) in fixture.records().enumerate() {
        let id = row.id();
        assert_eq!(id, format!("CAD-INVERSE-{:04}", position + 1));
        assert_eq!(row.text("operation")?, "yield_from_price");
        bump(&mut profiles, row.text("profile")?);
        bump(&mut equations, row.text("reference_equation")?);
        let bond = instrument(&row)?;
        let plan = plan(&row, &bond)?;
        let target = CanadianPrice::new(
            bits(row.text("target_bits")?)?,
            target_basis(row.text("target_basis")?)?,
        )?;
        let bracket = RootBracket::new(
            bits(row.text("bracket_lower_bits")?)?,
            bits(row.text("bracket_upper_bits")?)?,
        )?;
        let options = SolverOptions::new(
            bits(row.text("absolute_tolerance_bits")?)?,
            bits(row.text("relative_tolerance_bits")?)?,
            row.parse("evaluation_budget")?,
        )?;
        let outcome = plan.yield_from_price(target, bracket, options);
        if row.text("profile")? == "ideal_image" {
            assert_eq!(row.text("ideal_root_class")?, "outside_open_ideal_image");
            assert_eq!(row.text("informative")?, "false");
            assert_eq!(row.text("root_lo")?, "");
            assert_eq!(row.text("root_hi")?, "");
            match outcome {
                Err(CanadianYieldError::Solver(SolverError::NotBracketed{bracket:actual,lower_value,upper_value,evaluations,..}))=>{
                    assert_eq!(actual,bracket);assert!(lower_value>0.0 && upper_value>0.0,"{id}");
                    assert_eq!(evaluations.function(),2);assert_eq!(evaluations.derivative(),0);
                },other=>return Err(format!("{id}: far-below-image positive target must preserve counted same-sign failure, got {other:?}").into()),
            }
            outside += 1;
            continue;
        }
        let report = outcome.map_err(|e| format!("{id} inverse failed {e:?}"))?;
        metadata!(report, plan, bond, report.yield_rate());
        check_cached_report(&row, report, target, bracket, options)?;
        if row.text("observed_protocol")? == "cached_upper_function_zero" {
            assert_eq!(
                row.text("reference_equation")?,
                "no_ideal_root_observed_protocol"
            );
            assert_eq!(row.text("root_lo")?, "");
            assert_eq!(row.text("root_hi")?, "");
            assert_eq!(
                report.solve_report().reason(),
                TerminationReason::FunctionZero
            );
            assert_eq!(report.yield_rate().to_bits(), bracket.upper().to_bits());
            assert_eq!(
                report.solve_report().evaluations().function(),
                row.parse::<u32>("function_calls")?
            );
            assert_eq!(
                report.solve_report().evaluations().derivative(),
                row.parse::<u32>("derivative_calls")?
            );
            assert_eq!(
                report.model_dirty_per_100().to_bits(),
                target.value().to_bits()
            );
            observed += 1;
            continue;
        }
        assert_eq!(row.text("ideal_root_class")?, "unique_sign_certified");
        assert_eq!(row.text("informative")?, "true");
        let exact_difference = Rat::binary(report.model_dirty_per_100())
            .sub(&Rat::binary(report.model_dirty_target_per_100()));
        assert!(
            exact_difference.abs_le(&dyadic(row.text("residual_budget")?)?),
            "{id} exact cached residual exceeds predeclared condition"
        );
        // coordinate_error already contains the full reference-root width.
        // Compare to the certified lower endpoint without adding it again.
        assert!(
            Rat::binary(report.yield_rate())
                .sub(&dyadic(row.text("root_lo")?)?)
                .abs_le(&dyadic(row.text("coordinate_error")?)?),
            "{id} conditioned coordinate bound"
        );
        assert!(
            dyadic(row.text("coordinate_error")?)?.abs_le(&Rat::parse("0.0000001")?),
            "{id} informative coordinate policy"
        );
        let reference = Rat::parse(row.text("reference_model_target")?)?;
        let shift = Rat::binary(report.model_dirty_target_per_100()).sub(&reference);
        assert!(
            shift.abs_le(&dyadic(row.text("target_shift_error")?)?),
            "{id} reference-target shift bound"
        );
        if row.text("reference_equation")? == "cached_resolved_model_dirty" {
            assert!(shift.is_zero(), "{id} cached reference equation");
        }
        informative += 1;
    }
    assert_eq!((informative, outside, observed), (345, 3, 1));
    assert_eq!(profiles.get("ordinary_target"), Some(&324));
    assert_eq!(equations.get("external_exact_source"), Some(&336));
    assert_eq!(equations.get("cached_resolved_model_dirty"), Some(&12));
    assert_eq!(equations.get("no_ideal_root_observed_protocol"), Some(&1));
    Ok(())
}

/// FR-CAD-59:432 nonzero ordinary native quotes, each in all three typed bases.
/// These1,296 derived roundtrips are consistency checks, not new independent
/// financial references. The1e-8 coordinate threshold is fixed before execution.
#[test]
fn nonzero_native_quote_roundtrips_cover_every_ordinary_regime() -> Checked {
    let fixture = fixture(false)?;
    let mut inputs = 0;
    let mut roundtrips = 0;
    for row in fixture.records() {
        if row.text("profile")? != "ordinary" {
            continue;
        }
        let y = bits(row.text("yield_bits")?)?;
        if y.to_bits() & 0x7fff_ffff_ffff_ffff == 0 {
            continue;
        }
        let bond = instrument(&row)?;
        let plan = plan(&row, &bond)?;
        let value = plan.evaluate(y)?;
        for target in [
            value.model_dirty_price(),
            value.clean_price()?,
            value.settlement_dirty_price()?,
        ] {
            let bracket = RootBracket::new(y - 0.02, y + 0.02)?;
            let options = SolverOptions::new(1e-12, 0.0, 128)?;
            let result = plan.yield_from_price(target, bracket, options)?;
            assert!(
                Rat::binary(result.yield_rate())
                    .sub(&Rat::binary(y))
                    .abs_le(&Rat::parse("0.00000001")?),
                "{} {:?} roundtrip",
                row.id(),
                target.basis()
            );
            assert_eq!(result.target(), target);
            assert_eq!(result.solve_report().evaluations().derivative(), 0);
            assert!(result.solve_report().evaluations().function() <= 128);
            let difference = Rat::binary(result.model_dirty_per_100())
                .sub(&Rat::binary(result.model_dirty_target_per_100()));
            assert_eq!(
                result.amount_residual()?.to_bits(),
                difference.binary_round()?.to_bits()
            );
            roundtrips += 1;
        }
        inputs += 1;
    }
    assert_eq!((inputs, roundtrips), (432, 1296));
    Ok(())
}
