# Quoted CDS maturity implementation review

Completed 2026-10-15, Asia/Bangkok. Read-only review found no correction
required in the inspected quoted-maturity implementation against the frozen
[CDS contract](M7_CDS_CONTRACT.md). This conclusion covers the resolver and
its immediate value/error wiring, not the Following date-plan implementation
or aggregate validation.

## Scope and independence

The reviewer, `/root/cds_legacy_sources`, authored the independent Python
generator, fixture and Rust consumer before inspecting production CDS
maturity code. Their construction and earlier focused terminal checks are
recorded in [the oracle report](M7_CDS_MATURITY_ORACLES.md). This subsequent
source review does not mean the reviewer remains blind to production code.
No oracle, fixture, test, contract or production file was changed in this
review, and no additional Cargo command was run.

The review read all of
[`cds_maturity.rs`](../src/schedule/cds_maturity.rs), the relevant exports,
error variants/kinds/source wiring in [`schedule.rs`](../src/schedule.rs),
and the existing `Date::checked_add_months` and `Period::new` operations.
The [maturity design](M7_CDS_MATURITY_DESIGN.md) supplies the mathematical
definition, primary-source boundaries and their limits. This review compared
code to that recorded evidence; it did not retrieve new financial sources or
inspect QuantLib implementation code.

| Reviewed artifact | SHA-256 |
|---|---|
| `src/schedule/cds_maturity.rs`, 7,384 bytes | `e721e12dde2bbcb4e57989e3a191621d65c4862379ebd221ca66a102411dd28a` |
| Frozen `docs/M7_CDS_CONTRACT.md`, 10,950 bytes | `312df1e25a22c2f2575c30fd30853d315258ac3844ac0ab386d42a1db9c5aad5` |

The generator, fixture and consumer SHA-256 values were rechecked and still
match the oracle report. This records temporal independence and unchanged
artifacts; it is not a fresh fixture execution or a new runtime attestation.

## Findings

| Area | Review result |
|---|---|
| Stored tenor domain | Original length/unit is checked first. Nonnegative Years and quarter-multiple Months are accepted; negative lengths, Days/Weeks including zero, and other Months are rejected before any date or matured shortcut. |
| Definition identity | Private length/unit fields participate directly in derived Eq/Hash. `quoted_tenor()` reconstructs the original stored pair, so 1Y/12M and 0Y/0M remain distinct definitions with equivalent outcomes. Trade, explicit convention and outcome are retained. |
| Closed convention API | Both canonical names, canonical `ALL` order, Display and strict identifier parsing agree with the contract; there are no aliases, default regime or inferred transition date. |
| Quarter and semiannual selection | Euclidean remainder selects the preceding eligible twentieth inclusively, then adds three months. This gives the first strictly future quarter for Quarterly2009 and the required June/December base for Semiannual2015. |
| Matured outcome | Zero is tested against the abstract base before constructing a Date. The shared predicate cannot return Matured for Quarterly2009 because its base is strictly future. Positive accepted quotes are at least three months and always resolve strictly after trade. |
| Required date and cause | Only the final target month is constructed from the original trade. Out-of-range errors retain that trade, stored quote, explicit regime and actual `add months` displacement. Stable error kinds and `StdError::source()` preserve the Date cause. |
| Scope of public documentation | The API describes unadjusted analytical maturity, explicit conventions and proleptic extension. It does not infer protection, settlement, a payment calendar or premium accrual. |

At March/September 20, inclusive anchor selection moves both regimes to the
following June/December base. At June/December 20, Quarterly2009 moves to the
next quarter while Semiannual2015 retains its base and zero becomes Matured.
The corresponding day-19 cases retain the preceding selection. These are
the equality distinctions in the adopted primary-source-backed definition.

Near the lower civil bound, January 0001 has month coordinate zero and the
semiannual base is coordinate −1, abstract December 0000. Zero returns
Matured; 3M produces March 0001 without constructing year zero. At the upper
bound, December 9999 on/after the twentieth gives Matured for semiannual
zero but a required-final-date error for quarterly zero; the latter is not
hidden by the shared matured predicate.

Arithmetic is bounded: trade-month coordinates are 0–119,987, selected bases
are −1–119,990, and the largest accepted Years value converts to
25,769,803,764 months. The largest possible target is 25,769,923,754, safely
within i64. Negative i32 values are rejected without absolute-value or
negation overflow. `i32::MAX` Months fails the domain; 2,147,483,646 Months
and `i32::MAX` Years reach final-date range checking. Temporary day clamping
inside `checked_add_months` preserves the target year/month, and subsequent
day 20 is always valid.

## Limits

This is a code-level conformance review of the stated analytical contract.
The earlier independent oracle results retain their original scope; this
document makes no aggregate regression, coverage, package or platform-pass
claim. It does not close financial/owner acceptance, OldCDS's unresolved
automatic first-coupon boundary and start role, NI-01 settlement, or full M7.
