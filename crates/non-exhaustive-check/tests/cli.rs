#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("fixture cleanup failed for {}: {error}", self.0.display());
        }
    }
}

fn check(root: &Path, code: i32, diagnostic: &str) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_non-exhaustive-check"))
        .arg(root)
        .output()
        .expect("checker must execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = format!("{stdout}{stderr}");
    assert_eq!(output.status.code(), Some(code), "{text}");
    assert!(text.contains(diagnostic), "{text}");
    match code {
        0 => {
            assert!(
                stderr.is_empty(),
                "clean exit must emit nothing to stderr: {stderr}"
            );
            assert!(
                stdout.contains("0 violations"),
                "clean exit must emit summary to stdout"
            );
        }
        1 => {
            assert!(
                stderr.is_empty(),
                "violation exit must emit nothing to stderr: {stderr}"
            );
            assert!(
                stdout.contains("VIOLATION"),
                "violation exit must emit findings to stdout"
            );
        }
        2 => {
            assert!(
                stdout.is_empty(),
                "error exit must emit nothing to stdout: {stdout}"
            );
            assert!(
                !stderr.is_empty(),
                "error exit must emit diagnostic to stderr"
            );
        }
        unexpected => panic!("unsupported expected exit code: {unexpected}"),
    }
}

#[test]
fn cli_missing_arguments_exits_2() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_non-exhaustive-check"))
        .output()
        .expect("checker must execute");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
}

#[test]
fn cli_help_exits_0() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_non-exhaustive-check"))
        .arg("--help")
        .output()
        .expect("checker must execute");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:") || stdout.contains("USAGE:"));
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_version_exits_0() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_non-exhaustive-check"))
        .arg("--version")
        .output()
        .expect("checker must execute");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("non-exhaustive-check"));
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_plant_fail_revert_clean() {
    let root = std::env::temp_dir().join(format!(
        "gh-report-checker-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let _fixture = Fixture(root.clone());
    let file = root.join("lib.rs");
    std::fs::write(&file, "pub enum Clean { A }").unwrap();
    check(&root, 0, "0 violations");
    for (attribute, code) in [
        ("", 1),
        ("#[cfg(test)]", 0),
        ("#[cfg(not(test))]", 1),
        ("#[cfg(all(test, feature = \"x\"))]", 0),
        ("#[cfg(any(test, feature = \"x\"))]", 1),
        ("#[cfg(test, broken())]", 1),
    ] {
        std::fs::write(&file, format!("{attribute} #[derive(thiserror::Error)] #[non_exhaustive] pub enum PlantedError {{ A }}")).unwrap();
        check(
            &root,
            code,
            if code == 0 {
                "0 violations"
            } else {
                "PlantedError"
            },
        );
        std::fs::write(&file, "pub enum Clean { A }").unwrap();
        check(&root, 0, "0 violations");
    }
    std::fs::write(&file, "pub enum Broken {").unwrap();
    check(&root, 2, "failed to parse");
    std::fs::write(&file, "pub enum Clean { A }").unwrap();
    check(&root, 0, "0 violations");
    std::fs::remove_file(&file).unwrap();
    check(&root, 2, "no Rust sources");
    std::fs::remove_dir(&root).unwrap();
    check(&root, 2, "missing source directory");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(&file, "pub enum Clean { A }").unwrap();
    check(&root, 0, "0 violations");
}
