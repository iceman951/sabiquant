# Independent OldCDS civil and ICMA oracles

Completed 2026-10-15, Asia/Bangkok. Two Python generators and their schema-1
fixtures were written from the frozen [OldCDS contract](M7_OLD_CDS_CONTRACT.md),
the [ICMA composition design](M7_OLD_CDS_ICMA_DESIGN.md) and the
[generated ICMA contract](M7_ICMA_REFERENCE_CONTRACT.md). No file under `src/`
was read, and no SabiQuant or Rust output was used. QuantLib was not run and
its source was not read. This report covers oracle data only; when it was
written no Rust consumer existed. The consumers `schedule_old_cds_oracles.rs`
and `schedule_old_cds_icma.rs` now compare every row; their results are in
[M7_OLD_CDS_VALIDATION](M7_OLD_CDS_VALIDATION.md). Calendars, adjustment, TARGET coverage and coupon composition are
outside its scope.

## Independent construction

The [raw generator](../validation/generator/schedule_old_cds_oracles.py)
derives every expectation by enumeration over Python `datetime.date`:

- Q(x) scans civil days forward from x until it reaches a March, June,
  September or December twentieth. A three-month anchor shift is the next
  scanned quarter twentieth after A.
- Interior points come from every day-20 date in [B, E). A date is kept
  when its month distance from B is a multiple of the stride (Python `%`).
  The count is `len()`.
- An interval is Regular only when both endpoints are lattice members and
  their months differ by exactly m.

The contract's closed-form step-6 count, its closed-form quarter selection
and the literal `B + k·m` recurrence are evaluated only as assertions. The
generator also asserts the contract's listed consequences: S < B ≤ R, B is
always retained, ordering is strict, distance 0 retains S once, distances
30/31 retain A, A = R keeps the terminal, and the omitted quarter is exactly
Some(A) for 0 < d < 30 with A < R.

The [ICMA generator](../validation/generator/schedule_old_cds_icma_oracles.py)
is self-contained and does not import the raw generator. It reimplements the
raw rule differently:

- Q(x) is the least abstract quarter tuple of years x and x+1.
- B is A plus three month coordinates.
- Interior points come from a walk over every civil day in (S, E).

The minimal hull comes from enumerating every month coordinate one stride
beyond raw S..R. It keeps the coordinates congruent to B, then selects the
greatest day-20 tuple ≤ S and the least ≥ R. Years 0 and 10000 are allowed
as abstract tuples. Each overlap contributes `days / (events × period days)`
as an exact `Fraction`.

Cap and Date outcomes follow the order documented in the design:

1. Lower dates come first. Each is checked against `k + reserve ≤ cap`,
   where reserve is 1 iff B < R.
2. Upper dates follow. Each is checked against `running periods + 1 ≤ cap`.
3. A failing check reports `required = cap + 1`.
4. The generation position counts B as 0, then lower dates, then upper dates.

## Fixture schemas

[Raw fixture](../validation/fixtures/schedule/old-cds-oracle.fixture)
columns (one tab-separated line in the file):

```text
id profile start end length unit limit status initial_quarter recurrence_anchor
resolved_end omitted_quarter count dates regularity error_role error_input required
```

- `limit` is the `max_boundaries` argument.
- `status` is `value` or a stable kind. The kinds are
  `schedule.invalid_boundary_limit`, `schedule.invalid_old_cds_tenor`,
  `schedule.old_cds_date` and `schedule.boundary_limit_exceeded`.
- The provenance columns, `count`, `dates` and `regularity` are filled only
  on value rows. An empty `omitted_quarter` on a value row means None.
- `dates` is the `;`-joined raw S..R vector. `regularity` is `R`/`S` per
  interval, `;`-joined.
- When count exceeds 1,024 (67 full-civil-range rows), `dates` is
  `checksum:<n>` and `regularity` is `rle:<flag><run>;...`.
  `n = Σ_{i=0}^{count−1} (i+1)·ordinal(date_i) mod (2^61−1)`, with
  `ordinal(0001-01-01) = 1` (Python `toordinal`; in Rust, days since
  0001-01-01 plus one). The 119,987-boundary row stores
  `checksum:17526298095877429` and `rle:S1;R119985`.
- `error_role`/`error_input` appear only for `old_cds_date`, always as
  `terminal quarter` and E.
- `required` appears only for `boundary_limit_exceeded`, and equals the
  exact final count.

[ICMA fixture](../validation/fixtures/schedule/old-cds-icma-oracle.fixture)
columns:

```text
id profile start end length unit frequency events limit status initial_quarter
recurrence_anchor resolved_end omitted_quarter raw_dates references periods lower_steps
raw_fractions raw_floats span_fraction span_float required error_index error_anchor
error_operation error_count
```

- Every row's raw schedule builds with cap 3,652,059. Raw provenance and
  `raw_dates` appear on every row.
- `limit` is the `max_periods` argument.
- Statuses are `value`, `schedule.invalid_icma_period_limit`,
  `schedule.invalid_icma_frequency`, `schedule.icma_frequency_mismatch`,
  `schedule.icma_period_limit_exceeded` and `schedule.date`.
- Fractions are reduced `num/den` values. `raw_fractions` has one per raw
  interval; `span_fraction` covers S..R. The float columns hold the correctly
  rounded binary64 shortest `repr` of the same fractions.
- Date rows give the generation index, anchor B, `add months` or
  `add years`, and a signed count in the stored unit.

## Coverage and results

| Raw profile | Rows |
|---|---:|
| Every start 2000-01-01..2399-12-31 once; ten relative ends × thirteen tenors rotating (mod 10/13); cap 3,652,059 | 146,097 |
| Distances 0/1/2/28/29/30/31/60 before every quarter of 2000, 2001, 2024, 2100 and 2399; four ends; seven strides | 4,340 |
| Terminal E ±1 day at off-quarter 1/2/4/5M stride points, off-stride quarters, A = R and B = R spans | 10,728 |
| Tenor domain: four units × six lengths × three spans | 72 |
| Civil extremes: early, late, terminal overflow, huge tenors, full civil range | 516 |
| Caps count−1/count/count+1/2 on 323 bases (223 exceeded, 646 value) | 869 |
| Invalid caps and error precedence | 22 |
| **Total** (162,385 distinct inputs including cap) | **162,644** |

Raw outcomes:

| Status | Rows |
|---|---:|
| value | 162,301 |
| schedule.boundary_limit_exceeded | 223 |
| schedule.invalid_old_cds_tenor | 56 |
| schedule.old_cds_date | 48 |
| schedule.invalid_boundary_limit | 16 |

Successful rows contain 2,140,006 Regular and 206,408 Stub intervals. They
include these cases:

- 32,462 rows have Some(A) omitted. 80,759 have A = R, 2,803 have distance
  0, and 46,277 have A < R with d ≥ 30 (107,290 rows have d ≥ 30 at all).
- 3,051 rows retain an interior A at exactly 30 or 31 days.
- 11,266 rows skip a stride point B + k·m (k ≥ 0) in [E, R); counting any
  lattice point (k ∈ ℤ) gives 17,001. 39,197 rows have R off the lattice.
- 1,569 rows have E exactly on a non-quarter stride point.
- 34,684 rows use stored Years.

**Closed-form cross-check:** all **162,524** value and limit-exceeded rows
assert that `len(enumerated dates)` equals the contract's step-6 formula,
with **zero disagreements**. The literal recurrence and the closed-form
quarter selection also agree on every row. The civil maximum is **119,987**
boundaries, attained for example by 0001-01-01..9999-12-20 at 1M; any 1M
request starting on 0001-01-01..0001-02-18 and ending on
9999-11-21..9999-12-20 also reaches it. Requested ends of
9999-12-21 and 9999-12-31 fail as `terminal quarter`. Tenor errors come
before terminal failures, and terminal failures come before the count cap.

| ICMA profile | Rows |
|---|---:|
| Hand tables of the design, every number asserted | 30 |
| Cycle: every 23rd start, seven matching pairs and ten ends rotating | 6,353 |
| Cycle caps P−1 / P+1 on every 8th cycle row | 1,067 |
| Distances 0/1/29/30/31/60 at quarters of 2001/2024/2100, four ends, seven pairs | 2,016 |
| Civil early/late spans, plus cap-before-Date rows | 554 |
| Frequency domain: 15 tenors (5M, 9M, 18M, 2Y, 3Y, huge, …) × 11 frequencies × 2 spans | 330 |
| Precedence | 13 |
| **Total** (10,342 distinct inputs) | **10,363** |

ICMA outcomes:

| Status | Rows |
|---|---:|
| value | 9,391 |
| schedule.icma_period_limit_exceeded | 435 |
| schedule.icma_frequency_mismatch | 288 |
| schedule.date | 214 |
| schedule.invalid_icma_frequency | 31 |
| schedule.invalid_icma_period_limit | 4 |

Successful contexts carry 24,972 raw-interval fractions and 9,391 whole-span
fractions, with 1 to 39 periods each. Their lower steps reach 5 (1M) and
3 (2M). In 591 contexts an omitted A lies in the hull because it is on the
lattice; in 1,454 it does not. 2,413 hulls contain a non-raw reference strictly
between S and R (9,291 contain one anywhere, usually the lower bracket). There are 1,325 12M and 1,326 1Y Annual contexts. The
Date failures retain the stored unit: 159 lower and 55 upper, of which 48
use `add years`.

The generator asserts every design table:

- The six S = 2026-02-19 hulls: 125/372, 125/372, 119/360, 124/369, 121/364
  and 121/365. A is a reference date for 1M and 3M only.
- The 15/124, 1/12, 1/6 and 23/62 controls.
- 23/91, 46/183 and 8395/16653.
- The leap 30/91 and 29/(4·91) + 1/4.
- The three-period 1/4 control (31.25 amount).
- Every cap and Date control: required 2/2/4/2/2/4, indices 1/1/3/2/2/4,
  counts −1/−12/−3/+1/+12/+12, and 9999-12-01 succeeding at cap 1.

A census of all 146,097 cycle starts with E = S + 400 days reproduces the
maximum lower-step counts **5, 3, 2, 2, 1, 1** for 1/2/3/4/6/12M. The first
five-step Monthly start is **2001-02-19**.

## Additional checks (not retained artifacts)

A scratch verifier, not committed, used a third raw implementation:

- Candidate-date Q.
- The API review's phrasing of the shift.
- Day-walk interior points.
- Bisect lattice adjacency.

It reproduced every raw row, checksum and RLE. For ICMA it used an explicit
two-loop kernel simulation and per-day weight sums, and reproduced every ICMA
row, including correct rounding of each float. 1,941 ICMA rows matched raw
fixture rows with the same inputs.

That verifier also enforced the format rules of TESTING_STRATEGY §7.3. It
recomputed header counts, body SHA-256 and FNV-1a.

The frozen `roll-probes.fixture` was used only as a black-box sanity check.
70 of its OldCDS rows use Null calendar, Unadjusted/Unadjusted conventions,
no EOM or roles, positive Months/Years and status value. All 70 agree in
dates with this oracle, and 29 differ only in flags. That matches the
contract's 85/56 classification.

Two renders in separate processes, with randomized hash seeds, were
byte-identical. A one-character body change and a header count change were
detected by the `--check` comparison. A changed platform label was tolerated.

## Reproduction and integrity

The generators use only the standard library. They ran on Python 3.9.6,
Darwin arm64, in about 7 s and 2 s. Each `--check` re-renders in memory with
the recorded timestamp and compares header and body byte for byte, except
the `platform` and `reference-version` labels. That is the convention of
`cds_maturity_oracles.py`. Every command below exited with status zero:

```sh
python3 -I validation/generator/schedule_old_cds_oracles.py --check
python3 -I validation/generator/schedule_old_cds_icma_oracles.py --check
```

| Artifact | SHA-256 |
|---|---|
| Raw generator | `4b0c56ae65cb187795af1b129be67561a6c92afc1f5acc92073b5961cf5f7be6` |
| Raw fixture, 26,998,390 bytes | `3b510f0c46645484aa39097f67ed04e62d87c5673d7f3100b9b689e1f71e9d05` |
| Raw fixture body (FNV-1a `9caa7dc76387710d`) | `7129efd97acb9054d3fa46b4d2725367fdb439fb089a2e738852ce51ec140fce` |
| ICMA generator | `e1666ef77266802e87a63dab0132b0ab5c950bb754a5da1b96e0fc68f4fa532f` |
| ICMA fixture, 3,022,554 bytes | `b9958adcc7be06c5c3d585b0f79b5e0ded9ff9ebf1dd5b15f1aaee806104643d` |
| ICMA fixture body (FNV-1a `cae8d3325474ae6c`) | `d02fba8e7f00ef162babe718f55870bebb4a88a703ca3100d7bc98b765f86292` |
| Frozen contract (pinned in both headers) | `24aebdcb67cf5127df56705e85093c905767da787552325d83cb60a4e2b3b996` |
| ICMA design (pinned) | `cf89901c830167725d6a932b646bc6c7680a0aba1fb861ae661a8515ab1b119f` |
| Generated ICMA contract (pinned) | `abdc7dd61de1683399fbfc57403def012b251f72dd7ae734e6e322cc0eb25a74` |

## Limitations

The fixtures use the NullCalendar with Unadjusted ordinary and termination
conventions, so raw dates equal adjusted dates. Several obligations of the
contract are left to separate hand suites:

- The 49 convention pairs, holiday starts and collisions.
- Original and expanded TARGET guards.
- ICMA reference coverage.
- Identity and thread reuse.
- Coupon composition.

Terminal failures pin only the role and the input. The fixture does not pin
the inner DateError operation or displacement, because the contract does not
define it.

ICMA cap and Date rows model the generation order that the design documents.
They describe documented behavior, not an observed implementation.

The rows are a rotation over the cycle, not a Cartesian product. Repeated
inputs (259 raw, 21 ICMA) are natural overlaps between profiles. Checksums
and FNV are accidental-change checks, not authenticity claims.

The headers pin the generator, contract and design SHA-256 values, so
editing any of them requires regeneration and review. This evidence covers
only the adopted proleptic analytical definition. It does not establish
historical CDS premium practice, reference parity, financial or owner
acceptance, native Linux/Windows execution, or full M7 completion.
