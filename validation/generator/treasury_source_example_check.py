#!/usr/bin/env python3
"""Pre-contract primary-source checks of US Treasury fixed-coupon examples.

Reads only the pinned official eCFR XML of 31 CFR Part 356 and checks the
fixed-principal ("non-indexed") material of Appendix B, Formulas and Tables:
section I.A (coupons, Tables 1 and 2), section I.D (accrued interest) and
section II A-G (yield-to-price worked examples). Exact Python Fraction
arithmetic is authoritative; Decimal at two precisions is a separately
decoded second route. Printed cells are parsed from the XML text itself.

This adopts no market contract or API, generates no Rust fixture, reads no
SabiQuant production code and no QuantLib implementation, and writes one new
JSON report (never overwritten). Hard failures are limited to pins, parse
completeness and facts the source itself reproduces; staging, tie-rule and
clean-price alternatives are reported, not chosen.

Primary source (byte-identical 2026-10-15 and 2026-10-16 versions):
https://www.ecfr.gov/api/versioner/v1/full/2026-10-16/title-31.xml?part=356
"""

import argparse
from datetime import date, datetime, timezone
from decimal import Decimal, ROUND_HALF_EVEN, ROUND_HALF_UP, localcontext
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import platform
import re
import sys
import xml.etree.ElementTree as ElementTree

XML_SHA = '35fa6cb3febd8807c06b08e73d1f311a472b31f127796c6ef369237915b6699d'
SOURCE_URL = 'https://www.ecfr.gov/api/versioner/v1/full/2026-10-16/title-31.xml?part=356'
PRECISIONS = (160, 240)
MODES = ('half_away', 'half_even')  # "normal rounding procedures" leaves ties open
BLOCKS = {'HEAD', 'HD1', 'HD2', 'HD3', 'P', 'FP', 'FP-1', 'FP-2', 'EXAMPLE'}
MONTHS = {name: number for number, name in enumerate(
    ('January', 'February', 'March', 'April', 'May', 'June', 'July', 'August',
     'September', 'October', 'November', 'December'), 1)}


def sha(data):
    return hashlib.sha256(data).hexdigest()


# --------------------------------------------------------------------------
# Exact rounding helpers. Ties are reported under both candidate rules.

def rounded(value, places, mode):
    """Rounds an exact rational to `places` decimals; returns (result, tie)."""
    scale = 10 ** places
    magnitude = abs(value) * scale
    floor = magnitude.numerator // magnitude.denominator
    remainder = magnitude - floor
    tie = remainder == Fraction(1, 2)
    if remainder > Fraction(1, 2) or (tie and mode == 'half_away'):
        floor += 1
    elif tie and mode == 'half_even' and floor % 2:
        floor += 1
    result = Fraction(floor, scale)
    return (-result if value < 0 else result), tie


def places_of(text):
    return len(text.split('.', 1)[1]) if '.' in text else 0


def decimal_text(value, places):
    """Exact half-away display used only for reporting exact values."""
    with localcontext() as context:
        context.prec = 200
        return str(to_decimal(rounded(value, places, 'half_away')[0], 200).quantize(
            Decimal(1).scaleb(-places)))


def to_decimal(value, precision):
    with localcontext() as context:
        context.prec = precision
        return Decimal(value.numerator) / Decimal(value.denominator)


def matches_printed(printed, value):
    """Printed-cell agreement of an exact value under each tie rule."""
    places = places_of(printed)
    target = Fraction(printed)
    results = {mode: rounded(value, places, mode) for mode in MODES}
    return {'tie': results['half_away'][1],
            'half_away': results['half_away'][0] == target,
            'half_even': results['half_even'][0] == target}


def self_test_rounding():
    half = Fraction(5, 10 ** 6)
    assert rounded(half, 5, 'half_away') == (Fraction(1, 10 ** 5), True)
    assert rounded(half, 5, 'half_even') == (Fraction(0), True)
    assert rounded(-half, 5, 'half_away') == (Fraction(-1, 10 ** 5), True)
    assert rounded(Fraction(15, 10 ** 6), 5, 'half_even') == (Fraction(2, 10 ** 5), True)
    assert rounded(Fraction(1, 3), 2, 'half_even') == (Fraction(33, 100), False)
    assert rounded(Fraction(2, 3), 2, 'half_away') == (Fraction(67, 100), False)


# --------------------------------------------------------------------------
# XML access: Appendix B blocks and tables, in document order.

def text_of(element):
    return ' '.join(''.join(element.itertext()).split())


def appendix_b(root):
    for element in root.iter('DIV9'):
        if element.get('N') == 'Appendix B to Part 356':
            return element
    raise SystemExit('Appendix B to Part 356 not found')


def blocks(element):
    """Yields (tag, normalized text) for block elements, never nested twice."""
    for child in element:
        if child.tag == 'TABLE':
            continue
        if child.tag in BLOCKS:
            yield child.tag, text_of(child)
        else:
            yield from blocks(child)


def table(appendix, title, headnote_prefix=None):
    found = []
    for element in appendix.iter('TABLE'):
        caption = element.find('CAPTION')
        if caption is None:
            continue
        parts = [text_of(p) for p in caption.findall('P')]
        if parts and parts[0] == title and (
                headnote_prefix is None or any(p.startswith(headnote_prefix) for p in parts[1:])):
            found.append(element)
    assert len(found) == 1, (title, len(found))
    return found[0]


def body_rows(element):
    return [row.findall('TD') for row in element.find('TBODY').findall('TR')]


# --------------------------------------------------------------------------
# Section I.A, Tables 1 and 2.

def check_table_1(appendix):
    """Table 1 day counts by Gregorian arithmetic for every row and column."""
    rows = body_rows(table(appendix, 'Table 1'))
    assert len(rows) == 12, len(rows)
    cells = 0
    for cells_in_row in rows:
        label = text_of(cells_in_row[0])
        start_name, end_name = label.split(' to ')
        first, last = MONTHS[start_name], MONTHS[end_name]
        assert (last - first) % 12 == 6, label
        printed = [int(text_of(cell)) for cell in cells_in_row[1:]]
        assert len(printed) == 4, label
        computed = []
        for day_rule in ('1st or 15th', 'last day'):
            for leap in (False, True):
                # The year whose February the half-year contains, if any.
                february = 2024 if leap else 2023
                start_year = february if first <= 2 else february - 1 if first >= 8 else february
                end_year = start_year + (1 if last < first else 0)
                if day_rule == 'last day':
                    start = last_day(start_year, first)
                    end = last_day(end_year, last)
                    computed.append((end - start).days)
                else:
                    counts = {(date(end_year, last, d) - date(start_year, first, d)).days
                              for d in (1, 15)}
                    assert len(counts) == 1, (label, counts)
                    computed.append(counts.pop())
        assert computed == printed, (label, computed, printed)
        cells += 4
    return {'rows': len(rows), 'cells': cells, 'result': 'all match'}


def last_day(year, month):
    following = date(year + month // 12, month % 12 + 1, 1)
    return date.fromordinal(following.toordinal() - 1)


def table_2(appendix):
    """Parses Table 2 into {(eighths, days): printed text}."""
    element = table(appendix, 'Table 2', "[Decimal for one day's interest on $1,000")
    headers = [text_of(th) for th in element.find('THEAD').iter('TH')]
    days = [int(re.fullmatch(r'Half-year of (\d+) days', h).group(1)) for h in headers[1:]]
    assert days == [184, 183, 182, 181], days
    cells, order = {}, []
    for row in body_rows(element):
        label = row[0]
        whole = int((label.text or '0').strip() or 0)
        sup, sub = label.find('sup'), label.find('sub')
        rate = Fraction(whole) + (Fraction(int(sup.text), int(sub.text)) if sup is not None else 0)
        eighths = rate * 8
        assert eighths.denominator == 1, rate
        order.append(int(eighths))
        printed = [text_of(cell) for cell in row[1:]]
        assert len(printed) == 4 and all(re.fullmatch(r'0\.\d{9}', p) for p in printed), printed
        for count, text in zip(days, printed):
            cells[(int(eighths), count)] = text
    assert order == list(range(1, 161)), 'Table 2 rates are not 1/8..20 in 1/8 steps'
    return cells


def daily_exact(eighths, days):
    """One day's interest on $1,000: ($1,000 * rate/100) / 2 / days."""
    return Fraction(1000) * Fraction(eighths, 8) / 100 / 2 / days


def check_table_2(cells):
    ties = 0
    mismatches = []
    for (eighths, days), printed in sorted(cells.items()):
        exact = daily_exact(eighths, days)
        agreement = matches_printed(printed, exact)
        ties += agreement['tie']
        if not (agreement['half_away'] and agreement['half_even']):
            mismatches.append({'rate_eighths': eighths, 'days': days, 'printed': printed})
        for precision in PRECISIONS:
            with localcontext() as context:
                context.prec = precision
                value = Decimal(5 * eighths) / Decimal(8 * days)
                for rounding in (ROUND_HALF_UP, ROUND_HALF_EVEN):
                    assert str(value.quantize(Decimal('1e-9'), rounding=rounding)) == printed, (
                        eighths, days, precision)
    assert not mismatches, mismatches[:5]
    return {'cells': len(cells), 'rates': 160, 'half_year_lengths': [184, 183, 182, 181],
            'exact_ties_at_9dp': ties, 'mismatches': 0, 'result': 'all 640 match RN9(exact)'}


# --------------------------------------------------------------------------
# Sections I.A and I.D printed coupon, accrued-interest and nominal anchors.

def number(text):
    return Fraction(text.replace(',', ''))


def anchors(all_blocks, cells):
    text = ' '.join(t for _, t in all_blocks)
    found = []

    def expect(pattern):
        match = re.search(pattern, text)
        assert match, pattern
        return match.groups()

    def record(name, printed, staged, raw=None, places=None, note=''):
        printed_value = number(printed)
        entry = {'anchor': name, 'printed': printed, 'staged_exact': str(staged),
                 'staged_equals_printed': staged == printed_value}
        if places is not None:
            entry['places'] = places
            entry['staged_rounding'] = matches_printed(printed.replace(',', ''), staged)
        if raw is not None:
            entry['raw_exact'] = str(raw)
            entry['raw_rounding'] = matches_printed(printed.replace(',', ''), raw)
        if note:
            entry['note'] = note
        found.append(entry)
        return entry

    def table_cell(eighths, days, printed):
        assert cells[(eighths, days)] == printed, (eighths, days, printed)
        return Fraction(printed)

    # I.A.1: ($1,000 x .08)/2 = $40.
    (regular,) = expect(r'\(\$1,000 × \.08\)/2 = \$(\d+)\.')
    assert Fraction(1000) * Fraction('.08') / 2 == Fraction(regular)
    record('I.A.1 regular coupon per $1,000 at 8%', regular, Fraction(1000) * Fraction('.08') / 2)

    # I.A.3 short first coupon: 8 3/8%, July 2 -> December 31, 1990 in a 184-day half-year.
    full, accrued, daily, payment, scaled, cents = expect(
        r'8 3/8% interest was issued on July 2, 1990.*?was (\d+) \(See Table 1\.\)\. The number of days '
        r'for which interest actually accrued was (\d+) .*?\$(0\.\d{9}) \(See Table 2.*?payment of '
        r'\$([\d.]+) per \$1,000\. For \$20,000.*?payment of \$([\d.]+) \(\$([\d.]+)\)')
    assert int(full) == (date(1990, 12, 31) - date(1990, 6, 30)).days
    assert int(accrued) == (date(1990, 12, 31) - date(1990, 7, 2)).days
    decimal_day = table_cell(67, int(full), daily)
    raw = daily_exact(67, int(full)) * int(accrued)
    record('I.A.3 short coupon per $1,000', payment, decimal_day * int(accrued), raw, 9,
           'printed equals the nine-place Table 2 decimal times days, not RN9 of the exact ratio')
    record('I.A.3 coupon for $20,000 (unrounded)', scaled, decimal_day * int(accrued) * 20, raw * 20)
    record('I.A.3 coupon for $20,000 (cents)', cents, decimal_day * int(accrued) * 20, raw * 20, 2,
           'display to cents; the staged value rounds, it is not exact')

    # I.A.4 long first coupon: 7 7/8%, fractional December 3, 1990 -> February 15, 1991.
    regular_part, half_year, daily, days, fraction_part, total, cents, scaled, scaled_cents = expect(
        r'7 7/8% interest was issued on December 3, 1990.*?computed to be \$([\d.]+) per \$1,000.*?'
        r'fell in a (\d+)-day half-year \(August 15, 1990, to February 15, 1991\).*?7 7/8% was '
        r'\$(0\.\d{9})\. This decimal, multiplied by (\d+) .*?fractional portion of \$([\d.]+)\. .*?'
        r'payment of \$([\d.]+), or \$([\d.]+) per \$1,000.*?multiplied by 7, resulting in an interest '
        r'payment of \$([\d.]+) \(\$([\d.]+)\)')
    assert int(half_year) == (date(1991, 2, 15) - date(1990, 8, 15)).days
    assert int(days) == (date(1991, 2, 15) - date(1990, 12, 3)).days
    regular_exact = Fraction(1000) * Fraction(63, 8) / 100 / 2
    record('I.A.4 regular portion per $1,000', regular_part, regular_exact)
    decimal_day = table_cell(63, int(half_year), daily)
    staged_fraction = decimal_day * int(days)
    raw_fraction = daily_exact(63, int(half_year)) * int(days)
    record('I.A.4 fractional portion per $1,000', fraction_part, staged_fraction, raw_fraction, 9)
    record('I.A.4 long coupon per $1,000', total, regular_exact + staged_fraction,
           regular_exact + raw_fraction, 9)
    record('I.A.4 long coupon per $1,000 (cents)', cents, regular_exact + staged_fraction,
           regular_exact + raw_fraction, 2)
    record('I.A.4 coupon for $7,000 (unrounded)', scaled, (regular_exact + staged_fraction) * 7,
           (regular_exact + raw_fraction) * 7)
    record('I.A.4 coupon for $7,000 (cents)', scaled_cents, (regular_exact + staged_fraction) * 7,
           (regular_exact + raw_fraction) * 7, 2)

    # I.D example (1): 6 3/4%, May 15 -> August 15, 2000 in May 15 -> November 15, 2000.
    days, half_year, daily, product, rounded5, par, payable = expect(
        r'6 3/4%, originally issued on May 15, 2000.*?accrued for (\d+) days, from May 15 to August 15\. '
        r'The regular interest period from May 15 to November 15, 2000, covered (\d+) days\. Accordingly, '
        r'the daily interest decimal, \$(0\.\d{9}), multiplied by 92, resulted in accrued interest payable '
        r'of \$([\d.]+), or \$([\d.]+), for each \$1,000.*?then (\d+) is multiplied by \$16\.87500, '
        r'resulting in an amount payable of \$([\d,.]+)\.')
    assert int(days) == (date(2000, 8, 15) - date(2000, 5, 15)).days
    assert int(half_year) == (date(2000, 11, 15) - date(2000, 5, 15)).days
    decimal_day = table_cell(54, int(half_year), daily)
    staged = decimal_day * int(days)
    raw = daily_exact(54, int(half_year)) * int(days)
    record('I.D(1) accrued per $1,000 (unrounded)', product, staged, raw)
    record('I.D(1) accrued per $1,000 (5 places)', rounded5, staged, raw, 5)
    record('I.D(1) payable for $150,000', payable, Fraction(rounded5) * int(par))

    # I.D example (2): 10 3/4%, two half-years.
    first_days, first_half, second_days, second_half, first_daily, second_daily, first_product, \
        second_product, total, rounded5, payable, cents = expect(
            r'10 3/4% bond, originally issued on July 2, 1985.*?accrued for (\d+) days, from July 2 to '
            r'August 15, 1985, during a (\d+)-day half-year \(February 15 to August 15\); and for (\d+) '
            r'days, from August 15 to November 4, during a (\d+)-day half-year.*?Accordingly, '
            r'\$(0\.\d{9}) was multiplied by 44, and \$(0\.\d{9}) was multiplied by 81, resulting in '
            r'products of \$([\d.]+) and \$([\d.]+) which, added together, resulted in accrued interest '
            r'payable of \$([\d.]+), or \$([\d.]+), for each \$1,000.*?resulting in an amount payable of '
            r'\$([\d.]+) \(\$([\d.]+)\)')
    assert int(first_days) == (date(1985, 8, 15) - date(1985, 7, 2)).days
    assert int(first_half) == (date(1985, 8, 15) - date(1985, 2, 15)).days
    assert int(second_days) == (date(1985, 11, 4) - date(1985, 8, 15)).days
    assert int(second_half) == (date(1986, 2, 15) - date(1985, 8, 15)).days
    first_decimal = table_cell(86, int(first_half), first_daily)
    second_decimal = table_cell(86, int(second_half), second_daily)
    first_raw = daily_exact(86, int(first_half)) * int(first_days)
    second_raw = daily_exact(86, int(second_half)) * int(second_days)
    record('I.D(2) first half-year product', first_product, first_decimal * int(first_days), first_raw)
    record('I.D(2) second half-year product', second_product, second_decimal * int(second_days),
           second_raw)
    staged_total = first_decimal * int(first_days) + second_decimal * int(second_days)
    record('I.D(2) accrued per $1,000 (unrounded)', total, staged_total, first_raw + second_raw)
    record('I.D(2) accrued per $1,000 (5 places)', rounded5, staged_total, first_raw + second_raw, 5)
    record('I.D(2) payable for $11,000 (unrounded)', payable, Fraction(rounded5) * 11)
    record('I.D(2) payable for $11,000 (cents)', cents, Fraction(rounded5) * 11, places=2)

    failures = [a['anchor'] for a in found
                if not (a['staged_equals_printed'] or a.get('staged_rounding', {}).get('half_away'))]
    assert not failures, failures
    return found


# --------------------------------------------------------------------------
# Section II A-G worked examples.

# Statement facts (dates for r/s/n checks). Each phrase is asserted verbatim in
# the example's own XML paragraph before use; nothing here is a printed price.
EXAMPLES = {
    'A': {'kind': 'regular', 'phrase': 'issued May 15, 1990, due May 15, 2020',
          'issue': date(1990, 5, 15), 'maturity': date(2020, 5, 15), 'coupons': ((5, 15), (11, 15))},
    'B': {'kind': 'short', 'phrase': 'issued April 2, 1990, due March 31, 1992',
          'issue': date(1990, 4, 2), 'maturity': date(1992, 3, 31), 'coupons': ((3, 31), (9, 30))},
    'C': {'kind': 'long', 'phrase': 'issued March 1, 1990, due May 15, 1995',
          'issue': date(1990, 3, 1), 'maturity': date(1995, 5, 15), 'coupons': ((5, 15), (11, 15))},
    'D': {'kind': 'reopen-regular', 'phrase': 'issued November 29, 1985, due November 15, 1995',
          'issue': date(1985, 11, 29), 'maturity': date(1995, 11, 15), 'coupons': ((5, 15), (11, 15))},
    'E': {'kind': 'reopen-long-regular', 'phrase': 'reopened on November 4, 1985',
          'issue': date(1985, 11, 4), 'maturity': date(2005, 8, 15), 'coupons': ((2, 15), (8, 15))},
    'F': {'kind': 'reopen-short', 'phrase': 'reopened on August 15, 1983',
          'issue': date(1983, 8, 15), 'maturity': date(1991, 5, 15), 'coupons': ((5, 15), (11, 15))},
    'G': {'kind': 'reopen-long-fraction', 'phrase': 'reopened on November 15, 1988',
          'issue': date(1988, 11, 15), 'maturity': date(1994, 12, 15), 'coupons': ((6, 15), (12, 15))},
}
MATURITY_PHRASE = {'E': 'due August 15, 2005', 'F': 'due May 15, 1991', 'G': 'due December 15, 1994'}


def section_ii(all_blocks):
    start = next(k for k, (tag, t) in enumerate(all_blocks)
                 if tag == 'HD1' and t.startswith('II. Formulas for Conversion of Non-indexed'))
    end = next(k for k in range(start + 1, len(all_blocks)) if all_blocks[k][0] == 'HD1')
    section = all_blocks[start:end]
    starts = [k for k, (tag, t) in enumerate(section)
              if tag == 'P' and re.match(r'[A-G]\. (\(1\) )?For (non-indexed|new)', t)]
    letters = [section[k][1][0] for k in starts]
    assert letters == list('ABCDEFG'), letters
    bounds = starts + [len(section)]
    examples = {section[a][1][0]: [t for _, t in section[a:b]] for a, b in zip(bounds, bounds[1:])}
    return [t for _, t in section[:starts[0]]], examples


def one(lines, pattern, required=True):
    found = [m for m in (re.fullmatch(pattern, line) for line in lines) if m]
    if not required and not found:
        return None
    assert len(found) == 1, (pattern, len(found))
    return found[0]


def span_days(phrase):
    match = re.fullmatch(r'(\w+) (\d+)(?:, (\d{4}))?,? to (\w+) (\d+), (\d{4})(?:,.*)?', phrase)
    assert match, phrase
    first_month, first_day, first_year, last_month, last_day_, last_year = match.groups()
    first_year = int(first_year or last_year)
    start = date(first_year, MONTHS[first_month], int(first_day))
    end = date(int(last_year), MONTHS[last_month], int(last_day_))
    return (end - start).days


def coupon_count(facts):
    """Coupon frequency dates in (issue, maturity]; the source n is this minus 1."""
    count = 0
    for year in range(facts['issue'].year, facts['maturity'].year + 1):
        for month, day in facts['coupons']:
            if facts['issue'] < date(year, month, day) <= facts['maturity']:
                count += 1
    return count


def parse_example(letter, lines):
    facts = EXAMPLES[letter]
    statement = lines[lines.index('Example:') + 1]
    assert facts['phrase'] in statement, (letter, statement)
    if letter in MATURITY_PHRASE:
        assert MATURITY_PHRASE[letter] in statement, (letter, statement)
    coupon_text, yield_text = re.search(r'(\d+ \d/\d|\d+)% .*?yield of ([\d.]+)%', statement).groups()
    whole, _, fraction = coupon_text.partition(' ')
    statement_coupon = Fraction(int(whole)) + (Fraction(fraction) if fraction else 0)
    parsed = {'statement': statement}
    parsed['C'] = one(lines, r'C = ([\d.]+)\.').group(1)
    parsed['i'] = one(lines, r'i = (\.\d+)\.').group(1)
    assert Fraction(parsed['C']) == statement_coupon, letter
    assert Fraction(parsed['i']) == Fraction(yield_text) / 100, letter
    parsed['n'] = int(one(lines, r'n = (\d+)(?:\.| \(.*\)\.?)').group(1))
    for key, symbol in (('r', 'r'), ('s', 's'), ('rp', 'r′'), ('spp', 's″')):
        match = one(lines, re.escape(symbol) + r' = (\d+) \((.+)\)\.', required=key in ('r', 's'))
        if match:
            parsed[key] = int(match.group(1))
            assert span_days(match.group(2)) == parsed[key], (letter, key, match.group(2))
    assert coupon_count(facts) - 1 == parsed['n'], (letter, coupon_count(facts), parsed['n'])
    match = one(lines, r'v = 1 / \(1 \+ [\d.]+/2\), or (\.\d+)\.', required=False)
    parsed['v'] = match.group(1) if match else None
    parsed['vn'] = one(lines, r'v n = .*, or (\.\d+)\.?').group(1)
    parsed['an'] = one(lines, r'an⌉ = .*, or ([\d.]+)\.').group(1)
    for key, symbol in (('A', 'A'), ('AIp', 'AI′'), ('AI', 'AI')):
        match = one(lines, re.escape(symbol) + r' = .*, or ([\d.]+)\.', required=False)
        parsed[key] = match.group(1) if match else None
    resolution = lines[lines.index('Resolution:') + 2]
    parsed['resolution'] = resolution
    parsed['vn_resolution'] = re.search(r'100\((\.\d+)\)', resolution).group(1)
    parsed['an_resolution'] = re.search(r'/2\)\((\d+\.\d+)\)', resolution).group(1)
    v_resolution = re.search(r'\]\(?(\.\d+)\)?', resolution)
    parsed['v_resolution'] = v_resolution.group(1) if v_resolution else None
    line1 = one(lines, r'\(1\) (?:P|\(P \+ ([\d.]+)\))\[1 \+ (\.\d+)\] = (.+)\.')
    parsed['A_line'], parsed['rate'] = line1.group(1), line1.group(2)
    parsed['terms'] = line1.group(3).split(' + ')
    line2 = one(lines, r'\(2\) (?:P|\(P \+ [\d.]+\))\[(1\.\d+)\] = ([\d.]+)\.')
    parsed['denominator'], parsed['numerator'] = line2.groups()
    line3 = one(lines, r'\(3\) (?:P|\(P \+ [\d.]+\)) = ([\d.]+) / (1\.\d+)\.')
    assert line3.groups() == (parsed['numerator'], parsed['denominator']), letter
    if letter in 'ABC':
        parsed['P'] = one(lines, r'\(4\) P = ([\d.]+)\.').group(1)
    else:
        line4 = one(lines, r'\(4\) \(P \+ ([\d.]+)\) = ([\d.]+)\.')
        assert line4.group(1) == parsed['A_line'] == parsed['A'], letter
        parsed['dirty'] = line4.group(2)
        line5 = one(lines, r'\(5\) P = ([\d.]+) −\s*(\.?\s*[\d.]+)\.')
        parsed['line5_subtrahend_text'] = line5.group(2)
        assert line5.group(1) == parsed['dirty'], letter
        assert line5.group(2).replace(' ', '') == parsed['A'], letter
        parsed['P'] = one(lines, r'\(6\) P = ([\d.]+)\.').group(1)
    return parsed


def exact_quantities(letter, parsed):
    """Exact rational model of each example's formula; no printed value enters."""
    C, i = Fraction(parsed['C']), Fraction(parsed['i'])
    n, r, s = parsed['n'], parsed['r'], parsed['s']
    half = i / 2
    v = 1 / (1 + half)
    vn = v ** n
    an = sum(v ** k for k in range(1, n + 1))
    assert an == (1 - vn) / half, letter  # summation and closed form agree at i != 0
    c = C / 2
    q = {'v': v, 'vn': vn, 'an': an, 'rate': Fraction(r, s) * half, 'c_an': c * an,
         '100vn': 100 * vn, 'A': Fraction(0)}
    if letter in 'AB':
        q['terms'] = [c * Fraction(r, s), c * an, 100 * vn]
    elif letter == 'C':
        q['terms'] = [c * Fraction(r, s) * v, c * an, 100 * vn]
    elif letter == 'D':
        q['terms'] = [c, c * an, 100 * vn]
        q['A'] = Fraction(s - r, s) * c
    elif letter == 'E':
        rp, spp = parsed['rp'], parsed['spp']
        q['terms'] = [Fraction(rp, spp) * c, c, c * an, 100 * vn]
        q['AIp'], q['AI'] = Fraction(rp, spp) * c, Fraction(s - r, s) * c
        q['A'] = q['AIp'] + q['AI']
    elif letter == 'F':
        q['terms'] = [Fraction(parsed['rp'], s) * c, c * an, 100 * vn]
        q['A'] = Fraction(parsed['rp'] - r, s) * c
    else:
        q['terms'] = [Fraction(parsed['rp'], s) * c * v, c * an, 100 * vn]
        q['A'] = Fraction(parsed['rp'] - r, s) * c
    q['numerator'] = sum(q['terms'])
    q['denominator'] = 1 + q['rate']
    q['D'] = q['numerator'] / q['denominator']  # dirty P+A (= P for A-C)
    return q


def decimal_quantities(letter, parsed, precision):
    """Independent Decimal route: powers, closed-form annuity and accrued.

    All source inputs are decoded directly as Decimal, including the accrued
    components. No rounded printed amount or Fraction result enters this
    route. Arithmetic and subsequent rounding are performed at `precision`.
    """
    with localcontext() as context:
        context.prec = precision
        C, i = Decimal(parsed['C']), Decimal(parsed['i'])
        n, r, s = parsed['n'], Decimal(parsed['r']), Decimal(parsed['s'])
        half = i / 2
        v = 1 / (1 + half)
        vn = v ** n
        an = (1 - vn) / half
        c = C / 2
        rate = r / s * half
        if letter in 'AB':
            terms = [c * r / s, c * an, 100 * vn]
        elif letter == 'C':
            terms = [c * r / s * v, c * an, 100 * vn]
        elif letter == 'D':
            terms = [c, c * an, 100 * vn]
        elif letter == 'E':
            terms = [Decimal(parsed['rp']) / Decimal(parsed['spp']) * c, c, c * an, 100 * vn]
        elif letter == 'F':
            terms = [Decimal(parsed['rp']) / s * c, c * an, 100 * vn]
        else:
            terms = [Decimal(parsed['rp']) / s * c * v, c * an, 100 * vn]
        numerator = sum(terms, Decimal(0))
        accrued = Decimal(0)
        components = {}
        if letter == 'D':
            accrued = (s - r) / s * c
        elif letter == 'E':
            components['AIp'] = Decimal(parsed['rp']) / Decimal(parsed['spp']) * c
            components['AI'] = (s - r) / s * c
            accrued = components['AIp'] + components['AI']
        elif letter in 'FG':
            accrued = (Decimal(parsed['rp']) - r) / s * c
        return dict({'vn': vn, 'an': an, 'numerator': numerator, 'D': numerator / (1 + rate),
                     'A': accrued, 'terms': terms}, **components)


def trace_cell(label, printed, exact, chained=None):
    cell = {'cell': label, 'printed': printed, 'places': places_of(printed),
            'exact': decimal_text(exact, places_of(printed) + 6),
            'exact_model': matches_printed(printed, exact)}
    if chained is not None:
        cell['chained_model'] = matches_printed(printed, chained)
    models = []
    if cell['exact_model']['half_away'] or cell['exact_model']['half_even']:
        models.append('exact-rounded')
    if chained is not None and (cell['chained_model']['half_away'] or cell['chained_model']['half_even']):
        models.append('chained-from-printed')
    cell['models'] = models
    if not models:
        # Diagnostic only: coarser roundings whose value equals the printed
        # digits (a trailing-zero display), never promoted to a model.
        target = Fraction(printed)
        candidates = [('exact', exact)] + ([('chained', chained)] if chained is not None else [])
        cell['coarser_matches'] = [
            {'model': name, 'places': k, 'rule': mode}
            for name, value in candidates for k in range(places_of(printed))
            for mode in MODES if rounded(value, k, mode)[0] == target]
    return cell


def check_example(letter, parsed):
    q = exact_quantities(letter, parsed)
    F = Fraction
    c = F(parsed['C']) / 2
    cells = []
    if parsed['v'] is not None:
        cells.append(trace_cell('v', parsed['v'], q['v']))
    cells.append(trace_cell('v^n (definition line)', parsed['vn'], q['vn']))
    cells.append(trace_cell('v^n (resolution line)', parsed['vn_resolution'], q['vn']))
    cells.append(trace_cell('a_n (definition line)', parsed['an'], q['an'],
                            (1 - F(parsed['vn'])) / (F(parsed['i']) / 2)))
    cells.append(trace_cell('(r/s)(i/2)', parsed['rate'], q['rate']))
    cells.append(trace_cell('1+(r/s)(i/2)', parsed['denominator'], q['denominator'],
                            1 + F(parsed['rate'])))
    chained_terms = {'c_an': c * F(parsed['an_resolution']), '100vn': 100 * F(parsed['vn_resolution'])}
    term_labels = {
        'A': ['(C/2)(r/s)', '(C/2)a_n', '100v^n'], 'B': ['(C/2)(r/s)', '(C/2)a_n', '100v^n'],
        'C': ['[(C/2)(r/s)]v', '(C/2)a_n', '100v^n'], 'D': ['C/2', '(C/2)a_n', '100v^n'],
        'E': ["(r'/s'')(C/2)", 'C/2', '(C/2)a_n', '100v^n'], 'F': ["(r'/s)(C/2)", '(C/2)a_n', '100v^n'],
        'G': ["[(r'/s)(C/2)]v", '(C/2)a_n', '100v^n']}[letter]
    assert len(parsed['terms']) == len(term_labels), letter
    for label, printed, exact in zip(term_labels, parsed['terms'], q['terms']):
        chained = None
        if label == '(C/2)a_n':
            chained = chained_terms['c_an']
        elif label == '100v^n':
            chained = chained_terms['100vn']
        elif label.endswith(']v') and parsed['v_resolution']:
            chained = (exact / q['v']) * F(parsed['v_resolution'])
        cells.append(trace_cell(label, printed, exact, chained))
    cells.append(trace_cell('numerator (2)', parsed['numerator'], q['numerator'],
                            sum(F(t) for t in parsed['terms'])))
    printed_ratio = F(parsed['numerator']) / F(parsed['denominator'])
    result = {'letter': letter, 'kind': EXAMPLES[letter]['kind'], 'statement': parsed['statement'],
              'inputs': {k: parsed.get(k) for k in ('C', 'i', 'n', 'r', 's', 'rp', 'spp')}}
    if letter in 'ABC':
        cells.append(trace_cell('P (4)', parsed['P'], q['D'], printed_ratio))
        price = matches_printed(parsed['P'], q['D'])
        assert price['half_away'] and price['half_even'] and not price['tie'], (letter, price)
        result['price'] = {'printed': parsed['P'], 'exact_P': str(q['D']),
                           'exact_P_40dp': decimal_text(q['D'], 40), 'RN6_exact_matches': True}
    else:
        if letter == 'E':
            ai_p = trace_cell("AI'", parsed['AIp'], q['AIp'])
            ai = trace_cell('AI', parsed['AI'], q['AI'])
            cells += [ai_p, ai]
            component = F(parsed['AIp']) + F(parsed['AI'])
            cells.append(trace_cell('A = AI\' + AI', parsed['A'], q['A'], component))
            result['component_versus_total_AI'] = {
                'printed_A': parsed['A'], 'sum_of_printed_components': str(component),
                'RN6_raw_total_half_away': str(rounded(q['A'], 6, 'half_away')[0]),
                'agree_in_this_example': component == rounded(q['A'], 6, 'half_away')[0]
                == rounded(q['A'], 6, 'half_even')[0] == F(parsed['A']),
                'note': 'one example cannot establish a mandatory component-rounding rule'}
        else:
            cells.append(trace_cell('A', parsed['A'], q['A']))
        cells.append(trace_cell('P+A (4)', parsed['dirty'], q['D'], printed_ratio))
        a_pub = {mode: rounded(q['A'], 6, mode)[0] for mode in MODES}
        d_pub = {mode: rounded(q['D'], 6, mode)[0] for mode in MODES}
        printed_p = F(parsed['P'])
        assert F(parsed['dirty']) - F(parsed['A']) == printed_p, letter
        candidates = {}
        for mode in MODES:
            candidates[mode] = {
                'RN6(D-Araw)': str(rounded(q['D'] - q['A'], 6, mode)[0]),
                'RN6(D-Apub)': str(rounded(q['D'] - a_pub[mode], 6, mode)[0]),
                'RN6(D)-Apub': str(d_pub[mode] - a_pub[mode])}
        matching = sorted({name for mode in MODES for name, value in candidates[mode].items()
                           if F(value) == printed_p})
        trace_ok = all(d_pub[m] == F(parsed['dirty']) and a_pub[m] == F(parsed['A']) for m in MODES)
        assert trace_ok and 'RN6(D)-Apub' in matching, (letter, candidates)
        result['clean'] = {
            'printed_P': parsed['P'], 'printed_dirty': parsed['dirty'], 'printed_A': parsed['A'],
            'D_raw': str(q['D']), 'D_raw_40dp': decimal_text(q['D'], 40),
            'A_raw': str(q['A']), 'A_raw_40dp': decimal_text(q['A'], 40),
            'A_pub_6_per_100': str(a_pub['half_away']),
            'A_pub_5_per_1000': str(a_pub['half_away'] * 10),
            'candidates': candidates, 'candidates_matching_printed': matching,
            'candidates_all_equal': len({v for m in MODES for v in candidates[m].values()}) == 1,
            'source_printed_trace': 'RN6(D) then subtract published A',
            'line5_subtrahend_text': parsed['line5_subtrahend_text']}
    precisions = {}
    for precision in PRECISIONS:
        # Keep the independently decoded arithmetic, subtractions and all
        # quantizations inside the same explicitly chosen high-precision
        # context; the process-default 28 digits are not used here.
        with localcontext() as context:
            context.prec = precision
            d = decimal_quantities(letter, parsed, precision)
            tolerance = Fraction(1, 10 ** (precision - 25))
            checked = ['vn', 'an', 'numerator', 'D', 'A'] + (['AIp', 'AI'] if letter == 'E' else [])
            for key in checked:
                exact = q[key]
                approx = Fraction(d[key])
                assert abs(approx - exact) <= tolerance * max(1, abs(exact)), (letter, key, precision)
            routes = {}
            for mode, rule in (('half_away', ROUND_HALF_UP), ('half_even', ROUND_HALF_EVEN)):
                dirty = d['D'].quantize(Decimal('1e-6'), rounding=rule)
                accrued = d['A'].quantize(Decimal('1e-6'), rounding=rule)
                exact_dirty = rounded(q['D'], 6, mode)[0]
                exact_accrued = rounded(q['A'], 6, mode)[0]
                assert Fraction(dirty) == exact_dirty, (letter, precision, mode, 'D')
                assert Fraction(accrued) == exact_accrued, (letter, precision, mode, 'A')
                candidates = {
                    'RN6(D-Araw)': (d['D'] - d['A']).quantize(Decimal('1e-6'), rounding=rule),
                    'RN6(D-Apub)': (d['D'] - accrued).quantize(Decimal('1e-6'), rounding=rule),
                    'RN6(D)-Apub': dirty - accrued}
                exact_candidates = {
                    'RN6(D-Araw)': rounded(q['D'] - q['A'], 6, mode)[0],
                    'RN6(D-Apub)': rounded(q['D'] - exact_accrued, 6, mode)[0],
                    'RN6(D)-Apub': exact_dirty - exact_accrued}
                for name, value in candidates.items():
                    assert Fraction(value) == exact_candidates[name], (letter, precision, mode, name)
                assert Fraction(candidates['RN6(D)-Apub']) == Fraction(parsed['P']), (
                    letter, precision, mode, 'source printed P')
                components = {}
                for key in ('AIp', 'AI') if letter == 'E' else ():
                    value = d[key].quantize(Decimal('1e-6'), rounding=rule)
                    assert Fraction(value) == rounded(q[key], 6, mode)[0], (letter, precision, mode, key)
                    assert Fraction(value) == Fraction(parsed[key]), (letter, precision, mode, key, 'printed')
                    components[key] = str(value)
                routes[mode] = {'RN6_D': str(dirty), 'RN6_A': str(accrued),
                                'clean_candidates': {k: str(v) for k, v in candidates.items()},
                                'accrued_components': components,
                                'source_printed_P_matches_RN6_D_minus_Apub': True}
            precisions[str(precision)] = {
                'fraction_comparison_bound': 'abs(error) <= 1e-%d * max(1, abs(exact))' % (precision - 25),
                'quantities_compared_with_fraction': checked, 'rounding_routes': routes}
    result['decimal_routes'] = precisions
    result['trace'] = cells
    result['unexplained_cells'] = [cell['cell'] for cell in cells if not cell['models']]
    result['tie_cells'] = [cell['cell'] for cell in cells if cell['exact_model']['tie']]
    zero = dict(parsed, i='0')
    zero_q = zero_yield(letter, zero)
    result['zero_yield_control'] = zero_q
    return result


def zero_yield(letter, parsed):
    """i = 0 by the source's special case: a_n = n by summation, v = 1."""
    C, n, r, s = Fraction(parsed['C']), parsed['n'], parsed['r'], parsed['s']
    c = C / 2
    an = sum(Fraction(1) for _ in range(n))
    assert an == n
    terms = {'A': [c * Fraction(r, s)], 'B': [c * Fraction(r, s)], 'C': [c * Fraction(r, s)],
             'D': [c], 'E': [Fraction(parsed.get('rp') or 0, parsed.get('spp') or 1) * c, c],
             'F': [Fraction(parsed.get('rp') or 0, s) * c],
             'G': [Fraction(parsed.get('rp') or 0, s) * c]}[letter]
    value = sum(terms) + c * an + 100
    return {'a_n': n, 'price_or_dirty': str(value), 'finite': True}


# --------------------------------------------------------------------------
# Full Table 2 staging sweep and derived controls.

def staging_sweep(cells):
    """All 160 rates x half-years 181..184 x days 1..s: Table 2 decimal x days
    (the worked-example staging) versus the exact ratio, each rounded to the
    I.D.4 five places per $1,000 under both tie rules."""
    counts = {'cells': 0, 'raw_ties': 0, 'staged_ties': 0, 'raw_rule_sensitive': 0,
              'staged_rule_sensitive': 0, 'stage_changes_half_away': 0, 'stage_changes_half_even': 0,
              'stage_changes_either_rule': 0, 'coupon_9dp_differs_half_away': 0,
              'coupon_9dp_differs_half_even': 0, 'coupon_cents_differs_half_away': 0,
              'coupon_cents_differs_half_even': 0}
    max_coupon_gap = Fraction(0)
    examples = {'stage_change': [], 'raw_tie_rule_sensitive': [], 'staged_tie_rule_sensitive': []}
    for eighths in range(1, 161):
        for days in (184, 183, 182, 181):
            decimal_day = Fraction(cells[(eighths, days)])
            exact_day = daily_exact(eighths, days)
            for t in range(1, days + 1):
                counts['cells'] += 1
                raw, staged = exact_day * t, decimal_day * t
                gap = abs(staged - raw)
                max_coupon_gap = max(max_coupon_gap, gap)
                for mode in MODES:
                    # Coupon staging: the nine-place product is printed unrounded,
                    # versus the exact ratio displayed at 9 places or to cents.
                    if staged != rounded(raw, 9, mode)[0]:
                        counts['coupon_9dp_differs_' + mode] += 1
                    if rounded(staged, 2, mode)[0] != rounded(raw, 2, mode)[0]:
                        counts['coupon_cents_differs_' + mode] += 1
                raw5 = {m: rounded(raw, 5, m) for m in MODES}
                staged5 = {m: rounded(staged, 5, m) for m in MODES}
                raw_tie, staged_tie = raw5['half_away'][1], staged5['half_away'][1]
                counts['raw_ties'] += raw_tie
                counts['staged_ties'] += staged_tie
                key = {'rate_eighths': eighths, 'days_in_half_year': days, 'days': t,
                       'raw': str(raw), 'staged': str(staged)}
                if raw5['half_away'][0] != raw5['half_even'][0]:
                    counts['raw_rule_sensitive'] += 1
                    if len(examples['raw_tie_rule_sensitive']) < 5:
                        examples['raw_tie_rule_sensitive'].append(key)
                if staged5['half_away'][0] != staged5['half_even'][0]:
                    counts['staged_rule_sensitive'] += 1
                    if len(examples['staged_tie_rule_sensitive']) < 5:
                        examples['staged_tie_rule_sensitive'].append(key)
                changed = {m: raw5[m][0] != staged5[m][0] for m in MODES}
                counts['stage_changes_half_away'] += changed['half_away']
                counts['stage_changes_half_even'] += changed['half_even']
                if changed['half_away'] or changed['half_even']:
                    counts['stage_changes_either_rule'] += 1
                    if len(examples['stage_change']) < 10:
                        examples['stage_change'].append(dict(key, **{
                            'raw_RN5_' + m: str(raw5[m][0]) for m in MODES}, **{
                            'staged_RN5_' + m: str(staged5[m][0]) for m in MODES}))
    assert counts['cells'] == 160 * (184 + 183 + 182 + 181) == 116800
    counts['max_abs_staged_minus_raw_per_1000'] = str(max_coupon_gap)
    counts['max_abs_staged_minus_raw_per_1000_float'] = float(max_coupon_gap)
    return counts, examples


def decimal_sweep_check(cells, precision):
    """Decimal recomputation of every raw five-place result at one precision."""
    with localcontext() as context:
        context.prec = precision
        quantum = Decimal('1e-5')
        for eighths in range(1, 161):
            for days in (184, 183, 182, 181):
                exact_day = daily_exact(eighths, days)
                for t in range(1, days + 1):
                    value = Decimal(5 * eighths * t) / Decimal(8 * days)
                    for rounding, mode in ((ROUND_HALF_UP, 'half_away'), (ROUND_HALF_EVEN, 'half_even')):
                        assert Fraction(value.quantize(quantum, rounding=rounding)) == rounded(
                            exact_day * t, 5, mode)[0], (eighths, days, t, precision)
    return 'all 116800 raw RN5 results agree with Fraction under both tie rules'


def component_counterexamples():
    """Derived (not source): E-shaped AI with C=10.75, s''=181, s=184. Counts
    reopenings where per-component six-place rounding differs from rounding
    the raw total, showing the staging choice is material in general."""
    c = Fraction(1075, 200)
    disagreements, first = 0, None
    for rp in range(1, 181):
        for r in range(1, 184):
            ai_p, ai = Fraction(rp, 181) * c, Fraction(184 - r, 184) * c
            components = rounded(ai_p, 6, 'half_away')[0] + rounded(ai, 6, 'half_away')[0]
            total = rounded(ai_p + ai, 6, 'half_away')[0]
            if components != total:
                disagreements += 1
                if first is None:
                    first = {"r'": rp, 'r': r, 'components': str(components), 'total': str(total)}
    return {'grid': "r'=1..180, r=1..183, C=10.75, s''=181, s=184", 'cases': 180 * 183,
            'component_vs_total_disagreements': disagreements, 'first': first}


# --------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument('--xml', required=True, type=Path, help='pinned eCFR Part 356 XML')
    parser.add_argument('--report', required=True, type=Path, help='new JSON report; never overwritten')
    args = parser.parse_args()
    raw = args.xml.read_bytes()
    assert sha(raw) == XML_SHA, 'XML is not the pinned 2026-10-16 Part 356 snapshot'
    self_test_rounding()
    appendix = appendix_b(ElementTree.fromstring(raw))
    all_blocks = list(blocks(appendix))
    table1 = check_table_1(appendix)
    cells = table_2(appendix)
    table2 = check_table_2(cells)
    anchor_results = anchors(all_blocks, cells)
    definitions, examples = section_ii(all_blocks)
    special = next(t for t in definitions if t.startswith('Special Case:'))
    assert 'If i = 0, then an⌉ = n.' in special
    typo_headline = '(P + A)[1 + (r/s)(i/2)] = (r′s″)(C/2) + C 2 + (C/2)an⌉ + 100v n.'
    corrected_line = '(P + A)[1 + (r/s)(i/2)] = (r′/s″)(C/2) + C/2 + (C/2)an⌉ + 100v n or'
    assert typo_headline in examples['E'] and corrected_line in examples['E']
    assert "AI′ = (r′/s″)(C/2)," in examples['E']
    results = [check_example(letter, parse_example(letter, examples[letter])) for letter in 'ABCDEFG']
    printed_prices = {r['letter']: (r.get('price') or r['clean'])['printed' if 'price' in r else 'printed_P']
                      for r in results}
    # I.D(2) and II.E describe the same 10 3/4% reopening of November 4, 1985:
    # table-decimal accrued per $1,000 versus exact per-component A per 100.
    staged_id2 = next(a for a in anchor_results if a['anchor'] == 'I.D(2) accrued per $1,000 (5 places)')
    example_e = next(r for r in results if r['letter'] == 'E')
    cross_section = {
        'I.D(2) accrued per $1,000 / 10 equals II.E printed A':
            Fraction(staged_id2['printed']) / 10 == Fraction(example_e['clean']['printed_A'])}
    assert all(cross_section.values()), cross_section
    sweep, sweep_examples = staging_sweep(cells)
    decimal_sweep = {str(p): decimal_sweep_check(cells, p) for p in PRECISIONS}
    trace_cells = [cell for r in results for cell in r['trace']]
    report = {
        'scope': 'pre-contract primary-source comparison; no API/contract/fixture/Rust/QuantLib',
        'source_url': SOURCE_URL, 'xml_path': str(args.xml), 'xml_sha256': XML_SHA,
        'checker_sha256': sha(Path(__file__).read_bytes()),
        'generated_at_utc': datetime.now(timezone.utc).isoformat(timespec='seconds'),
        'platform': platform.platform(), 'python': sys.version.split()[0],
        'decimal_precisions': list(PRECISIONS), 'tie_rules_reported': list(MODES),
        'tie_rule_note': 'Appendix B says "normal rounding procedures"; the tie rule is unresolved',
        'table_1': table1, 'table_2': table2, 'anchors': anchor_results,
        'source_text_facts': {
            'II.E headline as codified': typo_headline,
            'II.E worked resolution line': corrected_line,
            'II.E headline missing divisions': ["r′s″ for r′/s″", 'C 2 for C/2'],
            'zero yield special case present': True},
        'printed_prices': printed_prices, 'examples': results, 'cross_section_checks': cross_section,
        'trace_summary': {
            'cells': len(trace_cells),
            'exact_rounded_without_chained_candidate': sum(
                c['models'] == ['exact-rounded'] and 'chained_model' not in c for c in trace_cells),
            'exact_rounded_only_against_evaluated_chained_candidate': sum(
                c['models'] == ['exact-rounded'] and 'chained_model' in c for c in trace_cells),
            'chained_only': sum(c['models'] == ['chained-from-printed'] for c in trace_cells),
            'both_models': sum(len(c['models']) == 2 for c in trace_cells),
            'unexplained': [(r['letter'], name) for r in results for name in r['unexplained_cells']],
            'exact_model_ties': [(r['letter'], name) for r in results for name in r['tie_cells']],
            'chained_model_ties': [(r['letter'], cell['cell']) for r in results for cell in r['trace']
                                   if cell.get('chained_model', {}).get('tie')]},
        'clean_candidate_disagreements': [r['letter'] for r in results
                                          if 'clean' in r and not r['clean']['candidates_all_equal']],
        'staging_sweep': sweep, 'staging_sweep_examples': sweep_examples,
        'staging_sweep_decimal': decimal_sweep,
        'derived_component_rounding': component_counterexamples(),
        'result': 'passed'}
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2, sort_keys=True)
        stream.write('\n')
    print(json.dumps({key: report[key] for key in (
        'result', 'printed_prices', 'clean_candidate_disagreements', 'trace_summary')}, indent=2))
    print(json.dumps({'table_2': table2['result'], 'staging_sweep': sweep}, indent=2))


if __name__ == '__main__':
    main()
