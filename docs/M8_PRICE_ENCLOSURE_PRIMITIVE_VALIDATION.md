# Bounded enclosure arithmetic research validation

Research checkpoint, 2026-10-26. This records an executable dependency of the
[complete price-enclosure proposal](M8_PRICE_ENCLOSURE_DESIGN.md). It adopts no
public SabiQuant API, financial contract or production pricing path. Full generic,
Gilt, Treasury and Canadian ideal-price enclosures remain unfinished.

The separate [research crate and tools](../validation/research/enclosure_scalar/README.md)
implement bounded Rust normalization, signed multiplication/division, coarse
directed addition and finite outward IEEE export. Fixed capacities are Raw34,
working52 words,16-word significands and33-word signed exponent metadata. Proposed
precisions are64/128/256/512/1024. Huge exponent differences use comparisons and
sticky decisions; no huge power or shift is materialized. Helpers write borrowed
outputs and share one531-word scalar frame. This is a research crate excluded
from the library package, with `publish=false` and no dependencies.

## Independent references and actual execution

The selected reference corpus was authored before the first Rust source freeze.
It contains11,727 unique primitive inputs:3,990 normalization,1,641 multiplication,
1,601 division,4,030 addition and465 IEEE export controls. Its exact case bytes
have SHA256 `0f05effa228ed54361c7659c32b9f3fac3b3e6aabcafff175f3fa49b4c3627df`;
input-only TSV is `4742399a23ff7eb8263d22a3324ea836edc6af0d818891e4a8d5664aacbe00e9`.
The generator reproduces these bytes without importing Rust or its outputs.
An independently expressed checker verifies relative Fraction containment, p-grid
adjacency, symbolic huge-gap neighbors, denominator-zero priority and exact
finite IEEE direction/availability. Python integers/Fraction are unbounded
research machinery; they do not establish Rust storage or work bounds.

The initial blind Rust source SHA256 is
`5075b408a83a29b60c5e0ac7c98044ee74a6217fcba979653d8f049d826fb893`.
Expected outputs were released after this source hash was independently checked.
The recorded source continuity proves the final mathematical implementation differs
only by rustfmt and a local reasoned Clippy argument-count allowance. Its SHA256 is
`58bdbea2a60cdbad1a8cc79c5e666ba8087afaee034392cd6b96315f09940597`.
No mathematical correction was needed after seeing the reference outputs.

Actual native macOS arm64 Rust1.99 debug and release consumers each pass all11,727
inputs. Their full outputs are byte-identical. Normalization/Mul/Div require exact
directed pairs; IEEE requires exact finite bits or the unavailable side. Add uses
the proposed coarse maximum-unit policy: its result must contain the tighter
reference pair.560 Add cases are wider while still containing it. Bit equality
with a tighter addition algorithm is deliberately not the contract.

Nine separate postimplementation Rust test functions cover readable1/3 equations,
signed cancellation, IEEE MAX/subnormal limits, canonical zero/denominator priority,
invalid precision and point encodings, work stops and counter overflow. They pass
in debug/release, as do strict prototype Clippy and formatting. These controls are
distinct from the preimplementation independent corpus and from library tests.

Eight real Rust mutations each compile, run to exit0 and fail the unchanged
independent output comparison: lost remainder, lost exponent carry, wrong negative
bound sign, wrong product sign, hidden0/0, wrong Add sticky direction, falsely finite
overflow bound and inward negative IEEE export. An unmodified copied baseline
passes. Compile failures are not counted as semantic kills. These are primitive
faults, separate from prior Canadian financial/resource mutant catalogs.

## Actual layouts and limitations

Native typed sizes are Point400 bytes, Raw560, Frame4,248 and Error1. Inspection of
46 runtime core MIR functions found no owned Point/Raw/Frame or large array local;
compile-time constant initializers are identified separately. A real caller of
all five operations owns one Frame and four Points. Its observed fixed native
frame is6,128 bytes in debug and5,968 in release. Core helper frames and readonly
constant-copy origins are retained in the private evidence. These are individual
frame observations, **not** call-chain/process peak stack or a whole producer proof.

The current primitive work counter charges bounded blocks before execution and
stops before exceeding max_work. All reference inputs use at most326,821 of its
abstract ticks. This is not the proposed full producer's cost table/minimum53680
and does not measure processor cycles. Primitive error returns can leave partial
scratch/output writes; callers must discard them and never turn them into evidence.

Actual whole-arena placement/construction/return paths, interval/log/exp composition,
source admission, exact-point encoding, complete work charges, public errors/traits/
lifetimes, independent dated financial preproduction, cash consequences and full
four-model producer validation remain required. Neither the26,264-byte proposal,
the storage-only marker nor these scalar measurements certify32KiB peak stack or
global allocator events. All library APIs,144/237 catalogs, old behavior and frozen
references remain unchanged by this research installation.

## Additional source and preproduction review

A separate manual audit maps the actual source caches to fixed common-denominator
roles: Gilt3, Treasury3 and Canadian at most5. Constructed exact algebra controls
pass37,800/34,560/376,320 respectively, plus12,096 generic prepared-bit controls.
These460,776 algebra controls are not legal dated scenarios or runtime producer
tests. Source-role divisibility plus the old source-width premises give bounded
zero-yield rational folds. The final Canadian argument adds principal as100*C
separately, avoiding an invalid inference from a reduced terminal denominator;
the earlier proof wording and peer correction are retained.

A private complete preproduction plan enumerates32 configurations, all four models
and source/lifecycle/cash/resource/API layers. Its72 planning obligations are not
adopted requirement IDs or tests already run. Complete price comparisons must
prove nesting in a separately tighter certified reference bracket, or directly
prove endpoint inequalities; mere bracket overlap is insufficient.

The existing library verification remains the `ae7bda7` checkpoint:980 debug and
979 release tests, corrected production coverage, package/API/negative/resource
checks. Research helper cases are not added to those totals or used to increase
the library coverage percentage. Formal financial/source/owner, foreign native,
release and broader M8 acceptance remain open.

## Retained local evidence

The source and failure history remain at these exact local artifacts. New evidence
is additive; the older proposal148 controls and accepted financial corpora are
unchanged. Installed-tool verification is recorded separately from the private
source checkpoint.

| Artifact | SHA256 |
|---|---|
| `/private/tmp/sabiquant-enclosure-primitive-references-v3/final-evidence.json` | `d3e84d5e41e7a8c5f32566a5f1202bb460b7ba315cd189b1cc0c2859f4ffc893` |
| `/private/tmp/sabiquant-m8-enclosure-scalar-v1/final-evidence.json` | `d4874eb5e3637471308387e9b7d9f5bc45f598db59cc7a1509b406a4f83d6073` |
| `/private/tmp/sabiquant-m8-enclosure-scalar-math-peer-v1/evidence.json` | `1c7b8ee1db2e89fd8a4beeed575afd83d3b2025eb200092c31547077da6a23ba` |
| `/private/tmp/sabiquant-m8-enclosure-scalar-mutants-v1/evidence.json` | `adaf88738e5325102031176c41bfde6a0c2445996bc00b644d78b96c1fc65a56` |
| `/private/tmp/sabiquant-m8-enclosure-source-role-audit-v1/final-evidence.json` | `05041e36515da064d8cc21ec99ee2ff05b0f1d69ef7f98ec5ba397ab2e02beff` |
| `/private/tmp/sabiquant-m8-enclosure-preproduction-plan-v1/evidence.json` | `25ab51ba452f0adf0dda62805bfa861094b490a59faf023dfa95cc7e9d37be14` |
| `/private/tmp/sabiquant-m8-enclosure-portable-python-v1/final-evidence.json` | `0e6e2c36e82e4ce32a9fb8cebb6018c8d473d95e2c3989868f2079e15a5067c5` |
