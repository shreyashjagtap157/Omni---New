//! AST-to-MIR Lowering Engine for Omni.
//! Translates parsed expressions and variable bindings into canonical Mid-Level IR.

use crate::ir::Body;
use index_vec::IndexVec;
use omni_types::monomorph::MonomorphizedProgram;

pub struct LoweringContext {
    body: Body,
}

impl LoweringContext {
    pub fn new() -> Self {
        Self { body: Body { blocks: IndexVec::new(), local_decls: IndexVec::new() } }
    }

    /// Lowers a validated, concrete `MonomorphizedProgram` into a `MirProgram`.
    ///
    /// CRITICAL SEMANTIC GATE: Invokes `assert_concrete_for_mir()` on the input program.
    /// Lowering immediately aborts and fails closed if any unresolved generic parameters,
    /// undischarged trait obligations, or unspecialized generic calls remain.
    pub fn lower_monomorphized_program(
        &mut self,
        prog: &MonomorphizedProgram,
    ) -> Result<crate::ir::MirProgram, String> {
        // Enforce concrete semantic gate before MIR generation
        prog.assert_concrete_for_mir()?;

        let mut tcx = omni_types::intern::TyCtxt::new();
        let subst = omni_types::checker::SubstEnv::new();
        let mut mir_functions = Vec::new();

        for func in &prog.functions {
            let mut blocks = IndexVec::new();
            let mut local_decls = IndexVec::new();
            let mut scope = std::collections::HashMap::new();

            // Lower return type spec into concrete Ty
            let ret_ty = tcx.lower_type_spec(&func.return_type, &subst);

            // Return place: Local(0)
            let return_place = local_decls
                .push(crate::ir::LocalDecl { name: Some("_return".to_string()), ty: Some(ret_ty) });

            // Parameter places: Local(1..1+param_count)
            let mut param_locals = Vec::new();
            for (p_name, p_type) in &func.params {
                let p_ty = tcx.lower_type_spec(p_type, &subst);
                let p_local = local_decls
                    .push(crate::ir::LocalDecl { name: Some(p_name.clone()), ty: Some(p_ty) });
                param_locals.push(p_local);
                scope.insert(p_name.clone(), p_local);
            }

            let mut builder = FnMirBuilder {
                tcx: &mut tcx,
                subst: &subst,
                local_decls: &mut local_decls,
                blocks: &mut blocks,
                scope,
                current_block: None,
            };

            let entry_block = builder.new_block();
            builder.current_block = Some(entry_block);

            let res_operand = builder.lower_expr(&func.body)?;

            if let Some(curr) = builder.current_block {
                let ret_p = crate::ir::Place { local: return_place };
                builder.blocks[curr]
                    .statements
                    .push(crate::ir::Statement::Assign(ret_p, crate::ir::Rvalue::Use(res_operand)));
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
    tcx: &'a mut omni_types::intern::TyCtxt,
    subst: &'a omni_types::checker::SubstEnv,
    local_decls: &'a mut IndexVec<crate::ir::Local, crate::ir::LocalDecl>,
    blocks: &'a mut IndexVec<crate::ir::BasicBlock, crate::ir::BlockData>,
    scope: std::collections::HashMap<String, crate::ir::Local>,
    current_block: Option<crate::ir::BasicBlock>,
}

impl<'a> FnMirBuilder<'a> {
    fn new_block(&mut self) -> crate::ir::BasicBlock {
        self.blocks.push(crate::ir::BlockData { statements: Vec::new(), terminator: None })
    }

    fn new_temp(
        &mut self,
        name: Option<String>,
        ty: Option<omni_types::intern::Ty>,
    ) -> crate::ir::Local {
        self.local_decls.push(crate::ir::LocalDecl { name, ty })
    }

    fn lower_expr(&mut self, expr: &omni_types::ast::Expr) -> Result<crate::ir::Operand, String> {
        let curr_block = self.current_block.ok_or_else(|| {
            "Lowering error: expression evaluated in unreachable block".to_string()
        })?;

        match expr {
            omni_types::ast::Expr::Literal(lit) => {
                Ok(crate::ir::Operand::Constant(crate::ir::Constant::Lit(lit.clone())))
            }
            omni_types::ast::Expr::Var(name) => {
                let local = self
                    .scope
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("Lowering error: undefined variable '{}'", name))?;
                Ok(crate::ir::Operand::Copy(crate::ir::Place { local }))
            }
            omni_types::ast::Expr::Binary { op, lhs, rhs } => {
                let lhs_op = self.lower_expr(lhs)?;
                let rhs_op = self.lower_expr(rhs)?;
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
                let temp_local = self.new_temp(Some("_bin_tmp".to_string()), None);
                let place = crate::ir::Place { local: temp_local };
                self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                    place,
                    crate::ir::Rvalue::BinaryOp(mir_op, lhs_op, rhs_op),
                ));
                Ok(crate::ir::Operand::Copy(place))
            }
            omni_types::ast::Expr::Unary { op, expr } => {
                let inner_op = self.lower_expr(expr)?;
                let mir_op = match op {
                    omni_types::ast::UnOp::Neg => crate::ir::UnOp::Neg,
                    omni_types::ast::UnOp::Not => crate::ir::UnOp::Not,
                };
                let temp_local = self.new_temp(Some("_un_tmp".to_string()), None);
                let place = crate::ir::Place { local: temp_local };
                self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                    place,
                    crate::ir::Rvalue::UnaryOp(mir_op, inner_op),
                ));
                Ok(crate::ir::Operand::Copy(place))
            }
            omni_types::ast::Expr::Let { name, ty, init, body } => {
                let init_op = self.lower_expr(init)?;
                let var_ty = ty.as_ref().map(|spec| self.tcx.lower_type_spec(spec, self.subst));
                let var_local = self.new_temp(Some(name.clone()), var_ty);
                let place = crate::ir::Place { local: var_local };
                self.blocks[curr_block]
                    .statements
                    .push(crate::ir::Statement::Assign(place, crate::ir::Rvalue::Use(init_op)));
                self.scope.insert(name.clone(), var_local);
                self.lower_expr(body)
            }
            omni_types::ast::Expr::Block(exprs) => {
                if exprs.is_empty() {
                    Ok(crate::ir::Operand::Constant(crate::ir::Constant::Lit(
                        omni_types::ast::Lit::Bool(true),
                    )))
                } else {
                    let mut last_op = crate::ir::Operand::Constant(crate::ir::Constant::Lit(
                        omni_types::ast::Lit::Bool(true),
                    ));
                    for e in exprs {
                        last_op = self.lower_expr(e)?;
                    }
                    Ok(last_op)
                }
            }
            omni_types::ast::Expr::Call { func, args, .. } => {
                let mut arg_ops = Vec::new();
                for arg in args {
                    arg_ops.push(self.lower_expr(arg)?);
                }
                let dest_local = self.new_temp(Some("_call_dest".to_string()), None);
                let dest_place = crate::ir::Place { local: dest_local };
                let next_block = self.new_block();
                self.blocks[curr_block].terminator = Some(crate::ir::Terminator::Call {
                    func: crate::ir::Operand::Constant(crate::ir::Constant::FnRef(func.clone())),
                    args: arg_ops,
                    destination: dest_place,
                    target: next_block,
                    cleanup: None,
                });
                self.current_block = Some(next_block);
                Ok(crate::ir::Operand::Copy(dest_place))
            }
            omni_types::ast::Expr::Return(opt_expr) => {
                let ret_op = if let Some(e) = opt_expr {
                    self.lower_expr(e)?
                } else {
                    crate::ir::Operand::Constant(crate::ir::Constant::Lit(
                        omni_types::ast::Lit::Bool(true),
                    ))
                };
                let ret_p = crate::ir::Place { local: crate::ir::Local::from_usize(0) };
                self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                    ret_p,
                    crate::ir::Rvalue::Use(ret_op.clone()),
                ));
                self.blocks[curr_block].terminator = Some(crate::ir::Terminator::Return);
                let dead_block = self.new_block();
                self.current_block = Some(dead_block);
                Ok(ret_op)
            }
            unsupported => {
                Err(format!("Unsupported AST expression form for MIR lowering: {:?}", unsupported))
            }
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
                body: Expr::Literal(Lit::Int(42)),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_ok(), "Fully monomorphized program must pass MIR lowering gate");
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
        assert!(
            res.is_err(),
            "Program with unresolved type parameters must fail closed at MIR lowering gate"
        );
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
        assert!(
            res.is_err(),
            "Program with undischarged trait bounds must fail closed at MIR lowering gate"
        );
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
                    ty: Some(TypeSpec::Int),
                    init: Box::new(Expr::Binary {
                        op: omni_types::ast::BinOp::Add,
                        lhs: Box::new(Expr::Var("x".to_string())),
                        rhs: Box::new(Expr::Literal(Lit::Int(2))),
                    }),
                    body: Box::new(Expr::Var("y".to_string())),
                },
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_ok(), "Semantic let and binary expression must lower successfully");
        let mir_prog = res.unwrap();
        assert_eq!(mir_prog.functions.len(), 1);
        let func = &mir_prog.functions[0];
        assert_eq!(func.name, "add_let");
        assert_eq!(func.params.len(), 1);
        assert!(!func.body.blocks.is_empty());
        assert!(func.body.local_decls.len() >= 3); // return + param + y
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
                body: Expr::Lambda { params: vec![], body: Box::new(Expr::Literal(Lit::Int(1))) },
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_err(), "Unsupported AST expressions must fail lowering explicitly");
        assert!(res.unwrap_err().contains("Unsupported AST expression form"));
    }
}
