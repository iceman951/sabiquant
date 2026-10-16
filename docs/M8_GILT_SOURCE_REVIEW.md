# M8 gilt source review and engineering implications

Research recorded 2026-10-16 from verified baseline `4da3f43`. This is source
and design evidence; no M8 runtime contract, market coverage or financial
acceptance is adopted by this document. The owner authorized continued local
work; formal milestone acceptance and publication remain separate.

## Retrieved publication and inspection

[UK DMO, Formulae for Calculating Gilt Prices from Yields, fourth edition,
18 December 2024](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf).
The publisher's [formula catalog](https://dmo.gov.uk/publications/gilt-market/formulae-and-examples/)
identifies this edition and separate example spreadsheets. Directly downloaded
PDF: 366,414 bytes, SHA-256
`bd752ac495ec340c0d34e7d6d1e6fe0af2e1d25819364935312f53a9f5037745`.
There are 29 PDF pages. All pages have been text-extracted; only the following
complete rendered pages have been visually inspected in this review:

| Printed page | PDF page (one-based) | Inspection |
|---|---:|---|
| 2 | 3 | Introductory definitions, dates and rounding |
| 4 | 5 | Conventional gilt price/yield formula and variables |
| 5 | 6 | One remaining payment and algebraic inverse |
| 17 | 18 | Standard accrued-interest cases and boundary inequalities |
| 28 | 29 | Notes, including current ex-dividend timing |

The read-only PDF skill was applied. Native PDFKit text extraction/rendering
used an explicit temporary Swift module cache after the default cache write
was denied. The downloaded publication and scratch renders stay outside the
repository. No claim is made to have inspected the separate example workbook
or every formula elsewhere in this publication.

## Selected source facts

For conventional gilts, define `v=1/(1+y/f)`, `a=r/s`. Printed p.4 gives
`P=v^a[d1+d2*v+c*v²(1−v^(n−1))/(f(1−v))+100*v^n]` for n≥1.
Printed p.5 gives `P=v^a(d1+100)` for n=0. P is dirty price per 100 nominal.
The quasi-coupon cycle follows maturity; settlement on a cycle date uses
r=s. Non-business-day dates are retained in this pricing cycle. [§1](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf#page=5)

For standard periods, p.17 uses `AI=(t/s)d1` on or before the ex-dividend
date and `AI=(t/s−1)d1` strictly after it. Note5 describes seven business
days before the dividend. The introduction defines decimal half-up rounding;
settlement values are rounded on the traded amount without intermediate
price rounding. Accrued settlement amounts also round on traded nominal.
[§3 and notes](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf#page=18)

The symbol d1 has different roles: p.4 uses the buyer-entitled next cash flow
(which can be zero ex-dividend), while p.17 uses the full next published
dividend. These must not share one already-detached input field.

These facts are specific to the reviewed gilt publication. Index-linked
rules, short/long first dividends and calendar maintenance require their own
review; they are not inferred from this standard-period summary.

## Derived checks and design decisions still required

The following are engineering deductions, not printed numerical examples.
For y=.05, f=2, one remaining coupon2.5 plus redemption100 and a=1/2,
70-digit Decimal arithmetic gives gilt dirty price
`101.2422836565829346662345114495058504211391969217290310880564863371367`.
Using a fractional simple denominator instead gives
`101.2345679012345679012345679012345679012345679012345679012345679012346`.
The roughly .007716 gap is a model distinction, not a permissible numerical
tolerance. The separate US Treasury review must pin the scope of its formula
before a named Treasury convention is adopted.

An independent rational discriminator with d1=5/2 and t/s=170/182 gives
cum-side AI425/182 and ex-side AI−15/91. Each side is derived at its own
boundary classification; this illustrates the full-coupon entitlement change,
not a claim that accrued interest alone or clean price is continuous there.

The implementation should consequently require a named yield model and explicit
settlement/entitlement context. Existing `InterestRate` is a generic quote,
not evidence of a universal market yield convention. A literal geometric
quotient is undefined at y=0 even when the financial price has a finite limit;
production must use an independently derived stable sum/limit and test zero
and neighboring yields. A material model difference must be classified before
any tolerance comparison.

The source's strict boundary must be preserved or deliberately modeled with
an explicit policy; silently selecting `settlement >= cutoff` would change
p.17 equality behavior. The current library has no maintained UK market
calendar, so a first generic contract can accept ex-dates explicitly while
leaving market date generation and its maintenance unclaimed.

NI-01 must separately specify exact decimal input/quantum and rounding rules.
Rounding the nearest binary64 value does not establish correct rounding of
an ideal price near half a penny. Negative tie behavior, multiple-stage rounding
and scaled traded nominal require explicit operational contracts and integer/
rational or independently rounded oracles. An adopted local contract must separately record how any new money type
relates to accepted ADR-0006; this source review does not change it.
