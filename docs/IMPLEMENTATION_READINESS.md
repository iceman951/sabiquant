# Implementation Readiness Assessment

> This records the M1 authorization assessment. M1 has since merged with
> passing primary-platform CI. The current milestone's authorization and
> delegated decisions are in [M2_READINESS](M2_READINESS.md); local validation
> evidence is in [M2_VALIDATION](M2_VALIDATION.md).

| Field | Value |
|---|---|
| Milestone | M0, final review, re-run after the owner's decisions |
| Assessment date | 2026-10-10 |
| Assessed against | Readiness to begin **M1 (core foundations)**. Items that block later milestones are listed with the milestone they block |
| Overall result | **AUTHORIZED for M1.** Every gate passes, and the project owner explicitly authorized M1 on 2026-10-10. Decisions that block M2 and later milestones remain open (§5) |

### Assessment history

| Date | Event | Result |
|---|---|---|
| 2026-10-10 | M0 final review | Not authorized. Gates A, B and D passed; C and E needed owner decisions; F was blocked |
| 2026-10-10 | Re-run after the owner accepted the M1 recommendations and authorized M1 | **Authorized for M1.** Every gate passes |

## 1. Rules of this assessment

| Status | Meaning |
|---|---|
| **PASS** | The criterion is met, and the evidence is linked |
| **NEEDS DECISION** | The material exists, but an owner decision or approval is outstanding |
| **BLOCKED** | It cannot pass until an external action happens |

- Implementation may begin **only** when every mandatory gate passes **and**
  the project owner grants permission separately and explicitly.
- **Passing this review does not authorize implementation**, and neither does
  write access to the repository.
- An assessment marked PASS for M1 may still list research items that block
  M2 or later. Those items are in the decision register (§5) and in the
  dependencies of each milestone in the [ROADMAP](ROADMAP.md).

## 2. Summary

| Gate | Topic | Result |
|---|---|---|
| A | Vision and scope | **PASS** |
| B | Domain research | **PASS** (M3 research items are tracked) |
| C | Architecture | **PASS** (the ADRs that M1 needs are accepted) |
| D | Financial correctness | **PASS** (fixture format decided, OQ-13) |
| E | Open-source readiness | **PASS** (licence selected: `MIT OR Apache-2.0`) |
| F | Implementation authorization | **PASS** (the owner authorized M1 on 2026-10-10) |

## 3. Gate assessments

### Gate A — Vision and scope

| # | Criterion | Status | Evidence |
|---|---|---|---|
| A1 | The project mission is clear | PASS | [VISION §1](VISION.md#1-mission) |
| A2 | The target audience is identified | PASS | [VISION §5](VISION.md#5-target-users) |
| A3 | The initial scope is bounded | PASS | [SCOPE §2](SCOPE.md#2-foundations-scope-h1); [FEATURE_MATRIX](FEATURE_MATRIX.md) §2–§6 (79 rows with status) |
| A4 | Non-goals are explicit | PASS | [SCOPE §3](SCOPE.md#3-explicit-non-goals) (15 non-goals) |
| A5 | Long-term parity goals are kept separate from initial requirements | PASS | [SCOPE §4](SCOPE.md#4-long-term-parity-versus-initial-requirements); [ADR-0003](adr/ADR-0003-quantlib-compatibility.md) |

### Gate B — Domain research

| # | Criterion | Status | Evidence |
|---|---|---|---|
| B1 | The QuantLib reference baseline is documented | PASS | QuantLib 1.43, tag `v1.43`, commit `6b57206e04598f092efee66e3b367efc84771995`, released 2026-07-14 ([QUANTLIB_REFERENCE §2](QUANTLIB_REFERENCE.md#2-reference-baseline)) |
| B2 | Foundational features are identified | PASS | [FEATURE_MATRIX §2–§6](FEATURE_MATRIX.md#2-core) |
| B3 | Required financial conventions are specified | PASS for M1. M3 items tracked | [REQUIREMENTS §4–§11](REQUIREMENTS.md#4-fs-core--core-types). OQ-15/16/21 now have local executable contracts; full SIFMA source acceptance, calendar maintenance, review and native CI remain milestone gates |
| B4 | Major domain dependencies are understood | PASS | [QUANTLIB_REFERENCE §7](QUANTLIB_REFERENCE.md#7-domain-dependency-graph-quantlib-simplified); [DOMAIN_MODEL §7](DOMAIN_MODEL.md#7-concept-dependencies-foundations); [ROADMAP §1](ROADMAP.md#1-milestone-dependency-graph) |
| B5 | Research gaps are visible | PASS | [QUANTLIB_REFERENCE §8](QUANTLIB_REFERENCE.md#8-known-reference-side-issues-and-ambiguities) and [§10](QUANTLIB_REFERENCE.md#10-research-limitations) (L-01 to L-10, QR-01 to QR-05) |

### Gate C — Architecture

| # | Criterion | Status | Evidence |
|---|---|---|---|
| C1 | Module boundaries are justified | PASS | [ARCHITECTURE §3–§4](ARCHITECTURE.md#4-proposed-crate-and-module-structure) |
| C2 | Dependency direction is clear | PASS | [ARCHITECTURE §5–§6](ARCHITECTURE.md#5-module-dependency-graph) |
| C3 | Architecture alternatives were evaluated | PASS | [ARCHITECTURE §3](ARCHITECTURE.md#3-alternatives-evaluated); the alternatives sections of each ADR |
| C4 | Important ADRs are documented | PASS | ADR-0001 to ADR-0011 exist ([index](adr/README.md)). ADR-0004, 0006, 0007, 0008, 0009, 0010 and 0011, and the proposed details of ADR-0001 (D2) and ADR-0002 (D2–D4), were **accepted on 2026-10-10**. ADR-0005 and the details of ADR-0003 (D2–D5) stay Proposed and need approval before M2 |
| C5 | The design respects the Standard Library-only constraint | PASS | [ADR-0002](adr/ADR-0002-standard-library-only.md) D3 (approach for each area); [ARCHITECTURE §11](ARCHITECTURE.md#11-build-configuration-policy-to-apply-from-m1); [TESTING_STRATEGY §2](TESTING_STRATEGY.md#2-test-levels-and-tooling) |

### Gate D — Financial correctness

| # | Criterion | Status | Evidence |
|---|---|---|---|
| D1 | Mathematical contracts are defined | PASS | [REQUIREMENTS §9–§11](REQUIREMENTS.md#9-fs-dcc--day-count-conventions) (formulas, domains, failure cases) |
| D2 | Edge cases are identified | PASS | Boundary conditions in each specification; [TESTING_STRATEGY §4](TESTING_STRATEGY.md#4-foundational-test-scenario-catalogue) (43 scenarios) |
| D3 | Exact and approximate validation are distinguished | PASS | [NUMERICAL_CORRECTNESS §7.3](NUMERICAL_CORRECTNESS.md#73-exactness-classes) (classes E0–E5) |
| D4 | Differential validation is planned | PASS | [ADR-0008](adr/ADR-0008-validation-strategy.md) (accepted); [TESTING_STRATEGY §7](TESTING_STRATEGY.md#7-differential-testing-against-quantlib). OQ-13 decided the encoding and the generator location |
| D5 | Numerical tolerance policies are justified | PASS | [NUMERICAL_CORRECTNESS §7](NUMERICAL_CORRECTNESS.md#7-tolerance-policy). Tolerances come from conditioning. The constants are calibrated in M4 |
| D6 | Reproducibility requirements are documented | PASS | NFR-03 and NFR-04; [NUMERICAL_CORRECTNESS §6](NUMERICAL_CORRECTNESS.md#6-determinism-and-cross-platform-reproducibility) (levels R1–R3); [ADR-0011](adr/ADR-0011-explicit-state.md) |

### Gate E — Open-source readiness

| # | Criterion | Status | Evidence |
|---|---|---|---|
| E1 | Licensing decisions are recorded or marked pending | PASS | `MIT OR Apache-2.0`, recorded in [ADR-0010](adr/ADR-0010-licensing.md) and [LICENSING_AND_ATTRIBUTION](LICENSING_AND_ATTRIBUTION.md). The licence files are added before the first code commit (OQ-02) |
| E2 | Attribution requirements were researched | PASS | [LICENSING_AND_ATTRIBUTION §3–§6](LICENSING_AND_ATTRIBUTION.md#3-quantlib-licence-review) |
| E3 | Contribution principles exist | PASS | [CONTRIBUTING_GUIDELINES](CONTRIBUTING_GUIDELINES.md) (adopted); root `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` and `SECURITY.md` |
| E4 | API stability principles exist | PASS | [API_DESIGN_PRINCIPLES](API_DESIGN_PRINCIPLES.md); [ADR-0009](adr/ADR-0009-api-stability.md) |
| E5 | Roadmap milestones have acceptance criteria | PASS | [ROADMAP §2–§4](ROADMAP.md#3-foundational-milestones-m0m5) |

### Gate F — Implementation authorization

| # | Criterion | Status | Evidence |
|---|---|---|---|
| F1 | The M0 documentation is complete | PASS | The owner reviewed and merged the M0 documentation (pull request #1). Cross-document review in §7 |
| F2 | Mandatory blocking issues are resolved | PASS | OQ-01 to OQ-08 were decided on 2026-10-10 (§5) |
| F3 | Required decisions are approved | PASS | The ADRs that M1 needs were accepted on 2026-10-10 (C4) |
| F4 | The project owner has explicitly authorized implementation | PASS | On 2026-10-10 the owner accepted every recommendation needed for M1 and explicitly asked for M1 to start. The authorization covers **M1 only**. Each later milestone needs its own readiness check and authorization ([ROADMAP](ROADMAP.md)) |

## 4. Verdict

**SabiQuant is ready to begin M1, and the project owner has authorized it.**
On 2026-10-10 the owner accepted the recommendations for every decision that
blocks M1 (§5) and asked for M1 to start. The authorization covers M1 only:

1. M1 is carried out as specified in the [ROADMAP](ROADMAP.md#m1--core-foundations).
2. The decisions that block M2 (OQ-09, OQ-10, OQ-12, ADR-0005 Part A, and
   ADR-0003 D2–D5) are taken before M2 starts, together with a readiness
   check for M2.
3. Every later milestone likewise needs its own readiness check and the
   owner's explicit permission.

The first M0 review on the same day reached the opposite verdict, because the
decisions above were still open (assessment history, at the top).

## 5. Decision register

Every decision, with its recommendation and status. "Blocks" is the earliest
milestone that cannot start without the decision. "Decided" means the owner
accepted the recommendation on the date shown, unless the status says
otherwise.

| ID | Decision | Recommendation | Main alternative(s) | Blocks | Record | Status |
|---|---|---|---|---|---|---|
| OQ-01 | Crate topology | One library crate `sabiquant` | Workspace of crates per domain | **M1** | [ADR-0004](adr/ADR-0004-project-structure.md) | **Decided** 2026-10-10 |
| OQ-02 | Project licence | `MIT OR Apache-2.0` | Apache-2.0; MIT; BSD-3-Clause | **M1** (before the first code commit) | [ADR-0010](adr/ADR-0010-licensing.md) | **Decided** 2026-10-10 |
| OQ-03 | Copyright line; DCO sign-off; code of conduct | "The SabiQuant Contributors"; adopt DCO; adopt an established code of conduct | The owner's name; no DCO | **M1** (before external contributions) | [ADR-0010](adr/ADR-0010-licensing.md), [CONTRIBUTING_GUIDELINES §9](CONTRIBUTING_GUIDELINES.md#9-governance-model) | **Decided** 2026-10-10. Code of conduct: Contributor Covenant 3.0 |
| OQ-04 | Edition and MSRV policy | Edition 2024; MSRV = the stable release current at M1, raised only in minor releases | Track latest stable only; an older MSRV | **M1** | [ADR-0009](adr/ADR-0009-api-stability.md) | **Decided** 2026-10-10. MSRV: Rust 1.99 |
| OQ-05 | Error model | Per-domain non-exhaustive errors plus a crate error; no panics on input | One crate-wide enum; opaque error struct | **M1** | [ADR-0007](adr/ADR-0007-error-handling.md) | **Decided** 2026-10-10 |
| OQ-06 | Numeric representation | Concrete `f64`; decimal amounts and AAD deferred (NI-01, NI-02) | Generic numeric layer now | **M1** | [ADR-0006](adr/ADR-0006-numeric-policy.md) | **Decided** 2026-10-10 |
| OQ-07 | Explicit-state policy | No global mutable state; no implicit today | Global settings like QuantLib's | **M1** | [ADR-0011](adr/ADR-0011-explicit-state.md) | **Decided** 2026-10-10 |
| OQ-08 | Platform tiers | Primary: x86_64 Linux, aarch64 macOS, x86_64 Windows | Add 32-bit or other targets to the primary tier | **M1** (CI) | [ARCHITECTURE §3.15](ARCHITECTURE.md#315-cross-platform-behavior), [TESTING_STRATEGY §11](TESTING_STRATEGY.md#11-cross-platform-testing) | **Decided** 2026-10-10 |
| OQ-09 | Supported date range | 0001-01-01 to 9999-12-31 | 1901–2199 (QuantLib); full 32-bit range | M2 | [ADR-0005](adr/ADR-0005-date-representation.md) | Decided under delegated authority 2026-10-11 ([M2_READINESS](M2_READINESS.md)) |
| OQ-10 | Period equality and ordering | Normalized equality; partial order within a normalized unit family; zero belongs to Days | Structural equality; QuantLib's bound-based comparisons | M2 | [REQUIREMENTS FR-PER-02](REQUIREMENTS.md#6-fs-per--periods-and-frequencies) | Decided under delegated authority 2026-10-11 ([M2_READINESS](M2_READINESS.md)) |
| OQ-11 | Operators for fallible date arithmetic | Checked methods only in the first release | Panicking or saturating operators | M2 | [ADR-0007](adr/ADR-0007-error-handling.md) | **Decided** 2026-10-10, by accepting ADR-0007 |
| OQ-12 | Excel/QuantLib serial interop | Deferred; an explicit, separate conversion if ever added | Include in M2 | M2 (scope) | [ADR-0005](adr/ADR-0005-date-representation.md) | Decided under delegated authority 2026-10-11 ([M2_READINESS](M2_READINESS.md)) |
| OQ-13 | Fixture encoding and generator location | Text header plus tab-separated records; generator in `validation/`, excluded from the package | JSON; separate repository | M1 (fixture reader), M2 | [ADR-0008](adr/ADR-0008-validation-strategy.md) | **Decided** 2026-10-10, with ADR-0008 |
| OQ-14 | Year fraction for reversed dates | Define as `−YF(E, S)` and document the divergence for 30/360 | Reject with an error; follow QuantLib for each convention | M3 | [REQUIREMENTS §9.1](REQUIREMENTS.md#91-general-specification) | Decided for local work under delegated authority 2026-10-11 ([M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)) |
| OQ-15 | 30/360 US February and end-of-month rules | Follow the primary SIFMA source. If the rules are end-of-month-qualified, model an explicit flag (QuantLib = flag on) | Copy QuantLib's unconditional rules | M3 | [REQUIREMENTS §9.7](REQUIREMENTS.md#97-the-30360-family) | Locally decided: explicit security EOM flag; dated ROC December 2025 A001 rules. Full SIFMA book unread; financial acceptance open ([contract](M3_US_AFB_CONTRACT.md)) |
| OQ-16 | Actual/Actual AFB counting-back rule | Decide from the primary source; test case 2024-02-28 → 2025-02-28 | Adopt QuantLib's rule | M3 | [REQUIREMENTS §9.6](REQUIREMENTS.md#96-actualactual-afb) | Locally decided from directly inspected ISDA 1999 §2(c), including leap-February promotion; original 1994 AFB publication unread ([contract](M3_US_AFB_CONTRACT.md)) |
| OQ-17 | Holiday-aware pilot calendar | TARGET only, in M3 | No market calendar in M3; several calendars | M3 | [ADR-0005](adr/ADR-0005-date-representation.md) | Decided for local work under delegated authority 2026-10-11 ([M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)) |
| OQ-18 | Behavior outside verified calendar coverage | Strict verified coverage; explicit projection with queryable status | Implicit extrapolation | M3 | [ADR-0005](adr/ADR-0005-date-representation.md) | Decided for local work under delegated authority 2026-10-11 ([M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)) |
| OQ-19 | Bound on business-day searches | 366 consecutive non-business days, then an error | Search to the edge of the range | M3 | [REQUIREMENTS §7](REQUIREMENTS.md#7-fs-cal--calendars) | Decided for local work under delegated authority 2026-10-11 ([M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)) |
| OQ-20 | Day-count variants in M3 | As in FEATURE_MATRIX §5 (P0 and P1 only) | Include the P2/P3 variants | M3 | [FEATURE_MATRIX §5](FEATURE_MATRIX.md#5-day-count-conventions) | Decided for local work under delegated authority 2026-10-11 ([M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)) |
| OQ-21 | Inputs for Actual/Actual ICMA | Explicit determination period and frequency; no inference | QuantLib-style inference | M3 | [REQUIREMENTS §9.5](REQUIREMENTS.md#95-actualactual-icma) | Decided for local work under delegated authority 2026-10-11 ([M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)) |
| OQ-22 | Domain policy for compound factors | Finite inputs; `t ≥ 0`; strictly positive and finite outputs; rounded active-domain and conservative range-boundary checks | Return NaN or ∞ as QuantLib does | M4 | [REQUIREMENTS §10](REQUIREMENTS.md#10-fs-rate--interest-rates-and-compounding) | Decided for local work under delegated authority 2026-10-11 ([M4_LOCAL_READINESS](M4_LOCAL_READINESS.md)) |
| OQ-23 | Hybrid compoundings in M4 | Include both (P1) | Defer | M4 | [FEATURE_MATRIX §6](FEATURE_MATRIX.md#6-interest-rates-compounding-and-discounting) | Decided for local work under delegated authority 2026-10-11 ([M4_LOCAL_READINESS](M4_LOCAL_READINESS.md)) |
| OQ-24 | Scalar wrapper types | None in the first release; revisit after the M4 API review | Wrappers for year fraction and discount factor | M4 | [DOMAIN_MODEL §6.4](DOMAIN_MODEL.md#64-scalar-wrapper-types-oq-24) | Decided for local work under delegated authority 2026-10-11 ([M4_LOCAL_READINESS](M4_LOCAL_READINESS.md)) |
| OQ-25 | Publication on crates.io and the name | Publish 0.1.0 only at M5 with approval; no placeholder | Reserve the name early | M5 | [ADR-0009](adr/ADR-0009-api-stability.md) | **Decided** 2026-10-10, by accepting ADR-0009. Publishing itself still needs the owner's approval at M5 |
| OQ-26 | Security reporting channel | GitHub private vulnerability reporting, enabled at M1 | Email contact | M5 (first release) | [CONTRIBUTING_GUIDELINES §10](CONTRIBUTING_GUIDELINES.md#10-security-reporting) | **Decided** 2026-10-10. Enabling it in the repository settings is an owner action |

Decided from the start (owner constraints): native Rust ([ADR-0001](adr/ADR-0001-native-rust.md));
standard library only ([ADR-0002](adr/ADR-0002-standard-library-only.md));
behavioral and numerical, not API, compatibility ([ADR-0003](adr/ADR-0003-quantlib-compatibility.md)).

## 6. Risk register

Impact and likelihood: H (high), M (medium), L (low).

| ID | Area | Risk | Impact | Likelihood | Mitigation | Milestone |
|---|---|---|---|---|---|---|
| R-01 | Financial correctness | Ambiguous conventions (30/360 US, AFB, ISDA cases) lead to wrong results | H | M | Primary sources; exact variant names; independent oracle as well as QuantLib; OQ-14 to OQ-16 and OQ-21 | M3 |
| R-02 | Financial correctness | Primary sources are not accessible: ISDA Definitions are licensed; the ISDA 1999 paper is a scanned image | H | M | Obtain licensed access or expert review; transcribe by hand; keep features Under Research until sourced | M3 |
| R-03 | Calendars | Holiday-data errors and maintenance burden | H | M | One pilot calendar; provenance; verified coverage; annual review | M3+ |
| R-04 | Numerics | Platform `libm` differences cause test failures | M | H | Tolerance classes; expected values never specific to a platform; R1–R3 levels | M4 |
| R-05 | Numerics | False discrepancies against QuantLib's naive formulas | M | H | High-precision oracle; D5 and D8 classification | M4 |
| R-06 | Dependency restriction | Defects in the large in-house numerical surface | H | M | Published algorithms; oracle tests; phased milestones; ADR-0002 triggers | M6+ |
| R-07 | Architecture scalability | Explicit state is unwieldy or slow for complex market data | M | M | Valuation-context ADR at M9; structural sharing; measure first | M9+ |
| R-08 | Architecture | Concrete `f64` blocks AAD later | M | M | Decide NI-02 before M10/M11 | M10 |
| R-09 | QuantLib parity | Baseline drift, deprecations, behavior changes | M | H | Pinned 1.43; re-baseline only at milestone boundaries | All |
| R-10 | QuantLib parity | Compatibility is over-claimed | H | L | Claims policy (ADR-0003 D5); review | All |
| R-11 | Maintenance | Solo-maintainer capacity and bus factor | H | M | Documentation, ADRs, scope gating, governance that lets maintainers be added | All |
| R-12 | Licensing | Accidental copying from QuantLib. (The other half of this risk, having no licence, was closed on 2026-10-10 when `MIT OR Apache-2.0` was selected) | H | M | Provenance attestation in every pull request; review | M1 |
| R-13 | Scope | Unbounded parity ambition | M | M | Milestone gating; scope change control | All |
| R-14 | Naming | Someone else claims the crate name before M5 | L | L | Owner decision (OQ-25) | M5 |
| R-15 | Toolchain | MSRV and toolchain churn | L | M | MSRV policy (ADR-0009) | All |
| R-16 | Validation | QuantLib-Python 1.43 wheels become unavailable | M | L | Record wheel hashes; fall back to a C++ build | M2+ |

## 7. Cross-document review (M0 Step 9)

| Check | Method | Result |
|---|---|---|
| Internal links and anchors | Automated: every relative link and `#anchor` checked against GitHub-style heading slugs in all 27 files | 404 relative links checked; **0 broken** |
| Identifier consistency | Automated: every referenced `OQ-`, `R-`, `TS-`, `F-`, `FR-`, `NFR-`, `INV-`, `C-`, `A-`, `QR-`, `L-`, `B-`, `NP-`, `NI-`, `TOL-` and `ADR-` identifier exists where it is defined | **0 undefined references.** Counts: 26 OQ, 16 R, 43 TS, 113 F, 43 FR, 15 NFR, 11 ADRs |
| No executable code | Automated: the only fenced code blocks are Mermaid; no Rust or other source code; no Cargo or `src/` files | 6 Mermaid diagrams, **no other code blocks**; the repository holds only Markdown |
| Factual corrections during review | Manual re-verification against the QuantLib 1.43 source and the independent oracle | Fixed: the day-count invariant table (the anniversary and monotonicity claims were narrowed after finding the AFB leap-year edge case); the scenario count (43); the description of the day-counter headers; the wording of the module-name conflict |
| Constraint compliance | Manual: no dependency proposed without an ADR-0002 exception; no global state; no API compatibility claim | Compliant |
| Unsupported claims | Manual: QuantLib behavior is labelled "observed in source, to be confirmed"; no performance claims; sources cited | Compliant; limitations L-01 to L-10 are listed |
| Terminology | Manual: glossary terms are used consistently (for example "determination period" alongside QuantLib's "reference period") | Consistent |
| Unbounded scope | Manual: every future domain is provisional, with dependencies and no dates | Bounded |
| Unresolved items | Listed in §5 (26 decisions) and §6 (16 risks) | Explicit |

## 8. Next recommended action

**Carry out M2** under [M2_READINESS](M2_READINESS.md). M1 has merged with
passing CI ([evidence](M2_READINESS.md#prerequisite-evidence)). Repository
administration remains for the owner to verify:

1. enables GitHub private vulnerability reporting for the repository (OQ-26);
2. makes the CI checks required for merging pull requests.

The owner named the private channel for code-of-conduct reports on
2026-10-10; it is in `CODE_OF_CONDUCT.md`.

The M2 decisions are now recorded under the owner's delegated authority.
The historical M1 assessment and its dates are preserved above.
