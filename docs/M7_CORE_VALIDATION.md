# Local M7 core schedule evidence

This records the restricted schedule tranche on 2026-10-12, following saved
M6 commit `5fac9d8`, under the owner's continuous local-only authority.
[M7_LOCAL_READINESS](M7_LOCAL_READINESS.md) and
[REQUIREMENTS §14](REQUIREMENTS.md#14-fs-sched--local-m7-core-schedules)
pin the adopted contract. The core implements Forward, Backward and Zero;
it does not complete ten-rule schedules, ICMA reference generation, cash
flows or full M7 acceptance. No remote write or publication was performed.

## Resulting public behavior

The seven public schedule values retain an explicit date range, owned
plain/TARGET calendar, ordinary/termination conventions and generation
metadata. Recurrent construction uses positive stored tenors and one original
directional anchor; Days/Weeks count calendar days and Months/Years recover
the original day after clamping. Calendar-EOM preservation activates from the
raw directional calendar-month-end seed, independently of business EOM.

Automatic None/Short/Long policies have exact directional residual semantics.
Long removes its neighboring regular boundary before appending the endpoint,
so its final count may fit where Short does not. An exact lattice never
discards a regular boundary just because Long or a smaller cap was requested.
Zero has exactly two raw boundaries, one NonPeriodic interval and absent
recurrent tenor/stub/EOM metadata. No frequency or accrual semantics are inferred.

Raw/adjusted dates remain in one-to-one correspondence. Ordinary adjustment
applies to start/interior dates; only the final chronological endpoint uses
termination adjustment. Every adjacent collision/reversal returns the exact
offending index and both raw/adjusted pairs. Nothing is sorted or silently
discarded. TARGET validates the whole raw span even Zero/Unadjusted, then
retains errors from visited adjustment candidates outside that span.

The explicit final cap is `2..=3,652,059`. Supported-range prechecks prove
when an unused next candidate must also cross the valid endpoint, without
swallowing arbitrary date errors. Schedule-vector growth reserves fallibly;
copying existing calendar context retains std Clone's allocation behavior.
There is no guarantee of recovering every system allocation failure.
Generation/storage are O(n), plus retained calendar storage/clone and
classification/adjustment/TARGET span-validation costs.

Definition equality/hash preserve original stored tenor length/unit, rule,
stub/EOM policy, calendar and both conventions. Normalized equal Periods can
produce identical date vectors while defining unequal schedules. A larger
accepted cap is not contractual metadata and produces the same value/key.
All inspectors borrow immutable data or return copy values; no global state,
I/O, dependency, unsafe code or mutable cache was introduced.

## Independent and reference evidence

| Evidence | Exact measured scope |
|---|---|
| Civil oracle | **18,916** successful rows from independent Python datetime/calendar span enumeration and anchor congruence; all four units, both directions, None/Short/Long and compatible EOM policies; eight year classes including centuries/leap/range edges |
| Gregorian-cycle core regression | **65,988** preparations across every month from 1600–1999, valid days 28–31, both directions and both EOM policies; independently stepped civil months and month lengths, five exact dates/four flags each |
| Adjustment controls | All **49** ordinary/termination pairs with independently hand-audited weekend goldens; short/long raw flags remain independent of adjusted interval length |
| Complete supported daily span | All **3,652,059** boundaries from Date MIN to MAX, every adjacent difference one, all raw/adjusted values identical and regular flags exact; cap one smaller returns the exact required lower bound |
| Collections and reuse | Ten distinct HashMap keys, including nine definitions sharing every raw/adjusted date and flag; cloned/reconstructed/larger-cap lookups; 128 concurrent builder reconstructions of a 25-boundary value |
| Failure composition | Equal/reversed ranges; every nonpositive unit and extreme positive tenor; incompatible Zero/EOM; residual and limit ±1; closed months/366-holiday search bounds; collision/reversal; typed nested date/calendar causes |
| TARGET | Inception under both policies, strict verified endpoints spanning the 2003–2025 gap, projection, overrides and null-member joint rules; ECB Easter closures; verified raw spans whose Nearest/Following adjustment visits 2025/2029 and retains a contextual coverage error |
| Public reference | **46** bounded, wheel/module-byte-attested QuantLib 1.43 observations; 34 supported inputs exercised in Rust: 25 exact shared results plus nine declared distinctions; twelve other observations remain preparation evidence |

The three integration files contain **16 tests**: twelve in
[schedule_core.rs](../tests/schedule_core.rs), three in
[schedule_properties.rs](../tests/schedule_properties.rs) and one in
[schedule_reference.rs](../tests/schedule_reference.rs). Dates, flags, kinds
and scoped reference outcomes are exact TOL-E0; no tolerance masks a mismatch.
The tests inspect public outputs, actual collection use and source layers.
The civil oracle enumerates congruent in-span dates rather than importing Rust
helpers or reproducing its step counter. Its successes do not stand in for
the separate failure, Zero, calendar or TARGET tests.

The reference generator bounds each isolated worker to ten seconds and safe
2026 inputs/output envelopes, reattesting the installed native/module bytes
against the established wheel. Both new generators reproduce their final
fixtures with `--check`. QuantLib and Python do not execute during Rust tests.
Complete provenance, output controls and scope limits are in
[M7_REFERENCE_PROBES](M7_REFERENCE_PROBES.md). The source baseline recorded
there is not independently established from the wheel binary.

The unchanged M2–M6 layers retain **941,824** rows in 42 files. This tranche
adds 18,962 rows in two files: **960,786 rows across 44 fixture files** total.
Counts measure declared scenarios, not independent features, branch coverage
or proof of all combinations. No prior financial fixture was regenerated.

## Classified reference outcomes

[DISCREPANCIES](../validation/DISCREPANCIES.md#d9-core-schedule-contract-distinctions--2026-10-12)
pins every permitted distinction in the supported 34-row consumer:

| Outcome | Rows | Exact classification |
|---|---:|---|
| Shared date/regularity results | 25 | Anchored recurrence, automatic directional stubs, compatible EOM/endpoints and calendar-day/week examples agree exactly |
| Calendar-EOM-only activation | 4 | Business-EOM-only Jan 30/May 29 seeds, both Unadjusted/Following; local preservation stays inactive and exact expected vectors differ |
| Colliding adjusted dates | 4 | Three daily WeekendsOnly rolling controls and daily TARGET Easter; reference returns fewer boundaries, local returns exact contextual collision errors |
| Zero metadata | 1 | Same dates; local absent recurrent fields/NonPeriodic versus reference 0 Years/regular, without interpreting either as a coupon frequency |
| Deferred preparation observations | 12 | Seven specialized rule families and five explicit-coupon configurations are not local features or parity evidence |

Every unexpected date, flag, status or label fails the consumer. TARGET source
coverage and supported-date overshoot behavior have separate project contracts;
the bounded reference probes do not measure those policies or waive them.
No full-family or market-convention compatibility claim follows.

## Native verification and production coverage

On Rust 1.99.0, aarch64 macOS, the source-frozen repository passes **407 debug
tests / 406 release tests** across 44 result groups, including **24 doctests**
(21 ordinary/three compile-fail), with zero failures or ignored checks.
The existing debug-only invariant assertion accounts for the one-test
difference. Formatting, strict all-target Clippy, warning-denied public/private
documentation, dependency policy and whitespace checks pass. All-target Linux
GNU and Windows MSVC compile checks pass; they do not execute native tests
or establish R3/remote CI.

Native LLVM instrumentation in `/private/tmp/sabiquant-m7-final-coverage`
measures **568 / 590 schedule production regions (96.27%)** with embedded
tests excluded. It includes the complete daily span and all current integration
suites. Source SHA-256 at this snapshot:
`76b3019e16b5d3057784a84291a82e98db89e4c80da36f5d8bbfd034616fd42a`.

Unexecuted regions are fallible-reservation failures, their error propagation,
and unexpected required-date causes excluded by the validated support-range
prechecks. Error catalog/formatter tests cover their public kind/context/source
shapes without corrupting private date/count invariants or forcing system OOM.
Coverage is a source-region metric, not branch coverage or a claim that every
calendar/stub/range combination has been tried. Refactoring the redundant
interior fallback changed region boundaries, so raw region counts cannot be
treated as a before/after accuracy comparison.

Seven new public values bring the repository inventory to **56**. All are
covered by standard value/error and thread-trait checks; root error tests now
sample **70** unique nested variants, preserving schedule then calendar/date
source layers. [M7_LOCAL_API_REVIEW](M7_LOCAL_API_REVIEW.md) separates the
independent design/oracle review from the subsequent root implementation
review. The independent source review was interrupted and remains incomplete.

## Extracted package evidence

The offline archive has **32 files**, **129,935 compressed bytes** and
561,423 content bytes. SHA-256:
`ea46bbbbc026de9ad4f0be5bab890f7e4232f8b9eabf1eea48d833be931bd188`.
It captures `5fac9d8` plus dirty M7 changes, not a clean release tag.
All 30 original archive files match the frozen workspace, including original
manifest/readme/changelog; normalized manifest and VCS metadata are generated.
Extracted verification passes **73 unit tests**, **24 doctests**, strict
all-target Clippy and warning-denied public/private documentation builds.

The allowlist includes library/licence/readme/changelog material, Clippy config
and four owned unit-test support files. Research documents, integration suites,
fixtures, generators, third-party tools and the reference wheel remain excluded.
Version stays `0.0.0`, no dependencies and `publish=false`. Canonical repository
links do not establish remote availability of unpublished local documents;
publication requires current reviewed remote targets and a separate decision.

## Remaining full-scope and release work

Explicit first/penultimate coupon dates, seven specialized roll families,
automatic ICMA quasi-period generation and the remaining cash-flow/coupon/leg/
NPV/accrued-interest scope are pending. NI-01 exact settlement is unresolved.
They remain part of the wider M7 objective rather than being removed to make
this core tranche appear complete.

Formal M5/M6 dependency acceptance, independent source/API/milestone review,
native primary-platform CI, security-setting verification, TARGET maintainer/
annual-source review, original ISDA/QR-01 inspection and US/AFB source decisions
remain open. No local test, cross-compile, package or reference probe supplies
those approvals. The core is locally reviewable and reusable while the full
goal and broader milestones remain active.

## Reproduction

```sh
cargo test --offline
cargo test --offline --release
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc --offline --no-deps --document-private-items
python3 validation/generator/schedule_oracles.py --check
python3 validation/generator/schedule_probes.py --quantlib-wheel PATH_TO_WHEEL --check
python3 validation/coverage.py --output NEW_REPORT_DIRECTORY
cargo package --offline --allow-dirty
```

Use the declared Rust toolchain and a new coverage directory. External
reference regeneration requires the attested wheel/runtime and is distinct
from ordinary offline verification. Package hashes identify the recorded
source/VCS snapshot; later commits may change archive metadata.
