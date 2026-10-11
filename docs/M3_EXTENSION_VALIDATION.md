# Local M3 Extension Validation

This records the Easter/TARGET and contextual day-count extension batch on
2026-10-11. Work is authorized locally under the owner's delegated instruction
to continue with broad meaningful tests. It does not establish remote CI,
owner review, publication approval or full M3 milestone acceptance.

## Implemented contracts

| Feature | Contract and evidence |
|---|---|
| Western Easter | USNO Oudin computus, independently checked against Gauss for all 9,999 supported years; Sunday and 22 March–25 April constraints; both correction classes and early-century floor division |
| TARGET | [Guarded euro settlement contract](M3_TARGET_CONTRACT.md); valid from 1999-01-04, strict source coverage by default, explicit projection; disjoint verified intervals 1999–2002 and 2026–2028; immutable overrides/joints preserve coverage |
| Bond Basis | [Extension contract](M3_DAYCOUNT_EXTENSIONS.md); named scalar convention, signed forward normalization and one exact-integer division |
| 30E/360 ISDA | Required termination date; February termination exception; canonical zero and signed normalization; distinction from European 30E/360 |
| Business/252 | Required plain or guarded TARGET calendar; ascending half-open interval then signed reversal; no default market or global cache; complete coverage error source chain |
| Actual/Actual ICMA | [Explicit determination-period contract](M3_ICMA_CONTRACT.md); nonempty ordered contiguous periods and contractual frequency; no schedule inference; compensated chronological summation |

The five stateless conventions remain a `Copy` enum. Required state lives in
separate immutable types. Every error kind has structured context, public
documentation, crate-level conversion, formatting and typed source checks.

## Independent enumeration

- Retain all M1/M2/core M3 regressions, including every supported civil date,
  period laws and numeric/identifier policies.
- Mandatory scalar windows enumerate 2,134,521 ordered pairs over 1999–2002
  and 1,199,025 over 2099–2101: 3,333,546 pairs, each checked for all five
  scalar conventions using independent integer/rational expectations.
- ISDA 30E/360 checks every ordered month-end pair in every supported year
  under three termination contexts: 4,319,568 cases, including February,
  century and range boundaries. Signed/equal cases have explicit expectations.
- Business/252 enumerates all 128 weekend masks with dated overrides,
  holiday/business endpoints, reversed pairs and exact single-division results.
- Calendar arithmetic counting checks daily totals over all 3,652,059 dates
  for every weekend mask and endpoint choice. Nested joints and overlapping
  overrides are checked in 256,000 deterministic cases against daily prefixes,
  rather than the production complete-week arithmetic.
- ICMA independently checks 3,338,660 ordered monthly-window pairs, all
  periodic frequencies and regular/short/long stub shapes. Maximum observed
  rational-oracle error is 1 ULP. The maximum 3,652,058 one-day reference
  segments give 0 ULP error; late clipped queries exercise binary search.
- TARGET checks every source-verified day and every modeled QuantLib date,
  policy-independent coverage labels, strict gaps, pre-launch rejection,
  omitted/equal endpoints, zero/Unadjusted operations, all search fallbacks,
  immutable overrides and joint Boolean short circuits. Full-range projected
  counting is compared with an independent daily count through Date::MAX.

ICMA's E2 bound is derived for the maximum supported number of positive
segments, including quotient rounding and compensated-sum error; it does not
reuse the earlier short ISDA operation-count argument. See its contract.
Single-division conventions are compared at 0 ULP. No tolerance was widened
to absorb a reference discrepancy.

## External reference data

| New fixture set | Records | Independent expectation |
|---|---:|---|
| Easter | 299 | Gauss computus; reference Easter recovered from the unique UK Settlement Good Friday in every 1901–2199 year |
| TARGET membership | 73,411 | Independent civil/Easter closing rules for 1999-01-04–2199-12-31; source coverage labels remain distinct from matching results |
| Extended day counts | 62,138 | Direct source formulas/endpoint counts checked against QuantLib, signed policy checked against exact Fraction quotients |
| ICMA exact oracle | 101 | Python Fraction across all frequencies, stubs, extreme and long reference contexts |
| ICMA QuantLib | 85 | Explicit public reference-period component calls; each full-period probe must establish its declared frequency |
| ICMA strict coverage | 2 | Equal accrual outside supplied references; precise intentional rejection versus reference zero |

The new total is 136,036 records. Together with the previous 307,950 records,
the repository contains 443,986 fixture records. Record counts measure inputs,
not independent features or branch coverage.

The installed QuantLib 1.43 runtime is byte-attested against the downloaded
macOS arm64 wheel, SHA-256
`85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c`.
Headers record generator revisions, environment and timestamp. The declared
source commit is separate from the observed wheel bytes; its presence does
not claim binary source-commit attestation.

Exactly 241 Bond signed, 103 ISDA signed, 10 ISDA February zero and 3,375
Business/252 signed differences are narrowly classified. ICMA has exactly
two zero-outside-coverage differences. [DISCREPANCIES](../validation/DISCREPANCIES.md)
specifies predicates, examples and resolutions. Unexpected differences fail.

## Reproduction and checks

Use Rust 1.99 and the attested external Python/QuantLib environment:

```sh
cargo test --offline
cargo test --offline --release
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --document-private-items
cargo check --offline --target x86_64-unknown-linux-gnu --all-targets
cargo check --offline --target x86_64-pc-windows-msvc --all-targets
python validation/generator/market_fixtures.py --quantlib-wheel /path/to/pinned-wheel.whl --check
python validation/generator/icma_fixtures.py --quantlib-wheel /path/to/pinned-wheel.whl --check
python3 validation/coverage.py --output /tmp/new-report-directory
```

All native tests run on aarch64 macOS. Linux/Windows commands are compilation
checks only; they do not constitute native execution on those platforms.
The crate and normal offline tests require no Python, QuantLib or Cargo
dependencies. Generators are outside the shipped library.

## Final local results

Rust 1.99.0 on aarch64 macOS passed **240 debug tests** and **239 release
tests**, including six doctests, with zero failures or ignored tests. The
one-test difference is the existing debug-only internal invariant assertion.
Strict Clippy, formatting, public/private rustdoc, both cross-target compile
checks, generator `--check`, dependency inspection and whitespace checks pass.
Independent audit additionally matched 52,000 scanned calendar-count cases.

Native LLVM instrumentation in
`/private/tmp/sabiquant-m3-extensions-coverage-complete` measured these
production source regions after excluding embedded test modules:

| File | Covered / regions | Coverage |
|---|---:|---:|
| `calendar.rs` | 750 / 789 | 95.06% |
| `calendar/easter.rs` | 32 / 32 | 100% |
| `calendar/target.rs` | 385 / 404 | 95.30% |
| `daycount.rs` | 310 / 310 | 100% |
| `daycount/icma.rs` | 163 / 164 | 99.39% |

Unexecuted calendar regions include the defensive per-date fallback of the
plain infallible counter: guarded TARGET values cannot escape through its
public API, so market counting uses the checked wrapper instead. Others are
redundant validated-date checks, invariant-only computus/date-construction
failures, the early-return Unadjusted match arm and defensive search exits.
ICMA's remaining region is a filter rejecting a nonpositive frequency count;
all named periodic frequencies return positive counts and Once returns None.
These measurements are source-region coverage, not branch coverage or proof
of every possible input. No private invalid-value fabrication was added to
inflate the result.

## Remaining gates

US 30/360 February/EOM qualification and AFB February backward rolling remain
Under Research (OQ-15/16) and outside this batch. A TARGET maintenance
appointment and ongoing annual source review remain release gates. External
primary-platform CI and owner/milestone review remain outstanding because
the user selected local-only work. No M2/M3 completion or publication claim
is inferred from local validation.
