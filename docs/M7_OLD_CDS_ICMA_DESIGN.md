# Proposed ICMA composition for the broader local OldCDS rule

Independent kernel/design review, 2026-10-15 UTC. This note concerns the
broader positive-Months/Years hypothesis currently under review, rather than
the earlier fixed-3M proposal in [the local design](M7_OLD_CDS_LOCAL_DESIGN.md).
It is not an adopted contract or implemented feature. Fresh public probes
and the root's contract freeze must settle the recurrence before production
oracles or implementation adopt it. This review changed no production code,
existing fixture, existing document or Git state.

The discussion specifies generic analytical ICMA references for a locally
defined raw Schedule. It does not establish that historical CDS premiums use
ICMA, resolve the historical trade/effective/start-role question, or approve
a financial OldCDS contract. Existing [generated ICMA](M7_ICMA_REFERENCE_CONTRACT.md),
[civil-roll ICMA](M7_ROLL_ICMA_CONTRACT.md) and
[cash-flow](M7_CASHFLOW_CONTRACT.md) contracts retain their own scope.

## Hypothesis being evaluated

Let S be generation start, E the requested end, and require S < E. Let A be
the first quarterly twentieth on or after S and R the first quarterly
twentieth on or after E. The positive stored Months/Years tenor defines a
month stride m, with Years converted to twelve months for civil-coordinate
selection. The hypothetical first recurrence anchor B is:

- If A = R, retain the required terminal: B = A.
- Otherwise, if A = S or 0 < A−S < 30 calendar days, B = A + three months.
- Otherwise, B = A.

Retain S, generate B + k*m for k >= 0 strictly before **E**, then append R.
The stopping boundary is E, while the final raw endpoint is R. These are
different inputs. Positive monthly strides need not retain only quarterly
dates after B. This note does not replace that hypothesis with the earlier
fixed-quarter rule or infer it from a few old reference vectors.

This definition proves S < B <= R. If A = S, increasing S/E implies A < R,
so its next quarter is no later than R. If the short-front rule applies,
consecutive quarterly A/R likewise bound the next quarter. Otherwise B=A>S.
The proof matters to ICMA: B lies inside the minimal reference hull of raw
S..R, including the case B=R. An out-of-range required R is a schedule-build
failure; ICMA must not repair it.

Provenance should retain S, E, A, R, B, the optional **actually omitted**
interior A, and original tenor length/unit. The omitted value is absent when
A=S (S is retained) or A=R (the terminal is retained). Missing later monthly
stride points caused by the E stopping rule are not mislabeled as the omitted
initial quarter. Schedule Eq/Hash retain these inputs and stored aliases.

## Compatibility with the existing ICMA kernel

The current `Schedule::build_icma_context` can support the proposed lattice
through an internal `IcmaLattice::Anchored { anchor: B, preserve: false }`.
Every twentieth exists, so direct signed month/year shifts from B exactly
produce the intended stride lattice without EOM adjustment or clamping drift.
The public `lattice_anchor()` can remain None for OldCDS: it continues to
describe ordinary Forward/Backward seeds. Internal reference selection must
read B from OldCDS provenance rather than route through that public inspector.

Use the retained raw endpoints S and R. Neither E nor adjusted payment dates
replace them. For the same signed lattice q(k), the minimal hull is bounded
by the greatest q(L) <= S and least q(U) >= R. All intervening lattice periods
are required, even when a corresponding raw coupon boundary was omitted or
was excluded by the stop-at-E generation rule.

An omitted quarterly A is **not automatically** a determination boundary.
It appears only if it belongs to the B+k*m lattice. For A=2026-Mar20 and
B=2026-Jun20, A is a reference boundary for 1M and 3M, but not 2M, 4M, 6M
or 12M. Inserting it unconditionally would change the declared frequency's
reference periods. The fixed-quarter statement that the omitted March coupon
is always a quasi-boundary must therefore not be generalized to this API.

The current kernel's lower-then-upper loops are compatible because B lies
within the hull. They already permit multiple negative steps, reverse the
negative sequence once, retain B once, and share one period budget across
both directions. The civil-roll proof of at most one negative step does not
apply here. Update explanatory comments without weakening the general loops.

The matching named frequencies for this proposed monthly/yearly domain are:

| Stored tenor | Explicit matching frequency | Annual event count |
|---|---|---:|
| 1M | Monthly | 12 |
| 2M | Bimonthly | 6 |
| 3M | Quarterly | 4 |
| 4M | EveryFourthMonth | 3 |
| 6M | Semiannual | 2 |
| 12M or 1Y | Annual | 1 |

Other positive tenors can belong to the proposed schedule domain without a
matching existing frequency. For example 5M, 9M, 18M, 2Y and huge positive
stored tenors must return the existing frequency-mismatch error before ICMA
reference arithmetic. Once remains invalid; daily/weekly frequencies cannot
match these month/year tenors. Keep normalized frequency equality and stored
tenor identity separate. Contexts from 1Y and 12M may compare equal while
their source schedules remain distinct definitions.

## Multiple lower steps and exact rational controls

Take S=2026-Feb19 and E=2026-Jun01. A=Mar20 is 29 days away and is omitted;
B=R=Jun20. Every listed tenor therefore gives the same two raw endpoints,
but its reference hull and frequency remain explicit and can differ.

| Tenor/frequency | Reference boundaries | Period count | Exact fraction S..R |
|---|---|---:|---:|
| 1M / Monthly | Jan20, Feb20, Mar20, Apr20, May20, Jun20 2026 | 5 | 125/372 |
| 2M / Bimonthly | Dec20 2025, Feb20, Apr20, Jun20 2026 | 3 | 125/372 |
| 3M / Quarterly | Dec20 2025, Mar20, Jun20 2026 | 2 | 119/360 |
| 4M / EveryFourthMonth | Oct20 2025, Feb20, Jun20 2026 | 2 | 124/369 |
| 6M / Semiannual | Dec20 2025, Jun20 2026 | 1 | 121/364 |
| 12M or 1Y / Annual | Jun20 2025, Jun20 2026 | 1 | 121/365 |

The Monthly result is `1/(12*31) + 4/12 = 125/372`: moving back four months
from Jun20 only reaches Feb20, still after Feb19, so Jan20 is also required.
Quarterly gives `29/(4*90) + 1/4 = 119/360`. EveryFourthMonth gives
`1/(3*123) + 1/3 = 124/369`. These are distinct denominator controls rather
than six copies of one expected fraction.

Additional hand controls:

| Inputs | Raw intervals and exact fractions | Required reference feature |
|---|---|---|
| S=2026-Feb06, E=May01, 1M/Monthly | Feb06..Mar20: 15/124; Mar20..Apr20: 1/12; Apr20..Jun20: 1/6; whole: 23/62 | B=Mar20, R=Jun20; May20 is absent from raw coupons because it is after E, but remains a required reference boundary. Hull Jan20..Jun20 has five periods. |
| S=2026-Mar20, E=Aug01, 6M/Semiannual | Mar20..Jun20: 23/91; Jun20..Sep20: 46/183; whole: 8395/16653 | A=S is retained, omitted-quarter None; B=Jun20, R=Sep20. Hull is Dec20 2025, Jun20 2026, Dec20 2026. Neither raw outer interval is an entire six-month period. |
| S=2024-Feb20, E=Jun01, 3M/Quarterly | Feb20..Jun20: 30/91 | Leap control: first piece is 29/(4*91), followed by 1/4. |

A one-off independent Python calculation used `datetime` civil dates and
`Fraction`, enumerated quarter candidates, and selected day-20 reference
months by congruence. It verified the table values above. It did not load
SabiQuant, invoke QuantLib or inspect C++ implementation. A separate one-off
enumeration of all **146,097 dates** in 2000..2399, using a sufficiently late
E so A<R, found maximum required negative step counts of **5, 3, 2, 2, 1, 1**
for matching strides 1/2/3/4/6/12M respectively. The first five-step Monthly
case in that enumeration was 2001-Feb19. These are completed proposal-level
mathematical checks, not tests of an implemented OldCDS builder or a retained
external oracle. Executable regression/oracle artifacts must follow the
root's adopted contract.

## Caps, typed causes and civil endpoints

Retain current precedence: valid ICMA cap; periodic source; valid frequency;
normalized tenor/frequency match; bounded required reference generation;
whole reference-hull coverage; period/context construction. A Schedule
boundary cap and an ICMA determination-period cap measure different results.
The two-boundary first example above needs five Monthly reference periods.

The existing lower-loop check is `count + 1 + remaining_upper`, where
`remaining_upper = 1` only when B<R. This is a proved lower bound on required
periods, not an up-front exact hull count. It must be checked before each
required negative date. When B=R, no extra upper period is reserved or built.
The upper loop checks its next required period before constructing that date.
No schedule-generator rule permitting an unused candidate beyond E may be
reused for a required ICMA reference beyond R.

Using the Anchored variant deliberately preserves stored-unit Date causes:
1Y uses `add years` with signed year count, while 12M uses `add months` with
signed month count. Both causes retain B. This follows the existing ordinary
anchored kernel and differs from civil-roll provenance's always-month causes.
Freeze this choice explicitly; do not normalize away the original tenor
before calling `checked_anchored_date`.

Required boundary controls include:

| Inputs and frequency | Expected kernel behavior |
|---|---|
| S=0001-Feb01, E=Apr01, 1Y/Annual; B=Mar20, R=Jun20 | Cap1 fails with required-at-least2 before arithmetic. Cap2 attempts the required predecessor and returns Date cause at boundary-generation index1, anchor 0001-Mar20, operation `add years`, count −1. |
| Same with 12M/Annual | Same cap behavior, but cause is `add months`, count −12. |
| S=0001-Jan01, E=Apr01, 1M/Monthly; B=Mar20, R=Jun20 | Lower steps Feb20 and Jan20 are valid but still after S. Cap3 fails with required-at-least4 before the third lower step. Cap4 reaches required year-zero Dec20 and returns index3, anchor Mar20, `add months`, count −3. The kernel has reserved only a proved minimum of one upper period. |
| S=9999-Feb01, E=Apr01, 1Y/Annual; B=Mar20, R=Jun20 | Cap1 fails with required-at-least2. Cap2 retains the lower 9998-Mar20 reference, then fails at index2 for upper year 10000, cause anchor 9999-Mar20, `add years`, count +1. 12M instead preserves count +12 months. |
| S=9999-Feb01, E=Oct01, 4M/EveryFourthMonth; B=Mar20, R=Dec20 | Raw generation retains Mar20/Jul20 and final Dec20. ICMA additionally requires Nov20 and then Mar20 of year 10000. Cap3 fails with required-at-least4; cap4 attempts that final reference and returns index4, anchor Mar20, `add months`, count +12. |
| S=9999-Dec01, E=Dec02, 1Y/Annual; A=B=R=Dec20 | Required terminal exception; one lower reference at 9998-Dec20 suffices. Cap1 succeeds without querying a successor in year 10000. |

All reference-date errors retain `ScheduleError::Date`, phase
`ICMA reference extension`, the boundary-generation position before reversal,
and the actual DateError source. No fictitious final-array index is supplied.
Magnitude multiplication remains bounded by the existing cap/i64 proof;
frequency mismatch rejects large unsupported strides before they can extend
references. Allocation failures and proof-excluded period/context wrappers
retain their current source behavior without fabricated private values.

## TARGET and generic coupon composition

The OldCDS Schedule constructor must first retain its own original/projected
span guard. The ICMA operation then independently validates the entire
extended q(L)..q(U) hull under that Schedule's calendar. Its existing failure
has phase `ICMA reference coverage`, index None and the preserved Calendar
cause. Verified endpoints cannot bridge an uncovered interior gap; overrides
and joint Null members cannot waive the retained TARGET policy. Adjusting a
payment never repairs missing raw reference coverage.

Useful distinguishing control: S=2026-Feb19, E=Jun01 succeeds as a raw 2026
schedule under strict TARGET if all other build options succeed. Monthly
references Jan20..Jun20 stay within 2026 verified coverage. Quarterly,
Bimonthly, EveryFourthMonth, Semiannual and Annual references extend into
2025 and must fail strict coverage; explicit projection can permit covered
rule projection but cannot erase inception restrictions. Full coverage of
both historical verified intervals and their interior gap remains required.

`Leg::fixed_rate` can consume the proposed Schedule only as generic analytical
raw-interval composition: each coupon accrues from its raw start to raw end
using the explicitly supplied ICMA context, with payment at the corresponding
adjusted end. It does not insert final-day inclusion, replace accrual starts
with payment dates or infer a CDS premium convention. Ordinary/terminal
adjustment changes must leave the ICMA hull and raw fractions unchanged.

For monetary controls, retain the existing exact-binary-operand product oracle
and propagated day-count tolerance. A rational fraction above is mathematical
accrual truth, not a promise that a binary64 coupon amount equals an ideal
decimal-currency result. An exactly representable simple control is
S=2026-Mar20 with 1M stride and B=Jun20: the first raw interval contains three
Monthly determination periods, fraction 1/4, and notional 1000/rate 1/8 gives
amount 31.25. Irregular fractions should exercise the existing E2 comparison
and coupon-product rules without widening tolerances.

## Implementation and validation obligations after freeze

1. Add OldCDS provenance selection to the internal ICMA lattice match, keeping
   public ordinary-seed, roll-provenance, EOM and explicit-role inspectors
   truthful. Update comments that currently discuss only ordinary/civil rolls.
2. Pin the actual B/stride rule independently after the public-probe review.
   Include distances 0/1/29/30/31 around all four quarters, leap/century classes,
   A=R terminal retention, B=R, raw E before/on/after a stride point, and a
   projected R off the retained stride lattice.
3. Retain all six matching frequencies, 1Y/12M identity versus equal contexts,
   invalid Once/day/week and nonmatching positive month/year tenors. Cover
   clipped, zero, reversed and hull-boundary accrual queries.
4. Pin the rational tables, exact/minus/plus ICMA caps, shared lower/upper
   budgets, all civil errors above, and successful no-unused-date endpoints.
   Compare date/hull/count metadata at exact E0 and fractions at existing E2.
5. Check all adjustment pairs for context invariance; strict/projected TARGET
   extension, inception, gap, override/joint controls; raw coupon composition;
   immutable equality/hash and thread reuse; and unchanged modern CDS paths.

The kernel read for this proposal was `src/schedule.rs` SHA-256
`b6cd16a27cb4e89e5679baebe0e33c4be94670d3ed4c6e2af69d8638125e88a5`.
No OldCDS implementation, fresh reference-runtime observation or financial
acceptance is claimed here. Full M7, historical-source, owner/native-platform,
NI-01 and publication requirements remain outstanding under their own gates.
