# Local M7 schedule and cash-flow readiness

Current local scope includes anchored and explicit schedules, four civil
rolls, date/EOM and same-roll ICMA references, analytical cash flows and the
modern CDS quoted-maturity resolver/Following date plan. The rule catalog
contains nine constructible labels. Automatic OldCDS remains required for
F-SCHED-01; its historical first-coupon cutoff and generic-start role remain
unresolved. The sections below retain the staged adoption history and then
record the current CDS extension. Full M7, financial/owner acceptance,
native primary-platform execution and NI-01 settlement remain open.

## Authority, dependencies and bounded scope

The owner asked for continuous local development with the broadest meaningful
tests feasible and explicitly selected local-only work. Under that delegated
authority, the agent adopts the concrete core schedule tranche below on
`feat/m7-schedule-core`, following saved M6 commit `5fac9d8`. This is an
explicit local staging exception while formal M5/M6 acceptance remains open;
it does not supply owner API approval, native primary-platform CI or release
permission. No remote write is authorized.

[M7_LOCAL_RESEARCH](M7_LOCAL_RESEARCH.md) records the preceding proposals,
primary-source inspection and full milestone questions.
[M7_REFERENCE_PROBES](M7_REFERENCE_PROBES.md) records 46 bounded, byte-attested
public QuantLib 1.43 observations across all ten rule families. Those
observations are reference behavior, not independent financial definitions.
The initial core decision adopted only Forward, Backward and Zero with explicit
contractual/adjusted boundaries and automatic short/long residual policies.

At core adoption, explicit first/penultimate coupon dates, the seven specialized
Wednesday/twentieth/CDS rules, automatic ICMA references, cash flows, coupons, legs,
NPV and accrued interest were separate work. NI-01 exact settlement is not
resolved by a date-only tranche. All ten rule families and the cash-flow scope
remain necessary for full M7 acceptance; this batch cannot complete M7.

## Adopted value and construction surface

The `schedule` module uses immutable, owned values with explicit calendar,
ordinary and termination adjustment. No implicit date, financial default or
global cache is added. Label-only enums use exact named identifiers and
fallible parsing; context values are not serialized by their labels alone.

| Value | Adopted meaning |
|---|---|
| `DateGenerationRule` | Forward, Backward, Zero. The first two are recurrent directions; Zero uses its distinct constructor |
| `StubPolicy` | None requires an exact regular lattice; Short retains a directional residual; Long merges it with the preceding regular interval |
| `PeriodRegularity` | Regular or Stub on a recurrent contractual lattice; NonPeriodic on Zero |
| `ScheduleCalendar` | Owned Plain calendar or guarded TARGET value. No unchecked TARGET-to-plain conversion |
| `ScheduleBuilder` | Private validated positive date range, owned calendar and explicit ordinary/termination conventions; reusable without mutation |
| `Schedule` | Contractual and adjusted boundary vectors with one-to-one correspondence, interval regularity and retained generation/adjustment metadata |
| `ScheduleError` | Non-exhaustive contextual input/resource/collision errors and typed nested date/calendar/TARGET causes |

`ScheduleBuilder::new(start,end,calendar,convention,termination_convention)`
requires `start < end`. `build_recurrent(tenor,rule,stub,eom,max_boundaries)`
accepts only a positive stored tenor and Forward/Backward. Zero passed to this
method is an incompatible-rule error. `build_zero(max_boundaries)` requires
no artificial zero tenor, stub, EOM option or periodic frequency.

Inspectors expose borrowed contractual/adjusted date and regularity slices,
rule, optional tenor/stub/EOM metadata, calendar and both conventions.
Zero reports absent recurrent metadata and one NonPeriodic interval.
Equality/hashing compare the defined inputs/results and retained context,
including the original stored tenor length/unit. Thus `7D` and `1W` can
generate the same dates while defining unequal schedules; normalized Period
equality alone is not schedule definition identity. This is not financial
equivalence. No `Default` selects financial behavior.
Standard thread traits describe owned values, not arbitrary external clients.

## Contractual recurrence, EOM and residuals

Each candidate comes from the same original directional anchor: start for
Forward, end for Backward. Days and Weeks count calendar days. Months and
Years use anchored clamping, recovering the original day after a short month.
The implementation never chains already-clamped or adjusted dates and never
dispatches recurrence through generic `Calendar::advance`, whose Days count
business days. `7D` and `1W` define the same calendar-day lattice here.

EOM Preserve is accepted only for Months/Years. It activates only when the
original directional anchor is the calendar month end, then places generated
regular candidates at their calendar month ends. The original supplied start
and end remain unchanged contractual endpoints. A business-EOM seed such as
2026-01-30 does not activate preservation. Calendar adjustment occurs later;
Following may move an EOM date into the next month. EOM flags do not imply
an adjusted business-EOM roll.

Backward construction returns strictly chronological boundaries and moves
its residual to the first interval. Forward moves its residual to the last.
None rejects a residual rather than guessing a stub. Short retains the last
regular boundary before the directional endpoint. Long removes that boundary
and combines the residual with its preceding regular interval. It requires
at least one regular interval to merge, so a range shorter than one tenor
cannot satisfy Long. An exact lattice has no artificial stub under Short or
Long. Regularity derives from that raw lattice and never from adjusted lengths.

Checked date arithmetic is used for every required candidate. An unused next
candidate beyond the supported date range stops normally when it must also
cross the already valid endpoint. Unrelated date errors cannot be swallowed.
Required quasi-coupon extensions have their own checked domain under the
subsequent ICMA contracts; core recurrence does not create them.

## Calendar composition, failures and resource bounds

The configured maximum boundary count is explicit and must be in
`2..=3,652,059`. It limits the final returned dates; Long may fit a limit that
Short exceeds by removing the last regular boundary before adding the endpoint.
Unused candidates do not consume returned-boundary slots. Schedule-vector
growth uses fallible reservation and contextual allocation/resource errors;
cloning the existing owned calendar context retains std Clone's allocation
behavior and is not a global out-of-memory recovery guarantee. No partial schedule is
returned. Arithmetic/progress/storage are bounded by the supported date range
and configured count. Structural generation is O(n), owned storage O(n).

The retained TARGET policy validates the entire contractual start/end span,
including Zero, Unadjusted, overrides and joint rules. Verified endpoints do
not establish coverage through the 2003–2025 gap. Any checked span/candidate
failure remains a TARGET cause. A plain calendar retains its defined immutable
classification rules. Neither constructor exposes a guard escape.

Start and interior raw dates use the ordinary convention; only the final
chronological endpoint uses the termination convention. The implementation
keeps one adjusted date per contractual boundary, then rejects every adjacent
adjustment collision or reversal with offending index and both raw/adjusted
contexts. It never sorts, deduplicates or removes a coupon. Calendar searches
retain their 366-holiday bound and pathological-month errors. Calendar query
cost and TARGET span validation are additional to O(n) recurrence, not a
constant-cost promise for arbitrary calendars.

Input precedence is explicit: builder range before build work; build limit,
rule/tenor/EOM compatibility before calendar span/classification; validated
span before generation/adjustment; final strict ordering before a value is
returned. Exact typed variants and source layering are reviewed against the
implemented API and executable tests, not inferred from this prose alone.

## Classified reference distinctions

| Adopted local behavior | QuantLib 1.43 observed distinction | Consequence |
|---|---|---|
| Calendar-EOM-only directional activation | Business-EOM Jan 30 activates Schedule EOM in the probe, even Unadjusted; direct Calendar.advance differs | Explicit local roll contract; no general EOM parity claim |
| All raw dates retained, adjustment collision/reversal rejected | Five-day weekend control becomes a three-boundary Following/Preceding schedule with regular flags | Deliberate contextual rejection rather than dropping boundaries; contractual data remains visible on success |
| Zero has absent tenor metadata and NonPeriodic regularity | Reference Zero reports 0 Years and true `isRegular` | Neither form is a recurring coupon frequency; local metadata makes absence explicit |
| Strict TARGET validates the entire span | The public reference has no SabiQuant verified-coverage policy | Reference agreement cannot waive local inception/gap/maintenance requirements |

Ordinary shared Forward/Backward anchored-date cases will be compared exactly.
At the core snapshot, observations of seven unsupported rules were reference
preparation only. Later implementation needs its own contract and evidence.
No numerical epsilon permits an extra, missing or reordered date.

## Mandatory executable evidence and remaining gates

The core requires independent civil-date recurrence/status fixtures, all
units/directions/stub policies, anchored Jan-31/Jan-30/Feb-29 and Gregorian
cycle month ends, all seven adjustments with independent termination options,
collisions/reversals, strict/projected TARGET spans/overrides/joints, closed
calendar/month searches, limits at ±1 and supported date extremes.
Traits, equality/hash use, immutable clone/thread reuse, catalogs, precise
error context/source and root-wrapper behavior are part of the public contract.
The full M7 research matrix remains applicable to later tranches.

Implementation/test/fixture results and observed limitations must be recorded
after they run. Native Linux/Windows execution, source/maintenance and owner
API/milestone review remain open. This local exception makes a concrete
candidate reviewable while preserving those formal gates; it does not ask
for repeated permission already granted for local work.

## Subsequent explicit coupon extension

The initial core's deferred explicit-date item is now implemented under the
same continuous local authority, with its own
[adopted contract](M7_EXPLICIT_STUB_CONTRACT.md),
[independent review](M7_EXPLICIT_API_REVIEW.md) and
[validation evidence](M7_EXPLICIT_VALIDATION.md). That extension accepts ordered
strictly interior first/penultimate roles, including one shared coupon for
equal roles, and requires an exact named regular middle. At that stage the
seven specialized rules, automatic ICMA references and cash-flow scope were
still separate work.
The earlier core evidence remains a snapshot of `86d5bb8`.

## Subsequent generated ICMA references

The explicit-frequency helper now extends the retained contractual lattice to
its minimal accrual-covering hull under
[the generated-reference contract](M7_ICMA_REFERENCE_CONTRACT.md). Required
civil extensions, shared period caps and the whole extended TARGET span stay
checked. [Local validation](M7_ICMA_VALIDATION.md) records exact civil/rational
oracles and source review separately from the primary financial definition.
No payment adjustment, inferred frequency or universally approved day/week
market roll is introduced. Cash flows and specialized date rules followed in
separate tranches.

## Subsequent analytical cash flows

The [cash-flow contract](M7_CASHFLOW_CONTRACT.md) adopts signed binary64 simple
flows, plain SIMPLE fixed coupons, shared complete accrual contexts and bounded
ordered legs. Flat NPV discounts each eligible payment directly from the supplied
valuation date, with explicit inclusion and contextual work budget; outstanding
accrual clips raw coupon intervals. One-round triple products and exact reduction
of rounded contributions have independent arithmetic/financial oracles. This
continued local analytical work without closing NI-01 exact settlement,
specialized date-rule work, later yield/risk, formal acceptance or native CI gates.
The earlier schedule/reference reports remain saved tranche snapshots.

## Subsequent four civil rolls

The [civil roll contract](M7_ROLL_CONTRACT.md) adopts ThirdWednesday,
ThirdWednesdayInclusive, Twentieth and TwentiethIMM with explicit endpoint
projection, retained requested/resolved ranges and stored-tenor identity.
`DateGenerationRule::Roll` names the selected family; genuine consecutive-roll
regularity is independent of reference generation flags. Whole original/expanded
TARGET guards and the existing bounded adjustment contract remain active.
The [same-roll ICMA extension](M7_ROLL_ICMA_CONTRACT.md) now specifies the minimal
Wednesday/twentieth reference hull over resolved raw endpoints, preserving
explicit frequency, shared caps, required-date causes and full extended TARGET
guards. Its validation evidence is separate from the saved civil-roll snapshot.
The civil-roll snapshot left the three CDS families pending. Modern CDS dates
are now covered by the separate contract below; product/source acceptance,
NI-01 settlement, native primary-platform execution and full M7 acceptance
remain open.

## Subsequent modern CDS date operations

The frozen [modern CDS contract](M7_CDS_CONTRACT.md) adopts two distinct
operations. `resolve_cds_maturity` takes trade date, original quoted tenor
and explicit Quarterly2009/Semiannual2015 convention. It accepts nonnegative
Years or quarter-multiple Months, preserves stored aliases as distinct
definitions, and returns either an unadjusted Date or typed Matured outcome.
Abstract out-of-range anchors are allowed; only a required final date is
constructed, with its exact checked-month displacement retained on failure.

`CdsDatePlan::standard_following` separately takes an already resolved
quarter-twentieth maturity, explicit regime and owned calendar. Its payment
cutoff is trade plus one calendar day; the first Following payment must be
strictly later. It retains raw quarters, adjusted payments and ordinary
accrual boundaries, with included unadjusted final maturity. It rejects
collisions, empty eligible-payment results and required date/calendar failures
without dropping or sorting coupons. Exact coupon caps `1..=39,996`, bounded
fallible storage, and original/precursor/final-adjusted TARGET span guards are
part of the operation.

The [maturity oracle](M7_CDS_MATURITY_ORACLES.md) records 148,688 fixture rows
and 297,376 exact regime comparisons, including every day of a Gregorian
cycle. Separate hand suites cover the primary 2009 payment/accrual examples,
the 2015 maturity tables, cycle-wide candidate enumeration, all weekday masks,
closures, TARGET policies, budgets and precise failure causes. Integrated
results are tracked in [M7_CDS_VALIDATION](M7_CDS_VALIDATION.md); this readiness
update does not pre-assert completion of those aggregate checks.

The five new public values bring the current crate-wide type catalog to 71.
CDS/CDS2015 are constructible date-plan labels; ordinary schedule builders
reject them. There is no implicit conversion to a Schedule, ICMA context or
plain fixed-rate leg. Legal protection, accrued rebates, CDS pricing/repricing
and settlement are not implemented by these date operations. Automatic
OldCDS remains an outstanding M7 obligation, with its source gap documented
in [M7_CDS_LEGACY_SOURCE_REVIEW](M7_CDS_LEGACY_SOURCE_REVIEW.md).
