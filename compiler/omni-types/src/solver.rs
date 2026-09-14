use ena::unify::{InPlaceUnificationTable, UnifyKey, UnifyValue};
use crate::intern::Ty;

/// A Type Variable, used during inference before the concrete type is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TyVar(pub u32);

impl UnifyKey for TyVar {
    type Value = TyVarValue;
    
    fn index(&self) -> u32 {
        self.0
    }
    
    fn from_index(u: u32) -> Self {
        TyVar(u)
    }
    
    fn tag() -> &'static str {
        "TyVar"
    }
}

/// Errors that can occur during type unification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeError {
    Conflict,
}

/// The value bound to a Type Variable. 
/// It can be known (a specific interned Ty) or unknown (None).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TyVarValue(pub Option<Ty>);

impl UnifyValue for TyVarValue {
    type Error = TypeError;
    
    fn unify_values(val1: &Self, val2: &Self) -> Result<Self, Self::Error> {
        match (val1.0, val2.0) {
            (Some(t1), Some(t2)) if t1 == t2 => Ok(TyVarValue(Some(t1))),
            (Some(_), Some(_)) => Err(TypeError::Conflict),
            (Some(t), None) | (None, Some(t)) => Ok(TyVarValue(Some(t))),
            (None, None) => Ok(TyVarValue(None)),
        }
    }
}

/// The Type Inference Solver using the Union-Find algorithm.
pub struct Solver {
    table: InPlaceUnificationTable<TyVar>,
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver {
    pub fn new() -> Self {
        Self {
            table: InPlaceUnificationTable::new(),
        }
    }

    /// Creates a fresh, unknown type variable (e.g., ?0).
    pub fn new_ty_var(&mut self) -> TyVar {
        self.table.new_key(TyVarValue(None))
    }

    /// Equates two type variables.
    pub fn equate_vars(&mut self, a: TyVar, b: TyVar) -> Result<(), TypeError> {
        self.table.unify_var_var(a, b)
    }

    /// Equates a type variable with a concrete interned type.
    pub fn equate_var_ty(&mut self, var: TyVar, ty: Ty) -> Result<(), TypeError> {
        self.table.unify_var_value(var, TyVarValue(Some(ty)))
    }

    /// Probes the solver table to find the concrete type bound to a variable, if any.
    pub fn probe(&mut self, var: TyVar) -> Option<Ty> {
        self.table.probe_value(var).0
    }
}
