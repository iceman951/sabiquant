# M8 public bond reference design and exploratory observations

Pre-contract investigation, 2026-10-16 (Asia/Bangkok), baseline `4da3f43`.
This is a finite public-behavior sample, not a frozen fixture, independent
oracle, adopted market convention, or completed M8 acceptance. Only this new
document is repository output; temporary scripts/JSON live outside the tree.
No QuantLib C++ or generated Python implementation body was inspected.

## Runtime evidence and bounded execution

The existing frozen [schedule probe attestation](../validation/generator/schedule_probes.py)
was loaded by path. Its `attest` function verified the wheel and every installed
shipped module **before** each native QuantLib import. Public runtime version
then matched `1.43`. A separate isolated help process collected only public
`__doc__` strings. Native observations used one new `python -I` subprocess per
case, the helper's ten-second worker timeout, cleaned `PYTHONPATH`/`PYTHONHOME`,
and at most six emitted cash flows. Every issue/maturity/settlement date lies
in 2024–2026, maturities follow issue, and all numeric inputs are finite.
No worker timed out or failed outside recorded public query errors.

| Evidence | SHA-256 / value |
|---|---|
| Wheel `quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl` | `85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c` |
| Installed three-file module manifest | `e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d` |
| Frozen attestation helper | `2c48705c96a6311922e8483cc47ae383f032555fd56f4c3f53567ca605725d6d` |
| `/private/tmp/sabiquant-m8-bond-help.py` | `18b70646b57d4de40b42058799dcf0258e99831004bac6dadd40b9634242733a` |
| `/private/tmp/sabiquant-m8-bond-help.json` (9,146 bytes) | `4eaa57d753b4ff1978eff611cef6ea18fd809ef42e497795aee569928eacf41d` |
| `/private/tmp/sabiquant-m8-bond-probes.py` | `76745f79717280ff9d1d0f6a85f85498156c761f9b1bfd550d566252119cc19d` |
| `/private/tmp/sabiquant-m8-bond-probes.json` (412,953 bytes) | `90ae82ac8595669394870c7c49a054dfc17f332794b95819afdcd1533945a1dc` |

Temporary reproduction, while these scratch files remain available:

```sh
/private/tmp/sabiquant-validation-venv/bin/python -I /private/tmp/sabiquant-m8-bond-help.py
/private/tmp/sabiquant-validation-venv/bin/python -I /private/tmp/sabiquant-m8-bond-probes.py
```

The second command writes only its temporary JSON/log evidence. It does not
write a fixture. JSON retains every case's constructor/convention inputs,
query argument lists, cash-flow metadata, runtime manifest, public error type
and unmodified error text. Output floats retain both Python `repr` and `hex`,
including `inf`/`nan` as strings; no invalid JSON numeric tokens are emitted.
Case scalar inputs use JSON's round-tripping Python representations. The
script and help dump pin the exact argument-to-public-call mapping.

The earlier 44-case script/JSON are separately preserved as temporary
`...-probes-v1-44.py` / `...-probes-v1-44.json`; the final 48-case run adds four
controls and explicit time-form discount controls. It retains the original
44 constructor cases and their raw errors.

## Public call surface actually inspected

Public signatures here are from the attested installed binding's help strings,
not inferred from an implementation or an unpinned online wrapper.

| Surface | Explicit inputs needed in these probes |
|---|---|
| `ZeroCouponBond` | Settlement days, calendar, face, raw maturity, payment convention, redemption percentage, issue date. Help defaults are Following / 100 / absent issue. |
| `FixedRateBond` | Settlement days, face, schedule, coupon vector, coupon day counter, payment convention, redemption, issue, payment calendar, ex-coupon period/calendar/convention/EOM flag. Help defaults include Following, 100, absent issue/calendar/period, Unadjusted ex convention, false ex EOM. |
| `InterestRate` | Rate, day counter, compounding and frequency; use no default constructor. Time-form `discountFactor(t)` needs no date or clock. |
| `Bond.cleanPrice` / `Bond.dirtyPrice` | Yield, day counter, compounding, frequency, nonempty settlement. |
| `BondFunctions.cleanPrice` | Either the same yield tuple or an `InterestRate`, plus explicit settlement; curve overloads exist but were skipped. |
| `BondFunctions.dirtyPrice` | Help exposes curve/spread overloads only; **no yield/InterestRate overload**. The explicit-yield `Bond.dirtyPrice` supplies this observation. |
| `BondFunctions.accruedAmount` | Bond and explicit settlement. Also checked `Bond.accruedAmount(settlement)`. |
| `BondFunctions.bondYield` | Bond, `BondPrice(amount, Clean/Dirty)`, day counter, compounding, frequency, settlement, accuracy, max iterations, guess. |
| `BondFunctions.duration` | Yield tuple or InterestRate; explicitly name Simple/Macaulay/Modified and settlement. Default type in help is Modified. |
| `BondFunctions.convexity` | Yield tuple or InterestRate, explicit settlement. |
| `Bond.settlementDate` | Explicit evaluation/reference date; this may derive settlement from its supplied date and constructor lag/calendar/issue. |

Public cash-flow inspectors recorded amount/payment date; `as_coupon` then
provided raw accrual start/end, reference start/end, ex date and accrual period.
No engine or curve was attached. Default engine-based `cleanPrice()` /
`dirtyPrice()`, engine-based yield, absent settlement/reference dates,
`Settings` reads/writes and implicit `hasOccurred` calls were skipped. Both
dates of every generated schedule were explicit; null first/penultimate dates
mean absent optional stubs, not an absent effective date.

This establishes that the selected public call surface accepts explicit date
inputs. Black-box observations do not prove an implementation never reads any
hidden global state. No hooks or global-state instrumentation were used.

## Probe census and selected results

The final run completed 48 isolated cases and 781 recorded query entries:
69 public `RuntimeError` entries and 20 explicitly skipped dependent yield
inversions. Three clean-price overloads agree bit-for-bit in all 38 cases
where all three return finite prices. Errors are observations, not local
error contracts or expected results for Rust.

| Group | Cases |
|---|---:|
| Coupon day minus/equal/plus; final maturity minus/equal/plus | 3 + 3 |
| Seven calendar-day ex period: before/equal/after ex and coupon day | 6 |
| Seven business-day ex period: before/equal/after ex | 3 |
| One-month ex period: Following/Preceding/Unadjusted, plus EOM | 4 |
| Negative compounded yields: -0.01, -1.9, -2, -2.1 | 4 |
| Final fixed coupon: all five compounding methods | 5 |
| Raw-vs-adjusted coupon; raw-vs-adjusted zero redemption | 4 + 4 |
| Final zero: simple/compounded and both hybrids | 4 |
| Explicit evaluation date, two-business-day lag, issue floor | 4 |
| One-month ex controls with Actual/365 Fixed yield day counter | 4 |

Common fixed-bond inputs are face 100, redemption 100%, annual coupon .05,
2024-01-15 issue, 2026-01-15 raw maturity, six-month backward schedule,
WeekendsOnly calendar, **Unadjusted raw schedule**, Following payments,
no EOM, and schedule-aware `ActualActual(ISMA,schedule)` coupon/yield day
counter. All arguments differing from these are in each JSON case. Reference
coupons print `2.499999999999991`, rather than exact 2.5: this is a potential
numerical classification against SabiQuant's existing one-round product,
not a reason to change that contract.

### Inclusion and ex-coupon observations

At settlement 2024-07-15, that day's coupon is excluded: accrued=0 and both
prices=100. On July 14 accrued=2.4862637362637274; July 16 accrued=
0.013586956521738358. Equal settlement therefore needs a declared policy,
separate from the preceding day's eligibility.

A positive seven-day period with NullCalendar produces ex date July 8;
WeekendsOnly instead gives July 4. At calendar ex equality July 8 the coupon
is excluded, accrued=-0.09615384615384581 and dirty=99.90507349685214.
At July 7 accrued remains positive. Positive ex periods thus move backward
in these constructors; ex equality is already ex-coupon in this sample.

For raw July 31, 2024 payment, a one-month ex period gives Following July 1,
Preceding June 28, Unadjusted June 30. Following plus ex EOM=true gives June 28.
These are convention-specific observations, not an inferred universal lag rule.

### Raw accrual, payment and schedule-context boundaries

The adjusted fixed group has raw coupon end Sunday 2025-06-15 and Following
payment Monday June 16. At raw June 15 the unpaid coupon remains eligible and
accrued is the full 2.499999999999991. At payment June 16 it is excluded, while
the next raw period has already accrued 0.013661202185799581. Payment equality
therefore need not make total accrued zero when payment lags raw accrual.

A zero bond with raw June 15 maturity and June 16 payment still prices at raw
maturity (99.98647071661046); at payment equality its `dirtyPrice` is zero,
while `cleanPrice` throws a non-tradable error naming raw June 15 maturity.
This wrapper distinction must be classified rather than copied implicitly.

The four original one-month ex cases also expose a separate pricing limit:
their final raw 2026-01-31 payment is adjusted to February 2 beyond the
schedule-aware yield day counter's supplied hull. The exact public error is:

```text
Dates out of range of schedule: date 1: July 31st, 2025, date 2: February 2nd, 2026, first date: January 31st, 2024, last date: January 31st, 2026
```

All four matching controls retain the coupon/schedule/ex context but use
Actual365Fixed **only for the yield day counter**; they price finitely. The
metadata/ex-date observations were valid even when yield pricing failed.
A local convention must state whether discount time follows raw accrual,
adjusted payment, explicit reference periods or another clock.

### Final interval, negative yields and risk

At 2025-10-15 settlement before final 2026-01-15 payment, fixed-bond accrued
is 1.2499999999999956 and the reference fraction is .25:

| Compounding | Dirty price |
|---|---:|
| Simple / SimpleThenCompounded | 101.23456790123457 |
| Compounded / CompoundedThenSimple | 101.24228365658296 |
| Continuous | 101.22672455062283 |

Explicit `InterestRate.discountFactor(.25)` controls reproduce the respective
simple/fractional-compounded/continuous discount behavior. Zero-bond controls
with `t=92/365` likewise distinguish simple 98.75541125541126 from compounded
98.76293704023463. These observations do not choose between the separate
Treasury formula reviewed in [the primary-source report](M8_US_TREASURY_SOURCE_REVIEW.md)
and a generic explicitly compounded yield convention.

At annual semiannual-compounded yield -0.01 the baseline price/inversion are
finite. At -1.9 price is finite but both inversions from guess .05 fail within
100 evaluations; raw text is preserved. At -2 the price is `inf`; at -2.1 it
is `nan`. Risk outputs can also be `nan`. Local finite-domain checks and solver
brackets must be specified independently; reference nonfinite behavior is
not an admissible Rust result merely because it is observed.

At coupon-date settlement July 15, Simple and Macaulay duration both print
1.4637120761451519; Modified prints 1.4280117816050264; convexity prints
2.763432576215753. Macaulay duration with Simple/Continuous/hybrid compounding
throws `compounded rate required` in seven sampled cases. These names/results
alone do not replace independent mathematical definitions or derivative tests.

Explicit settlement-date derivation with issue June 17, 2024, WeekendsOnly
and lag two: supplied evaluation June 12 returns June 17 (issue floor);
June 14/15 return June 18; June 17 returns June 19. No implicit today was used.

## Proposed next reference work after a contract exists

Freeze a new, attested generator and fresh fixture only after date roles,
ex entitlement, quote normalization, allowed yield conventions, cash-flow
arithmetic, risk definitions and bounded solver behavior are adopted. Keep
these exploratory errors and divergences visible. Separate direct-compatible,
locally specified, numerical and unsupported-context classifications.

Use independent exact/rational or high-precision models for price components,
finite-difference risk checks and bracket/endpoint/domain tests; yield
round trips alone can preserve a shared mistake. Extend cases to all supported
frequencies, issue/settlement/maturity/ex equality, payment shifts, irregular
first/final coupons, extreme-but-finite scales and supplied resource limits.
Do not turn this 48-case exploration into a claim of broad coverage.
