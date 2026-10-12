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
