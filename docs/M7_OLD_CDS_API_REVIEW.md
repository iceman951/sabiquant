# Proposed OldCDS API, provenance and error review

Read-only design review, 2026-10-15, Asia/Bangkok, against the modern CDS
baseline identified by the integrating author as `f814167`. The proposed
positive stored Months/Years operation is compatible with the existing
Schedule value model, subject to the explicit choices below. This is a
pre-implementation review, not an implemented-behavior or test-pass claim.
The reviewer wrote only this report and performed no Git operation.

The [initial local design](M7_OLD_CDS_LOCAL_DESIGN.md) supplied a fixed-3M
proposal. Its no-tenor signature, quarter-only regularity/ICMA, 39,997-boundary
maximum and 63-row expressible subset do **not** describe the broader domain
reviewed here. They must not silently become the adopted contract. The
[legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md) still leaves historical
financial start roles and exact thirty-day equality unresolved.

## Public operation and retained meaning

Recommend `ScheduleBuilder::build_old_cds(tenor: Period, max_boundaries: u32)
-> Result<Schedule, ScheduleError>`. Accept strictly positive stored Months
and Years, converting Years to months in i64. Reject zero, negative values
including i32::MIN, and Days/Weeks; do not normalize them into another domain.
There is no divisibility-by-three restriction. Positive enormous tenors remain
valid even when no further stride point is needed.

For the builder's generation start S and requested end E, with S<E, let A and
R be the first quarterly twentieths on/after S and E respectively. Start B at
A, replacing it by A+3M when A=S, or when 0<A−S<30 calendar days and A<R.
Return S, the points B+k times the supplied month stride strictly before E,
and R. B is the actual first retained twentieth, including when it equals
the required terminal R. The shift to B is three months regardless of stride.
It is distinct from adding one supplied tenor.

This automatically selects the front boundary and preserves a near terminal:
when A=R the cutoff cannot remove R. Equality at thirty days retains A;
equality A=S avoids duplicating S and records no omission. These are explicit
local engineering choices, not an inferred legal trade/effective-date rule.

The existing builder conventions apply to S/interiors and R separately.
They remain generic analytical options. Fresh reference probing by the
separate scope reviewer has identified differences involving holiday starts
and terminal selection; the adopted contract and validation must classify
them explicitly. Neither the OldCDS identifier nor agreement on a subset
establishes complete reference parity or historical market acceptance.

## Provenance, equality and catalogue

Add public immutable `OldCdsProvenance` with private fields and inspectors for
`generation_start`, `requested_end`, `initial_quarter` A, `resolved_end` R,
`first_retained_quarter` B, optional `omitted_quarter`, and original tenor.
Store tenor length/unit separately for derived Eq/Hash, following
`RollProvenance`; normalized Period equality must not merge 1Y and 12M.
`omitted_quarter` is Some(A) only for the actual interior cutoff omission,
not for A=S or A=R. A stored B is necessary because the recurrence uses B,
whereas A records the original cutoff candidate.

Schedule should retain `Some(original_tenor)`, `OldCds` and
`Some(old_cds_provenance)`. Stub policy, EOM, explicit roles, civil-roll
provenance and the public ordinary `lattice_anchor()` remain None. An internal
day-20 recurrence helper does not make Forward/EOM/civil-roll metadata true.
Existing Schedule constructors must initialize the new provenance to None.

Both manual Schedule equality and hashing must include the new provenance.
Different requested ends projecting to the same vectors remain distinct
definitions; stored aliases remain distinct. The cap is excluded from identity.
Calendar and both explicit conventions retain their existing identity roles.
Expose provenance by immutable borrow, with Copy/Clone/Debug/Eq/Hash and
Send/Sync/'static checks; existing Schedule remains Clone/Debug/Eq/Hash.

Add only `DateGenerationRule::OldCds`, canonical `OldCDS`, to the non-exhaustive
generation enum and its exact parser/display registry. `ALL` then has ten
labels; `SpecialRollRule::ALL` remains four. Recurrent and explicit builders
must reject OldCds through `InvalidRule`, after their cap check. Update
catalogue completeness and rejection tests without weakening unrelated
identifier strictness. One new public provenance type raises the current
71-value trait inventory to 72, subject to the final declaration census.

## Counts, guards and deterministic failures

The constructor continues to reject S>=E. For a valid builder, recommend this
build order: cap; stored tenor domain; original inclusive S..E calendar guard;
required terminal R; A and B; expanded inclusive S..R guard; exact final count
and cap; fallible raw allocation/generation; existing adjustment/order checks.
Calendar failures before a cap-exceeded result remain observable under this
ordering and need explicit precedence tests.

The terminal must be resolved first. An E after 9999-12-20 requires an
unrepresentable terminal and must fail; truncation is invalid. Once R exists,
S<B<=R proves that A and B cannot escape the civil range. Their checked error
propagations may remain, but coverage reporting should identify this proof
instead of manufacturing invalid private states.

Count interiors using the requested E, not R. If b is B's month coordinate
and L is E's month coordinate minus one when E.day<=20, the interior count is
zero for b>L, otherwise `floor((L-b)/stride)+1`. Final count is that value plus
two. This includes S and R, excludes the omitted A, and constructs no unused
huge or post-terminal candidate. Reuse the uniform 2..3,652,059 boundary-cap
domain; report the exact count in `BoundaryLimitExceeded.required_at_least`.
With 1M, the attainable civil maximum is **119,987** boundaries, not the
fixed-quarter maximum. An independent temporary Python tuple enumeration
confirmed this at Date::MIN through requested 9999-12-20, and agreed with the
closed count on seven additional short-span/huge-stride controls. This was
design arithmetic only, not production execution or a retained oracle.

Retain the existing Calendar failure context: index None for whole-span
coverage, a distinct phase such as `old CDS coverage` for S..R, and actual
zero-based indices with phase `adjustment` for individual dates. Original and
expanded guards must include TARGET inception, gaps, projection policy,
overrides and joint members, including Unadjusted or null joint components.
Every adjustment search must retain its own checked candidates, range/search
failures and guard. Reject collisions or reversal; do not repair them.

Recommended new non-exhaustive errors are:

| Variant and stable kind | Required context |
|---|---|
| `InvalidOldCdsTenor`, `schedule.invalid_old_cds_tenor` | generation_start, requested_end, original stored tenor |
| `OldCdsDate`, `schedule.old_cds_date` | same definition context, role, original role input and actual DateError cause |

Projection roles should distinguish terminal quarter, initial quarter and
first retained quarter. Use E as the terminal input, S as the initial input,
and A as the input for its required +3M shift. Preserve the actual signed
month displacement and Date cause; do not invent an output-array index for
projection. Reuse existing limit, allocation, Calendar and AdjustedOrder
errors where their metadata remains truthful. Extend Display, stable-kind,
root variant/formatter tests and `StdError::source()` through the existing
root ScheduleError wrapper. No new root wrapper is needed.

## Regularity, ICMA and integration risks

Regularity must use the actual B-anchored supplied-stride twentieth lattice:
both endpoints belong to it and differ by one stride. All other intervals
are Stub, never NonPeriodic. For 1M/2M/4M, interiors can be non-quarter
twentieths; the projected terminal can lie off-stride. An omitted A need not
be an ICMA quasi-date for a non-quarter stride.

Generated ICMA can reuse the internal anchored lattice with B and
preserve=false while keeping the public ordinary anchor None. Require the
existing explicit matching Frequency, including normalized 1Y/12M matching;
do not force Quarterly for every OldCDS tenor. Build the minimal contiguous
stride hull covering raw S..R, including any omitted points that belong to
that lattice. Ordinary matching/unsupported frequency errors remain valid.
Retain cap precedence, checked source causes, fallible storage and full
extended TARGET coverage.

There is no one-step lower-extension bound. For S=2026-02-19, omitted A=Mar20
and B=Jun20, Monthly references must step backward through May20, Apr20,
Mar20 and Feb20 to Jan20. The existing iterative minimal-hull machinery can
handle this; a civil-roll shortcut could omit required periods. A raw
schedule can succeed while separate ICMA fails for a required year-zero or
unverified prior reference. Such failure is not a raw-generation error.

Before implementation is called complete, retain independent cases for
domain/cap precedence, exact equality/cutoff/terminal behavior, all stored
strides and aliases, maximum count, unused huge candidates, full guards,
all convention pairs, immutable identity and source chains. Recheck every
generation-rule match, shared finalizer constructor, Schedule Eq/Hash path,
public trait inventory and existing reference classifier. Preserve historical
fixtures; new reference divergences require classification, not rewritten
expectations. Generic raw-accrual fixed-coupon composition does not establish
historical CDS premium accrual, legal protection or settlement. Modern CDS
plans and their distinct accrual roles remain separate.

## Inspected baseline

| File read | SHA-256 at review |
|---|---|
| `src/schedule.rs` | `b6cd16a27cb4e89e5679baebe0e33c4be94670d3ed4c6e2af69d8638125e88a5` |
| `src/schedule/roll.rs` | `0bc794c20615b55e6f9f576919765152ae0e42f8b05885badbb00b9be4a980fe` |
| `src/error.rs` | `4803233181ed884ab45c4d3e677212077a4fd7fbd5748f1a3b79d9883f064683` |
| Initial fixed-quarter proposal | `87043a75e7d6061a3f40d2ea4ed8a3b2c9db26ff85aa4ddc222617b9ee736baa` |

No OldCDS production implementation was present in these inspected bytes.
No new Cargo, reference-runtime or aggregate check was run for this review.
The adopted contract and later implementation review must resolve any
remaining choices; financial/owner, NI-01, native-platform and release gates
are unaffected.
