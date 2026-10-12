# M5 local package review

| Field | Evidence |
|---|---|
| Review date | 2026-10-12, local engineering audit under delegated authority |
| Scope | Package contents, offline build, declared MSRV, dependencies, licence/provenance boundaries and release gates |
| Status | Packaging defects fixed; all source-frozen archive checks pass locally. Publication remains on hold |
| Initial snapshot | HEAD `61a15db45e57756a8b1e6f93e85a10ecbd0d81f3` plus uncommitted M4 work; `.cargo_vcs_info.json` correctly records `dirty: true` |
| Final local snapshot | HEAD `9aa59c237301e6d029eee2184fc24c07fb71a64f` plus the frozen local M5 API/package changes; `dirty: true`, version `0.0.0`, `publish = false` |

This is technical evidence, not the owner's API review, milestone acceptance,
publication approval or a claim of completed cross-platform CI. No repository
settings, Git history or release flags were changed, and nothing was published.

## Initial archive and offline verification

The declared Rust 1.99 MSRV was used directly: `rustc 1.99.0`, commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM 23.1.1,
`aarch64-apple-darwin`; Cargo 1.99.0. Build output was isolated below
`/private/tmp/sabiquant-m5-package-audit`.

`cargo package --list --offline --locked --allow-dirty` lists 21 files.
`cargo package --offline --locked --allow-dirty` creates the archive and
successfully verifies its extracted library build. The archived snapshot is:

- Path: `/private/tmp/sabiquant-m5-package-audit/package/sabiquant-0.0.0.crate`.
- SHA-256: `6f9f5191f420ec7277bcf0c6b0751542088f5acc1399bcc079284182151fac9c`.
- Size: 81,009 bytes compressed; 352,517 bytes of file contents.
- Contents: 13 Rust source files, both licence files, README, changelog,
  normalized manifest, original manifest, lockfile and Cargo VCS metadata.
- No `validation/`, `tests/`, `docs/`, `.github/`, `target/`, Python, wheels,
  C++ binaries or fixture data. No absolute or parent-traversing archive paths.

`LICENSE-MIT`, `LICENSE-APACHE`, `Cargo.lock`, `README.md` and `CHANGELOG.md`
match the workspace bytes exactly. Cargo's normalized manifest retains the
edition, MSRV, licence and `publish = false`; it declares the library target
and no build script. This dirty, version-0.0.0 archive is an audit artifact,
not a release candidate or reproducible clean-tag attestation.

Additional checks against the extracted archive pass:

| Check | Result |
|---|---|
| Library build performed by `cargo package` | Pass, offline and locked |
| Public documentation with `RUSTDOCFLAGS=-D warnings` | Pass |
| Packaged doctests | Seven pass, including rate/date/calendar/ICMA examples |
| Packaged `cargo test --lib` | Fails to compile: missing `tests/support/mod.rs` |

The initial failure is concrete: `src/lib.rs` conditionally imports
`../tests/support/mod.rs`, while the include allowlist excludes `tests/`.
Normal downstream builds and doctests succeed because that module is only
enabled for library unit tests. The full repository suite cannot stand in for
this archive test.

The packaged README also has relative links into omitted documentation,
validation and governance files. Those targets are absent in the extracted
archive; the rustdoc crate overview already uses repository URLs. This audit
does not assume that an external registry renderer will rewrite every README
link. Repository-qualified links can preserve access without shipping the
external validation material. This is a documentation packaging finding,
distinct from a library compilation failure.

## Local remediation

Under delegated local authority, the manifest include allowlist now explicitly
ships the four standard-library files below `tests/support/` and `clippy.toml`.
The support module remains `cfg(test)` and private; no integration tests,
reference data, Python, tooling binaries or third-party crates are added.
The Clippy configuration preserves the existing test assertion exceptions
and explicit-state/I/O restrictions. An isolated extracted-package experiment
confirmed that omitting it breaks `cargo clippy --all-targets -- -D warnings`,
including the fixture reader's otherwise unfulfilled lint expectation.

The README's 48 repository-relative Markdown links now use canonical
`https://github.com/iceman951/sabiquant/blob/main/` targets. There are 38 distinct
targets; every target exists locally, and no relative Markdown links remain.
The README explicitly says that unreleased local changes must reach the remote
repository before publication. This local check does not claim those remote
pages already contain the unpublished work.

## Final source-frozen archive

After Calendar/TARGET option APIs, context hashing, source examples and the M5
changelog were frozen, packaging was repeated with the updated allowlist.
The final local archive is
`/private/tmp/sabiquant-m5-package-audit-final/package/sabiquant-0.0.0.crate`:

- SHA-256: `ff8756a4a9bf5ffed647e28d5df3820f3b94531a24b5113975dcd03528d54b88`.
- 26 files; 94,876 compressed bytes; 405,453 bytes of file contents.
- 13 library Rust files, four shared Rust test helpers, Clippy configuration,
  both licences, README, final changelog, manifests, lockfile and VCS metadata.
- The only `tests/` files are `support/{mod,fixture,float,prng}.rs`. There are
  no integration-test targets, fixtures, validation tooling/data, research
  documents, CI files, dependencies, external binaries or archive path anomalies.
- Every archived original file, including source, support, configuration,
  README, final changelog, licences and lockfile, matches the workspace bytes.
  Cargo-generated normalized manifest and VCS metadata are checked separately.

All checks ran from the extracted package directory, with isolated output
directories, offline mode and the lockfile enforced. This prevents an external
workspace's test helpers or lint configuration from masking a missing file.

| Final extracted-package check | Result |
|---|---|
| Cargo package verification build | Pass |
| Library unit tests | 73 pass |
| Ordinary doctests | 11 pass |
| API misuse compile-fail doctests | Three pass |
| Clippy, all targets, `-D warnings` | Pass |
| Public and private documentation, `-D warnings` | Both pass |
| All dependency edges | SabiQuant alone |
| README packaged link shape | No relative Markdown links remain |

Source-group hashes cover sorted archive-relative paths. For each file, hash
its UTF-8 path, NUL, eight-byte big-endian byte length and file bytes, in that
order; this makes the grouping reproducible without archive timestamps:

| Shipped content | SHA-256 |
|---|---|
| Library source group, `src/` (13 files) | `cca3d69e5046d403ad9a5431b82c2830aa6f0fcfbbb5031a42426d88ebada788` |
| Shared support group, `tests/support/` (four files) | `7dbe37c5a4550696a610502d427bea3d67fe4d408d582e7936c408f6128b3d1e` |
| README | `e133155cea1261357f083f14c6f60653751a71bfaaafff370309e626e83aa802` |
| Original manifest | `5429ed1e39b4ef9870575ce7f0664d366dde9c5943aaa271a765a8afab4e0fcc` |
| Clippy configuration | `00f892e8d87e03c282b17c959c02aeaeb4a61f4e2f06f04df42b13c6032dc219` |

This archive records the frozen local M5 source, not the earlier M3 HEAD as
its current provenance. Its VCS metadata identifies `9aa59c2` and the dirty
state accurately. A later clean release commit/tag, version bump or shipped
file change requires a new archive and digest.

## Dependency, licence and provenance controls

`cargo tree --offline --locked --edges all --prefix none` contains SabiQuant
alone. Cargo metadata has one package, an empty dependency list and no
features; the lockfile contains exactly the same sole package. No normal,
development, build, target-specific or workspace dependencies are declared.
The generated manifest has `build = false`; the archive contains no build
script or external reference implementation. These observations support the
standard-library-only package policy; they do not certify financial results.

The manifest licence is `MIT OR Apache-2.0`; both complete root texts are in
the archive. The MIT text carries the selected 2026 SabiQuant Contributors
copyright line. Licence files, contribution terms and provenance attestation
are consistent with [ADR-0010](adr/ADR-0010-licensing.md) and
[CONTRIBUTING](../CONTRIBUTING.md). QuantLib runtime observations and their
Python generators remain repository-only, as required by
[ADR-0008](adr/ADR-0008-validation-strategy.md). This is an inventory and
policy-consistency check, not a legal opinion or forensic originality audit.

Six non-merge commits, including the commit that introduced the DCO policy,
were checked through the final captured HEAD `9aa59c2`; the initial snapshot
contained five. Every one has an exact matching author `Signed-off-by` trailer.
Earlier M0 commits predate that policy and are outside this count. The PR
template separately requires the no-copying provenance attestation. New
commits and the eventual release review still need their own checks; this
snapshot does not attest unpublished future work.

## Remaining release gates

| Gate | Current local evidence and remaining action |
|---|---|
| Package test/support policy | Fixed and verified: cfg(test) support/config exceptions preserve packaged unit tests and lint policy while reference material remains excluded |
| Clean release artifact | Final code/documentation review, clean signed-off commit/tag, then regenerate archive and record its contents/digest |
| Version and publication | Manifest remains `0.0.0` and `publish = false`. [ADR-0009](adr/ADR-0009-api-stability.md) requires an explicit owner publication decision before 0.1.0; no name reservation or publication was attempted |
| Primary platform CI | Workflow configures Linux, macOS, Windows and MSRV jobs plus format, Clippy, documentation and dependency checks; this local audit observed only macOS arm64 Rust 1.99. Current snapshot CI results remain required |
| API/financial/numerical acceptance | Owner API review, compatibility/exclusion table, traceability and behaviour-change review remain distinct from successful packaging; M4's measured guard tier is not a platform libm proof |
| DCO and provenance | Commit sign-offs and PR provenance attestation for the final release changes; archived reference exclusions must remain enforced |
| Security reporting | `SECURITY.md` names GitHub private vulnerability reporting. Actual repository setting was not checked or changed in this local audit |
| TARGET maintenance | Source contract explicitly requires a calendar maintainer appointment and annual source review before a market-calendar release; projection agreement alone does not extend verified coverage |

The CI workflow grants read-only repository contents access, pins checkout to
a commit and disables persisted credentials. DCO enforcement compares exact
author trailers on non-merge PR commits. It has no publishing step. Workflow
configuration is not evidence that a particular run passed.

## Reproduction

Use the declared toolchain and an isolated output directory; the dirty-tree
flag is appropriate only for a local review snapshot:

```sh
CARGO_TARGET_DIR=/tmp/sabiquant-package-review cargo package --list --offline --locked --allow-dirty
CARGO_TARGET_DIR=/tmp/sabiquant-package-review cargo package --offline --locked --allow-dirty
cargo tree --offline --locked --edges all --prefix none
cargo test --manifest-path /tmp/sabiquant-package-review/package/sabiquant-0.0.0/Cargo.toml --offline --locked --doc
cargo test --manifest-path /tmp/sabiquant-package-review/package/sabiquant-0.0.0/Cargo.toml --offline --locked --lib
cd /tmp/sabiquant-package-review/package/sabiquant-0.0.0
cargo clippy --offline --locked --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --no-deps
RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --no-deps --document-private-items
```

Re-run these checks against the final clean release candidate; neither local
audit digest above may be reused as its digest.
