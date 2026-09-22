//! Machine-readable evidence contracts (0.0.0.8).
//!
//! Single authority for the seven evidence domains: diagnostics, witnesses,
//! conformance outcomes, verification failures, provenance records, regression
//! metadata, and fuzz-promotion records. JSON Schema files under
//! `spec/schemas/` are the documentary contracts; these Rust types plus the
//! validators below are the enforced contracts. Canonical bytes always come
//! from the `omni-canon` engine; nothing here reimplements canonicalization.
//!
//! Resolution model: every rule reference must exist in the supplied registry
//! view. References to `Superseded`/`Deprecated`/`Withdrawn`/`Proposed` rules
//! fail wherever a live (Candidate/Ratified) rule is required. File references
//! must be contained relative paths resolving against a caller-supplied file
//! set; absolute, escaping, or missing references fail closed.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use omni_canon::CanonError;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub struct EvidenceError(pub String);

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "evidence error: {}", self.0)
    }
}

impl std::error::Error for EvidenceError {}

impl EvidenceError {
    /// Inspect the message (used by consumers asserting failure classes).
    pub fn contains(&self, pat: &str) -> bool {
        self.0.contains(pat)
    }
}

fn fail<T>(msg: String) -> Result<T, EvidenceError> {
    Err(EvidenceError(msg))
}

/// Registry snapshot the evidence is checked against.
#[derive(Debug, Default)]
pub struct RegistryView {
    pub statuses: BTreeMap<String, String>,
    pub hashes: BTreeMap<String, String>,
}

impl RegistryView {
    pub fn from_rules(rules: Vec<(String, String, String)>) -> Self {
        let mut view = Self::default();
        for (id, status, hash) in rules {
            view.statuses.insert(id.clone(), status);
            view.hashes.insert(id, hash);
        }
        view
    }

    fn require_rule(&self, id: &str) -> Result<(), EvidenceError> {
        if self.statuses.contains_key(id) {
            Ok(())
        } else {
            fail(format!("unresolved rule ID: {id}"))
        }
    }

    fn require_live_rule(&self, id: &str) -> Result<(), EvidenceError> {
        self.require_rule(id)?;
        match self.statuses.get(id).map(String::as_str) {
            Some(status) if omni_registry::is_ownable_status(status) => Ok(()),
            Some(other) => fail(format!("rule {id} is {other}, not live")),
            None => fail(format!("unresolved rule ID: {id}")),
        }
    }
}

/// Caller-supplied resolution context: the registry, the active tree digest,
/// the toolchain identity when the caller can establish it (`None` skips
/// toolchain-equality checks but never the non-empty requirement), and the
/// set of known workspace-relative files.
#[derive(Debug)]
pub struct ValidationContext<'a> {
    pub registry: &'a RegistryView,
    pub tree_digest: &'a str,
    pub toolchain: Option<&'a str>,
    pub files: &'a BTreeSet<String>,
}

fn check_version(version: &str) -> Result<(), EvidenceError> {
    if version == "1.0.0" {
        Ok(())
    } else {
        fail(format!("unsupported schema version: {version}"))
    }
}

fn check_hex64(value: &str, what: &str) -> Result<(), EvidenceError> {
    if value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        Ok(())
    } else {
        fail(format!("malformed {what}: {value}"))
    }
}

fn check_rule_id(value: &str) -> Result<(), EvidenceError> {
    // Prefixes may contain digits (e.g. STAGE0); single-hyphen shape only
    // (VIBE-GRAM handled explicitly), mirroring the registry schema.
    let ok = if let Some(rest) = value.strip_prefix("VIBE-GRAM-") {
        rest.len() == 4 && rest.chars().all(|c| c.is_ascii_digit())
    } else {
        match value.split_once('-') {
            Some((head, tail)) => {
                !head.is_empty()
                    && head.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                    && tail.len() == 4
                    && tail.chars().all(|c| c.is_ascii_digit())
            }
            None => false,
        }
    };
    if ok {
        Ok(())
    } else {
        fail(format!("malformed rule ID: {value}"))
    }
}

fn check_ecode(value: &str) -> Result<(), EvidenceError> {
    if value.len() == 5 && value.starts_with('E') && value[1..].chars().all(|c| c.is_ascii_digit())
    {
        Ok(())
    } else {
        fail(format!("malformed diagnostic ID: {value}"))
    }
}

fn check_contained(path: &str) -> Result<(), EvidenceError> {
    if path.is_empty() {
        return fail("empty path reference".to_string());
    }
    if path.contains('\\') {
        return fail(format!("backslash path reference: {path}"));
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir
                    | std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
            )
        })
    {
        return fail(format!("non-contained path reference: {path}"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Records (mirror spec/schemas/*.schema.json; deny_unknown_fields enforced).
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub file: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fixit {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub schema_version: String,
    pub kind: String,
    pub id: String,
    pub severity: String,
    pub rule_id: String,
    #[serde(default)]
    pub related_rules: Vec<String>,
    #[serde(default)]
    pub domain: Option<String>,
    pub message: String,
    pub span: Span,
    #[serde(default)]
    pub related_spans: Vec<Span>,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub fixits: Vec<Fixit>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    pub schema_version: String,
    pub kind: String,
    pub witness_id: String,
    pub rule_id: String,
    #[serde(default)]
    pub rule_rev: Option<String>,
    pub test_ref: String,
    #[serde(default)]
    pub test_type: Option<String>,
    pub expected: String,
    pub observed: String,
    #[serde(default)]
    pub diagnostic_ids: Vec<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub spec_tree_sha256: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceOutcome {
    pub schema_version: String,
    pub kind: String,
    pub outcome_id: String,
    pub rule_id: String,
    #[serde(default)]
    pub witness_ids: Vec<String>,
    pub expected: String,
    pub actual: String,
    pub verdict: String,
    #[serde(default)]
    pub diagnostic_ids: Vec<String>,
    pub spec_tree_sha256: String,
    pub toolchain: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationFailure {
    pub schema_version: String,
    pub kind: String,
    pub failure_id: String,
    pub failure_class: String,
    #[serde(default)]
    pub rule_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    pub location: Span,
    pub explanation: String,
    pub impact: String,
    #[serde(default)]
    pub regression_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenConfig {
    pub opt_level: u8,
    pub emit_native: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub schema_version: String,
    pub kind: String,
    pub spec_tree_sha256: String,
    #[serde(default)]
    pub plan_sha256: Option<String>,
    pub toolchain: String,
    pub target: String,
    /// How the target string was produced: `host` (truthful build-environment
    /// description, never a selection) or `explicit` (declared semantic
    /// input). Host-derived values must never select target identity.
    #[serde(default)]
    pub target_source: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub compiler_id: Option<String>,
    #[serde(default)]
    pub compiler_version: Option<String>,
    #[serde(default)]
    pub source_revision: Option<String>,
    #[serde(default)]
    pub codegen: Option<CodegenConfig>,
    #[serde(default)]
    pub compiler_build_epoch: Option<u64>,
    pub artifact_sha256: String,
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Declared generation inputs. Freely constructible plain data: these are
/// claims awaiting validation, not a verified record.
#[derive(Debug, Clone)]
pub struct ProvenanceInputs {
    pub tree_digest: String,
    pub plan_digest: Option<String>,
    pub toolchain: String,
    pub target: String,
    pub target_source: Option<String>,
    pub profile: Option<String>,
    pub compiler_id: String,
    pub compiler_version: String,
    pub source_revision: Option<String>,
    pub codegen_opt_level: u8,
    pub codegen_emit_native: bool,
    pub compiler_build_epoch: u64,
    pub artifact_bytes: Vec<u8>,
}

/// Validated provenance: constructible only via [`validate_provenance_inputs`],
/// which derives every identity field from one input set, so mismatched
/// combinations (revision from A, digest from B) cannot be constructed.
#[derive(Debug, Clone)]
pub struct ValidatedProvenance {
    record: Provenance,
}

fn check_printable(value: &str, what: &str) -> Result<(), EvidenceError> {
    if value.is_empty() || value.chars().any(|c| c.is_control()) {
        return fail(format!("malformed {what}: {value:?}"));
    }
    Ok(())
}

fn check_revision(value: &str) -> Result<(), EvidenceError> {
    let core = value.strip_suffix("-dirty").unwrap_or(value);
    if (7..=64).contains(&core.len())
        && core.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        Ok(())
    } else {
        fail(format!("malformed source revision: {value}"))
    }
}

pub fn validate_provenance_inputs(
    inputs: ProvenanceInputs,
) -> Result<ValidatedProvenance, EvidenceError> {
    check_hex64(&inputs.tree_digest, "spec tree digest")?;
    if let Some(plan) = &inputs.plan_digest {
        check_hex64(plan, "plan digest")?;
    }
    check_printable(&inputs.toolchain, "toolchain")?;
    check_printable(&inputs.target, "target")?;
    if let Some(source) = &inputs.target_source {
        if source != "host" && source != "explicit" {
            return fail(format!("bad target source: {source}"));
        }
    }
    check_printable(&inputs.compiler_id, "compiler identity")?;
    check_printable(&inputs.compiler_version, "compiler version")?;
    if let Some(rev) = &inputs.source_revision {
        check_revision(rev)?;
    }
    Ok(ValidatedProvenance {
        record: Provenance {
            schema_version: "1.0.0".to_string(),
            kind: "provenance".to_string(),
            spec_tree_sha256: inputs.tree_digest,
            plan_sha256: inputs.plan_digest,
            toolchain: inputs.toolchain,
            target: inputs.target,
            target_source: inputs.target_source,
            profile: inputs.profile,
            compiler_id: Some(inputs.compiler_id),
            compiler_version: Some(inputs.compiler_version),
            source_revision: inputs.source_revision,
            codegen: Some(CodegenConfig {
                opt_level: inputs.codegen_opt_level,
                emit_native: inputs.codegen_emit_native,
            }),
            compiler_build_epoch: Some(inputs.compiler_build_epoch),
            artifact_sha256: omni_canon::sha256_of_normalized_bytes(&inputs.artifact_bytes),
            config: serde_json::Value::Object(Default::default()),
        },
    })
}

impl ValidatedProvenance {
    pub fn record(&self) -> &Provenance {
        &self.record
    }

    /// Canonical bytes via the shared engine (no duplicated canonicalization).
    pub fn canonical_bytes(&self, workspace_root: &Path) -> Result<Vec<u8>, CanonError> {
        canonical_record_bytes(&self.record, workspace_root)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Regression {
    pub schema_version: String,
    pub kind: String,
    pub regression_id: String,
    pub status: String,
    pub origin: String,
    pub input_ref: String,
    pub expected: String,
    #[serde(default)]
    pub rule_ids: Vec<String>,
    #[serde(default)]
    pub promotion_source: Option<String>,
    #[serde(default)]
    pub fuzz_promotion_id: Option<String>,
    pub spec_tree_sha256: String,
    pub toolchain: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FuzzPromotion {
    pub schema_version: String,
    pub kind: String,
    pub promotion_id: String,
    pub status: String,
    pub fuzz_target: String,
    pub seed_sha256: String,
    pub reproducer_ref: String,
    pub failure_class: String,
    #[serde(default)]
    pub sanitizer: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub regression_id: Option<String>,
    pub spec_tree_sha256: String,
    pub toolchain: String,
    #[serde(default)]
    pub target: Option<String>,
}

// ---------------------------------------------------------------------------
// Validators.
// ---------------------------------------------------------------------------

fn check_span(span: &Span, what: &str) -> Result<(), EvidenceError> {
    check_contained(&span.file)?;
    if span.start > span.end {
        return fail(format!("inverted span in {what}"));
    }
    Ok(())
}

const SEVERITIES: [&str; 4] = ["error", "warning", "note", "help"];

pub fn validate_diagnostic(d: &Diagnostic, ctx: &ValidationContext) -> Result<(), EvidenceError> {
    check_version(&d.schema_version)?;
    if d.kind != "diagnostic" {
        return fail(format!("bad kind: {}", d.kind));
    }
    check_ecode(&d.id)?;
    if !SEVERITIES.contains(&d.severity.as_str()) {
        return fail(format!("bad severity: {}", d.severity));
    }
    check_rule_id(&d.rule_id)?;
    ctx.registry.require_live_rule(&d.rule_id)?;
    for related in &d.related_rules {
        check_rule_id(related)?;
        ctx.registry.require_rule(related)?;
    }
    if d.message.is_empty() {
        return fail("empty diagnostic message".to_string());
    }
    check_span(&d.span, "diagnostic")?;
    for span in &d.related_spans {
        check_span(span, "related span")?;
    }
    for fixit in &d.fixits {
        check_span(&fixit.span, "fixit")?;
    }
    Ok(())
}

fn check_wit_id(value: &str, prefix: &str) -> Result<(), EvidenceError> {
    if value.len() == prefix.len() + 4
        && value.starts_with(prefix)
        && value[prefix.len()..].chars().all(|c| c.is_ascii_digit())
    {
        Ok(())
    } else {
        fail(format!("malformed ID: {value}"))
    }
}

pub fn validate_witness(w: &Witness, ctx: &ValidationContext) -> Result<(), EvidenceError> {
    check_version(&w.schema_version)?;
    if w.kind != "witness" {
        return fail(format!("bad kind: {}", w.kind));
    }
    check_wit_id(&w.witness_id, "WIT-")?;
    check_rule_id(&w.rule_id)?;
    ctx.registry.require_live_rule(&w.rule_id)?;
    if let Some(rev) = &w.rule_rev {
        check_hex64(rev, "rule revision")?;
        match ctx.registry.hashes.get(&w.rule_id) {
            Some(current) if current == rev => {}
            Some(_) => return fail(format!("stale rule revision for {}", w.rule_id)),
            None => return fail(format!("unresolved rule ID: {}", w.rule_id)),
        }
    }
    check_contained(&w.test_ref)?;
    if !ctx.files.contains(&w.test_ref) {
        return fail(format!("unresolved witness artifact: {}", w.test_ref));
    }
    for outcome in [&w.expected, &w.observed] {
        if outcome != "pass" && outcome != "failure" {
            return fail(format!("bad witness outcome: {outcome}"));
        }
    }
    if let Some(tree) = &w.spec_tree_sha256 {
        check_hex64(tree, "spec tree digest")?;
        if tree != ctx.tree_digest {
            return fail("witness tree digest mismatch".to_string());
        }
    }
    Ok(())
}

pub fn validate_conformance(
    c: &ConformanceOutcome,
    ctx: &ValidationContext,
) -> Result<(), EvidenceError> {
    check_version(&c.schema_version)?;
    if c.kind != "conformance-outcome" {
        return fail(format!("bad kind: {}", c.kind));
    }
    check_wit_id(&c.outcome_id, "CONF-")?;
    check_rule_id(&c.rule_id)?;
    ctx.registry.require_live_rule(&c.rule_id)?;
    if c.verdict != "pass" && c.verdict != "fail" {
        return fail(format!("bad verdict: {}", c.verdict));
    }
    if c.expected.is_empty() || c.actual.is_empty() {
        return fail("empty expected/actual result".to_string());
    }
    check_hex64(&c.spec_tree_sha256, "spec tree digest")?;
    if c.spec_tree_sha256 != ctx.tree_digest {
        return fail("conformance tree digest mismatch".to_string());
    }
    if c.toolchain.is_empty() {
        return fail("empty conformance toolchain".to_string());
    }
    if let Some(expected) = ctx.toolchain {
        if c.toolchain != expected {
            return fail(format!("conformance toolchain mismatch: {}", c.toolchain));
        }
    }
    for diag in &c.diagnostic_ids {
        check_ecode(diag)?;
    }
    for witness in &c.witness_ids {
        check_wit_id(witness, "WIT-")?;
    }
    Ok(())
}

const FAILURE_CLASSES: [&str; 7] = [
    "assertion",
    "invariant",
    "obligation",
    "counterexample",
    "timeout",
    "unsupported",
    "internal",
];

pub fn validate_verification_failure(
    v: &VerificationFailure,
    ctx: &ValidationContext,
) -> Result<(), EvidenceError> {
    check_version(&v.schema_version)?;
    if v.kind != "verification-failure" {
        return fail(format!("bad kind: {}", v.kind));
    }
    check_wit_id(&v.failure_id, "VF-")?;
    if !FAILURE_CLASSES.contains(&v.failure_class.as_str()) {
        return fail(format!("bad failure class: {}", v.failure_class));
    }
    match (&v.rule_id, &v.model) {
        (None, None) => return fail("failure needs a rule or model obligation".to_string()),
        (Some(rule), _) => {
            check_rule_id(rule)?;
            ctx.registry.require_rule(rule)?;
        }
        _ => {}
    }
    if let Some(model) = &v.model {
        if model.is_empty() {
            return fail("empty model reference".to_string());
        }
    }
    check_span(&v.location, "failure location")?;
    if v.explanation.is_empty() {
        return fail("empty failure explanation".to_string());
    }
    if v.impact != "release-blocking" && v.impact != "non-blocking" {
        return fail(format!("bad impact: {}", v.impact));
    }
    if let Some(reg) = &v.regression_id {
        check_wit_id(reg, "REG-")?;
    }
    Ok(())
}

pub fn validate_provenance(p: &Provenance, _ctx: &ValidationContext) -> Result<(), EvidenceError> {
    check_version(&p.schema_version)?;
    if p.kind != "provenance" {
        return fail(format!("bad kind: {}", p.kind));
    }
    // Historical records may bind older trees/toolchains, so identity fields
    // are format-checked, never equality-bound to the active context.
    check_hex64(&p.spec_tree_sha256, "spec tree digest")?;
    if let Some(plan) = &p.plan_sha256 {
        check_hex64(plan, "plan digest")?;
    }
    if p.toolchain.is_empty() || p.target.is_empty() {
        return fail("empty toolchain/target".to_string());
    }
    if let Some(source) = &p.target_source {
        if source != "host" && source != "explicit" {
            return fail(format!("bad target source: {source}"));
        }
    }
    if let Some(id) = &p.compiler_id {
        check_printable(id, "compiler identity")?;
    }
    if let Some(version) = &p.compiler_version {
        check_printable(version, "compiler version")?;
    }
    if let Some(rev) = &p.source_revision {
        check_revision(rev)?;
    }
    check_hex64(&p.artifact_sha256, "artifact digest")?;
    Ok(())
}

pub fn validate_regression(r: &Regression, ctx: &ValidationContext) -> Result<(), EvidenceError> {
    check_version(&r.schema_version)?;
    if r.kind != "regression" {
        return fail(format!("bad kind: {}", r.kind));
    }
    check_wit_id(&r.regression_id, "REG-")?;
    if !["open", "accepted", "closed"].contains(&r.status.as_str()) {
        return fail(format!("bad regression status: {}", r.status));
    }
    if r.origin.is_empty() {
        return fail("empty regression origin".to_string());
    }
    check_contained(&r.input_ref)?;
    if !ctx.files.contains(&r.input_ref) {
        return fail(format!("unresolved regression input: {}", r.input_ref));
    }
    if r.expected != "failure" && r.expected != "success" {
        return fail(format!("bad regression expectation: {}", r.expected));
    }
    for rule in &r.rule_ids {
        check_rule_id(rule)?;
        ctx.registry.require_rule(rule)?;
    }
    if let Some(fuzz) = &r.fuzz_promotion_id {
        check_wit_id(fuzz, "FUZZ-")?;
    }
    check_hex64(&r.spec_tree_sha256, "spec tree digest")?;
    if r.toolchain.is_empty() {
        return fail("empty regression toolchain".to_string());
    }
    Ok(())
}

pub fn validate_fuzz_promotion(
    f: &FuzzPromotion,
    ctx: &ValidationContext,
) -> Result<(), EvidenceError> {
    check_version(&f.schema_version)?;
    if f.kind != "fuzz-promotion" {
        return fail(format!("bad kind: {}", f.kind));
    }
    check_wit_id(&f.promotion_id, "FUZZ-")?;
    if !["candidate", "minimized", "promoted", "rejected"].contains(&f.status.as_str()) {
        return fail(format!("bad promotion status: {}", f.status));
    }
    if f.fuzz_target.is_empty() || f.failure_class.is_empty() {
        return fail("empty fuzz target/failure class".to_string());
    }
    check_hex64(&f.seed_sha256, "seed digest")?;
    check_contained(&f.reproducer_ref)?;
    if !ctx.files.contains(&f.reproducer_ref) {
        return fail(format!("unresolved reproducer: {}", f.reproducer_ref));
    }
    if let Some(reg) = &f.regression_id {
        check_wit_id(reg, "REG-")?;
    }
    check_hex64(&f.spec_tree_sha256, "spec tree digest")?;
    if f.toolchain.is_empty() {
        return fail("empty fuzz toolchain".to_string());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Bundle: cross-record linkage within one validated corpus.
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct EvidenceBundle {
    pub diagnostics: Vec<Diagnostic>,
    pub witnesses: Vec<Witness>,
    pub outcomes: Vec<ConformanceOutcome>,
    pub failures: Vec<VerificationFailure>,
    pub provenances: Vec<Provenance>,
    pub regressions: Vec<Regression>,
    pub promotions: Vec<FuzzPromotion>,
}

fn unique_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
    what: &str,
) -> Result<BTreeSet<String>, EvidenceError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id.to_string()) {
            return fail(format!("duplicate {what} ID: {id}"));
        }
    }
    Ok(seen)
}

pub fn validate_bundle(
    bundle: &EvidenceBundle,
    ctx: &ValidationContext,
) -> Result<(), EvidenceError> {
    for d in &bundle.diagnostics {
        validate_diagnostic(d, ctx)?;
    }
    for w in &bundle.witnesses {
        validate_witness(w, ctx)?;
    }
    for c in &bundle.outcomes {
        validate_conformance(c, ctx)?;
    }
    for v in &bundle.failures {
        validate_verification_failure(v, ctx)?;
    }
    for p in &bundle.provenances {
        validate_provenance(p, ctx)?;
    }
    for r in &bundle.regressions {
        validate_regression(r, ctx)?;
    }
    for f in &bundle.promotions {
        validate_fuzz_promotion(f, ctx)?;
    }

    let diag_ids = unique_ids(bundle.diagnostics.iter().map(|d| d.id.as_str()), "diagnostic")?;
    let wit_ids = unique_ids(bundle.witnesses.iter().map(|w| w.witness_id.as_str()), "witness")?;
    unique_ids(bundle.outcomes.iter().map(|c| c.outcome_id.as_str()), "outcome")?;
    unique_ids(bundle.failures.iter().map(|v| v.failure_id.as_str()), "failure")?;
    let reg_ids =
        unique_ids(bundle.regressions.iter().map(|r| r.regression_id.as_str()), "regression")?;
    let fuzz_ids =
        unique_ids(bundle.promotions.iter().map(|f| f.promotion_id.as_str()), "promotion")?;

    for w in &bundle.witnesses {
        for diag in &w.diagnostic_ids {
            if !diag_ids.contains(diag) {
                return fail(format!("witness {} links unknown diagnostic {diag}", w.witness_id));
            }
        }
    }
    for c in &bundle.outcomes {
        for witness in &c.witness_ids {
            if !wit_ids.contains(witness) {
                return fail(format!("outcome {} links unknown witness {witness}", c.outcome_id));
            }
        }
        for diag in &c.diagnostic_ids {
            if !diag_ids.contains(diag) {
                return fail(format!("outcome {} links unknown diagnostic {diag}", c.outcome_id));
            }
        }
    }
    for v in &bundle.failures {
        if let Some(reg) = &v.regression_id {
            if !reg_ids.contains(reg) {
                return fail(format!("failure {} links unknown regression {reg}", v.failure_id));
            }
        }
    }
    for r in &bundle.regressions {
        if let Some(fuzz) = &r.fuzz_promotion_id {
            if !fuzz_ids.contains(fuzz) {
                return fail(format!(
                    "regression {} links unknown promotion {fuzz}",
                    r.regression_id
                ));
            }
        }
    }
    for f in &bundle.promotions {
        if let Some(reg) = &f.regression_id {
            if !reg_ids.contains(reg) {
                return fail(format!(
                    "promotion {} links unknown regression {reg}",
                    f.promotion_id
                ));
            }
        }
    }
    Ok(())
}

/// Parse one evidence file: duplicate-key rejection, kind dispatch, strict shape.
/// Unknown kinds and future schema versions fail closed here, before typing.
pub fn parse_record(raw: &str) -> Result<EvidenceKind, EvidenceError> {
    omni_canon::reject_duplicate_keys(raw)
        .map_err(|e| EvidenceError(format!("ambiguous evidence keys: {e}")))?;
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| EvidenceError(format!("malformed evidence: {e}")))?;
    let version = value.get("schema_version").and_then(serde_json::Value::as_str).unwrap_or("");
    check_version(version)?;
    let kind = value.get("kind").and_then(serde_json::Value::as_str).unwrap_or("");
    match kind {
        "diagnostic" => Ok(EvidenceKind::Diagnostic(from_record(&value, "diagnostic")?)),
        "witness" => Ok(EvidenceKind::Witness(from_record(&value, "witness")?)),
        "conformance-outcome" => Ok(EvidenceKind::Outcome(from_record(&value, "outcome")?)),
        "verification-failure" => Ok(EvidenceKind::Failure(from_record(&value, "failure")?)),
        "provenance" => Ok(EvidenceKind::Provenance(from_record(&value, "provenance")?)),
        "regression" => Ok(EvidenceKind::Regression(from_record(&value, "regression")?)),
        "fuzz-promotion" => Ok(EvidenceKind::Promotion(from_record(&value, "promotion")?)),
        other => fail(format!("unknown evidence kind: {other}")),
    }
}

fn from_record<T>(value: &serde_json::Value, what: &str) -> Result<T, EvidenceError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(value.clone())
        .map_err(|e| EvidenceError(format!("malformed {what}: {e}")))
}

#[derive(Debug)]
pub enum EvidenceKind {
    Diagnostic(Diagnostic),
    Witness(Witness),
    Outcome(ConformanceOutcome),
    Failure(VerificationFailure),
    Provenance(Provenance),
    Regression(Regression),
    Promotion(FuzzPromotion),
}

/// Canonical bytes of any evidence record via the shared engine.
pub fn canonical_record_bytes<T: Serialize>(
    record: &T,
    workspace_root: &Path,
) -> Result<Vec<u8>, CanonError> {
    let mut value =
        serde_json::to_value(record).map_err(|e| CanonError::InvalidInput(e.to_string()))?;
    omni_canon::canonical_json_bytes(&mut value, workspace_root)
}

#[cfg(test)]
mod evidence_tests {
    use super::*;

    const LEX6_HASH: &str = "01c7cbb708f5d64c89b1f00105fa709bccc9baf67534d2fb4d825dabc5aa458a";
    const TREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn registry() -> RegistryView {
        RegistryView::from_rules(vec![
            ("LEX-0006".to_string(), "Candidate".to_string(), LEX6_HASH.to_string()),
            (
                "LEX-0002".to_string(),
                "Superseded".to_string(),
                "143add9642fe9d3d2d3ebf21f9cc539fcd683c18d2dc9fcb9a9f0c56ee7ca415".to_string(),
            ),
            ("RULE-0001".to_string(), "Candidate".to_string(), "b".repeat(64)),
            // Digit-bearing prefix (0.0.0.11 repair: STAGE0 was dropped by
            // an `[A-Z]+`-only ID class across extractor, schemas, validators).
            ("STAGE0-0007".to_string(), "Candidate".to_string(), "c".repeat(64)),
        ])
    }

    fn ctx<'a>(registry: &'a RegistryView, files: &'a BTreeSet<String>) -> ValidationContext<'a> {
        ValidationContext { registry, tree_digest: TREE, toolchain: Some("1.95.0"), files }
    }

    fn files() -> BTreeSet<String> {
        ["tests/corpus/a.omni", "wit/fix.rs", "repro/seed.bin"]
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    fn diag() -> Diagnostic {
        Diagnostic {
            schema_version: "1.0.0".to_string(),
            kind: "diagnostic".to_string(),
            id: "E0001".to_string(),
            severity: "error".to_string(),
            rule_id: "LEX-0006".to_string(),
            related_rules: vec![],
            domain: Some("OMNI-LEX".to_string()),
            message: "bad separator".to_string(),
            span: Span { file: "src/a.omni".to_string(), start: 4, end: 5 },
            related_spans: vec![],
            params: serde_json::Value::Null,
            fixits: vec![],
        }
    }

    fn wit() -> Witness {
        Witness {
            schema_version: "1.0.0".to_string(),
            kind: "witness".to_string(),
            witness_id: "WIT-0001".to_string(),
            rule_id: "LEX-0006".to_string(),
            rule_rev: Some(LEX6_HASH.to_string()),
            test_ref: "tests/corpus/a.omni".to_string(),
            test_type: Some("fixture".to_string()),
            expected: "failure".to_string(),
            observed: "failure".to_string(),
            diagnostic_ids: vec!["E0001".to_string()],
            target: None,
            profile: None,
            spec_tree_sha256: Some(TREE.to_string()),
        }
    }

    fn outcome() -> ConformanceOutcome {
        ConformanceOutcome {
            schema_version: "1.0.0".to_string(),
            kind: "conformance-outcome".to_string(),
            outcome_id: "CONF-0001".to_string(),
            rule_id: "LEX-0006".to_string(),
            witness_ids: vec!["WIT-0001".to_string()],
            expected: "reject".to_string(),
            actual: "reject E0001".to_string(),
            verdict: "pass".to_string(),
            diagnostic_ids: vec!["E0001".to_string()],
            spec_tree_sha256: TREE.to_string(),
            toolchain: "1.95.0".to_string(),
            target: None,
            profile: None,
        }
    }

    fn bundle() -> EvidenceBundle {
        EvidenceBundle {
            diagnostics: vec![diag()],
            witnesses: vec![wit()],
            outcomes: vec![outcome()],
            failures: vec![VerificationFailure {
                schema_version: "1.0.0".to_string(),
                kind: "verification-failure".to_string(),
                failure_id: "VF-0001".to_string(),
                failure_class: "obligation".to_string(),
                rule_id: Some("RULE-0001".to_string()),
                model: None,
                location: Span { file: "src/a.omni".to_string(), start: 0, end: 1 },
                explanation: "obligation unmet".to_string(),
                impact: "release-blocking".to_string(),
                regression_id: Some("REG-0001".to_string()),
            }],
            provenances: vec![Provenance {
                schema_version: "1.0.0".to_string(),
                kind: "provenance".to_string(),
                spec_tree_sha256: TREE.to_string(),
                plan_sha256: None,
                toolchain: "1.95.0".to_string(),
                target: "x86_64-unknown-linux-gnu".to_string(),
                target_source: Some("explicit".to_string()),
                profile: None,
                compiler_id: Some("omni-driver".to_string()),
                compiler_version: Some("0.0.0".to_string()),
                source_revision: Some("abc1234".to_string()),
                codegen: Some(CodegenConfig { opt_level: 0, emit_native: true }),
                compiler_build_epoch: Some(0),
                artifact_sha256: "c".repeat(64),
                config: serde_json::Value::Null,
            }],
            regressions: vec![Regression {
                schema_version: "1.0.0".to_string(),
                kind: "regression".to_string(),
                regression_id: "REG-0001".to_string(),
                status: "open".to_string(),
                origin: "fuzz crash".to_string(),
                input_ref: "repro/seed.bin".to_string(),
                expected: "failure".to_string(),
                rule_ids: vec!["LEX-0006".to_string()],
                promotion_source: Some("lexer_fuzz".to_string()),
                fuzz_promotion_id: Some("FUZZ-0001".to_string()),
                spec_tree_sha256: TREE.to_string(),
                toolchain: "1.95.0".to_string(),
                target: None,
                profile: None,
            }],
            promotions: vec![FuzzPromotion {
                schema_version: "1.0.0".to_string(),
                kind: "fuzz-promotion".to_string(),
                promotion_id: "FUZZ-0001".to_string(),
                status: "promoted".to_string(),
                fuzz_target: "lexer_fuzz".to_string(),
                seed_sha256: "d".repeat(64),
                reproducer_ref: "repro/seed.bin".to_string(),
                failure_class: "panic".to_string(),
                sanitizer: None,
                config: serde_json::Value::Null,
                regression_id: Some("REG-0001".to_string()),
                spec_tree_sha256: TREE.to_string(),
                toolchain: "1.95.0".to_string(),
                target: None,
            }],
        }
    }

    #[test]
    fn full_bundle_passes() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        validate_bundle(&bundle(), &c).expect("pass");
    }

    #[test]
    fn minimal_records_pass() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        validate_diagnostic(&diag(), &c).expect("diag");
        let mut w = wit();
        w.rule_rev = None;
        w.diagnostic_ids.clear();
        w.spec_tree_sha256 = None;
        validate_witness(&w, &c).expect("witness");
    }

    #[test]
    fn unknown_and_malformed_ids_fail() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        let mut d = diag();
        d.rule_id = "NOPE-9999".to_string();
        assert!(validate_diagnostic(&d, &c).is_err());
        d.rule_id = "bad id!".to_string();
        assert!(validate_diagnostic(&d, &c).is_err());
        d.rule_id = "LEX-0006".to_string();
        d.id = "X1".to_string();
        assert!(validate_diagnostic(&d, &c).is_err());
        d.id = "E0001".to_string();
        d.severity = "fatal".to_string();
        assert!(validate_diagnostic(&d, &c).is_err());
    }

    #[test]
    fn superseded_witness_and_stale_rev_fail() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        let mut w = wit();
        w.rule_id = "LEX-0002".to_string();
        w.rule_rev = None;
        assert!(validate_witness(&w, &c).expect_err("stale rule").contains("not live"));
        w.rule_id = "LEX-0006".to_string();
        w.rule_rev = Some("0".repeat(64));
        assert!(validate_witness(&w, &c).expect_err("stale rev").contains("stale"));
    }

    #[test]
    fn witness_artifact_must_resolve() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        for bad in ["missing.omni", "../escape.omni", "/abs.omni", "a\\b.omni"] {
            let mut w = wit();
            w.test_ref = bad.to_string();
            assert!(validate_witness(&w, &c).is_err(), "ref: {bad}");
        }
    }

    #[test]
    fn conformance_binding_enforced() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        let mut o = outcome();
        o.spec_tree_sha256 = "f".repeat(64);
        assert!(validate_conformance(&o, &c).expect_err("tree").contains("tree"));
        o.spec_tree_sha256 = TREE.to_string();
        o.toolchain = "9.9.9".to_string();
        assert!(validate_conformance(&o, &c).expect_err("toolchain").contains("toolchain"));
    }

    #[test]
    fn cross_links_resolve_in_bundle() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        let mut b = bundle();
        b.outcomes[0].witness_ids = vec!["WIT-9999".to_string()];
        assert!(validate_bundle(&b, &c).expect_err("witness link").contains("unknown witness"));
        b.outcomes[0].witness_ids = vec!["WIT-0001".to_string()];
        b.promotions[0].regression_id = Some("REG-9999".to_string());
        assert!(validate_bundle(&b, &c).expect_err("reg link").contains("unknown regression"));
        b.promotions[0].regression_id = Some("REG-0001".to_string());
        b.regressions[0].fuzz_promotion_id = Some("FUZZ-9999".to_string());
        assert!(validate_bundle(&b, &c).expect_err("fuzz link").contains("unknown promotion"));
    }

    #[test]
    fn duplicate_ids_rejected() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        let mut b = bundle();
        b.diagnostics.push(diag());
        assert!(validate_bundle(&b, &c).expect_err("dup").contains("duplicate"));
    }

    #[test]
    fn future_versions_rejected() {
        for raw in [
            r#"{"schema_version":"2.0.0","kind":"diagnostic"}"#,
            r#"{"kind":"diagnostic"}"#,
            r#"{"schema_version":"1.0.0","kind":"frobnicate"}"#,
            r#"{"schema_version":"1.0.0","kind":"diagnostic","id":"E0001","id":"E0002"}"#,
            r#"not json"#,
        ] {
            assert!(parse_record(raw).is_err(), "raw: {raw}");
        }
    }

    #[test]
    fn strict_shapes_rejected() {
        // Unknown field with otherwise valid diagnostic.
        let raw = r#"{"schema_version":"1.0.0","kind":"diagnostic","id":"E0001","severity":"error","rule_id":"LEX-0006","message":"m","span":{"file":"a","start":0,"end":1},"surprise":1}"#;
        assert!(parse_record(raw).is_err());
        // Missing required field.
        let raw = r#"{"schema_version":"1.0.0","kind":"diagnostic","id":"E0001","severity":"error","rule_id":"LEX-0006","span":{"file":"a","start":0,"end":1}}"#;
        assert!(parse_record(raw).is_err());
    }

    #[test]
    fn canonical_bytes_deterministic() {
        let root = Path::new("/ws");
        let a = canonical_record_bytes(&diag(), root).expect("bytes");
        let mut raw: serde_json::Value = serde_json::to_value(diag()).expect("value");
        if let serde_json::Value::Object(map) = &mut raw {
            let keys: Vec<String> = map.keys().cloned().collect();
            let mut rev = serde_json::Map::new();
            for k in keys.into_iter().rev() {
                rev.insert(k.clone(), map.remove(&k).expect("key"));
            }
            *map = rev;
        }
        let b = {
            let mut v = raw;
            omni_canon::canonical_json_bytes(&mut v, root).expect("bytes")
        };
        assert_eq!(a, b);
        assert_eq!(
            a,
            canonical_record_bytes(&diag(), root).expect("repeat"),
            "repeated runs identical"
        );
        let mut other = diag();
        other.message = "different".to_string();
        assert_ne!(a, canonical_record_bytes(&other, root).expect("bytes"));
    }

    #[test]
    fn verification_failure_needs_obligation() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        let mut v = VerificationFailure {
            schema_version: "1.0.0".to_string(),
            kind: "verification-failure".to_string(),
            failure_id: "VF-0001".to_string(),
            failure_class: "counterexample".to_string(),
            rule_id: None,
            model: None,
            location: Span { file: "m/mir".to_string(), start: 0, end: 0 },
            explanation: "x".to_string(),
            impact: "non-blocking".to_string(),
            regression_id: None,
        };
        assert!(validate_verification_failure(&v, &c).is_err());
        v.model = Some("ownership-safety".to_string());
        validate_verification_failure(&v, &c).expect("model suffices");
        v.model = None;
        v.rule_id = Some("GHOST-0001".to_string());
        assert!(validate_verification_failure(&v, &c).is_err());
    }

    #[test]
    fn digit_prefix_rule_ids_accepted() {
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        let mut d = diag();
        d.rule_id = "STAGE0-0007".to_string();
        validate_diagnostic(&d, &c).expect("STAGE0 diagnostic");
        let mut w = wit();
        w.rule_id = "STAGE0-0007".to_string();
        w.rule_rev = Some("c".repeat(64));
        validate_witness(&w, &c).expect("STAGE0 witness");
    }

    fn provenance_inputs() -> ProvenanceInputs {
        ProvenanceInputs {
            tree_digest: TREE.to_string(),
            plan_digest: None,
            toolchain: "1.95.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            target_source: Some("explicit".to_string()),
            profile: None,
            compiler_id: "omni-driver".to_string(),
            compiler_version: "0.0.0".to_string(),
            source_revision: Some("abc1234def5678".to_string()),
            codegen_opt_level: 2,
            codegen_emit_native: true,
            compiler_build_epoch: 0,
            artifact_bytes: b"object-bytes".to_vec(),
        }
    }

    #[test]
    fn provenance_generation_round_trip() {
        let validated = validate_provenance_inputs(provenance_inputs()).expect("valid");
        let record = validated.record();
        assert_eq!(record.kind, "provenance");
        assert_eq!(record.codegen.as_ref().expect("codegen").opt_level, 2);
        let reg = registry();
        let files = files();
        let c = ctx(&reg, &files);
        // Generated records validate against the schema contract.
        let raw = serde_json::to_string(record).expect("serialize");
        let parsed: Provenance = serde_json::from_str(&raw).expect("parse");
        validate_provenance(&parsed, &c).expect("valid");
        // Canonical identity is deterministic.
        let root = Path::new("/ws");
        let a = validated.canonical_bytes(root).expect("bytes");
        let b = validated.canonical_bytes(root).expect("bytes");
        assert_eq!(a, b);
    }

    #[test]
    fn provenance_inputs_fail_closed() {
        // Each malformed class rejects: empty toolchain/target/ids, bad
        // revision, bad tree digest.
        let mut bad = provenance_inputs();
        bad.toolchain.clear();
        assert!(validate_provenance_inputs(bad).is_err());
        let mut bad = provenance_inputs();
        bad.target.clear();
        assert!(validate_provenance_inputs(bad).is_err());
        let mut bad = provenance_inputs();
        bad.compiler_id.clear();
        assert!(validate_provenance_inputs(bad).is_err());
        let mut bad = provenance_inputs();
        bad.compiler_version = "bad\nversion".to_string();
        assert!(validate_provenance_inputs(bad).is_err());
        let mut bad = provenance_inputs();
        bad.source_revision = Some("not-a-sha!!".to_string());
        assert!(validate_provenance_inputs(bad).is_err());
        let mut bad = provenance_inputs();
        bad.source_revision = Some("ZZZ".to_string());
        assert!(validate_provenance_inputs(bad).is_err());
        let mut bad = provenance_inputs();
        bad.tree_digest = "short".to_string();
        assert!(validate_provenance_inputs(bad).is_err());
        // Dirty-suffixed full SHAs are the documented convention.
        let mut dirty = provenance_inputs();
        dirty.source_revision = Some(format!("{}-dirty", "a".repeat(40)));
        validate_provenance_inputs(dirty).expect("dirty convention");
        // Host paths never enter identity: none of the inputs accept them,
        // and canonical bytes contain no absolute host string.
        let validated = validate_provenance_inputs(provenance_inputs()).expect("valid");
        let bytes = validated.canonical_bytes(Path::new("/ws")).expect("bytes");
        let text = String::from_utf8(bytes).expect("utf8");
        assert!(!text.contains("/ws"), "host root leaked: {text}");
        assert!(!text.contains("C:\\"), "host path leaked");
    }

    #[test]
    fn provenance_identity_tracks_each_input() {
        let base = validated_bytes();
        let mut changed = provenance_inputs();
        changed.source_revision = Some("bbbbbbbbbbbbbbbb".to_string());
        assert_ne!(base, validated_bytes_with(changed), "revision");
        let mut changed = provenance_inputs();
        changed.tree_digest = "e".repeat(64);
        assert_ne!(base, validated_bytes_with(changed), "tree");
        let mut changed = provenance_inputs();
        changed.toolchain = "1.94.0".to_string();
        assert_ne!(base, validated_bytes_with(changed), "toolchain");
        let mut changed = provenance_inputs();
        changed.target = "aarch64-unknown-linux-gnu".to_string();
        assert_ne!(base, validated_bytes_with(changed), "target");
        let mut changed = provenance_inputs();
        changed.codegen_opt_level = 3;
        assert_ne!(base, validated_bytes_with(changed), "codegen");
        let mut changed = provenance_inputs();
        changed.artifact_bytes = b"other-bytes".to_vec();
        assert_ne!(base, validated_bytes_with(changed), "artifact");
        let mut changed = provenance_inputs();
        changed.compiler_build_epoch = 1_700_000_000;
        assert_ne!(base, validated_bytes_with(changed), "epoch");
    }

    fn validated_bytes() -> Vec<u8> {
        validated_bytes_with(provenance_inputs())
    }

    fn validated_bytes_with(inputs: ProvenanceInputs) -> Vec<u8> {
        validate_provenance_inputs(inputs)
            .expect("valid")
            .canonical_bytes(Path::new("/ws"))
            .expect("bytes")
    }

    #[test]
    fn unknown_toolchain_context_skips_equality() {
        let reg = registry();
        let files = files();
        let loose =
            ValidationContext { registry: &reg, tree_digest: TREE, toolchain: None, files: &files };
        let mut o = outcome();
        o.toolchain = "9.9.9".to_string();
        validate_conformance(&o, &loose).expect("equality skipped");
        o.toolchain = String::new();
        assert!(validate_conformance(&o, &loose).is_err());
    }
}
