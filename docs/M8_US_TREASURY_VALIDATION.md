# Fixed-principal Treasury local implementation validation

Verified local engineering checkpoint, 2026-10-21 on `feat/m8-us-treasury` after
pre-production checkpoint `ecd184f`. The complete adopted
[main](M8_US_TREASURY_CONTRACT.md) and [cash](M8_US_TREASURY_CASH_CONTRACT.md)
contracts are unchanged. The engineering scope is complete price, bounded
yield inversion, all six risk views and all four exact cash operations.
The complete implementation and final whole-tree verification below pass. All work remains local; no push, PR, publication or release occurred.

## Implementation and independent inputs

Fourteen new public types bring the catalog to 126. The complete public
declaration set equals the 126 external thread-safe value trait assertions.
Twenty-four Treasury and five yield error variants bring the domain error
catalog to 199, including stable kinds, typed nested sources, root conversions
and formatting failure behavior. Instruments own the supplied nominal cycle;
plans borrow the instrument and reserve one prepared raw bundle vector.
Reports own scalar provenance, both raw/published AI observations and policy.
Copy/Eq/Hash cash instructions retain complete compact source/evidence context.

The [pre-production gate](M8_US_TREASURY_PREPRODUCTION_VALIDATION.md) accepted
and froze all four independent corpora before any Treasury production:

| Corpus | Rows | Production connection |
|---|---:|---|
| Exact source/date/cash |30,572| Public constructor/plan/selectors/views/cash/inspectors and typed errors |
| Prescribed numerical primitives |5,593| Actual private helpers in an attested exact-copy temporary crate |
| Ideal price and all six risk views |714| Public API values and independently classified per-quantity bounds |
| Target-driven ideal inverse |145| Public bounded inversion, independent conditioning and cached reports |

These 37,024 records are reference inputs, not Cargo test-function counts.
The source/cash consumer has three tests, including helper/census controls;
ideal price/risk and inverse use two. Twenty-eight numerical and 38 source/cash
hand tests were authored and frozen independently before production. The
active numerical copy adapts private Interval constructors to public bound
inspectors and adds test-only lint/comment adjustments. Financial inputs,
expected bits and domain endpoints are unchanged; the source/cash copy is
byte-identical to its frozen original.

The source/cash public consumer binds 23 outputs directly or through formatted
inspectors and five rational outputs through public quantized/binary views.
It does not expose invented private rational inspectors. Out-of-range or
inexact public projections can fail to distinguish neighboring private
rationals; exact expected-corpus reconstruction is separate evidence.
The [cash implementation review](M8_US_TREASURY_CASH_IMPLEMENTATION_REVIEW.md)
records this scope and the complete actual temporary-copy census.

All 5,593 numerical rows pass both debug and release through the actual
prescribed private factor/PV/AI/term/reduction/residual helpers in a create-new
exact-copy crate, with strict copied-crate Clippy and original/copy/fixture/
runner guards. The adapter also checks the retained sums before a final
dirty overflow. Proof-only metadata such as ideal containment/tree weights
and selected generator intermediate-width counters is distinguished from
actual uninstrumented runtime outputs. No test hook or fixture-dependent
unit module is shipped in the real library.

## Current execution evidence

The initial focused executions passed 125 unit tests, all 66 hand tests,
the two 714/145-row ideal/inverse consumer tests and all three 30,572-row
source/cash consumer tests. The maximum-date ideal grid executes 19,997
actual payments; its initial debug consumer took 192.68s, source/cash 94.09s.
An actual native stack sample showed productive p256/checked-wide division
work, rather than a stalled reference helper. Those initial runs preceded the last source storage/form corrections;
the guarded v4 whole-tree runs establish the current bytes. A documented Treasury construction/pricing example also passes
as a real doctest.

| Evidence | Current result |
|---|---|
| Native whole-tree debug/release and strict checks | v4 PASS:914 debug /913 release, strict all-target Clippy, fmt, warning-denied public/private docs and diff check |
| Production-region coverage and exclusions | v4 PASS:19,801/21,088 regions (93.897003%),871 unit/integration functions in101 binaries; all493 missing Treasury coordinates classified |
| Extracted std-only package | PASS: actual archive, extraction, 54 payload byte comparisons, 125 units, 43 doctests, 8 external API tests and strict checks |
| Foreign Linux GNU/Windows MSVC | PASS compilation only on v4; no foreign runtime claimed |
| Semantic mutation sensitivity | PASS: three private numerical faults and 14 compiled semantic faults killed by runtime assertions; primitive-superset qualification retained |

Seven postproduction public test functions supplement the frozen hands:
four immutable policy/Debug tests, two solver residual boundary tests and
one two-state all-six-risk midpoint test. Debug uses valid raw-overflow and
raw-underflow reports, checks compact output and propagates every insufficient
formatter byte budget. Solver controls exercise an actual negative cached
amount residual and a ties-even residual carry with two counted callbacks.
The ideal consumer additionally binds14 A–G source-display observations to
six-place **clean P**; this corrects the frozen test-design description's
"dirty displays" label without changing expectations or adding kernel display
rounding. Current exact scope is mapped across all76 families in the
[semantic evidence ledger](M8_US_TREASURY_SEMANTIC_COVERAGE.md).

The earlier gilt827/826 native regression, coverage and package reports remain
historical. They are not substituted for these current Treasury checks.

The debug total is125 unit,746 integration and43 doctests; release has124
unit tests with the same746 integration and43 doctests. Instrumented coverage
excludes doctests and embedded test code from its production-region counts.
Treasury itself covers3,397/3,890 regions (87.33%):795/862 parent,
1,043/1,152 cash and1,559/1,876 numerical. The
[coordinate-level coverage review](M8_US_TREASURY_COVERAGE_REVIEW.md) classifies
all493 remaining coordinates:431 bounded/private-invariant defences,
38 caller-domain restrictions,13 unforced allocation/propagation paths and
11 valid primitive paths checked separately in the exact-copy adapter.
None is removed from the denominator. No adjusted, branch or exhaustive-input
percentage is claimed, and private-copy executions are not merged into the
native percentage.

The actual package contains56 archive members and54 byte-compared original
payload files. Against the prior gilt package, its additions are three
Treasury production modules and the test-only rational support file. It
contains no research fixtures, generators, AGENTS or private handoff state.
The extracted crate passes125 units,43 doctests, strict Clippy and both docs
builds; the unchanged external API suite passes8 tests against only its
extracted rlib. Metadata remains dependencies=[] and features={}.
The final package rerun includes the README status update after all
repository integration supplements. Its54 input guards and byte comparisons
pass; this archive is an actual local package test,
not a publish or release.

| Final evidence | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-treasury-native-verified-v4/evidence.json` | `67aeec2a3c07a0b9e989647fc3989fb5732f830e47d58b305ff26a2c027d5214` |
| `/private/tmp/sabiquant-treasury-coverage-v4-evidence/evidence.json` | `9f894ad27d35dca5189d30f3f9500c5a0f178cac44ba7c78e06a1b4aeb28aeee` |
| `/private/tmp/sabiquant-treasury-numeric-kernel-v7/evidence.json` | `091f9521233e6eb06df90ab94c86542a604077cdf9f89ed73047bf267bb57fc3` |
| `/private/tmp/sabiquant-treasury-package-v2/evidence.json` | `be5e0a1d75ae4e22c1f36d281056a861faed33b6960a896787b57d6217831cfc` |
| `/private/tmp/sabiquant-treasury-semantic-mutants-v3/evidence.json` | `4468082aa8df792dc792ed66d942af4b018c01eec260e82d3e703beec019f6f0` |
| `/private/tmp/sabiquant-bond-yield-extended-reproduction-v1/evidence.json` | `c1845758883485cddc75d309ebb5a506da5e08b26c8a3df200c6931194289ef5` |

The native and coverage runners guard all original source/tests, fixture
files, Cargo/Clippy configuration and every literal Rust include input,
along with their own runner bytes. Before/after guards pass; these establish
file equality at those checks rather than continuous filesystem monitoring.
The private adapter additionally guards its frozen inputs and the exact
allowed test-only appended deltas. It executes all5,593 rows plus three
synthetic shared-Brent transport and two positive primitive-superset controls
in both profiles. These supplemental checks are separate from standard test
function counts and do not invent financial reachability.

## Preserving earlier generic bond evidence

The first full native run failed an old generic oracle's source-document guard:
Treasury adoption had appended to the living numerical policy, whereas the
unchanged extended yield/risk generators and fixtures pin the earlier whole
file. The financial expectations and old production arithmetic did not change.
The exact pre-adoption policy is now retained as
[the historical reference](NUMERICAL_CORRECTNESS_YIELD_RISK_REFERENCE.md),
SHA-256 `360bc4cfe69d487cc7a94ba694ee791145530053eff7ee014a54db24a420c061`.
Only the two ordinary Rust consumers' include paths changed; all old expected
hashes and fixture bytes remain unchanged. All three focused release tests pass.

The additive
[historical reproducer](../validation/generator/bond_yield_extended_reproduction.py)
stages that snapshot at the original policy path in a new external 15-file
minimal tree and invokes both unchanged old generators with `--check`.
It reproduces all 23,374 rows in three frozen fixtures, with original,
living/historical policy, runner and staged-input guards passing. It does not
claim that the old direct commands reproduce metadata against today's living
policy. The current policy remains normative for current development.
The first failed native and partial coverage runs are retained; they are not
relabeled successful. Coverage stopped before producing a production report.

## Numerical/resource corrections and acceptance limits

Root corrected three implementation resource/form issues before final
verification: AI's generic IEEE conversion now widens temporary operands to
nine words while retaining five-word permanent caches; raw AI creation and
complete risk coefficient summands have explicit lexical end scopes; exact
accrued publication uses the adopted fused decimal-power form, avoiding an
uncancelled factor two beyond its 162-bit numerator bound. Numeric workspace
reservation is exact. These changes preserve the frozen numerical expectations.
The logical-copy budget is distinct from compiler stack frames or a caller's
stack limit. The complete cash review records 16 Scalar/14 Nine source work and
cash within 16 Price/12 Source slots, counting nested checked arithmetic copies.

During integration, the ideal consumer initially compared its raw A_j
encoding to selected published source-coupon views. Root corrected the
consumer to use raw coupon views and kept every frozen reference unchanged.
Earlier failed compilation/diagnostic logs remain retained. A concurrent
intended source edit caused a prior adapter whole-source guard to fail after
copied tests passed; the final unchanged-source rerun is the authoritative
evidence. Such guard failures are not reported as financial test failures
or silently overwritten successes.

Public outcomes remain supplied-observation results with conditional ideal
accuracy. The source identifier records the retained local formula snapshot;
it does not establish current law, auction terms, ownership, payment-calendar
eligibility or actual settlement enclosures. Uninformative ideal quantities,
unconditioned inverse coordinates and the excluded ideal-image control retain
their original qualifications. The complete semantic-family map, final
native/coverage checks and exclusion review support this local engineering
checkpoint. The
[numerical implementation review](M8_US_TREASURY_NUMERIC_IMPLEMENTATION_REVIEW.md)
records the independent arithmetic/resource audit and semantic mutation scope.
Owner financial,
calendar/announcement, Canadian models, ideal enclosures, native foreign
runtime, release and full M8 acceptance remain open.
