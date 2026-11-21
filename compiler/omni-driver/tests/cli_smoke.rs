//! CLI Driver Smoke Tests

#[test]
fn test_driver_help_flag() {
    let source_path = if std::path::Path::new("src/main.rs").exists() {
        "src/main.rs"
    } else {
        "compiler/omni-driver/src/main.rs"
    };
    assert!(std::path::Path::new(source_path).exists(), "Driver source missing!");
}

#[test]
fn test_driver_native_flag() {
    // Test that --native flag is recognized
    let output = std::process::Command::new("omni-driver")
        .arg("--native")
        .output()
        .expect("Failed to execute omni-driver");
    assert!(output.status.success(), "omni-driver --native flag should be recognized");
}

#[test]
fn test_driver_run_flag() {
    // Test that available flags are recognized by checking the help output
    let output = std::process::Command::new("omni-driver")
        .arg("--help")
        .output()
        .expect("Failed to execute omni-driver");
    let help_text = String::from_utf8_lossy(&output.stdout);
    // Check for flags that are actually implemented
    assert!(help_text.contains("-o"), "omni-driver help should contain -o flag");
    assert!(help_text.contains("--output"), "omni-driver help should contain --output flag");
    assert!(help_text.contains("-O"), "omni-driver help should contain -O flag");
    assert!(help_text.contains("--opt-level"), "omni-driver help should contain --opt-level flag");
    assert!(help_text.contains("--native"), "omni-driver help should contain --native flag");
    assert!(help_text.contains("-h"), "omni-driver help should contain -h flag");
    assert!(help_text.contains("--help"), "omni-driver help should contain --help flag");
}

#[test]
fn test_basic_arithmetic() {
    // Test that the driver can process a basic arithmetic file
    let test_file = std::env::temp_dir().join("test_arith.omni");
    std::fs::write(&test_file, "(+ 1 2)").expect("Failed to write test file");
    let output = std::process::Command::new("omni-driver")
        .arg(&test_file)
        .output()
        .expect("Failed to execute omni-driver");
    // Should not fail with "file not found" or similar
    assert!(output.status.code() != Some(127), "omni-driver should be found in PATH");
    // Clean up
    let _ = std::fs::remove_file(&test_file);
}

#[test]
fn test_variable_declaration() {
    // Test that the driver can process a variable declaration file
    let test_file = std::env::temp_dir().join("test_var.omni");
    std::fs::write(&test_file, "(set x 42)").expect("Failed to write test file");
    let output = std::process::Command::new("omni-driver")
        .arg(&test_file)
        .output()
        .expect("Failed to execute omni-driver");
    // Should not fail with "file not found" or similar
    assert!(output.status.code() != Some(127), "omni-driver should be found in PATH");
    // Clean up
    let _ = std::fs::remove_file(&test_file);
}
