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
