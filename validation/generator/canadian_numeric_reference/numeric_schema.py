"""Strict, finite, canonical schema1 input grammar (proposal v2).
Unknown keys at every object layer and all noncanonical spellings fail closed.
"""
import json,re
from fractions import Fraction as F
from datetime import date
from numeric_math import value,hex64,pow2
COLUMNS=('id','profile','operation','scope','input','expected','families','binding')
PROFILES=('exact_seed','native_2u','dated_exact','dated_seeded')
DEFINITION_KEYS=('accrual_start','original_issue','stated_maturity','true_maturity','first_coupon_date','last_coupon_start_date','anchor','frequency','end_of_month','coupon_coefficient','coupon_scale','source_policy','settlement','selection','date_cap','max_bundles')
BASIS=('ModelDirty','Clean','SettlementDirty')

def obj(v,keys):
    if type(v)!=dict or set(v)!=set(keys):raise ValueError('unknown/missing nested keys')

def integer(v,lo=None,hi=None):
    if type(v)!=str or not re.fullmatch(r'0|-?[1-9][0-9]*',v):raise ValueError('noncanonical integer')
    n=int(v)
    if (lo is not None and n<lo) or (hi is not None and n>hi):raise ValueError('integer bound')
    return n

def f64(v,positive=False):
    if type(v)!=str or not re.fullmatch('[0-9a-f]{16}',v):raise ValueError('IEEE spelling')
    b=int(v,16)
    if (b>>52)&2047==2047 or b==1<<63:raise ValueError('nonfinite/negative-zero alias')
    q=value(b)
    if positive and q<=0:raise ValueError('positive IEEE seed required')
    return q

def ratio(v,a='num',b='den',positive=False,maxbits=7168):
    n=integer(v[a]);d=integer(v[b],1)
    if max(abs(n).bit_length(),d.bit_length())>maxbits:raise ValueError('ratio primitive bound')
    r=F(n,d)
    if r.numerator!=n or r.denominator!=d or (positive and n<=0):raise ValueError('fraction not reduced')
    return r

def freq(v):
    if v not in ('2','4','12'):raise ValueError('frequency enum')
    return int(v)

def enum(v,choices):
    if v not in choices:raise ValueError('enum tag')

def ext(t):
    if type(t)!=list or len(t)!=3:raise ValueError('extended tuple')
    s=integer(t[0],-1,1);q=integer(t[1],0,(1<<53)-1);e=integer(t[2],-8192,2048)
    if (s==0 and (q!=0 or e!=0)) or (s!=0 and q<(1<<52)):raise ValueError('extended canonical form')

def definition(v):
    obj(v,DEFINITION_KEYS)
    for k in ('accrual_start','original_issue','stated_maturity','true_maturity','first_coupon_date','last_coupon_start_date','anchor','settlement'):
        if type(v[k])!=str or not re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}',v[k]):raise ValueError('date spelling')
        date.fromisoformat(v[k])
    freq(v['frequency']);enum(v['end_of_month'],('Ignore','Preserve'))
    integer(v['coupon_coefficient'],0,(1<<127)-1);integer(v['coupon_scale'],0,38)
    # DecimalAmount value identity removes trailing decimal zeros.
    c=int(v['coupon_coefficient']);s=int(v['coupon_scale'])
    if (c==0 and s!=0) or (s and c%10==0):raise ValueError('decimal value alias')
    enum(v['source_policy'],('CanonicalIiac13LocalV1',))
    sel=v['selection']
    if type(sel)!=dict:raise ValueError('typed selection')
    if sel.get('kind')=='Explicit':
        obj(sel,('kind','convention'));enum(sel['convention'],('Compound','MoneyMarket'))
    else:
        obj(sel,('kind',));enum(sel['kind'],('LastCouponMoneyMarket',))
    integer(v['date_cap'],2,20000);integer(v['max_bundles'],0,20000)

def validate(op,v):
    if op=='log_argument':
        obj(v,('frequency','yield_bits'));freq(v['frequency']);f64(v['yield_bits'])
    elif op=='log_scale':
        obj(v,('frequency','ell_bits'));freq(v['frequency']);f64(v['ell_bits'])
    elif op=='exponent':
        obj(v,('frequency','g_bits','lambda_num','lambda_den'));freq(v['frequency']);f64(v['g_bits']);r=ratio(v,'lambda_num','lambda_den',True,maxbits=30)
        if (not F(1,184)<=r<20000 or r.denominator>33856):raise ValueError('coordinate range')
    elif op=='tiny_bound':
        obj(v,('frequency','yield_bits','lambda_num','lambda_den'));f=freq(v['frequency']);y=f64(v['yield_bits']);r=ratio(v,'lambda_num','lambda_den',True,maxbits=30)
        if not abs(y)<pow2(-50) or (not F(1,184)<=r<20000 or r.denominator>33856):raise ValueError('tiny bound domain')
    elif op=='guard':
        z=f64(v.get('z_bits'))
        if not z:obj(v,('z_bits',))
        else:
            y=f64(v.get('yield_bits'));f=freq(v.get('frequency'))
            if y<=-f:raise ValueError('compound quote domain')
            tiny=abs(y)<pow2(-50)
            keys=('frequency','yield_bits','z_bits','lambda_num','lambda_den') if tiny else ('frequency','yield_bits','z_bits')
            obj(v,keys)
            if tiny:
                r=ratio(v,'lambda_num','lambda_den',True,maxbits=30)
                if (not F(1,184)<=r<20000 or r.denominator>33856):raise ValueError('coordinate range')
                from numeric_math import exponent
                if value(exponent(f,y,r)[0])!=z:raise ValueError('tiny guard z coupling')
    elif op=='guard_box':
        obj(v,('interval_lo_num','interval_hi_num','interval_exp','box_lo_num','box_hi_num','box_exp'))
        a=integer(v['interval_lo_num']);b=integer(v['interval_hi_num']);c=integer(v['box_lo_num']);d=integer(v['box_hi_num'])
        if a>b or c>d:raise ValueError('interval order')
        for lo,hi,e in ((a,b,v['interval_exp']),(c,d,v['box_exp'])):
            if max(abs(lo).bit_length(),abs(hi).bit_length())>1280:raise ValueError('interval primitive width')
            if (lo==0 and hi==0 and e!='0') or ((lo or hi) and lo%2==0 and hi%2==0):raise ValueError('dyadic interval common-power alias')
        integer(v['interval_exp'],-8192,2048);integer(v['box_exp'],-8192,2048)
    elif op=='elementary':
        obj(v,('function','argument_bits'));enum(v['function'],('ln','ln1p','exp'));a=f64(v['argument_bits'])
        if (v['function']=='ln' and a<=0) or (v['function']=='ln1p' and a<=-1) or (v['function']=='exp' and abs(a)>800):raise ValueError('elementary certificate domain')
    elif op=='mm_affine':
        obj(v,('yield_bits','days_total','days_remaining'));f64(v['yield_bits']);D=integer(v['days_total'],1,3652058);integer(v['days_remaining'],0,D-1)
    elif op in ('pv','source_ieee'):
        obj(v,('source_num','source_den','factor_bits') if op=='pv' else ('source_num','source_den'))
        r=ratio(v,'source_num','source_den',maxbits=160)
        if r.denominator.bit_length()>148:raise ValueError('reviewed source denominator width')
        if r<0:raise ValueError('source sign')
        if op=='pv':f64(v['factor_bits'],True)
    elif op=='extended':
        obj(v,('num','den','binary_exp'));ratio(v,maxbits=7168);integer(v['binary_exp'],-8192,2048)
    elif op=='risk_terms':
        c=v.get('convention');enum(c,('compound','mm'))
        keys=('convention','pv_bits','yield_bits','frequency','lambda_num','lambda_den') if c=='compound' else ('convention','pv_bits','yield_bits','days_total','days_remaining')
        obj(v,keys);p=f64(v['pv_bits']);y=f64(v['yield_bits'])
        if p<0:raise ValueError('PV sign')
        if c=='compound':
            f=freq(v['frequency']);r=ratio(v,'lambda_num','lambda_den',True,maxbits=30)
            if y<=-f or (not F(1,184)<=r<20000 or r.denominator>33856):raise ValueError('compound domain')
        else:
            D=integer(v['days_total'],1,3652058);R=integer(v['days_remaining'],0,D-1)
            if 365+y*D<=0:raise ValueError('MM domain')
    elif op=='retained':
        obj(v,('terms','dirty_bits'));f64(v['dirty_bits'],True)
        if type(v['terms'])!=list or len(v['terms'])!=3:raise ValueError('three measure arrays required')
        for ts in v['terms']:
            if type(ts)!=list or not 1<=len(ts)<=20000:raise ValueError('term count')
            for t in ts:ext(t)
    elif op=='dirty_sum':
        obj(v,('pv_bits',));ts=v['pv_bits']
        if type(ts)!=list or not 1<=len(ts)<=20000:raise ValueError('PV count')
        if any(f64(t)<0 for t in ts):raise ValueError('PV sign')
    elif op=='price_bases':
        c=v.get('convention');enum(c,('compound','mm'))
        obj(v,('convention','dirty_bits','a_bits','ai_bits') if c=='compound' else ('convention','dirty_bits','ai_bits'))
        f64(v['dirty_bits'],True);f64(v['ai_bits'])
        if c=='compound':f64(v['a_bits'])
    elif op=='target':
        c=v.get('convention');b=v.get('basis');enum(c,('compound','mm'));enum(b,BASIS)
        keys=['convention','basis','value_bits']
        if b=='Clean':keys+=['quote_accrued_bits']
        elif b=='SettlementDirty' and c=='compound':keys+=['a_bits','ai_bits']
        obj(v,keys);f64(v['value_bits'])
        for k in keys[3:]:f64(v[k])
    elif op=='residual':
        obj(v,('dirty_bits','target_bits'));f64(v['dirty_bits'],True);f64(v['target_bits'],True)
    elif op in ('dated_mm','dated_compound_seeded'):
        obj(v,('definition','yield_bits') if op=='dated_mm' else ('definition','yield_bits','g_bits','factor_bits'))
        definition(v['definition']);f64(v['yield_bits'])
        if op=='dated_compound_seeded':
            f64(v['g_bits']);ts=v['factor_bits']
            if type(ts)!=list or not 1<=len(ts)<=20000:raise ValueError('factor count')
            for t in ts:f64(t,True)
        from dated_reference import reconstruct,geometry
        state=reconstruct(v['definition']);want='MoneyMarket' if op=='dated_mm' else 'Compound'
        if state['convention']!=want:raise ValueError('operation/resolved convention mismatch')
        if op=='dated_compound_seeded':
            yy=f64(v['yield_bits']);gg=f64(v['g_bits'])
            if yy<=-state['f']:raise ValueError('dated compound quote outside authoring domain')
            if abs(yy)<pow2(-50) and gg!=yy:raise ValueError('dated tiny/zero g must equal represented y')
            if len(v['factor_bits'])!=len(state['future']):raise ValueError('factor count/future relation')
            from numeric_math import exponent
            for i,dt in state['future']:
                _,lam,_=geometry(state['q'],state['S'],dt)
                if exponent(state['f'],f64(v['g_bits']),lam)[1]=='overflow':raise ValueError('dated seeded exponent overflow outside authoring domain')
    else:raise ValueError('unknown operation')

def encode(v):return json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=True)
def decode(op,s):
    v=json.loads(s)
    if encode(v)!=s:raise ValueError('noncanonical JSON')
    validate(op,v);return v
