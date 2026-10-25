# Price enclosure research validation

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

This records mathematical/ordinary-case research and tooling reproduction. It is
not production enclosure implementation, a financial fixture, settlement receipt,
API/contract adoption, bounded Rust conformance or full M8 acceptance. Existing
zero/fixed generic, Gilt, Treasury and Canadian scope/requirements remain unchanged.

## Reproducible ordinary research

Private original `research.py` SHA256
`3a9e43026bcb29964f65ab9df6115bd0b57be0fa11166e0582a1b6714f5c43ce`
produced `ordinary-results.json` SHA256
`ec255adae325a9f333479b70f0ae7ff77b37c5acb351758962b86c4a1e34bd21`.
The proposed new CLI `bond_price_enclosure_research.py` changes path/argument/output
plumbing only; all14 arithmetic/helper function/class ASTs are identical. It
reproduced the entire ordinary-results file byte-for-byte at the same SHA256.
No old generator, fixture or production result was edited.

Reproduce the installed research file:

```sh
python3 -I validation/research/bond_price_enclosure_research.py \
  --repo . --output /private/tmp/sabiquant-enclosure-ordinary-new
```

The output directory must not exist, its parent must exist and it must be outside
the source repository. It is created exclusively; existing output is never rewritten.
Canadian metadata source bytes are pinned before execution. Five actual controls
passed: help, existing-output rejection/unchanged bytes, inside-repo output rejection,
missing-parent rejection and changed-metadata rejection. Rejected outputs were not
created. `run-evidence.json` separately records script and metadata byte hashes BEFORE and
AFTER the run, observed equality, output hash and scope. These are two endpoint
observations, explicitly not continuous monitoring. The guarded CLI SHA256 is
`33273ce7a7733bebd6e910af3fa0aafecd978bb9757d8d16d84145eb22c96577`;
its v2 ordinary output remains byte-identical ec255... and the five guards passed.
The earlier8858... CLI/output/plumbing evidence are preserved separately.

Actual ordinary observations:

| Family | Rows | Scope qualification |
|---|---:|---|
| Generic |33|32 compounding configurations with all ten frequencies/mixed threshold branches and signed coefficients; one exact-factor cancellation control. Arithmetic inputs, not complete dated-plan acceptance.|
| Gilt |4|Regular/short/long/ex-dividend coordinate controls, not all dated lifecycle/7 AI branches.|
| Treasury |3|Regular/short/long raw coefficient controls with selected fixed AI, not all dated/source-policy/tie cases.|
| Canadian |108|3 frequencies ×12 shapes ×3 selections; Preserve only. Existing independent generator supplies build_state/prepare metadata, so no new independent source/geometry validation is claimed. Old ideal/factor outputs are not called.|
| **Total** |**148**|All requested basis intervals <1e-18 wide in these ordinary cases.|

All148 decide one explicit face100/per100, nominal1,000,000, cent, nearest/even
**price-cell arithmetic control**. This is not a complete SettlementInstruction
pipeline or settlement receipt; it covers neither all rounding modes nor all source
stages/aggregation choices. It must not be reported as148 cash receipts.

P96 is a research precision, not an adopted API value. Fraction storage is unbounded.
Actual checks:1,659,975 exact directed-round inequalities,50 exact rational whole-price
containment comparisons,4 exact root-sign controls and198 Decimal corroborations at
220/300 digits. Decimal ln/exp contributes no containment premise. Constructive
series tails and directed integer/rational inequalities provide that premise.

## Independent mathematical challenge

The independent review pinned the exact original report, prototype and output.
It found no containment counterexample. Actual finite challenge controls passed:
272 signed roundings,4,556 rational operations at p2/3/16/64,9 log and9 exp comparisons
to longer exact rational series,16 tail inequalities,tiny +/-MIN_SUB log integral
bounds, and the H_s recurrence through s2049. These are mathematical controls,
not Rust tests or exhaustive all-input empirical proof.

The review confirms the generic exact-prepared versus named exact-source targets,
RN hybrid threshold, three Canadian bases, signed cancellation and finite export
semantics. A generic interval divisor including0 is never evidence of invalid exact
domain. The mandatory proposed affine divisors are exact Raw->positive extended
points, so that ambiguity is private primitive defense/future-extension scope,
not a reason to invent an unused public unresolved variant. Near-unit log intervals
may cross0 as arguments safely. Bounded precision can legitimately remain unresolved.

## Private capacity-controlled primitive witness

The new private Python witness uses33 fixed u64 exponent limbs, capacity-checks
Raw2176/working3328 bits and p<=1024, and avoids shifts by huge exponent values.
Actual controls passed:105 rational rounding pairs,600 rational operations,20 huge
metadata/cancellation controls and34 exact affine seeds, with1,476 normalizations.
Observed maxima: raw2149,working3172,quotient1025 and exponent magnitude2052bits.

An initial statistic counted only integer exponent inputs; the corrected statistic
also counts add/sub-result constructors, changing2051->2052. Arithmetic/control
results are unchanged; exact original witness/result versions remain retained.
Python allocation and builtin division are NOT Rust frame, limb-loop or work-cost
conformance. No whole-model bounded core or final IEEE exporter was implemented.

## Resource/liveness review status and limits

RESOURCE_PROPOSAL now supplies a constructive candidate with both exponent directions,
relative normalization, exact affine admission, borrowed-output/staging schedule,
paired floor/ceil primitives, raw clobber and external exponent rules, and reserved
initialization/terminal work. Its current logical budget is26,264bytes including
generous typed aggregate/return/status positions. All four models and all generic
branches remain in scope. This number is not actual Rust stack or allocator evidence.

Safe Rust shape/MIR research separately demonstrates that large by-value constructors
and returns can create multiple live arenas even without Copy/Clone. Such a witness
explains why actual constructor/return code and typed SettlementEvidence alignment
must be reviewed; a u64 marker array cannot safely stand in for a16-aligned typed
financial result. The witness's earlier24,344-byte marker must not be relabelled as
a measurement of the newer26,264-byte typed proposal or of production enclosure code.

Actual final layouts, helper borrow correctness, limb quotient/carry implementation,
exact matched work constants, every source/cache-role mapping and complete independent
preproduction remain open. The complete ledger must be frozen before any production
or API/contract adoption. Native platform execution, financial/owner acceptance,
maintained calendars/legal source acceptance and release gates remain separate.

## Artifact preservation and proposed promotion

The complete earlier design/API package remains under `frozen-design-api-v1/` with
evidence SHA256 `93fced558e7967768e3859f71682984ee2a7a9a6625c340e75447161a38443fb`.
Every reviewed paper revision and both witness statistics are preserved. The current
combined manifest binds exact current papers/CLI/results, previous revisions and
independent review evidence. It is create-only and separate from earlier evidence.

Proposed NEW files for root review/copy: design, API proposal, resource proposal,
this research-validation document, and `validation/research/bond_price_enclosure_research.py`.
No adopted contract/requirement register, production implementation, old generator/
fixture change or remote action is part of this proposal promotion.

## Repository transcription verification (2026-10-26)

The four proposal bodies were copied from the held reviewed private papers,
with repository titles, status and navigation added. The installed research
CLI is byte-identical to the reviewed guarded version33273ce7... above.
A fresh repository invocation reproduced all148 rows at the exact ec255...
output hash. Five installed CLI controls also passed; rejected output paths
were absent and an existing output's sentinel remained unchanged.

The complete create-only private manifest records the reviewed papers,
preserved revisions, independent reviews, witnesses and source observations:

| Artifact | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-m8-enclosure-design-v1/COMBINED_MANIFEST.v1.json` | `3dc509288d585fcda5f2a05e290d2f45fc35519e436547ae15d2b7b42d3b7c08` |
| `/private/tmp/sabiquant-m8-enclosure-resource-math-final-review-v1/REPORT.md` | `f40f793ac57ec8434c803b1ca911ad22134738b4ce43d63679065b5a767bc006` |
| `/private/tmp/sabiquant-m8-enclosure-resource-math-final-review-v1/evidence.json` | `41eb0b6abce9471de44e1f975e988254e330a48e53a82478a48263a4891e739a` |
| `/private/tmp/sabiquant-m8-enclosure-resource-peer-review-v1/final-evidence.json` | `ff269c745545c8dd9445c05b155eaddea214951f70c77a6c1c9ebb3a20bf314e` |
| `/private/tmp/sabiquant-enclosure-cli-repo-guards-v1/evidence.json` | `5aa212c5a2465e8bfa6b8cc015edee9809392babdbe8062cf589e9cdc357acdd` |

Those private input observations precede the repository readiness/index and
changelog entries for this proposal. The prior readiness text remains an
unchanged prefix; those new entries change no mathematical source, accepted
contract or frozen reference. Rust source/manifests and the144-type/237-error
catalog remain unchanged from `ae7bda7`. This documentation/research step
does not repeat or strengthen that checkpoint's native/package claims.
