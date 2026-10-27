//! Repository policies that every `cargo test` run enforces.
//!
//! - NFR-09, ADR-0002 D1, ADR-0012 D1: the library declares no runtime or
//!   build dependency, no build script and no `links` key, and is not a
//!   procedural-macro crate.
//! - ADR-0012 D2.1, D2.2, D2.4: test-only crates appear only in the root
//!   `[dev-dependencies]` table, as crates.io version requirements that the
//!   allowlist of ADR-0012 D5 names.
//! - ADR-0012 D2.2, D2.6: every locked package other than SabiQuant comes from
//!   crates.io with a checksum, is reachable from SabiQuant's dev-dependencies,
//!   and is not a native-code crate.
//! - ADR-0012 D2.5: no allowlisted crate is named in `src/` or
//!   `tests/support/`, which the crate package ships.
//!
//! The parsers accept only the TOML and Markdown forms that the repository
//! uses and reject everything else: a construct they do not understand is an
//! error, never a pass. Extending them is a deliberate change to this file.
//!
//! CI adds the checks that need Cargo or the network (ADR-0012 D7). Clippy
//! enforces the other code policies (no global state, clock, I/O or output in
//! the library), as configured in `clippy.toml` and `Cargo.toml`.

use std::path::{Path, PathBuf};

const ADR_0012: &str = "docs/adr/ADR-0012-test-only-dev-dependencies.md";
const ALLOWLIST_BEGIN: &str = "<!-- policy:dev-dependency-allowlist:begin -->";
const ALLOWLIST_END: &str = "<!-- policy:dev-dependency-allowlist:end -->";
const ALLOWLIST_HEADER: &str = "| Crate | Version requirement | Category | Licence | Transitive dependencies | Maintenance | Advisories | Why in-house carries more risk | Owner approval |";
const ALLOWLIST_SEPARATOR: &str = "|---|---|---|---|---|---|---|---|---|";
const ALLOWLIST_CATEGORIES: [&str; 2] = ["property-testing", "benchmarking"];
const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";
const NATIVE_HELPERS: [&str; 5] = ["bindgen", "cc", "cmake", "pkg-config", "vcpkg"];

fn read(relative: &str) -> Result<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    #[expect(
        clippy::disallowed_methods,
        reason = "the policy tests read committed files (NFR-13)"
    )]
    let text = std::fs::read_to_string(&path);
    text.map_err(|error| format!("cannot read {}: {error}", path.display()))
}

/// Returns `line` without its TOML comment, if it has one. A `#` inside a
/// quoted string does not start a comment.
fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (index, c) in line.char_indices() {
        match quote {
            Some('"') if escaped => escaped = false,
            Some('"') if c == '\\' => escaped = true,
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '#' => return &line[..index],
            None => {}
        }
    }
    line
}

// ---------------------------------------------------------------------------
// A fail-closed TOML subset: tables, array tables, dotted keys, basic and
// literal single-line strings, booleans, decimal integers, arrays (also over
// several lines) and inline tables.

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Str(String),
    Bool(bool),
    Int(i64),
    Array(Vec<Value>),
    Table(Vec<(String, Value)>),
}

#[derive(Debug)]
enum Item {
    /// `[path]`, or `[[path]]` when `array` is set.
    Header { path: Vec<String>, array: bool },
    /// An assignment, with the enclosing table's path prepended to its key.
    Assign { path: Vec<String>, value: Value },
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn new(text: &str) -> Self {
        Self {
            chars: text.chars().collect(),
            at: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.at + offset).copied()
    }

    fn eat(&mut self, c: char) -> bool {
        let found = self.peek() == Some(c);
        if found {
            self.at += 1;
        }
        found
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.eat(c) {
            Ok(())
        } else {
            Err(format!("expected `{c}`"))
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.at += 1;
        }
    }

    fn at_end(&self) -> bool {
        self.at == self.chars.len()
    }

    fn expect_end(&mut self) -> Result<(), String> {
        self.skip_whitespace();
        if self.at_end() {
            Ok(())
        } else {
            Err("unexpected text after the value".to_owned())
        }
    }

    /// A bare or quoted key segment.
    fn key_segment(&mut self) -> Result<String, String> {
        match self.peek() {
            Some('"') => self.basic_string(),
            Some('\'') => self.literal_string(),
            _ => {
                let start = self.at;
                while self
                    .peek()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    self.at += 1;
                }
                if self.at == start {
                    Err("expected a key".to_owned())
                } else {
                    Ok(self.chars[start..self.at].iter().collect())
                }
            }
        }
    }

    /// A dotted key such as `a."b".c`.
    fn key_path(&mut self) -> Result<Vec<String>, String> {
        let mut path = Vec::new();
        loop {
            self.skip_whitespace();
            path.push(self.key_segment()?);
            self.skip_whitespace();
            if !self.eat('.') {
                return Ok(path);
            }
        }
    }

    fn basic_string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        if self.peek() == Some('"') && self.peek_at(1) == Some('"') {
            return Err("multi-line strings are not supported".to_owned());
        }
        let mut text = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => return Err("unterminated string".to_owned()),
                Some('"') => {
                    self.at += 1;
                    return Ok(text);
                }
                Some('\\') => {
                    let escaped = match self.peek_at(1) {
                        Some('\\') => '\\',
                        Some('"') => '"',
                        _ => return Err("unsupported string escape".to_owned()),
                    };
                    text.push(escaped);
                    self.at += 2;
                }
                Some(c) => {
                    text.push(c);
                    self.at += 1;
                }
            }
        }
    }

    fn literal_string(&mut self) -> Result<String, String> {
        self.expect('\'')?;
        if self.peek() == Some('\'') && self.peek_at(1) == Some('\'') {
            return Err("multi-line strings are not supported".to_owned());
        }
        let start = self.at;
        loop {
            match self.peek() {
                None | Some('\n') => return Err("unterminated string".to_owned()),
                Some('\'') => {
                    let text = self.chars[start..self.at].iter().collect();
                    self.at += 1;
                    return Ok(text);
                }
                Some(_) => self.at += 1,
            }
        }
    }

    fn keyword(&mut self, word: &str) -> bool {
        let end = self.at + word.chars().count();
        let matches = self
            .chars
            .get(self.at..end)
            .is_some_and(|found| found.iter().copied().eq(word.chars()));
        let bounded = !self
            .chars
            .get(end)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-');
        if matches && bounded {
            self.at = end;
        }
        matches && bounded
    }

    fn integer(&mut self) -> Result<i64, String> {
        let start = self.at;
        if matches!(self.peek(), Some('+' | '-')) {
            self.at += 1;
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.at += 1;
        }
        let text: String = self.chars[start..self.at].iter().collect();
        text.parse()
            .map_err(|_| format!("unsupported number `{text}`"))
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') => self.basic_string().map(Value::Str),
            Some('\'') => self.literal_string().map(Value::Str),
            Some('[') => self.array(),
            Some('{') => self.inline_table(),
            Some(c) if c == '+' || c == '-' || c.is_ascii_digit() => self.integer().map(Value::Int),
            _ if self.keyword("true") => Ok(Value::Bool(true)),
            _ if self.keyword("false") => Ok(Value::Bool(false)),
            _ => Err("unsupported value".to_owned()),
        }
    }

    fn array(&mut self) -> Result<Value, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        loop {
            self.skip_whitespace();
            if self.eat(']') {
                return Ok(Value::Array(items));
            }
            items.push(self.value()?);
            self.skip_whitespace();
            if !self.eat(',') {
                self.skip_whitespace();
                self.expect(']')?;
                return Ok(Value::Array(items));
            }
        }
    }

    fn inline_table(&mut self) -> Result<Value, String> {
        self.expect('{')?;
        let mut fields = Vec::new();
        loop {
            self.skip_whitespace();
            if self.eat('}') {
                return Ok(Value::Table(fields));
            }
            let key = self.key_segment()?;
            self.skip_whitespace();
            if self.peek() == Some('.') {
                return Err("dotted keys in inline tables are not supported".to_owned());
            }
            self.expect('=')?;
            fields.push((key, self.value()?));
            self.skip_whitespace();
            if !self.eat(',') {
                self.expect('}')?;
                return Ok(Value::Table(fields));
            }
        }
    }
}

/// The nesting depth of `[` and `{` outside strings at the end of `text`.
fn open_brackets(text: &str) -> Result<usize, String> {
    let mut depth = 0_usize;
    let mut quote = None;
    let mut escaped = false;
    for c in text.chars() {
        match quote {
            Some('"') if escaped => escaped = false,
            Some('"') if c == '\\' => escaped = true,
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '[' || c == '{' => depth += 1,
            None if c == ']' || c == '}' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| "unbalanced closing bracket".to_owned())?;
            }
            None => {}
        }
    }
    Ok(depth)
}

fn parse_header(statement: &str) -> Result<(Vec<String>, bool), String> {
    let mut parser = Parser::new(statement);
    parser.expect('[')?;
    let array = parser.eat('[');
    let path = parser.key_path()?;
    parser.expect(']')?;
    if array {
        parser.expect(']')?;
    }
    parser.expect_end()?;
    Ok((path, array))
}

fn parse_assignment(statement: &str) -> Result<(Vec<String>, Value), String> {
    let mut parser = Parser::new(statement);
    let key = parser.key_path()?;
    parser.expect('=')?;
    let value = parser.value()?;
    parser.expect_end()?;
    Ok((key, value))
}

/// Parses `text` into numbered items, failing on anything outside the subset.
fn parse_document(file: &str, text: &str) -> Result<Vec<(usize, Item)>, String> {
    let mut items = Vec::new();
    let mut table: Vec<String> = Vec::new();
    let mut lines = text.lines().enumerate();
    while let Some((index, raw)) = lines.next() {
        let number = index + 1;
        let at = |error: String| format!("{file}:{number}: {error}");
        let mut statement = strip_comment(raw).trim().to_owned();
        if statement.is_empty() {
            continue;
        }
        if statement.starts_with('[') {
            let (path, array) = parse_header(&statement).map_err(at)?;
            table.clone_from(&path);
            items.push((number, Item::Header { path, array }));
            continue;
        }
        while open_brackets(&statement).map_err(at)? > 0 {
            let Some((_, next)) = lines.next() else {
                return Err(at("unterminated array or inline table".to_owned()));
            };
            statement.push('\n');
            statement.push_str(strip_comment(next));
        }
        let (key, value) = parse_assignment(&statement).map_err(at)?;
        let mut path = table.clone();
        path.extend(key);
        items.push((number, Item::Assign { path, value }));
    }
    Ok(items)
}

// ---------------------------------------------------------------------------
// The manifest (ADR-0012 D1, D2.1, D2.2).

#[derive(Debug, PartialEq)]
struct Dependency {
    name: String,
    version: String,
}

fn valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Checks one table or key path of the manifest against the policy.
fn manifest_rule(path: &[String]) -> Result<(), String> {
    let root = path.first().map_or("", String::as_str);
    let second = path.get(1).map(String::as_str);
    match root {
        "dependencies" | "build-dependencies" | "build_dependencies" => Err(format!(
            "`{root}` declares a runtime or build dependency, which the library never has (ADR-0002, NFR-09)"
        )),
        "target" => Err(
            "target-specific tables are not allowed: every platform runs the same suite (ADR-0012 D2.1, NFR-04)"
                .to_owned(),
        ),
        "dev-dependencies" | "lints" | "profile" | "bin" | "test" | "bench" | "example" => Ok(()),
        "package" => match second {
            Some(key @ ("build" | "links" | "workspace")) => Err(format!(
                "`package.{key}` is not allowed: no build script, native library or workspace inheritance (ADR-0012 D1)"
            )),
            _ => Ok(()),
        },
        "lib" => match second {
            Some(key @ ("proc-macro" | "proc_macro" | "plugin")) => Err(format!(
                "`lib.{key}` is not allowed: the library is not a procedural-macro crate (ADR-0012 D1)"
            )),
            _ => Ok(()),
        },
        _ => Err(format!(
            "`{}` is outside the manifest subset the policy accepts (fail-closed; extend tests/policy.rs deliberately)",
            path.join(".")
        )),
    }
}

fn dev_dependency(name: &str, fields: &[(String, Value)]) -> Result<Dependency, String> {
    if !valid_crate_name(name) {
        return Err(format!("dev-dependency `{name}` has an invalid name"));
    }
    let mut version = None;
    let mut seen: Vec<&str> = Vec::new();
    for (key, value) in fields {
        if seen.contains(&key.as_str()) {
            return Err(format!("dev-dependency `{name}` sets `{key}` twice"));
        }
        seen.push(key);
        match (key.as_str(), value) {
            ("version", Value::Str(requirement)) if !requirement.trim().is_empty() => {
                version = Some(requirement.clone());
            }
            ("default-features" | "default_features", Value::Bool(_)) => {}
            ("features", Value::Array(features))
                if features.iter().all(|f| matches!(f, Value::Str(_))) => {}
            ("version" | "default-features" | "default_features" | "features", _) => {
                return Err(format!(
                    "dev-dependency `{name}`: `{key}` has an unsupported value"
                ));
            }
            _ => {
                return Err(format!(
                    "dev-dependency `{name}`: `{key}` is not allowed; only crates.io version requirements, `default-features` and `features` are (ADR-0012 D2.2)"
                ));
            }
        }
    }
    let version = version.ok_or_else(|| {
        format!("dev-dependency `{name}` has no crates.io version requirement (ADR-0012 D2.2)")
    })?;
    Ok(Dependency {
        name: name.to_owned(),
        version,
    })
}

/// Checks the whole manifest and returns its dev-dependencies, sorted by name.
fn manifest_dev_dependencies(text: &str) -> Result<Vec<Dependency>, String> {
    let mut entries: Vec<(String, Vec<(String, Value)>)> = Vec::new();
    for (line, item) in parse_document("Cargo.toml", text)? {
        let (path, value) = match item {
            Item::Header { path, .. } => (path, None),
            Item::Assign { path, value } => (path, Some(value)),
        };
        let at = |error: String| format!("Cargo.toml:{line}: {error}");
        manifest_rule(&path).map_err(at)?;
        let Some((root, rest)) = path.split_first() else {
            continue;
        };
        if root != "dev-dependencies" {
            continue;
        }
        let (name, fields) = match (rest, value) {
            ([], None) => continue,
            ([name], None) => (name, Vec::new()),
            ([name], Some(Value::Str(version))) => {
                (name, vec![("version".to_owned(), Value::Str(version))])
            }
            ([name], Some(Value::Table(fields))) => (name, fields),
            ([name, key], Some(value)) => (name, vec![(key.clone(), value)]),
            _ => {
                return Err(at(format!(
                    "unsupported dev-dependency form `{}`",
                    path.join(".")
                )));
            }
        };
        match entries.iter_mut().find(|(known, _)| known == name) {
            Some((_, known_fields)) => known_fields.extend(fields),
            None => entries.push((name.clone(), fields)),
        }
    }
    let mut dependencies = entries
        .iter()
        .map(|(name, fields)| dev_dependency(name, fields))
        .collect::<Result<Vec<_>, _>>()?;
    dependencies.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(dependencies)
}

// ---------------------------------------------------------------------------
// The allowlist (ADR-0012 D5).

fn is_iso_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 10
        && bytes[..10].iter().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                *byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
}

fn parse_allowlist(markdown: &str) -> Result<Vec<Dependency>, String> {
    let lines: Vec<&str> = markdown.lines().collect();
    let position = |marker: &str| -> Result<usize, String> {
        let found: Vec<usize> = (0..lines.len()).filter(|&i| lines[i] == marker).collect();
        match found.as_slice() {
            [index] => Ok(*index),
            _ => Err(format!(
                "{ADR_0012}: expected the marker `{marker}` exactly once, found it {} times",
                found.len()
            )),
        }
    };
    let begin = position(ALLOWLIST_BEGIN)?;
    let end = position(ALLOWLIST_END)?;
    let block = lines
        .get(begin + 1..end)
        .ok_or_else(|| format!("{ADR_0012}: the allowlist end marker precedes its begin marker"))?;
    let [header, separator, rows @ ..] = block else {
        return Err(format!("{ADR_0012}: the allowlist has no table header"));
    };
    if *header != ALLOWLIST_HEADER || *separator != ALLOWLIST_SEPARATOR {
        return Err(format!(
            "{ADR_0012}: the allowlist header must be exactly:\n{ALLOWLIST_HEADER}\n{ALLOWLIST_SEPARATOR}"
        ));
    }
    let mut approved: Vec<Dependency> = Vec::new();
    for (offset, row) in rows.iter().enumerate() {
        let line = begin + 4 + offset;
        let at = |error: &str| format!("{ADR_0012}:{line}: {error}: {row}");
        let inner = row
            .strip_prefix('|')
            .and_then(|rest| rest.strip_suffix('|'))
            .ok_or_else(|| at("an allowlist row starts and ends with `|`"))?;
        let cells: Vec<&str> = inner.split('|').map(str::trim).collect();
        let [name, version, category, _, _, _, _, _, approval] = cells.as_slice() else {
            return Err(at("an allowlist row has exactly nine cells"));
        };
        if cells.iter().any(|cell| cell.is_empty()) {
            return Err(at("every allowlist cell is filled in"));
        }
        let name = name.trim_matches('`');
        if !valid_crate_name(name) {
            return Err(at("invalid crate name"));
        }
        if !ALLOWLIST_CATEGORIES.contains(category) {
            return Err(at("the category is `property-testing` or `benchmarking`"));
        }
        if !is_iso_date(approval) {
            return Err(at("the owner approval starts with an ISO date"));
        }
        if approved.iter().any(|known| known.name == name) {
            return Err(at("the crate is listed twice"));
        }
        approved.push(Dependency {
            name: name.to_owned(),
            version: version.trim_matches('`').to_owned(),
        });
    }
    Ok(approved)
}

fn check_allowlisted(dependencies: &[Dependency], approved: &[Dependency]) -> Result<(), String> {
    for dependency in dependencies {
        match approved.iter().find(|known| known.name == dependency.name) {
            None => {
                return Err(format!(
                    "dev-dependency `{}` is not in the allowlist of {ADR_0012} (ADR-0012 D2.4, D4)",
                    dependency.name
                ));
            }
            Some(known) if known.version != dependency.version => {
                return Err(format!(
                    "dev-dependency `{}` requires `{}`, but the allowlist approves `{}` (ADR-0012 D2.4)",
                    dependency.name, dependency.version, known.version
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The lockfile (ADR-0012 D2.2, D2.6, D6).

#[derive(Debug, Default)]
struct LockedPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    dependencies: Vec<String>,
}

fn locked_package(fields: &[(String, Value)]) -> Result<LockedPackage, String> {
    let mut package = LockedPackage::default();
    let mut seen: Vec<&str> = Vec::new();
    for (key, value) in fields {
        if seen.contains(&key.as_str()) {
            return Err(format!("a locked package sets `{key}` twice"));
        }
        seen.push(key);
        match (key.as_str(), value) {
            ("name", Value::Str(text)) => package.name.clone_from(text),
            ("version", Value::Str(text)) => package.version.clone_from(text),
            ("source", Value::Str(text)) => package.source = Some(text.clone()),
            ("checksum", Value::Str(text)) => package.checksum = Some(text.clone()),
            ("dependencies", Value::Array(items)) => {
                for item in items {
                    let Value::Str(text) = item else {
                        return Err("a locked dependency is not a string".to_owned());
                    };
                    package.dependencies.push(text.clone());
                }
            }
            _ => return Err(format!("unexpected locked-package key `{key}`")),
        }
    }
    if package.name.is_empty() || package.version.is_empty() {
        return Err("a locked package lacks its name or version".to_owned());
    }
    Ok(package)
}

fn parse_lockfile(text: &str) -> Result<Vec<LockedPackage>, String> {
    let mut format = None;
    let mut packages: Vec<Vec<(String, Value)>> = Vec::new();
    for (line, item) in parse_document("Cargo.lock", text)? {
        match item {
            Item::Header { path, array: true } if path == ["package"] => packages.push(Vec::new()),
            Item::Header { path, .. } => {
                return Err(format!(
                    "Cargo.lock:{line}: unexpected table `{}`",
                    path.join(".")
                ));
            }
            Item::Assign { path, value } => match (path.as_slice(), packages.last_mut()) {
                ([key], None) if key == "version" && format.is_none() => format = Some(value),
                ([table, key], Some(fields)) if table == "package" => {
                    fields.push((key.clone(), value));
                }
                _ => {
                    return Err(format!(
                        "Cargo.lock:{line}: unexpected key `{}`",
                        path.join(".")
                    ));
                }
            },
        }
    }
    if format != Some(Value::Int(4)) {
        return Err("Cargo.lock: expected lockfile format `version = 4`".to_owned());
    }
    packages
        .iter()
        .map(|fields| locked_package(fields).map_err(|error| format!("Cargo.lock: {error}")))
        .collect()
}

fn is_native(name: &str) -> bool {
    name.ends_with("-sys") || name.ends_with("_sys") || NATIVE_HELPERS.contains(&name)
}

/// Whether the lockfile dependency entry `entry` ("name", "name version" or
/// "name version (source)") refers to `package`.
fn refers_to(entry: &str, package: &LockedPackage) -> bool {
    let mut words = entry.split_whitespace();
    words.next() == Some(package.name.as_str())
        && words
            .next()
            .is_none_or(|version| version == package.version)
}

fn check_lockfile(packages: &[LockedPackage], dependencies: &[Dependency]) -> Result<(), String> {
    let roots: Vec<&LockedPackage> = packages.iter().filter(|p| p.name == "sabiquant").collect();
    let [root] = roots.as_slice() else {
        return Err(format!(
            "Cargo.lock lists SabiQuant {} times instead of once",
            roots.len()
        ));
    };
    if root.source.is_some() || root.checksum.is_some() {
        return Err("Cargo.lock: SabiQuant is the local package and has no source".to_owned());
    }
    for package in packages.iter().filter(|p| p.name != "sabiquant") {
        let id = format!("{} {}", package.name, package.version);
        if package.source.as_deref() != Some(CRATES_IO) {
            return Err(format!(
                "Cargo.lock: `{id}` does not come from crates.io (ADR-0012 D2.2)"
            ));
        }
        let checksum_ok = package.checksum.as_deref().is_some_and(|sum| {
            sum.len() == 64
                && sum
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        });
        if !checksum_ok {
            return Err(format!(
                "Cargo.lock: `{id}` has no valid checksum (ADR-0012 D6)"
            ));
        }
        if is_native(&package.name) {
            return Err(format!(
                "Cargo.lock: `{id}` is a native-code crate (ADR-0012 D2.6)"
            ));
        }
    }
    let mut direct: Vec<&str> = root
        .dependencies
        .iter()
        .filter_map(|entry| entry.split_whitespace().next())
        .collect();
    direct.sort_unstable();
    let declared: Vec<&str> = dependencies.iter().map(|d| d.name.as_str()).collect();
    if direct != declared {
        return Err(format!(
            "Cargo.lock: SabiQuant's locked dependencies {direct:?} differ from the dev-dependencies {declared:?} of Cargo.toml"
        ));
    }
    let mut reached = vec![false; packages.len()];
    let mut pending: Vec<&LockedPackage> = vec![*root];
    while let Some(package) = pending.pop() {
        for entry in &package.dependencies {
            let mut matches = packages
                .iter()
                .enumerate()
                .filter(|(_, p)| refers_to(entry, p));
            let (Some((index, target)), None) = (matches.next(), matches.next()) else {
                return Err(format!(
                    "Cargo.lock: `{}` names `{entry}`, which matches no single package",
                    package.name
                ));
            };
            if !reached[index] {
                reached[index] = true;
                pending.push(target);
            }
        }
    }
    match packages
        .iter()
        .zip(&reached)
        .find(|(p, reached)| p.name != "sabiquant" && !**reached)
    {
        Some((orphan, _)) => Err(format!(
            "Cargo.lock: `{} {}` is not reachable from SabiQuant",
            orphan.name, orphan.version
        )),
        None => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// Placement (ADR-0012 D2.5).

/// Whether `source` uses `crate_name` as a path root (`name::…`) or imports
/// it (`use name`, `extern crate name`). A targeted text search, not
/// compiler enforcement.
fn names_crate(source: &str, crate_name: &str) -> bool {
    let ident = crate_name.replace('-', "_");
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    source.match_indices(&ident).any(|(index, _)| {
        let before = &source[..index];
        let after = &source[index + ident.len()..];
        let bounded = !before.chars().next_back().is_some_and(is_ident)
            && !after.chars().next().is_some_and(is_ident);
        let head = before.trim_end();
        let keyword_before = ["use", "crate"].iter().any(|keyword| {
            head.strip_suffix(keyword)
                .is_some_and(|rest| !rest.chars().next_back().is_some_and(is_ident))
        });
        bounded && (after.trim_start().starts_with("::") || keyword_before)
    })
}

fn rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("cannot list {}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("cannot list {}: {error}", directory.display()))?
            .path();
        if path.is_dir() {
            rust_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The repository's own files.

/// NFR-09, ADR-0002 D1, ADR-0012 D1, D2.1, D2.2, D2.4: the manifest declares
/// no runtime or build dependency, and every dev-dependency is allowlisted.
#[test]
fn manifest_declares_only_allowlisted_dev_dependencies() -> Result<(), String> {
    let dependencies = manifest_dev_dependencies(&read("Cargo.toml")?)?;
    let approved = parse_allowlist(&read(ADR_0012)?)?;
    check_allowlisted(&dependencies, &approved)
}

/// NFR-09, ADR-0012 D2.2, D2.6, D6: the lockfile matches the manifest, and
/// every other package is a reachable, checksummed crates.io crate without
/// native code.
#[test]
fn lockfile_matches_the_manifest_and_policy() -> Result<(), String> {
    let dependencies = manifest_dev_dependencies(&read("Cargo.toml")?)?;
    let packages = parse_lockfile(&read("Cargo.lock")?)?;
    check_lockfile(&packages, &dependencies)
}

/// ADR-0012 D2.5: no allowlisted crate is named in the packaged code.
#[test]
fn allowlisted_crates_stay_out_of_packaged_code() -> Result<(), String> {
    let approved = parse_allowlist(&read(ADR_0012)?)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for directory in ["src", "tests/support"] {
        rust_files(&root.join(directory), &mut files)?;
    }
    files.sort();
    if !files.contains(&root.join("src/lib.rs"))
        || !files.contains(&root.join("tests/support/mod.rs"))
    {
        return Err(format!("the placement scan missed known files: {files:?}"));
    }
    for file in &files {
        #[expect(
            clippy::disallowed_methods,
            reason = "the policy tests read committed files (NFR-13)"
        )]
        let source = std::fs::read_to_string(file)
            .map_err(|error| format!("cannot read {}: {error}", file.display()))?;
        if let Some(found) = approved.iter().find(|a| names_crate(&source, &a.name)) {
            return Err(format!(
                "{} names the test-only crate `{}` (ADR-0012 D2.5)",
                file.display(),
                found.name
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Self-tests of the checks above, on synthetic inputs.

#[test]
fn comments_are_stripped_outside_strings_only() {
    assert_eq!(strip_comment("a = 1 # dependencies"), "a = 1 ");
    assert_eq!(strip_comment("# [dependencies]"), "");
    assert_eq!(strip_comment(r#"a = "x # y" # z"#), r#"a = "x # y" "#);
    assert_eq!(strip_comment(r#"a = "x \" # y" # z"#), r#"a = "x \" # y" "#);
    assert_eq!(strip_comment("a = 'x # y' # z"), "a = 'x # y' ");
    assert_eq!(strip_comment("[dependencies]"), "[dependencies]");
}

const PACKAGE: &str = "[package]\nname = \"sabiquant\"\nversion = \"0.1.0\"\n";

fn manifest_error(extra: &str) -> String {
    match manifest_dev_dependencies(&format!("{PACKAGE}{extra}")) {
        Ok(found) => format!("accepted: {found:?}"),
        Err(error) => error,
    }
}

/// ADR-0012 D1, D2.1, D2.2: every form of a runtime, build, target-specific,
/// non-crates.io or build-script declaration is rejected.
#[test]
fn manifest_rejects_every_forbidden_dependency_form() {
    let cases = [
        ("[dependencies]\nx = \"1\"\n", "runtime or build dependency"),
        (
            "[dependencies.x]\nversion = \"1\"\n",
            "runtime or build dependency",
        ),
        (
            "[build-dependencies]\nx = \"1\"\n",
            "runtime or build dependency",
        ),
        (
            "[build_dependencies]\nx = \"1\"\n",
            "runtime or build dependency",
        ),
        ("[lints]\n[dependencies]\n", "runtime or build dependency"),
        (
            "[target.'cfg(unix)'.dev-dependencies]\nx = \"1\"\n",
            "target-specific",
        ),
        (
            "[target.x86_64-pc-windows-msvc.dependencies]\nx = \"1\"\n",
            "target-specific",
        ),
        ("[workspace]\n", "outside the manifest subset"),
        (
            "[workspace.dependencies]\nx = \"1\"\n",
            "outside the manifest subset",
        ),
        (
            "[patch.crates-io]\nx = { path = \"x\" }\n",
            "outside the manifest subset",
        ),
        ("[replace]\n", "outside the manifest subset"),
        ("[features]\nx = []\n", "outside the manifest subset"),
        (
            "[dev_dependencies]\nx = \"1\"\n",
            "outside the manifest subset",
        ),
        (
            "[dev-dependencies]\nx = { path = \"../x\" }\n",
            "`path` is not allowed",
        ),
        (
            "[dev-dependencies]\nx = { git = \"https://example.org/x\" }\n",
            "`git` is not allowed",
        ),
        (
            "[dev-dependencies]\nx = { workspace = true }\n",
            "`workspace` is not allowed",
        ),
        (
            "[dev-dependencies]\nx = { version = \"1\", registry = \"other\" }\n",
            "`registry` is not allowed",
        ),
        (
            "[dev-dependencies]\nx = { version = \"1\", package = \"y\" }\n",
            "`package` is not allowed",
        ),
        (
            "[dev-dependencies]\nx = { version = \"1\", optional = true }\n",
            "`optional` is not allowed",
        ),
        (
            "[dev-dependencies.x]\npath = \"x\"\n",
            "`path` is not allowed",
        ),
        (
            "[dev-dependencies]\nx.git = \"https://example.org/x\"\n",
            "`git` is not allowed",
        ),
        (
            "[dev-dependencies]\nx = { features = [\"a\"] }\n",
            "no crates.io version requirement",
        ),
        ("[dev-dependencies.x]\n", "no crates.io version requirement"),
        ("[dev-dependencies]\nx = \"\"\n", "unsupported value"),
        (
            "[dev-dependencies]\nx = 1\n",
            "unsupported dev-dependency form",
        ),
        (
            "[dev-dependencies]\nx = \"1\"\nx.version = \"2\"\n",
            "sets `version` twice",
        ),
        ("[dev-dependencies]\n\"x y\" = \"1\"\n", "invalid name"),
        (
            "[dev-dependencies]\nx.y.z = \"1\"\n",
            "unsupported dev-dependency form",
        ),
    ];
    for (extra, needle) in cases {
        let error = manifest_error(extra);
        assert!(error.contains(needle), "{extra:?}: {error}");
    }
    // Keys before the first table header belong to the document root.
    let root_cases = [
        (
            "dependencies = { x = \"1\" }\n",
            "runtime or build dependency",
        ),
        ("dependencies.x = \"1\"\n", "runtime or build dependency"),
        (
            "build-dependencies.x.version = \"1\"\n",
            "runtime or build dependency",
        ),
        ("target.x.dev-dependencies.y = \"1\"\n", "target-specific"),
        (
            "dev-dependencies = \"1\"\n",
            "unsupported dev-dependency form",
        ),
        ("build = \"build.rs\"\n", "outside the manifest subset"),
        ("cargo-features = []\n", "outside the manifest subset"),
    ];
    for (root, needle) in root_cases {
        let error = match manifest_dev_dependencies(&format!("{root}{PACKAGE}")) {
            Ok(found) => format!("accepted: {found:?}"),
            Err(error) => error,
        };
        assert!(error.contains(needle), "{root:?}: {error}");
    }
    for key in [
        "build = \"build.rs\"",
        "build = false",
        "links = \"z\"",
        "workspace = \"..\"",
    ] {
        let error = manifest_error(&format!("{key}\n"));
        assert!(error.contains("ADR-0012 D1"), "{key}: {error}");
    }
    for key in ["proc-macro = true", "proc_macro = true", "plugin = true"] {
        let error = manifest_error(&format!("[lib]\n{key}\n"));
        assert!(error.contains("procedural-macro"), "{key}: {error}");
    }
}

/// ADR-0012 D2.1, D2.2: the accepted spellings of a crates.io dev-dependency.
#[test]
fn manifest_accepts_crates_io_dev_dependencies() {
    let manifest = format!(
        "{PACKAGE}include = [\n    \"/src/**\", # sources\n    \"/Cargo.toml\",\n]\n\
         [lints.rust]\nunsafe_code = \"forbid\"\n[lints.clippy]\nx = {{ level = \"warn\", priority = -1 }}\n\
         [lib]\npath = 'src/lib.rs'\n[[test]]\nname = \"t\"\n\
         [dev-dependencies]\nb = \"1.2\"\nc = {{ version = \"0.4\", default-features = false, features = [\"std\"] }}\n\
         d.version = \">=1, <2\"\n[dev-dependencies.a]\nversion = \"=2.0.0\"\nfeatures = []\n"
    );
    let found = manifest_dev_dependencies(&manifest);
    let expected = [
        ("a", "=2.0.0"),
        ("b", "1.2"),
        ("c", "0.4"),
        ("d", ">=1, <2"),
    ]
    .map(|(name, version)| Dependency {
        name: name.to_owned(),
        version: version.to_owned(),
    });
    assert_eq!(found, Ok(expected.into()));
    assert_eq!(manifest_dev_dependencies(PACKAGE), Ok(Vec::new()));
}

/// Fail-closed parsing: syntax outside the subset is an error, never a pass.
#[test]
fn manifest_parser_fails_closed_on_unknown_syntax() {
    let cases = [
        ("x = 1.5\n", "unexpected text"),
        ("x = 1e3\n", "unexpected text"),
        ("x = 0x10\n", "unexpected text"),
        ("x = 1979-05-27\n", "unexpected text"),
        ("x = inf\n", "unsupported value"),
        ("x = \"\"\"a\"\"\"\n", "multi-line strings"),
        ("x = '''a'''\n", "multi-line strings"),
        ("x = \"a\\nb\"\n", "unsupported string escape"),
        ("x = \"open\n", "unterminated string"),
        ("x = [1, 2\n", "unterminated array"),
        ("x = ]\n", "unbalanced closing bracket"),
        ("x = { a.b = 1 }\n", "dotted keys in inline tables"),
        ("x = { a = 1 b = 2 }\n", "expected `}`"),
        ("x\n", "expected `=`"),
        ("= 1\n", "expected a key"),
        ("[package\n", "expected `]`"),
        ("[[bin]\n", "expected `]`"),
        ("[package] x = 1\n", "unexpected text"),
        ("x = truthy\n", "unsupported value"),
    ];
    for (extra, needle) in cases {
        let error = manifest_error(&format!("[package]\n{extra}"));
        assert!(error.contains(needle), "{extra:?}: {error}");
        assert!(error.starts_with("Cargo.toml:"), "{extra:?}: {error}");
    }
    let quoted = parse_assignment(r#""a.b".'c' = "x \" \\ y""#);
    assert_eq!(
        quoted,
        Ok((
            vec!["a.b".to_owned(), "c".to_owned()],
            Value::Str("x \" \\ y".to_owned())
        ))
    );
    assert_eq!(
        parse_assignment("x = [true, false, -3, [], {}]").map(|(_, value)| value),
        Ok(Value::Array(vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Int(-3),
            Value::Array(Vec::new()),
            Value::Table(Vec::new()),
        ]))
    );
}

fn allowlist(rows: &str) -> String {
    format!(
        "# ADR\n\n{ALLOWLIST_BEGIN}\n{ALLOWLIST_HEADER}\n{ALLOWLIST_SEPARATOR}\n{rows}{ALLOWLIST_END}\n"
    )
}

const ROW: &str = "| `proptest` | `1.5` | property-testing | MIT OR Apache-2.0 | 12 | Active | None | Shrinking | 2026-10-27, project owner |\n";

/// ADR-0012 D5: the allowlist table is parsed exactly, and anything else in
/// its block is rejected.
#[test]
fn allowlist_is_parsed_fail_closed() {
    assert_eq!(parse_allowlist(&allowlist("")), Ok(Vec::new()));
    let proptest = Dependency {
        name: "proptest".to_owned(),
        version: "1.5".to_owned(),
    };
    assert_eq!(parse_allowlist(&allowlist(ROW)), Ok(vec![proptest]));

    let bench = ROW
        .replace("proptest", "criterion")
        .replace("property-testing", "benchmarking");
    assert_eq!(
        parse_allowlist(&allowlist(&format!("{ROW}{bench}"))).map(|a| a.len()),
        Ok(2)
    );

    let failures = [
        (allowlist(&format!("{ROW}{ROW}")), "listed twice"),
        (
            allowlist(&ROW.replace("property-testing", "oracle")),
            "category",
        ),
        (
            allowlist(&ROW.replace("2026-10-27, project owner", "project owner")),
            "ISO date",
        ),
        (allowlist(&ROW.replace("| 12 |", "| |")), "filled in"),
        (allowlist(&ROW.replace("| 12 |", "|")), "nine cells"),
        (
            allowlist(&ROW.replacen("| `proptest`", "`proptest`", 1)),
            "starts and ends",
        ),
        (
            allowlist(&ROW.replace("`proptest`", "prop test")),
            "invalid crate name",
        ),
        (allowlist(&format!("\n{ROW}")), "starts and ends"),
        (
            allowlist(ROW).replace(ALLOWLIST_HEADER, "| Crate |"),
            "header",
        ),
        (
            allowlist(ROW).replace(ALLOWLIST_SEPARATOR, "|---|"),
            "header",
        ),
        (allowlist(ROW).replace(ALLOWLIST_END, ""), "exactly once"),
        (
            format!("{}{ALLOWLIST_BEGIN}\n", allowlist("")),
            "exactly once",
        ),
        (format!("{ALLOWLIST_END}\n{ALLOWLIST_BEGIN}\n"), "precedes"),
        (
            format!("{ALLOWLIST_BEGIN}\n{ALLOWLIST_END}\n"),
            "no table header",
        ),
    ];
    for (markdown, needle) in failures {
        match parse_allowlist(&markdown) {
            Ok(found) => panic!("accepted {found:?} from {markdown:?}"),
            Err(error) => assert!(error.contains(needle), "{markdown:?}: {error}"),
        }
    }
}

/// ADR-0012 D2.4: a dev-dependency needs an allowlist row with the same
/// version requirement.
#[test]
fn dev_dependencies_must_match_the_allowlist() {
    let dependency = |name: &str, version: &str| Dependency {
        name: name.to_owned(),
        version: version.to_owned(),
    };
    let approved = [dependency("proptest", "1.5")];
    assert_eq!(check_allowlisted(&[], &[]), Ok(()));
    assert_eq!(
        check_allowlisted(&[dependency("proptest", "1.5")], &approved),
        Ok(())
    );
    let unlisted = check_allowlisted(&[dependency("criterion", "0.5")], &approved);
    assert!(unlisted.is_err_and(|e| e.contains("not in the allowlist")));
    let unlisted = check_allowlisted(&[dependency("proptest", "1.5")], &[]);
    assert!(unlisted.is_err_and(|e| e.contains("not in the allowlist")));
    let drifted = check_allowlisted(&[dependency("proptest", "1.6")], &approved);
    assert!(drifted.is_err_and(|e| e.contains("approves `1.5`")));
}

const LOCK_HEAD: &str = "# This file is automatically @generated by Cargo.\nversion = 4\n\n";
const SUM: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn lock_package(name: &str, version: &str, dependencies: &[&str]) -> String {
    let mut text = format!(
        "[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsource = \"{CRATES_IO}\"\nchecksum = \"{SUM}\"\n"
    );
    if !dependencies.is_empty() {
        text.push_str("dependencies = [\n");
        for dependency in dependencies {
            text.push_str(&format!(" \"{dependency}\",\n"));
        }
        text.push_str("]\n");
    }
    text.push('\n');
    text
}

fn lock_root(dependencies: &[&str]) -> String {
    let mut text = "[[package]]\nname = \"sabiquant\"\nversion = \"0.1.0\"\n".to_owned();
    if !dependencies.is_empty() {
        let quoted: Vec<String> = dependencies.iter().map(|d| format!("\"{d}\"")).collect();
        text.push_str(&format!("dependencies = [{}]\n", quoted.join(", ")));
    }
    text.push('\n');
    text
}

fn lock_error(lockfile: &str, dependencies: &[&str]) -> String {
    let declared: Vec<Dependency> = dependencies
        .iter()
        .map(|name| Dependency {
            name: (*name).to_owned(),
            version: "1".to_owned(),
        })
        .collect();
    match parse_lockfile(lockfile).and_then(|packages| check_lockfile(&packages, &declared)) {
        Ok(()) => "accepted".to_owned(),
        Err(error) => error,
    }
}

/// ADR-0012 D2.2, D2.6, D6: lockfile sources, checksums, native code,
/// reachability and agreement with the manifest.
#[test]
fn lockfile_checks_reject_every_violation() {
    let alone = format!("{LOCK_HEAD}{}", lock_root(&[]));
    assert_eq!(lock_error(&alone, &[]), "accepted");
    let graph = format!(
        "{LOCK_HEAD}{}{}{}{}",
        lock_root(&["proptest"]),
        lock_package("proptest", "1.5.0", &["bitflags 2.0.0", "rand"]),
        lock_package("bitflags", "2.0.0", &[]),
        lock_package("rand", "0.9.0", &[]),
    );
    assert_eq!(lock_error(&graph, &["proptest"]), "accepted");

    let failures = [
        (lock_error(&graph, &[]), "differ from the dev-dependencies"),
        (
            lock_error(&alone, &["proptest"]),
            "differ from the dev-dependencies",
        ),
        (
            lock_error(
                &graph.replace(CRATES_IO, "git+https://example.org/x"),
                &["proptest"],
            ),
            "does not come from crates.io",
        ),
        (
            lock_error(
                &graph.replace(CRATES_IO, "sparse+https://example.org/"),
                &["proptest"],
            ),
            "does not come from crates.io",
        ),
        (
            lock_error(
                &graph.replacen(&format!("checksum = \"{SUM}\"\n"), "", 1),
                &["proptest"],
            ),
            "no valid checksum",
        ),
        (
            lock_error(&graph.replacen(SUM, &SUM.to_uppercase(), 1), &["proptest"]),
            "no valid checksum",
        ),
        (
            lock_error(&graph.replacen(SUM, "abc", 1), &["proptest"]),
            "no valid checksum",
        ),
        (
            lock_error(
                &graph
                    .replace("\"rand", "\"openssl-sys")
                    .replace("rand\"", "openssl-sys\""),
                &["proptest"],
            ),
            "native-code crate",
        ),
        (
            lock_error(
                &graph.replace("\"rand", "\"cc").replace("rand\"", "cc\""),
                &["proptest"],
            ),
            "native-code crate",
        ),
        (
            lock_error(
                &format!("{graph}{}", lock_package("orphan", "1.0.0", &[])),
                &["proptest"],
            ),
            "not reachable",
        ),
        (
            lock_error(
                &graph.replace("bitflags 2.0.0", "bitflags 9.9.9"),
                &["proptest"],
            ),
            "matches no single package",
        ),
        (
            lock_error(
                &format!("{graph}{}", lock_package("rand", "0.9.0", &[])),
                &["proptest"],
            ),
            "matches no single package",
        ),
        (
            lock_error(&format!("{alone}{}", lock_root(&[])), &[]),
            "2 times",
        ),
        (
            lock_error(
                &format!("{LOCK_HEAD}{}", lock_package("sabiquant", "0.1.0", &[])),
                &[],
            ),
            "has no source",
        ),
        (
            lock_error(&alone.replace("version = 4", "version = 3"), &[]),
            "format `version = 4`",
        ),
        (
            lock_error(&alone.replace("version = 4\n", ""), &[]),
            "format `version = 4`",
        ),
        (
            lock_error(&format!("{alone}[metadata]\n"), &[]),
            "unexpected table",
        ),
        (
            lock_error(&format!("{alone}[[patch.unused]]\n"), &[]),
            "unexpected table",
        ),
        (
            lock_error(
                &alone.replace(
                    "version = \"0.1.0\"",
                    "version = \"0.1.0\"\nreplace = \"x\"",
                ),
                &[],
            ),
            "unexpected locked-package key",
        ),
        (
            lock_error(
                &alone.replace(
                    "name = \"sabiquant\"\n",
                    "name = \"sabiquant\"\nname = \"x\"\n",
                ),
                &[],
            ),
            "twice",
        ),
        (
            lock_error(&format!("{LOCK_HEAD}[[package]]\nname = \"x\"\n"), &[]),
            "name or version",
        ),
    ];
    for (error, needle) in failures {
        assert!(error.contains(needle), "{needle}: {error}");
    }
}

/// ADR-0012 D2.5: crate paths and imports are found, unrelated words are not.
#[test]
fn crate_names_are_found_as_paths_and_imports() {
    for source in [
        "use proptest::prelude::*;",
        "fn f() { proptest::test_runner::Config::default(); }",
        "let x = ::proptest :: num;",
        "extern crate proptest;",
        "#[macro_use]\nextern crate proptest as p;",
        "use proptest;",
        "use\n    proptest::{x};",
    ] {
        assert!(names_crate(source, "proptest"), "{source}");
    }
    for source in [
        "the proptest crate is not used here",
        "use myproptest::x;",
        "use proptests::x;",
        "fn proptest() {}",
        "reuse proptest",
        "let a = b_proptest::c;",
    ] {
        assert!(!names_crate(source, "proptest"), "{source}");
    }
    assert!(names_crate(
        "use pretty_assertions::assert_eq;",
        "pretty-assertions"
    ));
    assert!(!names_crate(
        "pretty-assertions is a crate",
        "pretty-assertions"
    ));
}
