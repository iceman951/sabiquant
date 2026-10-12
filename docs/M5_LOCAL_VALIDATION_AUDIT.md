# M5 local validation and release-gate audit

| Field | Evidence |
|---|---|
| Audit date | 2026-10-12 |
| Scope | Read-only production review of M1–M4 requirements, tests, fixtures, provenance and acceptance gates; targeted consistency edits to REQUIREMENTS, API_DESIGN_PRINCIPLES and DOMAIN_MODEL were subsequently delegated |
| Snapshot | Saved M4 commit `9aa59c237301e6d029eee2184fc24c07fb71a64f`; local M5 corrections prepared on `chore/m5-local-readiness`; refresh against the final reviewed commit before release |
| Authority | Delegated continuous **local** work; no external write, push, publication or separate milestone acceptance |
| Result | Strong scoped local evidence; M2–M4 acceptance and M5 release gates remain open |

This is a review record, not a declaration that M5 or a preceding milestone
is complete. [ROADMAP §2](ROADMAP.md#2-definition-of-done-for-implementation-milestones)
requires primary-platform execution, reviewed pull requests and owner
acceptance. M1's recorded merged PR and six-job CI success establish M1
only. Compilation for another target does not execute that target's tests.

## Evidence interpretation

The current native record reports 281 debug and 280 release tests, including
seven doctests, with no failed or ignored tests. The difference is the existing
debug-only internal invariant assertion. Strict Clippy, formatting,
warning-denied public/private rustdoc, zero-dependency inspection and Linux GNU
and Windows MSVC compilation checks pass. These results come from
[M4_NUMERICAL_VALIDATION](M4_NUMERICAL_VALIDATION.md); this audit did not repeat
the full expensive suite or pretend its recorded runs were new executions.

Those test totals describe the saved **pre-M5** implementation. The local M5
API corrections add three option enums, moving the reviewed public-type count
from 32 to 35, plus context Hash support. The independent API review records
99 focused integration tests, 11 executable and three compile-fail source
examples, an external 35-type consumer, strict Clippy and formatting after
those corrections. The full post-correction results are recorded separately
below; the older totals do not establish these changes passed.

Independent audit checks read every committed validation fixture, checked row
widths and unique IDs, verified each named generator's actual SHA-256 against
its header, checked both M4 imported-oracle digests and all QuantLib wheel
digests, and recomputed the inventories below. No inconsistency was found in
that snapshot. Generator `--check` and runtime byte-attestation are separately
recorded in the milestone reports. Hash agreement is integrity evidence, not
independent proof of a financial formula.

In the requirement tables, **local** means evidence exists on the documented
domains. **Review gap** means the literal specification or acceptance claim
still needs reconciliation. **Excluded** means no implementation or
compatibility claim is made. Every local row still needs the common
cross-platform and owner-review gates.

## Requirement-by-requirement evidence

The identifiers below are from [REQUIREMENTS](REQUIREMENTS.md). FS-DCC uses
section numbers rather than numbered `FR-DCC` requirements; this audit does
not invent identifiers for it.

| Requirement | Evidence and scope | Audit state |
|---|---|---|
| FR-CORE-01 | Domain/root `Result` errors; invalid inputs, formatter failures and root conversions in unit tests, `public_api`, and every domain's adversarial suite | Local |
| FR-CORE-02 | Stable kinds, structured context, `source()` downcasting and full Calendar→DayCount→Rate→root chains | Local |
| FR-CORE-03 | Numeric arbitrary-bit tests; nonfinite rate/time/factor/amount/unit inputs in `rates_core` and `rate_units` | Local |
| FR-CORE-04 | No clock/global evaluation date; policy lints, immutable contexts and concurrent reverse-order reuse | Local |
| FR-CORE-05 | Supported identifier values round-trip; contextual calendars, ICMA and Business/252 require explicit constructors, not name-based serialization | Local; A1 scope clarified |
| FR-CORE-06 | Exact aliases, whitespace/case/lookalike rejection and explicit ambiguity candidates; unsupported contextual names never create a guessed context | Local |
| FR-CORE-07 | Four named percent/basis-point helpers; exact basic operations, sign/zero/nonfinite/range tests | Local |
| FR-DATE-01 | Invalid component precedence, all first-invalid month days, supported range and extreme integers | Local: TS-DATE-03/04 |
| FR-DATE-02 | All 3,652,059 dates compared with independent civil/weekday counters; Python leap-boundary evidence for every year | Local: TS-DATE-01/06 |
| FR-DATE-03 | Signed day/week properties, adjacent range ends and widened overflow models | Local: TS-DATE-02/04 |
| FR-DATE-04 | All 119,988 month ends, independently clamped arithmetic and actual public QuantLib shifts | Local: TS-DATE-05 |
| FR-DATE-05 | Year/12-month equivalence across range and extreme shifts | Local: TS-DATE-05, INV-DATE-5 |
| FR-DATE-06 | Explicit noninvertibility/nonassociativity and absence of implicit EOM preservation | Local |
| FR-DATE-07 | Exact signed differences and chronological order over the full supported range | Local |
| FR-DATE-08 | Every-date ISO round-trip, malformed ASCII/Unicode/long strings; syntax versus semantic errors distinguished | Local: TS-DATE-07 |
| FR-DATE-09 | No today operation or clock access in production | Local policy evidence |
| FR-DATE-10 | Scanned nth weekdays over a complete 400-year cycle, absent fifth occurrences and inclusive search edges | Local |
| FR-DATE-11 | Private representation; no Excel/QuantLib serial conversion exposed | Excluded interop: OQ-12 |
| FR-PER-01 | Widened checked arithmetic, final-unit overflow, cancellation, zero identity and incompatible families | Local: TS-PER-03 |
| FR-PER-02 | Equality/hash/partial-order laws, zero substitution/transitivity and representation-dependent representability regressions | Local: TS-PER-01 |
| FR-PER-03 | Every frequency both ways and direct actual reference fields; exact Once/negative/normalized-tenor differences | Local: TS-PER-02 |
| FR-PER-04 | Signed stored text, ASCII grids, Unicode/composite/long-input rejection | Local |
| FR-PER-05 | Calendar Days versus Weeks semantics tested explicitly; equal normalized tenors need not dispatch alike | Local |
| FR-CAL-01 | Membership, business month start/end predicates and inclusive holiday/business lists; all mask and endpoint classes | Local: TS-CAL-01/02/05 |
| FR-CAL-02 | Null, weekends-only, all 128 custom weekend masks and both joint rules, including nested members | Local: TS-CAL-01/03 |
| FR-CAL-03 | Member/root overrides, weekend business overrides, cloning and immutability; no reference shared-state assumption | Local |
| FR-CAL-04 | Seven conventions, ties, half-month/month boundaries, empty months and bounded/range failures | Local: TS-BDC-01…05 |
| FR-CAL-05 | Zero adjustment, signed business Days, calendar Weeks, clamped Months/Years, adjusted/unadjusted EOM | Local: TS-CAL-04; corrected March-28 regression retained |
| FR-CAL-06 | Separate business-day stepping; zero is identity and 365/366/367 rejected-candidate cases are explicit | Local |
| FR-CAL-07 | All flags/equal/reverse cases; fast counts compared with independent daily prefixes over masks, overrides and joints | Local: TS-CAL-05 |
| FR-CAL-08 | All 9,999 Western Easter years against independent Gauss; 299 reference years and primary dated anchors | Local; pre-1582 results are mathematical, Orthodox excluded |
| FR-CAL-09 | Queryable TARGET provenance, disjoint verified intervals, strict/projection policies and checked guards throughout composition | Local; maintainer/review appointment outstanding |
| FR-CAL-10 | Pilot added only in the locally authorized M3 batch; no M0 dataset | Authorization history, not release acceptance |
| FR-BDC-01 | Every convention, business-day identity, half-month/tie/EOM cases and five exact pathological reference differences | Local: TS-BDC-01…05 |
| FR-BDC-02 | Closed enumeration with exact canonical identifiers and aliases | Local |
| FS-DCC §9.1 | Signed forward normalization, canonical zero within valid context, exact single quotients and anti-invariants | Local: TS-DCC-01/07; calendar endpoint counting remains separate |
| §9.2 Actual/360 | Exact actual integer count and one division; complete mandatory scalar windows and reference fixture | Local: F-DCC-01 |
| §9.3 Actual/365 Fixed | Fixed 365 denominator, leap-day numerator, ambiguous bare names rejected | Local: F-DCC-02 |
| §9.4 Actual/Actual ISDA | Independent daily leap/common weighting; bounded E2; source-conditioned D5 and upper-year D8 checks | Local: F-DCC-03; older QR-01 source comparison remains open |
| §9.5 Actual/Actual ICMA | Explicit ordered contiguous references/frequency, stubs, coverage failures, exact rational metadata, 3,338,660 window pairs and maximal segmentation | Local: F-DCC-04; no schedule inference or generation |
| §9.6 Actual/Actual AFB | February backward-roll primary rule unresolved | Excluded: F-DCC-05, OQ-16/QR-04 |
| §9.7 Bond Basis | Independent day-adjustment model, nonadditivity and signed direct-reference differences | Local: F-DCC-06 |
| §9.7 European 30E/360 | Independent capped-day integer model and exact quotient | Local: F-DCC-07 |
| §9.7 30E/360 ISDA | Required termination date; 4,319,568 month-end/context checks, February exception and signed/zero differences | Local: F-DCC-08 |
| §9.7 US 30/360 | February/EOM qualification not established by the retrieved primary material | Excluded: F-DCC-09, OQ-15/QR-02 |
| §9.8 Business/252 | Explicit custom/guarded calendar, half-open signed count and single division; no default Brazil/global cache | Local: F-DCC-10 |
| FR-RATE-01 | Five methods, ten periodic frequencies, all 32 configurations and exact branch selection | Local: TS-RATE-01…04/09 |
| FR-RATE-02 | Time/date factors for all eight owned accrual contexts, including strict TARGET and ICMA errors | Local: `rates_dates`; 24,320 quote configurations |
| FR-RATE-03 | Compound/discount inference, exact zero-time rule, tiny/subnormal/extreme inputs and rounded active-domain rejection | Local: TS-RATE-04…07 |
| FR-RATE-04 | 32² conversions and 1,245,184 explicit context/date combinations; unequal horizons, coverage and zero-target checks | Local: TS-RATE-08; representation conditioning documented |
| FR-RATE-05 | All invalid domains, nonfinite inputs, lost nonzero results and numerical range bands return typed errors | Local: TS-RATE-04/05/07; A4 computed-status qualification recorded |
| FR-RATE-06 | Negative valid rates, next valid negative-base neighbors and active Simple boundaries | Local: TS-RATE-03; A3 hybrid-conditioning qualification recorded |
| FR-DISC-01 | Single-amount time/date PV, negative amounts, exact identity, conditioned linearity and product overflow/underflow | Local: TS-DISC-01 |
| FR-DISC-02 | Reversed dates reject before accrual, amount or zero shortcuts | Local: `rates_dates` |

## Cross-cutting requirements

| Requirement | Evidence / remaining qualification |
|---|---|
| NFR-01 | Source/spec contracts exist; A5 wording corrected; QR-01 and US/AFB primary-source research remain open |
| NFR-02 | Typed boundary errors, deterministic adversarial campaigns and bounded searches; this is scoped evidence, not exhaustive proof over arbitrary allocation/input sizes |
| NFR-03 | Discrete/basic/identity determinism, owned contexts and concurrent reuse; transcendental results intentionally use conditioned tolerances |
| NFR-04 | M1 recorded primary CI; current M2–M4 native Linux/Windows execution missing; A4 numerical classification qualification recorded |
| NFR-05 | Public value/error Send/Sync assertions across `public_api`, `target`, `daycount_extended`, `icma` and rate suites |
| NFR-06 | Crate/manifest forbid unsafe; strict lints pass |
| NFR-07 | No observable mutable global state; context ownership and immutable overrides tested |
| NFR-08 | Warning-denied public/private rustdoc and doctests pass; domain sources reviewed; prose/package link review still needed |
| NFR-09 | Manifest/lockfile policy tests and Cargo tree identify this crate alone |
| NFR-10 | Declared Rust 1.99 MSRV and nonexhaustive growing types; clean candidate version/tag and API owner sign-off still required |
| NFR-11 | O(1) date conversion; bounded searches and optimized counts; A2 explicit ICMA O(log n+k) exception recorded |
| NFR-12 | All 32 fixture headers/schema IDs and source/dependency hashes checked; reference runtime separately attested |
| NFR-13 | Production I/O/clock/environment/output restrictions and review; tests alone read committed data |
| NFR-14 | Annual decimal rates and year fractions documented; named percent/basis-point helpers tested |
| NFR-15 | Kind/context/source chains and formatter-failure propagation checked; display wording not frozen |

## Fixture and provenance ledger

There are **32 files / 598,564 rows**: 480,863 actual public QuantLib rows and
117,701 independent-oracle rows. These are scenario records, not independent
features, branch counts or exhaustive binary64 coverage. The five domain
fingerprints below hash the UTF-8 sequence `relative_path<TAB>file_sha256<LF>`
in sorted path order, relative to `validation/fixtures/`. Each file hash covers
the complete recorded bytes, including provenance and timestamp. Regeneration
can legitimately change that fingerprint and therefore needs renewed review.

| Domain | Files / rows | Snapshot SHA-256 |
|---|---:|---|
| Time | 7 / 159,947 | `ebb1e555aa15e936ef245a73ad8a4d44425c744fc4e313f0b1703cde87a122ea` |
| Calendar | 8 / 75,227 | `9670d0fa576a5dc3c1c5ae0f988dfe28d8e3ddc987841e14605c8db4477aa20c` |
| Day count, including ICMA | 4 / 72,964 | `abc178701f50ff5b3e10755dce65d3cd32f7f5719bfc650148b0e3e6058ed6a7` |
| Market/Easter/contextual counts | 3 / 135,848 | `a99a58648702f8adce16d8c68cf66d010c0afafdeabe1750c2051026f911fff2` |
| Rates | 10 / 154,578 | `200685c73cc160af16124cc114f1c089ac340f0d94f45cf201be27d7593ea599` |

| Generator | Actual SHA-256 matched by fixture metadata |
|---|---|
| `time_fixtures.py` | `100e2ea12b8d8c55b2d5d7e8649794a2d987fec7c286766da66e139cd5b2441c` |
| `calendar_fixtures.py` | `31f27c5ffb9baa667fd6504e9355b9301c59467a5341ad63df2645fb5f5c738d` |
| `daycount_fixtures.py` | `512df5d48c3cdadffddfde38e217cf5f9f6f5ad41651fd59eae18c42af2638f2` |
| `market_fixtures.py` | `0ac191bc5db6074270a20306c75548b89275734f421bb476aa8a920ed6521719` |
| `icma_fixtures.py` | `3e804f6edcb9df52ffd2134d5e54ebabe56909c08cb4bd043b0674462972964f` |
| `rate_oracles.py` | `a7217309db77fc0c908de9571e0c172ac9e5df597b39ebc53ba984887d7dd573` |
| `rate_boundaries.py` | `52b448f5b0ad2751a5fdbadc18db143cf4d301fd9450241b9e59968503f7774f` |
| `rate_quantlib_fixtures.py` | `1938e25f76d63e302bea1662232d4270f851140ab829185f1a007336145b673e` |

The 23 QuantLib fixture sets all identify version 1.43 and wheel
`quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl`, SHA-256
`85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c`.
Generators compare installed package files, including the native binary, with
that artifact. Commit `6b57206e04598f092efee66e3b367efc84771995` is the declared
QuantLib source baseline; the wheel's source commit is **not independently
attested**. Reference agreement neither authenticates a market source nor
promotes a projected calendar year to verified.

| Financial source/evidence | Provenance and scope |
|---|---|
| Gregorian dates | FS-DATE rules, independent nested counters and Python datetime/calendar; broader date range has oracle evidence only |
| Core day counts / Bond / 30E ISDA | Original 2006 ISDA §4.13/4.16 document content reviewed on a TD Securities host, as recorded in [M3_DAYCOUNT_RESEARCH](M3_DAYCOUNT_RESEARCH.md); hosting authenticity/current amendment completeness not independently attested |
| ICMA | ICMA PMH Appendix A5, March 2022, §1.3; explicit determination boundaries, exact Fraction overlaps and compensated positive summation; no schedule-based general parity |
| Business/252 | B3 business-day/252 and start-inclusive/end-exclusive methodology; supplied calendar controls counts; no Brazilian-calendar claim |
| Easter/TARGET | USNO computus with independent Gauss; ECB launch, 1999–2002 decisions and dated 2026–2028 TARGET anchors; office holidays and other TARGET services excluded |
| Rates | FS-RATE equations plus independent Fraction and Decimal 120/200-digit formulas; public InterestRate comparisons and exact power-of-two anchors; precision agreement is not a correct-rounding certificate |

The current [discrepancy log](../validation/DISCREPANCIES.md) and
[M4 reference report](M4_REFERENCE_VALIDATION.md) require narrow predicates,
recorded outputs and tested counts. Time has explicit wider-range/frequency/
Once/order D9 cases. Day counts have 8,372 conditioned same-year ISDA D5 rows,
452 confirmed upper-year D8 rows, signed/zero D9 cases and two ICMA strict
coverage differences. Calendar has exactly five finite bounded/empty-month
D9 cases. M4 has 48,315 reference agreements, 21,594 source-rounding D5 rows
and 7,537 specified D9 rows; its 232 guard-edge annotations are orthogonal,
not extra rows. The financial gate rejects every true out-of-range finite
result and keeps unchanged E3/E4 bounds for finite truth. No generic
"QuantLib differs" exemption is acceptable.

## Audit findings and local dispositions

These are concrete review findings, not new features or requests to weaken
the tests. A correction should retain the explicit financial-value oracle.

1. **A1 — Identifier scope; clarified locally.** The earlier FR-CORE-05
   wording required value round-trip for every calendar/convention.
   `Calendar` and `TargetCalendar` have no
   Display/FromStr round-trip, and a convention name cannot supply a custom
   calendar, termination date or ICMA references. FR-CORE-05/06 and the API/domain
   documents now separate identifier round-trip from explicit contextual
   construction. No default or full-graph parser was invented.
2. **A2 — ICMA complexity; clarified locally.** The earlier NFR-11
   non-calendar year-fraction O(1) claim conflicted with the correct
   explicit-segment ICMA O(log n+k) evaluation.
   NFR-11 now records O(n) construction, O(log n+k) queries and the maximum
   valid segment count, without hiding work or dropping overlaps.
3. **A3 — Hybrid conditioning; qualified in saved M4.** NP-12's earlier
   rounding-level switch claim and unconditional TS-RATE-09/INV-R-4
   continuity/monotonicity are false near
   `r=-m` with a rounded `1/m` switch. A separate compiled public-API audit
   found EveryFourthWeek, `r=-12.999999999999998`,
   `t=0.07692307692307693`: SimpleThenCompounded factors are
   `8.113168256876145e-17` at `t` and `1.3664283380001849e-16` at
   `t.next_up()`, a 68.4% increase despite negative rate. Each selected
   formula agrees with its independent oracle. CompoundedThenSimple can
   instead cross its active Simple domain at the next horizon. Saved M4
   qualifies NP-12, TS-RATE-09 and INV-R-4 by input/switch conditioning,
   retaining the specified rounded switch and independent branch tests.
4. **A4 — Numerical status agreement; qualified in saved M4.** NFR-04's
   earlier unconditional identical discrete-output wording conflicted with the
   measured outer guard-edge
   policy, which can permit finite/Boundary or Value/Boundary depending on
   computed libm arguments. Exact date/enumeration/basic-domain statuses stay
   identical; saved M4 records the explicit R3 numerical financial gate.
5. **A5 — Source wording; corrected locally, research still open.** Earlier
   REQUIREMENTS §9 said the 2006 Definitions were not consulted and described
   1999 examples as verified, while the
   later source review recorded consulted original content and an unread
   1999 scan/QR-01 gap. The source paragraph and convention tables now name the
   consulted copy/authenticity/amendment limits and label formula-derived
   examples honestly. This does not resolve the unread 1999 source or US/AFB.
6. **A6 — Traceability/compatibility.** REQUIREMENTS §12.1 maps M1 concretely
   but later rows remained module-level. New §12.3 links this local
   per-requirement evidence map; owner review must give every foundational
   feature its scoped compatibility status. TS-DCC-02/04/07 cannot be called
   complete for US/AFB while those variants are excluded. NASD/QR-05 and other
   research variants remain later work.
7. **A7 — Package usability; corrected and verified locally.**
   The independent [M5 package review](M5_LOCAL_PACKAGE_REVIEW.md) initially
   found extracted `cargo test --lib` failing because `src/lib.rs` imported
   excluded `tests/support/mod.rs` under cfg(test), and packaged README
   relative links targeting omitted documents. The manifest now includes
   four private cfg(test) support files and `clippy.toml`; the configuration
   is needed for strict extracted-package Clippy. All 48 README links now use
   repository URLs, with 38 distinct targets checked locally. No reference
   data/generator or integration tests are shipped. The 26-file source-frozen
   archive passes 73 library unit tests, 11 ordinary and three compile-fail
   doctests, strict all-target Clippy and warning-denied public/private docs.
   A clean release archive remains a separate gate; local link checks do not
   prove unpublished remote pages already exist.
8. **A8 — Adopted API policy; corrected in the local candidate.** The independent
   [M5 API review](M5_LOCAL_API_REVIEW.md) identified Boolean calendar options
   where API_DESIGN_PRINCIPLES §4/§12 prescribes descriptive enums,
   `TargetCalendar::Default` despite §4/§7's no-calendar-default rule, and
   missing Hash on discrete Eq contexts `ActualActualIcma` and
   `AccrualConvention`. The candidate now has the three named option enums,
   explicit strict `TargetCalendar::new()` without Default, and Hash for both
   discrete Eq contexts. The focused and full native evidence passes; owner
   API review remains distinct from this implementation correction.
   Also retain the documented operational distinction: normalized `7D == 1W`
   advances a weekends-only calendar from 2026-01-02 to 2026-01-13 versus
   2026-01-09. Normalized Period equality/hash therefore cannot alone key a
   cache for calendar advancement; the stored operation unit is relevant.

No new unclassified financial-value mismatch was established in this audit.
The recorded clarifications are agent decisions under delegated local
authority ([REQUIREMENTS §12.2](REQUIREMENTS.md#122-local-m5-consistency-decisions)),
not owner API acceptance. Remaining gates prevent compatibility/release claims
from outrunning the actual evidence.

## Post-correction native evidence

The source-frozen local M5 run on aarch64 macOS/Rust 1.99 passes **292 debug
tests and 291 release tests across 29 suites**, including **14 doctests**,
with zero failures or ignored tests. Linux GNU and Windows MSVC all-target
compilation checks pass; they remain compile checks, not native R3 execution.
Strict Clippy, formatting and documentation evidence is retained with the
API/package reviews. These totals are the root's recorded complete run, not
a duplicated execution by this audit.

The fresh LLVM report is
`/private/tmp/sabiquant-m5-final-coverage`; embedded test modules are excluded
from these production source-region counts:

| Source | Covered / regions | Coverage |
|---|---:|---:|
| `calendar.rs` | 799 / 838 | 95.35% |
| `calendar/target.rs` | 386 / 405 | 95.31% |
| `daycount.rs` | 314 / 314 | 100% |
| `daycount/icma.rs` | 163 / 164 | 99.39% |
| `rates.rs` | 778 / 785 | 99.11% |

The invariant-only/defensive paths and compile-time effects discussed in the
M2–M4 reports remain qualifications. Region coverage is not branch coverage,
an input-domain proof or permission to fabricate invalid private states.

The frozen package at
`/private/tmp/sabiquant-m5-package-audit-final/package/sabiquant-0.0.0.crate`
has SHA-256
`ff8756a4a9bf5ffed647e28d5df3820f3b94531a24b5113975dcd03528d54b88`,
independently recomputed by this audit. It contains 26 files / 94,876 compressed
bytes. Its VCS metadata correctly identifies saved M4 `9aa59c2` plus a dirty
local M5 tree, not a clean accepted release or the older M3 snapshot.
The [package review](M5_LOCAL_PACKAGE_REVIEW.md) records extracted checks and
shipped-source/support group hashes. A version, source, shipped-document or
final commit/tag change requires a new archive and digest.

## Reachable checks and remaining gates, in order

1. Review the recorded clarifications and already checked A7/A8 package/API
   corrections, adding targeted regressions if behavior changes. Recheck
   existing fixtures after any
   generator change; adding another large duplicated random campaign has
   lower value than fixing these concrete gaps.
2. Freeze a reviewed M4/M5 candidate commit and regenerate the evidence
   manifest. Record exact commands, toolchain and report locations. Source
   regions are informational: rates 778/785 and the separately measured
   M2/M3 tables do not measure Rust branch coverage or all input combinations.
   The 28 saturated E4-bound rows remain excluded from numerical accuracy
   claims. Do not fabricate invalid private states to inflate coverage.
3. Execute current code on primary x86_64 Linux, aarch64 macOS and x86_64
   Windows, with the declared MSRV check and all shared oracle/error gates.
   The current workflow is configured for this; only M1 has recorded remote
   success. Cross-target checks and a local macOS run do not close this gate.
   Review any guard-edge differences using the existing financial gate,
   never platform-specific expected values or a widened global epsilon.
4. Resolve OQ-15/OQ-16 from the exact SIFMA/AFB source texts, **or** have the
   owner explicitly accept their first-release exclusion and corresponding
   milestone scope. Obtain/inspect the older ISDA examples for QR-01 rather
   than claiming that matching a formula transcribes the unread scan.
5. Appoint the TARGET calendar maintainer and record annual ECB review and
   historical-correction procedures, or explicitly exclude the pilot from
   release. Verified coverage remains only 1999-01-04–2002-12-31 and
   2026-01-01–2028-12-31; 2003–2025 and later years remain projected even when
   reference values match. Strict Nearest's coverage-boundary error and
   deferred validation of immutable override declarations need owner review.
6. Obtain independent financial/API review and owner acceptance of M2–M4
   scope/decisions; reviewed PRs must meet provenance and DCO requirements.
   Confirm required CI checks and GitHub private vulnerability reporting
   (OQ-26) from settings evidence. SECURITY.md's requested channel does not
   prove the setting is enabled. No settings or external writes were made.
7. Review a clean 0.1.0 candidate archive/tag, version/MSRV, licenses,
   changelog/behavior changes and current crate-name availability at release
   time. The current manifest remains `0.0.0`, `publish=false`. OQ-25 permits
   publication only after explicit owner approval at M5; record **publish or
   hold** as a separate final release decision. Local authorization does not
   supply that decision.

The next deliverable is a corrected, reviewable local candidate and complete
evidence record. External CI submission, merge, acceptance and publication
remain distinct actions with their own recorded authority.
