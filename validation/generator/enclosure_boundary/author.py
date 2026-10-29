#!/usr/bin/env python3
"""Independent exact B2 expectations, stdlib only, create-new output directories.
No Rust/output/financial-price-author/old observed result is read or imported.
Input literals/source roles are the frozen reviewed FAMILY and released source rows.
"""
import argparse
import datetime
from fractions import Fraction
import hashlib
import json
import platform
from pathlib import Path

HERE = Path(__file__).resolve().parent
FAMILY_SHA = '295247d3700a0a06c7fd1d9bde830740cb4a66bd927a38061e255150b8eb52aa'
FREEZE_SHA = '03eaebab2e9e1107f8efc3abf0c0c58a0dcb84676f203cacb2f0317c39e6ba8f'
THRESHOLD = 2098


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def canonical(j):
    return json.dumps(j, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


def rational(q):
    return {'n': str(q.numerator), 'd': str(q.denominator)}


def ratio(j):
    return Fraction(int(j['n']), int(j['d']))


def binary(bits):
    word = int(bits, 16)
    assert len(bits) == 16 and 0 <= word < (1 << 64)
    sign = -1 if word >> 63 else 1
    field = (word >> 52) & 2047
    fraction = word & ((1 << 52) - 1)
    assert field != 2047
    if field == 0:
        return Fraction(sign * fraction, 1 << 1074)
    significand = (1 << 52) + fraction
    exponent = field - 1075
    if exponent >= 0:
        return Fraction(sign * (significand << exponent), 1)
    return Fraction(sign * significand, 1 << (-exponent))


def floor_log2(n, d):
    k = n.bit_length() - d.bit_length()
    if k >= 0:
        return k - (n < (d << k))
    return k - ((n << (-k)) < d)


def rne_binary(q):
    """Single exact rational round-to-nearest/ties-even, canonical zero."""
    if q == 0:
        return '0000000000000000'
    sign = int(q < 0)
    n, d = abs(q.numerator), q.denominator
    k = floor_log2(n, d)
    shift = 1074 if k < -1022 else 52 - k
    if shift >= 0:
        n <<= shift
    else:
        d <<= -shift
    quotient, remainder = divmod(n, d)
    twice = 2 * remainder
    if twice > d or (twice == d and quotient % 2 == 1):
        quotient += 1
    if quotient == 0:
        return '0000000000000000'
    if k < -1022:
        assert quotient <= (1 << 52)
        return '%016x' % ((sign << 63) | quotient)
    if quotient == (1 << 53):
        quotient >>= 1
        k += 1
    assert -1022 <= k <= 1023, 'literal source/horizon rounded outside finite IEEE'
    assert (1 << 52) <= quotient < (1 << 53)
    word = (sign << 63) | ((k + 1023) << 52) | (quotient - (1 << 52))
    return '%016x' % word


def days(a, b):
    return (datetime.date.fromisoformat(b) - datetime.date.fromisoformat(a)).days


def source_plan(context, compounding):
    assert context['model'] == 'Generic'
    assert context['issue'] == context['settlement']
    face = binary(context['face_bits'])
    assert face > 0
    flows, coupons = [], []
    if context['kind'] == 'FixedRate':
        schedule = context['schedule']
        dates = schedule['raw_dates']
        assert dates[0] == context['issue']
        assert context['coupon_accrual'] == 'Actual365Fixed'
        assert schedule['calendar'] == 'Null' and schedule['convention'] == 'Unadjusted'
        assert schedule['termination_convention'] == 'Unadjusted'
        assert schedule['step_months'] == 12 and schedule['rule'] == 'Forward'
        assert schedule['stub'] == 'None' and schedule['end_of_month'] == 'Ignore'
        assert schedule['max_boundaries'] == len(dates) == 3
        assert context['ex_coupon'] == [None, None]
        rate = binary(context['coupon_rate_bits'])
        for index, (start, payment) in enumerate(zip(dates, dates[1:])):
            accrual = Fraction(days(start, payment), 365)
            assert accrual >= 0
            amount = rne_binary(face * rate * accrual)
            assert binary(amount) != 0 or face * rate * accrual == 0
            coupons.append(amount)
            horizon = rne_binary(Fraction(days(context['settlement'], payment), 365))
            flows.append({'index': index, 'kind': 'coupon', 'payment': payment,
                          'amount_bits': amount, 'horizon_bits': horizon})
        assert context['max_flows'] >= len(coupons) + 1
    else:
        assert context['kind'] == 'ZeroCoupon'
        assert context['contractual_maturity'] >= context['issue']
    payment = context['redemption_payment']
    assert payment >= context['issue']
    assert binary(context['redemption_bits']) > 0
    assert not flows or payment >= flows[-1]['payment']
    flows.append({'index': len(coupons), 'kind': 'redemption', 'payment': payment,
                  'amount_bits': context['redemption_bits'],
                  'horizon_bits': rne_binary(Fraction(days(context['settlement'], payment), 365))})
    return {'id': 'B2-SOURCE:' + context['id'] + ':' + compounding,
            'context_id': context['id'], 'model': 'Generic', 'compounding': compounding,
            'face_bits': context['face_bits'], 'cached_accrued_bits': '0000000000000000',
            'coupon_amount_bits': coupons, 'flows': flows}


def exact(q):
    return {'kind': 'exact_rational', 'value': rational(q)}


def exp_certificate(coefficient, argument, cancellations):
    """Strict rational comparison, never computes exp(argument) or 2**argument."""
    assert coefficient != 0 and argument != 0
    half_min = Fraction(1, 1 << 1075)
    threshold_power = 1 << THRESHOLD
    if argument < 0:
        assert -argument > THRESHOLD
        # e>2 and monotonicity give exp(-a)<2^-THRESHOLD.
        assert abs(coefficient) < half_min * threshold_power
        if coefficient > 0:
            lower, upper = Fraction(0), half_min
        else:
            lower, upper = -half_min, Fraction(0)
    else:
        assert argument > THRESHOLD and coefficient > 0
        lower, upper = Fraction(1 << 1075), None
        # exp(a)>2^THRESHOLD; a small fixed integer comparison suffices.
        assert coefficient * threshold_power > lower
    return {'kind': 'strict_range',
            'expression': {'coefficient': rational(coefficient), 'argument': rational(argument)},
            'lower': None if lower is None else rational(lower),
            'upper': None if upper is None else rational(upper),
            'lower_strict': lower is not None, 'upper_strict': upper is not None,
            'certificate': {'method': 'e_gt_2_integer_threshold', 'threshold': THRESHOLD,
                            'same_payment_cancellation': cancellations}}


def generic_target(plan, quote, basis):
    assert basis in ('Dirty', 'Clean')
    y = binary(quote['yield_bits'])
    scale = Fraction(100) / binary(plan['face_bits'])
    accrued = binary(plan['cached_accrued_bits'])
    if quote['compounding'] == 'Simple':
        total = Fraction(0)
        for flow in plan['flows']:
            denominator = 1 + y * binary(flow['horizon_bits'])
            assert denominator > 0  # even zero coefficient flows remain checked
            total += binary(flow['amount_bits']) / denominator
        if basis == 'Clean':
            total -= accrued
        return exact(total * scale)
    assert quote['compounding'] == 'Continuous'
    assert accrued == 0
    groups = {}
    indices = {}
    for flow in plan['flows']:
        argument = -y * binary(flow['horizon_bits'])
        groups[argument] = groups.get(argument, Fraction(0)) + binary(flow['amount_bits'])
        indices.setdefault(argument, []).append(flow['index'])
    cancellations = [indices[x] for x in sorted(groups) if groups[x] == 0 and len(indices[x]) > 1]
    terms = [(a, c) for a, c in groups.items() if c != 0]
    if not terms:
        return exact(Fraction(0))
    assert len(terms) == 1
    argument, coefficient = terms[0]
    if argument == 0:
        return exact(coefficient * scale)
    return exp_certificate(coefficient * scale, argument, cancellations)


def named_target(source, control, basis):
    y = binary(control['yield_bits'])
    model, s = source['model'], source['source']
    if model == 'Gilt':
        base = 1 + y / 2
        assert base > 0
        total = Fraction(0)
        for bundle in s['bundles']:
            exponent = ratio(bundle['lambda_'])
            assert exponent.denominator == 1 and 0 <= exponent.numerator <= 8
            total += ratio(bundle['amount']) / base ** exponent.numerator
        if basis == 'Clean': total -= ratio(s['source_accrued'])
        else: assert basis == 'Dirty'
    elif model == 'Treasury':
        base = 1 + y / 2
        prefix = 1 + y * Fraction(s['r'], 2 * s['s'])
        assert base > 0 and prefix > 0
        total = Fraction(0)
        for bundle in s['bundles']:
            assert 0 <= bundle['j'] <= 8
            total += ratio(bundle['amount']) / (prefix * base ** bundle['j'])
        if basis == 'Clean': total -= ratio(s['published_accrued'])
        else: assert basis == 'Dirty'
    else:
        assert model == 'Canadian' and s['active_convention'] == 'MoneyMarket'
        tau = Fraction(s['maturity_horizon_days'], 365)
        denominator = 1 + y * tau
        assert denominator > 0
        total = Fraction(0)
        for bundle in s['bundles']:
            rho = Fraction(bundle['remaining_horizon_days'], 365)
            assert 0 <= rho <= tau
            numerator = 1 + y * rho
            assert numerator > 0
            total += ratio(bundle['amount']) * numerator / denominator
        if basis == 'Clean': total -= ratio(s['settlement_accrued'])
        else: assert basis in ('ModelDirty', 'SettlementDirty')
    return exact(total)


def build():
    assert sha(HERE / 'FAMILY.json') == FAMILY_SHA
    family = json.loads((HERE / 'FAMILY.json').read_text())
    contexts = {c['id']: c for c in family['instrument_contexts']}
    sources = {r['id']: r for r in map(json.loads, (HERE / 'REUSED_SOURCE_ROWS.jsonl').read_text().splitlines())}
    plans, plan_index, cases = [], {}, []
    for call in family['quote_controls']:
        key = (call['context_id'], call['compounding'])
        if key not in plan_index:
            plan = source_plan(contexts[key[0]], key[1])
            plan_index[key] = plan
            plans.append(plan)
        plan = plan_index[key]
        for basis in call['bases']:
            cases.append({'id': call['id'] + ':' + basis, 'context_id': call['context_id'],
                          'model': 'Generic', 'basis': basis, 'compounding': call['compounding'],
                          'yield_bits': call['yield_bits'], 'family_sha256': FAMILY_SHA,
                          'source_plan_id': plan['id'], 'admission_case_id': None,
                          'target': generic_target(plan, call, basis)})
    for call in family['reused_named_controls']:
        source = sources[call['source_id']]
        assert source['input'] == call['source_input']
        for basis in call['bases']:
            cases.append({'id': call['id'] + ':' + basis, 'context_id': call['id'],
                          'model': call['model'], 'basis': basis,
                          'compounding': 'MoneyMarket' if call['model'] == 'Canadian' else 'Periodic',
                          'yield_bits': call['yield_bits'], 'family_sha256': FAMILY_SHA,
                          'source_plan_id': call['source_id'], 'admission_case_id': call['admission_case_id'],
                          'target': named_target(source, call, basis)})
    assert len(plans) == 11 and len(cases) == 35
    assert len({r['id'] for r in plans}) == 11 and len({r['id'] for r in cases}) == 35
    return plans, cases


COLUMNS = ('id model context_id source_plan_id admission_case_id basis compounding '
           'yield_bits input face_bits accrued_bits coupon_amount_bits flows target_kind '
           'numerator denominator coefficient_n coefficient_d argument_n argument_d '
           'lower_n lower_d upper_n upper_d lower_strict upper_strict lower_bits upper_bits '
           'range_class point_kind point_coefficient point_scale point_bits').split()
MAX_BITS = '7fefffffffffffff'
MIN_BITS = '0000000000000001'
ZERO_BITS = '0000000000000000'


def negate_bits(bits):
    if bits is None:
        return None
    word = int(bits, 16)
    return ZERO_BITS if word == 0 else '%016x' % (word ^ (1 << 63))


def positive_neighbors(q):
    assert q >= 0
    if q > binary(MAX_BITS):
        return MAX_BITS, None
    if q == 0:
        return ZERO_BITS, ZERO_BITS
    k = floor_log2(q.numerator, q.denominator)
    exponent = max(k - 52, -1074)
    unit = Fraction(1 << exponent) if exponent >= 0 else Fraction(1, 1 << -exponent)
    scaled = q / unit
    down = scaled.numerator // scaled.denominator
    lower_q = down * unit
    lower = rne_binary(lower_q)
    if lower_q == q:
        return lower, lower
    return lower, rne_binary(lower_q + unit)


def neighbors(q):
    if q >= 0:
        return positive_neighbors(q)
    lower, upper = positive_neighbors(-q)
    return negate_bits(upper), negate_bits(lower)


def point(q, lower, upper):
    if q == 0:
        return 'decimal', '0', '0', '_'
    denominator = q.denominator
    twos = fives = 0
    while denominator % 2 == 0:
        denominator //= 2
        twos += 1
    while denominator % 5 == 0:
        denominator //= 5
        fives += 1
    scale = max(twos, fives)
    if denominator == 1 and scale <= 38:
        coefficient = q.numerator * (2 ** (scale - twos)) * (5 ** (scale - fives))
        if -(1 << 127) <= coefficient <= (1 << 127) - 1:
            return 'decimal', str(coefficient), str(scale), '_'
    if lower is not None and lower == upper:
        return 'binary', '_', '_', lower
    return 'none', '_', '_', '_'


def range_class(q):
    maximum = binary(MAX_BITS)
    minimum = binary(MIN_BITS)
    if q > maximum:
        return 'above_max'
    if q < -maximum:
        return 'below_negative_max'
    if 0 < q < minimum:
        return 'positive_sub_min'
    if -minimum < q < 0:
        return 'negative_sub_min'
    return 'zero' if q == 0 else 'finite'


def literal(value):
    if value is None:
        return '_'
    if isinstance(value, bool):
        return 'true' if value else 'false'
    if isinstance(value, list):
        return ','.join(literal(x) for x in value)
    assert isinstance(value, (str, int))
    return str(value)


def input_cell(context):
    omit = {'id', 'model', 'focus'}
    flattened = {}
    for key, value in context.items():
        if key in omit:
            continue
        if isinstance(value, dict):
            for inner, item in value.items():
                flattened[key + '_' + inner] = literal(item)
        else:
            flattened[key] = literal(value)
    assert all(not any(c in x for c in '\t\n;=') for x in flattened.values())
    return ';'.join(k + '=' + flattened[k] for k in sorted(flattened))


def fixture_rows(plans, cases, family):
    plan_by_id = {p['id']: p for p in plans}
    inputs = {c['id']: c for c in family['instrument_contexts']}
    inputs.update({c['id']: c['source_input'] for c in family['reused_named_controls']})
    for case in cases:
        record = {k: '_' for k in COLUMNS}
        for k in ['id', 'model', 'context_id', 'source_plan_id', 'admission_case_id', 'basis', 'compounding', 'yield_bits']:
            record[k] = '_' if case[k] is None else str(case[k])
        record['input'] = input_cell(inputs[case['context_id']])
        if case['model'] == 'Generic':
            plan = plan_by_id[case['source_plan_id']]
            record['face_bits'] = plan['face_bits']
            record['accrued_bits'] = plan['cached_accrued_bits']
            record['coupon_amount_bits'] = ','.join(plan['coupon_amount_bits']) or '_'
            record['flows'] = ';'.join('@'.join([str(f['index']), f['payment'], f['amount_bits'], f['horizon_bits']]) for f in plan['flows'])
        target = case['target']
        record['target_kind'] = target['kind']
        if target['kind'] == 'exact_rational':
            q = ratio(target['value'])
            record['numerator'], record['denominator'] = str(q.numerator), str(q.denominator)
            lower, upper = neighbors(q)
            record['range_class'] = range_class(q)
            pk, pc, ps, pb = point(q, lower, upper)
            record['point_kind'], record['point_coefficient'], record['point_scale'], record['point_bits'] = pk, pc, ps, pb
        else:
            for key in ['coefficient', 'argument']:
                for side in ['n', 'd']:
                    record[key + '_' + side] = target['expression'][key][side]
            for key in ['lower', 'upper']:
                if target[key] is not None:
                    record[key + '_n'], record[key + '_d'] = target[key]['n'], target[key]['d']
                record[key + '_strict'] = '1' if target[key + '_strict'] else '0'
            coefficient = ratio(target['expression']['coefficient'])
            argument = ratio(target['expression']['argument'])
            if argument > 0:
                lower, upper, classification = MAX_BITS, None, 'above_max'
            elif coefficient > 0:
                lower, upper, classification = ZERO_BITS, MIN_BITS, 'positive_sub_min'
            else:
                lower, upper, classification = negate_bits(MIN_BITS), ZERO_BITS, 'negative_sub_min'
            record['range_class'] = classification
            record['point_kind'] = 'unclassified'
        record['lower_bits'] = lower if lower is not None else '_'
        record['upper_bits'] = upper if upper is not None else '_'
        yield record


def render(generated_at, generator_platform):
    plans, cases = build()
    family = json.loads((HERE / 'FAMILY.json').read_text())
    headers = [
        ('schema', '1'), ('fixture-set', 'M8-ENCLOSURE-BOUNDARY'), ('domain', 'bond-price-enclosure'),
        ('description', 'One finite public-input boundary family: exact prepared sources and complete per100 mathematical targets'),
        ('reference', 'oracle:exact integer IEEE RNE and Fraction; rigorous e>2 comparison certificates'),
        ('reference-version', 'Python >=3.9 stdlib; exact rational arithmetic; no approximate exp evaluation'),
        ('reference-source', 'Frozen M8 generic/named contracts; one canonical FAMILY and three released source/admission rows'),
        ('tolerance', 'TOL-E0; exact source bits, complete rational targets and strict analytic inequalities; no epsilon'),
        ('reproduction', 'python3 -I validation/generator/enclosure_boundary/author.py --check validation/fixtures/bonds/enclosure-boundary.fixture'),
        ('generated-at', generated_at), ('platform', generator_platform),
        ('family-sha256', FAMILY_SHA), ('author-sha256', sha(Path(__file__))),
        ('checker-sha256', sha(HERE / 'check.py')),
        ('reused-source-sha256', sha(HERE / 'REUSED_SOURCE_ROWS.jsonl')),
        ('reused-admission-sha256', sha(HERE / 'REUSED_ADMISSION_ROWS.jsonl')),
        ('first-private-author-sha256', 'f5a5afd810b17b8be2950cb5daf711de2e196786c1b0b2cea8299f1d1e7480a5'),
        ('record-count', str(len(cases))), ('generic-source-plan-count', str(len(plans))),
        ('encoding', 'TSV; bits=16lowerhex; absent=_; input=key=value semicolon pairs and comma arrays; flows=index@payment@amountbits@horizonbits semicolon records'),
        ('certificate', 'Strict exp comparison uses e>2, threshold2098 and exact source cancellation; never exp(MAX) or exponent-sized shift'),
        ('scope', 'Independent mathematical targets only; no API work/proofkind/outcome golden, no oldnative result premise, no E2 completion')]
    text = '\n'.join(k + ': ' + v for k, v in headers) + '\n\n' + '\t'.join(COLUMNS) + '\n'
    for record in fixture_rows(plans, cases, family):
        text += '\t'.join(record[k] for k in COLUMNS) + '\n'
    return text.encode('ascii')


def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--out', type=Path)
    group.add_argument('--check', type=Path)
    args = parser.parse_args()
    if args.check is not None:
        actual = args.check.read_bytes()
        header = dict(line.split(': ', 1) for line in actual.decode('ascii').split('\n\n', 1)[0].splitlines())
        expected = render(header['generated-at'], header['platform'])
        if expected != actual:
            raise SystemExit('fixture differs from the frozen reference author; do not overwrite accepted expectations')
        print('PASS byte reproduction:35 targets,11 Generic source plans')
    else:
        timestamp = datetime.datetime.now(datetime.timezone.utc).isoformat().replace('+00:00', 'Z')
        data = render(timestamp, platform.system() + ' ' + platform.machine())
        with args.out.open('xb') as f:
            f.write(data)
        print('CREATED ' + str(args.out) + ' sha256=' + hashlib.sha256(data).hexdigest())


if __name__ == '__main__':
    main()
