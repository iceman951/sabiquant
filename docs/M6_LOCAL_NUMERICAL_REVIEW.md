# Local M6 numerical review and corrective work

This records local engineering review on 2026-10-12, under the owner's
instruction to continue locally with broad tests. It is not an individual
owner sign-off, formal M5/M6 acceptance or native cross-platform evidence.
Review observations must be distinguished from subsequent fixes and checks.

## Fixed findings

| Finding | Resulting behavior | Evidence |
|---|---|---|
| Natural cubic stored physical curvature before multiplying by a large interval squared | Both cubic families prepare dimensionless widths/secants/coefficients and restore ordinate scale during evaluation. Physical derivatives need not be representable; required normalized quantities must retain supported precision | Exact three-node constraints, large/small uniform and nonuniform scales; the demonstrated 33⅓% error is now an exact-dyadic regression; `tests/math_interpolation.rs` |
| Half-distance scaling erased a representable tiny affine contribution | Use full finite centered distance and cancel identical factors before exponent scaling; extreme signed two-node identities preserve MIN_SUB | `x/y=[-MAX,MAX]`, queried at ±MIN_SUB, ±MIN_NORMAL, ±1 and ±MAX/2; all signed two-node methods |
| An affine extrapolation contribution overflowed before recoverable cancellation | Retry with scaled ordinates; no clipping or guessed result | `[0,-MAX],[1,-MAX/2]` extended to four recovers MAX |
| Summation oracle accepted any seven intermediate errors on finite-final-sum rows | Require exact named row, index, quantity and infinity sign; committed generator independently proves each overflowing exact prefix | 105,307 rational rows, seven named prefix proofs and contextual Rust checks |
| Only partial compensated overflow was directly tested | Verify final correction overflow separately on both signs | `[MAX,2^969,2^969]` keeps each partial sum finite, then fails the final corrected sum; source/error context is checked |
| Quantile approximation could straddle mathematical affine overflow despite a finite fused result | Guard distance to the actual overflow midpoint, propagating the measured standard-quantile allowance; preserve safe MAX means with tiny scale and the exact median | Quarter-scaled product/residual plus exact reduction; 19,946 precision-checked normal rows include 430 conditional inverse boundary cases |

The summation capacity/carry/borrow/rounding design was independently reviewed
by the solver agent before its execution quota ended; no code defect was found
there. Its concrete diagnostic/coverage findings above were addressed by the
root agent. The engineering review remains local; formal acceptance is open.

## Numerical scope

Exact stored interpolation knots, explicit identity paths and exact summation
are distinct from composite spline evaluation. For example, normalizing by a
neighboring ordinate difference of eleven makes even a dyadic mathematical
PCHIP golden undergo composite rounding. Its old bitwise assertion was too
strong; it now uses the already scoped eight-ULP selected-example check. The
independent polynomial oracle's conditioned absolute scale was not increased.
ULP distance to an exact zero can be enormous while absolute error remains
small; it supplies no useful relative-accuracy claim.

The exact summation accumulator has enough bits for every finite binary64
element and any slice length, separately for both signs. It rounds once after
integer cancellation. Compensated summation has a different contract: supplied
order and checked intermediates, with no universal correctly-rounded claim.
Ordinary reductions are calibrated against exact sums; intermediate/final
overflow diagnostics are checked explicitly rather than absorbed into a
numerical tolerance.

Solver stopping decisions use the pinned rounded operational tolerance.
97,851 exact Fraction cases distinguish that rule from real interval
arithmetic and validate its stated rounding allowances. A callback-zero result
does not certify the mathematical root; tiny scaled callbacks can themselves
underflow. Independent 120/200-digit polynomial roots cover 251 binary scales,
translation and negative intervals across all three solvers.

Normal identities, tails and inverse need independent precision/conditioning
evidence. The temporary crossover experiment is not a precision certificate.
The committed oracle checks center/continued-fraction overlap, alternating tail
convergent separation and two precisions. Density and affine quantile boundary
tiers must forbid a finite result for true rounded overflow while checking
every allowed finite observation against its numerical budget.

## Remaining review gates

Normal boundary regeneration and measurements, complete debug/release,
coverage, strict lint/docs, cross-target compile and packaged-source checks
are recorded in [M6_NUMERICAL_VALIDATION](M6_NUMERICAL_VALIDATION.md).
The subsequent traceability findings were addressed with mesh-conditioned
PCHIP refinement and an explicit fused-operation-order table, without changing
accuracy allowances. Native Linux/Windows execution, source/maintenance gates,
formal owner review and publication remain pending. A local passing check must
not be reported as one of those approvals or remote CI results.

## Independent continuation review — solver agent `/root/period`

This appended section is an independent read-only review on 2026-10-12,
continued after the earlier execution interruption. The reviewer changed no
production source, tests, generators, manifest or Git state. It considers the
subsequently repaired sum diagnostics and the normalized-cubic/normal-boundary
snapshot separately from the earlier findings. It is an engineering review,
not owner acceptance or a cross-platform attestation.

### Exact summation capacity and rounding proof

No defect was found in `src/math/sum.rs`. The following checks justify the
indexing and arithmetic invariants rather than relying on a million-element
test to represent every possible slice length.

1. A finite normal value with stored exponent `e` is an integer significand
   of at most 53 bits shifted by `e-1`, in units of `2^-1074`; subnormals use
   their unhidden fraction with shift zero. Thus the largest input magnitude
   occupies 2,098 bits, with highest possible set-bit index 2,097. The general
   representation and fixed-point principle agree with
   [Neal (2015), printed pp. 3–4](https://www.cs.toronto.edu/~radford/ftp/xsum.pdf).
2. For `B=usize::BITS`, a slice has fewer than `2^B` elements. Each separate
   unsigned sign accumulator is therefore strictly below `2^(2098+B)`.
   `ceil((2098+B)/64)` limbs suffice, including all partial positive additions.
   On the current 64-bit target this is 34 limbs, 2,176 available bits for
   a 2,162-bit bound, and 544 bytes across both arrays. The proof also covers
   a 32-bit `usize`; native 32-bit execution is not claimed here.
3. Input decomposition's shift is at most 2,045. A 53-bit significand shifted
   within a limb uses at most 116 bits, safely inside `u128`; direct low/high
   insertion addresses at most limbs 31 and 32. Nonnegative low insertion
   cannot exceed the complete subsequent addition. Carry propagation cannot
   leave the array under the capacity bound.
4. Reversed limb comparison is unsigned lexicographic comparison of the
   complete integers. It selects the larger magnitude before subtraction.
   Subtracting each word and the incoming borrow, then combining the two
   borrow flags with OR, implements radix-`2^64` subtraction. Its final borrow
   is false by the ordering invariant. Equal accumulators return canonical
   positive zero before this path.
5. A highest bit below 52 is already an exactly representable subnormal.
   Highest bit 52 produces the minimum-normal exponent without discarding
   bits. Otherwise the algorithm extracts exactly 53 leading bits, tests the
   next bit and every lower sticky bit, and increments only for greater than
   half or an odd halfway significand. A carry into bit 53 renormalizes the
   exponent. All extraction/sticky indices are bounded; a highest bit above
   2,097 rejects before narrower extraction. Exponent 2,047 after rounding
   rejects the overflow midpoint, including its tie, on the correct sign.
   A nonzero integer result cannot round below MIN_SUB, so no separate
   zero-underflow classification is missing.

The all-powers-plus-cancellation fixture is meaningful long-chain coverage:
adding one MIN_SUB to an accumulator whose lower 2,098 bits are set propagates
carry across 33 limbs, then subtraction propagates borrow back through them.
Every stored exponent and halfway perturbation exercises offset, guard,
sticky and renormalization transitions. Maximum slice length itself remains
covered by the capacity proof, not an impractical allocation experiment.
The private zero-magnitude fallback in `round_binary64` is defensive and is
unreachable from the public strict-magnitude subtraction path; fabricating
public inputs to inflate coverage of that branch is unwarranted.

### Compensation and independent oracle

The two compensated branches match method IV, equation IV.1 on printed p. 40
of the visually inspected
[Neumaier (1974) scan](https://arnold-neumaier.at/scan/01.pdf).
Its order-sensitive compensation is correctly distinguished from exact
summation: the source promises neither correct rounding nor permutation
invariance. The fixture's compensated allowance remains an empirical
conditioned check, not a universal error certificate or permission to accept
an unexpected arithmetic error.

The repaired consumer now pins all seven finite-final-sum prefix failures by
row, index, quantity and infinity sign. The generator proves the corresponding
first overflowing exact prefixes with `Fraction`; it does not run a copied
Neumaier or Rust limb loop. The reviewer also independently executed
`[±MAX, ±2^969, ±2^969]` under `/private/tmp`: each partial sum is finite,
the corrected final result fails with `index=None`, quantity
`compensated sum`, and the proper infinity sign. The subsequent committed
regression agrees.

Python's [Fraction documentation](https://docs.python.org/3.9/library/fractions.html)
confirms exact conversion of supplied binary64 values. The runtime inspection
found `Fraction.__float__` inherited from `numbers.Rational.__float__`; the
[CPython 3.9.6 integer true-division implementation](https://github.com/python/cpython/blob/v3.9.6/Objects/longobject.c#L3590-L3680)
describes correctly rounded conversion and adjusts subnormal handling to avoid
double rounding. This supports the independent oracle construction on the
attested local Python/IEEE target; it imports no project accumulator.

After the fixes, the reviewer reran seven `math_sums` tests and eight
`public_api` tests successfully, and `sum_oracles.py --check` reproduced
105,307 records. Reviewed sum source SHA-256:
`c0b794bf43276f17bcfd080e71aa3e70ca996d0750e364ef1622de1e6907d935`.
Generator SHA-256 at this checkpoint:
`9875efb1be64ddccba3fc454dc809c91fbdb9fe7507d06fbc2d25bce367af326`.

### Integration, limits and closeout items

There are 49 public struct/enum declarations: 35 prior values and 14 M6
values. All 14 math values appear in `tests/public_api.rs`. That file checks
41 values; contextual day-count, ICMA and TARGET tests cover the remaining
eight. All four math error domains are wrapped by the root error, with
matching `From`, transparent display/source and delegated stable kinds.
The exhaustive root test samples all 58 distinct nested error variants;
structured numeric causes are retained where present. None of the new math
definitions introduces a global cache, hidden configuration or persistent
callback object.

Solver reports remain conditional on continuity and reliable callbacks;
FunctionZero is an observed value, not a certified mathematical zero.
Operational rounded bracket tolerance and the shared check-before-call budget
are explicit. Compensation is supplied-order and bounded by input length;
exact summation has fixed storage. Cubics now describe normalized support
restrictions and avoid claiming a mathematical overflow from an unsupported
grid. Normal probability/density zero is meaningful underflow, while its log
results and overflow guards have separate checked domains. Elementary
precision and normal boundary allowances remain locally measured; these
declarations do not prove Rust `libm` precision on every platform.

The following are useful closeout items, not newly discovered arithmetic
defects:

- Exact-sum overflow rows currently assert kind/index but do not attest the
  `exact sum` quantity and positive/negative infinity source per row. A
  sign-bearing Fraction fixture field or explicit signed typed regressions
  would protect that diagnostic contract. This gap was sent to the root.
- At the reviewed documentation snapshot, README/changelog omitted M6,
  readiness described implemented reductions/oracles in future tense, and
  the architecture extension table still predicted solver/interpolator
  traits despite the adopted functions/value design. These are pending
  closeout consistency corrections, not evidence that an owner approved M6.
- Correction-accumulator overflow can require extremely long reductions;
  a feasible test set does not establish every such branch. Report uncovered
  defensive paths and proof-supported exclusions honestly instead of
  weakening tolerances or manufacturing implementation-mirroring tests.

The reviewer does not re-open the earlier affine/curvature defects as current
failures: their subsequent fixes and large independent oracle checks were
reported by the owning agents. This section independently validates the
sum/integration snapshot and its precise claims; aggregate coverage, native
Linux/Windows execution, package revalidation and final owner/release gates
remain the responsibility of the final milestone record.

**Subsequent closeout verification by the same reviewer:** the exact-overflow
diagnostic gap above is now closed. The generator's `overflow_sign` column
comes from the independently rounded Fraction result, and the consumer checks
`ResultNotFinite`, quantity `exact sum`, `index=None` and the corresponding
positive/negative infinity source on every error row. The reviewer re-read
those paths, reran all seven sum tests successfully and reran generator
`--check` successfully on the current 105,307 records. No concrete unresolved
arithmetic defect was found in the reviewed sum or root-error integration.
The root owns the remaining README/changelog/readiness/architecture closeout;
those snapshot observations do not require an additional source change here.
