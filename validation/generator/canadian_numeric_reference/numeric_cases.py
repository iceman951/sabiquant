"""Deterministic independent prescribed-stage case enumeration and evaluator."""
import math
from fractions import Fraction as F
from datetime import date,timedelta
from numeric_math import *
from numeric_schema import validate,encode
from dated_reference import dated_result,risk_views,back,source_state,ratio_json,ieee_json,ext_json

def inpnum(x):return str(x)
def rq(x,prefix=''):
    x=F(x);return {prefix+'num':str(x.numerator),prefix+'den':str(x.denominator)}
def hs(x):return hex64(ieee(x)[0])
def ext_input(t):return [str(x) for x in t]
def canon_interval(lo,hi,e):
    if lo==hi==0:return '0','0','0'
    while lo%2==0 and hi%2==0:lo//=2;hi//=2;e+=1
    return str(lo),str(hi),str(e)
def ij(b,c):return {'bits':hex64(b),'class':c}

def bounded_pv_input(x):
    """Same exact desired product, constructible five-word cached-source seed.
    Source/factor identity is a genuine primitive INPUT; not dated reachability.
    """
    x=F(x)
    factor=value(1) if 0<x<pow2(-1021) else pow2(1023) if x>pow2(1000) else F(1)
    source=x/factor
    # Avoid merging the distinct midpoint primitive with the existing 5/2*MIN_SUB control.
    if factor==value(1) and source==F(5,2):factor=value(2);source=x/factor
    assert source*factor==x
    assert source.numerator.bit_length()<=160 and source.denominator.bit_length()<=148
    return {**rq(source,'source_'),'factor_bits':hs(factor)}

def evaluate(op,v):
    validate(op,v)
    if op=='log_argument':
        branch,b=logarg(int(v['frequency']),value(v['yield_bits']))
        return {'branch':branch,'argument_or_identity_bits':None if b is None else hex64(b)}
    if op=='log_scale':return ieee_json(int(v['frequency'])*value(v['ell_bits']))
    if op=='exponent':return ij(*exponent(int(v['frequency']),value(v['g_bits']),F(int(v['lambda_num']),int(v['lambda_den']))))
    if op=='tiny_bound':
        f=int(v['frequency']);y=value(v['yield_bits']);lam=F(int(v['lambda_num']),int(v['lambda_den']));q=-y*lam/f;zb,zc=exponent(f,y,lam);z=value(zb);rad=abs(z-q)+lam*y*y/(2*f*f*(1-abs(y)/f))
        return {'exponent':ij(zb,zc),'ideal_linear_exponent':rq(q),'analytic_radius':rq(rad),'identity_still_has_error_bound':z==0 and rad>0}
    if op=='guard':
        z=value(v['z_bits'])
        if not z:return {'radius':rq(0),'branch':'zero_identity','boundary':False,'next_exp_calls':'0'}
        f=int(v['frequency']);y=value(v['yield_bits']);lam=F(int(v['lambda_num']),int(v['lambda_den'])) if 'lambda_num' in v else F(1)
        rad,br,hit=guard(f,abs(y)<pow2(-50),y,z,lam)
        return {'radius':rq(rad),'branch':br,'boundary':hit,'next_exp_calls':'0' if hit else '1'}
    if op=='guard_box':
        lo=int(v['interval_lo_num'])*pow2(int(v['interval_exp']));hi=int(v['interval_hi_num'])*pow2(int(v['interval_exp']))
        a=int(v['box_lo_num'])*pow2(int(v['box_exp']));b=int(v['box_hi_num'])*pow2(int(v['box_exp']))
        return {'closed_intersection':lo<=b and hi>=a}
    if op=='elementary':
        i,a=elementary(v['function'],value(v['argument_bits']))
        return {'ideal':[str(x) for x in i],'allowed':[str(x) for x in a],
                'hypothesis':'observed_positive_finite_exp_2u_plus_H' if v['function']=='exp' else 'native_log_relative_2u',
                'zero_inf_nan_classification':'explicit_native_validation_required' if v['function']=='exp' else 'not_inferred_from_ideal_bits'}
    if op=='mm_affine':
        d,n,b,c=mm(value(v['yield_bits']),int(v['days_total']),int(v['days_remaining']))
        return {'den_raw':str(d),'num_raw':str(n),'factor':None if b is None else ij(b,c),'domain':c!='domain'}
    if op=='pv':return ieee_json(F(int(v['source_num']),int(v['source_den']))*value(v['factor_bits']))
    if op=='source_ieee':return ieee_json(F(int(v['source_num']),int(v['source_den'])))
    if op=='extended':return {'extended':ext_input(ext53(F(int(v['num']),int(v['den']))*pow2(int(v['binary_exp']))))}
    if op=='risk_terms':
        c=v['convention'];p=value(v['pv_bits']);y=value(v['yield_bits'])
        f=int(v['frequency']) if c=='compound' else 2;lam=F(int(v['lambda_num']),int(v['lambda_den'])) if c=='compound' else F(1)
        D=int(v['days_total']) if c=='mm' else 1;R=int(v['days_remaining']) if c=='mm' else 0
        return {'terms':[ext_input(risk_term(c,k,p,y,f,lam,D,R)) for k in range(3)]}
    if op=='retained':
        ts=[[tuple(map(int,t)) for t in arr] for arr in v['terms']]
        return risk_views(ts,value(v['dirty_bits']))
    if op=='dirty_sum':return ieee_json(sum((value(x) for x in v['pv_bits']),F(0)))
    if op=='price_bases':
        c=v['convention'];d=value(v['dirty_bits']);ai=value(v['ai_bits']);a=value(v['a_bits']) if c=='compound' else F(0)
        return {b:ij(*bridge(c,b,d,a,ai)) for b in ('ModelDirty','Clean','SettlementDirty')}
    if op=='target':
        c=v['convention'];b=v['basis'];x=value(v['value_bits'])
        if b=='Clean':x+=value(v['quote_accrued_bits'])
        elif b=='SettlementDirty' and c=='compound':x+=-value(v['ai_bits'])+value(v['a_bits'])
        bb,cc=ieee(x)
        return {'resolved':ij(bb,cc),'positive':cc!='overflow' and value(bb)>0,'zero_evaluation_rejection':cc=='overflow' or value(bb)<=0}
    if op=='residual':
        s,scaled,raw=residual(value(v['dirty_bits']),value(v['target_bits']))
        return {'scale':str(s),'callback':ij(*scaled),'amount_residual':ij(*raw)}
    if op=='dated_mm':return dated_result(v['definition'],value(v['yield_bits']))
    if op=='dated_compound_seeded':return dated_result(v['definition'],value(v['yield_bits']),value(v['g_bits']),v['factor_bits'])
    raise AssertionError('operation not handled')

def base_definition(f,shape='regular',eom='Ignore',C=(5,0),settlement='inside',delay=0):
    anchor=date(2028,3,31);qs=[back(anchor,k,12//f,eom=='Preserve') for k in range(10,-1,-1)]
    D,Fst,L,M,A=qs[6],qs[7],qs[9],qs[10],qs[10]
    if shape=='short_first':D+=timedelta(days=1)
    if shape=='long_first':D=qs[5]+timedelta(days=1)
    if shape=='short_last':M=qs[9]+timedelta(days=10)
    if shape=='long_last':L=qs[8];M=qs[9]+timedelta(days=10)
    if shape=='combined':D=qs[5]+timedelta(days=1);L=qs[8];M=qs[9]+timedelta(days=10)
    if shape=='single_composite':D=qs[8]+timedelta(days=2);M=qs[9]+timedelta(days=10);Fst=M;L=D
    if shape=='single_same_cell':D=qs[9]+timedelta(days=2);M=qs[9]+timedelta(days=10);Fst=M;L=D
    if shape=='single_regular':D=qs[9];M=qs[10];Fst=M;L=D
    S=D if settlement=='empty' else Fst if settlement=='reset' and Fst<M else M-timedelta(days=1) if settlement=='last_day' else D+timedelta(days=1)
    T=M+timedelta(days=delay)
    coeff,scale=C
    while scale and coeff%10==0:coeff//=10;scale-=1
    return {'accrual_start':D.isoformat(),'original_issue':D.isoformat(),'stated_maturity':M.isoformat(),'true_maturity':T.isoformat(),
        'first_coupon_date':Fst.isoformat(),'last_coupon_start_date':L.isoformat(),'anchor':A.isoformat(),'frequency':str(f),
        'end_of_month':eom,'coupon_coefficient':str(coeff),'coupon_scale':str(scale),'source_policy':'CanonicalIiac13LocalV1',
        'settlement':S.isoformat(),'selection':{'kind':'Explicit','convention':'MoneyMarket'},'date_cap':'20000','max_bundles':'20000'}

FAMILY_BINDINGS={
'tiny_bound':'36,37','log_argument':'35,36','log_scale':'35,37','exponent':'33,34,36','guard':'35,36,37','guard_box':'37',
'elementary':'37','mm_affine':'38,39','pv':'40','extended':'47,51','risk_terms':'47,50,51',
'retained':'48,49,50,51','dirty_sum':'41','source_ieee':'30,32','price_bases':'42,43,44','target':'43,53,54','residual':'54',
'dated_mm':'09,13,15,16,17,18,19,20,21,23,24,25,26,27,28,29,30,32,38,39,41,42,47,48,49,50,51,60,70',
'dated_compound_seeded':'09,13,15,16,17,18,19,20,21,23,24,25,26,27,28,29,30,32,33,34,35,36,40,41,42,43,44,47,48,49,50,51,60,70'}

def cases():
    seen=set();seq=0
    def emit(op,v,profile='exact_seed',scope='primitive'):
        nonlocal seq
        validate(op,v);key=op+'\t'+encode(v)
        if key in seen:return None
        seen.add(key);seq+=1
        return {'id':f'CADNUM-{seq:06d}','profile':profile,'operation':op,'scope':scope,'input':v,
            'families':','.join('CAD'+x for x in FAMILY_BINDINGS[op].split(',')),
            'binding':op+':whole_result;'+('geometry_valid_seeds_not_public_native_reachability' if profile=='dated_seeded' else 'conditional_elementary_profile_not_native_bits' if profile=='native_2u' else 'prescribed_exact_stage')}
    def add(op,v,*args):
        r=emit(op,v,*args)
        if r is not None:return [r]
        return []
    lambdas=[F(1,184),F(1,183),F(1,2),F(1),F(3,2),F(7,3),F(365,184),F(19999),F(19999)+F(183,184)]
    # Exact quote branch, minimum delta and tiny boundaries across all frequencies.
    for f in (2,4,12):
        floorb=bits(float(-f));half=bits(float(-F(f,2)));t=bits(2.**-50)
        quotes={0,1,SIGN|1,bits(.05),bits(-.05),MAX_BITS,MAX_BITS|SIGN,floorb,floorb-1,floorb+1,half,half-1,half+1,t-1,t,t+1,SIGN|(t-1),SIGN|t,SIGN|(t+1)}
        for b in sorted(quotes):yield from add('log_argument',{'frequency':str(f),'yield_bits':hex64(b)})
        for ell in [F(0),pow2(-1074),-pow2(-1074),F(1,3),-F(1,3),F(709),F(-745),value(MAX_BITS)]:
            yield from add('log_scale',{'frequency':str(f),'ell_bits':hs(ell)})
        for g in [F(0),pow2(-1074),-pow2(-1074),pow2(-50),-pow2(-50),F(1,20),F(-1),F(700),F(-700),value(MAX_BITS)]:
            for lam in lambdas:yield from add('exponent',{'frequency':str(f),'g_bits':hs(g),**rq(lam,'lambda_')})
        # Exact seeded log-boundary/far guard controls.
        for box in (Z0,ZI):
            b=ieee(F(sum(box),2*(1<<256)))[0]
            for j in range(-12,13):
                yield from add('guard',{'frequency':str(f),'yield_bits':hs(F(1,20)),'z_bits':hex64(b+j)})
        for z in [pow2(24),-pow2(24),value(MAX_BITS),-value(MAX_BITS),F(1),F(-1),pow2(-1074),-pow2(-1074)]:
            yield from add('guard',{'frequency':str(f),'yield_bits':hs(F(1,20)),'z_bits':hs(z)})
        for y in [pow2(-1074),-pow2(-1074),value(bits(2.**-50)-1),-value(bits(2.**-50)-1)]:
            for lam in lambdas:
                yield from add('tiny_bound',{'frequency':str(f),'yield_bits':hs(y),**rq(lam,'lambda_')})
                z=exponent(f,y,lam)[0]
                if z&~SIGN:yield from add('guard',{'frequency':str(f),'yield_bits':hs(y),'z_bits':hex64(z),**rq(lam,'lambda_')})
    yield from add('guard',{'z_bits':'0000000000000000'})
    # Abstract touch/open-mutant controls at both exact box endpoints, no dated claim.
    for box in (Z0,ZI):
        for a,b in ((box[0]-1,box[0]-1),(box[0]-1,box[0]),(box[0],box[0]),(box[1],box[1]),(box[1],box[1]+1),(box[1]+1,box[1]+1)):
            aa,bb,ee=canon_interval(a,b,-256);cc,dd,ff=canon_interval(box[0],box[1],-256)
            yield from add('guard_box',{'interval_lo_num':aa,'interval_hi_num':bb,'interval_exp':ee,'box_lo_num':cc,'box_hi_num':dd,'box_exp':ff},'exact_seed','abstract_interval_superset')
    # Certified elementary intervals: ordinary, tiny, floor and range thresholds.
    for fun in ('ln','ln1p','exp'):
        if fun=='ln':args=[value(1),value(MIN_NORMAL),F(1,1<<55),F(1,2),F(1),F(2),F(12),value(MAX_BITS)]
        elif fun=='ln1p':args=[-1+pow2(-53),F(-1,2),F(-1,3),-pow2(-50),-pow2(-1074),F(0),pow2(-1074),pow2(-50),F(1,3),F(1),F(12),value(MAX_BITS)]
        else:
            args=[F(-750),F(-746),F(-745),F(-744),F(-710),F(-1),-pow2(-1074),F(0),pow2(-1074),F(1),F(709),F(710)]
            for box in (Z0,ZI):
                b=ieee(F(sum(box),2*(1<<256)))[0];args += [value(b-1),value(b),value(b+1)]
        if fun in ('ln','ln1p'):
            for f in (2,4,12):
                for yy in (value(bits(float(-f))-1),F(1,20)):
                    branch,argb=logarg(f,yy)
                    if branch==fun:args.append(value(argb))
        for arg in args:yield from add('elementary',{'function':fun,'argument_bits':hs(arg)},'native_2u','conditional_native_profile')
    # MM exact affine domains: non-dyadic neighbors, outside compound, rho0, huge rescue.
    for D in (1,2,7,28,89,181,184,365,366,10000,3652058):
        floorb=ieee(-F(365,D))[0]
        ys=[F(0),F(1,20),F(-1,20),value(MAX_BITS),value(1),-value(1),value(floorb-1),value(floorb),value(floorb+1)]
        if D==1:ys.append(F(-300))
        for y in ys:
            for R in sorted({0,D//2,D-1}):yield from add('mm_affine',{'yield_bits':hs(y),'days_total':str(D),'days_remaining':str(R)})
    # Source*factor direct rounding and source IEEE full scale controls.
    sources=[F(0),F(1,10**38),F(1,184*10**38),F(5,2),F(5,12),F((1<<127)-1),F((1<<127)-1,12),F((1<<127)-1,10**38)]
    sources += [F(1,10**s) for s in range(39)]
    factors=[value(1),value(2),value((1<<52)-1),value(1<<52),F(1),F(3,2),value(MAX_BITS)]
    for src in sources:
        yield from add('source_ieee',rq(src,'source_'))
        for fact in factors:yield from add('pv',{**rq(src,'source_'),'factor_bits':hs(fact)})
    for b in (0,1,2,3,(1<<52)-2,(1<<52)-1,1<<52,(1<<52)+1,bits(1.),MAX_BITS-1):
        lo,hi=value(b),value(b+1)
        for eps in (-F(1,1<<60),F(0),F(1,1<<60)):
            src=(lo+hi)/2+eps*(hi-lo)
            yield from add('pv',bounded_pv_input(src),'exact_seed','source_rounding_superset')
    ov=pow2(1024)-pow2(970)
    for src in (ov-pow2(969),ov,ov+pow2(969)):
        yield from add('pv',bounded_pv_input(src),'exact_seed','source_rounding_superset')
    # Extended whole-term boundaries and exponent retained extents, signed.
    for exp in (-8192,-4208,-3161,-1083,-1075,-1074,-1022,0,1023,1198,1293):
        for m in (F(1),1+pow2(-53),1+3*pow2(-53),2-pow2(-53),F(1,3)):
            for sign in (-1,1):yield from add('extended',{**rq(sign*m),'binary_exp':str(exp)})
    yield from add('extended',{**rq(0),'binary_exp':'0'})
    # ALL THREE exact term observations, branch inputs all read.
    for c in ('compound','mm'):
        for pv in (F(0),value(1),F(1),F(100),F((1<<127)-1),value(MAX_BITS)):
            for y in (F(0),F(1,20),F(-1,20),pow2(370),pow2(540),value(MAX_BITS)):
                if c=='compound':
                    for f in (2,4,12):
                        for lam in (F(1,184),F(1,2),F(3,2),F(19999)):
                            yield from add('risk_terms',{'convention':c,'pv_bits':hs(pv),'yield_bits':hs(y),'frequency':str(f),**rq(lam,'lambda_')})
                else:
                    for D,R in ((1,0),(181,0),(365,184),(3652058,3652057)):
                        if 365+y*D>0:yield from add('risk_terms',{'convention':c,'pv_bits':hs(pv),'yield_bits':hs(y),'days_total':str(D),'days_remaining':str(R)})
    # Group six getters, including independent raw failure vs finite normalized and vice versa.
    for magnitude in (-2000,-1075,-1074,-1022,-1000,-1,0,1000,1023,1040,1293):
        ts=[[ext_input(ext53(-pow2(magnitude)))],[ext_input(ext53(pow2(magnitude)))],[ext_input(ext53(pow2(magnitude)))]]
        for db in (1,MIN_NORMAL,bits(1.),bits(2.**500),MAX_BITS):
            yield from add('retained',{'terms':ts,'dirty_bits':hex64(db)},'exact_seed','retained_superset')
    zero=[['0','0','0']]
    yield from add('retained',{'terms':[zero,zero,zero],'dirty_bits':hs(1)})
    # Exact dirty reduction many-binade/carry and primitive overflow.
    for arr in ([0,1],[1,1],[1,2,3],[MIN_NORMAL-1,1],[bits(1.),1],[MAX_BITS,MAX_BITS],[bits(1.)]*20000):
        yield from add('dirty_sum',{'pv_bits':[hex64(x) for x in arr]})
    # Three bases and reverse target shifts; preserve signed external values.
    for c in ('compound','mm'):
        for d,a,ai in ((1,0,0),(1,2,4),(1,2**54,2**54),(1,2**54,2**54+4),(100,3,4),((1<<127)-1,5,6)):
            v={'convention':c,'dirty_bits':hs(d),'ai_bits':hs(ai)}
            if c=='compound':v['a_bits']=hs(a)
            yield from add('price_bases',v)
        for basis in ('ModelDirty','Clean','SettlementDirty'):
            for val in (-100,-1,0,1,100,value(MAX_BITS)):
                v={'convention':c,'basis':basis,'value_bits':hs(val)}
                if basis=='Clean':v['quote_accrued_bits']=hs(101)
                elif basis=='SettlementDirty' and c=='compound':v.update({'a_bits':hs(101),'ai_bits':hs(1)})
                yield from add('target',v)
    rb=(1,2,MIN_NORMAL,bits(1.),bits(2.**969),bits(2.**970)-1,bits(2.**970),bits(2.**970)+1,MAX_BITS-1,MAX_BITS)
    for d in rb:
        for t in rb:yield from add('residual',{'dirty_bits':hex64(d),'target_bits':hex64(t)})
    # Complete dated matrix; labels never enter canonical definition identity.
    shapes=('regular','short_first','long_first','short_last','long_last','combined','single_composite','single_same_cell','single_regular')
    for f in (2,4,12):
        for shape in shapes:
            for eom in ('Ignore','Preserve'):
                for C in ((0,0),(5,0),((1<<127)-1,0),(1,38)):
                    d=base_definition(f,shape,eom,C,'inside',7)
                    for y in (F(0),F(1,20),value(MAX_BITS)):
                        yield from add('dated_mm',{'definition':d,'yield_bits':hs(y)},'dated_exact','valid_dated')
                    dc={**d,'selection':{'kind':'Explicit','convention':'Compound'}}
                    n=len(source_state(dc)['future'])
                    for y,g,fs in ((F(0),F(0),[hs(1)]*n),(F(1,20),F(1,20),[hs(F(1,2))]*n)):
                        yield from add('dated_compound_seeded',{'definition':dc,'yield_bits':hs(y),'g_bits':hs(g),'factor_bits':fs},'dated_seeded','valid_geometry_seeded_stages')
    # Full scale grid, empty AI[S,S], reset original ordinal and strategy switching.
    for f in (2,4,12):
        for s in range(39):
            d=base_definition(f,'combined','Preserve',(1,s),'empty',0)
            yield from add('dated_mm',{'definition':d,'yield_bits':hs(F(1,20))},'dated_exact','valid_dated')
        for settle in ('reset','last_day'):
            for sel in ({'kind':'Explicit','convention':'MoneyMarket'},{'kind':'LastCouponMoneyMarket'}):
                d=base_definition(f,'regular','Preserve',(5,0),settle,0);d['selection']=sel
                if source_state(d)['convention']=='MoneyMarket':yield from add('dated_mm',{'definition':d,'yield_bits':hs(F(1,20))},'dated_exact','valid_dated')
                else:
                    n=len(source_state(d)['future']);yield from add('dated_compound_seeded',{'definition':d,'yield_bits':hs(0),'g_bits':hs(0),'factor_bits':[hs(1)]*n},'dated_seeded','valid_geometry_seeded_stages')
    # Explicit normal-raw/normalized-underflow witnesses, original ordinal after ceased coupons.
    for convention,y in (('mm',pow2(540)),('mm',pow2(370)),('compound',pow2(540))):
        d={'accrual_start':'2025-03-01','original_issue':'2025-03-01','stated_maturity':'2026-03-01','true_maturity':'2026-03-01',
            'first_coupon_date':'2025-09-01','last_coupon_start_date':'2025-09-01','anchor':'2026-03-01','frequency':'2','end_of_month':'Ignore',
            'coupon_coefficient':str((1<<127)-1),'coupon_scale':'0','source_policy':'CanonicalIiac13LocalV1',
            'settlement':'2025-06-01' if convention=='mm' else '2025-08-31','selection':{'kind':'Explicit','convention':'MoneyMarket' if convention=='mm' else 'Compound'},'date_cap':'3','max_bundles':'2'}
        if convention=='mm':yield from add('dated_mm',{'definition':d,'yield_bits':hs(y)},'dated_exact','valid_dated_underflow_witness')
        else:
            # Independent certified ideal seed choice. The seeds stay primitive inputs.
            f=2;br,ab=logarg(f,y);lo,hi=log_interval(value(ab),br=='ln1p');eb=ieee((lo+hi)/2)[0];gb=ieee(f*value(eb))[0]
            state=source_state(d);fs=[]
            for i,dt in state['future']:
                from dated_reference import geometry
                _,lam,_=geometry(state['q'],state['S'],dt);zb,zc=exponent(f,value(gb),lam);el,eu=exp_interval(value(zb));fs.append(hex64(ieee((el+eu)/2)[0]))
            yield from add('dated_compound_seeded',{'definition':d,'yield_bits':hs(y),'g_bits':hex64(gb),'factor_bits':fs},'dated_seeded','valid_geometry_seeded_underflow_witness')
    # Actual MM minus300 one-day; true-maturity delay admits subnormal factor/PV/dirty.
    for M,T,y in (('2025-03-02','2025-03-02',F(-300)),('2025-03-02','2025-03-02',F(-280)),('2025-03-02','9999-12-31',value(MAX_BITS))):
        d={'accrual_start':'2025-03-01','original_issue':'2025-03-01','stated_maturity':M,'true_maturity':T,'first_coupon_date':M,
            'last_coupon_start_date':'2025-03-01','anchor':'2025-09-01','frequency':'2','end_of_month':'Ignore','coupon_coefficient':'0','coupon_scale':'0',
            'source_policy':'CanonicalIiac13LocalV1','settlement':'2025-03-01','selection':{'kind':'Explicit','convention':'MoneyMarket'},'date_cap':'2','max_bundles':'1'}
        yield from add('dated_mm',{'definition':d,'yield_bits':hs(y)},'dated_exact','valid_dated_range_witness')

    # Maximum complete Q count with a long-first single coupon; minimal caps exact.
    for f in (2,4,12):
        A=date(9999,12,31);periods=min(19999,((A.year-1)*12+A.month-1)//(12//f));D=back(A,periods,12//f,False)
        d={'accrual_start':D.isoformat(),'original_issue':D.isoformat(),'stated_maturity':A.isoformat(),'true_maturity':A.isoformat(),
            'first_coupon_date':A.isoformat(),'last_coupon_start_date':D.isoformat(),'anchor':A.isoformat(),'frequency':str(f),'end_of_month':'Ignore',
            'coupon_coefficient':str((1<<127)-1),'coupon_scale':'0','source_policy':'CanonicalIiac13LocalV1','settlement':D.isoformat(),
            'selection':{'kind':'Explicit','convention':'MoneyMarket'},'date_cap':'20000','max_bundles':'1'}
        # f2/4 cannot span 19999 complete periods within years1..9999: use
        # maximal civil-valid multiple, still below cap. f12 attains full cap.
        yield from add('dated_mm',{'definition':d,'yield_bits':hs(0)},'dated_exact','valid_dated_long_first_cap')
    # Geometry-valid compound negative settlement-dirty seeded stage, ordinary finite factor.
    d={'accrual_start':'2025-09-01','original_issue':'2025-09-01','stated_maturity':'2026-03-01','true_maturity':'2026-03-01',
        'first_coupon_date':'2026-03-01','last_coupon_start_date':'2025-09-01','anchor':'2026-03-01','frequency':'2','end_of_month':'Ignore',
        'coupon_coefficient':'5','coupon_scale':'0','source_policy':'CanonicalIiac13LocalV1','settlement':'2025-12-01',
        'selection':{'kind':'Explicit','convention':'Compound'},'date_cap':'2','max_bundles':'1'}
    y=value(MAX_BITS);br,ab=logarg(2,y);lo,hi=log_interval(value(ab),br=='ln1p');gb=ieee(2*value(ieee((lo+hi)/2)[0]))[0]
    st=source_state(d);from dated_reference import geometry
    _,lam,_=geometry(st['q'],st['S'],st['M']);zb,zc=exponent(2,value(gb),lam);lo,hi=exp_interval(value(zb));fb=ieee((lo+hi)/2)[0]
    yield from add('dated_compound_seeded',{'definition':d,'yield_bits':hs(y),'g_bits':hex64(gb),'factor_bits':[hex64(fb)]},'dated_seeded','valid_geometry_seeded_negative_allin')
