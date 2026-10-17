# Independent bond yield and risk oracles (`sabiquant::bonds`)

| Field | Value |
|---|---|
| Scope | Frozen local M8 generic bond yield-to-maturity inversion and analytical risk [contract](M8_BOND_YIELD_RISK_CONTRACT.md) (`FR-YR-01..16`): `BondPricingPlan::yield_from_price`, `BondPricingPlan::risk`, `BondYieldReport`, `BondYieldSensitivity`, `BondRisk`, and `BondYieldError` |
| Baseline | `65ff67e`; local-only M8 Phase D tranche |
| Authority | Python standard-library exact rationals (`fractions.Fraction`), arbitrary-precision integers, and dual-precision `decimal.Decimal` (160 and 240 significant digits) for discount factors, derivative factors, extended products, and Brent root solving; no Rust executable, QuantLib runtime, or third-party package |
| Corpus | 141 rows, 141 distinct serialized operation/input tuples across 6 families, 162,037 bytes |
| Consumer status | `tests/bonds_yield_risk_oracles.rs` evaluates every row; integrated results are in [M8_BOND_YIELD_RISK_VALIDATION](M8_BOND_YIELD_RISK_VALIDATION.md) |

This document records the independent numerical oracle for the frozen
[M8 generic bond yield/risk contract](M8_BOND_YIELD_RISK_CONTRACT.md) and
requirement rows `FR-YR-01..16`. It is local engineering evidence, not owner API
acceptance, named-market compliance (UK DMO quasi-period powers, US Treasury
fractional first-period prefixes, or Canadian conventions), or multi-platform
release sign-off.

The artifacts are [validation/generator/bond_yield_risk_oracles.py](../validation/generator/bond_yield_risk_oracles.py)
and [validation/fixtures/bonds/yield-risk-oracles.fixture](../validation/fixtures/bonds/yield-risk-oracles.fixture).
Regenerate with:

```sh
python3 -I validation/generator/bond_yield_risk_oracles.py
python3 -I validation/generator/bond_yield_risk_oracles.py --check
```

`--check` recomputes all 141 rows, dual-precision 160/240-digit Decimal checks,
contract and requirement pins, and body/record/source SHA-256 and FNV-1a64
digests from scratch without reading any Rust binary or external input.

## Authority and cryptographic pins

| Artifact | SHA-256 | FNV-1a64 |
|---|---|---|
| Frozen contract (`docs/M8_BOND_YIELD_RISK_CONTRACT.md`) | `62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d` | `2a983dabd6706130` |
| `FR-YR-01..16` requirement rows (`docs/REQUIREMENTS.md`) | `eaecaec76703c6e7195684cf2f17fe93eec3ed8f00343361676f839379a24e08` | `a6273d939267f859` |
| Generator (`validation/generator/bond_yield_risk_oracles.py`) | `16949a31bf9845bfc72b2dbf2a0c2b2c159efeaaa367c4ec79b2f0a46ad9b087` | `065367972403362a` |
| Fixture body (`validation/fixtures/bonds/yield-risk-oracles.fixture`) | `f39603b65d479a2b94094d97df167a9cbae2ab23f75875d0c18b1ffa8e3434bd` | `8d530602803ecdd6` |
| Fixture records (excluding column header) | `05a16a9f5bc9daec8962a840ebbb74378b4b715c1dc2592174f2be8ef608be1a` | `512c09da58c1ca92` |
| Complete fixture (`validation/fixtures/bonds/yield-risk-oracles.fixture`, 162,037 bytes) | `f8fc3ab55daff1679698bb9c7d05f40cbf94c09a4b0a3be9c2c155c517fca3dd` | `e9bc408ef14bb5fc` |

## Practical fixture schema

After the `# key: value` metadata header, one blank line, and the tab-separated
column header line, each of the 141 LF-terminated records has 21 tab-separated
columns:

```text
id profile op face_bits rate_bits spec settlement inclusion yield_dcc yield_comp target_kind target_bits bracket_lo_bits bracket_hi_bits abs_tol_bits rel_tol_bits max_iter yield_bits expected context output
```

Sequential IDs are `BYR-000001` through `BYR-000141`. Unused input cells are `-`.
All IEEE 754 binary64 inputs and bit-exact outputs are encoded as `0x` followed
by 16 lowercase hex digits; ULP-enclosed outputs are encoded as
`0x<low_bits>..0x<high_bits>` (inclusive signed-ULP bit bounds).

| `op` | Active inputs | Public operation |
|---|---|---|
| `yield_from_price` | `face_bits`, `rate_bits`, `spec`, `settlement`, `inclusion`, `yield_dcc`, `yield_comp`, `target_kind`, `target_bits`, `bracket_lo_bits`, `bracket_hi_bits`, `abs_tol_bits`, `rel_tol_bits`, `max_iter` | `plan.yield_from_price(target, bracket, options)` and all `BondYieldReport` accessors |
| `risk` | `face_bits`, `rate_bits`, `spec`, `settlement`, `inclusion`, `yield_dcc`, `yield_comp`, `yield_bits` | `plan.risk(yield_rate)` and all six `BondRisk` analytical getters |

### Output and error encodings

- On `op=yield_from_price` and `expected=ok`, `output` contains 16 colon-separated fields:
  1. `conv_target_bits` (`TOL-E0`, exact single-rounded `amount_from_price_per_100`)
  2. `dirty_target_bits` (`TOL-E0`, exact `converted_target_amount` for `Dirty` or `rn(converted_target_amount + accrued_amount)` for `Clean`)
  3. `dirty_amount_bounds` (`0xlo..0xhi`, staged discount-factor enclosure at the accepted yield)
  4. `accrued_bits` (`TOL-E0`)
  5. `yield_bounds` (`0xlo..0xhi`, solver root enclosure)
  6. `status` (`ExactRootAtLower`, `ExactRootAtUpper`, `ExactRootInterior`, or `WithinTolerance`)
  7. `iterations_bounds` (`min..max`)
  8. `evaluations_bounds` (`min..max`)
  9. `final_bracket_lo_bounds` (`0xlo..0xhi`)
  10. `final_bracket_hi_bounds` (`0xlo..0xhi`)
  11. `solve_residual_bounds` (`0xlo..0xhi`)
  12. `scale_exp` (`residual_scale_exponent`, `TOL-E0`)
  13. `sensitivity` (`MathematicallyStrictlyDecreasing` or `Uncertified`, `TOL-E0`)
  14. `amount_residual_bounds` (`0xlo..0xhi`, `rn(dirty_amount - dirty_target_amount)`)
  15. `clean_amount_bounds` (`0xlo..0xhi` or `err:<kind>:<context>`)
  16. `dirty_p100_bounds` and `clean_p100_bounds` (`0xlo..0xhi` or `err:<kind>:<context>`)
- On `op=risk` and `expected=ok`, `output` contains 7 colon-separated fields:
  1. `dirty_amount_bounds` (`0xlo..0xhi`)
  2. `d1_bounds` (`price_derivative`, `0xlo..0xhi` or `err:<kind>:<context>`)
  3. `d2_bounds` (`price_second_derivative`, `0xlo..0xhi` or `err:<kind>:<context>`)
  4. `tw_bounds` (`time_weighted_amount`, `0xlo..0xhi` or `err:<kind>:<context>`)
  5. `mod_dur_bounds` (`modified_duration`, `0xlo..0xhi` or `err:<kind>:<context>`)
  6. `mac_dur_bounds` (`macaulay_duration`, `0xlo..0xhi` or `err:<kind>:<context>`)
  7. `conv_bounds` (`convexity`, `0xlo..0xhi` or `err:<kind>:<context>`)
- On error, `expected` records the exact `.kind()` string (`bond.yield.*` or `bond.*`) and `context` records comma-separated `key=value` fields verifying structured error payloads.

## Independent mathematical model

1. **Exact Coupon and Accrued Preparation:** Fixed-rate coupon amounts and
   accrued adjustments are computed using exact `Fraction` day-count year
   fractions and single-rounded binary64 triple products `rn(face * rate * yf)`,
   matching the frozen M7/M8 cashflow and bond-core contracts without calling
   Rust.
2. **Staged Discount and Active-Branch Model:** For each entitled cash flow at
   settlement-to-payment time `t_i`, the generator evaluates the staged discount
   pipeline in both 160-digit and 240-digit `Decimal` arithmetic (`dec_ln`,
   `dec_exp`, and exact `Fraction` rational steps), recording both the rounded
   staged binary64 discount factor `d_hat(y, t_i)` and the active branch
   (`AtSettlement`, `SimpleZeroRate`, `SimplePositive`, `PeriodicZeroRate`,
   `PeriodicPositive`, `ContinuousZeroRate`, `ContinuousGeneral`, or
   `SimpleCompoundPeriodic`).
3. **Extended-Precision Risk Accumulators (`Wide<35>` / 2240-bit):** For
   `op=risk`, each flow's derivative and time-weighted contributions are formed
   from the exact `Fraction` values of the binary64 operands `(A_i, d_i, t_i, f_{1,i}, f_{2,i})`,
   individually rounded once to binary64 (`TOL-E0` on zero/one-time/linear cases,
   1-ULP enclosure on transcendental factor cases), accumulated in exact rational
   arithmetic (mirroring the 35-limb `2^-1074` fixed-point accumulator), and
   divided by exact `dirty_amount` or `dirty_amount * (1 + y / k)` before final
   round-ties-to-even binary64 conversion.
4. **Scaled-Residual Brent Inversion:** For `op=yield_from_price`, the generator
   computes `scale_exp = max(0, max_i(exponent(A_i * d_i(y_lo)), exponent(A_i * d_i(y_hi))) - 960)`
   and runs an independent Python reference implementation of the M6 Brent solver
   (`f64_inverse_trial`, `f64_acceptable_trial`, `f64_midpoint`,
   `f64_tolerance_met`) on the scaled residual `rn(sum_i ldexp(pv_i, -s) - ldexp(dirty_target, -s))`.

## Corpus families

| Family | Rows | Coverage |
|---|---:|---|
| `SINGLE-FLOW-32` | 64 | Exact closed-form single-flow / zero-coupon yield inversion and analytical risk across all 32 compounding and day-count configurations (`Simple`, `Continuous`, `Periodic(1,2,4,12)`, `SimpleThenCompound(1,2,4,12)` across 4 day-count conventions), verifying exact `macaulay_duration == t` |
| `COUPON-32` | 64 | Multi-period fixed-rate coupon bonds across all 32 compounding/day-count configurations for both `yield_from_price` and `risk`, including `SimpleThenCompound` split-horizon (`t <= 1/k` and `t > 1/k`) cash flows, par/premium/discount quotes, and `macaulay_duration == (1 + y/k) * modified_duration` |
| `CONSTANT-PLAN` | 3 | Zero-horizon and empty-future-flow plans verifying `NoFutureCashFlows` and `NoYieldSensitivity` (both matched and mismatched constant amount) alongside `t == 0` analytical risk |
| `SIGNED-NONMONOTONE` | 3 | `BondYieldSensitivity` classification (`MathematicallyStrictlyDecreasing` vs `Uncertified`), negative coupon cancellation, same-date opposite-sign flows that pair off to zero, and non-monotone double-root brackets |
| `EXTREME-SCALE` | 3 | High-magnitude near-`f64::MAX` multi-flow bonds requiring positive `residual_scale_exponent > 0` during Brent bracketing/iteration and exact `Wide<35>` risk accumulation without intermediate binary64 overflow |
| `ERROR-TAXONOMY` | 4 | Structured failure precedence across `yield_from_price` and `risk` (`Callback` domain/overflow, `SolverError::InvalidBracket`, `SolverError::MaxEvaluationsExceeded`, and `BondError::Rate` yield domain rejection) |
| **Total** | **141** | **141 combined yield-and-risk records (134 yield `ok` + 7 yield `err`; 138 risk `ok` + 3 risk `err`)** |
