#!/usr/bin/env python3
"""Private independent ideal CAD references; stdlib, exact certificates first."""
import argparse
import calendar
import datetime as dt
from decimal import Decimal, localcontext
from fractions import Fraction as F
import hashlib
import json
import math
from pathlib import Path
import platform
import re
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
CONTRACT_SHA = '4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3'
P = 224
U = F(1, 2**53)
HALF = F(1, 2**1075)
MAX = F((2**53-1)*2**971)
QTY = ('model_dirty','clean','settlement_dirty','price_derivative',
       'price_second_derivative','time_weighted_amount','modified_duration',
       'macaulay_duration','convexity')
COMMON = ('accrual_start','issue','stated_maturity','true_maturity',
          'coupon_coefficient','coupon_scale','frequency','eom','quasi_dates',
          'coupon_dates','settlement','source_policy','selection','date_cap','bundle_cap')
PRICE_INPUT = COMMON + ('operation','yield_bits',)
INV_INPUT = COMMON + ('operation','target_basis','target_bits','bracket_lower_bits',
    'bracket_upper_bits','absolute_tolerance_bits','relative_tolerance_bits','evaluation_budget')
PRICE_COLS = ('id','profile') + PRICE_INPUT + ('convention','geometry','original_ordinals',
    'lambdas','source_a','source_ai','source_coefficients','ideal_image_floor',
    'observation_class','certificate_ids') + tuple(
        q+s for q in QTY for s in ('_lo','_hi','_error','_class','_informative'))
INV_COLS = ('id','profile') + INV_INPUT + ('convention','geometry','reference_equation',
    'exact_model_target','reference_model_target','cached_model_target_bits','target_shift_error','target_shift_included',
    'ideal_image_floor','ideal_root_class','root_lo','root_hi','lower_price_lo',
    'lower_price_hi','upper_price_lo','upper_price_hi','slope_lower',
    'price_error','residual_to_coordinate','residual_budget','coordinate_error','informative','observed_protocol',
    'function_calls','derivative_calls','certificate_ids')
ROOTS = {}
LOGS = {}
CERTS = {}
STATS = {'decimal_checks':0,'decimal_precision_counts':{},'root_sign_certificates':0,'mm_h_identities':0}
SHAPES=('regular','short_first','long_first','short_last','long_last','both_ends',
        'short_both','long_first_short_last','short_first_long_last','composite','same_cell','composite_exact_one')

def js(x):
    return json.dumps(x, sort_keys=True, separators=(',',':'), ensure_ascii=True)

def fnv(b):
    h = 0xcbf29ce484222325
    for v in b:
        h = ((h ^ v)*0x100000001b3) & ((1<<64)-1)
    return '%016x' % h

def pin(b):
    return {'sha256':hashlib.sha256(b).hexdigest(),'fnv64':fnv(b),'bytes':len(b)}

def rational(x):
    return '%d/%d' % (x.numerator,x.denominator)

def b2f(b):
    x = struct.unpack('>d', bytes.fromhex(b))[0]
    if not math.isfinite(x):
        raise ValueError('finite quote required')
    return F(*x.as_integer_ratio())

def bits(x):
    return struct.pack('>d',x).hex()

def p2(e):
    return F(2**e) if e>=0 else F(1,2**(-e))

def log2floor(x):
    assert x>0
    e=x.numerator.bit_length()-x.denominator.bit_length()
    if x<p2(e): e-=1
    return e

def outward(x, up):
    if not x: return F(0)
    unit=p2(log2floor(abs(x))-(P-1))
    q=x//unit
    if up and x!=q*unit: q+=1
    return q*unit

def dyad(x):
    assert x.denominator & (x.denominator-1)==0
    if not x: return '0@0'
    n=x.numerator; e=-(x.denominator.bit_length()-1)
    while n%2==0: n//=2; e+=1
    return '%d@%d' % (n,e)

class I:
    __slots__=('lo','hi')
    def __init__(self,lo,hi=None):
        self.lo=outward(F(lo),False)
        self.hi=outward(F(lo if hi is None else hi),True)
        assert self.lo<=self.hi
    def __add__(self,b):
        b=interval(b); return I(self.lo+b.lo,self.hi+b.hi)
    __radd__=__add__
    def __neg__(self): return I(-self.hi,-self.lo)
    def __sub__(self,b): return self+-interval(b)
    def __rsub__(self,b): return interval(b)+-self
    def __mul__(self,b):
        b=interval(b); xs=[self.lo*b.lo,self.lo*b.hi,self.hi*b.lo,self.hi*b.hi]
        return I(min(xs),max(xs))
    __rmul__=__mul__
    def __truediv__(self,b):
        b=interval(b); assert not b.lo<=0<=b.hi
        return self*I(1/b.hi,1/b.lo)
    def mag(self): return max(abs(self.lo),abs(self.hi))
    def encode(self): return [dyad(self.lo),dyad(self.hi)]

def interval(a): return a if isinstance(a,I) else I(a)

def power(a,k):
    assert k>=0
    r=I(1)
    while k:
        if k&1: r=r*a
        k//=2
        if k: a=a*a
    return r

def nthroot_floor(v,n):
    if n==1: return v
    x=1<<((v.bit_length()+n-1)//n)
    while True:
        z=((n-1)*x+v//pow(x,n-1))//n
        if z>=x: return x
        x=z

def root(b,n):
    key=(b,n)
    if key in ROOTS: return ROOTS[key]
    if n==1: return I(b),None
    e=log2floor(b)//n
    unit=e-(P-1)
    scaled=b/p2(unit*n)
    v=scaled.numerator//scaled.denominator
    m=nthroot_floor(v,n)
    assert m**n*scaled.denominator<=scaled.numerator
    exact=m**n*scaled.denominator==scaled.numerator
    mh=m if exact else m+1
    assert mh**n*scaled.denominator>=scaled.numerator
    rec={'base':rational(b),'degree':n,'unit':unit,'lower_m':str(m),
         'upper_m':str(mh),'exact':exact,'lower_power_le':True,'upper_power_ge':True}
    cid=hashlib.sha256(js(rec).encode()).hexdigest()[:24]
    CERTS[cid]=rec
    ans=(I(m*p2(unit),mh*p2(unit)),cid)
    ROOTS[key]=ans
    return ans

def log_interval(x):
    if x in LOGS: return LOGS[x]
    assert x>0
    k=log2floor(x); m=x/p2(k)
    def atanh_log(a):
        t=(a-1)/(a+1); t2=t*t; term=t; total=I(0); j=1
        while True:
            total=total+I(2*term/j)
            term*=t2; j+=2
            tail=2*abs(term)/(j*(1-t2))
            if tail<p2(-P-20):
                return total+I(-tail,tail)
    lm=atanh_log(m)
    if k:
        if F(2) not in LOGS:
            LOGS[F(2)]=atanh_log(F(2))
        lm=lm+k*LOGS[F(2)]
    LOGS[x]=lm
    return lm

def round_ieee(x):
    # Independent rational nearest/even, including direct subnormal conversion.
    if not x: return F(0)
    s=-1 if x<0 else 1; x=abs(x)
    e=log2floor(x); unit=p2(max(e-52,-1074))
    n,d=(x/unit).as_integer_ratio(); q,r=divmod(n,d)
    if 2*r>d or (2*r==d and q&1): q+=1
    z=q*unit
    if z>MAX: raise OverflowError('IEEE overflow')
    return s*z

def rne_bits(x):
    z=round_ieee(x)
    return bits(float(z))

def monthdate(anchor, offset, eom):
    ix=anchor.year*12+anchor.month-1+offset; y,m=divmod(ix,12); m+=1
    last=calendar.monthrange(y,m)[1]
    day=last if eom=='Preserve' and anchor.day==calendar.monthrange(anchor.year,anchor.month)[1] else min(anchor.day,last)
    return dt.date(y,m,day)

def fragments(q,a,b):
    assert q[0]<=a<=b<=q[-1]
    out=[]
    for x,z in zip(q,q[1:]):
        d=(min(b,z)-max(a,x)).days
        if d>0: out.append((d,(z-x).days))
    assert sum(d for d,s in out)==(b-a).days
    assert sum(d<s for d,s in out)<=2
    return out

def valuation(c,f,pieces): return c/f*sum((F(d,s) for d,s in pieces),F(0))

def payable(c,f,pieces):
    r=F(0)
    for d,s in pieces:
        if d==s: r+=c/f
        elif f*d<365: r+=c*d/365
        elif f*d>365: r+=c*(F(1,f)-F(s-d,365))
        else: raise AssertionError('impossible proper-branch equality')
    return r

def build_state(f,eom,shape,selection='Compound',coupon=(525,2),settled=False):
    if shape=='composite_exact_one':
        anchor=dt.date(2024,12,31) if f==4 else dt.date(2024,8,31)
        allq=[monthdate(anchor,-(3-i)*(12//f),eom) for i in range(4)]
        # Keep the original final anchor when selecting a valid sublattice.
        for j in (1,):
            a=(allq[j+1]-allq[j]).days; b=(allq[j+2]-allq[j+1]).days
            g=math.gcd(a,b)
            if g>1: break
        assert g>1
        q=allq[j:j+3]; d=q[0]+dt.timedelta(days=a//g); m=q[1]+dt.timedelta(days=b//g)
        cs=[m]; s=d+dt.timedelta(days=1)
        assert sum((F(x,z) for x,z in fragments(q,d,m)),F(0))==1
        return dict(accrual_start=d.isoformat(),issue=d.isoformat(),stated_maturity=m.isoformat(),
            true_maturity=m.isoformat(),coupon_coefficient=str(coupon[0]),coupon_scale=str(coupon[1]),
            frequency=str(f),eom=eom,quasi_dates=';'.join(x.isoformat() for x in q),
            coupon_dates=m.isoformat(),settlement=s.isoformat(),source_policy='CanonicalIiac13LocalV1',
            selection=('Explicit('+selection+')' if selection!='LastCouponMoneyMarket' else selection),
            date_cap='3',bundle_cap='1',geometry=shape)
    anchor=dt.date(2028,6,30)
    allq=[monthdate(anchor,-(8-i)*(12//f),eom) for i in range(9)]
    q=allq[:7]; d=q[0]; m=q[-1]
    coupons=q[1:]
    if shape=='short_first': d+=dt.timedelta(days=7)
    elif shape=='long_first': d+=dt.timedelta(days=7); coupons=q[3:]
    elif shape=='short_last': m-=dt.timedelta(days=7); coupons=q[1:-1]+[m]
    elif shape=='long_last': m-=dt.timedelta(days=7); coupons=q[1:4]+[m]
    elif shape=='both_ends':
        d+=dt.timedelta(days=7); m-=dt.timedelta(days=7); coupons=q[2:5]+[m]
    elif shape=='short_both':
        d+=dt.timedelta(days=7); m-=dt.timedelta(days=7); coupons=q[1:6]+[m]
    elif shape=='long_first_short_last':
        d+=dt.timedelta(days=7); m-=dt.timedelta(days=7); coupons=q[3:6]+[m]
    elif shape=='short_first_long_last':
        d+=dt.timedelta(days=7); m-=dt.timedelta(days=7); coupons=q[1:4]+[m]
    elif shape=='composite':
        d=q[4]+dt.timedelta(days=3); m=q[6]-dt.timedelta(days=5); q=q[4:]; coupons=[m]
    elif shape=='same_cell':
        d=q[5]+dt.timedelta(days=3); m=q[6]-dt.timedelta(days=4); q=q[5:]; coupons=[m]
    elif shape!='regular': raise ValueError(shape)
    first=coupons[0]
    s=d+dt.timedelta(days=min(17,max(1,(first-d).days//3)))
    if settled and len(coupons)>=3: s=coupons[1]+dt.timedelta(days=1)
    assert d<=s<m
    t=m+dt.timedelta(days=3 if shape in ('long_last','composite') else 0)
    return dict(accrual_start=d.isoformat(),issue=d.isoformat(),stated_maturity=m.isoformat(),
        true_maturity=t.isoformat(),coupon_coefficient=str(coupon[0]),coupon_scale=str(coupon[1]),
        frequency=str(f),eom=eom,quasi_dates=';'.join(x.isoformat() for x in q),
        coupon_dates=';'.join(x.isoformat() for x in coupons),settlement=s.isoformat(),
        source_policy='CanonicalIiac13LocalV1',selection=('Explicit('+selection+')' if selection!='LastCouponMoneyMarket' else selection),date_cap=str(len(q)),
        bundle_cap=str(sum(x>s for x in coupons)),geometry=shape)

def prepare(s):
    date=dt.date.fromisoformat
    q=[date(x) for x in s['quasi_dates'].split(';')]
    cs=[date(x) for x in s['coupon_dates'].split(';')]
    d=date(s['accrual_start']); settle=date(s['settlement']); t=date(s['true_maturity'])
    f=int(s['frequency']); c=F(int(s['coupon_coefficient']),10**int(s['coupon_scale']))
    assert q[0]<=d<q[1] and q[-2]<cs[-1]<=q[-1]
    assert cs[-1]==date(s['stated_maturity']) and d<=date(s['issue'])<cs[0]
    assert date(s['issue'])<=settle<cs[-1] and t>=cs[-1]
    for i,x in enumerate(q):
        assert x==monthdate(q[-1],-(len(q)-1-i)*(12//f),s['eom'])
    assert all(x<y for x,y in zip(cs,cs[1:]))
    assert all(x in q for x in cs[:-1])
    assert all(q.index(y)==q.index(x)+1 for x,y in zip(cs[:-2],cs[1:-1]))
    begin=d; rows=[]; last=d
    for i,end in enumerate(cs):
        parts=fragments(q,begin,end)
        if end<=settle: last=end
        else:
            edges=fragments(q,settle,end)
            lam=sum((F(a,b) for a,b in edges),F(0))
            assert F(1,184)<=lam<20000
            rows.append({'ordinal':i,'valuation':valuation(c,f,parts)+(100 if end==cs[-1] else 0),
                'payable':payable(c,f,parts)+(100 if end==cs[-1] else 0),
                'lambda':lam,'edges':edges,'paydate':t if end==cs[-1] else end})
        begin=end
    a=valuation(c,f,fragments(q,last,settle))
    ai=a if f==12 else payable(c,f,fragments(q,last,settle))
    conv='MoneyMarket' if s['selection']=='Explicit(MoneyMarket)' or (s['selection']=='LastCouponMoneyMarket' and len(rows)==1) else 'Compound'
    tau=F((t-settle).days,365)
    for r in rows:
        r['rho']=F((t-r['paydate']).days,365); r['h']=tau-r['rho']
        assert r['h']>0
    return {'f':f,'c':c,'a':a,'ai':ai,'rows':rows,'convention':conv,'tau':tau}

def factor(b,edges):
    # Reciprocal base first. Root degrees remain individual cell day lengths.
    v=I(1); ids=[]
    for d,s in edges:
        if d==s: v=v*I(1/b)
        else:
            rt,cid=root(1/b,s)
            v=v*power(rt,d)
            if cid: ids.append(cid)
    return v,ids

def ideal(p,y):
    f=p['f']; rows=p['rows']; ids=[]; weights=[]
    if p['convention']=='Compound':
        b=1+y/f
        if b<=0: raise ValueError('compound-domain')
        for r in rows:
            v,cids=factor(b,r['edges']); ids+=cids
            weights.append(v*r['valuation'])
        dirty=sum(weights,I(0)); delta=f+y
        d1=-sum((w*r['lambda']/delta for w,r in zip(weights,rows)),I(0))
        d2=sum((w*r['lambda']*(r['lambda']+1)/(delta*delta) for w,r in zip(weights,rows)),I(0))
        mom=sum((w*r['lambda']/f for w,r in zip(weights,rows)),I(0))
        clean=dirty-p['a']; settlement=dirty+(p['ai']-p['a']); floor=F(0)
    else:
        d=1+y*p['tau']; ns=[1+y*r['rho'] for r in rows]
        if d<=0 or any(n<=0 for n in ns): raise ValueError('mm-domain')
        weights=[I(r['payable']*n/d) for r,n in zip(rows,ns)]
        dirty=sum(weights,I(0)); h=sum((r['payable']*r['h'] for r in rows),F(0))
        assert h>0
        d1=I(-h/(d*d)); d2=I(2*p['tau']*h/(d*d*d))
        # Independent H versus per-flow identity verifies the algebraic route.
        assert sum((r['payable']*n/d*r['h']/(d*n) for r,n in zip(rows,ns)),F(0))==h/d**2
        STATS['mm_h_identities']+=1
        mom=sum((w*r['h'] for w,r in zip(weights,rows)),I(0))
        clean=dirty-p['ai']; settlement=dirty
        floor=sum((r['payable']*r['rho']/p['tau'] for r in rows),F(0))
    assert dirty.lo>0
    vals=[dirty,clean,settlement,d1,d2,mom,-d1/dirty,mom/dirty,d2/dirty]
    return vals,weights,sorted(set(ids)),floor

class Jet:
    """Independent Decimal automatic differentiation through scalar operations."""
    __slots__=('v','d1','d2')
    def __init__(self,v,d1=0,d2=0): self.v=Decimal(v); self.d1=Decimal(d1); self.d2=Decimal(d2)
    def __add__(self,b):
        b=b if isinstance(b,Jet) else Jet(b)
        return Jet(self.v+b.v,self.d1+b.d1,self.d2+b.d2)
    __radd__=__add__
    def __neg__(self): return Jet(-self.v,-self.d1,-self.d2)
    def __sub__(self,b): return self+-b
    def __mul__(self,b):
        b=b if isinstance(b,Jet) else Jet(b)
        return Jet(self.v*b.v,self.d1*b.v+self.v*b.d1,
                   self.d2*b.v+2*self.d1*b.d1+self.v*b.d2)
    __rmul__=__mul__
    def __pow__(self,a):
        a=Decimal(a)
        v=self.v**a
        return Jet(v,a*v*self.d1/self.v,
                   a*v*(self.d2/self.v+(a-1)*(self.d1/self.v)**2))
    def __truediv__(self,b):
        return self*(b**Decimal(-1)) if isinstance(b,Jet) else self*Jet(b)**Decimal(-1)

def decimal_values(p,y,prec):
    with localcontext() as ctx:
        ctx.prec=prec; ctx.Emax=999999999; ctx.Emin=-999999999
        cv=lambda x:Decimal(x.numerator)/Decimal(x.denominator)
        yy=Jet(cv(y),1); f=Decimal(p['f']); rows=p['rows']
        ws=[]
        for r in rows:
            if p['convention']=='Compound':
                w=cv(r['valuation'])*((1+yy/f)**(-cv(r['lambda'])))
            else: w=cv(r['payable'])*(1+yy*cv(r['rho']))/(1+yy*cv(p['tau']))
            ws.append(w)
        total=sum(ws,Jet(0)); dirty=total.v; d1=total.d1; d2=total.d2
        if p['convention']=='Compound':
            mom=sum((w.v*cv(r['lambda'])/f for w,r in zip(ws,rows)),Decimal(0))
            clean=dirty-cv(p['a']); settlement=dirty-cv(p['a'])+cv(p['ai'])
        else:
            mom=sum((w.v*cv(r['h']) for w,r in zip(ws,rows)),Decimal(0))
            clean=dirty-cv(p['ai']); settlement=dirty
        return [dirty,clean,settlement,d1,d2,mom,-d1/dirty,mom/dirty,d2/dirty]

def corroborate(p,y,vals):
    for precision in ((768,1024) if abs(y)>p2(100) else (100,160)):
        ds=decimal_values(p,y,precision)
        for v,d in zip(vals,ds):
            z=F(d)
            # Independent enclosure vastly wider than Decimal's final rounding.
            # Exact roots/rational arithmetic certify; this is corroboration only.
            slack=max(abs(z),v.mag(),p2(-8192))*p2(-260)
            assert v.lo-slack<=z<=v.hi+slack, (precision,rational(z),v.encode())
            STATS['decimal_checks']+=1
            k=str(precision)
            STATS['decimal_precision_counts'][k]=STATS['decimal_precision_counts'].get(k,0)+1

def budget(p,y,vals,weights):
    rows=p['rows']; f=p['f']; derr=[]; coeffs=[]; admission='value_under_profile'
    if p['convention']=='Compound':
        b=1+y/f; ln=log_interval(b)
        beta=(4 if f in (2,4) else 8)*U
        for r,w in zip(rows,weights):
            disc,_=factor(b,r['edges']); E=-ln*r['lambda']
            if y==0: R=F(0)
            elif abs(y)<p2(-50):
                q=-y*r['lambda']/f
                R=abs(round_ieee(q)-q)+r['lambda']*y*y/(2*f*f*(1-abs(y)/f))
            else: R=beta*E.mag()+HALF
            # exp(R)<=1/(1-R), rigorously for 0<=R<1.
            if R>=F(1,2): return None,'profile_radius_uninformative',None
            eta=(1+2*U)/(1-R)-1
            ferr=eta*disc.hi+HALF
            if disc.lo-ferr<=0 or disc.hi+ferr>=MAX:
                admission='factor_range_not_stable'
            # Guard union contains every allowed rounded exponent and its R(z).
            zrad=R
            guardrad=(beta*(E.mag()+zrad)+HALF)/(1-beta) if abs(y)>=p2(-50) else R
            z0=-1075*log_interval(F(2)); zi=log_interval(F(2**1024-2**970))
            glo=E.lo-zrad-guardrad; ghi=E.hi+zrad+guardrad
            if not (glo>z0.hi and ghi<zi.lo): admission='factor_range_not_stable'
            B=r['valuation']; pvpre=B*ferr
            if w.hi+pvpre>=MAX: admission='pv_range_not_stable'
            derr.append(pvpre+U*(w.hi+pvpre)+HALF)
        delta=f+y; dh=round_ieee(delta)
        for r in rows:
            l=r['lambda']
            coeffs.append(((l/delta,l/dh),(l*(l+1)/delta**2,l*(l+1)/dh**2),(l/f,l/f)))
    else:
        d=1+y*p['tau']
        for r,w in zip(rows,weights):
            n=1+y*r['rho']; fac=n/d
            ferr=U*fac+HALF
            derr.append(r['payable']*ferr+U*(w.hi+r['payable']*ferr)+HALF)
            coeffs.append(((r['h']/(d*n),r['h']/(d*n)),
                (2*p['tau']*r['h']/(d*d*n),2*p['tau']*r['h']/(d*d*n)),(r['h'],r['h'])))
    dpre=sum(derr,F(0)); dirtyerr=dpre+U*(vals[0].hi+dpre)+HALF
    if vals[0].hi+dpre>=MAX: admission='dirty_range_not_stable'
    ae=abs(round_ieee(p['a'])-p['a']); aie=abs(round_ieee(p['ai'])-p['ai'])
    qe=ae if p['convention']=='Compound' else aie
    cleanpre=dirtyerr+qe
    cleanerr=cleanpre+U*(vals[1].mag()+cleanpre)+HALF
    if p['convention']=='MoneyMarket': seterr=dirtyerr
    else:
        observed_shift=round_ieee(p['ai'])-round_ieee(p['a'])
        source_shift=p['ai']-p['a']
        sp=dirtyerr+abs(observed_shift-source_shift)
        seterr=sp if observed_shift==0 else sp+U*(vals[2].mag()+sp)+HALF
    errors=[dirtyerr,cleanerr,seterr]; retained=[]
    classes=['value','value','value']
    for k,v in enumerate(vals[3:6]):
        err=F(0)
        for r,w,we,cs in zip(rows,weights,derr,coeffs):
            exact,observed=cs[k]
            pre=we*observed+w.hi*abs(observed-exact)
            err+=pre+U*(w.hi+we)*observed
        retained.append(err)
        errors.append(err+U*(v.mag()+err)+HALF)
        classes.append(rounded_class(v,err))
    if vals[0].lo<=dirtyerr:
        return None,'dirty_denominator_uninformative',None
    for v,pre in zip(vals[6:],(retained[0],retained[2],retained[1])):
        e=(pre+v.mag()*dirtyerr)/(vals[0].lo-dirtyerr)
        errors.append(e+U*(v.mag()+e)+HALF)
        classes.append(rounded_class(v,e))
    return [outward(e,True) for e in errors],admission,classes

def rounded_class(v,preerror):
    alo=0 if v.lo<=0<=v.hi else min(abs(v.lo),abs(v.hi))
    low=max(F(0),alo-preerror); high=v.mag()+preerror
    midpoint=F(2**1024-2**970)
    if high<=HALF and alo>0:
        return 'underflow_negative_zero' if v.hi<0 else 'underflow_positive_zero'
    if low>=midpoint: return 'overflow_negative_infinity' if v.hi<0 else 'overflow_positive_infinity'
    if low>HALF and high<midpoint: return 'value'
    if high<midpoint: return 'bounded_value_or_underflow'
    return 'bounded_value_or_range_error'

def is_informative(v,e,status,kind):
    return status=='value_under_profile' and kind=='value' and e is not None and v.mag()>0 and e<=v.mag()*F(1,10**8)

def base_record(s): return {k:s[k] for k in COMMON}

def price_record(s,yb,profile):
    p=prepare(s); y=b2f(yb); vals,ws,ids,floor=ideal(p,y)
    corroborate(p,y,vals)
    errors,status,classes=budget(p,y,vals,ws)
    rec=base_record(s)
    rec.update(profile=profile,operation='evaluate_and_six_risk_getters',yield_bits=yb,convention=p['convention'],geometry=s['geometry'],
        original_ordinals=';'.join(str(r['ordinal']) for r in p['rows']),
        lambdas=';'.join(rational(r['lambda']) for r in p['rows']),source_a=rational(p['a']),
        source_ai=rational(p['ai']),source_coefficients=';'.join(rational(r['valuation'] if p['convention']=='Compound' else r['payable']) for r in p['rows']),
        ideal_image_floor=rational(floor),observation_class=status,certificate_ids=';'.join(ids))
    for j,(name,val) in enumerate(zip(QTY,vals)):
        rec[name+'_lo']=dyad(val.lo); rec[name+'_hi']=dyad(val.hi)
        rec[name+'_error']=dyad(errors[j]) if errors is not None else 'not_certified'
        kind=classes[j] if classes is not None and status=='value_under_profile' else 'ideal_only_unstable_factor_admission'
        rec[name+'_class']=kind
        rec[name+'_informative']=str(is_informative(val,None if errors is None else errors[j],status,kind)).lower()
    return rec

def ordinary_states():
    for f in (2,4,12):
        for eom in ('Preserve','Ignore'):
            for sh in SHAPES:
                for sel in ('Compound','MoneyMarket','LastCouponMoneyMarket'):
                    yield build_state(f,eom,sh,sel,settled=(sh=='regular'))

def appendix_state():
    anchor=dt.date(2023,6,1)
    q=[monthdate(anchor,-6*(32-i),'Ignore').isoformat() for i in range(33)]
    return dict(accrual_start=q[0],issue=q[0],stated_maturity=q[-1],true_maturity=q[-1],
        coupon_coefficient='8',coupon_scale='0',frequency='2',eom='Ignore',quasi_dates=';'.join(q),
        coupon_dates=';'.join(q[1:]),settlement='2007-07-09',source_policy='CanonicalIiac13LocalV1',
        selection='Explicit(Compound)',date_cap='33',bundle_cap='32',geometry='regular')

def price_rows():
    out=[]
    for s in ordinary_states():
        for y in (-0.03,0.0,0.04):
            yb=bits(y)
            if y==0 and s['eom']=='Ignore': yb='8000000000000000'
            out.append(price_record(s,yb,'ordinary'))
    for f in (2,4,12):
        for sel in ('Compound','MoneyMarket','LastCouponMoneyMarket'):
            for shape in SHAPES:
                out.append(price_record(build_state(f,'Ignore',shape,sel,(0,0)),bits(.04),'zero_coupon'))
            for c in ((0,0),(1,38),(2**127-1,0),(2**127-1,38)):
                s=build_state(f,'Preserve','composite',sel,c)
                s['date_cap']='20000'; s['bundle_cap']='20000'
                out.append(price_record(s,bits(0.05),'source_extreme'))
            s=build_state(f,'Ignore','same_cell',sel)
            for y in (p2(-1074),-p2(-1074),p2(-51),-p2(-51),F(100)):
                out.append(price_record(s,bits(float(y)),'yield_branch'))
            if sel!='MoneyMarket':
                near=math.nextafter(-float(f),math.inf)
                out.append(price_record(s,bits(near),'floor_extreme'))
            out.append(price_record(s,'7fefffffffffffff','maximum_yield'))
            # Accrual resets/zero A and exact coupon-date cessation, not date aliases.
            for position in ('dated','coupon'):
                s=build_state(f,'Ignore','regular',sel)
                s['settlement']=s['accrual_start'] if position=='dated' else s['coupon_dates'].split(';')[1]
                s['bundle_cap']=str(sum(x>s['settlement'] for x in s['coupon_dates'].split(';')))
                out.append(price_record(s,bits(.025),'settlement_boundary'))
        # Signed valid prices with normal compound factors; true monthly AI=A retained.
        start=dt.date(2025,9,1) if f==2 else dt.date(2025,12,1) if f==4 else dt.date(2026,2,1)
        maturity=dt.date(2026,3,1); settle=start+dt.timedelta(days=45 if f==2 else 20 if f==4 else 10)
        s=dict(accrual_start=start.isoformat(),issue=start.isoformat(),stated_maturity=maturity.isoformat(),
            true_maturity=maturity.isoformat(),coupon_coefficient='5',coupon_scale='0',frequency=str(f),eom='Ignore',
            quasi_dates=start.isoformat()+';'+maturity.isoformat(),coupon_dates=maturity.isoformat(),
            settlement=settle.isoformat(),source_policy='CanonicalIiac13LocalV1',selection='Explicit(Compound)',
            date_cap='2',bundle_cap='1',geometry='regular')
        out.append(price_record(s,bits(2.**500),'signed_price'))
        # Date-coupled raw-D2-normal / normalized-convexity-underflow controls.
        q0=dt.date(2025,3,1); q1=monthdate(q0,12//f,'Ignore'); q2=monthdate(q0,2*(12//f),'Ignore')
        s=dict(accrual_start=q0.isoformat(),issue=q0.isoformat(),stated_maturity=q2.isoformat(),
            true_maturity=q2.isoformat(),coupon_coefficient=str(2**127-1),coupon_scale='0',frequency=str(f),eom='Ignore',
            quasi_dates=';'.join(x.isoformat() for x in (q0,q1,q2)),coupon_dates=q1.isoformat()+';'+q2.isoformat(),
            settlement=(q1-dt.timedelta(days=1)).isoformat(),source_policy='CanonicalIiac13LocalV1',
            selection='Explicit(Compound)',date_cap='3',bundle_cap='2',geometry='regular')
        out.append(price_record(s,bits(2.**540),'raw_normal_normalized_underflow'))
        s=build_state(f,'Preserve','regular','MoneyMarket',(2**127-1,0))
        out.append(price_record(s,bits(2.**370),'raw_normal_normalized_underflow'))
    # Maximal calendar delay and a nonzero subnormal dirty value remain admitted MM.
    s=dict(accrual_start='0001-01-01',issue='0001-01-01',stated_maturity='0001-07-01',
        true_maturity='9999-12-31',coupon_coefficient='0',coupon_scale='0',frequency='2',eom='Ignore',
        quasi_dates='0001-01-01;0001-07-01',coupon_dates='0001-07-01',settlement='0001-01-01',
        source_policy='CanonicalIiac13LocalV1',selection='Explicit(MoneyMarket)',date_cap='2',bundle_cap='1',geometry='regular')
    out.append(price_record(s,'7fefffffffffffff','long_delay_subnormal_dirty'))
    out.append(price_record(appendix_state(),bits(.08000001),'appendix_binary_quote'))
    for i,r in enumerate(out): r['id']='CAD-IDEAL-%04d' % (i+1)
    return out

def exact_target(p,basis,target):
    if basis=='ModelDirty': return target
    q=p['a'] if p['convention']=='Compound' else p['ai']
    return target+q if basis=='Clean' else target-p['ai']+q

def cached_target(p,basis,target):
    if basis=='ModelDirty': return target
    q=round_ieee(p['a'] if p['convention']=='Compound' else p['ai'])
    return round_ieee(target+q if basis=='Clean' else target-round_ieee(p['ai'])+q)

def uniform_dirty_error(p,l,h,vl,wl,vh):
    """All quotes in supplied bracket, not only ideal-root endpoint observations."""
    errors=[]; f=p['f']
    if p['convention']=='Compound':
        beta=(4 if f in (2,4) else 8)*U
        ml=max(log_interval(1+l/f).mag(),log_interval(1+h/f).mag())
        for r,w in zip(p['rows'],wl):
            # Includes every tiny-branch analytic remainder, as an absolute bound.
            tiny=r['lambda']*p2(-100)/(2*f*f*(1-p2(-50)/f))
            R=beta*r['lambda']*ml+HALF+tiny
            if R>=F(1,2): return None
            eta=(1+2*U)/(1-R)-1
            disc,_=factor(1+l/f,r['edges'])
            fe=eta*disc.hi+HALF
            B=r['valuation']; pre=B*fe
            if w.hi+pre>=MAX: return None
            errors.append(pre+U*(w.hi+pre)+HALF)
    else:
        for r,w in zip(p['rows'],wl):
            # Every MM factor is decreasing because h=tau-rho>0.
            fac=(1+l*r['rho'])/(1+l*p['tau'])
            fe=U*fac+HALF; pre=r['payable']*fe
            errors.append(pre+U*(w.hi+pre)+HALF)
    pre=sum(errors,F(0))
    if vl[0].hi+pre>=MAX: return None
    return pre+U*(vl[0].hi+pre)+HALF

def inverse_record(s,basis,targ,lo,hi,profile='ordinary_target',equation='external_exact_source'):
    p=prepare(s); target=b2f(targ); exact=exact_target(p,basis,target)
    cached=cached_target(p,basis,target)
    ref=exact if equation=='external_exact_source' else cached
    l=b2f(lo); h=b2f(hi)
    v0,_,_,floor=ideal(p,l)
    rec=base_record(s)
    rec.update(profile=profile,operation='yield_from_price',target_basis=basis,target_bits=targ,bracket_lower_bits=lo,
        bracket_upper_bits=hi,absolute_tolerance_bits=bits(1e-12),relative_tolerance_bits=bits(0.),
        evaluation_budget='128',convention=p['convention'],geometry=s['geometry'],
        reference_equation=equation,exact_model_target=rational(exact),reference_model_target=rational(ref),
        cached_model_target_bits=rne_bits(cached),target_shift_error=dyad(outward(abs(cached-exact),True)),
        target_shift_included=str(equation=='external_exact_source').lower(),
        ideal_image_floor=rational(floor),ideal_root_class='',root_lo='',root_hi='',
        lower_price_lo='',lower_price_hi='',upper_price_lo='',upper_price_hi='',
        slope_lower='',price_error='',residual_to_coordinate='',residual_budget='',coordinate_error='',informative='false',
        observed_protocol='',function_calls='',derivative_calls='',certificate_ids='')
    if ref<=0:
        rec['ideal_root_class']='nonpositive_exact_resolved_target'
        return rec
    if p['convention']=='MoneyMarket' and ref<=floor:
        rec['ideal_root_class']='outside_open_ideal_image'
        return rec
    vh,_,_,_=ideal(p,h)
    if not (v0[0].lo>ref and vh[0].hi<ref):
        rec['ideal_root_class']='outside_supplied_ideal_bracket'
        return rec
    input_l,input_h=l,h
    input_vl,input_wl,_,_=ideal(p,l)
    input_vh,_,_,_=ideal(p,h)
    ids=[]
    # Exact interval bisection: every retained decision has a certified strict sign.
    for _ in range(49):
        mid=(l+h)/2; vm,_,ci,_=ideal(p,mid); ids+=ci
        if vm[0].lo>ref: l=mid
        elif vm[0].hi<ref: h=mid
        else: raise AssertionError('raise certificate precision; ambiguous sign')
    vl,wl,il,_=ideal(p,l); vu,wu,iu,_=ideal(p,h); ids+=il+iu
    assert vl[0].lo>ref and vu[0].hi<ref
    corroborate(p,l,vl); corroborate(p,h,vu)
    # Entire supplied bracket: ideal |D1| decreases in both models.
    slope=-input_vh[3].hi; assert slope>0
    de=uniform_dirty_error(p,input_l,input_h,input_vl,input_wl,input_vh)
    pe=None if de is None else de+abs(cached-ref)
    residual_budget=outward(slope*F(1,10**9),False)
    response=outward(1/slope,True)
    pe=None if pe is None else outward(pe,True)
    ce=None if pe is None else pe*response+(h-l)+residual_budget*response
    rec.update(ideal_root_class='unique_sign_certified',root_lo=dyad(l),root_hi=dyad(h),
        lower_price_lo=dyad(vl[0].lo),lower_price_hi=dyad(vl[0].hi),
        upper_price_lo=dyad(vu[0].lo),upper_price_hi=dyad(vu[0].hi),slope_lower=dyad(outward(slope,False)),
        price_error='not_certified' if pe is None else dyad(outward(pe,True)),
        residual_to_coordinate=dyad(response),
        residual_budget=dyad(residual_budget),
        coordinate_error='not_certified' if ce is None else dyad(outward(ce,True)),
        informative=str(ce is not None and ce<F(1,10**7)).lower(),
        certificate_ids=';'.join(sorted(set(ids))))
    STATS['root_sign_certificates']+=1
    return rec

def inverse_rows():
    out=[]
    for f in (2,4,12):
        for sh in SHAPES:
            for sel in ('Compound','MoneyMarket','LastCouponMoneyMarket'):
                s=build_state(f,'Preserve' if sh in ('regular','long_first','long_last','composite') else 'Ignore',sh,sel)
                p=prepare(s)
                # Independently selected external targets, not calculated quote roundtrips.
                for basis,t in (('Clean',97.25),('ModelDirty',103.5),('SettlementDirty',100.75)):
                    lo=-1.0 if p['convention']=='Compound' else -0.2/float(p['tau'])
                    hi=2.0
                    out.append(inverse_record(s,basis,bits(t),bits(lo),bits(hi)))
    # Signed/zero supplied external targets with large accrued source; resolved target positive.
    for f in (2,4,12):
        for sel in ('Compound','MoneyMarket'):
            s=build_state(f,'Ignore','composite',sel)
            for basis in ('Clean','SettlementDirty'):
                out.append(inverse_record(s,basis,bits(98.875),bits(-0.1),bits(2.),
                    'cached_target_reference','cached_resolved_model_dirty'))
    # Signed/zero supplied external targets with large accrued source; resolved target positive.
    for f in (2,4,12):
        s=build_state(f,'Preserve','long_first','Compound',(20000,0))
        first=dt.date.fromisoformat(s['coupon_dates'].split(';')[0])
        s['settlement']=(first-dt.timedelta(days=1)).isoformat()
        for y in (0.,-1.):
            out.append(inverse_record(s,'Clean',bits(y),bits(-0.05),bits(100.),'signed_external_target'))
    # Exact ideal image controls retain nonexistence, not fabricated coordinate roots.
    for f in (2,4,12):
        s=build_state(f,'Ignore','regular','MoneyMarket')
        out.append(inverse_record(s,'ModelDirty',bits(0.01),bits(0.),bits(100.),'ideal_image'))
    for basis,target in (('Clean',99.9875),('ModelDirty',100.125),('SettlementDirty',100.75)):
        out.append(inverse_record(appendix_state(),basis,bits(target),bits(-.1),bits(.2),'appendix_target'))
    # Complete reviewed observed image witness, independently re-entered from contract C6.
    s=dict(accrual_start='2025-03-01',issue='2025-03-01',stated_maturity='2026-03-01',
        true_maturity='2026-03-01',coupon_coefficient='1',coupon_scale='0',frequency='2',eom='Ignore',
        quasi_dates='2025-03-01;2025-09-01;2026-03-01',coupon_dates='2025-09-01;2026-03-01',
        settlement='2025-06-01',source_policy='CanonicalIiac13LocalV1',selection='Explicit(MoneyMarket)',
        date_cap='3',bundle_cap='2',geometry='reviewed_mm_image')
    r=inverse_record(s,'ModelDirty','3fd5375375375375',bits(0.),'7fefffffffffffff','observed_image_protocol')
    assert r['ideal_root_class']=='outside_open_ideal_image'
    r.update(reference_equation='no_ideal_root_observed_protocol',evaluation_budget='2',
        observed_protocol='cached_upper_function_zero',function_calls='2',derivative_calls='0')
    out.append(r)
    for i,r in enumerate(out): r['id']='CAD-INVERSE-%04d' % (i+1)
    return out

def semantic(rows,cols):
    records=[]
    for r in rows:
        cc=int(r['coupon_coefficient']); ss=int(r['coupon_scale'])
        while ss and cc%10==0: cc//=10; ss-=1
        t=[]
        for k in cols:
            v=r[k]
            if k=='coupon_coefficient': v=str(cc)
            if k=='coupon_scale': v=str(ss)
            if k=='yield_bits' or k.endswith('_bits'):
                if v=='8000000000000000': v='0000000000000000'
            t.append(v)
        records.append('\t'.join(t))
    assert len(records)==len(set(records)), 'duplicate semantic public input'
    return ('\n'.join(records)+'\n').encode('ascii')

def mm_protocol_certificate(rows):
    r=next(x for x in rows if x['profile']=='observed_image_protocol')
    p=prepare(r); y=b2f(r['bracket_upper_bits']); den=1+y*p['tau']
    certs=[]; pvs=[]
    def strict_cell(name,x):
        v=round_ieee(x); vb=rne_bits(v); ib=int(vb,16)
        assert 0<ib<0x7fefffffffffffff
        prev=b2f('%016x'%(ib-1)); nxt=b2f('%016x'%(ib+1))
        lower=(prev+v)/2; upper=(v+nxt)/2
        assert lower<x<upper
        certs.append({'stage':name,'exact_input':rational(x),'observed_bits':vb,
            'strict_lower_distance':rational(x-lower),'strict_upper_distance':rational(upper-x)})
        return v
    for row in p['rows']:
        fac=strict_cell('factor '+str(row['ordinal']),(1+y*row['rho'])/den)
        pv=strict_cell('source pv '+str(row['ordinal']),row['payable']*fac)
        pvs.append(pv)
    dirty=strict_cell('dirty sum',sum(pvs,F(0)))
    assert rne_bits(dirty)==r['target_bits'] and len(certs)==5
    floor=sum((x['payable']*x['rho']/p['tau'] for x in p['rows']),F(0))
    assert floor>dirty and floor==F(181,546)
    lower=round_ieee(sum((round_ieee(x['payable']) for x in p['rows']),F(0)))
    assert lower!=dirty
    return {'input_sha256':pin(semantic([r],INV_INPUT))['sha256'],'strict_cells':certs,
        'ideal_floor':rational(floor),'observed_dirty_bits':rne_bits(dirty),
        'floor_minus_observed':rational(floor-dirty),'lower_callback_not_zero':True,
        'upper_callback_zero':True,'protocol_function_calls':2,'protocol_derivative_calls':0,
        'scope':'exact independent selected-MM math and shared solver protocol; no Canadian runtime/ideal root'}

def source_pins():
    b=(ROOT/'docs/M8_CANADIAN_CONTRACT.md').read_bytes()
    assert pin(b)['sha256']==CONTRACT_SHA
    names=re.findall(rb'\| (M8_CANADIAN_[A-Z_]+\.md) \|[^\n]*\| ([0-9a-f]{64}) \|',b)
    assert len(names)==14
    components={}
    for name,sha in names:
        data=(ROOT/'docs'/name.decode()).read_bytes(); assert pin(data)['sha256']==sha.decode()
        components[name.decode()]=pin(data)
    requirements=(ROOT/'docs/REQUIREMENTS.md').read_bytes()
    lines=[line for line in requirements.splitlines() if re.match(rb'\|\s*FR-CAD-\d\d\s*\|',line)]
    assert len(lines)==72
    contract_lines=[line for line in b.splitlines() if re.match(rb'\|\s*FR-CAD-\d\d\s*\|',line)]
    assert lines==contract_lines
    assert pin(b'\n'.join(lines)+b'\n')['sha256']=='a31bb24356af3b8ed2f6454cd6512bdc052f19705e68211e5a300eafd96e37ae'
    return {'contract':pin(b),'components':components,'requirements_rows':pin(b'\n'.join(lines)+b'\n')}

def table(rows,cols):
    assert all(set(r)==set(cols) for r in rows), [(set(r)-set(cols),set(cols)-set(r)) for r in rows if set(r)!=set(cols)][:1]
    return ('\t'.join(cols)+'\n'+'\n'.join('\t'.join(r[c] for c in cols) for r in rows)+'\n').encode('ascii')

def render(rows,cols,inputs,kind,meta,pins,certpin,genpin):
    body=table(rows,cols); records=body.split(b'\n',1)[1]; inp=semantic(rows,inputs)
    profiles={k:sum(r['profile']==k for r in rows) for k in sorted({r['profile'] for r in rows})}
    hs=[('schema','1'),('fixture-set','CAD-IDEAL-'+kind.upper()),('domain','bonds'),
        ('description','Independent certified Canadian '+kind+' references before production'),
        ('reference','oracle:exact-Fraction-nth-root-and-positive-H'),
        ('reference-version','Python '+meta['python']+'; rational outward p224; Decimal100/160 ordinary and768/1024 extreme corroboration'),
        ('reference-source','Adopted CA-IIAC13-fixed-2026-10-23-local-v1; no Canadian implementation'),
        ('tolerance','TOL-E5 conditional propagated absolute bounds; no universal libm claim'),
        ('reproduction','python3 validation/generator/canadian_ideal_oracles.py --check'),
        ('generated-at',meta['generated-at']),('platform',meta['platform']),
        ('generator-sha256',genpin['sha256']),('generator-fnv64',genpin['fnv64']),
        ('adopted-contract-sha256',pins['contract']['sha256']),('adopted-contract-fnv64',pins['contract']['fnv64']),
        ('component-pins',js(pins['components'])),('requirements-rows-sha256',pins['requirements_rows']['sha256']),
        ('requirements-rows-fnv64',pins['requirements_rows']['fnv64']),('certificates-sha256',certpin['sha256']),
        ('certificates-fnv64',certpin['fnv64']),('body-sha256',pin(body)['sha256']),('body-fnv64',fnv(body)),
        ('records-sha256',pin(records)['sha256']),('records-fnv64',fnv(records)),('record-count',str(len(rows))),
        ('column-count',str(len(cols))),('profile-column','profile'),('profile-counts',js(profiles)),
        ('input-columns',','.join(inputs)),('unique-input-count',str(len(rows))),
        ('inputs-sha256',pin(inp)['sha256']),('inputs-fnv64',fnv(inp)),
        ('endpoint-encoding','m@e means exact signed integer m times 2^e; n/d exact rational'),
        ('primitive-hypotheses','native log relative2u; finite positive exp relative2u+2^-1075; actual validation required'),
        ('informative-policy','certified value class and relative allowance <=1e-8; inverse coordinate <1e-7 under checked residual condition'),
        ('inverse-conditioning','quote inside supplied bracket and independently checked absolute model-dirty residual <= residual_budget; coordinate_error includes uniform price profile error, target shift, root width and residual_budget/slope'),
        ('range-qualification','nonstable extreme class checks ideal mathematics only; ordinary value_under_profile requires successful actual primitive validation')]
    return ('\n'.join(k+': '+v for k,v in hs)+'\n\n').encode()+body

def read_meta(path):
    h=path.read_text(encoding='ascii').split('\n\n')[0]
    vals=dict(line.split(': ',1) for line in h.splitlines())
    return {'generated-at':vals['generated-at'],'platform':vals['platform'],
            'python':vals['reference-version'].split(';')[0][7:]}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--output-dir',type=Path)
    ap.add_argument('--repo-root',type=Path,help='private staging only; default repository from generator location')
    ap.add_argument('--check',action='store_true'); ap.add_argument('--schema-only',action='store_true')
    args=ap.parse_args()
    global ROOT
    if args.repo_root: ROOT=args.repo_root.resolve()
    schema={'price_columns':PRICE_COLS,'price_input_columns':PRICE_INPUT,
        'inverse_columns':INV_COLS,'inverse_input_columns':INV_INPUT,
        'profile_column':'profile','identity':'canonical bits zero and canonical Decimal amount; no reference equation/expected values',
        'endpoint':'signed dyadic m@e','root_certificate':'acyclic sidecar; degrees are individual proper quasi-cell day counts'}
    if args.schema_only:
        print(json.dumps(schema,indent=2)); return
    dest=(args.output_dir or ROOT/'validation/fixtures/bonds').resolve()
    names=('canadian-ideal-price-risk-oracle.fixture','canadian-ideal-inverse-oracle.fixture','canadian-ideal-certificates.json')
    paths=[dest/n for n in names]
    if args.check:
        assert all(p.is_file() and not p.is_symlink() for p in paths),'all current files required'
        meta=read_meta(paths[0]); assert meta==read_meta(paths[1])
    else:
        assert not any(p.exists() or p.is_symlink() for p in paths),'exclusive output creation; existing/alias refused'
        meta={'generated-at':dt.datetime.now(dt.timezone.utc).isoformat().replace('+00:00','Z'),
              'platform':platform.platform(),'python':platform.python_version()}
    pins=source_pins(); genpin=pin(Path(__file__).read_bytes())
    print('begin price rows',flush=True); price=price_rows()
    print('begin inverse rows',len(price),flush=True); inverse=inverse_rows()
    ip=semantic(price,PRICE_INPUT); ii=semantic(inverse,INV_INPUT)
    cert={'schema':1,'metadata':meta,'sources':pins,'generator':genpin,
          'inputs':{'price':pin(ip),'inverse':pin(ii)},'statistics':STATS,
          'root_certificates':CERTS,'schema_approval':schema,
          'mm_observed_protocol':mm_protocol_certificate(inverse)}
    cb=(json.dumps(cert,sort_keys=True,indent=2)+'\n').encode('ascii')
    outputs=[render(price,PRICE_COLS,PRICE_INPUT,'price-risk',meta,pins,pin(cb),genpin),
             render(inverse,INV_COLS,INV_INPUT,'inverse',meta,pins,pin(cb),genpin),cb]
    assert pins==source_pins() and genpin==pin(Path(__file__).read_bytes()),'input changed during producer run'
    if args.check:
        assert all(p.read_bytes()==b for p,b in zip(paths,outputs)),'reproduction differs'
    else:
        dest.mkdir(parents=True,exist_ok=True)
        # Exclusive file descriptors close the TOCTOU overwrite window.
        for p,b in zip(paths,outputs):
            with p.open('xb') as out: out.write(b)
    print(js({'check':args.check,'price_rows':len(price),'inverse_rows':len(inverse),
              'root_certificates':len(CERTS),'statistics':STATS,
              'outputs':{p.name:pin(b) for p,b in zip(paths,outputs)}}),flush=True)

if __name__=='__main__': main()
