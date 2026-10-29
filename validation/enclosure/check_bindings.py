#!/usr/bin/env python3
"""Check enclosure obligation/index bindings, without evaluating a price oracle.

Python 3.9+, standard library only. Pending consumers are never reported as run.
"""

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys


class BindingError(ValueError):
    """A manifest or selected immutable input does not match its binding."""


def require(condition, message):
    if not condition:
        raise BindingError(message)


def unique(items, label):
    require(len(items) == len(set(items)), "duplicate " + label)


def field(record, path):
    value = record
    for part in path.split("."):
        if not isinstance(value, dict) or part not in value:
            raise BindingError("unknown selector field: " + path)
        value = value[part]
    return value


def matches(record, selector):
    if "ids" in selector and record["id"] not in selector["ids"]:
        return False
    for clause in selector.get("where", []):
        value = field(record, clause["field"])
        if clause["op"] == "eq":
            accepted = value == clause["value"]
        elif clause["op"] == "in":
            accepted = value in clause["value"]
        elif clause["op"] == "contains":
            accepted = clause["value"] in value
        else:
            raise BindingError("unknown selector operation")
        if not accepted:
            return False
    return True


def records(path, encoding):
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt", encoding="utf-8", newline="") as handle:
        if encoding == "jsonl":
            for line in handle:
                if line.strip():
                    yield json.loads(line)
        elif encoding == "json_array":
            yield from json.load(handle)
        elif encoding == "fixture_tsv":
            header = None
            for line in handle:
                cells = line.rstrip("\n").split("\t")
                if header is None:
                    if cells[0] == "id":
                        header = cells
                    continue
                if not line.strip():
                    continue
                require(len(cells) == len(header), "fixture column count")
                yield dict(zip(header, cells))
            require(header is not None, "missing fixture header")
        else:
            raise BindingError("unknown artifact encoding")


def logical_sha256(path):
    opener = gzip.open if path.suffix == ".gz" else open
    digest = hashlib.sha256()
    with opener(path, "rb") as handle:
        while True:
            block = handle.read(1024 * 1024)
            if not block:
                break
            digest.update(block)
    return digest.hexdigest()


def validate_structure(manifest):
    require(manifest["schema"] == "sabiquant-enclosure-consumer-bindings-v1", "schema")
    require(manifest["scope"] == ["B1", "B5"], "bounded action scope")
    require(manifest["public_consumer_execution_claimed"] is False, "execution claim")
    groups = manifest["contract_groups"]
    require(groups == ["FR-ENCL-%02d" % n for n in range(1, 19)], "18 groups")
    expected_ids = []
    for category, count in [("G", 9), ("L", 5), ("U", 6), ("D", 8),
                            ("X", 7), ("N", 9), ("C", 11), ("A", 7),
                            ("R", 6), ("T", 4)]:
        expected_ids.extend("EP-%s%02d" % (category, n) for n in range(1, count + 1))
    rows = manifest["obligations"]
    require([row["id"] for row in rows] == expected_ids, "complete ordered 72 EP IDs")
    selectors = manifest["selectors"]
    unique([item["id"] for item in selectors], "selector ID")
    selector_ids = {item["id"] for item in selectors}
    for item in selectors:
        require(item["artifact"] in manifest["artifacts"], "selector artifact")
        require(item["selected_count"] > 0, "empty selector")
        require(re.fullmatch(r"[0-9a-f]{64}", item["selected_ids_sha256"]) is not None,
                "selected ID digest")
        require(item["example_ids"], "missing literal example IDs")
    assertion_ids = []
    used_groups = set()
    for row in rows:
        require(row["contract_groups"] and set(row["contract_groups"]) <= set(groups),
                "EP group mapping")
        used_groups.update(row["contract_groups"])
        require(row["selector_ids"] and set(row["selector_ids"]) <= selector_ids,
                "EP selector binding")
        consumer = row["future_consumer"]
        require(consumer["status"] == "pending_implementation", "consumer execution status")
        require(consumer["assertions"], "missing consumer assertions")
        require(consumer["planned_test_path"].startswith("tests/"), "consumer test path")
        assertion_ids.append(consumer["assertion_id"])
    unique(assertion_ids, "consumer assertion ID")
    require(used_groups == set(groups), "unbound contract group")
    for bridge in manifest["cash_bridges"]:
        require(bridge["status"] == "pending_originating_producer_consumer", "cash status")
        require(set(bridge["selector_ids"]) <= selector_ids, "cash selector binding")
        require(bridge["assertions"], "cash assertions")
    return len(rows)


def check_artifact(path, artifact, selectors):
    require(logical_sha256(path) == artifact["logical_sha256"],
            "artifact SHA256 mismatch: " + str(path))
    all_ids = set()
    selected = {item["id"]: [] for item in selectors}
    count = 0
    for record in records(path, artifact["encoding"]):
        identifier = record["id"]
        require(identifier not in all_ids, "duplicate artifact row ID: " + identifier)
        all_ids.add(identifier)
        count += 1
        for selector in selectors:
            if matches(record, selector):
                selected[selector["id"]].append(identifier)
    require(count == artifact["record_count"], "artifact record count")
    for selector in selectors:
        ids = selected[selector["id"]]
        digest = hashlib.sha256("".join(x + "\n" for x in ids).encode()).hexdigest()
        require(len(ids) == selector["selected_count"], "selector count: " + selector["id"])
        require(digest == selector["selected_ids_sha256"], "selector IDs: " + selector["id"])
        require(set(selector["example_ids"]) <= set(ids), "selector examples")
        if "ids" in selector:
            require(set(selector["ids"]) == set(ids), "unconsumed explicit selector IDs")
    return count


def check_document_mapping(repo, manifest):
    """The committed readable matrix and machine index must name the same map."""
    path = repo / "docs/M8_PRICE_ENCLOSURE_TEST_MATRIX.md"
    actual = {}
    for line in path.read_text().splitlines():
        if not line.startswith("| EP-"):
            continue
        cells = [cell.strip() for cell in line.split("|")]
        identifier = cells[1].split(" ", 1)[0]
        require(identifier not in actual, "duplicate readable EP ID")
        actual[identifier] = ["FR-ENCL-" + value.strip()
                              for value in cells[2].split(",")]
    expected = {row["id"]: row["contract_groups"] for row in manifest["obligations"]}
    require(actual == expected, "readable/machine 72-to-18 mapping differs")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path(__file__).with_name("consumer-bindings.json"))
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--reference-root", type=Path,
                        help="optional local work root for not-yet-promoted immutable artifacts")
    parser.add_argument("--require-promoted", action="store_true",
                        help="fail if any numerical artifact lacks a committed transport")
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    rows = validate_structure(manifest)
    check_document_mapping(args.repo, manifest)
    checked = {}
    pending = []
    for key, artifact in manifest["artifacts"].items():
        promoted = artifact.get("committed_path")
        path = args.repo / promoted if promoted else None
        if path is None or not path.is_file():
            pending.append(key)
            require(not args.require_promoted, "artifact promotion pending: " + key)
            path = (args.reference_root / artifact["original_member"]
                    if args.reference_root is not None else None)
        if path is None or not path.is_file():
            continue
        selectors = [item for item in manifest["selectors"] if item["artifact"] == key]
        checked[key] = check_artifact(path, artifact, selectors)
    print(json.dumps({"binding_structure": "pass", "obligations": rows,
                      "contract_groups": len(manifest["contract_groups"]),
                      "selectors": len(manifest["selectors"]),
                      "checked_reference_records": checked,
                      "pending_artifact_promotion": pending,
                      "numerical_oracle_evaluated": False,
                      "public_producer_consumers_executed": False}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (BindingError, KeyError, TypeError, ValueError, OSError) as error:
        print("binding check failed: " + str(error), file=sys.stderr)
        sys.exit(1)
