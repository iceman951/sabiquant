//! Test-only Canadian input binding and exact conditional interval comparisons.
//!
//! Generic unbounded arithmetic/SHA routines are reused from the earlier
//! independent test helper; no Treasury financial convention is reused.
#![allow(dead_code)]

use crate::arithmetic::{Rat, bits, bump, count_json, fnv64, sha256};
use crate::support::fixture::{Fixture, Record};
use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    CanadianBond, CanadianCouponSchedule, CanadianFrequency, CanadianPricingPlan,
    CanadianSourcePolicy, CanadianYieldConvention, CanadianYieldSelection,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::time::Date;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

pub(crate) type Checked<T = ()> = Result<T, Box<dyn Error>>;
pub(crate) const SOURCE: &str = "CA-IIAC13-fixed-2026-10-23-local-v1";
pub(crate) const PRICE: &str = "validation/fixtures/bonds/canadian-ideal-price-risk-oracle.fixture";
pub(crate) const INVERSE: &str = "validation/fixtures/bonds/canadian-ideal-inverse-oracle.fixture";
pub(crate) const PRICE_INPUTS: [&str; 17] = [
    "accrual_start",
    "issue",
    "stated_maturity",
    "true_maturity",
    "coupon_coefficient",
    "coupon_scale",
    "frequency",
    "eom",
    "quasi_dates",
    "coupon_dates",
    "settlement",
    "source_policy",
    "selection",
    "date_cap",
    "bundle_cap",
    "operation",
    "yield_bits",
];
pub(crate) const INVERSE_INPUTS: [&str; 23] = [
    "accrual_start",
    "issue",
    "stated_maturity",
    "true_maturity",
    "coupon_coefficient",
    "coupon_scale",
    "frequency",
    "eom",
    "quasi_dates",
    "coupon_dates",
    "settlement",
    "source_policy",
    "selection",
    "date_cap",
    "bundle_cap",
    "operation",
    "target_basis",
    "target_bits",
    "bracket_lower_bits",
    "bracket_upper_bits",
    "absolute_tolerance_bits",
    "relative_tolerance_bits",
    "evaluation_budget",
];

// Literal root-approved projections; filled from the already frozen approval
// registry during authoring, never selected from the fixture being checked.
const PRICE_COLUMNS: &str = "id\tprofile\taccrual_start\tissue\tstated_maturity\ttrue_maturity\tcoupon_coefficient\tcoupon_scale\tfrequency\teom\tquasi_dates\tcoupon_dates\tsettlement\tsource_policy\tselection\tdate_cap\tbundle_cap\toperation\tyield_bits\tconvention\tgeometry\toriginal_ordinals\tlambdas\tsource_a\tsource_ai\tsource_coefficients\tideal_image_floor\tobservation_class\tcertificate_ids\tmodel_dirty_lo\tmodel_dirty_hi\tmodel_dirty_error\tmodel_dirty_class\tmodel_dirty_informative\tclean_lo\tclean_hi\tclean_error\tclean_class\tclean_informative\tsettlement_dirty_lo\tsettlement_dirty_hi\tsettlement_dirty_error\tsettlement_dirty_class\tsettlement_dirty_informative\tprice_derivative_lo\tprice_derivative_hi\tprice_derivative_error\tprice_derivative_class\tprice_derivative_informative\tprice_second_derivative_lo\tprice_second_derivative_hi\tprice_second_derivative_error\tprice_second_derivative_class\tprice_second_derivative_informative\ttime_weighted_amount_lo\ttime_weighted_amount_hi\ttime_weighted_amount_error\ttime_weighted_amount_class\ttime_weighted_amount_informative\tmodified_duration_lo\tmodified_duration_hi\tmodified_duration_error\tmodified_duration_class\tmodified_duration_informative\tmacaulay_duration_lo\tmacaulay_duration_hi\tmacaulay_duration_error\tmacaulay_duration_class\tmacaulay_duration_informative\tconvexity_lo\tconvexity_hi\tconvexity_error\tconvexity_class\tconvexity_informative";
const INVERSE_COLUMNS: &str = "id\tprofile\taccrual_start\tissue\tstated_maturity\ttrue_maturity\tcoupon_coefficient\tcoupon_scale\tfrequency\teom\tquasi_dates\tcoupon_dates\tsettlement\tsource_policy\tselection\tdate_cap\tbundle_cap\toperation\ttarget_basis\ttarget_bits\tbracket_lower_bits\tbracket_upper_bits\tabsolute_tolerance_bits\trelative_tolerance_bits\tevaluation_budget\tconvention\tgeometry\treference_equation\texact_model_target\treference_model_target\tcached_model_target_bits\ttarget_shift_error\ttarget_shift_included\tideal_image_floor\tideal_root_class\troot_lo\troot_hi\tlower_price_lo\tlower_price_hi\tupper_price_lo\tupper_price_hi\tslope_lower\tprice_error\tresidual_to_coordinate\tresidual_budget\tcoordinate_error\tinformative\tobserved_protocol\tfunction_calls\tderivative_calls\tcertificate_ids";

#[expect(
    clippy::disallowed_methods,
    reason = "Reads frozen local validation evidence only."
)]
pub(crate) fn fixture(inverse: bool) -> Checked<Fixture> {
    let (path, expected, count, columns, set, inputs) = if inverse {
        (
            INVERSE,
            "5be49b2960330d61b0bd27a6a4b0448affdb5fa21e9c21ca37a88a6eacb7245e",
            349,
            INVERSE_COLUMNS,
            "CAD-IDEAL-INVERSE",
            INVERSE_INPUTS.as_slice(),
        )
    } else {
        (
            PRICE,
            "9318a324ff100a0fe3afd806264767d74daaa37d3f7511640dc7d689f3d66383",
            881,
            PRICE_COLUMNS,
            "CAD-IDEAL-PRICE-RISK",
            PRICE_INPUTS.as_slice(),
        )
    };
    let raw = std::fs::read(Fixture::repository_path(path))?;
    assert!(raw.ends_with(b"\n") && !raw.contains(&b'\r'));
    assert_eq!(sha256(&raw)?, expected, "frozen fixture {path}");
    let text = std::str::from_utf8(&raw)?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture body")?;
    let (actual_columns, records) = body.split_once('\n').ok_or("record payload")?;
    assert_eq!(actual_columns, columns, "literal approved columns");
    let fixture = Fixture::parse(text)?;
    assert_eq!(fixture.header("fixture-set"), Some(set));
    assert_eq!(fixture.record_count(), count);
    assert_eq!(fixture.columns().len(), if inverse { 51 } else { 74 });
    assert_eq!(
        fixture.header("body-sha256"),
        Some(sha256(body.as_bytes())?.as_str())
    );
    assert_eq!(
        fixture.header("body-fnv64"),
        Some(format!("{:016x}", fnv64(body.as_bytes())).as_str())
    );
    assert_eq!(
        fixture.header("records-sha256"),
        Some(sha256(records.as_bytes())?.as_str())
    );
    assert_eq!(
        fixture.header("records-fnv64"),
        Some(format!("{:016x}", fnv64(records.as_bytes())).as_str())
    );
    assert_eq!(
        fixture.header("input-columns"),
        Some(inputs.join(",").as_str())
    );
    assert_eq!(
        fixture.header("requirements-rows-sha256"),
        Some("a31bb24356af3b8ed2f6454cd6512bdc052f19705e68211e5a300eafd96e37ae")
    );
    let generator = include_bytes!("../../validation/generator/canadian_ideal_oracles.py");
    assert_eq!(
        fixture.header("generator-sha256"),
        Some(sha256(generator)?.as_str())
    );
    assert_eq!(
        fixture.header("generator-fnv64"),
        Some(format!("{:016x}", fnv64(generator)).as_str())
    );
    let certificate =
        include_bytes!("../../validation/fixtures/bonds/canadian-ideal-certificates.json");
    assert_eq!(
        sha256(certificate)?,
        "c33448a73576f5969ebfaba3a7926e5f90cd1a0c3c660990cd195a0c445129cb"
    );
    assert_eq!(
        fixture.header("certificates-sha256"),
        Some(sha256(certificate)?.as_str())
    );
    assert_eq!(
        fixture.header("certificates-fnv64"),
        Some(format!("{:016x}", fnv64(certificate)).as_str())
    );
    let contract = include_bytes!("../../docs/M8_CANADIAN_CONTRACT.md");
    assert_eq!(
        fixture.header("adopted-contract-sha256"),
        Some(sha256(contract)?.as_str())
    );
    assert_eq!(
        fixture.header("adopted-contract-fnv64"),
        Some(format!("{:016x}", fnv64(contract)).as_str())
    );
    let mut canonical_inputs = String::new();
    let mut unique = BTreeSet::new();
    let mut profiles = BTreeMap::new();
    for record in fixture.records() {
        let key = inputs
            .iter()
            .map(|name| {
                let text = record.text(name)?;
                if name.ends_with("_bits") && bits(text)?.to_bits() == 1_u64 << 63 {
                    Ok("0000000000000000".to_owned())
                } else {
                    Ok(text.to_owned())
                }
            })
            .collect::<Checked<Vec<_>>>()?
            .join("\t");
        assert!(
            unique.insert(key.clone()),
            "duplicate canonical public input {}",
            record.id()
        );
        canonical_inputs.push_str(&key);
        canonical_inputs.push('\n');
        bump(&mut profiles, record.text("profile")?);
    }
    assert_eq!(unique.len(), count);
    assert_eq!(
        fixture.header("inputs-sha256"),
        Some(sha256(canonical_inputs.as_bytes())?.as_str())
    );
    assert_eq!(
        fixture.header("inputs-fnv64"),
        Some(format!("{:016x}", fnv64(canonical_inputs.as_bytes())).as_str())
    );
    assert_eq!(
        fixture.header("profile-counts"),
        Some(count_json(&profiles).as_str())
    );
    Ok(fixture)
}

pub(crate) fn date(value: &str) -> Checked<Date> {
    Ok(value.parse()?)
}
pub(crate) fn dates(value: &str) -> Checked<Vec<Date>> {
    value.split(';').map(date).collect()
}
pub(crate) fn selection(value: &str) -> Checked<CanadianYieldSelection> {
    Ok(match value {
        "Explicit(Compound)" => CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        "Explicit(MoneyMarket)" => {
            CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket)
        }
        "LastCouponMoneyMarket" => CanadianYieldSelection::LastCouponMoneyMarket,
        _ => return Err("unknown selection".into()),
    })
}
pub(crate) fn instrument(record: &Record<'_>) -> Checked<CanadianBond> {
    assert_eq!(record.text("source_policy")?, "CanonicalIiac13LocalV1");
    let frequency = match record.text("frequency")? {
        "2" => CanadianFrequency::Semiannual,
        "4" => CanadianFrequency::Quarterly,
        "12" => CanadianFrequency::Monthly,
        _ => return Err("unknown frequency".into()),
    };
    let end_of_month = match record.text("eom")? {
        "Ignore" => EndOfMonth::Ignore,
        "Preserve" => EndOfMonth::Preserve,
        _ => return Err("unknown EOM".into()),
    };
    let schedule = CanadianCouponSchedule::new(
        date(record.text("accrual_start")?)?,
        date(record.text("stated_maturity")?)?,
        frequency,
        dates(record.text("quasi_dates")?)?,
        dates(record.text("coupon_dates")?)?,
        end_of_month,
        record.parse("date_cap")?,
    )?;
    Ok(CanadianBond::new(
        date(record.text("issue")?)?,
        DecimalAmount::new(
            record.parse("coupon_coefficient")?,
            record.parse("coupon_scale")?,
        )?,
        schedule,
        date(record.text("true_maturity")?)?,
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )?)
}
pub(crate) fn plan<'a>(
    record: &Record<'_>,
    bond: &'a CanadianBond,
) -> Checked<CanadianPricingPlan<'a>> {
    let plan = CanadianPricingPlan::new(
        bond,
        date(record.text("settlement")?)?,
        selection(record.text("selection")?)?,
        record.parse("bundle_cap")?,
    )?;
    assert!(std::ptr::eq(plan.bond(), bond));
    assert_eq!(plan.source_version(), SOURCE);
    assert_eq!(
        format!("{:?}", plan.yield_convention()),
        record.text("convention")?
    );
    Ok(plan)
}

pub(crate) fn zero() -> Checked<Rat> {
    Rat::parse("0/1")
}
pub(crate) fn add(left: &Rat, right: &Rat) -> Checked<Rat> {
    Ok(left.sub(&zero()?.sub(right)))
}
pub(crate) fn dyadic(text: &str) -> Checked<Rat> {
    let (mantissa, exponent) = text.split_once('@').ok_or("exact dyadic spelling")?;
    let exponent: i32 = exponent.parse()?;
    let mut value = Rat::parse(&format!("{mantissa}/1"))?;
    if exponent < 0 {
        value = value.divide_pow2(exponent.unsigned_abs());
    } else {
        for _ in 0..u32::try_from(exponent)? {
            value = add(&value, &value)?;
        }
    }
    Ok(value)
}
/// Exact comparison with [lo-error,hi+error], without rounding any endpoint.
pub(crate) fn within_interval(value: f64, lo: &str, hi: &str, error: &str) -> Checked<bool> {
    assert!(value.is_finite());
    let lo = dyadic(lo)?;
    let hi = dyadic(hi)?;
    let center = add(&lo, &hi)?.divide_pow2(1);
    let radius = add(&hi.sub(&lo).divide_pow2(1), &dyadic(error)?)?;
    Ok(Rat::binary(value).sub(&center).abs_le(&radius))
}
