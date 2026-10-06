//! Canonical High-Level Intermediate Representation for Omni.
//!
//! HIR is the semantic boundary between the validated frontend and MIR.
//! This first implementation preserves the already-semantic expression
//! vocabulary while making the stage explicit and fail-closed.

use std::collections::{HashMap, HashSet};

use omni_effects::{Capability, EffectRow};
use omni_types::ast::{Expr, GenericFnDef, StructDef, TypeSpec};
use omni_types::monomorph::MonomorphizedProgram;

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub functions: Vec<HirFunction>,
    pub struct_defs: HashMap<String, StructDef>,
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<(String, TypeSpec)>,
    pub return_type: TypeSpec,
    pub effects: EffectRow,
    pub capabilities: Vec<Capability>,
    pub body: HirExpr,
}

#[derive(Debug, Clone)]
pub struct HirExpr {
    pub expression: Expr,
}

impl HirExpr {
    fn from_expr(expr: Expr, function: &str) -> Result<Self, String> {
        validate_expr(&expr, function)?;
        Ok(Self { expression: expr })
    }

    pub fn as_expr(&self) -> &Expr {
        &self.expression
    }

    pub fn into_expr(self) -> Expr {
        self.expression
    }
}

impl HirProgram {
    pub fn from_monomorphized_program(
        program: &MonomorphizedProgram,
        struct_defs: HashMap<String, StructDef>,
    ) -> Result<Self, String> {
        program.assert_concrete_for_mir()?;
        let mut functions = Vec::with_capacity(program.functions.len());
        let mut names = HashSet::new();

        for function in &program.functions {
            if !names.insert(function.name.clone()) {
                return Err(format!(
                    "HIR construction error: duplicate function '{}'",
                    function.name
                ));
            }

            functions.push(HirFunction {
                name: function.name.clone(),
                type_params: function.type_params.clone(),
                params: function.params.clone(),
                return_type: function.return_type.clone(),
                effects: function.effects.clone(),
                capabilities: function.capabilities.clone(),
                body: HirExpr::from_expr(function.body.clone(), &function.name)?,
            });
        }

        Ok(Self { functions, struct_defs })
    }

    pub fn to_monomorphized_program(&self) -> MonomorphizedProgram {
        MonomorphizedProgram {
            functions: self
                .functions
                .iter()
                .map(|function| GenericFnDef {
                    name: function.name.clone(),
                    type_params: function.type_params.clone(),
                    bounds: Vec::new(),
                    params: function.params.clone(),
                    return_type: function.return_type.clone(),
                    effects: function.effects.clone(),
                    capabilities: function.capabilities.clone(),
                    body: function.body.expression.clone(),
                })
                .collect(),
        }
    }
}

fn validate_expr(expr: &Expr, function: &str) -> Result<(), String> {
    use Expr::*;

    match expr {
        MethodCall { .. } => Err(format!(
            "HIR construction error in '{}': unresolved method call reached HIR",
            function
        )),
        Call { args, generic_args, .. } => {
            if !generic_args.is_empty() {
                return Err(format!(
                    "HIR construction error in '{}': generic call arguments remain",
                    function
                ));
            }

            for arg in args {
                validate_expr(arg, function)?;
            }
        }
        Let { init, body, .. } => {
            validate_expr(init, function)?;
            validate_expr(body, function)?;
        }
        Binary { lhs, rhs, .. } => {
            validate_expr(lhs, function)?;
            validate_expr(rhs, function)?;
        }
        Unary { expr, .. } | Field { expr, .. } | Cast { expr, .. } => {
            validate_expr(expr, function)?;
        }
        Index { expr, index } => {
            validate_expr(expr, function)?;
            validate_expr(index, function)?;
        }
        Struct { fields, generic_args, .. } => {
            if !generic_args.is_empty() {
                return Err(format!(
                    "HIR construction error in '{}': generic struct arguments remain",
                    function
                ));
            }

            for (_, value) in fields {
                validate_expr(value, function)?;
            }
        }
        EnumVariant { args, generic_args, .. } => {
            if !generic_args.is_empty() {
                return Err(format!(
                    "HIR construction error in '{}': generic enum arguments remain",
                    function
                ));
            }

            for arg in args {
                validate_expr(arg, function)?;
            }
        }
        Tuple(elems) | Array(elems) | Block(elems) => {
            for elem in elems {
                validate_expr(elem, function)?;
            }
        }
        Range { start, end, .. } => {
            validate_expr(start, function)?;
            validate_expr(end, function)?;
        }
        Match { expr, arms } => {
            validate_expr(expr, function)?;

            for arm in arms {
                if let Some(guard) = &arm.guard {
                    validate_expr(guard, function)?;
                }
                validate_expr(&arm.body, function)?;
            }
        }
        If { condition, then_branch, else_branch } => {
            validate_expr(condition, function)?;
            validate_expr(then_branch, function)?;

            if let Some(branch) = else_branch {
                validate_expr(branch, function)?;
            }
        }
        Lambda { body, .. } | UnsafeBlock { body } | Loop { body, .. } => {
            validate_expr(body, function)?;
        }
        While { condition, body, .. } => {
            validate_expr(condition, function)?;
            validate_expr(body, function)?;
        }
        For { iterable, body, .. } => {
            validate_expr(iterable, function)?;
            validate_expr(body, function)?;
        }
        Interpolation(parts) => {
            for part in parts {
                validate_expr(part, function)?;
            }
        }
        Assign { target, value } | CompoundAssign { target, value, .. } => {
            validate_expr(target, function)?;
            validate_expr(value, function)?;
        }
        Break { value, .. } | Return(value) => {
            if let Some(value) = value {
                validate_expr(value, function)?;
            }
        }
        Continue { .. } | Literal(_) | Var(_) => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_types::ast::Lit;

    fn concrete_function(name: &str, body: Expr) -> GenericFnDef {
        GenericFnDef {
            name: name.to_string(),
            type_params: Vec::new(),
            bounds: Vec::new(),
            params: Vec::new(),
            return_type: TypeSpec::Int,
            effects: EffectRow::pure(),
            capabilities: Vec::new(),
            body,
        }
    }

    #[test]
    fn constructs_canonical_hir_from_concrete_program() {
        let program = MonomorphizedProgram {
            functions: vec![concrete_function("main", Expr::Literal(Lit::Int(7)))],
        };

        let hir = HirProgram::from_monomorphized_program(&program, HashMap::new())
            .expect("concrete program must construct HIR");

        assert_eq!(hir.functions.len(), 1);
        assert_eq!(hir.functions[0].name, "main");
        assert!(matches!(hir.functions[0].body.as_expr(), Expr::Literal(Lit::Int(7)));

        let round_trip = hir.to_monomorphized_program();
        assert_eq!(round_trip.functions, program.functions);
    }

    #[test]
    fn rejects_unresolved_method_call_at_hir_boundary() {
        let body = Expr::MethodCall {
            receiver: Box::new(Expr::Literal(Lit::Int(1))),
            method: "value".to_string(),
            generic_args: Vec::new(),
            args: Vec::new(),
        };
        let program = MonomorphizedProgram { functions: vec![concrete_function("main", body)] };

        let error = HirProgram::from_monomorphized_program(&program, HashMap::new())
            .expect_err("method calls must not cross the HIR boundary");

        assert!(error.contains("unresolved method call reached HIR"));
    }

    #[test]
    fn rejects_duplicate_function_names() {
        let program = MonomorphizedProgram {
            functions: vec![
                concrete_function("main", Expr::Literal(Lit::Int(1))),
                concrete_function("main", Expr::Literal(Lit::Int(2))),
            ],
        };

        let error = HirProgram::from_monomorphized_program(&program, HashMap::new())
            .expect_err("duplicate functions must fail HIR construction");

        assert!(error.contains("duplicate function 'main'"));
    }
}
