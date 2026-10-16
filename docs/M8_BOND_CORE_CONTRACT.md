# Local M8 bond core contract

Adopted for local implementation on 2026-10-16, Asia/Bangkok, from `8bfa292`,
under the owner's continuing local authority. This contract freezes the generic
zero-coupon and fixed-rate bond instrument terms, explicit settlement and
ex-coupon entitlement, earned-minus-detached accrued interest, exact binary64
per-100 quote conversion, and two-stage direct-horizon pricing plan specified in
[the bond API design, §§2–5](M8_BOND_API_DESIGN.md) and
[the design review, §§6–7](M8_DESIGN_REVIEW.md). It does not close NI-01
financial settlement adapters (phase C), bounded yield-to-maturity root
inversion, analytic duration/convexity, named market yield conventions (UK DMO
gilt, US Treasury 31 CFR Part 356 Appendix B, Canadian) or formal
milestone/platform/release acceptance.

## 1. Public surface (`sabiquant::bonds`)

```text
BondKind::{ZeroCoupon, FixedRate}                  Copy, Eq, Hash, ALL
ExCouponBoundary::{OnOrAfter, After}               Copy, Eq, Hash, ALL
ExCouponTerms::new(cutoff_date, boundary)          Copy, Eq, Hash
BondSettlement::new(date, payment_date_inclusion)  Copy, Eq, Hash
BondYieldConvention::new(accrual, compounding)     Clone, Eq, Hash
BondPriceKind::{Clean, Dirty}                      Copy, Eq, Hash, ALL
BondPrice::new(value_per_100_face, kind)           Result<BondPrice, BondError>
BondPrice::from_amount(amount, face, kind)         Result<BondPrice, BondError>
BondPrice::to_amount(self, face)                   Result<f64, BondError>
price_per_100_from_amount(amount, face)            Result<f64, BondError>
amount_from_price_per_100(value_per_100_face, face) Result<f64, BondError>
CouponEntitlement::{Ceased, BuyerEntitled, ExCouponDetached}  Copy, Eq, Hash, ALL
Bond::zero_coupon(issue_date, contractual_maturity, face, redemption)
Bond::fixed_rate(issue_date, schedule, face, coupon_rate, ex_coupon,
                 redemption, max_flows)
Bond::evaluate(&self, settlement, &yield_convention, yield_rate, max_accrual_work)
BondPricingPlan::new(&bond, settlement, &yield_convention, max_accrual_work)
BondPricingPlan::evaluate(&self, yield_rate)       Result<BondValuation, BondError>
BondValuation                                      Copy, PartialEq
BondError                                          Clone, PartialEq, Error
```

Every public type is immutable, `Send + Sync`, and has private fields. Discrete
types without `f64` implement `Eq + Hash`; types containing `f64` implement
`PartialEq` only and canonicalize input `-0.0` to `+0.0`. There is no `Default`,
implicit settlement lag, implicit par redemption, inferred frequency, currency
or global evaluation date. The public type census increases from 77 to 89.

## 2. Instrument construction and validation order

A `Bond` represents either a zero-coupon bullet bond or a fixed-rate bullet
bond with one constant strictly positive finite `face` and one separate strictly
positive finite `redemption` (`SimpleCashFlow`). `Bond::MAX_FLOWS` equals
`Leg::MAX_FLOWS` (`3,652,059`), counting the redemption payment, so a fixed-rate
bond accepts at most `3,652,058` coupons plus one redemption.

### `Bond::zero_coupon(issue_date, contractual_maturity, face, redemption)`

1. Reject `issue_date > contractual_maturity` with
   `BondError::InvalidIssueMaturity { issue_date, contractual_maturity }`.
2. Reject `issue_date > redemption.payment_date()` with
   `BondError::RedemptionBeforeIssue { issue_date, redemption_payment }`.
3. Validate `face` finite and `> 0.0` (`phase: "bond face"`).
4. Validate `redemption.amount() > 0.0` (`phase: "redemption amount"`).

### `Bond::fixed_rate(issue_date, schedule, face, coupon_rate, ex_coupon, redemption, max_flows)`

Count, shape and ordering checks precede allocation and coupon accrual work:

1. Reject `max_flows > Bond::MAX_FLOWS` with
   `BondError::InvalidFlowLimit { requested: max_flows, maximum: Bond::MAX_FLOWS }`.
2. Let `coupon_count = schedule.contractual_dates().len() - 1` and
   `total_flows = coupon_count + 1`. Reject `total_flows > max_flows as usize`
   with `BondError::FlowLimitExceeded { maximum: max_flows, required: total_flows }`.
3. Reject `ex_coupon.len() != coupon_count` with
   `BondError::ExCouponCountMismatch { expected: coupon_count, actual: ex_coupon.len() }`.
4. Let `contractual_maturity` be the final contractual date of `schedule`.
   Reject `issue_date > contractual_maturity` with
   `BondError::InvalidIssueMaturity { issue_date, contractual_maturity }`.
5. Reject `issue_date > redemption.payment_date()` with
   `BondError::RedemptionBeforeIssue { issue_date, redemption_payment }`.
6. Let `last_coupon_payment` be the final adjusted date of `schedule`. Reject
   `redemption.payment_date() < last_coupon_payment` with
   `BondError::RedemptionBeforeLastCoupon { last_coupon_payment, redemption_payment }`.
7. Validate `face` finite and `> 0.0` (`phase: "bond face"`).
8. Validate `redemption.amount() > 0.0` (`phase: "redemption amount"`).
9. In coupon index order `0..coupon_count`, for each `Some(terms)` in
   `ex_coupon`, let `payment_date = schedule.adjusted_dates()[index + 1]`;
   reject `terms.cutoff_date() > payment_date` with
   `BondError::ExCouponAfterPayment { index, cutoff_date, payment_date }`.
   Cutoffs on or before raw accrual start are permitted (for example on short
   stubs).
10. Reserve the coupon vector fallibly (`phase: "fixed-rate bond coupons"`),
    then construct one `FixedRateCoupon` per raw interval
    `[raw[i], raw[i + 1])` payable at `adjusted[i + 1]` with notional `face`
    and cloned `coupon_rate` (O(1) shared `Arc<AccrualConvention>`). Any
    accrual or product failure preserves `Some(i)` and the underlying
    `RateError` or `ValueError`.

The first raw accrual start may precede `issue_date` (for example, reopening
terms). Adjusted coupon payments may precede their raw accrual end under
Preceding/Modified Preceding schedule conventions and are retained as supplied.
The final coupon and redemption remain separate flows with stable indices even
when their payment dates coincide. Definition equality compares all retained
terms (`kind`, `issue_date`, `contractual_maturity`, `face`, `redemption`,
optional `schedule`, optional `coupon_rate`, `coupons`, and `ex_coupon`).

## 3. Settlement, entitlement and earned-minus-detached accrual

At settlement `S = settlement.date()` with `inclusion = settlement.payment_date_inclusion()`:

- `S < bond.issue_date()` fails first with
  `BondError::SettlementBeforeIssue { settlement_date: S, issue_date }`.
- A payment at `P` is **payment-eligible** iff `P > S`, or `P == S` with
  `PaymentDateInclusion::Include`.
- Redemption entitlement uses payment eligibility only; coupon ex-dates never
  detach principal redemption.
- A coupon at index `i` is:
  - `CouponEntitlement::Ceased` when its payment is not eligible;
  - `CouponEntitlement::ExCouponDetached` when its payment is eligible and
    `ex_coupon[i]` is `Some(terms)` with `S >= terms.cutoff_date()`
    (`ExCouponBoundary::OnOrAfter`) or `S > terms.cutoff_date()`
    (`ExCouponBoundary::After`);
  - `CouponEntitlement::BuyerEntitled` when its payment is eligible and not
    detached.

For any payment-eligible coupon with raw `[a, b)` and full rounded coupon
amount `A`, its earned interest `E(S)` is:

- `+0.0` when `S <= a` (no day-count query, 0 work units);
- `A` when `S >= b` (cached full coupon amount, no day-count query, 0 work
  units);
- `face * rate * YF_coupon(a, S)` when `a < S < b` (charged to the shared work
  budget under `phase: "partial accrual"`, validated `>= 0.0`, and multiplied
  via the shared 159-bit one-round triple product under
  `phase: "accrued amount"`).

Each payment-eligible coupon contributes to the bond's accrued adjustment:

- `E(S)` when `BuyerEntitled`;
- `exact_sum(&[E(S), -A])` when `ExCouponDetached` (`phase: "ex-coupon accrual"`).

Ceased coupons and redemption contribute nothing. All eligible coupon
contributions are reduced once via `exact_sum` (`phase: "accrued reduction"`)
into `accrued_amount` (canonical `+0.0`). In an ex-coupon window where `S < b`
and `A > 0`, `E(S) - A` is negative.

## 4. Exact binary64 per-100 quote conversion

Per-100 quote normalization and its inverse are binary64 analytical operations
rounded once from the exact binary rational of their IEEE 754 operands,
nearest with ties to even:

- `price_per_100_from_amount(amount, face)` validates finite `face > 0.0`
  (`phase: "bond face"`) first, then finite `amount` (`phase: "price per 100"`).
  For `amount == 0.0` it returns `+0.0`. For nonzero `amount = ±M_X · 2^e_X`
  and `face = M_F · 2^e_F` (`1 <= M < 2^53`), it rounds the exact rational
  `±(25 · M_X / M_F) · 2^(e_X - e_F + 2)` once to binary64.
- `amount_from_price_per_100(value_per_100_face, face)` validates finite
  `face > 0.0` (`phase: "bond face"`) first, then finite `value_per_100_face`
  (`phase: "bond price"`). For `value_per_100_face == 0.0` it returns `+0.0`.
  For nonzero `value_per_100_face = ±M_P · 2^e_P`, it rounds the exact rational
  `±(M_F · M_P / 25) · 2^(e_F + e_P - 2)` once to binary64.

Width proof: for any positive integer ratio `A / B · 2^E` with `1 <= A < 2^106`
and `1 <= B < 2^53`, let `k = floor(log2(A / B))` (`k` is either
`bits(A) - bits(B)` or `bits(A) - bits(B) - 1`, decided by one aligned shift
comparison in `u128`). The exact binade of the value is `top = k + E`:

- `top > 1023` overflows to signed infinity (`ValueError::ResultNotFinite`);
- `top < -1075` is strictly below half of `f64::from_bits(1)` (`2^-1075`) and
  underflows to signed zero (`ValueError::ResultOutOfDomain`);
- for `-1075 <= top <= 1023`, the target unit exponent is
  `u = max(top - 52, -1074)` and the shift is `s = E - u`, which satisfies
  `k + s = top - u in [-1, 52]`. When `s >= 0`, `N = A << s < B · 2^53 < 2^106`
  and `D = B < 2^53`; when `s < 0`, `N = A < 2^106` and
  `D = B << (-s) <= 2 · A < 2^107`. Thus `N`, `D`, and `2 * (N % D) < 2^108`
  all fit in `u128` without overflow. Quotient and remainder in `u128` decide
  nearest/ties-even rounding in a single step, including subnormal and
  binade-carry transitions, avoiding intermediate `100.0 * amount` overflow and
  `amount / face` underflow.

## 5. Two-stage direct-horizon pricing plan and work budget

`BondPricingPlan::new(bond, settlement, yield_convention, max_accrual_work)`
prepares entitlement, cached accrued interest, and direct discount horizons in
deterministic order:

1. Validate `settlement.date() >= bond.issue_date()`.
2. Initialize one shared `AccrualWork { maximum: max_accrual_work, used: 0 }`.
   Every `u64` budget, including `0`, is valid. As in M7, each actual query
   charges `1` plus ICMA overlapping determination periods or `1` plus absolute
   calendar-day span for Business/252; scalar, 30E/360 ISDA, and 30/360 US
   contexts charge `1`.
3. Charge and validate the equal-date discount anchor
   `yield_convention.accrual().year_fraction(S, S)` with `index: None`,
   `phase: "discount anchor"`, even if all flows are ceased, detached, or zero.
4. Reserve plan vectors fallibly (`phase: "pricing plan entitlements"`,
   `"pricing plan accrued terms"`, `"pricing plan discount flows"`).
5. Traverse coupons `0..bond.coupons().len()` in original order:
   - Determine `CouponEntitlement` (`Ceased`, `BuyerEntitled`, or
     `ExCouponDetached`).
   - If payment-eligible (`BuyerEntitled` or `ExCouponDetached`), compute
     `E(S)` (charging `"partial accrual"` only when `start < S < end`) and push
     `E(S)` or `exact_sum(&[E(S), -A])` (`phase: "ex-coupon accrual"`).
   - If `BuyerEntitled`, charge and validate the direct discount horizon
     `yield_convention.accrual().year_fraction(S, payment_date)` with
     `index: Some(i)`, `phase: "discount horizon"` (requiring `>= 0.0`, even
     when `coupon.amount() == 0.0`), and store the prepared flow. Detached
     coupons perform no discount horizon query.
6. If `bond.redemption()` is payment-eligible at `S`, charge and validate its
   direct discount horizon `yield_convention.accrual().year_fraction(S, redemption.payment_date())`
   at `index: Some(bond.coupons().len())`, `phase: "redemption discount horizon"`,
   and store the prepared principal flow.
7. Reduce accrued terms via `exact_sum` (`phase: "accrued reduction"`).

 Cumulative work bound: at most `MAX_FLOWS - 1` partial coupon queries plus at
most `MAX_FLOWS` direct discount queries plus `1` anchor query, each charging at
most `3,652,059` units, totaling `< 2.67e13 < u64::MAX`.

`BondPricingPlan::evaluate(&self, yield_rate)` evaluates a trial decimal annual
`yield_rate` without repeating date, calendar, ICMA or entitlement work:

1. Validate `yield_rate` against `yield_convention.compounding()` (finite, and
   `> -m` for periodic/hybrid methods) with `index: None`,
   `phase: "yield quote"`.
2. Reserve present-value terms fallibly (`phase: "present value terms"`).
3. In prepared flow order, evaluate the direct present value of each
   buyer-entitled flow at its prepared horizon using the shared M4 rate kernel
   (`index: Some(flow_index)`, `phase: "discount factor"`). Zero flow amounts
   still validate the discount factor first; nonpositive simple factors,
   `NumericalBoundary` uncertainties, and nonzero product underflow/overflow
   fail without being rescued by later cancellation.
4. Reduce present-value terms via `exact_sum` (`phase: "dirty price reduction"`)
   to `dirty_amount`.
5. Compute `clean_amount = exact_sum(&[dirty_amount, -accrued_amount])`
   (`phase: "clean price reduction"`).
6. Return `BondValuation`, which exposes amount-unit `dirty_amount`,
   `accrued_amount`, and `clean_amount`, plus fallible per-100 conversions
   (`dirty_price_per_100`, `accrued_per_100`, `clean_price_per_100`,
   `dirty_price`, `clean_price`).

`Bond::evaluate` runs `BondPricingPlan::new` followed by `evaluate`, preserving
the strict two-stage failure order where all preparation errors precede any
trial `yield_rate` error.

## 6. Error kinds and catalog

`BondError` is `#[non_exhaustive]` and wrapped by `Error::Bond(BondError)`:

| Variant | Stable kind | `source()` |
|---|---|---|
| `InvalidFlowLimit { requested, maximum }` | `bond.invalid_flow_limit` | `None` |
| `FlowLimitExceeded { maximum, required }` | `bond.flow_limit_exceeded` | `None` |
| `ExCouponCountMismatch { expected, actual }` | `bond.ex_coupon_count_mismatch` | `None` |
| `InvalidIssueMaturity { issue_date, contractual_maturity }` | `bond.invalid_issue_maturity` | `None` |
| `RedemptionBeforeIssue { issue_date, redemption_payment }` | `bond.redemption_before_issue` | `None` |
| `RedemptionBeforeLastCoupon { last_coupon_payment, redemption_payment }` | `bond.redemption_before_last_coupon` | `None` |
| `ExCouponAfterPayment { index, cutoff_date, payment_date }` | `bond.ex_coupon_after_payment` | `None` |
| `SettlementBeforeIssue { settlement_date, issue_date }` | `bond.settlement_before_issue` | `None` |
| `AllocationFailed { phase, requested }` | `bond.allocation_failed` | `None` |
| `AccrualWorkLimitExceeded { index, phase, maximum, used, required_at_least }` | `bond.accrual_work_limit_exceeded` | `None` |
| `Value { index, phase, cause }` | `bond.value` | `Some(cause)` |
| `Rate { index, phase, cause }` | `bond.rate` | `Some(cause)` |
| `Sum { phase, cause }` | `bond.sum` | `Some(cause)` |

The domain error catalog in `src/error.rs` increases from 112 to 125.
Requirements `FR-BOND-01..10` in `docs/REQUIREMENTS.md` §18 trace this contract.
