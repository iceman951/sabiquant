#!/usr/bin/env python3
"""Independent Treasury ideal price/all-six-risk and target-driven root references.

Reads only the adopted contracts/requirements and itself, never Rust or another
Treasury corpus. Fraction algebra is authoritative. Decimal 160/240 per-payment
sums corroborate a different exact finite-geometric/product-rule derivation.
Create-new output only; --check preserves frozen timestamp/platform metadata.
"""
import argparse
from calendar import monthrange
from collections import Counter
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal, localcontext, ROUND_CEILING, ROUND_FLOOR
from fractions import Fraction as F
import hashlib
import json
import math
from pathlib import Path
import platform
import re
import struct

ROOT = Path(__file__).resolve().parents[2]
MAIN = ROOT / 'docs/M8_US_TREASURY_CONTRACT.md'
CASH = ROOT / 'docs/M8_US_TREASURY_CASH_CONTRACT.md'
MAIN_SHA = '50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93'
CASH_SHA = '25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585'
OUT = ROOT / 'validation/fixtures/bonds/treasury-ideal-price-risk-oracle.fixture'
INV = ROOT / 'validation/fixtures/bonds/treasury-ideal-inverse-oracle.fixture'
T = F(1, 1 << 53)
H = F(1, 1 << 1075)
MIN_SUB = 2 * H
MIN_NORMAL = F(1, 1 << 1022)
MAX = F((1 << 53) - 1) * (1 << 971)
OVERFLOW = MAX + (1 << 970)
METRICS = ('dirty', 'clean', 'd1', 'd2', 'moment', 'modified', 'macaulay', 'convexity')
INPUTS = ('dated issue maturity coupon first_coupon eom quasi_count settlement daily_basis tie_rule').split()
COMMON = ['id', 'family'] + INPUTS + ('first_period next_index r s n raw_ai published_ai '
          'coupon_indices source_amounts positive_pv_count source_display requirements').split()
PRICE_COLUMNS = COMMON + ['yield_bits', 'input_fingerprint', 'pv_gate', 'comparison_method',
    'ai_cache_error', 'dirty_radius', 'delta_ideal', 'prefix_ideal', 'delta_rn_bits', 'prefix_rn_bits'] + [
    f'{m}_{p}' for m in METRICS for p in ('ideal', 'ideal_rn_bits', 'ideal_class', 'allowance', 'comparison')]
INV_COLUMNS = COMMON + ('target_kind target_bits dirty_target_bits ideal_dirty_target caller_lo_bits '
    'caller_hi_bits abs_tol_bits rel_tol_bits max_evals input_fingerprint ideal_image inverse_method '
    'root_lo root_hi root_lo_bits root_hi_bits left_sign right_sign bisections uniform_price_error '
    'target_shift_error slope_lower coordinate_allowance conditioned_lo_bits conditioned_hi_bits '
    'comparison residual_scale').split()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{value:016x}'


def bitsof(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def represented(word):
    value = struct.unpack('>d', struct.pack('>Q', word))[0]
    assert math.isfinite(value)
    return F(value)


def hx(word):
    return f'{word:016x}'


def rational(value):
    value = F(value)
    return f'{value.numerator}/{value.denominator}'


def decimal(value, precision=240):
    with localcontext() as ctx:
        ctx.prec = precision
        ctx.Emax = 999999999
        ctx.Emin = -999999999
        return Decimal(value.numerator) / Decimal(value.denominator)


def display(value):
    return format(decimal(value), '.79E') if value else '0'


def directed_display(value, lower=False):
    """80-digit bound serialization; division itself uses directed rounding."""
    if not value:
        return '0'
    with localcontext() as ctx:
        ctx.prec = 80
        ctx.Emax = 999999999
        ctx.Emin = -999999999
        ctx.rounding = ROUND_FLOOR if lower else ROUND_CEILING
        return str(Decimal(value.numerator) / Decimal(value.denominator))


def rn(value):
    """Exact rational to IEEE RNE, independent integer division/remainder."""
    if not value:
        return 0
    sign = (value < 0) << 63
    n, d = abs(value).numerator, abs(value).denominator
    k = n.bit_length() - d.bit_length()
    if (n < d << k) if k >= 0 else (n << -k < d):
        k -= 1
    if k > 1023:
        return sign | 0x7ff0000000000000
    if k < -1075:
        return sign
    unit = max(k - 52, -1074)
    if unit < 0:
        n <<= -unit
    else:
        d <<= unit
    q, rem = divmod(n, d)
    q += rem > d - rem or (rem == d - rem and q & 1)
    if not q:
        return sign
    if unit == -1074 and q < 1 << 52:
        return sign | q
    if q == 1 << 53:
        q >>= 1
        unit += 1
    if unit + 52 > 1023:
        return sign | 0x7ff0000000000000
    return sign | ((unit + 1075) << 52) | (q - (1 << 52))


def order(word):
    return (~word & ((1 << 64) - 1)) if word >> 63 else word | (1 << 63)


def unorder(word):
    return word & ((1 << 63) - 1) if word >> 63 else ~word & ((1 << 64) - 1)


def outward(value, lower):
    word = rn(value)
    if word & 0x7fffffffffffffff == 0x7ff0000000000000:
        if lower and value > 0:
            return 0x7fefffffffffffff
        if not lower and value < 0:
            return 0xffefffffffffffff
        return word
    if (lower and represented(word) > value) or (not lower and represented(word) < value):
        word = unorder(order(word) + (-1 if lower else 1))
    return word


def kind(value):
    word = rn(value)
    magnitude = word & 0x7fffffffffffffff
    sign = 'negative' if value < 0 else 'positive'
    if not value:
        return 'exact-zero'
    if magnitude == 0x7ff0000000000000:
        return sign + '-overflow'
    if not magnitude:
        return sign + '-underflow'
    return 'finite-subnormal' if magnitude < 1 << 52 else 'finite-normal'


def quantize(value, places, rule):
    scaled = value * 10 ** places
    q, rem = divmod(scaled.numerator, scaled.denominator)
    q += 2 * rem > scaled.denominator or (
        2 * rem == scaled.denominator and (rule == 'NearestTiesAwayFromZero' or q & 1))
    return F(q, 10 ** places)


def cycle(maturity, intervals, eom):
    anchored_end = maturity.day == monthrange(maturity.year, maturity.month)[1]
    result = []
    for offset in range(intervals, -1, -1):
        year, month0 = divmod(maturity.year * 12 + maturity.month - 1 - 6 * offset, 12)
        last = monthrange(year, month0 + 1)[1]
        day = last if eom == 'Preserve' and anchored_end else min(maturity.day, last)
        result.append(date(year, month0 + 1, day))
    assert all(181 <= (b - a).days <= 184 for a, b in zip(result, result[1:]))
    return result


def definition(family, maturity='2030-01-15', intervals=10, coupon='5', role='Regular',
               dated_offset=0, issue_offset=0, settlement_offset=1, settlement=None, eom='Ignore'):
    q = cycle(date.fromisoformat(maturity), intervals, eom)
    d = q[0] + timedelta(days=dated_offset)
    i = d + timedelta(days=issue_offset)
    first = 2 if role == 'Long' else 1
    s = date.fromisoformat(settlement) if settlement else i + timedelta(days=settlement_offset)
    assert q[0] <= d < q[1] and (role == 'Regular') == (d == q[0])
    assert d <= i < q[first] and i <= s < q[-1]
    c = F(coupon)
    assert 0 <= c <= (1 << 127) - 1
    return {'family': family, 'q': q, 'd': d, 'i': i, 'f': first, 's': s,
            'coupon': coupon, 'c': c, 'role': role, 'eom': eom}


def prepared(item, basis, tie):
    q, d, s, c = (item[k] for k in ('q', 'd', 's', 'c'))
    k = next(i for i, boundary in enumerate(q) if boundary > s)
    r, den = (q[k] - s).days, (q[k] - q[k - 1]).days
    indices = list(range(max(item['f'], k), len(q)))
    s0, d0 = (q[1] - q[0]).days, (q[1] - d).days
    first_amount = c / 2 if item['role'] == 'Regular' else c / 2 * (
        F(d0, s0) + (item['role'] == 'Long'))
    amounts = [(first_amount if i == item['f'] else c / 2) + (100 if i == len(q) - 1 else 0)
               for i in indices]
    pieces = []
    if s < q[item['f']]:
        if item['role'] == 'Long' and s >= q[1]:
            pieces = [(d0, s0), ((s - q[1]).days, (q[2] - q[1]).days)]
        else:
            pieces = [((s - d).days, s0)]
    else:
        previous = max(boundary for boundary in q[item['f']:] if boundary <= s)
        pieces = [((s - previous).days, den)]
    raw_ai = sum((c * F(days, 2 * half) for days, half in pieces), F())
    source_ai = F()
    for days, half in pieces:
        daily = 5 * c / half
        if basis == 'NineDecimal':
            daily = quantize(daily, 9, tie)
        source_ai += daily * days
    published = quantize(source_ai, 5, tie) / 10
    return {'k': k, 'r': r, 's': den, 'n': len(q) - 1 - k, 'indices': indices,
            'amounts': amounts, 'js': [i - k for i in indices], 'raw_ai': raw_ai,
            'published': published, 'basis': basis, 'tie': tie,
            'm': sum(a > 0 for a in amounts)}


def powersums(x, count):
    """Exact binary concatenation: x**count and sums of j**0,j,j**2*x**j."""
    if not count:
        return F(1), F(), F(), F()
    p, s0, s1, s2 = powersums(x, count // 2)
    half = count // 2
    out0 = s0 + p * s0
    out1 = s1 + p * (s1 + half * s0)
    out2 = s2 + p * (s2 + 2 * half * s1 + half * half * s0)
    square = p * p
    if count & 1:
        index = 2 * half
        out0 += square
        out1 += index * square
        out2 += index * index * square
        square *= x
    return square, out0, out1, out2


def moments(item, p, y):
    b = 1 + y / 2
    q = 1 + F(p['r'], 2 * p['s']) * y
    x = 1 / b
    low, high = p['js'][0], p['js'][-1]
    _, a0, a1, a2 = powersums(x, high + 1)
    _, z0, z1, z2 = powersums(x, low)
    c = item['c'] / 2
    sums = [c * (a0 - z0), c * (a1 - z1), c * (a2 - z2)]
    adjustment = p['amounts'][0] - c - (100 if len(p['amounts']) == 1 else 0)
    for power in range(3):
        sums[power] += adjustment * low ** power * x ** low + 100 * high ** power * x ** high
    return b, q, sums


def ideal(item, p, y):
    b, q, (a0, a1, a2) = moments(item, p, y)
    alpha = F(p['r'], p['s'])
    dirty = a0 / q
    d1 = -(alpha * a0 / (2 * q * q) + a1 / (2 * b * q))
    d2 = alpha * alpha * a0 / (2 * q ** 3) + alpha * a1 / (2 * b * q * q) + (a2 + a1) / (4 * b * b * q)
    moment = (a1 + alpha * a0) / (2 * q)
    return dict(zip(METRICS, (dirty, dirty - p['published'], d1, d2, moment,
                             -d1 / dirty, moment / dirty, d2 / dirty)))


def exact_term_corroboration(p, y, expected):
    """Second exact expression: independent term-wise logarithmic derivatives.

    Large date lattices use the proved concatenation identity instead of
    constructing 19,998 separate Fraction terms in this auxiliary check.
    They still receive both independent Decimal payment-sum checks.
    """
    if len(p['js']) > 128:
        return
    alpha, delta = F(p['r'], p['s']), 2 + y
    q, b = 1 + alpha * y / 2, 1 + y / 2
    dirty = d1 = d2 = moment = F()
    for j, amount in zip(p['js'], p['amounts']):
        value = amount / (q * b ** j)
        first = F(j, 1) / delta + alpha / (2 * q)
        second = F(j * (j + 1), 1) / delta ** 2 + j * alpha / (delta * q) + alpha ** 2 / (2 * q ** 2)
        dirty += value
        d1 -= value * first
        d2 += value * second
        moment += value * (j + alpha) / 2
    actual = dict(zip(METRICS, (dirty, dirty - p['published'], d1, d2, moment,
                               -d1 / dirty, moment / dirty, d2 / dirty)))
    assert actual == expected


def decimal_route(item, p, y, precision):
    """Independent per-payment recurrence and eta/kappa derivatives."""
    with localcontext() as ctx:
        ctx.prec = precision
        ctx.Emax = 999999999
        ctx.Emin = -999999999
        dy = Decimal(y.numerator) / Decimal(y.denominator)
        alpha = Decimal(p['r']) / Decimal(p['s'])
        b, delta = 1 + dy / 2, 2 + dy
        q = 1 + alpha * dy / 2
        factor = 1 / q / b ** p['js'][0]
        price = d1 = d2 = moment = Decimal(0)
        for j, amount in zip(p['js'], p['amounts']):
            a = Decimal(amount.numerator) / Decimal(amount.denominator)
            value = a * factor
            eta = Decimal(j) / delta + alpha / (2 * q)
            kappa = Decimal(j * (j + 1)) / delta ** 2 + Decimal(j) * alpha / (delta * q) + alpha ** 2 / (2 * q ** 2)
            price += value
            d1 -= value * eta
            d2 += value * kappa
            moment += value * (Decimal(j) + alpha) / 2
            factor /= b
        ai = Decimal(p['published'].numerator) / Decimal(p['published'].denominator)
        return dict(zip(METRICS, (price, price - ai, d1, d2, moment, -d1 / price, moment / price, d2 / price)))


def corroborate(item, p, y, exact):
    for precision in (160, 240):
        actual = decimal_route(item, p, y, precision)
        for name in METRICS:
            expected = decimal(exact[name], precision + 30)
            # Huge y amplifies alpha's Decimal encoding error on q; near-floor
            # cancellation loses at most 17 digits. 30 guard digits also cover
            # the <=19998 positive-sum accumulation. This is corroboration only.
            with localcontext() as ctx:
                ctx.prec = precision + 30
                ctx.Emax = 999999999
                ctx.Emin = -999999999
                allowance = Decimal(10) ** (-precision + 30) * max(abs(expected), Decimal(1).scaleb(-10000000))
                assert abs(actual[name] - expected) <= allowance, (item['family'], precision, name)


def coefficient_sums(item, p, y, delta, q):
    """Sum V*g_hat using exact ideal PVs and supplied prefix observations."""
    _, ideal_q, (a0, a1, a2) = moments(item, p, y)
    alpha = F(p['r'], p['s'])
    return [a1 / (ideal_q * delta) + alpha * a0 / (2 * ideal_q * q),
            (a2 + a1) / (ideal_q * delta ** 2) + alpha * a1 / (ideal_q * delta * q) + alpha ** 2 * a0 / (2 * ideal_q * q ** 2),
            (a1 + alpha * a0) / (2 * ideal_q)]


def comparison_budgets(item, p, y, values):
    gamma = 2 * T + T * T
    dirty_radius = ((1 + T) ** 3 - 1) * values['dirty'] + ((1 + T) * p['m'] + 1) * H
    delta, q = 2 + y, 1 + F(p['r'], 2 * p['s']) * y
    dh, qh = represented(rn(delta)), represented(rn(q))
    alpha = F(p['r'], p['s'])
    sums_hat = coefficient_sums(item, p, y, dh, qh)
    ideal_sums = [-values['d1'], values['d2'], values['moment']]
    # Uniform coefficient error envelopes; exact positive polynomial sums.
    ks = (1, 2, 0)
    raw_radii = []
    normalized_radii = []
    for which, (s_hat, s_ideal, k) in enumerate(zip(sums_hat, ideal_sums, ks)):
        # sum of coefficients for the absolute h contribution, no PV weights.
        j0, jn = p['js'][0], p['js'][-1]
        count = len(p['js'])
        sj = F((j0 + jn) * count, 2)
        sj2 = F(jn * (jn + 1) * (2 * jn + 1) - (j0 - 1) * j0 * (2 * j0 - 1), 6)
        if which == 0:
            g_sum = sj / delta + count * alpha / (2 * q)
            gh_sum = sj / dh + count * alpha / (2 * qh)
        elif which == 1:
            g_sum = (sj2 + sj) / delta ** 2 + sj * alpha / (delta * q) + count * alpha ** 2 / (2 * q ** 2)
            gh_sum = (sj2 + sj) / dh ** 2 + sj * alpha / (dh * qh) + count * alpha ** 2 / (2 * qh ** 2)
        else:
            g_sum = gh_sum = (sj + count * alpha) / 2
        relative_g = 1 / (1 - T) ** k - 1 if k else F()
        # EV*g + PVupper*Eg + t*PVupper*ghat, with PVupper=(1+gamma)V+h.
        er = gamma * s_ideal + H * g_sum + relative_g * ((1 + gamma) * s_ideal + H * g_sum) + T * ((1 + gamma) * s_hat + H * gh_sum)
        retained_upper = (1 + T) * ((1 + gamma) * s_hat + H * gh_sum)
        raw_radii.append(er + T * retained_upper + H)
        if p['m'] == 1:
            g = s_ideal / values['dirty']
            lower = (1 - T) ** 2 / (1 + T) ** k * g - H
            upper = (1 + T) ** 2 / (1 - T) ** k * g + H
            normalized_radii.append(max(g - lower, upper - g))
        elif values['dirty'] > dirty_radius:
            lower_d = values['dirty'] - dirty_radius
            normalized_radii.append(er / lower_d + s_ideal * dirty_radius / (lower_d * values['dirty']) + T * retained_upper / lower_d + H)
        else:
            normalized_radii.append(None)
    ai_error = abs(represented(rn(p['published'])) - p['published'])
    clean_magnitude = values['dirty'] + dirty_radius + abs(represented(rn(p['published'])))
    clean_radius = dirty_radius + ai_error + T * clean_magnitude + H
    # Numerator accumulation order is D1,D2,M; public normalized order is
    # Modified,Macaulay,Convexity. Map by name, never reuse positional order.
    budgets = {'dirty': dirty_radius, 'clean': clean_radius,
               'd1': raw_radii[0], 'd2': raw_radii[1], 'moment': raw_radii[2],
               'modified': normalized_radii[0], 'macaulay': normalized_radii[2],
               'convexity': normalized_radii[1]}
    # Source PV acceptance is only a sufficient hypothesis; exact E0 observation
    # comes from the separately owned prescribed-stage oracle, never this file.
    pv_gate = 'guaranteed-in-range'
    b = 1 + y / 2
    low_factor = (1 - T) * (1 - F(60001, 1 << 255))
    for j, amount in zip(p['js'], p['amounts']):
        if amount:
            value = amount / q / b ** j
            if value * low_factor <= H or value * (1 + T) >= OVERFLOW:
                pv_gate = 'requires-indexed-pv-gate'
                break
    return budgets, dirty_radius, ai_error, delta, q, dh, qh, pv_gate


def compressed(values, render=str):
    runs = []
    for value in values:
        if runs and runs[-1][0] == value:
            runs[-1][1] += 1
        else:
            runs.append([value, 1])
    return ';'.join(f'{render(value)}*{count}' for value, count in runs)


def common(item, p):
    row = {'family': item['family'], 'dated': item['d'].isoformat(), 'issue': item['i'].isoformat(),
           'maturity': item['q'][-1].isoformat(), 'coupon': item['coupon'],
           'first_coupon': item['q'][item['f']].isoformat(), 'eom': item['eom'],
           'quasi_count': str(len(item['q'])), 'settlement': item['s'].isoformat(),
           'daily_basis': p['basis'], 'tie_rule': p['tie'], 'first_period': item['role'],
           'next_index': str(p['k']), 'r': str(p['r']), 's': str(p['s']), 'n': str(p['n']),
           'raw_ai': rational(p['raw_ai']), 'published_ai': rational(p['published']),
           'coupon_indices': f"{p['indices'][0]}..{p['indices'][-1]}",
           'source_amounts': compressed(p['amounts'], rational), 'positive_pv_count': str(p['m']),
           'source_display': item.get('source_display', '-'),
           'requirements': 'FR-UST-03,FR-UST-04,FR-UST-05,FR-UST-06,FR-UST-07,FR-UST-08,FR-UST-09,FR-UST-11,FR-UST-12,FR-UST-13,FR-UST-14,FR-UST-15,FR-UST-25'}
    return row


def cases():
    result = []
    for role, offset in (('Regular', 0), ('Short', 37), ('Long', 37)):
        for settlement in (1, 70, 182, 184, 200, 365, 700):
            result.append(definition(f'{role.upper()}-BOUNDARY-{settlement}', role=role,
                          dated_offset=offset, settlement_offset=settlement))
    result += [definition('DATED-BEFORE-ISSUE', issue_offset=20, settlement_offset=30),
               definition('SHORT-DATED-BEFORE-ISSUE', role='Short', dated_offset=10, issue_offset=20),
               definition('LONG-ISSUE-AFTER-SKIPPED', role='Long', dated_offset=10, issue_offset=190),
               definition('ACTUAL-PAYMENT-RESET', settlement='2026-01-15'),
               definition('LONG-SKIPPED-Q1', role='Long', dated_offset=10, settlement='2025-07-15'),
               definition('ONE-HALF', maturity='2026-01-15', intervals=1, settlement='2025-10-15'),
               definition('ONE-SIMPLE', maturity='2026-01-15', intervals=1),
               definition('ZERO-COUPON', coupon='0'),
               definition('DAILY-TIE', coupon='0.0000023'),
               definition('FULL-COEFFICIENT', coupon=str((1 << 127) - 1)),
               definition('SCALE-38-MAX', coupon='1.70141183460469231731687303715884105727'),
               definition('TINY-COUPON', coupon='0.00000000000000000000000000000000000001')]
    for scale in range(39):
        result.append(definition(f'COUPON-SCALE-{scale:02}', coupon='1' if scale == 0 else '0.' + '0' * (scale - 1) + '1',
                                role='Short', dated_offset=1 + scale, settlement_offset=42))
    for eom in ('Preserve', 'Ignore'):
        result += [definition('EOM-' + eom.upper(), maturity='2028-08-31', intervals=6, eom=eom),
                   definition('FEB-LEAP-' + eom.upper(), maturity='2028-02-29', intervals=6, eom=eom)]
    result += [definition('RAW-RANGE', maturity='2035-07-15', intervals=20),
               definition('SUBNORMAL-DIRTY', maturity='2026-07-15', intervals=2, coupon='0'),
               definition('TINY-RAW-RISK', maturity='2026-01-15', intervals=1, coupon='0'),
               definition('MAXIMUM-DATE-LATTICE', maturity='9999-12-31', intervals=19997,
                          settlement_offset=0, eom='Preserve')]
    # Primary-source statement inputs, independently embedded; printed display
    # values are literature anchors, never tolerance for analytic ideal price.
    source = [
        ('A','1990-05-15','1990-05-15','2020-05-15',60,'8.75','Regular','1990-05-15','.0884','99.057893','Ignore'),
        ('B','1990-04-02','1990-04-02','1992-03-31',4,'8.5','Short','1990-04-02','.0859','99.838183','Preserve'),
        ('C','1990-03-01','1990-03-01','1995-05-15',11,'8.5','Long','1990-03-01','.0853','99.805118','Ignore'),
        ('D','1985-11-15','1985-11-29','1995-11-15',20,'9.5','Regular','1985-11-29','.0954','99.730918','Ignore'),
        ('E','1985-07-02','1985-07-02','2005-08-15',41,'10.75','Long','1985-11-04','.1047','102.214586','Ignore'),
        ('F','1983-05-16','1983-05-16','1991-05-15',16,'10.5','Short','1983-08-15','.1053','99.777074','Ignore'),
        ('G','1988-10-15','1988-10-15','1994-12-15',13,'9.75','Long','1988-11-15','.0979','99.738045','Ignore')]
    for letter, dated, issued, mat, intervals, coupon, role, settled, y, printed, eom in source:
        q = cycle(date.fromisoformat(mat), intervals, eom)
        item = definition('SOURCE-' + letter, mat, intervals, coupon, role,
                          dated_offset=(date.fromisoformat(dated) - q[0]).days,
                          issue_offset=(date.fromisoformat(issued) - date.fromisoformat(dated)).days,
                          settlement=settled, eom=eom)
        item.update(source_yield=y, source_display=printed)
        result.append(item)
    return result


def price_rows(definitions):
    rows = []
    seen = set()
    yields = [bitsof(-.5), 0, bitsof(.05), bitsof(2.)]
    policies = [('ExactRatio', 'NearestTiesEven'), ('NineDecimal', 'NearestTiesAwayFromZero')]
    for index, item in enumerate(definitions):
        if item['family'].startswith('SOURCE-'):
            ys = [bitsof(float(item['source_yield']))]
        elif item['family'] == 'MAXIMUM-DATE-LATTICE':
            ys = [0, bitsof(2.)]
        elif item['family'] == 'RAW-RANGE':
            ys = [0xbfffffffffffffff, bitsof(.05)]
        elif item['family'] in ('SUBNORMAL-DIRTY', 'TINY-RAW-RISK'):
            ys = [0x7fefffffffffffff, 0x7fdfffffffffffff, 0, bitsof(.05)]
            if item['family'] == 'SUBNORMAL-DIRTY':
                ys += [bitsof(math.ldexp(1., exponent)) for exponent in (520, 535, 540, 541, 542)]
        else:
            ys = yields + ([1, 0x8000000000000001, 0xbfffffffffffffff] if index % 7 == 0 else [])
        for basis, tie in policies:
            p = prepared(item, basis, tie)
            for word in ys:
                row = common(item, p)
                row['yield_bits'] = hx(word)
                key = '\t'.join(row[k] for k in INPUTS + ['yield_bits'])
                if key in seen:
                    continue
                seen.add(key)
                y = represented(word)
                values = ideal(item, p, y)
                exact_term_corroboration(p, y, values)
                corroborate(item, p, y, values)
                if item['family'].startswith('SOURCE-'):
                    assert quantize(values['clean'], 6, tie) == F(item['source_display'])
                if item['family'] == 'ONE-HALF' and word == bitsof(.05):
                    assert values['macaulay'] == F(1, 4)
                    # 41/162 is the exact-decimal .05 anchor. Binary .05
                    # changes the second ratio, so compare its exact formula.
                    assert values['macaulay'] != (1+y/2) * values['modified']
                budgets, dr, ae, delta, q, dh, qh, pv_gate = comparison_budgets(item, p, y, values)
                row.update(input_fingerprint=sha(key.encode()), pv_gate=pv_gate,
                           comparison_method='exact-Fraction-geometric;Decimal160+240-payment-sum',
                           ai_cache_error=directed_display(ae), dirty_radius=directed_display(dr), delta_ideal=display(delta),
                           prefix_ideal=display(q), delta_rn_bits=hx(rn(delta)), prefix_rn_bits=hx(rn(q)))
                for metric in METRICS:
                    value, radius = values[metric], budgets[metric]
                    center = display(value)
                    row[metric + '_ideal'] = center
                    row[metric + '_ideal_rn_bits'] = hx(rn(value))
                    row[metric + '_ideal_class'] = kind(value)
                    # Include reference-center serialization error as well as
                    # observational error; the serialized bound rounds upward.
                    row[metric + '_allowance'] = directed_display(radius + abs(F(center)-value)) if radius is not None else '-'
                    if radius is None or (value and radius >= abs(value)):
                        comparison = 'uninformative'
                    elif kind(value).endswith('overflow') or kind(value).endswith('underflow'):
                        comparison = 'ideal-range-reference'
                    else:
                        comparison = 'conditional-absolute'
                    row[metric + '_comparison'] = comparison
                row['id'] = f'UST-IDEAL-{len(rows) + 1:05}'
                rows.append(row)
    return rows


def root_enclosure(item, p, target, low, high, iterations=180):
    left = ideal(item, p, low)['dirty'] - target
    right = ideal(item, p, high)['dirty'] - target
    assert left >= 0 and right <= 0
    if not left:
        return low, low, 0
    if not right:
        return high, high, 0
    for iteration in range(iterations):
        middle = (low + high) / 2
        value = ideal(item, p, middle)['dirty'] - target
        if not value:
            return middle, middle, iteration + 1
        if value > 0:
            low = middle
        else:
            high = middle
    return low, high, iterations


def inverse_rows(definitions):
    rows, seen = [], set()
    chosen = [x for x in definitions if (x['family'].startswith(('REGULAR-BOUNDARY','SHORT-BOUNDARY','LONG-BOUNDARY','EOM-','SOURCE-')) or
              x['family'] in ('DATED-BEFORE-ISSUE','LONG-ISSUE-AFTER-SKIPPED','ONE-HALF','ONE-SIMPLE','ZERO-COUPON','TINY-COUPON'))]
    for item in chosen:
        for basis, tie in [('ExactRatio','NearestTiesEven'), ('NineDecimal','NearestTiesAwayFromZero')]:
            p = prepared(item, basis, tie)
            for target_kind in ('dirty', 'clean'):
                # Target chosen in native decimal quote space, not by pricing at
                # a hidden expected yield. Exact binary supplied quote retained.
                quote = F(100) if target_kind == 'dirty' else F(97)
                word = rn(quote)
                ideal_target = represented(word) + (p['published'] if target_kind == 'clean' else 0)
                target = represented(rn(represented(word) + (represented(rn(p['published'])) if target_kind == 'clean' else 0)))
                low, high = F(-3, 2), F(8)
                upper_image = p['amounts'][0] / (1 - F(p['r'], p['s'])) if p['n'] == 0 and p['r'] < p['s'] else None
                row = common(item, p)
                row['requirements'] += ',FR-UST-16,FR-UST-17'
                row.update(target_kind=target_kind, target_bits=hx(word), dirty_target_bits=hx(rn(target)),
                           ideal_dirty_target=rational(ideal_target), caller_lo_bits=hx(rn(low)), caller_hi_bits=hx(rn(high)),
                           abs_tol_bits=hx(bitsof(1e-13)), rel_tol_bits=hx(bitsof(1e-13)), max_evals='512',
                           residual_scale=str(int(abs(target) >= 1 << 970)))
                key = '\t'.join(row[k] for k in INPUTS + ['target_kind','target_bits','caller_lo_bits','caller_hi_bits','abs_tol_bits','rel_tol_bits','max_evals'])
                if key in seen:
                    continue
                seen.add(key)
                row['input_fingerprint'] = sha(key.encode())
                row['id'] = f'UST-INVERSE-{len(rows) + 1:05}'
                row['ideal_image'] = 'open-upper:' + rational(upper_image) if upper_image is not None else 'positive-unbounded-upper'
                # Expand only the REFERENCE bracket on paper. The API's own
                # caller bracket remains explicit and never auto-expanded.
                if ideal(item,p,low)['dirty'] < ideal_target or ideal(item,p,high)['dirty'] > ideal_target:
                    row.update(inverse_method='caller-bracket-does-not-cover-ideal-root', comparison='ideal-image-only')
                    for field in ('root_lo','root_hi','root_lo_bits','root_hi_bits','left_sign','right_sign','bisections','uniform_price_error','target_shift_error','slope_lower','coordinate_allowance','conditioned_lo_bits','conditioned_hi_bits'):
                        row[field] = '-'
                else:
                    if p['n'] == 0:
                        root = 2 * F(p['s'], p['r']) * (p['amounts'][0] / ideal_target - 1)
                        assert ideal(item,p,root)['dirty'] == ideal_target and low <= root <= high
                        lo = hi = root
                        count = 0
                        method = 'exact-one-payment-linear-prefix'
                    else:
                        lo, hi, count = root_enclosure(item,p,ideal_target,low,high)
                        method = 'exact-rational-sign-bisection'
                    uniform = ((1+T)**3-1) * ideal(item,p,low)['dirty'] + ((1+T)*p['m']+1)*H
                    shift = abs(target - ideal_target)
                    slope = -ideal(item,p,high)['d1']
                    solver_width = 2 * (F(1e-13) + F(1e-13) * max(abs(low),abs(high)))
                    allowance = (uniform + shift) / slope + solver_width
                    lsign = ideal(item,p,lo)['dirty'] - ideal_target
                    rsign = ideal(item,p,hi)['dirty'] - ideal_target
                    row.update(inverse_method=method, root_lo=rational(lo), root_hi=rational(hi),
                               root_lo_bits=hx(outward(lo,True)), root_hi_bits=hx(outward(hi,False)),
                               left_sign='zero' if not lsign else 'positive', right_sign='zero' if not rsign else 'negative',
                               bisections=str(count), uniform_price_error=directed_display(uniform), target_shift_error=directed_display(shift),
                               slope_lower=directed_display(slope,True), coordinate_allowance=directed_display(allowance),
                               conditioned_lo_bits=hx(outward(lo-allowance,True)), conditioned_hi_bits=hx(outward(hi+allowance,False)),
                               comparison='uninformative-coordinate' if allowance >= high-low else 'conditional-bracket-coordinate')
                rows.append(row)
    # n=0 alpha<1 excluded image is an ideal mathematical statement, NOT an
    # observed API precheck. A finite positive target above it has no ideal root.
    item = next(x for x in definitions if x['family']=='ONE-HALF')
    p = prepared(item,'ExactRatio','NearestTiesEven')
    row = common(item,p)
    supremum = p['amounts'][0] / (1-F(p['r'],p['s']))
    target = represented(rn(supremum)) + 1
    row.update(id=f'UST-INVERSE-{len(rows)+1:05}', family='EXCLUDED-IDEAL-UPPER', target_kind='dirty',
               target_bits=hx(rn(target)), dirty_target_bits=hx(rn(target)), ideal_dirty_target=rational(target),
               caller_lo_bits='bfffffffffffffff', caller_hi_bits=hx(bitsof(8.)), abs_tol_bits=hx(bitsof(1e-13)),
               rel_tol_bits=hx(bitsof(1e-13)), max_evals='512', ideal_image='open-upper:'+rational(supremum),
               inverse_method='exact-open-image-exclusion', comparison='ideal-image-only', residual_scale='0')
    key='\t'.join(row[k] for k in INPUTS+['target_kind','target_bits','caller_lo_bits','caller_hi_bits','abs_tol_bits','rel_tol_bits','max_evals'])
    assert key not in seen
    row['input_fingerprint']=sha(key.encode())
    for field in ('root_lo','root_hi','root_lo_bits','root_hi_bits','left_sign','right_sign','bisections','uniform_price_error','target_shift_error','slope_lower','coordinate_allowance','conditioned_lo_bits','conditioned_hi_bits'):
        row[field]='-'
    rows.append(row)
    return rows


def requirement_rows():
    lines = (ROOT/'docs/REQUIREMENTS.md').read_text().splitlines()
    selected = [line for line in lines if re.match(r'^\| FR-UST-\d\d \|', line)]
    assert len(selected)==26
    return ('\n'.join(selected)+'\n').encode()


def fixture_bytes(path, rows, columns, input_columns, checking):
    old = {}
    if checking:
        for line in path.read_text().split('\n\n',1)[0].splitlines():
            key,value=line.split(': ',1)
            old[key]=value
    records=('\n'.join('\t'.join(row[column] for column in columns) for row in rows)+'\n').encode()
    body=('\t'.join(columns)+'\n').encode()+records
    inputs=('\n'.join('\t'.join(row[column] for column in input_columns) for row in rows)+'\n').encode()
    assert len(set(inputs.splitlines()))==len(rows)
    generator=Path(__file__).read_bytes()
    requirements=requirement_rows()
    header=[('schema','1'),('fixture-set','M8-US-TREASURY-IDEAL-INVERSE' if path==INV else 'M8-US-TREASURY-IDEAL-PRICE-RISK'),
      ('domain','bonds'),('description','Independent ideal references; no Treasury implementation or prescribed-stage expectations read'),
      ('reference','Exact Fraction finite-geometric product derivatives; independent Decimal160/240 per-payment sums'),
      ('reference-version','Python standard library'),('reference-source','docs/M8_US_TREASURY_CONTRACT.md + docs/M8_US_TREASURY_CASH_CONTRACT.md'),
      ('tolerance','TOL-UST-RATIONAL / TOL-UST-INVERSE conditional absolute bounds; ideal bits are not E0 observed outputs'),
      ('reproduction','python3 -I validation/generator/treasury_ideal_oracles.py --check'),
      ('generated-at',old.get('generated-at',datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'))),
      ('platform',old.get('platform',f'{platform.system()} {platform.machine()}; Python {platform.python_version()}')),
      ('generator','validation/generator/treasury_ideal_oracles.py'),('generator-sha256',sha(generator)),('generator-fnv64',fnv(generator)),
      ('adopted-contract-sha256',sha(MAIN.read_bytes())),('adopted-contract-fnv64',fnv(MAIN.read_bytes())),
      ('cash-contract-sha256',sha(CASH.read_bytes())),('cash-contract-fnv64',fnv(CASH.read_bytes())),
      ('requirements-rows-sha256',sha(requirements)),('requirements-rows-fnv64',fnv(requirements)),
      ('body-sha256',sha(body)),('body-fnv64',fnv(body)),('records-sha256',sha(records)),('records-fnv64',fnv(records)),
      ('record-count',str(len(rows))),('column-count',str(len(columns))),('profile-column','family'),
      ('profile-counts',json.dumps(dict(sorted(Counter(row['family'] for row in rows).items())),separators=(',',':'))),
      ('input-columns',','.join(input_columns)),('unique-input-count',str(len(rows))),('inputs-sha256',sha(inputs)),('inputs-fnv64',fnv(inputs)),
      ('decimal-precisions','160,240'),('serialized-reference-significant-digits','80'),
      ('range-qualification','Per-quantity IDEAL range; actual source-PV gates and getters independently tested by prescribed-stage corpus'),
      ('root-qualification','Exact rational root signs are certificates; Decimal agreement alone is not an interval certificate'),
      ('expectation-status','Initial independent expectations; root reproduction and attestation required before freezing')]
    return ('\n'.join(f'{key}: {value}' for key,value in header)+'\n\n').encode()+body


def self_checks():
    for value in (F(), F(1), F(-1), MIN_SUB, H, 3*H, MIN_NORMAL, MAX, OVERFLOW, F(1,3),-F(1,3)):
        word=rn(value)
        if abs(value)<OVERFLOW:
            assert word==bitsof(float(value))
    assert rn(H)==0 and rn(3*H)==2 and rn(OVERFLOW)==0x7ff0000000000000
    for count in range(30):
        x=F(17,13)
        power,s0,s1,s2=powersums(x,count)
        assert power==x**count
        assert s0==sum((x**j for j in range(count)),F())
        assert s1==sum((j*x**j for j in range(count)),F())
        assert s2==sum((j*j*x**j for j in range(count)),F())


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    assert sha(MAIN.read_bytes())==MAIN_SHA and sha(CASH.read_bytes())==CASH_SHA
    if not args.check:
        if OUT.exists() or INV.exists():
            raise SystemExit('refusing to overwrite initial/frozen fixture; --check is read-only')
    self_checks()
    definitions=cases()
    prices=price_rows(definitions)
    inverse=inverse_rows(definitions)
    outputs=[(OUT,prices,PRICE_COLUMNS,INPUTS+['yield_bits']),
             (INV,inverse,INV_COLUMNS,INPUTS+['target_kind','target_bits','caller_lo_bits','caller_hi_bits','abs_tol_bits','rel_tol_bits','max_evals'])]
    payloads=[(path,fixture_bytes(path,rows,columns,inputs,args.check)) for path,rows,columns,inputs in outputs]
    for path,data in payloads:
        if args.check:
            if path.read_bytes()!=data:
                raise SystemExit('frozen byte mismatch: '+str(path))
        else:
            with path.open('xb') as stream:
                stream.write(data)
        print(json.dumps({'path':str(path.relative_to(ROOT)),'records':len(prices) if path==OUT else len(inverse),
                          'sha256':sha(data),'fnv64':fnv(data),'check':args.check},sort_keys=True))


if __name__=='__main__':
    main()
