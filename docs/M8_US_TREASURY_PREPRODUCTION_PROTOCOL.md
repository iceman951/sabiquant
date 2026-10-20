# Treasury independent pre-production protocol

Recorded after local contract adoption `a1b66f9` on2026-10-20. The complete
[main contract](M8_US_TREASURY_CONTRACT.md), [cash component](M8_US_TREASURY_CASH_CONTRACT.md)
and [76-family test plan](M8_US_TREASURY_TEST_DESIGN.md) govern this work.
No Treasury library implementation may precede accepted independent evidence
for all four corpora below. This protocol records production order and
artifact integrity; it is not an implementation result or financial acceptance.

| Independent artifact | Author and scope | Required connection after production |
|---|---|---|
| `treasury-source-cash-oracle.fixture` | Claude peer sabiquant-a2: exact source/date/cash from the complete pair | Public date preparation, source views, all four cash recipes and contextual decisions/errors |
| `treasury-numeric-oracle.fixture` | Claude peer sabiquant-d5: prescribed exact numerical primitives | Production-linked private factor/PV/risk/reduction/residual adapter |
| `treasury-ideal-price-risk-oracle.fixture` | Separate Codex author: ideal model independently of prescribed observations | Public price and all six raw/normalized risk comparisons with conditioning |
| `treasury-ideal-inverse-oracle.fixture` | Same ideal author: target-driven exact/sign-certified inverse references | Public bounded inversion and model/target/slope-qualified accuracy |

The files belong under `validation/fixtures/bonds/`; their generators are
`treasury_source_cash_oracles.py`, `treasury_numeric_oracles.py` and
`treasury_ideal_oracles.py` under `validation/generator/`. A further independent
Codex author prepares public numerical hand discriminators under
`validation/hand/` before reading Treasury production. Hand source is kept
outside `tests/` until the new API exists. Parsing it with rustfmt is not
compilation or execution. Final validation must preserve that distinction.

Authors use only the adopted pair, primary-source evidence and independently
derived mathematics for expected financial results. They do not import/read
future Treasury implementation or substitute observed production results.
Existing fixture-reader/public helper syntax may inform encoding. No QuantLib
C++ is read. The ideal author does not use prescribed primitive observations
as ideal expectations. Every primitive superset is labelled separately from
valid dated financial inputs; no impossible report or out-of-domain coupon
is used to claim public reachability.

## Provenance and immutable expectations

Each corpus uses the unchanged schema1 [shared Rust reader](../tests/support/fixture.rs).
Its eleven required headers, lowercase header keys, explicit unique IDs,
columns and equal-width records remain mandatory. Canonical artifact bytes
use LF and end in LF. Additional common headers are:

- `generator-sha256`/`generator-fnv64` over the actual generator bytes;
- `adopted-contract-sha256`/`adopted-contract-fnv64` over the main contract;
- `cash-contract-sha256`/`cash-contract-fnv64` over the cash component;
- `requirements-rows-sha256`/`requirements-rows-fnv64` over the26 `FR-UST`
  table rows in file order, joined by LF with a final LF;
- `body-sha256`/`body-fnv64` over columns plus LF plus all records and final LF;
- `records-sha256`/`records-fnv64` over all record lines and final LF;
- `record-count`, `column-count`, `profile-counts` as a positive-integer JSON
  object matching the `profile` or `family` column;
- `input-columns`, `unique-input-count`, `inputs-sha256`/`inputs-fnv64` over
  those input fields in each row, joined by TSV and LF in file order.

Root independently reviews and fixes each corpus's complete column list,
profile field and semantic input columns in the attestor. Fixture-supplied
column declarations alone cannot establish input uniqueness: an answer
column must not hide duplicate actual inputs. A schema not yet reviewed
fails closed. Quantity-specific classifications, exact contextual errors,
source precision and mathematical/conditional bounds remain separate from
artifact pins. SHA/FNV agreement cannot prove financial correctness.

Initial generation uses actual UTC, Python and platform metadata and refuses
existing output paths. `--check` independently regenerates byte-identical
expectations using the retained timestamp/platform metadata. Provisional
pre-acceptance corrections are reviewed and their earlier bytes archived;
they are not silently presented as frozen artifacts. Acceptance records root
reproduction, the actual shared reader, schema/provenance checks, independent
mathematical review and all final hashes. After acceptance, expectations and
their generators stay frozen; later supplements are additive. A production
mismatch is investigated rather than repaired by rewriting expectations.

## Artifact attestation

The new [attestor](../validation/generator/treasury_preproduction_check.py)
is an integrity tool, not a financial oracle. It compiles a small local driver
that includes the actual unchanged shared reader through a Rust path module.
It checks common pins/censuses, each root-approved schema/input list, generator
`--check` exit codes, reader acceptance, and unchanged tracked/new artifact
bytes through the whole run. Logs and an actual-time JSON report are written
only into a new external evidence directory. Existing directories are refused.

With the documented Rust1.99 toolchain on PATH, run:

```sh
python3 -I validation/generator/treasury_preproduction_check.py \
  --output-dir /private/tmp/sabiquant-treasury-preproduction-verified-v1 \
  --all-fixtures
```

All four corpora and all generator reproductions are required for its complete
scope. A subset or `--skip-reproduction` run is explicitly interim evidence.
Missing inputs, unreviewed schemas, mismatched pins, changed artifacts or failed
commands cannot close the gate. Successful attestation still does not mean a
Rust Treasury implementation passed, establish useful ideal accuracy for every
extreme quantity, or close broader financial/platform/owner/M8 acceptance.

The [integrity controls](../validation/generator/treasury_preproduction_check_test.py)
use synthetic temporary schema records only. They test deliberately corrupted
pins, changed contracts/requirements, counts/profiles, duplicate semantic
inputs, answer-column substitution, unreviewed schemas and malformed files
through the real reader. Their passing count belongs to validation tooling;
they are not Treasury financial cases or library regression tests.
