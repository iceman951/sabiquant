# Local exact decimal amount validation (M8 phase A)

Completed on 2026-10-16, Asia/Bangkok, on native macOS arm64 against the
local worktree based on `4da3f43`. This is engineering evidence for the
[adopted amount contract](M8_AMOUNT_CONTRACT.md), whose frozen SHA-256 is
`954571eb64be97e71c68877c8e5c705f5ca9312a2109d6b54f6a1883aead7df1`. It is not
owner API sign-off or financial, milestone or release acceptance. Phases B
(explicit exact/rounded binary64 boundary) and C (financial settlement
adapters), and the bond APIs, remain open. Work stayed local; no push or CI
run is part of this report.

## Implemented scope and authorship

Public module `amounts` adds `DecimalAmount`, `AmountQuantum`,
`AmountRounding` and `AmountError`. Together they provide:

- canonical i128 coefficient/scale values;
- exact parsing and Display;
- checked add, subtract, negate and multiply;
- quantization under seven explicit modes, with no default;
- single-rounding ratio, multiply-ratio and two-decimal multiply-divide;
- an exact permutation-invariant slice sum.

No currency, minor unit, clock or global rounding context exists. The root
`Error::Amount` wraps the domain error. The public type census is 77 and the
domain error catalog 111.

Codex adopted the contract and wrote the production draft. Separate Codex
agents wrote the eight-limb integer kernel `src/amounts/uint.rs`, the blind
hand suite and the independent Python oracle. Codex stopped at a usage limit
before integration was finished. Claude Code then took over. It found that the
oracle generator wrote a nonstandard `---` fixture layout, which the shared
reader rejects, and changed only the serialization. The 277,577 records stayed
byte-identical. Claude Code then wrote the oracle consumer and ran the
evidence below. Independent read-only review results are in the
[implementation review](M8_AMOUNT_IMPLEMENTATION_REVIEW.md).

## Retained suites

| Suite | Evidence |
|---|---|
| `amounts_oracles.rs`, 1 test | All 277,577 rows of the independent Python integer/Fraction oracle, cross-checked against two Decimal precisions. Exact canonical values, Display text and round trips, predicates, comparisons and mode identifiers are compared. Every structured error field is compared as a key/value map. Each row's outcome class must equal its expectation; operation and expectation censuses are tallied from those validated rows, and the error-operation census from actual error values |
| `amounts_hand.rs`, 18 tests | All 39 scales and signed limits; first-byte grammar errors; every string of length 0–5 over a seven-symbol alphabet against an independent grammar; parser precedence; Display precision; the mode registry; aligned arithmetic; normalization rescue; even/odd ties across scales and modes; arbitrary quanta; Inexact-before-range; zero denominators before shortcuts; coprime and widest fused paths; the fused price discriminator; permutation-invariant sums; numeric Ord/Hash; error context and root source; thread sharing |
| `src/amounts/uint.rs` units, 8 tests; `src/amounts.rs` unit | Kernel carries, borrows, shifts, multiplication and aligned division |
| Existing catalog suites | Public trait census 77; root error catalog 111 with source chains |

The oracle generator reproduces its fixture with `--check`. Every earlier
fixture and generator is byte-identical to `4da3f43`.

## Aggregate native checks

Every Rust command used Rust 1.99.0 from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`.
Each run had its own target directory under `/private/tmp/sabiquant-m8-verified-*`.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked`, warning-denied rustdoc | 650 passed in 74 groups: 93 unit, 521 integration, 36 doctests; no warnings |
| `cargo test --offline --locked --release`, warning-denied rustdoc | 649 passed: 92 unit, 521 integration, 36 doctests; no warnings |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Completed successfully |
| `cargo fmt --all --check`; `git diff --check` | Exit zero |
| Warning-denied public and private `cargo doc --no-deps` | Exit zero |
| Warning-denied `cargo check --all-targets` for x86_64 Linux and Windows | Exit zero; compile-only |
| `python3 -I validation/generator/amount_oracles.py --check` | `identical` |

The one debug/release difference is the inherited debug-assertion-only test.
Cross-compilation is not native Linux or Windows execution.

## Production-region instrumentation

`validation/coverage.py` ran 72 instrumented binaries; 614 tests passed.
Doctests are not instrumented, and no branch or input coverage is claimed.

| Production scope | Covered / regions |
|---|---:|
| Entire crate | 10,000 / 10,261 (97.4564%) |
| `amounts.rs` | 708 / 737 |
| `amounts/uint.rs` | 278 / 280 |
| `error.rs` | 376 / 379 (the inherited three) |

The 29 uncovered `amounts.rs` regions have identified limits:

- 27 are `?` propagations of checked 512-bit kernel steps. These cover
  magnitude and alignment conversions, widening multiplications, aligned
  addition and subtraction, the sum accumulator and final combine, the
  rounding increment, and one division whose zero divisor was rejected
  earlier.
- Each such operand stays inside the contract's width proof. The oracle's
  maximum measured widths are 381 numerator and 508 denominator bits for the
  fused path.
- The remaining two are the post-validation quantum constructor and a UTF-8
  guard in ASCII-only Display.

The two `uint.rs` regions propagate checked shift and subtract steps inside
aligned division whose operands are already ordered and bounded. No OOM was
forced and no private state was forged.

## Package integrity

`cargo package --offline --locked --allow-dirty` produced an actual archive.
A fresh extraction passed 93 unit tests, 31 executable and 5 compile-fail
doctests, strict Clippy and warning-denied public/private docs. The archive
has 43 regular files: 41 byte-checked originals and two Cargo-generated
files. Its allowlist differs from the US/AFB archive only by
`src/amounts.rs` and `src/amounts/uint.rs`; no fixture or generator is
packaged. Metadata shows one package, no dependencies or features and
`publish = false`. The archive is 184,775 bytes compressed and 813,493 bytes
of content. The packaged README contains no relative `docs/` link; every
documentation link in it is repository-qualified.

## Reproducibility pins

| Artifact | SHA-256 |
|---|---|
| Frozen contract | `954571eb64be97e71c68877c8e5c705f5ca9312a2109d6b54f6a1883aead7df1` |
| Oracle generator | `6fbc2e883e9fe22735b354df6283315ed9a710c540e6679ca7d8ff17e69bd00f` |
| Oracle fixture, 47,836,843 bytes | `9e0709ceb9a7ef539d82d70760ad9beabe24f4ab1a3d6f2a36e466c6ef3f40cd` |
| Oracle records only | `aa9d4dc530b333b34b3a4754b18d17a26c298050336bb4ead405b9e378da0d6b` |
| `.crate` archive under `/private/tmp/sabiquant-m8-package/target/package/` | `859f3e897f8e85acc932ae567b0c19672c6699f04dc44f0b6eb23f8cfcaff149` |
| `/private/tmp/sabiquant-m8-package/evidence.json` | `853d8a20c7f7848a63bfc2083184700bf144aca7c9230a293731878014d4bba7` |

## Remaining work

Phase A supplies exact decimal values only. The explicit binary64 boundary,
financial settlement adapters, currency/minor-unit policy, zero and fixed
bonds, settlement/entitlement, clean/dirty price, accrued interest, yield,
duration and convexity remain open, as do owner/financial acceptance, native
primary-platform execution and release gates.
