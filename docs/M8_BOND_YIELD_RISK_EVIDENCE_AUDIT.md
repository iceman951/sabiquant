# Generic bond yield/risk evidence audit

This is an independent audit of the evidence committed at `a33e8a7`, against
the frozen [yield/risk contract](M8_BOND_YIELD_RISK_CONTRACT.md) and
`FR-YR-01..16`. The reviewer read the contract, requirements, Python generator,
fixture, Rust fixture consumer and public hand tests. Rust bond yield/risk,
rate and solver production remained unread throughout oracle derivation.
Findings below concern the evidence; they do not infer a production defect.
The root review separately checks production and corrects living reports.

The original 141-row generator and fixture remain byte-identical. Their pins
are generator SHA-256
`16949a31bf9845bfc72b2dbf2a0c2b2c159efeaaa367c4ec79b2f0a46ad9b087`
and fixture SHA-256
`f8fc3ab55daff1679698bb9c7d05f40cbf94c09a4b0a3be9c2c155c517fca3dd`.
This document describes the consumer/report snapshot before root's corrective
edits; it does not assert that those edited descriptions still contain the
same issues.

## Concrete findings

1. The committed implementation review, validation report and oracle report
   described an obsolete architecture: endpoint-derived residual scaling,
   33/35/37-word arrays, binary64 admission of individual derivative terms,
   eager endpoint work and final repricing. The adopted contract specifies
   fixed target-derived scale 0/1, 34-word residual/constants, five-word scalar
   products, 243-word extended sums, and cached endpoint payloads. Those old
   descriptions and their proof-excluded coverage classification cannot
   substantiate the adopted implementation. Root's source audit confirmed the
   actual 34/5/243 architecture and cached payload; no production discrepancy
   was inferred from the stale descriptions.
2. The old generator's `active_branch` compares SimpleThenCompounded using
   strict `t < exact(1/m)`. The contract uses `t <= RN(1/m)`. For
   SimpleThenCompounded:2 at supplied time `.5`, the generator reports
   Periodic while the contract selects Simple. The mathematical functions
   coincide at exact rational `1/m`; that coincidence does not validate the
   selected branch or its staged metadata. At nonexact reciprocal cutoffs,
   near-minus-frequency conditioning makes the distinction material.
3. The old Simple flow model rejects only PV status `overflow`, admitting a
   nonzero amount rounded to zero. Its directly executed helper accepts
   `A=MIN_SUB,t=1,y=2` with `pv_bits=0`; the existing nonzero PV policy requires
   a signed underflow error. This is a latent oracle classification defect;
   the old finite corpus does not contain that case.
4. The old per-flow Decimal calculations use 160/240 digits, but their ideal
   sums run outside that context at default precision 28. A directly executed
   Continuous, zero-yield helper with equal times 1 and signed amounts
   `[1e40,1,-1e40]` returns ideal moment `2996333109248` and ideal derivative
   `-2996333109248`, rather than `1` and `-1`. Per-flow precision agreement
   cannot repair this aggregation loss. These are valid mathematical signed
   flow inputs to the oracle helper, not a claim that this arbitrary vector
   is constructible as a fixed-rate public Bond.
5. The original consumer checks numerical values, not only signs, which is
   useful. However, its elementary-stage bands of two ULP around a net result
   have no gross cancellation propagation. Its ideal risk comparison uses
   `1e-11*(1+abs(ideal))`, and its ideal-root comparison uses
   `1e-6*(1+abs(ideal_root))`, even for requested `1e-14` solver tolerances.
   Those checks alone do not satisfy the conditioned financial evidence in
   FR-YR-16. Elementary successful rows allow evaluation-count differences
   of two and do not check the expected termination reason. The fixture has
   no expected final bracket, iteration count or full callback trace, despite
   broader claims in the original report.
6. All 32 compounding configurations really occur in the original fixture.
   Its four DCC labels are Bond Basis 93 rows, A365F 16, ISDA termination 16,
   and ICMA 16. This does not cover every scalar/contextual convention,
   Business/252/TARGET/joint calendars, all US flags, or ICMA stubs. The hand
   suite's all-32 inversion test derives its target from production pricing;
   it is a useful inverse/reuse regression, not an independent closed-form
   numerical reference. Its MAX/MIN raw-range rescue controls and signed
   two-root/tangent controls are substantive direct public evidence.

The audit executed these generator counterexamples with the standard library
only. No earlier expected row was edited to make a comparison pass.

## Additive evidence and remaining boundaries

The [extended oracle report](M8_BOND_YIELD_RISK_EXTENDED_ORACLES.md) records new
independent exact residual/term/sum/getter evidence, actual hybrid branch
observations, source-real financial anchors with propagated stage intervals,
all-32 contextual risk, and independent one-flow inversion envelopes. It also
records actual consumer runs, pins and a schedule-construction metadata
erratum. Every new fixture row is consumed; difficult rows are not skipped.

These additions do not turn finite corpus testing into a formal libm bound,
an exhaustive date/market certification or mathematical existence/uniqueness
proof from a computed Brent result. Private-kernel numerical evidence does not
replace public financial construction tests. Root's separate source review,
M6 trace regression, reachable typed-error checks, allocation/resource proof,
production coverage and extracted-package verification remain required parts
of the complete local gate. Named-market models, financial acceptance, native
execution on other operating systems and full M8 acceptance stay open.
