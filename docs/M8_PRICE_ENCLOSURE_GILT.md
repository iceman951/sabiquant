# Conventional Gilt price enclosure implementation

`GiltPricingPlan::enclose_price_per_100` now produces controlled price evidence
from the actual retained source plan. This follows the Generic implementation
in E3 and preserves the adopted [contract](M8_PRICE_ENCLOSURE_CONTRACT.md).
Treasury, Canadian, originating cash composition and final integrated
verification remain open. Full financial debug execution is still pending;
this implementation unit does not establish M8 acceptance.

## Source and complete target

The target is the complete sum of retained bundled amounts times
`(1+y/2)^(-L/s)`. The adapter consumes each bundle's original index, exact
amount and coordinate; its terminal bundle already contains par exactly once.
Clean subtracts the exact signed retained AI recipe interpreted with the
actual decimal coupon. Rounded binary64 accrued/observed price getters are
not premises. There is no calendar query or prepared-plan clone per call.

Nonfinite quotes and the global `y > -2` condition precede frame initialization,
including zero coupons. Original-order admission metadata completes before
price work or valid-domain unresolved results. Source conversion uses bounded,
prepaid five-word blocks and borrowed limbs; no unmetered general Wide
multiplication, division or GCD enters the price operation.

Zero yield uses the consuming exact named fold. Its sealed semantic graph has
regular `2*10^scale`, an odd-first publication denominator when applicable
(otherwise a structural alias), and the original AI recipe denominator, with
at most three roles. This graph is retained for both price kinds; only Clean
adds the negated AI term. Bundled par and assigned source roles are preserved.

General arithmetic uses the shared bounded log/exp and signed streaming
reduction, then applies the basis shift before final fitting/export. The
preferred point policy, origin borrow, DECLARED cash evidence, independent
work budgets and honest unresolved outcomes are the same controlled contract
as Generic. No physical stack-peak or CPU-time promise is inferred; E5 remains
the whole-producer resource and final integration exit.

## Actual verification

The public [financial consumer](../tests/bonds_enclosure_gilt.rs) passes its
complete native release run: S300 source plans, P1464 price rows, Q870
admission rows exercised with both bases, H8 at all five precisions, both B2
rows as finite containing results, and repeated/budget controls. Source stayed
unchanged during that completed run. Expectations and frozen fixtures were
not regenerated or repaired.

Six self-contained [unit controls](../src/bonds/enclosure/gilt_controls.rs)
pass in debug and release, and from the extracted package. They exercise quote
precedence/original bits, original-flow admission/work stops, raw long-first AI
and first-source roles, zero-yield point fitting, complete-target cutpoints and
threaded budget independence. Strict all-target Clippy, format, private docs
and diff checks pass on the implementation.

Full financial debug execution remains live with durable output. Its partial
checkpoints are not a complete pass, and its result must still be inspected.
Final current integrated verification after the remaining models belongs to E5.
