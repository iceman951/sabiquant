# Enclosure logical protocol (E2 B3)

`protocol.json` freezes the finite logical cases and symbolic debit-site
relations required by the adopted
[contract](../../docs/M8_PRICE_ENCLOSURE_CONTRACT.md). It binds existing Q/source
IDs and the selectors in `consumer-bindings.json`. Its producer assertion IDs
are planned and have not executed. This is a protocol specification, not an
additional numerical reference family or an actual Rust case fixture.

The local E1 foundation is commit `13901b1`. The protocol records the exact
adopted contract SHA256. Existing source, price, admission, hand, cash and
certificate values are unchanged. There is no new generator, oracle run,
helper prototype, private review, numerical counter golden or semantic mutant.
Actual public producers and whole-call resource checks remain E3–E5 work.

## Symbolic debit cutpoints

Let `B` be the validated public work budget, `T=1024` its reserved terminal
allowance, `M=B-T`, and `U` the actually charged arithmetic work immediately
before the tested block. `U` is an implementation execution fact, not a
precomputed expected financial value. For a bounded block costing `c`:

| Remaining arithmetic work | Block and accounting |
|---|---|
| `c-1` | Debit fails; execute no block; leave `U` and its progress unchanged; terminal report uses `U+T`. |
| `c` | Predebit succeeds to `U+c=M`; execute the bounded block; its real mathematical/error result still applies. If another positive debit is needed, it stops. |
| `c+1` | Predebit succeeds to `U+c=M-1`; execute the bounded block; one arithmetic unit remains, without any completion promise. |

The table applies separately to entry64, Frame8560, lazy arena44032,
admission metadata16, raw sign64, each cached operand4096, original-flow
commit8, basis bookkeeping64, encoding bookkeeping256 and every actual helper
debit. Bridge copy/add/subtract/compare/checked-small-multiply prepays
`16*n+32` with `n<=5`; bounded pow10 prepays
`64+scale*(16*n+32)` with `scale<=38` (maximum4320). Full Wide multiplication,
division and GCD require the charged Frame graph. No flat fee authorizes an
uncharged traversal. If operand staging or a bridge block is needed during
exact admission, its failed debit is an admission stop at that site.

For a public cutpoint, `B=U+c+T+delta` must itself be valid and must actually
reach the same deterministic prefix. Entry and Frame examples below the
public minimum are **constructor-invalid**, not producer golden outcomes.
At the legal entry prefix every validated budget already covers entry and
Frame initialization. A fixture with no reachable public cutpoint is classified
as such; an isolated internal debit-law control does not count as a public
financial execution.

Terminal has a distinct rule: reserve1024 at entry, remove it from the
arithmetic ceiling, and add it exactly once on every producer return.
`U<=B-1024` guarantees `U+1024<=B`, including `B=u64::MAX`. Arithmetic and
Drop cannot consume or alter the reserve. A successful prepaid block charge
is retained if the block later returns its actual allocation/domain/invariant
failure. A failed debit is unchanged and never wraps. Terminal shortage under
these premises is an implementation-accounting defect, not a normal budget
outcome. Actual entry/init/transfer/return sufficiency is still an E5 check.

## Constructor and quote precedence

The constructor accepts only p64/128/256/512/1024 and `max_work>=53680`.
Unsupported precision wins over insufficient work. Every constructor failure
retains attempted p/max, reports work0, and never begins a producer call.
Minimum success promises configuration validity, not route completion, finite
output or an arena allocation.

Nonfinite quotes and applicable scalar global floors are checked before
Frame initialization and before any arena reservation. Their producer error
cost is1088 (entry64 plus terminal1024), with checked/reduced counts0,
`next_original_index=None`, `admission_complete=false` and the original quote
bits. Compiler stack prologues are outside this logical ordering claim.

Generic periodic/hybrid always requires `y>-m`, including selected Simple,
empty, zero amount and time-zero cases. Gilt/Treasury require `y>-2` and
Canadian Compound requires `y>-f`. Canadian active MM has its own exact
global denominator `1+y*(T-S)/365>0`; it does not inherit the compound floor.
The exact MM check follows Frame initialization at QuoteAdmission, with no
original flow index. A work stop during it is `AdmissionIncomplete`, even
when an existing Q row identifies what its completed sign would have been.

## Counts, indices and completion flags

`N` counts original price flows in the actual retained pricing vector,
including zero coefficients. It does not count historical coupons omitted
from that vector, virtual dates, or extra Fold AI/par terms. `i` is the
retained original source index, which need not equal a compressed position.

`admission_checked_flows` counts **completed checks, including a rejecting
check**. If `a` checks completed before the current flow, a work stop before
or during its predicate reports `a`; a completed exact rejection reports
`a+1`, preserves the rejecting `Some(i)`, and leaves admission incomplete.
This remains false even if the completed rejecting check makes checked=N.
For example `ENC-ADMISSION-0003488` rejects original index1 after the index0
check, so checked=2 and reduced=0. Exact sign64 must finish before that check
is recorded as complete. Zero amount or cancellation never waives admission.

Only the all-valid global and original-flow scan sets
`admission_complete=true`. Then checked=N; global Source/Kernel role/common-base
setup has reduced=0 and indexNone. While a current original price flow remains
uncommitted, reduction is `r` and index is `Some(i)`. Commit8 advances reduction
to `r+1` only after every part and accumulator update succeeds. A Canadian
terminal coupon and separate principal both finish before their one public
commit. Internal AI/par Fold steps do not increase that count.

Final source/basis/unit/export phases have reduced=N and indexNone. Finite
success uses Completion. Failed blocks retain their site stage; terminal
bookkeeping does not overwrite an error's meaningful stage with Completion.
The current index must be latched by prepaid metadata or obtained by a bounded
constant-time terminal read, never by scanning after work exhaustion.

`complete_extended_arithmetic` is an Unresolved-only accessor. It becomes true
immediately when the full requested target, all basis shifts and exact units
exist, **before codec/fit/export**. It remains true on a later fit, export or
NoFit arena-fallback work stop. A partial sum, helper endpoint, pre-AI target
or prefix equality does not establish it. Finite proof implies a completed
target, but there is no extra public completion-flag field on Finite or Error.

## Outcome partition

| Trigger | Required result |
|---|---|
| Budget stop before all admission passes | `Err(enclosure.admission_incomplete)`; no evidence; current original index or None for global MM. |
| Completed reachable domain rejection | Exact typed domain error and original rejecting/global context; it wins only if its work actually completed. |
| Debit stop after all-valid admission | `Unresolved(WorkLimitAfterAdmission)`; complete flag follows the full-target rule; no usable partial bound. |
| Sole arena reservation fails after successful predebit | `Err(enclosure.allocation_failed)` with requested_bytes19200 and immediate `TryReserveError`, even after admission. |
| Completed target lacks a finite outward direction | `Unresolved(FiniteEndpointUnavailable)`, complete flag true, Export stage; no clipping or inferred true overflow. |
| Complete finite proof | Finite with same origin, budget, model/basis, correct proof/encoding and declared evidence. |

Every EnclosureError preserves original attempted yield bits, including a
negative-zero input that later suffers allocation or invariant failure.
Finite and Unresolved successful outcomes canonicalize yield negative zero
to positive zero. The error catalogue in JSON names every adopted stable
kind, additional context and immediate typed source. Existing old observed
evaluate/risk/yield/source/cash methods and their failure order are unchanged.

Zero yield still completes admission first. Its mandatory full-source Fold
and preferred point codec can succeed with no arena. A complete Fold finish
sets the completion flag before fitting. On successful NoFittingPoint,
normalize the complete staged Raw0/E0 target into its destination interval
**before initialize_unit or another helper clobbers those fields**. A fallback
budget stop retains completion=true. There is no hidden cheaper route,
precision ladder, second reservation or partial identity certificate.

## Proof and encoding are independent

| Encoding | Permitted proof facts and conditions |
|---|---|
| ExactDecimalPoint | ExactRationalIdentity or IntervalCollapse; complete known point fits canonical Decimal, the preferred representation. |
| ExactBinary64Point | ExactRationalIdentity or IntervalCollapse; Decimal did not fit and an exact finite binary point does. |
| OutwardBinary64Bounds | Strict finite lower<upper. A known point retains ExactRationalIdentity/IntervalCollapse after successful NoFittingPoint; a general non-singleton enclosure uses DirectedArithmetic. |

Controlled complete final endpoint equality always triggers budgeted
Decimal-then-Binary fitting, including equality in extended space before
binary export. The adopted symbolic example `2^100+1` fits Decimal although
its binary outer endpoints differ. The adopted internal singleton
`(2^53+1)*2^-53` fits neither point encoding, so strict outward bounds may
retain IntervalCollapse. These examples describe existing contract laws;
they add no literal financial numerical reference family.

Fit exhaustion returns WorkLimitAfterAdmission with complete=true. It cannot
skip fitting, return partial proof, or downgrade to an equal-bound encoding.
Native observed equality, an intermediate singleton or a partial exact sum
never creates the wrapper's proof fact. Every resulting SettlementEvidence
remains DECLARED, and any determined cash receipt keeps the existing
UniqueForDeclaredEnclosure guarantee.

## Retained source roles and excluded guards

Generic consumes the exact retained IEEE A/t/AI/face bits and original
indices, direct cached horizons and stored hybrid thresholds. Named adapters
must establish their actual fixed semantic denominator roles: Gilt<=3,
Treasury<=3, Canadian<=5, with structural aliases omitted and each selected
term denominator dividing its **assigned** role. The general Fold union's
common-product check alone does not discharge these premises.

Gilt uses selected dividends and raw signed AI; Treasury uses raw analytic
coupons and selected published AI, not diagnostic raw AI. Canadian chooses
valuation/payable coupons, distinct A/AI basis shifts and trueT. In the split
identity stream its terminal coupon is `(N-100D)/D` plus integer principal100
once; this does not require a new GCD. All declared terms, including zero AI,
and original-flow commit counts must agree. JSON links the existing exact
source selectors and representative dated rows for these assertions.

Treasury q positivity follows `0<r<=s` and `y>-2`; Canadian MM numerators
follow `0<=rho_i<=tau` and positive global denominator. These are adapter
premises, not missing public domain goldens. Zero-containing normalized
divisors and violated capacity/source/role protocols are private invariant
guards. If an actual defensive guard reports a failure, it uses
InternalCapacity plus the operation tag, not an invented financial domain or
valid-input unresolved reason. Unsupported configurations and old observed
range gates are not permitted new-operation outcomes.

The fourteen linked Q IDs and seven source IDs are existing immutable rows.
They fix domain/input identity only; they do not predict actual cumulative
work, allocator outcome, proof route or every extreme price result. The
future public consumers must bind these relations to actual E3/E5 site
prefixes and current implementation bytes. No old numerical verification or
native baseline is rerun by freezing this protocol.
