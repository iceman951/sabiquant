# Local automatic OldCDS validation

Completed on 2026-10-15, Asia/Bangkok, on native macOS arm64 against the
local worktree based on `f814167`. This is engineering evidence for the
[adopted OldCDS contract](M7_OLD_CDS_CONTRACT.md), not full M7, financial,
owner or release acceptance. Work remained local; no push or CI run forms
part of this report.

## Implemented scope and authorship

`ScheduleBuilder::build_old_cds` builds the automatic local rule over a
positive stored Months/Years stride. It retains the generation start,
projects the initial quarter A and terminal R to quarterly twentieths, and
shifts the recurrence anchor B three months only when A < R is fewer than
thirty calendar days ahead. Interiors stop at the requested end and R is
appended. Exact lattice flags, immutable `OldCdsProvenance`, a pre-generation
count, original and expanded TARGET guards, two typed errors and B-anchored
ICMA references complete the operation. `DateGenerationRule` catalogs ten
labels. The declared public type census is 72, and all 72 have the central
trait checks. The root error catalog covers 103 variants.

Codex prepared the design inputs and the first 551 reference observations
before its usage limit. Claude Code then froze the contract and wrote the
production code and the fixture consumers. Separate agents wrote the civil
and ICMA oracle generators, the attested probe generator and the hand suite
from the contract. None of them read the OldCDS production code. The hand
suite's author stopped at a session limit before reporting; the integrator
then compiled, ran, linted and reviewed it unchanged. Two further read-only
reviewers examined the code and the evidence; findings and resolutions are in
the [implementation review](M7_OLD_CDS_IMPLEMENTATION_REVIEW.md).

Before the retained suites existed, a temporary differential run compared
146,097 cycle cases with an independent Python model over all seven
conventions on a weekend calendar. All raw/adjusted dates, flags, quarters,
omissions, domain rejections and collisions agreed. That scratch run is not
retained evidence; the suites below are.

## Retained suites

All date, count, status and flag comparisons are exact E0. ICMA binary64
fractions use the existing E2 profile of at most 4 ULP, and each rational is
also recomputed inside the test from the oracle's reference hull.

| Suite | Evidence |
|---|---|
| `schedule_old_cds.rs`, 27 tests | Label registry; cap/tenor/coverage/terminal/expanded-guard/cap/adjustment precedence; every stored-tenor domain error with unnormalized context; distances 0/1/29/30/31 before every 2026 quarter; leap cutoffs; stop-at-end for every stride; flags; huge tenors; B = R; provenance, aliases, identity and threads; exact counts and caps; the 119,987 civil maximum; terminal failures after 9999-12-20; Display/source/root chains; all 49 convention pairs with weekend starts, interiors and terminal; collisions and search failures; strict/projected TARGET, overrides and joints; a day-walk oracle over every 2027 and 2028 start × 13 ends × 15 strides |
| `schedule_old_cds_oracles.rs`, 1 test | All 162,644 civil-oracle rows: dates or the order-sensitive checksum, flags or run lengths, A/B/R/omission, metadata and typed errors; header status, profile, flag, omission, anchor-class and terminal-shape censuses recomputed from actual results |
| `schedule_old_cds_icma.rs`, 4 tests | All 10,363 ICMA rows: hulls, period counts, lower steps, 24,972 raw-interval and 9,391 whole-span fractions, caps and stored-unit Date causes; strict versus projected TARGET hulls; context equality across 49 convention pairs and 1Y/12M; raw-interval fixed-leg composition with the 31.25 control |
| `schedule_old_cds_reference.rs`, 2 tests | An independent Rust classifier places each of the 1,199 attested probes in its recorded class, none unclassified; the 98 frozen roll-probe rows classify as 56 exact, 29 flags-only, 4 tenor-domain and 9 EOM/role options with the recorded IDs |
| `old_cds.rs` units and doctests | Cutoff/terminal retention and stop-at-end units; the `build_old_cds` example and the OldCDS ICMA example (exact 119/360 bits) |
| Updated existing suites | The RULE-OLDCDS control in `schedule_reference.rs` now matches the local rule exactly; the ten-label registry in `schedule_roll.rs` (and `schedule_old_cds.rs`), an OldCDS parse check in `schedule_cds.rs`; the public trait census; the 103-variant error catalog and source table |

The three generators reproduce their fixtures with `--check`. The probe check
re-ran all 1,199 attested QuantLib 1.43 workers and confirmed the 551 earlier
rows field for field against Codex's temporary JSON.

## Reference classification

| Set | Rows | exact | flags-only | T1 | T2 | S1 | S1+T2 | domain | local collision |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Original fresh probes | 551 | 152 | 299 | 41 | 11 | 36 | 12 | 0 | 0 |
| Extra probes | 648 | 213 | 257 | 19 | 58 | 63 | 12 | 14 | 12 |
| Frozen roll probes | 98 | 56 | 29 | – | – | – | – | 4 | 9 options |

T1 is an off-quarter requested end on the stride lattice, T2 an adjusted-end
coincidence and S1 a holiday start kept unadjusted by the reference. Flags
differ where the reference marks a projected terminal or a first interval
by construction shape. Each is a deliberate local definition in the
contract, not a tolerance; see the [probe report](M7_OLD_CDS_REFERENCE_PROBES.md).

## Aggregate native checks

The explicit toolchain directory was first on PATH for every Rust command:

```text
/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin
rustc 1.99.0 (b940084d7 2026-09-28)
cargo 1.99.0 (5f94df478 2026-08-27)
```

Each command used its own target directory under
`/private/tmp/sabiquant-m7-oldcds-verified-*`, on stable sources.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked`, warning-denied rustdoc | 593 passed in 67 groups: 82 unit, 479 integration, 32 doctests; no warnings |
| `cargo test --offline --locked --release`, warning-denied rustdoc | 592 passed: 81 unit, 479 integration, 32 doctests; no warnings |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Completed successfully |
| `cargo fmt --all --check`; `git diff --check` | Exit zero |
| Warning-denied public and private `cargo doc --no-deps` | Exit zero |
| Warning-denied `cargo check --all-targets` for x86_64 Linux and Windows | Exit zero; compile-only |
| Three generator `--check` runs | Exit zero; fixtures reproduced byte for byte |

The one debug/release difference is the inherited debug-assertion-only test.
Cross-compilation is not native Linux or Windows execution.

## Production-region instrumentation

The matching LLVM tools and `validation/coverage.py` ran 65 instrumented
unit/integration binaries: 561 tests passed, none failed or ignored. Doctests
are not instrumented, and region counts do not certify branch, input or
financial coverage.

| Production scope | Covered / regions |
|---|---:|
| Entire crate | 8,836 / 9,065 (97.4738%) |
| `schedule.rs` | 1,167 / 1,208 |
| `schedule/old_cds.rs` | 165 / 171 |
| `schedule/roll.rs` | 324 / 335 |
| `schedule/cds.rs` | 316 / 330 |
| `schedule/cds_maturity.rs` | 102 / 102 |
| Five schedule files combined | 2,074 / 2,146 (96.6449%) |

The six uncovered OldCDS regions have identified limits. Three propagate
Date failures that cannot occur once the terminal exists: the initial
quarter, its three-month successor and an interior stride point (lines 172,
178 and 207). Three propagate allocation failure while storing boundaries
(lines 201, 217 and 229); no OOM was forced. The parent's 41 and the civil
roll's 11 uncovered regions keep their earlier classifications. An early
refactor had made six more roll regions unreachable; it was corrected, as
the [implementation review](M7_OLD_CDS_IMPLEMENTATION_REVIEW.md) records.

## Package integrity

`cargo package --offline --locked --allow-dirty` produced an actual archive;
a fresh extraction passed 82 unit tests, 29 executable and 3 compile-fail
doctests, strict Clippy and warning-denied public/private docs. The archive
has 39 regular files: 37 byte-checked originals and two Cargo-generated
files. Its allowlist differs from the previous modern CDS archive only by
`src/schedule/old_cds.rs`; no fixture, research document, integration test or
generator is packaged. Metadata shows one package, no dependencies or
features and `publish = false`. The archive is 166,090 bytes compressed and
732,789 bytes of content. The current fixture inventory is 1,659,518 rows in
56 files; every earlier fixture and generator is unchanged.

## Reproducibility pins

Temporary logs are local to this machine; retained tests, fixtures,
generators and documents are the repository reproduction inputs.

| Artifact | SHA-256 |
|---|---|
| `src/schedule/old_cds.rs` | `2a244e315eec3b93f9b516051d453b745af6607912d743758afce406bb1a08b6` |
| `src/schedule.rs` | `f5b44560f80b1d1cdc5beaa83dcc4d947ec2bf947f565426b9a064186fff6532` |
| `src/schedule/roll.rs` | `ae32cee32228f178b4c963c801631b7e1b4d0bd0623ff7c3af9eaefacaa98e9d` |
| `src/error.rs` | `0594b480b15cdda1fe7db3006dd9c8fb6006a2395331acca8f7bbd39c1f02640` |
| `docs/M7_OLD_CDS_CONTRACT.md` | `24aebdcb67cf5127df56705e85093c905767da787552325d83cb60a4e2b3b996` |
| `old-cds-oracle.fixture` | `3b510f0c46645484aa39097f67ed04e62d87c5673d7f3100b9b689e1f71e9d05` |
| `old-cds-icma-oracle.fixture` | `b9958adcc7be06c5c3d585b0f79b5e0ded9ff9ebf1dd5b15f1aaee806104643d` |
| `old-cds-probes.fixture` | `ab55159feb02b4fbe4b01cc70a9a9de3366173b65b14ef5ad376b6c03d1461f9` |
| `tests/schedule_old_cds.rs` | `ed1fce51ae255ec51eaae3b40ca0619ae95a16f40ee3f8953dd626b185c0dd03` |
| `tests/schedule_old_cds_oracles.rs` | `409537142b286210b1a0a2a794305230b94daab9453e33c358f2d6333c5744ea` |
| `tests/schedule_old_cds_icma.rs` | `6bfa6bf1b92cf042d1f77ea566164bb8dbf92e971a499bead448c988740372e1` |
| `tests/schedule_old_cds_reference.rs` | `04e7785a4afa75e8a06f4d147bef6b7a982250b8b7878136c63a41f58ca33f74` |
| Native debug log, `/private/tmp/sabiquant-m7-oldcds-verified-debug.log` | `f1e7e69e2d065f263865a0d5482493d695a9d5fda4eaa8a09f4eb1033bc65902` |
| Native release log, matching `-release.log` | `4132b2f0bb5e8dacbd32c26a21f66a15197871a98328a2dc12cf8817e73e96cf` |
| Strict Clippy log, matching `-clippy.log` | `d473e547eaa601309ec0995b02816f66bfa94ab4a6426f31b890c7115b2231c5` |
| `/private/tmp/sabiquant-m7-oldcds-verified-coverage/production-regions.json` | `61c2082361f9154394ce12376907aa394451bc8650f0796a798e9ffa48bd12ea` |
| `.crate` archive under `/private/tmp/sabiquant-m7-oldcds-package/target/package/` | `5f239d7fab331f2c8b7d84b7f1848b57905562656d0a46c7d53e106f91843b34` |
| `/private/tmp/sabiquant-m7-oldcds-package/evidence.json` | `b449b36c16e72b210f4c205c9c6a4e6c106530f2e1a3cd19a2b4ba50db835977` |

## Remaining work

The contract is a local engineering definition. The historical thirty-day
equality and generic start role stay unresolved in the
[legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md); no historical CDS
premium leg, protection date or settlement convention is implied. NI-01,
US 30/360 and AFB, owner/financial acceptance, native primary-platform
execution and the full M7/release gates remain open.
