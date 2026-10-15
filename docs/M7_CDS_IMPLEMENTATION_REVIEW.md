# Independent review of the modern CDS date plan

Reviewed on 2026-10-15 UTC against the frozen
[local CDS contract](M7_CDS_CONTRACT.md). The reviewer independently inspected
the root-authored [date-plan implementation](../src/schedule/cds.rs) and its
[schedule integration](../src/schedule.rs), including their calendar calls.
This is an engineering review, not financial or owner acceptance.

The reviewer authored `src/schedule/cds_maturity.rs` and the original
`tests/schedule_cds_maturity.rs`. Consequently this report does **not** count
as independent implementation review of the maturity resolver or its original
tests. The root owns that review, its later source-table expansion, and final
aggregate validation. No production file was edited during this review.

## Findings and disposition

No unresolved date-plan algorithm or integration defect was found in the
reviewed snapshot. One failure-precedence issue was reported and resolved:
the original implementation reserved the coupon vector before adjusting and
checking the first coupon/precursor pair. Frozen contract step 5 establishes
that pair before generation. The root now constructs `first_raw` and
`first_payment`, calls `check_order(0, ...)` before `try_reserve_exact`, and
reuses those dates for position zero. The reviewer read the corrected code
and exercised it in the independent diagnostic below. Initial collision
rejection no longer depends on allocating future coupon records.

A low-priority documentation clarification was also reported and resolved:
the opening module paragraph in `schedule.rs` now identifies ordinary
Schedule generation separately from modern CDS plans. The root also corrected
the date-plan doctest's root Error path to `sabiquant::error::Error`. These
are documentation-only changes after the reviewed snapshot recorded below;
they do not change constructed values or failures.

## Algorithm and boundary proofs checked

The following are deductions from the frozen local contract and inspected
implementation, rather than new assertions of universal CDS practice.

| Area | Review conclusion |
|---|---|
| Initial validation | Coupon cap, increasing trade/maturity and quarter-twentieth maturity are checked in the stated order before calendar work. An original TARGET span is guarded before cutoff/quarter construction. |
| Selection domain | Increasing whole-day dates imply `trade + 1 <= maturity`. The greatest raw quarter `q <= cutoff` therefore cannot exceed the supplied maturity. A required year-zero cutoff quarter fails through the typed date error rather than being silently dropped. |
| First payment | If `A(q) <= cutoff`, `q + 3M` is strictly after cutoff; Following cannot move it backward. If `q == maturity`, no eligible raw quarter remains and the typed no-payment outcome is correct. If `A(q) > cutoff`, the selected raw quarter is `q`, with its preceding quarter retained for accrual. |
| Precursor sufficiency | When `A(q) > cutoff`, a predecessor also adjusted after cutoff must share `A(q)`: neither raw point can have an earlier business day between it and cutoff. Their collision is rejected. Otherwise the precursor is at or before cutoff. One neighboring quarter suffices; no backward search or deduplication is needed. |
| Exact coupon count | The first raw quarter and maturity are congruent quarterly coordinates, with first no later than maturity. `(maturity_month - first_month)/3 + 1` is a positive integer bounded by 39,996. The cap is applied before an additional precursor extension or allocation. |
| Raw generation | Every retained point is a direct three-month coordinate from the selected first quarter. The loop ends at the required maturity and does not construct an unused next quarter. It cannot accumulate month-clamping drift. |
| Adjustment collisions | Following is nondecreasing for a fixed immutable business-day set. Every required adjacent pair is required to increase strictly; collapsed payments fail instead of being sorted, skipped or merged. |
| Final accrual | If the preceding adjusted start were on or after raw maturity, it would itself be a business day available to Following at maturity, so the final adjusted payment could not be strictly later than that start. Strict ordering therefore proves `accrual_start < raw maturity`. |
| Included final day | Ordinary counts use adjusted end minus adjusted start; the final count uses raw maturity minus adjusted start plus one in integer arithmetic. Both are positive and bounded by the civil span. No `maturity + 1` construction is needed. |
| Civil errors | Required quarters are constructed from the supplied trade and the displacement to the required month, then day 20. The typed error retains the actual checked-date cause and trade input. Abstract or unused source anchors are not manufactured. |
| Resource behavior | Exact required count is known before one fallible reservation. The vector holds one immutable record per required coupon. Subsequent failure drops the internal vector and returns no partial plan. Calendar clones retain their documented ordinary allocation behavior. |

The `bounded_count` conversions are supported by these public-input proofs:
coupon count is positive and at most 39,996; accrual days are positive and no
larger than the civil date-span difference plus one. The review did not forge
private invalid state to reach an impossible negative count. Likewise, with a
valid increasing trade/maturity and a maturity inside the supported range,
the initial trade-plus-one overflow branch is unreachable. Required cutoff
quarters and precursors near the lower civil boundary can still fail.

## Coverage, integration and source chains

The original whole TARGET span is retained. Cutoff-quarter coverage expands
its start when necessary, and precursor coverage expands it again only after
the exact coupon cap passes. Final coverage runs from the earliest required
raw/trade start through the last adjusted payment. Following never shortens
the maturity end, so this final union includes the original input interval.

`TargetCalendar::business_days_between` checks endpoints and interior coverage,
including the known interval gap. Joint calendars retain candidate checks;
a Null member or an explicit business override does not remove the TARGET
guard. `TargetCalendar::adjust` and the underlying Following search retain
per-candidate coverage checks, civil-range failures and the existing
366-rejection search bound. The review inspected these call paths; the
closure diagnostic and retained regression below do not claim dynamic TARGET
validation.

The new rule catalog contains the nine canonical constructible labels.
Recurrent and explicit ordinary builders reject `Cds` and `Cds2015` after
their cap check and before other rule-specific work. Ordinary Schedule
ICMA/lattice paths do not turn these labels into a CDS premium leg. Plan
construction preserves explicit maturity and regime independently: the regime
only identifies the plan and rule; it does not re-resolve supplied maturity.

Plan equality/hash include the stored trade, cutoff, raw maturity, calendar,
regime and coupon records, while excluding the successful resource cap.
Public coupon values expose the raw, payment and accrual roles separately.
Date and Calendar causes are forwarded through `ScheduleError::source`, and
the existing root error retains the Schedule cause. Calendar error phases,
queried spans and optional coupon indexes match the frozen contract in the
inspected paths.

## Independent diagnostic and retained regression

A temporary Rust program, outside the repository, independently enumerated
quarterly twentieths from 2024 through 2027 and selected payments by direct
`adjusted > trade + 1` comparison. Its Following oracle walked daily dates
against an explicit holiday set and weekday membership; it did not call
`Calendar::adjust` or reproduce the production cutoff-quarter arithmetic.
For successful plans it compared every raw start/end, payment date, accrual
start/end, inclusion flag and day count exactly. It separately checked
collision and no-eligible-payment error kinds.

The completed matrix had two weekend settings, four closure-start dates,
closure lengths 0/1/2/3/89/90/91/92/93/180/275 days, fourteen trade controls,
and five supplied maturities. Non-increasing trade/maturity combinations
were omitted from this diagnostic because their validation was statically
reviewed. Every retained request used Quarterly2009 and the maximum allowed
coupon cap; regime identity, cap errors and TARGET were outside this matrix.

| Exact diagnostic outcome | Cases |
|---|---:|
| Successful plans, every coupon field matched | 3,490 |
| Adjusted collision rejection matched | 647 |
| No eligible payment matched | 175 |
| Total | 4,312 |

The original native Rust 1.99 library build and external diagnostic
compilation/run completed successfully. That temporary diagnostic source was
`/private/tmp/sabiquant-cds-plan-review.rs`, SHA-256
`55eea099051e33cdd7e936f927727f57cca3fd092167d878be47d53eba6c1810`.
Its original run was review evidence, not a retained regression.

The same matrix is now retained in
[tests/schedule_cds_closures.rs](../tests/schedule_cds_closures.rs), with exact
assertions for the unchanged 4,312-case census above. It additionally checks
successful plan input provenance, all no-payment error fields, and collision
index/raw/adjusted context. Collision context is derived by enumerating the
eligible raw dates and their independent daily adjustments under the frozen
contract, not by copying the production coordinate formula or reading the
returned error to create an expectation. No private invalid state or external
dependencies are used.

The retained regression passed **1 test, 0 failed, 0 ignored**, followed by
clean strict focused Clippy, using native Rust 1.99 and the isolated build
directory `/private/tmp/sabiquant-cds-closure-target`. The final source was
also tested after a test-only Clippy iterator simplification. Commands were:

```sh
PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-cds-closure-target cargo test --offline --locked --test schedule_cds_closures
PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH CARGO_TARGET_DIR=/private/tmp/sabiquant-cds-closure-target cargo clippy --offline --locked --test schedule_cds_closures -- -D warnings
```

This adds reproducible regression coverage, not a market fixture, package
artifact or claim of exhaustive calendar coverage. The root's final aggregate
rebuild and validation report retain ownership of the remaining gates.

## Maturity source-table check and independence limit

The reviewer re-inspected indexed primary text of the final
[December 10, 2015 ISDA FAQ](https://www.isda.org/a/vGiDE/amend-single-name-on-the-run-frequency-faq-revised-as-of-12-10.pdf).
The returned Q11 table contains all six date windows and 21 quoted tenors
from zero through five years in quarter-year increments. Thus the last two
windows in the original hand tests have direct final-FAQ support; they are
not merely inferred extensions of the four-window
[Standard Model news table](https://www.cdsmodel.com/news.html). The direct
PDF remained unavailable; this claim is limited to the indexed final text.

The root subsequently expanded the literal expected values to every published
tenor and all six windows, yielding 252 endpoint comparisons plus stored-year
alias controls. The reviewer read the expanded table and its source labels.
This is a source/provenance check, not an independent review of the maturity
production code this reviewer authored. It adds no new QuantLib runtime
observations and does not change the 96 modern versus 48 excluded OldCDS
accounting in the existing 144-call reference artifact.

## Reviewed snapshot and remaining gates

| Artifact | SHA-256 at review |
|---|---|
| Frozen CDS contract | `312df1e25a22c2f2575c30fd30853d315258ac3844ac0ab386d42a1db9c5aad5` |
| Corrected `src/schedule/cds.rs` | `9eca357a95ca92734b039064723cb07b4faebe084e7731100ce198aff308c999` |
| `src/schedule.rs` integration | `0381cdfa33a9f8858a0e25601585f72ef369e42dcc39769cc34d1200134e02db` |

The snapshot hashes above preserve the original review state; later root
module-opening and doctest Error-path corrections were documentation-only.
This report does not independently rerun or certify the full debug/release,
Clippy, rustdoc, package-extraction, coverage, platform or external-oracle
suites. Its additional retained-test claim is limited to the focused debug
and strict Clippy runs stated above. It does not force allocator failure, claim native Linux/Windows
execution, or treat unreachable proof branches as dynamically exercised.
OldCDS, legal protection/settlement, premium-leg adapters, financial/owner
acceptance, full M7 and publication remain separate requirements.
