#!/usr/bin/env python3
"""Independent M8 Phase D bond yield-to-maturity and analytical risk oracle generator.

Computes independent exact Fraction + high-precision Decimal (160 and 240 digits)
references and staged binary64 / extended-wide integer oracles for FR-YR-01..16.
No Rust or QuantLib production output is imported.
"""
import argparse
import calendar
import collections
from datetime import date
from decimal import Decimal, localcontext
from fractions import Fraction as F
from functools import lru_cache
import hashlib
import json
import math
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / 'validation/fixtures/bonds/yield-risk-oracles.fixture'
PRECISIONS = (160, 240)
SIGN, INF = 1 << 63, 0x7ff0000000000000
TWO_POW_970_BITS = 0x7c90000000000000
FREQUENCIES = (1, 2, 3, 4, 6, 12, 13, 26, 52, 365)
COMPS = ('Simple', 'Continuous') + tuple(
    f'{method}:{f}'
    for method in ('Compounded', 'SimpleThenCompounded', 'CompoundedThenSimple')
    for f in FREQUENCIES
)
CONTRACTS = (
    'docs/M8_BOND_YIELD_RISK_CONTRACT.md',
    'docs/M8_BOND_CORE_CONTRACT.md',
    'docs/M3_ICMA_CONTRACT.md',
    'docs/M3_DAYCOUNT_EXTENSIONS.md',
    'docs/M4_DATE_CONTRACT.md',
    'docs/M7_CASHFLOW_CONTRACT.md',
)
COLUMNS = (
    'id family profile issue settlement inclusion daycount comp face_bits accrued_bits '
    'flows target_kind target_mode target_value_bits bracket_lo_bits bracket_hi_bits '
    'abs_step_tol_bits rel_step_tol_bits max_evals risk_yield_bits '
    'yield_expected yield_error_kind yield_error_phase yield_error_stage yield_error_evals '
    'sensitivity residual_scale_exp eval_count termination '
    'converted_target_bits dirty_target_bits solved_yield_lo_bits solved_yield_hi_bits '
    'solved_dirty_lo_bits solved_dirty_hi_bits solved_clean_lo_bits solved_clean_hi_bits '
    'solve_residual_lo_bits solve_residual_hi_bits amount_residual_lo_bits amount_residual_hi_bits '
    'ideal_root_bits '
    'risk_expected risk_error_kind risk_error_phase risk_error_index '
    'risk_dirty_lo_bits risk_dirty_hi_bits '
    'p1_status p1_lo_bits p1_hi_bits p2_status p2_lo_bits p2_hi_bits '
    'moment_status moment_lo_bits moment_hi_bits '
    'mod_dur_status mod_dur_lo_bits mod_dur_hi_bits '
    'mac_dur_status mac_dur_lo_bits mac_dur_hi_bits '
    'conv_status conv_lo_bits conv_hi_bits '
    'ideal_p1_bits ideal_p2_bits ideal_moment_bits '
    'ideal_mod_dur_bits ideal_mac_dur_bits ideal_conv_bits'
).split()


def sha(b):
    return hashlib.sha256(b).hexdigest()


def fnv(b):
    h = 0xcbf29ce484222325
    for x in b:
        h = ((h ^ x) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{h:016x}'


def hx(b):
    return f'0x{b:016x}'


def input_bits(x):
    return struct.unpack('>Q', struct.pack('>d', float(x)))[0]


def bits_to_float(b):
    return struct.unpack('>d', struct.pack('>Q', b))[0]


def canonical_zero_bits(b):
    return 0 if (b & ~SIGN) == 0 else b


def pow2(p):
    return F(1 << p) if p >= 0 else F(1, 1 << -p)


def binary(b):
    s, e, m = bool(b & SIGN), (b >> 52) & 2047, b & ((1 << 52) - 1)
    assert e != 2047, hx(b)
    if e == 0 and m == 0:
        return F(0)
    if e:
        m += 1 << 52
        p = e - 1075
    else:
        p = -1074
    x = F(m << p) if p >= 0 else F(m, 1 << -p)
    return -x if s else x


def rn_status(x):
    """Round exact Fraction x to IEEE-754 binary64 bits with status ('ok', 'underflow', 'overflow')."""
    if x == 0:
        return 'ok', 0
    neg, a = x < 0, abs(x)
    top = a.numerator.bit_length() - a.denominator.bit_length()
    if a < pow2(top):
        top -= 1
    unit = max(-1074, top - 52)
    z = a / pow2(unit)
    q, r = divmod(z.numerator, z.denominator)
    if 2 * r > z.denominator or (2 * r == z.denominator and q % 2 == 1):
        q += 1
    if q == 0:
        return 'underflow', 0
    top_q = q.bit_length() - 1 + unit
    if top_q > 1023:
        return 'overflow', INF | (SIGN if neg else 0)
    if top_q < -1022:
        bits = q << (unit + 1074)
    else:
        k = q.bit_length() - 53
        m = q >> k if k >= 0 else q << -k
        bits = ((top_q + 1023) << 52) | (m - (1 << 52))
    return 'ok', bits | (SIGN if neg else 0)


def rn(x):
    st, b = rn_status(x)
    assert st == 'ok', (st, x)
    return b


def next_bits(b, direction):
    b = canonical_zero_bits(b)
    if b == 0:
        return 1 if direction > 0 else (SIGN | 1)
    neg = bool(b & SIGN)
    mag = b & ~SIGN
    if not neg:
        if direction > 0:
            return min(INF - 1, mag + 1)
        return 0 if mag == 1 else mag - 1
    else:
        if direction < 0:
            return SIGN | min(INF - 1, mag + 1)
        return 0 if mag == 1 else (SIGN | (mag - 1))


def step_bits(b, ulps):
    cur = canonical_zero_bits(b)
    step = 1 if ulps >= 0 else -1
    for _ in range(abs(ulps)):
        cur = next_bits(cur, step)
    return cur


def dec_from_frac(x):
    return Decimal(x.numerator) / Decimal(x.denominator)


def dec_to_frac(d):
    s, digits, exp = d.as_tuple()
    num = 0
    for dg in digits:
        num = num * 10 + dg
    if s:
        num = -num
    return F(num * (10 ** exp), 1) if exp >= 0 else F(num, 10 ** (-exp))


def dec_ln(x):
    return x.ln()


def dec_exp(x):
    return x.exp()


@lru_cache(None)
def year_fraction(dc, start_iso, end_iso):
    s = date.fromisoformat(start_iso)
    e = date.fromisoformat(end_iso)
    assert s <= e
    if dc == 'A365F':
        return F((e - s).days, 365)
    if dc == 'A360':
        return F((e - s).days, 360)
    if dc == '30360':
        d1, d2 = s.day, e.day
        if d2 == 31 and d1 in (30, 31):
            d2 = 30
        if d1 == 31:
            d1 = 30
        return F(360 * (e.year - s.year) + 30 * (e.month - s.month) + (d2 - d1), 360)
    if dc.startswith('30E360ISDA:'):
        if s == e:
            return F(0)
        term = date.fromisoformat(dc.split(':')[1])
        d1, d2 = s.day, e.day
        f1 = s.month == 2 and d1 == calendar.monthrange(s.year, 2)[1]
        f2 = e.month == 2 and d2 == calendar.monthrange(e.year, 2)[1]
        if d1 == 31 or f1:
            d1 = 30
        if d2 == 31 or (f2 and e != term):
            d2 = 30
        return F(360 * (e.year - s.year) + 30 * (e.month - s.month) + (d2 - d1), 360)
    if dc.startswith('ICMA:'):
        _, freq_s, bounds_s = dc.split(':')
        freq = int(freq_s)
        bs = [date.fromisoformat(x) for x in bounds_s.split(',')]
        total = F(0)
        for a, b in zip(bs, bs[1:]):
            lo, hi = max(s, a), min(e, b)
            if lo < hi:
                total += F((hi - lo).days, freq * (b - a).days)
        return total
    raise AssertionError(dc)


def active_branch(comp, t_frac):
    if comp == 'Simple':
        return ('Simple', None)
    if comp == 'Continuous':
        return ('Continuous', None)
    method, f_s = comp.split(':')
    freq = int(f_s)
    cutoff = F(1, freq)
    if method == 'Compounded':
        return ('Periodic', freq)
    if method == 'SimpleThenCompounded':
        return ('Simple', None) if t_frac < cutoff else ('Periodic', freq)
    if method == 'CompoundedThenSimple':
        return ('Simple', None) if t_frac > cutoff else ('Periodic', freq)
    raise AssertionError(comp)


def validate_yield_quote(comp, y_bits):
    if (y_bits & INF) == INF:
        return ('bond.rate', 'yield quote')
    if comp.startswith(('Compounded:', 'SimpleThenCompounded:', 'CompoundedThenSimple:')):
        freq = int(comp.split(':')[1])
        m_frac = F(freq)
        y_frac = binary(y_bits)
        per_period = binary(rn(y_frac / m_frac))
        base = binary(rn(F(1) + per_period))
        if base <= 0:
            return ('bond.rate', 'yield quote')
    return None


def staged_flow_observation(comp, y_bits, amount_bits, t_bits, prec):
    """Compute both staged binary64 observation and ideal high-precision Decimal quantities."""
    with localcontext() as ctx:
        ctx.prec = prec
        y_f = binary(y_bits)
        c_f = binary(amount_bits)
        t_f = binary(t_bits)
        branch, freq = active_branch(comp, t_f)
        if t_f == 0:
            return {
                'ok': True,
                'branch': branch,
                'freq': freq,
                'pv_bits': canonical_zero_bits(amount_bits),
                'q_bits': input_bits(1.0),
                'w_bits': 0,
                'm_bits': input_bits(float(freq)) if freq else 0,
                'delta_bits': input_bits(1.0),
                'ideal_pv': dec_from_frac(c_f),
                'ideal_p1': Decimal(0),
                'ideal_p2': Decimal(0),
                'ideal_moment': Decimal(0),
            }
        y_d = dec_from_frac(y_f)
        c_d = dec_from_frac(c_f)
        t_d = dec_from_frac(t_f)
        if branch == 'Simple':
            q_st, q_bits = rn_status(F(1) + y_f * t_f)
            if q_st != 'ok' or binary(q_bits) <= 0:
                return {'ok': False, 'phase': 'discount factor'}
            q_f = binary(q_bits)
            df_st, df_bits = rn_status(F(1) / q_f)
            if df_st != 'ok' or binary(df_bits) <= 0:
                return {'ok': False, 'phase': 'discount factor'}
            pv_st, pv_bits = rn_status(c_f * binary(df_bits))
            if pv_st == 'overflow':
                return {'ok': False, 'phase': 'discount factor'}
            q_d = Decimal(1) + y_d * t_d
            pv_d = c_d / q_d
            p1_d = -pv_d * t_d / q_d
            p2_d = Decimal(2) * pv_d * t_d * t_d / (q_d * q_d)
            mom_d = pv_d * t_d
            return {
                'ok': True,
                'branch': 'Simple',
                'freq': None,
                'pv_bits': canonical_zero_bits(pv_bits),
                'q_bits': q_bits,
                'w_bits': 0,
                'm_bits': 0,
                'delta_bits': 0,
                'ideal_pv': pv_d,
                'ideal_p1': p1_d,
                'ideal_p2': p2_d,
                'ideal_moment': mom_d,
            }
        if branch == 'Continuous':
            if y_f == 0:
                df_bits = input_bits(1.0)
            else:
                rt_st, rt_bits = rn_status(y_f * t_f)
                if rt_st == 'overflow':
                    return {'ok': False, 'phase': 'discount factor'}
                cf_d_staged = dec_exp(dec_from_frac(binary(rt_bits)))
                cf_st, cf_bits = rn_status(dec_to_frac(cf_d_staged))
                if cf_st != 'ok' or binary(cf_bits) <= 0:
                    return {'ok': False, 'phase': 'discount factor'}
                df_st, df_bits = rn_status(F(1) / binary(cf_bits))
                if df_st != 'ok' or binary(df_bits) <= 0:
                    return {'ok': False, 'phase': 'discount factor'}
            pv_st, pv_bits = rn_status(c_f * binary(df_bits))
            if pv_st != 'ok' and c_f != 0:
                return {'ok': False, 'phase': 'discount factor'}
            pv_d = c_d * dec_exp(-y_d * t_d)
            p1_d = -pv_d * t_d
            p2_d = pv_d * t_d * t_d
            mom_d = pv_d * t_d
            return {
                'ok': True,
                'branch': 'Continuous',
                'freq': None,
                'pv_bits': canonical_zero_bits(pv_bits),
                'q_bits': 0,
                'w_bits': 0,
                'm_bits': 0,
                'delta_bits': 0,
                'ideal_pv': pv_d,
                'ideal_p1': p1_d,
                'ideal_p2': p2_d,
                'ideal_moment': mom_d,
            }
        # Periodic
        m_f = F(freq)
        m_bits = rn(m_f)
        per_st, per_bits = rn_status(y_f / m_f)
        if per_st == 'overflow':
            return {'ok': False, 'phase': 'discount factor'}
        base_st, base_bits = rn_status(F(1) + binary(per_bits))
        if base_st != 'ok' or binary(base_bits) <= 0:
            return {'ok': False, 'phase': 'discount factor'}
        delta_st, delta_bits = rn_status(m_f + y_f)
        if delta_st != 'ok' or binary(delta_bits) <= 0:
            return {'ok': False, 'phase': 'discount factor'}
        w_st, w_bits = rn_status(t_f + binary(rn(F(1) / m_f)))
        if w_st == 'overflow' or binary(w_bits) <= 0:
            return {'ok': False, 'phase': 'discount factor'}
        if y_f == 0:
            df_bits = input_bits(1.0)
        else:
            scaled_f = binary(per_bits)
            if abs(scaled_f) < pow2(-52):
                pg_bits = y_bits
            elif scaled_f <= F(-1, 2):
                ratio_st, ratio_bits = rn_status(binary(delta_bits) / m_f)
                if ratio_st != 'ok' or binary(ratio_bits) <= 0:
                    return {'ok': False, 'phase': 'discount factor'}
                ln_d = dec_ln(dec_from_frac(binary(ratio_bits)))
                ln_st, ln_bits = rn_status(dec_to_frac(ln_d))
                if ln_st == 'overflow':
                    return {'ok': False, 'phase': 'discount factor'}
                pg_st, pg_bits = rn_status(m_f * binary(ln_bits))
                if pg_st == 'overflow':
                    return {'ok': False, 'phase': 'discount factor'}
            else:
                ln1p_d = dec_ln(Decimal(1) + dec_from_frac(scaled_f))
                ln1p_st, ln1p_bits = rn_status(dec_to_frac(ln1p_d))
                if ln1p_st == 'overflow':
                    return {'ok': False, 'phase': 'discount factor'}
                pg_st, pg_bits = rn_status(m_f * binary(ln1p_bits))
                if pg_st == 'overflow':
                    return {'ok': False, 'phase': 'discount factor'}
            rt_st, rt_bits = rn_status(binary(pg_bits) * t_f)
            if rt_st == 'overflow':
                return {'ok': False, 'phase': 'discount factor'}
            cf_d_staged = dec_exp(dec_from_frac(binary(rt_bits)))
            cf_st, cf_bits = rn_status(dec_to_frac(cf_d_staged))
            if cf_st != 'ok' or binary(cf_bits) <= 0:
                return {'ok': False, 'phase': 'discount factor'}
            df_st, df_bits = rn_status(F(1) / binary(cf_bits))
            if df_st != 'ok' or binary(df_bits) <= 0:
                return {'ok': False, 'phase': 'discount factor'}
        pv_st, pv_bits = rn_status(c_f * binary(df_bits))
        if pv_st != 'ok' and c_f != 0:
            return {'ok': False, 'phase': 'discount factor'}
        m_d = Decimal(freq)
        delta_f = F(1) + y_f / m_f
        mt_f = m_f * t_f
        if mt_f.denominator == 1 and mt_f >= 0:
            df_exact = F(1) / (delta_f ** int(mt_f))
            pv_exact = c_f * df_exact
            p1_exact = -pv_exact * t_f / delta_f
            p2_exact = pv_exact * t_f * (t_f + F(1, freq)) / (delta_f * delta_f)
            mom_exact = pv_exact * t_f
            pv_d = dec_from_frac(pv_exact)
            p1_d = dec_from_frac(p1_exact)
            p2_d = dec_from_frac(p2_exact)
            mom_d = dec_from_frac(mom_exact)
        else:
            delta_d = Decimal(1) + y_d / m_d
            pv_d = c_d * dec_exp((-m_d * t_d) * dec_ln(delta_d))
            p1_d = -pv_d * t_d / delta_d
            p2_d = pv_d * t_d * (t_d + Decimal(1) / m_d) / (delta_d * delta_d)
            mom_d = pv_d * t_d
        return {
            'ok': True,
            'branch': 'Periodic',
            'freq': freq,
            'pv_bits': canonical_zero_bits(pv_bits),
            'q_bits': 0,
            'w_bits': w_bits,
            'm_bits': m_bits,
            'delta_bits': delta_bits,
            'ideal_pv': pv_d,
            'ideal_p1': p1_d,
            'ideal_p2': p2_d,
            'ideal_moment': mom_d,
        }


def extended_term_fraction(num_bits_list, den_bits_list, shift_exp, neg):
    """Exact 53-bit extended-exponent significand rounding in base 2^-8192."""
    num_f = F(1 << shift_exp)
    for b in num_bits_list:
        v = abs(binary(b))
        if v == 0:
            return F(0)
        num_f *= v
    den_f = F(1)
    for b in den_bits_list:
        v = abs(binary(b))
        assert v > 0
        den_f *= v
    ratio = num_f / den_f
    top = ratio.numerator.bit_length() - ratio.denominator.bit_length()
    if ratio < pow2(top):
        top -= 1
    unit = top - 52
    scaled = ratio / pow2(unit)
    q, r = divmod(scaled.numerator, scaled.denominator)
    if 2 * r > scaled.denominator or (2 * r == scaled.denominator and q % 2 == 1):
        q += 1
    val = F(q) * pow2(unit)
    return -val if neg else val


def evaluate_risk_oracle(comp, y_bits, flows, prec):
    err = validate_yield_quote(comp, y_bits)
    if err is not None:
        return {
            'expected': 'err',
            'error_kind': err[0],
            'error_phase': err[1],
            'error_index': '-',
        }
    pv_sum = F(0)
    p1_sum = F(0)
    p2_sum = F(0)
    mom_sum = F(0)
    ideal_pv_sum = Decimal(0)
    ideal_p1_sum = Decimal(0)
    ideal_p2_sum = Decimal(0)
    ideal_mom_sum = Decimal(0)
    for idx, (amt_bits, t_bits) in enumerate(flows):
        obs = staged_flow_observation(comp, y_bits, amt_bits, t_bits, prec)
        if not obs['ok']:
            return {
                'expected': 'err',
                'error_kind': 'bond.rate',
                'error_phase': obs['phase'],
                'error_index': str(idx),
            }
        pv_bits = obs['pv_bits']
        pv_f = binary(pv_bits)
        pv_sum += pv_f
        ideal_pv_sum += obs['ideal_pv']
        ideal_p1_sum += obs['ideal_p1']
        ideal_p2_sum += obs['ideal_p2']
        ideal_mom_sum += obs['ideal_moment']
        if binary(t_bits) == 0 or pv_f == 0:
            continue
        v_neg = bool(pv_bits & SIGN)
        mom_sum += extended_term_fraction([pv_bits, t_bits], [], 0, v_neg)
        if obs['branch'] == 'Simple':
            q_bits = obs['q_bits']
            p1_sum += extended_term_fraction([pv_bits, t_bits], [q_bits], 0, not v_neg)
            p2_sum += extended_term_fraction([pv_bits, t_bits, t_bits], [q_bits, q_bits], 1, v_neg)
        elif obs['branch'] == 'Continuous':
            p1_sum += extended_term_fraction([pv_bits, t_bits], [], 0, not v_neg)
            p2_sum += extended_term_fraction([pv_bits, t_bits, t_bits], [], 0, v_neg)
        else:
            w_bits = obs['w_bits']
            m_bits = obs['m_bits']
            delta_bits = obs['delta_bits']
            p1_sum += extended_term_fraction([pv_bits, t_bits, m_bits], [delta_bits], 0, not v_neg)
            p2_sum += extended_term_fraction(
                [pv_bits, t_bits, w_bits, m_bits, m_bits],
                [delta_bits, delta_bits],
                0,
                v_neg,
            )

    dirty_st, dirty_bits = rn_status(pv_sum)
    if dirty_st == 'overflow':
        return {
            'expected': 'err',
            'error_kind': 'bond.sum',
            'error_phase': 'dirty price reduction',
            'error_index': '-',
        }
    dirty_f = binary(dirty_bits)
    p1_st, p1_bits = rn_status(p1_sum)
    p2_st, p2_bits = rn_status(p2_sum)
    mom_st, mom_bits = rn_status(mom_sum)
    if dirty_f == 0:
        mod_st, mod_bits = ('zero_dirty', 0)
        mac_st, mac_bits = ('zero_dirty', 0)
        conv_st, conv_bits = ('zero_dirty', 0)
    else:
        mod_st, mod_bits = rn_status((-p1_sum) / dirty_f)
        mac_st, mac_bits = rn_status(mom_sum / dirty_f)
        conv_st, conv_bits = rn_status(p2_sum / dirty_f)

    with localcontext() as ctx:
        ctx.prec = prec
        if ideal_pv_sum == 0:
            ideal_mod_bits = 0
            ideal_mac_bits = 0
            ideal_conv_bits = 0
        else:
            _, ideal_mod_bits = rn_status(dec_to_frac((-ideal_p1_sum) / ideal_pv_sum))
            _, ideal_mac_bits = rn_status(dec_to_frac(ideal_mom_sum / ideal_pv_sum))
            _, ideal_conv_bits = rn_status(dec_to_frac(ideal_p2_sum / ideal_pv_sum))
        _, ideal_p1_bits = rn_status(dec_to_frac(ideal_p1_sum))
        _, ideal_p2_bits = rn_status(dec_to_frac(ideal_p2_sum))
        _, ideal_mom_bits = rn_status(dec_to_frac(ideal_mom_sum))

    # Allow 2 ULPs on staged binary64 outputs when exponential/pow libm enters, 0 ULP on Simple
    ulp_pad = 0 if all(active_branch(comp, binary(tb))[0] == 'Simple' for _, tb in flows if binary(tb) > 0) else 2

    def bounds(st, b):
        if st != 'ok':
            return st, 0, 0
        return 'ok', step_bits(b, -ulp_pad), step_bits(b, ulp_pad)

    p1_s, p1_lo, p1_hi = bounds(p1_st, p1_bits)
    p2_s, p2_lo, p2_hi = bounds(p2_st, p2_bits)
    mom_s, mom_lo, mom_hi = bounds(mom_st, mom_bits)
    mod_s, mod_lo, mod_hi = bounds(mod_st, mod_bits)
    mac_s, mac_lo, mac_hi = bounds(mac_st, mac_bits)
    conv_s, conv_lo, conv_hi = bounds(conv_st, conv_bits)
    dirty_lo = step_bits(dirty_bits, -ulp_pad)
    dirty_hi = step_bits(dirty_bits, ulp_pad)

    return {
        'expected': 'ok',
        'error_kind': '-',
        'error_phase': '-',
        'error_index': '-',
        'dirty_lo': dirty_lo,
        'dirty_hi': dirty_hi,
        'p1': (p1_s, p1_lo, p1_hi),
        'p2': (p2_s, p2_lo, p2_hi),
        'moment': (mom_s, mom_lo, mom_hi),
        'mod_dur': (mod_s, mod_lo, mod_hi),
        'mac_dur': (mac_s, mac_lo, mac_hi),
        'conv': (conv_s, conv_lo, conv_hi),
        'ideal_p1_bits': ideal_p1_bits,
        'ideal_p2_bits': ideal_p2_bits,
        'ideal_moment_bits': ideal_mom_bits,
        'ideal_mod_dur_bits': ideal_mod_bits,
        'ideal_mac_dur_bits': ideal_mac_bits,
        'ideal_conv_bits': ideal_conv_bits,
    }


def classify_sensitivity(comp, flows):
    keys = []
    all_nonneg = True
    has_pos = False
    for amt_bits, t_bits in flows:
        amt = binary(amt_bits)
        t = binary(t_bits)
        if amt < 0:
            all_nonneg = False
        if amt != 0 and t > 0:
            if amt > 0:
                has_pos = True
            br = active_branch(comp, t)
            keys.append((br, canonical_zero_bits(t_bits), amt_bits & ~SIGN, amt < 0))
    keys.sort(key=lambda x: (str(x[0]), x[1], x[2], x[3]))
    # Check pairing
    i = 0
    paired = True
    while i < len(keys):
        br, tb, mb, _ = keys[i]
        pos_c = 0
        neg_c = 0
        while i < len(keys) and keys[i][:3] == (br, tb, mb):
            if keys[i][3]:
                neg_c += 1
            else:
                pos_c += 1
            i += 1
        if pos_c != neg_c:
            paired = False
            break
    if paired:
        return 'Constant'
    if all_nonneg and has_pos:
        return 'MathematicallyStrictlyDecreasing'
    return 'Uncertified'


def eval_dirty_at(comp, y_bits, flows, prec):
    err = validate_yield_quote(comp, y_bits)
    if err is not None:
        return None
    s = F(0)
    for amt_bits, t_bits in flows:
        obs = staged_flow_observation(comp, y_bits, amt_bits, t_bits, prec)
        if not obs['ok']:
            return None
        s += binary(obs['pv_bits'])
    st, b = rn_status(s)
    if st != 'ok':
        return None
    return b


def f64_midpoint(a, b):
    hi = 1.7976931348623157e308 / 2.0
    lo = 2.2250738585072014e-308 * 2.0
    if abs(a) <= hi and abs(b) <= hi:
        return (a + b) / 2.0
    if abs(a) < lo:
        return a + b / 2.0
    if abs(b) < lo:
        return a / 2.0 + b
    return a / 2.0 + b / 2.0


def f64_zero(v):
    return 0.0 if v == 0.0 else v


def f64_same_sign(a, b):
    return (a < 0.0) == (b < 0.0)


def f64_inverse_trial(previous, best, opposite):
    scale = max(abs(previous[1]), abs(best[1]), abs(opposite[1]))
    a = previous[1] / scale
    b = best[1] / scale
    c = opposite[1] / scale
    if previous[0] == opposite[0] or a == c:
        ratio = best[1] / previous[1]
        cand = best[0] + (best[0] - previous[0]) * ratio / (1.0 - ratio)
    elif a == b or b == c:
        return None
    else:
        wa = (b / (a - b)) * (c / (a - c))
        wc = (a / (c - a)) * (b / (c - b))
        cand = best[0] + ((previous[0] - best[0]) * wa + (opposite[0] - best[0]) * wc)
    if not math.isfinite(cand):
        return None
    return f64_zero(cand)


def f64_acceptable_trial(point, from_x, to_x, lower, upper, historical_step):
    if not math.isfinite(point) or point <= lower or point >= upper:
        return False
    limit = f64_midpoint(f64_midpoint(from_x, to_x), to_x)
    before_limit = (point > from_x and point < limit) if to_x > from_x else (point < from_x and point > limit)
    return before_limit and abs(point - from_x) < historical_step / 2.0


def f64_tolerance_met(lower, upper, abs_tol, rel_tol):
    width = upper - lower
    magnitude = max(abs(lower), abs(upper))
    if math.isfinite(width):
        return width <= abs_tol + rel_tol * magnitude
    return upper / 2.0 - lower / 2.0 <= abs_tol / 2.0 + rel_tol * (magnitude / 2.0)


def brent_oracle(comp, flows, dirty_target_bits, scale_exp, lo_bits, hi_bits, abs_tol_bits, rel_tol_bits, max_evals, prec):
    """Independent Brent root-finder matching M6 SolverOptions + M8 fixed-scale residual."""
    target_f = binary(dirty_target_bits)
    scale_f = pow2(-scale_exp)

    def eval_point(x_bits):
        d_bits = eval_dirty_at(comp, x_bits, flows, prec)
        if d_bits is None:
            return None
        d_f = binary(d_bits)
        diff = d_f - target_f
        res_st, res_bits = rn_status(diff * scale_f)
        assert res_st == 'ok'
        amt_st, amt_bits = rn_status(diff)
        return d_bits, canonical_zero_bits(res_bits), (amt_st, canonical_zero_bits(amt_bits))

    if max_evals == 0:
        return {'expected': 'err', 'kind': 'bond_yield.solver', 'phase': '-', 'stage': '-', 'evals': '0'}

    out_a = eval_point(lo_bits)
    if out_a is None:
        return {'expected': 'err', 'kind': 'bond_yield.callback', 'phase': '-', 'stage': 'lower_endpoint', 'evals': '1'}
    da_bits, fa_bits, amt_a = out_a
    if binary(fa_bits) == 0:
        return {
            'expected': 'ok',
            'evals': 1,
            'term': 'ExactZero',
            'root_bits': canonical_zero_bits(lo_bits),
            'dirty_bits': da_bits,
            'res_bits': 0,
            'amt_res': amt_a,
        }
    if max_evals == 1:
        return {'expected': 'err', 'kind': 'bond_yield.solver', 'phase': '-', 'stage': '-', 'evals': '1'}

    out_b = eval_point(hi_bits)
    if out_b is None:
        return {'expected': 'err', 'kind': 'bond_yield.callback', 'phase': '-', 'stage': 'upper_endpoint', 'evals': '2'}
    db_bits, fb_bits, amt_b = out_b
    if binary(fb_bits) == 0:
        return {
            'expected': 'ok',
            'evals': 2,
            'term': 'ExactZero',
            'root_bits': canonical_zero_bits(hi_bits),
            'dirty_bits': db_bits,
            'res_bits': 0,
            'amt_res': amt_b,
        }

    fa = f64_zero(bits_to_float(fa_bits))
    fb = f64_zero(bits_to_float(fb_bits))
    if f64_same_sign(fa, fb):
        return {'expected': 'err', 'kind': 'bond_yield.solver', 'phase': '-', 'stage': '-', 'evals': '2'}

    lower = f64_zero(bits_to_float(lo_bits))
    upper = f64_zero(bits_to_float(hi_bits))
    lower_val = fa
    upper_val = fb
    lower_pay = (da_bits, fa_bits, amt_a)
    upper_pay = (db_bits, fb_bits, amt_b)

    def best_entry():
        if abs(lower_val) <= abs(upper_val):
            return (lower, lower_val, lower_pay)
        return (upper, upper_val, upper_pay)

    def opposite_entry():
        if abs(lower_val) <= abs(upper_val):
            return (upper, upper_val, upper_pay)
        return (lower, lower_val, lower_pay)

    previous_best = opposite_entry()
    previous_step = upper - lower
    historical_step = previous_step
    abs_tol = bits_to_float(abs_tol_bits)
    rel_tol = bits_to_float(rel_tol_bits)
    evals = 2

    while True:
        if f64_tolerance_met(lower, upper, abs_tol, rel_tol):
            bx, _, bpay = best_entry()
            return {
                'expected': 'ok',
                'evals': evals,
                'term': 'StepTolerance',
                'root_bits': canonical_zero_bits(input_bits(bx)),
                'dirty_bits': bpay[0],
                'res_bits': bpay[1],
                'amt_res': bpay[2],
            }
        mid = f64_zero(f64_midpoint(lower, upper))
        if not (mid > lower and mid < upper):
            return {'expected': 'err', 'kind': 'bond_yield.solver', 'phase': '-', 'stage': '-', 'evals': str(evals)}

        b_entry = best_entry()
        o_entry = opposite_entry()
        best = (b_entry[0], b_entry[1])
        opposite = (o_entry[0], o_entry[1])
        previous = (previous_best[0], previous_best[1])
        trial = None
        if abs(best[1]) < abs(previous[1]):
            cand = f64_inverse_trial(previous, best, opposite)
            if cand is not None and f64_acceptable_trial(cand, best[0], opposite[0], lower, upper, historical_step):
                trial = cand
        if trial is not None:
            historical_step = previous_step
            previous_step = abs(trial - best[0])
            point = trial
        else:
            previous_step = abs(mid - best[0])
            historical_step = previous_step
            point = mid

        if evals >= max_evals:
            return {'expected': 'err', 'kind': 'bond_yield.solver', 'phase': '-', 'stage': '-', 'evals': str(evals)}
        evals += 1
        point_bits = canonical_zero_bits(input_bits(point))
        out_trial = eval_point(point_bits)
        if out_trial is None:
            return {'expected': 'err', 'kind': 'bond_yield.callback', 'phase': '-', 'stage': 'trial_point', 'evals': str(evals)}
        t_dirty_bits, t_res_bits, t_amt_res = out_trial
        t_val = f64_zero(bits_to_float(t_res_bits))
        t_pay = (t_dirty_bits, t_res_bits, t_amt_res)
        if t_val == 0.0:
            return {
                'expected': 'ok',
                'evals': evals,
                'term': 'ExactZero',
                'root_bits': point_bits,
                'dirty_bits': t_dirty_bits,
                'res_bits': 0,
                'amt_res': t_amt_res,
            }
        previous_best = b_entry
        if f64_same_sign(t_val, lower_val):
            lower = f64_zero(point)
            lower_val = t_val
            lower_pay = t_pay
        else:
            upper = f64_zero(point)
            upper_val = t_val
            upper_pay = t_pay


def ideal_root_bits_for_flows(comp, flows, dirty_target_bits, lo_bits, hi_bits, prec):
    """High-precision bisection/secant reference root in Decimal(prec)."""
    with localcontext() as ctx:
        ctx.prec = prec
        target_d = dec_from_frac(binary(dirty_target_bits))

        def f(y_d):
            total = -target_d
            for amt_bits, t_bits in flows:
                c_d = dec_from_frac(binary(amt_bits))
                t_f = binary(t_bits)
                if t_f == 0:
                    total += c_d
                    continue
                t_d = dec_from_frac(t_f)
                br, freq = active_branch(comp, t_f)
                if br == 'Simple':
                    total += c_d / (Decimal(1) + y_d * t_d)
                elif br == 'Continuous':
                    total += c_d * dec_exp(-y_d * t_d)
                else:
                    m_d = Decimal(freq)
                    total += c_d * dec_exp((-m_d * t_d) * dec_ln(Decimal(1) + y_d / m_d))
            return total

        a = dec_from_frac(binary(lo_bits))
        b = dec_from_frac(binary(hi_bits))
        fa = f(a)
        fb = f(b)
        if fa == 0:
            return canonical_zero_bits(lo_bits)
        if fb == 0:
            return canonical_zero_bits(hi_bits)
        for _ in range(64):
            mid = (a + b) / Decimal(2)
            fm = f(mid)
            if fm == 0:
                a = b = mid
                break
            if (fa > 0 and fm > 0) or (fa < 0 and fm < 0):
                a, fa = mid, fm
            else:
                b, fb = mid, fm
        x0, f0 = a, fa
        x1, f1 = b, fb
        for _ in range(8):
            if f1 == f0 or f1 == 0:
                break
            x2 = x1 - f1 * (x1 - x0) / (f1 - f0)
            if not (min(a, b) <= x2 <= max(a, b)):
                x2 = (a + b) / Decimal(2)
            f2 = f(x2)
            if (fa > 0 and f2 > 0) or (fa < 0 and f2 < 0):
                a, fa = x2, f2
            else:
                b, fb = x2, f2
            x0, f0 = x1, f1
            x1, f1 = x2, f2
        _, rb = rn_status(dec_to_frac(x1))
        return canonical_zero_bits(rb)


def evaluate_yield_oracle(row, flows, prec):
    face_bits = int(row['face_bits'], 16)
    accrued_bits = int(row['accrued_bits'], 16)
    target_kind = row['target_kind']
    target_mode = row['target_mode']
    target_val_bits = int(row['target_value_bits'], 16)
    lo_bits = int(row['bracket_lo_bits'], 16)
    hi_bits = int(row['bracket_hi_bits'], 16)
    abs_tol_bits = int(row['abs_step_tol_bits'], 16)
    rel_tol_bits = int(row['rel_step_tol_bits'], 16)
    max_evals = int(row['max_evals'])
    comp = row['comp']

    face_f = binary(face_bits)
    accrued_f = binary(accrued_bits)
    tval_f = binary(target_val_bits)
    if target_mode == 'Amount':
        p100_st, p100_bits = rn_status((F(100) * tval_f) / face_f)
        assert p100_st == 'ok'
        p100_f = binary(p100_bits)
    else:
        p100_f = tval_f
    conv_st, conv_bits = rn_status((p100_f * face_f) / F(100))
    if conv_st != 'ok':
        return {'expected': 'err', 'kind': 'bond_yield.target', 'phase': 'target conversion', 'stage': '-', 'evals': '-'}
    conv_bits = canonical_zero_bits(conv_bits)

    conv_f = binary(conv_bits)
    if target_kind == 'Dirty':
        dirty_target_bits = conv_bits
    else:
        dt_st, dt_bits = rn_status(conv_f + accrued_f)
        if dt_st == 'overflow':
            return {'expected': 'err', 'kind': 'bond_yield.target', 'phase': 'target clean shift', 'stage': '-', 'evals': '-'}
        dirty_target_bits = canonical_zero_bits(dt_bits)

    dirty_target_f = binary(dirty_target_bits)
    scale_exp = 1 if abs(dirty_target_f) >= binary(TWO_POW_970_BITS) else 0
    sens = classify_sensitivity(comp, flows)

    if sens == 'Constant':
        d_bits = eval_dirty_at(comp, lo_bits, flows, prec)
        if d_bits is None:
            return {'expected': 'err', 'kind': 'bond_yield.callback', 'phase': '-', 'stage': 'lower_endpoint', 'evals': '1'}
        const_f = sum((binary(ab) for ab, tb in flows if binary(tb) == 0), F(0))
        kind = 'bond_yield.no_yield_sensitivity' if const_f == dirty_target_f else 'bond_yield.constant_price_mismatch'
        return {'expected': 'err', 'kind': kind, 'phase': '-', 'stage': '-', 'evals': '1'}

    res = brent_oracle(
        comp,
        flows,
        dirty_target_bits,
        scale_exp,
        lo_bits,
        hi_bits,
        abs_tol_bits,
        rel_tol_bits,
        max_evals,
        prec,
    )
    if res['expected'] != 'ok':
        return res

    root_bits = res['root_bits']
    dirty_bits = res['dirty_bits']
    amt_res_st, _ = res['amt_res']
    ideal_rb = ideal_root_bits_for_flows(comp, flows, dirty_target_bits, lo_bits, hi_bits, prec)

    ulp_pad = 0 if all(active_branch(comp, binary(tb))[0] == 'Simple' for _, tb in flows if binary(tb) > 0) else 2
    dirty_lo = step_bits(dirty_bits, -ulp_pad)
    dirty_hi = step_bits(dirty_bits, ulp_pad)
    scale_f = pow2(-scale_exp)
    _, clean_lo = rn_status(binary(dirty_lo) - accrued_f)
    _, clean_hi = rn_status(binary(dirty_hi) - accrued_f)
    _, solve_res_lo = rn_status((binary(dirty_lo) - dirty_target_f) * scale_f)
    _, solve_res_hi = rn_status((binary(dirty_hi) - dirty_target_f) * scale_f)
    _, amt_res_lo = rn_status(binary(dirty_lo) - dirty_target_f)
    _, amt_res_hi = rn_status(binary(dirty_hi) - dirty_target_f)

    return {
        'expected': 'ok',
        'kind': '-',
        'phase': '-',
        'stage': '-',
        'evals': str(res['evals']),
        'sensitivity': sens,
        'scale_exp': str(scale_exp),
        'term': res['term'],
        'conv_bits': conv_bits,
        'dirty_target_bits': dirty_target_bits,
        'root_lo': step_bits(root_bits, -ulp_pad),
        'root_hi': step_bits(root_bits, ulp_pad),
        'dirty_lo': dirty_lo,
        'dirty_hi': dirty_hi,
        'clean_lo': canonical_zero_bits(clean_lo),
        'clean_hi': canonical_zero_bits(clean_hi),
        'solve_res_lo': canonical_zero_bits(solve_res_lo),
        'solve_res_hi': canonical_zero_bits(solve_res_hi),
        'amt_res_lo': canonical_zero_bits(amt_res_lo) if amt_res_st == 'ok' else 0,
        'amt_res_hi': canonical_zero_bits(amt_res_hi) if amt_res_st == 'ok' else 0,
        'ideal_root_bits': ideal_rb,
    }


def format_flows(flows):
    if not flows:
        return '-'
    return ';'.join(f'{d}:{hx(ab)}:{hx(tb)}' for d, ab, tb in flows)


def parse_flows(s):
    if s == '-':
        return []
    out = []
    for item in s.split(';'):
        _, ab, tb = item.split(':')
        out.append((int(ab, 16), int(tb, 16)))
    return out


def make_rows():
    rows = []
    tight_abs = hx(input_bits(1e-14))
    tight_rel = hx(input_bits(1e-14))

    def add_row(**kw):
        r = dict.fromkeys(COLUMNS, '-')
        r.update(kw)
        rows.append(r)

    # Family 1: Closed-form single-flow inversion & risk across all 32 compounding configs (64 rows)
    for ci, comp in enumerate(COMPS):
        for variant in (0, 1):
            issue = '2024-01-01'
            settlement = '2024-01-01'
            maturity = '2024-07-01' if variant == 0 else '2025-07-01'
            dc = '30360'
            t_f = year_fraction(dc, settlement, maturity)
            t_bits = rn(t_f)
            face = 100.0 if variant == 0 else 1000.0
            face_bits = input_bits(face)
            accrued_bits = 0
            amt = 105.0 if variant == 0 else 1050.0
            amt_bits = input_bits(amt)
            y_true = 0.0425 if (ci + variant) % 2 == 0 else -0.0175
            y_bits = input_bits(y_true)
            obs = staged_flow_observation(comp, y_bits, amt_bits, t_bits, 160)
            assert obs['ok']
            dirty_bits = obs['pv_bits']
            if variant == 0:
                target_kind = 'Dirty'
                target_mode = 'Per100Face'
                tval_bits = dirty_bits
            else:
                target_kind = 'Clean'
                target_mode = 'Amount'
                tval_bits = dirty_bits
            add_row(
                family='SINGLE-FLOW-32',
                profile=f'SINGLE-{comp}-{"SHORT-DIRTY" if variant == 0 else "LONG-CLEAN"}',
                issue=issue,
                settlement=settlement,
                inclusion='Exclude',
                daycount=dc,
                comp=comp,
                face_bits=hx(face_bits),
                accrued_bits=hx(accrued_bits),
                flows=format_flows([(maturity, amt_bits, t_bits)]),
                target_kind=target_kind,
                target_mode=target_mode,
                target_value_bits=hx(tval_bits),
                bracket_lo_bits=hx(input_bits(-0.15)),
                bracket_hi_bits=hx(input_bits(0.35)),
                abs_step_tol_bits=tight_abs,
                rel_step_tol_bits=tight_rel,
                max_evals='200',
                risk_yield_bits=hx(y_bits),
            )

    # Family 2: Multi-flow coupon bond inversion & risk across all 32 compounding configs (64 rows)
    dc_cycle = (
        '30360',
        'A365F',
        '30E360ISDA:2026-01-01',
        'ICMA:2:2024-01-01,2024-07-01,2025-01-01,2025-07-01,2026-01-01',
    )
    pay_dates = ('2024-07-01', '2025-01-01', '2025-07-01', '2026-01-01')
    for ci, comp in enumerate(COMPS):
        for variant in (0, 1):
            dc = dc_cycle[(ci + variant) % len(dc_cycle)]
            issue = '2024-01-01'
            settlement = '2024-04-01' if variant == 0 else '2024-01-01'
            face_bits = input_bits(100.0)
            coupon_rate_bits = input_bits(0.05 if variant == 0 else 0.035)
            face_f = binary(face_bits)
            rate_f = binary(coupon_rate_bits)
            accrued_yf_bits = rn(year_fraction(dc, '2024-01-01', settlement))
            accrued_bits = rn(face_f * rate_f * binary(accrued_yf_bits))
            fl = []
            prev = '2024-01-01'
            for pd in pay_dates:
                yf_bits = rn(year_fraction(dc, prev, pd))
                cpn_bits = rn(face_f * rate_f * binary(yf_bits))
                tf_bits = rn(year_fraction(dc, settlement, pd))
                fl.append((pd, cpn_bits, tf_bits))
                prev = pd
            fl.append(('2026-01-01', face_bits, rn(year_fraction(dc, settlement, '2026-01-01'))))
            y_true = 0.048 if variant == 0 else -0.012
            y_bits = input_bits(y_true)
            dirty_at_y = eval_dirty_at(comp, y_bits, [(ab, tb) for _, ab, tb in fl], 160)
            assert dirty_at_y is not None
            if variant == 0:
                tk, tm = 'Clean', 'Per100Face'
                tv_bits = rn(binary(dirty_at_y) - binary(accrued_bits))
            else:
                tk, tm = 'Dirty', 'Per100Face'
                tv_bits = dirty_at_y
            add_row(
                family='COUPON-32',
                profile=f'COUPON-{comp}-{"MID-CLEAN" if variant == 0 else "ISSUE-DIRTY"}',
                issue=issue,
                settlement=settlement,
                inclusion='Exclude',
                daycount=dc,
                comp=comp,
                face_bits=hx(face_bits),
                accrued_bits=hx(accrued_bits),
                flows=format_flows(fl),
                target_kind=tk,
                target_mode=tm,
                target_value_bits=hx(tv_bits),
                bracket_lo_bits=hx(input_bits(-0.15)),
                bracket_hi_bits=hx(input_bits(0.35)),
                abs_step_tol_bits=tight_abs,
                rel_step_tol_bits=tight_rel,
                max_evals='200',
                risk_yield_bits=hx(y_bits),
            )

    # Family 3: Signed non-monotone two-flow bond P(y) = 100*(v^2 - v) (FR-YR-07)
    signed_flows = [
        ('2025-01-01', input_bits(-100.0), input_bits(1.0)),
        ('2026-01-01', input_bits(100.0), input_bits(2.0)),
    ]
    # Root 1/3 on [0.0, 0.8], risk at y = 0.0 (zero dirty price!)
    add_row(
        family='SIGNED-NONMONOTONE',
        profile='TWO-FLOW-LOWER-ROOT-AND-ZERO-DIRTY-RISK',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Compounded:1',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows(signed_flows),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(-18.75)),
        bracket_lo_bits=hx(input_bits(0.0)),
        bracket_hi_bits=hx(input_bits(0.8)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='200',
        risk_yield_bits=hx(input_bits(0.0)),
    )
    # Root 3.0 on [1.5, 4.0], risk at y = 1.0 (dirty = -25.0, P' = 0.0)
    add_row(
        family='SIGNED-NONMONOTONE',
        profile='TWO-FLOW-UPPER-ROOT-AND-STATIONARY-RISK',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Compounded:1',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows(signed_flows),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(-18.75)),
        bracket_lo_bits=hx(input_bits(1.5)),
        bracket_hi_bits=hx(input_bits(4.0)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='200',
        risk_yield_bits=hx(input_bits(1.0)),
    )
    # Tangent target -25.0 on [0.0, 2.0] -> InvalidBracket
    add_row(
        family='SIGNED-NONMONOTONE',
        profile='TWO-FLOW-TANGENT-REJECTED',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Compounded:1',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows(signed_flows),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(-25.0)),
        bracket_lo_bits=hx(input_bits(0.0)),
        bracket_hi_bits=hx(input_bits(2.0)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='200',
        risk_yield_bits=hx(input_bits(3.0)),
    )

    # Family 4: Constant-plan certification (FR-YR-06)
    # Empty post-maturity plan matching target 0.0 -> NoYieldSensitivity
    add_row(
        family='CONSTANT-PLAN',
        profile='POST-MATURITY-NO-SENSITIVITY',
        issue='2024-01-01',
        settlement='2025-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Compounded:1',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows='-',
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(0),
        bracket_lo_bits=hx(input_bits(-0.05)),
        bracket_hi_bits=hx(input_bits(0.20)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='100',
        risk_yield_bits=hx(input_bits(0.05)),
    )
    # Time-0 included payment 105.0 matching target 105.0 -> NoYieldSensitivity
    add_row(
        family='CONSTANT-PLAN',
        profile='TIME-ZERO-INCLUDED-NO-SENSITIVITY',
        issue='2024-01-01',
        settlement='2025-01-01',
        inclusion='Include',
        daycount='30360',
        comp='Compounded:1',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows([('2025-01-01', input_bits(105.0), 0)]),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(105.0)),
        bracket_lo_bits=hx(input_bits(-0.05)),
        bracket_hi_bits=hx(input_bits(0.20)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='100',
        risk_yield_bits=hx(input_bits(0.05)),
    )
    # Opposite same-factor pair (-100 coupon, +100 redemption at t = 1.0) with target 10.0 -> ConstantPriceMismatch
    add_row(
        family='CONSTANT-PLAN',
        profile='PAIred-OPPOSITE-CONSTANT-MISMATCH',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Compounded:2',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows([
            ('2025-01-01', input_bits(-100.0), input_bits(1.0)),
            ('2025-01-01', input_bits(100.0), input_bits(1.0)),
        ]),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(10.0)),
        bracket_lo_bits=hx(input_bits(-0.05)),
        bracket_hi_bits=hx(input_bits(0.20)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='100',
        risk_yield_bits=hx(input_bits(0.05)),
    )

    # Family 5: Fixed-scale residual threshold 2^970 and extreme-scale risk rescue (FR-YR-03, FR-YR-10)
    # Below 2^970 (s = 0) and at 2^970 (s = 1), plus MAX/t=2 overflow rescue
    add_row(
        family='EXTREME-SCALE',
        profile='RESIDUAL-BELOW-2POW970-AND-MAX-T2-RISK-RESCUE',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Continuous',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows([('2026-01-01', TWO_POW_970_BITS, input_bits(2.0))]),
        target_kind='Dirty',
        target_mode='Amount',
        target_value_bits=hx(TWO_POW_970_BITS - 1),
        bracket_lo_bits=hx(input_bits(0.0)),
        bracket_hi_bits=hx(input_bits(0.20)),
        abs_step_tol_bits=hx(input_bits(1e-6)),
        rel_step_tol_bits=hx(input_bits(1e-6)),
        max_evals='100',
        risk_yield_bits=hx(input_bits(0.0)),
    )
    add_row(
        family='EXTREME-SCALE',
        profile='RESIDUAL-AT-2POW970-AND-MAX-OVERFLOW-RESCUE',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Continuous',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows([('2026-01-01', 0x7fefffffffffffff, input_bits(2.0))]),
        target_kind='Dirty',
        target_mode='Amount',
        target_value_bits=hx(TWO_POW_970_BITS),
        bracket_lo_bits=hx(input_bits(0.0)),
        bracket_hi_bits=hx(input_bits(200.0)),
        abs_step_tol_bits=hx(input_bits(1e-6)),
        rel_step_tol_bits=hx(input_bits(1e-6)),
        max_evals='100',
        risk_yield_bits=hx(input_bits(0.0)),
    )
    add_row(
        family='EXTREME-SCALE',
        profile='MIN-SUBNORMAL-T05-UNDERFLOW-RESCUE',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Continuous',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows([('2024-07-01', 1, input_bits(0.5))]),
        target_kind='Dirty',
        target_mode='Amount',
        target_value_bits=hx(1),
        bracket_lo_bits=hx(input_bits(0.0)),
        bracket_hi_bits=hx(input_bits(0.20)),
        abs_step_tol_bits=hx(input_bits(1e-6)),
        rel_step_tol_bits=hx(input_bits(1e-6)),
        max_evals='100',
        risk_yield_bits=hx(input_bits(0.0)),
    )

    # Family 6: Error taxonomy (FR-YR-12, FR-YR-14)
    # Target conversion overflow, Target clean shift overflow, LowerEndpoint callback, Second-flow risk domain error
    add_row(
        family='ERROR-TAXONOMY',
        profile='TARGET-CONVERSION-OVERFLOW-AND-QUOTE-DOMAIN-ERR',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Compounded:1',
        face_bits=hx(0x7fefffffffffffff),
        accrued_bits=hx(0),
        flows=format_flows([('2025-01-01', input_bits(100.0), input_bits(1.0))]),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(200.0)),
        bracket_lo_bits=hx(input_bits(0.0)),
        bracket_hi_bits=hx(input_bits(0.10)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='100',
        risk_yield_bits=hx(input_bits(-1.0)),
    )
    add_row(
        family='ERROR-TAXONOMY',
        profile='TARGET-CLEAN-SHIFT-OVERFLOW-AND-FLOW-DOMAIN-ERR',
        issue='2024-01-01',
        settlement='2024-07-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Simple',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(TWO_POW_970_BITS),
        flows=format_flows([
            ('2025-01-01', 0x7ca0000000000000, input_bits(0.5)),
            ('2025-01-01', input_bits(100.0), input_bits(0.5)),
        ]),
        target_kind='Clean',
        target_mode='Amount',
        target_value_bits=hx(0x7fefffffffffffff),
        bracket_lo_bits=hx(input_bits(0.0)),
        bracket_hi_bits=hx(input_bits(0.10)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='100',
        risk_yield_bits=hx(input_bits(-2.0)),
    )
    add_row(
        family='ERROR-TAXONOMY',
        profile='CALLBACK-LOWER-ENDPOINT-ERROR',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Compounded:1',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows([('2025-01-01', input_bits(100.0), input_bits(1.0))]),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(95.0)),
        bracket_lo_bits=hx(input_bits(-1.0)),
        bracket_hi_bits=hx(input_bits(0.20)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='100',
        risk_yield_bits=hx(input_bits(0.05)),
    )
    add_row(
        family='ERROR-TAXONOMY',
        profile='CALLBACK-TRIAL-POINT-ERROR',
        issue='2024-01-01',
        settlement='2024-01-01',
        inclusion='Exclude',
        daycount='30360',
        comp='Simple',
        face_bits=hx(input_bits(100.0)),
        accrued_bits=hx(0),
        flows=format_flows([
            ('2025-01-01', input_bits(-1.0), input_bits(1.0)),
            ('2034-01-01', input_bits(100.0), input_bits(10.0)),
        ]),
        target_kind='Dirty',
        target_mode='Per100Face',
        target_value_bits=hx(input_bits(50.0)),
        bracket_lo_bits=hx(input_bits(-0.05)),
        bracket_hi_bits=hx(input_bits(2.0)),
        abs_step_tol_bits=tight_abs,
        rel_step_tol_bits=tight_rel,
        max_evals='100',
        risk_yield_bits=hx(input_bits(-0.10)),
    )

    for i, r in enumerate(rows, 1):
        r['id'] = f'BYR-{i:04d}'
    return rows


def evaluate_row(row, prec):
    flows = parse_flows(row['flows'])
    y_out = evaluate_yield_oracle(row, flows, prec)
    r_out = evaluate_risk_oracle(row['comp'], int(row['risk_yield_bits'], 16), flows, prec)

    out = {}
    if y_out['expected'] == 'ok':
        out.update(
            yield_expected='ok',
            yield_error_kind='-',
            yield_error_phase='-',
            yield_error_stage='-',
            yield_error_evals='-',
            sensitivity=y_out['sensitivity'],
            residual_scale_exp=y_out['scale_exp'],
            eval_count=y_out['evals'],
            termination=y_out['term'],
            converted_target_bits=hx(y_out['conv_bits']),
            dirty_target_bits=hx(y_out['dirty_target_bits']),
            solved_yield_lo_bits=hx(y_out['root_lo']),
            solved_yield_hi_bits=hx(y_out['root_hi']),
            solved_dirty_lo_bits=hx(y_out['dirty_lo']),
            solved_dirty_hi_bits=hx(y_out['dirty_hi']),
            solved_clean_lo_bits=hx(y_out['clean_lo']),
            solved_clean_hi_bits=hx(y_out['clean_hi']),
            solve_residual_lo_bits=hx(y_out['solve_res_lo']),
            solve_residual_hi_bits=hx(y_out['solve_res_hi']),
            amount_residual_lo_bits=hx(y_out['amt_res_lo']),
            amount_residual_hi_bits=hx(y_out['amt_res_hi']),
            ideal_root_bits=hx(y_out['ideal_root_bits']),
        )
    else:
        out.update(
            yield_expected='err',
            yield_error_kind=y_out['kind'],
            yield_error_phase=y_out['phase'],
            yield_error_stage=y_out['stage'],
            yield_error_evals=y_out['evals'],
            sensitivity='-',
            residual_scale_exp='-',
            eval_count='-',
            termination='-',
            converted_target_bits='-',
            dirty_target_bits='-',
            solved_yield_lo_bits='-',
            solved_yield_hi_bits='-',
            solved_dirty_lo_bits='-',
            solved_dirty_hi_bits='-',
            solved_clean_lo_bits='-',
            solved_clean_hi_bits='-',
            solve_residual_lo_bits='-',
            solve_residual_hi_bits='-',
            amount_residual_lo_bits='-',
            amount_residual_hi_bits='-',
            ideal_root_bits='-',
        )

    if r_out['expected'] == 'ok':
        def set_measure(prefix, tup):
            st, lo, hi = tup
            out[f'{prefix}_status'] = st
            out[f'{prefix}_lo_bits'] = hx(lo) if st == 'ok' else '-'
            out[f'{prefix}_hi_bits'] = hx(hi) if st == 'ok' else '-'

        out.update(
            risk_expected='ok',
            risk_error_kind='-',
            risk_error_phase='-',
            risk_error_index='-',
            risk_dirty_lo_bits=hx(r_out['dirty_lo']),
            risk_dirty_hi_bits=hx(r_out['dirty_hi']),
            ideal_p1_bits=hx(r_out['ideal_p1_bits']),
            ideal_p2_bits=hx(r_out['ideal_p2_bits']),
            ideal_moment_bits=hx(r_out['ideal_moment_bits']),
            ideal_mod_dur_bits=hx(r_out['ideal_mod_dur_bits']),
            ideal_mac_dur_bits=hx(r_out['ideal_mac_dur_bits']),
            ideal_conv_bits=hx(r_out['ideal_conv_bits']),
        )
        set_measure('p1', r_out['p1'])
        set_measure('p2', r_out['p2'])
        set_measure('moment', r_out['moment'])
        set_measure('mod_dur', r_out['mod_dur'])
        set_measure('mac_dur', r_out['mac_dur'])
        set_measure('conv', r_out['conv'])
    else:
        out.update(
            risk_expected='err',
            risk_error_kind=r_out['error_kind'],
            risk_error_phase=r_out['error_phase'],
            risk_error_index=r_out['error_index'],
            risk_dirty_lo_bits='-',
            risk_dirty_hi_bits='-',
            p1_status='-',
            p1_lo_bits='-',
            p1_hi_bits='-',
            p2_status='-',
            p2_lo_bits='-',
            p2_hi_bits='-',
            moment_status='-',
            moment_lo_bits='-',
            moment_hi_bits='-',
            mod_dur_status='-',
            mod_dur_lo_bits='-',
            mod_dur_hi_bits='-',
            mac_dur_status='-',
            mac_dur_lo_bits='-',
            mac_dur_hi_bits='-',
            conv_status='-',
            conv_lo_bits='-',
            conv_hi_bits='-',
            ideal_p1_bits='-',
            ideal_p2_bits='-',
            ideal_moment_bits='-',
            ideal_mod_dur_bits='-',
            ideal_mac_dur_bits='-',
            ideal_conv_bits='-',
        )
    return tuple(out[k] for k in COLUMNS[20:])


def generate():
    rows = make_rows()
    records = []
    family_counts = collections.Counter()
    yield_counts = collections.Counter()
    risk_counts = collections.Counter()
    for row in rows:
        res160 = evaluate_row(row, PRECISIONS[0])
        res240 = evaluate_row(row, PRECISIONS[1])
        assert res160 == res240, (row['id'], res160, res240)
        for k, v in zip(COLUMNS[20:], res240):
            row[k] = v
        family_counts[row['family']] += 1
        yield_counts[row['yield_expected']] += 1
        risk_counts[row['risk_expected']] += 1
        records.append('\t'.join(row[k] for k in COLUMNS) + '\n')

    assert len(rows) == 141, len(rows)
    body = '\t'.join(COLUMNS) + '\n' + ''.join(records)
    requirements = '\n'.join(
        line
        for line in (ROOT / 'docs/REQUIREMENTS.md').read_text().splitlines()
        if line.startswith('| FR-YR-')
    ) + '\n'
    source = Path(__file__).read_bytes()
    header = [
        ('schema', '1'),
        ('fixture-set', 'M8-BOND-YIELD-RISK-ORACLE'),
        ('domain', 'bonds'),
        ('description', 'Independent M8 Phase D generic bond yield-to-maturity inversion and analytical risk oracle corpus across all 32 compounding configurations'),
        ('reference', 'oracle:Fraction-exact-wide+Decimal160-240-Brent-and-analytic-derivatives'),
        ('reference-version', 'Python>=3.9 stdlib'),
        ('reference-source', 'Adopted M8_BOND_YIELD_RISK_CONTRACT.md, M6_SOLVER_CONTRACT.md, M4_RATES_CONTRACT.md; no Rust or QuantLib output'),
        ('tolerance', 'Exact 0-ULP on Simple/discrete stages; <=2-ULP staged binary64 enclosures where libm exp/ln/pow enters; dual-precision 160/240 ideal references'),
        ('reproduction', 'python3 -I validation/generator/bond_yield_risk_oracles.py --check'),
        ('generated-at', '2026-10-17T07:30:00Z'),
        ('platform', 'platform-independent stdlib integer/Fraction/Decimal'),
        ('generator-sha256', sha(source)),
        ('generator-fnv64', fnv(source)),
        ('requirements-rows-sha256', sha(requirements.encode())),
        ('requirements-rows-fnv64', fnv(requirements.encode())),
        ('record-count', str(len(rows))),
        ('family-counts', ';'.join(f'{k}={v}' for k, v in sorted(family_counts.items()))),
        ('yield-expectation-counts', ';'.join(f'{k}={v}' for k, v in sorted(yield_counts.items()))),
        ('risk-expectation-counts', ';'.join(f'{k}={v}' for k, v in sorted(risk_counts.items()))),
        ('precision-agreement-count', str(len(rows))),
        ('body-sha256', sha(body.encode())),
        ('body-fnv64', fnv(body.encode())),
    ]
    for file in CONTRACTS:
        b = (ROOT / file).read_bytes()
        name = Path(file).stem.lower().replace('_', '-')
        header += [(name + '-sha256', sha(b)), (name + '-fnv64', fnv(b))]
    text = ''.join(f'{k}: {v}\n' for k, v in header) + '\n' + body
    return text.encode(), {
        'records': len(rows),
        'families': dict(family_counts),
        'yield': dict(yield_counts),
        'risk': dict(risk_counts),
    }


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--check', action='store_true')
    args = p.parse_args()
    result, summary = generate()
    if args.check:
        assert DEST.read_bytes() == result, 'fixture differs'
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(result)
    print(json.dumps(summary, sort_keys=True))
    print(f'bytes={len(result)} sha256={sha(result)} check={args.check}')


if __name__ == '__main__':
    main()

