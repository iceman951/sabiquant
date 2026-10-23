#!/usr/bin/env python3
"""Reproduce Canadian primary-source research, without a production oracle.

Python stdlib only. No Rust, QuantLib, prior fixture or library result is read.
This tool makes source inconsistencies reproducible; it adopts no convention,
market rounding policy, API or Canadian numerical observation algorithm.

Provenance: the source/cash derivations expand the independent Codex review
retained at /private/tmp/sabiquant-canadian-sourcecash-challenge-v4/derive.py
(SHA256 42caf52a16b38a318b1762969b7a6220b5718d02c8ea7687ce12c150d4a60b4a).
This is a reviewed reproduction of that work, not another independent audit.
Complete material PDF pages were visually inspected; byte guards alone do not
prove formula transcription. See docs/M8_CANADIAN_SOURCE_REVIEW.md.

Four original source PDFs are required and must match their retained hashes.
The deterministic report is create-new only. --check-report compares an
existing report, without replacing it. All checks use explicit exceptions
and therefore also run under python -O. Decimal corroboration is distinct
from the exact rational enclosure that certifies the Appendix penny results.
"""

import argparse
import calendar
from collections import Counter
from datetime import date
from decimal import Decimal, localcontext
from fractions import Fraction as F
import hashlib
import json
from pathlib import Path
import sys


SOURCES = {
    "iiac-release-1.1.pdf": (
        "c149ba6d2c5a9e5668d0642e7511482a6b20a39b3358ec4957bc1b40df0c4671",
        "https://iiac.ca/wp-content/uploads/Canadian-Conventions-in-Fixed-Income-Markets-1.1.pdf",
    ),
    "iiac-release-1.2.pdf": (
        "e57d45a72d6141f36a2abf47ec553b5c3ab4e9f10f179553a8f46dcb471c6c20",
        "https://iiac.ca/wp-content/uploads/2013/05/Canadian-Conventions-in-FI-Markets-Release-1.2-Feb-2015.pdf",
    ),
    "iiac-release-1.3.pdf": (
        "8d9e80c8179af935f891c504d30cbb0023744b5bd0008b3efaa6a96145df2845",
        "https://iiac.ca/wp-content/uploads/Canadian-Conventions-in-FI-Markets-Release-1.3.pdf",
    ),
    "boc-cash-management-clean-price.pdf": (
        "f70b78eac8c75adc9556da14d6594b89de73c156332dcc8e9ea5b9f1bb34f6ff",
        "https://www.bankofcanada.ca/cal_clean_price/",
    ),
}
APPENDIX_PRINTED = (
    (1_000, "999.87", "8.33", "1008.20"),
    (10_000, "9998.71", "83.298", "10082.00"),
    (100_000, "99987.13", "832.88", "100820.01"),
    (1_000_000, "999871.35", "8328.77", "1008200.12"),
    (10_000_000, "9998713.46", "83287.67", "10082001.13"),
    (100_000_000, "99987134.59", "832876.71", "100820011.30"),
    (1_000_000_000, "999871345.93", "8328767.12", "1008200113.05"),
)


class ResearchFailure(Exception):
    """A source guard, mathematical check or report comparison failed."""


def require(condition, message):
    if not condition:
        raise ResearchFailure(message)


def ratio(value):
    value = F(value)
    return f"{value.numerator}/{value.denominator}"


def days(start, stop):
    return (date.fromisoformat(stop) - date.fromisoformat(start)).days


def source_ai(coupon, frequency, elapsed, period):
    """Literal IIAC1.1/1.3 section6.1; no monthly conflict is resolved here."""
    require(0 <= elapsed <= period, "invalid research day coordinate")
    threshold = F(365, frequency)
    if elapsed < threshold:
        return coupon * F(elapsed, 365)
    if elapsed > threshold:
        return coupon * (F(1, frequency) - F(period - elapsed, 365))
    raise ResearchFailure("source strict-inequality equality gap")


def nearest_unique(value, places):
    """Unique nearest only: a half tie is refused, never silently resolved."""
    value = F(value) * 10**places
    quotient, remainder = divmod(value.numerator, value.denominator)
    require(2 * remainder != value.denominator, "nearest rule unspecified at tie")
    return F(quotient + (2 * remainder > value.denominator), 10**places)


def money(value):
    value = F(value)
    integer = value * 100
    require(integer.denominator == 1, "money formatter needs an exact cent")
    sign = "-" if integer < 0 else ""
    integer = abs(integer.numerator)
    return f"{sign}{integer // 100}.{integer % 100:02d}"


def root_bracket(base, numerator, denominator, bits):
    """Enclose base**(-numerator/denominator), base>=1, by integer powers."""
    require(base >= 1 and numerator >= 0 and denominator > 0 and bits > 0,
            "invalid root enclosure input")
    unit = 1 << bits
    lower, upper = 0, unit
    rhs = unit**denominator * base.denominator**numerator
    lhs_factor = base.numerator**numerator
    while upper - lower > 1:
        mid = (lower + upper) // 2
        if mid**denominator * lhs_factor <= rhs:
            lower = mid
        else:
            upper = mid
    lo, hi = F(lower, unit), F(upper, unit)
    # For the exact endpoint one, retain a closed singleton; this prevents
    # claiming a strict upper inequality when numerator=0 or base=1.
    if base == 1 or numerator == 0:
        lo = hi = F(1)
    require(lo**denominator <= 1 / base**numerator <= hi**denominator,
            "root enclosure failed exact end-to-end comparison")
    return lo, hi


def helper_controls():
    """Mathematical/tool controls; not fabricated financial observations."""
    controls = []
    for value, places, expected in (
        (F(124, 100), 1, F(12, 10)),
        (F(126, 100), 1, F(13, 10)),
        (-F(124, 100), 1, -F(12, 10)),
        (-F(126, 100), 1, -F(13, 10)),
        (F(0), 2, F(0)),
        (F(10**40) + F(1, 1000), 2, F(10**40)),
    ):
        require(nearest_unique(value, places) == expected, "nearest helper control")
        controls.append("nearest-" + ratio(value))
    for value in (F(1, 200), -F(1, 200)):
        try:
            nearest_unique(value, 2)
        except ResearchFailure as error:
            require(str(error) == "nearest rule unspecified at tie", "wrong tie refusal")
        else:
            raise ResearchFailure("tie helper failed to refuse")
        controls.append("tie-refusal-" + ratio(value))
    for base, n, d, bits in ((F(4), 1, 2, 8), (F(1), 2, 3, 8),
                             (F(7, 3), 0, 5, 8), (F(26, 25), 1, 2, 96)):
        lo, hi = root_bracket(base, n, d, bits)
        require(hi - lo <= F(1, 2**bits), "root width helper")
        controls.append(f"root-{ratio(base)}-{n}-{d}")
    # Equality cannot occur for f2/4/12 and integer elapsed. f1 is only a
    # synthetic refusal control, not an admitted Canadian frequency.
    try:
        source_ai(F(5), 1, 365, 365)
    except ResearchFailure as error:
        require(str(error) == "source strict-inequality equality gap", "wrong AI refusal")
    else:
        raise ResearchFailure("source equality gap was silently filled")
    controls.append("synthetic-source-equality-gap")
    require(money(-F(123, 100)) == "-1.23", "signed formatter helper")
    controls.append("signed-money-formatter")
    return controls


def appendix():
    base = 1 + F(8_000_001, 100_000_000) / 2
    elapsed, period = days("2007-06-01", "2007-07-09"), days("2007-06-01", "2007-12-01")
    remaining = days("2007-07-09", "2007-12-01")
    require((elapsed, period, remaining) == (38, 183, 145), "Appendix date counts")
    inner = 100 / base**31 + sum((F(4) / base**k for k in range(32)), F(0))
    valuation_ai, settlement_ai = F(4 * elapsed, period), F(8 * elapsed, 365)
    lo, hi = root_bracket(base, remaining, period, 256)
    price_lo, price_hi = lo * inner - valuation_ai, hi * inner - valuation_ai
    decimal_prices = []
    for precision in (100, 160):
        with localcontext() as context:
            context.prec = precision
            b = 1 + Decimal("0.08000001") / 2
            a = Decimal(4) * elapsed / period
            pv = (100 / b**31 + sum((Decimal(4) / b**k for k in range(32)), Decimal(0))) / b**(Decimal(remaining) / period)
            p = pv - a
            require(price_lo < F(p) < price_hi, "Decimal does not corroborate exact price enclosure")
            decimal_prices.append(str(p))
    rows, cells, counts = [], [], Counter()
    for nominal, printed_capital, printed_ai, printed_total in APPENDIX_PRINTED:
        scale = F(nominal, 100)
        capital_lo = nearest_unique(price_lo * scale, 2)
        capital_hi = nearest_unique(price_hi * scale, 2)
        total_lo = nearest_unique((price_lo + settlement_ai) * scale, 2)
        total_hi = nearest_unique((price_hi + settlement_ai) * scale, 2)
        require(capital_lo == capital_hi, "principal enclosure does not decide unique cent")
        require(total_lo == total_hi, "total enclosure does not decide unique cent")
        accrued = nearest_unique(settlement_ai * scale, 2)
        values = (capital_lo, accrued, capital_lo + accrued)
        for column, printed, computed in zip(("principal", "settlement_ai", "component_total"),
                                            (printed_capital, printed_ai, printed_total), values):
            expected_typo = nominal == 10_000 and column == "settlement_ai"
            agrees = F(printed) == computed
            require(agrees != expected_typo, f"unexpected Appendix cell classification {nominal}/{column}")
            classification = "agrees" if agrees else "printed_arithmetic_error"
            counts[classification] += 1
            cells.append(dict(nominal=nominal, column=column, printed=printed,
                              reconstructed=money(computed), classification=classification))
        rows.append(dict(nominal=nominal, principal=money(capital_lo), accrued=money(accrued),
                         component_total=money(capital_lo + accrued), round_total=money(total_lo)))
    require(dict(counts) == {"agrees": 20, "printed_arithmetic_error": 1}, "Appendix cell census")
    require(rows[3]["component_total"] == "1008200.12" and rows[3]["round_total"] == "1008200.11",
            "Appendix aggregation discriminator lost")
    shown_price = F(99_987_135, 1_000_000)
    require(nearest_unique(shown_price * 10_000_000, 2) == F("999871350.00"),
            "Appendix six-place scaling discriminator lost")
    halfcent = shown_price * 1000 * 100
    require(halfcent - halfcent.numerator // halfcent.denominator == F(1, 2),
            "100k shown-price counterfactual must be qualified as tie")
    require(nearest_unique(settlement_ai * 10, 2) == F("8.33") and F("0.83") * 10 == F("8.30"),
            "per100 AI presentation discriminator lost")
    return dict(locator="IIAC1.1/1.2/1.3 PDF index54, printed53, Appendix2",
                yield_decimal="0.08000001", base=ratio(base), valuation_ai=ratio(valuation_ai),
                settlement_ai=ratio(settlement_ai), factor_lower=ratio(lo), factor_upper=ratio(hi),
                clean_price_lower=ratio(price_lo), clean_price_upper=ratio(price_hi),
                decimal_precisions=[100, 160], decimal_prices=decimal_prices, rows=rows, cells=cells,
                cell_counts=dict(counts), shown_6dp_price_1bn_principal="999871350.00",
                shown_6dp_price_100k_is_half_cent=True,
                midpoint_policy_inferred=False, all_seven_penny_results_exactly_certified=True)


def source_discriminators():
    c = F(5)
    checks = {}
    require((days("2005-08-01", "2005-12-01"), days("2005-08-01", "2006-02-01")) == (122, 184), "regular dates")
    a, ai = c * F(122, 2 * 184), source_ai(c, 2, 122, 184)
    require((a, ai) == (F(305, 184), F(122, 73)), "valuation/settlement separation")
    checks["valuation_vs_settlement"] = [ratio(a), ratio(ai)]
    before, after = source_ai(c, 2, 182, 184), source_ai(c, 2, 183, 184)
    require(after - before == -F(1, 146), "literal late accrued drop")
    checks["literal_184_day_AI_drop"] = dict(before=ratio(before), after=ratio(after), difference=ratio(after-before))
    require(days("2007-07-16", "2008-01-15") == 183 and days("2007-07-15", "2008-01-15") == 184, "odd dates")
    odd = (c * F(183, 2 * 184), source_ai(c, 2, 183, 184), c * F(183, 365))
    require(odd == (F(915, 368), F(363, 146), F(183, 73)), "odd coupon amount separation")
    checks["odd_coupon_valuation_payable_plain365"] = list(map(ratio, odd))
    checks["long_first_plus_completed_piece"] = dict(valuation=ratio(odd[0]+c/2), payable=ratio(odd[1]+c/2))
    checks["completed_piece_endpoint_ambiguity"] = {str(s):ratio(source_ai(c,2,s,s)) for s in (181,182,183,184)}
    require(checks["completed_piece_endpoint_ambiguity"]["181"] != ratio(c/2), "completed piece distinction")
    d_stub, d_actual = days("2007-03-01", "2007-06-01"), days("2007-03-01", "2007-12-01")
    require((d_stub,d_actual)==(92,275), "DIC1 dates")
    require(c*F(d_actual-d_stub,2*182)==F(915,364), "DIC1 excess")
    checks["DIC1_definition_ambiguity"] = dict(first_quasi_days=d_stub, first_actual_days=d_actual, extra_coupon=ratio(F(915,364)))
    require(days("2019-07-15", "2019-09-01")==48, "short-last days")
    computed = c*F(48,365)
    require(nearest_unique(computed,8)==F("0.65753425") and F("0.65734247")!=nearest_unique(computed,8), "48-day printed arithmetic classification")
    checks["printed_48_day_error"] = dict(printed="0.65734247", exact=ratio(computed), nearest_8dp="0.65753425")
    require(F("4.986301370") != nearest_unique(odd[1],9) and nearest_unique(odd[1]+c/2,9)==F("4.986301370"), "short-last NQ2 extra coupon")
    checks["printed_short_last_extra_coupon"] = dict(printed="4.986301370", exact_short_last=ratio(odd[1]), excess_full_coupon=ratio(c/2))
    require((days("2007-10-31", "2007-12-01"),days("2007-10-31", "2008-04-30"))==(31,182), "ActAct source dates")
    require(nearest_unique(c*F(31,2*182),6)==F("0.425824"), "ActAct source example")
    checks["31_day_ActAct_display_agrees"] = dict(exact=ratio(c*F(31,2*182)), printed="0.425824")
    monthly = (F(15,31), F(36,73))
    require(monthly[0] != monthly[1], "monthly conflicting source formulas")
    checks["monthly_AI_conflict"] = dict(coupon_per100=12, elapsed=15, period=31, section6=ratio(monthly[0]), section10_7_2=ratio(monthly[1]))
    checks["monthly_MM_double_terminal"] = []
    for count in (1,2,3,12):
        # y=0 removes all day/discount issues. Every remaining coupon is1.
        literal, once = F(count+100+1), F(count+100)
        require(literal-once==1, "terminal double count")
        checks["monthly_MM_double_terminal"].append(dict(count=count,literal=ratio(literal),terminal_once=ratio(once)))
    boc_dirty = (100+c/2)/(1+F(1,20)*F(91,365))
    boc_ai = c*F(91,365)
    require(boc_ai != c*F(91,2*182), "BoC scoped accrued distinction")
    checks["BoC_scoped_clean_formula"] = dict(T=91,AD=91,yield_decimal="0.05",dirty=ratio(boc_dirty),AI=ratio(boc_ai),clean=ratio(boc_dirty-boc_ai),compound_valuation_AI=ratio(c*F(91,2*182)))
    # Same half-period first factor, complete C8 four-payment semiannual
    # stream. All later factors are rational. A square comparison separates
    # fractional discounting from the US Treasury rational first factor.
    b=F(26,25)
    stream=100/b**3+sum((F(4)/b**k for k in range(4)),F(0))
    ust=stream/F(51,50)-2
    require(stream**2/b > (ust+2)**2, "fractional model accidentally equated with Treasury")
    checks["fractional_vs_Treasury_half_period"] = dict(base=ratio(b),compound_dirty_square=ratio(stream**2/b),Treasury_clean=ratio(ust),strict_compound_clean_greater=True)
    # Signed settlement dirty is mathematically possible, even though every
    # fractional-compound model PV is positive. Do not infer a contract from
    # this witness: ideal arithmetic has no binary observation stages here.
    cmax=F(2**127-1)
    a=cmax*F(1,2*181)
    ai=cmax*F(1,365)
    y=F(2**540)
    base=1+y/2
    # S=2025-03-02, previous03-01,next09-01 is184 days, not181.
    # Use actual valid previous2025-09-01,S09-02,next2026-03-01 (181).
    require(days("2025-09-01","2026-03-01")==181, "signed witness period")
    wlo,whi=root_bracket(base,180,181,600)
    dirty_lo=(100+cmax/2)*wlo
    dirty_hi=(100+cmax/2)*whi
    require(dirty_lo>0 and dirty_hi-a+ai<0, "signed settlement-dirty witness")
    checks["ideal_positive_model_dirty_negative_settlement_dirty"] = dict(previous="2025-09-01",settlement="2025-09-02",maturity="2026-03-01",coupon_per100=ratio(cmax),yield_value=ratio(y),model_dirty_lower=ratio(dirty_lo),model_dirty_upper=ratio(dirty_hi),valuation_AI=ratio(a),settlement_AI=ratio(ai),settlement_dirty_upper=ratio(dirty_hi-a+ai),observed_binary64_claim=False)
    return checks


def calendar_and_daily_census():
    """Finite Gregorian research census; no schedule/public API is adopted."""
    lengths={f:Counter() for f in (2,4,12)}
    for year in range(2000,2400):
        for month in range(1,13):
            for anchor in range(1,32):
                first=date(year,month,min(anchor,calendar.monthrange(year,month)[1]))
                for f in lengths:
                    end_month=year*12+month-1+12//f
                    ey,em=divmod(end_month,12)
                    last=date(ey,em+1,min(anchor,calendar.monthrange(ey,em+1)[1]))
                    lengths[f][(last-first).days]+=1
    expected={2:{181,182,183,184},4:{89,90,91,92},12:{28,29,30,31}}
    daily={}
    for f,counts in lengths.items():
        require(sum(counts.values()) == 400*12*31, f"unexpected f{f} calendar pair count")
        require(set(counts)==expected[f], f"unexpected f{f} calendar lengths")
        rows=early=late=equality=actact_differences=completed_differences=drops=0
        for s in sorted(counts):
            previous=None
            for t in range(s+1):
                rows+=1
                early+=t<F(365,f)
                late+=t>F(365,f)
                equality+=t==F(365,f)
                literal=source_ai(F(5),f,t,s)
                actact=F(5*t,f*s)
                actact_differences+=literal!=actact
                completed_differences+=t==s and literal!=F(5,f)
                if previous is not None:
                    drops+=literal<previous
                previous=literal
        require(equality==0 and early+late==rows, "admitted-frequency equality unexpectedly reachable")
        daily[str(f)]=dict(rows=rows,early=early,late=late,equality=equality,
                           literal_vs_ActAct_differences=actact_differences,
                           completed_piece_differences=completed_differences,negative_one_day_steps=drops,
                           monthly_literal_is_conflicting_source_candidate=f==12)
    return dict(years="2000..2399",anchors_per_month=31,
                direct_clamped_pairs_per_frequency=148800,
                lengths={str(f):{str(k):v for k,v in sorted(counts.items())} for f,counts in lengths.items()},daily=daily)


def report(source_directory):
    sources=[]
    for name,(expected,url) in SOURCES.items():
        body=(source_directory/name).read_bytes()
        actual=hashlib.sha256(body).hexdigest()
        require(actual==expected, f"source SHA256 mismatch: {name}")
        sources.append(dict(name=name,bytes=len(body),sha256=actual,url=url))
    return dict(schema="sabiquant-canadian-source-research-v1",
                scope="Source arithmetic/research only; no production oracle, adopted API or current-market certification.",
                script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                provenance="Reviewed reproduction of sourcecash challenge v4; not independent of that derivation.",
                source_pdfs=sources,helper_controls=helper_controls(),
                source_discriminators=source_discriminators(),appendix=appendix(),
                calendar_research=calendar_and_daily_census())


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-directory",type=Path,required=True)
    output=parser.add_mutually_exclusive_group(required=True)
    output.add_argument("--output",type=Path)
    output.add_argument("--check-report",type=Path)
    args=parser.parse_args()
    # Refuse an existing destination before doing mathematical work. Resolve
    # aliases too; exclusive creation remains the final race-safe guard.
    if args.output is not None:
        require(not args.output.exists(), "output already exists; overwrite refused")
    result=report(args.source_directory)
    rendered=(json.dumps(result,indent=2,sort_keys=True,allow_nan=False)+"\n").encode("utf-8")
    if args.check_report is not None:
        require(args.check_report.read_bytes()==rendered, "research report byte mismatch")
        print("PASS: exact research report reproduction; 21 Appendix cells classified")
    else:
        with args.output.open("xb") as stream:
            stream.write(rendered)
        print("PASS: research report created; 21 Appendix cells classified")


if __name__=="__main__":
    try:
        main()
    except (ResearchFailure,OSError,ValueError) as error:
        print(f"FAIL: {error}",file=sys.stderr)
        sys.exit(1)
