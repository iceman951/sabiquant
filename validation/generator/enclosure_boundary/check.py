#!/usr/bin/env python3
"""Separately expressed checker for the SAME finite B2 feature reference.
Same human/agent author as author.py, stated honestly; no author import or Rust
output. RNE uses a monotone IEEE-lattice search, not the author's quotient step.
"""
import argparse
import datetime
from fractions import Fraction as Q
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
FAMILY_SHA = '295247d3700a0a06c7fd1d9bde830740cb4a66bd927a38061e255150b8eb52aa'
COLS = ('id model context_id source_plan_id admission_case_id basis compounding yield_bits '
        'input face_bits accrued_bits coupon_amount_bits flows target_kind numerator denominator '
        'coefficient_n coefficient_d argument_n argument_d lower_n lower_d upper_n upper_d '
        'lower_strict upper_strict lower_bits upper_bits range_class point_kind point_coefficient '
        'point_scale point_bits').split()
TOP = 0x7fefffffffffffff


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def value(word):
    sign = -1 if word & (1 << 63) else 1
    power = (word >> 52) & 2047
    fraction = word & ((1 << 52) - 1)
    assert power != 2047
    if power == 0:
        return Q(sign * fraction, 1 << 1074)
    mantissa = fraction | (1 << 52)
    exponent = power - 1075
    return sign * Q(mantissa * (1 << exponent), 1) if exponent >= 0 else sign * Q(mantissa, 1 << -exponent)


def decode(s):
    assert len(s) == 16 and all(c in '0123456789abcdef' for c in s)
    return value(int(s, 16))


def floor_grid(x):
    """Greatest nonnegative finite IEEE value<=x, by independent64step search."""
    assert x >= 0
    lo, hi = 0, TOP
    while lo < hi:
        middle = (lo + hi + 1) // 2
        if value(middle) <= x:
            lo = middle
        else:
            hi = middle - 1
    return lo


def signed_hex(word, negative):
    if word is None:
        return '_'
    return '%016x' % ((word | (1 << 63)) if negative and word else word)


def adjacent(x):
    mag = abs(x)
    lower = floor_grid(mag)
    upper = lower if value(lower) == mag else (lower + 1 if lower != TOP else None)
    if x < 0:
        return signed_hex(upper, True), signed_hex(lower, True)
    return signed_hex(lower, False), signed_hex(upper, False)


def nearest(x):
    mag = abs(x)
    lo = floor_grid(mag)
    if value(lo) == mag:
        chosen = lo
    else:
        assert lo != TOP, 'source literal cannot round beyond finite grid'
        hi = lo + 1
        low_gap, high_gap = mag - value(lo), value(hi) - mag
        chosen = lo if low_gap < high_gap or (low_gap == high_gap and lo % 2 == 0) else hi
    return signed_hex(chosen, x < 0)


def rational_pair(n, d):
    assert n != '_' and d != '_'
    q = Q(int(n), int(d))
    assert int(d) > 0 and str(q.numerator) == n and str(q.denominator) == d
    return q


def field_q(row, prefix):
    return rational_pair(row[prefix + '_n'], row[prefix + '_d'])


def delta(a, b):
    return (datetime.date.fromisoformat(b) - datetime.date.fromisoformat(a)).days


def literal_input(context):
    def item(v):
        if v is None:
            return '_'
        if isinstance(v, bool):
            return ('false', 'true')[int(v)]
        if isinstance(v, list):
            return ','.join(item(x) for x in v)
        return str(v)
    cells = {}
    for k in sorted(context):
        if k in ['id', 'model', 'focus']:
            continue
        v = context[k]
        if isinstance(v, dict):
            for sub in v:
                cells[k + '_' + sub] = item(v[sub])
        else:
            cells[k] = item(v)
    return ';'.join(k + '=' + cells[k] for k in sorted(cells))


def prepared(c):
    assert c['issue'] == c['settlement']
    amounts, flows = [], []
    face = decode(c['face_bits'])
    assert face > 0 and decode(c['redemption_bits']) > 0
    if c['kind'] == 'FixedRate':
        dates = c['schedule']['raw_dates']
        assert dates == ['2021-01-01', '2022-01-01', '2023-01-01']
        assert c['ex_coupon'] == [None, None] and c['coupon_accrual'] == 'Actual365Fixed'
        for index in range(len(dates) - 1):
            a, b = dates[index:index + 2]
            coupon = nearest(Q(delta(a, b), 365) * decode(c['coupon_rate_bits']) * face)
            amounts.append(coupon)
            flows.append((index, b, coupon, nearest(Q(delta(c['settlement'], b), 365))))
    flows.append((len(amounts), c['redemption_payment'], c['redemption_bits'],
                  nearest(Q(delta(c['settlement'], c['redemption_payment']), 365))))
    return amounts, flows


def generic(c, call):
    coupons, flows = prepared(c)
    y = decode(call['yield_bits'])
    unit = Q(100, 1) / decode(c['face_bits'])
    if call['compounding'] == 'Simple':
        fractions = []
        for _, _, amount, time in flows:
            divisor = 1 + decode(time) * y
            assert divisor > 0
            fractions.append(decode(amount) / divisor)
        total = unit * sum(fractions, Q(0))
        return coupons, flows, ('rational', total)
    assert call['compounding'] == 'Continuous'
    groups = {}
    for _, _, amount, time in flows:
        exponent = -decode(time) * y
        groups[exponent] = groups.get(exponent, Q(0)) + unit * decode(amount)
    nonzero = [(e, c) for e, c in groups.items() if c]
    assert len(nonzero) == 1
    e, coefficient = nonzero[0]
    return coupons, flows, ('exp', coefficient, e)


def named(source, call):
    inp, s = source['input'], source['source']
    assert inp == call['source_input'] and inp['coupon_coefficient'] == '0'
    assert inp['settlement'] == inp['issue']
    y = decode(call['yield_bits'])
    # The reused named family has zero coupon, sourceAI0 and one terminal par.
    # Re-derive its complete principal expression without the author's sum loop.
    assert all(Q(int(b['amount']['n']), int(b['amount']['d'])) == (100 if i == len(s['bundles']) - 1 else 0)
               for i, b in enumerate(s['bundles']))
    if source['model'] in ['Gilt', 'Treasury']:
        assert y > -2
        reciprocal_base = Q(2) / (Q(2) + y)
        if source['model'] == 'Gilt':
            last = s['bundles'][-1]['lambda_']
            assert int(last['d']) == 1
            exponent = int(last['n'])
            assert int(s['source_accrued']['n']) == 0
        else:
            assert s['r'] == s['s'] and s['r'] > 0
            exponent = s['bundles'][-1]['j'] + 1
            assert int(s['published_accrued']['n']) == 0
        assert 0 <= exponent <= 8
        discount = Q(1)
        for _ in range(exponent):
            discount *= reciprocal_base
        return 100 * discount
    assert source['model'] == 'Canadian' and s['active_convention'] == 'MoneyMarket'
    tau_days = delta(inp['settlement'], inp['true_maturity'])
    assert tau_days == s['maturity_horizon_days']
    assert s['bundles'][-1]['actual_payment_date'] == inp['true_maturity']
    assert s['bundles'][-1]['remaining_horizon_days'] == 0
    assert int(s['settlement_accrued']['n']) == int(s['valuation_accrued']['n']) == 0
    denominator = Q(365) + tau_days * y
    assert denominator > 0
    return Q(36500) / denominator


def preferred(q):
    # Alternate normal form: test the39admitted decimal grids directly.
    for scale in range(39):
        scaled = q * (10 ** scale)
        if scaled.denominator == 1 and -(1 << 127) <= scaled.numerator < (1 << 127):
            return ('decimal', str(scaled.numerator), str(scale), '_')
    lower, upper = adjacent(q)
    if lower != '_' and lower == upper:
        return ('binary', '_', '_', lower)
    return ('none', '_', '_', '_')


def classified(q):
    if abs(q) > value(TOP):
        return 'above_max' if q > 0 else 'below_negative_max'
    if q == 0:
        return 'zero'
    if abs(q) < value(1):
        return 'positive_sub_min' if q > 0 else 'negative_sub_min'
    return 'finite'


def target(row, mathematical):
    if mathematical[0] == 'rational':
        q = mathematical[1]
        assert row['target_kind'] == 'exact_rational'
        assert rational_pair(row['numerator'], row['denominator']) == q
        assert (row['lower_bits'], row['upper_bits']) == adjacent(q)
        assert row['range_class'] == classified(q)
        assert tuple(row[k] for k in ['point_kind', 'point_coefficient', 'point_scale', 'point_bits']) == preferred(q)
        for k in ['coefficient_n', 'coefficient_d', 'argument_n', 'argument_d', 'lower_n', 'lower_d', 'upper_n', 'upper_d', 'lower_strict', 'upper_strict']:
            assert row[k] == '_'
        return
    _, coefficient, exponent = mathematical
    assert row['target_kind'] == 'strict_range'
    assert row['numerator'] == row['denominator'] == '_'
    assert field_q(row, 'coefficient') == coefficient and field_q(row, 'argument') == exponent
    assert abs(exponent) > 2098
    # e>2 (the first three positive Taylor terms suffice). Positive exp is
    # monotone; only2^2098 is formed, never2^abs(exponent) or exp(exponent).
    if exponent < 0:
        envelope = abs(coefficient) / (1 << 2098)
        lower, upper = field_q(row, 'lower'), field_q(row, 'upper')
        assert row['lower_strict'] == row['upper_strict'] == '1'
        if coefficient > 0:
            assert lower == 0 and envelope <= upper < value(1)
            assert (row['lower_bits'], row['upper_bits'], row['range_class']) == ('0000000000000000', '0000000000000001', 'positive_sub_min')
        else:
            assert upper == 0 and envelope <= -lower < value(1)
            assert (row['lower_bits'], row['upper_bits'], row['range_class']) == ('8000000000000001', '0000000000000000', 'negative_sub_min')
    else:
        assert coefficient > 0
        lower = field_q(row, 'lower')
        assert value(TOP) < lower <= coefficient * (1 << 2098)
        assert row['upper_n'] == row['upper_d'] == '_' and row['upper_strict'] == '0'
        assert row['lower_strict'] == '1'
        assert (row['lower_bits'], row['upper_bits'], row['range_class']) == ('7fefffffffffffff', '_', 'above_max')
    assert row['point_kind'] == 'unclassified'
    assert row['point_coefficient'] == row['point_scale'] == row['point_bits'] == '_'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('fixture', type=Path)
    args = parser.parse_args()
    assert digest(HERE / 'FAMILY.json') == FAMILY_SHA
    family = json.loads((HERE / 'FAMILY.json').read_text())
    raw = args.fixture.read_bytes()
    assert raw.isascii() and b'\r' not in raw and raw.endswith(b'\n')
    head, body = raw.decode('ascii').split('\n\n')
    pairs = [line.split(': ', 1) for line in head.splitlines()]
    assert len({a for a, b in pairs}) == len(pairs)
    headers = dict(pairs)
    for key, filename in [('author-sha256', 'author.py'), ('checker-sha256', 'check.py'),
                          ('family-sha256', 'FAMILY.json'), ('reused-source-sha256', 'REUSED_SOURCE_ROWS.jsonl'),
                          ('reused-admission-sha256', 'REUSED_ADMISSION_ROWS.jsonl')]:
        assert headers[key] == digest(HERE / filename)
    assert headers['record-count'] == '35' and headers['generic-source-plan-count'] == '11'
    lines = body.splitlines()
    assert lines[0].split('\t') == COLS
    rows = []
    for line in lines[1:]:
        cells = line.split('\t')
        assert len(cells) == len(COLS)
        assert not any(ord(c) < 32 for cell in cells for c in cell)
        rows.append(dict(zip(COLS, cells)))
    assert len(rows) == len({r['id'] for r in rows}) == 35
    source = {s['id']: s for s in map(json.loads, (HERE / 'REUSED_SOURCE_ROWS.jsonl').read_text().splitlines())}
    admissions = {a['id']: a for a in map(json.loads, (HERE / 'REUSED_ADMISSION_ROWS.jsonl').read_text().splitlines())}
    contexts = {c['id']: c for c in family['instrument_contexts']}
    offset = 0
    plans = set()
    kinds = {}
    for call in family['quote_controls'] + family['reused_named_controls']:
        generic_call = 'context_id' in call
        context_id = call['context_id'] if generic_call else call['id']
        if generic_call:
            c = contexts[context_id]
            coupons, flows, mathematical = generic(c, call)
            plan_id = 'B2-SOURCE:' + context_id + ':' + call['compounding']
            plans.add(plan_id)
            compounding, model, admission_id = call['compounding'], 'Generic', '_'
        else:
            c = call['source_input']
            mathematical = ('rational', named(source[call['source_id']], call))
            plan_id, model, admission_id = call['source_id'], call['model'], call['admission_case_id']
            compounding = 'MoneyMarket' if model == 'Canadian' else 'Periodic'
            a = admissions[admission_id]
            assert a['source_id'] == plan_id and a['yield_bits'] == call['yield_bits']
            assert a['expected']['predicate'] == 'MathematicallyAdmitted'
        for basis in call['bases']:
            row = rows[offset]
            offset += 1
            expected = {'id': call['id'] + ':' + basis, 'model': model, 'context_id': context_id,
                        'source_plan_id': plan_id, 'admission_case_id': admission_id,
                        'basis': basis, 'compounding': compounding, 'yield_bits': call['yield_bits'],
                        'input': literal_input(c)}
            for key in expected:
                assert row[key] == expected[key], (row['id'], key)
            if generic_call:
                assert row['face_bits'] == c['face_bits'] and row['accrued_bits'] == '0000000000000000'
                assert row['coupon_amount_bits'] == (','.join(coupons) or '_')
                flow_cell = ';'.join('%d@%s@%s@%s' % f for f in flows)
                assert row['flows'] == flow_cell
            else:
                assert all(row[k] == '_' for k in ['face_bits', 'accrued_bits', 'coupon_amount_bits', 'flows'])
            target(row, mathematical)
            kinds[row['target_kind']] = kinds.get(row['target_kind'], 0) + 1
    assert offset == 35 and len(plans) == 11
    print(json.dumps({'status': 'PASS_SEPARATELY_EXPRESSED_EXACT_REFERENCE_CHECK',
                      'targets': offset, 'generic_source_plans': len(plans), 'kinds': kinds,
                      'author_imported': False, 'Rust_output_read': False,
                      'same_agent_author_as_generator': True}, sort_keys=True))


if __name__ == '__main__':
    main()
