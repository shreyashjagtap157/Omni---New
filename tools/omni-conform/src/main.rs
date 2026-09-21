//! Specification-manifest conformance gate (0.0.0.5).
//!
//! Recomputes the deterministic specification-tree digest, binds it against
//! `spec/manifest/omni-edition1.manifest.json` and
//! `spec/release/foundation-gate.json`, and re-validates the registry
//! structurally. Any mismatch fails closed (exit 101) instead of proceeding.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use omni_canon::spec_tree;
use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    manifest_version: String,
    edition: u32,
    spec_tree_sha256: String,
}

#[derive(Deserialize)]
struct Gate {
    gate: String,
    spec_tree_sha256: String,
}

#[derive(Deserialize)]
struct RegistryRecord {
    rule_id: String,
    domain: String,
    status: String,
    normative: bool,
    text_hash: String,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    witness_tests: Vec<String>,
}

#[derive(Deserialize)]
struct RuleRegistry {
    schema_version: String,
    rules: Vec<RegistryRecord>,
}

#[derive(Debug)]
pub struct ConformReport {
    pub tree_digest: String,
    pub tree_files: usize,
    pub registry_rules: usize,
    pub normative_rules: usize,
}

fn fail(msg: String) -> Box<dyn std::error::Error> {
    format!("FAIL-CLOSED: {msg}").into()
}

/// Verify the specification tree rooted at `spec_root` against its manifest
/// binding and registry. Returns a report on success.
pub fn verify_spec(spec_root: &Path) -> Result<ConformReport, Box<dyn std::error::Error>> {
    let (tree_digest, tree_files) = spec_tree::spec_tree_digest(spec_root)
        .map_err(|e| fail(format!("spec-tree digest failed: {e}")))?;

    let manifest_raw = fs::read_to_string(spec_root.join("manifest/omni-edition1.manifest.json"))
        .map_err(|e| fail(format!("manifest unreadable: {e}")))?;
    let manifest: Manifest =
        serde_json::from_str(&manifest_raw).map_err(|e| fail(format!("manifest invalid: {e}")))?;
    if manifest.manifest_version != "1.0.0" {
        return Err(fail(format!("unsupported manifest_version {}", manifest.manifest_version)));
    }
    if manifest.spec_tree_sha256 != tree_digest {
        return Err(fail(format!(
            "manifest digest mismatch: manifest={} computed={}",
            manifest.spec_tree_sha256, tree_digest
        )));
    }

    let gate_raw = fs::read_to_string(spec_root.join("release/foundation-gate.json"))
        .map_err(|e| fail(format!("release gate unreadable: {e}")))?;
    let gate: Gate =
        serde_json::from_str(&gate_raw).map_err(|e| fail(format!("release gate invalid: {e}")))?;
    if gate.spec_tree_sha256 != manifest.spec_tree_sha256 {
        return Err(fail(format!(
            "gate digest mismatch: gate={} manifest={}",
            gate.spec_tree_sha256, manifest.spec_tree_sha256
        )));
    }

    // Registry/hash consistency: rules.json is inside the digested tree, and its
    // records must satisfy the structural contract (patterns, lifecycle,
    // duplicate and dependency integrity). Rule-text provenance (0.0.0.4) is
    // pinned by tree membership: any registry byte change alters tree_digest.
    let registry_raw = fs::read_to_string(spec_root.join("registry/rules.json"))
        .map_err(|e| fail(format!("registry unreadable: {e}")))?;
    let registry: RuleRegistry =
        serde_json::from_str(&registry_raw).map_err(|e| fail(format!("registry invalid: {e}")))?;
    if registry.schema_version != "1.0.0" {
        return Err(fail("unsupported registry schema_version".to_string()));
    }
    let mut seen = BTreeSet::new();
    for rule in &registry.rules {
        if !seen.insert(rule.rule_id.clone()) {
            return Err(fail(format!("duplicate rule {}", rule.rule_id)));
        }
        let id_ok = rule.rule_id.starts_with("VIBE-GRAM-")
            || (rule
                .rule_id
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
                && rule.rule_id.contains('-'));
        if !id_ok {
            return Err(fail(format!("malformed rule_id {}", rule.rule_id)));
        }
        if !rule.text_hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            || rule.text_hash.len() != 64
        {
            return Err(fail(format!("malformed text_hash {}", rule.rule_id)));
        }
        match rule.status.as_str() {
            "Proposed" | "Candidate" | "Ratified" | "Deprecated" | "Superseded" | "Withdrawn" => {}
            other => return Err(fail(format!("{} unknown status {other}", rule.rule_id))),
        }
        if rule.status == "Ratified" && rule.witness_tests.is_empty() {
            return Err(fail(format!("Ratified rule {} lacks witnesses", rule.rule_id)));
        }
        if !rule.domain.starts_with("OMNI-") {
            return Err(fail(format!("{} bad domain {}", rule.rule_id, rule.domain)));
        }
    }
    let ids: BTreeSet<&str> = registry.rules.iter().map(|r| r.rule_id.as_str()).collect();
    for rule in &registry.rules {
        for dep in &rule.dependencies {
            if !ids.contains(dep.as_str()) {
                return Err(fail(format!("{} depends on unknown {}", rule.rule_id, dep)));
            }
        }
    }

    if manifest.edition != 1 {
        return Err(fail(format!("unexpected edition {}", manifest.edition)));
    }
    if gate.gate.is_empty() {
        return Err(fail("release gate has no name".to_string()));
    }
    Ok(ConformReport {
        tree_digest,
        tree_files: tree_files.len(),
        registry_rules: registry.rules.len(),
        normative_rules: registry.rules.iter().filter(|r| r.normative).count(),
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spec_root = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("spec"));
    match verify_spec(&spec_root) {
        Ok(report) => {
            println!(
                "CONFORM PASS: tree={} files={} registry_rules={} normative={}",
                report.tree_digest,
                report.tree_files,
                report.registry_rules,
                report.normative_rules
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
        fs::write(root.join("schemas/rule-registry.schema.json"), br#"{"title":"s"}"#)
            .expect("write");
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
        assert_eq!(report.tree_files, 3);
        assert_eq!(report.registry_rules, 0);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn manifest_mismatch_is_rejected() {
        let root = scratch_spec(Some(&"0".repeat(64)));
        let err = verify_spec(&root).expect_err("must reject");
        assert!(err.to_string().contains("manifest digest mismatch"));
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
