# Canadian public API and metadata hand expectations

Recorded2026-10-24 Bangkok before Canadian production from the
[complete contract](M8_CANADIAN_CONTRACT.md), C8 and incorporated public §6.
The [reviewed body](../validation/hand/bonds_canadian_api_hand.rs) has seven
future Rust test functions. It complements the separate blind
[source/cash](M8_CANADIAN_SOURCE_CASH_HAND_DESIGN.md) and
[numerical](M8_CANADIAN_NUMERIC_HAND_DESIGN.md) financial expectations.
No Canadian API has compiled or run.

The author read the adopted specifications and existing module export paths,
without reading Canadian production or deriving new financial references.
The minimal instrument has C0, a real regular schedule and delayed true maturity.
Metadata reports use actual cached zero-yield observations; these are transport
expectations, not independent price/risk accuracy certificates. Root reviewed
the entire565-line body, trait/probe matrices, dated inputs and reachable errors
against the adopted signatures, then promoted all three files unchanged.

## Required future assertions

| Binding | Frozen expectation |
|---|---|
| Positive traits | All18 types: Debug/Clone/PartialEq/Send/Sync;12 Copy and12 Eq+Hash; both errors implement StdError. |
| Metadata | Exact five ordered ALL arrays3/4/1/2/3; f2/4/12, months6/3/1, three20,000 caps; no invented selection ALL/parser/default. |
| Typed prices | All three bases, signed finite extremes/MIN_SUB and canonical zero; equal values hash equally, without requiring unequal hashes to differ. |
| Reports and lifetimes | All three selections/resolved conventions; source version, policy and21 report metadata fields; Plan::bond retains the original bond lifetime and pointer. |
| Owned identity | Schedule ignores accepted cap and caller spare capacity; Risk/cash snapshots survive original Bond/Plan owners; receipt retains instruction identity. |
| Reachable error chains | Price NaN, cap sourceNone, quasi-cycle Date underflow index0, cash Exact inexactness, invoked lower compound-domain Callback, nonpositive target sourceNone and MM same-sign Solver failure. Root kind/Display and stored immediate typed causes remain intact. |

The [positive matrix](../validation/hand/canadian_api_trait_matrix.json)
contains prospective144 public types/237 domain variants as future census
expectations. Actual implemented catalogs remain126/199 at this checkpoint.
This body does not claim all33+5 Canadian errors have runtime coverage or
fabricate invariant-excluded capacity/allocation failures.

The [negative obligations](../validation/hand/canadian_api_negative_obligations.json)
contain92 separate future compile-fail sources:18 each for Default/PartialOrd/
Ord/FromStr, six each for Copy/Eq/Hash, and two CanadianPrice implicit From
conversions. First establish a successful positive Canadian API baseline, then
compile each probe separately and require its intended E0277 trait-bound
diagnostic. Missing types/imports do not establish forbidden-trait absence.
Privacy, external exhaustive matching and complete API inspection remain
additional implementation obligations.

## Frozen identity and actual verification

| Artifact/evidence | SHA256 |
|---|---|
| Reviewed public body | 2dd44229861007b7659cdc1d7f95c1a4d9474826d1356a0964f2beb79ea91f4b |
| Positive matrix | 53d464a6f4d3b628d486b7faa7cf30c817983726d2e5f4b1f09ae3697dcf1cc9 |
| Negative obligations | 9b81646bbec321f39af5103243a16674fba681c0176aa06e33894b7c8df603ad |
| Author final freeze | 0dbf1b4b2ac391849667883e0587039d6f4d80608a8a65e00e3a6ae53193949e |
| Author verification controls | 31ab207df1a92da0b8e5ffc157d6259863ed8bb8fa7a4ce2a0dbc0e4c035652a |
| Root complete review/identical promotion | e7ea839ff87ccb92c2630a3cf04ee80cca2804eff4a364bc5742632763c4832a |

Actual Rust1.99 macOS arm64 Rustfmt body check and all92 individual snippet
syntax parses passed; current source/snapshot and final candidate guards passed.
Root checked exact18/12/12 matrix counts,92 unique probes, unchanged promotion
bytes and the public body Rustfmt check. These are parsing/design checks:
Canadian typechecks0, runtime tests0, intended compile-fail diagnostics0.
The body stays outside Cargo under `validation/hand/` until production is
permitted by the [complete gate](M8_CANADIAN_PREPRODUCTION_PROTOCOL.md).
Actual compilation, debug/release/Clippy, negative diagnostics, resource checks
and the full [semantic binding ledger](M8_CANADIAN_TEST_BINDINGS.md) remain
implementation acceptance work.
