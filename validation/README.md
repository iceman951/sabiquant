# External validation

The Rust crate and its tests depend only on the standard library. QuantLib
never runs during `cargo test`; the integration suite reads committed schema-1
fixtures. Python and QuantLib are external tools for regenerating evidence.
`validation/` is excluded from the published crate.

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

The repository now contains 598,564 fixture rows across M2–M4. Ordinary Rust
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
