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

pub fn canonical_hash_json(input_path: &Path, root: &Path) -> Result<String, Box<dyn std::error::Error>> {
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
