use std::collections::BTreeSet;

pub mod predicates;

use predicates::{Stage0PredicateEngine, PredicateError, Restriction, Authority};

fn validate_restrictions(
    restrictions: &[Restriction],
    rules: &BTreeSet<String>,
    productions: &BTreeSet<String>,
) -> Result<(), PredicateError> {
    for restriction in restrictions {
        match &restriction.authority {
            Authority::Rule(rule_id) => {
                if !rules.contains(rule_id) {
                    return Err(PredicateError::DanglingAuthority(rule_id.clone()));
                }
            }
            Authority::Production(production_id) => {
                if !productions.contains(production_id) {
                    return Err(PredicateError::DanglingAuthority(production_id.clone()));
                }
            }
        }
    }
    Ok(())
}

fn main() {
    println!("Omni Stage-0 Predicate Engine Demo");
    
    // Test basic functionality
    let engine = Stage0PredicateEngine::from_lists(
        &["functions".to_string(), "affine_ownership".to_string()],
        &["macros".to_string(), "async".to_string()],
        "1.0.0"
    ).unwrap();
    
    println!("Engine created successfully");
    
    // Test feature validation
    match engine.select_profile("stage0") {
        Ok(profile) => {
            println!("Profile selected: stage0");
            match profile.check("functions") {
                Ok(_) => println!("✓ functions is allowed"),
                Err(e) => println!("✗ functions check failed: {}", e),
            }
            match profile.check("macros") {
                Ok(_) => println!("✓ macros is allowed"),
                Err(e) => println!("✗ macros check failed: {}", e),
            }
            
            // Test unknown feature
            match profile.check("unknown_feature") {
                Ok(_) => println!("✓ unknown_feature is allowed"),
                Err(e) => println!("✗ unknown_feature check failed: {}", e),
            }
        }
        Err(e) => println!("Failed to select profile: {}", e),
    }
    
    // Test restriction validation
    let rules: BTreeSet<String> = ["STAGE0-0007".to_string()].into_iter().collect();
    let productions: BTreeSet<String> = ["function_def".to_string()].into_iter().collect();
    
    println!("\nTesting restriction validation...")
    match validate_restrictions(&[], &rules, &productions) {
        Ok(_) => println!("✓ Valid restriction"),
        Err(e) => println!("✗ Restriction validation failed: {}", e),
    }
}