# Local M7 civil roll validation

Validation of the [adopted four-rule contract](M7_ROLL_CONTRACT.md) on the
local branch `feat/m7-civil-rolls`, following `ffa7fe3`. This report concerns
generic civil roll dates and their retained definition, not standard CDS
products or full M7 acceptance. No remote write is part of this work.

## Implemented definition

`build_roll` implements ThirdWednesday, ThirdWednesdayInclusive, Twentieth and
TwentiethIMM on an explicit month lattice. It retains requested/resolved
ranges and stored tenor in immutable `RollProvenance`; DateGenerationRule
truthfully returns Roll(rule). All endpoints are kept, adjusted one-to-one,
and strict order is required. Regular means genuine consecutive roll points.
Original and expanded TARGET ranges are guarded before generation.

There are 66 public declared value/error types, all covered by the central
public Clone/Debug/PartialEq/Send/Sync/static trait checks. Four new schedule
error kinds bring the exhaustive nested root-error catalog to 93 variants,
including display, formatting failure propagation and typed sources. The
new roll definition is a closed enum; evolving domain errors remain
non-exhaustive. Caps are operation budgets and do not affect value identity.

## Independent civil oracle

The Python datetime/calendar generator enumerates candidate civil months and
selects congruent points, independently of Rust recurrence. Wednesday truth
enumerates all weekdays in each month. The Rust consumer additionally uses
an independent Gregorian ordinal formula for Wednesday membership and exact
adjacent-roll flags. Neither side imports Rust-produced expected values.

There are **122,233 records**, comprising **122,183 distinct input tuples and
50 repeated controls**. All 4,800 months of a Gregorian 400-year cycle are
tested under all four rules and six endpoint/stride patterns, with rotating
tenors. This is not every possible date/tenor/calendar cross-product.
Additional century, civil-extreme, huge stored-tenor and rejected-domain rows
extend beyond the cycle. Most generated spans are at most 25 months; whole
civil-span resource tests are separate evidence.

| Outcome | Records |
|---|---:|
| Successful defined schedules | 116,811 |
| Collapsed inclusive resolved range | 5,290 |
| Required projected date outside civil range | 54 |
| Invalid stored tenor/rule combination | 78 |

Successful records check **354,671 exact boundaries**, **114,065 Regular**
and **123,795 Stub** flags, every provenance inspector, all absent artificial
metadata and exact cap/cap−1/cap+1 behavior. There are 20,607 projected starts
and 63,714 projected ends. Required-date causes retain the actual original
Date, checked month count and role. Dates, flags and statuses use exact E0.

The dedicated consumer passed debug and release; generator `--check` and
strict focused Clippy passed. These focused results do not substitute for
the final aggregate, package or coverage checks below.

## Public reference observations

The original 46-row fixture is unchanged. Its updated consumer classifies
33 shared dates/flags, four calendar-EOM distinctions, four adjustment
collision rejections, one Zero metadata distinction, one Wednesday flag
distinction and three still unsupported CDS-family observations.

The [538-row attested reference report](M7_ROLL_REFERENCE_PROBES.md) preserves
466 returned values and 72 constructor errors, including five non-strict
vectors. Of 248 requests compatible with the local API, 110 agree exactly,
100 differ only in flags, 13 differ in dates and 25 receive a local typed
rejection (19 invalid tenors, six inclusive collapses). The other 290 are
254 CDS-family observations and 36 requests with absent EOM/coupon-role options.

The 13 date distinctions are pinned individually: six retain a local third
Wednesday strictly before the requested end that the reference omits; seven
retain a projected Twentieth terminal boundary that the reference suppresses
under terminal Preceding. Off-roll first and off-stride final intervals can
be local Stub despite reference Regular flags. Reference outputs/errors remain
observations; no tolerance hides additional or missing dates. The 144 separate
maturity calls are not additional Schedule constructor cases. Wheel/module
byte attestation does not establish exact source-commit attestation.

## Source review and bounds

Independent review found no concrete implementation defect in the civil
selection, original/expanded span guard, definition identity or integer
progress. A positive month stride is at most 12*i32::MAX. Each entered loop
coordinate is within the resolved civil end (at most 119,987); adding one
stride remains safely in i64. Future unused coordinates are compared before
Date construction. Required endpoint overflow uses checked Date arithmetic
and cannot be dropped to fit a cap.

The signed-congruence flag check is valid for every reachable output: an
initial date before a twentieth anchor is off that eligible roll; Wednesday
anchors share the original start month. No negative on-roll member can be
misclassified as an adjacent regular period. The source never changes its
anchor after a clamp or payment adjustment. Final vectors reserve fallibly;
the inherited owned Calendar Clone retains std allocation behavior, so this
is not a promise to recover every system OOM. No forced OOM or fabricated
private invalid state is used to raise coverage.

## Final verification

Final native macOS arm64 full suites terminated successfully:

| Profile | Unit | Integration | Doctest | Total |
|---|---:|---:|---:|---:|
| Debug | 80 | 400 | 28 | **508** |
| Release | 79 | 400 | 28 | **507** |

Each saved v2 log has 56 result groups, zero failures, zero ignored tests and
zero warning lines. An inherited debug-assertion test accounts for the profile
difference. The 28 doctests include three compile-fail cases. The new hand,
civil-oracle and reference harnesses contribute 20, one and four integrations.
Formatting, strict all-target Clippy, public/private warning-denied rustdoc,
and Linux GNU/Windows MSVC all-target warning-denied checks passed. The latter
are cross-compilation, not native execution on those platforms.

The hand suite builds all four actual minimum-stride whole-civil-range
schedules: 119,989 ThirdWednesday, 119,988 Inclusive, 119,989 Twentieth and
39,997 TwentiethIMM boundaries, with respective Stub counts 2/0/1/1. Other
cases cover all 49 ordinary/terminal adjustments on one custom Twentieth
lattice, typed coverage/adjustment failures, immutable concurrent reuse and
independently pinned generic fixed-coupon composition. These are finite
designed tests, not every rule/calendar/convention cross-product.

Fresh LLVM production-region coverage completed successfully after adding
the publicly observable interior-cap precedence test found by the first run.
It measures **1,036/1,074** regions in `src/schedule.rs` and **285/296** in
`src/schedule/roll.rs`, combined **1,321/1,370 (96.42%)**. The new test checks
all ten insufficient interior/terminal caps across the four hand definitions
before any all-closed-calendar adjustment; four adequate-cap controls then
reach that calendar error. Cash-flow regions remain 688/703.

The eleven uncovered roll regions are three same-valid-month Wednesday
date-error propagations, six regions of the valid-month conversion error
closure, one initial vector-allocation propagation and one required interior
date-error propagation. Valid civil coordinates prove the date/month error
paths unreachable for these inputs; allocation failure is not artificially
forced. The parent's 38 uncovered regions retain the inherited invariant/
allocation-wrapper limits. The runner excludes embedded test source and unions
instantiations; these are source regions, not branch coverage, exhaustive
input coverage, doctest coverage or financial-accuracy proof.

Saved evidence and hashes:

| Local artifact | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-m7-roll-debug-v2.log` | `142d5856aa32470b186d5ea18d1a07d1ebca65175de8a932bb7d393275d7a4bd` |
| `/private/tmp/sabiquant-m7-roll-release-v2.log` | `f1a8f7391caf876ca25d4ea7f73918e186344e4ae4e81fbcf955818c7dae4a6b` |
| `/private/tmp/sabiquant-m7-roll-coverage-final-v2/production-regions.json` | `debb39ab30857308b8ce3698aacb129cfd45a23562daf5c93ed138dedd3497a3` |

The independent [API review](M7_ROLL_API_REVIEW.md) and
[requirement audit](M7_ROLL_TRACEABILITY_AUDIT.md) found no concrete production
defect and independently checked the saved aggregate logs, public type census
and actual package member bytes. They are local technical evidence, not formal
financial/owner approval.

## Actual package artifact

An independent reviewer created and inspected the actual Cargo archive at
`/private/tmp/sabiquant-m7-roll-package-target/package/sabiquant-0.0.0.crate`.
It contains exactly **36 files**, **152,746 compressed bytes** and **666,779
file-content bytes**. All **34 original files** match the workspace exactly;
the remaining two are Cargo's normalized manifest and VCS metadata. Its
23 library source files and four owned test-support files require no fixture,
generator, integration harness, research file or native binary in the archive.
The retained VCS snapshot is `ffa7fe3`, dirty=true, before the local commit.

Cargo's normal package auto-verification passed. A fresh extracted archive,
with a separate target directory, passed **80 unit tests and 28 doctests**
(25 ordinary, three compile-fail), strict all-target Clippy and warning-denied
public/private docs, with zero extracted warnings. The package command's
51 notices concern expected omitted integration tests; they are not compiler
warnings. This snapshot was created before the final new integration harnesses,
which are excluded and do not change the archive's source bytes.
Metadata verifies one unpublished package, zero dependencies/features/build
script, and both license files byte-identical. This is native macOS arm64
evidence, not native Linux/Windows testing or publication.

Archive SHA-256:
`177b64c2238726bc8781a705775ba14e1eb684a678ab9b03abbf8547adaa132e`.
Evidence JSON `/private/tmp/sabiquant-m7-roll-package-evidence.json` SHA-256:
`800ebf4bf13f18f5b65fc489fa9541996fcefed31db1c19e97d909374cd31dd6`.
The root rechecked the archive's size/hash and every recorded command exit.

## Reproduction and pins

Use Rust1.99.0 from the declared local toolchain, all Cargo operations offline.
The generator is external validation tooling; package code stays std-only.

```sh
python3 validation/generator/schedule_roll_oracles.py --check
python3 validation/generator/schedule_roll_probes.py --quantlib-wheel PATH_TO_WHEEL --check
cargo test --offline
cargo test --offline --release
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps
RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --document-private-items
python3 validation/coverage.py --output NEW_DIRECTORY
```

| Artifact | SHA-256 |
|---|---|
| `src/schedule.rs` | `0cd9ac70a575756347acc44a86e9331ec08b93eb129c1b5a4402f77514035476` |
| `src/schedule/roll.rs` | `28af37afc2ba94fffa5afdd673cd8d147d43224eb11d9177db10b96225739b38` |
| Civil generator | `8853262070dc3b6fe8fcbee36b3cd881548086d4a55c3658a1378bbc17ea4503` |
| Civil fixture, 20,731,031 bytes | `4fdc3f29b41dc09f8a07c175a8cd889d4a16e4ce9a967e4c2db0af2c34f7cc30` |
| Civil fixture body | `12174da007c65d01e615a8d55c369010beab9167ac7950d5bd7d675dfea70e2b` |
| Civil consumer | `3063a2080a47b90d46e0192d01cd8edc07f5d83a3669b488b90ff1c3d7792cca` |
| Public hand suite (20 tests) | `c10278d17fa6772d684f03f0ca06b8f1528acef7d1d0f040844fb6317bbd5c63` |
| Reference generator | `03907177449e185b565b02961a06fc33c1edc5f58e16e98ff3eed6e2b4ff27f3` |
| Reference fixture, 478,179 bytes | `88e2bacdc1ab999c1979f19c5d2f4fa485f3492cceab040cca6cc927e9edf35a` |
| Reference consumer | `37ded5f0ca8a21cd789cbdc3ae732cbdb0cf9b50f5983f9f7f296903e707332e` |

All 69 prior fixture/generator artifacts at `ffa7fe3` were compared byte for
byte and remain unchanged. Prior cash-flow/ICMA evidence remains scoped to
its saved source snapshots. This tranche's fixture inventory is **1,293,977
rows in 51 files**: the prior 1,171,206 rows plus 122,233 independent civil
records and 538 public observations. Rows are designed cases/queries, not a
claim of that many independent financial examples. The separate
[package review](M7_ROLL_PACKAGE_REVIEW.md) records archive reproduction.

## Remaining full scope

Generated civil-roll ICMA still returns a typed unsupported error after cap
validation; [its next design](M7_ROLL_ICMA_DESIGN.md) requires a same-roll
minimal hull and exact independent accrual evidence. OldCDS/CDS/CDS2015,
CDS maturity/product conventions, missing historical sources, NI-01 exact
settlement and formal owner/financial/native-platform/release gates remain
open. This tranche advances M7; it does not complete that milestone or the
continuing project objective.
