# Independent date-coupled Canadian MM range closure — final v2

Repository pre-contract review, recorded 2026-10-23 Bangkok. Root promotes the immutable
private review with editorial formatting and this provenance preface; no contract, API,
requirement, oracle or Canadian production is adopted. Read the [complete local
proposal](M8_CANADIAN_LOCAL_DESIGN.md), [numeric/test
design](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md), [source
review](M8_CANADIAN_SOURCE_REVIEW.md) and [research
validation](M8_CANADIAN_RESEARCH_VALIDATION.md) with their separate scopes. The original
producer's no-repository-write statements below describe its private work; root owns
this new repository record.

Original final review SHA256
`e35abf857c0be2cab655c67a23a41718f2ba653adb2ea844b5700f2fc9da5ae8`; private evidence
`/private/tmp/sabiquant-canadian-mm-range-review-v1/evidence.json` SHA256
`6dfe26700c24e392a393c1abb168c80d0a8cc80b7f3a34cd6c60b80e4476c1e1`. The [compound
profile](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md) is a separate conditional/native
result; these exact MM range proofs do not assume libm accuracy.

PRIVATE, unadopted numerical review, 2026-10-23. No repository, Git, AGENTS, QuantLib
implementation, network or production Rust writes/reads were used. The input proposal
snapshots and their original paths/hashes are recorded in `arithmetic.json`. The
whole-domain argument is the proof below. The finite exact-Fraction controls corroborate
specific algebra/staging discriminators; they are not substituted for a universal proof,
Rust runtime tests, source acceptance, libm certification, compiler-stack measurements
or contract adoption.

The review closes the stated range questions without narrowing coupon frequencies,
odd-period geometry, decimal input scales, date caps, true payment delay or valid MM
quotes. Some proposed errors remain defensive but have no admitted dated MM witness.
Nonzero subnormal factor/PV/dirty values remain admitted.

## 1. Exact domain and selected observations

Use all the input proposal's admitted schedules, C in [0,2^127-1], decimal scales 0..38,
frequencies f=2,4,12, Q.len()<=20,000, S<M<=T, and finite binary64 y for which exact
d=1+y*tau>0. Write D=days(S,T), tau=D/365, payment horizon h_i=days(S,t_i)/365>0,
rho_i=days(t_i,T)/365>=0. Thus h_i+rho_i=tau and 0<=rho_i<tau. Terminal payment is at T;
its rho=0, h=tau and coefficient is B_T=100+payable_coupon_T>=100. Nonterminal B_i are
payable coupons. All B_i>=0.

Do not replace these exact affines with rounded products:

- d=1+y*tau; n_i=1+y*rho_i; r_i=n_i/d.
- rhat_i=RN_IEEE(r_i), one exact-rational-to-binary64 nearest/even observation.
- What_i=RN_IEEE(B_i*rhat_i), one exact product of source rational and represented
  factor, with canonical zero only when B_i=0.
- S_PV=sum What_i exactly, then Dhat=RN_IEEE(S_PV).
- Selected term seeds are these SAME What_i. Before extended53 rounding,
  L_i=-What_i*h_i/(d*n_i), K_i=2*What_i*tau*h_i/(d^2*n_i), and U_i=What_i*h_i.   Round
each whole rational once to 53 normalized significand bits with unbounded   retained
exponent, then sum each signed family exactly. Write retained sums L,K,U.
- Raw getters observe L,K,U once to IEEE. Normalized getters observe -L/Dhat,
  U/Dhat,K/Dhat once, independently. No rounded raw getter gates another.

The negative first derivative has no cancellation: all nonzero L_i have the same
negative sign. K_i and U_i are positive for every positive seed. All following sum
bounds therefore apply to magnitudes without cancellation qualifications.

## 2. Payable-source coupling, including odd periods

For f2,4,12 the real quasi-day ranges are respectively 181..184,89..92,28..31. A full
virtual period contributes C/f. A positive proper fragment contributes C*d/365 if
f*d<365, or C*(1/f-(s-d)/365) if f*d>365. The latter proper-fragment case is possible
only for f=2,s=184,d=183; its coefficient is 363*C/730. Thus, for C>0, every positive
fragment or full piece is at least C/365 and at most C/f; every proper fragment is
strictly below C/f. No assertion about arbitrary cross-frequency primitive supersets is
used here.

Every nonterminal actual coupon date is on Q. Consequently, summing payable coupons over
all actual accrual intervals partitions [D_accrual,M] over Q cells: there is at most one
used fragment/full piece per Q cell. Odd first/last periods can span many cells; they do
not multiply a cell's contribution. If both endpoints lie in one cell the one fragment
is used once, exactly as the proposal requires. Therefore, for any retained future
subset,

  sum B_i <= C*(Q.len()-1)/f + 100           <= (2^127-1)*19999/2 + 100 < 2^142.

For C=0 this sum is exactly 100; no false strict comparison with a zero coupon bound is
needed. For C>0, the smallest positive C is 10^-38>2^-127, so a positive nonterminal
B_i>=C/365>2^-136. Terminal B_T>=100>2^6. These facts cover short, long and composite
ends, not merely regular coupons. Zero source coupons are still traversed and have their
factors observed.

Valuation accrual A has the same finite Q-cell bound using exact Actual/Actual pieces;
settlement AI uses the preceding payable bound or the monthly A policy. Conservatively
A,AI<2^143 and their independently rounded binary64 caches each satisfy
0<=Ahat,AIhat<2^144. This accrual bound is independent of the yield branch.

## 3. Shared affine and factor bounds

The civil-date domain gives 1<=D<2^22 and 1/365<=tau<2^14. For y>=0,
1<=n_i<=d<1+MAX*2^14<2^1039, so 0<r_i<=1. For a nonterminal date, R_i=days(t_i,T)>=1 and

  (1+y*R_i/365)/(1+y*D/365) >= R_i/D > 2^-22.

The terminal factor 1/d is strictly above 2^-1039.

For y<0, first consider d>=1/2. Otherwise d<1/2 implies
|y|>365/(2D)>2^-15. A finite binary64 number of magnitude above 2^-15 is an
integer multiple of 2^-67. Hence positive exact 365+y*D is an integer multiple of 2^-67
and is >=2^-67. Thus d>=2^-67/365>2^-76, and the deliberately looser bound d>2^-80 holds
throughout the negative domain. Because rho_i<tau and y<0, n_i>d and n_i<=1.
Consequently 1<=r_i<2^80. The exact affine numerators cannot fail positivity after the
one exact shared-domain check.

Combining both signs,

  d,n_i>2^-80; d,n_i<2^1039;   2^-1039 < r_i < 2^80, except that r_i=1 is also admitted.

The endpoints 2^-1039 and 2^80 are finite exactly representable IEEE numbers. Monotonic
nearest/even rounding therefore gives

  2^-1039 <= rhat_i <= 2^80.

No factor can become zero or infinity, and positive y always has rhat_i<=1. For a
positive nonterminal source coefficient the stronger rhat_i>=2^-22 holds. This proof
allows nonzero subnormal terminal factors; demanding normal factors would incorrectly
narrow the model.

## 4. PV and dirty admission

A positive nonterminal exact source-factor product is >2^-136*2^-22=2^-158. The terminal
exact source-factor product is >=100*2^-1039>2^-1033. Thus every nonzero PV product is
well above half MIN_SUB and finite. Its direct IEEE nearest/even observation cannot be
zero. By monotonicity, terminal What_T is >=2^-1033. Nonterminal positive PVs are
normal. A zero source coupon alone produces canonical +0, after its admitted factor
check.

The global source sum and rhat_i<=2^80 give sum B_i*rhat_i<2^222. For these positive
products, direct nearest/even loses/gains less than a factor two. Therefore S_PV=sum
What_i<2^223. A final exact-sum observation yields

  2^-1033 <= Dhat <= 2^223.

In particular dirty is positive finite. Nonzero-subnormal dirty is admitted. The factor,
PV and dirty zero/infinity error paths have no admitted dated MM witness under this
proposal; keeping defensive checks does not make them actual coverage obligations with
fabricated financial examples.

## 5. All raw and normalized overflow exclusions

Let u=2^-53. Every nonzero extended53 RNE term is between (1-u) and (1+u) times its
exact magnitude, with no IEEE underflow admission at this stage. Using h_i<=tau<2^14 and
d,n_i>2^-80 gives the conservative strict bounds:

| Quantity before final IEEE observation | Bound |
|---|---|
| abs(L) | <2^398 |
| K | <2^493 |
| U | <2^238 |

For example, sum abs(L_i)<S_PV*2^14*2^160<2^397 and extended53 loss/gain is <2. The
second multiplier is <2*2^14*2^14*2^240=2^269, giving K<2^493. The moment multiplier is
<2^14, giving U<2^238. No raw getter can overflow. These are useful broad bounds, not
claims of dated extremal attainment.

To normalize, do not divide the last broad bound by an unrelated tiny dirty bound. S_PV
is an exact sum of IEEE values, hence an integer multiple of MIN_SUB. If S_PV is
subnormal it is exactly representable. If it is normal, ordinary IEEE RNE relative error
is at most u. Thus throughout this domain,

  (1-u)*S_PV <= Dhat <= (1+u)*S_PV,   S_PV/Dhat <=1/(1-u)<2.

Applying this coupling before normalizing yields

| Quantity before final IEEE observation | Bound |
|---|---|
| -L/Dhat (Modified) | <2^176 |
| K/Dhat (Convexity) | <2^271 |
| U/Dhat (Macaulay) | <2^16 |

None can overflow. This uses accepted-PV seeds and the rounded dirty denominator, not
ideal source aggregate H. All final upper bounds are exactly representable normal values
far below the IEEE overflow midpoint.

## 6. Raw moment and Macaulay cannot underflow

For the terminal source contribution, exact rho=0 and h=tau, and for every finite
admitted y<=MAX,

  B_T*tau/d >=100/(y+1/tau)              >=100/(MAX+365) >50/MAX >2^-1019.

The denominators are positive because d>0. This also includes negative y; the MAX+365
comparison is deliberately generous. Each of the admitted factor IEEE rounding, admitted
PV IEEE rounding and extended53 moment-term rounding preserves strictly more than half
its positive input. Hence the retained TERMINAL moment term is > (50/MAX)/8 >2^-1022.
Every other moment term is nonnegative. The exact retained sum U is therefore above
MIN_NORMAL, and its final IEEE observation is normal. There is no raw moment underflow
or overflow.

A tighter, independently checked margin is available: factor relative loss is at most
2^-36 because r>2^-1039, PV relative loss at most 2^-42 because its terminal input is
>2^-1033, and extended53 relative loss at most u. This gives

  U >= (50/MAX)*(1-2^-36)*(1-2^-42)*(1-u) >2^-1020.

The simple three-half proof is already sufficient; the tighter constants are not
necessary hidden numerical-profile assumptions.

For the selected normalized moment, h_i>=1/365 and positive same-sign terms give

  U/Dhat >= ((1-u)/(1+u))*(1/365) >1/730.

This is the bound BEFORE the final IEEE observation. That last normal IEEE round
contributes a further relative factor (1-u), so the actual returned Macaulay is
>=(1-u)^2/(1+u)/365>1/730. Combined with U/Dhat<2^16, the final Macaulay getter is
normal finite.

The stronger exact inequality 'observed Macaulay>=1/365' is FALSE. The ideal source
moment obeys it, but the selected extended53 term followed by IEEE normalization need
not. A fully admitted one-day C0 MM instrument at y=-280 has accepted PV bits
407ad69696969696 and selected observed Macaulay bits 3f66719f36016719, which is below
exact 1/365 by 347/841632698362998292480. This does not threaten the underflow
exclusion; it prevents silently promoting an ideal lower bound into an observation
theorem.

## 7. Derivative underflow remains genuine and independent

No zero/inf factor or PV gate prevents all derivative underflows. The exact controls
reconstruct the selected staging and record:

- C0, one day, y=MAX: raw first and second derivatives round to signed/positive
  zero, while raw moment and normalized Macaulay are normal. Modified is nonzero
subnormal. Such nonzero-to-zero candidates are the named getter Value errors,   not
returned signed-zero successes.
- f2 C=2^127-1, accrual/issue2025-03-01, Q/coupons2025-09-01 and2026-03-01,
  S2025-06-01, T=M2026-03-01, y=2^540: raw first derivative is normal
(845c99d7ae5bf3d0) while Modified rounds zero. Both PVs are admitted.
- The same dated instrument at y=2^370: raw second derivative is normal
  (028c99d7ae5bf3d0) while Convexity rounds zero.

These are exact-Fraction checks of the NEW selected observation protocol, not Canadian
Rust runtime claims. Their Macaulay observation ends in ...021a; the earlier independent
IDEAL-MM table ends in ...021b. That one-bit distinction is expected because the
selected model uses accepted IEEE PVs and rounded terms. No ideal table is silently
rewritten to match selected observations.

## 8. Explicit admitted subnormal example and complete inputs

Set C0, f2, D_accrual=I=S0001-01-01, Q=[0001-01-01,0001-07-01], actual
coupon=[M0001-07-01], T9999-12-31, and y=MAX. These are valid complete schedule/source
inputs; terminal payment delay is explicitly supplied. The selected factor is nonzero
subnormal 0000001a331acd53, and PV/model-dirty is nonzero subnormal 00000a3bf678346c.
Raw moment is normal 00590000000026b1. No shortened delay or normal-only factor
restriction was imposed.

The one-day C0 examples in sections6/7 have f2, D_accrual=I=S2025-06-01,
Q=[2025-01-01,2025-07-01], actual coupon=[M2025-06-02], and T=M. This is the proposed
admitted single Composite interval, strictly inside one Q period; C0 payable amount is
exactly zero, terminal source100. For y=-300 or -280, exact365+y is positive even though
y is below every compound floor.

The two-flow CMAX examples use Q=[2025-03-01,2025-09-01,2026-03-01], actual coupons at
the latter two dates, f2, EndOfMonth Ignore, and regular payable C/2. The 117
scale/frequency controls use D_accrual=I=S2025-03-01; Q and actual coupon are the
adjacent regular next dates2025-09-01(f2),2025-06-01(f4),2025-04-01(f12), T=M,
C=10^-scale for each scale0..38. No unsupported frequency is introduced. The seventh
distinguished control retains both zero-valued actual coupons on the CMAX examples' f2
schedule; the nonterminal factor is still observed.

## 9. Cross-branch clean/bridge/target-shift closure

This section applies after an admitted positive finite Dhat for EITHER compound or MM.
It does not prove compound factor/PV admission or any elementary profile. It also
applies to any finite signed typed target scalar v. The independently rounded source
caches satisfy 0<=Ahat,AIhat<2^144. Thus the magnitude of any chosen one/two-cache
perturbation is <2^145. Every exact transformation in the proposal is a sum of at most v
or Dhat and these caches with the prescribed signs.

For IEEE binary64,

  MAX=2^1024-2^971,   overflow_RNE_midpoint=2^1024-2^970,
overflow_RNE_midpoint-MAX=2^970.

Because 2^145 <<2^970, the exact transformed magnitude is strictly below the
zero-to-infinity overflow midpoint even when the starting scalar is +/-MAX. One direct
IEEE RNE observation is finite. Also every summand is an integer multiple of MIN_SUB, so
their EXACT sum is either zero or has magnitude at least MIN_SUB. It cannot be a
nonzero-to-zero rounding. Exact cancellation becomes canonical +0. This does not justify
clean-then-add or any other double rounding.

Consequences: compound/MM clean and settlement bridge range errors are proof-excluded
under the chosen one-exact-sum staging. Target-shift range errors are similarly excluded
for finite CanadianPrice inputs. Resolved target positivity remains a real check: signed
finite Clean/SettlementDirty targets can resolve nonpositive, positive, or zero. The
result is not an invoice equality theorem.

## 10. amount_residual correction

After a successful solver trial, Dhat and the resolved model-dirty target are both
positive finite binary64 values. Their exact difference has magnitude below MAX and lies
on the MIN_SUB lattice. Its one direct IEEE nearest/even observation is finite and zero
iff the two inputs are equal. It CANNOT overflow, and cannot have a nonzero-to-zero
error. Keep an independently fallible defensive API if consistent with project style,
but the proposal sentence claiming this observed unscaled residual 'can overflow' must
be removed as mathematically false. This says nothing about arbitrary signed generic
price residuals elsewhere.

## 11. Actual evidence and limits

`derive.py` uses only Python standard-library Fraction, integer arithmetic, dates,
hashing and bit serialization. It writes outputs with exclusive creation; it does not
overwrite a prior report. Its direct IEEE RNE helper handles subnormal ties without
first rounding to extended53. Known even/odd/overflow ties and extended53
unbounded-exponent controls are explicit.

The actual run passed 19 exact constant checks; 15 helper controls (11 IEEE and 4
extended53); 51 exact negative-affine floor-neighbor controls over17 day counts; 124
selected-staging dated math cases (7 distinguished plus117 scale/frequency); 140 finite
signed bridge/target primitive controls; and25 positive residual controls. Bridge
extrema are explicitly primitive supersets rather than claimed dated source states. The
three tables of exponent bounds and whole-domain exclusions come from the universal
proofs above, not the size of these finite sets.

An additional independent checker, `final-controls.py`, passed all1,198 proper fragment
(f,s,d) controls over the three real frequency-specific day ranges, confirmed the sole
late proper case(2,184,183), and cross-checked1,024 direct IEEE helper results against
CPython Fraction-to-float, including signs and underflow/overflow outcomes. It
explicitly proves the final Macaulay IEEE margin. The initial arithmetic field
`observed_mac_correction` called its pre-final lower-bound expression a selected bound;
the exact expression there applies before final IEEE admission. `final-controls.json`
records the staging clarification and the final bound above. The initial run and report
remain unchanged; no arithmetic failure is hidden or overwritten.

The first run's arithmetic SHA256 is
8329ef00d6b2fbde7564e17bd4ba6fef7156b9dd0fbb92e69c4c1e8b4f321094. No numerical Canadian
implementation, actual Rust allocation/layout, public error transport, complete cash
snapshot geometry, compound elementary certification, foreign-platform runtime,
financial acceptance or release gate is claimed here.

## Root addendum: an observed MM target below the ideal image

This later exact selected-stage control preserves the preceding independent review.
It uses C1 per100, f2, D=I2025-03-01, S2025-06-01,
Q=[2025-03-01,2025-09-01,2026-03-01], actual coupons at the last two dates,
EndOfMonth Ignore, M=T2026-03-01, date cap3, bundle cap2 and explicit MM quotation.
There are two future payable coefficients0.5 and100.5. At y=MAX the selected
model-dirty observation has bits `3fd5375375375375`, strictly below the ideal
lower price limit `181/546` by `59/4917930793088581632`. The exact ideal price
at that same finite y remains strictly above its ideal limit.

All five exact rational-to-IEEE stages (two factors, two PVs, one exact PV
reduction) lie strictly inside their represented nearest cells. Both terminal
factor and PV are admitted; the tiny terminal contribution does not move the
normal first-PV dirty observation. This is selected-observation mathematics,
not an execution of a Canadian library. It demonstrates why an ideal-image
precheck would reject a valid observed target and why an ideal inverse need
not exist for every finite observed price.

A future protocol-only public test can use this ModelDirty target and bracket
[0,MAX]: lower observed price is higher, upper is the exact selected target,
so the shared solver should return cached upper-zero success after2 function
and0 derivative calls. That would test solver transport, not ideal-coordinate
accuracy. It must not be counted as a conditioned ideal inverse reference.

Create-new research source:
`/private/tmp/sabiquant-canadian-mm-observed-image-v1/derive.py`, SHA256
`d0b03bccac66abb2ba3835f4af8ea035f19e04cc5a361cf8d201a5a82abb7e4f`.
Actual five-cell evidence SHA256
`c306118cb5645e22b5340ca28810814a8b844c36c370b11da52a59880d65bac2`;
complete-definition/solver-scope metadata SHA256
`bd6dcd3fe0c2a224331c2c881d1fa8822df8414db03c0de10e5fa6afd955b05f`.
The original evidence and script remain unchanged; the separate metadata
supplies the explicit EOM/cap/policy/target choices. No old ideal expectation
or fixture is rewritten to accept this different observation.

## Independent observed-image challenge

A separate reviewer reproduced the unchanged witness and obtained byte-identical
evidence, then independently checked18 complete-input predicates and all five
nearest/even stages by integer quotient/remainder and strict rational cells.
Review `/private/tmp/sabiquant-canadian-mm-image-review-v1/review.md` SHA256
`908da3f2fa261b11094a40001065b788ed739288057d01fcea72027c7e9c4fcc`;
authoritative `final-evidence.json` SHA256
`fd28688a2c8ee65ea6bff4059cb8d3b5b5cd7ef3f4344c56a1601acd01ec0f98`.
No substantive flaw was found. The formal ideal inverse is
`-661921091825809883695/59` and gives negative d, outside the MM domain.

The future observed-protocol fixture must additionally pin
`RootBracket::new(0.0, f64::MAX)` and
`SolverOptions::new(1e-12, 0.0, 2)` (absolute tolerance bits3d719799812dea11,
relative tolerance bits0000000000000000). The original complete-definition
metadata did not select solver options; it is retained unchanged. A one-call
budget would not reach upper. With these valid two-call options, the expected
cached upper FunctionZero is a protocol assertion with two function/zero
derivative calls, without an ideal-root accuracy assertion or hidden final
callback. This is still future Rust behavior, not measured Canadian runtime.
