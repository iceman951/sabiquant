# Independent conventional gilt ideal comparisons

This additive corpus implements FR-GILT-03..13 and FR-GILT-22 from the frozen
[local engineering contract](M8_GILT_CONTRACT.md). It is authored before gilt
production, alongside separate exact source/cash and numerical-staging oracles.
It supplies conditional financial comparisons, not financial acceptance,
elementary-function certification, settlement enclosures or full M8 completion.

The [generator](../validation/generator/gilt_ideal_oracles.py) uses only Python's
standard library. During this oracle task the author read no Rust source or
observed production price, derivative, root or callback trace. The preceding
API review had inspected generic public types and residual error labels; this
report does not claim that the author's entire prior conversation was
Rust-unread. The source formulas, numerical comparison budgets and inverse
equations here derive independently from the adopted gilt contract and the
[official source examples](M8_GILT_SOURCE_EXAMPLES.md).

## Artifacts and reproduction

```sh
python3 -I validation/generator/gilt_ideal_oracles.py --check
```

The normal generation command refuses to overwrite existing files. `--check`
recomputes both precisions and all assertions, then compares complete frozen
bytes. Headers pin the generator, adopted contract and only the24 FR-GILT
requirement rows, so unrelated later requirements do not change this corpus.
Original actual generation UTC and Python/platform metadata are retained during --check. Schema1 SHA-256/FNV hashes
cover generator, body, records and contract; census fields are recomputed.

| Artifact | Inventory | SHA-256 |
|---|---|---|
| Adopted contract | Complete local engineering model | `03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292` |
| Generator | Independent exact source, ideal arithmetic and inverse author | eef3f534d615d9bb0ffe40788d7826abb73c0a63f96ab7490a1f03f0253726fe |
| [Price/risk fixture](../validation/fixtures/bonds/gilt-ideal-oracle.fixture) | 374 rows | ac6992aae85a2f743a862375e96ee932d268e2b59ce31780a8afebf916551bdf |
| [Inverse fixture](../validation/fixtures/bonds/gilt-inverse-ideal-oracle.fixture) | 51 rows | 7ae871f8a64df6898efb87bdeffe94d52ccbd5415b2b6e8742c265f4bf3ebd84 |

The eight printed prices and eight printed inverse yields come from the
independently inspected DMO example PDF, SHA-256
`8f491b8567c6f3a75ee2a7cbc02ccc4275810257df5533ed1fd349992465e018`.
The pinned [source checker](../validation/generator/gilt_source_example_check.py)
has SHA-256 `0bdf495eab0bc73fb5d250f5e9266c5186a3b905d127d7cdaf8eb6c4c3b33b02`.
This generator embeds the stated source cases independently and checks their
actual six-/nine-place display precisions. It does not read a PDF or invoke
that checker as its mathematical oracle.

## Exact source construction and schema

Each row contains the complete explicitly supplied instrument and settlement:
issue, maturity, exact decimal annual coupon, first actual dividend, EOM,
complete quasi and ex-date vectors, and settlement. Expected first role,
strictly next quasi index, r/s/n, exact AI numerator/denominator, retained
original bundle indices and exact grouped source coefficients are serialized.
Publication is integer positive half-up to a microunit; regular C/2 remains
an unrounded rational. Missing/detached payments are absent; actual entitled
zero coupons remain bundles and their original indices survive.

For the eight historical price/inverse embeddings, original issue is set to
the quoted previous quasi date. That is an explicit valid engineering
definition reproducing the quoted future source cash flows and current AI;
it does not assert the historic gilt's actual issue date. Supplied future
cutoffs are explicit compatible inputs, without a maintained UK-calendar
claim. The price rows use the represented binary64 yield, while the printed
source used a decimal quote: agreement at its printed precision is separate
from the richer represented-input ideal comparison.

Price rows carry `yield_bits`, optional printed source display, then four
fields for each of dirty, clean, first/second price derivative, time-weighted
amount, modified/Macaulay duration and convexity:

| Suffix | Meaning |
|---|---|
| `_ideal` |80-significant-digit ideal mathematical reference |
| `_class` | `finite`, signed `underflow`/`overflow`, or `uninformative` |
| `_lo_bits`, `_hi_bits` | Outward IEEE endpoints of the conditional comparison budget |

Finite comparisons use direct closed interval membership, avoiding an
additional rounded residual in the consumer. A signed underflow/overflow
classification refers to the getter's pre-IEEE retained-value interval, and
requires the corresponding typed range outcome. Infinity endpoints are
outward bounds, not admitted successful financial values. An uninformative
row supplies no permissive financial comparison: it must be counted and
reported. Exact staging, callback/error sequencing and factor acceptance
remain separate oracle/hand-test obligations.

## Ideal equations and conditional error propagation

Exact `lambda=(r+j*s)/s` and exact source coefficient A define
`W=A*(1+y/2)^(-lambda)`. Direct high-precision powers produce the price and
symbolic risk finite sums. A separate fractional power/integer-power recurrence
cross-checks every price. The derivative and moment definitions are the
contract's equations, with no convexity half. Four representative source
profiles also check central first/second differences at h, h/2 and h/4;
error decreases by more than3.9 on each halving, independently of binary
production price resolution.

Let u=2^-53 and m=2^-1074. For non-tiny yield the exponent allowance is
`E_z=(5u+4u^2)*abs(z_ideal)`. The tiny path instead propagates its analytic
log remainder through lambda/2 and adds one IEEE exponent-rounding radius.
For every accepted finite positive factor, the conditional contract profile
gives `E_d=d*expm1(E_z)+4u*d*exp(E_z)+m/2`. A source PV's allowance is
`E_V=A*E_d+u*A*(d+E_d)+m/2`; exact-zero source PV has zero error after its
factor obligation. Dirty adds one exact-reduction final rounding allowance.
Clean separately includes exact cached-AI rounding and final subtraction.

Represented delta is computed by exact rational RN of 2+y. Each numerical
risk multiplier g_hat is an exact rational from that represented delta and
integer L/s, while the ideal g uses exact 2+y. A retained term's allowance is
`E_V*abs(g_hat)+W*abs(g_hat-g)+u*(W+E_V)*abs(g_hat)`.
There is no IEEE range admission before the retained risk sum. Raw getters
add their final rounding radius. For an ideal normalized value N and
denominator D, direct normalization has pre-round allowance
`(E_raw+abs(N)*E_D)/(D-E_D)` only when D>E_D. Its final IEEE radius is added
separately. This preserves normalized comparisons even when a raw getter
overflows or underflows. It is not a blanket net-price ULP tolerance.

Bounds that cannot exclude a factor/PV/dirty range boundary or denominator0
are explicitly uninformative. Because the primitive hypotheses concern
admitted finite observations, these analytic comparisons do not promise that
an arbitrary Rust libm will admit every factor. Native factor observations,
typed errors and measured profiles require actual implementation validation.

## Reference precision, including tiny quotes

Both160 and240 Decimal digits recompute every serialized field independently;
all fields must agree, including directed binary bounds and status decisions.
Exact integers/Fraction control coupon publication, inputs, source AI,
represented delta, target clean shift and the independent IEEE converter.
That converter is additionally checked against Python Fraction conversion on
2,057 positive exact inputs and their negatives, including IEEE range ties.

The working corpus has at most100 source periods, finite input magnitudes
within the contract's binary/decimal domains, and no Decimal exponent-range
boundary. Correctly rounded Decimal divisions and documented Decimal ln/exp
operations contribute at most a few precision units each. Exact source inputs
are converted directly; no binary64 approximation enters the reference.
The price exponent sensitivity is bounded using lambda<=100 and abs(ln q)<710;
the finite sum has at most100 nonnegative terms. Even allowing1000 precision
units through each elementary/derivative/reduction path leaves substantially
more than20 guard digits before the deliberately conservative relative
reference padding `10^(-precision+30)` (1e-130 or1e-210). Gross positive
contributions, rather than a cancelled clean net value, determine that padding.
The independently repeated80-digit serialization leaves50 additional digits
at the lower precision. Agreement corroborates this finite budget; it is not
a formal directed-interval implementation or universal Decimal theorem.

For abs(y/2)<1e-8 the reference **never forms rounded 1+y/2 then takes ln**.
It evaluates the alternating log1p series starting from the exact represented
input ratio. The stopping test bounds the omitted geometric tail relative
to the retained nonzero total by approximately10^(-precision+8). For
±MIN_SUB the Decimal exponent range retains y/2 and y^2, despite their being
below binary64. The nonlinear correction is smaller than the exact-input
conversion/reference allowance. Thus a spurious ln(1)=0 is not mistaken for
a meaningful tiny reference.

Risk multipliers first form the full exact Fraction denominator 2+y (or its
square), then convert the complete multiplier once. The invisible relative
correction from 2±MIN_SUB is below MIN_SUB/2 and well inside the stated
reference allowance; both precisions may serialize identical80-digit risk
values without certifying that the mathematical corrections vanish. Cases
requiring exact subnormal-stage bit discrimination belong to the separate
integer staging oracle, not this ideal-comparison corpus.

## Target-driven inverse references

Every inverse starts with a supplied target quote, not a price generated at a
seed yield. Dirty T is its represented quote. Clean T is one exact rational
binary rounding of quote plus the separately RN-even cached source AI. The
fixture retains original kind/bits, resolved T bits, bracket, options and
fixed residual scale. Positive ideal sums are solved by300 bisections with
both precisions, using the fractional-power/integer-power price expression.
The remaining bracket width and the reference price allowance divided by a
positive high-end derivative lower bound widen the ideal root interval.

For one positive payment, independently evaluate
`y=2*(exp(ln(A/T)/lambda)-1)` and require agreement with bisection. Retained
zero coupons do not prevent this mathematical one-positive-payment check.
Exact A=T has exact root0; the selected remaining one-payment inputs are far
from catastrophic subtraction in exp(a)-1.

A uniform conditional price-error allowance covers the complete supplied
bracket: each factor uses maximum discount at the lower endpoint and maximum
absolute log at either endpoint, with the tiny-path remainder separately
bounded. Invert T+E and T-E to enclose every computed-zero location consistent
with that profile. Expand by the existing M6 full-width tolerance, bounded
above by `(abs_tol+rel_tol*max(abs(initial endpoints)))*(1+4u)`, for step
termination. This includes price conditioning and solver termination as
separate contributions. Global slope_lower is a conservative positive
high-end derivative bound for this fixed bracket, not a universal source
slope floor. `uniform_price_error`, `slope_lower`, `solver_width_bound` and
the ideal and conditioned root intervals remain separately visible.

An ideal model's continuous uniqueness does not imply binary-quote
attainability. For example, a one-day/182 last-period par100 claim cannot
reach target135 at any admissible binary64 yield near -2; that unbracketed
target is deliberately excluded from successful inverse rows. Such no-bracket
and precision-limited behavior is a separate solver/hand-test obligation.

## Coverage and limits

The corpus includes all three first roles, current cum/ex states, cutoff
equality and neighbors, skipped-Q1 and actual-F dates, EOM alternatives,
exact189/181 coordinates, regular scale39, a published-zero short coupon,
zero C, both yield zero signs and ±MIN_SUB, negative/large positive yields,
and legal quotes immediately above -2. Public source definitions exercise
raw derivative overflow/underflow, raw moment overflow, and finite raw
second derivative with normalized-only convexity underflow. No private
financial report or arbitrary accepted factor is forged for those cases.
The generator remains production-unread during authorship; runtime reachability
and typed outcomes must still be demonstrated by the normal public consumer.

The frozen inventory is 374 price/risk rows and 51 target-driven inverse rows. Semantic family counts remain in each fixture header and are recomputed by --check. The normal shared Rust schema1 reader accepted both files; actual gilt consumers and runtime reachability are still pending.

These fixtures do not replace exact staging, all cash-mode/source-rational
cases, callback traces, mixed error order, constructor rejection, resource
budgets, mutation tests or the full native/package/regression checks.
Cross-target compilation does not validate foreign libm runtime precision.
The broader calendar, market-policy, enclosure, financial/owner/release and
full-M8 gates remain open.

Before first acceptance, root replaced the draft synthetic midnight metadata
with actual generation/runtime metadata after the author reached its usage
limit. Prior unaccepted files remain archived intact in
`/private/tmp/sabiquant-gilt-preproduction-verify-v1`. Complete mathematical
body/record bytes are identical across that metadata correction; no expected
price, risk, target or inverse bound was changed.
