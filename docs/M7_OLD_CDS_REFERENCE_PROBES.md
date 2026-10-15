# M7 OldCDS public reference observations

This is agent engineering evidence for the
[adopted local OldCDS contract](M7_OLD_CDS_CONTRACT.md). It records actual QuantLib
1.43 public behavior for the OldCDS generation rule and classifies every difference
from the local definition exactly. It is not owner approval, a financial
specification, universal library equivalence or completion of M7.

**No QuantLib C++ source was read, copied, translated or paraphrased.** QuantLib
served only as a black-box behavioral reference through the public Python API of
the attested wheel. The observations are software behavior, not financial truth.
Neither agreement nor a named distinction establishes historical CDS market
practice, a legal trade/effective/protection date or a premium-accrual
convention. The thirty-day equality and start role stay unresolved, as the
[legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md) records.

## Actual collection and attestation

The [generator](../validation/generator/schedule_old_cds_probes.py) collected
**1,199 Schedule constructor observations** with `DateGeneration.OldCDS`, EOM false
and no explicit coupon roles. It retains every returned vector, regularity flag,
metadata value and RuntimeError. The
[fixture](../validation/fixtures/schedule/old-cds-probes.fixture) is
**2,049,813 bytes**. Its observations were collected at `2026-10-15T13:17:33Z`; a fresh
complete `--check`, re-running every worker, then passed on **2026-10-15**.

Each case runs in its own isolated Python `-I` subprocess, with a **10-second**
timeout and fresh wheel/module verification. Inputs are positive spans of at most
1,096 days inside 2025–2029. Positive tenors range over 1–120 months or 1–10
years; the seven nonpositive or Days/Weeks controls occur only in their named
group. Outputs are bounded to 100 dates within 2025–2030. A timeout, unexpected
stderr, worker failure or changed attestation aborts before writing. Errors and
non-strict vectors would be retained, not repaired; all 1,188 returned vectors
were in fact strictly increasing.

The actual platform is Darwin arm64 with Python 3.9.6. Every worker reported
QuantLib 1.43 and matched all three attested module files of the pinned wheel
`quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl`. The source baseline recorded in
the fixture is commit `6b57206e04598f092efee66e3b367efc84771995`; the wheel's exact
source commit has **not** been independently attested. Byte-attested
package/runtime evidence certifies neither source-to-wheel provenance nor a
market convention.

### The reproduced temporary probe

The first 551 rows, `OLD-TENOR-0000` through `OLD-TENOR-0550`, reproduce an earlier
temporary, uncommitted probe run with the same ids, inputs and order. Run with
`--compare-temporary-json`, the generator compared them field by field. Inputs,
status, dates, flags, strict order, returned rule (ordinal 7, recorded here as
`OldCDS`), stored tenor, EOM and module manifest are identical for all 551. The
fixture header records the temporary JSON's digest. The 648 `OLD-EXTRA` rows are
new.

| Scenario group | Ids | Rows |
|---|---|---:|
| Civil stop and first quarter: 6 starts × 9 ends × 1/2/3/4/6/12M | OLD-TENOR | 324 |
| Multi-year strides 6/12/18/24M and 1Y/2Y | OLD-TENOR | 108 |
| Saturday starts × 7 conventions × Unadjusted/Preceding terminal × 1M/3M | OLD-TENOR | 56 |
| Ends Nov 19/20/21 × 3 conventions × 7 terminal conventions, 1M | OLD-TENOR | 63 |
| Leap-year cutoff: starts 2028-02-18/19/20/21 and 03-19/20; 1/3/4/12M | OLD-EXTRA | 68 |
| Year-end crossings 2026→2027, 1/3/12M | OLD-EXTRA | 72 |
| Very short spans in or just across one quarter, 1/2/3/4/6/12M | OLD-EXTRA | 60 |
| Distances 0/1/29/30/31 before each 2026 quarter, 1/2/4/6/12M | OLD-EXTRA | 100 |
| Sunday end after a Saturday stride point, all 49 convention pairs, 1M | OLD-EXTRA | 49 |
| Same Sunday end, 3 conventions × 7 terminal conventions, 3M | OLD-EXTRA | 21 |
| Weekend ends Jun 20, Sep 19, Nov 22, Dec 19 × 3 × 7, 1M | OLD-EXTRA | 84 |
| Sunday starts × 7 conventions × Unadjusted/Preceding terminal × 1M/3M | OLD-EXTRA | 56 |
| Weekend quarter starts Jun 20/Sep 20 × 7 conventions, 1M/3M | OLD-EXTRA | 28 |
| End on a non-quarter 2M/4M stride point, and the next day | OLD-EXTRA | 38 |
| End on the stride lattice but before the recurrence anchor | OLD-EXTRA | 6 |
| Tenor domain 0M, −1M, 1D, 1W, 5D, 2W, 0Y on two spans | OLD-EXTRA | 14 |
| Odd and huge strides 5/7/9/36/120M and 3Y/10Y | OLD-EXTRA | 28 |
| One-interval adjusted collisions, 3 × 4 convention pairs | OLD-EXTRA | 24 |
| **Total** | | **1,199** |

All calendars are NullCalendar or WeekendsOnly. There are **1,188 returned values
and 11 constructor errors**: two −1M rejections and nine collision-control
rejections. These are bounded case observations with unique ids, not exhaustive
coverage of OldCDS inputs.

## Independent local-contract model

The annotation is a separate Python model of the contract, written for this
generator. It does not import the temporary classifier, and no reference output
feeds it. Q(x) is found by scanning calendar days to the next March, June,
September or December twentieth. With A=Q(S), R=Q(E) and d=A−S in days, B is A+3
months when A<R and d<30, otherwise A. Interior points are the twentieths from B
to before E whose month coordinate is congruent to B modulo the stride m (Years
times twelve). The contract's closed-form count must equal the enumerated count.
S<B≤R, B≥E ⇒ B=R and strict raw order are asserted for every row. An interval is
Regular only when both raw endpoints are such lattice twentieths exactly m months
apart.

Adjustment implements the seven FS-BDC conventions of
[REQUIREMENTS §8](REQUIREMENTS.md). WeekendsOnly closes Saturday and Sunday and
NullCalendar closes nothing. The single frozen TARGET control adds 1 January,
Good Friday, Easter Monday, 1 May and 25/26 December. S and interior points use
the ordinary convention and R the termination convention; any adjacent adjusted
collision or reversal is a local `AdjustedOrder`. Before collecting, the generator
checks its computus against the 2026–2028 dated anchors in
[M3_TARGET_CONTRACT](M3_TARGET_CONTRACT.md). It also checks both weekends-only rows
of the FS-BDC example table and the contract's 119,987-boundary maximum. During
development a second, independent computus formula agreed for every year
1583–4999.

Every row receives exactly one class:

| Class | Exact predicate |
|---|---|
| exact | Reference dates and flags equal the local adjusted dates and flags |
| flags-only | Dates equal; differing intervals are recorded as first, interior or last, or sole for a one-interval vector |
| T1-offquarter-exact-end | E = B + k·m for some k ≥ 0 in a non-quarter month, and the reference is the local vector with its final date replaced by adjust(E, termination) |
| T2-adjusted-end-coincidence | adjust(E, termination) equals the last raw stride point P adjusted under the termination convention, and the reference is the local vector without R, ending at adjust(E, termination) |
| S1-unadjusted-first | The ordinary convention moves S, and the reference is the local vector with S restored unadjusted |
| S1+T2 | Both S1 and T2 transformations |
| domain-rejected | The stored tenor is not a positive Months/Years length: local `InvalidOldCdsTenor` |
| local-AdjustedOrder | The local adjusted vector collides or reverses: local `AdjustedOrder` |
| outside-options | EOM or explicit coupon roles, which `build_old_cds` does not accept (frozen rows only) |
| UNCLASSIFIED | Anything else: a reference error for a local value, changed metadata, ambiguity, or an unlisted composition such as S1+T1 |

The contract's T2 wording does not state which convention adjusts the last stride
point. The fixture records both readings, and only the termination reading
decides the class. That reading explains every observed omission. The ordinary
reading alone would predict four omissions that did not occur (see below).

## Frozen roll-probe OldCDS rows

The 98 OldCDS rows of the frozen
[roll-probe fixture](../validation/fixtures/schedule/roll-probes.fixture) were
read only. They classify as follows:

| Class | Rows | Detail |
|---|---:|---|
| exact | 56 | Dates and flags agree |
| flags-only | 29 | First interval 9 (reference Stub, local Regular); last 13 and sole 7 (reference Regular, local Stub) |
| domain-rejected | 4 | 0M (reference rule Zero), −1M (reference error), 1D and 1W (reference values) |
| outside-options | 9 | Four EOM and five explicit-role requests, all reference constructor errors |
| **Total** | **98** | All 85 in-domain rows agree in dates |

The new fixture's header pins these counts, the frozen file's SHA-256 and the
flags-only, domain-rejected and outside-options ids. `--check` re-derives them.

## Fresh observations

| Set | Rows | exact | flags-only | T1 | T2 | S1 | S1+T2 | domain-rejected | local-AdjustedOrder | UNCLASSIFIED |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| OLD-TENOR, reproduced | 551 | 152 | 299 | 41 | 11 | 36 | 12 | 0 | 0 | **0** |
| OLD-EXTRA, new | 648 | 213 | 257 | 19 | 58 | 63 | 12 | 14 | 12 | **0** |
| **All fresh** | **1,199** | **365** | **556** | **60** | **69** | **99** | **24** | **14** | **12** | **0** |

The OLD-TENOR counts were re-derived independently and equal the contract's
table: 152 exact, 299 flags-only and 100 date differences (T1 41, T2 11, S1 36,
S1+T2 12). No fresh or frozen row is UNCLASSIFIED.

| OLD-EXTRA group | exact | flags-only | T1 | T2 | S1 | S1+T2 | Other |
|---|---:|---:|---:|---:|---:|---:|---|
| leap-year-cutoff | 32 | 36 | | | | | |
| year-end-crossing | 32 | 40 | | | | | |
| very-short-span | 21 | 39 | | | | | |
| quarter-distance-strides | 20 | 80 | | | | | |
| holiday-terminal-49-pairs | | 14 | | 35 | | | |
| holiday-terminal-quarterly | 6 | | | 15 | | | |
| holiday-terminal-ends | 78 | | | 6 | | | |
| holiday-start-sunday | 6 | | | 2 | 36 | 12 | |
| holiday-start-on-quarter | | 4 | | | 24 | | |
| offquarter-stride-end | 6 | 13 | 19 | | | | |
| lattice-end-before-anchor | 1 | 5 | | | | | |
| tenor-domain | | | | | | | 14 domain-rejected |
| odd-and-huge-stride | 2 | 26 | | | | | |
| adjusted-collision | 9 | | | | 3 | | 12 local-AdjustedOrder |

Every flags-only difference has one of two directions, and no interior interval
differs. Reference Regular with local Stub occurs only at a last or sole
interval: 268 last and 23 sole OLD-TENOR rows, 170 last and 72 sole OLD-EXTRA
rows. Reference Stub with local Regular occurs only at a first or sole interval:
7 first and 1 sole OLD-TENOR rows, 6 first and 2 sole OLD-EXTRA rows. Seven more
OLD-EXTRA rows differ at both ends, one in each direction. The reference thus
marks a projected terminal Regular even when it is not one stride after the
previous point. It also marks Stub a first interval whose start lies exactly
one stride before B. Local flags follow the contract's lattice definition;
adjustment never changes them.

## Concrete examples

Dates are 2026 unless a year is shown; flags are per interval (R Regular, S Stub).

| Class | Id and inputs | Reference | Local contract |
|---|---|---|---|
| exact | OLD-TENOR-0000: Feb06→Jun19, 1M, Null | Feb06, Mar20, Apr20, May20, Jun20; S R R R | Same |
| exact | OLD-EXTRA-0021: 2028-02-19→2028-12-31, 3M; d=30 in a leap year | 2028-02-19, Mar20, Jun20, Sep20, Dec20, 2029-03-20 | Same; A retained |
| flags-only | OLD-TENOR-0001: Feb06→Jun19, 2M | Feb06, Mar20, May20, Jun20; S R **R** | Same dates; S R **S**, as May20→Jun20 is one month |
| flags-only | OLD-EXTRA-0068: Nov20→2027-01-19, 1M | Nov20, Dec20, 2027-03-20; **S R** | Same dates; **R S** |
| T1 | OLD-TENOR-0024: Feb06→Aug20, 1M | Feb06, Mar20, …, Jul20, **Aug20** | Feb06, Mar20, …, Jul20, **Sep20** = Q(E) |
| T1 | OLD-EXTRA-0538: Feb06→May20, 2M | Feb06, Mar20, **May20**; S R | Feb06, Mar20, **Jun20**; S S |
| T2 | OLD-TENOR-0434: Feb07→Nov21, 1M, Weekends, Unadjusted/Preceding | Feb07, Mar20, …, Oct20, **Nov20** | Feb07, Mar20, …, Oct20, Nov20, **Dec18** |
| T2 | OLD-EXTRA-0350: Feb06→Jun21 (Sun), 3M, Weekends, Unadjusted/Following | Feb06, Mar20, **Jun22** | Feb06, Mar20, **Jun20, Sep21** |
| S1 | OLD-TENOR-0436: Feb07 (Sat)→Nov21, 1M, Weekends, Following/Unadjusted | **Feb07**, Mar20, Apr20, …, Dec20 | **Feb09**, Mar20, Apr20, …, Dec20 |
| S1 | OLD-EXTRA-0458: Feb08 (Sun)→Nov21, 1M, Weekends, Following/Unadjusted | **Feb08**, Mar20, Apr20, …, Dec20 | **Feb09**, Mar20, Apr20, …, Dec20 |
| S1+T2 | OLD-TENOR-0438: Feb07→Nov21, 1M, Weekends, Following/Preceding | **Feb07**, …, Oct20, **Nov20** | **Feb09**, …, Oct20, Nov20, **Dec18** |
| S1+T2 | OLD-EXTRA-0460: Feb08→Nov21, 1M, Weekends, Following/Preceding | **Feb08**, …, Oct20, **Nov20** | **Feb09**, …, Oct20, Nov20, **Dec18** |
| domain-rejected | OLD-EXTRA-0582: Feb06→Mar31, 0M | Feb06, Mar31; rule Zero, tenor 0Y; R | `InvalidOldCdsTenor` |
| domain-rejected | OLD-EXTRA-0584: Feb06→Mar31, 1D | Feb06, Mar20, then daily to Mar31: 13 dates | `InvalidOldCdsTenor` |
| local-AdjustedOrder | OLD-EXTRA-0626: Jun19 (Fri)→Jun20 (Sat), 3M, Unadjusted/Preceding | Error "degenerate single date (June 19th, 2026) schedule …" | Raw Jun19, Jun20 adjust to Jun19, Jun19 |
| local-AdjustedOrder | OLD-EXTRA-0641: Sep19 (Sat)→Sep20 (Sun), 3M, Following/Following | Sep19, Sep21; S | Raw Sep19, Sep20 adjust to Sep21, Sep21 |
| outside-options | Frozen ROLL-EOM-OLDCDS-Null-Unadjusted: EOM true | Error "endOfMonth convention incompatible with OldCDS date generation rule" | Not expressible |
| outside-options | Frozen ROLL-COUPON-OLDCDS-FIRST-WED: first 2026-03-18 | Error "first date incompatible with OldCDS date generation rule" | Not expressible |

## What the new probes establish

- **Leap-year cutoff.** In 2028, February 19 is 30 days before March 20 and
  retains it; February 20 (29 days) omits it. In 2026, February 19 is 29 days
  away and omits it (OLD-TENOR-0110). All 68 leap rows agree in dates, so the
  observed cutoff counts calendar days across February 29.
- **Year-end and short spans.** All 72 crossings and 60 short spans agree in
  dates. That includes A=R terminal preservation, distance zero with B=R
  (Jun20→Jun21 gives Jun20, Sep20) and spans entirely inside one quarter window.
- **Distances across strides.** All 100 distance controls agree in dates for
  1/2/4/6/12M. The observed three-month shift does not depend on the stride.
- **T1 scope.** Each of the 60 rows whose E is an off-quarter stride point
  B+k·m with k≥0 is T1 (41 reproduced, 19 new); the 19 next-day ends agree in
  dates. The six ends on the lattice but before B are not T1: the reference
  returns R exactly as the local rule does.
- **T2 convention.** All 93 rows whose last stride point and E coincide after
  termination-convention adjustment omit R (23 reproduced, 70 new). Thirty new
  rows coincide only under that reading. Four rows coincide only under the
  ordinary reading: Following, ModifiedFollowing or HalfMonthModifiedFollowing
  with a Nearest terminal and E = Sunday June 21. All four retain R. The local
  rule never omits R, so these rows sharpen the description, not the contract.
- **S1 holds for every probed weekend start.** The ordinary convention would
  move a Saturday or Sunday start in 128 requests. The reference returned a
  vector for 126 of them, always starting at the unadjusted S: 48 reproduced,
  75 new and 3 collision controls. The other two are collision controls that
  it rejected. The 126 include Sunday starts and weekend quarter starts (S=A).
- **Collisions.** For the 12 local `AdjustedOrder` rows, the reference raised its
  own "degenerate single date" error in 9. The other 3 return two increasing
  dates only because the reference keeps the Saturday start unadjusted.
- **Tenor domain.** 0M and 0Y return [S, E] with rule Zero, tenor 0 Years and a
  Regular flag. −1M fails with "non positive tenor (-1M) not allowed". Days and
  Weeks are accepted under OldCDS: 1D on Feb06→Mar31 returns Feb06, Mar20 and
  then every day to Mar31, ending at E rather than a projected quarter. 1W, 5D
  and 2W project to Jun20; on Mar20→Apr30 all four return Mar20, Jun20. The
  local operation rejects all 14 with `InvalidOldCdsTenor` and substitutes no
  unit or rule.
- **Odd and huge strides.** 5/7/9/36/120M and 3Y/10Y agree in dates (26
  flags-only, 2 exact). For 36M, 3Y, 120M and 10Y on Feb06→2028-11-21 both
  sides return Feb06, Mar20, 2028-12-20. The reference flags the final interval
  Regular; the local contract flags it Stub.

These are deliberate local choices, not tolerances. No reference row is
rewritten, and no classification establishes complete reference parity.

## Reproduction and pins

```sh
/private/tmp/sabiquant-validation-venv/bin/python -I \
  validation/generator/schedule_old_cds_probes.py \
  --quantlib-wheel /private/tmp/sabiquant-validation-wheels/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl --check
```

`--check` mirrors the committed probe generators. It re-runs every attested
worker and rebuilds the whole file in memory, copying `generated-at`, `platform`
and `python-version` from the committed header, then requires byte equality. It
needs the wheel and its installed runtime; there is no runtime-free mode. Adding
`--compare-temporary-json /private/tmp/sabiquant-old-cds-tenor-probes.json`
repeats the 551-row identity check while that file exists. Omit `--check` only
to write new evidence for review. The validation environment's interpreter is
the system Python 3.9.6 and supplies only the attested QuantLib package. The
generator uses the standard library and imports QuantLib only inside workers.

| Artifact | SHA-256 |
|---|---|
| New generator | `3399ba13bb1b75ef600706da9036b328b8a2b1dbbbb8a06197d59db7f12ffb28` |
| New fixture | `ab55159feb02b4fbe4b01cc70a9a9de3366173b65b14ef5ad376b6c03d1461f9` |
| Fixture body (`body-sha256`) | `83c143627d81190f65173d677330b6ef27c269a2eaf045a5544dccc41eec925a` |
| Adopted contract | `24aebdcb67cf5127df56705e85093c905767da787552325d83cb60a4e2b3b996` |
| Attestation helper `schedule_probes.py` | `2c48705c96a6311922e8483cc47ae383f032555fd56f4c3f53567ca605725d6d` |
| Frozen roll-probe fixture, read only | `88e2bacdc1ab999c1979f19c5d2f4fa485f3492cceab040cca6cc927e9edf35a` |
| Wheel | `85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c` |
| Three-file module manifest | `e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d` |
| Temporary probe JSON, uncommitted | `e83ca664c0d537d511cc9958ca0e7a06534579fb932db5b99258e2846999e35d` |
| Temporary probe script, uncommitted | `3a1eb4c6c52fac2a8360ed6c36e7c47b5c853e1a59b6d4b23ebc92dcc45dbd37` |

The fixture's FNV-1a 64-bit body checksum (`body-fnv64`) is `ef0b12f0b1369898`.
The frozen roll-probe fixture and generator and the attestation helper were not
edited. The contract's Regular wording was later corrected without changing
behavior (finding C1 in the [implementation review](M7_OLD_CDS_IMPLEMENTATION_REVIEW.md)).
This fixture was then regenerated for the new contract hash; its body is
byte-identical. `tests/schedule_old_cds_reference.rs` reclassifies every row
against the actual implementation. Local links refer to current workspace
files; no remote publication is implied.
