# Independent civil-roll ICMA oracle evidence

The generator and Rust consumer were authored separately from production by
`/root/roll_icma_oracle`. The production implementer reviewed their independence
and error indexing without editing them; the root subsequently reviewed the
source, formatted the consumer and ran the final checks after agent quota
interruption. This report records the final saved artifacts, not unobserved
completion of an interrupted agent command.

## Definition and independence

The [adopted contract](M7_ROLL_ICMA_CONTRACT.md) and unchanged historical
[design](M7_ROLL_ICMA_DESIGN.md) are SHA-pinned in the fixture header. Python
datetime/calendar enumerates actual Wednesdays in each civil month. For the
reference hull it enumerates every month in an envelope around the raw span,
selects congruent months and chooses the greatest/least bracketing roll points.
It does not reuse Rust's negative/positive extension loops, Rust expected
outputs, QuantLib outputs or a clamped Wednesday day.

The Rust consumer independently computes Gregorian ordinals, counts preceding
Wednesdays, checks quarter membership and month congruence, compares every
reference period exactly, and verifies endpoint bracketing/minimality. Reduced
i128 integer fractions recompute the Python exact numerator/denominator from
ordinal overlaps. The existing ICMA four-ULP profile applies only to the final
binary64 year fraction; dates, statuses, provenance and exact rationals use E0.

## Corpus and measured checks

There are **42,647 context records**, **42,642 distinct input tuples** and five
repeated controls. Tuple identity includes requested start/end, stored tenor
length/unit, rule and explicit frequency. The main cycle contains 38,400 rows:
all 4,800 months of 2000..2399 × four rules × two endpoint profiles. Frequencies
and before/on/after positions rotate; this is not their full Cartesian product.
Additional century/civil controls, extreme bounds and frequency-domain rows
test the remaining declared cases.

| Outcome | Records |
|---|---:|
| Successful minimal contexts | 42,179 |
| Required civil extension Date failures | 260 |
| Explicit frequency/tenor mismatch | 188 |
| Once rejected | 20 |

Successful rows contain **87,558 determination periods**, with one through six
periods per context, and **126,537 exact Fraction queries**: one forward and
one reversed raw-range query per context, plus a rotating zero/full-hull/
partial/interior query. They include **10,548 zero-length queries** and
**42,179 reversed queries**. The measured maximum in the final native debug
run is **1 ULP**, within the unchanged four-ULP threshold. This finite result
does not prove universal one-ULP accuracy.

Every successful row checks exact cap, cap+1 and, when valid, cap−1; every row
checks invalid cap0 before its value/error domain. There are **14,155 Annual
alias controls**, requiring unequal stored 12M/1Y source definitions but equal
reference contexts. All six matching monthly-family frequencies appear. The
consumer checks unchanged source values and all roll provenance inspectors.

The 260 Date errors retain exact q(0), signed month offset, generation-position
index and `ICMA reference extension` phase. Required lower failures precede
upper failures; enormous stored tenors fail frequency matching before date
arithmetic. This fixture uses Null/Unadjusted only. Calendar/adjustment guards,
every-insufficient-cap examples, coupon composition and actual whole-civil
resource hulls belong to the separate public hand suite.

## Reproduction and artifact pins

The root observed terminal exit zero for generator `--check` and the final
debug hand/oracle command. Release, full-suite and coverage results are owned
by the [aggregate report](M7_ROLL_ICMA_VALIDATION.md), avoiding inference from
an interrupted agent's pending processes. No library dependency was added.

```sh
python3 validation/generator/schedule_roll_icma_oracles.py --check
cargo test --offline --test schedule_roll_icma_oracles -- --nocapture
cargo test --offline --release --test schedule_roll_icma_oracles -- --nocapture
```

| Artifact | SHA-256 |
|---|---|
| Generator | `7ea36af365d4928a532a670a109ceee3196012e9a629d12f9827a6b7e1ed303d` |
| Fixture, 13,096,330 bytes | `376baa750cfdb06eca47e6ef77f544ae0f5698ee663f48536ca1ae1e2cea72c8` |
| Fixture body | `c7eaa35feb9e82d6491a4dc7a076541e153d5371e19664c8e24ab3552f209480` |
| Rust consumer | `ba4de96ac80666146bf47efe823b747cd4189fb9acb4f284b5085692ca58f944` |
| Adopted contract | `cf7fdc9603d8428b8924fa8afd88dc9d0a274415f2fac84885a08228f27529e4` |
| Historical design | `d085788605f43e3a67569fad7f554d930efe1f3d1b8f9d25044a6d7c39f0b26c` |

This evidence validates the explicit analytical reference inputs. Standard CDS
products, original financial-source acceptance and native-platform/release
gates remain separate work.
