# Independent Treasury ideal price, risk and inverse references

The [generator](../validation/generator/treasury_ideal_oracles.py) derives
ideal mathematical references from the adopted [main](M8_US_TREASURY_CONTRACT.md)
and [cash](M8_US_TREASURY_CASH_CONTRACT.md) contracts before Treasury Rust.
It reads neither Rust production nor another Treasury oracle's prescribed-stage
expectations. It uses Python's standard library, exact `Fraction` arithmetic
and two separate Decimal computations. Earlier fixtures stay unchanged.
These initial expectations require root reproduction, independent review and
shared-reader attestation before freezing; generation alone is not acceptance.

The [price/risk fixture](../validation/fixtures/bonds/treasury-ideal-price-risk-oracle.fixture)
has **714 rows and 74 columns**; each row carries dirty, clean and all six
risk references. The [inverse fixture](../validation/fixtures/bonds/treasury-ideal-inverse-oracle.fixture)
has **145 rows and 51 columns**. These are reference cases, not executed
Treasury API tests. The complementary source/cash and prescribed-stage corpora
own E0 publication, factor/PV observations and observed solver behavior.

## Independent exact mathematics

Every row begins with a valid dated definition, an explicitly supplied direct
maturity lattice and EOM policy. The generator derives the first strictly
future quasi index, `r/s`, original `j`, actual payment indices, raw amounts,
raw AI and selected published AI. It retains a long first period's skipped Q1
and admits D<I, including long I>Q1. Source coupons in this fixture's
`source_amounts` mean the **raw analytical A_j** before discounting, including
exact final principal; they do not mean the selected cash-publication coupon.

The primary price computation uses exact finite geometric sums. For
`x=1/b`, form `B0=sum(A_j*x^j)`, `B1=sum(j*A_j*x^j)` and
`B2=sum(j^2*A_j*x^j)`. Binary concatenation computes the three sums by the
identities for two consecutive blocks, with exact adjustment for the first
stub and final principal. No large-lattice subtraction loses bits. With
`a=r/s`, independent product differentiation gives

```text
D = B0/q
D' = -a*B0/(2*q^2) - B1/(2*b*q)
D'' = a^2*B0/(2*q^3) + a*B1/(2*b*q^2) + (B2+B1)/(4*b^2*q)
M = (B1+a*B0)/(2*q)
Modified=-D'/D; Macaulay=M/D; Convexity=D''/D.
```

A second exact implementation separately sums individual terms using the
contract's eta/kappa expressions for every row with at most128 payments.
It compares all eight quantities by rational equality. The binary
concatenation identities are also checked against explicit sums at counts
0..29. The four maximum-lattice rows use the concatenation identity and both
Decimal payment-sum checks rather than a second 19,997-payment Fraction sum.

Independent Decimal routes at **160 and 240 digits** instead recur through
payments and sum eta/kappa/moment terms individually. Their results must
agree with exact Fraction references within
`10^(-precision+30)*max(abs(exact),1e-10000000)`. The 30 guard digits cover
17-digit cancellation near y=-2, fewer than20,000 positive terms and the
scalar operations. MIN_SUB yield effects smaller than that relative
threshold may disappear in Decimal. This threshold is corroboration, not a
proof of Decimal error or an interval certificate; the rational references
and integer remainder decisions remain authoritative.

All ideal binary64 bits are obtained by exact integer quotient/remainder,
including ties, gradual underflow and the overflow midpoint. `*_ideal` is an
80-significant-digit presentation of the ideal rational; `*_ideal_rn_bits`
is its correctly rounded IEEE value. These bits **do not prescribe the
contract's p256/lower-endpoint observation** and must not be compared E0 to
production prices or risk.

## Comparison bounds and range qualifications

The fixture separately retains prefix observations, AI cache error, absolute
dirty radius and each metric's allowance. It uses N3's universal
`E_V=(2t+t^2)*V+h`, N5's coefficient error intervals, one extended-term
rounding, final getter rounding and direct dirty-denominator conditioning.
To avoid importing observed outputs, replace each admitted PV by the valid
upper bound `(1+2t+t^2)*V+h` when bounding its contribution. Polynomial
geometric sums compute weighted coefficient sums exactly. Positive absolute
`h*g` terms use an upper sum over all actual payment positions, including
zero coupons, which is conservative. Clean subtracts **selected published
AI**, with separate cached-AI and final exact-sum rounding allowances.

For one positive PV, normalized-risk bounds use N5's exact PV cancellation;
this makes admitted subnormal dirty cases meaningful. Other normalized
bounds require positive `D_ideal-E_D`; a missing useful denominator or a
radius as large as the quantity is `uninformative`. No blanket ULP rule
replaces that classification. Each allowance additionally includes the exact
80-digit center serialization error and is serialized with upward-directed
Decimal division. Slopes in inverse rows are serialized downward. Bounds
are therefore not accidentally reduced by display rounding.

`pv_gate=guaranteed-in-range` is a sufficient hypothesis: every positive
ideal PV, multiplied by the factor-error envelope, lies strictly between
the IEEE zero and overflow midpoints. It does not predict observed PV bits.
Eight other rows say `requires-indexed-pv-gate`; their ideal references
remain available but their successful plan/risk must not be assumed.
Getter classes are **ideal per-quantity range classes**, distinct from
observed getter admission. A raw range failure never erases normalized
references. Counts across all714x8 quantities are:

| Ideal class | Count |
|---|---:|
| finite-normal | 5,604 |
| finite-subnormal | 34 |
| positive-underflow | 50 |
| negative-underflow | 18 |
| positive-overflow | 4 |
| negative-overflow | 2 |

There are 5,638 conditional absolute comparisons, 68 explicitly uninformative
comparisons and 6 ideal range references. These counts are classifications,
not production passes. Fifty-six rows have exactly one positive PV.

## Diversity and literature anchors

The 714 rows contain 208 Regular, 422 Short and 84 Long definitions. They
include payment resets versus skipped Q1, D<I, long I>Q1, Preserve/Ignore
month ends and leap February, C=0, all 39 input decimal scales, maximum i128
coupon coefficient at scale 0/38 and derived scale39 half-coupons. The
maximum date-domain lattice has 19,998 quasi dates/19,997 actual payments,
from year 1 to 9999. Exact zero and negative/positive yields, positive and
negative MIN_SUB, next-up(-2), MAX and powers2^520/535/540/541/542 distinguish
ordinary, admitted subnormal and failed-PV hypotheses.

Two complementary policies, ExactRatio/even and NineDecimal/away, accompany
every definition. Their dirty/risk ideal formulas agree while selected AI
and clean references can differ. This is not a substitute for the separate
source/cash corpus's complete four-policy and tie-stage census.

Seven A–G statement cases embed primary source inputs from the pinned
31 CFR356 Appendix B snapshot described in the [source examples](M8_US_TREASURY_SOURCE_EXAMPLES.md).
The generator checks their selected clean prices rounded to six decimals
against printed 99.057893,99.838183,99.805118,99.730918,102.214586,
99.777074 and99.738045. That literature-display assertion is separate from
analytical comparison. Input yields are represented binary64, not exact
printed decimal percentages. The half-period single-payment anchor also
asserts Macaulay1/4 and discriminates b*Modified; for exact decimal y=.05
the latter is41/162.

## Target-driven inverses

Targets are chosen independently in native quote space: dirty100 or clean97,
then the exact published-AI ideal shift and the separately rounded cached-AI
binary target shift are both retained. No target is obtained by pricing at
a hidden expected yield. There are **136 multi-payment exact-rational
bisections**, **8 exact one-payment roots** and **1 excluded ideal-image
control**. Every multi-payment result has positive/negative exact residual
signs at its rational endpoints after 180 bisections. Every one-payment
result verifies `y=2*s/r*(A0/T-1)` by exact substitution and has zero/zero
endpoint signs. Outward IEEE endpoints retain these rigorous ideal roots.

Each ordinary caller bracket is [-1.5,8], with explicit options/cap.
`uniform_price_error` bounds N3 error using the greatest ideal price at its
lower endpoint. `slope_lower=-D'_ideal(upper)>0` follows from the positive
second derivative. `target_shift_error` retains the difference between the
resolved binary target and ideal quote-plus-publication target.
`coordinate_allowance` is their sum divided by that slope plus a
conservative twice-real solver tolerance over the original bracket. For
these finite brackets that dominates M6's `(1+4t)*T_real+2*MIN_SUB` bound.
Conditioned endpoints are rounded outward from exact rational arithmetic.
The bound is conditional on valid callbacks throughout the retained bracket;
it is not a prediction of solver counts or which cached endpoint returns.
Eight source A/E rows have a radius at least the entire 9.5-wide caller
bracket and explicitly say `uninformative-coordinate`; the remaining 136
say `conditional-bracket-coordinate`. Of these 144 radii, 116 are below
1e-6. The exact ideal root brackets remain meaningful even where the
observational comparison bound is uninformative.

For n=0,alpha<1, `ideal_image` states the **excluded** upper value
`A0/(1-alpha)`. The additional positive target above it has no ideal root.
It asserts no observed API rejection. An observed staircase may attain or
exceed an ideal supremum; the adopted contract forbids ideal-upper target
prechecks. The corpus does not infer observed continuity, strict decrease,
universal attainability or a result for every caller bracket. Actual solver
reasons, plateaus, counts and typed callback failures belong to independent
prescribed observations and later integration tests.

## Schema, provenance and reproduction

Both are schema1 fixtures for the unchanged shared fixture reader. The 74/51
column sequences are frozen by the generator's explicit lists. `num/den`
means exact rational, `a..b` means inclusive actual original indices, and
`num/den*count` separated by semicolons is run-length encoding of raw A_j.
Quasi dates are reconstructed directly from M, EOM and quasi_count, never
recursively clamped. Every row lists FR-UST requirement IDs.

Complete semantic input fingerprints use dated,issue,maturity,coupon,
first_coupon,eom,quasi_count,settlement,daily_basis,tie_rule and yield_bits
for price rows. Inverse inputs append target_kind,target_bits,caller_lo_bits,
caller_hi_bits,abs_tol_bits,rel_tol_bits,max_evals instead of yield_bits.
Expected coordinates, derived first-period and labels are excluded. Header
input digests cover these TSV projections plus LF in file order, and unique
input counts equal record counts. A row fingerprint hashes its selected
fields joined by TAB without a final LF; header input digests include each
record's final LF. Root's fail-closed approved schema registry
must independently approve that exact semantic input list.

Headers retain actual UTC time/platform/Python, generator SHA-256/FNV64,
both adopted contracts, all 26 exact FR-UST requirement rows, body including
column line, records alone, profile counts and input digests. The generator
refuses initial output if either path exists. `--check` reads frozen
header timestamp/platform and requires byte equality without writing.

```sh
python3 -I validation/generator/treasury_ideal_oracles.py --check
```

| Artifact | SHA-256 |
|---|---|
| Generator | `ca822f899925583ccb15187c110c05bd0d29e8e6ec932ba859461c5942bdb5b8` |
| Price/risk fixture | `08bdb9ce17a1c523ed7cbe456185be1b305a950ef3261ca0148ff5a8471b4a31` |
| Inverse fixture | `40a16012bd1378122b6f17f1792bad8b66433487c926ff3f40d85a266259e517` |
| Main contract | `50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93` |
| Cash contract | `25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585` |

Before root attestation, initial704/145 output bytes were preserved at
`/private/tmp/sabiquant-treasury-ideal-initial-v1/`. The second714/145 bytes
were preserved at `/private/tmp/sabiquant-treasury-ideal-initial-v2/` before
adding exact-term and literature checks. The latter check additions changed
no financial records. Initial-to-second corrections added10 subnormal-price
rows, corrected the kebab header key, directed bound serialization outward
and included reference-center display error. These are recorded provisional
revisions, not changes to frozen expectations. Root owns the final review,
attestation, expected-output freeze and implementation gate.


Root's independent pre-freeze audit then found a substantive label-mapping
error: normalized radii built in Modified/Convexity/Macaulay accumulation
order had been attached to Modified/Macaulay/Convexity output columns.
The third provisional bytes were preserved unchanged at
`/private/tmp/sabiquant-treasury-ideal-initial-v3/` before correction.
An explicit per-name dictionary now attaches all eight budgets. The corrected
fixture changes only the Macaulay/Convexity allowances and their comparison
labels; every ideal value/bit/class and all inverse records remain unchanged.
The corrected classification is the5,638/68/6 census above. A separate exact
half-period semantic check binds Modified20/81 to prefix power1,
Macaulay1/4 to power0 and Convexity800/6561 to power2, checking their complete
radii. This was a caught provisional-oracle defect, not a production or
adopted-contract change. Root's complementary review and reproduction must
use the corrected hashes above.


The fourth provisional bytes were then preserved at
`/private/tmp/sabiquant-treasury-ideal-initial-v4/` before classifying the
8 uninformative inverse coordinate radii explicitly. This final qualification
changes only those 8 comparison labels, with no numeric expectation changes.
All 714 price/risk records remain byte-identical to the corrected fourth
candidate. The initial expectation revisions were complete before root's
freeze; the generator continues to refuse output overwrite.
