# Canadian numerical and test-design review: independent first pass

Pre-contract research, recorded 2026-10-23 Bangkok. This is not an adopted contract,
Canadian API, implementation, accepted oracle/fixture, market certification, release
decision or foreign-platform runtime result. The original private producer made no
repository/Git/AGENTS/remote writes and read no QuantLib implementation. It did not open
the future Canadian API proposal before this first pass; readiness and source/cash
reviews had been read. Root promoted immutable review SHA256
`986f68cafe82ecefd1f96f9f6fd7c98ebc11666d43cd8bc093070f227f294901`, with editorial
formatting and repository links. This preserves independent mathematical derivation
after source review, without claiming blindness to those source findings. Read the
[source review](M8_CANADIAN_SOURCE_REVIEW.md), [research
reproduction](M8_CANADIAN_RESEARCH_VALIDATION.md), [later complete
proposal](M8_CANADIAN_LOCAL_DESIGN.md) and [numeric/test
companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md) with their respective ordering and
limits.

## Actual evidence and scope

The stdlib-only `derive.py` SHA-256 is
`e28a558b56ed7b1f4b73d258918d9f2631aff943a365fdcffa1036d29ecfa79b`; actual
`arithmetic.json` SHA-256 is
`6a552ca275461cb550b48b7acf3bbc794d15970892ac99dfd0add249ada80bfc`. `derive.log` records
an actual exit-0 PASS: 18 research groups, 104 exact integer/root certificates, 18 exact
two-edge exponent decompositions, and unchanged retained source inputs. The
frequency/shape/quote matrix contains 72 candidate cases and 504 dirty/all-six-risk
quantity enclosures; these are not 72 Rust functions, accepted financial instruments or
a complete preproduction corpus.

For nonrational powers, lower/upper dyadic endpoints are certified by strict integer
power comparisons. Every reported ideal binary64 result is the common integer-rational
nearest/even rounding of both certified endpoints. The ordinary IEEE extractor has 11
independent boundary/tie controls against Python's conversion. Decimal160/240
second-order automatic differentiation corroborates all seven quantities; Decimal
agreement is not the interval certificate. No Canadian library was run and no result is
labeled an observed implementation report.

Own complete-page visual inspection: IIAC1.3 zero-based PDF indices 6–23,29–49,54–56 (42
pages), plus the complete one-page BoC note (43 images total). Monthly41/42 and
amortizing43–46 were included to inspect scope, not quietly omit inconvenient
frequencies. Previously rendered pages were viewed in this conversation; pages41–46 were
additionally rendered into private v1. Text extraction was used only for
navigation/context. IIAC1.1/1.2 version comparisons are attributed to the retained
source-readiness and source/cash reviews, not falsely claimed as this author's own
complete visual inspection.

Primary retained bodies are IIAC1.3 SHA
`8d9e80c8179af935f891c504d30cbb0023744b5bd0008b3efaa6a96145df2845` and BoC SHA
`f70b78eac8c75adc9556da14d6594b89de73c156332dcc8e9ea5b9f1bb34f6ff`. The four retained
PDF hashes and review inputs are guarded by the arithmetic script. Hosting availability,
PDF metadata and retained release numbers do not establish current-market applicability.
Primary sources:
[IIAC1.3](https://iiac.ca/wp-content/uploads/Canadian-Conventions-in-FI-Markets-Release-1.3.pdf)
and [BoC scoped
note](https://www.bankofcanada.ca/wp-content/uploads/2010/07/cal_clean_price.pdf).

The retained source-readiness final index and source/cash v4 report remain distinct
evidence. Their useful Appendix2 price/invoice proofs are not rebranded as original work
here. No current internet-law or financial advice is supplied.

## 1. Source mathematics and local decisions must be separate

Use annual coupon amount C per100 in this report. IIAC's CPN is a decimal annual rate,
so C=100*CPN. f is payment frequency and, in the reviewed conventional compound
formulas, also yield compounding frequency. BoC's C already has per100 annual-coupon
units. Normalize percentage yield displays explicitly; do not use 8.000001 as a decimal
yield in the Appendix example.

| Topic | Source support | Remaining decision or limitation |
|---|---|---|
| Regular compound | IIAC10.1 semiannual and10.7 monthly; positive-base fractional power, valuation A from5 | Quarterly follows general-f notation but requires explicit generalization; do not identify all source cases as only f2 |
| Short first |4.2,5.2,7.2,10.2 | Price uses valuation coupon C/f times exact quasi ratio; cash payment uses4.5 instead |
| Long first |4.4 explicitly permits one or more full quasi periods plus stub;5.4/7.3 sums pieces |10.3/10.4 print two-quasi cases. DIC1 definition differs; a local interpretation/deviation record is required before extending examples |
| Short last |4.3,5.3,7.4,10.5/10.6 | Forward full quasi denominator differs from shortened actual last interval; first and last stubs can have different denominators |
| Long first + short last |General pieces give a coherent candidate |Not a complete printed joint price example; requires explicit composition decision and controls, not a source-complete assertion |
| Long last |Not supplied as a conventional bullet formula in reviewed10.1–10.7;4.1 says last odd period is generally short |Retain an explicit unresolved long-last extension question, rather than silently accepting or dropping it |
| Quarterly/monthly |6 explicitly covers quarterly/semiannual settlement and a monthly Act/Act exception;10.7 gives monthly compound;10.10.3 covers monthly/quarterly MM |Monthly AI conflict and MM terminal count below must be resolved. Other f values need their own declared eligibility and date rules |
| Amortization |10.8/10.9, visually inspected |Separate principal-balance model, not an automatic extension of a fixed-coupon bullet contract |
| One-flow MM |10.10.1; BoC corroborates a narrower cash-management repurchase operation |Define routing by quote/model, eligibility and actual remaining payments, not an undocumented branch based only on count |
| Two-flow MM |10.10.2 explicitly allows compound or MM-equivalent quote |They are different coordinates and different dirty/clean transformations; neither is a default inferred from the other |
| Multiple-flow MM |10.10.3 reinvests earlier coupons to maturity |Printed summation includes terminal coupon and separately adds CM: terminal-once consistency needs a disclosed local correction |

Important unresolved source facts are preserved:

- IIAC5/6 prescribe Act/Act monthly settlement AI;10.7.2 p42 prints C*DCS/365. C12,15 days of31 gives 15/31 versus36/73. These differ before rounding.
- For a184-day period, literal late-AI formulas decrease at182→183 days by C/(2*365), or1/146 for C5. Do not silently cap, smooth or replace this behavior. Completed181/182-day quasi periods also require an explicit full-coupon-versus-literal-piece decision.
- Strict threshold equality is unreachable at integer days for f2/4/12, because365/f is not integral. It is not a theorem for arbitrary frequencies.
-10.3 p33 defines DIC1 to the first actual coupon, while its formula already adds a full
coupon;10.4 defines DIC1 to the first quasi. The private long-first control retains both
literal and piece candidates and the915/364 per100 excess. Neither is silently adopted
as the repaired source.
- For monthly C12/f12 and three future payments1,1,101, terminal-once MM gives103 at y0, whereas literal p49 gives104. The same extra terminal coupon appears already in its NC1 case. Keep both readings, with provenance and a future source-deviation decision.
- The p16 short-last arithmetic error, p17 extra half coupon and Appendix83.298 cell are source discrepancies, not numerical tolerances. Appendix1million components-versus-total and1billion full-price-versus-display controls should be retained from the independent cash challenge.
- The appendix distinguishes bond-price presentation from full precision for principal, penny settlement AI and component aggregation. It does not establish midpoint ties, nine-decimal daily publication, arbitrary quantum defaults, or money-market-yield bonds' classification as money-market securities for rounding purposes.

## 2. Three amount roles and price transformations

Keep exact valuation cashflows V_i, payable cashflows K_i, valuation accrued A, and
settlement accrued AI separately. Their equality in regular cases is not permission to
merge their provenance.

For conventional compound pricing, the source model is D=sum(V_i*b^-lambda_i),
P_compound_clean=D−A, b=1+y/f. An invoice from that clean quote adds AI, so its
settlement-inclusive amount is D−A+AI, generally different from D. A target called dirty
must therefore identify whether it means model dirty or settlement-inclusive dirty.
Solve transformations must select the matching cached exact/represented shift once;
report both A and AI.

MM10.10 instead uses P_MM_clean+AI=D_MM, and BoC also subtracts/then adds its settlement
AI. Reusing the compound clean shift A for MM changes the model. A supplied analytic
model-dirty value cannot silently become a settlement invoice value.

Valuation odd coupons are C/f times quasi fractions. Payable odd coupons use the
selected Actual/365 Canadian Bond fragments plus completed periodic coupons. For
the183/184 first stub at C5, valuation915/368 differs from payable363/146 and plain
fixed365183/73. No payable coupon cent/publication stage or rate-to-float gate follows
from their displays.

Every scalar source/cash operation must retain all DecimalAmount scales0..38 and
coefficient domain. In117 exact research controls (3 frequencies×39 scales), C/f may be
nonterminating or need scale39/40 while nominal100*f scales it to the admitted amount C.
A public DecimalAmount source view must not gate a fused source-to-cash recipe. Exact
nominal and quantum staging must precede final money admission; wide rounded components
may cancel. All seven modes, positive arbitrary quantum coefficients, both aggregation
policies, interval pointness and provenance need explicit local contracts if included.

## 3. Compound ideal risk, coordinates and domain

For every future actual payment define exact lambda_i as the sum of fractions of full
quasi periods from settlement to that payment. Missing long-first quasi dates are
coordinates, not cashflows. For short last, its final beta=DCM/DQ is fractional; do not
replace it with one or rebuild its lattice backwards from stated maturity. Use original
indices/roles in errors.

With t_i=lambda_i/f and W_i=V_i*b^-lambda_i:

```text
D  = sum W_i                         y > -f
P' = -sum W_i*lambda_i/(f+y)
P''=  sum W_i*lambda_i*(lambda_i+1)/(f+y)^2
M  =  sum W_i*t_i
Modified=-P'/D; Macaulay=M/D; Convexity=P''/D.
```

These are derivatives of the ideal price as a function of annual decimal yield, not
derivatives of a discontinuous floating program. Convexity has no half. M is a specified
quasi-year clock, not automatically Actual/365. The ideal identity Macaulay=b*Modified
holds for this compound model; it must not be promoted to every Canadian quote mode.

Base seeds must use represented y exactly. At f12, next_up(-12) gives a base differing
from naive `1+y/f`; its relative error near the floor is financially significant. f+y,
division byf, rational lambda and lambda+1 each need a defined observation policy.
MIN_SUB/f may disappear before a long exponent amplifies it; a tiny branch requires a
deterministic remainder/error argument. For f12, `y/f` is not generally exact binary
scaling. Uniformly reusing a semiannual log1p profile is unjustified.

For positive coefficients, positive future lambda and par100, ideal D>0, P'<0, P''>0 and
M>0. Ideal compound price is strictly decreasing with image(0,infinity), but this does
not prove observed monotonicity, an exact floating inverse or a nonzero slope lower
bound. The current matrix uses independently certified ideal factors; it has not
selected any observed implementation policy.

## 4. MM one/two/multiple-flow risk and domains

A consistent terminal-once MM candidate reinvests each original flow a_i at time t_i to
terminal time tau, all times actual days/365. With h_i=tau−t_i:

```text
A0 = sum a_i                    B0=sum a_i*h_i
v  = 1+y*tau                    K=tau*A0-B0=sum a_i*t_i > 0
D  = (A0+B0*y)/v                y > -1/tau, validated exactly
P' = -K/v^2                     P''=2*tau*K/v^3
PV_i=a_i*(1+y*h_i)/v
M  = sum t_i*PV_i               # LOCAL risk-clock definition
Modified=-P'/D; Macaulay=M/D; Convexity=P''/D.
```

The source supplies the MM price/reinvestment formula, not these six risk names or a
Macaulay clock. The displayed M definition is a proposed local choice; another
convention requires its own name, proof and tests. Positivity follows because0<=h_i<tau
and v>0. For multiple flows, Macaulay generally differs from v*Modified and b*Modified.
It cannot be obtained by multiplying a derivative duration by a familiar base without
specifying the clock.

One flow has B0=0 and dirty image(0,infinity). Two or more flows with positive
nonterminal coupons have B0>0 and dirty image(B0/tau,infinity). C8, payments4 and104
at92/365 and273/365 has lower limit724/273 and the independent inverse
dirty3→yield7665/19. A generic positive-target check is insufficient to establish an
ideal MM root; an ideal-image reference is not permission to pre-reject a target using
an unproved observed-image bound.

The independent closed form y=(A0−T)/(tau*T−B0) is an inverse oracle. A public bounded
solver must either count its chosen price observation or use the shared counted solver;
no uncounted special success path is implied. The inverse becomes badly conditioned as T
approaches B0/tau. Its local derivative sensitivity is abs(dy/dT)=K/(tau*T−B0)^2, so
small price errors become arbitrarily large coordinate errors near the floor.

The valid MM quote domain is not y>−f. A one-day C0 principal at y=−300 has v=65/365>0
and dirty7300/13. Comparing y to a rounded −365/days threshold can misclassify an
adjacent float; test exact365+days*y before any chosen rounding. Distinguish stated
maturity, true payment maturity and tau. BoC explicitly defines true cashflow maturity,
while IIAC48 rolls nonbusiness maturity; neither settles all coupon adjustments,
accrual-day resets or calendar ownership.

## 5. Certified range discriminators and caller-domain policy

All bits below are certified rounding of ideal mathematics. They are not future Canadian
production expected-bit contracts.

| Candidate | Dirty | Raw P' | Raw P'' | M | Modified | Macaulay | Convexity |
|---|---|---|---|---|---|---|---|
|Compound CMAX,alpha1/184,y2^540|47a0cd9569529689|a56760cfe072e7b3|03a78156925d44c9|471760cfe072e7b3|1db642c8590b2164|3f6642c8590b2164|0000000000000000|
|MM2 CMAX,y2^540|47c5375375375375|845c99d7ae5bf3d0|0000000000000000|47a563f79021563f|0000000000000000|3fd021aa6ed1021b|0000000000000000|
|MM2 CMAX,y2^370|47c5375375375375|999c99d7ae5bf3d0|028c99d7ae5bf3d0|47a563f79021563f|11c591b893c1cf35|3fd021aa6ed1021b|0000000000000000|

For compound, dates are prior/issue2025-03-01, next2025-09-01, maturity2026-03-01,
settlement2025-08-31, C=2^127−1 per100. There are two actual payments C/2 and100+C/2
with lambda1/184 and185/184. Exact b=1+2^539<2^540, so firstPV>C/16>2^122. The second PV
is also positive normal and dirty<2^128. Raw P''>2^-968 is normal, whereas
Convexity<3*2^-1080<0.5MIN_SUB. This refutes the Treasury-specific implication
“normalized convexity underflow forces raw second derivative underflow” for fractional
compound. It remains valid with a positive binary discount gate: neither factor itself
underflows in this case.

For both MM2 cases, settlement2025-06-01 and future dates2025-09-01/2026-03-01 give
t1=92/365,tau273/365, amounts CMAX/2 and100+CMAX/2. Exact rational arithmetic determines
every row. y2^540 has normal negative raw P' while normalized Modified rounds+0. At
y2^370 raw P'' is normal while normalized Convexity rounds+0. Treasury/Gilt compound
exclusions for Modified cannot be reused in this MM model.

Two further conditional compound candidates exercise raw M underflow. At f4,
C=5878724413097077, accrual=settlement2025-03-22, skipped quasi2025-03-31 and first
actual/maturity2025-06-30 give lambda11/10. At f12,
C=33489637082511595851293634331048631, accrual=settlement2025-01-26, skipped
quasi2025-01-31 and first actual/maturity2025-02-28 give lambda36/31. At y=MAX, each has
certified ideal dirty rounding MIN_SUB, M rounding+0 and finite positive Macaulay. These
require a future policy admitting this one-flow compound geometry and extended
discount/source-PV rescue. A Gilt-style binary discount gate would reject their factor
earlier, and automatic MM routing could exclude the compound branch. They are therefore
design-dependent discriminators, not unconditional reachable public errors.

Caller-domain decisions must be independent: a valid dirty report must not depend on
every risk getter being representable. If retaining the existing strict risk domains,
positive/nonzero quantities rounding+0 and negative P' rounding−0 become the
corresponding typed range failures, with correct phase/quantity/sign. Each normalized
getter divides the retained extended quantity directly by dirty; it must not call a raw
getter first. Never create a fake report to hit an otherwise excluded branch.

Some exclusions can be proposed with explicit assumptions:

- Compound positive PVs, exact future quasi clock and rounded f+y imply Macaulay between positive time extrema; for the stated f2/4/12 geometry, t_min>=1/372 and t_max<10000. Modified has a positive lower bound of order2^-1032 and a finite upper bound below2^70; Convexity is bounded above below2^139 but may underflow. These are algebraic candidates requiring the eventual term/normalization staging proof, not libm theorems.
- MM ideal M cannot underflow: its principal terminal contribution is at least100*tau/(1+y*tau). For y<=0 it is at least100/365; for y>0 and y*tau<=1 it is at least50/365; otherwise it exceeds50/y>=50/MAX>2^-1019. To extend this to accepted PV/extended-term staging, carry the price/profile error and final rounding explicitly. For example an adequately proved relative error bound plus tau<2^14 leaves a large margin over the tau*0.5MIN_SUB absolute PV error. No unqualified observed exclusion is claimed before that proof.
- Dirty positivity follows from admitted positive par and nonnegative coefficients; zero-dirty/opposite-sign guards are not financial test states. Raw P'/P'' overflow, signed underflow and moment overflow require valid dated witnesses under the selected profile, not generic free coordinates.

## 6. Bounded numerical design choices still to freeze

Two defensible directions remain open. A certified elementary interval kernel can bound
exact-base fractional powers, select a prescribed factor observation and explicitly
classify unresolved rounding/range cells; its width, refinement cap and boundary policy
need proof. Alternatively, a precisely ordered log/log1p/exp observation model can
expose conditional empirical error allowances and named uncertainty guards, as existing
named-model contracts do. That alternative must derive new f12
cancellation/division/tiny-quote bounds and must not call empirical agreement a rigorous
interval or universally correctly rounded libm.

For either direction, freeze exact represented input staging, factor versus source-PV
range admission, direct once-only IEEE source-PV extraction, complete extended term
rounding, original-index traversal, exact dirty reduction, independent raw/normalized
getters and zero-source observation order. Decide whether tiny factors may be rescued by
source scaling. The two conditional M cases show why that decision changes reachable
error classes. Do not select expectations by running the future implementation.

For MM, an exact integer-rational seed for365+d*y avoids premature y*tau
overflow/cancellation. Common-maturity reinvestment must not be replaced with separate
simple discounts1/(1+y*t_i). If risk is based on accepted per-flow PVs, specify its
coherent derivative multipliers and retain their exact denominator contexts; ideal
aggregate A0/B0/K formulas alone do not define an observed sum protocol. A total error
allowance must account for PV error, multiplier error, extended rounding, dirty
reduction and denominator conditioning separately.

Inversion should retain caller-supplied finite brackets, target-fixed residual scaling,
exact represented residual subtraction, lower-first callback validation, failure counts
including the failing invocation, caps checked before invocation, cached selected
payload/no final repricing, derivative count0 for the shared derivative-free route, and
typed callback causes. Do not inherit the Treasury proof of interior failure
unreachability until the selected Canadian primitive observation/guard mapping is proved
monotone and range-safe. Ideal monotonicity alone does not prove it. Lower-zero success
with an invalid unqueried upper, cap1, same-sign errors, coordinate precision and failed
interior primitive/certification must remain separate tests or honest exclusions.

Clean/model-dirty/settlement-inclusive target transforms must resolve their chosen A/AI
cache once. Reject nonpositive resolved model target before callbacks where positivity
makes that a valid model rule; preserve zero counts. Ideal floor exclusions and
plateau/conditioning references remain qualified, especially for multi-flow MM. A
solver's small bracket, zero observed residual or termination reason is not a financial
enclosure.

## 7. Resource envelope and proof obligations

The research enumerated146097 Gregorian start dates and438291 direct clipped month
intervals over a400-year cycle. Observed length sets are f2:{181,182,183,184},
f4:{89,90,91,92}, f12:{28,29,30,31}. This is a civil/quasi arithmetic census, not a
Canadian holiday or announcement calendar.

A broad supported-Date geometry envelope can use n_f=f*10000 (20000/40000/120000
periods) conservatively. A service cap20000 is a different, tighter limit: do not
confuse the two or claim monthly full-Date coverage under that cap. Source rationals for
a contiguous interval reduce to complete periods plus at most two partial edges. Under
that assumption, the independent conservative source numerator/denominator bit bounds
are158/143 (f2),157/142 (f4),155/140 (f12); combining the global largest n and DQ
separately gives a looser160-bit numerator. These are coupled geometry bounds, not an
assertion that every numerator peak was executed or that a future representation needs
exactly these bits. Noncontiguous custom cashflow models need new bounds.

A five-word source scratch is a plausible conservative starting point, while exact
full-quote base/residual seeds need approximately1080 bits before normalization. f12
introduces a non-power-of-two division. For pure source cash, numerator scaling by
nominal and quantum can be bounded before any final public-money admission; a
conservative fused estimate is below roughly414 bits under the two-edge assumptions.
Count-times-quantum must be bounded by the division inequality, not an unrestricted
extra127-bit multiplication. Full price-evidence cash needs a separate signed
binary/decimal common-unit proof, wide cancellation, endpoint images and final
normalization; do not infer its width from the pure-source case.

For a candidate exact MM aggregate route, take a shared source denominator S<2^147,
total source numerator An<2^166, actual-day count d<2^22 and represented y=P/Q with
abs(P)<2^1024,Q<=2^1074. Let V=365Q+dP, positive in-domain; abs(V)<2^1084. A coarse
exact second-derivative numerator bound is about3442 bits and its denominator below3400
bits. Direct IEEE alignment adds at most1075 bits, so a conservative under4520-bit
scalar envelope is possible for that particular algebraic route. This does not approve
another accepted-PV multiplier route, all intermediate expression lifetimes, or a
complete112/128-word implementation. Every actual route must list retained inputs, local
copies, division/remainder scratch and caller-visible failure context before choosing
capacities.

For compound risk under the provisional geometry, admitted IEEE PV range plus exact
lambda and f+y gives a coarse term exponent span approximately−3135..1185 after term
rounding/summing. Existing243-word retained risk buffers appear ample; that is an
envelope candidate, not a final liveness audit. The numerical contract must prove
direct-normalization alignment as well as sum storage. No exponent-sized shifts or
per-exponent allocation should be used to realize a huge formal factor.

The reference generator also exposed a real work-bound issue: v2 attempted an integer
root whose degree was the product/LCM of two partial-period denominators and was
interrupted with exit130 while inside int_root. Its source and KeyboardInterrupt log are
preserved; it is not a PASS or production defect. V3/v4 prove an exact two-edge
decomposition, bound every root degree by184, cache repeated roots and finish
successfully. The 104 final certificates and18 decompositions are actual research
counts. This is a useful oracle-design obligation: bound proof work as well as
implementation work.

## 8. Preproduction oracle/hand partitions to complete before implementation

The following is a test design, not an executed Rust test inventory:

| Family | Independent expectation and discriminator | Required comparison/scope |
|---|---|---|
| Sources/versions |Retained body hashes, page formula provenance, literal/corrected discrepancy ledger |Exact pins; no current-market certification inferred |
| Definition/date roles |Accrual versus issue/reopening, supplied quasi versus actual payments, frequency/compounding, stated versus true maturity |Exact dates, original indices, count/cap/financial error precedence; no assumed calendar defaults |
| Geometry |Regular, short/long first; first/second/later quasi settlement; short last; first+last compositions; unresolved long last and one-payment overlapping stubs |Exact ratios/coefficients; explicit source/local labels; f2/4/12, leap/EOM/Date extremes |
| Amount roles |Valuation coupon versus payable coupon; valuation A versus AI, reset only at actual coupon |Exact rational/source caches and provenance; monthly exception, literal late branch and completed-piece choices |
| Exact source/cash |All39 scales, C0/CMAX, nonterminating/scale39/40 intermediates, arbitrary nominal/quanta, all7 modes, both signed limits and wide cancellation |Exact staged decisions; final public money admission, full instructions/receipts and evidence guarantees |
| Base/exponent seeds |Represented MIN_SUB/normal/MAX/±0, next_up(-f), f12 division, cancellation, exact lambda and lambda+1 |Exact primitive observations plus independent width proof; primitive supersets separated from dated cases |
| Compound factors/PV |Fractional versus rational-prefix witnesses, integral/exact-power cases, direct subnormal ties/double-rounding, source scaling rescue/gate choice |Prescribed bits only after profile adoption; certified ideal containment or explicit conditional allowances |
| Compound all six |Zero-yield closed anchors, symbolic derivatives versus Decimal jets, raw overflow/underflow independent of normalized admission |Exact chosen primitives; conditioned ideal bounds; CMAX normalizedCv/rawD2 and conditional f4/f12 M cases |
| MM price/risk |One/two/multiple flows, common-maturity reinvestment, terminal counted once, negative domain below−f, true maturity |Exact rational ideal/primitive comparisons; chosen M clock and accepted-PV derivative stage; no familiar duration shortcut |
| MM image/conditioning |Positive lower image for B0>0, independent closed inverse, targets near/below floor and high-yield plateaus |Conditioned coordinate bounds; ideal-only image labels never overassert observed rejection |
| Risk range |Three certified underflow discriminators; raw/getter independence; moment exclusions with explicit accepted-PV profile assumptions |Typed caller-domain sign/quantity/context; never force impossible private reports |
| Target/solver protocol |Every supported price basis, negative clean resolving positive, zero-count target rejection, lower-zero invalid upper, caps/order/errors/cache |Exact target bits and evaluation trace; inherited shared solver semantics named separately |
| Ownership/resources |Borrowed plan versus owned reports; original-index failures; fallible reserves; immutable callbacks; live temporary census |Runtime valid controls plus static/width proofs; allocation-only exclusions without OOM injection |
| Mutation discrimination |Wrong first factor, early base/alpha rounding, wrong A/AI, paid-versus-valuation coupon, wrong final beta, repeated terminal coupon, wrong MM denominator/clock, raw-normalized dependence |Compile before runtime kills; preserve source/oracle guards and survivors; no synthetic panic mutation |
| Acceptance |Independent author ordering, frozen hand/oracle evidence, actual full consumer execution, native/debug/release/Clippy/docs/package/coverage/foreign compilation |Report row counts versus test functions, partial scopes and missing controls honestly |

Price/risk accuracy bounds should use exact/certified absolute error near subnormals and
propagate magnitude/denominator uncertainty. A blanket ULP limit or price-relative norm
is insufficient. Label every uninformative ideal or inverse comparison. Keep source
literature displays distinct from native full-price bits. Independent inverses must
select target first rather than merely round-trip a price generated by a convenient seed
yield.

## Outcome and next boundary

The complete fixed-coupon candidate needs source decisions, a numerical
observation/error contract and independent preproduction acceptance before any Canadian
implementation. No easy semiannual/regular-only scope was silently substituted. The
source conflicts, long-last/general-frequency extensions, one-flow quote routing, actual
maturity, amount roles, all-six risk clocks/ranges, source/PV admission and whole-live
resource bounds remain explicit review/adoption questions. The actual18-group research
success is evidence for that decision process, not permission to treat the future
contract or all financial/native-platform gates as complete.
