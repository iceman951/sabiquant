# Existing admission references in FS1

`enclosure_admission.py` projects the same 119,631 cases in the frozen
`validation/enclosure/references/admission.jsonl.gz` into the standard FS1
`validation/fixtures/bonds/enclosure-admission.fixture`. The converter copies
every original field and reverses every row to the original object. It does
not evaluate a predicate, recompute an affine, decode a floating value, run an
old numerical checker, or read native output. The 7,410 referenced source plans
are an existing subset of the source fixture, not additional constructions.

From the repository root, using Python 3.9+ and only its standard library:

```sh
python3 -I validation/generator/enclosure_admission.py --check
```

`--write` creates a missing fixture and refuses to overwrite any existing file.
`--check` never writes. `--output PATH` selects an output location. Rendering
and all reverse comparisons finish before the output is opened. The source and
input hashes are recorded before generation in `enclosure_admission.provenance.json`.
The fixed header timestamp comes from that freeze, making reproduction independent
of the host clock or platform.

The 13 ASCII TSV columns are `id`, `model`, `source_id`, `yield_bits`,
`predicate`, `canonical_quote_bits`, `attempted_quote_bits`, `frequency`,
`original_flow_index`, `exact_affine_n`, `exact_affine_d`, `tau_days`, `witnesses`.
The original five predicate variants are preserved with their original spelling:

| Predicate | Original additional expected fields |
|---|---|
| `MathematicallyAdmitted` | `canonical_quote_bits` |
| `NonFiniteYield` | `attempted_quote_bits` |
| `YieldNotAboveFloor` | `frequency` |
| `NonPositiveSimpleDenominator` | `original_flow_index`, `exact_affine` numerator/denominator |
| `NonPositiveMoneyMarketDenominator` | `tau_days`, `exact_affine` numerator/denominator |

Absent columns contain `-`, which is not a zero. Empty witness lists also use
`-`; other lists are comma separated in their original order, with no sorting or
deduplication. The frozen input contains no conflicting witness delimiter.
An affine retains its exact original signed decimal numerator and positive
denominator strings; no rational normalization or arithmetic takes place here.
Frequency, flow index and maturity days retain their original integer values.

All quote fields use **bare sixteen-character lowercase hexadecimal**, including
nonfinite payloads and originally attempted signed zero. A Rust consumer reads
these through `Record::text` and `u64::from_str_radix(text,16)`; the shared fixture
reader's `real_bits` syntax expects a `0x` prefix. The converter never replaces
original bits with an observed value or reclassifies a logical predicate.

This fixture establishes mathematical admission expectations. API resource stops,
error spellings, counts and debit cutpoints remain governed by the committed
contract/protocol and their actual future public consumers. No producer execution,
budget golden, new source plan, price/cash family or M8 completion is implied.
