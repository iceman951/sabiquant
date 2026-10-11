# M3 day-count extensions: local implementation contract

| Field | Value |
|---|---|
| Recorded | 2026-10-11 |
| Status | Staged local implementation and validation; not whole-M3 acceptance |
| Scope | Bond Basis; termination-explicit 30E/360 ISDA; explicit-calendar Business/252 |
| Decisions | OQ-14 signed forward intervals; required convention context; preserved TARGET validity and coverage |
| Related | [Primary-source review](M3_DAYCOUNT_RESEARCH.md), [FS-DCC](REQUIREMENTS.md#9-fs-dcc--day-count-conventions), [TARGET contract](M3_TARGET_CONTRACT.md) |

These extensions implement previously specified, source-verified rules. US
February/EOM behavior and the original AFB February roll remain outside this
implementation. Readiness, cross-platform evidence, and research decisions
for the rest of M3 remain separate gates.

## Definitions and required context

**Bond Basis** is `DayCount::Thirty360BondBasis`. On an ascending interval,
change a starting 31st to 30; then change an ending 31st to 30 only when the
adjusted starting day is 30. February has no special adjustment. Divide
`360ΔY + 30ΔM + D′end − D′start` by 360. The original primary definition is
[ISDA 2006 §4.16(f), printed pages 11–12](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=23).
Its canonical identifier is `30/360 (Bond Basis)`; exact aliases are
`30/360 ISDA` and `360/360`. Bare `30/360` remains ambiguous.

**ISDA 30E/360** is `ThirtyE360Isda::new(termination_date)`. Adjust a starting
31st or last day of February to 30. Adjust an ending 31st to 30; adjust an
ending last day of February to 30 unless it equals the explicit termination
date. The denominator is 360. This is distinct from European 30E/360.
[ISDA 2006 §4.16(h), printed page 13](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=25).
Reversal retains the same contractual termination date. The date is never
inferred from an accrual endpoint. Equal endpoints return positive zero.

**Business/252** is `Business252::new(Calendar)` or
`Business252::with_target(TargetCalendar)`. Its numerator counts business
dates in the chronological forward interval, including its start and excluding
its end; its denominator is 252. B3 explicitly documents the business-day/252
basis and inclusive-start/exclusive-end DI accrual.
[B3 basis table](https://www.b3.com.br/en_us/products-and-services/trading/interest-rates/di-training-program/02-interest-rates.htm),
[B3 DI methodology](https://www.b3.com.br/pt_br/market-data-e-indices/indices/indices-de-segmentos-e-setoriais/di/metodologia-de-calcudo-acumulado-de-di/).
There is no default market calendar or global count cache.

The original ISDA publication was read through a TD Securities-hosted copy;
this identifies the definition reviewed, not an independently attested current
amendment set. The formulas are paraphrased and independently implemented.

## Signed intervals and guarded sources

The delegated OQ-14 decision computes an ascending interval, then negates its
integer count and fraction for reversed endpoints. A zero integer count always
produces positive floating-point zero. This is a SabiQuant API rule, rather
than a claim that applying a market formula directly backwards gives the same
answer. Bond Basis is legitimately nonadditive; tests preserve that fact.

Business/252 owns a private choice of rule calendar or guarded TARGET value.
`calendar()` returns `Option<&Calendar>` and `target_calendar()` returns
`Option<&TargetCalendar>`: exactly one is present. A guarded market calendar
cannot escape through the plain-calendar accessor. Its explicit projection
policy is retained.

`Business252::day_count` returns `Result<i32, DayCountError>`. A guarded
source validates both endpoints, even the excluded end or equal endpoints,
and every counted date. An interval whose endpoints are verified can still
fail when its interior crosses an unverified gap. Therefore equal dates return
zero **within the source's allowed domain**; equal dates outside TARGET
validity or strict coverage return an error. Overrides and joint composition
cannot extend that domain.

`DayCountError::Calendar` has stable kind `daycount.calendar` and exposes its
`CalendarError` source, including a further nested `DateError` where present.
`DayCountError::Value` retains kind `daycount.value`. Year fractions propagate
calendar errors before numeric validation and use one checked integer division
(TOL-E1, zero ULP from the correctly rounded quotient).

## Reference differences are explicit

Actual public QuantLib 1.43 calls confirmed the following cases. The
[committed reference fixture](../validation/fixtures/market/extended-daycounts.fixture)
also records the actual and independent expected outputs; discrepancies are
accepted only by their precise classifications.

| Scenario | SabiQuant | QuantLib 1.43 | Classification |
|---|---|---|---|
| Bond Basis, 2023-03-31→2023-01-15 | −76 days, negated from the forward 76 | −75 days from direct backward adjustments | `D9-BOND-SIGNED` |
| ISDA 30E/360, equal 2023-02-28 endpoints and termination | 0 days | −2 days | `D9-30E-ISDA-ZERO` |
| ISDA 30E/360, equal 2024-02-29 endpoints and termination | 0 days | −1 day | `D9-30E-ISDA-ZERO` |
| ISDA 30E/360, reversed endpoints | Negated chronological forward formula with unchanged termination | Direct directional formula | `D9-30E-ISDA-SIGNED`, only when different |
| Weekends-only Business/252, 2024-01-08→2024-01-06 | 0 days, from the same forward half-open interval | −1 day, counting the opposite included endpoint | `D9-B252-SIGNED` |

The 62,138-row fixture contains 241 Bond signed differences, 103 ISDA signed
differences, 10 ISDA equal-termination differences, and 3,375 Business/252
signed differences. These are counts for this recorded scenario set, not
universal statistics. TARGET reference projections use an explicit projection
policy and do not become source-verified merely by agreeing with QuantLib.

## Validation scope

[daycount_extended.rs](../tests/daycount_extended.rs) checks independently
computed integer civil formulas and an exact i128 weighted-day ISDA rational:
common-year days contribute 366 numerator units and leap-year days 365, over
the common denominator `365×366`. No global epsilon is used.

- TS-DCC-07: every ordered pair in 1999–2002 (2,134,521 pairs) and 2099–2101
  (1,199,025 pairs), each against all five scalar conventions.
- Every supported year's twelve-by-twelve month-end pairs, with three explicit
  termination contexts: 4,319,568 ISDA checks, plus Bond Basis checks.
- Thirty thousand deterministic full-range pairs, dates in years 1 and 9999,
  Gregorian leap-century boundaries, February maturity cases, signed zero,
  aliases, unknown/context-dependent names, and typed source chains.
- All 128 weekend masks, with and without explicit holiday/business overrides,
  against every ordered pair in a 29-day window; null, closed, joint, and
  full-range calendars; equal and reversed endpoint policies.
- Strict TARGET coverage, excluded endpoints, equal dates, interior coverage
  gaps, explicit projection, immutable overrides, and joint-calendar guards.

[market_validation.rs](../tests/market_validation.rs) separately checks actual
QuantLib fixture outputs and the narrowly allowed differences. Reproduction
uses an externally installed QuantLib 1.43 wheel with its recorded SHA-256;
the Rust crate still declares no dependencies:

```sh
python3 validation/generator/market_fixtures.py \
  --quantlib-wheel /path/to/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl --check
cargo test --offline --test daycount_extended --test market_validation
cargo clippy --offline --all-targets -- -D warnings
```

The wheel identity and generator hashes are recorded in fixture metadata. The
wheel's precise source commit is not independently attested; the repository's
source baseline is identified separately. Native coverage and platform checks
are recorded by the enclosing milestone validation, rather than inferred from
the number of cases here.
