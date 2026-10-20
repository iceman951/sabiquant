# US Treasury source refresh and interpretation boundaries

Recorded 2026-10-20 (Asia/Bangkok), on local `feat/m8-us-treasury` from
`fc3f629`. This supplements the historical
[2026-10-16 review](M8_US_TREASURY_SOURCE_REVIEW.md); it does not replace that
review's inspected-version record. No Treasury API, executable contract,
general fixture or production code is adopted here. No QuantLib implementation
was read.

## Source versions actually examined

The local research peer downloaded the official versioner snapshots on
2026-10-20. Root independently checked their hashes, parsed the XML, compared
the changed units and fixed-coupon subsections, and visually inspected the
material PDF pages. Root's separate API re-download attempts returned HTTP
406. Consequently this record relies on the pinned downloaded snapshots and
direct inspection, rather than claiming a second successful network fetch.

| Source | Version and scope | SHA-256 |
|---|---|---|
| [Part 356 XML, 2026-10-14](https://www.ecfr.gov/api/versioner/v1/full/2026-10-14/title-31.xml?part=356) | Before the amendment; 282,897 bytes | `8ea626ce8a505cf240028e10d35d97b352b4ef44d1fa34a6ae4ec860a5bf177c` |
| [Part 356 XML, 2026-10-15](https://www.ecfr.gov/api/versioner/v1/full/2026-10-15/title-31.xml?part=356) | After the amendment; 281,916 bytes | `35fa6cb3febd8807c06b08e73d1f311a472b31f127796c6ef369237915b6699d` |
| [Part 356 XML, 2026-10-16](https://www.ecfr.gov/api/versioner/v1/full/2026-10-16/title-31.xml?part=356) | Byte-identical to the 2026-10-15 snapshot | `35fa6cb3febd8807c06b08e73d1f311a472b31f127796c6ef369237915b6699d` |
| [eCFR version history](https://www.ecfr.gov/api/versioner/v1/versions/title-31.json?part=356) | Downloaded JSON, 24,281 bytes | `e991d5b9641a21f2f34a7e2a46daf7ca7cb7ce4567beb7e2dafe52134091b343` |
| [eCFR title status](https://www.ecfr.gov/api/versioner/v1/titles.json) | Title 31: amended 2026-10-15; current through 2026-10-16 | `a799b56daf937f88d9c1b4ec6e1f5a4104211891694370ca39ebad170ac204d4` |
| [FR Doc. 2026-21069](https://www.govinfo.gov/content/pkg/FR-2026-10-15/pdf/2026-21069.pdf) | 91 FR 65456–65460; published and effective 2026-10-15; five PDF pages | `0a2ce4028f42e1c0bb0c35453737410e205fff555bb54c1cdb8f08f4236ffa09` |
| [2004 final rule](https://www.treasurydirect.gov/files/laws-and-regulations/09-02-2004-final-rule.pdf) | Historical corroboration; eight PDF pages; pages 4–5 inspected | `8cc8635f527d1d874bbe10a8178e8bccc9062e824fa60ea13e182ce4fd1e56fd` |

Downloaded artifacts are under
`/private/tmp/sabiquant-treasury-source-refresh-v1/`. Title-level dates describe
these snapshots; they are not dates of every fixed-coupon formula. Earlier
HTML banners and the API snapshot have different observation dates.

## What the amendment changed

Element comparison finds exactly nine changed units: §§356.2, .11, .14, .15,
.20, .21, .23, .31 and Appendix B. Sections 356.10, .25 and .30 are unchanged.
Appendix B changes are confined to inflation-protected material in I.B, III
and V. The rule explains that the changes do not alter the TIPS calculations
it identifies. This review does not adopt or implement TIPS.

The following fixed-principal subsection bodies are identical across the
before/after XML snapshots. Hashes use ElementTree serialization of the
selected child-element spans, not PDF text or a claim of canonical XML.

| Subsection | Serialized bytes | SHA-256 |
|---|---:|---|
| I.A, coupons and Tables 1–2 | 54,920 | `15c4d9836ab6c89681ba12ff9ac1d9eedc942c565865f153c03a7e27a9c9c5a4` |
| I.D, accrued interest | 3,742 | `726822b27b1cbaab76664469fc26c98d50fce8e471217e42b77724c292077bf1` |
| II, fixed-principal pricing A–G | 14,784 | `01ac16d00d88c15c7eb026df32151deb044a0734bcb33a5112bf8d3ae1fae801` |

The source of the earlier fixed-coupon findings therefore remains usable.
The new date definitions still matter to an instrument model: §356.2 now
expressly identifies the **original dated date** and **original issue date**
for reopenings. They must not be merged into one field. Interest accrues from
the dated date; the obligation's original issuance and the current reopening
settlement have different roles.

## Formula and rounding refinements

The [source-example check](M8_US_TREASURY_SOURCE_EXAMPLES.md) examines all seven
II examples, both tables, the I.A/I.D anchors and derived staging controls.
Its exact rational results are source comparisons, not general binary64
oracles.

1. I.A uses a regular half-coupon and actual half-year day fractions for
   fractional coupons. Table 2 displays daily decimals to nine places. Its
   worked products multiply those displayed decimals; they differ from the
   exact fractions before final display rounding.
2. **I.D.4**, rather than I.A.4, specifies accrued interest rounded to five
   places per $1,000 before scaling to another nominal. Coupon computation
   under I.A.3/4 and accrued computation under I.D.4 have distinct staging.
3. II defines a price per 100 rounded to six places. The phrase describing
   normal rounding does not specify a half-tie rule. The seven examples
   contain no relevant tie and cannot choose one.
4. The preface explains that intermediate rounding in the examples is for
   illustration. The displayed steps do not define a universally consistent
   recurrence through every printed intermediate cell.
5. II.E's current XML headline omits both the `r′/s″` division and the `C/2`
   division. The 2004 PDF visibly retains `C/2`, but also omits the first
   division. Definitions and worked lines use the intended ratios. This is a
   source inconsistency to resolve explicitly in any local contract.
6. The zero-yield annuity case `a_n = n` is expressly present. It is not an
   unresolved source gap.

II.F separates raw analytical accrued interest from published accrued interest:
rounding the exact raw-clean price gives 99.777073; the printed clean price is
99.777074. Both rounding `D − A_pub` and subtracting `A_pub` from rounded `D`
reproduce that cell. It therefore does not uniquely require rounding dirty
price before subtraction. II.E likewise cannot establish mandatory
per-component accrued rounding merely because its displayed components add
to the rounded exact total. The accompanying checks retain counterexamples
where those choices differ.

## Scope that still needs an explicit local decision

The regulation supplies auction formulas and announcement terms. Section
356.10 makes the announcement controlling in a conflict, and §356.25 requires
auction payment by the issue date. Section 356.30 delays a weekend or Federal
Reserve closure payment to the next business day without extra interest.
These are not a complete investor-level secondary-market entitlement,
ex-coupon or calendar algorithm. The TreasuryDirect glossary and pricing page
are corroborating explanations; their wording is not fully synchronized with
the amended date definitions.

Before general oracles and Rust implementation, the local Treasury design
must specify together:

- original dated/issue dates, first coupon, quasi dates and all date equality
  and entitlement behavior;
- raw analytical amounts, published accrued amounts and the selected clean
  quote basis;
- exact versus table-decimal coupon/AI staging, component versus total AI
  rounding, tie rule and the four exact-money recipes;
- the whole price/yield domain, inverse attainability, all six risk views and
  their numerical observations;
- resource limits, deterministic error precedence and complete provenance.

Negative yields, broader coupon values or secondary settlements would be
named local engineering extensions. They cannot be attributed to the seven
auction examples as universally accepted market behavior. Owner financial
acceptance, current announcement/legal checks, calendar authority and native
foreign-runtime validation remain separate gates.

## Retained inspection evidence

| Root inspection artifact | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-treasury-source-unit-check-v1.json` | `d53bfb7af2ea2f9fc28188c2a157730873e4b3aba77d74011519d0da79255dd0` |
| `/private/tmp/sabiquant-treasury-fixed-subsections-v1.json` | `e85b07435d41b1f7aabb779451d946cad44b7aa785521703f2809234b5eed4f9` |
| `/private/tmp/sabiquant-treasury-primary-visual-evidence-v1.json` | `36828db05b43b7effbb2d4d4dff934c88d5cd1cdc0ca84534786091f02fa9d50` |

The visual record covers all five pages of the 2026 rule and the two material
2004 pages (printed 53622–53623). The PDF skill was applied read-only, with
PDFKit/AppKit rendering because Poppler was unavailable. No PDF was authored
or edited. Source files, rendered page hashes and failed-fetch qualifications
are retained in the local evidence. This is inspection evidence, not a legal
opinion or financial acceptance.
