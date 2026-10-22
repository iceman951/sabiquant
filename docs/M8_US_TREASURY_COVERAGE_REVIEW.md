# US Treasury actual native coverage review

Status: actual guarded coverage **v4 passed**; all **493** remaining Treasury
production coordinates are classified below. The additional public
seed-midpoint control passes the complete native debug/release and instrumented
runs. Its formerly missing coordinate is now covered.

This is a post-production source and coverage review, not a blind financial
oracle. The reviewer authored the cash implementation; independent frozen
hands/oracles and the separate numeric review provide complementary evidence.
No production, frozen expectation, test, Git or remote changes were made by
this coverage review. Only this new document and private audit artifacts were
written.

## Actual measured scope

The matching Rust 1.99 / LLVM23 native macOS arm64 v4 run completed with
101 successful test executables, 871 passing test functions, zero failures,
and unchanged source/runner guards. Doctests are outside this instrumented
pass. The separately guarded native v4 battery passed 914 debug and 913
release tests including doctests; this review does not substitute coverage
for that battery.

| Source | Covered | Production regions | Missing | Coverage |
|---|---:|---:|---:|---:|
| `src/bonds/treasury.rs` | 795 | 862 | 67 | 92.23% |
| `src/bonds/treasury/cash.rs` | 1,043 | 1,152 | 109 | 90.54% |
| `src/bonds/treasury/numeric.rs` | 1,559 | 1,876 | 317 | 83.10% |
| Treasury combined | 3,397 | 3,890 | 493 | 87.33% |

Whole-crate production coverage is 19,801/21,088 regions. Treasury percentages
above come from `production-regions.json`, not raw `llvm-cov report` figures,
which also include embedded unit-test code and have different denominators.

`validation/coverage.py` retains LLVM kind0 source regions, uses the maximum
execution count across duplicate monomorphized coordinates, and excludes
regions beginning at/after the first standalone `#[cfg(test)]` line in each
production file. Expansion, skipped, gap and branch regions are not included.
This is region coverage, not branch, condition or MC/DC coverage. A missing
single-character `?` often denotes only its error-propagation edge, while the
surrounding successful operation is covered. No category below is subtracted
from the denominator, and no adjusted percentage is reported.

The reviewed production bytes are unchanged from the source review and
actual extracted package:

| Source | SHA-256 |
|---|---|
| `src/bonds/treasury.rs` | `4edbd3d4c5478546d4d18ec9ee2ad78707f40945d0859b1d84c853ddab38976e` |
| `src/bonds/treasury/cash.rs` | `47039a9d25b40fe6ac038cd7169faf9195520a88e6866acd615837123f40011d` |
| `src/bonds/treasury/numeric.rs` | `64bd2930cea3ddb156632cd3fa5ce22be6c5fd48192a8ed7136002dd5a2954c0` |

Actual v4 evidence:

- `/private/tmp/sabiquant-treasury-coverage-v4-evidence/evidence.json`, SHA-256
  `9f894ad27d35dca5189d30f3f9500c5a0f178cac44ba7c78e06a1b4aeb28aeee`.
- `/private/tmp/sabiquant-treasury-coverage-v4/production-regions.json`, SHA-256
  `1f51cf5b67a4f89ee7941fdc17f1335283aee546432971254d02e93a6fc2f385`.
- `/private/tmp/sabiquant-treasury-native-verified-v4/evidence.json`, SHA-256
  `67aeec2a3c07a0b9e989647fc3989fb5732f830e47d58b305ff26a2c027d5214`.
- Complete coordinate/reason ledger:
  `/private/tmp/sabiquant-treasury-coverage-classification-v4-final/coordinate-ledger.json`,
  SHA-256 `2a836d019c86d2e6815b09c315e69d2b66283c3abdfbd7303ba1a56c801f95f5`.
  The original v3 reason/source ledger is preserved alongside the exact-set
  v4 comparison; only the now-covered seed coordinate was removed. All three
  production-source hashes are unchanged.

## Classification of every missing coordinate

| Class | Count | Meaning |
|---|---:|---|
| B | 431 | Bounded integer or validated private-invariant defence; failure excluded at admitted public call sites by the source/caller proofs below |
| D | 38 | More general helper branch excluded by its actual financial callers; private primitive supersets can exercise some of these branches |
| A | 13 | Environmental allocation failure or its propagation, left unforced |
| P | 11 | Valid numerical primitive native-suite control paths; actual separate kernel controls cover their arithmetic, without contributing to native LLVM counts |
| Total | 493 | Every actual missing Treasury coordinate, including closures and propagation edges |

B and D are source/caller proof classifications, not an inference from high
coverage or a successful finite sample. The machine-readable ledger assigns
an exact coordinate, function, selected source text, category and concrete
reason to every row. Appendix A retains every coordinate in this document.

### B: bounded arithmetic and private invariants

The adopted [main contract](M8_US_TREASURY_CONTRACT.md), sections E and N2/N5/N6,
and [cash contract](M8_US_TREASURY_CASH_CONTRACT.md), T9, give the required
bounds. The [cash/source implementation review](M8_US_TREASURY_CASH_IMPLEMENTATION_REVIEW.md)
and [numeric implementation review](M8_US_TREASURY_NUMERIC_IMPLEMENTATION_REVIEW.md)
check that the actual fused formulas and helper lifetimes match them.

Parent-source rows retain exact 320-bit caches: raw first coupon136/135-bit
numerator/denominator, raw AI144/143, selected source X161 and published
count147. ExactRatio publication uses the fused162-bit numerator form rather
than multiplying unreduced raw AI by10^6. Daily nearest rounding has positive
denominators and compares remainder to denominator-minus-remainder, so neither
complement subtraction nor count-plus-one exceeds these bounds. Source IEEE
conversion widens to9-word scratch before the shared converter. Prepared
coupon indices and two-date source snapshots derive from already validated
complete quasi cycles; repeated private guards cannot reject valid public
objects. The parent `capacity` constructor itself has four missing regions.

Cash rows use512 bits for source/view work and2,560 bits for price work. T9's
coupon cash382/397-bit numerator/denominator, accrued374/280 and views at most265
fit512. Decoding onto R0 needs2187 bits, invoice addition2190, price rounding
2433/1426 and component-net coefficients2434, all below2560. These are conservative
upper bounds, not claims that every maximum is attained by a dated instrument.
Scale conversions remain within combined scales0..38. Quantum denominators
are positive; signed subtraction occurs only after magnitude ordering. The
missing final-admission branches are defensive normalization failures; actual
Inexact and asymmetric signed-i128 OutOfRange results are exercised.

For the missing upper-endpoint `?` in `round_price`, Exact rejects nonpoint
input before endpoint rounding. A non-Exact endpoint quantizer therefore
cannot produce Inexact; the remaining failures are bounded-capacity defences.
Continuous declared interiors, stage input pointness and the distinction
between total and component invoice recipes retain their adopted semantics.
In particular, total invoices keep the per1000 formula when published AI=0.

Numeric endpoint products need512 bits; quotient alignment can require513 and
uses9 words. Exact seed numerators below2107 bits fit34 words. N5's coupled
same-y/r/s risk proof bounds the complete numerator below311 bits and
denominator below229 within320; arbitrary independent delta/q inputs are not
assumed. Retained risk integers below9367 bits and getter alignment below10187
fit243 words/15552 bits. Actual signed raw range failures and normalized
underflow outcomes remain public tested results; their existence is not
excluded by this capacity classification. Fixed-field casts cannot fail after
masking to11 exponent bits. Missing propagation edges into these helpers
inherit the same bounded validated inputs.

### D: financial caller restrictions

These38 coordinates include nonfinite/zero decoder guards, zero or malformed
normalized ratios, zero dirty in a successful risk object, opposite-sign
addition in the general residual helper, zero/range cases in residual packing,
and impossible clean-shift/reduction failures. Public quote admission precedes
exact seed decoding; accepted positive PV observations and positive dirty
targets precede financial residual work. Zero PV returns zero contributions
before decoding, and zero residual returns before its packer. Thus a generic
helper can support a broader domain than the actual financial caller.

The difference of two positive finite binary64 amounts is finite. Unscaled
nonzero differences are at least MIN_SUB; the target-fixed scale1 is used only
for huge targets, so it cannot introduce subnormal underflow. Published AI is
nonnegative and below2^128, far below the overflow half-ulp at MAX. Adding it
to a finite clean target or subtracting it from positive dirty cannot create
an IEEE overflow. These are narrow caller restrictions, not a claim that
arbitrary signed helper inputs cannot overflow or underflow.

### A: allocation remains environmental

The13 coordinates comprise parent bundle reservation3, numeric reservation4,
evaluate/risk propagation2, and yield Preparation mapping4. They are genuine
fallible allocation failures. This review does not force memory exhaustion,
install a global allocator or claim they are mathematically impossible. Typed
error catalog tests are distinct from executing an actual allocator failure.

### P and the closed public midpoint gap: meaningful arithmetic controls

`endpoint`314–315 has ten missing native regions around a257-bit upward carry.
A valid normalized private pair has significands2^255+1 and2^256-2; its exact
product is2^511-2. The ceil p256 endpoint renormalizes across a power of two.
This is valid primitive arithmetic, not an InternalCapacity exclusion. No
same-y dated public factor-tree witness is claimed.

`factor_observation`589:62–589:74 is the exact-half odd-significand clause. For
M=(2^52+1)*2^203+2^202 with exponent-255, the53-bit observation rounds upward
to significand2^52+2 at exponent-52. Frozen EXT53 controls already exercise
odd-half cases in the separate exact-copy checker, and the post-production
current v7 checker adds explicit positive controls for this case and endpoint carry.
Its actual debug/release/strict-Clippy/guarded result passed:
`/private/tmp/sabiquant-treasury-numeric-kernel-v7/evidence.json`, SHA-256
`091f9521233e6eb06df90ab94c86542a604077cdf9f89ed73047bf267bb57fc3`.
The scope is5593 unchanged frozen rows plus3 synthetic solver-transport
controls plus2 explicitly post-production positive primitive controls per
execution. These external tests are not merged into native LLVM coverage;
they are not additional dated financial instruments or new frozen rows.

`seed_observation`493:27–493:39 had a valid dated public gap in v3 and is
now covered by the complete v4 run. A regular
zero-coupon Treasury from2024-12-07 to2025-06-07 at y=3*2^-52 has the odd midpoint
q=1+3*2^-53 at full-period settlement. RN-even gives q_hat=1+2^-51. At half-period
settlement2025-03-08, q_hat=1+2^-52 while delta_hat=RN(2+y)=2+2^-50. Full-period
risk bits distinguish parity-down; half-period bits distinguish incorrectly
coupling q_hat to delta_hat/2. The latter is not a delta-only discriminator,
since j=0 cancels delta exactly in each complete coefficient.

An independent stdlib Fraction derivation directed-rounds the exact reciprocal
to p256 lower/upper, proves both yield the same53-bit factor, separately rounds
100*factor to the accepted PV, each complete first/second/moment term to
extended53, and retained-term/PV directly to IEEE. It uses no production or
existing generator output for expected values. Full and half dated rows each
pin all six getters and dirty bits. Actual external preflight against the
existing extracted package rlib passed before root installed the supplemental
public test. Focused checks and the complete native/coverage v4 runs all
pass. Reproduction source and byte-identical rerun:

- `/private/tmp/sabiquant-treasury-seed-midpoint-v2/derive.py`, SHA-256
  `ba8b594270a5b39082363f5cb44f7fe6e5860fe38900637c7a60b125d6c1abc7`.
- `/private/tmp/sabiquant-treasury-seed-midpoint-v3/derivation.json`, SHA-256
  `4eab7ff47bad45713186ac2a8fc755cc54b036c7723f31b5d34856396659aab5`;
  byte-identical to the original expected JSON.
- `/private/tmp/sabiquant-treasury-seed-midpoint-v2/evidence.json`, SHA-256
  `07d83937e19bf06af854ae6603aea2eec8e88c430ca091367d4afb8c792295c3`.

## Gaps closed by the actual v3 and v4 runs

Compared with v2, exactly73 previously missing Treasury coordinates became
covered:53 cash Debug,11 risk Debug,7 runtime source-policy inspectors/away
branch,1 negative amount-residual sign, and1 residual IEEE carry. The new
Debug tests exercise all four recipes, valid raw-overflow/underflow risk
reports, bounded public metadata formatting, and every insufficient formatter
byte budget. They do not demand potentially failing raw getters while
formatting. Policy tests retain all four explicit definitions and ordinary
runtime inspector behavior; earlier const-evaluated controls remain useful
but did not count as runtime LLVM execution.

The two solver supplements use genuine admitted one-payment plans: target81
on[0,1] selects cached dirty80 with residual-1, two callbacks, zero trials and
BracketTolerance; target2^-53 on[196,200] returns counted NotBracketed, with
lower exact2-2^-53 rounding to2 through a ties-even carry. No final pricing
callback is hidden by these tests. The earlier coverage v1 never produced a
valid coverage report: it stopped on the historical-policy pin regression;
its incomplete profiles are not part of these percentages.

The v4 run then covers the additional seed odd-half coordinate described
above, leaving exactly493 Treasury coordinates. The complete v3 report and
494-coordinate reason ledger remain preserved as historical evidence.

Coverage is complementary to semantic assertions, the independent four
corpora, frozen66 hands, exact-copy kernel and resource proofs. In particular,
source/cash public consumers bind23 direct/formatted outputs and five public
projections of exact rational expectations; a high region count does not turn those projections
into exact private rational equality checks. Financial acceptance and native
foreign-platform runtime verification remain separate gates.

## Appendix A: every actual v4 missing Treasury coordinate

The source SHA pins above fix the line/column meaning. Coordinates retain
LLVM's exact boundaries. Every row below is in the original denominator;
class P includes nested defensive edges inside an otherwise unvisited valid
primitive carry branch. No branch is relabeled as exercised merely because
an external checker covers related arithmetic.

| Source / function | Class | Missing coordinates (start line:column–end line:column) | Count |
|---|---|---|---:|
| `src/bonds/treasury.rs::with_par` | B | `63:28–63:36`, `63:37–63:48`, `63:91–63:92` | 3 |
| `src/bonds/treasury.rs::new` | B | `406:83–406:84`, `408:89–408:90`, `643:84–643:85`, `648:10–648:11`, `652:32–652:40`, `652:41–652:45`, `652:98–652:99`, `655:88–655:89`, `664:56–664:57`, `666:39–666:40` | 10 |
| `src/bonds/treasury.rs::raw_coupon` | B | `507:39–507:40` | 1 |
| `src/bonds/treasury.rs::source_coupon_formula` | B | `523:39–523:40` | 1 |
| `src/bonds/treasury.rs::new` | A | `661:28–661:36`, `662:13–662:14`, `662:15–662:16` | 3 |
| `src/bonds/treasury.rs::raw_coupon_amount` | B | `790:21–790:29`, `790:30–790:41`, `798:28–798:29`, `801:28–801:29` | 4 |
| `src/bonds/treasury.rs::nearest` | B | `814:19–814:27`, `814:28–814:33`, `814:35–814:40`, `814:42–814:51`, `815:65–815:66`, `816:65–816:66` | 6 |
| `src/bonds/treasury.rs::daily_count` | B | `833:19–833:27`, `833:28–833:33`, `836:26–836:27`, `842:34–842:35`, `850:34–850:35` | 5 |
| `src/bonds/treasury.rs::source_coupon_amount` | B | `872:19–872:27`, `872:28–872:39`, `875:26–875:27`, `887:58–887:59`, `892:66–892:67`, `893:60–893:61`, `900:26–900:27`, `903:26–903:27`, `905:57–905:58` | 9 |
| `src/bonds/treasury.rs::accrued_source` | B | `918:19–918:27`, `918:28–918:32`, `930:30–930:31`, `933:30–933:31`, `936:31–936:39`, `936:40–936:44`, `945:42–945:43`, `950:50–950:51`, `958:50–958:51`, `962:67–962:68`, `964:42–964:43`, `966:70–966:71`, `968:46–968:47`, `969:65–969:66`, `982:6–982:7` | 15 |
| `src/bonds/treasury.rs::source_binary` | B | `1000:19–1000:27`, `1000:28–1000:32`, `1000:34–1000:39`, `1000:41–1000:50`, `1001:67–1001:68`, `1002:71–1002:72` | 6 |
| `src/bonds/treasury.rs::capacity` | B | `1009:1–1013:19`, `1014:5–1018:6`, `1015:9–1015:14`, `1019:1–1019:2` | 4 |
| `src/bonds/treasury/cash.rs::raw_coupon_per_100` | B | `267:41–267:42` | 1 |
| `src/bonds/treasury/cash.rs::source_coupon_per_100` | B | `287:55–287:56` | 1 |
| `src/bonds/treasury/cash.rs::coupon_cash` | B | `324:28–324:32`, `324:33–324:41`, `324:59–324:60`, `331:48–331:49`, `336:58–336:59` | 5 |
| `src/bonds/treasury/cash.rs::accrued_cash` | B | `361:57–361:58`, `364:63–364:64` | 2 |
| `src/bonds/treasury/cash.rs::clean_accrued_cash` | B | `388:57–388:58`, `392:52–392:53` | 2 |
| `src/bonds/treasury/cash.rs::dirty_cash` | B | `416:57–416:58`, `420:58–420:59` | 2 |
| `src/bonds/treasury/cash.rs::settlement_context` | B | `435:18–435:53` | 1 |
| `src/bonds/treasury/cash.rs::accrued_context` | B | `448:28–448:32`, `448:33–448:41`, `448:59–448:60`, `450:54–450:55` | 4 |
| `src/bonds/treasury/cash.rs::new` | B | `476:18–476:53` | 1 |
| `src/bonds/treasury/cash.rs::evaluate_inner` | B | `753:71–753:72` | 1 |
| `src/bonds/treasury/cash.rs::invoice` | B | `774:61–774:62`, `775:72–775:73`, `782:73–782:74`, `790:32–790:40`, `790:41–790:45`, `790:75–790:76`, `795:36–795:44`, `795:45–795:49`, `795:77–795:78`, `799:36–799:44`, `799:45–799:49`, `799:77–799:78` | 12 |
| `src/bonds/treasury/cash.rs::round_price` | B | `834:28–834:32`, `834:33–834:41`, `834:56–834:57`, `843:32–843:36`, `843:37–843:45`, `843:60–843:61`, `853:32–853:36`, `853:37–853:45`, `853:63–853:64`, `860:41–860:42` | 10 |
| `src/bonds/treasury/cash.rs::capacity` | B | `1001:5–1001:64`, `1002:9–1002:17`, `1002:18–1002:28`, `1002:30–1002:40`, `1002:42–1002:51`, `1003:5–1003:6` | 6 |
| `src/bonds/treasury/cash.rs::scale_power` | B | `1017:48–1017:49`, `1018:44–1018:45` | 2 |
| `src/bonds/treasury/cash.rs::quantize_rational` | B | `1033:24–1033:28`, `1033:29–1033:37`, `1033:52–1033:53`, `1038:24–1038:28`, `1038:29–1038:37`, `1038:52–1038:53`, `1042:24–1042:28`, `1042:29–1042:37`, `1042:55–1042:56` | 9 |
| `src/bonds/treasury/cash.rs::quantize_source` | B | `1068:24–1068:28`, `1068:29–1068:37`, `1068:52–1068:53`, `1072:24–1072:28`, `1072:29–1072:37`, `1072:52–1072:53`, `1076:24–1076:28`, `1076:29–1076:37` | 8 |
| `src/bonds/treasury/cash.rs::add` | B | `1109:60–1109:61`, `1114:60–1114:61`, `1119:60–1119:61` | 3 |
| `src/bonds/treasury/cash.rs::decode` | B | `1146:51–1146:52`, `1147:51–1147:52`, `1151:60–1151:61`, `1152:60–1152:61`, `1155:13–1155:50` | 5 |
| `src/bonds/treasury/cash.rs::invoice_total` | B | `1164:64–1164:65`, `1166:28–1166:32`, `1166:33–1166:41`, `1166:58–1166:59`, `1172:32–1172:36`, `1172:37–1172:45`, `1172:62–1172:63`, `1175:32–1175:36`, `1175:37–1175:45`, `1178:37–1178:38`, `1179:37–1179:38` | 11 |
| `src/bonds/treasury/cash.rs::decode_decimal` | B | `1199:24–1199:28`, `1199:29–1199:37`, `1199:57–1199:58` | 3 |
| `src/bonds/treasury/cash.rs::decode_binary` | B | `1212:50–1212:54`, `1212:55–1212:63`, `1212:83–1212:84`, `1218:24–1218:28`, `1218:29–1218:37`, `1218:57–1218:58` | 6 |
| `src/bonds/treasury/cash.rs::round_ratio` | B | `1234:24–1234:28`, `1234:29–1234:37`, `1234:55–1234:56`, `1245:24–1245:28`, `1245:29–1245:37`, `1245:55–1245:56`, `1259:28–1259:32`, `1259:33–1259:41`, `1259:59–1259:60` | 9 |
| `src/bonds/treasury/cash.rs::admit` | B | `1278:28–1278:32`, `1278:33–1278:41`, `1278:60–1278:61`, `1298:56–1298:60`, `1298:61–1298:69` | 5 |
| `src/bonds/treasury/numeric.rs::reserve` | A | `216:13–216:18`, `217:24–217:29`, `218:9–218:10`, `218:11–218:12` | 4 |
| `src/bonds/treasury/numeric.rs::decode` | D | `228:16–228:60`, `228:20–228:28` | 2 |
| `src/bonds/treasury/numeric.rs::decode` | B | `232:57–232:65`, `232:66–232:71`, `232:73–232:78`, `232:97–232:98` | 4 |
| `src/bonds/treasury/numeric.rs::endpoint` | B | `277:19–277:27`, `277:28–277:33`, `277:35–277:40`, `277:42–277:51`, `281:58–281:62`, `281:65–281:66`, `282:60–282:64`, `282:67–282:68`, `288:34–288:35`, `290:66–290:67`, `298:34–298:35`, `306:34–306:35`, `309:70–309:71`, `310:69–310:70`, `312:77–312:78`, `318:62–318:63` | 16 |
| `src/bonds/treasury/numeric.rs::endpoint` | D | `279:16–279:27`, `320:16–320:27` | 2 |
| `src/bonds/treasury/numeric.rs::endpoint` | P | `314:24–314:32`, `314:33–314:46`, `314:50–314:60`, `314:61–314:65`, `314:66–314:67`, `315:26–315:36`, `315:37–315:48`, `315:52–315:62`, `315:63–315:67`, `315:68–315:69` | 10 |
| `src/bonds/treasury/numeric.rs::exact_y` | D | `335:16–335:53`, `335:20–335:28`, `335:29–335:33` | 3 |
| `src/bonds/treasury/numeric.rs::exact_y` | B | `338:57–338:65`, `338:66–338:70`, `338:72–338:77`, `338:79–338:88`, `338:90–338:91`, `347:24–347:32`, `347:33–347:37`, `347:39–347:44`, `347:46–347:55` | 9 |
| `src/bonds/treasury/numeric.rs::prefix_numerators` | B | `364:6–364:7`, `365:19–365:27`, `365:28–365:32`, `365:34–365:39`, `368:26–368:27`, `369:67–369:68`, `370:59–370:60`, `376:33–376:34`, `377:33–377:34` | 9 |
| `src/bonds/treasury/numeric.rs::prefix_numerators` | D | `379:16–379:27` | 1 |
| `src/bonds/treasury/numeric.rs::discard_bits` | B | `399:28–399:36`, `399:37–399:41`, `399:43–399:48`, `399:50–399:59`, `399:61–399:62` | 5 |
| `src/bonds/treasury/numeric.rs::seed_rounding` | B | `421:19–421:27`, `421:28–421:32`, `421:34–421:39`, `421:41–421:50`, `423:90–423:91`, `427:73–427:74`, `428:49–428:53`, `428:56–428:57`, `429:95–429:96`, `431:49–431:50` | 10 |
| `src/bonds/treasury/numeric.rs::seed_endpoints` | B | `441:72–441:73`, `447:23–447:31`, `447:32–447:36`, `450:30–450:31`, `452:30–452:31`, `455:64–455:65`, `456:64–456:65`, `459:64–459:65` | 8 |
| `src/bonds/treasury/numeric.rs::seeds` | B | `469:64–469:65`, `472:24–472:32`, `472:33–472:37`, `472:74–472:75`, `474:33–474:34`, `475:43–475:44` | 6 |
| `src/bonds/treasury/numeric.rs::seed_observation` | B | `481:19–481:27`, `481:28–481:32`, `488:6–488:7`, `492:26–492:27`, `497:64–497:65`, `505:22–505:23` | 6 |
| `src/bonds/treasury/numeric.rs::risk_prefix` | B | `511:53–511:54`, `514:24–514:32`, `514:33–514:37`, `514:63–514:64`, `515:32–515:33`, `515:68–515:69` | 6 |
| `src/bonds/treasury/numeric.rs::endpoint_product` | B | `525:19–525:27`, `525:28–525:39`, `529:26–529:27`, `530:66–530:67`, `531:26–531:27`, `535:24–535:32`, `535:33–535:44`, `535:71–535:72` | 8 |
| `src/bonds/treasury/numeric.rs::product` | B | `549:58–549:59`, `550:60–550:61` | 2 |
| `src/bonds/treasury/numeric.rs::reciprocal_endpoint` | B | `563:24–563:32`, `563:33–563:44`, `563:72–563:73`, `567:24–567:32`, `567:33–567:44`, `567:71–567:72` | 6 |
| `src/bonds/treasury/numeric.rs::factor_observation` | B | `580:19–580:27`, `580:28–580:39`, `581:76–581:77`, `582:75–582:76`, `586:26–586:27`, `587:69–587:70`, `588:70–588:71`, `593:64–593:65` | 8 |
| `src/bonds/treasury/numeric.rs::factor_observation` | P | `589:62–589:74` | 1 |
| `src/bonds/treasury/numeric.rs::discount` | B | `609:49–609:50`, `611:55–611:56`, `614:39–614:40`, `617:66–617:67`, `618:66–618:67` | 5 |
| `src/bonds/treasury/numeric.rs::source_pv` | B | `630:24–630:32`, `630:33–630:44`, `630:70–630:71`, `632:24–632:32`, `632:33–632:44`, `632:83–632:84` | 6 |
| `src/bonds/treasury/numeric.rs::observe` | B | `646:57–646:58` | 1 |
| `src/bonds/treasury/numeric.rs::scalar_product` | B | `659:28–659:36`, `659:37–659:48`, `659:50–659:55`, `659:79–659:80` | 4 |
| `src/bonds/treasury/numeric.rs::scalar_shift` | B | `671:42–671:50`, `671:51–671:62`, `671:64–671:69`, `671:93–671:94`, `674:24–674:32`, `674:33–674:44`, `674:46–674:51` | 7 |
| `src/bonds/treasury/numeric.rs::risk_terms` | B | `697:53–697:54`, `698:56–698:57`, `699:52–699:53`, `704:60–704:61`, `708:14–708:15`, `709:71–709:72`, `709:95–709:96`, `712:32–712:40`, `712:41–712:52`, `712:83–712:84`, `715:32–715:40`, `715:41–715:52`, `715:83–715:84`, `719:56–719:57`, `723:10–723:11`, `727:53–727:54`, `728:56–728:57`, `729:52–729:53`, `734:73–734:74`, `738:14–738:15`, `740:66–740:67`, `744:14–744:15`, `746:60–746:61`, `750:14–750:15`, `754:32–754:40`, `754:41–754:52`, `754:83–754:84`, `757:32–757:40`, `757:41–757:52`, `757:83–757:84`, `761:65–761:66`, `765:10–765:11`, `769:53–769:54`, `770:70–770:71`, `777:10–777:11` | 35 |
| `src/bonds/treasury/numeric.rs::extended_ratio` | D | `790:16–793:11`, `796:16–796:71`, `796:20–796:28` | 3 |
| `src/bonds/treasury/numeric.rs::extended_ratio` | B | `799:22–799:30`, `799:31–799:42`, `799:44–799:49`, `799:73–799:74`, `801:22–801:30`, `801:31–801:42`, `801:44–801:49`, `801:73–801:74`, `807:32–807:40`, `807:41–807:52`, `807:54–807:59`, `807:83–807:84`, `811:28–811:36`, `811:37–811:48`, `811:50–811:55`, `811:79–811:80`, `820:32–820:40`, `820:41–820:52`, `820:54–820:59`, `820:82–820:83`, `828:32–828:40`, `828:41–828:52`, `828:54–828:59`, `828:82–828:83`, `833:24–833:32`, `833:33–833:44`, `833:46–833:51`, `833:74–833:75`, `837:24–837:32`, `837:33–837:44`, `837:46–837:51`, `837:74–837:75`, `840:24–840:32`, `840:33–840:44`, `840:46–840:51`, `840:74–840:75` | 36 |
| `src/bonds/treasury/numeric.rs::accumulate` | B | `865:22–865:30`, `865:31–865:42`, `865:44–865:49`, `865:70–865:71`, `868:24–868:32`, `868:33–868:44`, `868:46–868:51`, `868:72–868:73`, `871:24–871:32`, `871:33–871:44`, `871:46–871:51`, `871:72–871:73` | 12 |
| `src/bonds/treasury/numeric.rs::raw` | B | `896:24–896:32`, `896:33–896:37`, `896:39–896:44`, `896:66–896:67` | 4 |
| `src/bonds/treasury/numeric.rs::normalized` | D | `907:16–910:11` | 1 |
| `src/bonds/treasury/numeric.rs::normalized` | B | `912:53–912:54`, `914:24–914:32`, `914:33–914:37`, `914:39–914:44`, `914:70–914:71` | 5 |
| `src/bonds/treasury/numeric.rs::decode_residual` | D | `920:16–920:65`, `920:20–920:28` | 2 |
| `src/bonds/treasury/numeric.rs::decode_residual` | B | `924:22–924:30`, `924:31–924:35`, `924:37–924:42`, `924:67–924:68`, `933:24–933:32`, `933:33–933:37`, `933:39–933:44`, `933:69–933:70`, `934:36–934:44` | 9 |
| `src/bonds/treasury/numeric.rs::difference` | B | `942:48–942:49`, `943:49–943:50`, `955:36–955:44`, `955:45–955:49`, `955:51–955:56`, `955:81–955:82`, `960:36–960:44`, `960:45–960:49`, `960:51–960:56`, `960:81–960:82` | 10 |
| `src/bonds/treasury/numeric.rs::difference` | D | `946:13–946:14`, `946:15–946:26`, `946:27–946:28`, `947:18–947:28`, `947:32–947:40`, `947:41–947:45`, `947:47–947:52`, `947:77–947:78`, `948:13–948:15` | 9 |
| `src/bonds/treasury/numeric.rs::residual_bits` | D | `975:16–975:21`, `980:16–980:28`, `983:16–983:21`, `1004:16–1004:28` | 4 |
| `src/bonds/treasury/numeric.rs::residual_bits` | B | `977:19–977:27`, `977:28–977:32`, `977:34–977:39`, `978:59–978:63`, `978:66–978:67`, `986:60–986:64`, `986:67–986:68`, `987:93–987:94`, `991:26–991:27`, `1006:64–1006:68`, `1006:71–1006:72` | 11 |
| `src/bonds/treasury/numeric.rs::residual` | B | `1012:65–1012:66`, `1016:73–1016:74` | 2 |
| `src/bonds/treasury/numeric.rs::residual` | D | `1018:16–1018:60` | 1 |
| `src/bonds/treasury/numeric.rs::amount_residual` | B | `1025:65–1025:66`, `1029:54–1029:55` | 2 |
| `src/bonds/treasury/numeric.rs::amount_residual` | D | `1032:16–1032:20`, `1034:9–1034:14`, `1035:5–1035:6` | 3 |
| `src/bonds/treasury/numeric.rs::clean_price` | D | `1041:9–1041:14`, `1042:5–1042:6`, `1042:7–1042:8` | 3 |
| `src/bonds/treasury/numeric.rs::evaluate` | A | `1482:75–1482:76` | 1 |
| `src/bonds/treasury/numeric.rs::risk` | A | `1499:80–1499:81` | 1 |
| `src/bonds/treasury/numeric.rs::risk` | B | `1500:45–1500:46`, `1501:56–1501:57`, `1505:97–1505:98`, `1511:59–1511:60` | 4 |
| `src/bonds/treasury/numeric.rs::yield_from_price` | D | `1550:21–1550:27`, `1552:28–1555:22`, `1556:17–1556:18`, `1556:19–1556:20` | 4 |
| `src/bonds/treasury/numeric.rs::yield_from_price` | A | `1573:86–1573:87`, `1574:17–1577:18`, `1578:13–1578:14`, `1578:15–1578:16` | 4 |
| `src/bonds/treasury/numeric.rs::yield_from_price` | B | `1583:66–1583:67` | 1 |
| `src/bonds/treasury/numeric.rs::dirty_validated_into` | B | `1617:45–1617:46` | 1 |
