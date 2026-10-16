# M8 phase A amount implementation review

Review completed 2026-10-16, Asia/Bangkok, on the local worktree based on
`4da3f43`, before the phase was committed. Two read-only reviewers worked
independently against the frozen [amount contract](M8_AMOUNT_CONTRACT.md):
one on the arithmetic kernel and public operations, one on documentation and
evidence. Neither edited repository files. The integrator verified each
finding before changing anything.

## Kernel and arithmetic review

No defect was found in `src/amounts.rs` or `src/amounts/uint.rs`. The
reviewer confirmed the following by reading the code:

- Carries and borrows are retained.
- Multiplication rejects any limb beyond 512 bits.
- Shifts never index past the eighth limb.
- Aligned division never forms an unchecked doubled remainder.
- The fused operation's widths stay within 381 numerator and 508 denominator
  bits, and sum accumulators stay below 2^318.
- The contract's error order holds: scale before range, Inexact before range,
  zero denominator before every zero-operand shortcut, quantum scale before
  positivity, and the parser's length → lexical → precision → range order.

A scratch differential harness then exercised the public API with overflow
checks and debug assertions enabled. It compared results against a separate
Python Fraction oracle written from the contract text. There were no
mismatches in any of these:

- About 1.82 million general and adversarial cases, including exact ties and
  near-ties for every rounding path and random quanta up to 127 bits.
- 400,000 extreme fused cases reaching 381/508-bit operands.
- About 400,000 results landing on the signed range boundaries.
- 4,500 cancelling sums in five orders each, plus six shuffled sums of about
  one million terms.
- 600,000 parser fuzz inputs.
- Display/parse round trips and Ord/Eq/Hash agreement on 220,000 pairs.

A counting allocator observed zero heap allocations across arithmetic,
parsing (including errors) and Display. That scratch work is not retained
evidence; the retained suites are listed in the
[validation report](M8_AMOUNT_VALIDATION.md).

## Documentation and evidence findings

The evidence reviewer confirmed:

- the 77-type and 111-variant catalogs;
- every cited FR-AMT identifier;
- the contract hash and its worked examples;
- the oracle `--check` and its census;
- that earlier fixtures and generators are unchanged.

It reported seven minor and three nit documentation findings. All were
resolved:

| ID | Finding | Resolution |
|---|---|---|
| D1 | New README links were relative `docs/` paths, which break in the packaged README | Repository-qualified URLs, as everywhere else in the README |
| D2 | ARCHITECTURE called later phases "adopted stages" | Now "later stages that are not yet adopted" |
| D3 | Oracle report byte count disagreed with its pin table | Corrected to 47,836,843 bytes |
| D4 | Validation report linked a missing review | This document |
| D5 | Validation index lacked an M8 section and had a stale "current" inventory | M8 section with reproduction commands; inventory 2,546,321 rows in 60 files |
| D6 | Living documents still said validation was in progress | README, NI-01, readiness and oracle report now link the validation report |
| D7 | F-CORE-09 was missing from the implementation-status table | Row added; header summary mentions phase A |
| D8 | Amount design still read as having no adopted contract | Marked as a historical proposal superseded by the contract |
| D9 | Oracle report kept pre-implementation wording | Past tense, adopted model, consumer pointer, `--check` metadata reuse stated |
| D10 | Suite descriptions overstated census recomputation, grammar coverage and FR scope | Wording narrowed; the consumer cites FR-AMT-01..12 |

## Integrator finding

The oracle generator first wrote a nonstandard `---` fixture layout, without
three required header keys, so the shared reader rejected it. Only the
serialization was changed, to the standard header, blank line and column
line. The 277,577 records are byte-identical to the first generation; their
digest equals that generation's body digest.

## Remaining limits

The reviews concern the local phase A definition. They do not establish the
binary64 boundary, settlement adapters, currency or minor-unit policy, bond
calculations, or owner, financial, platform and release acceptance.
