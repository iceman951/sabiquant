#!/usr/bin/env python3
"""Independent closed-form one-flow yield inverse envelopes, additive only.

No Rust production, solver implementation or observed solver roots are read.
The shared NEW helper source supplies civil formulas/exact binary arithmetic;
this file derives inverses and uniform price-error bounds from the contracts.
"""
import argparse
from decimal import Decimal, localcontext
from fractions import Fraction as F
import hashlib
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
H = runpy.run_path(str(HERE / 'bond_yield_risk_extended_oracles.py'))
ROOT = H['ROOT']
OUT = ROOT / 'validation/fixtures/bonds/yield-inverse-extended.fixture'
binary, rn, power, hx, bits = (H[k] for k in ('binary', 'rn', 'power', 'hx', 'bits'))
dec, branch, ideal_flow = (H[k] for k in ('dec', 'branch', 'ideal_flow'))
U = F(1, 1 << 53)
MIN = power(-1074)
COLS = ('id family issue maturity settlement yield_context face_bits redemption_bits comp '
        'target_kind target_bits bracket_lo_bits bracket_hi_bits abs_tol_bits rel_tol_bits '
        'max_evals expected_scale root_lo_bits root_hi_bits ideal_root_bits dirty_target_bits '
        'price_error_bits time_lo_bits time_hi_bits solved_dirty_lo_bits solved_dirty_hi_bits').split()


def inverse(comp, amount, target, t, precision):
    assert amount > 0 and target > 0 and t > 0
    br, m = branch(comp, t)
    if br == 'Simple':
        return (amount / target - 1) / t, F(0)
    with localcontext() as ctx:
        ctx.prec = precision
        log_ratio = dec(amount / target).ln()
        if br == 'Continuous':
            value = log_ratio / dec(t)
        else:
            value = Decimal(m) * ((log_ratio / dec(m * t)).exp() - 1)
        epsilon = max(F(1), abs(F(value))) * F(Decimal(10) ** (-precision + 24))
        return F(value), epsilon


def error_bound(comp, amount, times, low, high, precision):
    """Uniform absolute pricing error on a fixed finite bracket.

    Exact rounded Simple q/reciprocal/product propagation is separate from the
    empirical E3 elementary criterion. Positive times keep each fixed branch's
    real discount monotone in yield, so extrema occur at bracket endpoints.
    """
    bounds = []
    for t in times:
        br, _ = branch(comp, t)
        if br == 'Simple':
            qlo, qhi = 1 + low * t, 1 + high * t
            assert qlo > 0
            eq = U * qhi + MIN / 2
            assert qlo > eq
            reciprocal_max = 1 / (qlo - eq)
            reciprocal_round = U * reciprocal_max + MIN / 2
            ed = eq / (qlo * (qlo - eq)) + reciprocal_round
            product_round = U * amount * (reciprocal_max + reciprocal_round) + MIN / 2
        else:
            candidates = [ideal_flow(comp, y, F(1), t, precision) for y in (low, high)]
            dmax = max(v[0] + max(F(1), abs(v[0])) * eps for v, eps in candidates)
            with localcontext() as ctx:
                ctx.prec = precision
                lmax = max(abs(F(dec(v[0]).ln())) for v, _ in candidates)
            ed = dmax * 8 * U * (1 + lmax) + MIN
            ed += max(max(F(1), abs(v[0])) * eps for v, eps in candidates)
            product_round = U * amount * (dmax + ed) + MIN / 2
        bounds.append(amount * ed + product_round)
    return max(bounds)


def ordered(b):
    return (~b & ((1 << 64) - 1)) if b & H['SIGN'] else b | H['SIGN']


def unorder(k):
    return (k & ~H['SIGN']) if k & H['SIGN'] else (~k & ((1 << 64) - 1))


def outward(x, lower):
    b = rn(x)
    represented = binary(b)
    if (lower and represented > x) or (not lower and represented < x):
        b = unorder(ordered(b) + (-1 if lower else 1))
    return b


def row(profile, comp, seed, face, amount, index, precision, near=False):
    start = H['dates'](profile['settlement'])[0]
    maturity = H['dates'](profile['raw_dates'][-1])[0]
    raw_time, times = H['fraction_candidates'](profile['yield_context'], start, maturity)
    if near:
        m = int(comp.split(':')[1]) if ':' in comp else 1
        low, high = F(-m) * (1 - power(-21)), F(-m) * (1 - power(-19))
    else:
        low, high = seed - F(1, 100), seed + F(1, 100)
    low, high = binary(rn(low)), binary(rn(high))
    ideal_seed, seed_epsilon = ideal_flow(comp, seed, amount, raw_time, precision)
    quote = rn(ideal_seed[0] * 100 / face)
    quote_error = max(F(1), abs(ideal_seed[0])) * seed_epsilon * 100 / face
    assert rn(ideal_seed[0] * 100 / face - quote_error) == quote
    assert rn(ideal_seed[0] * 100 / face + quote_error) == quote
    target = binary(rn(binary(quote) * face / 100))
    error = error_bound(comp, amount, times, low, high, precision)
    assert target > error
    tol_abs = binary(bits(1e-14))
    tol_rel = binary(bits(1e-14))
    # M6 full-width tolerance is abs+rel*max(|endpoints|). Its floating
    # operation order is bounded upwards, including either rounding, by4u.
    tolerance = (tol_abs + tol_rel * max(abs(low), abs(high))) * (1 + 4 * U)
    lower_roots, upper_roots = [], []
    for t in times:
        lv, le = inverse(comp, amount, target + error, t, precision)
        uv, ue = inverse(comp, amount, target - error, t, precision)
        lower_roots.append(lv - le)
        upper_roots.append(uv + ue)
    root_low = max(low, min(lower_roots) - tolerance)
    root_high = min(high, max(upper_roots) + tolerance)
    lb, hb = outward(root_low, True), outward(root_high, False)
    assert low <= binary(lb) <= binary(hb) <= high
    ideal_root, ideal_error = inverse(comp, amount, target, raw_time, precision)
    ideal_bits = rn(ideal_root)
    assert rn(ideal_root - ideal_error) == rn(ideal_root + ideal_error) == ideal_bits
    dirty_values = [ideal_flow(comp, y, amount, t, precision)
                    for y in (binary(lb), binary(hb)) for t in times]
    dirty_low = min(v[0] - max(F(1), abs(v[0])) * eps for v, eps in dirty_values) - error
    dirty_high = max(v[0] + max(F(1), abs(v[0])) * eps for v, eps in dirty_values) + error
    return dict(zip(COLS, (f'BYIX-{index:05d}', 'NEAR-MINUS-M' if near else 'CONTEXT-ALL32',
                          profile.get('issue', profile['raw_dates'][0]), profile['raw_dates'][-1],
                          profile['settlement'], profile['yield_context'], hx(rn(face)), hx(rn(amount)),
                          comp, 'Dirty' if index & 1 else 'Clean', hx(quote), hx(rn(low)), hx(rn(high)),
                          hx(rn(tol_abs)), hx(rn(tol_rel)), '400', str(int(abs(target) >= power(970))),
                          hx(lb), hx(hb), hx(ideal_bits), hx(rn(target)), hx(H['up_bits'](error)),
                          hx(rn(min(times))), hx(rn(max(times))), hx(outward(dirty_low, True)),
                          hx(outward(dirty_high, False)))))


def rows(precision):
    index = 0
    for profile in H['public_profiles']():
        if profile['kind'] != 'Zero':
            continue
        for comp in H['COMPS']:
            for seed in (binary(bits(.0375)), binary(bits(-.0625))):
                index += 1
                face, amount = (F(100), F(105)) if index & 1 else (F(300), F(1000))
                yield row(profile, comp, seed, face, amount, index, precision)
    for comp in H['COMPS']:
        m = int(comp.split(':')[1]) if ':' in comp else 1
        context = f'ICMA:{m}:2027-01-01,2028-01-01'
        profile = dict(raw_dates=('2027-01-01', '2028-01-01'), settlement='2027-01-01',
                       yield_context=context)
        index += 1
        seed = binary(rn(F(-m) * (1 - power(-20))))
        yield row(profile, comp, seed, F(100), F(105), index, precision, near=True)


def generate():
    lo, hi = list(rows(160)), list(rows(240))
    assert lo == hi, 'inverse160/240 field disagreement'
    # Use the schema1 serializer from the NEW shared independent helper, then
    # replace its self-source metadata with THIS source plus a pinned helper.
    helper = (HERE / 'bond_yield_risk_extended_oracles.py').read_bytes()
    source = Path(__file__).read_bytes()
    data = H['fixture_bytes'](lo, COLS, [
        'helper-sha256: ' + hashlib.sha256(helper).hexdigest(),
        'helper-fnv64: ' + H['fnv'](helper),
        'decimal-precisions: 160,240', 'compounding-count: 32',
        'inversion-model: exact one-payment closed forms, uniform gross price-error propagation and full-width step tolerance',
    ]).decode()
    data = data.replace('validation/generator/bond_yield_risk_extended_oracles.py',
                        'validation/generator/bond_yield_inverse_extended_oracles.py')
    data = data.replace(hashlib.sha256(helper).hexdigest(), hashlib.sha256(source).hexdigest(), 1)
    data = data.replace(H['fnv'](helper), H['fnv'](source), 1)
    return data.encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    data = generate()
    if args.check:
        assert OUT.read_bytes() == data, 'inverse fixture not reproducible'
    else:
        OUT.write_bytes(data)
    print(f'{len(data)} bytes sha256={hashlib.sha256(data).hexdigest()} fnv64={H["fnv"](data)} check={args.check}', flush=True)


if __name__ == '__main__':
    main()
