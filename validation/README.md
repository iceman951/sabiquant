# External validation

The Rust crate and its tests depend only on the standard library. QuantLib
never runs during `cargo test`; the integration suite reads committed schema-1
fixtures. Python and QuantLib are external tools for regenerating evidence.
`validation/` is excluded from the published crate.

## M7 automatic OldCDS evidence

The [OldCDS contract](../docs/M7_OLD_CDS_CONTRACT.md) adds three fixtures.
The civil oracle has **162,644 rows**: every start day of 2000..2399 once,
with rotating ends and stored strides, plus distance, terminal, tenor-domain,
civil-extreme and exact-cap controls. Its generator scans civil days for each
quarter and enumerates interior twentieths. It agrees with the contract's
closed-form count on all 162,524 counted rows. Long schedules (67 rows above
1,024 boundaries, up to the civil maximum of 119,987) store an order-sensitive
checksum and run-length flags. The separate ICMA oracle has **10,363 rows**
with minimal hulls, exact Fraction accruals for 24,972 raw intervals and
whole spans, caps, civil causes and frequency-domain controls. Neither
generator read production code. See [the oracle report](../docs/M7_OLD_CDS_ORACLES.md).

The attested reference fixture has **1,199 rows**: Codex's 551 earlier
observations reproduced field for field, plus 648 new probes. Each row
records the public observation and an independent local-contract
classification. No row is unclassified. The Rust suite reclassifies every
row against the actual operation and also classifies the 98 frozen
roll-probe OldCDS rows (56 exact, 29 flags-only, 4 tenor-domain, 9 EOM/role
options). See the [probe report](../docs/M7_OLD_CDS_REFERENCE_PROBES.md).
The [validation report](../docs/M7_OLD_CDS_VALIDATION.md) records the
integrated runs.

```sh
python3 -I validation/generator/schedule_old_cds_oracles.py --check
python3 -I validation/generator/schedule_old_cds_icma_oracles.py --check
/tmp/sabiquant-validation-venv/bin/python -I validation/generator/schedule_old_cds_probes.py \
  --quantlib-wheel /tmp/sabiquant-validation-wheels/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl --check
cargo test --offline --test schedule_old_cds --test schedule_old_cds_oracles \
  --test schedule_old_cds_icma --test schedule_old_cds_reference
```

The current parsed inventory is **1,659,518 rows in 56 fixture files**. Older
totals below remain historical tranche snapshots. Rows may repeat controls or
carry several outputs; they are not all distinct financial examples. Legal
protection, premium-leg pricing, NI-01 settlement, the historical OldCDS
financial semantics and financial/owner, native-platform and full M7
acceptance remain open.

## M7 modern CDS date evidence

The [modern CDS contract](../docs/M7_CDS_CONTRACT.md) separates explicit
quoted-maturity resolution from a Following payment/accrual date plan. The
new maturity fixture contains **148,688 rows** and **297,376 regime
comparisons**, with **148,512 distinct stored input tuples**. Every day of
2000..2399 appears under both regimes with rotating tenors; boundary controls
cover all twelve stored tenor samples. There are 2,154 additional successful
Years/Months alias checks. Exact dates, Matured outcomes and typed error
causes use E0. The generator selects quarter candidates independently of the
production coordinate formula; [scope, results and pins](../docs/M7_CDS_MATURITY_ORACLES.md)
record the focused debug/release/Clippy and reproduction checks.

Separate hand suites enumerate 1,753,164 maturity comparisons across all
146,097 cycle days and six quotes under both regimes. The date-plan suite
contains 19,200 quarter-boundary/calendar/regime cases over 1600..1999, plus
primary 2009 payment/accrual examples, the 2015 maturity table, all 128 weekday
masks, closures, TARGET guards, civil limits and exact coupon budgets. These
are additional executable checks, not fixture rows. Integrated run status is
tracked in [M7_CDS_VALIDATION](../docs/M7_CDS_VALIDATION.md); no aggregate pass
claim follows from this inventory update.

```sh
python3 validation/generator/cds_maturity_oracles.py --check
cargo test --offline --test schedule_cds_maturity --test schedule_cds_maturity_oracles
cargo test --offline --test schedule_cds --test schedule_cds_calendars
```

At that tranche the parsed inventory was **1,485,312 rows in 53 fixture
files**. Rows may include repeated controls or several outputs; they are not
all distinct financial examples. The cited 2015 FAQ was inspected through its
primary search index after a direct fetch failed; it is not a complete
legal-text review. Automatic OldCDS followed in the next tranche above.

## M7 same-roll ICMA evidence

The [same-roll contract](../docs/M7_ROLL_ICMA_CONTRACT.md) adds **42,647**
context records (42,642 distinct input tuples), including all 4,800 Gregorian
cycle months under four rules and two rotating profiles. It checks 42,179
successful minimal hulls, 260 required Date failures, 188 mismatched frequencies
and 20 Once rejections. Independent integer-overlap Fractions supply 126,537
accrual queries; the existing four-ULP threshold remains unchanged.
The new hand suite also constructs the actual full-civil-range reference hulls.

```sh
python3 validation/generator/schedule_roll_icma_oracles.py --check
cargo test --offline --test schedule_roll_icma --test schedule_roll_icma_oracles
```

See the [oracle scope and pins](../docs/M7_ROLL_ICMA_ORACLES.md),
[review](../docs/M7_ROLL_ICMA_REVIEW.md) and
[aggregate checks](../docs/M7_ROLL_ICMA_VALIDATION.md). Inventory at this
snapshot is **1,336,624 rows in 52 fixture files**. Rows may be context/query
records and repeated controls; they are not all distinct financial examples.
Standard CDS dates were still pending at that snapshot; current modern-date
evidence is recorded above. Full acceptance remains open.

## M7 civil roll evidence

The four [adopted civil rolls](../docs/M7_ROLL_CONTRACT.md) add **122,233**
independent civil-enumeration records. Every month of a 400-year Gregorian
cycle is checked under all four rules and six endpoint/stride patterns;
additional domain/civil-boundary cases retain exact failures. The fixture has
116,811 successful definitions, 5,290 projected-range collapses, 54 required
date failures and 78 invalid tenors. Successful records also check exact
cap/cap−1/cap+1 behavior, requested/resolved provenance and stored tenor.
These are finite scenario records, not exhaustive cross-products or financial
market approval. Dates and regularity are compared exactly, without tolerance.

```sh
python3 validation/generator/schedule_roll_oracles.py --check
python3 validation/generator/schedule_roll_probes.py --quantlib-wheel PATH_TO_WHEEL --check
cargo test --offline --test schedule_roll_oracles --test schedule_roll
cargo test --offline --test schedule_roll_reference
```

The separately attested **538** probes preserve public values/errors and
explicitly classify local policy differences: 110 exact, 100 flag-only,
13 date-vector distinctions and 25 typed rejections among 248 compatible
requests; 290 remaining observations concern absent options or CDS families.
See the [reference report](../docs/M7_ROLL_REFERENCE_PROBES.md) and
[aggregate validation](../docs/M7_ROLL_VALIDATION.md). The fixture inventory
at this snapshot is **1,293,977 rows in 51 files**. All 69 prior
fixture/generator artifacts retain their bytes.
Same-roll ICMA follows under its [own contract](../docs/M7_ROLL_ICMA_CONTRACT.md)
and independent civil/Fraction evidence. OldCDS/CDS/CDS2015 production support
was pending at that snapshot; generated ordinary ICMA and cash-flow evidence
below stay scoped to their recorded tranches.

## M7 analytical cash-flow evidence

The analytical tranche adds **105,272** independent supplied-product rows and
**47,144** civil/Fraction/two-precision financial rows. Inventory at that snapshot is
**1,171,206** rows in **49** fixture files. Product kernel tests require 0 ULP
across 631,632 permutations; public consumers cover plain coupons, clipped
accrual, mixed signed NPV, rounded-term reduction and explicit contextual work.
Four ICMA input-rounding boundaries retain exact conditional outcomes; four
near-Simple-pole rows have materially large propagated errors, explicitly
recorded in [cash-flow validation](../docs/M7_CASHFLOW_VALIDATION.md).
No blanket relative accuracy or cash-flow QuantLib parity follows.

```sh
python3 validation/generator/cashflow_product_oracles.py --check
python3 validation/generator/cashflow_oracles.py --check
cargo test --offline --test cashflow_products --test cashflows_core
cargo test --offline --test cashflows_resources --test cashflows_oracles
```

The private dependency-free kernel is compiled from its actual source by the
excluded integration harness; no test-only public API or duplicated unit tests
are introduced. All public types/contexts remain immutable and std-only. The
[contract](../docs/M7_CASHFLOW_CONTRACT.md) preserves NI-01 settlement and full
M7/native-platform/financial/owner acceptance gates.

## M7 schedule evidence

The core adds **18,916** independent civil-lattice rows and **46** bounded,
byte-attested public QuantLib 1.43 observations. Together the M2–M7 evidence
contained **960,786** rows across **44** fixture files at the saved core snapshot.
The explicit coupon extension adds **21,883** independent civil-coordinate
rows and **29** separately attested public observations, bringing the current
total at that snapshot to **982,698** rows across **46** fixture files.
Generated ICMA adds **36,092** civil-envelope/
Fraction query rows for **18,046** contexts, bringing that snapshot inventory to
**1,018,790** rows in **47** files. These are query rows, with 998 deliberate
repeated second-query controls, not distinct financial examples. Rust tests use only std
and committed data. The civil oracle selects congruent dates over a bounded
span using Python datetime/calendar, rather than reproducing Rust recurrence
steps. Every success has exact dates/regularity and its final boundary limit.

```sh
python3 validation/generator/schedule_oracles.py --check
python3 validation/generator/schedule_probes.py --quantlib-wheel PATH_TO_WHEEL --check
python3 validation/generator/schedule_explicit_oracles.py --check
python3 validation/generator/schedule_icma_oracles.py --check
python3 validation/generator/schedule_explicit_probes.py --quantlib-wheel PATH_TO_WHEEL --check
cargo test --offline --test schedule_core --test schedule_properties --test schedule_reference
cargo test --offline --test schedule_explicit --test schedule_explicit_oracles
cargo test --offline --test schedule_icma --test schedule_icma_oracles --test schedule_icma_properties
```

The original 46-row reference consumer now checks 33 shared results exactly
(including the previously deferred five explicit coupons), four deliberate
calendar-EOM differences, four contextual collision errors and one Zero
metadata difference, plus one Wednesday regularity distinction. In this
historical core consumer, the three CDS families are reference preparation
only; modern date operations have separate evidence above. The separate
29-row explicit consumer checks 13 exact shared values,
two reversed-role rejections, three off-lattice-middle rejections, eight
strict-interior rejections (four reference errors), one named-lattice flag
distinction and two business-only-EOM date distinctions. No tolerance hides a date/status
difference. See [M7_CORE_VALIDATION](../docs/M7_CORE_VALIDATION.md),
[M7_REFERENCE_PROBES](../docs/M7_REFERENCE_PROBES.md) and
[M7_LOCAL_READINESS](../docs/M7_LOCAL_READINESS.md) for native evidence, scope
and remaining gates. The extension evidence is in
[M7_EXPLICIT_VALIDATION](../docs/M7_EXPLICIT_VALIDATION.md); it does not complete
M7 or supply cash flows.

Generated-reference evidence uses exact minimal hulls on the named contractual
lattice, explicit matched frequency and integer overlap Fractions. The fixture
uses safe ±5-step envelopes and at most two periods per explicit outer span;
civil-edge, one-period, longer spans and TARGET failures have separate hand/
property tests. Daily Short/Long input labels can be exact lattices, so labels
are not proof of an actual stub. TOL-E0 discrete checks and the existing TOL-E2
four-ULP ICMA profile stay unchanged. See
[the generated-reference contract](../docs/M7_ICMA_REFERENCE_CONTRACT.md) and
[ICMA validation](../docs/M7_ICMA_VALIDATION.md) for actual evidence and limits.

## M6 mathematical evidence

Ten new fixture sets contain **343,260** independent math rows: 105,307 exact
binary64 reductions, 99,072 interpolation values, 97,851 rounded solver
tolerance decisions, 21,084 precision-checked polynomial roots and 19,946
normal operation/boundary rows. Together with the unchanged M2–M4 sets, the
M2–M6 layers contain **941,824** rows across 42 fixture files. Record counts
describe declared scenarios, not features, branch coverage or mathematical
proofs. Rust verification stays offline and standard-library-only.

```sh
python3 validation/generator/sum_oracles.py --check
python3 validation/generator/interpolation_oracles.py --check
python3 validation/generator/solver_oracles.py --check
python3 validation/generator/solver_roots.py --check
python3 validation/generator/normal_oracles.py --check
cargo test --offline --test math_sums --test math_interpolation_oracles
cargo test --offline --test math_solver_oracles --test math_solver_roots
cargo test --offline --test math_normal_oracles
```

The generators use Python's standard library, exact `Fraction` or independent
120/200-digit Decimal equations. They never consume Rust output. Precision
agreement and conditioned error allowances are empirical evidence, not
certified intervals. Full formulas, support restrictions, conditional overflow
tiers, distributions and native/package checks are recorded in
[M6_NUMERICAL_VALIDATION](../docs/M6_NUMERICAL_VALIDATION.md) and its linked
research documents. Public QuantLib probes there are scoped observations;
these independent fixtures do not claim QuantLib parity.

## M4 interest-rate evidence

Three independent/reference layers contain 154,578 new rate records:
63,064 exact-Fraction/two-precision Decimal equation rows, 14,068 clear
range-boundary rows with exact rational anchors, and 77,446 public attested
QuantLib observations. Value tolerances retain k=8; source rounding and the
narrow computed-log guard-edge tier are classified explicitly. Normal-domain
and exact-status claims are scoped to their declared grids.

```sh
python3 validation/generator/rate_oracles.py --check
python3 validation/generator/rate_boundaries.py --check
python3 validation/generator/rate_quantlib_fixtures.py --quantlib-wheel PATH_TO_WHEEL --check
cargo test --offline --test rates_core --test rates_oracles --test rates_boundaries --test rates_quantlib
cargo test --offline --test rates_dates --test rates_properties --test rate_units
```

The M2–M4 layers contain 598,564 fixture rows. Ordinary Rust
verification remains fully offline and standard-library-only. See
[M4_NUMERICAL_VALIDATION](../docs/M4_NUMERICAL_VALIDATION.md),
[M4_REFERENCE_VALIDATION](../docs/M4_REFERENCE_VALIDATION.md),
[M4_BOUNDARY_VALIDATION](../docs/M4_BOUNDARY_VALIDATION.md) and
[M4_DATE_CONTRACT](../docs/M4_DATE_CONTRACT.md) for numerical conditioning,
provenance, native coverage and remaining acceptance gates.

## M3 extension scenarios

Three market fixture sets contain 135,848 records: 299 Easter Sundays recovered
from unique UK Settlement Good Friday closures, all 73,411 modeled TARGET dates
from 1999-01-04 through 2199-12-31, and 62,138 contextual day-count rows.
Independent Gauss computus, civil-date membership/prefixes and exact `Fraction`
quotients verify every reference value. TARGET projection coverage stays
projected even when QuantLib agrees. Signed/zero differences are specifically
classified, never absorbed into a numerical tolerance.

```sh
python3 validation/generator/market_fixtures.py --quantlib-wheel PATH_TO_WHEEL --check
python3 validation/generator/icma_fixtures.py --quantlib-wheel PATH_TO_WHEEL --check
```

Three additional ICMA sets contain 188 rows: 101 Python exact `Fraction`
oracles, 85 public QuantLib explicit-reference component/scenario rows, and two
precise zero-accrual coverage-domain differences. The independent rational
and finite reference results stay within four ULP of the Rust result.

The generator attests the wheel and records the input-samples generator hash.
See [M3_EXTENSION_VALIDATION](../docs/M3_EXTENSION_VALIDATION.md),
[M3_TARGET_CONTRACT](../docs/M3_TARGET_CONTRACT.md),
[M3_DAYCOUNT_EXTENSIONS](../docs/M3_DAYCOUNT_EXTENSIONS.md) and
[M3_ICMA_CONTRACT](../docs/M3_ICMA_CONTRACT.md) for contracts and remaining gates.

## Initial M3 core scenarios

Eight calendar fixture sets contain 75,227 actual QuantLib 1.43 rows with
independent set/scan expectations. The `bounded-divergences` set has precisely
five finite cases: three empty-month modified adjustments and two 366-rejection
search bounds. Never-open reference searches are not run.

The core day-count fixture contains 72,776 rows for Actual/360, Actual/365
Fixed, Actual/Actual ISDA and European 30E/360. Python's independent daily
leap/common-year counters form exact `Fraction` expectations. Specific ISDA
conditioning and upper-year reference issues have narrow tested classifications;
see [DISCREPANCIES](DISCREPANCIES.md).

Regenerate or verify with the attested temporary/reference environment:

```sh
python3 validation/generator/calendar_fixtures.py --quantlib-wheel PATH_TO_WHEEL --check
python3 validation/generator/daycount_fixtures.py --quantlib-wheel PATH_TO_WHEEL --check
```

Omit `--check` only when intentionally writing new evidence for review.
The downloaded wheel must match the installed module bytes. Normal Rust tests
read committed fixtures offline and need no Python or QuantLib. Full local
check/coverage results and remaining scope are in
[M3_CORE_VALIDATION](../docs/M3_CORE_VALIDATION.md).

## M2 scenario sets

| Fixture set | Records | Independent evidence |
|---|---:|---|
| `TIME-DATE-ORACLE` | 29,368 | Python `datetime`/`calendar`: 28 February and 1 March in every year, every day of 23 boundary years, 1,024 deterministic sampled ordinals |
| `TIME-DATE-ORACLE-ARITHMETIC` | 11,100 | Python date constructors, `timedelta`, and month lengths: days, weeks, clamped months/years, both signed-64-bit extremes |
| `TIME-DATE-QUANTLIB` | 109,208 | Every date in QuantLib's supported 1901–2199 range: ISO weekday, ordinal day of year, month end |
| `TIME-DATE-QUANTLIB-ARITHMETIC` | 5,200 | Public `Date + Period` calls with signed shifts; Python independently resolves the wider SabiQuant range |
| `TIME-PERIOD-QUANTLIB` | 964 | Public normalization and frequency calls, lengths −120 through 120 in all four units |
| `TIME-PERIOD-QUANTLIB-FREQUENCIES` | 11 | Public construction from every named frequency; direct assertions against the specified table |
| `TIME-PERIOD-QUANTLIB-COMPARISONS` | 4,096 | All ordered pairs of 64 boundary tenors; each comparison divergence classified |

There are 159,947 committed records. This count measures fixture records, not
independent financial features or branch coverage. All discrete outputs are
compared exactly (`TOL-E1`); no tolerance is widened to accommodate a mismatch.
The generator fails on unexpected date values, in-range date rejection,
normalization differences, periodic frequency-table differences, or
same-family comparison differences. Known differences are recorded individually
in [DISCREPANCIES.md](DISCREPANCIES.md).

The Rust integration tests additionally enumerate all 3,652,059 supported dates
with nested civil counters and a weekday counter starting at Monday
0001-01-01. They check components, chronological order, signed differences,
day of year, leap behavior, month ends, and ISO formatting/parsing without
reusing the library's conversion algorithm. They also enumerate all 119,988
month ends in both adjacent directions; scan all n-th weekdays in a complete
400-year Gregorian cycle; test inclusive weekday searches at range edges;
and run 20,000 reproducible arithmetic properties with run seed
`0x444154454D322026`. Period tests enumerate 929,296 ordered small-length pairs,
check ordering transitivity and equality substitution including every zero
unit, and exercise wide-intermediate arithmetic and cancellation.

## Reproduction

The oracle generator needs only Python 3.9 or newer:

```sh
python3 validation/generator/time_fixtures.py --reference oracle
python3 validation/generator/time_fixtures.py --reference oracle --check
```

Use an external virtual environment for the optional differential generator.
Download the pinned wheel first, then install that exact local artifact:

```sh
python3 -m venv /tmp/sabiquant-validation-venv
/tmp/sabiquant-validation-venv/bin/python -m pip download --no-deps --dest /tmp/sabiquant-validation-wheels QuantLib==1.43
/tmp/sabiquant-validation-venv/bin/python -m pip install --no-deps /tmp/sabiquant-validation-wheels/*.whl
/tmp/sabiquant-validation-venv/bin/python validation/generator/time_fixtures.py --reference all --quantlib-wheel /tmp/sabiquant-validation-wheels/*.whl
/tmp/sabiquant-validation-venv/bin/python validation/generator/time_fixtures.py --reference all --quantlib-wheel /tmp/sabiquant-validation-wheels/*.whl --check
```

`--check` regenerates every value and checks the generator revision and stable
provenance without rewriting files. It preserves the meaning of the recorded
generation timestamp and environment: those describe the run that wrote the
file. `--reference oracle` does not import QuantLib or need its wheel.

QuantLib generation requires the installed runtime to report exactly `1.43`.
The generator hashes the supplied wheel, and verifies that every installed
`QuantLib/` module file, including the native binary, has exactly the wheel's
bytes. Only public financial APIs are used to produce reference results.
The fixture headers record Python version, platform, UTC generation timestamp,
generator SHA-256, wheel filename, and wheel SHA-256. They separately identify
the source baseline recorded in `docs/QUANTLIB_REFERENCE.md`; the source commit
is not claimed to have been independently attested from the wheel binary.

The committed fixtures were generated with Python 3.9.6 on macOS arm64 using
`quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl`, SHA-256
`85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c`.
On another platform, values should remain identical, while wheel provenance
will identify that platform's downloaded artifact. Changes to reference values
require a discrepancy-log entry and review.

Run the standard-library integration verification with the declared Rust
toolchain:

```sh
cargo test --offline --test time_validation
cargo clippy --offline --test time_validation -- -D warnings
```

The exact local and CI evidence, coverage limitations, and milestone status
are recorded in [M2_VALIDATION.md](../docs/M2_VALIDATION.md). A passed local
fixture suite does not establish behavior outside its documented scenario
sets or prove cross-platform CI results.

## Native Rust coverage

Install the `llvm-tools` component for the same toolchain used to build the
crate, then run:

```sh
rustup component add llvm-tools
python3 validation/coverage.py
```

The runner creates a fresh temporary report directory, builds with
`-C instrument-coverage`, runs every unit/integration test executable, and
exports HTML, raw LLVM JSON, a file summary and a production-region summary.
`--output PATH` selects a new directory; an existing directory is rejected to
avoid accidentally mixing stale profiles. Doctests run separately in the
normal test suite. No Cargo dependency or nightly compiler is required.

The production-region summary excludes embedded test modules and unions
source regions across generic instantiations. It does not measure Rust branch
coverage or claim that every possible input has been tried. The measured M2
scope and unexecuted invariant paths are recorded in
[M2_VALIDATION](../docs/M2_VALIDATION.md#native-coverage-measurement).
