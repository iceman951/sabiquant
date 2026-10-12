# Local M7 core schedule API and implementation review

Review opened 2026-10-12 after the continuous local work instruction and
explicit local adoption of an M7 core tranche. M6 is saved at `5fac9d8`.
This independent review proceeds under that recorded local authority; it
does not request repeat permission or treat the earlier research snapshot
as overriding later authorization. Formal dependency/native-platform review,
full ten-rule schedules, cash flows and milestone acceptance remain separate
open gates. This review owns no implementation or test edits.

## Review snapshot

The first pass reviews the five adopted distinctions communicated for the
core tranche against [M7 research](M7_LOCAL_RESEARCH.md), the adopted
[API principles](API_DESIGN_PRINCIPLES.md), existing date/calendar contracts
and [46 frozen public reference probes](M7_REFERENCE_PROBES.md). At this first
snapshot, the local readiness document and schedule source had not yet been
written. The adopted [local readiness contract](M7_LOCAL_READINESS.md) is now
available and was reviewed in the second design pass. Source-level conclusions and measured validation will be added
after the implementation is available; a design pass is not an implementation
pass or milestone acceptance.

## Adopted distinctions and reference boundaries

The labels below identify review topics, not stable error kinds or newly
adopted discrepancy identifiers. The forthcoming local contract controls the
exact public names and error semantics.

| Review topic | Adopted local core behavior | Exact reference observation / limit | Required source property |
|---|---|---|---|
| Calendar-EOM directional activation | EOM preservation activates only from a calendar-EOM directional seed; business EOM alone does not activate it | `EOM-FORWARD-2026-01-30-PRESERVE-UNADJUSTED` reports March 31, although Jan 30 is only business EOM; its direct Calendar advancement control reports March 30 | Read the explicit unadjusted directional seed, use `Date::is_end_of_month`, and apply anchored month/year rolls before business adjustment; do not activate from calendar business-EOM queries |
| Collision rejection with contractual retention | Keep contractual dates and adjusted dates in correspondence; reject collapsed or reordered adjusted boundaries | `COLLISION-DAILY-FOLLOWING` returns Jan 9,12,13 from an Unadjusted Jan 9,10,11,12,13 control; Jan 10/11/12 all map to Jan 12. Both returned intervals still report regular | Preserve the original vector, adjust every required boundary through the owned calendar and reject `adjusted[i-1] >= adjusted[i]` with interval/date context; do not sort or deduplicate |
| Zero NonPeriodic metadata | Zero is one nonperiodic interval, with no invented recurrence tenor or periodic frequency | `RULE-ZERO` reports two input endpoints, tenor 0 Years and `isRegular(1)=true` despite a 3-Month constructor input | Separate nonperiodic metadata from periodic regular/stub classification; do not synthesize a zero tenor or coupon frequency |
| Unused out-of-range stopping | Stop at the supplied endpoint before constructing an unused candidate outside the supported date range | The frozen 2026 probe set does not cover reference date-range endpoints; no QuantLib difference is claimed | Compare wide coordinates before constructing the candidate; required endpoint/reference dates still use checked construction |
| Entire-span TARGET guard | Retain checked TARGET policy and validate the full generation span, including Zero and Unadjusted paths | QuantLib's 2026 Easter probe exposes holiday adjustment only; it does not implement SabiQuant's source-validity/disjoint verified-coverage policy | Validate the owned guarded interval even when only two output dates would be generated; joins/overrides cannot turn missing source coverage into verified dates |

The first three differences are observable in the pinned reference probes;
TARGET coverage is a project policy distinction grounded in the existing
[TARGET contract](M3_TARGET_CONTRACT.md). Unused-candidate stopping is a
SabiQuant correctness requirement with an independent exact-date model,
not a newly measured QuantLib divergence. A whole-family compatibility claim
would exceed this evidence.

## Design audit

The adopted distinctions are internally compatible with the existing
foundations. `Date` supplies checked anchored calendar arithmetic and
calendar EOM; schedule recurrence need not borrow the stored-unit business-day
dispatch of `Calendar::advance`. Strict TARGET interval counting provides
a checked whole-span operation without requiring an unchecked calendar
conversion. A nonperiodic interval can be a separate descriptive enum value;
that follows the no-sentinel/no-inferred-frequency API policy.

Contractual and adjusted vectors must have equal boundary counts and exactly
one classification per adjacent contractual pair. Classification belongs
to the unadjusted lattice: an adjusted three-day interval can still have
originated from a one-day recurrence, so adjusted distance cannot establish
regularity. Rejection on collision avoids silently changing that provenance.
Returning a borrowed slice does not grant mutation; index-based public
lookup must return `Option` or a typed bounds error rather than panic.

Direction chooses generation anchor, not output ordering. A backward build
still returns ascending dates. The January-to-October quarterly probes show
that Forward and Backward can legitimately have different residuals and
different date vectors. Equal/reversed ranges and nonpositive recurrent
tenors need explicit domain errors; the Zero identity must not become a
shortcut around range, resource, adjustment or TARGET validation.

The stopping requirement distinguishes unused candidates from required
dates. For a positive tenor larger than the remaining supported span, a
short residual can terminate at the supplied end without constructing an
unrepresentable next regular date. A future ICMA quasi-coupon extension
would be required output/context and cannot receive the same exemption.
Likewise, a date beyond the endpoint must not consume the accepted-boundary
limit or cause an unnecessary calendar query.

No concrete design defect has been established in these five adopted
choices. Implementation review will check that their invariants are
enforced, their named discrepancies match the local contract, and no broader
API or compatibility claim slips through.

The readiness contract resolves the initial scope and precedence questions:
only automatic None/Short/Long stubs are in the tranche; explicit coupon
dates and ICMA generation remain deferred. Builders validate the range once;
builds validate boundary limit, rule/tenor/EOM compatibility, then the full
calendar span before generation or adjustment. EOM accepts only Months/Years
and uses the original start or end as its calendar-only activation seed.
The resource limit applies to final returned boundaries, so a Long residual
may fit a limit that Short exceeds. This detail requires source inspection:
prematurely rejecting the pre-merge count would violate the adopted contract.

The proposed independent [civil oracle](../validation/generator/schedule_oracles.py)
selects dates by bounded ordinal/month enumeration and seed congruence rather
than reproducing a recurrence counter. Its 18,916 successful rows cover all
four units, both directions, month ends/centuries and the applicable stub/EOM
products with exact per-row final limits. Error cases, Zero, adjustment and
TARGET are deliberately separate behavioral tests; this fixture does not
claim to cover those domains. Oracle structure was reviewed independently,
but a passing Rust consumer remains required before it is implementation
evidence.

## Implementation review checklist

| Check | Review criterion | Status at first snapshot |
|---|---|---|
| Public construction | Required dates/options/calendar/limits explicit; only intended rule/stub scope exposed; no default financial conventions | Awaiting source |
| Domains and precedence | `start < end`; positive recurrent tenor; compatible EOM/rule/stub options; resource limits; deterministic errors | Awaiting source and local contract |
| Anchored recurrence | Wide checked offsets; no repeated-clamp drift; exact stop; ascending backward output | Awaiting source |
| EOM | Calendar-only activation from the declared directional seed; original endpoint exceptions explicit | Awaiting source and local contract |
| Zero | Two boundaries, one NonPeriodic classification; no tenor/frequency inference; all context checks retained | Awaiting source |
| Adjustment | Endpoint/interior conventions explicit; one-to-one retained vectors; collision/reversal error rather than repair | Awaiting source |
| TARGET | Private owned guard; whole-span validation; coverage/search failures preserved on joins, overrides and identity-like paths | Awaiting source |
| Bounds and allocation | Accepted boundaries and required work bounded; unused overshoot is not an error; fallible reservation where promised | Awaiting source |
| Errors | Structured offending parameters/indices/dates; stable kinds; domain then nested source chain; no public panics | Awaiting source |
| Value semantics | Private invariant fields, immutable owned context, meaningful equality/hash, Send/Sync, safe borrowed inspection | Awaiting source |
| Independent evidence | Exact-date oracle and behavioral tests cover the actual core contract; frozen reference probes remain separately scoped | Awaiting independent test/oracle candidate |

## Remaining acceptance scope

The local core tranche does not implement seven specialized generation
families, cash-flow analytics, settlement amounts or inferred/generated ICMA
reference periods merely by owning dates. Those require their own bounded
contracts and evidence. NI-01 remains relevant to cash settlement rather
than to discrete schedule date construction. Primary-platform native tests,
formal API/milestone review and publication retain their independent gates.

## Root implementation follow-up

The independent review above reached the design/oracle stage before its
execution quota ended. This later section is the root agent's implementation
review, not a completed independent source review or owner sign-off.

The root inspected the full schedule source and all public tests after their
handoff. The unfinished snapshot needed an explicit i64 conversion in the
available-month calculation, two external non-exhaustive test patterns and
routine lint/format corrections. Those integration failures were corrected
before accepting executable evidence. A redundant `None => endpoint` fallback
inside an already-Some interior path was replaced by a direct pattern/condition;
no ordinary date, flag, option precedence or numerical allowance changed.

The reviewed source enforces the adopted range and build-option precedence,
uses original directional anchors and wide support-range prechecks, retains
TARGET span checks through owned context, then applies chronological
ordinary/termination adjustment with explicit collision/reversal errors.
The limit check proves the minimum possible final count before adding an
interior boundary; Long removes its neighbor before adding a residual endpoint.
On an exact lattice Long cannot merge a regular interval merely to fit a cap.
The available-day/month/year prechecks do not catch and discard arbitrary
date errors. Count/tenor/factor bounds fit i64 under the configured hard cap.

Schedule-vector growth reserves fallibly, while existing calendar Clone keeps
its standard allocation behavior. There is no claim of recovering every
system allocation failure. The copied calendar context adds its own storage/
clone cost to the O(n) date vectors. Generic private invariant and allocation
failure paths are not exercised by corrupting values or forcing system OOM.

Definition equality/hash explicitly preserve the original stored tenor,
policy, direction, calendar and convention metadata. Actual HashMap tests
distinguish nine definitions that happen to share all date/regularity vectors,
then check Zero as a tenth key and reconstructed/cloned lookups. A larger
accepted cap produces the same key because the cap is a work limit, not
retained contractual provenance. Days/Weeks and Years/Months may be normalized
equal Periods while distinct schedule definitions.

Focused debug/release checks cover the 18,916-row independent civil oracle,
65,988 Gregorian-cycle preparations, exact resource-boundary and stub cases,
all 49 adjustment/termination pairs, collisions/reversals, TARGET spans/
overrides/joints and immutable reuse. The public reference consumer compares
25 shared values exactly and pins four EOM, four collision and one Zero
metadata distinction; twelve unsupported observations remain deferred.
Full-span testing reaches all 3,652,059 supported daily boundaries and checks
the cap one smaller. Typed root/source integration samples 70 distinct nested
error kinds. Final aggregate/package/coverage evidence and remaining gates
are in [M7_CORE_VALIDATION](M7_CORE_VALIDATION.md).

No remaining arithmetic or invariant defect was found by this root review.
That is a scoped local result. The independent source review is incomplete,
native Linux/Windows execution is absent, and broader rule/ICMA/cash-flow,
maintenance, formal API/milestone and publication gates remain open.
