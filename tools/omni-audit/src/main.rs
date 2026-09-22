//! Semantic linkage reaper (0.0.0.7).
//!
//! Ownership model (single, documented here):
//! - Implementation claims are `#[implements("RULE-ID")]` attributes and
//!   `implements!("RULE-ID")` macros found in `.rs` files under `compiler/`
//!   and `tools/` (generated `target/` dirs excluded). Claims inside test
//!   modules count as claims; attribution records their file.
//! - Every claim must resolve to a registry rule in an ownable lifecycle
//!   state (`Candidate`, `Ratified`). `Proposed`, `Deprecated`, `Superseded`,
//!   and `Withdrawn` ownership fails; unknown or malformed IDs fail;
//!   malformed claim syntax fails instead of being silently ignored.
//! - Witness entries must each resolve to a contained relative file under the
//!   workspace root or to a collected `#[test]` function name.
//! - Coverage is per-rule, not per-file: every `Ratified` rule must be both
//!   claimed and witnessed. No file is required to carry an annotation;
//!   infrastructure, derived, and oracle code without tags is permitted.
//! - The registry itself must be duplicate-free, dependency-closed, acyclic,
//!   and status-valid. Malformed inputs (JSON or Rust) fail closed without
//!   panicking. Traversal is filename-sorted, so verdicts are deterministic.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

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

/// One attributable implementation claim.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Claim {
    pub rule_id: String,
    pub file: String,
    pub kind: &'static str,
}

#[derive(Debug)]
pub struct AuditReport {
    pub claims: Vec<Claim>,
    pub files_walked: usize,
    pub test_fns: usize,
}

fn err(msg: String) -> Result<AuditReport, String> {
    Err(format!("REAPER ERROR: {msg}"))
}

struct LinkageVisitor {
    root: PathBuf,
    claims: Vec<Claim>,
    test_fns: BTreeSet<String>,
    error: Option<String>,
}

impl LinkageVisitor {
    fn rel(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().into_owned())
    }
}

impl<'ast> Visit<'ast> for LinkageVisitor {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if attr.path().is_ident("implements") {
            match attr.parse_args::<syn::LitStr>() {
                Ok(lit) => self.claims.push(Claim {
                    rule_id: lit.value(),
                    file: String::new(),
                    kind: "attribute",
                }),
                Err(_) => {
                    self.error.get_or_insert_with(|| {
                        "malformed #[implements(..)] attribute (expected one string literal)"
                            .to_string()
                    });
                }
            }
        }
        syn::visit::visit_attribute(self, attr);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("implements") {
            match syn::parse2::<syn::LitStr>(mac.tokens.clone()) {
                Ok(lit) => self.claims.push(Claim {
                    rule_id: lit.value(),
                    file: String::new(),
                    kind: "macro",
                }),
                Err(_) => {
                    self.error.get_or_insert_with(|| {
                        "malformed implements!(..) macro (expected one string literal)".to_string()
                    });
                }
            }
        }
        syn::visit::visit_macro(self, mac);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.attrs.iter().any(|a| a.path().is_ident("test")) {
            self.test_fns.insert(node.sig.ident.to_string());
        }
        syn::visit::visit_item_fn(self, node);
    }
}

/// A witness entry resolves to a contained workspace file or a test fn name.
/// Anything absolute, escaping, or missing fails closed.
fn resolve_witness(
    entry: &str,
    workspace_root: &Path,
    test_fns: &BTreeSet<String>,
) -> Result<(), String> {
    let looks_like_path = entry.contains('/') || entry.contains('\\') || entry.contains('.');
    if looks_like_path {
        let candidate = PathBuf::from(entry);
        if candidate.is_absolute()
            || candidate.components().any(|c| {
                matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir)
            })
        {
            return Err(format!("witness escapes workspace: {entry}"));
        }
        let full = workspace_root.join(&candidate);
        if full.is_file() {
            Ok(())
        } else {
            Err(format!("unresolved witness path: {entry}"))
        }
    } else if entry.is_empty() || !test_fns.contains(entry) {
        Err(format!("unresolved witness test: {entry}"))
    } else {
        Ok(())
    }
}

pub fn run_audit(workspace_root: &Path) -> Result<AuditReport, String> {
    let fail = |m: String| err(m);

    let rules_path = workspace_root.join("spec/registry/rules.json");
    if !rules_path.is_file() {
        return fail("missing spec/registry/rules.json".to_string());
    }
    let registry_data = fs::read_to_string(&rules_path)
        .map_err(|e| format!("REAPER ERROR: registry unreadable: {e}"))?;
    let registry: RuleRegistry = serde_json::from_str(&registry_data)
        .map_err(|e| format!("REAPER ERROR: registry invalid: {e}"))?;

    let mut rules_by_id: BTreeMap<String, &RegistryRecord> = BTreeMap::new();
    for rule in &registry.rules {
        if rules_by_id.contains_key(&rule.rule_id) {
            return fail(format!("duplicate rule {}", rule.rule_id));
        }
        rules_by_id.insert(rule.rule_id.clone(), rule);
    }
    for rule in rules_by_id.values() {
        if !omni_registry::LIFECYCLE_STATES.contains(&rule.status.as_str()) {
            return fail(format!("{} has unknown status {}", rule.rule_id, rule.status));
        }
        if rule.status == "Ratified" && rule.witness_tests.is_empty() {
            return fail(format!("Ratified rule {} has no witness tests", rule.rule_id));
        }
        for dep in &rule.dependencies {
            if !rules_by_id.contains_key(dep) {
                return fail(format!("{id} depends on unknown rule {dep}", id = rule.rule_id));
            }
        }
    }
    // Iterative cycle check (RULE-0005); explicit stack keeps memory bounded.
    {
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
                    Some(Mark::Temp) => return fail(format!("dependency cycle at {node}")),
                    None => {}
                }
                marks.insert(node, Mark::Temp);
                stack.push((node, true));
                if let Some(rule) = rules_by_id.get(node) {
                    for dep in &rule.dependencies {
                        match marks.get(dep.as_str()) {
                            Some(Mark::Temp) => {
                                return fail(format!("dependency cycle at {dep}"));
                            }
                            Some(Mark::Perm) => {}
                            None => stack.push((dep.as_str(), false)),
                        }
                    }
                }
            }
        }
    }

    let mut visitor = LinkageVisitor {
        root: workspace_root.to_path_buf(),
        claims: Vec::new(),
        test_fns: BTreeSet::new(),
        error: None,
    };
    let mut files_walked = 0usize;
    for tree in ["compiler", "tools"] {
        let base = workspace_root.join(tree);
        if !base.is_dir() {
            return fail(format!("missing source tree: {tree}"));
        }
        let walker = WalkDir::new(&base).sort_by_file_name().into_iter().filter_entry(|e| {
            // Generated build outputs carry no normative ownership.
            e.file_name().to_string_lossy() != "target"
        });
        for entry in walker {
            let entry = entry.map_err(|e| format!("REAPER ERROR: walk failed: {e}"))?;
            let path = entry.path();
            if entry.file_type().is_symlink() {
                return fail(format!("symlink not followed: {}", visitor.rel(path)));
            }
            if !entry.file_type().is_file() {
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            files_walked += 1;
            let content = fs::read_to_string(path)
                .map_err(|e| format!("REAPER ERROR: unreadable {}: {e}", visitor.rel(path)))?;
            let file = syn::parse_file(&content)
                .map_err(|e| format!("REAPER ERROR: unparseable {}: {e}", visitor.rel(path)))?;
            let rel = visitor.rel(path);
            let before = visitor.claims.len();
            visitor.visit_file(&file);
            for claim in &mut visitor.claims[before..] {
                claim.file = rel.clone();
            }
            if let Some(e) = visitor.error.clone() {
                return fail(format!("{} in {}", e, rel));
            }
        }
    }
    visitor.claims.sort();

    for claim in &visitor.claims {
        match rules_by_id.get(&claim.rule_id) {
            None => return fail(format!("unknown rule {}", claim.rule_id)),
            Some(rule) if !omni_registry::is_ownable_status(&rule.status) => {
                return fail(format!("{} rule {}", rule.status, claim.rule_id));
            }
            Some(rule) if rule.status == "Ratified" && rule.witness_tests.is_empty() => {
                return fail(format!("Ratified rule {} has no witness tests", claim.rule_id));
            }
            _ => {}
        }
    }

    for rule in rules_by_id.values() {
        for witness in &rule.witness_tests {
            resolve_witness(witness, workspace_root, &visitor.test_fns)
                .map_err(|e| format!("REAPER ERROR: {}: {e}", rule.rule_id))?;
        }
        if rule.status == "Ratified" {
            let claimed = visitor.claims.iter().any(|c| c.rule_id == rule.rule_id);
            if !claimed {
                return fail(format!("Ratified rule {} has no implementation", rule.rule_id));
            }
        }
    }

    Ok(AuditReport { claims: visitor.claims, files_walked, test_fns: visitor.test_fns.len() })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = match args.get(1).map(PathBuf::from) {
        Some(explicit) => explicit,
        None => match omni_registry::discover_spec_root()
            .and_then(|spec| omni_registry::workspace_root_for_spec(&spec))
        {
            Ok(root) => root,
            Err(e) => {
                eprintln!("REAPER ERROR: workspace discovery failed: {e}");
                std::process::exit(101);
            }
        },
    };
    match run_audit(&root) {
        Ok(report) => {
            println!(
                "Semantic linkage clean. Checked {} implementation tags in {} files ({} test fns).",
                report.claims.len(),
                report.files_walked,
                report.test_fns
            );
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(101);
        }
    }
}

#[cfg(test)]
mod reaper_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn rule(id: &str, status: &str, deps: &[&str], witnesses: &[&str]) -> String {
        let deps = deps.iter().map(|d| format!("\"{d}\"")).collect::<Vec<_>>().join(",");
        let wit = witnesses.iter().map(|w| format!("\"{w}\"")).collect::<Vec<_>>().join(",");
        format!(
            "{{\"rule_id\":\"{id}\",\"domain\":\"OMNI-LEX\",\"status\":\"{status}\",\
             \"normative\":true,\"text_hash\":\"{h}\",\"dependencies\":[{deps}],\
             \"witness_tests\":[{wit}]}}",
            h = "0".repeat(64)
        )
    }

    /// Minimal fixture repo: registry + both source trees always present.
    fn fixture_repo(rules: &[String], files: &[(&str, &str)]) -> PathBuf {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("omni-audit-{}-{id}", std::process::id()));
        fs::create_dir_all(root.join("spec/registry")).expect("mkdir");
        fs::create_dir_all(root.join("compiler")).expect("mkdir");
        fs::create_dir_all(root.join("tools")).expect("mkdir");
        fs::write(
            root.join("spec/registry/rules.json"),
            format!("{{\"schema_version\":\"1.0.0\",\"rules\":[{}]}}", rules.join(",")),
        )
        .expect("write registry");
        for (rel, content) in files {
            let path = root.join(rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("mkdir parent");
            }
            fs::write(&path, content).expect("write file");
        }
        root
    }

    fn clean() -> (Vec<String>, Vec<(&'static str, &'static str)>) {
        (
            vec![rule("LEX-0001", "Candidate", &[], &[]), rule("LEX-0002", "Candidate", &[], &[])],
            vec![
                ("compiler/a.rs", "#[implements(\"LEX-0001\")]\nfn a() {}\n#[test]\nfn t_a() {}\n"),
                ("tools/b.rs", "implements!(\"LEX-0002\");\nfn b() {}\n"),
            ],
        )
    }

    #[test]
    fn clean_fixture_passes() {
        let (rules, files) = clean();
        let root = fixture_repo(&rules, &files);
        let report = run_audit(&root).expect("pass");
        assert_eq!(report.claims.len(), 2);
        assert_eq!(report.test_fns, 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn repeated_runs_converge_exactly() {
        let (rules, files) = clean();
        let root = fixture_repo(&rules, &files);
        let first = run_audit(&root).expect("pass");
        let second = run_audit(&root).expect("pass");
        assert_eq!(first.claims, second.claims);
        assert_eq!(first.files_walked, second.files_walked);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn registry_order_does_not_change_verdict() {
        let (mut rules, files) = clean();
        rules.reverse();
        let root = fixture_repo(&rules, &files);
        let report = run_audit(&root).expect("pass");
        assert_eq!(report.claims.len(), 2);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn non_semantic_files_do_not_change_result() {
        let (rules, mut files) = clean();
        files.push(("compiler/notes.md", "# LEX-0001 mentioned in prose\n"));
        files.push(("tools/data.txt", "LEX-0002\n"));
        let root = fixture_repo(&rules, &files);
        let report = run_audit(&root).expect("pass");
        assert_eq!(report.claims.len(), 2);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn unknown_rule_fails() {
        let (rules, mut files) = clean();
        files.push(("compiler/bad.rs", "#[implements(\"NOPE-9999\")]\nfn b() {}\n"));
        let root = fixture_repo(&rules, &files);
        let err = run_audit(&root).expect_err("must fail");
        assert!(err.contains("unknown rule NOPE-9999"), "got: {err}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn non_ownable_lifecycles_fail() {
        for status in ["Proposed", "Deprecated", "Superseded", "Withdrawn", "ErratumCorrected"] {
            let rules = vec![rule("LEX-0001", status, &[], &[])];
            let files = vec![("compiler/a.rs", "#[implements(\"LEX-0001\")]\nfn a() {}\n")];
            let root = fixture_repo(&rules, &files);
            let err = run_audit(&root).expect_err("must fail");
            assert!(err.contains(status), "got: {err}");
            fs::remove_dir_all(&root).ok();
        }
    }

    #[test]
    fn ratified_requires_witness_and_implementation() {
        // No witness at all.
        let rules = vec![rule("LEX-0001", "Ratified", &[], &[])];
        let files = vec![("compiler/a.rs", "#[implements(\"LEX-0001\")]\nfn a() {}\n")];
        let root = fixture_repo(&rules, &files);
        assert!(run_audit(&root).is_err());
        fs::remove_dir_all(&root).ok();
        // Witness present but implementation missing.
        let rules = vec![rule("LEX-0001", "Ratified", &[], &["compiler/wit.rs"])];
        let files = vec![("compiler/wit.rs", "// witness artifact\n")];
        let root = fixture_repo(&rules, &files);
        let err = run_audit(&root).expect_err("must fail");
        assert!(err.contains("no implementation"), "got: {err}");
        fs::remove_dir_all(&root).ok();
        // Both present: passes (witness as file, plus test-fn witness).
        let rules = vec![rule("LEX-0001", "Ratified", &[], &["compiler/wit.rs", "t_wit"])];
        let files = vec![
            ("compiler/a.rs", "#[implements(\"LEX-0001\")]\nfn a() {}\n#[test]\nfn t_wit() {}\n"),
            ("compiler/wit.rs", "// witness artifact\n"),
        ];
        let root = fixture_repo(&rules, &files);
        assert!(run_audit(&root).is_ok());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn unresolved_witnesses_fail() {
        for witness in ["missing.rs", "../escape.rs", "/abs/path.rs", "no_such_test"] {
            let rules = vec![rule("LEX-0001", "Candidate", &[], &[witness])];
            let files = vec![("compiler/a.rs", "fn a() {}\n")];
            let root = fixture_repo(&rules, &files);
            let err = run_audit(&root).expect_err("must fail");
            assert!(err.contains("witness"), "got: {err}");
            fs::remove_dir_all(&root).ok();
        }
    }

    #[test]
    fn unknown_dependency_and_cycle_fail() {
        let rules = vec![rule("LEX-0001", "Candidate", &["GHOST-0001"], &[])];
        let files = vec![("compiler/a.rs", "fn a() {}\n")];
        let root = fixture_repo(&rules, &files);
        assert!(run_audit(&root).expect_err("must fail").contains("unknown rule"));
        fs::remove_dir_all(&root).ok();

        let rules = vec![
            rule("LEX-0001", "Candidate", &["LEX-0002"], &[]),
            rule("LEX-0002", "Candidate", &["LEX-0003"], &[]),
            rule("LEX-0003", "Candidate", &["LEX-0001"], &[]),
        ];
        let root = fixture_repo(&rules, &files);
        assert!(run_audit(&root).expect_err("must fail").contains("cycle"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn duplicate_registry_ids_fail() {
        let rules =
            vec![rule("LEX-0001", "Candidate", &[], &[]), rule("LEX-0001", "Candidate", &[], &[])];
        let files = vec![("compiler/a.rs", "fn a() {}\n")];
        let root = fixture_repo(&rules, &files);
        assert!(run_audit(&root).expect_err("must fail").contains("duplicate"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn malformed_inputs_fail_without_panic() {
        let files = vec![("compiler/a.rs", "fn a() {}\n")];
        // Malformed JSON.
        let root = fixture_repo(&[String::from("{\"broken\"")], &files);
        assert!(run_audit(&root).is_err());
        fs::remove_dir_all(&root).ok();
        // Missing registry.
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let bare = std::env::temp_dir().join(format!("omni-audit-{}-{id}", std::process::id()));
        fs::create_dir_all(bare.join("compiler")).expect("mkdir");
        fs::create_dir_all(bare.join("tools")).expect("mkdir");
        assert!(run_audit(&bare).expect_err("must fail").contains("missing"));
        fs::remove_dir_all(&bare).ok();
        // Unparseable Rust is a hard failure, not a silent skip.
        let (rules, _) = clean();
        let root = fixture_repo(&rules, &[("compiler/a.rs", "fn a( {}\n")]);
        assert!(run_audit(&root).expect_err("must fail").contains("unparseable"));
        fs::remove_dir_all(&root).ok();
        // Unknown lifecycle status.
        let rules = vec![rule("LEX-0001", "Erratumish", &[], &[])];
        let root = fixture_repo(&rules, &files);
        assert!(run_audit(&root).expect_err("must fail").contains("unknown status"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn malformed_claim_syntax_fails() {
        for src in [
            "#[implements(foo)]\nfn a() {}\n",
            "#[implements(\"A\", \"B\")]\nfn a() {}\n",
            "implements!(42);\nfn a() {}\n",
        ] {
            let (rules, _) = clean();
            let root = fixture_repo(&rules, &[("compiler/a.rs", src)]);
            let err = run_audit(&root).expect_err("must fail");
            assert!(err.contains("malformed"), "got: {err}");
            fs::remove_dir_all(&root).ok();
        }
    }

    #[test]
    fn doc_prose_is_not_linkage() {
        let (rules, _) = clean();
        let root = fixture_repo(
            &rules,
            &[("compiler/a.rs", "//! Linkage: #[implements(\"FAKE-0001\")]\nfn a() {}\n")],
        );
        let report = run_audit(&root).expect("prose is not a claim");
        assert!(report.claims.is_empty());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn duplicate_claims_are_attributed_not_hidden() {
        let (rules, _) = clean();
        let root = fixture_repo(
            &rules,
            &[
                ("compiler/a.rs", "#[implements(\"LEX-0001\")]\nfn a() {}\n"),
                ("tools/b.rs", "implements!(\"LEX-0001\");\nfn b() {}\n"),
            ],
        );
        let report = run_audit(&root).expect("collaboration allowed");
        assert_eq!(report.claims.len(), 2);
        assert_ne!(report.claims[0].file, report.claims[1].file);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn symlink_entries_fail_closed() {
        let (rules, files) = clean();
        let root = fixture_repo(&rules, &files);
        let target = root.join("compiler/a.rs");
        let link = root.join("compiler/link.rs");
        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(&target, &link).is_ok();
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_file(&target, &link).is_ok();
        #[cfg(not(any(unix, windows)))]
        let made = false;
        if !made {
            fs::remove_dir_all(&root).ok();
            return;
        }
        let err = run_audit(&root).expect_err("must fail");
        assert!(err.contains("symlink"), "got: {err}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn scale_fixture_passes() {
        // Hundreds of claims stay well within the memory cap and stay ordered.
        let (mut rules, _) = clean();
        let mut src = String::new();
        for i in 1..=200 {
            let id = format!("LEX-{i:04}");
            if i > 2 {
                rules.push(rule(&id, "Candidate", &[], &[]));
            }
            src.push_str(&format!("#[implements(\"{id}\")]\nfn f{i}() {{}}\n"));
        }
        let root = fixture_repo(&rules, &[("compiler/big.rs", &src)]);
        let report = run_audit(&root).expect("pass");
        assert_eq!(report.claims.len(), 200);
        let mut sorted = report.claims.clone();
        sorted.sort();
        assert_eq!(report.claims, sorted);
        fs::remove_dir_all(&root).ok();
    }
}
