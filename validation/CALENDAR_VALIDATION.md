# Staged local core-calendar validation

This batch covers immutable null, weekends-only, custom and joint calendars,
seven adjustments, advancement, explicit business-day addition, inclusive
lists, counting and business month ends. It implements part of M3 locally;
it does not establish M3 completion, TARGET/Easter support, source verification
for market calendars, or external CI approval.

Eight schema-1 fixtures contain **75,227 actual QuantLib 1.43 records**:
20,454 membership queries; 15,141 adjustments; 23,520 advancements; 12,096
endpoint counts; 2,163 month-end queries; 56 lists; 1,792 weekend-mask queries;
and five exactly identified finite pathological divergences. Ordinary results
were independently checked using Python date/set membership and bounded day
scans before writing. No fixture discrepancy is silently allowed.
The synthetic holiday sets are scenario inputs, not market-calendar data.

The Rust suite additionally checks all 128 weekend masks across 100,352
forward/reversed/equal endpoint combinations; immutable dated overrides on a
fully closed weekly calendar; exact 365/366/367 rejected-candidate boundaries
and counter resets; empty-business months; date-range failures; extreme
periods; Nearest with only one representable direction; and invalid lists.
`add_business_days(d, 0)` is identity, while zero-period `advance` applies its
convention. These distinct operations are not miscompared as one reference
behavior. Closed calendars are valid definitions for queries/composition;
searches fail with typed errors unless a reachable explicit business override
exists.

Generation caught and rejected an incorrect FS-CAL EOM proposal. For a custom
calendar closing 2024-03-29 and 2024-04-01, 2024-03-28 is business month end.
QuantLib's `advance(-2 Months, Unadjusted, EOM=true)` returns **2024-01-28**,
not 2024-01-31. Unadjusted EOM preservation activates only from calendar
month end; adjusted conventions use business month end. The corrected
contract and an explicit regression test retain this finding.

The generator requires the exact downloaded wheel and verifies every
installed `QuantLib/` module file, including its native binary, against it.
Headers record wheel SHA-256, generator SHA-256, runtime version, Python,
platform and actual generation timestamp. The source baseline commit is
separately labeled, without claiming the wheel binary attests that commit.
Reference calendars are fresh bespoke instances or reset primitives, so
mutable QuantLib state cannot leak into later scenarios.

```sh
/tmp/sabiquant-validation-venv/bin/python validation/generator/calendar_fixtures.py --quantlib-wheel /tmp/sabiquant-validation-wheels/*.whl
/tmp/sabiquant-validation-venv/bin/python validation/generator/calendar_fixtures.py --quantlib-wheel /tmp/sabiquant-validation-wheels/*.whl --check
cargo test --offline --test calendar_validation
cargo clippy --offline --test calendar_validation -- -D warnings
```

Committed reference results cover the documented ordinary calendars and dense
leap/century, half-month, month/year boundary, holiday-overlap and tie windows.
Five finite reference cases explicitly record outside-month results for three
modified conventions on closed February 2024 and out-of-bound results for two
long closure searches. Their exact inputs, actual reference dates and required
Rust error kinds are asserted. Other pathological failures are tested against
the specified Rust contract; the generator avoids unbounded QuantLib searches.
Source-backed market calendars, algorithmic Easter, full-cycle conventions,
and platform CI remain separate M3 acceptance work.
