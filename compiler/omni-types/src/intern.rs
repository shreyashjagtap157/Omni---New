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
    Char,
    Byte,
    String,
    Unit,
    GenericParam(String),
    Tuple(Vec<Ty>),
    Array(Ty, usize),
    Range(Ty),
    Fn(Vec<Ty>, Ty),
    Struct(String, Vec<Ty>),
    Enum(String, Vec<Ty>),
    Never,
    Infer(u32),
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
        if let Some(&ty) = self.dedup.get(&kind) {
            return ty;
        }

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

    /// Produces a deterministic, collision-free mangled representation of a type.
    pub fn mangle(&self, ty: Ty) -> String {
        match self.get(ty) {
            TyKind::Error => "error".to_string(),
            TyKind::Int => "i64".to_string(),
            TyKind::Float => "f64".to_string(),
            TyKind::Bool => "bool".to_string(),
            TyKind::Char => "char".to_string(),
            TyKind::Byte => "u8".to_string(),
            TyKind::String => "String".to_string(),
            TyKind::Unit => "unit".to_string(),
            TyKind::GenericParam(name) => format!("param_{}_{name}", name.len()),
            TyKind::Tuple(tys) => {
                let parts: Vec<String> = tys.iter().map(|&t| self.mangle(t)).collect();
                format!("tuple_{}_{}_end", tys.len(), parts.join("_"))
            }
            TyKind::Array(elem, len) => format!("arr_{len}_{}_end", self.mangle(*elem)),
            TyKind::Range(elem) => format!("range_{}_end", self.mangle(*elem)),
            TyKind::Fn(params, ret) => {
                let p: Vec<String> = params.iter().map(|&t| self.mangle(t)).collect();
                format!("fn_{}_{}_ret_{}_end", params.len(), p.join("_"), self.mangle(*ret))
            }
            TyKind::Struct(name, args) => {
                let a: Vec<String> = args.iter().map(|&t| self.mangle(t)).collect();
                format!("struct_{}_{name}_{}_{}_end", name.len(), args.len(), a.join("_"))
            }
            TyKind::Enum(name, args) => {
                let a: Vec<String> = args.iter().map(|&t| self.mangle(t)).collect();
                format!("enum_{}_{name}_{}_{}_end", name.len(), args.len(), a.join("_"))
            }
            TyKind::Never => "never".to_string(),
            TyKind::Infer(id) => format!("var_{id}"),
        }
    }

    /// Checks if a type contains no unresolved generic parameters or type variables.
    pub fn is_concrete(&self, ty: Ty) -> bool {
        match self.get(ty) {
            TyKind::Error | TyKind::GenericParam(_) | TyKind::Infer(_) => false,
            TyKind::Int
            | TyKind::Float
            | TyKind::Bool
            | TyKind::Char
            | TyKind::Byte
            | TyKind::String
            | TyKind::Unit
            | TyKind::Never => true,
            TyKind::Tuple(tys) => tys.iter().all(|&t| self.is_concrete(t)),
            TyKind::Array(elem, _) | TyKind::Range(elem) => self.is_concrete(*elem),
            TyKind::Fn(params, ret) => {
                params.iter().all(|&t| self.is_concrete(t)) && self.is_concrete(*ret)
            }
            TyKind::Struct(_, args) | TyKind::Enum(_, args) => {
                args.iter().all(|&t| self.is_concrete(t))
            }
        }
    }
}
