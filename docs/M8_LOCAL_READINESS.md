# Local M8 readiness and implementation record

Recorded 2026-10-16 from verified local commit `4da3f43`. The owner requested
continuous local progress with broad meaningful tests and explicitly retained
local-only work. That authority permits the staged decisions and implementation
below; it does not grant formal milestone, owner API, financial, native-platform
or publication acceptance. No remote action is proposed.

## Complete objective and current evidence

M8 retains the entire [roadmap](ROADMAP.md) scope: zero-coupon and fixed-rate
bonds; explicit settlement and coupon entitlement/ex-coupon; clean and dirty
price; accrued interest; yield inversion; duration and convexity. NI-01 exact
settlement amounts are a related numeric obligation, not a synonym for a
settlement date. Completing an arithmetic foundation or generic price kernel
alone must not replace the remaining M8 work or claim its acceptance.

The prerequisite local M3–M7 engineering tranche at `4da3f43` passed 619 debug
and 618 release tests, strict checks and fresh package/coverage verification
([evidence](M3_US_AFB_VALIDATION.md)). Native Linux/Windows runtime, TARGET
maintenance, full SIFMA and historical OldCDS financial gates remain separate.
A local staging exception follows the same continuing-authority approach used
for M6/M7; no external approval or CI result is invented.

| Work stream | Evidence/current state |
|---|---|
| Conventional gilt primary formulas | [DMO source review](M8_GILT_SOURCE_REVIEW.md): directly downloaded fourth-edition PDF; relevant complete formula/accrued pages visually inspected; explicit source boundaries and amount roles |
| Treasury primary formulas | [Treasury review](M8_US_TREASURY_SOURCE_REVIEW.md): current eCFR and historical Treasury PDF; published examples, rounding stages, auction scope and a source presentation inconsistency |
| Mathematical risk/yield definitions | [Historical ISMA review](M8_ICMA_SOURCE_REVIEW.md): rotated scanned book inspected visually; derivative definitions and independently diagnosed rounded-source example |
| Public reference feasibility | [Pre-contract probes](M8_BOND_REFERENCE_DESIGN.md): 48 attested isolated cases, raw public errors/nonfinite results preserved; no fixture or local financial contract inferred |
| Complete bond API/test proposal | [Bond design](M8_BOND_API_DESIGN.md), including distinct generic/named yield models, explicit date roles, prepared work, typed inversion and all required risk families |
| Exact amount design | [Amount design](M8_EXACT_AMOUNT_DESIGN.md): canonical decimal/quantum modes, full fused operation, independent bound proofs and future binary boundary |
| Phase A implementation evidence | [Oracle report](M8_AMOUNT_ORACLES.md), [validation](M8_AMOUNT_VALIDATION.md) and [implementation review](M8_AMOUNT_IMPLEMENTATION_REVIEW.md): 277,577 independent oracle rows and an 18-test blind hand suite pass; full native checks, coverage and package extraction recorded; owner/financial/platform gates open |
| Independent design challenge | [Review](M8_DESIGN_REVIEW.md): DMO symbol-role conflict, constant-price signed-flow degeneracy and resolved decimal-product API gap |

## Adopted immediate tranche: complete phase A exact decimal foundation

The agent adopts the explicit [amount contract](M8_AMOUNT_CONTRACT.md) under
continuing local authority. This records the NI-01 reconsideration triggered
by [ADR-0006](adr/ADR-0006-numeric-policy.md) without changing accepted H1
binary64 analytics or silently supplying a currency/quantum/rounding policy.
The contract includes canonical i128/scale0..38 values, exact ordering/text,
all seven explicit rounding modes, arbitrary positive decimal quantum,
checked exact arithmetic, cancellation-safe reduction and complete ratio/fused
two-decimal product/divisor quantization. Intermediate i128 overflow cannot
reject a representable final result; the eight-limb proof and signed limits
are part of the contract.

Implementation follows independent contract review and expectation generation.
The producer must not read oracle output to copy expectations into arithmetic.
At least one generator/hand suite author derives results before inspecting any
amount production implementation. All previous fixtures/generators remain
frozen. Required evidence includes exact integer/rational oracles, algebraic
ties, two-precision Decimal cross-checks with a justified precision, all scales,
resource/error ordering, MIN/MAX, fused rescue cases, threads, public/error
catalogs, full strict checks, fresh coverage and actual extracted package.

## Retained next stages and explicit open decisions

1. Phase B must define and implement all four distinct exact/rounded binary64
   conversions before a financial adapter accepts binary64 output as exact
   settlement input. Exact rounding of supplied bits is not a certificate for
   an ideal financial value near a quantum boundary.
2. Phase C must pin units, signs, quote normalization, denomination, quantum,
   mode and every rounding stage for its financial adapter. Current sources
   differ in their intermediate/settlement rounding. The library must not
   select a universal convention or claim every currency/market.
3. Adopt the complete bond core contract: owned zero/fixed terms, retained
   schedules/redemption, explicit settlement/payment inclusion/ex boundary,
   generic versus named accrued adjustment, prepared date work and source
   context. All stage outputs retain typed failures and original indices.
4. Price/yield models need explicit capability coverage. DMO fractional powers,
   Treasury fractional simple factors and generic direct-horizon rates are
   different models. A source-complete named adapter must separately verify
   dates, irregular periods, entitlement and examples. No maintained UK/US
   holiday calendar is inferred from WeekendsOnly.
5. Complete yield/risk work needs bounded source-preserving fallible callbacks,
   exact degeneracy qualifications for signed/cancelling flows, explicit yield
   coordinates, derivative order, scaled products/ratios and propagated
   numerical uncertainty. Round-trip consistency is insufficient evidence.
6. Freeze new reference fixtures only after the corresponding contract exists;
   retain every raw error, nonfinite reference observation, unsupported context,
   entitlement and rounding difference with an executable classification.

These are remaining work items, not a claim that all implementation is blocked.
They will be resolved and adopted locally in dependency order while the full
objective and independent financial/platform/owner/release gates remain visible.
