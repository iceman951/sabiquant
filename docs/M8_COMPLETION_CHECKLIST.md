# M8 completion checklist

Reporting checklist, audited at `4d7d7fa` on2026-10-27. It preserves the
existing scope and contracts; it adds no financial feature or acceptance.
The audit snapshot and later evidence remain distinct.

**M8 is not complete.** The earlier informal progress estimate is withdrawn.
Use the finite completion conditions below; research case counts and coverage
are evidence, not a measure of milestone completion.

## Scope that remains fixed

The [roadmap](ROADMAP.md#4-future-milestones-provisional) requires zero-coupon and
fixed-rate bonds, settlement, clean/dirty price, accrued interest, ex-coupon,
yield, duration and convexity, with literature/reference evidence.
[M8 readiness](M8_LOCAL_READINESS.md#complete-objective-and-current-evidence)
also retains justified analytical enclosures and the exact cash boundary.

The enclosure scope remains all four existing models: generic prepared bonds
with all32 compounding configurations; conventional UK gilt; fixed-principal
US Treasury; and Canadian f2/4/12, compound/MM/last-coupon selection, all admitted
geometry and three price bases. The existing exact input/source/date/entitlement
meanings remain fixed. No existing risk, yield or cash family is dropped.

Callable/convertible and other later bond products are outside the adopted M8
scope. This checklist does not add new countries, calendar providers or products.
Any newly found defect belongs to the applicable existing exit condition;
writing another test does not create a new completion goal.

## Local families already implemented and executed

| Adopted family | Current local delivery |
|---|---|
| [Exact amounts](M8_AMOUNT_CONTRACT.md) | Canonical decimals, exact/fused arithmetic, explicit quantum and seven rounding modes |
| [Binary boundary](M8_BINARY_BOUNDARY_CONTRACT.md) | Explicit exact and rounded binary64 conversions |
| [Generic bond core](M8_BOND_CORE_CONTRACT.md) | Zero/fixed terms, explicit settlement/entitlement, earned-minus-detached accrual, quotes and prepared direct-horizon pricing |
| [Exact settlement boundary](M8_SETTLEMENT_CONTRACT.md) | Four explicit monetary recipes over supplied or declared evidence; conditional receipts |
| [Generic yield/risk](M8_BOND_YIELD_RISK_CONTRACT.md) | Bounded yield inversion and all six risk views |
| [Conventional gilt](M8_GILT_CONTRACT.md) | Complete adopted source price/yield/risk and four cash operations |
| [US Treasury](M8_US_TREASURY_CONTRACT.md) and [cash](M8_US_TREASURY_CASH_CONTRACT.md) | Complete adopted fixed-principal price/yield/risk/cash scope |
| [Canadian](M8_CANADIAN_CONTRACT.md) | Complete adopted f2/4/12, compound/MM, three-basis price/yield/risk/cash scope |

These are local engineering results under their existing contracts, not formal
M8/owner/financial acceptance. The audit checked actual test logs and source/test
hashes against the [latest root-library validation](M8_CANADIAN_RESOURCE_FOLLOWUP.md),
including the consumer binaries for every family. All307 consumed inputs still
match; library source, tests and runtime configuration are unchanged from that
verified snapshot. Historical wording in frozen REQUIREMENTS/contract checkpoints
is not treated as a current implementation status statement.

The settlement code explicitly proves recipe consequences conditional on supplied
evidence. It does not currently produce a justified full financial price enclosure.
Separate scalar/interval/log-exp research is useful groundwork; successful helpers
do not close any of the financial producer conditions below on their own.

## Five remaining local engineering work packages

These are fixed reporting groups for the existing obligation, not new requirement
IDs, equal weights or a replacement for the adopted requirements.

- [ ] **E1 — Freeze the complete enclosure contract and API.** Resolve and adopt
  all four model targets, exact input/source/basis/unit meanings, controlled proof
  and outcome/error/lifetime/encoding API, admission/budget/unresolved behavior,
  and one finite requirement-to-test matrix. The current design/API/resource
  documents are proposals. Exit proof: a reviewed adopted contract and complete
  bindings, preserving earlier contracts and behavior.
- [ ] **E2 — Freeze independent financial preproduction.** Derive and review full
  dated four-model/source-role expectations, errors/budgets, signs/cancellation,
  exact points/bounds/unresolved outputs and cash consequences before the new
  financial producer. Reproduce, attest and freeze them additively. Exit proof:
  the complete independent financial matrix and blind controls; arithmetic helper
  references alone are insufficient.
- [ ] **E3 — Implement all four financial enclosure producers.** Integrate exact
  admission, rigorous factors, complete signed reduction, source AI/clean/unit
  shifts, finite outward export or justified exact points, controlled origin
  provenance and honest unresolved/resource outcomes into the actual prepared
  plans. Exit proof: public producer code and complete independent financial
  consumer results for the fixed model scope.
- [ ] **E4 — Connect proved price evidence to exact cash.** Compose finite producer
  evidence with the existing applicable generic/named recipes and source policies.
  Preserve units/bases, all seven modes, arbitrary quanta, total/component staging,
  universal Exact semantics and determined/ambiguous/unresolved results. No
  ModelDirty-to-cash ambiguity, midpoint/clipping or receipt-guarantee upgrade.
  Exit proof: reviewed API and end-to-end originating-model-to-cash cases.
- [ ] **E5 — Close whole-producer resources and integrated verification.** Match
  every actual source/limb/series/flow/interval/point/export/init/terminal path to
  the adopted finite work/resource policy; inspect actual arena/typed-return/error
  liveness and allocation choices. Pass the final financial matrix, regressions,
  meaningful fault tests, strict docs/catalogs/checks, scoped coverage and extracted
  package; preserve frozen artifacts and update traceability/status/discrepancy/
  changelog records. Exit proof: the complete integrated source/evidence/review
  packet, not helper layouts or unadopted payload/budget numbers.

E2 follows E1; the financial producer follows the frozen references; E4 follows
the producer. Resource analysis can progress in parallel, but E5 closes only on
the actual integrated implementation. All five exits are still open at this audit.

## Independent acceptance and formal completion

Local engineering completion and independent acceptance are separate states.
After the five engineering exits, the following still need affirmative evidence:

- [ ] **A1 — Financial/source acceptance:** independent review of the claimed
  model versions, literature/reference discrepancies and explicit calendar/legal/
  market boundaries, with prerequisite financial decisions properly resolved.
  This is review of the defined scope, not an implicit requirement to implement
  every market/calendar feature.
- [ ] **A2 — Native primary-platform execution:** actual required test execution
  on all primary platforms under OQ-08. Existing native macOS results and
  Linux/Windows compilation do not establish Linux/Windows runtime acceptance.
- [ ] **A3 — Owner and dependency acceptance:** explicit owner review of the final
  API/M8 delivery and formal M6/M7 dependency status. Continuing local authority
  and commit signoffs do not constitute this review.

The [roadmap Definition of Done](ROADMAP.md#2-definition-of-done-for-implementation-milestones)
also requires reviewed PR merges. Track **P1 review/merge** separately, but do not
waive it when claiming formal milestone completion. This local audit did not
query or change remote PR/merge state; a current authoritative record is required.
Local-only authorization remains in force.

Track **P2 release/publication** as its own decision. Publication is not itself
an M8 feature-completion requirement in §2; M5 separately calls for a publish-or-hold
release decision. A release cannot substitute for acceptance, and an unpublished
local engineering candidate may still have its engineering exits satisfied.

The finite completion rule is: preserve all adopted local families, close E1–E5,
then satisfy applicable independent/owner/native gates and the existing formal
Definition of Done. No test count, helper milestone, commit, merge or release
alone establishes all of those states.

## Audit record

The private audit packet contains a machine-readable `completion-ledger.json`
with every original FR-AMT/BIN/BOND/SETTLE/YR/GILT/UST/CAD row's file/line/hash,
family code/consumer/log identities, original scope anchors, the five finite
engineering exits and separate acceptance/process states. It does not create new
financial requirements or retroactively mark every individual row accepted.

The checked private full log/exp release result is a mathematical helper result
only. It closes no public producer/API, financial preproduction, cash integration,
whole-graph resource or acceptance item. Actual implementation/evidence may advance
after this snapshot; update an item's state only when its stated exit proof exists.

The local audit packet is
`/private/tmp/sabiquant-m8-completion-audit-v1/final-index.json`, SHA256
`f3b05ce2d660635c6c27ac5918f6a59464e33c71c2379fe6ac09923a3e36bb3e`.
Its machine ledger SHA256 is
`c314bf2d47286f254646321d2ba3652f771a37599d8226cc1e52b665d27aafb3`.
This transcription changes only the candidate's publication header and adds
these audit pointers; the scope and exit conditions are unchanged.
