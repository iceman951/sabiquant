# Bounded QuantLib 1.43 schedule observations

Snapshot: 2026-10-12. This is local M7 preparation requested after
[schedule research](M7_LOCAL_RESEARCH.md). It records public reference-runtime
outputs, not financial requirements, independent expected values, SabiQuant
compatibility, M7 implementation authorization or milestone acceptance.
No Rust source/tests or existing requirement documents changed for these
probes. They are separate from the M6 implementation and evidence counts.

## Artifact, provenance and execution bounds

[schedule_probes.py](../validation/generator/schedule_probes.py) generated
[probes.fixture](../validation/fixtures/schedule/probes.fixture): **46 rows**,
all successful public constructor/accessor observations. The fixture has
schema-v1 metadata and input columns, with a deterministic JSON field for
the complete public observation. It contains no SabiQuant expected-result
column. The existing std-only Rust fixture reader accepts all 46 records.

The runner verifies the established wheel SHA-256 before importing QuantLib,
then compares every shipped module file with the installed file bytes:
`QuantLib/QuantLib.py`, `QuantLib/__init__.py` and
`QuantLib/_QuantLib.abi3.so`. Each isolated worker repeats the attestation and
checks the public runtime version. The module-manifest digest is over sorted
module paths and their individual SHA-256 hashes. This attests the installed
binary/module bytes to this wheel; it does not establish the wheel's exact
source commit or inspect its internal implementation.

| Provenance item | Recorded value |
|---|---|
| Reference runtime | QuantLib 1.43, reported by each worker |
| External Python / native platform | Python 3.9.6 / macOS ARM64 |
| Wheel | `quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl` |
| Wheel SHA-256 | `85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c` |
| Three-file module-manifest SHA-256 | `e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d` |
| Generator SHA-256 | `2c48705c96a6311922e8483cc47ae383f032555fd56f4c3f53567ca605725d6d` |
| Project source baseline | QuantLib v1.43 commit `6b57206e04598f092efee66e3b367efc84771995`; wheel commit not independently attested |

All input ranges are positive, wholly inside 2026 and at most 366 days.
Tenors are positive and finite; optional first/penultimate dates, when
present, are ordered and strictly interior. Calendars are Null, WeekendsOnly
or TARGET, without hostile all-closed overrides. Every scenario runs in a
fresh subprocess with Python isolated mode and a **10-second timeout**,
including its native constructor and accessor calls. Public outputs must
have 2–400 dates inside 2025–2027. A timeout or process/attestation/output-bound
failure aborts the run before writing a fixture. The output envelope is an
additional validation guard, not a substitute for the subprocess deadline.
An injected timeout check confirmed that aborting preserves the existing
fixture bytes.

The runner invokes `Schedule`, its dates/regularity/endpoints/metadata
inspectors, and public `Calendar` EOM, adjustment and advancement controls.
It never reads `schedule.cpp`, translates reference code, or derives dates
from an implementation-source algorithm. Optional dates use the reference
API's absence sentinel, while effective and termination dates are always
explicit; no scenario relies on an implicit valuation date.

## Anchored recurrence and directional residuals

The NullCalendar monthly controls from 2026-01-31 to 2026-05-31 use EOM off
and Unadjusted dates. Both Forward and Backward return
`01-31, 02-28, 03-31, 04-30, 05-31`, with all four intervals reporting regular.
March recovers day 31 after February's clamping. This is an observed result
consistent with an anchored lattice; it is not an assertion about hidden
reference implementation mechanics.

For a quarterly input from 2026-01-15 to 2026-10-31, Forward returns
`01-15, 04-15, 07-15, 10-15, 10-31`, reporting `R R R I`.
Backward returns `01-15, 01-31, 04-30, 07-31, 10-31`, reporting `I R R R`.
The direction changes the anchor and residual; those input ranges do not
produce interchangeable date sequences.

Here and below, `R` means the public `isRegular(index)` accessor returned
true, and `I` means false. These are recorded flags, not independently
validated definitions of coupon regularity. All abbreviated dates in the
tables are in 2026 unless a full year is shown.

| Explicit quarterly stub input | Returned dates | Returned flags |
|---|---|---|
| Backward, first coupon 01-31, range 01-15..10-31 | 01-15, 01-31, 04-30, 07-31, 10-31 | I R R R |
| Backward, first coupon 04-30, same range | 01-15, 04-30, 07-31, 10-31 | I R R |
| Forward, penultimate coupon 10-31, range 01-31..11-15 | 01-31, 04-30, 07-31, 10-31, 11-15 | R R R I |
| Forward, penultimate coupon 07-31, same range | 01-31, 04-30, 07-31, 11-15 | R R I |
| Forward, first 01-31 and penultimate 10-31, range 01-15..11-15 | 01-15, 01-31, 04-30, 07-31, 10-31, 11-15 | I R R R I |

## Calendar EOM, business EOM and direct advancement

For WeekendsOnly, public EOM inspection reports 2026-01-30 and 2026-05-29
as business EOM but not calendar EOM. Jan 31 and May 31 report both. The 16
EOM rows cross those seeds, both directions, EOM on/off and
Unadjusted/Following. They also record direct public calendar advancement
by signed 1, 2 and 4 months from the directional seed.

With Forward seed 2026-01-30, monthly termination 2026-06-30 and Unadjusted:
EOM off returns `01-30, 02-28, 03-30, 04-30, 05-30, 06-30`; EOM on returns
`01-30, 02-28, 03-31, 04-30, 05-31, 06-30`. Thus this Schedule probe's EOM
option affects a business-EOM seed that is not calendar EOM. In the same
EOM-on configuration, direct `Calendar.advance` controls return March 30
for +2 months and May 30 for +4 months. The public operations differ.

With EOM on and Following, the Schedule returns
`01-30, 03-02, 03-31, 04-30, 06-01, 06-30`. Direct calendar advancement
from the same seed returns Feb 27 for +1 month and May 29 for +4 months.
Schedule dates therefore cannot be specified by simply chaining this
calendar advancement API. Following also permits the observed EOM Schedule
boundaries to move into the following month.

Backward EOM-on Unadjusted from the May 29 business-EOM endpoint returns
`01-01, 01-31, 02-28, 03-31, 04-30, 05-29`, reporting `I R R R R`.
Direct -4-month calendar advancement returns Jan 29. The unadjusted terminal
May 29 is retained while interior month-end dates change. These endpoint
and seed distinctions need explicit SabiQuant schedule contracts; the
observations do not resolve which market roll should be adopted.

## Adjustment collisions and endpoint metadata

The WeekendsOnly daily Unadjusted control over Jan 9..13 returns five dates:
`01-09, 01-10, 01-11, 01-12, 01-13`, reporting `R R R R`.
The Following Schedule returns only `01-09, 01-12, 01-13`, reporting `R R`.
Its separate public adjustment control maps Jan 10, 11 and 12 all to Jan 12.
Preceding maps Jan 9, 10 and 11 to Jan 9 and produces the same three-date
Schedule; ModifiedFollowing does likewise for this input.

These observations establish that the returned adjusted date vector has
fewer boundaries than the Unadjusted control and still reports regular
intervals. They do not expose an original contractual boundary vector or
establish how discarded/colliding dates are represented internally. A future
SabiQuant collision-rejection policy would need an explicit documented
divergence rather than a claim of matching these results.

For monthly Jan 31..May 31 with ordinary Following, every terminal-control
row starts on Feb 2 and has middle dates Mar 2, Mar 31 and Apr 30. Only the
explicit terminal convention changes:

| Terminal convention | Public end date | Returned flags |
|---|---|---|
| Unadjusted | May 31 | R R R R |
| Following | June 1 | R R R R |
| Preceding | May 29 | R R R R |
| ModifiedFollowing | May 29 | R R R R |

The fixture records public start/end dates, output count, rule, tenor,
EOM, ordinary/terminal conventions, calendar name, regularity presence and
strict ordering. Input effective/termination dates remain separate columns;
they must not be confused with those returned adjusted endpoints.

The TARGET Following daily probe over Apr 2..7 returns just Apr 2 and Apr 7,
reporting one regular interval. Direct adjustment maps Apr 3, 4, 5, 6 and 7
to Apr 7. This is a reference-calendar observation in a small 2026 range,
not evidence that QuantLib implements SabiQuant's validity/verified-coverage
policy. The 7 Days and 1 Week Unadjusted controls over Jan 9..Feb 6 both
return Jan 9, 16, 23, 30 and Feb 6 in this scenario.

## All ten rule-family observations

Every row below uses explicit input start 2026-02-06, termination 2026-11-17,
3 Months, NullCalendar, Unadjusted ordinary and terminal conventions,
EOM off and absent explicit stub dates. The same bounded input provides
one observation per rule; it is not exhaustive rule validation.

| Rule | Returned dates | Returned flags |
|---|---|---|
| Forward | 02-06, 05-06, 08-06, 11-06, 11-17 | R R R I |
| Backward | 02-06, 02-17, 05-17, 08-17, 11-17 | I R R R |
| Zero | 02-06, 11-17 | R |
| ThirdWednesday | 02-06, 05-20, 08-19, 11-17 | R R I |
| ThirdWednesdayInclusive | 02-18, 05-20, 08-19, 11-18 | R R R |
| Twentieth | 02-06, 02-20, 05-20, 08-20, 11-20 | I R R R |
| TwentiethIMM | 02-06, 03-20, 06-20, 09-20, 12-20 | I R R R |
| OldCDS | 02-06, 03-20, 06-20, 09-20, 12-20 | I R R R |
| CDS | 2025-12-20, 03-20, 06-20, 09-20, 12-20 | R R R R |
| CDS2015 | 2025-12-20, 03-20, 06-20, 09-20, 12-20 | R R R R |

Zero reports public tenor **0 Years**, despite the input 3 Months. Its true
regularity flag does not imply a recurring coupon frequency. Inclusive
Wednesday and twentieth/CDS families move one or both returned endpoints;
the contractual input interval must remain distinguishable. Matching
CDS/CDS2015 or TwentiethIMM/OldCDS outputs for this case proves no general
equivalence. Regime boundaries, stub compatibility and market sources remain
open research work from M7_LOCAL_RESEARCH.

## Reproduction and limits of the evidence

Run externally; QuantLib is not a crate dependency or a Rust-test runtime:

```sh
/private/tmp/sabiquant-validation-venv/bin/python validation/generator/schedule_probes.py \
  --quantlib-wheel /private/tmp/sabiquant-validation-wheels/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl \
  --check
```

`--check` reattests all worker runtimes, repeats the bounded public probes
and compares every fixture byte while retaining the original generation
timestamp, platform and Python-version metadata. The wheel, module-manifest,
generator and public outputs must remain identical. No elapsed runtime or
unstable process identifier is serialized. Exact regeneration passed locally.

This adds **one reference fixture and 46 preparation records** to the
repository. It adds no M6 oracle rows and no Rust integration test count.
The observations help choose and test a future schedule contract; they do
not override [API principles](API_DESIGN_PRINCIPLES.md), accept a market
convention, or establish full compatibility for any rule family.
