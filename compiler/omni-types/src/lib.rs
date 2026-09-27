//! Type system, interning arena, constraint solving, and monomorphization engine for Omni.

pub mod ast;
pub mod checker;
pub mod intern;
pub mod monomorph;
pub mod solver;

#[cfg(test)]
mod tests;

pub use ast::{
    BinOp, Expr, GenericFnDef, ImplDef, Lit, MethodSig, Pattern, TraitBound, TraitDef, TypeSpec,
    UnOp,
};
pub use checker::{SpecializationKey, SubstEnv, TraitObligationChecker, TypeChecker, TypeError};
pub use intern::{Ty, TyCtxt, TyKind};
pub use monomorph::{MonomorphizedProgram, Monomorphizer};
pub use solver::{Solver, TyVar, TyVarValue};
