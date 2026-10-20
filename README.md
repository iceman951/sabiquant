# SabiQuant

**SabiQuant** is a planned, independent quantitative finance library written
natively in Rust. Over time it aims to progressively reach functional parity
with the major domains of [QuantLib](https://github.com/lballabio/QuantLib),
with an idiomatic Rust API, financially correct behavior, and rigorously
validated numerics.

> **Project status: M1 is merged; M2–M6 foundations, M7 schedules/cash-flow analytics, and M8 exact amounts/bond core are implemented locally.**
> M3 includes immutable calendars, guarded TARGET coverage, Easter and ten
> day counts; M4 adds explicit quotes, compounding, factors and single-amount
> present values. M5 includes API/package review fixes; M6 adds checked solvers,
> four interpolators, normal functions and compensated/exact summation.
> M7 currently supports anchored Forward/Backward, explicit first/penultimate
> coupon dates, Zero schedules, four explicit civil month rolls and generated
> contractual ICMA reference contexts for both date/EOM and civil-roll lattices,
> plus simple cash flows, plain fixed coupons, legs, flat NPV and outstanding
> accrued amounts with explicit inclusion/work budgets. Modern CDS work adds
> explicit quarterly/semiannual quoted-maturity resolution and a Following
> payment/accrual date plan with distinct date roles. The automatic local
> OldCDS rule adds twentieth schedules on quarterly anchors with a thirty-day
> first-quarter cutoff, provenance and generated ICMA references.
> US 30/360 now requires explicit security EOM qualification; AFB follows
> ISDA's published backward-year clarification. Both compose with rates and
> cash flows ([contract](https://github.com/iceman951/sabiquant/blob/main/docs/M3_US_AFB_CONTRACT.md)). Full SIFMA source acceptance,
> named settlement conventions and further pricing remain pending. This is an unreleased
> development version awaiting milestone review.

The local M8 amount foundation adds exact canonical decimals with explicit
arbitrary quanta, seven rounding modes, cancellation-safe exact sums and fused
product/divisor quantization. It is specified in
[M8_AMOUNT_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M8_AMOUNT_CONTRACT.md); its local engineering
validation is recorded in [M8_AMOUNT_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M8_AMOUNT_VALIDATION.md),
with owner, financial and platform gates still open. Phase B adds four named
binary64 conversions: exact or explicitly rounded incoming conversion of a
float's exact bit value, and exact or correctly rounded outgoing conversion
([contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BINARY_BOUNDARY_CONTRACT.md),
[validation](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BINARY_BOUNDARY_VALIDATION.md)). The M8 bond core tranche
adds `sabiquant::bonds` with zero-coupon and fixed-rate bullet bonds, explicit
settlement and ex-coupon entitlement, earned-minus-detached accrued interest,
exact single-rounding per-100 quote conversion, and two-stage direct-horizon
`BondPricingPlan` / `BondValuation`
([contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_CORE_CONTRACT.md),
[validation](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_CORE_VALIDATION.md)), plus generic prepared-plan yield-to-maturity
inversion (`BondPricingPlan::yield_from_price`, `BondYieldReport`,
`BondYieldSensitivity`) and all six analytical risk views (`BondPricingPlan::risk`,
`BondRisk`: first/second price derivatives, time-weighted amount, modified/Macaulay
duration and convexity)
([contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_YIELD_RISK_CONTRACT.md),
[validation](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_YIELD_RISK_VALIDATION.md)). Named market yield conventions remain tracked in
[M8_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M8_LOCAL_READINESS.md).

The conventional UK gilt model adds `ConventionalGilt` and `GiltPricingPlan`
with complete caller-supplied semiannual quasi dates, explicit ex-dividend
cutoffs, source-specific first-dividend publication and accrued interest.
It includes native per100 pricing, bounded yield inversion, all six retained
risk views and exact dividend/accrued/dirty/clean-plus-accrued cash recipes
([contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_GILT_CONTRACT.md),
[validation](https://github.com/iceman951/sabiquant/blob/main/docs/M8_GILT_VALIDATION.md)).
Published first dividends and source accrued interest keep their distinct
exact rational meanings. Observed prices have conditional numerical budgets;
they do not provide analytical enclosures or market acceptance.

The local `settlement` module adds four exact monetary recipes with explicit
per100/per1000 units, positive par nominal, source/cash rounding stages and
signed quantum policies. Supplied decimal/binary values retain their exact
meaning; declared real intervals yield a determined receipt, ambiguous cash
extremes or an unresolved Exact stage. A receipt keeps the complete original
instruction and conditional provenance
([contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_SETTLEMENT_CONTRACT.md),
[validation](https://github.com/iceman951/sabiquant/blob/main/docs/M8_SETTLEMENT_VALIDATION.md)).
Financial enclosure producers, named market conventions and full M8 acceptance
remain tracked in the readiness record.

## What SabiQuant aims to become

A reusable, long-lived Rust library for quantitative finance. It starts from
rigorously specified foundations:

- dates, periods and frequencies;
- calendars and business-day conventions;
- day-count conventions;
- interest rates, compounding, and discounting.

From there it grows towards schedules, cash flows, bonds, term structures,
derivatives, pricing engines, stochastic processes, numerical methods, and
risk analytics. See the [vision](https://github.com/iceman951/sabiquant/blob/main/docs/VISION.md) and the
[roadmap](https://github.com/iceman951/sabiquant/blob/main/docs/ROADMAP.md). Functional parity with QuantLib is a long-term
ambition, not a promise. No compatibility is claimed until it is verified
independently ([ADR-0003](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0003-quantlib-compatibility.md)).

## Principles

- **Rust-native.** Designed around ownership, value semantics, enums, traits,
  and explicit `Result`-based errors. Not a binding to QuantLib and not a
  translation of it ([ADR-0001](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0001-native-rust.md)).
- **Standard Library only.** The current dependency policy allows no
  third-party crates of any kind: runtime, dev, build, or procedural macro
  ([ADR-0002](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0002-standard-library-only.md)).
- **QuantLib is a reference, not a dependency.** QuantLib is used to study
  behavior and to compare numerical results. It is never needed to build, run,
  or test SabiQuant ([ADR-0008](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0008-validation-strategy.md)).
- **Correctness before performance.** The order is correctness, then
  reliability, then maintainability, then performance
  ([numerical policy](https://github.com/iceman951/sabiquant/blob/main/docs/NUMERICAL_CORRECTNESS.md)).
- **Explicit state.** No global evaluation date, no singletons, no implicit
  "today" ([ADR-0011](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0011-explicit-state.md)).

## Decisions

The native-Rust, Standard-Library-only, and behavioral-compatibility
principles are constraints set by the project owner. On 2026-10-10 the owner
accepted the architecture decisions that M1 needs and authorized M1
([ADR index](https://github.com/iceman951/sabiquant/blob/main/docs/adr/README.md),
[decision register](https://github.com/iceman951/sabiquant/blob/main/docs/IMPLEMENTATION_READINESS.md#5-decision-register)).
Later decisions are recorded as each locally authorized batch is prepared.

On 2026-10-11 the owner delegated the next implementation choices and asked
for continuous progress with broad tests. M2 decisions and its authorization
are recorded in [M2_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M2_READINESS.md); validation evidence and
remaining acceptance gates are in [M2_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M2_VALIDATION.md).
The next local batch is recorded in [M3_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M3_LOCAL_READINESS.md)
and [M3_CORE_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M3_CORE_VALIDATION.md). Local M4 decisions and
numerical evidence are in [M4_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M4_LOCAL_READINESS.md)
and [M4_NUMERICAL_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M4_NUMERICAL_VALIDATION.md).
Local M5 corrections and M6 numerical evidence are recorded in
[M5_LOCAL_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M5_LOCAL_VALIDATION.md),
[M6_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M6_LOCAL_READINESS.md) and
[M6_NUMERICAL_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M6_NUMERICAL_VALIDATION.md). External writes are not
authorized: the owner selected continued work in the local repository.
The restricted M7 schedule tranche and its classified reference distinctions
are recorded in [M7_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M7_LOCAL_READINESS.md)
and [M7_CORE_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CORE_VALIDATION.md).
The explicit coupon extension is specified in
[M7_EXPLICIT_STUB_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_EXPLICIT_STUB_CONTRACT.md)
with evidence in [M7_EXPLICIT_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_EXPLICIT_VALIDATION.md).
Generated references use the same named lattice and an explicit matching
frequency, specified in [M7_ICMA_REFERENCE_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_ICMA_REFERENCE_CONTRACT.md).
The measured reference/accrual checks are recorded in
[M7_ICMA_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_ICMA_VALIDATION.md).
Analytical cash-flow definitions are in
[M7_CASHFLOW_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CASHFLOW_CONTRACT.md).
They retain independent coupon/discount conventions, signed binary64 amounts,
explicit payment inclusion and per-call contextual work. Exact monetary recipes
are separate in `settlement` under the local NI-01 extension. The shared immutable coupon context and one-round integer product avoid
quadratic context copying and recoverable intermediate product range failures.
Local arithmetic, financial and package evidence is tracked in
[M7_CASHFLOW_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CASHFLOW_VALIDATION.md).
Four civil roll definitions, requested/resolved provenance and their explicit
regularity policy are specified in
[M7_ROLL_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_ROLL_CONTRACT.md).
Local results and explicit reference differences are recorded in
[M7_ROLL_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_ROLL_VALIDATION.md).
Modern CDS date operations follow
[M7_CDS_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CDS_CONTRACT.md),
with independent maturity evidence in
[M7_CDS_MATURITY_ORACLES](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CDS_MATURITY_ORACLES.md)
and integrated checks tracked in
[M7_CDS_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CDS_VALIDATION.md).
The automatic OldCDS rule is a local engineering definition in
[M7_OLD_CDS_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_OLD_CDS_CONTRACT.md), with classified reference
observations in [M7_OLD_CDS_REFERENCE_PROBES](https://github.com/iceman951/sabiquant/blob/main/docs/M7_OLD_CDS_REFERENCE_PROBES.md)
and integrated checks in [M7_OLD_CDS_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_OLD_CDS_VALIDATION.md).
Its historical financial cutoff and start-date role remain unresolved in the
[legacy source review](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CDS_LEGACY_SOURCE_REVIEW.md).

## Documentation

| Document | Purpose |
|---|---|
| [VISION](https://github.com/iceman951/sabiquant/blob/main/docs/VISION.md) | Mission, users, priorities, landscape |
| [SCOPE](https://github.com/iceman951/sabiquant/blob/main/docs/SCOPE.md) | Scope horizons, non-goals, scope control |
| [REQUIREMENTS](https://github.com/iceman951/sabiquant/blob/main/docs/REQUIREMENTS.md) | Constraints, non-functional requirements, functional specifications |
| [QUANTLIB_REFERENCE](https://github.com/iceman951/sabiquant/blob/main/docs/QUANTLIB_REFERENCE.md) | Pinned QuantLib 1.43 baseline and architecture analysis |
| [FEATURE_MATRIX](https://github.com/iceman951/sabiquant/blob/main/docs/FEATURE_MATRIX.md) | Feature inventory with status, priority, milestone, and validation approach |
| [ARCHITECTURE](https://github.com/iceman951/sabiquant/blob/main/docs/ARCHITECTURE.md) | Crate and module architecture |
| [DOMAIN_MODEL](https://github.com/iceman951/sabiquant/blob/main/docs/DOMAIN_MODEL.md) | Domain concepts, glossary, typing strategy |
| [API_DESIGN_PRINCIPLES](https://github.com/iceman951/sabiquant/blob/main/docs/API_DESIGN_PRINCIPLES.md) | Public API rules |
| [NUMERICAL_CORRECTNESS](https://github.com/iceman951/sabiquant/blob/main/docs/NUMERICAL_CORRECTNESS.md) | Floating-point policy, tolerances, invariants |
| [TESTING_STRATEGY](https://github.com/iceman951/sabiquant/blob/main/docs/TESTING_STRATEGY.md) | Testing with the standard library only, and differential validation |
| [ROADMAP](https://github.com/iceman951/sabiquant/blob/main/docs/ROADMAP.md) | Milestones M0–M5 in detail, then provisional M6–M16 |
| [CONTRIBUTING_GUIDELINES](https://github.com/iceman951/sabiquant/blob/main/docs/CONTRIBUTING_GUIDELINES.md) | Contribution, review, release and governance |
| [LICENSING_AND_ATTRIBUTION](https://github.com/iceman951/sabiquant/blob/main/docs/LICENSING_AND_ATTRIBUTION.md) | Licence analysis and attribution policy |
| [IMPLEMENTATION_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/IMPLEMENTATION_READINESS.md) | Readiness gates, decision register, risk register |
| [M2_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M2_READINESS.md) | M2 authorization, delegated decisions and prerequisite CI evidence |
| [M2_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M2_VALIDATION.md) | M2 coverage, test evidence and remaining acceptance gates |
| [M3_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M3_LOCAL_READINESS.md) | Staged local M3 scope and delegated engineering decisions |
| [M3_CORE_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M3_CORE_VALIDATION.md) | Core calendar/day-count evidence |
| [M3_EXTENSION_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M3_EXTENSION_VALIDATION.md) | Easter/TARGET and contextual day-count evidence and remaining gates |
| [M4_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M4_LOCAL_READINESS.md) | Delegated rate-domain, compounding and numerical decisions |
| [M4_NUMERICAL_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M4_NUMERICAL_VALIDATION.md) | Independent high-precision rate evidence and conditioning analysis |
| [M4_REFERENCE_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M4_REFERENCE_VALIDATION.md) | Attested public QuantLib comparisons and classified differences |
| [M4_DATE_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M4_DATE_CONTRACT.md) | Owned day-count contexts, date conversion and valuation contracts |
| [M4_BOUNDARY_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M4_BOUNDARY_VALIDATION.md) | Conservative numerical range policy and exact boundary anchors |
| [M5_LOCAL_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M5_LOCAL_VALIDATION.md) | API and extracted-package correction evidence |
| [M6_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M6_LOCAL_READINESS.md) | Delegated numerical algorithms and domain contracts |
| [M6_NUMERICAL_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M6_NUMERICAL_VALIDATION.md) | Independent numerical oracles, native checks and remaining gates |
| [M6_LOCAL_NUMERICAL_REVIEW](https://github.com/iceman951/sabiquant/blob/main/docs/M6_LOCAL_NUMERICAL_REVIEW.md) | Concrete arithmetic findings, corrections and independent local review |
| [M7_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M7_LOCAL_READINESS.md) | Adopted core schedule contracts and scope |
| [M7_CORE_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CORE_VALIDATION.md) | Independent civil-date, guarded adjustment and reference evidence |
| [M7_EXPLICIT_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_EXPLICIT_VALIDATION.md) | Explicit coupon lattice, independent oracle and local validation |
| [M7_CDS_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CDS_CONTRACT.md) | Explicit modern quoted-maturity resolver and Following coupon-date roles |
| [M7_CDS_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CDS_VALIDATION.md) | Modern CDS date evidence and integrated checks (tranche snapshot) |
| [M7_OLD_CDS_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M7_OLD_CDS_CONTRACT.md) | Automatic local OldCDS cutoff, terminal, provenance and ICMA contract |
| [M7_OLD_CDS_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M7_OLD_CDS_VALIDATION.md) | OldCDS oracle, reference-classification and integrated evidence |
| [M8_LOCAL_READINESS](https://github.com/iceman951/sabiquant/blob/main/docs/M8_LOCAL_READINESS.md) | M8 primary-source audit, NI-01 settlement boundary and staged scope |
| [M8_AMOUNT_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M8_AMOUNT_CONTRACT.md) | Canonical decimal amounts, explicit quanta, rounding modes and fused ratio arithmetic |
| [M8_BINARY_BOUNDARY_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BINARY_BOUNDARY_CONTRACT.md) | Exact and explicitly rounded binary64 boundary conversions for decimal amounts |
| [M8_BOND_CORE_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_CORE_CONTRACT.md) | Zero-coupon and fixed-rate bullet bonds, settlement, ex-coupon, accrual, quotes and horizon pricing |
| [M8_BOND_CORE_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_CORE_VALIDATION.md) | Independent bond core oracle, hand suite, coverage and package validation |
| [M8_BOND_YIELD_RISK_CONTRACT](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_YIELD_RISK_CONTRACT.md) | Generic prepared-plan yield-to-maturity inversion, sensitivity classification and all six analytical risk views |
| [M8_BOND_YIELD_RISK_VALIDATION](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_YIELD_RISK_VALIDATION.md) | Independent bond yield/risk oracle, hand suite, coverage and package validation |
| [ADRs](https://github.com/iceman951/sabiquant/blob/main/docs/adr/README.md) | Architecture Decision Records ADR-0001 to ADR-0011 |

## Building and testing

SabiQuant needs Rust 1.99 or later and nothing else: it has no dependencies.

The crate package includes library sources, standard library unit-test helpers
and the Clippy policy configuration. Integration suites, research documents and
external validation fixtures stay in the repository. The links below point
to its `main` branch; unreleased local changes must reach the remote repository
before publication.

```sh
cargo test                                  # unit, integration and documentation tests
cargo clippy --all-targets -- -D warnings   # lints, as run in CI
cargo fmt --all --check                     # formatting
cargo doc --no-deps --open                  # API documentation
```

`sabiquant::error` provides typed errors and `sabiquant::time` provides checked
Gregorian dates, tenors, weekdays, months and frequencies.
`sabiquant::calendar` provides immutable core calendars and checked business-day
operations, Gregorian Easter and TARGET with strict source coverage by default.
`sabiquant::daycount` provides five stateless conventions plus explicit-period
ICMA, termination-sensitive 30E/360 ISDA and explicit-calendar Business/252.
`sabiquant::rates` provides five compounding methods, explicit frequency/context
quotes, compound and discount factors, implied/equivalent rates, single-amount
present values and named percent/basis-point conversions.
`sabiquant::math` provides bisection, Brent and safeguarded Newton with shared
callback budgets; immutable linear/log-linear/natural-cubic/PCHIP interpolation
with explicit extrapolation; standard and location/scale normal operations;
and checked compensated or exactly rounded summation. Positional solver
tolerances, supported cubic grids and empirical elementary-function accuracy
are explicit contracts, not universal precision guarantees.
`sabiquant::schedule` retains contractual/adjusted boundaries,
automatic short/long stubs, validated explicit coupon roles, calendar EOM and
owned TARGET guards. Explicit construction requires an exact named regular
middle and retains outer stub provenance. ICMA generation builds minimal
same-anchor determination periods with checked
required extensions, full TARGET hull validation and explicit frequency.
These references retain contractual accrual independently of adjusted payment
dates. Civil Wednesday/twentieth rolls retain original and projected ranges,
a month-coordinate lattice and genuine consecutive-roll regularity. Their
generated ICMA references use the same Wednesday/twentieth points and minimal
resolved raw hull with an explicit matching frequency; see the
[roll ICMA contract](https://github.com/iceman951/sabiquant/blob/main/docs/M7_ROLL_ICMA_CONTRACT.md).
Schedule adjustment rejects collisions instead of discarding boundaries.
`resolve_cds_maturity` preserves the supplied trade date, stored quoted tenor
and explicit `Quarterly2009`/`Semiannual2015` regime, returning a date or typed
Matured outcome. `CdsDatePlan::standard_following` takes a separately supplied
quarter-twentieth maturity and selects payments strictly after trade plus one
calendar day. It retains raw quarterly boundaries, Following payments,
adjusted ordinary accrual and included raw final maturity, with bounded coupons
and full retained TARGET guards. Its cutoff is for payment selection; legal
protection, accrued rebates, premium-leg pricing and settlement remain separate
work. `build_old_cds` takes a positive stored Months/Years stride, retains the
generation start, omits a first quarter fewer than thirty calendar days away
unless it is the terminal, stops interiors at the requested end and projects
the terminal to the next quarterly twentieth. Its provenance, exact lattice
flags, count caps and original/expanded TARGET guards are explicit; it is not
a historical CDS premium leg. All ten date-rule labels are constructible.
`sabiquant::amounts` provides canonical signed decimal amounts with explicit
quanta, seven rounding modes, exact sums, fused ratio quantization and four
named binary64 boundary conversions. `sabiquant::bonds` provides zero-coupon and
fixed-rate bullet bonds, explicit settlement and ex-coupon entitlement,
earned-minus-detached accrued interest, single-rounding per-100 par quote
conversions, two-stage direct-horizon `BondPricingPlan` / `BondValuation`,
bracketed yield-to-maturity inversion (`yield_from_price`, `BondYieldReport`,
`BondYieldSensitivity`), and all six analytical risk views (`risk`, `BondRisk`).
Tests read committed fixtures offline; regenerating them is documented in
[validation](https://github.com/iceman951/sabiquant/blob/main/validation/README.md).

## Relationship to QuantLib

SabiQuant uses [QuantLib](https://www.quantlib.org/) (BSD 3-Clause) as an
external behavioral and numerical reference. The current baseline is QuantLib
1.43. SabiQuant contains no QuantLib code. **SabiQuant is not affiliated with
or endorsed by the QuantLib project.**

## Licence

SabiQuant is licensed under either of

- the [Apache License, Version 2.0](https://github.com/iceman951/sabiquant/blob/main/LICENSE-APACHE), or
- the [MIT license](https://github.com/iceman951/sabiquant/blob/main/LICENSE-MIT),

at your option ([ADR-0010](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0010-licensing.md)).

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in SabiQuant by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](https://github.com/iceman951/sabiquant/blob/main/CONTRIBUTING.md) first: it
covers the Developer Certificate of Origin sign-off, the provenance rules, and
the rules for code. Everyone who takes part follows the
[Code of Conduct](https://github.com/iceman951/sabiquant/blob/main/CODE_OF_CONDUCT.md). Report security problems privately as
described in [SECURITY.md](https://github.com/iceman951/sabiquant/blob/main/SECURITY.md).
