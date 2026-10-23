#!/usr/bin/env python3
"""Blind Canadian hand derivations: exact ordered-lattice search, not libm.

This producer reads only the adopted contract/component bytes for identity.
It never imports any Canadian oracle/production module. All selected examples
below use zero compound exponent or exact money-market rational factors.
"""
import argparse
import hashlib
import json
import struct
from fractions import Fraction as Q
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAX = 0x7fefffffffffffff
SIGN = 1 << 63
CERTS = []

def p2(e):
    return Q(2**e) if e >= 0 else Q(1, 2**(-e))

def rational(x):
    return [str(x.numerator), str(x.denominator)]

def exact(b):
    s = -1 if b & SIGN else 1
    b &= SIGN - 1
    e, t = b >> 52, b & ((1 << 52) - 1)
    if e == 2047:
        raise ValueError('nonfinite lattice point')
    return s * (Q(t) * p2(-1074) if not e else Q((1 << 52) + t) * p2(e - 1075))

def rn(x, label):
    """Nearest IEEE cell by binary search of ALL positive finite bit patterns."""
    if x == 0:
        return 0
    s = SIGN if x < 0 else 0
    x = abs(x)
    overflow = p2(1024) - p2(970)
    if x >= overflow:
        CERTS.append({'label': label, 'exact': rational(x), 'result': f'{s | 0x7ff0000000000000:016x}', 'cell': 'overflow midpoint or beyond'})
        return s | 0x7ff0000000000000
    lo, hi = 0, MAX
    while lo < hi:
        mid = (lo + hi + 1) // 2
        if exact(mid) <= x:
            lo = mid
        else:
            hi = mid - 1
    # MAX is allowed below its overflow midpoint.
    if lo == MAX:
        chosen = lo
        upper = overflow
    else:
        upper = exact(lo + 1)
        a, b = x - exact(lo), upper - x
        chosen = lo if a < b or (a == b and lo % 2 == 0) else lo + 1
    y = exact(chosen)
    cell_lo = Q(0) if chosen == 0 else (exact(chosen - 1) + y) / 2
    cell_hi = overflow if chosen == MAX else (y + exact(chosen + 1)) / 2
    assert cell_lo <= x <= cell_hi
    if x in (cell_lo, cell_hi):
        assert chosen % 2 == 0
    b = s | chosen
    CERTS.append({'label': label, 'exact': rational(-x if s else x), 'result': f'{b:016x}', 'cell_lo': rational(cell_lo), 'cell_hi': rational(cell_hi), 'tie': x in (cell_lo, cell_hi)})
    return b

def extended(x, label):
    """Locate normalized53 cell using rational scaling and integer midpoint."""
    if not x:
        return Q(0)
    s = -1 if x < 0 else 1
    a = abs(x)
    e = a.numerator.bit_length() - a.denominator.bit_length()
    if a < p2(e):
        e -= 1
    assert p2(e) <= a < p2(e + 1)
    step = p2(e - 52)
    scaled = a / step
    q, r = divmod(scaled.numerator, scaled.denominator)
    assert (1 << 52) <= q < (1 << 53)
    if 2 * r > scaled.denominator or (2 * r == scaled.denominator and q % 2):
        q += 1
    result = s * q * step
    assert abs(result - x) <= step / 2
    if abs(result - x) == step / 2:
        assert q % 2 == 0
    # Keep extended exponents; this branch never gates a tiny term to IEEE zero.
    CERTS.append({'label': label, 'exact': rational(x), 'extended': rational(result), 'step': rational(step), 'tie': abs(result - x) == step / 2})
    return result

def selected_risks(weights, convention, f, y, lambdas=None, days=None, remaining=None):
    D = exact(rn(sum(weights), 'dirty reduction'))
    held = [Q(0), Q(0), Q(0)]
    for i, w in enumerate(weights):
        if convention == 'Compound':
            lam = lambdas[i]
            delta = exact(rn(Q(f) + y, 'represented delta'))
            raw = [-w * lam / delta, w * lam * (lam + 1) / delta**2, w * lam / f]
        else:
            tau, rho = Q(days, 365), Q(remaining[i], 365)
            h, d, n = tau - rho, 1 + y * tau, 1 + y * rho
            assert h > 0 and d > 0 and n > 0
            raw = [-w * h / (d * n), 2 * w * tau * h / (d*d*n), w * h]
        for j, v in enumerate(raw):
            held[j] += extended(v, f'flow{i} retained{j}')
    views = held + [-held[0] / D, held[2] / D, held[1] / D]
    return [rn(v, f'getter{i}') for i, v in enumerate(views)]

def as_hex(bs):
    return [f'{b:016x}' for b in bs]

def run(destination, check):
    rows = []
    for f, days in [(2, 184), (4, 92), (12, 31)]:
        for c in ('Compound', 'MoneyMarket'):
            B = Q(100 + 12 // f)
            bits = selected_risks([B], c, f, Q(0), [Q(1)], days, [0])
            rows.append({'id': f'zero-f{f}-{c}', 'frequency': f, 'days': days, 'convention': c, 'dirty_bits': f'{rn(B, "zero dirty"):016x}', 'six_bits': as_hex(bits)})
            if c=='Compound':
                for y in [p2(-1074),-p2(-1074)]:
                    z=rn(-y/f,'tiny signed exponent')
                    assert (z & (SIGN-1))==0
                    assert exact(rn(Q(f)+y,'tiny represented delta'))==f
    for y in [Q(-300), Q(-280), exact(MAX)]:
        # D=I=2025-01-01, M=T=2025-07-01, S=2025-06-30.
        factor = exact(rn(Q(365)/(365+y), 'one-day factor'))
        pv = exact(rn(100*factor, 'one-day PV'))
        bits = selected_risks([pv], 'MoneyMarket', 2, y, days=1, remaining=[0])
        rows.append({'id': 'one-day-' + (str(int(y)) if y != exact(MAX) else 'MAX'), 'yield_bits': f'{rn(y, "input y"):016x}', 'factor_bits': f'{rn(factor, "factor confirm"):016x}', 'dirty_bits': f'{rn(pv, "PV confirm"):016x}', 'six_bits': as_hex(bits), 'underflow_getters': [i for i, b in enumerate(bits) if (b & (SIGN-1)) == 0]})
    # Two regular coupons, settlement halfway through the184-day first cell.
    # Value accrued3; payable accrued1104/365. Future exponents1/2 and3/2.
    A, AI = Q(3), Q(1104,365)
    ahat, aihat = exact(rn(A,'A cache')), exact(rn(AI,'AI cache'))
    for c in ('Compound','MoneyMarket'):
        dirty = Q(112)
        qa = ahat if c == 'Compound' else aihat
        clean = rn(dirty-qa, 'clean bridge')
        settlement = rn(dirty-qa+aihat, 'three-term settlement bridge')
        risks = selected_risks([Q(6),Q(106)],c,2,Q(0),[Q(1,2),Q(3,2)],273,[181,0])
        targets = [rn(dirty,'model target'),clean,settlement]
        converted = [rn(exact(targets[0]),'resolve ModelDirty'),rn(exact(targets[1])+qa,'resolve Clean'),rn(exact(targets[2])-aihat+qa,'resolve SettlementDirty')]
        assert all(exact(b)==112 for b in converted)
        rows.append({'id':'midcell-'+c,'convention':c,'a_bits':f'{rn(ahat,"A confirm"):016x}','ai_bits':f'{rn(aihat,"AI confirm"):016x}','clean_bits':f'{clean:016x}','settlement_bits':f'{settlement:016x}','six_bits':as_hex(risks),'target_bits':as_hex(targets)})
    # Reviewed MM positive-image witness; derive all five selected factor/PV cells.
    # C1, f2, S Jun1, T Mar1 2026. D273, nonterminal R181; B=[1/2,201/2].
    y = exact(MAX)
    factors = [exact(rn((365+y*R)/(365+y*273), f'image factor R{R}')) for R in [181,0]]
    pvs = [exact(rn(b*t,'image PV')) for b,t in zip([Q(1,2),Q(201,2)],factors)]
    dirty = rn(sum(pvs),'image dirty')
    assert dirty == 0x3fd5375375375375
    floor = Q(181,546)
    assert exact(dirty) < floor
    rows.append({'id':'observed-image-upper','factor_bits':[f'{rn(t,"image factor confirm"):016x}' for t in factors], 'pv_bits':[f'{rn(t,"image PV confirm"):016x}' for t in pvs], 'dirty_bits':f'{dirty:016x}','ideal_floor':rational(floor),'below_floor_gap':rational(floor-exact(dirty)),'functions':2,'derivatives':0})
    # Positive target resolution from signed/zero external evidence: f2 Sep1-Mar1.
    # 181-day cell, elapsed1. A6/181 exceeds AI12/365; compound SettlementDirty0 valid.
    a, ai = exact(rn(Q(6,181),'early A')), exact(rn(Q(12,365),'early AI'))
    checks = []
    for basis,v,shift in [('Clean',Q(0),a),('Clean',-p2(-1074),a),('SettlementDirty',Q(0),a-ai)]:
        resolved = rn(v+shift,'signed target resolve')
        assert exact(resolved) > 0
        checks.append({'basis':basis,'external_bits':f'{rn(v,"external signed quote"):016x}','resolved_bits':f'{resolved:016x}'})
    rows.append({'id':'signed-targets','a_bits':f'{rn(a,"early A confirm"):016x}','ai_bits':f'{rn(ai,"early AI confirm"):016x}','checks':checks})
    # True terminal delay: C0, y0, terminal h=2/365, W100. Retained moment
    # is observed before division; do not assume ideal2/365 is the same bits.
    delayed=selected_risks([Q(100)],'MoneyMarket',2,Q(0),days=2,remaining=[0])
    rows.append({'id':'true-maturity-two-days','six_bits':as_hex(delayed)})
    # A second, independent check of rounded IEEE values, never the primary route.
    cross = 0
    for cert in CERTS:
        if 'result' not in cert:
            continue
        q = Q(*map(int,cert['exact']))
        expected = int(cert['result'],16)
        if (expected>>52)&2047==2047:
            continue
        native = struct.unpack('>Q',struct.pack('>d',float(q)))[0]
        assert native == expected
        cross += 1
    data = {'contract_sha256':hashlib.sha256((ROOT/'docs/M8_CANADIAN_CONTRACT.md').read_bytes()).hexdigest(),'cases':rows,'certificates':CERTS,'python_float_corroborations':cross,'derivation':'exact Fraction; nearest IEEE chosen by binary search over finite bit lattice; full terms normalized53 then independent getters; no elementary functions'}
    if data['contract_sha256']!='4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3':
        raise ValueError('adopted Canadian contract mismatch')
    encoded = (json.dumps(data,sort_keys=True,indent=2)+'\n').encode('ascii')
    if check:
        if destination.read_bytes()!=encoded:raise ValueError('changed derivation bytes')
    else:
        with destination.open('xb') as file:
            file.write(encoded)
    print(json.dumps({'cases':len(rows),'exact_rounding_certificates':len(CERTS),'python_float_corroborations':cross,'sha256':hashlib.sha256(encoded).hexdigest()}))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'validation/hand/canadian_numeric_hand_derivations.json')
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    run(args.output,args.check)
