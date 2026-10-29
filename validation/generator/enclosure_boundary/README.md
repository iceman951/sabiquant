# M8 enclosure boundary reference

This is the single B2 family used by the enclosure feature. It contains 35 requested price-basis assertions, including 11 distinct Generic prepared-source projections. It adds no model or geometry. The immutable input family has 10 new Generic instruments with 14 quote controls, plus three reused named instruments and admitted MAX quotes. Existing broad source, ordinary-price and conditional cash references are reused separately.

The reference is independent of the new financial producer and its outputs. The generator and checker have the same agent author; they are separately expressed implementations, not a claim of two independent people. Neither imports Rust output or the other program. `author.py` uses exact rational quotient rounding; `check.py` finds neighboring values by bisection over the ordered finite IEEE lattice. Decimal point fitting uses denominator factorization in the author and enumeration of the 39 admitted decimal grids in the checker.

`FAMILY.json` and the three-line source/admission extracts are unchanged copies of the frozen reviewed input family. The first private author was frozen before execution. Its source was retained unchanged; this portable author keeps its exact source/target construction and changes the transport to a committed fixture, adding exact floor/ceil and mathematical point classifications. `PROVENANCE.json` records that history and the portable author/checker freeze before first generation. No private directory is needed to reproduce this reference.

Run from the repository root:

```sh
python3 -I validation/generator/enclosure_boundary/author.py --out validation/fixtures/bonds/enclosure-boundary.fixture
python3 -I validation/generator/enclosure_boundary/author.py --check validation/fixtures/bonds/enclosure-boundary.fixture
python3 -I validation/generator/enclosure_boundary/check.py validation/fixtures/bonds/enclosure-boundary.fixture
```

`--out` creates a new file exclusively. `--check` reproduces every byte using the stored timestamp/platform; it never rewrites expectations. Accepted differences must be diagnosed and recorded rather than repaired from producer output.

The UTF-8/ASCII TSV fixture follows TESTING_STRATEGY §7.3. It has these 33 columns:

- Identity: `id model context_id source_plan_id admission_case_id basis compounding yield_bits`.
- Public literal/source binding: `input face_bits accrued_bits coupon_amount_bits flows`.
- Mathematical target: `target_kind numerator denominator coefficient_n coefficient_d argument_n argument_d lower_n lower_d upper_n upper_d lower_strict upper_strict`.
- Export/point consequences: `lower_bits upper_bits range_class point_kind point_coefficient point_scale point_bits`.

`_` means absent. Binary fields are 16 lowercase hexadecimal digits. `input` is a sorted list of `key=value` pairs separated by semicolons; arrays are comma-separated, missing optional entries are `_`, and booleans are `true`/`false`. One nested constructor/schedule field is flattened by joining its names with `_`. It contains the public constructor literals, including explicit dates, policies and caps. `flows` lists `original_index@payment_date@amount_bits@direct_horizon_bits` records separated by semicolons. Separate final coupon and redemption are retained even at one payment date. Source columns are repeated consistently across all relevant quotes and bases, rather than creating another fixture family. Named rows refer to their released exact source/admission IDs and carry the full literal constructor input in `input`.

A rational target is the complete requested per-100 exact real, with reduced signed `numerator` and positive `denominator`. `lower_bits`/`upper_bits` are its tight finite IEEE floor/ceil. `_` at one side means no finite endpoint exists in that direction. For binary64 output bounds, containing this tight pair is equivalent to containing the exact rational; it does not impose arbitrary bit equality on wider sound producer bounds. The reference's `point_kind` states a mathematical fitting classification, not an API proof kind or a promise that arbitrary nonzero-yield identities will be discovered. Decimal is preferred, then exact finite Binary64, then `none`.

The three Continuous controls produce six basis rows with strict range certificates. Each expression is `coefficient * exp(argument)` after exact same-horizon coefficient cancellation. The certificate uses `e > 2`, monotonicity and the fixed integer threshold 2098. Only `2^2098` is constructed, never a shift or loop by the huge argument. Positive/negative tiny targets lie strictly within the stated rational interval and below MIN_SUB in magnitude, proving the IEEE `[0, MIN_SUB]` or `[-MIN_SUB, 0]` pair. The positive huge target is rigorously above MAX; its missing upper finite endpoint is honest. Point classification is `unclassified`; no transcendence theorem or native exp/Decimal approximation is assumed.

The named additions use released zero-coupon-coefficient source geometry: regular integer-lambda Gilt, exact-q/integer-j Treasury, and regular Canadian MoneyMarket. Their MAX quote already has an independently checked exact admission verdict. The new complete rational target is computed from the retained source equation. Source construction/cache validation is reused, not replayed.

Generic cached inputs use exact binary decoding, independent integer RNE for the single rounded coupon product and direct Act365F horizon, and zero accrued amount at the explicit issue/start settlement with no ex terms. Simple denominators are checked exactly, including zero coefficients. Signed terms are summed completely before cached AI/basis and exact `100/face` normalization. API work budgets, proof kinds, outcome variants and old observed evaluations are not target premises or goldens. The new source/price fixture is frozen before any new Generic native probe or financial producer execution. Actual public consumers belong to the feature's integration tests; this reference alone does not close E2 or M8.
