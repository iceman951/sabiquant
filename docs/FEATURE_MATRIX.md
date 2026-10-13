# Feature Inventory and Matrix

| Field | Value |
|---|---|
| Milestone | M0 |
| Document status | Draft for project-owner review |
| QuantLib baseline | 1.43 (commit `6b57206e…`), see [QUANTLIB_REFERENCE](QUANTLIB_REFERENCE.md) |
| Implementation status | M1 merged; M2–M4 foundations, M6 numerics and M7 schedules are under local validation and await milestone acceptance. See [§9](#9-implementation-status); every feature not listed there is "Not started" |
| Related | [SCOPE](SCOPE.md), [REQUIREMENTS](REQUIREMENTS.md), [ROADMAP](ROADMAP.md) |

## 1. How to read this matrix

**Status** (planning status; never an implementation claim):

| Status | Meaning |
|---|---|
| Planned | In scope for a numbered milestone, with a specification or a clear path to one |
| Under Research | Behavior or sources not yet settled well enough to specify |
| Deferred | Wanted, but not before the milestone shown (or "later") |
| Out of Scope | Deliberately excluded. Changing that needs a scope change ([SCOPE §5](SCOPE.md#5-scope-change-control)) |
| Requires Decision | Blocked on an owner decision (OQ reference) |

**Priority:** **P0** is required for the foundational release; **P1** is
high (next in line); **P2** is medium; **P3** is low or opportunistic.

**Validation approach codes:**

| Code | Meaning |
|---|---|
| EXH | Exhaustive enumeration over a bounded domain |
| PROP | Property-based tests (deterministic pseudo-random inputs) |
| ORC | Independent oracle (exact rational arithmetic, or high precision computed outside SabiQuant) |
| LIT | Worked examples from literature or primary sources |
| DIFF | Differential comparison with QuantLib 1.43 fixtures |
| BND | Boundary and failure-case tests |
| CONV | Convergence-order tests (numerical methods) |
| SRC | Data checked against a primary published source (e.g., holiday announcements) |

QuantLib references are paths in the QuantLib 1.43 tree. "—" means QuantLib
has no direct equivalent.

The matrix grows by adding rows. Feature IDs are never reused.

**Tracking implementation (from M1).** Implementation status is tracked in
[§9](#9-implementation-status), with the values Not started, In progress,
Implemented, and Verified (or Verified with divergences, as in
[ADR-0003](adr/ADR-0003-quantlib-compatibility.md)). A separate table keeps
the wide planning tables above stable; it lists only features whose
implementation has started. A feature is marked Implemented only when its
milestone's Definition of Done is met
([ROADMAP §2](ROADMAP.md#2-definition-of-done-for-implementation-milestones)).

---

## 2. Core

| ID | Feature | Financial purpose | QuantLib reference | Depends on | Milestone | Priority | Compatibility considerations | Status | Validation |
|---|---|---|---|---|---|---|---|---|---|
| F-CORE-01 | Typed error model | Explicit, inspectable failure for invalid financial inputs | `ql/errors.hpp` (exceptions) | — | M1 | P0 | Behavioral parity means "both reject". Error kinds are SabiQuant-specific | Planned | BND |
| F-CORE-02 | Numeric validation policy | Reject NaN/∞ and out-of-domain values at the boundary | — | F-CORE-01 | M1 | P0 | QuantLib often does not validate. SabiQuant may reject where QuantLib returns NaN | Planned | BND, PROP |
| F-CORE-03 | Canonical identifiers and aliases | Unambiguous convention names in data and text | `name()` methods | — | M1 | P0 | Names need not match QuantLib's strings | Planned | EXH |
| F-CORE-04 | Ambiguous-label rejection | Prevent "30/360" or "Act/Act" being mapped silently | — | F-CORE-03 | M1 | P0 | A SabiQuant safety feature | Planned | EXH |
| F-CORE-05 | Test support: deterministic PRNG and ULP comparison (test-only) | Reproducible property tests | — | — | M1 | P0 | Not public API | Planned | PROP |
| F-CORE-06 | Fixture reader (test-only) | Read reference fixtures with std only | — | F-CORE-01 | M1 | P0 | Format per OQ-13 (decided) | Planned | BND |
| F-CORE-07 | Rounding conventions | Market rounding of rates and amounts | `ql/math/rounding.hpp` | F-CORE-02 | later | P2 | Rounding modes must match exactly | Deferred | LIT, DIFF |
| F-CORE-08 | Money and currency | Amounts with currency, conversions | `ql/money.hpp`, `ql/currency.hpp`, `ql/currencies/` | F-CORE-07 | later | P3 | Exchange-rate chaining semantics | Deferred | DIFF |
| F-CORE-09 | Exact decimal amounts | Settlement-exact cash amounts | — | ADR-0006 | later | P3 | QuantLib has none. [OQ-06](IMPLEMENTATION_READINESS.md#5-decision-register) deferred it; the design is decided under NI-01 before M7/M8 | Deferred | ORC |
| F-CORE-10 | Observer / lazy-object framework | Automatic recalculation | `ql/patterns/` | — | — | — | Replaced by explicit state ([ADR-0011](adr/ADR-0011-explicit-state.md)) | Out of Scope | — |
| F-CORE-11 | Global settings singleton | Process-wide evaluation date | `ql/settings.hpp` | — | — | — | Replaced by explicit context | Out of Scope | — |

## 3. Time: dates, periods, frequencies

| ID | Feature | Financial purpose | QuantLib reference | Depends on | Milestone | Priority | Compatibility considerations | Status | Validation |
|---|---|---|---|---|---|---|---|---|---|
| F-TIME-01 | Date value (proleptic Gregorian) | Represent value, payment and fixing dates | `ql/time/date.hpp` | F-CORE-01 | M2 | P0 | Range is wider than QuantLib's (1901–2199); differential tests only inside it | Planned | EXH, ORC, DIFF |
| F-TIME-02 | Leap-year, month and year lengths | Basis of all day counting | `Date::isLeap` | F-TIME-01 | M2 | P0 | QuantLib's 1900 leap quirk is outside both valid ranges | Planned | EXH |
| F-TIME-03 | Weekday and day of year | Calendar rules | `ql/time/weekday.hpp` | F-TIME-01 | M2 | P0 | QuantLib numbers weekdays from Sunday = 1. Compare names, not numbers | Planned | EXH, DIFF |
| F-TIME-04 | Checked day and week arithmetic | Date offsets | `Date::operator+` | F-TIME-01 | M2 | P0 | Out-of-range results are errors in both | Planned | EXH, BND |
| F-TIME-05 | Month and year arithmetic with clamping | Tenor-based dates | `Date::advance` | F-TIME-01 | M2 | P0 | Clamping semantics identical | Planned | EXH, DIFF |
| F-TIME-06 | Calendar-day end and start of month | Schedule and EOM rules | `Date::endOfMonth`, `isEndOfMonth` | F-TIME-01 | M2 | P0 | Identical | Planned | EXH, DIFF |
| F-TIME-07 | ISO 8601 `YYYY-MM-DD` text | Fixtures, interop, display | — (QuantLib has its own formatters) | F-TIME-01 | M2 | P0 | Not a parity item | Planned | EXH, BND |
| F-TIME-08 | N-th weekday, next and previous weekday | Holiday rules, IMM dates | `Date::nthWeekday`, `nextWeekday` | F-TIME-03 | M2 | P1 | Identical semantics expected | Planned | EXH, DIFF |
| F-TIME-09 | Excel/QuantLib serial conversion | Spreadsheet interop | `Date(serial)` | F-TIME-01 | — | P2 | Must reproduce the 1900 quirk where relevant | Requires Decision ([OQ-12](IMPLEMENTATION_READINESS.md#5-decision-register)) | ORC |
| F-TIME-10 | Period (length + unit) | Tenors | `ql/time/period.hpp` | F-CORE-01 | M2 | P0 | Normalized equality; cross-family ordering differs (OQ-10) | Planned | EXH, DIFF |
| F-TIME-11 | Period arithmetic and normalization | Tenor algebra | `Period` operators | F-TIME-10 | M2 | P0 | Incompatible-unit errors in both | Planned | EXH, BND |
| F-TIME-12 | Frequency enumeration | Coupon and compounding frequency | `ql/time/frequency.hpp` | — | M2 | P0 | No `OtherFrequency`. "No frequency" is an absent value | Planned | EXH |
| F-TIME-13 | Period ↔ Frequency mapping | Schedule construction | `Period(Frequency)`, `Period::frequency()` | F-TIME-10, F-TIME-12 | M2 | P0 | Unmapped periods give no frequency (QuantLib gives `OtherFrequency`) | Planned | EXH, DIFF |
| F-TIME-14 | Period text (`3M`, `1Y`) | Data input | `ql/utilities/dataparsers.hpp` | F-TIME-10 | M2 | P1 | Composite strings Deferred | Planned | EXH, BND |
| F-TIME-15 | IMM, ASX and ECB dates | Futures and maintenance-period dates | `ql/time/imm.hpp`, `asx.hpp`, `ecb.hpp` | F-TIME-08 | later | P2 | ECB dates need a data source | Deferred | LIT, DIFF |
| F-TIME-16 | Time grids | Discretisation for numerical methods | `ql/timegrid.hpp` | F-TIME-01 | M12+ | P2 | — | Deferred | PROP |
| F-TIME-17 | Intraday or high-resolution dates | Intraday timestamps | `QL_HIGH_RESOLUTION_DATE` | — | — | — | — | Out of Scope | — |
| F-TIME-18 | System-clock "today" | Implicit evaluation date | `Date::todaysDate()` | — | — | — | Excluded by design ([ADR-0011](adr/ADR-0011-explicit-state.md)) | Out of Scope | — |

## 4. Calendars and business-day conventions

| ID | Feature | Financial purpose | QuantLib reference | Depends on | Milestone | Priority | Compatibility considerations | Status | Validation |
|---|---|---|---|---|---|---|---|---|---|
| F-CAL-01 | Calendar abstraction (business-day test, holidays, weekends) | Market open and closed days | `ql/time/calendar.hpp` | F-TIME-01 | M3 | P0 | Immutable values. QuantLib mutates a shared state | Planned | EXH, DIFF |
| F-CAL-02 | Null calendar | Theoretical calculations | `calendars/nullcalendar.hpp` | F-CAL-01 | M3 | P0 | Identical | Planned | EXH |
| F-CAL-03 | Weekends-only calendar | Generic business days | `calendars/weekendsonly.hpp` | F-CAL-01 | M3 | P0 | Identical | Planned | EXH, DIFF |
| F-CAL-04 | Custom (bespoke) calendar | User-defined markets | `calendars/bespokecalendar.hpp` | F-CAL-01 | M3 | P0 | Changes produce new values | Planned | EXH, PROP |
| F-CAL-05 | Joint calendar (join holidays / join business days) | Multi-market settlement | `calendars/jointcalendar.hpp` | F-CAL-01 | M3 | P0 | Same two rules | Planned | EXH, DIFF |
| F-CAL-06 | Adjustment | Roll dates that are not business days | `Calendar::adjust` | F-BDC-* | M3 | P0 | Identical, except pathological calendars give errors | Planned | EXH, DIFF |
| F-CAL-07 | Advancement (business days; periods; EOM rule) | Spot lags, tenors, payment dates | `Calendar::advance` | F-CAL-06, F-TIME-05 | M3 | P0 | Same semantics, plus an explicit add-business-days operation | Planned | EXH, DIFF |
| F-CAL-08 | Business-day counting with inclusion options | Accrual and Business/252 | `Calendar::businessDaysBetween` | F-CAL-01 | M3 | P0 | Same sign and inclusion semantics | Planned | EXH, DIFF |
| F-CAL-09 | Holiday and business-day lists | Reporting, verification | `holidayList`, `businessDayList` | F-CAL-01 | M3 | P1 | Identical | Planned | EXH, DIFF |
| F-CAL-10 | Algorithmic Western Easter | Easter-based holidays | Precomputed tables, 1901–2199 | F-TIME-01 | M3 | P0 | Must match QuantLib's table for 1901–2199 | Planned | EXH, LIT, DIFF |
| F-CAL-11 | Orthodox Easter | Orthodox-calendar markets | Orthodox table in `calendar.cpp` | F-CAL-10 | later | P3 | — | Deferred | LIT, DIFF |
| F-CAL-12 | TARGET calendar (pilot) | Euro payments | `calendars/target.hpp` | F-CAL-10 | M3 | P1 | Guarded production validity from 1999-01-04; strict verified coverage with explicit projection | Planned | SRC, DIFF |
| F-CAL-13 | Calendar provenance and coverage metadata | Trustworthy holiday data | — | F-CAL-01 | M3 | P0 | A SabiQuant feature | Planned | Review |
| F-CAL-14 | Other market calendars (48 headers in QuantLib) | Country and exchange markets | `calendars/*.hpp` | F-CAL-13 | later | P2 | Each needs sources and a maintainer | Deferred | SRC, DIFF |
| F-CAL-15 | Lunar-based holidays (Islamic, Chinese, Thai and others) | Markets with lunar holidays | `calendars/islamicholidays.hpp`, country calendars | F-CAL-14 | later | P3 | Usually tables of announced dates | Under Research | SRC |
| F-CAL-16 | Weekend definitions that change over time | Jurisdictions that changed weekends | — | F-CAL-01 | later | P3 | — | Under Research | SRC |
| F-BDC-01 | Following, Preceding, Unadjusted | Core date rolling | `businessdayconvention.hpp` | F-CAL-01 | M3 | P0 | Identical | Planned | EXH, DIFF |
| F-BDC-02 | Modified Following, Modified Preceding | Same-month rolling | same | F-BDC-01 | M3 | P0 | Pathological calendars give errors (QuantLib returns another month) | Planned | EXH, DIFF |
| F-BDC-03 | Half-Month Modified Following | Some money-market conventions | same | F-BDC-01 | M3 | P1 | Identical | Planned | EXH, DIFF |
| F-BDC-04 | Nearest | Some fixing conventions | same | F-BDC-01 | M3 | P1 | Ties go to Following in both | Planned | EXH, DIFF |
| F-BDC-05 | FRN (Eurodollar) convention | FRN schedules | — | Schedules | later | P3 | A schedule-level rule | Under Research | LIT |

## 5. Day-count conventions

Detailed specifications: [REQUIREMENTS §9](REQUIREMENTS.md#9-fs-dcc--day-count-conventions).

| ID | Feature | Financial purpose | QuantLib reference | Depends on | Milestone | Priority | Compatibility considerations | Status | Validation |
|---|---|---|---|---|---|---|---|---|---|
| F-DCC-01 | Actual/360 | Money markets | `daycounters/actual360.hpp` | F-TIME-01 | M3 | P0 | Bitwise-identical expected | Planned | EXH, ORC, DIFF |
| F-DCC-02 | Actual/365 (Fixed) | Money markets, GBP and others | `actual365fixed.hpp` (Standard) | F-TIME-01 | M3 | P0 | Bare "Actual/365" rejected as ambiguous | Planned | EXH, ORC, DIFF |
| F-DCC-03 | Actual/Actual (ISDA) | Swaps | `actualactual.hpp` (ISDA) | F-TIME-02 | M3 | P0 | ULP tolerance (operation order); QR-01 | Planned | EXH, ORC, LIT, DIFF |
| F-DCC-04 | Actual/Actual (ICMA) | Bonds | `actualactual.hpp` (ISMA, with or without schedule) | F-TIME-01 | M3 | P0 | Explicit determination periods/frequency required; no inference | Planned | LIT, ORC, DIFF |
| F-DCC-05 | Actual/Actual (AFB) | French market | `actualactual.hpp` (AFB) | F-TIME-02 | M3 | P1 | Counting-back edge case (OQ-16) | Requires Decision ([OQ-16](IMPLEMENTATION_READINESS.md#5-decision-register)) | EXH, LIT, DIFF |
| F-DCC-06 | 30/360 Bond Basis | Bonds, swaps | `thirty360.hpp` (BondBasis / ISMA) | F-TIME-01 | M3 | P0 | Bitwise-identical expected; reversed-date semantics (OQ-14) | Planned | EXH, ORC, DIFF |
| F-DCC-07 | 30E/360 (Eurobond) | Eurobonds | `thirty360.hpp` (European) | F-TIME-01 | M3 | P0 | Bitwise-identical expected | Planned | EXH, ORC, DIFF |
| F-DCC-08 | 30E/360 (ISDA) | Swaps (German convention) | `thirty360.hpp` (ISDA / German) | F-TIME-02 | M3 | P0 | Termination date is a required input | Planned | EXH, ORC, DIFF |
| F-DCC-09 | 30/360 (US) | US agencies and corporates | `thirty360.hpp` (USA) | F-TIME-02 | M3 | P0 | February rule conditioning (OQ-15) | Requires Decision ([OQ-15](IMPLEMENTATION_READINESS.md#5-decision-register)) | EXH, ORC, DIFF |
| F-DCC-10 | Business/252 | Brazilian market | `business252.hpp` | F-CAL-08 | M3 | P1 | No default calendar; no global cache | Planned | EXH, DIFF |
| F-DCC-11 | Actual/360 (include last day) | Some money-market products | `actual360.hpp` (`includeLastDay`) | F-DCC-01 | later | P2 | Separate convention | Deferred | EXH, DIFF |
| F-DCC-12 | Actual/365 (No Leap) | Excludes 29 February | `actual365fixed.hpp` (NoLeap) | F-TIME-02 | later | P2 | Primary source needed | Under Research | LIT, DIFF |
| F-DCC-13 | Actual/365 (Canadian Bond) | Canadian bonds | `actual365fixed.hpp` (Canadian) | F-TIME-01 | later | P2 | Needs a reference period | Under Research | LIT, DIFF |
| F-DCC-14 | 30/360 Italian | Italian market | `thirty360.hpp` (Italian) | F-TIME-02 | later | P3 | Primary source needed | Under Research | LIT, DIFF |
| F-DCC-15 | 30/360 NASD | US (legacy) | `thirty360.hpp` (NASD) | F-DCC-06 | later | P3 | Equivalent to Bond Basis algebraically (QR-05) | Under Research | EXH, DIFF |
| F-DCC-16 | Actual/364, Actual/365.25, Actual/366, 30/365 | Niche | `actual364.hpp`, `actual36525.hpp`, `actual366.hpp`, `thirty365.hpp` | F-TIME-01 | later | P3 | Simple | Deferred | EXH, DIFF |
| F-DCC-17 | 1/1 | ISDA "1/1" | `one.hpp` | — | later | P3 | QuantLib returns 1 for `S = E` | Deferred | BND, DIFF |
| F-DCC-18 | Simple day counter | QuantLib test helper | `simpledaycounter.hpp` | — | — | — | Not a market convention | Out of Scope | — |
| F-DCC-19 | Year fraction to date (inverse) | Converting times back to dates | `yearfractiontodate.hpp` | F-DCC-01..10 | later | P2 | Inverses are not unique for 30/360 | Deferred | PROP, DIFF |

## 6. Interest rates, compounding and discounting

| ID | Feature | Financial purpose | QuantLib reference | Depends on | Milestone | Priority | Compatibility considerations | Status | Validation |
|---|---|---|---|---|---|---|---|---|---|
| F-RATE-01 | Compounding methods (Simple, Compounded, Continuous) | Rate quotation | `ql/compounding.hpp` | — | M4 | P0 | Same definitions | Planned | ORC, DIFF |
| F-RATE-02 | Hybrid compoundings (Simple then Compounded; Compounded then Simple) | Money-market and bond quotes | `ql/compounding.hpp` | F-RATE-01 | M4 | P1 | Boundary `t = 1/m`; local decision OQ-23 recorded | Planned | ORC, DIFF, BND |
| F-RATE-03 | Interest-rate quote value | A rate together with its conventions | `ql/interestrate.hpp` | F-DCC-*, F-TIME-12 | M4 | P0 | Validates more than QuantLib (domain checks) | Planned | BND, DIFF |
| F-RATE-04 | Compound and discount factors (by time and by dates) | Growth and discounting | `InterestRate::compoundFactor`, `discountFactor` | F-RATE-03 | M4 | P0 | Stable formulations differ in the last bits | Planned | ORC, DIFF, PROP |
| F-RATE-05 | Implied rate | Rate from a price or discount factor | `InterestRate::impliedRate` | F-RATE-04 | M4 | P0 | Same `t = 0`, `C = 1` rule | Planned | ORC, DIFF, PROP |
| F-RATE-06 | Equivalent rate | Convert between conventions | `InterestRate::equivalentRate` | F-RATE-05 | M4 | P0 | QuantLib converts through `C(t)`; tolerance scaled for cancellation | Planned | ORC, DIFF, PROP |
| F-RATE-07 | Negative-rate support within domains | Negative-rate markets | — | F-RATE-03 | M4 | P0 | QuantLib does not reject `r ≤ −m` (it returns NaN) | Planned | BND, PROP |
| F-RATE-08 | Period discount factor | Discount over one period | `InterestRate::discountFactor(d1, d2)` | F-RATE-04 | M4 | P0 | Identical definition | Planned | ORC, DIFF |
| F-RATE-09 | Present value of a single amount | Basic valuation | — | F-RATE-08 | M4 | P1 | — | Planned | ORC |
| F-RATE-10 | Basis-point and percent helpers | Explicit unit conversion | — | — | M4 | P2 | — | Planned | BND |

## 7. Future domains (grouped)

These rows are coarse on purpose. Each group is broken into detailed rows
when its milestone is planned. Milestone numbers are provisional
([ROADMAP §4](ROADMAP.md#4-future-milestones-provisional)).

| ID | Feature group | Financial purpose | QuantLib reference | Depends on | Milestone | Priority | Compatibility considerations | Status | Validation |
|---|---|---|---|---|---|---|---|---|---|
| F-MATH-01 | 1-D root finding (bisection, Brent, safeguarded Newton) | Yields, implied volatility, bootstrapping | `ql/math/solvers1d/` | F-CORE-02 | M6 | P1 | Converged values agree within solver tolerance | Deferred | CONV, ORC |
| F-MATH-02 | 1-D interpolation (linear, log-linear, cubic and monotone variants) | Curves and surfaces | `ql/math/interpolations/` | F-CORE-02 | M6 | P1 | Spline boundary conditions | Deferred | ORC, DIFF |
| F-MATH-03 | Normal distribution (pdf, cdf, inverse cdf) | Black-type formulas | `ql/math/distributions/` | F-CORE-02 | M6 | P1 | std has no stable `erf` (it is nightly-only) | Deferred | ORC, DIFF |
| F-MATH-04 | Numerical integration | Pricing integrals | `ql/math/integrals/` | F-CORE-02 | M12 | P2 | — | Deferred | CONV, ORC |
| F-MATH-05 | Dense linear algebra (Cholesky, LU, QR, SVD, symmetric eigen) | Calibration, correlation, splines | `ql/math/matrixutilities/` | F-CORE-02 | M12 | P2 | Built in-house (std only) | Deferred | ORC, PROP |
| F-MATH-06 | Optimization (Levenberg–Marquardt, simplex, BFGS, differential evolution) | Calibration | `ql/math/optimization/` | F-MATH-05 | M12 | P2 | Results depend on the algorithm path | Deferred | CONV |
| F-MATH-07 | Random numbers (Mersenne Twister, Sobol, Halton, Box–Muller, …) | Monte Carlo | `ql/math/randomnumbers/` | F-CORE-02 | M13 | P2 | Bit-identical sequences possible for MT19937; Sobol direction-number licensing | Deferred | ORC, DIFF |
| F-MATH-08 | Statistics (general, incremental, risk) | Monte Carlo estimates, VaR | `ql/math/statistics/` | F-CORE-02 | M13 | P2 | — | Deferred | ORC |
| F-SCHED-01 | Schedule generation (Forward, Backward, Zero, ThirdWednesday, ThirdWednesdayInclusive, Twentieth, TwentiethIMM, OldCDS, CDS, CDS2015) | Coupon dates | `ql/time/schedule.hpp`, `dategenerationrule.hpp` | F-CAL-07 | M7 | P1 | Stubs and EOM rules are the hard parts | Deferred | EXH, DIFF |
| F-CF-01 | Simple cash flows, fixed-rate coupons, legs | Cash-flow modelling | `ql/cashflows/` | F-SCHED-01, F-RATE-* | M7 | P1 | Accrual and payment-date conventions | Deferred | DIFF |
| F-CF-02 | Cash-flow analytics (NPV, BPS, yield, duration, convexity) | Fixed-income analytics | `ql/cashflows/cashflows.hpp` | F-CF-01, F-MATH-01 | M7–M8 | P1 | Settlement and inclusion flags are explicit | Deferred | DIFF, ORC |
| F-BOND-01 | Zero-coupon and fixed-rate bonds (clean and dirty price, accrued interest, yield) | Bond pricing | `ql/instruments/bonds/` | F-CF-02 | M8 | P1 | Ex-coupon and settlement conventions | Deferred | DIFF, LIT |
| F-BOND-02 | Floating, amortizing, callable, convertible and inflation bonds | Advanced fixed income | `ql/instruments/bonds/` | F-BOND-01, F-IDX-01 | later | P3 | — | Deferred | DIFF |
| F-TS-01 | Yield term structures (flat forward, interpolated discount/zero/forward curves) | Discounting curves | `ql/termstructures/yield/` | F-MATH-02, F-RATE-* | M9 | P1 | Reference date explicit; extrapolation policy | Deferred | DIFF, ORC |
| F-IDX-01 | Interest-rate indexes (IBOR, overnight) with explicit fixing histories | Floating legs | `ql/indexes/` | F-TS-01 | M10 | P1 | No global fixing store | Deferred | DIFF |
| F-CF-03 | Floating and overnight-compounded coupons | Floating-rate cash flows | `iborcoupon.hpp`, `overnightindexedcoupon.hpp` | F-IDX-01 | M10 | P1 | Lookback and lockout conventions | Deferred | DIFF |
| F-TS-02 | Piecewise curve bootstrapping (deposits, FRAs, futures, swaps, OIS) | Curve construction | `ql/termstructures/yield/` (rate helpers) | F-TS-01, F-MATH-01 | M10 | P1 | Bootstrapping accuracy settings | Deferred | DIFF |
| F-SWAP-01 | Vanilla swaps, OIS, FRAs with discounting engines | Linear rates products | `ql/instruments/`, `ql/pricingengines/swap/` | F-CF-03, F-TS-02 | M10 | P1 | — | Deferred | DIFF |
| F-OPT-01 | Black–Scholes–Merton, Black-76, Bachelier closed forms; Greeks; implied volatility | Vanilla options | `ql/pricingengines/vanilla/`, `blackformula.hpp` | F-MATH-03, F-MATH-01 | M11 | P1 | — | Deferred | ORC, DIFF |
| F-PROC-01 | Stochastic processes (GBM / Black–Scholes, Ornstein–Uhlenbeck, Hull–White, Heston, …) | Model dynamics | `ql/processes/` | F-TS-01 | M13 | P2 | — | Deferred | CONV, DIFF |
| F-MC-01 | Monte Carlo framework (paths, Brownian bridge, early exercise via least squares) | Simulation pricing | `ql/methods/montecarlo/` | F-MATH-07, F-PROC-01 | M13 | P2 | Seeds and determinism across threads | Deferred | CONV, ORC |
| F-FD-01 | Finite differences and lattices (binomial and trinomial trees, FD schemes) | PDE and tree pricing | `ql/methods/finitedifferences/`, `lattices/` | F-MATH-05, F-PROC-01 | M14 | P2 | Grid conventions | Deferred | CONV, DIFF |
| F-MODEL-01 | Short-rate models (Vasicek, Hull–White, Black–Karasinski, CIR, G2++) and calibration | Rates derivatives | `ql/models/shortrate/` | F-MATH-06, F-FD-01 | M15 | P2 | Calibration paths differ | Deferred | DIFF, CONV |
| F-VOL-01 | Volatility term structures; caps and floors; swaptions; SABR | Rates options | `ql/termstructures/volatility/` | F-TS-01, F-OPT-01 | M15 | P2 | — | Deferred | DIFF |
| F-RISK-01 | Bump-and-reprice sensitivities, key-rate durations, scenarios | Risk analytics | Sensitivity outputs in engines | F-TS-01, F-SWAP-01 | M16 | P2 | Not a QuantLib domain as such | Deferred | ORC |
| F-RISK-02 | Historical and parametric VaR and expected shortfall | Portfolio risk statistics | `ql/math/statistics/riskstatistics.hpp` | F-MATH-08 | M16 | P3 | — | Deferred | ORC |
| F-CREDIT-01 | Default-probability curves, CDS | Credit | `ql/termstructures/credit/`, `instruments/creditdefaultswap.hpp` | F-TS-01 | later | P3 | — | Under Research | DIFF |
| F-INFL-01 | Inflation indexes, curves, swaps, CPI bonds | Inflation | `ql/indexes/inflation/`, `ql/termstructures/inflation/` | F-TS-01 | later | P3 | Index interpolation conventions | Under Research | DIFF |
| F-EXOTIC-01 | Barrier, Asian, lookback, basket, cliquet and other exotic engines | Exotic options | `ql/pricingengines/*` | F-MC-01, F-FD-01 | later | P3 | — | Under Research | DIFF, CONV |
| F-MODEL-02 | Stochastic-volatility and market models (Heston, Bates, LMM) | Advanced models | `ql/models/equity/`, `marketmodels/` | F-MODEL-01 | later | P3 | — | Under Research | DIFF, CONV |
| F-AD-01 | Automatic or algorithmic differentiation | Fast sensitivities | — | ADR-0006 | — | — | Conflicts with concrete `f64` (ADR-0006 reconsideration trigger) | Requires Decision | — |
| F-EXP-01 | QuantLib `experimental/` content | Assorted research features | `ql/experimental/` | — | — | — | Assessed case by case later | Out of Scope | — |
| F-LEG-01 | QuantLib `legacy/` content | Superseded features | `ql/legacy/` | — | — | — | — | Out of Scope | — |
| F-BIND-01 | Language bindings (Python, C, WebAssembly) | Interoperability | QuantLib-SWIG (separate project) | — | — | — | Needs an ADR | Out of Scope | — |

## 8. Summary counts (foundational rows, §2–§6)

| Status | Count |
|---|---:|
| Planned | 51 |
| Requires Decision | 5 |
| Under Research | 7 |
| Deferred | 11 |
| Out of Scope | 5 |
| **Total** | **79** |

These counts cover §2–§6 only and are kept up to date when rows change.

## 9. Implementation status

Features whose implementation has started. Every other feature is Not
started.

| ID | Implementation | Notes |
|---|---|---|
| F-CORE-01 | Implemented | M1 merged in PR #2; M2 adds Date/Period errors and corrects typed source propagation, under review |
| F-CORE-02 | Implemented | Internal validation helpers for NP-01 and NP-02; M1 CI passed |
| F-CORE-03 | Implemented | Identifier tables with canonical identifiers and aliases; M2 adds time enumeration tables |
| F-CORE-04 | Implemented | Ambiguous labels are listed in the tables and rejected with their candidates; M1 CI passed |
| F-CORE-05 | Implemented | SplitMix64 generator, property runner, ULP and tolerance comparison (test-only); M1 CI passed |
| F-CORE-06 | Implemented | Fixture reader for schema 1 (test-only); M2 reads seven committed reference sets |
| F-TIME-01…08 | In progress | Local implementation and exhaustive/independent/differential validation pass; primary-platform CI and milestone acceptance remain pending ([M2_VALIDATION](M2_VALIDATION.md)) |
| F-TIME-10…14 | In progress | Local tenor/frequency APIs, laws, overflow and differential tests pass; documented intentional divergences; CI/review pending ([M2_VALIDATION](M2_VALIDATION.md)) |
| F-CAL-01…09 | In progress | Core calendars, adjustment/advancement/counting/lists pass local oracle/reference tests; no market provenance claim ([M3_CORE_VALIDATION](M3_CORE_VALIDATION.md)) |
| F-BDC-01…04 | In progress | All seven specified conventions pass local tests; empty-month/search bounds are deliberate reference divergences |
| F-DCC-01…03; F-DCC-07 | In progress | Actual/360, Actual/365 Fixed, Actual/Actual ISDA and European 30E/360 pass independent rational and actual reference tests; additional source-resolved variants are listed below |
| F-CAL-10, F-CAL-12…13 | In progress | All 9,999 computus years; guarded TARGET and explicit provenance/disjoint coverage pass local checks; maintenance appointment and CI/review remain ([M3_EXTENSION_VALIDATION](M3_EXTENSION_VALIDATION.md)) |
| F-DCC-04, F-DCC-06, F-DCC-08, F-DCC-10 | In progress | Explicit ICMA, Bond Basis, termination-sensitive ISDA and Business/252 pass local oracle/reference checks, with precise intentional signed/zero divergences ([M3_EXTENSION_VALIDATION](M3_EXTENSION_VALIDATION.md)) |
| F-RATE-01…10 | In progress | Five methods, owned accrual/frequency, factors, implied/equivalent rates, date conversion, present values and explicit unit helpers implemented locally; independent oracle/reference/boundary validation and CI/review tracked in [M4_NUMERICAL_VALIDATION](M4_NUMERICAL_VALIDATION.md) |
| F-MATH-01 | In progress | Three bracketed solvers with explicit shared callback cap, finite diagnostics and source-pinned safeguards; complete local independent tolerance/root/convergence evidence ([M6 validation](M6_NUMERICAL_VALIDATION.md)); native CI/acceptance open |
| F-MATH-02 | In progress | Linear, LogLinear, NaturalCubic and explicitly pinned PCHIP Hermite; 99,072 independent exact/two-precision rows plus shape/extreme tests; numerical support and extrapolation remain explicit ([contract](M6_INTERPOLATION_RESEARCH.md)) |
| F-MATH-03 | In progress | Coefficient-free normal probabilities/log tails and centered/log inverse with explicit location/scale; 19,946 independent numerical/guard rows and measured accuracy ([M6 validation](M6_NUMERICAL_VALIDATION.md)); native CI/acceptance open |
| F-SCHED-01 | In progress | Anchored Forward/Backward/Zero, automatic/explicit stubs, calendar EOM, guarded adjustment and minimal explicit-frequency ICMA references ([M7 readiness](M7_LOCAL_READINESS.md), [reference contract](M7_ICMA_REFERENCE_CONTRACT.md)); specialized seven rules and full M7/cash-flow acceptance remain pending |

Compensated and exact binary64 summation are also local M6 work under
FR-MATH-04/NP-09. Their 105,307 exact Fraction rows and reduction-domain tests
are recorded in [M6 validation](M6_NUMERICAL_VALIDATION.md). Local numerical
evidence does not close native platform, formal milestone or release gates.
