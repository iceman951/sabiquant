# Independent Treasury source, date and cash corpus

Initial candidate,2026-10-21 Asia/Bangkok. Root acceptance and the complete
four-corpus pre-production gate remain pending. No Treasury library Rust
was used or executed. This corpus implements the adopted
[main contract](M8_US_TREASURY_CONTRACT.md),
[cash component](M8_US_TREASURY_CASH_CONTRACT.md) and
[76-family test plan](M8_US_TREASURY_TEST_DESIGN.md), following the
[pre-production protocol](M8_US_TREASURY_PREPRODUCTION_PROTOCOL.md).

The producer is
[treasury_source_cash_oracles.py](../validation/generator/treasury_source_cash_oracles.py);
the new candidate is
[treasury-source-cash-oracle.fixture](../validation/fixtures/bonds/treasury-source-cash-oracle.fixture).
It contains30,572 distinct serialized inputs,53 columns and38 profiles.
Those counts are fixture records and partitions, not passed Rust tests or
38 completed requirements. The76 semantic families remain purposes whose
full obligations also need the other independent corpora, implementation,
mutants, library execution and final coverage/package verification.

## Authorship and correction provenance

Claude peer `sabiquant-a2` authored the initial provisional generator from
the adopted contracts before Treasury production. Codex took write ownership
after completing its separate blind38-test source/cash hand suite. Reading
and completing this generator is therefore **post-hand-read producer work**.
Neither completed hand source nor its expectations/document were changed.
The earlier numeric28 hand pair also remains unchanged. No QuantLib C++ or
future Treasury Rust was read. Existing accepted artifacts stay frozen.

No source/cash output had been accepted when these corrections were made:

| Provisional issue | Correction and retained discriminator |
|---|---|
| Month-end detection constructed year10000 for9999-12-31 | Pure Gregorian month lengths in route A; independent `calendar.monthrange` in B; valid Date::MAX rows remain |
| Approximate Decimal division crossed an exact AI5 tie | Route B now carries unreduced integer numerator/denominator forms to exact Decimal integer divmod/comparisons, with Inexact/Rounded trapped; C=.375,s184,t23 gives count23437.5→23438 and pubAI.023438 under both rules |
| I.A.3 cents assertion used a noncanonical amount encoding | Printed828.40 is canonical `8284:1`, not `82840:2`; the financial amount is unchanged |
| I.D(1) expected per100 AI had an erroneous denominator | Source published16.87500 per$1,000 is135/8; division by10 gives27/16 per100, not27/16000 |
| II.G expected published AI literal was incorrect | Printed.825820 is41291/50000. Raw C/2×31/183=403/488; selected daily9=.266393443 per$1,000 gives8.258196733, then AI5=8.25820 per$1,000 |
| B's total-invoice price multiplier depended on AI's truthiness | Explicit recipe selection always uses `10*B_price+published_count*R0/10^5` with its per1000 divisor, including AI0; it never switches the price unit when the addend is zero |

The source-unit corrections are checked against the pinned primary-source
checker and [source-example report](M8_US_TREASURY_SOURCE_EXAMPLES.md), not
chosen merely to match another provisional output. The source XML snapshot
and checker hashes are retained in the fixture. Source statements that do
not specify a full maturity cycle use local completions: I.A.3's1992-06-30,
I.A.4's1996-02-15 and I.D(1)'s2005-05-15 are the producer's explicit supplied
maturities, not newly quoted auction dates. Their original printed
first-coupon/accrual anchors are preserved separately.

The original failed generator `e1b48986…` is retained at
`/private/tmp/sabiquant-treasury-sourcecash-provisional-v0/`.
The takeover snapshot is
`/private/tmp/sabiquant-treasury-sourcecash-codex-takeover-v1/initial-generator.py`,
SHA-256 `eaa072adbeb1b7b8ac0ea2235c4e3485f148bc039e6362af380073a03d250ff9`.
That snapshot already includes Claude's month/Decimal-route corrections.
Intermediate source versions, failed diagnostics and successful external
candidate-v2 remain preserved; no failed run produced the accepted output
or silently rewrote a frozen fixture.

The additional zero-AI controls are deliberately labelled post-hand-read
oracle supplements. Their7,560 rows cover all four source policies, all
seven modes, regular/short/long definitions with zero/nonzero C, S=D and
actual coupon S, zero-C settlements between coupons, two nominal/quantum
pairs, signed/nonzero/zero prices and both invoice aggregations plus dirty.
For example N1000/P100 must determine1000 with AI0; the old B bug gave100.
Stage/context and the recipe remain present even when accrued is zero.

## Exact routes and precision capacity

Route A is authoritative. It validates the contract's ordered constructor,
plan and selector domains; uses pure Gregorian direct maturity anchoring;
forms raw/source coupon and raw/published AI Fractions; then applies exact
signed integer quotient/remainder quantization and final canonical range
admission. Nonpoint Exact is a structural unresolved outcome over continuous
real interiors, even for adjacent binary64 endpoints. Individual components
remain wide through component quantization and net cancellation.

Route B decodes serialized input fields separately. It rebuilds valid cycles
with `calendar.monthrange`, derives accrual as civil-interval overlaps,
recomputes source values from main E's `X/(10^dscale*s_part)` forms, and
uses cash T9's fused integer ratios. Binary evidence is independently decoded
using exact Decimal(binary64) onto R0. Money decisions use Decimal **integer**
divmod, sign/magnitude and exact remainder versus denominator-minus-remainder
comparisons. Canonicalization and final range admission have separate
Decimal-integer implementations. Binary AI views use a separate integer
IEEE RNE extractor rather than route A's Fraction-to-float conversion.

Both1536- and2048-digit contexts trap Inexact and Rounded. There is no
early approximate rational quotient, precision refinement, tolerance or
Fraction-derived Decimal boundary decision. Required significant precision
is bounded independently of the observed row maxima:

| Operand/work | Sufficient decimal digits |
|---|---:|
| Exact finite-binary64 Decimal coefficient, bounded by `(2^53-1)*5^1074` | 767 |
| Integer `R0=2^1074*5^38` | 350 |
| Their unreduced product, even before trailing-zero cancellation | 1117 |
| Other T9 integer operands/carries, including a conservative extra digit | 734 |

Thus1117 digits suffice;1536/2048 are fixed capacity choices. Decimal's
exact constructors, integer divmod, parity, addition/subtraction and bounded
canonical reduction fit. Traps ensure that an accidentally inexact operation
fails rather than picks a boundary using an approximation.

Each context was called for all30,572 records and compared242,410 monetary
value/observation/decision cells. Valid-cycle checks total61,060 across both
contexts; overlap-piece checks total38,790. A call count is not a full second
verification of every output column: constructor/plan/index/nominal error
precedence, some inspector metadata and guarantee labels come from route A's
contract ledger and explicit input assertions. Invalid constructor rows have
no successful B model. This qualification is separate from the independent
root arithmetic/schema review and the already frozen blind hand controls.

## Partitions and meaningful limits

| Partition | Records / evidence |
|---|---|
| Date/role/constructor/plan profiles | Ordered failures, all accepted roles, D=I/D<I/long I>Q1, direct EOM/clamp, Date::MIN/MAX, actual full civil span and lifecycle resets |
| Table1 day counts | 72 valid cycle rows spanning leap/ordinary years, day1/day15/month-end variants |
| Table2 daily | 2,560 rows:640 rate/half-year cells under four explicit policies |
| General-C daily ties | 64 rows including ties at each s181..184 |
| Staging grid | Full116,800-cell independent mathematical census;1,342 selected public cells×4 policies=5,368 fixture rows |
| Total-first AI | 96 two-piece long-first controls |
| Coupon39 scales | 5,616 rows, representative coefficients including i128::MAX, roles, views/cash/publication |
| Cash39 scales / evidence39 scales | 468/468 representative rows |
| Coupon/accrued cash grids | 1,679/812 rows with explicit modes/nominals/quanta |
| Price evidence / mode sweep | 1,344/1,176 rows, decimal/binary/declared kinds, signs, extremes and aggregations |
| Zero-AI price supplement | 7,560 rows closing the discovered denominator-selection gap |
| Remaining profiles | Explicit source anchors, view identity, tiny/zero C, arbitrary nominal, precedence, exactness, wide cancellation, endpoint pointness, signed limits and provenance |

The116,800 census is computed, not merely copied into a header: raw/staged
AI5 ties584/288, rule-sensitive counts292/144, basis-changing cells270 per
tie rule and350 under either rule. All640 t=s cells are not single-piece
accrued public states, because an actual regular coupon resets accrued and
a full coupon is H; none is a discriminating cell in that census. The
fixture emits every public discriminating cell plus one half-period control
per(C,s), rather than claiming that116,800 library plans were executed.

All39 scales appear for C, nominal, quantum, view quantum and decimal price
evidence, with representative coefficients and both signed final limits.
This is not every Cartesian product. Source-view and final money errors
cover all eight stages and point/lower/upper endpoints. Nineteen applicable
source/date/cash TreasuryError kinds appear; numeric quote/PV/reduction,
allocation, capacity and defensive zero-dirty outcomes belong to other
corpora or explicit proof/resource classifications.

Observed exact integer widths include source X161bits, publication count147,
coupon-cash denominator397, coupon-view denominator265, price endpoint2187,
invoice signed sum2190 and price numerator2433. They stay inside the adopted
width proofs. These observations do not prove all bounds or assert that every
conservative bound is attained financially; all per-operand runtime checks
remain active.

Expected outcomes are12,869 Determined,283 QuantumAmbiguous,112 unresolved
Exact stages,11,304 Some views,24 None views and815 successful
instrument/plan operations, plus the structured errors recorded in headers.
These are oracle outcomes, not Rust pass counts.

## Exact schema and semantic input declaration

The schema1 fixture uses the unchanged shared reader. In exact column order:

```text
id profile op dated issue maturity coupon first quasi eom date_cap basis tie settlement bundle_cap coupon_index recipe nominal evidence aggregation quantum mode view_quantum view_mode expected context first_period first_quasi coupon_count raw_first source_first next_quasi r s n bundles entitlements raw_ai raw_ai_bits pub_ai pub_ai_bits pieces recipe_id src_index src_date settle_pair day_ratios dated_pair source_value lower upper guarantee identity_group
```

The complete22 serialized input columns, in their hashed order, are:

```text
op dated issue maturity coupon first quasi eom date_cap basis tie settlement bundle_cap coupon_index recipe nominal evidence aggregation quantum mode view_quantum view_mode
```

They describe the composite operation: domain-value construction, Treasury
construction, optional plan, selector/factory, optional instruction source
view or evaluation. Profile, ID, annotation and all28 answer columns are
excluded from uniqueness. Every serialized input tuple is unique. Some
input encodings intentionally canonicalize to equal public values, such as
noncanonical Decimal coefficient/scale pairs or the two binary zero signs;
the count is not a claim of30,572 distinct canonical financial definitions.
Independent whole-corpus reconstruction counts30,506 canonical semantic
input tuples and66 additional serialized encoding aliases. These are
complete composite-call input tuples, not a count of different instruments
or successful operations.
Repeated complete encoded tuples retain the first record and annotation;
identity groups are aids rather than an exhaustive equality-pair registry.

Encoding is declared in the fixture header: decimal `coef:scale`; reduced
rational `N/D`; ISO civil dates; explicit full Q lists; `-` not applicable;
`none` Option None; binary evidence16hex bits; decimal/binary point/declared
evidence kinds; sorted contextual key/value fields with string values encoded
as UTF-8 hex. `bundles=first..last` uses absolute Q indices;
`entitlements=ceased/coupon_count`; `pieces` and `day_ratios` retain t/s.
Raw/published AI bits are separate from exact rational columns. The three
date-overflow rows pin the outer Date wrapper/index/phase only: the adopted
E5 contract does not select the nested date primitive's exact fields.

Common headers include adopted main/cash and fixed26-requirement-row pins,
generator pins, body/record/input SHA-256/FNV, positive JSON profile counts,
source checker/XML provenance and the complete schema/input declarations.
Root independently inspected and registered this exact53-column/22-input
schema in the attestor. The independent reviewer confirmed all eight stable
metadata identities and the exclusion of labels/answers. The actual shared
reader and complete four-corpus run remain required; the producer did not
add its own schema to that registry.

An independent reviewer, without importing or executing the producer,
reconstructed all30,572 records and compared all28 output fields:
856,016 exact cell comparisons passed, including constructor/plan/factory
precedence, contextual errors, source and price endpoints, all7,560 zero-AI
controls and both publication policies/ties. It separately rebuilt the full
116,800-cell staging census and checked640 Table2 cells in the retained
pinned XML against1,280 NineDecimal fixture observations. Header outcome,
policy, stage, endpoint and other row censuses also match. No fresh legal
fetch, Rust execution or complete feasible-width maximum is claimed.

Private independent evidence is
`/private/tmp/sabiquant-treasury-sourcecash-independent-audit-v1/evidence.json`,
SHA-256 `29973153d31e47f4eae91d1d4a4daee0b2943dac634570ee1e9e8fb89b0e7699`;
its supplemental source/schema/census evidence is
`supplemental-evidence.json`, SHA-256
`bc3cf90c1f5977a0abad2fe951ef5103b83417051289ff1d076a085144823927`.

## Candidate bytes and actual reproduction

| Artifact / payload | SHA-256 |
|---|---|
| Generator | `046214dc324e0fab1dd5f3387de14b44b61c39547c6e2fcf4f5522896338782e` |
| Initial fixture,16,261,036 bytes | `2347711a897cbf627da1afae1ea17881aea35a6b1609b7ecd3fc8e8f538c5d31` |
| Records | `3d97562ea042b26cc51ec10de9dd6abfc7a12b86f53b72cb4fae8d67ae053d8e` |
| Columns plus records | `5f92337cfa417cc8a205b028e44a3270efe06dc580522c979a6a0debfad01558` |
| Complete encoded inputs | `2d62ccecda58179151bd51cf8882eb0f2f38405a73f21957909d5578853e8394` |

Generation used actual2026-10-20T17:12:23Z, macOS26.6.2 arm64,
Python3.9.6. It created the default output exclusively and exited0.
`--check` regenerated every byte with the frozen timestamp/platform and
exited0 identical. A second ordinary generation refused overwrite before
running, exit1; existing output remained unchanged. The successful external
candidate-v2 has identical body/records/inputs; final changes only made
post-hand-read provenance/UTF-8 decoding explicit and changed generator pins.

```sh
python3 -I validation/generator/treasury_source_cash_oracles.py --check
```

Actual logs are `/private/tmp/sabiquant-treasury-sourcecash-codex-final-generate-v1.log`,
`…-final-check-v1.log` and `…-overwrite-refusal-v1.log`. Acceptance still
requires root's independent mathematical/schema review, attestor registration,
actual shared-reader run and the complete four-corpus gate before Treasury
production. Native library debug/release, all six risk/inverse implementation
checks, semantic mutant execution, coverage/package and broader financial,
platform and M8 acceptance are not established by this producer result.
