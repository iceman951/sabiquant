#!/usr/bin/env python3
"""Bounded attested US/AFB public observations, independently classified.

Reference observations are retained verbatim (signed integer counts, binary64
bits and errors). Local annotations use only the adopted contract and exact
integer/rational models. No production Rust or QuantLib implementation source
is inspected. One isolated worker/date-pair calls three public counters.
"""

import argparse
import calendar
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import datetime as dt
from fractions import Fraction
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "validation/fixtures/daycount/us-afb-probes.fixture"
CONTRACT = ROOT / "docs/M3_US_AFB_CONTRACT.md"
SOURCES = (ROOT / "docs/M3_THIRTY360_US_SOURCE_REVIEW.md", ROOT / "docs/M3_AFB_SOURCE_REVIEW.md")
HELPER = Path(__file__).with_name("schedule_probes.py")
US_HELPER = Path(__file__).with_name("daycount_us_oracles.py")
MIN, MAX = dt.date(1901, 1, 1), dt.date(2199, 12, 31)
COLUMNS = "id group counter start end reference_status reference_days reference_fraction reference_error local_days local_fraction numerator denominator whole_years residual_end residual_days residual_denominator classification reference_ulp".split()


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load pinned helper")
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


REFERENCE = module("frozen_daycount_reference_attestation", HELPER)
US = module("frozen_explicit_us_integer_oracle", US_HELPER)


def bits(value):
    return "0x%016x" % struct.unpack(">Q", struct.pack(">d", value))[0]


def real(text):
    return struct.unpack(">d", struct.pack(">Q", int(text, 16)))[0]


def ulps(a, b):
    def key(value):
        number = struct.unpack(">q", struct.pack(">d", value))[0]
        return -(1 << 63) - number if number < 0 else number
    return abs(key(a) - key(b))


def scenarios():
    result, seen = [], set()

    def add(group, start, end):
        start, end = str(start), str(end)
        if (start, end) not in seen:
            a, b = dt.date.fromisoformat(start), dt.date.fromisoformat(end)
            assert MIN <= a <= MAX and MIN <= b <= MAX
            result.append(dict(id="P%04d" % len(result), group=group, start=start, end=end))
            seen.add((start, end))

    for year in (1901, 2000, 2024, 2100, 2199):
        feb = dt.date(year, 2, calendar.monthrange(year, 2)[1])
        points = sorted({dt.date(year, month, day) for month, day in
                         ((1, 30), (1, 31), (2, 27), (2, 28), (3, 1), (3, 30), (3, 31), (12, 31))} | {feb})
        targets = (dt.date(year, 1, 31), feb, dt.date(year, 3, 31), dt.date(year, 12, 31))
        for i, point in enumerate(points):
            add("year-seams", point, point)
            for j, target in enumerate(targets):
                add("year-seams", *((point, target) if (i + j) % 2 else (target, point)))

    controls = (
        ("2024-02-28", "2025-02-28"), ("2023-02-28", "2024-02-29"),
        ("2023-03-01", "2024-02-29"), ("2024-02-29", "2025-02-28"),
        ("1999-02-28", "2000-02-28"), ("1999-02-28", "2000-02-29"),
        ("1999-03-01", "2000-02-29"), ("1999-03-01", "2001-02-28"),
        ("1996-02-28", "2000-02-28"), ("1996-02-29", "2004-02-28"),
        ("2099-02-28", "2100-02-28"), ("2096-02-29", "2104-02-28"),
        ("2020-02-28", "2024-02-28"), ("2020-02-29", "2024-02-29"),
        ("2023-01-15", "2023-03-31"), ("2023-01-30", "2023-01-31"),
        ("2023-02-28", "2023-03-01"), ("2024-02-29", "2024-03-01"),
    )
    for start, end in controls:
        add("annual-and-reversal-controls", start, end)
        add("annual-and-reversal-controls", end, start)
    edges = (MIN, MIN + dt.timedelta(days=1), dt.date(1901, 2, 28),
             dt.date(2199, 2, 28), MAX - dt.timedelta(days=1), MAX)
    for start in edges:
        for end in edges:
            add("reference-civil-limits", start, end)
    assert 500 <= len(result) * 3 <= 1000
    return result


def afb_model(start, end):
    first, remaining = sorted((start, end))
    years = 0
    while remaining.year > 1:
        year = remaining.year - 1
        day = min(remaining.day, calendar.monthrange(year, remaining.month)[1])
        if remaining.month == 2 and day == 28 and calendar.isleap(year):
            day = 29
        candidate = dt.date(year, remaining.month, day)
        if candidate < first:
            break
        remaining, years = candidate, years + 1
    days = (remaining - first).days
    denominator = 365
    for year in range(first.year, remaining.year + 1):
        if calendar.isleap(year) and first <= dt.date(year, 2, 29) < remaining:
            denominator = 366
    exact = Fraction(years * denominator + days, denominator)
    value = years + days / denominator
    if start > end:
        exact, value = -exact, -value
    if exact == 0:
        value = 0.0
    return dict(days=(end - start).days, fraction=bits(value), exact=exact,
                whole_years=years, residual_end=remaining.isoformat(), residual_days=days,
                residual_denominator=denominator)


def local_model(counter, start, end):
    if counter == "AFB":
        return afb_model(start, end)
    days = US.endpoint_oracle(start, end, counter == "USA")
    exact = Fraction(days, 360)
    return dict(days=days, fraction=bits(float(exact)), exact=exact)


def direct_signed_us(counter, start, end):
    # Reference-behavior hypothesis, independently checked against EVERY public
    # observation. Apply documented rules directly to the input order; unlike
    # the local signed contract this does not sort endpoints. Not a financial
    # definition or a source-code translation.
    d1, d2 = start.day, end.day
    if counter == "USA" and US.feb_end(start):
        if US.feb_end(end):
            d2 = 30
        d1 = 30
    if d2 == 31 and d1 >= 30:
        d2 = 30
    if d1 == 31:
        d1 = 30
    return 360 * (end.year - start.year) + 30 * (end.month - start.month) + d2 - d1


def worker(case, wheel):
    manifest, files = REFERENCE.attest(wheel)
    import QuantLib as ql
    if ql.__version__ != "1.43":
        raise ValueError("runtime version changed")
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))
    assert MIN <= start <= MAX and MIN <= end <= MAX
    a, b = ql.Date(start.day, start.month, start.year), ql.Date(end.day, end.month, end.year)
    counters = (("USA", ql.Thirty360(ql.Thirty360.USA)),
                ("BondBasis", ql.Thirty360(ql.Thirty360.BondBasis)),
                ("AFB", ql.ActualActual(ql.ActualActual.AFB)))
    observations = []
    for name, counter in counters:
        count = counter.dayCount(a, b)
        try:
            fraction = counter.yearFraction(a, b)
            observation = dict(counter=name, status="value", days=count, fraction=bits(fraction), error="")
        except RuntimeError as error:
            observation = dict(counter=name, status="raises", days=count, fraction="", error=str(error))
        observations.append(observation)
    return dict(runtime="1.43", manifest=manifest, files=files, observations=observations)


def classify(case, observation):
    start, end = (dt.date.fromisoformat(case[key]) for key in ("start", "end"))
    counter = observation["counter"]
    local = local_model(counter, start, end)
    if observation["status"] != "value":
        # The public AFB runtime proposes another preceding year before its
        # stopping comparison. A retained residual in 1901 can therefore
        # attempt year 1900. Pin the observed error and independent residual,
        # rather than treating a local valid date pair as rejected input.
        if (counter == "AFB" and local["residual_end"].startswith("1901-")
                and observation["days"] == local["days"]
                and observation["error"] == "year 1900 out of bound. It must be in [1901,2199]"):
            return "AFB-reference-lower-anchor", local, ""
        raise ValueError("UNCLASSIFIED reference error: " + str((case, observation)))
    reference = real(observation["fraction"])
    expected = real(local["fraction"])
    distance = ulps(reference, expected)
    if counter != "AFB":
        assert observation["days"] == direct_signed_us(counter, start, end)
        assert observation["fraction"] == bits(observation["days"] / 360)
        if observation["days"] != local["days"]:
            assert start > end
            label = "US-direct-reverse-count"
        elif local["days"] == 0 and observation["fraction"] != local["fraction"]:
            assert reference == expected == 0.0
            label = "reference-signed-zero"
        else:
            assert distance == 0
            label = "compatible-E1"
    else:
        assert observation["days"] == local["days"]
        assert ulps(expected, float(local["exact"])) <= 4
        if reference == expected == 0.0 and observation["fraction"] != local["fraction"]:
            label = "reference-signed-zero"
        else:
            if distance > 8:
                raise ValueError("UNCLASSIFIED AFB difference: " + str((case, observation, local, distance)))
            label = "compatible-E2"
    return label, local, distance


def collect(wheel, manifest, files):
    environment = dict(os.environ)
    environment.pop("PYTHONPATH", None)
    environment.pop("PYTHONHOME", None)

    def observe(case):
        process = subprocess.run([sys.executable, "-I", str(Path(__file__).resolve()),
                                  "--worker", case["id"], "--quantlib-wheel", str(wheel.resolve())],
                                 capture_output=True, text=True, timeout=10,
                                 env=environment, check=False)
        if process.returncode or process.stderr:
            raise ValueError("probe worker failed: " + case["id"] + ": " + process.stderr)
        observed = json.loads(process.stdout)
        assert (observed["runtime"], observed["manifest"], observed["files"]) == ("1.43", manifest, files)
        result = []
        for observation in observed["observations"]:
            result.append((case, observation, *classify(case, observation)))
        assert tuple(item[1]["counter"] for item in result) == ("USA", "BondBasis", "AFB")
        return result

    result = []
    with ThreadPoolExecutor(max_workers=4) as executor:
        for group in executor.map(observe, scenarios()):
            result.extend(group)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quantlib-wheel", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--worker", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        case = next(case for case in scenarios() if case["id"] == args.worker)
        print(json.dumps(worker(case, args.quantlib_wheel), sort_keys=True, separators=(",", ":")))
        return
    manifest, files = REFERENCE.attest(args.quantlib_wheel)
    records = collect(args.quantlib_wheel, manifest, files)
    rows = ["\t".join(COLUMNS)]
    classes, counters, groups, directions = (Counter() for _ in range(4))
    maxima = Counter()
    for case, observed, label, local, distance in records:
        counter = observed["counter"]
        row = (case["id"] + "-" + counter, case["group"], counter, case["start"], case["end"],
               observed["status"], observed["days"], observed["fraction"], observed["error"],
               local["days"], local["fraction"], local["exact"].numerator, local["exact"].denominator,
               local.get("whole_years", ""), local.get("residual_end", ""), local.get("residual_days", ""),
               local.get("residual_denominator", ""), label, distance)
        rows.append("\t".join(map(str, row)))
        classes[counter + "/" + label] += 1
        counters[counter] += 1
        groups[case["group"]] += 1
        directions["equal" if case["start"] == case["end"] else "forward" if case["start"] < case["end"] else "reverse"] += 1
        if label.startswith("compatible"):
            maxima[counter] = max(maxima[counter], distance)
    body = ("\n".join(rows) + "\n").encode("utf-8")
    counts = lambda data: ";".join(f"{key}={value}" for key, value in sorted(data.items()))
    metadata = {
        "schema": "1", "fixture-set": "US-AFB-ATTESTED-REFERENCE-PROBES", "domain": "daycount.reference-probes",
        "description": str(len(records)) + " unchanged signed public observations; independent explicit-US and repeated-rollback AFB annotations, not financial oracles",
        "reference": "QuantLib", "reference-version": "1.43 (attested public worker runtime)",
        "reference-source": "Public Thirty360(USA), Thirty360(BondBasis), ActualActual(AFB), dayCount and yearFraction; no implementation source inspected",
        "tolerance": "compatible US TOL-E1 0ULP; compatible AFB <=8ULP reference versus local operation; local AFB <=4ULP rounded rational; signed/count differences retained separately",
        "reproduction": "generator-sha256=" + hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
                        + "; python3 -I validation/generator/daycount_us_afb_probes.py --quantlib-wheel " + args.quantlib_wheel.name,
        "adopted-contract-sha256": hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),
        "attestation-helper-sha256": hashlib.sha256(HELPER.read_bytes()).hexdigest(),
        "us-oracle-helper-sha256": hashlib.sha256(US_HELPER.read_bytes()).hexdigest(),
        "us-source-sha256": hashlib.sha256(SOURCES[0].read_bytes()).hexdigest(),
        "afb-source-sha256": hashlib.sha256(SOURCES[1].read_bytes()).hexdigest(),
        "contract-fnv64": US.fnv64(CONTRACT.read_bytes()),
        "generator-fnv64": US.fnv64(Path(__file__).read_bytes()),
        "attestation-helper-fnv64": US.fnv64(HELPER.read_bytes()),
        "us-oracle-helper-fnv64": US.fnv64(US_HELPER.read_bytes()),
        "us-source-fnv64": US.fnv64(SOURCES[0].read_bytes()), "afb-source-fnv64": US.fnv64(SOURCES[1].read_bytes()),
        "generated-at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "platform": platform.system() + " " + platform.machine(), "python-version": platform.python_version(),
        "quantlib-wheel": args.quantlib_wheel.name, "quantlib-wheel-sha256": REFERENCE.WHEEL_SHA256,
        "quantlib-module-files": str(files), "quantlib-module-manifest-sha256": manifest,
        "quantlib-source-baseline": "commit6b57206e04598f092efee66e3b367efc84771995; wheel source commit not independently attested",
        "execution-bound": "One isolated -I subprocess per datepair,10s timeout,4workers maximum; three public counters per worker; endpoints restricted1901-01-01..2199-12-31; timeout/failure/unclassified aborts before write",
        "scope": "USA mapped only to explicit EOMon; BondBasis behavioral comparison to explicit EOMoff; no universal instrument/ISO/FpML financial equivalence; full SIFMA source unread; no financial acceptance",
        "record-count": str(len(records)), "datepair-count": str(len(scenarios())),
        "counter-counts": counts(counters), "group-counts": counts(groups),
        "direction-counts": counts(directions), "classification-counts": counts(classes),
        "compatible-max-ulp": counts(maxima),
        "body-sha256": hashlib.sha256(body).hexdigest(), "body-fnv64": US.fnv64(body),
    }
    if args.check:
        old_header = DEST.read_text().split("\n\n", 1)[0]
        old = dict(line.split(": ", 1) for line in old_header.splitlines())
        for key in ("generated-at", "platform", "python-version"):
            metadata[key] = old[key]
    output = ("\n".join(key + ": " + value for key, value in metadata.items()) + "\n\n").encode("utf-8") + body
    if args.check:
        if DEST.read_bytes() != output:
            raise SystemExit("US/AFB public reference fixture changed; review raw observations")
        action = "verified"
    else:
        DEST.parent.mkdir(parents=True, exist_ok=True)
        DEST.write_bytes(output)
        action = "generated"
    print(action, DEST.relative_to(ROOT))
    for key in ("record-count", "datepair-count", "classification-counts", "compatible-max-ulp", "body-sha256"):
        print(key + ": " + metadata[key])


if __name__ == "__main__":
    main()
