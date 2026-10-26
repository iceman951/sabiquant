# Bounded enclosure scalar research

This separate, unpublished Rust crate explores five arithmetic primitives needed
by the complete four-model ideal-price enclosure proposal. It provides no public
SabiQuant financial API or certificate and adopts no financial/source contract.
The [resource proposal](../../../docs/M8_PRICE_ENCLOSURE_RESOURCE_PROPOSAL.md)
remains a proposal; the scalar Work meter is a separate abstract research counter.

The std-only core uses M16 significands, E33 signed exponent metadata, Raw34 source
ratios and W52 scratch. Precision choices are64/128/256/512/1024. Normalization,
multiplication and division produce the tight directed pair. Add uses coarse
maximum-unit alignment and must contain the tight pair. IEEE export supplies
finite outward bounds or an unavailable side; zero is canonical positive.
Denominator zero precedes zero numerator after private point validation.

`references.py` constructs11,727 unique generic helper inputs and exact expected
results using Python integers/Fraction, without reading the Rust implementation
or its outputs. The separate `check_references.py` validates Fraction containment,
p-grid adjacency, symbolic huge-gap neighbors and finite IEEE directions without
importing the generator or the bounded core. `transport.py` converts the canonical
JSONL byte-exactly to a38-column TSV and a20-column input-only TSV. `compare.py`
checks a candidate's14-column output; it permits wider Add bounds only when they
contain the tight pair. Python requires only the standard library (3.9 or newer).

Generate references into a new directory outside the repository. All output
parents must exist; writers use create-only mode and refuse existing outputs.
The JSONL/TSV data are reproduced rather than adding large duplicate fixtures.
PROVENANCE.json retains selected v3 and preserved v1/v2 digests and independence
history. These are private helper controls; financial reachability is not asserted.
Existing accepted financial reference files must never be rewritten by this work.

From the repository root, with Rust1.99 on PATH:

```sh
set -C
enclosure_repo="$PWD"
enclosure_tools="$enclosure_repo/validation/research/enclosure_scalar"
enclosure_stage=$(mktemp -d "${TMPDIR:-/tmp}/sabiquant-enclosure-scalar.XXXXXX")
python3 -I "$enclosure_tools/references.py" --repo "$enclosure_repo" --output "$enclosure_stage/references"
python3 -I "$enclosure_tools/references.py" --repo "$enclosure_repo" --references "$enclosure_stage/references"
python3 -I "$enclosure_tools/check_references.py" --repo "$enclosure_repo" --references "$enclosure_stage/references" --output "$enclosure_stage/reference-check.json"
python3 -I "$enclosure_tools/transport.py" --repo "$enclosure_repo" --references "$enclosure_stage/references" --output "$enclosure_stage/transport"
python3 -I "$enclosure_tools/transport.py" --repo "$enclosure_repo" --references "$enclosure_stage/references" --output "$enclosure_stage/transport" --check
CARGO_TARGET_DIR="$enclosure_stage/target" cargo run --offline --locked --manifest-path "$enclosure_tools/Cargo.toml" --bin primitive-probe < "$enclosure_stage/transport/inputs.tsv" > "$enclosure_stage/debug.tsv"
python3 -I "$enclosure_tools/compare.py" --repo "$enclosure_repo" --references "$enclosure_stage/references" --candidate "$enclosure_stage/debug.tsv" --output "$enclosure_stage/debug-check.json"
CARGO_TARGET_DIR="$enclosure_stage/target" cargo run --offline --locked --release --manifest-path "$enclosure_tools/Cargo.toml" --bin primitive-probe < "$enclosure_stage/transport/inputs.tsv" > "$enclosure_stage/release.tsv"
python3 -I "$enclosure_tools/compare.py" --repo "$enclosure_repo" --references "$enclosure_stage/references" --candidate "$enclosure_stage/release.tsv" --output "$enclosure_stage/release-check.json"
CARGO_TARGET_DIR="$enclosure_stage/target" cargo test --offline --locked --manifest-path "$enclosure_tools/Cargo.toml"
CARGO_TARGET_DIR="$enclosure_stage/target" cargo test --offline --locked --release --manifest-path "$enclosure_tools/Cargo.toml"
CARGO_TARGET_DIR="$enclosure_stage/target" cargo clippy --offline --locked --all-targets --manifest-path "$enclosure_tools/Cargo.toml" -- -D warnings
cargo fmt --all --manifest-path "$enclosure_tools/Cargo.toml" --check
python3 -I "$enclosure_tools/test_compare.py"
```

Selected v3 case bytes must hash to
`0f05effa228ed54361c7659c32b9f3fac3b3e6aabcafff175f3fa49b4c3627df`.
The generated20-column input TSV hashes to
`4742399a23ff7eb8263d22a3324ea836edc6af0d818891e4a8d5664aacbe00e9`.
The full38-column TSV hashes to
`142e25d98d09571b3410f0a5163178776ec9fd2558d13b285a8e73916d165d2d`.
Metadata manifests include their producing script digest, so CLI portability
changes those manifests while preserving mathematical data bytes.

Candidate output follows the Rust harness's exact14-column, ASCII/LF, unquoted
TSV protocol. The comparator rejects unknown/reordered columns, malformed or
noncanonical signs/u64 words/work counters, CRLF and missing final LF. The15
Python protocol regressions above cover those input-integrity rules separately
from mathematical comparison and Rust execution.

The initial blind Rust math source was frozen at5075b408a83a29b60c5e0ac7c98044ee74a6217fcba979653d8f049d826fb893
before compilation or expected access; formatting and one Clippy argument-count
allowance preserve its mathematics. Reference v1/v2 predate that implementation;
v3 boundary additions were authored independently while the core remained blind.
Reference construction/review did not read the Rust source or any candidate output.
Expected data were released only after the frozen source hash was independently
checked. Subsequent comparisons and source reviews are recorded separately.

The stdin/stdout harness allocates and performs I/O; it is outside the arithmetic
core. Work charges are prepaid bounded limb/block actions, not CPU cycles or the
proposed complete producer charge schedule. Outputs can be partially changed on
an error: callers may consume them only after Ok. Controls cover malformed values,
precision, work limits/overflow and native type sizes. Sizes are not full caller
stack measurements. This research does not establish zero allocation, peak stack,
foreign runtime, log/exp, source conversion, financial model integration, complete
admission/refinement, exact point encoding or settlement-evidence correctness.
