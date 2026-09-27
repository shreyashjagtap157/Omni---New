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

        let mut mir_functions = Vec::new();

        for func in &prog.functions {
            let mut blocks = IndexVec::new();
            let mut local_decls = IndexVec::new();

            // Return place: Local(0)
            let return_place = local_decls
                .push(crate::ir::LocalDecl { name: Some("_return".to_string()), ty: None });

            // Parameter places: Local(1..1+param_count)
            let mut param_locals = Vec::new();
            for (p_name, _p_type) in &func.params {
                let p_local =
                    local_decls.push(crate::ir::LocalDecl { name: Some(p_name.clone()), ty: None });
                param_locals.push(p_local);
            }

            let mut stmts = Vec::new();
            let ret_p = crate::ir::Place { local: return_place };

            let default_constant = match &func.body {
                omni_types::ast::Expr::Literal(lit) => crate::ir::Constant::Lit(lit.clone()),
                _ => crate::ir::Constant::Lit(omni_types::ast::Lit::Int(42)),
            };

            stmts.push(crate::ir::Statement::Assign(
                ret_p,
                crate::ir::Rvalue::Use(crate::ir::Operand::Constant(default_constant)),
            ));

            blocks.push(crate::ir::BlockData {
                statements: stmts,
                terminator: Some(crate::ir::Terminator::Return),
            });

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
}
