# Contributing to SabiQuant

Thank you for your interest in SabiQuant. This file is the practical guide.
The full policy, including the review checklist and the governance model, is
in [docs/CONTRIBUTING_GUIDELINES.md](docs/CONTRIBUTING_GUIDELINES.md).

Everyone who takes part agrees to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
Report security problems privately as described in [SECURITY.md](SECURITY.md),
never in public issues.

## What you can contribute

- Reports of errors in the specifications, with the primary source that shows
  the error.
- Research input: primary sources for financial conventions and holiday
  authorities.
- Bug reports and numerical discrepancies.
  [Issue reporting](docs/CONTRIBUTING_GUIDELINES.md#2-issue-reporting) lists
  what each kind of report needs.
- Code for the milestone that is currently authorized (see the
  [roadmap](docs/ROADMAP.md#milestone-status)).

Discuss larger changes in an issue first. Specifications come before code: a
change in behavior needs an approved change to the specification, in the same
pull request or an earlier one.

## Licensing of contributions

SabiQuant is licensed under either of

- the [Apache License, Version 2.0](LICENSE-APACHE), or
- the [MIT license](LICENSE-MIT),

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in SabiQuant by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## Developer Certificate of Origin

Every commit must be signed off. By signing off, you certify the
[Developer Certificate of Origin 1.1](https://developercertificate.org/):
that you wrote the change, or otherwise have the right to submit it under the
project's licence.

Sign off by committing with `git commit -s`, which adds a trailer such as:

```text
Signed-off-by: Your Name <your.email@example.com>
```

Use your real name. To sign off commits you have already made, use
`git commit --amend -s` for the last commit, or `git rebase --signoff <base>`
for several.

## Independence and provenance

SabiQuant is an independent implementation. QuantLib is a behavioral
reference only.

- You may read QuantLib to understand behavior and to find edge cases.
- You must write SabiQuant code from SabiQuant's specifications and from
  primary sources. Never paste, translate or closely paraphrase QuantLib code,
  comments or test-data tables.
- Cite literature examples from their original publication. Paraphrase
  licensed publications such as the ISDA Definitions; never reproduce them.

Every pull request confirms this in its provenance attestation. Details are in
[LICENSING_AND_ATTRIBUTION §5](docs/LICENSING_AND_ATTRIBUTION.md#5-independence-and-provenance-policy).

## Rules for code

These rules come from the accepted architecture decisions and are checked in
review and, where possible, by CI.

- **No dependencies** of any kind. The package manifest declares none
  ([ADR-0002](docs/adr/ADR-0002-standard-library-only.md)).
- **No `unsafe` code** ([ADR-0001](docs/adr/ADR-0001-native-rust.md)).
- **No hidden state.** No global or thread-local state, no clock, no
  environment access, no I/O and no logging in the library
  ([ADR-0011](docs/adr/ADR-0011-explicit-state.md)).
- **No panics on any input.** Every operation that can fail returns `Result`
  with a typed error ([ADR-0007](docs/adr/ADR-0007-error-handling.md)).
- **Numerics** follow the [numerical correctness policy](docs/NUMERICAL_CORRECTNESS.md),
  and every comparison of real numbers names its tolerance policy.
- **Tests** cite the requirement and scenario IDs they verify
  ([TESTING_STRATEGY §3](docs/TESTING_STRATEGY.md#3-test-organization-and-naming-from-m1)).
- **Documentation** covers every public item, including its QuantLib
  correspondence ([API_DESIGN_PRINCIPLES §10](docs/API_DESIGN_PRINCIPLES.md#10-documentation-requirements)).
- **Changelog.** Update [CHANGELOG.md](CHANGELOG.md). Any change in numerical
  or financial output goes under "Behavior changes".

## Development workflow

You need Rust 1.99 or later, the minimum supported Rust version declared in
`Cargo.toml`, with the rustfmt and Clippy components. Before you open a pull
request, run the same checks as CI:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps --document-private-items
```

Where things go:

- Unit tests sit next to the code they test, in a `tests` module.
- Integration tests sit in `tests/`. Shared test support, such as the
  deterministic PRNG, the ULP and tolerance comparisons and the fixture
  reader, sits in `tests/support/`; the unit tests use it too.
- Reference fixtures follow the format in
  [TESTING_STRATEGY §7.3](docs/TESTING_STRATEGY.md#73-reference-fixture-specification)
  and live in `validation/fixtures/` (from M2).

If Clippy reports a lint that you must break deliberately, add an
`#[expect(lint, reason = "...")]` attribute on the smallest item possible and
explain why in the reason.

## Pull requests

- Keep each pull request to one focused change, linked to an issue, an open
  question or a milestone task.
- Fill in the pull-request template, including the checklist and the
  provenance attestation.
- CI must pass on every primary platform before a pull request is merged.
