# Local M7 explicit coupon-date validation

This report extends the saved schedule core at `86d5bb8` under the owner's
continuing local-only development authority. It records the explicit
first/penultimate coupon-date tranche, not completion of M7, market convention
approval or publication. The [adopted contract](M7_EXPLICIT_STUB_CONTRACT.md)
and [independent source/API review](M7_EXPLICIT_API_REVIEW.md) define and assess
the implementation separately from public-reference observations.
The [independent traceability audit](M7_EXPLICIT_TRACEABILITY_AUDIT.md) checks
requirements, executable assertions, identities, prior fixture bytes and links.

## Implemented behavior and identity

`ExplicitStubDates` is a private-field immutable Copy/Eq/Hash value. Its
constructor requires at least one role and rejects first after penultimate.
Both supplied dates must be strictly interior to the builder's original
range. Equal roles intentionally identify one shared coupon, with two outer
intervals and no middle interval. It is never an adjustment-collision repair.

`ScheduleBuilder::build_explicit` accepts positive stored tenor, Forward or
Backward, explicit roles, EOM and a final-boundary cap. Forward anchors at
first-or-start; Backward at penultimate-or-end. The other middle endpoint must
be on that exact original-anchor lattice. Month clamping never feeds a new
anchor, EOM activates only from the named anchor's calendar month end, and
supplied coupon dates remain raw inputs. Outer intervals are Regular only if
their endpoint is the adjacent signed quasi-boundary of this same lattice;
otherwise Stub. The operation does not invent a hidden internal stub.

Classification-only quasi dates beyond the civil range imply Stub because no
in-range endpoint can match them. This does not allow a future required ICMA
reference date to be missing. No frequency, accrual convention or ICMA context
is inferred. Whole original TARGET coverage precedes generation, and all
actual boundary adjustments retain the existing checked candidate policy.
Ordinary/final conventions remain separate and collisions/reversals fail.

Explicit roles are retained in `Schedule` definition equality/hash; automatic
stub policy is absent. Resource caps do not define contractual identity.
`lattice_anchor()` exposes the original raw seed, or None for Zero. Automatic
and Zero values have absent explicit-role metadata. The root catalog covers
**74 unique nested error variants**, including four new schedule kinds. The
crate has **57 public struct/enum declarations**, with trait assertions and
owned-context checks distributed across the API tests.

Validation precedence is cap, rule, tenor, EOM compatibility, first then
penultimate interior placement, original TARGET span, generation and
chronological adjustment. The final cap includes all outer/coupon/middle
boundaries. Independent step/count variables prevent an already stored outer
endpoint from skipping the first recurrence. Initial and strict-interior lower
bounds reserve the opposite endpoint and any trailing outer boundary. Errors
retain the original supplied cap; no invalid reduced-cap error escapes.

## Independent civil and public-reference evidence

The new [civil generator](../validation/generator/schedule_explicit_oracles.py)
selects dates across the bounded middle using independent integer
anchor-coordinate membership. It enumerates civil days for day/week units and
calendar months for month/year units, without reusing the Rust recurrence
counter or stopping path. Outer flags use adjacent coordinate membership.
Input endpoints may use direct civil shifts; expected outputs do not use Rust
or QuantLib. Its fixture is consumed by
[schedule_explicit_oracles.rs](../tests/schedule_explicit_oracles.rs).

| Civil fixture evidence | Count / scope |
|---|---|
| Total | 21,883 rows; 17,789 values and 4,094 contextual IrregularExplicitMiddle errors |
| Roles | 5,799 first-only; 5,805 penultimate-only; 5,795 both; 4,484 equal |
| Units | 4,081 Days; 4,014 Weeks; 7,524 Months; 6,264 Years |
| Direction / EOM | 10,971 Forward; 10,912 Backward; 14,989 Ignore; 6,894 Preserve |
| Civil classes | Years 1, 99, 100, 400, 1900, 2000, 2100, 9999; month/day/tenor variation and 72 literal civil-edge cases |
| Exact observations | 70,819 successful boundaries; 35,818 Regular and 17,212 Stub flags |
| Resources | Every success checks exact cap, one smaller with exact original cap/required count, and equality at a larger cap |
| Error provenance | Every irregular row checks direction, stored tenor, named anchor, opposite endpoint and last regular boundary |

The separate [API/property suite](../tests/schedule_explicit.rs) adds 13 tests,
including **230,400** independent civil-span preparations over the complete
2000–2399 Gregorian cycle: 227,709 successful schedules, 2,691 irregular
middle rejections and 682,230 exact interval-flag assertions. Every successful
preparation also rejects cap minus one. It exercises all units, both directions, all role
forms, month-end Preserve/Ignore, absence/order/interior precedence, named
clamp provenance, implicit-middle rejection, huge tenors, civil edges, all
49 adjustment-convention pairs, full TARGET span and candidate guards,
nested error sources, definition collections and 256 concurrent rebuilds.

The [resource property suite](../tests/schedule_properties.rs) now includes
explicit first=MIN+1/penultimate=MAX−1 construction across all **3,652,059**
civil dates in both directions. It checks every ordinal, Regular flag and
anchor, then requires an original-cap error with one fewer allowed boundary.
The original automatic full-span test is retained.

The new [public probe generator](../validation/generator/schedule_explicit_probes.py)
records **29** isolated bounded QuantLib 1.43 observations: 25 values,
four constructor errors and two unordered reversed-role vectors. Each worker
is isolated with `-I`, attests the established wheel/module bytes before
import, and has a ten-second timeout. Input spans are positive, inside 2026
and at most 366 days; output has 2–400 dates in 2025–2027. Worker/process or
timeout failure aborts before fixture writing. This attests the executed
binary files, not their upstream source revision.

The [reference consumer](../tests/schedule_reference.rs) checks all 29:

| Classification | Rows |
|---|---:|
| Exact shared adjusted dates and contractual flags | 13 |
| Reversed roles: unordered reference output, local typed rejection | 2 |
| Off-lattice middle: reference internal residual, local contextual rejection | 3 |
| Strict-interior local rejection | 8 (four reference errors; four endpoint-equality values) |
| Same dates, first flag differs on the named February-28 lattice | 1 |
| Business-only-EOM Preserve exact date distinctions | 2 |

The five previously deferred quarterly coupon rows in the original unchanged
46-row set now also pass exactly. That original consumer has 30 shared values,
the same four EOM/four collision/one Zero distinctions and seven unsupported
rules. Five quarterly examples are duplicated between probe sets, not 10
independent financial examples. All differences have exact predicates in
[the discrepancy ledger](../validation/DISCREPANCIES.md); no date tolerance,
deduplication or numerical-policy widening is used.

The cumulative fixture inventory is **982,698 rows in 46 files**. Earlier
fixtures and generators remain byte unchanged. Counts describe cases rather
than branches, mathematical proofs or universally equivalent schedules.

| Reproduction identity | SHA-256 |
|---|---|
| Schedule source | `be91b84dcaecd03b9ab7c667a742c3dffc5d27b5a04961a2badc49017e54ba06` |
| Civil generator | `a082c47a2f0b6c98d312dcc52105395b153e6619d5eff107b3619d2c651448ce` |
| Civil fixture | `0e4c8ef37e7980e29dcf692726a3d1ebed8b0a53bf5cf7d7f0a4cc010fc83f8c` |
| New public probe generator | `71fa299c315bb01c5836326b310632ae4c3ebf4e7ad9b685a74f1b5c9e9eb819` |
| New public fixture | `716bbda036e249bc2e36d8ae4c0f9049b4520515eac4e7f26e8d299a6f1d0880` |
| Frozen attestation helper | `2c48705c96a6311922e8483cc47ae383f032555fd56f4c3f53567ca605725d6d` |
| QuantLib wheel | `85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c` |
| Installed three-module manifest | `e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d` |

## Native, coverage and package checks

The final Rust 1.99.0/macOS ARM64 aggregate passes **424 debug** and
**423 release** tests across **46 result groups**, with zero failed/ignored
tests or warnings. These totals include **25 doctests** (22 ordinary and three
compile-fail) and **326 integration tests**; the library has 73 debug unit tests
and 72 release unit tests because a debug-assertion-only test is omitted in
release. Final focused independent runs verify the pinned Gregorian counters.

Formatting, strict all-target Clippy and warning-denied public/private rustdoc
pass. All four schedule generators' `--check` runs pass. Linux GNU and Windows
MSVC all-target checks pass with warnings denied; they are cross-compilation,
not native executions. Final aggregate logs are
`/private/tmp/sabiquant-m7-explicit-final-debug.log` and
`/private/tmp/sabiquant-m7-explicit-final-release.log`.

The native LLVM instrumented unit/integration run passes. Its production
source-region report at `/private/tmp/sabiquant-m7-explicit-coverage` covers
**816/846 schedule regions (96.45%)**. Embedded unit-test modules are excluded;
doctests are checked separately. This is region coverage, not branch or
exhaustive-input coverage. The 30 uncovered regions are fallible allocation
failures/propagation and required-date failures excluded by the supported-range
proof. They are not filled by corrupting private invariants or forcing OOM.
The independent API/source review assesses the cap and range proofs separately.
Earlier numerical modules retain their existing measured coverage and budgets.

Independent extracted-package verification passes Cargo's default package
verification build, **73 unit tests**, **25 doctests**, strict all-target Clippy
and warning-denied public/private documentation. The frozen archive has
**32 files**, **133,174 compressed bytes** and **579,057 content bytes**, SHA-256
`0eb675bca728b9fc3b02015a4a44c50c5cc34407833c4ed50a7e78ee5d7d07de`.
It records VCS base `86d5bb831bef6a5c0eebc70c195a777eb17ebe44` with dirty=true,
the pre-save explicit source snapshot. All 30 comparable original files match
the workspace; only the normalized manifest and generated VCS metadata are
exceptions. Source SHA remains the one recorded above.

Archive path:
`/private/tmp/sabiquant-m7-explicit-package-target/package/sabiquant-0.0.0.crate`.
Per-file hashes and terminal results are in
`/private/tmp/sabiquant-m7-explicit-package-evidence.json`. The allowlist contains
library/licence/readme/changelog material, lint config and four owned test
support files. Integration suites, research docs, fixtures, generators and the
reference wheel are excluded. Version stays 0.0.0 with no dependencies and
publish=false. Canonical main-branch documentation links require separately
reviewed remote availability before release; local existence is not publication.

No push, draft PR, external CI run, publication or formal milestone acceptance
is implied by any local result. Evidence collection began on 2026-10-12 UTC
and crossed into 2026-10-13 in Bangkok.

## Remaining work and reproduction

Generated ICMA references were pending at this explicit-date snapshot and are
now covered by [ICMA validation](M7_ICMA_VALIDATION.md).
Seven specialized roll families,
cash flows/coupons/legs/NPV/accrued interest and NI-01 exact settlement remain
separate work. Formal dependency/milestone/owner acceptance, primary-platform
native CI, security-setting verification and TARGET source/maintainer gates
remain open. The independent source review found no arithmetic/invariant
defect; two outdated rustdoc descriptions were corrected. No private invalid
state or system allocation failure is manufactured to inflate coverage.

```sh
cargo test --offline
cargo test --offline --release
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc --offline --no-deps --document-private-items
python3 validation/generator/schedule_oracles.py --check
python3 validation/generator/schedule_explicit_oracles.py --check
python3 validation/generator/schedule_probes.py --quantlib-wheel PATH_TO_WHEEL --check
python3 validation/generator/schedule_explicit_probes.py --quantlib-wheel PATH_TO_WHEEL --check
python3 validation/coverage.py --output NEW_REPORT_DIRECTORY
cargo package --offline --allow-dirty
```

Use Rust 1.99 and a new coverage output directory. External probe regeneration
requires the independently attested local reference installation; ordinary
Rust validation remains offline and std-only.
