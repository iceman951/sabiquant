# SabiQuant — Requirements and Functional Specifications

| Field | Value |
|---|---|
| Milestone | M0 |
| Document status | Draft for project-owner review |
| Scope | Owner constraints, non-functional requirements, and functional specifications for the foundational domains (M1–M4) |
| Related | [SCOPE](SCOPE.md), [DOMAIN_MODEL](DOMAIN_MODEL.md), [NUMERICAL_CORRECTNESS](NUMERICAL_CORRECTNESS.md), [TESTING_STRATEGY](TESTING_STRATEGY.md), [FEATURE_MATRIX](FEATURE_MATRIX.md), [Decision register](IMPLEMENTATION_READINESS.md#5-decision-register) |

> These specifications describe **behavior**, not code. They contain no Rust
> source and no API signatures. Concrete APIs are designed during each
> implementation milestone and must satisfy these specifications.

---

## 0. Conventions used in this document

**Requirement keywords.** MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are used
as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) and
[RFC 8174](https://www.rfc-editor.org/rfc/rfc8174). They carry that meaning
only when written in capitals.

**Identifiers.**

| Prefix | Meaning | Defined in |
|---|---|---|
| `C-nn` | Owner constraint (approved) | §1 |
| `NFR-nn` | Non-functional requirement (proposed) | §2 |
| `FR-<AREA>-nn` | Functional requirement (proposed) | §4–§11 |
| `INV-<AREA>-n` | Domain invariant | here and [NUMERICAL_CORRECTNESS §8](NUMERICAL_CORRECTNESS.md#8-domain-invariants) |
| `TS-<AREA>-nn` | Test scenario | [TESTING_STRATEGY §4](TESTING_STRATEGY.md#4-foundational-test-scenario-catalogue) |
| `OQ-nn` | Open question or decision | [IMPLEMENTATION_READINESS §5](IMPLEMENTATION_READINESS.md#5-decision-register) |
| `F-<AREA>-nn` | Inventory feature | [FEATURE_MATRIX](FEATURE_MATRIX.md) |

**Notation.**

- `S` and `E` are the start and end dates of an interval. Their components are
  `(Y1, M1, D1)` and `(Y2, M2, D2)`.
- `E − S` is the signed number of actual calendar days from `S` to `E`.
- Intervals are half-open, `[S, E)`: `S` is included and `E` is excluded,
  unless stated otherwise.
- `t` is a year fraction, `r` is an annual rate as a decimal, and `m` is a
  compounding frequency in periods per year.
- `B(Y)` is 366 if `Y` is a leap year and 365 otherwise.

**Source status.** Several market conventions are defined in licensed
publications, for example the ISDA Definitions. Where SabiQuant could not
consult the licensed text, the specification cites secondary or public sources
and says so. Each such item is tracked as a research risk
([R-02](IMPLEMENTATION_READINESS.md#6-risk-register)).

---

## 1. Owner constraints

These constraints come from the owner's M0 brief and are approved.

| ID | Constraint |
|---|---|
| C-01 | SabiQuant is implemented natively in Rust with idiomatic abstractions. No FFI, no QuantLib bindings, no C++-to-Rust translation. |
| C-02 | Rust Standard Library only. No third-party crates of any kind: runtime, dev, build, or procedural macro. |
| C-03 | Independent implementation. QuantLib source must not be copied or translated, and large sections of QuantLib documentation must not be reproduced. |
| C-04 | QuantLib is an external reference only. It is used to identify features, study behavior, compare outputs, find edge cases, and validate conventions. |
| C-05 | Priority order: correctness → reliability → maintainability → performance. |
| C-06 | M0 is documentation only. |
| C-07 | Implementation starts only after all mandatory readiness gates pass **and** the owner grants explicit permission ([IMPLEMENTATION_READINESS](IMPLEMENTATION_READINESS.md)). |
| C-08 | SabiQuant claims QuantLib compatibility for a feature only after that behavior has been independently verified ([ADR-0003](adr/ADR-0003-quantlib-compatibility.md)). |

---

## 2. Non-functional requirements

| ID | Requirement | Verification |
|---|---|---|
| NFR-01 | **Specified behavior.** Every public financial computation MUST trace to a specification in this document (or a successor) and to at least one cited source. | Review checklist ([CONTRIBUTING_GUIDELINES §5](CONTRIBUTING_GUIDELINES.md#5-financial-correctness-requirements-for-contributions)) |
| NFR-02 | **No panics on invalid input.** Public operations MUST NOT panic for any argument value. Invalid input produces an error value. Panics signal internal defects only. | Boundary and fuzz-style property tests |
| NFR-03 | **Determinism.** With the same inputs, binary, and platform, results MUST be bitwise identical across runs, threads, and call orders. Results MUST NOT depend on the system clock, time zone, locale, environment variables, or global state. | Repeated-run and concurrent-run tests |
| NFR-04 | **Cross-platform agreement.** Discrete outputs MUST be identical on all supported platforms. Floating-point outputs MUST agree within the tolerances in [NUMERICAL_CORRECTNESS §7](NUMERICAL_CORRECTNESS.md#7-tolerance-policy). | Cross-platform test matrix ([TESTING_STRATEGY §11](TESTING_STRATEGY.md#11-cross-platform-testing)) |
| NFR-05 | **Thread safety.** Every public type MUST be safe to send between threads and to share between threads, unless documented otherwise. No interior mutability may be observable. | Compile-time assertions in tests |
| NFR-06 | **No unsafe code** (proposed). Any future use needs an ADR. | Crate-level lint policy |
| NFR-07 | **Explicit state.** No global mutable configuration. Evaluation dates and market data are explicit inputs ([ADR-0011](adr/ADR-0011-explicit-state.md)). | Code review; architecture tests |
| NFR-08 | **Documentation.** Every public item MUST document its definition and formula, units, preconditions, errors, sources, and how it corresponds to and differs from QuantLib. | Lint policy (missing docs is an error); review |
| NFR-09 | **Standard-library-only build.** The package manifest MUST declare no dependencies of any kind. | Manifest review in every PR |
| NFR-10 | **API stability.** Semantic versioning; enums and structs that may grow are non-exhaustive; a documented MSRV ([ADR-0009](adr/ADR-0009-api-stability.md)). | Release checklist |
| NFR-11 | **Complexity.** No pathological complexity. Date ↔ civil conversions are O(1). Year fractions are O(1) except where a convention needs a calendar. Business-day adjustment is proportional to the run of consecutive non-business days. | Code review; complexity notes in docs |
| NFR-12 | **Provenance.** Every reference value used in tests MUST record its source, version, and method. | Fixture schema ([TESTING_STRATEGY §7.3](TESTING_STRATEGY.md#73-reference-fixture-specification)) |
| NFR-13 | **No I/O in the library.** No file, network, or environment access in H1. Tests may read committed fixtures. | Code review |
| NFR-14 | **Units.** Rates are annual decimal fractions (0.05 = 5%). Times are year fractions. There is no implicit percent scaling. | Doc review; tests |
| NFR-15 | **Inspectable errors.** Errors expose a programmatic kind and structured context, not only text. | Tests that match error kinds |

---

## 3. Specification template

Each functional specification below covers the same fields: purpose;
financial definitions; inputs; outputs; invariants; preconditions; failure
cases; boundary conditions; dependencies; expected behavior (numbered
requirements); validation criteria; and explicit exclusions.

---

## 4. FS-CORE — Core types

**Purpose.** Defines the concepts every domain uses: how values are
represented, validated, and identified, and how failures are reported.

**Financial definitions.**

| Term | Definition in SabiQuant |
|---|---|
| Rate | Annual interest rate as a decimal fraction. 0.05 means 5% per annum. Never a percentage. |
| Year fraction (time) | Dimensionless length of an interval in years, measured by a stated day-count convention |
| Compound factor | Value at the end of an interval of one unit invested at its start |
| Discount factor | Value at the start of an interval of one unit paid at its end. It is the reciprocal of the compound factor. |
| Spread | Additive difference between two rates, in rate units |
| Basis point | 0.0001 in rate units |

**Proposed representations.** These are conceptual. Final types are settled in
the design review of each milestone.

| Concept | Representation | Rationale | Record |
|---|---|---|---|
| Calendar date | Dedicated immutable date value | Exact arithmetic, total order, no invalid states | [ADR-0005](adr/ADR-0005-date-representation.md) |
| Weekday, month | Closed enumerations | Exhaustive matching; no confusion with integers | FS-DATE |
| Time unit | Closed enumeration: Days, Weeks, Months, Years | Intraday units excluded | FS-PER |
| Period | Signed integer length plus time unit | Tenors such as `3M` or `-1Y` | FS-PER |
| Frequency | Closed enumeration of named frequencies | QuantLib's "other frequency" excluded | FS-PER |
| Business-day convention | Closed enumeration | Finite, standardised set | FS-BDC |
| Day-count convention | Enumeration whose variants carry their required parameters | A required parameter (for example a termination date) cannot be forgotten | FS-DCC |
| Compounding | Closed enumeration; periodic variants carry their frequency | Invalid combinations rejected when the value is built | FS-RATE |
| Calendar | Immutable value made of rules and data | No shared mutable state | [ADR-0005](adr/ADR-0005-date-representation.md), [ADR-0011](adr/ADR-0011-explicit-state.md) |
| Interest-rate quote | Immutable value: rate, day count, compounding (with frequency) | A rate means nothing without its conventions | FS-RATE |
| Real numbers | IEEE 754 binary64 (`f64`) | Enough precision for analytics; available in std | [ADR-0006](adr/ADR-0006-numeric-policy.md) |
| Errors | One error enumeration per domain plus a crate-level error | Typed, inspectable failures | [ADR-0007](adr/ADR-0007-error-handling.md) |
| Identifiers | Canonical text names for conventions and calendars | Parsing, display, fixtures | FR-CORE-05, FR-CORE-06 |

Whether scalar quantities get wrapper types (year fraction, discount factor) is
open question [OQ-24](IMPLEMENTATION_READINESS.md#5-decision-register). The
current proposal is plain `f64` validated at the API boundary. The types that
carry conventions (rate quote, day-count convention) already hold the
information that is most often confused, and scalar wrappers add API surface
with no benefit shown so far ([DOMAIN_MODEL §6](DOMAIN_MODEL.md#6-typing-strategy-and-misuse-prevention)).

**Inputs and outputs.** Not applicable; this specification is cross-cutting.

**Invariants.**

- INV-CORE-1: Every value of a public type satisfies its documented invariants
  from construction onwards. There are no null, unset, or sentinel states.
  Absence is expressed with an optional value.
- INV-CORE-2: Public values are immutable after construction, and no interior
  mutability is observable.
- INV-CORE-3: Public types can be sent and shared between threads.

**Preconditions.** Real-valued inputs are finite unless a function documents
otherwise.

**Failure cases.**

| Failure | Required behavior |
|---|---|
| Non-finite real input (NaN, +∞, −∞) | Error naming the parameter |
| Value outside its documented domain | Error naming the parameter, the value, and the domain |
| Unknown identifier text | Error carrying the unrecognised text |
| Ambiguous identifier text | Error listing the candidate conventions |
| Internal invariant violation (defect) | May panic. Treated as a bug, never as API behavior |

**Boundary conditions.** Negative zero is accepted and behaves exactly like
positive zero wherever sign has no financial meaning. Subnormal inputs are
accepted unless a domain rule forbids them. Very large finite inputs are
accepted. If an output overflows, the result is an error
([NUMERICAL_CORRECTNESS §5](NUMERICAL_CORRECTNESS.md#5-numerical-policies)).

**Dependencies.** None. FS-CORE is the root of the dependency graph.

**Expected behavior.**

- **FR-CORE-01** Every operation that can fail for some input MUST return
  either a value or a typed error. Failure MUST NOT be signalled by a
  sentinel, NaN, or panic.
- **FR-CORE-02** Errors MUST be programmatically inspectable, with a kind and
  structured context such as the offending values. Message wording is not part
  of the stability contract.
- **FR-CORE-03** Real-valued inputs MUST be checked for finiteness at the
  public boundary.
- **FR-CORE-04** No public operation may depend on hidden state: globals, the
  system clock, locale, environment, or thread identity.
- **FR-CORE-05** Each convention, frequency, time unit, compounding method,
  and calendar MUST have exactly one canonical identifier. Formatting and then
  parsing MUST return the same value.
- **FR-CORE-06** Parsing MUST accept the canonical identifier plus a
  documented list of aliases. A label that names more than one convention,
  such as `30/360`, `Actual/Actual`, `ACT/ACT`, or `Actual/365`, MUST be
  rejected with an ambiguity error that lists the candidates. It is never
  silently mapped to a default. Matching is exact: case-sensitive, with no
  trimming or other normalization, so every accepted spelling appears in the
  documented list (clarified in M1).
- **FR-CORE-07** Rates are decimal fractions everywhere. Any conversion to or
  from percent or basis points MUST be explicit and named.

**Validation criteria.** At least one test triggers each error variant
(TS-CORE-01). Formatting and parsing round-trip exhaustively over every
enumeration value, and every listed ambiguous label is rejected (TS-CORE-02).
Every public type is asserted at compile time to be safe to send and share
between threads.

**Explicit exclusions.** Currency and money types; rounding conventions;
market quotes and observers; configuration objects; logging.

---

## 5. FS-DATE — Dates

**Purpose.** Represents calendar dates and provides exact, checked date
arithmetic. Every other domain builds on it.

**Financial definitions.**

- **Date.** A day in the proleptic Gregorian calendar as used by ISO 8601, with
  no time of day and no time zone. The Gregorian rules also apply to dates
  before 1582.
- **Leap year.** A year divisible by 4 but not by 100, or a year divisible by
  400. So 1900 and 2100 are not leap years, while 2000 and 2400 are.
- **Month length.** 31 days for January, March, May, July, August, October and
  December; 30 days for April, June, September and November; 28 days for
  February, or 29 in a leap year.
- **Actual day difference.** `E − S` is the signed count of calendar days.
- **End of month (calendar sense).** The last calendar day of the month. The
  business-day sense is defined in FS-CAL.
- **Weekday.** Monday to Sunday. Where a numeric weekday is exposed, it follows
  ISO 8601 (Monday = 1 … Sunday = 7). QuantLib numbers Sunday as 1; this
  affects interop only, not behavior.

**Supported range (proposed, [OQ-09](IMPLEMENTATION_READINESS.md#5-decision-register)).**
From 0001-01-01 to 9999-12-31 inclusive, which is 3,652,059 days. This covers
QuantLib's range of 1901-01-01 to 2199-12-31. It also matches the range of
four-digit ISO 8601 years and the range of Python's `datetime` module, which
is useful as an independent test oracle. Differential tests against QuantLib
are limited to QuantLib's range. See [ADR-0005](adr/ADR-0005-date-representation.md).

**Inputs.** Year, month, and day components; signed day counts; periods
(FS-PER); ISO 8601 text.

**Outputs.** Date values, signed integer day differences, weekdays,
booleans, and text.

**Invariants.**

- INV-DATE-1: Every date value lies within the supported range.
- INV-DATE-2: Valid `(year, month, day)` triples map one-to-one onto date
  values, and date order matches chronological order.
- INV-DATE-3: `(d + n days) − d = n` whenever `d + n days` is in range.
- INV-DATE-4: `weekday(d + 7k days) = weekday(d)`, and consecutive dates have
  consecutive weekdays.
- INV-DATE-5: Adding `12n` months gives the same date as adding `n` years.

**Anti-invariants.** These properties do **not** hold, and tests must not
assume them:

- Month addition is not invertible: 2023-01-31 + 1M − 1M = 2023-01-28.
- Month addition is not associative: (2023-01-31 + 1M) + 1M = 2023-03-28, but
  2023-01-31 + 2M = 2023-03-31.

**Preconditions.** Month is 1–12. Day is from 1 to the month length. Year is
within the supported range.

**Failure cases.**

| Case | Behavior |
|---|---|
| Invalid components: month 0 or 13, day 0 or 32, 30 February, 29 February in a common year, 31 April | Invalid-date error carrying the components |
| Year outside the supported range | Out-of-range error |
| Arithmetic result outside the supported range | Out-of-range error. Never wraps, saturates, or panics |
| Overflow in an intermediate value, for example adding the maximum representable day count | Out-of-range error |
| Malformed ISO 8601 text | Parse error carrying the input text |

**Boundary conditions.** 0001-01-01 minus one day and 9999-12-31 plus one day
both produce errors. Day-by-day increments across 28 February in common and
leap years; across 31 December; and across the century years 1900, 2000, 2100,
and 2400.

**Dependencies.** FS-CORE.

**Expected behavior.**

- **FR-DATE-01** A date MUST be constructible from valid components and MUST
  reject invalid components (see failure cases).
- **FR-DATE-02** The library MUST provide leap-year tests, month lengths, year
  lengths, day of year, weekday, first and last calendar day of the month, and
  a calendar end-of-month test.
- **FR-DATE-03** Adding or subtracting days and weeks is exact. One week is
  seven days.
- **FR-DATE-04** Adding `n` months moves the month by `n`, carrying into the
  year, and then **clamps** the day to the length of the target month.
  Examples: 2023-01-31 + 1M = 2023-02-28; 2024-01-31 + 1M = 2024-02-29;
  2024-03-31 − 1M = 2024-02-29. This matches QuantLib 1.43 (`Date::advance`).
- **FR-DATE-05** Adding `n` years is the same as adding `12n` months. Example:
  2024-02-29 + 1Y = 2025-02-28.
- **FR-DATE-06** Plain date arithmetic MUST NOT preserve end-of-month
  semantics implicitly. Example: 2023-02-28 + 1M = 2023-03-28. Preserving end
  of month is an explicit option of calendar advancement (FS-CAL) and of future
  schedule generation, and is never a side effect.
- **FR-DATE-07** The difference between two dates is a signed integer number
  of days.
- **FR-DATE-08** ISO 8601 extended format `YYYY-MM-DD` MUST be parsed and
  formatted with exactly four-digit years in the supported range. Formatting
  then parsing returns the same date.
- **FR-DATE-09** The library MUST NOT read the system clock. There is no
  "today" operation ([ADR-0011](adr/ADR-0011-explicit-state.md)).
- **FR-DATE-10** (P1) The n-th weekday of a month (for example the third
  Wednesday) and the next or previous given weekday, as needed by holiday
  rules and future IMM dates.
- **FR-DATE-11** The internal representation, such as a serial day number, is
  not a stable public API. Conversion to and from Excel or QuantLib serial
  numbers is Deferred ([OQ-12](IMPLEMENTATION_READINESS.md#5-decision-register)).

**Validation criteria.**

- Exhaustive: every date in the supported range round-trips through its
  components, and weekdays run in sequence (TS-DATE-06).
- Leap-year, month-end, invalid-construction, overflow, and clamping scenarios
  (TS-DATE-01 to TS-DATE-05, TS-DATE-07).
- Independent oracle: proleptic Gregorian algorithms published in the public
  domain ([Hinnant, *chrono-Compatible Low-Level Date Algorithms*](http://howardhinnant.github.io/date_algorithms.html)),
  and Python's `datetime` for fixture generation.
- Differential tests against QuantLib 1.43 for arithmetic between 1901 and
  2199, including month clamping.

**Explicit exclusions.** Time of day, time zones, leap seconds, the Julian
calendar and historical calendar reforms, non-Gregorian calendars (for example
Hijri, or the Thai Buddhist Era year numbering), and intraday or
high-resolution dates.

---

## 6. FS-PER — Periods and frequencies

**Purpose.** Represents tenors (`3M`, `1Y`) and periodic frequencies, and
converts between them.

**Financial definitions.**

- **Time unit:** Days, Weeks, Months, Years.
- **Period (tenor):** a signed integer length plus a time unit, for example
  `3M`, `-1Y`, `2W`, `0D`.
- **Frequency:** the number of regular events per year in a periodic schedule
  or compounding.

| Frequency | Events per year | Equivalent period |
|---|---|---|
| Annual | 1 | 1Y |
| Semiannual | 2 | 6M |
| EveryFourthMonth | 3 | 4M |
| Quarterly | 4 | 3M |
| Bimonthly | 6 | 2M |
| Monthly | 12 | 1M |
| EveryFourthWeek | 13 | 4W |
| Biweekly | 26 | 2W |
| Weekly | 52 | 1W |
| Daily | 365 | 1D |
| Once | single event (for example a zero-coupon payment) | none |

"No frequency" is expressed as the absence of a value, not as a frequency.
QuantLib's `OtherFrequency` is Out of Scope, because an unknown frequency
cannot be used in a calculation. The event counts above match QuantLib 1.43
(`ql/time/frequency.hpp`).

**Inputs.** Lengths, units, frequencies, and period text such as `3M`.

**Outputs.** Periods, frequencies, optional mappings, and text.

**Invariants.**

- INV-PER-1: Normalization is idempotent. `12M` normalizes to `1Y` and `14D`
  normalizes to `2W`. Every zero-length period normalizes to the same zero
  period.
- INV-PER-2: Equality and hashing agree (normalized equality, [OQ-10](IMPLEMENTATION_READINESS.md#5-decision-register)).
- INV-PER-3: Negating twice returns the original period, and multiplying by
  zero gives the zero period.
- INV-PER-4: For each named periodic frequency, mapping to a period and back
  returns the same frequency.

**Preconditions.** Lengths and multipliers are such that the results stay in
the representable range.

**Failure cases.** Overflow when multiplying or adding (error). Adding periods
with incompatible units, such as months plus days (error). Converting `Once`
to a period (no period exists). Malformed period text (parse error).

**Boundary conditions.** Zero-length periods; negative periods; periods that
are not a named frequency, such as `5M`, map to no frequency; `0Y` versus
`0D`.

**Dependencies.** FS-CORE.

**Expected behavior.**

- **FR-PER-01** Periods MUST support negation, multiplication by an integer
  (checked), and addition when the units are compatible. Years and months
  combine as months; weeks and days combine as days. Other combinations are an
  error.
- **FR-PER-02** Equality is **normalized** (proposed, OQ-10): `12M` equals
  `1Y` and `14D` equals `2W`. Ordering is **partial**. Only periods in the
  same family (Months/Years, or Days/Weeks) are comparable; other pairs are
  incomparable. QuantLib instead tries to decide cross-family comparisons
  using bounds on month lengths and raises an error when it cannot. That is a
  documented difference.
- **FR-PER-03** Every named periodic frequency MUST map to a period, as in the
  table. A period MUST map to a frequency only when the table has a matching
  row; otherwise there is no frequency.
- **FR-PER-04** Canonical period text is the length followed by an uppercase
  unit letter (`D`, `W`, `M`, `Y`). Parsing also accepts lowercase unit
  letters. Composite text such as `1Y6M` is Deferred.
- **FR-PER-05** Documentation MUST make clear that **what a `Days` period
  means depends on the operation**. Plain date arithmetic counts calendar
  days. Calendar advancement counts business days (FR-CAL-05).

**Validation criteria.** Exhaustive tests over small lengths and every unit
(TS-PER-01 to TS-PER-03). Differential tests against QuantLib for
period/frequency mappings and normalization.

**Explicit exclusions.** Intraday units (hours down to microseconds),
composite periods, and QuantLib's `OtherFrequency`.

---

## 7. FS-CAL — Calendars

**Purpose.** Decides whether a date is a business day for a market, and
adjusts, advances, and counts dates in business-day terms.

**Financial definitions.**

- **Business day:** a day on which the relevant market or system is open, as
  defined by the calendar.
- **Holiday:** any non-business day, weekends included (as in QuantLib).
  Queries can tell a weekend from a holiday that falls on a weekday.
- **Weekend:** the days of the week that the calendar treats as non-business
  by default. Weekend definitions differ between jurisdictions, so they are a
  property of each calendar.
- **Null calendar:** every day is a business day. It has no weekends.
- **Weekends-only calendar:** Saturdays and Sundays are non-business. There
  are no other holidays.
- **Custom calendar:** user-supplied weekend days and explicit holiday dates.
  QuantLib calls this a "bespoke" calendar.
- **Holiday-aware market calendar:** a calendar defined by an authoritative
  market source, for example TARGET, the euro-area payment system calendar.
- **Joint calendar:** a combination of calendars under one of two rules.
  *Join holidays*: a day is a non-business day if any member treats it as
  one. *Join business days*: a day is a business day if any member treats it
  as one.
- **Business end of month:** the last business day of a month. A date is "at
  business end of month" if it falls on or after that day. This matches
  QuantLib 1.43, where a holiday after the last business day also counts.

**Inputs.** Dates, signed counts, periods, business-day conventions,
end-of-month options, inclusion options for counting, and calendar
definitions.

**Outputs.** Booleans, dates, signed counts, and date lists.

**Invariants.**

- INV-CAL-1: Whether a date is a business day depends only on the calendar
  value and the date.
- INV-CAL-2: In a null calendar every date is a business day. In a
  weekends-only calendar a date is a business day exactly when it is not a
  Saturday or Sunday.
- INV-CAL-3: Under *join holidays*, a date is a business day exactly when
  every member says so. Under *join business days*, it is a business day when
  at least one member says so.
- INV-CAL-4: Adjusting a date that is already a business day returns it
  unchanged, for every convention.
- INV-CAL-5: Advancing by `n ≠ 0` business days returns a business day,
  strictly after the start date for `n > 0` and strictly before it for
  `n < 0`.
- INV-CAL-6: Counting business days from `b` to `a` gives the negative of
  counting from `a` to `b` with the inclusion options swapped.

**Preconditions.** A joint calendar has at least one member. A custom calendar
has at least one business weekday.

**Failure cases.**

| Case | Behavior |
|---|---|
| A search for a business day would leave the supported date range | Out-of-range error |
| No business day found within the documented search bound ([OQ-19](IMPLEMENTATION_READINESS.md#5-decision-register); proposed bound 366 consecutive days) | No-business-day error |
| Joint calendar with no members, or custom calendar whose seven weekdays are all weekend | Construction error |
| Query outside the calendar's rule validity or verified coverage | As decided in [OQ-18](IMPLEMENTATION_READINESS.md#5-decision-register). Proposed: rules extrapolate deterministically and coverage can be queried |

**Boundary conditions.** Holidays that fall on weekends are not counted twice
and are not substituted unless the calendar's rule says so. Holidays shared by
members of a joint calendar. Runs of holidays across a month end (for example
TARGET at Easter 2024) and across a year end (TARGET 25 and 26 December and
1 January). The first and last supported dates. A 29 February holiday in a
custom calendar.

**Dependencies.** FS-DATE, FS-PER, FS-BDC.

**Expected behavior.**

- **FR-CAL-01** The library MUST provide: a business-day test, a holiday test,
  a weekend test by weekday, the business start and end of the month with the
  matching tests, holiday lists over a range (optionally with weekends), and
  business-day lists over a range.
- **FR-CAL-02** It MUST provide null, weekends-only, custom, and joint
  calendars (both rules).
- **FR-CAL-03** Calendars are immutable. Adding or removing a holiday creates a
  **new** calendar value, and no other calendar value is affected. This
  differs from QuantLib 1.43, where built-in calendars share one
  implementation, so a holiday added through one instance shows up in every
  instance in the process.
- **FR-CAL-04** Adjustment applies a business-day convention (FS-BDC).
- **FR-CAL-05** Advancing a date `d` by `n` units with convention `c` and an
  end-of-month option. Proposed; this matches QuantLib 1.43 behavior:
  1. If `n = 0`: return `d` adjusted by `c`.
  2. Days (`n ≠ 0`): step one calendar day at a time towards the sign of `n`,
     counting only business days, until `abs(n)` business days have been
     counted. The result is a business day, and `c` is not applied. `d`
     itself does not have to be a business day.
  3. Weeks: add `7n` calendar days, then adjust by `c`.
  4. Months and Years: add the period with clamping (FR-DATE-04/05). If the
     end-of-month option is on: when `c` is Unadjusted and `d` is the last
     calendar day of its month, return the last calendar day of the target
     month. Otherwise, when `d` is at business end of month, return the last
     business day of the target month. If neither applies, adjust by `c`.
- **FR-CAL-06** A separate, explicitly named "add business days" operation
  MUST exist, so code does not depend on the special meaning of a `Days`
  period (FR-PER-05).
- **FR-CAL-07** Counting business days between `from` and `to` takes an
  explicit inclusion option for each endpoint; the proposed default includes
  the start and excludes the end. When `from > to`, the result is the negative
  count with the inclusion options swapped. When `from = to`, the result is 1
  only if both endpoints are included and the date is a business day,
  otherwise 0. This matches QuantLib 1.43.
- **FR-CAL-08** The library MUST compute Western (Gregorian) Easter
  algorithmically for every supported year, for example with the anonymous
  Gregorian computus. QuantLib 1.43 uses precomputed tables for 1901–2199.
  Orthodox Easter is Deferred.
- **FR-CAL-09** Every holiday-aware calendar MUST record its provenance:
  authoritative sources, retrieval date, verified-coverage years, rule
  effective-date ranges, known historical corrections, and the date it was
  last reviewed ([ADR-0005](adr/ADR-0005-date-representation.md)). Coverage
  MUST be queryable.
- **FR-CAL-10** No holiday dataset is written in M0. The first dataset (TARGET
  proposed, [OQ-17](IMPLEMENTATION_READINESS.md#5-decision-register)) is added
  in M3 only after approval.

**TARGET reference (for the M3 pilot, if approved).** The ECB's long-term
calendar of 14 December 2000 sets the TARGET closing days from 2002 "until
further notice": Saturdays, Sundays, New Year's Day, Good Friday, Easter
Monday, 1 May, Christmas Day, and 26 December
([ECB press release](https://www.ecb.europa.eu/press/pr/date/2000/html/pr001214_4.en.html)).
QuantLib 1.43 also encodes rules for earlier years: Good Friday, Easter Monday,
Labour Day, and 26 December from 2000, and 31 December in 1998, 1999 and 2001.
Those earlier rules must be verified against primary ECB sources before they
are used.

**Validation criteria.** Invariants checked exhaustively over windows of
several years for every calendar type (TS-CAL-01 to TS-CAL-05, TS-CAL-07).
TARGET holidays compared with primary ECB sources over the verified coverage
(TS-CAL-06). Differential tests against QuantLib 1.43 for adjustment,
advancement, and counting.

**Explicit exclusions.** Market calendars other than the approved pilot;
trading hours, early closes and half days; calendars that depend on lunar or
astronomical calculation (Islamic, Chinese, Thai Buddhist) until a data policy
covers them; weekend definitions that change over time (Under Research).

---

## 8. FS-BDC — Business-day conventions

**Purpose.** Moves a date that is not a business day to a business day,
according to a standard rule.

**Financial definitions.** `Following(d)` is the earliest business day on or
after `d`. `Preceding(d)` is the latest business day on or before `d`. The
wording below is SabiQuant's own. QuantLib 1.43 groups Following, Modified
Following, and Preceding as ISDA conventions and the others as non-ISDA. The
ISDA text itself has not been consulted ([R-02](IMPLEMENTATION_READINESS.md#6-risk-register)).

| Convention | Rule (when `d` is not a business day) |
|---|---|
| Unadjusted | Return `d` unchanged. |
| Following | Return `Following(d)`. |
| Modified Following | Return `Following(d)` if it is in the same month as `d`, otherwise `Preceding(d)`. |
| Preceding | Return `Preceding(d)`. |
| Modified Preceding | Return `Preceding(d)` if it is in the same month as `d`, otherwise `Following(d)`. |
| Half-Month Modified Following | Let `f = Following(d)`. If `f` is in a different month from `d`, or the day of `d` is ≤ 15 and the day of `f` is > 15, return `Preceding(d)`. Otherwise return `f`. |
| Nearest | Return whichever of `Preceding(d)` and `Following(d)` is fewer calendar days away. On a tie, return `Following(d)`. |

**Inputs.** A date, a calendar, and a convention. **Outputs.** A date.

**Invariants.**

- INV-BDC-1: If `d` is a business day, every convention returns `d`.
- INV-BDC-2: Unadjusted is the identity, even on non-business days.
- INV-BDC-3: Adjustment is idempotent: adjusting the result again with the
  same convention returns the same result.
- INV-BDC-4: Following is non-decreasing in `d` and returns a date `≥ d`.
  Preceding is non-decreasing in `d` and returns a date `≤ d`. No monotonicity
  is claimed for the other conventions until exhaustive tests confirm it.
- INV-BDC-5: Modified Following and Modified Preceding return a date in the
  same month as `d`, provided that month contains at least one business day.

**Preconditions.** None beyond valid inputs.

**Failure cases.** A search leaves the supported range, or exceeds the search
bound (FS-CAL). A modified convention finds no business day in `d`'s month;
the proposal is to return an error rather than a date in another month. That
can only happen with pathological calendars. QuantLib does not check for it.

**Boundary conditions.** Month ends and the 15th (Half-Month Modified
Following), the start and end of the year, the edges of the supported range,
and equal distances for Nearest.

**Reference examples.** Weekdays were verified independently.

| Calendar | Date | F | MF | P | MP | HMMF | Nearest |
|---|---|---|---|---|---|---|---|
| Weekends-only | 2024-08-31 (Sat) | 2024-09-02 | 2024-08-30 | 2024-08-30 | 2024-08-30 | 2024-08-30 | 2024-08-30 |
| Weekends-only | 2024-06-15 (Sat) | 2024-06-17 | 2024-06-17 | 2024-06-14 | 2024-06-14 | 2024-06-14 | 2024-06-14 |
| TARGET | 2024-03-29 (Good Friday) | 2024-04-02 | 2024-03-28 | 2024-03-28 | 2024-03-28 | 2024-03-28 | 2024-03-28 |
| Weekends-only plus holiday 2024-07-03 (Wed) | 2024-07-03 | 2024-07-04 | 2024-07-04 | 2024-07-02 | 2024-07-02 | 2024-07-04 | 2024-07-04 (tie goes to Following) |

**Dependencies.** FS-DATE, plus a calendar (FS-CAL).

**Expected behavior.**

- **FR-BDC-01** All seven conventions MUST be supported with the rules above.
- **FR-BDC-02** Conventions are a closed enumeration with canonical
  identifiers (FR-CORE-05).

**Validation criteria.** TS-BDC-01 to TS-BDC-05, including exhaustive
differential comparison with QuantLib 1.43 over date windows for each
calendar type.

**Explicit exclusions.** The FRN (Eurodollar) convention is Under Research; it
is a schedule-level rule rather than a single-date adjustment. Treating end
of month as a business-day convention is excluded; it is handled by the
end-of-month option (FR-CAL-05).

---

## 9. FS-DCC — Day-count conventions

### 9.1 General specification

**Purpose.** Measures the length of an accrual period, both as a day count
and as a year fraction, by a named market convention. Year fractions accrue
interest and turn dates into times for rate calculations.

**Financial definitions.** A *day-count convention* defines two things for an
accrual period `[S, E)`: a *day count* (an integer, where the convention
defines one) and a *year fraction* (also called the day count fraction).
Conventions with similar names are **different conventions** and MUST NOT be
treated as interchangeable.

**Inputs.** `S`, `E`, plus parameters required by the convention: a
determination (reference) period and frequency (Actual/Actual ICMA), a
termination date (30E/360 ISDA), a calendar (Business/252), and possibly an
end-of-month flag (30/360 US, [OQ-15](IMPLEMENTATION_READINESS.md#5-decision-register)).
Required parameters are part of the convention value, so they cannot be left
out by accident.

**Outputs.** A day count (integer) and a year fraction (`f64`).

**General invariants and their validity.** Several "obvious" properties are
false for some legitimate conventions. Properties hold **only** where the
table says so.

| Property | Holds for | Fails for (counterexample) |
|---|---|---|
| `YF(S, S) = 0` | All conventions in H1 scope | QuantLib's 1/1 convention returns 1 when `S = E` (out of H1 scope) |
| Antisymmetry `YF(E, S) = −YF(S, E)` | Actual/360 and Actual/365F (by construction); Actual/Actual ISDA, ICMA and AFB in QuantLib 1.43, which negates the result for reversed dates | 30/360 Bond Basis applied directly: 2023-01-15 → 2023-03-31 gives 76 days, but the reverse gives −75 |
| Additivity `YF(a, c) = YF(a, b) + YF(b, c)` (exact arithmetic) | Actual/360, Actual/365F, Actual/Actual ISDA, Business/252 | 30/360 Bond Basis: (01-15 → 01-31) 16 + (01-31 → 02-15) 15 ≠ (01-15 → 02-15) 30 days, all in 2023 |
| Same month and day one year later gives exactly 1 | 30/360 Bond Basis and 30E/360 (`S` not 29 February); Actual/Actual ICMA when the interval is one annual determination period; Actual/Actual AFB except at the 28/29 February edge below | Actual/Actual ISDA: 1999-07-01 → 2000-07-01 = 1.0013773486…; 30E/360 (ISDA): 2023-02-28 → 2024-02-28 = 358/360; Actual/360: 365/360; AFB under QuantLib 1.43's counting-back rule: 2024-02-28 → 2025-02-28 = 1 + 1/365 (OQ-16) |
| Non-decreasing in `E` for fixed `S` | Actual/360, Actual/365F, Actual/Actual ISDA, Business/252 | Not claimed for the other conventions until exhaustive tests confirm it |

**Reversed dates ([OQ-14](IMPLEMENTATION_READINESS.md#5-decision-register)).**
Proposed: for `S > E`, define `YF(S, E) = −YF(E, S)` for every convention, and
document that this differs from QuantLib for the 30/360 family, where QuantLib
applies the formula directly. The alternative is to reject `S > E` with an
error.

**Separation of concerns.** A convention uses date primitives from FS-DATE:
actual day difference, leap-year test, month length, and the last-of-February
test. Rules specific to one convention live only in the day-count module.

**Failure cases.** A required parameter is missing or invalid: a
determination period that does not contain the accrual dates, a frequency that
is not a valid determination frequency, or an empty calendar. Calendar search
errors are passed through from FS-CAL.

**Result exactness.** A convention whose year fraction is a single division of
two exactly representable integers (Actual/360, Actual/365F, the 30/360
family, Business/252) MUST compute it as **one** correctly rounded division.
That makes the result bitwise reproducible. Composite conventions (Actual/Actual
ISDA, ICMA, and AFB) have tolerances in ULPs
([NUMERICAL_CORRECTNESS §7](NUMERICAL_CORRECTNESS.md#7-tolerance-policy)).

**Validation criteria.** TS-DCC-01 to TS-DCC-08. These include exhaustive
enumeration of date pairs against an exact rational oracle, literature
examples, and differential comparison with QuantLib 1.43 for each convention.

**Explicit exclusions.** Schedule generation, including notional periods for
ICMA (M7); conventions that are not listed; and percentage or rounding
presentation.

**Sources.** Where the ISDA 2006 Definitions section 4.16 is cited below, the
section numbers come from public secondary sources. The licensed text was not
consulted ([R-02](IMPLEMENTATION_READINESS.md#6-risk-register)). The ICMA
definition comes from *ICMA Primary Market Handbook, Appendix A5 — Day count
fraction: ICMA Actual/Actual* (March 2022). The Actual/Actual worked examples
come from ISDA's 1999 paper *The Actual/Actual Day Count Fraction*. That paper
is an image-only scan; its examples were verified independently here, as
listed below.

### 9.2 Actual/360

| Field | Specification |
|---|---|
| Canonical name / aliases | `Actual/360`; aliases `Act/360`, `A/360` |
| Definition | $\mathrm{DC} = E - S$, $\quad \mathrm{YF} = (E - S)/360$ |
| Arguments | `S`, `E` |
| Reference period | Not used |
| Leap years | No special treatment; 29 February counts as a day |
| End of month | No effect |
| Boundary | `S = E` gives 0; one full year gives 365/360 or 366/360 |
| Result | Integer day count; year fraction is one correctly rounded division |
| Examples | 2024-01-01 → 2024-07-01: 182 days, 182/360 = 0.5055…; 2024-01-01 → 2025-01-01: 366/360 = 1.0166… |
| Sources | ISDA 2006 §4.16(e) (licensed; not consulted) |
| QuantLib 1.43 | `Actual360` (`ql/time/daycounters/actual360.hpp`). It also offers an "include last day" variant, which SabiQuant would treat as a separate convention (P2, Deferred) |
| Differential strategy | Bitwise equality expected against QuantLib and against the rational oracle |
| Priority / status | P0 / Planned (M3) |

### 9.3 Actual/365 (Fixed)

| Field | Specification |
|---|---|
| Canonical name / aliases | `Actual/365 (Fixed)`; aliases `Act/365 (Fixed)`, `A/365 (Fixed)`, `A/365F`. **`Actual/365` without "Fixed" is ambiguous and is rejected.** QuantLib 1.43 warns that ISDA treats "Actual/365" as an alias of Actual/Actual (ISDA) |
| Definition | $\mathrm{YF} = (E - S)/365$ |
| Arguments | `S`, `E` |
| Reference period | Not used |
| Leap years | Denominator is always 365, so a period spanning 29 February can exceed 1 |
| End of month | No effect |
| Boundary | 2024-01-01 → 2025-01-01 = 366/365 = 1.00273…; 2023-01-01 → 2024-01-01 = 1 |
| Result | One correctly rounded division |
| Examples | 2024-02-29 → 2024-03-31: 31/365 = 0.08493…; 2024-01-01 → 2024-07-01: 182/365 = 0.49863… |
| Sources | ISDA 2006 §4.16(d) (licensed; not consulted) |
| QuantLib 1.43 | `Actual365Fixed` (Standard). Its Canadian and NoLeap variants are separate conventions (Under Research, P2) |
| Differential strategy | Bitwise equality expected |
| Priority / status | P0 / Planned (M3) |

### 9.4 Actual/Actual (ISDA)

| Field | Specification |
|---|---|
| Canonical name / aliases | `Actual/Actual (ISDA)`; aliases `Act/Act (ISDA)`, `Actual/Actual (Historical)`. **Bare `Actual/Actual` and `Act/Act` are rejected as ambiguous** |
| Definition | Split `[S, E)` by calendar year. Days that fall in leap years count over 366, days in common years over 365. If `Y1 = Y2`: $\mathrm{YF} = (E - S)/B(Y_1)$. Otherwise: $\mathrm{YF} = \frac{J(Y_1{+}1) - S}{B(Y_1)} + (Y_2 - Y_1 - 1) + \frac{E - J(Y_2)}{B(Y_2)}$, where $J(Y)$ is 1 January of year $Y$ |
| Arguments | `S`, `E` |
| Reference period | Not used |
| Leap years | Days falling in leap years have denominator 366 |
| End of month | No effect |
| Boundary | `S = E` gives 0. An exact one-year anniversary is generally **not** 1 |
| Result | Composite: up to three rounded terms. Tolerance in ULPs |
| Examples (verified) | 2003-11-01 → 2004-05-01: 61/365 + 121/366 = 0.4977243805674…; 1999-07-01 → 2000-07-01: 184/365 + 182/366 = 1.0013773486039…; 2003-07-15 → 2004-01-15: 170/365 + 14/366 = 0.5040047907777…; 1999-02-01 → 1999-07-01: 150/365 = 0.4109589041095… |
| Sources | ISDA 2006 §4.16(b) (licensed; not consulted); ISDA 1999 paper. QuantLib 1.43's test suite marks the 2003-07-15 → 2004-01-15 case as disagreeing with an ISDA market-conventions document (`mktc1198.pdf`). This is a known reference-side ambiguity to resolve from primary sources |
| QuantLib 1.43 | `ActualActual(ISDA)`; aliases `Historical` and `Actual365` |
| Differential strategy | ULP-bounded comparison (operation order may differ); exact comparison against the rational oracle after rounding |
| Priority / status | P0 / Planned (M3) |

### 9.5 Actual/Actual (ICMA)

| Field | Specification |
|---|---|
| Canonical name / aliases | `Actual/Actual (ICMA)`; aliases `Act/Act (ICMA)`, `Actual/Actual (ISMA)`, `Act/Act (ISMA)` |
| Definition | Paraphrasing ICMA PMH Appendix A5 (Version 2). Let `F` be the number of determination periods that normally end in a year. If the accrual period lies within one determination period: $\mathrm{YF} = \dfrac{\text{days in accrual period}}{F \times \text{days in determination period}}$. If it spans several determination periods, the YF is the sum of that expression for the part of the accrual period inside each one |
| Arguments | `S`, `E`; the determination-period boundary dates covering `[S, E)`, including notional (quasi-coupon) periods for irregular stubs; and frequency `F` |
| Reference period | **Required and explicit** ([OQ-21](IMPLEMENTATION_READINESS.md#5-decision-register)). QuantLib 1.43, without a schedule, guesses the period length by rounding `12 × days / 365` to whole months. When no reference period is given, it uses `[S, E]` itself. SabiQuant does not guess |
| Leap years | Handled through the actual length of the determination period |
| End of month | Determination periods for end-of-month bonds must follow the bond's end-of-month roll. Generating notional periods belongs to schedules (M7). In M3 the boundary dates are inputs |
| Boundary | A regular full period gives exactly `1/F`. `S = E` gives 0 |
| Result | Composite: a sum of rounded quotients. Tolerance in ULPs |
| Examples (verified) | Regular: 2003-11-01 → 2004-05-01, semiannual, determination period identical: 182/(2 × 182) = 0.5. Short first: 1999-02-01 → 1999-07-01, annual, determination period 1998-07-01 → 1999-07-01: 150/365 = 0.4109589041…. Long first: 2002-08-15 → 2003-07-15, semiannual, notional period 2002-07-15 → 2003-01-15 (184 days, 153 of them accrued), regular period 2003-01-15 → 2003-07-15 (181 days): 153/368 + 181/362 = 0.9157608695652… |
| Sources | ICMA PMH Appendix A5 (March 2022; public PDF); ISDA 2006 §4.16(c) (licensed; not consulted); ISDA 1999 paper. ICMA notes the convention was not specifically designed for business-day-adjusted interest periods |
| QuantLib 1.43 | `ActualActual(ISMA)` / `ActualActual(Bond)`, with or without a `Schedule` |
| Differential strategy | Compare with QuantLib only when explicit reference dates are given to both; ULP-bounded |
| Priority / status | P0 / Planned (M3) |

### 9.6 Actual/Actual (AFB)

| Field | Specification |
|---|---|
| Canonical name / aliases | `Actual/Actual (AFB)`; alias `Actual/Actual (Euro)` (QuantLib name) |
| Definition | $\mathrm{YF} = N + \dfrac{E^{*} - S}{B^{*}}$. Here `N` is the number of whole years counted **back from `E`** without passing `S`. `E*` is `E` moved back by `N` years. `B* = 366` if `[S, E*)` contains a 29 February, otherwise 365 |
| Counting-back rule | QuantLib 1.43: when counting back one year lands on 28 February of a leap year, 29 February is used instead. Whether this matches the primary AFB/ISDA source is open ([OQ-16](IMPLEMENTATION_READINESS.md#5-decision-register)) |
| Arguments | `S`, `E` |
| Reference period | Not used |
| Leap years | Through `B*` and the counting-back rule |
| End of month | Only through the 28/29 February counting-back rule |
| Boundary | Whole-year intervals usually give whole numbers, but not always. Under QuantLib 1.43's rule, 2024-02-28 → 2025-02-28 gives 1 + 1/365 = 1.0027397…: counting back lands on 2024-02-28, which moves to 2024-02-29 and leaves a one-day stub. This is the key test case for OQ-16. `S = E` gives 0 |
| Result | Integer plus one rounded quotient |
| Examples (verified) | 2003-11-01 → 2004-05-01: 182/366 = 0.4972677595628…; 1999-07-01 → 2000-07-01: 1; 1999-02-01 → 1999-07-01: 150/365; 2023-03-01 → 2024-02-29: 365/365 = 1 |
| Sources | ISDA 1999 paper (scanned; must be transcribed by hand in M3); French market (AFB) conventions (primary source not yet identified) |
| QuantLib 1.43 | `ActualActual(AFB)` / `ActualActual(Euro)` |
| Differential strategy | Exhaustive pairs around 28/29 February across leap years; QuantLib differential; each disagreement resolved from primary sources |
| Priority / status | P1 / Requires Decision (OQ-16) |

### 9.7 The 30/360 family

All variants share one formula. Only the **day adjustments** differ:

$$
\mathrm{DC} = 360\,(Y_2 - Y_1) + 30\,(M_2 - M_1) + (D_2' - D_1'), \qquad \mathrm{YF} = \mathrm{DC}/360
$$

"Last day of February" means 28 February in a common year and 29 February in
a leap year. Rules are applied in the order listed, and each rule sees the
values left by the previous one.

| Variant | Canonical name (aliases) | Day adjustments | Extra input | Sources | QuantLib 1.43 | Priority / status |
|---|---|---|---|---|---|---|
| Bond Basis | `30/360 (Bond Basis)` (`30/360 ISDA`, `360/360`) | (1) If `D1 = 31`, set `D1 = 30`. (2) If `D2 = 31` and `D1 = 30`, set `D2 = 30` | — | ISDA 2006 §4.16(f) (not consulted) | `Thirty360(BondBasis)` = `ISMA` | P0 / Planned |
| Eurobond Basis | `30E/360` (`30/360 (Eurobond Basis)`) | (1) If `D1 = 31`, set `D1 = 30`. (2) If `D2 = 31`, set `D2 = 30` | — | ISDA 2006 §4.16(g) (not consulted) | `Thirty360(European)` = `EurobondBasis` | P0 / Planned |
| 30E/360 ISDA | `30E/360 (ISDA)` (`30/360 German`) | (1) If `D1 = 31` or `S` is the last day of February, set `D1 = 30`. (2) If `D2 = 31`, set `D2 = 30`. (3) If `E` is the last day of February **and `E` is not the termination date**, set `D2 = 30` | Termination (maturity) date | ISDA 2006 §4.16(h) (not consulted) | `Thirty360(ISDA, terminationDate)` = `German` | P0 / Planned |
| US | `30/360 (US)` | QuantLib 1.43 order: (1) if `S` is the last day of February, then set `D2 = 30` when `E` is also the last day of February, and set `D1 = 30`; (2) if `D2 = 31` and `D1 ≥ 30`, set `D2 = 30`; (3) if `D1 = 31`, set `D1 = 30`. **Secondary sources apply the February rules only to end-of-month securities** | Possibly an end-of-month flag ([OQ-15](IMPLEMENTATION_READINESS.md#5-decision-register)) | SIFMA practice (primary text not retrieved); secondary sources conflict | `Thirty360(USA)` | P0 / Requires Decision |
| Italian | `30/360 (Italian)` | Eurobond Basis rules, plus: any date in February with day > 27 becomes 30 | — | Not identified | `Thirty360(Italian)` | P3 / Under Research |
| NASD | `30/360 (NASD)` | As documented in QuantLib 1.43. Algebraically, these rules give the **same day count as Bond Basis**: setting `D2` to 1 and incrementing `M2` adds the same 31 − `D1`. A sampled check found no mismatches. Exhaustive differential confirmation is still needed | — | Not identified | `Thirty360(NASD)` | P3 / Under Research |

If "end-of-month applies" is modelled as an explicit flag for 30/360 (US),
QuantLib's unconditional behavior is the same as the flag set to true. With
the flag false, the rules reduce to Bond Basis. Primary-source confirmation is
required ([OQ-15](IMPLEMENTATION_READINESS.md#5-decision-register)).

**Common fields.** Arguments: `S`, `E`, plus the extra input shown. No
reference period. Leap years only matter through the last-of-February rules.
End of month matters through the 31st and last-of-February rules. Result:
integer day count and one correctly rounded division. Differential strategy:
exhaustive pairs against the rational oracle, and bitwise equality against
QuantLib (both compute one division).

**Divergence examples (verified; day counts).**

| `S` → `E` | Bond Basis | 30E/360 | 30E/360 ISDA | US (QuantLib rules) | Italian | Actual days |
|---|---|---|---|---|---|---|
| 2024-02-29 → 2024-03-31 | 32 | 31 | 30 | 30 | 30 | 31 |
| 2023-01-31 → 2023-02-28 | 28 | 28 | 30 (28 if `E` is the termination date) | 28 | 30 | 28 |
| 2023-01-15 → 2023-03-31 | 76 | 75 | 75 | 76 | 75 | 75 |

### 9.8 Business/252

| Field | Specification |
|---|---|
| Canonical name | `Business/252 (<calendar identifier>)` |
| Definition | $\mathrm{YF} = \dfrac{\text{number of business days in } [S, E) \text{ under calendar } K}{252}$ |
| Arguments | `S`, `E`, calendar `K`. **Required. There is no default calendar**; QuantLib 1.43 defaults to Brazil |
| Reference period | Not used |
| Leap years | Only through the calendar |
| End of month | No effect |
| Boundary | `S = E` gives 0; reversed dates follow OQ-14 |
| Result | Integer business-day count; one correctly rounded division |
| Example | Weekends-only calendar, 2024-01-01 (Mon) → 2024-01-08 (Mon): 5 business days, 5/252 |
| Sources | Brazilian market convention (primary source to be identified; Under Research) |
| QuantLib 1.43 | `Business252`. It memoizes business-day counts in process-wide maps keyed by calendar name. SabiQuant MUST NOT use observable process-wide caches ([ADR-0011](adr/ADR-0011-explicit-state.md)) |
| Differential strategy | Compare with QuantLib using calendars that exist in both libraries (weekends-only; TARGET if approved) |
| Priority / status | P1 / Planned (M3) |

### 9.9 Conventions deferred or under research

| Convention | Notes | Status |
|---|---|---|
| Actual/360 (include last day) | Adds one day to the numerator (QuantLib `Actual360(true)`) | Deferred (P2) |
| Actual/365 (No Leap) | Excludes 29 February from the day count (QuantLib `Actual365Fixed(NoLeap)`) | Under Research (P2) |
| Actual/365 (Canadian Bond) | Needs a reference period and frequency; uses a piecewise formula (QuantLib `Actual365Fixed(Canadian)`) | Under Research (P2) |
| Actual/364, Actual/365.25, Actual/366 | Simple actual-over-constant conventions | Deferred (P3) |
| 30/365 | 30E/360-style day count over 365 (QuantLib `Thirty365`) | Deferred (P3) |
| 1/1 | ISDA "1/1"; QuantLib returns 1 for `E ≥ S` (even `S = E`) and −1 otherwise | Deferred (P3) |
| Simple (QuantLib `SimpleDayCounter`) | A QuantLib testing helper, not a market convention | Out of Scope |

---

## 10. FS-RATE — Interest rates and compounding

**Purpose.** Represents an interest-rate quote together with its conventions,
and computes compound factors, discount factors, implied rates, and equivalent
rates.

**Financial definitions.**

- **Interest-rate quote:** `q = (r, day-count convention, compounding)`. The
  periodic compounding methods also carry a frequency `m`.
- **Compound factor** `C(t)` for a year fraction `t ≥ 0`:

| Compounding | $C(t)$ |
|---|---|
| Simple | $1 + r\,t$ |
| Compounded (frequency $m$) | $(1 + r/m)^{m t}$ |
| Continuous | $e^{r t}$ |
| Simple then compounded | $1 + r\,t$ if $t \le 1/m$; otherwise $(1 + r/m)^{m t}$ |
| Compounded then simple | $(1 + r/m)^{m t}$ if $t \le 1/m$; otherwise $1 + r\,t$ |

These definitions are standard and match QuantLib 1.43 (`ql/interestrate.cpp`,
`ql/compounding.hpp`). Both hybrid methods are continuous at $t = 1/m$, where
both branches equal $1 + r/m$.

- **Discount factor:** $P(t) = 1/C(t)$.
- **Implied rate:** given $C > 0$ and $t > 0$, the rate whose compound factor
  equals $C$. Simple: $r = (C - 1)/t$. Compounded: $r = m\,(C^{1/(m t)} - 1)$.
  Continuous: $r = \ln C / t$. Hybrids use the branch selected by $t$.
- **Equivalent rate:** for a quote `q1` and a target compounding (and
  frequency), the rate `r2` such that $C_2(t; r_2) = C_1(t; r_1)$ at horizon
  `t`. Between Compounded and Continuous (any frequencies), the equivalent
  rate **does not depend on `t`**. Wherever Simple or a hybrid is involved, it
  **does** depend on `t`.
- **Zero rate (H1 meaning):** the implied rate of a single discount factor at
  time `t` under stated conventions. Curves are H2.
- **Where the year fraction comes from.** The core operations take `t`
  directly. The date-based forms compute $t = \mathrm{YF}(S, E)$ with **the
  quote's own** day-count convention and its required parameters. Because the
  quote carries its convention, the date-based operations cannot mix
  conventions by mistake.

**Domain constraints** (proposed, [OQ-22](IMPLEMENTATION_READINESS.md#5-decision-register)).

| Condition | Required | Reason |
|---|---|---|
| `r` finite | Always | NFR / FR-CORE-03 |
| `t` finite and `t ≥ 0` | For compound and discount factors | Negative horizons have no defined convention. QuantLib 1.43 also rejects them |
| $r > -m$ | Compounded and hybrid methods | Otherwise $1 + r/m \le 0$ and the power is undefined or meaningless |
| $1 + r\,t > 0$ | Simple and the simple branch of hybrids | Otherwise the compound factor is not positive and the discount factor is undefined |
| Result finite and $> 0$ | All methods | For example $e^{r t}$ overflows when $r t \gtrsim 709.78$ and underflows to 0 when $r t \lesssim -745.13$. Either case is an error |
| Frequency is periodic | Compounded and hybrid methods | `Once` and "no frequency" are rejected when the value is built |
| Implied rate: `C` finite and `> 0`; `t > 0` | Always | If `t = 0` and `C = 1`, the rate is 0 (as in QuantLib 1.43). If `t = 0` and `C ≠ 1`, error |

**Numerical formulation** (normative intent; details in
[NUMERICAL_CORRECTNESS §9](NUMERICAL_CORRECTNESS.md#9-analyses-of-foundational-computations)).
Implementations SHOULD use formulations that avoid cancellation, for example
$C = \exp\!\big(m t \cdot \mathrm{log1p}(r/m)\big)$ and
$r = m \cdot \mathrm{expm1}\!\big(\ln C/(m t)\big)$. Equivalent rates between
Compounded and Continuous SHOULD be computed without forming `C`. For example:
$r_c = m\,\mathrm{log1p}(r_m/m)$; $r_m = m\,\mathrm{expm1}(r_c/m)$;
$r_2 = m_2\,\mathrm{expm1}\big(\tfrac{m_1}{m_2}\mathrm{log1p}(r_1/m_1)\big)$.
These formulations differ from QuantLib's direct formulas in the last bits,
and that is accounted for in differential tolerances.

**Inputs.** Rate, compounding (with frequency), day-count convention, `t` or
`(S, E)`, and compound factors.

**Outputs.** Compound factors, discount factors, and rates (`f64`).

**Invariants.** See [NUMERICAL_CORRECTNESS §8](NUMERICAL_CORRECTNESS.md#8-domain-invariants)
(INV-R-1 to INV-R-9). Among them: `C(0) = 1` exactly; `C = 1` exactly when
`r = 0`; `P · C ≈ 1`; equivalent-rate consistency; and round-trip
consistency, with tolerance scaled by the problem's conditioning.

**Preconditions.** As listed in the domain-constraints table.

**Failure cases.** Non-finite input; `t < 0`; non-periodic frequency for a
periodic method; $r \le -m$; $1 + r t \le 0$; overflow or underflow of `C`;
implied rate with `C ≤ 0`, or with `t = 0` and `C ≠ 1`; date-based use with
`E < S`.

**Boundary conditions.** `t = 0`; `r = 0`; `r = −0.0`; tiny `t` (one day under
Act/365F, `1e-6`, `1e-12`); tiny `r`; negative `r` approaching `−m`; large `t`
(100 years); large `r` (1.0 and 10.0); hybrids at exactly `t = 1/m`;
`m = 365` (Daily).

**Dependencies.** FS-CORE, FS-PER (frequency), FS-DCC (date-based forms).

**Expected behavior.**

- **FR-RATE-01** All five compounding methods MUST be supported with the
  definitions above.
- **FR-RATE-02** Compound and discount factors MUST be available both for a
  year fraction and for a date pair, using the quote's day-count convention.
- **FR-RATE-03** Implied rates MUST be computed from a compound factor (or
  discount factor), a horizon, and target conventions.
- **FR-RATE-04** Equivalent rates MUST be computed for a horizon, or for a
  date pair through a day-count convention. The documentation must state when
  the result does and does not depend on the horizon.
- **FR-RATE-05** Every domain violation in the constraints table MUST produce
  a typed error.
- **FR-RATE-06** Negative rates MUST be supported wherever they lie inside the
  domain.

**Validation criteria.** TS-RATE-01 to TS-RATE-10: zero time, zero rate,
negative rates, invalid domains, non-finite inputs, tiny `t`, large inputs,
equivalent-rate consistency, hybrid boundaries, and differential comparison
with QuantLib's `InterestRate`. A high-precision independent oracle provides
ground truth for the ULP-error measurements.

**Explicit exclusions.** Yield curves and term structures; forward rates
between curve points; spreads applied to curves; yield quotation conventions
for particular markets beyond the five compounding methods; rounding rates to
market precision; and QuantLib `OtherFrequency`.

---

## 11. FS-DISC — Rate-based discounting

**Purpose.** Discounts known amounts at known dates using one interest-rate
quote, without curves.

**Financial definitions.**

- **Period discount factor** under quote `q` for `[S, E]`, with `E ≥ S`:
  $P_q(S, E) = 1 / C_q\big(\mathrm{YF}_q(S, E)\big)$. The rate applies to that
  period as a whole.
- **Present value** at `S` of an amount `A` paid at `E`:
  $\mathrm{PV} = A \cdot P_q(S, E)$.
- Forward discount factors between two later dates are a **curve** concept and
  are not defined in H1. Under simple compounding,
  $P(t_2)/P(t_1) \ne 1/(1 + r(t_2 - t_1))$.

**Inputs.** Quote, dates, amount. **Outputs.** Discount factor, present value.

**Invariants.** `P(S, S) = 1` exactly; `PV` is linear in `A`; for `r > 0` and
`E > S`, `P < 1`; for `r < 0`, `P > 1`; `P` depends on the dates only through
`YF(S, E)`.

**Preconditions.** `E ≥ S`; `A` finite.

**Failure cases.** `E < S`; non-finite amount; any FS-RATE domain error;
overflow of the product.

**Boundary conditions.** Same-day discounting; very long periods; amount 0;
negative amounts (allowed: a negative cash flow is still a cash flow).

**Dependencies.** FS-RATE, FS-DCC.

**Expected behavior.**

- **FR-DISC-01** Period discount factors and the present value of a single
  amount MUST be provided as defined.
- **FR-DISC-02** `E < S` MUST be rejected. There is no implicit forward or
  backward valuation.

**Validation criteria.** TS-DISC-01; and differential comparison against
QuantLib, `InterestRate::discountFactor(d1, d2, …)` times the amount.

**Explicit exclusions.** Multi-cash-flow NPV (M7), settlement and ex-coupon
conventions (M8), credit and survival discounting, and curves.

---

## 12. Traceability

| Spec | Requirements | Milestone | Tests | Features | ADRs |
|---|---|---|---|---|---|
| FS-CORE | FR-CORE-01…07 | M1 | TS-CORE-01…02 | F-CORE-* | 0006, 0007, 0011 |
| FS-DATE | FR-DATE-01…11 | M2 | TS-DATE-01…07 | F-TIME-01…09 | 0005 |
| FS-PER | FR-PER-01…05 | M2 | TS-PER-01…03 | F-TIME-10…14 | 0005 |
| FS-CAL | FR-CAL-01…10 | M3 | TS-CAL-01…07 | F-CAL-* | 0005, 0011 |
| FS-BDC | FR-BDC-01…02 | M3 | TS-BDC-01…05 | F-BDC-* | — |
| FS-DCC | §9 | M3 | TS-DCC-01…08 | F-DCC-* | 0006, 0008 |
| FS-RATE | FR-RATE-01…06 | M4 | TS-RATE-01…10 | F-RATE-* | 0006 |
| FS-DISC | FR-DISC-01…02 | M4 | TS-DISC-01 | F-RATE-08…09 | 0006 |
| NFR-01…15 | — | M1–M5 | [TESTING_STRATEGY](TESTING_STRATEGY.md) | — | 0002, 0007, 0009, 0011 |

### 12.1 Where the M1 requirements are tested

| Requirement | What M1 delivers | Tests |
|---|---|---|
| FR-CORE-01, FR-CORE-02, NFR-15 | Typed, inspectable error types with stable kind names (`sabiquant::error`) | TS-CORE-01: `src/error.rs` (every variant, kind names, messages, transparency) |
| FR-CORE-03, NP-01, NP-02 | Validation helpers for inputs and results (internal `numeric` module) | TS-CORE-01: `src/numeric.rs` (each error, signed zeros, subnormals, and a property test over arbitrary bit patterns) |
| FR-CORE-05, FR-CORE-06 | Identifier tables with aliases and ambiguous labels (internal `identifier` module) | `src/identifier.rs` (round trip, ambiguity, exact matching, table checks). TS-CORE-02 completes as each enumeration arrives (M2–M4) |
| FR-CORE-04, NFR-07, NFR-13 | No hidden state, clock, environment or I/O in the library | Clippy configuration (`clippy.toml`), run in CI |
| NFR-02 | No panics on input | Clippy restriction lints in the manifest; unit tests of every error path |
| NFR-05, INV-CORE-3 | Public types can be sent and shared between threads | `tests/public_api.rs` |
| NFR-06, NFR-08 | No unsafe code; every public item documented | Manifest lint policy; documentation build in CI |
| NFR-09 | No dependencies | `tests/policy.rs`; dependency check in CI |
| F-CORE-05, F-CORE-06 | Test support: PRNG, ULP and tolerance comparison, fixture reader | `tests/test_support.rs` |
