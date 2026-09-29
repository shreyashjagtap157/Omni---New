use crate::def_id::DefId;
use omni_syntax::{SyntaxKind, SyntaxNode};
use omni_unicode::ident::{canonical_ident, CanonicalIdentError};
use std::collections::HashMap;

/// A declared name, keeping the source spelling and the canonical key apart.
///
/// SRC-0003 requires that identifier *equality* use Unicode NFC while the
/// original spelling stays available for source fidelity and diagnostics. A
/// plain `String` cannot express both: it is either the spelling (so two
/// equivalent spellings compare unequal) or the key (so the spelling is lost).
///
/// This type therefore carries both, and is the only place the two are
/// reconciled:
///
/// * [`CanonicalName::key`] is what name resolution, shadowing detection, and
///   definition identity compare.
/// * [`CanonicalName::spelling`] is what diagnostics, source maps, and any
///   ABI-facing or generated name must use, because it is what the source
///   actually said.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalName {
    spelling: String,
    key: String,
}

impl CanonicalName {
    /// Builds a canonical name from an already-tokenized identifier token.
    ///
    /// Fails only for a prohibited code point, which the lexer should already
    /// have rejected (SRC-0005); re-checking here means a caller that bypasses
    /// the lexer still cannot push a bidi control into the name space.
    pub fn new(spelling: &str) -> Result<Self, CanonicalIdentError> {
        let key = canonical_ident(spelling)?;
        Ok(Self { spelling: spelling.to_string(), key })
    }

    /// The canonical comparison key (NFC), used for equality and lookup.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The original source spelling, used for diagnostics and source fidelity.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Whether the source spelling was already in canonical form.
    pub fn is_canonical(&self) -> bool {
        self.spelling == self.key
    }
}

#[derive(Debug, Default, Clone)]
pub struct Rib {
    /// Bindings keyed by canonical key, never by raw spelling (SRC-0003).
    pub bindings: HashMap<String, DefId>,
    /// The original spelling of each binding, so a diagnostic can report what
    /// the source actually said.
    pub spellings: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    ShadowingViolation {
        name: String,
        existing: DefId,
        span: Option<u32>,
    },
    UnresolvedName {
        name: String,
        span: Option<u32>,
    },
    /// The identifier contained a code point SRC-0005 prohibits. The lexer
    /// rejects these first; this variant exists so that a caller reaching name
    /// resolution through another path cannot silently accept one.
    ProhibitedIdentifier {
        name: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedNames {
    pub definitions: HashMap<u32, DefId>,
    pub references: HashMap<u32, DefId>,
    /// The original source spelling for each `DefId`, so a diagnostic reports
    /// what the source said rather than the NFC key.
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
    /// Declares `spelling` in the current rib.
    ///
    /// SRC-0003: the binding is keyed by the NFC canonical form, so a name
    /// declared as `e` + COMBINING ACUTE ACUTE collides with one declared as
    /// the precomposed `e` + ACUTE. The original spelling is retained in
    /// `ResolvedNames::names` and in the rib, and is what a diagnostic reports.
    pub fn declare(&mut self, name: &str) -> Result<DefId, ResolveError> {
        let canonical = CanonicalName::new(name)
            .map_err(|_| ResolveError::ProhibitedIdentifier { name: name.to_string() })?;
        let key = canonical.key().to_string();
        if let Some(existing) = self.ribs.iter().rev().find_map(|r| r.bindings.get(&key).copied()) {
            return Err(ResolveError::ShadowingViolation {
                name: name.to_string(),
                existing,
                span: None,
            });
        }
        let id = DefId::new(self.current_package, self.current_module, self.next_index);
        self.next_index += 1;
        let rib = self.ribs.last_mut().expect("global rib");
        rib.spellings.insert(key.clone(), name.to_string());
        rib.bindings.insert(key, id);
        Ok(id)
    }

    /// Resolves a reference by its source spelling.
    ///
    /// SRC-0003: lookup is by canonical key, so a reference may use a
    /// different but canonically equivalent spelling from its declaration.
    pub fn resolve(&self, name: &str) -> Result<DefId, ResolveError> {
        let canonical = CanonicalName::new(name)
            .map_err(|_| ResolveError::ProhibitedIdentifier { name: name.to_string() })?;
        self.ribs
            .iter()
            .rev()
            .find_map(|r| r.bindings.get(canonical.key()).copied())
            .ok_or_else(|| ResolveError::UnresolvedName { name: name.into(), span: None })
    }

    /// The original spelling recorded for a binding in the global rib, if any.
    pub fn spelling_of(&self, key: &str) -> Option<&str> {
        self.ribs.iter().rev().find_map(|r| r.spellings.get(key).map(String::as_str))
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
            SyntaxKind::ConstDef | SyntaxKind::StaticDef => {
                for c in node.children() {
                    if is_expression_node(c.kind()) {
                        self.resolve_node(&c, out, errors);
                    }
                }
            }
            SyntaxKind::ModuleDecl => {
                for c in node.children() {
                    match c.kind() {
                        SyntaxKind::FnDef => self.resolve_fn(c, out, errors),
                        SyntaxKind::ConstDef | SyntaxKind::StaticDef => self.resolve_node(&c, out, errors),
                        _ => {}
                    }
                }
            }
            SyntaxKind::StructDef
            | SyntaxKind::EnumDef
            | SyntaxKind::TraitDef
            | SyntaxKind::ImplDef
            | SyntaxKind::TypeAlias
            | SyntaxKind::UseDecl
            | SyntaxKind::ExternCrateDecl
            | SyntaxKind::Attribute
            | SyntaxKind::GenericParams
            | SyntaxKind::GenericParam
            | SyntaxKind::Type
            | SyntaxKind::Path
            | SyntaxKind::PathSegment
            | SyntaxKind::TypeBound
            | SyntaxKind::TraitRef
            | SyntaxKind::TypeArgs
            | SyntaxKind::Lifetime
            | SyntaxKind::LifetimeArg
            | SyntaxKind::WhereClause
            | SyntaxKind::WherePredicate => {}
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
        match self.declare(&name) {
            Ok(id) => {
                // SRC-0003: record the *source spelling*, not the NFC key, so a
                // diagnostic about this definition quotes what the file said.
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

fn is_expression_node(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ExprStmt
            | SyntaxKind::FinalExpr
            | SyntaxKind::ReturnExpr
            | SyntaxKind::BreakExpr
            | SyntaxKind::ContinueExpr
            | SyntaxKind::YieldExpr
            | SyntaxKind::BinaryExpr
            | SyntaxKind::AssignExpr
            | SyntaxKind::RangeExpr
            | SyntaxKind::UnaryExpr
            | SyntaxKind::CallExpr
            | SyntaxKind::MethodCallExpr
            | SyntaxKind::FieldExpr
            | SyntaxKind::IndexExpr
            | SyntaxKind::CastExpr
            | SyntaxKind::LiteralExpr
            | SyntaxKind::PathExpr
            | SyntaxKind::MacroInvocation
            | SyntaxKind::ParenthesizedExpr
            | SyntaxKind::StructExpr
            | SyntaxKind::ArrayExpr
            | SyntaxKind::TupleExpr
            | SyntaxKind::ClosureExpr
            | SyntaxKind::AsyncBlock
            | SyntaxKind::UnsafeBlock
            | SyntaxKind::TryBlock
            | SyntaxKind::TryExpr
            | SyntaxKind::IfExpr
            | SyntaxKind::MatchExpr
            | SyntaxKind::LoopExpr
            | SyntaxKind::WhileExpr
            | SyntaxKind::ForExpr
            | SyntaxKind::AwaitExpr
            | SyntaxKind::Block
    )
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
