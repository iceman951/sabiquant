#!/usr/bin/env python3
"""Independent comparison of bounded scalar output to preimplementation references."""
from pathlib import Path
import argparse,collections,csv,hashlib,json
def sha(b):return hashlib.sha256(b).hexdigest()
def words(s,limit):
 a=s.split(',');assert 1<=len(a)<=limit and all(len(w)==16 for w in a)
 n=sum(int(w,16)<<(64*j)for j,w in enumerate(a))
 assert len(a)==1 or int(a[-1],16)!=0
 return n
def point(r,side,p):
 s=int(r[side+'_sign']);m=words(r[side+'_m_words'],16);ks=int(r[side+'_K_sign']);km=words(r[side+'_K_words'],33)
 assert s in[-1,0,1]and ks in[-1,0,1]and bool(ks)==bool(km)
 k=ks*km
 if m:assert s and m.bit_length()==p
 else:assert s==k==0
 return(s,m,k)
def ref(a):return(a['sign'],int(a['m'],16),int(a['K'],16))
def compare(a,b):
 sa,ma,ka=a;sb,mb,kb=b
 if sa!=sb:return(sa>sb)-(sa<sb)
 if not sa:return 0
 c=((ka>kb)-(ka<kb))if ka!=kb else((ma>mb)-(ma<mb))
 return c*sa


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
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--repo',required=True,type=Path);ap.add_argument('--references',required=True,type=Path);ap.add_argument('--candidate',required=True,type=Path);ap.add_argument('--output',required=True,type=Path);args=ap.parse_args()
 args.output=output_path(ap,args.repo,args.output)
 P=args.references.resolve(strict=True)
 expected=[json.loads(l)for l in(P/'cases.jsonl').read_text().splitlines()]
 with args.candidate.open()as f:rows=list(csv.DictReader(f,delimiter='\t'))
 assert len(rows)==len(expected)==11727
 errors=[];counts=collections.Counter();widened=collections.Counter();status=collections.Counter();work=[]
 for e,r in zip(expected,rows):
  counts[e['op']]+=1;status[r['status']]+=1
  try:
   assert e['id']==r['id']and e['op']==r['op']
   used=int(r['work_used']);assert 0<=used<1<<64;work.append(used)
   o=e['expected'];p=e['p']
   if 'precondition_error'in o:
    assert r['status']==o['precondition_error']
    assert all(r[x]=='_'for x in r if x not in ['id','op','status','work_used'])
   elif e['op']=='export_binary64':
    assert r['status']=='ok'
    for side in ['lower','upper']:
     a=o[side];want=a['bits']if a['status']=='finite'else'unavailable'
     assert r[side+'_binary64_bits']==want,(side,want,r[side+'_binary64_bits'])
    assert all(r[x]=='_'for x in r if x not in ['id','op','status','work_used','lower_binary64_bits','upper_binary64_bits'])
   else:
    assert r['status']=='ok',r['status']
    lo,hi=point(r,'lower',p),point(r,'upper',p);a,b=ref(o['lower']),ref(o['upper'])
    assert compare(lo,hi)<=0
    if e['op']=='add':
     assert compare(lo,a)<=0 and compare(hi,b)>=0,('noncontainment',lo,hi,a,b)
     if lo!=a or hi!=b:widened[str(p)]+=1
    else:assert lo==a and hi==b,('exact_pair_mismatch',lo,hi,a,b)
    assert r['lower_binary64_bits']==r['upper_binary64_bits']=='_'
  except(AssertionError,ValueError,KeyError)as exc:errors.append({'id':e['id'],'op':e['op'],'message':str(exc),'actual':r,'expected':e})
 report={'status':'PASS'if not errors else'FAIL','cases':len(rows),'operation_counts':dict(counts),'candidate_status_counts':dict(status),'add_widened_but_containing_by_precision':dict(widened),'work_used_min':min(work),'work_used_max':max(work),'failures':len(errors),'first_failures':errors[:25],'candidate_path':str(args.candidate),'candidate_sha256':sha(args.candidate.read_bytes()),'references_sha256':sha((P/'cases.jsonl').read_bytes()),'comparator_sha256':sha(Path(__file__).read_bytes()),'scope':'Generic private primitive arithmetic output only; no full financial producer, work-budget completeness, stack or allocation conformance.'}
 with args.output.open('x')as f:json.dump(report,f,indent=2,sort_keys=True);f.write('\n')
 print(json.dumps({k:v for k,v in report.items()if k!='first_failures'},sort_keys=True))
 if errors:raise SystemExit(1)
if __name__=='__main__':main()
