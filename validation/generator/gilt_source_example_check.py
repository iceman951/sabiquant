#!/usr/bin/env python3
"""Pre-contract independent checks of official conventional-gilt examples.

This does not generate a Rust fixture or adopt a market contract. It reads
pinned official PDF/text evidence, uses only Python's standard library, and
writes a new scratch JSON report. No production module or QuantLib is read.
The text is the complete PDFKit extraction with page separators used by the
source review; its hash avoids silently accepting a changed transcription.
"""

import argparse
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal, ROUND_HALF_UP, localcontext
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import re

PDF_SHA = '8f491b8567c6f3a75ee2a7cbc02ccc4275810257df5533ed1fd349992465e018'
TEXT_SHA = '616e51583c7607dd6dc4859babc587234a28a8536945bf36eb641120df4d8109'
SOURCE_URL = 'https://www.dmo.gov.uk/media/ftjpyv1z/yldconv.pdf'

# Inputs and printed cells were checked visually against both four-column
# panels on PDF pages1/2. Yields are exact supplied decimal coordinates here.
# y,c,d1,d2,r,s,n,printed_price,target,printed_inverse
CASES = (
    ('0.04445', '8', '4', '4', 14, 182, 33, '145.012268', '135', '0.051620635'),
    ('0.04445', '8', '4', '4', 12, 182, 33, '145.047301', '135', '0.051649111'),
    ('0.04445', '8', '0', '4', 11, 182, 33, '141.070132', '135', '0.04871425'),
    ('0.04445', '8', '4', '4', 183, 183, 32, '141.257676', '135', '0.048856785'),
    ('0.04634', '6.75', '3.375', '3.375', 16, 181, 11, '113.315543', '108', '0.057009527'),
    ('0.04634', '6.75', '3.375', '3.375', 9, 181, 11, '113.415969', '108', '0.057253361'),
    ('0.04634', '6.75', '0', '3.375', 8, 181, 11, '110.058738', '108', '0.050413308'),
    ('0.04634', '6.75', '3.375', '3.375', 184, 184, 10, '110.170218', '108', '0.050652248'),
)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def decimal_fraction(value):
    return Decimal(value.numerator) / Decimal(value.denominator)


def price(y, c, d1, d2, r, s, n):
    """Individual cash-flow powers; this is not the geometric quotient."""
    logarithm = (1 + y / 2).ln()
    alpha = Decimal(r) / s
    if n == 0:
        return (d1 + 100) * (-alpha * logarithm).exp()
    terms = [d1 * (-alpha * logarithm).exp(),
             d2 * (-(alpha + 1) * logarithm).exp()]
    terms += [(c / 2) * (-(alpha + j) * logarithm).exp()
              for j in range(2, n + 1)]
    terms.append(100 * (-(alpha + n) * logarithm).exp())
    return sum(terms, Decimal(0))


def compact_price(y, c, d1, d2, r, s, n):
    """Separate geometric expression, with its removable zero-yield limit."""
    v = 1 / (1 + y / 2)
    factor = (Decimal(r) / s * v.ln()).exp()
    if n == 0:
        return factor * (d1 + 100)
    annuity = Decimal(n - 1) if not y else (1 - v ** (n - 1)) / (1 - v)
    return factor * (d1 + d2 * v + (c / 2) * v * v * annuity + 100 * v ** n)


def displayed(value, places):
    return value.quantize(Decimal(1).scaleb(-places), rounding=ROUND_HALF_UP)


def displayed_fraction(value, places):
    # Source rational cells have small integer operands, and are far from a
    # displayed midpoint. Keep all such divisions inside an explicit context.
    with localcontext() as context:
        context.prec = 80
        return displayed(decimal_fraction(value), places)


def panels(precision):
    results = []
    with localcontext() as context:
        context.prec = precision
        for index, row in enumerate(CASES, 1):
            y, c, d1, d2 = map(Decimal, row[:4])
            r, s, n, expected, target, inverse = row[4:]
            target = Decimal(target)
            p = price(y, c, d1, d2, r, s, n)
            assert displayed(p, 6) == Decimal(expected), (index, p, expected)
            alternate = compact_price(y, c, d1, d2, r, s, n)
            assert abs(p - alternate) < Decimal(10) ** (-precision + 20)
            lower, upper = Decimal('-.1'), Decimal('.2')
            assert price(lower, c, d1, d2, r, s, n) > target
            assert price(upper, c, d1, d2, r, s, n) < target
            # 340 steps bound the bracket width by .3*2^-340 (<1e-102).
            for _ in range(340):
                middle = (lower + upper) / 2
                if price(middle, c, d1, d2, r, s, n) > target:
                    lower = middle
                else:
                    upper = middle
            assert displayed(lower, 9) == displayed(upper, 9) == Decimal(inverse)
            # Stable source comparison plus a richer independent 60dp anchor.
            results.append({'id': index, 'price_60dp': str(displayed(p, 60)),
                            'inverse_60dp': str(displayed((lower + upper) / 2, 60))})
        for n in (0, 1, 2, 33):
            assert price(Decimal(0), Decimal(8), Decimal(4), Decimal(4), 1, 2, n) == 100 + 4 * (n + 1)
            assert compact_price(Decimal(0), Decimal(8), Decimal(4), Decimal(4), 1, 2, n) == 100 + 4 * (n + 1)
    return results


def long_first_ai(settlement):
    issue, skipped, payment = date(1999, 5, 7), date(1999, 6, 7), date(1999, 12, 7)
    if settlement == payment:
        return Fraction(0)
    if settlement <= skipped:
        return 3 * Fraction((settlement - issue).days, 182)
    elapsed = Fraction((settlement - skipped).days, 183)
    # The source table transitions after1999-11-26. This reads its explicit
    # historical ex state; it does not generate a legal UK business calendar.
    return 3 * (elapsed - 1 if settlement > date(1999, 11, 26)
                else Fraction(31, 182) + elapsed)


def daily_rows(text):
    page = text.split('--- PDF PAGE 5 ---\n', 1)[1]
    matches = re.findall(r'(\d{2}-[A-Za-z]{3}-99)\s+(\d+)\s+(-?\d+(?:\.\d+)?)', page)
    seen = {}
    for label, time, printed in matches:
        settlement = datetime.strptime(label, '%d-%b-%y').date()
        assert settlement not in seen, label
        expected_time = ((settlement - date(1999, 5, 7)).days
                         if settlement <= date(1999, 6, 7)
                         else (settlement - date(1999, 6, 7)).days)
        assert int(time) == expected_time, (label, time, expected_time)
        exact = long_first_ai(settlement)
        # The ex tail uses5dp; other rows use6dp, stripping trailing zeroes.
        places = 5 if exact < 0 else 6
        assert displayed_fraction(exact, places) == Decimal(printed), (label, exact, printed)
        seen[settlement] = {'time': int(time), 'printed': printed, 'places': places,
                            'exact': str(exact)}
    expected_dates = {date(1999, 5, 8) + timedelta(days=i) for i in range(214)}
    assert set(seen) == expected_dates, (len(seen), expected_dates - set(seen))
    for settlement, printed in ((date(1999, 5, 29), '.362637'),
                                (date(1999, 9, 7), '2.019186'),
                                (date(1999, 12, 6), '-.016393')):
        assert displayed_fraction(long_first_ai(settlement), 6) == Decimal(printed)
    dividend = 3 * (1 + Fraction(31, 182))
    assert displayed_fraction(dividend, 6) == Decimal('3.510989')
    return seen, dividend - Fraction('3.510989')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-pdf', required=True, type=Path)
    parser.add_argument('--source-text', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path, help='new scratch JSON file; never overwritten')
    args = parser.parse_args()
    assert sha(args.source_pdf.read_bytes()) == PDF_SHA
    raw = args.source_text.read_bytes()
    assert sha(raw) == TEXT_SHA
    low, high = panels(160), panels(240)
    assert low == high, 'independent60dp source anchors disagree between precisions'
    daily, rounding_gap = daily_rows(raw.decode())
    evidence = {'scope': 'pre-contract primary-source comparison; no Rust/QuantLib/fixture',
                'source_url': SOURCE_URL, 'source_pdf_sha256': PDF_SHA,
                'source_text_sha256': TEXT_SHA, 'generator_sha256': sha(Path(__file__).read_bytes()),
                'generated_at': datetime.now(timezone.utc).isoformat(),
                'decimal_precisions': [160, 240], 'price_cases': 8, 'inverse_cases': 8,
                'daily_ai_cases': len(daily), 'separate_ai_anchors': 3,
                'first_dividend_cases': 1, 'zero_yield_checks': 8,
                'dividend_rounding_gap_exact': str(rounding_gap), 'anchors': high,
                'daily_ai': {str(k): v for k, v in sorted(daily.items())}, 'result': 'passed'}
    with args.output.open('x') as stream:
        json.dump(evidence, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k: evidence[k] for k in ('result', 'price_cases', 'inverse_cases', 'daily_ai_cases', 'first_dividend_cases')}))


if __name__ == '__main__':
    main()
