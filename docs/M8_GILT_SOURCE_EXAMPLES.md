# Conventional-gilt primary examples and independent checks

Recorded 2026-10-18 after the generic yield/risk evidence audit. This is
pre-contract source validation for the next named-market tranche. It does not
adopt a market API, rewrite a fixture or supply a production implementation.

## Primary artifacts and inspection

The [DMO formula catalog](https://www.dmo.gov.uk/publications/gilt-market/formulae-and-examples/)
links the [1998 calculation examples](https://www.dmo.gov.uk/media/ftjpyv1z/yldconv.pdf)
alongside the [fourth-edition formulas](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf).
The five-page example PDF was downloaded directly. Complete PDF pages1,2,5
were rendered with native PDFKit and visually inspected, including all columns,
dates, ex-dividend flags and decimal formats. Pages3/4 concern index-linked
examples and are outside this comparison. The PDF skill's read-only workflow
was used. The [earlier source review](M8_GILT_SOURCE_REVIEW.md) identifies the
fourth-edition source and the distinct next-dividend symbol roles.

| Artifact | SHA-256 |
|---|---|
| Official example PDF, 5 pages | `8f491b8567c6f3a75ee2a7cbc02ccc4275810257df5533ed1fd349992465e018` |
| Complete PDFKit text extraction | `616e51583c7607dd6dc4859babc587234a28a8536945bf36eb641120df4d8109` |
| Independent checker | `0bdf495eab0bc73fb5d250f5e9266c5186a3b905d127d7cdaf8eb6c4c3b33b02` |
| Scratch evidence JSON | `a96664b5b386c405020cece1414a43b4c55f59c418d2b4a074ae8d346ad4a7f3` |

The PDF is `/private/tmp/sabiquant-m8-dmo-examples-1998.pdf`; the pinned text
is the same stem with `.txt`. Renders are scratch source-review artifacts,
not project fixtures or library inputs.

## Numerical comparison

The new [checker](../validation/generator/gilt_source_example_check.py) reads
only the pinned PDF/text and its explicit mathematical inputs. It uses no
Rust, QuantLib or observed production price/root. Python stdlib Fraction
handles rational dividend/accrued calculations; Decimal160/240 independently
evaluates prices and solves inverse equations.

| Source comparison | Actual result |
|---|---|
| Conventional prices on page1 | All8 match at6 decimal places |
| Conventional inverse yields on page2 | All8 match at9 decimal places, including suppressed trailing zeroes |
| Long-first daily accrued table on page5 | All214 dates from1999-05-08 through1999-12-07 match, with no gaps/duplicates; the time-variable column also matches |
| Separately printed accrued anchors on page5 | All3 match at6 decimals |
| Published long-first dividend | Matches at6 decimals |
| Zero-yield controls | Eight independent sum/geometric-limit checks pass for n=0,1,2,33 |

The ex-dividend tail of the daily table displays5 decimals; its three
separate worked anchors display6. Positive daily cells display up to6.
The checker preserves these formats rather than assigning every cell a
common tolerance. Historical ex status is supplied explicitly; this does not
test a maintained UK calendar or infer current legal cutoff dates.

For each price case the checker evaluates individual quasi-period powers and
a separate compact geometric expression. It compares the two well away from
their reference-precision error. The inverse uses an explicit sign-changing
bracket and340 bisections, whose coordinate width is below1e-102. Both
precisions agree on every serialized60-decimal anchor. These are finite
high-precision comparisons of printed source cells, not a formal interval
implementation, universal Decimal error proof or binary64 production oracle.

## Derived staging discriminator

The following is an engineering deduction from the separately checked source
inputs and formulas, rather than another printed result. For coupon6 per100,
the exact long first dividend in this example is

```text
unrounded = 3 * (1 + 31/182) = 639/182
published = 3.510989
unrounded - published = 1/91000000
```

Accrued before the skipped first quasi date is `3*t/182`. In the following
quasi period its cum-side expression is `3*(31/182+t/183)` and its ex-side
expression is `3*(t/183-1)`. Subtracting the published rounded dividend from
the cum-side value instead would add `1/91000000` to the prescribed ex value.
Thus a named plan needs distinct published cash-flow and unrounded accrued
ingredients. Generic earned-minus-detached-coupon accrual cannot silently
substitute for this source model. The difference is small but structural;
it must be modeled before any numerical comparison allowance is selected.

The example also supplies an explicit skipped quasi date. At that date
accrued is nonzero because no dividend is paid there. A schedule that keeps
only actual payments would lose information needed for both price exponents
and accrued calculation. The design must retain the complete quasi lattice.

## Reproduction and remaining adoption work

The text extraction format is deterministic for the inspected PDFKit output:
`pages=5`, followed by each `--- PDF PAGE N ---` separator and the complete
`PDFPage.string`. Its exact hash is required because extraction layouts may
vary across PDF engines. A different extraction must be checked and recorded
separately; the script does not silently trust text just because a PDF path
was supplied.

```sh
python3 -I validation/generator/gilt_source_example_check.py \
  --source-pdf /private/tmp/sabiquant-m8-dmo-examples-1998.pdf \
  --source-text /private/tmp/sabiquant-m8-dmo-examples-1998.txt \
  --output /private/tmp/sabiquant-gilt-source-examples-reproduction.json
```

Output must be a new file. The actual run produced
`/private/tmp/sabiquant-gilt-source-examples-v2.json` and exited zero. No frozen
market fixture was generated before a corresponding contract exists.

The next design must resolve complete quasi-date admission, first-dividend
rounding, exact accrued ingredients, separate clean/dirty and settlement
stages, full yield/risk definitions, numerical/resource bounds and typed
capability rejection. Published example agreement is one input to that work;
it does not close all conventional instruments, other named markets, native
platforms, financial approval or full M8 acceptance.
