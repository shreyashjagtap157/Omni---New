use std::collections::{HashMap, HashSet};

use crate::ast::{Expr, GenericFnDef, TypeSpec};
use crate::checker::{SpecializationKey, SubstEnv, TypeChecker, TypeError};
use crate::intern::{Ty, TyKind};

/// Output of monomorphizing a generic program: specialized non-generic function definitions.
#[derive(Debug, Clone)]
pub struct MonomorphizedProgram {
    pub functions: Vec<GenericFnDef>,
}

impl MonomorphizedProgram {
    /// Strict semantic gate before MIR: verifies that all monomorphized functions
    /// have 0 type parameters, 0 unsatisfied bounds, no unresolved generic calls,
    /// and no unresolved effect row variables.
    pub fn assert_concrete_for_mir(&self) -> Result<(), String> {
        for func in &self.functions {
            if !func.type_params.is_empty() {
                return Err(format!(
                    "MIR semantic gate violation: function `{}` has unresolved type parameters: {:?}",
                    func.name, func.type_params
                ));
            }
            if !func.bounds.is_empty() {
                return Err(format!(
                    "MIR semantic gate violation: function `{}` has unresolved trait bounds",
                    func.name
                ));
            }
            Self::verify_type_spec_concrete(&func.return_type, &func.name)?;
            for (_, param_type) in &func.params {
                Self::verify_type_spec_concrete(param_type, &func.name)?;
            }
            // Effect gate: no unresolved effect row variables
            if let Some(var) = func.effects.tail_var() {
                return Err(format!(
                    "MIR semantic gate violation: function `{}` has unresolved effect row variable `{}`",
                    func.name, var
                ));
            }
            Self::verify_expr_concrete(&func.body, &func.name)?;
        }
        Ok(())
    }

    fn verify_type_spec_concrete(spec: &TypeSpec, enclosing_fn: &str) -> Result<(), String> {
        match spec {
            TypeSpec::GenericParam(name) => Err(format!(
                "MIR semantic gate violation in {}: unresolved type parameter {} remains in a function signature",
                enclosing_fn, name
            )),
            TypeSpec::Known(ty) => Err(format!(
                "MIR semantic gate violation in {}: raw interned type handle {:?} remains in a function signature",
                enclosing_fn, ty
            )),
            TypeSpec::Tuple(items) => {
                for item in items {
                    Self::verify_type_spec_concrete(item, enclosing_fn)?;
                }
                Ok(())
            }
            TypeSpec::Array(elem, _) | TypeSpec::Range(elem) => {
                Self::verify_type_spec_concrete(elem, enclosing_fn)
            }
            TypeSpec::Reference { inner, .. } => {
                Self::verify_type_spec_concrete(inner, enclosing_fn)
            }
            TypeSpec::Fn(params, ret) => {
                for param in params {
                    Self::verify_type_spec_concrete(param, enclosing_fn)?;
                }
                Self::verify_type_spec_concrete(ret, enclosing_fn)
            }
            TypeSpec::Struct(_, args) | TypeSpec::Enum(_, args) | TypeSpec::TraitObject { args, .. } => {
                for arg in args {
                    Self::verify_type_spec_concrete(arg, enclosing_fn)?;
                }
                Ok(())
            }
            TypeSpec::Int
            | TypeSpec::Float
            | TypeSpec::Bool
            | TypeSpec::Char
            | TypeSpec::Byte
            | TypeSpec::String
            | TypeSpec::Unit
            | TypeSpec::Never => Ok(()),
        }
    }
    fn verify_expr_concrete(expr: &Expr, enclosing_fn: &str) -> Result<(), String> {
        match expr {
            Expr::Call { func: _, generic_args, args } => {
                if !generic_args.is_empty() {
                    return Err(format!(
                        "MIR semantic gate violation in `{enclosing_fn}`: call retains unresolved generic arguments"
                    ));
                }
                for a in args {
                    Self::verify_expr_concrete(a, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Let { init, body, .. } => {
                Self::verify_expr_concrete(init, enclosing_fn)?;
                Self::verify_expr_concrete(body, enclosing_fn)
            }
            Expr::Binary { lhs, rhs, .. } => {
                Self::verify_expr_concrete(lhs, enclosing_fn)?;
                Self::verify_expr_concrete(rhs, enclosing_fn)
            }
            Expr::Unary { expr, .. } => Self::verify_expr_concrete(expr, enclosing_fn),
            Expr::Field { expr, .. } => Self::verify_expr_concrete(expr, enclosing_fn),
            Expr::Index { expr, .. } => Self::verify_expr_concrete(expr, enclosing_fn),
            Expr::Tuple(elems) | Expr::Array(elems) => {
                for e in elems {
                    Self::verify_expr_concrete(e, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Range { start, end, .. } => {
                Self::verify_expr_concrete(start, enclosing_fn)?;
                Self::verify_expr_concrete(end, enclosing_fn)
            }
            Expr::If { condition, then_branch, else_branch } => {
                Self::verify_expr_concrete(condition, enclosing_fn)?;
                Self::verify_expr_concrete(then_branch, enclosing_fn)?;
                if let Some(e) = else_branch {
                    Self::verify_expr_concrete(e, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Cast { expr, ty } => {
                Self::verify_expr_concrete(expr, enclosing_fn)?;
                Self::verify_type_spec_concrete(ty, enclosing_fn)
            }
            Expr::Loop { body, .. } => Self::verify_expr_concrete(body, enclosing_fn),
            Expr::While { condition, body, .. } => {
                Self::verify_expr_concrete(condition, enclosing_fn)?;
                Self::verify_expr_concrete(body, enclosing_fn)
            }
            Expr::For { iterable, body, .. } => {
                Self::verify_expr_concrete(iterable, enclosing_fn)?;
                Self::verify_expr_concrete(body, enclosing_fn)
            }

            Expr::Break { value, .. } => {
                if let Some(v) = value {
                    Self::verify_expr_concrete(v, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Continue { .. } => Ok(()),
            Expr::Match { expr, arms } => {
                Self::verify_expr_concrete(expr, enclosing_fn)?;
                for arm in arms {
                    if let Some(g) = &arm.guard {
                        Self::verify_expr_concrete(g, enclosing_fn)?;
                    }
                    Self::verify_expr_concrete(&arm.body, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Lambda { body, .. } => Self::verify_expr_concrete(body, enclosing_fn),
            Expr::Interpolation(parts) => {
                for p in parts {
                    Self::verify_expr_concrete(p, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Assign { target, value } => {
                Self::verify_expr_concrete(target, enclosing_fn)?;
                Self::verify_expr_concrete(value, enclosing_fn)
            }
            Expr::CompoundAssign { target, value, .. } => {
                Self::verify_expr_concrete(target, enclosing_fn)?;
                Self::verify_expr_concrete(value, enclosing_fn)
            },
            Expr::Block(stmts) => {
                for s in stmts {
                    Self::verify_expr_concrete(s, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Return(opt_e) => {
                if let Some(e) = opt_e {
                    Self::verify_expr_concrete(e, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Struct { generic_args, fields, .. } => {
                for g in generic_args {
                    Self::verify_type_spec_concrete(g, enclosing_fn)?;
                }
                for (_, value) in fields {
                    Self::verify_expr_concrete(value, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::EnumVariant { generic_args, args, .. } => {
                for g in generic_args {
                    Self::verify_type_spec_concrete(g, enclosing_fn)?;
                }
                for a in args {
                    Self::verify_expr_concrete(a, enclosing_fn)?;
                }
                Ok(())
            }
            Expr::Literal(_) | Expr::Var(_) => Ok(()),
        }
    }
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
        let (_, key, subst_env, _callee_effects) =
            self.checker.infer_call(fn_name, explicit_args, arg_exprs, &env, &local_vars)?;

        self.monomorphize_fn(&key, &subst_env)?;

        let mut functions: Vec<_> = self.cache.values().cloned().collect();
        functions.sort_by(|a, b| a.name.cmp(&b.name));
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
            bounds: vec![],
            params: spec_params,
            return_type: spec_ret,
            effects: fn_def.effects.clone(),
            capabilities: fn_def.capabilities.clone(),
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
                let (_, nested_key, nested_env, _nested_effects) = self.checker.infer_call(
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
            Expr::Let { pattern, ty, init, body } => {
                let sub_ty = ty.as_ref().map(|t| self.substitute_type_spec(t, env));
                let mono_init = self.monomorphize_expr(init, env, local_vars)?;
                let init_ty = self.checker.infer_expr(&mono_init, env, local_vars)?;
                let mut inner_vars = local_vars.clone();
                self.checker.bind_pattern(pattern, init_ty, &mut inner_vars)?;
                let mono_body = self.monomorphize_expr(body, env, &inner_vars)?;
                Ok(Expr::Let {
                    pattern: pattern.clone(),
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
            Expr::EnumVariant { enum_name, variant, generic_args, args } => {
                let generic_args = generic_args
                    .iter()
                    .map(|a| self.substitute_type_spec(a, env))
                    .collect::<Vec<_>>();
                let args = args
                    .iter()
                    .map(|arg| self.monomorphize_expr(arg, env, local_vars))
                    .collect::<Result<Vec<_>, TypeError>>()?;
                Ok(Expr::EnumVariant {
                    enum_name: enum_name.clone(),
                    variant: variant.clone(),
                    generic_args,
                    args,
                })
            }
            Expr::Struct { name, generic_args, fields } => {
                let args = generic_args
                    .iter()
                    .map(|a| self.substitute_type_spec(a, env))
                    .collect::<Vec<_>>();
                let fields = fields
                    .iter()
                    .map(|(field, value)| {
                        Ok((
                            field.clone(),
                            self.monomorphize_expr(value, env, local_vars)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, TypeError>>()?;
                Ok(Expr::Struct {
                    name: name.clone(),
                    generic_args: args,
                    fields,
                })
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
            Expr::Range { start, end, inclusive } => {
                let mono_start = self.monomorphize_expr(start, env, local_vars)?;
                let mono_end = self.monomorphize_expr(end, env, local_vars)?;
                Ok(Expr::Range { start: Box::new(mono_start), end: Box::new(mono_end), inclusive: *inclusive })
            }
            Expr::If { condition, then_branch, else_branch } => {
                let mono_condition = self.monomorphize_expr(condition, env, local_vars)?;
                let mono_then = self.monomorphize_expr(then_branch, env, local_vars)?;
                let mono_else = if let Some(e) = else_branch {
                    Some(Box::new(self.monomorphize_expr(e, env, local_vars)?))
                } else {
                    None
                };
                Ok(Expr::If {
                    condition: Box::new(mono_condition),
                    then_branch: Box::new(mono_then),
                    else_branch: mono_else,
                })
            }
            Expr::Cast { expr, ty } => {
                let mono_expr = self.monomorphize_expr(expr, env, local_vars)?;
                let mono_ty = self.substitute_type_spec(ty, env);
                Ok(Expr::Cast { expr: Box::new(mono_expr), ty: mono_ty })
            }
            Expr::Loop { label, body } => {
                Ok(Expr::Loop {
                    label: label.clone(),
                    body: Box::new(self.monomorphize_expr(body, env, local_vars)?),
                })
            }
            Expr::While { label, condition, body } => {
                Ok(Expr::While {
                    label: label.clone(),
                    condition: Box::new(self.monomorphize_expr(condition, env, local_vars)?),
                    body: Box::new(self.monomorphize_expr(body, env, local_vars)?),
                })
            }
            Expr::For { label, pattern, iterable, body } => {
                Ok(Expr::For {
                    label: label.clone(),
                    pattern: pattern.clone(),
                    iterable: Box::new(self.monomorphize_expr(iterable, env, local_vars)?),
                    body: Box::new(self.monomorphize_expr(body, env, local_vars)?),
                })
            }
            Expr::Break { label, value } => Ok(Expr::Break {
                label: label.clone(),
                value: value
                    .as_ref()
                    .map(|v| self.monomorphize_expr(v, env, local_vars))
                    .transpose()?
                    .map(Box::new),
            }),
            Expr::Continue { label } => Ok(Expr::Continue { label: label.clone() }),
            Expr::Match { expr, arms } => {
                let mono_expr = self.monomorphize_expr(expr, env, local_vars)?;
                let mut mono_arms = Vec::new();
                for arm in arms {
                    let mono_guard = if let Some(g) = &arm.guard {
                        Some(self.monomorphize_expr(g, env, local_vars)?)
                    } else {
                        None
                    };
                    let mono_body = self.monomorphize_expr(&arm.body, env, local_vars)?;
                    mono_arms.push(crate::ast::MatchArm {
                        pattern: arm.pattern.clone(),
                        guard: mono_guard,
                        body: mono_body,
                    });
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
            Expr::CompoundAssign { op, target, value } => {
                let mono_target = self.monomorphize_expr(target, env, local_vars)?;
                let mono_val = self.monomorphize_expr(value, env, local_vars)?;
                Ok(Expr::CompoundAssign {
                    op: *op,
                    target: Box::new(mono_target),
                    value: Box::new(mono_val),
                })
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
            TypeSpec::Reference { lifetime, mutable, inner } => TypeSpec::Reference {
                lifetime: lifetime.clone(),
                mutable: *mutable,
                inner: Box::new(self.substitute_type_spec(inner, env)),
            },
            TypeSpec::Fn(params, ret) => TypeSpec::Fn(
                params.iter().map(|p| self.substitute_type_spec(p, env)).collect(),
                Box::new(self.substitute_type_spec(ret, env)),
            ),
            TypeSpec::Struct(name, args) => TypeSpec::Struct(
                name.clone(),
                args.iter().map(|a| self.substitute_type_spec(a, env)).collect(),
            ),
            TypeSpec::Enum(name, args) => TypeSpec::Enum(
                name.clone(),
                args.iter().map(|a| self.substitute_type_spec(a, env)).collect(),
            ),
            TypeSpec::Never => TypeSpec::Never,
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
            TyKind::Never => TypeSpec::Never,
            TyKind::GenericParam(name) => TypeSpec::GenericParam(name.clone()),
            TyKind::Tuple(tys) => {
                TypeSpec::Tuple(tys.iter().map(|&t| self.ty_to_type_spec(t)).collect())
            }
            TyKind::Array(elem, len) => {
                TypeSpec::Array(Box::new(self.ty_to_type_spec(*elem)), *len)
            }
            TyKind::Range(elem) => TypeSpec::Range(Box::new(self.ty_to_type_spec(*elem))),
            TyKind::Reference { lifetime, mutable, inner } => TypeSpec::Reference {
                lifetime: lifetime.clone(),
                mutable: *mutable,
                inner: Box::new(self.ty_to_type_spec(*inner)),
            },
            TyKind::Fn(params, ret) => TypeSpec::Fn(
                params.iter().map(|&p| self.ty_to_type_spec(p)).collect(),
                Box::new(self.ty_to_type_spec(*ret)),
            ),
            TyKind::Struct(name, args) => TypeSpec::Struct(
                name.clone(),
                args.iter().map(|&a| self.ty_to_type_spec(a)).collect(),
            ),
            TyKind::Enum(name, args) => TypeSpec::Enum(
                name.clone(),
                args.iter().map(|&a| self.ty_to_type_spec(a)).collect(),
            ),
            _ => TypeSpec::Known(ty),
        }
    }
}
