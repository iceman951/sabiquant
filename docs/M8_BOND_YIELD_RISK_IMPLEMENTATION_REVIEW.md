# Generic bond yield/risk source and evidence review

Reviewed on 2026-10-18 against local `a33e8a7`, then the audit working tree.
Authority is the frozen [contract](M8_BOND_YIELD_RISK_CONTRACT.md),
FR-YR-01..16. This review is local engineering evidence; numerical validation,
named market acceptance and native-platform/owner/release gates remain separate.

The earlier version of this report described nonexistent functions/errors and
33/35/37-limb algorithms. Its claims of eager endpoint validation, a scale-63
residual, binary64 intermediate risk terms and final repricing did not describe
the committed source. They are replaced below with the actual implementation.
No earlier contract or fixture is changed to match those incorrect descriptions.

## Actual implementation

| Contract area | Observed source behavior |
|---|---|
| Target | `yield_from_price` calls the existing single-round quote-to-amount conversion, then one exact-sum clean/AI shift if needed. Both precede workspace reservation and callbacks. |
| Residual | `s=0` for `abs(T)<2^970`, else 1. `min_sub_difference` retains the exact signed binary64 difference on `2^-1074` in `Wide<34>`; only the output exponent changes. `compute_scaled_residual` rounds once. |
| Workspace and classification | PV and key vectors reserve fallibly once. Keys retain shared active branch, exact positive horizon bits, amount magnitude and sign; only equal opposite-magnitude pairs certify mathematical constancy. Original pricing flows remain in their original order. |
| Constants | Separate 34-limb positive/negative sums retain exact time-zero `C`. One ordinary lower pricing call precedes match/mismatch errors. The exact comparison uses `C`, not its rounded optional diagnostic or observed callback price. |
| Brent | `brent_with_payload` generalizes the existing solver. Endpoints/history retain coordinate, residual and payload together. Lower exact-zero success uses one call; cap checks occur before invocation and failed invocations count. Reports take the selected cached dirty amount with no repricing. |
| Dirty-only evaluator | `evaluate_dirty_into` validates the quote and traverses the prepared flows, then exact-sums accepted PV terms. Inversion/risk do not form an unused clean or per100 view. |
| Shared factor observation | `observe_present_value_at` returns the same accepted branch, PV and Simple compound factor used by risk. Quote, factor and nonzero-PV policies remain the existing M4 policies; zero amount still validates its factor. |
| Risk terms | Simple, Continuous and Periodic formulas match contract §6. `delta=RN(m+y)` and `w=RN(t+RN(1/m))` use finite represented operands. `round_extended_term` forms at most five numerator/two denominator significands in `Wide<5>`, locates the exact binade and rounds once to 53 normalized bits with a separate exponent. |
| Risk accumulation | Six 243-word unsigned arrays on 32/64-bit targets collect positive/negative contributions exactly on `2^-8192`. One final subtraction per measure retains three signed sums. There is no per-term binary64 admission. |
| Getters | Raw views independently round retained sums. Normalized views divide extended sums directly by observed dirty price and check dirty zero first. Raw overflow/underflow does not block a finite normalized result. Negative dirty sign, signed underflow error zero and canonical successful zero are retained. |
| Report views | Amount residual and clean/quote views use cached fields only. Clean uses the existing exact-sum reduction and quote views use the existing single-round conversion. Their failures cannot add callbacks. |

The four public additions are `BondYieldReport`, `BondYieldSensitivity`,
`BondRisk` and `BondYieldError`, yielding 101 public types. The six yield-error
variants are `Target`, `Preparation`, `Solver`, `Callback`, `NoYieldSensitivity`
and `ConstantPriceMismatch`. The two bond variants are `ZeroDirtyPrice` and
`InternalCapacity`, yielding 139 domain variants. Root `Error::BondYield`
preserves typed immediate causes. These names replace the previous report's
invented `TargetConversion`, `DirtyTargetOverflow`, `NoFutureCashFlows`,
`YieldDomain` and `RiskDirtyPriceZero` variants.

## Capacity and rounding assessment

The source uses the contract widths, with checked carries/borrows/shifts and
quotient/remainder rounding. A residual coefficient is below `2^2099`; a
bounded time-zero constant accumulator is below `2^2120`. Both fit in 2176 bits.
The scalar numerator/denominator significands are below `2^265`/`2^106`, and
53-bit alignment remains below 320 bits. An extended term has no bit below
`2^-7470` and stays below `2^7269`; its `2^-8192` coefficient is below
`2^15461`. Adding `usize::BITS` headroom gives 243 words on 32/64-bit targets.
Normalized getter alignment remains inside the same accumulator width.

`round_wide_ratio_to_f64` handles subnormal ties, normal carries and the IEEE
overflow midpoint directly. It compares remainder with `d-r`. Genuine final
nonzero underflow/overflow produces typed `Value` errors, while private
capacity exhaustion is a distinct proof-excluded guard. The review found no
reachable arithmetic defect in these source paths; additive independent
full-range kernel evidence is still required to substantiate the contract.

## Corrections to test evidence

The original embedded suite mutated private `BondYieldReport` fields and
constructed an impossible extended exponent to force defensive branches.
The audit removes those manufactured financial states. Raw risk admission
controls now obtain retained sums through valid finite operands (`MIN_SUB*.5`
and `2^1023*2`), the actual term rounder and accumulator. All five amended
unit tests pass focused debug verification.

The new [public review suite](../tests/bonds_yield_risk_review.rs) has six tests
that pass debug/release and strict Clippy. It constructs a detached huge coupon
with negative accrued amount plus MAX redemption to prove that a dirty solve
and risk can succeed while an unused clean view overflows. Other controls
cover signed subnormal error fields, finite normalized results, cancellation
of individually huge risk terms, constant/callback/budget order, opposite-MAX
finite scaled error residuals and an earlier endpoint's retained payload.

Successful `amount_residual` overflow has a stronger exclusion than the earlier
report stated. Let `O=MAX+2^970` be the overflow midpoint. A bracket endpoint
with unscaled residual magnitude at least `O` has an opposite-sign endpoint
residual of magnitude at most `2*MAX-O=MAX-2^970`. With the fixed scale 1,
nearest-even halving preserves a strict ordering: the bad midpoint rounds to
`2^1023`, while the opposite side remains below it. Brent selects the smaller
absolute residual, so that overflowing endpoint cannot be a successful selected
best. Scale 0 cannot overflow by its target threshold, and FunctionZero reports
have exact `Dhat=T`. The defensive getter guard remains; tests do not forge a
report merely to execute it. This reasoning was independently challenged by
the public-test author and depends on preserved point/payload pairing.

## Coverage of the remaining guards

Fresh production-region instrumentation leaves 89 of 1,243 `yield_risk.rs`
regions unexecuted. The [validation report](M8_BOND_YIELD_RISK_VALIDATION.md)
assesses each one: eight propagate a refused workspace reservation, five build
the `amount_residual` overflow error excluded above, and 76 propagate a private
capacity or conversion guard. Those 76 are excluded by finite nonzero operand
invariants, the 34/5/243-limb width proofs, or a checked subtraction that runs
only after an explicit magnitude comparison. Every other production file keeps
exactly the uncovered source text of the phase C instrumentation.

## Evidence status

The existing 141-row generator/fixture remain frozen. Its hybrid comparison
uses strict `< exact(1/m)` in one branch instead of the adopted `<= RN(1/m)`;
its blanket 2-ULP output padding and unconditioned ideal tolerances are limited
regression checks, not a complete proof of the contract's numerical obligations.
The additive exact-kernel and conditioned financial fixtures, full native
checks, fresh coverage and extracted-package verification have completed. Their
results and pins are in the [validation report](M8_BOND_YIELD_RISK_VALIDATION.md).

No QuantLib implementation was read. Production arithmetic has not changed
in this audit; the only Rust source edit is confined to embedded tests. Named M8
market models and analytical enclosure producers remain required work.
