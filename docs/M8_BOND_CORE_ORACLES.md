# Independent bond core oracles (`sabiquant::bonds`)

| Field | Value |
|---|---|
| Scope | Frozen local M8 bond core [contract](M8_BOND_CORE_CONTRACT.md): per-100 exact binary64 quote conversion (`price_per_100_from_amount`, `amount_from_price_per_100`), zero-coupon and fixed-rate bullet bond settlement planning (`BondPricingPlan::new`), coupon/redemption entitlement classification, earned-minus-detached accrued interest, and explicit accrual work-budget bounds (`FR-BOND-01..10`) |
| Baseline | `8bfa292`; local-only M8 bond core tranche |
| Authority | IEEE 754 bit fields decoded with Python integer shifts; exact Python rational arithmetic (`fractions.Fraction`); Python `datetime.date` civil arithmetic and actual day counting; no Rust, QuantLib, or floating-point intermediary expectations for quote conversion or entitlement classification |
| Additional checks | The frozen generator evaluates quote expressions at 256/512 Decimal digits but checks only their sign; this is not a value cross-check. Plan discount factors use Python binary64 arithmetic. See the post-commit audit below. |
| Corpus | 18,152 rows, 18,128 distinct serialized operation/input tuples, 2,742,789 bytes |
| Consumer status | `tests/bonds_core_oracles.rs` evaluates every row; integrated results are in [M8_BOND_CORE_VALIDATION](M8_BOND_CORE_VALIDATION.md) |

The [generator](../validation/generator/bond_core_oracles.py) was authored against
the frozen [bond core contract](M8_BOND_CORE_CONTRACT.md) and its ten `FR-BOND`
requirement rows (`FR-BOND-01..10`). It loads no Rust binary, external reference
runtime, or third-party package, using only the Python standard library.

The [fixture](../validation/fixtures/bonds/bond-core-oracles.fixture) covers:
1. `per_100_from_amount`: exact single-rounding IEEE 754 binary64 round-ties-to-even conversion of `(amount * 100) / face_amount` across normal binades, subnormal boundaries, exact half-ULP ties, underflow to signed zero, overflow to signed infinity, and face/amount input validation.
2. `amount_from_price_100` (`amount_from_per_100`): exact single-rounding IEEE 754 binary64 round-ties-to-even conversion of `(face_amount * price_per_100) / 100` across normal binades, subnormal boundaries, exact half-ULP ties, underflow to signed zero, overflow to signed infinity, and face/price input validation.
3. `bond_plan`: zero-coupon and fixed-rate planning (`BondPricingPlan::new`) over explicit settlement/payment/ex-coupon dates, `OnOrAfter` and `After` ex-coupon boundaries, `Include` and `Exclude` payment-date inclusion, and selected explicit work caps. The fixture does not generate a business-day settlement lag or ex-coupon cutoff calendar. Its rotating 400-year sample is not a translation-invariance comparison.

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
| `QUOTE-MATRIX` | 672  | Structured product of 14 faces including 1, 25, 50, 75, 100, 1000, 1,000,000, .01, 3, 7, minimum subnormal, maximum subnormal, minimum normal and MAX, with signed/zero values |
| `QUOTE-BINADE-SWEEP` | 12,276  | Exponent-field samples `range(0,2047,8)`, four fraction patterns (0, 1, alternating bits, all bits), and faces 100/1000/75; this is not every binary64 exponent |
| `QUOTE-SUBNORMAL-TIE` | 1,260 | Directed subnormal and half-ULP tie construction (`2*q + 1` in unit `2^-1075`) for both `/face * 100` and `*face / 100` |
| `QUOTE-RANDOM` | 2,400  | Deterministic `Random(2026101608)` bit-pattern cases |
| `PLAN-ZERO-COUPON` | 450  | Selected settlement dates and both inclusion modes, five compounding families with three supplied yields |
| `PLAN-EX-COUPON-MATRIX` | 462  | Seven explicit ex-coupon configurations, eleven dates, two inclusion modes, three signed coupon rates; Actual/365 Fixed accrual |
| `PLAN-DELAYED-PAYMENT` | 80  | 2026-07-15 first raw accrual end with payment 2026-07-20, and second payment 2027-01-18; earned accrual clips at raw end |
| `PLAN-EARLY-PAYMENT` | 80  | 2026-07-15 first raw accrual end with payment 2026-07-13; a payment before settlement is ceased and contributes no accrued adjustment |
| `PLAN-REOPENING-WORK-BUDGET` | 24  | Selected caps 0, 1, 40, 45, 50 and 500; all 24 rows fail (6 pre-issue, 18 work-limit), so this profile supplies no successful reopening or cap-equality evidence |
| `PLAN-400YEAR-CYCLE` | 400  | One rotating case for each year 2000..2399; no paired +400/+800-year equality assertion |
| **Total** | **18,152** | **17,011 `ok`, 832 `bond.value`, 213 `bond.accrual_work_limit_exceeded`, 96 `bond.settlement_before_issue`** |

## Post-commit evidence correction

A source audit after `5ea07cd` corrected the profile/API descriptions above.
The generator and fixture remain byte-identical; the old header's reference
label still contains `decimal-256-512`, which records performed arithmetic,
not a successful comparison of rounded values. Exact integer/Fraction quote
rounding remains the authoritative expectation. Plan value expectations are
binary64 model comparisons, with the consumer's two-ULP allowance scoped to
these retained rows and outputs, not an ideal-price accuracy guarantee.

New supplemental evidence is recorded in
[M8_BOND_CORE_DECIMAL_AUDIT](M8_BOND_CORE_DECIMAL_AUDIT.md). The existing consumer
now verifies the fixed body/source fingerprints, sequential IDs, actual
18,128 distinct input tuples, unused quote fields and full declared censuses;
duplicate context/census keys are rejected. New public edge suites separately
check partial-product underflow and gross-conditioned signed cancellation.
A near-zero net price can have a large relative/ULP error despite individually
small flow errors; the ideal-price analysis must propagate absolute errors
against gross discounted exposure.
