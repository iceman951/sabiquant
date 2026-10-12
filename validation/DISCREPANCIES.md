# Reference discrepancies

Entries are retained permanently. Every difference below was observed by
executing public APIs of the exact QuantLib 1.43 wheel recorded in the fixture
headers. SabiQuant expectations come from the independently specified
[FS-DATE and FS-PER requirements](../docs/REQUIREMENTS.md), with Python
`datetime`/`calendar` used where a reference date is outside QuantLib's range.
The generator revision, wheel digest, and complete scenario inputs and outputs
are committed alongside each entry. No QuantLib implementation code is used
to derive SabiQuant algorithms.

## D9-DATE-RANGE — 2026-10-11

- **Class:** D9, intentional divergence.
- **Fixture set:** `TIME-DATE-QUANTLIB-ARITHMETIC`, 721 records bearing this ID.
- **Observed:** QuantLib raises for a date shift outside 1901-01-01 through
  2199-12-31. The independent Python oracle returns a date inside SabiQuant's
  supported 0001-01-01 through 9999-12-31 range. For example, 1901-01-01 minus
  one day gives Python/SabiQuant 1900-12-31 and a QuantLib exception.
- **Resolution:** Preserve SabiQuant's wider range. Each affected row retains
  `reference_behavior=raises`, an empty `reference_result`, and the independent
  `expected_result`. The generator rejects a QuantLib exception if the expected
  date is within QuantLib's own range.
- **Requirements:** FR-DATE-01, FR-DATE-03..05, ADR-0005 Part A.
- **Sources consulted:** FS-DATE; actual public `Date + Period` calls recorded
  by `validation/generator/time_fixtures.py`; Python standard-library execution.

## D9-PERIOD-FREQUENCY — 2026-10-11

- **Class:** D9, intentional divergence.
- **Fixture set:** `TIME-PERIOD-QUANTLIB`, 15 records bearing this ID.
- **Observed:** QuantLib maps 11 negative named tenors to positive frequency
  names (for example −1Y to Annual). It maps 0Y to Once. It reports
  `OtherFrequency=999` for 7D, 14D, and 28D even though normalization gives
  1W, 2W, and 4W. Both raw reference frequency codes and the supported names
  are retained in the fixture.
- **Resolution:** SabiQuant returns no frequency for zero and negative
  tenors. Positive equivalent tenors map by normalized equality, so 7D maps
  to Weekly, 14D to Biweekly, and 28D to EveryFourthWeek. These mappings agree
  with the positive FS-PER table and normalized equality. The generator fails
  if a mismatch does not fall in these explicitly documented classes.
- **Requirements:** FR-PER-02/03, INV-PER-2/4.
- **Sources consulted:** FS-PER frequency table; actual public
  `Period.frequency()` and `Period.normalized()` calls in the committed rows.

## D9-ONCE-PERIOD — 2026-10-11

- **Class:** D9, intentional divergence.
- **Fixture:** `FREQ-QL-Once` in `TIME-PERIOD-QUANTLIB-FREQUENCIES`.
- **Observed:** QuantLib constructs `Period(Once)` as a 0Y sentinel. The
  frequency integer is 0, reflecting a single event rather than a periodic
  event count.
- **Resolution:** SabiQuant `Frequency::Once.period()` and
  `events_per_year()` return `None`. The generator and Rust test assert the
  exact QuantLib 0Y sentinel; all ten periodic rows are compared directly to
  their actual QuantLib length and unit.
- **Requirements:** FR-PER-03, INV-PER-4; DOMAIN_MODEL absence semantics.
- **Sources consulted:** FS-PER; actual public `Period(Once)` construction.

## D9-PERIOD-ORDER — 2026-10-11

- **Class:** D9, intentional divergence.
- **Fixture set:** `TIME-PERIOD-QUANTLIB-COMPARISONS`, 2,040 records bearing
  this ID among 4,096 ordered pairs.
- **Observed:** QuantLib can order some calendar-month versus calendar-day
  pairs and throws for ambiguous pairs, such as 1M versus 30D. SabiQuant
  returns an incomparable result for every pair from different normalized
  families. All four zero periods normalize to the day family; zero versus
  a nonzero calendar-month/year period is therefore also incomparable.
- **Resolution:** Preserve the specified family-only partial order. Compare
  signed month counts inside Months/Years and signed day counts inside
  Days/Weeks. Canonical zero keeps ordering consistent with normalized
  equality: equal zero values have the same comparisons to every third value.
  Tests verify equality substitution, reverse comparisons, and transitivity.
  The other 2,056 pairs agree with QuantLib; the generator rejects any
  unexpected comparison difference within one normalized family.
- **Requirements:** FR-PER-02, INV-PER-1/2; OQ-10.
- **Sources consulted:** FS-PER's documented partial-order policy; actual
  public Period equality and ordering operations in the committed rows.

## D5-ISDA-SAME-YEAR-CANCELLATION — 2026-10-11

- **Class:** D5, reference comparison tolerance/conditioning issue.
- **Fixture:** `CORE-DAYCOUNTS`, exactly 8,372 rows; only Actual/Actual ISDA
  with endpoints in the same calendar year and reference/oracle distance over
  eight ULP. Other rows still use the original comparison policy.
- **Observed example:** 1901-01-29 → 1901-01-31 has exact fraction `2/365`.
  The correctly rounded oracle is `0.005479452054794521`; actual QuantLib
  1.43 returns `0.005479452054794498` (26 ULP). The maximum distance observed
  in this fixture set is 230 ULP.
- **Evidence:** the pinned reference's
  [ISDA implementation](https://github.com/lballabio/QuantLib/blob/6b57206e04598f092efee66e3b367efc84771995/ql/time/daycounters/actualactual.cpp)
  combines a negative whole-year term and two partial-year fractions even
  when both dates share a year. Subtraction can cancel most significant bits.
  This is an analysis of reference behavior, not copied implementation code.
- **Bound and resolution:** same-year magnitudes of each quotient and each
  intermediate are at most one. Two divisions and two additions contribute
  at most `4u` absolute error; rounding the rational oracle adds at most `u`.
  Here `u = epsilon/2`. Use a conservative reference-to-oracle absolute bound
  `3*epsilon = 6u`, only for the exact witnessed same-year class. The generator
  rejects results outside that bound. This is a conditioning-based reference
  comparison, **not** a relaxation of SabiQuant's TOL-E2 contract: Rust stays
  within four ULP of the rational oracle and same-year cases match at zero ULP.
- **Tests:** reference values, known count, same-year condition, distance over
  eight ULP, precise absolute bound, and Rust/oracle accuracy are all checked.

## D8-ISDA-MAX-YEAR-INTERMEDIATE — 2026-10-11

- **Class:** D8, confirmed reference range issue.
- **Fixture:** `CORE-DAYCOUNTS`, exactly 452 rows. Both distinct endpoints
  are in 2199 and valid within QuantLib's documented date range.
- **Observed behavior:** actual QuantLib 1.43 rejects the fraction while
  trying to construct an intermediate January date in 2200. Day counts still
  agree; the independent mathematical year fraction is finite.
- **Evidence:** primary ISDA definitions and the daily weighted rational oracle
  define these intervals; the pinned reference implementation above constructs
  a following-year boundary. The runtime exception names year 2200.
- **Resolution:** SabiQuant keeps the finite oracle-correct result. A fixture
  marks the specific reference failure; every other unexpected exception is
  rejected. Tests require distinct 2199 endpoints, exact error context and
  preserved Rust oracle accuracy. No upstream message is sent.

## D9-CALENDAR-EMPTY-MONTH / D9-CALENDAR-SEARCH-BOUND — 2026-10-11

- **Class:** D9, specified intentional calendar safety behavior.
- **Fixture:** `CALENDAR-BOUNDED-DIVERGENCES`, exactly five finite synthetic
  scenarios. No never-terminating reference search is executed.
- **Empty month:** with every February 2024 date a holiday, Modified Following,
  Modified Preceding and Half-Month Modified Following move 2024-02-15 outside
  its month in actual QuantLib. SabiQuant returns
  `calendar.no_business_day_in_month`, as FS-BDC requires.
- **Search bound:** Following/Preceding on a calendar closed for 367 consecutive
  candidates find a finite reference result 367 days away. SabiQuant returns
  `calendar.search_limit` at 366 rejected candidates.
- **Resolution:** preserve the specified month/search bounds. The generator
  and tests require the exact scenario definitions and actual reference dates;
  unexpected discrepancy labels or inputs fail. Other calendar fixtures carry
  no accepted divergence.

## D9 contextual signed/zero day counts — 2026-10-11

The attested `EXTENDED-DAYCOUNTS` fixture contains exactly 62,138 rows.
All differences below are required domain policies, not widened tolerance:

| Label | Rows | Predicate and meaning |
|---|---:|---|
| `D9-BOND-SIGNED` | 241 | Reversed Bond Basis endpoints; normalize forward then negate. 2023-01-15→03-31 is 76 here and in QuantLib, reverse is −76 here versus −75 directly in QuantLib |
| `D9-30E-ISDA-SIGNED` | 103 | Reversed endpoints with fixed explicit termination context; direct reference formula differs from negated forward formula |
| `D9-30E-ISDA-ZERO` | 10 | Equal endpoints are the last February day and explicit termination. Rust returns zero; direct QuantLib returns day-of-month minus 30 (−2 or −1) |
| `D9-B252-SIGNED` | 3,375 | Reversed business-day counts differ by exactly one because direct fixed start-inclusive/end-exclusive calendar counting swaps included endpoints. Saturday 2024-01-06→Monday 01-08 is zero here in both directions; QuantLib reverse is −1 |

The generator independently checks QuantLib against the direct source/endpoint
formula, then checks SabiQuant's signed policy against exact rational expectations.
The integration suite checks exact discrepancy counts, predicates and zero ULP
from the oracle. Every unclassified difference fails generation/testing.
`TARGET-MEMBERSHIP` separately labels Verified versus Projected source coverage;
reference agreement cannot convert projection to historical verification.
Pre-1999-01-04 market queries remain invalid despite reference extrapolation.

## D9-ICMA-ZERO-COVERAGE — 2026-10-11

Exactly two rows in `icma-strict-coverage.fixture`, IDs
`ICMA-ZERO-BEFORE` and `ICMA-ZERO-AFTER`, use Semiannual references
2024-01-01–2024-07-01 and equal accrual endpoints 2023-12-31 or 2024-07-02.
Actual QuantLib 1.43 with explicit references returns positive zero. SabiQuant
validates its owned reference context first and returns `icma.out_of_coverage`.
This intentional OQ-21 domain policy preserves reference validity even for
zero accrual; equal queries at the supplied outer boundary dates remain valid.
Tests require the precise IDs, dates, reference zero and structured error.
No other ICMA discrepancy is accepted by this label.

## D9 core schedule contract distinctions — 2026-10-12

The byte-attested 46-row `M7-SCHEDULE-REFERENCE-PROBES` set records public
QuantLib 1.43 outputs, separately from independent civil-date expectations.
`tests/schedule_reference.rs` exercises 34 locally supported inputs: 25 exact
shared date/regularity results and the nine explicitly scoped distinctions
below. Seven specialized rules and five explicit-coupon inputs are deferred,
not silently accepted or counted as local parity.

| Label | Rows | Exact scope and local result |
|---|---:|---|
| `D9-SCHEDULE-CALENDAR-EOM` | 4 | EOM Preserve with a WeekendsOnly business-EOM seed that is not calendar EOM: Forward Jan 30 or Backward May 29, Unadjusted/Following. Local calendar-only activation stays inactive; exact raw/adjusted goldens differ from the reference while regularity agrees |
| `D9-SCHEDULE-COLLISION` | 4 | Jan 9..13 daily WeekendsOnly Following/Preceding/ModifiedFollowing and Apr 2..7 daily TARGET Following. The reference returns fewer dates; local construction rejects with exact offending boundary index, raw pair and equal adjusted date |
| `D9-SCHEDULE-ZERO-METADATA` | 1 | `RULE-ZERO` has the same two dates. Reference reports 0 Years/regular; local reports absent recurrence metadata and NonPeriodic, without inferring a frequency |

These are adopted local choices in [M7_LOCAL_READINESS](../docs/M7_LOCAL_READINESS.md),
not newly asserted market conventions. Strict TARGET whole-span validation
retains the separately specified source-coverage policy; reference equality
cannot waive that guard. Unused out-of-range stopping is tested against the
supported civil domain, not claimed as a measured reference discrepancy.
Every unclassified mismatch in the 34 supported rows fails verification;
no epsilon or date deduplication absorbs it. No upstream message is sent.
