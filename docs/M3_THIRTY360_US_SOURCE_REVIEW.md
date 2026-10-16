# 30/360 US: February and EOM source review

| Field | Value |
|---|---|
| Reviewed | 2026-10-16, Asia/Bangkok |
| Scope | FS-DCC; OQ-15; QR-02; [REQUIREMENTS §9.7](REQUIREMENTS.md#97-the-30360-family) |
| Status | Research and proposed local contract; not adopted financial semantics or M3 acceptance |
| Method | Primary publications, publisher pages, regulator rules; no QuantLib implementation source |

The retrieved primary reporting specification supports **two explicit EOM
settings**. It does not support silently applying the February adjustments
to every security. The exact SIA/SIFMA book algorithm remains uninspected;
the available author sample omits its day-count chapter. An implementation
can proceed under an expressly adopted local contract while retaining that
financial-source acceptance gap.

## Direct primary evidence

### SSCM author and publisher

[The author's fourth-edition Volume 1 description](https://www.sscmfi.com/index.php?task=G-V1)
states that the 30/360 methodology was changed to account for EOM coupon
cycles. This is direct publisher evidence of a qualification, but gives no
complete adjustment algorithm or equivalence to an earlier edition.

The [public samples page](https://www.sscmfi.com/index.php?task=G-S) offers
read-only sample forms. Submitting its `task=Volume1-P` form exposed the
[public Volume 1 PDF sample](https://www.sscmfi.com/files/SSCM%20Volume%201%20-%20Sample.pdf).
Its cover identifies Jan Mayle, fourth edition, revision
`0113-1323-16W1`; the copyright page says 2021. The 16 PDF pages were read
using macOS PDFKit text extraction. The TOC places Day Count Calculations
at printed page 11. The sample then selects other component, price and
benchmark pages: it does **not** expose that chapter. The sample does not
close the exact SIA/SIFMA February-rule gap.

The publisher's web sample was also followed through the public
`task=Volume1-W` form to its `V1/index.html` viewer. Its landing HTML alone
provided no readable day-count definition. No subscription was purchased,
restricted chapter requested, or account accessed.

### ROC reporting specification: exact EOM-qualified algorithm

[ROC Revised CDE Technical Guidance v4, December 2025](https://www.leiroc.org/publications/gls/Final_ROC202531-Revised-CDE-Technical-Guidance-Version-4.pdf#page=157),
§3.4 Table 4, A001, **FIX/FIXML definition column**, PDF page 157 /
printed **166**, supplies four numbered adjustments (`F` = February month
end; `eom` = security's EOM qualification):

1. `eom && F(S) && F(E)`: `D2 ← 30`.
2. `eom && F(S)`: `D1 ← 30`.
3. `D2 == 31 && D1 in {30,31}`: `D2 ← 30`.
4. `D1 == 31`: `D1 ← 30`.

FIX value `1` is named `ThirtyThreeSixtyUS`; footnote 56 credits FIX.
Adjacent ISO 20022/FpML columns treat February differently. This is a
dated primary reporting snapshot, not the SIFMA book. Web text and a
local PDFKit render independently confirmed the column and page.

[CPMI-IOSCO April 2018 technical guidance](https://www.bis.org/publications/harmonisation-critical-otc-derivatives-data-elements-other-uti-and-upi-technical-guidance.pdf#page=130),
Table 4, A001, PDF page 130 / printed page 128, already reproduces the
same four numbered rules in its FIX column. Its §1.2 describes guidance
to authorities about reporting data, rather than direct participant rules.
The report itself directs readers to ROC for updates. It corroborates the
historical wording; use the dated ROC snapshot for this proposed contract.

### Current official FIX reference has changed

The [official FIXimate root](https://fiximate.fixtrading.org/) now redirects
to the FIX Protocol Ltd repository on Orchimate. The retrieved
[CouponDayCountCodeSet](https://orchimate.org/fixtrading/fix-latest/codeSets/CouponDayCountCodeSet)
identifies field 1950, value 1, `ThirtyThreeSixtyUS`, updated EP266, and
cross-refers to ISO 15022 A001. It does **not** print the four-rule
algorithm above. Its page reported EP309 while the root reported EP312;
the available cached views therefore do not establish one current version.
The old `/latestEP/` URL cited by ROC returned HTTP 404. Do not cite the
current FIX page as though it still contained the historical algorithm.

### FINRA and MSRB: precise but different scope

[FINRA Rule 11620(b)](https://www.finra.org/rules-guidance/rulebooks/finra-rules/11620)
sets a 360-day year and gives 30-month settlement-interest examples,
including February 23 to March 3 = 10 days. It gives no explicit
February-month-end/EOM switch. This is not sufficient evidence for the
entire proposed US algorithm. Its settlement timing and rounding rules
are outside the day-count function.

[MSRB Rule G-33(a), (e)(i)–(ii)](https://www.msrb.org/Rules-and-Interpretations/MSRB-Rules/General/Rule-G-33)
provides the `360ΔY + 30ΔM + ΔD` numerator, the two conditional
31-to-30 adjustments, and includes the beginning day while excluding the
ending day. It supplies no February EOM adjustment in that rule. Securities
paying only at redemption may use the issuer's chosen basis. It supports
the off-setting arithmetic in its stated municipal-rule scope, not a
universal equivalence between regulatory and market names.

The [SIFMA mortgage-security formulas](https://www.sifma.org/wp-content/uploads/2017/08/chsf.pdf),
§E.1, are a separate convention: the search-indexed primary text changes a
February month-end start to 30 but does not supply the both-February
end adjustment and clamps the numerator to zero. This source was located
by primary search indexing, not directly rendered in this review. Do not
substitute its mortgage scope for the securities convention under review.

## Proposed API and adoption boundary

A `Thirty360Us` context with a **required explicit EOM boolean** is a
reasonable local API name. The canonical display name can remain
`30/360 (US)` if its documentation links the adopted contract and explains
the flag. No source found makes that label a globally unique algorithm.
Do not add an implicit EOM choice to stateless `DayCount`, a default
constructor, or a text parser. Do not infer the flag from endpoint dates,
payment adjustments, or an EOM schedule. A security's contractual cycle
is information that two dates cannot establish.

The contract should separately record these SabiQuant decisions:

- Work on ordered endpoints, then negate for reversed input; equal input
  returns positive zero. These signed/equal rules extend the published
  forward-period formulas and follow the existing local OQ-14 policy.
- Preserve each original February-month-end predicate before replacing
  day numbers. Then apply numbered adjustments in order, using the
  modified start day in step 3.
- Compute `360ΔY + 30ΔM + (D2′−D1′)` and divide by 360. The off setting
  yields existing Bond Basis arithmetic, while the on setting can differ.
  Keep the context distinct in equality and hashing even for intervals
  producing the same result.
- The EOM flag does not mean every month-end end date becomes 30:
  January 31 to February month end retains the actual February end day.
  No maturity exception, business calendar, successor-date construction,
  or mortgage-specific zero clamp follows from the proposed four rules.

Implementing both settings under this explicit local contract advances
OQ-15 without claiming full SIFMA inspection or financial acceptance.
Retain a named acceptance item for checking the exact relevant SIA/SIFMA
edition and any errata against the frozen contract. The primary reporting
snapshot is adequate evidence for a reproducible engineering definition;
it is not evidence that every instrument labelled US, A001, SIA, NASD,
or Bond Basis uses it.

## Independently calculated discriminating examples

These are arithmetic consequences of the **proposal**, not copied source
benchmarks or financial acceptance. `on`/`off` refer only to the explicit
EOM input; counts are synthetic 30/360 days.

| Ascending interval | on | off | Distinction |
|---|---:|---:|---|
| 2023-02-28 → 2023-03-31 | 30 | 33 | February start replacement must precede the end-31 condition |
| 2024-02-29 → 2024-03-31 | 30 | 32 | Leap-year February month end |
| 2023-02-28 → 2024-02-29 | 360 | 361 | Both original endpoints are February month ends |
| 2024-02-29 → 2025-02-28 | 360 | 359 | Leap to common-year endpoint change |
| 2023-01-31 → 2023-02-28 | 28 | 28 | February end alone does not trigger replacement |
| 2024-02-28 → 2024-03-31 | 33 | 33 | February 28 is not month end in a leap year |
| 2023-08-31 → 2024-02-29 | 179 | 179 | Other-month start does not qualify the February end |
| 2023-01-29 → 2023-01-31 | 2 | 2 | Unqualified end 31 remains 31 |
| 2023-01-30 → 2023-01-31 | 0 | 0 | Distinct civil dates may have zero synthetic days |

Tests should include both flags over month-end pairs across a full
Gregorian cycle, common/leap/century transitions, original-predicate rule
order, equal and reversed inputs, MIN/MAX without out-of-range dates,
context traits, rate/cashflow composition, and qualified behavioral
reference comparisons. QuantLib agreement cannot decide the security's
EOM input. No QuantLib source or implementation was consulted here.

## Temporary retrieval evidence

The downloads are local research evidence, not repository fixtures or
redistributable book content. SHA-256 values identify the bytes read:

| Public artifact | Local filename under `/private/tmp/` | SHA-256 |
|---|---|---|
| SSCM public Volume 1 sample | `sabiquant-src-us-sscm-v1-sample.pdf` | `cce1ab543338ab8dd196d521f099015412c1ac51654f016ba38d530368c45be3` |
| ROC CDE v4 | `sabiquant-src-us-roc-cde-v4.pdf` | `b19a5d99cb202da7d1d92d878bc8358b5e10840df29654e83c8f575edf5f28f1` |

The A001 visual check is `sabiquant-src-us-roc-cde-v4-a001.png`, rendered
from zero-based page 156. No licensed source text is reproduced beyond
brief paraphrases needed to identify the proposed convention.
