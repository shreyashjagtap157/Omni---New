//! # Unsafe Assumption Tokens
//! Implements assumption tokens for unsafe verification blocks.
//! No rule ownership is claimed here: the referenced identifier was never a
//! registered rule, and prose must not mimic linkage syntax.

use crate::ir::Place;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssumptionId(pub u32);

/// Represents an unsafe assumption token verified by the mechanical verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assumption {
    pub id: AssumptionId,
    pub obligation: String,
    pub deps: Vec<Place>,
}

impl Assumption {
    pub fn new(id: AssumptionId, obligation: impl Into<String>, deps: Vec<Place>) -> Self {
        Self {
            id,
            obligation: obligation.into(),
            deps,
        }
    }
}
