# ADR-0003: QuantLib Functional Compatibility Philosophy

| Field | Value |
|---|---|
| ADR identifier | ADR-0003 |
| Title | QuantLib functional compatibility philosophy |
| Status | **Accepted — owner constraint** for D1. D2–D5 **accepted under delegated authority on 2026-10-11** ([decision record](#decision-record)) |
| Date | 2026-10-10 |
| Decision owner | Project owner |

## Context

SabiQuant's long-term ambition is parity with QuantLib's major domains. The
word "compatibility" is ambiguous. It can mean having the same features,
behaving the same way, producing the same numbers, or exposing the same
interfaces. Unqualified claims would mislead users. The owner's brief rules
out API compatibility as a goal and requires independent verification before
any claim is made.

## Decision proposal

**D1 (owner constraint).** SabiQuant aims for **behavioral and numerical**
compatibility with QuantLib within specified domains. **API compatibility is
not a goal.** QuantLib's API does not dictate SabiQuant's architecture.

**D2 (proposed). Definitions.**

| Category | Definition | Goal? |
|---|---|---|
| Functional parity | SabiQuant offers an equivalent financial capability | Long-term, per domain |
| Behavioral parity | Under the same conventions and inputs, observable behavior is the same: dates, adjustments, error versus value, branch choices | Yes, for features in scope, except **documented intentional divergences** |
| Numerical parity | Real-valued outputs agree within justified tolerances ([NUMERICAL_CORRECTNESS §7](../NUMERICAL_CORRECTNESS.md#7-tolerance-policy)) | Yes, for features in scope |
| API compatibility | Same type names, signatures, or class structure | **No** |

**D3 (proposed). QuantLib is not presumed correct.** When QuantLib disagrees
with authoritative sources, SabiQuant follows the authoritative source and
records the difference (classes D7 and D8 in
[TESTING_STRATEGY §7.5](../TESTING_STRATEGY.md#75-discrepancy-classification)).
SabiQuant may also be *stricter* on purpose, for example by rejecting inputs
outside a domain where QuantLib returns NaN. Such cases are intentional
divergences (D9) and are documented in the specification.

**D4 (proposed). Compatibility status for each feature.**

| Status | Meaning |
|---|---|
| Not assessed | No differential evidence yet |
| Under validation | Fixtures exist; some discrepancies are unresolved |
| Verified | Every fixture in the documented scenario set passes against the named QuantLib version, and every discrepancy is classified and resolved |
| Verified with divergences | As Verified, plus documented intentional divergences |
| Not compatible | Behavior differs and is not planned to converge (documented) |

**D5 (proposed). Claims policy.**

1. A compatibility claim always names the **feature**, the **QuantLib version
   (and commit)**, the **scenario set**, and the **tolerance policy**. For
   example: "Actual/360 year fractions: verified against QuantLib 1.43 on
   fixture set DCC-A360 (TOL-E1)."
2. A blanket statement such as "SabiQuant is QuantLib-compatible" is never
   made.
3. The README and documentation list compatibility per feature only after
   verification.
4. Statements of intent, such as "aims for parity", are worded as intent.
5. The QuantLib name is used only to describe the reference, never to imply
   endorsement (ADR-0010; the BSD licence's no-endorsement clause).

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| API compatibility (same class names and signatures) | Ruled out by the owner; would force C++ idioms into Rust |
| No QuantLib reference at all | Loses the best available independent benchmark for conventions |
| Bitwise numerical parity | Impossible to guarantee across platforms and `libm` implementations, and incompatible with SabiQuant's numerically stable formulations |
| Treat QuantLib as ground truth | QuantLib's own tests note disagreements with ISDA documents (QR-01); ground truth must come from primary sources |

## Advantages

- Claims are precise, verifiable, and honest.
- SabiQuant is free to use Rust-native designs.
- Defects can be found in either library, not only in SabiQuant.

## Disadvantages

- Users who expect drop-in API familiarity will need to learn SabiQuant's
  naming. Documentation search aliases and mapping tables help
  ([API_DESIGN_PRINCIPLES §2](../API_DESIGN_PRINCIPLES.md#2-naming-conventions)).
- The validation workload is significant.

## Consequences

- Every feature in the matrix has a "compatibility considerations" note
  ([FEATURE_MATRIX](../FEATURE_MATRIX.md)).
- Differential infrastructure is needed from M2 (ADR-0008).
- Release notes report changes in compatibility status.

## Risks

- R-09: drift when the QuantLib baseline moves. Mitigated by pinning and the
  re-baselining procedure.
- R-10: over-claiming. Mitigated by D5 and review.

## Reconsideration triggers

- The owner changes the long-term goal, for example to provide a
  QuantLib-like API layer. That would need a new ADR, and D1 would change.
- QuantLib changes the behavior of a convention SabiQuant has verified. That
  triggers re-validation, not a change of philosophy.

## Related requirements and documents

C-03, C-04, C-08; [QUANTLIB_REFERENCE](../QUANTLIB_REFERENCE.md);
[ADR-0008](ADR-0008-validation-strategy.md); [ADR-0010](ADR-0010-licensing.md).

## Decision record

On 2026-10-11 the owner authorized the agent to choose the next work and
continue autonomously with broad tests. D2–D5 were accepted under that
delegation for M2: feature-specific compatibility definitions, independent
source priority, qualified claims and discrepancy handling. See
[M2_READINESS](../M2_READINESS.md). No blanket compatibility claim is made.
