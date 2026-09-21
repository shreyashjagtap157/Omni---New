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

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("implements") {
            if let Ok(lit) = syn::parse2::<syn::LitStr>(mac.tokens.clone()) {
                self.implements.insert(lit.value());
            }
        }
        syn::visit::visit_macro(self, mac);
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
        if rules_by_id.contains_key(&rule.rule_id) {
            eprintln!("REAPER ERROR: duplicate rule {}", rule.rule_id);
            std::process::exit(1);
        }
        rules_by_id.insert(rule.rule_id.clone(), rule);
    }

    for rule in rules_by_id.values() {
        match rule.status.as_str() {
            "Proposed" | "Candidate" | "Ratified" | "Deprecated" | "Superseded" | "Withdrawn" => {}
            other => {
                eprintln!("REAPER ERROR: {} has unknown status {other}", rule.rule_id);
                std::process::exit(1);
            }
        }
        if rule.status == "Ratified" && rule.witness_tests.is_empty() {
            eprintln!("REAPER ERROR: Ratified rule {} has no witness tests", rule.rule_id);
            std::process::exit(1);
        }
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
            Some(rule) if rule.status == "Superseded" => {
                eprintln!("REAPER ERROR: Superseded rule {impl_rule}");
                std::process::exit(1);
            }
            Some(rule) if rule.status == "Withdrawn" => {
                eprintln!("REAPER ERROR: Withdrawn rule {impl_rule}");
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

    // Dependency cycles fail the reaper (RULE-0005: dependencies acyclic).
    // Iterative DFS keeps stack usage bounded on large registries.
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum Mark {
        Temp,
        Perm,
    }
    let mut marks: BTreeMap<&str, Mark> = BTreeMap::new();
    for id in rules_by_id.keys() {
        if marks.contains_key(id.as_str()) {
            continue;
        }
        let mut stack: Vec<(&str, bool)> = vec![(id.as_str(), false)];
        while let Some((node, expanded)) = stack.pop() {
            if expanded {
                marks.insert(node, Mark::Perm);
                continue;
            }
            match marks.get(node) {
                Some(Mark::Perm) => continue,
                Some(Mark::Temp) => {
                    eprintln!("REAPER ERROR: dependency cycle at {node}");
                    std::process::exit(1);
                }
                None => {}
            }
            marks.insert(node, Mark::Temp);
            stack.push((node, true));
            if let Some(rule) = rules_by_id.get(node) {
                for dep in &rule.dependencies {
                    match marks.get(dep.as_str()) {
                        Some(Mark::Temp) => {
                            eprintln!("REAPER ERROR: dependency cycle at {dep}");
                            std::process::exit(1);
                        }
                        Some(Mark::Perm) => {}
                        None => stack.push((dep.as_str(), false)),
                    }
                }
            }
        }
    }

    println!("Semantic linkage clean. Checked {} implementation tags.", visitor.implements.len());
    Ok(())
}
