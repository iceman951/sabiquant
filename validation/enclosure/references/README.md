# Frozen enclosure generator inputs

These eleven gzip files are compact, lossless transports of the **same existing
sealed references**, not a new oracle family. `PROVENANCE.json` identifies each
original member, exact uncompressed byte count and SHA256, and compressed byte
count and SHA256. The original JSON/JSONL whitespace, ordering, values,
certificates and trailing newlines are preserved byte for byte on decompression.

| Input | Existing scope |
|---|---|
| `sources.jsonl.gz` | V3 literal construction/source plans, 30,920 rows |
| `prices.jsonl.gz` | Full financial exact/certified targets, 173,448 rows |
| `admission.jsonl.gz` | Exact mathematical admission predicates, 119,631 rows |
| `hands.json.gz` | 19 literal contexts and 43 basis targets |
| `cash-hands.json.gz` | 38 conditional cash controls |
| `hybrid-controls.jsonl.gz` | 144 existing public ICMA controls |
| `price-*-certificates.*.gz` | Existing log, factor, full-target and ln2 certificates |
| `hand-certificates.json.gz` | Existing hand certificates |

Total transport payload: 8,099,896 bytes. The decompressed originals total
175,552,276 bytes. No cache export, native output, duplicate reproduction,
new numerical expectation or extra research corpus is included. Original
schema/freeze metadata is identified by its historical pins in the catalogue;
its local paths remain documentary. Every source plan already carries the
explicit V3 literal construction fields.

Portable byte-identity check, using Python 3.9+ and its standard library:

```sh
python3 validation/enclosure/promote_references.py --check
python3 validation/enclosure/check_bindings.py --require-promoted
```

The first command checks compressed and decompressed byte identities. The
second checks all 73 existing case selectors, ordered ID digests and the full
72-to-18 obligation map using committed inputs, including the legacy cash
fixtures referenced in place. Neither command runs a numerical oracle,
certificate verifier, Rust baseline or public producer.

The original transport can be recreated from the held originals with:

```sh
python3 validation/enclosure/promote_references.py --write --source-root .agents/work
```

Write mode verifies each source pin before compression and creates only
missing assets. Existing different files are rejected, never overwritten.
Gzip uses level9, mtime0 and an empty filename. The committed transport SHA
pins specify the actual archive bytes; checking requires no private work path.

These are **generator input assets**. Actual Rust feature-case fixtures must
use the repository's standard FS1 format. No JSON reader, gzip library, I/O
or dependency is added to the runtime library. Promotion preserves all old
provenance and scope qualifications; it does not turn source/helper checks or
conditional cash values into executions of the new financial producers.
