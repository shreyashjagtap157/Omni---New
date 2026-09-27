use crate::intern::Ty;

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
    Known(Ty),
}

/// Literals supported in AST expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    Eq,
    Ne,
    Lt,
    Gt,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

/// Patterns used in match arms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Wildcard,
    Binding(String),
    Lit(Lit),
}

/// AST expression representation covering expressions required for monomorphization.
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
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Range { start: Box<Expr>, end: Box<Expr> },
    Match { expr: Box<Expr>, arms: Vec<(Pattern, Expr)> },
    Lambda { params: Vec<(String, TypeSpec)>, body: Box<Expr> },
    Interpolation(Vec<Expr>),
    Assign { target: Box<Expr>, value: Box<Expr> },
    Block(Vec<Expr>),
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
}

/// A trait definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDef {
    pub name: String,
    pub supertraits: Vec<String>,
    pub methods: Vec<MethodSig>,
    pub is_local: bool,
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

/// Generic function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericFnDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub bounds: Vec<(String, TraitBound)>,
    pub params: Vec<(String, TypeSpec)>,
    pub return_type: TypeSpec,
    pub body: Expr,
}
