# Explicit US 30/360 independent oracle evidence

This report covers the adopted [local US/AFB contract](M3_US_AFB_CONTRACT.md),
US integer/rational oracle and exhaustive US windows. The
[source review](M3_THIRTY360_US_SOURCE_REVIEW.md) preserves the unread full
SIA/SIFMA source and financial-acceptance limits. No QuantLib output or US
production source supplied these expected values. This is local engineering
validation, not M3 acceptance or a universal interpretation of US/A001/SIA.

## Construction and numerical contract

[The Python generator](../validation/generator/daycount_us_oracles.py) maps
chronological endpoints onto synthetic month coordinates and selects their
day components from an explicit category table. It uses original Gregorian
February predicates and the required EOM flag. A separately written literal
four-rule evaluation cross-checks every emitted integer. All 393,158 agree.
The two calculations share the adopted contract; agreement is not an
independent financial source. Reverse inputs negate the integer; all zero
integers become positive-zero fractions.

The exact rational is `Fraction(days, 360)`, recorded in reduced form and
converted directly to correctly rounded binary64 bits. The Rust consumer
compares integer counts at TOL-E0 and fractions at **zero ULP**, TOL-E1. It
also checks the rational's integer identity without floating arithmetic.
For the widest civil pair, January 1, year 1 to December 31, year 9999,
the count is 3,599,640: an end day of 31 remains 31 when the start day is 1.
No endpoint is constructed outside the supported civil domain.

## Frozen corpus census

The [fixture](../validation/fixtures/daycount/us-30360-oracle.fixture) has
**393,158 rows**, **392,772 distinct `(start,end,eom)` inputs**, and 386
repeated inputs with distinct coverage roles. Each input-pair definition
emits adjacent flag-off/flag-on rows. **26,164 pairs produce different
counts under the two settings**. Repetition is retained visibly rather
than represented as additional unique coverage.

| Corpus | Rows | Exact coverage |
|---|---:|---|
| Gregorian cycle | 292,194 | Every one of 146,097 dates from 2000-01-01 through 2399-12-31 is an anchor under both flags; one of twelve rotating endpoint profiles |
| All supported years | 99,990 | Five classes for every year 0001 through 9999, under both flags; odd years reverse the normalized pair |
| Civil limits | 578 | Cartesian 17 × 17 endpoint set spanning MIN/MAX and nearby January/February/March/December edges, under both flags |
| Endpoint neighborhoods | 396 | Eleven common/leap/century and synthetic-zero center pairs; all 3 × 3 endpoint perturbations by −1/0/+1 day, both orientations and flags |

The cycle profiles select equality; next/previous civil day; month end;
month end plus one; month start; February end; March 31; next/previous-year
anniversary; MIN to anchor; or anchor to MAX. Profile selection is
`zero_based_anchor_offset % 12`. This covers every anchor, **not** every
anchor/profile combination or every endpoint pair in the cycle.

Every supported year contributes February-end to March 31, January 31 to
February end, adjacent-year February ends, a rotating month-end neighbor,
and February-end minus one to March 31. The neighbor month rotates over
one month per year; this corpus does not claim every month's edges in
every year. Separate hand tests cover the wider month-end matrix.

| Derived class | Rows |
|---|---:|
| Equal dates | 26,128 |
| Strictly increasing dates | 205,578 |
| Strictly decreasing dates | 161,452 |
| Nonzero synthetic count | 365,562 |
| Distinct dates, forward synthetic zero | 996 |
| Distinct dates, reverse synthetic zero | 472 |
| Flag off / flag on | 196,579 / 196,579 |

[The Rust consumer](../tests/daycount_us_oracles.rs) recomputes all census
headers, paired-flag agreement on inputs, every cycle anchor/flag and every
year/class/flag. It rejects duplicate cycle and year coverage claims and
checks odd-year reversal, context inspectors, canonical identifier and all
positive-zero bit patterns. Contract, source review and generator are pinned
by SHA-256 in metadata and checked against their FNV-1a byte fingerprints in
Rust. The body has both SHA-256 and FNV-1a pins; FNV is a deterministic
integrity check, not a cryptographic authenticity claim.

## TS-DCC-07 exhaustive windows

The same Rust test file separately enumerates **every oriented pair**, equal
dates included, inside each required window. It uses independent original
endpoint categories and synthetic coordinates; it does not read the US
implementation or expand a multi-million-row fixture. Each query checks
both the exact count and the zero-ULP single-division fraction.

| Closed civil window | Dates | Oriented pairs | Queries including both EOM flags |
|---|---:|---:|---:|
| 1999-01-01 through 2002-12-31 | 1,461 | 2,134,521 | 4,269,042 |
| 2099-01-01 through 2101-12-31 | 1,095 | 1,199,025 | 2,398,050 |
| Total | — | 3,333,546 | **6,667,092** |

Those 6,667,092 queries are separate from the 393,158 fixture records.
They are exhaustive for the two stated windows, not the full 0001–9999
Cartesian domain. The Fraction fixture supplies an independent
cross-language numerical check; the exhaustive Rust oracle supplies the
required broad discrete pair coverage.

## Reproduction and focused results

```sh
python3 -I validation/generator/daycount_us_oracles.py --check
env PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-m3-us-oracle-target cargo test --offline --locked --test daycount_us_oracles
env PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-m3-us-oracle-target cargo test --offline --locked --release --test daycount_us_oracles
env PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-m3-us-oracle-target cargo clippy --offline --locked --test daycount_us_oracles -- -D warnings
```

The generator's actual `--check` completed successfully. It re-evaluates all
rows, checks all stable metadata and body bytes, and reuses the recorded
timestamp. Only recorded Python-version and platform labels are excluded
from cross-environment comparison. Focused native debug and release each
passed **2 tests**, including both complete exhaustive windows. Focused
strict Clippy passed. Repository-wide tests, package extraction, docs,
coverage and platform checks are recorded by the parent validation tranche;
these focused results do not substitute for those checks.

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| Contract | 8,987 | `595df058cdd8e64a935b6fc992ef0ca903e7e7fbdfa4ed613bbe598a3afe10b4` |
| US source review | 9,917 | `c90c18543f28ed0f2def9744f56ef9e397561f4f7b2d602a18e79f97158c1958` |
| Generator | 13,399 | `c9de4d22e655ed6911d161d1c5d9888f36563d5dbcae29bbaa20b70a92c33afc` |
| Fixture | 32,060,549 | `92cb385dc1b7b6f79dcce023352cc556704a79f37f07067cf14ed95876ec425a` |
| Rust consumer | 11,658 | `aed5452bd78c92647cb58a48398285793b1fc5d3ba1b5173bab9062ef8d6dc7e` |

Fixture body SHA-256:
`1098bbad2cacb39a60fec7689f017e6c995c5e7e7302179299e438bc741f2dfc`.
Body FNV-1a-64: `2cfd06e603b8aba2`.
