# Independent review of the four civil roll APIs

Reviewer: `/root/roll_audit`, 2026-10-14. This is a read-only production/test
review of the [adopted local contract](M7_ROLL_CONTRACT.md), followed by inspection
of the saved final validation logs and package bytes. The reviewer edited only
this report and [its requirement audit](M7_ROLL_TRACEABILITY_AUDIT.md). No production
code, tests, generator, shared requirement, Git state or remote service was changed.

## Finding and exact scope

No concrete production defect was found in the implemented four-rule contract.
The source preserves its month lattice, distinguishes requested from resolved
dates, validates original and expanded TARGET spans, and exposes typed failures
without returning a partial Schedule. These conclusions concern the explicit
analytical definitions; they do not approve market products or universal
QuantLib parity. Generated roll ICMA and the three CDS families remain open M7
work, as do source, owner, financial, native-platform and release gates.

The inspected production bytes are frozen as follows:

| File | SHA-256 |
|---|---|
| `src/schedule.rs` | `0cd9ac70a575756347acc44a86e9331ec08b93eb129c1b5a4402f77514035476` |
| `src/schedule/roll.rs` | `28af37afc2ba94fffa5afdd673cd8d147d43224eb11d9177db10b96225739b38` |
| `src/error.rs` | `bb3e7661f6867ed236c3513e4b5a89c0e0545362fa190f2607dc468802245c6a` |
| Adopted contract | `1787bedf20c07fd45dd2077c2ed6eeda3641c402111440e1807eaae6d347cd68` |

The review read all new roll production code, affected Schedule builder,
finishing, ICMA dispatch, equality/hash and error paths, the central public trait
and root-error tests, all three new integration harnesses, and the independent
civil generator. The old 46-row reference consumer's changed dispatch and
classification were also inspected. No C++ implementation or external financial
source was used to derive the review's civil expectations.

## API and semantic review

`SpecialRollRule` is a closed four-variant value. Its exact canonical identifiers
and `DateGenerationRule::Roll(rule)` agree, and the latter retains the existing
non-exhaustive generation enum. Both identifier registries have completeness and
round-trip checks. No trim, case-folding, aliases, implicit direction, EOM policy
or coupon-role interpretation enters `build_roll`.

Stored Months/Years validation precedes calendar work; zero, negative and
day/week periods are rejected. Years are converted in i64, while the original
length/unit is retained. TwentiethIMM additionally requires a stride divisible
by three. Normalized `Period` equality is deliberately insufficient for
definition identity: Schedule equality/hash use stored fields, and private
`RollProvenance` stores them separately. Both also retain the original input
range, so identical projected vectors can represent unequal definitions.

The civil selection matches each adopted definition:

- ThirdWednesday preserves the two requested endpoints and starts its interior
  Wednesday lattice one stride after the requested start month.
- ThirdWednesdayInclusive uses the third Wednesday of each endpoint month,
  rejects a collapsed range, and keeps the same start-month lattice.
- Twentieth keeps the requested start, resolves anchor and end independently
  to the first eligible twentieth on/after their inputs, and selects interior
  points congruent with the anchor and stride.
- TwentiethIMM restricts those anchor/end months to March/June/September/December.
  The additional stride restriction keeps all recurrent months eligible.

Regularity requires actual roll membership at both ends and exactly one stride
between them. The signed congruence check cannot produce a false Regular flag
through a negative lattice member on reachable inputs. Wednesday start/anchor
months coincide. A twentieth requested start before its anchor either is not
day 20 or is in an ineligible quarter month; an eligible on-roll start is already
the anchor. Off-stride terminal rolls therefore remain Stub. Adjustment never
re-anchors or changes these raw-period flags.

`roll_date` first uses checked month arithmetic and then selects the weekday or
day 20 in that month. Intermediate day clipping does not alter the selected
month. Required civil-range failures retain the original input, rule, stored
tenor, role and underlying DateError. A Wednesday within a valid input month
always exists; its public required-date error branches are structurally excluded
under the present Date invariants, rather than omitted behavior.

## Validation, resources and composition

The observed sequence is cap, stored tenor domain, original TARGET span,
required endpoint/anchor resolution, resolved range, requested/resolved union
coverage, bounded generation, then adjustment and strict order. Recurrent and
explicit builders reject Roll after cap validation and before other inputs.
Generated ICMA similarly checks its cap before the explicit unsupported-roll
error, even for Once or a mismatched frequency. This prevents accidental reuse
of the old clamped-date ICMA lattice.

The union is `[min(requested_start, resolved_start),
max(requested_end, resolved_end)]`. Original-span validation prevents endpoint
projection from hiding unverified inputs; union validation prevents forward
projection from introducing unchecked coverage. Existing calendar adjustment
also checks any searched candidate. Overrides, joint calendars and Unadjusted
do not bypass those market guards.

Positive stride and increasing month coordinates bound generation by returned
boundaries plus a terminal comparison. A coordinate used to construct a date is
at most the valid resolved end's month, 119,987. Its next value is at most
`119987 + 12*i32::MAX`, safely inside i64. The comparison happens before Date
construction, so an unused enormous next step is harmless; a required endpoint
cannot be swallowed by the same argument. Final cap failures report a sound
lower bound and occur before adjustment. The new cap-precedence hand test exercises
both interior and terminal cap failures with an all-closed calendar, making
that ordering observable.

The shared finalizer keeps one adjusted date per raw date, uses the distinct
terminal convention only for the final boundary, and rejects collisions or
reversal without repairing the vector. Vector growth reserves fallibly.
The inherited owned-calendar clone uses standard allocation semantics; no
system-wide OOM-recovery promise follows from fallible vector reservation.
Neither forced OOM nor fabricated private invalid state was used for evidence.

The public coupon composition test explicitly supplies Actual/365 Fixed and
checks raw accrual periods, adjusted payments and direct zero-rate valuation.
It demonstrates generic fixed-coupon interoperability. It does not establish
CDS premium accrual, settlement, maturity or payment conventions.

## Validation evidence and limits

The reviewer independently counted 66 public struct/enum declarations and
exactly 66 central value/copy trait checks, without omissions or extra names.
The two new values satisfy Copy/Debug/PartialEq/Send/Sync/'static and Eq/Hash.
The exhaustive root variant classifier now covers 93 unique nested variants;
the four new Schedule error kinds participate in kind, display, formatter-error
and source tests. Real public RollDate and calendar failures also inspect typed
root-to-domain-to-cause chains.

The final v2 logs were read and hashed rather than rerunning passed large suites:

| Native macOS arm64 log | Verified content | SHA-256 |
|---|---|---|
| `/private/tmp/sabiquant-m7-roll-debug-v2.log` | 508 passed; 56 result groups; zero failures/ignored/warning lines | `142d5856aa32470b186d5ea18d1a07d1ebca65175de8a932bb7d393275d7a4bd` |
| `/private/tmp/sabiquant-m7-roll-release-v2.log` | 507 passed; 56 result groups; zero failures/ignored/warning lines | `f1a8f7391caf876ca25d4ea7f73918e186344e4ae4e81fbcf955818c7dae4a6b` |

The root reports terminal exit zero for both commands. Counts split into
80 debug/79 release unit tests, 400 integration tests and 28 doctests, including
three compile-fail cases. Both logs include the final 20-test public hand suite,
one civil-oracle consumer and four reference-observation tests.

Fresh v2 coverage completed with root-reported exit zero. The reviewer read the
coverage script, rehashed `production-regions.json`, checked its counts and
inspected all 11 uncovered roll regions against source. The report is
`/private/tmp/sabiquant-m7-roll-coverage-final-v2/production-regions.json`, SHA-256
`debb39ab30857308b8ce3698aacb129cfd45a23562daf5c93ed138dedd3497a3`.
It reports `src/schedule.rs` 1,036/1,074 and `src/schedule/roll.rs` 285/296;
together 1,321/1,370 production regions, 96.42%. All 54 instrumented executable
logs were independently counted: 480 passes, zero failures and zero ignored.

The uncovered roll regions are three impossible zero-offset Wednesday-date
error propagations, six regions of the impossible valid-Date-to-Month error
mapping, one initial vector allocation-failure propagation, and one bounded
interior roll-date error propagation. The last is excluded because the loop
constructs only valid civil month coordinates, and every selected month has a
third Wednesday/day 20. The interior cap-error propagation at line 296 is now
covered by the new real public cap test. No uncovered reachable ordinary input
path was identified in this roll-source review.

This metric unions executed LLVM production source regions across native
unit/integration executables, excludes embedded unit-test source, and does not
instrument doctests or measure branch/input completeness. The parent
[validation report](M7_ROLL_VALIDATION.md) owns the full command/platform
inventory; neither percentage nor passing tests certify financial accuracy.

The reviewer also reopened the actual package archive and verified 36 members,
152,746 compressed bytes, 666,779 content bytes and all 34 original workspace
files byte-for-byte. Archive SHA-256 is
`177b64c2238726bc8781a705775ba14e1eb684a678ab9b03abbf8547adaa132e`.
The separate [package review](M7_ROLL_PACKAGE_REVIEW.md) records the actual
extracted 80-unit/28-doctest, strict lint and documentation runs. This review
does not claim to have repeated those runs or native Linux/Windows execution.
