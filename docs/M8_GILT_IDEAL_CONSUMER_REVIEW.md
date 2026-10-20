# Gilt ideal-oracle public consumer review

Reviewed on 2026-10-20, Asia/Bangkok, against the local working tree on
`feat/m8-conventional-gilt` (baseline `1bada34` plus uncommitted gilt
production). This report covers the public consumer
[`tests/bonds_gilt_ideal_oracles.rs`](../tests/bonds_gilt_ideal_oracles.rs) of
the two frozen [ideal oracle](M8_GILT_IDEAL_ORACLES.md) fixtures. It is
engineering evidence for FR-GILT-03..13/22, not financial, elementary-function,
platform or release acceptance.

## Authorship and separation

Claude Code wrote the consumer after gilt production existed. It read the
frozen [contract](M8_GILT_CONTRACT.md), the oracle report, the fixture headers
and rows, and the public gilt API. It did not edit any fixture, generator,
contract or production file, and no expected value enters model arithmetic:
expected cells are only compared with public outputs. The consumer is not an
independent oracle; the fixtures are. Operational factor/error order remains
the job of the separate 21 public numeric hand controls.

## Pins and census

For both fixtures the consumer checks:

- the exact column order: 53 price/risk columns and 37 inverse columns;
- the record counts 374 and 51, and the header record counts;
- the FNV-1a fingerprints of the body, the generator, the contract and the
  24 `FR-GILT` requirement rows, each recomputed from repository bytes;
- the pinned SHA-256 header strings of the generator, contract and requirement
  rows;
- the fixture set, domain, schema, Decimal precisions and bisection count;
- the family census, the price metric-status counts and the inverse method
  counts, each recomputed from the records.

The FNV checks recompute bytes; the SHA-256 values are compared as pinned header
strings, because the test crate has no SHA-256 implementation.

## Price and risk rows

Each of the 374 rows rebuilds the public `ConventionalGilt` from its complete
source definition. Construction must succeed, and every retained definition
inspector must equal the input: dates, coupon, end-of-month policy, quasi and
ex-dividend vectors, first period, first dividend index and dividend dates. The
public `GiltPricingPlan` must then report the fixture's next quasi index, `r`,
`s`, `n` and retained bundle indices. Its cached accrued value must be the
correctly rounded binary64 of the exact `ai_num/ai_den`, computed by an exact
integer helper that the consumer tests against IEEE division and ties.

Grouped source coefficients are not public, so they are counted but not
compared; the separate source-cash corpus owns them.

At the represented yield, `evaluate` and `risk` must either both succeed or
fail with the same typed financial range or `NumericalBoundary` error. On
success the consumer checks the canonical yield, the valuation and risk
context, the dirty-price view, and the clean view. The clean view is compared
against the correctly rounded `dirty - accrued`, which is the exact-sum
identity. Each of the eight metrics then follows its fixture class:

| Class | Requirement |
|---|---|
| `finite` | Successful value inside the closed outward interval `[lo, hi]` |
| Signed underflow/overflow | `GiltError::Value` with index None, the getter phase and quantity, the signed infinity or signed zero, and the matching open half-line domain |
| `uninformative` | No financial comparison. The outcome is counted, and any error must be a typed financial range or factor-boundary failure |

Observed census on native macOS arm64, Rust 1.99.0:

| Cells | Count | Rows |
|---|---:|---|
| Finite, interval membership checked | 2,805 | All 352 informative rows |
| Typed range outcome checked | 11 | `GILT-I-00370..00374` |
| Uninformative, counted only | 176 | 22 rows listed below |
| Informative but unavailable | 0 | None |

The eleven typed range cells are:

| Row | Family | Range cells |
|---|---|---|
| `GILT-I-00370` | NEAR-FLOOR-18 | second derivative positive overflow |
| `GILT-I-00371` | NEAR-FLOOR-19 | first derivative negative overflow; second derivative positive overflow |
| `GILT-I-00372` | RAW-UNDERFLOW | first derivative negative underflow; second derivative and convexity positive underflow |
| `GILT-I-00373` | NORMALIZED-ONLY-UNDERFLOW | second derivative and convexity positive underflow |
| `GILT-I-00374` | MOMENT-OVERFLOW | first derivative negative overflow; second derivative and moment positive overflow |

All 22 uninformative rows use the yield `1e100`. In every one, `evaluate` and
`risk` fail identically before any view. The error is
`GiltError::Rate { phase: "discount factor" }` at absolute quasi index 4, 5
or 6, with cause `ValueError::ResultOutOfDomain` on quantity
`discount factor`, value `0` and domain `(0, inf)`. That is, a discount factor
underflows to zero. These rows therefore carry only construction, context and
typed-failure assertions, with **no meaningful financial assertion**:

`GILT-I-00020`, `00080`, `00092`, `00104`, `00116`, `00128`, `00140`, `00152`,
`00164`, `00176`, `00188`, `00200`, `00212`, `00224`, `00236`, `00248`, `00260`,
`00272`, `00284`, `00296`, `00308`, `00368`.

The test does not pin these outcomes: another libm could return a
`NumericalBoundary` or a successful value there. It does require that every
failure is typed and that no informative cell is unavailable on this host.
Factor admission on other platforms is not established.

The eight printed DMO prices are a separate check: each production dirty price
lies within half a printed sixth-decimal unit of its display. This printed
agreement is distinct from the represented-yield ideal budgets.

## Inverse rows

All 51 target-driven inverses must succeed. For each row the consumer checks:

- **Inputs retained:** the original target, the resolved dirty target bits,
  residual scale 0, the bracket, the options and the decreasing sensitivity
  are all preserved.
- **Accrued value:** the plan's cached accrued value is reported unchanged.
- **Solver report:**
  - the reported yield equals the solve point;
  - the evaluation count is between one and the cap, with no derivative calls;
  - the termination reason is `FunctionZero` or `BracketTolerance`;
  - the final bracket lies inside the input bracket and contains the point.
- **Residuals:** both the scaled residual and the amount residual equal the
  correctly rounded difference between the cached dirty price and the resolved
  target.
- **Price views:** the dirty and clean views match the cached payload.
- **Context:** the settlement and quasi context are reported.

The cached root must lie inside the independent conditioned root interval, which
must itself contain the ideal root interval. No planted yield or global ULP
allowance is used. The eight printed DMO yields lie within half a printed ninth
decimal of the root, as a separate display check. Observed: 51 of 51 roots
inside their conditioned intervals.

## Mutation check

Five single-line production mutants were each run in a fresh copy of the
crate under `/private/tmp/sabiquant-gilt-ideal-mutants-v1`. The repository was
untouched, and the unmutated copy passed all three tests. Every mutant compiled
and was killed:

| Mutant | Failing tests |
|---|---|
| Every accepted discount factor times `1 + 1e-12` | price/risk and inverse |
| Scaled logarithm divides by `2.000001` instead of 2 | price/risk and inverse |
| Dirty reduction omits the first retained bundle | price/risk and inverse |
| Clean inversion target omits the cached accrued shift | inverse |
| Macaulay duration divides the first derivative instead of the moment | price/risk |

The relative `1e-12` factor perturbation shows that the conditional budgets are
far tighter than a blanket tolerance. The evidence file is
`/private/tmp/sabiquant-gilt-ideal-mutants-v1/evidence.json`, SHA-256
`e6d9c214071de0c2fe10a3f44085f394e0f4ce54ab2f6e7a443d4bd51285631b`.

## Verification

On native macOS arm64 with Rust 1.99.0 and a private target directory, the
three consumer tests pass in debug and in release. Strict Clippy
(`--test bonds_gilt_ideal_oracles -- -D warnings`) passes and rustfmt is clean.

The full native battery, coverage and package checks belong to root
integration. An earlier all-target Clippy run failed outside this file, with
`result_large_err` at old test helpers in `src/schedule/old_cds.rs` and
`src/schedule/roll.rs`. Root then set the inclusive error-size threshold to 161
for the measured 160-byte error. Root reports that its strict all-target Clippy
rerun passed; its log `/private/tmp/sabiquant-gilt-first-clippy-v2.log` is
empty, with no diagnostics. A separate all-target Clippy run with
`-D warnings` on the updated tree also exited zero here. Root's combined final
battery is still pending.

## Pins

| Artifact | SHA-256 |
|---|---|
| `tests/bonds_gilt_ideal_oracles.rs` | `31048a1d928595aca1fc0ba741a6475dda15dc1e18bcdf1b8194b20af124a22e` |
| Price/risk fixture | `ac6992aae85a2f743a862375e96ee932d268e2b59ce31780a8afebf916551bdf` |
| Inverse fixture | `7ae871f8a64df6898efb87bdeffe94d52ccbd5415b2b6e8742c265f4bf3ebd84` |
| Generator | `eef3f534d615d9bb0ffe40788d7826abb73c0a63f96ab7490a1f03f0253726fe` |
| Contract | `03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292` |
| 24 FR-GILT rows | `290dc8817459d51dbee1264891db255e9f7fa48036fcd9f8950788d267d6a7fc` |

## Remaining limits

- **Not proved:** the consumer does not prove factor admission by every libm,
  rigorous price enclosures or financial acceptance.
- **Not covered here:** exact staging, callback traces, mixed error order,
  constructor rejection and cash recipes belong to the other gilt corpora and
  hand controls.
- **Single host:** the census above is one native host's observation.
