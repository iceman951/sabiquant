# M3 Calendar Research

This records the pre-implementation research snapshot. Subsequent delegated
decisions and implemented contracts are in [M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)
and [M3_EXTENSION_VALIDATION](M3_EXTENSION_VALIDATION.md).

| Field | Value |
|---|---|
| Reviewed / sources retrieved | 2026-10-11, Asia/Bangkok |
| Scope | FS-CAL, FS-BDC and ADR-0005 Part B; OQ-17/18/19 |
| Status | Research and implementation proposals only; no M3 code, fixtures or completed-milestone claim |
| Authority | Next-step preparation under the owner's instruction to continue autonomously with broad meaningful test coverage; decisions below remain proposed |

The primary ECB evidence supports TARGET's distinct 1999, 2000 and 2001
calendars. It does **not** justify describing QuantLib's 1998 calendar as
historical TARGET operation. The pilot should model euro settlement/value
dates, not every national banking holiday or every modern TARGET service.

## Primary-source evidence

All entries were opened or searched on 2026-10-11. Rules below are paraphrased;
no third-party source code or holiday dataset is copied.

| Source / publication date | Evidence checked | Scope and limit |
|---|---|---|
| ECB, [TARGET testing completed](https://www.ecb.europa.eu/press/pr/date/1998/html/pr981217.en.html), 1998-12-17; [Successful launch of TARGET](https://www.ecb.europa.eu/press/pr/date/1999/html/pr990104_1.en.html), 1999-01-04 | Testing ended in December 1998; production began on 1999-01-04 at 07:00 ECB time | No historical TARGET operational calendar in 1998. QuantLib's 1998-12-31 exception is reference behavior, not verified operational history |
| ECB, [Year 2000 — Closing of TARGET on 31 December 1999](https://www.ecb.europa.eu/press/pr/date/1999/html/pr990331.en.html), 1999-03-31 | 1999-12-31 was an exceptional closure of interlinking and all connected domestic euro RTGS systems | Establishes the one-off 1999 closure; not an annual New Year's Eve rule |
| ECB, [TARGET closing days in 2000](https://www.ecb.europa.eu/press/pr/date/1999/html/pr990715_1.en.html), 1999-07-15 | 1999 closures were weekends, New Year's Day, Christmas Day and 31 December; Good Friday and Easter Monday had actual payment traffic. For 2000, weekends plus 1 January, Good Friday, Easter Monday, 1 May, 25 December and 26 December became euro-market non-settlement days | Verifies the 1999/2000 rule change. Some national systems could perform limited domestic activity in 2000; the pilot must not claim universal national-system closure |
| ECB, [TARGET closing days in 2001](https://www.ecb.europa.eu/press/pr/date/2000/html/pr000525_2.en.html), 2000-05-25 | 2001 retained the 2000 schedule and added 31 December for euro conversion; no cross-border payments on closing days | Verifies 2001-12-31. Limited domestic-operation exceptions existed on some closing days; these are outside the pilot |
| ECB, [Long-term calendar for TARGET closing days](https://www.ecb.europa.eu/press/pr/date/2000/html/pr001214_4.en.html), 2000-12-14 | From 2002 until further notice: weekends plus 1 January, Western Good Friday, Western Easter Monday, 1 May, 25 December and 26 December | Establishes the effective recurring rule; does not independently check every future year or every later exceptional change |
| ECB, [ECB public holidays](https://www.ecb.europa.eu/ecb/contacts/working-hours/html/index.en.html), live page retrieved 2026-10-11 | The six days marked as TARGET closures are listed explicitly for 2026, 2027 and 2028; Good Friday/Easter Monday are 2026-04-03/06, 2027-03-26/29 and 2028-04-14/17 | Use only TARGET-marked entries. ECB office holidays such as Ascension, 24 December and 31 December are not additional TARGET closures |
| ECB, [Infoguide, April 2025, Part 1 Fundamentals](https://www.ecb.europa.eu/paym/target/consolidation/profuse/shared/pdf/2025.APR_p1_fundamentals.en.pdf), §2.6 | Euro CLM, RTGS and T2S Cash retain the six-day schedule; Danish-krone settlement has a different calendar, and T2S can process free-of-payment transactions on 1 May | Corroborates the euro scope and service/currency distinctions. No claim for TIPS, DKK, securities free-of-payment or all TARGET services |

The ECB press releases permit reproduction with source acknowledgement.
The implementation should retain source attribution and independently encode
facts and rules; the current live page and Infoguide should have their terms
recorded before any substantial text or table reproduction.

## Coverage proposed for the pilot

| Dates / years | Evidence status in this research | Proposed treatment |
|---|---|---|
| Before 1999 | TARGET production had not launched; QuantLib extrapolates and encodes a 1998 exception | Outside market-rule validity; reject historical TARGET queries. A separate reference-only test may document this intentional divergence |
| 1999 | Annual scheduled closing rules and year-end exception verified from ECB releases | Include published year rules, with launch date 1999-01-04 separately recorded; 1–3 January must not be described as operating production days |
| 2000–2001 | Annual scheduled euro-market non-settlement rules verified from ECB releases | Include differentiated effective rules and 2001 year-end closure |
| 2002 | Long-term rule effective from this year is source-backed | Include the recurring rule; independently generate and inspect the dated holiday fixture before marking implementation verified |
| 2003–2025 | Recurring-rule inference is supported, including a 2025 euro-service corroboration; annual exceptional changes were not exhaustively reviewed | Projected / not individually verified in this research; do not imply a continuous source-checked history |
| 2026–2028 | Explicit six-date annual lists checked on the ECB page | Candidate dated reference coverage; snapshot and generate fixtures before promoting implementation validation status |
| 2029–9999 | Long-term rule extrapolation only | Projected, never verified merely because computus is deterministic |

“Source-backed rules” and “a tested implementation verified against a dated
fixture” are separate statuses. No implemented calendar has the latter status
yet. Coverage is a set of intervals, not a single min/max enclosing gaps.
There is no observed-day substitution in the listed TARGET schedules: a
weekend holiday does not itself create a Monday closure. This interpretation
should be made explicit in the fixture specification and checked against a
dated publication before asserting a particular Monday is source-verified.

## Proposed agent decisions

| Item | Proposal | Required specification clarification |
|---|---|---|
| OQ-17 | TARGET euro settlement/value-date calendar only, alongside null, weekends-only, custom and joint calendars | Name the modeled market meaning; exclude pre-1999 production history, domestic exceptions, DKK and unrelated TARGET services |
| OQ-18 | Strict source-verified coverage by default; permit deterministic projection only through an explicit policy selected when creating the immutable calendar | This chooses the strict alternative to the current ADR proposal. Queries expose verified/projected status; projection never extends market-rule validity before 1999. Joint calendars propagate member errors rather than hide unverified coverage through short-circuiting |
| OQ-19 | Stop after 366 rejected consecutive non-business candidates in either search direction | Precise proposed boundary: 365 closed candidates followed by a business day succeeds; the 366th closed candidate returns `no_business_day`, without inspecting candidate 367. Advancement resets the counter after each business day; range failure takes precedence when the next required date cannot be represented |
| Modified conventions | Require a business result in the original month, otherwise a typed no-business-day-in-month error | Spell out boundary behavior when the initially preferred direction leaves the date range but the fallback direction has a valid business day in that month |
| Nearest | Tie goes to Following, as specified; a representable business candidate may be used when the other direction leaves the date range | Specify how a coverage error or exhausted search in one direction affects the other; do not silently select a farther result while an unverified closer candidate is possible |
| Immutable content | Canonical weekend sets and deduplicated explicit holidays; removing a holiday does not remove a weekend unless an explicit business-day override is a separately approved feature | Clarify whether “remove holiday” suppresses a built-in recurring rule, a weekend, or only an explicit holiday; equality must follow documented content semantics |

Null/weekends-only/custom calendars have explicit user-defined rules rather
than historical market provenance. A custom calendar must retain at least one
business weekday, but a joint join-holidays calendar can still have none
because its members' weekend sets collectively cover all seven days. Searches
must therefore remain bounded even when every member is independently valid.

## Tests to prepare before implementation acceptance

- Exhaustively classify every date for null and weekends-only calendars over
  the supported range; enumerate all 127 permitted custom weekend sets and
  both joint rules over complete weeks. Include complementary members whose
  join-holidays calendar has no business weekday.
- Build source fixtures for all days of 1999–2002 and 2026–2028. Test each
  fixed/Easter-relative closure and its neighbors, 1999 Good Friday/Easter
  Monday being open, and 2000 versus 2001 versus 2002 year-end behavior.
  Distinguish weekend masking from the presence of a holiday rule: 1 May and
  26 December 1999 both fall on weekends and cannot alone prove that the
  later recurring rules were active.
- Test TARGET's 1998-12-31 reference divergence and strict invalidity before
  inception. Coverage tests include each verified-interval endpoint, gaps,
  explicit projections and both member orders in joint-calendar queries.
- Validate Gregorian computus independently for every supported year and
  constrain Easter to 22 March–25 April. Source-marked 2026–2028 Easter
  neighbors provide independent anchors; QuantLib's table is an additional
  differential oracle only within 1901–2199.
- Enumerate all seven adjustments across multi-year leap/century windows and
  every month boundary, the 14th/15th/16th, equal-distance Nearest ties,
  empty business months and MIN/MAX. Check identity, idempotence, directional
  bounds and same-month results where the specification requires them.
- Enumerate ±business-day counts from both business and holiday starts;
  zero adjustment, week addition, month/year clamping, EOM on/off and
  Unadjusted EOM. Count every pair in small windows using all four endpoint
  inclusion combinations, reversed intervals and equal dates.
- Deliberately construct 365/366/367-day closure runs in both directions,
  several separated runs during advancement, fully closed months, duplicate
  and overlapping holidays, leap-day holidays, and immutable add/remove
  operations. Assert exact error kind and input context, not just failure.
- Run pinned QuantLib 1.43 differential fixtures only after independently
  specifying inputs and classifying historical coverage, mutability,
  modified-month and bounded-search divergences. No blanket compatibility
  claim follows from matching one window.

## Questions still requiring resolution

1. Name a TARGET maintainer and annual review process; source retrieval today
   does not create ongoing maintenance capacity.
2. Review intervening annual/operating schedules and historical corrections
   before adding 2003–2025 to verified coverage. Define the fixture snapshot
   and terms/provenance record for live pages.
3. Record OQ-17/18/19 and ADR-0005 Part B decisions, plus the search and
   boundary clarifications above, before production calendar implementation.
4. Complete the separate M3 day-count research and readiness decisions
   OQ-14/15/16/20/21. This calendar note neither settles those conventions nor
   establishes M3 completion or authorization beyond next-step research.
