#!/usr/bin/env python3
"""Independent two-precision Decimal oracle for the FS-RATE equations."""

import argparse
import datetime as dt
from decimal import Decimal as D, localcontext, getcontext
from fractions import Fraction as F
import hashlib
import math
import pathlib
import platform
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/rates"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
FREQUENCIES = (("Annual", 1), ("Semiannual", 2), ("EveryFourthMonth", 3),
               ("Quarterly", 4), ("Bimonthly", 6), ("Monthly", 12),
               ("EveryFourthWeek", 13), ("Biweekly", 26), ("Weekly", 52), ("Daily", 365))
METHODS = [("Simple", "", 0), ("Continuous", "", 0)] + [
    (method, frequency, count) for frequency, count in FREQUENCIES
    for method in ("Compounded", "SimpleThenCompounded", "CompoundedThenSimple")]
MIN_SUB = float.fromhex("0x0.0000000000001p-1022")
MAX = sys.float_info.max


class DomainError(Exception):
    pass


def decimal(value):
    return D.from_float(value)


def rounded(value):
    try:
        return float(value)
    except OverflowError:
        return math.inf if value > 0 else -math.inf


def log1p(value):
    # Preserve an arbitrarily tiny increment independently of Rust/libm.
    # The convergent series tail is below the working precision relative to
    # its sum; normal-sized arguments use Decimal's correctly rounded ln.
    if abs(value) >= D("0.125"):
        return (1 + value).ln()
    if not value:
        return D(0)
    total, power, index = D(0), value, 1
    threshold = D(10) ** (-getcontext().prec - 8)
    while True:
        term = power / index
        total += term if index % 2 else -term
        if abs(term) < abs(total) * threshold:
            return total
        power *= value
        index += 1
        if index > 5000:
            raise ArithmeticError("log1p series failed to converge")


def expm1(value):
    if value > 2000:
        return D("Infinity")
    if value < -2000:
        return D(-1)
    if abs(value) >= D("0.125"):
        return value.exp() - 1
    if not value:
        return D(0)
    total, term, index = D(0), value, 1
    threshold = D(10) ** (-getcontext().prec - 8)
    while True:
        total += term
        if abs(term) < abs(total) * threshold:
            return total
        index += 1
        term *= value / index
        if index > 5000:
            raise ArithmeticError("expm1 series failed to converge")


def phase(method, count, time):
    if method == "Simple" or method == "SimpleThenCompounded" and time <= 1.0/count or method == "CompoundedThenSimple" and time > 1.0/count:
        return "Simple"
    return "Continuous" if method == "Continuous" else "Compounded"


def validate_quote(rate, method):
    if method[2] and rate <= -method[2]:
        raise DomainError("periodic rate boundary")


def log_growth(rate, time, method):
    r, t = decimal(rate), decimal(time)
    kind = phase(method[0], method[2], time)
    if kind == "Simple":
        if 1+r*t <= 0:
            raise DomainError("nonpositive simple factor")
        return log1p(r*t)
    if kind == "Continuous":
        return r*t
    m = D(method[2])
    return m*t*log1p(r/m)


def factor_from_log(growth):
    if growth > 1000:
        return D("Infinity")
    if growth < -1000:
        return D(0)
    return growth.exp()


def unresolved_boundary(growth, boundary):
    # Independently evaluate the declared conservative logarithmic range
    # policy in Decimal; this does not certify platform elementary functions.
    allowance = (D(2) ** -53) * 8 * (1 + abs(growth))
    return abs(growth - boundary) <= allowance


def inverse(growth, time, method):
    t = decimal(time)
    kind = phase(method[0], method[2], time)
    if kind == "Simple":
        return expm1(growth)/t
    if kind == "Continuous":
        return growth/t
    m = D(method[2])
    return m*expm1(growth/(m*t))


def expected(case, precision):
    op, source, rate, time, factor, target = case
    with localcontext() as context:
        context.prec = precision
        try:
            if op in ("compound", "discount", "equivalent"):
                validate_quote(rate, source)
            if time < 0:
                raise DomainError("negative horizon")
            if op == "equivalent":
                if not time or not rate:
                    value = D(0)
                else:
                    growth = log_growth(rate, time, source)
                    value = inverse(growth, time, target)
            elif op in ("implied", "implied-discount"):
                if factor <= 0:
                    raise DomainError("nonpositive factor")
                if not time:
                    if factor != 1:
                        return "error", "rate.zero_time", ""
                    value = D(0)
                else:
                    growth = decimal(factor).ln()
                    if op == "implied-discount":
                        growth = -growth
                    if phase(source[0],source[2],time) == "Simple":
                        supplied = F.from_float(factor)
                        value = ((supplied-1) if op == "implied" else (1/supplied-1))/F.from_float(time)
                    else:
                        value = inverse(growth,time,source)
            else:
                if not time or not rate:
                    value = D(1)
                elif phase(source[0],source[2],time) == "Simple":
                    value = 1+F.from_float(rate)*F.from_float(time)
                else:
                    growth = log_growth(rate,time,source)
                    if (unresolved_boundary(growth, decimal(MAX).ln()) or
                            unresolved_boundary(growth, decimal(MIN_SUB).ln()-D(2).ln())):
                        return "error", "rate.numerical_boundary", ""
                    value = factor_from_log(growth)
                binary_factor = rounded(value)
                # Discount follows the project policy: a representable C is
                # required, then one correctly rounded binary64 reciprocal.
                if not math.isfinite(binary_factor) or binary_factor <= 0:
                    return "error", "rate.value", ""
                if op == "discount":
                    if time and rate and phase(source[0], source[2], time) != "Simple":
                        if unresolved_boundary(-decimal(binary_factor).ln(), decimal(MAX).ln()):
                            return "error", "rate.numerical_boundary", ""
                    value = 1/F.from_float(binary_factor)
            binary = rounded(value)
            if not math.isfinite(binary):
                return "error", "rate.value", ""
            if op in ("compound", "discount"):
                if binary <= 0:
                    return "error", "rate.value", ""
            else:
                selector = target if op == "equivalent" else source
                if selector[2] and binary <= -selector[2]:
                    return "error", "rate.value", ""
                if value and not binary:
                    return "error", "rate.value", ""
                if time and phase(selector[0], selector[2], time) == "Simple":
                    # The rounded inferred/converted quote must remain usable
                    # in its active simple domain at the requested horizon.
                    # Positive infinite factors remain allowed here because
                    # conversion does not need a materialized factor output.
                    if 1 + F.from_float(binary) * F.from_float(time) <= 0:
                        return "error", "rate.value", ""
            return "value", "", repr(0.0 if binary == 0 else binary)
        except DomainError:
            return "error", "rate.value", ""


def cases(operation):
    if operation in ("compound", "discount"):
        for method in METHODS:
            rates = [0.0, -0.0, MIN_SUB, -MIN_SUB, 1e-20, -1e-20, 1e-12, -0.005, -0.05, 0.05, 1.0, 10.0, MAX, -MAX]
            if method[2]:
                rates += [-float(method[2]), math.nextafter(-float(method[2]), math.inf)]
            times = [0.0, MIN_SUB, 1e-12, 1e-6, 1.0/365.0, 0.1, 0.5, 1.0, 10.0, 100.0, 744.0, 750.0, MAX]
            if method[2]:
                threshold = 1.0/method[2]
                times += [math.nextafter(threshold,0.0),threshold,math.nextafter(threshold,math.inf)]
            for rate in sorted(set(rates)):
                for time in sorted(set(times)):
                    yield operation, method, rate, time, 1.0, method
        # Exact cancellation: ordinary multiplication rounds to -1, fused C
        # retains the positive 2^-104 residual.
        yield operation, METHODS[0], -1.0-math.ulp(1.0), 1.0-math.ulp(1.0), 1.0, METHODS[0]
    elif operation in ("implied", "implied-discount"):
        factors = [MIN_SUB, sys.float_info.min, 1e-100, 0.5, math.nextafter(1.0,0.0),1.0,math.nextafter(1.0,math.inf),1.1,2.0,1e100,MAX]
        for method in METHODS:
            for factor in factors:
                for time in [0.0,MIN_SUB,1e-12,1.0/365.0,0.1,1.0,100.0,MAX]:
                    yield operation,method,0.0,time,factor,method
    else:
        for source in METHODS:
            rates = [-0.005,0.0,-1e-20,1e-20,0.05,MIN_SUB]
            if source[2]:
                rates.append(math.nextafter(-float(source[2]),math.inf))
            for target in METHODS:
                for rate in rates:
                    for time in [0.0,1e-12,1.0/365.0,1.0,100.0,MAX]:
                        yield operation,source,rate,time,1.0,target


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    columns = ("id","operation","method","frequency","rate","time","factor","target_method","target_frequency","behavior","error_kind","expected","tolerance")
    for operation in ("compound","discount","implied","implied-discount","equivalent"):
        lines, counts = ["\t".join(columns)], {"value":0,"error":0}
        for index, case in enumerate(cases(operation)):
            low = expected(case,120)
            high = expected(case,200)
            if low != high:
                raise ArithmeticError("unstable oracle rounding: " + str((case,low,high)))
            behavior,error,value = high
            _,source,rate,time,factor,target = case
            row = ("RATE-%s-%06d" % (operation.upper(),index),operation,source[0],source[1],repr(rate),repr(time),repr(factor),target[0],target[1],behavior,error,value,"TOL-E3" if operation in ("compound","discount") else "TOL-E4")
            lines.append("\t".join(row));counts[behavior] += 1
        metadata = {
            "schema":"1","fixture-set":"RATE-ORACLE-"+operation.upper(),"domain":"rates",
            "description":"FS-RATE equations; exact binary64 inputs; stable Decimal log1p/expm1 series; two-precision rounded-output check",
            "reference":"oracle:Python Decimal","reference-version":platform.python_version()+" Decimal 120/200 digits",
            "reference-source":"Project FS-RATE equations and Python Decimal independently rounded elementary functions; no Rust or QuantLib outputs",
            "tolerance":"TOL-E3/E4 initial k=8; calibration pending; positive subnormal factors require absolute rounding-unit analysis",
            "reproduction":"generator-sha256="+REVISION+"; python3 validation/generator/rate_oracles.py",
            "generated-at":dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00","Z"),
            "platform":platform.system()+" "+platform.machine(),"switch-threshold":"rounded binary64 1/frequency, as specified in NP-12",
        }
        body = "\n".join(lines)+"\n"
        path = DEST/(operation+"-oracle.fixture")
        if args.check:
            header,old_body = path.read_text().split("\n\n",1)
            old = dict(line.split(": ",1) for line in header.splitlines())
            stable = metadata.keys()-{"generated-at","platform"}
            if old_body != body or any(old.get(key)!=metadata[key] for key in stable):
                raise SystemExit("fixture differs: "+str(path))
            action = "verified"
        else:
            DEST.mkdir(parents=True,exist_ok=True)
            path.write_bytes(("\n".join(key+": "+value for key,value in metadata.items())+"\n\n"+body).encode())
            action = "generated"
        print(action,path.relative_to(ROOT),"records="+str(len(lines)-1),counts,flush=True)


if __name__ == "__main__":
    main()
