# US/AFB implementation review

Local engineering review on 2026-10-16 against the adopted
[contract](M3_US_AFB_CONTRACT.md), based on `1c16daf`. This is neither
financial acceptance nor native Linux/Windows validation.

The primary integrator wrote AFB and the parent integrations. A separate
agent wrote US and its hand tests, then independently reviewed AFB without
editing its production formula. Two other agents authored the US and AFB
oracle generators/consumers without reading those production implementations.
The primary integrator completed unfinished reference/documentation work
after the delegates reached a usage limit; authorship independence does not
extend to every downstream consumer or documentation edit.

## Findings and resolutions

1. The O(1) AFB reduction is consistent with maximal repeated annual rollback.
   Every positive rollback of a February 28/29 end reaches the target February
   end; other month/day pairs stay fixed. Comparing the candidate in the
   starting year selects d or d−1 complete years. The residual is therefore
   in the start year or its successor. N=0 must retain the original end;
   promoting the caller's leap-year February 28 would be a defect. Production
   keeps that special case.
2. Residual February 29 is tested on the half-open interval. An included
   start February 29 selects 366, an excluded ending February 29 does not.
   Neither a year-zero/year-10000 sentinel nor an intermediate out-of-range
   Date is needed. Integer quantities fit their declared types throughout
   the supported domain. Whole-plus-quotient rounding is documented and
   retains the existing four-ULP policy.
3. Public fraction comparisons alone can hide distinct discrete decompositions
   with equal totals. The retained second embedded unit test now compares the
   actual private `(whole years, residual days, denominator)` tuple to an
   independent repeated-year/manual residual-day oracle before floating-point
   evaluation. Its 346,198 profile rows comprise every Gregorian cycle start,
   twenty profiles in each supported year and an 11×11 civil endpoint set;
   repetitions are visible. Both AFB unit tests passed in the root focused run.
4. The independent reviewer also ran a temporary snapshot harness: 1,265,989
   tuples, 173,012,004 actual annual attempts, maximum residual 365 days,
   maximum numerical distance one ULP. The original production snapshot was
   `c0f9a6ff2038a9798abd613826eed332ff273702392bfed44162e4163d295ab0`.
   This scratch evidence is supplementary; it is not a retained fixture or
   a claim that every civil date pair was enumerated.
5. US retains original February predicates and applies the four adjustments
   in the frozen order. Flag off is algebraically Bond Basis forward
   arithmetic but remains a distinct context. Chronological normalization
   ensures antisymmetry and canonical zero. The shared adjusted-day bound
   3,599,640 is valid and attained by MIN→MAX; the separate European bound
   3,599,639 remains valid for that separate helper.
6. The adapters preserve complete context in equality, hashing and conversion
   targets. Cash-flow query accounting charges one unit for both O(1)
   operations; no annual loop is hidden behind that charge. The composition
   suite verifies hand-derived times, all 32 compounding configurations,
   partial/raw versus payment roles, zero scalars and exact/short work limits.
7. Module documentation originally described only ISDA's operation order;
   it now states AFB's distinct residual-first order and source qualification.
   The US oracle report's duplicate-input subtraction was corrected to
   386 = 393,158 − 392,772. No fixture expectation changed for either finding.

No production algorithm defect was found in this review. Financial-source
limits remain explicit: US pins the reviewed ROC December 2025 A001 column,
not the unread full SIA/SIFMA book or a universal interpretation of the label.
AFB follows ISDA's published clarification; the original September 1994 AFB
publication remains unread. Full integrated checks and measured coverage are
recorded in [M3_US_AFB_VALIDATION](M3_US_AFB_VALIDATION.md).
