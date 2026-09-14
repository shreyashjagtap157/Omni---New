//! Type system, interning arena, and constraint solving for Omni.

pub mod intern;
pub mod solver;

pub use intern::{Ty, TyCtxt, TyKind};
pub use solver::{Solver, TyVar, TyVarValue};
