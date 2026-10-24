#!/usr/bin/env python3
"""Attested postproduction Canadian semantic mutation controls (stdlib only).

Builds one create-new scratch crate outside this repository from exact copies
of the Canadian production sources, every Canadian integration test, shared
test support and the frozen corpora they read. Before each case the four
production modules are restored and byte-checked; a mutant then changes one
declared financial/numerical/source/cash expression. Tier 1 (debug) runs the
29 frozen hands, the private Canadian unit tests and the full private N kernel
replay (validation/harness/canadian_numeric_kernel.rs) plus one postproduction
public residual-scale boundary control. Only a tier-1 survivor
runs tier 2 (release): the complete S, P and Y public corpus consumers.

A compile failure is never a kill. A kill requires a compiled crate followed
by a failing assertion located in a test file or a cfg(test) region. Surviving
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
sys.path.insert(0, str(Path(__file__).resolve().parent))
import canadian_numeric_kernel_check as kernel  # noqa: E402

PARENT = 'src/bonds/canadian.rs'
SOURCE = 'src/bonds/canadian/source.rs'
NUMERIC = 'src/bonds/canadian/numeric.rs'
CASH = 'src/bonds/canadian/cash.rs'
PRODUCTION = (PARENT, SOURCE, NUMERIC, CASH, 'src/bonds/canadian/wide.rs')
HANDS = {'bonds_canadian_api_hand': 7, 'bonds_canadian_numeric_hand': 8,
         'bonds_canadian_source_cash_hand': 14}
CORPORA = {'bonds_canadian_source_cash_oracles': 1, 'bonds_canadian_ideal_oracles': 3}
# Postproduction public controls authored after a mutation survived; not hands.
SUPPLEMENTS = {'bonds_canadian_solver_boundaries': 1}
# Literal include_str!/include_bytes! inputs and quoted repository-relative
# validation/docs paths read at run time are discovered from the copied tests.
INCLUDE = re.compile(r'include_(?:bytes|str)!\s*\(\s*"([^"\n]+)"')
RUNTIME = re.compile(r'"((?:validation|docs)/[^"\n]+)"')

# Exact, unique source anchors make production drift fail closed. Replacement
# bodies implement recognizable domain mistakes; none inserts a synthetic panic.
MUTANTS = [
 ('last_coupon_mm_two_remaining', PARENT, 'FR-CAD selection: money-market applied with two remaining coupons',
  'if count == 1 {\n                    CanadianYieldConvention::MoneyMarket',
  'if count <= 2 {\n                    CanadianYieldConvention::MoneyMarket'),
 ('settlement_day_coupon_future', PARENT, 'coupon paid on the settlement date stays in future cash flows',
  '            .coupon_dates\n            .partition_point(|date| *date <= settlement);',
  '            .coupon_dates\n            .partition_point(|date| *date < settlement);'),
 ('compound_quote_settlement_ai', PARENT, 'compound quote accrued uses settlement (payable) accrued',
  'quote_accrued_per_100: if convention == CanadianYieldConvention::Compound {\n                valuation_observed',
  'quote_accrued_per_100: if convention == CanadianYieldConvention::Compound {\n                settlement_observed'),
 ('compound_prices_payable_coupons', PARENT, 'compound/MM swap valuation and payable coupon sources',
  'bond.valuation_coupon(index)?\n            } else {\n                bond.payable_coupon(index)?',
  'bond.payable_coupon(index)?\n            } else {\n                bond.valuation_coupon(index)?'),
 ('par_on_contractual_date', PARENT, 'terminal par discounted from the contractual, not true, maturity',
  'amount = amount.with_par(index)?;\n                bond.true_maturity_date',
  'amount = amount.with_par(index)?;\n                date'),
 ('monthly_payable_settlement_ai', PARENT, 'monthly exception dropped: settlement accrued uses payable rule',
  'if schedule.frequency == CanadianFrequency::Monthly {',
  'if false && schedule.frequency == CanadianFrequency::Monthly {'),
 ('payable_late_rule_dropped', SOURCE, 'payable accrued ignores the late-period C/f - remaining/365 rule',
  'u64::from(365 - f * (fragment.period_days - days))', 'u64::from(f * days)'),
 ('payable_basis_366', SOURCE, 'payable accrued day basis 366 instead of 365',
  '(n, f * 365)', '(n, f * 366)'),
 ('coordinate_drops_second_fragment', SOURCE, 'exact quasi coordinate omits the second proper fragment',
  '+ u64::from(a.0) * u64::from(b.1)\n            + u64::from(b.0) * u64::from(a.1);',
  '+ u64::from(a.0) * u64::from(b.1);'),
 ('terminal_par_omitted', SOURCE, 'terminal bundle omits the par100 principal',
  '!wide::mul_small_assign(&mut par, 100)', '!wide::mul_small_assign(&mut par, 0)'),
 ('clean_bridge_settlement_ai', NUMERIC, 'clean price subtracts settlement instead of quote accrued',
  'exact_sum(&[dirty, -metadata.quote_accrued_per_100])',
  'exact_sum(&[dirty, -metadata.settlement_accrued_per_100])'),
 ('clean_target_settlement_ai', NUMERIC, 'clean yield target shifted by settlement instead of quote accrued',
  'CanadianPriceBasis::Clean => exact_sum(&[target.value(), metadata.quote_accrued_per_100]),',
  'CanadianPriceBasis::Clean => exact_sum(&[target.value(), metadata.settlement_accrued_per_100]),'),
 ('mm_basis_360', NUMERIC, 'money-market affine factor uses a 360-day year',
  'wide::set_u128(&mut self.base, 365),', 'wide::set_u128(&mut self.base, 360),'),
 ('log_scale_drops_frequency', NUMERIC, 'compound scaled logarithm omits the frequency multiplier',
  'f64::from(f) * ell', 'ell'),
 ('residual_threshold_969', NUMERIC, 'residual scaling starts too early at 2^969 rather than 2^970',
  'const SCALE_THRESHOLD: u64 = 1993_u64 << 52;', 'const SCALE_THRESHOLD: u64 = 1992_u64 << 52;'),
 ('tiny_threshold_2pow49', NUMERIC, 'tiny-yield identity branch widened from 2^-50 to 2^-49',
  'const TINY: f64 = f64::from_bits(973_u64 << 52);', 'const TINY: f64 = f64::from_bits(974_u64 << 52);'),
 ('residual_sign_flip', NUMERIC, 'amount residual sign lost when |dirty| < |target|',
  'checked(wide::copy_into(&b, &mut a), None, PHASE, OP)?;\n        !dirty.is_sign_negative()',
  'checked(wide::copy_into(&b, &mut a), None, PHASE, OP)?;\n        dirty.is_sign_negative()'),
 ('accrued_cash_valuation_ai', CASH, 'accrued cash uses valuation instead of settlement accrued',
  'value: self.settlement_ai,', 'value: self.valuation_ai,'),
 ('coupon_cash_valuation_coupon', CASH, 'coupon cash pays the valuation instead of payable coupon',
  'value: self.payable_coupon(index)?,', 'value: self.valuation_coupon(index)?,'),
 ('ties_even_as_away', CASH, 'nearest/even cash rounding breaks ties away from zero',
  'midpoint == Ordering::Greater || (midpoint == Ordering::Equal && wide::is_odd(quotient))',
  'midpoint != Ordering::Less'),
 ('ceiling_sign_flip', CASH, 'toward-positive-infinity cash rounding uses the floor direction',
  'AmountRounding::TowardPositiveInfinity => !negative,', 'AmountRounding::TowardPositiveInfinity => negative,'),
 ('components_double_accrued', CASH, 'round-components invoice also adds accrued inside the capital stage',
  'let initial_accrued = if components { &Source::ZERO } else { accrued };',
  'let initial_accrued = accrued;'),
]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(obj):
    return (json.dumps(obj, sort_keys=True, separators=(',', ':')) + '\n').encode()


def snapshot():
    names = {'Cargo.toml', 'Cargo.lock', 'clippy.toml', 'README.md', 'CHANGELOG.md',
             'LICENSE-MIT', 'LICENSE-APACHE'}
    names.update(str(p.relative_to(ROOT)) for p in (ROOT / 'src').rglob('*') if p.is_file())
    names.update(str(p.relative_to(ROOT)) for p in (ROOT / 'tests/support').rglob('*.rs'))
    names.update('tests/%s.rs' % name for name in (*HANDS, *SUPPLEMENTS, *CORPORA))
    for name in sorted(n for n in names if n.startswith('tests/') and n.endswith('.rs')):
        body = (ROOT / name).read_text()
        for literal in INCLUDE.findall(body):
            names.add(str(((ROOT / name).parent / literal).resolve().relative_to(ROOT)))
        # Doc examples may name absent paths; only existing files are inputs.
        names.update(x for x in RUNTIME.findall(body) if (ROOT / x).is_file())
    result = {}
    for name in sorted(names):
        path = ROOT / name
        if path.is_symlink() or not path.is_file():
            raise ValueError('input must be a regular nonsymlink file: ' + name)
        result[name] = path.read_bytes()
    return result


def manifest(files):
    return {name: {'bytes': len(data), 'sha256': sha(data)} for name, data in sorted(files.items())}


def create(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('xb') as output:
        output.write(data)
    if path.read_bytes() != data:
        raise RuntimeError('copy mismatch: ' + str(path))


def command(argv, crate, env, folder, label):
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    completed = subprocess.run(argv, cwd=crate, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    log = folder / (label + '.log')
    create(log, completed.stdout)
    return ({'argv': argv, 'started_at': started, 'finished_at': dt.datetime.now(dt.timezone.utc).isoformat(),
             'returncode': completed.returncode, 'log': str(log), 'log_sha256': sha(completed.stdout)},
            completed.stdout.decode('utf-8', errors='replace'))


def test_summary(output):
    pattern = (r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; '
               r'(\d+) measured; (\d+) filtered out;')
    return [dict(zip(('result', 'passed', 'failed', 'ignored', 'measured', 'filtered'), m))
            for m in re.findall(pattern, output)]


def test_assertions(output, crate):
    locations = []
    for path, line in re.findall(r'panicked at ([^\s:]+):(\d+):\d+:', output):
        normalized, line = path.replace('\\', '/'), int(line)
        if normalized.startswith('tests/'):
            locations.append({'path': normalized, 'line': line, 'kind': 'integration-test-assertion'})
        elif normalized.startswith('src/') and (crate / normalized).is_file():
            prefix = '\n'.join((crate / normalized).read_text().splitlines()[:line])
            if '#[cfg(test)]' in prefix:
                locations.append({'path': normalized, 'line': line, 'kind': 'private-test-assertion'})
    return locations


def run_tier(cargo, crate, env, folder, case, tier):
    if tier == 1:
        build = [cargo, 'test', '--offline', '--locked', '--no-run', '--lib']
        runs = [('hand-' + name, ['--test', name]) for name in (*HANDS, *SUPPLEMENTS)]
        runs.append(('private-canadian', ['--lib', 'bonds::canadian::']))
    else:
        build = [cargo, 'test', '--offline', '--locked', '--release', '--no-run']
        runs = [('corpus-' + name, ['--release', '--test', name]) for name in CORPORA]
    for name in ((*HANDS, *SUPPLEMENTS) if tier == 1 else CORPORA):
        build += ['--test', name]
    c, text = command(build, crate, env, folder, 'tier%d-compile' % tier)
    case['commands'].append(c)
    if c['returncode']:
        return 'compile-failure-not-kill', text[-3000:], []
    outputs, assertions = [], []
    for label, selection in runs:
        c, text = command([cargo, 'test', '--offline', '--locked', *selection, '--', '--test-threads=1'],
                          crate, env, folder, 'tier%d-%s' % (tier, label))
        c['test_summaries'] = test_summary(text)
        case['commands'].append(c)
        outputs.append((label, c))
        assertions.extend(test_assertions(text, crate))
    return None, outputs, assertions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='create-new scratch output, outside repository')
    parser.add_argument('--only', action='append', default=[], help='selected mutant id (repeatable)')
    parser.add_argument('--list', action='store_true', help='list mutation domains without running')
    parser.add_argument('--cargo', default='cargo')
    parser.add_argument('--rustc', default='rustc')
    args = parser.parse_args()
    if args.list:
        print(json.dumps([{'id': m[0], 'path': m[1], 'domain': m[2]} for m in MUTANTS], indent=2))
        return
    unknown = set(args.only) - {m[0] for m in MUTANTS}
    if unknown:
        parser.error('unknown mutants: ' + ','.join(sorted(unknown)))
    selected = [m for m in MUTANTS if not args.only or m[0] in args.only]
    destination = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix='sabiquant-canadian-mutants-'))
    if destination == ROOT or ROOT in destination.parents or destination in ROOT.parents:
        parser.error('scratch output must be outside repository and not its ancestor')
    if args.output:
        destination.mkdir(parents=True, exist_ok=False)
    fixture = (ROOT / kernel.FIXTURE).read_bytes()
    if sha(fixture) != kernel.FIXTURE_SHA:
        parser.error('frozen Canadian numeric fixture bytes changed')
    original, runner, harness = snapshot(), Path(__file__).read_bytes(), kernel.HARNESS.read_bytes()
    evidence = {
        'schema': 1, 'started_at': dt.datetime.now(dt.timezone.utc).isoformat(), 'python': sys.version,
        'platform': platform.platform(), 'runner_sha256': sha(runner), 'harness_sha256': sha(harness),
        'numeric_fixture_sha256': kernel.FIXTURE_SHA, 'source_manifest': manifest(original),
        'source_manifest_sha256': sha(canonical(manifest(original))),
        'scope': 'tier1 debug: 29 frozen hands + 1 postproduction public boundary control + private bonds::canadian:: unit tests + full private N '
                 'kernel replay; tier2 release only for tier1 survivors: complete S and P/Y corpora',
        'family_qualification': 'Selected concrete semantic mistakes only, not all72 obligations or a completeness proof',
        'selected_mutants': [m[0] for m in selected], 'full_catalogue': len(selected) == len(MUTANTS),
        'cases': [], 'result': 'incomplete'}
    failure = None
    try:
        cargo, rustc = shutil.which(args.cargo), shutil.which(args.rustc)
        if not cargo or not rustc:
            raise ValueError('matching cargo/rustc required')
        cargo, rustc = str(Path(cargo).absolute()), str(Path(rustc).absolute())
        env = os.environ.copy()
        for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'LLVM_PROFILE_FILE', 'RUSTC_WRAPPER',
                    'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_TARGET'):
            env.pop(key, None)
        env.update(RUSTC=rustc, CARGO_TARGET_DIR=str(destination / 'target'), CARGO_INCREMENTAL='0',
                   RUST_BACKTRACE='1')
        evidence['toolchain'] = {}
        for name, executable, flags in (('cargo', cargo, ['-V']), ('rustc', rustc, ['-Vv'])):
            c, text = command([executable, *flags], ROOT, env, destination, name + '-version')
            if c['returncode']:
                raise RuntimeError(name + ' version failed')
            evidence['toolchain'][name] = {'command': c, 'version': text,
                                           'executable_sha256': sha(Path(executable).read_bytes())}
        crate = destination / 'crate'
        for relative, data in original.items():
            create(crate / relative, data)
        tsv, _ = kernel.transport(fixture)
        for name, data in tsv.items():
            create(crate / name, data)
        # The kernel harness is part of every case, baseline included.
        base = {relative: original[relative] for relative in PRODUCTION}
        base[NUMERIC] = original[NUMERIC] + b'\n' + harness
        for mutation in [None, *selected]:
            name = 'baseline' if mutation is None else mutation[0]
            folder = destination / name
            folder.mkdir()
            for relative, data in base.items():
                (crate / relative).write_bytes(data)
            drift = [r for r, d in original.items() if r not in base and (crate / r).read_bytes() != d]
            if drift:
                raise RuntimeError('scratch crate drifted before %s: %s' % (name, drift))
            case = {'id': name, 'commands': []}
            if mutation:
                _, relative, domain, before, after = mutation
                source = base[relative].decode()
                if source.count(before) != 1:
                    raise ValueError('mutation anchor is not unique: ' + name)
                content = source.replace(before, after, 1).encode()
                (crate / relative).write_bytes(content)
                case.update(domain=domain, mutation={'path': relative, 'before': before, 'after': after,
                                                     'original_sha256': sha(original[relative]),
                                                     'mutated_sha256': sha(content)})
            classification = None
            for tier in (1, 2):
                failed, outputs, assertions = run_tier(cargo, crate, env, folder, case, tier)
                if failed:
                    classification = failed
                    case['compile_output_summary'] = outputs
                    break
                case.setdefault('assertion_locations', []).extend(assertions)
                expected = {**HANDS, **SUPPLEMENTS} if tier == 1 else CORPORA
                summaries = {label: c['test_summaries'] for label, c in outputs}
                if any(len(s) != 1 for s in summaries.values()):
                    raise RuntimeError('incomplete test summaries for %s tier%d' % (name, tier))
                counts = {label: int(s[0]['passed']) for label, s in summaries.items()}
                case['tier%d_passed' % tier] = counts
                if mutation is None:
                    bad = [n for n, want in expected.items()
                           if counts[('hand-' if tier == 1 else 'corpus-') + n] != want]
                    if any(c['returncode'] for _, c in outputs) or bad or (tier == 1 and counts['private-canadian'] < 1):
                        classification = 'baseline-failed'
                        break
                    case['private_canadian_passed'] = counts.get('private-canadian', case.get('private_canadian_passed'))
                    classification = 'baseline-passed'
                    continue
                if any(c['returncode'] for _, c in outputs):
                    classification = ('killed-tier%d-by-test-assertion' % tier) if assertions \
                        else 'runtime-failure-not-assertion-kill'
                    break
                classification = 'survived-all-tiers'
            case['classification'] = classification
            evidence['cases'].append(case)
            print(json.dumps({'case': name, 'classification': classification}), flush=True)
            if mutation is None and classification != 'baseline-passed':
                raise RuntimeError('unmutated baseline must pass both tiers')
        counts = {}
        for c in evidence['cases'][1:]:
            counts[c['classification']] = counts.get(c['classification'], 0) + 1
        evidence['mutation_classifications'] = counts
        evidence['result'] = 'completed'
    except Exception as error:
        failure = error
        evidence['result'] = 'failed'
        evidence['failure'] = {'type': type(error).__name__, 'message': str(error)}
    evidence['original_guard'] = {
        'source_unchanged': all((ROOT / p).read_bytes() == d for p, d in original.items()
                                if p not in ('README.md', 'CHANGELOG.md')),
        'fixture_unchanged': (ROOT / kernel.FIXTURE).read_bytes() == fixture,
        'runner_unchanged': Path(__file__).read_bytes() == runner,
        'harness_unchanged': kernel.HARNESS.read_bytes() == harness}
    if not all(evidence['original_guard'].values()):
        evidence['result'] = 'invalidated-original-input-changed'
        failure = RuntimeError('original sources/frozen inputs/runner changed during run')
    evidence['finished_at'] = dt.datetime.now(dt.timezone.utc).isoformat()
    report = destination / 'evidence.json'
    create(report, (json.dumps(evidence, indent=2, sort_keys=True) + '\n').encode())
    print(json.dumps({'result': evidence['result'], 'evidence': str(report), 'evidence_sha256': sha(report.read_bytes()),
                      'mutations': evidence.get('mutation_classifications', {})}), flush=True)
    if failure:
        raise SystemExit(str(failure))


if __name__ == '__main__':
    main()
