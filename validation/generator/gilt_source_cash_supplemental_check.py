#!/usr/bin/env python3
"""Supplemental POST-IMPLEMENTATION public conventional-gilt source/cash audit.

Authored after reading the production implementation. Expectations use Python
Fraction/date arithmetic independently; this is NOT a pre-production oracle.
Copies an exact attested crate, adds only one generated integration test, and
runs native debug/release plus strict Clippy offline. Neither source files nor
frozen fixtures are modified. All output is in a create-new external directory.
Requires Python >=3.9 and Rust >=1.99 on PATH; standard library only.
"""
from pathlib import Path
from fractions import Fraction as F
from datetime import date, timedelta, datetime, timezone
import argparse
import hashlib
import json
import os
import platform
import random
import shutil
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
CONTRACT = "docs/M8_GILT_CONTRACT.md"
CONTRACT_SHA = "03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292"

Q = [date(1998,12,7), date(1999,6,7), date(1999,12,7), date(2000,6,7), date(2000,12,7), date(2001,6,7)]
MIN, MAX = -(1<<127), (1<<127)-1
def pub(x):
    n = x * 10**6
    q, r = divmod(n.numerator, n.denominator)
    return F(q + (2*r >= n.denominator), 10**6)
def source(role, c, s, dividend):
    issue = Q[0] if role == 0 else date(1999,5,7)
    first = 2 if role == 2 else 1
    b = c/2
    s1 = (Q[1]-Q[0]).days
    r1 = (Q[1]-issue).days
    if dividend:
        return b if role == 0 else pub(b * F(r1 + (s1 if role == 2 else 0),s1))
    nxt = next(k for k,q in enumerate(Q) if q>s)
    actual = max(nxt,first)
    ex = s > Q[actual]-timedelta(days=7)
    if s >= Q[first] or role == 0:
        t = (s-Q[nxt-1]).days
        length = (Q[nxt]-Q[nxt-1]).days
        return b * F(t-length if ex else t,length)
    if role == 1:
        t=(s-issue).days
        return b * F(t-r1 if ex else t,s1)
    if s<Q[1]:
        return b*F((s-issue).days,s1)
    r2=(s-Q[1]).days
    s2=(Q[2]-Q[1]).days
    return b*F(r2-s2,s2) if ex else b*(F(r1,s1)+F(r2,s2))
def rounded(x,quantum,mode):
    f=x/quantum
    q,r=divmod(abs(f.numerator),f.denominator)
    if r and mode==0: return None
    inc = (mode==2 or (mode==3 and f>=0) or (mode==4 and f<0)
           or (mode==5 and (2*r>f.denominator or (2*r==f.denominator and q%2)))
           or (mode==6 and 2*r>=f.denominator)) if r else False
    return (-1 if f<0 else 1)*(q+inc)
def amount(coeff,scale):
    if coeff==0:return '0:0'
    while scale and coeff%10==0:
        coeff//=10;scale-=1
    if coeff<MIN or coeff>MAX:return None
    return f'{coeff}:{scale}'
def result(op,src,n,qc,qs,mode,lo,hi,guarantee):
    quantum=F(qc,10**qs)
    def rr(x,stage,point):
        if mode==0 and not point:return f'U:{stage}'
        v=rounded(x,quantum,mode)
        return f'E:{stage}:point:quantize:inexact' if v is None else v*qc
    if op in ('dv','av','div','ai'):
        stage={'dv':'source dividend','av':'source accrued','div':'cash coupon','ai':'cash accrued'}[op]
        v=rr(src*(n/100 if op in ('div','ai') else 1),stage,True)
        if isinstance(v,str):return v
        a=amount(v,qs)
        return f'V:{a}' if a is not None else f'E:{stage}:point:quantize:range:{int(v<0)}'
    stage='cash capital' if op=='comp' else 'cash total'
    point=lo==hi
    if op=='total':lo+=src;hi+=src
    if mode==0 and not point:return f'U:{stage}'
    left=rr(n*lo/100,stage,point)
    if isinstance(left,str):return left
    right=left if point else rr(n*hi/100,stage,point)
    if isinstance(right,str):return right
    operation='quantize'
    if op=='comp':
        a=rr(n*src/100,'cash accrued',True)
        if isinstance(a,str):return a
        left+=a;right+=a;point=left==right;stage='cash net';operation='sum'
    aa=amount(left,qs)
    if aa is None:return f'E:{stage}:{"point" if point else "lower"}:{operation}:range:{int(left<0)}'
    bb=amount(right,qs)
    if bb is None:return f'E:{stage}:upper:{operation}:range:{int(right<0)}'
    return f'D:{aa}:{guarantee}' if aa==bb else f'A:{aa}:{bb}'
def fbits(bits):return F.from_float(struct.unpack('>d',bits.to_bytes(8,'big'))[0])

rows=[]
census={}
def add(op,role,cc,cs,settle,nc,ns,qc,qs,mode,kind='d',lp='0:0',hp='0:0'):
    c=F(cc,10**cs);n=F(nc,10**ns)
    src=source(role,c,settle,op in ('dv','div'))
    if kind in ('b','bi'):lo=fbits(int(lp,16));hi=fbits(int(hp,16))
    else:
        def dec(s):
            a,b=s.split(':');return F(int(a),10**int(b))
        lo=dec(lp);hi=dec(hp)
    guarantee='declared' if kind in ('di','bi') else ('binary' if kind=='b' else 'decimal')
    expect=result(op,src,n,qc,qs,mode,lo,hi,guarantee)
    row=[str(len(rows)),op,str(role),str(cc),str(cs),settle.isoformat(),str(nc),str(ns),str(qc),str(qs),str(mode),kind,lp,hp,expect]
    rows.append('|'.join(row));census[op]=census.get(op,0)+1

branches=[(0,date(1999,5,30)),(0,date(1999,6,1)),(1,date(1999,5,30)),(1,date(1999,6,1)),(2,date(1999,5,30)),(2,date(1999,11,30)),(2,date(1999,12,1)),(2,Q[1]),(2,Q[2])]
for branch,(role,s) in enumerate(branches):
    for scale in range(39):
        for mode in range(7):
            # Full coefficient, minimum positive coefficient, arbitrary quanta,
            # and opposite scale extremes retain source/nominal cancellation.
            cc=[0,1,MAX,MAX-1,123456789][(branch+scale+mode)%5]
            nc=[1,100,MAX][(branch+scale+mode)%3]
            ns=[0,38,scale][(branch+mode)%3]
            qc=[1,3,MAX][(scale+mode)%3]
            qs=[0,38,scale][(branch+scale)%3]
            add('av' if mode%2 else 'ai',role,cc,scale,s,nc,ns,qc,qs,mode)
for role in range(3):
    for scale in range(39):
        for mode in range(7):
            add('dv' if mode%2 else 'div',role,[1,MAX][scale%2],scale,date(1999,5,30),[1,MAX][mode%2],38-scale,[1,3,MAX][mode%3],scale,mode)
rng=random.Random(0x47534f55524345)
binary=[0,1,2,0x000fffffffffffff,0x0010000000000000,0x3fb999999999999a,0x3ff0000000000000,0x4004000000000000,0x7fefffffffffffff]
signed=sorted(binary+[x|(1<<63) for x in binary],key=fbits)
for i in range(700):
    role,s=branches[i%len(branches)]
    op=['dirty','total','comp'][i%3]
    kind=['d','di','b','bi'][i%4]
    if kind in ('d','di'):
        points=[(MIN,0),(-MAX,38),(-1,38),(0,0),(1,38),(MAX,38),(MAX,0)]
        left=rng.choice(points);right=rng.choice(points)
        left,right=sorted([left,right],key=lambda x:F(x[0],10**x[1]))
        lp=f'{left[0]}:{left[1]}';hp=lp if kind=='d' else f'{right[0]}:{right[1]}'
    else:
        left,right=sorted([rng.choice(signed),rng.choice(signed)],key=fbits)
        lp=f'{left:016x}';hp=lp if kind=='b' else f'{right:016x}'
    add(op,role,[0,1,MAX][i%3],[0,38,i%39][(i//3)%3],s,[1,MAX][i%2],[0,38][(i//2)%2],[1,3,MAX][(i//4)%3],[0,38,i%39][(i//5)%3],i%7,kind,lp,hp)
# Additional algebraic publication, Exact-order and collapsed-image witnesses.
# Q0..Q1 has 182 days; C=182e-6 gives a half-micro short/long dividend.
for role in (1, 2):
    add('dv',role,182,6,date(1999,5,30),100,0,1,6,0)
# At S=June 6 regular ex AI = -C/364: C=109382 gives exactly -300.5.
# A total invoice cancels exactly, while components fail at capital Exact.
for op in ('total','comp'):
    add(op,0,109382,0,date(1999,6,6),MAX,0,1,0,0,'d','3005:1','3005:1')
# Capital is exactly 3*MAX, then the nondecimal source AI fails Exact.
add('comp',0,109383,0,date(1999,6,6),MAX,0,1,0,0,'d','300:0','300:0')
# Both endpoint outputs collapse to 2*MAX. Total reports lower; net reports point.
for op in ('dirty','total','comp'):
    add(op,0,0,0,date(1999,5,30),MAX,0,MAX,0,5,'di','201:0','202:0')
# Quantized source view can fail while nominal scaling resolves exact source cash.
add('dv',0,1,38,date(1999,5,30),200,0,1,38,0)
add('div',0,1,38,date(1999,5,30),200,0,1,38,0)

RUST=r'''
//! Supplemental post-implementation exact source/cash audit (FR-GILT-01..06/15..21/24).
#![expect(clippy::unwrap_used, reason = "self-contained generated test inputs are fixed and invalid test data must fail")]
#![expect(clippy::panic, reason = "unexpected oracle outcome variants must fail this test")]
use sabiquant::amounts::{AmountError,AmountQuantum,AmountRounding,DecimalAmount};
use sabiquant::bonds::{ConventionalGilt,GiltCashDecision,GiltError,GiltPricingPlan};
use sabiquant::calendar::EndOfMonth;
use sabiquant::settlement::{CashAggregation,SettlementEvidence,SettlementGuarantee,SettlementRounding};
use sabiquant::time::Date;
fn dt(s:&str)->Date{let p:Vec<i32>=s.split('-').map(|x|x.parse().unwrap()).collect();Date::from_ymd(p[0],u32::try_from(p[1]).unwrap(),u32::try_from(p[2]).unwrap()).unwrap()}
fn dec(s:&str)->DecimalAmount{let (c,s)=s.split_once(':').unwrap();DecimalAmount::new(c.parse().unwrap(),s.parse().unwrap()).unwrap()}
fn av(a:DecimalAmount)->String{format!("{}:{}",a.coefficient(),a.scale())}
fn error(e:GiltError)->String{match e{GiltError::CashArithmetic{stage,endpoint,cause,..}=>match cause{
 AmountError::Inexact{operation,..}=>format!("E:{stage}:{endpoint}:{operation}:inexact"),
 AmountError::OutOfRange{operation,negative,..}=>format!("E:{stage}:{endpoint}:{operation}:range:{}",u8::from(negative)),
 _=>panic!("Unexpected cash cause {cause:?}")},_=>panic!("Unexpected error {e:?}")}}
fn decision(d:Result<GiltCashDecision,GiltError>)->String{match d{
 Err(e)=>error(e),Ok(GiltCashDecision::Determined(r))=>{let g=match r.price_evidence_guarantee(){None=>"none",Some(SettlementGuarantee::ForSuppliedDecimals)=>"decimal",Some(SettlementGuarantee::ForSuppliedBinary64)=>"binary",Some(SettlementGuarantee::UniqueForDeclaredEnclosure)=>"declared",_=>panic!("new guarantee")};format!("D:{}:{g}",av(r.amount()))},
 Ok(GiltCashDecision::QuantumAmbiguous{lower_amount,upper_amount,..})=>format!("A:{}:{}",av(lower_amount),av(upper_amount)),
 Ok(GiltCashDecision::ExactStageUnresolved{stage,..})=>format!("U:{stage}"),_=>panic!("new decision")}}
// FR-GILT-01..06/15..21/24: post-implementation independent Fraction audit.
#[test]fn independently_derived_source_cash_extremes(){
 let cases=include_str!("../../cases.txt");let modes=[AmountRounding::Exact,AmountRounding::TowardZero,AmountRounding::AwayFromZero,AmountRounding::TowardPositiveInfinity,AmountRounding::TowardNegativeInfinity,AmountRounding::NearestTiesEven,AmountRounding::NearestTiesAwayFromZero];
 let mut count=0;
 for line in cases.lines(){let p:Vec<&str>=line.split('|').collect();let role:usize=p[2].parse().unwrap();
 let q:Vec<Date>=["1998-12-07","1999-06-07","1999-12-07","2000-06-07","2000-12-07","2001-06-07"].map(dt).to_vec();
 let first=if role==2{2}else{1};let ex=q[first..].iter().map(|d|d.checked_sub_days(7).unwrap()).collect();
 let c=DecimalAmount::new(p[3].parse().unwrap(),p[4].parse().unwrap()).unwrap();
 let g=ConventionalGilt::new(if role==0{q[0]}else{dt("1999-05-07")},q[5],c,q[first],q,ex,EndOfMonth::Ignore,6).unwrap();
 let plan=GiltPricingPlan::new(&g,dt(p[5]),6).unwrap();
 let n=DecimalAmount::new(p[6].parse().unwrap(),p[7].parse().unwrap()).unwrap();
 let rounding=SettlementRounding::new(AmountQuantum::new(p[8].parse().unwrap(),p[9].parse().unwrap()).unwrap(),modes[p[10].parse::<usize>().unwrap()]);
 let e=match p[11]{"d"=>SettlementEvidence::exact_decimal(dec(p[12])),"di"=>SettlementEvidence::declared_decimal_interval(dec(p[12]),dec(p[13])).unwrap(),"b"=>SettlementEvidence::supplied_binary64(f64::from_bits(u64::from_str_radix(p[12],16).unwrap())).unwrap(),"bi"=>SettlementEvidence::declared_binary64_interval(f64::from_bits(u64::from_str_radix(p[12],16).unwrap()),f64::from_bits(u64::from_str_radix(p[13],16).unwrap())).unwrap(),_=>panic!("kind")};
 let expected=p[14];
 let out=match p[1]{
  "dv"=>match g.dividend_per_100(0,rounding.quantum(),rounding.mode()){Ok(a)=>format!("V:{}",av(a)),Err(e)=>error(e)},
  "av"=>match plan.accrued_cash(n,rounding).unwrap().source_value_per_100(rounding.quantum(),rounding.mode()){Ok(Some(a))=>format!("V:{}",av(a)),Err(e)=>error(e),_=>panic!("view")},
  op=>{let i=match op{"div"=>g.dividend_cash(0,n,rounding),"ai"=>plan.accrued_cash(n,rounding),"dirty"=>plan.dirty_cash(n,e,rounding),"total"=>plan.clean_accrued_cash(n,e,CashAggregation::RoundTotal,rounding),"comp"=>plan.clean_accrued_cash(n,e,CashAggregation::RoundComponentsThenSum,rounding),_=>panic!("op")}.unwrap();
    assert_eq!(i.source_version(),g.source_version());assert_eq!(i.coupon_per_100(),c);assert_eq!(i.nominal(),n);assert_eq!(i.cash_rounding(),rounding);
    assert_eq!(i.issue_quasi_dates(),(dt("1998-12-07"),dt("1999-06-07")));
    if matches!(op,"dirty"|"total"|"comp"){assert_eq!(i.price_evidence(),Some(e));}
    let value=decision(i.evaluate());if matches!(op,"div"|"ai")&&value.starts_with("D:"){value.strip_suffix(":none").unwrap().replacen("D:","V:",1)}else{value}
  }
 };assert_eq!(out,expected,"audit row {}: {line}",p[0]);count+=1;
 } assert_eq!(count,__ROWS__);
}
'''


def sha(data):
    return hashlib.sha256(data).hexdigest()


def snapshot():
    paths = [Path(name) for name in (
        "Cargo.toml", "Cargo.lock", "clippy.toml", "README.md", "CHANGELOG.md",
        "LICENSE-MIT", "LICENSE-APACHE",
    )]
    for directory in (ROOT / "src", ROOT / "tests/support"):
        paths.extend(path.relative_to(ROOT) for path in directory.rglob("*") if path.is_file())
    data = {}
    for relative in sorted(set(paths)):
        path = ROOT / relative
        if path.is_symlink() or not path.is_file():
            raise ValueError("snapshot requires regular nonsymlink files: " + str(relative))
        data[str(relative)] = path.read_bytes()
    return data


def manifest(files):
    return [{"path": name, "bytes": len(data), "sha256": sha(data)}
            for name, data in sorted(files.items())]


def run(argv, cwd, env, output, label, evidence):
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, check=False)
    stdout = output / (label + ".stdout")
    stderr = output / (label + ".stderr")
    stdout.write_bytes(result.stdout)
    stderr.write_bytes(result.stderr)
    evidence.setdefault("commands", []).append({
        "label": label, "argv": argv, "exit_code": result.returncode,
        "stdout": {"path": str(stdout), "sha256": sha(result.stdout)},
        "stderr": {"path": str(stderr), "sha256": sha(result.stderr)},
    })
    if result.returncode:
        raise RuntimeError("%s failed with exit %d; inspect %s and %s" %
                           (label, result.returncode, stdout, stderr))
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="new evidence directory outside the repository")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable on PATH or absolute path")
    parser.add_argument("--rustc", default="rustc", help="matching rustc executable")
    args = parser.parse_args()
    output = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix="sabiquant-gilt-source-cash-supplemental-"))
    if output == ROOT or ROOT in output.parents:
        parser.error("evidence output must be outside the repository")
    if args.output:
        output.mkdir(parents=True, exist_ok=False)
    contract = (ROOT / CONTRACT).read_bytes()
    if sha(contract) != CONTRACT_SHA:
        parser.error("frozen adopted contract bytes changed")
    runner = Path(__file__).read_bytes()
    original = snapshot()
    crate = output / "crate"
    crate.mkdir()
    cases = ("\n".join(rows) + "\n").encode()
    (output / "cases.txt").write_bytes(cases)
    rust = RUST.replace("__ROWS__", str(len(rows))).encode()
    evidence = {
        "started_at": datetime.now(timezone.utc).isoformat(),
        "qualification": "Supplemental POST-IMPLEMENTATION source/cash audit, authored after production read; not frozen pre-production evidence or full acceptance.",
        "platform": platform.platform(), "python": sys.version,
        "runner_sha256": sha(runner), "contract_sha256": sha(contract),
        "record_count": len(rows), "operation_census": census,
        "cases_sha256": sha(cases), "generated_test_sha256": sha(rust),
        "source_manifest": manifest(original), "library_source_changes": [],
    }
    failure = None
    try:
        cargo = shutil.which(args.cargo)
        rustc = shutil.which(args.rustc)
        if not cargo or not rustc:
            raise ValueError("cargo/rustc unavailable")
        env = dict(os.environ)
        env["CARGO_TARGET_DIR"] = str(output / "target")
        env["RUSTFLAGS"] = "-D warnings"
        env["RUSTDOCFLAGS"] = "-D warnings"
        evidence["toolchain"] = {
            "cargo": run([cargo, "--version"], ROOT, env, output, "cargo-version", evidence).decode(),
            "rustc": run([rustc, "--version", "--verbose"], ROOT, env, output, "rustc-version", evidence).decode(),
        }
        for relative, data in original.items():
            target = crate / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            if target.read_bytes() != data:
                raise ValueError("copy differs: " + relative)
        evidence["copied_manifest_before_test"] = manifest({name: (crate / name).read_bytes() for name in original})
        if evidence["copied_manifest_before_test"] != evidence["source_manifest"]:
            raise ValueError("copy manifest differs")
        test = crate / "tests/audit.rs"
        test.write_bytes(rust)
        if snapshot() != original:
            raise ValueError("original source changed during snapshot/copy")
        for label, command in (
            ("debug", [cargo, "test", "--offline", "--locked", "--test", "audit"]),
            ("release", [cargo, "test", "--offline", "--locked", "--release", "--test", "audit"]),
            ("clippy", [cargo, "clippy", "--offline", "--locked", "--test", "audit", "--", "-D", "warnings"]),
        ):
            stdout = run(command, crate, env, output, label, evidence)
            if label in ("debug", "release"):
                if b"test result: ok. 1 passed; 0 failed; 0 ignored;" not in stdout:
                    raise ValueError("expected one nonignored test to execute")
        changed = [name for name, data in original.items() if (crate / name).read_bytes() != data]
        if changed or test.read_bytes() != rust:
            raise ValueError("unexpected copied crate mutation: " + repr(changed))
        if Path(__file__).read_bytes() != runner or (ROOT / CONTRACT).read_bytes() != contract:
            raise ValueError("runner/contract changed during execution")
        current = snapshot()
        evidence["original_changed_paths"] = sorted(name for name in set(original) | set(current)
                                                    if original.get(name) != current.get(name))
        if evidence["original_changed_paths"]:
            raise ValueError("original crate source changed during execution")
        evidence["original_files_unchanged"] = True
        evidence["all_records_checked_debug_and_release"] = len(rows)
        evidence["result"] = "passed"
    except Exception as error:
        failure = error
        evidence["result"] = "failed"
        evidence["failure"] = {"type": type(error).__name__, "message": str(error)}
        current = snapshot()
        evidence["original_changed_paths"] = sorted(name for name in set(original) | set(current)
                                                    if original.get(name) != current.get(name))
        evidence["original_files_unchanged"] = not evidence["original_changed_paths"]
    evidence["finished_at"] = datetime.now(timezone.utc).isoformat()
    result = output / "evidence.json"
    result.write_text(json.dumps(evidence, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"result": evidence["result"], "evidence": str(result),
                      "evidence_sha256": sha(result.read_bytes()), "record_count": len(rows)}, sort_keys=True))
    if failure:
        raise SystemExit(str(failure))


if __name__ == "__main__":
    main()
