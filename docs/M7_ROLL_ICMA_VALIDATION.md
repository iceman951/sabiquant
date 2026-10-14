# Local M7 same-roll ICMA validation

This report concerns the [adopted same-roll contract](M7_ROLL_ICMA_CONTRACT.md)
after civil-roll baseline `af4fa73`, on local branch `feat/m7-roll-icma`.
It extends analytical reference generation to all four existing civil rules.
It does not establish standard CDS conventions, full M7 acceptance or release
approval. The earlier roll/ordinary-ICMA reports retain their saved snapshots.

## Implementation and review

The private `IcmaLattice` dispatch shares the existing bounded lower-then-upper
hull construction, cap checks, calendar guard and context constructor.
Ordinary date/EOM points still call `checked_anchored_date` with their original
arguments. Roll points use the retained original anchor year/month, checked
signed month arithmetic and a fresh Wednesday/day-20 selection in each month.
All four current builders place q(0) inside the minimal reference hull; this
invariant justifies starting at q(0), including same-month ranges entirely
before or after it. No payment or requested endpoint replaces a raw endpoint.

The temporary unreleased `RollIcmaUnsupported` variant/kind was explicitly
removed. There remain 66 public declared value/error types and 92 unique nested
root-error variants. Existing Date errors preserve exact q(0), signed month
offset, generation index and `ICMA reference extension` phase. Reconstruction
of the privately valid anchor additionally retains a proof-excluded typed
wrapper with index 0 and phase `ICMA reference anchor`; no manufactured invalid
provenance is used to exercise it. Caps remain operation budgets, not context
identity. Source schedule provenance remains distinct from context equality.

The root reviewed the production diff and the separate implementation reviewer
checked the adopted contract and anchor/minimality proof. Neither found a
concrete production defect. This is technical review, not external financial
or owner acceptance. Final independent hand/oracle review is recorded below.

## Independent evidence and aggregate checks

The final native macOS arm64 full suites terminated with exit zero:

| Profile | Unit | Integration | Doctest | Total |
|---|---:|---:|---:|---:|
| Debug | 80 | 415 | 29 | **524** |
| Release | 79 | 415 | 29 | **523** |

Each log contains 58 result groups, zero failed/ignored tests and zero warning
lines. The inherited debug-assertion test accounts for the profile difference;
29 doctests include three compile-fail cases. Formatting, strict all-target
Clippy and warning-denied Linux GNU/Windows MSVC all-target checks passed.
Cross-target checks are compile-only, not native execution. Public/private
warning-denied documentation also passed on the actual extracted source.

The new **14-test public hand suite** covers 21 eligible rule/frequency pairs,
six same-month Wednesday controls, nine exact required-date cases, Year/Month
cause equivalence, every insufficient cap on literal hulls, all 196 adjustment
pairs on the chosen four schedules/custom calendar, 56 TARGET extension
policy/override/joint cases, inception and resolved-endpoint controls, generic
shared-context coupon composition and 256 threaded rebuilds. Full-period
Regular accrual equals the explicitly declared frequency divisor.

It also constructs the actual maximal exact-endpoint month-roll hulls:
119,987 reference periods for each Wednesday rule and Twentieth; 39,995 for
TwentiethIMM. Every date/adjacency is inspected, with independent Gregorian
ordinal Wednesday checks, exact cap/cap−1 and full-span accrual. Unnecessary
year10000 successors are never required at these exact endpoints.

The [independent oracle](M7_ROLL_ICMA_ORACLES.md) contributes one integration
consumer, **42,647 context records** (42,642 distinct tuples), **87,558 exact
periods** and **126,537 rational queries**. The final focused native debug
run measures a maximum **1 ULP**, within the unchanged four-ULP ICMA profile.
There are 42,179 successful contexts, 260 required-date failures, 188 mismatches
and 20 Once rejections. All 4,800 Gregorian-cycle months appear under all four
rules and two rotating profiles, not every endpoint/frequency cross-product.
Generator `--check` and final focused tests both terminated with exit zero.
The aggregate release run includes this same frozen consumer and fixture.

The [source/test review](M7_ROLL_ICMA_REVIEW.md) maps FR-SCHED-14..17 and 21..23
to inspected implementation and meaningful cases. All 73 prior fixture/generator
artifacts remain byte-identical. This tranche's inventory is **1,336,624 rows
in 52 fixture files**; rows are not all distinct financial examples.

| Saved native log | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-m7-roll-icma-final-debug.log` | `397c0ca808e837af69914de9af36320b1fe450e68932e40cfb1a38c4a8d88269` |
| `/private/tmp/sabiquant-m7-roll-icma-final-release.log` | `ffda377744a11315108c1163ea9d4c7b7000812c54ff3c199c5e3dcd5bd35232` |
| `/private/tmp/sabiquant-m7-roll-icma-final-focused.log` | `1748934a0e13172fb5ebeb23beab742c1fbf30ebc81b18242765f371f5531ab5` |
| `/private/tmp/sabiquant-m7-roll-icma-final-clippy.log` | `fcfa30fb819c4580d0801e4cdfaa85a841fbf026ab2052567918f9c92747755f` |

Fresh LLVM source-region coverage terminated with exit zero. All **56**
instrumented unit/integration executables passed **495 tests**, with no failures
or ignored tests. The runner excludes embedded unit-test source and unions
instantiations; doctests are checked separately.

| Production file | Covered / total regions |
|---|---:|
| `src/schedule.rs` | 1,069 / 1,110 |
| `src/schedule/roll.rs` | 317 / 328 |
| Combined schedule implementation | **1,386 / 1,438 (96.38%)** |

The roll module adds 32 regions, all covered, and retains the same eleven
uncovered allocation/invariant-only regions as the previous snapshot. Parent
schedule uncovered regions increase from 38 to 41: the three new regions are
the privately valid anchor reconstruction's Date-error closure and propagation
(lines 1004, 1672, 1673 at the pinned source). No public input reaches these
branches. Existing ordinary required-Date failures and both roll directions
are exercised. The unchanged ICMA kernel remains 163/164 regions and the
cash-flow implementation remains 688/703. No forced OOM or private-state
corruption was introduced to improve the percentage.

Coverage JSON:
`/private/tmp/sabiquant-m7-roll-icma-final-coverage/production-regions.json`,
SHA-256 `6358593e2bef46f012cec42f3fec257ed45243b922596131c6b0dee17b0e2883`.
These are source regions, not branch coverage, exhaustive-input evidence or
proof of financial accuracy. The percentage denominator changes with source.

## Actual extracted package

The root created a fresh actual Cargo archive using offline, locked commands
and Cargo's default package verification. It contains **36 regular files**,
**153,923 compressed bytes** and **670,882 file-content bytes**. All **34
original members** match workspace bytes; the other two are Cargo's normalized
manifest and VCS metadata. Member paths have no traversal, duplicates or links.
The source snapshot is `af4fa73`, dirty=true, before the local ICMA commit.

Archive path:
`/private/tmp/sabiquant-m7-roll-icma-package/target/package/sabiquant-0.0.0.crate`.
SHA-256: `36bb093c5b9f8155505732b9d1ae4eb69392f5c703d64c7078ca7ede5ae4df72`.

A fresh extraction with a separate target directory passed **80 unit tests**,
**29 doctests** (26 ordinary, three compile-fail), strict all-target Clippy and
warning-denied public/private rustdoc. Every command terminated with exit zero.
Extracted checks emitted no warnings; package notices concern omitted
integration tests. Metadata confirms one unpublished version 0.0.0 package,
zero dependencies/features/build script, edition 2024, Rust 1.99 and both
byte-preserved licenses. No integration fixture, generator or research file
is required by the archive. Packaged originals were unchanged after checking.

Detailed commands, log hashes and original member hashes are saved in
`/private/tmp/sabiquant-m7-roll-icma-package/evidence.json`, SHA-256
`e17be38eac45d26d93777d5db24ea30bbb321c74cc47f06bd5bc4d37caf6c1e3`.
The archive/evidence under `/private/tmp` are local audit artifacts. This is
native macOS arm64 verification, not publication or native Linux/Windows testing.

## Frozen production pins

| File | SHA-256 |
|---|---|
| `src/schedule.rs` | `21a183ed6a7b86cbb837b2601be2f1843fe55ba78d165e901231a0bfd46d603b` |
| `src/schedule/roll.rs` | `0bc794c20615b55e6f9f576919765152ae0e42f8b05885badbb00b9be4a980fe` |
| `src/error.rs` | `1ecee9df1e9a9516fae765c3b43f2374cea2cd8562acb4add549e9700ad0c02e` |
| `tests/schedule_roll_icma.rs` (14 tests) | `6b25bc1b9021785d9c472c9fee218cd9bc7dd78639824daab0e08f6689e4e0de` |
| `tests/schedule_roll_icma_oracles.rs` | `ba4de96ac80666146bf47efe823b747cd4189fb9acb4f284b5085692ca58f944` |

## Reproduction and open scope

Use the declared native Rust1.99 toolchain; library/tests remain std-only and
all Cargo operations run offline. Python is external oracle/coverage tooling.

```sh
python3 validation/generator/schedule_roll_icma_oracles.py --check
cargo test --offline
cargo test --offline --release
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps
RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --document-private-items
python3 validation/coverage.py --output NEW_DIRECTORY
```

OldCDS/CDS/CDS2015, CDS maturity/product/source conventions, NI-01 exact
settlement and financial/owner/native-platform/release gates remain open.
No source-region percentage, finite oracle corpus or successful generic
coupon composition is a universal accuracy or market-product claim.
