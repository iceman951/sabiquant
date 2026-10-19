#!/usr/bin/env python3
"""Independent conventional-gilt ideal price/risk and target-driven inverses.

Python standard library only. No Rust, observed production values, solver
implementation or QuantLib is read. The fixed adopted contract is the model.
Exact source rationals are distinct from conditional elementary-error bounds.
"""
import argparse
from calendar import monthrange
from collections import Counter
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal, localcontext, ROUND_CEILING, ROUND_FLOOR, ROUND_HALF_UP
from fractions import Fraction as F
import hashlib
import json
import math
from pathlib import Path
import platform
import struct

ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / 'docs/M8_GILT_CONTRACT.md'
CONTRACT_SHA = '03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292'
SOURCE_CHECKER_SHA = '0bdf495eab0bc73fb5d250f5e9266c5186a3b905d127d7cdaf8eb6c4c3b33b02'
OUT = ROOT / 'validation/fixtures/bonds/gilt-ideal-oracle.fixture'
INV_OUT = ROOT / 'validation/fixtures/bonds/gilt-inverse-ideal-oracle.fixture'
U = F(1, 1 << 53)
MIN = F(1, 1 << 1074)
MAX = F((1 << 53) - 1) * (1 << 971)
OVERFLOW = MAX + (1 << 970)
METRICS = ('dirty', 'clean', 'd1', 'd2', 'moment', 'modified', 'macaulay', 'convexity')
COMMON = ('id family issue maturity coupon first_dividend eom quasi_dates ex_dates '
          'settlement first_period next_index r s n ai_num ai_den bundle_indices source_coefficients').split()
PRICE_COLS = COMMON + ['yield_bits', 'source_display'] + [
    f'{metric}_{part}' for metric in METRICS for part in ('ideal', 'class', 'lo_bits', 'hi_bits')]
INV_COLS = COMMON + ('target_kind target_bits dirty_target_bits bracket_lo_bits bracket_hi_bits '
                    'abs_tol_bits rel_tol_bits max_evals residual_scale ideal_root_lo_bits '
                    'ideal_root_hi_bits conditioned_lo_bits conditioned_hi_bits uniform_price_error '
                    'slope_lower solver_width_bound inverse_method source_display').split()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    value = 0xcbf29ce484222325
    for item in data:
        value = ((value ^ item) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{value:016x}'


def bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def binary(value):
    value = struct.unpack('>d', struct.pack('>Q', value))[0]
    assert math.isfinite(value)
    return F(value)


def hx(value):
    return f'{value:016x}'


def p2(exponent):
    return F(1 << exponent) if exponent >= 0 else F(1, 1 << -exponent)


def rn(value):
    """Exact rational -> single IEEE nearest/ties-even, including subnormals."""
    value = F(value)
    if not value:
        return 0
    sign = int(value < 0) << 63
    n, d = abs(value).numerator, abs(value).denominator
    k = n.bit_length() - d.bit_length()
    if (n < d << k) if k >= 0 else (n << -k < d):
        k -= 1
    if k > 1023:
        return sign | 0x7ff0000000000000
    if k < -1075:
        return sign
    unit = max(k - 52, -1074)
    if unit >= 0:
        d <<= unit
    else:
        n <<= -unit
    q, r = divmod(n, d)
    q += r > d - r or (r == d - r and bool(q & 1))
    if q == 0:
        return sign
    if unit == -1074 and q < 1 << 52:
        return sign | q
    if q == 1 << 53:
        q >>= 1
        unit += 1
    exponent = unit + 52
    if exponent > 1023:
        return sign | 0x7ff0000000000000
    return sign | ((exponent + 1023) << 52) | (q - (1 << 52))


def ordered(value):
    return (~value & ((1 << 64) - 1)) if value >> 63 else value | (1 << 63)


def unorder(value):
    return value & ((1 << 63) - 1) if value >> 63 else ~value & ((1 << 64) - 1)


def outward(value, lower):
    value = F(value)
    result = rn(value)
    if result & 0x7fffffffffffffff == 0x7ff0000000000000:
        if lower and value > 0:
            return 0x7fefffffffffffff
        if not lower and value < 0:
            return 0xffefffffffffffff
        return result
    represented = binary(result)
    if (lower and represented > value) or (not lower and represented < value):
        result = unorder(ordered(result) + (-1 if lower else 1))
    return result


def dec(value):
    value = F(value)
    return Decimal(value.numerator) / Decimal(value.denominator)


def text(value):
    """80 significant digits for reference; upper radii are separately carried."""
    if not value:
        return '0'
    return format(value, '.79E')


def upper_text(value):
    with localcontext() as context:
        context.prec = 80
        context.rounding = ROUND_CEILING
        return str(+value)


def lower_text(value):
    with localcontext() as context:
        context.prec = 80
        context.rounding = ROUND_FLOOR
        return str(+value)


def cycle(maturity, intervals, eom):
    result = []
    anchor_end = maturity.day == monthrange(maturity.year, maturity.month)[1]
    for j in range(intervals, -1, -1):
        coordinate = maturity.year * 12 + maturity.month - 1 - 6 * j
        year, month0 = divmod(coordinate, 12)
        month = month0 + 1
        last = monthrange(year, month)[1]
        day = last if eom == 'Preserve' and anchor_end else min(maturity.day, last)
        result.append(date(year, month, day))
    return tuple(result)


def definition(family, maturity, intervals, coupon='6', role='Regular', issue_offset=0,
               settlement=None, eom='Ignore', cutoff_days=7, first_cutoff=None):
    q = cycle(date.fromisoformat(maturity), intervals, eom)
    issue = q[0] if role == 'Regular' else q[0] + timedelta(days=issue_offset)
    first_index = 2 if role == 'Long' else 1
    assert q[0] <= issue < q[1] and (role == 'Regular') == (issue == q[0])
    ex = []
    for index in range(first_index, len(q)):
        lower = q[1] if role == 'Long' and index == first_index else (
            issue if index == first_index else q[index - 1])
        cutoff = max(lower, q[index] - timedelta(days=cutoff_days))
        if index == first_index and first_cutoff:
            cutoff = date.fromisoformat(first_cutoff)
        assert lower <= cutoff <= q[index]
        ex.append(cutoff)
    settlement = issue if settlement is None else (
        date.fromisoformat(settlement) if isinstance(settlement, str) else settlement)
    assert issue <= settlement < q[-1]
    return dict(family=family, issue=issue, q=q, coupon=F(coupon), coupon_text=coupon,
                first=first_index, role=role, ex=tuple(ex), sdate=settlement, eom=eom)


def published(raw):
    count, remainder = divmod(raw.numerator * 10**6, raw.denominator)
    return F(count + (remainder * 2 >= raw.denominator), 10**6)


def source(model):
    q, issue, settlement = model['q'], model['issue'], model['sdate']
    b = model['coupon'] / 2
    k = next(i for i in range(len(q) - 1) if q[i] <= settlement < q[i + 1])
    r, s = (q[k + 1] - settlement).days, (q[k + 1] - q[k]).days
    s1, r1 = (q[1] - q[0]).days, (q[1] - issue).days
    first_value = b if model['role'] == 'Regular' else published(
        b * (F(r1, s1) + int(model['role'] == 'Long')))
    bundles = []
    for index in range(max(k + 1, model['first']), len(q)):
        ordinal = index - model['first']
        amount = first_value if ordinal == 0 else b
        entitled = settlement <= model['ex'][ordinal]
        if entitled or index == len(q) - 1:
            amount = (amount if entitled else F(0)) + (100 if index == len(q) - 1 else 0)
            bundles.append((index, amount, F(r + (index - k - 1) * s, s)))
    assert bundles and bundles[-1][0] == len(q) - 1
    next_actual = max(k + 1, model['first'])
    ex = settlement > model['ex'][next_actual - model['first']]
    if settlement >= q[model['first']] or model['role'] == 'Regular':
        t = (settlement - q[k]).days
        ai = b * F(t - (s if ex else 0), s)
    elif model['role'] == 'Short':
        t = (settlement - issue).days
        ai = b * F(t - (r1 if ex else 0), s1)
    elif settlement < q[1]:
        assert not ex
        ai = b * F((settlement - issue).days, s1)
    else:
        s2, r2 = (q[2] - q[1]).days, (settlement - q[1]).days
        ai = b * (F(r2 - s2, s2) if ex else F(r1, s1) + F(r2, s2))
    return dict(k=k, r=r, s=s, n=len(q) - k - 2, ai=ai, bundles=tuple(bundles))


def common(model, state, identifier):
    q = model['q']
    return dict(zip(COMMON, (identifier, model['family'], model['issue'].isoformat(),
        q[-1].isoformat(), model['coupon_text'], q[model['first']].isoformat(), model['eom'],
        ','.join(v.isoformat() for v in q), ','.join(v.isoformat() for v in model['ex']),
        model['sdate'].isoformat(), model['role'], str(state['k'] + 1), str(state['r']),
        str(state['s']), str(state['n']), str(state['ai'].numerator), str(state['ai'].denominator),
        ','.join(str(i) for i, _, _ in state['bundles']),
        ','.join(f'{a.numerator}/{a.denominator}' for _, a, _ in state['bundles']))))


def logarithm(y):
    """Ideal ln(1+y/2), with a series retaining tiny represented inputs."""
    x = dec(y / 2)
    if abs(x) >= Decimal('1e-8'):
        return dec(1 + y / 2).ln()
    term, total, index = x, x, 1
    if not x:
        return Decimal(0)
    while True:
        index += 1
        term *= -x
        added = term / index
        total += added
        if abs(added) < abs(total) * Decimal(10) ** (-__import__('decimal').getcontext().prec + 8):
            return total


def ideal(model, state, y):
    """Independent ideal finite sum from exact source rational coordinates.

    This mathematical expression differs from the production per-bundle log
    observation. Exact rational source coefficients and coordinates are inputs.
    """
    logq = logarithm(y)
    dirty = d1 = d2 = moment = Decimal(0)
    flows = []
    for index, amount, lam in state['bundles']:
        # Direct real power per retained source flow; no rounded lambda input.
        z = -dec(lam) * logq
        discount = z.exp()
        value = dec(amount) * discount
        one, two, time = dec(lam/(2+y)), dec(lam*(lam+1)/(2+y)**2), dec(lam/2)
        dirty += value
        d1 -= value * one
        d2 += value * two
        moment += value * time
        flows.append((amount, lam, discount, z, value, one, two, time))
    ai = dec(state['ai'])
    return dict(dirty=dirty, clean=dirty-ai, d1=d1, d2=d2, moment=moment,
                modified=-d1/dirty, macaulay=moment/dirty, convexity=d2/dirty), flows


def factor_error(y, lam, discount, z):
    u, minimum = dec(U), dec(MIN)
    if not y:
        return Decimal(0)
    if abs(y) < p2(-52):
        ez = dec(lam * y*y / (8 * (1 - abs(y)/2)))
        ez += u * dec(abs(lam*y/2)) + minimum/2
    else:
        ez = (5*u+4*u*u) * abs(z)
    return discount * (ez.exp()-1) + 4*u*discount*ez.exp() + minimum/2


def budgets(model, state, y, values, flows, precision):
    u, minimum = dec(U), dec(MIN)
    eps = Decimal(10) ** (-precision + 30)
    delta_hat = binary(rn(2 + y))
    pv_error = Decimal(0)
    retained_errors = {name: Decimal(0) for name in ('d1','d2','moment')}
    factor_safe = True
    for amount, lam, discount, z, value, one, two, time in flows:
        ed = factor_error(y, lam, discount, z) + discount*eps
        if discount-ed <= dec(MIN/2) or discount+ed >= dec(OVERFLOW):
            factor_safe = False
        if not amount:
            continue
        ev = dec(amount)*ed + u*dec(amount)*(discount+ed) + minimum/2 + abs(value)*eps
        if value-ev <= dec(MIN/2) or value+ev >= dec(OVERFLOW):
            factor_safe = False
        pv_error += ev
        exacts = (lam/(2+y), lam*(lam+1)/(2+y)**2, lam/2)
        hats = (lam/delta_hat, lam*(lam+1)/delta_hat**2, lam/2)
        for name, exact, hat in zip(('d1','d2','moment'), exacts, hats):
            retained_errors[name] += ev*dec(abs(hat)) + value*dec(abs(hat-exact))
            retained_errors[name] += u*(value+ev)*dec(abs(hat)) + abs(value*dec(hat))*eps
    dirty_error = pv_error + u*(values['dirty']+pv_error) + minimum/2 + values['dirty']*eps
    if values['dirty']+dirty_error >= dec(OVERFLOW):
        factor_safe = False
    ai_hat = binary(rn(state['ai']))
    ai_error = dec(abs(ai_hat-state['ai']))
    errors = dict(dirty=dirty_error,
                  clean=dirty_error+ai_error+u*(values['dirty']+dirty_error+dec(abs(ai_hat)))+minimum/2)
    pre = dict(dirty=dirty_error, clean=errors['clean'])
    for name, error in retained_errors.items():
        pre[name] = error
        errors[name] = error + u*(abs(values[name])+error) + minimum/2
    for name, raw in (('modified','d1'),('macaulay','moment'),('convexity','d2')):
        if values['dirty'] <= dirty_error:
            errors[name] = pre[name] = Decimal('Infinity')
        else:
            error = (retained_errors[raw]+abs(values[name])*dirty_error)/(values['dirty']-dirty_error)
            pre[name] = error
            errors[name] = error + u*(abs(values[name])+error)+minimum/2
    return errors, pre, factor_safe


def metric(value, error, before, safe, name):
    if not safe or not error.is_finite():
        return text(value), 'uninformative', '-', '-'
    lo, hi = value-error, value+error
    mag = abs(value)
    if name not in ('dirty','clean') and mag+before < dec(MIN/2):
        status = ('negative' if value < 0 else 'positive') + '_underflow'
    elif name not in ('dirty','clean') and mag-before > dec(OVERFLOW):
        status = ('negative' if value < 0 else 'positive') + '_overflow'
    elif name not in ('dirty','clean') and (mag-before <= dec(MIN/2) or mag+before >= dec(OVERFLOW)):
        status = 'uninformative'
    else:
        status = 'finite'
    return text(value), status, hx(outward(F(lo), True)), hx(outward(F(hi), False))


def source_models():
    settlements = ('1999-05-24','1999-05-26','1999-05-27','1999-06-07',
                   '1999-05-10','1999-05-17','1999-05-18','1999-05-26')
    prices = ('145.012268','145.047301','141.070132','141.257676',
              '113.315543','113.415969','110.058738','110.170218')
    roots = ('0.051620635','0.051649111','0.04871425','0.048856785',
             '0.057009527','0.057253361','0.050413308','0.050652248')
    for i, settlement in enumerate(settlements):
        m = definition(f'SOURCE-{i+1}', '2015-12-07' if i<4 else '2004-11-26',
                       34 if i<4 else 12, '8' if i<4 else '6.75', settlement=settlement,
                       first_cutoff='1999-05-26' if i<4 else '1999-05-17')
        yield m, bits(.04445 if i<4 else .04634), prices[i], roots[i], 135 if i<4 else 108


def profiles():
    yield definition('REGULAR', '2030-06-07', 8, settlement='2026-08-11')
    yield definition('ONE-FULL', '2027-06-07', 1)
    q = cycle(date(2027,6,7), 1, 'Ignore')
    yield definition('ONE-HALF', '2027-06-07', 1, '5', settlement=q[-1]-timedelta(days=91))
    yield definition('ONE-EX', '2027-06-07', 1, '5', settlement='2027-06-06')
    yield definition('RATIONAL-189-181', '2026-11-26', 2, settlement='2026-05-18')
    for role in ('Short','Long'):
        for settlement in ('1999-05-07','1999-06-06','1999-06-07','1999-06-08',
                           '1999-11-25','1999-11-26','1999-11-27','1999-12-06',
                           '1999-12-07','1999-12-08'):
            yield definition(f'{role.upper()}-{settlement}', '2001-12-07', 6,
                             role=role, issue_offset=151, settlement=settlement,
                             first_cutoff='1999-05-27' if role=='Short' else '1999-11-26')
    yield definition('PUBLISHED-ZERO', '2027-06-07', 2, '0.0000001', role='Short', issue_offset=181)
    yield definition('REGULAR-SCALE39', '2027-06-07', 2,
                     '1.00000000000000000000000000000000000001')
    yield definition('EOM-PRESERVE', '2028-02-29', 4, eom='Preserve', settlement='2026-08-31')
    yield definition('EOM-IGNORE', '2028-02-29', 4, eom='Ignore', settlement='2026-08-29')
    yield definition('ZERO-C', '2030-06-07', 8, '0', settlement='2026-08-11')


def price_inputs():
    for model, y, display, _, _ in source_models():
        yield model, y, display
    quotes = (0, 1<<63, 1, (1<<63)|1, bits(2**-52), bits(-2**-52),
              bits(.05), bits(-.05), bits(-1), bits(1), bits(1e10), bits(1e100))
    for model in profiles():
        for y in quotes:
            yield model, y, '-'
    for intervals in (1,18,19):
        yield definition(f'NEAR-FLOOR-{intervals}', '2040-06-07', intervals, '0'), bits(math.nextafter(-2,0)), '-'
    yield definition('RAW-UNDERFLOW', '2027-06-07', 1, '0'), bits(float.fromhex('0x1.fffffffffffffp+1023')), '-'
    yield definition('NORMALIZED-ONLY-UNDERFLOW', '2027-11-26', 1, str((1<<127)-1),
                     settlement='2027-11-25'), bits(1e170), '-'
    # The fixed quote is RN(2*(exp(-10.12*ln(2))-1)), independently agreeing
    # at160/240 digits. No platform pow/libm is used to construct this input.
    yield definition('MOMENT-OVERFLOW', '2080-06-07', 100, '0'), 0xbffff8a374a83311, '-'


def price_rows(precision):
    with localcontext() as context:
        context.prec = precision
        for index, (model, y_bits, display) in enumerate(price_inputs(), 1):
            state, y = source(model), binary(y_bits)
            values, flows = ideal(model,state,y)
            alternate = price_only(state,y)
            assert abs(alternate-values['dirty']) <= values['dirty']*Decimal(10)**(-precision+20)
            errors, pre, safe = budgets(model,state,y,values,flows,precision)
            row = common(model,state,f'GILT-I-{index:05d}')
            row.update(yield_bits=hx(y_bits), source_display=display)
            for name in METRICS:
                value, status, lo, hi = metric(values[name],errors[name],pre[name],safe,name)
                for part, item in zip(('ideal','class','lo_bits','hi_bits'),(value,status,lo,hi)):
                    row[f'{name}_{part}'] = item
            if display != '-':
                assert values['dirty'].quantize(Decimal('.000001'),rounding=ROUND_HALF_UP) == Decimal(display)
            yield row


def price_only(state,y):
    logq = logarithm(y)
    # One fractional power and integer powers: independent from direct powers
    # in ideal(). Cached for inverse bisection work within this invocation.
    initial = (-dec(F(state['r'],state['s']))*logq).exp()
    v = (-logq).exp()
    return sum((dec(a)*initial*v**(index-state['k']-1)
                for index,a,_ in state['bundles']),Decimal(0))


def solve(state,target,low,high,precision):
    assert price_only(state,low) >= target and price_only(state,high) <= target
    for _ in range(300):
        mid = (low+high)/2
        if price_only(state,mid) > target:
            low = mid
        else:
            high = mid
    # The reference arithmetic budget is relative to gross positive prices;
    # convert it to a coordinate radius with a global positive slope lower bound.
    hi_price = price_only(state,high)
    slope = sum((dec(a)*(-dec(lam)*logarithm(high)).exp()*dec(lam/(2+high))
                 for _,a,lam in state['bundles']),Decimal(0))
    slope *= 1-Decimal(10)**(-precision+30)
    rounding = max(abs(target),abs(hi_price))*Decimal(10)**(-precision+30)/slope
    return dec(low)-rounding, dec(high)+rounding


def uniform_error(state, low, high, precision):
    u, minimum = dec(U),dec(MIN)
    eps = Decimal(10)**(-precision+30)
    total = error = Decimal(0)
    maxlog = max(abs(logarithm(low)),abs(logarithm(high)))
    for _,amount,lam in state['bundles']:
        maximum = (-dec(lam)*logarithm(low)).exp()
        ez = (5*u+4*u*u)*dec(lam)*maxlog
        tiny = dec(lam*p2(-104)/(8*(1-p2(-53)))) + u*dec(lam*p2(-53))+minimum/2
        ez = max(ez,tiny)
        ed = maximum*(ez.exp()-1)+4*u*maximum*ez.exp()+minimum/2+maximum*eps
        ev = dec(amount)*ed+u*dec(amount)*(maximum+ed)+minimum/2
        if amount:
            total += dec(amount)*maximum
            error += ev
    return error+u*(total+error)+minimum/2+total*eps


def inverse_inputs():
    for model,_,_,display,target in source_models():
        yield model,'Dirty',bits(float(target)),bits(-.1),bits(.2),display
    for index,model in enumerate(profiles()):
        if index % 3 == 0 or model['family'].startswith('ONE'):
            for target in (90.,105.,135.):
                if model['family']=='ONE-EX':
                    # One day/182 has a restricted binary-quote attainable
                    # range near -2;135 is not bracketed by valid f64 yields.
                    if target==135.:
                        continue
                    low,high=math.nextafter(-2,0),1e10 if target==90. else 10.
                else:
                    low,high=-.9,10.
                yield model,'Dirty' if index&1 else 'Clean',bits(target),bits(low),bits(high),'-'
    for model in (definition('INVERSE-C0','2027-06-07',1,'0'),
                  definition('INVERSE-LONG','2040-06-07',30,'0')):
        for target in (50.,100.,200.,1e10):
            low = -1. if model['family']=='INVERSE-LONG' else (-1.99999999 if target==1e10 else -1.9)
            yield model,'Dirty',bits(target),bits(low),bits(100.),'-'


def inverse_rows(precision):
    with localcontext() as context:
        context.prec = precision
        for index,(model,kind,quote,low_bits,high_bits,display) in enumerate(inverse_inputs(),1):
            state = source(model)
            target_bits = rn(binary(quote)+(binary(rn(state['ai'])) if kind=='Clean' else 0))
            target = dec(binary(target_bits))
            low,high = binary(low_bits),binary(high_bits)
            assert target > 0
            error = uniform_error(state,low,high,precision)
            assert target > error, (model['family'],target,error)
            ll,lh = solve(state,target,low,high,precision)
            roots_lo = solve(state,target+error,low,high,precision)[0]
            roots_hi = solve(state,target-error,low,high,precision)[1]
            atol,rtol = binary(bits(1e-14)),binary(bits(1e-14))
            width = dec((atol+rtol*max(abs(low),abs(high)))*(1+4*U))
            cond_lo,cond_hi = max(dec(low),roots_lo-width),min(dec(high),roots_hi+width)
            method = 'bracketed-positive-sum'
            positive_flows = [(i,a,l) for i,a,l in state['bundles'] if a]
            if len(positive_flows)==1:
                _,amount,lam = positive_flows[0]
                closed = 2*((dec(amount)/target).ln()/dec(lam)).exp()-2
                closed_error = max(abs(closed),Decimal(1))*Decimal(10)**(-precision+30)
                assert ll-closed_error <= closed <= lh+closed_error
                method = 'closed-form-and-independent-bisection'
            slope = sum((dec(a)*(-dec(lam)*logarithm(high)).exp()*dec(lam/(2+high))
                         for _,a,lam in state['bundles']),Decimal(0))
            slope *= 1-Decimal(10)**(-precision+30)
            row = common(model,state,f'GILT-V-{index:05d}')
            items = (kind,hx(quote),hx(target_bits),hx(low_bits),hx(high_bits),hx(bits(1e-14)),
                     hx(bits(1e-14)),'500',str(int(binary(target_bits)>=p2(970))),
                     hx(outward(F(ll),True)),hx(outward(F(lh),False)),
                     hx(outward(F(cond_lo),True)),hx(outward(F(cond_hi),False)),
                     upper_text(error),lower_text(slope),upper_text(width),method,display)
            row.update(zip(INV_COLS[len(COMMON):],items))
            if display != '-':
                assert ll.quantize(Decimal('1e-9'),rounding=ROUND_HALF_UP) == lh.quantize(
                    Decimal('1e-9'),rounding=ROUND_HALF_UP) == Decimal(display)
            yield row


def fixture(rows, columns, family, generated_at, platform_text):
    records = '\n'.join('\t'.join(row[column] for column in columns) for row in rows)+'\n'
    body = '\t'.join(columns)+'\n'+records
    source_bytes = Path(__file__).read_bytes()
    requirements = ''.join(line+'\n' for line in (ROOT/'docs/REQUIREMENTS.md').read_text().splitlines()
                           if line.startswith('| FR-GILT-')).encode()
    assert len(requirements.splitlines())==24
    census = Counter(row['family'] for row in rows)
    fields = [
        'schema: 1',f'fixture-set: M8-GILT-{family}', 'domain: bonds',
        'description: Independent exact-source and conditional ideal comparison; no Rust production reads',
        'generator: validation/generator/gilt_ideal_oracles.py',
        'reference: exact Fraction source; dual-precision Decimal positive quasi sum, symbolic derivatives and target-driven roots',
        'reference-version: Python standard library', 'reference-source: docs/M8_GILT_CONTRACT.md',
        'generated-at: '+generated_at,
        'platform: '+platform_text,
        'reproduction: python3 -I validation/generator/gilt_ideal_oracles.py --check',
        'tolerance: TOL-GILT-ELEMENTARY conditional primitive hypotheses; exact source metadata TOL-E0; no enclosure certification',
        'generator-sha256: '+sha(source_bytes),'generator-fnv64: '+fnv(source_bytes),
        f'record-count: {len(rows)}','body-sha256: '+sha(body.encode()),'body-fnv64: '+fnv(body.encode()),
        'census: '+json.dumps(dict(sorted(census.items())),separators=(',',':')),
        'records-sha256: '+sha(records.encode()),'contract-m8-gilt-contract-sha256: '+sha(CONTRACT.read_bytes()),
        'contract-m8-gilt-contract-fnv64: '+fnv(CONTRACT.read_bytes()),
        'requirements-sha256: '+sha(requirements),'requirements-fnv64: '+fnv(requirements),
        'source-example-checker-sha256: '+SOURCE_CHECKER_SHA,
        'source-example-pdf-sha256: 8f491b8567c6f3a75ee2a7cbc02ccc4275810257df5533ed1fd349992465e018',
        'source-case-embedding: original issue set to previous quoted quasi date; synthetic valid definition, not historical issuance evidence',
        'decimal-precisions: 160,240','reference-serialization-significant-digits: 80',
        'inverse-bisections: 300','inverse-targets: original supplied quote plus one exact cached-AI binary clean shift',
        'financial-qualification: comparison bounds are conditional; uninformative is explicit, never pass-by-blanket-ULP',
        'metric-status-counts: '+json.dumps(dict(sorted(Counter(
            row[column] for row in rows for column in columns if column.endswith('_class')).items())),separators=(',',':')),
        'inverse-method-counts: '+json.dumps(dict(sorted(Counter(
            row['inverse_method'] for row in rows if 'inverse_method' in row).items())),separators=(',',':')),
    ]
    return ('\n'.join(fields)+'\n\n'+body).encode()


def generate(check=False):
    assert sha(CONTRACT.read_bytes()) == CONTRACT_SHA, 'adopted contract pin changed'
    assert sha((ROOT/'validation/generator/gilt_source_example_check.py').read_bytes()) == SOURCE_CHECKER_SHA
    self_checks()
    result = []
    for producer,columns,path,family in ((price_rows,PRICE_COLS,OUT,'IDEAL-PRICE-RISK'),
                                       (inverse_rows,INV_COLS,INV_OUT,'IDEAL-INVERSE')):
        low,high = list(producer(160)),list(producer(240))
        assert low == high, f'{family} dual-precision serialized-field disagreement'
        generated_at=datetime.now(timezone.utc).isoformat(timespec='seconds').replace('+00:00','Z')
        platform_text=f'{platform.system()} {platform.machine()}; Python {platform.python_version()}'
        if check and path.exists():
            metadata=dict(line.split(': ',1) for line in path.read_text().split('\n\n',1)[0].splitlines())
            generated_at,platform_text=metadata['generated-at'],metadata['platform']
        result.append((path,fixture(low,columns,family,generated_at,platform_text),len(low)))
    return result


def self_checks():
    """Independent primitive agreement and symbolic-derivative convergence."""
    inputs = [F(0),MIN/2,MIN*F(3,2),MIN*F(5,2),MAX,OVERFLOW,
              (OVERFLOW+MAX)/2,p2(-1022)-MIN/2,p2(-1022)+MIN/2]
    seed = 0x51a7c0de
    for _ in range(2048):
        seed = (seed*6364136223846793005+1442695040888963407) & ((1<<64)-1)
        exponent = int(seed%2300)-1150
        inputs.append(F((seed>>11)+1,((seed>>31)&0xffff)+1)*p2(exponent))
    for value in inputs:
        for signed in (value,-value):
            try:
                expected=bits(float(signed))
            except OverflowError:
                expected=(int(signed<0)<<63)|0x7ff0000000000000
            assert rn(signed)==expected, 'rational IEEE conversion cross-check'
    with localcontext() as context:
        context.prec=160
        for model in list(profiles())[:3]+[list(profiles())[15]]:
            state=source(model)
            y=F(1,20)
            values,_=ideal(model,state,y)
            previous=None
            for h in (F(1,1000),F(1,2000),F(1,4000)):
                lower,center,upper=(price_only(state,y+shift) for shift in (-h,F(0),h))
                d1=(upper-lower)/dec(2*h)
                d2=(upper-2*center+lower)/dec(h*h)
                errors=(abs(d1-values['d1']),abs(d2-values['d2']))
                if previous:
                    assert all(new<old/Decimal('3.9') for new,old in zip(errors,previous))
                previous=errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args = parser.parse_args()
    for path,data,count in generate(args.check):
        if args.check:
            assert path.read_bytes()==data, f'{path.name}: reproducibility mismatch'
        else:
            assert not path.exists(), f'refuse to overwrite existing oracle: {path}'
            path.write_bytes(data)
        print(f'{path.name}: rows={count} bytes={len(data)} sha256={sha(data)} check={args.check}',flush=True)


if __name__ == '__main__':
    main()
