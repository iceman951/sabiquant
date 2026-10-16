# Local binary64 boundary validation (M8 phase B)

Completed on 2026-10-16, Asia/Bangkok, on native macOS arm64 against the
local worktree based on `0ae9168`. This is engineering evidence for the
[binary boundary contract](M8_BINARY_BOUNDARY_CONTRACT.md), whose adopted
SHA-256 is `7f3c24a089e17115b9d85527797c79617a4e7e599f8bb4474aed5a7be2ccec31`.
It is not owner API sign-off or financial, settlement or release acceptance.
Phase C settlement adapters and the bond models remain open. Work stayed
local; no push or CI run is part of this report.

## Implemented scope and authorship

`DecimalAmount` gains four named conversions:

- `from_binary64_exact`: the exact bit-field rational of a finite binary64,
  or `ResultScale`/`OutOfRange`.
- `from_binary64_rounded`: that rational rounded once to an explicit quantum
  and mode, through a 1216-bit integer ratio.
- `to_binary64_nearest`: the correctly rounded nearest-even binary64, decided
  by integer comparisons only.
- `to_binary64_exact`: the same bits only when exactly representable,
  otherwise `Inexact`.

NaN and infinities fail first with `amount.non_finite`, which records the
existing `NonFinite` class and has no nested source. The private kernel is now
generic over its fixed limb count with unchanged algorithms. Phase A keeps
eight limbs and the rounded conversion uses nineteen. The public type census
stays 77; the domain error catalog is 112.

Claude Code adopted the contract and wrote the production code and the oracle
consumer. A `ValueError` cause was first specified for `NonFinite`; it was
replaced before any code depended on it, because `ValueError` is not `Eq` and
the phase A contract requires `AmountError: Eq`. Separate agents wrote the
Python oracle and the blind hand suite from the contract without reading the
phase B production code. Review results are in the
[implementation review](M8_BINARY_BOUNDARY_IMPLEMENTATION_REVIEW.md).

## Retained suites

| Suite | Evidence |
|---|---|
| `amounts_binary_oracles.rs`, 1 test | All 196,773 independent oracle rows: exact canonical amounts, binary64 bit patterns and every structured error field compared as a key/value map, with no nested source. Operation and expectation censuses are tallied over the row-validated outcomes; error operations come from the actual errors |
| `amounts_binary_hand.rs`, 21 tests | Literal exact/scale tables; NaN payload and sign classes and infinities under every mode and quantum; signed zeros; scale 37/38/39 and i128 edges; every subnormal leading bit; every exponent field with eight fraction patterns and both signs; half-quantum ties and neighbors at seven quanta; binary64 versus decimal `2.675`; tiny and huge inputs with `Exact` precedence; about 280,000 dyadic and 215,000 exponent-sweep quantization checks; outgoing tables, midpoint ties with binade carry and round trips; root error wrapping; absence of conversion traits; threads |
| Phase A suites | The 277,577-row amount oracle, the 18-test hand suite, kernel units and the catalog suites pass unchanged on the generalized kernel |

The oracle generator reproduces its fixture with `--check`, as does the
phase A generator. Every earlier fixture and generator is byte-identical
to `0ae9168`.

## Aggregate native checks

Every Rust command used Rust 1.99.0 from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`.
Each run had its own target directory under `/private/tmp/sabiquant-m8b-verified-*`.

| Check | Observed result |
|---|---|
| `cargo test --offline --locked`, warning-denied rustdoc | 675 passed in 76 groups: 93 unit, 543 integration, 39 doctests; no warnings |
| `cargo test --offline --locked --release`, warning-denied rustdoc | 674 passed: 92 unit, 543 integration, 39 doctests; no warnings |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Completed successfully |
| `cargo fmt --all --check`; `git diff --check` | Exit zero |
| Warning-denied public and private `cargo doc --no-deps` | Exit zero |
| Warning-denied `cargo check --all-targets` for x86_64 Linux and Windows | Exit zero; compile-only |
| Both amount generators' `--check` | `identical` |

The one debug/release difference is the inherited debug-assertion-only test.
Cross-compilation is not native Linux or Windows execution.

## Production-region instrumentation

`validation/coverage.py` ran 74 instrumented binaries; 636 tests passed.
Doctests are not instrumented, and no branch or input coverage is claimed.

| Production scope | Covered / regions |
|---|---:|
| Entire crate | 10,307 / 10,586 (97.3644%) |
| `amounts/binary.rs` | 267 / 285 |
| `amounts/uint.rs` | 302 / 304 |
| `amounts.rs` | 724 / 753 (phase A's 29) |
| `error.rs` | 376 / 379 (the inherited three) |

All 18 uncovered `binary.rs` regions propagate checked operations whose
failure the contract's width proofs exclude:

- **Exact incoming:** `5^t` for `t <= 38`, and `M·5^t < 2^142`.
- **Rounded incoming:** `10^s` within 1216 bits; shifts and products bounded
  by `A < 2^1151` and `B < 2^1201`; `count·Q <= A + Q`.
- **Outgoing:** shifts of at most 180 or 75 bits; division by a nonzero
  power of ten; `r < d`; a significand below `2^53 + 1`; a biased exponent of
  at least 896.

The oracle's measured maxima reach these bounds exactly: 1151, 1201, 180 and
142 bits. The two `uint.rs` regions are phase A's aligned-division
propagations. No OOM was forced and no private state was forged.

## Package integrity

`cargo package --offline --locked --allow-dirty` produced an actual archive.
A fresh extraction passed 93 unit tests, 34 executable and 5 compile-fail
doctests, strict Clippy and warning-denied public/private docs. The archive
has 44 regular files: 42 byte-checked originals and two Cargo-generated
files. Its allowlist differs from the phase A archive only by
`src/amounts/binary.rs`; no fixture or generator is packaged. Metadata shows
one package, no dependencies or features and `publish = false`. The archive
is 188,903 bytes compressed and 828,861 bytes of content. The packaged
README has no relative `docs/` link.

## Reproducibility pins

| Artifact | SHA-256 |
|---|---|
| Binary boundary contract | `7f3c24a089e17115b9d85527797c79617a4e7e599f8bb4474aed5a7be2ccec31` |
| `src/amounts/binary.rs` | `0f08ac8a3af1eae82dd5c2466869cb1c60fe451c5ed30b2bd4888755f646771c` |
| `src/amounts/uint.rs` | `1beb80cef9c539bfb54afe31fa0e57fa338c8e2604c2eaf6b2f4d81507e9f17b` |
| `src/amounts.rs` | `c60baccb65c4fdbb7f7bfff375af15f57743825dfd71bc962f2c4e99a6754dee` |
| Oracle generator | `346188ff94900c3a793c43248a272bb0e312fb3151518e7135ce743a6ef33e4e` |
| Oracle fixture, 22,676,994 bytes | `b0b7bda628d5eaa8b0118e5752da0b52445784e0a4ec94d3259ddd7f66bec667` |
| `tests/amounts_binary_oracles.rs` | `bbb1cea0a7ad4ab9af64c7a19d9feaad807d59878915dbea78a6e6f26d6b16c9` |
| `tests/amounts_binary_hand.rs` | `9a1ee1b3cb80e51a29e3c77d76a22f6ee6eca3ba662f1579699fb4a30a6fe4e8` |
| Native debug log, `/private/tmp/sabiquant-m8b-verified-debug.log` | `ae3f98bd66ea2b4f6e8fd061c1fae27d07899e8c644f9b300a560d3587e5b94d` |
| Native release log, matching `-release.log` | `cbaec1f7631c2067ff23c71ab05d38a0e9ccc594cf344ee11d30f17559088973` |
| `/private/tmp/sabiquant-m8b-verified-coverage/production-regions.json` | `bd1011423e0bc4531899f4251f35507019bb64f8f2e26ed243ff3901fbd49731` |
| `.crate` archive under `/private/tmp/sabiquant-m8b-package/target/package/` | `ee7017639ad98585bd65294eacca26da5e73d39993eb23cd72b7849e6e8257fd` |
| `/private/tmp/sabiquant-m8b-package/evidence.json` | `8d24038ee49f81a37d564e43da0a87d47aa6ab3b54b105df97bcbdcefc76361f` |

## Remaining work

Phase B fixes the meaning of supplied binary64 values only. Rounding a
computed binary64 is not a certificate for an ideal financial value near a
quantum boundary. Phase C settlement adapters (units, signs, quote
normalization, denomination and every rounding stage), currency policy, the
zero/fixed bond models and owner/financial/platform/release acceptance
remain open.
