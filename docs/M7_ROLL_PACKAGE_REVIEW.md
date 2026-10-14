# Independent local civil-roll package review

Reviewer: `/root/period`, 2026-10-14. This records actual local archive checks
and a read-only audit of another agent's production roll implementation. It
does not grant owner, financial, native-platform or release acceptance.

## Frozen snapshot and artifact

The package contains HEAD `ffa7fe36da3ae1c6f7a89686a765d6257a7e46b8` plus
the frozen local civil-roll changes (`dirty: true`, version `0.0.0`,
`publish = false`). No Git operation or publication was performed.

| Original file | SHA-256 |
|---|---|
| `src/schedule/roll.rs` | `28af37afc2ba94fffa5afdd673cd8d147d43224eb11d9177db10b96225739b38` |
| `src/schedule.rs` | `0cd9ac70a575756347acc44a86e9331ec08b93eb129c1b5a4402f77514035476` |
| `src/error.rs` | `bb3e7661f6867ed236c3513e4b5a89c0e0545362fa190f2607dc468802245c6a` |
| `src/lib.rs` | `cebdaa7eb0b124284754404fee6ac2f1c8a2ee9d3a931b29dc46c895cd7eeb4a` |
| `README.md` | `e966439fae0b69522b6be582f1ac2fb7c198201a93c22241da8afe85a86e7aba` |
| `CHANGELOG.md` | `0ca8d65d2408f15d87824b789e43048075f6bf702be6d648ceab808cb5c2a7d4` |
| `Cargo.toml` | `5429ed1e39b4ef9870575ce7f0664d366dde9c5943aaa271a765a8afab4e0fcc` |

The actual archive is
`/private/tmp/sabiquant-m7-roll-package-target/package/sabiquant-0.0.0.crate`:

- SHA-256 `177b64c2238726bc8781a705775ba14e1eb684a678ab9b03abbf8547adaa132e`.
- 36 regular files; 152,746 compressed bytes and 666,779 file bytes.
- Exactly the allowed library/package files: 23 source files, four owned
  `tests/support/` files, lint configuration, manifests/lockfile, licenses,
  README/changelog and generated Cargo metadata.
- All 34 original files match workspace bytes, including the original
  manifest retained as `Cargo.toml.orig`. No integration tests, fixtures,
  research, generators, dependencies or external binaries are shipped.
- No absolute/traversing paths, duplicate members or links. Both license
  files match their originals. The VCS metadata retains the stated HEAD
  and dirty status.

Detailed evidence and log hashes are saved locally in
`/private/tmp/sabiquant-m7-roll-package-evidence.json` (22,358 bytes), SHA-256
`800ebf4bf13f18f5b65fc489fa9541996fcefed31db1c19e97d909374cd31dd6`.
These temporary files are audit artifacts, not a release archive or permanent
repository artifact. All packaged originals remained unchanged after checks.

## Actual extracted-package verification

Host: macOS arm64; `rustc 1.99.0 (b940084d7 2026-09-28)` and
`cargo 1.99.0 (5f94df478 2026-08-27)`. Commands ran offline and locked, with
fresh extraction and target directories under `/private/tmp/`.

| Check | Actual result |
|---|---|
| `cargo package --list --offline --locked --allow-dirty` | Pass; exact archive allowlist |
| `cargo package --offline --locked --allow-dirty` | Pass, including Cargo's default verification build |
| Extracted `cargo test --lib` | 80 passed |
| Extracted `cargo test --doc`, warnings denied | 28 passed: 25 ordinary and three compile-fail |
| Extracted `cargo clippy --all-targets -- -D warnings` | Pass |
| Public/private `cargo doc --no-deps`, warnings denied | Both pass |
| Extracted Cargo metadata/lockfile | One package; zero dependencies/features/build script; edition 2024, MSRV 1.99, MIT OR Apache-2.0; publishing disabled |
| Packaged README links | No relative Markdown targets into omitted files |

The packaging command's 51 warnings all concern deliberately omitted
integration tests. The extracted build/tests/lint/docs emitted no warnings.
These checks establish the package's local self-contained build behavior,
not numerical/financial accuracy or native Linux/Windows execution.

## Independent source and civil evidence

No arithmetic or adopted-contract defect was found in the frozen roll source.
Positive strides are at most `12*i32::MAX`. Each loop examines a coordinate
within the supported civil month range; its next coordinate is at most
`119987 + 12*i32::MAX`, so i64 arithmetic remains safe. Month progress makes
dates strictly increasing. A required projected anchor/end uses checked Date
arithmetic with retained role/input/cause; an unused coordinate beyond resolved
end is rejected before constructing a Date. No error is swallowed to fit a cap.

The signed congruence regularity predicate is valid on reachable outputs.
A requested twentieth start before its retained anchor is off-roll; a valid
on-roll start is already the anchor. Wednesday outer endpoints are checked
against the actual third Wednesday. Thus a negative congruent off-lattice
outer coordinate cannot acquire a false Regular flag. Requested/resolved
union coverage protects both original and projected TARGET spans. Provenance
stores original length/unit and requested range, and is attached only after
successful adjustment and strict-order validation.

The independent Python oracle enumerates civil days/months and selects span
members by congruence, without production recurrence or QuantLib outputs.
Its Rust consumer additionally counts Wednesdays using a separate Gregorian
ordinal formula. It checks 122,233 rows (122,183 distinct input tuples and
50 repeats), including all 4,800 months of 2000..2399 under every rule:
116,811 values, 5,290 collapsed resolved ranges, 54 required Date errors and
78 invalid tenors. It verifies 354,671 boundaries, 114,065 Regular and
123,795 Stub intervals, provenance, and exact/cap−1/cap+1 behavior for each
successful row. Debug, release, strict focused Clippy and generator `--check`
passed. Oracle hashes and exact profile counts are retained in the JSON;
fixture SHA-256 is
`4fdc3f29b41dc09f8a07c175a8cd889d4a16e4ce9a967e4c2db0af2c34f7cc30`.

The oracle's ordinary spans are at most 25 months; its huge-tenor controls
exercise unused-candidate range handling. Whole-supported-range resource
tests are separate evidence, assigned to the public hand suite after this
review identified the gap. Allocation failures are not tested by forcing
system OOM. Local coverage and successful checks do not prove universal
performance, platform accuracy, market conventions or QuantLib equivalence.
Same-roll ICMA and CDS/product families remain follow-on work; the current
roll helper's typed ICMA exclusion and other release gates remain explicit.
