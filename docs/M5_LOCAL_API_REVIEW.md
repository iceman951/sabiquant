# Local M5 Public API Review

| Field | Record |
|---|---|
| Review date | 2026-10-12 |
| Reviewer | Independent agent review of the current local M2–M4 implementation |
| Scope | Public construction, context ownership, units, errors, parsing, traits, examples and semantic consistency |
| Authority | Local review preparation under delegated work; no owner sign-off, release acceptance or publication approval |
| Governing documents | [API principles](API_DESIGN_PRINCIPLES.md), [requirements](REQUIREMENTS.md), [ADR-0007](adr/ADR-0007-error-handling.md), [ADR-0009](adr/ADR-0009-api-stability.md) |

The current API provides explicit immutable financial values and typed failures.
No new calculation defect was identified by this API reviewer. Initial policy
gaps led to the local fixes recorded below; normalized period equality retains
an important operation-specific limitation. Documentation completeness and
remaining numerical/source gates still require review. This record is not
the owner API approval required by [M5](ROADMAP.md).

## Public surface and construction

The crate exports five domain modules and 35 public value/error types after
the three named calendar options were added. Internal
identifier tables, numeric helpers, calendar graphs and test support remain
private. There is no public extension trait or convenience prelude to stabilize.

| Module | Public concepts | Construction and inspection contract |
|---|---|---|
| [error](../src/error.rs) | `Error`, `ValueError`, `IdentifierError`, `NonFinite`, `Interval` | Structured domain wrappers; stable `kind`; `Interval` is inspected through finite bounds and `contains`, not constructed by callers |
| [time](../src/time/mod.rs) | `Date`, `DateError`, `Month`, `Weekday`, `Period`, `PeriodError`, `TimeUnit`, `Frequency` | Validated Gregorian dates; every signed `i32` tenor is valid; checked arithmetic; frequency absence is `Option` |
| [calendar](../src/calendar.rs) | `Calendar`, `CalendarError`, `JointRule`, `BusinessDayConvention`, `EndOfMonth`, `EndpointInclusion`, `HolidayWeekendInclusion`, `TargetCalendar`, `TargetPolicy`, `TargetCoverage`; `western_easter` | Explicit calendar definitions and immutable overrides; named enum-only options; bounded searches; TARGET queries preserve policy and validity |
| [daycount](../src/daycount.rs) | `DayCount`, `DayCountError`, `ThirtyE360Isda`, `Business252`, `DeterminationPeriod`, `ActualActualIcma`, `IcmaError` | Five scalar choices; required termination, calendar, or determination-period context supplied explicitly |
| [rates](../src/rates.rs) | `CompoundingMethod`, `Compounding`, `AccrualConvention`, `InterestRate`, `RateError`; four named unit conversions | Validated periodic frequency; finite decimal quote with owned accrual and compounding; checked factors, inference, conversion and present values |

All public structs have private fields. Getters return copies or immutable
references; ICMA periods are exposed as a borrowed slice. Calendar overrides
return new values. `Business252` exposes either its plain calendar or its
guarded TARGET source through distinct optional accessors; it does not expose
TARGET as an unchecked calendar. `InterestRate` owns its full context and
conversion constructs a separate target quote. These boundaries meet the
explicit-state and immutable-value intent.

Construction establishes structural validity, not validity at every possible
query. Negative Simple rates require horizon-specific positivity checks;
finite quotes may still overflow at a long horizon. An ICMA frequency is an
explicit contractual input: the constructor checks ordered contiguous periods
but does not infer or certify their financial consistency with that frequency.
TARGET overrides are declarations checked when queried and cannot enlarge
source coverage.

## Findings and proposed resolutions

The table preserves the initial findings and recommendations. The subsequent
local resolution record describes implemented changes under delegated work;
neither table is independent owner approval.

| ID | Finding and evidence | Proposed resolution before release |
|---|---|---|
| API-01 | Initially Calendar `advance`, endpoint counting and holiday lists took `bool` options, as did TARGET. [API principles §4.3](API_DESIGN_PRINCIPLES.md#4-type-and-signature-principles) instead require descriptive two-valued enums. Behavior was tested, but call sites such as `true, false` hid meaning. | Introduce explicit EOM/inclusion/list options before stabilizing. Implemented locally below. |
| API-02 | Initially `TargetCalendar` implemented `Default`, choosing strict TARGET. [Trait policy](API_DESIGN_PRINCIPLES.md#7-standard-trait-policy) says never provide a calendar default. The earlier [M3 TARGET contract](M3_TARGET_CONTRACT.md) recorded the implementation choice. | Remove `Default`, retaining explicitly named `new`. Implemented locally below. |
| API-03 | FR-CORE-05 promises format/parse round trips for every calendar and convention. `Calendar`/TARGET have no text parser; contextual day counts deliberately require context. A convention name alone cannot reconstruct termination dates, ICMA periods, overrides or coverage policy. `Compounding` has `Display` but no `FromStr`; only its method identifier parses. | Distinguish catalog identifiers from complete contextual values in FR-CORE-05 and identifier rustdoc. Preserve required context rather than inventing parser defaults. Decide separately whether any explicit built-in-calendar identifier catalog is needed; avoid adding arbitrary graph serialization only to satisfy overly broad wording. |
| API-04 | Initially `ActualActualIcma` and `AccrualConvention` were discrete `Eq` values without `Hash`, although the trait policy calls for `Eq`/`Hash` on discrete values. Their existing trait tests did not require hashing. | Add definition-based `Hash` with equal-value hash tests. Implemented locally below; no `Hash` or ordering for `InterestRate`, which contains `f64`. |
| API-05 | The initial seven compiled examples did not cover Calendar/TARGET/B252/termination composition. No `doc(alias)` existed for QuantLib discoverability; rates lacked its reference/mapping/numerical overview and quote-equality explanation. | Add executable contextual examples, mapping/source notes, equality semantics and useful aliases. Specific additions are recorded below; a passing missing-docs lint still does not establish §10 completeness. |
| API-06 | [API principles §5](API_DESIGN_PRINCIPLES.md#5-error-semantics) says root `Error::source` is that of the wrapped domain error. Production and tests intentionally expose the wrapped typed domain error itself. | Clarify the prose: Display and kind delegate; source adds the typed domain layer. Preserve the existing chain and inspectable context. This implementation matches ADR-0007. |
| API-07 | Normalized `Period` equality does not imply equivalent calendar advancement. `7D == 1W`, but Days count business days and Weeks add calendar weeks. Stored units also affect checked period arithmetic representability. | Keep the specified financial dispatch, add a dedicated advancement regression/example, and state that normalized Eq/Hash cannot key operation results without the stored representation. See the concrete example below. |
| API-08 | The independent M5 numerical audit identified a conditioning limit at a rounded hybrid switch near the negative periodic base. With frequency 13, the next valid rate above −13 and time `1/13`, switching at the next binary64 time can increase a negative-rate factor by about 68.4%. Each selected branch follows its declared formula; unconditional rounding-level continuity/monotonicity does not hold there. | Qualify hybrid-switch invariants with threshold/base conditioning and preserve the specified rounded switch. Record the numerical audit's independent branch oracle and regression before treating these claims as verified. |

Private-field structs already prevent external literals and permit internal
field additions. Growable `DayCount`, `AccrualConvention` and all error enums
are non-exhaustive; extensible structured error variants also require `..` in
external matches. Closed month/weekday/unit/frequency/BDC/joint/compounding and
TARGET-policy sets have narrow lint explanations. Changing those declared
closed sets remains an API decision. Numerical behavior and coverage changes
also require behavior-change notes under ADR-0009.

## Local resolution record

The API-01/02 correction changes source signatures before first publication;
it preserves calendar membership, advancement, endpoint counting and coverage
semantics. There are no public `From<bool>` adapters or overloaded boolean
signatures. Private implementation predicates and independent fixture flags
remain booleans where appropriate.

| Finding | Implemented local evidence | Remaining limit |
|---|---|---|
| API-01 | Closed `EndOfMonth::{Preserve, Ignore}`, `EndpointInclusion::{Include, Exclude}`, and `HolidayWeekendInclusion::{Include, Exclude}` replace public boolean choices in Calendar/TARGET. All have `Copy/Eq/Hash`, exact identifier parsing, `ALL` and formatting. Existing fixture consumers migrated explicitly; compile-fail doctests reject old bool call sites. | Pre-publication API change needs behavior/API release notes and final review. |
| API-02 | `TargetCalendar::Default` removed; explicit `new()` remains strict. A compile-fail doctest rejects default construction. A narrow `new_without_default` expectation explains the adopted policy. | No policy exception is introduced. |
| API-03/06 | Shared API/requirements/model prose now distinguishes identifiers from context serialization and preserves the typed root domain source layer. | Catalog scope and complete documentation still require final review; no context-free default parser was added. |
| API-04 | Root added `Hash` to ICMA and `AccrualConvention`; [context_hash](../tests/context_hash.rs) verifies independently built equal definitions retrieve the same keys and preserve different contexts. | No floating quote hashing or ordering was added. |
| API-05 | Calendar, TARGET, Business/252 and termination-sensitive examples compile. Calendar constructor/TARGET aliases and module correspondence were added. Root added rates references, mapping, numerical notes and quote-equality semantics. | Item-by-item §10 completeness remains a separate gate. |
| API-07 | Calendar/Period rustdoc states the stored-unit limitation. A dedicated regression checks equal `7D`/`1W` tenors under all seven BDCs and both EOM options, while Date advancement remains equal. | The specified semantic limitation is intentional, not removed by the API correction. |
| API-08 | Independent numerical audit supplied the counterexample and proposed threshold/base-conditioned expectations. | Review its final invariant/oracle record; ordinary E3 alone does not establish near-boundary switch continuity. |

## Units, intervals and misuse boundaries

| Caller assumption | Actual contract and protection |
|---|---|
| `5.0` is a 5% quote | It is a 500% decimal rate. Use `decimal_from_percent(5.0)` or `decimal_from_basis_points(500.0)`. Construction cannot infer a mistaken unit. |
| A time-only method validates the quote's date context | It receives an externally supplied finite nonnegative year fraction. Only date-based methods query ICMA or guarded calendar coverage. |
| Every increasing date pair has positive accrual | 30-day conventions and closed/weekend intervals can have zero accrual. Conversion into a zero target horizon can return `rate.zero_target_time`. |
| Zero amount/rate or equal dates bypass coverage | Date methods still check contextual coverage. Reversed rate dates fail before accrual lookup; standalone day counters may negate the chronological forward result. |
| All interval APIs have the same endpoint semantics | Calendar lists are inclusive. Calendar counts take two named endpoint options; equal dates count one only when both are included and business. Day counts use chronological half-open intervals and equal dates give zero. |
| Removing a weekend holiday changes weekend rules | It explicitly makes that date business while `is_weekend` remains true. Lists/classification distinguish the resulting business state from the weekday rule. |
| A SOURCE-verified TARGET rule is verified for every date | Coverage is disjoint and queryable. Strict policy rejects gaps/pre-launch dates; projection remains explicitly labeled. Classification, zero advancement and excluded endpoints retain the guard. |
| Equal calendar definitions mean all Boolean-equivalent calendars compare equal | Calendar Eq/Hash canonicalize definition order, duplicates and supported flattening; they do not solve general Boolean equivalence. |
| Nonzero positive rate implies a factor strictly above one | Tiny growth can round to one. Elementary comparisons use conditioned tolerances; crossing a rounded hybrid switch near a negative base needs additional threshold/base conditioning, not only an ordinary E3 allowance. |
| A correctly rounded target rate guarantees a small relative reconstructed-factor error | Near Simple cancellation, target-rate representation sensitivity dominates. Rounded nonpositive active factors reject; valid near-boundary reconstruction needs the derivative term. |

No panicking Date/Period arithmetic operator, default quote, default day count,
implicit evaluation date or context-free contextual parser is present.
`Frequency::Once` is represented for frequency use but cannot construct periodic
compounding or ICMA accrual. This is absence/invalidity expressed explicitly,
not a sentinel periodic frequency.

## Examples reviewed against the current API

These reviewed consumer examples accompany the implemented rustdoc additions.
They were compiled and
executed in a temporary external Rust 2024 consumer against the current local
library, then checked directly with `rustdoc --test` on this Markdown file.
Markdown examples themselves are not part of Cargo's normal documentation-test
run; the corresponding source examples are.

Stored units select different financial operations despite normalized equality:

```rust
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::time::{Date, Period, TimeUnit};

let start = Date::from_ymd(2026, 1, 2)?; // Friday
let days = Period::new(7, TimeUnit::Days);
let week = Period::new(1, TimeUnit::Weeks);
assert_eq!(days, week);
let calendar = Calendar::weekends_only();
assert_eq!(calendar.advance(start, days, BusinessDayConvention::Following, EndOfMonth::Ignore)?,
           Date::from_ymd(2026, 1, 13)?);
assert_eq!(calendar.advance(start, week, BusinessDayConvention::Following, EndOfMonth::Ignore)?,
           Date::from_ymd(2026, 1, 9)?);
# Ok::<(), sabiquant::error::Error>(())
```

Required calendar context and rate units remain explicit at valuation:

```rust
use sabiquant::calendar::{Calendar, TargetCalendar};
use sabiquant::daycount::Business252;
use sabiquant::rates::{Compounding, InterestRate, decimal_from_percent};
use sabiquant::time::Date;

let quote = InterestRate::new(decimal_from_percent(5.0)?,
    Business252::new(Calendar::weekends_only()), Compounding::SIMPLE)?;
let present = quote.present_value_between(-1_000.0,
    Date::from_ymd(2026, 1, 2)?, Date::from_ymd(2026, 1, 9)?)?;
assert!(present > -1_000.0 && present < 0.0);
let target = Business252::with_target(TargetCalendar::new());
assert_eq!(target.day_count(Date::from_ymd(2026, 4, 2)?,
                           Date::from_ymd(2026, 4, 7)?)?, 1);
# Ok::<(), sabiquant::error::Error>(())
```

A termination date affects the convention; it is not inferred from each query:

```rust
use sabiquant::daycount::ThirtyE360Isda;
use sabiquant::time::Date;

let start = Date::from_ymd(2023, 1, 31)?;
let end = Date::from_ymd(2023, 2, 28)?;
assert_eq!(ThirtyE360Isda::new(end).day_count(start, end), 28);
let intermediate = ThirtyE360Isda::new(Date::from_ymd(2023, 3, 31)?);
assert_eq!(intermediate.day_count(start, end), 30);
# Ok::<(), sabiquant::error::Error>(())
```

## Parsing and error review

Date parsing requires exactly ten ASCII bytes `YYYY-MM-DD`, then validates
components and supported range. Period text permits one ASCII sign and leading
zeros, with a single D/W/M/Y letter; formatting preserves stored representation.
Identifier enums accept only canonical spelling and documented aliases, without
case folding or trimming. Ambiguous bare day-count labels return candidates,
including conventions outside the implemented scalar subset. Contextual
construction is intentionally separate from labels.

Errors expose stable domain kinds and structured values, not a stable message.
The nested path `Error -> RateError -> DayCountError -> CalendarError` and
`Error -> RateError -> IcmaError` retain typed causes. A numerical-boundary
error is an unresolved computed-log range classification, not an ordinary
nonfinite input and not a promised mathematical-domain result. No blanket
few-ULP or platform-bitwise guarantee is inferred from the API.

## Evidence and remaining review gates

On native aarch64 macOS with Rust 1.99, this review independently ran:

- `cargo test --offline --test public_api`: **8 passed**.
- `cargo test --offline --doc`: **7 passed**.
- `rustdoc --edition 2024 --test docs/M5_LOCAL_API_REVIEW.md` with the current
  library supplied through `--extern`: **3 reviewed Markdown examples passed**.
- A temporary external consumer at the initial review: **all 32 public types** satisfied
  `Clone + Debug + PartialEq + Send + Sync + 'static`; contextual construction,
  unit conversion, guarded failure, ICMA conversion, termination sensitivity
  and the equal-tenor advancement example compiled and executed successfully.
  Recompiled after the correction with all **35** public types and enum-only
  call sites.

After the local correction, **99 focused integration tests** passed across
calendar/core/count/reference, TARGET/adversarial, day-count/core/context,
public API, rate-date composition and context hashing. The full source
documentation run passed **11 executable examples and 3 compile-fail tests**.
Strict all-target Clippy and `cargo fmt --all --check` passed. The new option
test rejects bool text, case changes, whitespace, NUL and Unicode lookalikes;
existing independent count/reference grids preserve financial behavior.

Existing tests provide complementary evidence: [calendar_core](../tests/calendar_core.rs),
[target](../tests/target.rs), [daycount_extended](../tests/daycount_extended.rs),
[icma](../tests/icma.rs), [time_arithmetic_semantics](../tests/time_arithmetic_semantics.rs),
[rates_dates](../tests/rates_dates.rs) and [rates_properties](../tests/rates_properties.rs).
The latter includes 20,000 replayable normal-domain cases and 8,192 concurrent
quote observations. This review did not regenerate fixtures or repeat the
full M4 debug/release/coverage campaign; that evidence remains in the M4 reports.

The resolution ledger does not replace independent sign-off. Cross-platform execution,
feature/discrepancy traceability, documentation completeness and independent
owner sign-off remain separate M5 gates. `publish = false` remains appropriate;
this record authorizes no remote writes, settings changes or release.
