# Independent complete Canadian API proposal review v2

Repository pre-contract review, recorded 2026-10-23 Bangkok. Root promotes the immutable
private review with editorial formatting and this provenance preface; no contract, API,
requirement, oracle or Canadian production is adopted. Read the [complete local
proposal](M8_CANADIAN_LOCAL_DESIGN.md), [numeric/test
design](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md), [source
review](M8_CANADIAN_SOURCE_REVIEW.md) and [research
validation](M8_CANADIAN_RESEARCH_VALIDATION.md) with their separate scopes. The original
producer's no-repository-write statements below describe its private work; root owns
this new repository record.

The three API findings below are closed against actual edited proposal bytes in the
appended post-fix addendum. Original pre-fix review SHA256
`05e9eb450d0c7042e6d1a31e52ee3590531aea81d7620b60aff4198ac850f33d`; post-fix addendum
SHA256 `4c609dd057c5e6cc0c08cef109d720d34a692578678aedf979d82cddc6dc276e`. Private
evidence `/private/tmp/sabiquant-canadian-api-challenge-v2/evidence.json` SHA256
`78793f96125f5f2f24b51bd030f23f7d9ff848144b87bf8e61ce6b0fed6cc4ce`. Controls SHA256
`af94fdd6840a8197a1b5755ca40dd33614738e116a99f136a5a14880b2400763`. This closes the
identified API definitions, not complete contract adoption or independent preproduction.
Later quantitative profile integration is separately reviewed in the [compound
profile](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md) and [MM range
proof](M8_CANADIAN_MM_RANGE_REVIEW.md).

Reviewed 2026-10-23 Bangkok. PRIVATE research/review only, local only. This review does
not adopt a contract, source interpretation, public API, error catalog, oracle, Canadian
production implementation or financial/current-market acceptance. No repository, Git,
AGENTS or handoff file was modified by this reviewer. No QuantLib C++ was accessed. The
review examines the existing full proposal; it does not narrow it to regular/semiannual
coupons or remove independent risk/cash operations.

## Frozen inputs and result

The review pins the two **pre-fix** snapshots in this directory, captured before root's
three requested public-document corrections:

| Snapshot | SHA256 |
|---|---|
| M8_CANADIAN_LOCAL_DESIGN.md | aa8b41bcf98a56e3aab1b524839f298fa900be4a89188b26d2f8515e7aec79ca |
| M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md | 5014a4bbfab23e74b48c5d7c59f56995115698b96a91ac6667abd0ac0892447e |

The earlier stopped v1 snapshots have hashes ac25b4d1... and db1a4053... . Exact
comparison shows only scale39-to-scale39/40 wording changes: one API line and two
numeric/test lines. The v1 controls are retained as earlier research, not counted as new
v2 controls. This review does not claim to have checked root's subsequent edited bytes
until an explicit follow-up binds them.

Result: **three medium documentation/contract-definition corrections**, all already
identified for root, and no additional substantive API defect found in this review. The
complete API shape is coherent conditional on those corrections and on the separately
declared elementary-profile, coupled-MM range, exact resource/liveness, source/cash
closure and full preproduction gates. Those gates are explicitly open in the reviewed
proposal; they are not newly discovered defects or proof of production readiness. No
extra type, domain error or reduced financial scope is required by the three
corrections.

## Findings and precise closure

### API-01 — amount residual overflow is excluded (medium)

Numeric snapshot N4 lines179–180 says the unscaled amount_residual can overflow without
invalidating a solve. This is inherited generic behavior that does not apply to these
successful positive-model-dirty reports. A successful report has finite binary64 D_hat>0
and resolved target T>0. Therefore

`-MAX <= exact(D_hat)-exact(T) <= MAX`.

Rounding that difference cannot overflow. Each represented finite input is an integer
multiple of MIN_SUB; a nonzero exact difference has magnitude at least MIN_SUB and
cannot round to zero. Exact zero occurs iff the two observations are equal. The getter
can retain Result and defensive invariant failure transport without advertising an
ordinary reachable financial range failure.

Requested closure: replace the false overflow statement by this positive-input proof and
label any defensive bounded-helper failure accordingly. Preserve the existing residual
scale rule and selected cached payload API. For s=1, selected only when T>=2^970, a
nonzero nearby difference has very large representable spacing; scaling cannot create a
spurious zero. No financial overflow witness should be fabricated for CAD54/CAD71.

V2 controls independently convert exact Fraction residuals to binary64 for 2,052
selected positive IEEE values, 14,352 actual ordered pairs, including 2,326 scale-one
pairs. All finite bounds, sign, unscaled/scaled zero-iff-equality checks pass. This is
finite support for the general proof, not exhaustive IEEE testing or Canadian runtime.

### API-02 — entitlement argument needs an original-ordinal definition (medium)

API snapshot lines346–347 gives both bundle_coupon_index(index) and
coupon_entitlement(index) without explicitly defining their different argument domains.
Numeric error indices are correctly required to use original actual coupon ordinals
elsewhere, but that does not unambiguously define an accessor's input.

Concrete valid discriminator: D=I2025-03-01; semiannual Q2025-03-01,
2025-09-01,2026-03-01; actual coupons2025-09-01,2026-03-01; S2025-09-01. There is one
future bundle. bundle_coupon_index(0) returns original ordinal1. coupon_entitlement(0)
must mean original ordinal0 and return Ceased, whereas a mistaken compressed-position
interpretation would return BuyerEntitled.

Requested closure: state explicitly that coupon_entitlement takes an original actual
coupon ordinal in0..bond.schedule().coupon_count(), returning None outside that full
domain, Ceased for coupon_date<=S and BuyerEntitled for coupon_date>S. State explicitly
that bundle_coupon_index takes compressed future-bundle position0..bundle_count() and
returns the original ordinal, or None. Keep original ordinals in all financial error
contexts. No method rename, added error or compressed-vector reconstruction is needed.

### API-03 — logical vector caps do not cap caller spare capacity (medium)

Numeric snapshot N5 lines239–241 transfers supplied owned Vec allocations and discusses
length/payload caps. A valid length-two Q vector and length-one actual coupon vector may
each arrive with a much larger already allocated capacity. Moving a Vec transfers that
allocation and capacity. Length caps and reserve_exact requests are therefore not a
bound on total retained allocator bytes, spare capacity or metadata/allocator overhead.
The proposal already distinguishes logical wide payload from compiler stack frames; it
needs the corresponding heap qualification.

Requested closure: bound logical element counts, initialized payload and explicit
reservation requests. Exclude caller-established excess input capacity and allocator
rounding/metadata from that accounting. Keep ownership transfer and no redundant
source-vector copy. Do not add a capacity rejection, allocation, copy or error merely to
make a stronger unrequested claim true.

Actual private Rust1.99 generic Vec ownership control compiled with -D warnings and ran
successfully: two moved Vec<u32> retain len2/len1 and capacity100003/100003. This is a
small bounded witness, not an OOM experiment, Canadian API allocation measurement,
allocator byte census or claim that physical capacity is unlimited.

## Full identity and geometry closure

The proposed compact common metadata is sufficient under the actual schedule rules:

1. Direct anchor, frequency and EOM determine the entire nominal direct cycle.
   Minimal outer coverage with D/M uniquely fixes the supplied Q extent, including a
forward Q endpoint after an off-cycle maturity.
2. If F=M, strict actual-date order permits exactly one coupon[M]; L=D. This includes
   regular, short, long and off-cycle composite single intervals. Both-off-cycle
Composite is classified before the short/long length test, so a nonregular    composite
of exact quasi length1 does not fall through an undefined case.
3. If F<M, F and L are nonterminal Q members. Every interior actual period is regular,
   so actual dates must be the contiguous Q slice F..L followed once by M. F=L is
exactly the two-coupon case. Long first/last can omit Q dates before F or after L;
they cannot create a second hidden choice inside the forced slice.
4. I,C,T,source policy plus these schedule fields recover the full supplied financial
   definition. Caps and caller spare allocation are resource properties, not financial
identity. Exposed first/last kinds are derived consistency metadata. The added L
closes the earlier equal-first-cash/different-long-last collision. There is no need
to clone complete vectors into receipts or weaken Eq/Hash identity.

Independent v2 dated controls reconstruct **19,692 distinct definitions** over all three
frequencies, EOM Ignore/Preserve, six anchors including leap-day/month-end and public
civil endpoints, Q counts2..8, and first/last regular/short/long/composite
crossproducts. All reconstructed Q and actual-date vectors match. There are17 valid
single Composite cases with exact unit quasi length. This finite enumeration supports
the preceding general proof; it does not establish full civil-calendar coverage.

Accrued source_interval is [latest actual<=S or D,S], not the whole next-coupon period.
At S=D or an actual coupon date, S<M ensures the right-hand containing Q pair exists.
The empty interval has the same right pair twice, full_count0 and no fragment; the next
actual coupon remains selected. For a nonempty interval an end on Q uses the left pair,
and a start on Q uses the right pair. A one-quasi interval has one fragment, not two
overlapping fragments. C=0 changes the value, not this date geometry.

V2 checks **108,864 case/settlement accrued geometries** against exact chi differences:
58,878 empty intervals and49,986 nonempty intervals whose C0 value is zero while the
full/fragment geometry stays present. Each geometry has at most two proper fragments.
These are API geometry controls, not Canadian source-amount or native pricing tests.

## Exact census and preserved model breadth

The machine-readable controls bind the actual reviewed proposal, rather than an
anticipated alternative:18 public types, six metadata enums,33 CanadianError plus five
CanadianYieldError variants, and exactly CAD01..CAD72 without gaps/duplicates. Root
Error wrappers are not counted as extra domain variants. The six independent risk
getters and four exact cash recipe identifiers are separately extracted and checked.

The proposal retains all three frequencies2/4/12; regular, short/long first and last,
and composite intervals; compound/MM/last-coupon selection; Clean/ModelDirty/
SettlementDirty; positive model-dirty risk denominator; all six independent raw/
normalized risk views and all four cash recipes. Exact source C/f is not prematurely
gated by DecimalAmount representability. Signed all-in prices and external targets
remain allowed before positive resolved-model-target admission. Source/cash amount
semantics and the three-term binary bridge are intentionally distinct. No equality
between fine-quantum exact invoices and supplied binary bridge evidence is promised.

N6 is correctly future semantic test coverage, not72 implemented tests or a fixture row
count. Declared primitive supersets, ideal/source controls and dated reachable cases
remain separate. Existing open quantitative/numeric/resource gates must be closed before
contract adoption, complete frozen independent corpora and production; this API review
does not substitute for them.

## Actual execution and evidence limitations

control_checks.py completed with exit0. It only reads these frozen snapshots; its 19,692
cases are dated research models, and its Rust program exercises generic Vec ownership.
No Canadian model exists to execute. No old native/coverage/package verification was
rerun or relabelled. Two earlier harness setup failures are preserved as
control_checks.initial.py, control_checks.second.py and control-initial-failure.txt: an
initially unscoped metadata-enum regex and redundant header deletion. Neither was
interpreted as a financial/API failure. The final runner scopes the metadata declaration
block and extracts actual error rows.

evidence.json binds input snapshots, final runner, controls, Rust source/binary and this
report by SHA256. Follow-up closure must separately hash and review root's edited public
documents. This review's current result remains recommendations against the explicit
pre-fix bytes above.

# Post-fix closure addendum

This separately reviewed closure binds root's edited public proposal bytes, not merely
intent to apply corrections. Reviewed 2026-10-23 Bangkok. API SHA256
d1ab79f02aa507725c7a08cb91ab73af8ec7a2014b75dc315177343c824afb0f; numeric/test SHA256
3eec010639d9e0aa166ab73b6082d55c5f9d455baa0b49b3b70fb0e631b799f2. The pre-fix snapshots
and main review remain unchanged in the parent directory.

Exact diffs contain only two API hunks and two numeric/test hunks:

- API-01 closed: N4 now proves that positive finite admitted dirty/target values have
  finite unscaled residual on the MIN_SUB lattice. Defensive Result remains. No fake
financial overflow case is required.
- API-02 closed: entitlement explicitly takes an original ordinal in the full actual
  coupon schedule, including ceased coupons; bundle_coupon_index explicitly takes a
compressed future position. The first-coupon-settlement discriminator is included.
- API-03 closed: financial identity ignores spare capacity; moved Vec capacity,
  caller excess input capacity/storage and allocator overhead are excluded from the
logical temporary payload accounting. Reservation requests do not promise granted
allocator bytes/capacity. No extra copy, rejection, type or error was added.

Exact re-extraction equals the complete pre-fix census: 18 public types, six metadata
enums, 33+5 domain error variants, CAD01..CAD72, all six risk getter names and all four
cash recipe identifiers unchanged. Existing full financial scope is preserved. The dated
reconstruction/geometry and generic Vec controls need no rerun for these text
corrections; their explicit reviewed inputs remain the original snapshots.

Result: all three identified API documentation findings are closed in the pinned
post-fix proposal. This is conditional API review, not contract adoption, full numeric
profile/MM/resource proof closure, source/cash financial acceptance, frozen independent
preproduction evidence or a claim of Canadian production/tests. Those separate gates
remain to be satisfied by their actual authoritative evidence.
