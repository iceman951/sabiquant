# Treasury pre-production evidence progress

Recorded after contract commit `a1b66f9` on2026-10-20. The complete
[main](M8_US_TREASURY_CONTRACT.md)/[cash](M8_US_TREASURY_CASH_CONTRACT.md)
pair and [protocol](M8_US_TREASURY_PREPRODUCTION_PROTOCOL.md) remain in force.
All four corpora and both blind hand suites are now accepted and frozen
after complementary review and root reproduction. The
[complete gate](M8_US_TREASURY_PREPRODUCTION_VALIDATION.md) passed before
Treasury production began. Earlier partial evidence below remains historical.
No Treasury API test, coverage, package or financial acceptance is claimed here.

| Required independent work | Actual evidence at this checkpoint |
|---|---|
| Exact source/date/all-four-cash corpus |30,572 accepted rows; independent reconstruction of all28 answer columns, source tables/censuses, actual regeneration and shared reader pass |
| Prescribed numerical primitives |5,593 accepted rows; every operation partition independently challenged, actual regeneration and shared reader pass |
| Ideal price and all six risk views |714 accepted reference rows; root regeneration, schema/pin checks, unchanged shared reader and complementary mathematical review pass |
| Target-driven inverse references |145 accepted reference rows; same checks pass;136 conditioned,8 explicitly uninformative,1 ideal-image-only |
| Blind public numerical hand controls |28 future integration tests, independently derived before production; Rustfmt parse/check passed, **not compiled or run** |
| Blind public source/cash hand controls |38 future integration tests; independent rational/API review and root mathematical reproduction pass; Rustfmt parse/check passed, **not compiled or run** |

The [ideal report](M8_US_TREASURY_IDEAL_ORACLES.md) explains exact geometric
references, independent160/240-digit Decimal routes, directed serialization,
source/date coverage and per-quantity limitations. The [hand report](M8_US_TREASURY_HAND_DESIGN.md)
records exact dated anchors, source/PV/risk failures, independent inverse
targets and a direct-versus-double-rounding discriminator. The separate
[source/cash hand report](M8_US_TREASURY_SOURCE_CASH_HAND_DESIGN.md) adds
constructor/source-policy/lifecycle, all four money recipes, continuous Exact,
stage pointness, wide cancellation, range order and identity controls.

## Root reproduction and independent challenges

Root ran the original blind mathematical scripts in a fresh directory and
compared every financial result with the author's preserved outputs. Both
scripts exited0 and results matched exactly. This is mathematical expectation
reproduction, not execution of the future Rust tests.

The source/cash hand author derived38 additional test bodies without reading
the source/cash oracle or future production. Root reran its standalone
Fraction/Decimal160 script in a fresh directory; all17 named mathematical
records matched the preserved author output. A complementary independent
review read every body against the contract/API and checked10 groups using
integer floor/remainders, including all four policies, signed cash modes,
39-scale constructions and the wide cancellation/pointness discriminators.
Its review found no blocker. The earlier27 decimal-parser spelling corrections
were archived and preserve exact values; neither this acceptance nor Rustfmt
parsing establishes compilation/execution of the future Treasury tests.

Root separately checked the ideal generator with exact second-order dual
numbers using product/inverse rules and individual payments, independently
of the generator's finite-geometric derivative formulas. All696 valid dated
cases and5,568 quantity equalities passed. A separate existing p256 observation
model checked692 admitted cases across dirty/clean and all six mathematical
rounded getter quantities against the proposed bounds, with zero violations;
two other cases failed an indexed PV gate. The maximum error/bound ratios
were below.70. Rounded-zero controls are mathematical checks without a claim
that the corresponding public getter admits zero; overflowed raw quantities
are excluded from finite-error comparisons. The maximum-date vector was
dual-checked at y0, not swept through the observational helper.

This root check caught a real provisional oracle defect: normalized radii
were accumulated as Modified/Convexity/Macaulay and attached in
Modified/Macaulay/Convexity column order. The author archived earlier bytes,
replaced positional attachment with explicit names, and regenerated.
Every ideal value/bit/class stayed unchanged; only the affected allowances
and their usefulness labels changed. A later qualification labels eight
inverse bounds wider than the caller bracket as uninformative. Earlier bytes
and exact column-level revision audits remain preserved. No frozen earlier
fixture, adopted formula or production result was altered.

A complementary independent reviewer reconstructed all859 source/date/AI
definitions and semantic inputs from the final fixture rows. Independent
closed-form geometric sums, distinct from the author's binary concatenation,
matched all5,712 exact ideal quantities; ideal IEEE bits/classes and80-digit
centers also matched. All144 inverse root/shift/slope/coordinate/outward
certificates and the one excluded-image case passed. The reviewer inspected
the budget formulas and confirmed that the final qualification delta changes
only eight labels. No remaining blocker was found in that stated scope.

## Actual shared-reader and integrity checks

The [attestor](../validation/generator/treasury_preproduction_check.py)
independently regenerated the final candidate ideal pair and checked the
actual generator, both contracts, adopted26 requirement rows, table/input
hashes, exact approved schemas and all declared counts. It compiled the
unchanged Rust schema1 reader with Rust1.99 and warning denial, then parsed
both fixtures:859 total rows. All455 tracked baseline files and selected new
generator/fixture/checker bytes stayed unchanged throughout.

The earlier ideal-only report explicitly says `complete_scope:false`: only two of the four
required corpora were selected. A successful subset must not open the
implementation gate. At that checkpoint the registry approved the ideal
schemas only. The current registry also approves the independently reviewed
source/cash and numerical schemas; encoding identity does not certify
financial accuracy.

The infrastructure's synthetic integrity controls passed29 tests, including
the actual shared reader and deliberately corrupt files/pins, duplicate
semantic inputs, output-column substitution, resealed changed requirements
and nonstandard JSON. Three main-level controls run actual generator/reader
subprocesses: an unchanged subset succeeds without closing the four-corpus
gate, while generator self-mutation and checker mutation fail the whole-run
guard even after successful regeneration and reader acceptance. These use
temporary synthetic roots with Git identity/snapshots explicitly mocked;
they do not produce financial reference cases.

Complementary review independently ran the earlier26 controls, reproduced the
earlier false passes, confirmed their fixes, and exercised an actual failing
subprocess with exit7. Its persisted FAIL report retained the command, exit
and log hash, and reused output paths were refused. The reviewed core was
then extended only with the inspected ideal schema registry; the output-column
test was strengthened to exactly reproduce three distinct answers masking
one repeated actual input. The final29-control run is separate from that
earlier independent review. These are tooling tests, not financial library tests.

| Evidence | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-treasury-ideal-attestation-v1/evidence.json` | `9fca4e3f4eefab5f19a1510aa449583cb279b16b9aecfe72917c0d0d1164ca95` |
| `/private/tmp/sabiquant-treasury-ideal-root-audit-v1/evidence-v2.json` | `15a14f8c331b2c8a433fe542195a6c2328c7594c238f42480323b0ae4f008575` |
| `/private/tmp/sabiquant-treasury-blind-hand-root-reproduction-v1/root-reproduction.json` | `22182929eb732f2a26ea6a4ef99ac33f81606356c95b09b093daf41d45ee5298` |
| `/private/tmp/sabiquant-treasury-attestation-review-v2.md` | `fa503c098b08ac5834f27ce0f8184574038fb08dd9987387d9d61cc9e9f48aab` |
| `/private/tmp/sabiquant-treasury-ideal-independent-review-v1.md` | `031bd1b66bc1d5f9011c862d9aa5216353f341ee21944c61a88ef32114be876f` |
| `/private/tmp/sabiquant-treasury-ideal-root-audit-v1/final-label-delta.json` | `546dae0ff8f316f874c4ef9234956f32f8b0d708f2962b666b6b4019e65c9083` |
| `/private/tmp/sabiquant-treasury-source-cash-hand-root-reproduction-v1/root-reproduction.json` | `922d1e84c9006bbae114231374798c233779a215c1cdda91fcb926b38075c39c` |
| `/private/tmp/sabiquant-treasury-sourcecash-hand-independent-review-v1.md` | `878f01bf7aaf91baa89442b91bfad9ab4e0d7333ba82c0c36fdbb3b8fcb1e063` |

The dual/budget report pins generator snapshot `e99a9ee0...`; the final
candidate is `ca822f89...`. The intervening change is the eight inverse
usefulness labels only, verified by a preserved column-level audit; price
records and all formula/allowance/root values are identical. The attestation
uses the final candidate hashes in the ideal report.

The subsequent [complete verification](M8_US_TREASURY_PREPRODUCTION_VALIDATION.md)
reproduced all four corpora,37,024 rows, and parsed all77 fixture files through
the unchanged shared Rust reader. Its report is`complete_scope:true`, with
both whole-run guards passing; the29 tooling controls passed again after
the schema-data additions. No Treasury Rust existed during that gate.

The subsequent implementation connects the frozen corpora to the actual
API/private primitives and executes the blind tests. Current native,
coverage, semantic-family and extracted-package evidence is recorded in
[implementation validation](M8_US_TREASURY_VALIDATION.md); this historical
pre-production record does not claim those later runtime results.
The earlier gilt regression/coverage/package evidence remains historical and
unchanged. Broader financial/owner/calendar/enclosure/Canadian/M8 gates stay open.
