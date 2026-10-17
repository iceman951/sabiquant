# Local generic bond yield and risk implementation review

| Field | Value |
|---|---|
| Date | 2026-10-17, Asia/Bangkok |
| Authority | [Frozen local contract](M8_BOND_YIELD_RISK_CONTRACT.md), `FR-YR-01..16` |
| Reviewed production | [src/bonds/yield_risk.rs](../src/bonds/yield_risk.rs), [src/bonds.rs](../src/bonds.rs), [src/rates.rs](../src/rates.rs), [src/math/solver.rs](../src/math/solver.rs), [src/error.rs](../src/error.rs) |
| Result | No remaining reachable arithmetic, wide-reduction, rounding, sensitivity, solver-payload, or error-precedence defect found |
| Acceptance | Local engineering review; complete validation and independent financial/platform/owner gates remain separate |

## Scope and architectural integration

M8 Phase D implements generic prepared-plan yield-to-maturity inversion
(`BondPricingPlan::yield_from_price`) and all six analytical risk views
(`BondPricingPlan::risk` returning `BondRisk`) on top of the frozen M8 bond core
(`BondPricingPlan`), M4 interest-rate conventions (`Compounding`), M6 bracketed
Brent root-finding (`math::solver`), and M8 fixed-width unsigned arithmetic
(`amounts::uint::Wide<N>`).

Four new public types (`BondYieldReport`, `BondYieldSensitivity`, `BondRisk`,
and `BondYieldError`), six `BondYieldError` variants (`TargetConversion`,
`DirtyTargetOverflow`, `NoFutureCashFlows`, `NoYieldSensitivity`,
`CallbackEvaluation`, `Solver`), two new `BondError` variants (`YieldDomain`,
`RiskDirtyPriceZero`), and root `Error::BondYield` bring the public type catalog
to **101** (from 97) and the domain error catalog to **139** (from 131).

## Contract and mathematical assessment

| Contract area | Source assessment | Evidence / limits |
|---|---|---|
| Target resolution order (§2) | `resolve_target` converts `target.value()` via single-rounded `amount_from_price_per_100(target.value(), face)`, mapping failures to `BondYieldError::TargetConversion`. For `Clean` targets, `dirty_target_amount` adds `accrued_amount` in binary64 and maps overflow to `BondYieldError::DirtyTargetOverflow`. Both precede horizon/sensitivity/bracket checks. | Verified by hand suite and `FAILURE-PRECEDENCE` oracle rows. |
| Horizon and sensitivity classification (§3) | `classify_horizon` aggregates positive-time flows by `t.to_bits()` using an exact `Wide<33>` (2,112-bit) fixed-point accumulator on `2^-1074` (`SensitivitySum`). Same-time flows whose exact net sum is zero are excluded from positive-horizon count and positivity checks. If no positive-time flow has nonzero net sum, `NoFutureCashFlows` (when `entitled_flows.is_empty()`) or `NoYieldSensitivity` (reporting net zero-horizon `constant_amount` and `matches_target`) is returned before any yield trial. | Verified by `SENSITIVITY` oracle rows, opposite-sign same-date cancellation hand test, and unit tests. |
| Endpoint evaluation & power-of-two residual scaling (§4) | `evaluate_endpoint` evaluates all entitled flows at `bracket.lower()` then `bracket.upper()` via `rates::observe_present_value_at`. Maximum biased binary64 exponent across endpoint PV terms and `dirty_target_amount` sets `s = max(0, max_exp - 960)`. When `s == 0`, the unscaled exact sum minus `dirty_target_amount` is rounded once via `min_sub`. When `s > 0`, every term is scaled by `2^-s` (`scale_down_exact`, which cannot underflow normal exponents `>= 961 - 1023 = -62` into subnormal rounding loss) before exact wide reduction and `min_sub`. | Verified by `SCALED-EXTREME` oracle rows (`s = 63`), unit tests on `compute_scale_exponent`, `scale_down_exact`, and `min_sub`. |
| Fallible Brent solver integration (§5) | `math::solver::brent_with_payload` preserves the exact M6 Brent sequence (`f64_inverse_trial`, `f64_acceptable_trial`, `f64_midpoint`, `f64_tolerance_met`, canonical `+0.0` roots and residuals) while allowing interior trial callbacks to short-circuit with typed `FallibleSolverError::Callback(E)`. Endpoint exact roots (`f(a) == 0` or `f(b) == 0`) still evaluate both endpoints first so upper-endpoint domain/overflow errors are never masked by a lower-endpoint root. | Verified by `FAILURE-PRECEDENCE` and `ZERO-CLOSED` / `COUPON-32` oracle rows. |
| Post-solve report & lazy getters (§6) | After `brent_with_payload` converges at `y*`, `evaluate_unscaled_dirty_amount(plan, y*)` recomputes the unscaled `dirty_amount` at `y*`. `BondYieldReport` stores `dirty_amount` and computes `amount_residual` (`rn(dirty_amount - dirty_target_amount)`), `clean_amount` (`rn(dirty_amount - accrued_amount)`), `dirty_price`, and `clean_price` on demand with exact `BondError` contexts (`"bond yield report amount residual"`, `"bond yield report clean amount"`). | Verified by hand tests, oracle rows, and unit tests. |
| Analytical derivative & time-weight factors (§7) | `evaluate_flow_factors` in `src/bonds/yield_risk.rs` switches on `ActiveBranch` returned by `rates::observe_present_value_at`: `AtSettlement` (`t == 0`), `SimpleZeroRate` (`y == 0`), `SimplePositive` (`u = rn(1 + r*t)`), `PeriodicZeroRate` (`y == 0`), `PeriodicPositive` (`u = rn(1 + y/k)`), `ContinuousZeroRate`, `ContinuousGeneral`, and `SimpleCompoundPeriodic` (`SimpleThenCompound` using simple branch when `t <= 1/k` and periodic branch when `t > 1/k`). Domain violations `1 + y <= 0` (`Simple` with `t > 0`) and `1 + y/k <= 0` (`Periodic` / `SimpleThenCompound` with `t > 0`) fail with `BondError::YieldDomain` before discount/factor evaluation. | Verified across all 32 compounding/day-count combinations in `COUPON-32` and `FAILURE-PRECEDENCE`. |
| Extended-precision term products & `Wide<35>` reduction (§7) | Each flow's raw derivative and time-weighted terms `A_i * d_i * t_i * f_{1,i}`, `A_i * d_i * t_i * f_{2,i}`, and `A_i * d_i * t_i` are formed as exact 4-factor or 3-factor products of their binary64 significands (`u64` / `u128` -> `Wide<4>` 256-bit integer significand) and rounded **once** to binary64 via `round_extended_term` before being accumulated into a signed `Wide<35>` (2,240-bit) fixed-point register on `2^-1074`. | Verified by exact subnormal/midpoint/binade-carry unit tests and 141 oracle rows. |
| Normalized duration & convexity ratios (§7) | `modified_duration`, `macaulay_duration`, and `convexity` check `dirty_amount != 0.0` (else `BondError::RiskDirtyPriceZero`) and divide the retained exact `Wide<35>` sum (`-sum_d1`, `sum_tw`, or `sum_d2`) by the exact `2^-1074` decomposition of `dirty_amount` via `divide_wide_by_f64_to_f64` (using `Wide<37>` 2,368-bit shift-and-divide with exact remainder tie-to-even). For `Periodic(k)` (and `SimpleThenCompound(k)` when no positive-time flow uses the simple branch), `macaulay_duration` multiplies the exact `sum_tw` numerator by the exact integer significand of `rn(1 + y/k)` and divides by `dirty_amount * rn(1 + y/k)` in a single wide division, avoiding double rounding while still checking `rn(1 + y/k)` for overflow/underflow. | Verified by par-bond identity `D_mac == (1 + y/k) * D_mod`, zero-coupon identity `D_mac == t`, and unit tests. |

## Width and capacity verification

1. **Sensitivity accumulator (`Wide<33>`, 2,112 bits):**
   - Maximum finite binary64 value `f64::MAX = (2^53 - 1) * 2^971` has integer
     value `(2^53 - 1) * 2^2045 < 2^2098` in units of `2^-1074`.
   - Up to $2^{14} = 16,384$ flows fit within $2098 + 14 = 2112$ bits (`33 * 64`).
2. **Extended 4-factor term significand (`Wide<4>`, 256 bits):**
   - Four 53-bit significands have product $< 2^{212} < 2^{256}$ (`4 * 64`), so
     `Wide::<4>::from_u128(s_a * s_b) * Wide::<4>::from_u128(s_c * s_d)` never
     overflows.
3. **Risk accumulator (`Wide<35>`, 2,240 bits) and ratio kernel (`Wide<37>`, 2,368 bits):**
   - Each rounded binary64 term in units of `2^-1074` is $< 2^{2098}$, so a sum
     of bounded schedule flows fits comfortably in `Wide<35>` (2,240 bits).
   - In `divide_wide_by_f64_to_f64` and `divide_wide_by_f64_product_to_f64`, shifting
     the 35-limb numerator by up to 108 bits (`53 + 55`) requires at most
     $2240 + 108 = 2348 \le 2368$ bits (`Wide<37>`).

## Reviewed source snapshot

| File | Role |
|---|---|
| `src/bonds/yield_risk.rs` | `BondYieldReport`, `BondYieldSensitivity`, `BondRisk`, `BondYieldError`, `yield_from_price`, `risk`, extended-product & `Wide<35>`/`Wide<37>` kernels, and unit tests |
| `src/bonds.rs` | Module integration and re-exports |
| `src/rates.rs` | `ActiveBranch` and `observe_present_value_at` |
| `src/math/solver.rs` | `FallibleSolverError` and `brent_with_payload` |
| `src/error.rs` | `Error::BondYield` root integration and exhaustive 139-variant catalog tests |
