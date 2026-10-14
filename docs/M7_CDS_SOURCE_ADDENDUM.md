# Additional CDS source checks for subsequent M7 work

Research on 2026-10-14 UTC continues the
[special-rule source ledger](M7_SPECIAL_RULES_RESEARCH.md). It changes no
production contract and does not make a generic roll Schedule a CDS premium
leg. Runtime probe observations and published financial descriptions remain
separate kinds of evidence.

The directly inspected [OpenGamma quantitative research note 17](https://quant.opengamma.io/Single-Name-CDS-Conventions.pdf),
Yukinori Iwashita, first version December 2, 2013, printed p. 3 footnotes 1–2,
describes the historical short/long first-coupon distinction around thirty
days. This is the author's published technical account, not a pre-2009
confirmation or the ISDA Definitions. Its wording does not resolve the exact
equality case, nor establish that a library's generic start parameter is the
trade date. The observed 29/30-day OldCDS runtime transition therefore remains
a software observation, not a newly verified legal boundary. Printed p. 4
separates adjusted ordinary accrual dates from unadjusted, included maturity.
The note predates the 2015 maturity convention change.

The [2009 Station Casinos ISDA protocol FAQ](https://www.isda.org/traditional-protocol/2009-station-casinos-cds-protocol/)
also describes adjusted ordinary accrual endings and an included unadjusted
maturity in its auction/rebate setting, referencing §1.6 of the 2003 Definitions.
Its primary search-index text was inspected; direct fetch returned 403.
This is corroboration within that protocol's stated scope, not a complete
review of the underlying Definitions or a source for the OldCDS cutoff.
The unrelated thirty-day missed-payment grace period in the same page must
not be mistaken for a first-coupon scheduling threshold.

The [CDS Standard Model news ledger](https://cdsmodel.com/news.html), July 21,
2022 entry, explains the addition of New York holiday handling after the
June 2022 Juneteenth payment-date issue, and the use of combined calendars
corresponding to applicable transaction/currency terms. Consequently, the
2013 note's weekends-only simplification for non-JPY examples cannot establish
a timeless current CDS calendar. An explicit immutable calendar remains a
required input; this is an inference for API design, not an implemented
complete market-calendar catalog.

Next source work should obtain the referenced March 13, 2009 Markit Big Bang
report or an applicable historical confirmation, resolve exact cutoff/day-role
semantics, and distinguish them from modern full-first-coupon contracts.
Maturity regime selection, coupon frequency, accrual endpoints, payment
adjustment, risk-model step-in and legal protection dates need separate
explicit inputs and evidence. None should be inferred from the equality of
one observed CDS/CDS2015 date vector. The current four-rule civil tranche and
its acceptance limits remain unchanged.
