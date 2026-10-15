# OldCDS implementation review

Review completed 2026-10-15, Asia/Bangkok, on the local worktree based on
`f814167`, before the tranche was committed. Two read-only reviewers worked
independently. One examined production correctness and robustness; the other
examined evidence, test strength and documentation consistency. Neither
edited repository files. The integrator verified every finding before
changing anything. Their independent oracles had been frozen before they read
production code; this report does not claim continued blindness after review.

## Method

The code reviewer read `src/schedule/old_cds.rs`, the OldCDS paths of
`src/schedule.rs`, the `roll.rs` helper changes and the error catalog against
the [contract](M7_OLD_CDS_CONTRACT.md). A separate scratch oracle written from
the contract text compared about 3.16 million builds with overflow checks and
debug assertions enabled. It covered starts from 2023-11 to 2025-02, ends one
to five hundred days later, thirteen stored tenors including `i32::MAX`
Months/Years, every exact cap and cap − 1, and ICMA hulls for matching
frequencies. Civil-extreme spans near 0001-01-01 and 9999-12-31 were also
checked. There were no mismatches.

The evidence reviewer rechecked every pinned hash, fixture size, record count
and classification count, resolved relative links, confirmed the cited
requirement IDs, reran all three generator `--check` modes and the new tests
in release, and built one scratch mutant to test a Display assertion. That
scratch work is not retained evidence.

## Correctness conclusions

No correctness or robustness defect was found. The reviewer confirmed the
quarter projections, the B rule, stop-at-end generation, the R terminal,
regularity, provenance and omission semantics, validation order, the
closed-form count, error roles/inputs/causes, Eq/Hash and the ICMA lattice.
No input can panic: arithmetic stays in bounded i64, `12 * i32::MIN` fits,
every stride used in `rem_euclid` is positive, the only conversion is a
`u32::try_from(..).unwrap_or(..)` fallback and every index is guarded.

## Findings and resolutions

| ID | Severity | Finding | Resolution |
|---|---|---|---|
| C1 | minor | The contract said intervals spanning an omitted quarter are Stub. With 4M from 20 February, February 20 → June 20 is Regular under the contract's own definition | Contract reworded; behavior unchanged. Two pinned hand rows (2026 and leap 2024) added. Oracle fixtures regenerated for the new contract hash, with bodies byte-identical |
| C2 | nit | "Quarter-twentieth" wording although interiors follow the stride | Reworded to twentieth schedules on quarterly anchors in rustdoc, crate docs, README and requirements |
| C3 | nit | `old_cds_provenance` rustdoc overstated what raw dates cannot recover | Now names the requested end and stored tenor as the unrecoverable inputs |
| C4 | nit | `build_old_cds` error order omitted fallible storage | `AllocationFailed` added to the documented order |
| E1 | minor | Oracle report called 0001-01-01..9999-12-20 the only maximizing input | Reworded; the maximizing input set is stated |
| E2 | minor | FR-SCHED-33 cited a ten-label registry in `schedule_cds.rs` | Cites `schedule_roll.rs`/`schedule_old_cds.rs`; `schedule_cds.rs` has a parse check |
| E3 | minor | Probe consumer did not check AdjustedOrder context or flag-difference positions | Index, raw pair and model adjusted pair are now compared; positions are recomputed for every row; frozen census first 9, last 13, sole 7 is asserted |
| E4 | minor | Three oracle-report census sentences named broader predicates | Predicates corrected; the integrator recomputed 46,277/107,290, 11,266/17,001 and 2,413/9,291 |
| E5 | nit | Display test could not detect a missing input | Asserts `required terminal quarter from 9999-12-25` |
| E6 | nit | Inception claim was tested only with plain policies | The inception request now runs over overrides and every joint rule |
| E7 | nit | Oracle report said no Rust consumer existed | Marked historical with a pointer to the validation report |
| E8 | nit | "Current" type catalog of 71 in two documents | Marked as that tranche's count; the readiness section states 72 |
| E9 | nit | Modern CDS report still read as current for labels/OldCDS | Snapshot note and forward links added |
| E10 | nit | Validation report linked this missing review | This document |

## Integrator finding

The first coverage run showed that an early refactor routed Twentieth and
TwentiethIMM endpoint projection around the shared `roll_date` helper. Six
civil-roll error regions became reachable only through Wednesday rules, where
they are proof-excluded. The refactor was narrowed so `roll.rs` keeps its
original control flow and shares only the month offset and checked roll-date
helpers. Its uncovered regions returned to the original eleven.

## Remaining limits

These reviews concern the local engineering definition. They do not resolve
the historical cutoff or start role in the
[legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md), CDS premium accrual,
protection or settlement. Native primary-platform execution and owner,
financial and release gates stay open.
