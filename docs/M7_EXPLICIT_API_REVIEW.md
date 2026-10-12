# Independent M7 explicit-coupon API review

Review opened 2026-10-12 under the owner's recorded continuous local-only
authority. The core schedule batch is saved at `86d5bb8` on
`feat/m7-schedule-core`. This review owns only this document; implementation,
tests, generators and Git state are unchanged. No repeat permission request
is required for the already authorized local work. Native primary-platform
CI, formal owner/API/milestone acceptance, all ten generation families,
ICMA generation and cash-flow scope remain separate gates.

Final focused verification recorded 2026-10-13 (Asia/Bangkok).

## Independent core source review

The full saved [schedule source](../src/schedule.rs), adopted
[core contract](M7_LOCAL_READINESS.md), independent civil generator,
three integration suites and [core validation report](M7_CORE_VALIDATION.md)
were inspected. This completes the previously interrupted independent source
review for the saved core snapshot; it is distinct from the root follow-up
in [the earlier review](M7_LOCAL_API_REVIEW.md).

Reviewed source SHA-256:
`76b3019e16b5d3057784a84291a82e98db89e4c80da36f5d8bbfd034616fd42a`.
The conclusions below apply to that core source, not an uninspected later
explicit-coupon extension.

| Property | Independent source finding |
|---|---|
| Original-anchor recurrence | Every candidate is derived from the original start/end with positive stored tenor and step count. Months/years do not feed a clamped result into the next step; day/week dispatch uses calendar arithmetic |
| Calendar EOM | Activation is exactly `Preserve && anchor.is_end_of_month()`, using the original raw directional endpoint. Generated month/year candidates use their calendar month end; the supplied endpoints remain unchanged |
| Direction and flags | Backward dates and interval flags are reversed together. An initial chronological Stub therefore corresponds to the last generated directional residual, with exactly one flag per adjacent raw pair |
| Long final cap | For an interior candidate, the minimum eventual final count is current count + 1 for Long, versus + 2 otherwise. That lower-bound check permits a later Long merge to fit. An exact endpoint still needs current count + 1 and is not removed to fit a cap |
| Unused unsupported candidate | Day/week/month/year availability checks prove a next candidate is beyond every valid endpoint before invoking date construction. They return absence only for that proof; required arithmetic errors are mapped to the typed Date cause |
| Arithmetic bounds | The count is bounded by 3,652,059; positive `i32` tenor × count and factors 7/12 fit `i64`. The availability formulas remain within supported year/day bounds. Count + 2 fits `u32` |
| Raw/adjusted provenance | `finish` visits every raw boundary, uses ordinary adjustment except at the final chronological endpoint and rejects `value <= previous_adjusted`. Error indexing is guarded by an existing prior value, so the index-minus-one access cannot occur at zero |
| Zero | It constructs two raw boundaries and one NonPeriodic flag, with absent tenor/stub/EOM metadata. It still checks the cap, whole TARGET span and both adjustments |
| TARGET | The owned Target variant uses checked whole-span counting, including endpoints, before generation. Existing coverage, inception, join and override guards remain in the typed Calendar source; adjustment failures beyond the raw span remain visible |
| Value/hash contract | Equality and Hash use the same fields and the original stored tenor length/unit. Normalized-equal periods can intentionally define distinct schedules; the accepted resource cap is not retained contractual provenance |
| Allocation and invariants | Vector growth reserves fallibly before pushes, and count/flag updates remain synchronized. Reservation failure returns no value. Calendar cloning retains ordinary std allocation behavior, as documented |

No concrete core implementation defect was found. The proofs do not establish
market-convention approval or coverage of every calendar definition. Private
invariants and system allocation failure were not fabricated or corrupted to
increase coverage.

Focused independent execution on Rust 1.99.0/macOS ARM64 used a separate
temporary target directory:

```sh
CARGO_TARGET_DIR=/private/tmp/sabiquant-m7-independent-core-review \
  cargo test --offline --test schedule_core --test schedule_reference --test schedule_properties
```

All **16 tests pass**: 12 core, three property and one scoped reference test.
This includes the 18,916-row civil oracle, 65,988 Gregorian-cycle preparations,
all 3,652,059 supported daily boundaries, resource/stub limits, both directional
EOM seeds, adjustment collisions/reversals, TARGET composition and collections.
These are existing tests rerun independently, not additional test counts.
The separate 46 public QuantLib probe rows remain reference-only observations.

## Explicit-coupon contract review status

At the initial snapshot, the explicit contract had not yet been written.
The adopted [explicit coupon-date contract](M7_EXPLICIT_STUB_CONTRACT.md) is
now available and was independently reviewed. No concrete contract defect
was found. The explicit source is now available and was independently
inspected as recorded below. The existing local authority covers this work; no
additional permission request follows from reviewing a new contract.

The contract resolves these questions, and source review checks their exact
implementation:

| Review item | Required distinction or failure case |
|---|---|
| Directional regular anchor | Forward uses first-or-original-start; Backward uses penultimate-or-original-end. The opposite middle endpoint must be exactly on this named lattice |
| EOM activation | Activation follows the named regular anchor. A Jan-31 start with Feb-28 first coupon does not combine two day-of-month lattices: under Ignore, the named February-28 lattice classifies the January-31 outer start as Stub |
| Explicit boundary retention | Preserve supplied first/penultimate dates as contractual boundaries; calendar EOM or business adjustment must not silently relocate their raw values |
| Middle lattice | Exact congruence is required for every nonzero middle; an off-lattice endpoint yields IrregularExplicitMiddle. Equal supplied roles have a zero-length middle |
| Boundary regularity | Outer predicates use adjacent signed steps of the same anchor, not one-step arithmetic from a clamped interior date. This preserves a defined provenance even when another roll convention could call the interval regular |
| Optional-date domains | None/None is rejected; both supplied roles must be ordered, with equality deliberately allowed. Build checks each supplied date strictly inside the original range |
| Automatic policy interaction | There is no automatic policy argument or merge. Supplied boundaries remain present and stub_policy() is None rather than a fabricated automatic plan |
| Final cap | Count original endpoints, explicit boundaries and middle candidates exactly once. A valid outer cap of two must report insufficient final boundaries when an interior coupon is required, not leak an invalid reduced inner cap error |
| Unsupported unused classification candidate | A hypothetical predecessor before Date::MIN or successor after Date::MAX may prove an outer interval is a stub without becoming a required quasi-coupon reference date. Do not confuse this with future required ICMA extensions |
| Context checks | The entire original TARGET span must remain guarded; checked adjustment of explicit boundaries must preserve source errors and collision/reversal context |
| Metadata and identity | Retain explicit-date choices separately from automatic plans and include them consistently in definition equality/hash. No frequency or ICMA context is inferred from coupon dates |
| Backward ordering | Reverse dates and flags with compatible meaning; ordinary/terminal adjustment still follows final chronological order rather than generation order |

The frozen public probes provide five explicit-coupon examples with safe
quarterly dates. They establish observed reference vectors and flags for
those cases only, not all possible first/penultimate rules or the extension's
financial contract. An exact-date oracle must derive expectations from
the adopted anchored definition rather than reference implementation source.

The contract deliberately refines the earlier research proposal about equal
roles: one interior date can be both first and penultimate, producing three
final boundaries. This is input role reuse, not post-adjustment collision
repair. Reversed roles remain rejected. Additional bounded reference
observations are recorded in a separate 29-row fixture with their own
provenance; they are not part of the original frozen 46-row set.

Practical financial meaning is explicitly limited: Regular means one interval
of this named civil lattice. Stub does not assert that every market convention
would regard the interval as irregular. The Jan-31/Feb-28 example makes that
qualification visible, and exposing `lattice_anchor()` gives future ICMA
generation enough context to respect it. The date-only extension still does
not supply contractual frequency, denominators or payment/accrual conventions.

## Explicit source proof obligations

Let `n` be the number of middle boundaries, and let `e` be the number of
supplied roles: one or two. The final boundary count is `n + e`, because
the absent-role endpoint already belongs to the middle while a supplied role
adds its original outer endpoint. Equal roles have `n=1, e=2`, so the shared
coupon appears once and the original outer endpoints make three boundaries.
Middle flags number `n-1` and outer flags number `e`; the total remains one
less than the final boundary count.

With `c` middle boundaries currently stored, a new strictly interior middle
candidate requires at least `c+2+e` final boundaries: the candidate and the
still-distinct opposite middle endpoint. An exact endpoint requires
`c+1+e`. These checks must use the caller's unchanged cap. A valid cap of two
cannot become an InvalidBoundaryLimit error from a derived inner cap; it
must fail with the appropriate proved final lower bound. Separate the
anchor-relative step counter from final/outer storage counts, especially if
original endpoints are inserted before generating the middle.

For exact middle step count `k`, Forward classifies the first outer interval
at step -1 and the final outer interval at step `k+1`. Backward uses step +1
for the final interval and step `-(k+1)` for the first. The equal-role case
uses `k=0`. Intermediate or outer steps never start from an already-clamped
date. The original core availability checks can prove an unsupported
classification-only quasi-date cannot equal the in-range outer endpoint;
that produces Stub without swallowing a required-date error.

Classification-only quasi-dates use the civil lattice and do not create
additional calendar queries or required reference coverage. The whole
original TARGET span and every actual adjustment remain checked. A future
ICMA helper that requires those extra boundaries must separately validate
their date range and market coverage.

## Explicit implementation findings

Reviewed explicit source snapshot SHA-256:
`be91b84dcaecd03b9ab7c667a742c3dffc5d27b5a04961a2badc49017e54ba06`.
The adopted contract snapshot SHA-256 is
`7dd48795d0d9667b5c8f652ea68806abe0777d2355cd8771e131d4f69c714abd`.
These identify the inspected source/contract, not a release or a promise
that subsequent edits are automatically reviewed.

| Implementation check | Independent result |
|---|---|
| Constructor domains | Private role fields, no Default, presence check before ordering, equal roles allowed and reversed roles rejected. Build checks first then penultimate strictly interior after option validation |
| Named anchor | Forward chooses first-or-start; Backward chooses penultimate-or-end. EOM reads that anchor's calendar month end; supplied coupons/endpoints are not rewritten |
| Step/count separation | `step` starts at zero independently of `rolling.count`, which can already contain a leading original endpoint. This prevents skipping the first middle recurrence step |
| Initial cap proof | `extra + (anchor==endpoint ? 1 : 2)` accounts for both supplied outer roles and the minimum middle. Equal roles require three final boundaries, with the coupon stored once |
| Interior cap proof | With the leading outer already stored, `rolling.count + 2 + trailing` reserves the new strict-interior candidate, opposite middle endpoint and any trailing original endpoint. The caller's original cap is retained in every error |
| Exact endpoint push | The initial minimum or immediately preceding interior check proves space for the uncapped exact-endpoint push and trailing endpoint. No public reduced-cap constructor is invoked |
| Middle mismatch | An overshoot or support-range absence yields IrregularExplicitMiddle with direction, tenor, anchor, endpoint and last regular boundary; no implicit residual is appended |
| Outer classification | Leading comparison uses step one in the opposite direction; trailing comparison uses `step+1` in the middle direction. Both operate on the unchanged anchor; equal roles use step zero for the middle |
| Final chronology | Dates and flags reverse together for Backward, then the existing chronological adjustment/collision machinery is reused unchanged |
| Context validation | Whole original TARGET span validation still precedes generation; classification-only quasi-dates cause no additional calendar query. Every actual boundary adjustment retains its source chain |
| Metadata and collections | Explicit roles are retained, automatic stub policy is absent, and equality/Hash include the same explicit-date field. `lattice_anchor()` derives the declared seed from retained roles and raw endpoints; Zero returns None |
| Error API | Four new stable kinds preserve empty roles, reversed dates, offending outside role/range and off-lattice middle context. They remain inside the existing schedule/root error model |

No arithmetic, cap, ordering, guard or retained-metadata defect was found in
this snapshot. Two concrete documentation defects were reported immediately:
the module overview still called explicit coupons deferred and described only
endpoint anchors, while the Stub variant described only automatic residuals.
The root corrected both without changing arithmetic. Those corrections are
present in the source hash above.

The existing three suites were rerun against the new API: all 16 then-compiled
tests passed, including the reference consumer's five newly enabled explicit
quarterly cases. A second reference test was added after that compilation.
It was subsequently run independently with `cargo test --offline --test
schedule_reference`: **both current reference tests pass**. The 29 additional
explicit public observations are separately classified by that consumer;
they are not retroactively part of the original frozen 46-row probe set.
This execution adds no test counts beyond the tests actually present.

The independent [explicit civil generator](../validation/generator/schedule_explicit_oracles.py)
and [Rust consumer](../tests/schedule_explicit_oracles.rs) are now available
and were independently inspected and executed. The consumer passes all
**21,883 rows**: 17,789 values and 4,094 exact IrregularExplicitMiddle contexts.
Successful rows check every raw/adjusted date, all flags, retained metadata,
anchor, original-cap failure one smaller and equality with a larger cap.
The observed totals are 70,819 boundaries, 35,818 Regular flags and
17,212 Stub flags. Generator `--check` reproduces the fixture exactly.

The oracle selects the entire bounded civil middle using exact anchor-coordinate
membership, and classifies intervals by adjacent coordinates. It does not
reproduce the source's recurrence counter or early cap path. Input construction
uses civil shifts to choose examples; the separately enumerated expected
middle also includes perturbed endpoints and off-lattice error context. Its
scope is positive tenors, valid ordered/equal interior roles, NullCalendar and
Unadjusted conventions. Input-domain/calendar failures require the separate
behavioral suite rather than being inferred from these rows.

The dedicated [explicit API/error/property suite](../tests/schedule_explicit.rs)
is now frozen and independently executed. Its final test-source SHA-256 is
`e5db0bda9220766831c16bce9cc9d59f1ab4e371cc8c5edeb6c431c8093aef7b`.
All **13 tests pass in debug and release** against the unchanged reviewed
source. The exact Gregorian-cycle product is **230,400 preparations**:
227,709 successful schedules, 2,691 irregular-middle errors and 682,230 checked
interval flags, with cap-minus-one checks on every success. This covers all
four units, both directions, four role combinations and compatible EOM
policies across 400 years of month-end seeds. Other tests cover role/domain
precedence, equal roles and huge tenors, clamp provenance, classification-only
range edges, all adjustment pairs, whole-span/candidate TARGET guards,
typed source chains, definition collections and concurrent rebuilding.

The explicit oracle and both reference tests also pass independently in
release, so that final release invocation passes **16 focused tests**.
Debug executions cover the same three final suites separately. The earlier
dedicated snapshot checked lower bounds on cycle totals; it was rerun after
the author replaced those with exact final count assertions. No other source
or test change is included implicitly in these claims.

Focused reproduction uses the declared Rust 1.99.0 toolchain:

```sh
cargo test --offline --test schedule_explicit --test schedule_explicit_oracles --test schedule_reference
cargo test --offline --release --test schedule_explicit --test schedule_explicit_oracles --test schedule_reference
python3 validation/generator/schedule_explicit_oracles.py --check
```

This is a completed independent contract/source review with scoped native
macOS evidence. It does not claim independent execution of every project
test, final package/coverage audit, Linux/Windows native CI, owner API approval
or full M7 acceptance.

## Remaining scope

Explicit coupon dates alone do not add Wednesday/twentieth/CDS market rolls,
ICMA quasi-periods, coupons, legs, NPV, accrued interest or exact settlement.
Review findings will distinguish a concrete source/contract defect from a
deliberate named reference divergence or a remaining full-milestone gate.
