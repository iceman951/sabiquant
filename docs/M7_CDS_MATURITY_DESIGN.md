# Proposed M7 CDS quoted-maturity design

Independent source and arithmetic review, 2026-10-14 UTC (2026-10-15
Asia/Bangkok). This is a proposed local analytical definition for the next
integrated CDS tranche. It records completed read-only checks; it does not
claim a production implementation, newly collected reference calls, financial
acceptance, or completion of M7. The existing
[seven-family research](M7_SPECIAL_RULES_RESEARCH.md),
[attested roll observations](M7_ROLL_REFERENCE_PROBES.md), and
[CDS source addendum](M7_CDS_SOURCE_ADDENDUM.md) retain their evidential limits.
No QuantLib C++ implementation was inspected, copied or translated.

## Scope and primary evidence

The resolver takes an explicit trade date, stored quoted contract tenor, and
`CdsMaturityConvention::{Quarterly2009,Semiannual2015}`. Its result concerns
unadjusted quoted maturity only. There is no inferred regime, implicit today,
calendar, business-day adjustment, settlement date, protection start, coupon
frequency or premium-leg construction.

The historical financial scope is standard single-name corporate and sovereign
CDS. The final [ISDA December 10, 2015 FAQ](https://www.isda.org/a/vGiDE/amend-single-name-on-the-run-frequency-faq-revised-as-of-12-10.pdf),
Q1–4, places the new rolls in March and September, keeps quarterly coupons,
and excludes loan and mortgage CDS. Q2's earlier-convention example switches
on June 20, 2015 to a five-year contract maturing September 20, 2020: equality
at the trade-date roll therefore selects the *next* quarter. Q10–11 retains
quarterly short maturities and supplies a zero-month matured state. Its table
also includes 1.25Y, 1.5Y and 1.75Y quotes. These can be represented here as
15M, 18M and 21M. The direct PDF request returned HTTP 403; the final dated
document's indexed primary text and table were inspected. This is not a
complete PDF or legal-document review.

The directly inspected [CDS Standard Model news](https://www.cdsmodel.com/news.html),
December 20, 2015 entry, records the new convention's 0M/3M/6M/9M/1Y through
5Y table. Its dated columns make June 20 and December 20 inclusive starts of
the matured 0M windows. It also gives December 22, 2015 + 5Y as December 20,
2020 under the new convention and March 20, 2021 under the earlier convention.
The news reports the operational switch on December 21, whereas the final
FAQ says December 20 go-live. Explicit regime selection preserves that
distinction without inventing a universal effective instant.

The formulas below are the local mathematical generalization of those
published examples and boundaries. Applying them throughout years 0001–9999,
supporting every representable nonnegative quarter-multiple quote, typed
errors/outcomes, and stored-input identity are engineering choices. They do
not assert that these contracts were traded throughout that proleptic range
or that every mathematical tenor is liquid or market-standard.

## Stored tenor domain and definition identity

Inspect the original `Period::length()` and `Period::unit()` before any
normalization. Accept nonnegative stored Months divisible by three and
nonnegative stored Years. Convert Years to twelve months in `i64`. Reject
Days, Weeks, negative lengths and non-quarter-multiple Months with a typed
tenor error. Both 0M and 0Y are valid; 0D and 0W remain invalid even though
normalized `Period` equality can identify zero values across units.

Keep the exact trade date, original length/unit and explicit convention
inspectable in the resolved definition, including a matured outcome. Equal
resolved dates do not erase different caller definitions. In particular,
1Y/12M, 5Y/60M and 0Y/0M calculate alike but retain distinct stored metadata.
Definition Eq/Hash must compare the stored length and unit explicitly because
`Period` equality itself normalizes equivalent aliases. No resource budget
belongs to financial definition identity.

The stored `i32` domain needs no absolute-value operation: reject a negative
length before arithmetic, including `i32::MIN`. The largest positive Years
value converts to 25,769,803,764 months, safely within `i64`; a required date
can still be out of range. `i32::MAX` Months is not divisible by three and
fails the tenor domain first; 2,147,483,646 Months is a valid mathematical
tenor whose resolved date is out of range. Do not conflate these failures.

## Exact maturity definitions

Let the trade be `(year, month, day)`. Its signed civil month coordinate is
`m = 12 * (year - 1) + month - 1`, so January 0001 has coordinate zero.
Define `D(k)` abstractly as the twentieth of civil month coordinate `k`;
this notation does not require constructing a library `Date`.

For `Quarterly2009`, choose the first March/June/September/December twentieth
*strictly after* the trade date. Its month coordinate is the base `b`.
The outcome is `D(b + quoted_months)`. Consequently 0M is an ordinary future
maturity in this convention, subject to the final date's representable range.

For `Semiannual2015`, choose the last March/September twentieth on or before
the trade date. Its month coordinate plus three is the base `b`, identifying
a June/December twentieth. If the quote is zero and `D(b) <= trade`, return
the typed `Matured` outcome. Otherwise return `D(b + quoted_months)`.
Only zero can mature within this accepted tenor domain: adding at least three
months always places the result strictly after the trade.

An equivalent constant-time coordinate calculation is:

```text
cutoff = m - (day < 20 ? 1 : 0)
step = 3 for Quarterly2009, 6 for Semiannual2015
b = cutoff - (cutoff - 2).rem_euclid(step) + 3
matured = quoted_months == 0 && (b, 20) <= (m, day)
target = b + quoted_months
```

The offset two denotes March in zero-based month coordinates; six-month
spacing selects March/September. Euclidean remainder is required for the
negative coordinates near `Date::MIN`. The matured predicate is false for
every Quarterly2009 input; it must not be used to conceal range errors.

The following consequences distinguish all four quarter boundaries. `Y` is
the trade year, `q` the nonnegative quoted months, and all dates are twentieths.

| Trade date | Quarterly2009 base before adding q | Semiannual2015 base before adding q | Semiannual2015 0M |
|---|---|---|---|
| March 19 | March Y | December Y−1 | Matured |
| March 20 or 21 | June Y | June Y | June Y |
| June 19 | June Y | June Y | June Y |
| June 20 or 21 | September Y | June Y | Matured |
| September 19 | September Y | June Y | Matured |
| September 20 or 21 | December Y | December Y | December Y |
| December 19 | December Y | December Y | December Y |
| December 20 or 21 | March Y+1 | December Y | Matured |

For example, the new convention on March 20, 2016 maps 0/3/6/9/12/60M to
June 2016, September 2016, December 2016, March 2017, June 2017 and June 2021.
On June 20, the positive-quote results stay the same while 0M becomes Matured.
Thus an integer-year quote ends in June/December, while short and fractional
year quotes can end in March/September. No supplied explicit maturity is
reclassified by this quoted-tenor resolver.

## Required date construction and error cause

Return a typed outcome equivalent to `Date(Date)` or `Matured`, with the
caller definition retained separately or by its enclosing immutable value.
Do not encode Matured as a null/sentinel date, and do not require a
Date-valued historical roll anchor or base in the outcome or provenance.
Both may be outside the civil range even when the requested result is valid.

Calculate the matured predicate using the signed coordinate/day pair first.
For a non-matured result, calculate the final target month in `i64`, then call
`trade.checked_add_months(target - m)`. Preserve its `DateError` as the cause
of the typed maturity-resolution error. This associates failure with the
caller trade date and the displacement to the actual required target month.
After success, use that returned date's year/month to construct day 20.
Its temporary day-of-month clamping cannot change the target year/month, and
day 20 exists in every month. Do not repeatedly add a tenor, manufacture an
intermediate anchor `Date`, or report a failure for an unused adjacent roll.

Examples that the production tests must retain:

| Trade and quote | Convention | Required outcome |
|---|---|---|
| 0001-01-01 + 0M | Semiannual2015 | Matured; the abstract base is year 0000 December |
| 0001-01-01 + 3M | Semiannual2015 | 0001-03-20; no intermediate Date in year 0000 |
| 0001-03-19 + 3M | Semiannual2015 | 0001-03-20 |
| 0001-01-01 + 0M | Quarterly2009 | 0001-03-20 |
| 9999-06-20 + 6M | Semiannual2015 | 9999-12-20 |
| 9999-06-20 + 6M | Quarterly2009 | Required final date out of range |
| 9999-12-19 + 0M | Either | 9999-12-20 |
| 9999-12-20 or Date::MAX + 0M | Semiannual2015 | Matured |
| 9999-12-20 or Date::MAX + 0M | Quarterly2009 | Required final date out of range |

Validation order is stored tenor domain, coordinate selection, typed matured
outcome, required final date construction. Domain errors must not disappear
behind a zero-tenor or range shortcut. Resolution errors retain trade date,
stored tenor, convention, and the actual final-target `DateError` cause.
Closed convention parsing/display and immutable value/error traits should
follow the established local API conventions; exact public names and stable
error kinds are to be pinned when adopting the integrated contract.

## Completed independent checks and their limits

The review executed a one-off Python arithmetic check against the unchanged
[roll fixture](../validation/fixtures/schedule/roll-probes.fixture), without
loading SabiQuant or running QuantLib. It parsed the existing JSON maturity
observations and independently selected candidate twentieths by enumerating
months in the preceding/current/following civil years.

| Existing public `cdsMaturity` observations | Count | Review result |
|---|---:|---|
| CDS, quoted 0M/3M/6M/12M | 48 | 48/48 agree with Quarterly2009 |
| CDS2015, quoted 0M/3M/6M/12M | 48 | 48/48 agree; recorded null Date maps to local Matured |
| OldCDS, positive quoted controls | 36 | Preserved, outside the two-regime comparison |
| OldCDS, 0M errors | 12 | Preserved, outside the two-regime comparison |
| Total observations retained in the original artifact | 144 | 96/96 modern comparisons; no claim of 144 matches |

The fixture SHA-256 observed in that run was
`88e2bacdc1ab999c1979f19c5d2f4fa485f3492cceab040cca6cc927e9edf35a`,
matching the existing report. The fixture's original collection timestamp is
`2026-10-13T17:48:07Z`; its prior attestation and reproduction details remain
in [M7_ROLL_REFERENCE_PROBES.md](M7_ROLL_REFERENCE_PROBES.md). This review did
not recollect or reattest that runtime. In particular, 9M/60M, stored aliases,
huge tenors, and years 0001/9999 are not added reference-runtime observations.

The same one-off check visited every one of the **146,097 valid dates** from
2000-01-01 through 2399-12-31. For both regimes, the coordinate formula agreed
with independently enumerated candidate anchors on every date: **292,194
anchor comparisons**, zero mismatches. For each date/regime it additionally
checked that 3M/6M/9M/12M/60M resolves strictly after trade, **1,460,970
comparisons**, zero failures. Separate arithmetic evaluations covered the
MIN/MAX examples above and the signed i32-to-i64 month conversion bounds.
These are completed mathematical review checks, not production tests or
financial evidence for proleptic dates. The ephemeral check was not saved as
a generator or fixture; production validation must retain its own independently
reviewable oracle and exact cases.

## Integrated next tranche and outstanding acceptance

The root implementation should adopt this resolver together with the modern
CDS date-plan contract, preserving the separation between quoted maturity,
already-resolved explicit maturity, requested date roles, contractual quarterly
dates, and adjusted payment dates. Date-plan calendars, full required coverage,
terminal inclusion/accrual semantics, resource bounds, structured failures,
and exact independent tests need their own explicit contract. A resolver
alone does not complete the integrated modern CDS tranche or approve reuse
of the ordinary raw-accrual coupon factory for a CDS premium leg.

OldCDS remains an explicit outstanding part of M7. The existing 29/30-day
first-roll observations are retained engineering evidence; this two-regime
resolver does not waive its historical source, start-role, equality-boundary,
date-plan or validation work. Full M7 and financial/owner/platform acceptance
remain governed by the existing project requirements.
