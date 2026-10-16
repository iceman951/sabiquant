# Actual/Actual AFB source review

| Field | Value |
|---|---|
| Reviewed | 2026-10-16, Asia/Bangkok |
| Scope | OQ-16 / QR-04; Actual/Actual AFB backward-year and leap-day rules |
| Status | Primary ISDA clarification inspected; implementation and acceptance are separate decisions |
| Independence | No QuantLib implementation source inspected or used |

The previous inability to inspect the ISDA scans in
[M3_DAYCOUNT_RESEARCH](M3_DAYCOUNT_RESEARCH.md) is resolved. The official
1998 memo and 1999 paper both explicitly contain the disputed February
backward-roll rule. An implementation can therefore select **AFB with the
published ISDA clarification**, recording its executable boundary choices,
rather than selecting a rule because it agrees with a reference library.
The original September 1994 AFB publication has not been recovered in this
review; this result does not establish that the clarification appeared there.

## Primary evidence and inspection method

The following downloads were retrieved directly from their publishers on
2026-10-16 Bangkok time. The web reader returned HTTP 403 for the two ISDA
PDFs, but a normal read-only HTTPS download with a browser user-agent
succeeded. A failed web-reader request is consequently not evidence that
these publications are unavailable. Files and renders remain under
`/private/tmp/sabiquant-src-afb-*`, outside the repository.

| Publication | Relevant location | What was actually inspected |
|---|---|---|
| [ISDA, EMU and Market Conventions: Recent Developments, 25 November 1998](https://www.isda.org/a/AIJEE/1998-ISDA-memo-EMU-and-Market-Conventions-Recent-Developments.pdf#page=3) | §4, printed/PDF p.3 of 14; document BS:9951.1 | Downloaded PDF; PDFKit text extraction and full-page rendered image |
| [ISDA, The Actual/Actual Day Count Fraction, 3 June 1999](https://www.isda.org/a/pIJEE/The-Actual-Actual-Day-Count-Fraction-1999.pdf#page=1) | §2(c) and endpoint note, PDF p.1; examples §3, pp.2–7; document ICM:341961.1 | Downloaded seven-page scan; **all seven pages visually inspected** |
| [FBE, French interest-rate addendum, Edition 2004, hosted by FBF](https://www.fbf.fr/uploads/2021/02/Convention_cadre_FBE_additif_operation_taux_interet_2004.pdf#page=6) | §4(7)(i), printed/PDF p.6; §4(11), pp.6–7 | Publisher PDF text; rendered p.6; continuation on p.7 read in web extraction |
| [ICE Trade Vault SEC SBSDR field definitions](https://www.theice.com/publicdocs/data/ICE_Trade_Vault_SEC_SBSDR_Field_List_Validations.pdf#page=8) | Day Count Convention Description table, A010, PDF p.8 of 13 | Publisher PDF text and full table render; enlarged lower table render |

System Poppler and Python PDF packages were absent. The read-only PDF skill
was applied using native macOS PDFKit rendering instead. The 1999 scan has
no useful extractable body text: PDFKit exposes only a copyright line on
page 1. Its substantive evidence above comes from visual reading, not an
empty extraction or a search snippet. Native rendering of the 1998 PDF
emitted a generic CoreGraphics PDF warning; the relevant complete page was
legible and its wording also agrees with the extracted publisher text.

| Download | Bytes | SHA-256 |
|---|---:|---|
| ISDA 1998 | 72,569 | `f35faf99f7eec9ee099514beb68fe356795ffa77ae7e32188a8ce116d0c18a23` |
| ISDA 1999 | 235,422 | `068ad765b5afcdf7a30a3d206540764640663dd077d837be69683fd49b931ab9` |
| FBE/FBF 2004 French | 157,203 | `44c8bb55b84e1f3567d519d9f424cd183b05886b83e09dbcd1ce4f870642dceb` |
| ICE field definitions | 612,382 | `cc50ed03c5f61b7bfe0a19d8c544d5c7e05604b3ba385ac31cec31cd1561aabc` |

These identify retrieved bytes, not an independent publisher-authenticity or
contract-amendment attestation. No downloaded document or image is added to
the repository.

## Verified financial content

The [1998 memo, §4 p.3](https://www.isda.org/a/AIJEE/1998-ISDA-memo-EMU-and-Market-Conventions-Recent-Developments.pdf#page=3)
uses actual days including the first endpoint and excluding the second.
Its AFB denominator is 366 when the relevant period includes February 29,
otherwise 365. For a period longer than one year, it removes complete years
backward from the end and applies the ordinary fractional rule to the initial
stub. Crucially, a backward step from February 28 uses February 29 in the
previous year when that date exists. This is explicit publisher text.

The [1999 paper, §2(c) and note, p.1](https://www.isda.org/a/pIJEE/The-Actual-Actual-Day-Count-Fraction-1999.pdf#page=1)
repeats that clarification and endpoint treatment. It describes **ISDA's
understanding** of the three Actual/Actual approaches; it does not reproduce
the September 1994 AFB document. Selected AFB numerical anchors printed in
its examples are:

| Interval | Printed AFB fraction | Location |
|---|---|---|
| 2003-11-01 → 2004-05-01 | `182/366` | §3(a), p.2 |
| 1999-02-01 → 1999-07-01 | `150/365` | §3(b), pp.2–3 |
| 1999-07-01 → 2000-07-01 | `366/366` | §3(b), p.3 |
| 2002-08-15 → 2003-07-15 | `334/365` | §3(c), p.4 |
| 2000-01-30 → 2000-06-30 | `152/366` | §3(d), pp.5–6 |

The [French 2004 text, §4(7)(i)](https://www.fbf.fr/uploads/2021/02/Convention_cadre_FBE_additif_operation_taux_interet_2004.pdf#page=6)
supports actual-day fractions, the February-29 denominator test and backward
whole years, but **does not specify the February 28→29 promotion**. Its
calculation-period definition includes the starting date and excludes the
following endpoint. The [publisher's English version, §4(7)(i) and §4(11), p.4](https://www.fbf.fr/uploads/2021/09/Master_agreement_financial_transactions_interest_rate_transactions_2004_EN.pdf#page=4)
also omits a February-roll clarification. Neither should be represented as
the original 1994 AFB wording.

[ICE A010](https://www.theice.com/publicdocs/data/ICE_Trade_Vault_SEC_SBSDR_Field_List_Validations.pdf#page=8)
independently presents the February promotion in its ISO-20022-description
and FpML-description columns. Its FIX/FIXML column cites the 2004 AFB text.
This corroborates a reporting convention; it does not turn ICE into the
original AFB author or replace the direct ISDA evidence.

## Proposed executable interpretation

This section records an engineering proposal derived from those rules. It
is not a quotation, a new printed worked example, or a claim that the source
specifies a software algorithm.

For ordered dates `S < E`, maintain a residual endpoint `R`, initially `E`,
and a whole-year count `N`, initially zero. A proposed backward anniversary
is one calendar year before `R`, using the same month/day, with these explicit
February choices: February 29 clamps to February 28 in a common target year;
February 28 promotes to February 29 in a leap target year. Accept that
anniversary and increment `N` while it is on or after `S`. Stop before the
first anniversary preceding `S`. Then:

```text
YF(S,E) = N + actual_days(S,R) / B
B = 366 iff S <= February_29 < R; otherwise 365
```

The equality in the acceptance condition removes a complete final year and
leaves a zero stub. A zero stub need not construct a leap date. The source's
phrase about periods longer than a year is operationalized by this
anniversary test, rather than an arbitrary `actual_days > 365` switch. The
sources give no February-anniversary worked example; the contract must
state this choice explicitly.

For the project's reversed-date API, calculate the ordered interval and
negate; equal endpoints return positive zero. These are local signed-input
decisions. `day_count` remains the signed actual-day difference, irrespective
of the decomposition used for `year_fraction`.

## Independently derived boundary checks

The following values were calculated with Python Gregorian dates and exact
`Fraction` arithmetic, without a reference-library call or production code.
They are **consequences of the proposed executable interpretation**, not
additional examples printed in the publications.

| Interval | Whole years; residual endpoint | Expected fraction |
|---|---|---|
| 2024-02-28 → 2025-02-28 | `1`; 2024-02-29 | `1 + 1/365` |
| 2023-02-28 → 2024-02-28 | `1`; 2023-02-28 | `1` |
| 2024-02-29 → 2025-02-28 | `1`; 2024-02-29 | `1` |
| 2023-02-28 → 2024-02-29 | `1`; 2023-02-28 | `1` |
| 2024-02-28 → 2024-02-29 | `0`; 2024-02-29 | `1/365` |
| 2024-02-29 → 2024-03-01 | `0`; 2024-03-01 | `1/366` |
| 2023-03-01 → 2024-02-29 | `0`; 2024-02-29 | `365/365` |
| 2024-02-28 → 2028-02-28 | `4`; 2024-02-29 | `4 + 1/365` |
| 1994-02-10 → 1997-06-30 | `3`; 1994-06-30 | `3 + 140/365` |
| 2023-12-31 → 2024-02-29 | `0`; 2024-02-29 | `60/365` |
| 2023-12-31 → 2024-03-01 | `0`; 2024-03-01 | `61/366` |
| 0001-01-01 → 9999-12-31 | `9998`; 0001-12-31 | `9998 + 364/365` |

The first row resolves the requested discriminator: the one-day stub is
`[2024-02-28,2024-02-29)`, which excludes February 29 and uses 365. Replacing
the upper-bound comparison by inclusion would silently change it. The
same-year one-day rows isolate the endpoint rule without any year rollback.
The four-year row requires continued February promotion after intervening
common years. At the civil limits, stop without constructing year 0 or
10000, and test reversed/equal endpoints separately.

The oracle should enumerate every civil day over a Gregorian 400-year cycle,
all February endpoint combinations across 1900/2000/2100/2400, and long
intervals. A separately formulated anniversary selector should cross-check
the repeated-year model. Arbitrary split additivity and universal exact
anniversary equality must not become invariants: the denominator and
backward decomposition depend on the requested interval.

## Decision and remaining gates

The source question “does a primary ISDA publication actually prescribe the
February promotion?” now has direct visual evidence. Recommend resolving
OQ-16 for the explicitly selected ISDA-clarified AFB interpretation when its
contract is adopted and independently tested. Preserve the historical
research snapshot rather than rewriting its then-valid access limitation.

The original September 1994 AFB document remains uninspected. The former
Banque de France archive URL for the January 1999 bulletin currently
redirects to its home page; a third-party transcription was located but not
used as primary authority. No claim about the exact original wording,
universal market adoption, legal suitability, complete M3 acceptance, or
unrelated financial gates follows from this source review.
