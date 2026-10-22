# Treasury numerical implementation review

This is an independent **postproduction, read-only** review of the adopted
Treasury observation, retained risk arithmetic, source caches and bounded
inverse. No blocking mathematical defect was found in the reviewed source.
The temporary-storage census fits N8 after the explicit coefficient/raw-source
blocks and nine-word source-cache extraction were added. This is a source-level
logical-payload conclusion; it is not a compiler stack-size measurement.

The review does not adopt new financial rules, amend frozen expectations or
close native foreign-runtime acceptance. The semantic mutation evidence below
uses selected controls, rather than all 76 validation obligations.

## Inputs and provenance

The authoritative inputs are [the main contract](M8_US_TREASURY_CONTRACT.md)
(SHA-256 `50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93`)
and [the cash contract](M8_US_TREASURY_CASH_CONTRACT.md)
(`25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585`).
The public/frozen oracle and blind-hand expectations predate Treasury Rust.
This review and its supplemental mutant controls were written after reading
production. They must not be described as additional blind expectations.
No QuantLib C++ was read or used.

Reviewed files at the time of the semantic run:

| File | SHA-256 |
|---|---|
| `src/bonds/treasury/numeric.rs` | `64bd2930cea3ddb156632cd3fa5ce22be6c5fd48192a8ed7136002dd5a2954c0` |
| `src/bonds/treasury.rs` | `4edbd3d4c5478546d4d18ec9ee2ad78707f40945d0859b1d84c853ddab38976e` |
| `src/bonds/treasury/cash.rs` | `47039a9d25b40fe6ac038cd7169faf9195520a88e6866acd615837123f40011d` |
| `src/bonds/yield_risk.rs` | `85584994ffcff5e122663b26efcabf0334a2dc132591afd46475752464003d66` |
| `src/amounts/uint.rs` | `fa2b371da92f3c0aee949bd0c5b022f976f691b5dfeeff4b26c5b06234fcaaf9` |

The last two files were read to follow the actual copied arithmetic arguments,
normalization and division helpers. This review did not modify any of them.

## Mathematical findings

**N1/N2: prescribed factor observation.** `prefix_numerators` decodes the
represented yield onto the exact `2^-1074` grid. It forms `2B+Y` and
`2sB+rY` by checked signed-magnitude addition/subtraction. The seed extraction
keeps the division remainder and all discarded bits. Down/up conversion,
257-bit upward carry, endpoint product and reciprocal preserve the mandated
256-bit normalized endpoints. The generic endpoint path has nine words,
including the possible 513th quotient-alignment bit.

`discount` uses `q` for `j=0`. Otherwise it powers `b` from the leading bit,
squaring and multiplying by the original seed for every remaining one bit.
Each payment starts that tree independently. The original `bundle.j` is passed
through; missing actual coupons do not renumber the discount exponent. Both
reciprocal endpoints are computed and the low endpoint alone supplies the
once-rounded, unbounded-exponent 53-bit factor. No elementary floating power,
logarithm or intermediate IEEE factor gate participates.

**N3: source PV and dirty reduction.** `source_pv` multiplies the retained
source numerator by the factor significand and passes the exact ratio plus
exponent directly to IEEE extraction. There is no intervening extended53
product rounding. Early top-exponent classification prevents million-bit
absolute shifts. Quotient/remainder, complement comparison, subnormal units,
53-bit carry and normal/subnormal crossover follow N3. A zero source observes
its factor first and then returns canonical zero. A positive source that rounds
to zero/infinity fails at its original source index. Accepted PVs reach one
`exact_sum`; clean reduction is requested separately.

**N4/N5: complete risk terms.** The first coefficient aligns the exact
`2sj*q_hat + r*delta_hat` numerator. The second aligns all three terms
`2j(j+1)s²*q_hat²`, `2jrs*delta_hat*q_hat` and `r²*delta_hat²` on
`2*min(ed,eq)`. The middle term retains its factor two. Only the complete
coefficient, multiplied by the admitted PV significand, is rounded to extended53.
The moment uses `(r+js)/(2s)` directly. The explicit numerator blocks end the
individual coefficient operands before the nested ratio division.

All three positive magnitudes accumulate exactly on `2^-8192` in 243 words.
Raw sign/range admission occurs in the requested getter. Modified duration,
Macaulay duration and convexity divide the retained numerator directly by the
observed dirty price before their sole final IEEE rounding. They do not request
an admitted raw derivative or moment. Macaulay remains source quasi time; no
stub `b*Modified` substitution or convexity half is present. N5a's raw overflow
and signed tiny getter outcomes therefore remain independent of normalized
admission. Debug formats metadata without exposing or formatting wide sums.

A separate stdlib `Fraction` check compared the N4 second-derivative formula
with its exponent-aligned integer form for **516 coupled primitive coordinate
samples**. It included adjacent-to-`-2`, signed minimum-subnormal and zero
coordinates, ordinary yields, `1e300` and `MAX`, four valid `r/s` choices and
indices through the conservative primitive bound 19,999. Every exact identity
passed; observed numerator/denominator widths were 284/229 bits and the
observed exponent difference was `[-51,9]`. These samples support the algebra
review; they are neither a whole-domain proof nor 516 extra dated financial
states. The contract's coupled capacity proof remains necessary.

The exact check is recorded in
`/private/tmp/sabiquant-treasury-numeric-review-v1-nlz5xxue/evidence.json`,
SHA-256 `ab6fc0b5c048af84a11cca3badfc3650a54e247e53503ad8c2ae30c8a7e694f9`.

**Source caches and N6.** Raw coupon/accrual source values remain exact bounded
ratios. The parent retains the original dated/issue roles, and long accrual
keeps the skipped quasi period until the actual first coupon. Publication
rounds the total accrued source, rather than each piece. The cache converter
widens its local operands to nine words and correctly rounds the retained
ratio once; the analytic and published AI caches remain distinct. Constructor
and plan source errors precede later view/factory operations. Numeric quote
admission precedes workspace reservation and source traversal, and source
failures preserve their original indices and selected phase.

**N7: inversion and cache identity.** Clean targets receive one exact-sum shift
by the selected published AI cache. Nonpositive resolved targets return zero
counts before workspace reservation. The existing payload-carrying bounded
Brent engine receives a pure quote/PV/reduction/residual callback; it preserves
lower-first invocation, budgets and failed-call counts. There is no extra final
pricing call, automatic bracket growth or ideal-upper-image pre-rejection.

`difference` forms the exact represented signed dirty-minus-target difference.
The residual rounds that difference once at the target-fixed scale, chosen at
`2^970`. It never separately halves the operands. The amount-residual getter
uses the cached unscaled difference. Returned yield, dirty observation,
options, initial bracket, policy and both AI caches are retained snapshots.
No getter runs a solver, calendar, factor or workspace preparation.

An independent follow-up reviewed the UST-53 interior-failure obligation.
For a fixed prepared plan, exact positive b/q seeds, fixed-tree directed
p256 multiplication, reciprocal selection and extended53 rounding are
monotone across binade carries. Each nonzero-source observed PV and the
dirty sum are consequently nonincreasing in yield. Zero-source bundles
remain positive zero after observing their factor. If both bracket endpoints
admit every nonzero PV and the dirty sum, all interior values stay within
those admitted endpoint ranges. The quote domain is convex. Residual scale0
preserves exact MIN_SUB-grid differences; scale1 is selected only for targets
at least2^970, where a nonzero difference cannot halve into zero. Workspace
is reserved once, and the callback queries no dates or source definitions.
Under the adopted arithmetic bounds, a financial TrialPoint failure is thus
excluded after both endpoints admit. This is a proof qualification, not an
executed Treasury callback failure or OOM test.

The current exact-copy checker adds three explicitly synthetic shared-Brent
controls to its existing supplemental function: a third-call TrialPoint
`Err` preserves point0.5, count3 and its marker cause; a successful third call
returns its cached payload with no final callback; budget2 prevents that third
invocation. Debug and release each pass all5,593 frozen rows plus these three
controls, and copied-crate strict Clippy passes. The controls test shared
transport and do not invent a Treasury financial state. Current evidence is
`/private/tmp/sabiquant-treasury-numeric-kernel-v5/evidence.json`, SHA-256
`b28f6ee9ad2a1063894c52d52bd4a8cdaf9165d2d15ddc1bd5906bf9fad75f6d`;
runner SHA-256
`0a72c01e441aa7e2c03a9db2f58942d7f24c3972da39711b36e7797f4025d873`.
The independent proof/review is retained at
`/private/tmp/sabiquant-treasury-interior-callback-unreachability-review-v2.md`,
SHA-256 `953789c8b85a93043b45093a866d5dfe8d01f7c4d039f0e66a6f116091866055`.
The earlier v4 whole-row success and checker bytes remain preserved.

Coverage review subsequently added a dated midpoint discriminator in
[`bonds_treasury_numeric_boundaries.rs`](../tests/bonds_treasury_numeric_boundaries.rs).
With y=3*2^-52, delta_hat=RN(2+y)=2+2^-50. A zero-coupon one-payment
definition tests both a full period (q_hat=1+2^-51) and a half period
(q_hat=1+2^-52). Independent Fraction arithmetic constructs the exact seeds,
directed p256 reciprocal endpoints, selected extended53 factor, admitted PV,
three extended53 risk terms and final raw/normalized IEEE getters. Both
reciprocal endpoints round to the same factor in these two cases. Dirty and
all six getter bits are checked exactly. Wrong odd-midpoint rounding changes
four full-period getters; substituting delta_hat/2 for q_hat changes four
half-period getters. This postproduction supplement is distinct from the
frozen blind hands and ideal-accuracy corpus.

The self-contained independent derivation is
`/private/tmp/sabiquant-treasury-seed-midpoint-v3/derive.py`, SHA-256
`6549e06e47b145cf45b25f5c5f75197be6989a2b916c506600f1efd0c3568c1c`.
Its `derivation.json` SHA-256 is
`4eab7ff47bad45713186ac2a8fc755cc54b036c7723f31b5d34856396659aab5`,
byte-identical to the initial derivation used by the successful external
package preflight. Its bit extractor explicitly admits only normal positive
magnitudes, which covers every derived magnitude here; it makes no subnormal
claim. Actual final whole-tree execution is recorded separately in
[implementation validation](M8_US_TREASURY_VALIDATION.md).

The private adapter also adds two positive normalized primitive-superset
controls. The product (2^255+1)(2^256-2)=2^511-2 must round upward across a
p256 binade while its downward endpoint remains the last old-binade point.
An extended53 extraction of 1+3*2^-53 must round its odd-lower midpoint up to
1+2^-51. The integer identities were independently checked outside Rust;
they do not assert coupled-yield or dated reachability of these arbitrary
endpoint inputs. These are two added controls, not new frozen fixture rows
or proof of complete native production-region coverage.

The final current checker run is
`/private/tmp/sabiquant-treasury-numeric-kernel-v7/evidence.json`, SHA-256
`091f9521233e6eb06df90ab94c86542a604077cdf9f89ed73047bf267bb57fc3`.
Debug and release each execute all5,593 unchanged rows, three synthetic
shared-Brent controls and the two added positive primitive controls; strict
copied-crate Clippy and all original/copy/frozen/runner guards pass. The runner
SHA-256 is `960d6d2bf8e906d1787b4cb80ecdbbba1d8b6b42b902bbcb6aaf438819720287`;
its appended Rust adapter SHA-256 is
`1b4b3bb1b97ba9c80921516d46d18033590bbf5525f165da65164f08003daf91`.
The earlier v6 run used identical Rust adapter bytes and passed; v7 clarifies
only the evidence scope/names to list both new controls explicitly. All
previous versions and their source bytes remain preserved.

## N8 logical temporary census

The accounting includes live local wide values, copied by-value arguments and
nested integer helper scratch. It excludes borrowed immutable input payload
(the supplied quasi vector, instrument caches, prepared-bundle vector and
caller-owned report). A local copy of any cache field is included. This
separation follows N8's distinct bounds for owned source/bundle payload and
numeric **temporaries**. It does not depend on the compiler eliminating Copy
locals after their last use; explicit block endings provide the needed scope.

| Temporary group | Adopted ceiling | Reviewed limiting chain |
|---|---:|---|
| 243-word risk values | 24 | getter argument, IEEE ratio arguments/aligned operands and nested `div_rem`/subtract scratch; accumulator return copies also fit |
| 34-word seed/residual magnitudes | 8 | exact prefix creation and chunked seed/residual extraction; the quotient replaces its mutable input |
| Nine-word endpoint scratch | 32 | retained endpoint/tree copies plus product/reciprocal normalization, aligned division and nested scratch; source-cache extraction needs at most 14 |
| Five-word scalar values | 16 | source PV/complete risk-ratio extraction and source publication/daily rounding after explicit scope endings |

The exact-prefix creation peaks at eight 34-word logical magnitudes: its four
creation values, the previously selected numerator and the remaining checked
addition's copied operands/result. Seed extraction uses small-divisor/chunk
operations, rather than a long-division chain carrying entire 34-word operands.
Residual extraction follows the same bounded replacement pattern.

The 243-word IEEE extraction path stays below its 24-slot ceiling even when
conservatively charging the three retained report numerators in addition to
the copied raw/normalized argument. There is no nested factor allocation or
243-word coefficient computation. Endpoint argument and helper copies remain
within the separate 32-slot group; four-word stored endpoints can conservatively
be charged as nine-word slots in that count.

The parent scalar limiting chain is the second NineDecimal daily piece:
retained raw source (two), first piece (one), daily base/ratio locals (three),
`nearest` arguments (two) and conservative nested division scratch (eight),
for sixteen. ExactRatio publication uses at most fourteen. The old raw creation
operands end before this nested call. At cache extraction the five-word parent
local source/cache copies are bounded by eight and the nested division uses
nine-word scratch; it does not add a second five-word extraction chain.

The payload ceilings sum to 51,776 bytes before bounded small metadata, within
64 KiB. Compiler stack frames, inlining, ordinary user `Clone`, operating-system
stack limits and OOM behavior are explicitly outside this logical-payload
claim. The source/PV vectors retain their separately bounded heap allocations.

## Semantic mutation controls

The new stdlib-only runner is
[`treasury_semantic_mutants.py`](../validation/generator/treasury_semantic_mutants.py).
It creates each scratch crate from exact copied source bytes, applies one
unique declared semantic replacement, and appends the same two `cfg(test)`
financial controls to every baseline/mutant. Each crate must compile before a
runtime assertion failure can count as a kill. Compile failures, unrelated
runtime failures and surviving mutants have distinct classifications.

The copied public numeric hand adapter is separately pinned at
`117c38b35b3f2061f68a50808458cd3317de1ec34d93eda6b39516f96bdf64fb`;
the frozen original remains
`09676aff8114429ebfd28271830c206e2d40a895ff0b7b90817f0cc230371cab`.
Their documented differences use public interval-bound inspectors instead of
private convenience constructors, plus comments/test lint annotations; financial
values and exact endpoints remain unchanged. The source/cash integration hand
is byte-identical to its frozen `546f9e80…` original. Every scratch copy preserves
its accepted public adapter bytes.

The two supplemental public financial controls are a real 19-payment
zero-coupon price between `2^969` and `2^970`, requiring an unscaled one-call
cached inverse report, and a zero-AI clean invoice retaining the mandatory
price-times-ten per1000 units. They use valid public definitions. A third
supplemental control is explicitly a **positive p256 primitive-superset**:
it tests endpoint selection on an arbitrary normalized interval and makes no
coupled-y/r/s or dated financial reachability claim. All three are postproduction
controls, not members of the 66 blind hands or new fixture rows.

For that primitive control, choose `M=1+2^-53` and `q=1/M`. Independently,
`m=floor(2^309/(2^53+1))` is a 256-bit integer. The supplied q endpoints are
`m*2^-256` and `(m+1)*2^-256`. Their directed reciprocals strictly straddle M;
the prescribed low endpoint rounds to extended53 value1, while the high rounds
to `1+2^-52`. The fixed integer limbs in the test were independently checked
with stdlib Fraction, rather than computed by the production endpoint routine.
This exercises generic p256 helper wiring. It does not resolve whether a
corresponding midpoint straddle is reachable from a valid Treasury date/yield.
The proof is retained at
`/private/tmp/sabiquant-treasury-semantic-mutants-v2/high-endpoint-independent-proof.json`,
SHA-256 `cf321165681fc9a4e0235894601503957301f63b9946f9e26463312a5a6043a1`.

The final baseline selects 28 numeric hands, 38 source/cash hands, eight
existing Treasury private controls and three supplemental controls:
**77 Rust test functions**. The earlier two-control baseline selected76;
neither count denotes the test plan's76 semantic obligations. The maximum-date
ideal corpus is excluded from per-mutant execution. Full primitive, ideal,
source/cash, regression, package and platform gates remain separate.

## Executed mutation ledger and limits

The actual final run used Rust1.99 on native macOS arm64, from
2026-10-21 02:03:31.918128 UTC through 02:06:41.747713 UTC. Its baseline passed
all77 selected functions. **All14 semantic mutants compiled successfully and
were killed by runtime assertions**. There were zero compile-failure kills,
unrelated-runtime-failure kills or surviving mutants in this final selection.
All original Rust/test/manifest inputs and pinned Treasury frozen artifacts
remained unchanged. README/CHANGELOG are copied/attested initially but excluded
explicitly from the end guard: they are not consumed by these financial tests
and the root agent was updating living documentation concurrently.

The source snapshot, exact replacements, copied-file deltas, executable
versions/hashes, compile and runtime commands, test counts, assertion locations
and log hashes are retained in:

`/private/tmp/sabiquant-treasury-semantic-mutants-v3/evidence.json`
SHA-256 `4468082aa8df792dc792ed66d942af4b018c01eec260e82d3e703beec019f6f0`.

The runner bytes used by that run have SHA-256
`c86164434e8ed89a37b4c4461d22963fe480bb00cc5054521463f7748f80f3f3`;
the three appended controls have SHA-256
`9e36e3341f371b7a0a8f4c809fce73969269816746a632ff0d8d164ee4b653cb`.
Each row below is a compiled semantic mutation. All rows support UST-74;
additional family links identify the discriminator, not complete family coverage.

| Mutation | Related families | Runtime discriminator | Failing functions |
|---|---|---|---:|
| `j0_uses_regular_b` | UST-25/39 | fractional simple-prefix price/risk/inverse | 5 |
| `renumber_original_j` | UST-10/25 | missing original quasi ordinal and indexed PV behavior | 9 |
| `rounded_seed_prefix` | UST-21/22/27 | stub all-risk and observed-supremum exact staging | 3 |
| `select_high_factor_endpoint` | UST-27/28 | supplemental p256 midpoint primitive-superset only | 1 |
| `source_pv_double_round` | UST-31/32/33 | dated direct subnormal7 versus double-rounded6 MIN_SUB | 1 |
| `kappa_cross_half` | UST-39/40/41 | complete three-term second derivative | 6 |
| `convexity_half` | UST-39/40/47 | exact and subnormal normalized convexity | 9 |
| `macaulay_b_modified` | UST-39/49 | stub1/4 versus periodic identity41/162 | 3 |
| `normalized_from_ieee_raw` | UST-42/43/44/45/46/48 | retained normalized values after raw overflow/underflow | 4 |
| `clean_target_raw_ai` | UST-38/51 | published-AI clean target and retained report | 1 |
| `residual_threshold_969` | UST-55/56 | valid financial price below2^970, cached one-call report | 1 |
| `dated_replaced_by_issue` | UST-05/19 | D<I and long original issue after skipped Q1 | 2 |
| `long_skipped_ai_reset` | UST-05/19/20 | first accrued piece persists until actual payment | 4 |
| `zero_ai_invoice_wrong_units` | UST-62 | public zero-AI invoice and existing private units/cancellation control | 2 |

The complete selected semantic catalogue was rerun after adding the midpoint
primitive. This is14 concrete mistakes, not all possible mutations or all76
validation families. In particular, killing the high-endpoint helper mutation
with an arbitrary p256 interval does not establish a dated midpoint witness,
a uniform ideal-RNE theorem or stronger financial accuracy than N1 prescribes.
Resource guards, proof-excluded private capacity states and OOM are reviewed
and qualified, not manufactured as financially reachable mutant inputs.

The public source/cash consumer's30,572 rows include all640 daily-table
coordinates under all four policies (2,560 public source-coupon view calls).
For ExactRatio, nonterminating decimal views retain their typed failure;
the primary XML/math comparison remains separate evidence. The full116,800
mathematical staging cells (UST-15) were independently enumerated before
production; the public consumer executes the selected observable
discriminators and representatives, not every staging cell. Likewise,
its28 output-column bindings comprise23 direct/formatted public observations
and five independently checked rational projections; they do not establish
28 exact private rational observations. Neither that consumer nor this partial
mutation ledger establishes blanket completion of76 families. Current full
acceptance statuses belong to [the Treasury validation report](M8_US_TREASURY_VALIDATION.md).

Historical evidence is preserved:

- v1 compiled and passed the66 hands, then failed the author's supplemental
  setup because SolverOptions rejects absolute tolerance0. It stopped before
  running mutants, counted no kill and did not identify a production defect.
  Its evidence SHA is
  `c6ec529e1ed739d7f9dd4068cde617d429ff4c8afc3f6b3eb4a4f894c1651ce6`.
- v2 fixed that setup, passed76 baseline functions and compiled all14 mutants.
  Thirteen were killed; selecting the high endpoint survived those selected
  controls. Its evidence SHA is
  `fa58062ef4f3b8ded6dc08d1e7e6f027db88be21fda345b5f4d2f4f796adb0eb`.
  The v2 runner bytes and full surviving case remain in its create-new directory.
- v3 adds the independently checked primitive-superset control described above.
  Its14 runtime kills do not erase or relabel the earlier survivor.

Five runner controls also passed: catalogue listing14 entries, rejection of an
unknown mutation, refusal to overwrite an existing output and rejection of the
repository or its ancestor as scratch destinations. Python source syntax was
compiled without creating bytecode. Their evidence is
`/private/tmp/sabiquant-treasury-semantic-tool-controls-v1-qwz83p4e/evidence.json`,
SHA-256 `43b26172ba109b840abc112a155dc20e101caee685594b9469d2c4ae8c93229c`.
The existing evidence and runner remained unchanged during these controls.

To reproduce, use a new scratch path and the matching toolchain:

```sh
PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH \
python3 -I validation/generator/treasury_semantic_mutants.py \
  --output /private/tmp/sabiquant-treasury-semantic-mutants-new-run
```

This review/mutation tranche changes only its new runner and this new report.
Final repository regression, whole-corpus numerical verification, coverage,
packaging and native foreign-runtime decisions belong to their separate
acceptance evidence. No remote action is part of this work.
