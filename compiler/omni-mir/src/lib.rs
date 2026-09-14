//! Mid-level Intermediate Representation (MIR) for Omni.
//! A flat, control-flow graph representation used for borrow checking and optimization.

pub mod ir;
pub mod drop;

pub use ir::*;
pub use drop::DropElaborator;
