//! Discovery authority exercised through the real conformer (0.0.0.13).
//!
//! The registry unit tests prove the discovery primitive itself; these
//! integration tests prove the downstream consumer behaves identically from
//! the repository root, nested crate directories, and deeper directories,
//! and fails closed outside any anchored specification tree. Exactly one
//! production discovery implementation exists (`omni-registry`), and this
//! is the gate that previously carried a competing resolver.

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tools")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn run_conform(dir: &Path) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_omni-conform"))
        .current_dir(dir)
        .env_remove("OMNI_SPEC_ROOT")
        .output()
        .expect("spawn conformer");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

fn tree_digest(text: &str) -> String {
    text.split("tree=")
        .nth(1)
        .expect("tree field")
        .split_whitespace()
        .next()
        .expect("digest")
        .to_string()
}

#[test]
fn root_and_nested_invocations_resolve_identically() {
    let ws = workspace_root();
    let dirs = [
        ws.clone(),
        ws.join("tools/omni-conform"),
        ws.join("tools/omni-conform/src"),
        ws.join("compiler/omni-driver/src"),
    ];
    let mut digests = Vec::new();
    for dir in &dirs {
        let (ok, text) = run_conform(dir);
        assert!(ok, "conform failed from {}: {text}", dir.display());
        assert!(text.contains("CONFORM PASS"), "unexpected output from {}: {text}", dir.display());
        digests.push(tree_digest(&text));
    }
    let first = digests[0].clone();
    for (dir, digest) in dirs.iter().zip(&digests) {
        assert_eq!(digest, &first, "digest diverged for {}", dir.display());
    }
}

#[test]
fn invocation_outside_any_tree_fails_closed() {
    let outside =
        std::env::temp_dir().join(format!("omni-conform-outside-{}", std::process::id()));
    std::fs::create_dir_all(&outside).expect("mkdir");
    let (ok, text) = run_conform(&outside);
    assert!(!ok, "must fail closed outside a specification tree: {text}");
    assert!(text.contains("FAIL-CLOSED"), "expected fail-closed diagnostics: {text}");
    std::fs::remove_dir_all(&outside).ok();
}

#[test]
fn explicit_override_is_honored_or_fails_closed() {
    let ws = workspace_root();
    let out = Command::new(env!("CARGO_BIN_EXE_omni-conform"))
        .current_dir(&ws)
        .env("OMNI_SPEC_ROOT", ws.join("spec"))
        .output()
        .expect("spawn");
    assert!(out.status.success(), "explicit valid root must pass");
    assert!(String::from_utf8_lossy(&out.stdout).contains("CONFORM PASS"));

    let bogus = std::env::temp_dir().join(format!("omni-conform-nospec-{}", std::process::id()));
    std::fs::create_dir_all(&bogus).expect("mkdir");
    let out = Command::new(env!("CARGO_BIN_EXE_omni-conform"))
        .current_dir(&ws)
        .env("OMNI_SPEC_ROOT", &bogus)
        .output()
        .expect("spawn");
    assert!(!out.status.success(), "invalid explicit root must fail closed");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("FAIL-CLOSED"),
        "expected fail-closed diagnostics"
    );
    std::fs::remove_dir_all(&bogus).ok();
}
