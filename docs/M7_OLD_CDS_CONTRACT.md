# Local M7 OldCDS schedule contract

Adopted 2026-10-15, Asia/Bangkok, after `f814167` under the continuing
local-only authority. It freezes the broader positive Months/Years
hypothesis of the [API review](M7_OLD_CDS_API_REVIEW.md) and the
[ICMA composition review](M7_OLD_CDS_ICMA_DESIGN.md). It supersedes the
fixed-3M scope of the [initial local design](M7_OLD_CDS_LOCAL_DESIGN.md),
whose terminal-preserving cutoff it retains. Reference observations are
classified in [M7_OLD_CDS_REFERENCE_PROBES](M7_OLD_CDS_REFERENCE_PROBES.md).

This is an explicit **local engineering definition** of an automatic
F-SCHED-01 date rule. Its operands are a generic generation start and a
requested end. Neither is a legal trade, effective, protection or maturity
date, and no T+1, business-day cutoff or premium-accrual convention is
inferred. The [legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md) remains
authoritative that the historical thirty-day equality and start role are
unresolved. Financial acceptance, NI-01, owner and release gates stay open.

## API and domain

`ScheduleBuilder::build_old_cds(tenor, max_boundaries) -> Result<Schedule, ScheduleError>`
uses the builder's start S, end E, calendar and both conventions.

The stored tenor must have unit Months or Years and a strictly positive
length. Years convert to twelve months in i64, giving the month stride m.
There is no divisibility restriction; enormous positive tenors are valid and
simply generate no interior point beyond the first. Zero, negative lengths
(including `i32::MIN`), Days and Weeks are rejected without substituting
another unit. There are no EOM, stub-policy or explicit-role arguments.

`DateGenerationRule::OldCds` has canonical text `OldCDS`, with no aliases.
`DateGenerationRule::ALL` lists ten labels in this order: Forward, Backward,
Zero, the four civil rolls, OldCDS, CDS, CDS2015. `SpecialRollRule::ALL`
keeps four. `build_recurrent` and `build_explicit` reject OldCds with
`InvalidRule`, after their cap check, exactly as for Zero/Roll/CDS labels.

## Exact raw definition

Let Q(x) be the first March, June, September or December twentieth on or
after x. Let A = Q(S) and R = Q(E). Let d be the calendar-day distance A − S.

1. **Recurrence anchor.** B = A + 3 months when A < R and d < 30; otherwise
   B = A. The shift is always three months, independent of m.
2. **Raw boundaries.** S, then every B + k·m months for k ≥ 0 that is
   strictly before E, then R.

Consequences, all exact and proved by construction:

- S < B ≤ R. If A = S then A < R because R ≥ E > S. If 0 < d < 30 and
  A < R, the next quarter after A is no later than R. Otherwise B = A > S.
- B ≥ E implies B = R, so B is always retained: as the first interior point
  or as the terminal. Interior points are strictly between S and E ≤ R, so
  the vector strictly increases and never duplicates S or R.
- Distance 0 retains S once and starts at the following quarter. Distances
  1–29 omit A when A < R. Distances 30 and 31 retain A. When A = R, the
  terminal is retained regardless of distance; R is never extended merely to
  lengthen a short first interval.
- Generation stops at **E**; the final raw boundary is **R**. Stride points in
  [E, R) are not raw boundaries. For m = 3 none exist; for other strides the
  final interval can span skipped stride points or end off the stride.

Every point is a checked civil month coordinate with day 20. No adjusted
date, clamped date or business-day count re-anchors the lattice.

## Regularity and adjustment

An interval is Regular iff both raw endpoints lie on the lattice
L = {B + k·m months : k ∈ ℤ} and their month coordinates differ by exactly m.
Every other interval is Stub, such as an off-lattice start or an off-stride
terminal. An interval spanning an omitted quarter is Stub unless both of its
endpoints are lattice points one stride apart: with 4M, a start on 20
February is B − 4M, so February 20 → June 20 is Regular although March 20 is
omitted. OldCDS never reports NonPeriodic. Adjustment never changes these
flags.

S and interior points use the ordinary convention; R uses the termination
convention. The existing bounded adjustment rejects every adjacent collision
or reversal with `AdjustedOrder`; nothing is sorted, merged or dropped.

## Provenance and identity

New public immutable `OldCdsProvenance` (Copy, Clone, Debug, Eq, Hash,
Send, Sync, 'static) has private fields and these inspectors:

| Inspector | Value |
|---|---|
| `generation_start()` | S |
| `requested_end()` | E |
| `initial_quarter()` | A |
| `recurrence_anchor()` | B |
| `resolved_end()` | R |
| `omitted_quarter()` | `Some(A)` only when B ≠ A and A ≠ S; otherwise None |
| `tenor()` | original stored tenor, unnormalized |

The tenor is stored as length/unit, so 1Y and 12M remain distinct
definitions. The schedule returns `generation_rule() = OldCds`,
`tenor() = Some(original)`, `old_cds_provenance() = Some(&provenance)`, and
None for stub policy, EOM, explicit roles, roll provenance and the ordinary
`lattice_anchor()` seed. Every other constructor sets the new provenance to
None. Schedule equality and hashing include it: different requested ends that
project to identical vectors remain different definitions. Caps are operation
budgets, not identity.

## Validation order, counts and errors

1. `InvalidBoundaryLimit` unless the cap is in `2..=3,652,059`.
2. `InvalidOldCdsTenor` for the stored-tenor domain.
3. Whole original S..E span guard: `Calendar`, index None, phase `coverage`.
4. Required terminal R, then A, then B. Failures return `OldCdsDate` with
   role `terminal quarter` (input E), `initial quarter` (input S) or
   `recurrence anchor` (input A), plus the actual DateError. Terminal-first
   order makes any E after 9999-12-20 a terminal failure; no truncation.
   Once R exists, the initial/anchor failures are proof-excluded.
5. Whole expanded S..R span guard: `Calendar`, index None, phase
   `old CDS coverage`. Because R ≥ E, this is the union span.
6. Exact final count and cap. Let b be B's month coordinate and L be E's
   coordinate, minus one when E's day is at most 20. The interior count is 0
   when b > L, otherwise ⌊(L − b)/m⌋ + 1; the final count adds S and R.
   `BoundaryLimitExceeded.required_at_least` is that exact count. No unused
   candidate is constructed. The attainable maximum is **119,987** boundaries
   (1M, S = 0001-01-01, E = 9999-12-20).
7. Fallible raw generation and storage (`AllocationFailed`). A stride-point
   Date failure would return `OldCdsDate` role `stride point`, input B; it is
   proof-excluded because every generated coordinate precedes E.
8. Existing adjustment: `Calendar` with the zero-based index and phase
   `adjustment`, then `AdjustedOrder`.

| New variant | Stable kind | Context |
|---|---|---|
| `InvalidOldCdsTenor` | `schedule.invalid_old_cds_tenor` | generation_start, requested_end, original tenor |
| `OldCdsDate` | `schedule.old_cds_date` | generation_start, requested_end, original tenor, role, input, DateError source |

Both are non-exhaustive, displayed with their context and wrapped by the
existing root `Error::Schedule`; `OldCdsDate` exposes its DateError through
`source()`. Existing limit, allocation, calendar and order errors are reused
only where their context remains truthful.

## Generated ICMA references

`build_icma_context` selects the anchored kernel with anchor B and no EOM
preservation, read from the provenance; the public `lattice_anchor()` stays
None. Every twentieth exists, so signed stored-unit steps from B produce L
exactly. The existing precedence is unchanged: valid ICMA cap, periodic
source, periodic frequency, normalized tenor/frequency match, bounded
reference generation, whole extended coverage, context construction.

Matching frequencies are 1M Monthly, 2M Bimonthly, 3M Quarterly, 4M
EveryFourthMonth, 6M Semiannual and 12M/1Y Annual. Other positive tenors
build valid schedules but return `IcmaFrequencyMismatch` before reference
arithmetic. The hull is minimal over raw S..R, so it may need several lower
steps and includes an omitted A or skipped stride points only when they lie
on L. Required reference Date causes keep the stored unit (`add years` for
1Y, `add months` for 12M). The whole extended hull is validated under the
schedule's calendar with phase `ICMA reference coverage`; a raw schedule can
succeed while its required references fail. Generic `Leg::fixed_rate`
composition over these raw intervals is analytical only, not a historical
CDS premium leg.

## Classified reference distinctions

Frozen OldCDS roll probes: 98 rows. 85 have positive Months/Years, EOM false
and no explicit roles; all 85 agree in dates (56 also in flags). Four are
outside the tenor domain (zero, negative, 1D, 1W) and nine use EOM or
explicit roles, which this operation does not accept. Fresh attested probes:
551 rows; 152 agree exactly, 299 differ only in flags and 100 in dates.

| Class | Fresh rows | Reference observation | Local contract |
|---|---:|---|---|
| T1 off-quarter exact end | 41 | E equal to a non-quarter stride point becomes the terminal | R = Q(E) always; terminal monotone in E |
| T2 adjusted-end coincidence | 11 (+12 with S1) | Projected R omitted when adjusted E equals the adjusted last stride point | Raw terminal R is independent of adjustment |
| S1 holiday start | 36 (+12 with T2) | First date returned unadjusted | S uses the ordinary convention, like every Schedule |
| F flags | 299 | Projected terminal and some first intervals flagged by construction shape | Regular only for exact lattice intervals |

These are deliberate local choices, not tolerances. No reference row is
rewritten; neither agreement nor classification establishes complete
reference parity or historical market acceptance.

## Evidence required

Independent civil enumeration over a full Gregorian cycle, every distance
class and all supported strides; exact counts, caps and cap ± 1; the civil
maximum; terminal failures after 9999-12-20; huge tenors; stored aliases and
identity; every error, role and precedence; all 49 convention pairs with
holiday starts/terminals; collisions; original and expanded TARGET guards
with policies, overrides and joint calendars; immutable thread reuse; exact
ICMA hulls, rational fractions, caps, causes and coverage; and exact
classification of both reference sets. Dates, counts and flags use exact E0;
fractions keep the existing E2 policy. Native debug/release, strict lint,
docs, package and cross-target checks are reported separately.
