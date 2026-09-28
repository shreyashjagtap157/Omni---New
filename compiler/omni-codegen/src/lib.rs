//! Native Code Generation via Cranelift for Omni.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::{types, AbiParam, Signature};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

/// Compiles a fully qualified, concrete monomorphized program to a native object file.
/// This is the only source of native emission; frontend orchestration belongs to omni-driver.
fn native_abi_type(
    spec: &omni_mir::ast::TypeSpec,
) -> Result<Option<cranelift_codegen::ir::Type>, String> {
    match spec {
        omni_mir::ast::TypeSpec::Unit => Ok(None),
        omni_mir::ast::TypeSpec::Int
        | omni_mir::ast::TypeSpec::Bool
        | omni_mir::ast::TypeSpec::Byte
        | omni_mir::ast::TypeSpec::Char => Ok(Some(types::I64)),
        other => {
            Err(format!("Codegen error: native backend does not yet support ABI type {:?}", other))
        }
    }
}

/// Enforces MIR lowering semantic gate and MirVerifier before native emission.
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

    let mut function_ids = std::collections::HashMap::new();

    // Predeclare every concrete MIR function with the same ABI signature before emitting
    // any body, so calls can only reference already-qualified declarations.
    for mir_func in &mir_prog.functions {
        let source_def =
            prog.functions.iter().find(|f| f.name == mir_func.name).ok_or_else(|| {
                format!("Codegen error: missing source function '{}'", mir_func.name)
            })?;
        if mir_func.params.len() != source_def.params.len() {
            return Err(format!(
                "Codegen error: MIR/source parameter count mismatch for '{}'",
                mir_func.name
            ));
        }

        let mut sig = Signature::new(module.isa().default_call_conv());
        if let Some(ret_ty) = native_abi_type(&source_def.return_type)? {
            sig.returns.push(AbiParam::new(ret_ty));
        }
        for (_, param_spec) in &source_def.params {
            let param_ty = native_abi_type(param_spec)?.ok_or_else(|| {
                format!(
                    "Codegen error: Unit parameter is not representable in native ABI for '{}'",
                    source_def.name
                )
            })?;
            sig.params.push(AbiParam::new(param_ty));
        }

        let func_id = module
            .declare_function(&mir_func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Function declaration error: {}", e))?;
        function_ids.insert(mir_func.name.clone(), func_id);
    }

    for mir_func in &mir_prog.functions {
        let source_def =
            prog.functions.iter().find(|f| f.name == mir_func.name).ok_or_else(|| {
                format!("Codegen error: missing source function '{}'", mir_func.name)
            })?;
        let func_id = *function_ids.get(&mir_func.name).ok_or_else(|| {
            format!("Codegen error: function '{}' was not predeclared", mir_func.name)
        })?;

        let mut ctx = module.make_context();
        let mut sig = Signature::new(module.isa().default_call_conv());
        if let Some(ret_ty) = native_abi_type(&source_def.return_type)? {
            sig.returns.push(AbiParam::new(ret_ty));
        }
        for (_, param_spec) in &source_def.params {
            let param_ty = native_abi_type(param_spec)?.ok_or_else(|| {
                format!(
                    "Codegen error: Unit parameter is not representable in native ABI for '{}'",
                    source_def.name
                )
            })?;
            sig.params.push(AbiParam::new(param_ty));
        }
        ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);

        let mut cl_blocks = std::collections::HashMap::new();
        for (b_idx, _) in mir_func.body.blocks.iter().enumerate() {
            let cl_b = builder.create_block();
            cl_blocks.insert(b_idx, cl_b);
        }

        let entry_cl_block = *cl_blocks.get(&0).ok_or_else(|| {
            format!("Codegen error: function '{}' has no entry block", mir_func.name)
        })?;
        builder.append_block_params_for_function_params(entry_cl_block);
        builder.switch_to_block(entry_cl_block);

        let mut locals_map = std::collections::HashMap::new();
        for (p_idx, &param_local) in mir_func.params.iter().enumerate() {
            let cl_val =
                builder.block_params(entry_cl_block).get(p_idx).copied().ok_or_else(|| {
                    format!(
                        "Codegen error: function '{}' has no Cranelift parameter for MIR parameter {}",
                        mir_func.name, p_idx
                    )
                })?;
            locals_map.insert(param_local, cl_val);
        }

        for (b_idx, mir_block) in mir_func.body.blocks.iter().enumerate() {
            let cl_b = *cl_blocks
                .get(&b_idx)
                .ok_or_else(|| format!("Codegen error: missing Cranelift block {}", b_idx))?;
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

            let term = mir_block.terminator.as_ref().ok_or_else(|| {
                format!(
                    "Codegen error: MIR block {:?} in '{}' is unterminated",
                    b_idx, mir_func.name
                )
            })?;

            match term {
                omni_mir::ir::Terminator::Return => {
                    if native_abi_type(&source_def.return_type)?.is_some() {
                        let ret_val = locals_map
                            .get(&mir_func.return_place)
                            .copied()
                            .ok_or_else(|| {
                                format!(
                                    "Codegen error: Return place {:?} was not assigned in function '{}'",
                                    mir_func.return_place, mir_func.name
                                )
                            })?;
                        builder.ins().return_(&[ret_val]);
                    } else {
                        builder.ins().return_(&[]);
                    }
                }
                omni_mir::ir::Terminator::Goto(target) => {
                    let target_cl = *cl_blocks.get(&target.index()).ok_or_else(|| {
                        format!("Codegen error: undefined goto target {:?}", target)
                    })?;
                    builder.ins().jump(target_cl, &[]);
                }
                omni_mir::ir::Terminator::SwitchInt { discr, targets, otherwise } => {
                    let discr_val = lower_operand_to_cl(&mut builder, discr, &locals_map)?;
                    let otherwise_cl = *cl_blocks.get(&otherwise.index()).ok_or_else(|| {
                        format!("Codegen error: undefined switch target {:?}", otherwise)
                    })?;
                    let mut switch = cranelift_frontend::Switch::new();
                    for (val, t_block) in targets {
                        let target_cl = *cl_blocks.get(&t_block.index()).ok_or_else(|| {
                            format!("Codegen error: undefined switch target {:?}", t_block)
                        })?;
                        switch.set_entry(u128::from(*val), target_cl);
                    }
                    switch.emit(&mut builder, discr_val, otherwise_cl);
                }
                omni_mir::ir::Terminator::Call { func, args, destination, target, cleanup: _ } => {
                    let fn_name =
                        match func {
                            omni_mir::ir::Operand::Constant(omni_mir::ir::Constant::FnRef(
                                name,
                            )) => name,
                            _ => return Err(
                                "Indirect function calls not yet supported in Cranelift emission"
                                    .into(),
                            ),
                        };
                    let source_callee =
                        prog.functions.iter().find(|f| f.name == *fn_name).ok_or_else(|| {
                            format!("Codegen error: call target '{}' not found", fn_name)
                        })?;
                    if args.len() != source_callee.params.len() {
                        return Err(format!(
                            "Codegen error: call '{}' expected {} arguments, found {}",
                            fn_name,
                            source_callee.params.len(),
                            args.len()
                        ));
                    }

                    let mut call_args = Vec::with_capacity(args.len());
                    for (arg, (_, param_spec)) in args.iter().zip(source_callee.params.iter()) {
                        let value = lower_operand_to_cl(&mut builder, arg, &locals_map)?;
                        let expected = native_abi_type(param_spec)?.ok_or_else(|| {
                            format!(
                                "Codegen error: Unit argument is not representable in native ABI for '{}'",
                                source_callee.name
                            )
                        })?;
                        if builder.func.dfg.value_type(value) != expected {
                            return Err(format!(
                                "Codegen error: call '{}' argument ABI type mismatch: expected {:?}, found {:?}",
                                fn_name,
                                expected,
                                builder.func.dfg.value_type(value)
                            ));
                        }
                        call_args.push(value);
                    }

                    let callee_id = *function_ids.get(fn_name).ok_or_else(|| {
                        format!("Codegen error: callee '{}' was not predeclared", fn_name)
                    })?;
                    let local_callee = module.declare_func_in_func(callee_id, builder.func);
                    let call_inst = builder.ins().call(local_callee, &call_args);
                    let results = builder.inst_results(call_inst);

                    match destination {
                        Some(destination) => {
                            if results.len() != 1 {
                                return Err(format!(
                                    "Codegen error: call '{}' has no return value but MIR requests destination {:?}",
                                    fn_name, destination
                                ));
                            }
                            locals_map.insert(destination.local, results[0]);
                        }
                        None => {
                            if !results.is_empty() {
                                return Err(format!(
                                    "Codegen error: call '{}' returns a value but MIR has no destination",
                                    fn_name
                                ));
                            }
                        }
                    }

                    let target_cl = *cl_blocks.get(&target.index()).ok_or_else(|| {
                        format!("Codegen error: undefined call target {:?}", target)
                    })?;
                    builder.ins().jump(target_cl, &[]);
                }
                omni_mir::ir::Terminator::Unreachable => {
                    builder.ins().trap(cranelift_codegen::ir::TrapCode::UnreachableCodeReached);
                }
            }
        }

        for cl_b in cl_blocks.values().copied() {
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
                omni_mir::ir::BinOp::Eq => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Ne => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Lt => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Gt => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Le => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Ge => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                    l,
                    r,
                ),
            }
        }
        omni_mir::ir::Rvalue::UnaryOp(op, operand) => {
            let val = lower_operand_to_cl(builder, operand, locals)?;
            match op {
                omni_mir::ir::UnOp::Neg => Ok(builder.ins().ineg(val)),
                omni_mir::ir::UnOp::Not => {
                    let one = builder.ins().iconst(types::I64, 1);
                    Ok(builder.ins().bxor(val, one))
                }
            }
        }
    }
}

fn lower_int_comparison(
    builder: &mut FunctionBuilder,
    condition: cranelift_codegen::ir::condcodes::IntCC,
    lhs: cranelift_codegen::ir::Value,
    rhs: cranelift_codegen::ir::Value,
) -> Result<cranelift_codegen::ir::Value, String> {
    let predicate = builder.ins().icmp(condition, lhs, rhs);
    let one = builder.ins().iconst(types::I64, 1);
    let zero = builder.ins().iconst(types::I64, 0);
    Ok(builder.ins().select(predicate, one, zero))
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
                body: ast::Expr::Block(vec![]),
            }],
        };

        let res = compile_monomorphized_program(&prog);
        assert!(res.is_ok(), "Valid monomorphized program compiles to object");
        let bytes = res.unwrap();
        assert!(!bytes.is_empty(), "Object bytes must not be empty");
    }

    #[test]
    fn test_compile_monomorphized_program_int_return_and_call() {
        let callee = ast::GenericFnDef {
            name: "inc".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![("x".to_string(), ast::TypeSpec::Int)],
            return_type: ast::TypeSpec::Int,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Binary {
                op: ast::BinOp::Add,
                lhs: Box::new(ast::Expr::Var("x".to_string())),
                rhs: Box::new(ast::Expr::Literal(ast::Lit::Int(1))),
            },
        };
        let caller = ast::GenericFnDef {
            name: "main".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: ast::TypeSpec::Int,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Call {
                func: "inc".to_string(),
                generic_args: vec![],
                args: vec![ast::Expr::Literal(ast::Lit::Int(41))],
            },
        };

        let bytes = compile_monomorphized_program(&MonomorphizedProgram {
            functions: vec![callee, caller],
        })
        .expect("typed MIR with a concrete call must compile");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_compile_monomorphized_program_unit_call_has_no_result() {
        let callee = ast::GenericFnDef {
            name: "touch".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: ast::TypeSpec::Unit,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Block(vec![]),
        };
        let caller = ast::GenericFnDef {
            name: "main".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: ast::TypeSpec::Unit,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Call { func: "touch".to_string(), generic_args: vec![], args: vec![] },
        };

        let bytes = compile_monomorphized_program(&MonomorphizedProgram {
            functions: vec![callee, caller],
        })
        .expect("Unit call must compile without fabricating a result");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_compile_monomorphized_program_bool_comparison() {
        let prog = MonomorphizedProgram {
            functions: vec![ast::GenericFnDef {
                name: "main".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: ast::TypeSpec::Bool,
                effects: Default::default(),
                capabilities: vec![],
                body: ast::Expr::Binary {
                    op: ast::BinOp::Eq,
                    lhs: Box::new(ast::Expr::Literal(ast::Lit::Int(1))),
                    rhs: Box::new(ast::Expr::Literal(ast::Lit::Int(1))),
                },
            }],
        };

        let bytes = compile_monomorphized_program(&prog)
            .expect("boolean comparison must have a concrete native representation");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_compile_monomorphized_program_logical_not() {
        let prog = MonomorphizedProgram {
            functions: vec![ast::GenericFnDef {
                name: "main".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: ast::TypeSpec::Bool,
                effects: Default::default(),
                capabilities: vec![],
                body: ast::Expr::Unary {
                    op: ast::UnOp::Not,
                    expr: Box::new(ast::Expr::Literal(ast::Lit::Bool(false))),
                },
            }],
        };

        let bytes = compile_monomorphized_program(&prog)
            .expect("logical boolean not must have a concrete native representation");
        assert!(!bytes.is_empty());
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
