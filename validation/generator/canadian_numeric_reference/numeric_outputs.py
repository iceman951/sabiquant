"""Closed typed expected grammar for numeric corpus. Answers are never input identity."""
from numeric_schema import obj,integer,enum
from datetime import date
import re

def ratio(v,nonnegative=False):
    from fractions import Fraction
    obj(v,('num','den'));n=integer(v['num']);d=integer(v['den'],1);f=Fraction(n,d)
    if f.numerator!=n or f.denominator!=d or (nonnegative and n<0):raise ValueError('noncanonical expected ratio')

def ieee(v):
    obj(v,('bits','class'));b=v['bits'];cl=v['class']
    if type(b)!=str or not re.fullmatch('[0-9a-f]{16}',b):raise ValueError('output IEEE spelling')
    b=int(b,16);e=(b>>52)&2047;t=b&((1<<52)-1)
    enum(cl,('zero','subnormal','normal','underflow','overflow'))
    if cl=='zero' and b!=0:raise ValueError('zero output')
    if cl=='underflow' and b&((1<<63)-1)!=0:raise ValueError('underflow output')
    if cl=='overflow' and (e!=2047 or t!=0):raise ValueError('overflow output')
    if cl=='normal' and not 1<=e<=2046:raise ValueError('normal output')
    if cl=='subnormal' and not(e==0 and t):raise ValueError('subnormal output')

def boolean(v):
    if type(v)!=bool:raise ValueError('output bool')

def ext(v):
    if type(v)!=list or len(v)!=3:raise ValueError('extended output tuple')
    s=integer(v[0],-1,1);q=integer(v[1],0,(1<<53)-1);e=integer(v[2],-16384,16384)
    if (s==0 and (q or e)) or (s!=0 and q<1<<52):raise ValueError('extended output canonical')

def terms(v):
    if type(v)!=list or len(v)!=3:raise ValueError('three output terms')
    for t in v:ext(t)

def datestr(v):
    if type(v)!=str or not re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}',v):raise ValueError('output date spelling')
    date.fromisoformat(v)

def pair(v):
    if type(v)!=list or len(v)!=2:raise ValueError('Q pair output')
    for x in v:datestr(x)
    if v[0]>=v[1]:raise ValueError('Q pair order')

def geometry(v):
    obj(v,('start','end','full','fragments','left_pair','right_pair'));datestr(v['start']);datestr(v['end']);integer(v['full'],0,19999)
    pair(v['left_pair']);pair(v['right_pair'])
    fs=v['fragments']
    if type(fs)!=list or len(fs)>2:raise ValueError('edge fragment count')
    for t in fs:
        obj(t,('cell_left','cell_right','piece_start','piece_end','days','cell_days'))
        for key in ('cell_left','cell_right','piece_start','piece_end'):datestr(t[key])
        s=integer(t['cell_days'],28,184);integer(t['days'],1,s-1)
    if v['start']==v['end'] and (v['full']!='0' or fs or v['left_pair']!=v['right_pair']):raise ValueError('empty geometry')

def risk(v):
    obj(v,('retained_hex','views'));rs=v['retained_hex']
    if type(rs)!=list or len(rs)!=3:raise ValueError('three retained outputs')
    for r in rs:
        if type(r)!=str or not re.fullmatch(r'0|-?[1-9a-f][0-9a-f]*',r):raise ValueError('retained signed hex')
    names=('price derivative','price second derivative','time weighted amount','modified duration','macaulay duration','convexity')
    obj(v['views'],names)
    for k in names:ieee(v['views'][k])

def bases(v):
    obj(v,('Clean','ModelDirty','SettlementDirty'))
    for k in v:ieee(v[k])

def guard(v,flow=False):
    obj(v,('branch','boundary','radius') if flow else ('branch','boundary','radius','next_exp_calls'))
    enum(v['branch'],('zero_identity','tiny_bypass','far_bypass','wide_guard'));boolean(v['boundary']);ratio(v['radius'],True)
    if not flow:enum(v['next_exp_calls'],('0','1'))

def validate_expected(op,e):
    if op in ('log_scale','exponent','pv','source_ieee','dirty_sum'):ieee(e)
    elif op=='log_argument':
        obj(e,('branch','argument_or_identity_bits'));enum(e['branch'],('domain','zero','tiny','ln','ln1p'))
        if e['branch']=='domain':
            if e['argument_or_identity_bits'] is not None:raise ValueError('domain output tail')
        else:
            b=e['argument_or_identity_bits']
            if type(b)!=str or not re.fullmatch('[0-9a-f]{16}',b):raise ValueError('argument output bits')
    elif op=='tiny_bound':
        obj(e,('exponent','ideal_linear_exponent','analytic_radius','identity_still_has_error_bound'));ieee(e['exponent']);ratio(e['ideal_linear_exponent']);ratio(e['analytic_radius'],True);boolean(e['identity_still_has_error_bound'])
    elif op=='guard':guard(e)
    elif op=='guard_box':obj(e,('closed_intersection',));boolean(e['closed_intersection'])
    elif op=='elementary':
        obj(e,('ideal','allowed','hypothesis','zero_inf_nan_classification'))
        for key in ('ideal','allowed'):
            t=e[key]
            if type(t)!=list or len(t)!=3:raise ValueError('dyadic output interval')
            lo=integer(t[0]);hi=integer(t[1]);integer(t[2],-20000,20000)
            if lo>hi:raise ValueError('interval order')
        enum(e['hypothesis'],('observed_positive_finite_exp_2u_plus_H','native_log_relative_2u'))
        enum(e['zero_inf_nan_classification'],('explicit_native_validation_required','not_inferred_from_ideal_bits'))
    elif op=='mm_affine':
        obj(e,('den_raw','num_raw','factor','domain'));integer(e['den_raw']);integer(e['num_raw']);boolean(e['domain'])
        if e['domain']:ieee(e['factor'])
        elif e['factor'] is not None:raise ValueError('MM failure tail')
    elif op=='extended':obj(e,('extended',));ext(e['extended'])
    elif op=='risk_terms':obj(e,('terms',));terms(e['terms'])
    elif op=='retained':risk(e)
    elif op=='price_bases':bases(e)
    elif op=='target':
        obj(e,('resolved','positive','zero_evaluation_rejection'));ieee(e['resolved']);boolean(e['positive']);boolean(e['zero_evaluation_rejection'])
    elif op=='residual':
        obj(e,('scale','callback','amount_residual'));enum(e['scale'],('0','1'));ieee(e['callback']);ieee(e['amount_residual'])
    elif op in ('dated_mm','dated_compound_seeded'):
        common={'quasi_dates','actual_dates','future_ordinals','convention','valuation_accrued','settlement_accrued','a','ai','accrual_geometry','flows','status'}
        st=e.get('status')
        if st=='MoneyMarketDomain':common.add('domain_days')
        elif st in ('complete','seeded_complete'):common.update(('dirty','bases','risk'))
        elif st in ('seeded_dirty_range','unexpected_mm_dirty_range'):common.add('dirty')
        elif st not in ('compound_domain','seeded_pv_range','unexpected_mm_pv_range'):raise ValueError('dated status')
        obj(e,common);enum(e['convention'],('compound','mm'))
        for key in ('quasi_dates','actual_dates'):
            ts=e[key]
            if type(ts)!=list or not 1<=len(ts)<=20000:raise ValueError('dated count output')
            for x in ts:datestr(x)
            if any(a>=b for a,b in zip(ts,ts[1:])):raise ValueError('date order output')
        if type(e['future_ordinals'])!=list:raise ValueError('future ordinal output')
        for x in e['future_ordinals']:integer(x,0,19999)
        for key in ('valuation_accrued','settlement_accrued'):ratio(e[key],True)
        ieee(e['a']);ieee(e['ai']);geometry(e['accrual_geometry'])
        if 'domain_days' in e:integer(e['domain_days'],1,3652058)
        fs=e['flows']
        if type(fs)!=list:raise ValueError('flows output')
        for i,row in enumerate(fs):
            keys={'original_ordinal','contractual_date','payment_date','source','coupon_geometry','lambda','factor','pv'}
            if e['convention']=='mm':keys.add('affine')
            else:keys.add('exponent');keys.add('guard')
            if row['pv']['class'] not in ('underflow','overflow'):keys.add('terms')
            obj(row,keys);integer(row['original_ordinal'],0,19999);datestr(row['contractual_date']);datestr(row['payment_date'])
            ratio(row['source'],True);ratio(row['lambda'],True);geometry(row['coupon_geometry']);ieee(row['pv'])
            obj(row['factor'],('bits','class'))
            if e['convention']=='mm':
                obj(row['affine'],('den','num'));integer(row['affine']['den'],1);integer(row['affine']['num'],1);ieee(row['factor'])
            else:
                enum(row['factor']['class'],('seeded_positive_finite',))
                from numeric_schema import f64
                f64(row['factor']['bits'],True);ieee(row['exponent']);guard(row['guard'],True)
            if 'terms' in row:terms(row['terms'])
        if 'dirty' in e:ieee(e['dirty'])
        if 'bases' in e:bases(e['bases'])
        if 'risk' in e:risk(e['risk'])
    else:raise ValueError('unknown output operation')
