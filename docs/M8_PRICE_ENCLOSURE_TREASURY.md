# Fixed-principal Treasury price enclosure implementation

The actual `TreasuryPricingPlan::enclose_price_per_100` is implemented under
the adopted [contract](M8_PRICE_ENCLOSURE_CONTRACT.md), following Generic and
Gilt. Its full financial release partition and unit/package checks pass.
Full financial debug remains pending. Canadian, E4 and final E5 verification
remain open; this local unit does not establish M8 acceptance.

## Retained target

Dirty is the complete sum of retained raw coupon/par bundles divided by
`q*b^j`, with `q=(2s+r*y)/(2s)`, `b=(2+y)/2` and each original cached integer
`j`. Par is already included once in the terminal bundle. Clean subtracts
the selected published AI, exactly `published_count/10^6`; raw AI and rounded
public getters are not price premises.

Nonfinite yields and the global `y > -2` condition precede frame placement.
Prepared `0<r<=s` then proves exact q positive. Admission metadata runs for
every original bundle, including zero coupons, before valid unresolved work.
The producer queries no calendar and clones no prepared plan.

Zero yield uses the shared consuming exact fold with the sealed regular,
raw-first (or structural regular alias), and published-AI denominator roles,
at most three. Original first indices and cached source denominators remain
authoritative. Source bridging uses prepaid small blocks and borrowed limbs;
old publication/GCD/division routines are not called as uncharged shortcuts.

General pricing builds exact q before its directed reciprocal. Integer j0
uses exact unit; a common bounded ln(b) is prepared lazily for positive j.
Complete signed reduction and the selected published AI shift precede final
point fitting/export. The shared controlled proof, budget, origin, declared
evidence and unresolved rules apply without a native affine/PV range gate.

## Actual checks

The [public consumer](../tests/bonds_enclosure_treasury.rs) passes S2160,
P9024, Q2040 exercised with both bases, H12 at all five precisions, both B2
rows as finite containing results, and repeat/budget controls in the final
frozen-source release run. Its source/test pins match before and after.
An earlier completed run had a production helper extraction during execution;
it is preserved separately and is not used as final-current proof.

Eight self-contained [controls](../src/bonds/enclosure/treasury_controls.rs)
pass debug/release and in the extracted package. They cover original-bit
global precedence, exact q admission, original j/role selection, published-AI
choice, original-flow work stops, completion/point/export cutpoints, preferred
points and independent threaded budgets. The first completion-cut sample was
too short to stop before target completion; only its flow-count trigger was
corrected, retaining the mathematical expectations and first failure.

Current strict all-target Clippy, format, private docs and diff checks pass.
The full financial debug run has durable output and remains required. Typed
resource structure is not a physical peak/CPU-time claim; whole-producer
resource and final integrated-current verification close only in E5.
