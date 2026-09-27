use index_vec::{define_index_type, IndexVec};
use omni_types::ast::{Lit, TypeSpec};
use omni_types::intern::Ty;

// Strongly-typed indices to prevent array-lookup mixups.
define_index_type! { pub struct BasicBlock = u32; }
define_index_type! { pub struct Local = u32; }

/// Whole-program MIR container holding monomorphized function MIR definitions.
#[derive(Debug, Clone)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
}

/// Monomorphized function represented in canonical Mid-Level IR.
#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<Local>,
    pub return_place: Local,
    pub return_type: TypeSpec,
    pub body: Body,
}

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

/// Metadata about a local variable (type, name, and index).
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub name: Option<String>,
    pub ty: Option<Ty>,
}

/// Unary operations supported in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

/// Binary arithmetic and logical operations supported in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
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
    /// Conditional branch based on an integer/boolean value.
    SwitchInt { discr: Operand, targets: Vec<(u64, BasicBlock)>, otherwise: BasicBlock },
    /// Invoke a function and branch based on success/unwind.
    /// A None destination represents a Unit-returning call.
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Option<Place>,
        target: BasicBlock,
        cleanup: Option<BasicBlock>,
    },
    /// Return to the caller.
    Return,
    /// Unreachable control flow path.
    Unreachable,
}

// --- Supporting Types ---

/// A location in memory (e.g., local, field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub local: Local,
}

/// A value produced by an operation.
#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
}

/// A literal scalar value in MIR.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Lit(Lit),
    FnRef(String),
}

/// A value consumed by an operation.
#[derive(Debug, Clone)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

/// A formal assumption for the mechanical verifier.
#[derive(Debug, Clone)]
pub struct Assumption {}
