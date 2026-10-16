# Independent additive bond core numerical audit

This audit adds executable numerical evidence for the frozen local
[bond core contract](M8_BOND_CORE_CONTRACT.md). It preserves the existing
generator, fixture, contract and Rust production bytes. It is local engineering
evidence, not financial certification, named-market acceptance or a universal
floating-point precision guarantee.

Run the complete audit with:

```sh
python3 -I validation/generator/bond_core_decimal_audit.py --check
```

The [new generator](../validation/generator/bond_core_decimal_audit.py) uses
Python standard-library integers, `Fraction`, civil dates and `Decimal`.
It never imports SabiQuant or QuantLib and was written without inspecting
Rust bond/quote production or the previous generator's calculation bodies.
Initial schema/header inspection also exposed the first basic invalid-face,
zero and subnormal quote expectations; those values were not calculation
inputs. The independent model was drafted before inspecting the remaining
frozen expectations. Each comparison receives only input cells until its
independent answer has been constructed.

The complete source fixture is pinned before execution. Sequential IDs,
18,128 distinct input tuples, profile/operation/status censuses and the source
body digest are recomputed. The source generator is read only to hash its
bytes. The additive [audit fixture](../validation/fixtures/bonds/bond-core-decimal-audit.fixture)
contains all 1,496 source plan rows and one supplementary cancellation case;
quote rows are recomputed in full and summarized with an answer digest.
It is consumed by the Python audit, not by a new Rust fixture test.

## Actual quote values, including ties and range statuses

Finite binary64 operands are decoded to signed integer significands and powers
of two without binary floating arithmetic. `Fraction` constructs the exact
`100 * amount / face` or `face * price / 100` expression. An integer
quotient/remainder nearest-even decision produces the result bits, including
subnormal, binade-carry and overflow transitions. Nonfinite/invalid inputs
retain validation order and error context; nonzero rounded zero remains an
underflow error under the local contract.

A second calculation divides **scaled Decimal operand values**, rounds the
actual quotient to its nearest-even integer, and encodes its binary target
grid. It compares full result bits and range status at **256 and 512 digits**;
it does not merely compare signs. Scaling before Decimal rounding avoids
converting an approximate decimal midpoint directly to binary64.

For quotients near a rounding decision, the exact reduced scaled numerator and
denominator are bounded by 108 bits under the contract; the actual maximum
on these source rows is **106 bits**. An exact half-integer has denominator 2
and is exactly representable at these Decimal precisions. Off a half-integer,
the distance is at least `1/(2*denominator)`, while the scaled magnitude is
below `2^53`; 256-digit quotient error is much smaller than this separation.
Values below one quarter of a minimum-subnormal grid unit are safely below
the half-unit decision. This explains precision adequacy for these rational
quote checks independently of observing agreement at two precisions.

| Recomputed quote evidence | Count |
|---|---:|
| Source quote rows | 16,656 |
| Finite, valid-face input expressions checked at both precisions | 16,524 |
| Actual Decimal rounded-value/status comparisons | 33,048 |
| Successful finite outputs, including exact zero | 15,824 |
| Overflow errors / nonzero zero-underflow errors | 360 / 340 |
| Input-validation errors | 132 |
| Exact half-ULP midpoint rows | 1,102 |
| Even / odd lower-integer midpoint parity | 40 / 1,062 |
| Successful nonzero subnormal outputs | 1,417 |
| Unclassified frozen mismatches | 0 |

Twenty hand-anchor evaluations (19 distinct rational values, with zero
evaluated under both signs) additionally check normal ties of both parities,
binade carry, half-minimum-subnormal ties, the maximum finite value and the
exact overflow midpoint. Three per-100 values at both precisions for each of
1,187 successful frozen valuations are also recomputed from their supplied
rounded amount values: **7,122 additional Decimal comparisons**.

## Independent direct-horizon factors and staged amounts

The input model independently implements the six selected conventions:
Actual/365 Fixed, Actual/360, Bond Basis, 30E/360, explicit EOM-qualified
30/360 US, and Business/252 with weekends only. Integer day counts are divided
once and rounded to binary64 before becoming coupon or discount horizons.
Coupon and partial-accrual amounts round the exact supplied three-factor
binary rational once. Ex-coupon subtraction rounds each contribution once;
the accrued reduction rounds the exact sum of those supplied contributions
once. This retains the contract's full-coupon/partial-coupon staging.

Here, an **ideal price** holds those staged binary64 coupon amounts and
year-fraction values fixed and evaluates the discount formula at high
precision. It is not a financial ideal using unrounded coupons or exact raw
day-count rationals. This audit does not propagate the earlier coupon or
year-fraction rounding error into a bound against that different model.

Settlement eligibility, ex-date equality, direct payment-date horizons and
the shared work ledger are derived from input dates. The audit does not
generate business-day ex-dates or infer a settlement calendar. The frozen
fixed-rate profiles retain two coupons; the supplementary discriminator has
one. Coupon payment adjustments are explicit supplied dates.

For supplied rounded amount `A`, rounded horizon `t` and exact binary64 yield
`r`, the independent ideal discount functions are:

```text
Simple:      D = 1 / (1 + r*t)
Continuous:  D = exp(-r*t)
Compounded:  D = exp(-m*t*ln(1 + r/m))
```

Both hybrid methods select their fixed-time branch using the rounded binary64
`1/m`, as the contract requires. Ideal factors, ideal net prices and bounds
are evaluated independently at **160 and 240 decimal digits**. The resulting
ideal and staged binary64 price bits agree at both precisions. Decimal prices,
gross exposure and bounds must also agree within `1e-145 * max(gross, 1)`;
the gross scale avoids a misleading relative comparison after cancellation.
This is stable high-precision evidence on this grid, not an interval proof of
correct rounding for all elementary inputs.

The staged comparison separately rounds the compound factor, its reciprocal,
each flow's product, and the final exact reduction. Simple compound factors
are rounded from exact `1+r*t`, matching the contract's fused operation.
For elementary branches the staged oracle uses the independently computed
ideal compound factor; its bit agreement with the frozen values is an observed
result on these rows, not a guarantee that every platform libm follows that
same ideal rounding pipeline.

| Recomputed plan evidence | Count |
|---|---:|
| Frozen plan inputs | 1,496 |
| Successful frozen valuations | 1,187 |
| Exact work-limit errors / pre-issue errors | 213 / 96 |
| Supplementary successful cancellation case | 1 |
| Successful 160/240-digit price agreements | 1,188 |
| Active Simple / Compounded / Continuous flow factors | 623 / 1,491 / 158 |
| Frozen dirty / clean values matching independent staged bits | 1,187 / 1,187 |

Raw ideal-versus-frozen ULP distances are retained rather than used to enlarge
a blanket tolerance:

| Distance | Dirty rows | Clean rows |
|---|---:|---:|
| 0 ULP | 754 | 745 |
| 1 ULP | 416 | 417 |
| 2 ULP | 17 | 25 |

All entitlement codes, redemption eligibility, flow counts, accrued amount
bits, work counts and structured error fields match independently. There are
no unexplained mismatches.

## Gross-conditioned allowance and production linkage

Let `u=2^-53`, `eta=2^-1074`, `D_i` be an ideal discount and `A_i` the supplied
rounded flow amount. The measured M4 elementary acceptance criterion supplies
`b_i = 8*u*(1+abs(ln(C_i)))*abs(D_i) + eta` for the discount error. This is the
existing empirical criterion, not a proven worst-case bound for Rust libm.
For Simple, the audit instead propagates the exact fused-factor discrepancy:

```text
C_exact = 1 + r*t; C_stored = RN(C_exact)
b_i = abs(1/C_stored - 1/C_exact) + u*abs(1/C_stored) + eta/2
p_i = abs(A_i)*b_i + u*abs(A_i)*(abs(D_i)+b_i) + eta/2
```

Exact identity/zero products have zero factor/product uncertainty. With
`S=sum(A_i*D_i)`, and `AI` the independently computed staged accrued amount:

```text
B_dirty = sum(p_i) + u*(abs(S)+sum(p_i)) + eta/2
B_clean = B_dirty + u*(abs(S-AI)+B_dirty) + eta/2
```

Each frozen price must satisfy its own absolute bound. Maximum observed
`abs(error)/B` is **0.594990082805109669477** for dirty
(`BOND-018147`) and **0.518703739944551928851** for clean (`BOND-018084`).
The reduction can be highly cancelled while `sum(abs(A_i*D_i))` remains large;
neither small net-relative error nor a fixed ideal-price ULP distance follows.
Serialized bounds include a `1e-24` relative enlargement to avoid understating
a bound when printing 26 significant decimal digits.

The existing [Rust consumer](../tests/bonds_core_oracles.rs) links production
to the frozen fixture with exact amount/entitlement/error checks and at most
two same-sign integer-bit steps for dirty/clean prices. Consequently the
transitive ideal-price allowance is **the conditioned frozen-price bound plus
the exact radius admitted by those two bit steps**. The additive fixture
records this as `linked_dirty_bound` and `linked_clean_bound`, including
asymmetric spacing across binade boundaries. The Python audit does not claim
to have directly sampled current Rust price outputs. Native Rust test evidence
and the separate public cancellation regression are recorded by the main
[validation report](M8_BOND_CORE_VALIDATION.md).

The supplemental `AUDIT-CANCELLATION-001` uses a one-year coupon of `-100`,
redemption `next_up(100)`, settlement at issue and Simple yield `0.25`.
Rounded PVs are `-80` and `next_up(80)`, so the staged dirty value is `2^-46`
(`0x3d10000000000000`), while the ideal net price is `0.8 * 2^-46`
(`0x3d0999999999999a` after rounding). The staged result has **25%** relative
error against that small ideal net and still lies within its gross-conditioned
absolute allowance. This is an intentional staged-product effect, not a defect
or a reason to reinterpret the frozen two-ULP comparison as an ideal guarantee.

## Reproduction and immutable pins

The initial generation and subsequent complete `--check` run both succeeded.
All 18,152 original rows are audited; the new fixture has **1,497 plan rows**.
The required schema-1 `tolerance` header states the exact and conditioned
comparison scopes; adding it left the complete record body byte-identical.
No original generator, fixture or contract was modified.

| Artifact | SHA-256 |
|---|---|
| Frozen source fixture | `5a5dc4f4ec573d189b8b7f87aeaf335b5a253f1ad983633612dfbe2ae8ae7817` |
| Frozen source generator | `afe27c0ce1a362492a98202c2882271cabbc3b1d510bb49dd3d5215460055b18` |
| Frozen bond core contract | `009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c` |
| Additive audit generator | `7cae8cfc0583003c10c62df45750ba8a6e0d1311e0aaaaf3d4a5b901a41e3f56` |
| Additive audit fixture (780,362 bytes) | `c888d7b94be5ad367d7bc12ccab726fa5ca74939bb0b40ad9b726dd1ca58ad19` |
| Additive body, including column row | `de97d448254e67b8501b78fd90869e8a4538dac5ab9054fc4d0ee436c99f4542` |
| Independently rebuilt source quote answers | `2eeb7e4547aa74d810ac99f4388b3a3d7e022709e0074ba0243fff08409f7f26` |

The fixture header carries the full deterministic census, classifications and
maximum-score row IDs. `--check` recomputes every answer, bound and pin before
requiring byte identity. No test for unavailable financial adapters, yield
inversion, duration/convexity, calendar-generated ex-dates or named market
pricing is implied by this audit.
