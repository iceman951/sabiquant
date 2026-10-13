# Local M7 generated ICMA-reference validation

This tranche follows the explicit coupon commit `d570b45` under continuing
local-only authority. It implements
[the adopted reference contract](M7_ICMA_REFERENCE_CONTRACT.md), informed by
[primary-source research](M7_ICMA_REFERENCE_RESEARCH.md). Its
[independent source/API review](M7_ICMA_API_REVIEW.md) and
[traceability audit](M7_ICMA_TRACEABILITY_AUDIT.md) remain separate from owner,
financial-convention, platform and release acceptance. Full M7 is not complete.

## Behavior and checked provenance

`Schedule::build_icma_context(frequency,max_periods)` returns an owned immutable
ActualActualIcma with ordered positive contiguous DeterminationPeriods. It
requires a periodic schedule, a supplied named periodic frequency and
normalized tenor equivalence. Zero/Once and mismatches fail; 12M/1Y,
7D/1W,14D/2W,28D/4W can match while the source Schedule retains stored units.
The conventional annual counts 1,2,3,4,6,12,13,26,52,365 are explicit; calendar
year day counts do not change them. Day/week support is an analytical local
definition, not a claim that the handbook approves every market roll.

The helper extends the SAME retained directional/coupon anchor with fixed
calendar-EOM activation. It selects the minimal hull qL<=raw start<raw end<=qR
and every quasi-period inside it, including boundaries omitted by Long.
Exact lattice endpoints have no gratuitous extra period. Equal explicit roles
can cover multiple periods on both sides. Adjusted payment dates and business
conventions do not rewrite references or change accrual.

Unlike the unused schedule candidate, every reference extension is REQUIRED:
actual checked Date errors preserve anchor, signed scaled offset and operation.
The Schedule wrapper uses phase `ICMA reference extension` and a GLOBAL
boundary-generation position, not a signed lattice index. Both directions
share the final-period budget 1..3,652,058. The negative phase reserves a
necessary positive-side period before its next date computation; cap failures
retain the original caller cap. Steps/products fit the existing wide integers.
Boundary and period-vector growth reserve fallibly before construction.

Once a finite minimal hull exists, the entire retained TARGET span, including
both quasi endpoints, gaps, overrides and joints, is checked with phase
`ICMA reference coverage`. Endpoint-first underlying coverage-error semantics
are preserved. Projection remains explicit on the source Schedule; a context
does not turn projected dates into verified history. Later ICMA queries use
only immutable references/coverage and perform no calendar queries. Its
unrestricted actual `day_count` remains distinct from covered `year_fraction`.

Validation order is cap, recurring provenance, periodic frequency, normalized
match, bounded Date/cap/allocation generation, whole reference coverage, then
period/context validation. Structural ICMA causes are wrapped without unwrap.
The root exhaustive catalog now has **80 unique nested variants**, including
**22 schedule kinds**. No public type is added: all **57** struct/enum
declarations retain their compile-time trait checks. Definition identity and
clone/hash/thread behavior of the existing ICMA value remain unchanged.

## Independent dates, rational accrual and boundaries

The [external oracle](../validation/generator/schedule_icma_oracles.py) selects
congruent dates throughout a bounded civil envelope, then takes the nearest
bracketing points around raw accrual. It does not reuse Rust's signed
extension loops, early stopping or cap decisions. It reuses the frozen Python
civil-coordinate membership definition from the explicit oracle, separate
from Rust recurrence. Accrual sums independent exact Fraction overlaps.
The [consumer](../tests/schedule_icma_oracles.rs) reconstructs those reduced
ratios with checked i128 integer products, checks every reference boundary
and exact cap/cap-minus-one, and compares f64 fractions under existing TOL-E2.

| Fixture evidence | Actual scope/count |
|---|---|
| Query rows / contexts | 36,092 / 18,046; two rows per context |
| Builder forms | 7,734 automatic and 10,312 explicit contexts; each of seven input roles has 2,578 contexts |
| Direction | 9,023 Forward and 9,023 Backward contexts |
| Frequencies/aliases | All ten independently specified periodic names, including 12M/1Y and all three day/week stored aliases |
| Queries | 27,069 forward; 4,512 reversed; 4,511 zero; 8,158 full-hull rows |
| References | 147,388 period comparisons across rows; contexts have 2,3,4 or 6 periods |
| Exact ratio size | Maximum numerator 20 bits and denominator 18 bits; checked i128 arithmetic is much wider |
| Measured accuracy | Maximum **1 ULP**, within unchanged **4 ULP TOL-E2**; zero remains exact positive zero |

There are 998 deliberate repeated second-query controls. Rows are query
observations, not distinct contexts or financial examples. A fixed ±5-step
input-envelope guard omits unsafe seeds; literal civil-edge required failures
and one-period contexts are hand-test evidence. Fixture explicit spans have
at most two quasi-periods per outer side. Daily Short/Long input labels can be
exact lattices and therefore do not imply an actual residual.

The [hand suite](../tests/schedule_icma.rs) has eight tests: all names/aliases
and mismatches; cap/Zero/Once precedence; Short/Long and 7.5-year equal-role
outer spans; minimal stopping at civil/market limits; both signed required
Date failures with source layering; shared cap before Date arithmetic;
extended TARGET gap/inception/projection; adjustment/closed-calendar
independence and full 3,652,058 one-day references under one shared cap.
The full-range case checks every period's ordinal and day length, original
cap minus one and exact complete fraction against a single rational division.

The separate [property suite](../tests/schedule_icma_properties.rs) uses
independently declared tenor/divisor tuples, civil membership hulls and reduced
integer overlaps throughout the 2000–2399 Gregorian cycle. Its six tests also
cover long formula-derived 337/368 and 1/4 examples, EOM across larger outer
spans, all 49 convention pairs and distinct calendars, TARGET join/override/
inception guards, definition collections and eight-thread rebuilding.

| Gregorian product | Exact observed count |
|---|---:|
| Attempted builder configurations | 614,400 |
| Successful Schedule/context hulls | 594,618 |
| Independently off-lattice middle rejections | 19,782 |
| Determination periods | 1,777,260 |
| Independently reduced rational forward checks | 2,966,496 |
| Forward and bitwise antisymmetric reverse calls | 5,932,992 |
| Canonical zero calls | 2,973,090 |
| Exact full-period 1/F calls | 1,777,260 |
| Total `year_fraction` calls in the product | 10,683,342 |

Its maximum observed error is also **1 ULP**; the enforced allowance remains
four. The final property-source SHA-256 is
`e006c18f6e2248d9383a6290d6bb35d6d287d0b69742e020b1e980f6b465f40b`.
No QuantLib guessed-reference call is used as an oracle. Existing explicit
reference/component fixtures remain unchanged and still run offline.

The cumulative inventory is **1,018,790 rows across 47 fixture files**.
Earlier fixture and generator bytes remain unchanged. New identities:

| Artifact | SHA-256 |
|---|---|
| Schedule source | `5e92f66d8435b21aa4bb31727ef4eb4376a01498c5f625befa4a393bfd5fbf30` |
| ICMA generator | `0f30968d42e3d6f18debb2aae1ea6ef7178229f765e754988c135aef09a7c31e` |
| ICMA fixture | `0df58ae610aac332f0c18deb9cd5ebc69460b2202ee3c489a9bfa8495bf22a55` |
| Frozen civil membership helper | `a082c47a2f0b6c98d312dcc52105395b153e6619d5eff107b3619d2c651448ce` |

## Aggregate, coverage and package evidence

The final Rust 1.99.0/macOS ARM64 aggregate passes **440 debug** and
**439 release** tests in **49 result groups**, with zero failures, ignored tests
or warnings. This includes **341 integration tests**, **26 doctests**
(23 ordinary and three compile-fail), and 73 debug/72 release unit tests. The
single debug-only assertion test explains the aggregate difference. Independent
reviewers separately execute all 15 new focused tests in debug and release.

Formatting, strict all-target Clippy and warning-denied public/private rustdoc
pass. The new generator's byte-exact `--check` passes. Linux GNU and Windows
MSVC all-target checks pass with warnings denied; they are cross-compilation,
not native executions. Aggregate logs are
`/private/tmp/sabiquant-m7-icma-debug.log` and
`/private/tmp/sabiquant-m7-icma-release.log`.

The native LLVM instrumented unit/integration suite passes. Its production
region report at `/private/tmp/sabiquant-m7-icma-coverage` covers **993/1031
schedule regions (96.31%)**, excluding embedded unit-test modules; doctests
are checked separately. This is region coverage, not branch or exhaustive
input coverage. The 38 uncovered regions are allocation failure/propagation,
proof-excluded unused-candidate Date propagation, the impossible missing
private raw vector and structural ICMA constructor failures. Required civil
extension Date errors now execute normally in the public hand tests. No
private corruption or forced OOM is used to turn proof-excluded paths green.
Region mappings changed with the extracted arithmetic helper; percentages
are not numerical-accuracy or before/after behavior measurements.

Independent extracted-package verification passes Cargo's default package
verification build, **73 unit tests**, **26 doctests**, strict all-target Clippy
and warning-denied public/private docs. The archive has **32 exact allowlist
files**, **136,048 compressed bytes** and **593,920 content bytes**, SHA-256
`ed27d22bbbe94900ef7645f7ca13e5c122bb2e95f26a0a85cc8ad151e8c58953`.
All 30 comparable original files match the workspace before and after checks;
only the normalized manifest and generated VCS metadata are exceptions.
The archive records base `d570b4571ec8d55c3e952f739c8ad44242d86735`, dirty=true,
the frozen pre-save ICMA source. Schedule source SHA remains the one above.

Archive path:
`/private/tmp/sabiquant-m7-icma-package-target/package/sabiquant-0.0.0.crate`.
Per-file hashes, allowlist and complete terminal results are retained in
`/private/tmp/sabiquant-m7-icma-package-evidence.json`, SHA-256
`295242858e8755bda35940c1aabab5e51d7f902f91c003b714256e564ecb3fac`.
Integration suites, research docs, fixtures, generators and native reference
tools remain excluded. Cargo's 46 expected omitted-target packaging notices
are separate from the warning-free extracted compiler/lint/docs checks.
Version remains 0.0.0, dependency tables remain absent and publish=false.
Canonical main-branch links establish local file targets only; remote
availability and publication require a separate reviewed release decision.

## Remaining scope and reproduction

The independent review checks implementation/proofs and scoped local behavior;
it does not approve every financial product or waive the existing release
gates. Seven specialized rolls, cash flows/coupons/legs/NPV/accrued interest,
NI-01 exact settlement, formal owner/milestone acceptance, native platform CI,
market-source/maintenance and security-setting review remain separate work.
No private invalid state, system allocation failure or structural impossible
context is forced to inflate production coverage. No push/PR/publication occurs.

```sh
cargo test --offline
cargo test --offline --release
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc --offline --no-deps --document-private-items
python3 validation/generator/schedule_icma_oracles.py --check
python3 validation/coverage.py --output NEW_REPORT_DIRECTORY
cargo package --offline --allow-dirty
```

Use the declared Rust version and fresh coverage output directory. Ordinary
tests need only std and committed fixtures; external oracle tooling is
excluded from the package, which remains version 0.0.0 with publish=false.
