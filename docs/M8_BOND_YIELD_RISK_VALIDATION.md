# Local generic bond yield/risk validation and evidence audit

Completed on 2026-10-18, Asia/Bangkok, against local `a33e8a7` plus the audit
working tree on `test/m8-yield-risk-evidence-audit`, on native macOS arm64.
The frozen [contract](M8_BOND_YIELD_RISK_CONTRACT.md) and FR-YR-01..16 remain
authoritative. Work is local only. This report supplies engineering evidence;
named market, financial, owner, native-platform and release acceptance remain
separate.

Codex ran the audit, the additive evidence and the first terminal checks.
Claude Code then checked each terminal artifact against the current bytes and
re-ran whatever no longer matched. It re-ran all three yield/risk generator
checks and the shared fixture inventory, and assessed each uncovered
production region. It also added a public failure-order regression. A
read-only adversarial review found ten evidence issues: untested branches,
unsupported or false report sentences, and imprecise coverage wording. The
suite was strengthened and the report corrected. The native checks, coverage
and kernel harness were then re-run on the final tree. The package was rebuilt
after the last change to any packaged file. A subsequent local metadata change
at `61eb382`/`c70bf5f` selected version `0.1.0-alpha.1` and removed the manifest
publication restriction. The metadata refresh below records checks against
those current bytes; it performs no remote operation or publication.

## Corrected implementation record

The former version described nonexistent errors/functions and an algorithm
with endpoint-derived scaling, binary64 intermediate risk admission and
33/35/37-limb arithmetic. Those descriptions did not match the committed
source. The [source review](M8_BOND_YIELD_RISK_IMPLEMENTATION_REVIEW.md) records
the actual target-selected scale 0/1, 34-limb exact residual/constant
arithmetic, 5-limb normalized 53-bit extended terms and 243-limb signed sums
on `2^-8192`. Fallible Brent retains point/residual/dirty payload together and
performs no final repricing. All six raw/normalized risk getters admit their
result independently. The public type/error censuses remain 101/139.

No production arithmetic changed during this audit. The 50,053 bytes of
`src/bonds/yield_risk.rs` before `#[cfg(test)]` are identical to `a33e8a7`.
The embedded tests now use valid operand-derived states; forged report
mutations and impossible extended exponents were removed. All 103 previously
tracked generator/fixture artifacts are byte-identical to `a33e8a7`; new
evidence is additive.

## Retained and additive suites

| Suite | Evidence |
|---|---|
| `bonds_yield_risk_oracles.rs`, 1 test | All 141 frozen paired scenarios. The strengthened consumer checks each actual prepared payment date, amount and context horizon, sequential IDs and distinct inputs. The [oracle report](M8_BOND_YIELD_RISK_ORACLES.md) records its 72-column schema and numerical limits |
| `bonds_yield_risk_hand.rs`, 11 tests | The unchanged blind hand suite |
| Embedded `bonds::yield_risk` tests, 5 | Sensitivity-key pairing, signed residual scaling, extended-term and wide-ratio rounding, risk accumulation/getter admission, and zero-horizon constants. Raw admission controls build retained sums from valid finite operands through the actual term rounder and accumulator |
| [`bonds_yield_risk_review.rs`](../tests/bonds_yield_risk_review.rs), 6 tests | Reachable unused-clean-view overflow, signed raw subnormal errors, finite normalized results, huge term cancellation, opposite-`MAX` finite scaled residuals, and callback/constant/budget/payload ordering |
| [`bonds_yield_risk_normalized_range.rs`](../tests/bonds_yield_risk_normalized_range.rs), 1 test | Exact-binary Simple/ICMA bond with negative dirty price, raw second-derivative underflow and Macaulay-only normalized overflow; modified duration and convexity match exact Fraction-derived bits |
| `bonds_yield_risk_extended_oracles.rs`, 2 tests | All 2,181 conditioned public risk rows across 34 profiles and all 32 compounding configurations, plus five exact controls |
| `bonds_yield_inverse_extended_oracles.rs`, 1 test | All 992 independent closed-form one-payment yield enclosures |
| [`bonds_yield_risk_failure_order.rs`](../tests/bonds_yield_risk_failure_order.rs), 7 tests | FR-YR-12/14 failure order and branch selection, written by the root from the contract and the M4 policies; it is not a blind suite. **Quote floor:** `y>-m` holds for all 30 periodic/hybrid configurations, in `risk` and the lower callback. The plans are paid, time-zero, a one-day flow (Simple-active for every `SimpleThenCompounded` frequency) and a two-year flow (Simple-active for every `CompoundedThenSimple` frequency). **No floor:** Simple/Continuous fail at the original flow index, including index 1 when coupon 0 is paid. **Branch selection:** public Actual/365 and ICMA horizons equal to `RN(1/m)` or one binary step away (`m=12,13,365`) select the contracted branch. At exact cutoffs `t=1/m` (`m=1,2,4`), `next_up(-m)` gives factor `2^-53`, an exact dirty price and six exact risk views. **Ordering:** the quote precedes zero/negative active-Simple factors. An overflowing PV inside an exactly cancelling constant plan fails at its own index before the constant comparison. `ln(MAX)` is a `NumericalBoundary`, and target conversion precedes an invalid lower quote. **Mutation check:** the suite kills all 11 single-line production mutants recorded in the mutation evidence |
| Private kernel harness, outside the crate tests | All 20,201 exact residual, term, sum and flow rows through actual private operations in a byte-attested temporary copy |

The [extended oracle report](M8_BOND_YIELD_RISK_EXTENDED_ORACLES.md) describes
the additive corpora and the kernel harness. The
[evidence audit](M8_BOND_YIELD_RISK_EVIDENCE_AUDIT.md) records why the frozen
oracle needed this additive evidence.

All three generators reproduce their fixtures with `--check`, re-run on the
final bytes:

| Generator | Result |
|---|---|
| Frozen `bond_yield_risk_oracles.py` | 141 records, 162,037 bytes, `check=True` |
| `bond_yield_risk_extended_oracles.py` | Kernel 5,258,376 bytes and public 1,988,926 bytes, both `check=True` |
| `bond_yield_inverse_extended_oracles.py` | 380,070 bytes, `check=True` |

The kernel harness was re-run on the final tree in
`/private/tmp/sabiquant-yield-risk-kernel-final-v3`. One test passed over all
20,201 rows, and strict Clippy passed on the same temporary copy
(`clippy-evidence.json` beside its evidence). Its 47-file source manifest
matched the pre-version-change bytes. Codex's earlier `final-v2` run also passed,
but `CHANGELOG.md` changed after it. The current-metadata kernel rerun below
supersedes its source-manifest attestation; every kernel expectation is unchanged.

A scalar `days/basis` horizon can equal a rounded cutoff `RN(1/m)` but is never
one of its binary neighbours. Public Actual/Actual ICMA horizons are
compensated sums of per-period quotients, so they can be either. The private
kernel tier covers every cutoff and both neighbours directly. The public
failure-order suite checks the reachable public cases listed above.

The failure-order suite was also run against 11 single-line production
mutants, each in a fresh copy of the crate under
`/private/tmp/sabiquant-yr-failure-order-mutants-v1`. The unmutated copy
passed, and every mutant compiled and failed at least one test. The mutants:

- accept `y=-m`;
- move either hybrid cutoff comparison, or compare `RN(t*m)` with 1;
- skip the quote floor in `risk`, or in callbacks for Simple-active
  `CompoundedThenSimple` plans;
- report traversal positions instead of original flow indices, in `risk` or
  in pricing;
- accept an overflowing PV;
- disable `NumericalBoundary`;
- round the Simple product before adding one.

## Aggregate native checks

Every Rust command used Rust 1.99.0 from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`,
with `RUSTDOCFLAGS='-D warnings'`. Each command had its own target directory
under `/private/tmp/sabiquant-yr-audit-verified-v3/`. The runner hashed all
127 `src`/`tests` Rust files before the first command and asserted them
unchanged after the last. The first command started after the final source
edit, and the same 127 hashes still match. Earlier runs of the same commands
also passed: Codex's `...-verified-v1` before the failure-order suite existed
(757 debug, 756 release), and `...-verified-v2` with its first six tests (763,
762).

| Check | Observed result |
|---|---|
| `cargo test --offline --locked` | 764 passed in 90 result groups: 101 unit, 622 integration, 36 executable and 5 compile-fail doctests; no failure, ignore or warning |
| `cargo test --offline --locked --release` | 763 passed: 100 unit, 622 integration, 41 doctests |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit zero |
| `cargo fmt --all --check`; `git diff --check` | Exit zero |
| Warning-denied public and private `cargo doc --no-deps` | Exit zero |
| `cargo check --all-targets` for x86_64 Linux and Windows, `RUSTFLAGS='-D warnings'` | Exit zero; compile-only |

The one debug/release difference is the inherited debug-assertion-only test
`error::tests::invalid_interval_is_a_defect`. Cross-compilation is not native
Linux or Windows execution.

## Metadata refresh after the audit commits

Local commits `61eb382` and `c70bf5f` changed only `Cargo.toml` and
`Cargo.lock` after the v3 checks. All 127 Rust source/test hashes in the v3
manifest still match. No numerical expectation, production arithmetic or
fixture changed. The current package version is `0.1.0-alpha.1`; Cargo metadata
reports `publish: null` because `publish = false` is now commented out. This
metadata is recorded as observed; local-only authorization remains in force.

A fresh `/private/tmp/sabiquant-alpha1-metadata-check-v1` run passed the policy
suite, warning-denied native/Linux/Windows all-target checks and all-target
strict Clippy. The standalone current-version package passed its 101 unit
and 41 doc tests. Repeating the expensive unchanged numerical fixtures was
unnecessary for this metadata-only edit; the aggregate 764/763 results above
remain tied to v3's exact Rust-source manifest.

The private checker was rerun against all 47 current source/package files in
`/private/tmp/sabiquant-alpha1-kernel-v1`: all 20,201 rows passed, with strict
Clippy on that same temporary copy. Only its copied module received the test
adapter; original files and all fixture/adapter/runner bytes remained unchanged.

## Shared fixture inventory

A program that includes the repository's own `tests/support/fixture.rs`
reader loaded every fixture under `validation/fixtures`. It ran after the
inverse header's final change. All **69 files and 2,951,635 rows** passed every
schema-1 check, including 992 inverse, 2,181 public risk, 20,201 kernel and 141
frozen yield/risk rows.

## Production-region instrumentation

`validation/coverage.py` built and ran 88 instrumented binaries into
`/private/tmp/sabiquant-yr-audit-coverage-v3`; 723 tests passed. Every binary
was compiled after the final source edit. Its production-region file is
byte-identical to the earlier runs `...-coverage-v1` (Codex, 87 binaries, 716
tests, before the failure-order suite) and `...-coverage-v2` (722 tests).
Doctests are not instrumented, and no branch or input coverage is claimed.

| Production scope | Covered / regions | Uncovered |
|---|---:|---:|
| Entire crate | 13,540 / 13,994 (96.7558%) | 454 |
| `bonds/yield_risk.rs` | 1,154 / 1,243 (92.8399%) | 89 |
| `bonds.rs` | 815 / 832 | 17 |
| `rates.rs` | 857 / 865 | 8 |
| `error.rs` | 412 / 415 | 3 |
| `math/solver.rs` | 743 / 743 | 0 |
| `bonds/quote.rs` | 139 / 139 | 0 |

The phase C instrumentation (`/private/tmp/sabiquant-m8-settlement-v1-coverage`)
measured 12,198 / 12,563 regions. Every production file other than
`yield_risk.rs` still has exactly the same multiset of uncovered source text
there; only line numbers moved. That includes `bonds.rs`, `rates.rs`,
`error.rs` and `math/solver.rs`, which phase D changed. Their gaps keep the
classifications in the earlier validation reports. All 89 new uncovered regions
are in `yield_risk.rs`. Line numbers below refer to the pinned file.

**Workspace reservation (8 regions).** `yield_from_price` maps a refused
present-value or sensitivity-key reservation into `BondYieldError::Preparation`
(lines 559–560 and 562–567, 7 regions). `risk` propagates its present-value
reservation (line 652). Each request has at most `Bond::MAX_FLOWS = 3,652,059`
slots of at most 32 bytes, so at most 116,865,888 bytes. That is below
`isize::MAX` on 32- and 64-bit targets, so `CapacityOverflow` is impossible and
only allocator refusal can fail a reservation. No OOM was injected.

**Unscaled residual overflow in `amount_residual` (5 regions, lines 176–184).**
A successful report is one of two kinds:

- a `FunctionZero` point, either an endpoint or an interior trial, with
  `Dhat = T` exactly;
- the endpoint of a sign-changing bracket with the smaller absolute residual
  (the lower endpoint on a tie).

A zero residual comes only from an exactly zero difference. A nonzero
difference that rounded to zero would be a callback error (line 1000, excluded
below), not a success. For the bracket case:

- With scale 0, `abs(T) < 2^970` keeps `abs(Dhat - T)` below the overflow
  midpoint.
- With scale 1, an endpoint whose difference reaches the midpoint has a halved
  residual that rounds to at least `2^1023`. The opposite endpoint's residual
  rounds to at most `2^1023 - 2^970`.

Brent therefore cannot select the overflowing endpoint. The implementation
review gives the full argument; no report is forged to execute the guard.

**Private capacity and conversion guards (76 regions).** Each propagates
`InternalCapacity` or `None` from fixed-width exact arithmetic:

- **Exact residual and constant arithmetic on `2^-1074` (34 limbs), 25
  regions.**
  - Lines 165, 174, 602, 987, 998 and 1000 cover the residual path. Lines
    1016–1059 and 1072–1096 cover `min_sub_difference` and `decode_min_sub`.
    Lines 821–844, 896–927 and 972 cover the constant-plan path.
  - Every decoded value is finite: an evaluated or cached dirty amount, the
    dirty target or a prepared flow amount.
  - A shifted significand is below `2^2098` and a difference below `2^2099`.
    Fewer than `2^usize::BITS` constant terms sum below `2^2162`, inside 2,176
    bits.
  - The zero/infinite scaled-residual guard at line 1000 is excluded. With
    scale 0, a nonzero multiple of `2^-1074` rounds to at least `MIN_SUB`. With
    scale 1 (`abs(T) >= 2^970`), a nonzero `abs(Dhat - T)` is at least
    `2^917` and at most `2*MAX`, so its half rounds to a nonzero finite value.
- **Normalized extended terms (5 limbs), 12 regions.** These are lines
  1140–1231 in `round_extended_term` and the `decompose_nonzero` guard at line
  1113. Every factor is finite and nonzero:
  - an accepted nonzero present value and a positive horizon;
  - a positive Simple factor and `m >= 1`;
  - `delta = RN(m+y)` and `w = RN(t+RN(1/m))`, both positive.

  The numerator is below `2^265` and the denominator below `2^106`. Alignment
  stays below 320 bits, the divisor is nonzero and the quotient has 53 bits.
- **Signed risk sums (243 limbs), 25 regions.**
  - Lines 678–786 and 798–802 are `?` propagations in `risk`: term rounding,
    accumulation and the three `subtract_risk_sums` calls. Lines 1256–1274
    are inside `accumulate_risk_term`; lines 1294 and 1305 are inside
    `subtract_risk_sums`.
  - Term exponents lie in `[-7470, 7216]`, so the `2^-8192` shift is
    nonnegative. Each coefficient is below `2^15461`.
  - At most one term per flow enters each accumulator, which keeps every sum
    below `2^(15461+usize::BITS)`. Subtraction runs only after a magnitude
    comparison.
- **Final rounding, 14 regions.** These are lines 1324 and 1344–1353 in the
  private helpers `round_raw_risk` and `round_normalized_risk`, and lines
  1406–1446 in the generic `round_wide_ratio_to_f64`. Bit counts fit `i32`, and every alignment stays
  inside the instantiated width. Divisors are nonzero, quotients have at most
  53 bits and a normal biased exponent is at least 1. A normalized getter
  rejects zero dirty price before decoding it.

None of these guards was exercised by forging private state.

## Package integrity

Current verification is `/private/tmp/sabiquant-alpha1-metadata-package-v1`.
`cargo package --offline --locked --allow-dirty` produced
`sabiquant-0.1.0-alpha.1.crate`; extraction and every subsequent check completed.

- **Contents.** 49 regular files: 47 byte-checked originals plus two
  Cargo-generated files. After normalizing the versioned archive directory,
  the member list differs from phase C only by `src/bonds/yield_risk.rs`.
  All 47 originals match the current tree before and after verification.
- **Extracted crate.** 101 unit tests and 36 executable plus 5 compile-fail
  doctests pass offline. Strict Clippy and warning-denied public/private docs
  pass. No integration fixture is present in this standalone archive.
- **Metadata.** One package, no dependencies/features/build script; version
  `0.1.0-alpha.1`, `publish: null` as observed above.
- **Size.** 227,390 compressed bytes; 1,024,094 bytes of content.
- **Exclusions.** No fixture, generator, harness or integration suite ships.

Earlier v1/v2 `0.0.0` packages passed their checks but predate the Cargo
metadata edit. Their archived pins below are historical; the current alpha
archive and source attestation are pinned separately.

## Reproducibility pins

| Artifact | SHA-256 |
|---|---|
| Yield/risk contract | `62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d` |
| `src/bonds/yield_risk.rs` | `07204d70658a2bf2a6d57be5023c15b16f70b9d55a92adc7b01f5a7c27dc78ba` |
| Its production prefix before `#[cfg(test)]` | `58ee1eaa018253cad493fc04b0fc3207d41b7808707e95edecb66a55cf58b878` |
| `src/bonds.rs` | `4b3527861d734aa64ceb33a636b0d075f912de5b05b129c01c0dca366b21a190` |
| `src/rates.rs` | `cce6e34466202e3628648f44341f2cc7a89157359678a5799e8634e07d86af5d` |
| `src/math/solver.rs` | `182907c97c0ea617560289187b2a93ac08414ec6e7bdf4fc328abe2b6007a3c2` |
| `src/error.rs` | `77d5b27ec995f5175e6079ddefbfbd828bf7391bdbe7a80d4403ba514d4c2f04` |
| Frozen generator | `16949a31bf9845bfc72b2dbf2a0c2b2c159efeaaa367c4ec79b2f0a46ad9b087` |
| Frozen fixture | `f8fc3ab55daff1679698bb9c7d05f40cbf94c09a4b0a3be9c2c155c517fca3dd` |
| Extended risk generator | `9c5e77b68dbd9c2212b4ae0fb2cd7cdbc9b420505a55d29c01afa0b559541b85` |
| Kernel fixture | `7c36d7bb9b04ad62295205f209ae7f7c6527b158a60ef7f7f1326945a25fba13` |
| Public risk fixture | `095d4d138be98a71b5c3b89307afcc529a63360e8d31e948d051c335886655d3` |
| Inverse generator | `70723437953a120d18e92bc608a9b2a61112270a3b600b25f01b44a36a4ff258` |
| Inverse fixture | `00c710b1fa82805f23cb8ab9d422d7d5f83df5288a9dbb7039e4db9598dd547b` |
| Kernel runner | `be607eb014d8f98e1442b6fdeb88743ef6f06d8ade89a389998149ba0eb55f77` |
| Kernel Rust adapter | `efdc66adf5ddebb9cad27c7e9f24e25421ef2aabbc8c4217ef7f3f7546df9adc` |
| `tests/bonds_yield_risk_oracles.rs` | `d164555ccf01227feea561dd33e44dc2d442161a254fd4ef7b66c328af83414c` |
| `tests/bonds_yield_risk_hand.rs` | `ac9b7eefc189abc1552459313bfba39442f9f939c1c06135f39e8db64949da01` |
| `tests/bonds_yield_risk_review.rs` | `198fd48e59a2b677524d4b237865e86e4d9a40dd70b7169b677bf80d69521c08` |
| `tests/bonds_yield_risk_normalized_range.rs` | `dab1bd077f69f9dfeb3a7eb04436fb0242958b8347fb1f11206ded7204933a14` |
| `tests/bonds_yield_risk_extended_oracles.rs` | `3cdf425394645604904388ade7330cf76efff59a751d959d3b2136dfcd47281c` |
| `tests/bonds_yield_inverse_extended_oracles.rs` | `68eefc4acffd098a83b5406c981e07ce6bac54731c6887c67c5947f90b55ac8e` |
| `tests/bonds_yield_risk_failure_order.rs` | `6a7de99a3e5d925aea3f6ba040877a86004940b49d3c8c59fd2b9f810c935e34` |
| `/private/tmp/sabiquant-yr-failure-order-mutants-v1/evidence.json` | `415785b37a5b0df8dd5dbff168c3e57dd920810b52aeed87bb41ed585a0e5c44` |
| `/private/tmp/sabiquant-yr-audit-verified-v3/evidence.json` | `89255db89b61417914f8571bd8a0a071d9eae953fbb328f34332547411898345` |
| Native debug log, `debug.log` there | `5bacf23785531350cef9d46c91b496794151404add7d836792bb46981400662d` |
| Native release log, `release.log` there | `961fb7397136d351c3591b9d904ab88fd9786779ba8bb12a15f1446b20fbeafa` |
| Earlier `/private/tmp/sabiquant-yr-audit-verified-v1/evidence.json` | `3d300a1a016cfd83c4b1d7208dd0aaac3457787246208dd6d00d7859d8b9feda` |
| `production-regions.json` in `...-coverage-v3` (and `-v1`, `-v2`) | `f334fa4f851a968ec45d724ef320aa00c19be4ec0cbf31505634fa314ccb9b88` |
| `/private/tmp/sabiquant-yield-risk-kernel-final-v3/evidence.json` | `a2e053fe2f8dd3f0ac2bf101af0e8c3a9168f1c5bf6dc81e3c62118c18059a95` |
| Kernel copy Clippy, `clippy-evidence.json` there | `2c0b6981392a2c93ea6112e7c3fa61a0c36d9070f9c023cc88e8eea587ba0329` |
| Earlier `/private/tmp/sabiquant-yield-risk-kernel-final-v2/evidence.json` | `0e43e82734b6c098888c38f08f5e359e97f1431f405055a093eb6dd726bbd67a` |
| `/private/tmp/sabiquant-yr-audit-preservation.json` | `38a0882730d349e64b7612ff763d1987d3cd529005b36003d8a035433bbf223a` |
| `.crate` archive under `/private/tmp/sabiquant-yr-audit-package-v2/target/package/` | `a6e8d895f4fd67d175d68fd752fc0f6154891de01ebada5c02af3b5e846ce387` |
| `/private/tmp/sabiquant-yr-audit-package-v2/evidence.json` | `d050bd3548ef370e07bed89568528f8c2cc6453f43b0528bb0480a887b6e7fce` |
| Current alpha `.crate` archive | `26d49022365d928068210f43aa3b1cd4590c7dae110de02ee94b71692cee3e62` |
| `/private/tmp/sabiquant-alpha1-metadata-package-v1/evidence.json` | `8ae71316e361210abd4aebd95a4ca329cc07599f0cfc206f2cd69358b8bf1c38` |
| `/private/tmp/sabiquant-alpha1-metadata-check-v1/evidence.json` | `6f97244b5548bdc30b9bdb9c15bfdc1fca0fa3cef1899c3b345dcd4dd029915f` |
| `/private/tmp/sabiquant-alpha1-kernel-v1/evidence.json` | `60eb003c45b8d80bb9a18d2edf3db27adfbcbbb6bb6b725b74f8a25ef8849b66` |
| Current kernel copy `clippy-evidence.json` | `0a888ff3739aaff0afd53f7acb9ac6ffc31a9c4ff71169c48d6c4a16a1b60e27` |

## Historical result boundary

The `a33e8a7` report recorded 747 debug/746 release tests and compile-only
Linux/Windows checks. It also recorded 96.7917% production-region coverage and
an extracted 49-member package. Those are historical claims, not verification
of this audit tree. Its blanket classification of all 77 missed yield/risk
regions was unsupported and used incorrect widths. It is withdrawn and replaced
by the individual assessment above.

LLVM source-region coverage does not measure exhaustive inputs or branch
coverage. Doctests and the supplemental copied-crate kernel run are separate.
No OOM injection, private financial-state forgery, platform-libm proof,
QuantLib C++ reading, remote CI run or publication is included.

## Remaining work

- The financial comparisons in the extended tiers are conditional on the
  measured M4 elementary profile. They are not analytical enclosures or a
  formal libm bound.
- Named market yield models, analytical enclosure producers, and owner,
  financial, native-platform and release acceptance remain open.
