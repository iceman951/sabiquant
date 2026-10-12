# Local M4 Date-Context Contract

This records the local rate prototype's composition contract and tests. It is
not M4 acceptance, cross-platform evidence or an individually reviewed owner
decision. The current day-count scope is five stateless conventions plus
explicit ISDA 30E/360, ICMA and Business/252 contexts; it does not claim all
planned market day-count variants are implemented.

## Owned accrual and ordered intervals

An `InterestRate` owns its decimal rate, compounding/frequency and complete
`AccrualConvention`. Date factor and present-value operations compute the year
fraction from that owned value. No default calendar, inferred ICMA schedule or
replacement termination date is introduced by construction or conversion.

Date valuation requires `start <= end`, even though a standalone day counter
can return a negative fraction for reversed endpoints. A reversed rate query
returns `rate.reversed_dates` before accrual lookup or an amount shortcut.
For valid ordered queries, contextual coverage is checked before exact factor
identities: zero rate, equal dates and zero cash-flow amounts do not bypass
ICMA coverage or guarded TARGET policy. A time-only method has no dates to
validate; callers supplying its horizon deliberately take responsibility for
the externally computed fraction.

## Equivalent rates across distinct accrual conventions

`equivalent_rate_between` computes both source and target fractions under
their separate owned contexts, for the same ordered date interval. The target
quote preserves the period's source growth; it does not preserve the numeric
rate when the fractions differ. With continuous compounding, Actual/360 to
Actual/365 (Fixed) changes a 5% quote to `0.05 × 365/360`.

Preservation is a numerical contract within representable active domains.
Near a negative simple rate's zero-factor boundary, rounding the target rate
can amplify factor reconstruction error; a rounded nonpositive active simple
factor must be rejected rather than returned as a usable quote. Factor
round-trip tolerances must account for this target-rate sensitivity instead
of assuming a few ULPs of rate error imply a small relative factor error.

Both contextual computations remain required even when the source rate is
zero. A zero source accrual chooses zero target rate, matching the specified
unit-factor inference policy. A positive source accrual with a nonzero rate
cannot be represented at zero target time and returns `rate.zero_target_time`.
Distinct ordered dates can have zero accrual: 30E/360 counts 30 January to
31 January as zero, and weekends-only Business/252 counts Saturday to Sunday
as zero. Therefore positive calendar-day distance alone does not justify an
inference or conversion denominator.

## Context preservation and typed causes

- ICMA preserves all contractual determination periods and frequency.
  Coverage includes its boundary dates for equal-date queries and is never
  enlarged by quote conversion.
- ISDA 30E/360 retains its explicit termination date. A final February endpoint
  can differ from an intermediate endpoint; converting into another context
  recomputes that target fraction instead of borrowing the source fraction.
- Business/252 retains its immutable calendar, including explicit business-day
  and holiday overrides. Guarded TARGET remains guarded, including omitted
  endpoints, equal dates and intervals spanning an unverified source gap.
- Errors preserve `RateError -> DayCountError -> CalendarError` or
  `RateError -> IcmaError` through standard error sources. Cloning does not
  discard dates, coverage intervals or the original context.

## Present values and numerical evidence

`present_value_between` discounts a finite signed cash flow using the quote's
date fraction. Equal covered dates return the amount exactly, except signed
zero is canonical positive zero. Negative cash flows remain negative. A
nonzero product that overflows or rounds to zero is a typed numeric failure;
a zero amount still requires a valid covered horizon and factor.

Computed transcendental results use the numerical policy's conditioned
tolerances. Their precision is not guaranteed bitwise identical by
[Rust's official floating-point documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.exp).
The exact assertions below are limited to explicit identity paths and basic
arithmetic with demonstrated exact results. Sign and power-of-two scaling of
present values are exact only in the Simple test grid, whose factor path uses
basic fused arithmetic. Other methods combine the two independently evaluated
discounts' conditioned E3 budgets and product rounding; cash-flow scaling does
not establish a `libm` determinism guarantee.

## Independent composition tests

[rates_dates.rs](../tests/rates_dates.rs) covers all eight supported accrual
forms and all 32 compounding/frequency configurations over leap years,
non-leap centuries, year boundaries and 30-day month-end distinctions.
Actual days come from exact date differences. A Monday-epoch weekday prefix
models Business/252 independently of calendar classification. Integer common-
year and leap-year prefixes supply one exact rational ISDA/annual-ICMA oracle;
explicit 30-day anchors distinguish Bond Basis, European and termination
rules. The tests also cover every source/target context and method pair,
zero accrual on distinct dates, typed source chains, immutable overrides,
TARGET gaps, signed cash flows and extreme product failures.

[rates_properties.rs](../tests/rates_properties.rs) adds 20,000 replayable
normal-domain cases using seed `0x5AB1_0004_2026_1011`, with every compounding
configuration visited. It checks monotonicity within conditioned tolerances,
non-strict rounded sign, reciprocal consistency, equivalent-rate round trips,
and present-value linearity conditioned on gross cash flows when net amounts
cancel. Semigroup equality applies only to Compounded and Continuous;
counterexamples distinguish Simple and both hybrids. Eight threads make
8,192 observations of 256 immutable quotes spanning all eight accrual forms,
varying clone and call order. Context copies and explicit identities are
exact; elementary-function observations are compared within their E3 budgets.

These composition tests complement the scalar high-precision and QuantLib
fixtures; they do not replace tolerance calibration, external CI, remaining
convention research or milestone acceptance.
