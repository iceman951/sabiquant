# Canadian bounded numerical resource review

Unadopted independent review promoted2026-10-23 Bangkok. Original review
SHA256 `32ff6a31aec8b7b5229b055856087e9a01737ad6a96921dbacf7b896aec6b5dc`;
evidence SHA256 `1a5979a2cf2e932e79b511f3f1a44cc768e1f1fa4bedea89b8c4bf7b0fdb41db`.
Private artifacts are retained in
`/private/tmp/sabiquant-canadian-numeric-resource-review-v1/`.
The original text follows unchanged below. Its separate outward-guard gate
has since received the [bounded guard design](M8_CANADIAN_COMPOUND_GUARD_DESIGN.md);
full cash-resource integration and actual future Rust lifetime/layout auditing
remain separate. Read with the [complete local design](M8_CANADIAN_LOCAL_DESIGN.md),
[numeric/test companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md) and
[date-coupled MM range review](M8_CANADIAN_MM_RANGE_REVIEW.md).

## Original independent review

PRIVATE, unadopted independent mathematical review, 2026-10-23.

This proves available bounded arithmetic routes for the proposed MM factors,
selected MM/compound risk terms, direct source PV observation and retained-risk
normalization. It does not claim an implementation exists, count actual Rust
allocations/copies, measure compiler stack frames, or approve every unrelated
cash/compound-guard helper. No repository/Git/AGENTS writes, network access or
QuantLib implementation access were used. Own SabiQuant `src/math/sum.rs` was
read as design context for its two fixed arrays; it was not modified.

Use this with the separate exact MM range closure. Finite integer controls below
corroborate boundary arithmetic; the universal bounds and algorithm restrictions
are the proof. All existing C input coefficients/scales0..38 remain admitted.
No hidden requirement that C/f or source amounts fit DecimalAmount is added.

## 1. Common representation and rounding restrictions

For a positive admitted binary64 value write W=s_W*2^e_W, with
1<=s_W<2^53 and -1074<=e_W<=971. A subnormal W uses its integer fraction as
s_W and e_W=-1074. Keep the exponent as a small signed integer. Do not expand W
onto a universal large integer grid merely to form a small ratio.

A quoted capacity is the capacity of EACH unsigned magnitude. A source pair N/D
can own two five-word integers (80 bytes), not an assumed total40-byte ratio.
Signs, bit lengths and retained exponents are scalar metadata. Source/coordinate
cache inputs are borrowed; large arithmetic outputs are written by reference.
Do not use a helper that returns/accepts a 112-/243-word integer by value while
silently omitting those copies from the liveness census.

Selected risk terms round once to an extended53 significand with separate
unbounded IEEE-range exponent. Source PVs and final IEEE getters round DIRECTLY
to IEEE, including subnormal ties. An extended53-then-IEEE double rounding is
not equivalent and is not part of this design.

## 2. Relative53 division lemma

Let N,D be positive integers. Determine k=floor(log2(N/D)) from bit-length
difference and one exact shifted comparison, without converting N/D to float.
For a normalized53 observation form

  t=52-k;
  U=N*2^t,V=D          when t>=0;
  U=N,V=D*2^(-t)      when t<0.

Then 2^52<=U/V<2^53. Divide exactly and round q=floor(U/V) according to
2*remainder versus V and the quotient's parity. The rounded q has at most54 bits;
if q=2^53, shift it right once and increment the retained exponent. This is
RNE carry into the next binade, not a range failure. The separate input exponent
e is added to k-52 only after this operation; it does not size an allocation.

When t>=0, U<D*2^53, so bit_len(U)<=bit_len(D)+53. When t<0,
V<=N/2^52, so V is no wider than N. Bit-length comparison needs at most one
additional carry/leading bit. Twice the remainder is below2V. Thus a safe
scratch-width bound is

  max(bit_len(N), bit_len(D)+53)+1.

The +1 includes all conservative comparison/carry margins. The result is a
relative alignment bound. Adding an unrelated absolute exponent to an operand
bit count would give an irrelevant much larger bound.

A division specialized to this normalized quotient need scan only53 quotient
bits, j=52..0: compare the mutable U remainder with V<<j, subtract in place if
large enough, and set the corresponding u64 quotient bit. Shifted comparisons
can inspect borrowed limbs rather than construct more arrays. Final2*remainder
comparison also fits the bounds. Subnormal direct IEEE division has at most52
floor quotient bits, followed by a possible carry into MIN_NORMAL. No quotient
array, recursive division, value-dependent retry, or exponent-sized loop is
needed. This is an explicit algorithm recommendation, not an assertion about
unspecified general division code.

## 3. Forty-word MM affines and factor observation

Let E=2^1074 and Y=y*E, an exact signed integer for every finite binary64 y.
Since |y|<2^1024, |Y|<2^2098. Civil-day integers D=days(S,T),R=days(t_i,T)
satisfy D<2^22 and 0<=R<D. Compute unsigned positive/negative branches of

  d_raw=365E+Y*D,
  n_raw=365E+Y*R.

A negative Y uses sign-aware subtraction and an exact positivity check, not
wrapping unsigned subtraction. Every product magnitude is <2^2120 and the
added365E<2^1083. The proposal's looser |affine|<2^2122 therefore has spare
headroom, including before invalid negative-y domain rejection. Shared admitted
d_raw>0 implies n_raw>0. Affine generation is a40-word small-integer multiply,
add/subtract and sign check, with no allocation.

Exact factor n_raw/d_raw has N,D<2^2122. Relative53 alignment requires fewer
than2176 bits including carry, below40*64=2560. For DIRECT subnormal IEEE
observation one must instead round (n_raw*2^1074)/d_raw once. It is incorrect
to reject this as an unconditional2122+1074-bit product: the subnormal branch
condition itself says

  n_raw*2^1074 < d_raw*2^52,

so the actual numerator has at most2174 bits. Underflow/overflow classification
can first use k and the IEEE exponent thresholds without any large absolute
shift. All branches fit40 words. The separate MM range proof shows admitted
factors cannot zero/overflow, while nonzero subnormals remain admitted; the
capacity argument here also covers broader primitive ratio controls honestly.

## 4. Exact selected MM term forms fit112 words

Keep W=s_W*2^e_W, H=days(S,t_i), D=days(S,T), and positive raw affines d_r,n_r.
Because d=d_r/(365E), n=n_r/(365E), h=H/365 and tau=D/365, the selected exact
terms before extended53 observation can be constructed as follows:

| Term | Unsigned integer numerator | Unsigned denominator | Separate power |
|---|---|---|---|
| abs(first) | s_W*H*365*E^2 | d_r*n_r | 2^e_W |
| second | 2*s_W*D*H*365*E^3 | d_r^2*n_r | 2^e_W |
| moment | s_W*H | 365 | 2^e_W |

The first term's sign is negative, the others positive. Zero source/PV seeds
produce canonical zero terms after their required factor/PV checks. There is
no source-aggregate-H substitution and no independently rounded d,n,h,tau.

The following strict universal magnitude bounds include the displayed powers
of E; they do not require cancelling factors of two or a gcd:

| Magnitude | Bound |
|---|---|
| First numerator | <2^(53+22+9+2148)=2^2232 |
| First denominator | <2^4244 |
| Second numerator | <2^(1+53+22+22+9+3222)=2^3329 |
| Second denominator | <2^6366 |
| Moment numerator/denominator | <2^75 / 2^9 respectively |

Construct the E^2/E^3 factors by fixed shifts2148/3222 of the small product;
no quote-dependent exponent is shifted into these integers. Construct d_r^2
and then multiply by n_r. Every partial product is smaller than the listed
final positive product. The relative53 lemma gives a safe maximum
6366+53+1=6420 bits, below6500 and below112*64=7168. This includes the
normalized numerator, denominator, remainder comparison and RNE carry. The
first term's corresponding coarse ceiling is4298 bits. No whole-value float
or huge dyadic expansion is used. Positive raw-affine products have no sign
cancellation or hidden denominator-zero branch after domain admission.

## 5. Compound selected terms and five-word PV route

Let exact lambda=L/K with L<2^30,K<2^16, and lambda+1=(L+K)/K with L+K<2^31.
Let delta_hat=s_delta*2^e_delta be the observed RN(f+y), positive finite.
The exact lower floor distance for f2/4/12 and f+y's tiny increment at MAX
ensure delta_hat>=2^-52 and delta_hat<=MAX. Delta is normal; it need not be
expanded onto the MIN_SUB grid.

| Compound term | Numerator bits | Denominator bits | Separate exponent |
|---|---|---|---|
| abs(first) | s_W*L: <83 | K*s_delta: <69 | e_W-e_delta |
| second | s_W*L*(L+K): <114 | K^2*s_delta^2: <138 | e_W-2e_delta |
| moment | s_W*L: <83 | K*f: <20 | e_W |

Here '<83' means magnitude<2^83, equivalently bit length<=83; the same
convention applies across the table. Relative53 alignment plus carry is at
most192 bits. Each magnitude and every positive intermediate fits five words.
The exact rational exponent q=-g*lambda/f uses the same small pair form
(s_g*L)/(K*f) and separate represented-g exponent; its IEEE observation also
fits this style. The outward elementary radius/guard requires its SEPARATE
review, not approval by this small-coordinate fact.

For source PV, borrow the exact source B=N_B/D_B with N_B<2^160,D_B<2^148.
An admitted binary64 factor is s_f*2^e_f. Build N=N_B*s_f<2^213,D=D_B and
keep e_f separate. Normal relative IEEE observation requires at most214 bits.
For subnormal IEEE observation e_f>=-1074 gives the direct integer quotient

  round_even(N*2^(e_f+1074)/D).

The branch condition says this quotient's exact input is below2^52, therefore
the shifted numerator is <D*2^52<2^200; it does not require an unconditional
213+2045-bit allocation. Tiny/overflow branches are classified by exact integer
exponent comparisons before shifts. All paths fit5*64=320 bits, with one direct
IEEE observation of source times the accepted factor.

The cache bounds retain all scales0..38: a conservative valuation common form
has numerator coupon_coefficient*L<2^157, denominator10^scale*f*K<2^148.
Adding exact par100 before PV observation increases the numerator by100*D_B
and remains below the declared2^160 bound. The integer controls deliberately
exercise all39 scales, both minimum/maximum coefficient and all3 frequencies;
their maximal coordinate cross-products are labelled primitive supersets.

A concrete source-compatible double-round discriminator is regular f2,
C=(10^38+2)*10^-38, so B=C/2=1/2+10^-38, and an accepted factor primitive
MIN_SUB. Direct source-product IEEE rounding yields MIN_SUB. Rounding the
product first to extended53 yields half MIN_SUB, whose next IEEE round is zero.
This demonstrates why the direct route matters. MIN_SUB factor is not claimed
as date-coupled MM reachable; its valid source amount and primitive factor
combination are a deliberately wider factor/PV control.

## 6. Retention and final direct normalization fit243 words

The proposal's broad MM pre-rounded positive magnitude envelopes are

  first: (2^-3161,2^1198),
  second: (2^-4208,2^1293),
  moment: (2^-1083,2^1038).

They follow from W in[MIN_SUB,MAX], h,tau>2^-9, h,tau<2^14,
d,n>2^-80 and d,n<2^1039. They are independent primitive bounds, not claims
that every endpoint combination is attained by dates.

For compound, admitted exact lambda is >1/184>2^-8 and <20000<2^15;
lambda+1<20001<2^15. Together with delta_hat in[2^-52,MAX] this gives:

  first: (2^-2106,2^1091),
  second: (2^-3130,2^1158),
  moment: (2^-1086,2^1038).

Thus both conventions fit a common loose pre-round term envelope
(2^-4208,2^1293). Extended53 RNE preserves more than half/less than twice a
nonzero term. A safe rounded-term binade interval is[-4209,1293]; the least
possible53-bit unit exponent is >=-4261, far above the fixed2^-8192 basis.
Every rounded term can therefore be inserted EXACTLY by shifting its u64
significand into that basis. With fewer2^15 terms, the retained sum magnitude
is <2^1309 and its basis-scaled unsigned integer X is <2^9501.
243*64=15552 bits leave substantial margin; carry scratch9502 bits still fits.
There is no cancellation below the basis: each inserted term is an integer
multiple of that basis and each derivative family has a common sign.

For raw IEEE getters observe X*2^-8192 directly. Normal observation uses
relative53 extraction. Subnormal observation is round_even(X/2^7118), so a
7119-bit power-of-two denominator is enough. Do not first round the retained
sum to extended53 before subnormal admission.

For normalized getters decode admitted Dhat=s_D*2^e_D, with s_D<2^53 and
-1074<=e_D<=971, and observe DIRECTLY

  (X/s_D)*2^(-8192-e_D).

Normal observation needs at most9502 bits after relative53 comparison/carry.
For a subnormal result the exact integer quotient is

  round_even(X / (s_D*2^(7118+e_D))).

The denominator exponent7118+e_D lies in[6044,8089]; denominator width is at
most53+8089=8142 bits. Numerator width<=9501, and its double-remainder/carry
work also fits243 words. Exact zero, underflow midpoint, MIN_NORMAL carry and
overflow carry can be classified without constructing exponent-sized integers.
There is no need to multiply X by a huge exact reciprocal or round a raw
getter first. This proves the retained/direct-normalization width, not the
existence of every resulting getter range error in a dated model.

## 7. Bounded operations and a constructive by-reference schedule

This is a proposed callable contract and an available resource schedule, not a
measurement of future code. New large helpers should accept borrowed input
magnitudes and caller-supplied mutable output/scratch slots. Wide temporary
copies must be explicit. Borrow disjoint slots rather than returning large
arrays through nested by-value expressions. Integer products must use bounded
schoolbook or an equivalently reviewed bounded algorithm; no heap bignum,
recursive fallback, data-dependent precision growth or arbitrary wider retry.

One constructible design reserves24 slots of112 words for numeric traversal.
A conservative role assignment is:

| Slots | Roles (one unsigned magnitude each) |
|---|---|
|0..2|Y magnitude, shared d_raw, per-flow n_raw|
|3..4|term numerator and term denominator|
|5..6|product outputs before explicit in-frame swap/copy|
|7..9|normalized numerator/remainder, denominator, division/rounding scratch|
|10..11|source numerator and denominator when an owned prepared-cache copy is necessary|
|12..13|source-PV numerator/denominator (five-word-sized values padded here)|
|14..16|Y*D, Y*R and365E during exact affine construction|
|17..19|integer compare/shift/subtract scratch, with explicit caller-selected ownership|
|20..23|spare borrowed-helper work/carry/endpoint images; never implicit value returns|

Many roles are sequential and can be overlaid. This intentionally gives a
simple generous allocation without depending on optimizer last-use analysis.
Input cache pairs may instead be borrowed and their two slots remain unused.
Each numerator/denominator/persistent affine already fits a single slot by the
proofs above. Multiplication writes a distinct output slot, then explicitly
reuses an old slot; it need not preserve an unbounded expression tree.

The64-bit word multiply step can use u128: a*b+old+carry is at most2^128-1,
so one scalar u128 carry suffices. A full112-word schoolbook product has at
most112^2=12,544 word multiply updates; there are two such products to form
d_raw^2*n_raw. Small day/sig/365 factors use bounded scalar passes and E shifts
are fixed2148/3222-bit limb shifts. Normalized quotient extraction scans53
bits, each requiring at most a fixed-width comparison and in-place subtraction.
At112 words it takes at most53 comparisons and53 subtract passes of112 words,
plus bounded preparation/rounding passes. Forty-word and243-word routes have
the same fixed53-bit quotient limit. These are algorithmic upper bounds,
not timing measurements or claimed operation counts for existing helpers.

Keep3 working243-word risk accumulators. Insert each extended53 term in place
with a u64 shifted-add carry; do not build a fourth243-word term array. The
return transfer can temporarily own both3 working and3 result arrays; count
all6 explicitly. Each getter borrows its Risk's3 retained arrays and uses at
most3 mutable243-word scratch arrays: U, V and remainder/copy work. Quotient
and rounded carry are u64, and subtraction mutates the remainder. There is no
extra243-word quotient/subtract buffer. A reference may point to a retained
input but must not silently copy it.

The unchanged own `exact_sum` implementation on64-bit targets owns TWO34-word
arrays. Its per-element helpers borrow arrays and return scalars. Those two
arrays must be counted even when called from a Canadian entry. With the24-slot
frame alive, pad them as two additional112-word logical slots:26<=32. The
remaining six logical slots provide explicit room for small by-value residual
or helper arrays, subject to the final implementation's census. Never allocate
an entire32-slot frame and then call exact_sum while claiming its arrays were
inside the already allocated frame. Alternatively end the traversal frame in
an explicit scope before reduction/getters, or prove a separate actual schedule.
The proposed24-slot route needs no change to old exact_sum behavior.

The claimed ceiling is therefore an ACCEPTANCE LIMIT on simultaneously live
unsigned-magnitude payload, not an instruction to allocate all buffers twice:

  <=32*112*8 + 6*243*8 =40,336 bytes <65,536 bytes.

This includes proposed helper scratch, incoming owned wide temporaries,
explicit small by-value helper copies and temporary result transfer within the
slot budget. It does not follow from bit width alone; the concrete by-reference
schedule is part of the recommendation, and future Rust code must prove it.

## 8. Explicit exclusions and remaining gates

Excluded from this temporary-wide-payload metric: caller-owned input/cache
storage, caller-owned Vec spare capacity, allocator headers/granted excess
capacity, the prepared bundle Vec and PV workspace heap payload, unrelated
caller-retained Risk clones, scalar metadata, borrowed references and compiler
stack-frame overhead/spills/calling-convention copies. These exclusions do not
claim such memory disappears or that actual stack<=40,336 bytes.

The separate input length caps still apply. A moved supplied Vec can have
arbitrarily larger capacity than its len. `try_reserve_exact` describes requested
slots, not a promise about allocator bytes/capacity. A planned<=128-byte bundle
would bound20,000 logical payload slots at2,560,000 bytes;20,000 f64 PV slots are
160,000 logical payload bytes. These layout figures remain design constraints;
no Rust size_of/allocation counter was run here. Actual per-path lifetimes,
layout, allocation attempts, copy behavior and compiler stacks must be checked
after implementation, before claiming the implementation meets the contract.

This review does not independently close source-cash8-word / price-evidence-
cash40-word full recipes, signed quantum/result construction, snapshot geometry,
or all seven monetary modes. It also does not close the compound OUTWARD
log-radius/exp-boundary guard's full bounded algorithm. Those are distinct
review tasks. Such a guard may reuse this numeric frame in an explicit phase
before per-bundle risk construction, but its real widths/lifetimes need their
own proof. No Canadian production should start while required whole-contract
resource gates remain open. This selective closure is not a narrower instrument
scope or permission to omit fixed-coupon cases.

## 9. Actual exact-integer controls

`derive.py` uses Python standard-library integers/Fraction only, captures the
input bytes in exclusive snapshots, and writes `arithmetic.json` exclusively.
The actual run passed:

-14 exact magnitude/payload constants;
-45 relative53 numerator/denominator extrema and even/odd/carry controls;
-36 admitted exact affine-factor controls, including MAX and negative-floor neighbors;
-12 direct IEEE source/factor/retained controls, cross-checked with CPython Fraction-to-float;
-234 source coefficient/scale/frequency controls (39*2*3);
-6 fixed-basis boundary controls, with conservative primitive overshoot cases explicitly
 broader than the actual term envelope.

Observed maximum intermediate bit lengths in the finite relative controls:
MM first4297, MM second6419, MM factor2175, source PV213, compound first122,
compound second191, compound moment/exponent83, retained normalization9501.
These maxima corroborate the proven ceilings; they are not exhaustive maxima
over every dated input. The source-product direct/double-round discriminator
produced respectively0000000000000001 and0000000000000000 as predicted.

Arithmetic SHA256:
`eec43d6cc6e50eddf04357d18c6aa3c9e57ac9ce9a07177982cbff329e22390d`.
No Rust test count, platform runtime claim, financial acceptance, adoption or
published numerical fixture is inferred from this private evidence.

## Executed fixed-quotient addendum

The independent producer also retained `addendum.md` SHA256
`00232d68a3c55169276f15204cd2a059fd8f270e9eed37aa0abdeccdd695be56`,
included in the evidence manifest above:

### Additional executed quotient and source-context evidence

`quotient-controls.py` actually executed the fixed53-iteration quotient scan
on all45 relative numerator/denominator controls. Each result and remainder
matched exact Python divmod, and every RNE carry matched the independent
normalized helper. These are measured Python loop counts, not Rust word-work,
compiler-copy, allocation or stack measurements. The algorithmic <=53 word
comparison/subtraction passes in review.md are constructive implementation
ceilings for the proposed by-reference route.

`source-context.json` records the hash of the unchanged own exact_sum source
read. Its two owned arrays total68 u64 words on64-bit targets. A24-slot numeric
frame plus these two34-word arrays consumes at most26 padded112-word logical
slots. Allocating a full32-slot frame and then overlooking external callee
arrays would be invalid accounting. No existing summation code was changed.

The main review proves widths for the chosen routes and an available logical
by-reference schedule. It does not certify unknown future Rust helper code.
Source-cash/price-cash full recipes and the outward compound guard retain their
separate capacity/liveness gates; no whole-Canadian resource adoption is claimed.
