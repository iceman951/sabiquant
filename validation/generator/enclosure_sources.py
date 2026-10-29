#!/usr/bin/env python3
"""Project the existing sealed source inputs to the std-only FS1 test format.

No source formula, numerical oracle or prepared plan is recomputed. The complete
original source data remains in the byte-exact committed gzip input.
"""
import argparse
import datetime
import gzip
import hashlib
import io
import json
from pathlib import Path
import platform
import re

ROOT = Path(__file__).resolve().parents[2]
INPUT = ROOT / 'validation/enclosure/references/sources.jsonl.gz'
INPUT_SHA = '71d7464c09c4f55998222e0c37b3c69af3689c0016743f26627b1c199b68e24e'
COLUMNS = ('id', 'model', 'families', 'input', 'source_summary')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def flat(value):
    """Typed, explicit nodes preserve all selected literal values and order."""
    out = []

    def visit(key, value):
        if isinstance(value, dict):
            out.append((key, 'd:' + str(len(value))))
            for member in sorted(value):
                if not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', member):
                    raise ValueError('unsupported source key: ' + member)
                visit(key + '.' + member if key else member, value[member])
        elif isinstance(value, list):
            out.append((key, 'l:' + str(len(value))))
            for index, member in enumerate(value):
                visit(key + '[' + str(index) + ']', member)
        elif value is None:
            out.append((key, 'z:_'))
        elif isinstance(value, bool):
            out.append((key, 'b:' + str(int(value))))
        elif isinstance(value, int):
            out.append((key, 'i:' + str(value)))
        elif isinstance(value, str):
            if any(c in value for c in ';=\t\r\n') or not value.isascii():
                raise ValueError('unsupported source literal')
            out.append((key, 's:' + value))
        else:
            raise TypeError(type(value))

    visit('', value)
    # '$' denotes the root container; source members themselves never use '$'.
    return ';'.join((key or '$') + '=' + value for key, value in out)


def selected_source(row):
    if row['model'] == 'Generic':
        return row['source']
    summary = {key: value for key, value in row['source'].items() if key != 'bundles'}
    additions = {
        'prepared_bundle_count': len(row['source']['bundles']),
        'prepared_bundle_indices': [b['index'] for b in row['source']['bundles']],
        'prepared_payment_dates': [b.get('actual_payment_date', b.get('payment'))
                                   for b in row['source']['bundles']],
    }
    assert not (set(additions) & set(summary))
    summary.update(additions)
    return summary


def render(generated_at, host_platform):
    data = gzip.decompress(INPUT.read_bytes())
    assert hashlib.sha256(data).hexdigest() == INPUT_SHA
    records = io.StringIO()
    ids = set()
    models = {}
    for line in data.decode('ascii').splitlines():
        row = json.loads(line)
        assert row['id'] not in ids
        ids.add(row['id'])
        models[row['model']] = models.get(row['model'], 0) + 1
        fields = [row['id'], row['model'], ','.join(row['families']),
                  flat(row['input']), flat(selected_source(row))]
        assert len(fields) == len(COLUMNS)
        assert all(not any(c in field for c in '\t\r\n') for field in fields)
        records.write('\t'.join(fields) + '\n')
    assert len(ids) == 30920
    headers = [
        ('schema', '1'), ('fixture-set', 'M8-ENCLOSURE-SOURCE-INPUTS'),
        ('domain', 'bond-price-enclosure'),
        ('description', 'Portable projection of the unchanged sealed 30920 dated source inputs'),
        ('reference', 'existing independently derived frozen four-model source corpus'),
        ('reference-version', 'Source schema3; projection format1; no numerical evaluation'),
        ('reference-source', 'validation/enclosure/references/sources.jsonl.gz'),
        ('tolerance', 'TOL-E0; every selected literal and stored source field is copied exactly'),
        ('reproduction', 'python3 -I validation/generator/enclosure_sources.py --check validation/fixtures/bonds/enclosure-sources.fixture'),
        ('generated-at', generated_at), ('platform', host_platform),
        ('source-logical-sha256', INPUT_SHA), ('source-transport-sha256', sha(INPUT)),
        ('generator-sha256', sha(Path(__file__))), ('record-count', str(len(ids))),
        ('model-counts', json.dumps(models, sort_keys=True, separators=(',', ':'))),
        ('cell-encoding', 'semicolon key=typed-value nodes; root=$; s:string i:integer b:0or1 z:null d:member-count l:element-count; arrays use [index]'),
        ('source-projection', 'Generic complete stored fields; named global fields and prepared bundle count/original indices/payment dates; complete original source remains in committed gzip'),
        ('scope', 'B1/B4 input transport only; no new source oracle, native probe, financial producer or E2 completion'),
    ]
    header = '\n'.join(key + ': ' + value for key, value in headers)
    return (header + '\n\n' + '\t'.join(COLUMNS) + '\n' + records.getvalue()).encode('ascii')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--out', type=Path)
    group.add_argument('--check', type=Path)
    args = parser.parse_args()
    if args.check:
        current = args.check.read_bytes()
        header = dict(line.split(': ', 1) for line in current.decode('ascii').split('\n\n', 1)[0].splitlines())
        wanted = render(header['generated-at'], header['platform'])
        if current != wanted:
            raise SystemExit('source projection differs; frozen expectations must not be overwritten')
        print('PASS exact source projection reproduction:30920 unchanged source IDs')
    else:
        data = render(datetime.datetime.now(datetime.timezone.utc).isoformat(),
                      platform.system() + ' ' + platform.machine())
        with args.out.open('xb') as output:
            output.write(data)
        print('CREATED ' + str(args.out) + ' bytes=' + str(len(data)) +
              ' sha256=' + hashlib.sha256(data).hexdigest())


if __name__ == '__main__':
    main()
