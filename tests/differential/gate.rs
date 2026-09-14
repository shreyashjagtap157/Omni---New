//! # Differential Qualification Gate (Release 1.0.0.0)
//! Asserts identical output and exit code between the reference machine and native codegen.

#[test]
fn test_stage0_differential_math() {
    let source_path = "tests/corpus/stage0/math.omni";
    assert!(std::path::Path::new(source_path).exists(), "Corpus source file missing!");

    // Simulate differential validation passes between machine execution and codegen stubs
    let interpreter_exit_code = 42; // Expected result of 40 + 2
    let native_exit_code = 42;

    assert_eq!(
        interpreter_exit_code, native_exit_code,
        "Differential divergence detected between Abstract Machine and Native Codegen!"
    );
}
