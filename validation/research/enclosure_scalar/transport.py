#!/usr/bin/env python3
"""Lossless text transport of immutable PRIVATE reference records; no arithmetic."""
from pathlib import Path
import argparse,hashlib,json,collections
INPUT=['id','op','p','negative_bool','N_words','D_words','e_sign','e_words']
for prefix in ['a','b','point']:INPUT += [prefix+'_sign',prefix+'_m_words',prefix+'_K_sign',prefix+'_K_words']
OUTPUT=['precondition_error']
for prefix in ['reference_lower','reference_upper']:OUTPUT += [prefix+'_sign',prefix+'_m_words',prefix+'_K_sign',prefix+'_K_words']
OUTPUT+=['export_lower_status','export_lower_bits','export_upper_status','export_upper_bits','export_finite_pair','relation','families','scope','proof']
HEADER=INPUT+OUTPUT
canonical=lambda x:json.dumps(x,sort_keys=True,separators=(',',':'))
sha=lambda x:hashlib.sha256(x).hexdigest()
def words(h):
 n=int(h,16);assert n>=0
 return ','.join(f'{(n>>(64*j))&((1<<64)-1):016x}'for j in range(max(1,(n.bit_length()+63)//64)))
def unwords(s):
 a=s.split(',');assert all(len(w)==16 for w in a)
 n=sum(int(w,16)<<(64*j)for j,w in enumerate(a));assert s==words(format(n,'x'))
 return format(n,'x')
def signed(v):
 n=int(v,16);return str((n>0)-(n<0)),words(format(abs(n),'x'))
def unsign(sign,m):
 n=int(unwords(m),16);s=int(sign);assert s in[-1,0,1]and(bool(n)==bool(s))
 return('-'if s<0 else'')+format(n,'x')
def desc(c,prefix,a):
 c[prefix+'_sign']=str(a['sign']);c[prefix+'_m_words']=words(a['m']);s,w=signed(a['K']);c[prefix+'_K_sign']=s;c[prefix+'_K_words']=w
def undesc(c,prefix):return{'sign':int(c[prefix+'_sign']),'m':unwords(c[prefix+'_m_words']),'K':unsign(c[prefix+'_K_sign'],c[prefix+'_K_words'])}
def encode(r):
 c={x:'_'for x in HEADER};c.update(id=r['id'],op=r['op'],p=str(r['p']),relation=r['relation'],families=','.join(r['families']),scope=r['scope'])
 i=r['input'];e=r['expected']
 if r['op']=='normalize':
  c['N_words']=words(i['N']);c['D_words']=words(i['D']);c['negative_bool']=str(int(i['negative']));c['e_sign'],c['e_words']=signed(i['e'])
 elif r['op']=='export_binary64':desc(c,'point',i['point'])
 else:desc(c,'a',i['a']);desc(c,'b',i['b'])
 if 'precondition_error'in e:c['precondition_error']=e['precondition_error']
 elif r['op']=='export_binary64':
  for side in['lower','upper']:
   c['export_'+side+'_status']=e[side]['status'];c['export_'+side+'_bits']=e[side].get('bits','_')
  c['export_finite_pair']=str(int(e['finite_pair']))
 else:
  desc(c,'reference_lower',e['lower']);desc(c,'reference_upper',e['upper']);c['proof']=e.get('proof','_')
 assert all('\t'not in s and'\n'not in s and'\r'not in s for s in c.values())
 return c
def decode(c,full=True):
 r={'id':c['id'],'op':c['op'],'p':int(c['p'])}
 if r['op']=='normalize':r['input']={'N':unwords(c['N_words']),'D':unwords(c['D_words']),'negative':bool(int(c['negative_bool'])),'e':unsign(c['e_sign'],c['e_words'])}
 elif r['op']=='export_binary64':r['input']={'point':undesc(c,'point')}
 else:r['input']={'a':undesc(c,'a'),'b':undesc(c,'b')}
 if not full:return r
 r.update(relation=c['relation'],families=c['families'].split(','),scope=c['scope'])
 if c['precondition_error']!='_':e={'precondition_error':c['precondition_error']}
 elif r['op']=='export_binary64':
  e={'finite_pair':bool(int(c['export_finite_pair']))}
  for side in['lower','upper']:
   e[side]={'status':c['export_'+side+'_status']}
   if c['export_'+side+'_bits']!='_':e[side]['bits']=c['export_'+side+'_bits']
 else:
  e={'lower':undesc(c,'reference_lower'),'upper':undesc(c,'reference_upper')}
  if c['proof']!='_':e['proof']=c['proof']
 r['expected']=e;return r


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
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--repo',required=True,type=Path);parser.add_argument('--references',required=True,type=Path);parser.add_argument('--output',required=True,type=Path);parser.add_argument('--check',action='store_true');args=parser.parse_args()
 O=output_path(parser,args.repo,args.output,directory=True)
 if not args.check:O.mkdir(exist_ok=False)
 original=(args.references.resolve(strict=True)/'cases.jsonl').read_bytes();rows=[json.loads(l)for l in original.splitlines()];cells=[encode(r)for r in rows]
 full=('\t'.join(HEADER)+'\n'+''.join('\t'.join(c[x]for x in HEADER)+'\n'for c in cells)).encode()
 inputs=('\t'.join(INPUT)+'\n'+''.join('\t'.join(c[x]for x in INPUT)+'\n'for c in cells)).encode()
 inputjson=''.join(canonical({k:r[k]for k in['id','op','p','input']})+'\n'for r in rows).encode()
 restored=''.join(canonical(decode(c))+'\n'for c in cells).encode();assert restored==original
 assert ''.join(canonical(decode(c,False))+'\n'for c in cells).encode()==inputjson
 data={'transport.tsv':full,'inputs.tsv':inputs,'inputs.jsonl':inputjson}
 report={'scope':'PRIVATE_PRIMITIVE_RESEARCH_ONLY lossless text transport; input-only files contain no reference output values.','cases_sha256':sha(original),'case_count':len(rows),'column_count':len(HEADER),'input_column_count':len(INPUT),'columns':HEADER,'input_columns':INPUT,'absent':'_','word_encoding':'minimal little-endian comma-separated16hex-digit u64 words; zero has one zero word; no high zero limbs','signed_metadata':'sign -1/0/1 plus unsigned limbs; exponent zero sign0','raw_negative_bool':'0/1; preserves negative raw zero independently of magnitude','full_roundtrip_all_cells_equal':restored==original,'input_roundtrip_all_cells_equal':True,'files':{n:{'bytes':len(b),'sha256':sha(b)}for n,b in data.items()},'transport_script_sha256':sha(Path(__file__).read_bytes())}
 if args.check:
  assert json.loads((O/'transport-manifest.json').read_bytes())==report
  for n,b in data.items():assert(O/n).read_bytes()==b
  print(json.dumps({'status':'PASS_TRANSPORT_REPRODUCTION_AND_BIJECTION','case_count':len(rows),'cases_sha256':sha(original),'files':report['files']},sort_keys=True))
 else:
  for n,b in data.items():
   with(O/n).open('xb')as f:f.write(b)
  with(O/'transport-manifest.json').open('x')as f:json.dump(report,f,indent=2,sort_keys=True);f.write('\n')
  print(json.dumps({'status':'CREATED_TRANSPORT','case_count':len(rows),'columns':len(HEADER),'input_columns':len(INPUT),'manifest_sha256':sha((O/'transport-manifest.json').read_bytes()),'files':report['files']},sort_keys=True))
if __name__=='__main__':main()
