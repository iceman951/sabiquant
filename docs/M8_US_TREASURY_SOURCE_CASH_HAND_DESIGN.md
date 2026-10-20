# Blind Treasury source and cash hand controls

The new [hand source](../validation/hand/bonds_treasury_source_cash_hand.rs)
contains38 proposed public integration tests derived from the adopted
[main contract](M8_US_TREASURY_CONTRACT.md),
[cash contract](M8_US_TREASURY_CASH_CONTRACT.md) and
[test plan](M8_US_TREASURY_TEST_DESIGN.md). It was authored before Treasury
production without reading `treasury_source_cash_oracles.py`, its fixture,
or the other new Treasury expected corpora. Only existing public amount,
date, settlement and root-error interfaces supplied API scaffolding.

The source remains under `validation/hand`, outside Cargo's test tree.
**Rust1.99 rustfmt parsing/checking passed; it has not been compiled or run.**
The Python exact arithmetic below passed independently. Neither statement
constitutes executed Treasury library tests, mutant kills or coverage.

## Provenance and frozen inputs

| Input / new artifact | SHA-256 |
|---|---|
| Adopted main contract | `50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93` |
| Adopted cash contract | `25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585` |
| Adopted76-family test plan | `5e91934b538d03297659674cc9f10f51f91b1d35dd21437472a7901d157e0629` |
| New source/cash hand Rust,1451 lines/38 tests | `546f9e802ee36249a8ab481757026098e0d5d8001496fe77bd643b7288895a83` |

The earlier numeric hand source/document remain frozen unchanged:
`09676aff8114429ebfd28271830c206e2d40a895ff0b7b90817f0cc230371cab`
and `3bc7cdf427ba400b9b30ba41a1cc59ef2e13cdf4e2b28a5186c82c3387df58ea`.
This tranche adds two repository files only; it does not modify requirements,
registry, production, existing tests/generators/fixtures or Git state.

Independent derivations are retained in
`/private/tmp/sabiquant-treasury-source-cash-blind-hand-v1/`:

| Artifact | SHA-256 | Executed scope |
|---|---|---|
| `check.py` | `fe812d8bc738fc19a66f8da191433597f2e5ac0c6f60b6d8b64318952c607322` | stdlib Fraction source/AI/cash, direct civil-date lattices, Decimal160 signed quantization corroboration and scale partitions |
| `evidence.json` | `961829614bc0dad90c81e08de695b77f08c340b209a95c4e50ad33da88e9686e` | Python3.9.6,2026-10-20T12:15:09 UTC;17 named records; terminal exit0 |

The script imports no project module, reads no production/corpus and creates
its strict-JSON report exclusively. Its report includes actual UTC, Python
version and self hash. Named records include four source-policy combinations
and four total-first-publication combinations; a record count is not a
financial coverage count. These temporary derivation artifacts need separate
preservation for durable replay; the Rust constants and the algebra below
remain in the repository. Reproduction must create a new report path rather
than overwrite the retained evidence.

The final existing-API grammar review corrected27 author-draft decimal
strings such as `.1` to `0.1`: DecimalAmount's existing parser requires an
integer digit. Their exact mathematical values/expectations did not change.
The pre-correction source/document are retained in the same temporary
directory as `source-before-decimal-grammar-correction.rs` (SHA-256
`e51f2c52ac1385bc549855b7a4fd0f8ff7cbc6cad84f1e8031a2695133b3129b`)
and `design-before-decimal-grammar-correction.md` (SHA-256
`91b10be981921c64eec831c47dc7d9626e2961a4cae35170f21006f21d22bad4`).

## Constructor, dates and source semantics

Tests1–10 specify existing contract order and actual civil inputs:

- Explicit enum arrays/const policy inspectors and compact Copy/Eq/Hash
  traits; no inferred source tie rule.
- Date cap before vector count, count before minimum length/financial fields,
  then negative C, D>I, I>=M, maturity boundary, ascending order, minimal
  coverage, direct cycle and first-coupon role.
- Original index on date mismatch; Preserve at April30 yields October31,
  while Ignore yields October30. An August30 cycle restores day30 after
  February28 instead of recursively propagating day28.
- Date::MIN and Date::MAX occur only in valid year1/year9999 cycles.
  No out-of-domain civil dates are constructed.
- Regular/Short/Long roles, D=Q0/F=Q2 rejection, I>=F rejection, strict-future
  settlement and explicit plan-cap order. Zero coupons retain actual
  entitlement and principal bundles. Actual payment on S is excluded.

The date/source expectations are from the adopted local engineering
definition. In particular D-based short/long coupon construction and long
I>Q1 are explicit local interpretations; the tests do not assert real auction
eligibility, announcement compliance or a payment calendar.

The principal daily-tie definition is Q0=2025-07-15, Q1=2026-01-15,
Q2=2026-07-15, C=23e-7 and D=I=2025-10-03. Its first fraction has104 days
out of184. Raw first coupon per100 is exactly.0000006500. The selected
short source coupons per100 are:

| Basis | Tie | Selected source per100 |
|---|---|---:|
| ExactRatio | either | .0000006500 |
| NineDecimal | even | .0000006448 |
| NineDecimal | away | .0000006552 |

The corresponding long source first coupons add exact C/2=.00000115,
giving .0000018000/.0000017948/.0000018052. No coupon-level5/6-place
quantizer intervenes. For every policy and representative quantum, all
seven modes compare the coupon-instruction source view directly with the
instrument's selected-source view, including identical Inexact errors.
A full-C long control also checks identical OutOfRange errors: its exact
source per100 is `18*i128::MAX/23`; nearest quantization to2^126 yields
2^127, outside the final amount domain. Construction/factory succeed
before this optional public-view admission.

Other source discriminators establish:

- Regular C5 coupon per$1,000 is exact25. Rebuilding it as
  `RN9(25/184)*184` gives24.999999960 and fails the exact cash expectation.
- Regular C1e-38 has a raw half coupon at scale39. An Exact1e-38 view is
  Inexact, yet fused nominal2000 cash is exact1e-37. Optional Decimal views
  cannot gate source caches or exact cash.
- Tiny C1e-38 short daily9 becomes zero while raw accrual remains positive;
  zero source cash still validates its nominal.
- C5 after one elapsed day of182 has raw AI5/364 but published AI.013736
  per100. The accrued instruction view/cash use the latter exact point.
- Long I=2026-02-01 after skipped Q1 retains D=2025-10-03 and the two
  source pieces `(104,184),(17,181)`. A separate long definition shows
  non-reset at skipped Q1 and zero/reset metadata at an actual first coupon
  before maturity, with next actual coupon ordinal1.

## Publication and nominal-stage controls

A long C=.125 definition with D=I=2026-01-14, F=M=2026-07-15,
Q0=2025-07-15 and Q1=2026-01-15 is valid. Settlement2026-01-17 has
pieces `(1,184),(2,181)`. Under all four source policies the selected
total-first published AI per100 is.001030, whereas rounding the two
components separately would give.001031. Nominal100000 and an Exact.001
cash quantum make the public result1.030, discriminating the wrong1.031.

For C=.000368 and one elapsed day of184, published AI is exact1e-6
per100 under all four policies. Nominal499600 gives exact cash.004996,
which rounds to cent0. An extra five-place stage after nominal would
produce.00500 and then cent.01 under nearest/away. Nominal500000 produces
a true final half cent: final even gives0 and final away gives.01,
independently of the source's retained tie rule.

The39 scale controls use C=2*10^-scale, nominal100 and quantum10^-scale
for exact coupon cash10^-scale. A second family uses C2000 and
nominal10^-scale, yielding cash10*10^-scale exactly at quantum1e-38.
All source/quantum/nominal scales0..38 are covered by these constructions;
they are representative exact partitions, not every coefficient/scale
cross product or proof of all private arithmetic bounds.

## Complete cash surface and exact evidence

The inspector matrix uses C5 on a regular182-day first period, settlement
after91 days and nominal1000. Coupon cash25, accrued cash12.50, clean100
invoice1012.50 and dirty101.25 trade1012.50 establish all four recipes.
Assertions cover the literal recipe/source identifiers, policy, nominal,
rounding, D/I/F/M, EOM, dated/settlement quasi pairs, actual coupon metadata,
source ratios, evidence/aggregation and selected-source Some/None views.
Dirty source views must return None even with an Exact quantum that would
be irrelevant to price. Source receipts have no price-evidence guarantee;
price receipts label only their supplied price ingredient.

All seven cash modes are exercised with quantum.25 at exact cash±.625.
Positive expected results in registry order are
`.5,.75,.75,.5,.5,.75,Inexact`; negative dirty results are
`-.5,-.75,-.5,-.75,-.5,-.75,Inexact`. Coupon/accrued/total-invoice/dirty
all use the positive control; signed dirty supplies negative rounding.
Python's exact Fraction results are independently corroborated using
Decimal160 and the six corresponding Decimal rounding modes.

A regular C2 half-period plan has published AI.5 per100. With nominal1
and clean.5, capital/accrued are each.005: total-even cash.01,
component-even cash0, component-away cash.02. Signed clean-1.5 gives
total-even-.01 but component-even-.02. Both aggregation recipes retain
their own provenance without adding a final quantizer after components.

Further exact evidence controls cover:

- Equal price1.25 supplied as decimal, binary, declared-decimal point and
  declared-binary point: equal money, four distinct instruction/receipt
  snapshots and the proper guarantee labels.
- Nonpoint declared interiors remain continuous, including adjacent binary
  floats. Exact returns an unresolved stage even if endpoints are quantum
  multiples; rounded-image collapse does not establish point Exactness.
- A genuine point's Inexact errors retain `raw coupon`, `source coupon`,
  `source accrued`, `cash coupon`, `cash accrued`, `cash capital` or
  `cash total` and the `point` endpoint as applicable. Component capital
  Exact precedes accrued work.
- Binary.1 at quantum1e-20 is Inexact, while decimal.1 is exact. No decimal
  interpretation of the binary dyadic is inserted.
- A price interval1.24..1.26 yields QuantumAmbiguous bounds. Those diagnostic
  decisions can compare equal across different source-policy instructions;
  only the retained instructions own full provenance.

## Wide arithmetic, range order and identity

The cancellation control uses valid C=2^126 on the common182-day regular
period, with half-period settlement. Published AI is exactly2^124 per100.
At nominal1000 and clean-2^124, capital and accrued are respectively
`-10*2^124` and `10*2^124`, both outside i128. With unit Exact rounding,
both total and component recipes must still determine final0. At quantum3,
total0 remains exact, while component capital is Inexact first; cancellation
cannot waive that earlier stage.

For endpoint pointness, the same AI with nominal400, clean evidence
`[2^124-1,2^124+1]` and quantum2^126 gives a nonpoint capital interval.
Both capital endpoints round to2^126; accrued rounds to2^126. The component
net input is now the point2^127, so final failure is
`cash net/point/sum`. RoundTotal instead processes a nonpoint sum image,
so final failure is `cash total/lower/quantize`. This distinguishes stage
input pointness from the fact that both final rounded bounds can coincide.

Dirty nominal100 with exact decimal evidence i128::MIN/MAX admits both
signed final limits. Doubling nominal makes the final amount out of range.
A declared range with both overflow signs must report lower/negative first;
a valid lower0 followed by an overflowing upper reports upper/positive.
Subnormal binary price evidence can round to zero cash with its binary
guarantee retained. Even minimum positive nominal cannot rescue binary MAX
into the Decimal money domain, so that final overflow is tested separately.

Definition/instruction/receipt identity preserves all four policies and
original D/I roles despite equal source money. Different resource caps do
not change definition or cash identity. Owned Copy instructions survive
definition/plan destruction; a source cash error retains its Treasury kind,
typed AmountError cause and root wrapper chain. No private report is forged
and no OOM injection or proof-excluded capacity failure is manufactured.

## Partial family mapping and execution limits

Numbers refer to the38 test functions in source order. Families are mapped
to specific intended clauses; this does not mark an entire family complete.

| Test numbers | Semantic families | Intended assertions |
|---|---|---|
| 1–4 | UST-02/04/72 | Traits/arrays, constructor precedence and typed error context |
| 5–10 | UST-05–12/73 | Civil extremes, direct/EOM cycle, roles, minimal coverage, plan caps and strict-future zero coupons |
| 11–15 | UST-14/16–18/20/59 | General daily ties, selected/raw source views, regular full coupon, scale39 and tiny source zero |
| 16–19 | UST-19/20/38/60/61/68 | Published AI, total-first publication, arbitrary nominal, final/source tie independence and39 scale controls |
| 20–22 | UST-10/19/60/63/70 | Complete four-recipe metadata, long I>Q1/reset and index/nominal order |
| 23–24 | UST-59/62/68 | Aggregation, signed ties and seven cash modes |
| 25–31 | UST-64/65/67/69/70 | Evidence guarantees/identity, continuous Exact, stage pointness, ambiguity and exact binary decoding |
| 32–36 | UST-66–69 | Wide cancellation, earlier Exact failure, signed final range order and extreme binary prices |
| 37–38 | UST-03/70/71/72 | Definition/cash identity, cap independence, owned snapshots and typed cause chains |

These hand controls do not replace all640 daily cells, all source examples,
the complete staging grid, the full source/cash oracle, every coefficient
and scale combination, private width proof, allocation/resource audit,
successful semantic mutant runs or the full public/error census. No native
debug/release, lint/docs, coverage, package or foreign-runtime result is
claimed. Following frozen pre-production artifact review, the implementation
must expose the adopted public API and execute this unchanged hand body
through a normal integration consumer. Any later correction needs explicit
independent justification and preserved prior provenance.
