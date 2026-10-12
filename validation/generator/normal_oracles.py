#!/usr/bin/env python3
"""Independent exact-input, two-precision Decimal normal-distribution oracles."""

import argparse
import datetime as dt
from decimal import Decimal as D, localcontext, getcontext
from functools import lru_cache
import hashlib
import math
import pathlib
import platform
import struct
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/math/normal"
REVISION = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
MIN_SUB = float.fromhex("0x0.0000000000001p-1022")
MAX = sys.float_info.max
UNIT = D.from_float(float.fromhex("0x1p-53"))
OPS = ("pdf", "cdf", "survival", "log-cdf", "log-survival", "inverse")


def bits(value):
    return "0x%016x" % struct.unpack(">Q", struct.pack(">d", value))[0]


def atan_series(x):
    total, power, index = x, x, 1
    square = x*x
    while True:
        power *= -square
        old = total
        total += power / (2*index+1)
        if total == old:
            return total
        index += 1
        if index > 10000:
            raise ArithmeticError("pi arctangent series exceeded its bound")


@lru_cache(None)
def constants(precision):
    with localcontext() as context:
        context.prec = precision + 20
        # Exact Machin identity, evaluated through independently convergent
        # arctangent series; no imported decimal coefficient table.
        pi = 16*atan_series(D(1)/5)-4*atan_series(D(1)/239)
        root = (2*pi).sqrt()
        return root, root.ln()


def density_log(z, precision):
    return -(z*z)/2 - constants(precision)[1]


def centered(z, precision):
    total, term, square = z, z, z*z
    threshold = D(10) ** (-precision-10)
    for index in range(1, 4097):
        term *= square / (2*index+1)
        total += term
        ratio = square/(2*index+3)
        if ratio < 1 and term*ratio/(1-ratio) <= abs(total)*threshold:
            return density_log(z, precision).exp() * total
    raise ArithmeticError("oracle center series exceeded its bound")


def mills(z, depth):
    tail = D(0)
    for numerator in range(depth, 0, -1):
        tail = numerator/(z+tail)
    return 1/(z+tail)


def checked_mills(z):
    depth = 64 if z >= 20 else 128
    lower, upper = mills(z, depth+1), mills(z, depth)
    if not 0 < lower <= upper or upper-lower > upper*D("1e-70"):
        raise ArithmeticError("Mills convergent separation too large: " + str(z))
    return (lower+upper)/2


def log_small_tail(z, precision):
    if z < 10:
        return (D("0.5") - centered(z, precision)).ln()
    return density_log(z, precision) + checked_mills(z).ln()


@lru_cache(None)
def inverse(probability, precision):
    with localcontext() as context:
        context.prec = precision
        p = D.from_float(probability)
        if p == D("0.5"):
            return D(0)
        negative = p < D("0.5")
        q = min(p, 1-p)
        # Independent bisection, with a tighter near-center bracket. It does
        # not use the Rust Newton solver, its branch threshold or its outputs.
        if q >= D("0.25"):
            target = abs(p-D("0.5"))
            lower, upper = D(0), target*3
            for _ in range(180):
                middle = (lower+upper)/2
                if centered(middle, precision) < target:
                    lower = middle
                else:
                    upper = middle
        else:
            target = q.ln()
            lower, upper = D(0), D(40)
            for _ in range(180):
                middle = (lower+upper)/2
                if log_small_tail(middle, precision) > target:
                    lower = middle
                else:
                    upper = middle
        value = (lower+upper)/2
        return -value if negative else value


def rounded(value):
    try:
        return float(value)
    except OverflowError:
        return math.copysign(math.inf, -1 if value < 0 else 1)


@lru_cache(None)
def forward(mean, scale, variate, precision):
    with localcontext() as context:
        context.prec = precision
        z = (D.from_float(variate)-D.from_float(mean))/D.from_float(scale)
        square = z*z/2
        scale_log = D.from_float(scale).ln()
        log_pdf = -square-scale_log-constants(precision)[1]
        pdf = D("Infinity") if log_pdf > 1000 else D(0) if log_pdf < -1000 else log_pdf.exp()
        log_q = log_small_tail(abs(z), precision)
        q = D(0) if log_q < -1000 else log_q.exp()
        # Avoid cancellation for logarithms of a probability close to one.
        # A convergent ln(1-q) series retains arbitrarily tiny q.
        if q < D("0.125"):
            log_one_minus_q, power = D(0), q
            threshold = D(10) ** (-precision-10)
            for index in range(1, 4097):
                term = power/index
                log_one_minus_q -= term
                if term <= max(abs(log_one_minus_q), D("1e-999999"))*threshold:
                    break
                power *= q
            else:
                raise ArithmeticError("log1p oracle exceeded its bound")
        else:
            log_one_minus_q = (1-q).ln()
        if z <= 0:
            cdf, survival, log_cdf, log_survival = q, 1-q, log_q, log_one_minus_q
        else:
            cdf, survival, log_cdf, log_survival = 1-q, q, log_one_minus_q, log_q
        uncertainty_scale = 1+square+abs(scale_log)+constants(precision)[1]
        return {"pdf":pdf, "cdf":cdf, "survival":survival,
                "log-cdf":log_cdf, "log-survival":log_survival}, z, log_pdf, uncertainty_scale


def expected(case, precision):
    operation, mean, scale, argument = case
    if not all(map(math.isfinite, (mean, scale, argument))) or scale <= 0:
        return "error", "normal.value", "", "error", "ordinary", "0", "0", "0", "0", "0"
    with localcontext() as context:
        context.prec = precision
        if operation == "inverse":
            if not 0 < argument < 1:
                return "error", "normal.value", "", "error", "ordinary", "0", "0", "0", "0", "0"
            z = inverse(argument, precision)
            value = D.from_float(scale)*z + D.from_float(mean)
            log_pdf, uncertainty_scale = D(0), D(0)
        else:
            values, z, log_pdf, uncertainty_scale = forward(mean, scale, argument, precision)
            value = values[operation]
        binary = rounded(value)
        true_behavior = "value" if math.isfinite(binary) else "error"
        behavior, error, tier = true_behavior, "" if true_behavior == "value" else "normal.value", "ordinary"
        affine_distance, affine_allowance = D(0), D(0)
        if operation == "pdf" and scale != 1:
            distance = abs(log_pdf-D.from_float(MAX).ln())
            unit_scale = UNIT*uncertainty_scale
            if distance <= 8*unit_scale:
                behavior, error = "error", "normal.numerical_boundary"
            if 6*unit_scale <= distance <= 10*unit_scale:
                tier = "guard-edge"
        elif operation == "inverse" and z != 0:
            # Compare the independent true affine quantile with the actual
            # nearest-even overflow midpoint, not MAX or a rounded Rust z.
            # Quarter scaling mirrors units only, not the Rust FMA/accumulator.
            midpoint = D.from_float(MAX) + D(2)**970
            affine_distance = abs(abs(value)/4-midpoint/4)
            p = D.from_float(argument)
            if D("0.25") <= p <= D("0.75"):
                standard_allowance = 16*UNIT*abs(z)
            else:
                log_tail = min(p,1-p).ln().copy_abs()
                standard_allowance = 8*UNIT*(1+abs(z)+log_tail/abs(z))
            affine_allowance = D.from_float(scale)/4*standard_allowance + 2*UNIT*affine_distance + 2*D.from_float(MIN_SUB)
            if affine_distance <= affine_allowance:
                behavior, error = "error", "normal.numerical_boundary"
            if D("0.75")*affine_allowance <= affine_distance <= D("1.25")*affine_allowance:
                tier = "guard-edge"
        # Preserve the true finite value even when the declared guard returns
        # an error: a guard-edge row may permit a finite observed result, whose
        # numerical accuracy must still be checked rather than waved through.
        result = "" if true_behavior == "error" else bits(0.0 if binary == 0 else binary)
        return (behavior, error, result, true_behavior, tier, repr(rounded(z)),
                repr(rounded(log_pdf)), repr(rounded(uncertainty_scale)),
                repr(rounded(affine_distance)), repr(rounded(affine_allowance)))


def neighbors(value, count=2):
    result = {value}
    lower, upper = value, value
    for _ in range(count):
        lower = math.nextafter(lower, -math.inf)
        upper = math.nextafter(upper, math.inf)
        if math.isfinite(lower): result.add(lower)
        if math.isfinite(upper): result.add(upper)
    return result


def seeded_units(count, seed):
    state=seed
    mask=(1<<64)-1
    for _ in range(count):
        state=(state+0x9e3779b97f4a7c15)&mask
        value=state
        value=((value^(value>>30))*0xbf58476d1ce4e5b9)&mask
        value=((value^(value>>27))*0x94d049bb133111eb)&mask
        value^=value>>31
        yield (value>>11)*2.0**-53


def cases():
    values = {index/8 for index in range(-96,97)}
    values.update(sign*index/16 for sign in (-1,1) for index in range(448,641))
    values.update(sign*math.ldexp(1.0, exponent) for sign in (-1,1)
                  for exponent in range(-1074,1024,17))
    for edge in (0.0,1.5,10.0,38.0,38.5,40.0,math.sqrt(MAX),math.sqrt(MAX)*math.sqrt(2)):
        values.update(neighbors(edge,8))
        values.update(-value for value in neighbors(edge,8))
    values.update((-MAX,MAX,-1e100,1e100,-1000.,1000.))
    # Non-dyadic standardized inputs exercise quadratic-argument rounding,
    # which the exact dyadic center/tail grids deliberately isolate away.
    values.update(80*unit-40 for unit in seeded_units(256,0xa387c31f529bc680))
    seen = set()
    for argument in sorted(values):
        for operation in OPS[:-1]:
            case = operation, 0.0, 1.0, argument
            seen.add(case)
            yield case
    probabilities = {0.0,1.0,0.5,0.25,0.75,0.01,0.025,0.1,0.9,0.975,0.99}
    probabilities.update(math.ldexp(1.0,-exponent) for exponent in range(1,1075))
    probabilities.update(struct.unpack(">d",struct.pack(">Q",index))[0] for index in range(1,129))
    probabilities.update(unit for unit in seeded_units(256,0x8130bcba496aea1f) if unit>0)
    probabilities.update(math.ldexp(1+unit,-exponent) for exponent,unit in
                         zip(range(1,1075,17),seeded_units(64,0x7a6f218c48f2b9d1)))
    for edge in (0.25,0.5,0.75): probabilities.update(neighbors(edge,32))
    probabilities.update(1-value for value in list(probabilities) if 0 < value < 1)
    for probability in sorted(probabilities):
        yield "inverse",0.0,1.0,probability
    means = (-MAX,-1e100,-10.0,-0.0,10.0,1e100,MAX)
    scales = (MIN_SUB,sys.float_info.min,1e-100,1.0,15.0,1e100,MAX)
    for mean in means:
        for scale in scales:
            points = {mean}
            for z in (-40.,-39.,-8.,-1.5,-.1,.1,1.5,8.,39.,40.):
                value = mean+scale*z
                if math.isfinite(value): points.add(value)
            for argument in sorted(points):
                for operation in OPS[:-1]:
                    case=operation,mean,scale,argument
                    if case not in seen:
                        seen.add(case); yield case
            for probability in (MIN_SUB,1e-100,.025,.1,.25,.5,.75,.9,.975,math.nextafter(1.,0.)):
                yield "inverse",mean,scale,probability
    # Mean-point density overflow probes independently target exact-real
    # logarithm offsets. Subnormal scale spacing is much larger than u*scale:
    # ±1024 neighbors extend through the entire central guard and edge tier.
    with localcontext() as context:
        context.prec=200
        critical=rounded(1/(D.from_float(MAX)*constants(200)[0]))
    for scale in sorted(neighbors(critical,1024)):
        yield "pdf",0.0,scale,0.0
    # Boundary neighborhoods use independently bisected real quantiles.
    # Both output signs, same/opposite-sign means, center/tail probabilities
    # and every neighboring scale preserve exact binary64 affine inputs.
    # The wide scale neighborhoods cross the full guard and its edge tier.
    affine_seen = set()
    with localcontext() as context:
        context.prec=200
        midpoint=D.from_float(MAX)+D(2)**970
        for probability in (MIN_SUB,.01,.1,.25,.75,.9,.99,math.nextafter(1.,0.)):
            z=inverse(probability,200)
            sign=D(-1) if z<0 else D(1)
            for mean in (-MAX,-MAX/2,0.,MAX/2,MAX):
                critical=rounded((sign*midpoint-D.from_float(mean))/z)
                if math.isfinite(critical) and critical>0:
                    for scale in sorted(neighbors(critical,128)):
                        case="inverse",mean,scale,probability
                        if case not in affine_seen:
                            affine_seen.add(case); yield case
    for operation in OPS:
        for argument in (math.nan,math.inf,-math.inf):
            yield operation,0.0,1.0,argument


def verify_overlap():
    with localcontext() as context:
        context.prec=200
        for value in (D("1.5"),D(2),D(3),D(4),D(6),D(8),D(10)):
            center=D("0.5")-centered(value,200)
            a,b=mills(value,8192),mills(value,8193)
            tail=density_log(value,200).exp()*(a+b)/2
            if abs(center-tail)>abs(tail)*D("1e-100"):
                raise ArithmeticError("independent center/CF overlap disagreement")


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check",action="store_true")
    args=parser.parse_args()
    verify_overlap()
    rows={operation:[] for operation in OPS}
    columns=("id","operation","mean_bits","scale_bits","argument_bits","behavior","error_kind","expected_bits","true_behavior","tier","standardized","log_density","uncertainty_scale","affine_guard_distance","affine_guard_allowance","tolerance")
    for index,case in enumerate(cases()):
        low,high=expected(case,120),expected(case,200)
        if low != high:
            raise ArithmeticError("unstable two-precision oracle: "+str((case,low,high)))
        operation,mean,scale,argument=case
        row=("NORMAL-%06d"%index,operation,bits(mean),bits(scale),bits(argument),*high,
             "TOL-E4" if operation=="inverse" else "TOL-E3")
        rows[operation].append("\t".join(row))
        if index and index%2000==0: print("evaluated",index,"records",flush=True)
    for operation in OPS:
        path=DEST/(operation+"-oracle.fixture")
        metadata={"schema":"1","fixture-set":"NORMAL-ORACLE-"+operation.upper(),"domain":"math.normal",
            "description":"Exact binary64 location/scale/argument; independent Decimal center+Mills identities and quantile bisection",
            "reference":"oracle:Python Decimal","reference-version":platform.python_version()+" Decimal 120/200 digits",
            "reference-source":"DLMF 7.6.2/7.20.1; Gasull-Utzet 1307.3433v1 Thm1/Cor2/Eq6; no Rust or QuantLib outputs",
            "tolerance":"Conditioned E3/E4 k=8 (central quantile k=16); MIN_SUB absolute floor; density/affine guard-edge conditional tiers",
            "reproduction":"generator-sha256="+REVISION+"; python3 validation/generator/normal_oracles.py",
            "generated-at":dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00","Z"),
            "platform":platform.system()+" "+platform.machine(),
            "oracle-method":"Machin pi; positive center geometric-tail bound; >=10 Mills adjacent bounds relative separation <=1e-70; inverse 180 bisections",
            "guard-edge":"PDF 6uS <= abs(log_density-ln(MAX)) <= 10uS, S=1+z^2/2+abs(ln(scale))+ln(sqrt(2pi)); inverse 0.75A <= quarter-scaled distance to MAX+2^970 <= 1.25A, A=scale*standard-quantile-budget/4+2u*distance+2MIN_SUB; exact u=2^-53; finite truth permits finite/Boundary, overflow truth permits Value/Boundary"}
        body="\t".join(columns)+"\n"+"\n".join(rows[operation])+"\n"
        if args.check:
            old_header,old_body=path.read_text().split("\n\n",1)
            old=dict(line.split(": ",1) for line in old_header.splitlines())
            metadata["generated-at"]=old["generated-at"]
            metadata["platform"]=old["platform"]
            expected_text="\n".join(key+": "+value for key,value in metadata.items())+"\n\n"+body
            if path.read_bytes()!=expected_text.encode():
                raise SystemExit("fixture differs: "+str(path))
            action="verified"
        else:
            DEST.mkdir(parents=True,exist_ok=True)
            path.write_bytes(("\n".join(key+": "+value for key,value in metadata.items())+"\n\n"+body).encode())
            action="generated"
        print(action,path.relative_to(ROOT),"records="+str(len(rows[operation])),flush=True)


if __name__=="__main__": main()
