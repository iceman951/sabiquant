#!/usr/bin/env python3
"""Reproduce frozen extended yield fixtures against their historical policy.

The original generators and fixtures remain unchanged. A create-new external
minimal tree places the checked historical policy at the original policy path,
then invokes both old generators with --check. This does not assert that their
old direct commands reproduce metadata against today's living policy document.
Only Python's standard library is used; production source is not staged/read.
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
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
RUNNER = Path(__file__).resolve()
RISK_GENERATOR = 'validation/generator/bond_yield_risk_extended_oracles.py'
INVERSE_GENERATOR = 'validation/generator/bond_yield_inverse_extended_oracles.py'
GENERATORS = (RISK_GENERATOR, INVERSE_GENERATOR)
FIXTURES = (
    'validation/fixtures/bonds/yield-risk-extended.fixture',
    'validation/fixtures/bonds/yield-risk-extended-public.fixture',
    'validation/fixtures/bonds/yield-inverse-extended.fixture',
)
CONTRACTS = (
    'docs/M8_BOND_YIELD_RISK_CONTRACT.md', 'docs/M8_BOND_CORE_CONTRACT.md',
    'docs/M4_DATE_CONTRACT.md', 'docs/M3_ICMA_CONTRACT.md',
    'docs/M3_US_AFB_CONTRACT.md', 'docs/M3_DAYCOUNT_EXTENSIONS.md',
    'docs/M3_TARGET_CONTRACT.md', 'docs/M6_SOLVER_RESEARCH.md',
    'docs/NUMERICAL_CORRECTNESS.md',
)
LIVE_POLICY = 'docs/NUMERICAL_CORRECTNESS.md'
HISTORICAL_POLICY = 'docs/NUMERICAL_CORRECTNESS_YIELD_RISK_REFERENCE.md'
REQUIREMENTS = 'docs/REQUIREMENTS.md'
PINNED = {
    RISK_GENERATOR: '9c5e77b68dbd9c2212b4ae0fb2cd7cdbc9b420505a55d29c01afa0b559541b85',
    INVERSE_GENERATOR: '70723437953a120d18e92bc608a9b2a61112270a3b600b25f01b44a36a4ff258',
    FIXTURES[0]: '7c36d7bb9b04ad62295205f209ae7f7c6527b158a60ef7f7f1326945a25fba13',
    FIXTURES[1]: '095d4d138be98a71b5c3b89307afcc529a63360e8d31e948d051c335886655d3',
    FIXTURES[2]: '00c710b1fa82805f23cb8ab9d422d7d5f83df5288a9dbb7039e4db9598dd547b',
    HISTORICAL_POLICY: '360bc4cfe69d487cc7a94ba694ee791145530053eff7ee014a54db24a420c061',
}
REQUIREMENTS_SHA256 = 'eaecaec76703c6e7195684cf2f17fe93eec3ed8f00343361676f839379a24e08'
REQUIREMENTS_FNV64 = 'a6273d939267f859'
POLICY_FNV64 = '1df1aeaa0859744a'
EXPECTED_ROWS = (20_201, 2_181, 992)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fnv(data: bytes) -> str:
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{value:016x}'


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def read_regular(path: Path) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError('regular nonsymlink file required: ' + str(path))
    return path.read_bytes()


def manifest(files: dict[str, bytes]) -> list[dict]:
    return [dict(path=name, bytes=len(data), sha256=sha(data))
            for name, data in sorted(files.items())]


def fixture_header(data: bytes) -> tuple[dict[str, str], bytes]:
    header, separator, body = data.partition(b'\n\n')
    if not separator:
        raise ValueError('fixture header separator missing')
    fields = {}
    for line in header.decode('utf-8').splitlines():
        key, value = line.split(': ', 1)
        if key in fields or not re.fullmatch('[a-z][a-z0-9-]*', key) or not value:
            raise ValueError('duplicate or invalid fixture metadata')
        fields[key] = value
    return fields, body


def validate_inputs(original: dict[str, bytes]) -> list[dict]:
    for name, expected in PINNED.items():
        if sha(original[name]) != expected:
            raise ValueError('frozen input pin mismatch: ' + name)
    if fnv(original[HISTORICAL_POLICY]) != POLICY_FNV64:
        raise ValueError('historical policy FNV mismatch')
    requirement_rows = ''.join(line + '\n' for line in original[REQUIREMENTS].decode('utf-8').splitlines()
                               if line.startswith('| FR-YR-')).encode('utf-8')
    if sha(requirement_rows) != REQUIREMENTS_SHA256 or fnv(requirement_rows) != REQUIREMENTS_FNV64:
        raise ValueError('frozen FR-YR requirements projection changed')
    result = []
    for index, name in enumerate(FIXTURES):
        fields, body = fixture_header(original[name])
        _, newline, records = body.partition(b'\n')
        if fields['schema'] != '1' or not newline or int(fields['record-count']) != EXPECTED_ROWS[index]:
            raise ValueError('frozen fixture schema/count mismatch: ' + name)
        if len(records.splitlines()) != EXPECTED_ROWS[index]:
            raise ValueError('actual frozen row count mismatch: ' + name)
        for key, actual in (('body-sha256', sha(body)), ('body-fnv64', fnv(body)), ('records-sha256', sha(records))):
            if fields[key] != actual:
                raise ValueError('fixture integrity mismatch: ' + name + ':' + key)
        generator = INVERSE_GENERATOR if index == 2 else RISK_GENERATOR
        for suffix, actual in (('sha256', sha(original[generator])), ('fnv64', fnv(original[generator]))):
            if fields['generator-' + suffix] != actual:
                raise ValueError('fixture generator mismatch: ' + name)
        if index == 2 and (fields['helper-sha256'] != sha(original[RISK_GENERATOR])
                           or fields['helper-fnv64'] != fnv(original[RISK_GENERATOR])):
            raise ValueError('inverse frozen helper mismatch')
        for contract in CONTRACTS:
            key = Path(contract).stem.lower().replace('_', '-')
            data = original[HISTORICAL_POLICY if contract == LIVE_POLICY else contract]
            if fields['contract-' + key + '-sha256'] != sha(data) or fields['contract-' + key + '-fnv64'] != fnv(data):
                raise ValueError('frozen contract dependency mismatch: ' + name + ':' + contract)
        if fields['requirements-sha256'] != sha(requirement_rows) or fields['requirements-fnv64'] != fnv(requirement_rows):
            raise ValueError('fixture FR-YR requirements mismatch')
        result.append(dict(path=name, bytes=len(original[name]), sha256=sha(original[name]),
                           fnv64=fnv(original[name]), record_count=EXPECTED_ROWS[index],
                           body_sha256=sha(body), records_sha256=sha(records)))
    return result


def guard(original: dict[str, bytes], runner: bytes, tree: Path,
          staged: dict[str, bytes]) -> dict:
    changed_original = []
    for name, data in original.items():
        try:
            unchanged = read_regular(ROOT / name) == data
        except (OSError, ValueError):
            unchanged = False
        if not unchanged:
            changed_original.append(name)
    runner_unchanged = read_regular(RUNNER) == runner
    current = {}
    if tree.exists():
        for path in tree.rglob('*'):
            if path.is_file() or path.is_symlink():
                current[str(path.relative_to(tree))] = read_regular(path)
    staged_changes = sorted(name for name in set(staged) | set(current) if staged.get(name) != current.get(name))
    return dict(original_inputs_unchanged=not changed_original, changed_original_paths=changed_original,
                runner_unchanged=runner_unchanged, staged_tree_unchanged=not staged_changes,
                changed_staged_paths=staged_changes)


def run(command: list[str], tree: Path, environment: dict[str, str],
        output: Path, label: str, evidence: dict) -> str:
    started = now()
    tick = time.monotonic()
    process = subprocess.run(command, cwd=tree, env=environment,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    entry = dict(label=label, argv=command, cwd=str(tree), started_at=started, finished_at=now(),
                 elapsed_seconds=time.monotonic() - tick, exit_code=process.returncode)
    for key, data in (('stdout', process.stdout), ('stderr', process.stderr)):
        path = output / (label + '.' + key)
        with path.open('xb') as stream:
            stream.write(data)
        entry[key] = dict(path=str(path), bytes=len(data), sha256=sha(data))
    evidence.setdefault('commands', []).append(entry)
    if process.returncode:
        raise RuntimeError(f'{label} failed with exit {process.returncode}; see captured logs')
    return process.stdout.decode('utf-8')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='create-new evidence directory outside the repository')
    args = parser.parse_args()
    if args.output:
        output = args.output.resolve()
        if output == ROOT or ROOT in output.parents:
            parser.error('output must be outside the repository')
        output.mkdir(parents=True, exist_ok=False)
    else:
        output = Path(tempfile.mkdtemp(prefix='sabiquant-yield-extended-reproduction-')).resolve()
    tree = output / 'tree'
    runner = read_regular(RUNNER)
    original: dict[str, bytes] = {}
    staged: dict[str, bytes] = {}
    evidence = dict(schema=1, started_at=now(), result='incomplete', output=str(output), staged_root=str(tree),
                    python=sys.version, platform=platform.platform(),
                    interpreter=dict(path=sys.executable, sha256=sha(Path(sys.executable).read_bytes())),
                    runner=dict(path=str(RUNNER), bytes=len(runner), sha256=sha(runner)),
                    scope='Reproduction of unchanged old generators and all three frozen extended fixtures against their original historical policy metadata. No production source or tests are staged/read.',
                    limitation='The old direct --check commands against the living repository policy are not asserted to reproduce. This wrapper provides the explicit historical staging route.',
                    staged_policy_override=dict(path=LIVE_POLICY, source=HISTORICAL_POLICY,
                                                sha256=PINNED[HISTORICAL_POLICY], fnv64=POLICY_FNV64))
    failure = None
    try:
        names = set(GENERATORS + FIXTURES + CONTRACTS + (REQUIREMENTS, HISTORICAL_POLICY))
        original = {name: read_regular(ROOT / name) for name in sorted(names)}
        evidence['original_manifest'] = manifest(original)
        evidence['fixture_checks'] = validate_inputs(original)
        evidence['live_policy'] = dict(sha256=sha(original[LIVE_POLICY]), bytes=len(original[LIVE_POLICY]),
                                      matches_historical=original[LIVE_POLICY] == original[HISTORICAL_POLICY])
        staged = {name: data for name, data in original.items() if name != HISTORICAL_POLICY}
        staged[LIVE_POLICY] = original[HISTORICAL_POLICY]
        if len(staged) != 15 or any(name.startswith(('src/', 'tests/')) for name in staged):
            raise ValueError('minimal staged file census mismatch')
        for name, data in staged.items():
            path = tree / name
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open('xb') as stream:
                stream.write(data)
        evidence['staged_manifest'] = manifest(staged)
        initial = guard(original, runner, tree, staged)
        evidence['initial_guard'] = initial
        if not all(initial[key] for key in ('original_inputs_unchanged', 'runner_unchanged', 'staged_tree_unchanged')):
            raise RuntimeError('initial exact staging guard failed')
        environment = os.environ.copy()
        removed = []
        for key in ('PYTHONPATH', 'PYTHONHOME', 'PYTHONSTARTUP', 'PYTHONINSPECT', 'PYTHONUSERBASE'):
            if key in environment:
                removed.append(key)
                del environment[key]
        evidence['environment'] = dict(removed_keys=removed, isolated_python=True, bytecode_disabled=True)
        for label, generator in (('risk-and-public-check', RISK_GENERATOR), ('inverse-check', INVERSE_GENERATOR)):
            stdout = run([sys.executable, '-I', '-B', generator, '--check'], tree, environment, output, label, evidence)
            if generator == RISK_GENERATOR:
                lines = stdout.splitlines()
                if len(lines) != 2:
                    raise ValueError('risk generator did not attest exactly two frozen outputs')
                for line, expected in zip(lines, evidence['fixture_checks'][:2]):
                    actual = json.loads(line)
                    if actual != dict(path=expected['path'], bytes=expected['bytes'], sha256=expected['sha256'],
                                      fnv64=expected['fnv64'], check=True):
                        raise ValueError('risk generator output attestation mismatch')
            else:
                expected = evidence['fixture_checks'][2]
                exact = f"{expected['bytes']} bytes sha256={expected['sha256']} fnv64={expected['fnv64']} check=True\n"
                if stdout != exact:
                    raise ValueError('inverse generator output attestation mismatch')
            current = guard(original, runner, tree, staged)
            evidence.setdefault('post_command_guards', []).append(dict(label=label, **current))
            if not all(current[key] for key in ('original_inputs_unchanged', 'runner_unchanged', 'staged_tree_unchanged')):
                raise RuntimeError('original/runner/staged bytes changed during ' + label)
        evidence['result'] = 'passed'
        evidence['reproduced_fixture_rows'] = sum(EXPECTED_ROWS)
        evidence['actual_generator_check_count'] = 2
    except Exception as error:
        failure = error
        evidence['result'] = 'failed'
        evidence['failure'] = dict(type=type(error).__name__, message=str(error))
    finally:
        try:
            final_guard = guard(original, runner, tree, staged)
            evidence['final_guard'] = final_guard
            if not all(final_guard[key] for key in ('original_inputs_unchanged', 'runner_unchanged', 'staged_tree_unchanged')):
                evidence['result'] = 'failed'
                if failure is None:
                    failure = RuntimeError('final original/runner/staged guard failed')
                    evidence['failure'] = dict(type=type(failure).__name__, message=str(failure))
        except Exception as error:
            failure = failure or error
            evidence['result'] = 'failed'
            evidence['final_guard_failure'] = dict(type=type(error).__name__, message=str(error))
        evidence['finished_at'] = now()
        path = output / 'evidence.json'
        with path.open('x', encoding='utf-8') as stream:
            json.dump(evidence, stream, indent=2, sort_keys=True)
            stream.write('\n')
        print(json.dumps(dict(result=evidence['result'], evidence=str(path), evidence_sha256=sha(path.read_bytes())), sort_keys=True), flush=True)
    if failure is not None:
        raise SystemExit(str(failure))


if __name__ == '__main__':
    main()
