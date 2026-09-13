use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArtifactProvenance {
    pub spec_tree_sha256: String,
    pub plan_sha256: String,
    pub compiler_build_epoch: u64,
    pub target_triple: String,
    pub artifact_sha256: String,
}

impl ArtifactProvenance {
    #[allow(dead_code)]
    pub fn compute(spec_sha: &str, plan_sha: &str, target: &str, artifact_bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(artifact_bytes);
        let artifact_sha = hex::encode(hasher.finalize());
        Self {
            spec_tree_sha256: spec_sha.to_owned(),
            plan_sha256: plan_sha.to_owned(),
            compiler_build_epoch: std::env::var("SOURCE_DATE_EPOCH")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
            target_triple: target.to_owned(),
            artifact_sha256: artifact_sha,
        }
    }
}
