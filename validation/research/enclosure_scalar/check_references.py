#!/usr/bin/env python3
"""Independent containment/tightness audit of private reference records.
Does not calculate candidate bounded outputs or import the reference generator.
"""
from pathlib import Path
from fractions import Fraction as Q
import argparse,collections,hashlib,json
S=1<<63;MAX=0x7fefffffffffffff
C=collections.Counter()
def h(s):return int(s,16)
def parts(a):return a['sign'],h(a['m']),h(a['K'])
def val(a,base,p):
 sign,m,k=parts(a)
 if not m:return Q(0)
 e=k-p+1-base;assert abs(e)<=6000
 return sign*Q(m)*(Q(2)**e)
def is_adjacent(lo,hi,p):
 sl,ml,kl=parts(lo);sh,mh,kh=parts(hi)
 if sl==sh==1:return(kl==kh and mh==ml+1)or(kh==kl+1 and ml==(1<<p)-1 and mh==1<<(p-1))
 if sl==sh==-1:
  return(kl==kh and ml==mh+1)or(kl==kh+1 and mh==(1<<p)-1 and ml==1<<(p-1))
 return False
def check_pair(pair,x,base,p):
 lo,hi=pair['lower'],pair['upper'];a,b=val(lo,base,p),val(hi,base,p)
 assert a<=x<=b
 if lo==hi:assert x==a
 else:assert a<x<b and is_adjacent(lo,hi,p)
 C['independent_tight_fraction_checks']+=1

def f64(bits):
 sign=-1 if bits&S else 1;field=(bits>>52)&2047;sig=bits&((1<<52)-1);assert field<2047
 if field:sig|=1<<52
 e=field-1075 if field else -1074
 return sign*sig*Q(2)**e
def fsucc(b):return 0 if b==S|1 else b-1 if b&S else b+1

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
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--repo',required=True,type=Path);parser.add_argument('--references',required=True,type=Path);parser.add_argument('--output',required=True,type=Path);args=parser.parse_args()
 args.output=output_path(parser,args.repo,args.output)
 O=args.references.resolve(strict=True)
 for line in(O/'cases.jsonl').read_text().splitlines():
  row=json.loads(line);p=row['p'];inp=row['input'];out=row['expected'];op=row['op'];C['cases']+=1;C[op]+=1
  def valid(a):
   sign,m,k=parts(a);assert abs(k).bit_length()<=2112
   if m:assert sign in(-1,1) and m.bit_length()==p
   else:assert sign==k==0
  if op=='normalize':
   n,d,e=h(inp['N']),h(inp['D']),h(inp['e']);assert max(n.bit_length(),d.bit_length())<=2176
   if d==0:assert out=={'precondition_error':'zero_denominator'};C['independent_zero_denominator_checks']+=1;continue
   x=Q(-n if inp['negative']else n,d);check_pair(out,x,e,p)
  elif op in('mul','div'):
   a,b=inp['a'],inp['b'];valid(a);valid(b);sa,ma,ka=parts(a);sb,mb,kb=parts(b)
   if op=='div'and mb==0:assert out=={'precondition_error':'zero_denominator'};C['independent_zero_denominator_checks']+=1;continue
   e=(ka-p+1)+(kb-p+1)if op=='mul'else(ka-p+1)-(kb-p+1)
   x=Q(sa*sb*ma*mb)if op=='mul'else Q(sa*sb*ma,mb)
   check_pair(out,x,e,p)
  elif op=='add':
   a,b=inp['a'],inp['b'];valid(a);valid(b);sa,ma,ka=parts(a);sb,mb,kb=parts(b)
   if not ma or not mb or abs(ka-kb)<=4096:
    base=max(ka,kb)if ma and mb else ka if ma else kb
    check_pair(out,val(a,base,p)+val(b,base,p),base,p)
   else:
    big,small=(a,b)if ka>kb else(b,a);ss,ms,ks=parts(small);_,md,kd=parts(big)
    assert kd-ks>p+2 and ms and md and is_adjacent(out['lower'],out['upper'],p)
    assert out['lower']==big if ss>0 else out['upper']==big
    C['independent_symbolic_neighbor_checks']+=1
  elif op=='export_binary64':
   a=inp['point'];valid(a);sign,m,k=parts(a);lo,hi=out['lower'],out['upper']
   for endpoint in [lo,hi]:
    assert endpoint['status'] in ['finite','unavailable_finite_bound']
    if endpoint['status']=='finite':assert h(endpoint['bits'])!=S,'negative zero must be canonical positive'
   finite=lambda z:z['status']=='finite';avail=finite(lo)and finite(hi);assert out['finite_pair']==avail
   if not m:assert lo==hi=={'status':'finite','bits':'0000000000000000'}
   elif abs(k)<=4096:
    x=val(a,0,p)
    if finite(lo):assert f64(h(lo['bits']))<=x
    else:assert x< -f64(MAX)and hi=={'status':'finite','bits':f'{S|MAX:016x}'}
    if finite(hi):assert x<=f64(h(hi['bits']))
    else:assert x>f64(MAX)and lo=={'status':'finite','bits':f'{MAX:016x}'}
    if avail and lo!=hi:assert fsucc(h(lo['bits']))==h(hi['bits'])
   elif k>1023:
    if sign>0:assert lo=={'status':'finite','bits':f'{MAX:016x}'}and hi=={'status':'unavailable_finite_bound'}
    else:assert lo=={'status':'unavailable_finite_bound'}and hi=={'status':'finite','bits':f'{S|MAX:016x}'}
   elif k< -1074:
    if sign>0:assert lo=={'status':'finite','bits':'0000000000000000'}and hi=={'status':'finite','bits':'0000000000000001'}
    else:assert lo=={'status':'finite','bits':'8000000000000001'}and hi=={'status':'finite','bits':'0000000000000000'}
   else:raise AssertionError('huge export classification')
   C['independent_export_checks']+=1
  else:raise AssertionError(op)
  if op!='export_binary64':valid(out['lower']);valid(out['upper'])
 report={'status':'PASS_INDEPENDENT_REFERENCE_CONTAINMENT_AND_TIGHTNESS_AUDIT','counts':dict(C),'candidate_outputs_read':False,'cases_sha256':hashlib.sha256((O/'cases.jsonl').read_bytes()).hexdigest(),'checker_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'method':'Exact Fraction containment and independently expressed p-grid adjacency; huge-gap symbolic dominance; finite IEEE lattice/direction tests. No normalization implementation imported and no absolute huge powers.'}
 with(args.output).open('x')as f:json.dump(report,f,indent=2,sort_keys=True);f.write('\n')
 print(json.dumps(report,sort_keys=True))

if __name__=='__main__':main()
