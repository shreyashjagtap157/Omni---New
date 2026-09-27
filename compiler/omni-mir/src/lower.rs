//! AST-to-MIR Lowering Engine for Omni.
//! Translates validated, monomorphized expressions into canonical Mid-Level IR.

use crate::ir::Body;
use index_vec::IndexVec;
use omni_types::intern::{Ty, TyCtxt, TyKind};
use omni_types::monomorph::MonomorphizedProgram;
use std::collections::HashMap;

pub struct LoweringContext {
    body: Body,
}

impl LoweringContext {
    pub fn new() -> Self {
        Self {
            body: Body {
                blocks: IndexVec::new(),
                local_decls: IndexVec::new(),
            },
        }
    }

    /// Lowers a concrete MonomorphizedProgram into typed MIR.
    ///
    /// The concrete-program gate runs before any MIR is produced. Every materialized
    /// local receives an explicit Ty; Unit is represented by the absence of a value.
    pub fn lower_monomorphized_program(
        &mut self,
        prog: &MonomorphizedProgram,
    ) -> Result<crate::ir::MirProgram, String> {
        prog.assert_concrete_for_mir()?;

        let mut tcx = TyCtxt::new();
        let subst = omni_types::checker::SubstEnv::new();
        let mut fn_sigs: HashMap<String, (Vec<Ty>, Ty)> = HashMap::new();

        for func in &prog.functions {
            let params = func
                .params
                .iter()
                .map(|(_, spec)| tcx.lower_type_spec(spec, &subst))
                .collect::<Vec<_>>();
            let ret = tcx.lower_type_spec(&func.return_type, &subst);
            if fn_sigs.insert(func.name.clone(), (params, ret)).is_some() {
                return Err(format!(
                    "MIR lowering error: duplicate monomorphized function '{}'",
                    func.name
                ));
            }
        }

        let mut mir_functions = Vec::with_capacity(prog.functions.len());

        for func in &prog.functions {
            let mut blocks = IndexVec::new();
            let mut local_decls = IndexVec::new();
            let mut scope = HashMap::new();

            let ret_ty = tcx.lower_type_spec(&func.return_type, &subst);
            let return_place = local_decls.push(crate::ir::LocalDecl {
                name: Some("_return".to_string()),
                ty: Some(ret_ty),
            });

            let mut param_locals = Vec::with_capacity(func.params.len());
            for (p_name, p_type) in &func.params {
                let p_ty = tcx.lower_type_spec(p_type, &subst);
                let p_local = local_decls.push(crate::ir::LocalDecl {
                    name: Some(p_name.clone()),
                    ty: Some(p_ty),
                });
                param_locals.push(p_local);
                scope.insert(p_name.clone(), p_local);
            }

            let mut builder = FnMirBuilder {
                tcx: &mut tcx,
                subst: &subst,
                local_decls: &mut local_decls,
                blocks: &mut blocks,
                scope,
                fn_sigs: &fn_sigs,
                return_ty: ret_ty,
                current_block: None,
            };

            let entry_block = builder.new_block();
            builder.current_block = Some(entry_block);

            let result = builder.lower_expr(&func.body)?;

            if let Some(curr) = builder.current_block {
                match result {
                    Some((operand, value_ty)) => {
                        if value_ty != ret_ty {
                            return Err(format!(
                                "MIR lowering error in '{}': function body has type {:?}, expected {:?}",
                                func.name, value_ty, ret_ty
                            ));
                        }
                        let ret_p = crate::ir::Place { local: return_place };
                        builder.blocks[curr].statements.push(crate::ir::Statement::Assign(
                            ret_p,
                            crate::ir::Rvalue::Use(operand),
                        ));
                    }
                    None => {
                        let unit_ty = builder.tcx.intern(TyKind::Unit);
                        if ret_ty != unit_ty {
                            return Err(format!(
                                "MIR lowering error in '{}': function body has no value, expected {:?}",
                                func.name, ret_ty
                            ));
                        }
                    }
                }

                if builder.blocks[curr].terminator.is_none() {
                    builder.blocks[curr].terminator = Some(crate::ir::Terminator::Return);
                }
            }

            let body = Body { blocks, local_decls };
            mir_functions.push(crate::ir::MirFunction {
                name: func.name.clone(),
                params: param_locals,
                return_place,
                return_type: func.return_type.clone(),
                body,
            });
        }

        let mir_prog = crate::ir::MirProgram { functions: mir_functions };
        if let Some(first_fn) = mir_prog.functions.first() {
            self.body = first_fn.body.clone();
        }
        Ok(mir_prog)
    }

    /// Legacy Stage-1 snippet probe. It remains isolated from the typed production path.
    pub fn lower_snippet(&mut self, source: &str) -> Result<Body, String> {
        if source.contains('+')
            || source.contains('-')
            || source.contains('*')
            || source.contains('/')
        {
            Ok(self.body.clone())
        } else {
            Err("Unsupported expression form for Stage-1 lowering".into())
        }
    }
}

struct FnMirBuilder<'a> {
    tcx: &'a mut TyCtxt,
    subst: &'a omni_types::checker::SubstEnv,
    local_decls: &'a mut IndexVec<crate::ir::Local, crate::ir::LocalDecl>,
    blocks: &'a mut IndexVec<crate::ir::BasicBlock, crate::ir::BlockData>,
    scope: HashMap<String, crate::ir::Local>,
    fn_sigs: &'a HashMap<String, (Vec<Ty>, Ty)>,
    return_ty: Ty,
    current_block: Option<crate::ir::BasicBlock>,
}

impl<'a> FnMirBuilder<'a> {
    fn new_block(&mut self) -> crate::ir::BasicBlock {
        self.blocks.push(crate::ir::BlockData {
            statements: Vec::new(),
            terminator: None,
        })
    }

    fn new_temp(&mut self, name: Option<String>, ty: Ty) -> crate::ir::Local {
        self.local_decls
            .push(crate::ir::LocalDecl { name, ty: Some(ty) })
    }

    fn local_ty(&self, local: crate::ir::Local) -> Result<Ty, String> {
        self.local_decls[local]
            .ty
            .ok_or_else(|| format!("MIR lowering error: local {:?} has no type", local))
    }

    fn literal_ty(&mut self, lit: &omni_types::ast::Lit) -> Ty {
        match lit {
            omni_types::ast::Lit::Int(_) => self.tcx.intern(TyKind::Int),
            omni_types::ast::Lit::Float(_) => self.tcx.intern(TyKind::Float),
            omni_types::ast::Lit::Bool(_) => self.tcx.intern(TyKind::Bool),
            omni_types::ast::Lit::Char(_) => self.tcx.intern(TyKind::Char),
            omni_types::ast::Lit::Byte(_) => self.tcx.intern(TyKind::Byte),
            omni_types::ast::Lit::String(_) => self.tcx.intern(TyKind::String),
        }
    }

    fn lower_expr(
        &mut self,
        expr: &omni_types::ast::Expr,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let curr_block = self.current_block.ok_or_else(|| {
            "MIR lowering error: expression evaluated after control flow terminated".to_string()
        })?;

        match expr {
            omni_types::ast::Expr::Literal(lit) => {
                let ty = self.literal_ty(lit);
                Ok(Some((
                    crate::ir::Operand::Constant(crate::ir::Constant::Lit(lit.clone())),
                    ty,
                )))
            }
            omni_types::ast::Expr::Var(name) => {
                let local = self
                    .scope
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("MIR lowering error: undefined variable '{}'", name))?;
                let ty = self.local_ty(local)?;
                Ok(Some((
                    crate::ir::Operand::Copy(crate::ir::Place { local }),
                    ty,
                )))
            }
            omni_types::ast::Expr::Binary { op, lhs, rhs } => {
                let (lhs_op, lhs_ty) = self
                    .lower_expr(lhs)?
                    .ok_or_else(|| "MIR lowering error: left operand is Unit".to_string())?;
                let (rhs_op, rhs_ty) = self
                    .lower_expr(rhs)?
                    .ok_or_else(|| "MIR lowering error: right operand is Unit".to_string())?;
                if lhs_ty != rhs_ty {
                    return Err(format!(
                        "MIR lowering error: binary operands have incompatible types {:?} and {:?}",
                        lhs_ty, rhs_ty
                    ));
                }
                let mir_op = match op {
                    omni_types::ast::BinOp::Add => crate::ir::BinOp::Add,
                    omni_types::ast::BinOp::Sub => crate::ir::BinOp::Sub,
                    omni_types::ast::BinOp::Mul => crate::ir::BinOp::Mul,
                    omni_types::ast::BinOp::Div => crate::ir::BinOp::Div,
                    omni_types::ast::BinOp::Eq => crate::ir::BinOp::Eq,
                    omni_types::ast::BinOp::Ne => crate::ir::BinOp::Ne,
                    omni_types::ast::BinOp::Lt => crate::ir::BinOp::Lt,
                    omni_types::ast::BinOp::Gt => crate::ir::BinOp::Gt,
                };
                let temp_local = self.new_temp(Some("_bin_tmp".to_string()), lhs_ty);
                let place = crate::ir::Place { local: temp_local };
                self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                    place,
                    crate::ir::Rvalue::BinaryOp(mir_op, lhs_op, rhs_op),
                ));
                Ok(Some((crate::ir::Operand::Copy(place), lhs_ty)))
            }
            omni_types::ast::Expr::Unary { op, expr } => {
                let (inner_op, inner_ty) = self
                    .lower_expr(expr)?
                    .ok_or_else(|| "MIR lowering error: unary operand is Unit".to_string())?;
                let mir_op = match op {
                    omni_types::ast::UnOp::Neg => crate::ir::UnOp::Neg,
                    omni_types::ast::UnOp::Not => crate::ir::UnOp::Not,
                };
                let temp_local = self.new_temp(Some("_un_tmp".to_string()), inner_ty);
                let place = crate::ir::Place { local: temp_local };
                self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                    place,
                    crate::ir::Rvalue::UnaryOp(mir_op, inner_op),
                ));
                Ok(Some((crate::ir::Operand::Copy(place), inner_ty)))
            }
            omni_types::ast::Expr::Let { name, ty, init, body } => {
                let (init_op, init_ty) = self
                    .lower_expr(init)?
                    .ok_or_else(|| {
                        "MIR lowering error: Unit-valued let initializers are not materialized"
                            .to_string()
                    })?;
                let var_ty = if let Some(spec) = ty {
                    self.tcx.lower_type_spec(spec, self.subst)
                } else {
                    init_ty
                };
                if var_ty != init_ty {
                    return Err(format!(
                        "MIR lowering error: let binding '{}' declares {:?} but initializer has {:?}",
                        name, var_ty, init_ty
                    ));
                }
                let saved_scope = self.scope.clone();
                let var_local = self.new_temp(Some(name.clone()), var_ty);
                let place = crate::ir::Place { local: var_local };
                self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                    place,
                    crate::ir::Rvalue::Use(init_op),
                ));
                self.scope.insert(name.clone(), var_local);
                let result = self.lower_expr(body);
                self.scope = saved_scope;
                result
            }
            omni_types::ast::Expr::Block(exprs) => {
                let mut last = None;
                for expr in exprs {
                    last = self.lower_expr(expr)?;
                }
                Ok(last)
            }
            omni_types::ast::Expr::Call {
                func,
                generic_args: _,
                args,
            } => {
                let (param_tys, ret_ty) = self.fn_sigs.get(func).cloned().ok_or_else(|| {
                    format!(
                        "MIR lowering error: call target '{}' is not present in monomorphized program",
                        func
                    )
                })?;
                if args.len() != param_tys.len() {
                    return Err(format!(
                        "MIR lowering error in call '{}': expected {} arguments, found {}",
                        func,
                        param_tys.len(),
                        args.len()
                    ));
                }

                let mut arg_ops = Vec::with_capacity(args.len());
                for (arg, expected_ty) in args.iter().zip(param_tys.iter()) {
                    let (arg_op, arg_ty) = self
                        .lower_expr(arg)?
                        .ok_or_else(|| "MIR lowering error: Unit-valued call argument".to_string())?;
                    if arg_ty != *expected_ty {
                        return Err(format!(
                            "MIR lowering error in call '{}': expected argument type {:?}, found {:?}",
                            func, expected_ty, arg_ty
                        ));
                    }
                    arg_ops.push(arg_op);
                }

                let next_block = self.new_block();
                let destination = if ret_ty == self.tcx.intern(TyKind::Unit) {
                    None
                } else {
                    let dest_local = self.new_temp(Some("_call_dest".to_string()), ret_ty);
                    Some(crate::ir::Place { local: dest_local })
                };

                self.blocks[curr_block].terminator = Some(crate::ir::Terminator::Call {
                    func: crate::ir::Operand::Constant(crate::ir::Constant::FnRef(func.clone())),
                    args: arg_ops,
                    destination,
                    target: next_block,
                    cleanup: None,
                });
                self.current_block = Some(next_block);

                match destination {
                    Some(place) => Ok(Some((crate::ir::Operand::Copy(place), ret_ty))),
                    None => Ok(None),
                }
            }
            omni_types::ast::Expr::Return(opt_expr) => {
                let ret_result = if let Some(inner) = opt_expr {
                    let value = self.lower_expr(inner)?.ok_or_else(|| {
                        "MIR lowering error: explicit return expression evaluates to Unit"
                            .to_string()
                    })?;
                    if value.1 != self.return_ty {
                        return Err(format!(
                            "MIR lowering error: return expression has type {:?}, expected {:?}",
                            value.1, self.return_ty
                        ));
                    }
                    Some(value)
                } else {
                    let unit_ty = self.tcx.intern(TyKind::Unit);
                    if self.return_ty != unit_ty {
                        return Err(format!(
                            "MIR lowering error: bare return requires Unit return type, found {:?}",
                            self.return_ty
                        ));
                    }
                    None
                };

                if let Some((operand, _)) = &ret_result {
                    let ret_p = crate::ir::Place {
                        local: crate::ir::Local::from_usize(0),
                    };
                    self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                        ret_p,
                        crate::ir::Rvalue::Use(operand.clone()),
                    ));
                }
                self.blocks[curr_block].terminator = Some(crate::ir::Terminator::Return);
                self.current_block = None;
                Ok(ret_result)
            }
            unsupported => Err(format!(
                "Unsupported AST expression form for MIR lowering: {:?}",
                unsupported
            )),
        }
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_types::ast::{Expr, GenericFnDef, Lit, TypeSpec};

    #[test]
    fn test_mir_lowering_concrete_gate_pass() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "main_spec".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Block(vec![]),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_ok(), "Concrete Unit program must lower successfully");
    }

    #[test]
    fn test_mir_lowering_concrete_gate_fails_on_unresolved_generic() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unspecialized".to_string(),
                type_params: vec!["T".to_string()],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Literal(Lit::Int(42)),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_err(), "Program with unresolved type parameters must fail closed at MIR lowering gate");
        assert!(res.unwrap_err().contains("unresolved type parameters"));
    }

    #[test]
    fn test_mir_lowering_concrete_gate_fails_on_undischarged_bounds() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unresolved_bound".to_string(),
                type_params: vec![],
                bounds: vec![(
                    "T".to_string(),
                    omni_types::ast::TraitBound::Positive("Display".to_string()),
                )],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Literal(Lit::Int(42)),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_err(), "Program with undischarged trait bounds must fail closed at MIR lowering gate");
        assert!(res.unwrap_err().contains("unresolved trait bounds"));
    }

    #[test]
    fn test_mir_lowering_semantic_binary_and_let_expressions() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "add_let".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![("x".to_string(), TypeSpec::Int)],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Let {
                    name: "y".to_string(),
                    ty: None,
                    init: Box::new(Expr::Binary {
                        op: omni_types::ast::BinOp::Add,
                        lhs: Box::new(Expr::Var("x".to_string())),
                        rhs: Box::new(Expr::Literal(Lit::Int(2))),
                    }),
                    body: Box::new(Expr::Var("y".to_string())),
                },
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog).unwrap();
        let func = &res.functions[0];
        assert_eq!(func.params.len(), 1);
        assert!(!func.body.blocks.is_empty());
        assert!(func.body.local_decls.iter().all(|decl| decl.ty.is_some()));
    }

    #[test]
    fn test_mir_lowering_call_uses_callee_signature() {
        let mut ctx = LoweringContext::new();
        let callee = GenericFnDef {
            name: "inc".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![("x".to_string(), TypeSpec::Int)],
            return_type: TypeSpec::Int,
            effects: omni_effects::EffectRow::default(),
            capabilities: vec![],
            body: Expr::Var("x".to_string()),
        };
        let caller = GenericFnDef {
            name: "main_spec".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: TypeSpec::Int,
            effects: omni_effects::EffectRow::default(),
            capabilities: vec![],
            body: Expr::Call {
                func: "inc".to_string(),
                generic_args: vec![],
                args: vec![Expr::Literal(Lit::Int(41))],
            },
        };
        let prog = MonomorphizedProgram { functions: vec![callee, caller] };

        let res = ctx.lower_monomorphized_program(&prog).unwrap();
        let caller_mir = res
            .functions
            .iter()
            .find(|f| f.name == "main_spec")
            .unwrap();
        assert!(matches!(
            caller_mir.body.blocks[0].terminator,
            Some(crate::ir::Terminator::Call { destination: Some(_), .. })
        ));
    }

    #[test]
    fn test_mir_lowering_rejects_return_type_mismatch() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "bad".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Literal(Lit::Bool(true)),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("function body has type"));
    }

    #[test]
    fn test_mir_lowering_bare_return_requires_unit() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unit_return".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Return(None),
            }],
        };

        assert!(ctx.lower_monomorphized_program(&prog).is_ok());
    }

    #[test]
    fn test_mir_lowering_unsupported_expression_fails_explicitly() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unsupported".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Lambda {
                    params: vec![],
                    body: Box::new(Expr::Literal(Lit::Int(1))),
                },
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_err(), "Unsupported AST expressions must fail lowering explicitly");
        assert!(res.unwrap_err().contains("Unsupported AST expression form"));
    }
}
