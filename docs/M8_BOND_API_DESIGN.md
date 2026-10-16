# M8 bond API and validation proposal

Status: **design proposal only**, prepared 2026-10-16 on the local
`feat/m8-bond-readiness` branch after `4da3f43`. Nothing in this document
adopts a financial convention, extends the accepted requirements, implements
M8, closes NI-01, or grants milestone/platform/publication acceptance. The
owner's continuing local development authority permits preparing this design;
there is no proposed remote action or request for another approval.

The requested M8 scope is the entire [roadmap](ROADMAP.md) row: zero-coupon
and fixed-rate bonds, settlement, entitlement/ex-coupon, clean/dirty price,
accrued interest, yield, duration and convexity (F-BOND-01 and the remaining
flat-rate portion of F-CF-02). Callable, convertible, amortizing, floating,
inflation, credit and curve pricing remain their existing later features.
Finishing a generic kernel alone must not be reported as market-convention
coverage or full M8 acceptance.

## 1. Evidence and dependencies

Read-only inspection covered the current [cash-flow contract](M7_CASHFLOW_CONTRACT.md),
[cash-flow source](../src/cashflows.rs), [rates](../src/rates.rs),
[schedules](../src/schedule.rs), [root solvers](../src/math/solver.rs),
[requirements](REQUIREMENTS.md), [numeric policy](NUMERICAL_CORRECTNESS.md),
and accepted [ADR-0006](adr/ADR-0006-numeric-policy.md). No QuantLib C++ source
was read, translated or used to design an algorithm.

The Department of Finance Canada's [bond price/yield guide](https://www.canada.ca/en/department-finance/programs/financial-sector-policy/securities/securities-technical-guide/determining-bond-treasury-bill-prices-yields.html)
was read directly on 2026-10-16. It supports the distinction between face-normalized
quoted price, accrued interest and future-cash-flow discounting. It does not
select this library's settlement, ex-coupon or day-count rules. Its abbreviated
between-coupon discussion is insufficient to adopt a complete Canadian kernel.

Two other primary-source work streams are being reviewed independently by the
parent and another agent: UK DMO [Formulae for Calculating Gilt Prices from
Yields, version 4](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf), and
31 CFR Part 356 Appendix B Treasury equations. Their exact eligibility,
rounding, irregular-period and first/final-period domains must be pinned in
separate source reviews before named implementations are adopted. The parent
has identified a material distinction: fractional-power DMO discounting and a
Treasury fractional-first-period simple factor must not be conflated. This
proposal does not certify either complete market formula.

The parent also visually inspected ICMA-hosted historical 1998 ISMA material
for duration/convexity definitions. Its source ledger will distinguish exact
formula anchors from intermediate-rounded printed examples. In particular,
printed duration rounded before division can produce a different displayed
modified duration from a direct full-precision calculation. Record that
rounding path as literature provenance; do not loosen the general oracle
tolerance to match a rounded intermediate. Historical material does not by
itself adopt current market dates or settlement conventions.

Existing foundations can be reused with their contracts intact:

- Schedule retains raw boundaries, adjusted payments and generation/calendar
  provenance; FixedRateCoupon retains its full simple-accrual convention.
- Leg retains duplicate ordered payment dates, bounded flow counts and finite
  signed amounts. Its outstanding accrued amount is **not** already a bond
  clean-price adjustment or ex-coupon calculation.
- InterestRate has five explicit compounding methods and ten named periodic
  frequencies; accrual contexts include ICMA, Business/252 and explicit US EOM.
- M6 has bounded bracketed solvers and exact_sum of rounded finite terms.
  Existing solver callbacks return only f64; typed pricing failures require
  an explicit extension/adapter, rather than a NaN callback sentinel.

NI-01 remains a separate decision. An analytical settlement **date** can govern
which binary64 flows belong to the buyer without introducing currencies,
minor units, cash postings, exact decimal amounts or a rounding convention.
Any future exact amount proposal must separately reconsider ADR-0006. Rounding
an approximate analytical price to decimal cannot certify an ideal settlement
amount near a rounding boundary.

## 2. Proposed immutable instrument surface

Use one opaque `Bond` value with named `zero_coupon` and `fixed_rate` factories,
plus a small inspectable `BondKind` discriminator. Avoid duplicating analytics
between zero/fixed wrapper types. All fields remain private. Definitions retain
all terms, not merely generated amounts; floating-containing values implement
PartialEq, not Eq/Hash. Every public type is Send + Sync; no global evaluation
date, mutable cache, market parser, currency or default is introduced.

The following signatures are sketches, not promised public declarations:

```rust
Bond::zero_coupon(issue_date, contractual_maturity, face,
                  redemption: SimpleCashFlow)
Bond::fixed_rate(issue_date, schedule: Schedule, face,
                 coupon_rate: FixedCouponRate,
                 ex_coupon: Vec<Option<ExCouponTerms>>,
                 redemption: SimpleCashFlow, max_flows)
BondSettlement::new(date, payment_date_inclusion)
ExCouponTerms::new(cutoff_date, boundary: ExCouponBoundary)
BondYieldConvention::new(accrual: AccrualConvention, compounding: Compounding)
BondPrice::new(value_per_100_face, kind: BondPriceKind)
BondPricingPlan::new(&bond, settlement, &yield_convention, max_accrual_work)
```

`BondKind::{ZeroCoupon,FixedRate}`, `BondPriceKind::{Clean,Dirty}` and
`ExCouponBoundary::{OnOrAfter,After}` are explicit discrete choices with no
Default. Existing PaymentDateInclusion can be reused for settlement equality.
There is no inferred issuer, settlement lag, payment calendar or yield market.

A bond has one finite strictly positive constant face. Coupons use that same
face and one retained finite signed FixedCouponRate. Negative coupon rates
remain valid analytical inputs; negative/zero face is rejected. Redemption is
one separate finite strictly positive SimpleCashFlow, retained even when its
date equals the last coupon's date. Its amount is supplied explicitly: no
implicit par principal and no assumption that redemption equals face.
Zero-coupon bonds have no fixed coupon; their issue date, raw maturity and
payment date remain distinct. A zero coupon **rate** fixed bond is still a
fixed-rate definition, and still validates all accrual contexts.

The fixed factory owns the original Schedule, retaining its complete provenance
without an implicit clone. It builds exactly one plain coupon per raw interval,
payable at that interval's adjusted end. The ex-coupon vector must contain
exactly one entry per coupon, including explicit None entries. Count/shape/cap
checks precede accrual/allocation work. The final redemption payment may equal
or follow the last coupon payment; it cannot precede it. Its date is supplied
rather than inferred from a raw maturity. This supports an explicitly delayed
principal without inventing an extra adjustment rule. A market-specific factory
may impose a narrower equal-final-payment requirement.

Issue must not exceed raw maturity or redemption payment. The first raw accrual
start may precede issue (for example, supplied reopening terms); historical
coupons remain inspectable. Payments produced by a valid Schedule may precede
their raw end under an explicit convention. They are not silently moved again
or rejected merely for being early. A named market adapter must validate the
relationships its own formula needs.

The final coupon and redemption are separate flows with stable original indices;
never sort, merge or discard equal-date or zero-valued coupons. The supported
maximum remains Leg::MAX_FLOWS = 3,652,059, counting redemption, so a maximum
valid schedule contributes at most 3,652,058 coupons plus principal. This is a
resource policy. A public unrestricted arbitrary-cashflow Bond constructor
should not be slipped into F-BOND-01: Leg already serves generic flows, and an
unrestricted constructor could accidentally imply amortizing or multiple
redemption support belonging to F-BOND-02.

Borrowed inspectors expose issue, face, raw maturity, redemption, source
schedule (None for zero), coupon quote, coupons, each ex-coupon cutoff and
boundary, and kind. A source schedule is never rebuilt from adjusted dates.
Factory resource caps are not financial value metadata. Arc/ordinary Clone
allocation retains the established system-OOM qualification; growing library
vectors reserve fallibly.

## 3. Explicit settlement and entitlement

Every query supplies one `BondSettlement { date, payment_date_inclusion }`.
There is no today(), default trade date, legal transfer inference or automatic
calendar adjustment. Settlement before issue is a typed error. Settlement after
all payments is a valid zero-valued analytical state; yield and normalized risk
then return explicit degeneracy errors rather than a fabricated zero yield.

A payment is potentially eligible iff payment > settlement, or payment ==
settlement with Include. Redemption uses only that test; it is never removed by
a coupon ex-date. For an eligible coupon with supplied ex terms, it is detached
from the buyer iff settlement >= cutoff for OnOrAfter, or settlement > cutoff
for After. A detached coupon is absent from buyer PV even if its amount is
negative or zero. This is a date/role decision, not an amount-sign decision.

An ex cutoff must not follow its payment date. Do not require it to follow the
raw accrual start: a very short stub can have an explicitly supplied earlier
cutoff. In such a case the generic accrued definition below can be negative
before accrual starts; this is an inspectable consequence of the supplied
terms. No business-day lag is guessed. A future lag factory must retain the
source calendar, lag unit, anchor, adjustment order, equality boundary and
checked generated cutoff. The UK DMO boundary mentioned by the parent uses a
strictly-after distinction; choose After explicitly if that source interpretation
is adopted. No UK holiday calendar currently exists in this crate, so a claimed
UK seven-business-day implementation cannot be inferred from WeekendsOnly.

Tests must pin cutoff-1/cutoff/cutoff+1 independently of payment equality,
including cutoff==payment, zero coupons, negative coupons and redemption on the
same date. Entitlement is retained in a pricing plan and exposed by flow index
for audit; it is never destructively applied to instrument terms.

## 4. Proposed generic accrued and clean/dirty definitions

For a coupon with raw [a,b), full rounded amount A, and settlement S, define
its earned amount E(S) as +0 for S<=a, the original-convention simple coupon
product N*r*YF(a,S) for a<S<b, and cached A for S>=b. Validate every required
context before zero shortcuts, as in M7. This is not full-coupon scaling by
elapsed calendar days, nor YF(a,b)-YF(S,b), since conventions need not be
additive and rounding paths differ.

The proposed local bond accrued adjustment is summed over **payment-eligible**
coupons, with term E(S) for buyer-entitled coupons and exact_sum([E(S),-A])
for detached coupons. Ceased coupons contribute zero. Redemption contributes
zero. Thus ex-coupon accrual may be negative. Multiple delayed unpaid coupons
can contribute full earned amounts simultaneously; an early payment can end
eligibility before raw accrual ends. These choices must be recorded as a
local engineering contract, not described as universal bond market practice.
A named market kernel must validate its coupon-day/reset/ex-dividend mapping
against its own complete primary source and worked examples.

Buyer dirty amount D is PV at settlement of all buyer-entitled eligible
coupons plus eligible redemption. Generic accrued adjustment AI is independent
of discount yield. Clean amount is exact_sum([D,-AI]). Ex-detachment changes
D and AI together; it does not erase the source coupon. Cash-flow payment
inclusion and ex cutoff equality are independently explicit.

Return a `BondValuation` with finite amount-unit D, AI and clean amount, plus
face and settlement provenance. Conversion to prices per 100 is a separate
fallible operation so a finite amount result need not fail because its tiny
face makes the normalized quote unrepresentable. Given rounded amount X, use
one correctly rounded exact binary rational 100*X/face. Convert a supplied
price per 100 back using one correctly rounded exact rational face*price/100.
Avoid intermediate amount/face underflow or 100*amount overflow. A private
bounded-integer quotient helper needs its own Fraction oracle, finite-domain
proof and range/error tests; this is binary64 analytics, not decimal money.

Choose and document the rounding direction once: calculate clean in amount
units, then normalize each returned amount independently. Independently rounded
clean_per_100 and dirty_per_100-accrued_per_100 need not be bitwise identical.
Likewise converting a rounded quoted clean target to dirty amount adds rounded
AI once; do not claim that every round trip is exact. Mathematical clean/dirty
identities have propagated rounding tests; exact assertions use representable
hand cases.

## 5. Two-stage generic pricing plan and work budget

`BondPricingPlan` borrows the immutable Bond and BondYieldConvention and stores
only bounded derived horizons/eligibility and cached accrued results. Its
lifetime prevents dangling references. Plan construction, not instrument
construction, selects settlement and discount context. It retains the complete
convention and settlement; no context is rebuilt from a guessed frequency.

Proposed deterministic preparation order:

1. Validate settlement and plan count/cap inputs.
2. Charge and validate one equal-date discount anchor at S, even if all flows
   are excluded or their amounts are zero.
3. Traverse original coupon/redemption order. For a payment-eligible coupon,
   compute any necessary partial earned accrual first, then detached adjustment.
   For each buyer-entitled payment, charge and validate the direct S-to-payment
   accrual horizon, including a zero amount. A detached coupon does no discount
   query, but its required partial accrued query still runs.
4. Reduce accrued terms by exact_sum and finish the immutable plan.

A query charge is one plus ICMA overlap periods, or one plus absolute calendar
span for Business/252; other current conventions charge one. Structural,
ceased and cached paths charge zero, consistently with M7. One shared u64 ledger
covers both coupon and discount queries, charges before each query and reports
original index/phase/used/required total. Calendar tree/override searches and
binary partition costs remain additional declared factors. Plan preparation
can use O(n+budget) interval work; repeated direct ICMA prefixes are not claimed
unconditionally linear. Derive a fresh safe cumulative bound for both kinds of
query; do not copy M7's one-query-per-flow bound unchanged.

All horizons are prepared once, so bounded yield iterations need not rescan
Business/252 days, clone large ICMA contexts or redo entitlement/accrual. One
convenience price call can prepare then evaluate, with exactly this two-stage
failure order: date/context preparation errors precede all yield-factor errors.
This is explicit new API behavior; it does not alter existing M7 failure order.

A private shared rate kernel taking (rate, Compounding, time) should be factored
from InterestRate and reused. Do not create a dummy Actual/365 quote for its
factor methods or repeatedly clone full AccrualConvention inside callbacks.
Refactoring needs complete M4/M7 parity tests. Plan evaluation preserves M4's
factor-required and NumericalBoundary domain: an individually invalid factor
or product is not rescued by later cancellation. Each horizon is discounted
directly from S; do not chain adjacent factors for simple/hybrid compounding.

Finite rounded PV terms reduce by exact_sum, giving the exact sum of supplied
rounded terms with one final rounding, not an exact dot product. Zero amounts
still validate the factor. Excluded/detached flows create no factor trial.
All finite signs of yield are accepted where the retained method's domain and
representability guards allow them.

## 6. Yield inversion and what a root proves

Propose `plan.yield_from_price(target: BondPrice, bracket: RootBracket,
options: SolverOptions) -> Result<BondYieldReport,BondError>`. Target kind is
mandatory. Convert a clean target using this plan's AI; do not recompute another
settlement/context or silently reinterpret clean as dirty. Target prices are
finite signed values, not artificially restricted positive when signed coupons
are allowed.

The bracket is finite and explicit. No guessed initial yield, automatic bracket
expansion, default accuracy, hidden iteration count or unbounded loop is
allowed. Both endpoints must be valid for the entire pricing plan. Preserve
M6 endpoint order, callback counts, exact callback-zero versus bracket-tolerance
termination and adjacent-float exhaustion. A small price residual is not a
standalone stopping rule. Every callback costs O(n) finite factor/product work;
options.max_evaluations bounds total trials, and date work was charged once.

The current M6 f64-only callback cannot return a BondError. Recommended design:
extract a private fallible Brent core returning either SolverError or a typed
callback error with stage, trial yield and actual counts; implement existing
infallible public brent through an Infallible lift. Bond analytics uses that
same core. This preserves one numerical algorithm and requires regression tests
for all current solver output/count/failure semantics. Do not communicate a
pricing failure as NaN/infinity, discard its typed cause, or return the solver's
apparently successful report after a failed callback. A separately proved
abort-latch adapter is an alternative, but not an adopted shortcut.

The callback residual is a documented computed price-minus-target operation.
Its representability and rounding must be treated explicitly: subtracting two
finite values can overflow. A typed residual-range error is preferable to a
NaN sentinel. A normalized residual alternative needs a separate sign/zero,
continuity, interpolation and error-bound proof; changing residual scale during
iterations cannot be silently hidden behind an unqualified M6 equivalence claim.
Retain the final repriced amount, target basis and financial-unit residual
where representable, alongside the underlying SolveReport.

For nonnegative buyer-entitled cash flows with at least one positive payment
whose prepared horizon t>0, all supported fixed-time discount branches are
strictly decreasing in yield inside their common mathematical domain. This
provides a **conditional mathematical uniqueness** certificate; fixed-time
zero-horizon terms are constants. Equal civil dates are not the only way t can
be zero (30/360 can collapse distinct dates). Remove neither zero terms nor
contexts when determining this condition.

Negative coupons permit multiple roots, no roots, and zero/negative prices.
A bracketed solve may still return a root in its supplied bracket, but must
mark global uniqueness uncertified. Do not reject all signed instruments merely
to advertise uniqueness. If all entitled nonzero flows have t=0, distinguish
an identically matching target (every yield, no unique yield) from a different
constant target (no yield). A fully paid/ex-detached plan has the same problem.
Return typed `NoYieldSensitivity`/`ConstantPriceMismatch` (final names pending),
not zero yield. Finite-domain computed zeros and libm accuracy never prove
mathematical exactness, continuity or uniqueness by themselves.

## 7. Duration, convexity and branch definitions

For the generic plan at fixed settlement, dates, amounts, entitlement and
compounding, let P(y)=sum A_i*d_i(y) denote the real-valued model with its
supplied rounded binary64 amounts and prepared times held fixed. Analytic
derivatives refer to this model, not the discontinuous finite set of rounded
program outputs. Use **dirty** buyer value for conventional
normalization, never a clean price that subtracts yield-independent accrued.
Return a `BondRisk` with P, P', P'', and explicitly named measures:

- Modified duration = -P'/P, in years when y is an annual decimal rate.
- Convexity = P''/P, with respect to that same decimal yield (no automatic /100
  or /10,000 factor). The factor 1/2 belongs in the second-order Taylor
  price-change term, not in this convexity definition.
- Macaulay time-weighted duration = sum(t_i*PV_i)/P using the plan's supplied
  time coordinate. This is an algebraic measure for signed flows, not always
  a convex weighted mean or a market-defined duration.
- First-order amount change for +1 bp = P'*1e-4; positive-loss amount DV01, if
  supplied, must be named and documented as -P'*1e-4, not confused with an
  actual finite repricing difference.

For zero P, normalized measures are undefined and return a typed error while
raw P'/P'' can remain available. Negative P or signed flows may produce signed
normalized values; do not silently apply an absolute denominator. A positive
price can have zero derivative because of cancellation; that is a valid risk
result, not necessarily a yield uniqueness claim.

Analytic branch equations, before numerical operation order is fixed:

| Active branch | d(y) | d'/d | d''/d |
|---|---|---|---|
| Simple | 1/(1+y*t) | -t/(1+y*t) | 2*t*t/(1+y*t)^2 |
| Continuous | exp(-y*t) | -t | t*t |
| Periodic m | (1+y/m)^(-m*t) | -t/(1+y/m) | t*(t+1/m)/(1+y/m)^2 |

At t=0 derivatives are exactly +0 after normal quote-domain validation. A
hybrid uses the same rounded fixed-time branch selection as InterestRate for
EACH payment. The branch is independent of trial y, so yield derivatives are
smooth inside that selected domain. The hybrid's switch can be nondifferentiable
with respect to time/date variation; no universal time derivative or date
finite difference is promised. At exact t=1/m the two expressions agree, but
the existing switch compares to rounded binary64 1/m; adjacent-time tests and
negative-base conditioning remain necessary. Never describe a fixed-time yield
derivative as a time derivative through that switch.

Macaulay=(1+y/m)*Modified is justified for a pure periodic common-m kernel;
Macaulay=Modified for pure continuous. Neither identity is universal for Simple
or a mixture of hybrid branches. Named Treasury/DMO kernels need derivatives
of their ACTUAL formulas; replacing a fractional-first simple term with a
periodic power gives different price, yield and risk.

Document stable evaluation for derivative terms separately. Avoid materializing
an overflowing t*t or inverse-base square when the full weighted term is finite.
Use bounded exact products/ratios or scaling with its own proof. Risk may fail
representability even when pricing succeeds; pricing should not compute every
risk term eagerly. Reduce finite rounded derivative/time-weight terms by
exact_sum and record that this is not an exact dot product. Analytic derivatives
are the definition; independently converged finite differences are a check,
not the production definition or a blanket error certificate.

## 8. Named market yield conventions are distinct work

A generic `BondYieldConvention { accrual, compounding }` is deliberately named
as a direct-horizon analytical convention. It must not be marketed as US
Treasury, UK gilt, Canadian, street or universal yield-to-maturity. F-BOND-01
still needs selected fully sourced market example coverage; the roadmap's high
risk comes from those conventions.

A named market context must retain source/version and complete determination
periods, frequency, irregular first/last coupon roles, settlement equality,
ex-coupon status, redemption treatment, yield units/domain and rounding rules.
It may compile its own immutable weighted-factor plan rather than reuse direct
YF(S,payment). Use a distinct constructor/type or explicit enum variant that
cannot be created with missing data. Reject unsupported instruments with typed
capability errors; do not approximate them by the generic kernel.

In particular, validate on-coupon settlement, fractional first period, last
period, short/long stubs, ex-dividend intervals, negative yields and printed
price precision independently. A primary source that supplies only regular
semiannual formulas does not authorize all irregular dates or all instruments.
The parent must decide which source-complete market kernels are feasible for
this tranche and retain any remaining M8 gate honestly; this proposal makes no
financial adoption or completeness assertion.

## 9. Errors, bounds and numerical evidence

Propose non-exhaustive BondError with stable kinds for invalid face/redemption,
issue/maturity/settlement relation, ex-vector shape/cutoff, redemption ordering,
flow/work cap, allocation, unsupported market instrument, zero price risk,
constant-price yield degeneracy, numeric value/range, contextual cash-flow/rate,
reduction and solver/callback failures. Preserve typed source chains, original
flow index, operation phase, trial yield and actual work/evaluation counts.
Do not flatten RateError or callback failure into a string or NotBracketed.

Specify constructor and query validation order before coding; the proposal's
order above is intended to make count/resource/anchor/index tests executable.
No partial plan, partial bond, partial risk or successful root escapes after an
error. MIN/MAX arithmetic uses checked Date/Calendar functions. Explicit dates
need no synthetic day before MIN/day after MAX. A future settlement-lag helper
must distinguish calendar-day versus business-day lag, preserve TARGET policy
and overrides, and propagate Date/Calendar errors rather than silently clamp.

Current exactness classes remain unchanged. Integer dates/entitlement/indices
are E0. Binary price conversion is a separately proved correctly rounded
rational, tested 0 ULP against Fraction. Coupon amounts keep their supplied
binary64 product contract. Generic ideal price/derivative errors propagate
absolute per-flow accrual, factor, product and reduction uncertainty with gross
conditioning; net-relative or universal ULP comparisons cannot certify heavy
cancellation. Named market formulas need their own numerical profiles and
printed-example rounding evidence. Yield-coordinate tolerance and price
residual uncertainty are separate; near P'=0 an apparently small residual is
weak yield evidence. No existing tolerance is widened to make parity pass.

## 10. Proposed traceability and test program

The identifiers below are **proposal identifiers**. They are not existing
accepted FR-BOND/TS-BOND requirements; promote/reconcile them into REQUIREMENTS
only when a local contract is adopted. Tests must also cite relevant existing
FR-CF, FR-RATE, FR-DISC, NP and solver obligations.

| Proposal | Behavior | Independent evidence and adversarial checks |
|---|---|---|
| P-BOND-01 | Zero/fixed bullet terms, positive face/redemption, signed coupon, retained provenance | Constructor error precedence; NaN/infinities/signed zero/subnormal/MAX; source schedule and contexts; zero-coupon-rate versus zero-coupon identity; late reopening issue |
| P-BOND-02 | Raw accrual and adjusted payment remain distinct | All 49 ordinary/termination conventions, early/delayed payment, stubs/EOM and each schedule family, same-date principal/coupon retained; source raw dates never rebuilt |
| P-BOND-03 | Explicit settlement equality and ex cutoff boundary | Full cutoff-1/cutoff/cutoff+1 x payment Include/Exclude x After/OnOrAfter matrix; None, cutoff==payment, cutoff before raw start, negative/zero coupon, redemption unaffected |
| P-BOND-04 | Generic ex-adjusted earned accrual and clean/dirty amount identity | Independent civil/Fraction models of clipped partial accrual, actual/ICMA/30-360/B252 nonadditivity, coupon-date resets, before/after issue, before/after final payment, delayed multiple accruals and negative AI |
| P-BOND-05 | Per100 conversion and inverse are robust exact binary rationals | Fraction fixture covering every exponent field, ties-even/sticky, signs, smallest subnormal, MAX, recoverable intermediates, exact zero, nonzero zero-underflow/overflow; no decimal-accuracy claim |
| P-BOND-06 | Direct-horizon flat buyer PV for all compounding | All 32 configurations x every supported accrual context; no chaining; unit coupon/redemption closed forms; factor boundaries; zero amounts still query; heavy signed cancellation |
| P-BOND-07 | Shared bounded preparation work | Exact required cap, cap-1, zero and u64::MAX; shared AI/discount accounting; ICMA overlap and B252 span independently counted; excluded/detached paths; full-range MIN/MAX; no repeated date work during yield iterations |
| P-BOND-08 | Finite bracketed yield with source-preserving failure | Independently known one-flow roots; two/many flow two-precision Decimal roots; endpoint root, bracket tolerance, adjacent floats, cap1/cap2/exact cap, invalid interior, callback source/index/stage; no automatic expansion |
| P-BOND-09 | Conditional uniqueness and degeneracy | Nonnegative positive-time certificate; future civil dates with zero 30/360 horizon; no payments, constant target/mismatch; signed cash flows with independently constructed multiple roots and flat slopes; bracket-specific outputs never claim global uniqueness |
| P-BOND-10 | Analytic first/second derivatives and named risk units | Fraction simple/periodic integer-exponent cases, two-precision Decimal general cases, converged symmetric/one-sided finite differences, zero/negative P, cancellation, near-zero yield, near-domain boundary, huge t with finite scaled product |
| P-BOND-11 | Hybrid and duration qualifications | Both switch neighbors for all m, exact rational-versus-rounded threshold classes, mixed branch plans, counterexamples to inappropriate Macaulay identities, distinguish time kink from yield derivative |
| P-BOND-12 | Market-specific kernels only within sourced capabilities | Every printed worked example and rounding instruction; fractional-first/final periods, regular/irregular eligibility, ex equality and redemption; explicit capability rejection; independent source formula and frozen attested black-box classification |
| P-BOND-13 | Public contracts, error catalog, resource/ownership safety | Traits/docs/doctests, value-definition equality, shared contexts, immutable concurrent plans with independent settlements/flags, complete reachable error/source/format tables, max count; no forced OOM or forged private invalid states |
| P-BOND-14 | Reproducible validation and packaging | Independent generators --check, pinned contracts/source versions and reference wheel, fixtures with status/distribution/conditioning metadata, debug/release/strict lint/docs, fresh region coverage, actual extracted package checks, platform evidence scoped to execution versus compilation |

A feasible independent oracle should enumerate a complete 400-year civil cycle
for settlement/cutoff/payment boundary constructions, all supported-year leap and
century classes (years 1..9999, never year 0/10000), dense short
windows, maximum civil spans, every periodic frequency, all explicit flags and
hand anchors. A full Cartesian product of all dimensions is not automatically
useful: report exact profile counts and selected exhaustive subspaces, plus
fixed-seed deterministic wide combinations, rather than claim universal
exhaustiveness. At least one generator/test author should derive expected
financial/date results before reading production code.

Reference fixtures must be NEW and byte-attested; preserve all existing frozen
fixtures/generators. QuantLib public Python probes provide behavioral comparison
only, never the ground truth for a conflict with the adopted explicit contract.
Classify default settlement, inclusion, ex equality, coupon schedule, yield
convention and rounding differences separately; no broad tolerance absorbs a
semantic mismatch. Existing M4/M6/M7 tests remain required regression evidence.

## 11. Recommended implementation sequence

1. Finish and pin primary DMO/Treasury source reviews; decide complete local
   settlement/ex/accrued and numerical contracts, and the exact named market
   capability coverage. Record existing accepted ADR-0006 boundary explicitly.
2. Add authoritative requirements and proposed API/errors; independently author
   hand cases and oracle schemas before production inspection.
3. Implement immutable zero/fixed terms, settlement/entitlement and robust quote
   conversion; validate these before discount/yield complexity.
4. Add prepared generic direct-horizon price/accrual, shared rate kernel and
   complete work-ledger accounting without altering existing cash-flow APIs.
5. Add fallible shared bounded root core, yield reports/uniqueness qualification,
   and analytic risk with the correctly sourced derivative kernels.
6. Add each source-complete market kernel, its capability checks, independent
   oracle and printed literature examples; retain remaining named gaps.
7. Complete independent review, all checks, measured fresh coverage/package and
   source/reference discrepancy reports. Update living readiness accurately;
   local signed-off commit only after verified evidence. Formal acceptance and
   remote runtime CI remain distinct from this local tranche.
