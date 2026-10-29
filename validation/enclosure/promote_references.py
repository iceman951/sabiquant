#!/usr/bin/env python3
"""Promote/check byte-exact enclosure reference input transports, not oracles.

Default/check mode needs only committed files and Python's standard library.
Write mode requires the original sealed local work root and never overwrites.
"""

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import sys


class TransportError(ValueError):
    """A source or committed transport does not match its frozen byte pin."""


def require(condition, message):
    if not condition:
        raise TransportError(message)


def digest_stream(handle):
    digest = hashlib.sha256()
    size = 0
    while True:
        block = handle.read(1024 * 1024)
        if not block:
            break
        digest.update(block)
        size += len(block)
    return size, digest.hexdigest()


def verify_identity(path, expected_size, expected_sha256, compressed=False):
    opener = gzip.open if compressed else open
    with opener(path, "rb") as handle:
        size, actual_sha256 = digest_stream(handle)
    require(size == expected_size, "byte count mismatch: " + str(path))
    require(actual_sha256 == expected_sha256, "SHA256 mismatch: " + str(path))


def verify_transport(path, asset):
    verify_identity(path, asset["transport_bytes"], asset["transport_sha256"])
    verify_identity(path, asset["logical_bytes"], asset["logical_sha256"], compressed=True)


def promote(source, target, asset):
    verify_identity(source, asset["logical_bytes"], asset["logical_sha256"])
    if target.exists():
        verify_transport(target, asset)
        return False
    # Exclusive creation: a differing existing file is never rewritten. The
    # frozen original is read directly; JSON is not parsed or reserialized.
    with target.open("xb") as output:
        with gzip.GzipFile(filename="", mode="wb", fileobj=output,
                           compresslevel=9, mtime=0) as archive:
            with source.open("rb") as original:
                while True:
                    block = original.read(1024 * 1024)
                    if not block:
                        break
                    archive.write(block)
    verify_transport(target, asset)
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group()
    action.add_argument("--check", action="store_true", help="verify committed transports (default)")
    action.add_argument("--write", action="store_true", help="create missing transports only")
    parser.add_argument("--source-root", type=Path, help="original local work root; required for --write")
    parser.add_argument("--references", type=Path,
                        default=Path(__file__).with_name("references"))
    args = parser.parse_args()
    require(not args.write or args.source_root is not None, "--write requires --source-root")
    provenance = json.loads((args.references / "PROVENANCE.json").read_text())
    require(provenance["schema"] == "sabiquant-enclosure-reference-transport-v1", "schema")
    assets = provenance["assets"]
    require(len(assets) == provenance["asset_count"] == 11, "eleven approved input assets")
    names = [item["name"] for item in assets]
    require(len(names) == len(set(names)), "duplicate transport name")
    created = []
    checked = []
    for asset in assets:
        require(asset["transport"] == "gzip", "only approved gzip input assets")
        require(Path(asset["name"]).name == asset["name"], "unsafe transport name")
        member = Path(asset["original_member"])
        require(not member.is_absolute() and ".." not in member.parts, "unsafe original member")
        target = args.references / asset["name"]
        if args.write:
            if promote(args.source_root / member, target, asset):
                created.append(asset["name"])
        else:
            verify_transport(target, asset)
            if args.source_root is not None:
                verify_identity(args.source_root / member, asset["logical_bytes"],
                                asset["logical_sha256"])
        checked.append(asset["name"])
    require(sum(item["logical_bytes"] for item in assets) == provenance["logical_total_bytes"],
            "logical byte total")
    require(sum(item["transport_bytes"] for item in assets) == provenance["transport_total_bytes"],
            "transport byte total")
    print(json.dumps({"byte_identity": "pass", "checked_assets": checked,
                      "created_assets": created,
                      "logical_bytes": provenance["logical_total_bytes"],
                      "transport_bytes": provenance["transport_total_bytes"],
                      "numerical_oracle_or_native_execution": False}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (TransportError, OSError, KeyError, TypeError, ValueError) as error:
        print("reference transport failed: " + str(error), file=sys.stderr)
        sys.exit(1)
