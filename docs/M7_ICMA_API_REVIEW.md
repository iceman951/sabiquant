# Independent review of the local M7 generated ICMA API

Review snapshot: 2026-10-13, macOS arm64, Rust 1.99.0. This review covers
`Schedule::build_icma_context` and the shared checked anchor arithmetic at the
source hashes below. It follows the saved explicit-schedule base `d570b45` and
the adopted [generated ICMA contract](M7_ICMA_REFERENCE_CONTRACT.md). No
production, test, generator or Git changes were made by this reviewer.

No concrete arithmetic, resource-bound or public-invariant defect remains in
the reviewed snapshot. The implementation follows the adopted local contract.
This is a scoped implementation review, not formal M7, owner, financial-product
or native multi-platform acceptance.

## API and domain checks

The API returns an owned `ActualActualIcma` without consuming its source
Schedule. It requires a caller-supplied named periodic frequency; date lengths,
stub flags, adjusted dates and explicit roles do not infer that frequency.
Stored tenor comparison uses Period's normalized equality: 12M/Annual,
7D/Weekly, 14D/Biweekly and 28D/EveryFourthWeek match. A coincidentally equal
observed day length cannot equate day/week and month/year families. Generation
still uses the retained original unit, including that unit's checked Date
operation and signed count in any failure.

The ordered input checks are cap validity, periodic schedule, periodic
frequency, then normalized tenor/frequency match. Zero reports
`schedule.non_periodic_icma`, including when Once was also supplied; a periodic
schedule with Once reports `schedule.invalid_icma_frequency`. Missing periodic
metadata cannot be repaired by guessing. Current private constructors establish
the remaining metadata and nonempty chronological contractual slices, so the
defensive empty-slice path is not a demonstrated public failure.

The retained directional lattice remains the domain decision: Forward uses
explicit first or raw start; Backward uses explicit penultimate or raw end.
EOM activation is fixed from that original anchor's **calendar** month-end
status. Extending from a clamped neighbor, the other explicit role, a business
month-end or an adjusted payment date would change this contract. The reviewed
source does none of those things. Both absent Long coupon dates and all
multi-period outer spans are restored as reference intervals. Equal explicit
roles provide an interior anchor; they do not restrict the context to two
determination periods.

The existing named daily/weekly event counts remain explicit 365/52/26/13
divisors. Generated leap-year daily references do not change 365. These are
local analytical conventions, with the financial limits already stated in the
[primary-source research](M7_ICMA_REFERENCE_RESEARCH.md). This review does not
extend that evidence into approval of every market roll or adjusted accrual.

## Source proof checklist

| Obligation | Independent source reasoning and result |
|---|---|
| Minimal hull | The validated anchor is inside the raw increasing range. Fixed-anchor positive tenors produce strictly monotone signed lattice dates. The negative loop stops at the first date no later than raw start; the positive loop stops at the first date no earlier than raw end. These are respectively the greatest lower and least upper bracketing dates. Every intervening lattice date is retained. |
| Exact endpoints | The loop conditions stop on equality. Neither direction computes an unnecessary next date or expands a market coverage query beyond the exact hull. |
| Positive contiguous result | Start is strictly before end, so at least one interval is generated. Reverse only the negative sequence, retain the anchor once, then append the positive sequence. Thus at least two boundaries exist and adjacent windows are ordered, positive and contiguous. `boundaries.len() - 1` is safe under private construction. |
| Shared cap | Both directions update one count of final reference periods. A negative step reserves one additional upper interval whenever anchor is before raw end: `count + 1 + remaining_upper`. An interior anchor with cap one therefore proves two required periods before required Date arithmetic. The upper loop uses `count + 1`, sharing all earlier negative periods. The original caller cap is preserved. |
| Cap stopping | Each count check precedes computing the disallowed required date. A valid final N-period hull passes cap N and larger caps; for N ≥ 2, cap N−1 fails with the proved next lower bound N. Cap zero instead fails input validation. A smaller valid cap may stop earlier and correctly reports a lower bound rather than claiming the undiscovered full count. |
| Wide arithmetic | Successful increments cannot exceed the hard cap. Even the generic widest positive i32 tenor multiplied by the bounded u32 step, and by the week/year conversion factors seven/twelve, fits i64. Count additions fit u32. Matching named frequencies impose tighter actual tenor bounds. |
| Required versus unused dates | ICMA calls `checked_anchored_date` directly. It does not use `anchored_candidate`'s proved unused-candidate `Ok(None)` path. A required reference outside the civil range preserves the actual DateError; it cannot silently shorten the hull. The existing schedule generator still uses the bounded unused-candidate path before the shared helper. |
| Fallible allocation and work | Boundary growth uses `try_reserve(1)` only when capacity is exhausted, permitting amortized growth rather than exact one-element reallocation on every step. The period vector reserves the exact known final size fallibly. Each required date is visited once, negative reversal is once, and construction is O(p) time/storage plus the retained calendar's declared full-span cost. No partial value is returned. |
| Constructor layering | Adjacent windows are passed through `DeterminationPeriod::new` and the entire vector through `ActualActualIcma::new`. Their typed `IcmaError` is preserved in `ScheduleError::Icma`, rather than unwrapped. Failure of these structural checks is proof-excluded for current valid generation; catalog entries do not manufacture a reachable public failure. |
| Retained policy | The entire finite hull, including both quasi endpoints and dates absent from coupons, is checked through the Schedule's owned TARGET definition before a context is returned. Projection, overrides, guarded joints and coverage gaps retain their original policy. No replacement calendar or fresh coverage grant is introduced. |
| Value semantics | The returned context stores periods/frequency and their coverage, not adjustment or calendar metadata. Different Schedule definitions can yield equal contexts. This is existing context definition equality, not financial-product equivalence. Schedule provenance remains inspectable on the original value; neither value gains a mutable cache. |
| Numerical policy | Generation uses exact civil dates and integer counts. Accrual uses the unchanged existing ICMA overlap formula and compensated sum. No floating-point tolerance or frequency inference was introduced. |

For required-date errors, `Date.index` is the boundary-generation position
before final chronological ordering, not a signed lattice index. Before a
negative failure it counts the anchor and retained preceding negative steps;
after reversal an upper failure position also includes the already retained
negative boundaries. The typed Date cause separately retains the original
anchor, original unit operation and actual signed scaled offset. This agrees
with the adopted contract and FR-SCHED-15. The required phase is
`ICMA reference extension`.

Full-hull coverage errors have index None and phase `ICMA reference coverage`.
The root Error retains ScheduleError as its first source; Date, Calendar and
ICMA causes remain the next typed layer. Reviewed display/kind/source branches
include all six new schedule variants. Allocation failure and proof-excluded
structural wrappers are catalog checks, not forced OOM/private-corruption
tests. FR-SCHED-14 through FR-SCHED-17 state the same scoped obligations in
[REQUIREMENTS](REQUIREMENTS.md).

An early overview observation concerned wording that still deferred generated
ICMA. Root had already updated that wording by the current source snapshot;
the overview now describes matching frequency and retained contractual lattice
and defers specialized rolls/cash flows. No arithmetic change resulted from
that observation.

## Independently executed evidence

All commands below were run offline using Rust 1.99.0 on this native macOS
arm64 host, with separate temporary Cargo target directories. The focused
debug and release runs each passed **15 tests**: eight hand cases, one fixture
consumer and six independent property tests. These are this reviewer's actual
runs; whole-project results, doc tests and CI are separate evidence.

```text
cargo test --offline --test schedule_icma
cargo test --offline --test schedule_icma_oracles --test schedule_icma_properties -- --nocapture
cargo test --offline --release --test schedule_icma --test schedule_icma_oracles --test schedule_icma_properties -- --nocapture
/private/tmp/sabiquant-validation-venv/bin/python validation/generator/schedule_icma_oracles.py --check
```

After the property oracle's frequency tuples were made independent of the
production mapping, its final hashed six-test suite was separately rerun in
both debug and release and passed. Schedule source, hand tests, fixture
consumer, generator and fixture stayed at the listed hashes.

The final command reported `generated ICMA oracle reproducible`: exact output
and recorded generator/helper provenance matched. Python fixture expectations
use bounded civil-envelope membership and exact Fraction overlaps; they do not
read Rust results or QuantLib outputs. The Rust consumer independently
recomputes each reduced rational with checked i128 pre-reduced products before
comparing the existing binary64 fraction.

| Evidence | Scope and actual result |
|---|---|
| Hand cases | Eight tests passed in debug/release: all ten named periodic frequencies and aliases/mismatches, input precedence, Short/Long and multi-period equal roles, exact civil/market endpoints, signed MIN/MAX required failures, shared-cap-before-Date failure, retained strict/projected TARGET, adjustment independence, and the full 3,652,058 one-day-reference hull. The 7.5-year equal-role case checks nine annual references and caps one through eight. |
| Civil/Fraction fixture | 36,092 query rows from 18,046 contexts; exact minimal references, anchor, explicit event metadata, exact cap, cap−1 where valid and larger-cap equality. 147,388 period comparisons, 4,511 equal-date queries and 4,512 reversed queries. Measured maximum **1 ULP** in both runs against the existing four-ULP profile. |
| Gregorian membership property | 614,400 attempted builds over 2000..2399: 594,618 successes and 19,782 correctly rejected irregular middles. Exact checks cover 1,777,260 reference periods and 2,966,496 forward overlap fractions across all ten periodic frequencies, directions, explicit role shapes and applicable EOM policies. Reverse fractions, equal-date +0, each full period and cap−1/exact/larger-cap checks also pass. The suite asserts the four-ULP bound; this review did not independently log its finer maximum. |
| Oracle independence | The property suite enumerates civil-day congruence membership using its own Gregorian ordinal and month lengths. Its ten tenor/divisor tuples are declared independently and checked against Frequency's production mapping once before driving the scan. It neither calls production Date shifts for expected references nor shares compensated summation. Checked reduced rational numerator and denominator are explicitly bounded by 2^53, making both binary64 division inputs exact. |
| Policy/value properties | Active/inactive original-anchor EOM across long outer spans; 49 ordinary/termination convention products plus two other calendar definitions produce 51 distinct Schedules and one reference context; TARGET overrides/null joints/gap/inception/projected policy; typed root causes; clone/Eq/Hash/divisor distinction; eight threads × sixteen rebuilds × ten periodic frequencies. All six property tests passed in both builds. |

The fixture's fixed ±5 civil envelope excludes inputs with unsafe envelope
neighbors, so it is not the required-civil-error oracle. Its explicit inputs
use at most two quasi-periods per outer side, and 998 second query rows repeat
their context's first query. Those rows are not additional independent input
contexts or unique queries. The hand suite supplies required MIN/MAX failures,
exact stopping and a larger nine-period outer example; the separate 400-year
membership scan supplies a broader civil residue check. Automatic day/week
Short/Long inputs can be exact because one-day endpoint shifts may remain on
their lattice; their labels alone do not establish irregular-stub coverage.

## Reproducible reviewed snapshot

| File | SHA-256 |
|---|---|
| [schedule source](../src/schedule.rs) | `5e92f66d8435b21aa4bb31727ef4eb4376a01498c5f625befa4a393bfd5fbf30` |
| [existing ICMA source](../src/daycount/icma.rs) | `b2a14424548798368782ab58f84a774bae1fa29f40e51a5c186034ac5eae6d08` |
| [adopted contract](M7_ICMA_REFERENCE_CONTRACT.md) | `abdc7dd61de1683399fbfc57403def012b251f72dd7ae734e6e322cc0eb25a74` |
| [hand tests](../tests/schedule_icma.rs) | `7ddc8d6574261b935f1fcee5f67a5522534d2b0dc6664149e7931224ed538eee` |
| [fixture consumer](../tests/schedule_icma_oracles.rs) | `bb31db8e9d6b1ffb3a8b6af8f0a26242c814a686e55b3237b318f92ff3358b77` |
| [civil property tests](../tests/schedule_icma_properties.rs) | `e006c18f6e2248d9383a6290d6bb35d6d287d0b69742e020b1e980f6b465f40b` |
| [generator](../validation/generator/schedule_icma_oracles.py) | `0f30968d42e3d6f18debb2aae1ea6ef7178229f765e754988c135aef09a7c31e` |
| [fixture](../validation/fixtures/schedule/icma_oracles.fixture) | `0df58ae610aac332f0c18deb9cd5ebc69460b2202ee3c489a9bfa8495bf22a55` |

Source/test changes after these hashes need their own affected review and
verification. This snapshot establishes scoped native execution; it does not
establish Linux/Windows or other native toolchain results, reference
compatibility, adjusted accrual semantics, specialized ten-rule completion,
cash-flow/coupon/leg/NPV/accrued-interest work, maintenance/release approval or
formal owner/financial acceptance. Those remain the separate recorded M7 gates.
