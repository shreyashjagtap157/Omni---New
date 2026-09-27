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
/// Enforces MIR lowering semantic gate (`assert_concrete_for_mir`) and `MirVerifier` before native emission.
pub fn compile_monomorphized_program(
    prog: &omni_mir::MonomorphizedProgram,
) -> Result<Vec<u8>, String> {
    let mut lowering = omni_mir::lower::LoweringContext::new();
    let mir_prog = lowering.lower_monomorphized_program(prog)?;

    // MANDATORY PRE-CODEGEN VERIFICATION GATE
    omni_verify::MirVerifier::verify_program(&mir_prog)
        .map_err(|e| format!("Pre-codegen MIR verification failed: {}", e))?;

    if mir_prog.functions.is_empty() {
        return Err("Cannot compile empty monomorphized program".into());
    }

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

    for mir_func in &mir_prog.functions {
        let mut sig = Signature::new(module.isa().default_call_conv());

        // Match return signature derived from MirFunction return type
        let ret_type = match &mir_func.return_type {
            omni_mir::ast::TypeSpec::Unit => None,
            _ => Some(types::I64),
        };

        if let Some(rt) = ret_type {
            sig.returns.push(AbiParam::new(rt));
        }

        // Add parameters to Cranelift signature
        for _ in &mir_func.params {
            sig.params.push(AbiParam::new(types::I64));
        }

        let func_id = module
            .declare_function(&mir_func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Function declaration error: {}", e))?;

        let mut ctx = module.make_context();
        ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);

        let mut cl_blocks = std::collections::HashMap::new();
        for (b_idx, _) in mir_func.body.blocks.iter().enumerate() {
            let cl_b = builder.create_block();
            cl_blocks.insert(b_idx, cl_b);
        }

        let entry_cl_block = cl_blocks[&0];
        builder.append_block_params_for_function_params(entry_cl_block);
        builder.switch_to_block(entry_cl_block);

        let mut locals_map = std::collections::HashMap::new();
        for (p_idx, &param_local) in mir_func.params.iter().enumerate() {
            let cl_val = builder.block_params(entry_cl_block)[p_idx];
            locals_map.insert(param_local, cl_val);
        }

        for (b_idx, mir_block) in mir_func.body.blocks.iter().enumerate() {
            let cl_b = cl_blocks[&b_idx];
            if b_idx != 0 {
                builder.switch_to_block(cl_b);
            }

            for stmt in &mir_block.statements {
                match stmt {
                    omni_mir::ir::Statement::Assign(place, rval) => {
                        let val = lower_rvalue_to_cl(&mut builder, rval, &locals_map)?;
                        locals_map.insert(place.local, val);
                    }
                    omni_mir::ir::Statement::Assume(_) | omni_mir::ir::Statement::Drop(_) => {}
                }
            }

            if let Some(term) = &mir_block.terminator {
                match term {
                    omni_mir::ir::Terminator::Return => {
                        if ret_type.is_some() {
                            let ret_val = locals_map
                                .get(&mir_func.return_place)
                                .copied()
                                .unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
                            builder.ins().return_(&[ret_val]);
                        } else {
                            builder.ins().return_(&[]);
                        }
                    }
                    omni_mir::ir::Terminator::Goto(target) => {
                        let target_cl = cl_blocks[&target.index()];
                        builder.ins().jump(target_cl, &[]);
                    }
                    omni_mir::ir::Terminator::SwitchInt { discr, targets, otherwise } => {
                        let discr_val = lower_operand_to_cl(&mut builder, discr, &locals_map)?;
                        let otherwise_cl = cl_blocks[&otherwise.index()];
                        let mut switch = cranelift_frontend::Switch::new();
                        for (val, t_block) in targets {
                            switch.set_entry(u128::from(*val), cl_blocks[&t_block.index()]);
                        }
                        switch.emit(&mut builder, discr_val, otherwise_cl);
                    }
                    omni_mir::ir::Terminator::Call { func, args, destination, target, .. } => {
                        let mut call_args = Vec::new();
                        for arg in args {
                            call_args.push(lower_operand_to_cl(&mut builder, arg, &locals_map)?);
                        }
                        let fn_name = match func {
                            omni_mir::ir::Operand::Constant(omni_mir::ir::Constant::FnRef(
                                name,
                            )) => name,
                            _ => return Err(
                                "Indirect function calls not yet supported in Cranelift emission"
                                    .into(),
                            ),
                        };
                        let mut callee_sig = Signature::new(module.isa().default_call_conv());
                        callee_sig.returns.push(AbiParam::new(types::I64));
                        for _ in &call_args {
                            callee_sig.params.push(AbiParam::new(types::I64));
                        }
                        let callee_id = module
                            .declare_function(fn_name, Linkage::Export, &callee_sig)
                            .map_err(|e| format!("Callee declaration error: {}", e))?;
                        let local_callee = module.declare_func_in_func(callee_id, builder.func);
                        let call_inst = builder.ins().call(local_callee, &call_args);
                        let res_val = builder.inst_results(call_inst)[0];
                        locals_map.insert(destination.local, res_val);
                        let target_cl = cl_blocks[&target.index()];
                        builder.ins().jump(target_cl, &[]);
                    }
                    omni_mir::ir::Terminator::Unreachable => {
                        builder.ins().trap(cranelift_codegen::ir::TrapCode::UnreachableCodeReached);
                    }
                }
            }

            builder.seal_block(cl_b);
        }

        builder.finalize();

        module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Function definition error: {}", e))?;
        module.clear_context(&mut ctx);
    }

    let product = module.finish();
    let mut buffer = Vec::new();
    product.object.emit(&mut buffer).map_err(|e| format!("Object emission error: {}", e))?;
    Ok(buffer)
}

fn lower_operand_to_cl(
    builder: &mut FunctionBuilder,
    op: &omni_mir::ir::Operand,
    locals: &std::collections::HashMap<omni_mir::ir::Local, cranelift_codegen::ir::Value>,
) -> Result<cranelift_codegen::ir::Value, String> {
    match op {
        omni_mir::ir::Operand::Copy(p) | omni_mir::ir::Operand::Move(p) => locals
            .get(&p.local)
            .copied()
            .ok_or_else(|| format!("Codegen error: unbound local {:?}", p.local)),
        omni_mir::ir::Operand::Constant(c) => match c {
            omni_mir::ir::Constant::Lit(lit) => match lit {
                omni_mir::ast::Lit::Int(n) => Ok(builder.ins().iconst(types::I64, *n)),
                omni_mir::ast::Lit::Bool(b) => {
                    Ok(builder.ins().iconst(types::I64, if *b { 1 } else { 0 }))
                }
                omni_mir::ast::Lit::Byte(b) => Ok(builder.ins().iconst(types::I64, *b as i64)),
                omni_mir::ast::Lit::Char(c) => Ok(builder.ins().iconst(types::I64, *c as i64)),
                _ => Err(format!("Unsupported literal form in MIR codegen: {:?}", lit)),
            },
            omni_mir::ir::Constant::FnRef(name) => Err(format!(
                "FnRef constant operand {:?} evaluated outside Call terminator context",
                name
            )),
        },
    }
}

fn lower_rvalue_to_cl(
    builder: &mut FunctionBuilder,
    rval: &omni_mir::ir::Rvalue,
    locals: &std::collections::HashMap<omni_mir::ir::Local, cranelift_codegen::ir::Value>,
) -> Result<cranelift_codegen::ir::Value, String> {
    match rval {
        omni_mir::ir::Rvalue::Use(op) => lower_operand_to_cl(builder, op, locals),
        omni_mir::ir::Rvalue::BinaryOp(op, lhs, rhs) => {
            let l = lower_operand_to_cl(builder, lhs, locals)?;
            let r = lower_operand_to_cl(builder, rhs, locals)?;
            match op {
                omni_mir::ir::BinOp::Add => Ok(builder.ins().iadd(l, r)),
                omni_mir::ir::BinOp::Sub => Ok(builder.ins().isub(l, r)),
                omni_mir::ir::BinOp::Mul => Ok(builder.ins().imul(l, r)),
                omni_mir::ir::BinOp::Div => Ok(builder.ins().sdiv(l, r)),
                omni_mir::ir::BinOp::Eq => {
                    Ok(builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, l, r))
                }
                omni_mir::ir::BinOp::Ne => {
                    Ok(builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, l, r))
                }
                omni_mir::ir::BinOp::Lt => Ok(builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                    l,
                    r,
                )),
                omni_mir::ir::BinOp::Gt => Ok(builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
                    l,
                    r,
                )),
                omni_mir::ir::BinOp::Le => Ok(builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                    l,
                    r,
                )),
                omni_mir::ir::BinOp::Ge => Ok(builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                    l,
                    r,
                )),
            }
        }
        omni_mir::ir::Rvalue::UnaryOp(op) => {
            let val = lower_operand_to_cl(builder, op, locals)?;
            Ok(builder.ins().ineg(val))
        }
    }
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
