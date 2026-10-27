# ADR-0012: Test-Only Development Dependencies

| Field | Value |
|---|---|
| ADR identifier | ADR-0012 |
| Title | Test-only development dependencies |
| Status | **Accepted** on 2026-10-27 ([decision record](#decision-record)). Partially supersedes [ADR-0002](ADR-0002-standard-library-only.md) D1, D2 and D4 for development dependencies and developer tools only |
| Date | 2026-10-27 |
| Decision owner | Project owner |

## Context

[ADR-0002](ADR-0002-standard-library-only.md) forbids third-party crates of
every kind, including development and test dependencies. It records the cost
of that choice for testing: property tests have no automatic shrinking, and
benchmarks have no statistical tooling. The project owner decided to allow
third-party crates **for tests only**, while the library itself stays
standard-library-only.

Two facts shape the decision:

- Cargo never resolves a package's `[dev-dependencies]` for the packages that
  depend on it, so library users do not download or build them. A published
  manifest still lists the dev-dependencies that have a version requirement,
  but no test-dependency code is part of the library.
- The crate package ships `tests/support/**`, and the library's unit tests
  include those files through a `#[path]` module (`src/lib.rs`). Test support
  in those files must therefore stay standard-library-only.

## Decision proposal

**D1. The library stays standard-library-only.** The `sabiquant` package
declares no `[dependencies]` and no `[build-dependencies]`, has no build script
(`build.rs` or a `build` key), declares no `links` key, and is not a
procedural-macro crate. The runtime part of ADR-0002 and NFR-09 is unchanged.

**D2. Development dependencies are allowed under all of these conditions.**

1. Declared only in the root manifest's `[dev-dependencies]` table. Never in
   target-specific tables (`[target.….dev-dependencies]`), because every
   primary platform runs the same suite (NFR-04).
2. Taken from crates.io with a version requirement only: no `path`, `git`,
   `registry` or `workspace` sources, and no vendored copies (D6).
3. In one of two categories: **property testing** or **benchmarking**. A
   dev-dependency never serves as a numerical oracle (oracle values remain
   committed fixtures with provenance, [ADR-0008](ADR-0008-validation-strategy.md))
   and never replaces the tolerance policy
   ([NUMERICAL_CORRECTNESS §7](../NUMERICAL_CORRECTNESS.md#7-tolerance-policy)).
4. Listed in the allowlist (D5) with the same version requirement as in the
   manifest.
5. Used only by integration-test and benchmark targets (`tests/*.rs` and
   `benches/`). Never in `src/`, including `#[cfg(test)]` modules, and never in
   `tests/support/`, because both are part of the crate package.
6. No native code anywhere in its dependency graph: no `-sys` crates and no
   C toolchain helpers (`cc`, `cmake`, `pkg-config`, `bindgen`, `vcpkg`).
   A transitive build script is allowed only when the allowlist row names it
   and `deny.toml` lists it in `bans.build.allow-build-scripts`, which is
   otherwise empty. A transitive procedural macro is allowed only when the
   allowlist row names it; `cargo-deny` has no procedural-macro gate, so this
   rests on review.
7. No network access, clock or unseeded randomness at test time
   ([TESTING_STRATEGY §1](../TESTING_STRATEGY.md#1-principles)).

**D3. Developer tools.** In addition to the toolchain components of ADR-0002
D2, two third-party Cargo tools are allowed in CI:

| Tool | Version | Purpose |
|---|---|---|
| `cargo-deny` | 0.20.2 | Licences, security advisories, crate sources, banned crates and build scripts across the whole dependency graph (`deny.toml`) |
| `cargo-nextest` | 0.9.148 | An additional test runner with per-test timeouts and no retries (`.config/nextest.toml`) |

CI installs each with `cargo install --locked <tool>@<version>` from crates.io
and uses no third-party GitHub Actions. Neither tool is needed to build or test
the crate locally; `cargo test` stays the authoritative gate on every platform
and on the MSRV. Coverage instrumentation still comes from the toolchain alone
([TESTING_STRATEGY §2](../TESTING_STRATEGY.md#2-test-levels-and-tooling)).

**D4. Approval process for a dev-dependency.** This replaces ADR-0002 D4 for
development dependencies only. A crate is approved by adding a row to the
allowlist (D5) in a change that records:

1. the crate, its version requirement and its category (D2.3);
2. its licence, which `deny.toml` must accept;
3. its transitive dependencies (count, build scripts, procedural macros) and
   its maintenance status;
4. its security-advisory status;
5. why the in-house alternative carries more correctness or maintenance risk;
6. the project owner's explicit approval, with the date.

A new major version of an approved crate needs a new review of its row.
Runtime and build dependencies still follow ADR-0002 D4 unchanged.

**D5. Allowlist.** `tests/policy.rs` parses the table between the two markers
below. A row has exactly nine cells; the category is `property-testing` or
`benchmarking`; the approval cell starts with an ISO date. Anything else in
the block fails the policy test. The allowlist is empty: no crate is approved
yet.

<!-- policy:dev-dependency-allowlist:begin -->
| Crate | Version requirement | Category | Licence | Transitive dependencies | Maintenance | Advisories | Why in-house carries more risk | Owner approval |
|---|---|---|---|---|---|---|---|---|
<!-- policy:dev-dependency-allowlist:end -->

**D6. Reproducibility.** `Cargo.lock` stays committed and every build uses
`--locked`. Third-party source is never copied or vendored into the
repository, including with `cargo vendor`. Offline work uses Cargo's own cache:
run `cargo fetch --locked` once, then `--offline --locked`.

**D7. Enforcement.**

| Check | Where | Rule |
|---|---|---|
| Manifest structure | `tests/policy.rs` (every `cargo test`) | Fail-closed parse: only `[dev-dependencies]` declares dependencies (D1, D2.1–D2.2) |
| Allowlist | `tests/policy.rs` | Every dev-dependency is in D5 with the same version requirement (D2.4) |
| Lockfile | `tests/policy.rs` | Every other package comes from crates.io with a checksum and is not a native-code crate (D2.2, D2.6) |
| Placement | `tests/policy.rs` | No allowlisted crate is named in `src/` or `tests/support/` (D2.5) |
| No runtime or build graph | CI `production` job | `cargo tree -e normal,build` lists SabiQuant alone; the library builds and passes Clippy alone |
| Whole graph | CI `supply-chain` job | `cargo deny check` with dev-dependencies included: advisories, licences, crates.io sources, native helpers and build scripts (D2.2, D2.6, D4.2, D4.4) |
| Extra runner | CI `nextest` job | `cargo nextest run`, with no retries |

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Keep ADR-0002 unchanged | The owner chose to allow test-only crates |
| Amend ADR-0002 in place | Accepted ADRs are not edited in substance ([ADR process](README.md#process)) |
| One ADR per crate (ADR-0002 D4) | Too heavy for test-only crates; an allowlist row records the same evidence |
| Allow any test-only crate | Numerical oracles and tolerance helpers would bypass ADR-0008 and the tolerance policy |
| Allow target-specific dev-dependencies | Platforms would run different suites (NFR-04) |
| Keep the allowlist only in `deny.toml` | An empty `allow` list in `cargo-deny` admits every crate, so it fails open; a single table read by the policy test cannot drift from this ADR |
| Use the `toml` crate or `cargo metadata` in the policy test | A third-party parser would check the dependency policy for itself; `cargo metadata` needs a process, a `cargo` binary and possibly the network |
| Vendor dependencies for offline builds | Rejected by ADR-0002 and adds licence obligations |

## Advantages

- Property tests can gain automatic shrinking, and benchmarks statistical
  tooling, without changing the library's dependency graph.
- Licences, advisories and crate sources are checked automatically.
- Library users see no change: the runtime and build graphs stay empty.

## Disadvantages

- Contributors who run the full suite must download the approved crates once.
- CI takes longer: two tools are built from source.
- The published manifest lists versioned dev-dependencies, although users never
  resolve them.

## Consequences

- `docs/REQUIREMENTS.md` C-02 and NFR-09 still say "no dependencies of any
  kind". That file is pinned byte for byte by a frozen fixture, so it is not
  edited here. **This ADR supersedes the development and test part of C-02 and
  NFR-09; their runtime and build part is unchanged.** Updating the
  requirement text is separate work that follows the frozen-fixture process.
- The in-house PRNG, property runner, tolerance comparison and fixture reader
  remain the default test support and stay standard-library-only.
- `Cargo.lock` lists more than one package only once a crate is approved.

## Risks

- Supply-chain exposure through test builds. Mitigated by the allowlist,
  crates.io-only sources, the native-code ban and `cargo deny check`.
- The placement check (D2.5) searches source text for allowlisted names. It is
  a targeted check, not compiler enforcement; review still applies.
- Transitive procedural macros are gated by review only (D2.6).
- A test crate may raise its minimum Rust version above the MSRV. Then an older
  compatible version is pinned, or the MSRV change follows ADR-0009.

## Reconsideration triggers

- A needed test crate falls outside the two categories in D2.3.
- `cargo-deny` or `cargo-nextest` becomes unmaintained or changes its
  configuration format incompatibly.
- Cargo gains a way to omit dev-dependencies from published manifests.

## Related requirements and documents

C-02, NFR-04, NFR-09; [ADR-0002](ADR-0002-standard-library-only.md);
[ADR-0008](ADR-0008-validation-strategy.md);
[TESTING_STRATEGY](../TESTING_STRATEGY.md); `tests/policy.rs`; `deny.toml`;
`.config/nextest.toml`; `.github/workflows/ci.yml`.

## Decision record

| Date | Decision | Approval |
|---|---|---|
| 2026-10-27 | D1–D7 accepted. The allowlist starts empty; no crate is added | Project owner, after a recorded scope review of every option above |
