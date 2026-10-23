# M8 Canadian fixed-coupon source review

Primary-source research retained on 2026-10-21; repository record prepared
on 2026-10-23 (Asia/Bangkok). This document adopts no Canadian contract, API,
source policy, rounding default or market certification. It records inspected
sources, independently reconstructed examples, contradictions and decisions
still required for the Canadian part of [M8 readiness](M8_LOCAL_READINESS.md).
Existing analytical, amount, settlement, gilt and Treasury contracts remain
unchanged. No QuantLib implementation source was read.

Every candidate resolution below is labelled **PROPOSED LOCAL CHOICE**. That
label describes an engineering option requiring a later explicit contract;
it does not correct the publisher's PDF or assert present-day market practice.

## 1. Retained primary publications

| Source | Direct primary URL | Retained body |
|---|---|---|
| IIAC 1.1 | [Canadian Conventions in Fixed Income Markets, release 1.1](https://iiac.ca/wp-content/uploads/Canadian-Conventions-in-Fixed-Income-Markets-1.1.pdf) | 287,794 bytes; 58 PDF pages |
| IIAC 1.2 | [Canadian Conventions in Fixed Income Markets, release 1.2](https://iiac.ca/wp-content/uploads/2013/05/Canadian-Conventions-in-FI-Markets-Release-1.2-Feb-2015.pdf) | 606,439 bytes; 58 PDF pages |
| IIAC 1.3 | [Canadian Conventions in Fixed Income Markets, release 1.3](https://iiac.ca/wp-content/uploads/Canadian-Conventions-in-FI-Markets-Release-1.3.pdf) | 344,507 bytes; 58 PDF pages |
| BoC | [Calculation of the Clean Price of a Bond in a Cash Management Bond Repurchase Operation](https://www.bankofcanada.ca/wp-content/uploads/2010/07/cal_clean_price.pdf) | 10,118 bytes; 1 PDF page; reached from the Bank's `/cal_clean_price/` redirect |

| Retained filename | SHA-256 |
|---|---|
| `iiac-release-1.1.pdf` | `c149ba6d2c5a9e5668d0642e7511482a6b20a39b3358ec4957bc1b40df0c4671` |
| `iiac-release-1.2.pdf` | `e57d45a72d6141f36a2abf47ec553b5c3ab4e9f10f179553a8f46dcb471c6c20` |
| `iiac-release-1.3.pdf` | `8d9e80c8179af935f891c504d30cbb0023744b5bd0008b3efaa6a96145df2845` |
| `boc-cash-management-clean-price.pdf` | `f70b78eac8c75adc9556da14d6594b89de73c156332dcc8e9ea5b9f1bb34f6ff` |

The covers identify releases 1.1/1.2/1.3 without a printed edition date. The
following dates are provenance observations, not convention effective dates:

| Source | PDF creation metadata, UTC | PDF modification metadata, UTC | Qualification |
|---|---|---|---|
| IIAC 1.1 | 2008-04-25 12:12:53 | 2013-08-21 14:34:31 | Metadata title identifies a release 1.1 Word document |
| IIAC 1.2 | 2015-02-06 15:07:28 | 2015-02-06 15:15:15 | URL filename additionally identifies February 2015 |
| IIAC 1.3 | 2008-04-25 12:12:53 | 2015-10-20 16:53:17 | Cover says 1.3; internal metadata title still says 1.1 |
| BoC | 2005-01-12 10:38:59 | 2005-03-24 16:09:12 | The hosting path contains 2010/07; that is not an edition date |

All four genuine bodies were retrieved successfully on 2026-10-21. HTTP
Last-Modified dates, PDF metadata, URL directory dates and retrieval dates
are not interchangeable. Release 1.3 is the highest-numbered retained release
here, not a claim that no later guidance exists.

The legacy `iiac-accvm.ca` candidate is excluded: the local request returned 403
and a browser request redirected to an unrelated sports site. Neither that
host nor its redirected body is used as authority. A Finance Canada HTML
page was inspected by the source coordinator, but its raw-byte retrieval
timed out; it is not one of the four retained, hash-pinned bodies used for the
formula and cash conclusions in this review.

## 2. Actual inspection and page locators

The read-only PDF skill was applied. Complete pages were rendered with native
Apple PDFKit/AppKit and visually inspected, with extracted text used for
navigation and version comparison. The generic CoreGraphics advisory emitted
while rendering was retained; reviewed formulas and worked values were
legible. Some older text spacing/rendering artifacts are not classified as
financial source defects. No PDF was authored, edited or installed as a
repository fixture.

The independent source/cash challenge inspected 47 unique complete pages:

| Body | Zero-based PDF indices | Scope |
|---|---|---|
| IIAC 1.3 | `3,9..21,29..42,44,46..49,54` | Revision note; date/units discussion; ordinary and odd coupon/AI formulas; compound and money-market reference formulas; selected amortizing AI context; rounding appendix |
| IIAC 1.1 | `16,17,20,33,54` | Numerical errors, late AI, long-first definition and rounding comparison |
| IIAC 1.2 | `3,16,17,20,30,33,54` | Revision note, changed late AI, numerical errors, long-first definition and rounding comparison |
| BoC | `0` | Complete one-page note |

The three IIAC covers were additionally inspected for this repository record.
The coordinator separately inspected the complete 1.3 conventional semiannual
material at indices 6..23 and 29..40, one/two-flow money-market pages 47..49,
rounding/mnemonic/reference pages 54..56 and selected version-comparison pages.
Those complementary scopes are recorded separately in private provenance;
neither reviewer claims visual inspection of all 58 pages of every release.

Use the locators below rather than assuming printed and PDF numbering agree:

| Subject | Printed page | PDF page, one-based | Zero-based index |
|---|---:|---:|---:|
| Revision summary | 3 | 4 | 3 |
| Day ownership and precision | 10 | 11 | 10 |
| Odd coupon definitions and payable formulas | 11..17 | 12..18 | 11..17 |
| Valuation Actual/Actual AI | 18..19 | 19..20 | 18..19 |
| Settlement Actual/365 AI | 20..21 | 21..22 | 20..21 |
| Regular and first/last-stub reference formulas | 29..40 | 30..41 | 29..40 |
| Monthly regular price and settlement AI | 41..42 | 42..43 | 41..42 |
| One/two/generalized money-market formulas | 47..49 | 48..50 | 47..49 |
| Appendix 2, rounding and settlement example | 53 | 55 | 54 |
| BoC complete note | Unnumbered | 1 | 0 |

The discussion below uses printed page numbers, with the table supplying the
PDF locator. The Appendix 2 offset is particularly easy to misread.

## 3. What the sources distinguish

Write `CPN` for the annual decimal coupon rate, `C=100*CPN` for its annual
per 100 amount, and `f` for coupon frequency. The ordinary/odd compound
valuation formulas and settlement cash formulas have different roles:

| Quantity | IIAC source | Formula meaning |
|---|---|---|
| Coupon used in compound valuation | Sections 4.1..4.4, pp 11..13 | Actual/Actual quasi-period fraction times `C/f` |
| Coupon actually payable for an odd period | Sections 4.5.1..4.5.3, pp 14..17 | Actual/365 Canadian Bond piecewise formula, including complete periodic coupons for long first periods |
| `A`, removed from compound-model dirty to obtain clean | Section 5, pp 18..19; reference price formulas | Actual/Actual accrual |
| `AI`, paid in semiannual/quarterly trade settlement | Section 6, pp 20..21 | Actual/365 Canadian Bond accrual |
| Monthly settlement AI | General sections 5/6 versus p 42 | Conflicting source instructions, discussed below |
| AI removed in money-market-equivalent clean pricing | Sections 10.10.1..10.10.3 and BoC | Actual/365, within those formula scopes |

These are source facts about the retained publications. They do not authorize
using one cached coupon/AI amount for every purpose or applying one pricing
mode's clean/dirty definition to another.

### Compound price is a distinct model

The regular semiannual reference on p 29 uses fractional powers of
`b=1+Y/2`, including the settlement exponent `DSC/DCC`. The narrative
comparison with US markets on p 9 does not establish equivalence to every
Treasury fractional-period formula in the separately reviewed
[Treasury sources](M8_US_TREASURY_SOURCE_REVIEW.md).

For an algebraic discriminator, annual decimal yield 0.05, a half-period
remaining and one payment 102.5 give Canadian compound dirty
`102.5/sqrt(1.025) = 5*sqrt(410)`, approximately 101.242283656583. A fractional
simple prefix instead gives `102.5/(1+0.05/4) =8200/81`, approximately
101.234567901235. Their difference is a model distinction, not a numerical
tolerance. This is a derived example, not a printed literature value or an
accepted binary64 oracle.

### Valuation and settlement accrued interest differ

For `C=5`, previous coupon 2005-08-01, settlement 2005-12-01 and next
coupon 2006-02-01, elapsed days are 122 and the full period 184. Section 5 gives
valuation `A=305/184 =1.657608695652...`; section 6 gives settlement
`AI=122/73 =1.671232876712...`, both per 100.

Consequently, the compound model's `D_model` and an all-in settlement amount
need not be the same price: its quoted clean relation is
`P=D_model-A`, while the trade relation is `P+AI`.
The difference `AI-A=183/13432` in this example is exact. These source roles
must remain distinguishable before any future report or money instruction is
designed. Rounding a binary64 clean/dirty observation and applying an exact
source-AI cash formula are also different operations.

### Odd first and final coupons have two amount roles

For accrual 2007-07-16 to first coupon 2008-01-15 within the full quasi period
2007-07-15..2008-01-15, `DIC=183` and `DQ=184`. Compound valuation uses
`(C/2)*(183/184)=915/368 =2.486413043478...`. The payable odd-coupon formula
gives `C*(1/2-(184-183)/365)=363/146 =2.486301369863...`. Plain fixed 365 would
give `183/73 =2.506849315068...` and is neither amount.

Section 4.4 allows a long first period to span one or more complete quasi
periods plus a stub, summing Actual/Actual pieces for valuation. Section 4.5.2
instead adds full periodic coupons to the Actual/365 payable stub. Short-final
valuation uses a full quasi period starting at the penultimate coupon date;
the shortened final interval must not become its own denominator. The
Actual/Actual 31-day final example on p 13 gives `155/364`, consistent with the
displayed 0.425824, without establishing a universal six-place payable-coupon
rounding rule.

## 4. Release differences and unresolved formula interpretations

### Late Actual/365 behavior and its downward step

For regular settlement periods, releases 1.1/1.3 p 20 explicitly print

```text
AI = C*t/365                         when t <365/f
AI = C*(1/f-(s-t)/365)              when t >365/f,
```

where `t` is elapsed days and `s` is the full coupon period. For the 184-day
2005-08-01..2006-02-01 interval and `C=5`:

| Settlement | Elapsed days | Literal AI per 100 |
|---|---:|---|
| 2006-01-30 | 182 | `182/73 =2.493150684931...` |
| 2006-01-31 | 183 | `363/146 =2.486301369863...` |

One additional day decreases the printed-formula AI by
`1/146 =0.006849315068...` per 100. This is an exact consequence of the source
branches, not floating-point cancellation. Payable odd-fragment formulas
switch similarly on the total fragment length `DIC`/`DCM`, not merely on
elapsed settlement days.

In 1.2 p 20 the late branch is prose about prorating the remaining difference;
the explicit late formula and 183-day example are absent. Its p 30 regular
semiannual reference also shows only the less-than 182.5 branch. The 1.3
revision summary identifies a change from 1.2. The formula cannot be uniquely
reconstructed from 1.2 prose by calling it an unchanged universal rule.

The printed comparisons omit threshold equality. The thresholds `365/f`
are nonintegers for frequencies 2/4/12, so equality is unreachable with
integer elapsed days at those frequencies. That observation does not resolve
arbitrary other frequencies.

**PROPOSED LOCAL CHOICE:** select an explicit source/version interpretation
and either preserve the literal interior step or specify a different local
rule with an admitted source deviation. This review selects neither. A cap,
smooth interpolation, straight fixed 365 or Actual/Actual replacement must not
be presented as the unmodified printed formula.

### Completed quasi-periods

Sections 6.3/10.4.2 direct the reader to sum settlement AI across quasi pieces.
Applying the literal regular formula at `t=s=181` or 182 gives `C*s/365`, not
`C/f`; payable long-first section 4.5.2 explicitly adds `C/f` for each complete
period. The general AI instructions do not fully resolve completed virtual
piece endpoints in arbitrary multi-period long-first contexts.

**PROPOSED LOCAL CHOICE:** treat a completed virtual period canonically as
one full periodic coupon, while specifying partial-piece rules separately.
Alternatively, a literal endpoint application would need its own complete
definition. Neither behavior is adopted here, and future expectations must
distinguish them.

### Long-first DIC1 definition

All three retained p 33 pages define `DIC1` as days from accrual to the first
payment, although the formula already contains `1+DIC1/DQ1`. The p 35 formula
instead defines `DIC1` through the first quasi date. For the illustrated
2007-03-01 accrual, 2007-06-01 first quasi and 2007-12-01 first actual coupon,
the two lengths are 92 and 275, with first quasi denominator 182. Literal use
of the first-actual definition in `C/2*(1+DIC1/DQ1)` adds
`915/364 =2.513736263736...` per 100 beyond the section 4.4 construction.
Text extraction differs in object ordering at p 33 between 1.1 and 1.3; visual
inspection does not turn that extraction difference into a corrected formula.

**PROPOSED LOCAL CHOICE:** use the accrual-to-first-quasi stub length when
combining a stub with the separately counted full periods, recording the
p 33 definition as a source inconsistency. This is not an adopted correction.

### Monthly settlement conflict

General sections 5/6 say monthly-pay settlement uses Actual/Actual. Section 6
in 1.2 additionally refers to monthly compounding. Yet the 1.3 monthly regular
reference, section 10.7.2 p 42, prints `AI=C*DCS/365`. For `C=12`,15 elapsed
days in a 31-day month, the general exception gives `15/31 =0.483870967741...`
per 100, while the p 42 expression gives `36/73 =0.493150684931...`.

**PROPOSED LOCAL CHOICE:** give the repeated general monthly exception
precedence and disclose departure from p 42; a different explicit precedence
would require a separate justification. This document adopts neither and
does not verify the current status of the historical regulation cited by IIAC.

### Terminal coupon in the generalized money-market formula

The p 49 section 10.10.3 expression sums regular coupons through `NC`, defined
as the remaining coupon-payment count, and then separately adds `CP+CM`.
For monthly `CPN=.12`, `f=12`, one remaining coupon, zero yield, redemption 100
and terminal coupon 1, the literal expression gives 102. One terminal payment
is 101. It does not reduce consistently to the one-flow terminal principle
used on p 47.

**PROPOSED LOCAL CHOICE:** sum only nonterminal coupons, then include the
terminal coupon/redemption once. This is a local consistency resolution,
not the literal printed summation bound or an adopted market convention.

## 5. Printed numerical inconsistencies

The following values were visually confirmed in all three retained releases:

| Location | Printed calculation/result | Independent exact result | Classification |
|---|---|---|---|
| Section 4.5.3 p 16 | `5*48/365 =0.65734247` | `48/73 =0.657534246575...`; unique nearest at 8 decimal places is 0.65753425 | Arithmetic/display error, not a rounding-mode choice |
| Section 4.5.3 p 17 | A 183-day short-final example sets `NQ=2` and gives 4.986301370 | Preceding short-final formula gives `363/146 =2.486301369863...` | Copied long-first expression adds an extra 2.50 |
| Appendix 2 p 53, nominal 10,000 | Settlement AI 83.298 | `(304/365)*100 =83.287671232876...`; unique nearest cent 83.29 | Numeric/precision error; printed total 10,082.00 agrees with 83.29 |

The date counts in the first two cases agree with their stated dates. The
defects are not repaired by changing the day count or choosing another normal
nearest tie mode.

**PROPOSED LOCAL CHOICE:** use independently derived formulas for local
expectations, preserve each printed mismatch as an explicit source-error
classification, and never silently rewrite a literature anchor. This review
does not create accepted test fixtures or replace the original publications.

## 6. Independent Appendix 2 reconstruction

The example is Canada 8%, maturity 2023-06-01, settlement 2007-07-09, displayed
price 99.987135 and displayed yield 8.000001. Normalizing the yield to annual
decimal 0.08000001, the current coupon interval has 183 days, elapsed 38 and
remaining 145; 32 coupons remain. The valuation AI is `152/183`; settlement AI
is `304/365`, both per 100.

With `b=1+.08000001/2`, the independently reconstructed compound clean price is

```text
P = b^(-145/183) * [100*b^(-31) +4*sum(b^(-k),k=0..31)] -152/183
  = 99.9871345926088375011453170586975994...
```

One route uses Decimal precision 100/160 and agrees through the compared
leading 90 characters. A second route gives a rigorous rational enclosure:
bracket `w=b^(-145/183)` on a 256-bit dyadic grid by exact integer comparisons
of `w^183` with `b^(-145)`, multiply by the exact annuity bracket, then
subtract `152/183`. Both endpoints round to the same principal and total
penny at every quantity below. AI is exact Fraction arithmetic. No actual
full-precision row is a half-cent tie.

| Nominal | Full-price principal, nearest cent | Settlement AI, nearest cent | Sum of rounded components | Nearest unrounded total |
|---:|---:|---:|---:|---:|
| 1,000 | 999.87 | 8.33 | 1,008.20 | 1,008.20 |
| 10,000 | 9,998.71 | 83.29 | 10,082.00 | 10,082.00 |
| 100,000 | 99,987.13 | 832.88 | 100,820.01 | 100,820.01 |
| 1,000,000 | 999,871.35 | 8,328.77 | 1,008,200.12 | 1,008,200.11 |
| 10,000,000 | 9,998,713.46 | 83,287.67 | 10,082,001.13 | 10,082,001.13 |
| 100,000,000 | 99,987,134.59 | 832,876.71 | 100,820,011.30 | 100,820,011.30 |
| 1,000,000,000 | 999,871,345.93 | 8,328,767.12 | 1,008,200,113.05 | 1,008,200,113.05 |

Of the 21 printed principal/AI/total cells,20 agree with the independently
derived component results. The only mismatch is 83.298 above. The 1 million
row distinguishes component rounding from rounding the unrounded total:
the source prints 1,008,200.12, not 1,008,200.11.

The 1 billion row also distinguishes full bond-price precision from scaling
the displayed six-place quote. The latter gives principal 999,871,350.00,
not 999,871,345.93. Likewise scaling the displayed per 100 AI 0.83 gives 8.30
at nominal 1,000, not the source's 8.33.

The nominal 100,000 counterfactual needs care: multiplying 99.987135 gives
99,987.135, an exact half-cent. Choosing 99,987.14 there assumes an even/away
tie rule; it is not a mode-independent discriminator. The larger-quantity
differences above avoid that ambiguity.

These are source-formula research comparisons, not Canadian runtime test
results, prescribed binary64 observations or an ideal-price rounding
certificate for arbitrary inputs.

## 7. Rounding statements and remaining precision gaps

Appendix 2 describes machine-precision calculations with a minimum stated as
ten significant decimals. For bonds it discusses seven-place price accuracy
and six-place rounding, while explicitly retaining full price precision
when scaling principal and rounding the resulting principal to pennies.
Trade settlement AI is also rounded to pennies; present-value AI retains
full precision. Its worked table supplies the concrete discriminators above.

The separate money-market-security paragraph instead describes three-place
price rounding before principal scaling and two-place yield presentation.
That instrument category is not automatically the same as a bond quoted in
money-market-equivalent yield. The source does not settle the unit conversion
or rounding scope for every future API merely by using that yield label.

No inspected passage specifies a Treasury-style daily 9/publication 5 pipeline,
a fixed ten-decimal truncation algorithm, a deterministic binary64 elementary
observation protocol or universal six-place coupon-payment rounding. The
appendix title does not supply those missing rules.

Halfway ties, signed negative ties, arbitrary quanta, coupon-payment cent
stages and refund/negative-invoice behavior are not resolved. For example,
annual coupon 1.825 per 100, semiannual payments, one elapsed day and nominal
100 give an exact settlement AI 0.005. Nearest-even and nearest-away disagree,
while all seven
actual Appendix 2 full-precision quantities above are non-ties.

**PROPOSED LOCAL CHOICE:** use explicit caller/local quantum, tie mode,
aggregation and evidence semantics in a future exact-money boundary; treat
the table as evidence for component aggregation in that example. Neither
arbitrary monetary conventions nor a seven-mode market preset follow from
the source. The existing [settlement contract](M8_SETTLEMENT_CONTRACT.md)
provides engineering vocabulary, not an adopted Canadian adapter.

## 8. Bank of Canada scope and date roles

The BoC note concerns cash-management repurchases of bonds with six months
or less to maturity. Its expressions are

```text
D_MM = (100+C/2)/(1+Y*T/365)
AI   = C*AD/365
P    = D_MM-AI;  settlement price =P+AI.
```

Here `T` runs to the true cash-payment maturity, which the note says may
differ from stated maturity. `AD` measures accrual from the last coupon
payment day to settlement. `C` must be normalized to an annual per 100 amount
in the displayed `100+C/2`, unlike IIAC's decimal `CPN` multiplied by 100.
This is a units inference from the formula, not an implicit percentage
conversion authorized for library inputs.

The note corroborates Actual/365 clean subtraction inside this specific
simple-yield repurchase scope. It does not override IIAC compound-model
Actual/Actual clean subtraction for all tenors, supply odd-coupon cash rules,
resolve ties or define a complete Canadian holiday calendar. IIAC p 48 also
discusses rolling money-market maturity to the next business day; it does not
supply the maintained calendar, settlement lag or every coupon adjustment.

IIAC p 10 describes opening accrued with start included/end excluded. BoC's
last-coupon inclusion wording differs. Ordinary date differences have the
same cardinality when one endpoint is included and the other excluded; these
wordings do not justify inventing an extra day. Coupon-date entitlement,
cash-payment timing and settlement boundary ownership still need explicit
scope.

**PROPOSED LOCAL CHOICE:** distinguish supplied stated/source dates from any
supplied true payment date and state which pricing mode uses each. Infer no
calendar, lag, adjusted-payment policy, ex-coupon rule or post-maturity
trading window from this review.

## 9. Existing requirements and readiness limits

This source ledger supports `NFR-01` specified behavior, `NFR-08` documented
definitions, `NFR-12` reference provenance and `NFR-14` explicit decimal-rate
units in [REQUIREMENTS](REQUIREMENTS.md). It creates no new functional IDs.
The Actual/365 Canadian Bond entry in requirements section 9.9 remains
Under Research; the generic fixed Actual/365 day counter is not this
reference-period-sensitive piecewise convention.

Generic `FR-SETTLE-04/06/12` already require explicit quantum/mode,
aggregation and conditional evidence semantics. Those obligations explain
why a named adapter needs its own source decisions; they do not adopt the
Canadian formulas or import Treasury's source quantizers. The generic
`FR-BOND-04` accrual relation is likewise not silently replaced by this
research.

**PROPOSED LOCAL CHOICE:** a later complete contract can define source-version
precedence, supported frequencies and odd shapes, valuation/payable/quote AI,
completed-piece endpoints, price modes and their time coordinates, the
identified source-error resolutions, exact cash stages, explicit date roles,
range/error/resource proofs and independent expectations. Each choice needs
an explicit source-deviation record where applicable.

No such contract is adopted here. Source availability does not close the
Canadian implementation, independent financial/owner acceptance, maintained
calendar, analytical-enclosure, native foreign-runtime or wider M8 gates in
[local readiness](M8_LOCAL_READINESS.md). The existing
[numerical policy](NUMERICAL_CORRECTNESS.md) and
[ADR-0006](adr/ADR-0006-numeric-policy.md) continue to govern numerical and
money-boundary decisions.

## 10. Reproducible research and retained private provenance

The stdlib-only [research checker](../validation/generator/canadian_source_research.py)
reproduces and expands this source arithmetic. It requires the four original
PDF bodies and verifies their hashes. It is a transparent reproduction of
the independent challenge below, rather than another independent derivation.
Its [validation record](M8_CANADIAN_RESEARCH_VALIDATION.md) distinguishes
exact Appendix enclosures, finite Gregorian research, source-cell
classifications and tool-integrity controls. Those checks create no Canadian
runtime, adopted API or general production oracle.

The direct source bodies, complete-page images, extraction metadata and exact
research arithmetic remain outside the repository/package. They are cited
as investigative provenance, not build dependencies or accepted fixtures:

| Evidence | Private location | SHA-256 |
|---|---|---|
| Actual retrieval and redirect record | `/private/tmp/sabiquant-canadian-primary-review-v2/fetch-evidence.json` | `255e105de4b3227284a9a9f19cfb674c5a479dcb8e70a891da56f50bf3937e9d` |
| Coordinator source readiness, metadata and 70 render/text artifacts | `/private/tmp/sabiquant-canadian-source-readiness-final-v1/evidence.json` | `5f468b3510877bfaa23a0c63f8b5c1983d323cb2af6e4b3cf539b5b214636812` |
| Independent 47-page/source-cash challenge manifest | `/private/tmp/sabiquant-canadian-sourcecash-challenge-v4/evidence.json` | `a377cb699d32aa0f54c06401260be44270d9c5eaae8f5302bffdc8c5c11e67f1` |
| Exact Fraction/Decimal research source | `/private/tmp/sabiquant-canadian-sourcecash-challenge-v4/derive.py` | `42caf52a16b38a318b1762969b7a6220b5718d02c8ea7687ce12c150d4a60b4a` |
| Sixteen arithmetic groups, including exact root enclosure and 21-cell comparison inputs | `/private/tmp/sabiquant-canadian-sourcecash-challenge-v4/arithmetic.json` | `ac3f3ff12bd94b349ecf1e7a159c3a595cc9475f96b2006dcda9be25d80a892b` |
| Complete independent challenge narrative | `/private/tmp/sabiquant-canadian-sourcecash-challenge-v4/review.md` | `71b9fef82fc1bb61613e4b47797eda8b77a92232ed8c37a517be55e03eb43cfa` |

Source discovery/download coordination and the independent formula/cash
challenge are distinct roles. The latter used the genuine retained bodies,
complete visuals and its own stdlib Fraction/Decimal derivations, without
reading Canadian production or copying another generator's arithmetic.
Earlier private evidence remains immutable. A later reproduction must state
whether it is independent or a transparent reproduction of this research.
