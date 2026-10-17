//! Bounded signed rational images for the four settlement recipes.
//!
//! Source coordinates use D0=2^1074*5^38. Scaling and rounding retain their
//! complete ratios; only the final cash endpoints enter DecimalAmount.
//! The frozen contract bounds all intermediates below 2^2447 (40 limbs).

use super::{
    AmountError, AmountRounding, CashAggregation, DecimalAmount, EvidenceKind, Recipe,
    SettlementDecision, SettlementError, SettlementEvidence, SettlementInstruction,
    SettlementReceipt, SettlementRounding, capacity,
};
use crate::amounts::uint::Wide;

/// A private 2,560-bit unsigned magnitude, with no public numeric surface.
type Int = Wide<40>;

/// Signed magnitude; zero always has a positive sign.
#[derive(Clone, Copy, Eq, PartialEq)]
struct Signed {
    magnitude: Int,
    negative: bool,
}

impl Signed {
    fn new(magnitude: Int, negative: bool) -> Self {
        Self {
            magnitude,
            negative: negative && !magnitude.is_zero(),
        }
    }

    /// Exact signed addition, retaining cancellation before any range check.
    fn add(self, other: Self) -> Option<Self> {
        if self.negative == other.negative {
            Some(Self::new(
                self.magnitude.checked_add(other.magnitude)?,
                self.negative,
            ))
        } else if self.magnitude >= other.magnitude {
            Some(Self::new(
                self.magnitude.checked_sub(other.magnitude)?,
                self.negative,
            ))
        } else {
            Some(Self::new(
                other.magnitude.checked_sub(self.magnitude)?,
                other.negative,
            ))
        }
    }

    fn multiply(self, factor: Int) -> Option<Self> {
        Some(Self::new(
            self.magnitude.checked_mul(factor)?,
            self.negative,
        ))
    }
}

/// Ordered source endpoints on D0, with enough structure for Exact domains.
/// Each nonzero step denotes an independent, variable consecutive source count.
#[derive(Clone, Copy)]
struct Image {
    lower: Signed,
    upper: Signed,
    continuous: bool,
    steps: [Int; 2],
}

impl Image {
    fn is_point(self) -> bool {
        self.lower == self.upper
    }

    /// The recipes use only positive weights, so endpoint order is preserved.
    fn weight(self, weight: u32, stage: &'static str) -> Result<Self, Failure> {
        let factor = Int::from_u128(u128::from(weight));
        Ok(Self {
            lower: checked(self.lower.multiply(factor), stage, "scale cash")?,
            upper: checked(self.upper.multiply(factor), stage, "scale cash")?,
            continuous: self.continuous,
            steps: [
                checked(self.steps[0].checked_mul(factor), stage, "scale cash")?,
                checked(self.steps[1].checked_mul(factor), stage, "scale cash")?,
            ],
        })
    }

    /// The two evidence sources are independent; their variable steps remain
    /// separate even if their endpoint widths or signed bases coincide.
    fn combine(self, other: Self, stage: &'static str) -> Result<Self, Failure> {
        let mut steps = [Int::ZERO; 2];
        let mut used = 0;
        for step in self.steps.into_iter().chain(other.steps) {
            if !step.is_zero() {
                let slot = steps
                    .get_mut(used)
                    .ok_or_else(|| capacity(stage, "combine cash"))?;
                *slot = step;
                used += 1;
            }
        }
        Ok(Self {
            lower: checked(self.lower.add(other.lower), stage, "combine cash")?,
            upper: checked(self.upper.add(other.upper), stage, "combine cash")?,
            continuous: self.continuous || other.continuous,
            steps,
        })
    }
}

/// Rounded endpoint coefficients at the explicitly retained quantum scale.
/// The input distinction determines endpoint context even after output collapse.
struct Rounded {
    lower: Signed,
    upper: Signed,
    scale: u8,
    input_point: bool,
}

/// An Exact-domain result is a decision, not an arithmetic/representation error.
enum Failure {
    Error(SettlementError),
    Unresolved(&'static str),
}

impl From<SettlementError> for Failure {
    fn from(error: SettlementError) -> Self {
        Self::Error(error)
    }
}

fn checked<T>(
    value: Option<T>,
    stage: &'static str,
    operation: &'static str,
) -> Result<T, Failure> {
    value.ok_or_else(|| capacity(stage, operation).into())
}

/// Evaluates in frozen source-price/source-accrued/capital/accrued order.
pub(super) fn evaluate(
    instruction: &SettlementInstruction,
) -> Result<SettlementDecision, SettlementError> {
    match evaluate_inner(instruction) {
        Ok(decision) => Ok(decision),
        Err(Failure::Unresolved(stage)) => Ok(SettlementDecision::ExactStageUnresolved { stage }),
        Err(Failure::Error(error)) => Err(error),
    }
}

fn evaluate_inner(instruction: &SettlementInstruction) -> Result<SettlementDecision, Failure> {
    let first_stage = match instruction.recipe {
        Recipe::Staged { .. } => "source price",
        Recipe::Coupon => "cash coupon",
        Recipe::Dirty
        | Recipe::Clean {
            aggregation: CashAggregation::RoundTotal,
            ..
        } => "cash total",
        Recipe::Clean {
            aggregation: CashAggregation::RoundComponentsThenSum,
            ..
        } => "cash capital",
    };
    let basis = checked(
        Int::pow10(38).and_then(|x| x.checked_shl(1036)),
        first_stage,
        "decode evidence",
    )?;
    let primary = decode(instruction.primary, basis, first_stage)?;
    let (rounded, stage, operation) = match instruction.recipe {
        Recipe::Dirty | Recipe::Coupon => {
            let (divisor, stage) = if matches!(instruction.recipe, Recipe::Dirty) {
                (100, "cash total")
            } else {
                (1000, "cash coupon")
            };
            (
                cash_round(primary, instruction, basis, divisor, stage)?,
                stage,
                "quantize",
            )
        }
        Recipe::Clean {
            accrued,
            aggregation,
        } => {
            let secondary_stage = match aggregation {
                CashAggregation::RoundTotal => "cash total",
                CashAggregation::RoundComponentsThenSum => "cash accrued",
            };
            let secondary = decode(accrued, basis, secondary_stage)?;
            two_components(primary, secondary, aggregation, instruction, basis, false)?
        }
        Recipe::Staged {
            accrued,
            aggregation,
            price,
            accrued_rounding,
        } => {
            // Finish price Exact/image work before even decoding accrued.
            let primary = source_round(primary, basis, price, "source price")?;
            let secondary = decode(accrued, basis, "source accrued")?;
            let secondary = source_round(secondary, basis, accrued_rounding, "source accrued")?;
            two_components(primary, secondary, aggregation, instruction, basis, true)?
        }
    };
    let lower = admit(
        rounded.lower,
        rounded.scale,
        stage,
        if rounded.input_point {
            "point"
        } else {
            "lower"
        },
        operation,
    )?;
    let upper = if rounded.input_point {
        lower
    } else {
        admit(rounded.upper, rounded.scale, stage, "upper", operation)?
    };
    if lower == upper {
        Ok(SettlementDecision::Determined(SettlementReceipt {
            amount: lower,
            instruction: *instruction,
            guarantee: instruction.guarantee(),
        }))
    } else {
        Ok(SettlementDecision::QuantumAmbiguous {
            lower_amount: lower,
            upper_amount: upper,
        })
    }
}

/// Executes the prescribed total or capital-then-accrued stage sequence.
fn two_components(
    capital: Image,
    accrued: Image,
    aggregation: CashAggregation,
    instruction: &SettlementInstruction,
    basis: Int,
    staged: bool,
) -> Result<(Rounded, &'static str, &'static str), Failure> {
    let divisor = if staged { 1000 } else { 100 };
    match aggregation {
        CashAggregation::RoundTotal => {
            let capital = if staged {
                capital.weight(10, "cash total")?
            } else {
                capital
            };
            let total = capital.combine(accrued, "cash total")?;
            Ok((
                cash_round(total, instruction, basis, divisor, "cash total")?,
                "cash total",
                "quantize",
            ))
        }
        CashAggregation::RoundComponentsThenSum => {
            let capital = cash_round(capital, instruction, basis, 100, "cash capital")?;
            let accrued = cash_round(accrued, instruction, basis, divisor, "cash accrued")?;
            // Same cash policy means both coefficients have the same scale.
            let lower = checked(capital.lower.add(accrued.lower), "cash net", "combine cash")?;
            let upper = checked(capital.upper.add(accrued.upper), "cash net", "combine cash")?;
            Ok((
                Rounded {
                    lower,
                    upper,
                    scale: capital.scale,
                    input_point: lower == upper,
                },
                "cash net",
                "sum",
            ))
        }
    }
}

/// Converts exact validated evidence to integer coordinates on D0.
fn decode(evidence: SettlementEvidence, basis: Int, stage: &'static str) -> Result<Image, Failure> {
    let (lower, upper) = match evidence.kind {
        EvidenceKind::Decimal { lower, upper, .. } => (
            decode_decimal(lower, basis, stage)?,
            decode_decimal(upper, basis, stage)?,
        ),
        EvidenceKind::Binary { lower, upper, .. } => {
            (decode_binary(lower, stage)?, decode_binary(upper, stage)?)
        }
    };
    Ok(Image {
        lower,
        upper,
        continuous: lower != upper,
        steps: [Int::ZERO; 2],
    })
}

fn decode_decimal(
    value: DecimalAmount,
    basis: Int,
    stage: &'static str,
) -> Result<Signed, Failure> {
    let divisor = checked(
        Int::pow10(u16::from(value.scale())),
        stage,
        "decode evidence",
    )?;
    let (factor, remainder) = checked(basis.div_rem(divisor), stage, "decode evidence")?;
    if !remainder.is_zero() {
        return Err(capacity(stage, "decode evidence").into());
    }
    let magnitude = checked(
        factor.checked_mul(Int::from_u128(value.coefficient().unsigned_abs())),
        stage,
        "decode evidence",
    )?;
    Ok(Signed::new(magnitude, value.coefficient() < 0))
}

/// Finite bits M*2^e become M*5^38*2^(e+1074), including subnormals.
fn decode_binary(bits: u64, stage: &'static str) -> Result<Signed, Failure> {
    let field = (bits >> 52) & 0x7ff;
    let fraction = bits & ((1_u64 << 52) - 1);
    let (significand, shift) = if field == 0 {
        (fraction, 0)
    } else {
        (
            fraction | (1_u64 << 52),
            u32::try_from(field - 1).map_err(|_| capacity(stage, "decode evidence"))?,
        )
    };
    let magnitude = checked(
        Int::from_u128(u128::from(significand))
            .checked_mul(Int::from_u128(5_u128.pow(38)))
            .and_then(|x| x.checked_shl(shift)),
        stage,
        "decode evidence",
    )?;
    Ok(Signed::new(magnitude, bits >> 63 != 0))
}

/// Source quantizers have consecutive reachable integer counts. Reconstruct
/// their endpoints and one-count increment on D0 without public admission.
fn source_round(
    image: Image,
    basis: Int,
    policy: SettlementRounding,
    stage: &'static str,
) -> Result<Image, Failure> {
    let rounded = quantize(
        image,
        Int::from_u128(1),
        basis,
        policy,
        stage,
        "source rounding",
    )?;
    let power = checked(
        Int::pow10(u16::from(rounded.scale)),
        stage,
        "source rounding",
    )?;
    let (factor, remainder) = checked(basis.div_rem(power), stage, "source rounding")?;
    if !remainder.is_zero() {
        return Err(capacity(stage, "source rounding").into());
    }
    let lower = checked(rounded.lower.multiply(factor), stage, "source rounding")?;
    let upper = checked(rounded.upper.multiply(factor), stage, "source rounding")?;
    let step = if lower == upper {
        Int::ZERO
    } else {
        checked(
            factor.checked_mul(Int::from_u128(
                policy.quantum().amount().coefficient().unsigned_abs(),
            )),
            stage,
            "source rounding",
        )?
    };
    Ok(Image {
        lower,
        upper,
        continuous: false,
        steps: [step, Int::ZERO],
    })
}

/// Cash value = source_coordinate*Ncoef/(D0*10^Nscale*divisor).
fn cash_round(
    image: Image,
    instruction: &SettlementInstruction,
    basis: Int,
    divisor: u32,
    stage: &'static str,
) -> Result<Rounded, Failure> {
    let decimal = checked(
        Int::pow10(u16::from(instruction.nominal.scale())),
        stage,
        "scale cash",
    )?;
    let denominator = checked(
        basis
            .checked_mul(decimal)
            .and_then(|x| x.checked_mul_small(divisor)),
        stage,
        "scale cash",
    )?;
    quantize(
        image,
        Int::from_u128(instruction.nominal.coefficient().unsigned_abs()),
        denominator,
        instruction.cash,
        stage,
        "cash rounding",
    )
}

/// Rounds an exact rational image. Continuous nonpoint Exact images are
/// unresolved; discrete images need base AND every independent variable step.
fn quantize(
    image: Image,
    multiplier: Int,
    denominator: Int,
    policy: SettlementRounding,
    stage: &'static str,
    operation: &'static str,
) -> Result<Rounded, Failure> {
    let input_point = image.is_point();
    if policy.mode() == AmountRounding::Exact && !input_point && image.continuous {
        return Err(Failure::Unresolved(stage));
    }
    let quantum = policy.quantum().amount();
    let units = Int::from_u128(quantum.coefficient().unsigned_abs());
    let power = checked(Int::pow10(u16::from(quantum.scale())), stage, operation)?;
    let numerator_factor = checked(multiplier.checked_mul(power), stage, operation)?;
    let denominator = checked(denominator.checked_mul(units), stage, operation)?;
    if policy.mode() == AmountRounding::Exact && !input_point {
        for magnitude in [image.lower.magnitude, image.steps[0], image.steps[1]] {
            let numerator = checked(
                magnitude.checked_mul(numerator_factor),
                stage,
                "exact lattice",
            )?;
            let (_, remainder) = checked(numerator.div_rem(denominator), stage, "exact lattice")?;
            if !remainder.is_zero() {
                return Err(Failure::Unresolved(stage));
            }
        }
    }
    let lower = round_endpoint(
        image.lower,
        numerator_factor,
        denominator,
        units,
        policy.mode(),
        stage,
        if input_point { "point" } else { "lower" },
        operation,
    )?;
    let upper = if input_point {
        lower
    } else {
        round_endpoint(
            image.upper,
            numerator_factor,
            denominator,
            units,
            policy.mode(),
            stage,
            "upper",
            operation,
        )?
    };
    Ok(Rounded {
        lower,
        upper,
        scale: quantum.scale(),
        input_point,
    })
}

/// Signed seven-mode rounding uses exact quotient/remainder and d-r at ties.
#[expect(
    clippy::too_many_arguments,
    reason = "explicit frozen stage/endpoint and ratio components avoid hidden policy"
)]
fn round_endpoint(
    value: Signed,
    factor: Int,
    denominator: Int,
    units: Int,
    mode: AmountRounding,
    stage: &'static str,
    endpoint: &'static str,
    operation: &'static str,
) -> Result<Signed, Failure> {
    let numerator = checked(value.magnitude.checked_mul(factor), stage, operation)?;
    let (mut count, remainder) = checked(numerator.div_rem(denominator), stage, operation)?;
    if !remainder.is_zero() {
        if mode == AmountRounding::Exact {
            return Err(SettlementError::Arithmetic {
                stage,
                endpoint,
                cause: AmountError::Inexact {
                    operation: "quantize",
                },
            }
            .into());
        }
        let complement = checked(denominator.checked_sub(remainder), stage, operation)?;
        let increment = match mode {
            AmountRounding::TowardZero | AmountRounding::Exact => false,
            AmountRounding::AwayFromZero => true,
            AmountRounding::TowardPositiveInfinity => !value.negative,
            AmountRounding::TowardNegativeInfinity => value.negative,
            AmountRounding::NearestTiesEven => {
                remainder > complement || (remainder == complement && count.is_odd())
            }
            AmountRounding::NearestTiesAwayFromZero => remainder >= complement,
        };
        if increment {
            count = checked(count.checked_add(Int::from_u128(1)), stage, operation)?;
        }
    }
    Ok(Signed::new(
        checked(count.checked_mul(units), stage, operation)?,
        value.negative,
    ))
}

/// Canonicalize fully before checking asymmetric signed i128 limits.
fn admit(
    value: Signed,
    mut scale: u8,
    stage: &'static str,
    endpoint: &'static str,
    operation: &'static str,
) -> Result<DecimalAmount, Failure> {
    let mut magnitude = value.magnitude;
    if magnitude.is_zero() {
        scale = 0;
    }
    while scale > 0 {
        let (quotient, remainder) = checked(magnitude.div_rem_small(10), stage, "normalize cash")?;
        if remainder != 0 {
            break;
        }
        magnitude = quotient;
        scale -= 1;
    }
    let range = || SettlementError::Arithmetic {
        stage,
        endpoint,
        cause: AmountError::OutOfRange {
            operation,
            negative: value.negative,
        },
    };
    let magnitude = magnitude.to_u128().ok_or_else(range)?;
    let coefficient = if value.negative && magnitude == 1_u128 << 127 {
        i128::MIN
    } else {
        let coefficient = i128::try_from(magnitude).map_err(|_| range())?;
        if value.negative {
            -coefficient
        } else {
            coefficient
        }
    };
    DecimalAmount::new(coefficient, scale).map_err(|_| capacity(stage, "normalize cash").into())
}
