# Bond yield/risk retained oracle and its limits

This corrected description was audited on 2026-10-18 against local `a33e8a7`.
It describes the actual frozen 141-row artifact, replacing the previous
21-column/one-operation schema, old error names and 33/35/37-limb algorithm
claims. The [contract](M8_BOND_YIELD_RISK_CONTRACT.md) and all existing generator
and fixture bytes remain unchanged. These rows are retained regression evidence;
the post-commit audit adds stronger numerical evidence separately.

The [generator](../validation/generator/bond_yield_risk_oracles.py) uses Python
stdlib integer/Fraction arithmetic and 160/240-digit Decimal models. It does not
invoke Rust or QuantLib. `--check` recomputes all output cells and requires the
two precisions to agree on their serialized results. That agreement is finite
corpus evidence; it alone is not a universal precision or platform-libm proof.

```sh
python3 -I validation/generator/bond_yield_risk_oracles.py --check
cargo test --offline --locked --test bonds_yield_risk_oracles
```

## Actual corpus and layout

The [fixture](../validation/fixtures/bonds/yield-risk-oracles.fixture) has 141
paired scenarios, 141 distinct serialized inputs, 72 TSV columns and 162,037 bytes.
Each row supplies both an inversion scenario and a risk-yield scenario, rather
than an `op` selector. Yield outcomes are 134 success/7 failure; risk construction
outcomes are 138 success/3 failure. Getter errors are retained separately.

| Family | Scenarios |
|---|---:|
| SINGLE-FLOW-32 | 64 |
| COUPON-32 | 64 |
| CONSTANT-PLAN | 3 |
| SIGNED-NONMONOTONE | 3 |
| EXTREME-SCALE | 3 |
| ERROR-TAXONOMY | 4 |

All 32 real compounding configurations occur: Simple, Continuous, plus each of
Compounded, SimpleThenCompounded and CompoundedThenSimple at event counts
1, 2, 3, 4, 6, 12, 13, 26, 52 and 365. These are 32 compounding configurations,
not 32 compounding/day-count combinations. The four day-count context labels
occur as `30360` (Bond Basis) in 93 rows, `A365F` in 16, terminal `30E360ISDA`
in 16 and explicit `ICMA` in 16.
US EOM flags, Business/252/TARGET/joint/override and other scalar day counts
are not covered by this retained corpus.

The complete actual column order is:

```text
id family profile issue settlement inclusion daycount comp
face_bits accrued_bits flows target_kind target_mode target_value_bits bracket_lo_bits bracket_hi_bits
abs_step_tol_bits rel_step_tol_bits max_evals risk_yield_bits yield_expected yield_error_kind yield_error_phase yield_error_stage
yield_error_evals sensitivity residual_scale_exp eval_count termination converted_target_bits dirty_target_bits solved_yield_lo_bits
solved_yield_hi_bits solved_dirty_lo_bits solved_dirty_hi_bits solved_clean_lo_bits solved_clean_hi_bits solve_residual_lo_bits solve_residual_hi_bits amount_residual_lo_bits
amount_residual_hi_bits ideal_root_bits risk_expected risk_error_kind risk_error_phase risk_error_index risk_dirty_lo_bits risk_dirty_hi_bits
p1_status p1_lo_bits p1_hi_bits p2_status p2_lo_bits p2_hi_bits moment_status moment_lo_bits
moment_hi_bits mod_dur_status mod_dur_lo_bits mod_dur_hi_bits mac_dur_status mac_dur_lo_bits mac_dur_hi_bits conv_status
conv_lo_bits conv_hi_bits ideal_p1_bits ideal_p2_bits ideal_moment_bits ideal_mod_dur_bits ideal_mac_dur_bits ideal_conv_bits
```

IDs use `BYR-0001` through `BYR-0141`. Inputs include issue/settlement/inclusion,
context, compounding, face, accrued amount, explicit flow date/amount/time bits,
target kind/mode/value, bracket/tolerances/cap and risk yield. The consumer
reconstructs bonds through family-specific zero/fixed schedule recipes.
Expected output cells include target conversions, scale/classification/callback
count/termination, solved-yield/dirty/clean/residual ranges, and every raw and
normalized risk view. `*_status` distinguishes success, nonzero underflow,
overflow and dirty-zero normalization. Errors retain the available
kind/phase/stage/count/index columns. `-` denotes unused cells.

## Numerical model actually used

The generator forms exact signed binary64 rationals, rounds Simple fused
`1+y*t`, reciprocal and PV operations, and uses Decimal log/exp models for
elementary branches with explicitly rounded binary64 intermediate seeds.
Its extended-term model rounds exact rational products once to 53 significant
bits with a separate exponent; it then sums these terms exactly and admits
raw/normalized results independently. It models target-selected residual scale
0/1 at `2^970` and a Python Brent traversal. The earlier report's endpoint-derived
`max_exp-960` scale and binary64-admitted 35-limb term model were descriptions
of algorithms that are absent from this source.

Per-flow ideal derivatives and root models request Decimal 160/240. However,
`evaluate_risk_oracle` accumulates its ideal Decimal sums outside the local
precision context, at the default 28 digits. Both requested precisions can
therefore agree on the same incorrect ideal aggregate. The consumer compares
numerical values, but several
comparison criteria are weaker than the complete contract:

- Non-Simple staged outputs receive a blanket 2-ULP padding. This is a retained
  corpus tolerance, not a derived bound near signed cancellation.
- Ideal derivative/risk checks use `1e-11*(1+abs(ideal))`, and ideal-root checks
  use `1e-6*(1+abs(ideal_root))`. Neither is a gross-conditioned error propagation
  or a slope-qualified root accuracy certificate.
- `active_branch` compares SimpleThenCompounded with strict `< exact(1/m)`.
  The frozen contract instead uses `<= RN(1/m)`; CompoundedThenSimple also
  requires the rounded threshold. Existing scenarios passing does not validate
  unexercised threshold/adjacent-time behavior.
- A direct source-only counterexample at SimpleThenCompounded frequency 2,
  `t=.5` returns Periodic instead of the contracted Simple branch.
  For Continuous, `y=0`, `t=1` and amounts `[binary64(1e40), 1, -binary64(1e40)]`,
  the exact ideal moment is 1; both 160/240 runs instead serialize 2996333109248
  because of the default-precision aggregate. These counterexamples are outside
  the 141 frozen rows and establish limits of the generator, not a production
  defect or a reason to rewrite historical expectations.
- The Simple flow model accepts a zero PV for nonzero `MIN_SUB`, `t=1`, `y=2`;
  the contracted shared nonzero-PV policy requires an underflow error. Its
  existing corpus does not validate that unexercised range boundary.
- The original consumer reconstructed profile-specific coupons and parsed a
  serialized flow time without checking it against actual preparation. The
  audit now checks every active public payment/amount and its supplied context
  horizon (exact scalar, adopted four-ULP ICMA), sequential IDs and distinct
  serialized inputs. All 141 retained scenarios pass the strengthened debug
  consumer; supplemental evidence still addresses the numerical model limits.
- This corpus has no complete finite-input residual/extended-term/accumulator
  census. Its 141 scenarios cannot replace the required full-range kernel cases.

These limitations are recorded without rewriting frozen expectations or
claiming that green regression tests prove the remaining numerical obligations.
The supplemental audit uses exact kernel cases and conditioned financial
bounds, including actual rounded thresholds, wide raw/normalized range rescue,
signed cancellation and additional contexts. Its own evidence is recorded
separately; named market and full M8 acceptance remain open.

## Frozen pins

| Artifact | SHA-256 |
|---|---|
| Generator | `16949a31bf9845bfc72b2dbf2a0c2b2c159efeaaa367c4ec79b2f0a46ad9b087` |
| Fixture body | `f39603b65d479a2b94094d97df167a9cbae2ab23f75875d0c18b1ffa8e3434bd` |
| Contract | `62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d` |
| FR-YR rows (LF terminated) | `eaecaec76703c6e7195684cf2f17fe93eec3ed8f00343361676f839379a24e08` |
| Complete fixture | `f8fc3ab55daff1679698bb9c7d05f40cbf94c09a4b0a3be9c2c155c517fca3dd` |

The consumer verifies frozen source/contract/requirements/body fingerprints.
Future changes to either numerical model require additive fixtures; this
historical source and artifact remain available for exact reproduction.
