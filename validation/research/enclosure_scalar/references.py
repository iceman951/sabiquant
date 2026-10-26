#!/usr/bin/env python3
"""Independent PRIVATE RESEARCH primitive references, not production fixtures.
No proposed/actual bounded implementation or its outputs are imported. Exact integer
quotients/Fraction checks use relative exponents only. Huge K is symbolic metadata.
"""
import argparse,collections,datetime,hashlib,json,pathlib,sys
from fractions import Fraction as F
PSET=(64,128,256,512,1024);RAW=2176;EWORDS=33;H=1<<2051
SIGN=1<<63;MAXF=0x7fefffffffffffff;FRAC=(1<<52)-1
CHECKS=collections.Counter();CASES={}
def canonical(x):return json.dumps(x,sort_keys=True,separators=(',',':'))
def sha(x):return hashlib.sha256(x).hexdigest()
def sh(x):return ('-' if x<0 else '')+format(abs(x),'x')
def ui(s):return int(s,16)
def pow2(e):
 assert abs(e)<=6000,'absolute/huge power is forbidden in reference arithmetic'
 return F(1<<e) if e>=0 else F(1,1<<(-e))
def endpoint(sign,m,k,p):
 if not m:return {'sign':0,'m':'0','K':'0'}
 assert sign in(-1,1) and m.bit_length()==p and abs(k).bit_length()<=EWORDS*64
 return {'sign':sign,'m':sh(m),'K':sh(k)}
def components(a):return a['sign'],ui(a['m']),ui(a['K'])
def relative(a,base,p):
 s,m,k=components(a)
 return F(0) if not m else s*m*pow2(k-(p-1)-base)
def positive_step(m,k,p,up):
 if up:return (m+1,k) if m<(1<<p)-1 else (1<<(p-1),k+1)
 return (m-1,k) if m>1<<(p-1) else ((1<<p)-1,k-1)
def step(a,p,up):
 s,m,k=components(a);assert m
 n,e=positive_step(m,k,p,up if s>0 else not up)
 return endpoint(s,n,e,p)
def norm(n,d,e,p):
 if not d:return {'precondition_error':'zero_denominator'}
 assert d>0
 if not n:
  z=endpoint(0,0,0,p);return {'lower':z,'upper':z}
 negative=n<0;a=abs(n);k=a.bit_length()-d.bit_length()
 if (a<d<<k if k>=0 else a<<(-k)<d):k-=1
 shift=p-1-k
 assert abs(shift)<=RAW+p+2
 nn,dd=(a<<shift,d) if shift>=0 else(a,d<<(-shift))
 q,r=divmod(nn,dd);assert 1<<(p-1)<=q<1<<p
 def node(q,s):
  kk=e+k
  if q==1<<p:q>>=1;kk+=1
  return endpoint(s,q,kk,p)
 low=node(q,-1 if negative else 1);high=node(q+bool(r),-1 if negative else 1)
 return {'lower':high if negative else low,'upper':low if negative else high}
def verify_tight(pair,x,base,p):
 assert 'precondition_error' not in pair
 lo,hi=relative(pair['lower'],base,p),relative(pair['upper'],base,p)
 assert lo<=x<=hi
 if pair['lower']==pair['upper']:assert lo==x
 else:
  assert lo<x<hi and step(pair['lower'],p,True)==pair['upper']
 for a in pair.values():
  s,m,k=components(a)
  if not m:assert s==k==0
 CHECKS['exact_tight_pair_checks']+=1

def normalized_case(p,n,d,e,negative,tag):
 assert 0<=n.bit_length()<=RAW and 0<=d.bit_length()<=RAW
 pair=norm(-n if negative else n,d,e,p)
 if d:verify_tight(pair,F(-n if negative else n,d),e,p)
 else:assert pair=={'precondition_error':'zero_denominator'};CHECKS['denominator_zero_priority']+=1
 add('normalize',p,{'N':sh(n),'D':sh(d),'negative':negative,'e':sh(e)},pair,'exact_directed_pair',tag)
def binary_case(op,p,a,b,tag):
 sa,ma,ka=components(a);sb,mb,kb=components(b);ea=ka-(p-1);eb=kb-(p-1)
 if op=='mul':
  n=sa*sb*ma*mb;d=1;e=ea+eb
 elif op=='div':
  n=sa*sb*ma;d=mb;e=ea-eb
 else:raise AssertionError(op)
 assert max(abs(n).bit_length(),d.bit_length())<=RAW
 pair=norm(n,d,e,p)
 if d:verify_tight(pair,F(n,d),e,p)
 else:assert pair=={'precondition_error':'zero_denominator'};CHECKS['denominator_zero_priority']+=1
 add(op,p,{'a':a,'b':b},pair,'exact_directed_pair',tag)
def add_case(p,a,b,tag):
 sa,ma,ka=components(a);sb,mb,kb=components(b)
 if not ma:pair={'lower':b,'upper':b};proof='zero_identity'
 elif not mb:pair={'lower':a,'upper':a};proof='zero_identity'
 else:
  dominant,small=(a,b) if ka>=kb else(b,a);sd,md,kd=components(dominant);ss,ms,ks=components(small);gap=kd-ks
  if gap>p+2:
   pair={'lower':dominant if ss>0 else step(dominant,p,False),'upper':step(dominant,p,True) if ss>0 else dominant}
   assert ms and md and gap>=p+3
   assert step(pair['lower'],p,True)==pair['upper']
   proof='symbolic_neighbor: |small| < dominant_unit/8; nonzero exact sum lies strictly toward sign(small) between dominant and adjacent p-value'
   CHECKS['symbolic_huge_gap_pairs']+=1
  else:
   ea=ka-(p-1);eb=kb-(p-1);e=min(ea,eb)
   n=sa*ma*(1<<(ea-e))+sb*mb*(1<<(eb-e))
   assert abs(n).bit_length()<=RAW
   pair=norm(n,1,e,p);verify_tight(pair,F(n),e,p);proof='exact_relative_integer_sum'
 if ma and mb and abs(ka-kb)<=4096:
  base=max(ka,kb);x=relative(a,base,p)+relative(b,base,p);verify_tight(pair,x,base,p)
 elif not ma or not mb:
  base=kb if mb else ka;verify_tight(pair,relative(a,base,p)+relative(b,base,p),base,p)
 inputs={'a':a,'b':b}
 add('add',p,inputs,dict(pair,proof=proof),'candidate_must_cover_tight_pair_not_bit_equal',tag)

def finite(bits):return {'status':'finite','bits':f'{bits:016x}'}
def unavailable():return {'status':'unavailable_finite_bound'}
def positive_export(m,k,p):
 if k>1023:return finite(MAXF),unavailable()
 if k< -1074:return finite(0),finite(1)
 unit=max(k-52,-1074);shift=k-(p-1)-unit
 nn,dd=(m<<shift,1) if shift>=0 else(m,1<<(-shift))
 q,r=divmod(nn,dd)
 def encode(c):
  top=k
  if k< -1022:return finite(c)
  if c==1<<53:c>>=1;top+=1
  if top>1023:return unavailable()
  return finite(((top+1023)<<52)|(c&FRAC))
 return encode(q),encode(q+bool(r))
def negate_export(x):
 if x['status']!='finite':return dict(x)
 b=int(x['bits'],16);return finite(0 if not b else b^SIGN)
def ieee_export(a,p):
 s,m,k=components(a)
 if not m:return {'lower':finite(0),'upper':finite(0),'finite_pair':True}
 lo,hi=positive_export(m,k,p)
 if s<0:lo,hi=negate_export(hi),negate_export(lo)
 return {'lower':lo,'upper':hi,'finite_pair':lo['status']==hi['status']=='finite'}
def ieee_fraction(bits):
 sign=-1 if bits&SIGN else 1;field=(bits>>52)&2047;f=bits&FRAC;assert field<2047
 sig=f if not field else f+(1<<52);e=-1074 if not field else field-1075
 return sign*sig*pow2(e)
def ieee_successor(bits):
 if bits==MAXF:return None
 if bits&SIGN:return 0 if bits==SIGN|1 else bits-1
 return bits+1

def export_case(p,a,tag):
 out=ieee_export(a,p);s,m,k=components(a)
 if not m:assert out=={'lower':finite(0),'upper':finite(0),'finite_pair':True}
 elif abs(k)<=4096:
  x=relative(a,0,p);lo,hi=out['lower'],out['upper']
  if lo['status']=='finite':assert ieee_fraction(int(lo['bits'],16))<=x
  else:assert x< -ieee_fraction(MAXF)
  if hi['status']=='finite':assert x<=ieee_fraction(int(hi['bits'],16))
  else:assert x>ieee_fraction(MAXF)
  if out['finite_pair'] and lo!=hi:assert ieee_successor(int(lo['bits'],16))==int(hi['bits'],16)
 elif k>1023:
  assert not out['finite_pair']
 elif k< -1074:
  assert out['finite_pair']
 else:raise AssertionError('unclassified huge export')
 CHECKS['ieee_outward_export_checks']+=1
 add('export_binary64',p,{'point':a},out,'exact_finite_outward_pair_or_unavailable_side',tag)

def add(op,p,inputs,expected,relation,tag):
 key=canonical({'op':op,'p':p,'input':inputs})
 if key in CASES:
  assert CASES[key]['expected']==expected;CASES[key]['families'].add(tag);return
 CASES[key]={'op':op,'p':p,'input':inputs,'expected':expected,'relation':relation,'families':{tag},'scope':'PRIVATE_PRIMITIVE_RESEARCH_ONLY; generic helper controls; admitted financial reachability not asserted'}
def generate():
 for p in PSET:
  lo=1<<(p-1);hi=(1<<p)-1;smallE=[0,-1074,971,-2148,1942]
  basic=[(0,1),(0,0),(1,0),(1,1),(1,2),(1,3),(2,3),(3,2),((1<<(p+1))-1,2),((1<<(p+1))-3,2),((1<<p)+1,2)]
  for n,d in basic:
   for neg in [False,True]:
    for e in smallE:normalized_case(p,n,d,e,neg,'zero_priority_identity_division_and_carry')
  for w in range(1,35):
   b=64*w;mx=(1<<b)-1
   for n,d in [(mx,1),(1,mx),((1<<(b-1))+1,mx),(mx-2,mx),(mx,(1<<(b-1))+1)]:
    for neg in [False,True]:normalized_case(p,n,d,smallE[w%len(smallE)],neg,'all34_raw_word_joins_and_fullwidth_ratios')
  for j in range(1,33):
   e=(1<<(64*j))-1
   for es in [e,-e]:
    for n,d in [(1,3),(3,2)]:
     for neg in [False,True]:normalized_case(p,n,d,es,neg,'all32_metadata_word_joins_within_graph_envelope')
  for e in [H,-H,H+63,-H-63]:
   for n,d in [(1,3),((1<<(p+1))-1,2)]:
    for neg in [False,True]:normalized_case(p,n,d,e,neg,'2052bit_metadata_no_absolute_shift')
  mant={lo,lo+1,hi,hi-1,lo+(1<<(p//2))}
  for bit in range(1,p-1):
   if bit%64 in(0,1,63):mant.add(lo+(1<<bit))
  mant=sorted(mant)
  for i,ma in enumerate(mant):
   for mb in [lo,hi,mant[(i*7+3)%len(mant)]]:
    for sa in[-1,1]:
     for sb in[-1,1]:
      ka,kb=[(0,0),(-1074,971),(H,-H),(H,H),(-H,-H)][i%5]
      a=endpoint(sa,ma,ka,p);b=endpoint(sb,mb,kb,p)
      binary_case('mul',p,a,b,'signed_products_mantissa_word_joins_and_huge_metadata')
      binary_case('div',p,a,b,'signed_quotients_mantissa_word_joins_and_huge_metadata')
  z=endpoint(0,0,0,p)
  for s in[-1,1]:
   a=endpoint(s,hi,H,p)
   for aa,bb in[(z,z),(z,a),(a,z)]:
    binary_case('mul',p,aa,bb,'canonical_zero_products')
    binary_case('div',p,aa,bb,'denominator_zero_before_numerator_zero')
  for j in range(1,33):
   k=1<<(64*j)
   for sign in[-1,1]:
    binary_case('mul',p,endpoint(sign,lo,k-1,p),endpoint(1,lo,1,p),'metadata_add_carry_at_each_join')
    binary_case('div',p,endpoint(sign,lo,k,p),endpoint(1,lo,1,p),'metadata_sub_borrow_at_each_join')
  # Explicit whole-product ceiling carries at every precision, before any core.
  # (lo+1)*(2*lo-2)=2*lo*lo-2 lies just below the next binade.
  for ka,kb in [(0,0),(H,-H)]:
   for sa in[-1,1]:
    for sb in[-1,1]:
     binary_case('mul',p,endpoint(sa,lo+1,ka,p),endpoint(sb,hi-1,kb,p),'explicit_mul_p_plus_one_carry_all_precisions')
  # Couple quotient carry with each metadata limb carry, rather than testing
  # those mechanisms only in separate cases. Raw bounds stay<=2176 bits.
  for j in range(1,33):
   e=(1<<(64*j))-p
   for neg in[False,True]:
    normalized_case(p,(1<<(p+1))-1,2,e,neg,'paired_mantissa_and_metadata_carry_at_every_join')
  gaps=sorted({0,1,p-1,p,p+1,p+2,p+3,64,128,512,1024,2176,H})
  for ka in[0,-1074,1023,H,-H]:
   for i,ma in enumerate([lo,lo+1,hi]):
    for gap in gaps:
     for sa in[-1,1]:
      for sb in[-1,1]:
       add_case(p,endpoint(sa,ma,ka,p),endpoint(sb,[lo,hi,lo+1][i],ka-gap,p),'signed_add_binades_cancellation_and_gap_boundaries')
   for s in[-1,1]:
    add_case(p,endpoint(s,lo+1,ka,p),endpoint(-s,lo,ka,p),'maximal_close_cancellation_p_minus_one_bits')
    add_case(p,endpoint(s,hi,ka,p),endpoint(-s,hi,ka,p),'exact_opposite_cancellation_canonical_zero')
    add_case(p,z,endpoint(s,hi,ka,p),'zero_add_identity')
  for j in range(1,33):
   ka=(1<<(64*j))-1
   add_case(p,endpoint(1,lo,ka,p),endpoint(1,1<<(p-1),ka-H,p),'symbolic_neighbors_across_each_metadata_join')
  landmarks=[1,2,3,(1<<52)-1,1<<52,(1<<52)+1,0x3fefffffffffffff,0x3ff0000000000000,0x3ff0000000000001,MAXF-1,MAXF]
  export_case(p,z,'canonical_zero_export')
  for bits in landmarks:
   field=bits>>52;frac=bits&FRAC;sig=frac if field==0 else frac+(1<<52);e=-1074 if field==0 else field-1075
   point=norm(sig,1,e,p)['lower'];assert point==norm(sig,1,e,p)['upper']
   for q in[step(point,p,False),point,step(point,p,True)]:
    for s in[-1,1]:
     _,m,k=components(q);export_case(p,endpoint(s,m,k,p),'MINsub_MINnormal_one_MAX_and_adjacent_exact_p_dyadics')
  for n,e in[((1<<54)-1,970),(1,1024),(1,-1075)]:
   q=norm(n,1,e,p)['lower']
   for s in[-1,1]:
    _,m,k=components(q);export_case(p,endpoint(s,m,k,p),'IEEE_midpoints_and_unavailable_finite_bounds')
  for k in[H,-H,H+63,-H-63]:
   for m in[lo,hi]:
    for s in[-1,1]:export_case(p,endpoint(s,m,k,p),'huge_metadata_export_classification_without_power')
 # Explicit incoming 2053-bit metadata controls. These are generic helper
 # controls, not an assertion that every endpoint pair is financially reachable.
 # One huge operand plus ordinary metadata keeps all outputs/relative gaps
 # strictly within |K/e|<2^2053; no power-of-two shift uses huge metadata.
 J=1<<2052
 for p in PSET:
  lo=1<<(p-1);hi=(1<<p)-1
  for e in [J,-J]:
   for n,d in [(1,3),(3,2),((1<<(p+1))-1,2)]:
    for neg in [False,True]:normalized_case(p,n,d,e,neg,'incoming_2053bit_raw_metadata_within_temporary_envelope')
   for sa in [-1,1]:
    for sb in [-1,1]:
     a=endpoint(sa,hi,e,p);b=endpoint(sb,lo+1,0,p)
     binary_case('mul',p,a,b,'incoming_2053bit_endpoint_metadata_with_ordinary_other_operand')
     binary_case('div',p,a,b,'incoming_2053bit_endpoint_metadata_with_ordinary_other_operand')
     add_case(p,endpoint(sa,lo+1,e,p),endpoint(sb,hi,e-p,p),'incoming_2053bit_add_metadata_and_precision_gap')
   for sign in [-1,1]:
    add_case(p,endpoint(sign,hi,e,p),endpoint(-sign,hi,e,p),'incoming_2053bit_exact_opposite_cancellation')
    export_case(p,endpoint(sign,hi,e,p),'incoming_2053bit_finite_export_classification')
 rows=[]
 for i,key in enumerate(sorted(CASES),1):
  row=dict(CASES[key]);row['id']=f'EP-PRIM-{i:06d}';row['families']=sorted(row['families']);rows.append(row)
 return rows


def output_path(parser, repo, value, directory=False):
    """Resolve a create-only output outside the supplied repository."""
    repo = repo.resolve(strict=True)
    if not repo.is_dir():
        parser.error("--repo must name an existing repository directory")
    output = value.resolve()
    if output == repo or repo in output.parents:
        parser.error("output must be outside --repo")
    if not output.parent.is_dir():
        parser.error("output parent directory must already exist")
    return output

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--repo',required=True,type=pathlib.Path);g=parser.add_mutually_exclusive_group(required=True);g.add_argument('--output',type=pathlib.Path,help='create a new reference directory');g.add_argument('--references',type=pathlib.Path,help='verify an existing directory');args=parser.parse_args()
 if args.output:args.output=output_path(parser,args.repo,args.output,directory=True)
 rows=generate();payload=''.join(canonical(r)+'\n' for r in rows).encode();counts=dict(sorted(collections.Counter(r['op'] for r in rows).items()));by_p={str(p):dict(sorted(collections.Counter(r['op'] for r in rows if r['p']==p).items())) for p in PSET}
 spec={'schema':1,'status':'PRIVATE_PRIMITIVE_RESEARCH_ONLY','precisions':list(PSET),'raw_bits':RAW,'exponent_storage_words':EWORDS,'maximum_huge_input_K_magnitude_bits':2053,'case_count':len(rows),'operation_counts':counts,'by_precision':by_p,'cases_sha256':sha(payload),'cases_bytes':len(payload),'generator_sha256':sha(pathlib.Path(__file__).read_bytes()),'independence':'No bounded Rust/prototype implementation or implementation output imported; exact integer quotient and relative Fraction mathematics only.','add_policy':'Tight reference pair must be covered by candidate output. Exact equality is not required for the proposed coarse directed alignment.','zero_policy':'All output zero endpoints and binary zero bounds are canonical positive zero. Zero denominator takes priority over zero numerator.','scope_limits':['Generic helper controls do not assert admitted financial reachability.','No public fixtures/contracts/API/error kinds adopted.','No Rust/loop/work/layout/allocation conformance established.','No absolute shift by huge K; powers are limited to magnitude6000 and huge-gap Add neighbors are symbolic.','Metadata joins cover every64-bit boundary inside the proposed2053-bit graph/temporary envelope, not unspecified behavior beyond E33 capacity.','Reproduction checks the same independent reference producer, not candidate implementation correctness.']}
 if args.output:
  out=args.output
  out.mkdir(exist_ok=False)
  for name,data in [('cases.jsonl',payload),('manifest.json',(json.dumps(spec,indent=2,sort_keys=True)+'\n').encode())]:
   with(out/name).open('xb')as f:f.write(data)
  evidence={'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'manifest_sha256':sha((out/'manifest.json').read_bytes()),'case_count':len(rows),'operation_counts':counts,'oracle_self_checks':dict(CHECKS),'candidate_implementation_executed':False,'reference_inputs':'constructed mathematical controls only; no external financial/implementation inputs','all_powers_bounded_relative':True}
  with(out/'evidence.json').open('x')as f:json.dump(evidence,f,indent=2,sort_keys=True);f.write('\n')
  print(json.dumps({'output':str(out),'manifest_sha256':evidence['manifest_sha256'],'cases_sha256':spec['cases_sha256'],'count':len(rows),'counts':counts,'evidence_sha256':sha((out/'evidence.json').read_bytes())},sort_keys=True))
 else:
  out=args.references.resolve(strict=True);saved=json.loads((out/'manifest.json').read_bytes())
  assert saved==spec,'reference manifest/source changed'
  assert (out/'cases.jsonl').read_bytes()==payload,'reference case bytes differ'
  print(json.dumps({'status':'PASS_REFERENCE_REPRODUCTION','manifest_sha256':sha((out/'manifest.json').read_bytes()),'cases_sha256':sha(payload),'case_count':len(rows),'operation_counts':counts,'oracle_self_checks':dict(CHECKS),'candidate_implementation_executed':False},sort_keys=True))
if __name__=='__main__':main()
