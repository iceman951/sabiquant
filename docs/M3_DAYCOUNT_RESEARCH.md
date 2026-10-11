# M3 day-count source review

This records the pre-implementation research snapshot. Subsequent delegated
decisions and implemented contracts are in [M3_LOCAL_READINESS](M3_LOCAL_READINESS.md)
and [M3_EXTENSION_VALIDATION](M3_EXTENSION_VALIDATION.md).

| Field | Value |
|---|---|
| Reviewed | 2026-10-11 |
| Status | Research and implementation proposals; **not M3 acceptance or readiness** |
| Scope | FS-DCC; OQ-14, OQ-15, OQ-16, OQ-20, OQ-21; QR-01 through QR-04 |
| Method | Primary publications and publishers' own documentation; no implementation source translated |

The smallest defensible next implementation slice is **Actual/360,
Actual/365 (Fixed), Actual/Actual (ISDA), and 30E/360 (Eurobond Basis)**.
Their forward-period definitions were verified directly. ICMA has a usable
primary definition but needs an explicit-input contract. US February rules
and the original AFB February roll remain research items. This review does
not pass the other M3 gates or broaden compatibility claims.

The [2006 ISDA Definitions](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=23)
were accessible as an original ISDA publication hosted by TD Securities.
The relevant locations are §4.13, printed page 10, and §4.16, printed pages
11–13, PDF pages 23–25. This is primary document content on a third-party
host; its authenticity/current amendment completeness was not independently
attested. No licensed text is reproduced here. Existing statements that the
2006 definitions were never consulted describe earlier work, not this review.

## Evidence and proposed scope

Here `DC` is a convention's integer day count, `N = E − S` is the actual
calendar-day count, and `[S,E)` includes the start and excludes the end.
“Source verified” describes evidence, not an implemented feature.

| Convention | Verified rule and primary evidence | Proposal / remaining issue |
|---|---|---|
| Actual/360 | `DC=N`, `YF=N/360`; ISDA §4.16(e), with calculation-period endpoints in §4.13. [Primary definition](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=23). | Source verified; first slice. Denominator never changes for leap years. |
| Actual/365 (Fixed) | `DC=N`, `YF=N/365`; ISDA §4.16(d). [Primary definition](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=23). | Source verified; first slice. Keep “Fixed” explicit in identifiers. |
| Actual/Actual (ISDA) | Sum actual days in common years over 365 and in leap years over 366; §4.16(b). [Primary definition](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=23). | Source verified; first slice. QR-01's older worked-example discrepancy is still open. |
| 30E/360, Eurobond Basis | Cap each day-of-month at 30. `DC=360ΔY+30ΔM+ΔD′`; `YF=DC/360`; §4.16(g). [Primary definition](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=24). | Source verified; first slice. February is not automatically changed to 30. |
| Bond Basis; 30E/360 (ISDA) | Distinct definitions exist in §4.16(f) and (h); the latter has a February termination-date exception. [Primary definitions](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=24). | Later slice. Require explicit termination input for ISDA; resolve signed semantics before Bond Basis. |
| Actual/Actual (ICMA) | For each determination period, divide accrued overlap days by `F × determination-period days`, then sum. Appendix A5 §1.3 defines the boundaries and permits extending the two-period wording. [ICMA, March 2022, A5-2–A5-3](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf#page=2). | Source verified for explicit boundaries. OQ-21: no frequency or quasi-coupon inference. Schedule generation stays M7. |
| Actual/Actual (AFB) | The French primary document specifies actual days, 366 when February 29 is included, and backward whole-year decomposition for periods over one year. [FBF-hosted 2004 addendum, §4(7)(i), printed p.6](https://www.fbf.fr/uploads/2021/02/Convention_cadre_FBE_additif_operation_taux_interet_2004.pdf#page=6). | **Under Research.** This text does not specify the leap-year February 28→29 roll. OQ-16/QR-04 remain open. |
| 30/360 (US) | FINRA gives 360-year/30-month settlement examples, but does not supply the disputed February/EOM switch. [FINRA Rule 11620(b)](https://www.finra.org/rules-guidance/rulebooks/finra-rules/11620). | **Under Research.** These examples do not establish SIFMA's modern full algorithm. OQ-15/QR-02 remain open. |
| Business/252 | B3 explicitly identifies the business-day/252 basis. Its DI methodology uses an inclusive start and exclusive end. [B3 basis table](https://www.b3.com.br/en_us/products-and-services/trading/interest-rates/di-training-program/02-interest-rates.htm), [B3 accumulated DI methodology](https://www.b3.com.br/pt_br/market-data-e-indices/indices/indices-de-segmentos-e-setoriais/di/metodologia-de-calcudo-acumulado-de-di/). | Source-backed candidate after calendar validation. Calendar is a required input; no default Brazil calendar. Market-specific holiday rules need separate evidence. |

## Decisions to record before implementation

**OQ-14 — signed inputs.** Recommend calculating the forward interval and
negating for reversed dates, consistently for integer counts and fractions.
This is a SabiQuant API decision, not a sourced claim about market contracts.
For Bond Basis it deliberately differs from direct reversed formula
application; retain the 2023-01-15→2023-03-31 counterexample, 76 forward
days versus −75 under that direct reverse formula. Equal dates must return
positive zero. Do not assume additivity for every 30/360 variant.

**OQ-20 — staged scope.** Keep H1's existing P0/P1 inventory, but stage the
four verified cases first. A feature's priority does not close its research
gap. Record any local implementation exception separately from M3 acceptance.

**OQ-21 — ICMA contract.** Require ordered, positive-length determination
periods covering the accrual interval without gaps or overlaps, and an explicit
positive annual determination count. These validation requirements are proposed
API safeguards. Neither dates nor frequency should be guessed from an accrual
span. A payment date adjustment must not silently alter determination boundaries:
ICMA explicitly notes this convention was not designed around adjusted interest
periods. [ICMA A5 §1.1–1.3](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf#page=2).

**OQ-15/OQ-16 — obtain exact texts.** SIFMA's accessible callable-security
appendix refers readers to *Standard Securities Calculation Methods*, Vol.1,
pp.17–23; it does not restate the required EOM algorithm.
[SIFMA appendix, p.7](https://www.sifma.org/wp-content/uploads/2024/03/Government_Sponsored_Enterprises_GSE-Practice_Guidelines_Appendix_A-European_Callable_Securities_Formula.pdf#page=7).
For AFB, ICE's own A010 reporting definition explicitly includes the February
28→29 backward roll, supporting its use in that reporting context.
[ICE field definitions, PDF p.8](https://www.theice.com/publicdocs/data/ICE_Trade_Vault_SEC_SBSDR_Field_List_Validations.pdf#page=8).
It is not the original 1994 AFB text or the ISDA 1999 clarification. The
[official ISDA 1999 paper](https://www.isda.org/a/pIJEE/The-Actual-Actual-Day-Count-Fraction-1999.pdf)
and [1998 memo](https://www.isda.org/a/AIJEE/1998-ISDA-memo-EMU-and-Market-Conventions-Recent-Developments.pdf)
were located, but their scans could not be read through this session's tools.
Do not mark their ambiguities resolved.

## Tests that distinguish the choices

Expected values below are independently derived from the verified formulas,
except explicitly labelled candidate comparisons; they are not new fixtures.

| Scenario | Required check |
|---|---|
| 2024-01-01→2025-01-01 | Actual/360 `366/360`; Fixed `366/365`; ISDA `1`; equal/reversed variants. |
| 1999-07-01→2000-07-01 | ISDA `184/365 + 182/366`; splitting at January 1 preserves the mathematical fraction. |
| 2003-07-15→2004-01-15 | Formula-derived ISDA `170/365 + 14/366`; separately retain QR-01 until the older primary example is inspected. |
| 9999-01-01→9999-12-31; MIN→MAX | No construction of year 10000 during ISDA segmentation; finite result and exact endpoint handling. |
| 2024-02-29→2024-03-31; 2023-01-31→2023-02-28 | European `31/360` and `28/360`; distinguish February-adjusting variants. |
| 2024-02-28→2025-02-28 | AFB candidate `1` versus QuantLib/clarified backward-roll candidate `1+1/365`; do not choose by majority agreement. |
| Last February→March 31; last February→last February | US with proposed EOM flag on/off; cover common, leap, and century years before fixing expected values. |
| ICMA regular, short first, long first | Full reference period gives `1/F`; explicit two-period overlap arithmetic; reject missing coverage, duplicates, gaps, zero lengths, and invalid frequency. |
| Business/252 Monday→Monday | Weekends-only `5/252`; add a weekday holiday, holiday endpoints, equal dates, reversed dates, and empty/pathological calendars. |

For the first slice, use integer/rational oracles, Gregorian-cycle date windows,
all month-end pairs, deterministic wide-range pairs, invalid identifiers,
typed error contexts, and qualified QuantLib differential results. A matching
reference output is evidence for that case, not a replacement for the primary
definition or the unresolved decisions above.
