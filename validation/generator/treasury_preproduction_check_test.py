#!/usr/bin/env python3
"""Integrity-gate controls, not financial Treasury oracle expectations.

Synthetic schema fixtures test deliberate corruptions and the actual unchanged
Rust reader. They never enter validation/fixtures and cannot close a Treasury
financial, accuracy, implementation or preproduction-corpus gate.
"""
from datetime import datetime, timedelta, timezone
import functools
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('treasury_attestation', HERE / 'treasury_preproduction_check.py')
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


def independent_fnv(data):
    return '%016x' % functools.reduce(
        lambda h, b: ((h ^ b) * 1099511628211) % (2**64), data,
        14695981039346656037)


def pin(header, name, data):
    header[name + '-sha256'] = hashlib.sha256(data).hexdigest()
    header[name + '-fnv64'] = independent_fnv(data)


class IntegrityControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.scratch = tempfile.TemporaryDirectory(prefix='sabiquant-treasury-attestation-controls-')
        cls.work = Path(cls.scratch.name)
        cls.executable, cls.reader = checker.compile_reader(checker.ROOT, cls.work)

    @classmethod
    def tearDownClass(cls):
        cls.scratch.cleanup()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='case-', dir=self.work)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.relative = checker.FIXTURES[0]
        self.fixture = self.root / self.relative
        self.fixture.parent.mkdir(parents=True)
        self.generator = self.root / checker.GENERATORS[self.relative]
        self.generator.parent.mkdir(parents=True)
        self.generator.write_bytes(b'# synthetic integrity control; never an oracle\n')
        for name in (checker.MAIN, checker.CASH, 'docs/REQUIREMENTS.md'):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes((checker.ROOT / name).read_bytes())
        self.columns = ['id', 'profile', 'input', 'expected']
        approved = {self.relative: {'columns': tuple(self.columns),
                                    'profile': 'profile', 'inputs': ('input',),
                                    'metadata': {'fixture-set': 'INTEGRITY-CONTROL', 'domain': 'bonds'}}}
        self.schema_patch = patch.dict(checker.APPROVED_SCHEMAS, approved)
        self.schema_patch.start()
        self.addCleanup(self.schema_patch.stop)
        self.rows = [['A', 'LEFT', 'one', 'ok'], ['B', 'RIGHT', 'two', 'ok'],
                     ['C', 'LEFT', 'three', 'ok']]
        self.header = dict(zip(checker.REQUIRED, [
            '1', 'INTEGRITY-CONTROL', 'bonds', 'synthetic format control only',
            'synthetic', '1', 'no financial reference', 'TOL-E0 artifact only',
            'no oracle reproduction', datetime.now(timezone.utc).isoformat(),
            'test runner actual host']))
        self.refresh_pins()

    def refresh_pins(self):
        records = ('\n'.join('\t'.join(row) for row in self.rows) + '\n').encode()
        body = ('\t'.join(self.columns) + '\n').encode() + records
        inputs = ('\n'.join(row[2] for row in self.rows) + '\n').encode()
        self.header.update({'record-count': str(len(self.rows)),
                            'column-count': str(len(self.columns)),
                            'profile-counts': json.dumps({'LEFT': 2, 'RIGHT': 1}),
                            'input-columns': 'input',
                            'unique-input-count': str(len(set(row[2] for row in self.rows)))})
        pin(self.header, 'generator', self.generator.read_bytes())
        pin(self.header, 'adopted-contract', (self.root / checker.MAIN).read_bytes())
        pin(self.header, 'cash-contract', (self.root / checker.CASH).read_bytes())
        pin(self.header, 'requirements-rows', checker.requirement_rows(self.root))
        pin(self.header, 'body', body)
        pin(self.header, 'records', records)
        pin(self.header, 'inputs', inputs)
        self.body = body

    def write(self):
        self.fixture.write_bytes(('\n'.join(k + ': ' + v for k, v in self.header.items()) + '\n\n').encode() + self.body)

    def reject(self, pattern):
        self.write()
        with self.assertRaisesRegex(ValueError, pattern):
            checker.attest_fixture(self.root, self.relative)

    def test_valid_frame_and_real_reader(self):
        self.write()
        result = checker.attest_fixture(self.root, self.relative)
        self.assertEqual((result['rows'], result['unique_inputs']), (3, 3))
        completed = subprocess.run([self.executable, self.fixture], capture_output=True)
        self.assertEqual(completed.returncode, 0, completed.stdout + completed.stderr)
        self.assertIn(b'TOTAL\t3\n', completed.stdout)

    def test_fnv_known_vectors_and_independent_route(self):
        for payload, expected in ((b'', 'cbf29ce484222325'), (b'hello', 'a430d84680aabd0b')):
            self.assertEqual(checker.fnv(payload), expected)
            self.assertEqual(independent_fnv(payload), expected)

    def test_generator_changed_while_fixture_pin_unchanged(self):
        self.generator.write_bytes(b'# changed\n')
        self.reject('generator-sha256')

    def test_contract_changed_even_if_fixture_resealed(self):
        contract = self.root / checker.MAIN
        contract.write_bytes(contract.read_bytes() + b'\n')
        self.refresh_pins()
        self.reject('adopted contract changed')

    def test_cash_contract_changed_even_if_fixture_resealed(self):
        contract = self.root / checker.CASH
        contract.write_bytes(contract.read_bytes() + b'\n')
        self.refresh_pins()
        self.reject('adopted contract changed')

    def test_requirement_behavior_changed(self):
        path = self.root / 'docs/REQUIREMENTS.md'
        path.write_text(path.read_text().replace('| FR-UST-01 | Retain', '| FR-UST-01 | Replace', 1))
        self.reject('adopted requirement rows changed')

    def test_missing_requirement_id_is_not_just_a_new_pin(self):
        path = self.root / 'docs/REQUIREMENTS.md'
        path.write_text(path.read_text().replace('| FR-UST-26 |', '| FR-UST-99 |', 1))
        self.write()
        with self.assertRaisesRegex(ValueError, 'FR-UST row census'):
            checker.attest_fixture(self.root, self.relative)

    def test_record_mutation_is_detected(self):
        self.body = self.body.replace(b'three', b'four')
        self.reject('body-sha256')

    def test_each_independent_pin_is_mandatory(self):
        pristine = dict(self.header)
        for prefix in ('generator', 'adopted-contract', 'cash-contract',
                       'requirements-rows', 'body', 'records', 'inputs'):
            for suffix in ('sha256', 'fnv64'):
                with self.subTest(prefix=prefix, suffix=suffix):
                    self.header = dict(pristine)
                    del self.header[prefix + '-' + suffix]
                    self.reject(prefix + '-' + suffix)

    def test_counts_are_verified_independently(self):
        pristine = dict(self.header)
        for key in ('record-count', 'column-count', 'unique-input-count'):
            with self.subTest(key=key):
                self.header = dict(pristine)
                self.header[key] = '99'
                self.reject(key)

    def test_profile_census_and_integer_types(self):
        for value in ('{"LEFT":1,"RIGHT":2}', '{"LEFT":true,"RIGHT":1}',
                      '{"LEFT":2.0,"RIGHT":1}', '{}', '[]'):
            with self.subTest(value=value):
                self.header['profile-counts'] = value
                self.reject('profile-counts')

    def test_duplicate_semantic_input_rejected_even_with_valid_new_pins(self):
        self.rows[1][2] = 'one'
        self.refresh_pins()
        self.reject('duplicate semantic input')

    def test_input_selection_cannot_be_ids_profiles_missing_or_duplicate_names(self):
        for columns in ('id', 'profile', 'missing', 'input,input', ''):
            with self.subTest(columns=columns):
                self.header['input-columns'] = columns
                self.reject('input-columns|empty header')

    def test_answer_column_cannot_mask_duplicate_semantic_inputs(self):
        for row, answer in zip(self.rows, ('first-answer', 'second-answer', 'third-answer')):
            row[2] = 'one'
            row[3] = answer
        self.refresh_pins()
        self.header['input-columns'] = 'expected'
        self.header['unique-input-count'] = '3'
        pin(self.header, 'inputs', ('\n'.join(row[3] for row in self.rows) + '\n').encode())
        self.reject('unapproved input-columns')

    def test_unreviewed_schema_fails_closed(self):
        self.write()
        with patch.dict(checker.APPROVED_SCHEMAS, {}, clear=True):
            with self.assertRaisesRegex(ValueError, 'schema not reviewed'):
                checker.attest_fixture(self.root, self.relative)

    def test_added_answer_column_cannot_extend_approved_schema(self):
        self.columns.append('another_answer')
        for row in self.rows:
            row.append('different')
        self.refresh_pins()
        self.reject('unapproved corpus columns')

    def test_duplicate_id_rejected_separately_from_inputs(self):
        self.rows[1][0] = 'A'
        self.refresh_pins()
        self.reject('duplicate record IDs')

    def test_timestamp_must_be_actual_aware_past_utc(self):
        for stamp in ('2026-01-01T00:00:00', '2026-01-01T00:00:00+07:00',
                      (datetime.now(timezone.utc) + timedelta(days=1)).isoformat()):
            with self.subTest(stamp=stamp):
                self.header['generated-at'] = stamp
                self.reject('UTC|future')

    def test_canonical_bytes_are_not_silently_normalized(self):
        self.write()
        for raw in (self.fixture.read_bytes().replace(b'\n', b'\r\n'),
                    self.fixture.read_bytes().rstrip(b'\n')):
            with self.subTest(payload=raw[-50:]):
                with self.assertRaisesRegex(ValueError, 'LF'):
                    checker.read_table(raw)

    def test_real_reader_rejects_schema_and_width_corruption(self):
        self.write()
        positive = self.fixture.read_bytes()
        cases = {
            'uppercase-header': positive.replace(b'domain: bonds', b'Domain: bonds'),
            'duplicate-header': positive.replace(b'domain: bonds', b'domain: bonds\ndomain: bonds'),
            'missing-required': positive.replace(b'tolerance: TOL-E0 artifact only\n', b''),
            'wrong-schema': positive.replace(b'schema: 1', b'schema: 2'),
            'width': positive.replace(b'A\tLEFT\tone\tok', b'A\tLEFT\tone'),
            'duplicate-id': positive.replace(b'B\tRIGHT', b'A\tRIGHT'),
        }
        for label, payload in cases.items():
            with self.subTest(label=label):
                path = self.root / (label + '.fixture')
                path.write_bytes(payload)
                completed = subprocess.run([self.executable, path], capture_output=True)
                self.assertNotEqual(completed.returncode, 0)
                self.assertIn(b'shared fixture reader', completed.stderr)

    def test_duplicate_json_profile_key_cannot_be_silently_overwritten(self):
        self.header['profile-counts'] = '{"LEFT":999,"LEFT":2,"RIGHT":1}'
        self.reject('duplicate JSON key')

    def test_nonstandard_json_constants_rejected(self):
        self.header['profile-counts'] = '{"LEFT":NaN,"RIGHT":1}'
        self.reject('nonstandard JSON constant')

    def test_corpus_identity_cannot_be_relabelled(self):
        for field, value in (('fixture-set', 'OTHER-CORPUS'), ('domain', 'other')):
            with self.subTest(field=field):
                saved = self.header[field]
                self.header[field] = value
                self.reject('unapproved metadata identity')
                self.header[field] = saved

    def test_resealed_requirement_mutation_cannot_change_adopted_baseline(self):
        path = self.root / 'docs/REQUIREMENTS.md'
        path.write_text(path.read_text().replace('| FR-UST-01 | Retain', '| FR-UST-01 | Replace', 1))
        with self.assertRaisesRegex(ValueError, 'adopted requirement rows changed'):
            self.refresh_pins()

    def test_unknown_corpus_cannot_substitute_for_required_one(self):
        with self.assertRaisesRegex(ValueError, 'unregistered'):
            checker.attest_fixture(self.root, 'unregistered.fixture')

    def test_logged_command_failure_records_actual_exit(self):
        with self.assertRaisesRegex(checker.CommandFailed, 'command failed') as context:
            checker.run_logged(['false'], self.root, self.root, 'negative')
        self.assertTrue((self.root / 'negative.log').is_file())
        self.assertEqual(context.exception.result['exit_code'], 1)
        self.assertEqual(context.exception.result['command'], ['false'])
        self.assertEqual(context.exception.result['log_sha256'], hashlib.sha256(b'').hexdigest())

    def run_isolated_main(self, mutation):
        """Real generator/reader subprocesses in a synthetic, nonfinancial root."""
        code = {
            'none': 'pass\n',
            'generator': 'from pathlib import Path\np=Path(__file__)\np.write_bytes(p.read_bytes()+b"#changed\\n")\n',
            'checker': 'from pathlib import Path\np=Path(__file__).parents[2]/"checker.py"\np.write_bytes(p.read_bytes()+b"#changed\\n")\n',
        }[mutation]
        self.generator.write_text(code)
        self.refresh_pins()
        self.write()
        actual_checker = Path(checker.__file__)
        fake_checker = self.root / 'checker.py'
        fake_checker.write_bytes(actual_checker.read_bytes())
        reader = self.root / 'tests/support/fixture.rs'
        reader.parent.mkdir(parents=True)
        reader.write_bytes((checker.ROOT / 'tests/support/fixture.rs').read_bytes())
        output = self.root.parent / (self.root.name + '-attestation')
        actual_output = subprocess.check_output

        def identity_command(args, **kwargs):
            if args[:3] == ['git', 'rev-parse', 'HEAD']:
                return 'synthetic-integrity-control\n'
            return actual_output(args, **kwargs)

        with patch.object(checker, 'ROOT', self.root), \
             patch.object(checker, '__file__', str(fake_checker)), \
             patch.object(checker, 'tracked_snapshot', return_value={}), \
             patch.object(checker.subprocess, 'check_output', side_effect=identity_command):
            result = checker.main(['--output-dir', str(output), '--fixtures', self.relative])
        return result, json.loads((output / 'evidence.json').read_text())

    def test_real_subset_success_never_closes_four_corpus_gate(self):
        result, evidence = self.run_isolated_main('none')
        self.assertEqual(result, 0)
        self.assertEqual(evidence['status'], 'PASS')
        self.assertFalse(evidence['complete_scope'])
        self.assertTrue(evidence['new_artifact_guard'])

    def test_real_generator_self_mutation_fails_whole_run_guard(self):
        result, evidence = self.run_isolated_main('generator')
        self.assertEqual(result, 1)
        self.assertEqual(evidence['status'], 'FAIL')
        self.assertEqual(evidence['reproduction'][0]['exit_code'], 0)
        self.assertEqual(evidence['shared_reader']['execution']['exit_code'], 0)
        self.assertIn('bytes changed during entire check', evidence['failure'])

    def test_real_checker_mutation_fails_whole_run_guard(self):
        result, evidence = self.run_isolated_main('checker')
        self.assertEqual(result, 1)
        self.assertEqual(evidence['status'], 'FAIL')
        self.assertEqual(evidence['shared_reader']['execution']['exit_code'], 0)
        self.assertIn('bytes changed during entire check', evidence['failure'])


if __name__ == '__main__':
    unittest.main(verbosity=2)
