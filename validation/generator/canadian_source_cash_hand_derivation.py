"""Blind source/cash hand arithmetic from contract equations only.
Does NOT import numeric producer or sibling source/cash generator/expectations.
"""
from fractions import Fraction
from pathlib import Path
import json,hashlib,argparse
F=Fraction
ROOT=Path(__file__).resolve().parents[2]
CONTRACT_SHA='4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3'
if hashlib.sha256((ROOT/'docs/M8_CANADIAN_CONTRACT.md').read_bytes()).hexdigest()!=CONTRACT_SHA:
 raise ValueError('adopted Canadian contract mismatch')
MODES=('TowardZero','AwayFromZero','TowardPositiveInfinity','TowardNegativeInfinity','NearestTiesEven','NearestTiesAwayFromZero','Exact')

def round_quantum(x,q,mode):
 t=x/q;a,b=divmod(t.numerator,t.denominator)
 if mode=='Exact':
  if b:raise ValueError('Inexact')
  k=a
 elif mode=='TowardNegativeInfinity':k=a
 elif mode=='TowardPositiveInfinity':k=a+bool(b)
 elif mode=='TowardZero':k=a+(bool(b) and t<0)
 elif mode=='AwayFromZero':k=a+(bool(b) and t>0)
 else:
  c=2*b-t.denominator
  k=a+(c>0 or (c==0 and ((a%2!=0) if mode=='NearestTiesEven' else t>0)))
 return k*q

def rat(v):return {'num':str(v.numerator),'den':str(v.denominator)}
def dec(v):
 d=v.denominator;s=0
 while d>1 and s<=38 and (10**s)%d:s+=1
 if s>38 or (10**s)%d:return None
 c=v.numerator*((10**s)//d)
 while s and c%10==0:c//=10;s-=1
 return {'coefficient':str(c),'scale':s,'admitted':-(1<<127)<=c<(1<<127)}

records=[]
def add(id,formula,v):records.append({'id':id,'formula':formula,'exact':rat(v),'canonical_decimal_if_terminating':dec(v)})
# Literal day counts and algebra are deliberately entered independently.
add('LATE182','5*182/365',F(182,73))
add('LATE183','5*(1/2-1/365)',F(363,146))
add('AI_DROP','LATE183-LATE182',-F(1,146))
add('COMPLETED184','5/2 (full cell, proper branch forbidden)',F(5,2))
add('COMPLETED181','5/2 (full cell, proper branch forbidden)',F(5,2))
add('MONTHLY_SHORT_VALUE','5/12*30/31',F(25,62))
add('MONTHLY_SHORT_PAYABLE','5*30/365',F(30,73))
add('MONTHLY_AI_29','5/12*29/31',F(145,372))
add('MONTHLY_365_DISCRIMINATOR','5*29/365',F(29,73))
add('SAME_CELL_COMPOSITE_VALUE','5/12*18/31',F(15,62))
add('SAME_CELL_COMPOSITE_PAYABLE','5*18/365',F(18,73))
add('LONG_FIRST_VALUE','5/2*(2+151/182)',F(2575,364))
add('LONG_FIRST_PAYABLE','5+5*151/365',F(516,73))
add('SKIPPED_Q_VALUE','5/2*(151/182+31/184)',F(5,2)*(F(151,182)+F(31,184)))
add('SKIPPED_Q_AI','5*(151+31)/365',F(182,73))
add('AI_TIE_SOURCE','(73/8)/365',F(1,40))
add('AI_TIE_VALUE_DIFFERENT','(73/8)/(2*184)',F(73,2944))
# All seven signed modes, positive pure cash and signed dirty/evidence.
for mode in MODES:
 for sign in (-1,1):
  x=sign*F(1,40);q=F(1,100)
  try:r=round_quantum(x,q,mode);answer={'rounded':rat(r),'decimal':dec(r)}
  except ValueError:answer={'error':'Inexact','stage':'cash total','endpoint':'point'}
  records.append({'id':'SIGNED-TIE-'+mode+'-'+str(sign),'formula':'N1 * supplied price(sign*2.5)/100 at cent','input':{'exact_cash':rat(x),'quantum':rat(q),'mode':mode},'answer':answer})
# Full39 scale endpoints through independent hand scalar, not a copied source generator.
for scale in range(39):
 N=F(1,10**scale)
 for q in (F(1),F(1,100),F(1,10**38)):
  r=round_quantum(N*F(1,40)/100,q,'NearestTiesEven')
  records.append({'id':f'SCALE-{scale}-{q.denominator}','formula':'N*exact AI(1/40)/100 one quantum, no AI rounding','input':{'nominal':rat(N),'AI':rat(F(1,40)),'quantum':rat(q)},'answer':rat(r)})
# Capital/AI .025 each roundeven down separately; total .05 exactly.
assert round_quantum(F(1,40),F(1,100),'NearestTiesEven')*2==F(1,25)
assert round_quantum(F(1,20),F(1,100),'NearestTiesEven')==F(1,20)
add('INVOICE_COMPONENTS','.025 roundeven cent + .025 roundeven cent',F(1,25))
add('INVOICE_TOTAL','(.025+.025) one roundeven cent',F(1,20))
# Wide canonical intermediate cancellation, no admission until final net.
AI=F(1<<127);X=-AI
for N,mode in ((F(100),'Exact'),(F((1<<127)-1),'NearestTiesEven')):
 cap=round_quantum(N*X/100,F(1),mode);acc=round_quantum(N*AI/100,F(1),mode)
 assert cap+acc==0
 records.append({'id':'WIDE-CANCEL-'+mode,'formula':'N*(-2^127)/100 and N*(2^127)/100 quantized separately; exact net0','input':{'N':rat(N),'AI':rat(AI),'X':rat(X),'mode':mode},'wide_capital':rat(cap),'wide_accrued':rat(acc),'net':rat(cap+acc)})
assert dec(AI)['admitted'] is False
MAX=(1<<127)-1
assert dec(F(MAX))['admitted'] and dec(F(MAX-1))['admitted']
assert dec(F(2*MAX-1,2))['admitted'] is False
records.append({'id':'RANGE-HOLE','formula':'Endpoints MAX-1,MAX admitted; midpoint MAX-1/2 not admitted; return endpoint ambiguity without interior certificate','lower':rat(F(MAX-1)),'upper':rat(F(MAX)),'interior_hole':rat(F(2*MAX-1,2))})
# IEEE source-AI once-rounded input is supplied price evidence, not exact money.
add('BINARY_AI_PRICE_DISCRIMINATOR','RN(1/40)=3602879701896397/2^57, then one cash round at1e-38',round_quantum(F(3602879701896397,1<<57),F(1,10**38),'NearestTiesEven'))
# Crosscheck signed rounding without reusing branch implementation: monotone bracketing,
# exact integrality, sign symmetry for symmetric modes, integer inputs fixed.
controls=0
for n in range(-101,102):
 for den in (1,2,3,7,40,365):
  x=F(n,den)
  for mode in MODES[:-1]:
   r=round_quantum(x,F(1,100),mode);assert (r*100).denominator==1
   if mode.startswith('Nearest'):assert abs(r-x)<=F(1,200)
   if mode in ('TowardZero','AwayFromZero','NearestTiesEven','NearestTiesAwayFromZero'):assert round_quantum(-x,F(1,100),mode)==-r
   controls+=1
out={'status':'PASS','scope':'Blind hand arithmetic, not production/public execution','records':records,'record_count':len(records),'signed_quantizer_crosschecks':controls,'notes':['All formulas independently entered from contract; no sibling generator/fixture/expected input read.','Continuous Exact is mathematical divisibility of every real stage input, not endpoint-only enumeration.','Rust body remains future public API binding; not counted as runtime tests before production.']}
parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');parser.add_argument('--output',type=Path,default=ROOT/'validation/hand/canadian_source_cash_hand_derivations.json');args=parser.parse_args()
expected=(json.dumps(out,indent=2)+'\n').encode()
if args.check:
 if args.output.read_bytes()!=expected:raise ValueError('hand derivation reproduction mismatch')
else:
 with args.output.open('xb') as f:f.write(expected)
print(json.dumps({'status':'PASS','mode':'check' if args.check else 'exclusive-create','records':len(records),'signed_quantizer_crosschecks':controls,'sha256':hashlib.sha256(expected).hexdigest()}))
