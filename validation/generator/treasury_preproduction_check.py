#!/usr/bin/env python3
"""Attest independent Treasury corpora before production; no financial oracle.

Python stdlib only. Compiles the unchanged shared Rust fixture reader, checks
actual generator/contract/requirement/body/input pins and censuses, optionally
reproduces each independent generator, and writes create-only local evidence.
This checks artifact integrity, not financial accuracy or Treasury integration.
"""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = (
    'validation/fixtures/bonds/treasury-source-cash-oracle.fixture',
    'validation/fixtures/bonds/treasury-numeric-oracle.fixture',
    'validation/fixtures/bonds/treasury-ideal-price-risk-oracle.fixture',
    'validation/fixtures/bonds/treasury-ideal-inverse-oracle.fixture',
)
GENERATORS = {
    FIXTURES[0]: 'validation/generator/treasury_source_cash_oracles.py',
    FIXTURES[1]: 'validation/generator/treasury_numeric_oracles.py',
    FIXTURES[2]: 'validation/generator/treasury_ideal_oracles.py',
    FIXTURES[3]: 'validation/generator/treasury_ideal_oracles.py',
}
MAIN = 'docs/M8_US_TREASURY_CONTRACT.md'
CASH = 'docs/M8_US_TREASURY_CASH_CONTRACT.md'
CONTRACT_HASHES = {
    MAIN: '50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93',
    CASH: '25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585',
}
# Fail closed until root reviews each author's finalized schema and exact semantic
# input list. Fixture-supplied input-columns alone cannot establish uniqueness.
# Schema approval binds encoding/input identity, not financial accuracy.
# The two ideal schemas are independently inspected; other families still fail closed.
APPROVED_SCHEMAS = {'validation/fixtures/bonds/treasury-ideal-price-risk-oracle.fixture': {'columns': ('id',
                                                                                    'family',
                                                                                    'dated',
                                                                                    'issue',
                                                                                    'maturity',
                                                                                    'coupon',
                                                                                    'first_coupon',
                                                                                    'eom',
                                                                                    'quasi_count',
                                                                                    'settlement',
                                                                                    'daily_basis',
                                                                                    'tie_rule',
                                                                                    'first_period',
                                                                                    'next_index',
                                                                                    'r',
                                                                                    's',
                                                                                    'n',
                                                                                    'raw_ai',
                                                                                    'published_ai',
                                                                                    'coupon_indices',
                                                                                    'source_amounts',
                                                                                    'positive_pv_count',
                                                                                    'source_display',
                                                                                    'requirements',
                                                                                    'yield_bits',
                                                                                    'input_fingerprint',
                                                                                    'pv_gate',
                                                                                    'comparison_method',
                                                                                    'ai_cache_error',
                                                                                    'dirty_radius',
                                                                                    'delta_ideal',
                                                                                    'prefix_ideal',
                                                                                    'delta_rn_bits',
                                                                                    'prefix_rn_bits',
                                                                                    'dirty_ideal',
                                                                                    'dirty_ideal_rn_bits',
                                                                                    'dirty_ideal_class',
                                                                                    'dirty_allowance',
                                                                                    'dirty_comparison',
                                                                                    'clean_ideal',
                                                                                    'clean_ideal_rn_bits',
                                                                                    'clean_ideal_class',
                                                                                    'clean_allowance',
                                                                                    'clean_comparison',
                                                                                    'd1_ideal',
                                                                                    'd1_ideal_rn_bits',
                                                                                    'd1_ideal_class',
                                                                                    'd1_allowance',
                                                                                    'd1_comparison',
                                                                                    'd2_ideal',
                                                                                    'd2_ideal_rn_bits',
                                                                                    'd2_ideal_class',
                                                                                    'd2_allowance',
                                                                                    'd2_comparison',
                                                                                    'moment_ideal',
                                                                                    'moment_ideal_rn_bits',
                                                                                    'moment_ideal_class',
                                                                                    'moment_allowance',
                                                                                    'moment_comparison',
                                                                                    'modified_ideal',
                                                                                    'modified_ideal_rn_bits',
                                                                                    'modified_ideal_class',
                                                                                    'modified_allowance',
                                                                                    'modified_comparison',
                                                                                    'macaulay_ideal',
                                                                                    'macaulay_ideal_rn_bits',
                                                                                    'macaulay_ideal_class',
                                                                                    'macaulay_allowance',
                                                                                    'macaulay_comparison',
                                                                                    'convexity_ideal',
                                                                                    'convexity_ideal_rn_bits',
                                                                                    'convexity_ideal_class',
                                                                                    'convexity_allowance',
                                                                                    'convexity_comparison'),
                                                                        'profile': 'family',
                                                                        'inputs': ('dated',
                                                                                   'issue',
                                                                                   'maturity',
                                                                                   'coupon',
                                                                                   'first_coupon',
                                                                                   'eom',
                                                                                   'quasi_count',
                                                                                   'settlement',
                                                                                   'daily_basis',
                                                                                   'tie_rule',
                                                                                   'yield_bits'),
                                                                        'metadata': {'fixture-set': 'M8-US-TREASURY-IDEAL-PRICE-RISK',
                                                                                     'domain': 'bonds',
                                                                                     'description': 'Independent '
                                                                                                    'ideal '
                                                                                                    'references; '
                                                                                                    'no '
                                                                                                    'Treasury '
                                                                                                    'implementation '
                                                                                                    'or '
                                                                                                    'prescribed-stage '
                                                                                                    'expectations '
                                                                                                    'read',
                                                                                     'reference': 'Exact '
                                                                                                  'Fraction '
                                                                                                  'finite-geometric '
                                                                                                  'product '
                                                                                                  'derivatives; '
                                                                                                  'independent '
                                                                                                  'Decimal160/240 '
                                                                                                  'per-payment '
                                                                                                  'sums',
                                                                                     'reference-version': 'Python '
                                                                                                          'standard '
                                                                                                          'library',
                                                                                     'reference-source': 'docs/M8_US_TREASURY_CONTRACT.md '
                                                                                                         '+ '
                                                                                                         'docs/M8_US_TREASURY_CASH_CONTRACT.md',
                                                                                     'tolerance': 'TOL-UST-RATIONAL '
                                                                                                  '/ '
                                                                                                  'TOL-UST-INVERSE '
                                                                                                  'conditional '
                                                                                                  'absolute '
                                                                                                  'bounds; '
                                                                                                  'ideal '
                                                                                                  'bits '
                                                                                                  'are '
                                                                                                  'not '
                                                                                                  'E0 '
                                                                                                  'observed '
                                                                                                  'outputs',
                                                                                     'reproduction': 'python3 '
                                                                                                     '-I '
                                                                                                     'validation/generator/treasury_ideal_oracles.py '
                                                                                                     '--check'}},
 'validation/fixtures/bonds/treasury-ideal-inverse-oracle.fixture': {'columns': ('id',
                                                                                 'family',
                                                                                 'dated',
                                                                                 'issue',
                                                                                 'maturity',
                                                                                 'coupon',
                                                                                 'first_coupon',
                                                                                 'eom',
                                                                                 'quasi_count',
                                                                                 'settlement',
                                                                                 'daily_basis',
                                                                                 'tie_rule',
                                                                                 'first_period',
                                                                                 'next_index',
                                                                                 'r',
                                                                                 's',
                                                                                 'n',
                                                                                 'raw_ai',
                                                                                 'published_ai',
                                                                                 'coupon_indices',
                                                                                 'source_amounts',
                                                                                 'positive_pv_count',
                                                                                 'source_display',
                                                                                 'requirements',
                                                                                 'target_kind',
                                                                                 'target_bits',
                                                                                 'dirty_target_bits',
                                                                                 'ideal_dirty_target',
                                                                                 'caller_lo_bits',
                                                                                 'caller_hi_bits',
                                                                                 'abs_tol_bits',
                                                                                 'rel_tol_bits',
                                                                                 'max_evals',
                                                                                 'input_fingerprint',
                                                                                 'ideal_image',
                                                                                 'inverse_method',
                                                                                 'root_lo',
                                                                                 'root_hi',
                                                                                 'root_lo_bits',
                                                                                 'root_hi_bits',
                                                                                 'left_sign',
                                                                                 'right_sign',
                                                                                 'bisections',
                                                                                 'uniform_price_error',
                                                                                 'target_shift_error',
                                                                                 'slope_lower',
                                                                                 'coordinate_allowance',
                                                                                 'conditioned_lo_bits',
                                                                                 'conditioned_hi_bits',
                                                                                 'comparison',
                                                                                 'residual_scale'),
                                                                     'profile': 'family',
                                                                     'inputs': ('dated',
                                                                                'issue',
                                                                                'maturity',
                                                                                'coupon',
                                                                                'first_coupon',
                                                                                'eom',
                                                                                'quasi_count',
                                                                                'settlement',
                                                                                'daily_basis',
                                                                                'tie_rule',
                                                                                'target_kind',
                                                                                'target_bits',
                                                                                'caller_lo_bits',
                                                                                'caller_hi_bits',
                                                                                'abs_tol_bits',
                                                                                'rel_tol_bits',
                                                                                'max_evals'),
                                                                     'metadata': {'fixture-set': 'M8-US-TREASURY-IDEAL-INVERSE',
                                                                                  'domain': 'bonds',
                                                                                  'description': 'Independent '
                                                                                                 'ideal '
                                                                                                 'references; '
                                                                                                 'no '
                                                                                                 'Treasury '
                                                                                                 'implementation '
                                                                                                 'or '
                                                                                                 'prescribed-stage '
                                                                                                 'expectations '
                                                                                                 'read',
                                                                                  'reference': 'Exact '
                                                                                               'Fraction '
                                                                                               'finite-geometric '
                                                                                               'product '
                                                                                               'derivatives; '
                                                                                               'independent '
                                                                                               'Decimal160/240 '
                                                                                               'per-payment '
                                                                                               'sums',
                                                                                  'reference-version': 'Python '
                                                                                                       'standard '
                                                                                                       'library',
                                                                                  'reference-source': 'docs/M8_US_TREASURY_CONTRACT.md '
                                                                                                      '+ '
                                                                                                      'docs/M8_US_TREASURY_CASH_CONTRACT.md',
                                                                                  'tolerance': 'TOL-UST-RATIONAL '
                                                                                               '/ '
                                                                                               'TOL-UST-INVERSE '
                                                                                               'conditional '
                                                                                               'absolute '
                                                                                               'bounds; '
                                                                                               'ideal '
                                                                                               'bits '
                                                                                               'are '
                                                                                               'not '
                                                                                               'E0 '
                                                                                               'observed '
                                                                                               'outputs',
                                                                                  'reproduction': 'python3 '
                                                                                                  '-I '
                                                                                                  'validation/generator/treasury_ideal_oracles.py '
                                                                                                  '--check'}}}
ADOPTED_REQUIREMENTS_SHA256 = '722e090018bff9bbe25a2574f778766c06068c832dee6db9fe90041e1bcafeb3'

REQUIRED = ('schema', 'fixture-set', 'domain', 'description', 'reference',
            'reference-version', 'reference-source', 'tolerance', 'reproduction',
            'generated-at', 'platform')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fnv(data):
    result = 0xcbf29ce484222325
    for value in data:
        result = ((result ^ value) * 0x100000001b3) & ((1 << 64) - 1)
    return '%016x' % result


def strict_json(text):
    def object_pairs(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, 'duplicate JSON key: ' + key)
            result[key] = value
        return result
    def reject_constant(value):
        raise ValueError('nonstandard JSON constant: ' + value)
    return json.loads(text, object_pairs_hook=object_pairs, parse_constant=reject_constant)


class CommandFailed(ValueError):
    def __init__(self, result):
        super().__init__('command failed; see ' + result['log'])
        self.result = result


def requirement_rows(root):
    rows = [line for line in (root / 'docs/REQUIREMENTS.md').read_text().splitlines()
            if re.match(r'^\| FR-UST-\d+ \|', line)]
    require([line.split('|')[1].strip() for line in rows] ==
            ['FR-UST-%02d' % i for i in range(1, 27)], 'FR-UST row census/order')
    data = ('\n'.join(rows) + '\n').encode()
    require(sha(data) == ADOPTED_REQUIREMENTS_SHA256, 'adopted requirement rows changed')
    return data


def read_table(data):
    """Extract canonical LF artifact bytes after shared-reader acceptance."""
    require(b'\r' not in data, 'attested artifact must use canonical LF')
    require(data.endswith(b'\n'), 'fixture must end in LF')
    text = data.decode('utf-8')
    require('\n\n' in text, 'header/table separator')
    raw_header, body = text.split('\n\n', 1)
    header = {}
    for line in raw_header.splitlines():
        key, separator, value = line.partition(': ')
        require(bool(separator) and bool(re.fullmatch(r'[a-z][a-z0-9-]*', key)),
                'malformed header key')
        require(key not in header, 'duplicate header key: ' + key)
        require(bool(value), 'empty header value: ' + key)
        header[key] = value
    require(all(k in header for k in REQUIRED), 'missing required schema header')
    require(header['schema'] == '1', 'unsupported fixture schema')
    lines = body.splitlines()
    require(len(lines) >= 2, 'fixture needs columns and records')
    columns = lines[0].split('\t')
    require(columns[0] == 'id' and len(columns) == len(set(columns)),
            'invalid id/duplicate columns')
    require(all(re.fullmatch(r'[a-z][a-z0-9_]*', c) for c in columns),
            'malformed columns')
    records = [line.split('\t') for line in lines[1:]]
    require(all(len(row) == len(columns) for row in records), 'record width')
    identifiers = [row[0] for row in records]
    require(all(identifiers) and len(set(identifiers)) == len(identifiers),
            'missing/duplicate record IDs')
    for row in records:
        for field in row:
            require(not any(ord(c) < 32 or ord(c) == 127 for c in field),
                    'record control character')
    return header, columns, records, body.encode(), ('\n'.join(lines[1:]) + '\n').encode()


def check_pair(header, prefix, data):
    for suffix, value in (('sha256', sha(data)), ('fnv64', fnv(data))):
        key = prefix + '-' + suffix
        require(header.get(key) == value, 'pin mismatch: ' + key)


def attest_fixture(root, relative):
    require(relative in GENERATORS, 'unregistered Treasury corpus: ' + relative)
    path = root / relative
    raw = path.read_bytes()
    header, columns, records, body, record_bytes = read_table(raw)
    require(relative in APPROVED_SCHEMAS, 'corpus schema not reviewed: ' + relative)
    schema = APPROVED_SCHEMAS[relative]
    require(tuple(columns) == tuple(schema['columns']), 'unapproved corpus columns')
    require(bool(schema['metadata']), 'reviewed metadata identity required')
    for key, expected in schema['metadata'].items():
        require(header.get(key) == expected, 'unapproved metadata identity: ' + key)
    for contract, expected in CONTRACT_HASHES.items():
        require(sha((root / contract).read_bytes()) == expected,
                'adopted contract changed: ' + contract)
    generator = GENERATORS[relative]
    check_pair(header, 'generator', (root / generator).read_bytes())
    check_pair(header, 'adopted-contract', (root / MAIN).read_bytes())
    check_pair(header, 'cash-contract', (root / CASH).read_bytes())
    check_pair(header, 'requirements-rows', requirement_rows(root))
    check_pair(header, 'body', body)
    check_pair(header, 'records', record_bytes)
    require(header.get('record-count') == str(len(records)), 'record-count')
    require(header.get('column-count') == str(len(columns)), 'column-count')
    profile_column = schema['profile']
    require(profile_column in columns, 'no profile/family column')
    profile_counts = Counter(row[columns.index(profile_column)] for row in records)
    require(all(profile_counts), 'empty profile name')
    declared_profiles = strict_json(header['profile-counts'])
    require(isinstance(declared_profiles, dict) and
            all(type(v) is int and v > 0 for v in declared_profiles.values()),
            'profile-counts must be a positive integer JSON object')
    require(dict(profile_counts) == declared_profiles, 'profile-counts census')
    input_columns = header['input-columns'].split(',')
    require(tuple(input_columns) == tuple(schema['inputs']), 'unapproved input-columns')
    require(input_columns and len(input_columns) == len(set(input_columns)) and
            all(c in columns for c in input_columns) and
            'id' not in input_columns and profile_column not in input_columns,
            'input-columns must name unique actual input fields excluding id/profile')
    positions = [columns.index(c) for c in input_columns]
    input_rows = ['\t'.join(row[i] for i in positions) for row in records]
    unique_count = len(set(input_rows))
    require(header.get('unique-input-count') == str(unique_count), 'unique-input-count')
    require(unique_count == len(records), 'duplicate semantic input rows')
    check_pair(header, 'inputs', ('\n'.join(input_rows) + '\n').encode())
    stamp = datetime.fromisoformat(header['generated-at'].replace('Z', '+00:00'))
    require(stamp.tzinfo is not None and stamp.utcoffset().total_seconds() == 0,
            'generated-at must be aware UTC')
    require(stamp <= datetime.now(timezone.utc), 'generated-at in future')
    return {'path': relative, 'sha256': sha(raw), 'rows': len(records),
            'columns': len(columns), 'profiles': dict(sorted(profile_counts.items())),
            'unique_inputs': unique_count, 'input_columns': input_columns,
            'generator': generator, 'generator_sha256': sha((root / generator).read_bytes()),
            'generated_at_utc': header['generated-at']}


def tracked_snapshot(root):
    output = subprocess.check_output(['git', 'ls-files', '-z'], cwd=root)
    return {p.decode(): sha((root / p.decode()).read_bytes())
            for p in output.split(b'\0') if p}


def run_logged(args, cwd, output_dir, name, env=None):
    completed = subprocess.run(args, cwd=cwd, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, check=False)
    log = output_dir / (name + '.log')
    with log.open('xb') as stream:
        stream.write(completed.stdout)
    result = {'command': [str(a) for a in args], 'exit_code': completed.returncode,
              'log': str(log), 'log_sha256': sha(completed.stdout)}
    if completed.returncode != 0:
        raise CommandFailed(result)
    return result


def compile_reader(root, output_dir):
    """Use a path module: no edited/reimplemented Rust parser or library."""
    source_path = root / 'tests/support/fixture.rs'
    module_path = json.dumps(str(source_path))
    harness = '''#![allow(dead_code, unreachable_pub)]
#[path = MODULE_PATH]
mod fixture;
fn main() {
    let mut total = 0usize;
    for path in std::env::args().skip(1) {
        let fixture = fixture::Fixture::load(&path).expect("shared fixture reader");
        if let Some(expected) = fixture.header("record-count") {
            assert_eq!(fixture.record_count().to_string(), expected, "{path}");
        }
        if let Some(expected) = fixture.header("column-count") {
            assert_eq!(fixture.columns().len().to_string(), expected, "{path}");
        }
        println!("{}\\t{}\\t{}", path, fixture.record_count(), fixture.columns().len());
        total += fixture.record_count();
    }
    println!("TOTAL\\t{total}");
}
'''.replace('MODULE_PATH', module_path)
    driver = output_dir / 'shared-reader.rs'
    with driver.open('x') as stream:
        stream.write(harness)
    executable = output_dir / 'shared-reader'
    env = dict(os.environ, CARGO_MANIFEST_DIR=str(root))
    compiled = run_logged(['rustc', '--edition=2024', '-Dwarnings', str(driver),
                           '-o', str(executable)], root, output_dir, 'reader-compile', env)
    return executable, {'unchanged_reader_path': str(source_path),
                        'unchanged_reader_sha256': sha(source_path.read_bytes()),
                        'harness_sha256': sha(driver.read_bytes()), 'compile': compiled}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', required=True,
                        help='new local evidence directory; existing paths refused')
    parser.add_argument('--fixtures', nargs='+', choices=FIXTURES, default=list(FIXTURES))
    parser.add_argument('--skip-reproduction', action='store_true',
                        help='integrity-only interim check; cannot close preproduction gate')
    parser.add_argument('--all-fixtures', action='store_true',
                        help='also parse every earlier fixture through the unchanged reader')
    args = parser.parse_args(argv)
    require(len(set(args.fixtures)) == len(args.fixtures), 'repeated fixture argument')
    for relative in args.fixtures:
        require((ROOT / relative).is_file(), 'missing corpus: ' + relative)
    output_dir = Path(args.output_dir).resolve()
    require(not output_dir.exists(), 'output directory exists: ' + str(output_dir))
    require(not output_dir.is_relative_to(ROOT), 'evidence output must be outside repository')
    output_dir.mkdir(parents=True, exist_ok=False)
    baseline = tracked_snapshot(ROOT)
    protected_paths = set(args.fixtures) | {GENERATORS[p] for p in args.fixtures}
    protected_paths.add(str(Path(__file__).relative_to(ROOT)))
    protected_before = {p: sha((ROOT / p).read_bytes()) for p in protected_paths}
    report = {'generated_at_utc': datetime.now(timezone.utc).isoformat(),
              'scope': 'artifact provenance, independent regeneration and unchanged shared-reader acceptance; no Treasury runtime/accuracy pass',
              'complete_scope': set(args.fixtures) == set(FIXTURES) and not args.skip_reproduction,
              'platform': platform.platform(), 'python': platform.python_version(),
              'checker_sha256': sha(Path(__file__).read_bytes()), 'status': 'FAIL'}
    code = 1
    try:
        report['rustc'] = subprocess.check_output(['rustc', '--version'], text=True).strip()
        report['commit'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
        report['corpora'] = [attest_fixture(ROOT, p) for p in args.fixtures]
        generators = list(dict.fromkeys(GENERATORS[p] for p in args.fixtures))
        report['reproduction'] = []
        fixture_before = {p: sha((ROOT / p).read_bytes()) for p in args.fixtures}
        if not args.skip_reproduction:
            for index, generator in enumerate(generators):
                report['reproduction'].append(run_logged(
                    [sys.executable, '-I', str(ROOT / generator), '--check'], ROOT,
                    output_dir, 'generator-%d' % index))
        require(fixture_before == {p: sha((ROOT / p).read_bytes()) for p in args.fixtures},
                'fixture bytes changed during reproduction')
        executable, reader = compile_reader(ROOT, output_dir)
        paths = sorted(ROOT.glob('validation/fixtures/**/*.fixture')) if args.all_fixtures else [ROOT / p for p in args.fixtures]
        reader['execution'] = run_logged([str(executable)] + [str(p) for p in paths],
                                         ROOT, output_dir, 'reader-execution')
        reader['fixture_count'] = len(paths)
        report['shared_reader'] = reader
        require(baseline == tracked_snapshot(ROOT), 'tracked inputs changed during check')
        require(protected_before == {p: sha((ROOT / p).read_bytes()) for p in protected_paths},
                'new corpus/generator/checker bytes changed during entire check')
        report['new_artifact_guard'] = True
        report['protected_artifact_hashes'] = dict(sorted(protected_before.items()))
        report['tracked_input_count'] = len(baseline)
        report['tracked_input_guard'] = True
        report['total_new_rows'] = sum(item['rows'] for item in report['corpora'])
        report['status'] = 'PASS'
        code = 0
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        report['failure'] = str(error)
        if isinstance(error, CommandFailed):
            report['failed_command'] = error.result
    report['finished_at_utc'] = datetime.now(timezone.utc).isoformat()
    report_path = output_dir / 'evidence.json'
    with report_path.open('x') as stream:
        json.dump(report, stream, indent=2, allow_nan=False)
        stream.write('\n')
    print(json.dumps({'status': report['status'], 'complete_scope': report['complete_scope'],
                      'evidence': str(report_path), 'sha256': sha(report_path.read_bytes())}))
    return code


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1)
