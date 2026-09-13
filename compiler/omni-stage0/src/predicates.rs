use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Stage0PredicateEngine {
    allowed: BTreeSet<String>,
    forbidden: BTreeSet<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PredicateError {
    FeatureForbidden(String),
    FeatureNotExplicitlyAllowed(String),
}

impl Stage0PredicateEngine {
    pub fn from_manifest(manifest_json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let val: serde_json::Value = serde_json::from_str(manifest_json)?;
        let allowed = val["stage0_feature_predicates"]["allowed"]
            .as_array()
            .ok_or("missing allowed predicates")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        let forbidden = val["stage0_feature_predicates"]["forbidden"]
            .as_array()
            .ok_or("missing forbidden predicates")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        Ok(Self { allowed, forbidden })
    }

    pub fn check(&self, feature: &str) -> Result<(), PredicateError> {
        if self.forbidden.contains(feature) {
            return Err(PredicateError::FeatureForbidden(feature.to_owned()));
        }
        if !self.allowed.contains(feature) {
            return Err(PredicateError::FeatureNotExplicitlyAllowed(feature.to_owned()));
        }
        Ok(())
    }
}
