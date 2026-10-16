# Local US 30/360 and AFB validation

Completed on 2026-10-16, Asia/Bangkok, on native macOS arm64 against the
worktree based on `1c16daf`, branch `feat/m3-us-afb-daycounts`. This records
local engineering evidence for the [adopted contract](M3_US_AFB_CONTRACT.md).
Financial, owner, milestone, native-platform and publication acceptance
remain separate; the owner's local-only instruction is retained.

## Implemented scope and independence

`Thirty360Us::new(end_of_month_security)` requires explicit immutable security
qualification, applies the four ordered dated ROC December 2025 A001 rules,
normalizes reverse intervals and computes one rounded division of the exact
signed count. Both flag definitions retain distinct Eq/Hash identity.
`DayCount::ActualActualAfb` follows ISDA's published backward-year February
clarification, with original-end preservation at N=0, actual half-open residual
leap membership and fixed residual-division-then-whole-year-addition order.
Its production calculation reduces the annual recurrence to O(1) integer
arithmetic without an out-of-range sentinel. Both compose with rates,
equivalent-rate targets, plain coupons and bounded legs, charging one work
unit per query. There are six scalar and four contextual day-count definitions,
73 public types with matching central trait assertions, and 103 error variants.

The integrator wrote AFB and its parent integrations; a separate agent wrote
US and its hand suite. Two other agents independently authored the oracle
generators and consumers without reading these production implementations.
A separate reviewer checked AFB and wrote the retained private decomposition
oracle. The integrator completed the reference lower-bound classifier,
reference consumer, composition suite and current documentation after the
delegates hit a usage limit. Independence claims apply to the stated artifacts,
not every line of this tranche. Findings and resolutions are recorded in the
[implementation review](M3_US_AFB_IMPLEMENTATION_REVIEW.md).

## Retained evidence and requirements

| Suite | Actual scope |
|---|---|
| `daycount_us.rs`, 9 tests | Hand rule distinctions; both flags; supported-year month-end matrices; leap/century neighborhoods; every Gregorian-cycle day; full civil limits and deterministic wide pairs; every supported equal date and collapsed 30th/31st pair; context identity and concurrent reuse |
| `daycount_us_oracles.rs`, 2 tests | All 393,158 category-table/Fraction fixture rows, 392,772 distinct flag-bearing inputs; exact integers and zero-ULP fractions. Separately, all oriented pairs in 1999..2002 and 2099..2101 under both flags: 6,667,092 queries, each checking count and fraction (TS-DCC-07) |
| `daycount_afb.rs`, 7 tests | All 215,327 repeated-year fixture records, 205,673 distinct chronological/equal pairs; a separate manual Rust repeated-year/residual-day oracle checks all discrete fields before public fractions. Five actually printed ISDA anchors, derived February/civil boundaries, exact identifiers, nonadditivity/anniversary counterexamples and sixteen-thread reuse |
| Same AFB suite, exhaustive test | All 1,668,051 chronological/equal pairs in the two required TS-DCC-07 windows plus each reverse: 3,333,546 distinct oriented inputs. Equal queries are evaluated twice by the symmetric loop. Independent annual rollback/leap-date/ordinal expectations; 1,268,318 accepted yearly moves, maximum 1 ULP |
| Embedded AFB units, 2 tests | Ten hand decomposition tuples plus 346,198 retained independent iterative/day-walk profiles comparing the actual private discrete tuple before floating reduction; every cycle start, twenty profiles per supported year and an 11×11 endpoint set |
| `daycount_us_afb_reference.rs`, 1 test | All 741 attested public observations on 247 distinct ordered pairs reclassified from raw counts/bits/errors, independent repeated-year annotations and explicit discrepancy predicates; complete runtime/input/class censuses and byte fingerprints |
| `daycount_us_afb_composition.rs`, 4 tests | Hand-derived fractions through all 32 compounding configurations, source/target accrual conversion, both US flags retained at zero/equal horizons; raw versus adjusted-payment roles, partial accrual, exact and one-short budgets, full civil spans and zero scalars |
| Existing integration/unit/doc suites | Identifier catalogs, 14 context-hash definitions, 352 concurrently reused rate quotes, 73 public trait assertions, 103 error kinds/source chains, and all previous financial/numerical/calendar/schedule/package policy checks |

The [US oracle report](M3_US_ORACLES.md) and
[AFB oracle report](M3_AFB_ORACLES.md) state exact rotating profiles and
repetitions. Daily cycle anchors do not imply every anchor/profile combination
or all civil pairs. AFB maximum observed error was **1 ULP** against the exact
rational; its existing **4-ULP** limit is unchanged. US exact-rational fractions
use zero ULP. Composition elementary results use the existing conditioned
E3/E4 policies. No tolerance was widened.

The new [reference corpus](M3_US_AFB_REFERENCE_PROBES.md) contains **602
compatible observations at zero observed ULP**, **92 US direct reverse-count
differences** and **47 AFB year-1900 reference-anchor errors**. No result is
unclassified and no raw observation was rewritten to force agreement.
USA compares to flag on; BondBasis is a flag-off behavioral comparison only.
The dated rules and unread complete SIFMA source remain separate from those
observations. The matching pinned public wheel and missing temporary wrappers
were restored and attested before this run; no C++ implementation was read.

## Aggregate native checks

Every Rust command used the explicit Rust 1.99 toolchain directory first on
PATH: `/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`.
Observed versions were `rustc 1.99.0 (b940084d7 2026-09-28)` and
`cargo 1.99.0 (5f94df478 2026-08-27)`. Independent target directories avoided
cross-run interference. Logs and structured evidence are under
`/private/tmp/sabiquant-m3-us-afb-verified/`.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked`, warning-denied rustdoc | 619 passed in 72 groups: 84 unit, 502 integration, 33 doctests; no failures/ignored/warnings |
| Same command with `--release` | 618 passed: 83 unit, 502 integration, 33 doctests; no failures/ignored/warnings |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0 |
| `cargo fmt --all --check`; `git diff --check` | Exit 0 |
| Warning-denied public and private `cargo doc --offline --locked --no-deps` | Exit 0 |
| Warning-denied `cargo check --all-targets` for x86_64 Linux GNU and Windows MSVC | Both exit 0; compile-only |
| All three new generator `--check` commands | Actual regeneration completed, exit 0; public probe check re-called all 247 isolated workers |

The inherited debug-assertion-only test explains the one-test debug/release
difference. Cross-compilation is not native Linux or Windows execution.

## Production-region instrumentation

The unchanged `validation/coverage.py` and matching LLVM tools executed
**70 instrumented unit/integration binaries**, **586 tests**, with no failures
or ignored tests. The exported production-region union excludes embedded unit
test source. It is not branch, exhaustive-input, doctest or financial coverage.
Reports are under `/private/tmp/sabiquant-m3-us-afb-coverage/`.

| Scope | Covered / regions |
|---|---:|
| Entire crate | 9,002 / 9,232 (97.5087%) |
| `daycount.rs` | 321 / 321 |
| `daycount/afb.rs` | 87 / 87 |
| `daycount/us.rs` | 64 / 64 |
| `daycount/icma.rs` | 163 / 164 |
| `rates.rs` | 786 / 794 |
| `cashflows.rs` | 513 / 527 |

Every reported production region in the two new modules was covered. The
additional uncovered rate-adapter region at `rates.rs:314` is propagation of
a US finite-result failure that valid Date components cannot produce: the
absolute integer count is at most 3,599,640 and division by 360 is finite.
No invalid private state or forced OOM was used to reach such a guard. Existing
uncovered allocations, invariant-excluded failures and prior numerical paths
retain their earlier limits; coverage is not represented as a proof of them.

## Package, frozen evidence and source limits

An actual `cargo package --offline --locked --allow-dirty` archive was freshly
extracted. That extraction passed **84 unit tests**, **30 executable and 3
compile-fail doctests**, strict all-target Clippy, warning-denied public/private
docs and metadata checks. The archive has **41 regular files**: **39 byte-checked
originals** and two Cargo-generated files. Its only allowlist additions over
the OldCDS archive are `src/daycount/afb.rs` and `src/daycount/us.rs`.
Compressed size is **171,159 bytes**, content **751,565 bytes**, archive SHA-256
`08d18c35060f683f289cc390176de3a8f9c399e261bd03e4cf6e06b866f1bbab`.
Metadata shows one package, no dependencies/features/custom build and
`publish = false`. Research, generators, reference fixtures and integration
suites remain outside the package.

The final file-link audit checked 906 relative links with no missing file;
all listed reproduction pins matched their current bytes.
All **83 previously tracked fixture/generator files** are byte-identical to
`1c16daf`. The independently parsed current inventory is **2,268,744 records
in 59 fixture files**; repeated controls and multi-output rows are explicit.
The source records distinguish the directly inspected ISDA 1998/1999 material
and ROC December 2025 A001 column from the unread full SIA/SIFMA book and
original September 1994 AFB publication. Full financial/milestone acceptance,
TARGET maintenance, native primary-platform CI/review, NI-01 settlement and
publication remain separate obligations.

## Reproduction pins

The retained artifacts and commands above reproduce this local tranche;
temporary logs are not shipped evidence on another machine.

| Artifact | SHA-256 |
|---|---|
| `docs/M3_US_AFB_CONTRACT.md` | `595df058cdd8e64a935b6fc992ef0ca903e7e7fbdfa4ed613bbe598a3afe10b4` |
| `docs/M3_THIRTY360_US_SOURCE_REVIEW.md` | `c90c18543f28ed0f2def9744f56ef9e397561f4f7b2d602a18e79f97158c1958` |
| `docs/M3_AFB_SOURCE_REVIEW.md` | `3b30092d8e45e01af12870cffc973a03d4fe116a48b9617a513ddf6a81a9681d` |
| `src/daycount/us.rs` | `29e03361d9bee1f2a446c69ee576662c5e73686d892d87f2f8bc8cbc57cd191c` |
| `src/daycount/afb.rs` | `847f2282554373952e0de5da9e01ad97b8349488990fc6bf83597c471ec55350` |
| `src/daycount.rs` | `38686ba19c33a4375ecddb273fd432a3208f335583e8fffbbb129ec97d0d36e7` |
| `src/rates.rs` | `6955a3b16753907bafd60ce381ab113f92c55728c1b1988b7bef7587331c07ed` |
| `src/cashflows.rs` | `6b8a8e179ed9ad43ca913decc130149598e3eaf4148e0713d9e9d5700c6babc6` |
| `validation/generator/daycount_us_oracles.py` | `c9de4d22e655ed6911d161d1c5d9888f36563d5dbcae29bbaa20b70a92c33afc` |
| `validation/generator/daycount_afb_oracles.py` | `aa2e75fe9f95e3365b934967c0abee79b2f6c9bd7e4941268f24de893e03faa7` |
| `validation/generator/daycount_us_afb_probes.py` | `79db031c315c6e68e657031312943f2346d2bfb648d3e425ac0878593ef41e1d` |
| `validation/fixtures/daycount/us-30360-oracle.fixture` | `92cb385dc1b7b6f79dcce023352cc556704a79f37f07067cf14ed95876ec425a` |
| `validation/fixtures/daycount/afb-oracle.fixture` | `00aab23ecc7fd9094355c1bbdc54e9cbdac24f98a41960683e1f123aa0b20a11` |
| `validation/fixtures/daycount/us-afb-probes.fixture` | `6ef7c19fcb991ca04ce4e0b5c8e10b5a328329850f5bec1f4c936136ccb44c86` |
| `tests/daycount_us.rs` | `6d63028566568cf98ce07440611283f8729322221088376d4bbb12acbe65cb97` |
| `tests/daycount_us_oracles.rs` | `aed5452bd78c92647cb58a48398285793b1fc5d3ba1b5173bab9062ef8d6dc7e` |
| `tests/daycount_afb.rs` | `d18a757301e6ad1163529633f42d1f4157ca4a84260b354ec875552e827603a1` |
| `tests/daycount_us_afb_reference.rs` | `8f3980ee1f85b42678605cc9c91894ac2c427652e5207a4878ad30a3eb11ad29` |
| `tests/daycount_us_afb_composition.rs` | `6af42c86b00d31dc313965cf0533f630ab2cf471bab92c16a5766dc3ec3e0a75` |
| `/private/tmp/sabiquant-m3-us-afb-verified/debug.log` | `caaa0e6b1fbe8bc453660a66508ce37067476ce801aad43c56ea628571869e55` |
| `/private/tmp/sabiquant-m3-us-afb-verified/release.log` | `11741c5c601ab826c48eb8fe0d092e08f73da20483816d30269397be6d41b592` |
| `/private/tmp/sabiquant-m3-us-afb-coverage/production-regions.json` | `ccfab3947ad20e940efa6047144ffadc3e7c234f6c384087b607f29c2fef0fa0` |
| `/private/tmp/sabiquant-m3-us-afb-package/evidence.json` | `d14185f0b16c27c9b1a5147696be60613a9448b082b2ab59cc3046d60a2f6f60` |
