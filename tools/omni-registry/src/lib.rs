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
    UnsupportedLayout,
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
    /// Normative erratum overlays (REL-0007 mechanics): tree-relative paths
    /// resolved against the publication set. Absent means no overlays. Every
    /// listed overlay must carry a machine-readable `rel-0007` metadata block
    /// whose declared state agrees with this manifest (see
    /// `validate_rel0007_overlay`), so a draft can never be read as applied.
    #[serde(default)]
    pub errata: Vec<String>,
}

impl Manifest {
    /// Typed Stage-0 predicate lists from the loaded manifest. Presence,
    /// array shape, and string elements are enforced (no silent drops);
    /// allowed/forbidden disjointness is enforced by the predicate engine
    /// at construction. Returns `(allowed, forbidden)`.
    pub fn stage0_feature_sets(&self) -> Result<(Vec<String>, Vec<String>), LoadError> {
        let artifact = Some("manifest/omni-edition1.manifest.json".to_string());
        let get_list = |key: &str| -> Result<Vec<String>, LoadError> {
            let items = self.stage0_feature_predicates.get(key).and_then(|v| v.as_array()).ok_or(
                LoadError {
                    class: LoadErrorClass::MalformedArtifact,
                    phase: ValidationPhase::Schema,
                    artifact: artifact.clone(),
                    detail: format!("stage0_feature_predicates.{key} missing or not an array"),
                },
            )?;
            items
                .iter()
                .map(|v| {
                    v.as_str().map(ToString::to_string).ok_or(LoadError {
                        class: LoadErrorClass::MalformedArtifact,
                        phase: ValidationPhase::Schema,
                        artifact: artifact.clone(),
                        detail: format!("stage0_feature_predicates.{key} has non-string entry"),
                    })
                })
                .collect()
        };
        let allowed = get_list("allowed")?;
        let forbidden = get_list("forbidden")?;
        if allowed.is_empty() {
            return fail(
                LoadErrorClass::MalformedArtifact,
                ValidationPhase::Schema,
                artifact.clone(),
                "stage0_feature_predicates.allowed is empty".to_string(),
            );
        }
        let allowed_set: BTreeSet<&str> = allowed.iter().map(String::as_str).collect();
        if let Some(conflict) = forbidden.iter().find(|f| allowed_set.contains(f.as_str())) {
            return fail(
                LoadErrorClass::InconsistentReference,
                ValidationPhase::Consistency,
                artifact.clone(),
                format!("stage0 feature both allowed and forbidden: {conflict}"),
            );
        }
        Ok((allowed, forbidden))
    }
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

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleText {
    rule_id: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleTexts {
    schema_version: String,
    texts: Vec<RuleText>,
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

/// Machine shape of a rule identity (`PREFIX-NNNN`, digit-bearing prefixes
/// included; `VIBE-GRAM` handled explicitly). Shared by the loader and its
/// tests so overlay reference scans cannot drift from table extraction.
pub fn is_table_rule_id(id: &str) -> bool {
    if let Some(rest) = id.strip_prefix("VIBE-GRAM-") {
        return rest.len() == 4 && rest.chars().all(|c| c.is_ascii_digit());
    }
    match id.split_once('-') {
        Some((head, tail)) => {
            !head.is_empty()
                && head.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                && tail.len() == 4
                && tail.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Canonical rule lifecycle states (RULE-0003; spelling fixed by
/// docs/adr/0001-erratum-corrected-lifecycle-state.md). Single source of
/// truth consumed by the loader, the audit, conformance, and evidence
/// validation instead of parallel local copies.
pub const LIFECYCLE_STATES: [&str; 7] = [
    "Proposed",
    "Candidate",
    "Ratified",
    "Deprecated",
    "Superseded",
    "Withdrawn",
    "ErratumCorrected",
];

/// States an implementation claim may target.
pub fn is_ownable_status(status: &str) -> bool {
    matches!(status, "Candidate" | "Ratified")
}

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
            if !LIFECYCLE_STATES.contains(&rule.status.as_str()) {
                return fail(
                    LoadErrorClass::SchemaViolation,
                    ValidationPhase::Schema,
                    Some("registry/rules.json".to_string()),
                    format!("{} unknown status {}", rule.rule_id, rule.status),
                );
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

        // Mechanical rule-text/hash binding (gate closure 2): every stored
        // hash must rederive from the committed normative text. Stale or
        // mutated hashes fail here, not in a human-rerun script.
        let texts_raw = read_contained("registry/rule-texts.json")?;
        let texts_text = std::str::from_utf8(&texts_raw).map_err(|e| LoadError {
            class: LoadErrorClass::MalformedArtifact,
            phase: ValidationPhase::Parse,
            artifact: Some("registry/rule-texts.json".to_string()),
            detail: e.to_string(),
        })?;
        omni_canon::reject_duplicate_keys(texts_text).map_err(|e| LoadError {
            class: LoadErrorClass::MalformedArtifact,
            phase: ValidationPhase::Parse,
            artifact: Some("registry/rule-texts.json".to_string()),
            detail: e.to_string(),
        })?;
        let texts: RuleTexts = serde_json::from_str(texts_text).map_err(|e| LoadError {
            class: LoadErrorClass::MalformedArtifact,
            phase: ValidationPhase::Parse,
            artifact: Some("registry/rule-texts.json".to_string()),
            detail: e.to_string(),
        })?;
        if texts.schema_version != "1.0.0" {
            return fail(
                LoadErrorClass::UnsupportedVersion,
                ValidationPhase::Version,
                Some("registry/rule-texts.json".to_string()),
                format!("schema_version: {}", texts.schema_version),
            );
        }
        let mut text_by_id = BTreeMap::new();
        for entry in &texts.texts {
            if text_by_id.insert(entry.rule_id.clone(), entry.text.clone()).is_some() {
                return fail(
                    LoadErrorClass::DuplicateArtifact,
                    ValidationPhase::Schema,
                    Some("registry/rule-texts.json".to_string()),
                    format!("duplicate text {}", entry.rule_id),
                );
            }
        }
        for rule in &registry.rules {
            match text_by_id.remove(&rule.rule_id) {
                None => {
                    return fail(
                        LoadErrorClass::InconsistentReference,
                        ValidationPhase::Consistency,
                        Some("registry/rule-texts.json".to_string()),
                        format!("rule {} has no normative text", rule.rule_id),
                    );
                }
                Some(text) => {
                    let derived = omni_canon::rule_text_hash(&text);
                    if derived != rule.text_hash {
                        return fail(
                            LoadErrorClass::InconsistentReference,
                            ValidationPhase::Consistency,
                            Some("registry/rules.json".to_string()),
                            format!("stale text hash for {}", rule.rule_id),
                        );
                    }
                }
            }
        }
        if let Some(orphan) = text_by_id.keys().next() {
            return fail(
                LoadErrorClass::InconsistentReference,
                ValidationPhase::Consistency,
                Some("registry/rule-texts.json".to_string()),
                format!("text without rule: {orphan}"),
            );
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

        // Erratum overlays: every listed path must be a contained tree
        // artifact, and every rule identity it cites must resolve in the
        // registry (typo/drift-proofing without hardcoding overlay content).
        for overlay in &manifest.errata {
            let parsed = Path::new(overlay);
            if parsed.is_absolute()
                || parsed.components().any(|c| {
                    matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir)
                })
            {
                return fail(
                    LoadErrorClass::ContainmentViolation,
                    ValidationPhase::Containment,
                    Some(overlay.clone()),
                    "erratum path escapes the tree".to_string(),
                );
            }
            let raw = read_contained(overlay)?;
            let text = std::str::from_utf8(&raw).map_err(|e| LoadError {
                class: LoadErrorClass::MalformedArtifact,
                phase: ValidationPhase::Parse,
                artifact: Some(overlay.clone()),
                detail: e.to_string(),
            })?;
            if text.trim().is_empty() {
                return fail(
                    LoadErrorClass::MalformedArtifact,
                    ValidationPhase::Parse,
                    Some(overlay.clone()),
                    "empty erratum overlay".to_string(),
                );
            }
            for token in text.split(|c: char| {
                !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '/')
            }) {
                let id = token.trim_matches('/');
                if id.contains('-') && is_table_rule_id(id) && !ids.contains(id) {
                    return fail(
                        LoadErrorClass::InconsistentReference,
                        ValidationPhase::Consistency,
                        Some(overlay.clone()),
                        format!("erratum cites unknown rule {id}"),
                    );
                }
            }
            validate_rel0007_overlay(text, &manifest, overlay)?;
        }

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

/// Anchor specification discovery (gate closure 4). Resolution order:
/// explicit `OMNI_SPEC_ROOT`, else walk up from the invocation directory to
/// the first ancestor holding `spec/manifest/omni-edition1.manifest.json`.
/// Every CWD under one repository resolves to the same tree; anything else
/// fails closed. The manifest file itself is the anchor: no Cargo.toml
/// coupling, no silent fallback.
pub fn discover_spec_root() -> Result<PathBuf, LoadError> {
    if let Ok(explicit) = std::env::var("OMNI_SPEC_ROOT") {
        let path = PathBuf::from(&explicit);
        return validate_spec_dir(&path).map_err(|_| LoadError {
            class: LoadErrorClass::MissingArtifact,
            phase: ValidationPhase::Containment,
            artifact: None,
            detail: format!("OMNI_SPEC_ROOT does not anchor a specification: {explicit}"),
        });
    }
    let mut dir = std::env::current_dir().map_err(|e| LoadError {
        class: LoadErrorClass::ContainmentViolation,
        phase: ValidationPhase::Containment,
        artifact: None,
        detail: format!("invocation directory unreadable: {e}"),
    })?;
    loop {
        let candidate = dir.join("spec");
        if validate_spec_dir(&candidate).is_ok() {
            return Ok(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    fail(
        LoadErrorClass::MissingArtifact,
        ValidationPhase::Containment,
        None,
        "no anchored specification: no ancestor holds spec/manifest/omni-edition1.manifest.json"
            .to_string(),
    )
}

fn validate_spec_dir(path: &Path) -> Result<PathBuf, LoadError> {
    if path.join("manifest/omni-edition1.manifest.json").is_file() {
        Ok(path.to_path_buf())
    } else {
        fail(
            LoadErrorClass::MissingArtifact,
            ValidationPhase::Containment,
            Some(path.display().to_string()),
            "not a specification tree".to_string(),
        )
    }
}

/// Workspace root for a discovered tree under the standard layout
/// (`<workspace>/spec`). Anything else fails: workspace-relative features
/// must not guess.
pub fn workspace_root_for_spec(spec_root: &Path) -> Result<PathBuf, LoadError> {
    if spec_root.file_name().is_some_and(|n| n == "spec") {
        if let Some(parent) = spec_root.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    fail(
        LoadErrorClass::UnsupportedLayout,
        ValidationPhase::Containment,
        Some(spec_root.display().to_string()),
        "specification tree is not in standard <workspace>/spec layout".to_string(),
    )
}

/// REL-0007 machine-readable overlay metadata. Every manifest-listed erratum
/// overlay must carry exactly this shape inside a fenced `rel-0007` block;
/// the surrounding prose explains the block but never extends it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rel0007Overlay {
    overlay_id: String,
    classification: String,
    status: String,
    effective_releases: Vec<String>,
    replacement_source_path: String,
    replacement_source_sha256: String,
    implementation_impact: String,
    migration: String,
    immutability: String,
    signature_state: String,
}

/// Extract the body of the first fenced `rel-0007` block, if present.
fn rel0007_block(text: &str) -> Option<String> {
    let mut inside = false;
    let mut body = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !inside {
            inside = trimmed == "```rel-0007";
            continue;
        }
        if trimmed == "```" {
            return Some(body);
        }
        body.push_str(line);
        body.push('\n');
    }
    None
}

/// Validate the REL-0007 elements an overlay declares against the manifest
/// that lists it. Fail-closed on a missing element, a classification or
/// status mismatch, and any attempt to mark an overlay applied without a
/// signature and a signed (non-candidate) base release. This is what keeps a
/// Candidate-2 amendment from being read as a promoted baseline.
fn validate_rel0007_overlay(
    text: &str,
    manifest: &Manifest,
    overlay: &str,
) -> Result<(), LoadError> {
    let artifact = Some(overlay.to_string());
    let malformed = |detail: String| LoadError {
        class: LoadErrorClass::MalformedArtifact,
        phase: ValidationPhase::Schema,
        artifact: artifact.clone(),
        detail,
    };
    let raw = rel0007_block(text).ok_or_else(|| {
        malformed("erratum overlay has no ```rel-0007 metadata block".to_string())
    })?;
    let meta: Rel0007Overlay =
        serde_json::from_str(&raw).map_err(|e| malformed(format!("rel-0007 metadata: {e}")))?;

    if meta.overlay_id.trim().is_empty() {
        return Err(malformed("rel-0007 overlay_id is empty".to_string()));
    }
    if meta.classification != "rel-0007-erratum-overlay" {
        return Err(malformed(format!(
            "rel-0007 classification must be rel-0007-erratum-overlay, got {}",
            meta.classification
        )));
    }
    if meta.status != "pre-release" && meta.status != "applied" {
        return Err(malformed(format!(
            "rel-0007 status must be pre-release or applied, got {}",
            meta.status
        )));
    }
    if meta.signature_state != "pending" && meta.signature_state != "attached" {
        return Err(malformed(format!(
            "rel-0007 signature_state must be pending or attached, got {}",
            meta.signature_state
        )));
    }
    if meta.immutability != "spec-tree-sha256" {
        return Err(malformed(format!(
            "rel-0007 immutability must be spec-tree-sha256, got {}",
            meta.immutability
        )));
    }
    if meta.effective_releases.is_empty()
        || meta.effective_releases.iter().any(|r| r.trim().is_empty())
    {
        return Err(malformed(
            "rel-0007 effective_releases must name at least one release".to_string(),
        ));
    }
    let base = manifest.status.as_deref().unwrap_or_default();
    if base.trim().is_empty() {
        return Err(malformed(
            "base manifest declares no status, so overlay effectiveness is undefined".to_string(),
        ));
    }
    if !meta.effective_releases.iter().any(|r| r == base) {
        return Err(malformed(format!(
            "rel-0007 effective_releases must name the base manifest status {base}"
        )));
    }
    if meta.implementation_impact.trim().is_empty() {
        return Err(malformed("rel-0007 implementation_impact is empty".to_string()));
    }
    if meta.migration.trim().is_empty() {
        return Err(malformed("rel-0007 migration is empty".to_string()));
    }
    let source = Path::new(&meta.replacement_source_path);
    if meta.replacement_source_path.trim().is_empty()
        || source.is_absolute()
        || source
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir))
    {
        return Err(malformed(
            "rel-0007 replacement_source_path must be a relative path".to_string(),
        ));
    }
    if meta.replacement_source_sha256.len() != 64
        || !meta
            .replacement_source_sha256
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    {
        return Err(malformed(
            "rel-0007 replacement_source_sha256 must be 64 lowercase hex digits".to_string(),
        ));
    }
    if meta.status == "applied" {
        if meta.signature_state != "attached" {
            return Err(malformed(
                "rel-0007 applied overlay must be signed (signature_state attached)".to_string(),
            ));
        }
        if base.to_ascii_lowercase().contains("candidate") {
            return Err(malformed(format!(
                "rel-0007 overlay cannot be applied to candidate base release {base}"
            )));
        }
    }
    Ok(())
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
        // Coherent fixture hashes: every rule gets a texts entry whose hash
        // is rederived, so the loader's mechanical binding holds by
        // construction and tests exercise targeted mutations only.
        let mut parsed: serde_json::Value =
            serde_json::from_str(&format!("{{\"rules\":[{}]}}", rules.join(","))).expect("rules");
        let mut texts = String::from("{\"schema_version\":\"1.0.0\",\"texts\":[");
        let mut first = true;
        for rule in parsed["rules"].as_array_mut().expect("array") {
            let id = rule["rule_id"].as_str().expect("id").to_string();
            let text = format!("fixture text for {id}");
            rule["text_hash"] = serde_json::Value::String(omni_canon::rule_text_hash(&text));
            if !first {
                texts.push(',');
            }
            first = false;
            texts.push_str(&format!("{{\"rule_id\":\"{id}\",\"text\":\"{text}\"}}"));
        }
        texts.push_str("]}");
        fs::write(
            root.join("registry/rules.json"),
            format!(
                "{{\"schema_version\":\"1.0.0\",\"rules\":{}}}",
                serde_json::to_string(&parsed["rules"]).expect("serialize")
            ),
        )
        .expect("write");
        fs::write(root.join("registry/rule-texts.json"), texts).expect("write");
        // Minimal erratum overlay: present for the publication set and
        // carrying a valid pre-release REL-0007 block, so overlay validation
        // runs on every load instead of being skipped by absent metadata.
        fs::write(
            root.join("grammar/candidate2-erratum.md"),
            fixture_erratum("pre-release", "t", "pending"),
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
                 \"spec_tree_sha256\":\"{bound}\",\"modules\":[\"OMNI-LEX\"],\
                 \"errata\":[\"grammar/candidate2-erratum.md\"]}}"
            ),
        )
        .expect("write");
        root
    }

    /// Fixture overlay with a valid pre-release REL-0007 metadata block bound
    /// to the given manifest status, so overlay state can be mutated per test.
    fn fixture_erratum(status: &str, effective: &str, signature: &str) -> String {
        format!(
            "# fixture erratum\n\n```rel-0007\n{{\n  \"overlay_id\": \"fixture-overlay\",\n  \
             \"classification\": \"rel-0007-erratum-overlay\",\n  \"status\": \"{status}\",\n  \
             \"effective_releases\": [\"{effective}\"],\n  \
             \"replacement_source_path\": \"docs/specification/fixture-source.md\",\n  \
             \"replacement_source_sha256\": \"{hash}\",\n  \
             \"implementation_impact\": \"none\",\n  \"migration\": \"none\",\n  \
             \"immutability\": \"spec-tree-sha256\",\n  \"signature_state\": \"{signature}\"\n}}\n```\n",
            hash = "0".repeat(64)
        )
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
        assert_eq!(loaded.identity().artifacts.len(), 12);
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
    fn stale_text_hash_fails_closed() {
        // Mutating a normative text without updating its hash must fail,
        // even though the registry remains structurally valid.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        let raw = fs::read_to_string(root.join("registry/rule-texts.json")).expect("r");
        fs::write(
            root.join("registry/rule-texts.json"),
            raw.replace("fixture text for LEX-0001", "tampered text"),
        )
        .expect("w");
        let err = load_specification(root.clone()).expect_err("stale hash");
        assert_eq!(err.class, LoadErrorClass::InconsistentReference);
        assert!(err.detail.contains("stale text hash"), "got: {err:?}");
        fs::remove_dir_all(&root).ok();
        // Text without a rule fails symmetrically.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        let raw = fs::read_to_string(root.join("registry/rule-texts.json")).expect("r");
        fs::write(
            root.join("registry/rule-texts.json"),
            raw.replace("\"texts\":[", "\"texts\":[{\"rule_id\":\"GHOST-0001\",\"text\":\"x\"},"),
        )
        .expect("w");
        let err = load_specification(root.clone()).expect_err("orphan text");
        assert!(err.detail.contains("text without rule"), "got: {err:?}");
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
    fn lifecycle_authority_agrees_mechanically() {
        // ADR-0001: the schema file's status enum must equal the canonical
        // seven-state set consumed by every validator; the vocabulary is
        // present even though no record uses ErratumCorrected yet.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec");
        let raw = fs::read_to_string(root.join("schemas/rule-registry.schema.json")).expect("r");
        let schema: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        let mut enumerated: Vec<String> = schema["properties"]["rules"]["items"]["properties"]
            ["status"]["enum"]
            .as_array()
            .expect("enum")
            .iter()
            .map(|v| v.as_str().expect("string").to_string())
            .collect();
        enumerated.sort();
        let mut canonical: Vec<String> = LIFECYCLE_STATES.iter().map(ToString::to_string).collect();
        canonical.sort();
        assert_eq!(enumerated, canonical);
        let loaded = load_specification(root).expect("live load");
        assert!(
            !loaded.registry().rules.iter().any(|r| r.status == "ErratumCorrected"),
            "vocabulary present but unpopulated"
        );
    }

    /// The normative specification document is the single source of rule
    /// texts: the committed texts file must equal a fresh table-grammmar
    /// extraction byte-for-byte in content. This replaces the procedural
    /// extraction script with a mechanical in-repo check.
    #[test]
    fn rule_texts_match_normative_document() {
        let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.2_vibe.md",
        );
        let raw = fs::read_to_string(&docs).expect("spec doc");
        let raw = raw.replace("\r\n", "\n");
        let mut extracted = BTreeMap::new();
        for line in raw.split('\n') {
            let line = line.trim_end();
            // Greedy-equivalent row grammar: id is the first cell, text runs
            // to the final pipe, so inner pipes in requirement text survive.
            if !line.starts_with("| `") || !line.ends_with('|') || line.len() < 6 {
                continue;
            }
            let inner = &line[1..line.len() - 1];
            let Some((id_cell, text_cell)) = inner.split_once('|') else { continue };
            let id = id_cell.trim().trim_matches('`');
            if !is_table_rule_id(id) {
                continue;
            }
            let text = text_cell.trim().to_string();
            if extracted.insert(id.to_string(), text).is_some() {
                panic!("duplicate rule row in spec document: {id}");
            }
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec");
        let committed: RuleTexts = serde_json::from_str(
            &fs::read_to_string(root.join("registry/rule-texts.json")).expect("texts"),
        )
        .expect("parse texts");
        let mut committed_map = BTreeMap::new();
        for entry in &committed.texts {
            committed_map.insert(entry.rule_id.clone(), entry.text.clone());
        }
        assert_eq!(extracted, committed_map, "texts file diverged from normative document");
        assert_eq!(extracted.len(), 567);
    }

    #[test]
    fn erratum_overlay_requires_rel0007_block() {
        let root = fixture_tree(&minimal_rules(), None, &[]);
        fs::write(root.join("grammar/candidate2-erratum.md"), "# no metadata\n").expect("write");
        let err = load_specification(root.clone()).expect_err("missing block");
        assert_eq!(err.class, LoadErrorClass::MalformedArtifact);
        assert!(err.to_string().contains("rel-0007"), "{err}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn overlay_effectiveness_must_name_the_base_manifest_status() {
        let root = fixture_tree(&minimal_rules(), None, &[]);
        fs::write(
            root.join("grammar/candidate2-erratum.md"),
            fixture_erratum("pre-release", "9.9.9", "pending"),
        )
        .expect("write");
        let err = load_specification(root.clone()).expect_err("wrong base");
        assert!(err.to_string().contains("base manifest status"), "{err}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn applied_overlay_needs_signature_and_a_signed_base() {
        // Applied without a signature is a claim the process never made.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        fs::write(
            root.join("grammar/candidate2-erratum.md"),
            fixture_erratum("applied", "t", "pending"),
        )
        .expect("write");
        let err = load_specification(root.clone()).expect_err("unsigned applied");
        assert!(err.to_string().contains("must be signed"), "{err}");
        fs::remove_dir_all(&root).ok();

        // Signed but applied to a candidate base would be silent promotion.
        let root = fixture_tree(&minimal_rules(), None, &[]);
        let raw = fs::read_to_string(root.join("manifest/omni-edition1.manifest.json")).expect("r");
        fs::write(
            root.join("manifest/omni-edition1.manifest.json"),
            raw.replace("\"status\":\"t\"", "\"status\":\"1.0.0-candidate.1\""),
        )
        .expect("w");
        fs::write(
            root.join("grammar/candidate2-erratum.md"),
            fixture_erratum("applied", "1.0.0-candidate.1", "attached"),
        )
        .expect("write");
        let err = load_specification(root.clone()).expect_err("candidate base");
        assert!(err.to_string().contains("candidate base release"), "{err}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn published_overlay_declares_the_actual_replacement_source() {
        let overlay = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../spec/grammar/candidate2-erratum.md"),
        )
        .expect("overlay");
        let raw = rel0007_block(&overlay).expect("rel-0007 block");
        let meta: Rel0007Overlay = serde_json::from_str(&raw).expect("metadata");
        let document = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(&meta.replacement_source_path);
        let bytes = fs::read(&document).expect("replacement source exists");
        assert_eq!(
            omni_canon::sha256_of_normalized_bytes(&bytes),
            meta.replacement_source_sha256,
            "declared replacement digest drifted from the document"
        );
        // The overlay must remain pre-release: signature and application are
        // outstanding, and neither is claimed here.
        assert_eq!(meta.status, "pre-release");
        assert_eq!(meta.signature_state, "pending");
        assert_eq!(meta.classification, "rel-0007-erratum-overlay");
    }

    static GLOBAL_GUARD: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct RestoreCwd {
        saved: PathBuf,
    }

    impl Drop for RestoreCwd {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.saved);
        }
    }

    #[test]
    fn discovery_is_cwd_independent() {
        let _lock = GLOBAL_GUARD.lock().expect("lock");
        let saved = std::env::current_dir().expect("cwd");
        let _restore = RestoreCwd { saved };
        let workspace =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("ws");
        let expected = workspace.join("spec").canonicalize().expect("spec");
        // Repository root and a nested crate directory resolve identically.
        std::env::set_current_dir(&workspace).expect("chdir root");
        std::env::remove_var("OMNI_SPEC_ROOT");
        assert_eq!(discover_spec_root().expect("root"), expected);
        std::env::set_current_dir(workspace.join("compiler/omni-lex")).expect("chdir nested");
        std::env::remove_var("OMNI_SPEC_ROOT");
        assert_eq!(discover_spec_root().expect("nested"), expected);
        // Outside any anchored tree fails closed, never silently elsewhere.
        let outside = std::env::temp_dir().join(format!("omni-nowhere-{}", std::process::id()));
        std::fs::create_dir_all(&outside).expect("mkdir");
        std::env::set_current_dir(&outside).expect("chdir outside");
        std::env::remove_var("OMNI_SPEC_ROOT");
        let err = discover_spec_root().expect_err("outside");
        assert_eq!(err.class, LoadErrorClass::MissingArtifact);
        std::fs::remove_dir_all(&outside).ok();
    }

    #[test]
    fn explicit_root_override_wins_or_fails() {
        let _lock = GLOBAL_GUARD.lock().expect("lock");
        let prior = std::env::var("OMNI_SPEC_ROOT").ok();
        let workspace =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("ws");
        let expected = workspace.join("spec").canonicalize().expect("spec");
        std::env::set_var("OMNI_SPEC_ROOT", &expected);
        assert_eq!(discover_spec_root().expect("override"), expected);
        let nowhere = std::env::temp_dir().join(format!("omni-nowhere-{}", std::process::id()));
        std::fs::create_dir_all(&nowhere).expect("mkdir");
        std::env::set_var("OMNI_SPEC_ROOT", &nowhere);
        assert!(discover_spec_root().is_err());
        std::fs::remove_dir_all(&nowhere).ok();
        match prior {
            Some(value) => std::env::set_var("OMNI_SPEC_ROOT", value),
            None => std::env::remove_var("OMNI_SPEC_ROOT"),
        }
    }

    #[test]
    fn workspace_layout_is_explicit() {
        let workspace =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("ws");
        let spec = workspace.join("spec");
        assert_eq!(workspace_root_for_spec(&spec).expect("layout"), workspace);
        let odd = std::env::temp_dir();
        let err = workspace_root_for_spec(&odd).expect_err("layout");
        assert_eq!(err.class, LoadErrorClass::UnsupportedLayout);
    }

    #[test]
    fn manifest_permits_empty_model_data_domains() {
        // The manifest declares module NAMES, never model/data FILES; empty
        // domains are therefore contract-consistent, verified explicitly.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec");
        let raw = fs::read_to_string(root.join("manifest/omni-edition1.manifest.json")).expect("r");
        let manifest: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert!(manifest.get("models").is_none(), "no file-level model declarations");
        assert!(manifest.get("data").is_none(), "no file-level data declarations");
        let loaded = load_specification(root).expect("live load");
        assert_eq!(loaded.models(), &EmptyDomain(()));
        assert_eq!(loaded.data(), &EmptyDomain(()));
    }

    #[test]
    fn live_repository_loads() {
        // The actual normative tree (relative to this crate) loads with the
        // full 567-rule registry and explicit empty model/data states.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec");
        let loaded = load_specification(root).expect("live load");
        assert_eq!(loaded.registry().rules.len(), 567);
        assert_eq!(loaded.schemas().len(), 8);
        assert_eq!(loaded.models(), &EmptyDomain(()));
        assert_eq!(loaded.data(), &EmptyDomain(()));
        assert_eq!(loaded.identity().tree_digest, loaded.manifest().spec_tree_sha256);
    }
}
