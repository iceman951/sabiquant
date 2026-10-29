# Existing enclosure price references in FS1

`enclosure_prices.py` transports the existing 173,448 financial price cases
from `validation/enclosure/references/prices.jsonl.gz` into
`validation/fixtures/bonds/enclosure-prices-v2.fixture`. It retains every original
case field, including all exact rational or normalized dyadic values and the
price certificate ID. The original gzip files and certificates remain unchanged.
Each generated row is reversed and compared with its original JSON case.

This program does **not** generate a financial target, evaluate a financial
model, rerun a certificate checker, refine a bracket, or read Rust output. It
adds exact expected representation metadata to already sealed mathematical
references. The source and its input hashes are frozen before first generation
in `enclosure_prices.provenance.json`, which is also pinned by the FS1 header.

From the repository root, with Python 3.9+ and its standard library:

```sh
python3 -I validation/generator/enclosure_prices.py --check
```

To create a missing fixture, use `--write`. Existing files are rejected even
if identical; `--check` never writes. `--output PATH` selects a different output
for either operation. Reproduction is deterministic: the header uses the
recorded source-freeze time, not the current clock or host platform.

The active transport is version2 of the same reference family. The 20 ASCII TSV
column names satisfy the unchanged FS1 lowercase-name grammar:

| Columns | Meaning |
|---|---|
| `id`, `source_id`, `model`, `basis`, `yield_bits` | Original case identity and public call selection |
| `reference_kind` | `exact_rational` or `certified_interval` |
| `reference_n`, `reference_d` | Original reduced signed numerator and positive denominator; `-` for a certified interval |
| `reference_precision` | Original precision1120 for a certified interval; otherwise `-` |
| `reference_lower_sign`, `reference_lower_m`, `reference_lower_k` | Original normalized lower endpoint fields; otherwise `-` |
| `reference_upper_sign`, `reference_upper_m`, `reference_upper_k` | Original normalized upper endpoint fields; otherwise `-` |
| `outward_lower_bits`, `outward_upper_bits` | Tight directed finite Binary64 endpoints around the rational or reference bracket; `-` means that finite endpoint is unavailable |
| `point_encoding`, `point_value` | Exact-rational preferred-fit classification described below |
| `price_certificate_id` | Original ID into `price-target-certificates.jsonl.gz`; all model/source/quote bindings are retained and checked |

A dyadic endpoint denotes
`sign * int(m,16) * 2^(int(K,16)-(reference_precision-1))`. Both `m` and the
signed `K` metadata use radix16: `K=80` means128 and `K=-17` means-23. This
applies to digit-only strings too. Original strings are copied without reducing
or rounding them. Rational fields are copied unchanged.
The `-` sentinel is never a numerical zero.

Bits are **bare sixteen-character lowercase hexadecimal**, consistent with the
boundary fixture. A Rust consumer uses `Record::text` and
`u64::from_str_radix(text,16)` followed by `f64::from_bits`; the fixture reader's
`real_bits` syntax requires a `0x` prefix and must not be used directly here.
Decimal `point_value` is `coefficient:scale`, with a signed i128 coefficient
and scale0..38. Binary `point_value` is the exact bare bit pattern. All successful
zero bits and decimal zero are canonical positive zero.

For an exact rational, `point_encoding` is `Decimal` if canonical DecimalAmount
fits, then `Binary` if finite Binary64 is exact, otherwise `NoFittingPoint`.
The decimal check examines every permitted scale with exact integer division,
including the asymmetric i128 minimum. `point_value` is `-` for NoFittingPoint.
This is the reference value's representability, not an assertion that every
producer path must discover every algebraic identity or use a particular proof
tag. The adopted call contract determines when known-point fitting is required.

For a certified bracket, `point_encoding=NotProvedInterval` and `point_value=-`.
A point lying inside a reference bracket is not thereby proved exact. A public
consumer must prove **both** returned lower <= reference lower and returned
upper >= reference upper, or report an unproved/failing comparison. Comparing
finite returned Binary64 bounds with the two generated outward bit columns is
equivalent to this exact nested-reference test. Overlap, midpoint, epsilon and
reference-bracket membership of an asserted point are insufficient.

The converter uses exact integer/Fraction arithmetic for directed IEEE rounding,
including signed tiny values, subnormals, MAX and missing finite endpoints. It
contains a few fixed encoding seam assertions, with no additional financial
family or output corpus. The copied 173,448 references and their old scope and
independence qualifications remain unchanged. The B2 boundary fixture is separate
and is neither read nor modified by this program.

The first transport attempt stopped before creating a fixture because it
incorrectly parsed `K` as decimal. Its exact source/provenance and actual failure
record are retained in `enclosure_prices_history/`. The corrected source was
frozen again before conversion; no original reference bytes were repaired.

The first public standard-reader check then rejected version1's uppercase `K`
column-name suffix at line22. The entire invalid version1 fixture is retained
as deterministic `enclosure_prices_history/invalid-fs1-v1.fixture.gz`, with its
source, provenance, result and actual failure history. Decompression recreates
the original invalid file byte for byte; that invalid representation was removed
from the active fixture directory. Version2 changes the two column names to
lowercase `k` and updates legitimate version/source/provenance/time/reproduction
header metadata. All173,448 numerical row bytes are identical. Its header uses
the actual version2 freeze/generation epoch, never the earlier invalid transport's
date. Original reference-family dates remain in the original input provenance.
No mathematical value, reference, frozen reader or B2 fixture changed.
