//! Canonical hashing for Omni foundation artifacts.
//!
//! Two domains:
//! - JSON documents: key-sorted, span/location-stripped, path-normalized SHA-256.
//! - Specification trees (0.0.0.5): deterministic digest over the normative
//!   publication set with LF normalization and no host-path leakage.

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

pub mod spec_tree {
    //! Deterministic specification-tree identity (0.0.0.5).
    //!
    //! Publication set (mechanical, cited): `grammar/omni-edition1.ebnf`
    //! (named normative by GRAM-0001), `registry/rules.json` (populated
    //! 0.0.0.4), `schemas/rule-registry.schema.json` (named normative by
    //! RULES section 4), and anything under `models/` or `data/` (reserved;
    //! 0.0.0.10+ rebinds on change). Excluded: `.gitkeep` VCS scaffolding and
    //! the `manifest/` and `release/` directories, which hold the digest
    //! (anti-self-reference, documented in `docs/foundation-gates.md`). Any
    //! other file under an included directory is an undeclared artifact and
    //! fails closed.
    //!
    //! Digest procedure: walk included dirs in lexicographic UTF-8 byte order
    //! of slash-joined relative paths (host separators normalized before
    //! comparison); normalize file bytes CRLF/CR to LF; take per-file SHA-256
    //! hex over normalized bytes; concatenate tree manifest lines of the form
    //! relative-path, newline, hex, newline in path order; the tree digest is
    //! SHA-256 over that concatenation.

    use super::*;

    /// Directories of `spec/` that participate in the tree digest.
    pub const INCLUDED_DIRS: [&str; 5] = ["grammar", "registry", "schemas", "models", "data"];

    /// Files explicitly declared outside `models/` and `data/`.
    pub const DECLARED_FILES: [&str; 3] =
        ["grammar/omni-edition1.ebnf", "registry/rules.json", "schemas/rule-registry.schema.json"];

    /// Normalize a relative path to `/`-separated form for ordering and hashing.
    pub fn normalize_rel(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    /// Normalize raw file bytes: CRLF/CR become LF. Returns owned bytes.
    pub fn normalize_bytes(raw: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(raw.len());
        let mut i = 0;
        while i < raw.len() {
            if raw[i] == b'\r' {
                out.push(b'\n');
                if i + 1 < raw.len() && raw[i + 1] == b'\n' {
                    i += 1;
                }
            } else {
                out.push(raw[i]);
            }
            i += 1;
        }
        out
    }

    pub fn sha256_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Compute the tree digest. Returns `(digest_hex, sorted_rel_paths)`.
    /// Fails closed on missing required artifacts and undeclared files.
    pub fn spec_tree_digest(
        spec_root: &Path,
    ) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
        let mut entries: Vec<(String, String)> = Vec::new();
        for dir in INCLUDED_DIRS {
            let base = spec_root.join(dir);
            if !base.is_dir() {
                return Err(format!("missing required spec directory: {dir}").into());
            }
            let mut rels: Vec<PathBuf> = Vec::new();
            collect_files(&base, &base, &mut rels)?;
            for rel in rels {
                let rel_str = normalize_rel(&rel);
                let full = format!("{dir}/{rel_str}");
                if rel.file_name().is_some_and(|n| n == ".gitkeep") {
                    continue;
                }
                let under_models_data = dir == "models" || dir == "data";
                if !under_models_data && !DECLARED_FILES.contains(&full.as_str()) {
                    return Err(format!("undeclared normative artifact: {full}").into());
                }
                let raw = fs::read(base.join(&rel))?;
                let digest = sha256_hex(&normalize_bytes(&raw));
                entries.push((full, digest));
            }
        }
        for required in DECLARED_FILES {
            if !entries.iter().any(|(p, _)| p == required) {
                return Err(format!("missing required spec artifact: {required}").into());
            }
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let mut manifest_bytes = Vec::new();
        for (path, hex) in &entries {
            manifest_bytes.extend_from_slice(path.as_bytes());
            manifest_bytes.push(b'\n');
            manifest_bytes.extend_from_slice(hex.as_bytes());
            manifest_bytes.push(b'\n');
        }
        let digest = sha256_hex(&manifest_bytes);
        let files = entries.into_iter().map(|(p, _)| p).collect();
        Ok((digest, files))
    }

    fn collect_files(
        base: &Path,
        dir: &Path,
        out: &mut Vec<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut names: Vec<std::ffi::OsString> = Vec::new();
        for entry in fs::read_dir(dir)? {
            names.push(entry?.file_name());
        }
        names.sort();
        for name in names {
            let path = dir.join(&name);
            let meta = fs::symlink_metadata(&path)?;
            if meta.is_dir() {
                collect_files(base, &path, out)?;
            } else if meta.is_file() {
                out.push(path.strip_prefix(base)?.to_path_buf());
            }
        }
        Ok(())
    }
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
        assert_eq!(hash_of(unix, "/ws"), hash_of(windows, "C:\\ws"));
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

    /// Build a minimal spec tree under a temp dir. `files` maps
    /// `dir/rel` to bytes; all five included dirs are created.
    fn scratch_tree(files: &[(&str, &[u8])]) -> PathBuf {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("omni-tree-test-{}-{id}", std::process::id()));
        for dir in spec_tree::INCLUDED_DIRS {
            fs::create_dir_all(root.join(dir)).expect("mkdir");
        }
        for (rel, bytes) in files {
            let path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("mkdir parent");
            }
            fs::write(&path, bytes).expect("write");
        }
        root
    }

    fn minimal_tree() -> Vec<(&'static str, &'static [u8])> {
        vec![
            ("grammar/omni-edition1.ebnf", b"(* t *)\n" as &[u8]),
            ("registry/rules.json", br#"{"schema_version":"1.0.0","rules":[]}"# as &[u8]),
            ("schemas/rule-registry.schema.json", br#"{"title":"s"}"# as &[u8]),
        ]
    }

    #[test]
    fn identical_trees_digest_identically() {
        let a = scratch_tree(&minimal_tree());
        let b = scratch_tree(&minimal_tree());
        let (da, _) = spec_tree::spec_tree_digest(&a).expect("digest a");
        let (db, _) = spec_tree::spec_tree_digest(&b).expect("digest b");
        assert_eq!(da, db);
        fs::remove_dir_all(&a).ok();
        fs::remove_dir_all(&b).ok();
    }

    #[test]
    fn file_ordering_does_not_affect_digest() {
        // Creation order differs; walker sorts, so digests agree.
        let mut fwd = minimal_tree();
        fwd.push(("models/m1.json", b"{}" as &[u8]));
        let mut rev = vec![("models/m1.json", b"{}" as &[u8])];
        rev.extend(minimal_tree());
        let a = scratch_tree(&fwd);
        let b = scratch_tree(&rev);
        let (da, _) = spec_tree::spec_tree_digest(&a).expect("digest a");
        let (db, _) = spec_tree::spec_tree_digest(&b).expect("digest b");
        assert_eq!(da, db);
        fs::remove_dir_all(&a).ok();
        fs::remove_dir_all(&b).ok();
    }

    #[test]
    fn separators_do_not_affect_rel_paths() {
        assert_eq!(spec_tree::normalize_rel(Path::new("a\\b/c")), "a/b/c");
    }

    #[test]
    fn crlf_normalizes_to_lf() {
        let mut crlf = minimal_tree();
        crlf[1] = (
            "registry/rules.json",
            b"{\"schema_version\":\"1.0.0\",\r\n\"rules\":[]}\r\n" as &[u8],
        );
        let mut lf = minimal_tree();
        lf[1] =
            ("registry/rules.json", b"{\"schema_version\":\"1.0.0\",\n\"rules\":[]}\n" as &[u8]);
        let a = scratch_tree(&crlf);
        let b = scratch_tree(&lf);
        let (da, _) = spec_tree::spec_tree_digest(&a).expect("digest a");
        let (db, _) = spec_tree::spec_tree_digest(&b).expect("digest b");
        assert_eq!(da, db);
        fs::remove_dir_all(&a).ok();
        fs::remove_dir_all(&b).ok();
    }

    #[test]
    fn changing_one_file_changes_digest() {
        let a = scratch_tree(&minimal_tree());
        let mut altered = minimal_tree();
        altered[0] = ("grammar/omni-edition1.ebnf", b"(* t2 *)\n" as &[u8]);
        let b = scratch_tree(&altered);
        let (da, _) = spec_tree::spec_tree_digest(&a).expect("digest a");
        let (db, _) = spec_tree::spec_tree_digest(&b).expect("digest b");
        assert_ne!(da, db);
        fs::remove_dir_all(&a).ok();
        fs::remove_dir_all(&b).ok();
    }

    #[test]
    fn gitkeep_variance_does_not_change_digest() {
        let a = scratch_tree(&minimal_tree());
        let mut with_sector = minimal_tree();
        with_sector.push(("models/.gitkeep", b"" as &[u8]));
        let b = scratch_tree(&with_sector);
        // .gitkeep files are excluded from the digest domain entirely.
        fs::write(b.join("data").join(".gitkeep"), b"comment\n").ok();
        let (da, _) = spec_tree::spec_tree_digest(&a).expect("digest a");
        let (db, _) = spec_tree::spec_tree_digest(&b).expect("digest b");
        assert_eq!(da, db);
        fs::remove_dir_all(&a).ok();
        fs::remove_dir_all(&b).ok();
    }

    #[test]
    fn missing_required_artifact_fails() {
        let mut files = minimal_tree();
        files.retain(|(p, _)| *p != "grammar/omni-edition1.ebnf");
        let root = scratch_tree(&files);
        assert!(spec_tree::spec_tree_digest(&root).is_err());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn extra_undeclared_artifact_fails() {
        let mut files = minimal_tree();
        files.push(("grammar/extra.ebnf", b"(* x *)\n" as &[u8]));
        let root = scratch_tree(&files);
        let err = spec_tree::spec_tree_digest(&root).expect_err("must fail");
        assert!(err.to_string().contains("undeclared"));
        fs::remove_dir_all(&root).ok();
    }
}
