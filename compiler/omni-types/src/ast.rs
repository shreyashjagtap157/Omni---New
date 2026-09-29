use crate::intern::Ty;
use omni_effects::{Capability, EffectRow};

/// Declarative type representation used in generic AST definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    Int,
    Float,
    Bool,
    Char,
    Byte,
    String,
    Unit,
    GenericParam(String),
    Tuple(Vec<TypeSpec>),
    Array(Box<TypeSpec>, usize),
    Range(Box<TypeSpec>),
    Fn(Vec<TypeSpec>, Box<TypeSpec>),
    Struct(String, Vec<TypeSpec>),
    Enum(String, Vec<TypeSpec>),
    Never,
    Known(Ty),
}

/// Literals supported in AST expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lit {
    Int(i64),
    Float(u64),
    Bool(bool),
    Char(char),
    Byte(u8),
    String(String),
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    LogicalAnd,
    LogicalOr,
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

/// Unary operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
}

/// Range boundaries for pattern matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternRangeBoundary {
    Inclusive(Lit),
    Exclusive(Lit),
    Unbounded,
}

/// Patterns used in match arms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Wildcard,
    Binding(String),
    Lit(Lit),
    Tuple(Vec<Pattern>),
    Struct { name: String, fields: Vec<(String, Pattern)> },
    Variant { enum_name: String, variant: String, subpatterns: Vec<Pattern> },
    Range { start: PatternRangeBoundary, end: PatternRangeBoundary },
    Or(Vec<Pattern>),
    Never,
}

/// Match arm carrying pattern, optional guard expression, and body expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Assignment operators preserved by the semantic AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Literal(Lit),
    Var(String),
    Call { func: String, generic_args: Vec<TypeSpec>, args: Vec<Expr> },
    Let { name: String, ty: Option<TypeSpec>, init: Box<Expr>, body: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Unary { op: UnOp, expr: Box<Expr> },
    Field { expr: Box<Expr>, field: String },
    Index { expr: Box<Expr>, index: Box<Expr> },
    Struct {
        name: String,
        generic_args: Vec<TypeSpec>,
        fields: Vec<(String, Expr)>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        generic_args: Vec<TypeSpec>,
        args: Vec<Expr>,
    },
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Range { start: Box<Expr>, end: Box<Expr>, inclusive: bool },
    Cast { expr: Box<Expr>, ty: TypeSpec },
    Match { expr: Box<Expr>, arms: Vec<MatchArm> },
    If { condition: Box<Expr>, then_branch: Box<Expr>, else_branch: Option<Box<Expr>> },
    Lambda { params: Vec<(String, TypeSpec)>, body: Box<Expr> },
    Interpolation(Vec<Expr>),
    Assign { target: Box<Expr>, value: Box<Expr> },
    CompoundAssign { op: AssignOp, target: Box<Expr>, value: Box<Expr> },
    Block(Vec<Expr>),
    Loop { label: Option<String>, body: Box<Expr> },
    While { label: Option<String>, condition: Box<Expr>, body: Box<Expr> },
    For { label: Option<String>, pattern: Pattern, iterable: Box<Expr>, body: Box<Expr> },
    Break { label: Option<String>, value: Option<Box<Expr>> },
    Continue { label: Option<String> },
    Return(Option<Box<Expr>>),
}

/// Generic bound on a type parameter (positive or negative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitBound {
    Positive(String), // e.g. T: Debug
    Negative(String), // e.g. T: !Debug
}

/// A method signature inside a trait or impl definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSig {
    pub name: String,
    pub params: Vec<(String, TypeSpec)>,
    pub return_type: TypeSpec,
    pub has_default: bool,
    pub effects: EffectRow,
    pub capabilities: Vec<Capability>,
}

/// A trait definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDef {
    pub name: String,
    pub supertraits: Vec<String>,
    pub methods: Vec<MethodSig>,
    pub is_local: bool,
}

/// A declared field in a source struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldDef {
    pub name: String,
    pub ty: TypeSpec,
}

/// A source struct declaration used by semantic field lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<StructFieldDef>,
}

/// A trait implementation block (`impl Trait for Type`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplDef {
    pub trait_name: String,
    pub target_ty: TypeSpec,
    pub conditions: Vec<(String, TraitBound)>,
    pub methods: Vec<(String, Expr)>,
    pub is_local: bool,
}

/// Enum variant payload definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantDef {
    pub name: String,
    pub payload: Vec<TypeSpec>,
}

/// ADT Enum definition (e.g. Option[T], Result[T, E], or custom user enums).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<EnumVariantDef>,
}

/// Generic function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericFnDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub bounds: Vec<(String, TraitBound)>,
    pub params: Vec<(String, TypeSpec)>,
    pub return_type: TypeSpec,
    pub effects: EffectRow,
    pub capabilities: Vec<Capability>,
    pub body: Expr,
}
