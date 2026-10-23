# Canadian complete source/cash/API integration review

Unadopted complementary review promoted2026-10-23 Bangkok. Original review
SHA256 `a566b354ae6adf8569d96a9467c8aa86e0c9b5371751050dbde4ea95f38faaa5`;
evidence SHA256 `89f6182c5c7636e308bea912c1071393af753eca48294ae81d7a75dc4640589c`.
The original review and actual14-group/2,380-comparison controls are retained
under `/private/tmp/sabiquant-canadian-contract-integration-review-v1/`.

The review below binds its captured initial/closure snapshots. Later public
numeric/resource wording corrections and cash-resource integration require
separate pins; this promotion does not substitute current hashes for its
snapshots. Its source-version/recipe/source-view mappings are accepted design
clarifications to carry into the complete adopted contract. The source policy
and API are still proposals at this checkpoint.

Read with the [complete local design](M8_CANADIAN_LOCAL_DESIGN.md),
[numeric/test companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md),
[earlier API closure](M8_CANADIAN_API_REVIEW.md) and
[research validation](M8_CANADIAN_RESEARCH_VALIDATION.md).

## Original independent review

Private read-only review, 2026-10-23 Bangkok. No repository, Git, AGENTS, handoff,
remote or QuantLib implementation access/mutation was made. This is a review of a
complete local-engineering candidate, not adoption, Canadian production, source/market
certification or a claim that future semantic test families have already run.

## Result and exact scope

The two original complementary source/cash findings are closed in the actual proposed
definitions. No further financial-formula, date-role, source-identity, cash-staging or
public-operation gap was found that requires narrowing the full model or changing its
18-type/33+5-error surface. The earlier three API corrections remain closed.

One concrete integration defect was identified: the initial main proposal §1 put
preproduction acceptance before contract adoption, creating a circular prerequisite
with evidence authored from that adopted immutable contract. Root corrected it during
this review; the separately captured post-edit input contains the noncircular order.

Literal version/recipe output strings and source-view correspondences below are
mechanical normative clarifications for the integrated contract. They do not introduce
a new market convention, owner approval, default, type, error or supported-scope change.
Root accepted these mappings for the contract. The current proposal remains unadopted;
future actual API behavior must be tested after implementation.

Known complete cash-resource integration is being completed separately by root. The
guard/numeric-resource/MM/profile review results were read as separate design evidence;
this source/cash/API review does not claim a second proof of every resource bound or
future Rust lifetime. Their known remaining integration work is not a new finding.

## Reviewed inputs and changing-worktree boundary

input-snapshots.json captures eight initial public Canadian documents. The review then
captures ten files in closure-input-snapshots.json after root promoted bounded-guard and
numeric-resource inputs and corrected the adoption-order wording. The corresponding
bytes live in inputs/ and closure-inputs/; original snapshots are retained. Exact
post-edit main/numeric diffs are retained. The final evidence manifest binds all hashes.

Important post-edit hashes:

| Input | SHA256 |
|---|---|
| Main complete proposal | 260dba7715d334fed22f691847b34651b56327291752d3cbb9d5941aceca78c4 |
| Numeric/test proposal | 95fe941eb612efc348a3b117d6717b8f11547894dfd471e8c9b886a8c1242b20 |
| Source review | dcc9efe40595bf09a4e107c454c9d8cdc875efff3557dcf10f729864726fff2b |
| API closure | f68dca447eb12a0ec2b5dc945c6b27c672a9cf2cc4ce3c204ea2b149a12ca5a5 |
| Compound profile closure | 3c36b9c4e6100739c584a9b46db9f7b7247031748dd22eb834b5acc1a2af5b3c |
| Bounded guard design | eda561678408453539a246f79a9f47c84c7c3253ce1470fd201b011a63f5e2eb |
| MM range closure | 93a08ffd40e67c5daf64c6a061a9edb3320cdb553c6cdf50aeef4700f45a27a2 |
| Numeric resource review | cf774fa87cddcb278a1b2583e23b475e796a56fe8b411396c766fbb9cf19ba95 |

The independent first-pass numerical review and current research-validation snapshot
are also included in the ten-file manifest. Their historical unresolved statements are
qualified by their chronology and newer linked closure records; they are not silently
rewritten as though those earlier producers had reviewed later choices.

This report binds the recorded snapshots. Later cash-resource/identifier promotion
must be separately byte-pinned and incorporated into the final contract; a later
worktree hash is not silently substituted for these reviewed bytes.

## Original source/cash findings genuinely closed

The original complementary report is
/private/tmp/sabiquant-canadian-proposal-sourcecash-review-v1/review.md,
SHA74547939f866fe45353816dd9d335759049368b8a5d6d09251fd71cd473e77a2.
Its findings and closure are:

| Finding | Actual closure and reason |
|---|---|
| CA-SC-01, missing last-coupon start permits equal first-cash/different-instrument collision | Common bond/report/cash metadata retains nonoptional L=last_coupon_start_date, including dirty-only and first-coupon recipes. Eq/Hash includes it. Direct anchor/f/EOM plus minimal D/M coverage fixes Q. F=M forces the sole coupon[M],L=D; otherwise actual dates are the contiguous Q slice F..L followed by M. F=L covers two coupons, and long ends only omit dates outside the slice. The full definition is reconstructible without a vector clone. |
| CA-SC-02, ambiguous AI source interval and zero geometry | AI source_interval is exactly [latest actual<=S or D,S]. At S=D or an actual coupon, [S,S] is present with a repeated right-hand containing Q pair, full_count0 and no fragments; next actual coupon remains selected. S<M guarantees that right pair exists. C0 on a nonempty interval retains the full/fragment geometry and does not impersonate an empty interval. |

The original 516 abstract controls are not the general proof. The later independent API
review's 19,692 dated reconstructions and108,864 geometry checks corroborate that proof
without constituting an exhaustive input test. Its evidence SHA
78793f96125f5f2f24b51bd030f23f7d9ff848144b87bf8e61ce6b0fed6cc4ce
was verified, not rerun or counted as new work in this review.

For nonempty geometry, start-on-Q selects the right pair and end-on-Q the left pair.
A one-quasi interval produces one proper fragment, not overlapping edges. A full-only
C0 interval can have no proper ratios and a positive full count; an empty interval has
full count0. Selected interval endpoints and count keep them distinguishable.

## Source policy and complete financial scope

CanonicalIiac13LocalV1 is explicitly a reconciled local interpretation. Retained
IIAC1.1/1.2/1.3 and the scoped BoC note remain separately hash-pinned;1.3 is the greatest
retained version, not a claim of latest guidance. The source ledger's proposed-choice
language does not contradict the later proposal selecting a local engineering choice.
An integrated contract must carry that choice and its deviation labels forward.

All substantive conflicts have determinate proposed behavior:

- Compound coupons and A use Actual/Actual quasi geometry; MM coupons/coupon cash use
  the payable Canadian365 construction; cash uses exact settlement AI. Model-dirty,
  clean and settlement-dirty stay distinct.
- Monthly settlement AI=A follows the repeated general exception, with the p42
  departure explicit; monthly odd payable coupons still use the stated payable rule.
- Complete virtual cells contribute C/f. Proper fragments preserve the historical
  late branch and its exact184-day downward step; no cap/smoothing/fixed365 substitute.
- Long-first DIC1 is resolved by interval/quasi intersections. Long last and fully
  composite endpoints are explicit local extensions. A one-cell composite uses its
  one actual fragment once.
- MM includes each actual coupon once and terminal par once. True terminal payment T
  is caller supplied; no maintained calendar, lag or additional delay interest is
  invented. Prepared S stays I<=S<M, including when T>M.
- Printed p16/p17/Appendix errors remain source divergences. Appendix displays do not
  become hidden coupon/daily/AI/price quantizers. An MM yield label does not switch the
  instrument to money-market-security rounding.
- All seven signed modes, arbitrary supported quantum and both aggregations are
  explicit caller inputs. Source minimum scales39/40/nontermination are preserved
  internally, with no public DecimalAmount view gate before otherwise valid cash.

Full scope remains f2/4/12; regular, short/long first and last, composite/single coupon
and combinations; compound/MM/explicit last-coupon strategy; three price bases; bounded
inversion; all six independent risk views and all four exact cash recipes. No source
ambiguity is hidden by a regular-only subset. Separate amortizers/floaters/indexed/
optional bonds remain the explicitly declared model boundary.

## Cash staging and compatibility

The old amount, settlement and frozen Treasury cash contracts were read for engineering
compatibility and their hashes are in context-inputs.json. Their source formulas and
Treasury publication quantizers were not imported into Canada.

The four stated cash formulas use exact source U/AI or exact caller price evidence,
then the chosen final quantum stages. Pure source recipes have no price guarantee;
price recipes label only their ingredient. A binary bridge uses represented AI_hat,
whereas an exact clean invoice uses exact AI; fine-quantum equality between these
recipes is deliberately not promised.

Capital Exact completes before accrued Exact, then wide exact net; total aggregation
has a single total quantizer. Only final values are admitted, lower before upper for a
nonpoint final stage image. Intermediate wide components can exceed i128 and later
cancel. They cannot waive an earlier Exact failure. A continuous nonpoint Exact stage
is unresolved even when its endpoints are multiples. Equal declared endpoints are a
point while keeping declared provenance. Non-Exact capital quantization can collapse a
later net image to a point, so endpoint labels follow the current stage image.

Actual new Fraction controls:14 named groups and2,380 signed quantizer comparisons,
all passed exit0. These corroborate the late step,181/182 completed-piece rule,
monthly AI/payable distinction, odd amount roles, all three Cmin/f source-view failures
with fused cash rescue, capital-first Exact, continuous Exact, interval-to-point
collapse, and two valid large-source/range-rescue constructions. One long-first CMAX
case has rounded capital=-2^128 and rounded AI=3*2^126-2, both outside public range;
their net=-2^126-2 is admitted. Its Exact path instead fails at accrued before any net
rescue. These are mathematical/proposal controls, not existing Canadian receipt or
error-transport executions.

## Mechanical normative mappings for the integrated contract

Use this exact local version string for CanonicalIiac13LocalV1:

`CA-IIAC13-fixed-2026-10-23-local-v1`.

The date identifies the local engineering-contract version, not IIAC publication,
regulatory effectiveness, latest-market applicability or financial acceptance.
All Bond/Plan/Valuation/Risk/YieldReport/CashInstruction version inspectors must return
that same literal for the selected policy. Another interpretation needs another policy
variant/version even when outputs coincide. This is a recommended adopted-candidate
identifier; no current implementation of these inspectors exists.

| Factory | Exact recipe_identifier() output |
|---|---|
| CanadianBond::coupon_cash | PayableCouponPer100 |
| CanadianPricingPlan::accrued_cash | SettlementAccruedPer100 |
| CanadianPricingPlan::clean_accrued_cash | CleanPriceAndSettlementAccruedPer100 |
| CanadianPricingPlan::dirty_cash | SettlementDirtyPricePer100 |

No implicit prefix/suffix, registry/parser, currency, rounding mode or default is added.
The literal strings preserve the proposal's existing formula labels.

Bind selected source views explicitly, including their existing diagnostic context:

| Instruction recipe | source_value_per_100(q,m) | View stage; capacity index |
|---|---|---|
| Coupon | Same value/quantization error as payable_coupon_per_100(i,q,m), wrapped Some | source payable coupon; Some(original actual ordinal i) |
| Accrued or clean/accrued invoice | Same value/quantization error as settlement_accrued_at_quantum(q,m), wrapped Some | source settlement accrued; None |
| Dirty-only | Ok(None), without a quantum operation | No source view phase/index |

Every real view uses endpoint point and AmountError quantize semantics; optional view
failure does not invalidate instruction creation/evaluation. q/m is the independent
call's policy, not retained invoice rounding. Common source policy/version and complete
instrument identity remain present even in the dirty-only case.

## Adoption-order correction and remaining boundary

CA-INT-01 (medium) is CLOSED in the post-edit main hash above. The required order is:

1. Finish complementary integration of the complete source/model/API/cash/numeric/
   resource contract and adopt that complete immutable local engineering contract.
2. Author, independently review, validate and freeze all required source/date/cash,
   prescribed primitive, ideal price/six-risk, inverse and blind-hand evidence from
   that contract, preserving every old frozen artifact.
3. Begin Canadian Rust only after the complete preproduction gate passes. Then bind
   actual production consumers/kernel checks/mutants and native/package/coverage proof.

No extra owner-approval gate is inferred. Financial/current-market/calendar/release and
native foreign-runtime acceptance remain distinct wider milestones, not prerequisites
invented to stop already authorized local engineering work.

The exact census is still18 types, six metadata enums,33+5 domain errors and72 future
CAD families. Their full names/order are recorded in census.json. This review closes
the stated source/cash/API integration questions; it does not assert that all full
preproduction fixtures, actual Rust behavior or whole M8 acceptance exists.
