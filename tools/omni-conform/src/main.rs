//! Specification-manifest conformance gate (0.0.0.5, loader-integrated 0.0.0.10).
//!
//! Authority boundary: `omni-registry` owns typed loading and load-time
//! validation; this gate consumes the loaded specification and adjudicates
//! release bindings (manifest/gate digests) plus the evidence corpus policy.
//! It performs no independent manifest/registry parsing, so no parallel trust
//! path exists. Any mismatch fails closed (exit 101) instead of proceeding.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use omni_evidence::{parse_record, EvidenceBundle, EvidenceKind, RegistryView, ValidationContext};
use omni_registry::load_specification;
use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Deserialize)]
struct Gate {
    gate: String,
    spec_tree_sha256: String,
}

#[derive(Debug)]
pub struct ConformReport {
    pub tree_digest: String,
    pub tree_files: usize,
    pub registry_rules: usize,
    pub normative_rules: usize,
    pub evidence_records: usize,
}

fn fail(msg: String) -> Box<dyn std::error::Error> {
    format!("FAIL-CLOSED: {msg}").into()
}

/// Verify the specification tree rooted at `spec_root` against its manifest
/// binding and registry. The tree is loaded through the `omni-registry`
/// loader (load-time validation authority); this gate adjudicates release
/// bindings and evidence policy on the loaded values. Returns a report.
pub fn verify_spec(spec_root: &Path) -> Result<ConformReport, Box<dyn std::error::Error>> {
    let loaded = load_specification(spec_root.to_path_buf())
        .map_err(|e| fail(format!("spec load failed: {e}")))?;
    let tree_digest = loaded.identity().tree_digest.clone();
    let tree_files: Vec<String> = loaded.identity().artifacts.keys().cloned().collect();

    let gate_raw = fs::read_to_string(spec_root.join("release/foundation-gate.json"))
        .map_err(|e| fail(format!("release gate unreadable: {e}")))?;
    let gate: Gate =
        serde_json::from_str(&gate_raw).map_err(|e| fail(format!("release gate invalid: {e}")))?;
    if gate.spec_tree_sha256 != tree_digest {
        return Err(fail(format!(
            "gate digest mismatch: gate={} loaded={}",
            gate.spec_tree_sha256, tree_digest
        )));
    }
    if gate.gate.is_empty() {
        return Err(fail("release gate has no name".to_string()));
    }

    // Evidence corpus (0.0.0.8): every record under spec/evidence/ validates
    // against the evidence schemas with registry/tree/toolchain binding.
    // An absent or file-empty corpus passes vacuously; the mechanism (not
    // fabricated records) is what this gate qualifies.
    let workspace_root = workspace_root_of(spec_root);
    let toolchain = read_toolchain_channel(&workspace_root);
    let workspace_files = workspace_file_set(&workspace_root);
    let view = RegistryView::from_rules(
        loaded
            .registry()
            .rules
            .iter()
            .map(|r| (r.rule_id.clone(), r.status.clone(), r.text_hash.clone()))
            .collect(),
    );
    let evidence_records = verify_evidence_corpus(
        spec_root,
        &view,
        &tree_digest,
        toolchain.as_deref(),
        &workspace_files,
    )?;

    Ok(ConformReport {
        tree_digest,
        tree_files: tree_files.len(),
        registry_rules: loaded.registry().rules.len(),
        normative_rules: loaded.registry().rules.iter().filter(|r| r.normative).count(),
        evidence_records,
    })
}

/// Workspace root for evidence resolution: the parent of `spec/` in standard
/// layout (recognized by a sibling `Cargo.toml`), else `spec/` itself.
fn workspace_root_of(spec_root: &Path) -> PathBuf {
    if spec_root.file_name().is_some_and(|n| n == "spec") {
        if let Some(parent) = spec_root.parent() {
            if parent.join("Cargo.toml").is_file() {
                return parent.to_path_buf();
            }
        }
    }
    spec_root.to_path_buf()
}

/// Declared toolchain channel from `rust-toolchain.toml`, if resolvable.
fn read_toolchain_channel(workspace_root: &Path) -> Option<String> {
    let raw = fs::read_to_string(workspace_root.join("rust-toolchain.toml")).ok()?;
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("channel") {
            let rest = rest.trim().strip_prefix('=')?.trim();
            let channel = rest.trim_matches('"');
            if !channel.is_empty() {
                return Some(channel.to_string());
            }
        }
    }
    None
}

/// Slash-joined relative file set under the workspace root, excluding
/// generated `target/` trees, `.git/`, and the local `.cargo/` dir.
fn workspace_file_set(workspace_root: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    let walker = WalkDir::new(workspace_root).sort_by_file_name().into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        name != "target" && name != ".git" && name != ".cargo"
    });
    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(workspace_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if !rel.is_empty() {
            files.insert(rel);
        }
    }
    files
}

fn verify_evidence_corpus(
    spec_root: &Path,
    view: &RegistryView,
    tree_digest: &str,
    toolchain: Option<&str>,
    workspace_files: &BTreeSet<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let dir = spec_root.join("evidence");
    if !dir.is_dir() {
        return Ok(0);
    }
    let mut names: Vec<std::ffi::OsString> = Vec::new();
    for entry in fs::read_dir(&dir)? {
        names.push(entry?.file_name());
    }
    names.sort();
    let mut bundle = EvidenceBundle::default();
    let mut count = 0usize;
    for name in names {
        let name_str = name.to_string_lossy().into_owned();
        if name_str == ".gitkeep" || !name_str.ends_with(".json") {
            continue;
        }
        let raw = fs::read_to_string(dir.join(&name))?;
        let record = parse_record(&raw).map_err(|e| fail(format!("evidence {name_str}: {e}")))?;
        match record {
            EvidenceKind::Diagnostic(d) => bundle.diagnostics.push(d),
            EvidenceKind::Witness(w) => bundle.witnesses.push(w),
            EvidenceKind::Outcome(c) => bundle.outcomes.push(c),
            EvidenceKind::Failure(v) => bundle.failures.push(v),
            EvidenceKind::Provenance(p) => bundle.provenances.push(p),
            EvidenceKind::Regression(r) => bundle.regressions.push(r),
            EvidenceKind::Promotion(f) => bundle.promotions.push(f),
        }
        count += 1;
    }
    let ctx = ValidationContext { registry: view, tree_digest, toolchain, files: workspace_files };
    omni_evidence::validate_bundle(&bundle, &ctx)
        .map_err(|e| fail(format!("evidence corpus invalid: {e}")))?;
    Ok(count)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spec_root = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("spec"));
    match verify_spec(&spec_root) {
        Ok(report) => {
            println!(
                "CONFORM PASS: tree={} files={} registry_rules={} normative={} evidence={}",
                report.tree_digest,
                report.tree_files,
                report.registry_rules,
                report.normative_rules,
                report.evidence_records
            );
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(101);
        }
    }
}

#[cfg(test)]
mod conform_tests {
    use super::*;
    use omni_canon::spec_tree;

    fn scratch_spec(manifest_digest: Option<&str>) -> PathBuf {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("omni-conform-{id}"));
        for dir in ["manifest", "release", "grammar", "registry", "schemas", "models", "data"] {
            fs::create_dir_all(root.join(dir)).expect("mkdir");
        }
        fs::write(root.join("grammar/omni-edition1.ebnf"), b"(* t *)\n").expect("write");
        fs::write(root.join("registry/rules.json"), br#"{"schema_version":"1.0.0","rules":[]}"#)
            .expect("write");
        for schema in [
            "rule-registry",
            "diagnostic",
            "witness",
            "conformance-outcome",
            "verification-failure",
            "provenance",
            "regression",
            "fuzz-promotion",
        ] {
            fs::write(
                root.join(format!("schemas/{schema}.schema.json")),
                format!(
                    "{{\"$schema\":\"http://json-schema.org/draft-07/schema#\",\"title\":\"{schema}\",\"type\":\"object\"}}"
                ),
            )
            .expect("write");
        }
        let (digest, _) = spec_tree::spec_tree_digest(&root).expect("digest");
        let bound = manifest_digest.unwrap_or(&digest).to_string();
        fs::write(
            root.join("manifest/omni-edition1.manifest.json"),
            format!(r#"{{"manifest_version":"1.0.0","edition":1,"spec_tree_sha256":"{bound}"}}"#),
        )
        .expect("write");
        fs::write(
            root.join("release/foundation-gate.json"),
            format!(r#"{{"gate":"T","spec_tree_sha256":"{bound}"}}"#),
        )
        .expect("write");
        root
    }

    #[test]
    fn matching_binding_passes() {
        let root = scratch_spec(None);
        let report = verify_spec(&root).expect("pass");
        assert_eq!(report.tree_files, 10);
        assert_eq!(report.registry_rules, 0);
        assert_eq!(report.evidence_records, 0);
        fs::remove_dir_all(&root).ok();
    }

    fn corpus_spec() -> PathBuf {
        let root = scratch_spec(None);
        // Registry with one live rule; corpus files live outside hashed dirs
        // so the bound digest is unaffected.
        fs::write(
            root.join("registry/rules.json"),
            format!(
                "{{\"schema_version\":\"1.0.0\",\"rules\":[{{\"rule_id\":\"LEX-0006\",\"domain\":\"OMNI-LEX\",\"status\":\"Candidate\",\"normative\":true,\"text_hash\":\"{}\",\"dependencies\":[],\"witness_tests\":[]}}]}}",
                "1".repeat(64)
            ),
        )
        .expect("write");
        fs::create_dir_all(root.join("fix")).expect("mkdir");
        fs::write(root.join("fix/a.omni"), b"let x = 1;\n").expect("write");
        fs::create_dir_all(root.join("evidence")).expect("mkdir");
        // Rebind after registry rewrite.
        let (digest, _) = spec_tree::spec_tree_digest(&root).expect("digest");
        let manifest =
            format!(r#"{{"manifest_version":"1.0.0","edition":1,"spec_tree_sha256":"{digest}"}}"#);
        fs::write(root.join("manifest/omni-edition1.manifest.json"), manifest).expect("write");
        fs::write(
            root.join("release/foundation-gate.json"),
            format!(r#"{{"gate":"T","spec_tree_sha256":"{digest}"}}"#),
        )
        .expect("write");
        root
    }

    #[test]
    fn valid_evidence_corpus_passes() {
        let root = corpus_spec();
        fs::write(
            root.join("evidence/d1.json"),
            r#"{"schema_version":"1.0.0","kind":"diagnostic","id":"E0001","severity":"error","rule_id":"LEX-0006","message":"m","span":{"file":"fix/a.omni","start":0,"end":1}}"#,
        )
        .expect("write");
        fs::write(
            root.join("evidence/w1.json"),
            r#"{"schema_version":"1.0.0","kind":"witness","witness_id":"WIT-0001","rule_id":"LEX-0006","test_ref":"fix/a.omni","expected":"failure","observed":"failure","diagnostic_ids":["E0001"]}"#,
        )
        .expect("write");
        fs::write(
            root.join("evidence/c1.json"),
            r#"{"schema_version":"1.0.0","kind":"conformance-outcome","outcome_id":"CONF-0001","rule_id":"LEX-0006","witness_ids":["WIT-0001"],"expected":"reject","actual":"reject","verdict":"pass","diagnostic_ids":["E0001"],"spec_tree_sha256":"PLACEHOLDER","toolchain":"x"}"#,
        )
        .expect("write");
        // Bind the outcome record to this tree digest.
        let (digest, _) = spec_tree::spec_tree_digest(&root).expect("digest");
        let raw = fs::read_to_string(root.join("evidence/c1.json")).expect("read");
        fs::write(root.join("evidence/c1.json"), raw.replace("PLACEHOLDER", &digest))
            .expect("write");
        let report = verify_spec(&root).expect("pass");
        assert_eq!(report.evidence_records, 3);
        assert_eq!(report.registry_rules, 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn invalid_evidence_corpus_fails() {
        let root = corpus_spec();
        fs::write(
            root.join("evidence/bad.json"),
            r#"{"schema_version":"1.0.0","kind":"diagnostic","id":"E0001","severity":"error","rule_id":"GHOST-0001","message":"m","span":{"file":"fix/a.omni","start":0,"end":1}}"#,
        )
        .expect("write");
        let err = verify_spec(&root).expect_err("must reject");
        assert!(err.to_string().contains("unresolved rule"), "got: {err}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn manifest_mismatch_is_rejected() {
        let root = scratch_spec(Some(&"0".repeat(64)));
        let err = verify_spec(&root).expect_err("must reject");
        assert!(err.to_string().contains("ManifestMismatch"), "got: {err}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn gate_mismatch_is_rejected() {
        let root = scratch_spec(None);
        let (digest, _) = spec_tree::spec_tree_digest(&root).expect("digest");
        fs::write(
            root.join("release/foundation-gate.json"),
            format!(r#"{{"gate":"T","spec_tree_sha256":"{}"}}"#, "f".repeat(64)),
        )
        .expect("write");
        let err = verify_spec(&root).expect_err("must reject");
        assert!(err.to_string().contains("gate digest mismatch"));
        assert!(digest.len() == 64);
        fs::remove_dir_all(&root).ok();
    }
}
