use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use syn::visit::Visit;
use walkdir::WalkDir;

#[derive(Deserialize)]
struct RegistryRecord {
    rule_id: String,
    status: String,
    dependencies: Vec<String>,
    witness_tests: Vec<String>,
}

#[derive(Deserialize)]
struct RuleRegistry {
    rules: Vec<RegistryRecord>,
}

struct LinkageVisitor {
    implements: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for LinkageVisitor {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if attr.path().is_ident("implements") {
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                self.implements.insert(lit.value());
            }
        }
        syn::visit::visit_attribute(self, attr);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_path = PathBuf::from("spec/registry/rules.json");
    if !rules_path.exists() {
        eprintln!("REAPER ERROR: missing spec/registry/rules.json");
        std::process::exit(1);
    }

    let registry_data = fs::read_to_string(&rules_path)?;
    let registry: RuleRegistry = serde_json::from_str(&registry_data)?;
    let mut rules_by_id = BTreeMap::new();
    for rule in registry.rules {
        rules_by_id.insert(rule.rule_id.clone(), rule);
    }

    let mut visitor = LinkageVisitor { implements: BTreeSet::new() };
    for entry in WalkDir::new("compiler").into_iter().filter_map(Result::ok) {
        if entry.path().extension().is_some_and(|ext| ext == "rs") {
            let content = fs::read_to_string(entry.path())?;
            if let Ok(file) = syn::parse_file(&content) {
                visitor.visit_file(&file);
            }
        }
    }

    for impl_rule in &visitor.implements {
        match rules_by_id.get(impl_rule) {
            None => {
                eprintln!("REAPER ERROR: unknown rule {impl_rule}");
                std::process::exit(1);
            }
            Some(rule) if rule.status == "Proposed" => {
                eprintln!("REAPER ERROR: Proposed rule {impl_rule}");
                std::process::exit(1);
            }
            Some(rule) if rule.status == "Deprecated" => {
                eprintln!("REAPER ERROR: Deprecated rule {impl_rule}");
                std::process::exit(1);
            }
            Some(rule) if rule.status == "Ratified" && rule.witness_tests.is_empty() => {
                eprintln!("REAPER ERROR: Ratified rule {impl_rule} has no witness tests");
                std::process::exit(1);
            }
            _ => {}
        }
    }

    for rule in rules_by_id.values() {
        for dep in &rule.dependencies {
            if !rules_by_id.contains_key(dep) {
                eprintln!("REAPER ERROR: {id} depends on unknown rule {dep}", id = rule.rule_id);
                std::process::exit(1);
            }
        }
    }

    println!("Semantic linkage clean. Checked {} implementation tags.", visitor.implements.len());
    Ok(())
}
