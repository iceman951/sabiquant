# TARGET Calendar Contract — Local M3 Prototype

This is a reviewable local implementation contract, not M3 acceptance or a
claim of completed cross-platform validation. TARGET models euro settlement
and value dates. ECB office holidays, domestic RTGS exceptions, Danish-krone
settlement and unrelated TARGET services are outside its scope.

## Validity, coverage and policy

`TargetCalendar::new()` requires verified source coverage. The prototype's
`Default` implementation was removed during local M5 API review to honor the
no-default-calendar rule; choose the named constructor explicitly.
`TargetCalendar::with_policy(TargetPolicy::AllowProjection)` explicitly admits
deterministic projected dates. Both policies reject dates before the
[ECB's production launch on 1999-01-04](https://www.ecb.europa.eu/press/pr/date/1999/html/pr990104_1.en.html).
The wrapper does not expose a plain `Calendar` that could bypass this guard.

| Interval | Coverage | Strict policy |
|---|---|---|
| Before 1999-01-04 | OutsideValidity | Error, also under projection |
| 1999-01-04 through 2002-12-31 | Verified | Allowed |
| 2003-01-01 through 2025-12-31 | Projected | Error |
| 2026-01-01 through 2028-12-31 | Verified | Allowed |
| 2029-01-01 through 9999-12-31 | Projected | Error |

These are disjoint source intervals. A long-term recurring rule is not an
annual operational review of every intervening year. The `coverage(date)`
query returns the same status under either policy, including rejected dates.
Compatibility evidence separately records a named QuantLib version, scenario
set, discrepancy classifications and test outcomes.

## Encoded closing rules

Saturday and Sunday are weekends. There are no substitute Monday holidays
merely because a fixed closing date falls on a weekend.

| Effective calendar years | Additional scheduled closing dates | Primary evidence |
|---|---|---|
| 1999 | 1 January, 25 December, exceptional 31 December; the January API validity begins on the 4th | [ECB 1999/2000 calendar](https://www.ecb.europa.eu/press/pr/date/1999/html/pr990715_1.en.html), [year-end closure decision](https://www.ecb.europa.eu/press/pr/date/1999/html/pr990331.en.html) |
| 2000 | 1 January, Western Good Friday, Western Easter Monday, 1 May, 25 December, 26 December | [ECB 2000 calendar decision](https://www.ecb.europa.eu/press/pr/date/1999/html/pr990715_1.en.html) |
| 2001 | Same six recurring dates, plus 31 December | [ECB 2001 calendar decision](https://www.ecb.europa.eu/press/pr/date/2000/html/pr000525_2.en.html) |
| From 2002 | The six recurring dates, without a recurring 31 December closure | [ECB long-term decision](https://www.ecb.europa.eu/press/pr/date/2000/html/pr001214_4.en.html) |
| 2026–2028 dated anchors | 2026 Good Friday/Easter Monday: 3/6 April; 2027: 26/29 March; 2028: 14/17 April | [ECB TARGET-marked annual holidays](https://www.ecb.europa.eu/ecb/contacts/working-hours/html/index.en.html) |

Western Easter is independently implemented from USNO computus and checked
against an independent Gauss formula throughout years 1–9999. The pre-1582
extension is mathematical, not a historical observance claim. QuantLib's
pre-inception TARGET calendar, including 1998-12-31, is intentionally outside
this market-validity contract.

## Operations cannot extend source coverage

- Business, holiday and weekend classification return typed errors outside
  the selected policy's coverage. Unadjusted adjustment and zero counts still
  validate their input.
- Search and advancement validate each required candidate. Modified fallback
  and Nearest must preserve coverage errors; another successful direction must
  not hide a query that required unverified data. Nearest conservatively
  completes both directional searches before selecting a result; therefore
  strict 2026-01-01 returns an unverified-coverage error for 2025-12-31 even
  though the verified 2026-01-02 candidate would win a distance tie. This
  documented policy avoids silently ignoring a searched coverage boundary.
- Counting checks both endpoints even when excluded, plus its relevant
  interval. Verified endpoints cannot bridge the 2003–2025 gap. Pure TARGET
  counting validates the interval before using weekday blocks and unique
  per-year closures; immutable overrides contribute exact corrections.
  Joint definitions use checked per-date classification.
- Inclusive date lists validate endpoints and every candidate. A reversed
  valid interval is a typed range error.
- `with_holiday` and `without_holiday` create immutable overrides, including
  explicit weekend business days. They do not alter source coverage.
  Declaring an override does not itself reject an unsupported date; a later
  query still rejects it. `joint_with` retains the TARGET member's guard under either joint rule,
  even if another member determines the Boolean result by itself.
- The inception month can encounter outside-validity candidates when searching
  from 1 January. It must report that limitation rather than classify days
  before the system's launch as historical production dates.

## Provenance and maintenance

Primary sources were reviewed on 2026-10-11; their titles, URLs, intervals and
review date are available as read-only constants. Press-release source
acknowledgements are retained; rules are independently encoded, not copied
from QuantLib. Further research evidence is in
[M3_CALENDAR_RESEARCH](M3_CALENDAR_RESEARCH.md).

The required role is **SabiQuant calendar maintainer**, currently an
appointment requirement rather than an assigned person. Before a market
calendar release, the project owner must appoint the maintainer, establish
annual review of the ECB's dated schedules and change notices, and record any
coverage extension or historical correction with source and behavior-change
notes. A future year is never promoted to verified by extrapolation alone.
User authorization for local work does not imply individual owner acceptance
of this contract or completion of M3's remaining conventions and CI gates.
