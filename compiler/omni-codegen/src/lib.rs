//! Native Code Generation via Cranelift for Omni.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::{types, AbiParam, Signature};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

/// Compiles a fully qualified, concrete monomorphized program to a native object file.
/// This is the only source of native emission; frontend orchestration belongs to omni-driver.
fn native_abi_type_from_ty(
    tcx: &omni_mir::TyCtxt,
    ty: omni_mir::Ty,
) -> Result<Option<cranelift_codegen::ir::Type>, String> {
    match tcx.get(ty) {
        omni_mir::TyKind::Unit => Ok(None),
        omni_mir::TyKind::Int
        | omni_mir::TyKind::Bool
        | omni_mir::TyKind::Byte
        | omni_mir::TyKind::Char => Ok(Some(types::I64)),
        omni_mir::TyKind::Float => Ok(Some(types::F64)),
        omni_mir::TyKind::Reference { .. } => Err(
            "Codegen error: reference ABI requires pointer storage and ownership lowering".into()
        ),
        other => Err(format!(
            "Codegen error: native backend does not yet support MIR ABI type {:?}",
            other
        )),
    }
}

fn native_abi_type(
    spec: &omni_mir::ast::TypeSpec,
) -> Result<Option<cranelift_codegen::ir::Type>, String> {
    match spec {
        omni_mir::ast::TypeSpec::Unit => Ok(None),
        omni_mir::ast::TypeSpec::Int
        | omni_mir::ast::TypeSpec::Bool
        | omni_mir::ast::TypeSpec::Byte
        | omni_mir::ast::TypeSpec::Char => Ok(Some(types::I64)),
        omni_mir::ast::TypeSpec::Float => Ok(Some(types::F64)),
        other => {
            Err(format!("Codegen error: native backend does not yet support ABI type {:?}", other))
        }
    }
}

fn ensure_source_mir_type_match(
    tcx: &omni_mir::TyCtxt,
    spec: &omni_mir::ast::TypeSpec,
    ty: omni_mir::Ty,
    context: &str,
) -> Result<(), String> {
    use omni_mir::ast::TypeSpec;
    use omni_mir::TyKind;

    let matches = matches!(
        (spec, tcx.get(ty)),
        (TypeSpec::Unit, TyKind::Unit)
            | (TypeSpec::Int, TyKind::Int)
            | (TypeSpec::Bool, TyKind::Bool)
            | (TypeSpec::Byte, TyKind::Byte)
            | (TypeSpec::Char, TyKind::Char)
            | (TypeSpec::Float, TyKind::Float)
    );
    if matches {
        Ok(())
    } else {
        Err(format!(
            "Codegen error: source/MIR semantic type mismatch for {}: source {:?}, MIR {:?}",
            context,
            spec,
            tcx.get(ty)
        ))
    }
}

/// Enforces MIR lowering semantic gate and MirVerifier before native emission.
pub fn compile_monomorphized_program(
    prog: &omni_mir::MonomorphizedProgram,
) -> Result<Vec<u8>, String> {
    let mut lowering = omni_mir::lower::LoweringContext::new();
    let mir_prog = lowering.lower_monomorphized_program(prog)?;

    omni_verify::MirVerifier::verify_program(&mir_prog)
        .map_err(|e| format!("Pre-codegen MIR verification failed: {}", e))?;

    if mir_prog.functions.is_empty() {
        return Err("Cannot compile empty monomorphized program".into());
    }

    compile_mir_program(prog, &mir_prog)
}

fn compile_mir_program(
    prog: &omni_mir::MonomorphizedProgram,
    mir_prog: &omni_mir::ir::MirProgram,
) -> Result<Vec<u8>, String> {
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

        let return_mir_ty =
            mir_func.body.local_decls[mir_func.return_place].ty.ok_or_else(|| {
                format!(
                    "Codegen error: return local {:?} has no type for '{}'",
                    mir_func.return_place, mir_func.name
                )
            })?;
        ensure_source_mir_type_match(
            &mir_prog.tcx,
            &source_def.return_type,
            return_mir_ty,
            &format!("return type of '{}'", mir_func.name),
        )?;
        let return_source_abi = native_abi_type(&source_def.return_type)?;
        let return_mir_abi = native_abi_type_from_ty(&mir_prog.tcx, return_mir_ty)?;
        if return_source_abi != return_mir_abi {
            return Err(format!(
                "Codegen error: source/MIR return ABI mismatch for '{}': source {:?}, MIR {:?}",
                mir_func.name, return_source_abi, return_mir_abi
            ));
        }

        for (param_index, ((_, source_spec), &mir_param)) in
            source_def.params.iter().zip(&mir_func.params).enumerate()
        {
            let mir_ty = mir_func.body.local_decls[mir_param].ty.ok_or_else(|| {
                format!(
                    "Codegen error: parameter local {:?} has no type for '{}'",
                    mir_param, mir_func.name
                )
            })?;
            ensure_source_mir_type_match(
                &mir_prog.tcx,
                source_spec,
                mir_ty,
                &format!("parameter {} of '{}'", param_index, mir_func.name),
            )?;
            let source_abi = native_abi_type(source_spec)?;
            let mir_abi = native_abi_type_from_ty(&mir_prog.tcx, mir_ty)?;
            if source_abi != mir_abi {
                return Err(format!(
                    "Codegen error: source/MIR parameter ABI mismatch for '{}' parameter {}: source {:?}, MIR {:?}",
                    mir_func.name, param_index, source_abi, mir_abi
                ));
            }
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

        let mut variables = std::collections::HashMap::new();
        for (local_idx, local_decl) in mir_func.body.local_decls.iter().enumerate() {
            let local = omni_mir::ir::Local::from_usize(local_idx);
            let ty = local_decl
                .ty
                .ok_or_else(|| format!("Codegen error: MIR local {:?} has no type", local))?;
            if let Some(native_ty) = native_abi_type_from_ty(&mir_prog.tcx, ty)? {
                let variable = Variable::from_u32(local_idx as u32);
                builder.declare_var(variable, native_ty);
                variables.insert(local, variable);
            }
        }

        for (p_idx, &param_local) in mir_func.params.iter().enumerate() {
            let cl_val =
                builder.block_params(entry_cl_block).get(p_idx).copied().ok_or_else(|| {
                    format!(
                        "Codegen error: function '{}' has no Cranelift parameter for MIR parameter {}",
                        mir_func.name, p_idx
                    )
                })?;
            let variable = *variables.get(&param_local).ok_or_else(|| {
                format!(
                    "Codegen error: function parameter local {:?} has no native representation",
                    param_local
                )
            })?;
            builder.def_var(variable, cl_val);
        }

        for (b_idx, mir_block) in mir_func.body.blocks.iter().enumerate() {
            let cl_b = *cl_blocks
                .get(&b_idx)
                .ok_or_else(|| format!("Codegen error: missing Cranelift block {}", b_idx))?;
            builder.switch_to_block(cl_b);

            for stmt in &mir_block.statements {
                match stmt {
                    omni_mir::ir::Statement::Assign(place, rval) => {
                        let variable = *variables.get(&place.local).ok_or_else(|| {
                            format!(
                                "Codegen error: assignment target local {:?} has no native representation",
                                place.local
                            )
                        })?;
                        let val = lower_rvalue_to_cl(&mut builder, rval, &variables, &mir.tcx)?;
                        builder.def_var(variable, val);
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
                        let variable = *variables.get(&mir_func.return_place).ok_or_else(|| {
                            format!(
                                "Codegen error: return local {:?} has no native representation",
                                mir_func.return_place
                            )
                        })?;
                        let ret_val = builder.use_var(variable);
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
                    let discr_val = lower_operand_to_cl(&mut builder, discr, &variables)?;
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
                omni_mir::ir::Terminator::Call { func, args, destination, target, cleanup } => {
                    if cleanup.is_some() {
                        return Err(format!(
                            "Codegen error: call in '{}' has an unsupported cleanup/unwind edge",
                            mir_func.name
                        ));
                    }
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
                        let value = lower_operand_to_cl(&mut builder, arg, &variables)?;
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
                            let variable = *variables.get(&destination.local).ok_or_else(|| {
                                format!(
                                    "Codegen error: call destination local {:?} has no native representation",
                                    destination.local
                                )
                            })?;
                            builder.def_var(variable, results[0]);
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

        builder.seal_all_blocks();

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
    variables: &std::collections::HashMap<omni_mir::ir::Local, Variable>,
) -> Result<cranelift_codegen::ir::Value, String> {
    match op {
        omni_mir::ir::Operand::Copy(place) | omni_mir::ir::Operand::Move(place) => {
            let variable = *variables
                .get(&place.local)
                .ok_or_else(|| format!("Codegen error: unbound local {:?}", place.local))?;
            Ok(builder.use_var(variable))
        }
        omni_mir::ir::Operand::Constant(c) => match c {
            omni_mir::ir::Constant::Lit(lit) => match lit {
                omni_mir::ast::Lit::Int(n) => Ok(builder.ins().iconst(types::I64, *n)),
                omni_mir::ast::Lit::Bool(b) => {
                    Ok(builder.ins().iconst(types::I64, if *b { 1 } else { 0 }))
                }
                omni_mir::ast::Lit::Byte(b) => Ok(builder.ins().iconst(types::I64, *b as i64)),
                omni_mir::ast::Lit::Char(c) => Ok(builder.ins().iconst(types::I64, *c as i64)),
                omni_mir::ast::Lit::Float(bits) => Ok(builder.ins().f64const(f64::from_bits(*bits))),
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
    variables: &std::collections::HashMap<omni_mir::ir::Local, Variable>,
    tcx: &omni_mir::TyCtxt,
) -> Result<cranelift_codegen::ir::Value, String> {
    match rval {
        omni_mir::ir::Rvalue::Use(op) => lower_operand_to_cl(builder, op, variables),
        omni_mir::ir::Rvalue::BinaryOp(op, lhs, rhs) => {
            let l = lower_operand_to_cl(builder, lhs, variables)?;
            let r = lower_operand_to_cl(builder, rhs, variables)?;
            let is_float = builder.func.dfg.value_type(l) == types::F64;
            match op {
                omni_mir::ir::BinOp::Add if is_float => Ok(builder.ins().fadd(l, r)),
                omni_mir::ir::BinOp::Sub if is_float => Ok(builder.ins().fsub(l, r)),
                omni_mir::ir::BinOp::Mul if is_float => Ok(builder.ins().fmul(l, r)),
                omni_mir::ir::BinOp::Div if is_float => Ok(builder.ins().fdiv(l, r)),
                omni_mir::ir::BinOp::Add => Ok(builder.ins().iadd(l, r)),
                omni_mir::ir::BinOp::Sub => Ok(builder.ins().isub(l, r)),
                omni_mir::ir::BinOp::Mul => Ok(builder.ins().imul(l, r)),
                omni_mir::ir::BinOp::Div => Ok(builder.ins().sdiv(l, r)),
                omni_mir::ir::BinOp::Rem => Ok(builder.ins().srem(l, r)),
                omni_mir::ir::BinOp::Eq if is_float => lower_float_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::FloatCC::Equal,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Eq => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Ne if is_float => lower_float_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::FloatCC::NotEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Ne => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Lt if is_float => lower_float_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::FloatCC::LessThan,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Lt => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Gt if is_float => lower_float_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::FloatCC::GreaterThan,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Gt => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Le if is_float => lower_float_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::FloatCC::LessThanOrEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Le => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Ge if is_float => lower_float_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::FloatCC::GreaterThanOrEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::Ge => lower_int_comparison(
                    builder,
                    cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                    l,
                    r,
                ),
                omni_mir::ir::BinOp::BitAnd => Ok(builder.ins().band(l, r)),
                omni_mir::ir::BinOp::BitOr => Ok(builder.ins().bor(l, r)),
                omni_mir::ir::BinOp::BitXor => Ok(builder.ins().bxor(l, r)),
                omni_mir::ir::BinOp::Shl => Ok(builder.ins().ishl(l, r)),
                omni_mir::ir::BinOp::Shr => Ok(builder.ins().sshr(l, r)),
            }
        }
        omni_mir::ir::Rvalue::Aggregate { kind, .. } => {
            return Err(format!(
                "Codegen error: aggregate {:?} requires an explicit target layout/ABI contract",
                kind
            ));
        }
        omni_mir::ir::Rvalue::Field { field, .. } => {
            return Err(format!(
                "Codegen error: field projection '{}' requires aggregate layout metadata",
                field
            ));
        }
        omni_mir::ir::Rvalue::Struct { name, .. } => {
            return Err(format!(
                "Codegen error: struct constructor '{}' requires target aggregate layout metadata",
                name
            ));
        }
        omni_mir::ir::Rvalue::EnumVariant { enum_name, variant, .. } => {
            return Err(format!(
                "Codegen error: enum constructor '{}::{}' requires target tagged-layout metadata",
                enum_name, variant
            ));
        }
        omni_mir::ir::Rvalue::Range { .. } => {
            return Err("Codegen error: range value representation requires target layout metadata".into());
        }
        omni_mir::ir::Rvalue::Index { .. } => {
            return Err("Codegen error: index projection requires aggregate layout metadata".into());
        }

        omni_mir::ir::Rvalue::Cast { operand, from, to } => {
            let value = lower_operand_to_cl(builder, operand, variables)?;
            let from_float = matches!(tcx.get(*from), omni_mir::TyKind::Float);
            let to_float = matches!(tcx.get(*to), omni_mir::TyKind::Float);
            match (from_float, to_float) {
                (false, false) => Ok(value),
                (false, true) => Ok(builder.ins().fcvt_from_sint(types::F64, value)),
                (true, false) => Ok(builder.ins().fcvt_to_sint(types::I64, value)),
                (true, true) => Ok(value),
            }
        }
        omni_mir::ir::Rvalue::UnaryOp(op, operand) => {
            let val = lower_operand_to_cl(builder, operand, variables)?;
            match op {
                omni_mir::ir::UnOp::Neg => {
                    if builder.func.dfg.value_type(val) == types::F64 {
                        Ok(builder.ins().fneg(val))
                    } else {
                        Ok(builder.ins().ineg(val))
                    }
                },
                omni_mir::ir::UnOp::Not => {
                    let one = builder.ins().iconst(types::I64, 1);
                    Ok(builder.ins().bxor(val, one))
                }
                omni_mir::ir::UnOp::BitNot => {
                    let all_ones = builder.ins().iconst(types::I64, -1);
                    Ok(builder.ins().bxor(val, all_ones))
                },
            }
        }
    }
}

fn lower_float_comparison(
    builder: &mut FunctionBuilder,
    condition: cranelift_codegen::ir::condcodes::FloatCC,
    lhs: cranelift_codegen::ir::Value,
    rhs: cranelift_codegen::ir::Value,
) -> Result<cranelift_codegen::ir::Value, String> {
    let predicate = builder.ins().fcmp(condition, lhs, rhs);
    let one = builder.ins().iconst(types::I64, 1);
    let zero = builder.ins().iconst(types::I64, 0);
    Ok(builder.ins().select(predicate, one, zero))
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
    fn test_cfg_join_uses_cranelift_variable_ssa() {
        use std::fs;
        use std::process::Command;
        use std::sync::atomic::{AtomicU64, Ordering};

        let source = ast::GenericFnDef {
            name: "main".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: ast::TypeSpec::Int,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Literal(ast::Lit::Int(0)),
        };
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);

        let mut locals = index_vec::IndexVec::new();
        let ret = locals
            .push(omni_mir::ir::LocalDecl { name: Some("_return".to_string()), ty: Some(int) });

        let mut blocks = index_vec::IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(omni_mir::ir::Terminator::SwitchInt {
                discr: omni_mir::ir::Operand::Constant(omni_mir::ir::Constant::Lit(ast::Lit::Int(
                    1,
                ))),
                targets: vec![(1, omni_mir::ir::BasicBlock::from_usize(1))],
                otherwise: omni_mir::ir::BasicBlock::from_usize(2),
            }),
        });
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![omni_mir::ir::Statement::Assign(
                omni_mir::ir::Place { local: ret },
                omni_mir::ir::Rvalue::Use(omni_mir::ir::Operand::Constant(
                    omni_mir::ir::Constant::Lit(ast::Lit::Int(41)),
                )),
            )],
            terminator: Some(omni_mir::ir::Terminator::Goto(omni_mir::ir::BasicBlock::from_usize(
                3,
            ))),
        });
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![omni_mir::ir::Statement::Assign(
                omni_mir::ir::Place { local: ret },
                omni_mir::ir::Rvalue::Use(omni_mir::ir::Operand::Constant(
                    omni_mir::ir::Constant::Lit(ast::Lit::Int(7)),
                )),
            )],
            terminator: Some(omni_mir::ir::Terminator::Goto(omni_mir::ir::BasicBlock::from_usize(
                3,
            ))),
        });
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(omni_mir::ir::Terminator::Return),
        });

        let mir = omni_mir::ir::MirProgram {
            tcx,
            functions: vec![omni_mir::ir::MirFunction {
                name: "main".to_string(),
                params: vec![],
                return_place: ret,
                return_type: ast::TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals },
            }],
        };
        omni_verify::MirVerifier::verify_program(&mir).expect("hand-built CFG must verify");

        let source_program = omni_mir::MonomorphizedProgram { functions: vec![source] };
        let object = compile_mir_program(&source_program, &mir).expect("CFG native emission");

        static SEQ: AtomicU64 = AtomicU64::new(0);
        let stem = format!(
            "omni-codegen-cfg-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let dir = std::env::temp_dir();
        let object_path = dir.join(format!("{stem}.o"));
        let exe_path = dir.join(&stem);
        fs::write(&object_path, object).expect("object write");

        let link = Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&exe_path)
            .status()
            .expect("cc must be available");
        assert!(link.success(), "link failed: {link}");

        let run = Command::new(&exe_path).status().expect("executable must run");
        assert_eq!(run.code(), Some(41));

        fs::remove_file(object_path).ok();
        fs::remove_file(exe_path).ok();
    }

    #[test]
    fn test_compile_mir_program_rejects_source_mir_semantic_type_mismatch() {
        let source = ast::GenericFnDef {
            name: "main".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: ast::TypeSpec::Bool,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Literal(ast::Lit::Bool(true)),
        };
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);

        let mut locals = index_vec::IndexVec::new();
        let ret = locals
            .push(omni_mir::ir::LocalDecl { name: Some("_return".to_string()), ty: Some(int) });
        let mut blocks = index_vec::IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![omni_mir::ir::Statement::Assign(
                omni_mir::ir::Place { local: ret },
                omni_mir::ir::Rvalue::Use(omni_mir::ir::Operand::Constant(
                    omni_mir::ir::Constant::Lit(ast::Lit::Int(1)),
                )),
            )],
            terminator: Some(omni_mir::ir::Terminator::Return),
        });

        let mir = omni_mir::ir::MirProgram {
            tcx,
            functions: vec![omni_mir::ir::MirFunction {
                name: "main".to_string(),
                params: vec![],
                return_place: ret,
                return_type: ast::TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals },
            }],
        };
        omni_verify::MirVerifier::verify_program(&mir)
            .expect("MIR fixture must be internally typed");

        let err =
            compile_mir_program(&omni_mir::MonomorphizedProgram { functions: vec![source] }, &mir)
                .expect_err("source Bool and MIR Int must not share an ABI class");
        assert!(err.contains("source/MIR semantic type mismatch"));
    }

    #[test]
    fn test_compile_mir_program_rejects_cleanup_edge() {
        let touch = ast::GenericFnDef {
            name: "touch".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: ast::TypeSpec::Unit,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Block(vec![]),
        };
        let main = ast::GenericFnDef {
            name: "main".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: ast::TypeSpec::Unit,
            effects: Default::default(),
            capabilities: vec![],
            body: ast::Expr::Block(vec![]),
        };

        let mut tcx = omni_mir::TyCtxt::new();
        let unit = tcx.intern(omni_mir::TyKind::Unit);

        let mut touch_locals = index_vec::IndexVec::new();
        let touch_ret = touch_locals
            .push(omni_mir::ir::LocalDecl { name: Some("_return".to_string()), ty: Some(unit) });
        let mut touch_blocks = index_vec::IndexVec::new();
        touch_blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(omni_mir::ir::Terminator::Return),
        });

        let mut main_locals = index_vec::IndexVec::new();
        let main_ret = main_locals
            .push(omni_mir::ir::LocalDecl { name: Some("_return".to_string()), ty: Some(unit) });
        let mut main_blocks = index_vec::IndexVec::new();
        main_blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(omni_mir::ir::Terminator::Call {
                func: omni_mir::ir::Operand::Constant(omni_mir::ir::Constant::FnRef(
                    "touch".to_string(),
                )),
                args: vec![],
                destination: None,
                target: omni_mir::ir::BasicBlock::from_usize(1),
                cleanup: Some(omni_mir::ir::BasicBlock::from_usize(2)),
            }),
        });
        main_blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(omni_mir::ir::Terminator::Return),
        });
        main_blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(omni_mir::ir::Terminator::Return),
        });

        let mir = omni_mir::ir::MirProgram {
            tcx,
            functions: vec![
                omni_mir::ir::MirFunction {
                    name: "touch".to_string(),
                    params: vec![],
                    return_place: touch_ret,
                    return_type: ast::TypeSpec::Unit,
                    body: omni_mir::ir::Body { blocks: touch_blocks, local_decls: touch_locals },
                },
                omni_mir::ir::MirFunction {
                    name: "main".to_string(),
                    params: vec![],
                    return_place: main_ret,
                    return_type: ast::TypeSpec::Unit,
                    body: omni_mir::ir::Body { blocks: main_blocks, local_decls: main_locals },
                },
            ],
        };

        let err = compile_mir_program(&MonomorphizedProgram { functions: vec![touch, main] }, &mir)
            .expect_err("native backend must not erase a MIR cleanup edge");
        assert!(err.contains("unsupported cleanup/unwind edge"));
    }

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
