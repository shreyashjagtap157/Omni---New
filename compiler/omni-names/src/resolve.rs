use crate::def_id::DefId;
use std::collections::HashMap;

/// A Rib represents a single lexical scope (e.g., a block, a function body).
#[derive(Debug, Default, Clone)]
pub struct Rib {
    pub bindings: HashMap<String, DefId>,
}

/// Errors that can occur during name resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// Thrown when a name is declared that already exists in the current or parent scopes.
    ShadowingViolation { name: String, existing: DefId },
    /// Thrown when an identifier is used but was never declared.
    UnresolvedName { name: String },
}

/// The Multi-Pass Resolver manages lexical scopes and maps string names to canonical DefIds.
pub struct Resolver {
    ribs: Vec<Rib>,
    next_index: u32,
    current_package: u32,
    current_module: u32,
}

impl Resolver {
    pub fn new(package: u32, module: u32) -> Self {
        Self {
            ribs: vec![Rib::default()], // Always start with a global/module rib
            next_index: 0,
            current_package: package,
            current_module: module,
        }
    }

    /// Enter a new lexical scope.
    pub fn push_rib(&mut self) {
        self.ribs.push(Rib::default());
    }

    /// Exit the current lexical scope.
    pub fn pop_rib(&mut self) {
        self.ribs.pop().expect("Cannot pop the global module rib");
    }

    /// Declares a new name in the current scope, assigning it a DefId.
    /// Traps shadowing violations (NAME-0006).
    pub fn declare(&mut self, name: String) -> Result<DefId, ResolveError> {
        // NAME-0006: Strictly prohibit shadowing across all active scopes
        for rib in &self.ribs {
            if let Some(&existing) = rib.bindings.get(&name) {
                return Err(ResolveError::ShadowingViolation { name, existing });
            }
        }

        let id = DefId::new(self.current_package, self.current_module, self.next_index);
        self.next_index += 1;

        self.ribs.last_mut().unwrap().bindings.insert(name, id);
        Ok(id)
    }

    /// Resolves a name to its canonical DefId by walking up the Rib stack.
    pub fn resolve(&self, name: &str) -> Result<DefId, ResolveError> {
        for rib in self.ribs.iter().rev() {
            if let Some(&id) = rib.bindings.get(name) {
                return Ok(id);
            }
        }

        Err(ResolveError::UnresolvedName { name: name.to_string() })
    }
}

#[cfg(any())]
#[implements("NAME-0006")]
fn _audit_shadowing_rules() {}
