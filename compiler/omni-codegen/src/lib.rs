//! Native Code Generation via Cranelift for Omni.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::{types, AbiParam, Signature, StackSlot, StackSlotData, StackSlotKind};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;
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
        omni_mir::TyKind::Reference { .. } => {
            Err("Codegen error: reference ABI requires pointer storage and ownership lowering"
                .into())
        }
        other => Err(format!(
            "Codegen error: native backend does not yet support MIR ABI type {:?}",
            other
        )),
    }
}

/// Native storage class for a single MIR local.
///
/// Stage 4C boundary: scalar locals keep Cranelift SSA `Variable` storage and
/// the existing scalar execution path is untouched. Aggregate locals take
/// Cranelift `StackSlot` storage, and the slot's stored `TypeLayout` is the
/// single authority for every field/element offset derived from it. The stored
/// layout means projection walks never recompute offsets ad hoc.
#[derive(Debug, Clone)]
enum NativeStorage {
    /// Scalar local: an SSA value.
    Scalar(Variable),
    /// Aggregate local: a stack base address plus its layout.
    Aggregate { slot: StackSlot, layout: TypeLayout },
}

/// Stage 4C storage classification, decided by the concrete MIR type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageClass {
    Scalar,
    Aggregate,
}

/// Classifies a concrete MIR local type into Stage 4C native storage.
///
/// This is a function of the type, not of how the local happens to be used:
/// `Int`/`Float`/`Bool`/`Char`/`Byte` are scalar, `Tuple`/`Array`/struct types
/// are aggregate. `Unit` has no storage (`None`). Any type without a defined
/// native representation is rejected rather than given an invented one.
///
/// Aggregate function parameters and returns use the Stage 4E address-based
/// convention after local storage classification; the representation is
/// therefore still derived from the same concrete TargetLayout.

fn classify_local_storage(
    tcx: &omni_mir::TyCtxt,
    ty: omni_mir::Ty,
) -> Result<Option<StorageClass>, String> {
    match tcx.get(ty) {
        omni_mir::TyKind::Unit => Ok(None),
        omni_mir::TyKind::Int
        | omni_mir::TyKind::Bool
        | omni_mir::TyKind::Byte
        | omni_mir::TyKind::Char
        | omni_mir::TyKind::Float => Ok(Some(StorageClass::Scalar)),
        omni_mir::TyKind::Tuple(_) | omni_mir::TyKind::Array(..) | omni_mir::TyKind::Struct(..) => {
            Ok(Some(StorageClass::Aggregate))
        }
        other => Err(format!(
            "Codegen error: native backend does not yet support MIR storage type {:?}",
            other
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeParamAbi {
    Direct(cranelift_codegen::ir::Type),
    AggregateAddress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeReturnAbi {
    Unit,
    Direct(cranelift_codegen::ir::Type),
    AggregateAddress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeFunctionAbi {
    return_abi: NativeReturnAbi,
    params: Vec<NativeParamAbi>,
}

/// Derives the backend-internal aggregate-capable calling convention from
/// concrete verified MIR.
///
/// This is deliberately not the stable external omni_v1 ABI. Aggregate
/// parameters are passed as pointers to storage and aggregate returns use a
/// hidden result pointer with no direct Cranelift return value.
fn native_function_abi(
    mir_func: &omni_mir::ir::MirFunction,
    source_def: &omni_mir::ast::GenericFnDef,
    tcx: &omni_mir::TyCtxt,
    layout: &TargetLayout<'_>,
    layout_workspace: &mut omni_mir::TyCtxt,
) -> Result<NativeFunctionAbi, String> {
    if mir_func.params.len() != source_def.params.len() {
        return Err(format!(
            "Codegen error: MIR/source parameter count mismatch for '{}'",
            mir_func.name
        ));
    }

    let return_ty = mir_func.body.local_decls[mir_func.return_place].ty.ok_or_else(|| {
        format!(
            "Codegen error: return local {:?} has no type for '{}'",
            mir_func.return_place, mir_func.name
        )
    })?;
    ensure_source_mir_type_match(
        tcx,
        &source_def.return_type,
        return_ty,
        &format!("return type of '{}'", mir_func.name),
    )?;

    let return_abi = match tcx.get(return_ty) {
        omni_mir::TyKind::Unit => NativeReturnAbi::Unit,
        omni_mir::TyKind::Int
        | omni_mir::TyKind::Bool
        | omni_mir::TyKind::Byte
        | omni_mir::TyKind::Char
        | omni_mir::TyKind::Float => {
            NativeReturnAbi::Direct(native_abi_type_from_ty(tcx, return_ty)?.ok_or_else(|| {
                format!("Codegen error: scalar return type for '{}' has no ABI type", mir_func.name)
            })?)
        }
        omni_mir::TyKind::Tuple(_) | omni_mir::TyKind::Array(..) | omni_mir::TyKind::Struct(..) => {
            layout.aggregate_layout(layout_workspace, return_ty).map_err(|e| {
                format!(
                    "Codegen error: aggregate return type of '{}' has no target layout: {}",
                    mir_func.name, e
                )
            })?;
            NativeReturnAbi::AggregateAddress
        }
        other => {
            return Err(format!(
                "Codegen error: native backend does not yet support return ABI type {:?} in '{}'",
                other, mir_func.name
            ));
        }
    };

    let mut params = Vec::with_capacity(source_def.params.len());
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
            tcx,
            source_spec,
            mir_ty,
            &format!("parameter {} of '{}'", param_index, mir_func.name),
        )?;

        let abi = match tcx.get(mir_ty) {
            omni_mir::TyKind::Unit => {
                return Err(format!(
                    "Codegen error: Unit parameter {} is not representable in native ABI for '{}'",
                    param_index, mir_func.name
                ));
            }
            omni_mir::TyKind::Int
            | omni_mir::TyKind::Bool
            | omni_mir::TyKind::Byte
            | omni_mir::TyKind::Char
            | omni_mir::TyKind::Float => {
                NativeParamAbi::Direct(native_abi_type_from_ty(tcx, mir_ty)?.ok_or_else(|| {
                    format!(
                        "Codegen error: scalar parameter {} of '{}' has no ABI type",
                        param_index, mir_func.name
                    )
                })?)
            }
            omni_mir::TyKind::Tuple(_)
            | omni_mir::TyKind::Array(..)
            | omni_mir::TyKind::Struct(..) => {
                layout.aggregate_layout(layout_workspace, mir_ty).map_err(|e| {
                    format!(
                        "Codegen error: aggregate parameter {} of '{}' has no target layout: {}",
                        param_index, mir_func.name, e
                    )
                })?;
                NativeParamAbi::AggregateAddress
            }
            other => {
                return Err(format!(
                    "Codegen error: native backend does not yet support parameter {} ABI type {:?} in '{}'",
                    param_index, other, mir_func.name
                ));
            }
        };
        params.push(abi);
    }

    Ok(NativeFunctionAbi { return_abi, params })
}

fn signature_for_native_abi(module: &ObjectModule, abi: &NativeFunctionAbi) -> Signature {
    let pointer_type = module.isa().pointer_type();
    let mut sig = Signature::new(module.isa().default_call_conv());
    if matches!(abi.return_abi, NativeReturnAbi::AggregateAddress) {
        sig.params.push(AbiParam::new(pointer_type));
    }
    if let NativeReturnAbi::Direct(ty) = abi.return_abi {
        sig.returns.push(AbiParam::new(ty));
    }
    for param in &abi.params {
        sig.params.push(AbiParam::new(match param {
            NativeParamAbi::Direct(ty) => *ty,
            NativeParamAbi::AggregateAddress => pointer_type,
        }));
    }
    sig
}

fn ensure_source_mir_type_match(
    tcx: &omni_mir::TyCtxt,
    spec: &omni_mir::ast::TypeSpec,
    ty: omni_mir::Ty,
    context: &str,
) -> Result<(), String> {
    use omni_mir::ast::TypeSpec;
    use omni_mir::TyKind;

    // Structural agreement between the source declaration and the MIR type.
    // Agreement is semantic, not ABI representability: an aggregate matches
    // its identical aggregate here and is then rejected downstream by the ABI
    // gate with an ABI reason, rather than being misreported as a semantic
    // mismatch.
    fn matches(tcx: &omni_mir::TyCtxt, spec: &TypeSpec, ty: omni_mir::Ty) -> bool {
        match (spec, tcx.get(ty)) {
            (TypeSpec::Unit, TyKind::Unit)
            | (TypeSpec::Int, TyKind::Int)
            | (TypeSpec::Bool, TyKind::Bool)
            | (TypeSpec::Byte, TyKind::Byte)
            | (TypeSpec::Char, TyKind::Char)
            | (TypeSpec::Float, TyKind::Float) => true,
            (TypeSpec::Tuple(specs), TyKind::Tuple(tys)) => {
                specs.len() == tys.len()
                    && specs.iter().zip(tys.iter()).all(|(s, t)| matches(tcx, s, *t))
            }
            (TypeSpec::Array(spec, len), TyKind::Array(ty, length)) => {
                len == length && matches(tcx, spec, *ty)
            }
            (TypeSpec::Struct(spec_name, spec_args), TyKind::Struct(ty_name, ty_args)) => {
                spec_name == ty_name
                    && spec_args.len() == ty_args.len()
                    && spec_args.iter().zip(ty_args.iter()).all(|(s, t)| matches(tcx, s, *t))
            }
            _ => false,
        }
    }
    if matches(tcx, spec, ty) {
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
    compile_monomorphized_program_with_structs(prog, HashMap::new())
}

/// Enforces MIR lowering semantic gate and MirVerifier before native emission, with
/// struct declarations available for field-projection typing.
pub fn compile_monomorphized_program_with_structs(
    prog: &omni_mir::MonomorphizedProgram,
    struct_defs: HashMap<String, omni_mir::ast::StructDef>,
) -> Result<Vec<u8>, String> {
    let mut lowering = omni_mir::lower::LoweringContext::new();
    lowering.set_struct_defs(struct_defs);
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

    let target_facts = TargetFacts::from_isa(&isa);
    let builder = ObjectBuilder::new(
        isa,
        "omni_module".to_string(),
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| format!("Object builder error: {}", e))?;
    let mut module = ObjectModule::new(builder);

    let mut function_ids = std::collections::HashMap::new();

    // Stage 4C: TargetLayout is the single authority for aggregate size,
    // alignment and field/element offsets. `layout_tcx` is a workspace clone
    // of the program context: laying out a struct interns its substituted
    // field types, which needs `&mut`, while the MIR program itself is only
    // borrowed. Cloning preserves every existing `Ty` index, so types read
    // from `mir_prog.tcx` stay valid in the workspace.
    let layout_engine = TargetLayout::new(target_facts, &mir_prog.struct_defs);
    let mut layout_tcx = mir_prog.tcx.clone();

    // Precompute one concrete backend ABI for every function, then predeclare
    // every function with exactly that signature before emitting any body.
    let pointer_type = module.isa().pointer_type();
    let mut function_abis = std::collections::HashMap::new();
    for mir_func in &mir_prog.functions {
        let source_def =
            prog.functions.iter().find(|f| f.name == mir_func.name).ok_or_else(|| {
                format!("Codegen error: missing source function '{}'", mir_func.name)
            })?;
        let abi = native_function_abi(
            mir_func,
            source_def,
            &mir_prog.tcx,
            &layout_engine,
            &mut layout_tcx,
        )?;
        let sig = signature_for_native_abi(&module, &abi);
        let func_id = module
            .declare_function(&mir_func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Function declaration error: {}", e))?;
        function_abis.insert(mir_func.name.clone(), abi);
        function_ids.insert(mir_func.name.clone(), func_id);
    }

    for mir_func in &mir_prog.functions {
        let func_id = *function_ids.get(&mir_func.name).ok_or_else(|| {
            format!("Codegen error: function '{}' was not predeclared", mir_func.name)
        })?;

        let abi = function_abis.get(&mir_func.name).ok_or_else(|| {
            format!("Codegen error: ABI for '{}' was not precomputed", mir_func.name)
        })?;
        let mut ctx = module.make_context();
        ctx.func.signature = signature_for_native_abi(&module, abi);
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

        // Stage 4C: storage is classified by concrete MIR type. Scalars keep
        // SSA Variables; aggregates take StackSlots sized and aligned by
        // TargetLayout. Unit has no storage.
        let mut storage = std::collections::HashMap::new();
        for (local_idx, local_decl) in mir_func.body.local_decls.iter().enumerate() {
            let local = omni_mir::ir::Local::from_usize(local_idx);
            let ty = local_decl
                .ty
                .ok_or_else(|| format!("Codegen error: MIR local {:?} has no type", local))?;
            match classify_local_storage(&mir_prog.tcx, ty)? {
                None => {}
                Some(StorageClass::Scalar) => {
                    let native_ty =
                        native_abi_type_from_ty(&mir_prog.tcx, ty)?.ok_or_else(|| {
                            format!(
                                "Codegen error: scalar local {:?} has no native value type",
                                local
                            )
                        })?;
                    let variable = Variable::from_u32(local_idx as u32);
                    builder.declare_var(variable, native_ty);
                    storage.insert(local, NativeStorage::Scalar(variable));
                }
                Some(StorageClass::Aggregate) => {
                    let layout =
                        layout_engine.aggregate_layout(&mut layout_tcx, ty).map_err(|e| {
                            format!(
                                "Codegen error: cannot lay out aggregate local {:?} in '{}': {}",
                                local, mir_func.name, e
                            )
                        })?;
                    if !layout.align.is_power_of_two() || layout.align == 0 {
                        return Err(format!(
                            "Codegen error: aggregate local {:?} in '{}' has non-power-of-two alignment {}",
                            local, mir_func.name, layout.align
                        ));
                    }
                    let size = u32::try_from(layout.size).map_err(|_| {
                        format!(
                            "Codegen error: aggregate local {:?} in '{}' exceeds the stack-slot size range",
                            local, mir_func.name
                        )
                    })?;
                    let slot = builder.create_sized_stack_slot(StackSlotData {
                        kind: StackSlotKind::ExplicitSlot,
                        size,
                        align_shift: layout.align.trailing_zeros() as u8,
                    });
                    storage.insert(local, NativeStorage::Aggregate { slot, layout });
                }
            }
        }

        let mut emitter = PlaceEmitter {
            tcx: &mir_prog.tcx,
            layout_workspace: &mut layout_tcx,
            struct_defs: &mir_prog.struct_defs,
            layout: &layout_engine,
            storage: &storage,
            body: &mir_func.body,
            func_name: &mir_func.name,
            pointer_type,
        };

        let mut abi_param_cursor = 0usize;
        let result_pointer = if matches!(abi.return_abi, NativeReturnAbi::AggregateAddress) {
            let value = *builder
                .block_params(entry_cl_block)
                .get(abi_param_cursor)
                .ok_or_else(|| {
                    format!(
                        "Codegen error: aggregate-returning function '{}' is missing its hidden result pointer",
                        mir_func.name
                    )
                })?;
            abi_param_cursor += 1;
            Some(value)
        } else {
            None
        };

        for (p_idx, &param_local) in mir_func.params.iter().enumerate() {
            let cl_val = *builder
                .block_params(entry_cl_block)
                .get(abi_param_cursor)
                .ok_or_else(|| {
                    format!(
                        "Codegen error: function '{}' has no Cranelift parameter for MIR parameter {}",
                        mir_func.name, p_idx
                    )
                })?;
            abi_param_cursor += 1;
            match abi.params.get(p_idx).ok_or_else(|| {
                format!(
                    "Codegen error: missing ABI entry for parameter {} of '{}'",
                    p_idx, mir_func.name
                )
            })? {
                NativeParamAbi::Direct(_) => {
                    let variable = match emitter.storage.get(&param_local) {
                        Some(NativeStorage::Scalar(variable)) => *variable,
                        _ => {
                            return Err(format!(
                                "Codegen error: scalar parameter local {:?} has no scalar native representation in '{}'",
                                param_local, mir_func.name
                            ));
                        }
                    };
                    builder.def_var(variable, cl_val);
                }
                NativeParamAbi::AggregateAddress => {
                    let site = resolve_aggregate_operand(
                        &mut emitter,
                        &omni_mir::ir::Operand::Copy(omni_mir::ir::Place::local(param_local)),
                        &format!("aggregate parameter {} of '{}'", p_idx, mir_func.name),
                    )?;
                    copy_aggregate_from_pointer(
                        &mut builder,
                        &mut emitter,
                        &site,
                        cl_val,
                        &format!(
                            "initialization of aggregate parameter {} in '{}'",
                            p_idx, mir_func.name
                        ),
                    )?;
                }
            }
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
                        emit_assign(&mut builder, &mut emitter, place, rval)?;
                    }
                    omni_mir::ir::Statement::Assume(_) | omni_mir::ir::Statement::Drop(_) => {}
                    omni_mir::ir::Statement::BoundsCheck { index, length } => {
                        // Generate runtime bounds check: trap if index >= 0 && index < length is false
                        let storage = emitter.storage.get(&index).ok_or_else(|| {
                            format!(
                                "Codegen error: bounds check index local {:?} has no storage",
                                index
                            )
                        })?;
                        let index_val = match storage {
                            NativeStorage::Scalar(variable) => builder.use_var(*variable),
                            NativeStorage::Aggregate { .. } => {
                                return Err(
                                    "Codegen error: bounds check index must be scalar".to_string()
                                );
                            }
                        };

                        // Check if index >= 0
                        let is_non_negative = builder.ins().icmp_imm(
                            cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                            index_val,
                            0,
                        );

                        // Check if index < length
                        let is_less_than_length = builder.ins().icmp_imm(
                            cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                            index_val,
                            *length as i64,
                        );

                        // Both conditions must be true: index >= 0 && index < length
                        let is_valid = builder.ins().band(is_non_negative, is_less_than_length);

                        // Trap if the index is out of bounds
                        let valid_block = builder.create_block();
                        let trap_block = builder.create_block();

                        builder.ins().brif(is_valid, valid_block, &[], trap_block, &[]);
                        builder.switch_to_block(trap_block);
                        builder.ins().trap(cranelift_codegen::ir::TrapCode::User(1));
                        builder.switch_to_block(valid_block);
                    }
                }
            }

            let term = mir_block.terminator.as_ref().ok_or_else(|| {
                format!(
                    "Codegen error: MIR block {:?} in '{}' is unterminated",
                    b_idx, mir_func.name
                )
            })?;

            match term {
                omni_mir::ir::Terminator::Return => match abi.return_abi {
                    NativeReturnAbi::Unit => {
                        builder.ins().return_(&[]);
                    }
                    NativeReturnAbi::Direct(_) => {
                        let variable = match emitter.storage.get(&mir_func.return_place) {
                            Some(NativeStorage::Scalar(variable)) => *variable,
                            _ => {
                                return Err(format!(
                                    "Codegen error: scalar return local {:?} has no scalar native representation in '{}'",
                                    mir_func.return_place, mir_func.name
                                ));
                            }
                        };
                        let ret_val = builder.use_var(variable);
                        builder.ins().return_(&[ret_val]);
                    }
                    NativeReturnAbi::AggregateAddress => {
                        let result_pointer = result_pointer.ok_or_else(|| {
                            format!(
                                "Codegen error: aggregate-returning function '{}' lost its hidden result pointer",
                                mir_func.name
                            )
                        })?;
                        let site = resolve_aggregate_operand(
                            &mut emitter,
                            &omni_mir::ir::Operand::Copy(omni_mir::ir::Place::local(
                                mir_func.return_place,
                            )),
                            &format!("aggregate return of '{}'", mir_func.name),
                        )?;
                        copy_aggregate_to_pointer(
                            &mut builder,
                            &mut emitter,
                            &site,
                            result_pointer,
                            &format!("aggregate return of '{}'", mir_func.name),
                        )?;
                        builder.ins().return_(&[]);
                    }
                },
                omni_mir::ir::Terminator::Goto(target) => {
                    let target_cl = *cl_blocks.get(&target.index()).ok_or_else(|| {
                        format!("Codegen error: undefined goto target {:?}", target)
                    })?;
                    builder.ins().jump(target_cl, &[]);
                }
                omni_mir::ir::Terminator::SwitchInt { discr, targets, otherwise } => {
                    let discr_val = lower_operand_to_cl(&mut builder, &mut emitter, discr)?;
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

                    let callee_abi = function_abis.get(fn_name).ok_or_else(|| {
                        format!("Codegen error: ABI for callee '{}' was not precomputed", fn_name)
                    })?;
                    if args.len() != callee_abi.params.len() {
                        return Err(format!(
                            "Codegen error: call '{}' ABI parameter count mismatch: expected {}, found {}",
                            fn_name,
                            callee_abi.params.len(),
                            args.len()
                        ));
                    }

                    let mut call_args = Vec::with_capacity(args.len() + 1);
                    if matches!(callee_abi.return_abi, NativeReturnAbi::AggregateAddress) {
                        let destination = destination.as_ref().ok_or_else(|| {
                            format!(
                                "Codegen error: aggregate-returning call '{}' requires a destination",
                                fn_name
                            )
                        })?;
                        let result_site = resolve_aggregate_operand(
                            &mut emitter,
                            &omni_mir::ir::Operand::Copy(destination.clone()),
                            &format!("aggregate call destination for '{}'", fn_name),
                        )?;
                        call_args.push(builder.ins().stack_addr(
                            emitter.pointer_type,
                            result_site.slot,
                            stack_offset(result_site.offset, emitter.func_name)?,
                        ));
                    }

                    for (arg_index, (arg, param_abi)) in
                        args.iter().zip(callee_abi.params.iter()).enumerate()
                    {
                        match param_abi {
                            NativeParamAbi::Direct(expected) => {
                                let value = lower_operand_to_cl(&mut builder, &mut emitter, arg)?;
                                if builder.func.dfg.value_type(value) != *expected {
                                    return Err(format!(
                                        "Codegen error: call '{}' argument {} ABI type mismatch: expected {:?}, found {:?}",
                                        fn_name,
                                        arg_index,
                                        expected,
                                        builder.func.dfg.value_type(value)
                                    ));
                                }
                                call_args.push(value);
                            }
                            NativeParamAbi::AggregateAddress => {
                                let site = resolve_aggregate_operand(
                                    &mut emitter,
                                    arg,
                                    &format!("aggregate argument {} to '{}'", arg_index, fn_name),
                                )?;
                                call_args.push(builder.ins().stack_addr(
                                    emitter.pointer_type,
                                    site.slot,
                                    stack_offset(site.offset, emitter.func_name)?,
                                ));
                            }
                        }
                    }

                    let callee_id = *function_ids.get(fn_name).ok_or_else(|| {
                        format!("Codegen error: callee '{}' was not predeclared", fn_name)
                    })?;
                    let local_callee = module.declare_func_in_func(callee_id, builder.func);
                    let call_inst = builder.ins().call(local_callee, &call_args);
                    let results = builder.inst_results(call_inst);

                    match (&callee_abi.return_abi, destination) {
                        (NativeReturnAbi::AggregateAddress, Some(_)) => {
                            if !results.is_empty() {
                                return Err(format!(
                                    "Codegen error: aggregate-returning call '{}' unexpectedly produced direct results",
                                    fn_name
                                ));
                            }
                        }
                        (NativeReturnAbi::AggregateAddress, None) => {
                            return Err(format!(
                                "Codegen error: aggregate-returning call '{}' requires a destination",
                                fn_name
                            ));
                        }
                        (NativeReturnAbi::Direct(_), Some(destination)) => {
                            if results.len() != 1 {
                                return Err(format!(
                                    "Codegen error: direct-value call '{}' result count mismatch for destination {:?}",
                                    fn_name, destination
                                ));
                            }
                            if !destination.is_local() {
                                return Err(format!(
                                    "Codegen error: direct-value call '{}' destination {:?} must be a whole scalar local",
                                    fn_name, destination
                                ));
                            }
                            let variable = match emitter.storage.get(&destination.local) {
                                Some(NativeStorage::Scalar(variable)) => *variable,
                                _ => {
                                    return Err(format!(
                                        "Codegen error: direct-value call '{}' destination {:?} has no scalar native storage",
                                        fn_name, destination
                                    ));
                                }
                            };
                            builder.def_var(variable, results[0]);
                        }
                        (NativeReturnAbi::Direct(_), None) => {
                            return Err(format!(
                                "Codegen error: value-returning call '{}' has no destination",
                                fn_name
                            ));
                        }
                        (NativeReturnAbi::Unit, Some(destination)) => {
                            return Err(format!(
                                "Codegen error: Unit-returning call '{}' unexpectedly has destination {:?}",
                                fn_name, destination
                            ));
                        }
                        (NativeReturnAbi::Unit, None) => {
                            if !results.is_empty() {
                                return Err(format!(
                                    "Codegen error: Unit-returning call '{}' unexpectedly produced results",
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

/// Per-function lowering context for Stage 4C place resolution.
///
/// `tcx` is the program context the MIR was verified against (read-only).
/// `layout_workspace` is a clone used whenever computing a projected type
/// needs to intern substituted struct field types. `layout` is the
/// [`TargetLayout`] authority for every offset; nothing here invents one.
struct PlaceEmitter<'a, 'l> {
    tcx: &'a omni_mir::TyCtxt,
    layout_workspace: &'a mut omni_mir::TyCtxt,
    struct_defs: &'a HashMap<String, omni_mir::ast::StructDef>,
    layout: &'a TargetLayout<'l>,
    storage: &'a HashMap<omni_mir::ir::Local, NativeStorage>,
    body: &'a omni_mir::ir::Body,
    func_name: &'a str,
    pointer_type: cranelift_codegen::ir::Type,
}

/// Emits `place = rvalue`, honouring the Stage 4C storage split.
///
/// A whole scalar local keeps the historical `def_var` path. A whole
/// aggregate local is initialized element-wise from an aggregate rvalue or
/// copied from another aggregate. A projected place is a scalar store through
/// the layout-derived address; storing an aggregate value through a projection
/// is rejected because a projection names one scalar sublocation.
fn emit_assign(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    place: &omni_mir::ir::Place,
    rval: &omni_mir::ir::Rvalue,
) -> Result<(), String> {
    let storage = emitter.storage.get(&place.local).cloned().ok_or_else(|| {
        format!(
            "Codegen error: assignment target local {:?} in '{}' has no native representation",
            place.local, emitter.func_name
        )
    })?;
    match storage {
        NativeStorage::Scalar(variable) => {
            if !place.is_local() {
                return Err(format!(
                    "Codegen error: assignment to {} in '{}' projects into scalar local {:?}, which has no sublocations",
                    place, emitter.func_name, place.local
                ));
            }
            let val = lower_rvalue_to_cl(builder, emitter, rval)?;
            builder.def_var(variable, val);
            Ok(())
        }
        NativeStorage::Aggregate { slot, layout } => {
            if place.is_local() {
                let ty = emitter.body.local_decls[place.local].ty.ok_or_else(|| {
                    format!(
                        "Codegen error: aggregate local {:?} in '{}' has no type",
                        place.local, emitter.func_name
                    )
                })?;
                let site = AggregateSite { slot, offset: 0, ty, layout };
                emit_aggregate_store(
                    builder,
                    emitter,
                    &site,
                    rval,
                    &format!("initialization of aggregate local {:?}", place.local),
                )
            } else {
                let (address, _) =
                    resolve_place_address(builder, emitter, place, "assignment target")?;
                let val = lower_rvalue_to_cl(builder, emitter, rval)?;
                match address {
                    NativeAddress::Stack { slot, offset } => {
                        builder.ins().stack_store(
                            val,
                            slot,
                            stack_offset(offset, emitter.func_name)?,
                        );
                    }
                    NativeAddress::Pointer(ptr) => {
                        builder.ins().store(
                            cranelift_codegen::ir::MemFlags::new(),
                            val,
                            ptr,
                            0,
                        );
                    }
                }
                Ok(())
            }
        }
    }
}

/// One end of an aggregate move: where it lives plus what it is.
///
/// Bundling the slot, offset, MIR type and layout keeps the copy and store
/// helpers honest about what travels together instead of threading four
/// parallel arguments. The layout is owned (a small clone) so a site never
/// borrows the lowering context it is passed alongside.
#[derive(Debug, Clone)]
struct AggregateSite {
    slot: StackSlot,
    offset: u64,
    ty: omni_mir::Ty,
    layout: TypeLayout,
}

impl AggregateSite {
    /// Derives the child site for one layout part.
    fn child(
        &self,
        part_ty: omni_mir::Ty,
        part: &PartLayout,
        context: &str,
    ) -> Result<Self, String> {
        let offset = self
            .offset
            .checked_add(part.offset)
            .ok_or_else(|| format!("Codegen error: {} overflows its aggregate offset", context))?;
        Ok(Self { slot: self.slot, offset, ty: part_ty, layout: part.layout.clone() })
    }
}

/// Converts a layout byte offset into a Cranelift stack offset.
///
/// Stack slots in Stage 4C are small compiler-managed locals, so an offset
/// that does not fit is a symptom of a corrupt layout, not a large object.
fn stack_offset(offset: u64, func_name: &str) -> Result<i32, String> {
    i32::try_from(offset).map_err(|_| {
        format!(
            "Codegen error: aggregate offset {} in '{}' exceeds the stack-offset range",
            offset, func_name
        )
    })
}

/// Resolves a place rooted at an aggregate local to its stack slot, byte
/// offset and final layout by walking the projection chain.
///
/// `Field` covers struct fields by name and tuple positions by numeric name;
/// `ConstantIndex` covers constant tuple/array positions. A runtime `Index`
/// is rejected: address arithmetic alone would skip the bounds check, which
/// is the Stage 4D milestone. `Deref` is rejected: references have no
/// aggregate storage yet.
fn aggregate_place_address(
    storage: &HashMap<omni_mir::ir::Local, NativeStorage>,
    place: &omni_mir::ir::Place,
    context: &str,
) -> Result<(StackSlot, u64, TypeLayout), String> {
    let (slot, mut layout) = match storage.get(&place.local) {
        Some(NativeStorage::Aggregate { slot, layout }) => (*slot, layout.clone()),
        _ => {
            return Err(format!(
                "Codegen error: {} {} is not rooted at an aggregate local",
                context, place
            ));
        }
    };
    let mut offset = 0u64;
    for projection in &place.projections {
        let part = match projection {
            omni_mir::ir::Projection::Field(name) => layout
                .parts
                .iter()
                .find(|p| p.name.as_deref() == Some(name.as_str()))
                .or_else(|| name.parse::<usize>().ok().and_then(|i| layout.parts.get(i)))
                .ok_or_else(|| {
                    format!("Codegen error: {} {} has no field '{}'", context, place, name)
                })?,
            omni_mir::ir::Projection::ConstantIndex(index) => {
                layout.parts.get(*index).ok_or_else(|| {
                    format!(
                        "Codegen error: {} {} indexes out of bounds at position {}",
                        context, place, index
                    )
                })?
            }
            omni_mir::ir::Projection::Index(_) => {
                return Err(format!(
                    "Codegen error: {} {} uses a runtime index, which requires runtime bounds checks (Stage 4D); only constant indices are natively addressable",
                    context, place
                ));
            }
            omni_mir::ir::Projection::Deref => {
                return Err(format!(
                    "Codegen error: {} {} dereferences a reference, which has no aggregate storage",
                    context, place
                ));
            }
        };
        offset = offset.checked_add(part.offset).ok_or_else(|| {
            format!("Codegen error: {} {} overflows its aggregate offset", context, place)
        })?;
        layout = part.layout.clone();
    }
    Ok((slot, offset, layout))
}

/// Resolves the MIR type of one struct field after generic substitution.
///
/// This mirrors the lowering-time authority: the declaration supplies the
/// field's `TypeSpec`, the concrete type arguments supply the substitution.
fn struct_field_ty(
    emitter: &mut PlaceEmitter,
    struct_ty: omni_mir::Ty,
    field: &str,
) -> Result<omni_mir::Ty, String> {
    match emitter.tcx.get(struct_ty).clone() {
        omni_mir::TyKind::Struct(name, args) => {
            let def = emitter.struct_defs.get(&name).cloned().ok_or_else(|| {
                format!("Codegen error: unknown struct '{}' in '{}'", name, emitter.func_name)
            })?;
            let mut env = omni_mir::SubstEnv::new();
            for (param, arg) in def.type_params.iter().zip(args.iter()) {
                env.insert(param.clone(), *arg);
            }
            let spec =
                def.fields.iter().find(|f| f.name == field).map(|f| f.ty.clone()).ok_or_else(
                    || {
                        format!(
                            "Codegen error: struct '{}' in '{}' has no field '{}'",
                            name, emitter.func_name, field
                        )
                    },
                )?;
            Ok(emitter.layout_workspace.lower_type_spec(&spec, &env))
        }
        other => Err(format!(
            "Codegen error: field projection in '{}' requires a struct, found {:?}",
            emitter.func_name, other
        )),
    }
}

/// Resolves the type of a projected place by walking the chain from the root
/// local's type. Same authority as MIR lowering, so a projection cannot be
/// typed one way when read and another way when written.
fn projected_ty(
    emitter: &mut PlaceEmitter,
    base_ty: omni_mir::Ty,
    projections: &[omni_mir::ir::Projection],
) -> Result<omni_mir::Ty, String> {
    let mut current = base_ty;
    for projection in projections {
        current = match projection {
            omni_mir::ir::Projection::Field(name) => match emitter.tcx.get(current).clone() {
                omni_mir::TyKind::Struct(..) => struct_field_ty(emitter, current, name)?,
                omni_mir::TyKind::Tuple(elements) => {
                    let index: usize = name.parse().map_err(|_| {
                        format!(
                            "Codegen error: tuple field '{}' in '{}' is not a numeric index",
                            name, emitter.func_name
                        )
                    })?;
                    *elements.get(index).ok_or_else(|| {
                        format!(
                            "Codegen error: tuple field index {} in '{}' is out of bounds",
                            index, emitter.func_name
                        )
                    })?
                }
                other => {
                    return Err(format!(
                        "Codegen error: field projection in '{}' requires a struct or tuple, found {:?}",
                        emitter.func_name, other
                    ));
                }
            },
            omni_mir::ir::Projection::ConstantIndex(index) => {
                match emitter.tcx.get(current).clone() {
                    omni_mir::TyKind::Array(element, length) => {
                        if *index >= length {
                            return Err(format!(
                                "Codegen error: array index {} in '{}' is out of bounds for length {}",
                                index, emitter.func_name, length
                            ));
                        }
                        element
                    }
                    omni_mir::TyKind::Tuple(elements) => {
                        *elements.get(*index).ok_or_else(|| {
                            format!(
                                "Codegen error: tuple index {} in '{}' is out of bounds",
                                index, emitter.func_name
                            )
                        })?
                    }
                    other => {
                        return Err(format!(
                            "Codegen error: constant index projection in '{}' requires an array or tuple, found {:?}",
                            emitter.func_name, other
                        ));
                    }
                }
            }
            // The element type is statically known; the address walk still
            // rejects the access itself until Stage 4D bounds checks exist.
            omni_mir::ir::Projection::Index(_) => match emitter.tcx.get(current).clone() {
                omni_mir::TyKind::Array(element, _) => element,
                other => {
                    return Err(format!(
                        "Codegen error: index projection in '{}' requires an array, found {:?}",
                        emitter.func_name, other
                    ));
                }
            },
            omni_mir::ir::Projection::Deref => match emitter.tcx.get(current).clone() {
                omni_mir::TyKind::Reference { inner, .. } => inner,
                other => {
                    return Err(format!(
                        "Codegen error: deref projection in '{}' requires a reference, found {:?}",
                        emitter.func_name, other
                    ));
                }
            },
        };
    }
    Ok(current)
}

/// Lists the `(name, type)` parts of an aggregate type in layout order.
///
/// Tuple and array parts are positional; struct parts follow declaration
/// order with generic substitution applied, matching [`TargetLayout`].
fn aggregate_part_tys(
    emitter: &mut PlaceEmitter,
    aggregate_ty: omni_mir::Ty,
) -> Result<Vec<(Option<String>, omni_mir::Ty)>, String> {
    match emitter.tcx.get(aggregate_ty).clone() {
        omni_mir::TyKind::Tuple(elements) => {
            Ok(elements.into_iter().map(|element| (None, element)).collect())
        }
        omni_mir::TyKind::Array(element, length) => Ok(vec![(None, element); length]),
        omni_mir::TyKind::Struct(name, args) => {
            let def = emitter.struct_defs.get(&name).cloned().ok_or_else(|| {
                format!("Codegen error: unknown struct '{}' in '{}'", name, emitter.func_name)
            })?;
            let mut env = omni_mir::SubstEnv::new();
            for (param, arg) in def.type_params.iter().zip(args.iter()) {
                env.insert(param.clone(), *arg);
            }
            let mut lowered = Vec::with_capacity(def.fields.len());
            for f in &def.fields {
                lowered.push((
                    Some(f.name.clone()),
                    emitter.layout_workspace.lower_type_spec(&f.ty, &env),
                ));
            }
            Ok(lowered)
        }
        other => Err(format!(
            "Codegen error: aggregate part types in '{}' require a tuple, array or struct, found {:?}",
            emitter.func_name, other
        )),
    }
}

#[derive(Debug, Clone, Copy)]
enum NativeAddress {
    Stack { slot: StackSlot, offset: u64 },
    Pointer(cranelift_codegen::ir::Value),
}

/// Resolves an aggregate-rooted place to either a stack-slot offset or a
/// computed pointer. Static field/constant projections remain stack-relative;
/// the first dynamic array projection switches to pointer arithmetic and all
/// following projections extend that pointer. The MIR verifier guarantees that
/// every dynamic index reaches this code only after its corresponding
/// BoundsCheck.
fn resolve_place_address(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    place: &omni_mir::ir::Place,
    context: &str,
) -> Result<(NativeAddress, TypeLayout), String> {
    let (slot, root_layout) = match emitter.storage.get(&place.local) {
        Some(NativeStorage::Aggregate { slot, layout }) => (*slot, layout.clone()),
        _ => {
            return Err(format!(
                "Codegen error: {} {} is not rooted at an aggregate local",
                context, place
            ));
        }
    };
    let root_ty = emitter.body.local_decls[place.local].ty.ok_or_else(|| {
        format!(
            "Codegen error: local {:?} in '{}' has no type",
            place.local, emitter.func_name
        )
    })?;

    let mut current_ty = root_ty;
    let mut layout = root_layout;
    let mut static_offset = 0u64;
    let mut pointer = None;

    for (projection_index, projection) in place.projections.iter().enumerate() {
        let part = match projection {
            omni_mir::ir::Projection::Field(name) => layout
                .parts
                .iter()
                .find(|p| p.name.as_deref() == Some(name.as_str()))
                .or_else(|| name.parse::<usize>().ok().and_then(|i| layout.parts.get(i)))
                .ok_or_else(|| {
                    format!("Codegen error: {} {} has no field '{}'", context, place, name)
                })?
                .clone(),
            omni_mir::ir::Projection::ConstantIndex(index) => {
                layout.parts.get(*index).cloned().ok_or_else(|| {
                    format!(
                        "Codegen error: {} {} indexes out of bounds at position {}",
                        context, place, index
                    )
                })?
            }
            omni_mir::ir::Projection::Index(index_local) => {
                let array_len = match emitter.tcx.get(current_ty).clone() {
                    omni_mir::TyKind::Array(_, length) => length,
                    other => {
                        return Err(format!(
                            "Codegen error: {} dynamic index requires an array, found {:?}",
                            context, other
                        ));
                    }
                };
                if layout.parts.len() != array_len {
                    return Err(format!(
                        "Codegen error: {} array layout has {} parts for length {}",
                        context,
                        layout.parts.len(),
                        array_len
                    ));
                }
                let stride = match array_len {
                    0 => 0,
                    1 => layout.size,
                    _ => layout.parts[1]
                        .offset
                        .checked_sub(layout.parts[0].offset)
                        .ok_or_else(|| {
                            format!(
                                "Codegen error: {} array element offsets are not monotonic",
                                context
                            )
                        })?,
                };

                let index_value = match emitter.storage.get(index_local) {
                    Some(NativeStorage::Scalar(variable)) => builder.use_var(*variable),
                    _ => {
                        return Err(format!(
                            "Codegen error: {} dynamic index local {:?} has no scalar storage",
                            context, index_local
                        ));
                    }
                };
                let index_pointer = if emitter.pointer_type == cranelift_codegen::ir::types::I64 {
                    index_value
                } else if emitter.pointer_type == cranelift_codegen::ir::types::I32 {
                    builder.ins().ireduce(cranelift_codegen::ir::types::I32, index_value)
                } else {
                    return Err(format!(
                        "Codegen error: {} target pointer type {:?} is unsupported for array addressing",
                        context, emitter.pointer_type
                    ));
                };
                let stride_value = builder.ins().iconst(
                    emitter.pointer_type,
                    i64::try_from(stride).map_err(|_| {
                        format!(
                            "Codegen error: {} array stride {} exceeds pointer immediate range",
                            context, stride
                        )
                    })?,
                );
                let byte_offset = builder.ins().imul(index_pointer, stride_value);

                let base_pointer = match pointer {
                    Some(ptr) => ptr,
                    None => builder.ins().stack_addr(
                        emitter.pointer_type,
                        slot,
                        stack_offset(static_offset, emitter.func_name)?,
                    ),
                };
                let ptr = builder.ins().iadd(base_pointer, byte_offset);
                pointer = Some(ptr);
                static_offset = 0;
                let prefix_end = projection_index + 1;
                current_ty = projected_ty(emitter, root_ty, &place.projections[..prefix_end])?;
                if projection_index + 1 < place.projections.len() {
                    layout = emitter.layout.aggregate_layout(
                        emitter.layout_workspace,
                        current_ty,
                    ).map_err(|e| {
                        format!(
                            "Codegen error: {} cannot lay out dynamic array element: {}",
                            context, e
                        )
                    })?;
                }
                continue;
            }
            omni_mir::ir::Projection::Deref => {
                return Err(format!(
                    "Codegen error: {} {} dereferences a reference, which has no aggregate storage",
                    context, place
                ));
            }
        };

        static_offset = static_offset.checked_add(part.offset).ok_or_else(|| {
            format!("Codegen error: {} {} overflows its aggregate offset", context, place)
        })?;
        if let Some(ptr) = pointer {
            let delta = builder.ins().iconst(
                emitter.pointer_type,
                i64::try_from(part.offset).map_err(|_| {
                    format!(
                        "Codegen error: {} projection offset {} exceeds pointer immediate range",
                        context, part.offset
                    )
                })?,
            );
            pointer = Some(builder.ins().iadd(ptr, delta));
        }
        let prefix_end = projection_index + 1;
        current_ty = projected_ty(emitter, root_ty, &place.projections[..prefix_end])?;
        layout = part.layout;
    }

    let final_layout = layout;
    let address = match pointer {
        Some(ptr) => NativeAddress::Pointer(ptr),
        None => NativeAddress::Stack { slot, offset: static_offset },
    };
    Ok((address, final_layout))
}

/// Loads a scalar value out of a projected place: address from the layout
/// walk, Cranelift type from the projected MIR type.
/// Loads a scalar value out of a projected place: address from the layout
/// walk, Cranelift type from the projected MIR type.
fn emit_projected_place_load(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    place: &omni_mir::ir::Place,
) -> Result<cranelift_codegen::ir::Value, String> {
    let context = format!("read of {}", place);
    let root_ty = emitter.body.local_decls[place.local].ty.ok_or_else(|| {
        format!("Codegen error: local {:?} in '{}' has no type", place.local, emitter.func_name)
    })?;
    let ty = projected_ty(emitter, root_ty, &place.projections)?;
    let clif_ty = emitter.layout.scalar_clif_type(emitter.tcx, ty).ok_or_else(|| {
        format!(
            "Codegen error: {} in '{}' is aggregate-typed and has no scalar SSA form; copy it into an aggregate destination instead",
            context, emitter.func_name
        )
    })?;
    let (address, _) = resolve_place_address(builder, emitter, place, &context)?;
    Ok(match address {
        NativeAddress::Stack { slot, offset } => {
            builder.ins().stack_load(clif_ty, slot, stack_offset(offset, emitter.func_name)?)
        }
        NativeAddress::Pointer(ptr) => {
            builder.ins().load(clif_ty, cranelift_codegen::ir::MemFlags::new(), ptr, 0)
        }
    })
}

/// Resolves an operand that must denote a whole or projected aggregate to its
/// storage triple plus MIR type and layout. Scalar-typed projections are
/// refused: they belong in scalar destinations, not aggregate copies.
fn resolve_aggregate_operand(
    emitter: &mut PlaceEmitter,
    operand: &omni_mir::ir::Operand,
    context: &str,
) -> Result<AggregateSite, String> {
    let place = match operand {
        omni_mir::ir::Operand::Copy(place) | omni_mir::ir::Operand::Move(place) => place.clone(),
        omni_mir::ir::Operand::Constant(_) => {
            return Err(format!(
                "Codegen error: {} requires an aggregate place, found a constant",
                context
            ));
        }
    };
    if place.is_local() {
        match emitter.storage.get(&place.local).cloned() {
            Some(NativeStorage::Aggregate { slot, layout }) => {
                let ty = emitter.body.local_decls[place.local].ty.ok_or_else(|| {
                    format!(
                        "Codegen error: aggregate local {:?} in '{}' has no type",
                        place.local, emitter.func_name
                    )
                })?;
                Ok(AggregateSite { slot, offset: 0, ty, layout })
            }
            _ => Err(format!(
                "Codegen error: {} requires an aggregate local, found {:?} in '{}'",
                context, place.local, emitter.func_name
            )),
        }
    } else {
        let (slot, offset, layout) = aggregate_place_address(emitter.storage, &place, context)?;
        let root_ty = emitter.body.local_decls[place.local].ty.ok_or_else(|| {
            format!("Codegen error: local {:?} in '{}' has no type", place.local, emitter.func_name)
        })?;
        let ty = projected_ty(emitter, root_ty, &place.projections)?;
        if emitter.layout.scalar_clif_type(emitter.tcx, ty).is_some() {
            return Err(format!(
                "Codegen error: {} reads scalar-typed {}; use a scalar destination instead",
                context, place
            ));
        }
        Ok(AggregateSite { slot, offset, ty, layout })
    }
}

/// Resolves an `Rvalue::Field`/`Rvalue::Index` that yields a whole aggregate
/// (for example a nested-struct read) to its storage triple.
fn resolve_projection_source(
    emitter: &mut PlaceEmitter,
    base: &omni_mir::ir::Operand,
    projection: omni_mir::ir::Projection,
    ty: omni_mir::Ty,
    context: &str,
) -> Result<AggregateSite, String> {
    let base_place = match base {
        omni_mir::ir::Operand::Copy(place) | omni_mir::ir::Operand::Move(place) => place.clone(),
        omni_mir::ir::Operand::Constant(_) => {
            return Err(format!(
                "Codegen error: {} base must be a place, found a constant",
                context
            ));
        }
    };
    let mut full = base_place;
    full.projections.push(projection);
    let root_ty = emitter.body.local_decls[full.local].ty.ok_or_else(|| {
        format!("Codegen error: local {:?} in '{}' has no type", full.local, emitter.func_name)
    })?;
    let recomputed = projected_ty(emitter, root_ty, &full.projections)?;
    if recomputed != ty {
        return Err(format!(
            "Codegen error: {} in '{}' declares type {:?} but the projection resolves to {:?}",
            context,
            emitter.func_name,
            emitter.tcx.get(ty),
            emitter.tcx.get(recomputed)
        ));
    }
    let (slot, offset, layout) = aggregate_place_address(emitter.storage, &full, context)?;
    Ok(AggregateSite { slot, offset, ty, layout })
}

/// Copies an aggregate from an incoming pointer into callee-owned stack
/// storage. TargetLayout remains the only authority for member offsets.
fn copy_aggregate_from_pointer(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    dst: &AggregateSite,
    src_ptr: cranelift_codegen::ir::Value,
    context: &str,
) -> Result<(), String> {
    fn visit(
        builder: &mut FunctionBuilder,
        emitter: &mut PlaceEmitter,
        dst: &AggregateSite,
        src_ptr: cranelift_codegen::ir::Value,
        src_offset: u64,
        context: &str,
    ) -> Result<(), String> {
        if let Some(clif_ty) = emitter.layout.scalar_clif_type(emitter.tcx, dst.ty) {
            if dst.layout.size != u64::from(clif_ty.bytes()) {
                return Err(format!(
                    "Codegen error: {} scalar layout size {} disagrees with {:?}",
                    context, dst.layout.size, dst.ty
                ));
            }
            let value = builder.ins().load(
                clif_ty,
                cranelift_codegen::ir::MemFlags::new(),
                src_ptr,
                stack_offset(src_offset, emitter.func_name)?,
            );
            builder.ins().stack_store(
                value,
                dst.slot,
                stack_offset(dst.offset, emitter.func_name)?,
            );
            return Ok(());
        }

        let part_tys = aggregate_part_tys(emitter, dst.ty)?;
        if part_tys.len() != dst.layout.parts.len() {
            return Err(format!(
                "Codegen error: {} aggregate type and layout disagree on part count",
                context
            ));
        }
        for (index, ((name, part_ty), part)) in
            part_tys.into_iter().zip(dst.layout.parts.iter()).enumerate()
        {
            if part.name != name {
                return Err(format!(
                    "Codegen error: {} part {} names {:?} but the layout names {:?}",
                    context, index, name, part.name
                ));
            }
            let next_src_offset = src_offset.checked_add(part.offset).ok_or_else(|| {
                format!("Codegen error: {} overflows its source pointer offset", context)
            })?;
            visit(
                builder,
                emitter,
                &dst.child(part_ty, part, context)?,
                src_ptr,
                next_src_offset,
                context,
            )?;
        }
        Ok(())
    }

    visit(builder, emitter, dst, src_ptr, 0, context)
}

/// Copies a callee-owned aggregate into caller-provided result storage.
fn copy_aggregate_to_pointer(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    src: &AggregateSite,
    dst_ptr: cranelift_codegen::ir::Value,
    context: &str,
) -> Result<(), String> {
    fn visit(
        builder: &mut FunctionBuilder,
        emitter: &mut PlaceEmitter,
        src: &AggregateSite,
        dst_ptr: cranelift_codegen::ir::Value,
        dst_offset: u64,
        context: &str,
    ) -> Result<(), String> {
        if let Some(clif_ty) = emitter.layout.scalar_clif_type(emitter.tcx, src.ty) {
            if src.layout.size != u64::from(clif_ty.bytes()) {
                return Err(format!(
                    "Codegen error: {} scalar layout size {} disagrees with {:?}",
                    context, src.layout.size, src.ty
                ));
            }
            let value = builder.ins().stack_load(
                clif_ty,
                src.slot,
                stack_offset(src.offset, emitter.func_name)?,
            );
            builder.ins().store(
                cranelift_codegen::ir::MemFlags::new(),
                value,
                dst_ptr,
                stack_offset(dst_offset, emitter.func_name)?,
            );
            return Ok(());
        }

        let part_tys = aggregate_part_tys(emitter, src.ty)?;
        if part_tys.len() != src.layout.parts.len() {
            return Err(format!(
                "Codegen error: {} aggregate type and layout disagree on part count",
                context
            ));
        }
        for (index, ((name, part_ty), part)) in
            part_tys.into_iter().zip(src.layout.parts.iter()).enumerate()
        {
            if part.name != name {
                return Err(format!(
                    "Codegen error: {} part {} names {:?} but the layout names {:?}",
                    context, index, name, part.name
                ));
            }
            let next_dst_offset = dst_offset.checked_add(part.offset).ok_or_else(|| {
                format!("Codegen error: {} overflows its result pointer offset", context)
            })?;
            visit(
                builder,
                emitter,
                &src.child(part_ty, part, context)?,
                dst_ptr,
                next_dst_offset,
                context,
            )?;
        }
        Ok(())
    }

    visit(builder, emitter, src, dst_ptr, 0, context)
}

/// Copies one aggregate value onto another, recursing through nested
/// aggregates. Both sides share one concrete type, so their layouts must be
/// identical; scalar leaves move through ordinary SSA loads and stores.
fn copy_aggregate(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    dst: &AggregateSite,
    src: &AggregateSite,
    context: &str,
) -> Result<(), String> {
    if dst.ty != src.ty {
        return Err(format!(
            "Codegen error: {} copies {:?} into {:?}, which have different types",
            context,
            emitter.tcx.get(src.ty),
            emitter.tcx.get(dst.ty)
        ));
    }
    if dst.layout != src.layout {
        return Err(format!("Codegen error: {} source and destination layouts disagree", context));
    }
    match emitter.tcx.get(dst.ty).clone() {
        omni_mir::TyKind::Int
        | omni_mir::TyKind::Bool
        | omni_mir::TyKind::Byte
        | omni_mir::TyKind::Char
        | omni_mir::TyKind::Float => {
            let clif_ty =
                emitter.layout.scalar_clif_type(emitter.tcx, dst.ty).ok_or_else(|| {
                    format!("Codegen error: {} has no scalar representation", context)
                })?;
            let value = builder.ins().stack_load(
                clif_ty,
                src.slot,
                stack_offset(src.offset, emitter.func_name)?,
            );
            builder.ins().stack_store(
                value,
                dst.slot,
                stack_offset(dst.offset, emitter.func_name)?,
            );
            Ok(())
        }
        omni_mir::TyKind::Tuple(_) | omni_mir::TyKind::Array(..) | omni_mir::TyKind::Struct(..) => {
            let part_tys = aggregate_part_tys(emitter, dst.ty)?;
            if part_tys.len() != dst.layout.parts.len() {
                return Err(format!(
                    "Codegen error: {} type and layout disagree on part count",
                    context
                ));
            }
            for (index, ((name, part_ty), part)) in
                part_tys.into_iter().zip(dst.layout.parts.iter()).enumerate()
            {
                if part.name != name {
                    return Err(format!(
                        "Codegen error: {} part {} names {:?} but the layout names {:?}",
                        context, index, name, part.name
                    ));
                }
                copy_aggregate(
                    builder,
                    emitter,
                    &dst.child(part_ty, part, context)?,
                    &src.child(part_ty, part, context)?,
                    context,
                )?;
            }
            Ok(())
        }
        other => {
            Err(format!("Codegen error: {} cannot copy unrepresentable type {:?}", context, other))
        }
    }
}

/// Stores one constructor operand into a single aggregate part.
///
/// Scalar parts lower the operand to an SSA value, gated against the part's
/// MIR type so an `I64` can never silently land in an `F64` field. Nested
/// aggregate parts recurse through [`copy_aggregate`]. Zero-sized parts
/// (empty aggregates) occupy nothing and store nothing.
fn emit_part_store(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    site: &AggregateSite,
    operand: &omni_mir::ir::Operand,
    context: &str,
) -> Result<(), String> {
    if site.layout.size == 0 && site.layout.parts.is_empty() {
        return Ok(());
    }
    if let Some(clif_ty) = emitter.layout.scalar_clif_type(emitter.tcx, site.ty) {
        if site.layout.size != u64::from(clif_ty.bytes()) {
            return Err(format!(
                "Codegen error: {} part layout size {} disagrees with its {:?} type",
                context,
                site.layout.size,
                emitter.tcx.get(site.ty)
            ));
        }
        let value = lower_operand_to_cl(builder, emitter, operand)?;
        if builder.func.dfg.value_type(value) != clif_ty {
            return Err(format!(
                "Codegen error: {} part expects {:?} but the operand lowers to {:?}",
                context,
                clif_ty,
                builder.func.dfg.value_type(value)
            ));
        }
        builder.ins().stack_store(value, site.slot, stack_offset(site.offset, emitter.func_name)?);
        Ok(())
    } else {
        match emitter.tcx.get(site.ty) {
            omni_mir::TyKind::Tuple(_)
            | omni_mir::TyKind::Array(..)
            | omni_mir::TyKind::Struct(..) => {}
            other => {
                return Err(format!(
                    "Codegen error: {} cannot store unrepresentable part type {:?}",
                    context, other
                ));
            }
        }
        let src = resolve_aggregate_operand(emitter, operand, context)?;
        copy_aggregate(builder, emitter, site, &src, context)
    }
}

/// Reads the constant position out of an index operand.
///
/// Only literal subscripts are natively addressable. A computed subscript
/// needs the Stage 4D bounds-check path, so it is rejected with that reason
/// rather than being emitted as an unchecked access.
fn constant_index_position(
    index: &omni_mir::ir::Operand,
    func_name: &str,
) -> Result<usize, String> {
    match index {
        omni_mir::ir::Operand::Constant(omni_mir::ir::Constant::Lit(omni_mir::ast::Lit::Int(n)))
            if *n >= 0 =>
        {
            Ok(*n as usize)
        }
        _ => Err(format!(
            "Codegen error: dynamic array indexing in '{}' requires runtime bounds checks (Stage 4D); only constant indices are natively addressable",
            func_name
        )),
    }
}

/// Loads a scalar out of an `Rvalue::Field`/`Rvalue::Index`.
///
/// The MIR-declared result type is cross-checked against the recomputed
/// projection type, and must be scalar: an aggregate-typed projection read
/// belongs to [`emit_aggregate_store`], not to SSA value lowering.
fn emit_projection_load(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    base: &omni_mir::ir::Operand,
    projection: omni_mir::ir::Projection,
    ty: omni_mir::Ty,
    context: &str,
) -> Result<cranelift_codegen::ir::Value, String> {
    let base_place = match base {
        omni_mir::ir::Operand::Copy(place) | omni_mir::ir::Operand::Move(place) => place.clone(),
        omni_mir::ir::Operand::Constant(_) => {
            return Err(format!(
                "Codegen error: {} base must be a place, found a constant",
                context
            ));
        }
    };
    let mut full = base_place;
    full.projections.push(projection);
    let root_ty = emitter.body.local_decls[full.local].ty.ok_or_else(|| {
        format!("Codegen error: local {:?} in '{}' has no type", full.local, emitter.func_name)
    })?;
    let projected = projected_ty(emitter, root_ty, &full.projections)?;
    if projected != ty {
        return Err(format!(
            "Codegen error: {} in '{}' declares type {:?} but projection resolves to {:?}",
            context,
            emitter.func_name,
            emitter.tcx.get(ty),
            emitter.tcx.get(projected)
        ));
    }
    let clif_ty = emitter.layout.scalar_clif_type(emitter.tcx, ty).ok_or_else(|| {
        format!(
            "Codegen error: {} in '{}' is aggregate-typed and has no scalar SSA form; store it into an aggregate destination instead",
            context, emitter.func_name
        )
    })?;
    let (address, _) = resolve_place_address(builder, emitter, &full, context)?;
    Ok(match address {
        NativeAddress::Stack { slot, offset } => {
            builder.ins().stack_load(clif_ty, slot, stack_offset(offset, emitter.func_name)?)
        }
        NativeAddress::Pointer(ptr) => {
            builder.ins().load(clif_ty, cranelift_codegen::ir::MemFlags::new(), ptr, 0)
        }
    })
}

fn emit_index_projection_load(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    base: &omni_mir::ir::Operand,
    projection_operand: &omni_mir::ir::Operand,
    ty: omni_mir::Ty,
    context: &str,
) -> Result<cranelift_codegen::ir::Value, String> {
    let base_place = match base {
        omni_mir::ir::Operand::Copy(place) | omni_mir::ir::Operand::Move(place) => place.clone(),
        omni_mir::ir::Operand::Constant(_) => {
            return Err(format!(
                "Codegen error: {} base must be a place, found a constant",
                context
            ));
        }
    };
    let projection = match projection_operand {
        omni_mir::ir::Operand::Constant(omni_mir::ir::Constant::Lit(
            omni_mir::ast::Lit::Int(n),
        )) if *n >= 0 => omni_mir::ir::Projection::ConstantIndex(*n as usize),
        omni_mir::ir::Operand::Copy(place) | omni_mir::ir::Operand::Move(place) => {
            omni_mir::ir::Projection::Index(place.local)
        }
        _ => {
            return Err(format!(
                "Codegen error: {} has an unsupported dynamic index operand",
                context
            ));
        }
    };
    let mut full = base_place;
    full.projections.push(projection);
    let root_ty = emitter.body.local_decls[full.local].ty.ok_or_else(|| {
        format!("Codegen error: local {:?} in '{}' has no type", full.local, emitter.func_name)
    })?;
    let projected = projected_ty(emitter, root_ty, &full.projections)?;
    if projected != ty {
        return Err(format!(
            "Codegen error: {} in '{}' declares type {:?} but projection resolves to {:?}",
            context,
            emitter.func_name,
            emitter.tcx.get(ty),
            emitter.tcx.get(projected)
        ));
    }
    let clif_ty = emitter.layout.scalar_clif_type(emitter.tcx, ty).ok_or_else(|| {
        format!(
            "Codegen error: {} in '{}' is aggregate-typed and has no scalar SSA form; store it into an aggregate destination instead",
            context, emitter.func_name
        )
    })?;
    let (address, _) = resolve_place_address(builder, emitter, &full, context)?;
    Ok(match address {
        NativeAddress::Stack { slot, offset } => {
            builder.ins().stack_load(clif_ty, slot, stack_offset(offset, emitter.func_name)?)
        }
        NativeAddress::Pointer(ptr) => {
            builder.ins().load(clif_ty, cranelift_codegen::ir::MemFlags::new(), ptr, 0)
        }
    })
}

/// Resolves an `Rvalue::Field`/`Rvalue::Index` to its storage address,
/// checking the declared result type against the recomputed projection type.
fn resolve_projection_address(
    emitter: &mut PlaceEmitter,
    base: &omni_mir::ir::Operand,
    projection: omni_mir::ir::Projection,
    ty: omni_mir::Ty,
    context: &str,
) -> Result<(StackSlot, u64, TypeLayout), String> {
    let site = resolve_projection_source(emitter, base, projection, ty, context)?;
    Ok((site.slot, site.offset, site.layout))
}

/// Stores an rvalue into a whole aggregate destination.
///
/// Constructors write each part through [`emit_part_store`], which recurses
/// into nested aggregates. Whole-aggregate reads (`Use` of an aggregate
/// place, or an aggregate-typed `Field`/`Index` such as a nested-struct
/// read) copy through [`copy_aggregate`]. Scalar rvalues are rejected: they
/// belong in scalar destinations.
fn emit_aggregate_store(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    site: &AggregateSite,
    rval: &omni_mir::ir::Rvalue,
    context: &str,
) -> Result<(), String> {
    match rval {
        omni_mir::ir::Rvalue::Struct { name, fields, ty } => {
            if *ty != site.ty {
                return Err(format!(
                    "Codegen error: {} declares {:?} but the destination is {:?}",
                    context,
                    emitter.tcx.get(*ty),
                    emitter.tcx.get(site.ty)
                ));
            }
            match emitter.tcx.get(site.ty).clone() {
                omni_mir::TyKind::Struct(def_name, _) if def_name == *name => {}
                other => {
                    return Err(format!(
                        "Codegen error: {} constructs struct '{}' into {:?}",
                        context, name, other
                    ));
                }
            }
            let part_tys = aggregate_part_tys(emitter, site.ty)?;
            if fields.len() != site.layout.parts.len() || fields.len() != part_tys.len() {
                return Err(format!(
                    "Codegen error: {} constructs struct '{}' with {} fields but the layout has {}",
                    context,
                    name,
                    fields.len(),
                    site.layout.parts.len()
                ));
            }
            for (field_name, operand) in fields {
                let index = part_tys
                    .iter()
                    .position(|(part_name, _)| part_name.as_deref() == Some(field_name.as_str()))
                    .ok_or_else(|| {
                        format!(
                            "Codegen error: {} constructs unknown field '{}.{}'",
                            context, name, field_name
                        )
                    })?;
                let part = &site.layout.parts[index];
                emit_part_store(
                    builder,
                    emitter,
                    &site.child(part_tys[index].1, part, context)?,
                    operand,
                    &format!("{} field '{}.{}'", context, name, field_name),
                )?;
            }
            Ok(())
        }
        omni_mir::ir::Rvalue::Aggregate { kind, operands, ty } => {
            if *ty != site.ty {
                return Err(format!(
                    "Codegen error: {} declares {:?} but the destination is {:?}",
                    context,
                    emitter.tcx.get(*ty),
                    emitter.tcx.get(site.ty)
                ));
            }
            let part_tys = match kind {
                omni_mir::ir::AggregateKind::Tuple => match emitter.tcx.get(site.ty).clone() {
                    omni_mir::TyKind::Tuple(elements) => elements
                        .into_iter()
                        .map(|element| (None::<String>, element))
                        .collect::<Vec<_>>(),
                    other => {
                        return Err(format!(
                            "Codegen error: {} builds a tuple into {:?}",
                            context, other
                        ));
                    }
                },
                omni_mir::ir::AggregateKind::Array => match emitter.tcx.get(site.ty).clone() {
                    omni_mir::TyKind::Array(element, length) => {
                        if operands.len() != length {
                            return Err(format!(
                                "Codegen error: {} builds an array of {} elements into length {}",
                                context,
                                operands.len(),
                                length
                            ));
                        }
                        vec![(None, element); length]
                    }
                    other => {
                        return Err(format!(
                            "Codegen error: {} builds an array into {:?}",
                            context, other
                        ));
                    }
                },
            };
            if operands.len() != site.layout.parts.len() || operands.len() != part_tys.len() {
                return Err(format!(
                    "Codegen error: {} builds {} operands but the layout has {} parts",
                    context,
                    operands.len(),
                    site.layout.parts.len()
                ));
            }
            for (index, operand) in operands.iter().enumerate() {
                let part = &site.layout.parts[index];
                emit_part_store(
                    builder,
                    emitter,
                    &site.child(part_tys[index].1, part, context)?,
                    operand,
                    &format!("{} part {}", context, index),
                )?;
            }
            Ok(())
        }
        omni_mir::ir::Rvalue::Use(operand) => {
            let src = resolve_aggregate_operand(emitter, operand, context)?;
            copy_aggregate(builder, emitter, site, &src, context)
        }
        omni_mir::ir::Rvalue::Field { base, field, ty } => {
            if *ty != site.ty {
                return Err(format!(
                    "Codegen error: {} declares {:?} but the destination is {:?}",
                    context,
                    emitter.tcx.get(*ty),
                    emitter.tcx.get(site.ty)
                ));
            }
            let src = resolve_projection_source(
                emitter,
                base,
                omni_mir::ir::Projection::Field(field.clone()),
                *ty,
                context,
            )?;
            copy_aggregate(builder, emitter, site, &src, context)
        }
        omni_mir::ir::Rvalue::Index { base, index, ty } => {
            if *ty != site.ty {
                return Err(format!(
                    "Codegen error: {} declares {:?} but the destination is {:?}",
                    context,
                    emitter.tcx.get(*ty),
                    emitter.tcx.get(site.ty)
                ));
            }
            let position = constant_index_position(index, emitter.func_name)?;
            let src = resolve_projection_source(
                emitter,
                base,
                omni_mir::ir::Projection::ConstantIndex(position),
                *ty,
                context,
            )?;
            copy_aggregate(builder, emitter, site, &src, context)
        }
        other => Err(format!(
            "Codegen error: {} cannot be initialized from scalar rvalue {:?}; aggregates initialize from constructors and aggregate copies",
            context, other
        )),
    }
}

fn lower_operand_to_cl(
    builder: &mut FunctionBuilder,
    emitter: &mut PlaceEmitter,
    op: &omni_mir::ir::Operand,
) -> Result<cranelift_codegen::ir::Value, String> {
    match op {
        omni_mir::ir::Operand::Copy(place) | omni_mir::ir::Operand::Move(place) => {
            if place.is_local() {
                match emitter.storage.get(&place.local).cloned() {
                    Some(NativeStorage::Scalar(variable)) => Ok(builder.use_var(variable)),
                    Some(NativeStorage::Aggregate { .. }) => Err(format!(
                        "Codegen error: aggregate local {:?} in '{}' cannot be used as a scalar SSA value; copy it into an aggregate destination instead",
                        place.local, emitter.func_name
                    )),
                    None => Err(format!(
                        "Codegen error: unbound local {:?} in '{}'",
                        place.local, emitter.func_name
                    )),
                }
            } else {
                emit_projected_place_load(builder, emitter, place)
            }
        }
        omni_mir::ir::Operand::Constant(c) => match c {
            omni_mir::ir::Constant::Lit(lit) => match lit {
                omni_mir::ast::Lit::Int(n) => Ok(builder.ins().iconst(types::I64, *n)),
                omni_mir::ast::Lit::Bool(b) => {
                    Ok(builder.ins().iconst(types::I64, if *b { 1 } else { 0 }))
                }
                omni_mir::ast::Lit::Byte(b) => Ok(builder.ins().iconst(types::I64, *b as i64)),
                omni_mir::ast::Lit::Char(c) => Ok(builder.ins().iconst(types::I64, *c as i64)),
                omni_mir::ast::Lit::Float(bits) => {
                    Ok(builder.ins().f64const(f64::from_bits(*bits)))
                }
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
    emitter: &mut PlaceEmitter,
    rval: &omni_mir::ir::Rvalue,
) -> Result<cranelift_codegen::ir::Value, String> {
    match rval {
        omni_mir::ir::Rvalue::Use(op) => lower_operand_to_cl(builder, emitter, op),
        omni_mir::ir::Rvalue::BinaryOp(op, lhs, rhs) => {
            let l = lower_operand_to_cl(builder, emitter, lhs)?;
            let r = lower_operand_to_cl(builder, emitter, rhs)?;
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
        omni_mir::ir::Rvalue::Aggregate { kind, .. } => Err(format!(
            "Codegen error: aggregate {:?} cannot initialize a scalar destination; store it into an aggregate local instead",
            kind
        )),
        omni_mir::ir::Rvalue::Field { base, field, ty } => emit_projection_load(
            builder,
            emitter,
            base,
            omni_mir::ir::Projection::Field(field.clone()),
            *ty,
            "field projection",
        ),
        omni_mir::ir::Rvalue::Struct { name, .. } => Err(format!(
            "Codegen error: struct constructor '{}' cannot initialize a scalar destination; store it into an aggregate local instead",
            name
        )),
        omni_mir::ir::Rvalue::EnumVariant { enum_name, variant, .. } => Err(format!(
            "Codegen error: enum constructor '{}::{}' requires target tagged-layout metadata",
            enum_name, variant
        )),
        omni_mir::ir::Rvalue::Range { .. } => Err(
            "Codegen error: range value representation requires target layout metadata".into(),
        ),
        omni_mir::ir::Rvalue::Index { base, index, ty } => emit_index_projection_load(
            builder,
            emitter,
            base,
            index,
            *ty,
            "index projection",
        )

        omni_mir::ir::Rvalue::Cast { operand, from, to } => {
            let value = lower_operand_to_cl(builder, emitter, operand)?;
            let from_float = matches!(emitter.tcx.get(*from), omni_mir::TyKind::Float);
            let to_float = matches!(emitter.tcx.get(*to), omni_mir::TyKind::Float);
            match (from_float, to_float) {
                (false, false) => Ok(value),
                (false, true) => Ok(builder.ins().fcvt_from_sint(types::F64, value)),
                (true, false) => Ok(builder.ins().fcvt_to_sint(types::I64, value)),
                (true, true) => Ok(value),
            }
        }
        omni_mir::ir::Rvalue::UnaryOp(op, operand) => {
            let val = lower_operand_to_cl(builder, emitter, operand)?;
            match op {
                omni_mir::ir::UnOp::Neg => {
                    if builder.func.dfg.value_type(val) == types::F64 {
                        Ok(builder.ins().fneg(val))
                    } else {
                        Ok(builder.ins().ineg(val))
                    }
                }
                omni_mir::ir::UnOp::Not => {
                    let one = builder.ins().iconst(types::I64, 1);
                    Ok(builder.ins().bxor(val, one))
                }
                omni_mir::ir::UnOp::BitNot => {
                    let all_ones = builder.ins().iconst(types::I64, -1);
                    Ok(builder.ins().bxor(val, all_ones))
                }
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

pub mod layout;

pub use layout::{LayoutError, PartLayout, TargetFacts, TargetLayout, TypeLayout};

/// The rules deciding how Omni aggregates use target facts.
///
/// This is the *policy* half, and every rule here is an Omni decision that the
/// target does **not** make for us. They are written out so the resulting ABI is
/// reviewable rather than emergent:
///
/// 1. **Scalar size comes from the selected Cranelift type**, and scalar
///    alignment is that same size. Not from the Omni type's name and not from a
///    hard-coded table. This matters because codegen currently maps `Int`,
///    `Bool`, `Byte` and `Char` all to `I64`; a `Bool` occupying eight bytes is a
///    consequence of that mapping, not a target fact, and the layout layer must
///    not pretend otherwise.
/// 2. **Aggregate alignment is the strictest part alignment.** An aggregate is
///    at least as aligned as its most aligned part.
/// 3. **Each part starts at the next offset satisfying its own alignment**,
///    producing interior padding. This is the ordinary cost of honouring field
///    alignment and keeps every field addressable at its natural alignment.
/// 4. **Parts appear in declaration order.** Declaration order is the only
///    ordering the repository currently specifies; no reordering for padding
///    density is performed, because that would be an ABI choice the
///    specification has not made.
/// 5. **Aggregate size is rounded up to its own alignment**, so an array of the
///    aggregate repeats on a correct stride.
/// 6. **An empty aggregate is zero-sized and aligned to 1.** The grammar admits
///    zero-field structs, and stating that they occupy nothing is better than
///    fabricating a size for them.
///
/// These rules are policy, and changing any of them changes Omni's ABI. They
/// are recorded here as a single reviewable list precisely so that such a change
/// is a deliberate decision rather than a side effect.
pub const LAYOUT_POLICY_RULES: [&str; 6] = [
    "scalar size from the selected Cranelift type; scalar alignment equals its size",
    "aggregate alignment is the strictest part alignment",
    "each part starts at the next offset satisfying its own alignment",
    "parts appear in declaration order; no padding-density reordering",
    "aggregate size is rounded up to its own alignment",
    "an empty aggregate is zero-sized and aligned to 1",
];

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
            name: "omni_main".to_string(),
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
                omni_mir::ir::Place::local(ret),
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
                omni_mir::ir::Place::local(ret),
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
                name: "omni_main".to_string(),
                params: vec![],
                return_place: ret,
                return_type: ast::TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals },
            }],
            struct_defs: std::collections::HashMap::new(),
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
        let shim_path = dir.join(format!("{stem}_shim.rs"));
        let exe_path = dir.join(format!("{stem}.exe"));
        fs::write(&object_path, object).expect("object write");
        // Linked through `rustc` rather than `cc`: the toolchain running
        // these tests always ships its own linker driver, so end-to-end
        // execution does not depend on a separately installed C toolchain. A
        // two-line shim supplies the entry point and calls `omni_main`.
        fs::write(
            &shim_path,
            "unsafe extern \"C\" { fn omni_main() -> i64; }\nfn main() { std::process::exit(unsafe { omni_main() } as i32); }\n",
        )
        .expect("shim write");

        let link = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&shim_path)
            .arg("-o")
            .arg(&exe_path)
            .arg("-C")
            .arg(format!("link-arg={}", object_path.display()))
            .output()
            .expect("rustc must be available");
        assert!(
            link.status.success(),
            "rustc link failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&link.stdout),
            String::from_utf8_lossy(&link.stderr)
        );

        let run = Command::new(&exe_path).status().expect("executable must run");
        assert_eq!(run.code(), Some(41));

        fs::remove_file(object_path).ok();
        fs::remove_file(shim_path).ok();
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
                omni_mir::ir::Place::local(ret),
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
            struct_defs: std::collections::HashMap::new(),
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
            struct_defs: std::collections::HashMap::new(),
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
