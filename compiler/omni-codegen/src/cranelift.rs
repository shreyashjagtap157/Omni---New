//! # Cranelift Code Generation Lowering
//! Translates Omni MIR structures into Cranelift IR function contexts.

use cranelift_codegen::ir::{types, AbiParam, Function, Signature};
use cranelift_codegen::isa::CallConv;
use target_lexicon::Triple;
use omni_mir::ir::Body;

pub struct CraneliftLowering {
    pub target_triple: String,
}

impl CraneliftLowering {
    pub fn new(target_triple: impl Into<String>) -> Self {
        Self {
            target_triple: target_triple.into(),
        }
    }

    /// Lower a MIR Body into a Cranelift Function signature and body stub
    pub fn lower_body(&self, name: &str, body: &Body) -> Result<Function, String> {
        let triple = self.target_triple.parse::<Triple>().unwrap_or_else(|_| Triple::host());
        let call_conv = CallConv::triple_default(&triple);
        let mut sig = Signature::new(call_conv);
        sig.returns.push(AbiParam::new(types::I64));

        let func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, 0),
            sig,
        );

        let _ = (name, body);

        Ok(func)
    }
}
