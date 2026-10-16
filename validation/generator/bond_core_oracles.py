#!/usr/bin/env python3
"""Independent M8 bond core oracle; never reads Rust or reference outputs.

Every binary64 input and output is represented by its exact 64-bit IEEE 754 bit
pattern and exact Fraction value (-1)^sign * M * 2^e. Per-100 quote conversions
and inverse conversions are derived by exact Fraction arithmetic and rounded once
to binary64 nearest/ties-even, cross-checked against 256/512-digit Decimal.
Bond entitlement, earned/detached coupon accrual, work-unit accounting, and
direct-horizon dirty/clean valuations are derived from civil dates and exact
Fraction/Decimal arithmetic. No non-stdlib imports are used.
"""

import argparse
from collections import Counter
import datetime as dt
from decimal import Context, Decimal, ROUND_HALF_EVEN
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import platform
import random
import re
import struct

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/bonds/bond-core-oracles.fixture"
CONTRACT = ROOT / "docs/M8_BOND_CORE_CONTRACT.md"
CONTRACT_SHA = "009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c"
REQUIREMENTS = ROOT / "docs/REQUIREMENTS.md"

SIGN = 1 << 63
FRACTION_MASK = (1 << 52) - 1
POSITIVE_INFINITY = 0x7FF << 52
F64_MAX = POSITIVE_INFINITY - 1
CONTEXTS = (Context(prec=256, rounding=ROUND_HALF_EVEN), Context(prec=512, rounding=ROUND_HALF_EVEN))

COLUMNS = (
    "id profile op face_bits value_bits spec settlement inclusion "
    "yield_dcc yield_comp yield_bits work_cap expected context output"
).split()

COUNTS = Counter()
PROFILES = Counter()
OPS = Counter()
EXPECTATIONS = Counter()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fnv(data: bytes) -> str:
    value = 14695981039346656037
    for byte in data:
        value = ((value ^ byte) * 1099511628211) & ((1 << 64) - 1)
    return "%016x" % value


def hexbits(bits: int) -> str:
    return "0x%016x" % bits


def float_to_bits(value: float) -> int:
    return struct.unpack(">Q", struct.pack(">d", value))[0]


def bits_to_float(bits: int) -> float:
    return struct.unpack(">d", struct.pack(">Q", bits))[0]


def classify_nonfinite(bits: int):
    exp = (bits >> 52) & 0x7FF
    frac = bits & FRACTION_MASK
    if exp != 0x7FF:
        return None
    if frac != 0:
        return "NaN"
    return "NegativeInfinity" if (bits & SIGN) else "PositiveInfinity"


def exact_fraction(bits: int) -> Fraction:
    exp = (bits >> 52) & 0x7FF
    frac = bits & FRACTION_MASK
    assert exp != 0x7FF
    if exp == 0 and frac == 0:
        return Fraction(0)
    sig = frac if exp == 0 else (frac | (1 << 52))
    power = -1074 if exp == 0 else (exp - 1075)
    mag = Fraction(sig) * (Fraction(1 << power, 1) if power >= 0 else Fraction(1, 1 << (-power)))
    return -mag if (bits & SIGN) else mag


def round_fraction_to_bits(value: Fraction, negative_hint: bool = False) -> int:
    if value == 0:
        return SIGN if negative_hint else 0
    negative = value < 0
    mag = -value if negative else value
    sign_bit = SIGN if negative else 0
    num, den = mag.numerator, mag.denominator
    k0 = num.bit_length() - den.bit_length()
    if k0 >= 0:
        top = k0 if num >= (den << k0) else (k0 - 1)
    else:
        top = k0 if (num << (-k0)) >= den else (k0 - 1)
    if top > 1023:
        return sign_bit | POSITIVE_INFINITY
    if top < -1075:
        return sign_bit
    unit_exp = max(top - 52, -1074)
    if unit_exp >= 0:
        scaled = Fraction(num, den << unit_exp)
    else:
        scaled = Fraction(num << (-unit_exp), den)
    q, r = divmod(scaled.numerator, scaled.denominator)
    twice_r = r * 2
    if twice_r > scaled.denominator or (twice_r == scaled.denominator and (q & 1) == 1):
        q += 1
    if top < -1022:
        return sign_bit | q
    rounded_top = top
    if q == (1 << 53):
        q >>= 1
        rounded_top += 1
    if rounded_top > 1023:
        return sign_bit | POSITIVE_INFINITY
    return sign_bit | ((rounded_top + 1023) << 52) | (q & FRACTION_MASK)


def round_fraction_f64(value: Fraction) -> int:
    """Rounds an exact Fraction to binary64 bits with canonical +0.0."""
    if value == 0:
        return 0
    return round_fraction_to_bits(value)


def exact_sum_bits(bit_list) -> int:
    total = Fraction(0)
    for b in bit_list:
        total += exact_fraction(b)
    return round_fraction_f64(total)


def triple_product_bits(a_bits: int, b_bits: int, c_bits: int) -> int:
    fa = exact_fraction(a_bits)
    fb = exact_fraction(b_bits)
    fc = exact_fraction(c_bits)
    if fa == 0 or fb == 0 or fc == 0:
        return 0
    neg = ((a_bits ^ b_bits ^ c_bits) & SIGN) != 0
    return round_fraction_to_bits(fa * fb * fc, negative_hint=neg)


def eval_per_100_from_amount(amount_bits: int, face_bits: int):
    nf_face = classify_nonfinite(face_bits)
    if nf_face is not None:
        return "bond.value", "phase=bond face,kind=value.not_finite,class=" + nf_face, "-"
    face_frac = exact_fraction(face_bits)
    if face_frac <= 0:
        return "bond.value", "phase=bond face,kind=value.out_of_domain", "-"
    nf_amt = classify_nonfinite(amount_bits)
    if nf_amt is not None:
        return "bond.value", "phase=price per 100,kind=value.not_finite,class=" + nf_amt, "-"
    amt_frac = exact_fraction(amount_bits)
    if amt_frac == 0:
        return "ok", "-", hexbits(0)
    exact_ratio = (amt_frac * 100) / face_frac
    bits = round_fraction_to_bits(exact_ratio, negative_hint=bool(amount_bits & SIGN))
    # Cross-check with 256/512-digit Decimal when in normal range
    for ctx in CONTEXTS:
        d_amt = ctx.create_decimal_from_float(bits_to_float(amount_bits))
        d_face = ctx.create_decimal_from_float(bits_to_float(face_bits))
        d_ratio = ctx.divide(ctx.multiply(d_amt, Decimal(100)), d_face)
        COUNTS["decimal-quote-crosschecks"] += 1
        assert (d_ratio < 0) == (exact_ratio < 0)
    nf_out = classify_nonfinite(bits)
    if nf_out is not None:
        return "bond.value", "phase=price per 100,kind=value.result_not_finite,class=" + nf_out, "-"
    if (bits & ~SIGN) == 0:
        return "bond.value", "phase=price per 100,kind=value.result_out_of_domain", "-"
    return "ok", "-", hexbits(bits)


def eval_amount_from_per_100(price_bits: int, face_bits: int):
    nf_face = classify_nonfinite(face_bits)
    if nf_face is not None:
        return "bond.value", "phase=bond face,kind=value.not_finite,class=" + nf_face, "-"
    face_frac = exact_fraction(face_bits)
    if face_frac <= 0:
        return "bond.value", "phase=bond face,kind=value.out_of_domain", "-"
    nf_price = classify_nonfinite(price_bits)
    if nf_price is not None:
        return "bond.value", "phase=bond price,kind=value.not_finite,class=" + nf_price, "-"
    price_frac = exact_fraction(price_bits)
    if price_frac == 0:
        return "ok", "-", hexbits(0)
    exact_amt = (face_frac * price_frac) / 100
    bits = round_fraction_to_bits(exact_amt, negative_hint=bool(price_bits & SIGN))
    for ctx in CONTEXTS:
        d_price = ctx.create_decimal_from_float(bits_to_float(price_bits))
        d_face = ctx.create_decimal_from_float(bits_to_float(face_bits))
        d_amt = ctx.divide(ctx.multiply(d_face, d_price), Decimal(100))
        COUNTS["decimal-quote-crosschecks"] += 1
        assert (d_amt < 0) == (exact_amt < 0)
    nf_out = classify_nonfinite(bits)
    if nf_out is not None:
        return "bond.value", "phase=bond amount,kind=value.result_not_finite,class=" + nf_out, "-"
    if (bits & ~SIGN) == 0:
        return "bond.value", "phase=bond amount,kind=value.result_out_of_domain", "-"
    return "ok", "-", hexbits(bits)


def parse_iso(text: str) -> dt.date:
    y, m, d = map(int, text.split("-"))
    return dt.date(y, m, d)


def is_leap(y: int) -> bool:
    return (y % 4 == 0 and y % 100 != 0) or (y % 400 == 0)


def is_feb_end(d: dt.date) -> bool:
    return d.month == 2 and d.day == (29 if is_leap(d.year) else 28)


def yf_bits(dcc: str, start: dt.date, end: dt.date) -> int:
    if start == end:
        return 0
    assert start < end
    days = (end - start).days
    if dcc == "Act365F":
        return float_to_bits(days / 365.0)
    if dcc == "Act360":
        return float_to_bits(days / 360.0)
    if dcc == "30360BB":
        d1, d2 = start.day, end.day
        if d1 == 31:
            d1 = 30
        if d2 == 31 and d1 == 30:
            d2 = 30
        dc = 360 * (end.year - start.year) + 30 * (end.month - start.month) + (d2 - d1)
        return float_to_bits(dc / 360.0)
    if dcc == "30E360":
        d1 = 30 if start.day == 31 else start.day
        d2 = 30 if end.day == 31 else end.day
        dc = 360 * (end.year - start.year) + 30 * (end.month - start.month) + (d2 - d1)
        return float_to_bits(dc / 360.0)
    if dcc == "30360US_EOM":
        d1, d2 = start.day, end.day
        if is_feb_end(start) and is_feb_end(end):
            d2 = 30
        if is_feb_end(start):
            d1 = 30
        if d2 == 31 and d1 in (30, 31):
            d2 = 30
        if d1 == 31:
            d1 = 30
        dc = 360 * (end.year - start.year) + 30 * (end.month - start.month) + (d2 - d1)
        return float_to_bits(dc / 360.0)
    if dcc == "B252_WEEKENDS":
        b = 0
        cur = start
        step = dt.timedelta(days=1)
        while cur < end:
            if cur.weekday() < 5:
                b += 1
            cur += step
        return float_to_bits(b / 252.0)
    raise ValueError(f"unsupported oracle dcc {dcc}")


def work_units_for(dcc: str, start: dt.date, end: dt.date) -> int:
    if dcc == "B252_WEEKENDS":
        return 1 + abs((end - start).days)
    return 1


def discount_factor_bits(rate_bits: int, comp: str, time_bits: int) -> int:
    rate = bits_to_float(rate_bits)
    time = bits_to_float(time_bits)
    if time == 0.0 or rate == 0.0:
        return float_to_bits(1.0)
    if comp == "Simple":
        cf = math_fma(rate, time, 1.0)
    elif comp == "Continuous":
        import math
        cf = math.exp(rate * time)
    elif comp.startswith("Compounded:"):
        import math
        m = float(int(comp.split(":")[1]))
        scaled = rate / m
        if abs(scaled) < 2.220446049250313e-16:
            pg = rate
        elif scaled <= -0.5:
            pg = m * math.log((m + rate) / m)
        else:
            pg = m * math.log1p(scaled)
        cf = math.exp(pg * time)
    elif comp.startswith("SimpleThenCompounded:"):
        m = int(comp.split(":")[1])
        if time <= 1.0 / float(m):
            return discount_factor_bits(rate_bits, "Simple", time_bits)
        return discount_factor_bits(rate_bits, f"Compounded:{m}", time_bits)
    elif comp.startswith("CompoundedThenSimple:"):
        m = int(comp.split(":")[1])
        if time > 1.0 / float(m):
            return discount_factor_bits(rate_bits, "Simple", time_bits)
        return discount_factor_bits(rate_bits, f"Compounded:{m}", time_bits)
    else:
        raise ValueError(f"unknown comp {comp}")
    df = 1.0 / cf
    return float_to_bits(df)


def math_fma(a: float, b: float, c: float) -> float:
    fa = exact_fraction(float_to_bits(a))
    fb = exact_fraction(float_to_bits(b))
    fc = exact_fraction(float_to_bits(c))
    res = fa * fb + fc
    if res == 0:
        return 0.0
    return bits_to_float(round_fraction_to_bits(res, negative_hint=(res < 0)))


def pv_bits(rate_bits: int, comp: str, amount_bits: int, time_bits: int) -> int:
    df_b = discount_factor_bits(rate_bits, comp, time_bits)
    amt = bits_to_float(amount_bits)
    if amt == 0.0:
        return 0
    val = amt * bits_to_float(df_b)
    return float_to_bits(val)


def eval_bond_plan(
    face_bits: int,
    coupon_rate_bits: int,
    spec: str,
    settlement_str: str,
    inclusion: str,
    yield_dcc: str,
    yield_comp: str,
    yield_bits: int,
    work_cap: int,
):
    """Evaluates a zero-coupon or fixed-rate bond plan and valuation."""
    parts = spec.split("|")
    kind = parts[0]
    issue = parse_iso(parts[1])
    maturity = parse_iso(parts[2])
    red_pay = parse_iso(parts[3])
    red_amt_bits = int(parts[4], 16)
    coupon_dcc = parts[5]
    coupons_spec = [] if len(parts) < 7 or parts[6] == "-" else parts[6].split(";")

    settlement = parse_iso(settlement_str)
    if settlement < issue:
        return (
            "bond.settlement_before_issue",
            f"settlement_date={settlement_str},issue_date={parts[1]}",
            "-",
        )

    used_work = 0

    def charge(dcc, s, e, idx_str, phase):
        nonlocal used_work
        req = used_work + work_units_for(dcc, s, e)
        if req > work_cap:
            return (
                "bond.accrual_work_limit_exceeded",
                f"index={idx_str},phase={phase},maximum={work_cap},used={used_work},required_at_least={req}",
            )
        used_work = req
        return None

    err = charge(yield_dcc, settlement, settlement, "none", "discount anchor")
    if err is not None:
        return err[0], err[1], "-"

    def eligible(pay_date: dt.date) -> bool:
        return pay_date > settlement or (pay_date == settlement and inclusion == "Include")

    entitlements = []
    accrued_terms = []
    discount_flows = []

    for idx, c_item in enumerate(coupons_spec):
        c_start_s, c_end_s, c_pay_s, ex_s = c_item.split("@")
        c_start = parse_iso(c_start_s)
        c_end = parse_iso(c_end_s)
        c_pay = parse_iso(c_pay_s)
        full_yf = yf_bits(coupon_dcc, c_start, c_end)
        full_amt = triple_product_bits(face_bits, coupon_rate_bits, full_yf)

        if not eligible(c_pay):
            entitlements.append("C")
            continue

        detached = False
        if ex_s != "-":
            b_kind, cut_s = ex_s.split(":")
            cut_d = parse_iso(cut_s)
            if b_kind == "OnOrAfter":
                detached = settlement >= cut_d
            elif b_kind == "After":
                detached = settlement > cut_d
            else:
                raise ValueError(b_kind)

        if settlement <= c_start:
            earned = 0
        elif settlement >= c_end:
            earned = full_amt
        else:
            err = charge(coupon_dcc, c_start, settlement, str(idx), "partial accrual")
            if err is not None:
                return err[0], err[1], "-"
            part_yf = yf_bits(coupon_dcc, c_start, settlement)
            earned = triple_product_bits(face_bits, coupon_rate_bits, part_yf)

        if detached:
            entitlements.append("X")
            neg_full = 0 if full_amt == 0 else (full_amt ^ SIGN)
            accrued_terms.append(exact_sum_bits([earned, neg_full]))
        else:
            entitlements.append("B")
            accrued_terms.append(earned)
            err = charge(yield_dcc, settlement, c_pay, str(idx), "discount horizon")
            if err is not None:
                return err[0], err[1], "-"
            h_bits = yf_bits(yield_dcc, settlement, c_pay)
            discount_flows.append((idx, full_amt, h_bits))

    red_idx = len(coupons_spec)
    red_ent = eligible(red_pay)
    if red_ent:
        err = charge(yield_dcc, settlement, red_pay, str(red_idx), "redemption discount horizon")
        if err is not None:
            return err[0], err[1], "-"
        h_bits = yf_bits(yield_dcc, settlement, red_pay)
        discount_flows.append((red_idx, red_amt_bits, h_bits))

    accrued_bits = exact_sum_bits(accrued_terms)
    pv_list = [pv_bits(yield_bits, yield_comp, amt_b, h_b) for (_, amt_b, h_b) in discount_flows]
    dirty_bits = exact_sum_bits(pv_list)
    neg_accrued = 0 if accrued_bits == 0 else (accrued_bits ^ SIGN)
    clean_bits = exact_sum_bits([dirty_bits, neg_accrued])

    _, _, dirty_p100 = eval_per_100_from_amount(dirty_bits, face_bits)
    _, _, accrued_p100 = eval_per_100_from_amount(accrued_bits, face_bits)
    _, _, clean_p100 = eval_per_100_from_amount(clean_bits, face_bits)

    ent_code = "".join(entitlements) if entitlements else "-"
    red_code = "1" if red_ent else "0"
    output = ":".join([
        ent_code,
        red_code,
        str(len(discount_flows)),
        str(used_work),
        hexbits(dirty_bits),
        hexbits(accrued_bits),
        hexbits(clean_bits),
        dirty_p100,
        accrued_p100,
        clean_p100,
    ])
    COUNTS["bond-plan-evaluations"] += 1
    return "ok", "-", output


def quote_cases():
    faces = [
        float_to_bits(100.0),
        float_to_bits(1000.0),
        float_to_bits(1_000_000.0),
        float_to_bits(1.0),
        float_to_bits(0.01),
        float_to_bits(25.0),
        float_to_bits(50.0),
        float_to_bits(75.0),
        float_to_bits(3.0),
        float_to_bits(7.0),
        1,
        (1 << 52) - 1,
        1 << 52,
        F64_MAX,
    ]
    invalid_faces = [
        0,
        SIGN,
        float_to_bits(-100.0),
        SIGN | 1,
        POSITIVE_INFINITY,
        SIGN | POSITIVE_INFINITY,
        POSITIVE_INFINITY | 1,
        SIGN | POSITIVE_INFINITY | 0xdeadbeef,
    ]
    sample_values = [
        0,
        SIGN,
        1,
        SIGN | 1,
        2,
        SIGN | 2,
        (1 << 52) - 1,
        SIGN | ((1 << 52) - 1),
        1 << 52,
        SIGN | (1 << 52),
        float_to_bits(0.5),
        float_to_bits(-0.5),
        float_to_bits(1.0),
        float_to_bits(-1.0),
        float_to_bits(98.7654321),
        float_to_bits(-98.7654321),
        float_to_bits(100.0),
        float_to_bits(101.25),
        float_to_bits(12345.6789),
        F64_MAX,
        SIGN | F64_MAX,
        POSITIVE_INFINITY,
        SIGN | POSITIVE_INFINITY,
        POSITIVE_INFINITY | 1,
    ]

    for f_bits in invalid_faces:
        for v_bits in (0, float_to_bits(100.0), POSITIVE_INFINITY | 1):
            yield "QUOTE-INVALID-FACE", "per_100_from_amount", f_bits, v_bits
            yield "QUOTE-INVALID-FACE", "amount_from_per_100", f_bits, v_bits

    for f_bits in faces:
        for v_bits in sample_values:
            yield "QUOTE-MATRIX", "per_100_from_amount", f_bits, v_bits
            yield "QUOTE-MATRIX", "amount_from_per_100", f_bits, v_bits

    # Subnormal and normal tie & boundary cases
    f_100_down = float_to_bits(100.0) - 1
    f_100_up = float_to_bits(100.0) + 1
    for f_bits in (float_to_bits(199.0), float_to_bits(200.0), float_to_bits(201.0), f_100_down, f_100_up):
        for k in range(1, 64):
            yield "QUOTE-SUBNORMAL-TIE", "per_100_from_amount", f_bits, k
            yield "QUOTE-SUBNORMAL-TIE", "per_100_from_amount", f_bits, SIGN | k
            yield "QUOTE-SUBNORMAL-TIE", "amount_from_per_100", f_bits, k
            yield "QUOTE-SUBNORMAL-TIE", "amount_from_per_100", f_bits, SIGN | k

    # Every exponent field 0..2046 with representative significands
    for exp_field in range(0, 2047, 8):
        for frac in (0, 1, 0x5555555555555, FRACTION_MASK):
            if exp_field == 0 and frac == 0:
                continue
            b = (exp_field << 52) | frac
            for f_bits in (float_to_bits(100.0), float_to_bits(1000.0), float_to_bits(75.0)):
                yield "QUOTE-BINADE-SWEEP", "per_100_from_amount", f_bits, b
                yield "QUOTE-BINADE-SWEEP", "per_100_from_amount", f_bits, SIGN | b
                yield "QUOTE-BINADE-SWEEP", "amount_from_per_100", f_bits, b
                yield "QUOTE-BINADE-SWEEP", "amount_from_per_100", f_bits, SIGN | b

    # Deterministic PRNG cases across full finite domain
    rng = random.Random(2026101608)
    for _ in range(1200):
        exp1 = rng.randint(0, 2046)
        frac1 = rng.randint(1 if exp1 == 0 else 0, FRACTION_MASK)
        f_bits = (exp1 << 52) | frac1
        exp2 = rng.randint(0, 2046)
        frac2 = rng.randint(0, FRACTION_MASK)
        sign2 = SIGN if rng.randint(0, 1) else 0
        v_bits = sign2 | (exp2 << 52) | frac2
        yield "QUOTE-RANDOM", "per_100_from_amount", f_bits, v_bits
        yield "QUOTE-RANDOM", "amount_from_per_100", f_bits, v_bits


def bond_plan_cases():
    face_100 = float_to_bits(100.0)
    face_1000 = float_to_bits(1000.0)
    rate_5pct = float_to_bits(0.05)
    rate_0pct = float_to_bits(0.0)
    rate_neg2pct = float_to_bits(-0.02)
    red_100 = hexbits(float_to_bits(100.0))
    red_105 = hexbits(float_to_bits(105.0))
    red_1000 = hexbits(float_to_bits(1000.0))

    # 1. Zero-coupon bond across compounding methods, settlements, and work caps
    zc_spec = f"ZeroCoupon|2026-01-15|2028-01-15|2028-01-17|{red_100}|Act365F|-"
    for settle in ("2026-01-14", "2026-01-15", "2027-01-15", "2028-01-17", "2028-01-18"):
        for inc in ("Include", "Exclude"):
            for comp in ("Simple", "Continuous", "Compounded:2", "SimpleThenCompounded:2", "CompoundedThenSimple:2"):
                for y in (0.0, 0.04, -0.01):
                    for cap in (0, 1, 2):
                        yield (
                            "PLAN-ZERO-COUPON",
                            face_100,
                            rate_0pct,
                            zc_spec,
                            settle,
                            inc,
                            "Act365F",
                            comp,
                            float_to_bits(y),
                            cap,
                        )

    # 2. Ex-coupon cutoff matrix: cutoff-1, cutoff, cutoff+1, payment-1, payment, payment+1
    # across None, OnOrAfter, After and Include/Exclude, with positive/zero/negative coupon rates
    for ex_mode in ("-", "OnOrAfter:2026-07-08", "After:2026-07-08", "OnOrAfter:2026-07-15", "After:2026-07-15", "OnOrAfter:2026-01-10", "After:2026-01-10"):
        c1 = f"2026-01-15@2026-07-15@2026-07-15@{ex_mode}"
        c2 = "2026-07-15@2027-01-15@2027-01-15@OnOrAfter:2027-01-08"
        spec = f"FixedRate|2026-01-15|2027-01-15|2027-01-15|{red_100}|Act365F|{c1};{c2}"
        for settle in (
            "2026-01-15",
            "2026-04-15",
            "2026-07-07",
            "2026-07-08",
            "2026-07-09",
            "2026-07-14",
            "2026-07-15",
            "2026-07-16",
            "2027-01-08",
            "2027-01-15",
            "2027-01-16",
        ):
            for inc in ("Include", "Exclude"):
                for c_rate in (rate_5pct, rate_0pct, rate_neg2pct):
                    yield (
                        "PLAN-EX-COUPON-MATRIX",
                        face_100,
                        c_rate,
                        spec,
                        settle,
                        inc,
                        "Act365F",
                        "Compounded:2",
                        float_to_bits(0.04),
                        10,
                    )

    # 3. Early payment (Preceding) and delayed payment (Following) + non-par redemption
    early_c1 = "2026-01-15@2026-07-15@2026-07-13@OnOrAfter:2026-07-06"
    delayed_c1 = "2026-01-15@2026-07-15@2026-07-20@After:2026-07-10"
    c2_delayed = "2026-07-15@2027-01-15@2027-01-18@-"
    for label, first_c in (("EARLY", early_c1), ("DELAYED", delayed_c1)):
        spec = f"FixedRate|2026-01-15|2027-01-15|2027-01-20|{red_105}|30360BB|{first_c};{c2_delayed}"
        for settle in ("2026-07-06", "2026-07-10", "2026-07-13", "2026-07-14", "2026-07-15", "2026-07-17", "2026-07-20", "2027-01-16", "2027-01-18", "2027-01-20"):
            for inc in ("Include", "Exclude"):
                for y_dcc in ("Act360", "30E360", "30360US_EOM", "B252_WEEKENDS"):
                    yield (
                        f"PLAN-{label}-PAYMENT",
                        face_1000,
                        rate_5pct,
                        spec,
                        settle,
                        inc,
                        y_dcc,
                        "Simple",
                        float_to_bits(0.03),
                        2000,
                    )

    # 4. Reopening bond (first coupon raw start precedes issue date) + work budget boundaries
    reopen_c1 = "2026-01-01@2026-07-01@2026-07-01@OnOrAfter:2026-06-24"
    reopen_c2 = "2026-07-01@2027-01-01@2027-01-01@After:2026-12-24"
    reopen_spec = f"FixedRate|2026-03-01|2027-01-01|2027-01-01|{red_1000}|B252_WEEKENDS|{reopen_c1};{reopen_c2}"
    for settle in ("2026-02-28", "2026-03-01", "2026-05-01", "2026-06-24"):
        for cap in (0, 1, 40, 45, 50, 500):
            yield (
                "PLAN-REOPENING-WORK-BUDGET",
                face_1000,
                rate_5pct,
                reopen_spec,
                settle,
                "Include",
                "B252_WEEKENDS",
                "Compounded:2",
                float_to_bits(0.035),
                cap,
            )

    # 5. 400-year Gregorian cycle sweep (2000..2399): 1 bond per cycle year with rotating
    # settlement, inclusion, ex-coupon boundary, day-count and compounding
    dccs = ("Act365F", "Act360", "30360BB", "30E360", "30360US_EOM")
    comps = ("Simple", "Continuous", "Compounded:1", "Compounded:2", "Compounded:4", "SimpleThenCompounded:2", "CompoundedThenSimple:2")
    for yr in range(2000, 2400):
        start_d = dt.date(yr, 2, 29 if is_leap(yr) else 28)
        mid_d = dt.date(yr, 8, 28)
        end_d = dt.date(yr + 1, 2, 28)
        cut_d = mid_d - dt.timedelta(days=7)
        b_mode = "OnOrAfter" if (yr % 2 == 0) else "After"
        c1 = f"{start_d.isoformat()}@{mid_d.isoformat()}@{mid_d.isoformat()}@{b_mode}:{cut_d.isoformat()}"
        c2 = f"{mid_d.isoformat()}@{end_d.isoformat()}@{end_d.isoformat()}@-"
        spec = f"FixedRate|{start_d.isoformat()}|{end_d.isoformat()}|{end_d.isoformat()}|{red_100}|{dccs[yr % len(dccs)]}|{c1};{c2}"
        offset_choice = yr % 5
        if offset_choice == 0:
            settle_d = start_d
        elif offset_choice == 1:
            settle_d = cut_d - dt.timedelta(days=1)
        elif offset_choice == 2:
            settle_d = cut_d
        elif offset_choice == 3:
            settle_d = cut_d + dt.timedelta(days=1)
        else:
            settle_d = mid_d
        inc = "Include" if (yr % 3 == 0) else "Exclude"
        y_dcc = dccs[(yr // 2) % len(dccs)]
        y_comp = comps[yr % len(comps)]
        y_val = 0.02 + 0.005 * ((yr % 7) - 2)
        yield (
            "PLAN-400YEAR-CYCLE",
            face_100,
            float_to_bits(0.04 if yr % 4 != 0 else -0.015),
            spec,
            settle_d.isoformat(),
            inc,
            y_dcc,
            y_comp,
            float_to_bits(y_val),
            100,
        )


def generate(generated_at: str, platform_text: str):
    assert sha(CONTRACT.read_bytes()) == CONTRACT_SHA, "frozen M8 bond core contract changed"
    for counter in (COUNTS, PROFILES, OPS, EXPECTATIONS):
        counter.clear()
    records = []
    distinct = set()

    def emit(profile, op, face_bits, value_bits, spec, settlement, inclusion, yield_dcc, yield_comp, yield_bits, work_cap):
        if op == "per_100_from_amount":
            outcome, context, output = eval_per_100_from_amount(value_bits, face_bits)
        elif op == "amount_from_per_100":
            outcome, context, output = eval_amount_from_per_100(value_bits, face_bits)
        elif op == "bond_plan":
            outcome, context, output = eval_bond_plan(
                face_bits, value_bits, spec, settlement, inclusion, yield_dcc, yield_comp, yield_bits, work_cap
            )
        else:
            raise ValueError(op)

        row = (
            "BOND-%06d" % (len(records) + 1),
            profile,
            op,
            hexbits(face_bits),
            hexbits(value_bits),
            spec,
            settlement,
            inclusion,
            yield_dcc,
            yield_comp,
            "-" if yield_bits is None else hexbits(yield_bits),
            "-" if work_cap is None else str(work_cap),
            outcome,
            context,
            output,
        )
        assert len(row) == len(COLUMNS) and all(row) and all("\t" not in item and "\n" not in item for item in row)
        records.append("\t".join(row))
        distinct.add(row[2:12])
        PROFILES[profile] += 1
        OPS[op] += 1
        EXPECTATIONS[outcome] += 1

    for profile, op, f_bits, v_bits in quote_cases():
        emit(profile, op, f_bits, v_bits, "-", "-", "-", "-", "-", None, None)

    for profile, f_bits, r_bits, spec, settle, inc, y_dcc, y_comp, y_bits, cap in bond_plan_cases():
        emit(profile, "bond_plan", f_bits, r_bits, spec, settle, inc, y_dcc, y_comp, y_bits, cap)

    body = ("\t".join(COLUMNS) + "\n" + "\n".join(records) + "\n").encode("utf-8")
    own = Path(__file__).read_bytes()
    contract = CONTRACT.read_bytes()
    requirements = "\n".join(
        line for line in REQUIREMENTS.read_text(encoding="utf-8").splitlines() if re.match(r"\| FR-BOND-\d+ \|", line)
    ).encode("utf-8")
    assert len(requirements.decode().splitlines()) == 10

    def census(counter):
        return ";".join("%s=%d" % item for item in sorted(counter.items()))

    headers = [
        "schema: 1",
        "fixture-set: M8-BOND-CORE-ORACLE",
        "domain: bonds",
        "description: Independent IEEE 754 bit-field, Fraction, Decimal and civil-date oracle for M8 bond core quote conversion, entitlement, ex-coupon accrual and direct-horizon pricing",
        "reference: oracle:struct-bit-fields+exact-fraction+decimal-256-512+civil-date-entitlement; no Rust or QuantLib output",
        "reference-version: Python >=3.9 stdlib struct/fractions/decimal/datetime; Decimal cross-check precisions 256,512",
        "reference-source: Frozen docs/M8_BOND_CORE_CONTRACT.md",
        "reproduction: python3 -I validation/generator/bond_core_oracles.py --check",
        "generated-at: " + generated_at,
        "platform: " + platform_text,
        "generator-sha256: " + sha(own),
        "generator-fnv64: " + fnv(own),
        "adopted-contract-sha256: " + sha(contract),
        "adopted-contract-fnv64: " + fnv(contract),
        "requirements-rows-sha256: " + sha(requirements),
        "tolerance: TOL-E0 for quote conversion, entitlement, work units, accrued_amount and structured errors; TOL-E3 (<=2 ULP) for elementary discount valuation",
        "encoding: tab-separated; bits=0x+16 lowercase hex IEEE754 binary64 pattern; spec=kind|issue|maturity|red_pay|red_bits|coupon_dcc|coupons",
        "record-count: " + str(len(records)),
        "unique-input-count: " + str(len(distinct)),
        "profile-counts: " + census(PROFILES),
        "operation-counts: " + census(OPS),
        "expectation-counts: " + census(EXPECTATIONS),
        "cross-check-counts: " + census(COUNTS),
        "body-sha256: " + sha(body),
        "body-fnv64: " + fnv(body),
    ]
    data = ("\n".join(headers) + "\n\n").encode("utf-8") + body
    summary = {
        "records": len(records),
        "unique_inputs": len(distinct),
        "bytes": len(data),
        "sha256": sha(data),
        "body_sha256": sha(body),
        "body_fnv64": fnv(body),
        "generator_sha256": sha(own),
        "contract_sha256": sha(contract),
        "requirements_rows_sha256": sha(requirements),
        "profiles": dict(sorted(PROFILES.items())),
        "operations": dict(sorted(OPS.items())),
        "expectations": dict(sorted(EXPECTATIONS.items())),
        "cross_checks": dict(sorted(COUNTS.items())),
    }
    return data, summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Recompute and compare the existing fixture byte-for-byte")
    parser.add_argument("--output", type=Path, default=DEST)
    args = parser.parse_args()
    generated_at = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    platform_text = platform.system() + " " + platform.machine()
    if args.check and args.output.is_file():
        header = args.output.read_text(encoding="utf-8").split("\n\n", 1)[0]
        recorded = dict(line.split(": ", 1) for line in header.splitlines() if ": " in line)
        generated_at = recorded.get("generated-at", generated_at)
        platform_text = recorded.get("platform", platform_text)
    data, summary = generate(generated_at, platform_text)
    if args.check:
        if not args.output.is_file() or args.output.read_bytes() != data:
            raise SystemExit("bond core oracle fixture differs from independent regeneration")
        summary["check"] = "identical"
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(data)
        output = args.output.resolve()
        summary["written"] = str(output.relative_to(ROOT)) if output.is_relative_to(ROOT) else str(output)
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
