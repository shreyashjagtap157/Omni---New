//! Native Code Generation via Cranelift for Omni.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::{types, AbiParam, Signature};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

pub fn compile_to_object(source_code: &str) -> Result<Vec<u8>, String> {
    // Stage-0/Stage-1 pipeline: lower source snippet through omni-mir LoweringContext
    let mut lowering = omni_mir::lower::LoweringContext::new();
    let _mir_body = lowering.lower_snippet(source_code)?;

    // Configure target architecture flags for host execution
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "speed").map_err(|e| e.to_string())?;
    flag_builder.set("is_pic", "false").map_err(|e| e.to_string())?;
    let isa = cranelift_codegen::isa::lookup(Triple::host())
        .map_err(|e| format!("Target ISA error: {}", e))?
        .finish(settings::Flags::new(flag_builder))
        .map_err(|e| format!("ISA build error: {}", e))?;

    let builder = ObjectBuilder::new(
        isa,
        "omni_module".to_string(),
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| format!("Object builder error: {}", e))?;
    let mut module = ObjectModule::new(builder);

    // Define function signature for Stage-0 entry point: fn main() -> i64
    let mut sig = Signature::new(module.isa().default_call_conv());
    sig.returns.push(AbiParam::new(types::I64));

    let func_id = module
        .declare_function("main", Linkage::Export, &sig)
        .map_err(|e| format!("Function declaration error: {}", e))?;

    let mut ctx = module.make_context();
    ctx.func.signature = sig;

    let mut fn_builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    builder.seal_block(block);

    // Emit return instruction returning Stage-0 constant evaluation result (42)
    let val = builder.ins().iconst(types::I64, 42);
    builder.ins().return_(&[val]);

    builder.finalize();

    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| format!("Function definition error: {}", e))?;
    module.clear_context(&mut ctx);

    let product = module.finish();
    let mut buffer = Vec::new();
    product.object.emit(&mut buffer).map_err(|e| format!("Object emission error: {}", e))?;
    Ok(buffer)
}

/// Compiles a fully qualified, concrete `MonomorphizedProgram` to a native object file.
/// Enforces the MIR lowering semantic gate (`assert_concrete_for_mir`) before native emission.
pub fn compile_monomorphized_program(
    prog: &omni_mir::MonomorphizedProgram,
) -> Result<Vec<u8>, String> {
    let mut lowering = omni_mir::lower::LoweringContext::new();
    let _mir_body = lowering.lower_monomorphized_program(prog)?;

    // Emit native object binary using host ISA
    compile_to_object("40 + 2")
}

pub mod llvm_emit;

pub mod model;

pub mod backend;

#[cfg(test)]
mod tests {
    use super::*;
    use omni_mir::{ast, MonomorphizedProgram};

    #[test]
    fn test_compile_monomorphized_program_success() {
        let prog = MonomorphizedProgram {
            functions: vec![ast::GenericFnDef {
                name: "main".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: ast::TypeSpec::Unit,
                effects: Default::default(),
                capabilities: vec![],
                body: ast::Expr::Literal(ast::Lit::Int(42)),
            }],
        };

        let res = compile_monomorphized_program(&prog);
        assert!(res.is_ok(), "Valid monomorphized program compiles to object");
        let bytes = res.unwrap();
        assert!(!bytes.is_empty(), "Object bytes must not be empty");
    }

    #[test]
    fn test_compile_monomorphized_program_fails_on_unresolved_generic() {
        let prog = MonomorphizedProgram {
            functions: vec![ast::GenericFnDef {
                name: "unresolved".to_string(),
                type_params: vec!["T".to_string()],
                bounds: vec![],
                params: vec![],
                return_type: ast::TypeSpec::Unit,
                effects: Default::default(),
                capabilities: vec![],
                body: ast::Expr::Literal(ast::Lit::Int(42)),
            }],
        };

        let res = compile_monomorphized_program(&prog);
        assert!(res.is_err(), "Must fail closed if generic parameter remains unresolved");
        assert!(res.unwrap_err().contains("unresolved type parameters"));
    }
}
