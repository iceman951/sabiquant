# Local bond core validation (`sabiquant::bonds`)

Completed on 2026-10-16, Asia/Bangkok, on native macOS arm64 against the local
worktree based on `8bfa292`. This is engineering evidence for the
[bond core contract](M8_BOND_CORE_CONTRACT.md), whose adopted SHA-256 is
`009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c`. It is not
owner API sign-off or financial, settlement, or release acceptance. Bounded
yield-to-maturity root inversion, analytic duration/convexity, named market
yield conventions (UK DMO gilt, US Treasury, Canadian) and NI-01 phase C
settlement adapters remain open. Work stayed local; no push or remote CI run is
part of this report.

## Implemented scope

`sabiquant::bonds` provides twelve public types and two free quote functions:

- `BondKind::{ZeroCoupon, FixedRate}`
- `ExCouponBoundary::{OnOrAfter, After}` and `ExCouponTerms`
- `BondSettlement` and `CouponEntitlement::{Ceased, BuyerEntitled, ExCouponDetached}`
- `BondYieldConvention`
- `BondPriceKind::{Clean, Dirty}`, `BondPrice`, `price_per_100_from_amount`, and `amount_from_price_per_100`
- `Bond`, `BondPricingPlan`, `BondValuation`, and `BondError` (13 variants, wrapped as `Error::Bond`)

The public type census increases from 77 to 89; the domain error catalog
increases from 112 to 125.

## Retained suites

| Suite | Evidence |
|---|---|
| `bonds_core_oracles.rs`, 1 test | All 18,152 independent oracle rows (`TOL-E0` on quote conversion, entitlement, work units, `accrued_amount`, and structured errors; `<= 2` ULP on elementary discount valuation) |
| `bonds_core_hand.rs`, 7 tests | Constructor validation precedence across all 13 `BondError` variants; all 49 (7×7) ordinary/termination schedule business-day convention pairs; Forward, Backward, Zero, all 4 `SpecialRollRule` variants, and `OldCDS` schedules; scalar, `ActualActualIcma`, `ThirtyE360Isda`, `Thirty360Us`, and `Business252` (including `TargetCalendar` coverage guard failures) conventions; all 32 `Compounding` configurations; summation overflow (`dirty price reduction`, `accrued reduction`, `clean price reduction`); multi-threaded `BondPricingPlan` evaluation |
| `src/bonds/quote.rs` unit tests, 2 tests | Intermediate overflow/underflow rescue by single rounding; subnormal midpoint ties-to-even, subnormal-to-normal carry, normal binade carry, and `top == 1023` overflow carry |
| `public_api.rs` and `src/error.rs` | Exhaustive 89-type public API trait/send/sync census and 125-variant error catalog/Display/source checks |

The oracle generator reproduces its fixture byte-for-byte with `--check`
(`"check": "identical"`). Every earlier frozen fixture and generator remains
byte-identical to `8bfa292`.

## Aggregate native checks

Every Rust command used Rust 1.99.0 from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked --all-targets` and `--doc` | All unit, integration (`bonds_core_oracles`, `bonds_core_hand`, `public_api`, and all M1–M8 phase A/B suites), and 35+5 doctests passed with zero warnings |
| `cargo test --offline --locked --release` | All release tests passed with zero warnings |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Completed with zero warnings |
| `cargo fmt --all --check`; `git diff --check` | Exit zero |
| `cargo doc --offline --locked --no-deps --document-private-items` | Exit zero, zero warnings |
| `cargo check --offline --locked --all-targets` for `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-msvc` | Exit zero; compile-only |
| `python3 -I validation/generator/bond_core_oracles.py --check` | `"check": "identical"` |

## Production-region instrumentation

`validation/coverage.py` ran all instrumented binaries with zero failures:

| Production scope | Covered / regions |
|---|---:|
| Entire crate (table total) | 16,313 / 16,726 (97.53%) |
| `src/bonds/quote.rs` (production regions) | 139 / 139 (100.00%) |
| `src/bonds.rs` (production regions) | 767 / 787 (97.46%) |
| `src/error.rs` (production regions) | 388 / 391 (99.23%) |

All 20 uncovered production regions in `src/bonds.rs` are `try_reserve_exact`
allocation-failure branches or defensive error-propagation branches on partial
coupon accrual `[start, S) \subset [start, end)` whose full interval `[start, end)`
was already validated at bond construction (`0 <= E(S) <= A <= f64::MAX`). No
OOM was forced and no private state was forged.

## Package integrity

`cargo package --offline --locked --allow-dirty --no-verify` packaged 46 files
(869.5 KiB uncompressed, 196.3 KiB compressed), adding only `src/bonds.rs` and
`src/bonds/quote.rs` to the phase B package set; no test fixture or generator
is packaged.

## Reproducibility pins

| Artifact | SHA-256 |
|---|---|
| Frozen bond core contract (`docs/M8_BOND_CORE_CONTRACT.md`) | `009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c` |
| `src/bonds.rs` | `11d2e7d7f321a3c1a19e66d6842dc3d1f77d19e9a929f0316be62ce61b5e3599` |
| `src/bonds/quote.rs` | `fde910b6cebb4fc1df4a316d9d0a526e8e78052a8ccaff47bd7a1b2bcc5cc8a3` |
| Oracle generator (`validation/generator/bond_core_oracles.py`) | `afe27c0ce1a362492a98202c2882271cabbc3b1d510bb49dd3d5215460055b18` |
| Oracle fixture (`validation/fixtures/bonds/bond-core-oracles.fixture`) | `5a5dc4f4ec573d189b8b7f87aeaf335b5a253f1ad983633612dfbe2ae8ae7817` |
| `tests/bonds_core_oracles.rs` | `1c8e148154c5be2bd6fef843c265a0469e14d457fac9ab240059dc16920a602e` |
| `tests/bonds_core_hand.rs` | `530b72d3f7d98794f35042d152f5b7bd1dcd28f1b96268f7f8d4d8d00033ded3` |
