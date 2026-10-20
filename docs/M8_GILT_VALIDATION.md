# Conventional gilt local implementation validation

This record covers the complete conventional UK gilt engineering contract,
implemented locally on `feat/m8-conventional-gilt` after the pre-production
oracle checkpoint `1bada34`. The [adopted contract](M8_GILT_CONTRACT.md),
SHA-256 `03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292`,
is unchanged. This is not full M8, financial, owner, native foreign-platform
or release acceptance. All work remains local; no push, PR or publication was
performed.

## Implemented scope

Eleven types bring the public catalog to **112**: `ConventionalGilt`,
`GiltFirstPeriod`, `GiltPricingPlan`, `GiltValuation`, `GiltRisk`,
`GiltYieldReport`, `GiltYieldError`, `GiltError`, `GiltCashInstruction`,
`GiltCashDecision` and `GiltCashReceipt`. The26 gilt and five gilt-yield error
variants bring the domain catalog to **170**. The public trait census and root
error catalog exercise those additions and typed nested sources.

The complete source model includes:

- explicit minimal, complete maturity-anchored semiannual quasi dates and
  compatible ex-dividend cutoffs, with Regular/Short/Long first roles;
- exact six-place half-up first-dividend publication, unrounded regular
  dividends and seven separate raw-source accrued-interest branches;
- strict-after detachment, actual-payment lifecycle, retained entitled zero
  dividends and final par100 grouping;
- direct observed quasi-period discounts, one IEEE rounding of exact source
  coefficient times accepted discount, and exact dirty reduction;
- all six risk getters from retained extended-exponent terms and exact sums,
  with independent raw and normalized range admission;
- bounded yield inversion with one cached accrued shift, target-fixed
  residual units, counted callbacks and cached selected price payload;
- exact dividend, accrued, dirty and clean-plus-accrued cash, including both
  aggregation choices, all seven signed modes, point versus continuous Exact
  semantics, wide cancellation, complete instruction identity and scoped
  price-ingredient provenance.

The source/date implementation and exact cash were written by separate local
Claude coding peers. Root wrote numerical price/risk/inversion and shared
integration. Those implementations were derived from the frozen contract
without reading oracle expected rows. Consumers were written after production
and use expected values only as comparison data. No QuantLib implementation
was read or translated.

## Independent and supplemental evidence

The [pre-production checkpoint](M8_GILT_PREPRODUCTION_VALIDATION.md) froze all
four fixtures and their generators before any gilt production existed.
Their mathematical records, metadata and generator bytes remain unchanged.

| Corpus | Rows | Actual production boundary exercised |
|---|---:|---|
| [Source/date/cash](M8_GILT_SOURCE_CASH_ORACLES.md) | 10,448 | Public instrument, plan, quantized source views, cash instructions/decisions/receipts and typed failures |
| [Numerical primitives](M8_GILT_NUMERIC_ORACLES.md) | 75,849 | Actual private source-PV, quasi-exponent, rational-to-binary, extended-risk, reduction and residual helpers in an attested temporary exact copy |
| [Ideal price/risk](M8_GILT_IDEAL_ORACLES.md) | 374 | Public price and eight metric views, direct conditional interval membership or typed range outcome |
| [Independent inverse](M8_GILT_IDEAL_ORACLES.md) | 51 | Public target-driven solve, independently conditioned root intervals and cached metadata |

These are86,722 records, not86,722 Cargo tests. Public source/cash uses seven
ordinary integration tests; public ideal/inverse uses three. The private
numeric runner contributes separately counted supplemental tests and never
changes the real library or exposes a test hook in its public API.

The source/cash consumer validates exact schema/census, unique IDs, actual
SHA-256 and byte fingerprints, context, error fields and cause chains.
Unavailable private raw coefficients are not asserted through invented
inspectors: public quantized views and exact cash exercise their consequences,
and the private numerical adapter tests the supplied exact primitives.

The [ideal consumer ledger](M8_GILT_IDEAL_CONSUMER_REVIEW.md) records2,805
finite metric comparisons and11 exact typed range outcomes. Another176 cells
on22 rows are **uninformative**: all fail discount admission before a view on
this host. They contribute construction/context/typed-error assertions only,
with no financial accuracy claim. No informative cell was unavailable on this
host. All51 inverse roots lie inside their independently conditioned bounds.
The eight DMO printed prices and eight printed inverse yields are checked
separately at their printed decimal precision.

The original21 public numerical controls were authored before production and
preserved in `validation/hand/bonds_gilt_numeric_hand.rs`. The active test
copy only adapts private Interval constructors to the public lower/upper-bound
interface; no mathematical input or expected number changed. They cover exact
zero-yield price/all-six-risk anchors, genuine quote/factor/PV/reduction range
failures, independently available normalized views after raw range failure,
target/callback order, counted endpoint payloads, independently conditioned
inverse anchors, concurrent reuse and the actual19,997-period civil maximum.
The [kernel review](M8_GILT_NUMERIC_KERNEL_CHECK.md) independently re-derives
their range and count preconditions.

Seven post-production public controls add explicit2^970 residual-policy
neighbors, scale1 one-callback payloads, honest coordinate precision
exhaustion, not-bracketed endpoint observations, both exponential boundaries,
outer guard sides and both signed tiny-branch boundaries. Their derivation
and limits are recorded in the
[numerical implementation review](M8_GILT_NUMERIC_IMPLEMENTATION_REVIEW.md).
The staging controls check supplied-value arithmetic and solver behavior;
they do not claim a financial root certificate from a computed residual.

The [source/cash implementation review](M8_GILT_SOURCE_CASH_IMPLEMENTATION_REVIEW.md)
retains a separate reproducible Fraction checker authored **after reading
production**. Its3,986 records passed one supplemental debug and one release
test plus strict copied-library/test Clippy. They include half-micro publication
ties, all39 scales/all seven modes, binary subnormals/MAX, Exact stage priority,
wide capital/AI cancellation and lower-versus-net-point endpoint labels.
This strengthens finite evidence without relabeling it pre-production.

Eight additional public identity/context controls cover equal-cash evidence,
rounding/aggregation/cutoff/EOM distinctions, canonical values, complete owned
metadata, independent getter order, upper-endpoint payload counts, positive
subnormal dirty prices, all four owned cash recipes after definition drop and
public Debug without private buffers. Concurrent cash use uses scoped threads
and retains complete instruction/receipt identity.

The private numerical runner also checks272 post-production exact guard
inputs:26 typed boundary rejections and246 passes, including both adjacent
band edges and both exact boundary constants. No exact distance-equals-radius
case was found in that set. Its two supplemental functions run in both
profiles, independently of normal Cargo test counts.

Five source-model mutants were each compiled in a fresh temporary copy and
killed by the public ideal/inverse consumer: factor perturbation, exponent
perturbation, omitted first bundle, omitted clean-target AI shift and wrong
Macaulay numerator. This demonstrates sensitivity to those five faults; it
does not establish general mutation completeness.

The public source/cash consumer separately kills19 compiled production mutants:
publication, long-second ex AI, cutoff equality/fill/windows, nominal/selector
priority, Exact-stage order, endpoint/range labels, guarantee provenance,
source-date context and n. Each has actual GSC row witnesses and a failing test;
compilation failure is never counted as killed. Root verified every log/patch
hash, current original-input bytes, all19 compiled-and-failed results and
7/7 passing controls before and after restoration. Authoritative evidence:
`/private/tmp/sabiquant-gilt-a2-mutation-v2/summary.json`, SHA-256
`514d6a5cbaae105e33f10fd62f2be5263ee49ebd210359459dd1ea4f58a1303b`.
The initial v1 restore-control mtime artifact is documented by the author and
superseded by that rerun. These19 checks and the five ideal checks are finite
mutation sensitivity evidence, not a completeness score.

## Numerical and resource qualifications

The finite elementary profile is conditional. Exact integer/Fraction
references establish supplied-observation rounding independently; they do not
prove every platform's `ln`, `ln_1p` or `exp`. The source exponent uses exact
integer quasi coordinates and a direct single IEEE rational conversion.
The guard rejects a conservative outward band around exponential overflow
and the zero midpoint before observing `exp`. Successful finite factors and
their specified comparison budgets remain distinct from typed operational
factor rejection.

Source prices are per100; annual yields are decimal coordinates. Raw risk
derivatives describe the ideal source model under the adopted observation
profile, not the discontinuous floating program. A computed callback zero or
small cached residual alone does not prove an ideal root. Receipt guarantees
describe only the supplied price-evidence ingredient and never promote an
observed model price to an analytical enclosure.

| Private width | Purpose |
|---|---|
| Five64-bit words | Source rational products, exact exponent/PV conversion and normalized53-bit extended terms |
| 34 words | Exact supplied-binary residual difference and final one-round admission |
| Three243-word magnitudes | Fixed-sign source-model risk sums on2^-8192 |
| Eight words | Combined-decimal exact source/cash quantization |
| 40 words | Exact binary/decimal price evidence, source AI, cash aggregation and final range admission |

The independent implementation reviews inspect the actual expression staging
and last-use storage census against the adopted bounds. Public financial
states are never forged to force excluded branches. Positive final par makes
zero dirty, normalized duration range failures and convexity overflow
proof-excluded on successfully admitted source plans. Cached source AI bounds
exclude clean/target-shift financial overflow; two positive finite cached
amounts exclude unscaled residual overflow. Defensive capacity variants and
uninduced allocation failures remain explicit limits.

The generic bond arithmetic is unchanged. Its ratio-to-IEEE helper gained
module visibility for exact source conversion, and the private Wide type
gained Hash for complete cash snapshot identity. Root errors retain typed
inline causes. Native root error size is160bytes; changing Clippy's inclusive
threshold160 to161 preserves the effective160-byte payload budget. This is
not a layout guarantee on foreign targets.

## Integration verification

The final native battery completed on an unchanged source/test snapshot:
`/private/tmp/sabiquant-gilt-verified-v4/evidence.json`, SHA-256
`c70af435d02ce692db0d65c9d7913bf4e977d962052435accbf3c408dde4a7e6`.

| Check | Result |
|---|---|
| Native debug | **827 passed**,0 failed/ignored,96 groups:117 unit+668 integration+37 executable doctests+5 compile-fail doctests |
| Native release | **826 passed**,0 failed/ignored,96 groups; one debug-only unit is excluded |
| Clippy | All targets, offline/locked, `-D warnings`, exit0 |
| Format and whitespace | `cargo fmt --all --check` and `git diff --check`, exit0 |
| Public/private rustdoc | Both warning-denied builds, exit0 |
| Policy/metadata | Standard-library-only, no unsafe/build script/features/dependencies; alpha metadata preserved |
| Linux GNU x86_64 / Windows MSVC x86_64 | Warning-denied all-target compilation, both exit0 |

Fresh production coverage ran94 binaries/785 runtime tests, with unchanged
source/test snapshots. It measured **16,380/17,174 regions =95.3767%**. The
[complete missing-region review](M8_GILT_COVERAGE_REVIEW.md) classifies every
new-gilt missing region and carries forward the unchanged earlier-module
review. These are unexecuted allocation/defensive/proof-qualified/private
primitive paths, not invented financial-state passes. Supplemental kernel and
Fraction tests are not merged into this standard coverage percentage.
Coverage evidence:
`/private/tmp/sabiquant-gilt-coverage-v4/evidence.json`, SHA-256
`7ba21ea46372482f6a512c6a6b0a37cefda73360f237f9adaedf07f71348b634`.

The final byte audit covers138 frozen fixture/generator/hand/contract files,
all identical to `1bada34`. Another121 original Rust files are byte-identical;
six intentional existing Rust changes are limited to module/public/root-error
integration, the public census, Wide Hash and helper visibility. The audit
asserts the latter two changes are exactly those single edits, preserving the
entire generic arithmetic body. Evidence:
`/private/tmp/sabiquant-gilt-frozen-byte-audit-v1.json`, SHA-256
`81b39ee52c24b000469710a83dbf4c97cc499635b98778faad66f29151aeb088`.
The native battery separately attests all120 pinned validation/gilt-contract
inputs and every current source/test byte.

Earlier exploratory runs remain preserved. V1 completed the then-current
819/818 suite. V3 completed826/825 before the final formatter control. V2
input tests changed while coverage review added identity controls, so its
processes were stopped and its partial results are not final evidence. Final
v4 includes every added test; no expectation or production arithmetic changed
in response to an oracle mismatch.

Final extracted-package evidence (including the final README validation link):
`/private/tmp/sabiquant-gilt-package-v2/evidence.json`, SHA-256
`1adbeedfd3a2c4936be0c9baacd444352765be93b3b58dd4d517856fba0acb5f`.
The52 archive members add only the three gilt source modules to the previous
alpha package. All50 original package inputs match repository bytes. The
extracted crate passes117 unit tests,37 executable and five compile-fail
doctests, strict Clippy, public/private warning-denied rustdoc and metadata
checks. Version remains0.1.0-alpha.1, metadata.publish remains null, there are
no dependencies/features/build scripts and no publication was attempted.

The final warning-denied all-target Linux GNU x86_64 and Windows MSVC x86_64
checks passed. These are compilation checks only. Native runtime evidence
here is macOS arm64 with Rust1.99; foreign runtime/libm behavior and CI are
unverified.

## Remaining acceptance

The source inputs do not certify issued-instrument legal terms, record-date
ownership, UK market calendars or signed cash-policy selection. Treasury and
Canadian conventions, analytical enclosure producers, wider M8 work and
independent financial, owner, native-platform and release review remain open.
The earlier source, generic bond, amount and settlement contracts and their
recorded acceptance limits retain their meanings.
