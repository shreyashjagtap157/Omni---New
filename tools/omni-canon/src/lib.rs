//! Canonical hashing for Omni foundation artifacts (0.0.0.6 engine).
//!
//! Two domains with an explicit contract distinction:
//! - JSON documents: key-sorted, span/location-stripped, LF-normalized,
//!   numerically folded, structurally path-virtualized SHA-256. Arrays keep
//!   document order (order is semantic in JSON). Raw artifact bytes are never
//!   canonicalized here; see the byte-hash domain below.
//! - Specification trees (0.0.0.5): deterministic digest over the normative
//!   publication set with LF normalization and no host-path leakage.
//! - Byte-hash domain (intentionally separate, not canonicalization):
//!   plain SHA-256 over artifact bytes, used by provenance records and the
//!   tree file entries. Documented here so the two must not be conflated.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Fail-closed canonicalization errors.
#[derive(Debug, PartialEq, Eq)]
pub enum CanonError {
    /// An absolute host path outside the declared virtualization root would
    /// leak host identity into the digest (REPRO-0004, BUILD-0005).
    UnresolvedHostPath(String),
    /// Duplicate object keys are ambiguous input and must not hash silently.
    DuplicateKey(String),
    /// Malformed input or I/O failure.
    InvalidInput(String),
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnresolvedHostPath(s) => write!(f, "unresolved host path: {s}"),
            Self::DuplicateKey(s) => write!(f, "duplicate object key: {s}"),
            Self::InvalidInput(s) => write!(f, "invalid input: {s}"),
        }
    }
}

impl std::error::Error for CanonError {}

/// Normalize a virtualization root: slash separators, no trailing slash
/// (except a bare `/`), ASCII drive letter uppercased.
fn normalize_root(root: &Path) -> String {
    let mut rn = root.to_string_lossy().replace('\\', "/");
    while rn.len() > 1 && rn.ends_with('/') {
        rn.pop();
    }
    let mut chars: Vec<char> = rn.chars().collect();
    if chars.len() >= 2 && chars[1] == ':' && chars[0].is_ascii_alphabetic() {
        chars[0] = chars[0].to_ascii_uppercase();
    }
    chars.into_iter().collect()
}

/// Structural path classification: slash-normalize, uppercase a drive
/// letter, then compare against the normalized root on component boundary.
/// A bare `/` root virtualizes every absolute path. Anything absolute-shaped
/// but outside the root is an unresolved host identity and is rejected;
/// anything else is not a path and passes through byte-identical.
fn classify_path(s: &str, root_norm: &str) -> Result<Option<String>, CanonError> {
    let mut sn = s.replace('\\', "/");
    let mut chars: Vec<char> = sn.chars().collect();
    if chars.len() >= 2 && chars[1] == ':' && chars[0].is_ascii_alphabetic() {
        chars[0] = chars[0].to_ascii_uppercase();
        sn = chars.into_iter().collect();
    }
    if !root_norm.is_empty() {
        if sn == root_norm {
            return Ok(Some("/omni-root".to_string()));
        }
        if root_norm == "/" {
            if sn.starts_with('/') {
                return Ok(Some(format!("/omni-root{sn}")));
            }
        } else if let Some(rest) = sn.strip_prefix(root_norm) {
            if rest.starts_with('/') {
                return Ok(Some(format!("/omni-root{rest}")));
            }
        }
    }
    let absolute = sn.starts_with('/')
        || (sn.len() >= 3
            && sn.as_bytes()[1] == b':'
            && sn.as_bytes()[0].is_ascii_alphabetic()
            && sn.as_bytes()[2] == b'/');
    if absolute {
        return Err(CanonError::UnresolvedHostPath(s.to_string()));
    }
    Ok(None)
}

/// Normalize line endings inside a JSON string value (RULES section 4).
fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn canonicalize_value(val: &mut Value, root_norm: &str) -> Result<(), CanonError> {
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
                canonicalize_value(&mut value, root_norm)?;
                sorted.insert(key, value);
            }
            *map = sorted;
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                canonicalize_value(item, root_norm)?;
            }
        }
        Value::String(s) => {
            let lf = normalize_line_endings(s);
            if let Some(virtualized) = classify_path(&lf, root_norm)? {
                *s = virtualized;
            } else {
                *s = lf;
            }
        }
        Value::Number(n) => {
            // MANIFEST-0002: canonical numbers are integers. A finite float
            // that is exactly integral folds to integer form; anything else
            // (fractional floats) has no canonical integer form and is kept.
            if let Some(f) = n.as_f64() {
                if f.is_finite()
                    && f.fract() == 0.0
                    && (-9_007_199_254_740_992.0..=9_007_199_254_740_992.0).contains(&f)
                {
                    let i = f as i64;
                    if (i as f64) == f {
                        *val = Value::Number(i.into());
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Reject duplicate object keys at any depth. JSON text permits them but the
/// winning value is ambiguous, so canonical input must not contain them.
/// String-aware scanner: only `{`/`}`/`[`/`]` outside strings affect structure;
/// a string directly followed by `:` inside an object frame is a key.
pub fn reject_duplicate_keys(raw: &str) -> Result<(), CanonError> {
    let bytes = raw.as_bytes();
    let mut i = 0;
    // Each frame holds the keys of one open object; arrays push None.
    let mut stack: Vec<Option<Vec<String>>> = Vec::new();
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'"' => {
                let start = i;
                i += 1;
                let mut closed = false;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => {
                            i += 1;
                            if i < bytes.len() && bytes[i] == b'u' {
                                i += 4;
                            }
                            i += 1;
                        }
                        b'"' => {
                            closed = true;
                            break;
                        }
                        _ => i += 1,
                    }
                }
                if !closed {
                    return Err(CanonError::InvalidInput("unterminated string".to_string()));
                }
                let text = &raw[start + 1..i];
                i += 1;
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b':' {
                    if let Some(Some(keys)) = stack.last_mut() {
                        if keys.contains(&text.to_string()) {
                            return Err(CanonError::DuplicateKey(text.to_string()));
                        }
                        keys.push(text.to_string());
                    }
                }
            }
            b'{' => {
                stack.push(Some(Vec::new()));
                i += 1;
            }
            b'[' => {
                stack.push(None);
                i += 1;
            }
            b'}' | b']' => {
                stack.pop();
                i += 1;
            }
            _ => i += 1,
        }
    }
    Ok(())
}

/// Normative rule-text hash (RULE-0002): trim, LF-normalize, SHA-256 over UTF-8.
/// This is the procedure the 0.0.0.4 registry was built with; the engine
/// re-derives it here so registry hashes stay mechanically checkable.
pub fn rule_text_hash(text: &str) -> String {
    let norm = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut hasher = Sha256::new();
    hasher.update(norm.trim().as_bytes());
    hex::encode(hasher.finalize())
}

/// Canonicalize a JSON value in place against the declared root.
/// Shared engine for evidence and tooling domains; byte construction is
/// identical to [`canonical_hash_json`].
pub fn canonicalize_json_value(val: &mut Value, root: &Path) -> Result<(), CanonError> {
    let root_norm = normalize_root(root);
    canonicalize_value(val, &root_norm)
}

/// Canonical bytes of an already-parsed value (compact JSON, UTF-8).
pub fn canonical_json_bytes(val: &mut Value, root: &Path) -> Result<Vec<u8>, CanonError> {
    canonicalize_json_value(val, root)?;
    serde_json::to_vec(val).map_err(|e| CanonError::InvalidInput(e.to_string()))
}

/// SHA-256 over LF-normalized raw bytes (artifact identity primitive).
pub fn sha256_of_normalized_bytes(raw: &[u8]) -> String {
    spec_tree::sha256_hex(&spec_tree::normalize_bytes(raw))
}

pub fn canonical_hash_json(
    input_path: &Path,
    root: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(input_path)?;
    reject_duplicate_keys(&raw)?;
    let mut val: Value = serde_json::from_str(&raw)?;
    let root_norm = normalize_root(root);
    canonicalize_value(&mut val, &root_norm)?;
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
    pub const DECLARED_FILES: [&str; 12] = [
        "grammar/omni-edition1.ebnf",
        "grammar/candidate2-erratum.md",
        "registry/rules.json",
        "registry/rule-texts.json",
        "schemas/rule-registry.schema.json",
        "schemas/diagnostic.schema.json",
        "schemas/witness.schema.json",
        "schemas/conformance-outcome.schema.json",
        "schemas/verification-failure.schema.json",
        "schemas/provenance.schema.json",
        "schemas/regression.schema.json",
        "schemas/fuzz-promotion.schema.json",
    ];

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
        // Same checkout expressed with native separators virtualizes identically.
        let unix = r#"{"path":"/ws/target/debug/foo"}"#;
        let glass = r#"{"path":"/ws\\target/debug/foo"}"#;
        assert_eq!(hash_of(unix, "/ws"), hash_of(glass, "/ws"));
        // Drive-letter case is insignificant (ASCII drive rule, documented).
        let upper = r#"{"path":"C:\\ws\\target\\debug\\foo"}"#;
        let lower = r#"{"path":"c:/ws/target/debug/foo"}"#;
        assert_eq!(hash_of(upper, "C:\\ws"), hash_of(lower, "C:\\ws"));
        // Unix-style and Windows-style roots for the same logical tree agree
        // once each is virtualized against its own declared root.
        assert_eq!(hash_of(unix, "/ws"), hash_of(lower, "c:/ws"));
    }

    #[test]
    fn non_path_strings_pass_through_byte_identical() {
        // Backslash-containing literals are NOT paths: no separator mangling.
        let backslash = r#"{"s":"a\\b"}"#;
        let slash = r#"{"s":"a/b"}"#;
        assert_ne!(hash_of(backslash, "/ws"), hash_of(slash, "/ws"));
        // A root appearing mid-string is not a structural prefix: untouched.
        let mid = r#"{"m":"see /ws/x for details"}"#;
        let other = r#"{"m":"see /ws/y for details"}"#;
        assert_ne!(hash_of(mid, "/ws"), hash_of(other, "/ws"));
        // Drive-relative `C:foo` is not absolute-shaped: not a path, kept.
        let rel = r#"{"p":"C:foo"}"#;
        assert_eq!(hash_of(rel, "/ws"), hash_of(rel, "/other"));
    }

    #[test]
    fn unresolved_host_paths_are_rejected() {
        for doc in [
            r#"{"p":"/etc/passwd"}"#,
            r#"{"p":"D:/other/place"}"#,
            r#"{"nested":{"p":"/ws2/x"}}"#,
            r#"{"arr":["/ws/ok","/elsewhere/no"]}"#,
        ] {
            let path = scratch(doc);
            let err = canonical_hash_json(&path, Path::new("/ws")).expect_err("must reject");
            assert!(err.to_string().contains("unresolved host path"), "unexpected error: {err}");
            fs::remove_file(&path).ok();
        }
        // Rooted paths are fine, including the root itself and nested values.
        let ok = r#"{"p":"/ws","q":{"r":"/ws/a"},"arr":["/ws/b"]}"#;
        let path = scratch(ok);
        assert!(canonical_hash_json(&path, Path::new("/ws")).is_ok());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn string_line_endings_normalize() {
        // Escaped CRLF/CR inside string values normalize to LF (RULES section 4).
        let crlf = "{\"t\":\"a\\r\\nb\\rc\"}";
        let lf = "{\"t\":\"a\\nb\\nc\"}";
        assert_eq!(hash_of(crlf, "/ws"), hash_of(lf, "/ws"));
        // CRLF as inter-token file whitespace is insignificant whitespace.
        let file_crlf = "{\r\n\"t\" : \"a\",\r\n\"u\" : \"b\"\r\n}";
        let file_lf = "{\n\"t\" : \"a\",\n\"u\" : \"b\"\n}";
        assert_eq!(hash_of(file_crlf, "/ws"), hash_of(file_lf, "/ws"));
    }

    #[test]
    fn integral_floats_fold_to_integers() {
        assert_eq!(hash_of(r#"{"n":1.0}"#, "/ws"), hash_of(r#"{"n":1}"#, "/ws"));
        assert_eq!(hash_of(r#"{"n":1e2}"#, "/ws"), hash_of(r#"{"n":100}"#, "/ws"));
        assert_eq!(hash_of(r#"{"n":-0.0}"#, "/ws"), hash_of(r#"{"n":0}"#, "/ws"));
        // Fractional values have no canonical integer form and are kept.
        assert_ne!(hash_of(r#"{"n":1.5}"#, "/ws"), hash_of(r#"{"n":1}"#, "/ws"));
        assert_ne!(hash_of(r#"{"n":1.5}"#, "/ws"), hash_of(r#"{"n":2}"#, "/ws"));
    }

    #[test]
    fn array_order_is_semantic() {
        assert_ne!(hash_of(r#"{"a":[1,2]}"#, "/ws"), hash_of(r#"{"a":[2,1]}"#, "/ws"));
        assert_eq!(hash_of(r#"{"a":[1,2]}"#, "/ws"), hash_of(r#"{"a":[1,2]}"#, "/ws"));
    }

    #[test]
    fn duplicate_keys_are_rejected() {
        for doc in [
            r#"{"a":1,"a":2}"#,
            r#"{"o":{"x":1,"x":2}}"#,
            r#"[{"k":1},{"k":2,"k":3}]"#,
            "{\"a\":1, \"a\" : 2}",
        ] {
            let path = scratch(doc);
            let err = canonical_hash_json(&path, Path::new("/ws")).expect_err("must reject");
            assert!(err.to_string().contains("duplicate"), "unexpected error: {err}");
            fs::remove_file(&path).ok();
        }
        // Same key text in disjoint objects is fine.
        let ok = r#"[{"k":1},{"k":2}]"#;
        let path = scratch(ok);
        assert!(canonical_hash_json(&path, Path::new("/ws")).is_ok());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn malformed_input_is_rejected() {
        for doc in [r#"{"a":}"#, r#"{"a":1"#, "not json", ""] {
            let path = scratch(doc);
            assert!(canonical_hash_json(&path, Path::new("/ws")).is_err());
            fs::remove_file(&path).ok();
        }
    }

    #[test]
    fn unicode_is_byte_stable() {
        // Unusual Unicode is stable across runs; no normalization is applied,
        // so NFC and NFD forms are (documentedly) distinct byte identities.
        let satellite = char::from_u32(0x1F6CF).expect("satellite");
        let uni = format!("{{\"u\":\"héllo{satellite}\"}}");
        assert_eq!(hash_of(&uni, "/ws"), hash_of(&uni, "/ws"));
        let nfc = "{\"u\":\"\u{e9}\"}";
        let nfd = "{\"u\":\"e\u{301}\"}";
        assert_ne!(hash_of(nfc, "/ws"), hash_of(nfd, "/ws"));
    }

    #[test]
    fn empty_and_deep_structures_are_deterministic() {
        assert_eq!(hash_of("{}", "/ws"), hash_of("{ }", "/ws"));
        assert_eq!(hash_of("[]", "/ws"), hash_of("[ ]", "/ws"));
        let mut deep_a = String::from("0");
        let mut deep_b = String::from("0");
        for _ in 0..64 {
            deep_a = format!("{{\"k\":{deep_a}}}");
            deep_b = format!("{{\"k\" : {deep_b} }}");
        }
        assert_eq!(hash_of(&deep_a, "/ws"), hash_of(&deep_b, "/ws"));
        let long = format!("{{\"s\":\"{}\"}}", "y".repeat(10_000));
        assert_eq!(hash_of(&long, "/ws"), hash_of(&long, "/ws"));
    }

    #[test]
    fn rule_text_hash_matches_registry_vectors() {
        // Vectors from spec/registry/rules.json (0.0.0.4 procedure): exact
        // normative cell text must re-derive the registered hash.
        assert_eq!(
            rule_text_hash("Integer separators `_` are allowed only between digits of the same radix. Leading, trailing, adjacent-to-prefix, adjacent-to-suffix, or doubled separators are errors."),
            "01c7cbb708f5d64c89b1f00105fa709bccc9baf67534d2fb4d825dabc5aa458a"
        );
        assert_eq!(
            rule_text_hash("Whitespace separates tokens but is otherwise insignificant. Newlines never terminate statements."),
            "143add9642fe9d3d2d3ebf21f9cc539fcd683c18d2dc9fcb9a9f0c56ee7ca415"
        );
        assert_eq!(
            rule_text_hash("Newline MAY terminate a statement when the parser is at a complete statement boundary and the next token cannot continue the current expression or declaration."),
            "ac0e5f74249ae78cdb73b0f236ce5126ec5010f6b6da0571a4100607e5019e25"
        );
        // CRLF-authored text hashes identically (LF normalization domain).
        assert_eq!(rule_text_hash("a\r\nb"), rule_text_hash("a\nb"));
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
        let mut files: Vec<(&'static str, &'static [u8])> = vec![
            ("grammar/omni-edition1.ebnf", b"(* t *)\n" as &[u8]),
            ("grammar/candidate2-erratum.md", b"# fixture erratum\n" as &[u8]),
            ("registry/rules.json", br#"{"schema_version":"1.0.0","rules":[]}"# as &[u8]),
            ("registry/rule-texts.json", br#"{"schema_version":"1.0.0","texts":[]}"# as &[u8]),
        ];
        // Every declared schema file must be present (required-artifact rule).
        for schema in [
            "schemas/rule-registry.schema.json",
            "schemas/diagnostic.schema.json",
            "schemas/witness.schema.json",
            "schemas/conformance-outcome.schema.json",
            "schemas/verification-failure.schema.json",
            "schemas/provenance.schema.json",
            "schemas/regression.schema.json",
            "schemas/fuzz-promotion.schema.json",
        ] {
            files.push((schema, br#"{"title":"s"}"# as &[u8]));
        }
        files
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
        fn swap_rules(
            mut files: Vec<(&'static str, &'static [u8])>,
            bytes: &'static [u8],
        ) -> Vec<(&'static str, &'static [u8])> {
            let pos = files.iter().position(|(p, _)| *p == "registry/rules.json").expect("rules");
            files[pos] = ("registry/rules.json", bytes);
            files
        }
        let crlf = swap_rules(
            minimal_tree(),
            b"{\"schema_version\":\"1.0.0\",\r\n\"rules\":[]}\r\n" as &[u8],
        );
        let lf = swap_rules(
            minimal_tree(),
            b"{\"schema_version\":\"1.0.0\",\n\"rules\":[]}\n" as &[u8],
        );
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
