# Explicit Actual/Actual ICMA contract

This is the staged local FS-DCC §9.5 implementation, with TS-DCC-01/03/07/08
evidence. It does not claim full M3 completion or external platform approval.

## Primary-source definition

[ICMA Primary Market Handbook, Appendix A5, March 2022](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf)
was read directly on 2026-10-11. Version 2 §1.3(a) divides accrued days within
a determination period by its length and the annual determination frequency.
Section 1.3(b) adds contributions when accrual spans periods; the final note
allows extension beyond two periods. The March 2022 note distinguishes
contractual accrual dates from business-day-adjusted payment dates. This
implementation accepts explicit accrual/reference dates and performs no
calendar adjustment.

## API and domain decisions

- `DeterminationPeriod::new(start, end)` requires `start < end`; fields are
  private and getters return the supplied dates.
- `ActualActualIcma::new(periods, frequency)` requires a nonempty, supplied
  chronological vector with exact adjacency. It rejects gaps, overlaps,
  duplicates and unordered periods without sorting or expanding them.
- Every named periodic `Frequency` is accepted; Once is rejected. Frequency
  is checked before an empty vector. The declared count determines the
  divisor. The constructor does not guess frequency from date differences,
  verify a market schedule, or generate quasi-coupon boundaries; callers
  supply those contractual inputs, including reference periods for stubs.
- `year_fraction(start, end)` requires both endpoints inside the closed
  outer boundary range. Zero accrual at either boundary is valid and yields
  positive zero; zero accrual outside coverage is an error. Reversed queries
  validate the same coverage, compute chronologically, and negate once.
- `day_count(start, end)` is the signed actual-day difference for any valid
  dates and does not require reference coverage. It remains defined when a
  fraction cannot be calculated.
- Values are immutable and `Send + Sync`; there is no default convention,
  implicit schedule, clock, global state, I/O or third-party crate dependency.

Construction is O(n). Fraction queries locate the first relevant period by
binary search, then visit only overlapping segments: O(log n + k).

The stable error kinds are `icma.invalid_period`, `icma.empty_periods`,
`icma.invalid_frequency`, `icma.non_contiguous_periods`, and
`icma.out_of_coverage`. Errors retain rejected dates/frequency or the offending
index and preceding boundary. They implement the standard error traits and
convert through `Error::Icma` with the domain error as its source.

## E2 justification for many segments

Each overlap contributes `days / (F × reference_days)`. The integer product
is at most `365 × 3,652,058`, so its floating representation is exact. Positive
periods cannot number more than 3,652,058 within the supported date range.
Every contribution is positive, finite and normal; the sum is bounded by the
period count. Reversal introduces no cancellation in the summation.

The code uses chronological Neumaier summation, selecting the larger addend
before recovering its exact addition residual. This is the compensated
algorithm discussed in §4 of [Ogita, Rump and Oishi, *Accurate Sum and Dot Product* (2005)](https://www.tuhh.de/ti3/paper/rump/OgRuOi05.pdf).
Their Proposition 4.5 bounds its error on rounded inputs by
`u × |sum| + γ(n−1)² × sum(|terms|)`, where
`u = 2⁻⁵³` and `γ(m) = m × u / (1 − m × u)`.

For this positive sum, conditioning is 1. Adding the quotient-rounding error
gives the derived bound
`[u + (u + γ(n−1)²) × (1 + u)] × exact_sum`.
At the maximum period count, `γ(n−1)² < 1.644 × 10⁻¹⁹`; the coefficient is
less than `2.002 × u`. Allowing the correctly rounded oracle's own rounding
still fits the conservative **4 ULP TOL-E2** comparison. This argument uses
IEEE round-to-nearest arithmetic and exact integral inputs; it does not rely
on the earlier five-rounding proposal. Equal endpoints are exact zero.

## Measured evidence

Eleven integration tests pass on Rust 1.99.0/aarch64 macOS. They cover all
error kinds and structured contexts; all frequencies; regular, short-first,
long-first, short-final and long-final shapes; coverage and reversal; full
date-range denominators; public value/error traits; and actual reference data.

The independent integer-weight oracle tests **3,338,660 ordered endpoint
pairs** over 1999–2002 and 2099–2101, including outer reference boundaries.
It assigns each calendar day an exact `LCM(28,29,30,31)/month_length` weight
and rounds the resulting rational once. Maximum observed error is **1 ULP**.
The maximum **3,652,058 one-day segments** produce **0 ULP** error against
the single-division exact rational result. Late clipped intervals exercise
binary search independently of total reference count.

Committed fixtures contain 101 Python `Fraction` cases (maximum 1 ULP),
85 actual QuantLib 1.43 component scenarios (maximum 1 ULP), and two explicit
coverage divergences. Fixture consumers independently reconstruct reduced
checked i128 fractions from supplied overlaps and assert numerator/denominator
metadata exactly; Python supplies the correctly rounded binary64 target. The generator checks the downloaded wheel against every
installed module file, then records its SHA-256 and generator revision.

QuantLib comparisons use public `ActualActual(ISMA).yearFraction` calls with
explicit reference starts/ends for each supplied segment. A full-reference
probe must equal `1/F`, proving the intended frequency interpretation before
comparison. Component values are recorded separately; multi-segment totals
are an external `math.fsum` composition of those calls, not a claim about an
inferred QuantLib schedule. Weekly/daily and arbitrary reference shapes that
the API cannot express with the declared frequency receive oracle evidence
only; they are not labeled differential parity.

`D9-ICMA-ZERO-COVERAGE` is restricted to exactly two recorded cases with
reference dates 2024-01-01/2024-07-01 and Semiannual frequency: equal accrual
dates 2023-12-31 and 2024-07-02. QuantLib returns zero; SabiQuant deliberately
returns `icma.out_of_coverage`. No other discrepancy is automatically allowed.

```sh
python validation/generator/icma_fixtures.py --quantlib-wheel /path/to/quantlib-1.43-wheel.whl --check
cargo test --offline --test icma -- --nocapture
cargo clippy --offline --test icma -- -D warnings
```

Run the generator with the attested external Python/QuantLib environment.
The crate consumes committed data using standard-library test support; it
does not execute QuantLib. Global M3 sources, remaining conventions and
cross-platform CI retain their separate readiness gates.
