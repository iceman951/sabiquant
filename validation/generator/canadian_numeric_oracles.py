#!/usr/bin/env python3
"""Independent Canadian prescribed numeric oracle; exclusive writes and exact --check.
No production/QuantLib implementation imports. Reference helpers use exact Python
integers/Fraction, bounded rational elementary certificates and explicit seed profiles.
"""
import sys
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent/'canadian_numeric_reference'))
import argparse,json,hashlib,platform,datetime,collections,os
from pathlib import Path
from numeric_cases import cases,evaluate
from numeric_schema import COLUMNS,PROFILES,encode,decode,validate
from numeric_outputs import validate_expected
ROOT=Path(__file__).resolve().parent
REPO=Path(__file__).resolve().parents[2]
SOURCE_NAMES=('validation/generator/canadian_numeric_oracles.py',)+tuple('validation/generator/canadian_numeric_reference/'+name for name in ('numeric_math.py','numeric_schema.py','numeric_cases.py','dated_reference.py','numeric_outputs.py'))

def sha(b):return hashlib.sha256(b).hexdigest()
def fnv(b):
    n=0xcbf29ce484222325
    for x in b:n=((n^x)*0x100000001b3)&((1<<64)-1)
    return f'{n:016x}'
def loadpins():
    contract_path=REPO/'docs/M8_CANADIAN_CONTRACT.md'
    contract_bytes=contract_path.read_bytes()
    if sha(contract_bytes)!='4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3':raise ValueError('adopted contract mismatch')
    import re
    components=re.findall(rb'\| (M8_CANADIAN_[A-Z_]+\.md) \|[^\n]*\| ([0-9a-f]{64}) \|',contract_bytes)
    if len(components)!=14:raise ValueError('component set mismatch')
    sources=[('docs/M8_CANADIAN_CONTRACT.md',sha(contract_bytes))]+[('docs/'+n.decode(),h.decode()) for n,h in components]+[('tests/support/fixture.rs','ba6bacbb76bd2c6ed40ffe45308dd43bca80445aed841526cd92e3b9518f216e')]
    manifest=[];used={}
    for source,expected in sources:
        path=REPO/source;b=path.read_bytes()
        if sha(b)!=expected:raise ValueError('consumed input mismatch '+source)
        manifest.append({'source':source,'sha256':expected,'bytes':len(b)})
        used[str(path)]=b
    req=('\n'.join(x for x in contract_bytes.decode().splitlines() if x.startswith('|FR-CAD-'))+'\n').encode()
    if len(req.splitlines())!=72 or sha(req)!='a31bb24356af3b8ed2f6454cd6512bdc052f19705e68211e5a300eafd96e37ae':raise ValueError('requirements row mismatch')
    central=REPO/'docs/REQUIREMENTS.md';rb=central.read_bytes()
    extracted=b'\n'.join(x for x in rb.splitlines() if x.startswith(b'|FR-CAD-'))+b'\n'
    if extracted!=req:raise ValueError('central requirements differ')
    used[str(central)]=rb
    for name in SOURCE_NAMES:used[str(REPO/name)]=(REPO/name).read_bytes()
    return manifest,req,used

def rows_and_census():
    rows=[];identities=[];seen=set();ops=collections.Counter();profiles=collections.Counter();scopes=collections.Counter();families=collections.Counter()
    dated_defs=set();dated_quote=set();instruments=set();financial_definitions=set();stages=collections.Counter();all6=collections.Counter();public_mm=0
    for r in cases():
        op=r['operation'];inp=encode(r['input']);decode(op,inp);identity=op+'\t'+inp+'\n'
        if identity in seen:raise ValueError('duplicate semantic input (answer excluded)')
        seen.add(identity);identities.append(identity)
        try:
            e=evaluate(op,r['input']);validate_expected(op,e)
        except BaseException as exc:
            failure={'row':r,'exception':repr(exc)}
            q=ROOT/('execution-failure-'+str(len(list(ROOT.glob('execution-failure-*.json')))+1)+'.json')
            q.write_text(json.dumps(failure,indent=2)+'\n');raise
        r['expected']=e
        row=[r[k] if k not in ('input','expected') else encode(r[k]) for k in COLUMNS]
        if any('\t' in x or '\n' in x or '\r' in x for x in row):raise ValueError('record control character')
        rows.append('\t'.join(row)+'\n');ops[op]+=1;profiles[r['profile']]+=1;scopes[r['scope']]+=1
        for f in r['families'].split(','):families[f]+=1
        if op.startswith('dated_'):
            de=encode(r['input']['definition']);dated_defs.add(de);dated_quote.add(de+'\t'+r['input']['yield_bits'])
            dd=dict(r['input']['definition']);dd.pop('date_cap');dd.pop('max_bundles');financial_definitions.add(encode(dd));dd.pop('settlement');dd.pop('selection');instruments.add(encode(dd))
            stages[e['status']]+=1
            if op=='dated_mm' and e['status']=='complete':public_mm+=1
            if 'risk' in e:
                for name,ob in e['risk']['views'].items():all6[name+':'+ob['class']]+=1
    return rows,identities,{'records':len(rows),'unique_serialized_operation_inputs':len(seen),'operation_counts':dict(sorted(ops.items())),
        'profile_counts':dict(sorted(profiles.items())),'scope_counts':dict(sorted(scopes.items())),
        'family_annotation_counts_NOT_closure':dict(sorted(families.items())),
        'unique_dated_public_call_definitions':len(dated_defs),'unique_dated_financial_definitions_without_caps':len(financial_definitions),'unique_instrument_identities':len(instruments),'unique_dated_definition_and_quote':len(dated_quote),
        'dated_mm_complete_exact_observations':public_mm,'dated_status_counts':dict(sorted(stages.items())),
        'dated_risk_view_outcomes':dict(sorted(all6.items()))}

def header_list(manifest,req,used,census,body,inputs,metadata):
    src=[{'name':name,'sha256':sha((REPO/name).read_bytes()),'fnv64':fnv((REPO/name).read_bytes())} for name in SOURCE_NAMES]
    contract=next(x for x in manifest if x['source']=='docs/M8_CANADIAN_CONTRACT.md')
    h=[('schema','1'),('fixture-set','canadian-prescribed-numeric-v1'),('domain','bonds'),
      ('description','Independent represented-input Canadian numeric stages and certified conditional elementary intervals'),
      ('reference','Adopted complete Canadian local engineering contract; exact rational/RNE and rational-tail proofs'),
      ('reference-version','CA-IIAC13-fixed-2026-10-23-local-v1'),('reference-source','docs/M8_CANADIAN_CONTRACT.md'),
      ('tolerance','exact_seed/dated_exact/dated_seeded integer-RNE; native_2u ideal interval plus explicit conditional allowance'),
      ('reproduction','python3 -I validation/generator/canadian_numeric_oracles.py --check'),
      ('generated-at',metadata['generated-at']),('platform',metadata['platform']),('python',metadata['python']),
      ('generator-sha256',src[0]['sha256']),('generator-fnv64',src[0]['fnv64']),
      ('generator-input-sources',encode(src)),('contract-sha256',contract['sha256']),('contract-fnv64',fnv(used[str(REPO/contract['source'])])),
      ('incorporated-components',encode([{'source':x['source'],'sha256':x['sha256'],'fnv64':fnv(used[str(REPO/x['source'])])} for x in manifest[1:15]])),
      ('reader-sha256',manifest[15]['sha256']),('requirements-rows','72'),('requirements-sha256',sha(req)),('requirements-fnv64',fnv(req)),
      ('column-count',str(len(COLUMNS))),('record-count',str(census['records'])),('body-sha256',sha(body)),('body-fnv64',fnv(body)),
      ('records-sha256',sha(body.split(b'\n',1)[1])),('records-fnv64',fnv(body.split(b'\n',1)[1])),
      ('input-columns','operation,input'),('input-encoding','canonical strict typed JSON; operation TAB input LF'),
      ('unique-input-count',str(census['unique_serialized_operation_inputs'])),('input-sha256',sha(inputs)),('input-fnv64',fnv(inputs)),
      ('profile-counts',encode(census['profile_counts'])),('operation-counts',encode(census['operation_counts'])),
      ('dated-definition-count',str(census['unique_dated_public_call_definitions'])),('dated-quote-count',str(census['unique_dated_definition_and_quote'])),
      ('qualification','Seeded valid geometry is not native/public reachability; exact source/cash and ideal/inverse corpora are separate; no universal libm theorem or Canadian Rust acceptance')]
    return h

def parse_old(data):
    if b'\r' in data or not data.endswith(b'\n'):raise ValueError('noncanonical LF fixture bytes')
    head,body=data.split(b'\n\n',1);h={}
    for line in head.decode().splitlines():
        k,v=line.split(': ',1)
        if k in h:raise ValueError('duplicate header')
        h[k]=v
    if h.get('schema')!='1' or body.split(b'\n',1)[0].decode().split('\t')!=list(COLUMNS):raise ValueError('unknown/malformed schema')
    for key in ('generated-at','platform','python'):
        if key not in h:raise ValueError('missing metadata')
    return {k:h[k] for k in ('generated-at','platform','python')}

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,default=REPO/'validation/fixtures/bonds/canadian-numeric-oracle.fixture');p.add_argument('--check',action='store_true');p.add_argument('--census-output',type=Path)
    args=p.parse_args();output=args.output.resolve()
    manifest,req,used=loadpins()
    if output in [Path(x).resolve() for x in used] or output==Path(__file__).resolve():raise ValueError('output aliases consumed source')
    if args.check:
        old=output.read_bytes();metadata=parse_old(old)
    else:
        if output.exists():raise FileExistsError('exclusive output already exists')
        metadata={'generated-at':datetime.datetime.now(datetime.timezone.utc).isoformat(timespec='seconds').replace('+00:00','Z'),
                  'platform':platform.platform(),'python':platform.python_version()}
    rows,identities,census=rows_and_census();body=('\t'.join(COLUMNS)+'\n'+''.join(rows)).encode();ib=''.join(identities).encode()
    headers=header_list(manifest,req,used,census,body,ib,metadata)
    expected=(''.join(k+': '+v+'\n' for k,v in headers)+'\n').encode()+body
    # Entire read interval is byte-guarded, including every frozen component and helper.
    for path,b in used.items():
        if Path(path).read_bytes()!=b:raise ValueError('consumed input changed during run '+path)
    if args.check:
        if old!=expected:raise ValueError('reproduction mismatch')
    else:
        with output.open('xb') as f:f.write(expected)
    census.update({'status':'PASS','mode':'check' if args.check else 'exclusive-create','fixture_sha256':sha(expected),'fixture_bytes':len(expected),
                  'input_sha256':sha(ib),'body_sha256':sha(body),'generator_sources':{name:sha((REPO/name).read_bytes()) for name in SOURCE_NAMES},
                  'contract_sha256':manifest[0]['sha256'],'requirements_sha256':sha(req)})
    if args.census_output:
        with args.census_output.open('x') as f:json.dump(census,f,indent=2);f.write('\n')
    print(encode(census))

if __name__=='__main__':main()
