#!/usr/bin/env python3
"""Losslessly project the existing 119631 admission references into FS1.

No admission predicate, exact affine, native result or mathematical oracle is
recomputed. This stdlib-only converter copies frozen fields and proves their
round-trip identity before creating the output.
"""

import argparse
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PROVENANCE = HERE / "enclosure_admission.provenance.json"
REFERENCE = ROOT / "validation/enclosure/references/admission.jsonl.gz"
DEFAULT_OUTPUT = ROOT / "validation/fixtures/bonds/enclosure-admission.fixture"
COUNT = 119631
SOURCE_COUNT = 7410
COLUMNS = (
    "id", "model", "source_id", "yield_bits", "predicate",
    "canonical_quote_bits", "attempted_quote_bits", "frequency",
    "original_flow_index", "exact_affine_n", "exact_affine_d", "tau_days",
    "witnesses",
)
EXPECTED_FIELDS = {
    "MathematicallyAdmitted": {"canonical_quote_bits", "predicate"},
    "NonFiniteYield": {"attempted_quote_bits", "predicate"},
    "YieldNotAboveFloor": {"frequency", "predicate"},
    "NonPositiveSimpleDenominator": {"exact_affine", "original_flow_index", "predicate"},
    "NonPositiveMoneyMarketDenominator": {"exact_affine", "tau_days", "predicate"},
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def original_integer(value):
    require(type(value) is int, "original field is not an integer")
    return str(value)


def bits(text):
    require(isinstance(text, str) and re.fullmatch(r"[0-9a-f]{16}", text) is not None,
            "original quote bits must be bare sixteen-character lowercase hexadecimal")
    return text


def project(record):
    require(set(record) == {"expected", "id", "model", "source_id", "witnesses", "yield_bits"},
            "unexpected original admission fields")
    expected = record["expected"]
    predicate = expected["predicate"]
    require(predicate in EXPECTED_FIELDS and set(expected) == EXPECTED_FIELDS[predicate],
            "unexpected original predicate fields")
    row = {column: "-" for column in COLUMNS}
    for key in ("id", "model", "source_id", "yield_bits"):
        row[key] = record[key]
    bits(row["yield_bits"])
    row["predicate"] = predicate
    for key in ("canonical_quote_bits", "attempted_quote_bits"):
        if key in expected:
            row[key] = bits(expected[key])
    for key in ("frequency", "original_flow_index", "tau_days"):
        if key in expected:
            row[key] = original_integer(expected[key])
    if "exact_affine" in expected:
        affine = expected["exact_affine"]
        require(set(affine) == {"n", "d"}, "unexpected original affine fields")
        for component in ("n", "d"):
            value = affine[component]
            require(isinstance(value, str) and str(int(value)) == value,
                    "original affine integer is not canonical decimal text")
            row["exact_affine_" + component] = value
    witnesses = record["witnesses"]
    require(isinstance(witnesses, list), "original witness field is not a list")
    require(all(isinstance(value, str) and value and value != "-"
                and not any(c in value for c in ",\t\r\n") for value in witnesses),
            "witness cannot be transported unambiguously")
    row["witnesses"] = ",".join(witnesses) if witnesses else "-"
    return [row[column] for column in COLUMNS]


def restore(fields):
    row = dict(zip(COLUMNS, fields))
    expected = {"predicate": row["predicate"]}
    for key in ("canonical_quote_bits", "attempted_quote_bits"):
        if row[key] != "-":
            expected[key] = row[key]
    for key in ("frequency", "original_flow_index", "tau_days"):
        if row[key] != "-":
            expected[key] = int(row[key])
    if row["exact_affine_n"] != "-":
        expected["exact_affine"] = {"n": row["exact_affine_n"], "d": row["exact_affine_d"]}
    record = {key: row[key] for key in ("id", "model", "source_id", "yield_bits")}
    record["expected"] = expected
    record["witnesses"] = [] if row["witnesses"] == "-" else row["witnesses"].split(",")
    return record


def header(provenance, provenance_bytes):
    fields = [
        ("schema", "1"),
        ("fixture-set", "M8-ENCLOSURE-ADMISSION"),
        ("domain", "bond-price-enclosure"),
        ("description", "Lossless projection of the existing exact mathematical admission references"),
        ("reference", "oracle:previously sealed exact admission predicates and affine witnesses"),
        ("reference-version", "Original119631 predicates; Python3.9+ stdlib transport only"),
        ("reference-source", "validation/enclosure/references/admission.jsonl.gz"),
        ("tolerance", "TOL-E0; exact original predicate, fields and witness order; no numerical recomputation"),
        ("reproduction", "python3 -I validation/generator/enclosure_admission.py --check"),
        ("generated-at", provenance["source_frozen_at_utc"]),
        ("platform", "Portable Python stdlib projection; no float or native output"),
        ("generator-sha256", provenance["generator_sha256"]),
        ("provenance-sha256", sha(provenance_bytes)),
        ("original-admission-sha256", provenance["input"]["logical_sha256"]),
        ("record-count", str(COUNT)),
        ("distinct-source-count", str(SOURCE_COUNT)),
        ("encoding", "ASCII TSV; absent=-; bits=bare16lowerhex; witnesses=ordered comma list; empty list=-"),
        ("scope", "Frozen logical references only; no API work budget, source plans, price/cash targets or native consumer result generated"),
    ]
    return ("\n".join(key + ": " + value for key, value in fields)
            + "\n\n" + "\t".join(COLUMNS) + "\n").encode("ascii")


def convert(provenance, provenance_bytes):
    packed = REFERENCE.read_bytes()
    original = provenance["input"]
    require(len(packed) == original["transport_bytes"]
            and sha(packed) == original["transport_sha256"], "frozen admission gzip changed")
    data = gzip.decompress(packed)
    require(len(data) == original["logical_bytes"] and sha(data) == original["logical_sha256"],
            "frozen admission reference bytes changed")
    require(data.endswith(b"\n") and b"\r" not in data, "original JSONL LF termination changed")
    output = bytearray(header(provenance, provenance_bytes))
    statistics = Counter()
    sources = set()
    ordered_ids = hashlib.sha256()
    for number, line in enumerate(data.splitlines(), 1):
        require(line, "empty admission reference row")
        record = json.loads(line)
        require(record["id"] == "ENC-ADMISSION-%07d" % number,
                "original case missing/duplicate/reordered")
        require(record["model"] in ("Generic", "Gilt", "Treasury", "Canadian"),
                "unknown original model")
        require(re.fullmatch(r"ENC-SOURCE-[0-9]{6}", record["source_id"]) is not None,
                "original source ID changed")
        fields = project(record)
        require(len(fields) == len(COLUMNS) == 13, "admission column count changed")
        require(all(isinstance(value, str) and value and not any(c in value for c in "\t\r\n")
                    for value in fields), "unsafe or empty FS1 cell")
        require(restore(fields) == record, "FS1 projection lost an original case field")
        output.extend(("\t".join(fields) + "\n").encode("ascii"))
        sources.add(record["source_id"])
        statistics["rows"] += 1
        statistics["model_" + record["model"]] += 1
        statistics["predicate_" + record["expected"]["predicate"]] += 1
        statistics["rows_with_witnesses"] += int(bool(record["witnesses"]))
        ordered_ids.update((record["id"] + "\n").encode("ascii"))
    require(statistics["rows"] == COUNT, "original admission row count changed")
    require(len(sources) == SOURCE_COUNT, "original source subset changed")
    return output, statistics, ordered_ids.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="create a missing fixture; never overwrite")
    mode.add_argument("--check", action="store_true", help="reproduce the existing fixture without writing")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    if args.write and args.output.exists():
        parser.error("create-only output already exists; use --check")
    provenance_bytes = PROVENANCE.read_bytes()
    provenance = json.loads(provenance_bytes)
    require(provenance["generator_sha256"] == sha(Path(__file__).read_bytes()),
            "projection source changed after pre-generation freeze")
    require(provenance["columns"] == list(COLUMNS), "frozen projection schema changed")
    data, statistics, ordered_ids = convert(provenance, provenance_bytes)
    if args.check:
        require(args.output.read_bytes() == data, "fixture differs from the frozen projection")
    else:
        with args.output.open("xb") as handle:
            handle.write(data)
    print(json.dumps({
        "status": "PASS_CHECK" if args.check else "PASS_CREATE_ONLY",
        "fixture": str(args.output), "fixture_bytes": len(data), "fixture_sha256": sha(data),
        "ordered_case_ids_sha256": ordered_ids, "distinct_sources": SOURCE_COUNT,
        "generator_sha256": provenance["generator_sha256"],
        "provenance_sha256": sha(provenance_bytes), "counts": dict(sorted(statistics.items())),
        "scope": "Existing admission fields transported without numerical recomputation or native output",
    }, sort_keys=True))


if __name__ == "__main__":
    main()
