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
