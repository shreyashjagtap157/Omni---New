//! Conformance tests for Omni Edition 1 core language features.
//! These tests validate that the compiler correctly implements the Edition 1
//! specification as defined in spec/grammar/omni-edition1.ebnf.
//!
//! Each test compiles a valid Omni program and verifies the expected behavior,
//! either through the reference machine (abstract interpreter) or native codegen.

use std::process::Command;

/// Compile and run an Omni program via the abstract machine interpreter,
/// returning the exit code.
fn run_interpreter(source: &str) -> i32 {
    // Write source to a temp file
    let temp_file = std::env::temp_dir().join(format!("conformance_{}.omni", std::process::id()));
    std::fs::write(&temp_file, source).expect("write test source");

    // Find the omni-driver binary
    let driver = std::env::target_dir().join("debug/omni-driver.exe");
    if !driver.exists() {
        // Try without .exe (non-Windows)
        let driver = std::env::target_dir().join("debug/omni-driver");
        if !driver.exists() {
            panic!("omni-driver binary not found");
        }
    }

    let output = Command::new(&driver)
        .arg(temp_file.to_str().unwrap())
        .output()
        .expect("execute driver");

    output.status.code().unwrap_or(1)
}

/// Compile an Omni program to native object and run it via the native executable,
/// returning the exit code.
fn run_native(source: &str) -> i32 {
    let temp_file = std::env::temp_dir().join(format!("conformance_native_{}.omni", std::process::id()));
    std::fs::write(&temp_file, source).expect("write test source");

    let driver = std::env::target_dir().join("debug/omni-driver.exe");
    if !driver.exists() {
        let driver = std::env::target_dir().join("debug/omni-driver");
        if !driver.exists() {
            panic!("omni-driver binary not found");
        }
    }

    let exe_path = std::env::temp_dir().join(format!("conformance_native_{}.exe", std::process::id()));

    // Compile to native
    let compile_output = Command::new(&driver)
        .arg("--native")
        .arg("-o")
        .arg(&exe_path)
        .arg(&temp_file)
        .output()
        .expect("execute driver for native compile");

    if !compile_output.status.success() {
        eprintln!("Native compilation failed: {}", String::from_utf8_lossy(&compile_output.stderr));
        return 1;
    }

    // Run the native executable
    let run_output = Command::new(&exe_path).output().expect("run native executable");
    run_output.status.code().unwrap_or(1)
}

#[test]
fn test_basic_integer_arithmetic() {
    let source = r#"
fn main() -> i64 {
    let x = 41;
    return x + 1;
}
"#;
    let result = run_interpreter(source);
    assert_eq!(result, 42, "Interpreter should compute 41 + 1 = 42");
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
    let result = run_interpreter(source);
    assert_eq!(result, 1, "1 == 1 should be true");
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
    let result = run_interpreter(source);
    assert_eq!(result, 1, "!(1 == 2) should be true");
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
    let result = run_interpreter(source);
    assert_eq!(result, 5, "while loop should count to 5");
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
    let result = run_interpreter(source);
    assert_eq!(result, 10, "for loop 0..5 should sum to 10 (0+1+2+3+4=10)");
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
    let result = run_interpreter(source);
    assert_eq!(result, 1, "if x > 5 with x=10 should return 1");
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
    let result = run_interpreter(source);
    assert_eq!(result, 99, "match x=3 should fall through to _ => 99");
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
    let result = run_interpreter(source);
    assert_eq!(result, 7, "struct Point { x: 3, y: 4 } should have x+y=7");
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
    let result = run_interpreter(source);
    assert_eq!(result, 42, "add(10, 32) should return 42");
}

#[test]
fn test_native_end_to_end() {
    let source = r#"
fn main() -> i64 {
    return 42;
}
"#;
    let result = run_native(source);
    assert_eq!(result, 42, "native executable should return 42");
}

#[test]
fn test_native_if_else() {
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
    let result = run_native(source);
    assert_eq!(result, 1, "native if x > 5 with x=10 should return 1");
}
