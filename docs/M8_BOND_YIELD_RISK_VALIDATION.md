# Local generic bond yield-to-maturity inversion and analytical risk validation

Completed on 2026-10-17, Asia/Bangkok, on native macOS arm64 against the
local worktree based on `65ff67e`. This is engineering evidence for the
adopted [generic bond yield and risk contract](M8_BOND_YIELD_RISK_CONTRACT.md)
(`FR-YR-01..16`, SHA-256
`62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d`, FNV-1a64
`2a983dabd6706130`). It is not owner API sign-off, named DMO/Treasury/Canadian
market compliance, an analytical ideal-price enclosure proof, or multi-platform
release acceptance. Work remained local; no push or remote CI run is part of
this report.

## Implemented scope and authorship

`sabiquant::bonds` (`src/bonds/yield_risk.rs`) adds four public types
(`BondYieldReport`, `BondYieldSensitivity`, `BondRisk`, and `BondYieldError`),
six `BondYieldError` variants (`TargetConversion`, `DirtyTargetOverflow`,
`NoFutureCashFlows`, `NoYieldSensitivity`, `CallbackEvaluation`, and `Solver`),
two new `BondError` variants (`YieldDomain` and `RiskDirtyPriceZero`), and root
`Error::BondYield`. The public type census is **101** (up from 97) and the
exhaustive domain error catalog is **139** (up from 131).

Two complete operations are added to `BondPricingPlan`:

- `BondPricingPlan::yield_from_price(&self, target: BondPrice, bracket: RootBracket, options: SolverOptions) -> Result<BondYieldReport, BondYieldError>`:
  resolves the target per-100 quote via `amount_from_price_per_100` and (for
  `Clean` targets) adds `accrued_amount`, classifies positive-horizon net flows
  using an exact `Wide<33>` (2,112-bit) fixed-point accumulator on `2^-1074`,
  computes power-of-two endpoint residual scaling `s = max(0, max_exp - 960)`,
  and solves for the annual decimal yield via crate-private
  `math::solver::brent_with_payload`.
- `BondPricingPlan::risk(&self, yield_rate: f64) -> Result<BondRisk, BondError>`:
  checks the yield domain (`1 + y > 0` for `Simple` with `t > 0`; `1 + y/k > 0`
  for `Periodic` and `SimpleThenCompound` with `t > 0`), evaluates each entitled
  flow's present value and active branch via `rates::observe_present_value_at`,
  computes single-rounded 4-factor and 3-factor extended products in `Wide<4>`
  (256-bit) significand arithmetic, and accumulates exact signed sums in
  `Wide<35>` (2,240-bit) fixed-point registers on `2^-1074`. `BondRisk` provides
  `price_derivative`, `price_second_derivative`, `time_weighted_amount`,
  `modified_duration`, `macaulay_duration`, and `convexity` via single-rounded
  `Wide<37>` (2,368-bit) ratio division.

## Retained suites and fixture inventory

| Suite | Evidence |
|---|---|
| `bonds_yield_risk_oracles.rs`, 1 test (141 rows) | Consumes all **141** rows across 6 families (`SINGLE-FLOW-32`, `COUPON-32`, `CONSTANT-PLAN`, `SIGNED-NONMONOTONE`, `EXTREME-SCALE`, and `ERROR-TAXONOMY`) of `validation/fixtures/bonds/yield-risk-oracles.fixture`: exact target conversions, solve status/brackets/iterations/evaluations, power-of-two residual scaling exponents, `BondYieldSensitivity`, `amount_residual`, `clean_amount`, per-100 quotes, all six `BondRisk` getters across all 32 compounding/day-count configurations, structured error payloads, and contract/requirements/generator/body SHA-256 and FNV-1a64 pins |
| `bonds_yield_risk_hand.rs`, 11 tests | Closed-form zero-coupon yield and risk across `Simple`, `Periodic`, and `Continuous` compounding; par fixed-rate bond yield and Macaulay/modified duration identity `D_mac = (1 + y/k) * D_mod`; `SimpleThenCompound` split-horizon (`t <= 1/k` and `t > 1/k`) yield and risk; opposite-sign same-date net flow cancellation and non-monotone double-root brackets (`Uncertified`); extreme `f64::MAX`-scale positive `residual_scale_exponent > 0` inversion; exhaustive failure precedence (`TargetConversion`, `DirtyTargetOverflow`, `NoFutureCashFlows`, `NoYieldSensitivity`, `CallbackEvaluation`, `Solver`, `YieldDomain`, `RiskDirtyPriceZero`); root `Error::BondYield` transparency; and 4-thread concurrent `BondPricingPlan` and `BondRisk` sharing |
| `bonds::yield_risk::tests`, 5 unit tests | Direct white-box verification of `compute_scale_exponent`, `scale_down_exact`, `min_sub`, `evaluate_scaled_residual`, `round_extended_term` (including exact subnormal ties and binade-carry midpoint `2^54 - 1`), `divide_wide_by_f64_to_f64`, `divide_wide_by_f64_product_to_f64`, `SensitivityKey`/`SensitivitySum` multiplicity pairing, negative zero-horizon net constants, and `Wide<35>` risk sum overflow |
| Existing M1–M8 suites | All prior unit, integration, oracle, reference, property, policy, and public-API suites pass unchanged |

The shared Rust fixture reader parses **66 committed fixture files** containing
**2,928,261 rows** without error. All earlier generator and fixture artifacts
remain byte-identical to `65ff67e`.

## Aggregate native checks

Every Rust command used Rust 1.99.0 from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked`, warning-denied rustdoc | **747 passed**: 101 unit, 605 integration, 41 doctests (36 executable + 5 compile-fail); 0 failed |
| `cargo test --offline --locked --release`, warning-denied rustdoc | **746 passed**: 100 unit, 605 integration, 41 doctests; 0 failed |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0; 0 warnings |
| `cargo fmt --all --check` and `git diff --check` | Exit 0 |
| Warning-denied public and private `cargo doc --offline --locked --no-deps` | Exit 0 |
| Warning-denied `cargo check --offline --locked --all-targets` for `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-msvc` | Exit 0 (compile-only target checks) |
| `python3 -I validation/generator/bond_yield_risk_oracles.py --check` | `identical` (141 rows, dual-precision 160/240-digit Decimal + exact Fraction agreement) |

## Production-region instrumentation

`validation/coverage.py` compiled and ran **83** instrumented test binaries
(**706** unit and integration tests passed). Doctests run separately and are
not instrumented; region counts measure LLVM source regions.

| Production scope | Covered / total regions | Share | Uncovered |
|---|---:|---:|---:|
| Entire crate (with unit tests) | 19,096 / 19,676 | 97.0522% | 580 |
| Entire crate (production regions only) | 13,365 / 13,808 | 96.7917% | 443 |
| `src/bonds/yield_risk.rs` (production) | 1,167 / 1,244 | 93.8103% | 77 |
| `src/bonds.rs` (production) | 814 / 832 | 97.8365% | 18 |
| `src/bonds/quote.rs` (production) | 139 / 139 | 100.0000% | 0 |
| `src/math/solver.rs` (production) | 743 / 743 | 100.0000% | 0 |
| `src/rates.rs` (production) | 857 / 865 | 99.0751% | 8 |
| `src/amounts/uint.rs` (production) | 302 / 304 | 99.3421% | 2 |
| `src/error.rs` (production) | 412 / 415 | 99.2771% | 3 |

The remaining 77 uncovered regions in `src/bonds/yield_risk.rs` are defensive
fixed-width capacity guards (`Wide<4>`, `Wide<33>`, `Wide<35>`, `Wide<37>`) and
unreachable internal fallback branches excluded by the contract's bit-width
invariants (256-bit 4-factor significands $< 2^{212}$, 33-limb sensitivity
accumulators, 35-limb risk accumulators, and 37-limb shifted division numerators).

## Package integrity

`cargo package --offline --locked --allow-dirty --no-verify` produced
`sabiquant-0.0.0.crate` (**222.0 KiB** compressed, **1,001.0 KiB** unpacked
content, **49** archive members: 47 repository files plus `.cargo_vcs_info.json`
and `Cargo.toml.orig`). The allowlist differs from the phase C settlement
archive solely by `src/bonds/yield_risk.rs`; all validation fixtures,
generators, and integration tests remain excluded.

Extracting `sabiquant-0.0.0.crate` into `/tmp/sabiquant-m8-yr-pkg` and running
`cargo test --offline --locked` against the standalone extracted manifest passed
**101** unit tests, **36** executable doctests, and **5** compile-fail doctests
with zero warnings or errors.
