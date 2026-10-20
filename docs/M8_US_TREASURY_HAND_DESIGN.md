# Blind Treasury numerical hand controls

Authored before Treasury production against the adopted complete
[main contract](M8_US_TREASURY_CONTRACT.md),
[cash contract](M8_US_TREASURY_CASH_CONTRACT.md) and
[76-family test plan](M8_US_TREASURY_TEST_DESIGN.md). The new source is
[bonds_treasury_numeric_hand.rs](../validation/hand/bonds_treasury_numeric_hand.rs):
28 proposed public integration tests, currently outside Cargo's test tree.
The source has been parsed/formatted by Rust1.99 rustfmt. **It has not been
compiled or run.** Python derivations below passed; these are independent
expectation/precondition checks, not library test results or coverage.

The author did not read Treasury production or the other new Treasury
oracle producers' expected corpora. Existing public Date, DecimalAmount,
BondPrice and solver interfaces, together with existing gilt hand-test API
scaffolding, were consulted. Treasury expectations were derived from the
adopted rational model and independently implemented Fraction arithmetic.
No QuantLib implementation or reference result supplies these expectations.

## Inputs, provenance and frozen bytes

The adopted inputs at authorship are:

| File | SHA-256 |
|---|---|
| `M8_US_TREASURY_CONTRACT.md` | `50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93` |
| `M8_US_TREASURY_CASH_CONTRACT.md` | `25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585` |
| `M8_US_TREASURY_TEST_DESIGN.md` | `5e91934b538d03297659674cc9f10f51f91b1d35dd21437472a7901d157e0629` |
| New hand source, rustfmt output | `09676aff8114429ebfd28271830c206e2d40a895ff0b7b90817f0cc230371cab` |

Independent local derivation artifacts are retained under
`/private/tmp/sabiquant-treasury-blind-hand-v1/`. They import Python stdlib
only, no project module or library. Evidence JSON was created with exclusive
creation and strict finite JSON encoding. v1 evidence was preserved when
the additive short/inverse/direct-rounding checks were added.

| Artifact | SHA-256 | Executed scope |
|---|---|---|
| `check.py` | `b9ea698d8ae16b9f64e524507b0dc2013515fe85b8731e80f417926c39e3ca2a` | Exact dates, p256 directed rational operations, extended53 factors/complete risk terms, direct IEEE RNE, 17 named controls |
| `evidence.json` | `855c5aa0bdaaa53eefe75d0c402fe71455b17e625e3ba8f94f0fe49d67955b02` | Python3.9.6, 2026-10-20T08:20:43 UTC; terminal exit0 |
| `extra_checks.py` | `ad557ed31bd5cb9a8999a1d5cf289597c1c9c813a1dc291d1285e724cf266a27` | Additive short-first, negative-clean and target-driven double-rounding search; uses v1 helper definitions without executing its main |
| `extra-evidence.json` | `fe62bf203ab25d46fcded0ad0994e397833c3e53206c7f487c56445287a7d16b` | Python3.9.6, 2026-10-20T08:26:24 UTC; terminal exit0; four named records including the search recipe |

The JSON timestamps, helper/self hashes and exact date/quote bits are
retained in those artifacts. They are local evidence, not packaged fixture
files. The Rust source itself retains the discriminating constants and their
derivations; future reproducibility must preserve the scripts/evidence or
adopt them separately. A script rerun must use a new output directory or
name, rather than overwrite this evidence.

## Independent arithmetic and domain checks

The Python model represents binary64 input y exactly as Fraction. It forms
`b=1+y/2`, `q=1+r*y/(2*s)` without an early floating half or rounded alpha.
Its p256 enclosures use exact numerator/denominator quotient and remainder,
outward rounding at every operation and the specified independent left to
right power tree. The lower reciprocal endpoint is rounded to extended53;
the exact source coefficient times this observation is rounded directly to
IEEE. The integer IEEE routine is also checked against Python's independent
Fraction-to-float conversion, including signed zeros and overflow.

Risk uses the accepted binary PVs, correctly rounded q/delta observations,
one complete Fraction for each derivative/moment contribution, extended53
term rounding, exact retained sums and direct normalization. It never
obtains a normalized value from a raw IEEE getter. This implements the
prescribed observation to freeze E0 hand constants; it is not a proof that
an observed factor is the ideal factor's correct rounding.

Separate exact rational ideal derivatives corroborate the numerical anchors.
Decimal90 and Decimal150 independently check the .05 half-stub ideal
price/first derivative/second derivative/moment, and corroborate the raw
overflow witness's magnitude classification. These two fixed precisions
are corroboration, not interval containment proofs or runtime refinement.

Every modelled financial definition checks nonnegative coupon within
`i128::MAX`, valid D<=I<F<=M, complete minimal coverage, direct maturity
six-month recurrence, accepted first-period role, all half-year lengths in
181..184, and I<=S<M. Helpers use day7 or day15 with explicit Ignore; no
ambiguous month-end rolling is needed. Fractional coordinates are derived
from civil dates rather than assigned independently.

## Exact anchors and discriminators

The common regular cycle starts D=I=2024-12-07; its first coupon is
2025-06-07. At S=2025-03-08, r/s=91/182=1/2. With C=5 and one payment,
`D(y)=102.5/(1+y/4)` and the published/raw AI at S both equal1.25.
At y=0 the tuple `(dirty,D',D'',M,Modified,Macaulay,Convexity)` is exactly
`(102.5,-25.625,12.8125,25.625,1/4,1/4,1/8)`. The second derivative is the
fractional-simple value; a power-stub interpretation would change it.

For y=.05 the exact decimal-y ideal has q=81/80,
Modified=20/81, Convexity=800/6561 and Macaulay=1/4, whereas
`b*Modified=41/162`. The binary input is the exact represented .05,
not exact1/20. The source freezes its prescribed observation bits and also
compares against these rational ideals with a deliberately bounded local
allowance `8*EPSILON*abs(ideal)`. N3/N5, final reference rounding and the
input perturbation below2^-58 all fit that allowance here. It is confined
to this well-conditioned normal fixture; it is not a global ULP profile.

Additional exact zero/power anchors are:

| Definition / settlement | dirty | D' | D'' | M | Modified / Macaulay | Convexity |
|---|---:|---:|---:|---:|---|---|
| Regular C6, two payments, common half-period S, y0 | 106 | -78 | 90.5 | 78 | both39/53 | 181/212 |
| Short D=I=2025-03-08, F2025-06-07, M2025-12-07, C6, S=D, y0 | 104.5 | -77.625 | 90.3125 | 77.625 | both621/836 | 1445/1672 |
| Long same D, F2025-12-07, M2026-06-07, C6, S=D, y0 | 107.5 | -132.125 | 222.8125 | 132.125 | both1057/860 | 713/344 |
| Regular C0, three periods, S=D, y2 | 12.5 | -9.375 | 9.375 | 18.75 | 3/4 and3/2 | 3/4 |

The long first coupon pays at original Q2, hence j1 before skipped Q1;
the final Q3 retains j2. It must not be relabelled j0/j1. Raw first cash is
4.5, including its half initial period and full second period. At skipped
2025-06-07 accrued remains1.5; actual first payment2025-12-07 resets it.
The explicit I=2025-07-01 variant remains admitted and still accrues from
D: raw231/122, published1.893443 at I. That is the contract's stated local
dated-date interpretation, not a claim about announcement eligibility.

The policy discriminator uses C=23e-7, D=I=2025-07-15,
M=F=2026-01-15 and S=2025-10-03: 80 elapsed days out of184. Raw AI is
exactly5e-7 per100. Exact daily per$1,000 is62.5e-9; daily9 even/away
select62e-9/63e-9. Published total5 AI is0/1e-6 per100 respectively,
under either daily basis. All four policies must retain identical dirty
and six risk observations while reporting their own policy/published AI.

The separate published-target witness uses the common C5 note at
2024-12-08: rawAI=5/364, publishedAI=.013736. Clean target bits
`40599f1ef30a4e38` plus cached published AI resolve exactly to dirty102.5;
substituting raw AI gives bits`4059a000011b2f46`, a distinct target.
Another exact target is clean-.25 at y406 with AI1.25 and dirty1; negative
clean alone must not trigger nonpositive-target rejection.

## Financially admitted range controls

All following entries are actual public definitions; no forged report,
uncoupled q/delta, C=2^127 or impossible calendar is used.

| Control | Inputs and independently derived outcome |
|---|---|
| Zero source amounts retain extended factors | C0, regular21, S=D, y=next_up(-2): zero Q20 observes an extended factor2^1060; first nonzero failure is principal PV at absolute Q21, positive infinity |
| First nonzero PV underflow | C1e-38, regular2, S=D, y=MAX: Q1 source coupon rounds to+0 and fails before later principal |
| Full-C source PV overflow | C=i128::MAX, regular21, S=D, y=next_up(-2): first failure is Q17, not a source amount/IEEE factor gate |
| Reduction after finite PVs | C2^126, 2162 periods from2000-01-15 to3081-01-15, S=D, y=-.5: each PV finite, exact dirty reduction overflows |
| Raw getters overflow independently | C5, D=I2025-07-15, M2035-07-15, S2025-07-16, y=next_up(-2): finite dirty bits`7fc26affffffff96`; D'=-infinity, D''/M=+infinity; all three normalized getters finite |
| Signed raw underflow, subnormal Modified | Common one-payment C0, half-period S, y=MAX: D'=-0 error, D''=+0 error, Convexity=+0 error; Modified bits`0004000000000000`, Macaulay1/4; rawM remains finite normal |
| MIN_SUB dirty cancellation | C0, regular2, S=D, y bits`61c8000000000000` (=3*2^540): dirty and rawM both MIN_SUB, raw derivatives underflow, Modified bits`1e25555555555555`, Macaulay1, Convexity positive-zero error |
| Admitted subnormal Convexity | C0, one full period, S=D, y=2^537: Convexity2*MIN_SUB even though rawD'' underflows; rawD'=-200*MIN_SUB, Macaulay1/2 |

The direct-versus-double-rounding public witness has C0, two regular
periods, S=D and y bits`61af60eab9a5d3a2`. Decimal180 supplied a quote
candidate around the target6.5*MIN_SUB; Fraction checked its exact bits and
the prescribed p256 result. The accepted extended factor is exactly
`1170935903116329/2^1128`. Thus the source product in MIN_SUB units is
`13/2+2^-52`. Direct IEEE RNE gives7*MIN_SUB. An extra extended53 product
rounding erases the positive excess and the subsequent6.5 tie gives even6.
Only the final PV is positive, so public dirty itself discriminates these
algorithms. The hand source also checks rawM7*MIN_SUB and Macaulay1.

No normalized-only Convexity-underflow case is asserted: the Treasury
contract proves that Convexity underflow also forces rawD'' underflow.
Raw moment zero, Modified zero/infinity and Macaulay zero/infinity are
likewise not manufactured. A finite subnormal Modified or Convexity is
accepted; their ideal accuracy needs the absolute/m=1 profile, not a normal
relative-error assumption.

## Inverse and cached-payload obligations

Independent half-stub targets102.5,51.25,164,82 imply roots0,4,-1.5,1
from `y=4*(102.5/T-1)`, without evaluating the future library to choose
expectations. On the four listed brackets, ideal dirty<=171 and slope
magnitude>4. N3 gives uniform absolute observation error below2^-40.
Each inverse assertion includes the returned coordinate bracket width,
cached amount residual divided by4 and this model allowance. A planted
round trip alone is not treated as inverse accuracy evidence.

The suite checks nonpositive resolved targets before invalid quotes with
zero evaluations, lower exact-zero success with one callback and an invalid
unqueried upper principal PV, lower/upper error stages and counts including
failed calls, cap1 preventing the upper invocation, and a two-call
same-sign NotBracketed result. Report getters repeatedly retain the same
cached payload/counts. This verifies their observable contract once run;
the absence of hidden work also requires production inspection/instrumentation.
No interior-callback, precision-exhaustion or scale2^970 claim is made here.

The observed-image discriminator is the common C5 note at S2025-06-06,
r1/s182 and y=next_up(-2). Exact ideal supremum is18655/181. Correctly
rounded ideal PV has bits`4059c43e3b673fa5`, while the prescribed factor/PV
observation gives`4059c43e3b673fa6`, above that supremum. The latter exact
dirty target must succeed at the lower endpoint in one callback; an ideal
upper-image precheck would reject a valid observed round trip.

## Family mapping and remaining work

The mapping identifies intended assertions, not completed families. In
particular a family containing an untested clause remains partly open.

| Hand source test numbers, in source order | Semantic families | Intended assertions |
|---|---|---|
| 1–4 | UST-25/31/37/39/40 | Exact normal anchors, all six risk views, alpha1 ordinary identity and simple-stub distinction |
| 5–7 | UST-05/09/10/19/39 | Short/long raw amounts, missing Q1 coordinates, AI resets, D/I distinction and I>Q1 |
| 8–10 | UST-03/14/20/21/29/38 | Retained policy, raw/published AI, identical raw numerics, tiny quotes and quote errors |
| 11–14 | UST-33/35/36 | Nonzero PV admission/order and later dirty reduction |
| 15–19 | UST-32/33/39/43–48 | Direct PV rounding, signed getter failures, raw moment overflow/subnormal and direct normalized cancellation |
| 20–23 | UST-38/51/56 | Published clean shift, owned/cached metadata, nonpositive and negative-clean target distinctions |
| 24–25 | UST-52/53/54 | Lower-first callback order, endpoint failures, cap before invocation and NotBracketed counts |
| 26–27 | UST-57/58 | Independent conditioned inverses and observed supremum overshoot |
| 28 | UST-71 | Owned reports after definition/plan drop and scoped immutable plan reuse |

General independent source/cash, prescribed primitive, ideal all-six-risk
and inverse corpora remain separate obligations. This hand tranche does not
claim all source tables/policies/scales, complete p256 operation coverage,
full error census, all capacity bounds, cash/evidence semantics, successful
mutant execution, native debug/release, Clippy/docs, coverage or package
verification. Rustfmt parse success cannot establish API resolution or
numerical correctness. After the pre-production artifacts are frozen,
implementation must expose the adopted API and run these unchanged
expectations through a normal integration consumer, then perform the full
final-byte verification and record any independently justified correction.
