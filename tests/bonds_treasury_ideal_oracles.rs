//! FR-UST-03..17/25: the frozen preproduction ideal reference corpus.
//! 714 price/risk rows (5,712 quantity classifications) and 145 inverse rows.
//! Conditional absolute bounds are compared with exact test-only rational
//! arithmetic, including the serialized decimal allowance; no blanket ULP
//! tolerance is added. Ideal range labels do not override the source-PV gate.
//! The 68 uninformative quantities, eight uninformative inverse coordinates
//! and single ideal-image-only inverse remain explicitly qualified.
mod support;
#[path = "support/treasury.rs"]
mod treasury_support;
use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding};
use sabiquant::bonds::{
    BondPrice, BondPriceKind, BondYieldSensitivity, TreasuryError, TreasuryPricingPlan,
    TreasuryYieldError, UsTreasury,
};
use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{RootBracket, SolverOptions, TerminationReason};
use std::collections::BTreeMap;
use std::ops::Bound;
use support::fixture::{Fixture, Record};
use treasury_support::*;
const PRICE: &str = "validation/fixtures/bonds/treasury-ideal-price-risk-oracle.fixture";
const INVERSE: &str = "validation/fixtures/bonds/treasury-ideal-inverse-oracle.fixture";
const METRICS: [(&str, &str); 8] = [
    ("dirty", "dirty"),
    ("clean", "clean"),
    ("d1", "price derivative"),
    ("d2", "price second derivative"),
    ("moment", "time weighted amount"),
    ("modified", "modified duration"),
    ("macaulay", "macaulay duration"),
    ("convexity", "convexity"),
];
fn fixture(inverse: bool) -> TestResult<Fixture> {
    let (path, sha, count, cols, set) = if inverse {
        (
            INVERSE,
            "40a16012bd1378122b6f17f1792bad8b66433487c926ff3f40d85a266259e517",
            145,
            51,
            "M8-US-TREASURY-IDEAL-INVERSE",
        )
    } else {
        (
            PRICE,
            "08bdb9ce17a1c523ed7cbe456185be1b305a950ef3261ca0148ff5a8471b4a31",
            714,
            74,
            "M8-US-TREASURY-IDEAL-PRICE-RISK",
        )
    };
    let f = pinned(path, sha, count, cols)?;
    assert_eq!(f.header("fixture-set"), Some(set));
    assert_eq!(
        f.header("generator-sha256"),
        Some("ca822f899925583ccb15187c110c05bd0d29e8e6ec932ba859461c5942bdb5b8")
    );
    Ok(f)
}
fn indices(s: &str) -> TestResult<Vec<usize>> {
    let (a, b) = s.split_once("..").ok_or("index range")?;
    Ok((a.parse()?..=b.parse()?).collect())
}
fn prepared<'a>(r: &Record<'_>, t: &'a UsTreasury) -> TestResult<TreasuryPricingPlan<'a>> {
    let id = r.id();
    assert_eq!(t.source_version(), SOURCE, "{id}");
    assert_eq!(
        t.source_policy(),
        policy(r.text("daily_basis")?, r.text("tie_rule")?)?
    );
    assert_eq!(t.original_dated_date(), date(r.text("dated")?)?);
    assert_eq!(t.original_issue_date(), date(r.text("issue")?)?);
    assert_eq!(t.maturity_date(), date(r.text("maturity")?)?);
    assert_eq!(t.coupon_per_100(), r.text("coupon")?.parse()?);
    assert_eq!(t.first_coupon_date(), date(r.text("first_coupon")?)?);
    assert_eq!(format!("{:?}", t.first_period()), r.text("first_period")?);
    assert_eq!(t.end_of_month().to_string(), r.text("eom")?);
    assert_eq!(t.quasi_dates().len(), r.parse::<usize>("quasi_count")?);
    assert_eq!(
        t.coupon_dates(),
        &t.quasi_dates()[t.first_coupon_quasi_index()..]
    );
    let p = TreasuryPricingPlan::new(t, date(r.text("settlement")?)?, 20_000)?;
    assert!(std::ptr::eq(p.treasury(), t));
    assert_eq!(p.source_version(), SOURCE);
    assert_eq!(p.source_policy(), t.source_policy());
    assert_eq!(p.settlement_date(), date(r.text("settlement")?)?);
    assert_eq!(p.next_quasi_index(), r.parse::<usize>("next_index")?);
    assert_eq!(p.remaining_days(), r.parse::<u32>("r")?);
    assert_eq!(p.quasi_period_days(), r.parse::<u32>("s")?);
    assert_eq!(p.remaining_quasi_periods(), r.parse::<u32>("n")?);
    for (k, v) in [
        ("raw_ai", p.raw_accrued_per_100()),
        ("published_ai", p.accrued_per_100()),
    ] {
        assert_eq!(
            Rat::parse(r.text(k)?)?.binary_round()?.to_bits(),
            v.to_bits(),
            "{id} cached {k}"
        );
    }
    let expected = indices(r.text("coupon_indices")?)?;
    assert_eq!(p.bundle_count(), expected.len());
    for (i, q) in expected.iter().enumerate() {
        assert_eq!(p.bundle_quasi_index(i), Some(*q), "{id} bundle{i}");
    }
    assert_eq!(p.bundle_quasi_index(expected.len()), None);
    // Bind the complete raw A_j run encoding through the public raw coupon
    // views, subtracting the contractual redemption100 only at maturity.
    let mut source = Vec::new();
    for run in r.text("source_amounts")?.split(';') {
        let (value, count) = run.split_once('*').ok_or("source run")?;
        for _ in 0..count.parse::<usize>()? {
            source.push(Rat::parse(value)?);
        }
    }
    assert_eq!(source.len(), expected.len());
    let positive = source.iter().filter(|v| !v.is_zero()).count();
    assert_eq!(positive, r.parse::<usize>("positive_pv_count")?);
    for (q, source) in expected.iter().zip(&source) {
        let value = if *q == t.quasi_dates().len() - 1 {
            source.sub(&Rat::parse("100/1")?)
        } else {
            source.clone()
        };
        let ordinal = q - t.first_coupon_quasi_index();
        for scale in [0, 38] {
            let m = AmountRounding::NearestTiesEven;
            let result = t.raw_coupon_per_100(ordinal, AmountQuantum::new(1, scale)?, m);
            match (value.rounded(scale, m)?, result) {
                (Ok(e), Ok(a)) => assert_eq!(a, e, "{id} source{q} scale{scale}"),
                (
                    Err((kind, negative)),
                    Err(TreasuryError::CashArithmetic {
                        stage: "raw coupon",
                        endpoint: "point",
                        cause,
                        ..
                    }),
                ) => {
                    assert_eq!(cause.kind(), kind);
                    if let AmountError::OutOfRange { negative: n, .. } = cause {
                        assert_eq!(n, negative)
                    }
                }
                (e, a) => return Err(format!("{id} source{q} expected{e:?} actual{a:?}").into()),
            }
        }
    }
    for ordinal in 0..t.coupon_count() {
        use sabiquant::bonds::CouponEntitlement;
        assert_eq!(
            p.coupon_entitlement(ordinal),
            Some(if t.coupon_dates()[ordinal] <= p.settlement_date() {
                CouponEntitlement::Ceased
            } else {
                CouponEntitlement::BuyerEntitled
            })
        );
    }
    assert_eq!(p.coupon_entitlement(t.coupon_count()), None);
    let names = if r.text("input_fingerprint").is_ok() {
        if r.text("target_kind").is_ok() {
            vec![
                "dated",
                "issue",
                "maturity",
                "coupon",
                "first_coupon",
                "eom",
                "quasi_count",
                "settlement",
                "daily_basis",
                "tie_rule",
                "target_kind",
                "target_bits",
                "caller_lo_bits",
                "caller_hi_bits",
                "abs_tol_bits",
                "rel_tol_bits",
                "max_evals",
            ]
        } else {
            vec![
                "dated",
                "issue",
                "maturity",
                "coupon",
                "first_coupon",
                "eom",
                "quasi_count",
                "settlement",
                "daily_basis",
                "tie_rule",
                "yield_bits",
            ]
        }
    } else {
        return Err("fingerprint column".into());
    };
    let input = names
        .iter()
        .map(|k| r.text(k))
        .collect::<Result<Vec<_>, _>>()?
        .join("\t");
    assert_eq!(
        sha256(input.as_bytes())?,
        r.text("input_fingerprint")?,
        "{id} row input fingerprint"
    );
    assert!(!r.text("requirements")?.is_empty());
    Ok(p)
}
macro_rules! metadata {
    ($v:expr,$p:expr,$t:expr,$id:expr) => {{
        let v = &$v;
        let p = &$p;
        let t = &$t;
        assert_eq!(v.source_version(), SOURCE, "{}", $id);
        assert_eq!(v.source_policy(), t.source_policy());
        assert_eq!(v.original_dated_date(), t.original_dated_date());
        assert_eq!(v.original_issue_date(), t.original_issue_date());
        assert_eq!(v.maturity_date(), t.maturity_date());
        assert_eq!(v.coupon_per_100(), t.coupon_per_100());
        assert_eq!(v.first_coupon_date(), t.first_coupon_date());
        assert_eq!(v.first_period(), t.first_period());
        assert_eq!(v.end_of_month(), t.end_of_month());
        assert_eq!(v.settlement_date(), p.settlement_date());
        assert_eq!(v.next_quasi_date(), t.quasi_dates()[p.next_quasi_index()]);
        assert_eq!(v.remaining_quasi_periods(), p.remaining_quasi_periods());
        assert_eq!(v.remaining_days(), p.remaining_days());
        assert_eq!(v.quasi_period_days(), p.quasi_period_days());
        assert_eq!(
            v.raw_accrued_per_100().to_bits(),
            p.raw_accrued_per_100().to_bits()
        );
        assert_eq!(v.accrued_per_100().to_bits(), p.accrued_per_100().to_bits());
    }};
}
fn financial(e: &TreasuryError) -> bool {
    matches!(
        e,
        TreasuryError::Value {
            cause: ValueError::ResultNotFinite { .. } | ValueError::ResultOutOfDomain { .. },
            ..
        } | TreasuryError::Sum { .. }
    )
}
fn range(id: &str, phase: &str, class: &str, outcome: &Result<f64, TreasuryError>) {
    let negative = class.starts_with("negative");
    let ok = match outcome {
        Err(TreasuryError::Value {
            index: None,
            phase: p,
            cause,
            ..
        }) if *p == phase => match cause {
            ValueError::ResultNotFinite {
                quantity, value, ..
            } => {
                class.ends_with("overflow")
                    && *quantity == phase
                    && *value
                        == if negative {
                            NonFinite::NegativeInfinity
                        } else {
                            NonFinite::PositiveInfinity
                        }
            }
            ValueError::ResultOutOfDomain {
                quantity,
                value,
                domain,
                ..
            } => {
                class.ends_with("underflow")
                    && *quantity == phase
                    && *value == 0.
                    && value.is_sign_negative() == negative
                    && if negative {
                        domain.lower() == Bound::Unbounded && domain.upper() == Bound::Excluded(0.)
                    } else {
                        domain.lower() == Bound::Excluded(0.) && domain.upper() == Bound::Unbounded
                    }
            }
            _ => false,
        },
        _ => false,
    };
    assert!(ok, "{id} {phase} ideal range {class}: {outcome:?}");
}
/// FR-UST-07..15: all714 public valuations and six independent retained risk
/// getters, conditional on the frozen N3 source-PV and N5 hypotheses.
#[test]
fn ideal_price_and_six_risks_with_classified_bounds() -> TestResult {
    let f = fixture(false)?;
    let mut count = BTreeMap::new();
    let mut source_displays = BTreeMap::new();
    let mut gated = 0;
    let mut direct = 0;
    let mut uninformative = 0;
    let mut ranges = 0;
    for (position, r) in f.records().enumerate() {
        let id = r.id();
        assert_eq!(id, format!("UST-IDEAL-{:05}", position + 1));
        let t = ideal_instrument(&r)?;
        let p = prepared(&r, &t)?;
        let y = bits(r.text("yield_bits")?)?;
        let valuation = p.evaluate(y);
        let risk = p.risk(y);
        let (v, risk) = match (valuation, risk) {
            (Ok(v), Ok(k)) => (v, k),
            (Err(e), Err(other)) => {
                assert_eq!(
                    r.text("pv_gate")?,
                    "requires-indexed-pv-gate",
                    "{id} guaranteed gate rejected {e:?}"
                );
                assert_eq!(e, other);
                assert!(
                    matches!(
                        e,
                        TreasuryError::Value {
                            index: Some(_),
                            phase: "source present value",
                            ..
                        }
                    ) && financial(&e),
                    "{id} {e:?}"
                );
                gated += 1;
                for (k, _) in METRICS {
                    bump(&mut count, r.text(&format!("{k}_comparison"))?);
                }
                continue;
            }
            (v, k) => return Err(format!("{id} traversal mismatch {v:?} vs{k:?}").into()),
        };
        metadata!(v, p, t, id);
        metadata!(risk, p, t, id);
        let canonical = if y == 0. { 0. } else { y };
        assert_eq!(v.yield_rate().to_bits(), canonical.to_bits());
        assert_eq!(risk.yield_rate().to_bits(), canonical.to_bits());
        assert_eq!(v.dirty_per_100().to_bits(), risk.dirty_per_100().to_bits());
        assert_eq!(v.dirty_price().kind(), BondPriceKind::Dirty);
        assert_eq!(
            v.dirty_price().value_per_100_face().to_bits(),
            v.dirty_per_100().to_bits()
        );
        let clean = v.clean_price().map(BondPrice::value_per_100_face);
        if let Ok(c) = clean {
            let expected = Rat::binary(v.dirty_per_100())
                .sub(&Rat::binary(p.accrued_per_100()))
                .binary_round()?;
            assert_eq!(
                c.to_bits(),
                expected.to_bits(),
                "{id} clean public reduction"
            );
        }
        // FR-UST-01/12/25, UST-37: the frozen primary A-G displays are
        // six-place CLEAN P. Quantize only this test observation; the public
        // kernel continues to return its full native per100 price.
        let display = r.text("source_display")?;
        if display != "-" {
            assert!(r.text("family")?.starts_with("SOURCE-"));
            let observed = v.clean_price()?.value_per_100_face();
            assert_eq!(
                Rat::binary(observed).rounded(6, t.source_policy().tie_rule().amount_rounding())?,
                Ok(display.parse()?),
                "{id} primary clean display"
            );
            bump(&mut source_displays, r.text("family")?);
        }
        let outcomes = [
            Ok(v.dirty_per_100()),
            clean,
            risk.price_derivative(),
            risk.price_second_derivative(),
            risk.time_weighted_amount(),
            risk.modified_duration(),
            risk.macaulay_duration(),
            risk.convexity(),
        ];
        for ((prefix, phase), value) in METRICS.iter().zip(&outcomes) {
            let comparison = r.text(&format!("{prefix}_comparison"))?;
            bump(&mut count, comparison);
            match comparison {
                "conditional-absolute" => {
                    let observed = value
                        .as_ref()
                        .map_err(|e| format!("{id} {prefix} conditional rejected {e:?}"))?;
                    assert!(observed.is_finite(), "{id} {prefix}");
                    let ideal = Rat::parse(r.text(&format!("{prefix}_ideal"))?)?;
                    let radius = Rat::parse(r.text(&format!("{prefix}_allowance"))?)?;
                    let error = Rat::binary(*observed).sub(&ideal);
                    assert!(
                        error.abs_le(&radius),
                        "{id} {prefix}: observed {observed:e} outside ideal={} +/- {}",
                        r.text(&format!("{prefix}_ideal"))?,
                        r.text(&format!("{prefix}_allowance"))?
                    );
                    assert_eq!(
                        ideal.binary_round()?.to_bits(),
                        bits(r.text(&format!("{prefix}_ideal_rn_bits"))?)?.to_bits(),
                        "{id} ideal rounded reference {prefix}"
                    );
                    direct += 1
                }
                "uninformative" => {
                    if let Err(e) = value {
                        assert!(financial(e), "{id} uninformative {e:?}");
                    } else {
                        assert!(value.as_ref().is_ok_and(|v| v.is_finite()));
                    }
                    uninformative += 1
                }
                "ideal-range-reference" => {
                    range(id, phase, r.text(&format!("{prefix}_ideal_class"))?, value);
                    ranges += 1
                }
                other => return Err(format!("{id} unknowncomparison{other}").into()),
            }
        }
    }
    assert_eq!(count.get("conditional-absolute"), Some(&5638));
    assert_eq!(count.get("uninformative"), Some(&68));
    assert_eq!(count.get("ideal-range-reference"), Some(&6));
    assert_eq!(count.values().sum::<usize>(), 714 * 8);
    assert_eq!(direct + uninformative + ranges + 8 * gated, 714 * 8);
    assert_eq!(source_displays.len(), 7);
    for family in [
        "SOURCE-A", "SOURCE-B", "SOURCE-C", "SOURCE-D", "SOURCE-E", "SOURCE-F", "SOURCE-G",
    ] {
        assert_eq!(
            source_displays.get(family),
            Some(&2),
            "{family} both policies"
        );
    }
    assert!(gated <= 8);
    Ok(())
}
/// FR-UST-16/17/25: all145 target-driven inverses. Eight coordinates retain
/// their uninformative label. The last row certifies only ideal open-image
/// exclusion; either observed rejection or an admitted observed solve is legal.
#[test]
fn ideal_inverse_roots_qualified_and_cached_reports() -> TestResult {
    let f = fixture(true)?;
    let mut count = BTreeMap::new();
    let mut image = 0;
    for (position, r) in f.records().enumerate() {
        let id = r.id();
        assert_eq!(id, format!("UST-INVERSE-{:05}", position + 1));
        let t = ideal_instrument(&r)?;
        let p = prepared(&r, &t)?;
        let target = BondPrice::new(
            bits(r.text("target_bits")?)?,
            match r.text("target_kind")? {
                "dirty" => BondPriceKind::Dirty,
                "clean" => BondPriceKind::Clean,
                _ => return Err("targetkind".into()),
            },
        )?;
        let bracket = RootBracket::new(
            bits(r.text("caller_lo_bits")?)?,
            bits(r.text("caller_hi_bits")?)?,
        )?;
        let opts = SolverOptions::new(
            bits(r.text("abs_tol_bits")?)?,
            bits(r.text("rel_tol_bits")?)?,
            r.parse("max_evals")?,
        )?;
        let comparison = r.text("comparison")?;
        bump(&mut count, comparison);
        let outcome = p.yield_from_price(target, bracket, opts);
        let report = match outcome {
            Ok(report) => report,
            Err(e) if comparison == "ideal-image-only" => {
                assert!(
                    matches!(
                        e,
                        TreasuryYieldError::Solver(_) | TreasuryYieldError::Callback { .. }
                    ),
                    "{id} image-only error{e:?}"
                );
                image += 1;
                continue;
            }
            Err(e) => return Err(format!("{id} inverse failure {e:?}").into()),
        };
        metadata!(report, p, t, id);
        assert_eq!(report.target(), target);
        assert_eq!(report.initial_bracket(), bracket);
        assert_eq!(report.options(), opts);
        assert_eq!(
            report.dirty_target_per_100().to_bits(),
            bits(r.text("dirty_target_bits")?)?.to_bits()
        );
        assert_eq!(
            report.residual_scale_exponent(),
            r.parse::<u8>("residual_scale")?
        );
        assert_eq!(
            report.sensitivity(),
            BondYieldSensitivity::MathematicallyStrictlyDecreasing
        );
        let rate = report.yield_rate();
        let solve = report.solve_report();
        assert_eq!(rate.to_bits(), solve.point().to_bits());
        assert!(rate.is_finite() && rate > -2.);
        assert!(
            solve.evaluations().function() >= 1
                && solve.evaluations().function() <= opts.max_evaluations()
        );
        assert_eq!(solve.evaluations().derivative(), 0);
        assert!(matches!(
            solve.reason(),
            TerminationReason::FunctionZero | TerminationReason::BracketTolerance
        ));
        let final_bracket = solve.bracket();
        assert!(
            bracket.lower() <= final_bracket.lower() && final_bracket.upper() <= bracket.upper()
        );
        assert!(final_bracket.lower() <= rate && rate <= final_bracket.upper());
        let difference =
            Rat::binary(report.dirty_per_100()).sub(&Rat::binary(report.dirty_target_per_100()));
        assert_eq!(
            report.amount_residual()?.to_bits(),
            difference.binary_round()?.to_bits(),
            "{id} cached amount residual"
        );
        assert_eq!(
            solve.residual().to_bits(),
            difference
                .divide_pow2(u32::from(report.residual_scale_exponent()))
                .binary_round()?
                .to_bits(),
            "{id} cached solver residual"
        );
        assert_eq!(report.dirty_price().kind(), BondPriceKind::Dirty);
        assert_eq!(
            report.dirty_price().value_per_100_face().to_bits(),
            report.dirty_per_100().to_bits()
        );
        let clean = report.clean_price()?;
        assert_eq!(clean.kind(), BondPriceKind::Clean);
        assert_eq!(
            clean.value_per_100_face().to_bits(),
            Rat::binary(report.dirty_per_100())
                .sub(&Rat::binary(report.accrued_per_100()))
                .binary_round()?
                .to_bits()
        );
        match comparison {
            "conditional-bracket-coordinate" => {
                let lo = bits(r.text("conditioned_lo_bits")?)?;
                let hi = bits(r.text("conditioned_hi_bits")?)?;
                let il = bits(r.text("root_lo_bits")?)?;
                let ih = bits(r.text("root_hi_bits")?)?;
                assert!(lo <= il && il <= ih && ih <= hi);
                assert!(
                    lo <= rate && rate <= hi,
                    "{id} root{rate:e} outside[{lo:e},{hi:e}]"
                );
                assert!(matches!(r.text("left_sign")?, "positive" | "zero"));
                assert!(matches!(r.text("right_sign")?, "negative" | "zero"));
            }
            "uninformative-coordinate" => {
                assert!(
                    bits(r.text("conditioned_lo_bits")?)? <= rate
                        && rate <= bits(r.text("conditioned_hi_bits")?)?,
                    "{id} uninformative supplied interval sanity"
                );
            }
            "ideal-image-only" => {
                image += 1;
                assert_eq!(r.text("ideal_image")?, "open-upper:205/1");
                assert_eq!(r.text("root_lo")?, "-");
                assert_eq!(r.text("root_hi")?, "-");
                assert_eq!(r.text("conditioned_lo_bits")?, "-");
            }
            other => return Err(format!("inversecomparison{other}").into()),
        }
    }
    assert_eq!(count.get("conditional-bracket-coordinate"), Some(&136));
    assert_eq!(count.get("uninformative-coordinate"), Some(&8));
    assert_eq!(count.get("ideal-image-only"), Some(&1));
    assert_eq!(image, 1);
    Ok(())
}
