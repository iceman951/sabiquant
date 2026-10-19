# Independent conventional gilt numerical primitive oracles

This additive **75849-row** corpus was authored before gilt
production from the frozen [contract](M8_GILT_CONTRACT.md), chiefly
FR-GILT-07..14/20..22. All rows are explicitly **private primitive controls**.
Supplied admitted log, discount and PV observations do not assert that a
particular public yield/libm call produces those values. No invalid private
GiltRisk or invented signed financial plan is used.

The producer derived exact expectations with integer/Fraction arithmetic and
independently certified adjacent-representable rounding cells/parity. The
producer read no Rust financial implementation during this oracle task;
prior numeric-contract review and later cross-review are separate activities.
Root completed reproduction, schema and metadata checks after authoring.

## Artifacts and operation schema

| Artifact | SHA-256 |
|---|---|
| Frozen contract | `03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292` |
| [Generator](../validation/generator/gilt_numeric_oracles.py) | `be197c8408a5d3d0a394701925468d42a2eeca03b08d32f74d9ef37bbc9e7160` |
| [Fixture](../validation/fixtures/bonds/gilt-numeric-oracle.fixture) | `40a8bb169b9876906e60465805b49b95c49ac6289d0720608819505120f36915` |

| Operation | Rows |
|---|---:|
| AI | 2,932 |
| PV | 7,929 |
| REDUCE | 208 |
| RESIDUAL | 6,305 |
| TERM | 13,455 |
| Z | 45,020 |

PV rounds an exact source coefficient times an admitted binary discount once
to IEEE, including subnormal midpoint/carry and overflow cells. Z rounds the
exact represented g and rational coordinate directly. AI converts exact source
AI once. TERM forms exact represented-PV/represented-delta risk multipliers
and rounds once to53 normalized bits with a retained exponent. REDUCE retains
those terms on the2^-8192 basis and normalizes directly by observed dirty.
RESIDUAL uses positive targets, fixed scale0/1 and exact operand subtraction.

All 54 columns are named; unused values use a hyphen:

```text
id, op, family, reachability, source, c_coef, c_scale, r, s, j, redeem, dates, ai_branch, t, s2, y_bits, g_bits, discount_bits, pv_bits, delta_bits, measure, recipes, bundle_count, dirty_bits, target_bits, scale, exact_n, exact_d, pow2, negative, significand, exponent, basis_bit_lengths, basis_fnv64, status, expected_bits, alternate_bits, normalized_status, normalized_bits, amount_status, amount_bits, p1_status, p1_bits, p2_status, p2_bits, moment_status, moment_bits, modified_status, modified_bits, macaulay_status, macaulay_bits, convexity_status, convexity_bits, detail
```

Source/date inputs identify legitimate coefficient/coordinate provenance.
Z/TERM/REDUCE dates are previousQ|S|nextQ|payment; PV/AI dates are
Q0|I|Q1|F. Reduction recipes encode semicolon-separated runs of admitted
16hex discount bits times count, C/2 each and par100 at the final bundle.
Risk magnitudes use exactly243 little-endian u64 words (1944bytes), with
bit-length/FNV certificates. Negative first derivative and other positive
measures have separate IEEE statuses. Public reachability must be established
by actual source plans and the normal consumers, never by that private label.

## Independent rounding and precision checks

The primary IEEE converter uses CPython Fraction conversion and exact
adjacent-float midpoint/parity certificates, distinct from production's
specified aligned quotient/division algorithm. Exact ties use integer parity.
Non-ties are independently separated from a rounding boundary by more than
twice their Decimal160/240 half-ULP radius; exact zero is handled separately.
The check performed61692 divisions at each precision,117 exact ties and7644
exact zeros. Agreement is corroboration of the exact certificate, not a
universal Decimal or libm theorem.

Five-word scalar and243-word aggregate limits follow the frozen width proof.
The corpus exercises source scale39/147bit coefficients, grouped redemption,
early coefficient-rounding and subnormal double-rounding discriminators,
all IEEE exponent fields, civil r/s offsets, long L amplification, carries,
positive-target binades and direct normalization after raw range failures.
REDUCE inputs share a coherent delta and ordered coordinates with positive
principal; signed/zero-dirty forbidden financial states are not fabricated.

| Semantic family | Rows |
|---|---:|
| ALL-CIVIL-OFFSETS-EXACT-COORDINATES | 40,880 |
| CANONICAL-SOURCE-ZERO | 7 |
| EARLY-COEFFICIENT-DISCRIMINATOR | 2 |
| EXACT-DELTA-WIDTH-AND-RANGE | 648 |
| EXTENDED-SIGNIFICAND-TIES-AND-CARRY | 9 |
| FIXED-SEED-QUOTIENT-REMAINDERS | 512 |
| FULL-DATE-SPAN-ZERO-COUPONS-POSITIVE-REDEMPTION | 3 |
| FULL-DECIMAL-SCALES | 7,722 |
| GROUPED-REDEMPTION-DISCRIMINATOR | 1 |
| IEEE-RANGE-MIDPOINTS | 5 |
| LOG-SIGNIFICAND-BINADE-CENSUS | 4,125 |
| MAXIMUM-DATE-SPAN-AMPLIFICATION | 14 |
| NO-EARLY-QUASI-RATIO-ROUNDING | 1 |
| NORMALIZED-ONLY-CONVEXITY-UNDERFLOW | 1 |
| POSITIVE-SOURCE-RAW-NORMALIZED-INDEPENDENCE | 175 |
| POSITIVE-TARGET-ALL-BINADES | 6,305 |
| PV-ALL-EXPONENT-FIELDS | 12,282 |
| RAW-MOMENT-OVERFLOW-NORMALIZED-FINITE | 1 |
| RETAINED-53-THEN-IEEE-DISCRIMINATOR | 1 |
| SEVEN-SOURCE-BRANCHES-ALL-SCALES | 2,925 |
| SOURCE-COEFFICIENT-AND-PV-EXPONENT-MIX | 28 |
| SUBNORMAL-DOUBLE-ROUND-DISCRIMINATOR | 1 |
| TIES-CARRY | 132 |
| ZERO-DIVIDEND-REDEMPTION | 66 |
| ZERO-SOURCE-COEFFICIENT | 3 |

## Reproduction and retained obligations

```sh
python3 -I validation/generator/gilt_numeric_oracles.py --check
```

Every row/census/certificate is recomputed. Existing fixtures are not
silently rewritten. Original actual generation UTC and Python/platform are
preserved during --check. Before first acceptance, synthetic draft midnight
metadata was replaced with actual runtime metadata; complete prior drafts are
archived under `/private/tmp/sabiquant-gilt-preproduction-verify-v1`, and all
mathematical body/record bytes are required to remain identical.

A production-linked private adapter must still exercise these primitive
expectations; public price/risk/yield/cash tests must establish actual source
reachability. These are exact E0 staged references, not an empirical log/exp
profile, ideal-price enclosure, market convention or executed implementation
validation. Full generic regression, source audit, coverage, package and all
broader independent M8 gates remain required.
