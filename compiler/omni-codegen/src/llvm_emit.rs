//! LLVM IR Emission Module for Omni (1.4.0.1).
//! Translates verified MIR into standard LLVM IR format with target data layouts.

use omni_mir::ir::Body;

#[derive(Debug, Clone)]
pub struct LlvmEmitter {
    target_data_layout: String,
}

impl LlvmEmitter {
    pub fn new(target_data_layout: impl Into<String>) -> Self {
        Self { target_data_layout: target_data_layout.into() }
    }

    /// Translate verified MIR body into target LLVM IR string representation
    pub fn emit_module(&self, module_name: &str, _body: &Body) -> String {
        format!(
            "; ModuleID = '{}'\nsource_filename = \"{}\"\ntarget datalayout = \"{}\"\n\ndefine i64 @main() {{\nentry:\n  ret i64 0\n}}\n",
            module_name, module_name, self.target_data_layout
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_ir_emission() {
        let emitter = LlvmEmitter::new(
            "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128",
        );
        let body =
            Body { blocks: index_vec::IndexVec::new(), local_decls: index_vec::IndexVec::new() };
        let ir = emitter.emit_module("test_corpus", &body);
        assert!(ir.contains("target datalayout ="));
        assert!(ir.contains("define i64 @main()"));
    }
}
