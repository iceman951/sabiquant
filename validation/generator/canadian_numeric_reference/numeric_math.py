"""Independent exact arithmetic from adopted Canadian contract; no production import.
Python integers/Fraction are reference-only. Runtime fixed storage is a separate proof.
"""
from fractions import Fraction as F
import struct

U=F(1,1<<53); H=F(1,1<<1075)
MAX_BITS=0x7fefffffffffffff
MIN_NORMAL=0x0010000000000000
SIGN=1<<63
Z0=(-86280532199940657026901447021783325824039967682459192601582082568326453749384799,
    -86280532199940657026901447021783325824039967682459192601582082568326453749384798)
ZI=(82187223230455100268499765730225550946232633676658463179948381980536898214648393,
    82187223230455100268499765730225550946232633676658463179948381980536898214648394)

def pow2(e):return F(1<<e) if e>=0 else F(1,1<<-e)
def hex64(i):return f'{i:016x}'
def bits(x):return int.from_bytes(struct.pack('>d',x),'big')
def value(b):
    if isinstance(b,str):b=int(b,16)
    ef=(b>>52)&2047; t=b&((1<<52)-1)
    if ef==2047:raise ValueError('nonfinite seed')
    v=F(t)*pow2(-1074) if ef==0 else F((1<<52)+t)*pow2(ef-1023-52)
    return -v if b&SIGN else v

def floorlog(v):
    assert v>0
    n,d=v.numerator,v.denominator;e=n.bit_length()-d.bit_length()
    if v<pow2(e):e-=1
    return e

def rn_int(v):
    n,d=v.numerator,v.denominator;q,r=divmod(n,d)
    return q+(2*r>d or (2*r==d and q&1))

def ieee(v):
    """One rational -> IEEE RN-even, including direct subnormal rounding.
    Result includes IEEE bits and range class; exact zero is canonical +0.
    """
    v=F(v)
    if v==0:return 0,'zero'
    sign=SIGN if v<0 else 0;a=abs(v);e=floorlog(a)
    if e < -1022:
        q=rn_int(a*pow2(1074))
        if q==0:return sign,'underflow'
        return sign|q,'subnormal' if q<(1<<52) else 'normal'
    if e>1023:return sign|0x7ff0000000000000,'overflow'
    q=rn_int(a/pow2(e-52))
    if q==(1<<53):q>>=1;e+=1
    if e>1023:return sign|0x7ff0000000000000,'overflow'
    return sign|((e+1023)<<52)|(q-(1<<52)),'normal'

def ext53(v):
    """A whole-term RN-even with unbounded retained exponent, no IEEE admission."""
    v=F(v)
    if not v:return (0,0,0)
    a=abs(v);e=floorlog(a)-52;q=rn_int(a/pow2(e))
    if q==(1<<53):q>>=1;e+=1
    return (-1 if v<0 else 1,q,e)

def ext_value(t):return F(t[0]*t[1])*pow2(t[2])
def retained(ts):
    v=sum((ext_value(t) for t in ts),F(0));r=v*pow2(8192)
    if r.denominator!=1:raise ValueError('outside retained basis (primitive input)')
    return r.numerator

def risk_term(convention,measure,pv,y,f,lam,D,R):
    pv=F(pv);y=F(y);lam=F(lam)
    if convention=='compound':
        delta_b,cl=ieee(F(f)+y)
        if cl not in ('normal','subnormal'):raise ValueError('delta invalid')
        delta=value(delta_b)
        terms=(-pv*lam/delta,pv*lam*(lam+1)/(delta*delta),pv*lam/f)
    else:
        tau=F(D,365);rho=F(R,365);h=tau-rho;d=1+y*tau;n=1+y*rho
        if d<=0 or n<=0:raise ValueError('MM affine domain')
        terms=(-pv*h/(d*n),2*pv*tau*h/(d*d*n),pv*h)
    return ext53(terms[measure])

def mm(y,D,R):
    """E=2^1074 representation produces exactly the same reduced ratio."""
    y=F(y);E=1<<1074;Y=y*E
    assert Y.denominator==1 and 0<=R<D
    den=365*E+Y.numerator*D;num=365*E+Y.numerator*R
    if den<=0:return den,num,None,'domain'
    b,c=ieee(F(num,den));return den,num,b,c

def logarg(f,y):
    y=F(y)
    if y<=-f:return 'domain',None
    if not y:return 'zero',0
    if abs(y)<pow2(-50):return 'tiny',ieee(y)[0]
    if y<=-F(f,2):return 'ln',ieee((f+y)/f)[0]
    return 'ln1p',ieee(y/f)[0]

def exponent(f,g,lam):return ieee(-F(g)*F(lam)/f)

def guard(f,tiny,y,z,lam):
    """Return exact radius, branch and closed-box hit. No elementary evaluation."""
    z=F(z);y=F(y);lam=F(lam)
    if not z:return F(0),'zero_identity',False
    if tiny:
        q=-y*lam/f
        r=abs(z-q)+lam*y*y/(2*f*f*(1-abs(y)/f))
        assert abs(z)+r<pow2(-34)
        return r,'tiny_bypass',False
    k=4 if f in (2,4) else 8
    r=(k*U*abs(z)+H)/(1-k*U)
    if abs(z)>=pow2(24):
        assert r<abs(z)/2
        return r,'far_bypass',False
    lo,hi=z-r,z+r
    hit=any(lo<=F(b[1],1<<256) and hi>=F(b[0],1<<256) for b in (Z0,ZI))
    # Independently reconstruct the prescribed cancelled-integer endpoints.
    A=abs(z)*(1<<1075);assert A.denominator==1
    B=(1<<53)-k;C=A.numerator*B;N=k*A.numerator+(1<<53)
    ends=(C-N,C+N) if z>0 else (-(C+N),N-C)
    checks=[]
    for box in (Z0,ZI):
        checks.append(ends[0]<=box[1]*B*(1<<819) and ends[1]>=box[0]*B*(1<<819))
    assert hit==any(checks)
    assert max(abs(t).bit_length() for t in ends)<1153
    return r,'wide_guard',hit

def bridge(convention,basis,dirty,a,ai):
    qa=F(a) if convention=='compound' else F(ai)
    if basis=='ModelDirty':return ieee(dirty)
    if basis=='Clean':return ieee(F(dirty)-qa)
    return ieee(dirty if convention=='mm' else F(dirty)-qa+F(ai))

def target(convention,basis,v,a,ai):
    qa=F(a) if convention=='compound' else F(ai)
    x=F(v) if basis=='ModelDirty' else F(v)+qa if basis=='Clean' else F(v)-F(ai)+qa
    b,c=ieee(x)
    return b,c,value(b)>0 if c not in ('overflow',) else False

def residual(d,t):
    s=0 if abs(t)<pow2(970) else 1
    return s,ieee((F(d)-F(t))*pow2(-s)),ieee(F(d)-F(t))

def dyadic_interval(lo,hi,precision=320):
    """Outward dyadic endpoints with relative precision; no float operations."""
    assert lo<=hi
    m=max(abs(lo),abs(hi))
    if not m:return 0,0,0
    e=floorlog(m)-precision;scale=pow2(-e)
    a=lo*scale;b=hi*scale
    L=a.numerator//a.denominator;U=-((-b.numerator)//b.denominator)
    return L,U,e

def expand_interval(i):return i[0]*pow2(i[2]),i[1]*pow2(i[2])
def _ln1(v,precision=390):
    """Atanh positive series for 1<=v<=2; exact geometric-tail enclosure."""
    assert 1<=v<=2
    t=(v-1)/(v+1)
    if t==0:return F(0),F(0)
    t2=t*t;p=t;s=F(0);n=0
    while True:
        s+=2*p/(2*n+1);p*=t2;n+=1
        tail=2*p/((2*n+1)*(1-t2))
        if tail<=max(abs(s),pow2(-1200))*pow2(-precision):return s,s+tail
        if n>2000:raise ArithmeticError('ln certificate work cap')

_LN2={}
def log_interval(arg,ln1p=False,precision=390):
    v=1+F(arg) if ln1p else F(arg)
    if v<=0:raise ValueError('log domain')
    if v<1:
        # Exact reciprocal keeps near-one negative ln1p away from ln2 cancellation.
        # log(v)=-log(1/v); the positive near1 atanh tail is relative to tiny log.
        lo,hi=log_interval(1/v,False,precision)
        return -hi,-lo
    k=floorlog(v);m=v/pow2(k);a,b=_ln1(m,precision)
    if k:
        if precision not in _LN2:_LN2[precision]=_ln1(F(2),precision+20)
        c,d=_LN2[precision]
        a,b=(a+k*c,b+k*d) if k>0 else (a+k*d,b+k*c)
    return a,b

def exp_interval(z,precision=390):
    """Certified Taylor + reciprocal + outward squaring, |z|<=800 only.
    w=|z|/2^k<=1/4. Positive tail bounded by next/(1-w/(n+2)).
    Dyadic outward reduction after each square prevents denominator explosion.
    """
    z=F(z)
    if abs(z)>800:raise ValueError('elementary certificate authoring range')
    if not z:return F(1),F(1)
    k=max(0,floorlog(abs(z))+3);w=abs(z)/pow2(k)
    s=F(1);term=F(1);n=0
    while True:
        n+=1;term=term*w/n;s+=term
        nxt=term*w/(n+1);tail=nxt/(1-w/(n+2))
        if tail<pow2(-precision-40):break
        if n>2000:raise ArithmeticError('exp certificate work cap')
    lo,hi=s,s+tail
    if z<0:lo,hi=1/hi,1/lo
    for _ in range(k):lo,hi=expand_interval(dyadic_interval(lo*lo,hi*hi,precision+30))
    return lo,hi

def elementary(function,arg):
    if function=='exp':
        lo,hi=exp_interval(arg)
        # Conditional assertion only for OBSERVED positive finite exp values.
        alo=lo*(1-2*U)-H;ahi=hi*(1+2*U)+H
    else:
        lo,hi=log_interval(arg,function=='ln1p')
        # Relative primitive ln/ln1p hypothesis, including signed ideal values.
        error=2*U*max(abs(lo),abs(hi));alo,ahi=lo-error,hi+error
    return dyadic_interval(lo,hi),dyadic_interval(alo,ahi)
