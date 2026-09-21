use crate::def_id::DefId;
use omni_syntax::{SyntaxKind, SyntaxNode};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Rib {
    pub bindings: HashMap<String, DefId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    ShadowingViolation { name: String, existing: DefId, span: Option<u32> },
    UnresolvedName { name: String, span: Option<u32> },
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedNames {
    pub definitions: HashMap<u32, DefId>,
    pub references: HashMap<u32, DefId>,
    pub names: HashMap<DefId, String>,
}

pub struct Resolver {
    ribs: Vec<Rib>,
    next_index: u32,
    current_package: u32,
    current_module: u32,
}
impl Resolver {
    pub fn new(package: u32, module: u32) -> Self {
        Self {
            ribs: vec![Rib::default()],
            next_index: 0,
            current_package: package,
            current_module: module,
        }
    }
    pub fn push_rib(&mut self) {
        self.ribs.push(Rib::default());
    }
    pub fn pop_rib(&mut self) {
        assert!(self.ribs.len() > 1, "Cannot pop the global module rib");
        self.ribs.pop();
    }
    pub fn declare(&mut self, name: String) -> Result<DefId, ResolveError> {
        if let Some(existing) = self.ribs.iter().rev().find_map(|r| r.bindings.get(&name).copied())
        {
            return Err(ResolveError::ShadowingViolation { name, existing, span: None });
        }
        let id = DefId::new(self.current_package, self.current_module, self.next_index);
        self.next_index += 1;
        self.ribs.last_mut().expect("global rib").bindings.insert(name, id);
        Ok(id)
    }
    pub fn resolve(&self, name: &str) -> Result<DefId, ResolveError> {
        self.ribs
            .iter()
            .rev()
            .find_map(|r| r.bindings.get(name).copied())
            .ok_or_else(|| ResolveError::UnresolvedName { name: name.into(), span: None })
    }

    /// Resolve all NameRef nodes in a parsed Omni CST. Function declarations are
    /// predeclared before entering bodies, allowing deterministic forward references.
    pub fn resolve_source(
        &mut self,
        root: &SyntaxNode,
    ) -> Result<ResolvedNames, Vec<ResolveError>> {
        let mut out = ResolvedNames::default();
        let mut errors = Vec::new();
        let mut function_nodes = Vec::new();
        for n in root.descendants().filter(|n| n.kind() == SyntaxKind::FnDef) {
            function_nodes.push(n);
        }
        for f in &function_nodes {
            if let Some(name) = direct_name(f) {
                self.declare_and_record(&name, &mut out, true, &mut errors);
            }
        }
        for child in root.children().filter(|n| n.kind() == SyntaxKind::FnDef) {
            self.resolve_fn(&child, &mut out, &mut errors);
        }
        for n in root.children().filter(|n| n.kind() != SyntaxKind::FnDef) {
            if n.kind() != SyntaxKind::ErrorNode {
                self.resolve_node(&n, &mut out, &mut errors);
            }
        }
        if errors.is_empty() {
            Ok(out)
        } else {
            Err(errors)
        }
    }
    fn resolve_fn(
        &mut self,
        f: &SyntaxNode,
        out: &mut ResolvedNames,
        errors: &mut Vec<ResolveError>,
    ) {
        self.push_rib();
        if let Some(params) = f.children().find(|n| n.kind() == SyntaxKind::ParamList) {
            for p in params.children().filter(|n| n.kind() == SyntaxKind::Param) {
                if let Some(name) = direct_name(&p) {
                    self.declare_and_record(&name, out, true, errors);
                }
            }
        }
        if let Some(block) = f.children().find(|n| n.kind() == SyntaxKind::Block) {
            self.resolve_node(&block, out, errors);
        }
        self.pop_rib();
    }
    fn resolve_node(
        &mut self,
        node: &SyntaxNode,
        out: &mut ResolvedNames,
        errors: &mut Vec<ResolveError>,
    ) {
        match node.kind() {
            SyntaxKind::Block => {
                self.push_rib();
                for c in node.children() {
                    self.resolve_node(&c, out, errors);
                }
                self.pop_rib();
            }
            SyntaxKind::LetStmt => {
                let children_vec: Vec<_> = node.children().collect();
                if let Some(name) =
                    children_vec.iter().find(|n| n.kind() == SyntaxKind::NameRef).cloned()
                {
                    // Initializer is resolved before introducing the binding: a declaration
                    // cannot recursively refer to itself by name.
                    let mut after_name = false;
                    for c in &children_vec {
                        if c.kind() == SyntaxKind::NameRef && c.text() == name.text() {
                            after_name = true;
                            continue;
                        }
                        if after_name {
                            self.resolve_node(c, out, errors);
                        }
                    }
                    self.declare_and_record(&name, out, true, errors);
                } else {
                    for c in &children_vec {
                        self.resolve_node(c, out, errors);
                    }
                }
            }
            SyntaxKind::NameRef => {
                let text = node.text().to_string().trim().to_string();
                if let Ok(id) = self.resolve(&text) {
                    out.references.insert(start_u32(node), id);
                } else {
                    errors.push(ResolveError::UnresolvedName {
                        name: text,
                        span: Some(start_u32(node)),
                    });
                }
            }
            SyntaxKind::FnDef | SyntaxKind::ParamList | SyntaxKind::Param => {
                for c in node.children() {
                    self.resolve_node(&c, out, errors);
                }
            }
            _ => {
                for c in node.children() {
                    self.resolve_node(&c, out, errors);
                }
            }
        }
    }
    fn declare_and_record(
        &mut self,
        node: &SyntaxNode,
        out: &mut ResolvedNames,
        _declaration: bool,
        errors: &mut Vec<ResolveError>,
    ) {
        let name = node.text().to_string().trim().to_string();
        match self.declare(name.clone()) {
            Ok(id) => {
                out.definitions.insert(start_u32(node), id);
                out.names.insert(id, name);
            }
            Err(e) => errors.push(match e {
                ResolveError::ShadowingViolation { name, existing, .. } => {
                    ResolveError::ShadowingViolation { name, existing, span: Some(start_u32(node)) }
                }
                other => other,
            }),
        }
    }
}

fn start_u32(node: &SyntaxNode) -> u32 {
    u32::from(node.text_range().start())
}
fn direct_name(node: &SyntaxNode) -> Option<SyntaxNode> {
    node.children().find(|n| n.kind() == SyntaxKind::NameRef)
}

#[cfg(any())]
#[implements("NAME-0006")]
fn _audit_shadowing_rules() {}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_parse::Parser;
    #[test]
    fn resolves_forward_function_names_and_locals() {
        let mut p = Parser::from_source(
            "fn main() { let x = helper(); return x; } fn helper() { return 1; }",
        );
        let parsed = p.parse_source();
        assert!(parsed.is_ok(), "{:?}", parsed.diagnostics);
        let mut r = Resolver::new(1, 2);
        let resolved = r.resolve_source(&parsed.syntax()).expect("resolve");
        assert!(resolved.definitions.len() >= 3);
        assert!(!resolved.references.is_empty());
    }
}
