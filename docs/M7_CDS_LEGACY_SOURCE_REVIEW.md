# Legacy CDS first-coupon source review

Research completed 2026-10-15, Asia/Bangkok, continuing the
[source addendum](M7_CDS_SOURCE_ADDENDUM.md) and
[special-rule research](M7_SPECIAL_RULES_RESEARCH.md). This is local source
research only. No production behavior, fixtures, acceptance gate or financial
contract is changed. No QuantLib C++ implementation was inspected.

The original Markit report is now available and its relevant diagram was
visually inspected. It improves the historical evidence but does **not** close
the exact first-coupon cutoff or generic-start-role gap. Its prose and diagram
cannot safely be combined into a precise thirty-calendar-day specification.

## Direct source evidence

| Source and inspection | Evidence and limits |
|---|---|
| [Markit report dated March 13, 2009, hosted by CDFA](https://www.cdfa.net/cdfa/cdfaweb.nsf/ord/c262660031945ca488257936006983fd/%24file/cds_big_bang.pdf), printed pp. 13-18; p. 17 rendered locally and visually inspected | The 26-page report bears Markit branding and copyright. This is a third-party-hosted original document, not CDFA commentary; hosting does not establish a publisher-certified copy. |
| [Otis Casey, Society of Actuaries, August 2009](https://www.soa.org/49359b/globalassets/assets/library/newsletters/risks-and-rewards/2009/august/rar-2009-iss54-casey.pdf), printed pp. 1, 4 and 9; direct PDF text | Page 1 identifies the author as Markit's VP of credit products and identifies an authorized Markit Magazine reprint. Page 9 repeats the relevant date windows. This corroborates the diagram's transcription, not its calendar arithmetic. |
| [Original 2003 ISDA Credit Derivatives Definitions, hosted by Standard Chartered](https://www.sc.com/en/uploads/sites/66/content/docs/2003-ISDA-Credit-Derivative-Definitions.pdf), printed pp. 1, 10 and 63; direct PDF text | ISDA-authored original Definitions, with 2003 copyright, on a bank's corporate host. Distinct from that host's separately available consolidation incorporating 2009 supplements. Only the identified provisions were reviewed for this question. |

Markit p. 17 describes first-coupon selection using the trade date: an earlier
trade pays the initial partial coupon, while a trade within thirty days defers
it. It does not expressly define equality. Its diagram instead classifies
effective dates: December 21-February 20 pay March 20, 2009; February 21-March
20 defer payment to June 20. Page 13 describes historical protection beginning
the next business day. Pages 14 and 18 discuss changed protection/accrual roles
and full-coupon treatment. Those later conventions do not repair the legacy
boundary. [Markit, pp. 13-18](https://www.cdfa.net/cdfa/cdfaweb.nsf/ord/c262660031945ca488257936006983fd/%24file/cds_big_bang.pdf)

The Definitions distinguish confirmation-specified effective date from trade
date in sections 1.4-1.5. Section 2.9 starts initial accrual on the effective
date; sections 2.8 and 2.10 use confirmation-specified period-end/payment dates.
Exhibit A, p. 63, supplies payment-date blanks. These provisions establish
distinct date roles but provide no thirty-day first-payment selection rule.
They do not make a library's generic start input the trade date, nor impose a
universal T+1 transformation. [ISDA, pp. 1, 10, 63](https://www.sc.com/en/uploads/sites/66/content/docs/2003-ISDA-Credit-Derivative-Definitions.pdf)

## Independent arithmetic and remaining gap

Ordinary Gregorian date subtraction, independently checked with Python's
`datetime.date`, gives:

| Date | Days until 2009-03-20 | Weekday |
|---|---:|---|
| 2009-02-18 | 30 | Wednesday |
| 2009-02-19 | 29 | Thursday |
| 2009-02-20 | 28 | Friday |
| 2009-02-21 | 27 | Saturday |

The displayed transition is therefore not a thirty-day boundary measured from
the effective date. Relabeling it as a trade-date rule also cannot resolve the
problem without additional assumptions: even a next-business-day conversion
would normally associate Friday February 20 with Thursday February 19, only
29 calendar days before March 20. The diagram's inclusion of Saturday February
21 further prevents treating its shaded range as a complete executable
business-day mapping. These are arithmetic observations and consistency
checks, not a correction to the source.

The material cannot distinguish an imprecise illustration, a one-month
convention, an omitted date-adjustment convention, or another historical rule.
Selecting any of those explanations would be speculation. In particular,
the February 18 equality case has no independently reconciled contractual
answer here. An applicable historical confirmation or explicit contemporary
market specification is still needed, including which date is tested,
calendar versus business days, equality, and adjusted versus unadjusted first
payment.

The previously attested OldCDS observation that start distances 1/29 omit the
nearby roll while 30/31 retain it remains software evidence. No runtime was
executed for this review. Neither these source findings nor that observation
authorize identifying the generic start with a legal trade/effective date.
The legacy financial gate remains open; modern full-first-coupon date-plan
work can proceed under its own separately sourced contract.

## Retrieval record

The former Markit-hosted URL and the Stace Sirmans mirror failed direct web
retrieval. The CDFA copy succeeded; its p. 17 browser screenshot failed, so the
public PDF was downloaded into temporary storage and rendered with macOS
PDFKit for inspection. Poppler was unavailable. The original source was not
edited or added to the repository.

- Download size: 836,434 bytes; 26 pages.
- SHA-256: `87cba47c6719953f765ca8e83d5e2aad1a0cd8e3be310b4d1053c0fc568659fa`.
- Cover date: March 13, 2009; PDF metadata author: `otis.casey`.
- PDF metadata creation/modification timestamp: 2009-03-14 02:12:38 UTC.
- Markit printed p. 17 is zero-based PDF page 16; SOA printed p. 9 is PDF
  page 7; ISDA printed pp. 1, 10 and 63 are PDF pages 7, 16 and 69.

The unversioned SOA URL returned 404; the linked publisher-hosted hashed URL
was directly readable. No inference of changed historical content follows
from that URL difference. Source summaries are intentionally short; no source
document or full-page reproduction is distributed with this review.
