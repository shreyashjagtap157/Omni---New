use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

fn canonicalize_value(val: &mut Value, root: &Path) {
    match val {
        Value::Object(map) => {
            for key in ["span", "loc", "source_file_id", "line", "column"] {
                map.remove(key);
            }
            let mut sorted = Map::new();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                let mut value = map.remove(&key).expect("key obtained from map");
                canonicalize_value(&mut value, root);
                sorted.insert(key, value);
            }
            *map = sorted;
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                canonicalize_value(item, root);
            }
        }
        Value::String(s) => {
            let root_str = root.to_string_lossy();
            if s.contains(root_str.as_ref()) {
                *s = s.replace(root_str.as_ref(), "/omni-root");
            }
            *s = s.replace('\\', "/");
        }
        _ => {}
    }
}

pub fn canonical_hash_json(
    input_path: &Path,
    root: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(input_path)?;
    let mut val: Value = serde_json::from_str(&raw)?;
    canonicalize_value(&mut val, root);
    let bytes = serde_json::to_vec(&val)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: omni-canon <file.json> <workspace_root>");
        std::process::exit(1);
    }
    let target_file = PathBuf::from(&args[1]);
    let workspace_root = PathBuf::from(&args[2]);
    println!("{}", canonical_hash_json(&target_file, &workspace_root)?);
    Ok(())
}

#[cfg(test)]
mod determinism_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn scratch(content: &str) -> PathBuf {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let path =
            std::env::temp_dir().join(format!("omni-canon-test-{}-{id}.json", std::process::id()));
        fs::write(&path, content).expect("scratch write");
        path
    }

    fn hash_of(content: &str, root: &str) -> String {
        let path = scratch(content);
        let digest = canonical_hash_json(&path, Path::new(root)).expect("hash");
        fs::remove_file(&path).ok();
        digest
    }

    #[test]
    fn reordered_keys_hash_identically() {
        let a = r#"{"rule_id":"LEX-0006","status":"Candidate","text_hash":"abc"}"#;
        let b = r#"{"text_hash":"abc","rule_id":"LEX-0006","status":"Candidate"}"#;
        assert_eq!(hash_of(a, "/omni-root"), hash_of(b, "/omni-root"));
    }

    #[test]
    fn insignificant_whitespace_hashes_identically() {
        let a = r#"{"rule_id":"LEX-0006","status":"Candidate"}"#;
        let b = "{\n  \"rule_id\" : \"LEX-0006\" ,\n  \"status\"  :  \"Candidate\"\n}\n";
        assert_eq!(hash_of(a, "/omni-root"), hash_of(b, "/omni-root"));
    }

    #[test]
    fn out_of_domain_keys_do_not_affect_hash() {
        let a = r#"{"rule_id":"LEX-0006","span":[1,2],"loc":"x","line":3,"column":4,"source_file_id":9}"#;
        let b = r#"{"rule_id":"LEX-0006"}"#;
        assert_eq!(hash_of(a, "/omni-root"), hash_of(b, "/omni-root"));
    }

    #[test]
    fn checkout_paths_normalize_identically() {
        let unix = r#"{"path":"/ws/target/debug/foo"}"#;
        let windows = r#"{"path":"C:\\ws\\target\\debug\\foo"}"#;
        // Same logical checkout expressed under different roots normalizes identically.
        assert_eq!(hash_of(unix, "/ws"), hash_of(windows, "C:\\ws"));
        // Backslash separators normalize even under a common root-free view.
        let mixed = r#"{"path":"a\\b/c"}"#;
        let clean = r#"{"path":"a/b/c"}"#;
        assert_eq!(hash_of(mixed, "/omni-root"), hash_of(clean, "/omni-root"));
    }

    #[test]
    fn normative_change_changes_hash() {
        let a = r#"{"rule_id":"LEX-0006","status":"Candidate"}"#;
        let b = r#"{"rule_id":"LEX-0006","status":"Superseded"}"#;
        assert_ne!(hash_of(a, "/omni-root"), hash_of(b, "/omni-root"));
    }

    #[test]
    fn lifecycle_change_is_deterministic() {
        let b = r#"{"rule_id":"LEX-0002","status":"Superseded"}"#;
        assert_eq!(hash_of(b, "/omni-root"), hash_of(b, "/omni-root"));
    }
}
