//! Typed specification loader (0.0.0.10).
//!
//! Trust boundary between the normative `spec/` publication set and future
//! compiler consumers. State-separated API:
//!
//! ```text
//! RawSpecification      (unvalidated root handle; freely constructible)
//!        | validate()
//! ValidatedSpecification (phase-ordered checks passed; construction sealed)
//!        | load()
//! LoadedSpecification    (immutable typed artifacts + verified identity)
//! ```
//!
//! Validation order is fixed: containment, structural parse (with
//! duplicate-key rejection), version, schema shape, canonical form (via the
//! `omni-canon` engine, never duplicated here), digest verification against
//! the manifest binding, cross-artifact consistency, typed construction,
//! provenance capture. Any earlier failure prevents later semantic use.
//!
//! Authority seams (no parallel trust paths): `omni-canon` owns bytes and
//! digests; this loader owns typed validation; `omni-conform` owns release
//! bindings (manifest/gate) and evidence policy on top of loaded values;
//! `omni-audit` owns linkage claims and dependency cycles. Dependency-cycle
//! detection and witness *resolution* stay with the audit; rule-text to hash
//! binding stays procedural (texts live outside the tree).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use omni_canon::spec_tree;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Errors: machine-readable class + validation phase + artifact identity.
// ---------------------------------------------------------------------------

/// Stable failure classes; distinct states never collapse into one another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadErrorClass {
    MissingArtifact,
    UndeclaredArtifact,
    DuplicateArtifact,
    MalformedArtifact,
    SchemaViolation,
    UnsupportedVersion,
    DigestMismatch,
    ManifestMismatch,
    CanonicalizationFailure,
    ContainmentViolation,
    InconsistentReference,
}

/// Validation phase; every phase precedes any semantic use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationPhase {
    Containment,
    Parse,
    Version,
    Schema,
    Canonical,
    Digest,
    Consistency,
    Construction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    pub class: LoadErrorClass,
    pub phase: ValidationPhase,
    pub artifact: Option<String>,
    pub detail: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "spec load {:?} in {:?} ({}): {}",
            self.class,
            self.phase,
            self.artifact.as_deref().unwrap_or("<root>"),
            self.detail
        )
    }
}

impl std::error::Error for LoadError {}

fn fail<T>(
    class: LoadErrorClass,
    phase: ValidationPhase,
    artifact: Option<String>,
    detail: String,
) -> Result<T, LoadError> {
    Err(LoadError { class, phase, artifact, detail })
}

// ---------------------------------------------------------------------------
// Typed representations (mirror the normative schemas; strict shapes).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: String,
    pub edition: u32,
    pub status: Option<String>,
    pub spec_tree_sha256: String,
    pub plan_sha256: Option<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub stage0_feature_predicates: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleRecord {
    pub rule_id: String,
    pub domain: String,
    pub status: String,
    pub normative: bool,
    pub text_hash: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
    #[serde(default)]
    pub witness_tests: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema_version: String,
    pub rules: Vec<RuleRecord>,
}

/// A validated schema publication: structural properties only (no full
/// meta-schema interpreter; the enforced subset is documented here).
#[derive(Debug, Clone)]
pub struct SchemaDescriptor {
    pub rel_path: String,
    pub digest: String,
}

/// Grammar publication: bytes identity only. Interpreting the grammar is
/// language semantics and explicitly out of scope for the loader.
#[derive(Debug, Clone)]
pub struct GrammarPublication {
    pub rel_path: String,
    pub digest: String,
    pub byte_len: u64,
}

/// An intentionally empty model/data domain. Constructible only inside this
/// crate: emptiness is a verified state, never an assumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyDomain(());

/// Verified specification identity: tree digest plus per-artifact digests in
/// deterministic (lexicographic path) order.
#[derive(Debug, Clone)]
pub struct SpecIdentity {
    pub tree_digest: String,
    pub artifacts: BTreeMap<String, String>,
}

/// Load-time provenance record (0.0.0.10 boundary, not 0.0.0.12 build
/// provenance): what was loaded, in what order, under which versions.
#[derive(Debug, Clone)]
pub struct LoadProvenance {
    pub loader_version: String,
    pub tree_digest: String,
    pub artifact_count: usize,
    pub ordering: String,
}

pub const LOADER_VERSION: &str = "1.0.0";
const DRAFT07: &str = "http://json-schema.org/draft-07/schema#";

// ---------------------------------------------------------------------------
// State-separated API.
// ---------------------------------------------------------------------------

/// Unvalidated root handle.
#[derive(Debug)]
pub struct RawSpecification {
    spec_root: PathBuf,
}

impl RawSpecification {
    pub fn new(spec_root: PathBuf) -> Self {
        Self { spec_root }
    }

    pub fn validate(self) -> Result<ValidatedSpecification, LoadError> {
        ValidatedSpecification::check(self.spec_root)
    }
}

/// Sealed post-validation state; constructible only via [`RawSpecification`].
#[derive(Debug)]
pub struct ValidatedSpecification {
    spec_root: PathBuf,
    manifest: Manifest,
    registry: Registry,
    schemas: Vec<SchemaDescriptor>,
    grammar: GrammarPublication,
    models: EmptyDomain,
    data: EmptyDomain,
    identity: SpecIdentity,
}

impl ValidatedSpecification {
    fn check(spec_root: PathBuf) -> Result<Self, LoadError> {
        // Phase 1: containment. The root must be a real directory; every
        // artifact path below is joined to it and re-checked for escape.
        if !spec_root.is_dir() {
            return fail(
                LoadErrorClass::MissingArtifact,
                ValidationPhase::Containment,
                None,
                format!("spec root is not a directory: {}", spec_root.display()),
            );
        }
        let read_contained = |rel: &str| -> Result<Vec<u8>, LoadError> {
            let parsed = Path::new(rel);
            if parsed.is_absolute()
                || parsed.components().any(|c| {
                    matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir)
                })
            {
                return fail(
                    LoadErrorClass::ContainmentViolation,
                    ValidationPhase::Containment,
                    Some(rel.to_string()),
                    "non-contained artifact path".to_string(),
                );
            }
            fs::read(spec_root.join(parsed)).map_err(|e| LoadError {
                class: LoadErrorClass::MissingArtifact,
                phase: ValidationPhase::Containment,
                artifact: Some(rel.to_string()),
                detail: e.to_string(),
            })
        };

        // Phase 2: structural parse with duplicate-key rejection at the
        // boundary; typed construction only after validation succeeds.
        let parse_json = |rel: &str| -> Result<serde_json::Value, LoadError> {
            let raw = read_contained(rel)?;
            let text = std::str::from_utf8(&raw).map_err(|e| LoadError {
                class: LoadErrorClass::MalformedArtifact,
                phase: ValidationPhase::Parse,
                artifact: Some(rel.to_string()),
                detail: e.to_string(),
            })?;
            omni_canon::reject_duplicate_keys(text).map_err(|e| LoadError {
                class: LoadErrorClass::MalformedArtifact,
                phase: ValidationPhase::Parse,
                artifact: Some(rel.to_string()),
                detail: e.to_string(),
            })?;
            serde_json::from_str(text).map_err(|e| LoadError {
                class: LoadErrorClass::MalformedArtifact,
                phase: ValidationPhase::Parse,
                artifact: Some(rel.to_string()),
                detail: e.to_string(),
            })
        };

        let manifest_value = parse_json("manifest/omni-edition1.manifest.json")?;
        let registry_value = parse_json("registry/rules.json")?;

        // Phase 3: versions before semantics. Only 1.0.0/edition 1 loadable.
        let manifest_version = manifest_value
            .get("manifest_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if manifest_version != "1.0.0" {
            return fail(
                LoadErrorClass::UnsupportedVersion,
                ValidationPhase::Version,
                Some("manifest/omni-edition1.manifest.json".to_string()),
                format!("manifest_version: {manifest_version}"),
            );
        }
        let registry_version =
            registry_value.get("schema_version").and_then(serde_json::Value::as_str).unwrap_or("");
        if registry_version != "1.0.0" {
            return fail(
                LoadErrorClass::UnsupportedVersion,
                ValidationPhase::Version,
                Some("registry/rules.json".to_string()),
                format!("schema_version: {registry_version}"),
            );
        }

        // Phase 4: schema shape. Strict shapes via typed construction with
        // unknown-field rejection; schemas themselves structurally checked.
        let manifest: Manifest = serde_json::from_value(manifest_value).map_err(|e| LoadError {
            class: LoadErrorClass::SchemaViolation,
            phase: ValidationPhase::Schema,
            artifact: Some("manifest/omni-edition1.manifest.json".to_string()),
            detail: e.to_string(),
        })?;
        if manifest.edition != 1 {
            return fail(
                LoadErrorClass::UnsupportedVersion,
                ValidationPhase::Version,
                Some("manifest/omni-edition1.manifest.json".to_string()),
                format!("edition: {}", manifest.edition),
            );
        }
        let registry: Registry = serde_json::from_value(registry_value).map_err(|e| LoadError {
            class: LoadErrorClass::SchemaViolation,
            phase: ValidationPhase::Schema,
            artifact: Some("registry/rules.json".to_string()),
            detail: e.to_string(),
        })?;

        // Registry load integrity: duplicates, statuses, dependency existence.
        // (Cycle detection and witness resolution stay with omni-audit;
        // Ratified policy stays with omni-conform; seams documented.)
        let mut seen = BTreeSet::new();
        for rule in &registry.rules {
            if !seen.insert(rule.rule_id.clone()) {
                return fail(
                    LoadErrorClass::DuplicateArtifact,
                    ValidationPhase::Schema,
                    Some("registry/rules.json".to_string()),
                    format!("duplicate rule {}", rule.rule_id),
                );
            }
            match rule.status.as_str() {
                "Proposed" | "Candidate" | "Ratified" | "Deprecated" | "Superseded"
                | "Withdrawn" => {}
                other => {
                    return fail(
                        LoadErrorClass::SchemaViolation,
                        ValidationPhase::Schema,
                        Some("registry/rules.json".to_string()),
                        format!("{} unknown status {other}", rule.rule_id),
                    );
                }
            }
            if rule.text_hash.len() != 64
                || !rule.text_hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            {
                return fail(
                    LoadErrorClass::SchemaViolation,
                    ValidationPhase::Schema,
                    Some("registry/rules.json".to_string()),
                    format!("malformed text_hash {}", rule.rule_id),
                );
            }
        }
        let ids: BTreeSet<&str> = registry.rules.iter().map(|r| r.rule_id.as_str()).collect();
        for rule in &registry.rules {
            for dep in &rule.dependencies {
                if !ids.contains(dep.as_str()) {
                    return fail(
                        LoadErrorClass::InconsistentReference,
                        ValidationPhase::Consistency,
                        Some("registry/rules.json".to_string()),
                        format!("{} depends on unknown {dep}", rule.rule_id),
                    );
                }
            }
        }

        // Schema publications: required set derived from the single authority
        // (canon publication set), each structurally validated.
        let mut schemas = Vec::new();
        for declared in spec_tree::DECLARED_FILES {
            let Some(name) = declared.strip_prefix("schemas/") else { continue };
            let rel = format!("schemas/{name}");
            let raw = read_contained(&rel)?;
            let text = std::str::from_utf8(&raw).map_err(|e| LoadError {
                class: LoadErrorClass::MalformedArtifact,
                phase: ValidationPhase::Parse,
                artifact: Some(rel.clone()),
                detail: e.to_string(),
            })?;
            omni_canon::reject_duplicate_keys(text).map_err(|e| LoadError {
                class: LoadErrorClass::MalformedArtifact,
                phase: ValidationPhase::Parse,
                artifact: Some(rel.clone()),
                detail: e.to_string(),
            })?;
            let value: serde_json::Value = serde_json::from_str(text).map_err(|e| LoadError {
                class: LoadErrorClass::MalformedArtifact,
                phase: ValidationPhase::Parse,
                artifact: Some(rel.clone()),
                detail: e.to_string(),
            })?;
            if value.get("$schema").and_then(serde_json::Value::as_str) != Some(DRAFT07) {
                return fail(
                    LoadErrorClass::SchemaViolation,
                    ValidationPhase::Schema,
                    Some(rel.clone()),
                    "schema dialect must be draft-07".to_string(),
                );
            }
            if value.get("title").and_then(serde_json::Value::as_str).is_none_or(str::is_empty) {
                return fail(
                    LoadErrorClass::SchemaViolation,
                    ValidationPhase::Schema,
                    Some(rel.clone()),
                    "schema title missing".to_string(),
                );
            }
            schemas.push(SchemaDescriptor {
                digest: omni_canon::sha256_of_normalized_bytes(&raw),
                rel_path: rel,
            });
        }

        // Grammar publication: identity only, no semantic interpretation.
        let grammar_raw = read_contained("grammar/omni-edition1.ebnf")?;
        if grammar_raw.is_empty() {
            return fail(
                LoadErrorClass::MalformedArtifact,
                ValidationPhase::Parse,
                Some("grammar/omni-edition1.ebnf".to_string()),
                "empty grammar publication".to_string(),
            );
        }
        let grammar = GrammarPublication {
            rel_path: "grammar/omni-edition1.ebnf".to_string(),
            digest: omni_canon::sha256_of_normalized_bytes(&grammar_raw),
            byte_len: grammar_raw.len() as u64,
        };

        // Models/data: explicit empty states. Any present file fails closed:
        // no model schema is registered, so nothing is validatable.
        for domain in ["models", "data"] {
            let mut entries: Vec<String> = Vec::new();
            let dir = spec_root.join(domain);
            if dir.is_dir() {
                let mut names: Vec<std::ffi::OsString> = Vec::new();
                for entry in fs::read_dir(&dir).map_err(|e| LoadError {
                    class: LoadErrorClass::MalformedArtifact,
                    phase: ValidationPhase::Parse,
                    artifact: Some(domain.to_string()),
                    detail: e.to_string(),
                })? {
                    names.push(
                        entry
                            .map_err(|e| LoadError {
                                class: LoadErrorClass::MalformedArtifact,
                                phase: ValidationPhase::Parse,
                                artifact: Some(domain.to_string()),
                                detail: e.to_string(),
                            })?
                            .file_name(),
                    );
                }
                names.sort();
                for name in names {
                    let name_str = name.to_string_lossy().into_owned();
                    if name_str != ".gitkeep" {
                        entries.push(format!("{domain}/{name_str}"));
                    }
                }
            }
            if let Some(first) = entries.first() {
                return fail(
                    LoadErrorClass::UndeclaredArtifact,
                    ValidationPhase::Consistency,
                    Some(first.clone()),
                    "model/data artifact present with no registered model schema".to_string(),
                );
            }
        }

        // Phase 5+6: canonical identity via the canon engine, then digest
        // verification against the manifest binding. Inputs are never mutated
        // before verification; the engine output is compared, not edited.
        let mut artifacts = BTreeMap::new();
        for declared in spec_tree::DECLARED_FILES {
            let raw = read_contained(declared)?;
            artifacts.insert(declared.to_string(), omni_canon::sha256_of_normalized_bytes(&raw));
        }
        let (tree_digest, _) = spec_tree::spec_tree_digest(&spec_root).map_err(|e| LoadError {
            class: LoadErrorClass::CanonicalizationFailure,
            phase: ValidationPhase::Canonical,
            artifact: None,
            detail: e.to_string(),
        })?;
        if tree_digest != manifest.spec_tree_sha256 {
            return fail(
                LoadErrorClass::ManifestMismatch,
                ValidationPhase::Digest,
                Some("manifest/omni-edition1.manifest.json".to_string()),
                format!("manifest={} computed={}", manifest.spec_tree_sha256, tree_digest),
            );
        }

        Ok(Self {
            spec_root,
            manifest,
            registry,
            schemas,
            grammar,
            models: EmptyDomain(()),
            data: EmptyDomain(()),
            identity: SpecIdentity { tree_digest, artifacts },
        })
    }

    pub fn load(self) -> Result<LoadedSpecification, LoadError> {
        let provenance = LoadProvenance {
            loader_version: LOADER_VERSION.to_string(),
            tree_digest: self.identity.tree_digest.clone(),
            artifact_count: self.identity.artifacts.len(),
            ordering: "lexicographic-rel-path".to_string(),
        };
        Ok(LoadedSpecification {
            manifest: self.manifest,
            registry: self.registry,
            schemas: self.schemas,
            grammar: self.grammar,
            models: self.models,
            data: self.data,
            identity: self.identity,
            provenance,
            spec_root: self.spec_root,
        })
    }
}

/// Immutable loaded specification. No mutators, no interior mutability, no
/// public constructor: only [`RawSpecification::validate`] then `load()`.
#[derive(Debug, Clone)]
pub struct LoadedSpecification {
    manifest: Manifest,
    registry: Registry,
    schemas: Vec<SchemaDescriptor>,
    grammar: GrammarPublication,
    models: EmptyDomain,
    data: EmptyDomain,
    identity: SpecIdentity,
    provenance: LoadProvenance,
    spec_root: PathBuf,
}

impl LoadedSpecification {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    pub fn schemas(&self) -> &[SchemaDescriptor] {
        &self.schemas
    }
    pub fn grammar(&self) -> &GrammarPublication {
        &self.grammar
    }
    pub fn models(&self) -> &EmptyDomain {
        &self.models
    }
    pub fn data(&self) -> &EmptyDomain {
        &self.data
    }
    pub fn identity(&self) -> &SpecIdentity {
        &self.identity
    }
    pub fn provenance(&self) -> &LoadProvenance {
        &self.provenance
    }
    pub fn spec_root(&self) -> &Path {
        &self.spec_root
    }
}

/// Convenience: validate and load in one step.
pub fn load_specification(spec_root: PathBuf) -> Result<LoadedSpecification, LoadError> {
    RawSpecification::new(spec_root).validate()?.load()
}

#[cfg(test)]
mod loader_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn rule(id: &str, status: &str) -> String {
        format!(
            "{{\"rule_id\":\"{id}\",\"domain\":\"OMNI-LEX\",\"status\":\"{status}\",\
             \"normative\":true,\"text_hash\":\"{}\",\"dependencies\":[],\"witness_tests\":[]}}",
            "a".repeat(64)
        )
    }

    /// Fixture spec tree: manifest + registry + all required schemas +
    /// grammar, with the manifest bound to the computed digest unless
    /// overridden. Models/data stay absent (explicit empty state).
    fn fixture_tree(
        rules: &[String],
        manifest_digest: Option<&str>,
        extra: &[(&str, &[u8])],
    ) -> PathBuf {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("omni-loader-{id}",));
        for dir in ["manifest", "registry", "schemas", "models", "data", "grammar"] {
            fs::create_dir_all(root.join(dir)).expect("mkdir");
        }
        fs::write(
            root.join("registry/rules.json"),
            format!("{{\"schema_version\":\"1.0.0\",\"rules\":[{}]}}", rules.join(",")),
        )
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
                format!("{{\"$schema\":\"{DRAFT07}\",\"title\":\"{schema}\",\"type\":\"object\"}}"),
            )
            .expect("write");
        }
        fs::write(root.join("grammar/omni-edition1.ebnf"), b"(* t *)\n").expect("write");
        for (rel, bytes) in extra {
            let path = root.join(rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("mkdir parent");
            }
            fs::write(&path, bytes).expect("write");
        }
        // Trees that are intentionally unbindable (extra artifacts) fall back
        // to a dummy bound; the load under test fails before comparison.
        let bound = manifest_digest.map(ToString::to_string).unwrap_or_else(|| {
            spec_tree::spec_tree_digest(&root)
                .map(|(digest, _)| digest)
                .unwrap_or_else(|_| "0".repeat(64))
        });
        fs::write(
            root.join("manifest/omni-edition1.manifest.json"),
            format!(
                "{{\"manifest_version\":\"1.0.0\",\"edition\":1,\"status\":\"t\",\
                 \"spec_tree_sha256\":\"{bound}\",\"modules\":[\"OMNI-LEX\"]}}"
            ),
        )
        .expect("write");
        root
    }

    fn minimal_rules() -> Vec<String> {
        vec![rule("LEX-0001", "Candidate")]
    }

    #[test]
    fn current_tree_loads_with_empty_model_states() {
        let root = fixture_tree(&minimal_rules(), None, &[]);
        let loaded = load_specification(root.clone()).expect("load");
        assert_eq!(loaded.registry().rules.len(), 1);
        assert_eq!(loaded.schemas().len(), 8);
        assert_eq!(loaded.models(), &EmptyDomain(()));
        assert_eq!(loaded.data(), &EmptyDomain(()));
        assert_eq!(loaded.identity().artifacts.len(), 10);
        assert_eq!(loaded.provenance().loader_version, LOADER_VERSION);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn repeated_loads_agree() {
        let root = fixture_tree(&minimal_rules(), None, &[]);
        let first = load_specification(root.clone()).expect("load");
        let second = load_specification(root.clone()).expect("load");
        assert_eq!(first.identity().tree_digest, second.identity().tree_digest);
        assert_eq!(first.identity().artifacts, second.identity().artifacts);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn malformed_and_version_failures_classified() {
        // Malformed JSON.
        let root = fixture_tree(&minimal_rules(), None, &[("registry/rules.json", b"{oops")]);
        let err = load_specification(root.clone()).expect_err("malformed");
        assert_eq!(err.class, LoadErrorClass::MalformedArtifact);
        assert_eq!(err.phase, ValidationPhase::Parse);
        fs::remove_dir_all(&root).ok();
        // Duplicate keys rejected at the boundary.
        let root = fixture_tree(
            &minimal_rules(),
            None,
            &[("registry/rules.json", br#"{"schema_version":"1.0.0","rules":[],"rules":[]}"#)],
        );
        let err = load_specification(root.clone()).expect_err("dup keys");
        assert_eq!(err.class, LoadErrorClass::MalformedArtifact);
        fs::remove_dir_all(&root).ok();
        // Future manifest version.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        let raw = fs::read_to_string(root.join("manifest/omni-edition1.manifest.json")).expect("r");
        fs::write(
            root.join("manifest/omni-edition1.manifest.json"),
            raw.replace("\"1.0.0\"", "\"2.0.0\""),
        )
        .expect("w");
        let err = load_specification(root.clone()).expect_err("future");
        assert_eq!(err.class, LoadErrorClass::UnsupportedVersion);
        fs::remove_dir_all(&root).ok();
        // Unknown field under strict manifest shape.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        let raw = fs::read_to_string(root.join("manifest/omni-edition1.manifest.json")).expect("r");
        fs::write(
            root.join("manifest/omni-edition1.manifest.json"),
            raw.replace("\"edition\":1", "\"edition\":1,\"surprise\":true"),
        )
        .expect("w");
        let err = load_specification(root.clone()).expect_err("unknown field");
        assert_eq!(err.class, LoadErrorClass::SchemaViolation);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn trust_states_distinguished() {
        // Missing required artifact.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        fs::remove_file(root.join("registry/rules.json")).expect("rm");
        let err = load_specification(root.clone()).expect_err("missing");
        assert_eq!(err.class, LoadErrorClass::MissingArtifact);
        fs::remove_dir_all(&root).ok();
        // Present-but-undeclared model artifact fails; it is never invented.
        let root = fixture_tree(&minimal_rules(), None, &[("models/m1.json", b"{}")]);
        let err = load_specification(root.clone()).expect_err("undeclared");
        assert_eq!(err.class, LoadErrorClass::UndeclaredArtifact);
        fs::remove_dir_all(&root).ok();
        // Extra publication artifact where forbidden (canon boundary).
        let root = fixture_tree(&minimal_rules(), None, &[("schemas/extra.schema.json", b"{}")]);
        let err = load_specification(root.clone()).expect_err("extra");
        assert!(
            err.class == LoadErrorClass::CanonicalizationFailure
                || err.class == LoadErrorClass::UndeclaredArtifact,
            "got: {err:?}"
        );
        fs::remove_dir_all(&root).ok();
        // Duplicate logical artifact.
        let root = fixture_tree(
            &[rule("LEX-0001", "Candidate"), rule("LEX-0001", "Candidate")],
            None,
            &[],
        );
        let err = load_specification(root.clone()).expect_err("dup rule");
        assert_eq!(err.class, LoadErrorClass::DuplicateArtifact);
        fs::remove_dir_all(&root).ok();
        // Unknown dependency reference.
        let root = fixture_tree(
            &[rule("LEX-0001", "Candidate")
                .replace("\"dependencies\":[]", "\"dependencies\":[\"GHOST-0001\"]")],
            None,
            &[],
        );
        let err = load_specification(root.clone()).expect_err("unknown dep");
        assert_eq!(err.class, LoadErrorClass::InconsistentReference);
        fs::remove_dir_all(&root).ok();
        // Unknown lifecycle status.
        let root = fixture_tree(&[rule("LEX-0001", "Erratumish")], None, &[]);
        let err = load_specification(root.clone()).expect_err("status");
        assert_eq!(err.class, LoadErrorClass::SchemaViolation);
        fs::remove_dir_all(&root).ok();
        // Corrupt schema (wrong dialect).
        let root = fixture_tree(
            &minimal_rules(),
            None,
            &[("schemas/diagnostic.schema.json", br#"{"$schema":"draft-99","title":"x"}"#)],
        );
        let err = load_specification(root.clone()).expect_err("dialect");
        assert_eq!(err.class, LoadErrorClass::SchemaViolation);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn digest_and_manifest_mismatch_classified() {
        // Tampered file after binding.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        fs::write(root.join("grammar/omni-edition1.ebnf"), b"(* tampered *)\n").expect("w");
        let err = load_specification(root.clone()).expect_err("tamper");
        assert_eq!(err.class, LoadErrorClass::ManifestMismatch);
        assert_eq!(err.phase, ValidationPhase::Digest);
        fs::remove_dir_all(&root).ok();
        // Wrong digest bound.
        let root = fixture_tree(&minimal_rules(), Some(&"f".repeat(64)), &[]);
        let err = load_specification(root.clone()).expect_err("bound");
        assert_eq!(err.class, LoadErrorClass::ManifestMismatch);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn live_repository_loads() {
        // The actual normative tree (relative to this crate) loads with the
        // full 560-rule registry and explicit empty model/data states.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec");
        let loaded = load_specification(root).expect("live load");
        assert_eq!(loaded.registry().rules.len(), 560);
        assert_eq!(loaded.schemas().len(), 8);
        assert_eq!(loaded.models(), &EmptyDomain(()));
        assert_eq!(loaded.data(), &EmptyDomain(()));
        assert_eq!(loaded.identity().tree_digest, loaded.manifest().spec_tree_sha256);
    }
}
