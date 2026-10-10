# ADR-0006: Floating-Point and Numerical Precision Policy

| Field | Value |
|---|---|
| ADR identifier | ADR-0006 |
| Title | Floating-point and numerical precision policy |
| Status | **Proposed** |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M1 ([OQ-06](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

## Context

Financial calculations need well-understood accuracy. Rust's `f64` follows
IEEE 754 binary64, but the documentation describes transcendental functions
as having unspecified, non-deterministic precision that may vary by platform
and version. The standard library has no decimal type, no arbitrary-precision
arithmetic, and no stable special functions. Third-party crates for these are
excluded (ADR-0002). QuantLib uses a configurable `Real`, which is normally
`double`, and a default closeness test with a fixed 42ε.

## Decision proposal

1. **Concrete `f64`** for every real-valued quantity in H1. No generic numeric
   abstraction.
2. **The policies NP-01 to NP-12** in
   [NUMERICAL_CORRECTNESS §5](../NUMERICAL_CORRECTNESS.md#5-numerical-policies):
   finite inputs; finite, in-domain outputs; stable formulations; exact
   discrete arithmetic; one division for integer-ratio year fractions; no
   fast-math; no NaN output; signed-zero neutrality; deterministic summation;
   documented operation order; careful branch conditions.
3. **Exactness classes and tolerance policies** E0 to E5, with no global
   epsilon ([NUMERICAL_CORRECTNESS §7](../NUMERICAL_CORRECTNESS.md#7-tolerance-policy)).
   The TOL-E3 and TOL-E4 constants are calibrated with evidence in M4.
4. **Reproducibility levels** R1 to R3. Bitwise determinism on one platform.
   Cross-platform agreement within tolerance for transcendental results.
5. **Stable formulations** with `log1p` and `expm1` for compounding, implied
   rates, and equivalent rates. Their numerical differences from QuantLib are
   expected and accounted for.
6. **Exact decimal settlement amounts** are out of scope for H1 and recorded
   as NI-01.

## Alternatives considered

| Alternative | Assessment |
|---|---|
| Generic numeric type parameter (allows AD and decimals later) | Expensive in API complexity and compile time now. Benefits unproven for H1. Recorded as NI-02 |
| In-house decimal arithmetic for everything | Slow. Transcendental functions in decimal are hard. Not needed for analytics |
| Extended or quad precision | Not assumed available on stable Rust. Unnecessary for H1 |
| In-house deterministic elementary functions now | Large verification effort before any user needs it. Deferred (NI-03) |
| Global epsilon comparisons (QuantLib style) | Tolerances without justification. Rejected |

## Advantages

- Simple, fast, universally understood numerics.
- Accuracy statements rest on conditioning and evidence.
- Tests are robust across platforms without hiding real defects.

## Disadvantages

- Automatic differentiation cannot be added later without broad refactoring.
- Results are not guaranteed bitwise identical across platforms for
  transcendental functions.
- Exact cash amounts need a separate future design.

## Consequences

- Every real-valued output declares its exactness class and tolerance policy
  ID in its documentation and tests.
- M4 has to produce measured error distributions.
- Release notes flag any change in numerical output (ADR-0009).

## Risks

- R-04: platform `libm` differences cause test failures. Mitigated by tolerance
  policies and expected values that are never platform-specific.
- R-05: false discrepancies against QuantLib's naive formulas. Mitigated by
  the high-precision oracle and discrepancy classification.
- R-08: the concrete-`f64` choice blocks AAD.

## Reconsideration triggers

- An owner decision that AAD or generic scalars are required. This must
  happen **before** pricing engines (M10/M11), because retrofitting is costly.
- Cash settlement or accounting features (M7/M8), which lead to NI-01.
- A user or regulatory need for bitwise cross-platform reproducibility, which
  leads to NI-03.
- Evidence that `f64` is inadequate for a planned domain.
- The scalar-wrapper question (OQ-24) after the M4 API review.

## Related requirements and documents

NFR-03, NFR-04, NFR-14; [NUMERICAL_CORRECTNESS](../NUMERICAL_CORRECTNESS.md);
[TESTING_STRATEGY](../TESTING_STRATEGY.md); [ADR-0008](ADR-0008-validation-strategy.md).
