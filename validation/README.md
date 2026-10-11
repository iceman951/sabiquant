# External validation

The Rust crate and its tests depend only on the standard library. QuantLib
never runs during `cargo test`; the integration suite reads committed schema-1
fixtures. Python and QuantLib are external tools for regenerating evidence.
`validation/` is excluded from the published crate.

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
