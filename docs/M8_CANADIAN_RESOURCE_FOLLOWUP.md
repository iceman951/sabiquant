# Canadian resource and solver followup

Local engineering followup to `da0f2d4`, 2026-10-25 Bangkok, on native macOS
arm64 with Rust1.99. The complete [contract](M8_CANADIAN_CONTRACT.md), its
fourteen components and all35 admission-frozen files retain their accepted
bytes. No public API, financial reference, source interpretation or numerical
allowance changed. This supplements the original
[implementation checkpoint](M8_CANADIAN_VALIDATION.md).

## Repairs and regression controls

C9 requires normalized numerical division to scan at most53 quotient bits.
The previous scalar alignment could scan a leading zero followed by53
significant positions; its full-width streaming route could likewise feed
one extra leading zero. The divider now classifies that alignment before
the quotient loop, and streaming skips leading zeros using the exact
relative ratio exponent. These preparations use the same three supplied
scratch magnitudes. The arbitrary private scalar quotient still admits64
bits; cash retains its full eight-/40-word division and is unchanged.

Three new private tests observe the actual shared quotient/feed loops with a
borrowed scalar callback. Production supplies a zero-sized no-op callback;
there is no heap hook or global state. Controls cover1/3, full-width
1/(2^(64W)-1) for W=1/5/8/20/40/112/243, both midpoint parities,
significand carry, direct subnormal rounding and MIN_NORMAL carry. Normalized
controls scan53 positions, subnormal controls at most52, and a general
64-bit quotient remains exact. No new financial expectation is derived from
a library observation.

Two selected resource mutants restore the old leading-zero alignment and
streaming leap separately. Both compile, then fail the actual loop-count
regression with54 versus53; the unmodified baseline passes. These are
resource faults, counted separately from financial semantic mutants.

The prior risk initializer used an in-statement repeated array. A strict
source-statement lifetime count could include a243-word seed, a three-array
aggregate and the final three arrays. A private immutable `ZERO_SUMS`
constant removes that ambiguity. Current MIR has no runtime seed or repeat
aggregate; LLVM copies5,832 bytes directly from read-only data to the risk
report. The retained working/transfer budget is six243-word arrays.
The native debug risk frame decreases from31,472 to23,696 bytes, exactly
the removed1,944-byte seed and5,832-byte aggregate. This is a resource
construction repair, not a change to risk arithmetic.

Four new public tests in
[`bonds_canadian_solver_protocol.rs`](../tests/bonds_canadian_solver_protocol.rs)
close FR-CAD-54/55/58 protocol cases. Independent Python Fraction arithmetic
and integer IEEE nearest/even derivations were persisted before Rust ran:

- Evaluation caps1/2/3 stop before the next callback and preserve the exact
  observed best point, residual and bracket.
- Two-call `BracketTolerance` returns the earlier lower endpoint's cached
  price payload, without repricing.
- The flat observed MM image at quotes±MIN_SUB produces the counted
  same-sign `NotBracketed` failure.
- Adjacent valid MM quotes around-300 bracket a target inside a four-ULP
  price gap and produce genuine two-call `PrecisionExhausted`.

All four pass in debug and release, with strict targeted Clippy and
rustfmt. These are postproduction derived protocol controls, not additional
blind hands or ideal-root proofs.

Two further public controls in
[`bonds_canadian_reduction_overflow.rs`](../tests/bonds_canadian_reduction_overflow.rs)
use1,016 regular semiannual periods and annual coupon202 at y=-1.
Independent exact arithmetic gives largest ideal PV201*2^1016 and total
302*2^1016-202. Even a coarse1% allowance derived from the adopted conditional
f2 profile leaves each PV finite and the final sum above the IEEE overflow
midpoint. Actual `evaluate` and `risk` reach the final dirty reduction's
typed Sum/Arithmetic/PositiveInfinity chain; an invoked yield lower callback
preserves that chain with point-1, one function call and no derivative calls.
No reference expectation was fitted to native observations.

The coverage review also led to four existing private test bodies being
extended: zero/zero shifted comparison, whole-word shifts checked on4,000
native u128 operands, canonical extended zero at exponent extremes, and
initially nonzero streamed quotient/remainder cases at all seven widths.
The latter count53 actual loop steps, with exact1 and independently proved
nearest/even2 controls. The wide test-function count remains twelve.

Two additional financial mutants saturate a failed dirty reduction to MAX
or hide a failed callback as the target price. Both compile and the new
public tests kill them by runtime assertion; their unmutated baseline passes.
These are separate from the original22-fault catalogue and the two resource
faults above.

## Resource audit

The audit reads the complete numerical/preparation/source/wide call chain
and checks actual current-byte MIR, LLVM transfers, native layouts and
compiler prologues. No optimizer last-use credit is needed for the logical
wide-payload schedule.
The final wide file differs from the audited file only inside its test
module: the preceding19,840 bytes are identical, SHA-256
`e0d60b8bfb1059883c890a746991c3ae3283ba019f507e60d1363e75393957cf`.
The numerical module is unchanged. This continuity preserves the actual
MIR/layout/reservation audit without repeating it for test-only additions.

| Resource | Actual construction or observation |
|---|---|
| Workspace | Eleven112-word magnitudes; source initializers plus caller destination require22 logical slots during construction |
| Dirty reduction | Two34-word arrays, conservatively counted as two additional112-word slots |
| Residual | Five34-word arrays; even retaining Workspace gives16 padded slots |
| Guard | Six20-word arrays, inside the incorporated twelve-slot allowance |
| Risk | Three working plus three transfer243-word arrays; getters borrow retained sums and own three scratch arrays |
| Wide helpers | Borrow all magnitude buffers; return scalar/Option results, no additional owned magnitude array |
| Prepared bundle |112 native bytes, below128; at most20,000 logical slots |
| Owned cash values | Instruction416, receipt464 and decision464 native bytes, each below2KiB |

The conservative contract ceiling remains42,256 logical payload bytes,
below65,536. Cash's existing ten-by-eight and24-by-forty-word frames remain
unchanged. The metric excludes compiler/ABI overhead as specified in C9;
it is not a claim about peak process stack.

Actual native debug fixed frames include Workspace construction10,000,
evaluate11,872, risk23,696, solve13,472, traversal1,936, risk terms11,344,
getter6,832, residual5,824 and guard3,664 bytes. These are individual emitted
function frames, including page-probe allocations and saves, not a summed
call-chain maximum or foreign-platform guarantee. The report separately
records direct callees and actual wide-array interfaces/transfers.

Safe private-copy instrumentation observes calls immediately before the
real `try_reserve_exact`, then checks vector pointer and capacity after
every push and across callbacks. Production has no instrumentation state.

| Operation | Actual valid bundle counts | Reservation attempts | Later vector growth |
|---|---|---:|---:|
| Preparation |1/17/19,999|one each|0|
| evaluate |1/17/19,999|one PV each|0|
| risk plus six getters |1/17/19,999|one PV each|0|
| Nonzero-yield solve |17|one reused PV; seven callbacks/119 pushes|0|
| Invalid quote/domain, insufficient preparation cap or nonpositive target |Applicable controls above|0|0|

The large valid case supplies20,000 quasi dates. Its initialized bundle
payload is2,239,888 bytes and requested PV payload159,992 bytes. The probe
checks capacity at least equal to the request and no later growth, without
promising allocator-granted capacity equals the request. Source/MIR review
finds no other heap route in these construction/pricing/cash paths.

These are actual reservation-call attempts and no-growth observations,
not process-wide allocator-event counts. Allocation failure is not forced.
Explicit user cloning of Schedule/Bond/Plan can clone their vectors;
user formatting can allocate in its selected sink. The original broad
allocation sentence has been qualified accordingly. Native domain errors
are112/160 bytes; the root wrapper is176. The Clippy threshold remains161,
and its comment now distinguishes returned domain errors from the wrapper.

## Current-byte verification

| Check | Executed result |
|---|---|
| Whole native tree | Debug980 and release979 tests;112 result groups each; zero failures/ignored tests |
| Strict checks | All-target Clippy, rustfmt, warning-denied public/private rustdoc and diff check pass |
| Foreign compilation | All-target warning-denied Linux GNU and Windows MSVC checks pass; no foreign runtime |
| Actual N replay | All3,546 rows in debug/release, including1,219 dated cases,3,388 flow stages and603 seeded exp observations |
| Wide replay | Twelve in-file tests per mode;3,045 frozen helper and952 generic controls,14 test functions including those twelve |
| Negative API |92 separate external compilations reject the intended type/trait with E0277 alone |
| Financial mutation | Original22/22 catalogue killed by runtime assertions:21 tier1, one tier2; two additional final-source reduction/callback faults killed by the new public tests; unmutated baselines pass |
| Extracted package |62 members,60 byte-identical payloads;146 units,43 doctests,8 external API tests, strict Clippy/docs; no dependencies/features |

The full native run includes all S/P/Y consumers,29 frozen public hands,
the previous six public supplements, four new protocol controls and two
reduction/callback controls. The22-fault catalogue ran before the final
test-only extensions; its production prefix and other production inputs are
identical. The two additional faults ran against the final source/tests.
The later required changelog entry changes only a publication input;
numerical source and tests are unchanged. N and package checks were
refreshed for that entry; mutation runs retain their original snapshots.
The resource/layout probe passes two private tests. Guards establish
before/after input equality; they are not continuous filesystem monitoring.

## Coverage correction

The original coverage tool excluded every source region after the first
`#[cfg(test)]` line. In `wide.rs`, the first such item was a small test-only
wrapper before six real production rounding helpers. Consequently, the
old report omitted294 production regions, including20 uncovered regions.
The old profiles and report are retained; their denominator is incomplete.

Reclassifying those same original profiles against exact `da0f2d4` source
gives24,093/25,724 global regions,4,268/4,612 Canadian and825/868 wide,
instead of23,819/25,430,3,994/4,318 and551/574 respectively. This is a
scope correction, not new execution or a test-coverage gain.

[`coverage.py`](../validation/coverage.py) now excludes complete test-only
item spans, retaining later production. Its lexical boundary scanner treats
strings, raw strings, character literals, lifetimes and nested comments
appropriately. It uses half-open line/UTF8-byte-column spans, retaining
same-line production.
It preflights every src Rust file, including files without LLVM rows. Direct
test functions and supported gated fn/impl/module shapes are recognized;
unsupported inner/conditional cfg, const-generic signature ambiguities,
code-generating conditional attributes, gated macros and external source
redirects fail closed. Explicit external support is limited to a real tests
subtree outside src without symlinks or Rust-source redirects. This is a
scoped scanner, not a general Rust parser or macro-expansion model.

Twenty-three Python regressions cover those cases, raw identifiers,
opaque literals/comments, region merging and all current source gates.
They are separate from Cargo test counts. An independent valid-Rust LLVM
probe confirms byte columns and exclusive ends.

The final corrected instrumented run executes937 unit/integration functions
in110 binaries, with no failure or ignored test. It covers
**24,116/25,744 production regions (93.676196%)** globally and
**4,291/4,632 (92.638169%)** in Canadian. Doctests and test-only items are
excluded; no private replay credit is merged into these numbers.

| Canadian module | Covered / regions |
|---|---:|
| Parent |873 /899|
| Cash |1,029 /1,142|
| Numeric |1,269 /1,417|
| Source |274 /285|
| Wide |846 /889|

The final review retains every one of341 missing regions with exact
coordinates, source context and hash bindings. Nothing is removed from the
denominator:

| Classification | Regions |
|---|---:|
| Bounded arithmetic and invariant construction defenses |278|
| Unreachable on admitted public production states |39|
| Closed-guard boundary and propagation |3|
| Allocation failure, not injected |13|
| Already-validated propagation |8|

The private N replay directly binds guard cells and
`requires_exponential`; it does not directly replay the factor wrapper's
guard-error propagation. Historical labels were refined accordingly.
All seven previously valid-unforced regions are now measured covered:
three financial reduction/callback regions and four generic primitive
regions. This is a statement about the reviewed coordinate set, not a proof
of exhaustive inputs, branch coverage or all possible faults.

## Evidence identities

All paths below are local retained artifacts; no remote CI is implied.

| Evidence | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-canadian-c9-loop-repair-v1/final-evidence.json` | `e360c698985ddfb1b78f27520490b3ab7d63944d0de9fc8e58a45231032e1f3f` |
| `/private/tmp/sabiquant-canadian-numeric-resource-implementation-review-v1/FINAL_CURRENT_REVIEW.md` | `a503b9e0c82fdb9ada70de61af89bbce8d36f3e8730fcd1d8c242a147c460fcb` |
| `/private/tmp/sabiquant-canadian-solver-protocol-v1/evidence.json` | `7bf812ef6cd82ef2cef27921ce33e767644dfc442ee0440dc7d137630f2ee9ce` |
| `/private/tmp/sabiquant-canadian-c9-native-v2/evidence.json` | `e14d3bfdf14f179a1dd9273d6a0a178383302a98fcb8f6728ca6f1464a5f9c0d` |
| `/private/tmp/sabiquant-canadian-c9-kernel-final-v3/evidence.json` | `9c7b41e85f53b51c40ad2e6ea975191084311ca14121a8c569c649ba6bc08613` |
| `/private/tmp/sabiquant-canadian-c9-api-negative-final-v2/evidence.json` | `61b1aeb8530ab9fb61205738d232c61f0f4072cc787170de965c9ebf30691442` |
| `/private/tmp/sabiquant-canadian-c9-package-final-v3/evidence.json` | `3bb8b4410b3df50f8e2dd60a5e4b32ec6d5f62b41c21438876e039031ac63527` |
| `/private/tmp/sabiquant-canadian-c9-mutants-v1/evidence.json` | `0bce2cf6c05a5fe3cd848232457e12905b99b3f73be4394586d2ad9c791949ee` |
| `/private/tmp/sabiquant-canadian-c9-resource-mutants-v1/evidence.json` | `35dcceca32d0b070581a8fdb43d57662b6eb9e6b5bdc84b999cc5d7ba42a9a48` |
| `/private/tmp/sabiquant-canadian-c9-numeric-peer-review-v1/evidence.json` | `9678bb75596a7afe66a014eaa7c46f4d450ec447e9eecba4fa8c6331e727bf0b` |
| `/private/tmp/sabiquant-canadian-c9-final-freeze-v1/evidence.json` | `affd8ab342087f3ed2b84adb48dfc50dfa6abf20867d8a78e77ef58e6ec6a544` |
| `/private/tmp/sabiquant-canadian-reduction-overflow-v1/evidence.json` | `7687d97a3b6f539dfa46de470c8d35ad34deac3fc3e49edf3b4fe538a69aeb81` |
| `/private/tmp/sabiquant-canadian-reduction-mutants-v2/evidence.json` | `8445e3189c70f05b57ce6fc0b12ef383fd6957637e9c3fb522c1a587e65d58f4` |
| `/private/tmp/sabiquant-canadian-wide-primitive-gap-controls-v1/final-evidence.json` | `1b9bfcf423da02c11d96905669b19952ceb9357cf086374fea77c8b49692ae92` |
| `/private/tmp/sabiquant-coverage-scope-v2/final-evidence.json` | `91edd9125b7413be0cc898c3a19c2e31af3099d095231fd717339a9f3bd1db3e` |
| `/private/tmp/sabiquant-canadian-c9-coverage-v2-evidence.json` | `889610df5e3a51ebb2f54387ee5b3aa73d2c2cde137aafa48073cdcf434340cd` |
| `/private/tmp/sabiquant-canadian-c9-coverage-v2/production-regions.json` | `08bd63051f4abc7626af2bcaee2482abce2a72663373b6c9bdc4777f507db8e6` |
| `/private/tmp/sabiquant-canadian-c9-coverage-review-v2/current-classification.json` | `1e9475e1268b3bba0dca581507303c4fbcef188f6833d8a07888938eeea19b6b` |

The first probe compilation lacked copied test support and did not run;
the corrected copy passed. The first frame extractor counted a page-probe
loop only once; corrected evidence uses its fixed emitted page count.
Initial coverage-tool regressions also found a macOS symlink-root mismatch
and a conditional lint attribute incorrectly rejected as exclusion; both
are fixed. These failed tooling attempts are not counted as passes.
The first additional financial-mutant runner also refused an ambiguous
two-occurrence anchor before compiling; the corrected callback-specific
anchor was unique. The first scanner revision's same-line/inner/const-generic
and external-module counterexamples are preserved separately; the final
scanner rejects unsupported forms instead of guessing their scope.

Financial/owner acceptance, foreign native execution, maintained calendars,
analytical ideal-price enclosures, release and wider M8 remain open. Work
and commits remain local.
