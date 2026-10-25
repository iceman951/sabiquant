"""Create-only ordinary-case research CLI. Fraction storage is unbounded, not a Rust
resource route. Output is not a financial fixture/production acceptance oracle.
No production imports. Existing Canadian generator is used for source metadata only;
no pre-existing factor/ideal/output code is called. Decimal is corroboration only.
"""
from fractions import Fraction as F
from decimal import Decimal, localcontext
from pathlib import Path
import argparse, hashlib, json, math, runpy, sys
from datetime import datetime, timezone
sys.dont_write_bytecode = True
PARSER=argparse.ArgumentParser(description='Unadopted ordinary enclosure research; unbounded Fraction storage, no Rust or financial acceptance.')
PARSER.add_argument('--repo',type=Path,required=True,help='Local SabiQuant source repository (read-only).')
PARSER.add_argument('--output',type=Path,required=True,help='New directory outside --repo; must not exist and parent must exist.')
ARGS=PARSER.parse_args()
SCRIPT_PATH=Path(__file__).resolve()
SCRIPT_SHA_BEFORE=hashlib.sha256(SCRIPT_PATH.read_bytes()).hexdigest()
ROOT=ARGS.repo.resolve()
if not ROOT.is_dir(): PARSER.error('--repo must be an existing directory')
if ARGS.output.exists() or ARGS.output.is_symlink(): PARSER.error('--output must not already exist')
OUT=ARGS.output.resolve()
if OUT.is_relative_to(ROOT): PARSER.error('--output must be outside --repo')
if not OUT.parent.is_dir(): PARSER.error('--output parent must already exist')
METADATA_PATH=ROOT/'validation/generator/canadian_ideal_oracles.py'
METADATA_SHA='b1c7fa91078288fa44c858b9bd4e839aaf1bb143cc39ecb82d7b563b65961a7e'
if not METADATA_PATH.is_file(): PARSER.error('Canadian metadata source is missing')
METADATA_SHA_BEFORE=hashlib.sha256(METADATA_PATH.read_bytes()).hexdigest()
if METADATA_SHA_BEFORE!=METADATA_SHA:
    PARSER.error('Canadian metadata source bytes differ from the reviewed research input')
OUT.mkdir(exist_ok=False)

P=96
COUNTS={'round_checks':0,'log_calls':0,'exp_calls':0,'rational_price_checks':0,'decimal_checks':0,'root_sign_checks':0}
LOG={}
def pow2(e): return F(1<<e) if e>=0 else F(1,1<<(-e))
def floorlog(x):
    assert x>0
    e=x.numerator.bit_length()-x.denominator.bit_length()
    return e-1 if x<pow2(e) else e
def rounded(x,up):
    if not x: return F(0)
    u=pow2(floorlog(abs(x))-P+1)
    q=x//u
    z=(q+(bool(up) and x!=q*u))*u
    assert (z>=x if up else z<=x)
    COUNTS['round_checks']+=1
    return z
class I:
    __slots__=('lo','hi')
    def __init__(self,a,b=None):
        a=F(a); b=a if b is None else F(b)
        assert a<=b
        self.lo=rounded(a,False); self.hi=rounded(b,True)
    def __add__(self,b):
        b=ii(b); return I(self.lo+b.lo,self.hi+b.hi)
    __radd__=__add__
    def __neg__(self): return I(-self.hi,-self.lo)
    def __sub__(self,b): return self+-ii(b)
    def __mul__(self,b):
        b=ii(b); a=[self.lo*b.lo,self.lo*b.hi,self.hi*b.lo,self.hi*b.hi]
        return I(min(a),max(a))
    __rmul__=__mul__
    def __truediv__(self,b):
        b=ii(b); assert not b.lo<=0<=b.hi
        return self*I(1/b.hi,1/b.lo)
def ii(x): return x if isinstance(x,I) else I(x)
def atanh(m):
    assert 1<=m<=2
    z=I((m-1)/(m+1)); z2=z*z; term=z; total=I(0)
    for j in range(P):
        total=total+term*F(2,2*j+1)
        term=term*z2
    # z_true<=1/3, so tail <=2^(1-2P), proved in REPORT.md.
    return total+I(0,pow2(1-2*P))
def logq(q):
    q=F(q); assert q>0
    if q==1: return I(0)
    if q in LOG: return LOG[q]
    COUNTS['log_calls']+=1
    k=floorlog(q); m=q/pow2(k)
    l2=atanh(F(2))
    z=atanh(m)+k*l2
    LOG[q]=z
    return z
def exp_point(x):
    COUNTS['exp_calls']+=1
    if not x: return I(1)
    negative=x<0; a=abs(x); squares=0
    while a>F(1,2): a/=2; squares+=1
    assert squares<=2049
    term=I(1); total=I(1)
    for j in range(1,2*P+1):
        term=term*a/F(j); total=total+term
    ans=total+I(0,pow2(-2*P))
    # Intersect with the proved e^a in [1,2], a in [0,1/2].
    ans=I(max(ans.lo,F(1)),min(ans.hi,F(2)))
    for _ in range(squares): ans=ans*ans
    return I(1)/ans if negative else ans
def expi(x): return I(exp_point(x.lo).lo,exp_point(x.hi).hi)
def factor(base,lam): return expi(-logq(base)*lam)
def dc(x): return Decimal(x.numerator)/Decimal(x.denominator)
def corroborate(value,reference):
    # Decimal at two precisions is corroborative and not a certificate.
    with localcontext() as c:
        c.prec=220
        ref=reference()
        assert dc(value.lo)<=ref<=dc(value.hi),(value.lo,value.hi,ref)
    COUNTS['decimal_checks']+=1
    with localcontext() as c:
        c.prec=300
        ref=reference()
        assert dc(value.lo)<=ref<=dc(value.hi)
    COUNTS['decimal_checks']+=1

def cash_cell(x):
    # N=1,000,000, price per100, final cent nearest/even.
    def rnd(v):
        z=v*1000000; q,r=divmod(z.numerator,z.denominator)
        return q+(2*r>z.denominator or (2*r==z.denominator and q%2))
    return rnd(x.lo)==rnd(x.hi)
ROWS=[]
def record(name,model,vals,exact=None,reference=None,qual='arithmetic coordinates'):
    x=vals['dirty']
    if exact is not None:
        assert x.lo<=exact<=x.hi
        COUNTS['rational_price_checks']+=1
    if reference is not None: corroborate(x,reference)
    ROWS.append({'name':name,'model':model,'qualification':qual,
       'basis_count':len(vals),'all_basis_width_below_1e-18':all(v.hi-v.lo<F(1,10**18) for v in vals.values()),
       'all_basis_cash_decided':all(cash_cell(v) for v in vals.values()),
       'bounds':{k:[str(v.lo),str(v.hi)] for k,v in vals.items()}})

Y=F(*(.043).as_integer_ratio())
FREQ=[1,2,3,4,6,12,13,26,52,365]
for method,m in [('Simple',None),('Continuous',None)]+[(a,m) for a in ('Compounded','SimpleThenCompounded','CompoundedThenSimple') for m in FREQ]:
    threshold=None if m is None else 1.0/m
    times=[F(1,8),F(3,4),F(9,4)] if m is None else [F(*math.nextafter(threshold,-math.inf).as_integer_ratio()),F(*threshold.as_integer_ratio()),F(*math.nextafter(threshold,math.inf).as_integer_ratio())]
    amounts=[F(100),F(-40),F(7)]
    branches=[]; terms=[]
    for a,t in zip(amounts,times):
        simple=(method=='Simple' or method=='SimpleThenCompounded' and float(t)<=threshold or method=='CompoundedThenSimple' and float(t)>threshold)
        branches.append('Simple' if simple else 'Continuous' if method=='Continuous' else 'Periodic')
        if simple: terms.append(I(a/(1+Y*t)))
        elif method=='Continuous': terms.append(a*expi(I(-Y*t)))
        else: terms.append(a*factor(1+Y/m,m*t))
    dirty=sum(terms,I(0)); ai=F(*(.1234).as_integer_ratio())
    def ref(amounts=amounts,times=times,branches=branches,m=m):
        ans=Decimal(0)
        for a,t,b in zip(amounts,times,branches):
            d=1/(1+dc(Y*t)) if b=='Simple' else (-dc(Y*t)).exp() if b=='Continuous' else (-dc(m*t)*dc(1+Y/m).ln()).exp()
            ans+=dc(a)*d
        return ans
    exact=sum((a/(1+Y*t) for a,t in zip(amounts,times)),F(0)) if all(b=='Simple' for b in branches) else None
    record(method+('' if m is None else '_'+str(m)),'generic',{'dirty':dirty,'clean':dirty-ai},exact,ref,'all32 configurations; signed amounts; mixed branches include RN(1/m) and both adjacent times')
# Identical factor cancellation is mathematically exact; independent intervals may be wide.
z=factor(1+Y/2,F(13,7)); summed=100*z-100*z
assert summed.lo<=0<=summed.hi
record('exact_opposite_factor_cancellation','generic',{'dirty':summed},F(0),qual='arithmetic-only cancellation; not a production plan/domain test')
# Root sign controls check the output against exact rational inequalities.
for base,lam in [(F(9,4),F(1,2)),(F(8),F(1,3)),(F(101,100),F(77,183)),(F(99,100),F(13,31))]:
    z=factor(base,lam); n,d=lam.numerator,lam.denominator
    assert z.lo**d*base**n<=1<=z.hi**d*base**n
    COUNTS['root_sign_checks']+=1
# Gilt arithmetic coordinates: exact published first coupon; omitted ex bundles retain j.
for shape in ('regular','short_first','long_first','ex_dividend'):
    C=F(525,100); first=C/2
    if shape=='short_first': first=F(round(C/2*F(70,182)*10**6),10**6)
    if shape=='long_first': first=F(round(C/2*(1+F(70,182))*10**6),10**6)
    pairs=[(first,F(77,183)),(C/2,1+F(77,183)),(100+C/2,3+F(77,183))]
    if shape=='ex_dividend': pairs=pairs[1:]
    dirty=sum((a*factor(1+Y/2,lam) for a,lam in pairs),I(0)); ai=-C/2*F(1,183) if shape=='ex_dividend' else C/2*F(106,183)
    record(shape,'gilt',{'dirty':dirty,'clean':dirty-ai},reference=lambda pairs=pairs:sum((dc(a)*(-dc(lam)*dc(1+Y/2).ln()).exp() for a,lam in pairs),Decimal(0)),qual='source-coordinate arithmetic examples; not full dated lifecycle admission')
# Rational Treasury full price, exact raw coefficients and selected AI held fixed.
for shape in ('regular','short_first','long_first'):
    c=F(525,200); first=c if shape=='regular' else c*F(70,182) if shape=='short_first' else c*(1+F(70,182))
    q=1+Y*F(92,184); b=1+Y/2
    dirty=sum((I(a/(q*b**j)) for a,j in [(first,0),(c,1),(100+c,3)]),I(0))
    exact=sum((a/(q*b**j) for a,j in [(first,0),(c,1),(100+c,3)]),F(0)); ai=F(12345,10**6)
    record(shape,'treasury',{'dirty':dirty,'clean':dirty-ai},exact,qual='raw coefficient shapes; selected published AI fixed; not all source policy/date paths')
# Existing independent Canadian generator supplies dated geometry/source coefficients ONLY.
meta=runpy.run_path(str(ROOT/'validation/generator/canadian_ideal_oracles.py'),run_name='research_metadata_only')
for f in (2,4,12):
  for shape in meta['SHAPES']:
   for mode in ('Compound','MoneyMarket','LastCouponMoneyMarket'):
    s=meta['build_state'](f,'Preserve',shape,selection=mode); p=meta['prepare'](s)
    if p['convention']=='Compound':
      pairs=[(r['valuation'],r['lambda']) for r in p['rows']]
      dirty=sum((a*factor(1+Y/f,lam) for a,lam in pairs),I(0)); clean=dirty-p['a']; settle=dirty+(p['ai']-p['a'])
      ref=lambda pairs=pairs,f=f:sum((dc(a)*(-dc(lam)*dc(1+Y/f).ln()).exp() for a,lam in pairs),Decimal(0))
      exact=None
    else:
      exact=sum((r['payable']*(1+Y*r['rho'])/(1+Y*p['tau']) for r in p['rows']),F(0))
      dirty=I(exact); clean=dirty-p['ai']; settle=dirty; ref=None
    record(str(f)+'_'+shape+'_'+mode,'canadian',{'dirty':dirty,'clean':clean,'settlement':settle},exact,ref,'dated metadata reused from frozen independent source generator; fresh arithmetic; no production read or existing ideal/factor called')
summary={'status':'PASS_ORDINARY_RESEARCH_ONLY','precision_bits':P,'rows':len(ROWS),'by_model':{m:sum(r['model']==m for r in ROWS) for m in ('generic','gilt','treasury','canadian')},'counts':COUNTS,
 'all_basis_width_below_1e-18':sum(r['all_basis_width_below_1e-18'] for r in ROWS),
 'all_basis_cash_decided':sum(r['all_basis_cash_decided'] for r in ROWS),
 'limitations':['Fraction prototype is unbounded storage; no Rust resource or production test claim.','Decimal checks are corroboration only; log/exp containment depends on proved series and directed operation induction.','Canadian source/date preparation reuses an accepted independent producer; it is not independent new metadata validation.','Gilt/Treasury rows are arithmetic-coordinate controls, not complete dated/public/source-policy tests.','P96 is a research choice, no API/contract precision adopted.']}
with (OUT/'ordinary-results.json').open('x',encoding='utf-8') as stream:
    stream.write(json.dumps({'summary':summary,'rows':ROWS},indent=2,sort_keys=True)+'\n')
OUTPUT_SHA=hashlib.sha256((OUT/'ordinary-results.json').read_bytes()).hexdigest()
EXPECTED_ORDINARY_SHA='ec255adae325a9f333479b70f0ae7ff77b37c5acb351758962b86c4a1e34bd21'
assert OUTPUT_SHA==EXPECTED_ORDINARY_SHA,'ordinary research arithmetic/result bytes changed'
SCRIPT_SHA_AFTER=hashlib.sha256(SCRIPT_PATH.read_bytes()).hexdigest()
METADATA_SHA_AFTER=hashlib.sha256(METADATA_PATH.read_bytes()).hexdigest()
OBSERVED_INPUTS_EQUAL=SCRIPT_SHA_BEFORE==SCRIPT_SHA_AFTER and METADATA_SHA_BEFORE==METADATA_SHA_AFTER
with (OUT/'run-evidence.json').open('x',encoding='utf-8') as stream:
    stream.write(json.dumps({'status':'REPRODUCED_ORDINARY_RESEARCH_ONLY' if OBSERVED_INPUTS_EQUAL else 'RESEARCH_PROVENANCE_CHANGED','generated_at_utc':datetime.now(timezone.utc).isoformat(),
      'script_sha256_before':SCRIPT_SHA_BEFORE,'script_sha256_after':SCRIPT_SHA_AFTER,
      'metadata_sha256_before':METADATA_SHA_BEFORE,'metadata_sha256_after':METADATA_SHA_AFTER,
      'observed_input_hashes_equal':OBSERVED_INPUTS_EQUAL,
      'observation_scope':'Two before/after byte observations only; not continuous monitoring.',
      'repo':str(ROOT),'output':str(OUT),
      'metadata_source_sha256':METADATA_SHA,'ordinary_results_sha256':OUTPUT_SHA,
      'original_prototype_sha256':'3a9e43026bcb29964f65ab9df6115bd0b57be0fa11166e0582a1b6714f5c43ce',
      'limitations':summary['limitations']+['The cash-cell arithmetic control is not a full settlement receipt.','P96 is research only; no API precision adopted.']},indent=2,sort_keys=True)+'\n')
assert OBSERVED_INPUTS_EQUAL,'script or metadata bytes differed at the before/after observations'
print(json.dumps(summary,sort_keys=True))
