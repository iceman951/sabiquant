# Generic prepared-price enclosure implementation

The actual `BondPricingPlan::enclose_price_per_100(yield_rate, basis, budget)`
is implemented in [the Generic adapter](../src/bonds/enclosure/generic.rs).
This is the Generic part of E3 under the adopted
[four-model contract](M8_PRICE_ENCLOSURE_CONTRACT.md). Gilt, Treasury and
Canadian producers, originating cash composition and integrated resource
verification remain open. This report does not establish M8 acceptance.

## Exact target and execution

The adapter consumes the actual retained binary64 amounts, direct horizons,
accrued amount and positive face. It uses the existing compounding selector,
including its RN(1/m) hybrid switch. It performs no calendar query, plan clone
or observed valuation, and allocates no present-value vector.

Nonfinite quotes and global periodic floors precede frame initialization.
Original-order exact Simple checks include zero coefficients and complete
before any valid-domain unresolved outcome. Rejection counts the completed
check while retaining the rejecting original index; reduction progress is
independent. Every error retains the attempted quote bits.

Zero yield and proved unit/zero contributions use the consuming exact fold,
including explicit Clean AI and exact final100/face. General arithmetic uses
the existing bounded directed rational/log/exp operations and a complete
signed streaming sum. The final unit operand is staged as one exact raw100/face
ratio, avoiding unnecessary widening from a reciprocal interval for100/100.
There is no binary64 prefix range gate or native elementary-function premise.

Only the completed target, after basis and units, can establish an exact point
or final interval collapse. Point fitting prefers canonical Decimal, then
finite Binary64, otherwise strict outward bounds. Complete-target state is
retained through later fitting, allocation and export stops. The returned
evidence remains DECLARED and borrows the actual immutable originating plan.

The adapter and arithmetic share one borrowed scalar Frame. The mandatory
exact-point route can avoid allocation; other paths reserve the fixed24-slot
arena at most once. Entry, initialization, source, arithmetic, commits, basis,
encoding and reserved terminal work follow the adopted tariff. These source
choices are not a physical stack-peak or CPU-time guarantee; the final
whole-producer resource exit is E5.

## Current verification

Nine [actual Generic controls](../src/bonds/enclosure/generic_controls.rs)
pass in native debug and release: frozen HYBRID144 cached horizons and the
actual selector, global error precedence, admission/reduction progress,
work stops, complete-target cutpoints, signed preferred points, final
singleton Decimal/NoFit/Binary laws and independent threaded budgets.
Strict all-target Clippy, format, private documentation and diff checks pass.

The [public financial consumer](../tests/bonds_enclosure_generic.rs) uses the
unchanged standard FS1 reader and frozen E2 inputs. Its required partitions
are S8960, P38400, Q13161, Generic B2's28 basis rows, H8 and HYBRID144,
with all32 configurations and all five precisions represented. It requires
ordinary finite containment and the predeclared useful-width subset; stress
unresolved outcomes remain distinct from finite comparison passes.

Full public financial **release execution passes** all these partitions,
including the all-precision crossings, with unchanged source during the run.
Full debug execution is in progress. Earlier quota-interrupted handles had
no recoverable terminal result and are not counted as passed; the resumed
runs preserve output and terminal metadata on disk.

The first extracted-package run compiled and passed eight controls, but the
HYBRID unit depended on a repository-only fixture file and failed with
`NotFound`. The repaired unit mechanically consumes the same144 cases as
self-contained Rust literals. All nine controls now pass debug/release and
from the extracted package; projection reproduction and strict Clippy pass.
The production financial binary does not compile that cfg(test)-only module.

This implementation unit delivers the actual Generic producer and passing
release financial/packaged unit results. The already running full financial
debug check remains required; it is not inferred from release or counted as
passed. E3 remains open for the complete four-model delivery and outstanding
checks. No frozen reference or expected value is changed.
