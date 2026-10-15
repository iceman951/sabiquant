# Independent quoted CDS maturity oracle

Completed 2026-10-15, Asia/Bangkok. The generator, fixture and Rust consumer
were authored by `/root/cds_legacy_sources` from the frozen
[CDS contract](M7_CDS_CONTRACT.md) and
[maturity design](M7_CDS_MATURITY_DESIGN.md), without reading production CDS
implementation or using SabiQuant/QuantLib outputs to generate expectations.
This report covers the quoted-maturity resolver only. Primary market anchors,
the Following date plan, aggregate regressions and wider acceptance are
separate evidence.

## Independent construction and fixture schema

The [Python generator](../validation/generator/cds_maturity_oracles.py)
enumerates March/June/September/December twentieth tuples in neighboring civil
years. Quarterly2009 selects the minimum candidate strictly after trade.
Semiannual2015 selects the maximum eligible March/September candidate no later
than trade, then the next quarterly candidate. Zero quote is tested for a
matured outcome before constructing a `datetime.date`.

Tuples permit abstract years zero and 10000. They prevent an unused
out-of-range anchor from becoming a date error. To apply a quote, the oracle
splits whole years from its month count, enumerates quarter candidates in the
shifted years and indexes the remaining zero-to-three quarters. Thus even a
huge valid quote requires only a bounded candidate set. It does not reproduce
the production cutoff/Euclidean-remainder anchor formula, repeatedly add
months to a Date, or iterate billions of quarters. The overflow displacement
is calculated from the independently selected final year/month and the
original trade year/month.

The schema-1 tab-separated
[fixture](../validation/fixtures/schedule/cds-maturity-oracle.fixture) stores
one input and both regime results per row:

```text
id profile trade length unit
quarterly_status quarterly_date quarterly_error_months
semiannual_status semiannual_date semiannual_error_months
```

The column line is a single line in the actual fixture. Status is `date`,
`matured`, `invalid_tenor` or `date_error`. A date appears only for `date`;
signed final-target month displacement appears only for `date_error`.
Original tenor length/unit remains explicit. No missing or null date encodes
Matured implicitly.

## Retained coverage and results

There are **148,688 rows**, **148,512 distinct stored input tuples** and
**297,376 exact regime comparisons**. The 176 repeated inputs deliberately
overlap corpus profiles. The daily portion covers all **146,097 dates** from
2000-01-01 through 2399-12-31, with both regimes on every date. Twelve stored
tenors rotate: 0/3/6/9/12/15/18/21/60 Months and 0/1/5 Years. This is not a
full daily tenor Cartesian product.

| Profile | Rows |
|---|---:|
| Complete daily Gregorian cycle | 146,097 |
| Quarter twentieth minus/on/plus one day, all twelve tenors, sixteen selected years | 2,304 |
| MIN/MAX and nearby civil limits | 120 |
| Tenor-domain and precedence controls | 144 |
| Huge valid tenors | 8 |
| Long quotes reaching or crossing the civil ceiling | 15 |

The selected boundary years include 1, 4, 99, 100, 400, 1900, 2000, 2001,
2015, 2016, 2099, 2100, 2399, 2400, 9998 and 9999. Domain controls include
negative `i32::MIN`, invalid zero Days/Weeks, non-quarter Months, invalid
`i32::MAX` Months, and valid enormous Years/quarter-multiple Months.

| Outcome | Quarterly2009 | Semiannual2015 |
|---|---:|---:|
| Exact maturity date | 148,328 | 135,981 |
| Matured | 0 | 12,377 |
| Required final date out of range | 256 | 226 |
| Invalid stored tenor | 104 | 104 |

The [Rust consumer](../tests/schedule_cds_maturity_oracles.rs) passed these
comparisons with **zero mismatches** in native debug and release. It checks
all successful definition inspectors, original stored tenor metadata on
errors, typed variants and stable kinds, and exact `DateError::OutOfRange`
input/operation/month displacement. It also checks that every returned date
is a future quarter twentieth. Independent Gregorian ordinal arithmetic
proves the daily rows are contiguous; the complete declared boundary matrix
is checked for membership.

There are **2,154 additional successful alias comparisons** at explicit
boundary controls. They require identical outcomes and distinct immutable
definitions for stored Years versus equivalent Months, while checking both
sets of inspectors. Daily alias rows are not duplicated to obtain these
checks. All comparisons use exact E0; no floating point is involved.

## Reproduction and integrity

The following commands completed with terminal exit zero on Darwin arm64:

```sh
python3 validation/generator/cds_maturity_oracles.py --check
cargo test --offline --test schedule_cds_maturity_oracles -- --nocapture
cargo test --offline --release --test schedule_cds_maturity_oracles -- --nocapture
cargo clippy --offline --test schedule_cds_maturity_oracles -- -D warnings
```

Rust and Cargo were explicitly selected from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`;
the observed versions were `rustc 1.99.0 (b940084d7 2026-09-28)` and
`cargo 1.99.0 (5f94df478 2026-08-27)`. Focused target directories under
`/private/tmp/sabiquant-cds-maturity-oracle-{debug,release,clippy}` isolated
the builds. Clippy also needed this toolchain directory first on PATH.

The fixture pins SHA-256 of generator, frozen contract and historical design.
It records body SHA-256 and FNV-1a 64-bit. The Rust consumer checks the body
FNV, exact SHA headers, source-file FNV pins, counts and profile membership.
FNV is an accidental-change check, not a cryptographic authenticity claim.
Generator `--check` recomputes the entire body and compares all provenance
headers except recorded Python/platform labels, retaining the original
timestamp. No regeneration was needed during final checking.

| Artifact | SHA-256 |
|---|---|
| Generator | `a1705d627d34facca566cdf0c1e0d9d6ba6c5c9c51f7dc19cb070e8b24d774f0` |
| Fixture, 12,743,426 bytes | `4c3c4965b5a26fd5ab80dd0b45e9cfe3e097a441742875a6ff2271c30f12af4a` |
| Fixture body | `83dfee265e35923f8a836a46b2e9063b789ec0705889de2be24d3ec3a28b7d1f` |
| Rust consumer | `ce3fa7aa2c86f103cfc0fd744d25e418ab81d368ae381529c83a4ad12ad2b930` |
| Frozen contract | `312df1e25a22c2f2575c30fd30853d315258ac3844ac0ab386d42a1db9c5aad5` |
| Historical maturity design | `4c221a428544d0fbd54736505331689c2bcf33405b3973a666ab94c90b64a2d8` |

This is validation of the adopted proleptic analytical definition. It does
not establish market practice throughout years 0001-9999, financial/owner
acceptance, a legal effective date, a premium leg, OldCDS support, native
Linux/Windows execution, or full M7 completion.
