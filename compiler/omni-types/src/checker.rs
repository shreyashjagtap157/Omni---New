use std::collections::HashMap;

use crate::ast::{Expr, GenericFnDef, Lit, TypeSpec};
use crate::intern::{Ty, TyCtxt, TyKind};
use crate::solver::Solver;
use omni_effects::{CapabilityContext, EffectRow};

/// Errors encountered during type checking and substitution resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    UnresolvedSubstitution(String),
    MismatchedTypes { expected: String, found: String },
    FunctionNotFound(String),
    VariableNotFound(String),
    SolverFailure(String),
    TraitObligationUnsatisfied(String),
    NonExhaustiveMatch { scrutinee_ty: String, missing: String },
    UnreachablePattern { arm_index: usize, detail: String },
    EffectViolation(String),
    ArgumentCountMismatch { expected: usize, found: usize },
    GenericArgumentCountMismatch { expected: usize, found: usize },
    UnsupportedOperator(String),
    FieldNotFound { ty: String, field: String },
    UnsupportedCast { from: String, to: String },
    BreakOutsideLoop,
    ContinueOutsideLoop,
    InvalidLoopBreakType { expected: String, found: String },
    UnsupportedPattern(String),
}

/// Concrete generic substitution environment mapping parameter names to concrete interned types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubstEnv {
    pub bindings: HashMap<String, Ty>,
}

impl SubstEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, param: String, ty: Ty) {
        self.bindings.insert(param, ty);
    }

    pub fn get(&self, param: &str) -> Option<Ty> {
        self.bindings.get(param).copied()
    }

    /// Verifies if all bindings in this environment are fully concrete types.
    pub fn is_concrete(&self, tcx: &TyCtxt) -> bool {
        self.bindings.values().all(|&ty| tcx.is_concrete(ty))
    }

    /// Applies the substitution environment to a type, replacing generic parameters.
    pub fn substitute_ty(&self, tcx: &mut TyCtxt, ty: Ty) -> Ty {
        match tcx.get(ty).clone() {
            TyKind::GenericParam(name) => {
                if let Some(&concrete) = self.bindings.get(&name) {
                    concrete
                } else {
                    ty
                }
            }
            TyKind::Tuple(tys) => {
                let substituted: Vec<Ty> =
                    tys.into_iter().map(|t| self.substitute_ty(tcx, t)).collect();
                tcx.intern(TyKind::Tuple(substituted))
            }
            TyKind::Array(elem, len) => {
                let sub_elem = self.substitute_ty(tcx, elem);
                tcx.intern(TyKind::Array(sub_elem, len))
            }
            TyKind::Range(elem) => {
                let sub_elem = self.substitute_ty(tcx, elem);
                tcx.intern(TyKind::Range(sub_elem))
            }
            TyKind::Fn(params, ret) => {
                let sub_params = params.into_iter().map(|p| self.substitute_ty(tcx, p)).collect();
                let sub_ret = self.substitute_ty(tcx, ret);
                tcx.intern(TyKind::Fn(sub_params, sub_ret))
            }
            TyKind::Struct(name, args) => {
                let sub_args = args.into_iter().map(|a| self.substitute_ty(tcx, a)).collect();
                tcx.intern(TyKind::Struct(name, sub_args))
            }
            TyKind::Enum(name, args) => {
                let sub_args = args.into_iter().map(|a| self.substitute_ty(tcx, a)).collect();
                tcx.intern(TyKind::Enum(name, sub_args))
            }
            _ => ty,
        }
    }
}

/// Unique key representing a monomorphized specialization of a generic function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecializationKey {
    pub fn_name: String,
    pub type_args: Vec<Ty>,
}

impl SpecializationKey {
    pub fn mangled_name(&self, tcx: &TyCtxt) -> String {
        if self.type_args.is_empty() {
            self.fn_name.clone()
        } else {
            let args: Vec<String> = self.type_args.iter().map(|&t| tcx.mangle(t)).collect();
            format!("{}_spec_{}", self.fn_name, args.join("_"))
        }
    }
}

use std::sync::Arc;

pub type TraitObligationChecker = Arc<
    dyn Fn(&[(String, crate::ast::TraitBound)], &SubstEnv, &TyCtxt) -> Result<(), String>
        + Send
        + Sync,
>;

/// The Type Checker is the authoritative source of truth for type inference and generic substitutions.
pub struct TypeChecker {
    pub tcx: TyCtxt,
    pub solver: Solver,
    pub fn_defs: HashMap<String, GenericFnDef>,
    pub struct_defs: HashMap<String, crate::ast::StructDef>,
    pub enum_defs: HashMap<String, crate::ast::EnumDef>,
    loop_break_types: Vec<(Option<String>, Option<Ty>)>,
    pub trait_checker: Option<TraitObligationChecker>,
    pub cap_context: CapabilityContext,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            tcx: TyCtxt::new(),
            solver: Solver::new(),
            fn_defs: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            loop_break_types: Vec::new(),
            trait_checker: None,
            cap_context: CapabilityContext::new(),
        }
    }

    pub fn set_trait_checker(&mut self, checker: TraitObligationChecker) {
        self.trait_checker = Some(checker);
    }

    pub fn register_fn(&mut self, fn_def: GenericFnDef) {
        self.fn_defs.insert(fn_def.name.clone(), fn_def);
    }

    pub fn register_struct(&mut self, struct_def: crate::ast::StructDef) {
        self.struct_defs.insert(struct_def.name.clone(), struct_def);
    }

    pub fn register_enum(&mut self, enum_def: crate::ast::EnumDef) {
        self.enum_defs.insert(enum_def.name.clone(), enum_def);
    }

    /// Lowers a `TypeSpec` into an interned `Ty` under a given `SubstEnv`.
    pub fn lower_type_spec(&mut self, spec: &TypeSpec, env: &SubstEnv) -> Ty {
        self.tcx.lower_type_spec(spec, env)
    }

    /// Infers the type of a literal expression.
    pub fn infer_literal(&mut self, lit: &Lit) -> Ty {
        match lit {
            Lit::Int(_) => self.tcx.intern(TyKind::Int),
            Lit::Float(_) => self.tcx.intern(TyKind::Float),
            Lit::Bool(_) => self.tcx.intern(TyKind::Bool),
            Lit::Char(_) => self.tcx.intern(TyKind::Char),
            Lit::Byte(_) => self.tcx.intern(TyKind::Byte),
            Lit::String(_) => self.tcx.intern(TyKind::String),
        }
    }

    /// Performs type inference for a call site and derives authoritative generic substitutions.
    /// Returns (return_type, specialization_key, subst_env, callee_effects).
    pub fn infer_call(
        &mut self,
        fn_name: &str,
        explicit_generic_args: &[TypeSpec],
        args: &[Expr],
        env: &SubstEnv,
        local_vars: &HashMap<String, Ty>,
    ) -> Result<(Ty, SpecializationKey, SubstEnv, EffectRow), TypeError> {
        let fn_def = self
            .fn_defs
            .get(fn_name)
            .cloned()
            .ok_or_else(|| TypeError::FunctionNotFound(fn_name.to_string()))?;

        if explicit_generic_args.len() > fn_def.type_params.len() {
            return Err(TypeError::GenericArgumentCountMismatch {
                expected: fn_def.type_params.len(),
                found: explicit_generic_args.len(),
            });
        }
        if args.len() != fn_def.params.len() {
            return Err(TypeError::ArgumentCountMismatch {
                expected: fn_def.params.len(),
                found: args.len(),
            });
        }

        let mut derived_subst = SubstEnv::new();

        // Bind explicit generic arguments if supplied or in active environment
        for (i, param_name) in fn_def.type_params.iter().enumerate() {
            if i < explicit_generic_args.len() {
                let ty = self.lower_type_spec(&explicit_generic_args[i], env);
                derived_subst.insert(param_name.clone(), ty);
            } else if let Some(ty) = env.get(param_name) {
                derived_subst.insert(param_name.clone(), ty);
            }
        }

        // Infer concrete types from arguments for remaining generic parameters
        for (i, (_param_name, param_spec)) in fn_def.params.iter().enumerate() {
            let expected_param_ty = self.lower_type_spec(param_spec, &derived_subst);
            if i < args.len() {
                let arg_ty = self.infer_expr(&args[i], env, local_vars)?;
                self.unify_types(expected_param_ty, arg_ty, &mut derived_subst)?;
            }
        }

        // Collect final specialization key type arguments
        let mut key_args = Vec::new();
        for param_name in &fn_def.type_params {
            if let Some(concrete_ty) = derived_subst.get(param_name) {
                if !self.tcx.is_concrete(concrete_ty) {
                    return Err(TypeError::UnresolvedSubstitution(param_name.clone()));
                }
                key_args.push(concrete_ty);
            } else {
                return Err(TypeError::UnresolvedSubstitution(param_name.clone()));
            }
        }

        // If trait bounds are specified on this function, verify them against concrete substitutions
        if !fn_def.bounds.is_empty() {
            if let Some(ref checker) = self.trait_checker {
                checker(&fn_def.bounds, &derived_subst, &self.tcx)
                    .map_err(TypeError::TraitObligationUnsatisfied)?;
            }
        }

        // Verify capability requirements at call site
        let missing = self.cap_context.missing(&fn_def.capabilities);
        if let Some(first_missing) = missing.into_iter().next() {
            return Err(TypeError::EffectViolation(format!(
                "missing capability `{first_missing}` required to call `{fn_name}`"
            )));
        }

        let ret_ty = self.lower_type_spec(&fn_def.return_type, &derived_subst);
        let key = SpecializationKey { fn_name: fn_name.to_string(), type_args: key_args };
        let callee_effects = fn_def.effects.clone();

        Ok((ret_ty, key, derived_subst, callee_effects))
    }

    fn unify_types(
        &mut self,
        expected: Ty,
        found: Ty,
        subst: &mut SubstEnv,
    ) -> Result<(), TypeError> {
        match (self.tcx.get(expected).clone(), self.tcx.get(found).clone()) {
            (TyKind::GenericParam(name), _) => {
                subst.insert(name, found);
                Ok(())
            }
            (TyKind::Tuple(e_tys), TyKind::Tuple(f_tys)) if e_tys.len() == f_tys.len() => {
                for (e, f) in e_tys.into_iter().zip(f_tys) {
                    self.unify_types(e, f, subst)?;
                }
                Ok(())
            }
            (TyKind::Array(e_elem, e_len), TyKind::Array(f_elem, f_len)) if e_len == f_len => {
                self.unify_types(e_elem, f_elem, subst)
            }
            (TyKind::Range(e_elem), TyKind::Range(f_elem)) => {
                self.unify_types(e_elem, f_elem, subst)
            }
            (TyKind::Fn(e_p, e_r), TyKind::Fn(f_p, f_r)) if e_p.len() == f_p.len() => {
                for (e, f) in e_p.into_iter().zip(f_p) {
                    self.unify_types(e, f, subst)?;
                }
                self.unify_types(e_r, f_r, subst)
            }
            (TyKind::Struct(e_name, e_args), TyKind::Struct(f_name, f_args))
                if e_name == f_name && e_args.len() == f_args.len() =>
            {
                for (e, f) in e_args.into_iter().zip(f_args) {
                    self.unify_types(e, f, subst)?;
                }
                Ok(())
            }
            (TyKind::Enum(e_name, e_args), TyKind::Enum(f_name, f_args))
                if e_name == f_name && e_args.len() == f_args.len() =>
            {
                for (e, f) in e_args.into_iter().zip(f_args) {
                    self.unify_types(e, f, subst)?;
                }
                Ok(())
            }
            (_e, _f) if expected == found => Ok(()),
            _ => Err(TypeError::MismatchedTypes {
                expected: self.tcx.mangle(expected),
                found: self.tcx.mangle(found),
            }),
        }
    }

    /// Infers the type of an expression under environment and local variables.
    pub fn infer_expr(
        &mut self,
        expr: &Expr,
        env: &SubstEnv,
        local_vars: &HashMap<String, Ty>,
    ) -> Result<Ty, TypeError> {
        match expr {
            Expr::Literal(lit) => Ok(self.infer_literal(lit)),
            Expr::Var(name) => local_vars
                .get(name)
                .copied()
                .ok_or_else(|| TypeError::VariableNotFound(name.clone())),
            Expr::Call { func, generic_args, args } => {
                let (ret_ty, _, _, _) =
                    self.infer_call(func, generic_args, args, env, local_vars)?;
                Ok(ret_ty)
            }
            Expr::Let { name, ty, init, body } => {
                let init_ty = self.infer_expr(init, env, local_vars)?;
                let declared_ty =
                    if let Some(spec) = ty { self.lower_type_spec(spec, env) } else { init_ty };
                if declared_ty != init_ty {
                    return Err(TypeError::MismatchedTypes {
                        expected: self.tcx.mangle(declared_ty),
                        found: self.tcx.mangle(init_ty),
                    });
                }
                let mut inner_vars = local_vars.clone();
                inner_vars.insert(name.clone(), declared_ty);
                self.infer_expr(body, env, &inner_vars)
            }
            Expr::Binary { op, lhs, rhs } => {
                let l_ty = self.infer_expr(lhs, env, local_vars)?;
                let r_ty = self.infer_expr(rhs, env, local_vars)?;
                if l_ty != r_ty {
                    return Err(TypeError::MismatchedTypes {
                        expected: self.tcx.mangle(l_ty),
                        found: self.tcx.mangle(r_ty),
                    });
                }

                match op {
                    crate::ast::BinOp::Eq
                    | crate::ast::BinOp::Ne
                    | crate::ast::BinOp::Lt
                    | crate::ast::BinOp::Le
                    | crate::ast::BinOp::Gt
                    | crate::ast::BinOp::Ge
                    | crate::ast::BinOp::LogicalAnd
                    | crate::ast::BinOp::LogicalOr => Ok(self.tcx.intern(TyKind::Bool)),
                    crate::ast::BinOp::Add
                    | crate::ast::BinOp::Sub
                    | crate::ast::BinOp::Mul
                    | crate::ast::BinOp::Div
                    | crate::ast::BinOp::Rem
                    | crate::ast::BinOp::BitAnd
                    | crate::ast::BinOp::BitOr
                    | crate::ast::BinOp::BitXor
                    | crate::ast::BinOp::Shl
                    | crate::ast::BinOp::Shr => Ok(l_ty),
                }
            }
            Expr::Unary { op, expr } => {
                let inner_ty = self.infer_expr(expr, env, local_vars)?;
                let expected = match op {
                    crate::ast::UnOp::Neg => self.tcx.intern(TyKind::Int),
                    crate::ast::UnOp::Not => self.tcx.intern(TyKind::Bool),
                };
                if inner_ty != expected {
                    return Err(TypeError::MismatchedTypes {
                        expected: self.tcx.mangle(expected),
                        found: self.tcx.mangle(inner_ty),
                    });
                }
                Ok(expected)
            }
            Expr::Field { expr, field } => {
                let struct_ty = self.infer_expr(expr, env, local_vars)?;
                let TyKind::Struct(name, args) = self.tcx.get(struct_ty).clone() else {
                    return Err(TypeError::FieldNotFound {
                        ty: self.tcx.mangle(struct_ty),
                        field: field.clone(),
                    });
                };
                let def = self
                    .struct_defs
                    .get(&name)
                    .ok_or_else(|| TypeError::FieldNotFound {
                        ty: name.clone(),
                        field: field.clone(),
                    })?;
                let field_def = def
                    .fields
                    .iter()
                    .find(|f| f.name == *field)
                    .ok_or_else(|| TypeError::FieldNotFound {
                        ty: name.clone(),
                        field: field.clone(),
                    })?;
                let mut field_env = SubstEnv::new();
                for (param, arg) in def.type_params.iter().zip(args.iter()) {
                    field_env.insert(param.clone(), *arg);
                }
                Ok(self.lower_type_spec(&field_def.ty, &field_env))
            }
            Expr::Index { expr, index } => {
                let arr_ty = self.infer_expr(expr, env, local_vars)?;
                let index_ty = self.infer_expr(index, env, local_vars)?;
                if index_ty != self.tcx.intern(TyKind::Int) {
                    return Err(TypeError::MismatchedTypes {
                        expected: "int".to_string(),
                        found: self.tcx.mangle(index_ty),
                    });
                }
                match self.tcx.get(arr_ty).clone() {
                    TyKind::Array(elem, _) => Ok(elem),
                    other => Err(TypeError::MismatchedTypes {
                        expected: "array".to_string(),
                        found: self.tcx.mangle(self.tcx.intern(other)),
                    }),
                }
            }
            Expr::Tuple(elems) => {
                let elem_tys: Result<Vec<Ty>, TypeError> =
                    elems.iter().map(|e| self.infer_expr(e, env, local_vars)).collect();
                Ok(self.tcx.intern(TyKind::Tuple(elem_tys?)))
            }
            Expr::Array(elems) => {
                if elems.is_empty() {
                    return Ok(self.tcx.intern(TyKind::Array(
                        self.tcx.intern(TyKind::Never),
                        0,
                    )));
                }
                let element_ty = self.infer_expr(&elems[0], env, local_vars)?;
                for elem in &elems[1..] {
                    let ty = self.infer_expr(elem, env, local_vars)?;
                    if ty != element_ty {
                        return Err(TypeError::MismatchedTypes {
                            expected: self.tcx.mangle(element_ty),
                            found: self.tcx.mangle(ty),
                        });
                    }
                }
                Ok(self.tcx.intern(TyKind::Array(element_ty, elems.len())))
            }
            Expr::Range { start, end } => {
                let start_ty = self.infer_expr(start, env, local_vars)?;
                let end_ty = self.infer_expr(end, env, local_vars)?;
                if start_ty != end_ty {
                    return Err(TypeError::MismatchedTypes {
                        expected: self.tcx.mangle(start_ty),
                        found: self.tcx.mangle(end_ty),
                    });
                }
                Ok(self.tcx.intern(TyKind::Range(start_ty)))
            }
            Expr::If { condition, then_branch, else_branch } => {
                let condition_ty = self.infer_expr(condition, env, local_vars)?;
                let bool_ty = self.tcx.intern(TyKind::Bool);
                if condition_ty != bool_ty {
                    return Err(TypeError::MismatchedTypes {
                        expected: "bool".to_string(),
                        found: self.tcx.mangle(condition_ty),
                    });
                }
                let then_ty = self.infer_expr(then_branch, env, local_vars)?;
                if let Some(else_expr) = else_branch {
                    let else_ty = self.infer_expr(else_expr, env, local_vars)?;
                    if then_ty != else_ty {
                        return Err(TypeError::MismatchedTypes {
                            expected: self.tcx.mangle(then_ty),
                            found: self.tcx.mangle(else_ty),
                        });
                    }
                    Ok(then_ty)
                } else {
                    let unit_ty = self.tcx.intern(TyKind::Unit);
                    if then_ty != unit_ty {
                        return Err(TypeError::MismatchedTypes {
                            expected: self.tcx.mangle(unit_ty),
                            found: self.tcx.mangle(then_ty),
                        });
                    }
                    Ok(unit_ty)
                }
            }
            Expr::Cast { expr, ty } => {
                let source_ty = self.infer_expr(expr, env, local_vars)?;
                let target_ty = self.lower_type_spec(ty, env);
                if !matches!(self.tcx.get(source_ty), TyKind::Int | TyKind::Byte | TyKind::Char)
                    || !matches!(self.tcx.get(target_ty), TyKind::Int | TyKind::Byte | TyKind::Char)
                {
                    return Err(TypeError::UnsupportedCast {
                        from: self.tcx.mangle(source_ty),
                        to: self.tcx.mangle(target_ty),
                    });
                }
                Ok(target_ty)
            }
            Expr::Loop { label, body } => {
                self.loop_break_types.push((label.clone(), None));
                let body_result = self.infer_expr(body, env, local_vars)?;
                let (_, break_ty) = self.loop_break_types.pop().expect("loop stack balanced");
                match break_ty {
                    Some(ty) => Ok(ty),
                    None => {
                        if matches!(self.tcx.get(body_result), TyKind::Never) {
                            Ok(self.tcx.intern(TyKind::Never))
                        } else {
                            Ok(self.tcx.intern(TyKind::Never))
                        }
                    }
                }
            }
            Expr::While { label, condition, body } => {
                let condition_ty = self.infer_expr(condition, env, local_vars)?;
                let bool_ty = self.tcx.intern(TyKind::Bool);
                if condition_ty != bool_ty {
                    return Err(TypeError::MismatchedTypes {
                        expected: "bool".to_string(),
                        found: self.tcx.mangle(condition_ty),
                    });
                }
                self.loop_break_types.push((label.clone(), Some(self.tcx.intern(TyKind::Unit))));
                self.infer_expr(body, env, local_vars)?;
                self.loop_break_types.pop();
                Ok(self.tcx.intern(TyKind::Unit))
            }
            Expr::Break { label, value } => {
                let loop_index = match label {
                    Some(label) => self
                        .loop_break_types
                        .iter()
                        .rposition(|(loop_label, _)| loop_label.as_deref() == Some(label.as_str()))
                        .ok_or(TypeError::BreakOutsideLoop)?,
                    None => self.loop_break_types.len().checked_sub(1).ok_or(TypeError::BreakOutsideLoop)?,
                };
                let value_ty = if let Some(expr) = value {
                    self.infer_expr(expr, env, local_vars)?
                } else {
                    self.tcx.intern(TyKind::Unit)
                };
                let expected = self.loop_break_types[loop_index].1;
                if let Some(expected) = expected {
                    if expected != value_ty {
                        return Err(TypeError::InvalidLoopBreakType {
                            expected: self.tcx.mangle(expected),
                            found: self.tcx.mangle(value_ty),
                        });
                    }
                } else {
                    self.loop_break_types[loop_index].1 = Some(value_ty);
                }
                Ok(self.tcx.intern(TyKind::Never))
            }
            Expr::Continue { label } => {
                let found = match label {
                    Some(label) => self.loop_break_types.iter().rev().any(|(loop_label, _)| {
                        loop_label.as_deref() == Some(label.as_str())
                    }),
                    None => !self.loop_break_types.is_empty(),
                };
                if found {
                    Ok(self.tcx.intern(TyKind::Never))
                } else {
                    Err(TypeError::ContinueOutsideLoop)
                }
            }
            Expr::Match { expr, arms } => {
                for arm in arms {
                    if matches!(
                        arm.pattern,
                        crate::ast::Pattern::Range {
                            end: crate::ast::PatternRangeBoundary::Unbounded,
                            ..
                        }
                    ) {
                        return Err(TypeError::UnsupportedPattern(
                            "unbounded range patterns are parsed but not yet type-checked".into(),
                        ));
                    }
                }

                let scrutinee_ty = self.infer_expr(expr, env, local_vars)?;

                for arm in arms {
                    if let Some(guard_expr) = &arm.guard {
                        let guard_ty = self.infer_expr(guard_expr, env, local_vars)?;
                        let bool_ty = self.tcx.intern(TyKind::Bool);
                        if guard_ty != bool_ty {
                            return Err(TypeError::MismatchedTypes {
                                expected: "bool".to_string(),
                                found: self.tcx.mangle(guard_ty),
                            });
                        }
                    }
                }

                // Enforce pattern usefulness and exhaustiveness analysis
                let mut pat_checker =
                    crate::pattern::PatternChecker::new(&mut self.tcx, &self.enum_defs);
                pat_checker.check_match(scrutinee_ty, arms)?;

                if let Some(first_arm) = arms.first() {
                    let first_ty = self.infer_expr(&first_arm.body, env, local_vars)?;
                    for arm in &arms[1..] {
                        let arm_ty = self.infer_expr(&arm.body, env, local_vars)?;
                        if arm_ty != first_ty {
                            return Err(TypeError::MismatchedTypes {
                                expected: self.tcx.mangle(first_ty),
                                found: self.tcx.mangle(arm_ty),
                            });
                        }
                    }
                    Ok(first_ty)
                } else {
                    Ok(self.tcx.intern(TyKind::Unit))
                }
            }
            Expr::Lambda { params, body } => {
                let mut lambda_vars = local_vars.clone();
                let mut param_tys = Vec::new();
                for (p_name, p_spec) in params {
                    let p_ty = self.lower_type_spec(p_spec, env);
                    lambda_vars.insert(p_name.clone(), p_ty);
                    param_tys.push(p_ty);
                }
                let body_ty = self.infer_expr(body, env, &lambda_vars)?;
                Ok(self.tcx.intern(TyKind::Fn(param_tys, body_ty)))
            }
            Expr::Interpolation(parts) => {
                for part in parts {
                    self.infer_expr(part, env, local_vars)?;
                }
                Ok(self.tcx.intern(TyKind::String))
            }
            Expr::Assign { target, value } => {
                let target_ty = self.infer_expr(target, env, local_vars)?;
                let value_ty = self.infer_expr(value, env, local_vars)?;
                if target_ty != value_ty {
                    return Err(TypeError::MismatchedTypes {
                        expected: self.tcx.mangle(target_ty),
                        found: self.tcx.mangle(value_ty),
                    });
                }
                Ok(target_ty)
            }
            Expr::CompoundAssign { op, target, value } => {
                let target_ty = self.infer_expr(target, env, local_vars)?;
                let value_ty = self.infer_expr(value, env, local_vars)?;
                if target_ty != value_ty {
                    return Err(TypeError::MismatchedTypes {
                        expected: self.tcx.mangle(target_ty),
                        found: self.tcx.mangle(value_ty),
                    });
                }
                if !matches!(target_ty, ty if ty == self.tcx.intern(TyKind::Int)) {
                    return Err(TypeError::MismatchedTypes {
                        expected: "int".to_string(),
                        found: self.tcx.mangle(target_ty),
                    });
                }
                match op {
                    crate::ast::AssignOp::Assign => Ok(target_ty),
                    _ => Ok(target_ty),
                }
            }
            Expr::Block(stmts) => {
                let mut last_ty = self.tcx.intern(TyKind::Unit);
                for stmt in stmts {
                    last_ty = self.infer_expr(stmt, env, local_vars)?;
                }
                Ok(last_ty)
            }
            Expr::Return(opt_expr) => {
                if let Some(inner) = opt_expr {
                    self.infer_expr(inner, env, local_vars)
                } else {
                    Ok(self.tcx.intern(TyKind::Unit))
                }
            }
        }
    }

    /// Infer compositional effect row produced by an expression.
    pub fn infer_expr_effects(
        &mut self,
        expr: &Expr,
        env: &SubstEnv,
        local_vars: &HashMap<String, Ty>,
    ) -> Result<EffectRow, TypeError> {
        match expr {
            Expr::Literal(_) | Expr::Var(_) => Ok(EffectRow::pure()),
            Expr::Call { func, generic_args, args } => {
                let mut effects = EffectRow::pure();
                for arg in args {
                    let arg_eff = self.infer_expr_effects(arg, env, local_vars)?;
                    effects = effects.union(&arg_eff);
                }
                let (_, _, _, callee_effects) =
                    self.infer_call(func, generic_args, args, env, local_vars)?;
                Ok(effects.union(&callee_effects))
            }
            Expr::Let { init, body, .. } => {
                let init_eff = self.infer_expr_effects(init, env, local_vars)?;
                let body_eff = self.infer_expr_effects(body, env, local_vars)?;
                Ok(init_eff.union(&body_eff))
            }
            Expr::Binary { lhs, rhs, .. } => {
                let l_eff = self.infer_expr_effects(lhs, env, local_vars)?;
                let r_eff = self.infer_expr_effects(rhs, env, local_vars)?;
                Ok(l_eff.union(&r_eff))
            }
            Expr::Unary { expr, .. } | Expr::Field { expr, .. } => {
                self.infer_expr_effects(expr, env, local_vars)
            }
            Expr::Index { expr, index } => {
                let e_eff = self.infer_expr_effects(expr, env, local_vars)?;
                let i_eff = self.infer_expr_effects(index, env, local_vars)?;
                Ok(e_eff.union(&i_eff))
            }
            Expr::Tuple(elems) | Expr::Array(elems) => {
                let mut eff = EffectRow::pure();
                for elem in elems {
                    eff = eff.union(&self.infer_expr_effects(elem, env, local_vars)?);
                }
                Ok(eff)
            }
            Expr::Range { start, end } => {
                let s_eff = self.infer_expr_effects(start, env, local_vars)?;
                let e_eff = self.infer_expr_effects(end, env, local_vars)?;
                Ok(s_eff.union(&e_eff))
            }
            Expr::Loop { body, .. } => self.infer_expr_effects(body, env, local_vars),
            Expr::While { condition, body, .. } => {
                let mut eff = self.infer_expr_effects(condition, env, local_vars)?;
                eff = eff.union(&self.infer_expr_effects(body, env, local_vars)?);
                Ok(eff)
            }
            Expr::Break { value, .. } => value
                .as_ref()
                .map(|e| self.infer_expr_effects(e, env, local_vars))
                .transpose()
                .map(|e| e.unwrap_or_else(EffectRow::pure)),
            Expr::Continue { .. } => Ok(EffectRow::pure()),
            Expr::If { condition, then_branch, else_branch } => {
                let mut eff = self.infer_expr_effects(condition, env, local_vars)?;
                eff = eff.union(&self.infer_expr_effects(then_branch, env, local_vars)?);
                if let Some(e) = else_branch {
                    eff = eff.union(&self.infer_expr_effects(e, env, local_vars)?);
                }
                Ok(eff)
            }
            Expr::Cast { expr, .. } => self.infer_expr_effects(expr, env, local_vars),
            Expr::Match { expr, arms } => {
                let mut eff = self.infer_expr_effects(expr, env, local_vars)?;
                for arm in arms {
                    if let Some(g) = &arm.guard {
                        eff = eff.union(&self.infer_expr_effects(g, env, local_vars)?);
                    }
                    eff = eff.union(&self.infer_expr_effects(&arm.body, env, local_vars)?);
                }
                Ok(eff)
            }
            Expr::Lambda { body, .. } => self.infer_expr_effects(body, env, local_vars),
            Expr::Interpolation(parts) => {
                let mut eff = EffectRow::pure();
                for p in parts {
                    eff = eff.union(&self.infer_expr_effects(p, env, local_vars)?);
                }
                Ok(eff)
            }
            Expr::Assign { target, value } => {
                let t_eff = self.infer_expr_effects(target, env, local_vars)?;
                let v_eff = self.infer_expr_effects(value, env, local_vars)?;
                Ok(t_eff.union(&v_eff))
            }
            Expr::CompoundAssign { target, value, .. } => {
                let t_eff = self.infer_expr_effects(target, env, local_vars)?;
                let v_eff = self.infer_expr_effects(value, env, local_vars)?;
                Ok(t_eff.union(&v_eff))
            }
            Expr::Block(stmts) => {
                let mut eff = EffectRow::pure();
                for stmt in stmts {
                    eff = eff.union(&self.infer_expr_effects(stmt, env, local_vars)?);
                }
                Ok(eff)
            }
            Expr::Return(opt_expr) => {
                if let Some(e) = opt_expr {
                    self.infer_expr_effects(e, env, local_vars)
                } else {
                    Ok(EffectRow::pure())
                }
            }
        }
    }

    /// Check function effect obligations against declared effect row and capabilities.
    pub fn check_fn_effects(&mut self, fn_def: &GenericFnDef) -> Result<EffectRow, TypeError> {
        let env = SubstEnv::new();
        let mut local_vars = HashMap::new();
        for (p_name, p_spec) in &fn_def.params {
            let p_ty = self.lower_type_spec(p_spec, &env);
            local_vars.insert(p_name.clone(), p_ty);
        }
        let body_effects = self.infer_expr_effects(&fn_def.body, &env, &local_vars)?;

        omni_effects::check_effect_obligations(
            &body_effects,
            &fn_def.effects,
            &fn_def.capabilities,
            &self.cap_context,
            &fn_def.name,
        )
        .map_err(|e| TypeError::EffectViolation(e.to_string()))?;

        Ok(body_effects)
    }
}
