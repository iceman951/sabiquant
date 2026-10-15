# Local modern CDS date validation

> Snapshot of the modern CDS tranche. Counts of labels, types and error
> variants below are as of that tranche. Automatic OldCDS has since been
> added under [M7_OLD_CDS_CONTRACT](M7_OLD_CDS_CONTRACT.md); see
> [M7_OLD_CDS_VALIDATION](M7_OLD_CDS_VALIDATION.md).

Completed on 2026-10-15, Asia/Bangkok, on native macOS arm64 against the
local worktree based on `1fde1c1`. This is engineering evidence for the
[adopted modern CDS contract](M7_CDS_CONTRACT.md), not full M7, financial,
owner or release acceptance. Work remained local; no push or CI run forms
part of this report.

## Implemented scope and review

The quoted-maturity resolver explicitly selects Quarterly2009 or
Semiannual2015. It preserves stored tenor identity, returns a distinct
Matured outcome, and constructs only required final dates. The separately
fallible standard Following plan preserves trade, supplied raw maturity,
calendar and regime; it selects payments strictly after unadjusted trade plus
one day, accrues ordinary periods between adjusted boundaries, and includes
raw final maturity. These date operations do not construct premium amounts,
protection coverage, accrued rebates, upfront settlement or a priced product.

Five immutable public types raise the declared-type census from 66 to 71.
All 71 have the central public trait checks. The nine constructible generation
labels include CDS and CDS2015. Ordinary builders reject those plan labels;
OldCDS remains a required subsequent automatic operation. The root error
catalog covers all 101 current variants, stable kinds, formatting failures,
transparent wrappers and typed source chains.

The root authored the plan and public integration; another agent authored the
maturity implementation. Independent plan/calendar tests and external-oracle
expectations were written without reading production implementation. The root
reviewed the maturity code and expanded the literal primary table. The
[plan implementation review](M7_CDS_IMPLEMENTATION_REVIEW.md) records its
independence limits, proofs and resolved first-pair-before-allocation finding.
The [maturity oracle report](M7_CDS_MATURITY_ORACLES.md) fixes construction,
counts, hashes and reproduction. Proposed design documents remain historical
inputs; the frozen contract governs actual implementation.
The subsequent [maturity implementation review](M7_CDS_MATURITY_IMPLEMENTATION_REVIEW.md)
found no correction required. Its author generated and froze the independent
oracle before inspecting production; the report explicitly records that
chronology and does not claim continued blindness after source review.

## Numerical and calendar evidence

All new comparisons use exact E0 for dates, outcomes, counts and inclusion;
there is no floating-point tolerance in these operations.

| Retained suite | Evidence |
|---|---|
| `schedule_cds_maturity.rs`, 8 tests | Literal final-FAQ table: 21 tenors, six windows, both endpoints, 252 comparisons plus 72 stored-Year aliases; strict old-regime example and published old/new five-year pair. Independent candidate enumeration checks 1,753,164 outcomes over all 146,097 cycle days, two regimes and six quotes. MIN/MAX, huge/invalid stored tenors, identity, typed causes, threads and 96/96 existing modern reference calls are checked. |
| `schedule_cds_maturity_oracles.rs`, 1 test | 148,688 rows, 148,512 distinct stored inputs, 297,376 regime comparisons and 2,154 additional successful alias comparisons. Every cycle day appears under both regimes with rotating tenors, plus domain/civil/roll-boundary controls. Exact final-target Date causes are retained. |
| `schedule_cds.rs`, 17 tests | 19,200 independently enumerated quarter/calendar/regime cases over 1600..1999, exact cap and cap±1, source anchors, supplied off-run maturity, separately fallible resolver/plan composition, full 39,995-coupon civil plan, closures/collisions, 365/366-day search boundaries, TARGET spans/policies/joints/overrides, root causes, identity and threads. Added real precursor and later-payment search failures retain distinct phase/index context. |
| `schedule_cds_calendars.rs`, 3 tests | 16,384 requests across all 128 weekday masks, 16 roll anchors, four trade/maturity controls and two regimes: 14,208 plans, 2,048 no-payment outcomes, 128 no-business-weekday failures. A further 24 closure/override/joint requests yield 14 plans, two no-payment outcomes and eight collisions. Two complete primary five-coupon examples give day counts 88/94/91/91/90 and the distinct final payment/raw end. |
| `schedule_cds_closures.rs`, 1 test | A retained independent daily holiday-set diagnostic checks 4,312 requests: 3,490 complete successful coupon vectors, 647 collision rejections and 175 no-payment outcomes. Error context is also checked. This matrix uses Quarterly2009 and maximum cap; separate tests own regime identity, TARGET and exhaustion. |

The source table was inspected through indexed text of the final December 10,
2015 ISDA FAQ after direct PDF retrieval failed. All six windows are in that
indexed final table; the directly inspected Standard Model news page
corroborates nine tenors in its first four windows. The directly inspected
Standard CDS Examples supplies the 2009 payment/accrual anchors. Source roles
and limitations are recorded in the contract/design/review documents. These
facts do not establish a full legal-document review or proleptic trading
practice. Existing 144 maturity observations remain 96 modern comparisons
plus 48 excluded OldCDS calls, not 144 modern matches.

## Aggregate native checks

The explicit toolchain directory was placed first on PATH for every Rust
command:

```text
/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin
rustc 1.99.0 (b940084d7 2026-09-28)
cargo 1.99.0 (5f94df478 2026-08-27)
```

Final native debug/release and Clippy used separate target directories under
`/private/tmp/sabiquant-m7-cds-verified-{debug,release,clippy}`. Production and
packaged files were stable before those runs. Both full logs contain all
63 successful result groups, zero failed/ignored tests, no compiler warnings
or errors, and the completed doctest summary.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked`, warning-denied rustdoc | 555 passed: 80 unit, 445 integration, 30 doctests |
| `cargo test --offline --locked --release`, warning-denied rustdoc | 554 passed: 79 unit, 445 integration, 30 doctests |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Completed successfully |
| `cargo fmt --all --check`; `git diff --check` | Exit zero |
| `python3 validation/generator/cds_maturity_oracles.py --check` | Exit zero; complete body/provenance reproduced without regeneration |
| Warning-denied `cargo check --all-targets --target x86_64-unknown-linux-gnu` | Exit zero; compile-only |
| Warning-denied `cargo check --all-targets --target x86_64-pc-windows-msvc` | Exit zero; compile-only |

The one native profile difference is the inherited debug-assertion-only test.
Doctests include 27 executable examples and three compile-fail examples.
Cross-compilation is not native Linux or Windows execution.

Earlier attempts are not substituted for this final evidence. The first
aggregate run found the newly recognized CDS labels entering an incompatible
ordinary reference consumer; it now explicitly classifies the unchanged
generic-start observations outside the modern trade-date plan domain. An
ordinary catalog test needed its new nine-label count. A doctest used the
wrong root Error path and was corrected. A shared-target rebuild also removed
a debug rlib during doctests; final runs used isolated directories and stable
sources. All affected checks are superseded by the final successful runs.

## Production-region instrumentation

The matching LLVM tools and `validation/coverage.py` executed 61 instrumented
unit/integration binaries: 525 tests passed, zero failed/ignored. The runner
unions LLVM source regions and excludes embedded unit-test modules. It does
not instrument doctests or certify branch coverage, exhaustive input coverage,
financial correctness or Rust elementary-function accuracy.

| Production scope | Covered / regions |
|---|---:|
| Entire crate | 8,633 / 8,856 (97.4819%) |
| Schedule parent | 1,136 / 1,177 |
| Modern CDS plan | 316 / 330 |
| CDS maturity resolver | 102 / 102 |
| Existing civil-roll module | 317 / 328 |
| Four schedule files combined | 1,871 / 1,937 (96.5927%) |

All 14 uncovered new plan regions have identified limits: four map a cutoff
overflow excluded by trade < representable maturity; four map impossible
day-20 reconstruction failure after a valid target month; two propagate
required coupon-quarter Date failures excluded by the checked first/maturity
coordinate bounds; three belong to allocation failure; one is the final full
TARGET guard's failure return after the original/precursor hull and every
Following candidate already succeeded. The guard remains deliberate defense.
The root added the two reachable calendar failures revealed by the first
coverage run, then reran instrumentation. No forced OOM or fabricated invalid
private state was used. The inherited parent 41 and civil-roll 11 uncovered
regions retain their earlier proof/allocation classifications.

## Actual package and repository integrity

`cargo package --offline --locked --allow-dirty` produced an actual archive,
then a fresh extraction was checked with unit tests, doctests, strict Clippy,
warning-denied public/private docs and metadata. Every command in the package
evidence has exit zero. Extracted unit tests pass 80; doctests pass 30. The
archive contains 38 regular files: 36 byte-checked originals and two Cargo
generated files. The allowlist differs from the previous actual archive only
by `src/schedule/cds.rs` and `src/schedule/cds_maturity.rs`; no research,
fixture, integration-test or external-generator payload was added.

There are no duplicate names, links, absolute paths or parent traversal.
All 36 originals were rechecked against the stable worktree after verification.
The archive is 161,748 bytes compressed and 710,383 bytes of file content.
Metadata has one package, zero dependencies/features/build scripts and
`publish = false`; both license files retain their bytes. Cargo's 59 expected
omitted-integration-test notices are package-selection notices, not extracted
source warnings. Every extracted validation log has zero warnings. The
precommit VCS record correctly identifies baseline `1fde1c1` with dirty=true.

All 75 prior fixture/generator files at the baseline remain byte-identical.
The current parsed inventory is 1,485,312 rows in 53 fixture files. Rows may
contain repeated controls or multiple outputs; they are not 1,485,312 distinct
financial examples. New fixture/design/contract/generator pins were verified.

## Reproducibility pins

Temporary evidence is local to this machine; retained tests, fixtures,
contracts and generators are the repository reproduction inputs.

| Artifact | SHA-256 |
|---|---|
| `src/schedule.rs` | `b6cd16a27cb4e89e5679baebe0e33c4be94670d3ed4c6e2af69d8638125e88a5` |
| `src/schedule/cds.rs` | `a647b2408b6948ae2f3bc16b80d5dd1334f037c807bef175b2edd8622b870767` |
| `src/schedule/cds_maturity.rs` | `e721e12dde2bbcb4e57989e3a191621d65c4862379ebd221ca66a102411dd28a` |
| `src/error.rs` | `4803233181ed884ab45c4d3e677212077a4fd7fbd5748f1a3b79d9883f064683` |
| Native debug log, `/private/tmp/sabiquant-m7-cds-verified-debug.log` | `4688e3b2d885ff2cd08625812a2724ed024a99e3c450c9256e23ab6b989efbee` |
| Native release log, matching `-release.log` | `b2baf44f13a8b0c8b68a7a808527ef2302809e662275d10c994ffa78131d0021` |
| Strict Clippy log, matching `-clippy.log` | `0e8ddee377b0eb9edd2759873e2a71ff048c23410fed58817b8f2783ca9d43b6` |
| `/private/tmp/sabiquant-m7-cds-verified-coverage/production-regions.json` | `8ce5c7f8bc98d6dff8078b35bff89b6e0c1bd9a1d29a61e31248bb1acfd30a7e` |
| Actual `.crate` archive under `/private/tmp/sabiquant-m7-cds-package/target/package/` | `aa0a37c3ee7a862f36280fad0e7a150ced7bc4a26973dfc2d40ca86028f66274` |
| `/private/tmp/sabiquant-m7-cds-package/evidence.json` | `5f6854ec6baf7678adb3a45fc0b8d2858dad4377cbde057c68772ed80b855d27` |

## Remaining work

Automatic OldCDS remains explicit F-SCHED-01 work. Its generic start role and
historical thirty-day boundary are not settled by the
[legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md); the
[proposed local engineering design](M7_OLD_CDS_LOCAL_DESIGN.md) does not itself
implement or financially accept it. NI-01 settlement, protection-date/product
models, premium-leg adapters/pricing, owner/financial acceptance, native
primary-platform execution and full M7/release gates remain open. This
completed modern date tranche does not redefine those obligations.
