# Four-model price enclosure preproduction

E2 closes with the local signed-off unit containing this report, its frozen
inputs, protocols and focused public tests. E1 is already committed at
`13901b1` under the adopted [contract](M8_PRICE_ENCLOSURE_CONTRACT.md).
E3–E5 remain open: no public financial enclosure producer, originating
producer-to-cash execution or whole-producer resource result is claimed here.
Engineering closure does not establish financial, owner, native-platform or
formal M8 acceptance; see the [completion checklist](M8_COMPLETION_CHECKLIST.md).

## Finite B1–B5 exit evidence

| Action | Delivered evidence |
|---|---|
| B1: bind existing expectations | [Consumer manifest](../validation/enclosure/consumer-bindings.json) and [guide](../validation/enclosure/CONSUMER_BINDINGS.md): all 72 EP planning obligations map to 18 FR-ENCL groups and 73 exact existing selectors, with original IDs, predicates, ordered-ID digests and future public assertions. The existing direct-horizon witness is `ENC-SOURCE-002273`. |
| B2: fill only missing financial boundaries | One [boundary fixture](../validation/fixtures/bonds/enclosure-boundary.fixture), [family/derivations](../validation/generator/enclosure_boundary/README.md) and [provenance](../validation/generator/enclosure_boundary/PROVENANCE.json): 35 complete basis targets, comprising 29 rational and six strict range targets; eleven Generic prepared plans and three reused named inputs. |
| B3: freeze logical outcomes | [Protocol](../validation/enclosure/PROTOCOL.md) and [machine specification](../validation/enclosure/protocol.json): constructor, admission/error/progress, symbolic debit cutpoints, complete-target flags, exact source-role premises, preferred point fitting and proof/encoding combinations. Actual cumulative work remains an E3/E5 execution fact. |
| B4: freeze comparison and useful ordinary results | [Comparison rules](../validation/enclosure/COMPARISON.md) and [policy](../validation/enclosure/comparison.json), with standard FS1 source/price/admission/control projections and current provenance. Require exact containment or certified-reference nesting; overlap and old observed equality are insufficient. Ordinary p64 uses the explicit test allowance of `10^12` work units, with a separately selected width criterion. |
| B5: bind unchanged cash consequences | The consumer manifest binds HC38 and existing settlement/Gilt/Treasury/Canadian cash profiles to future compatible originating evidence. Seven modes, arbitrary quanta, aggregation, source stages, universal Exact, ambiguity and receipt provenance remain unchanged. Generic Clean requires independently exact `100*cached_AI/face`; Canadian ModelDirty is excluded from SettlementDirty cash. Execution belongs to E4. |

These are finite specification/reference actions. Their planned public
producer assertions are not reported as executed merely because the input
bindings and fixture readers pass.

## Portable inputs and chronology

The same existing independent families now have portable inputs and FS1
projections: [S30920](../validation/fixtures/bonds/enclosure-sources.fixture),
[P173448](../validation/fixtures/bonds/enclosure-prices-v2.fixture),
[Q119631](../validation/fixtures/bonds/enclosure-admission.fixture),
[H43](../validation/fixtures/bonds/enclosure-hands.fixture),
[HC38](../validation/fixtures/bonds/enclosure-cash-hands.fixture) and
[HYBRID144](../validation/fixtures/bonds/enclosure-hybrid.fixture).
H43 represents 19 literal contexts with 43 basis targets. P has 136,096
exact-rational and 37,352 independently certified interval targets over the
same 30,920 source plans. Its 65,964 certificate contexts are source-plus-quote
contexts, not additional prepared plans.

Eleven [compressed input/certificate assets](../validation/enclosure/references/README.md)
retain the original logical bytes and SHA256 pins. They total 8,099,896
compressed bytes and reproduce 175,552,276 original bytes. They are generator
inputs; Rust tests use FS1 through the unchanged shared reader, with no runtime
JSON/gzip dependency. Source/price/admission/control projection records retain
input, generator, certificate and row identity. Old numerical references were
not regenerated to fit a producer.

The existing S/P/Q/H expectations predate new financial producer code. The
single B2 generator and separately expressed checker were frozen before their
first execution and before its new public Generic source probe. The **same
agent authored both B2 programs**; the checker imports no author implementation
and uses separately expressed arithmetic. This establishes independence from
new Rust financial output, not two independent human authors or producer-author
blindness. Transport authors already knew the frozen values. B2 adds only the
identified boundary triggers; it does not rebuild a Cartesian price/cash family.

Actual transport failures are preserved. The first HC conversion rejected four
legitimate source-only controls with null basis before creating output. Price
projection first encountered signed hexadecimal exponent metadata, then its
first public FS1 read rejected uppercase column names. The active price v2
uses lowercase `reference_*_k`, retains signed-hex K, and has byte-identical
numerical data rows to preserved v1. Source/provenance/failure history and
[row identity](../validation/generator/enclosure_prices_history/V2_ROW_IDENTITY.json)
remain available; no expectation or shared-reader rule was changed to hide
those failures.

## Executed focused checks

The following native macOS public test targets pass in both debug and release:

- [Boundary sources](../tests/bonds_enclosure_boundary_sources.rs): one test
  consumes all 35 boundary rows and constructs the eleven Generic plans through
  actual public APIs; reused named literals/IDs remain linked. It does not
  inspect private caches or evaluate a new financial producer.
- [Fixture integrity](../tests/bonds_enclosure_fixture_integrity.rs): two tests
  consume all seven FS1 tables, check complete typed rows/cross-links and stream
  the recorded writer/archive fingerprints. These are integrity assertions,
  not financial equation or analytical containment execution.

Focused strict Clippy, format and diff checks pass. Portable transport and
selector checks also pass without a private work directory. The new B2
reference passed first generation, byte reproduction and its separately
expressed mathematical check. Existing S/P/Q/H/control projections passed
their recorded lossless conversion and reproduction checks. No old native
baseline, numerical verifier or full helper matrix was rerun for this report.

The actual full financial consumers must now implement each model partition,
retain every row's origin/basis/quote/unit identity and obey the frozen
containment/usefulness rules (E3). Produced evidence must then enter the same
origin's exact cash recipes (E4). Actual debit cutpoints, allocation, liveness,
return paths and final integrated regressions/package/coverage remain E5.
