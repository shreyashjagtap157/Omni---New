//! Mid-Level Intermediate Representation for Omni.

pub mod ir;
pub mod lower;

pub mod continuation;

pub mod concurrency;

pub use omni_types::ast;
pub use omni_types::intern::{Ty, TyCtxt, TyKind};
pub use omni_types::monomorph::MonomorphizedProgram;
