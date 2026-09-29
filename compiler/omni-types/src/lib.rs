//! Type system, interning arena, constraint solving, and monomorphization engine for Omni.

pub mod ast;
pub mod checker;
pub mod intern;
pub mod monomorph;
pub mod pattern;
pub mod solver;

#[cfg(test)]
mod tests;

pub use ast::{
    AssignOp, BinOp, EnumDef, EnumVariantDef, Expr, GenericFnDef, ImplDef, Lit, MatchArm, MethodSig, Pattern,
    PatternRangeBoundary, TraitBound, TraitDef, TypeSpec, UnOp,
};
pub use checker::{SpecializationKey, SubstEnv, TraitObligationChecker, TypeChecker, TypeError};
pub use intern::{Ty, TyCtxt, TyKind};
pub use monomorph::{MonomorphizedProgram, Monomorphizer};
pub use pattern::{Constructor, MatchAnalysisResult, PatternChecker};
pub use solver::{Solver, TyVar, TyVarValue};

pub use omni_effects;
