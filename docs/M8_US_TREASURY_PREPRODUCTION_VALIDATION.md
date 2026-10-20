# Treasury complete pre-production verification

Recorded2026-10-21 Asia/Bangkok, before Treasury Rust production began.
The complete [main](M8_US_TREASURY_CONTRACT.md)/[cash](M8_US_TREASURY_CASH_CONTRACT.md)
contract pair remains frozen. All four independently derived corpora and
both blind hand suites are accepted and frozen after the reviews and actual
checks below. The [protocol](M8_US_TREASURY_PREPRODUCTION_PROTOCOL.md)'s
four-corpus prerequisite is closed. This permits implementation; it does
not establish a Treasury library test pass or financial acceptance.

| Frozen corpus | Rows | Columns | Independent reference scope |
|---|---:|---:|---|
| [Source/date/cash](M8_US_TREASURY_SOURCE_CASH_ORACLES.md) |30,572|53| Full composite public operations, all four source policies and all four cash recipes |
| [Numerical primitives](M8_US_TREASURY_NUMERIC_ORACLES.md) |5,593|79| Prescribed p256/extended53/IEEE, source AI, exact risk sums and residuals |
| [Ideal price/risk](M8_US_TREASURY_IDEAL_ORACLES.md) |714|74| Independent ideal dirty/clean and all six risks with individual conditional bounds |
| [Ideal inverse](M8_US_TREASURY_IDEAL_ORACLES.md) |145|51| Independent target-driven roots, conditioning and excluded ideal-image control |
| Total fixture records |37,024|—| Reference records, not passed Rust test functions |

The28 [numerical](M8_US_TREASURY_HAND_DESIGN.md) and38
[source/cash](M8_US_TREASURY_SOURCE_CASH_HAND_DESIGN.md) blind public test
bodies were frozen before production. Rustfmt parsed/checked them, but they
had **not been compiled or run** at this gate. Their authorship predates
reading the corresponding oracle expectations. Later completion of the
source/cash generator by its hand author is explicitly post-hand-read work;
the frozen hand bytes were not edited.

## Mathematical challenge and limits

The source/cash producer uses exact Fraction/integer decisions and separately
decoded Decimal-integer routes at1536/2048 digits with Inexact/Rounded
trapped. A third author independently reconstructed all30,572 rows and all28
answer fields:856,016 cell comparisons, including absent-field markers,
matched exactly. It separately rebuilt the116,800-cell staging census,
the5,368 selected public grid rows, all7,560 zero-AI controls and17 header
censuses. Retained primary XML Table2's640 cells matched1,280 NineDecimal
fixture observations. The22-field serialized projection is unique;30,506
canonicalized diagnostic tuples retain66 deliberate encoding aliases.

The numerical reviewer independently challenged every operation partition:
245 SEED,2,112 FACTOR,40 EXT53,491 PV,1,200 AI,1,260 TERM,229 REDUCE and16
RESIDUAL rows. It reconstructed all2,631 dated witnesses and checked all
1,802 affordable ideal factor relations and114 alternate power trees.
The21-field witness-qualified projection has5,593 identities; actual
prepared/literal primitive argument projections have5,517 tuples, including
76 repeated AI arguments across different dated witnesses. This is not a
claim of5,593 different economic instruments.

Earlier provisional failures were preserved and corrected before freezing:
Date::MAX month arithmetic, an approximate Decimal AI tie, three source-unit
or encoding assertions, zero-AI invoice denominator selection, excessive
absolute-Fraction certificate allocation, a p256 carry-cell predecessor and
the once-resolved clean residual target. The final numerical schema removes
derived labels from inputs and gives REDUCE a separate remaining-period
output. No old frozen artifact was rewritten. Each corpus report records
the exact correction and review scope.

The already accepted ideal pair retains the earlier complementary exact
dual/product and closed-form reviews. Its price/risk quantities comprise
5,638 conditional absolute comparisons,68 uninformative comparisons and6
ideal range references. Inverse references comprise136 conditioned,
8 uninformative-coordinate and1 ideal-image-only row. None is replaced by a
blanket ULP assertion or an ideal-upper-bound public rejection. Python
reference widths and observed sample maxima do not prove actual Rust
workspace use or attain every conservative mathematical bound. Family/profile
counts do not close the76 semantic execution obligations.

## Actual complete integrity and shared-reader gate

With Rust1.99 on PATH, root ran:

```sh
python3 -I -B validation/generator/treasury_preproduction_check.py \
  --output-dir /private/tmp/sabiquant-treasury-preproduction-verified-v1 \
  --all-fixtures
```

It started2026-10-20T17:23:48.668487Z and finished17:24:35.215260Z on
macOS26.6.2 arm64, Python3.9.6, rustc1.99.0. It exited0 with
`status:PASS` and **`complete_scope:true`**. All three generators' actual
`--check` subprocesses exited0 and reproduced all four corpora byte-for-byte.
The checker verified exact approved schemas/semantic input projections,
generator/main/cash/adopted26-requirement-row/body/record/input pins, unique
IDs, profile counts and UTC provenance. Registration changes only add the
two reviewed schema identities; the earlier checker logic is unchanged.

The unchanged [shared Rust reader](../tests/support/fixture.rs), SHA-256
`ba6bacbb76bd2c6ed40ffe45308dd43bca80445aed841526cd92e3b9518f216e`,
compiled with edition2024 and `-Dwarnings` and parsed **all77 fixtures,
3,075,381 rows**. The four Treasury corpora supply37,024 of those rows.
Both the473-file tracked snapshot guard and selected-new-artifact guard
passed across the whole run. The baseline commit was`5c339e0`; newly staged
corpora and reports were included in the473 tracked inputs. No Treasury
library production existed during the run. No remote action was taken.

After registering the two additional schemas, the29 synthetic integrity
controls passed again in3.340s. Deliberate corruption/mutation subprocesses
correctly write FAIL reports; the surrounding negative controls pass.
Those tooling tests do not add financial scenarios or library test counts.
Both new initial generators also refused a second ordinary generation with
exit1 and unchanged fixture bytes; `--check` is read-only.

| Private evidence | SHA-256 |
|---|---|
| Full gate `/private/tmp/sabiquant-treasury-preproduction-verified-v1/evidence.json` | `0b91013dfee5a065515126591d13663c69f53e997e936df7a779a26bb6d6420e` |
| Actual checker | `e02c1a61a38f7d3d3a1acd9823f8c83b7b1ba59b27975ac9dd4561cb0eb24aa4` |
| Source/cash independent review `/private/tmp/sabiquant-treasury-sourcecash-independent-audit-v1.md` | `2b632007ea98543f0be2c783bda94a24445d58d57a2c58d97e8200e1a21a6955` |
| Numerical independent review `/private/tmp/sabiquant-treasury-numeric-core-review-v1.md` | `3c00ae74e67047821c71d3f1c89fe59299db465931ba857066ad09c08a22b1d2` |

Exact artifact/body/input/generator hashes and separate corroboration evidence
are in the two corpus reports. The earlier two-corpus report remains a
historical `complete_scope:false` checkpoint; it has not been relabelled.

## Work authorized by this gate

Implement the entire adopted price/yield/all-six-risk/all-four-cash scope,
connect all four frozen corpora to public APIs/private production primitives,
compile and execute the66 blind tests, and audit semantic families/mutants.
Then complete native debug/release, strict Clippy/fmt/public/private docs,
error/type censuses, actual production resource/coverage classification,
extracted packaging and accurately distinguished foreign compilation/runtime
checks on final bytes. The historical gilt827/826 regression and coverage
evidence remains historical; this gate does not rerun or replace it.
Owner financial, calendar/announcement, current legal, enclosure, Canadian,
foreign-native and complete-M8 acceptance remain open.
