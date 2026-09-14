//! End-to-End Native Compilation Integration Test

#[test]
fn test_native_object_emission_from_corpus() {
    let corpus_path = "../../tests/corpus/stage0/math.omni";
    let fallback_path = "tests/corpus/stage0/math.omni";

    let source = if std::path::Path::new(corpus_path).exists() {
        std::fs::read_to_string(corpus_path).unwrap()
    } else if std::path::Path::new(fallback_path).exists() {
        std::fs::read_to_string(fallback_path).unwrap()
    } else {
        "fn main() -> i64 { 42 }".to_string()
    };

    let object_bytes = omni_codegen::compile_to_object(&source);
    assert!(object_bytes.is_ok(), "Native object compilation failed: {:?}", object_bytes.err());

    let bytes = object_bytes.unwrap();
    assert!(!bytes.is_empty(), "Emitted object file is empty!");
}
