# M3 Local Implementation Record

| Field | Value |
|---|---|
| Date | 2026-10-11 |
| Authority | The owner's instruction to choose the next work and continue autonomously with the broadest feasible meaningful tests |
| Scope | Staged local calendars/business-day conventions, Easter/TARGET and eight day-count conventions |
| Prerequisite | M2 local debug/release, exhaustive, oracle/differential and lint/doc checks pass ([M2_VALIDATION](M2_VALIDATION.md)) |
| Scheduling exception | Local engineering continues while M2 external CI/review is pending. Neither M2 nor M3 is marked Complete from local evidence alone |
| External writes | The user chose local-only work after automatic review rejected the M2 push; no repository upload is authorized |

## Core batch decisions

The agent makes these engineering choices under the delegated local-work
authority. They are not represented as individually reviewed owner decisions.

- Implement immutable Null, weekends-only, custom and joint calendars; date
  overrides produce new values. Reject empty joint calendars and cap nested
  composition depth to avoid input-triggered stack overflow.
- A custom calendar may have all seven weekend days. It is a valid closed
  calendar for classification/composition, and explicit business-date overrides
  can open selected days. This clarifies the earlier construction restriction;
  operations that need business days return typed errors. Structural rejection
  must never hide a reachable explicit business-day override.
- Bound a search at 366 rejected candidates (OQ-19); test 365/366/367 and range
  endpoints. Define `add_business_days(date, 0)` as identity; zero-period
  `advance` still applies its explicit convention.
- Implement the seven specified business-day conventions. Nearest ties favor
  the following business day. Lists have inclusive endpoints and reject
  reversed bounds; signed counting honors both inclusion flags.
- For `advance` with Unadjusted and end-of-month enabled, calendar end-of-month
  is the only activation condition. A business-end start that is not calendar
  end remains clamped and unadjusted. An actual QuantLib fixture exposed an
  ambiguity in FR-CAL-05: with holidays on 2024-03-29 and 2024-04-01,
  2024-03-28 minus two months must be 2024-01-28, not 2024-01-31. The
  specification is corrected and a regression case is required.
- Implement Actual/360, Actual/365 Fixed, Actual/Actual ISDA and European
  30E/360 from the cited primary definitions. Reversed year fractions are the
  negative forward result (OQ-14), equal dates produce positive zero, and
  day-count/floating-point exactness classes remain explicit.
- Keep day-count identifiers precise. Generic labels such as `Actual/Actual`,
  `Actual/365` and `30/360` are rejected with ambiguity information.

## Exclusions and evidence gaps

The extension batch implements Western Gregorian Easter, a guarded TARGET
pilot, Bond Basis, termination-sensitive 30E/360 ISDA, Business/252 and
explicit-period Actual/Actual ICMA. Contracts and source evidence are in
[M3_TARGET_CONTRACT](M3_TARGET_CONTRACT.md),
[M3_DAYCOUNT_EXTENSIONS](M3_DAYCOUNT_EXTENSIONS.md) and
[M3_ICMA_CONTRACT](M3_ICMA_CONTRACT.md).

The following additional choices are made under the same delegated authority:

- OQ-17: TARGET euro settlement/value dates only. Production validity begins
  1999-01-04; no pre-launch operational history is invented.
- OQ-18: strict source coverage by default, with explicit `AllowProjection`.
  Verified intervals are 1999-01-04–2002-12-31 and 2026-01-01–2028-12-31.
  Every market query and visited candidate remains checked through overrides,
  joints and Business/252. Matching a reference never promotes projected years.
- OQ-14: integer counts and fractions normalize ascending endpoints and
  negate the forward result when reversed. Equal intervals return positive
  zero within their required context's domain. This intentionally differs from
  direct QuantLib Bond Basis, 30E/360 ISDA and Business/252 formulas.
- OQ-21: ICMA owns explicit ordered contiguous determination periods and a
  periodic frequency. No schedule, reference boundary or frequency is inferred.
  Zero-length fractions still require reference coverage.
- The stateless `DayCount` enum remains `Copy`; conventions requiring state
  use separate types whose constructors require it. Business/252 owns either
  a plain `Calendar` or guarded `TargetCalendar`, with no default market/cache.
- OQ-20: implement the eight source-resolved H1 conventions locally. US 30/360
  and Actual/Actual AFB remain Under Research and excluded from this batch:
  modern US February/EOM qualification and AFB February backward rolling are
  not resolved by the available primary publications (OQ-15/16).

TARGET maintenance is an explicit release gate: a calendar maintainer must be
appointed before release, review new annual ECB publications, record sources
and date coverage, and release corrections as behavior changes. Today's source
review does not appoint a person or establish a continuing maintenance service.

## Validation and completion gates

Follow [M3_TEST_PLAN](M3_TEST_PLAN.md): enumerate small calendar definitions,
independent civil/business-day counters, pathological search inputs, all
conventions and inclusion combinations, exact rational oracles and actual
QuantLib 1.43 reference values with specific discrepancy predicates. Retain
the M2 regression suite.

Record each implemented batch and actual check result before committing it.
Do not mark full M3 complete or claim market coverage from this partial batch.
Primary-platform CI, review and the remaining M3 scope/source decisions still
apply before milestone acceptance. No publication is authorized by this record.
