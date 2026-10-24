#!/usr/bin/env python3
"""Independent Canadian source/date/cash oracle from the adopted whole contract.

AUTHORING DRAFT: no Canadian production/QuantLib implementation or Rust financial
expectations read. Standard-library dates, exact fractions and integer rounding.
Outputs are exclusive; accepted artifacts require root review/whole gate.
"""
from __future__ import annotations
import argparse
import bisect
import calendar
from collections import Counter
from dataclasses import dataclass
from datetime import date, datetime, timedelta, timezone
from fractions import Fraction as F
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import struct
import sys

CONTRACT_SHA='4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3'
SCHEMA_SHA='1d5cb99720285f15f39b5234e4077f739c6968622ecbd548401319700a807a4d'
REGISTRY_SHA='fe016372ae2239d26fa501079c8de04aace8d25f2c2aa60d8a42b785964fd430'
APPENDIX_SHA='74fcfa2db8ae9b624a66a94445b0b59c7a54920cd7d4bd63e50b25425b353e1a'
SOURCE_VERSION='CA-IIAC13-fixed-2026-10-23-local-v1'
POLICY='CanonicalIiac13LocalV1'
MAX=(1<<127)-1
MODES=('TowardZero','AwayFromZero','TowardPositiveInfinity','TowardNegativeInfinity',
       'NearestTiesEven','NearestTiesAwayFromZero','Exact')
SELECTIONS=('Explicit:Compound','Explicit:MoneyMarket','LastCouponMoneyMarket')
HERE=Path(__file__).resolve().parent


def canonical_json(value):return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True)
def sha(data):return hashlib.sha256(data).hexdigest()
def fnv(data):
    h=0xcbf29ce484222325
    for b in data:h=((h^b)*0x100000001b3)&((1<<64)-1)
    return f'{h:016x}'
def rat(x):return f'{x.numerator}/{x.denominator}'
def dec(text):
    n,s=map(int,text.split(':'));assert 0<=s<=38 and -(1<<127)<=n<=MAX
    if not n:return 0,0
    while s and n%10==0:n//=10;s-=1
    return n,s
def dectext(pair):return f'{pair[0]}:{pair[1]}'
def amount(n,s):
    if not n:return '0:0'
    while s and n%10==0:n//=10;s-=1
    return f'{n}:{s}'
def value(pair):return F(pair[0],10**pair[1])
def dates(text):return [] if text=='empty' else [date.fromisoformat(x) for x in text.split(',')]
def datevec(values):return ','.join(x.isoformat() for x in values) if values else 'empty'
def pow2(e):return F(1<<e) if e>=0 else F(1,1<<-e)


class ModelError(Exception):
    def __init__(self,suffix,**context):self.kind='canadian.'+suffix;self.context=context

def fail(suffix,**context):raise ModelError(suffix,**context)
def encode_context(context):
    def scalar(x):
        if isinstance(x,bool):return 'true' if x else 'false'
        if isinstance(x,int):return str(x)
        if isinstance(x,date):return x.isoformat()
        return 'x:'+str(x).encode().hex()
    return ';'.join(k+'='+scalar(v) for k,v in sorted(context.items()))


def shift_months(anchor,months,eom):
    serial=12*(anchor.year-1)+anchor.month-1+months
    year,month=divmod(serial,12);year+=1;month+=1
    if year<1 or year>9999:raise ValueError(year)
    target_last=calendar.monthrange(year,month)[1]
    anchor_last=calendar.monthrange(anchor.year,anchor.month)[1]
    day=target_last if eom=='Preserve' and anchor.day==anchor_last else min(anchor.day,target_last)
    return date(year,month,day)


def ieee_bits(x):
    """Exact rational -> one IEEE64 nearest/even; A/AI outputs are finite normal."""
    if not x:return '0000000000000000'
    sign=int(x<0);x=abs(x)
    e=x.numerator.bit_length()-x.denominator.bit_length()
    if x<pow2(e):e-=1
    step=1074 if e<-1022 else 52-e
    scaled=x*pow2(step);q,r=divmod(scaled.numerator,scaled.denominator)
    q+=int(2*r>scaled.denominator or (2*r==scaled.denominator and q&1))
    if e<-1022:bits=q
    else:
        if q==1<<53:q>>=1;e+=1
        bits=(0x7ff<<52) if e>1023 else ((e+1023)<<52)|(q-(1<<52))
    bits|=sign<<63
    # Independent standard-library Fraction conversion corroborates exact integer RNE.
    try:reference=struct.pack('>d',float(-x if sign else x)).hex()
    except OverflowError:reference=f'{(sign<<63)|(0x7ff<<52):016x}'
    assert f'{bits:016x}'==reference
    return f'{bits:016x}'


def binary_fraction(bits):
    b=int(bits,16);sign=-1 if b>>63 else 1;e=(b>>52)&2047;s=b&((1<<52)-1)
    assert e!=2047
    if e:s|=1<<52;power=e-1075
    else:power=-1074
    return sign*s*pow2(power)


def chi(Q,t):
    if t==Q[-1]:return F(len(Q)-1)
    i=bisect.bisect_right(Q,t)-1
    assert 0<=i<len(Q)-1 and Q[i]<=t<Q[i+1]
    return i+F((t-Q[i]).days,(Q[i+1]-Q[i]).days)


def source(Q,a,b,C,f):
    """Intersection route; separate common-integer source route crosschecks each result."""
    assert Q[0]<=a<=b<=Q[-1]
    start=bisect.bisect_right(Q,a)-1
    assert 0<=start<len(Q)-1
    if a==b:
        pair=(Q[start],Q[start+1])
        return {'V':F(0),'P':F(0),'interval':(a,b),'pairs':(pair,pair),'full':0,'parts':()}
    stop=bisect.bisect_left(Q,b)-1
    assert start<=stop<len(Q)-1
    V=P=F(0);full=0;parts=[]
    for i in range(start,stop+1):
        left=max(a,Q[i]);right=min(b,Q[i+1]);d=(right-left).days;s=(Q[i+1]-Q[i]).days
        assert 0<d<=s
        if d==s:V+=C/f;P+=C/f;full+=1
        else:
            parts.append((d,s));V+=C*F(d,f*s)
            assert f*d!=365
            P+=C*F(d,365) if f*d<365 else C*(F(1,f)-F(s-d,365))
    assert len(parts)<=2
    V2=C/f*(chi(Q,b)-chi(Q,a))
    integer=365*full+sum(f*d if f*d<365 else 365-f*(s-d) for d,s in parts)
    P2=C*F(integer,365*f)
    assert V==V2 and P==P2 and V>=0 and P>=0
    assert V.numerator.bit_length()<=160 and V.denominator.bit_length()<=148
    assert P.numerator.bit_length()<=160 and P.denominator.bit_length()<=148
    return {'V':V,'P':P,'interval':(a,b),'pairs':((Q[start],Q[start+1]),(Q[stop],Q[stop+1])),
            'full':full,'parts':tuple(parts)}


def period_kind(Q,a,b,single):
    aq=a in Q;bq=b in Q
    if aq and bq and Q.index(b)-Q.index(a)==1:return 'Regular'
    if single and not aq and not bq:return 'Composite'
    length=chi(Q,b)-chi(Q,a)
    assert length!=1
    return 'Short' if length<1 else 'Long'


def schedule(inp):
    D=date.fromisoformat(inp['dated']);M=date.fromisoformat(inp['stated_maturity'])
    Q=dates(inp['quasi']);actual=dates(inp['coupon_dates']);f=int(inp['frequency']);cap=int(inp['date_cap'])
    assert f in (2,4,12) and inp['eom'] in ('Preserve','Ignore')
    if not 2<=cap<=20000:fail('invalid_date_cap',cap=cap,minimum=2,maximum=20000)
    if len(Q)>cap:fail('too_many_quasi_dates',count=len(Q),cap=cap)
    if len(Q)<2:fail('too_few_quasi_dates',count=len(Q),minimum=2)
    if len(actual)>cap:fail('too_many_coupons',count=len(actual),cap=cap)
    if not actual:fail('no_coupons')
    if D>=M:fail('accrual_not_before_maturity',accrual_start=D,stated_maturity=M)
    for i in range(1,len(Q)):
        if Q[i]<=Q[i-1]:fail('quasi_date_order',index=i,previous=Q[i-1],date=Q[i])
    if not (Q[0]<=D<Q[1] and Q[-2]<M<=Q[-1]):
        fail('quasi_coverage',accrual_start=D,stated_maturity=M,first=Q[0],next=Q[1],penultimate=Q[-2],last=Q[-1])
    for i,q in enumerate(Q):
        try:expected=shift_months(Q[-1],-(12//f)*(len(Q)-1-i),inp['eom'])
        except ValueError as e:fail('date',index=i,phase='quasi cycle',cause_variant='YearOutOfRange',cause_year=e.args[0])
        if q!=expected:fail('quasi_date_mismatch',index=i,supplied=q,expected=expected,anchor=Q[-1],frequency=f,end_of_month=inp['eom'])
    if actual[-1]!=M:fail('coupon_boundary',last_coupon=actual[-1],stated_maturity=M)
    for i in range(1,len(actual)):
        if actual[i]<=actual[i-1]:fail('coupon_date_order',index=i,previous=actual[i-1],date=actual[i])
    if actual[0]<=D:fail('coupon_before_accrual',index=0,date=actual[0],accrual_start=D)
    for i,q in enumerate(actual[:-1]):
        if q not in Q:fail('coupon_off_cycle',index=i,date=q)
    index={q:i for i,q in enumerate(Q)}
    for i in range(1,len(actual)-1):
        if index[actual[i]]!=index[actual[i-1]]+1:
            fail('irregular_interior_period',index=i,previous=actual[i-1],date=actual[i])
    kinds=[period_kind(Q,a,b,len(actual)==1) for a,b in zip([D]+actual[:-1],actual)]
    return {'D':D,'M':M,'Q':Q,'actual':actual,'f':f,'kinds':kinds,'L':D if len(actual)==1 else actual[-2]}


def instrument(inp):
    s=schedule(inp);I=date.fromisoformat(inp['issue']);T=date.fromisoformat(inp['true_maturity'])
    cp=dec(inp['coupon']);C=value(cp)
    assert inp['source_policy']==POLICY
    if C<0:fail('negative_coupon',coupon_per_100=dectext(cp))
    if s['D']>I:fail('dated_after_issue',accrual_start=s['D'],original_issue=I)
    if I>=s['actual'][0]:fail('issue_not_before_first_coupon',original_issue=I,first_coupon=s['actual'][0])
    if T<s['M']:fail('true_maturity_before_stated',true_maturity=T,stated_maturity=s['M'])
    s|={'I':I,'T':T,'C':C,'coupon':cp}
    s['sources']=[source(s['Q'],a,b,C,s['f']) for a,b in zip([s['D']]+s['actual'][:-1],s['actual'])]
    return s


def prepared(inp,bond):
    cap=int(inp['bundle_cap']);S=date.fromisoformat(inp['settlement'])
    if cap>20000:fail('invalid_bundle_cap',cap=cap,maximum=20000)
    if S<bond['I']:fail('settlement_before_issue',settlement=S,original_issue=bond['I'])
    if S>=bond['M']:fail('settlement_at_or_after_maturity',settlement=S,stated_maturity=bond['M'])
    indices=[i for i,d in enumerate(bond['actual']) if d>S]
    if len(indices)>cap:fail('too_many_bundles',count=len(indices),cap=cap)
    selection=inp['selection'];assert selection in SELECTIONS
    convention='MoneyMarket' if selection=='LastCouponMoneyMarket' and len(indices)==1 else 'Compound' if selection=='LastCouponMoneyMarket' else selection.split(':')[1]
    next_i=indices[0];start=bond['D'] if next_i==0 else bond['actual'][next_i-1]
    accrued=source(bond['Q'],start,S,bond['C'],bond['f'])
    A=accrued['V'];AI=A if bond['f']==12 else accrued['P']
    A_bits=ieee_bits(A);AI_bits=ieee_bits(AI)
    return {'S':S,'indices':indices,'convention':convention,'selection':selection,'next_i':next_i,
            'accrued':accrued,'A':A,'AI':AI,'A_bits':A_bits,'AI_bits':AI_bits,
            'quote_bits':A_bits if convention=='Compound' else AI_bits}


def reference_count(x,mode):
    lo=x.numerator//x.denominator;r=x.numerator-lo*x.denominator;d=x.denominator
    if mode=='Exact':return lo if not r else None
    if mode=='TowardZero':return int(x)
    if mode=='AwayFromZero':return -((-x.numerator+d-1)//d) if x<0 else (x.numerator+d-1)//d
    if mode=='TowardPositiveInfinity':return -((-x.numerator)//d)
    if mode=='TowardNegativeInfinity':return lo
    if mode=='NearestTiesEven':return lo+int(2*r>d or (2*r==d and lo&1))
    if mode=='NearestTiesAwayFromZero':return lo+int(2*r>d or (2*r==d and x>=0))
    raise AssertionError(mode)


def quantize(x,q,mode,stage):
    cq,sq=q;assert cq>0 and mode in MODES
    count=x/F(cq,10**sq);a=abs(count);n,r=divmod(a.numerator,a.denominator)
    if r and mode=='Exact':result=None
    else:
        complement=a.denominator-r
        increment=False if not r else {
            'TowardZero':False,'AwayFromZero':True,'TowardPositiveInfinity':count>=0,
            'TowardNegativeInfinity':count<0,
            'NearestTiesEven':r>complement or (r==complement and n&1),
            'NearestTiesAwayFromZero':r>=complement,'Exact':False}[mode]
        n+=bool(increment);result=-n if count<0 and n else n
    assert result==reference_count(count,mode)
    if result is None:fail('cash_arithmetic',stage=stage,endpoint='point',cause_variant='Inexact',cause_operation='quantize')
    return result*cq


def admit(c,scale,stage,endpoint,operation='quantize'):
    negative=c<0;n=abs(c)
    if not n:return '0:0'
    while scale and n%10==0:n//=10;scale-=1
    if n>(1<<127) if negative else n>MAX:
        fail('cash_arithmetic',stage=stage,endpoint=endpoint,cause_variant='OutOfRange',cause_operation=operation,cause_negative=negative)
    return f'{-n if negative else n}:{scale}'


def evidence(text):
    parts=text.split(':');kind=parts[0];assert kind in ('d','di','b','bi')
    declared=len(kind)==2
    if kind.startswith('d'):
        lo=value(dec(':'.join(parts[1:3])));hi=value(dec(':'.join(parts[3:5]))) if declared else lo
    else:
        lo=binary_fraction(parts[1]);hi=binary_fraction(parts[2]) if declared else lo
    assert lo<=hi
    return lo,hi,'UniqueForDeclaredEnclosure' if declared else 'ForSuppliedDecimals' if kind=='d' else 'ForSuppliedBinary64'


def geometry_output(g):
    a,b=g['interval'];pairs=g['pairs']
    return {'src_interval':a.isoformat()+':'+b.isoformat(),
            'src_quasi_pairs':','.join(x.isoformat()+':'+y.isoformat() for x,y in pairs),
            'src_full_count':str(g['full']),
            'src_day_ratios':','.join(f'{d}/{s}' for d,s in g['parts']) if g['parts'] else 'none'}


def common_output(b):
    out={'source_version':SOURCE_VERSION,'first_period':b['kinds'][0],'last_period':b['kinds'][-1],
         'first_coupon':b['actual'][0].isoformat(),'last_coupon_start':b['L'].isoformat(),
         'quasi_count':str(len(b['Q'])),'coupon_count':str(len(b['actual'])),'anchor':b['Q'][-1].isoformat()}
    if 'sources' in b:
        out|={'valuation_coupons':','.join(rat(x['V']) for x in b['sources']),
              'payable_coupons':','.join(rat(x['P']) for x in b['sources'])}
    return out


def plan_output(b,p):
    indices=p['indices'];next_i=p['next_i'];actual=b['actual'];convention=p['convention'];S=p['S']
    return {'next_coupon_index':str(next_i),'next_coupon_date':actual[next_i].isoformat(),
       'next_payment_date':(b['T'] if next_i==len(actual)-1 else actual[next_i]).isoformat(),
       'actual_accrual_start':p['accrued']['interval'][0].isoformat(),
       'valuation_ai':rat(p['A']),'settlement_ai':rat(p['AI']),
       'valuation_ai_bits':p['A_bits'],'settlement_ai_bits':p['AI_bits'],'quote_accrued_bits':p['quote_bits'],
       'resolved_convention':convention,'bundle_ordinals':','.join(map(str,indices)),
       'bundle_coefficients':','.join(rat(b['sources'][i]['V' if convention=='Compound' else 'P']+(100 if i==len(actual)-1 else 0)) for i in indices),
       'bundle_lambdas':','.join(rat(chi(b['Q'],actual[i])-chi(b['Q'],S)) for i in indices),
       'bundle_h_days':','.join(str(((b['T'] if i==len(actual)-1 else actual[i])-S).days) for i in indices),
       'bundle_r_days':','.join(str((b['T']-(b['T'] if i==len(actual)-1 else actual[i])).days) for i in indices),
       'entitlements':','.join('BuyerEntitled' if d>S else 'Ceased' for d in actual)}


def selected_coupon(b,i):
    if not 0<=i<len(b['actual']):fail('invalid_coupon_index',index=i,count=len(b['actual']))
    return b['sources'][i]


def cash_snapshot(inp,b,p,recipe):
    if recipe=='coupon':
        i=int(inp['coupon_index']);g=selected_coupon(b,i)
    else:i=p['next_i'] if recipe!='dirty' else None;g=p['accrued'] if recipe!='dirty' else None
    nominal=dec(inp['nominal'])
    if nominal[0]<=0:fail('nonpositive_nominal',nominal=dectext(nominal))
    out={'recipe_identifier':{'coupon':'PayableCouponPer100','accrued':'SettlementAccruedPer100',
         'dirty':'SettlementDirtyPricePer100','invoice':'CleanPriceAndSettlementAccruedPer100'}[recipe]}
    if recipe=='dirty':
        out|={'source_value':'none','src_interval':'none','src_quasi_pairs':'none','src_full_count':'none',
              'src_day_ratios':'none','src_index':'none','src_date':'none','src_payment':'none'}
        raw=None
    else:
        raw=g['P'] if recipe=='coupon' else p['AI']
        out|=geometry_output(g)
        out|={'source_value':rat(raw),'src_index':str(i),'src_date':b['actual'][i].isoformat(),
              'src_payment':(b['T'] if i==len(b['actual'])-1 else b['actual'][i]).isoformat()}
    return nominal,raw,out


def cash_evaluate(inp,nominal,raw,recipe):
    q=dec(inp['quantum']);assert q[0]>0;mode=inp['mode'];N=value(nominal)
    if recipe in ('coupon','accrued'):
        stage='cash coupon' if recipe=='coupon' else 'cash accrued'
        c=quantize(N*raw/100,q,mode,stage)
        final=admit(c,q[1],stage,'point')
        return {'expected':'determined','lower_amount':final,'upper_amount':final,'guarantee':'none'}
    lo,hi,guarantee=evidence(inp['evidence']);point=lo==hi
    components=recipe=='invoice' and inp['aggregation']=='RoundComponentsThenSum'
    stage='cash capital' if components else 'cash total'
    if mode=='Exact' and not point:
        return {'expected':'exact_unresolved','context':encode_context({'stage':stage})}
    shift=raw if recipe=='invoice' and not components else F(0)
    lower=quantize(N*(lo+shift)/100,q,mode,stage)
    upper=lower if point else quantize(N*(hi+shift)/100,q,mode,stage)
    if components:
        accrued=quantize(N*raw/100,q,mode,'cash accrued')
        lower+=accrued;upper+=accrued;point=lower==upper;stage='cash net';operation='sum'
    else:operation='quantize'
    point=lower==upper
    assert lower<=upper
    a=admit(lower,q[1],stage,'point' if point else 'lower',operation)
    z=a if point else admit(upper,q[1],stage,'upper',operation)
    if a==z:return {'expected':'determined','lower_amount':a,'upper_amount':z,'guarantee':guarantee}
    return {'expected':'ambiguous','lower_amount':a,'upper_amount':z}


def evaluate(inp,bindings):
    op=inp['op']
    if op=='source_reference':
        entry=bindings[inp['reference']]
        out={'expected':'source_observation','source_observation':entry['printed_observation'],
             'reference_classification':entry['classification']}
        calculation=entry['fixed_calculation']
        if calculation!='none':
            computed={'5*48/365':F(5*48,365),'5*(1/2-1/365)':5*(F(1,2)-F(1,365)),
                      '8*38/365*100':F(8*38*100,365)}[calculation]
            assert computed==F(entry['correct_exact_value'])
            out['source_value']=rat(computed)
        return out
    try:
        b=schedule(inp) if op=='schedule' else instrument(inp)
        out=common_output(b)
        if op=='schedule':out.pop('source_version');return out|{'expected':'ok'}
        if op=='instrument':return out|{'expected':'ok'}
        needs_plan=op in ('plan','source_view_accrued','cash_accrued','cash_dirty','cash_invoice',
                         'instruction_view_accrued','instruction_view_dirty','instruction_view_invoice')
        p=prepared(inp,b) if needs_plan else None
        if p:out|=plan_output(b,p)
        if op=='plan':return out|{'expected':'ok'}
        if op.startswith('source_view_'):
            item=inp['source_item'];q=dec(inp['quantum']);mode=inp['mode']
            if op=='source_view_coupon':
                g=selected_coupon(b,int(inp['coupon_index']))
                assert item in ('valuation_coupon','payable_coupon')
                raw=g['V' if item=='valuation_coupon' else 'P']
                stage='source valuation coupon' if item=='valuation_coupon' else 'source payable coupon'
            else:
                assert item in ('valuation_accrued','settlement_accrued')
                raw=p['A' if item=='valuation_accrued' else 'AI']
                stage='source valuation accrued' if item=='valuation_accrued' else 'source settlement accrued'
            c=quantize(raw,q,mode,stage)
            return out|{'expected':'amount','source_value':rat(raw),'view_amount':admit(c,q[1],stage,'point')}
        recipe=op.split('_')[-1]
        assert recipe in ('coupon','accrued','dirty','invoice')
        nominal,raw,snap=cash_snapshot(inp,b,p,recipe);out|=snap
        if op.startswith('instruction_view_'):
            if raw is None:return out|{'expected':'none','view_amount':'none'}
            q=dec(inp['view_quantum']);mode=inp['view_mode']
            stage='source payable coupon' if recipe=='coupon' else 'source settlement accrued'
            c=quantize(raw,q,mode,stage)
            return out|{'expected':'some','view_amount':admit(c,q[1],stage,'point')}
        return out|cash_evaluate(inp,nominal,raw,recipe)
    except ModelError as error:
        # No partial-object output masquerades as a completed public observation.
        return {'expected':error.kind,'context':encode_context(error.context)}


def amount_input(text):
    pair=dec(text)
    assert text==dectext(pair),'noncanonical input amount '+text
    return text


def canonical_evidence(text):
    pieces=text.split(':');kind=pieces[0]
    assert kind in ('d','di','b','bi'),'unknown evidence kind'
    assert len(pieces)=={'d':3,'di':5,'b':2,'bi':3}[kind],'unused or missing evidence token'
    if kind.startswith('d'):
        for i in (1,3) if kind=='di' else (1,):amount_input(':'.join(pieces[i:i+2]))
    else:
        for b in pieces[1:]:
            assert re.fullmatch(r'[0-9a-f]{16}',b),'noncanonical IEEE64 spelling'
            x=binary_fraction(b)
            assert x or b=='0000000000000000'
    evidence(text)
    return text


def canonical_date(text):
    assert re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}',text),'noncanonical Date spelling'
    assert date.fromisoformat(text).isoformat()==text
    return text


def validate_input_record(inp,schema,bindings):
    op=inp['op'];assert op in schema['operation_input_fields'],'unknown operation'
    allowed=schema['operation_input_fields'][op]
    for key in schema['input_columns']:
        assert (inp[key]!='-') if key in allowed else (inp[key]=='-'),'unused or missing input '+key
    for key in ('dated','issue','stated_maturity','true_maturity','settlement'):
        if inp[key]!='-':canonical_date(inp[key])
    for key in ('quasi','coupon_dates'):
        if inp[key]!='-' and inp[key]!='empty':
            for x in inp[key].split(','):canonical_date(x)
    for key in ('date_cap','bundle_cap','coupon_index'):
        if inp[key]!='-':assert re.fullmatch(r'0|[1-9][0-9]*',inp[key]) and int(inp[key])<(1<<64),'noncanonical usize input '+key
    for key in ('coupon','nominal','quantum','view_quantum'):
        if inp[key]!='-':
            amount_input(inp[key])
            if key in ('quantum','view_quantum'):assert dec(inp[key])[0]>0,'nonpositive typed quantum'
    if inp['frequency']!='-':assert inp['frequency'] in ('2','4','12')
    if inp['eom']!='-':assert inp['eom'] in ('Ignore','Preserve')
    if inp['source_policy']!='-':assert inp['source_policy']==POLICY
    if inp['selection']!='-':assert inp['selection'] in SELECTIONS
    if inp['aggregation']!='-':assert inp['aggregation'] in ('RoundTotal','RoundComponentsThenSum')
    for key in ('mode','view_mode'):
        if inp[key]!='-':assert inp[key] in MODES
    if inp['evidence']!='-':canonical_evidence(inp['evidence'])
    if inp['reference']!='-':assert inp['reference'] in bindings
    if inp['source_item']!='-':
        assert inp['source_item'] in (('valuation_coupon','payable_coupon') if op=='source_view_coupon' else ('valuation_accrued','settlement_accrued'))


def validate_artifact(raw,schema,registry,data,reqrows):
    generation=fixture_generation(raw)
    head,body=raw.split(b'\n\n',1);headers=dict(line.decode().split(': ',1) for line in head.splitlines())
    basekeys={'schema','fixture-set','domain','description','reference','reference-version','reference-source','tolerance','reproduction',
       'generated-at','platform','python','source-version','schema-sha256','source-bindings-sha256','adopted-contract-sha256','inputs',
       'requirements-rows-sha256','requirements-rows-fnv64','operation-input-fields','definition-digest-scope','canonical-bytes','status'}
    statskeys={'record-count','unique-serialized-inputs','unique-financial-definitions','unique-schedule-definitions','unique-supplied-definition-digests',
       'scope-counts','column-count','profile-counts','operation-counts','expectation-counts','overlapping-enumerators-skipped',
       'records-sha256','records-fnv64','canonical-inputs-sha256','canonical-inputs-fnv64'}
    assert set(headers)==basekeys|statskeys,'unknown/missing header'
    pins={key:{'sha256':sha(value),'fnv64':fnv(value),'bytes':len(value)} for key,value in sorted(data.items())}
    assert headers['inputs']==canonical_json(pins),'actual consumed input pin mismatch'
    assert headers['adopted-contract-sha256']==CONTRACT_SHA and headers['requirements-rows-sha256']==sha(reqrows)
    assert headers['requirements-rows-fnv64']==fnv(reqrows)
    assert headers['operation-input-fields']==canonical_json(schema['operation_input_fields'])
    lines=body.decode('ascii').splitlines();assert lines[0].split('\t')==schema['columns'],'unknown columns'
    expected_width=len(schema['columns']);seen=set();profiles=Counter();ops=Counter();expectations=Counter();scopes=Counter()
    bindings={e['reference']:e for e in registry['bindings']};serial=[];defs=set();schedules=set();fulldefs=set()
    for i,line in enumerate(lines[1:],1):
        fields=line.split('\t');assert len(fields)==expected_width,'record width'
        row=dict(zip(schema['columns'],fields));inp={k:row[k] for k in schema['input_columns']}
        validate_input_record(inp,schema,bindings)
        semantic=canonical_json({k:inp[k] for k in schema['operation_input_fields'][inp['op']]}).encode()
        assert semantic not in seen,'duplicate actual semantic input (answers/annotations excluded)'
        seen.add(semantic);serial.append(semantic+b'\n')
        assert row['id']==f'CA{i:06d}','noncanonical row identity'
        assert row['scope']==('source_reference' if row['op']=='source_reference' else 'public')
        if row['expected'].startswith('canadian.'):
            assert all(row[k]=='-' for k in schema['output_columns'] if k not in ('expected','context')),'uncomputed output after failure'
        if row['op']=='source_reference':assert row['source_observation']==bindings[row['reference']]['printed_observation'],'primary cell changed'
        definition={k:inp[k] for k in ('dated','issue','stated_maturity','true_maturity','frequency','eom','quasi','coupon_dates','coupon','source_policy') if inp[k]!='-'}
        expected_def=sha(canonical_json(definition).encode()) if definition else '-'
        assert row['definition_sha256']==expected_def,'definition annotation changed'
        if expected_def!='-':defs.add(expected_def)
        if row['op']=='schedule':schedules.add(expected_def)
        if inp['coupon']!='-':fulldefs.add(expected_def)
        profiles[row['profile']]+=1;ops[row['op']]+=1;expectations[row['expected']]+=1;scopes[row['scope']]+=1
    for key,actual in [('record-count',str(len(lines)-1)),('unique-serialized-inputs',str(len(seen))),('column-count',str(expected_width)),
       ('unique-financial-definitions',str(len(fulldefs))),('unique-schedule-definitions',str(len(schedules))),
       ('unique-supplied-definition-digests',str(len(defs))),('profile-counts',canonical_json(profiles)),('operation-counts',canonical_json(ops)),
       ('expectation-counts',canonical_json(expectations)),('scope-counts',canonical_json(scopes))]:assert headers[key]==actual,'census drift '+key
    assert headers['records-sha256']==sha(body) and headers['records-fnv64']==fnv(body),'body hash mismatch'
    serial=b''.join(serial)
    assert headers['canonical-inputs-sha256']==sha(serial) and headers['canonical-inputs-fnv64']==fnv(serial),'actual canonical input hash mismatch'
    return generation


class Cases:
    def __init__(self,schema,bindings):
        self.schema=schema;self.bindings=bindings;self.rows=[];self.seen={};self.duplicates=Counter()
    def add(self,profile,base,op,fr,identity_group='-',**params):
        allowed=self.schema['operation_input_fields'][op]
        supplied=dict(base);supplied.update(params);supplied['op']=op
        inp={key:str(supplied[key]) if key in supplied else '-' for key in self.schema['input_columns']}
        for key in self.schema['input_columns']:
            if key not in allowed:inp[key]='-'
        for key in allowed:assert inp[key]!='-',(op,'missing input',key)
        validate_input_record(inp,self.schema,self.bindings)
        for key in ('coupon','nominal','quantum','view_quantum'):
            if inp[key]!='-':amount_input(inp[key])
        if inp['evidence']!='-':canonical_evidence(inp['evidence'])
        semantic=canonical_json({key:inp[key] for key in allowed}).encode()
        if semantic in self.seen:
            # Intentional overlapping profile enumerators do not create duplicate records.
            self.duplicates[profile]+=1
            return False
        expected=evaluate(inp,self.bindings)
        row={c:'-' for c in self.schema['columns']}
        row.update(inp);row.update(expected)
        row.update({'id':f'CA{len(self.rows)+1:06d}','profile':profile,
                    'scope':'source_reference' if op=='source_reference' else 'public',
                    'fr_families':','.join('FR-CAD-'+str(x).zfill(2) for x in sorted(set(fr))),
                    'identity_group':identity_group})
        definition={k:inp[k] for k in ('dated','issue','stated_maturity','true_maturity','frequency','eom','quasi','coupon_dates','coupon','source_policy') if inp[k]!='-'}
        if definition:row['definition_sha256']=sha(canonical_json(definition).encode())
        assert set(row)==set(self.schema['columns'])
        assert all(not any(ch in str(v) for ch in '\r\n\t') for v in row.values())
        self.seen[semantic]=len(self.rows);self.rows.append(row)
        return True


def base_for(f,shape='regular',eom='Ignore',anchor=date(2027,1,31),coupon='5:0'):
    """Full supplied definitions; no hidden inferred schedule or pseudo cash basis."""
    n=6 if shape in ('long_first','long_last','long_both','long_first_short_last','short_first_long_last') else 4
    if shape in ('composite_same','single_regular'):n=2
    Q=[shift_months(anchor,-(12//f)*(n-1-i),eom) for i in range(n)]
    D=Q[0];M=Q[-1];actual=Q[1:]
    if shape=='short_first':D+=timedelta(days=7)
    elif shape=='short_both':D+=timedelta(days=7);M-=timedelta(days=7);actual=Q[1:-1]+[M]
    elif shape=='long_first':D+=timedelta(days=7);actual=Q[3:]
    elif shape=='short_last':M-=timedelta(days=7);actual=Q[1:-1]+[M]
    elif shape=='long_last':M-=timedelta(days=7);actual=Q[1:3]+[M]
    elif shape=='long_both':D+=timedelta(days=7);M-=timedelta(days=7);actual=[Q[3],M]
    elif shape=='long_first_short_last':D+=timedelta(days=7);M-=timedelta(days=7);actual=Q[3:-1]+[M]
    elif shape=='short_first_long_last':D+=timedelta(days=7);M-=timedelta(days=7);actual=Q[1:3]+[M]
    elif shape=='composite_same':D+=timedelta(days=3);M=Q[0]+timedelta(days=20);actual=[M]
    elif shape=='composite_across':D+=timedelta(days=7);M-=timedelta(days=7);actual=[M]
    elif shape=='single_long':actual=[M]
    elif shape not in ('regular','single_regular'):raise AssertionError(shape)
    return dict(dated=D.isoformat(),issue=D.isoformat(),stated_maturity=M.isoformat(),true_maturity=M.isoformat(),
        frequency=str(f),eom=eom,quasi=datevec(Q),coupon_dates=datevec(actual),coupon=coupon,date_cap='20000',
        source_policy=POLICY,settlement=(D+timedelta(days=1)).isoformat(),selection=SELECTIONS[0],bundle_cap='20000',
        nominal='100:0',quantum='1:2',mode='NearestTiesEven',evidence='d:100:0',aggregation='RoundTotal',
        coupon_index='0',source_item='payable_coupon',view_quantum='1:6',view_mode='NearestTiesEven')


def catalog_source(c):
    for reference,entry in c.bindings.items():
        family=2 if 'short-last-48day' in reference else 3 if 'short-last-183day' in reference else 4 if ';section=appendix2;' in reference else 5 if 'late-Canadian' in reference else 6 if 'monthly' in reference else 7 if 'general-MM' in reference else 8 if 'DIC1' in reference else 23
        c.add('primary_cell',{},'source_reference',[1,family],reference=reference)
    shapes=('regular','short_first','short_both','long_first','short_last','long_last','long_both',
            'long_first_short_last','short_first_long_last','composite_same','composite_across','single_regular','single_long')
    for f in (2,4,12):
        for shape in shapes:
            for eom in ('Ignore','Preserve'):
                for C in ('0:0','5:0','1:38',str(MAX)+':0'):
                    b=base_for(f,shape,eom,coupon=C)
                    c.add('complete_shapes',b,'schedule',[9,11,12,13,14,15,16,17,18,19,20,21])
                    c.add('complete_shapes',b,'instrument',[9,13,14,15,16,17,18,19,20,21,22,23,25,26,27,28,30])
                    D=date.fromisoformat(b['dated']);actual=dates(b['coupon_dates']);M=actual[-1]
                    # First, interior skipped-Q and exact actual-date resets are all retained.
                    settlements={D,D+timedelta(days=1),M-timedelta(days=1)}
                    settlements.update(x for x in dates(b['quasi']) if D<=x<M)
                    settlements.update(x for x in actual if x<M)
                    for S in sorted(settlements):
                        for selection in SELECTIONS:
                            c.add('complete_plans',b,'plan',[9,23,24,25,26,28,29,30,32,60],settlement=S.isoformat(),selection=selection)
                    # Every actual coupon ordinal; valuation versus payable never substituted.
                    for i in range(len(actual)):
                        for item in ('valuation_coupon','payable_coupon'):
                            for mode in MODES:
                                c.add('source_coupon_views',b,'source_view_coupon',[9,25,26,27,28,30,31],
                                      coupon_index=str(i),source_item=item,quantum='1:6',mode=mode)
                    for item in ('valuation_accrued','settlement_accrued'):
                        for mode in MODES:
                            c.add('source_accrued_views',b,'source_view_accrued',[9,24,25,26,28,29,30,31,32],
                                  source_item=item,quantum='1:6',mode=mode)
    # Composite exact quasi length one is still Composite, independently of length test.
    for f,anchor,D,M in ((12,date(2024,9,1),date(2024,7,16),date(2024,8,16)),
                         (4,date(2025,1,1),date(2024,7,16),date(2024,10,16)),
                         (2,date(2024,9,30),date(2023,10,15),date(2024,4,15))):
        Q=[shift_months(anchor,-(12//f)*i,'Preserve') for i in (2,1,0)]
        b=base_for(f);b.update(dated=D.isoformat(),issue=D.isoformat(),stated_maturity=M.isoformat(),true_maturity=M.isoformat(),
            quasi=datevec(Q),coupon_dates=M.isoformat(),eom='Preserve',settlement=D.isoformat())
        assert chi(Q,M)-chi(Q,D)==1
        c.add('composite_length_one',b,'instrument',[20,25])
    # Direct anchor clamping and end-of-month all civil month lengths and centuries.
    for f in (2,4,12):
        for year in (1900,2000,2023,2024,2100):
            for month in (2,4,7,12):
                for day in sorted({28,calendar.monthrange(year,month)[1]}):
                    for eom in ('Ignore','Preserve'):
                        b=base_for(f,'regular',eom,date(year,month,day))
                        c.add('anchor_civil',b,'schedule',[13])
    # Explicit full scale grid for source coupon admission; not just scale endpoints.
    for f in (2,4,12):
        for scale in range(39):
            for coef in (1,MAX):
                b=base_for(f,'long_both',coupon=amount(coef,scale))
                c.add('coupon_scales',b,'instrument',[9,25,26,28,30])
                for item in ('valuation_coupon','payable_coupon'):
                    c.add('coupon_scales',b,'source_view_coupon',[25,26,28,30,31],source_item=item,mode='Exact',quantum='1:38')
    # Literal Canadian365 discontinuity and endpoint full-piece override.
    for f,anchor in ((2,date(2025,1,31)),(4,date(2024,10,31)),(12,date(2024,8,31))):
        b=base_for(f,'single_regular','Preserve',anchor)
        Q=dates(b['quasi']);d0=Q[0]
        threshold=365//f
        for elapsed in sorted({0,1,(Q[1]-d0).days-1,threshold,threshold+1}):
            S=d0+timedelta(days=elapsed)
            if S>=Q[1]:continue
            for item in ('valuation_accrued','settlement_accrued'):
                c.add('literal_branch',b,'source_view_accrued',[25,26,27,28,29],settlement=S.isoformat(),source_item=item,quantum='1:9')
    # Same first cash with different L still has distinct complete financial identity.
    b=base_for(2);Q=[date(2024+i//2,1 if i%2==0 else 7,1) for i in range(7)]
    b.update(dated=Q[0].isoformat(),issue=Q[0].isoformat(),stated_maturity=Q[-1].isoformat(),true_maturity=Q[-1].isoformat(),
             quasi=datevec(Q),settlement=Q[0].isoformat())
    for ac in ([Q[1],Q[2],Q[-1]],[Q[1],Q[2],Q[3],Q[-1]]):
        bb=b|{'coupon_dates':datevec(ac)}
        c.add('identity_last_start',bb,'cash_coupon',[61,69,70],identity_group='same-first-cash-different-L')
        c.add('identity_last_start',bb,'cash_dirty',[61,69,70],identity_group='same-dirty-cash-different-L')
    # True maturity shifts payments/MM horizons while preserving contractual source geometry.
    for f in (2,4,12):
        b=base_for(f,'long_both');T=date.fromisoformat(b['stated_maturity'])+timedelta(days=11)
        b['true_maturity']=T.isoformat()
        for selection in SELECTIONS:
            c.add('true_maturity',b,'plan',[23,24,29,60],selection=selection)
            c.add('true_maturity',b,'cash_coupon',[23,61,69],coupon_index=str(len(dates(b['coupon_dates']))-1),selection=selection)
    # Civil extrema admitted as typed dates, including widest available MM horizon.
    for f,Q in ((2,[date(1,1,1),date(1,7,1)]),(12,[date(9999,11,30),date(9999,12,30)])):
        b=base_for(f);b.update(dated=Q[0].isoformat(),issue=Q[0].isoformat(),quasi=datevec(Q),
            coupon_dates=Q[1].isoformat(),stated_maturity=Q[1].isoformat(),true_maturity=date.max.isoformat(),
            settlement=Q[0].isoformat(),coupon='0:0')
        c.add('civil_extrema',b,'plan',[12,13,23,24,29,32],selection='Explicit:MoneyMarket')
    # Actual max count, explicit full vectors; no checksum substitutes for financial input identity.
    anchor=date(9999,12,31);Q=[shift_months(anchor,-i,'Preserve') for i in range(19999,-1,-1)]
    b=base_for(12);b.update(dated=Q[0].isoformat(),issue=Q[0].isoformat(),quasi=datevec(Q),coupon_dates=datevec(Q[1:]),
        stated_maturity=Q[-1].isoformat(),true_maturity=Q[-1].isoformat(),settlement=Q[-2].isoformat(),coupon='0:0',eom='Preserve')
    c.add('maximum_date_cap',b,'schedule',[11,12,13])
    c.add('maximum_date_cap',b,'plan',[11,13,24,29,30,32],bundle_cap='1')


def catalog_errors(c):
    b=base_for(2);Q=dates(b['quasi']);ac=dates(b['coupon_dates']);D=date.fromisoformat(b['dated']);M=ac[-1]
    edits=[('cap_low',{'date_cap':'1'}),('cap_zero',{'date_cap':'0'}),('cap_high',{'date_cap':'20001'}),
      ('Q_count',{'date_cap':'2'}),('Q_empty',{'quasi':'empty'}),('Q_single',{'quasi':Q[0].isoformat()}),
      ('coupon_count',{'quasi':datevec([Q[0],Q[-1]]),'date_cap':'2','coupon_dates':datevec(ac)}),
      ('coupon_empty',{'coupon_dates':'empty'}),('D_equal_M',{'dated':M.isoformat()}),('D_after_M',{'dated':(M+timedelta(days=1)).isoformat()}),
      ('Q_equal',{'quasi':datevec([Q[0],Q[1],Q[1],Q[-1]])}),('Q_reverse',{'quasi':datevec([Q[1],Q[0],Q[2],Q[-1]])}),
      ('coverage_prefix',{'dated':Q[1].isoformat()}),('coverage_suffix',{'stated_maturity':Q[-2].isoformat()}),
      ('anchor_mismatch',{'quasi':datevec([Q[0]+timedelta(days=1)]+Q[1:])}),
      ('last_boundary',{'coupon_dates':datevec(ac[:-1])}),
      ('coupon_equal',{'coupon_dates':datevec([ac[0],ac[0],M])}),('coupon_reverse',{'coupon_dates':datevec([ac[1],ac[0],M])}),
      ('coupon_at_D',{'coupon_dates':datevec([D]+ac)}),('coupon_before_D',{'coupon_dates':datevec([D-timedelta(days=1)]+ac)}),
      ('offcycle',{'coupon_dates':datevec([ac[0]+timedelta(days=1)]+ac[1:])})]
    for label,edit in edits:c.add('schedule_error_'+label,b,'schedule',[11,12,13,14,71],**edit)
    bb=base_for(2,'long_last');qq=dates(bb['quasi']);bb['coupon_dates']=datevec([qq[1],qq[3],date.fromisoformat(bb['stated_maturity'])])
    c.add('schedule_error_interior',bb,'schedule',[14,71])
    # Count20001 rejected before any Q ordering/calendar arithmetic.
    c.add('schedule_error_count20001',b,'schedule',[11,71],quasi=','.join([Q[0].isoformat()]*20001))
    # Valid typed daily dates/minimal coverage, but impossible20000-month direct cycle.
    qq=[date.min+timedelta(days=i) for i in range(20000)]
    bb=base_for(12);bb.update(dated=qq[0].isoformat(),issue=qq[0].isoformat(),quasi=datevec(qq),coupon_dates=qq[-1].isoformat(),
        stated_maturity=qq[-1].isoformat(),true_maturity=qq[-1].isoformat())
    c.add('schedule_error_date_underflow',bb,'schedule',[11,12,13,14,71])
    for label,edit in (('negative_C',{'coupon':'-1:38'}),('D_after_I',{'issue':(D-timedelta(days=1)).isoformat()}),
                       ('I_at_F',{'issue':ac[0].isoformat()}),('T_before_M',{'true_maturity':(M-timedelta(days=1)).isoformat()})):
        c.add('instrument_error_'+label,b,'instrument',[11,12,13,14,71],**edit)
    # Intentionally combined failures pin validation precedence.
    c.add('constructor_precedence',b,'instrument',[11,22,71],coupon='-1:0',issue=ac[0].isoformat(),date_cap='1')
    for label,edit in (('cap',{'bundle_cap':'20001'}),('zero_cap',{'bundle_cap':'0'}),('count',{'bundle_cap':'2'}),
        ('S_before_I',{'settlement':(D-timedelta(days=1)).isoformat()}),('S_at_M',{'settlement':M.isoformat()}),
        ('S_after_M',{'settlement':(M+timedelta(days=1)).isoformat()})):
        c.add('plan_error_'+label,b,'plan',[11,24,71],**edit)
    for op in ('cash_coupon','source_view_coupon','instruction_view_coupon'):
        for index in (str(len(ac)),str((1<<64)-1)):
            c.add('coupon_ordinal_error',b,op,[14,61,69,71],coupon_index=index,nominal='0:0')
    for op in ('cash_coupon','cash_accrued','cash_dirty','cash_invoice'):
        for N in ('0:0','-1:38',str(-(1<<127))+':0'):
            c.add('nominal_error',b,op,[61,71],nominal=N)
    # An issue after a skipped virtual boundary is allowed when before actual F.
    bb=base_for(2,'long_first');q=dates(bb['quasi']);bb.update(issue=(q[1]+timedelta(days=1)).isoformat(),settlement=(q[1]+timedelta(days=2)).isoformat())
    c.add('issue_skips_virtual',bb,'plan',[17,22,24,29])


def catalog_cash(c):
    # Complete geometry cash crossproduct; enum policy/selection resolved by real plan.
    for f in (2,4,12):
        for shape in ('regular','short_first','long_first','short_last','long_last','composite_same','composite_across'):
            b=base_for(f,shape)
            for mode in MODES:
                for op in ('cash_coupon','cash_accrued','cash_dirty','cash_invoice'):
                    for agg in ('RoundTotal','RoundComponentsThenSum') if op=='cash_invoice' else ('RoundTotal',):
                        c.add('cash_geometry_modes',b,op,[9,25,26,28,61,62,65,66,69],mode=mode,aggregation=agg)
    # Independent source views keep invoice rounding separate, including dirty Ok(None).
    for recipe in ('coupon','accrued','dirty','invoice'):
        for mode in MODES:
            for view_mode in MODES:
                b=base_for(2,'long_both')
                c.add('instruction_source_views',b,'instruction_view_'+recipe,[31,61,69],mode=mode,view_mode=view_mode,
                    view_quantum='7:5',aggregation='RoundComponentsThenSum')
    # Signed exact-decimal, supplied-binary and continuous enclosures incl. equal declarations.
    evs=('d:0:0','d:125:2','d:-125:2','d:'+str(MAX)+':0','d:'+str(-(1<<127))+':0',
         'b:3ff4000000000000','b:bff4000000000000','b:0000000000000001','b:7fefffffffffffff',
         'b:ffefffffffffffff','di:0:0:1:0','di:-1:0:1:0','di:125:2:125:2',
         'bi:3ff0000000000000:4000000000000000','bi:bff0000000000000:3ff0000000000000',
         'bi:0000000000000000:0000000000000000')
    for f in (2,4,12):
        b=base_for(f,'long_first')
        for ev in evs:
            for mode in MODES:
                for op in ('cash_dirty','cash_invoice'):
                    for agg in ('RoundTotal','RoundComponentsThenSum') if op=='cash_invoice' else ('RoundTotal',):
                        c.add('cash_evidence_kinds',b,op,[61,62,64,65,66,67,68,70],evidence=ev,mode=mode,aggregation=agg)
    # All39x39 nominal/quantum scales; coupon scale rotates through all39, not39^3.
    for sn in range(39):
        for sq in range(39):
            sc=(sn+7*sq)%39;f=(2,4,12)[(sn+sq)%3]
            b=base_for(f,'long_both',coupon=amount(7,sc));b.update(nominal=amount(11,sn),quantum=amount(3,sq),evidence='d:-17:3')
            for mode in MODES:
                for op in ('cash_coupon','cash_accrued','cash_dirty','cash_invoice'):
                    for agg in ('RoundTotal','RoundComponentsThenSum') if op=='cash_invoice' else ('RoundTotal',):
                        c.add('cash_scale_grid',b,op,[30,61,62,64,65,66,67],mode=mode,aggregation=agg)
    # Half ties of both signs at arbitrary positive quantum, including i128 boundaries.
    b=base_for(2,coupon='0:0')
    for q in ('1:0','7:3','1:38',str(MAX)+':0'):
        cq,sq=dec(q)
        for k in (-3,-2,-1,0,1,2):
            x=F(2*k+1,2)*F(cq,10**sq)
            # Use binary for exact representable small half ties, otherwise admitted decimal half where possible.
            decimal_scale=sq+1
            coeff=(2*k+1)*cq*5
            if decimal_scale<=38 and -(1<<127)<=coeff<=MAX:ev='d:'+amount(coeff,decimal_scale)
            elif x.denominator&(x.denominator-1)==0 and abs(x)<pow2(1024):ev='b:'+ieee_bits(x)
            else:continue
            for mode in MODES:c.add('signed_quantum_ties',b,'cash_dirty',[61,62,67],evidence=ev,quantum=q,mode=mode)
    # Exact capital failure precedes accrued; nonpoint Exact refuses despite integral endpoints.
    for ev in ('d:1:0','di:0:0:100:0','di:1:0:1:0'):
        for agg in ('RoundTotal','RoundComponentsThenSum'):
            c.add('exact_stage_order',base_for(2,'short_first'), 'cash_invoice',[61,64,65],nominal='1:0',evidence=ev,quantum='1:0',mode='Exact',aggregation=agg)
    # Wide components can each exceed i128 and cancel. C regular coupon=AI near MAX/2.
    b=base_for(2,'single_regular',coupon=str(MAX)+':0');D=date.fromisoformat(b['dated']);M=date.fromisoformat(b['stated_maturity'])
    b.update(settlement=(M-timedelta(days=1)).isoformat(),nominal=str(MAX)+':0',evidence='d:'+str(-MAX)+':0',quantum='1:0')
    for agg in ('RoundTotal','RoundComponentsThenSum'):
        for mode in MODES:c.add('wide_cancellation',b,'cash_invoice',[61,62,65,67],aggregation=agg,mode=mode)
    # Exact AI cancellation as a decimal source at zero settlement accrual and signed finite range.
    b=base_for(2,coupon='0:0');b.update(nominal='100:0',quantum='1:0')
    for ev in ('d:'+str(MAX)+':0','d:'+str(-(1<<127))+':0','di:'+str(MAX-1)+':0:'+str(MAX)+':0',
               'bi:7feffffffffffffe:7fefffffffffffff'):
        for mode in MODES:c.add('final_admission_bounds',b,'cash_dirty',[61,62,67],evidence=ev,mode=mode)


def catalog_additional(c):
    # Daily endpoint sweeps retain actual public intervals, not arbitrary d/s primitives.
    for f in (2,4,12):
        for anchor in (date(2023,7,31),date(2024,3,31),date(2024,7,31)):
            b=base_for(f,'single_regular','Preserve',anchor)
            Q=dates(b['quasi']);span=(Q[-1]-Q[0]).days
            for day in range(1,span):
                for end in ('first','last'):
                    bb=dict(b)
                    if end=='first':
                        D=Q[0]+timedelta(days=day);bb.update(dated=D.isoformat(),issue=D.isoformat(),settlement=D.isoformat())
                    else:
                        M=Q[0]+timedelta(days=day);bb.update(stated_maturity=M.isoformat(),true_maturity=M.isoformat(),coupon_dates=M.isoformat(),settlement=Q[0].isoformat())
                    c.add('daily_short_'+end,bb,'instrument',[16 if end=='first' else 18,25,26,27])
    # Thousands of skipped virtual boundaries in a long first interval are not payments.
    anchor=date(9999,12,31);Q=[shift_months(anchor,-i,'Preserve') for i in range(19999,-1,-1)]
    b=base_for(12);D=Q[0]+timedelta(days=1);M=Q[-1]
    b.update(dated=D.isoformat(),issue=Q[10000].isoformat(),settlement=Q[10001].isoformat(),quasi=datevec(Q),
        coupon_dates=datevec([Q[-2],M]),stated_maturity=M.isoformat(),true_maturity=M.isoformat(),eom='Preserve',coupon=str(MAX)+':0',nominal='1:38')
    c.add('thousands_long_first',b,'instrument',[17,22,25,28,30])
    c.add('thousands_long_first',b,'source_view_coupon',[17,30,31],quantum='1:0')
    c.add('source_view_overflow_cash_rescue',b,'cash_coupon',[17,30,31,61,69],quantum='1:2')
    # Dated exact wide cancellation with both components beyond the final coefficient range.
    b=base_for(2,'single_regular','Preserve',date(2027,1,31),str(146*10**36)+':0')
    M=date.fromisoformat(b['stated_maturity']);b.update(settlement=(M-timedelta(days=1)).isoformat(),nominal=str(MAX)+':0',quantum='1:0')
    p=prepared(b,instrument(b));assert p['AI'].denominator==1
    b['evidence']='d:'+str(100-p['AI'].numerator)+':0'
    assert value(dec(b['nominal']))*p['AI']/100>MAX
    for mode in MODES:
        for agg in ('RoundTotal','RoundComponentsThenSum'):
            c.add('wide_net_rescue',b,'cash_invoice',[61,62,65,67],mode=mode,aggregation=agg)
    # A real source AI=MAX: two complete semiannual periods at an omitted actual boundary.
    b=base_for(2,'single_long',coupon=str(MAX)+':0');qq=dates(b['quasi'])
    b.update(settlement=qq[2].isoformat(),nominal='200:0',evidence='d:'+str(-MAX)+':0')
    assert prepared(b,instrument(b))['AI']==MAX
    for q in ('1:0','3:0'):
        for agg in ('RoundTotal','RoundComponentsThenSum'):
            c.add('exact_cancellation_does_not_waive_stage',b,'cash_invoice',[29,61,62,65],quantum=q,mode='Exact',aggregation=agg)
    # Canonicalization before admission, Exact source failure after exact zero capital.
    for b in (base_for(2,'long_first',coupon=str(MAX)+':0'),base_for(12,'short_first',coupon='1:0')):
        for op in ('cash_accrued','cash_invoice'):
            c.add('canonical_final_and_accrued_exact',b,op,[30,31,61,65,67],nominal='100:0',quantum='1:38',
                  evidence='d:0:0',mode='Exact',aggregation='RoundComponentsThenSum')
    b=base_for(2,coupon='0:0')
    for agg in ('RoundTotal','RoundComponentsThenSum'):
        c.add('current_stage_pointness',b,'cash_invoice',[64,65,66,67],evidence='di:'+str(MAX-1)+':0:'+str(MAX)+':0',
              nominal='200:0',quantum=str(MAX)+':0',aggregation=agg)
    c.add('dirty_final_image_pointness',b,'cash_dirty',[64,65,66,67],evidence='di:'+str(MAX-1)+':0:'+str(MAX)+':0',nominal='200:0',quantum=str(MAX)+':0')
    for ev in ('di:0:0:2:0','d:170141183460469231731687303715884105727:38'):
        c.add('range_hole_endpoints',b,'cash_dirty',[64,67],evidence=ev,quantum='1:38')
    for ev in ('bi:4130000000000000:4130000000000001','di:0:0:1:0'):
        c.add('continuous_exact_interior',b,'cash_dirty',[64,66],evidence=ev,mode='Exact',quantum='1:38')
    # Exact AI versus supplied binary AIhat: source precision is never silently substituted.
    b=base_for(2,'long_first');p=prepared(b,instrument(b));assert p['AI']!=binary_fraction(p['AI_bits'])
    for op,ev in (('cash_invoice','d:0:0'),('cash_dirty','b:'+p['AI_bits'])):
        c.add('exact_AI_vs_binary_ingredient',b,op,[61,68,69,70],evidence=ev,quantum='1:38',identity_group='sourceAI-vs-observedAIhat')
    # No source quantum spill: retained cash q differs independently from q/m query.
    for op in ('instruction_view_coupon','instruction_view_accrued','instruction_view_invoice','instruction_view_dirty'):
        c.add('view_failure_independent_cash',base_for(2,'long_first'),op,[31,61,69],quantum='1:0',mode='TowardZero',
              view_quantum='1:38',view_mode='Exact')


def catalog_source_named_and_boundaries(c):
    b=base_for(2);Q=[date(2006,12,1),date(2007,6,1),date(2007,12,1),date(2008,6,1),date(2008,12,1)]
    b.update(dated='2007-03-01',issue='2007-03-01',settlement='2007-06-01',quasi=datevec(Q),
             coupon_dates=datevec(Q[2:]),stated_maturity=Q[-1].isoformat(),true_maturity=Q[-1].isoformat())
    assert instrument(b)['sources'][0]['V']==F(5,2)*(1+F(92,182))
    assert F(5,2)*(1+F(275,182))-instrument(b)['sources'][0]['V']==F(915,364)
    c.add('source_DIC1_diagram',b,'instrument',[8,17,25])
    c.add('source_DIC1_diagram',b,'plan',[8,17,24,29])
    for Q,M,correct in (([date(2007,6,1),date(2007,12,1)],date(2007,7,19),F(48,73)),
                        ([date(2005,8,1),date(2006,2,1)],date(2006,1,31),F(363,146))):
        b=base_for(2);b.update(dated=Q[0].isoformat(),issue=Q[0].isoformat(),settlement=Q[0].isoformat(),quasi=datevec(Q),
            coupon_dates=M.isoformat(),stated_maturity=M.isoformat(),true_maturity=M.isoformat(),date_cap='2')
        assert instrument(b)['sources'][0]['P']==correct
        c.add('source_wrong_printed_short_last',b,'source_view_coupon',[2,3,18,25,26,31],quantum='1:8')
    for anchor in (date(2006,8,1),date(2008,8,1)):
        b=base_for(2,'single_regular',anchor=anchor);b['date_cap']='2'
        assert instrument(b)['sources'][0]['P']==F(5,2)
        c.add('completed_181_182',b,'instrument',[11,15,25,27])
    b=base_for(12,'single_regular',anchor=date(2024,8,1),coupon='12:0');b.update(settlement='2024-07-16',date_cap='2')
    p=prepared(b,instrument(b));assert p['AI']==F(15,31)
    c.add('monthly_general_exception',b,'source_view_accrued',[6,28],source_item='settlement_accrued',quantum='1:9')
    b.update(stated_maturity='2024-07-16',true_maturity='2024-07-16',coupon_dates='2024-07-16',settlement='2024-07-01')
    assert instrument(b)['sources'][0]['P']==F(36,73)
    c.add('monthly_payable_odd',b,'source_view_coupon',[6,28],quantum='1:9')
    for f in (2,4,12):
        for shape in ('single_regular','composite_same'):
            c.add('minimum_valid_date_cap',base_for(f,shape),'schedule',[11,12,13,15,20],date_cap='2')
    c.add('minimum_coupon_negative',base_for(2),'instrument',[22,30,71],coupon=str(-(1<<127))+':0')
    # Full-scale maximum coefficient controls complement the rotated small-coefficient scale grid.
    for scale in range(39):
        b=base_for((2,4,12)[scale%3],'long_both',coupon=amount(MAX,scale))
        for mode in MODES:
            for op in ('cash_coupon','cash_accrued','cash_dirty','cash_invoice'):
                for agg in ('RoundTotal','RoundComponentsThenSum') if op=='cash_invoice' else ('RoundTotal',):
                    c.add('maximum_coefficients_all_scales',b,op,[30,61,62,65,67],nominal=amount(MAX,scale),quantum=amount(MAX,38-scale),
                          evidence='d:-17:3',aggregation=agg,mode=mode)
    b=base_for(2,coupon='0:0')
    for q in ('1:0','7:3','1:38',str(MAX)+':0'):
        for sign in (-1,1):
            cq,sq=dec(q);ev='d:'+amount(sign*cq,sq)
            for N in ('50:0','150:0','250:0'):
                for mode in MODES:
                    c.add('arbitrary_quantum_half_ties',b,'cash_dirty',[61,62,67],nominal=N,evidence=ev,quantum=q,mode=mode)
    c.add('range_hole_actual_interior',b,'cash_dirty',[64,67],evidence='d:'+str(10**38-1)+':38',nominal='200:0',quantum='1:38')
    c.add('upper_only_final_failure',b,'cash_dirty',[65,67],evidence='di:'+str(MAX-1)+':0:'+str(MAX)+':0',quantum='2:0')
    c.add('lower_before_upper_failure',b,'cash_dirty',[65,67],evidence='bi:ffefffffffffffff:7fefffffffffffff',quantum='1:0')
    c.add('positive_2pow127_rejected',b,'cash_dirty',[67],evidence='b:47e0000000000000',mode='Exact',quantum='1:0')


def catalog_appendix(c,certificate):
    assert certificate['contract_sha256']==CONTRACT_SHA
    assert certificate['inputs']['yield']=='8000001/100000000'
    assert certificate['input_sha256']==sha((canonical_json(certificate['inputs'])+'\n').encode())
    b=F(certificate['base']);rad=F(certificate['root_radicand']);assert b==1+F(8000001,100000000)/2 and rad==b**-145
    lo_m=int(certificate['root_lower_m']);hi_m=int(certificate['root_upper_m']);unit=pow2(certificate['root_unit'])
    assert hi_m==lo_m+1 and certificate['root_degree']==183
    wlo=lo_m*unit;whi=hi_m*unit
    assert wlo**183<rad<whi**183
    annuity=100*b**-31+4*sum((b**-k for k in range(32)),F(0))
    assert annuity==F(certificate['annuity'])
    lower=F(certificate['clean_lower']);upper=F(certificate['clean_upper'])
    assert lower==wlo*annuity-F(152,183) and upper==whi*annuity-F(152,183)
    scale=36;flo=(lower*10**scale).numerator//(lower*10**scale).denominator
    cei=-((-upper*10**scale).numerator//(-upper*10**scale).denominator)
    assert flo<cei and F(flo,10**scale)<lower<upper<F(cei,10**scale) and cei<=MAX
    declared='di:'+amount(flo,scale)+':'+amount(cei,scale)
    inputs=certificate['inputs'];bb=base_for(2)
    bb.update(dated=inputs['D'],issue=inputs['I'],stated_maturity=inputs['M'],true_maturity=inputs['T'],frequency='2',
       eom=inputs['EOM'],quasi=','.join(inputs['Q']),coupon_dates=','.join(inputs['coupons']),coupon='8:0',
       date_cap=str(inputs['date_cap']),settlement=inputs['S'],selection='Explicit:Compound',bundle_cap=str(inputs['bundle_cap']),
       quantum='1:2',evidence=declared)
    p=prepared(bb,instrument(bb));assert p['A']==F(152,183) and p['AI']==F(304,365)
    c.add('appendix_geometry',bb,'plan',[4,23,24,25,26,29,32,63])
    table=[]
    for N in (1000,10000,100000,1000000,10000000,100000000,1000000000):
        row={'nominal':str(N)}
        for mode in ('NearestTiesEven','NearestTiesAwayFromZero'):
            c.add('appendix_certified_accrued',bb,'cash_accrued',[4,61,63],nominal=str(N)+':0',mode=mode)
            for agg in ('RoundTotal','RoundComponentsThenSum'):
                c.add('appendix_certified_invoice',bb,'cash_invoice',[4,61,62,63,64,65],nominal=str(N)+':0',mode=mode,aggregation=agg)
        caplo=reference_count(F(N,100)*lower*100,'NearestTiesEven')
        caphi=reference_count(F(N,100)*upper*100,'NearestTiesEven');assert caplo==caphi
        ai=reference_count(F(N,100)*p['AI']*100,'NearestTiesEven')
        totlo=reference_count(F(N,100)*(lower+p['AI'])*100,'NearestTiesEven')
        tothi=reference_count(F(N,100)*(upper+p['AI'])*100,'NearestTiesEven');assert totlo==tothi
        # Certify no half-cent occurs anywhere in either retained ideal enclosure.
        margins={}
        for name,L,H in (('capital',N*lower,N*upper),('total',N*(lower+p['AI']),N*(upper+p['AI']))):
            midpoint=F(2*(L.numerator//L.denominator)+1,2)
            assert not L<=midpoint<=H
            margins[name]=rat(min(abs(L-midpoint),abs(H-midpoint)))
        row.update(principal_amount=amount(caplo,2),accrued_amount=amount(ai,2),components_amount=amount(caplo+ai,2),
                   total_amount=amount(totlo,2),certified_nonmidpoint_margin_in_cent_units=margins)
        table.append(row)
    assert table[3]['components_amount']=='100820012:2' and table[3]['total_amount']=='100820011:2'
    # Declared exact source price versus its printed6dp number are distinct ACTUAL cash inputs.
    for agg in ('RoundTotal','RoundComponentsThenSum'):
        c.add('appendix_display_price_counterfactual',bb,'cash_invoice',[61,63,68],nominal='1000000000:0',
              evidence='d:99987135:6',aggregation=agg,identity_group='appendix-full-vs-six-decimal-input')
    matches=[]
    for entry in c.bindings.values():
        if entry.get('appendix_nominal'):
            N=entry['appendix_nominal'];component=entry['appendix_component']
        elif entry['reference']=='iiac13#pdf-index=54;section=appendix2;cell=nominal10000-settlementAI':N='10000';component='settlementAI'
        else:continue
        r=next(x for x in table if x['nominal']==N)
        field={'principal':'principal_amount','settlementAI':'accrued_amount','componentTotal':'components_amount'}[component]
        numeric=entry.get('normalized_numeric_interpretation',entry['printed_observation'].replace(',',''))
        actual=value(dec(r[field]));printed=F(numeric)
        match=actual==printed
        assert match or (N=='10000' and component=='settlementAI' and printed==F(83298,1000))
        matches.append({'reference':entry['reference'],'source_observation':entry['printed_observation'],
                        'normalized_numeric_interpretation':numeric,'derived_amount':r[field],'numeric_match':match})
    assert len(matches)==21 and sum(x['numeric_match'] for x in matches)==20
    c.appendix_details={'certificate_sha256':APPENDIX_SHA,'certificate_input_sha256':certificate['input_sha256'],
        'equation_scope':'exact source-example decimal yield8000001/100000000; external declared clean-price ingredient, not binary64 native quote',
        'declared_decimal_evidence':declared,'price_enclosure_verified_by':'strict exact integer183rd-power endpoints plus exact annuity',
        'financial_inputs_are_cash_calls':'yield and source observation are not extra serialized cash inputs',
        'table':table,'primary_cell_comparisons':matches,
        'public_principal_binding':'component invoice receipt minus independently determined accrued receipt; capital stage is not a fabricated public getter',
        'source_punctuation':'literal100,820,01 retained; normalized numeric interpretation100820.01 explicitly separate',
        'guarantee_scope':'UniqueForDeclaredEnclosure certifies supplied external ingredient only; no actual native model enclosure claimed'}


def make_cases(schema,registry,certificate):
    c=Cases(schema,{b['reference']:b for b in registry['bindings']})
    catalog_source(c);catalog_errors(c);catalog_cash(c);catalog_additional(c)
    catalog_source_named_and_boundaries(c);catalog_appendix(c,certificate)
    assert {r['op'] for r in c.rows}==set(schema['operation_input_fields'])
    return c


def read_inputs(repo):
    paths={'generator':Path(__file__).resolve(),'schema':HERE/'canadian_source_cash_schema.json',
           'source-bindings':HERE/'canadian_source_cash_sources.json','adoption-pins':HERE/'canadian_source_cash_adopted_pins.json',
           'contract':repo/'docs/M8_CANADIAN_CONTRACT.md','requirements':repo/'docs/REQUIREMENTS.md',
           'shared-reader':repo/'tests/support/fixture.rs',
           'appendix-certificate':repo/'validation/fixtures/bonds/canadian-appendix-declared-price.json'}
    data={k:p.read_bytes() for k,p in paths.items()}
    assert sha(data['contract'])==CONTRACT_SHA,'wrong adopted contract bytes'
    assert sha(data['appendix-certificate'])==APPENDIX_SHA,'wrong frozen external Appendix certificate'
    assert sha(data['schema'])==SCHEMA_SHA,'unknown or changed approved schema'
    assert sha(data['source-bindings'])==REGISTRY_SHA,'unknown or changed source binding registry'
    schema=json.loads(data['schema']);registry=json.loads(data['source-bindings']);adoption=json.loads(data['adoption-pins'])
    assert len(schema['columns'])==schema['column_count']==74
    assert len(schema['columns'])==len(set(schema['columns']))
    assert len(schema['input_columns'])==25 and len(schema['output_columns'])==43
    assert schema['schema']==1 and len(schema['operation_input_fields'])==14
    assert set(schema['columns'])==set(schema['input_columns']+schema['output_columns']+schema['annotation_columns']+['id'])
    for op,fields in schema['operation_input_fields'].items():assert fields[0]=='op' and set(fields)<=set(schema['input_columns']) and len(fields)==len(set(fields))
    assert len(registry['bindings'])==registry['binding_count']==36
    assert len({b['reference'] for b in registry['bindings']})==36
    for name,digest in adoption['components'].items():
        key='component-'+name;paths[key]=repo/'docs'/name;data[key]=paths[key].read_bytes()
        assert sha(data[key])==digest,'incorporated component changed: '+name
    rows=[line for line in data['requirements'].splitlines() if re.match(rb'^\|FR-CAD-\d\d\|',line)]
    assert len(rows)==72 and [line.split(b'|')[1].decode() for line in rows]==[f'FR-CAD-{i:02}' for i in range(1,73)]
    reqrows=b''.join(line+b'\n' for line in rows)
    assert sha(reqrows)==adoption['requirements_rows_sha256'],'requirements rows changed'
    assert adoption['contract_sha256']==CONTRACT_SHA
    for b in registry['bindings']:
        assert [b['pdf_filename'],b['pdf_sha256']]==registry['pdfs'][b['document']]
        assert b['reference'].startswith(b['document']+'#pdf-index='+str(b['pdf_index'])+';')
    return paths,data,schema,registry,reqrows,json.loads(data['appendix-certificate'])


def corpus_metadata(c):
    rows=c.rows
    semantics=b''.join(canonical_json({k:r[k] for k in c.schema['operation_input_fields'][r['op']]}).encode()+b'\n' for r in rows)
    assert len(c.seen)==len(rows),'duplicate semantic input'
    body=('\t'.join(c.schema['columns'])+'\n'+''.join('\t'.join(r[k] for k in c.schema['columns'])+'\n' for r in rows)).encode()
    profiles=Counter(r['profile'] for r in rows);ops=Counter(r['op'] for r in rows);expectations=Counter(r['expected'] for r in rows)
    stats={'record-count':str(len(rows)),'unique-serialized-inputs':str(len(c.seen)),
       'unique-financial-definitions':str(len({r['definition_sha256'] for r in rows if r['coupon']!='-' and r['definition_sha256']!='-'})),
       'unique-schedule-definitions':str(len({r['definition_sha256'] for r in rows if r['op']=='schedule'})),
       'unique-supplied-definition-digests':str(len({r['definition_sha256'] for r in rows if r['definition_sha256']!='-'})),
       'scope-counts':canonical_json(Counter(r['scope'] for r in rows)),
       'column-count':str(len(c.schema['columns'])),'profile-counts':canonical_json(profiles),
       'operation-counts':canonical_json(ops),'expectation-counts':canonical_json(expectations),
       'overlapping-enumerators-skipped':canonical_json(c.duplicates),'records-sha256':sha(body),'records-fnv64':fnv(body),
       'canonical-inputs-sha256':sha(semantics),'canonical-inputs-fnv64':fnv(semantics)}
    return body,semantics,stats


def family_coverage():
    result={}
    for i in range(1,73):
        if i==1:scope='documentary/source byte audit';detail='registry pins all4 bodies; optional actual byte audit and retained independent content review; no current-market certification'
        elif 2<=i<=8:scope='documentary plus source ingredients';detail='retained fixed primary observations; public exact source discriminators where applicable; numeric formula/price closure remains other corpora'
        elif i==10:scope='API/blind-hand obligation';detail='no fixture row count proves type traits/census/roots or no implicit conversions'
        elif 9<=i<=32:scope='source/date/public-operation expectations';detail='full typed dated operations, exact independent source math and metadata; immediate Rust causes/allocations/bound proof remain separate'
        elif 33<=i<=59:scope='other corpora';detail='source geometry/A/AI/selected coefficient ingredients only; numerical/ideal/inverse and blind hands must establish these families'
        elif i==60:scope='public preparation expectations';detail='explicit conventions and actual last-coupon strategy on both sides of original coupon reset'
        elif 61<=i<=70:scope='cash/public-operation expectations';detail='four recipes, signed policies, evidence/provenance/geometry and stages; later actual public consumer required'
        elif i==71:scope='partial public error expectations plus other corpora/proofs';detail='source/date/cash reachable variants exact; numerical/yield variants and invariant-excluded capacities are not fabricated'
        else:scope='tooling/root whole-gate obligation';detail='this generator/check and reader are interim; all4 corpora/hands and postimplementation checks remain root gate'
        if i==4:detail+=';21 iiac13 Appendix cells,20 numeric agreements and one retained83.298 mismatch; literal100,820,01 punctuation retained'
        if i==63:detail+=';externally authored exact-decimal-y certificate is consumed as declared cash ingredient; not independently rederived ideal/native price'
        result[f'FR-CAD-{i:02}']={'scope':scope,'qualification':detail,'acceptance':'not closed by this annotation'}
    return result


def render(c,data,reqrows,generation):
    body,semantics,stats=corpus_metadata(c)
    pins={key:{'sha256':sha(raw),'fnv64':fnv(raw),'bytes':len(raw)} for key,raw in sorted(data.items())}
    headers=[('schema','1'),('fixture-set','CANADIAN-SOURCE-DATE-CASH-ORACLE'),('domain','bonds'),
       ('description','Independent adopted Canadian source/date/cash exact calendar and integer oracle; preproduction candidate'),
       ('reference','adopted Canadian C0-C10; fixed retained primary cells; independent calendar/Fraction derivation'),
       ('reference-version',SOURCE_VERSION),('reference-source','four retained primary SHA256 hashes and literal registry pinned; optional separate primary-byte audit; no Canadian production or QuantLib C++ read'),
       ('tolerance','exact structured values; IEEE64 accrued bits by integer nearest/even; no tolerance for money'),
       ('reproduction','python3 -I validation/generator/canadian_source_cash_oracles.py --repo . --output validation/fixtures/bonds/canadian-source-cash-oracle.fixture --report validation/reports/canadian-source-cash-oracle.json --check'),
       ('generated-at',generation['generated-at']),('platform',generation['platform']),('python',generation['python']),
       ('source-version',SOURCE_VERSION),('schema-sha256',SCHEMA_SHA),('source-bindings-sha256',REGISTRY_SHA),
       ('adopted-contract-sha256',CONTRACT_SHA),('inputs',canonical_json(pins)),
       ('requirements-rows-sha256',sha(reqrows)),('requirements-rows-fnv64',fnv(reqrows)),
       ('operation-input-fields',canonical_json(c.schema['operation_input_fields'])),
       ('definition-digest-scope','full supplied operation definition; schedule-only subset distinct; caps omitted; not library Eq/Hash'),
       ('canonical-bytes','UTF8 ASCII subset; LF only; final LF'),('status','preproduction candidate; root financial/reader/gate acceptance required')]
    headers.extend(stats.items())
    raw=(''.join(key+': '+value+'\n' for key,value in headers)+'\n').encode()+body
    assert raw.endswith(b'\n') and b'\r' not in raw
    report={'status':'PREPRODUCTION_CANDIDATE_NOT_ACCEPTED','generation':generation,'fixture':{'sha256':sha(raw),'fnv64':fnv(raw),'bytes':len(raw)},
        'statistics':stats,'input_pins':pins,'requirements_rows':{'sha256':sha(reqrows),'fnv64':fnv(reqrows),'count':72},
        'schema':{'sha256':SCHEMA_SHA,'columns':c.schema['columns'],'operation_input_fields':c.schema['operation_input_fields']},
        'independence':{'math':'own exact interval intersections plus independent chi/source common-integer and signed-floor rounding crosschecks',
            'not_read':['future Canadian production','QuantLib C++','Rust Canadian expected financial results'],
            'earlier_own_cash_math':'design context only; this executable computes actual supplied dated states',
            'not_claimed':['Canadian runtime pass','actual allocation or layouts','native foreign runtime','all72 semantic families closed','financial acceptance']},
        'scope_qualifications':[
            'Output rationals are mathematical derivations, not invented public rational inspectors; later public views and numerical checks must bind them.',
            '53,235 attempted scale-grid calls cover39x39 nominal/quantum scales with rotated coupon scale, not39^3.',
            'Overlapping enumerators are explicitly skipped before emission; serialized input uniqueness excludes expected columns and annotations.',
            'Source primary numeric errors are preserved rather than corrected in the observation field; formula transcriptions are normalized math, not verbatim prose.',
            'Source/reference observations are distinct from public Canadian operations.',
            'Typed price bases and numerical factors/risk/solver behavior belong independent numerical/ideal/inverse corpora; the externally authored Appendix exact-decimal-y certificate is a frozen declared-input artifact, not a second independent ideal derivation here.',
            'An exact arithmetic assertion or fixture schema is not a public Rust behavior test.'],
        'source_bindings':{'sha256':REGISTRY_SHA,'count':36,'reference_cells':[b['reference'] for b in c.bindings.values()]},
        'expected_stage_context_counts':dict(sorted(Counter(r['context'] for r in c.rows if r['context']!='-').items())),
        'appendix_certified_cash':c.appendix_details,'all72_family_bindings':family_coverage(),
        'financial_family_annotation_counts':dict(sorted(Counter(x for r in c.rows for x in r['fr_families'].split(',')).items()))}
    report_raw=(json.dumps(report,sort_keys=True,indent=2)+'\n').encode()
    return raw,report_raw


def fixture_generation(raw):
    assert raw.endswith(b'\n') and b'\r' not in raw,'noncanonical fixture byte endings'
    head,body=raw.split(b'\n\n',1)
    pairs=[line.decode().split(': ',1) for line in head.splitlines()]
    assert len(pairs)==len({p[0] for p in pairs}),'duplicate header'
    h=dict(pairs)
    assert pairs[0]==['schema','1'] and h['fixture-set']=='CANADIAN-SOURCE-DATE-CASH-ORACLE'
    assert h['schema-sha256']==SCHEMA_SHA and h['source-bindings-sha256']==REGISTRY_SHA
    return {k:h[k] for k in ('generated-at','platform','python')}


def guard(paths,data):
    for key,path in paths.items():assert path.read_bytes()==data[key],'consumed input changed during run: '+key


def primary_audit(directory,output):
    generator_path=Path(__file__).resolve();generator_bytes=generator_path.read_bytes()
    registry_path=HERE/'canadian_source_cash_sources.json';registry_bytes=registry_path.read_bytes()
    assert sha(registry_bytes)==REGISTRY_SHA
    registry=json.loads(registry_bytes);targets=[directory/name for name,digest in registry['pdfs'].values()]
    assert not output.exists(),'exclusive create: audit report already exists'
    assert output.resolve()!=registry_path.resolve() and all(output.resolve()!=p.resolve() for p in targets),'audit output alias'
    snapshots={p:p.read_bytes() for p in targets}
    pins={}
    for key,(name,digest) in registry['pdfs'].items():
        raw=snapshots[directory/name];assert sha(raw)==digest,'wrong primary source bytes '+name
        pins[key]={'filename':name,'sha256':sha(raw),'fnv64':fnv(raw),'bytes':len(raw)}
    for p,raw in snapshots.items():assert p.read_bytes()==raw,'primary source mutated during audit'
    assert registry_path.read_bytes()==registry_bytes,'registry changed during audit'
    assert generator_path.read_bytes()==generator_bytes,'generator changed during audit'
    payload={'status':'ACTUAL_PRIMARY_BYTE_AUDIT_PASS','actual_utc':datetime.now(timezone.utc).isoformat(),
       'generator_sha256':sha(generator_bytes),'registry_sha256':REGISTRY_SHA,'primary_pins':pins,
       'scope':'optional separate primary-byte audit; does not generate fixture or certify PDF contents/financial correctness'}
    with output.open('x') as out:out.write(json.dumps(payload,indent=2,sort_keys=True)+'\n')
    print('PASS optional primary-byte audit:4 exact retained bodies; no fixture dependency on source directory')


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--repo',type=Path);p.add_argument('--output',type=Path);p.add_argument('--report',type=Path)
    p.add_argument('--check',action='store_true');p.add_argument('--audit-primary-dir',type=Path);p.add_argument('--audit-report',type=Path)
    a=p.parse_args()
    if a.audit_primary_dir is not None:
        assert a.audit_report is not None and a.output is None and a.report is None and not a.check,'source audit is separate from deterministic generation'
        primary_audit(a.audit_primary_dir.resolve(),a.audit_report.resolve());return
    assert a.repo is not None and a.output is not None and a.report is not None and a.audit_report is None,'repo/output/report required'
    repo=a.repo.resolve();paths,data,schema,registry,reqrows,certificate=read_inputs(repo)
    targets=[a.output.resolve(),a.report.resolve()]
    assert targets[0]!=targets[1],'output/report alias refused'
    assert not any(t==path.resolve() for t in targets for path in paths.values()),'output aliases a consumed input'
    if a.check:
        fixture=targets[0].read_bytes();report=targets[1].read_bytes();generation=validate_artifact(fixture,schema,registry,data,reqrows)
    else:
        assert not any(t.exists() for t in targets),'exclusive create: an output already exists'
        generation={'generated-at':datetime.now(timezone.utc).isoformat(timespec='seconds').replace('+00:00','Z'),
                    'platform':platform.platform(),'python':platform.python_version()+' '+platform.python_implementation()}
    cases=make_cases(schema,registry,certificate);expected,expected_report=render(cases,data,reqrows,generation)
    guard(paths,data)
    if a.check:
        assert fixture==expected,'fixture differs from complete recomputation'
        assert report==expected_report,'report differs from complete recomputation (canonical bytes required)'
        print('PASS actual --check: '+str(len(cases.rows))+' unique semantic inputs; fixture/report bytes match')
    else:
        with targets[0].open('xb') as out:out.write(expected)
        with targets[1].open('xb') as out:out.write(expected_report)
        guard(paths,data)
        print('CREATED preproduction candidate: '+str(len(cases.rows))+' unique semantic inputs')
    print('fixture sha256 '+sha(expected));print('report sha256 '+sha(expected_report))


if __name__=='__main__':main()
