#!/usr/bin/env python3
"""Compile every frozen Canadian negative API probe against the actual library.

FR-CAD-10: each of the 92 sources in
validation/hand/canadian_api_negative_obligations.json is compiled as its own
external crate against a fresh rlib of the unchanged repository. A probe
passes only when rustc exits nonzero with coded E0277 errors alone that name
the intended public type and forbidden trait; a missing type, import or any
other diagnostic fails the run. Sources, the obligations, the trait matrix,
the API hand and the rlib are byte-guarded before and after. Stdlib only.
"""

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
OBLIGATIONS = Path('validation/hand/canadian_api_negative_obligations.json')
GUARDED = (Path('Cargo.toml'), Path('Cargo.lock'), OBLIGATIONS,
           Path('validation/hand/canadian_api_trait_matrix.json'), Path('tests/bonds_canadian_api_hand.rs'))
PROBES = 92


def sha(data):
    return hashlib.sha256(data).hexdigest()


def snapshot():
    paths = sorted((ROOT / 'src').rglob('*.rs')) + [ROOT / p for p in GUARDED]
    return {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in paths}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='new evidence directory outside the repository')
    parser.add_argument('--cargo', default='cargo')
    parser.add_argument('--rustc', default='rustc')
    args = parser.parse_args()
    if args.output:
        out = args.output.resolve()
        if out == ROOT or ROOT in out.parents:
            parser.error('evidence output must be outside the original repository')
        out.mkdir(parents=True, exist_ok=False)
    else:
        out = Path(tempfile.mkdtemp(prefix='sabiquant-canadian-api-negative-'))
    cargo, rustc = shutil.which(args.cargo), shutil.which(args.rustc)
    if not cargo or not rustc:
        parser.error('matching cargo/rustc required')
    # Keep the invoked filename: rustup proxies dispatch on argv[0].
    cargo, rustc = str(Path(cargo).absolute()), str(Path(rustc).absolute())
    initial = snapshot()
    env = os.environ.copy()
    for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                'CARGO_BUILD_TARGET', 'LLVM_PROFILE_FILE'):
        env.pop(key, None)
    target = out / 'target'
    env.update(RUSTC=rustc, CARGO_TARGET_DIR=str(target))
    build = [cargo, 'build', '--offline', '--locked', '--lib']
    result = subprocess.run(build, cwd=ROOT, env=env, capture_output=True)
    (out / 'build.log').write_bytes(result.stdout + result.stderr)
    if result.returncode:
        raise SystemExit('library build failed; see ' + str(out / 'build.log'))
    libs = list((target / 'debug/deps').glob('libsabiquant-*.rlib'))
    if len(libs) != 1:
        raise SystemExit('expected exactly one sabiquant rlib')
    lib = libs[0]
    lib_sha = sha(lib.read_bytes())
    probes = json.loads((ROOT / OBLIGATIONS).read_bytes())
    if len(probes['probes']) != probes['count'] or probes['count'] != PROBES:
        raise SystemExit('frozen negative obligation census changed')
    checks = []
    for probe in probes['probes']:
        folder = out / 'cases' / probe['id']
        folder.mkdir(parents=True)
        source = folder / 'probe.rs'
        source.write_text(probe['source'])
        argv = [rustc, '--edition=2024', '--error-format=json', '--crate-name', 'canadian_negative_probe',
                '--emit=metadata', '--extern', 'sabiquant=' + str(lib), '-L', 'dependency=' + str(target / 'debug/deps'),
                str(source), '--out-dir', str(folder)]
        r = subprocess.run(argv, capture_output=True)
        (folder / 'stderr.jsonl').write_bytes(r.stderr)
        diagnostics = [json.loads(line) for line in r.stderr.decode().splitlines() if line.startswith('{')]
        errors = [x for x in diagnostics if x.get('level') == 'error' and x.get('code') is not None]
        uncoded = [x for x in diagnostics if x.get('level') == 'error' and x.get('code') is None
                   and not x.get('message', '').startswith('aborting due to')]
        if r.returncode == 0 or not errors or uncoded or any(x['code']['code'] != 'E0277' for x in errors):
            raise SystemExit('probe %s did not fail with E0277 alone' % probe['id'])
        rendered = '\n'.join(x.get('message', '') + ' ' + (x.get('rendered') or '') for x in errors)
        name, bound = probe['type'].split('<')[0], probe['forbidden_bound']
        trait = 'From' if bound.startswith('From<') else bound.rsplit('::', 1)[-1]
        if name not in rendered or trait not in rendered:
            raise SystemExit('probe %s diagnostic does not name %s/%s' % (probe['id'], name, trait))
        if bound.startswith('From<') and ('DecimalAmount' if 'DecimalAmount' in bound else 'f64') not in rendered:
            raise SystemExit('probe %s diagnostic does not name the refused conversion' % probe['id'])
        checks.append({'id': probe['id'], 'type': probe['type'], 'forbidden_bound': bound,
                       'exit_status': r.returncode, 'coded_error_count': len(errors),
                       'source_sha256': sha(source.read_bytes()), 'stderr_sha256': sha(r.stderr)})
    if sha(lib.read_bytes()) != lib_sha:
        raise SystemExit('rlib changed during probes')
    if snapshot() != initial:
        raise SystemExit('consumed repository source changed during probes')
    evidence = {'status': 'PASS_ALL92_INTENDED_NEGATIVE_TRAIT_DIAGNOSTICS',
                'finished_at': dt.datetime.now(dt.timezone.utc).isoformat(), 'build_command': build,
                'rlib_sha256': lib_sha, 'source_guards': initial, 'probes': checks, 'probe_count': len(checks),
                'rustc_version': subprocess.check_output([rustc, '-Vv'], text=True),
                'qualification': 'Separate external compilations fail only with coded E0277 naming the '
                                 'intended public type/trait. API negative diagnostics only, not financial '
                                 'correctness, resource or whole-suite acceptance.'}
    report = out / 'evidence.json'
    report.write_text(json.dumps(evidence, sort_keys=True, indent=2) + '\n')
    print(json.dumps({'status': evidence['status'], 'evidence': str(report), 'sha256': sha(report.read_bytes())}))


if __name__ == '__main__':
    main()
