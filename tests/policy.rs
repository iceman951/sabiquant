//! Repository policies that every `cargo test` run enforces.
//!
//! - NFR-09, ADR-0002 D1: the package declares no dependencies of any kind,
//!   and the lockfile contains SabiQuant alone.
//!
//! Clippy enforces the other code policies (no global state, clock, I/O or
//! output in the library), as configured in `clippy.toml` and `Cargo.toml`.

use std::path::Path;

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

/// NFR-09, ADR-0002 D1: no line of the manifest outside a comment mentions
/// dependencies. That rules out every form a dependency can take:
/// `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`,
/// target-specific and workspace tables, and dotted or inline keys.
#[test]
fn manifest_declares_no_dependencies() -> Result<(), String> {
    let manifest = read("Cargo.toml")?;
    for (index, line) in manifest.lines().enumerate() {
        if strip_comment(line).contains("dependencies") {
            return Err(format!(
                "Cargo.toml:{}: the manifest must not declare dependencies (ADR-0002): {line}",
                index + 1
            ));
        }
    }
    Ok(())
}

/// NFR-09: the lockfile lists exactly one package, SabiQuant itself.
#[test]
fn lockfile_lists_only_this_package() -> Result<(), String> {
    let lockfile = read("Cargo.lock")?;
    let packages = lockfile
        .lines()
        .filter(|line| line.trim() == "[[package]]")
        .count();
    let names: Vec<&str> = lockfile
        .lines()
        .filter_map(|line| line.strip_prefix("name = "))
        .collect();
    if packages == 1 && names == ["\"sabiquant\""] {
        Ok(())
    } else {
        Err(format!(
            "Cargo.lock must list SabiQuant alone, but lists {packages} packages: {names:?}"
        ))
    }
}

#[test]
fn comments_are_stripped_outside_strings_only() {
    assert_eq!(strip_comment("a = 1 # dependencies"), "a = 1 ");
    assert_eq!(strip_comment("# [dependencies]"), "");
    assert_eq!(strip_comment(r#"a = "x # y" # z"#), r#"a = "x # y" "#);
    assert_eq!(strip_comment(r#"a = "x \" # y" # z"#), r#"a = "x \" # y" "#);
    assert_eq!(strip_comment("a = 'x # y' # z"), "a = 'x # y' ");
    assert_eq!(strip_comment("[dependencies]"), "[dependencies]");
}
