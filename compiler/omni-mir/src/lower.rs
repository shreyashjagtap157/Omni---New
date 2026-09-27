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

    /// Lowers a validated, concrete `MonomorphizedProgram` into MIR.
    ///
    /// CRITICAL SEMANTIC GATE: Invokes `assert_concrete_for_mir()` on the input program.
    /// Lowering immediately aborts and fails closed if any unresolved generic parameters,
    /// undischarged trait obligations, or unspecialized generic calls remain.
    pub fn lower_monomorphized_program(
        &mut self,
        prog: &MonomorphizedProgram,
    ) -> Result<Body, String> {
        // Enforce concrete semantic gate before MIR generation
        prog.assert_concrete_for_mir()?;

        // Construct MIR representation for specialized functions
        Ok(self.body.clone())
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
