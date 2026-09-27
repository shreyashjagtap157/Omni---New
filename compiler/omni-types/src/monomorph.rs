use std::collections::{HashMap, HashSet};

use crate::ast::{Expr, GenericFnDef, TypeSpec};
use crate::checker::{SpecializationKey, SubstEnv, TypeChecker, TypeError};
use crate::intern::{Ty, TyKind};

/// Output of monomorphizing a generic program: specialized non-generic function definitions.
#[derive(Debug, Clone)]
pub struct MonomorphizedProgram {
    pub functions: Vec<GenericFnDef>,
}

/// The Monomorphization Engine.
pub struct Monomorphizer<'a> {
    pub checker: &'a mut TypeChecker,
    pub cache: HashMap<SpecializationKey, GenericFnDef>,
    pub in_progress: HashSet<SpecializationKey>,
}

impl<'a> Monomorphizer<'a> {
    pub fn new(checker: &'a mut TypeChecker) -> Self {
        Self { checker, cache: HashMap::new(), in_progress: HashSet::new() }
    }

    /// Monomorphizes an entry-point call (e.g. `main` or a concrete instantiation call).
    pub fn monomorphize_entry(
        &mut self,
        fn_name: &str,
        explicit_args: &[TypeSpec],
        arg_exprs: &[Expr],
    ) -> Result<MonomorphizedProgram, TypeError> {
        let env = SubstEnv::new();
        let local_vars = HashMap::new();
        let (_, key, subst_env) =
            self.checker.infer_call(fn_name, explicit_args, arg_exprs, &env, &local_vars)?;

        self.monomorphize_fn(&key, &subst_env)?;

        let functions = self.cache.values().cloned().collect();
        Ok(MonomorphizedProgram { functions })
    }

    /// Monomorphizes a generic function under a concrete `SubstEnv`.
    pub fn monomorphize_fn(
        &mut self,
        key: &SpecializationKey,
        env: &SubstEnv,
    ) -> Result<String, TypeError> {
        if !env.is_concrete(&self.checker.tcx) {
            for (p_name, ty) in &env.bindings {
                if !self.checker.tcx.is_concrete(*ty) {
                    return Err(TypeError::UnresolvedSubstitution(p_name.clone()));
                }
            }
        }

        let mangled_name = key.mangled_name(&self.checker.tcx);

        if self.cache.contains_key(key) || self.in_progress.contains(key) {
            return Ok(mangled_name);
        }

        let fn_def = self
            .checker
            .fn_defs
            .get(&key.fn_name)
            .cloned()
            .ok_or_else(|| TypeError::FunctionNotFound(key.fn_name.clone()))?;

        self.in_progress.insert(key.clone());

        // Substitute parameters and return type using concrete environment
        let mut spec_params = Vec::new();
        let mut local_vars = HashMap::new();
        for (p_name, p_spec) in &fn_def.params {
            let sub_spec = self.substitute_type_spec(p_spec, env);
            let p_ty = self.checker.lower_type_spec(&sub_spec, env);
            local_vars.insert(p_name.clone(), p_ty);
            spec_params.push((p_name.clone(), sub_spec));
        }

        let spec_ret = self.substitute_type_spec(&fn_def.return_type, env);

        // Monomorphize function body under active substitution environment
        let spec_body = self.monomorphize_expr(&fn_def.body, env, &local_vars)?;

        let monomorphized_def = GenericFnDef {
            name: mangled_name.clone(),
            type_params: vec![],
            params: spec_params,
            return_type: spec_ret,
            body: spec_body,
        };

        self.in_progress.remove(key);
        self.checker.register_fn(monomorphized_def.clone());
        self.cache.insert(key.clone(), monomorphized_def);

        Ok(mangled_name)
    }

    /// Recursively monomorphizes an expression, processing nested calls and substituting type nodes.
    fn monomorphize_expr(
        &mut self,
        expr: &Expr,
        env: &SubstEnv,
        local_vars: &HashMap<String, Ty>,
    ) -> Result<Expr, TypeError> {
        match expr {
            Expr::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            Expr::Var(name) => Ok(Expr::Var(name.clone())),
            Expr::Call { func, generic_args, args } => {
                // 1. Substitute any active generic parameters in nested generic_args using active environment
                let sub_generic_args: Vec<TypeSpec> =
                    generic_args.iter().map(|g| self.substitute_type_spec(g, env)).collect();

                // 2. Monomorphize sub-expression arguments
                let mut mono_args = Vec::new();
                for arg in args {
                    mono_args.push(self.monomorphize_expr(arg, env, local_vars)?);
                }

                // 3. Perform type inference / handoff for call target
                let (_, nested_key, nested_env) = self.checker.infer_call(
                    func,
                    &sub_generic_args,
                    &mono_args,
                    env,
                    local_vars,
                )?;

                // 4. Specialize target function under concrete nested environment
                let mangled_target = self.monomorphize_fn(&nested_key, &nested_env)?;

                Ok(Expr::Call { func: mangled_target, generic_args: vec![], args: mono_args })
            }
            Expr::Let { name, ty, init, body } => {
                let sub_ty = ty.as_ref().map(|t| self.substitute_type_spec(t, env));
                let mono_init = self.monomorphize_expr(init, env, local_vars)?;

                let init_ty = self.checker.infer_expr(&mono_init, env, local_vars)?;
                let mut inner_vars = local_vars.clone();
                inner_vars.insert(name.clone(), init_ty);

                let mono_body = self.monomorphize_expr(body, env, &inner_vars)?;

                Ok(Expr::Let {
                    name: name.clone(),
                    ty: sub_ty,
                    init: Box::new(mono_init),
                    body: Box::new(mono_body),
                })
            }
            Expr::Binary { op, lhs, rhs } => {
                let mono_lhs = self.monomorphize_expr(lhs, env, local_vars)?;
                let mono_rhs = self.monomorphize_expr(rhs, env, local_vars)?;
                Ok(Expr::Binary {
                    op: op.clone(),
                    lhs: Box::new(mono_lhs),
                    rhs: Box::new(mono_rhs),
                })
            }
            Expr::Unary { op, expr } => {
                let mono_expr = self.monomorphize_expr(expr, env, local_vars)?;
                Ok(Expr::Unary { op: op.clone(), expr: Box::new(mono_expr) })
            }
            Expr::Field { expr, field } => {
                let mono_expr = self.monomorphize_expr(expr, env, local_vars)?;
                Ok(Expr::Field { expr: Box::new(mono_expr), field: field.clone() })
            }
            Expr::Index { expr, index } => {
                let mono_expr = self.monomorphize_expr(expr, env, local_vars)?;
                let mono_index = self.monomorphize_expr(index, env, local_vars)?;
                Ok(Expr::Index { expr: Box::new(mono_expr), index: Box::new(mono_index) })
            }
            Expr::Tuple(elems) => {
                let mut mono_elems = Vec::new();
                for elem in elems {
                    mono_elems.push(self.monomorphize_expr(elem, env, local_vars)?);
                }
                Ok(Expr::Tuple(mono_elems))
            }
            Expr::Array(elems) => {
                let mut mono_elems = Vec::new();
                for elem in elems {
                    mono_elems.push(self.monomorphize_expr(elem, env, local_vars)?);
                }
                Ok(Expr::Array(mono_elems))
            }
            Expr::Range { start, end } => {
                let mono_start = self.monomorphize_expr(start, env, local_vars)?;
                let mono_end = self.monomorphize_expr(end, env, local_vars)?;
                Ok(Expr::Range { start: Box::new(mono_start), end: Box::new(mono_end) })
            }
            Expr::Match { expr, arms } => {
                let mono_expr = self.monomorphize_expr(expr, env, local_vars)?;
                let mut mono_arms = Vec::new();
                for (pat, arm_body) in arms {
                    let mono_arm = self.monomorphize_expr(arm_body, env, local_vars)?;
                    mono_arms.push((pat.clone(), mono_arm));
                }
                Ok(Expr::Match { expr: Box::new(mono_expr), arms: mono_arms })
            }
            Expr::Lambda { params, body } => {
                let mut sub_params = Vec::new();
                let mut lambda_vars = local_vars.clone();
                for (p_name, p_spec) in params {
                    let s_spec = self.substitute_type_spec(p_spec, env);
                    let p_ty = self.checker.lower_type_spec(&s_spec, env);
                    lambda_vars.insert(p_name.clone(), p_ty);
                    sub_params.push((p_name.clone(), s_spec));
                }
                let mono_body = self.monomorphize_expr(body, env, &lambda_vars)?;
                Ok(Expr::Lambda { params: sub_params, body: Box::new(mono_body) })
            }
            Expr::Interpolation(parts) => {
                let mut mono_parts = Vec::new();
                for part in parts {
                    mono_parts.push(self.monomorphize_expr(part, env, local_vars)?);
                }
                Ok(Expr::Interpolation(mono_parts))
            }
            Expr::Assign { target, value } => {
                let mono_target = self.monomorphize_expr(target, env, local_vars)?;
                let mono_val = self.monomorphize_expr(value, env, local_vars)?;
                Ok(Expr::Assign { target: Box::new(mono_target), value: Box::new(mono_val) })
            }
            Expr::Block(stmts) => {
                let mut mono_stmts = Vec::new();
                for stmt in stmts {
                    mono_stmts.push(self.monomorphize_expr(stmt, env, local_vars)?);
                }
                Ok(Expr::Block(mono_stmts))
            }
            Expr::Return(opt_expr) => {
                let mono_opt = if let Some(inner) = opt_expr {
                    Some(Box::new(self.monomorphize_expr(inner, env, local_vars)?))
                } else {
                    None
                };
                Ok(Expr::Return(mono_opt))
            }
        }
    }

    /// Substitutes generic parameters in a `TypeSpec` with concrete types from `SubstEnv`.
    fn substitute_type_spec(&mut self, spec: &TypeSpec, env: &SubstEnv) -> TypeSpec {
        match spec {
            TypeSpec::GenericParam(name) => {
                if let Some(concrete_ty) = env.get(name) {
                    self.ty_to_type_spec(concrete_ty)
                } else {
                    TypeSpec::GenericParam(name.clone())
                }
            }
            TypeSpec::Tuple(specs) => {
                TypeSpec::Tuple(specs.iter().map(|s| self.substitute_type_spec(s, env)).collect())
            }
            TypeSpec::Array(elem, len) => {
                TypeSpec::Array(Box::new(self.substitute_type_spec(elem, env)), *len)
            }
            TypeSpec::Range(elem) => {
                TypeSpec::Range(Box::new(self.substitute_type_spec(elem, env)))
            }
            TypeSpec::Fn(params, ret) => TypeSpec::Fn(
                params.iter().map(|p| self.substitute_type_spec(p, env)).collect(),
                Box::new(self.substitute_type_spec(ret, env)),
            ),
            TypeSpec::Struct(name, args) => TypeSpec::Struct(
                name.clone(),
                args.iter().map(|a| self.substitute_type_spec(a, env)).collect(),
            ),
            other => other.clone(),
        }
    }

    /// Converts an interned `Ty` back into a `TypeSpec`.
    fn ty_to_type_spec(&self, ty: Ty) -> TypeSpec {
        match self.checker.tcx.get(ty) {
            TyKind::Int => TypeSpec::Int,
            TyKind::Float => TypeSpec::Float,
            TyKind::Bool => TypeSpec::Bool,
            TyKind::Char => TypeSpec::Char,
            TyKind::Byte => TypeSpec::Byte,
            TyKind::String => TypeSpec::String,
            TyKind::Unit => TypeSpec::Unit,
            TyKind::GenericParam(name) => TypeSpec::GenericParam(name.clone()),
            TyKind::Tuple(tys) => {
                TypeSpec::Tuple(tys.iter().map(|&t| self.ty_to_type_spec(t)).collect())
            }
            TyKind::Array(elem, len) => {
                TypeSpec::Array(Box::new(self.ty_to_type_spec(*elem)), *len)
            }
            TyKind::Range(elem) => TypeSpec::Range(Box::new(self.ty_to_type_spec(*elem))),
            TyKind::Fn(params, ret) => TypeSpec::Fn(
                params.iter().map(|&p| self.ty_to_type_spec(p)).collect(),
                Box::new(self.ty_to_type_spec(*ret)),
            ),
            TyKind::Struct(name, args) => TypeSpec::Struct(
                name.clone(),
                args.iter().map(|&a| self.ty_to_type_spec(a)).collect(),
            ),
            _ => TypeSpec::Known(ty),
        }
    }
}
