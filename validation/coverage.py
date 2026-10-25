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
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def rust_tokens(source):
    """Lex opaque Rust literals/comments and UTF8 byte spans, not Rust types.

    A token is (value, first_line, first_byte_column, last_line,
    last_byte_column_exclusive, first_character_offset, last_character_offset).
    The retained character offsets are only for reading a path string literal.
    LLVM's columns count UTF8 bytes, including for multibyte identifiers.
    """
    if source.startswith('\ufeff'):
        raise ValueError('UTF8 BOM source needs explicit column normalization')
    tokens = []
    cursor, line, column = 0, 1, 1
    while cursor < len(source):
        start, first_line, first_column = cursor, line, column
        char, value = source[cursor], None
        if char.isspace():
            cursor += 1
        elif source.startswith('//', cursor):
            end = source.find('\n', cursor)
            cursor = len(source) if end < 0 else end
        elif source.startswith('/*', cursor):
            cursor += 2
            depth = 1
            while depth and cursor < len(source):
                if source.startswith('/*', cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith('*/', cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            if depth:
                raise ValueError('unterminated Rust block comment')
        else:
            raw = re.match(r'(?:b|c)?r(\#{0,255})"', source[cursor:])
            if raw:
                end = source.find('"' + raw[1], cursor + raw.end())
                if end < 0:
                    raise ValueError('unterminated Rust raw string')
                cursor = end + 1 + len(raw[1])
                value = '<literal>'
            elif char == '"' or (char == "'" and cursor + 2 < len(source)
                                 and (source[cursor + 1] == '\\'
                                      or source[cursor + 2] == "'")):
                quote = char
                cursor += 1
                while cursor < len(source) and source[cursor] != quote:
                    cursor += 2 if source[cursor] == '\\' else 1
                if cursor >= len(source):
                    raise ValueError('unterminated Rust quoted literal')
                cursor += 1
                value = '<literal>'
            elif (source.startswith('r#', cursor) and cursor + 2 < len(source)
                  and (source[cursor + 2].isalpha() or source[cursor + 2] == '_')):
                # Raw identifiers have the same Rust name as their plain form.
                cursor += 3
                while cursor < len(source) and (source[cursor].isalnum()
                                                or source[cursor] == '_'):
                    cursor += 1
                value = source[start + 2:cursor]
            elif char.isalpha() or char == '_':
                cursor += 1
                while cursor < len(source) and (source[cursor].isalnum()
                                                or source[cursor] == '_'):
                    cursor += 1
                value = source[start:cursor]
            else:
                cursor += 1
                value = char
        fragment = source[start:cursor]
        newlines = fragment.count('\n')
        line += newlines
        column = (len(fragment.rsplit('\n', 1)[1].encode('utf-8')) + 1 if newlines
                  else column + len(fragment.encode('utf-8')))
        if value is not None:
            tokens.append((value, first_line, first_column, line, column, start, cursor))
    return tokens


def reject_symlink_components(path):
    """Refuse aliases before resolve discards the written path components."""
    path = pathlib.Path(path)
    if any(part.is_symlink() for part in (path, *path.parents)):
        raise ValueError('source/module path contains a symlink component')


def test_only_ranges(source, filename=None, root=None):
    """Locate half-open (line, UTF8 byte column) spans of test-only items.

    Supports literal cfg(test) and direct test attributes on fn/impl/mod items.
    Conditional lint attributes are harmless; other cfg_attr code generation,
    inner test cfgs, gated macros, const-generic signature braces and external
    test modules under src fail closed. External modules need explicit file
    context, an unconditional literal path, and a resolved tests subtree outside
    src without path/include redirects or symlinks.
    This deliberately does not infer general macro expansion or cfg semantics.
    """
    tokens = rust_tokens(source)
    values = [token[0] for token in tokens]
    matching, nesting, stack = {}, [], []
    pairs = {'(': ')', '[': ']', '{': '}'}
    for index, value in enumerate(values):
        nesting.append(sum(values[opening] == '{' for opening in stack))
        if value in pairs:
            stack.append(index)
        elif value in pairs.values():
            if not stack or pairs[values[stack[-1]]] != value:
                raise ValueError('mismatched Rust delimiter')
            begin = stack.pop()
            matching[begin] = index
    if stack:
        raise ValueError('unterminated Rust item/attribute')

    def attribute(index):
        inner = values[index:index + 2] == ['#', '!']
        begin = index + 2 if inner else index + 1
        if values[begin:begin + 1] != ['[']:
            return None
        return inner, begin, matching[begin]

    def comma_parts(first, last):
        parts, begin, at = [], first, first
        while at < last:
            if values[at] in pairs:
                at = matching[at] + 1
            elif values[at] == ',':
                parts.append((begin, at))
                at += 1
                begin = at
            else:
                at += 1
        if begin < last:
            parts.append((begin, last))
        return parts

    def inspect_attribute(begin, end, inner):
        body = values[begin + 1:end]
        if body[:1] == ['cfg_attr']:
            if body[1:2] != ['('] or matching.get(begin + 2) != end - 1:
                raise ValueError('unsupported conditional attribute')
            parts = comma_parts(begin + 3, end - 1)
            if len(parts) < 2:
                raise ValueError('conditional attribute has no effect')
            for first, last in parts[1:]:
                if (values[first:first + 1] not in
                        (['allow'], ['warn'], ['deny'], ['forbid'], ['expect'])
                        or values[first + 1:first + 2] != ['(']
                        or matching.get(first + 1) != last - 1):
                    raise ValueError('unsupported conditional code-generating attribute')
            return False
        gated = body == ['test'] or body == ['cfg', '(', 'test', ')']
        if body[:1] == ['cfg'] and 'test' in body and not gated:
            raise ValueError('unsupported test-related cfg expression')
        if inner and gated:
            raise ValueError('unsupported inner test attribute')
        return gated

    def external_module(attrs, at):
        if filename is None or root is None or nesting[at] != 0:
            raise ValueError('external test module needs top-level file/path context')
        paths = [(begin, end) for begin, end in attrs
                 if values[begin + 1:begin + 2] == ['path']]
        if len(paths) != 1:
            raise ValueError('external test module needs one explicit literal path')
        begin, end = paths[0]
        if values[begin + 2:end] != ['=', '<literal>']:
            raise ValueError('unsupported external test module path')
        literal = source[tokens[begin + 3][5]:tokens[begin + 3][6]]
        if re.fullmatch(r'"[^"\\\n]*"', literal):
            path = literal[1:-1]
        else:
            raw = re.fullmatch(r'r(\#{0,255})"(.*?)"\1', literal, re.DOTALL)
            if not raw or '\n' in raw[2]:
                raise ValueError('unsupported external test module path literal')
            path = raw[2]
        written = pathlib.Path(filename).parent / path
        reject_symlink_components(written)
        reject_symlink_components(filename)
        reject_symlink_components(root)
        resolved = written.resolve()
        source_root = (pathlib.Path(root).resolve() / 'src').resolve()
        try:
            resolved.relative_to(source_root)
        except ValueError:
            if not resolved.is_file():
                raise ValueError('external test module path does not name a file')
            try:
                resolved.relative_to(pathlib.Path(root).resolve() / 'tests')
            except ValueError:
                raise ValueError('external test modules must resolve inside repository tests')
            # Plain descendant modules remain outside src. Refuse path/include
            # redirects and symlinks rather than infer a general Rust module graph.
            for child in resolved.parent.rglob('*'):
                if child.is_symlink():
                    raise ValueError('external test subtree contains a symlink')
                if child.is_file() and child.suffix == '.rs':
                    parts = [t[0] for t in rust_tokens(child.read_text(encoding='utf-8'))]
                    if any(parts[pos:pos + 2] == ['include', '!']
                           or parts[pos:pos + 3] in (['#', '[', 'path'],
                                                      ['#', '[', 'cfg_attr'])
                           or parts[pos:pos + 4] == ['#', '!', '[', 'cfg_attr']
                           for pos in range(len(parts))):
                        raise ValueError('external test subtree contains an unsupported redirect')
        else:
            raise ValueError('external test modules under src need explicit subtree support')

    def reject_macro_gates(first, last):
        for pos in range(first, last):
            if values[pos] == '#' and attribute(pos):
                _, begin, _ = attribute(pos)
                if values[begin + 1:begin + 2] in (['cfg'], ['cfg_attr'], ['test']):
                    raise ValueError('test/configuration attributes in macro tokens are unsupported')

    ranges, index = [], 0
    while index < len(tokens):
        if (values[index:index + 2] == ['macro_rules', '!']
                and values[index + 3:index + 4] in (['{'], ['('], ['['])):
            end = matching[index + 3]
            reject_macro_gates(index + 4, end)
            index = end + 1
            continue
        if values[index] == '!' and values[index + 1:index + 2] in (['{'], ['('], ['[']):
            end = matching[index + 1]
            reject_macro_gates(index + 2, end)
            index = end + 1
            continue
        if values[index] != '#' or not attribute(index):
            index += 1
            continue
        first = index
        inner, begin, end = attribute(index)
        if inner:
            inspect_attribute(begin, end, True)
            index = end + 1
            continue
        gated, attrs = False, []
        while index < len(tokens) and values[index] == '#' and attribute(index):
            inner, begin, end = attribute(index)
            if inner:
                raise ValueError('inner attribute in outer item attributes')
            gated |= inspect_attribute(begin, end, False)
            attrs.append((begin, end))
            index = end + 1
        if not gated:
            continue
        item = index
        if values[item:item + 1] == ['pub']:
            item += 1
            if values[item:item + 1] == ['(']:
                item = matching[item] + 1
        if values[item:item + 1] not in (['fn'], ['impl'], ['mod']):
            raise ValueError('unsupported test-only item kind')
        kind, at, angles = values[item], item + 1, 0
        while at < len(tokens):
            value = values[at]
            if value in ('(', '['):
                at = matching[at] + 1
            elif value == '<':
                angles += 1
                at += 1
            elif value == '>' and values[at - 1] != '-':
                angles -= 1
                if angles < 0:
                    raise ValueError('unsupported test-only item signature')
                at += 1
            elif value == '{':
                if angles:
                    raise ValueError('unsupported const-generic signature body ambiguity')
                end = matching[at]
                for pos in range(at + 1, end):
                    if values[pos] == 'mod':
                        terminator = pos + (4 if values[pos + 1:pos + 3] == ['r', '#'] else 2)
                        if values[terminator:terminator + 1] == [';']:
                            raise ValueError('nested external module in test-only item')
                    if values[pos:pos + 2] == ['include', '!']:
                        raise ValueError('Rust include in test-only item')
                at = end
                break
            elif value == ';':
                if kind != 'mod' or angles:
                    raise ValueError('unsupported test-only item declaration')
                external_module(attrs, first)
                break
            else:
                at += 1
        if at == len(tokens):
            raise ValueError('test-only item has no body/terminator')
        ranges.append((tokens[first][1], tokens[first][2], tokens[at][3], tokens[at][4]))
        index = at + 1
    return ranges


def production_regions(coverage, root):
    """Merge LLVM regions using source-item spans, never a first-gate cutoff.

    Preflight every src Rust file, including files with no LLVM function rows,
    so an unsupported gate on an external module cannot escape validation.
    """
    reject_symlink_components(root)
    root = root.resolve()
    production, scopes = {}, {}
    reject_symlink_components(root / 'src')
    for path in sorted((root / 'src').rglob('*')):
        if path.is_symlink():
            raise ValueError('src preflight encountered a symlink')
        if path.is_file() and path.suffix == '.rs':
            scopes[str(path.relative_to(root))] = test_only_ranges(
                path.read_text(encoding='utf-8'), path, root)
    for data in coverage['data']:
        for function in data.get('functions', []):
            for region in function.get('regions', []):
                start, column, end, end_column, count, file_id, _, kind = region
                if kind != 0:
                    continue
                filename = pathlib.Path(function['filenames'][file_id]).resolve()
                try:
                    relative = filename.relative_to(root)
                except ValueError:
                    continue
                if relative.parts[:1] != ('src',):
                    continue
                key = str(relative)
                if key not in scopes:
                    scopes[key] = test_only_ranges(filename.read_text(encoding='utf-8'),
                                                   filename, root)
                point = start, column
                if any((first, first_column) <= point < (last, last_column)
                       for first, first_column, last, last_column in scopes[key]):
                    continue
                regions = production.setdefault(key, {})
                location = (start, column, end, end_column)
                regions[location] = max(regions.get(location, 0), count)
    report = {}
    for filename, regions in sorted(production.items()):
        missing = [list(location) for location, count in sorted(regions.items()) if count == 0]
        report[filename] = {'regions': len(regions), 'covered': len(regions) - len(missing),
                            'uncovered_regions': missing}
    return report, scopes


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

    # Regions are emitted by LLVM. Item ranges only determine test exclusion;
    # an early test-only helper must not hide later production functions.
    coverage = json.loads(exported)
    report, scopes = production_regions(coverage, ROOT)
    (destination / 'test-only-ranges.json').write_text(
        json.dumps(scopes, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    (destination / "production-regions.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(summary)
    print("Production regions (excluding embedded unit tests):")
    for filename, values in report.items():
        print("%s: %s/%s" % (filename, values["covered"], values["regions"]))
    print("Reports: " + str(destination))


if __name__ == "__main__":
    main()
