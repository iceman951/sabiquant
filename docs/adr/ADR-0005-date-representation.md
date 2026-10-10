# ADR-0005: Date and Calendar Representation

| Field | Value |
|---|---|
| ADR identifier | ADR-0005 |
| Title | Date and calendar representation |
| Status | **Proposed** |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M2 for dates (OQ-09, OQ-12); M3 for calendars (OQ-17, OQ-18, OQ-19) |

## Context

Dates and calendars underlie every financial computation. The standard
library has no civil-date type, no calendar arithmetic, and no time-zone
database (ADR-0002). QuantLib 1.43 stores dates as Excel-compatible serial
numbers, limited to 1901-01-01 to 2199-12-31. Its built-in calendars share
mutable implementation objects, so adding a holiday changes every instance in
the process (B-01, B-07). Holiday data needs ongoing maintenance and has
provenance and licensing considerations.

## Decision proposal

### Part A: dates

1. **Model.** A date is a day in the proleptic Gregorian calendar, with no
   time of day and no time zone (FS-DATE).
2. **Internal representation.** A single signed 32-bit day number counted from
   a fixed internal epoch (proposed: 1970-01-01). This gives O(1) comparison
   and differences, a total order, a 4-byte `Copy` value, and conversion to
   and from components using public-domain civil algorithms
   ([Hinnant, *chrono-Compatible Low-Level Date Algorithms*](http://howardhinnant.github.io/date_algorithms.html),
   "donated to the public domain"). The representation and epoch are
   **private** and may change without breaking the API.
3. **Supported range** ([OQ-09](../IMPLEMENTATION_READINESS.md#5-decision-register)):
   0001-01-01 to 9999-12-31 inclusive. Every operation whose result would fall
   outside it returns an error.
4. **Text.** ISO 8601 `YYYY-MM-DD` with four-digit years.
5. **No clock.** There is no "today" (ADR-0011).
6. **Interop.** Conversion to and from Excel/QuantLib serial numbers is
   **deferred** ([OQ-12](../IMPLEMENTATION_READINESS.md#5-decision-register)).
   If added, it is an explicit, separately named conversion that documents
   the 1900 leap-year quirk.

### Part B: calendars

1. **Immutable values.** A calendar is a value. "Adding a holiday" returns a
   new calendar. No process-wide registry and no shared mutable state.
2. **Composition.** Null, weekends-only, custom (weekend set plus explicit
   holidays), and joint (*join holidays* or *join business days*) calendars.
3. **Market calendars as rules plus data.** Recurring holidays are encoded as
   rules: fixed date, n-th weekday, Easter-relative, and observance or
   substitution, each with an effective-date range. One-off closures are
   explicit lists. Western Easter is computed algorithmically.
4. **Holiday data is part of the source.** Data is compiled into the crate.
   There is no runtime file loading (NFR-13), so behavior is deterministic. If
   size requires it later, large sets of calendars may go behind Cargo
   features.
5. **Provenance (required for every market calendar).** Fields: authoritative
   sources (publisher, URL), retrieval date, **verified coverage** (years
   checked against the source), rule effective ranges, known historical
   corrections, licence or terms notes, and last review date (FR-CAL-09).
6. **Coverage behavior** ([OQ-18](../IMPLEMENTATION_READINESS.md#5-decision-register)).
   Proposed: outside verified coverage, rules extrapolate deterministically,
   coverage can be queried, and documentation states that projected years are
   not verified. A strict option that returns errors outside coverage is the
   alternative.
7. **Search bounds** ([OQ-19](../IMPLEMENTATION_READINESS.md#5-decision-register)).
   Searches for business days are bounded. The proposed limit is 366
   consecutive non-business days, after which an error is returned.
8. **Pilot** ([OQ-17](../IMPLEMENTATION_READINESS.md#5-decision-register)).
   TARGET is the only holiday-aware calendar in M3. Its rules come from an
   authoritative ECB publication
   ([long-term calendar, 14 December 2000](https://www.ecb.europa.eu/press/pr/date/2000/html/pr001214_4.en.html)).
   Rules for years before 2002 must be verified from primary ECB sources.
9. **Maintenance.** Each market calendar has a named maintainer. Data is
   reviewed every year when the authority publishes the next year's holidays.
   Corrections are released as **behavior changes** in the changelog. A
   calendar without an active maintainer is marked as such and is not
   extended.
10. **No holiday datasets in M0.**

## Alternatives considered

| Alternative | Assessment |
|---|---|
| Date as packed (year, month, day) fields | Fast access to components, but arithmetic and differences need conversion. Rejected as the primary representation |
| Store both serial and components | Larger values and redundant invariants. Rejected |
| QuantLib/Excel-compatible serial as the representation | Couples the core to a spreadsheet quirk and to the 1901–2199 range. Rejected; interop is a conversion only |
| Range 1901–2199 (QuantLib) | Too narrow for some long-dated and historical uses. Rejected |
| Full 32-bit range of days | Allows absurd dates and complicates text formats and year-based rules. Rejected |
| Mutable calendars with global holiday registration (QuantLib) | Breaks determinism and thread safety. Rejected (ADR-0011) |
| Load holiday data from files at runtime | I/O in the library, environment-dependent behavior. Rejected for H1 |
| Bundle many market calendars from the start | No maintenance capacity; provenance not established. Rejected |

## Advantages

- Exact, fast, compact dates with a clearly documented range.
- Calendars are thread-safe, reproducible, and easy to reason about.
- Holiday data is auditable, with provenance and stated coverage.

## Disadvantages

- More code than wrapping an existing date library, which ADR-0002 does not
  allow anyway.
- Users who want to tweak a calendar get a new value rather than mutating a
  shared one. That is a different workflow from QuantLib.
- Coverage semantics add API surface.

## Consequences

- Differential tests are limited to QuantLib's date range.
- Calendar equality is defined by content (rules plus data). Two
  independently built calendars with the same definition are equal.
- Releases may contain calendar-data behavior changes, so release notes must
  call them out (ADR-0009).

## Risks

- R-03: holiday-data errors and the maintenance burden. Mitigated by one pilot
  calendar, provenance, and annual reviews.
- Wrong rules for historical years. Mitigated by verified coverage and
  primary sources.

## Reconsideration triggers

- Demand for intraday timestamps or time zones (a new ADR; possibly a
  separate type).
- Holiday-data release cadence starts to diverge from code (see ADR-0004
  option C).
- A credible maintenance plan exists for additional calendars, which allows
  them in.

## Related requirements and documents

FS-DATE, FS-CAL, FR-CAL-03, FR-CAL-09, NFR-03, NFR-13;
[REQUIREMENTS §5](../REQUIREMENTS.md#5-fs-date--dates), [§7](../REQUIREMENTS.md#7-fs-cal--calendars);
[LICENSING_AND_ATTRIBUTION §6](../LICENSING_AND_ATTRIBUTION.md#6-reference-data-and-third-party-publications);
[ADR-0011](ADR-0011-explicit-state.md).
