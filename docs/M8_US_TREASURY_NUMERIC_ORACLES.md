# Independent Treasury prescribed numerical observations

The [generator](../validation/generator/treasury_numeric_oracles.py) derives
the [numerical fixture](../validation/fixtures/bonds/treasury-numeric-oracle.fixture)
from N1–N8 of the adopted [main contract](M8_US_TREASURY_CONTRACT.md) and
T3 of its [cash component](M8_US_TREASURY_CASH_CONTRACT.md), before any
Treasury Rust implementation. It reads no Treasury production, observed
library results or QuantLib source. Python standard-library integers and
`Fraction` implement the prescribed stages; separate exact cell certificates
and Decimal computations challenge those results. These are **5,593 reference
rows, not executed Treasury library tests**. Root acceptance and the full
[pre-production gate](M8_US_TREASURY_PREPRODUCTION_PROTOCOL.md) are separate.

## Operations and reachability

| Operation | Rows | Expected observations |
|---|---:|---|
| SEED |245| Exact represented-yield validation, outward p256 b/q endpoints, once-rounded delta/qhat and seed widths |
| FACTOR |2,112| Independent left-to-right binary power for each j, reciprocal interval, weights, selected lower-endpoint extended53 value and rounding cell |
| EXT53 |40| Constructed tie/parity/carry cells at ordinary and extreme exponents |
| PV |491| One direct IEEE rounding of exact analytical amount times selected factor; range, ties and double-rounding discriminators |
| AICACHE |1,200| Raw and selected published AI under all four source policies, with once-rounded binary views |
| TERM |1,260| Complete eta/kappa/moment product rounded once to extended53 |
| REDUCE |229| Dated plan PV gates, exact dirty sum, three wide risk sums, all six independent getters and first failed original Q index |
| RESIDUAL |16| Resolved dirty target, sign/range gate, scale selection and exact-difference IEEE residuals |

The reachability labels distinguish **2,631 dated-financial**, **2,160
coordinate-primitive**, and **802 primitive-superset** rows. Dated rows carry
Q0,D,I,F,S,M, EOM, coupon and source policy as applicable. Their valid direct
maturity lattice, first-period shape, D<=I<F and I<=S<M are reconstructed by
the mathematical review. A coordinate label states a valid coupled y/r/s/j
coordinate without claiming a particular dated instrument. Supersets include
constructed endpoints, arbitrary factor/PV pairs, invalid-yield controls,
residual-only inputs and j=19,997/19,999 above the Date-range financial
maximum19,996. They do not establish financial reachability.

The corpus includes Regular, Short and Long definitions, D<I and long I>Q1,
skipped-Q1 indexing, both EOM choices, five selected coupon scales
(0,3,6,7,38), zero coupons and maximum coefficients. The separate source/cash
corpus and hand controls cover all39 input scales. Numeric grids include signed zero, signed MIN_SUB,
next-up(-2), MAX, power-tree boundaries and large exponent separations.
Constructed cells supplement dated public controls; they do not replace them.
The `family` column is a primary test-plan tag selected by grid priority.
Its census does **not** prove that all76 semantic families have been executed
or that every row completely exercises the named family. That later audit
must inspect actual Rust consumers and public outcomes.

## Exact staging and independent certificates

The factor route first constructs b=1+y/2 and q=1+r*y/(2s) from the exact
represented binary64 yield. Normalized p256 lower/upper endpoints are
directed by integer quotient/remainder. Every multiplication and reciprocal
is challenged by an exact operand-product or quotient inequality, including
the predecessor spacing at a binade carry. Each j starts a fresh prescribed
left-to-right binary power; j=0 omits multiplication by one. Operation
weights and the relative-width inequality are retained. Selected rows also
compare a cumulative power tree, without treating that tree as contractual.

For1,802 factor rows, a separately formed exact ideal rational is affordable
and lies between both endpoints. Other rows retain per-operation certificates
instead of expanding powers beyond the generator's1,048,576-bit estimate
for b^j. That criterion precedes multiplication by the q factor; the largest
observed complete ideal operand is1,048,583bits. It is not a bound on every
reference operand or Python memory. No such row loses its prescribed
factor expectation. Interval midpoint classification first asserts a width
strictly smaller than one extended53 unit. The contract selects RNE53 of the
**lower endpoint**, which can be above or below the ideal factor after
rounding; it is not a lower-price promise or correctly rounded ideal factor.

Extended53 certificates check the exact adjacent cell, midpoint equality and
even parity. They rescale by a power of two before forming a `Fraction`, so
extreme factor exponents do not create unnecessary20-million-bit denominators.
This leaves cells, parity and expected values unchanged. IEEE results use
integer remainder decisions for normal/subnormal values, the zero midpoint
and overflow midpoint, checked independently against exact cell boundaries
and CPython's rational-to-float conversion where applicable. Two PV records
explicitly distinguish direct IEEE rounding from an intermediate extended53
product. Nonzero PV underflow/overflow retains the failed original index.

AI uses exact raw pieces and the selected publication policy before either
binary view is rounded. TERM uses once-rounded delta/qhat from the same
yield, then evaluates each **complete** coefficient-times-PV expression and
rounds it once. It does not round individual summands. REDUCE separately
accumulates exact extended terms as integers on basis2^-8192. Each sum's
243-u64-word little-endian encoding is fingerprinted with FNV64. The raw
getter and the direct division by rounded dirty are independently rounded;
failure of a raw view does not suppress normalized views. Dirty is the exact
sum of admitted PV doubles rounded once. The largest observed sum is9,326
bits; this is a sample maximum, not the243-word capacity proof.

Sixteen residual primitives explicitly resolve T=RN(quote+cached AI) for
clean quotes before positivity, threshold and difference decisions. Dirty
quotes use their represented value. The callback residual rounds the exact
dirty-minus-T difference after the prescribed scale; amount residual rounds
the unscaled exact difference. They do not assert Brent callback counts,
cached endpoint selection or public inverse convergence.

Decimal160/240 computations corroborate separated IEEE decisions. Their
near-tie cases are explicitly counted and rely on exact integer/cell
certificates rather than a Decimal tolerance. Counts such as156,582 p256
certificates and10,513 extended53 certificates are internal corroboration
events, not additional independent financial inputs or Rust tests.

## Review and preserved provisional corrections

Root diagnostics found three provisional defects before acceptance:

- Certifying a tiny factor by constructing its absolute `Fraction` could
  spend minutes in enormous GCD operations. Relative power-of-two rescaling
  preserves the exact certificate while keeping it small.
- The upward p256 certificate used a full new-binade unit for the
  predecessor after a carry. The exact predecessor is a half unit there;
  the corrected assertion strengthens certification without changing
  computed endpoints.
- Clean residuals subtracted the unrounded quote-plus-AI sum. The contract
  requires the resolved binary target first. One of16 residual expectations
  changed; earlier generator bytes and diagnostic outputs remain archived.

Independent focused review checked518 normalization cases,40 constructed
extended53 cells,24 selected factor coordinates and all16 residuals. The
full core review separately reconstructed every dated state and checked all
491 PV rows,1,200 AI rows,1,260 TERM rows,229 reductions and16 residuals using
separately authored rational/IEEE witness code. Its detailed final report
records inputs, exact scope and limits. Neither review used future Treasury
Rust outputs as expected values.

The same independent reviewer then swept **all245 SEED,2,112 FACTOR and40
EXT53 rows** with the separately authored witness kernel. Endpoints, selected
extended values, tie/carry flags, expanded operation weights, width and
midpoint classification agree. It also reproduced all1,802 budgeted exact
ideal-factor relations and114 alternate cumulative-tree probes. Relative
cell comparisons avoid absolute giant-Fraction constructions. Together with
the core sweep, every operation's complete fixture partition is independently
challenged; this remains reference verification before Rust execution.

The final schema correction removed derived `reachability` and `shape`
labels from the input digest, gave REDUCE its own `remaining_periods` output
instead of overloading j, and classified five invalid-yield controls as
primitive-supersets. Root's preserved column comparison proves that all other
common cells are unchanged from the completed earlier5,593-row diagnostic.
No previously frozen fixture was rewritten. These provisional changes are
not production bug fixes or library test passes.

## Schema, provenance and reproduction

This schema1 fixture has79 columns. The21-field semantic input projection is
`op,dates,eom,y_bits,r,s,j,c_coef,c_scale,basis,tie,amount_num,amount_den,low_point,factor_hat,pv_bits,measure,dirty_bits,target_kind,target_bits,ai_bits`.
It identifies operation inputs together with supplied dated witnesses where
present; it does not count distinct economic instruments or prove every
primitive numerical coordinate differs. IDs, family/profile/case labels,
reachability, shape and all expected output columns are excluded. All5,593
projected row identities are unique. Removing dated provenance and projecting
the actual prepared/literal operation arguments gives5,517 distinct tuples:
the1,200 AICACHE rows use1,124 distinct arguments, with76 repeats across
different dated witnesses; all other operation projections are unique.
These are retained witness checks, not extra primitive coordinate diversity.
Raw yield-bit identity deliberately retains both signed-zero probes.
Derived `remaining_periods` is an
output of dated REDUCE preparation; j is unused for that operation.

Binary64 cells are `0x` plus16 hex digits. A p256 or extended53 point is a
hex significand followed by `:exponent`, denoting significand*2^exponent.
`dates` uses Q0|D|I|F|S|M; `weights` uses c,o,N. A dash means not applicable.
Headers bind actual generator bytes, both contracts, all26 adopted FR-UST
rows, columns plus records, records alone, input projection, profile and
operation/reachability/status counts. Counts include4,393 TOL-E0 and1,200
TOL-E1 AI-view records. The unchanged shared Rust fixture reader and root's
approved schema registry must verify these pins independently.

```sh
python3 -I -B validation/generator/treasury_numeric_oracles.py --check
```

Initial generation used actual UTC2026-10-20T17:15:36Z on Darwin arm64,
Python3.9.6, and refuses an existing output path. `--check` retains the frozen
timestamp/platform and regenerates exact bytes without writing. Seeded
sampling uses Python floating exponentiation to select represented input
bits; a foreign Python/libm reproduction is not claimed until executed.
The fixture's stored bit patterns are the explicit inputs for later Rust
consumption. Python arbitrary-precision reference allocations, including
large ideal ratios and certificate operands, are not evidence of Rust
workspace usage. `peak_bits` measures selected generator operations only;
the adopted capacity ledger and later implementation/resource audit remain
required.

| Artifact | SHA-256 |
|---|---|
| Generator | `4915101db4c1040702e820570cbe117ceddc72fd55413a7aedd91397256ede54` |
| Fixture | `b24208d7150c65a0ad3b5714f588cfbbd3b6481a511ff086fa94ade132464adc` |
| Records | `7cc711cfb8861eae4066cfff54ef9765a940ad28a4171ffc8847f0d3cb7830e2` |
| Inputs | `7450993faca8c2d20aa93b5b5f7f3bc93674908d9e49f114c475e2ba5f0805ed` |
| Focused private review `/private/tmp/sabiquant-treasury-numeric-factor-review-v1.md` | `7ed3e2e8e5651dab0bfbc3c0ae981e24af201ccf3cc056c0b8bba1d72accfdf4` |
| Root schema delta `/private/tmp/sabiquant-treasury-numeric-projection-delta-v1.json` | `df54424861cf4160c381bc997f12f02e98b7b3abce9577c971f9f1fbac05a589` |
| Independent core `/private/tmp/sabiquant-treasury-numeric-core-review-v1/core-check-evidence.json` | `646e5b077a4359331eef2e7adec2489f049a226404331aecb6cd285a25e7d584` |
| Independent full factor grid `/private/tmp/sabiquant-treasury-numeric-core-review-v1/full-factor-check-evidence.json` | `5e6d7d9dfc66060e572ae85ff415b0bea5f80c232c76ce4c4814bb058432e34e` |

This exact-stage corpus complements the independent
[ideal price/risk/inverse references](M8_US_TREASURY_IDEAL_ORACLES.md),
[numerical hand controls](M8_US_TREASURY_HAND_DESIGN.md), and
[source/cash hand controls](M8_US_TREASURY_SOURCE_CASH_HAND_DESIGN.md).
Only later production-linked consumers, public test execution, complete
regression/coverage/package checks and precisely labelled platform evidence
can establish implementation results. Financial/owner/calendar/enclosure,
Canadian and full-M8 acceptance remain separate open gates.
