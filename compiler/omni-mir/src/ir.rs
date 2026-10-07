use index_vec::{define_index_type, IndexVec};
use omni_types::ast::{Lit, TypeSpec};
use omni_types::intern::Ty;

// Strongly-typed indices to prevent array-lookup mixups.
define_index_type! { pub struct BasicBlock = u32; }
define_index_type! { pub struct Local = u32; }

/// Whole-program MIR container holding monomorphized function MIR definitions.
#[derive(Debug, Clone)]
pub struct MirProgram {
    /// Type context that owns every Ty handle used by this MIR program.
    pub tcx: omni_types::intern::TyCtxt,
    pub functions: Vec<MirFunction>,
    /// Struct declarations needed to resolve named field projections.
    ///
    /// A place may carry `Projection::Field`, and re-deriving that field's type
    /// requires the declaration. Carrying it in the program means the verifier
    /// sees the same definitions lowering used, rather than the two disagreeing
    /// about what a struct contains. It is empty when no struct is declared.
    pub struct_defs: std::collections::HashMap<String, omni_types::ast::StructDef>,
}

impl MirProgram {
    /// Creates a program with no struct declarations.
    ///
    /// Struct-field projection checking needs the declarations; a program that
    /// declares none can only contain aggregate-free places, which this
    /// constructor expresses without each call site having to supply an empty
    /// map.
    pub fn new(tcx: omni_types::intern::TyCtxt, functions: Vec<MirFunction>) -> Self {
        Self { tcx, functions, struct_defs: std::collections::HashMap::new() }
    }

    /// Attaches struct declarations, returning the program for chaining.
    pub fn with_struct_defs(
        mut self,
        defs: std::collections::HashMap<String, omni_types::ast::StructDef>,
    ) -> Self {
        self.struct_defs = defs;
        self
    }
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
    /// Basic blocks that were entered from inside an `unsafe` block or function.
    ///
    /// UNSAFE-0002 requires that an unsafe operation's effect "is visible in MIR
    /// and audit reports". Recording the blocks rather than a single boolean is
    /// deliberate: `unsafe` scopes a *region*, so an audit must be able to say
    /// which operations were covered by a programmer's proof and which were not.
    /// A function-wide flag could not distinguish `unsafe { raw() }` from `raw()`
    /// sitting next to it, and would silently widen every future check.
    ///
    /// This is metadata only. It grants nothing: nothing in verification,
    /// execution, or codegen consults it yet, because no raw-memory operation
    /// exists to require an unsafe context (UNSAFE-0002). It is carried so the
    /// context is not lost before such an operation is added.
    pub unsafe_blocks: Vec<BasicBlock>,
}

impl Default for Body {
    /// An empty body: no blocks, no locals, and no unsafe region.
    ///
    /// `unsafe_blocks` defaults to empty, which is the correct default rather
    /// than merely the convenient one: an empty body contains no operation, so
    /// it is trivially outside every unsafe region. A default that widened the
    /// region would silently mark hand-built bodies as programmer-proofed.
    fn default() -> Self {
        Self { blocks: IndexVec::new(), local_decls: IndexVec::new(), unsafe_blocks: Vec::new() }
    }
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
    BitNot,
}

/// Binary arithmetic and logical operations supported in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
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
    /// Runtime bounds check for dynamic array indexing.
    ///
    /// Traps if `index >= 0 && index < length` is false. Otherwise continues execution.
    /// This is a first-class safety operation, not a codegen convention.
    BoundsCheck { index: Local, length: usize },
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

/// A projection component identifying a subplace within an aggregate or reference.
///
/// The vocabulary is derived from the types the existing checker, ownership
/// engine, and MIR lowering already agree on, rather than being invented
/// independently. `Field` covers struct/enum named fields, `ConstantIndex`
/// covers tuple positions and constant array subscripts, `Index` covers a
/// runtime-computed subscript, and `Deref` covers reference indirection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Projection {
    /// Named field projection.
    Field(String),
    /// Constant tuple/array index projection.
    ConstantIndex(usize),
    /// Subscript computed at runtime from the given local's value.
    ///
    /// The index is kept as a local rather than a folded constant so the
    /// verifier can still check the bounds of the access against the aggregate's
    /// static length.
    Index(Local),
    /// Dereference through a reference place.
    Deref,
}

impl Projection {
    /// Renders this projection in source-like form for diagnostics.
    pub fn display(&self) -> String {
        match self {
            Projection::Field(name) => format!(".{name}"),
            Projection::ConstantIndex(index) => format!("[{index}]"),
            Projection::Index(local) => format!("[{:?}]", local.index()),
            Projection::Deref => ".*".to_string(),
        }
    }
}

/// A location in memory: a local plus an ordered projection chain.
///
/// This is the single place identity shared by MIR lowering, MIR verification,
/// ownership, and definite initialization. Previously MIR carried only the
/// local, so a projection such as `p.x` or `a[i]` could be *read* as an
/// `Rvalue::Field`/`Rvalue::Index` but could never be an assignment target,
/// because the destination carried no projection. The projection chain makes
/// the destination expressible, so assignment through a projection is
/// represented rather than being folded back onto the root local.
///
/// A place is canonical: the chain is ordered from the root outward, and
/// equality/ordering over `Place` is defined by the chain so that one place has
/// exactly one identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    /// The root local this place is derived from.
    pub local: Local,
    /// Ordered projections applied to the root, outermost last.
    pub projections: Vec<Projection>,
}

impl Place {
    /// Creates a place referring to a whole local, with no projections.
    pub fn local(local: Local) -> Self {
        Self { local, projections: Vec::new() }
    }

    /// Creates a projected child place.
    pub fn project(&self, projection: Projection) -> Self {
        let mut next = self.clone();
        next.projections.push(projection);
        next
    }

    /// Returns the same place with `projections` truncated to `len` components.
    pub fn truncate(&self, len: usize) -> Self {
        Self { local: self.local, projections: self.projections[..len].to_vec() }
    }

    /// Returns the place one step above this one, or `None` if already a local.
    pub fn parent(&self) -> Option<Self> {
        self.projections
            .is_empty()
            .then(|| Self::local(self.local))
            .or_else(|| self.projections.len().checked_sub(1).map(|len| self.truncate(len)))
    }

    /// True when this place refers to a whole local with no projection.
    pub fn is_local(&self) -> bool {
        self.projections.is_empty()
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_{:?}", self.local.index())?;
        for projection in &self.projections {
            write!(f, "{}", projection.display())?;
        }
        Ok(())
    }
}

/// A value produced by an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateKind {
    Tuple,
    Array,
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Cast {
        operand: Operand,
        from: Ty,
        to: Ty,
    },
    Aggregate {
        kind: AggregateKind,
        operands: Vec<Operand>,
        ty: Ty,
    },
    Struct {
        name: String,
        fields: Vec<(String, Operand)>,
        ty: Ty,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        operands: Vec<Operand>,
        ty: Ty,
    },
    /// Reifies a borrow of a place as a typed logical reference.
    ///
    /// Native representation is a later ABI concern; MIR keeps the semantic
    /// place, mutability, and resulting reference type explicit.
    Reference {
        place: Place,
        mutable: bool,
        ty: Ty,
    },
    Range {
        start: Operand,
        end: Operand,
        inclusive: bool,
        ty: Ty,
    },
    Field {
        base: Operand,
        field: String,
        ty: Ty,
    },
    Index {
        base: Operand,
        index: Operand,
        ty: Ty,
    },
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

/// The verified unsafe-assumption token is defined in the dedicated assumption module.
pub use crate::assume::{Assumption, AssumptionId};
