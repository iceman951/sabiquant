#!/usr/bin/env python3
"""Independent Appendix algebra certificate; exact decimal yield, not IEEE quote."""
from fractions import Fraction as F
from decimal import Decimal, localcontext
import datetime as dt
import hashlib
import json
from pathlib import Path
import argparse

def nth(v,n):
    x=1<<((v.bit_length()+n-1)//n)
    while True:
        z=((n-1)*x+v//x**(n-1))//n
        if z>=x:return x
        x=z

def rat(v):return '%d/%d'%(v.numerator,v.denominator)

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--repo-root',type=Path,default=Path(__file__).resolve().parents[2])
    ap.add_argument('--output',type=Path);ap.add_argument('--check',action='store_true')
    args=ap.parse_args();repo=args.repo_root.resolve()
    y=F(8000001,100000000); b=1+y/2
    radicand=b**(-145); degree=183; unit=-320
    scaled=radicand*2**(320*degree)
    m=nth(scaled.numerator//scaled.denominator,degree)
    lo=F(m,2**320); hi=F(m+1,2**320)
    assert lo**degree<radicand<hi**degree
    annuity=100*b**(-31)+4*sum((b**(-k) for k in range(32)),F(0))
    a=F(152,183); ai=F(304,365)
    lower=lo*annuity-a; upper=hi*annuity-a
    q=['%04d-%02d-01'%(2007+k//2,6 if k%2==0 else 12) for k in range(33)]
    # Alternating arithmetic above ends2023-06-01 and starts2007-06-01.
    assert q[-1]=='2023-06-01'
    inputs={'D':'2007-06-01','I':'2007-06-01','M':'2023-06-01','T':'2023-06-01',
        'S':'2007-07-09','C':'8/1','f':2,'EOM':'Ignore','Q':q,'coupons':q[1:],
        'policy':'CanonicalIiac13LocalV1','selection':'Explicit(Compound)',
        'operation':'Appendix_exact_rational_y_ideal_clean','yield':rat(y),
        'date_cap':33,'bundle_cap':32}
    ib=(json.dumps(inputs,sort_keys=True,separators=(',',':'))+'\n').encode()
    with localcontext() as c:
        c.prec=90
        preview=[str(Decimal(v.numerator)/Decimal(v.denominator)) for v in (lower,upper)]
    source=(repo/'docs/M8_CANADIAN_CONTRACT.md').read_bytes()
    out={'qualification':'exact source-example decimal yield; NOT binary64 quote0.08000001 and NOT a runtime result',
        'contract_sha256':hashlib.sha256(source).hexdigest(),
        'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'reproduction':'python3 validation/generator/canadian_appendix_certificate.py --check',
        'inputs':inputs,'input_sha256':hashlib.sha256(ib).hexdigest(),
        'source_A':rat(a),'source_AI':rat(ai),'base':rat(b),'root_radicand':rat(radicand),
        'root_degree':degree,'root_unit':unit,'root_lower_m':str(m),'root_upper_m':str(m+1),
        'root_strict_integer_power_certificates':True,'annuity':rat(annuity),
        'clean_lower':rat(lower),'clean_upper':rat(upper),'preview_only':preview}
    assert out['contract_sha256']=='4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3'
    p=args.output or repo/'validation/fixtures/bonds/canadian-appendix-declared-price.json'
    data=(json.dumps(out,sort_keys=True,indent=2)+'\n').encode()
    if args.check:assert p.is_file() and not p.is_symlink() and p.read_bytes()==data
    else:
        assert not p.is_symlink()
        with p.open('xb') as f:f.write(data)
    print(hashlib.sha256(p.read_bytes()).hexdigest(),out['input_sha256'],preview)

if __name__=='__main__':main()
