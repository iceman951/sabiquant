# Independent conventional gilt source, date and cash oracles

This corpus was mathematically authored from the frozen [contract](M8_GILT_CONTRACT.md)
before any gilt library production. It covers FR-GILT-01..06/09/15..22 and
contains **10448 distinct serialized inputs** in one schema1
fixture. Source/date/cash decisions are TOL-E0. They are independent expected
results, not executed gilt implementation tests or financial acceptance.

The original producer used the contract, source examples and earlier cash
proof review, without reading new gilt production or Rust financial code
during oracle authorship. Root completed metadata/reproduction/reader checks
after that producer reached its usage limit. Root did not inspect expected
record values while preparing production arithmetic.

## Artifacts and exact model

| Artifact | SHA-256 |
|---|---|
| Frozen contract | `03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292` |
| [Generator](../validation/generator/gilt_source_cash_oracles.py) | `4a4306414acd2874b8d1973ee1994dc298956b55c5225e3df48e8ccb4693b72a` |
| [Fixture](../validation/fixtures/bonds/gilt-source-cash-oracle.fixture) | `21ec34fb1f0c59c321a87a8e4ab782fdab9918e99fbed10d791fee3ad3394d88` |

The primary reference is Python integer/Fraction arithmetic: direct anchored
civil cycles, minimal issue coverage, three first roles, exact half-up
six-place publication, seven unrounded source AI branches and strict-after
entitlement. Standard C/2 is never forced into a scale38 public amount.
Actual zero published coupons remain actual dividends; missing long-first
quasi payments and detached buyer payments keep their distinct date roles.

Each cash operation is derived from exact source and supplied evidence.
Nominal is positive par; all seven signed modes, arbitrary decimal quanta,
RoundTotal and component-round/exact-sum are covered. Universal Exact is
checked on the full continuous price image. It is not inferred from endpoint
divisibility, binary interior enumeration or a midpoint result.

The independent Decimal1536/2048 route separately decodes binary endpoints as
exact integer ratios and evaluates unreduced rational expressions. It checks
source values, binary AI admission, every visited cash quantizer and final
canonical normalization. Explicit continuous nonmultiple witnesses corroborate
119 unresolved Exact stages. The frozen forty-word numerator/denominator
bounds are below2458/1440bits; observed unreduced operands reached2455/1438bits.
At those sizes,1536 digits leave more than300 decimal digits beyond the
worst integer-cell discrimination requirement (combined numerator/denominator
bit lengths). Exact zero, tie and divisibility remain integer authorities.
Dual-precision agreement alone is not the proof of an Exact decision.

## Input schema, outcomes and coverage

The fixture has 38 columns. Complete source definitions are serialized
per row, followed by operation-specific inputs and retained expected metadata.

```text
id, profile, op, issue, maturity, coupon, first, quasi, ex_dates, eom, date_cap, settlement, bundle_cap, dividend_index, recipe, nominal, evidence, aggregation, quantum, mode, expected, context, first_period, first_quasi, dividends, source_values, ai, ai_bits, next_quasi, r, s, n, bundles, entitlements, source_value, lower, upper, guarantee
```

Decimals use coefficient:scale; rational values numerator/denominator; dates
are ISO. Evidence is d:C:S, di:C:S:C:S, b:16hex or bi:16hex:16hex. A hyphen
means unused; an empty date vector is empty text. Context is a structured
key/value list with UTF8hex for text. Source dividend ordinals and absolute
pricing quasi indices retain the contract's separate meanings.

Operations: `cash=5551;dividend_view=1189;instruction_view=35;instrument=24;plan=3649`.
The one direct-cycle date failure records the existing checked-month cause
as a reference; its outer Some(k)/quasi-cycle error is normative. The contract
does not mandate an alternative implementation's DateError construction path.

| Semantic family | Rows |
|---|---:|
| ADJACENT-BINARY-CONTINUOUS-EXACT | 9 |
| AGGREGATION-EXACT-PRECEDENCE | 112 |
| ALL-SOURCE-INSTRUCTION-VIEWS | 28 |
| CASH-39-SCALES-FULL-RANGE | 1092 |
| CIVIL-ENDS-AND-FEBRUARY | 8 |
| CONSTRUCTOR-FAILURE-ORDER | 22 |
| COUPON-39-SCALES-FULL-RANGE | 120 |
| DIRECT-CYCLE-DATE-ERROR | 1 |
| DIVIDEND-INDEX-BEFORE-NOMINAL | 2 |
| DIVIDEND-PUBLICATION | 11 |
| DIVIDEND-VIEW-INVALID-INDEX | 2 |
| DMO-LONG-DAILY-SOURCE | 214 |
| EVIDENCE-TIES-THRESHOLD-PROVENANCE | 315 |
| EXACT-BUNDLE-CAP | 9 |
| EXPLICIT-SIGNED-I128-ADMISSION | 168 |
| FINITE-BINARY-SIGNED-LIMITS-WIDE | 1120 |
| GREGORIAN-400Y-DIRECT-EOM | 3200 |
| LIFECYCLE-AND-STRICT-AFTER | 69 |
| NOMINAL-BEFORE-ZERO-SHORTCUT | 16 |
| PLAN-FAILURE-ORDER | 18 |
| PUBLICATION-HALFUP-TIES | 84 |
| PUBLISHED-ZERO-ACTUAL-COUPON | 1 |
| RECURSIVE-CLAMP-REJECTED | 1 |
| REGULAR-SCALE39-SOURCE | 14 |
| REPRESENTABLE-EXTREMA-INTERIOR-PRECISION-HOLE | 7 |
| ROTATING-EXACT-SOURCE-CASH | 1800 |
| SIGNED-ARBITRARY-QUANTUM-PARITY | 672 |
| SOURCE-AI-NEGATIVE-I128-BOUNDARY | 21 |
| SOURCE-AI-NONDECIMAL-SCALING-RESCUE | 7 |
| SOURCE-VIEW-INEXACT-CASH-SUCCESS | 7 |
| SOURCE-VIEWS-39-SCALES | 1092 |
| STAGE-POINTNESS-AFTER-COLLAPSE | 42 |
| VALID-SOURCE-WIDE-COMPONENT-CANCELLATION | 14 |
| WIDE-SOURCE-PUBLICATION-AND-REGULAR | 80 |
| WIDE-SOURCE-VIEW-RANGE-CASH-RESCUE | 70 |

All39 coupon/nominal/quantum scales, both EOM policies, full positive input
coefficients and signed final limits are exercised. Independent source controls
include the full214-date DMO long-first trajectory,400-year February/August
anchors, source-view failure rescued by nominal scaling, source AI -1/61,
wide component cancellation and representable extrema across precision holes.
Exact source-only receipts have no price-ingredient guarantee. Declared point
provenance remains declared even when its numerical image is a point.

## Reproduction and limitations

```sh
python3 -I validation/generator/gilt_source_cash_oracles.py --check
```

Generation refuses to overwrite existing evidence; --check preserves original
actual generation/runtime metadata and compares every regenerated byte.
Headers pin the generator, adopted contract,24 requirement rows, body and
records with SHA-256/FNV. Future unrelated requirements do not change this pin.

Before first acceptance, the shared Rust reader rejected an uppercase header
key. Root corrected it to ai-branch-counts. The initial unaccepted file is
archived intact under `/private/tmp/sabiquant-gilt-preproduction-verify-v1`.
Root also replaced synthetic/runtime-incomplete draft metadata with actual UTC
and Python/platform metadata. All mathematical body/record bytes are checked
against those archived drafts; no expected row was changed. These corrections
precede freezing this new corpus and alter no prior frozen evidence.

Normal public fixture consumers, every receipt/accessor/error-field assertion,
blind hand cases, actual runtime range/failure-order controls, full strict
regression, coverage and extracted-package checks remain required. This corpus
alone does not test library allocation behavior or prove financial, calendar,
owner, native foreign-platform, enclosure, release or full-M8 acceptance.
