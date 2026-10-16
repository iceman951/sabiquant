# M8 historical ISMA mathematical source review

Research on 2026-10-16, baseline `4da3f43`; no executable contract adopted.
Primary [ICMA-hosted scan: Patrick J. Brown, Bond Markets: Structures and
Yield Calculations](https://www.icmagroup.org/assets/documents/Media/Bondmarketsbook/Bond%20markets_structures%20and%20yield%20calculations.pdf).
The copyright page identifies publication in 1998 in association with ISMA,
ISBN1-901912-02-7. This is historical mathematical evidence, not verification
of current settlement cycles, market calendars or every contemporary rule.

## Retrieval and actual inspection

Direct download: 19,227,550 bytes; SHA-256
`b9826905add291b80761d3d5060c6e0bfcd4592584c3ff534262cb99fcc0837c`.
There are 54 scanned two-page spreads and no extractable body text. A failed
web-reader extraction did not establish that the publication was unavailable.
Native PDFKit thumbnails handle the 270-degree page rotation and offset crop;
the initial unnormalized drawing clipped the spread and was not used to read
its formulas. Full corrected spreads were visually inspected at zero-based
indices0,3,4,5,13,15,18,19,20,21,41,49,50, corresponding to the cover, publishing
information/contents/introduction and printed pages16–17,20–21,26–33,72–73,
88–91. Other pages are not claimed inspected. Scratch PDF/images remain local.

## Selected source definitions

Printed p17 eq4.3/4.4 defines Macaulay duration by present-value-weighted
cash-flow time. Printed p20 eq4.7 defines modified duration as `−P′(y)/P`;
p90 defines convexity as `P″(y)/P`, with no Taylor factor1/2. In the annual
coordinate, `v=1/(1+y)`:

```text
P = sum(CF_i * v^L_i)
D = sum(CF_i * L_i * v^L_i) / P
MD = D * v
CX = sum(CF_i * L_i * (L_i+1) * v^(L_i+2)) / P
```

Printed pp30–33 distinguish compound redemption yield, a simple final-period
money-market yield, and a zero-coupon case. Printed pp72–73 distinguish clean
and dirty quotation and warn that settlement/accrual/ex-coupon practices vary.
These definitions do not select current market-specific day-count dates.

## Source precision versus independently calculated expectations

The p21 worked example supplies rounded Macaulay duration3.531 for an annual
9% four-year par bond, then reports modified duration3.239. Independent
70-digit Decimal sums on the explicit ideal cash flows give:

```text
D  = 3.531294665988174782186344861775296151669190419673999578387819886658908
MD = 3.239719877053371359803986111720455185017605889609173925126440262989824
3.531 / 1.09 = 3.239449541284403669724770642201834862385321100917431192660550458715596
```

Thus the printed result is consistent with its rounded intermediate input,
while rounding the ideal MD to three decimals gives3.240. This is a source
precision classification; it does not justify widening the library's numerical
policy. A future literature test should either reproduce the explicitly rounded
input or propagate its rounding interval, separately from a high-precision oracle.

The p33 zero-coupon example has redemption100, price30 and stated life10.5:
printed yield0.12150 (12.150%). Independently calculating
`exp(ln(100/30)/10.5)−1` gives
`0.121496637326743232515409996310564182066262044565890344125483530812640`,
which matches the displayed rounding. This example defines life directly;
using its calendar dates with an arbitrary day counter would add an assumption.

## Implications for the local proposal

Risk units must name the yield coordinate and time measure. Annual identities
cannot silently substitute for semiannual, continuous, simple or mixed hybrid
quotes. For a fixed set of horizons, a hybrid's branch is fixed as yield varies;
its time-switch issue is distinct from a yield derivative. Future finite-difference
checks must state the varied coordinate and compare with independently derived
analytic derivatives and two-precision oracles. Numerical inversion must retain
bounded work and typed callback failures instead of adopting an unbounded
Newton iteration solely because it is described in a mathematical reference.
