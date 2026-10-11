# M3 test plan

**Planning only — 2026-10-11.** Apply the M3 readiness gate in
[ROADMAP](ROADMAP.md#m3--financial-conventions) before implementation; this
plan does not claim M3 completion.

Resolve [OQ-14–21](IMPLEMENTATION_READINESS.md#5-decision-register): reversed
day counts/year fractions; US February/EOM rules; AFB counting back; TARGET
selection; coverage behavior; precise search-bound inclusivity; convention
scope; explicit ICMA determination boundaries/frequency. Research R-01–03
remains material. REQUIREMENTS currently has no `FR-DCC` IDs; use FS-DCC
§9.1–9.8 and F-DCC-01–10 until traceability IDs are assigned.

## Required scenario matrix

| Contract and scenario IDs | Enumeration and assertions |
|---|---|
| FR-CAL-01/02; TS-CAL-01/02; INV-CAL-1/2 | Null/weekends-only over a full 400-year weekday/leap cycle. Custom calendars: all 127 permitted weekend masks; duplicate, overlapping, weekend and 29-February holidays; business start/end and sorted lists with both weekend options. |
| FR-CAL-02/03; TS-CAL-03; INV-CAL-3 | Both joint rules over all boolean member combinations and overlapping holiday sets. Empty joint/all-weekend custom constructors fail. Adding/removing holidays leaves originals, clones and other instances unchanged. |
| FR-BDC-01/02; TS-BDC-01–05 | Every date in 1999–2002 and 2099–2101 × seven conventions × approved calendars. Business-day identity, idempotence, Following/Preceding monotonicity, modified same-month results; 14/15/16, month/year ends, Nearest ties. Canonical identifiers, aliases and ambiguity errors. |
| FR-CAL-04–06; TS-CAL-04 | Counts −31…31 from business/holiday starts; zero counts; all units/conventions/EOM flags. Days count business days; weeks count calendar weeks. Test calendar-EOM versus business-EOM and Unadjusted explicitly. Extreme counts/range edges return typed errors without panic. |
| FR-CAL-07; TS-CAL-05; INV-CAL-6 | All four endpoint options, both directions and equal dates. Exhaustive pairs in 28-day windows around leap/year/month boundaries; longer deterministic pairs. Independently scan calendar membership into prefix counts. |
| FR-CAL-08–10; TS-CAL-06 | Easter for years 1–9999 against independently sourced corrected Gauss computus and known dates; assert Sunday/March22–April25; QuantLib overlap 1901–2199. TARGET after approval: holiday lists, effective boundaries, coverage metadata and outside-coverage behavior. |
| FS-CAL/BDC failures; TS-CAL-07 | Non-business runs of 365/366/367 days in both directions; first business day immediately at/beyond the decided limit; empty-business months; MIN/MAX searches; nested-calendar errors retain their source/context. |
| FS-DCC §9.2–9.4/9.7; TS-DCC-01/02/04/05/07/08 | All ordered pairs in both windows; all selected 30/360 variants, 27/28/29/30/31 and termination-date equality. Exact integer numerators; E1 bitwise fractions; ISDA independently weighted daily totals. |
| FS-DCC §9.5/9.6; TS-DCC-02/03/08 | ICMA regular/short/long first/final stubs, long multi-period spans; gaps, overlaps, unsorted/empty boundaries and invalid frequencies. AFB dense February pairs, whole-year boundaries, approved counting-back regressions. Required-parameter omission gets compile-fail documentation tests. |
| FS-DCC §9.8; TS-DCC-06–08 | Business/252 with explicit null/weekends/custom/joint calendars; dense closures, zero/reversed intervals and calendar error propagation. No default calendar or observable global cache. |

## Independent evidence and practical cost

Use Python `datetime`, declarative holiday/30-day decision tables and
`fractions.Fraction`; do not translate QuantLib or Rust implementation code.
ISDA's oracle assigns each individual day weight `1/365` or `1/366`, with
exact weighted prefix sums, rather than copying the production year-split
formula. ICMA intersects supplied determination intervals before summing
exact fractions. Round once: TOL-E1 is 0 ULP; proposed TOL-E2 is ≤4 ULP
versus the oracle and ≤8 versus QuantLib. Re-justify E2 for long ICMA sums;
record maximum errors/worst inputs before considering tolerance changes.

The two pair windows contain 2,134,521 + 1,199,025 = 3,333,546 ordered pairs.
Shard by convention/window; use O(1) prefix oracles, avoid quadratic storage
and interval rescanning. Commit compact boundary fixtures; external comparisons
can stream. Add full-range endpoint samples. Measure debug/release cost; make
exhaustive release shards required, retaining quick boundary/error feedback.

Do not assume month advancement is associative/invertible, all conventions
are monotone, every anniversary accrues one year, or 30/360 is additive.
Reversal follows OQ-14; additivity applies only to the FS-DCC validity table.

## Source and differential gates

Required sources: ECB's [2000 TARGET calendar](https://www.ecb.europa.eu/press/pr/date/2000/html/pr001214_4.en.html)
plus primary evidence for historical exceptions; manually checked
[ISDA 1999 examples](https://www.isda.org/a/pIJEE/The-Actual-Actual-Day-Count-Fraction-1999.pdf)
and resolution of QR-01; licensed ISDA definitions where accessible; ICMA PMH
Appendix A5; primary SIFMA, AFB and Brazilian Business/252 publications.
Missing evidence remains Under Research and excluded, never inferred from
QuantLib agreement.

Extend the attested QuantLib 1.43 generator through public `Calendar` queries,
`adjust`, `advance`, `businessDaysBetween`, lists and named `DayCounter`
methods. Use fresh reference calendars per scenario to prevent mutable-state
contamination. Supply identical explicit ICMA inputs and verify the reference
interprets the intended frequency.

D9 requires exact scenario IDs and witnessed predicates: wider-range results
outside 1901–2199; a finite closure exceeding the approved search limit;
modified adjustment of a proven empty-business month; approved reversed
30/360 inputs whose direct reference result differs from negated-forward
evaluation. Record actual reference results/errors and independent expected
results separately. No blanket D9 allowance covers US, AFB, ICMA or coverage
disagreements; resolve their OQs first and approve exact exceptional rows.

## Implementation order and exit evidence

1. Close decisions/source gaps and freeze error contracts.
2. Immutable primitive/custom/joint calendars and membership/list tests.
3. Seven adjustments, bounded searches, counts and advancement.
4. Actual/360, Actual/365F, ISDA and sourced 30/360 variants.
5. Explicit ICMA, approved AFB/Business252, sourced TARGET.
6. Run exhaustive/oracle/differential shards and platform CI; publish evidence.

Every in-scope scenario and error variant needs evidence. Run coverage to
find missed branches, without treating a percentage as correctness. Require
formatting, strict Clippy, doctests and dependency-policy checks; identical
discrete results on Linux/macOS/Windows. Classify every discrepancy and state
remaining source, domain, runtime or platform limitations before closing M3.
