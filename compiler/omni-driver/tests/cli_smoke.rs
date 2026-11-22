//! Cross-platform CLI integration tests for the Omni driver.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const DRIVER: &str = env!("CARGO_BIN_EXE_omni-driver");

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "omni-driver-cli-smoke-{}-{label}",
            std::process::id()
        ));
        if path.exists() {
            std::fs::remove_dir_all(&path).expect("remove stale test directory owned by this process");
        }
        std::fs::create_dir(&path).expect("create isolated CLI test directory");
        Self { path }
    }

    fn source(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path.join(name);
        std::fs::write(&path, contents).expect("write Omni source fixture");
        path
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_driver(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(DRIVER)
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Cargo-provided omni-driver executable must be launchable")
}

#[test]
fn test_driver_help_flag() {
    let output = run_driver(&[std::ffi::OsStr::new("--help")]);
    assert!(
        output.status.success(),
        "driver --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let help = String::from_utf8_lossy(&output.stdout);
    for expected in ["Usage:", "--output", "--opt-level", "--run", "--native", "--help"] {
        assert!(help.contains(expected), "driver help is missing {expected:?}: {help}");
    }
}

#[test]
fn test_driver_native_flag() {
    let temp = TempTestDir::new("native");
    let source = temp.source("native.omni", "fn main() -> i64 { return 40 + 2; }");
    let object = temp.path().join("native.o");
    let output = Command::new(DRIVER)
        .arg("--native")
        .arg("--output")
        .arg(&object)
        .arg(&source)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Cargo-provided omni-driver executable must be launchable");

    assert!(
        output.status.success(),
        "native object emission failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = std::fs::metadata(&object).expect("native object must be created");
    assert!(metadata.len() > 0, "native object must not be empty");
}

#[test]
fn test_driver_run_flag() {
    let temp = TempTestDir::new("run");
    let source =
        temp.source("run.omni", "fn main() -> i64 { let answer = 40 + 2; return answer; }");
    let output = Command::new(DRIVER)
        .arg("--run")
        .arg(&source)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Cargo-provided omni-driver executable must be launchable");

    assert!(
        output.status.success(),
        "reference execution failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Exit code: 42"),
        "driver did not report the computed result: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_basic_arithmetic() {
    let temp = TempTestDir::new("arithmetic");
    let source = temp.source("arithmetic.omni", "fn main() -> i64 { return 1 + 2; }");
    let output = Command::new(DRIVER)
        .arg(&source)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Cargo-provided omni-driver executable must be launchable");

    assert!(
        output.status.success(),
        "default reference execution failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Exit code: 3"),
        "driver did not compute 1 + 2: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_variable_declaration() {
    let temp = TempTestDir::new("variable");
    let source =
        temp.source("variable.omni", "fn main() -> i64 { let answer = 40 + 2; return answer; }");
    let output = Command::new(DRIVER)
        .arg(&source)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Cargo-provided omni-driver executable must be launchable");

    assert!(
        output.status.success(),
        "variable declaration execution failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Exit code: 42"),
        "driver did not preserve the variable's value: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
