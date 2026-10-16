# M8 US Treasury source review

Read-only readiness research, 2026-10-16 (Asia/Bangkok), from local baseline
`4da3f43`. This document adopts no bond API or executable contract. No
QuantLib implementation was read.

## Actually inspected sources

| ID | Primary source and locator | Inspection scope/version |
|---|---|---|
| T1 | [TreasuryDirect, Understanding Pricing and Interest Rates](https://www.treasurydirect.gov/marketable-securities/understanding-pricing/) | Bonds and Notes; detailed-formulas link 61 followed to T2. Page has no displayed publication date. |
| T2 | [31 CFR 356 Appendix B, Formulas and Tables](https://www.ecfr.gov/current/title-31/subtitle-B/chapter-II/subchapter-A/part-356/appendix-Appendix%20B%20to%20Part%20356) | Preface; I.A.1–4/Tables 1–2; I.D.1–5/examples; II definitions/A–G; amendment footer. HTML displays Title 31 current through 2026-10-14, last amended 2026-10-06; these are title-level dates. |
| T3 | [Treasury, September 2, 2004 final rule](https://www.treasurydirect.gov/files/laws-and-regulations/09-02-2004-final-rule.pdf#page=4) | PDF pages 4–5, printed Federal Register 53622–53623, visually inspected in full after PDFKit rendering. Historical corroboration of II, not a current consolidated regulation. |
| T4 | [31 CFR Part 356](https://www.ecfr.gov/current/title-31/subtitle-B/chapter-II/subchapter-A/part-356) | §§356.1, .2, .10, .20(d), .25, .30(a)/(c). Same current-through title date as T2. |
| T5 | [TreasuryDirect marketable-securities glossary](https://www.treasurydirect.gov/help-center/glossary-for-marketable-securities/) | Dated Date, Issue Date, First Interest Payment Date. No displayed publication date. |

T3 downloaded bytes: 356,059; SHA-256
`8cc8635f527d1d874bbe10a8178e8bccc9062e824fa60ea13e182ce4fd1e56fd`.
Scratch PDF/render files are outside the repository. PDF skill used read-only;
Poppler unavailable, macOS PDFKit used. No PDF was authored or edited.

## Findings from T1

Fixed non-indexed notes/bonds pay semiannually. The page publishes two rounded
auction prices: 20-year, coupon 1.750%, yield 1.850%, price 98.336995; 7-year,
coupon 1.375%, yield 1.461%, price 99.429922. It supplies neither complete
cash-flow dates nor a general settlement/ex-coupon algorithm. T2 is the
substantive formula source linked by Treasury itself.

## Findings from T2, corroborated where indicated by T3

I.A: regular coupon=annual-interest/2; fractional coupons use actual half-year
days; I.A.3 excludes issue date and includes first payment date. I.A.3/4 scale unrounded per-$1,000 coupon amounts. I.D.4 instead rounds
accrued interest to five decimals per-$1,000 before scaling. Table 2 displays
nine-decimal daily decimals. II defines price per-$100 rounded to six decimals;
its preface warns of illustrative intermediate rounding.

Write `c=C/2`, `h=i/2`, `v=1/(1+h)`, `a=sum(v^k,k=1..n)`; `i` is decimal
nominal annual yield. At zero yield, `a=n`.

| II paragraph | Price relation |
|---|---|
| A/B, regular/short first coupon | `P*(1+(r/s)*h)=c*(r/s)+c*a+100*v^n` |
| C, long first coupon | `P*(1+(r/s)*h)=c*(r/s)*v+c*a+100*v^n` |
| D, regular-period reopening | `(P+A)*(1+(r/s)*h)=c+c*a+100*v^n`; `A=((s-r)/s)*c` |

`r/s` denotes the applicable actual fractional half-year; definitions and
coupon-date treatment of `n` matter. E/F/G cover long/short reopening cases.

Published II example prices A–G: 99.057893; 99.838183; 99.805118; 99.730918;
102.214586; 99.777074; 99.738045. I.D example (1): accrued $16.87500/$1,000,
then $2,531.25/$150,000.

**Source inconsistency:** II.E's headline omits division between `r′` and `s″`,
including T3 p5. Its `AI′` definition and worked resolution use `r′/s″`.

**Algebraic inference:** D with `n=0` gives `(P+A)=(100+c)/(1+(r/s)*h)`;
no published final-period example was located.

## Date roles and entitlement: T4/T5

Part 356 governs specified Treasury auctions and their announcements, not
all bond markets (§356.1). An announcement controls conflicting terms
(§356.10). Dated date starts accrual; issue date creates the obligation;
maturity ends interest (§356.2). Settlement is delivery plus complete payment;
settlement amount includes premium/discount and accrued interest (§356.2).
Auction payment is due by issue date (§356.25).

§356.30(a) delays a payment falling on a weekend/Federal-Reserve closure to
the next business day without additional interest. §356.30(c) describes
Treasury's discharge of payment obligations. These provisions do not provide
an investor-level secondary-market record-date/ex-coupon rule. T5 independently
distinguishes dated date from issue/settlement date.

The direct single-section §356.30 view returned an older 2026-05-28 title
banner; the current Part 356 view above was also inspected. Its relevant
payment text agrees. Do not infer the appendix itself changed on either
title-level amendment date.

## Consequences for local M8 readiness

A secondary-market extension needs an explicit convention and tests. The
algebraic inference above is not a universal market rule.

Printed rounded examples can become literature anchors with stated decimal
comparison policies; they are not exact binary64 oracles. Zero yield needs
a defined finite limit. Yield inversion, duration/convexity definitions,
negative-yield domain, irregular final coupons, secondary coupon entitlement,
and settlement-date equality behavior remain decisions for the M8 contract.
No numerical tolerances, defaults or error precedence are adopted here.

Analytical settlement dates and eligible future cash flows can be specified
without choosing a money representation. Exact decimal settlement, currency,
and monetary rounding policy remain the separate NI-01 decision under
[ADR-0006](adr/ADR-0006-numeric-policy.md). These
source rules do not authorize silently adding an exact-money API or changing
existing analytical cash-flow arithmetic.
