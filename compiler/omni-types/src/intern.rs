use std::collections::HashMap;

/// A lightweight, O(1) comparable handle to an interned type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ty(pub u32);

/// The base kinds of types in the Omni language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyKind {
    Error,
    Int,
    Float,
    Bool,
    // We will expand this with Structs, Enums, and Functions as the grammar grows.
}

/// The Type Context (Arena) responsible for interning types.
/// It ensures two identical TyKinds map to the exact same Ty index.
#[derive(Debug, Default)]
pub struct TyCtxt {
    dedup: HashMap<TyKind, Ty>,
    arena: Vec<TyKind>,
}

impl TyCtxt {
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns a TyKind and returns its O(1) Ty handle.
    pub fn intern(&mut self, kind: TyKind) -> Ty {
        // If we've seen this type before, return its existing index.
        if let Some(&ty) = self.dedup.get(&kind) {
            return ty;
        }

        // Otherwise, mint a new index, store it, and return it.
        let index = self.arena.len() as u32;
        let ty = Ty(index);
        
        self.dedup.insert(kind.clone(), ty);
        self.arena.push(kind);
        
        ty
    }

    /// Retrieves the actual TyKind structure for a given Ty handle.
    pub fn get(&self, ty: Ty) -> &TyKind {
        &self.arena[ty.0 as usize]
    }
}
