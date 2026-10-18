#!/usr/bin/env python3
"""Run a private bond yield/risk fixture against an attested temporary crate.

Only the copied yield_risk.rs gains a cfg(test) child module. Original sources,
old fixtures, public APIs and package membership are never modified. Evidence
includes exact original/copied source manifests, fixture/adapter/runner hashes,
toolchain and subprocess output hashes, and a post-run original-byte guard.
Requires Python >=3.9 and an installed Rust toolchain; uses stdlib only.
"""

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
ADAPTER = ROOT / "validation/harness/bond_yield_risk_kernel.rs"
DEFAULT_FIXTURE = ROOT / "validation/fixtures/bonds/yield-risk-extended.fixture"
INSTALLED_FIXTURE = Path("validation/fixtures/bonds/yield-risk-extended.fixture")
PRODUCTION_MODULE = Path("src/bonds/yield_risk.rs")
TEST_NAME = "bonds::yield_risk::validation_kernel::independent_extended_kernel_matches_all_rows"
CONTRACT = Path("docs/M8_BOND_YIELD_RISK_CONTRACT.md")
CONTRACT_SHA = "62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d"
INJECTION_START = b"\n// BEGIN TEMPORARY PRIVATE KERNEL VALIDATION ADAPTER\n#[cfg(test)]\nmod validation_kernel {\n"
INJECTION_END = b"\n}\n// END TEMPORARY PRIVATE KERNEL VALIDATION ADAPTER\n"
SCHEMA = (
    "id op family comp y_bits t_bits amount_bits dirty_bits target_bits numerators "
    "denominators pow2 negative recipes scale status sign significand exponent "
    "expected_bits error_kind error_phase error_index detail"
).split()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical_json(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def snapshot():
    paths = [Path(name) for name in (
        "Cargo.toml", "Cargo.lock", "clippy.toml", "README.md", "CHANGELOG.md",
        "LICENSE-MIT", "LICENSE-APACHE",
    )]
    for directory in (ROOT / "src", ROOT / "tests/support"):
        paths.extend(path.relative_to(ROOT) for path in directory.rglob("*") if path.is_file())
    result = {}
    for relative in sorted(set(paths)):
        path = ROOT / relative
        if path.is_symlink() or not path.is_file():
            raise ValueError("source must be a regular nonsymlink file: " + str(relative))
        result[str(relative)] = path.read_bytes()
    return result


def manifest(files):
    return [{"path": name, "bytes": len(data), "sha256": sha(data)}
            for name, data in sorted(files.items())]


def fixture_header(data):
    header, separator, body = data.partition(b"\n\n")
    if not separator:
        raise ValueError("fixture needs standard metadata/blank-line/body layout")
    fields = {}
    for line in header.decode("utf-8").splitlines():
        key, value = line.split(": ", 1)
        if not re.fullmatch(r"[a-z][a-z0-9-]*", key) or not value or value.strip() != value or any(ord(c) < 32 for c in value):
            raise ValueError("fixture header violates shared-reader key/value grammar: " + key)
        if key in fields:
            raise ValueError("duplicate fixture header key: " + key)
        fields[key] = value
    required = ("schema", "fixture-set", "domain", "description", "reference", "reference-version",
                "reference-source", "tolerance", "reproduction", "generated-at", "platform")
    if fields.get("schema") != "1" or any(key not in fields for key in required):
        raise ValueError("fixture does not supply the shared-reader schema1 required metadata")
    columns, _, records = body.partition(b"\n")
    if columns.decode().split("\t") != SCHEMA:
        raise ValueError("extended fixture does not match the frozen 24-column schema")
    if not records.endswith(b"\n"):
        raise ValueError("fixture records need final LF")
    if fields.get("body-sha256") not in (None, sha(body)):
        raise ValueError("fixture body SHA does not match metadata")
    if fields.get("records-sha256") not in (None, sha(records)):
        raise ValueError("fixture records SHA does not match metadata")
    count = len(records.splitlines())
    if int(fields.get("record-count", "0")) != count or not count:
        raise ValueError("fixture record count mismatch or empty corpus")
    return fields, {"bytes": len(data), "sha256": sha(data), "body_sha256": sha(body),
                    "records_sha256": sha(records), "record_count": count}


def command(command_args, cwd, env, destination, label, evidence):
    result = subprocess.run(command_args, cwd=cwd, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, check=False)
    stdout = destination / (label + ".stdout")
    stderr = destination / (label + ".stderr")
    stdout.write_bytes(result.stdout)
    stderr.write_bytes(result.stderr)
    evidence.setdefault("commands", []).append({
        "label": label, "argv": command_args, "cwd": str(cwd), "exit_code": result.returncode,
        "stdout": {"path": str(stdout), "bytes": len(result.stdout), "sha256": sha(result.stdout)},
        "stderr": {"path": str(stderr), "bytes": len(result.stderr), "sha256": sha(result.stderr)},
    })
    if result.returncode:
        raise RuntimeError("%s failed with exit%d; see %s and %s" %
                           (label, result.returncode, stdout, stderr))
    return result.stdout


def check_originals(original, evidence):
    current = snapshot()
    changed = sorted(name for name in set(original) | set(current)
                     if original.get(name) != current.get(name))
    evidence["original_files_unchanged"] = not changed
    evidence["original_changed_paths"] = changed
    return not changed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, default=DEFAULT_FIXTURE)
    parser.add_argument("--output", type=Path, help="new evidence directory outside the repository")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable on PATH or absolute path")
    parser.add_argument("--rustc", default="rustc", help="matching rustc executable")
    args = parser.parse_args()
    fixture_path = args.fixture.resolve()
    fixture_data = fixture_path.read_bytes()
    header, fixture_info = fixture_header(fixture_data)
    adapter_data = ADAPTER.read_bytes()
    runner_data = Path(__file__).read_bytes()
    contract_data = (ROOT / CONTRACT).read_bytes()
    if sha(contract_data) != CONTRACT_SHA:
        parser.error("adopted yield/risk contract bytes changed")
    original = snapshot()
    if args.output:
        destination = args.output.resolve()
        if destination == ROOT or ROOT in destination.parents:
            parser.error("evidence output must be outside the original repository")
        destination.mkdir(parents=True, exist_ok=False)
    else:
        destination = Path(tempfile.mkdtemp(prefix="sabiquant-yield-risk-kernel-"))
    crate = destination / "crate"
    evidence = {
        "schema": 1, "started_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "python": sys.version, "platform": platform.platform(), "output": str(destination),
        "crate": str(crate), "fixture": dict(fixture_info, original_path=str(fixture_path)),
        "fixture_header": header,
        "runner": {"path": str(Path(__file__).resolve()), "bytes": len(runner_data), "sha256": sha(runner_data)},
        "adapter": {"path": str(ADAPTER), "bytes": len(adapter_data), "sha256": sha(adapter_data)},
        "contract": {"path": str(CONTRACT), "sha256": sha(contract_data)},
        "test_name": TEST_NAME, "source_manifest": manifest(original),
        "source_manifest_sha256": sha(canonical_json(manifest(original))),
        "result": "incomplete",
    }
    failure = None
    try:
        cargo = shutil.which(args.cargo)
        rustc = shutil.which(args.rustc)
        if not cargo or not rustc:
            raise ValueError("Cargo and rustc executables must exist")
        # Keep the invoked filename: rustup proxies dispatch on argv[0], so
        # resolving a cargo/rustc symlink to the rustup executable is incorrect.
        cargo, rustc = str(Path(cargo).absolute()), str(Path(rustc).absolute())
        env = os.environ.copy()
        # Prevent prior coverage, target directories or injected rustc wrappers
        # from changing which crate/toolchain this isolated evidence executes.
        removed = []
        for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "LLVM_PROFILE_FILE", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
            if key in env:
                del env[key]
                removed.append(key)
        env.update({"CARGO_TARGET_DIR": str(destination / "target"), "CARGO_INCREMENTAL": "0",
                    "RUSTC": rustc, "RUSTFLAGS": "-D warnings", "RUST_BACKTRACE": "1"})
        evidence["environment"] = {"removed_keys": removed, "CARGO_TARGET_DIR": env["CARGO_TARGET_DIR"],
                                   "RUSTC": rustc, "RUSTFLAGS": "-D warnings", "CARGO_INCREMENTAL": "0"}
        toolchain = {}
        for name, executable, flags in (("rustc", rustc, ["-Vv"]), ("cargo", cargo, ["-V"])):
            version = command([executable, *flags], ROOT, env, destination, name + "-version", evidence)
            toolchain[name] = {"path": executable, "executable_sha256": sha(Path(executable).read_bytes()),
                               "version": version.decode(), "version_sha256": sha(version)}
        evidence["toolchain"] = toolchain
        evidence["toolchain_sha256"] = sha(canonical_json(toolchain))
        for relative, data in original.items():
            copied = crate / relative
            copied.parent.mkdir(parents=True, exist_ok=True)
            copied.write_bytes(data)
            if copied.read_bytes() != data:
                raise RuntimeError("copied source bytes differ: " + relative)
        evidence["copied_source_manifest_before_injection"] = manifest({
            relative: (crate / relative).read_bytes() for relative in original})
        assert evidence["copied_source_manifest_before_injection"] == evidence["source_manifest"]
        copied_module = crate / PRODUCTION_MODULE
        source = original[str(PRODUCTION_MODULE)]
        if b"mod validation_kernel" in source:
            raise ValueError("production already contains a validation_kernel module")
        injected = source + INJECTION_START + adapter_data + INJECTION_END
        copied_module.write_bytes(injected)
        assert copied_module.read_bytes()[:len(source)] == source
        evidence["injection"] = {"path": str(PRODUCTION_MODULE), "original_prefix_bytes": len(source),
                                 "original_prefix_sha256": sha(source), "injected_sha256": sha(injected),
                                 "appended_bytes": len(injected)-len(source), "cfg_test_only": True}
        installed = crate / INSTALLED_FIXTURE
        installed.parent.mkdir(parents=True, exist_ok=True)
        installed.write_bytes(fixture_data)
        assert installed.read_bytes() == fixture_data
        metadata_bytes = command([cargo, "metadata", "--offline", "--locked", "--format-version=1", "--no-deps"],
                                 crate, env, destination, "metadata", evidence)
        metadata = json.loads(metadata_bytes)
        if len(metadata["packages"]) != 1 or metadata["packages"][0]["dependencies"]:
            raise ValueError("copied crate is not a single std-only package")
        listing = command([cargo, "package", "--list", "--allow-dirty", "--offline", "--locked"],
                          crate, env, destination, "package-list", evidence).decode().splitlines()
        if any(line.startswith("validation/") for line in listing):
            raise ValueError("validation fixture/adapter would be shipped by the package manifest")
        evidence["package_excludes_validation_artifacts"] = True
        stdout = command([cargo, "test", "--offline", "--locked", "--release", "--lib", TEST_NAME,
                          "--", "--exact", "--test-threads=1", "--nocapture"],
                         crate, env, destination, "kernel-test", evidence).decode()
        if not re.search(r"test result: ok\. 1 passed; 0 failed; 0 ignored;", stdout):
            raise ValueError("filtered kernel test did not execute exactly one nonignored test")
        evidence["test_result_sha256"] = sha(stdout.encode())
        evidence["all_fixture_rows_checked"] = fixture_info["record_count"]
        # Only one copied source can differ, and it must be exactly the recorded
        # original prefix plus marked cfg(test) child; no edits to algorithms.
        changed = [relative for relative, data in original.items() if (crate / relative).read_bytes() != data]
        if changed != [str(PRODUCTION_MODULE)] or copied_module.read_bytes() != injected:
            raise ValueError("unexpected copied source mutation")
        evidence["copied_source_changes"] = changed
        if fixture_path.read_bytes() != fixture_data or ADAPTER.read_bytes() != adapter_data or Path(__file__).read_bytes() != runner_data:
            raise ValueError("fixture/adapter/runner changed during execution")
        if not check_originals(original, evidence):
            raise ValueError("original crate files changed while the check ran")
        evidence["result"] = "passed"
    except Exception as error:
        failure = error
        evidence["result"] = "failed"
        evidence["failure"] = {"type": type(error).__name__, "message": str(error)}
        check_originals(original, evidence)
    evidence["finished_at"] = dt.datetime.now(dt.timezone.utc).isoformat()
    result = destination / "evidence.json"
    result.write_bytes(json.dumps(evidence, indent=2, sort_keys=True).encode() + b"\n")
    print(json.dumps({"result": evidence["result"], "evidence": str(result), "evidence_sha256": sha(result.read_bytes()),
                      "source_manifest_sha256": evidence["source_manifest_sha256"], "fixture_sha256": fixture_info["sha256"]}, sort_keys=True))
    if failure:
        raise SystemExit(str(failure))


if __name__ == "__main__":
    main()
