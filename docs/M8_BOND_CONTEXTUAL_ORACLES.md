# Independent contextual bond numerical corpus

This additive corpus strengthens `FR-BOND-04/06/07/08/09/10` under the adopted
[bond core](M8_BOND_CORE_CONTRACT.md), [explicit ICMA](M3_ICMA_CONTRACT.md),
[contextual day-count](M3_DAYCOUNT_EXTENSIONS.md), [US](M3_US_AFB_CONTRACT.md),
[TARGET](M3_TARGET_CONTRACT.md), [date-rate](M4_DATE_CONTRACT.md), and
[cash-flow](M7_CASHFLOW_CONTRACT.md) contracts. The numerical definitions were
derived before inspecting Rust bond/quote production; no such production or
QuantLib source/output was read. Existing generators, fixtures and contracts
were preserved. This is local engineering evidence, not financial or named
market certification, a proof of upstream ideal-price enclosures, or native
multi-platform evidence.

The [generator](../validation/generator/bond_contextual_oracles.py) and
[fixture](../validation/fixtures/bonds/contextual-oracles.fixture) use only
Python standard-library integers, civil dates, `Fraction` and `Decimal`.
Reproduce all expectations with:

```sh
python3 -I validation/generator/bond_contextual_oracles.py --check
```

Generation and complete `--check` both succeeded on 2026-10-17. The fixture has
**11,136 distinct input rows**, **52 columns**, and **15,639,401 bytes**.
Every complete oracle evaluation agrees at **160 and 240 decimal digits**.
This includes 10,432 successful numerical valuations; the remaining 704
evaluations agree on exact preparation failures and stopped query ledgers.
The [separate Rust consumer](../tests/bonds_contextual_oracles.rs) links these
independent expectations to the public API; its executed test evidence belongs
to the main validation report.

## Coverage and exact census

All **32** configurations occur **348 times each**: Simple, Continuous, and
Compounded, SimpleThenCompounded and CompoundedThenSimple at explicit event
counts **1, 2, 3, 4, 6, 12, 13, 26, 52 and 365**. Every supplied yield and
coupon rate is nonzero. Yields have 5,568 positive and 5,568 negative inputs;
coupon rates have 3,836 positive and 7,300 negative inputs. Coupon rates are
`0.05`, `-0.07` and `-1.0`; yields are `0.04` and `-0.02`, interpreted as their
exact supplied binary64 bits. Faces are 100 and 1,000, and explicit non-par
redemption is retained separately.

| Successful profile family | Rows |
|---|---:|
| ICMA regular, short first/final and long first/final; explicit references | 3,200 |
| 30E/360 ISDA final-February and intermediate-February roles | 1,280 |
| 30/360 US EOM qualification off/on | 1,280 |
| Business/252 custom Friday/Saturday and Sunday/Monday weekend masks, holiday and business overrides | 1,280 |
| Verified guarded TARGET, including 2027–2028 closing rules | 640 |
| ICMA coupon / ISDA yield, delayed payment | 704 |
| US coupon / custom Business/252 yield, early payment | 704 |
| ISDA coupon / ICMA yield, reopening after first accrual start | 704 |
| Custom Business/252 coupon / US yield, delayed payment | 640 |
| **Successful total** | **10,432** |

Each successful family visits both payment inclusion choices, ex-cutoff
neighbors, raw boundaries, payment boundaries, the day before final payment
and post-payment settlement. Across families, the first coupon's explicit
boundary alternates `After` and `OnOrAfter`; the second uses `OnOrAfter`.
This is not every boundary-policy combination for each context. Initial issue
is explicit. The extra one-day horizon supplies non-identity short-time
hybrid cases under ICMA; a day count whose smallest positive fraction exceeds
`1/365` cannot supply a positive SimpleThenCompounded/Daily simple horizon.
The corpus does not invent one to inflate branch coverage.

| Preparation failures | Rows |
|---|---:|
| Work cap one below complete required work | 512 |
| ICMA equal-date anchor outside explicit coverage | 32 |
| ICMA required horizon outside explicit coverage | 32 |
| TARGET projected equal-date anchor | 32 |
| TARGET verified endpoints crossing the historical source gap | 32 |
| TARGET pre-inception equal-date anchor | 32 |
| TARGET unsupported post-maturity anchor, despite ceased flows | 32 |

Every context family has work-cap controls in all 32 configurations. The
largest successful shared work count is **920**. These controls supplement
the existing zero-cap, u64-limit and constructor/error suites rather than
claiming every invalid definition is covered here.

The exact serialized ledger census is:

| Query role/status | Count |
|---|---:|
| Full constructor coupon, successful | 22,272 |
| Discount anchor, successful / coverage failure | 11,008 / 128 |
| Partial accrual, successful | 6,624 |
| Coupon horizon, successful / coverage failure | 9,952 / 64 |
| Redemption horizon, successful / work failure | 8,896 / 512 |
| Successful zero-ULP / four-ULP YF profiles | 41,248 / 17,504 |
| Serialized supplied-YF to exact rounded-product entries | 114,400 |

The last number counts entries appearing in the fixture, including deliberately
repeated observations across configuration/settlement scenarios. It is not a
claim of 114,400 distinct products or financial contracts.

## Numerical construction and interpretation

Actual-day counts and 30/360 adjustments use exact integer civil arithmetic.
ICMA explicitly sums each overlap `days / (F * reference_days)` as a reduced
`Fraction`; periods and frequency come from the input and are never inferred.
Business/252 counts the half-open interval under its separate accrual calendar,
including explicit business-day openings and holiday closings. Strict TARGET
checks both endpoints and the covered span before counting. Coupon and yield
contexts can differ; schedule payment adjustments never substitute for either.

For a non-ICMA fraction, one rounded exact rational supplies the sole admissible
binary64 YF. For positive ICMA fractions, every binary64 value within the adopted
**four-ULP** profile around the rounded rational is enumerated: at most nine
values. Equal ICMA endpoints give exact positive zero after coverage validation.
The oracle does not require all raw-rational ICMA targets to be bit-identical
to compensated production sums.

For each full or partial coupon query, every admissible YF maps to
`RN(exact_binary_face * exact_binary_rate * supplied_binary_YF)`. Here `RN`
is nearest with ties to even. A separate Decimal product calculation at both
precisions must recover the actual exact rational product and the same rounded
bits; these bounded corpus dyadics terminate within 160 digits. The consumer
selects the map entry for its actual public YF, allowing a **zero-ULP product
assertion** even when ICMA itself legitimately differs from the raw target.

Full coupon amount intervals, partial amount intervals and their detached
differences propagate with exact `Fraction` arithmetic. Each required coupon
contribution is rounded once, then its reduction is rounded once. Unknown
correlation between a full and partial fraction is conservatively discarded;
the enclosure does not assume they attain their extrema together.

For each admissible horizon, Simple factors follow the exact operational path
`RN(1 + r*t)`, then `RN(1/C)`. For an elementary branch, the independent ideal
discount is evaluated with Decimal and expanded by the existing measured M4
criterion:

```text
Continuous: log(C) = r*t
Periodic:   log(C) = m*t*ln(1+r/m)
D = exp(-log(C))
b = 8 * 2^-53 * (1+abs(log(C))) * abs(D) + 2^-1074
```

The factor interval is converted **outward** to adjacent binary64 bounds.
Signed amount/factor endpoint products then use exact rational multiplication
and one nearest-even rounding. Per-flow bounds reduce once by exact summation;
clean bounds subtract accrued bounds and round once. This propagates gross
signed-flow exposure rather than applying a relative error to a cancelled net
price. There is no global epsilon or final-price ULP multiplier. Every possible
ICMA horizon is evaluated separately, so a hybrid interval crossing its rounded
switch is not handled by an unsupported global monotonicity assumption.

The M4 elementary criterion remains a measured acceptance condition, **not a
worst-case theorem for Rust libm**. Matching 160/240-digit output bits is stable
high-precision evidence on this finite grid, not interval certification of
every logarithm/exponential input. Typed factor-required range/guard behavior
is unchanged; this corpus's successful quotes remain safely in their normal
factor domains.

## Raw-rational ideal versus staged output

`raw_ideal_*_bits` are rounded anchors for a **different model**: coupon and
partial accrued amounts use exact raw day-count rationals without intermediate
binary64 YF/product rounding, and discount formulas use exact raw horizons.
The declared rounded binary64 hybrid switch is retained. Equal raw horizons
combine exact signed amounts before evaluating their common ideal factor.
Raw accrued interest has an independent exact rational check; valuations whose
active branches are all Simple additionally check their complete raw dirty and
clean anchors against exact rational formulas.

The six `*_lo_bits` / `*_hi_bits` fields bound the staged production model;
they need not contain the raw ideal. Indeed, the raw ideal RN anchor lies
outside those staged intervals in **1,092 dirty**, **2,328 accrued**, and
**1,041 clean** rows. Treating these two targets as identical would be wrong.

Each extra `raw_ideal_X_error_bits` field is the upper-directed binary64 value
of this exact rational distance, for its serialized anchor `J=RN(I)`:

```text
E_X = upward_binary64(max(abs(staged_lower - J),
                         abs(staged_upper - J)))
```

Thus a staged observation satisfying its enclosure also satisfies
`abs(observed - J) <= E_X`. This comparison uses the **rounded anchor J**.
Against the exact raw ideal `I`, the additional `abs(I-J)` oracle-rounding
term must also be included. The fixture does not silently absorb that term or
claim its numerical anchor equals an exact transcendental value. Near
cancellation, only the propagated absolute comparison is justified.

## Complete input and output schema

The schema-1 fixture uses tab-separated columns, `0x` plus 16 lowercase hex
digits for IEEE754 bits, ISO civil dates, and `-` for explicitly unused cells.

- `id`, `profile`, `issue`; `raw_dates` and `pay_dates` contain all three
  contractual/adjusted dates; `redemption_date` and `redemption_bits` are
  explicit and never inferred from par or the final coupon.
- `ex_terms` has two semicolon-separated `After:date` or `OnOrAfter:date`
  values. `face_bits`, `coupon_rate_bits`, `yield_bits` preserve exact inputs.
- `coupon_context`, `yield_context` use `A365F`, `ISDA:termination`, `US:0|1`,
  `ICMA:F:comma_reference_boundaries`, `B252:T` for strict TARGET, or
  `B252:C:weekend_mask:holiday_dates:business_dates`. Weekend mask bit 0 is
  Monday. Every adjacent ICMA boundary pair is one supplied determination
  period. `F` is its explicit event count, independent of discount compounding.
- `settlement`, `inclusion`, `comp`, `work_cap` define the explicit trial.
  `comp` is `Simple`, `Continuous`, or `method:F` with the 32 configurations
  listed above.
- `sched_tenor`, `sched_rule`, `sched_first`, `sched_penultimate`, `sched_eom`,
  `sched_bdc`, `sched_term_bdc`, `sched_holidays`, `sched_business`, `sched_max`
  fully specify construction. The base schedule calendar is explicitly Null;
  these profiles use `build_explicit`, tenor `6M`, Forward, identical supplied
  first/penultimate at the middle raw date, Ignore EOM and cap 3. Explicit
  holiday overrides realize the short early/delayed payment adjustments.
  Accrual-calendar overrides remain separate.
- `expected` is `ok`, `work`, `icma` or `calendar`. The seven `error_*` fields
  retain index (`none` for anchor), phase, nested stable kind, rejected date
  when applicable, and exact maximum/used/required work. Coverage errors use
  `-` for inapplicable work-error fields. ICMA error coverage endpoints are
  retained in the explicit context, and its query identifies rejected dates.
- `entitlements` encodes `B` buyer, `X` detached and `C` ceased;
  `redemption_entitled` is `0|1`. `work` is cumulative charged query work. A
  work-failed query is not charged; a coverage-failed query has been charged.
- `full_queries` and `plan_queries` are semicolon-separated ledgers with
  `role~start~end~raw_num/den~rn_bits~ulp~charged_cost~parts~products~status`.
  Roles are `C0/C1`, `A`, `P0/P1`, `H0/H1`, `R2`. Full constructor queries
  have zero shared-plan charge. Parts are `n/d+n/d`; products are comma-separated
  `yf_bits=amount_bits` pairs, or `-`. Status is `ok`, `work` or `coverage`;
  failed-query rational/bit/profile/parts/product fields are `-`, and the ledger
  stops at that first failure. Anchor, partial and horizon order follows the
  complete bond core contract.
- The final twelve fields are the six staged accrued/dirty/clean endpoints,
  three raw ideal anchors in dirty/accrued/clean order, and three matching
  raw-anchor absolute error bounds. They are all `-` for preparation failures.

All 52 columns and nested formats are consumed explicitly. A preliminary
consumer run exposed an oracle error at an equal-date final-February ISDA
anchor: applying endpoint adjustments produced `-1/360`. The already adopted
[zero-before-adjustment rule](M3_DAYCOUNT_EXTENSIONS.md) requires positive zero.
The independent model was corrected **before freezing this new fixture**;
ICMA/TARGET guards were preserved. Twelve direct equal-date and guarded
coverage controls now execute before generation. This was an oracle defect,
not a production defect or a rewrite of an earlier frozen expectation.

## Reproducibility pins and limits

| Artifact | SHA-256 |
|---|---|
| New generator | `3f2398b98f735c9cafde9321040cfb28009fc67d9b1857223be032e9095ba588` |
| New fixture | `6d1e46384edbc2e0f50e5e86cf6f1a46280451befc396c95c15e8fb9a22f3c21` |
| Body including column row | `7c2c7f613fbbf5f081d5c5325a9d49d942ecd4d9a60ec91262586e251342adb6` |

Generator/body FNV64 are respectively `fe1c4f7a8217402d` and
`166be7e404bf1b4f`. The fixture also pins every listed contract and the exact
current `FR-BOND-*` requirement rows with SHA-256 and FNV64. Sequential IDs,
distinct input tuples, exact profile/configuration/status counts, stopped query
ledgers, YF profiles and product-map census are reproducible. Full `--check`
logs are at `/private/tmp/sabiquant-bond-contextual-check.log` in this local run;
the committed generator/fixture are the portable evidence.

The corpus covers two-coupon fixed-rate instruments, explicit semiannual ICMA
references, two custom masks and strict TARGET. It does not claim all schedules,
all possible calendar definitions, every ICMA frequency/reference vector,
constructor failure or elementary range boundary. Earlier focused suites
retain their separate coverage. It supplies numerical context composition
evidence; yield inversion, duration/convexity, settlement adapters, named market
models and financial/platform/release gates remain separate work.
