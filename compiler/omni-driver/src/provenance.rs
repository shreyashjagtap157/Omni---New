//! Artifact provenance emission (0.0.0.12).
//!
//! The record FORMAT belongs to the 0.0.0.8 evidence schema and record
//! construction belongs to `omni-evidence`; this module only gathers actual
//! driver build context and writes the canonical sidecar. No identity value
//! is invented: unresolvable inputs fail the emission instead.
//!
//! Known limitation (documented, not hidden): the specification tree is
//! discovered as `./spec` relative to the invocation directory, and the
//! toolchain pin as `./rust-toolchain.toml`. Running the driver elsewhere
//! fails closed on the native-emission path; other paths are unaffected.

use std::path::{Path, PathBuf};

/// Build context assembled from actual compilation inputs.
#[derive(Debug, Clone)]
pub struct EmissionContext {
    pub tree_digest: String,
    pub plan_digest: Option<String>,
    pub toolchain: String,
    pub target_descriptor: String,
    pub compiler_version: String,
    pub source_revision: Option<String>,
    pub opt_level: u8,
    pub emit_native: bool,
    pub build_epoch: u64,
}

/// Locate `./spec` from the invocation directory.
pub fn discover_spec_root() -> Result<PathBuf, String> {
    let candidate = PathBuf::from("spec");
    if candidate.is_dir() {
        Ok(candidate)
    } else {
        Err("specification tree not resolvable: expected ./spec".to_string())
    }
}

/// Read the pinned channel from `./rust-toolchain.toml`.
pub fn read_toolchain_channel() -> Result<String, String> {
    let raw = std::fs::read_to_string("rust-toolchain.toml")
        .map_err(|e| format!("toolchain pin unreadable: {e}"))?;
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("channel") {
            let rest = rest.trim().strip_prefix('=').unwrap_or("").trim();
            let channel = rest.trim_matches('"');
            if !channel.is_empty() {
                return Ok(channel.to_string());
            }
        }
    }
    Err("toolchain channel not declared".to_string())
}

/// Host descriptor (`arch-os`). This is a truthful host description, not a
/// canonical TARGET-0001 triple (target registry is future work).
pub fn target_descriptor() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

/// Declared build epoch (`SOURCE_DATE_EPOCH`, else zero). Wall-clock time is
/// never consulted.
pub fn build_epoch() -> u64 {
    std::env::var("SOURCE_DATE_EPOCH").ok().and_then(|value| value.parse().ok()).unwrap_or(0)
}

/// Git source revision with dirty-state marking. Returns `None` when git is
/// unavailable rather than fabricating a revision; detached HEAD still yields
/// its SHA because that is the truthful revision.
pub fn source_revision(workdir: &Path) -> Option<String> {
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workdir)
        .output()
        .ok()?;
    if !head.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&head.stdout).trim().to_string();
    if sha.is_empty() {
        return None;
    }
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(workdir)
        .output()
        .ok()
        .is_some_and(|out| out.status.success() && !out.stdout.is_empty());
    Some(if dirty { format!("{sha}-dirty") } else { sha })
}

/// Build, validate, and write the canonical provenance sidecar next to the
/// emitted artifact. Returns the sidecar path.
pub fn emit_provenance_sidecar(
    artifact_path: &Path,
    artifact_bytes: &[u8],
    ctx: &EmissionContext,
    workspace_root: &Path,
) -> Result<PathBuf, String> {
    let inputs = omni_evidence::ProvenanceInputs {
        tree_digest: ctx.tree_digest.clone(),
        plan_digest: ctx.plan_digest.clone(),
        toolchain: ctx.toolchain.clone(),
        target: ctx.target_descriptor.clone(),
        profile: None,
        compiler_id: "omni-driver".to_string(),
        compiler_version: ctx.compiler_version.clone(),
        source_revision: ctx.source_revision.clone(),
        codegen_opt_level: ctx.opt_level,
        codegen_emit_native: ctx.emit_native,
        compiler_build_epoch: ctx.build_epoch,
        artifact_bytes: artifact_bytes.to_vec(),
    };
    let validated = omni_evidence::validate_provenance_inputs(inputs)
        .map_err(|e| format!("provenance invalid: {e}"))?;
    let bytes = validated
        .canonical_bytes(workspace_root)
        .map_err(|e| format!("provenance canonicalization failed: {e}"))?;
    let sidecar = artifact_path.with_extension("provenance.json");
    std::fs::write(&sidecar, &bytes).map_err(|e| format!("sidecar write failed: {e}"))?;
    Ok(sidecar)
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    fn context() -> EmissionContext {
        EmissionContext {
            tree_digest: "a".repeat(64),
            plan_digest: None,
            toolchain: "1.95.0".to_string(),
            target_descriptor: "x86_64-windows".to_string(),
            compiler_version: "0.0.0".to_string(),
            source_revision: Some("abc1234def5678".to_string()),
            opt_level: 0,
            emit_native: true,
            build_epoch: 0,
        }
    }

    #[test]
    fn sidecar_round_trip_validates() {
        let dir = std::env::temp_dir().join(format!("omni-prov-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let artifact = dir.join("output.o");
        let bytes = b"object-bytes";
        let sidecar = emit_provenance_sidecar(&artifact, bytes, &context(), &dir).expect("emit");
        assert_eq!(sidecar.extension().and_then(|e| e.to_str()), Some("json"));
        let raw = std::fs::read_to_string(&sidecar).expect("read");
        let record: omni_evidence::Provenance = serde_json::from_str(&raw).expect("parse");
        assert_eq!(record.compiler_id.as_deref(), Some("omni-driver"));
        assert_eq!(record.codegen.as_ref().expect("codegen").opt_level, 0);
        // Artifact binding is exact: recompute independently via the engine.
        assert_eq!(record.artifact_sha256, omni_canon::sha256_of_normalized_bytes(bytes));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unresolvable_inputs_fail_closed() {
        // Empty toolchain cannot bind an identity.
        let mut ctx = context();
        ctx.toolchain.clear();
        let dir = std::env::temp_dir();
        assert!(emit_provenance_sidecar(&dir.join("x.o"), b"bytes", &ctx, &dir).is_err());
    }

    #[test]
    fn host_descriptor_is_truthful() {
        let descriptor = target_descriptor();
        assert!(descriptor.contains(std::env::consts::ARCH));
        assert!(!descriptor.chars().any(|c| c.is_control()));
    }

    #[test]
    fn revision_shape_tolerated_or_absent() {
        // Helper must never fabricate: Some(valid-sha, documented -dirty
        // suffix allowed) or None only.
        if let Some(rev) = source_revision(Path::new(".")) {
            let core = rev.strip_suffix("-dirty").unwrap_or(&rev);
            assert!(
                (7..=64).contains(&core.len()) && core.chars().all(|c| c.is_ascii_hexdigit()),
                "unexpected revision shape: {rev}"
            );
        }
    }
}
