//! FR-YR-02/03/04/10/11/14: independently derived one-payment inverses.
//!
//! Targets, inverse enclosures, raw ideal RN roots and uniform price-error
//! allowances come from closed-form Fraction/Decimal calculations. No planted
//! yield or observed Rust solver root selects an expected result. The bounds
//! include supplied-time uncertainty, measured M4 E3 pricing error, final
//! product rounding and the complete M6 bracket-width tolerance.

mod support;

use sabiquant::bonds::{
    Bond, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement, BondYieldConvention,
    BondYieldSensitivity,
};
use sabiquant::calendar::{Calendar, JointRule, TargetCalendar};
use sabiquant::cashflows::{PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, Thirty360Us, ThirtyE360Isda,
};
use sabiquant::math::{RootBracket, SolverOptions, TerminationReason};
use sabiquant::rates::{AccrualConvention, Compounding};
use sabiquant::time::{Date, Frequency, Weekday};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const COLUMNS: [&str; 26] = [
    "id",
    "family",
    "issue",
    "maturity",
    "settlement",
    "yield_context",
    "face_bits",
    "redemption_bits",
    "comp",
    "target_kind",
    "target_bits",
    "bracket_lo_bits",
    "bracket_hi_bits",
    "abs_tol_bits",
    "rel_tol_bits",
    "max_evals",
    "expected_scale",
    "root_lo_bits",
    "root_hi_bits",
    "ideal_root_bits",
    "dirty_target_bits",
    "price_error_bits",
    "time_lo_bits",
    "time_hi_bits",
    "solved_dirty_lo_bits",
    "solved_dirty_hi_bits",
];

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn pinned_fixture() -> TestResult<Fixture> {
    #[expect(
        clippy::disallowed_methods,
        reason = "test fingerprints independent fixture bytes"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(
        "validation/fixtures/bonds/yield-inverse-extended.fixture",
    ))?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture body separator")?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    assert_eq!(fixture.record_count(), 992);
    assert_eq!(fnv64(text.as_bytes()), 0xeaf48f57f38c3441);
    assert_eq!(fnv64(body.as_bytes()), 0xd153_25d9_5af5_24d8);
    for (key, value) in [
        ("fixture-set", "M8-BOND-YIELD-RISK-EXTENDED"),
        ("domain", "bonds"),
        ("record-count", "992"),
        ("body-fnv64", "d15325d95af524d8"),
        (
            "body-sha256",
            "cd435beba6036dc3ae71d7aac901b698e4105385dc4303cd8c92364349ff6224",
        ),
        (
            "records-sha256",
            "11a82e7e29313ca5611902c52e33985421b5e446929e3358816309157106e88b",
        ),
        (
            "generator-sha256",
            "70723437953a120d18e92bc608a9b2a61112270a3b600b25f01b44a36a4ff258",
        ),
        (
            "helper-sha256",
            "9c5e77b68dbd9c2212b4ae0fb2cd7cdbc9b420505a55d29c01afa0b559541b85",
        ),
        ("helper-fnv64", "d9d9e01d78bbd4b2"),
        ("decimal-precisions", "160,240"),
        ("compounding-count", "32"),
        (
            "inversion-model",
            "exact one-payment closed forms, uniform gross price-error propagation and full-width step tolerance",
        ),
        (
            "requirements-sha256",
            "eaecaec76703c6e7195684cf2f17fe93eec3ed8f00343361676f839379a24e08",
        ),
    ] {
        assert_eq!(fixture.header(key), Some(value), "{key}");
    }
    for (name, bytes, pin, sha) in [
        (
            "m8-bond-yield-risk-contract",
            include_bytes!("../docs/M8_BOND_YIELD_RISK_CONTRACT.md").as_slice(),
            0x2a98_3dab_d670_6130,
            "62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d",
        ),
        (
            "m8-bond-core-contract",
            include_bytes!("../docs/M8_BOND_CORE_CONTRACT.md").as_slice(),
            0xe86f_8537_32e7_dcc0,
            "009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c",
        ),
        (
            "m4-date-contract",
            include_bytes!("../docs/M4_DATE_CONTRACT.md").as_slice(),
            0xe7c7_194f_cb13_d1df,
            "47216e578082eb56daaffb6b49c45951a416f21fe36372baede2bd0a29cdea98",
        ),
        (
            "m3-icma-contract",
            include_bytes!("../docs/M3_ICMA_CONTRACT.md").as_slice(),
            0x8317_e3e3_1fdd_3d7d,
            "87b8da50d887d847919f611f0e3d073893d7065f0378e691ac4832ac979f0bfb",
        ),
        (
            "m3-us-afb-contract",
            include_bytes!("../docs/M3_US_AFB_CONTRACT.md").as_slice(),
            0xd44f_56d8_7d76_8f19,
            "595df058cdd8e64a935b6fc992ef0ca903e7e7fbdfa4ed613bbe598a3afe10b4",
        ),
        (
            "m3-daycount-extensions",
            include_bytes!("../docs/M3_DAYCOUNT_EXTENSIONS.md").as_slice(),
            0x6281_24e5_94b2_1b2c,
            "7ffd2f02d4c430fd8fd095054ce6e67233ee9b27ef260c37a6370790809aee58",
        ),
        (
            "m3-target-contract",
            include_bytes!("../docs/M3_TARGET_CONTRACT.md").as_slice(),
            0x3c3d_89a2_a022_8578,
            "37a4696bbd8a45e3648ba5b8d28a48e30ab480ecf7f833d02875a67960212ede",
        ),
        (
            "m6-solver-research",
            include_bytes!("../docs/M6_SOLVER_RESEARCH.md").as_slice(),
            0xb64e_3465_3d1b_9d41,
            "bdbbf00b6f83bef259c82777bdc62544a616dbc7a13fe4734522d08239040b42",
        ),
        (
            "numerical-correctness",
            include_bytes!("../docs/NUMERICAL_CORRECTNESS.md").as_slice(),
            0x1df1_aeaa_0859_744a,
            "360bc4cfe69d487cc7a94ba694ee791145530053eff7ee014a54db24a420c061",
        ),
    ] {
        assert_eq!(fnv64(bytes), pin, "{name} source");
        assert_eq!(
            fixture.header(&format!("contract-{name}-fnv64")),
            Some(format!("{pin:016x}").as_str())
        );
        assert_eq!(
            fixture.header(&format!("contract-{name}-sha256")),
            Some(sha)
        );
    }
    let source = include_bytes!("../validation/generator/bond_yield_inverse_extended_oracles.py");
    assert_eq!(fnv64(source), 0xe9fa_e2ab_a674_2f8c);
    assert_eq!(fixture.header("generator-fnv64"), Some("e9fae2aba6742f8c"));
    assert_eq!(
        fnv64(include_bytes!(
            "../validation/generator/bond_yield_risk_extended_oracles.py"
        )),
        0xd9d9_e01d_78bb_d4b2
    );
    let requirements = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-YR-"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(fnv64(requirements.as_bytes()), 0xa627_3d93_9267_f859);
    assert_eq!(
        fixture.header("requirements-fnv64"),
        Some("a6273d939267f859")
    );
    Ok(fixture)
}

fn real(text: &str) -> TestResult<f64> {
    if text.len() != 16
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("expected sixteen lowercase binary64 hex digits: {text}").into());
    }
    let value = f64::from_bits(u64::from_str_radix(text, 16)?);
    if !value.is_finite() {
        return Err("fixture real must be finite".into());
    }
    Ok(value)
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(',').map(|value| Ok(value.parse()?)).collect()
}

fn frequency(text: &str) -> TestResult<Frequency> {
    let count: u32 = text.parse()?;
    Frequency::ALL
        .into_iter()
        .find(|value| value.events_per_year() == Some(count))
        .ok_or_else(|| format!("unsupported explicit frequency {text}").into())
}

fn compounding(text: &str) -> TestResult<Compounding> {
    match text {
        "Simple" => Ok(Compounding::SIMPLE),
        "Continuous" => Ok(Compounding::CONTINUOUS),
        _ => {
            let (method, count) = text
                .split_once(':')
                .ok_or("compounding frequency missing")?;
            let frequency = frequency(count)?;
            match method {
                "Compounded" => Ok(Compounding::compounded(frequency)?),
                "SimpleThenCompounded" => Ok(Compounding::simple_then_compounded(frequency)?),
                "CompoundedThenSimple" => Ok(Compounding::compounded_then_simple(frequency)?),
                _ => Err(format!("unknown compounding {text}").into()),
            }
        }
    }
}

fn plain_calendar(mask: &str, holidays: &str, business: &str) -> TestResult<Calendar> {
    let mask: u8 = mask.parse()?;
    if mask >= 128 {
        return Err("weekend mask exceeds seven bits".into());
    }
    let weekends = Weekday::ALL
        .into_iter()
        .enumerate()
        .filter_map(|(index, day)| (mask & (1 << index) != 0).then_some(day));
    let mut calendar = Calendar::custom(weekends, dates(holidays)?);
    for date in dates(business)? {
        calendar = calendar.without_holiday(date);
    }
    Ok(calendar)
}

fn accrual(text: &str) -> TestResult<AccrualConvention> {
    Ok(match text.split(':').collect::<Vec<_>>().as_slice() {
        ["A360"] => DayCount::Actual360.into(),
        ["A365F"] => DayCount::Actual365Fixed.into(),
        ["AAISDA"] => DayCount::ActualActualIsda.into(),
        ["AFB"] => DayCount::ActualActualAfb.into(),
        ["30360"] => DayCount::Thirty360BondBasis.into(),
        ["30E360"] => DayCount::ThirtyE360.into(),
        ["US", "0"] => Thirty360Us::new(false).into(),
        ["US", "1"] => Thirty360Us::new(true).into(),
        ["ISDA", termination] => ThirtyE360Isda::new(termination.parse()?).into(),
        ["ICMA", count, references] => {
            let boundaries = dates(references)?;
            let periods = boundaries
                .windows(2)
                .map(|pair| DeterminationPeriod::new(pair[0], pair[1]))
                .collect::<Result<Vec<_>, _>>()?;
            ActualActualIcma::new(periods, frequency(count)?)?.into()
        }
        ["B252", "C", mask, holidays, business] => {
            Business252::new(plain_calendar(mask, holidays, business)?).into()
        }
        ["B252", "T"] => Business252::with_target(TargetCalendar::new()).into(),
        ["B252", "J", rule, mask, holidays, business] => {
            let rule = match *rule {
                "H" => JointRule::JoinHolidays,
                "B" => JointRule::JoinBusinessDays,
                _ => return Err(format!("unknown joint rule {rule}").into()),
            };
            Business252::with_target(
                TargetCalendar::new()
                    .joint_with(vec![plain_calendar(mask, holidays, business)?], rule)?,
            )
            .into()
        }
        _ => return Err(format!("unknown complete accrual context {text}").into()),
    })
}

fn enclosure(value: f64, lo: f64, hi: f64, label: &str) {
    assert!(lo <= hi, "{label} ordered bounds");
    assert!(
        value.is_finite() && lo <= value && value <= hi,
        "{label}: {value:?} ({:016x}) outside {lo:?}..{hi:?}",
        value.to_bits()
    );
}

/// All992 independently derived roots and cached dirty payloads are checked.
#[test]
fn independent_one_payment_inverse_accuracy() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut families = BTreeMap::<String, usize>::new();
    let mut comps = BTreeMap::<String, usize>::new();
    let mut profiles = BTreeMap::<String, BTreeMap<String, usize>>::new();
    let mut kinds = BTreeMap::<String, usize>::new();
    let mut faces = BTreeMap::<String, usize>::new();
    let mut redemptions = BTreeMap::<String, usize>::new();
    for (index, row) in fixture.records().enumerate() {
        assert_eq!(row.id(), format!("BYIX-{:05}", index + 1));
        let family = row.text("family")?;
        assert!(matches!(family, "CONTEXT-ALL32" | "NEAR-MINUS-M"));
        *families.entry(family.to_owned()).or_default() += 1;
        let context_name = row.text("yield_context")?;
        let context = accrual(context_name)?;
        let comp_name = row.text("comp")?;
        let comp = compounding(comp_name)?;
        *comps.entry(comp_name.to_owned()).or_default() += 1;
        if family == "CONTEXT-ALL32" {
            *profiles
                .entry(context_name.to_owned())
                .or_default()
                .entry(comp_name.to_owned())
                .or_default() += 1;
        }
        let issue: Date = row.text("issue")?.parse()?;
        let maturity: Date = row.text("maturity")?.parse()?;
        let settlement_date: Date = row.text("settlement")?.parse()?;
        let face = real(row.text("face_bits")?)?;
        let amount = real(row.text("redemption_bits")?)?;
        *faces.entry(row.text("face_bits")?.to_owned()).or_default() += 1;
        *redemptions
            .entry(row.text("redemption_bits")?.to_owned())
            .or_default() += 1;
        let bond = Bond::zero_coupon(
            issue,
            maturity,
            face,
            SimpleCashFlow::new(maturity, amount)?,
        )?;
        let settlement = BondSettlement::new(settlement_date, PaymentDateInclusion::Exclude);
        let convention = BondYieldConvention::new(context.clone(), comp);
        let plan = BondPricingPlan::new(&bond, settlement, &convention, u64::MAX)?;
        assert_eq!(plan.accrued_amount().to_bits(), 0);
        assert!(plan.coupon_entitlements().is_empty());
        assert!(plan.redemption_entitled());
        assert_eq!(plan.entitled_flow_count(), 1);
        enclosure(
            context.year_fraction(settlement_date, maturity)?,
            real(row.text("time_lo_bits")?)?,
            real(row.text("time_hi_bits")?)?,
            &format!("{} prepared time", row.id()),
        );
        let kind_name = row.text("target_kind")?;
        *kinds.entry(kind_name.to_owned()).or_default() += 1;
        let kind = match kind_name {
            "Dirty" => BondPriceKind::Dirty,
            "Clean" => BondPriceKind::Clean,
            other => return Err(format!("unknown target kind {other}").into()),
        };
        let target = BondPrice::new(real(row.text("target_bits")?)?, kind)?;
        let dirty_target = real(row.text("dirty_target_bits")?)?;
        assert_eq!(
            bond.amount_from_price_per_100(target)?.to_bits(),
            dirty_target.to_bits()
        );
        let lower = real(row.text("bracket_lo_bits")?)?;
        let upper = real(row.text("bracket_hi_bits")?)?;
        let bracket = RootBracket::new(lower, upper)?;
        let absolute = real(row.text("abs_tol_bits")?)?;
        let relative = real(row.text("rel_tol_bits")?)?;
        let cap: u32 = row.text("max_evals")?.parse()?;
        assert_eq!(cap, 400);
        let options = SolverOptions::new(absolute, relative, cap)?;
        let expected_scale: u8 = row.text("expected_scale")?.parse()?;
        assert_eq!(
            expected_scale, 0,
            "all supplemental targets below fixed scale threshold"
        );
        let root_lo = real(row.text("root_lo_bits")?)?;
        let root_hi = real(row.text("root_hi_bits")?)?;
        assert!(lower <= root_lo && root_hi <= upper);
        enclosure(
            real(row.text("ideal_root_bits")?)?,
            root_lo,
            root_hi,
            &format!("{} raw ideal RN root", row.id()),
        );
        let price_error = real(row.text("price_error_bits")?)?;
        assert!(price_error > 0.0);
        let dirty_lo = real(row.text("solved_dirty_lo_bits")?)?;
        let dirty_hi = real(row.text("solved_dirty_hi_bits")?)?;
        assert!(
            dirty_lo <= dirty_target - price_error && dirty_target + price_error <= dirty_hi,
            "{} uniform pricing error is retained by payload enclosure",
            row.id()
        );
        let work = plan.accrual_work_used();
        let report = plan
            .yield_from_price(target, bracket, options)
            .map_err(|error| format!("{} independent inverse: {error:?}", row.id()))?;
        assert_eq!(
            plan.accrual_work_used(),
            work,
            "{} no new contextual work",
            row.id()
        );
        enclosure(
            report.yield_rate(),
            root_lo,
            root_hi,
            &format!("{} solved yield", row.id()),
        );
        enclosure(
            report.dirty_amount(),
            dirty_lo,
            dirty_hi,
            &format!("{} cached dirty", row.id()),
        );
        assert_eq!(report.target(), target);
        assert_eq!(
            report.converted_target_amount().to_bits(),
            dirty_target.to_bits()
        );
        assert_eq!(
            report.dirty_target_amount().to_bits(),
            dirty_target.to_bits()
        );
        assert_eq!(report.face().to_bits(), face.to_bits());
        assert_eq!(report.accrued_amount().to_bits(), 0);
        assert_eq!(report.settlement(), settlement);
        assert_eq!(report.compounding(), comp);
        assert_eq!(report.initial_bracket(), bracket);
        assert_eq!(report.options(), options);
        assert_eq!(report.residual_scale_exponent(), expected_scale);
        assert_eq!(
            report.sensitivity(),
            BondYieldSensitivity::MathematicallyStrictlyDecreasing
        );
        let solve = report.solve_report();
        assert_eq!(solve.point().to_bits(), report.yield_rate().to_bits());
        assert_eq!(
            solve.residual().to_bits(),
            report.amount_residual()?.to_bits()
        );
        assert_eq!(
            report.amount_residual()?.to_bits(),
            (report.dirty_amount() - dirty_target).to_bits()
        );
        assert!(solve.evaluations().function() > 0 && solve.evaluations().function() <= cap);
        assert_eq!(solve.evaluations().derivative(), 0);
        assert_eq!(
            solve.iterations(),
            solve.evaluations().function().saturating_sub(2)
        );
        assert!(lower <= solve.bracket().lower() && solve.bracket().upper() <= upper);
        assert!(
            solve.bracket().lower() <= solve.point() && solve.point() <= solve.bracket().upper()
        );
        match solve.reason() {
            TerminationReason::FunctionZero => assert_eq!(solve.residual().to_bits(), 0),
            TerminationReason::BracketTolerance => {
                let width = solve.bracket().upper() - solve.bracket().lower();
                let tolerance = absolute
                    + relative
                        * solve
                            .bracket()
                            .lower()
                            .abs()
                            .max(solve.bracket().upper().abs());
                assert!(width <= tolerance);
            }
            other => return Err(format!("unexpected termination {other:?}").into()),
        }
        if family == "NEAR-MINUS-M" {
            assert!(lower < upper && upper < 0.0);
            assert!(context_name.starts_with("ICMA:"));
        }
    }
    assert_eq!(
        families,
        BTreeMap::from([
            ("CONTEXT-ALL32".to_owned(), 960),
            ("NEAR-MINUS-M".to_owned(), 32)
        ])
    );
    assert_eq!(
        fixture.header("census"),
        Some("{\"CONTEXT-ALL32\":960,\"NEAR-MINUS-M\":32}")
    );
    let mut expected_comps = BTreeSet::from(["Simple".to_owned(), "Continuous".to_owned()]);
    for method in ["Compounded", "SimpleThenCompounded", "CompoundedThenSimple"] {
        for frequency in [1, 2, 3, 4, 6, 12, 13, 26, 52, 365] {
            expected_comps.insert(format!("{method}:{frequency}"));
        }
    }
    assert_eq!(
        comps.keys().cloned().collect::<BTreeSet<_>>(),
        expected_comps
    );
    assert!(comps.values().all(|count| *count == 31));
    assert_eq!(profiles.len(), 15);
    for profile in profiles.values() {
        assert_eq!(
            profile.keys().cloned().collect::<BTreeSet<_>>(),
            expected_comps
        );
        assert!(profile.values().all(|count| *count == 2));
    }
    assert_eq!(
        kinds,
        BTreeMap::from([("Clean".to_owned(), 496), ("Dirty".to_owned(), 496)])
    );
    assert_eq!(
        faces,
        BTreeMap::from([
            ("4059000000000000".to_owned(), 512),
            ("4072c00000000000".to_owned(), 480)
        ])
    );
    assert_eq!(
        redemptions,
        BTreeMap::from([
            ("405a400000000000".to_owned(), 512),
            ("408f400000000000".to_owned(), 480)
        ])
    );
    for (key, counts) in [("family-counts", &families), ("compounding-counts", &comps)] {
        let encoded = counts
            .iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(fixture.header(key), Some(encoded.as_str()));
    }
    Ok(())
}
