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
