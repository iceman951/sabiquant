# Same-roll ICMA source and test review

The root independently reviewed the production diff authored by
`/root/roll_icma_plan`, the hand suite authored by `/root/roll_icma_tests`, and
the generator/consumer authored by `/root/roll_icma_oracle`. The production
implementer separately reviewed the oracle's civil enumeration, rational
independence and Date-error index arithmetic before its quota interruption.
The root added the whole-civil-range resource test and owns the final aggregate
and actual package verification. This is not a claim of independent review
of the root's own test or a formal financial/owner approval.

## Findings and exact scope

No concrete production defect was found in the adopted four-rule extension.
The new private lattice dispatch leaves ordinary Date/EOM candidate arguments
and the shared hull algorithm intact. Frequency validation precedes anchor
reconstruction and required extensions. Roll point generation shifts q(0)'s
month then selects the new Wednesday/day20, so clamping cannot change its seed.
Source requested/resolved provenance remains immutable and context equality
continues comparing actual references and frequency.

For current roll builders, q(0) lies inside the minimal hull. Wednesday raw
starts share its month; twentieth starts occur after the preceding eligible
roll and at/before their anchor. At most one negative step is needed for these
rules. The shared loop remains correct when a same-month raw end lies before
q(0): the upper-period lower bound is exactly `u32::from(anchor < end)`.
Crossing q(0) proves two periods before required predecessor arithmetic.

Roll offsets are at most `12 * i32::MAX * MAX_ICMA_PERIODS`, safely in i64;
successful frequency matching further limits the month stride to 1..12.
Required dates remain checked and cannot be suppressed by the schedule
generator's unused-candidate rule. Lower-first evaluation and boundary-index
causes are preserved. Anchor reconstruction errors are proof-excluded by
private valid provenance; no forced OOM/private corruption was used.

The original requested/resolved calendar guard remains on source construction.
The shared reference builder independently guards the entire extended hull,
including endpoints, gaps, overrides and joints. Contexts retain dates and
frequency without creating additional calendar permission. All 196 hand
ordinary/terminal pairs produce the same raw contexts for the chosen custom
calendar and four schedules; this is not every possible calendar definition.

## Requirement traceability

| Requirement | Inspected evidence |
|---|---|
| FR-SCHED-14 explicit matched frequency | All 21 eligible rule/frequency combinations, stored annual aliases, all alternate named frequencies, Once, huge mismatches and oracle domain records |
| FR-SCHED-15 minimal same-anchor hull | Existing ordinary ICMA fixtures/properties remain unchanged; literal same-month before/after/crossing controls and independent civil enumeration check the new point function |
| FR-SCHED-16 shared caps and immutable context | Exact/cap−1/cap+1 oracle controls, every insufficient cap on literal four-rule hulls, cap-before-required-Date controls, actual whole-civil hulls and concurrent value/hash/query reuse |
| FR-SCHED-17 complete extended TARGET guard | 56 strict/projected/override/joint cases, inception failures, resolved inclusive endpoints, adjustment-invariant raw hulls and unchanged ordinary coverage regressions |
| FR-SCHED-21 periodic roll dispatch | Removed temporary unreleased unsupported variant; existing roll tests now check cap/Once/mismatch/valid frequency. Recurrent/explicit builder rejection is unchanged |
| FR-SCHED-22 same-roll raw minimality | Exact Wednesday/twentieth membership, all 4,800 cycle months under every rule and two rotating profiles, off-stride terminal references, requested-versus-resolved controls and 126,537 exact Fraction queries |
| FR-SCHED-23 limits, causes, calendar and composition | Nine literal required-date cases, stored-Year/month cause equivalence, full typed root chains, original error phases, generic shared-context coupons and direct zero-rate valuation |

The hand suite checks full-period Regular fractions against the explicit
event divisor. Coupon controls use N=8 and r=1/8 so the final product equals
the supplied accrual fraction; independent fractions and pinned rounded-sum
values distinguish full-span accrual from a sum of rounded coupon amounts.
This is generic coupon composition, not CDS premium-convention evidence.

The whole-civil hand test actually constructs 119,987 periods for each
Wednesday rule and Twentieth, and 39,995 for TwentiethIMM. It checks every
reference date, adjacency, independent Gregorian ordinal Wednesday membership,
quarter membership, exact endpoints, final cap−1 and full-span accrual. Exact
upper endpoints prove no unnecessary year10000 successor is constructed.

Final counts, source hashes, command exits, source-region limits and package
bytes are recorded in [aggregate validation](M7_ROLL_ICMA_VALIDATION.md).
The [oracle report](M7_ROLL_ICMA_ORACLES.md) states its finite sampling and
input/record distinction. No full M7, standard CDS, native Linux/Windows or
release completion follows from this technical review.
