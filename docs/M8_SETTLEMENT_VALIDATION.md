# Local exact settlement boundary and contextual bond validation

Completed on 2026-10-17, Asia/Bangkok, on native macOS arm64 against the
local worktree based on `ea22049`. This is engineering evidence for the
adopted [settlement boundary contract](M8_SETTLEMENT_CONTRACT.md)
(`FR-SETTLE-01..16`, SHA-256
`47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba`) and the
additive [contextual bond corpus](M8_BOND_CONTEXTUAL_ORACLES.md)
(`FR-BOND-04/06/07/08/09/10`). It is not owner API sign-off, named DMO/Treasury
market compliance, an analytical ideal-price enclosure proof, or multi-platform
release acceptance. Work remained local; no push or remote CI run is part of
this report.

## Implemented scope and authorship

`sabiquant::settlement` adds eight public types (`SettlementEvidence`,
`SettlementRounding`, `CashAggregation`, `SettlementInstruction`,
`SettlementDecision`, `SettlementReceipt`, `SettlementGuarantee`, and
`SettlementError`) and six domain error variants (`NonPositiveNominal`,
`DecimalIntervalOrder`, `BinaryIntervalOrder`, `NonFinite`, `Arithmetic`, and
`InternalCapacity`), wrapped by `Error::Settlement`. The public type census is
**97** (up from 89) and the exhaustive domain error catalog is **131** (up from
125).

Four bounded recipes operate on strictly positive exact `DecimalAmount` par
nominal:

- `dirty_per_100` (`DirtyPer100`): `Rc(N * P / 100)`.
- `clean_accrued_per_100` (`CleanAccruedPer100`): `capital = N * C / 100` and
  `accrued = N * A / 100`, combined under `RoundTotal` or
  `RoundComponentsThenSum`.
- `staged_price100_accrued1000` (`StagedPrice100Accrued1000`): fixed source
  quanta `1e-6` (price) and `1e-5` (accrued per 1,000) with explicit caller
  rounding modes before nominal scaling and cash aggregation.
- `coupon_per_1000` (`CouponPer1000`): `Rc(N * K / 1000)` with no source
  rounding stage.

`src/settlement/arithmetic.rs` instantiates the shared crate-private
`amounts::uint::Wide<40>` (2,560-bit) unsigned kernel on common denominator
`D0 = 2^1074 * 5^38`, retaining continuous versus discrete source-count image
structure for `Exact` domain checks and admitting only final cash endpoints to
`DecimalAmount`. Separate authors produced the
[independent settlement oracle](M8_SETTLEMENT_ORACLES.md), the
[blind hand suite](M8_SETTLEMENT_HAND_TESTS.md), the
[post-hand implementation review](M8_SETTLEMENT_IMPLEMENTATION_REVIEW.md), and
the [contextual bond oracle](M8_BOND_CONTEXTUAL_ORACLES.md).

## Retained suites and fixture inventory

| Suite | Evidence |
|---|---|
| `settlement_oracles.rs`, 2 tests | Consumes all **154,241** rows (154,063 distinct serialized inputs) of `validation/fixtures/settlement/oracles.fixture`: exact canonical amounts, Display strings, evidence bounds and canonical zero bits, provenance, identifier registries, retained instruction policies, receipt identity, structured error fields, root/domain/`AmountError` source chains, row-derived censuses, and source/contract/requirements/body/record/full-fixture SHA-256 and FNV-1a64 pins |
| `settlement_hand.rs`, 27 tests | Independently derived literal/rational controls, all 39 scales x 7 modes x 2 signs, complete enumeration of **432** small staged image recipes, source/cash range rescue, `i128::MAX` wide cancellation, staged 10+1 weights, continuous and discrete `Exact` interior detection, precision holes between representable extrema, error precedence, and 8-thread concurrent reuse |
| `bonds_contextual_oracles.rs`, 2 tests | Consumes all **11,136** distinct rows (52 columns, all 32 compounding/frequency configurations x 348 rows) of `validation/fixtures/bonds/contextual-oracles.fixture`: ICMA regular/stub periods, terminal 30E/360 ISDA, both US 30/360 EOM flags, custom and strict TARGET Business/252, reopening, payment shifts, stopped query ledgers, exact supplied-YF product maps (114,400 entries), staged enclosures, and raw-rational ideal anchors |
| Existing M1–M8 suites | All prior unit, integration, oracle, reference, property, policy, and public-API suites pass unchanged |

The shared Rust fixture reader parses **65 committed fixture files** containing
**2,928,120 rows** without error. All **97** prior generator and fixture
artifacts remain byte-identical to `ea22049` (verified in
`/private/tmp/sabiquant-m8-settlement-immutability-types.json`).

## Aggregate native checks

Every Rust command used Rust 1.99.0 from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`
with isolated target directories under
`/private/tmp/sabiquant-m8-settlement-v1-verified`.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked`, warning-denied rustdoc | **730 passed** in 83 groups: 96 unit, 593 integration, 41 doctests (36 executable + 5 compile-fail); 0 failed |
| `cargo test --offline --locked --release`, warning-denied rustdoc | **729 passed** in 83 groups: 95 unit, 593 integration, 41 doctests; 0 failed |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0; 0 warnings |
| `cargo fmt --all --check` and `git diff --check` | Exit 0 |
| Warning-denied public and private `cargo doc --offline --locked --no-deps` | Exit 0 |
| Warning-denied `cargo check --offline --locked --all-targets` for `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-msvc` | Exit 0 (compile-only target checks) |
| `python3 -I validation/generator/settlement_oracles.py --check` | `identical` (154,241 rows, 498,566 Decimal quotients at 1536/2048 digits) |
| `python3 -I validation/generator/bond_contextual_oracles.py --check` | `identical` (11,136 rows, 160/240-digit agreement) |

The single test-count difference between debug (730) and release (729) is the
inherited `#[cfg(debug_assertions)]` interval invariant unit test. Focused
debug and release runs on the final `tests/settlement_oracles.rs` consumer
(`f1e1b723...`, which added the full-fixture header FNV-1a64 pin after the
aggregate battery started) also passed both tests in debug (331.65 s) and
release (8.15 s) with zero production or row-expectation changes.

## Production-region instrumentation

`validation/coverage.py` compiled and ran **81** instrumented test binaries
(**689** unit and integration tests passed) into
`/private/tmp/sabiquant-m8-settlement-v1-coverage`. Doctests run separately
and are not instrumented; region counts measure LLVM source regions, not
branch or full input-space coverage.

| Production scope | Covered / total regions | Share | Uncovered |
|---|---:|---:|---:|
| Entire crate | 12,198 / 12,563 | 97.0946% | 365 |
| `src/settlement.rs` | 283 / 290 | 97.5862% | 7 |
| `src/settlement/arithmetic.rs` | 632 / 694 | 91.0663% | 62 |
| `src/amounts/uint.rs` | 302 / 304 | 99.3421% | 2 |
| `src/amounts.rs` | 724 / 753 | 96.1487% | 29 |
| `src/amounts/binary.rs` | 267 / 285 | 93.6842% | 18 |
| `src/bonds.rs` | 770 / 787 | 97.8399% | 17 |
| `src/bonds/quote.rs` | 139 / 139 | 100.0000% | 0 |
| `src/error.rs` | 400 / 403 | 99.2556% | 3 |

Every uncovered region in the new settlement files is a defensive
`InternalCapacity` path or `?` propagation excluded by the contract's
40-limb (2,560-bit) capacity and domain invariants:

- **`src/settlement.rs` (7 regions):**
  - Lines 443 and 445 (4 regions): `AmountQuantum::new(1, 6)` and
    `AmountQuantum::new(1, 5)` constant source-quantum constructors in
    `staged_price100_accrued1000`, which are always valid (`1 > 0` and
    `scale <= 38`).
  - Lines 743–745 (3 regions): private helper `capacity(stage, operation)`,
    which constructs `SettlementError::InternalCapacity` when a 40-limb
    intermediate invariant fails.
- **`src/settlement/arithmetic.rs` (62 regions):**
  - Lines 36, 41, 46, 54 (4 regions): `Signed::add` and `Signed::multiply`
    40-limb `?` propagations.
  - Lines 79, 80, 83, 84, 98 (3 regions), 104, 105 (9 regions): `Image::weight`
    and `Image::combine` step-slot (`<= 2` independent source steps) and
    endpoint addition propagations (`11 * C < 2^2190`).
  - Line 138 (4 regions): `checked(...)` error closure calling `capacity(...)`.
  - Lines 170, 171, 193, 204, 252, 256, 267, 268, 287, 288, 291 (2 regions)
    (12 regions): `evaluate_inner`, `two_components`, and `decode` `?`
    propagations on `D0 = 10^38 << 1036`, source decoding, weighting, and
    component sum (`< 2^1281`).
  - Lines 311, 312, 314 (2 regions), 320, 333 (3 regions), 342 (9 regions):
    `decode_decimal` (`10^scale | D0`, product `< 2^2188`) and `decode_binary`
    (`1 <= field <= 2046`, shifted product `< 2^2188`).
  - Lines 366, 367, 369 (2 regions), 371, 372, 382, 404, 411 (9 regions):
    `source_round` reconstruction on `D0` and `cash_round` denominator
    (`D0 * 10^Nscale * divisor < 2^1427`).
  - Lines 438, 439, 440, 447, 448, 476, 501, 502, 514, 526, 530 (11 regions):
    `quantize` and `round_endpoint` 40-limb multiplications/divisions
    (numerator `< 2^2446`, denominator `< 2^1427`, `count * Q < 2^2447`) and
    upper-endpoint rounding after a discrete `Exact` lattice check has already
    proven every reachable point is divisible.
  - Lines 548, 574 (3 regions) (4 regions): `admit` division by 10 and final
    `DecimalAmount::new(coefficient, scale)` after trailing-zero normalization
    and `i128` range admission.

No invalid private state was forged and no OOM was injected to artificially
execute proof-excluded capacity branches.

## Package integrity

`cargo package --offline --locked --allow-dirty` in
`/private/tmp/sabiquant-m8-settlement-v3-package` produced
`sabiquant-0.0.0.crate` (**212,780 bytes** compressed, **943,651 bytes**
unpacked content, **48** archive members: 46 byte-verified repository files
plus `.cargo_vcs_info.json` and `Cargo.toml.orig`). The allowlist differs from
the bond-core archive solely by `src/settlement.rs` and
`src/settlement/arithmetic.rs`; all validation fixtures, generators, and
integration tests remain excluded.

Extracting the `.crate` archive into a standalone directory and running against
it offline passed **96** unit tests, **36** executable doctests, **5**
compile-fail doctests, strict `cargo clippy --offline --locked --all-targets -- -D warnings`,
warning-denied public and private `cargo doc`, and `cargo metadata` (zero
dependencies, zero features, `publish = []`).

## Reproducibility pins

| Artifact | SHA-256 |
|---|---|
| `docs/M8_SETTLEMENT_CONTRACT.md` | `47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba` |
| `docs/M8_SETTLEMENT_ORACLES.md` | `db18b7304b2cbd8c93e8bd1f4f678ca8c46dde4d026fc186123e77a474e8c4bc` |
| `docs/M8_SETTLEMENT_HAND_TESTS.md` | `815086d532653a8623719e300bf904ec0d3374629e81a98391b4767acb9ac31b` |
| `docs/M8_SETTLEMENT_IMPLEMENTATION_REVIEW.md` | `0aeef196bf3198880147c234d4b2975dbaa8ae4fa0445f8cac1c52f7acb82ba3` |
| `docs/M8_BOND_CONTEXTUAL_ORACLES.md` | `bf914edcb00cb22ceace485bf2b51e14bdf8a82cca7f6db7b61b10bf6a2dc638` |
| `src/settlement.rs` | `eaf0306e738d90c44154de46e2fd834050400ea1cb63a17548436a78d7358363` |
| `src/settlement/arithmetic.rs` | `eb8d66b195e0d6a142f63c388bd0bcdd7f80b11e23b2390dd037ae87d8a82dd1` |
| `src/amounts/uint.rs` | `1e3cb59207ea9e13e135d00a192971370279bf43cacc352730be43bfe24c54ec` |
| `src/amounts.rs` | `c3031d3fb24ad1ca579c3ba0115d100f7bead99172cddea9879ebb71e0f8a9bf` |
| `src/error.rs` | `f40430d465ffad358d0312a7d890e7f2f37b3f71914a820502fa279fa9d1a126` |
| `src/lib.rs` | `b5e97322a5115c065e6d5fa9eab101241954b3c48eb90bf5b678d83a6aaf2935` |
| `validation/generator/settlement_oracles.py` | `dda662320a74b485438b23be7a06124567d0dd6794fc85584006aae3c5043a69` |
| `validation/fixtures/settlement/oracles.fixture` (38,656,756 bytes) | `fa97ed1298693f364eccc903d4763829693e7320606bc6e95a0733cbe1477f4e` |
| `validation/generator/bond_contextual_oracles.py` | `3f2398b98f735c9cafde9321040cfb28009fc67d9b1857223be032e9095ba588` |
| `validation/fixtures/bonds/contextual-oracles.fixture` (15,639,401 bytes) | `6d1e46384edbc2e0f50e5e86cf6f1a46280451befc396c95c15e8fb9a22f3c21` |
| `tests/settlement_oracles.rs` | `f1e1b7231476f786c4ea2f1bc3416953927c7f115112d4449517b7e992da2bb9` |
| `tests/settlement_hand.rs` | `163446e073a9b19ea3e6147876efd6b81d2b51bb81d8fe1c26b5067df2ea1070` |
| `tests/bonds_contextual_oracles.rs` | `f37b149c0aed3df2e287a0391f8bfec20b06e16f9a4eb03d3943c469f91220b7` |
| `tests/public_api.rs` | `d6b78a9994aef6c9d5b8ad1495068712833015e7f662c03a5c38c3e6feaf8859` |
| `/private/tmp/sabiquant-m8-settlement-v1-verified/debug.log` | `98bfaf71a8b3021cf6b5e8795e9fc2b48f4976a54b0b0c96129401a9d9b1bde7` |
| `/private/tmp/sabiquant-m8-settlement-v1-verified/release.log` | `ea344647f1285ff1bae8a89c61a1cf61adb9d6344bf34e74439721fbc3f9d613` |
| `/private/tmp/sabiquant-m8-settlement-v1-verified/evidence.json` | `fc87c6e89384db4efdc4654e33c8cc78b1f2ac6994454ee200154a0203ad6e44` |
| `/private/tmp/sabiquant-m8-settlement-v1-coverage/production-regions.json` | `dc4b75b8614eb70c45caf36ccb1e6e83b721a168c07625d85de1cc2d9bd5c7be` |
| `/private/tmp/sabiquant-m8-settlement-v3-package/target/package/sabiquant-0.0.0.crate` | `531037ea37f22242b5622895f942b2c0a560975e2ba344a918cda44a71b48083` |
| `/private/tmp/sabiquant-m8-settlement-v3-package/evidence.json` | `b89640c4c76738255d86c65772079791da9e77eb2eb7ce5b8c0f243fdae00588` |

## Remaining gates

Completing `sabiquant::settlement` and the contextual bond corpus does not close
M8. Generic bond yield-to-maturity inversion, Macaulay/modified duration,
convexity, basis-point value (`docs/M8_BOND_YIELD_RISK_CONTRACT_DRAFT.md`),
source-complete named market yield/settlement profiles (DMO gilt and US
Treasury), analytical enclosure producers, and independent owner, financial,
native primary-platform CI, and release acceptance remain open in
[M8_LOCAL_READINESS](M8_LOCAL_READINESS.md).
