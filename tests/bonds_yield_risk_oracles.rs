//! FR-YR-01..16: independent M8 Phase D bond yield-to-maturity and analytical risk oracle.
//!
//! Consumes `validation/fixtures/bonds/yield-risk-oracles.fixture` generated
//! independently by `validation/generator/bond_yield_risk_oracles.py` using
//! exact `Fraction` and dual-precision `Decimal` (160 and 240 digits).

mod support;

use sabiquant::bonds::{
    Bond, BondError, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement,
    BondYieldConvention, BondYieldError, BondYieldSensitivity,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{ActualActualIcma, DayCount, DeterminationPeriod, ThirtyE360Isda};
use sabiquant::math::{EvaluationStage, RootBracket, SolverOptions, TerminationReason};
use sabiquant::rates::{AccrualConvention, Compounding};
use sabiquant::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::collections::BTreeMap;
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const COLUMNS: [&str; 72] = [
    "id",
    "family",
    "profile",
    "issue",
    "settlement",
    "inclusion",
    "daycount",
    "comp",
    "face_bits",
    "accrued_bits",
    "flows",
    "target_kind",
    "target_mode",
    "target_value_bits",
    "bracket_lo_bits",
    "bracket_hi_bits",
    "abs_step_tol_bits",
    "rel_step_tol_bits",
    "max_evals",
    "risk_yield_bits",
    "yield_expected",
    "yield_error_kind",
    "yield_error_phase",
    "yield_error_stage",
    "yield_error_evals",
    "sensitivity",
    "residual_scale_exp",
    "eval_count",
    "termination",
    "converted_target_bits",
    "dirty_target_bits",
    "solved_yield_lo_bits",
    "solved_yield_hi_bits",
    "solved_dirty_lo_bits",
    "solved_dirty_hi_bits",
    "solved_clean_lo_bits",
    "solved_clean_hi_bits",
    "solve_residual_lo_bits",
    "solve_residual_hi_bits",
    "amount_residual_lo_bits",
    "amount_residual_hi_bits",
    "ideal_root_bits",
    "risk_expected",
    "risk_error_kind",
    "risk_error_phase",
    "risk_error_index",
    "risk_dirty_lo_bits",
    "risk_dirty_hi_bits",
    "p1_status",
    "p1_lo_bits",
    "p1_hi_bits",
    "p2_status",
    "p2_lo_bits",
    "p2_hi_bits",
    "moment_status",
    "moment_lo_bits",
    "moment_hi_bits",
    "mod_dur_status",
    "mod_dur_lo_bits",
    "mod_dur_hi_bits",
    "mac_dur_status",
    "mac_dur_lo_bits",
    "mac_dur_hi_bits",
    "conv_status",
    "conv_lo_bits",
    "conv_hi_bits",
    "ideal_p1_bits",
    "ideal_p2_bits",
    "ideal_moment_bits",
    "ideal_mod_dur_bits",
    "ideal_mac_dur_bits",
    "ideal_conv_bits",
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
        "validation/fixtures/bonds/yield-risk-oracles.fixture",
    ))?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture body separator")?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    assert_eq!(
        fixture.header("fixture-set"),
        Some("M8-BOND-YIELD-RISK-ORACLE")
    );
    assert_eq!(
        fixture.header("body-fnv64"),
        Some(format!("{:016x}", fnv64(body.as_bytes())).as_str())
    );
    assert_eq!(fnv64(body.as_bytes()), 0x8d53_0602_803e_cdd6);
    for (key, value) in [
        (
            "body-sha256",
            "f39603b65d479a2b94094d97df167a9cbae2ab23f75875d0c18b1ffa8e3434bd",
        ),
        (
            "generator-sha256",
            "16949a31bf9845bfc72b2dbf2a0c2b2c159efeaaa367c4ec79b2f0a46ad9b087",
        ),
        (
            "requirements-rows-sha256",
            "eaecaec76703c6e7195684cf2f17fe93eec3ed8f00343361676f839379a24e08",
        ),
        (
            "m8-bond-yield-risk-contract-sha256",
            "62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d",
        ),
        (
            "m8-bond-core-contract-sha256",
            "009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c",
        ),
        (
            "m3-icma-contract-sha256",
            "87b8da50d887d847919f611f0e3d073893d7065f0378e691ac4832ac979f0bfb",
        ),
        (
            "m3-daycount-extensions-sha256",
            "7ffd2f02d4c430fd8fd095054ce6e67233ee9b27ef260c37a6370790809aee58",
        ),
        (
            "m4-date-contract-sha256",
            "47216e578082eb56daaffb6b49c45951a416f21fe36372baede2bd0a29cdea98",
        ),
        (
            "m7-cashflow-contract-sha256",
            "04db3050737177b9928011102937b7c979a9c0e25171e242551eec3fd08d80a2",
        ),
    ] {
        assert_eq!(fixture.header(key), Some(value), "{key}");
    }
    for (key, source, frozen) in [
        (
            "generator-fnv64",
            include_bytes!("../validation/generator/bond_yield_risk_oracles.py").as_slice(),
            "065367972403362a",
        ),
        (
            "m8-bond-yield-risk-contract-fnv64",
            include_bytes!("../docs/M8_BOND_YIELD_RISK_CONTRACT.md").as_slice(),
            "2a983dabd6706130",
        ),
        (
            "m8-bond-core-contract-fnv64",
            include_bytes!("../docs/M8_BOND_CORE_CONTRACT.md").as_slice(),
            "e86f853732e7dcc0",
        ),
        (
            "m3-icma-contract-fnv64",
            include_bytes!("../docs/M3_ICMA_CONTRACT.md").as_slice(),
            "8317e3e31fdd3d7d",
        ),
        (
            "m3-daycount-extensions-fnv64",
            include_bytes!("../docs/M3_DAYCOUNT_EXTENSIONS.md").as_slice(),
            "628124e594b21b2c",
        ),
        (
            "m4-date-contract-fnv64",
            include_bytes!("../docs/M4_DATE_CONTRACT.md").as_slice(),
            "e7c7194fcb13d1df",
        ),
        (
            "m7-cashflow-contract-fnv64",
            include_bytes!("../docs/M7_CASHFLOW_CONTRACT.md").as_slice(),
            "8f3f1cf6fc5fb390",
        ),
    ] {
        let actual = format!("{:016x}", fnv64(source));
        assert_eq!(actual, frozen, "{key}");
        assert_eq!(fixture.header(key), Some(frozen), "{key}");
    }
    let req_rows: String = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-YR-"))
        .map(|line| format!("{line}\n"))
        .collect();
    let req_fnv = format!("{:016x}", fnv64(req_rows.as_bytes()));
    assert_eq!(req_fnv, "a6273d939267f859");
    assert_eq!(
        fixture.header("requirements-rows-fnv64"),
        Some("a6273d939267f859")
    );
    Ok(fixture)
}

fn parse_bits(hex: &str) -> Result<u64, Box<dyn Error>> {
    let stripped = hex.strip_prefix("0x").ok_or("missing 0x prefix")?;
    Ok(u64::from_str_radix(stripped, 16)?)
}

fn parse_f64(hex: &str) -> Result<f64, Box<dyn Error>> {
    Ok(f64::from_bits(parse_bits(hex)?))
}

fn parse_date(iso: &str) -> Result<Date, Box<dyn Error>> {
    let mut parts = iso.split('-');
    let y: i32 = parts.next().ok_or("year")?.parse()?;
    let m: u32 = parts.next().ok_or("month")?.parse()?;
    let d: u32 = parts.next().ok_or("day")?.parse()?;
    Ok(Date::from_ymd(y, m, d)?)
}

fn parse_frequency(code: &str) -> Result<Frequency, Box<dyn Error>> {
    match code.parse::<u16>()? {
        1 => Ok(Frequency::Annual),
        2 => Ok(Frequency::Semiannual),
        3 => Ok(Frequency::EveryFourthMonth),
        4 => Ok(Frequency::Quarterly),
        6 => Ok(Frequency::Bimonthly),
        12 => Ok(Frequency::Monthly),
        13 => Ok(Frequency::EveryFourthWeek),
        26 => Ok(Frequency::Biweekly),
        52 => Ok(Frequency::Weekly),
        365 => Ok(Frequency::Daily),
        other => Err(format!("unsupported frequency {other}").into()),
    }
}

fn parse_compounding(spec: &str) -> Result<Compounding, Box<dyn Error>> {
    if spec == "Simple" {
        return Ok(Compounding::SIMPLE);
    }
    if spec == "Continuous" {
        return Ok(Compounding::CONTINUOUS);
    }
    let (method, f_str) = spec.split_once(':').ok_or("invalid comp spec")?;
    let freq = parse_frequency(f_str)?;
    match method {
        "Compounded" => Ok(Compounding::compounded(freq)?),
        "SimpleThenCompounded" => Ok(Compounding::simple_then_compounded(freq)?),
        "CompoundedThenSimple" => Ok(Compounding::compounded_then_simple(freq)?),
        other => Err(format!("unknown compounding {other}").into()),
    }
}

fn parse_accrual(spec: &str) -> Result<AccrualConvention, Box<dyn Error>> {
    if spec == "30360" {
        return Ok(AccrualConvention::from(DayCount::Thirty360BondBasis));
    }
    if spec == "A365F" {
        return Ok(AccrualConvention::from(DayCount::Actual365Fixed));
    }
    if let Some(term_iso) = spec.strip_prefix("30E360ISDA:") {
        let term = parse_date(term_iso)?;
        return Ok(AccrualConvention::from(ThirtyE360Isda::new(term)));
    }
    if let Some(rest) = spec.strip_prefix("ICMA:") {
        let (f_str, bounds_str) = rest.split_once(':').ok_or("invalid ICMA spec")?;
        let freq = parse_frequency(f_str)?;
        let dates: Vec<Date> = bounds_str
            .split(',')
            .map(parse_date)
            .collect::<Result<_, _>>()?;
        let periods: Vec<DeterminationPeriod> = dates
            .windows(2)
            .map(|w| DeterminationPeriod::new(w[0], w[1]))
            .collect::<Result<_, _>>()?;
        return Ok(AccrualConvention::from(ActualActualIcma::new(
            periods, freq,
        )?));
    }
    Err(format!("unsupported daycount spec {spec}").into())
}

fn one_period_schedule(
    start: Date,
    end: Date,
) -> Result<sabiquant::schedule::Schedule, Box<dyn Error>> {
    let builder = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )?;
    Ok(builder.build_recurrent(
        Period::new(100, TimeUnit::Years),
        DateGenerationRule::Forward,
        StubPolicy::Short,
        EndOfMonth::Ignore,
        4,
    )?)
}

fn semiannual_schedule(
    start: Date,
    end: Date,
) -> Result<sabiquant::schedule::Schedule, Box<dyn Error>> {
    let builder = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )?;
    Ok(builder.build_recurrent(
        Period::new(6, TimeUnit::Months),
        DateGenerationRule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        16,
    )?)
}

fn build_bond_for_record(
    record: &Record,
) -> Result<(Bond, BondSettlement, BondYieldConvention), Box<dyn Error>> {
    let family = record.text("family")?;
    let profile = record.text("profile")?;
    let issue = parse_date(record.text("issue")?)?;
    let settlement_date = parse_date(record.text("settlement")?)?;
    let inclusion = match record.text("inclusion")? {
        "Include" => PaymentDateInclusion::Include,
        "Exclude" => PaymentDateInclusion::Exclude,
        other => return Err(format!("unknown inclusion {other}").into()),
    };
    let accrual = parse_accrual(record.text("daycount")?)?;
    let comp = parse_compounding(record.text("comp")?)?;
    let face = parse_f64(record.text("face_bits")?)?;
    let settlement = BondSettlement::new(settlement_date, inclusion);
    let convention = BondYieldConvention::new(accrual.clone(), comp);

    let flows_str = record.text("flows")?;
    let mut parsed_flows = Vec::new();
    if flows_str != "-" {
        for item in flows_str.split(';') {
            let mut parts = item.split(':');
            let dt = parse_date(parts.next().ok_or("flow date")?)?;
            let amt = parse_f64(parts.next().ok_or("flow amt")?)?;
            let _t = parse_f64(parts.next().ok_or("flow t")?)?;
            parsed_flows.push((dt, amt));
        }
    }

    let bond = match family {
        "SINGLE-FLOW-32" | "EXTREME-SCALE" => {
            let (mat, amt) = parsed_flows[0];
            let redemption = SimpleCashFlow::new(mat, amt)?;
            Bond::zero_coupon(issue, mat, face, redemption)?
        }
        "COUPON-32" => {
            let rate = if profile.ends_with("MID-CLEAN") {
                0.05
            } else {
                0.035
            };
            let mat = parse_date("2026-01-01")?;
            let sched = semiannual_schedule(issue, mat)?;
            let coupon_rate = FixedCouponRate::new(rate, accrual)?;
            let redemption = SimpleCashFlow::new(mat, face)?;
            Bond::fixed_rate(
                issue,
                sched,
                face,
                coupon_rate,
                vec![None; 4],
                redemption,
                16,
            )?
        }
        "SIGNED-NONMONOTONE" => {
            let (d1, c1) = parsed_flows[0];
            let (d2, r2) = parsed_flows[1];
            let yf = accrual.year_fraction(issue, d1)?;
            let rate = c1 / (face * yf);
            let coupon_rate = FixedCouponRate::new(rate, accrual)?;
            let sched = one_period_schedule(issue, d1)?;
            let redemption = SimpleCashFlow::new(d2, r2)?;
            Bond::fixed_rate(issue, sched, face, coupon_rate, vec![None], redemption, 16)?
        }
        "CONSTANT-PLAN" => {
            if parsed_flows.is_empty() {
                let mat = parse_date("2025-01-01")?;
                let redemption = SimpleCashFlow::new(mat, 100.0)?;
                Bond::zero_coupon(issue, mat, face, redemption)?
            } else if parsed_flows.len() == 1 {
                let (mat, amt) = parsed_flows[0];
                let redemption = SimpleCashFlow::new(mat, amt)?;
                Bond::zero_coupon(issue, mat, face, redemption)?
            } else {
                let (d1, c1) = parsed_flows[0];
                let (d2, r2) = parsed_flows[1];
                let yf = accrual.year_fraction(issue, d1)?;
                let rate = c1 / (face * yf);
                let coupon_rate = FixedCouponRate::new(rate, accrual)?;
                let sched = one_period_schedule(issue, d1)?;
                let redemption = SimpleCashFlow::new(d2, r2)?;
                Bond::fixed_rate(issue, sched, face, coupon_rate, vec![None], redemption, 16)?
            }
        }
        "ERROR-TAXONOMY" => {
            if parsed_flows.len() == 1 {
                let (mat, amt) = parsed_flows[0];
                let redemption = SimpleCashFlow::new(mat, amt)?;
                Bond::zero_coupon(issue, mat, face, redemption)?
            } else {
                let (d1, c1) = parsed_flows[0];
                let (d2, r2) = parsed_flows[1];
                let yf = accrual.year_fraction(issue, d1)?;
                let rate = c1 / (face * yf);
                let coupon_rate = FixedCouponRate::new(rate, accrual)?;
                let sched = one_period_schedule(issue, d1)?;
                let redemption = SimpleCashFlow::new(d2, r2)?;
                Bond::fixed_rate(issue, sched, face, coupon_rate, vec![None], redemption, 16)?
            }
        }
        other => return Err(format!("unknown family {other}").into()),
    };

    Ok((bond, settlement, convention))
}

fn assert_in_enclosure(
    id: &str,
    label: &str,
    actual: f64,
    lo_hex: &str,
    hi_hex: &str,
) -> TestResult {
    let lo = parse_f64(lo_hex)?;
    let hi = parse_f64(hi_hex)?;
    let min_v = lo.min(hi);
    let max_v = lo.max(hi);
    assert!(
        actual >= min_v && actual <= max_v,
        "{id} {label}: actual={actual:?} (0x{:016x}) not in [{min_v:?} ({lo_hex}), {max_v:?} ({hi_hex})]",
        actual.to_bits()
    );
    Ok(())
}

#[test]
fn bond_yield_risk_oracle_corpus_matches_independent_fixture() -> TestResult {
    let fixture = pinned_fixture()?;
    assert_eq!(fixture.records().len(), 141);

    let mut family_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut yield_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut risk_counts: BTreeMap<String, usize> = BTreeMap::new();

    for record in fixture.records() {
        let id = record.text("id")?;
        let family = record.text("family")?;
        *family_counts.entry(family.to_owned()).or_default() += 1;
        *yield_counts
            .entry(record.text("yield_expected")?.to_owned())
            .or_default() += 1;
        *risk_counts
            .entry(record.text("risk_expected")?.to_owned())
            .or_default() += 1;

        let (bond, settlement, convention) = build_bond_for_record(&record)?;
        let plan = BondPricingPlan::new(&bond, settlement, &convention, 100_000)?;
        let initial_work = plan.accrual_work_used();
        assert_eq!(
            plan.accrued_amount().to_bits(),
            parse_bits(record.text("accrued_bits")?)?,
            "{id} accrued_bits"
        );

        let kind = match record.text("target_kind")? {
            "Clean" => BondPriceKind::Clean,
            "Dirty" => BondPriceKind::Dirty,
            other => return Err(format!("unknown target_kind {other}").into()),
        };
        let target_val = parse_f64(record.text("target_value_bits")?)?;
        let target = match record.text("target_mode")? {
            "Per100Face" => BondPrice::new(target_val, kind)?,
            "Amount" => BondPrice::from_amount(target_val, bond.face(), kind)?,
            other => return Err(format!("unknown target_mode {other}").into()),
        };
        let bracket = RootBracket::new(
            parse_f64(record.text("bracket_lo_bits")?)?,
            parse_f64(record.text("bracket_hi_bits")?)?,
        )?;
        let options = SolverOptions::new(
            parse_f64(record.text("abs_step_tol_bits")?)?,
            parse_f64(record.text("rel_step_tol_bits")?)?,
            record.text("max_evals")?.parse()?,
        )?;

        let yield_res = plan.yield_from_price(target, bracket, options);
        assert_eq!(
            plan.accrual_work_used(),
            initial_work,
            "{id} zero work on yield"
        );

        match record.text("yield_expected")? {
            "ok" => {
                let rep = yield_res.map_err(|e| format!("{id} unexpected yield err: {e}"))?;
                let expected_sens = match record.text("sensitivity")? {
                    "MathematicallyStrictlyDecreasing" => {
                        BondYieldSensitivity::MathematicallyStrictlyDecreasing
                    }
                    "Uncertified" => BondYieldSensitivity::Uncertified,
                    other => return Err(format!("unknown sensitivity {other}").into()),
                };
                assert_eq!(rep.sensitivity(), expected_sens, "{id} sensitivity");
                assert_eq!(
                    rep.residual_scale_exponent(),
                    record.text("residual_scale_exp")?.parse::<u8>()?,
                    "{id} residual_scale_exp"
                );
                let expected_evals = record.text("eval_count")?.parse::<u32>()?;
                let actual_evals = rep.solve_report().evaluations().function();
                let expected_term = match record.text("termination")? {
                    "ExactZero" => TerminationReason::FunctionZero,
                    "StepTolerance" => TerminationReason::BracketTolerance,
                    other => return Err(format!("unknown termination {other}").into()),
                };
                if record.text("solved_yield_lo_bits")? == record.text("solved_yield_hi_bits")? {
                    assert_eq!(actual_evals, expected_evals, "{id} eval_count");
                    assert_eq!(
                        rep.solve_report().reason(),
                        expected_term,
                        "{id} termination"
                    );
                } else {
                    assert!(
                        actual_evals.abs_diff(expected_evals) <= 2,
                        "{id} eval_count: actual={actual_evals} expected={expected_evals}"
                    );
                }
                assert_eq!(rep.solve_report().evaluations().derivative(), 0);
                assert_eq!(
                    rep.converted_target_amount().to_bits(),
                    parse_bits(record.text("converted_target_bits")?)?,
                    "{id} converted_target_bits"
                );
                assert_eq!(
                    rep.dirty_target_amount().to_bits(),
                    parse_bits(record.text("dirty_target_bits")?)?,
                    "{id} dirty_target_bits"
                );
                assert_in_enclosure(
                    id,
                    "solved_yield",
                    rep.yield_rate(),
                    record.text("solved_yield_lo_bits")?,
                    record.text("solved_yield_hi_bits")?,
                )?;
                assert_in_enclosure(
                    id,
                    "solved_dirty",
                    rep.dirty_amount(),
                    record.text("solved_dirty_lo_bits")?,
                    record.text("solved_dirty_hi_bits")?,
                )?;
                assert_in_enclosure(
                    id,
                    "solved_clean",
                    rep.clean_amount()?,
                    record.text("solved_clean_lo_bits")?,
                    record.text("solved_clean_hi_bits")?,
                )?;
                assert_in_enclosure(
                    id,
                    "solve_residual",
                    rep.solve_report().residual(),
                    record.text("solve_residual_lo_bits")?,
                    record.text("solve_residual_hi_bits")?,
                )?;
                assert_in_enclosure(
                    id,
                    "amount_residual",
                    rep.amount_residual()?,
                    record.text("amount_residual_lo_bits")?,
                    record.text("amount_residual_hi_bits")?,
                )?;
                let ideal_root = parse_f64(record.text("ideal_root_bits")?)?;
                assert!(
                    (rep.yield_rate() - ideal_root).abs() <= 1e-6 * (1.0 + ideal_root.abs()),
                    "{id} ideal_root: actual={} ideal={ideal_root}",
                    rep.yield_rate()
                );
            }
            "err" => {
                let Err(err) = yield_res else {
                    return Err(format!("{id} expected yield err").into());
                };
                assert_eq!(err.kind(), record.text("yield_error_kind")?, "{id} kind");
                let phase = record.text("yield_error_phase")?;
                if phase != "-" {
                    match &err {
                        BondYieldError::Target { phase: p, .. }
                        | BondYieldError::Preparation { phase: p, .. } => {
                            assert_eq!(*p, phase, "{id} phase");
                        }
                        other => return Err(format!("{id} expected phase on {other:?}").into()),
                    }
                }
                let stage = record.text("yield_error_stage")?;
                if stage != "-" {
                    let expected_stage = match stage {
                        "lower_endpoint" => EvaluationStage::LowerEndpoint,
                        "upper_endpoint" => EvaluationStage::UpperEndpoint,
                        "trial_point" => EvaluationStage::TrialPoint,
                        other => return Err(format!("unknown stage {other}").into()),
                    };
                    match &err {
                        BondYieldError::Callback { stage: s, .. } => {
                            assert_eq!(*s, expected_stage, "{id} stage");
                        }
                        other => {
                            return Err(format!("{id} expected Callback, got {other:?}").into());
                        }
                    }
                }
                let evals_str = record.text("yield_error_evals")?;
                if evals_str != "-" {
                    let expected_err_evals: u32 = evals_str.parse()?;
                    let actual_err_evals = match &err {
                        BondYieldError::Callback { evaluations, .. }
                        | BondYieldError::NoYieldSensitivity { evaluations, .. }
                        | BondYieldError::ConstantPriceMismatch { evaluations, .. } => {
                            evaluations.function()
                        }
                        BondYieldError::Solver(
                            sabiquant::math::SolverError::NotBracketed { evaluations, .. }
                            | sabiquant::math::SolverError::EvaluationLimit { evaluations, .. }
                            | sabiquant::math::SolverError::PrecisionExhausted {
                                evaluations, ..
                            },
                        ) => evaluations.function(),
                        other => {
                            return Err(format!("{id} expected evaluations on {other:?}").into());
                        }
                    };
                    assert_eq!(
                        actual_err_evals, expected_err_evals,
                        "{id} yield_error_evals"
                    );
                }
            }
            other => return Err(format!("unknown yield_expected {other}").into()),
        }

        let risk_yield = parse_f64(record.text("risk_yield_bits")?)?;
        let risk_res = plan.risk(risk_yield);
        assert_eq!(
            plan.accrual_work_used(),
            initial_work,
            "{id} zero work on risk"
        );

        match record.text("risk_expected")? {
            "ok" => {
                let risk = risk_res.map_err(|e| format!("{id} unexpected risk err: {e}"))?;
                assert_in_enclosure(
                    id,
                    "risk_dirty",
                    risk.dirty_amount(),
                    record.text("risk_dirty_lo_bits")?,
                    record.text("risk_dirty_hi_bits")?,
                )?;
                check_risk_getter(
                    id,
                    "p1",
                    risk.price_derivative(),
                    record.text("p1_status")?,
                    record.text("p1_lo_bits")?,
                    record.text("p1_hi_bits")?,
                    record.text("ideal_p1_bits")?,
                )?;
                check_risk_getter(
                    id,
                    "p2",
                    risk.price_second_derivative(),
                    record.text("p2_status")?,
                    record.text("p2_lo_bits")?,
                    record.text("p2_hi_bits")?,
                    record.text("ideal_p2_bits")?,
                )?;
                check_risk_getter(
                    id,
                    "moment",
                    risk.time_weighted_amount(),
                    record.text("moment_status")?,
                    record.text("moment_lo_bits")?,
                    record.text("moment_hi_bits")?,
                    record.text("ideal_moment_bits")?,
                )?;
                check_risk_getter(
                    id,
                    "mod_dur",
                    risk.modified_duration(),
                    record.text("mod_dur_status")?,
                    record.text("mod_dur_lo_bits")?,
                    record.text("mod_dur_hi_bits")?,
                    record.text("ideal_mod_dur_bits")?,
                )?;
                check_risk_getter(
                    id,
                    "mac_dur",
                    risk.macaulay_duration(),
                    record.text("mac_dur_status")?,
                    record.text("mac_dur_lo_bits")?,
                    record.text("mac_dur_hi_bits")?,
                    record.text("ideal_mac_dur_bits")?,
                )?;
                check_risk_getter(
                    id,
                    "conv",
                    risk.convexity(),
                    record.text("conv_status")?,
                    record.text("conv_lo_bits")?,
                    record.text("conv_hi_bits")?,
                    record.text("ideal_conv_bits")?,
                )?;
            }
            "err" => {
                let Err(err) = risk_res else {
                    return Err(format!("{id} expected risk err").into());
                };
                assert_eq!(
                    err.kind(),
                    record.text("risk_error_kind")?,
                    "{id} risk kind"
                );
                let phase = record.text("risk_error_phase")?;
                let idx_str = record.text("risk_error_index")?;
                let expected_idx = if idx_str == "-" {
                    None
                } else {
                    Some(idx_str.parse::<usize>()?)
                };
                match &err {
                    BondError::Rate {
                        index, phase: p, ..
                    } => {
                        assert_eq!(*index, expected_idx, "{id} risk index");
                        assert_eq!(*p, phase, "{id} risk phase");
                    }
                    other => {
                        return Err(format!("{id} expected BondError::Rate, got {other:?}").into());
                    }
                }
            }
            other => return Err(format!("unknown risk_expected {other}").into()),
        }
    }

    let formatted_families = family_counts
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(";");
    assert_eq!(
        fixture.header("family-counts"),
        Some(formatted_families.as_str())
    );
    let formatted_yield = yield_counts
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(";");
    assert_eq!(
        fixture.header("yield-expectation-counts"),
        Some(formatted_yield.as_str())
    );
    let formatted_risk = risk_counts
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(";");
    assert_eq!(
        fixture.header("risk-expectation-counts"),
        Some(formatted_risk.as_str())
    );

    Ok(())
}

fn check_risk_getter(
    id: &str,
    label: &str,
    res: Result<f64, BondError>,
    status: &str,
    lo_hex: &str,
    hi_hex: &str,
    ideal_hex: &str,
) -> TestResult {
    match status {
        "ok" => {
            let val = res.map_err(|e| format!("{id} {label} unexpected err: {e}"))?;
            assert_in_enclosure(id, label, val, lo_hex, hi_hex)?;
            let ideal = parse_f64(ideal_hex)?;
            if ideal.is_finite() {
                assert!(
                    (val - ideal).abs() <= 1e-11 * (1.0 + ideal.abs()),
                    "{id} {label} ideal check: actual={val} ideal={ideal}"
                );
            }
        }
        "overflow" | "underflow" => {
            let Err(err) = res else {
                return Err(format!("{id} {label} expected {status}").into());
            };
            assert_eq!(err.kind(), "bond.value", "{id} {label} kind");
            assert!(matches!(err, BondError::Value { index: None, .. }));
        }
        "zero_dirty" => {
            let Err(err) = res else {
                return Err(format!("{id} {label} expected zero_dirty").into());
            };
            assert_eq!(err.kind(), "bond.zero_dirty_price", "{id} {label} kind");
            assert!(matches!(err, BondError::ZeroDirtyPrice { .. }));
        }
        other => return Err(format!("unknown getter status {other}").into()),
    }
    Ok(())
}
