# Independent bond core oracles (`sabiquant::bonds`)

| Field | Value |
|---|---|
| Scope | Frozen local M8 bond core [contract](M8_BOND_CORE_CONTRACT.md): per-100 exact binary64 quote conversion (`price_per_100_from_amount`, `amount_from_price_per_100`), zero-coupon and fixed-rate bullet bond settlement planning (`Bond::plan`), coupon/redemption entitlement classification, earned-minus-detached accrued interest, and explicit accrual work-budget bounds (`FR-BOND-01..10`) |
| Baseline | `8bfa292`; local-only M8 bond core tranche |
| Authority | IEEE 754 bit fields decoded with Python integer shifts; exact Python rational arithmetic (`fractions.Fraction`); independent Howard (1989) civil date algorithm and actual day counting; no Rust, QuantLib, or floating-point intermediary expectations for quote conversion or entitlement classification |
| Additional checks | 128-digit Python `decimal.Decimal` `ROUND_HALF_EVEN` cross-check on non-tie normal per-100 quotes; independent entitlement count/order/accrual cross-check on every successful `BondPricingPlan` |
| Corpus | 18,152 rows, 18,128 distinct serialized operation/input tuples, 2,742,789 bytes |
| Consumer status | `tests/bonds_core_oracles.rs` evaluates every row; integrated results are in [M8_BOND_CORE_VALIDATION](M8_BOND_CORE_VALIDATION.md) |

The [generator](../validation/generator/bond_core_oracles.py) was authored against
the frozen [bond core contract](M8_BOND_CORE_CONTRACT.md) and its ten `FR-BOND`
requirement rows (`FR-BOND-01..10`). It loads no Rust binary, external reference
runtime, or third-party package, using only the Python standard library.

The [fixture](../validation/fixtures/bonds/bond-core-oracles.fixture) covers:
1. `per_100_from_amount`: exact single-rounding IEEE 754 binary64 round-ties-to-even conversion of `(amount * 100) / face_amount` across normal binades, subnormal boundaries, exact half-ULP ties, underflow to signed zero, overflow to signed infinity, and face/amount input validation.
2. `amount_from_price_100` (`amount_from_per_100`): exact single-rounding IEEE 754 binary64 round-ties-to-even conversion of `(face_amount * price_per_100) / 100` across normal binades, subnormal boundaries, exact half-ULP ties, underflow to signed zero, overflow to signed infinity, and face/price input validation.
3. `bond_plan`: zero-coupon and fixed-rate settlement planning (`Bond::plan`) across settlement horizons (pre-issue error, issue, mid-period, period boundary, ex-coupon window, delayed payment after accrual end, early payment before accrual end, maturity boundary, post-maturity, and 400-year Gregorian cycles), `Inclusive` and `Exclusive` ex-coupon boundaries, `IncludeOnSettlement` and `ExcludeOnSettlement` payment-date inclusion modes, and explicit `max_accrual_work_units` budget enforcement (`cap - 1`, `cap`, and multi-decade reopening sweeps).

## Reproduction and pins

From the repository root, using Python 3.9 or later:

```sh
python3 -I validation/generator/bond_core_oracles.py
python3 -I validation/generator/bond_core_oracles.py --check
```

The first command writes `validation/fixtures/bonds/bond-core-oracles.fixture`.
The second recomputes every expectation and cross-check, verifies both the
contract pin and the `FR-BOND` requirements pin, and compares the complete
fixture byte-for-byte (`"check": "identical"`).

| Artifact | SHA-256 |
|---|---|
| Frozen M8 bond core contract (`docs/M8_BOND_CORE_CONTRACT.md`) | `009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c` |
| `FR-BOND-01..10` requirement rows (`docs/REQUIREMENTS.md`) | `dbae973922e98619094814a7b0edf1a77d747d5cf7246254699063a7e996b8cb` |
| Generator (`validation/generator/bond_core_oracles.py`) | `afe27c0ce1a362492a98202c2882271cabbc3b1d510bb49dd3d5215460055b18` |
| Fixture body (`validation/fixtures/bonds/bond-core-oracles.fixture`) | `b7c7e1f8bf9de9d7b90c807a14b51131fd90d848de1396c0d87f2c024f2e6e09` |
| Complete fixture (`validation/fixtures/bonds/bond-core-oracles.fixture`, 2,742,789 bytes) | `5a5dc4f4ec573d189b8b7f87aeaf335b5a253f1ad983633612dfbe2ae8ae7817` |

The fixture body FNV-1a64 is `2915133b0fe0a519`.

## Practical fixture schema

After the `# key: value` metadata header, one blank line, and the column line,
every UTF-8 record has fifteen tab-separated fields:

```text
id profile op face_bits value_bits spec settlement inclusion yield_dcc yield_comp yield_bits work_cap expected context output
```

IDs are sequential `BOND-000001` through `BOND-018152`. Unused fields are `-`.

| `op` | Active input fields | Public operation |
|---|---|---|
| `per_100_from_amount` | `face_bits`, `value_bits` | `price_per_100_from_amount(f64::from_bits(value_bits), f64::from_bits(face_bits))` |
| `amount_from_per_100` | `face_bits`, `value_bits` | `amount_from_price_per_100(f64::from_bits(value_bits), f64::from_bits(face_bits))` |
| `bond_plan` | `face_bits`, `value_bits` (coupon rate), `spec`, `settlement`, `inclusion`, `yield_dcc`, `yield_comp`, `yield_bits`, `work_cap` | `BondPricingPlan::new(&bond, settlement, &yield_convention, work_cap)` and `plan.evaluate(yield_rate)` |

### Quote conversion rows (`per_100_from_amount`, `amount_from_per_100`)

- Inputs and outputs use exact 16-hex-digit IEEE 754 bit patterns (`0x...`), decoded via `f64::from_bits` and compared via `f64::to_bits` (`TOL-E0`).
- On `expected=ok`, `output` is the exact 64-bit IEEE 754 bit pattern of the rounded result (canonicalizing `±0.0` input to `+0.0`).
- On `expected=bond.value`, `context` records `phase=<bond face|price per 100|bond price|bond amount>,kind=<value.not_finite|value.out_of_domain|value.result_not_finite|value.result_out_of_domain>[,class=<NaN|PositiveInfinity|NegativeInfinity>]`.

### Bond plan rows (`bond_plan`)

- `spec` encodes `kind|issue|maturity|red_pay|red_bits|coupon_dcc|coupons`, where `coupons` is `-` for `ZeroCoupon` or `start@end@pay@ex;...` for `FixedRate`.
- `settlement` and `inclusion` (`Include` or `Exclude`) construct `BondSettlement`.
- `yield_dcc` and `yield_comp` construct `BondYieldConvention`, evaluated at `yield_bits` with `work_cap`.
- On `expected=ok`, `output` contains ten colon-separated fields:
  `ent_code:red_entitled:entitled_flow_count:accrual_work_used:dirty_bits:accrued_bits:clean_bits:dirty_p100_bits:accrued_p100_bits:clean_p100_bits`.
- On error (`bond.settlement_before_issue` or `bond.accrual_work_limit_exceeded`), `context` encodes exact structured fields (`settlement_date,issue_date` or `index,phase,maximum,used,required_at_least`).

## Corpus breakdown

| Profile | Rows | Description |
|---|---:|---|
| `QUOTE-INVALID-FACE` | 48 | Non-finite and non-positive `face_amount` and non-finite `amount`/`price_per_100` precedence checks |
| `QUOTE-MATRIX` | 672 | Structured Cartesian product of par/non-par face amounts (`100`, `1000`, `1_000_000`, `0.01`, `3`, `7`, `2^50`, `2^-40`, `1e12`, `1e-9`) and positive/negative/zero amounts and prices |
| `QUOTE-BINADE-SWEEP` | 12,276 | Sweep across binary64 exponents `[-1074, 1023]` with tie-sensitive significands (`1`, `1+ulp`, `1.5`, `2-ulp`, `3/7`, `100/3`) |
| `QUOTE-SUBNORMAL-TIE` | 1,260 | Directed subnormal and half-ULP tie construction (`2*q + 1` in unit `2^-1075`) for both `/face * 100` and `*face / 100` |
| `QUOTE-RANDOM` | 2,400 | Deterministic `Random(20261016)` bit-level stress cases across the full exponent range |
| `PLAN-ZERO-COUPON` | 450 | Zero-coupon settlement before issue, on issue, mid-life, on maturity, and after maturity under both inclusion modes and 3 discount conventions |
| `PLAN-EX-COUPON-MATRIX` | 462 | Semiannual fixed-rate coupon schedules across `ExCouponTerms` (`None`, `7d Inclusive`, `7d Exclusive`), both inclusion modes, 3 day-count conventions (`Act365F`, `Act360`, `ThirtyE360`), and 11 boundary settlement dates |
| `PLAN-DELAYED-PAYMENT` | 80 | Payment date after `accrual_end` (`2025-07-05` after `2025-06-30`), proving accrual clamps at `accrual_end` while entitlement follows `payment_date` and `ex_coupon_date` |
| `PLAN-EARLY-PAYMENT` | 80 | Payment date before `accrual_end` (`2025-06-25` before `2025-06-30`), proving detached remaining accrual when `payment_date < settlement < accrual_end` |
| `PLAN-REOPENING-WORK-BUDGET` | 24 | Multi-period reopening before settlement with `max_accrual_work_units` at `0`, `cap - 1`, `cap`, and `cap + 10` |
| `PLAN-400YEAR-CYCLE` | 400 | 400-year Gregorian cycle translations (`+400y`, `+800y`) preserving day-count, entitlement, and price bit patterns |
| **Total** | **18,152** | **17,011 `ok`, 832 `bond.value`, 213 `bond.accrual_work_limit_exceeded`, 96 `bond.settlement_before_issue`** |
