# AFB independent oracle validation

Recorded 2026-10-16, Asia/Bangkok. This report covers the independently
authored AFB oracle and focused tests for the adopted
[US/AFB contract](M3_US_AFB_CONTRACT.md). It is not full M3 acceptance or an
assertion that the original 1994 AFB publication was inspected. Primary-source
inspection and its limits are recorded separately in
[M3_AFB_SOURCE_REVIEW](M3_AFB_SOURCE_REVIEW.md).

## Independence and numerical checks

The [Python generator](../validation/generator/daycount_afb_oracles.py)
actually moves the remaining endpoint backward **one year at a time**. It
constructs civil dates, clamps an impossible February 29, promotes February
28 in a leap target year, and stops before a candidate preceding the start.
It checks actual February-29 dates for membership in the residual half-open
interval, then evaluates an exact `Fraction`. No production code,
reference-library implementation, or reference-library output was read or
used to obtain these expectations. In particular, the oracle does not
reimplement the production constant-time year-selection formula.

The [Rust suite](../tests/daycount_afb.rs) independently repeats annual
rollback using a manual Gregorian month table. It builds year prefixes from
that table and **walks every residual civil day** to identify included
February 29. This differs from Python's date construction and per-year
membership query. Before checking the public year fraction, the consumer
compares all recorded discrete fields at E0: actual days, accepted years,
residual endpoint, stub days, denominator and reduced rational numerator and
denominator. This is agreement between two independently authored oracles;
the public API does not expose its internal decomposition.

The public integer count is checked exactly in both directions. For fractions,
Rust divides the exact reduced integer numerator by its denominator **once**,
instead of reproducing production's whole-years-plus-quotient evaluation.
The integers are exactly representable in binary64; this independently
rounded rational must have exactly the fixture's recorded bit pattern.
Production values are then compared at TOL-E2, at most **4 ULP**. The largest
observed distance was **1 ULP in both native debug and release**; the allowed
tolerance remains four. Every record also checks bit-exact reversal and
positive zero at equal endpoints.

## Fixture scope and census

[afb-oracle.fixture](../validation/fixtures/daycount/afb-oracle.fixture) is
schema 1, with **215,327 records**, **205,673 distinct ordered input pairs**
and **19,621 equal-date records**. Some profiles deliberately repeat input
pairs; the record count is not a distinct-case count.

| Profile | Records | Scope |
|---|---:|---|
| Primary printed anchors | 5 | Selected ISDA 1999 AFB fractions, separately asserted before generation |
| Derived hand boundaries | 12 | Explicit decomposition including the leap-promotion discriminator and MIN→MAX |
| Gregorian cycle | 146,097 | Every start from 2000-01-01 through 2399-12-31 |
| All-years February/equal | 49,995 | Five same-year profiles in every civil year 1–9999 |
| Adjacent-year anniversaries | 9,998 | One rotating profile for every ending year 2–9999 |
| February century windows | 4,971 | Every chronological pair among selected February/March dates in each documented window |
| Deterministic wide pairs | 4,096 | Explicitly sampled from all civil ordinals with the recorded LCG64 seed |
| Civil-limit Cartesian pairs | 153 | Every chronological/equal pair among 17 explicit endpoints |

The cycle rotates these sixteen durations, in days:
`0,1,2,28,59,60,90,180,364,365,366,367,730,1096,1461,2922`.
This is **one duration per daily start**, not their Cartesian product. Rust
recomputes the profile choice and checks that all 146,097 start offsets
occur exactly once.

The five all-years profiles are February 28→March 1, February 1→last February
day, last February day→March 1, January 1→March 1, and equal last-February
dates. The adjacent-year section rotates eight endpoint shapes by the ending
year modulo eight; it does not claim all eight shapes for every year.
Rust checks the full year sets for both sections.

Century windows are centered on years
`1,4,100,400,1600,1700,1800,1899,1900,1901,1999,2000,2001,2099,2100,2101,2399,2400,2401,9995,9996,9997,9998,9999`.
For each, valid dates on February 27/28/29 and March 1/2 within two years
either side are paired chronologically, retaining equal dates. Overlapping
windows intentionally repeat some pairs. This is not exhaustive pairing of
every civil day in those multi-year windows, and does not by itself complete
TS-DCC-07's broader whole-window requirement. A separate seventh test now
covers that requirement directly: every chronological/equal pair in
1999-01-01..2002-12-31 (1,067,991) and 2099-01-01..2101-12-31 (600,060),
plus each reverse, for 1,668,051 pairs and 3,333,546 distinct oriented inputs.
The equal inputs are evaluated twice by the symmetric loop. Its expected
decomposition repeats annual moves and uses independently constructed leap
dates/ordinals, avoiding a residual-day scan for this larger Cartesian matrix.
It performs 1,268,318 accepted annual steps; maximum observed error is 1 ULP.

The civil endpoint set is:
`0001-01-01,0001-01-02,0001-02-28,0001-03-01,0001-12-31,0002-01-01,0004-02-28,0004-02-29,0004-03-01,9996-02-28,9996-02-29,9996-03-01,9999-01-01,9999-02-28,9999-03-01,9999-12-30,9999-12-31`.
The wide section has no hidden scan cap: all accepted annual moves really
execute, including long intervals. Its sample size and ordinal-generation
algorithm are explicit and independently rechecked by the Rust consumer.

| Recomputed census | Count |
|---|---:|
| Zero accepted whole years | 138,743 |
| Positive accepted whole years | 76,584 |
| Residual denominator 365 | 196,847 |
| Residual denominator 366 | 18,480 |
| Actual accepted year steps across all records | 14,743,934 |
| Largest accepted whole-year count | 9,998 |
| Largest observed residual day count | 365 |

The fixture records `N`, `E*`, residual days, selected denominator, the reduced
exact rational, and its correctly rounded IEEE-754 bits. Its header pins the
contract, finalized source report and generator. SHA-256 checks are performed
by regeneration; the Rust consumer additionally checks their FNV-1a-64 pins
and the entire body fingerprint, then recomputes the profile/decomposition
censuses. FNV is a reproducibility checksum, not a cryptographic attestation.

## Hand tests and actual verification

The seven Rust tests cover the full fixture; five actual printed ISDA fraction
anchors; fifteen explicit February/civil decompositions; exact identifier and
alias handling; anniversary and split-additivity counterexamples; and
reproducibility across sixteen scoped threads. Each thread repeats five
boundary calculations 32 times, comparing the result bits with a sequential
baseline as well as independent rational values. The raw `N=0` February-28
endpoint is checked separately so it cannot be silently promoted.

Actual focused results on the final suite:

| Check | Result |
|---|---|
| Python generator `--check` | Terminal exit 0; all 215,327 rows reproduced |
| Native debug `daycount_afb` | 7 passed, 0 failed/ignored; no warnings; maximum 1 ULP |
| Native release `daycount_afb` | 7 passed, 0 failed/ignored; no warnings; maximum 1 ULP |
| Focused strict Clippy | Terminal exit 0 with `-D warnings` |

Commands use Rust 1.99 explicitly:

```sh
python3 -I validation/generator/daycount_afb_oracles.py --check
env PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-m3-afb-target cargo test --offline --locked --test daycount_afb -- --nocapture
env PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-m3-afb-target cargo test --offline --locked --release --test daycount_afb -- --nocapture
env PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-m3-afb-target cargo clippy --offline --locked --test daycount_afb -- -D warnings
```

Logs are temporary local evidence, under
`/private/tmp/sabiquant-m3-afb-focused-{debug,release,clippy}.log` and
`/private/tmp/sabiquant-m3-afb-oracle-check.log`. They are not packaged files.
These focused checks do not stand in for the parent tranche's complete test,
documentation, package, production-coverage or platform verification.

## Reproduction pins

The generated timestamp is `2026-10-15T17:33:32Z`. `--check` retains it and
compares every byte except the Python-version and platform labels. The
original version label is Python 3.9.6, Darwin arm64. Expectations use the
standard library only.

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| Generator | 13,187 | `aa2e75fe9f95e3365b934967c0abee79b2f6c9bd7e4941268f24de893e03faa7` |
| Fixture | 21,040,693 | `00aab23ecc7fd9094355c1bbdc54e9cbdac24f98a41960683e1f123aa0b20a11` |
| Fixture body, including columns and trailing newline | — | `ffb58632c7a79ed3f6a9d8f3fb9d4bb90ec227ec1d01184c6d70828a6a50edef` |
| Rust suite | 21,952 | `d18a757301e6ad1163529633f42d1f4157ca4a84260b354ec875552e827603a1` |
| Adopted contract | 8,987 | `595df058cdd8e64a935b6fc992ef0ca903e7e7fbdfa4ed613bbe598a3afe10b4` |
| AFB source review | 10,507 | `3b30092d8e45e01af12870cffc973a03d4fe116a48b9617a513ddf6a81a9681d` |

Fixture-body FNV-1a-64 is `865dd598739ac10c`. No prior frozen fixture or
generator was edited by this independent-oracle work. Financial acceptance,
the unread original 1994 AFB source, complete milestone readiness, native
Linux/Windows execution and NI-01 settlement remain separate gates.
