#!/usr/bin/env python3
"""Complete byte/schema/reproduction/reader gate; no Canadian financial truth claim."""
from __future__ import annotations
import argparse,collections,datetime,hashlib,importlib.util,json,math,os,re,subprocess,sys,time
from fractions import Fraction
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[2]
REGISTRY_SHA='f50e2f78e10cf786aaa4267220ca200f6d9fc2eef464d425ef98909e161505fa'
MAX=(1<<127)-1
MODES=('TowardZero','AwayFromZero','TowardPositiveInfinity','TowardNegativeInfinity','NearestTiesEven','NearestTiesAwayFromZero','Exact')
REQUIRED=('schema','fixture-set','domain','description','reference','reference-version','reference-source','tolerance','reproduction','generated-at','platform')

def require(condition,message):
    if not condition:raise ValueError(message)
def sha(raw):return hashlib.sha256(raw).hexdigest()
def fnv(raw):
    h=0xcbf29ce484222325
    for x in raw:h=((h^x)*0x100000001b3)&((1<<64)-1)
    return f'{h:016x}'
def pin(raw):return {'sha256':sha(raw),'fnv64':fnv(raw),'bytes':len(raw)}
def js(v):return json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=True)
def decode(text):
    def pairs(values):
        d={}
        for k,v in values:require(k not in d,'duplicate JSON key');d[k]=v
        return d
    v=json.loads(text,object_pairs_hook=pairs,parse_constant=lambda x:(_ for _ in ()).throw(ValueError('nonfinite JSON')))
    require(js(v)==text,'noncanonical JSON spelling');return v

def integer(s,lo=None,hi=None):
    require(type(s)==str and re.fullmatch(r'0|-?[1-9][0-9]*',s),'noncanonical integer')
    n=int(s);require((lo is None or lo<=n) and (hi is None or n<=hi),'integer range');return n

def date(s):
    require(type(s)==str and re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}',s),'noncanonical Date')
    require(datetime.date.fromisoformat(s).isoformat()==s,'noncanonical Date')
def amount(s,positive=False):
    require(type(s)==str and len(s.split(':'))==2,'amount spelling');a,b=s.split(':');n=integer(a,-(1<<127),MAX);scale=integer(b,0,38)
    require(not (n==0 and scale) and not (scale and n%10==0),'decimal value alias')
    require(not positive or n>0,'positive typed quantum');return n,scale

def bits(s,negative_zero=False):
    require(type(s)==str and re.fullmatch(r'[0-9a-f]{16}',s),'noncanonical IEEE64 spelling')
    n=int(s,16);require((n>>52)&2047!=2047,'nonfinite IEEE input')
    require(negative_zero or n!=1<<63,'negativezero alias');return '0000000000000000' if n==1<<63 else s

def rational(s):
    require(type(s)==str and len(s.split('/'))==2,'rational spelling');n,d=s.split('/');a=integer(n);b=integer(d,1)
    require(math.gcd(abs(a),b)==1,'nonreduced rational');return Fraction(a,b)
def dyadic(s):
    require(type(s)==str and len(s.split('@'))==2,'dyadic spelling');n,e=s.split('@');a=integer(n);b=integer(e)
    require((a==0 and b==0) or (a!=0 and a%2!=0),'noncanonical dyadic mantissa')

def source_input(row,projection,registry):
    allowed=projection[row['op']]
    allinputs={k for fields in projection.values() for k in fields}
    for k in allinputs:require((row[k]!='-') if k in allowed else row[k]=='-','unused/missing source input '+k)
    for k in ('dated','issue','stated_maturity','true_maturity','settlement'):
        if row[k]!='-':date(row[k])
    for k in ('quasi','coupon_dates'):
        if row[k] not in ('-','empty'):
            for s in row[k].split(','):date(s)
    for k in ('date_cap','bundle_cap','coupon_index'):
        if row[k]!='-':integer(row[k],0,(1<<64)-1)
    for k in ('coupon','nominal','quantum','view_quantum'):
        if row[k]!='-':amount(row[k],k in ('quantum','view_quantum'))
    for k,allowedvalues in [('frequency',('2','4','12')),('eom',('Ignore','Preserve')),('source_policy',('CanonicalIiac13LocalV1',)),
        ('selection',('Explicit:Compound','Explicit:MoneyMarket','LastCouponMoneyMarket')),('aggregation',('RoundTotal','RoundComponentsThenSum')),('mode',MODES),('view_mode',MODES)]:
        if row[k]!='-':require(row[k] in allowedvalues,'unknown source enum '+k)
    if row['source_item']!='-':require(row['source_item'] in (('valuation_coupon','payable_coupon') if row['op']=='source_view_coupon' else ('valuation_accrued','settlement_accrued')),'source item')
    if row['evidence']!='-':
        parts=row['evidence'].split(':');kind=parts[0];require(kind in ('d','di','b','bi'),'evidence kind')
        require(len(parts)=={'d':3,'di':5,'b':2,'bi':3}[kind],'ignored evidence operand')
        values=[]
        if kind[0]=='d':
            for start in ((1,3) if kind=='di' else (1,)):
                n,scale=amount(':'.join(parts[start:start+2]));values.append(Fraction(n,10**scale))
        else:
            for s in parts[1:]:
                bits(s);n=int(s,16);sign=-1 if n>>63 else 1;ex=(n>>52)&2047;sig=(n&((1<<52)-1))|((1<<52) if ex else 0);e=ex-1075 if ex else -1074
                values.append(sign*Fraction(sig)*(Fraction(1<<e) if e>=0 else Fraction(1,1<<-e)))
        require(len(values)==1 or values[0]<=values[1],'typed price enclosure order')
    if row['op']=='source_reference':
        require(row['reference'] in registry,'unapproved primary locator')
        require(row['source_observation']==registry[row['reference']]['printed_observation'],'primary observation substitution')
    if row['expected'].startswith('canadian.'):
        for k in SOURCE_OUTPUTS:
            if k not in ('expected','context'):require(row[k]=='-','uncomputed output after failed source stage')
    for k in ('view_amount','lower_amount','upper_amount'):
        if row[k] not in ('-','none'):amount(row[k])
    for k in ('valuation_ai_bits','settlement_ai_bits','quote_accrued_bits'):
        if row[k]!='-':bits(row[k])
    for k in ('valuation_coupons','payable_coupons','bundle_coefficients','bundle_lambdas'):
        if row[k]!='-':
            for term in row[k].split(','):rational(term)
    for k in ('valuation_ai','settlement_ai','source_value'):
        if row[k] not in ('-','none'):rational(row[k])
    return (js({k:row[k] for k in allowed})+'\n').encode()

# Literal root-approved numeric object grammar. Expected outputs never select keys.
NUM_KEYS={
 'log_argument':('frequency','yield_bits'),'log_scale':('frequency','ell_bits'),
 'exponent':('frequency','g_bits','lambda_num','lambda_den'),'tiny_bound':('frequency','yield_bits','lambda_num','lambda_den'),
 'guard_box':('interval_lo_num','interval_hi_num','interval_exp','box_lo_num','box_hi_num','box_exp'),
 'elementary':('function','argument_bits'),'mm_affine':('yield_bits','days_total','days_remaining'),
 'pv':('source_num','source_den','factor_bits'),'source_ieee':('source_num','source_den'),
 'extended':('num','den','binary_exp'),'retained':('terms','dirty_bits'),'dirty_sum':('pv_bits',),
 'residual':('dirty_bits','target_bits'),'dated_mm':('definition','yield_bits'),
 'dated_compound_seeded':('definition','yield_bits','g_bits','factor_bits')}
DEF_KEYS=('accrual_start','original_issue','stated_maturity','true_maturity','first_coupon_date','last_coupon_start_date','anchor','frequency','end_of_month','coupon_coefficient','coupon_scale','source_policy','settlement','selection','date_cap','max_bundles')

def object_keys(v,keys):require(type(v)==dict and set(v)==set(keys),'unknown/missing nested operand')
def numeric_input(op,text,operations,relational_validator):
    require(op in operations,'unknown numeric operation');v=decode(text)
    if op=='guard':
        z=bits(v.get('z_bits'));zero=int(z,16)==0
        if zero:keys=('z_bits',)
        else:
            y=bits(v.get('yield_bits'));n=int(y,16);ex=(n>>52)&2047;sig=(n&((1<<52)-1))|((1<<52) if ex else 0);power=ex-1075 if ex else -1074
            tiny=sig==0 or sig.bit_length()-1+power<-50
            keys=('frequency','yield_bits','z_bits','lambda_num','lambda_den') if tiny else ('frequency','yield_bits','z_bits')
    elif op=='risk_terms':keys=('convention','pv_bits','yield_bits','frequency','lambda_num','lambda_den') if v.get('convention')=='compound' else ('convention','pv_bits','yield_bits','days_total','days_remaining')
    elif op=='price_bases':keys=('convention','dirty_bits','a_bits','ai_bits') if v.get('convention')=='compound' else ('convention','dirty_bits','ai_bits')
    elif op=='target':
        keys=['convention','basis','value_bits'];b=v.get('basis');c=v.get('convention')
        if b=='Clean':keys+=['quote_accrued_bits']
        elif b=='SettlementDirty' and c=='compound':keys+=['a_bits','ai_bits']
    else:keys=NUM_KEYS[op]
    object_keys(v,keys)
    if 'definition' in v:
        d=v['definition'];object_keys(d,DEF_KEYS);sel=d['selection']
        require(type(sel)==dict,'typed nested selection')
        object_keys(sel,('kind','convention') if sel.get('kind')=='Explicit' else ('kind',))
    def walk(x,key=''):
        if type(x)==dict:
            for k,a in x.items():walk(a,k)
        elif type(x)==list:
            for a in x:walk(a,key)
        else:
            require(type(x)==str,'numeric operands must be typed strings')
            if key.endswith('_bits'):bits(x)
    walk(v)
    # Frozen root-reviewed producer schema adds bounded integer/ratio/domain/dated
    # coupling validation; the independent object-key authority above is literal.
    if op in ('pv','source_ieee'):
        require(abs(int(v['source_num'])).bit_length()<=160 and int(v['source_den']).bit_length()<=148,'reviewed source pair160/148bits')
    if 'lambda_num' in v:
        n=int(v['lambda_num']);d=int(v['lambda_den']);require(0<d<=33856 and 0<n<20000*d and 184*n>=d,'reviewed coordinate30/16bit form')
    if op=='extended':require(max(abs(int(v['num'])).bit_length(),int(v['den']).bit_length())<=7168,'physical112word extended ratio')
    if op=='guard_box':require(max(abs(int(v[k])).bit_length() for k in ('interval_lo_num','interval_hi_num','box_lo_num','box_hi_num'))<=1280,'physical20word guard operands')
    relational_validator(op,v)
    return (op+'\t'+js(v)+'\n').encode()

def ideal_input(row,fields,kind):
    normalized=dict(row)
    for k in ('accrual_start','issue','stated_maturity','true_maturity','settlement'):date(row[k])
    for k in ('quasi_dates','coupon_dates'):
        for s in row[k].split(';'):date(s)
    c=integer(row['coupon_coefficient'],0,MAX);s=integer(row['coupon_scale'],0,38)
    require(not(s and c%10==0),'ideal decimal alias')
    require(row['frequency'] in ('2','4','12') and row['eom'] in ('Ignore','Preserve'),'ideal frequency/EOM')
    require(row['source_policy']=='CanonicalIiac13LocalV1','ideal policy')
    require(row['selection'] in ('Explicit(Compound)','Explicit(MoneyMarket)','LastCouponMoneyMarket'),'ideal selection')
    integer(row['date_cap'],2,20000);integer(row['bundle_cap'],0,20000)
    for k in fields:
        if k.endswith('_bits'):normalized[k]=bits(row[k],True)
    require(row['operation']==('evaluate_and_six_risk_getters' if kind=='price' else 'yield_from_price'),'ideal operation')
    if kind=='inverse':
        integer(row['evaluation_budget'],1);require(row['target_basis'] in ('Clean','ModelDirty','SettlementDirty'),'target basis')
    for k,v in row.items():
        if k in fields or v=='-':continue
        if k in ('lambdas','source_coefficients'):
            for term in v.split(';'):rational(term)
        elif '@' in v:dyadic(v)
        elif re.fullmatch(r'-?[0-9]+/[0-9]+',v):rational(v)
    return ('\t'.join(normalized[k] for k in fields)+'\n').encode()

SOURCE_OUTPUTS=()
def parse_corpus(raw,spec,kind,source_registry,relational_validator,operations):
    require(raw.endswith(b'\n') and b'\r' not in raw,'canonical LF/finalLF required')
    head,body=raw.split(b'\n\n',1);pairs=[]
    for line in head.decode('ascii').splitlines():
        require(': ' in line,'malformed header');k,v=line.split(': ',1)
        require(re.fullmatch(r'[a-z][a-z0-9-]*',k) and v and v.strip()==v,'header grammar');pairs.append((k,v))
    require(len(pairs)==len(dict(pairs)),'duplicate header');h=dict(pairs)
    require(pairs[0]==('schema','1') and all(k in h for k in REQUIRED),'schema1 required headers')
    require(set(h)==set(spec['header_keys']),'unknown/missing header/schema')
    require(h['fixture-set']==spec['fixture_set'],'unknown corpus kind')
    lines=body.decode('ascii').splitlines();cols=lines[0].split('\t')
    require(cols==spec['columns'],'unknown columns');require(int(h['column-count'])==len(cols),'column census')
    profiles=collections.Counter();ops=collections.Counter();seen=set();inputs=[];records=[];definitions=set();full_definitions=set();schedule_definitions=set();expectations=collections.Counter();scopes=collections.Counter()
    for line in lines[1:]:
        fields=line.split('\t');require(len(fields)==len(cols),'row width');row=dict(zip(cols,fields));records.append(row)
        if kind=='source':
            require(row['op'] in spec['inputs'],'unknown source operation');identity=source_input(row,spec['inputs'],source_registry);ops[row['op']]+=1
            definition={k:row[k] for k in ('dated','issue','stated_maturity','true_maturity','frequency','eom','quasi','coupon_dates','coupon','source_policy') if row[k]!='-'}
            digest=sha(js(definition).encode()) if definition else '-';require(row['definition_sha256']==digest,'source definition digest')
            if digest!='-':definitions.add(digest)
            if row['coupon']!='-':full_definitions.add(digest)
            if row['op']=='schedule':schedule_definitions.add(digest)
            require(row['scope']==('source_reference' if row['op']=='source_reference' else 'public'),'source operation scope')
            expectations[row['expected']]+=1;scopes[row['scope']]+=1
        elif kind=='numeric':
            require(row['profile'] in ('exact_seed','native_2u','dated_exact','dated_seeded'),'numeric profile')
            identity=numeric_input(row['operation'],row['input'],operations,relational_validator);expected=decode(row['expected']);relational_validator.expected(row['operation'],expected);ops[row['operation']]+=1
        else:identity=ideal_input(row,spec['inputs'],kind)
        require(identity not in seen,'duplicate semantic input (answer/id/profile excluded)');seen.add(identity);inputs.append(identity);profiles[row['profile']]+=1
    require(len(records)==int(h['record-count'])==spec['record_count'],'record count drift')
    require(dict(profiles)==json.loads(h['profile-counts'])==spec['frozen_profile_counts'],'profile drift')
    payload=body.split(b'\n',1)[1];ib=b''.join(inputs)
    if kind=='source':
        require(h['operation-input-fields']==js(spec['inputs']),'unapproved source projection')
        for suffix,computed in [('records',body),('canonical-inputs',ib)]:
            require(h[suffix+'-sha256']==sha(computed) and h[suffix+'-fnv64']==fnv(computed),'source '+suffix+' pin')
        require(json.loads(h['operation-counts'])==dict(ops),'operation census')
        require(int(h['unique-serialized-inputs'])==len(seen),'unique count')
        require(set(ops)==set(spec['inputs']),'source14operation census')
        require(json.loads(h['expectation-counts'])==dict(expectations) and json.loads(h['scope-counts'])==dict(scopes),'source outcome/scope census')
        require(int(h['unique-financial-definitions'])==len(full_definitions) and int(h['unique-schedule-definitions'])==len(schedule_definitions) and int(h['unique-supplied-definition-digests'])==len(definitions),'source supplied-definition census')
    else:
        require(h['input-columns']==','.join(spec['inputs']),'unapproved declared input projection')
        require(h['body-sha256']==sha(body) and h['body-fnv64']==fnv(body),'body pin')
        require(h['records-sha256']==sha(payload) and h['records-fnv64']==fnv(payload),'record payload pin')
        prefix='input' if kind=='numeric' else 'inputs'
        require(h[prefix+'-sha256']==sha(ib) and h[prefix+'-fnv64']==fnv(ib),'input pin')
        require(int(h['unique-input-count'])==len(seen),'unique count')
        if kind=='numeric':require(json.loads(h['operation-counts'])==dict(ops) and len(ops)==19,'numeric operation census')
    return h,{'records':len(records),'columns':len(cols),'unique_actual_inputs':len(seen),'profiles':dict(profiles),'operations':dict(ops),'input_sha256':sha(ib),'input_fnv64':fnv(ib),'input_bytes':len(ib)}


def verify_reference_inventory(root):
    # Python can read a pre-existing .pyc even with dont_write_bytecode=True.
    # The fixed reference directory contains exactly these five source files;
    # reject bytecode, alternate extension modules, extra files and directories.
    folder=root/'validation/generator/canadian_numeric_reference'
    names={'numeric_schema.py','numeric_math.py','numeric_cases.py','dated_reference.py','numeric_outputs.py'}
    require(folder.is_dir() and not folder.is_symlink(),'numeric reference directory alias')
    require({p.name for p in folder.iterdir()}==names,'unconsumed numeric reference helper/bytecode input')
    require(all((folder/name).is_file() and not (folder/name).is_symlink() for name in names),'numeric reference helper alias')


def guard(root,initial):
    verify_reference_inventory(root)
    for rel,expected in initial.items():
        p=root/rel;require(p.is_file() and not p.is_symlink() and sha(p.read_bytes())==expected,'consumed input changed during run: '+rel)

def load_numeric_validator(root):
    folder=root/'validation/generator/canadian_numeric_reference';sys.path.insert(0,str(folder))
    for name in ('numeric_schema','numeric_math','dated_reference','numeric_outputs'):sys.modules.pop(name,None)
    import numeric_schema,numeric_outputs
    numeric_schema.validate.expected=numeric_outputs.validate_expected
    return numeric_schema.validate

def verify_file(root,rel,expected):
    p=root/rel;require(p.is_file() and not p.is_symlink(),'missing/symlink consumed input '+rel)
    raw=p.read_bytes();require(pin(raw)==expected,'frozen consumed artifact/source pin mismatch '+rel)
    return raw


def structural(root,registry,progress=None):
    initial={}
    verify_reference_inventory(root)
    expected_hist=set(registry['historical_fixtures']);new={x['path'] for x in registry['fixtures'].values()}
    actual={str(p.relative_to(root)) for p in (root/'validation/fixtures').rglob('*.fixture')}
    require(actual==expected_hist|new and len(expected_hist)==77 and len(new)==4,'complete corpus inventory77historical+4new required')
    for i,(rel,expected) in enumerate(registry['files'].items()):
        raw=verify_file(root,rel,expected);initial[rel]=sha(raw)
        if progress:progress('pin '+str(i+1)+'/'+str(len(registry['files'])))
    cb=(root/'docs/M8_CANADIAN_CONTRACT.md').read_bytes();require(sha(cb)==registry['contract_sha256'],'wrong contract')
    matches=re.findall(rb'\| (M8_CANADIAN_[A-Z_]+\.md) \|[^\n]*\| ([0-9a-f]{64}) \|',cb)
    fixed={'docs/'+a.decode():b.decode() for a,b in matches};require(len(matches)==14 and fixed==registry['component_shas'],'fixed14component contract binding')
    ap=json.loads((root/'validation/generator/canadian_source_cash_adopted_pins.json').read_bytes())
    require({'docs/'+k:v for k,v in ap['components'].items()}==fixed,'source adoption14pins mismatch')
    rows=lambda data:b'\n'.join(line for line in data.splitlines() if line.startswith(b'|FR-CAD-'))+b'\n'
    req=rows((root/'docs/REQUIREMENTS.md').read_bytes());require(req==rows(cb) and len(req.splitlines())==72 and sha(req)==registry['requirements_rows_sha256'],'72requirements actual row mismatch')
    sr=json.loads((root/'validation/generator/canadian_source_cash_sources.json').read_bytes());source_registry={r['reference']:r for r in sr['bindings']}
    require(len(source_registry)==sr['binding_count']==36,'fixed source reference count')
    global SOURCE_OUTPUTS
    ss=(root/'validation/generator/canadian_source_cash_schema.json').read_bytes();require(sha(ss)==registry['source_schema_sha256'],'approved source schema')
    SOURCE_OUTPUTS=tuple(json.loads(ss)['output_columns'])
    validator=load_numeric_validator(root);parsed={};headers={}
    for kind,spec in registry['fixtures'].items():
        headers[kind],parsed[kind]=parse_corpus((root/spec['path']).read_bytes(),spec,kind,source_registry,validator,registry['numeric_operations'])
        if progress:progress('parsed '+kind)
    # Bind every source/header pin to ACTUAL consumed bytes, not to self-assertions.
    component_pins={Path(k).name:pin((root/k).read_bytes()) for k in fixed}
    source_map={'generator':'validation/generator/canadian_source_cash_oracles.py','schema':'validation/generator/canadian_source_cash_schema.json',
      'source-bindings':'validation/generator/canadian_source_cash_sources.json','adoption-pins':'validation/generator/canadian_source_cash_adopted_pins.json',
      'contract':'docs/M8_CANADIAN_CONTRACT.md','requirements':'docs/REQUIREMENTS.md','shared-reader':'tests/support/fixture.rs',
      'appendix-certificate':'validation/fixtures/bonds/canadian-appendix-declared-price.json'}
    source_map.update({'component-'+Path(k).name:k for k in fixed})
    sh=headers['source'];require(sh['adopted-contract-sha256']==sha(cb) and sh['requirements-rows-sha256']==sha(req) and sh['requirements-rows-fnv64']==fnv(req),'source contract/requirements headers')
    require(sh['schema-sha256']==registry['source_schema_sha256'] and sh['source-bindings-sha256']==registry['source_registry_sha256'],'source approved schema/registry headers')
    require(json.loads(headers['source']['inputs'])=={k:pin((root/p).read_bytes()) for k,p in source_map.items()},'source actual22input header binding')
    nh=headers['numeric'];ns=json.loads(nh['generator-input-sources'])
    source_names=['validation/generator/canadian_numeric_oracles.py']+['validation/generator/canadian_numeric_reference/'+n for n in ('numeric_math.py','numeric_schema.py','numeric_cases.py','dated_reference.py','numeric_outputs.py')]
    require([e['name'] for e in ns]==source_names,'numeric fixed6source identities')
    ng=pin((root/source_names[0]).read_bytes());require(nh['generator-sha256']==ng['sha256'] and nh['generator-fnv64']==ng['fnv64'],'numeric actual wrapper headers')
    require(nh['requirements-rows']=='72','numeric72requirement census')
    for e in ns:
        p=pin((root/e['name']).read_bytes());require(e['sha256']==p['sha256'] and e['fnv64']==p['fnv64'],'numeric helper actual pin')
    require(nh['contract-sha256']==sha(cb) and nh['contract-fnv64']==fnv(cb),'numeric contract header')
    require(nh['requirements-sha256']==sha(req) and nh['requirements-fnv64']==fnv(req),'numeric requirements header')
    require(nh['reader-sha256']==sha((root/'tests/support/fixture.rs').read_bytes()),'numeric actual reader pin')
    require(json.loads(nh['incorporated-components'])==[{'source':k,'sha256':component_pins[Path(k).name]['sha256'],'fnv64':component_pins[Path(k).name]['fnv64']} for k in sorted(fixed)],'numeric14actual component headers')
    cp=pin((root/'validation/fixtures/bonds/canadian-ideal-certificates.json').read_bytes());gp=pin((root/'validation/generator/canadian_ideal_oracles.py').read_bytes())
    for kind in ('price','inverse'):
        h=headers[kind];require(h['generator-sha256']==gp['sha256'] and h['generator-fnv64']==gp['fnv64'],'ideal actual generator')
        require(h['adopted-contract-sha256']==sha(cb) and h['adopted-contract-fnv64']==fnv(cb),'ideal contract header')
        require(json.loads(h['component-pins'])==component_pins,'ideal14actual component headers')
        require(h['certificates-sha256']==cp['sha256'] and h['certificates-fnv64']==cp['fnv64'],'ideal sidecar actual pin')
        require(h['requirements-rows-sha256']==sha(req) and h['requirements-rows-fnv64']==fnv(req),'ideal requirements rows')
    cert=json.loads((root/'validation/fixtures/bonds/canadian-ideal-certificates.json').read_bytes())
    require(cert['generator']==gp and cert['sources']=={'contract':pin(cb),'components':component_pins,'requirements_rows':pin(req)},'ideal sidecar actual source binding')
    for kind,label in [('price','price'),('inverse','inverse')]:
        summary=parsed[kind];require(cert['inputs'][label]=={'sha256':summary['input_sha256'],'fnv64':summary['input_fnv64'],'bytes':summary['input_bytes']},'acyclic sidecar actual input binding')
    cs=cert['schema_approval']
    require(cs['price_columns']==registry['fixtures']['price']['columns'] and cs['price_input_columns']==registry['fixtures']['price']['inputs'] and cs['inverse_columns']==registry['fixtures']['inverse']['columns'] and cs['inverse_input_columns']==registry['fixtures']['inverse']['inputs'],'sidecar literal approved schema')
    anc=json.loads((root/'validation/fixtures/bonds/canadian-appendix-declared-price.json').read_bytes())
    require(anc['contract_sha256']==sha(cb) and anc['generator_sha256']==sha((root/'validation/generator/canadian_appendix_certificate.py').read_bytes()),'Appendix actual producer/source pins')
    require(anc['input_sha256']==sha((js(anc['inputs'])+'\n').encode()),'Appendix actual input LF binding')
    report_raw=(root/'validation/reports/canadian-source-cash-oracle.json').read_bytes();require(b'\r' not in report_raw and report_raw.endswith(b'\n'),'canonical report bytes')
    report=json.loads(report_raw);require(report['input_pins']==json.loads(sh['inputs']) and report['fixture']==pin((root/registry['fixtures']['source']['path']).read_bytes()),'source report actual fixture/input binding')
    require(report['generation']=={k:sh[k] for k in ('generated-at','platform','python')},'source preserved generation metadata')
    guard(root,initial)
    return initial,parsed


def run_logged(command,root,out,label,initial,progress=None):
    if progress:progress('running '+label)
    (out/(label+'.running')).write_text('actual subprocess beginning\n')
    started=datetime.datetime.now(datetime.timezone.utc).isoformat();logpath=out/(label+'.log')
    with logpath.open('xb') as log:
        result=subprocess.run(command,cwd=root,stdout=log,stderr=subprocess.STDOUT,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1','CARGO_MANIFEST_DIR':str(root)})
    entry={'label':label,'command':command,'exit_status':result.returncode,'started_utc':started,'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'log_sha256':sha(logpath.read_bytes())}
    (out/(label+'.status.json')).write_text(json.dumps(entry,sort_keys=True,indent=2)+'\n')
    require(result.returncode==0,'actual subprocess failed '+label+' exit '+str(result.returncode))
    guard(root,initial)
    return entry


def verify_reader_output(text,root,registry):
    """Bind every actual reader row to the exact requested file and frozen census."""
    fixtures=registry['historical_fixtures']+[s['path'] for s in registry['fixtures'].values()]
    lines=text.splitlines()
    require(len(lines)==82,'actual reader must emit81file rows and1total')
    counts={}
    for expected,line in zip(fixtures,lines[:-1]):
        cells=line.split('\t');require(len(cells)==2,'reader row format')
        require(cells[0]==str(root/expected),'actual reader path/order mismatch')
        require(expected not in counts,'actual reader duplicate path')
        counts[expected]=integer(cells[1],0)
    for spec in registry['fixtures'].values():
        require(counts[spec['path']]==spec['record_count'],'actual reader new corpus census mismatch '+spec['path'])
    historical=sum(counts[p] for p in registry['historical_fixtures'])
    require(historical==3075381,'actual reader historical77row census')
    total=sum(counts.values())
    require(len(counts)==81 and total==3159951,'actual reader complete81row census')
    require(lines[-1]=='TOTAL\t81\t3159951','actual reader exact total mismatch')
    return {'files':81,'rows':total,'historical_files':77,'historical_rows':historical,
            'new_files':4,'new_rows':total-historical,'per_file_rows':counts}


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--repo',type=Path,default=ROOT);ap.add_argument('--output-dir',type=Path,required=True);ap.add_argument('--rustc',default='rustc')
    args=ap.parse_args();root=args.repo.resolve();out=args.output_dir.resolve();rp=Path(__file__).resolve().with_name('canadian_preproduction_approved.json')
    registry_raw=rp.read_bytes();require(sha(registry_raw)==REGISTRY_SHA,'unknown approved registry');registry=json.loads(registry_raw)
    require(all(out!=root/r and out not in (root/r).parents for r in registry['files']),'output aliases/contains consumed source')
    require(not out.exists() and not out.is_symlink(),'exclusive evidence output already exists/alias')
    out.mkdir(parents=True);self_path=Path(__file__).resolve();self_raw=self_path.read_bytes();commands=[]
    def progress(step):
        (out/'progress.json').write_text(json.dumps({'step':step,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat()})+'\n');print(step,flush=True)
    try:
        initial,parsed=structural(root,registry,progress)
        commandspec=[('source',[sys.executable,'-I',str(root/'validation/generator/canadian_source_cash_oracles.py'),'--repo',str(root),'--output',str(root/'validation/fixtures/bonds/canadian-source-cash-oracle.fixture'),'--report',str(root/'validation/reports/canadian-source-cash-oracle.json'),'--check']),
          ('numeric',[sys.executable,'-I',str(root/'validation/generator/canadian_numeric_oracles.py'),'--check']),
          ('ideal',[sys.executable,'-I',str(root/'validation/generator/canadian_ideal_oracles.py'),'--check']),
          ('appendix',[sys.executable,'-I',str(root/'validation/generator/canadian_appendix_certificate.py'),'--check'])]
        for label,command in commandspec:commands.append(run_logged(command,root,out,label,initial,progress))
        driver=out/'reader-driver.rs'
        driver.write_text('#[path='+json.dumps(str(root/'tests/support/fixture.rs'))+']\nmod fixture;\nfn main(){let mut files=0;let mut rows=0;for p in std::env::args().skip(1){let f=fixture::Fixture::load(&p).expect("actual shared reader");let n=f.records().count();println!("{}\\t{}",p,n);files+=1;rows+=n;}println!("TOTAL\\t{}\\t{}",files,rows);}\n')
        commands.append(run_logged([args.rustc,'--edition=2024',str(driver),'-o',str(out/'reader-driver')],root,out,'reader-compile',initial,progress))
        fixtures=registry['historical_fixtures']+[s['path'] for s in registry['fixtures'].values()]
        commands.append(run_logged([str(out/'reader-driver')]+[str(root/p) for p in fixtures],root,out,'reader-run',initial,progress))
        reader=verify_reader_output((out/'reader-run.log').read_text(),root,registry)
        guard(root,initial);require(rp.read_bytes()==registry_raw and self_path.read_bytes()==self_raw,'attestor/registry mutated during run')
        evidence={'status':'PASS_COMPLETE_FOUR_CORPUS_ARTIFACT_GATE','actual_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'tool_sha256':sha(self_raw),'tool_pin':pin(self_raw),'approved_registry_sha256':REGISTRY_SHA,'approved_registry_pin':pin(registry_raw),
         'corpora':parsed,'actual_commands':commands,'byte_guarded_input_count':len(initial)+2,'historical_fixtures':77,'new_corpora':4,'actual_reader':reader,
         'generation_metadata_preserved_by_actual_check':True,'scope':'structural integrity, exact reproducibility and actual unchanged reader acceptance; not financial truth or Canadian runtime/foreign native/allocation/release acceptance'}
        with (out/'evidence.json').open('x') as f:f.write(json.dumps(evidence,sort_keys=True,indent=2)+'\n')
        progress('COMPLETE PASS')
    except BaseException as error:
        failure={'status':'FAIL_INCOMPLETE_GATE','exception':type(error).__name__,'diagnostic':str(error),'actual_commands_completed':commands,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
        with (out/'failure.json').open('x') as f:f.write(json.dumps(failure,sort_keys=True,indent=2)+'\n')
        raise

if __name__=='__main__':main()
