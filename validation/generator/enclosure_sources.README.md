# Existing enclosure source inputs: portable projection

`enclosure_sources.py` converts the unchanged sealed schema3 source JSONL into
`validation/fixtures/bonds/enclosure-sources.fixture`. The full original bytes
remain in `validation/enclosure/references/sources.jsonl.gz`. This is an input
transport for the feature's Rust tests, not another source oracle or B2 family.

Use `--out PATH` once to create a file exclusively, or `--check PATH` to reproduce
its bytes using its recorded timestamp/platform. Neither mode reads Rust output
or recalculates coupons, horizons, source ratios or entitlement.

All 30,920 original ids, model labels and constructor literals are retained.
Generic `source_summary` copies its full stored-source expectation. Named
summaries copy the global fields plus actual bundle count, original indices and
payment dates; the full unchanged source corpus retains their rational bundles.
The five FS1 columns are `id model families input source_summary`.

The two structured cells contain explicit typed nodes separated by semicolons:
`key=type:value`. `$` is the root. Dictionary members append `.name`; array
elements append `[index]`. Types are `d` member count, `l` element count, `s`
literal string, `i` integer, `b` boolean0/1 and `z:_` null. Container counts retain
empty containers as well as array order. The original constructor/source strings
contain no semicolon, equals sign or control character; the converter rejects
those rather than introducing an ambiguous field. Decimal integers and binary
bit strings are kept distinct. A consumer uses the typed values directly and
must reject missing or duplicate constructor keys.

Actual public producer tests must construct the selected literal instrument and
prepared plan. Public source getters alone do not establish private cached-flow
identity. The existing source/cache evidence and new complete financial consumer
results retain their separate meanings.
