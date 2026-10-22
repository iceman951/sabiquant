#!/usr/bin/env python3
"""Attested postproduction Treasury semantic mutation controls (stdlib only).

Creates new, exact-copy scratch crates outside this repository. Each mutation
changes one declared numerical/source/cash expression in its own crate. Frozen
66 accepted public hand-test adapters are copied unchanged; two supplemental public financial
controls are appended in a cfg(test) module, identically in baseline/mutants.
A compile failure is never a kill. A kill requires a successfully compiled
crate followed by a failing assertion at a hand/private-test location. Surviving
mutants are retained explicitly, not converted into claimed coverage.
"""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
NUMERIC = 'src/bonds/treasury/numeric.rs'
PARENT = 'src/bonds/treasury.rs'
CASH = 'src/bonds/treasury/cash.rs'
HANDS = ('bonds_treasury_numeric_hand', 'bonds_treasury_source_cash_hand')
INTEGRATION_PINS = {
 'tests/bonds_treasury_numeric_hand.rs': '117c38b35b3f2061f68a50808458cd3317de1ec34d93eda6b39516f96bdf64fb',
 'tests/bonds_treasury_source_cash_hand.rs': '546f9e802ee36249a8ab481757026098e0d5d8001496fe77bd643b7288895a83',
}
PINS = {
 'docs/M8_US_TREASURY_CONTRACT.md': '50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93',
 'docs/M8_US_TREASURY_CASH_CONTRACT.md': '25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585',
 'validation/generator/treasury_source_cash_oracles.py': '046214dc324e0fab1dd5f3387de14b44b61c39547c6e2fcf4f5522896338782e',
 'validation/fixtures/bonds/treasury-source-cash-oracle.fixture': '2347711a897cbf627da1afae1ea17881aea35a6b1609b7ecd3fc8e8f538c5d31',
 'validation/generator/treasury_ideal_oracles.py': 'ca822f899925583ccb15187c110c05bd0d29e8e6ec932ba859461c5942bdb5b8',
 'validation/fixtures/bonds/treasury-ideal-price-risk-oracle.fixture': '08bdb9ce17a1c523ed7cbe456185be1b305a950ef3261ca0148ff5a8471b4a31',
 'validation/fixtures/bonds/treasury-ideal-inverse-oracle.fixture': '40a16012bd1378122b6f17f1792bad8b66433487c926ff3f40d85a266259e517',
 'validation/generator/treasury_numeric_oracles.py': '4915101db4c1040702e820570cbe117ceddc72fd55413a7aedd91397256ede54',
 'validation/fixtures/bonds/treasury-numeric-oracle.fixture': 'b24208d7150c65a0ad3b5714f588cfbbd3b6481a511ff086fa94ade132464adc',
 'validation/hand/bonds_treasury_numeric_hand.rs': '09676aff8114429ebfd28271830c206e2d40a895ff0b7b90817f0cc230371cab',
 'validation/hand/bonds_treasury_source_cash_hand.rs': '546f9e802ee36249a8ab481757026098e0d5d8001496fe77bd643b7288895a83',
}

# These tests are postproduction controls, not additional blind hand rows.
# Their expected cash units and threshold are financial contract statements;
# neither test merely compares an implementation constant with itself.
CONTROLS = r'''
// BEGIN ATTESTED POSTPRODUCTION SEMANTIC MUTATION CONTROLS
#[cfg(test)]
mod treasury_semantic_mutation_controls {
    use super::*;
    use crate::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
    use crate::calendar::EndOfMonth;
    use crate::settlement::{CashAggregation, SettlementEvidence, SettlementRounding};
    use super::super::{UsTreasury, TreasuryDailyBasis, TreasurySourcePolicy, TreasuryTieRule};
    fn financial_zero_coupon() -> UsTreasury {
        let issue = Date::from_ymd(2020, 1, 15).unwrap();
        let quasi: Vec<_> = (0..=19).map(|i| issue.checked_add_months(i*6).unwrap()).collect();
        UsTreasury::new(issue, issue, quasi[19], DecimalAmount::new(0, 0).unwrap(), quasi[1], quasi,
            EndOfMonth::Ignore, TreasurySourcePolicy::new(TreasuryDailyBasis::ExactRatio,
            TreasuryTieRule::NearestTiesEven), 20).unwrap()
    }
    /// FR-UST-09: positive p256 primitive-superset, not a dated financial seed.
    /// Independently: M=1+2^-53; q=1/M; floor(q*2^256)=m below.
    /// The reciprocal endpoints straddle M, distinguishing low/high wiring.
    #[test]
    fn primitive_superset_reciprocal_midpoint_selects_low_endpoint() {
        let m = Wide::<4>::from_u128(340282366920938425684442744474610696191_u128)
            .checked_shl(128).unwrap()
            .checked_add(Wide::from_u128(340282366762482138434845949836866355199_u128)).unwrap();
        let one = Endpoint { significand: Wide::<4>::from_u128(1).checked_shl(255).unwrap(), exponent: -255 };
        let input = Seeds { b: Endpoints { low: one, high: one }, q: Endpoints {
            low: Endpoint { significand: m, exponent: -256 },
            high: Endpoint { significand: m.checked_add(Wide::from_u128(1)).unwrap(), exponent: -256 },
        }};
        let observed = discount(&input, 0, 0).unwrap();
        assert_eq!((observed.significand, observed.exponent), (1_u64 << 52, -52));
    }
    /// FR-UST-16/17: a real finite source price lies between 2^969 and 2^970.
    #[test]
    fn financial_price_below_2pow970_keeps_unscaled_cached_residual() {
        let t = financial_zero_coupon();
        let plan = TreasuryPricingPlan::new(&t, t.original_issue_date(), 19).unwrap();
        let y = f64::from_bits((-2.0_f64).to_bits()-5);
        let dirty = plan.evaluate(y).unwrap().dirty_per_100();
        assert!(f64::from_bits(1992_u64<<52) < dirty && dirty < f64::from_bits(1993_u64<<52));
        let target = BondPrice::new(dirty, BondPriceKind::Dirty).unwrap();
        let report = plan.yield_from_price(target, RootBracket::new(y, 0.0).unwrap(),
            SolverOptions::new(1e-12, 1e-12, 1).unwrap()).unwrap();
        assert_eq!(report.residual_scale_exponent(), 0);
        assert_eq!(report.solve_report().evaluations().function(), 1);
        assert_eq!(report.solve_report().residual().to_bits(), 0);
        assert_eq!(report.amount_residual().unwrap().to_bits(), 0);
    }
    /// FR-UST-22/23: invoice units remain per1000 when published AI equals0.
    #[test]
    fn zero_ai_invoice_retains_price_times_ten_and_total_stage_units() {
        let t = financial_zero_coupon();
        let plan = TreasuryPricingPlan::new(&t, t.original_issue_date(), 19).unwrap();
        let instruction = plan.clean_accrued_cash(DecimalAmount::new(1000,0).unwrap(),
            SettlementEvidence::exact_decimal(DecimalAmount::new(100,0).unwrap()),
            CashAggregation::RoundTotal, SettlementRounding::new(AmountQuantum::new(1,2).unwrap(),
            AmountRounding::Exact)).unwrap();
        match instruction.evaluate().unwrap() {
            super::super::TreasuryCashDecision::Determined(receipt) => assert_eq!(receipt.amount(), DecimalAmount::new(1000,0).unwrap()),
            other => panic!("expected determined invoice, got {other:?}"),
        }
    }
}
// END ATTESTED POSTPRODUCTION SEMANTIC MUTATION CONTROLS
'''

# Exact, unique source anchors make production drift fail closed. Replacement
# bodies implement recognizable domain mistakes; none inserts a synthetic panic.
MUTANTS = [
 ('j0_uses_regular_b', NUMERIC, 'N1 j0 simple prefix replaced with full regular prefix',
  'let denominator = if j == 0 {\n        seeds.q',
  'let denominator = if j == 0 {\n        seeds.b'),
 ('renumber_original_j', NUMERIC, 'N1 skipped original quasi ordinal is renumbered',
  'discount(seeds, bundle.j, bundle.index)?',
  'discount(seeds, bundle.j.saturating_sub(1), bundle.index)?'),
 ('rounded_seed_prefix', NUMERIC, 'N1 exact seeds replaced by risk binary64 prefix observations',
  '''let (nb, nq) = prefix_numerators(y, r, s, "discount seeds")?;
    let denominator = s
        .checked_mul(2)
        .ok_or_else(|| capacity(None, "discount seeds", "derive prefix"))?;
    Ok(Seeds {
        b: seed_endpoints(nb, 2)?,
        q: seed_endpoints(nq, denominator)?,
    })''',
  '''let (delta, q) = risk_prefix(y, r, s)?;
    Ok(Seeds {
        b: seed_endpoints(exact_y(delta, "discount seeds", "decode yield")?, 2)?,
        q: seed_endpoints(exact_y(q, "discount seeds", "decode yield")?, 1)?,
    })'''),
 ('select_high_factor_endpoint', NUMERIC, 'N1 selected reciprocal low replaced by high endpoint',
  'factor_observation(low, index)', 'factor_observation(_high, index)'),
 ('source_pv_double_round', NUMERIC, 'N3 source PV first rounds into extended53 before IEEE',
  'let bits = round_wide_ratio_to_f64(numerator, amount.denominator, factor.exponent)',
  '''let rounded = extended_ratio(numerator, amount.denominator, factor.exponent, index, PHASE)?;
    let bits = round_wide_ratio_to_f64(Scalar::from_u128(u128::from(rounded.significand)), Scalar::from_u128(1), rounded.exponent)'''),
 ('kappa_cross_half', NUMERIC, 'N4 complete second derivative loses half its cross term',
  'scalar_product(&[2, j, r, s, d, q], index, PHASE)?',
  'scalar_product(&[j, r, s, d, q], index, PHASE)?'),
 ('convexity_half', NUMERIC, 'N4 applies an extra convexity half',
  'normalized(self.second, self.dirty, self.yield_rate, "convexity")',
  'normalized(self.second, self.dirty, self.yield_rate, "convexity").map(|value| value / 2.0)'),
 ('macaulay_b_modified', NUMERIC, 'N4 incorrectly extends periodic Macaulay identity to stubs',
  '''normalized(
            self.moment,
            self.dirty,
            self.yield_rate,
            "macaulay duration",
        )''',
  'self.modified_duration().map(|value| value * (1.0 + self.yield_rate / 2.0))'),
 ('normalized_from_ieee_raw', NUMERIC, 'N4 normalized getter depends on separately admitted raw IEEE value',
  '''let bits = round_wide_ratio_to_f64(sum, RiskWide::from_u128(u128::from(sig)), -8192 - exponent)
        .ok_or_else(|| capacity(None, phase, "normalize risk value"))?;
    finish_risk(bits, false, phase)''',
  '''let _ = (sig, exponent);
    raw(sum, false, phase).map(|value| value / dirty)'''),
 ('clean_target_raw_ai', NUMERIC, 'N7 shifts clean target by analytical raw AI instead of publication',
  'BondPriceKind::Clean => exact_sum(&[target.value_per_100_face(), self.accrued_per_100])',
  'BondPriceKind::Clean => exact_sum(&[target.value_per_100_face(), self.raw_accrued_per_100])'),
 ('residual_threshold_969', NUMERIC, 'N7 scales too early at 2^969 rather than2^970',
  'const SCALE_THRESHOLD: u64 = 1993_u64 << 52;',
  'const SCALE_THRESHOLD: u64 = 1992_u64 << 52;'),
 ('dated_replaced_by_issue', PARENT, 'E dated/source coupon fraction incorrectly starts at issue',
  '''let d0 = quasi_dates[1]
            .days_since(original_dated_date)''',
  '''let d0 = quasi_dates[1]
            .days_since(original_issue_date)'''),
 ('long_skipped_ai_reset', PARENT, 'E long first accrual prematurely resets at skipped Q1',
  'if next_quasi_index > f || treasury.first_period == TreasuryFirstPeriod::Regular',
  'if next_quasi_index >= f || treasury.first_period == TreasuryFirstPeriod::Regular'),
 ('zero_ai_invoice_wrong_units', CASH, 'T clean invoice leaves price per100 whenever AI is zero',
  '''let price = value
                .magnitude
                .checked_mul_small(10)''',
  '''let price = value
                .magnitude
                .checked_mul_small(if published.is_zero() { 1 } else { 10 })'''),
]

def sha(data):
    return hashlib.sha256(data).hexdigest()

def manifest(files):
    return {name: {'bytes':len(data), 'sha256':sha(data)} for name,data in sorted(files.items())}

def canonical(obj):
    return (json.dumps(obj, sort_keys=True, separators=(',',':'))+'\n').encode()

def snapshot():
    names = {'Cargo.toml','Cargo.lock','clippy.toml','README.md','CHANGELOG.md','LICENSE-MIT','LICENSE-APACHE'}
    names.update(str(p.relative_to(ROOT)) for p in (ROOT/'src').rglob('*') if p.is_file())
    names.update(str(p.relative_to(ROOT)) for p in (ROOT/'tests/support').rglob('*.rs'))
    names.update('tests/'+name+'.rs' for name in HANDS)
    return {name:(ROOT/name).read_bytes() for name in sorted(names)}

def create(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('xb') as output:
        output.write(data)
    if path.read_bytes()!=data:
        raise RuntimeError('copy mismatch: '+str(path))

def command(argv, crate, env, folder, label):
    started=dt.datetime.now(dt.timezone.utc).isoformat()
    completed=subprocess.run(argv, cwd=crate, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    output=completed.stdout.decode('utf-8', errors='replace')
    log=folder/(label+'.log');create(log,completed.stdout)
    return {'argv':argv,'started_at':started,'finished_at':dt.datetime.now(dt.timezone.utc).isoformat(),
            'returncode':completed.returncode,'log':str(log),'log_sha256':sha(completed.stdout)},output

def test_summary(output):
    pattern=r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;'
    return [dict(zip(('result','passed','failed','ignored','measured','filtered'),m)) for m in re.findall(pattern, output)]

def test_assertions(output, files):
    locations=[]
    for path,line in re.findall(r'panicked at ([^\s:]+):(\d+):\d+:',output):
        normalized=path.replace('\\','/');line=int(line)
        if normalized in ('tests/'+name+'.rs' for name in HANDS):
            locations.append({'path':normalized,'line':line,'kind':'frozen-hand-assertion'})
        elif normalized in (NUMERIC,CASH,PARENT):
            source=files[normalized].decode();prefix='\n'.join(source.splitlines()[:line])
            if '#[cfg(test)]' in prefix:
                locations.append({'path':normalized,'line':line,'kind':'private-or-supplemental-test-assertion'})
    return locations

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,help='create-new scratch output, outside repository')
    parser.add_argument('--only',action='append',default=[],help='selected mutant id (repeatable); subset scope is reported')
    parser.add_argument('--list',action='store_true',help='list mutation domains without running')
    parser.add_argument('--cargo',default='cargo');parser.add_argument('--rustc',default='rustc')
    args=parser.parse_args()
    if args.list:
        print(json.dumps([{'id':m[0],'path':m[1],'domain':m[2]}for m in MUTANTS],indent=2));return
    selected=[m for m in MUTANTS if not args.only or m[0] in args.only]
    unknown=set(args.only)-{m[0]for m in MUTANTS}
    if unknown:parser.error('unknown mutants: '+','.join(sorted(unknown)))
    destination=args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix='sabiquant-treasury-semantic-mutants-'))
    if destination==ROOT or ROOT in destination.parents or destination in ROOT.parents:
        parser.error('scratch output must be outside repository and not its ancestor')
    if args.output:destination.mkdir(parents=True,exist_ok=False)
    original=snapshot();runner=Path(__file__).read_bytes()
    pinned={name:(ROOT/name).read_bytes() for name in PINS}
    for name,data in pinned.items():
        if sha(data)!=PINS[name]:raise ValueError('frozen pin mismatch: '+name)
    # Accepted public integration adapters are immutable inputs to this run.
    # Numeric's adapter uses public Interval bound inspectors instead of private
    # constructors and has comment/lint fixes. Both original and adapter pins
    # are preserved; no claim of byte identity between those two files is made.
    for name, expected in INTEGRATION_PINS.items():
        if sha(original[name])!=expected:
            raise ValueError('accepted integration adapter pin mismatch: '+name)
    evidence={'schema':1,'started_at':dt.datetime.now(dt.timezone.utc).isoformat(),'python':sys.version,
              'platform':platform.platform(),'runner_sha256':sha(runner),'source_manifest':manifest(original),
              'source_manifest_sha256':sha(canonical(manifest(original))),'frozen_manifest':manifest(pinned),
              'integration_adapter_pins':INTEGRATION_PINS,'numeric_hand_adapter_qualification':'Public Interval bound inspectors plus comment/lint corrections; exact financial endpoints preserved; original and accepted adapter separately pinned',
              'controls_sha256':sha(CONTROLS.encode()),'controls_postproduction':True,'controls_classification':{'public_financial':2,'positive_p256_primitive_superset':1},'controls_count':3,
              'scope':'66 accepted unchanged hand adapters + Treasury private focused tests +2 postproduction public financial controls +1 positive p256 primitive-superset; no full corpus per mutant',
              'family_qualification':'Selected concrete semantic mistakes only, not all76 obligations or a completeness proof',
              'nonconsumed_snapshot_files_excluded_from_end_guard':['README.md','CHANGELOG.md'],
              'selected_mutants':[m[0]for m in selected],'full_catalogue':len(selected)==len(MUTANTS),'cases':[], 'result':'incomplete'}
    failure=None
    try:
        cargo,rustc=shutil.which(args.cargo),shutil.which(args.rustc)
        if not cargo or not rustc:raise ValueError('matching cargo/rustc required')
        env=os.environ.copy()
        for key in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','LLVM_PROFILE_FILE','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_BUILD_TARGET'):
            env.pop(key,None)
        env.update(RUSTC=rustc,CARGO_TARGET_DIR=str(destination/'target'),CARGO_INCREMENTAL='0',RUST_BACKTRACE='1')
        evidence['toolchain']={}
        for name,executable,flags in [('cargo',cargo,['-V']),('rustc',rustc,['-Vv'])]:
            c,text=command([executable,*flags],ROOT,env,destination,name+'-version')
            if c['returncode']:raise RuntimeError(name+' version failed')
            evidence['toolchain'][name]={'command':c,'version':text,'executable_sha256':sha(Path(executable).read_bytes())}
        for mutation in [None,*selected]:
            name='baseline' if mutation is None else mutation[0]
            folder=destination/name;folder.mkdir();crate=folder/'crate';crate.mkdir()
            for relative,data in original.items():create(crate/relative,data)
            copied={relative:(crate/relative).read_bytes()for relative in original}
            if copied!=original:raise RuntimeError('not an exact initial copy: '+name)
            case={'id':name,'initial_exact_copy':True,'initial_manifest_sha256':evidence['source_manifest_sha256'],'commands':[]}
            changed={}
            if mutation:
                _,relative,domain,before,after=mutation
                source=original[relative].decode()
                if source.count(before)!=1:raise ValueError('mutation anchor is not unique: '+name)
                content=source.replace(before,after,1).encode();(crate/relative).write_bytes(content);changed[relative]=content
                case.update(domain=domain,mutation={'path':relative,'before':before,'after':after,'original_sha256':sha(original[relative]),'mutated_sha256':sha(content)})
            source=(crate/NUMERIC).read_bytes();(crate/NUMERIC).write_bytes(source+CONTROLS.encode());changed[NUMERIC]=source+CONTROLS.encode()
            case['injected_controls']={'module':NUMERIC,'original_prefix_sha256':sha(source),'appended_sha256':sha(CONTROLS.encode()),'cfg_test_only':True}
            compile_cmd=[cargo,'test','--offline','--locked','--no-run','--lib']
            for hand in HANDS:compile_cmd+=['--test',hand]
            c,text=command(compile_cmd,crate,env,folder,'compile');case['commands'].append(c)
            if c['returncode']:
                case['classification']='compile-failure-not-kill';case['compile_output_summary']=text[-3000:]
            else:
                outputs=[];assertions=[]
                for label,selection in [('numeric-hands',['--test',HANDS[0]]),('source-cash-hands',['--test',HANDS[1]]),('private-boundaries',['--lib','bonds::treasury::'])]:
                    c,text=command([cargo,'test','--offline','--locked',*selection,'--','--test-threads=1','--nocapture'],crate,env,folder,label)
                    c['test_summaries']=test_summary(text);case['commands'].append(c);outputs.append(c)
                    assertions.extend(test_assertions(text,{k:(crate/k).read_bytes()for k in (NUMERIC,CASH,PARENT)}))
                case['assertion_locations']=assertions
                summaries=[s for c in outputs for s in c['test_summaries']]
                if len(summaries)!=3:raise RuntimeError('three selected test binaries did not report complete summaries: '+name)
                case['test_counts']={'passed':sum(int(s['passed'])for s in summaries),'failed':sum(int(s['failed'])for s in summaries)}
                if mutation is None:
                    if any(c['returncode']for c in outputs) or int(summaries[0]['passed'])!=28 or int(summaries[1]['passed'])!=38 or int(summaries[2]['passed'])<11:
                        case['classification']='baseline-failed';evidence['cases'].append(case);raise RuntimeError('unmutated baseline controls must pass exactly28+38 hands and>=11private controls')
                    case['classification']='baseline-passed'
                elif any(c['returncode']for c in outputs) and assertions:
                    case['classification']='killed-by-runtime-test-assertion'
                elif any(c['returncode']for c in outputs):
                    case['classification']='runtime-failure-not-assertion-kill'
                else:case['classification']='survived-selected-controls'
            actual_changes={k:(crate/k).read_bytes()for k,v in original.items()if(crate/k).read_bytes()!=v}
            if actual_changes!=changed:raise RuntimeError('unexpected copied-source changes: '+name)
            case['changed_files']={k:sha(v)for k,v in changed.items()};evidence['cases'].append(case)
            print(json.dumps({'case':name,'classification':case['classification'],'counts':case.get('test_counts')}),flush=True)
            if mutation is None and case['classification']!='baseline-passed':raise RuntimeError('baseline compile failed')
        counts={}
        for c in evidence['cases'][1:]:counts[c['classification']]=counts.get(c['classification'],0)+1
        evidence['mutation_classifications']=counts
        evidence['result']='completed'
    except Exception as error:
        failure=error;evidence['result']='failed';evidence['failure']={'type':type(error).__name__,'message':str(error)}
    evidence['original_guard']={'source_unchanged':all((ROOT/p).read_bytes()==data for p,data in original.items() if p not in ('README.md','CHANGELOG.md')),
                                'frozen_unchanged':all((ROOT/p).read_bytes()==data for p,data in pinned.items()),
                                'runner_unchanged':Path(__file__).read_bytes()==runner}
    if not all(evidence['original_guard'].values()):
        evidence['result']='invalidated-original-input-changed';failure=RuntimeError('original sources/frozen inputs/runner changed during run')
    evidence['finished_at']=dt.datetime.now(dt.timezone.utc).isoformat();report=destination/'evidence.json';create(report,(json.dumps(evidence,indent=2,sort_keys=True)+'\n').encode())
    print(json.dumps({'result':evidence['result'],'evidence':str(report),'evidence_sha256':sha(report.read_bytes()),'mutations':evidence.get('mutation_classifications',{})}),flush=True)
    if failure:raise SystemExit(str(failure))

if __name__=='__main__':
    main()
