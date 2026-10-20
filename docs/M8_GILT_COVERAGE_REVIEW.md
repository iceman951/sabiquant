# Conventional gilt production-region review

Coverage measures executed LLVM source regions with embedded test modules
excluded. It is not branch coverage, an exhaustive-input proof or an ideal
financial accuracy certificate. The [validation record](M8_GILT_VALIDATION.md)
and [requirements map](M8_GILT_EVIDENCE_MAP.md) give the separate numerical,
source, error and acceptance evidence.

## Source identity and measurement

| Production file | SHA-256 |
|---|---|
| `src/bonds/gilt.rs` | `ae35bd7a7d74203ebfbd8765026f65ff63216d8216f80e1c0af37dc73345c001` |
| `src/bonds/gilt/cash.rs` | `ec07451e86975910bddf7af171a69db7705714df72677399fac2a887702e3135` |
| `src/bonds/gilt/numeric.rs` | `1c53413faa374a4048391b89333ab6032b54394816d5de83d412fc6eafbe148d` |

The fresh final v4 measurement executed **94 native binaries and785 runtime
tests**, with source/test snapshots unchanged before and after execution.
It measured **16,380/17,174 production regions =95.3767%**. The three new gilt
modules cover2,816/3,156 regions; their340 missing regions are classified below.
Doctests run separately in the native verification battery.

Final evidence:
`/private/tmp/sabiquant-gilt-coverage-v4/evidence.json`, SHA-256
`7ba21ea46372482f6a512c6a6b0a37cefda73360f237f9adaedf07f71348b634`.
The production-region JSON SHA-256 is
`28c72d8a8c86852d7f1b943ce9b8f6fca891d3be917b801dd6a2b942da776956`.
The tuple-by-tuple `gilt-uncovered-classification.json` SHA-256 is
`6aba1f8cf907c2f8047a80aab404385b12a87809353d2ea391cca0d3229ab586`.
It retains every start/end line/column tuple and source text.

Coverage review drove additional public tests: v1 exposed metadata/Debug gaps;
v3 confirmed their closure except three cash Debug regions at `cash.rs:697`.
The final all-recipe formatting test closes those three regions in v4. Public
quantized source views use a separate dispatch and were already exercised.
No arithmetic or frozen expectation changed during these additions.

## Every missing gilt region

The counts below are the final340 missing regions. Multiple columns and propagation
edges on one source line are distinct LLVM regions. The JSON retains every
start/end line/column tuple and source text; the tables list all start lines.
None of the excluded regions is counted as executed.

| File | Covered / regions | Missing |
|---|---:|---:|
| Source/date | 748/779 | 31 |
| Exact cash | 1,085/1,219 | 134 |
| Numerical model | 983/1,158 | 175 |

| Classification | Regions | Reason / evidence |
|---|---:|---|
| Source allocation | 6 | Two fallible vector reservations. Actual allocation failure was not induced. |
| Source capacity/invariant propagation | 25 | Validated day denominators, full-C publication, exact source/PV preparation fit the adopted five-word proof. |
| Cash capacity/invariant propagation | 133 | Closed validated snapshots/evidence and the eight-/40-word proof exclude these failures for valid finite inputs. |
| Private published-source day branch | 1 | `SourceValue::day` is used for price invoices, whose source is raw AI. Published dividend recipes use the separate source quantizer. |
| Numerical allocation/propagation | 10 | PV workspace reservations and their wrapper propagation. No OOM or modified global allocator was used. |
| Fixed-policy/financial exclusions | 19 | Semiannual compounding construction, finite scaled logarithm, positive dirty, bounded clean/target shifts and positive-operand amount residual. |
| Numerical capacity/invariant propagation | 131 | Five-/34-/243-word proof, positive represented delta and exact valid scalar/reduction/residual staging. |
| Private primitives / positive-model exclusions | 15 | Internal zero/carry/sign branches distinguished from public financial reachability below. |

### Source/date locations

Allocation:499,500,776,777.

Capacity or propagation:102,122,131,134,137,166,167,171,175,180,183,184,
507,510,767,769,792,797. The publication/source numerator and denominator
are bounded before scalar operations; source AI's direct IEEE conversion is
within range. Parent and bundle `?` error edges cannot fail after validated
source inputs under those bounds. This is a proof qualification, not a forced
private-state test.

### Cash locations

Capacity/invariant propagation:294,316,340,361,364,387,391,412,416,434,449,
454,476,671,716,722,727,751,752,753,756,757,764,769,773,796,798,799,802,
806,812,822,829,962,963,964,1002,1006,1017,1042,1045,1048,1050,1056,1086,
1090,1093,1097,1101,1112,1134,1159,1164,1169,1208,1209,1213,1214,1217,
1226,1229,1230,1251,1264,1271,1288,1299,1313,1334,1354.

The snapshot lookups cannot fail for constructed instruments/plans. Sealed
SettlementEvidence always exposes exactly one supported coordinate kind.
Nominal and quantum are positive; every division denominator is nonzero.
Signed additions/subtractions retain their ordering/carry, and coefficients
remain below405 or2460 bits. Propagated quantization failures in these listed
regions are private capacity failures. Public Inexact/OutOfRange outcomes,
Exact-unresolved decisions and final endpoint range failures are separately
executed by the frozen public corpus and supplemental Fraction checker.

The unused published-day branch is1028. It is a valid private helper branch,
not a missing supported invoice shape: source AI is never a published dividend.
The public Debug source dispatch at697 is now covered for all four recipes.

### Numerical locations

Allocation or propagation:225,226,227,1040,1058,1130,1131,1135.

Fixed-policy/financial exclusions:209,211,212,319,321,322,548,639,641,642,
648,649,1108,1110,1114. Semiannual is a supported fixed frequency. Successful
quotes give finite logarithms under the adopted elementary profile. Positive
redemption makes dirty positive. Exact source AI has magnitude below2^127,
excluding IEEE clean/target-shift overflow. Two positive finite cached values
exclude unscaled residual overflow. No invalid report was created to visit
these defensive branches.

Capacity/invariant propagation:233,237,254,267,270,273,323,344,348,350,388,
391,393,399,403,412,420,425,429,432,457,460,463,472,490,491,495,496,500,
502,503,507,509,514,542,556,558,564,568,577,586,587,599,604,613,622,624,
631,636,1063,1069,1140. These include the callers' error-propagation edges,
not additional financial range failures. In particular, target-fixed scale0/1
prevents a nonzero callback residual from rounding to zero or infinity; the
exact positive-operand difference fits34 words. Retained risk terms and sums
fit the explicit scalar/basis bounds.

Private primitives or positive-model exclusions:265,382,436,539,554,578,
590,591,592. Zero g is a valid private exponent input but the public discount
path handles quote zero first; admitted nonzero public g remains nonzero under
the selected profile. `risk_terms` handles zero PV before calling the strictly
positive scalar normalizer. Retained-significand carry is independently
exercised by the frozen private corpus, outside this standard LLVM run.
Positive par excludes a zero retained risk sum. Dirty and target operands are
positive in actual callbacks/reports, so the residual opposite-sign operand
branch is not a named financial state. These facts do not make the primitive
functions universally unreachable or extend coverage to the supplemental run.

## Earlier modules and actual exclusions

The454 non-gilt missing regions preserve the earlier
[generic yield/risk review](M8_BOND_YIELD_RISK_VALIDATION.md) classifications.
A direct JSON comparison found identical measurements outside `bonds.rs`
and `error.rs`; their17 and three missing regions respectively move by
exactly12 and22 lines with the new module documentation/exports/root wrappers.
The new root-wrapper regions are covered. The generic ratio helper changed
only module visibility; Wide changed only its Hash derive. No generic
arithmetic was rewritten to obtain this result.

Allocation failures are unexecuted. Defensive capacity and positive-model
financial exclusions are algebraic/source-review results; they are not marked
as test passes. Private exact observations remain separate from publicly
reachable yield/libm observations. The numerical and source/cash implementation
reviews give the actual expression staging and storage census. Coverage does
not establish financial, owner, foreign-native-platform or full-M8 acceptance.
