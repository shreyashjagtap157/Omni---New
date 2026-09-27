use std::collections::HashMap;

use crate::ast::{Expr, GenericFnDef, Lit, TypeSpec};
use crate::intern::{Ty, TyCtxt, TyKind};
use crate::solver::Solver;

/// Errors encountered during type checking and substitution resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    UnresolvedSubstitution(String),
    MismatchedTypes { expected: String, found: String },
    FunctionNotFound(String),
    VariableNotFound(String),
    SolverFailure(String),
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

/// The Type Checker is the authoritative source of truth for type inference and generic substitutions.
pub struct TypeChecker {
    pub tcx: TyCtxt,
    pub solver: Solver,
    pub fn_defs: HashMap<String, GenericFnDef>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self { tcx: TyCtxt::new(), solver: Solver::new(), fn_defs: HashMap::new() }
    }

    pub fn register_fn(&mut self, fn_def: GenericFnDef) {
        self.fn_defs.insert(fn_def.name.clone(), fn_def);
    }

    /// Lowers a `TypeSpec` into an interned `Ty` under a given `SubstEnv`.
    pub fn lower_type_spec(&mut self, spec: &TypeSpec, env: &SubstEnv) -> Ty {
        match spec {
            TypeSpec::Int => self.tcx.intern(TyKind::Int),
            TypeSpec::Float => self.tcx.intern(TyKind::Float),
            TypeSpec::Bool => self.tcx.intern(TyKind::Bool),
            TypeSpec::Char => self.tcx.intern(TyKind::Char),
            TypeSpec::Byte => self.tcx.intern(TyKind::Byte),
            TypeSpec::String => self.tcx.intern(TyKind::String),
            TypeSpec::Unit => self.tcx.intern(TyKind::Unit),
            TypeSpec::GenericParam(name) => {
                if let Some(concrete) = env.get(name) {
                    concrete
                } else {
                    self.tcx.intern(TyKind::GenericParam(name.clone()))
                }
            }
            TypeSpec::Tuple(specs) => {
                let tys: Vec<Ty> = specs.iter().map(|s| self.lower_type_spec(s, env)).collect();
                self.tcx.intern(TyKind::Tuple(tys))
            }
            TypeSpec::Array(elem_spec, len) => {
                let elem_ty = self.lower_type_spec(elem_spec, env);
                self.tcx.intern(TyKind::Array(elem_ty, *len))
            }
            TypeSpec::Range(elem_spec) => {
                let elem_ty = self.lower_type_spec(elem_spec, env);
                self.tcx.intern(TyKind::Range(elem_ty))
            }
            TypeSpec::Fn(param_specs, ret_spec) => {
                let param_tys: Vec<Ty> =
                    param_specs.iter().map(|s| self.lower_type_spec(s, env)).collect();
                let ret_ty = self.lower_type_spec(ret_spec, env);
                self.tcx.intern(TyKind::Fn(param_tys, ret_ty))
            }
            TypeSpec::Struct(name, arg_specs) => {
                let arg_tys: Vec<Ty> =
                    arg_specs.iter().map(|s| self.lower_type_spec(s, env)).collect();
                self.tcx.intern(TyKind::Struct(name.clone(), arg_tys))
            }
            TypeSpec::Known(ty) => *ty,
        }
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
    pub fn infer_call(
        &mut self,
        fn_name: &str,
        explicit_generic_args: &[TypeSpec],
        args: &[Expr],
        env: &SubstEnv,
        local_vars: &HashMap<String, Ty>,
    ) -> Result<(Ty, SpecializationKey, SubstEnv), TypeError> {
        let fn_def = self
            .fn_defs
            .get(fn_name)
            .cloned()
            .ok_or_else(|| TypeError::FunctionNotFound(fn_name.to_string()))?;

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

        let ret_ty = self.lower_type_spec(&fn_def.return_type, &derived_subst);
        let key = SpecializationKey { fn_name: fn_name.to_string(), type_args: key_args };

        Ok((ret_ty, key, derived_subst))
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
                let (ret_ty, _, _) = self.infer_call(func, generic_args, args, env, local_vars)?;
                Ok(ret_ty)
            }
            Expr::Let { name, ty, init, body } => {
                let init_ty = self.infer_expr(init, env, local_vars)?;
                let declared_ty =
                    if let Some(spec) = ty { self.lower_type_spec(spec, env) } else { init_ty };
                let mut inner_vars = local_vars.clone();
                inner_vars.insert(name.clone(), declared_ty);
                self.infer_expr(body, env, &inner_vars)
            }
            Expr::Binary { op: _, lhs, rhs } => {
                let l_ty = self.infer_expr(lhs, env, local_vars)?;
                let _r_ty = self.infer_expr(rhs, env, local_vars)?;
                Ok(l_ty)
            }
            Expr::Unary { op: _, expr } => self.infer_expr(expr, env, local_vars),
            Expr::Field { expr, field: _ } => {
                let struct_ty = self.infer_expr(expr, env, local_vars)?;
                if let TyKind::Struct(_, args) = self.tcx.get(struct_ty).clone() {
                    if let Some(&first_arg) = args.first() {
                        return Ok(first_arg);
                    }
                }
                Ok(self.tcx.intern(TyKind::Int))
            }
            Expr::Index { expr, index: _ } => {
                let arr_ty = self.infer_expr(expr, env, local_vars)?;
                if let TyKind::Array(elem, _) = self.tcx.get(arr_ty).clone() {
                    return Ok(elem);
                }
                Ok(self.tcx.intern(TyKind::Int))
            }
            Expr::Tuple(elems) => {
                let elem_tys: Result<Vec<Ty>, TypeError> =
                    elems.iter().map(|e| self.infer_expr(e, env, local_vars)).collect();
                Ok(self.tcx.intern(TyKind::Tuple(elem_tys?)))
            }
            Expr::Array(elems) => {
                let first_ty = if let Some(first) = elems.first() {
                    self.infer_expr(first, env, local_vars)?
                } else {
                    self.tcx.intern(TyKind::Int)
                };
                Ok(self.tcx.intern(TyKind::Array(first_ty, elems.len())))
            }
            Expr::Range { start, end: _ } => {
                let elem_ty = self.infer_expr(start, env, local_vars)?;
                Ok(self.tcx.intern(TyKind::Range(elem_ty)))
            }
            Expr::Match { expr, arms } => {
                let _scrutinee_ty = self.infer_expr(expr, env, local_vars)?;
                if let Some((_, arm_expr)) = arms.first() {
                    self.infer_expr(arm_expr, env, local_vars)
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
            Expr::Assign { target: _, value } => self.infer_expr(value, env, local_vars),
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
}
