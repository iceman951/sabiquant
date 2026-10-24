"""Independent direct-anchor civil reconstruction and exact interval intersection.
No source/production schedule/date/amount implementation is imported.
"""
from datetime import date
import calendar
from fractions import Fraction as F
from numeric_math import *
from numeric_schema import definition

def ds(x):return date.fromisoformat(x)
def iso(x):return x.isoformat()
def back(anchor,k,step,preserve):
    m=anchor.year*12+anchor.month-1-k*step;y,m=divmod(m,12);m+=1
    if not 1<=y<=9999:raise ValueError('cycle outside civil range')
    last=calendar.monthrange(y,m)[1]
    anchor_last=calendar.monthrange(anchor.year,anchor.month)[1]
    day=last if preserve and anchor.day==anchor_last else min(anchor.day,last)
    return date(y,m,day)

def reconstruct(v):
    definition(v)
    D,I,M,T,Fst,L,A,S=(ds(v[k]) for k in ('accrual_start','original_issue','stated_maturity','true_maturity','first_coupon_date','last_coupon_start_date','anchor','settlement'))
    f=int(v['frequency']);preserve=v['end_of_month']=='Preserve';step=12//f
    if not D<=I<Fst or not D<M<=T or not I<=S<M:raise ValueError('dated admission')
    q=[A];k=1
    while q[-1]>D:
        q.append(back(A,k,step,preserve));k+=1
        if len(q)>20000:raise ValueError('Q count')
    q=q[::-1]
    if not q[0]<=D<q[1] or not q[-2]<M<=q[-1] or len(q)>int(v['date_cap']):raise ValueError('minimal coverage/cap')
    if Fst==M:
        if L!=D:raise ValueError('single-coupon L identity')
        actual=[M]
    else:
        if Fst not in q or L not in q or not D<Fst<=L<M:raise ValueError('F/L reconstruction')
        actual=q[q.index(Fst):q.index(L)+1]+[M]
    if len(actual)>int(v['date_cap']) or len(actual)>20000:raise ValueError('actual cap')
    if any(a>=b for a,b in zip(actual,actual[1:])) or actual[0]<=D or actual[-1]!=M:raise ValueError('actual order')
    for a,b in zip(actual[:-2],actual[1:-1]):
        if q.index(b)!=q.index(a)+1:raise ValueError('interior adjacency')
    future=[(i,c) for i,c in enumerate(actual) if c>S]
    if not future or len(future)>int(v['max_bundles']):raise ValueError('future cap')
    sel=v['selection'];convention=sel.get('convention') if sel['kind']=='Explicit' else ('MoneyMarket' if len(future)==1 else 'Compound')
    C=F(int(v['coupon_coefficient']),10**int(v['coupon_scale']))
    return {'D':D,'I':I,'M':M,'T':T,'S':S,'q':q,'actual':actual,'future':future,'f':f,'C':C,'convention':convention}

def geometry(q,a,b):
    if a==b:
        j=max(i for i,x in enumerate(q[:-1]) if x<=a)
        return {'start':iso(a),'end':iso(b),'full':'0','fragments':[],
                'left_pair':[iso(q[j]),iso(q[j+1])],'right_pair':[iso(q[j]),iso(q[j+1])]},F(0),F(0)
    assert a<b
    full=0;frags=[];lam=F(0);pay=F(0)
    used=[]
    for j,(l,r) in enumerate(zip(q,q[1:])):
        x=max(l,a);y=min(r,b)
        if x>=y:continue
        d=(y-x).days;s=(r-l).days;used.append(j)
        lam+=F(d,s)
        if x==l and y==r:full+=1
        else:frags.append({'cell_left':iso(l),'cell_right':iso(r),'piece_start':iso(x),'piece_end':iso(y),'days':str(d),'cell_days':str(s)})
    assert len(frags)<=2 and used
    g={'start':iso(a),'end':iso(b),'full':str(full),'fragments':frags,
       'left_pair':[iso(q[used[0]]),iso(q[used[0]+1])],'right_pair':[iso(q[used[-1]]),iso(q[used[-1]+1])]}
    return g,lam,None

def amounts(q,a,b,C,f):
    g,lam,_=geometry(q,a,b);v=C*lam/f;p=C*int(g['full'])/f
    for t in g['fragments']:
        d=int(t['days']);s=int(t['cell_days'])
        assert 0<d<s and f*d!=365
        p+=C*F(d,365) if f*d<365 else C*(F(1,f)-F(s-d,365))
    return g,lam,v,p

def source_state(v):
    r=reconstruct(v);q,actual,D,S,C,f=r['q'],r['actual'],r['D'],r['S'],r['C'],r['f']
    prev=D;coupons=[]
    for i,c in enumerate(actual):
        g,lam,V,P=amounts(q,prev,c,C,f)
        coupons.append({'ordinal':i,'date':c,'geometry':g,'lambda':lam,'valuation':V,'payable':P});prev=c
    start=max([D]+[d for d in actual if d<=S]);ag,al,A,AI=amounts(q,start,S,C,f)
    if f==12:AI=A
    r.update({'coupons':coupons,'accrual_geometry':ag,'A':A,'AI':AI,'a_bits':ieee(A)[0],'ai_bits':ieee(AI)[0]})
    return r

def ratio_json(x):return {'num':str(x.numerator),'den':str(x.denominator)}
def ieee_json(x):
    b,c=ieee(x);return {'bits':hex64(b),'class':c}
def ext_json(t):return [str(x) for x in t]

def risk_views(terms,dirty):
    rs=[retained(ts) for ts in terms];v=[F(n)*pow2(-8192) for n in rs]
    names=('price derivative','price second derivative','time weighted amount','modified duration','macaulay duration','convexity')
    vals=v+[-v[0]/dirty,v[2]/dirty,v[1]/dirty]
    return {'retained_hex':[('-' if n<0 else '')+format(abs(n),'x') for n in rs],
            'views':{k:ieee_json(x) for k,x in zip(names,vals)}}

def dated_result(defn,y,gseed=None,factorseeds=None):
    r=source_state(defn);c='compound' if r['convention']=='Compound' else 'mm'
    if gseed is not None and c!='compound':raise ValueError('seeded compound operation requires Compound resolved convention')
    if gseed is None and c!='mm':raise ValueError('MM operation requires MoneyMarket resolved convention')
    f=r['f'];S=r['S'];T=r['T'];D=(T-S).days
    base={'quasi_dates':[iso(x) for x in r['q']],'actual_dates':[iso(x) for x in r['actual']],
          'future_ordinals':[str(i) for i,_ in r['future']], 'convention':c,
          'valuation_accrued':ratio_json(r['A']),'settlement_accrued':ratio_json(r['AI']),
          'a':ieee_json(r['A']),'ai':ieee_json(r['AI']),'accrual_geometry':r['accrual_geometry'],
          'flows':[]}
    if c=='mm' and 365+y*D<=0:
        base['status']='MoneyMarketDomain';base['domain_days']=str(D);return base
    if c=='compound' and y<=-f:
        base['status']='compound_domain';return base
    if factorseeds is not None and len(factorseeds)!=len(r['future']):raise ValueError('factor seed count != future count')
    terms=[[],[],[]];pvs=[]
    for j,(i,cdate) in enumerate(r['future']):
        cp=r['coupons'][i];pay=T if cdate==r['M'] else cdate;R=(T-pay).days
        _,lam,_=geometry(r['q'],S,cdate)
        B=cp['valuation'] if c=='compound' else cp['payable']
        if cdate==r['M']:B+=100
        row={'original_ordinal':str(i),'contractual_date':iso(cdate),'payment_date':iso(pay),
             'source':ratio_json(B),'coupon_geometry':cp['geometry'],'lambda':ratio_json(lam)}
        if c=='mm':
            den,num,fb,cl=mm(y,D,R);row['affine']={'den':str(den),'num':str(num)}
        else:
            zb,zc=exponent(f,gseed,lam);z=value(zb) if zc!='overflow' else None
            row['exponent']={'bits':hex64(zb),'class':zc}
            if z is not None:
                rad,branch,hit=guard(f,abs(y)<pow2(-50),y,z,lam)
                row['guard']={'branch':branch,'boundary':hit,'radius':ratio_json(rad)}
            # Explicit supplied factor seed is independent of guard/actual exp reachability.
            fb=int(factorseeds[j],16);cl='seeded_positive_finite'
        row['factor']={'bits':hex64(fb),'class':cl}
        Wb,Wcl=ieee(B*value(fb));row['pv']={'bits':hex64(Wb),'class':Wcl};base['flows'].append(row)
        if Wcl in ('underflow','overflow'):
            base['status']='seeded_pv_range' if c=='compound' else 'unexpected_mm_pv_range';return base
        W=value(Wb);pvs.append(W)
        rt=[risk_term(c,k,W,y,f,lam,D,R) for k in range(3)]
        row['terms']=[ext_json(t) for t in rt]
        for k,t in enumerate(rt):terms[k].append(t)
    db,dc=ieee(sum(pvs,F(0)));base['dirty']={'bits':hex64(db),'class':dc}
    if dc in ('underflow','overflow','zero'):
        base['status']='seeded_dirty_range' if c=='compound' else 'unexpected_mm_dirty_range';return base
    dirty=value(db);a=value(r['a_bits']);ai=value(r['ai_bits'])
    base['bases']={b:{'bits':hex64(bb),'class':cc} for b in ('Clean','ModelDirty','SettlementDirty') for bb,cc in [bridge(c,b,dirty,a,ai)]}
    base['risk']=risk_views(terms,dirty)
    base['status']='seeded_complete' if c=='compound' else 'complete'
    return base
