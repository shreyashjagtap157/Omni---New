//! Stage-0 feature predicates (0.0.0.11).
//!
//! Profile/predicate layer over the Edition-1 authority, not a second
//! language: the single known profile (`stage0`) answers whether a feature is
//! enabled from the manifest's allowed/forbidden sets. Unknown features fail
//! as unknown (never silently disabled); forbidden features fail as
//! forbidden. Construction takes already-validated lists only; raw manifest
//! JSON is never parsed here (loading belongs to `omni-registry`).
//!
//! Restriction records (STAGE0-0007) map a Stage-0 restriction to its
//! Edition-1 authority without reinterpreting either side. No restriction
//! mappings are curated yet; the mechanism, not fabricated mappings, is what
//! this milestone qualifies.

use std::collections::BTreeSet;

/// Profile identity. The manifest defines exactly one profile today.
pub const STAGE0_PROFILE: &str = "stage0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateError {
    FeatureForbidden(String),
    FeatureNotExplicitlyAllowed(String),
    UnknownProfile(String),
    DuplicateFeature(String),
    ConflictingAssignment(String),
    EmptyAllowedSet,
    UnsupportedPredicateVersion(String),
    DanglingAuthority(String),
}

impl std::fmt::Display for PredicateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FeatureForbidden(feature) => write!(f, "feature forbidden: {feature}"),
            Self::FeatureNotExplicitlyAllowed(feature) => {
                write!(f, "feature unknown (not explicitly allowed): {feature}")
            }
            Self::UnknownProfile(profile) => write!(f, "unknown profile: {profile}"),
            Self::DuplicateFeature(feature) => write!(f, "duplicate feature: {feature}"),
            Self::ConflictingAssignment(feature) => {
                write!(f, "feature both allowed and forbidden: {feature}")
            }
            Self::EmptyAllowedSet => write!(f, "allowed feature set is empty"),
            Self::UnsupportedPredicateVersion(version) => {
                write!(f, "unsupported predicate version: {version}")
            }
            Self::DanglingAuthority(authority) => {
                write!(f, "restriction authority unresolvable: {authority}")
            }
        }
    }
}

impl std::error::Error for PredicateError {}

#[derive(Debug, Clone)]
pub struct Stage0PredicateEngine {
    allowed: BTreeSet<String>,
    forbidden: BTreeSet<String>,
}

impl Stage0PredicateEngine {
    /// Build from already-validated lists. Duplicates (within either list),
    /// allowed/forbidden overlap, and empty allowed sets fail closed.
    /// `version` is the manifest predicate version; only `1.0.0` loads.
    pub fn from_lists(
        allowed: &[String],
        forbidden: &[String],
        version: &str,
    ) -> Result<Self, PredicateError> {
        if version != "1.0.0" {
            return Err(PredicateError::UnsupportedPredicateVersion(version.to_owned()));
        }
        // Duplicates are detected per list first so that an item present in
        // both lists reports as a conflict, not a duplicate.
        for (list, items) in [("allowed", allowed), ("forbidden", forbidden)] {
            let mut seen = BTreeSet::new();
            for feature in items {
                if !seen.insert(feature.clone()) {
                    return Err(PredicateError::DuplicateFeature(format!("{list}:{feature}")));
                }
            }
        }
        let allowed: BTreeSet<String> = allowed.iter().cloned().collect();
        let forbidden: BTreeSet<String> = forbidden.iter().cloned().collect();
        if allowed.is_empty() {
            return Err(PredicateError::EmptyAllowedSet);
        }
        if let Some(conflict) = allowed.intersection(&forbidden).next() {
            return Err(PredicateError::ConflictingAssignment(conflict.clone()));
        }
        Ok(Self { allowed, forbidden })
    }

    /// Select the single known profile. Anything else fails closed.
    pub fn select_profile(&self, profile: &str) -> Result<Stage0Profile<'_>, PredicateError> {
        if profile == STAGE0_PROFILE {
            Ok(Stage0Profile { engine: self })
        } else {
            Err(PredicateError::UnknownProfile(profile.to_owned()))
        }
    }

    /// Direct query: forbidden beats unknown; unknown is never disabled.
    pub fn check(&self, feature: &str) -> Result<(), PredicateError> {
        if self.forbidden.contains(feature) {
            return Err(PredicateError::FeatureForbidden(feature.to_owned()));
        }
        if !self.allowed.contains(feature) {
            return Err(PredicateError::FeatureNotExplicitlyAllowed(feature.to_owned()));
        }
        Ok(())
    }

    pub fn allowed(&self) -> &BTreeSet<String> {
        &self.allowed
    }

    pub fn forbidden(&self) -> &BTreeSet<String> {
        &self.forbidden
    }
}

/// Borrowed view bound to the selected profile; queries are deterministic
/// (set membership only: no ordering, environment, or target dependence).
#[derive(Debug, Clone, Copy)]
pub struct Stage0Profile<'a> {
    engine: &'a Stage0PredicateEngine,
}

impl Stage0Profile<'_> {
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.engine.check(feature).is_ok()
    }

    pub fn check(&self, feature: &str) -> Result<(), PredicateError> {
        self.engine.check(feature)
    }
}

/// A Stage-0 restriction mapped to its Edition-1 authority (rule ID or
/// grammar production name). The mapping is validated for resolvability;
/// semantic interpretation belongs to future parser/model consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Restriction {
    pub rule_id: String,
    pub authority: Authority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Authority {
    Rule(String),
    Production(String),
}

/// Validate restriction authorities against a rule-ID set and a grammar
/// production-name set supplied by the caller. No curation happens here.
pub fn validate_restrictions(
    restrictions: &[Restriction],
    known_rules: &BTreeSet<String>,
    known_productions: &BTreeSet<String>,
) -> Result<(), PredicateError> {
    for restriction in restrictions {
        match &restriction.authority {
            Authority::Rule(id) if !known_rules.contains(id) => {
                return Err(PredicateError::DanglingAuthority(id.clone()));
            }
            Authority::Production(name) if !known_productions.contains(name) => {
                return Err(PredicateError::DanglingAuthority(name.clone()));
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod predicate_tests {
    use super::*;

    fn engine() -> Stage0PredicateEngine {
        Stage0PredicateEngine::from_lists(
            &["functions".to_string(), "bool".to_string()],
            &["macros".to_string(), "async".to_string()],
            "1.0.0",
        )
        .expect("fixture")
    }

    #[test]
    fn known_predicates_are_deterministic() {
        let engine = engine();
        let profile = engine.select_profile("stage0").expect("profile");
        assert!(profile.is_enabled("functions"));
        assert!(profile.is_enabled("bool"));
        assert!(!profile.is_enabled("macros"));
        assert!(!profile.is_enabled("async"));
        // Repeated queries agree.
        assert_eq!(profile.is_enabled("functions"), profile.is_enabled("functions"));
    }

    #[test]
    fn unknown_is_not_disabled() {
        let engine = engine();
        let err = engine.check("frobnicate").expect_err("unknown");
        assert_eq!(err, PredicateError::FeatureNotExplicitlyAllowed("frobnicate".to_string()));
        assert_ne!(
            err,
            PredicateError::FeatureForbidden("frobnicate".to_string()),
            "unknown must differ from forbidden"
        );
    }

    #[test]
    fn forbidden_beats_allowed_lookup() {
        let engine = engine();
        assert_eq!(
            engine.check("macros"),
            Err(PredicateError::FeatureForbidden("macros".to_string()))
        );
    }

    #[test]
    fn profiles_and_versions_fail_closed() {
        let engine = engine();
        assert_eq!(
            engine.select_profile("stage1").expect_err("profile"),
            PredicateError::UnknownProfile("stage1".to_string())
        );
        assert_eq!(
            engine.select_profile("").expect_err("empty"),
            PredicateError::UnknownProfile(String::new())
        );
        let err = Stage0PredicateEngine::from_lists(&["a".to_string()], &[], "2.0.0")
            .expect_err("version");
        assert_eq!(err, PredicateError::UnsupportedPredicateVersion("2.0.0".to_string()));
    }

    #[test]
    fn duplicates_conflicts_and_degenerate_sets_fail() {
        let err =
            Stage0PredicateEngine::from_lists(&["a".to_string(), "a".to_string()], &[], "1.0.0")
                .expect_err("dup");
        assert_eq!(err, PredicateError::DuplicateFeature("allowed:a".to_string()));
        let err =
            Stage0PredicateEngine::from_lists(&["a".to_string()], &["a".to_string()], "1.0.0")
                .expect_err("conflict");
        assert_eq!(err, PredicateError::ConflictingAssignment("a".to_string()));
        let err =
            Stage0PredicateEngine::from_lists(&[], &["a".to_string()], "1.0.0").expect_err("empty");
        assert_eq!(err, PredicateError::EmptyAllowedSet);
    }

    #[test]
    fn restrictions_resolve_or_fail() {
        let rules: BTreeSet<String> = ["STAGE0-0007".to_string()].into_iter().collect();
        let productions: BTreeSet<String> = ["function_def".to_string()].into_iter().collect();
        validate_restrictions(
            &[Restriction {
                rule_id: "STAGE0-0007".to_string(),
                authority: Authority::Rule("STAGE0-0007".to_string()),
            }],
            &rules,
            &productions,
        )
        .expect("rule authority");
        validate_restrictions(
            &[Restriction {
                rule_id: "STAGE0-0004".to_string(),
                authority: Authority::Production("function_def".to_string()),
            }],
            &rules,
            &productions,
        )
        .expect("production authority");
        let err = validate_restrictions(
            &[Restriction {
                rule_id: "STAGE0-0007".to_string(),
                authority: Authority::Rule("GHOST-0001".to_string()),
            }],
            &rules,
            &productions,
        )
        .expect_err("dangling rule");
        assert_eq!(err, PredicateError::DanglingAuthority("GHOST-0001".to_string()));
        let err = validate_restrictions(
            &[Restriction {
                rule_id: "STAGE0-0004".to_string(),
                authority: Authority::Production("no_such_production".to_string()),
            }],
            &rules,
            &productions,
        )
        .expect_err("dangling production");
        assert_eq!(err, PredicateError::DanglingAuthority("no_such_production".to_string()));
    }

    /// Integration across the trust chain: the real validated manifest feeds
    /// the predicate engine. Canon -> registry loader -> predicate queries.
    #[test]
    fn live_manifest_drives_predicates() {
        use std::path::PathBuf;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec");
        let loaded = omni_registry::load_specification(root).expect("live load");
        let manifest = loaded.manifest();
        let (allowed, forbidden) = manifest.stage0_feature_sets().expect("sets");
        let engine =
            Stage0PredicateEngine::from_lists(&allowed, &forbidden, &manifest.manifest_version)
                .expect("live engine");
        let profile = engine.select_profile("stage0").expect("profile");
        // Spot-checks from the normative manifest lists.
        assert!(profile.is_enabled("functions"));
        assert!(profile.is_enabled("affine_ownership"));
        assert_eq!(
            profile.check("macros"),
            Err(PredicateError::FeatureForbidden("macros".to_string()))
        );
        assert_eq!(
            profile.check("async"),
            Err(PredicateError::FeatureForbidden("async".to_string()))
        );
        assert!(profile.check("frobnicate").is_err());
        // Restriction obligations: STAGE0 rules resolve in the live registry.
        let known_rules: BTreeSet<String> =
            loaded.registry().rules.iter().map(|r| r.rule_id.clone()).collect();
        assert!(known_rules.contains("STAGE0-0007"));
        validate_restrictions(&[], &known_rules, &BTreeSet::new()).expect("empty set valid");
    }
}
