# Conventional gilt private numerical kernel check

The new [runner](../validation/generator/gilt_numeric_kernel_check.py) drove all
**75,849 frozen exact rows** through the actual private gilt helpers in an
attested temporary crate. Debug and release passed. A separate, explicitly
postproduction guard reference added **272 pure numerical inputs**, also passed
in both profiles. Strict Clippy on the copied library and its tests passed.
There are **two supplemental test functions, four profile executions**; these
are separate from the normal unit/integration/doctest totals.

The [numeric oracle](M8_GILT_NUMERIC_ORACLES.md) was authored from the frozen
[contract](M8_GILT_CONTRACT.md) before gilt production. Its generator, fixture
and every mathematical row remain unchanged. This consumer was written after
production, using private interfaces mechanically. It neither selects new
expectations from observed library results nor claims that every supplied
primitive observation is produced by a public yield or native libm call.

## Actual evidence and source identity

The complete successful run is
`/private/tmp/sabiquant-gilt-kernel-adapter-v3/evidence.json`, SHA-256
`92921738146e8990b7f7bed2a4e207be8876523e44be6c783c05fe9812036fb8`.
It records the source and copied manifests, toolchain executable hashes,
arguments, native environment, every subprocess exit and output hash, both
injections, frozen inputs and post-run unchanged-byte guards.

| Item | SHA-256 |
|---|---|
| Runner | `eb3e350630b86d9d133acdaac59ca666d997f1dea14e58f451f8cae4c504909b` |
| Embedded Rust adapter | `192ef515fdcea88e0a9a826f0b1442d1b56177ccff1c3a11166b75c4f9a31a64` |
| Copied-only word accessor | `462bf783b5583d80541fbc4b4b2698c1671359690d3019c7ee0f3373ad395af7` |
| Canonical original-source manifest | `3b648b5d1414320b7bb7d9b5a1ef4e6ef831df4e20e9980c713a194d7b93c45e` |
| Frozen numeric fixture | `40a8bb169b9876906e60465805b49b95c49ac6289d0720608819505120f36915` |
| Frozen numeric generator | `be197c8408a5d3d0a394701925468d42a2eeca03b08d32f74d9ef37bbc9e7160` |
| Frozen gilt contract | `03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292` |
| Exact-copy gilt source | `ae35bd7a7d74203ebfbd8765026f65ff63216d8216f80e1c0af37dc73345c001` |
| Exact-copy gilt numeric source | `1c53413faa374a4048391b89333ab6032b54394816d5de83d412fc6eafbe148d` |
| Exact-copy gilt cash source | `ec07451e86975910bddf7af171a69db7705714df72677399fac2a887702e3135` |
| Exact-copy original Wide source | `fa2b371da92f3c0aee949bd0c5b022f976f691b5dfeeff4b26c5b06234fcaaf9` |

The runner copied **50 original files**: 39 Rust source files, four test-support
files, and seven manifest/lock/readme/changelog/license/lint files. Every copied
byte was verified before injection. Original bytes stayed unchanged throughout
the run. Only two copied files differ afterward, each by its recorded suffix:

* `src/bonds/gilt/numeric.rs` receives a `cfg(test)` child containing the adapter.
* `src/amounts/uint.rs` receives a `cfg(test)` child with an immutable raw-word
  accessor. It returns the actual stored limbs; it neither reconstructs arrays
  from expected values nor exposes/mutates the real library's storage.

No original source, public interface, frozen evidence or Git state is changed
by the runner. There is no unsafe code, dependency or interior-mutation
exception in the injected code. The copied package-list check confirms that
validation artifacts remain excluded; this is not a new extracted-package
verification. The native toolchain was Rust 1.99.0, host
`aarch64-apple-darwin`, with `-D warnings`, offline/locked Cargo and a separate
scratch target directory.

| Executed check | Result |
|---|---|
| Frozen kernel, debug | 75,849 rows; one test passed, zero failed/ignored; 14.41 seconds |
| Frozen kernel, release | 75,849 rows; one test passed, zero failed/ignored; 0.66 seconds |
| Additive guard, debug | 272 rows; one test passed, zero failed/ignored |
| Additive guard, release | 272 rows; one test passed, zero failed/ignored |
| Copied library/test Clippy | Exit 0 with `-D warnings` |
| Metadata, package exclusion, original/copied byte guards | Passed |

The first scratch run passed all frozen rows in both profiles but failed
Clippy on test-adapter plumbing: interior-mutable field tracking and an
accessor item after an existing test module. Both were replaced in the new
runner; no production or expected value changed. The second run passed all
checks without the additive guard. Both earlier evidence directories are
preserved. Between complete runs v2 and v3, root updated `README.md`,
`CHANGELOG.md`, `src/lib.rs` and `src/bonds.rs`; v3 independently captured the
new bytes and detected no changes during its own execution. Evidence is tied
to the recorded snapshot, not to unverified later edits.

## What the frozen-row adapter actually exercises

| Operation | Rows | Actual production operations |
|---|---:|---|
| PV | 7,929 | Parent source/publication helpers from original C/role/r/s, exact `with_par` grouping when requested, then `source_pv`; direct IEEE bits and typed range errors |
| Z | 45,020 | `quasi_exponent` from represented g and exact L/s; internal rounded zero, sign, MIN_SUB amplification and original coordinate provenance |
| AI | 2,932 | `source_binary` on the supplied exact signed source rational; one IEEE admission and canonical zero |
| TERM | 13,455 | Source-derived accepted PV, represented delta, `risk_terms`; exact retained significand/exponent, `extended_ratio`, actual accumulation, raw admission and direct scalar normalization |
| REDUCE | 208 | Source C/2 plus final par100, admitted discount runs, `source_pv`, all `risk_terms`, actual accumulators, `exact_sum`, raw and all normalized admissions |
| RESIDUAL | 6,305 | `residual` and `amount_residual` from original positive operands and target-fixed scale |

AI rational numerators/denominators are **primitive inputs**, not outputs
copied into the library. This check verifies their binary conversion. It does
not substitute for deriving the seven AI branches through the public source
and cash consumer. Likewise, no libm factor is inferred from a supplied
accepted discount or represented g. Factor-before-zero ordering belongs to
public `observe` tests; direct `source_pv` here receives admitted positive
factors only.

Every reduction reconstructs actual source coefficients and PVs, uses one
coherent represented delta and ordered quasi coordinates, retains positive
par100, and produces positive finite dirty. No GiltRisk or GiltYieldReport is
forged. Each actual 243-word accumulator is checked by significant bit length
and FNV of all 1,944 stored little-endian bytes. Raw and normalized getters
remain independently checked after raw range failures. A zero scalar term
has no standalone normalization denominator and is explicitly not applicable;
no zero-dirty financial report is manufactured.

Overflow/underflow checks assert the complete typed Value cause, outer gilt
kind, quantity, sign, original-index token or None, phase and strict signed
domain. Every row ID/order, operation/family/status/reachability census and
all 54 columns are accounted for; irrelevant columns must be literal hyphens.
Alternate-bit controls must differ from the accepted answer. Expected numbers
are never installed in production arithmetic or accumulator state.

## Additive actual-guard check

The new runner independently constructs a **postproduction** exact Fraction
reference for N3's rounded additions, scaling and adjacent-bit operations. Its
IEEE conversion candidates are certified with exact neighboring midpoint and
parity cells. Reference construction reads the contract constants; it does
not observe the actual helper's output. It writes a new temporary fixture
only in the new scratch crate:

`validation/fixtures/bonds/gilt-guard-adapter.fixture`.

The deterministic body SHA-256 is
`d73daaa9c23027d1dc8fd5e8b1e8fbcf858c88f449ddcdfb29366e11d6d4aba6`;
records SHA-256 is
`6cb3f904c71ca7b04ad1ec27fdc74b808e2572f9bfc1fe88d73d3e145a89e61c`.
Actual generation metadata and complete fixture hash are retained in the run.
The ephemeral file is not one of the frozen preproduction fixtures.

The 272 inputs include every adjacent float within 64 positions of each H/Z
constant, exact constant equality, both neighboring inside/outside edges of
each rejection band, signed zero, subnormals, normal finite values and signed
MAX. There are 26 boundary rejections (13 at H and 13 at Z) and 246 passes.
No exact `d_lower == G` equality was found in this input set; no such case is
claimed. Both adjacent edges and exact z-constant equality are covered.

| Boundary | Just outside / admitted | First rejected | Constant / rejected | Last rejected | Just outside / admitted |
|---|---|---|---|---|---|
| H, increasing numeric order | `40862e42fefa39e8` | `40862e42fefa39e9` | `40862e42fefa39ef` | `40862e42fefa39f5` | `40862e42fefa39f6` |
| Z, increasing numeric order | `c0874910d52d3059` | `c0874910d52d3058` | `c0874910d52d3052` | `c0874910d52d304c` | `c0874910d52d304b` |

The adapter calls the actual `guard` and asserts Rate/NumericalBoundary,
index 17, phase/quantity `discount factor` and the exact original logarithm
bits. A companion native-expression check compares the allowance, outward
radius and lower-distance bits to the independent exact reference. Pure
helper inputs such as signed MAX need not be realizable quasi exponents;
they are not public financial states. H and Z rejection bands are disjoint
here, and the error has no boundary-name field: these checks do not claim
that swapping two observationally equivalent loop branches is detectable.
This is arithmetic/actual-helper coverage, not a universal libm theorem.

## Independent review of the 21 hand controls

After the oracle had been frozen, the producer separately reviewed
[the original hand suite](../validation/hand/bonds_gilt_numeric_hand.rs) and
[its active copy](../tests/bonds_gilt_numeric_hand.rs). Their only difference
is the import/helper and five assertion replacements needed to inspect
Interval through its public bounds. Inputs and mathematical expectations
remain unchanged. Original SHA-256 is
`c833c9f2c974c09ca2474d29cd7433d7b678ecb3d10a8b3a69444f5e2d9ad66e`;
reviewed active SHA-256 is
`ded73145de7af135892eeb0913dfa8b152ac0baff53c214921dea8e10968d5f7`.
No precondition defect was found in any of the 21 controls.

Independent exact/Fraction/date checks and Decimal160/240 corroboration were
actually executed and saved at
`/private/tmp/sabiquant-gilt-hand-preconditions-independent-v2.json`, SHA-256
`4e48eb02460dbbdbc36dcea2bb4fa8207fbe5f191d74cedeeca884f1005baaee`.
The scratch checker SHA-256 is
`102dfd0cd45f2f12d15c7b798b884c53caec80fc35b7b91e34a93d8ac3cbce2d`.
Printed transcendental margins agree at 40 significant digits; exact dates,
identities and rational inequalities are checked without decimal tolerance.
This is a mathematical review, not an additional public Rust execution or a
formal elementary enclosure.

* The half period is 91/182, with exact A=102.5. Its six values and the
  two-payment A=(3,103), lambda=(1/2,3/2) anchors follow exactly; adjacent zero
  yields preserve the selected identity while retaining ideal sensitivity.
* At next_up(-2), q=2^-53. With C=0, the first overflowing discount is at
  lambda20 (index20), before redemption21. With full C, coupon PV16 is below
  range while PV17 exceeds it. With C=1e-38 and y=MAX, coupon PV1 underflows
  before the second discount can underflow.
* The 2,162-period q=.75 control has largest PV log2 approximately1022.3111
  and positive sum log2 approximately1024.3111. Exact rational inequalities
  confirm the wide margins; individual admission cannot be substituted for
  dirty reduction.
* For C=2^126, lambda1/182 and y=2^540, normalized convexity log2 is
  approximately-1087.4999 while raw d2 log2 is approximately-965.4614. Thus
  a positive finite raw numerator and positive-zero normalized failure coexist.
  At half-period C=0/y=MAX, raw signed derivatives underflow while modified
  duration is a positive subnormal and Macaulay remains1/4.
* Nineteen near-floor periods with C=0 have finite dirty and moment but raw
  derivative overflows. With C=2^15, dirty log2 is approximately1021.0088 and
  moment log2 approximately1024.2567, establishing genuine moment overflow.
  The retained positive weighted time remains9.5 within the stated allowance.
* The target/lower-zero/callback-cap controls follow exact positive prices,
  the single AI shift and lower-first counted invocation rules. The three
  independent inverse anchors are y=0,6,-1.5. Across their complete brackets,
  P<230, slope magnitude>2.6843 and a conditional uniform price allowance
  below2.551e-13<2^-40 justify the stated inverse bound; solver width and the
  cached amount residual are included separately.
* A six-month anchored cycle spans at most floor(119987/6)=19997 complete
  periods inside years1..9999. The EOM case starts0001-06-30 and endsDateMAX,
  has19998 Q dates and19997 retained bundles. At y=0 its final lambda19997
  gives exact d1=-999850, d2=9997500150, moment999850, both durations9998.5,
  and convexity99975001.5. A further period requires year zero. Count/cap and
  original-index assertions therefore exercise actual admissible boundaries.

## Reproduction and limitations

```sh
env PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH \
  python3 -I validation/generator/gilt_numeric_kernel_check.py \
  --output /private/tmp/sabiquant-gilt-kernel-NEW
```

The output directory must be new and outside the repository. Every invocation
pins all three frozen inputs, snapshots current sources, verifies the copy,
generates the additive temporary reference, runs all four supplemental test
executions and Clippy, and checks for source/runner/input changes afterward.
A failed command or source drift produces failed evidence, not a final-tree
claim. The runner clears inherited coverage/wrapper/target flags and uses a
separate target directory with matching rustc/Cargo.

This closes the production-linked private primitive-consumer obligation for
the recorded snapshot. It does not replace public source/cash/ideal/inverse
consumers, failure-order tests, full regression, production coverage, actual
package extraction, cross-target runtime evidence, formal elementary
certification or the remaining financial/owner/release/full-M8 gates.
