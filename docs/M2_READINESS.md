# M2 Readiness and Decision Record

| Field | Value |
|---|---|
| Assessment date | 2026-10-11 |
| Scope | M2: dates, periods, frequencies, reference generation and validation |
| Authorization | The owner asked the agent to choose the next work and continue autonomously, with the broadest feasible meaningful tests, in this session on 2026-10-11 |
| Decision authority | Implementation choices below were made by the agent under that delegated authority; they are not represented as individually reviewed owner decisions |
| Status | Authorized for local implementation; acceptance evidence is tracked in [M2_VALIDATION](M2_VALIDATION.md) |

## Prerequisite evidence

M0 and M1 were merged in [PR #1](https://github.com/iceman951/sabiquant/pull/1)
and [PR #2](https://github.com/iceman951/sabiquant/pull/2). The M1 head
`9cde4f9236e5774fd5d4b7240f4eba1f78725d70` passed
[CI run 38061356805](https://github.com/iceman951/sabiquant/actions/runs/38061356805):
tests on x86_64 Linux, aarch64 macOS and x86_64 Windows, MSRV, format/lints/docs
and dependency policy, and DCO. All six jobs completed successfully. The
owner merged the milestone. No cross-platform claim about subsequent local
changes is inferred from that earlier run.

## Decisions made for M2

| Item | Decision | Reason |
|---|---|---|
| OQ-09; ADR-0005 Part A | Date range 0001-01-01 through 9999-12-31, proleptic Gregorian; private 32-bit ordinal representation | Bounded, exact, four-digit ISO text; compatible with Python's independent date oracle |
| OQ-10 | Normalize equality and hashing across Years/Months and Weeks/Days; all zeros are equal to 0D | Equivalent tenors behave identically in maps and sets |
| OQ-10 ordering clarification | Nonzero tenors compare only within their unit family. Every zero uses the canonical Days family for ordering, regardless of its stored unit | Preserves the Rust equality/ordering laws and transitivity; comparing zero in both families would bridge otherwise incomparable tenors |
| OQ-10 arithmetic clarification | Zero is an additive identity even across families; nonzero cross-family addition remains an error. Same-unit operations preserve the unit; mixed compatible units produce the finer unit; check the final widened result | Zero acts identically across stored units; large terms can cancel without false intermediate overflow. Representability of nonzero arithmetic results depends on the specified output unit, not just normalized equality |
| OQ-11 (already decided) | Checked methods only; no arithmetic operators that panic | Input and range failures are typed errors |
| OQ-12 | Defer Excel/QuantLib serial interop | Avoids coupling date correctness to spreadsheet epoch quirks |
| OQ-13 (already decided) | Committed text fixtures, external generator in `validation/`, std-only Rust reader | Normal Rust tests remain offline and independent of QuantLib |
| ADR-0003 D2–D5 | Adopt feature-specific behavioral/numerical compatibility definitions, independent-source priority, discrepancy classes and qualified claims | Differential tests must distinguish intended differences from defects |
| Date text | Syntax must be exactly ASCII `YYYY-MM-DD`; syntax errors carry the original text; valid syntax with invalid components returns the constructor's structured error | Distinguishes text errors from invalid dates without hiding useful context |
| Weekday search | Next/previous target weekday are inclusive; search at most six days and return an error if the result is out of range | Clear, bounded behavior at MIN/MAX |

OQ-17 to OQ-24 and ADR-0005 Part B remain later-milestone decisions. Calendars,
day counts and interest rates are outside this M2 implementation.

## Engineering and validation gates

- Keep the accepted single-crate, standard-library-only, no-unsafe and
  explicit-state policies. No generator dependency becomes a Cargo dependency.
- Document every public item and stable error kind. Expose domain errors
  through the crate error's `source` for downcasting.
- Test every date across the full range, leap/century transitions, strict text,
  invalid components, clamping and extreme arithmetic.
- Check Period normalization, equality/hash agreement, order substitutability
  and transitivity, compatible arithmetic, overflow and every frequency.
- Generate reference values by actually executing Python datetime and pinned
  QuantLib 1.43; record provenance and intentional divergences.
- Run debug/release tests, format, Clippy, public/private rustdoc and dependency
  checks using the declared Rust 1.99 toolchain.
- A future PR must pass the primary-platform CI matrix and owner acceptance
  before M2 is recorded as complete. Local tests alone do not satisfy that gate.
