# Existing enclosure controls in FS1

`enclosure_controls.py` transports the existing H43 price hands, HC38 conditional cash hands and HYBRID144 public ICMA controls from their committed gzip assets to ordinary FS1 fixtures. These are the same existing families, not new expectations or a second B2 boundary family. No mathematical equation, source value, elementary function, certificate, price, cash image or horizon is evaluated. All 43 hands already have tight IEEE lower/upper fields; the converter copies them unchanged.

The generator and input/dependency hashes are frozen in `enclosure_controls.provenance.json` before the first output. It imports only the existing `flat(value)` literal encoder from the pinned `enclosure_sources.py`. All gzip and decompressed bytes must match the frozen assets. Every projected row is reconstructed to its original JSON value and compared exactly before writing. The program uses exclusive creation; `--check` reproduces the same bytes without rewriting a fixture.

From the repository root:

```sh
python3 -I validation/generator/enclosure_controls.py --family hands --out validation/fixtures/bonds/enclosure-hands.fixture
python3 -I validation/generator/enclosure_controls.py --family hybrid --out validation/fixtures/bonds/enclosure-hybrid.fixture
python3 -I validation/generator/enclosure_controls.py --family cash-hands --out validation/fixtures/bonds/enclosure-cash-hands.fixture
python3 -I validation/generator/enclosure_controls.py --family hands --check validation/fixtures/bonds/enclosure-hands.fixture
python3 -I validation/generator/enclosure_controls.py --family hybrid --check validation/fixtures/bonds/enclosure-hybrid.fixture
python3 -I validation/generator/enclosure_controls.py --family cash-hands --check validation/fixtures/bonds/enclosure-cash-hands.fixture
```

H has 21 columns:

```text
id context_id model basis input source_exact metadata_expected reference_kind lower_n lower_d upper_n upper_d outward_lower_bits outward_upper_bits certificate_id canonical_decimal_point declared_test_encoding equation derivation future_work_policy preferred_encoding_qualification
```

The row ID is the original context ID followed by `:` and the original basis. The 19 original context IDs, model spelling and basis order remain unchanged. `input`, `source_exact` and `metadata_expected` use typed nodes. `lo` and `hi` numerator/denominator strings and the preexisting tight IEEE fields are copied directly. Certificate references remain `e` or `sqrt2` and bind the original committed hand certificate asset; null is `-`. All original target and explanatory fields are retained. For certified targets the actual consumer must contain both original rational reference endpoints; mere overlap or a point inside the bracket does not prove the target.

HYBRID has 27 columns, preserving every original record key:

```text
id method quote_frequency m threshold_bits threshold_bit_offset label issue settlement payment face_literal redemption_literal trial_yield_literal accrual_frequency accrual_events_per_year periods overlap_days reference_days exact_unrounded_horizon expected_horizon_bits expected_branch_from_adopted_rule expected_dirty_amount_bits expected_moment_and_macaulay_bits expected_entitled_flows expected_accrual_work neumaier_trace market_scope
```

`periods`, `overlap_days`, `reference_days` and `neumaier_trace` use typed nodes. All original IDs, explicit public context periods, source literals, trial zero yield, expected horizon bits, selected branches and threshold offsets are copied unchanged. Nullable quote frequency, count, threshold bits and offset use `-`. Integers remain integers on reverse projection. These controls establish only their original public selected-basis reachability, not global public-API unreachability or named-market geometry.

HC has the ten original record keys as columns:

```text
id case basis recipe nominal quantum mode aggregation evidence expected
```

`expected` uses typed nodes and keeps every original decision/amount/error/provenance field. Null aggregation and null price basis are `-`. Four existing source_coupon controls have no price basis or price-evidence guarantee; they link only the original context. Evidence and other explanatory strings stay plain TSV text; semicolons and equals signs in such text are not typed-node delimiters. Every price-bearing case/basis links an existing H target; source-only cash controls retain their null basis and guarantee. The evidence remains conditional; transport creates no PriceEnclosure or stronger receipt guarantee.

Typed nodes are the existing lossless source encoding: semicolon-separated `path=tag:value`, root `$`, dictionary `d:member-count`, list `l:element-count`, string `s:text`, integer `i:integer`, boolean `b:0`/`b:1`, and null `z:_`. Dictionary members use `.name` and lists use `[index]`; root dictionary member names omit the leading dot. Containers and their sizes are explicit. The decoder checks complete members, order and exact re-encoding. Only constructor/source/structured expectation cells use this encoding. No JSON or gzip parser is needed by the std-only Rust fixture reader.

FS1 text is UTF-8 TSV under TESTING_STRATEGY §7.3. Each fixture has all required metadata, input/code/provenance hashes and its exact row count. Any malformed transport or byte mismatch is diagnosed without changing sealed input values. The converter performs no native evaluation, oracle rerun, reference refinement, new-family generation or B2 edit. Actual public source/selector/price/cash assertions are feature consumers, not results implied by this transport.


The first H43/HYBRID144 projections succeeded and remain byte-identical. The first HC conversion failed before file creation because it incorrectly required every cash record to have a price basis. Four original source_coupon controls legitimately have basis:null and price_evidence_guarantee:null. The minimal correction keeps those values and their source-only context links; original numerical data is unchanged. First source/provenance/schema/failure are preserved in `enclosure_controls_history/`.

The existing H/HYBRID headers truthfully retain their first generator/provenance hashes. Reproduction validates the actual recorded first source and provenance against fixed known hashes, their input/dependency bindings, and byte-identical shared/H/HYBRID projection function source. It also requires the entire rendered H/HYBRID fixture to match its known first output hash. It never accepts arbitrary header pins or merely trusts an unchanged flag. HC uses the corrected source freeze. There remains one active converter and one actual family per mode.
