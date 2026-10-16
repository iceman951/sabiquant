# M3 US 30/360 and AFB: API and integration design

| Field | Value |
|---|---|
| Recorded | 2026-10-16, Asia/Bangkok |
| Status | Design proposal; neither an adopted financial contract nor implementation evidence |
| Baseline | Local branch `feat/m3-us-afb-daycounts`, commit `1c16daf` |
| Scope | Explicit US context, stateless AFB, numerical order, downstream composition, validation changes |
| Decision state | OQ-15 primary February/EOM qualification pending; official ISDA-published AFB interpretation inspected by the source reviewer, awaiting adopted contract |
| Related | [FS-DCC](REQUIREMENTS.md#9-fs-dcc--day-count-conventions), [decision register](IMPLEMENTATION_READINESS.md#5-decision-register), [earlier source review](M3_DAYCOUNT_RESEARCH.md), [current extensions](M3_DAYCOUNT_EXTENSIONS.md) |

The owner authorized the next local day-count tranche. This proposal records
the API and integration choices developed alongside independent
primary-source reviews. It does not select an unresolved financial
rule because a reference implementation happens to use it. An adopted contract
must identify the reviewed source, the exact rule and its scope, and any
remaining research or acceptance gate before production implementation begins.

While this design was being written, the source reviewer reported direct
inspection of all seven pages of ISDA's official
[*The Actual/Actual Day Count Fraction* (1999)](https://www.isda.org/a/pIJEE/The-Actual-Actual-Day-Count-Fraction-1999.pdf).
Its page 1, §2(c), explicitly describes the February 28-to-29 backward-year
treatment and the included-start/excluded-end interval. That resolves the
previous unread-scan gap for ISDA's published AFB interpretation. It does not
establish that the original 1994 AFB publication was retrieved. The final
financial contract and source-review record should preserve that distinction;
this document remains an implementation proposal.

## Public shape and explicit context

If the reviewed US February adjustments are qualified by a security's
end-of-month treatment, introduce `daycount::Thirty360Us` as a distinct immutable
context. A minimal proposed surface is:

```text
Thirty360Us::new(end_of_month_security: bool) -> Thirty360Us
Thirty360Us::end_of_month_security(self) -> bool
Thirty360Us::canonical_identifier(self) -> &'static str  // "30/360 (US)"
Thirty360Us::day_count(self, start: Date, end: Date) -> i32
Thirty360Us::year_fraction(self, start: Date, end: Date)
    -> Result<f64, DayCountError>
```

Both Boolean values are complete valid contexts. There is no `Default`, no
zero-argument constructor, no implicit security classification, and no
context-free `FromStr`. `Thirty360Us::new(true)` records an explicit caller
choice; it must not infer the flag from either calculation endpoint. Document
the parameter at the call site and show both values in examples. The scalar
`DayCount` enum must not gain a context-dropping `Thirty360Us` variant.

A dedicated closed two-choice enum is also sound if the adopted API prefers
named arguments at call sites. Its variants must describe the *security's US
February treatment*, not generic calendar advancement. That alternative adds
one public type and its identifier catalog. Reusing
[`calendar::EndOfMonth`](../src/calendar.rs) would be misleading: its existing
`Preserve`/`Ignore` values describe calendar/business month-end advancement,
which is not a security convention flag. The minimal Boolean proposal avoids
changing the meaning of that established type.

Derive `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, and `Hash` for the US context.
Its definition includes the explicit flag, so two contexts must remain unequal
when they have different flags, even when a particular interval gives the same
fraction. Equality is definition equality rather than output equivalence.
`canonical_identifier()` names the convention; its text does not encode the
complete context or promise value round-tripping.

If the primary review instead establishes an unconditional convention, record
that decision explicitly before choosing a stateless representation. Do not
implement an EOM switch merely to create a wider API than the adopted source
requires. Conversely, a flag-qualified source must not become an unconditional
scalar value to match QuantLib's API.

AFB has no external calendar, frequency, termination date, or reference-period
input. Subject to adopting the reviewed ISDA interpretation, add
`DayCount::ActualActualAfb`, with
canonical identifier `Actual/Actual (AFB)` and exact alias
`Actual/Actual (Euro)` as already specified in FS-DCC §9.6. No extra AFB context
type is necessary. `day_count` returns the actual signed civil-day difference;
the fractional-year algorithm is separate from that integer count.

The existing `DayCount` enum is non-exhaustive. Its `ALL` changes from five
values to six, and its canonical table, `Display`, `FromStr`, match arms and
round-trip self-check must agree. Exact, case-sensitive parsing without
trimming remains unchanged. `Actual/Actual`, `Act/Act`, and `ACT/ACT` remain
ambiguous, including after all three listed Actual/Actual candidates have an
implementation. `30/360` remains ambiguous. A scalar parse of `30/360 (US)`
still returns `IdentifierError::Unknown` when that convention requires context,
consistent with `30E/360 (ISDA)` and `Business/252` today.

## Signed intervals and finite integer bounds

Retain the adopted OQ-14 API policy: calculate a chronological forward
interval, then negate for reversed endpoints. Preserve the same US flag when
normalizing dates. An equal interval returns integer zero and floating-point
positive zero. A distinct interval that a 30/360 rule maps to zero also returns
positive floating-point zero. Do not manufacture negative zero by negating a
zero forward result without canonicalization.

Neither new convention introduces calendar validity or coverage. Every pair
of valid dates, from `Date::MIN` through `Date::MAX`, belongs to its domain.
There must be no predecessor of year 1 or successor of year 9999 used merely
to discover that no additional whole year or leap day is needed. A reverse
full-range call is the negation of the corresponding forward call, rather than
a separate directional market formula.

For US 30/360, adjusted day-of-month values remain small integers, and
`360Δyear + 30Δmonth + Δadjusted_day` fits `i32` over the supported range.
Use exact component arithmetic, followed by one `f64` division by 360. All
integers involved are exactly representable as binary64. If the adopted
flag-off formula reduces to existing Bond Basis, that is a useful exact
cross-check, but it must be established by the approved formula rather than
assumed from the API shape.

AFB actual days fit `i32`; whole years are at most 9,998. Its residual fraction
is determined by the adopted backward-year rule. Test the actual numerator,
whole-year count, residual endpoint and denominator independently before using
only the final binary64 output as evidence. AFB is not asserted additive or
monotonic merely because those properties hold for Actual/360 or ISDA.

## AFB complexity and a conditional constant-time derivation

NFR-11 currently requires stateless year fractions to be O(1). A literal loop
that rolls the end date backward one year at a time is O(number of years).
The supported 9,998-year limit is a useful hard bound, but it does not justify
calling that algorithm constant-time under the existing requirement. The
preferred implementation is a proven constant-time decomposition with a
separate iterative test oracle. If the adopted financial rule prevents that,
amend the complexity and downstream work-budget contract explicitly before
using an annual loop in production.

The following is an *engineering derivation*, not the adopted financial
contract. It applies if OQ-16 adopts the reviewed recurrence that clamps
a one-year-back move and replaces February 28 in a leap target year with
February 29:

1. For a non-February-28/29 end, every backward candidate has the end's same
   month and day. For an end on February 28 or 29, every candidate after at
   least one backward step is the last day of February in its target year.
   This follows by induction: a February 29 becomes February 28 in a common
   year, and either February day becomes February 29 in a leap target year.
2. Let `d = end.year - start.year`. If `d = 0`, no full backward year is
   eligible. Otherwise compare the candidate in `start.year` with `start`.
   If it is on or after `start`, take `N = d`; otherwise take `N = d - 1`.
   Candidates are strictly decreasing with target year, so no other search is
   necessary. If `N = 0`, retain the original end unchanged; do not apply a
   February transformation to the original endpoint.
3. The residual end is consequently in `start.year` or the following year.
   Its day-of-year follows from the selected target year's leap status and
   the original month/day. Compute residual days with day-of-year integers
   and the start year's length. This requires no intermediate `Date`
   construction and cannot request a date outside the public range.
4. Determine whether a February 29 lies in the residual *half-open* interval
   using at most these two years. Check actual membership: an excluded
   February-29 residual end does not by itself select 366. There is no need
   to construct a February 29 for a common year or a year-10000 sentinel.

This derivation must be checked against a separate repeated-year rollback
oracle with its own civil month table and ordinal conversion. In particular,
retaining the original endpoint for `N = 0` is essential when that endpoint is
February 28 in a leap year. If the source selects a different recurrence,
discard this candidate and derive the correct constant-time formula afresh.

## Numerical order and tolerance

US year fractions retain TOL-E1: one correctly rounded quotient of exact
integer day count and 360. Zero-ULP comparison is required against an exact
rational oracle rounded once. Multiple decimal operations or a blanket epsilon
would weaken the existing 30/360 contract.

The proposed AFB operation order is: compute `N`, residual day count `r`, and
denominator `B` entirely with integers; then compute `q = f64(r) / f64(B)` and
`fraction = f64(N) + q`. Apply the sign after that forward computation, and
canonicalize zero. This is one rounded quotient and at most one rounded
addition. Integer whole years must not be accumulated through repeated
floating-point additions, and annual subinterval fractions must not be summed
as a substitute for the backward decomposition.

The exact reference rational is `(N × B + r) / B`. Its numerator is small
enough to represent exactly as binary64 over this domain, so a single
independent correctly rounded division is an adequate final rational oracle.
An i128 or Python `Fraction` construction additionally keeps the discrete
derivation explicit. This oracle must not copy the production two-operation
evaluation and then call it exact.

Use the existing TOL-E2 limit of at most four ULP against that independently
rounded rational, with a written error argument for two nonnegative terms:
`N` is exact, the division has at most one rounding, and the final addition has
at most one rounding. No cancellation is introduced by reversal. If `N = 0`,
the computation reduces to one rounded division; if `r = 0`, the whole number
is exact. Measure the actual maximum ULP distance without changing the
acceptance limit after observing outputs. A different production evaluation
order requires its own recorded decision and matching numerical evidence.

No many-term Neumaier reduction is necessary for this fixed two-term AFB
decomposition. The existing [ICMA argument](M3_ICMA_CONTRACT.md#e2-justification-for-many-segments)
continues to apply to ICMA's variable number of overlaps. Update NP-09 and the
TOL-E2 scope in [NUMERICAL_CORRECTNESS](NUMERICAL_CORRECTNESS.md) so the broad
wording about compensated composite accrual accurately distinguishes ICMA
from fixed bounded ISDA/AFB evaluation. This does not alter M6 `exact_sum` or
M7 cash-flow reduction: they round supplied binary64 terms, not the underlying
financial rational formula or an exact dot product.

The existing `DayCountError::Value` numeric guard is sufficient for both
bounded formulas; valid date inputs cannot make their results non-finite.
Do not add an unreachable public `Date` error solely because an implementation
chooses a checked intermediate construction that the mathematical domain never
requires. Do not force impossible private states or invalid dates to claim
coverage of a finite-result guard.

## Rate and cash-flow composition

Add `AccrualConvention::Thirty360Us(Thirty360Us)` and
`From<Thirty360Us> for AccrualConvention`. Its `year_fraction` match delegates
to the complete context. AFB follows the existing
`AccrualConvention::Scalar(DayCount::ActualActualAfb)` path and requires no
additional wrapper variant. Preserve owned immutable context, definition-based
`Eq`/`Hash`, and `RateError::DayCount` propagation through the root error.

`InterestRate`, `FixedCouponRate`, `FixedRateCoupon` and `Leg` accept contexts
through existing `Into<AccrualConvention>` APIs. No separate convenience API
should default the US flag. Rate conversion must retain a supplied target US
context even when the converted rate or horizon is zero. The source and target
accrual conventions in equivalent-rate operations remain independent.

`AccrualWork::charge` currently charges one unit per scalar or
termination-explicit query, one plus intersecting ICMA periods, and one plus
absolute calendar-span days for Business/252. With the constant-time AFB
algorithm above, both AFB and US continue to charge one unit. Extend the
exhaustive cash-flow match with `AccrualConvention::Thirty360Us(_)` alongside
the existing constant-time forms. The
[cash-flow contract](M7_CASHFLOW_CONTRACT.md) must explicitly include US among
those forms rather than describing all non-scalar contexts as scanning work.

If production AFB is an annual loop instead, the existing one-unit scalar
charge becomes an inaccurate interval-work claim. A budget amendment must
charge its visited years before each query, or reclassify AFB as an explicit
bounded scan with an accurately documented charge. Do not silently leave the
cash-flow O(n + budget) claim unchanged while adding uncharged annual work.

Downstream tests must use independent expected times, coupon amounts and
factors. Calling the new `year_fraction` to generate all expected values would
test dispatch but could conceal a shared financial error. Preserve the
existing rule that zero rates, zero amounts and equal-date horizons still
perform required context and budget checks. Cash-flow accrued interest still
clips at raw accrual endpoints independently of adjusted payment dates.

## Regression and evidence changes

The baseline has 72 declared public structs/enums and 103 catalogued error
variants. Under the Boolean US proposal, the public type census becomes 73;
AFB adds a variant to an existing type. A dedicated US option enum would make
the census 74. Verify actual declared types against `tests/public_api.rs`,
rather than updating a printed number alone. Add the US context's Copy/value,
Send/Sync/static and Eq/Hash assertions. No new error variant is proposed, so
the error census remains 103 unless the adopted contract adds a genuinely
reachable new failure class.

`tests/context_hash.rs` currently builds eleven distinct contexts: five scalar
values and six existing contextual definitions. Adding AFB and two explicit US
contexts makes fourteen in that same test. Independently reconstructed equal
US values must retrieve the same map entry, while the two flag values remain
distinct. Avoid asserting that unequal contexts must have unequal hash
integers; only equal values are required to hash equally.

Known integration points requiring deliberate review are:

- `src/daycount.rs`: module sources, stateless identifier table, `ALL`,
  forward/equal/reverse behavior, documentation and numeric guard scope.
- `src/rates.rs`: imported context type, `AccrualConvention`, delegation and
  `From`. `src/cashflows.rs`: the exhaustive work-budget match.
- `tests/daycount_core.rs`: its five-value identifier table, unsupported AFB
  label, `ALL` checks, and loops whose historical source comment covers only
  the original four-value `CORE`. Keep that historical oracle scope honest.
- `tests/daycount_extended.rs`: hard-coded five-value census and continued
  rejection of US in the scalar parser. Its original five-convention oracle
  remains valid without pretending to cover AFB automatically.
- `tests/rates_properties.rs`: sampled `DayCount::ALL` gains AFB; this changes
  deterministic sample paths, so inspect generic properties for accidental
  assumptions about additivity or anniversary fractions. Context lists in
  rates and cash-flow suites must also include both US choices explicitly.
  Its concurrent owned-context test currently asserts 256 quotes from eight
  contexts and 32 compounding configurations. With AFB and both US choices
  actually added to that list, the corresponding census is 352, not 256.
- `tests/public_api.rs`, `tests/context_hash.rs`, and type inventories in
  architecture/readiness documentation: the observed current API surface.
- Requirements, readiness OQ-15/OQ-16 and QR-02/QR-04, numerical policy,
  README, feature matrix, roadmap and changelog: exact adopted scope and
  surviving source, financial-acceptance and native-platform gates.

Frozen existing generators and fixtures remain byte-for-byte unchanged.
Introduce separately named new day-count fixtures and generators, rather than
adding rows to historical market data and rewriting their recorded evidence.
Record generator and contract/source-review hashes, body hash, row census,
date domain, conventions, explicit US flag and exact rational representation.
Public QuantLib calls are a behavioral comparison only; preserve reference
wheel identity and distinguish it from an independently attested source build.
Never inspect or translate QuantLib's implementation.

The new reference classification must separate at least: compatible ascending
US flag-on rows, flag-off scope differences if applicable, signed-policy
differences from direct reverse 30/360 calculation, and AFB financial-rule
differences if the approved primary rule differs from the reference. Do not
allow a broad exception merely because the convention name matches. Classify
each actual discrepancy with specific endpoints, context and output.

## Proposed independent validation matrix

| Scope | Required evidence |
|---|---|
| Primary decision anchors | Actual quoted source locations and inspected examples; separate retrieved/inspected content from unread scan or secondary description; no source gap closed by reference agreement alone |
| US February classification | February 27/28/29 and March 1/30/31; common, leap, divisible-by-100 non-leap and divisible-by-400 leap years; each February adjustment trigger individually and in combination; both explicit flag values |
| US 31st ordering | Starting 28/29/30/31 against ending 30/31 and February ends; prove rule order against independent integer civil calculation, with cases where applying the ending rule too soon changes the answer |
| US broad discrete domain | All ordered pairs in a tractable multi-year century/leap window; all supported years' month-end pairs; deterministic full-range pairs and exact MIN/MAX endpoints; exact count and TOL-E1 fraction |
| US signed policy | Both directions of each primary anchor; equal endpoints; distinct endpoints mapping to zero; positive-zero bits; actual reference reverse discrepancies classified precisely |
| AFB rollback distinction | `2024-02-28` to `2025-02-28` and its reverse; starts and ends on February 28/29 in leap/common years; zero-year versus one-year decomposition; retain the candidate `1 + 1/365` only if the source decision adopts that rule |
| AFB residual denominator | February 29 exactly at the included start, strictly inside, exactly at the excluded residual end, and just outside; residual wholly before/after February; residual crossing a non-leap century |
| AFB full-year decomposition | Exact whole-year cases, one-day residuals, residual just below a rolling year, long spans containing many leap years; compare discrete `N`, residual ordinal and denominator to an independently iterative civil oracle |
| AFB cycle and range | Complete 400-year Gregorian anchor cycle with targeted offsets and month ends, multiple centuries, deterministic full-range pairs, year 1/year 9999 and MIN/MAX; no unbounded production scan or out-of-range sentinel |
| AFB exact rational | Construct `(N×B+r)/B` independently and round once; TOL-E2 ≤4 ULP; record maximum measured distance and exact/equal/reverse zero paths without widening the contract |
| API identity | Canonical/alias parsing, rejected case/whitespace/unknown spellings, preserved generic ambiguity candidates, scalar US rejection, constructor/accessor round-trip and independent Eq/Hash contexts |
| Rate composition | All existing compounding forms and periodic frequencies; positive/negative/zero quotes in the existing domain; equal and reversed horizons where accepted; direct factors and equivalent-rate conversion against independent times |
| Cash-flow composition | US flag-sensitive and AFB rollback-sensitive coupons, accrued fractions, raw/payment-date separation, NPV with independent discount context, work cap exactly sufficient/one short/zero, flow ordering and context retention |
| Structural checks | Complete public-type trait census; stable error catalog/source chains; fmt, strict Clippy, warning-denied docs, debug/release full suites, compile-only foreign targets clearly labelled, extracted local package verification |
| Reproducibility | New generators' `--check`, independent fixture-body census/hash, unchanged historical fixture inventory, deterministic repeated/concurrent calls; actual native coverage report with proof-excluded paths stated honestly |

The test matrix is a proposal, not a result count. Report actual executed
scenarios and the exact verified snapshot only after implementation and final
validation. Compilation for Linux or Windows is not native test execution, and
passing local tests does not by itself establish whole-M3 financial acceptance.
