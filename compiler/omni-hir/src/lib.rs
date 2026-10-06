//! High-Level Intermediate Representation (HIR) for Omni.
//!
//! HIR is the first canonical semantic representation after front-end analysis.
//! It sits between the syntax-oriented AST and MIR: generic parameters have
//! been materialized, method calls have been eliminated by monomorphization,
//! and function signatures carry canonical interned concrete type handles.
//! Declarative syntax is retained only as a bridge for the existing MIR builder.

use std::collections::{HashMap, HashSet};

use omni_effects::{Capability, EffectRow};
use omni_types::ast::{EnumDef, Expr, GenericFnDef, StructDef, TypeSpec};
use omni_types::{MonomorphizedProgram, Ty, TyCtxt};

/// A semantically lowered Omni function.
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    /// Declarative signature retained for the current MIR bridge.
    pub params: Vec<(String, TypeSpec)>,
    /// Canonical interned parameter types owned by this HIR program's context.
    pub param_tys: Vec<Ty>,
    /// Declarative return type retained for the current MIR bridge.
    pub return_type: TypeSpec,
    /// Canonical interned return type.
    pub return_ty: Ty,
    pub effects: EffectRow,
    pub capabilities: Vec<Capability>,
    /// Monomorphized semantic expression tree.
    pub body: Expr,
}

/// Program-wide canonical HIR.
#[derive(Debug, Clone)]
pub struct HirProgram {
    pub tcx: TyCtxt,
    pub functions: Vec<HirFunction>,
    pub struct_defs: HashMap<String, StructDef>,
    pub enum_defs: HashMap<String, EnumDef>,
}

impl HirProgram {
    /// Construct HIR from a concrete monomorphized program.
    ///
    /// The concrete-program gate is enforced here so downstream MIR consumers
    /// receive a representation with no unresolved type parameters or generic
    /// call arguments. Function ordering is canonicalized by name.
    pub fn from_monomorphized(
        program: &MonomorphizedProgram,
        struct_defs: HashMap<String, StructDef>,
        enum_defs: HashMap<String, EnumDef>,
    ) -> Result<Self, String> {
        program.assert_concrete_for_mir()?;

        let mut tcx = TyCtxt::new();
        let env = omni_types::SubstEnv::new();
        let mut functions = Vec::with_capacity(program.functions.len());
        let mut seen = HashSet::new();

        for function in &program.functions {
            if !seen.insert(function.name.clone()) {
                return Err(format!(
                    "HIR construction error: duplicate function '{}'",
                    function.name
                ));
            }

            let mut params = Vec::with_capacity(function.params.len());
            let mut param_tys = Vec::with_capacity(function.params.len());
            for (name, spec) in &function.params {
                let ty = tcx.lower_type_spec(spec, &env);
                if !tcx.is_concrete(ty) {
                    return Err(format!(
                        "HIR construction error: parameter '{}' of '{}' is not concrete",
                        name, function.name
                    ));
                }
                params.push((name.clone(), spec.clone()));
                param_tys.push(ty);
            }

            let return_ty = tcx.lower_type_spec(&function.return_type, &env);
            if !tcx.is_concrete(return_ty) {
                return Err(format!(
                    "HIR construction error: return type of '{}' is not concrete",
                    function.name
                ));
            }

            validate_concrete_expr(&function.body, &function.name)?;

            functions.push(HirFunction {
                name: function.name.clone(),
                params,
                param_tys,
                return_type: function.return_type.clone(),
                return_ty,
                effects: function.effects.clone(),
                capabilities: function.capabilities.clone(),
                body: function.body.clone(),
            });
        }

        functions.sort_by(|a, b| a.name.cmp(&b.name));

        let hir = Self { tcx, functions, struct_defs, enum_defs };
        hir.validate()?;
        Ok(hir)
    }

    /// Compatibility name used by the current driver/frontend integration.
    pub fn from_monomorphized_program(
        program: &MonomorphizedProgram,
        struct_defs: HashMap<String, StructDef>,
    ) -> Result<Self, String> {
        Self::from_monomorphized(program, struct_defs, HashMap::new())
    }

    pub fn from_monomorphized_without_defs(program: &MonomorphizedProgram) -> Result<Self, String> {
        Self::from_monomorphized(program, HashMap::new(), HashMap::new())
    }

    /// Validate the canonical HIR invariants required by MIR.
    pub fn validate(&self) -> Result<(), String> {
        let mut names = HashSet::new();
        for function in &self.functions {
            if !names.insert(&function.name) {
                return Err(format!(
                    "HIR validation error: duplicate function '{}'",
                    function.name
                ));
            }
            if function.params.len() != function.param_tys.len() {
                return Err(format!(
                    "HIR validation error: '{}' has mismatched parameter metadata",
                    function.name
                ));
            }
            if !self.tcx.is_concrete(function.return_ty) {
                return Err(format!(
                    "HIR validation error: return type of '{}' is not concrete",
                    function.name
                ));
            }
            for (index, ty) in function.param_tys.iter().copied().enumerate() {
                if !self.tcx.is_concrete(ty) {
                    return Err(format!(
                        "HIR validation error: parameter {} of '{}' is not concrete",
                        index, function.name
                    ));
                }
            }
            validate_concrete_expr(&function.body, &function.name)?;
        }
        Ok(())
    }

    /// Compatibility bridge used by the current MIR lowering engine.
    pub fn to_monomorphized_program(&self) -> MonomorphizedProgram {
        MonomorphizedProgram {
            functions: self
                .functions
                .iter()
                .map(|function| GenericFnDef {
                    name: function.name.clone(),
                    type_params: Vec::new(),
                    bounds: Vec::new(),
                    params: function.params.clone(),
                    return_type: function.return_type.clone(),
                    effects: function.effects.clone(),
                    capabilities: function.capabilities.clone(),
                    body: function.body.clone(),
                })
                .collect(),
        }
    }
}

fn validate_concrete_expr(expr: &Expr, function_name: &str) -> Result<(), String> {
    if contains_unresolved_syntax(expr) {
        return Err(format!(
            "HIR validation error: '{}' retains generic call arguments or an unresolved method call",
            function_name
        ));
    }
    Ok(())
}

fn type_spec_is_unresolved(spec: &TypeSpec) -> bool {
    match spec {
        TypeSpec::GenericParam(_) => true,
        TypeSpec::Tuple(items) => items.iter().any(type_spec_is_unresolved),
        TypeSpec::Array(inner, _) | TypeSpec::Range(inner) => type_spec_is_unresolved(inner),
        TypeSpec::Reference { inner, .. } => type_spec_is_unresolved(inner),
        TypeSpec::Fn(params, ret) => {
            params.iter().any(type_spec_is_unresolved) || type_spec_is_unresolved(ret)
        }
        TypeSpec::Struct(_, args)
        | TypeSpec::Enum(_, args)
        | TypeSpec::TraitObject { args, .. } => args.iter().any(type_spec_is_unresolved),
        TypeSpec::Int
        | TypeSpec::Float
        | TypeSpec::Bool
        | TypeSpec::Char
        | TypeSpec::Byte
        | TypeSpec::String
        | TypeSpec::Unit
        | TypeSpec::Never
        | TypeSpec::Known(_) => false,
    }
}

fn contains_unresolved_syntax(expr: &Expr) -> bool {
    match expr {
        Expr::Call { generic_args, args, .. } => {
            generic_args.iter().any(type_spec_is_unresolved)
                || args.iter().any(contains_unresolved_syntax)
        }
        Expr::MethodCall { .. } => true,
        Expr::Let { ty, init, body, .. } => {
            ty.as_ref().is_some_and(type_spec_is_unresolved)
                || contains_unresolved_syntax(init)
                || contains_unresolved_syntax(body)
        }
        Expr::Binary { lhs, rhs, .. } => {
            contains_unresolved_syntax(lhs) || contains_unresolved_syntax(rhs)
        }
        Expr::Unary { expr, .. }
        | Expr::Field { expr, .. }
        | Expr::Loop { body: expr, .. }
        | Expr::UnsafeBlock { body: expr } => contains_unresolved_syntax(expr),
        Expr::Index { expr, index } => {
            contains_unresolved_syntax(expr) || contains_unresolved_syntax(index)
        }
        Expr::Cast { expr, ty } => type_spec_is_unresolved(ty) || contains_unresolved_syntax(expr),
        Expr::Struct { generic_args, fields, .. } => {
            generic_args.iter().any(type_spec_is_unresolved)
                || fields.iter().any(|(_, value)| contains_unresolved_syntax(value))
        }
        Expr::EnumVariant { generic_args, args, .. } => {
            generic_args.iter().any(type_spec_is_unresolved)
                || args.iter().any(contains_unresolved_syntax)
        }
        Expr::Tuple(items)
        | Expr::Array(items)
        | Expr::Interpolation(items)
        | Expr::Block(items) => items.iter().any(contains_unresolved_syntax),
        Expr::Range { start, end, .. } => {
            contains_unresolved_syntax(start) || contains_unresolved_syntax(end)
        }
        Expr::Match { expr, arms } => {
            contains_unresolved_syntax(expr)
                || arms.iter().any(|arm| {
                    arm.guard.as_ref().is_some_and(contains_unresolved_syntax)
                        || contains_unresolved_syntax(&arm.body)
                })
        }
        Expr::If { condition, then_branch, else_branch } => {
            contains_unresolved_syntax(condition)
                || contains_unresolved_syntax(then_branch)
                || else_branch.as_ref().is_some_and(|expr| contains_unresolved_syntax(expr))
        }
        Expr::Lambda { params, body } => {
            params.iter().any(|(_, spec)| type_spec_is_unresolved(spec))
                || contains_unresolved_syntax(body)
        }
        Expr::Assign { target, value } | Expr::CompoundAssign { target, value, .. } => {
            contains_unresolved_syntax(target) || contains_unresolved_syntax(value)
        }
        Expr::While { condition, body, .. } => {
            contains_unresolved_syntax(condition) || contains_unresolved_syntax(body)
        }
        Expr::For { iterable, body, .. } => {
            contains_unresolved_syntax(iterable) || contains_unresolved_syntax(body)
        }
        Expr::Break { value, .. } => value.as_ref().is_some_and(|expr| contains_unresolved_syntax(expr)),
        Expr::Continue { .. } | Expr::Return(None) | Expr::Literal(_) | Expr::Var(_) => false,
        Expr::Return(Some(expr)) => contains_unresolved_syntax(expr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn concrete_program() -> MonomorphizedProgram {
        MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "main".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Int,
                effects: EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Literal(omni_types::ast::Lit::Int(42)),
            }],
        }
    }

    #[test]
    fn concrete_program_constructs_canonical_hir() {
        let hir = HirProgram::from_monomorphized_without_defs(&concrete_program()).unwrap();
        hir.validate().unwrap();
        assert_eq!(hir.functions.len(), 1);
        assert!(matches!(hir.tcx.get(hir.functions[0].return_ty), omni_types::TyKind::Int));
    }

    #[test]
    fn generic_function_is_rejected_before_hir() {
        let mut program = concrete_program();
        program.functions[0].type_params.push("T".to_string());
        assert!(HirProgram::from_monomorphized_without_defs(&program).is_err());
    }

    #[test]
    fn unresolved_method_call_is_rejected() {
        let mut program = concrete_program();
        program.functions[0].body = Expr::MethodCall {
            receiver: Box::new(Expr::Var("x".to_string())),
            method: "get".to_string(),
            generic_args: vec![],
            args: vec![],
        };
        assert!(HirProgram::from_monomorphized_without_defs(&program).is_err());
    }
}
