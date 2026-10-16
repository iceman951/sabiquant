# US/AFB public reference probes

Recorded 2026-10-16 on native macOS arm64. The
[fixture](../validation/fixtures/daycount/us-afb-probes.fixture) preserves
**741 observations on 247 distinct ordered date pairs**, three public
QuantLib 1.43 counters per pair. This is bounded behavioral evidence for the
[local contract](M3_US_AFB_CONTRACT.md), not a financial definition.

## Runtime provenance and bounded execution

The [generator](../validation/generator/daycount_us_afb_probes.py) imports the
unchanged attestation helper from `schedule_probes.py`. Before any native
reference call, the parent and every isolated worker verify the wheel digest,
installed distribution version and all three shipped QuantLib module files
against wheel bytes. The wheel is
`quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl`, SHA-256
`85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c`.
The module-manifest digest is
`e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d`.
The previously missing temporary wheel/Python wrapper files and venv config
were restored from that same pinned official PyPI wheel before this run;
the existing native binary also matched the restored wheel exactly.
The recorded source baseline commit is not an independently attested source
commit of the wheel. No QuantLib C++ implementation was inspected.

One Python `-I` subprocess per pair calls public `dayCount` and `yearFraction`
on `Thirty360(USA)`, `Thirty360(BondBasis)` and `ActualActual(AFB)`. Endpoints
are restricted to 1901-01-01..2199-12-31, with a ten-second worker timeout
and at most four workers. Parent iteration retains deterministic input order.
Worker failure, timeout or an unclassified result aborts the generation.
Expected library errors are retained as complete strings, alongside the
unchanged signed count; successful fractions retain their binary64 bits.

The independent US category-table oracle supplies local US expectations;
the AFB model actually rolls the remaining date back one year at a time,
then obtains an exact rational. The reference generator was independently
authored before production inspection; the integrator completed its
lower-bound error classifier and the Rust consumer after reading production.
No claim of independent authorship is made for that final consumer.

## Scope and complete classification

| Group | Observations |
|---|---:|
| Year seams in 1901, 2000, 2024, 2100 and 2199 | 543 |
| Annual, leap-promotion and reversal controls | 105 |
| Reference civil limits | 93 |
| Total | 741 |

There are 135 equal, 306 forward and 300 reverse observations. Pair inputs
are deduplicated before counter expansion; each has all three counter labels.
The lower-bound controls include full reference spans. These are selected
controls, not exhaustive pairs over the reference domain.

| Public counter / local comparison | Compatible | Direct reverse count difference | Lower reference anchor error |
|---|---:|---:|---:|
| USA / explicit US flag on | 190 | 57 | 0 |
| BondBasis / explicit US flag off | 212 | 35 | 0 |
| AFB / scalar AFB | 200 | 0 | 47 |

All compatible rows have **zero observed ULP distance**. The declared US
single-division policy remains zero ULP; AFB reference comparisons retain the
existing eight-ULP bound and local exact-rational comparisons the four-ULP
bound. No tolerance was widened. No signed-zero difference occurred in this
corpus; the classifier preserves such a difference if it occurs.

For US, the generator and Rust consumer separately apply the documented
ordered rules directly to the original input order to check every reference
integer. The 92 differences occur on reverse inputs: local evaluation instead
negates the chronological forward result under OQ-14. Both raw results are
kept. Bond Basis is a behavioral flag-off comparison; it does not erase US
context identity or establish universal instrument equivalence.

The 47 AFB errors retain the exact reference message
`year 1900 out of bound. It must be in [1901,2199]`. An independently repeated
rollback leaves a residual endpoint in 1901, explaining why a further backward
proposal can cross the reference's bound before its stopping comparison.
This is an observed behavior hypothesis, not a claim to have inspected its
implementation. SabiQuant supports the input pair and its full 0001–9999
civil domain without constructing such a sentinel. None of these errors is
silently recast as a numerical match.

The Rust consumer checks every annotation, independently reclassifies each
row, recomputes all census headers and enforces complete per-pair counter
coverage. It checks the exact AFB decomposition by repeated rollback before
comparison, E0 signed counts, local rational accuracy, compatible numerical
bounds, complete error strings, and fingerprints of the raw body, generator,
contract, source records and unchanged helpers. SHA-256 regeneration pins
are supplemented by FNV-1a byte checks in Rust; FNV is not an authenticity claim.

## Reproduction

```sh
/private/tmp/sabiquant-validation-venv/bin/python -I validation/generator/daycount_us_afb_probes.py \
  --quantlib-wheel /private/tmp/sabiquant-validation-wheels/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl --check
cargo test --offline --locked --test daycount_us_afb_reference
```

The root's focused consumer passed. The aggregate validation report records
actual fixture reproduction, complete-suite and strict-check outcomes. The
body SHA-256 is
`cce8244b329fcfa0b7c284e8e2b99dd35317e39cef17c5a6aa673b510f320be1`;
FNV-1a-64 is `76d2db8d5b1bdc8f`. All earlier fixtures and generators remain
byte-identical to `1c16daf`. Full SIFMA source acceptance, the unread original
1994 AFB publication, native Linux/Windows execution and milestone acceptance
remain separate gates.
