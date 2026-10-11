#!/usr/bin/env python3
"""Measure Rust test coverage with the toolchain's native LLVM tools only.

No Cargo dependency is added. Uses instrument-coverage, cargo JSON artifacts,
and llvm-cov/llvm-profdata from the matching Rust llvm-tools component.
Exports JSON and HTML plus uncovered production source regions. Doc tests are
validated separately by cargo test; this runner instruments unit/integration
test executables and does not claim branch or exhaustive-input coverage.
"""

import argparse
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def run(command, env, output=None):
    result = subprocess.run(command, cwd=ROOT, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if output:
        output.write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode:
        sys.stderr.write(result.stdout + result.stderr)
        raise SystemExit(result.returncode)
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path,
                        help="new directory for build artifacts, profiles and reports")
    args = parser.parse_args()
    destination = args.output.resolve() if args.output else pathlib.Path(
        tempfile.mkdtemp(prefix="sabiquant-coverage-"))
    if args.output:
        # Never merge stale profiles or silently overwrite another report.
        destination.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    rustc = shutil.which("rustc")
    cargo = shutil.which("cargo")
    if not rustc or not cargo:
        parser.error("rustc and cargo must be on PATH")
    sysroot = pathlib.Path(run([rustc, "--print", "sysroot"], env).strip())
    verbose = run([rustc, "-Vv"], env)
    host = next(line.split(": ", 1)[1] for line in verbose.splitlines()
                if line.startswith("host: "))
    llvm_dir = sysroot / "lib" / "rustlib" / host / "bin"
    suffix = ".exe" if os.name == "nt" else ""
    profdata = llvm_dir / ("llvm-profdata" + suffix)
    cov = llvm_dir / ("llvm-cov" + suffix)
    if not profdata.exists() or not cov.exists():
        parser.error("matching LLVM tools missing; run rustup component add llvm-tools")
    profiles = destination / "raw"
    profiles.mkdir()
    env["CARGO_TARGET_DIR"] = str(destination / "target")
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + " -C instrument-coverage"
    env["LLVM_PROFILE_FILE"] = str(profiles / "%m-%p.profraw")
    artifacts = run([cargo, "test", "--locked", "--offline", "--no-run",
                     "--message-format=json"], env, destination / "build.log")
    binaries = []
    for line in artifacts.splitlines():
        message = json.loads(line)
        if (message.get("reason") == "compiler-artifact"
                and message.get("profile", {}).get("test")
                and message.get("executable")):
            binary = message["executable"]
            if binary not in binaries:
                binaries.append(binary)
    if not binaries:
        raise SystemExit("cargo emitted no test executables")
    for index, binary in enumerate(binaries):
        log = destination / ("tests-%02d.log" % index)
        stdout = run([binary, "--test-threads=1"], env, log)
        result = next(line for line in stdout.splitlines()
                      if line.startswith("test result:"))
        print(pathlib.Path(binary).name + ": " + result)
    raw = sorted(profiles.glob("*.profraw"))
    if not raw:
        raise SystemExit("test executions produced no coverage profiles")
    merged = destination / "tests.profdata"
    run([str(profdata), "merge", "-sparse", *map(str, raw), "-o", str(merged)], env)
    objects = [binaries[0]]
    for binary in binaries[1:]:
        objects.extend(["-object", binary])
    common = [*objects, "-instr-profile=" + str(merged),
              "-ignore-filename-regex=/rustc/|/tests/|/target/"]
    summary = run([str(cov), "report", *common], env)
    (destination / "summary.txt").write_text(summary, encoding="utf-8")
    exported = run([str(cov), "export", *common], env)
    (destination / "coverage.json").write_text(exported, encoding="utf-8")
    run([str(cov), "show", *common, "-format=html",
         "-output-dir=" + str(destination / "html")], env)

    # Exclude embedded #[cfg(test)] modules when reporting production regions.
    # Regions are emitted by LLVM, not guessed from line text. A function can
    # have multiple regions; line/region/function metrics are distinct.
    coverage = json.loads(exported)
    production = {}
    for data in coverage["data"]:
        for function in data.get("functions", []):
            for region in function.get("regions", []):
                start, column, end, end_column, count, file_id, _, kind = region
                if kind != 0:  # Ignore expansion, skipped, gap and branch regions.
                    continue
                filename = pathlib.Path(function["filenames"][file_id]).resolve()
                try:
                    relative = filename.relative_to(ROOT)
                except ValueError:
                    continue
                if not str(relative).startswith("src/"):
                    continue
                lines = filename.read_text(encoding="utf-8").splitlines()
                test_start = next((i + 1 for i, line in enumerate(lines)
                                   if line.strip() == "#[cfg(test)]"), len(lines) + 1)
                if start >= test_start:
                    continue
                key = str(relative)
                regions = production.setdefault(key, {})
                region_key = (start, column, end, end_column)
                regions[region_key] = max(regions.get(region_key, 0), count)
    report = {}
    for filename, regions in sorted(production.items()):
        missing = [list(location) for location, count in sorted(regions.items()) if count == 0]
        report[filename] = {"regions": len(regions), "covered": len(regions) - len(missing),
                            "uncovered_regions": missing}
    (destination / "production-regions.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(summary)
    print("Production regions (excluding embedded unit tests):")
    for filename, values in report.items():
        print("%s: %s/%s" % (filename, values["covered"], values["regions"]))
    print("Reports: " + str(destination))


if __name__ == "__main__":
    main()
