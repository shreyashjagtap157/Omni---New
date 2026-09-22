//! Compiler-crate topology invariant (0.0.0.9).
//!
//! Pipeline-order tiers (higher number = later stage; a production edge may
//! only point at the same or an earlier tier). The driver orchestrates and
//! may use any compiler tier. The below-MIR consumers (`machine`, `codegen`,
//! `verify`) additionally declare exact feed tiers so frontend syntax can
//! never be consumed past the lowering boundary. Infrastructure (`stage0`)
//! carries no compiler edges at all; `tools/*` may use tools and externals
//! but never `compiler/*`, and no compiler crate may depend on tooling.
//! Dev/build edges are direction-exempt but cycle-checked like all edges.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

/// Pipeline tier rank; higher = later stage.
fn tier(crate_name: &str) -> Option<u8> {
    Some(match crate_name {
        "omni-source" => 0,
        "omni-lex" => 1,
        "omni-syntax" => 2,
        "omni-parse" => 3,
        "omni-names" => 4,
        "omni-types" => 5,
        "omni-own" | "omni-effects" | "omni-traits" => 6,
        "omni-hir" => 7,
        "omni-mir" => 8,
        "omni-verify" => 9,
        "omni-machine" | "omni-codegen" => 10,
        "omni-runtime" => 11,
        _ => return None,
    })
}

fn is_driver(crate_name: &str) -> bool {
    crate_name == "omni-driver"
}

fn is_infra(crate_name: &str) -> bool {
    crate_name == "omni-stage0"
}

fn is_tool(crate_name: &str) -> bool {
    crate_name.starts_with("omni-")
        && matches!(
            crate_name,
            "omni-audit"
                | "omni-bindgen"
                | "omni-canon"
                | "omni-conform"
                | "omni-evidence"
                | "omni-registry"
                | "omni-topology"
        )
}

fn is_compiler(crate_name: &str) -> bool {
    tier(crate_name).is_some() || is_driver(crate_name) || is_infra(crate_name)
}

/// Exact feed tiers for below-MIR consumers: lowered representations only.
fn feeds(crate_name: &str) -> Option<&'static [&'static str]> {
    match crate_name {
        "omni-machine" => Some(&["omni-mir"]),
        "omni-codegen" => Some(&["omni-mir", "omni-verify"]),
        "omni-verify" => Some(&["omni-mir"]),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub from: String,
    pub to: String,
    /// "normal", "dev", or "build".
    pub kind: String,
}

#[derive(Debug)]
pub struct TopologyReport {
    pub members: usize,
    pub normal_edges: usize,
    pub dev_edges: usize,
    pub build_edges: usize,
}

fn err<T>(msg: String) -> Result<T, String> {
    Err(format!("TOPOLOGY ERROR: {msg}"))
}

fn classify(name: &str) -> Result<(), String> {
    if is_compiler(name) || is_tool(name) {
        Ok(())
    } else {
        err(format!("unknown/unclassified crate: {name}"))
    }
}

/// Check a workspace-internal edge set. Edges are sorted first, so verdicts
/// and diagnostics are deterministic regardless of input order.
pub fn check_graph(members: &[String], edges: &[Edge]) -> Result<TopologyReport, String> {
    for member in members {
        classify(member)?;
    }
    let mut sorted: Vec<&Edge> = edges.iter().collect();
    sorted.sort();
    for edge in &sorted {
        classify(&edge.from)?;
        classify(&edge.to)?;
    }

    // Global acyclicity over every edge category (iterative DFS).
    {
        #[derive(PartialEq, Eq, Clone, Copy)]
        enum Mark {
            Temp,
            Perm,
        }
        let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in &sorted {
            adj.entry(edge.from.as_str()).or_default().push(edge.to.as_str());
        }
        for lists in adj.values_mut() {
            lists.sort();
            lists.dedup();
        }
        let mut marks: BTreeMap<&str, Mark> = BTreeMap::new();
        for id in adj.keys() {
            if marks.contains_key(*id) {
                continue;
            }
            let mut stack: Vec<(&str, bool)> = vec![(id, false)];
            while let Some((node, expanded)) = stack.pop() {
                if expanded {
                    marks.insert(node, Mark::Perm);
                    continue;
                }
                match marks.get(node) {
                    Some(Mark::Perm) => continue,
                    Some(Mark::Temp) => return err(format!("dependency cycle at {node}")),
                    None => {}
                }
                marks.insert(node, Mark::Temp);
                stack.push((node, true));
                if let Some(next) = adj.get(node) {
                    for dep in next.iter() {
                        match marks.get(*dep) {
                            Some(Mark::Temp) => {
                                return err(format!("dependency cycle at {dep}"));
                            }
                            Some(Mark::Perm) => {}
                            None => stack.push((dep, false)),
                        }
                    }
                }
            }
        }
    }

    let mut normal = 0usize;
    let mut dev = 0usize;
    let mut build = 0usize;
    for edge in sorted {
        match edge.kind.as_str() {
            "dev" => {
                dev += 1;
                continue;
            }
            "build" => {
                build += 1;
                continue;
            }
            _ => normal += 1,
        }
        let from = edge.from.as_str();
        let to = edge.to.as_str();
        if is_tool(from) {
            if is_compiler(to) {
                return err(format!("tooling depends on compiler crate: {from} -> {to}"));
            }
            continue;
        }
        if is_tool(to) {
            return err(format!("compiler crate depends on tooling: {from} -> {to}"));
        }
        if is_infra(from) {
            return err(format!("infrastructure carries compiler edge: {from} -> {to}"));
        }
        if is_driver(from) {
            continue;
        }
        if is_driver(to) {
            return err(format!("upward edge: {from} depends on orchestrator {to}"));
        }
        let (Some(from_tier), Some(to_tier)) = (tier(from), tier(to)) else {
            return err(format!("unranked compiler edge: {from} -> {to}"));
        };
        if to_tier > from_tier {
            return err(format!("upward edge: {from} (tier {from_tier}) -> {to} (tier {to_tier})"));
        }
        if let Some(feed) = feeds(from) {
            if !feed.contains(&to) {
                return err(format!("{from} bypasses its declared feeds (uses {to})"));
            }
        }
    }

    Ok(TopologyReport {
        members: members.len(),
        normal_edges: normal,
        dev_edges: dev,
        build_edges: build,
    })
}

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
    resolve: MetadataResolve,
    workspace_members: Vec<String>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct MetadataResolve {
    nodes: Vec<MetadataNode>,
}

#[derive(Deserialize)]
struct MetadataNode {
    id: String,
    deps: Vec<MetadataDep>,
}

#[derive(Deserialize)]
struct MetadataDep {
    pkg: String,
    dep_kinds: Vec<MetadataDepKind>,
}

#[derive(Deserialize)]
struct MetadataDepKind {
    kind: Option<String>,
}

/// Build the workspace-internal edge set from `cargo metadata` output.
/// Only workspace members participate; external crates cannot cycle back.
fn edges_from_metadata(raw: &str) -> Result<(Vec<String>, Vec<Edge>), String> {
    let meta: Metadata =
        serde_json::from_str(raw).map_err(|e| format!("TOPOLOGY ERROR: metadata invalid: {e}"))?;
    let names: BTreeMap<&str, &str> =
        meta.packages.iter().map(|p| (p.id.as_str(), p.name.as_str())).collect();
    let member_names: BTreeSet<&str> =
        meta.workspace_members.iter().filter_map(|id| names.get(id.as_str()).copied()).collect();
    let mut members: Vec<String> = member_names.iter().map(ToString::to_string).collect();
    members.sort();
    let mut edges = BTreeSet::new();
    for node in &meta.resolve.nodes {
        let Some(from) = names.get(node.id.as_str()).copied() else { continue };
        if !member_names.contains(from) {
            continue;
        }
        for dep in &node.deps {
            let Some(to) = names.get(dep.pkg.as_str()).copied() else { continue };
            if !member_names.contains(to) {
                continue;
            }
            for kind in &dep.dep_kinds {
                edges.insert(Edge {
                    from: from.to_string(),
                    to: to.to_string(),
                    kind: kind.kind.clone().unwrap_or_else(|| "normal".to_string()),
                });
            }
        }
    }
    Ok((members, edges.into_iter().collect()))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| {
            eprintln!("TOPOLOGY ERROR: cannot run cargo metadata: {e}");
            std::process::exit(101);
        });
    if !output.status.success() {
        eprintln!(
            "TOPOLOGY ERROR: cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(101);
    }
    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    let (members, edges) = edges_from_metadata(&raw).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(101);
    });
    match check_graph(&members, &edges) {
        Ok(report) => {
            println!(
                "TOPOLOGY PASS: {} members, {} normal + {} dev + {} build internal edges.",
                report.members, report.normal_edges, report.dev_edges, report.build_edges
            );
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(101);
        }
    }
}

#[cfg(test)]
mod topology_tests {
    use super::*;

    fn edge(from: &str, to: &str) -> Edge {
        Edge { from: from.to_string(), to: to.to_string(), kind: "normal".to_string() }
    }

    fn dev_edge(from: &str, to: &str) -> Edge {
        Edge { from: from.to_string(), to: to.to_string(), kind: "dev".to_string() }
    }

    fn members(names: &[&str]) -> Vec<String> {
        names.iter().map(ToString::to_string).collect()
    }

    /// Mirrors the real workspace production shape after the 0.0.0.9 fixes.
    fn healthy() -> (Vec<String>, Vec<Edge>) {
        let members = members(&[
            "omni-source",
            "omni-lex",
            "omni-syntax",
            "omni-parse",
            "omni-names",
            "omni-types",
            "omni-own",
            "omni-effects",
            "omni-traits",
            "omni-hir",
            "omni-mir",
            "omni-verify",
            "omni-machine",
            "omni-codegen",
            "omni-runtime",
            "omni-driver",
            "omni-stage0",
            "omni-audit",
            "omni-canon",
        ]);
        let edges = vec![
            edge("omni-lex", "omni-source"),
            edge("omni-syntax", "omni-lex"),
            edge("omni-parse", "omni-syntax"),
            edge("omni-parse", "omni-lex"),
            edge("omni-names", "omni-syntax"),
            edge("omni-own", "omni-types"),
            edge("omni-effects", "omni-types"),
            edge("omni-traits", "omni-types"),
            edge("omni-hir", "omni-types"),
            edge("omni-hir", "omni-own"),
            edge("omni-verify", "omni-mir"),
            edge("omni-machine", "omni-mir"),
            edge("omni-codegen", "omni-mir"),
            edge("omni-driver", "omni-machine"),
            edge("omni-driver", "omni-codegen"),
            dev_edge("omni-names", "omni-parse"),
            edge("omni-canon", "omni-audit"),
        ];
        (members, edges)
    }

    #[test]
    fn healthy_graph_passes() {
        let (members, edges) = healthy();
        let report = check_graph(&members, &edges).expect("pass");
        assert_eq!(report.normal_edges, 16);
        assert_eq!(report.dev_edges, 1);
    }

    #[test]
    fn unknown_crates_detected() {
        let (members, edges) = healthy();
        let mut with_ghost = edges.clone();
        with_ghost.push(edge("omni-lex", "omni-ghost"));
        let err = check_graph(&members, &with_ghost).expect_err("unknown endpoint");
        assert!(err.contains("unknown/unclassified"), "got: {err}");
        let mut members = members;
        members.push("omni-mystery".to_string());
        let err = check_graph(&members, &edges).expect_err("unknown member");
        assert!(err.contains("unknown/unclassified"), "got: {err}");
    }

    #[test]
    fn upward_edges_fail() {
        let (members, _) = healthy();
        for (from, to) in
            [("omni-lex", "omni-parse"), ("omni-types", "omni-hir"), ("omni-mir", "omni-verify")]
        {
            let err = check_graph(&members, &[edge(from, to)]).expect_err("upward");
            assert!(err.contains("upward edge"), "got: {err}");
        }
    }

    #[test]
    fn feed_bypass_fails() {
        let (members, _) = healthy();
        for (from, to) in [
            ("omni-machine", "omni-parse"),
            ("omni-codegen", "omni-parse"),
            ("omni-verify", "omni-types"),
        ] {
            let err = check_graph(&members, &[edge(from, to)]).expect_err("bypass");
            assert!(err.contains("bypass"), "got: {err}");
        }
        // Declared feeds pass, including the codegen/verify interface.
        let report = check_graph(
            &members,
            &[
                edge("omni-machine", "omni-mir"),
                edge("omni-codegen", "omni-verify"),
                edge("omni-verify", "omni-mir"),
            ],
        )
        .expect("feeds pass");
        assert_eq!(report.normal_edges, 3);
    }

    #[test]
    fn driver_may_orchestrate_but_never_be_used_upward() {
        let (members, _) = healthy();
        check_graph(&members, &[edge("omni-driver", "omni-types")]).expect("orchestration");
        let err = check_graph(&members, &[edge("omni-types", "omni-driver")]).expect_err("upward");
        assert!(err.contains("upward edge"), "got: {err}");
    }

    #[test]
    fn tool_and_infra_boundaries_hold() {
        let (members, _) = healthy();
        let err = check_graph(&members, &[edge("omni-canon", "omni-lex")]).expect_err("tool");
        assert!(err.contains("tooling depends on compiler"), "got: {err}");
        let err = check_graph(&members, &[edge("omni-lex", "omni-canon")]).expect_err("tool");
        assert!(err.contains("depends on tooling"), "got: {err}");
        let err = check_graph(&members, &[edge("omni-stage0", "omni-mir")]).expect_err("infra");
        assert!(err.contains("infrastructure carries"), "got: {err}");
        check_graph(&members, &[edge("omni-canon", "omni-audit")]).expect("tools-tools");
    }

    #[test]
    fn dev_edges_exempt_from_direction_but_not_cycles() {
        let (members, _) = healthy();
        // Direction-exempt (mirrors the real names dev-dependency on parse).
        check_graph(&members, &[dev_edge("omni-names", "omni-parse")]).expect("dev exempt");
        // Cycles fail even when carried by dev edges.
        let err = check_graph(
            &members,
            &[edge("omni-lex", "omni-source"), dev_edge("omni-source", "omni-lex")],
        )
        .expect_err("cycle");
        assert!(err.contains("cycle"), "got: {err}");
    }

    #[test]
    fn verdicts_deterministic_regardless_of_order() {
        let (members, mut clean) = healthy();
        clean.push(edge("omni-machine", "omni-parse"));
        let mut reversed = clean.clone();
        reversed.reverse();
        let first = check_graph(&members, &clean).expect_err("bypass");
        let second = check_graph(&members, &reversed).expect_err("bypass");
        assert_eq!(first, second);
        // And removal restores a deterministic pass.
        clean.retain(|e| e != &edge("omni-machine", "omni-parse"));
        check_graph(&members, &clean).expect("restored");
    }

    #[test]
    fn dormant_downward_edges_pass() {
        // Declared-but-unused downward edges (e.g. hir scaffolding) are tier
        // intent, not violations.
        let (members, _) = healthy();
        check_graph(&members, &[edge("omni-hir", "omni-own"), edge("omni-syntax", "omni-lex")])
            .expect("dormant ok");
    }
}
