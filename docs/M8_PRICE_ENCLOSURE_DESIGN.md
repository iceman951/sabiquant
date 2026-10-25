# Complete-model price enclosure design (proposal)

Unadopted engineering proposal, recorded 2026-10-26. This adds no public API,
production implementation or accepted contract. The local runtime checkpoint is
`ae7bda7`; existing contracts and frozen references remain unchanged.

The body records the preceding private research checkpoint. Its statements
about repository writes refer to that checkpoint; this transcription adds
proposal documents and the research driver.

[Design](M8_PRICE_ENCLOSURE_DESIGN.md) ·
[API](M8_PRICE_ENCLOSURE_API_PROPOSAL.md) ·
[Resources](M8_PRICE_ENCLOSURE_RESOURCE_PROPOSAL.md) ·
[Research validation](M8_PRICE_ENCLOSURE_RESEARCH_VALIDATION.md)

Research at local baseline `da0f2d481dc5e349c8ba517b4d5e6083d92bd39c`.
Report drafted 2026-10-25 13:32 Asia/Bangkok. Root concurrently owns Canadian C9,
solver/resource/coverage follow-up changes; this report does not certify that
follow-up or claim full Canadian conformance. No repository, contract, API,
requirements, fixture, Git or remote change is made by this research.

## Outcome and precise scope

A constructive route exists for all existing model equations using exact input
rationals, directed normalized dyadic intervals, a rigorously bounded logarithm
series, and a rigorously bounded exponential series. No native libm hypothesis is
needed for containment. Rational Treasury/MM paths are valuable specializations;
they do not replace fractional/Continuous/mixed branches or any named market.

This is a mathematical design and ordinary-case feasibility result, not an adopted
production design. A bounded Rust implementation, simultaneous storage/stack,
primitive scratch capacities, actual allocation paths, API/error/admission policy
and complete independent preproduction still require review. In particular, an
`i32` exponent is not sufficient for the entire generic mathematical input domain.

Complete scope retains zero/fixed generic plans, all 32 compounding configurations
and per-flow hybrid branches; complete conventional gilt; complete fixed-principal
Treasury; Canadian f2/4/12, Compound/MM/explicit last-coupon strategy, every admitted
odd/composite shape, both EOM policies, true maturity and all three price bases.
Signed generic coefficients, zero and negative clean/all-in prices are first-class.
No existing observed evaluator, risk, inverse, source or exact-cash behavior changes.

## 1. The exact function being enclosed

The certificate would say only: **the selected mathematical model at the stated
exact supplied/prepared inputs belongs to this interval**. It does not establish
current market/legal convention, source acceptance, historical issuance, maintained
calendars, owner acceptance, or the correctness of an uncertain input quote.

| Model | Authoritative exact input meaning and target |
|---|---|
| Generic | Buyer-entitled `PreparedDiscountFlow.amount` and `.horizon` are the exact real values of their finite binary64 bits. Hold entitlement, source flow index, compounding and branch fixed. With exact represented y and t>=0, Simple d=1/(1+y*t), Continuous d=exp(-y*t), Periodic d=(1+y/m)^(-m*t). P=sum A*d in amount units. |
| Generic clean/quote | Clean amount is P minus the **exact real value of the cached binary64** `BondPricingPlan.accrued_amount`; per100 is 100/face times that exact expression, with face the exact positive represented input. This intentionally certifies the prepared-plan model, not unrounded coupon/day-count/accrual mathematics upstream. It is distinct from enclosing the already rounded `BondValuation.clean_amount` or quote getter. |
| Gilt | A_j are exact selected source dividends, including exact published six-place first dividend and exact terminal par addition. Missing/detached dividends do not create bundles or renumber j. Exact lambda=(r+j*s)/s. Dirty=sum A_j*(1+y/2)^(-lambda); clean=dirty minus exact signed source A. Preserve all seven accrued lifecycle branches and strict-after ex-dividend status. |
| Treasury | Exact raw analytic A_j (not published coupon cash), b=1+y/2, q=(2s+r*y)/(2s), dirty=sum A_j/(q*b^j). Clean subtracts exact **selected published AI** under the retained daily policy and tie rule; neither raw AI nor its binary64 cache substitutes for it. Regular/short/long first, D versus I, missing Q1 payment, and original j remain distinct. |
| Canadian Compound | Exact valuation B_i and exact chi-derived lambda_i. D=sum B_i*(1+y/f)^(-lambda_i); Clean=D-A; SettlementDirty=D-A+AI. A and AI are exact canonical source ratios. Actual paid dates reset accrual; virtual skipped Q dates do not. |
| Canadian MM | Exact payable B_i; tau=(T-S)/365, rho_i=(T-payment_i)/365 and d=1+y*tau. D=sum B_i*(1+y*rho_i)/d; Clean=D-AI; SettlementDirty=D. Terminal payment uses true T. Preserve explicit last-coupon selection and true-T effects. |

Actual Rust preparation was inspected: generic plan pushes coupon.amount and a
checked direct horizon, then caches rounded earned-minus-detached accrued reduction.
Gilt retains exact source plus a separate binary cache. Treasury retains raw/source
publication data and separate raw/published binary caches. Canadian retains exact
valuation_ai/settlement_ai and separate observed metadata. A new named enclosure
must borrow/reconstruct exact retained ingredients, not call public Ahat getters.

For generic hybrid selection, Rust compares prepared t to the represented result
of `1.0/f64::from(m)`: SimpleThenCompounded uses Simple when `t<=threshold`;
CompoundedThenSimple uses Simple when `t>threshold`. Preserve that selected branch,
including the exact threshold bits and both adjacent times. Replacing it by an
exact-real t<=1/m test can change the model. All ten counts are retained:
1,2,3,4,6,12,13,26,52,365. Periodic/hybrid global y>-m validation remains applicable
even when the selected flow branch is Simple, the plan is empty or t=0.

No observed delta_hat, g_hat, ln/exp result, rounded lambda, observed factor/PV or
named Ahat is substituted into these targets. Existing observed failure order and
conditional comparison profiles remain untouched. Whether a future separate producer
also admits mathematical prices for inputs whose old observed PV is unrepresentable
is an explicit contract decision; it cannot silently change the old evaluator.

## 2. Directed number representation and arithmetic

Candidate endpoint: signed p-bit significand times 2^e, with arbitrary real range
represented by a **bounded wide integer exponent**, exact zero separately, and no
IEEE underflow/overflow inside the proof. Research precisions are finite and explicit;
64..1024 is an illustrative design domain, not an adopted API or precision ladder.

Normalize exact N/D*2^E by integer significant-bit comparison and quotient/remainder.
Select floor for a lower bound and ceiling for an upper bound, including sign and
power-of-two carry. A discarded nonzero bit must survive as sticky information.
Multiplication takes extrema of all four endpoint products; division requires an
interval excluding zero and uses ordered reciprocal endpoints. Signed addition
must use directed alignment, never silently drop a small opposite-sign operand.

A bounded alignment route is to select a target unit near the larger endpoint,
floor both aligned operands for a lower sum and ceil both for an upper sum, then
normalize outward. A huge exponent difference is compared as metadata and becomes
an exact sticky/tail decision; it never becomes a huge allocation or integer shift.
This may widen cancellation bounds but preserves containment. Close/equal exponents
can use exact local alignment. Exact source ratios are normalized using bounded
bit-length-relative shifts, not absolute exponents.

An exact valid denominator
can have a rounded interval that still contains zero. That is arithmetic ambiguity,
not invalid-domain evidence: refine within budget, use an exact rational denominator
route, or return unresolved. Never assert/panic or manufacture a financial domain
error. The ordinary Fraction prototype's division assertion is not a public policy.

Each primitive has a local rational inequality to prove. Induction then proves
whole-expression containment. Frame sizes/borrowed slices and exact quotient loop
bounds must be derived for the new operations; an abstract arithmetic description
is not an accepted Rust capacity proof.

## 3. Rigorous logarithm — no elementary primitive assumption

For positive exact rational b, find integer k exactly with 2^k<=b<2^(k+1), and set
m=b/2^k in [1,2). Let z=(m-1)/(m+1), so 0<=z<1/3. The geometric expansion of
2/(1-z^2), integrated from zero, gives

    ln(m)=2*sum(j=0..N-1, z^(2j+1)/(2j+1)) + R_N
    0<=R_N<=2*z^(2N+1)/((2N+1)*(1-z^2)).

The same expansion at z=1/3 encloses ln(2). Since z<=1/3<1/2,
R_N<=2^(1-2N) is a deliberately loose dyadic tail bound. Evaluate the finite positive
sum and recurrence using directed intervals, add [0,2^(1-2N)], then combine
ln(b)=ln(m)+k*ln(2), reversing endpoints when k<0. Input conversion and every
arithmetic rounding are enclosed independently. N=p is a finite research choice.
An exactly unit base returns exact log zero; it does not inherit an artificial tail.

Proof sketch: the integrand has an everywhere nonnegative omitted geometric tail
on [0,z]; each omitted denominator is at least 2N+1, yielding the displayed bound.
This proof concerns the exact rational z. Rounded interval upper z need not equal
1/3; the analytic tail uses the independently proved exact input range. Its
finite-sum interval already contains all conversion/operation rounding.

## 4. Rigorous exponential — all signs and finite binary64 exponent arguments

For each exact dyadic endpoint x of an argument interval:

1. x=0 gives exact1. Otherwise choose s>=0 with a=abs(x)/2^s<=1/2.
2. Evaluate S_N=sum(j=0..N,a^j/j!) with a positive directed recurrence.
3. Add a rigorous tail [0,2^-N]. Indeed the positive Taylor tail is at most
   sum(j=N+1..infinity,a^j)<=2^-N for 0<=a<=1/2. N=2p is a finite research choice.
4. Intersect with independently proved exp(a) in [1,2]. The upper follows from
   exp(a)<=sum a^j<=2; the lower is1. An empty intersection indicates a bug.
5. Square the positive interval s times. If x<0, take its directed reciprocal.
6. Exp is increasing, so exp([L,U]) is enclosed by [lower(exp(L)),upper(exp(U))].

The exponential is the mathematical power series and satisfies the exact doubling
identity. No native ln/exp call appears in the containment path. Directed local
rounding plus the positive Taylor remainder and monotonicity establish containment.
The Fraction research prototype follows this route only at ordinary magnitudes;
it does not implement the eventual bounded exponent/alignment storage.

## 5. Whole-model termination and exponent range

These are abstract algebraic bounds, with assumptions stated; they are not measured
Rust stack/heap evidence or adopted limb budgets.

Every finite represented |y| and t is <2^1024. Generic Continuous exact exponent
abs(y*t)<2^2048. For a periodic base b=(m+y)/m with y>-m and integer 1<=m<=365,
all finite y lie on the 2^-1074 dyadic grid; hence positive m+y>=2^-1074. Therefore

    2^-1083 < b < 2^1025,
    abs(ln b) < 1083 < 2^11,
    abs(m*t*ln b) < 2^1044.

Here ln2<1 follows, for example, by bounding its positive atanh series by
2*(1/3)/(1-1/9)=3/4. Gilt and Canadian compound have exact lambda<20000<2^15 and
the same looser base envelope, so their exponent magnitude is <2^26. Treasury
needs only rational q and integer powers; using exp(-j*ln b) as an alternative
still lies inside this envelope. MM/Simple are rational and need no exponential.

Intersect computed argument intervals with the appropriate independently proven
closed magnitude envelope. In particular every exp argument endpoint can be held
inside [-2^2048,2^2048]. This gives **s<=2049**, regardless of native observations.
No loop waits for numerical convergence: log uses p terms, exp uses 2p terms,
squaring has this explicit cap, and every outer traversal uses the prepared count
plus an explicit new work budget. The budget must account for every directed
endpoint operation, retries/precision stages and source conversion, not merely rows.

A loose exponent-metadata derivation: for p>=2, positive normalized outward rounding
is within a factor of2 of its exact positive operand. Starting exp(a) in [1,2],
let H_0=1, H_(j+1)=2H_j+1. Squared endpoints lie in [2^-H_j,2^H_j], with
H_s=2^(s+1)-1. A final reciprocal/outward round gives the slightly looser exponent
magnitude 2^(s+1), at most 2^2050. Adding p<=1024, finite source/face scaling and at
most a 64-bit prepared count still leaves magnitude below2^2051 under this loose
arithmetic construction. Even a factor8 enlargement per signed sum adds at most
3*(2^64-1) to the exponent envelope, far below that bound.

Thus a signed exponent needs up to about **2052 bits** under this loose proof;
33 u64 words (2112 bits) is a candidate metadata capacity. This is not a claim that
33 words is an approved final Rust resource route. Check every source/intermediate
form, normalize/carry convention, exponent temporary, comparison and full metadata
operation before adopting an exact count. A small i32 exponent would fail the
complete generic mathematical scope, even though all ordinary/named cases are small.
Absolute shifts by this exponent are forbidden. Mantissa/seed/division scratch widths
and simultaneous live endpoint slots remain an explicitly open resource proof.

The current normalized representation can return enormous conservative endpoints
without materializing enormous powers of two. It need not decide a finite cash result
for every finite input. Bounded resource exhaustion or nonexportable endpoints must
be visible, never reclassified as unsupported compounding or false financial overflow.

## 6. Signed coefficients, dependence, cancellation and exact specializations

Generic coefficients can be negative. Enclose each signed term correctly and add
signed intervals; no positive-flow relative bound, positive lower price assumption,
or gross-to-net division is valid universally. Dirty/clean/all-in may cross zero.
A zero net value does not justify discarding invalid input/domain checks.

For the new mathematical producer, identical **exact factor functions** can be grouped
and their exact rational coefficients summed. This can prove exact cancellation or
constant cases before expensive interval work. Same rounded factor bits are not an
identity proof; different factor functions whose current sum is zero are not thereby
constant. Exact y=0/t=0 and unit bases are valid algebraic controls. Whether to add
this optional optimization is a future contract decision. It does not bypass any
old observed factor/PV gates, because those existing APIs are unchanged.

Enormous opposite-sign terms may have a finite or zero exact sum. A huge term or an
individual nonexportable bound is therefore **not** final-price overflow. Keep it
in the extended interval representation, sum/shift/scale first, then inspect final
bounds. Loose cancellation may leave a wide interval even after all allowed work;
return unresolved. Never clip, assume cancellation precision or manufacture a sign.

Rational specializations:

- Generic selected Simple and Canadian MM: exact affine/ratio evaluation or directed
  normalized dyadic rational evaluation, with exact denominator positivity checks.
- Treasury: retain exact b,q seeds and both directed p256 endpoints; its existing
  proof is useful, but new full-price products/sums/AI/export still need proof.
  Existing factor_hat, IEEE PVs and dirty sum are observations, not price bounds.
- Gilt/Canadian rational powers: exact root sign certificates can specialize bounded
  day-length degrees. The Canadian reference certifies m^n inequalities. Generic
  dyadic prepared times may induce degree2^1074; direct degree-sized nth-root work is
  not a uniform bounded route. The log/exp construction covers those configurations.

## 7. From a price interval to all-real settlement evidence

Finish the selected exact clean/dirty/basis and unit expression before endpoint
export. Outward binary64 export must find a lower finite float <=L and an upper
finite float >=U. Keep directed zero/subnormal boundaries: a tiny positive real
may need [0,MIN_SUB]; a tiny negative real may need [-MIN_SUB,0]. This is containment,
not a nonzero observed-PV underflow error. Zero and negative clean/all-in are valid.

If the upper required bound exceeds MAX, or a lower bound is below -MAX, a finite
binary64 enclosure does not exist at that precision. This does not prove the true
final price overflows: the interval may simply be too wide after cancellation.
Return finite-endpoint-unresolved/budget-unresolved, or a separately proved range
outcome if the full interval excludes the finite range. Never clamp to MAX, replace
an interval by its midpoint or treat adjacent floats as the only possible values.

Create declared `SettlementEvidence` using the existing constructor. Its interval
means **every real between the exact endpoints**, including between adjacent floats.
Existing receipt guarantees remain `UniqueForDeclaredEnclosure`; the new producer's
reviewed containment proof supplies the separate premise. An immutable controlled
producer wrapper could retain model/version/input meaning, basis/units, precision,
work and bounds; no constructor/field/type/API is adopted here, and a caller-provided
boolean or free-form proof label cannot provide that proof.

Cash composition must preserve exact source AI and its model-specific role:
Gilt clean+source A; Treasury clean+selected published AI; Canadian Compound clean
D-A plus source AI yields D-A+AI, while MM clean D-AI plus AI yields D. Feeding
Canadian ModelDirty to SettlementDirty cash without its exact shift is wrong.
The generic direct-dirty route avoids an artificial independent clean/AI rectangle;
using two interval inputs may be conservative because their dependency is lost.
Pure coupon/source-accrued cash already has exact source arithmetic and need not
acquire a fictitious analytical-factor certificate.

Run the complete existing cash pipeline for the declared all-real interval. Every
mode, source stage, total/component aggregation and Exact continuous/discrete image
rule remains in force. Source ambiguity can collapse at final cash. Nonpoint Exact
may remain unresolved. A true irrational value or exact tie need not become a unique
cash recommendation within any fixed precision. Refine only within explicit total
work and precision limits; intersect successive rigorous intervals to retain earlier
information. No automatic unbounded retry or universal determination guarantee.

## 8. Actual private ordinary research evidence

`research.py` and `ordinary-results.json` are retained, with exact hashes in the
private evidence manifest. No production arithmetic is imported. The fresh Fraction
interval implementation uses p=96, log N=p, exp N=2p, exact directed normalization,
and explicit analytic tails. Python Fraction storage is unbounded; this is not the
proposed bounded Rust representation or a worst-case resource test.

Actual result: **148 ordinary research rows**, all requested basis intervals narrower
than1e-18 and all deciding the specified face100/per100, N=1,000,000, cent, nearest/even
arithmetic control. This is one explicit illustrative cash policy, not every recipe.

- Generic33: all32 configurations, including all ten periodic frequencies and each
  hybrid's threshold and adjacent times with signed [100,-40,7] coefficients; plus
  an exact opposite-factor cancellation control. This is arithmetic research, not
  an independently generated complete dated/public-plan acceptance suite.
- Gilt4: regular/short-first/long-first/ex-dividend **coordinate controls**. They are
  not all dated lifecycle/AI branches or public constructor tests.
- Treasury3: regular/short/long raw-coefficient **coordinate controls**, with one
  explicitly fixed selected AI. They do not cover every daily/tie policy or date path.
- Canadian108: 3 frequencies ×12 shapes ×3 selections, using **Preserve only**.
  Dated metadata and exact source preparation reuse the accepted independent
  `canadian_ideal_oracles.py` build_state/prepare routines. Its existing ideal/factor/
  bound outputs are not called. This proves no new independent geometry/source
  validation and does not cover Ignore, all lifecycle settlements or every extreme.

Checks: 1,659,975 exact local directed-round inequalities; 50 exact rational full-price
containment checks; four exact rational root-sign endpoint controls; 198 Decimal
corroborations (220 and300 digits). Decimal ln/exp is used only in corroboration,
never the containment path. These counts are mathematical operation/check counts,
not Rust test counts or independent expectations for all admitted inputs. No native
libm accuracy premise or QuantLib source/code participates.

## 9. Independent preproduction and review still required

A new additive contract/requirement register is needed; do not alter the full-byte
pinned REQUIREMENTS, Canadian main/14 components, shared reader, frozen35 inputs or
old fixtures. No API or requirement is adopted by this report.

Before production:

1. Independent mathematical reviewer challenges all directed identities, sign/tail
   directions, range reductions and metadata bounds; retains counterexamples and
   reviews actual new prototype bytes. This report's unreviewed claims cannot be
   promoted directly to an adopted resource contract.
2. Derive final exact mantissa/source quotient widths, exponent temporaries, scratch
   slots and simultaneous live lifetimes; specify hard work/precision/flow budgets
   and honest unresolved outcomes. Review a safe bounded prototype with meaningful
   guard tests before promising production resource conformance.
3. Freeze full-model target/source semantics and all return/error/provenance rules.
   Distinguish prepared generic model from upstream mathematical source amounts and
   named exact source policy. Decide producer admission when old observed evaluation
   fails; preserve every current evaluator/risk/inversion result and failure order.
4. An independent author derives new expectations without reading new production.
   Exact rational and root sign certificates plus series remainder certificates are
   primary. Two-precision decimal checks corroborate; native conditional profiles
   are separately retained and never become the certificate.
5. New whole-model corpus covers both generic signs/cancellation/zero plans, all32
   configurations and mixed branches; all Gilt period/entitlement/7 AI branches;
   Treasury issue/reopening/odd periods/both source policies/tie rules; Canadian all
   f, both EOM policies, first/last/both/composite periods, all lifecycle states,
   trueT and three bases, both explicit models and last-coupon selection.
6. Test all seven cash modes, quanta, source stages and both aggregation policies;
   inside/crossing/touching ties, Exact point/continuous/discrete outcomes, collapse,
   subnormals, finite export failure, enormous signed terms with tiny/zero net,
   precision/work cap boundaries, errors/provenance/traits/threads as applicable.
7. Include semantic mutants: inward rounding, dropped tail/sticky, wrong sign on
   negative log k/coefficients, wrong hybrid threshold, rounded lambda/Ahat/AI policy,
   early term overflow, clipped endpoint, false constant, fake certainty at a tie.
   Freeze input/producer/output bindings and blind hands before new production.
8. Actual eventual implementation needs full regression, strict policy/docs checks,
   coverage with sound production/test classification, extracted std-only package
   and accurately scoped native/compile evidence. Root's current Canadian C9/resource/
   coverage follow-up is separate and remains under its own evidence/commit review.

No unresolved mathematical fact was filled with an external assertion. The series
and elementary bounds above are derived explicitly. No web lookup was needed.

## 10. Independent mathematical challenge and subsequent clarification

The exact reviewed draft is retained as `REPORT.reviewed-draft-v1.md`, SHA256
`3c82bb0ce0fd89606c5f812d674dbb75982bf9425a90e6982c8364867eedae92`.
The independent review is
`/private/tmp/sabiquant-m8-enclosure-math-peer-review-v1/REPORT.md`, SHA256
`2f0f7f7296745e737c61f98ff5903cde3ccd578fe58185ca565e2d55e77904f7`;
its evidence SHA256 is
`a1a822e758a55adc1cf096a69020a784c5b047627899d8511008b180a6f9900a`.

No containment counterexample was found. The reviewer independently checked signed
rounding/reciprocals, both remainder bounds, exact argument clipping/s<=2049,
squaring/reciprocal growth, signed cancellation and finite export semantics. Actual
finite challenge controls passed272 signed rounding checks,4,556 rational operation
checks across p2/3/16/64,9 log and9 exp comparisons to longer exact rational series,
16 tail controls,tiny +/-MIN_SUB log bounds and H_s recurrence through s2049.
These are mathematical controls, not Rust runtime or resource acceptance.

This version adds the reviewer's explicit zero-containing denominator policy in §2.
For signed-add magnitude padding, choosing
u=2^(max floorlog(|a|,|b|)-p+1) gives u<=M/2; aligned directed endpoint magnitude is
at most2M+2u<=3M, and final positive outward magnitude normalization is below2,
hence below6M<8M. This justifies the loose factor8 growth statement abstractly; it
does not measure a Rust frame or prove cancellation accuracy.

The loose fixed log remainder can dominate a tiny near-unit logarithm multiplied
by a huge time. Such valid extreme cases must be retained as explicit unresolved
controls, or improved with a proven z-dependent remainder within budget. Neither
this review nor the ordinary corpus promises universal cash determination.

`API_PROPOSAL.md` makes candidate entrypoints, input admission, endpoint/evidence
kinds, budgets and unresolved outcomes concrete. It remains unadopted and requires
its own API/resource/preproduction review.

## 11. Constructive resource and core divisor follow-up

`RESOURCE_PROPOSAL.md` now gives the complete-model unadopted Raw34/W52/E33 candidate,
paired directed primitives,24-slot borrowed-output schedule, both exponent directions,
source-role point shortcuts and initialized/terminal-reserved work accounting. These
are candidate bounds/costs requiring actual helper matching/layout/MIR review.

For this mandatory graph, affine divisors are built exactly in Raw and positive
point-converted with extended exponents; exp reciprocals remain positive. Thus the
general §2 zero-containing-divisor principle is not a reachable public unresolved
case here. Keep it as a private primitive defense/future conservative-extension
policy; the initial API must not invent an unused unresolved variant. Near-unit
log bounds can cross0 as arguments safely. No old evaluator behavior is changed.

The earlier math-reviewed REPORT c2b6369d... is preserved as
`REPORT.math-reviewed-v2.md` and in the frozen design/API package. The resource/API
follow-up does not alter the original148-row arithmetic/result bytes.
