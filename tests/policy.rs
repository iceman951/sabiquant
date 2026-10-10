//! Repository policies that every `cargo test` run enforces.
//!
//! - NFR-09, ADR-0002 D1: the package declares no dependencies of any kind,
//!   and the lockfile contains SabiQuant alone.
//!
//! The checks are pure functions of the file text, so the tests can also show
//! that they catch every form of violation. Clippy enforces the other code
//! policies (no global state, clock, I/O or output in the library), as
//! configured in `clippy.toml` and `Cargo.toml`.

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

/// Returns the numbers of the manifest lines that mention dependencies
/// outside a comment. That covers every form a dependency can take:
/// `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`,
/// target-specific and workspace tables, and dotted or inline keys.
fn dependency_lines(manifest: &str) -> Vec<usize> {
    manifest
        .lines()
        .enumerate()
        .filter(|(_, line)| strip_comment(line).contains("dependencies"))
        .map(|(index, _)| index + 1)
        .collect()
}

/// Returns a description of the problem if the lockfile does not list
/// SabiQuant alone.
fn lockfile_problem(lockfile: &str) -> Option<String> {
    let packages = lockfile
        .lines()
        .filter(|line| line.trim() == "[[package]]")
        .count();
    let names: Vec<&str> = lockfile
        .lines()
        .filter_map(|line| line.strip_prefix("name = "))
        .collect();
    if packages == 1 && names == ["\"sabiquant\""] {
        None
    } else {
        Some(format!(
            "Cargo.lock must list SabiQuant alone, but lists {packages} packages: {names:?}"
        ))
    }
}

/// NFR-09, ADR-0002 D1: the manifest declares no dependencies.
#[test]
fn manifest_declares_no_dependencies() -> Result<(), String> {
    let manifest = read("Cargo.toml")?;
    match dependency_lines(&manifest).as_slice() {
        [] => Ok(()),
        lines => Err(format!(
            "Cargo.toml must not declare dependencies (ADR-0002), but lines {lines:?} do"
        )),
    }
}

/// NFR-09: the lockfile lists exactly one package, SabiQuant itself.
#[test]
fn lockfile_lists_only_this_package() -> Result<(), String> {
    let lockfile = read("Cargo.lock")?;
    lockfile_problem(&lockfile).map_or(Ok(()), Err)
}

/// The dependency check catches every form of dependency, and only those.
#[test]
fn the_dependency_check_catches_every_form() {
    for manifest in [
        "[dependencies]\nserde = \"1\"",
        "[dev-dependencies]\nproptest = \"1\"",
        "[build-dependencies]\ncc = \"1\"",
        "[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"",
        "[target.x86_64-pc-windows-msvc.dev-dependencies]\nwindows = \"0.58\"",
        "[workspace.dependencies]\nrand = \"0.8\"",
        "[dependencies.serde]\nversion = \"1\"",
        "dependencies.serde = \"1\"",
        "dev-dependencies = { proptest = \"1\" }",
        "[package]\nname = \"x\"\n\n[dependencies] # a comment after the table",
        "  [ dependencies ]",
    ] {
        assert!(!dependency_lines(manifest).is_empty(), "{manifest}");
    }
    for manifest in [
        "# [dependencies]",
        "[package]\n# serde = { dependencies }",
        "[lints.rust]\nunsafe_code = \"forbid\"",
        "",
    ] {
        assert!(dependency_lines(manifest).is_empty(), "{manifest}");
    }
    assert_eq!(
        dependency_lines("[package]\nname = \"x\"\n\n[dependencies]\n"),
        [4]
    );
}

/// The lockfile check catches any package besides SabiQuant.
#[test]
fn the_lockfile_check_catches_extra_packages() {
    let alone = "version = 4\n\n[[package]]\nname = \"sabiquant\"\nversion = \"0.0.0\"\n";
    assert_eq!(lockfile_problem(alone), None);
    let two = format!("{alone}\n[[package]]\nname = \"serde\"\nversion = \"1.0.228\"\n");
    assert!(lockfile_problem(&two).is_some());
    let renamed = alone.replace("sabiquant", "other");
    assert!(lockfile_problem(&renamed).is_some());
    assert!(lockfile_problem("version = 4\n").is_some());
    assert!(lockfile_problem("").is_some());
}

/// A file that cannot be read gives an error that names it, rather than
/// a passing check.
#[test]
fn an_unreadable_file_is_reported() {
    let error = read("tests/data/no-such-file.toml").expect_err("the file does not exist");
    assert!(
        error.starts_with("cannot read ") && error.contains("no-such-file.toml"),
        "{error}"
    );
}

#[test]
fn comments_are_stripped_outside_strings_only() {
    assert_eq!(strip_comment("a = 1 # dependencies"), "a = 1 ");
    assert_eq!(strip_comment("# [dependencies]"), "");
    assert_eq!(strip_comment(r#"a = "x # y" # z"#), r#"a = "x # y" "#);
    assert_eq!(strip_comment(r#"a = "x \" # y" # z"#), r#"a = "x \" # y" "#);
    assert_eq!(strip_comment(r#"a = "x\\" # z"#), r#"a = "x\\" "#);
    assert_eq!(strip_comment("a = 'x # y' # z"), "a = 'x # y' ");
    assert_eq!(strip_comment(r"a = 'x\' # z"), r"a = 'x\' ");
    assert_eq!(strip_comment(r#"a = "it's" # z"#), r#"a = "it's" "#);
    assert_eq!(strip_comment("[dependencies]"), "[dependencies]");
    assert_eq!(strip_comment(""), "");
}
