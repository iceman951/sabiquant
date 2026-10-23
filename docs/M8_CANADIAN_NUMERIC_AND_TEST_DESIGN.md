# Numeric, resource and full semantic test proposal

Unadopted companion to the [complete local design](M8_CANADIAN_LOCAL_DESIGN.md),
recorded 2026-10-23 Bangkok. The mathematical/source/API/cash recommendations cover the
whole fixed-coupon model. This companion states proposed observations and pre-contract
gates; ideal equations alone do not select a bounded IEEE implementation. Root promoted
immutable private v3 SHA256
`ea2306ac4789d177745bd22dd2afa48195396d4eb982103298c0b1d16c9a85eb`, with editorial
formatting/links and the exact C0 inequality correction in N8 (`<=`, rather than `<`).
The [independent numerical review](M8_CANADIAN_NUMERICAL_DESIGN_REVIEW.md) and [research
validation](M8_CANADIAN_RESEARCH_VALIDATION.md) remain separate evidence. Later
[API closure](M8_CANADIAN_API_REVIEW.md), [conditional compound profile](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md)
and [exact MM range closure](M8_CANADIAN_MM_RANGE_REVIEW.md) are integrated below.
The old10u/16u review target is superseded by the derived per-frequency bounds.
The [bounded guard design](M8_CANADIAN_COMPOUND_GUARD_DESIGN.md) supplies the
exact threshold comparison and error behavior. The
[complete cash-resource review](M8_CANADIAN_CASH_RESOURCE_REVIEW.md) and
[independent integration challenge](M8_CANADIAN_RESOURCE_INTEGRATION_REVIEW.md)
close the full proposed arithmetic/storage routes. Complete contract adoption and
independent preproduction remain required before implementation.

## N1. Exact prepared data and unconditional metadata bounds

Retain original actual-coupon ordinal on each bundle, not compressed position or a
synthetic maturity quasi index. An admitted future contractual date is strictly after S;
final MM payment T>=M>S. Full Q count<=20,000 and minimal coverage imply fewer
than20,000 complete nominal periods between any admitted endpoints. Each retained actual
coupon is distinct and uses either an on-Q nonterminal date or the one terminal M, so
actual count<=Q.len()-1 (the public20,000 cap is conservative). Skipped odd virtual
dates do not become coupons.

Each chi difference is an integer plus at most two fractions with denominators<=184, so
its direct common denominator is<=184^2<2^16 and numerator<20,000*184^2<2^30. These
deliberately conservative bounds are primitive supersets, not claims of joint dated
attainment. For source rationals the declared160-bit numerator/148-bit denominator
allowances include C and decimal scale<=38. Group terminal coupon and100 exactly before
any PV observation.

AI observations can be prepared eagerly WITHOUT a hidden f64 admission restriction on C.
If nonzero, C>=10^-38>2^-127. Nonzero valuation accrual has at least one day, thus
A>=C/(12*184)>2^-139; nonzero Canadian365 proper-fragment AI>=C/365>2^-136, completed
C/f is larger, and monthly AI=A. All source terms are nonnegative; each actual accrual
interval spans<20,000 nominal periods, so A,AI<2^143. Therefore every nonzero exact A/AI
rounds to a finite NORMAL binary64; exact zero alone gives+0. This supports
independently cached `A_hat=RN_even(A)` and `AI_hat=RN_even(AI)` across the entire input
domain, including C/f scale39/40/nonterminating cases. It does not assert that an optional
DecimalAmount view is representable.

## N2. Recommended deterministic observations

Use std-only bounded unsigned helpers, separate signs/exponents, no unsafe,
panic-on-input, arbitrary-precision retry, global rounding mode or I/O. Existing
generic/gilt/Treasury algorithms remain unchanged. The following is the recommended NEW
named-model observation contract for later adoption. The independent conditional
profile and bounded range-guard proofs are now integrated below; the complete contract,
threshold literals and independent preproduction evidence must still be frozen before
production begins.

### Compound factor

For finite y require exact y>-f and canonicalize either zero. Compute one shared scaled
logarithm per evaluate/risk/solve-callback, after quote validation and its required PV
workspace reservation, before bundle traversal:

```text
if y == 0: g=0                         # no elementary call
else if abs(y) < 2^-50: g=y            # explicitly qualified tiny identity
else if y <= -f/2:
    delta_exact = exact(f)+exact(y)
    b_hat = RN_even(delta_exact/f)
    ell_hat = observe_native_ln(b_hat)
    g = RN_even(f * exact(ell_hat))
else:
    x_hat = RN_even(y/f)
    ell_hat = observe_native_ln_1p(x_hat)
    g = RN_even(f * exact(ell_hat))
require finite g
for original coupon i:
    z_i=RN_even(-exact(g)*lambda_i/f)   # one exact-rational-to-IEEE rounding
    if z_i==0: discount_i=1
    else: apply new range-boundary guard, then observe exp(z_i) once
    require finite positive discount_i
```

Here ln/ln1p/exp are actual native elementary observations, not a requirement that
native primitives return the unique correctly-rounded ideal logarithm/exponential.
RN_even binds the explicitly represented-input arithmetic stages. The separate2u
hypotheses bound elementary error conditionally; they do not fix one portable libm
bit pattern. Numerical fixtures must distinguish exact represented-stage controls
from certified elementary intervals and informative public composed comparisons.

For f2/4 the division/multiplication by f is power-of-two scaling in ordinary paths; f12
introduces an additional argument/result rounding and must NOT inherit the old gilt
error profile unchanged. The tiny path prevents MIN_SUB/f from vanishing before
multiplication by a long exact lambda. It is a declared approximation, with exact
analytic remainder

`|f*ln(1+y/f)-y| <= y^2/[2*f*(1-|y|/f)]`.

Near the lower floors, exact Fraction controls give `f+next_up(-f)` equal to2^-52
(f2),2^-51 (f4),2^-49 (f12). The middle value is NOT2^-50. Both near-floor branch
arguments stay positive; f12's division by12 is explicitly rounded. At MAX, RN(f+y)=MAX
for these f; do not reject a valid quote because an imagined exact f+y is slightly
larger than MAX. At MIN_SUB, y/12 is zero but the tiny g=y path preserves the signal
until final exponent admission. Controls are retained in `control-evidence.json` with24
floor/admission rows; they are Python proposal checks, not Rust runtime claims.

The expected ideal exponent has |lambda*ln(1+y/f)|<20,000*710<2^24 over finite valid y.
This follows from exact-input bounds; it is not a theorem that unspecified elementary
observations meet a profile. Validate all observed nonfinite/domain values and preserve
typed Rate/Value causes. A shared-log failure has index=None; per-discount/PV errors
identify the original coupon ordinal.

The [completed native profile and independent derivation](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md)
replace the former unproved10u/16u target. With u=2^-53, assume the declared ln/ln1p
primitive relative error <=2u. Then exact binary scaling for f2/4 gives
`|g-f*ln(1+y/f)| <=2u*|f*ln(1+y/f)|`; f12 argument/final-product rounding gives
6u instead. The proof is conditional on those explicit primitive hypotheses;
finite native agreement does not prove universal libm accuracy or foreign runtime.

For exact lambda>0, q=-g*lambda/f and z=RN_even(q), let H=2^-1075. The non-tiny
exponent radius is exactly

`R=(beta*abs(z)+H)/(1-beta)`, beta=4u for f2/4 or8u for f12.

For tiny y, use the tighter exact
`R=abs(z-q)+lambda*y^2/[2*f^2*(1-abs(y)/f)]`; signed-zero input has R=0.
The ideal exponent lies in [z-R,z+R]. Certified outward256-bit dyadic intervals
for `Z0=-1075*ln(2)` and `Zinf=ln(2^1024-2^970)` are derived from exact rational
series in the linked review. The [complete bounded guard](M8_CANADIAN_COMPOUND_GUARD_DESIGN.md)
uses exact signed endpoint numerators with GuardWide<20>, not an inward-rounded R.
For non-tiny finite0<abs(z)<2^24 set E=2^1075, A=abs(z)*E, B=2^53-k,
C=A*B and N=k*A+2^53, where k=4/8. The common positive denominator is B*E.
For positive z the signed endpoints are C-N,C+N; for negative z they are
-(C+N),N-C. Compare them to each signed certified threshold numerator T by
T*B*2^819, cancelling its2^256 denominator first. Endpoint magnitudes<2^1153
and threshold comparands<2^1138 fit20 words. Reconstruct five-limb little-endian
threshold constants by reverse-order Horner steps; do not parse or allocate.

A CLOSED intersection with either threshold box gives indexed
RateError::NumericalBoundary before exp; touching counts. A disjoint interval,
including one entirely beyond a threshold, proceeds to exactly one actual exp and
its positive-finite validation. This preserves actual zero/inf Value causes rather
than inventing observations. Tiny quotes bypass the wide guard because their full
error interval lies inside(-2^-34,2^-34); their nonzero analytic error is retained.
Defensive finite abs(z)>=2^24 also bypasses construction: R<abs(z)/2 puts the whole
interval beyond both boxes. No elementary-profile hypothesis is used as an
unchecked capacity assumption. Zero-exponent identity still precedes exp.

The guard has at most12 logical20-word slots (1,920bytes), no heap, no division,
no precision retry. Its proof and actual7,374-row exact replay,36 capacity controls,
10 closed-touch predicates and72 tiny controls are in the linked review. On those
existing records the proposed guard would refuse63 rows before exp, leaving7,291
exp calls and20 identities; those are simulated future flow counts, distinct from
the actual native measurement's7,354 exp calls. Future Rust must independently
verify the helper lifetimes and original-ordinal errors.

For an observed positive finite exp result e, the separate conditional budget is
`abs(e-exp(z)) <=2u*exp(z)+H`. Keep the absolute term near subnormals. All zero,
infinity and NaN outcomes still require explicit runtime validation; this
positive-finite-only hypothesis does not establish a universal no-range-error theorem.
Reference comparisons must label informative and uninformative bounds honestly.
The native run has1,207 log rows and7,374 exponent/discount rows, including20
zero-exponent identities and7,354 nonzero exp invocations per mode. The63 ambiguous
range rows remain ambiguous despite25 positive finite observations. These are private
primitive measurements, not Canadian pricing tests or an adopted financial kernel.

### MM factor

No ln/exp or rounded tau/rho inputs are needed. Let E=2^1074 and integer Y=y*E,
D=days(S,T), R_i=days(payment_i,T). Form the SHARED exact affine denominator
`d_raw=365*E+Y*D`, and `n_raw_i=365*E+Y*R_i`. Validate d_raw>0 exactly; terminal R_i=0
is not skipped. Since0<=R_i<D, positivity of every n_raw follows from the same exact
y/domain (for y<0 n_raw_i>=d_raw; for y>=0 n_raw_i>=365E). Do not infer it from
independently rounded tau/rho products.

Observe `discount_i=RN_even(n_raw_i/d_raw)` once directly. This avoids premature
overflow of y*tau. Observe even zero-coupon factors, then canonical zero PV for zero
source coefficients. The exact rational factor is decreasing for positive h_i; this is
not a compound-generic shortcut.

### Source PV and dirty reduction

For either convention, after an admitted finite positive observed factor, compute exact
source B_i times its represented binary64 factor and round DIRECTLY once to IEEE
nearest/even. Never round B_i first, and never round an extended53 PV before IEEE
admission at a subnormal tie. Nonzero-to-zero/inf is a typed indexed PV range failure;
zero source coefficient gives+0 after its factor check. Reduce all accepted PVs by
existing exact_sum, one final model-dirty observation D_hat. No source/display
quote/cash rounding or unused clean/all-in check participates.

## N3. Exact selected risk-observation choice

Ideal equations in the main proposal do NOT by themselves choose whether risk follows
source aggregate H or accepted PVs. Recommend the following explicit choice, compatible
with earlier immutable independent getters:

- Each bundle's seed W_hat_i is its SAME accepted IEEE PV from N2. If zero source, risk terms are exact zero after factor/PV checks.
- Compound: form `delta_hat=RN_even(exact(f)+exact(y))` once (positive finite on the full domain), then exact represented rational terms `-W_hat*lambda/delta_hat`, `W_hat*lambda*(lambda+1)/delta_hat^2`, `W_hat*lambda/f`. Lambda and lambda+1 remain exact rational coordinates; no rounded horizon is substituted.
- MM: use exact d=1+y*tau, n_i=1+y*rho_i from the shared integer affines, not rounded scalar denominators. Exact represented rational terms are `-W_hat*h_i/(d*n_i)`, `2*W_hat*tau*h_i/(d^2*n_i)`, and `W_hat*h_i`.
- Round each FULL term once to53 normalized significand bits nearest/even with a separately retained integer exponent. No early IEEE range admission. Preserve exact zero/sign. Sum those extended53 terms exactly on a fixed2^-8192 basis; retain3 canonical immutable signed sums. This is a selected observation model, not a claim that terms equal the exact ideal derivatives.
- Raw getters round retained sums once to IEEE. Modified, Macaulay and Convexity divide the corresponding retained sum directly by admitted positive D_hat, then round once. They do not divide rounded raw getters or use settlement-dirty/clean as denominator. No raw getter can gate another.

The positive aggregate H=sum B_i*h_i is ONLY the ideal MM derivative/strict-monotonicity
proof and an independent reference route. Replacing the selected MM observed terms by
exact source-H derivatives would change the chosen numerical observations and requires
separate review/versioning.

Exact zero results are+0. Nonzero-to-signed0/inf is an independent signed Value error
with the named getter quantity; normalization does not erase that error. Defensive
ZeroDirtyPrice is not a fabricated source case. Raw/normalized range outcomes must be
enumerated independently: a Canadian fractional compound witness C=i128MAX,
D=I2025-03-01, coupons2025-09-01/2026-03-01, S2025-08-31,y2^540 has both PVs normal, raw
second derivative normal while normalized convexity underflows. This ideal mathematical
witness will be pinned to the independent numeric review, not claimed as existing
runtime coverage.

For MM conservative exact-domain bounds sufficient for magnitude planning: Ddays<2^22,
tau<2^14. When an exact negative-y affine d<1/2, |y|>365/(2Ddays)>2^-15, hence its
represented LSB unit>=2^-67; nonzero365+y*Ddays>=2^-67 and d>2^-76. Use the safe bound
d,n_i>2^-80; both are<2^1039. With h_i>=1/365>2^-9 and accepted W_hat∈[MIN_SUB,MAX],
nonzero first/second/moment terms stay within the conservative envelopes
`(2^-3161,2^1198)`, `(2^-4208,2^1293)`, `(2^-1083,2^1038)` respectively before
extended53 rounding. These are deliberately loose valid primitive bounds, not dated
extremal attainment. With53-bit rounding and fewer20,000 terms, the243-word basis covers
them with a large margin. Compound coordinates fit the existing smaller style of bound
but require the f12/coordinate proof, not Treasury q coupling.

## N4. Inversion, observations and failure transport

Resolve all three typed target bases once as defined in the main proposal; a nonpositive
external Clean/SettlementDirty scalar is NOT itself a rejection. Only a nonpositive
RESOLVED model-dirty target rejects at zero evaluations. Use cached A_hat/AI_hat in the
one chosen shift, not exact source cash AI or a second rounded clean target.

Select residual exponent s0 if abs(T_dirty)<2^970, otherwise1. Every dirty-only callback
returns one RN_even of exact `(D_hat-T_dirty)*2^-s` and cached D_hat payload.
Existing34-word MIN_SUB-basis residual mechanics preserve cancellation, finite sign and
zero iff identical observed dirty/target. `amount_residual` independently computes
the unscaled cached difference. Both admitted D_hat and resolved T_dirty are positive
finite binary64, so their exact difference has magnitude below MAX and lies on the
MIN_SUB lattice. Its final RN cannot overflow or turn a nonzero difference into zero.
The defensive Result signature remains; this domain excludes its numeric range
failures. Signed generic-model residual exclusions must not be imported unchanged.

Reuse unchanged fallible shared Brent with exact evaluation cap/stage/point/cause/count
and derivative count0, lower-first, lower-zero one-call success, cached selected payload
and no uncounted final callback. One preallocated PV workspace is reserved before
callbacks; no per-trial date/calendar/source/cash/risk work or allocation. Every source
flow keeps its index and factor/PV checks, even a zero coupon. Both ideal branches are
strictly decreasing because all coefficients are nonnegative and final par has positive
horizon; this gives conditional mathematical uniqueness, not existence, continuity/error
certification or a guarantee that the rounded compound callback is monotone.

MM additionally has a finite positive lower image as y→+infinity when any nonterminal
positive coupon has rho>0: `D_infinity=sum(nonterminal B_i*rho_i/tau)`. A positive
resolved target at/below that limit need not be attainable at any finite y. Return
actual bracketed-solver behavior, not a new unsupported global no-root certificate. MM
may also admit y far below the compound floor: the one-day C0 terminal case y=-300 is
valid because365-300>0. Never apply y>-f as a universal precheck.

Do not import the Treasury proof excluding every interior callback failure: the compound
candidate has elementary observations/guards and a new primitive profile. Classify
genuine reachable numerical/domain failures versus bounds-excluded capacity conditions
after the actual selected kernel exists; shared synthetic fallible transport controls
are separately labelled infrastructure, not invented financial reports.

## N5. Proposed resource limits and proof obligations

All limits below are proposed acceptance constraints, not measured counters or claims
about existing Rust. No arbitrary wider fallback is allowed to silently bypass a
reviewed bound.

| Area | Proposed fixed magnitude storage | Reason / obligation |
|---|---|---|
| Cached source ratios/ordinary exact source PV inputs |5u64words320bits EACH magnitude | source N<2^160,D<2^148; direct source×53-bit factor requires at most214bits, including subnormal branch coupling |
| Pure source views/coupon/AI cash |8u64words512bits | source×nominal numerator<2^287; including quantum scale adds<127bits; normalized rounded coefficient has a tighter<2^390 bound |
| Price-evidence cash |40u64words2560bits | binary evidence MIN_SUB integer<2^2098, source denominator148bits, nominal127bits, quantum scale127bits and carry give numerator<2^2501; denominator/carry/relative alignment remain bounded |
| MM exact affine/factor |40u64words2560bits | Y<2^2098, day product+365E<2^2122; relative/direct-subnormal observation at most2176bits, no arbitrary absolute shift |
| MM selected risk full rational |112u64words7168bits | exact first numerator/denominator<2^2232/2^4244; second<2^3329/2^6366; relative53+carry at most6420bits |
| Retained risk and final normalization |3×243u64words; basis2^-8192 | common term envelope gives retained numerator<2^9501; direct getter/normalization scratch at most9502bits |
| Residual |34u64words | exact signed difference coefficient<2^2099; exponent scale stored separately |
| Compound closed-threshold guard |20u64words per magnitude, <=12slots | exact signed endpoints<2^1153 and threshold comparands<2^1138; zero/tiny/far paths bounded separately |

Pure cash rounded result bound must use actual `|R(x)|<=|x|+q`, not multiply unrelated
maximum count/Q bounds. Intermediate components may exceed public i128 and later cancel.
The40-word cash bound preserves signed evidence, continuous intervals, original kinds,
both aggregations and all7modes; changing the source common-denominator form requires
rechecking2501, not treating it as an observed magic constant.

The [constructive numerical resource review](M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md)
proves the listed numeric widths and a by-reference helper route. Decode W=s*2^e
and retain e separately. The MM selected first numerator is s*H*365*2^2148,
denominator d_raw*n_raw; second is2*s*D*H*365*2^3222 over d_raw^2*n_raw.
Do not expand W onto a universal dyadic grid. Relative53 division aligns only the
ratio; it uses53 fixed quotient-bit comparisons/subtractions and a u64 quotient,
not an extra wide quotient array or exponent-sized loop. The minimal exact
lambda is >=1/184 (equality is dated-reachable), hence still strictly>2^-8;
the review's historical first strict comparison is corrected in its integration
note. Compound full terms need at most192bits. Direct source PV and normalized IEEE subnormal branches use their
own range-conditioned alignment, not an extended53-then-IEEE double round.

Proposed Rust-level simultaneous wide-value limits:16 source-cash8word slots;32
price-cash40word slots;32 large112word numeric-risk slots. Include incoming arguments,
owned local values, helper by-value copies, aligned division
numerator/denominator/remainder/quotient/subtract work and retained endpoint images. Use
explicit scopes; do not credit optimizer last-use drops. The available numeric schedule
uses24 caller-owned112-word slots with borrowed inputs/mutable outputs. The existing
exact_sum owns TWO34-word arrays; count them explicitly as two more padded slots,
26<=32. Allocating all32slots and then adding exact_sum outside that budget is invalid.
Risk traversal inserts a shifted u64 directly into3 working243-word arrays; transfer
may hold3working+3result. Getter normalization borrows the3retained and owns at most
3mutable scratch arrays, with u64 quotient/in-place subtraction. These constraints
are part of the helper design and must be audited against actual Rust copies.

The32large112word slots plus6×243word accumulator arrays total40,336bytes of
numeric payload. Conservatively adding all12guard20word slots gives42,256bytes,
below the proposed64KiB live-numeric-payload ceiling. Guard and per-bundle risk
phases should instead reuse scoped scratch where explicit. These are available
payload/live-value schedules, not measured compiler stack frames or a consequence
of integer capacity alone. The [complete cash-resource proof](M8_CANADIAN_CASH_RESOURCE_REVIEW.md) supplies
the separate cash routes. Its pure numerator/divisor<2^414/2^409, coupled rounded
coefficient<2^390 and branch-specific binary-price/source numerator/divisor
<2^2501/2^1483 fit8/40words. Decode binary evidence as I/2^1074 and decimal evidence
on its own decimal denominator; combine one cached exact AI denominator once.
Do not multiply an uncancelled Treasury2^1074*5^38 basis by the whole source
denominator. Cash division has a separate WIDE quotient slot, unlike numeric
normalized53. The available10x8scratch+4sourcepairs route is960<=1024bytes;
24x40scratch plus the nested pure call/copies is8640<=10240bytes. Stored source
pairs in the complete candidate are gcd-reduced, zero0/1; cash evaluation copies
cached source and performs no gcd/source reconstruction. Actual future Rust
sizes/copies/heap behavior still require their implementation audit. Caller-owned vector spare
capacity, allocator overhead and caller input storage are outside this temporary
numeric-payload ceiling. Schedule caps count lengths; moving the supplied Vec retains
its capacity. Prepared `try_reserve_exact` operations specify requested slots and do
not promise actual allocator bytes or granted capacity. Cash snapshots/receipts
should each stay<=2KiB and own no vectors; source instruction construction/evaluation
performs0heap allocations. A prepared bundle should stay<=128bytes with one selected
coefficient (not both conventions), original ordinal and exact coordinate/day
metadata;20,000 bundles then bound payload at2,560,000bytes. The PV workspace has at
most20,000f64 slots160,000bytes. Input-owned Q/coupon Date vectors retain their
independent caps; their supplied allocations are transferred, not duplicated. Freeze the proposed layout/lifetime constraints before contract adoption;
verify actual layouts and copies after implementation, before implementation
acceptance. The proposed figures are not implementation measurements. Risk Debug excludes giant arrays.

Constructor source work is linear in Q+actualcount with source-cache decomposition
bounded by the same vectors. Plan preparation is bounded linear traversal/count plus one
exact-sized bundle reservation. Each direct evaluate/risk entry reserves its own
exact-sized PV workspace once. Each yield entry reserves one workspace once and every
callback reuses it; callbacks allocate zero times. There are no per-flow allocations. No
complete source vector clone/callback/globalstate/implicit date query. Future
implementation must record actual allocation attempts and semantic liveness review
separately from corpus certificate metadata.

## N6. Required full semantic test families

These are required future families, NOT72 implemented tests, not row-count-only
acceptance and not a claim that any Canadian generator/fixture exists. Each must bind
actual values/errors/causes/indices/provenance to reviewed independent expectations,
with explicit source decisions and stage definitions. Public dated cases must
reconstruct valid complete inputs; primitive supersets remain labelled separately.

| Families | Required semantic assertions |
|---|---|
|CAD01|All4 primary PDF hashes/version/scope; as-of/metadata/legacy-redirect limits; no current-market certification|
|CAD02|Literal p16 wrong result preserved, independent48/73 exact reference and rounding distinct|
|CAD03|p17 extra full coupon preserved; corrected local short-last formula differs by C/f|
|CAD04|Appendix83.298 divergence; other20 cells; no hidden fix to retained source|
|CAD05|1.1/1.2/1.3 lateAI differences; text-order differences not falsely mathematical|
|CAD06|Monthly ActAct exception vs p42 fixed365; policy identity and local decision explicit|
|CAD07|MMp49 terminal once atNC1/NC2 and zero/nonzero yield; direct exact coefficients|
|CAD08|LongfirstDIC1 stub versus full-first-payment definition; diagram-based concrete difference|
|CAD09|Every frequency2/4/12 and annual coupon/yield unit conversion; unknown frequencies absent, no defaults|
|CAD10|All metadata enums/type census/traits/no implicit float-money conversion/new roots and errorcatalog|
|CAD11|Schedulecap0/1/2/20000/20001, Qcount and actualcount precedence, no allocation before structure|
|CAD12|Minimal outer coverage, duplicate/decreasing Q, redundant prefix/suffix and Date edge failures|
|CAD13|Direct anchor vs recursive clamp/EOM/Ignore, leap/nonleap/monthend across all frequencies|
|CAD14|Actual coupon order/boundary/offcycle/interior adjacency and original index errors|
|CAD15|Regular first/last with C0/Cpositive; zero coupon amounts remain actual coupon dates|
|CAD16|Short first dates at each endpoint/day; value/payable/day-denominator distinctions|
|CAD17|Long first one/several/thousands full periods; issue after skipped virtual boundary; no invented payments|
|CAD18|Short final including source48/183day cases and forward full quasi denominator|
|CAD19|Long final local extension with0/1/many interior actual coupons and explicit source-deviation label|
|CAD20|Composite one-coupon both ends offcycle, same/different quasi periods, no duplicated coupon/principal|
|CAD21|Shortfirst+shortlast, longfirst+shortlast, longfirst+longlast crossproducts and single shared period|
|CAD22|D<=I<F, reopenings, equality/invalidorder precedence before trueT/cache work|
|CAD23|T=M/T>M/T<M, no extra coupon accrual or inferred calendar, MM horizons change only|
|CAD24|S<I,S=I,S=D,S on skippedQ/actualcoupon,S=M-1/M; visible post-M rejection even T>M|
|CAD25|ActAct source value from independent interval intersections, full and proper fragments, no overlapping single-quasi pieces|
|CAD26|Canadian365 proper branch and -1/146 drop, exact threshold unreachable for2/4/12|
|CAD27|Canonical completed181/182period exactlyC/f; literal partial-at-full counterexample retained|
|CAD28|Monthly payable oddCB vs monthly AI=ActAct; don't merge coupon and accrued policies|
|CAD29|A/AI reset only actual coupons, persist at skippedQ, C0 and zero accrued geometry metadata|
|CAD30|C MIN positive/MAX/scales0..38, C/f scale39/40/nonterminating, no premature public amount gate|
|CAD31|Source q/m views all7modes, exact nonterminating failure, OutOfRange independent of final cash|
|CAD32|Full source AI normal/finiteness at min/day/max/cap plus exactzero, independent numerator/denominator bounds|
|CAD33|Fractional compound alpha1/2 vs Treasury rational prefix, exact squared-factor discriminator|
|CAD34|General chi exponent exact2partial coordinate vs rounded-lambda/annualtime substitutions|
|CAD35|Compoundfloors and neighbors allf; exact δ steps; f12 argument rounding; invalid upper unqueried|
|CAD36|MIN_SUB/tiny threshold/zero exponent identities, non-tiny branches and primitivecallcounts|
|CAD37|Compound ln/ln1p/exp certified reference and new per-frequency profile; informative boundary and rejected cases|
|CAD38|MM exact domain neighbors/non-dyadic floor and sharedaffine positivity; terminalrho0; valid y=-300 one-day MM outside compounddomain|
|CAD39|MM intermediate overflow recoverable by exact ratio; all source factors observed including zeros|
|CAD40|Direct source×observedfactor IEEE subnormal/overflow ties and double-rounding mutant kills|
|CAD41|Dirty reduction errors, original coupon indices and strict operation precedence|
|CAD42|Three typed price bases and reports; compoundA vs MMAI; no automatic BondPrice::Dirty ambiguity|
|CAD43|Direct3-term settlementbridge and target reverse vs roundedclean/add, including signed/zero all-in targets|
|CAD44|Negative all-in valid witness, no positive-price gate on bridge/cash; independent modelpositive denominator|
|CAD45|All6 ideal compound derivatives/moment via independent dual2/quasi controls, allf/allstub shapes|
|CAD46|All6 MM ideal quantities via positiveaggregateH and independent per-flow derivatives; trueT changes|
|CAD47|Selectedobservedrisk PV-seeded terms vs exactsourceH route; exact one53bit rounding and all held ratios|
|CAD48|Raw±overflow/underflow/exactzero vs independently finite normalized views; no getter dependency|
|CAD49|Fractional rawD2normal/Cvunderflow witness; no borrowed Treasury range exclusions|
|CAD50|Normalized denominator D_hat, not clean/allin; MM moment convention, no periodic Macaulay shortcut|
|CAD51|Extended accumulation243word bit binding, directnorm, signedzero errors, sourcezero without invalid normalization|
|CAD52|Risk/publicreport inspectors/provenance/Debug no giantbuffers; independent clean/bridge getter failures|
|CAD53|Yield all3basis target resolution including externalnegative/zero but positiveModelTarget; shift before allocation|
|CAD54|NonpositiveModelTarget0calls, both residualscales threshold/equality, exact signs/cancellation/zeroiff equality|
|CAD55|Brent lowerzero1call, upperzero, same sign, countcap1/2/3, cachedselectedpayload no finalcall|
|CAD56|Actual financial/numerical callback failures if reachable; exact stagepointcountcause; synthetic transport separate|
|CAD57|Negativevalidy/compoundfloor/MMdomain errors counted onlywhenqueried, original index and immediate causes|
|CAD58|MM positive lowerimage and unattainablepositivetargets; illconditioning/flat roundedimages/precisionexhausted/brackettolerance; no fabricated root certification|
|CAD59|Roundtrip targets at cached observations, exactcertificate root brackets with slope-qualified informative/uninformative coordinate radii|
|CAD60|MM/compound/strategy identity and branchswitch boundaries around last actual coupon, no hidden mode default|
|CAD61|All4 cash recipes actualsource quantities/principal excludedcoupon; positiveN evenzero inputs|
|CAD62|All7modes×2aggregations×signedpointprice/nominalscale/quantumscale and exact midpoint cases|
|CAD63|Appendix certified sevennominals;1mcomponent-vs-total cent;1bnfullprice-vs6dp nonmidpoint discriminator|
|CAD64|Continuousdeclared interval Exact unresolved evenendpointmultiples; equaldeclaredbounds point with declaredprovenance|
|CAD65|Capital-first thenAI Exact precedence; intermediatewideoverflow+netcancellation; final lowerbeforeupper|
|CAD66|Stageinputpointness vs intervalconstructor, quantizercollapse thennetpoint; no endpoint-only universalExact|
|CAD67|Decimalrangeholes/canonicalcoefficientlimits/OutOfRange sign and operation; no clip or suggestion|
|CAD68|Dirtyallin suppliedevidence vs observedbridge and exactAIinvoice, no unjustified cross-recipe guarantee|
|CAD69|Owned cash snapshotgeometry/fullcount/source ratios/None semantics, sourceviewfailure independent fromcash|
|CAD70|EqHash retainscomplete reconstructible instrument viaF/L/anchor/f/EOM; equalcashdifferentL counterexample; selection/evidencekind; determined vs nondetermined provenance|
|CAD71|All33+5 structuredvariants/stablekind/sourcechain/stages/indices; capacityproof exclusions distinguished from valid runtime failures|
|CAD72|Preproductionadmission guard/schemas/pins/mutants/debugrelease/Clippy/docs/targets/package/coverage; allocation/liveness review is not fixture metadata|

Required independent routes: exact Fraction/Decimal source/cash corpus; certified dyadic
nth-root or interval exponentiation ideal price/risk/inverse corpus; blind hand controls
before production; separate numerical primitive/boundary corpus; actual
production-linked private kernel checker after implementation plus public consumers;
meaningful semantic mutants killed by runtime assertions. Complete schema input
fingerprints exclude expected values/derived reachability/provenance labels that could
mask duplicates. Distinguish serialized witness identities, operation-specific primitive
diversity and dated financial reachability. Never turn shape certificates/stageproof
columns into invented runtime counters.

The small private `control_checks.py` has ACTUAL24 floor/admission rows,1222 full-policy
fragment arithmetic controls and2569 exact MM affine/domain-neighbor rows. Those finite
proposal controls do not substitute for these72 families, do not implement Canadian
pricing, and do not certify the candidate elementary profile. Frozen artifact generation
and production remain future, separately authorized/adopted work.

## N7. Independent research evidence available before this proposal freeze

The separate ideal-math first pass (before reading this proposal) is
`/private/tmp/sabiquant-canadian-numeric-challenge-v3/arithmetic.json`,
SHA808d0e6f5a544982f4eb97e062529adde554e1d23279adb070df4583ade4b854; source `derive.py`
SHA3498f846033efc47f9a356ddf9134ac5a05bbb5b015ae5ab4d55a9c4dc4e87d2. It records17
research groups,102 root certificates and72 frequency/shape/quote candidates, with exact
MM references and independently certified fractional compound roots. These are
ideal/source/local-geometry controls, not selected observed-kernel fixtures or runtime
coverage. It confirms normalized-underflow/raw-normal splits in compound and MM, MM's
positive lower image, the valid negative-y MM domain and MM Macaulay differing from a
periodic conversion. Preserve those distinctions when comparing this proposal.

The separate source/cash v4 challenge certifies all7 Appendix principal/aggregation rows
with256-bit dyadic root bounds and classifies the21 printed cells20match/1numericAI
mismatch. Use its1billion nominal full-price/display discriminator without a half-cent
tie assumption. Earlier100k displayed-price counterfactual lands exactly at a half-cent
and must retain an explicit tie policy; it is not a mode-independent discriminator.

## N8. Closed date-coupled MM ranges and price transformations

The [independent exact MM review](M8_CANADIAN_MM_RANGE_REVIEW.md) closes the earlier
staged range questions for the complete proposed model. Its proof uses the admitted
source/date coupling and all selected rounding stages, not arbitrary-PV primitive
extrema. Future implementation must match this protocol; generic primitive controls
remain separately labelled and must not fabricate impossible dated financial errors.

Payable pieces partition at most Q.len()-1 nominal cells, so including terminal par,
`sum B_i <= C*(Q.len()-1)/f+100 <2^142`, including C0. Exact shared affines satisfy
`d,n_i>2^-80`, `tau<2^14`. Positive-y factors are <=1; negative-y factors are <2^80.
A positive nonterminal coupon is >2^-136 and its factor is >=2^-22. The terminal
factor is >2^-1039 and its source coefficient is >=100. After direct IEEE rounds,
all nonzero PVs remain nonzero, exact PV sum S_PV<2^223, and
`2^-1033 <= Dhat <=2^223`. Factor/PV/dirty zero/infinity errors are proof-excluded
for valid MM states. Nonzero subnormal factors/PVs/dirty remain admitted, including
C0 with Q0001-01-01..0001-07-01, settlement0001-01-01, stated M0001-07-01,
true T9999-12-31 and y=MAX. No normal-only restriction or shorter delay is introduced.

| Selected retained quantity before final IEEE | Proved bound |
|---|---|
| Raw derivative magnitude abs(L) | <2^398 |
| Raw second derivative K | <2^493 |
| Raw time-weighted amount U | <2^238 and >MIN_NORMAL |
| Modified (-L/Dhat) | <2^176 |
| Convexity (K/Dhat) | <2^271 |
| Macaulay (U/Dhat) | <2^16 |

No MM risk getter can overflow. The terminal ideal moment exceeds50/MAX>2^-1019;
factor IEEE, PV IEEE and extended53 term rounds each preserve more than half,
so its retained contribution exceeds MIN_NORMAL. Raw moment is normal finite.
The final observed Macaulay lower bound is
`(1-u)^2/((1+u)*365)>1/730`, including its final IEEE round. Do not replace it
with the ideal >=1/365: a valid one-day C0 case at y=-280 gives selected bits
`3f66719f36016719`, strictly below exact1/365. Raw derivative/second derivative
and Modified/Convexity underflows remain genuine; exact selected-stage CMAX
witnesses are preserved separately from the earlier ideal tables.

For either convention after admitted finite Dhat, clean/settlement-bridge transforms
and finite typed target shifts are exact sums of represented binary64 terms.
Ahat/AIhat<2^144 gives perturbation<2^145, far below the MAX overflow midpoint
gap2^970. Such a transform cannot overflow. Its nonzero exact sum is an integer
multiple of MIN_SUB, so it cannot round a nonzero value to zero. Exact cancellation
is canonical+0; resolved model-target positivity remains a real independent check.
The defensive Result signatures remain without requiring fabricated financial range
failures. These binary transforms still promise no equality with exact-AI cash invoices.

The observed MM image also needs a distinct inverse control: the complete two-flow
MAX-yield witness in the linked review has Dhat bits3fd5375375375375, below the ideal
lower image181/546 by59/4917930793088581632. Its cached upper endpoint can be a valid
observed target even though no ideal inverse exists. Do not reject it by an ideal-image
precheck or assign a conditioned ideal-root accuracy assertion. It is an exact staged
Python certificate, not an existing Canadian Rust solver result.

N3's broad primitive envelopes remain storage bounds. This date-coupled closure does
not replace the complete guard/resource integration, actual implementation
liveness/allocation review, typed transport or independent preproduction evidence.
Source/cash arithmetic with signed evidence can still overflow or cancel and keeps
its full range/staging tests.
