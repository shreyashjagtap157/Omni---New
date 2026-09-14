//! Unified Backend Trait and Text LLVM Backend Adapter.

use crate::model::LlvmModule;

pub trait Backend {
    fn emit_module(&self, module: &LlvmModule) -> String;
}

#[derive(Debug, Clone, Default)]
pub struct TextLlvmBackend;

impl Backend for TextLlvmBackend {
    fn emit_module(&self, module: &LlvmModule) -> String {
        module.emit_text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_backend_adapter() {
        let module = LlvmModule::new("backend_test", "e-m:e");
        let backend = TextLlvmBackend;
        let output = backend.emit_module(&module);
        assert!(output.contains("; ModuleID = 'backend_test'"));
    }
}
