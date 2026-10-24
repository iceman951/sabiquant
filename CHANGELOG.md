# Changelog

All notable changes to SabiQuant are documented in this file.

The format follows [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
with one extra section, **Behavior changes**, which lists every change in
numerical or financial output, even when the API does not change
([ADR-0009](docs/adr/ADR-0009-api-stability.md)). The project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Nothing has been released yet. The first release, 0.1.0, is planned for
milestone M5 and needs the project owner's approval.

## [Unreleased]

### Added

- Local Canadian fixed-coupon model: `CanadianFrequency`,
  `CanadianPeriodKind`, `CanadianSourcePolicy`, `CanadianYieldConvention`,
  `CanadianYieldSelection`, `CanadianPriceBasis`, `CanadianPrice`,
  `CanadianCouponSchedule`, `CanadianBond`, `CanadianPricingPlan`,
  `CanadianValuation`, `CanadianRisk`, `CanadianYieldReport`, `CanadianError`,
  `CanadianYieldError`, `CanadianCashInstruction`, `CanadianCashDecision` and
  `CanadianCashReceipt`. Explicit f2/4/12 quasi and coupon dates own regular,
  short, long and composite periods; exact canonical valuation and payable
  sources keep the late-period and monthly accrued rules separate. Plans
  resolve compound, money-market or last-coupon routing once, observe direct
  IEEE source present values, three price bases, all six independently
  admitted risk views and counted bounded yield inversion. Four exact cash
  recipes preserve source geometry, signed rounding, continuous Exact
  semantics and complete provenance. Eighteen types bring the public catalog
  to144;33 Canadian and5 yield errors bring the domain catalog to237. The
  complete independent pre-production gate precedes implementation; final
  local verification is recorded separately
  ([contract](docs/M8_CANADIAN_CONTRACT.md),
  [validation](docs/M8_CANADIAN_VALIDATION.md)). Ideal-price enclosures,
  current-market certification and independent acceptance remain open.

- Local fixed-principal US Treasury model: `UsTreasury`,
  `TreasuryFirstPeriod`, `TreasuryDailyBasis`, `TreasuryTieRule`,
  `TreasurySourcePolicy`, `TreasuryPricingPlan`, `TreasuryValuation`,
  `TreasuryRisk`, `TreasuryYieldReport`, `TreasuryError`, `TreasuryYieldError`,
  `TreasuryCashInstruction`, `TreasuryCashDecision` and `TreasuryCashReceipt`.
  Separate original dated/issue roles, explicit direct maturity cycles and
  all four publication policies retain raw analytical coefficients alongside
  selected source cash values. Integer p256 factor observations feed direct
  IEEE present values, all six independent risk views and counted bounded
  yield inversion. Four exact cash recipes preserve source publication,
  signed rounding, continuous Exact semantics and complete provenance.
  Fourteen types bring the public catalog to126;24 Treasury and5 yield
  errors bring the domain catalog to199. The complete independent
  pre-production gate precedes implementation; final local verification is
  recorded separately ([contract](docs/M8_US_TREASURY_CONTRACT.md),
  [validation](docs/M8_US_TREASURY_VALIDATION.md)). Canadian models,
  ideal-price enclosures and independent acceptance remain open.

- Local conventional UK gilt model: `ConventionalGilt`, `GiltFirstPeriod`,
  `GiltPricingPlan`, `GiltValuation`, `GiltRisk`, `GiltYieldReport`,
  `GiltYieldError`, `GiltError`, `GiltCashInstruction`, `GiltCashDecision` and
  `GiltCashReceipt`. Complete explicit maturity-anchored quasi dates and
  ex-dividend cutoffs determine lifecycle and seven source accrual branches.
  Published short/long first dividends use exact six-place half-up rounding;
  source accrued interest retains its separate unrounded rational. Pricing
  rounds exact source coefficient times each observed discount once, yield
  inversion retains counted cached payloads, and all six risk views use
  extended terms and exact wide sums. Four explicit cash recipes preserve
  rational source values, evidence provenance, Exact-stage semantics and wide
  cancellation ([contract](docs/M8_GILT_CONTRACT.md)). Eleven public types
  brought the catalog at that checkpoint to112;31 new domain errors brought
  it to170. Its original remaining-market scope is tracked by later entries.

- Local M8 phase D generic bond yield-to-maturity inversion and analytical risk
  in `sabiquant::bonds`: `BondPricingPlan::yield_from_price`,
  `BondPricingPlan::risk`, `BondYieldReport`, `BondYieldSensitivity`,
  `BondRisk`, and `BondYieldError`. Yield inversion uses explicit caller
  brackets and solver options, exact target-selected residual scaling (`s=0`
  or `1`), and a payload-preserving Brent solver (`brent_with_payload`). Risk computes
  all six analytical views (`price_derivative`, `price_second_derivative`,
  `time_weighted_amount`, `modified_duration`, `macaulay_duration`, and
  `convexity`) using five-limb rational products rounded to 53-bit
  extended-exponent terms and exact 243-limb fixed-point accumulators on
  `2^-8192`. Raw and normalized views have independent range admission.
  Named market yield models remain open
  ([contract](docs/M8_BOND_YIELD_RISK_CONTRACT.md)).

- Local M8 settlement boundary in `sabiquant::settlement`: four explicit
  per100/per1000 recipes with positive exact par nominal, supplied decimal or
  binary64 evidence, and caller-declared closed real intervals. Seven signed
  modes apply at explicit source/cash stages; total versus component rounding
  is a required choice. A bounded 2,560-bit integer kernel retains wide
  cancellation and checks continuous/discrete Exact domains before final cash
  admission. Receipts preserve the complete instruction and conditional
  provenance. Named market acceptance and analytical enclosure production
  remain open ([contract](docs/M8_SETTLEMENT_CONTRACT.md)).

- Local M8 bond core in `sabiquant::bonds`: `Bond::zero_coupon` and
  `Bond::fixed_rate`, retained schedules/redemption, explicit `BondSettlement`
  and per-coupon `ExCouponTerms`, earned-minus-detached signed accrual,
  exact single-rounding binary64 per-100 quote conversion, and two-stage
  direct-horizon `BondPricingPlan` / `BondValuation` with work budgets.
  Independent oracles and hand tests cover quotes, entitlement and valuation;
  an additive contextual oracle covers ICMA, 30E/360 ISDA, US 30/360 and
  Business/252 across all32 compounding configurations. Yield inversion,
  duration, convexity and named market models remain open
  ([contract](docs/M8_BOND_CORE_CONTRACT.md)).

- Local M8 phase B binary64 boundary for exact amounts: four named
  conversions with no implicit cast. `from_binary64_exact` reads a finite
  binary64's exact bit-field rational (binary64 `0.1` needs scale 55 and is
  rejected); `from_binary64_rounded` rounds that rational once to an explicit
  quantum and mode in a 1216-bit integer ratio; `to_binary64_nearest`
  correctly rounds an amount, ties to even, by integer comparisons only; and
  `to_binary64_exact` succeeds only for exactly representable values. NaN and
  infinities fail first with `AmountError::NonFinite`. The private kernel is
  now generic over its limb count with unchanged algorithms
  ([contract](docs/M8_BINARY_BOUNDARY_CONTRACT.md)).

- Local M8 exact decimal foundation: canonical signed i128 values at scale0–38,
  exact text and numeric ordering, explicit positive arbitrary quantum and seven
  rounding modes, checked exact arithmetic, cancellation-safe slice reduction,
  and single-round ratio/two-decimal product-divisor operations. A bounded
  original 512-bit stack kernel preserves signed limits and recoverable
  intermediate overflow. Eight typed amount errors integrate with the root
  error. Existing f64 analytics retain their policies; named financial conventions
  and full M8 yield/risk work remain open
  ([contract](docs/M8_AMOUNT_CONTRACT.md)).

- Explicit `Thirty360Us::new(end_of_month_security)` with both immutable
  security contexts, ordered February/31st rules and exact signed integer counts;
  scalar `DayCount::ActualActualAfb` with the ISDA 1999 backward-year
  clarification, original-end preservation and half-open residual leap-day
  counting. Both take constant time across the full civil range and compose
  with rates, equivalent-rate targets, coupons and bounded legs. Independent
  table/iterative rational oracles and attested public reference probes retain
  deliberate signed differences. US rules pin ROC December 2025 A001;
  the complete SIFMA book and financial/milestone acceptance remain open
  ([contract](docs/M3_US_AFB_CONTRACT.md)).

- Automatic local OldCDS schedules through `ScheduleBuilder::build_old_cds`:
  positive stored Months/Years strides, quarter-twentieth terminal projection,
  a thirty-calendar-day first-quarter cutoff that never removes the terminal,
  generation that stops at the requested end, exact B-anchored lattice flags,
  immutable `OldCdsProvenance` in definition identity, exact pre-generation
  counts, original/expanded TARGET guards, typed `InvalidOldCdsTenor` and
  `OldCdsDate` errors and B-anchored generated ICMA references. Independent
  civil/ICMA oracles and attested reference probes classify every observed
  difference. Historical financial acceptance of the cutoff and start role
  remains open.

- Explicit modern CDS quoted-maturity resolution for Quarterly2009 and
  Semiannual2015, with stored tenor identity, a typed Matured outcome,
  abstract civil anchors and exact required-date error causes.
- Immutable `CdsDatePlan::standard_following` and `CdsCouponDates`: quarterly
  raw boundaries, strict payment selection after trade plus one calendar day,
  Following payments, adjusted ordinary accrual and included raw final
  maturity. Explicit coupon caps, whole-span TARGET guards and typed
  ordering/calendar failures retain their contexts. Independent candidate
  enumeration, Gregorian-cycle controls and primary numerical examples cover
  these analytical date operations. Legal protection, premium-leg pricing,
  settlement and full M7 acceptance remain open.

- Generated ICMA contexts for all four civil rolls, using actual same-roll
  Wednesday/twentieth points and minimal resolved raw hulls, explicit matching
  frequencies, shared period caps and retained TARGET coverage. Ordinary
  date/EOM references retain their existing semantics.

- Four local M7 civil roll definitions: ThirdWednesday, ThirdWednesdayInclusive,
  Twentieth and TwentiethIMM, with explicit endpoint projection, genuine
  adjacent-roll flags, original/stored-tenor provenance, bounded generation,
  whole requested/resolved TARGET guards and typed failures. DateGenerationRule
  represents them as Roll(SpecialRollRule). Modern CDS date plans have a
  separate construction contract; existing date-rule outputs retain theirs.

- Local M7 analytical cash flows: finite signed simple amounts, plain SIMPLE
  fixed coupons with one-round exact binary64 triple products, shared complete
  accrual contexts and bounded ordered legs. Raw contractual schedule accrual
  remains separate from adjusted payment dates.
- Flat-rate NPV and outstanding accrued amounts with explicit date inclusion,
  sequential contextual work budgets, original indexed typed causes and exact
  reduction of rounded terms. NI-01 decimal settlement and full M7 acceptance
  remain open. Independent Fraction/Decimal oracles cover arithmetic, accrual,
  discount configurations and signed cancellation.

- Local M7 generated ICMA contexts: minimal same-anchor contractual reference
  hulls, explicitly matched periodic frequency, required checked civil
  extensions, whole TARGET hull coverage, shared final-period caps and typed
  constructor causes. Independent civil-envelope/Fraction fixtures verify
  exact periods and the existing accrual accuracy profile.

- Local M7 explicit coupon-date construction: validated optional first and
  penultimate roles, one shared interior date for equal roles, exact anchored
  regular middle, outer lattice regularity and retained definition metadata.
- Independent explicit civil-span date/error/cap fixtures and separately
  attested bounded public-reference probes, with all domain/roll/flag
  differences classified. The complete daily civil range also exercises
  explicit outer-role counting in both directions.

- Local M7 core schedules: owned immutable Forward/Backward/Zero values,
  anchored calendar recurrence, explicit automatic None/Short/Long stubs and
  calendar-EOM policy, separate contractual/adjusted dates and raw regularity,
  explicit ordinary/termination adjustment and retained checked TARGET.
- Independent civil-lattice schedule fixtures, a complete supported daily
  span, Gregorian-cycle month-end cases and bounded attested public reference
  probes. Core evidence classifies EOM/collision/Zero metadata distinctions;
  seven specialized rules and later cash-flow work were pending at that
  saved core snapshot; subsequent analytical scope is listed above.

- Local M6 numerical foundations in `sabiquant::math`: bracketed bisection,
  Brent and safeguarded Newton with complete callback accounting; immutable
  linear, log-linear, natural cubic and pinned PCHIP interpolators with explicit
  extrapolation; standard/location-scale normal density, CDF, survival, log
  tails and inverse; checked Neumaier and exactly rounded fixed-point sums.
- Independent exact-Fraction and 120/200-digit Decimal math fixtures, finite
  extreme-grid/boundary regressions, callback traces and scaled polynomial
  roots. Numerical contracts, local reviews and remaining platform/acceptance
  gates are recorded separately from observed accuracy.

- Local M5 review fixes: descriptive calendar EOM/endpoint/holiday-weekend
  enums, definition-based hashing for ICMA/accrual contexts, executable
  contextual examples, reference search aliases and equality/cache guidance.
- Packaged library unit tests include their owned standard-library support
  source and lint configuration; packaged README links use repository URLs.

- Local M4 rates: immutable quotes owning scalar/contextual accrual, all five
  compounding methods and ten explicit periodic frequencies; time/date factors,
  implied compound/discount rates, equivalent conventions, signed single-amount
  present values and explicit percent/basis-point conversions.
- Independent two-precision Decimal rate fixtures, all-pair date-context
  composition tests, seeded financial invariants and attested public QuantLib
  comparisons with explicitly classified numerical/domain differences.

- Local M3 extensions: algorithmic Western Easter across years 1–9999;
  guarded TARGET euro settlement rules, source attribution and disjoint verified
  intervals, with strict coverage by default and explicit projection.
- Bond Basis; explicit-termination 30E/360 ISDA; explicit-calendar Business/252;
  Actual/Actual ICMA with required contiguous determination periods/frequency.
- Exhaustive/rational and attested reference validation of the extensions;
  weekday-only calendar counting uses complete weeks and distinct dated
  override corrections without a global cache.

- Initial local M3 core batch: immutable Null, weekends-only, custom and joint
  calendars; seven business-day conventions; bounded business-day arithmetic,
  endpoint counting, business month ends and inclusive lists.
- Four source-verified day counts: Actual/360, Actual/365 Fixed, Actual/Actual
  ISDA and European 30E/360, with independent exact/rational and actual
  QuantLib reference validation.
- M3 calendar/day-count primary-source reviews, staged scope/test plan,
  pathological-calendar cases and qualified reference discrepancy records.

- M2 date/time APIs in `sabiquant::time`: checked Gregorian dates across
  0001–9999, strict ISO text, month/year clamping, weekdays and months,
  normalized tenors, partial comparison and periodic frequency mappings.
- Date and period domain errors, propagated through the crate-level error.
- M2 delegated decisions, exhaustive date validation and independent Python
  and pinned QuantLib reference generation (see `docs/M2_VALIDATION.md`).

- Licence: `MIT OR Apache-2.0`, at the user's option (`LICENSE-MIT`,
  `LICENSE-APACHE`).
- Governance files: `CONTRIBUTING.md` (licensing of contributions, Developer
  Certificate of Origin sign-off, provenance rules), `CODE_OF_CONDUCT.md`
  (Contributor Covenant 3.0), `SECURITY.md` (private vulnerability reporting),
  and a pull-request template with the review checklist and the provenance
  attestation.
- Owner decisions for M1 recorded in the ADRs and the decision register;
  M1 authorized.
- The `sabiquant` crate (M1, core foundations): Rust 2024 edition, MSRV 1.99,
  no dependencies, no unsafe code, and every public item documented.
- `sabiquant::error`: the crate-level `Error`, and the shared `ValueError`
  (rejected real numbers, NP-01 and NP-02) and `IdentifierError` (unknown or
  ambiguous identifiers, FR-CORE-05 and FR-CORE-06), with stable kind names
  such as `value.not_finite`. `NonFinite` classifies non-finite values, and
  `Interval` describes the domain of a parameter or result.
- Internal foundations for later milestones: numeric validation helpers and
  identifier tables with aliases and ambiguous labels.
- Test support (test-only): the SplitMix64 generator with a property-test
  runner, ULP and tolerance comparison, and a reader for the fixture format
  of TESTING_STRATEGY §7.3.
- CI on x86_64 Linux, aarch64 macOS and x86_64 Windows, plus the MSRV,
  formatting, Clippy, documentation, the dependency policy and DCO sign-off.

### Behavior changes

- `DateGenerationRule` now catalogs ten constructible labels: CDS and CDS2015
  through the modern date-plan API, and `OldCDS` (variant `OldCds`) through
  `build_old_cds`. Ordinary recurrent/explicit schedule builders reject all
  three. Text `OldCDS` previously failed to parse; existing rules' outputs
  are unchanged.

- Civil-roll schedules now generate same-roll ICMA references instead of the
  temporary unsupported error. Remove the unreleased `RollIcmaUnsupported`
  variant/kind; invalid cap, Once, tenor mismatch and required extension errors
  use the existing typed ICMA contract. No standard CDS product is implied.

- Explicit coupon inputs must be strictly interior and ordered; reversed
  roles and off-lattice middle endpoints produce contextual errors. Outer
  Regular means adjacency on the named middle lattice, including explicit
  EOM activation and month-clamp provenance. No implicit internal stub or
  rewritten contractual coupon is introduced to match reference behavior.

- Schedule adjustment rejects every adjacent collision/reversal with both
  raw/adjusted contexts. Calendar-EOM-only activation and explicit NonPeriodic
  Zero metadata are local contract choices with recorded reference differences.
  Final-boundary limits are checked without rejecting an unused out-of-range
  next candidate; Long compression may fit where Short exceeds the cap.

- New cubic preparation uses dimensionless coefficients to preserve small
  variations on large/small grids. Exact knots and affine identities are
  retained; unsupported normalized grids return contextual errors.
- Normal affine quantiles use the actual overflow midpoint and a conditioned
  uncertainty guard. Safe finite MAX means and exact medians are preserved;
  unresolved boundary results return a typed numerical-boundary error.

- Calendar and TARGET option parameters now require named enums. The
  adjustment, endpoint and holiday rules remain the same; boolean adapters
  are deliberately absent. TARGET has no `Default`; choose its named
  constructor explicitly. These API changes precede any public release.

- Derived rates reject a rounded nonpositive active simple factor. Log-space
  equivalence preserves tiny rates and can avoid intermediate-factor overflow.
- Elementary factor calculations reject an unresolved binary64 range boundary
  under the documented conservative policy. Rate/factor reconstruction bounds
  include exponential and rate-representation conditioning; the multiplier is
  unchanged. Same-execution bitwise claims are qualified to match Rust's
  documented elementary-function contract.

- Bond Basis, 30E/360 ISDA and Business/252 reverse the chronological forward
  result; equal accruals give zero within the context domain. Precise departures
  from direct QuantLib signed/February formulas are recorded and tested.
- TARGET queries preserve coverage errors through all candidates, overrides,
  joint rules and Business/252, including zero or Unadjusted operations.

- Unadjusted EOM advancement activates only for a calendar-end start; the
  earlier draft's business-end fallback was corrected after a reference fixture
  rejected the expectation.
- Closed custom calendars are valid classification/composition values. Business
  operations honor explicit date overrides and report typed bounded-search errors.

- `Error::source` now exposes the wrapped domain error, as required by
  ADR-0007, instead of skipping it. Callers can downcast to inspect its context.
- M2 tenor equality normalizes unit families; zero belongs to the canonical
  day family for ordering. Zero and a nonzero month/year tenor are incomparable.
  This is an intentional divergence from QuantLib's comparison semantics.
