//! Executable Edition 1 conformance smoke corpus.
//!
//! The test target is registered by compiler/omni-driver/Cargo.toml and runs
//! against Cargo's actual omni-driver binary. These cases validate the
//! reference-machine path and native object emission. They do not claim that
//! native objects have been linked into standalone executables.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const DRIVER: &str = env!("CARGO_BIN_EXE_omni-driver");
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir()
            .join(format!("omni-edition1-conformance-{}-{id}-{label}", std::process::id()));
        std::fs::create_dir(&path).expect("create unique conformance test directory");
        Self { path }
    }

    fn source(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path.join(name);
        std::fs::write(&path, contents).expect("write Omni conformance source");
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

fn run_interpreter(source: &str) -> i64 {
    let temp = TempTestDir::new("interpreter");
    let source_path = temp.source("case.omni", source);
    let output = Command::new(DRIVER)
        .arg("--run")
        .arg(&source_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("execute omni-driver reference-machine path");

    assert!(
        output.status.success(),
        "reference execution failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("Exit code: "))
        .unwrap_or_else(|| panic!("driver did not report an integer result: {stdout}"));
    result.parse().unwrap_or_else(|error| panic!("invalid driver result {result:?}: {error}"))
}

fn compile_native_object(source: &str) -> (u64, u64) {
    let temp = TempTestDir::new("native-object");
    let source_path = temp.source("case.omni", source);
    let object_path = temp.path().join("case.obj");
    let output = Command::new(DRIVER)
        .arg("--native")
        .arg("--output")
        .arg(&object_path)
        .arg(&source_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("execute omni-driver native object emission");

    assert!(
        output.status.success(),
        "native object emission failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let object_size = std::fs::metadata(&object_path).expect("native object must be created").len();
    assert!(object_size > 0, "native object must not be empty");

    let sidecar = object_path.with_extension("provenance.json");
    let sidecar_size = std::fs::metadata(&sidecar)
        .expect("native object provenance sidecar must be created")
        .len();
    assert!(sidecar_size > 0, "provenance sidecar must not be empty");

    (object_size, sidecar_size)
}

#[test]
fn test_basic_integer_arithmetic() {
    let source = r#"
fn main() -> i64 {
    let x = 41;
    return x + 1;
}
"#;
    assert_eq!(run_interpreter(source), 42);
}

#[test]
fn test_boolean_comparison() {
    let source = r#"
fn main() -> i64 {
    if 1 == 1 {
        return 1;
    }
    return 0;
}
"#;
    assert_eq!(run_interpreter(source), 1);
}

#[test]
fn test_boolean_not() {
    let source = r#"
fn main() -> i64 {
    if !(1 == 2) {
        return 1;
    }
    return 0;
}
"#;
    assert_eq!(run_interpreter(source), 1);
}

#[test]
fn test_while_loop() {
    let source = r#"
fn main() -> i64 {
    let mut n = 0;
    while n < 5 {
        n = n + 1;
    }
    return n;
}
"#;
    assert_eq!(run_interpreter(source), 5);
}

#[test]
fn test_for_loop() {
    let source = r#"
fn main() -> i64 {
    let mut sum = 0;
    for i in 0..5 {
        sum = sum + i;
    }
    return sum;
}
"#;
    assert_eq!(run_interpreter(source), 10);
}

#[test]
fn test_if_else() {
    let source = r#"
fn main() -> i64 {
    let x = 10;
    if x > 5 {
        return 1;
    } else {
        return 0;
    }
}
"#;
    assert_eq!(run_interpreter(source), 1);
}

#[test]
fn test_match_expression() {
    let source = r#"
fn main() -> i64 {
    let x = 3;
    return match x {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => 99,
    };
}
"#;
    assert_eq!(run_interpreter(source), 99);
}

#[test]
fn test_struct_construction() {
    let source = r#"
struct Point {
    x: i64,
    y: i64,
}

fn main() -> i64 {
    let p = Point { x: 3, y: 4 };
    return p.x + p.y;
}
"#;
    assert_eq!(run_interpreter(source), 7);
}

#[test]
fn test_function_call() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

fn main() -> i64 {
    return add(10, 32);
}
"#;
    assert_eq!(run_interpreter(source), 42);
}

#[test]
fn test_native_end_to_end_emits_object_and_provenance() {
    let source = r#"
fn main() -> i64 {
    return 42;
}
"#;
    let (object_size, provenance_size) = compile_native_object(source);
    assert!(object_size > 0 && provenance_size > 0);
}

#[test]
fn test_native_if_else_emits_object_and_provenance() {
    let source = r#"
fn main() -> i64 {
    let x = 10;
    if x > 5 {
        return 1;
    } else {
        return 0;
    }
}
"#;
    let (object_size, provenance_size) = compile_native_object(source);
    assert!(object_size > 0 && provenance_size > 0);
}
