# Local bond core validation (`sabiquant::bonds`)

The core implementation was completed from `8bfa292` and committed as
`5ea07cd`. This report incorporates the subsequent local evidence audit on
2026-10-16, Asia/Bangkok, on native macOS arm64. The frozen
[bond core contract](M8_BOND_CORE_CONTRACT.md) has SHA-256
`009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c`.
No production Rust source changed during the follow-up audit. Its corrections
concern test coverage, independent numerical evidence and the scope of claims.
There was no push or remote CI run.

The complete M8 objective remains open: bounded yield inversion, analytic
first/second derivatives, duration/convexity, named market models, and NI-01
financial settlement adapters. This is engineering evidence, not owner API,
financial, native Linux/Windows, milestone or release acceptance.

## Implemented scope and retained suites

The module retains twelve immutable public types for zero/fixed bullet bonds,
explicit settlement/payment inclusion/ex-coupon boundaries, clean/dirty quotes,
entitlement, convention, prepared pricing and valuation. Two free functions
convert exact supplied binary64 amount/face ratios per100, rounded once.
There are 89 public types and 125 domain error variants in the whole crate;
all 89 actual declared public types match the central trait-assertion census.

| Suite | Actual scope |
|---|---|
| `bonds_core_oracles.rs`, 1 test | Every 18,152 frozen row: 16,656 exact Fraction quote rows and 1,496 plan/status rows. Exact discrete/accrued fields; at most 2 same-sign ULP steps against retained binary64 model prices. Body/source pins,18,128 distinct input tuples, sequential IDs, unused quote cells and profile/op/status censuses are verified; duplicate context/census keys are rejected |
| `bonds_core_hand.rs`, 7 tests | Constructor precedence; 49 schedule-convention pairs; supported schedule families; contextual-day-count and32-compounding smoke checks; targeted sum/coverage/work/thread checks. Contextual numeric smoke is not an independent high-precision oracle for every composition |
| `bonds_core_edges.rs`, 5 tests | Valid full minsubnormal coupons with signed partial underflow; work-before-underflow and preparation-before-NaN order; typed causes/full quote error fields; cached start/full/ceased paths; distinct contractual/redemption payment roles; successful reopening and exact cap/cap−1 across128 weekend masks |
| `bonds_core_review.rs`, 7 tests | 49,104 exact quote assertions across every finite normal exponent and midpoint parities; 32 compounding configurations under same-factor signed cancellation; negative ex-coupon liabilities; zero-amount factor validation; ICMA cap/context/source ordering; successful reopened Business/252 cap equality; staged-versus-ideal cancellation discriminator |
| `bonds/quote.rs`, 2 unit tests | Intermediate overflow/underflow rescue, subnormal/normal midpoints and carries, top-binade overflow carry |
| Existing amount oracle consumer | All277,577 phase A rows and276,760 distinct inputs now additionally verify frozen body/source pins, profile/op/status/error-operation censuses, IDs, active/unused fields, mode context and duplicate-key rejection |

The original [bond oracle report](M8_BOND_CORE_ORACLES.md) now records actual
profiles. The frozen generator's Decimal loops checked only signs, and plan
factors used Python binary64. Those are not high-precision value checks.
The original reopening profile has 24 errors and no successful plan; the new
public regressions supply successful exact-cap cases. Every one of the 95
previously tracked fixture/generator files is byte-identical to `5ea07cd`.

## Additive independent Decimal evidence

The new [audit](M8_BOND_CORE_DECIMAL_AUDIT.md) independently decodes supplied
bits with integers/Fraction, calculates quote values at 256/512 Decimal digits,
and evaluates plan factors/prices at 160/240 digits. It inspects frozen
expectations only after constructing answers. It has no Rust or QuantLib
runtime/source dependency and leaves the original expectations unchanged.

Actual checks cover all 16,656 quote inputs, 33,048 complete Decimal quote
bits/status comparisons and 7,122 valuation-quote comparisons. All 1,187
successful frozen plans agree with independent staged bits at both precisions;
entitlement/accrual/work/error fields agree exactly. There are no unexplained
mismatches. The new 1,497-row audit fixture contains all 1,496 plan inputs plus
one signed-cancellation discriminator and is reproduced by Python `--check`.
It is not directly consumed by a new Rust fixture suite.

Here ideal price means high-precision discounting with supplied rounded coupon
and year-fraction bits held fixed. It does not mean a financial ideal using
unrounded coupon products or raw exact DCC fractions. The measured M4 E3
criterion remains conditional, not a libm worst-case theorem. Production
linkage combines each ideal-versus-frozen gross-conditioned absolute allowance
with the exact two-bit-step radius admitted by the current Rust consumer.
The supplemental report retains both components and all limitations.

A same-horizon coupon −100 and redemption next_up(100), at Simple yield .25,
have staged price 2^-46 and ideal net .8*2^-46. That 25% net-relative difference
is consistent with the staged contract. A universal net-relative or few-ULP
ideal-price guarantee would be false; gross-conditioned absolute errors matter.

## Fresh native verification

Every Rust command used declared Rust 1.99.0 at
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`.
Independent runs used separate target directories; warning-denied rustdoc
was enabled. Logs and exact commands are under
`/private/tmp/sabiquant-m8-core-audit-verified/`.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked` | 698 passed in 80 groups: 95 unit, 563 integration, 40 doctests (35 executable/5 compile-fail); 0 failed/ignored/warnings |
| `cargo test --offline --locked --release` | 697 passed:94 unit, 563 integration, 40 doctests; 0 failed/ignored/warnings; one inherited debug-assertion-only difference |
| `cargo clippy --offline --locked --all-targets -- -D warnings` |Exit 0 |
| `cargo fmt --all --check`; `git diff --check` |Exit 0 |
| Public and private `cargo doc --offline --locked --no-deps` |Exit 0 with warnings denied |
| Warning-denied Linux GNU x86_64 and Windows MSVC x86_64 `cargo check --all-targets` |Exit 0; compile-only, no native runtime claim |
| Frozen `bond_core_oracles.py --check` |Byte-identical18,152-row fixture |
| Additive `bond_core_decimal_audit.py --check` |Complete independent arithmetic/bound/pin comparison; byte identity required |

## Production-region instrumentation

A fresh unchanged `validation/coverage.py` run executed 78 instrumented binaries
and 658 runtime tests, all passing. Doctests are separate. The production report
excludes embedded tests and unions source regions across instantiations.
This is neither branch/exhaustive-input coverage nor financial certification.
The earlier 16,313/16,726 figure was a raw LLVM table total including embedded
test/generic regions; it must not be labelled the production-only total.

| Production scope | Covered / regions |
|---|---:|
| Whole crate, production-only union | 11,271 / 11,567 (97.4410%) |
| `src/bonds.rs` | 770 / 787 (97.8399%) |
| `src/bonds/quote.rs` | 139 / 139 (100%) |
| `src/error.rs` | 388 / 391 |
| `src/amounts.rs` | 724 / 753 |
| `src/amounts/binary.rs` | 267 / 285 |
| `src/amounts/uint.rs` | 302 / 304 |

The 17 uncovered bond regions comprise 9 allocation-failure propagation regions,
3 ex-coupon reduction regions, one partial day-count propagation, and 4
nonnegative finite horizon-result validation regions. Earned and cached full
amounts share the coupon-rate sign; their difference cannot overflow when each
is finite. Ordered valid-context subintervals preserve the guarded accrual
coverage and finite nonnegative year fraction. These are qualified invariant
paths, not proof that every partial calculation is infallible: partial product
underflow is reachable and now tested. No OOM or invalid private state was
forced. Other modules retain their documented inherited gaps.

## Actual extracted-package integrity

An actual `cargo package --offline --locked --allow-dirty` archive was built
with normal verification, extracted, and checked independently under
`/private/tmp/sabiquant-m8-core-audit-package/`. It contains 46 regular files,
44 of which were byte-checked against original packaged inputs; the other two
are Cargo-generated metadata. The only additions over the phase B archive are
`src/bonds.rs` and `src/bonds/quote.rs`. There are 201,877 compressed and 893,323
content bytes, archive SHA-256
`4b087d29cdb02a0541290a2cf9b4f49528ad764d7afc645e3c3a2f0047a3df93`.
The extracted crate passed 95 units, 35 executable and 5 compile-fail doctests,
strict Clippy, public/private warning-denied docs and metadata checks.
It still has no dependencies/features/build script and `publish=false`.
Research, fixtures and integration suites remain outside the archive.

## Current reproducibility pins

| Artifact | SHA-256 |
|---|---|
| `docs/M8_BOND_CORE_CONTRACT.md` | `009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c` |
| `src/bonds.rs` | `11d2e7d7f321a3c1a19e66d6842dc3d1f77d19e9a929f0316be62ce61b5e3599` |
| `src/bonds/quote.rs` | `fde910b6cebb4fc1df4a316d9d0a526e8e78052a8ccaff47bd7a1b2bcc5cc8a3` |
| `validation/generator/bond_core_oracles.py` | `afe27c0ce1a362492a98202c2882271cabbc3b1d510bb49dd3d5215460055b18` |
| `validation/fixtures/bonds/bond-core-oracles.fixture` | `5a5dc4f4ec573d189b8b7f87aeaf335b5a253f1ad983633612dfbe2ae8ae7817` |
| `tests/amounts_oracles.rs` | `86889e392448b9b0f427c81125267a04fa2312c2b914bd23030e87d312521ebb` |
| `tests/bonds_core_oracles.rs` | `180b752c6f7907957d345add796405ad98d13586361a82e4708fc9c2a4e375e7` |
| `tests/bonds_core_edges.rs` | `92d0f870a6a0eca18b728732ed16c264edc9628170e5805a9a240bfca3f2a57f` |
| `tests/bonds_core_review.rs` | `30183ae1586611c2a617e3f213f9c884e41158fca57ac402a9ffe2aa5e38df0b` |
| `/private/tmp/sabiquant-m8-core-audit-verified/evidence.json` | `92d132902abc00f8f36f4a1b34dcfafb6055e9ed36aee5a9f036e459dd3610c6` |
| `/private/tmp/sabiquant-m8-core-audit-coverage/production-regions.json` | `9acd1fe4775081f17d1086270a3f1319462bf0edc0a9b2263e131c90e0aa7edc` |
| `/private/tmp/sabiquant-m8-core-audit-package/evidence.json` | `b29e9e2fa4ddf2de220923185f3b1de675aa01010067c99b421e67915192bfb0` |
| `validation/generator/bond_core_decimal_audit.py` | `7cae8cfc0583003c10c62df45750ba8a6e0d1311e0aaaaf3d4a5b901a41e3f56` |
| `validation/fixtures/bonds/bond-core-decimal-audit.fixture` | `c888d7b94be5ad367d7bc12ccab726fa5ca74939bb0b40ad9b726dd1ca58ad19` |
| `/private/tmp/sabiquant-m8-core-fixture-inventory.log` | `b3cb92141195c511e8d5152a358e57947a6110d35deaea122c9e795747101a5f` |

The unchanged shared Rust fixture reader additionally parsed all 63 fixture
files successfully: 2,762,743 total rows, including the new audit's required
schema-1 tolerance header. File links were checked for existence separately;
that check does not validate every external URL or Markdown anchor.
