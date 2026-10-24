#!/usr/bin/env python3
"""Replay every frozen Canadian N row through an attested scratch crate.

Only the copied src/bonds/canadian/numeric.rs gains the cfg(test) harness from
validation/harness/canadian_numeric_kernel.rs. Immutable fixture operands and
expectations are transported to three plain TSV files beside the scratch
Cargo.toml; integer shifts only re-serialize given dyadic tuples and no
producer financial helper is imported. Original sources, the frozen fixture,
public APIs and package membership are never modified. Evidence records the
source manifest, fixture/harness/runner hashes, toolchain, per-command output
hashes and a post-run original-byte guard. Python >=3.9, stdlib only.
"""

import argparse
import collections
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
HARNESS = ROOT / "validation/harness/canadian_numeric_kernel.rs"
FIXTURE = Path("validation/fixtures/bonds/canadian-numeric-oracle.fixture")
FIXTURE_SHA = "77e0532a644ad137b0775a06fd72dd8fa243081a94b998355a9067ff993dae10"
CONTRACT = Path("docs/M8_CANADIAN_CONTRACT.md")
CONTRACT_SHA = "4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3"
PRODUCTION_MODULE = Path("src/bonds/canadian/numeric.rs")
TEST_FILTER = "bonds::canadian::numeric::frozen_"
TEST_NAMES = (
    "bonds::canadian::numeric::frozen_numeric_replay::all_frozen_primitive_rows_bind_actual_functions",
    "bonds::canadian::numeric::frozen_dated_replay::all_dated_frozen_inputs_bind_source_and_actual_traversal",
)
QUANTITIES = ("price derivative", "price second derivative", "time weighted amount",
              "modified duration", "macaulay duration", "convexity")
PRIMITIVE_ROWS, PRIMITIVE_OPS = 2327, 17
DATED_COUNTS = {"dated_mm": 782, "dated_compound_seeded": 437}
DATED_FLOWS = 3388


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical_json(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def hx(n):
    return ("-" if n < 0 else "") + format(abs(n), "x")


def observation(x):
    return [x["bits"], x["class"]]


def retained(terms):
    # Exact signed q*2^(e+8192) integers; zero-sign terms stay literal zero.
    out = []
    for sign, q, exponent in terms:
        sign, q, exponent = int(sign), int(q), int(exponent)
        if sign and exponent + 8192 < 0:
            raise ValueError("retained term below the transported 2^-8192 grid")
        out.append(hx(sign * (q << (exponent + 8192))) if sign else "0")
    return out


def fixture_rows(data):
    header, separator, body = data.partition(b"\n\n")
    if not separator:
        raise ValueError("fixture needs standard metadata/blank-line/body layout")
    lines = body.decode("utf-8").splitlines()
    columns = lines[0].split("\t")
    for line in lines[1:]:
        cells = line.split("\t")
        if len(cells) != len(columns):
            raise ValueError("fixture row width differs from header")
        yield dict(zip(columns, cells))


def primitive_fields(op, label, v, e):
    fields = [op, label]
    if op == "log_argument":
        fields += [v["frequency"], v["yield_bits"], e["branch"], e["argument_or_identity_bits"] or "-"]
    elif op == "log_scale":
        fields += [v["frequency"], v["ell_bits"]] + observation(e)
    elif op in ("exponent", "tiny_bound"):
        fields += [v["frequency"], v.get("g_bits", v.get("yield_bits")), v["lambda_num"], v["lambda_den"]]
        fields += observation(e if op == "exponent" else e["exponent"])
    elif op == "guard":
        fields += [v.get("frequency", "2"), v.get("yield_bits", "0000000000000000"),
                   v["z_bits"], str(e["boundary"]).lower(), e["next_exp_calls"]]
    elif op == "guard_box":
        ex = min(int(v["interval_exp"]), int(v["box_exp"]))
        for name, original in (("interval_lo_num", "interval_exp"), ("interval_hi_num", "interval_exp"),
                               ("box_lo_num", "box_exp"), ("box_hi_num", "box_exp")):
            fields.append(hx(int(v[name]) << (int(v[original]) - ex)))
        fields.append(str(e["closed_intersection"]).lower())
    elif op == "elementary":
        lo, hi, ex = e["allowed"]
        fields += [v["function"], v["argument_bits"], hx(int(lo)), hx(int(hi)), str(ex)]
    elif op == "mm_affine":
        fields += [v["yield_bits"], v["days_total"], v["days_remaining"], str(e["domain"]).lower(),
                   hx(int(e["den_raw"])), hx(int(e["num_raw"]))]
        fields += observation(e["factor"]) if e["factor"] is not None else ["-", "-"]
    elif op in ("pv", "source_ieee"):
        fields += [hx(int(v["source_num"])), hx(int(v["source_den"])),
                   v.get("factor_bits", "3ff0000000000000")] + observation(e)
    elif op == "extended":
        fields += [hx(int(v["num"])), hx(int(v["den"])), v["binary_exp"]] + e["extended"]
    elif op == "risk_terms":
        fields += [v["convention"], v.get("frequency", "2"), v["yield_bits"], v["pv_bits"],
                   v.get("lambda_num", "1"), v.get("lambda_den", "1"),
                   v.get("days_total", "1"), v.get("days_remaining", "0")]
        fields += retained(e["terms"])
    elif op == "retained":
        fields.append(v["dirty_bits"])
        for terms, expected in zip(v["terms"], e["retained_hex"]):
            # Input encoding only; the expected column is never used as input.
            n = sum(int(s) * (int(q) << (int(x) + 8192)) for s, q, x in terms if int(s))
            if hx(n) != expected:
                raise ValueError("retained input encoding disagrees with frozen sum: " + label)
            fields.append(hx(n))
        for name in QUANTITIES:
            fields += observation(e["views"][name])
    elif op == "dirty_sum":
        fields += [",".join(v["pv_bits"])] + observation(e)
    elif op == "price_bases":
        fields += [v["convention"], v["dirty_bits"], v.get("a_bits", "0000000000000000"), v["ai_bits"]]
        for name in ("ModelDirty", "Clean", "SettlementDirty"):
            fields += observation(e[name])
    elif op == "target":
        fields += [v["convention"], v["basis"], v["value_bits"],
                   v.get("quote_accrued_bits", v.get("a_bits", "0000000000000000")),
                   v.get("ai_bits", "0000000000000000")]
        fields += observation(e["resolved"]) + [str(e["positive"]).lower()]
    elif op == "residual":
        fields += [v["dirty_bits"], v["target_bits"], e["scale"]]
        fields += observation(e["amount_residual"]) + observation(e["callback"])
    else:
        raise ValueError("unknown primitive N operation: " + op)
    return fields


def geometry(g):
    fragments = "~".join(",".join((x["cell_left"], x["cell_right"], x["days"], x["cell_days"]))
                         for x in g["fragments"]) or "-"
    return "|".join([g["start"], g["end"], g["full"], *g["left_pair"], *g["right_pair"], fragments])


def dated_case(op, label, v, e, flows):
    case = {"operation": op, "id": label}
    for key, value in v["definition"].items():
        if key == "selection":
            case["d_selection"] = value["kind"]
            case["d_explicit_convention"] = value.get("convention", "-")
        else:
            case["d_" + key] = value
    case["d_yield_bits"] = v["yield_bits"]
    case["d_g_bits"] = v.get("g_bits", "-")
    case["d_factor_bits"] = ",".join(v.get("factor_bits", [])) or "-"
    case.update(
        e_quasi_dates=",".join(e["quasi_dates"]), e_actual_dates=",".join(e["actual_dates"]),
        e_future_ordinals=",".join(e["future_ordinals"]), e_convention=e["convention"],
        e_a_bits=e["a"]["bits"], e_ai_bits=e["ai"]["bits"],
        e_valuation_num=hx(int(e["valuation_accrued"]["num"])),
        e_valuation_den=hx(int(e["valuation_accrued"]["den"])),
        e_settlement_num=hx(int(e["settlement_accrued"]["num"])),
        e_settlement_den=hx(int(e["settlement_accrued"]["den"])),
        e_dirty_bits=e["dirty"]["bits"], e_dirty_class=e["dirty"]["class"],
        e_accrued_geometry=geometry(e["accrual_geometry"]), e_status=e["status"])
    for k, name in enumerate(("ModelDirty", "Clean", "SettlementDirty")):
        case["e_basis%d_bits" % k] = e["bases"][name]["bits"]
        case["e_basis%d_class" % k] = e["bases"][name]["class"]
    for k, name in enumerate(QUANTITIES):
        case["e_view%d_bits" % k] = e["risk"]["views"][name]["bits"]
        case["e_view%d_class" % k] = e["risk"]["views"][name]["class"]
    for k, n in enumerate(e["risk"]["retained_hex"]):
        case["e_retained%d" % k] = n
    for flow in e["flows"]:
        f = dict(id=label, ordinal=flow["original_ordinal"], contractual_date=flow["contractual_date"],
                 payment_date=flow["payment_date"], source_num=hx(int(flow["source"]["num"])),
                 source_den=hx(int(flow["source"]["den"])), lambda_num=flow["lambda"]["num"],
                 lambda_den=flow["lambda"]["den"], factor_bits=flow["factor"]["bits"],
                 pv_bits=flow["pv"]["bits"], pv_class=flow["pv"]["class"],
                 coupon_geometry=geometry(flow["coupon_geometry"]))
        for k, value in enumerate(retained(flow["terms"])):
            f["retained%d" % k] = value
        if "exponent" in flow:
            f.update(exponent_bits=flow["exponent"]["bits"], affine_den="-", affine_num="-")
        else:
            f.update(exponent_bits="-", affine_den=hx(int(flow["affine"]["den"])),
                     affine_num=hx(int(flow["affine"]["num"])))
        flows.append(f)
    return case


def table(rows):
    header = list(rows[0])
    if any(list(row) != header for row in rows):
        raise ValueError("transported dated rows need one common column order")
    out = ["\t".join(header)]
    for row in rows:
        out.append("\t".join(row[key] for key in header))
    return ("\n".join(out) + "\n").encode()


def transport(data):
    primitive, cases, flows = [], [], []
    counts = collections.Counter()
    for row in fixture_rows(data):
        op, label = row["operation"], row["id"]
        v, e = json.loads(row["input"]), json.loads(row["expected"])
        if op.startswith("dated_"):
            cases.append(dated_case(op, label, v, e, flows))
        else:
            primitive.append(primitive_fields(op, label, v, e))
        counts[op] += 1
    for fields in primitive + [list(r.values()) for r in cases + flows]:
        if any(type(x) is not str or any(c in x for c in "\t\n\r") for x in fields):
            raise ValueError("transported cell must be a single-line string")
    primitive_counts = {k: n for k, n in counts.items() if not k.startswith("dated_")}
    dated_counts = {k: n for k, n in counts.items() if k.startswith("dated_")}
    if (len(primitive) != PRIMITIVE_ROWS or len(primitive_counts) != PRIMITIVE_OPS
            or dated_counts != DATED_COUNTS or len(flows) != DATED_FLOWS):
        raise ValueError("frozen N row/operation/flow census changed")
    files = {
        "primitive-inputs.tsv": "".join("\t".join(f) + "\n" for f in primitive).encode(),
        "dated-cases.tsv": table(cases),
        "dated-flows.tsv": table(flows),
    }
    return files, dict(counts)


def snapshot():
    paths = [Path(name) for name in ("Cargo.toml", "Cargo.lock", "clippy.toml", "README.md",
                                     "CHANGELOG.md", "LICENSE-MIT", "LICENSE-APACHE")]
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
    return [{"path": name, "bytes": len(data), "sha256": sha(data)} for name, data in sorted(files.items())]


def command(argv, cwd, env, destination, label, evidence):
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    stdout, stderr = destination / (label + ".stdout"), destination / (label + ".stderr")
    stdout.write_bytes(result.stdout)
    stderr.write_bytes(result.stderr)
    evidence.setdefault("commands", []).append({
        "label": label, "argv": argv, "cwd": str(cwd), "exit_code": result.returncode,
        "stdout": {"path": str(stdout), "bytes": len(result.stdout), "sha256": sha(result.stdout)},
        "stderr": {"path": str(stderr), "bytes": len(result.stderr), "sha256": sha(result.stderr)},
    })
    if result.returncode:
        raise RuntimeError("%s failed with exit%d; see %s and %s" % (label, result.returncode, stdout, stderr))
    return result.stdout


def check_originals(original, evidence):
    current = snapshot()
    changed = sorted(name for name in set(original) | set(current) if original.get(name) != current.get(name))
    evidence["original_files_unchanged"] = not changed
    evidence["original_changed_paths"] = changed
    return not changed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="new evidence directory outside the repository")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable on PATH or absolute path")
    parser.add_argument("--rustc", default="rustc", help="matching rustc executable")
    args = parser.parse_args()
    fixture_data = (ROOT / FIXTURE).read_bytes()
    if sha(fixture_data) != FIXTURE_SHA:
        parser.error("frozen Canadian numeric fixture bytes changed")
    contract_data = (ROOT / CONTRACT).read_bytes()
    if sha(contract_data) != CONTRACT_SHA:
        parser.error("adopted Canadian contract bytes changed")
    harness_data = HARNESS.read_bytes()
    runner_data = Path(__file__).read_bytes()
    original = snapshot()
    if args.output:
        destination = args.output.resolve()
        if destination == ROOT or ROOT in destination.parents:
            parser.error("evidence output must be outside the original repository")
        destination.mkdir(parents=True, exist_ok=False)
    else:
        destination = Path(tempfile.mkdtemp(prefix="sabiquant-canadian-numeric-kernel-"))
    crate = destination / "crate"
    evidence = {
        "schema": 1, "started_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "python": sys.version, "platform": platform.platform(), "output": str(destination),
        "crate": str(crate), "fixture": {"path": str(FIXTURE), "bytes": len(fixture_data), "sha256": FIXTURE_SHA},
        "contract": {"path": str(CONTRACT), "sha256": CONTRACT_SHA},
        "runner": {"path": str(Path(__file__).resolve()), "bytes": len(runner_data), "sha256": sha(runner_data)},
        "harness": {"path": str(HARNESS), "bytes": len(harness_data), "sha256": sha(harness_data)},
        "test_names": list(TEST_NAMES), "source_manifest": manifest(original),
        "source_manifest_sha256": sha(canonical_json(manifest(original))),
        "qualification": "Private copied actual numeric module plus appended cfg(test) harness; "
                         "all frozen N input rows routed to actual stages. Dated MM rows also execute "
                         "public evaluate/risk; dated Compound rows supply explicit g/exp factor inputs "
                         "through the original-ordinal observer, not native/public reachability. Tiny "
                         "linear/radius certificates remain reference-only proof fields. Native rows "
                         "validate predeclared conditional intervals or actual class on this host only.",
        "result": "incomplete",
    }
    failure = None
    try:
        tsv, counts = transport(fixture_data)
        evidence["transport"] = {"operation_counts": counts, "rows": sum(counts.values()), "flows": DATED_FLOWS,
                                 "files": {name: sha(data) for name, data in tsv.items()}}
        cargo, rustc = shutil.which(args.cargo), shutil.which(args.rustc)
        if not cargo or not rustc:
            raise ValueError("Cargo and rustc executables must exist")
        # Keep the invoked filename: rustup proxies dispatch on argv[0].
        cargo, rustc = str(Path(cargo).absolute()), str(Path(rustc).absolute())
        env = os.environ.copy()
        removed = [key for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "LLVM_PROFILE_FILE",
                                   "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER") if env.pop(key, None) is not None]
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
        for relative, data in original.items():
            copied = crate / relative
            copied.parent.mkdir(parents=True, exist_ok=True)
            copied.write_bytes(data)
        for name, data in tsv.items():
            (crate / name).write_bytes(data)
        source = original[str(PRODUCTION_MODULE)]
        if b"mod frozen_numeric_replay" in source or b"mod frozen_dated_replay" in source:
            raise ValueError("production already contains a frozen replay module")
        injected = source + b"\n" + harness_data
        (crate / PRODUCTION_MODULE).write_bytes(injected)
        evidence["injection"] = {"path": str(PRODUCTION_MODULE), "original_prefix_bytes": len(source),
                                 "original_prefix_sha256": sha(source), "injected_sha256": sha(injected),
                                 "cfg_test_only": True}
        listing = command([cargo, "package", "--list", "--allow-dirty", "--offline", "--locked"],
                          crate, env, destination, "package-list", evidence).decode().splitlines()
        if any(line.startswith("validation/") or line.endswith(".tsv") for line in listing):
            raise ValueError("validation fixture/harness/transport would be shipped by the package")
        evidence["package_excludes_validation_artifacts"] = True
        for profile, flags in (("debug", []), ("release", ["--release"])):
            stdout = command([cargo, "test", "--offline", "--locked", *flags, "--lib", TEST_FILTER,
                              "--", "--test-threads=1", "--nocapture"],
                             crate, env, destination, "kernel-test-" + profile, evidence).decode()
            if not re.search(r"test result: ok\. 2 passed; 0 failed; 0 ignored;", stdout):
                raise ValueError(profile + " replay did not execute exactly two nonignored tests")
            # --nocapture prints each census line between "test name ..." and "ok".
            for name in TEST_NAMES:
                if not re.search(re.escape("test %s ... " % name) + r"(?:ACTUAL_[^\n]*\n)?ok$", stdout, re.M):
                    raise ValueError(profile + " replay missing passing test " + name)
            census = re.findall(r"ACTUAL_[A-Z_]+ [^\n]*", stdout)
            if len(census) != 2:
                raise ValueError(profile + " replay did not print both census lines")
            evidence[profile] = {"passed": 2, "census": census, "stdout_sha256": sha(stdout.encode())}
        changed = [relative for relative, data in original.items() if (crate / relative).read_bytes() != data]
        if changed != [str(PRODUCTION_MODULE)] or (crate / PRODUCTION_MODULE).read_bytes() != injected:
            raise ValueError("unexpected copied source mutation")
        evidence["copied_source_changes"] = changed
        if (ROOT / FIXTURE).read_bytes() != fixture_data or HARNESS.read_bytes() != harness_data \
                or Path(__file__).read_bytes() != runner_data:
            raise ValueError("fixture/harness/runner changed during execution")
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
    print(json.dumps({"result": evidence["result"], "evidence": str(result),
                      "evidence_sha256": sha(result.read_bytes()),
                      "source_manifest_sha256": evidence["source_manifest_sha256"]}, sort_keys=True))
    if failure:
        raise SystemExit(str(failure))


if __name__ == "__main__":
    main()
