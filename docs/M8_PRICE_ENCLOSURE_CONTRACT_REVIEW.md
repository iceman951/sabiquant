# Price enclosure contract adoption review

Local engineering adoption, 2026-10-28, against `79780ab` plus this documentation
change. E1 adopts the contract and installs its tested API foundation; no
financial producer or financial acceptance is claimed, and E2 remains open. The [contract](M8_PRICE_ENCLOSURE_CONTRACT.md) preserves all four
prepared models and every existing price basis and generic configuration.

## Decisions and independent review

Root resolved the earlier design/API/resource proposals into an explicit target,
exact admission order, fixed precision/work policy, controlled borrowed output,
encoding/proof partition and error/progress ledger. Independent reviews by
`primitive_oracles` and `preproduction_plan` identified these concrete issues:

- The private Fold's common-product divisibility and union cap do not enforce
  model-specific roles or source identity. The actual adapter must establish
  the Gilt/Treasury/Canadian 3/3/5 graph, own-role divisibility, selected published
  Treasury AI, Canadian principal once, and exact originating basis/source.
- Public reduction progress counts original price flows. An extra AI term or
  split Canadian coupon/principal must not inflate it. Global setup has no
  current original index; complete arithmetic is recorded before fit/export.
- Exact-point fallback must normalize the preserved full Raw0/E0 before unit
  initialization clobbers the shared frame. No partial stream becomes evidence.
- Complete internal singleton and binary64 export equality are different facts.
  Root's final policy preserves Decimal-first fitting for both the fitting
  integer `2^100+1` and a nonfitting singleton `(2^53+1)*2^-53`. Proof kind and
  encoding stay independent; NoFittingPoint may produce strict outward bounds.
- Fixed source lookup fees do not authorize uncharged arithmetic. The final
  policy explicitly charges bounded small Wide operations and pow10 separately;
  old full multiplication/division/GCD may not bypass the charged Frame graph.
- Frame initialization order is an algorithmic rule. Compiler prologues may
  reserve stack earlier. Typed sizes alone prove no peak-stack or terminal-cost
  bound; actual whole-producer conformance remains FR-ENCL-13/14 and E5.

The final singleton/encoding combination and bounded bridge restrictions are
root technical decisions responding to both reviews. Their actual numerical
implementation is not claimed independently reviewed or executed here. The
reviews found no remaining structural/type/cause/lifetime obstacle in the opaque
API after the accepted refinements. No additional financial feature was added.

## Actual evidence and honest scope

The final opaque API was first compiled in an isolated snapshot of the real crate,
including the actual root `Error::Enclosure`/`EnclosureBudget` wrappers. It has
14 structs/enums and one result alias, private origin construction, two borrowed
lifetimes, explicit traits and bounded diagnostics. No price proof was fabricated
and no prepared plan's new producer method exists in this snapshot.

The first unit compilation exposed three existing exhaustive root catalogue
matches requiring the new wrappers. That failure is retained. The additive fix
keeps every one of the old 237 stable kinds and adds nine new kinds; all 246 are
sampled by the existing full catalogue tests. Four new metadata/error tests pass
in debug/release; all 11 root error tests pass in debug. Sixteen external delta
controls pass for traits, actual root conversions, privacy, removed scaffolding
and borrow-preserving versus invalid borrow-erasing signatures. Earlier facade
34+8 controls retain their historical scope and were not rerun or recounted.
Strict library Clippy, formatting and private rustdoc pass for the compiled
snapshot. Later refinements change doc comments only, with the runtime body
verified unchanged; current documentation checks are recorded separately.

Native macOS arm64 Rust1.99 layouts are Context72, Origin16, Finite176,
Unresolved96, Outcome176, Result192, EnclosureError136, BudgetError16 and root
Error176 bytes. EnclosureError satisfies the local <=160-byte target. These are
opaque API layouts, not public producer stack or allocation evidence. The E1 library change installs fourteen nominal configuration/evidence types
and a result alias, plus nine stable error kinds. The root catalogue preserves
all 237 earlier kinds and samples all 246. A standalone domain cause also exposes
its stable kind, as required by the common error policy.

Separately, the complete exact source Fold has current debug/release full-matrix,
focused and ownership results. Its first-source code/resource peer retains all
caller conditions. These are implementation candidates and source bindings;
they do not prove general nonzero financial enclosures or end-to-end cash.

## Installed library and public tests

The reviewed shapes and root conversions are now in `src/bonds/enclosure.rs`,
re-exported from `bonds`; the metadata/catalogue checks are ordinary unit tests.
`tests/bonds_enclosure_api.rs` exercises actual public budget failures, every u16
precision input, work boundaries/precedence, typed root sources, promised and
forbidden traits, and standalone domain causes. It creates no price proof.
Debug/release focused library, root-catalogue and public API/policy tests pass;
all-target strict Clippy, formatting and private docs pass. E1 validation does
not rerun sealed research or claim financial execution. The actual public price
methods and originating cash consumers remain E3/E4.

## Persistent local records

The owner keeps unpublished research/evidence in git-excluded `.agents/work/`.
The complete records are in these folders, each with its evidence JSON:

- `sabiquant-m8-enclosure-contract-finalization-v1`: contract decisions,
  compiled snapshot, `api-final-checks.json`, external delta and native layouts.
- `sabiquant-enclosure-source-fold-v1/FINAL_INDEX.json`: complete Fold component
  and links to independent current/first comparisons, ownership and resource peers.
- `sabiquant-m8-enclosure-e2-gap-audit-v1/FINAL_SEAL.json`: unchanged 72-row
  inventory and finite remaining preproduction actions.
- `sabiquant-enclosure-financial-boundary-v1/FAMILY_FREEZE.json`: small additive
  public boundary INPUT family, not generated targets or executed tests.

The full source snapshots, initial failures, source continuity and file hashes
are preserved there. Earlier `/private/tmp/sabiquant-*` names are compatibility
symlinks. A later portable preproduction/consumer installation must retain these
provenance distinctions and ship the required frozen reference artifacts.

## Remaining exits

E1 closes on the adopted contract, finite matrix and installed tested API
foundation. Exact row selectors, consumer bindings and the remaining independent
source/target/protocol/comparison/originating-cash expectations are E2 work. The finite audit prescribes
B1 evidence bindings, B2 one small public boundary supplement, B3 logical outcomes,
B4 comparison/usefulness rules and B5 reuse of existing cash consequences. It
requires no second giant price or cash oracle.

Actual four-model public producer code, all financial consumers, real output-to-cash
integration and whole-graph resource/regression/package verification remain E3–E5.
Independent financial/source, primary-platform execution, owner/dependency and
formal review/merge acceptance remain separate under the
[M8 checklist](M8_COMPLETION_CHECKLIST.md). No percentage is assigned.
