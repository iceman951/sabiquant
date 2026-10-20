# Treasury outward-power design check

Reviewed on 2026-10-20, Asia/Bangkok, against the local working tree on
`feat/m8-us-treasury` at `fc3f629`. This is **research for root's Treasury
numerical design**. It is not an adopted numerical contract, API, requirement or
fixture, and nothing here is adopted. Root will challenge the bounds
independently before any contract.

The check is
[`validation/generator/treasury_power_design_check.py`](../validation/generator/treasury_power_design_check.py).

## Authorship and separation

Claude Code (peer `sabiquant-d5`) wrote the initial script and report. Codex
independently audited that version, proved the coupled exponent premise, and
corrected the helper diagnostics and process gates in v2. The pre-fix audit
and v1 evidence remain preserved. V3 makes the final artifact strict JSON;
all v2 arithmetic and numerical observations are unchanged. Inputs include root's Treasury numeric
design notes and numerical design reviews v1–v3, pinned below.

- **Standard library only.** The script uses only Python's standard library,
  and no Rust production code was written.
- **Nothing else edited.** This assignment changed only the two new research
  files. No pre-existing production, fixture, generator or contract was
  changed, and nothing was committed.
- **No QuantLib** was read.
- **One production read.** The script models the shift steps of gilt's private
  `extended_ratio` (`src/bonds/gilt/numeric.rs:374`) only to check the
  capacity of its temporaries. No expected value enters any production path.

**Inputs.**

- **Source cases:** Section II examples A–G of 31 CFR 356 Appendix B,
  transcribed as exact rationals. Each case's definition lines and printed
  prices are verified as anchor strings in the pinned eCFR XML snapshot.
- **Edge yields:** represented binary64 `y` values.
- **Day counts:** `(r, s)` pairs within the source range, with `181 ≤ s ≤ 184`
  and `1 ≤ r ≤ s`.
- **Amounts:** source cells use the source amounts `A_j`. Edge cells are factor
  cells (`A = 1`), not financial reports.

## Candidate kernel modeled

`V_j = A_j / (q·b^j)`, with `b = 1 + y/2` and `q = (2s + r·y)/(2s)`. Both come
from one exact `y` and one `(r, s)`. `b` and `q` are converted from their exact
rationals, never from a rounded `y/2` or `alpha`.

Each value is a p-bit significand (`p ∈ {192, 256}`) with an unbounded integer
exponent. Every operation rounds the low endpoint down and the high endpoint
up. Powers use left-to-right binary exponentiation. Three orders are compared:

| Variant | Order |
|---|---|
| `forward` (preferred) | `b̂ = conv(b)`, `P = b̂^j`, `Q = q̂·P` (skipped at `j=0`), `V = Â/Q` |
| `reciprocal_exact_seed` | `conv(1/b)^j · conv(1/q) · Â` |
| `reciprocal_rounded_seed` | `(1 ⊘ conv(b))^j · conv(1/q) · Â`; the seed's own rounding is raised to weight `j` |

## Certified width

**Weights.** Each enclosure carries certified weights `(c, o)`:

- `c` counts inexact conversions (0 when a conversion is exact);
- `o` counts outward operations;
- a product or quotient of two enclosures has weights `(c1+c2, o1+o2+1)`.

Unfolding any multiplication chain for `b^j` gives a tree with `j` leaves and
`j−1` products, so the chain carries `o = j−1` and `c = j·c_b`.

**Bound.** With `u = 2^(1−p)`, `γ = (1+u)/(1−u)` and `N = c + 2o`:

```text
hi/lo ≤ (1+u)^c · γ^o ≤ (1−u)^(−N) ≤ 1/(1 − N·u)      (N·u < 1)
```

This uses `1+u ≤ 1/(1−u)`, `γ ≤ (1−u)^(−2)` and Bernoulli's
`(1−u)^N ≥ 1 − N·u`. The last form is checked exactly with p-bit integers. No
large power is materialized.

**Closed-form weights, asserted for every cell.** `[A]` is 1 for a PV cell and
0 for a factor cell.

| Variant | `c` | `o`, `j ≥ 1` | `o`, `j = 0` | `N` bound, PV cell |
|---|---|---|---|---|
| `forward` | `j·c_b + c_q + c_A` | `j + 1` | 1 | `≤ 3j + 4` |
| `reciprocal_exact_seed` | `j·c_(1/b) + c_(1/q) + c_A` | `j + [A]` | `[A]` | `≤ 3j + 4` |
| `reciprocal_rounded_seed` | `j·c_b + c_(1/q) + c_A` | `2j + [A]` | `[A]` | `≤ 5j + 4` |

At the cap `j = 19,999` the forward budget `N ≤ 60,001` certifies a relative
width of at most `2^-175.13` at p=192 and `2^-239.13` at p=256.

**Why forward is preferred.** `b = 1 + y/2` is often exactly a p-bit value,
because `2 + y` has at most p significant bits. Then `c_b = 0`, and only the
operations count. `1/b` is usually not dyadic, so the reciprocal-exact seed
pays a weight-`j` conversion. The rounded seed pays a weight-`j` operation as
well. Observed widths at `j = 19,999`, `r/s = 92/184`, factor cells:

| y | p | Variant | c | o | N | Observed log2 | Certified log2 |
|---|---:|---|---:|---:|---:|---:|---:|
| `MIN_SUB` | 192 | forward | 20,000 | 20,000 | 60,000 | −175.712 | −175.127 |
| `MIN_SUB` | 192 | reciprocal exact seed | 20,000 | 19,999 | 59,998 | −177.712 | −175.127 |
| `MIN_SUB` | 192 | reciprocal rounded seed | 20,000 | 39,998 | 99,996 | −176.712 | −174.390 |
| `RN(.0884)` | 192 | forward | 0 | 20,000 | 40,000 | −177.863 | −175.712 |
| `RN(.0884)` | 192 | reciprocal exact seed | 20,000 | 19,999 | 59,998 | −176.667 | −175.127 |
| `RN(.0884)` | 192 | reciprocal rounded seed | 1 | 39,998 | 79,997 | −176.667 | −174.712 |
| `−1.9` | 192 | forward | 0 | 20,000 | 40,000 | −178.383 | −175.712 |
| `1e300` | 192 | forward | 20,000 | 20,000 | 60,000 | −176.232 | −175.127 |
| `1e300` | 256 | forward | 20,000 | 20,000 | 60,000 | −240.446 | −239.127 |

The evidence file holds all 24 rows: four yields, two precisions and three
variants.

## Containment: proof and cross-checks

**Proof by induction, every cell.** Each conversion, outward product and
outward quotient is checked exactly with integers to satisfy its inequality:

- conversion: `lo ≤ x ≤ hi`;
- product: `lo ≤ a.lo·b.lo` and `a.hi·b.hi ≤ hi`;
- quotient: `lo·b.hi ≤ a.lo` and `a.hi ≤ hi·b.lo`.

All endpoints are positive, and products and quotients of positive intervals
are monotone. So every final enclosure contains its exact value. The v2 run
checked 24,708 conversions, 43,680 products and 4,040 quotients, with 0
failures.

**End-to-end exact comparison (independent check).** Wherever
`j·bits(b) ≤ 2^22`, each enclosure is also compared with the exact integer
ratio `A·q_den·b_den^j / (A_den·q_num·b_num^j)`. Huge values are never
normalized. This covered 1,944 source cells and 3,024 edge cells, with 0
failures.

**Directed Decimal cross-check, not proof.** The 96 edge cells at
`j = 19,999` with `y ∈ {MIN_SUB, MIN_NORMAL, 1e300, MAX}` exceed the exact
budget. They get floor/ceiling `Decimal` enclosures at 400 and 600 digits.
In all 96 cells the Decimal interval lies strictly inside the binary one, with
0 overlap-only and 0 disjoint cases. This corroborates the induction; it is
not part of the proof.

## Counterexample: `y = MIN_SUB`, `j = 19,999`

Here `b = 1 + 2^-1075` converts to `[1, 1 + 2^(1−p)]`. The script checks
exactly that the power enclosure has `(hi−lo)/lo ≥ j·2^(1−p)`.

| p | Helper `b^j` width, log2 | Exact-products width `(1+2^(1−p))^j − 1`, log2 | Certified upper, log2 | Exact `1/b` seed width, log2 |
|---:|---:|---:|---:|---:|
| 192 | −175.712 | −176.712 | −175.127 | −177.712 |
| 256 | −239.712 | −240.712 | −239.127 | −241.712 |

- **Against v1's claim:** at p=192 the helper width is **624.95 × 2^-185**.
  The exact-products column shows that the linear term exists even with no
  intermediate rounding.
- **Reciprocal seed:** the exact `1/b` seed converts to `[1 − 2^-p, 1]` and
  keeps width `≥ j·2^-p`.

## Binary64 round-to-nearest-even decisions (honest counts)

A cell is **decided** when both endpoints round to the same binary64 value.

A cell is **near** when the exact or reference value lies within the certified
relative width `N·u/(1 − N·u)` of a binary64 rounding boundary. The **margin**
is the log2 distance to the nearest midpoint minus the log2 certified width.

| Set | Cells | Decided | Unresolved | Near | Exact ties | Exact point enclosures | Least margin |
|---|---:|---:|---:|---:|---:|---:|---|
| Source (7 cases × decimal and binary64 `y` × every bundle × 2 p × 3 variants) | 1,944 | 1,944 (all normal) | 0 | 0 | 0 | 0 | 118.6 bits (A, binary64 `y`, `j=41`, p=192, rounded seed) |
| Edge (13 `y` × 4 `(r,s)` × 10 `j` × 2 p × 3 variants) | 3,120 | 3,120: normal 2,340, zero 528, overflow 240, subnormal 12 | 0 | 0 | 0 | 240 | 83.4 bits (`y=MAX`, `r/s=92/184`, `j=0`, forward: distance `2^-106`, width `2^-189.4`) |

The 240 exact point enclosures arise where `b` and `q` are powers of two:
`r = s` and `y ∈ {next_up(−2), −1.5, −1, 2}`. 180 of them are binary64 values.
The other 60 overflow, except `y = 2` at `j = 19,999`, which underflows to
zero.

**Interpretation:**

- **Zero is an observation, not a guarantee.** Zero unresolved cells on this
  selected set promises nothing for other inputs.
- **Unresolved cells are possible whenever** an exact value lies within the
  certified width of a binary64 midpoint.
- **Exact midpoints:** an enclosure **strictly straddling** a midpoint is
  unresolved. Merely touching a midpoint can resolve: the nondegenerate
  interval `[1, 1+2^-53]` has both endpoints RNE=1, including its tie-even upper
  endpoint. An **exact point** enclosure resolves by the tie rule. The earlier
  wording that every nondegenerate enclosure containing a midpoint is
  unresolved was too absolute.
- **Where ties can be detected:** exact ties are detectable only on cells with
  an exact reference. The 96 Decimal-only cells were checked for proximity
  only.
- **No promise of unique rounding.** The design must define the unresolved
  outcome; this check promises no uniform, unique rounding.

**Source dirty sums.** For each source case, at both precisions and for both
the decimal and binary64 `y`, the outward sum of forward PV enclosures:

- contains the exact dirty price;
- decides the six-place display, which matches the printed dirty price for both
  yield forms, with no ties;
- is between `2^-189.6` and `2^-184.9` wide at p=192.

R3 `RN6(D) − RN6(A)` matches every printed P. All the source comparisons,
containment and RN6-decision observations are PASS gates for both yield forms.
Sum weights are not part of the certified bound above.

## Coupled five-word widths (inherited from review v2)

**Inputs.** 20,211 `y` values (seed 20261020: random ranges, the first 199
values above −2, and binade edges) × 30 `(r, s)` pairs, with `j = 19,999` and
PV significand `2^53 − 1`. `δ̂ = RN(2+y)` and `q̂ = RN((2s + r·y)/(2s))` always
come from the same `y`, `r` and `s`.

| Quantity | Observed | Limit |
|---|---:|---:|
| `e_δ − e_q` | [−51, 9] | [−51, 9] proved; [−52, 9] conservative width budget |
| Unaligned second-derivative terms T1/T2/T3 | 151/137/122 bits | below `2^152`/`2^138`/`2^122`, analytic |
| Second-derivative aligned numerator | 306 bits | ≤311 bits analytic, 320 helper |
| Second-derivative denominator `2s²δ̂²q̂²` | 229 bits | 267 helper |
| Peak `extended_ratio` temporary, second derivative | 306 bits | 320 |
| First derivative numerator / denominator / peak | 180 / 115 / 180 bits | 320 |
| Moment peak | 75 bits | 320 |

The worst input is `y = next_up(−2)`, `r = 1`, `s = 181`.

**Analytic exponent premise (independent of the search).** Let
`α = r/s ∈ [1/184, 1]`, `δ = 2+y` and `q = 1+αy/2`. Since represented finite
`y > −2`, the nearest allowed endpoint gives `δ ≥ 2^-52`. Also `q ≥ 2^-53`:
for negative y, `q ≥ δ/2`; for nonnegative y, `q ≥ 1`. Both observations are
normal and finite. In particular, `MAX+2 < MAX+2^970`, the overflow midpoint,
and `q ≤ 1+MAX/2 < MAX`.

For `−2 < y < 0`, `q ≥ δ/2` and RNE monotonicity plus exact normal scaling
give `q̂ ≥ δ̂/2`, so the binade difference is at most 1. For `−1 ≤ y < 0`,
`δ̂ ≥ 1` and `q̂ ≤ 1`, so it is nonnegative. For `y < −1`,
`q < 1−1/368 < 1−2^-54`, the midpoint immediately below 1; therefore `q̂ < 1`.
Combined with `δ̂ ≥ 2^-52`, this gives the lower bound −51.

For `y ≥ 0`, `2 ≤ δ/q ≤ 2/α ≤ 368`. Binary64 normal RNE has relative error
at most `v = 2^-53`, so
`1 < 2(1−v)/(1+v) ≤ δ̂/q̂ ≤ 368(1+v)/(1−v) < 512`. The strict upper inequality
is the exact integer check `368·(2^53+1) < 512·(2^53−1)`. A binade difference
of at least 10 would imply a ratio strictly greater than 512, so the upper
bound is 9. The normalized 53-bit decoder exponent difference is the same
as this binade difference. Thus **[−51, 9] is a universal analytic bound on
the stated domain**; it is not inferred from the random search.

The script checks the exact constants used in this proof separately from
sampling. An additional independent confirmation covered all 730 `(r,s)`
pairs and 275 biased represented yields, 200,750 cells with observed [−51, 9].

**Analytic capacity chain, using the weaker [−52, 9] width budget.**

- **Terms:** `T1 < 2^152`, `T2 < 2^138` and `T3 < 2^122`.
- **Alignment:** onto `2·min(e_δ, e_q)` adds at most 104 bits. With negative
  exponent difference the three shifts are at most 104/52/0; with positive
  difference they are at most 0/9/18. The former gives the larger bound.
- **Sum and product:** the sum of three is below `2^258`, and multiplying by
  PV gives below `2^311`.
- **Denominator:** below `2^229`.

`extended_ratio` admits a nonzero numerator of up to 320 bits and a nonzero
denominator of up to 267 bits. Its initial alignment reaches at most the
larger input width; its numerator normalization shift reaches at most
`denominator_bits+53 ≤ 320`; its denominator shift reaches at most
`numerator_bits−52 ≤ 268`. Subtraction and quotient rounding add no wider
integer. Thus integer capacity failures are excluded for the stated modeled
operands, and the helper needs no change. This is a caller algebra and
capacity proof; no Treasury production caller exists yet.

## Clean-recipe tie rules (qualifying review v3)

Let `R2 = RN6(D − a)` and `R3 = RN6(D) − a`, with `a = RN6(A)` on the `10^-6`
grid. For each rule and set, the table counts the cases where
`R(x − a) ≠ R(x) − a`:

| Rule | Non-tie, any sign (20,000) | Ties, `x − a ≥ 0` (10,000) | Ties, `x − a < 0 < x` (10,000) |
|---|---:|---:|---:|
| Ties toward +∞, `floor(x + 1/2)` | 0 | 0 | 0 |
| Ties away from zero (`AmountRounding::NearestTiesAwayFromZero`, `src/amounts.rs:89`) | 0 | 0 | 10,000 |
| Ties to even | 0 | 4,988 | 4,972 |

- **Ties toward +∞:** "half-up translation" holds globally only for this rule.
- **Ties away from zero:** R2 = R3 is guaranteed when the clean value `D − a`
  is nonnegative (D is always positive). The clean value can be negative in
  the extended domain, for example at very high yields with a large AI, and
  every tie that crosses zero then breaks the equality. Non-ties still agree
  throughout the signed domain.
- **Ties to even:** R2 and R3 differ at ties whenever `a` has an odd last
  digit.
- **Non-tie values:** all three rules agree.

## Proof scope

**Proved for every cell in this run:**

- containment, by per-operation induction;
- the certified bound `hi/lo ≤ 1/(1 − N·u)`;
- the closed-form weights `(c, o)`;
- the `MIN_SUB` lower bound.

**Proved analytically on the stated domain:** the coupled exponent range
[−51, 9] and the modeled derivative integer capacity bounds. Exact constants
supporting the exponent proof are checked separately from the sample.

**Shown on the selected inputs only:** RNE decisions, proximity, and the
observed maxima of the coupled widths.

**Not established:**

- how often unresolved or near cells occur across the whole domain;
- a certified bound for sums;
- performance;
- any production design or API;
- results on other hosts or Python versions.

The Decimal cross-check is corroboration only.

## Independent audit and exact controls

The historical v1 script was reviewed without edits first. Its retained audit
found four helper and gate defects, plus overbroad midpoint wording:

- the distance diagnostic considered only the next midpoint, missing the
  closer midpoint below a binade boundary;
- hypothetical midpoints above the overflow region were tagged as exact
  binary64 ties;
- the source PASS gate omitted binary64 display comparisons and RN6 decisions;
- a failed gate printed FAIL but the process returned success.

All are corrected in v2. The selected source/edge observed counts and least
margins in the tables are unchanged; the old claims about these selected
cells remain observations. The exact witness `x=1+2^-60` now gives nearest
relative distance `log2=-53.97763218697155`, correcting the prior
`-53.01131531322783`. Ties above the actual overflow threshold are no longer
counted as binary64 boundary ties.

The checker now gates 24 exact mathematical self-controls: 11 RNE controls
including zero, subnormal, normal and overflow ties; 9 nearest-boundary
controls including binade spacing changes and range boundaries; and 4
midpoint interval controls distinguishing strict straddles, touching, and
exact points. These are helper probes, never fabricated financial states.

Separate retained post-fix controls independently compare 7,166 exact
rationals with CPython Fraction-to-float and adjacent-float midpoint geometry,
with no value/tie/representability disagreements and a largest log-distance
difference of `2.185e-13` from log display arithmetic. They also verify 109
source metadata guard rejections. A forced failed research gate now produces
FAIL and process exit 1; overwriting an existing report and a bad source pin
both fail with exit 1, preserving existing evidence. These deliberately
injected guard failures are separate from the passing numerical run.

Root's reproduction also found bare `-Infinity` log-distance diagnostics in
v2. Python accepts these as an extension; standard JSON parsers reject them.
V3 encodes infinite diagnostics as explicit strings, rejects NaN, serializes
with `allow_nan=False`, and parses both serialized and written text with a
rejecting `parse_constant` hook. Ten additional format controls check this
boundary. Root compared every numerical observation with v2 after only this
diagnostic encoding, then repeated the 7,166 neighbor-boundary and 109 guard
controls successfully. Passing and deliberately failed reports/stdout all
parse strictly. Historical v1/v2 files remain unchanged and retain their
nonstandard-JSON qualification.

## Decisions left to root (not adopted)

1. **Kernel choice:** outward powers or an admitted-libm profile.
2. **Precision:** p=192 or p=256, sized from `N ≤ 3j + 4` at the chosen `j`
   cap.
3. **Order:** `forward` is recommended. A reciprocal seed needs its own bound;
   the rounded seed's is `N ≤ 5j + 4`.
4. **Observation policy:** a typed unresolved outcome, a deterministic
   precision ladder, or a specified bounded approximation such as root's
   [lower-endpoint proposal](M8_US_TREASURY_LOCAL_DESIGN.md), which does not
   promise ideal correct rounding; preserve exact-point handling.
5. **Domain:** the `j` cap and exponent type, and the ordering and bound for
   PV sums.
6. **Clean recipe:** R1, R2 or R3 with its tie rule, including the signed-domain
   qualification above (review v3 decisions 1–2).
7. **Helper integration:** carry the modeled algebra/capacity proof into the
   adopted Treasury caller, including exponent arithmetic and its actual
   observation-domain and error-order choices.

## Reproduction and pins

```sh
python3 -I validation/generator/treasury_power_design_check.py \
  --source-xml /private/tmp/sabiquant-treasury-source-refresh-v1/part356-2026-10-16.xml \
  --output /private/tmp/sabiquant-treasury-power-design-reproduction.json
```

**Run environment:** CPython 3.9.6, macOS-26.6.2-arm64, from
2026-10-20T06:40:26.862944Z to 06:40:32.631043Z, about 6 seconds, exit 0, `PASS` with every
gate true. The output file is opened create-new, so an existing file is never
overwritten. Choose a new output path when reproducing. A failed PASS gate
returns exit 1 after writing its failure evidence.

| Artifact | SHA-256 |
|---|---|
| `validation/generator/treasury_power_design_check.py` (current v3) | `39ef382e51a0517de090796681ae32d31ad77a205b21d64045519b34269b5d6c` |
| Root v3 `/private/tmp/sabiquant-treasury-root-review-v1/power-design-v3.json` | `2fcfbb8332f331d30f80ff14a567cec5ac61067182e8f4b80afa9b0f2afc81bd` |
| Root v3 encoding comparison `/private/tmp/sabiquant-treasury-root-review-v1/power-v3-encoding-verification.json` | `4b425679c2bb437c1cb276117bfc4e90a0c2beb0b04a9354f4e81f65ec1e2d6b` |
| Root v3 controls `/private/tmp/sabiquant-treasury-root-review-v1/independent-controls.json` | `9a7cd73154cf553d6d6318f3793efeeed9351f06fbed57037100fd9a7653298e` |
| Root control script `/private/tmp/sabiquant-treasury-root-review-v1/power-independent-controls.py` | `90eab56a8132ac37598bfd86d069b8723be0e1ca429fdb8da7fbadd14f5185f0` |
| Historical v2 script (before strict JSON) | `7246071a4cea01863e64f701695135511d4a316fcde3dc599811d743e8b8e786` |
| `/private/tmp/sabiquant-treasury-power-design-check-v2/evidence.json` | `78fb1c544530390de6081eba8197cf88d97e6b7197459e4be212eb0c0c778563` |
| `/private/tmp/sabiquant-treasury-power-design-check-v2/stdout.txt` | `9aecf6f70ba9667e687a30c06e58d9cd5fbb011bb822b2b8a197c8add27525af` |
| `/private/tmp/sabiquant-treasury-power-design-check-v2/independent-controls.json` | `1c978cf77c80c53aa220d44e82e56bdae0c4f002501645109059c8f375d1ce06` |
| Post-fix control script `/private/tmp/sabiquant-treasury-power-design-check-v2-independent.py` | `a32d6b501b50beff1ba941197ce0950eef5d2bdbb7bb921e1b84fad547671268` |
| Historical v1 script (before fixes) | `5fdf0a9d6465ee194114d67cc89565399210c8357295f4c0bb72a809fc9c4d63` |
| `/private/tmp/sabiquant-treasury-power-design-check-v1/evidence.json` | `0a23c533217bb38bc3a64e4e40bb68e6866c4f587cc8c381a06fa264f7bf4bec` |
| `/private/tmp/sabiquant-treasury-power-design-check-v1/stdout.txt` | `f3cdbba7da4f5c1b28be9901d3d003a511912c2db93f33d612319ac4788ec020` |
| Historical pre-fix audit `/private/tmp/sabiquant-treasury-numeric-independent-audit-v1/REVIEW.md` | `bee083aeb058b2d2d30e171dc0344ce867b0b6d4510620e69e6637bc55bd6e49` |
| Historical pre-fix audit `/private/tmp/sabiquant-treasury-numeric-independent-audit-v1/evidence.json` | `00eba200fec367a1a2d89f91f8dd6ea07087bcf4c792d167d3bbeb1d0acff729` |
| eCFR Part 356 XML, 2026-10-16 | `35fa6cb3febd8807c06b08e73d1f311a472b31f127796c6ef369237915b6699d` |
| Root design notes | `7672dbafa773b194efa7557a799a86af90c16a0eef0e94519f0dca31f64a5251` |
| Numeric design review v1 / v2 / v3 | `06026dbe…` / `d72c9b67…` / `ddb27ea4…` |
