//! Public-producer test support for the frozen M8 enclosure fixtures.
//! No observed price or native elementary result supplies a price premise.

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    Bond, BondPriceKind, BondPricingPlan, BondSettlement, BondYieldConvention, EnclosureBasis,
    EnclosureBudget, EnclosureContext, EnclosureEncoding, EnclosureOrigin, EnclosureProofKind,
    EnclosureStage, ExCouponBoundary, ExCouponTerms, PriceEnclosure,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::DayCount;
use sabiquant::rates::Compounding;
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, ScheduleBuilder, ScheduleCalendar, StubPolicy,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::collections::BTreeMap;
use std::error::Error;

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub(crate) const ORDINARY_WORK: u64 = 1_000_000_000_000;
pub(crate) const PRECISIONS: [u16; 5] = [64, 128, 256, 512, 1024];

pub(crate) fn bits(text: &str) -> TestResult<u64> {
    if text.len() != 16
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(format!("invalid IEEE field {text}").into());
    }
    Ok(u64::from_str_radix(text, 16)?)
}
pub(crate) fn real(text: &str) -> TestResult<f64> {
    Ok(f64::from_bits(bits(text)?))
}
pub(crate) fn price_kind(text: &str) -> TestResult<BondPriceKind> {
    match text {
        "Dirty" => Ok(BondPriceKind::Dirty),
        "Clean" => Ok(BondPriceKind::Clean),
        _ => Err(format!("unexpected Generic basis {text}").into()),
    }
}

#[derive(Debug)]
pub(crate) struct Flat<'a> {
    nodes: BTreeMap<&'a str, (&'a str, &'a str)>,
}
impl<'a> Flat<'a> {
    pub(crate) fn parse(text: &'a str) -> TestResult<Self> {
        let mut nodes = BTreeMap::new();
        for cell in text.split(';') {
            let (path, typed) = cell.split_once('=').ok_or("node needs equals")?;
            let (tag, value) = typed.split_once(':').ok_or("node needs type")?;
            assert!(matches!(tag, "d" | "l" | "s" | "i" | "b" | "z"));
            assert!(nodes.insert(path, (tag, value)).is_none());
        }
        assert!(nodes.contains_key("$"));
        Ok(Self { nodes })
    }
    pub(crate) fn node(&self, path: &str) -> TestResult<(&'a str, &'a str)> {
        self.nodes
            .get(path)
            .copied()
            .ok_or_else(|| format!("missing literal {path}").into())
    }
    pub(crate) fn string(&self, path: &str) -> TestResult<&'a str> {
        let (tag, value) = self.node(path)?;
        assert_eq!(tag, "s");
        Ok(value)
    }
    pub(crate) fn integer(&self, path: &str) -> TestResult<usize> {
        let (tag, value) = self.node(path)?;
        assert_eq!(tag, "i");
        Ok(value.parse()?)
    }
    pub(crate) fn count(&self, path: &str) -> TestResult<usize> {
        let (tag, value) = self.node(path)?;
        assert_eq!(tag, "l");
        Ok(value.parse()?)
    }
    pub(crate) fn optional(&self, path: &str) -> TestResult<Option<&'a str>> {
        let (tag, value) = self.node(path)?;
        if tag == "z" {
            assert_eq!(value, "_");
            Ok(None)
        } else {
            assert_eq!(tag, "s");
            Ok(Some(value))
        }
    }
    pub(crate) fn number(&self, path: &str) -> TestResult<f64> {
        let (tag, value) = self.node(path)?;
        assert!(matches!(tag, "s" | "i"));
        Ok(value.parse()?)
    }
}

pub(crate) fn frequency(count: u32) -> TestResult<Frequency> {
    Frequency::ALL
        .into_iter()
        .find(|f| f.events_per_year() == Some(count))
        .ok_or_else(|| format!("unknown frequency {count}").into())
}
pub(crate) fn compounding(method: &str, count: Option<u32>) -> TestResult<Compounding> {
    Ok(match method {
        "Simple" => Compounding::SIMPLE,
        "Continuous" => Compounding::CONTINUOUS,
        "Compounded" => Compounding::compounded(frequency(count.ok_or("missing frequency")?)?)?,
        "SimpleThenCompounded" => {
            Compounding::simple_then_compounded(frequency(count.ok_or("missing frequency")?)?)?
        }
        "CompoundedThenSimple" => {
            Compounding::compounded_then_simple(frequency(count.ok_or("missing frequency")?)?)?
        }
        other => return Err(format!("unknown compounding {other}").into()),
    })
}
fn day_count(name: &str) -> TestResult<DayCount> {
    match name {
        "Actual365Fixed" => Ok(DayCount::Actual365Fixed),
        "Actual360" => Ok(DayCount::Actual360),
        _ => Err(format!("unknown daycount {name}").into()),
    }
}

#[derive(Debug)]
pub(crate) struct Flow {
    pub(crate) index: usize,
    pub(crate) amount_bits: u64,
    pub(crate) horizon_bits: u64,
    pub(crate) payment: Date,
}
#[derive(Debug)]
pub(crate) struct GenericSource {
    pub(crate) id: String,
    pub(crate) bond: Bond,
    pub(crate) convention: BondYieldConvention,
    pub(crate) settlement: BondSettlement,
    pub(crate) preparation_work: u64,
    pub(crate) accrued_bits: u64,
    pub(crate) flows: Vec<Flow>,
    pub(crate) useful: bool,
    pub(crate) configuration: String,
}
impl GenericSource {
    pub(crate) fn from_source(id: &str, literal: &str, summary: &str) -> TestResult<Self> {
        let input = Flat::parse(literal)?;
        let source = Flat::parse(summary)?;
        let issue: Date = input.string("issue")?.parse()?;
        let face = real(input.string("face_bits")?)?;
        let redemption = SimpleCashFlow::new(
            input.string("construction.redemption_payment")?.parse()?,
            real(input.string("construction.redemption_amount_bits")?)?,
        )?;
        let bond = match input.string("kind")? {
            "ZeroCoupon" => Bond::zero_coupon(
                issue,
                input.string("construction.contractual_maturity")?.parse()?,
                face,
                redemption,
            )?,
            "FixedRate" => {
                let p = "construction.schedule";
                assert_eq!(input.string(&format!("{p}.calendar"))?, "Null");
                assert_eq!(
                    input.string(&format!("{p}.ordinary_convention"))?,
                    "Unadjusted"
                );
                assert_eq!(
                    input.string(&format!("{p}.termination_convention"))?,
                    "Unadjusted"
                );
                assert_eq!(input.string(&format!("{p}.tenor_unit"))?, "Months");
                let rule = match input.string(&format!("{p}.rule"))? {
                    "Forward" => DateGenerationRule::Forward,
                    "Backward" => DateGenerationRule::Backward,
                    _ => return Err("unexpected rule".into()),
                };
                let eom = match input.string(&format!("{p}.end_of_month"))? {
                    "Ignore" => EndOfMonth::Ignore,
                    "Preserve" => EndOfMonth::Preserve,
                    _ => return Err("unexpected EOM".into()),
                };
                let first = input
                    .optional(&format!("{p}.first"))?
                    .map(str::parse)
                    .transpose()?;
                let penultimate = input
                    .optional(&format!("{p}.penultimate"))?
                    .map(str::parse)
                    .transpose()?;
                let schedule = ScheduleBuilder::new(
                    input.string(&format!("{p}.start"))?.parse()?,
                    input.string(&format!("{p}.end"))?.parse()?,
                    ScheduleCalendar::Plain(Calendar::null()),
                    BusinessDayConvention::Unadjusted,
                    BusinessDayConvention::Unadjusted,
                )?
                .build_explicit(
                    Period::new(
                        i32::try_from(input.integer(&format!("{p}.tenor_length"))?)?,
                        TimeUnit::Months,
                    ),
                    rule,
                    ExplicitStubDates::new(first, penultimate)?,
                    eom,
                    u32::try_from(input.integer(&format!("{p}.max_boundaries"))?)?,
                )?;
                let rate = FixedCouponRate::new(
                    real(input.string("coupon_rate_bits")?)?,
                    day_count(input.string("coupon_accrual")?)?,
                )?;
                let mut ex = Vec::new();
                for i in 0..input.count("construction.coupon_ex_cutoffs")? {
                    let cutoff = input.optional(&format!("construction.coupon_ex_cutoffs[{i}]"))?;
                    ex.push(if let Some(cutoff) = cutoff {
                        Some(ExCouponTerms::new(
                            cutoff.parse()?,
                            match input.string("ex_boundary")? {
                                "OnOrAfter" => ExCouponBoundary::OnOrAfter,
                                "After" => ExCouponBoundary::After,
                                _ => return Err("missing ex boundary".into()),
                            },
                        ))
                    } else {
                        None
                    });
                }
                Bond::fixed_rate(
                    issue,
                    schedule,
                    face,
                    rate,
                    ex,
                    redemption,
                    u32::try_from(input.integer("construction.max_flows")?)?,
                )?
            }
            other => return Err(format!("unknown instrument {other}").into()),
        };
        let method = input.string("compounding")?;
        let (tag, freq) = input.node("frequency")?;
        let count = if tag == "z" {
            None
        } else {
            assert_eq!(tag, "i");
            Some(freq.parse()?)
        };
        let convention = BondYieldConvention::new(
            day_count(input.string("yield_accrual")?)?,
            compounding(method, count)?,
        );
        let inclusion = match input.string("inclusion")? {
            "Include" => PaymentDateInclusion::Include,
            "Exclude" => PaymentDateInclusion::Exclude,
            _ => return Err("unknown inclusion".into()),
        };
        let settlement = BondSettlement::new(input.string("settlement")?.parse()?, inclusion);
        let accrued_bits = bits(source.string("cached_accrued_bits")?)?;
        let mut flows = Vec::new();
        for i in 0..source.count("prepared_flows")? {
            flows.push(Flow {
                index: source.integer(&format!("prepared_flows[{i}].index"))?,
                amount_bits: bits(source.string(&format!("prepared_flows[{i}].amount_bits"))?)?,
                horizon_bits: bits(source.string(&format!("prepared_flows[{i}].horizon_bits"))?)?,
                payment: source
                    .string(&format!("prepared_flows[{i}].payment"))?
                    .parse()?,
            });
        }
        let useful = (1.0..=1024.0).contains(&face)
            && f64::from_bits(accrued_bits).abs() <= 1024.0
            && flows
                .iter()
                .all(|f| f64::from_bits(f.amount_bits).abs() <= 1024.0);
        Ok(Self {
            id: id.to_owned(),
            bond,
            convention,
            settlement,
            preparation_work: u64::try_from(input.integer("construction.max_accrual_work")?)?,
            accrued_bits,
            flows,
            useful,
            configuration: format!(
                "{method}:{}",
                count.map_or_else(|| "-".to_owned(), |n| n.to_string())
            ),
        })
    }
    pub(crate) fn plan(&self) -> TestResult<BondPricingPlan<'_>> {
        let plan = BondPricingPlan::new(
            &self.bond,
            self.settlement,
            &self.convention,
            self.preparation_work,
        )?;
        assert_eq!(
            plan.accrued_amount().to_bits(),
            self.accrued_bits,
            "{} actual cachedAI",
            self.id
        );
        assert_eq!(plan.entitled_flow_count(), self.flows.len());
        assert!(
            std::ptr::eq(plan.bond(), &self.bond)
                && std::ptr::eq(plan.yield_convention(), &self.convention)
        );
        for flow in &self.flows {
            let (amount, payment) = if let Some(c) = self.bond.coupons().get(flow.index) {
                (c.amount(), c.payment_date())
            } else {
                assert_eq!(flow.index, self.bond.coupons().len());
                (
                    self.bond.redemption().amount(),
                    self.bond.redemption().payment_date(),
                )
            };
            assert_eq!(amount.to_bits(), flow.amount_bits);
            assert_eq!(payment, flow.payment);
            // Public direct reconstruction is indirect; actual cached-vector
            // binding comes from the new producer's complete target checks.
            assert_eq!(
                self.convention
                    .accrual()
                    .year_fraction(self.settlement.date(), payment)?
                    .to_bits(),
                flow.horizon_bits
            );
        }
        Ok(plan)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Preferred {
    Decimal(DecimalAmount),
    Binary(u64),
    NoFit,
    NotProved,
}
#[derive(Clone, Debug)]
pub(crate) struct Reference {
    pub(crate) lower: Option<f64>,
    pub(crate) upper: Option<f64>,
    pub(crate) point: Preferred,
}
impl Reference {
    pub(crate) fn price(lower: &str, upper: &str, encoding: &str, point: &str) -> TestResult<Self> {
        let point = match encoding {
            "Decimal" => {
                let (n, s) = point.split_once(':').ok_or("missing decimal separator")?;
                Preferred::Decimal(DecimalAmount::new(n.parse()?, s.parse()?)?)
            }
            "Binary" => Preferred::Binary(bits(point)?),
            "NoFittingPoint" => Preferred::NoFit,
            "NotProvedInterval" => Preferred::NotProved,
            _ => return Err("unknown preferred point".into()),
        };
        Ok(Self {
            lower: if lower == "-" {
                None
            } else {
                Some(real(lower)?)
            },
            upper: if upper == "-" {
                None
            } else {
                Some(real(upper)?)
            },
            point,
        })
    }
    pub(crate) fn hand(lower: &str, upper: &str, kind: &str, decimal: &str) -> TestResult<Self> {
        let point = if kind != "exact_rational" {
            Preferred::NotProved
        } else if decimal != "-" {
            Preferred::Decimal(decimal.parse()?)
        } else if lower == upper {
            Preferred::Binary(bits(lower)?)
        } else {
            Preferred::NoFit
        };
        Ok(Self {
            lower: Some(real(lower)?),
            upper: Some(real(upper)?),
            point,
        })
    }
    pub(crate) fn boundary(
        lower: &str,
        upper: &str,
        kind: &str,
        coefficient: &str,
        scale: &str,
        point_bits: &str,
    ) -> TestResult<Self> {
        let point = match kind {
            "decimal" => {
                Preferred::Decimal(DecimalAmount::new(coefficient.parse()?, scale.parse()?)?)
            }
            "binary" => Preferred::Binary(bits(point_bits)?),
            "none" => Preferred::NoFit,
            "unclassified" => Preferred::NotProved,
            _ => return Err("unknown B2 point".into()),
        };
        Ok(Self {
            lower: if lower == "_" {
                None
            } else {
                Some(real(lower)?)
            },
            upper: if upper == "_" {
                None
            } else {
                Some(real(upper)?)
            },
            point,
        })
    }
    pub(crate) fn compare(
        &self,
        report: &PriceEnclosure<'_, '_>,
        useful: bool,
        label: &str,
    ) -> TestResult {
        let evidence = report.settlement_evidence();
        assert!(
            evidence.is_declared_interval(),
            "{label} declared provenance"
        );
        match report.encoding() {
            EnclosureEncoding::ExactDecimalPoint => {
                let Some((lo, hi)) = evidence.decimal_bounds() else {
                    return Err(
                        format!("{label}: decimal encoding missing decimal evidence").into(),
                    );
                };
                assert_eq!(lo, hi, "{label}");
                assert_eq!(report.decimal_point(), Some(lo));
                let Preferred::Decimal(expected) = self.point else {
                    return Err(format!("{label}: independently unproved decimal point").into());
                };
                assert_eq!(lo, expected, "{label} exact preferred point");
                assert!(matches!(
                    report.proof_kind(),
                    EnclosureProofKind::ExactRationalIdentity
                        | EnclosureProofKind::IntervalCollapse
                ));
            }
            EnclosureEncoding::ExactBinary64Point => {
                let Some((lo, hi)) = evidence.binary64_bounds() else {
                    return Err("binary point lacks binary evidence".into());
                };
                assert_eq!(lo.to_bits(), hi.to_bits());
                let Preferred::Binary(expected) = self.point else {
                    return Err(format!(
                        "{label}: binary point lost Decimal preference or is unproved"
                    )
                    .into());
                };
                assert_eq!(lo.to_bits(), expected, "{label}");
                assert!(matches!(
                    report.proof_kind(),
                    EnclosureProofKind::ExactRationalIdentity
                        | EnclosureProofKind::IntervalCollapse
                ));
            }
            EnclosureEncoding::OutwardBinary64Bounds => {
                let Some((lo, hi)) = evidence.binary64_bounds() else {
                    return Err("binary bounds missing".into());
                };
                assert!(
                    lo.is_finite() && hi.is_finite() && lo < hi,
                    "{label} strict finite bounds"
                );
                let (Some(reference_lo), Some(reference_hi)) = (self.lower, self.upper) else {
                    return Err(
                        format!("{label}: finite enclosure for true nonexportable target").into(),
                    );
                };
                assert!(
                    lo <= reference_lo && hi >= reference_hi,
                    "{label}: containment failed actual[{lo:?},{hi:?}] reference[{reference_lo:?},{reference_hi:?}]"
                );
                if matches!(
                    report.proof_kind(),
                    EnclosureProofKind::ExactRationalIdentity
                        | EnclosureProofKind::IntervalCollapse
                ) {
                    assert!(
                        matches!(self.point, Preferred::NoFit),
                        "{label}: known point failed fitting preference"
                    );
                }
                if useful {
                    let width = hi - lo;
                    assert!(width.is_finite() && width > 0.0, "{label} width");
                    // Correctly rounded positive subtraction lies below its
                    // successor. This is a conservative DYADIC upper bound;
                    // the RHS power-of-two scale is exact and normal (>=2^-36).
                    let upper_width = f64::from_bits(width.to_bits() + 1);
                    let scale = 1.0_f64.max(reference_lo.abs()).max(reference_hi.abs());
                    let allowance = scale * f64::from_bits((1023_u64 - 36) << 52);
                    assert!(
                        upper_width <= allowance,
                        "{label}: useful width {upper_width:?} exceeds {allowance:?}"
                    );
                }
            }
            _ => return Err("unknown enclosure encoding".into()),
        }
        Ok(())
    }
}

pub(crate) fn check_context(
    context: EnclosureContext,
    budget: EnclosureBudget,
    quote: u64,
    kind: BondPriceKind,
    count: usize,
    successful: bool,
) {
    assert_eq!(context.budget(), budget);
    assert!(context.work_used() <= budget.max_work());
    assert_eq!(context.basis(), EnclosureBasis::Generic(kind));
    assert_eq!(
        context.yield_bits(),
        if successful && quote & !(1_u64 << 63) == 0 {
            0
        } else {
            quote
        }
    );
    assert!(
        context.admission_checked_flows() <= count && context.reduction_completed_flows() <= count
    );
    if context.admission_complete() {
        assert_eq!(context.admission_checked_flows(), count);
    }
}
pub(crate) fn check_origin(origin: EnclosureOrigin<'_, '_>, plan: &BondPricingPlan<'_>) {
    assert!(origin.generic_plan().is_some_and(|p| std::ptr::eq(p, plan)));
    assert!(
        origin.gilt_plan().is_none()
            && origin.treasury_plan().is_none()
            && origin.canadian_plan().is_none()
    );
}
pub(crate) fn finite_complete(
    report: &PriceEnclosure<'_, '_>,
    plan: &BondPricingPlan<'_>,
    budget: EnclosureBudget,
    quote: u64,
    kind: BondPriceKind,
    count: usize,
) {
    check_origin(report.origin(), plan);
    check_context(report.context(), budget, quote, kind, count, true);
    assert!(report.context().admission_complete());
    assert_eq!(report.context().reduction_completed_flows(), count);
    assert_eq!(report.context().next_original_index(), None);
    assert_eq!(report.context().stage(), EnclosureStage::Completion);
}

pub(crate) fn bare(text: &str) -> TestResult<BTreeMap<&str, &str>> {
    let mut values = BTreeMap::new();
    for part in text.split(';') {
        let (k, v) = part.split_once('=').ok_or("missing bare literal equals")?;
        assert!(!k.is_empty() && !v.is_empty() && values.insert(k, v).is_none());
    }
    Ok(values)
}
pub(crate) fn item<'a>(map: &BTreeMap<&'a str, &'a str>, key: &str) -> TestResult<&'a str> {
    map.get(key)
        .copied()
        .ok_or_else(|| format!("missing {key}").into())
}
impl GenericSource {
    pub(crate) fn from_boundary(
        id: &str,
        literal: &str,
        method: &str,
        accrued: &str,
        flow_cell: &str,
    ) -> TestResult<Self> {
        let v = bare(literal)?;
        let issue: Date = item(&v, "issue")?.parse()?;
        let face = real(item(&v, "face_bits")?)?;
        let redemption = SimpleCashFlow::new(
            item(&v, "redemption_payment")?.parse()?,
            real(item(&v, "redemption_bits")?)?,
        )?;
        let bond = if item(&v, "kind")? == "ZeroCoupon" {
            Bond::zero_coupon(
                issue,
                item(&v, "contractual_maturity")?.parse()?,
                face,
                redemption,
            )?
        } else {
            assert_eq!(item(&v, "kind")?, "FixedRate");
            let schedule = ScheduleBuilder::new(
                item(&v, "schedule_start")?.parse()?,
                item(&v, "schedule_end")?.parse()?,
                ScheduleCalendar::Plain(Calendar::null()),
                BusinessDayConvention::Unadjusted,
                BusinessDayConvention::Unadjusted,
            )?
            .build_recurrent(
                Period::new(item(&v, "schedule_step_months")?.parse()?, TimeUnit::Months),
                DateGenerationRule::Forward,
                StubPolicy::None,
                EndOfMonth::Ignore,
                item(&v, "schedule_max_boundaries")?.parse()?,
            )?;
            let ex = item(&v, "ex_coupon")?
                .split(',')
                .map(|s| {
                    assert_eq!(s, "_");
                    None
                })
                .collect();
            Bond::fixed_rate(
                issue,
                schedule,
                face,
                FixedCouponRate::new(
                    real(item(&v, "coupon_rate_bits")?)?,
                    DayCount::Actual365Fixed,
                )?,
                ex,
                redemption,
                item(&v, "max_flows")?.parse()?,
            )?
        };
        let mut flows = Vec::new();
        for part in flow_cell.split(';') {
            let pieces = part.split('@').collect::<Vec<_>>();
            let [index, payment, amount, horizon] = pieces.as_slice() else {
                return Err("malformed boundary flow".into());
            };
            flows.push(Flow {
                index: index.parse()?,
                payment: payment.parse()?,
                amount_bits: bits(amount)?,
                horizon_bits: bits(horizon)?,
            });
        }
        Ok(Self {
            id: id.to_owned(),
            bond,
            convention: BondYieldConvention::new(
                DayCount::Actual365Fixed,
                compounding(method, None)?,
            ),
            settlement: BondSettlement::new(
                item(&v, "settlement")?.parse()?,
                PaymentDateInclusion::Include,
            ),
            preparation_work: item(&v, "max_accrual_work")?.parse()?,
            accrued_bits: bits(accrued)?,
            flows,
            useful: false,
            configuration: method.to_owned(),
        })
    }
    pub(crate) fn from_hand(id: &str, literal: &str, source: &str) -> TestResult<Self> {
        let v = Flat::parse(literal)?;
        let s = Flat::parse(source)?;
        let issue: Date = v.string("issue")?.parse()?;
        let maturity: Date = v.string("maturity")?.parse()?;
        let face = v.number("face")?;
        let redemption_amount = v.number("redemption")?;
        let redemption = SimpleCashFlow::new(maturity, redemption_amount)?;
        let dc = day_count(v.string("day_count")?)?;
        let bond = if v.string("kind")? == "FixedRateScheduleZero" {
            assert_eq!(v.string("calendar")?, "weekends_only");
            assert_eq!(v.string("adjustment")?, "Unadjusted");
            let schedule = ScheduleBuilder::new(
                issue,
                maturity,
                ScheduleCalendar::Plain(Calendar::weekends_only()),
                BusinessDayConvention::Unadjusted,
                BusinessDayConvention::Unadjusted,
            )?
            .build_zero(2)?;
            Bond::fixed_rate(
                issue,
                schedule,
                face,
                FixedCouponRate::new(v.number("coupon_rate")?, dc)?,
                vec![None],
                redemption,
                u32::try_from(v.integer("max_flows")?)?,
            )?
        } else {
            assert_eq!(v.string("kind")?, "ZeroCoupon");
            Bond::zero_coupon(issue, maturity, face, redemption)?
        };
        let method = v.string("compounding")?;
        let c = if method == "Compounded(Annual)" {
            Compounding::compounded(Frequency::Annual)?
        } else {
            compounding(method, None)?
        };
        // Existing Generic hands use integer or power-of-two rational source
        // coordinates. This converts their frozen source representation only.
        let dyadic = |name: &str| -> TestResult<f64> {
            let n = s.string(&format!("{name}[0]"))?;
            let d = s.string(&format!("{name}[1]"))?;
            let numerator: i64 = n.parse()?;
            let denominator: u64 = d.parse()?;
            assert!(
                numerator.unsigned_abs() <= (1_u64 << 53)
                    && denominator.is_power_of_two()
                    && denominator <= (1_u64 << 53)
            );
            Ok(n.parse::<f64>()? / d.parse::<f64>()?)
        };
        let horizon_bits = dyadic("prepared_horizon")?.to_bits();
        let mut flows = Vec::new();
        if v.string("kind")? == "FixedRateScheduleZero" {
            flows.push(Flow {
                index: 0,
                amount_bits: dyadic("prepared_coupon")?.to_bits(),
                horizon_bits,
                payment: maturity,
            });
        }
        flows.push(Flow {
            index: flows.len(),
            amount_bits: redemption_amount.to_bits(),
            horizon_bits,
            payment: maturity,
        });
        Ok(Self {
            id: id.to_owned(),
            bond,
            convention: BondYieldConvention::new(dc, c),
            settlement: BondSettlement::new(
                v.string("settlement")?.parse()?,
                PaymentDateInclusion::Include,
            ),
            preparation_work: u64::try_from(v.integer("preparation_max_work")?)?,
            accrued_bits: dyadic("cached_AI")?.to_bits(),
            flows,
            useful: false,
            configuration: method.to_owned(),
        })
    }
}
