#!/usr/bin/env python3
"""Research TSV integrity regressions; no financial requirement adoption.

These controls reject text that Python's permissive int/CSV parsers would accept
but the Rust harness cannot emit. They do not manufacture numerical expectations.
"""
from pathlib import Path
import importlib.util
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('enclosure_compare', Path(__file__).with_name('compare.py'))
compare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compare)


class CandidateProtocol(unittest.TestCase):
    def setUp(self):
        self.row = dict.fromkeys(compare.FIELDS, '_')
        self.row.update(id='EP-PRIM-000001', op='normalize', status='ok',
                        lower_sign='1', lower_m_words='8000000000000000',
                        lower_K_sign='0', lower_K_words='0000000000000000',
                        upper_sign='1', upper_m_words='8000000000000000',
                        upper_K_sign='0', upper_K_words='0000000000000000',
                        work_used='184')

    def payload(self, row=None, fields=None):
        fields = compare.FIELDS if fields is None else fields
        row = self.row if row is None else row
        return ('\t'.join(fields)+'\n'+'\t'.join(row[f] for f in fields)+'\n').encode('ascii')

    def read(self, payload):
        with tempfile.TemporaryDirectory(prefix='sabiquant-compare-') as directory:
            path = Path(directory)/'actual.tsv'
            path.write_bytes(payload)
            return compare.read_candidate(path)

    def test_exact_positive_point_and_zero_metadata(self):
        rows = self.read(self.payload())
        self.assertEqual(rows, [self.row])
        self.assertEqual(compare.point(rows[0], 'lower', 64), (1, 1<<63, 0))

    def test_error_rows_keep_absent_value_markers(self):
        row = dict.fromkeys(compare.FIELDS, '_')
        row.update(id='EP-PRIM-000002', op='div', status='zero_denominator', work_used='116')
        self.assertEqual(self.read(self.payload(row)), [row])

    def test_export_finite_and_unavailable_cells(self):
        row = dict.fromkeys(compare.FIELDS, '_')
        row.update(id='EP-PRIM-000003', op='export_binary64', status='ok', work_used='107',
                   lower_binary64_bits='7fefffffffffffff', upper_binary64_bits='unavailable')
        self.assertEqual(self.read(self.payload(row)), [row])

    def test_reject_missing_final_lf(self):
        with self.assertRaises(AssertionError): self.read(self.payload()[:-1])

    def test_reject_crlf_blank_and_quoted_rows(self):
        data = self.payload()
        for bad in (data.replace(b'\n', b'\r\n'), data+b'\n', data.replace(b'\t1\t', b'\t"1"\t', 1)):
            with self.subTest(bad=bad[:50]), self.assertRaises(AssertionError): self.read(bad)

    def test_reject_bom_and_non_ascii(self):
        for data in (b'\xef\xbb\xbf'+self.payload(), self.payload().replace(b'EP-PRIM', '\u0e01'.encode('utf-8'))):
            with self.subTest(data=data[:20]), self.assertRaises(AssertionError): self.read(data)

    def test_reject_unknown_duplicate_and_reordered_headers(self):
        for fields in (compare.FIELDS+['extra'], ['unknown']+compare.FIELDS[1:],
                       [compare.FIELDS[1], compare.FIELDS[0]]+compare.FIELDS[2:],
                       [compare.FIELDS[0]]+compare.FIELDS[:-1]):
            row = dict(self.row, extra='_', unknown='_')
            with self.subTest(fields=fields[:2]), self.assertRaises(AssertionError): self.read(self.payload(row, fields))

    def test_reject_missing_or_extra_row_cells(self):
        header, row, _ = self.payload().split(b'\n')
        for bad in (header+b'\n'+row.rsplit(b'\t', 1)[0]+b'\n', header+b'\n'+row+b'\t_\n'):
            with self.subTest(bad=bad[-20:]), self.assertRaises(AssertionError): self.read(bad)

    def test_reject_python_accepted_noncanonical_signs(self):
        for value in ('+1', '01', '-0', '+0', ' 1', '2'):
            for field in ('lower_sign', 'upper_K_sign'):
                row = dict(self.row); row[field] = value
                with self.subTest(field=field, value=value), self.assertRaises(AssertionError): self.read(self.payload(row))

    def test_reject_signed_uppercase_short_and_overwide_limbs(self):
        for value in ('+000000000000000', '-000000000000001', '000000000000000A',
                      '0', '', '0'*17, ',0000000000000000', '8000000000000000,0000000000000000'):
            with self.subTest(value=value), self.assertRaises(AssertionError): compare.words(value, 16)
        with self.assertRaises(AssertionError): compare.words(','.join(['0000000000000001']*17), 16)

    def test_valid_low_zero_limb_and_metadata_width(self):
        text = '0000000000000000,8000000000000000'
        self.assertEqual(compare.words(text, 16), 1<<127)
        self.assertEqual(compare.words(','.join(['ffffffffffffffff']*33), 33), (1<<2112)-1)

    def test_reject_invalid_work_counter_lexemes_and_overflow(self):
        for value in ('+1', '01', '-0', '-1', '1.0', '1e2', '', str(1<<64)):
            row = dict(self.row, work_used=value)
            with self.subTest(value=value), self.assertRaises(AssertionError): self.read(self.payload(row))
        for value in ('0', str((1<<64)-1)):
            row = dict(self.row, work_used=value)
            self.assertEqual(self.read(self.payload(row)), [row])

    def test_reject_unknown_operation_status_and_empty_id(self):
        for field, value in (('op', 'pow'), ('status', 'success'), ('id', ''), ('id', 'has space')):
            row = dict(self.row); row[field] = value
            with self.subTest(field=field, value=value), self.assertRaises(AssertionError): self.read(self.payload(row))

    def test_reject_invalid_binary64_cell(self):
        for value in ('7FEFFFFFFFFFFFFF', 'nan', '+000000000000000', '0', ''):
            row = dict(self.row, lower_binary64_bits=value)
            with self.subTest(value=value), self.assertRaises(AssertionError): self.read(self.payload(row))

    def test_point_semantics_reject_forged_zero_and_wrong_precision(self):
        for changes in ({'lower_m_words': '0000000000000000'}, {'lower_K_sign': '-1'},
                        {'lower_m_words': '0000000000000001'}):
            row = dict(self.row); row.update(changes)
            with self.subTest(changes=changes), self.assertRaises(AssertionError): compare.point(row, 'lower', 64)


if __name__ == '__main__':
    unittest.main()
