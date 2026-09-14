use index_vec::{define_index_type, IndexVec};

// Strongly-typed indices to prevent array-lookup mixups.
define_index_type! { pub struct BasicBlock = u32; }
define_index_type! { pub struct Local = u32; }

/// The entire MIR control-flow graph for a single function.
#[derive(Debug, Clone)]
pub struct Body {
    pub blocks: IndexVec<BasicBlock, BlockData>,
    pub local_decls: IndexVec<Local, LocalDecl>,
}

/// A sequence of non-branching statements ending in a single terminator.
#[derive(Debug, Clone)]
pub struct BlockData {
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
}

/// Metadata about a local variable (e.g., type, mutability).
#[derive(Debug, Clone)]
pub struct LocalDecl {
    // TODO: Link to omni-types::Ty once we wire the crates together
}

/// A discrete action within a basic block.
#[derive(Debug, Clone)]
pub enum Statement {
    /// Writes an rvalue into a memory place.
    Assign(Place, Rvalue),
    /// A mechanical verifier directive (e.g., bounds check assumption).
    Assume(Assumption),
    /// Explicitly invalidates a place, running its destructor if necessary.
    Drop(Place),
}

/// The branching instruction at the end of every basic block.
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Jump unconditionally to another block.
    Goto(BasicBlock),
    /// Invoke a function and branch based on success/unwind.
    Call {
        func: Operand,
        args: Vec<Operand>,
        target: BasicBlock,
        cleanup: Option<BasicBlock>,
    },
    /// Return to the caller.
    Return,
}

// --- Supporting Types ---

/// A location in memory (e.g., x, x.y, *x).
#[derive(Debug, Clone)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]`npub struct Place {
    pub local: Local,
}

/// A value produced by an operation (e.g.,  + b, &x).
#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
}

/// A value consumed by an operation.
#[derive(Debug, Clone)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant, // To be expanded
}

/// A formal assumption for the mechanical verifier.
#[derive(Debug, Clone)]
pub struct Assumption {}

