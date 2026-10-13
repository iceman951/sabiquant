# Local M7 generated ICMA-reference contract

Adopted under continuing local-only authority after explicit schedules were
saved at `d570b45`. This is a bounded analytical reference-context extension,
not full M7 acceptance, adjusted-accrual product approval or exact settlement.

The [ICMA March 2022 Appendix A5](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf)
was reread directly on 2026-10-12 UTC/13 Bangkok. Version 2 §1.3 splits accrual
over determination periods with an explicitly normal annual event count; its
note permits longer multi-period accrual. The payment-day note supports keeping
contractual interest dates separate from payment adjustment. It does not
specify this library's generic roll generator or certify all named day/week
frequencies as market conventions. The adopted named lattice supplies those
separate inputs to the existing [explicit ICMA contract](M3_ICMA_CONTRACT.md).

## API, frequency and absence

`Schedule::build_icma_context(frequency: Frequency, max_periods: u32)` returns
`Result<ActualActualIcma, ScheduleError>`. There is no cache, implicit frequency,
calendar adjustment or inferred coupon interval. Zero cannot generate a
periodic context. Once is invalid. The supplied frequency's period must be
normalized-equal to the stored schedule tenor. Thus 12M/1Y and 7D/1W may be
compatible, while two years/Annual or two days/Daily are incompatible. The
schedule continues retaining its original stored unit independently.

All ten existing periodic frequencies are supported with their explicitly
declared annual counts: 1,2,3,4,6,12,13,26,52,365. Day/week counts are delegated
analytical scope; neither a 364-day weekly year nor a leap-year daily schedule
changes the declared divisor. No broad market-convention parity follows.

The returned context owns ordered positive contiguous DeterminationPeriods
and the explicit frequency. It has no calendar because queries use only these
immutable validated dates and require their coverage. It does not authorize
future TARGET queries or convert projected source dates into verified ones.
Schedule/calendar provenance remains available on the original Schedule.

## Minimal hull on the retained contractual lattice

Use `lattice_anchor()`, the stored tenor and retained EOM policy from the built
Schedule, including automatic Short/Long and explicit first/penultimate/equal
roles. Every quasi-date is derived from that SAME anchor at a signed integer
step, with calendar-EOM activation from that anchor alone. Never reuse a
clamped result, adjusted payment date or removed Long boundary as a new seed.

Generate the MINIMAL contiguous lattice hull qL <= original raw start <
original raw end <= qR. qL is the greatest lattice date no later than start;
qR is the least lattice date no earlier than end. Exact lattice endpoints
need no extra predecessor/successor. Long outer spans include every
intervening quasi-period, including dates omitted from contractual coupons.
Equal explicit roles can therefore produce multiple determination periods.
Nothing forces an irregular contractual outer endpoint onto the lattice.

Starting from the known anchor, generate negative steps until the raw start
is covered, then positive steps until the raw end is covered. Reverse the
negative sequence once; retain the anchor once. Adjacent chronological dates
become positive half-open DeterminationPeriods. Current positive supported
tenors guarantee strict progress; counts and wide scaled-tenor arithmetic are
bounded by the civil range and period cap. Construction is O(reference periods)
with fallible boundary/period vector reservation, plus declared calendar costs.
Existing ICMA accrual queries remain O(log n+k) and retain the established
TOL-E2 four-ULP rational-oracle profile. No tolerance is widened.

Every extension date is REQUIRED. If checked civil arithmetic leaves years
1..9999, preserve the actual typed DateError in ScheduleError::Date with phase
`ICMA reference extension` and the boundary-generation position before final
chronological ordering. The date cause retains original anchor, operation and
signed scaled offset. The schedule generator's proved unused-candidate absence
is not permission to drop a required reference date.

## Resources, guard and errors

Associated constants `Schedule::MIN_ICMA_PERIODS = 1` and
`MAX_ICMA_PERIODS = 3_652_058` define valid final-period caps. The cap counts
positive determination periods, not schedule coupon dates or reference
boundaries. Reject unsupported caps before other work. A lower-bound error
retains the caller's original cap and minimum required period count. Both
required directions consume the same budget; an interior anchor proves at
least two periods before any date arithmetic.

Precedence: cap validity; periodic schedule; named periodic frequency;
normalized tenor/frequency match; bounded required-date generation; full
extended TARGET span; period/context construction. A proved cap insufficiency
precedes computing the disallowed required date. No partial context is
returned. Once a finite reference hull exists, check its ENTIRE original
calendar/TARGET span including both quasi endpoints, omitted holidays, gaps,
overrides and joint calendars, under the retained strict/projected policy.
Coverage failures use ScheduleError::Calendar with index None and phase
`ICMA reference coverage`. Ordinary/termination conventions do not adjust
reference dates. A projection opt-in remains visible on the source Schedule.

New schedule kinds: `schedule.invalid_icma_period_limit`,
`schedule.non_periodic_icma`, `schedule.invalid_icma_frequency`,
`schedule.icma_frequency_mismatch`, `schedule.icma_period_limit_exceeded`,
and `schedule.icma` (preserved IcmaError source). Existing allocation/date/
calendar wrappers retain typed causes and phases. An invalid returned period
or context is proof-excluded in current valid construction, but its typed
cause is preserved rather than unwrapped. Catalog tests are not manufactured
public failure paths.

## Executable obligations and open scope

Independent civil-coordinate hull/date/period oracles and exact Fraction
accrual rows must cover every frequency, directions, raw automatic short/long
and explicit outer shapes, equal roles, calendar EOM and month-clamp recovery,
centuries/leaps, multiple quasi-period stubs, clipped/reversed/zero accrual and
queries at hull boundaries. Required failure cases include Zero/Once/mismatch,
cap +/-1, shared-direction cap insufficiency, civil MIN/MAX required extension,
no unnecessary out-of-range/guard query at exact endpoints, full extended
TARGET gap/inception/candidate ranges, projected policies, and source layering.
Check maximum possible one-day references, immutable Eq/Hash/thread reuse,
all new kinds/root conversions and unchanged existing tests. Reference probes
must supply the same explicit determination inputs; they cannot hide a
disagreement by asking the reference to guess missing periods.

Full M7 still needs seven specialized rules and cash-flow/coupon/leg/NPV/
accrued-interest scope. Formal owner/financial acceptance, native primary
platform CI, market-source/maintenance and release gates remain separate.
