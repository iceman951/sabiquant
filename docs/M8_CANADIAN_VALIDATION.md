# Canadian fixed-coupon local implementation validation

Verified local engineering checkpoint, 2026-10-24 Bangkok on
`feat/m8-canadian-fixed-coupon` after the frozen preproduction admission
`ceb8475`. The complete [adopted contract](M8_CANADIAN_CONTRACT.md) and its
fourteen components are unchanged. The engineering scope is the complete
f2/4/12 regular, short, long and composite schedule, compound/MM/last-coupon
routing, three price bases, bounded yield inversion, six independently
admitted risk views and four exact cash recipes. All work remains local; no
push, PR, publication or release occurred.

## Implementation

Eighteen new public types bring the catalog to **144**; 33 `CanadianError`
and five `CanadianYieldError` variants bring the domain catalog to **237**.
The external thread-safe value assertions equal the complete public
declaration set, and the root error catalog test samples every variant once
with its stable kind, Display text and immediate typed source. Two root
wrappers forward kind, Display and source and convert with `?`; they are not
counted again as domain variants.

| Module | Lines | Responsibility |
|---|---:|---|
| `src/bonds/canadian.rs` | 1,915 | Enums, typed price, schedule, bond, pricing plan and both error types |
| `src/bonds/canadian/source.rs` | 535 | Canonical exact valuation/payable sources and complete civil geometry |
| `src/bonds/canadian/wide.rs` | 1,031 | Borrowed-slice5- to243-word integer engine and streaming IEEE rounding |
| `src/bonds/canadian/numeric.rs` | 1,585 | Quote/guard/factor/PV stages, retained risk, bases, residual and Brent |
| `src/bonds/canadian/cash.rs` | 1,244 | Four cash factories, owned snapshots, source views and the price frame |

Schedules own the supplied quasi and coupon vectors without cloning. The only
production allocations are the plan's bundle vector and the numerical
present-value vector, each a single `try_reserve_exact` after every
structural, quote and target gate that precedes it (FR-CAD-11). Cash uses
fixed ten-by-eight-word source and24-by-forty-word price frames. `CanadianRisk`
retains three243-word sums (5,968 bytes native) and its Debug omits them
(FR-CAD-52). The root `Error` grows from160 to176 bytes on native macOS
arm64; `CanadianYieldError` is160 and `CanadianError`112 bytes.

## Independent references and their actual consumers

| Route | Frozen reference | Actual consumer | Executed result |
|---|---|---|---|
| S | 79,794 source/date/cash rows, 74 columns | `tests/bonds_canadian_source_cash_oracles.rs`, one test | All 79,758 public operations and 36 documentary rows; every one of 43 output cells compared; 2,421 financial and 321 schedule definitions; all pinned input bytes and censuses |
| N | 3,546 prescribed numerical rows | `validation/generator/canadian_numeric_kernel_check.py` with [the harness](../validation/harness/canadian_numeric_kernel.rs) | 2,327 primitive rows over 17 operations and 1,219 dated rows (3,388 flow stages, 603 seeded exp observations), debug and release |
| P | 881 ideal price/six-risk inputs | `tests/bonds_canadian_ideal_oracles.rs` | 7,890 informative values inside frozen conditional intervals and 39 classified underflows; 648 ordinary rows over 216 regimes |
| Y | 349 target-driven inverses | same file | 345 informative roots, three ideal nonexistence controls and one observed-image protocol row |
| HS | 14 blind source/cash hands | `tests/bonds_canadian_source_cash_hand.rs` | 14 pass |
| HN | 8 blind numerical hands | `tests/bonds_canadian_numeric_hand.rs` | 8 pass; byte-identical to the frozen body |
| A | 7 API hands and 92 negative obligations | `tests/bonds_canadian_api_hand.rs`, `validation/generator/canadian_api_negative_check.py` | 7 pass; all 92 external probes fail with E0277 alone |
| R | Implementation-specific checks | Private unit tests, mutants, native/coverage/package runs below | See the R table |

These are reference inputs, not Cargo test-function counts. FR-CAD-59 adds
1,296 derived nonzero roundtrips (432 ordinary native quotes in all three
bases) as consistency checks, not new independent financial references.
The frozen [binding ledger](M8_CANADIAN_TEST_BINDINGS.md) remains the
authoritative per-family route map; this record states which actual consumer
now executes each route.

Two integration copies differ from their frozen originals only in
compilation plumbing. The source/cash copy replaces `use sabiquant::*` with
explicit imports and drops one `#[expect(clippy::panic)]` on a test function,
which `clippy.toml`'s `allow-panic-in-tests` leaves unfulfillable. The API copy
elides one helper lifetime that Clippy flags as needless. Both update their
header comment; every input and assertion is unchanged, and the frozen
originals under `validation/hand/` are untouched.

Six postproduction public test functions supplement the frozen hands; none
is a blind hand. `tests/bonds_canadian_solver_boundaries.rs` binds the2^970
residual-scale switch (see the mutation result below).
`tests/bonds_canadian_public_supplements.rs` adds five functions authored
after the first coverage run: interior and short-last period kinds, interior
and short-last coupon cash with exact expected amounts and source geometry,
plan policy and risk-report price bridges, typed out-of-range ordinals, and
the first failing original ordinal for factor overflow, present-value
overflow and factor underflow in both `evaluate` and `risk`. Each expected
value or ordinal follows from independent exact arithmetic stated in the test.

## R obligations

| Requirement | Actual evidence |
|---|---|
| FR-CAD-11 | Code review: schedule/bond construction allocates nothing; the plan's bundle vector and each PV vector are single fallible reservations after all preceding gates; S binds cap0/1/2/20,000/20,001 precedence |
| FR-CAD-26 | Private source tests bind the late-184 drop and completed181/184-day periods; S and HS bind every admitted frequency |
| FR-CAD-32 | Private canonicalization tests bind the160/148-bit bounds with consecutive-Fibonacci coprime controls and the zero-denominator refusal |
| FR-CAD-36/37/40/49 | The kernel replay drives actual private stages with bounded source/factor arguments; native elementary rows on this host gave35 finite nonzero, two infinite and six zero observations, each checked against its predeclared allowance or actual class |
| FR-CAD-51 | Nine private wide tests bind capacity, carry, streaming rounding and division at every supported width; the kernel replay binds actual retained sums |
| FR-CAD-52 | Debug omits retained buffers; measured native layouts above |
| FR-CAD-59 | The1,296 public roundtrips above |
| FR-CAD-71 | The root catalog test covers all 237 variants; invariant-excluded capacity paths stay in the coverage denominator |
| FR-CAD-72 | Native, coverage, package, foreign-compilation and mutation evidence below |

## Current execution evidence

| Evidence | Result |
|---|---|
| Native whole tree | PASS: debug971 (143 unit,785 integration,43 doctests), release970 (142 unit); 110 result groups, zero failures or ignored tests |
| Strict checks | PASS: all-target Clippy `-D warnings`, rustfmt, warning-denied public and private rustdoc, `git diff --check` |
| Foreign Linux GNU/Windows MSVC | PASS compilation only: all-target `cargo check -D warnings` for x86_64 Linux GNU and Windows MSVC; no foreign runtime claimed |
| Numerical kernel replay | PASS debug and release: 2 harness tests, all3,546 rows; copied numeric module is the exact original prefix plus the cfg(test) harness |
| Negative API probes | PASS: 92/92 separate external compilations fail with coded E0277 alone, naming the intended type and trait |
| Semantic mutation | PASS: 22/22 compiled semantic mutants killed by runtime assertions (21 tier 1, 1 tier 2) |
| Production-region coverage | 23,819/25,430 regions (93.664963%) from928 unit/integration functions in108 binaries; Canadian3,994/4,318 (92.50%), every remaining Canadian region classified |
| Extracted std-only package | PASS: actual archive of62 members,60 byte-identical payloads, extracted143 units,43 doctests, strict Clippy/docs and the external8-test API suite; `dependencies=[]`, `features={}` |

The [mutation runner](../validation/generator/canadian_semantic_mutants.py)
restores the four production modules byte-for-byte before each case. Tier 1
(debug) runs the29 hands, the residual-scale supplement, the20 private
Canadian unit tests and the full numerical kernel replay; only a tier-1
survivor proceeds to tier 2 (release), the complete S and P/Y consumers. The
unmutated baseline passes both tiers. The22
mistakes span the parent (last-coupon MM boundary, settlement-day coupon,
compound quote accrued, valuation/payable coupon swap, par on the contractual
date, monthly settlement exception), source (late-period payable rule,366-day
basis, dropped second fragment, omitted par), numerics (clean bridge and
clean target accrued, MM360-day basis, frequency-free logarithm scale,
2^969 residual threshold, widened tiny branch, residual sign) and cash
(valuation instead of settlement accrued, valuation instead of payable coupon,
ties-away, ceiling sign, double-counted component accrued). Only
`coupon_cash_valuation_coupon` survives tier 1; the S corpus kills it. These
are selected concrete faults, not a proof of completeness for all72 families.

Instrumented coverage excludes doctests and embedded test modules from the
production denominator. By module, Canadian covers873/899 parent,
1,029/1,142 cash,1,267/1,418 numerical,274/285 source and551/574 wide
regions. All324 remaining Canadian regions are classified, and none is
removed from the denominator:

| Class | Regions | Meaning |
|---:|---:|---|
| Bounded-arithmetic capacity defenses | 256 | Checked wide operations, canonical source bounds, rounding/normalization and admitted amount construction, plus their propagation; the reviewed resource budgets keep admitted inputs inside these limits |
| Unreachable on admitted states | 30 | Positive dirty and target exclude the opposite-sign residual, zero-dirty and zero-sum getters; finite logarithms and exponents; accrued interest far below an ulp at `f64::MAX` excludes bridge/target-shift overflow; one dead Exact rounding arm, one impossible fragment order and equal-endpoint admission after distinct quantized magnitudes |
| Guard boundary internals | 14 | The closed exponent-guard rejection band and its construction; the private kernel replay executes all249 guard and12 guard-box rows |
| Allocation failure | 13 | The plan bundle and the three present-value reservations |
| Already-validated propagation | 8 | Private index/source helpers after the public ordinal check, nonempty bundles and clamped civil dates |
| Valid extreme paths not forced | 3 | An overflowing dirty-price reduction of finite present values and a factor/PV failure inside an invoked yield callback; the same stage failures are bound through `evaluate` and `risk` |

No adjusted, branch or exhaustive-input percentage is claimed, and the
private kernel replay is not merged into the native percentage.

| Final evidence | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-canadian-native-verified-v2/evidence.json` | `470935c896d5492c692c7e535cc99272ce863fa3da0502a610ed4ed3c2092c8a` |
| `/private/tmp/sabiquant-canadian-coverage-v2/production-regions.json` | `da271a6ad40301aad660ae3c465f2bf1742d87891d0895ea075e7df22123d0d0` |
| `/private/tmp/sabiquant-canadian-coverage-v2-review/classification.json` | `f151fbca889abc31781f61fdce5a7ff22621c27d943d20340d528f078f27b3c3` |
| `/private/tmp/sabiquant-canadian-numeric-kernel-final-v1/evidence.json` | `a622bcde96627b8e93ca868ebc13c27c60325ed8aeb60093b91f5d3b2b0a5f2e` |
| `/private/tmp/sabiquant-canadian-api-negative-final-v1/evidence.json` | `f9c7f86363ac5b97e16509ed35676a38b50c3fc96e04ac6d41477a85f3786012` |
| `/private/tmp/sabiquant-claude-canadian-mutants-v3/evidence.json` | `f3bfe3f0522abb444bdf31e457c95a85ebfc660b8423c7ea13d1f82dd3ec158a` |
| `/private/tmp/sabiquant-canadian-package-v1/evidence.json` | `618a5221ed3cddca83016f5bb6c93aa6989ad18157a0bfb278eb5036cbc56d72` |

The native, kernel, mutation and package runners guard their original
inputs before and after execution; these establish equality at those checks,
not continuous filesystem monitoring. The kernel, negative-probe, mutation
and package evidence predates only `tests/bonds_canadian_public_supplements.rs`,
which none of them consumes and which is not packaged; the mutation run
includes the residual-scale control, and the native and coverage runs include
both supplement files.

## Corrections and retained failures

Integration found and fixed defects before this checkpoint; none changed a
frozen expectation:

- A real numerical defect: a positive tiny `g` with exact `g·λ/f` underflow
  produced `+0` instead of IEEE `-0` before the zero identity. Two private
  regressions now bind the signed underflow and the ceased-ordinal factor
  observation before the zero-PV shortcut.
- The source-cash doc comment `[S,S]` was an unresolved intra-doc link under
  warning-denied rustdoc; it is now code-formatted.
- The first whole-tree Clippy run failed on the two integration-copy lint
  issues described above.
- The first mutation catalogue run left `residual_threshold_969` alive: no
  hand, private test, kernel row or corpus distinguished a2^969 from the
  adopted2^970 residual-scaling threshold. The postproduction public control
  `tests/bonds_canadian_solver_boundaries.rs` bisects public evaluations of a
  forty-period zero-coupon bond into each binade and requires scale0 then1
  with an exact lower-endpoint root. It is not a frozen blind hand.
- The first coverage run showed valid public paths without any executing
  test: interior/last period-kind and coupon-geometry inspectors, plan source
  policy, the risk report's price bridges and failure propagation inside
  `evaluate`/`risk`. The five supplement functions above now execute them.
  An interrupted second native/coverage attempt, stopped to add those tests,
  was discarded and is not reported as evidence.
- Earlier runner failures (a misplaced inner doc comment in the appended
  harness, `--nocapture` output interleaving, missing included documents in
  the first mutation crate, and a private harness's nonexistent Date overload)
  were tooling errors. Their logs remain; none is counted as a pass or a kill.

## Acceptance limits

Prices, yields and risks are supplied-observation results with the adopted
conditional accuracy; they are not ideal-price enclosures. The source version
records the retained local interpretation; it does not certify current
market practice, calendars, ownership or settlement enclosures. Native
elementary allowances are qualified to this host. Linux and Windows checks
are compilation only. Financial/owner, foreign native runtime,
maintained-calendar/legal, analytical-enclosure, release and wider M8
acceptance remain open.
