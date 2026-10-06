//! CFG-aware affine ownership and initialization verification for MIR.
use omni_mir::ir::{
    BasicBlock, Constant, Local, MirFunction, MirProgram, Operand, Place, Projection, Rvalue,
    Statement, Terminator,
};
use omni_own::{
    AccessKind, OwnershipError, OwnershipState, Place as OwnershipPlace,
    Projection as OwnershipProjection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnershipVerificationError {
    pub(crate) function: String,
    pub(crate) block: BasicBlock,
    pub(crate) context: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlowState {
    ownership: OwnershipState,
}

impl FlowState {
    fn new() -> Self {
        Self { ownership: OwnershipState::new() }
    }

    fn join(states: &[Self]) -> Self {
        let inputs = states.iter().map(|state| &state.ownership).collect::<Vec<_>>();
        Self { ownership: OwnershipState::join_all(&inputs) }
    }
}

pub(crate) fn verify_program(program: &MirProgram) -> Result<(), OwnershipVerificationError> {
    for function in &program.functions {
        verify_function(function)?;
    }
    Ok(())
}

fn verify_function(function: &MirFunction) -> Result<(), OwnershipVerificationError> {
    if function.body.blocks.is_empty() {
        return Ok(());
    }

    let block_count = function.body.blocks.len();
    let mut entry_states = vec![None::<FlowState>; block_count];
    let mut edge_states = vec![Vec::<(BasicBlock, FlowState)>::new(); block_count];

    let mut initial = FlowState::new();
    for &param in &function.params {
        initial.ownership.declare_initialized(local_name(function, param));
    }

    const MAX_ITERATIONS: usize = 4096;
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            return Err(OwnershipVerificationError {
                function: function.name.clone(),
                block: BasicBlock::from_usize(0),
                context: "ownership dataflow".into(),
                message:
                    "ownership analysis did not converge within its deterministic iteration bound"
                        .into(),
            });
        }

        let mut changed = false;
        for block_index in 0..block_count {
            let block = BasicBlock::from_usize(block_index);
            let mut incoming = Vec::new();
            if block_index == 0 {
                incoming.push(initial.clone());
            }
            for predecessor_edges in &edge_states {
                for (target, state) in predecessor_edges {
                    if *target == block {
                        incoming.push(state.clone());
                    }
                }
            }
            if incoming.is_empty() {
                continue;
            }

            let entry = FlowState::join(&incoming);
            if entry_states[block_index].as_ref() != Some(&entry) {
                entry_states[block_index] = Some(entry.clone());
                changed = true;
            }

            let new_edges = transfer_block(function, block, entry)?;
            if edge_states[block_index] != new_edges {
                edge_states[block_index] = new_edges;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    for (block_index, entry) in entry_states.into_iter().enumerate() {
        if let Some(entry) = entry {
            let _ = transfer_block(function, BasicBlock::from_usize(block_index), entry)?;
        }
    }
    Ok(())
}

fn transfer_block(
    function: &MirFunction,
    block: BasicBlock,
    mut state: FlowState,
) -> Result<Vec<(BasicBlock, FlowState)>, OwnershipVerificationError> {
    let data = &function.body.blocks[block];

    for (statement_index, statement) in data.statements.iter().enumerate() {
        let context = format!("statement {}", statement_index);
        match statement {
            Statement::Assign(destination, rvalue) => {
                transfer_rvalue(function, block, &context, rvalue, &mut state)?;
                assign_place(function, block, &context, destination, &mut state)?;
            }
            Statement::Drop(place) => {
                access_place(function, block, &context, place, AccessKind::Drop, &mut state)?;
            }
            Statement::BoundsCheck { index, .. } => {
                let place = Place::local(*index);
                access_place(function, block, &context, &place, AccessKind::Read, &mut state)?;
            }
            Statement::Assume(_) => {}
        }
    }

    match data.terminator.as_ref() {
        Some(Terminator::Goto(target)) => Ok(vec![(*target, state)]),
        Some(Terminator::SwitchInt { discr, targets, otherwise }) => {
            transfer_operand(function, block, "switch discriminant", discr, &mut state)?;
            let mut edges =
                targets.iter().map(|(_, target)| (*target, state.clone())).collect::<Vec<_>>();
            edges.push((*otherwise, state));
            Ok(edges)
        }
        Some(Terminator::Call { func: callee, args, destination, target, cleanup }) => {
            transfer_operand(function, block, "call callee", callee, &mut state)?;
            for (index, argument) in args.iter().enumerate() {
                transfer_operand(
                    function,
                    block,
                    &format!("call argument {}", index),
                    argument,
                    &mut state,
                )?;
            }
            let cleanup_state = state.clone();
            let mut normal_state = state;
            if let Some(destination) = destination {
                assign_place(function, block, "call destination", destination, &mut normal_state)?;
            }
            let mut edges = vec![(*target, normal_state)];
            if let Some(cleanup) = cleanup {
                edges.push((*cleanup, cleanup_state));
            }
            Ok(edges)
        }
        Some(Terminator::Return) => {
            let return_place = Place::local(function.return_place);
            access_place(
                function,
                block,
                "function return",
                &return_place,
                AccessKind::Read,
                &mut state,
            )?;
            Ok(Vec::new())
        }
        Some(Terminator::Unreachable) | None => Ok(Vec::new()),
    }
}

fn transfer_rvalue(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    rvalue: &Rvalue,
    state: &mut FlowState,
) -> Result<(), OwnershipVerificationError> {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) | Rvalue::Cast { operand, .. } => {
            transfer_operand(function, block, context, operand, state)
        }
        Rvalue::BinaryOp(_, lhs, rhs) => {
            transfer_operand(function, block, &format!("{context} lhs"), lhs, state)?;
            transfer_operand(function, block, &format!("{context} rhs"), rhs, state)
        }
        Rvalue::Aggregate { operands, .. } | Rvalue::EnumVariant { operands, .. } => {
            for (index, operand) in operands.iter().enumerate() {
                transfer_operand(
                    function,
                    block,
                    &format!("{context} operand {}", index),
                    operand,
                    state,
                )?;
            }
            Ok(())
        }
        Rvalue::Struct { fields, .. } => {
            for (name, operand) in fields {
                transfer_operand(
                    function,
                    block,
                    &format!("{context} field {name}"),
                    operand,
                    state,
                )?;
            }
            Ok(())
        }
        Rvalue::Reference { place, .. } => {
            transfer_place_access(function, block, context, place, AccessKind::Read, state)
        }
        Rvalue::Range { start, end, .. } => {
            transfer_operand(function, block, &format!("{context} range start"), start, state)?;
            transfer_operand(function, block, &format!("{context} range end"), end, state)
        }
        Rvalue::Field { base, .. } => transfer_operand(function, block, context, base, state),
        Rvalue::Index { base, index, .. } => {
            transfer_operand(function, block, &format!("{context} base"), base, state)?;
            transfer_operand(function, block, &format!("{context} index"), index, state)
        }
    }
}

fn transfer_operand(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    operand: &Operand,
    state: &mut FlowState,
) -> Result<(), OwnershipVerificationError> {
    match operand {
        Operand::Copy(place) => {
            transfer_place_access(function, block, context, place, AccessKind::Read, state)
        }
        Operand::Move(place) => {
            transfer_place_access(function, block, context, place, AccessKind::Move, state)
        }
        Operand::Constant(Constant::Lit(_)) | Operand::Constant(Constant::FnRef(_)) => Ok(()),
    }
}

fn transfer_place_access(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    place: &Place,
    access: AccessKind,
    state: &mut FlowState,
) -> Result<(), OwnershipVerificationError> {
    let ownership_place = ownership_place(function, place);
    state
        .ownership
        .access(ownership_place, access)
        .map_err(|error| violation(function, block, context, error))
}

fn assign_place(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    place: &Place,
    state: &mut FlowState,
) -> Result<(), OwnershipVerificationError> {
    let ownership_place = ownership_place(function, place);
    state
        .ownership
        .assign(ownership_place)
        .map_err(|error| violation(function, block, context, error))
}

fn ownership_place(function: &MirFunction, place: &Place) -> OwnershipPlace {
    let root = local_name(function, place.local);
    let projections = place
        .projections
        .iter()
        .map(|projection| match projection {
            Projection::Field(name) => OwnershipProjection::Field(name.clone()),
            Projection::ConstantIndex(index) => OwnershipProjection::Index(*index),
            Projection::Index(_) => OwnershipProjection::IndexAny,
            Projection::Deref => OwnershipProjection::Deref,
        })
        .collect();
    OwnershipPlace { root, projections }
}

fn local_name(function: &MirFunction, local: Local) -> String {
    function.body.local_decls[local]
        .name
        .clone()
        .unwrap_or_else(|| format!("__local{}", local.index()))
}

fn violation(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    error: OwnershipError,
) -> OwnershipVerificationError {
    OwnershipVerificationError {
        function: function.name.clone(),
        block,
        context: context.to_string(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use index_vec::IndexVec;
    use omni_mir::ast::TypeSpec;
    use omni_mir::ir::LocalDecl;

    fn make_program(
        local_types: Vec<(String, omni_mir::Ty)>,
        params: Vec<Local>,
        return_place: Local,
        statements: Vec<Statement>,
    ) -> MirProgram {
        let tcx = omni_mir::TyCtxt::new();
        let mut locals = IndexVec::new();
        for (name, ty) in local_types {
            locals.push(LocalDecl { name: Some(name), ty: Some(ty) });
        }
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData { statements, terminator: Some(Terminator::Return) });
        MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "main".into(),
                params,
                return_place,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        )
    }

    #[test]
    fn moved_local_cannot_be_read_again() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Move(Place::local(Local::from_usize(0)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(0)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "main".into(),
                params: vec![Local::from_usize(0)],
                return_place: Local::from_usize(1),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );
        let error = verify_program(&program).expect_err("use after move must be rejected");
        assert!(error.message.contains("moved"));
    }

    #[test]
    fn assignment_reinitializes_moved_local() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Move(Place::local(Local::from_usize(0)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(0)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(9)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(0)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "main".into(),
                params: vec![Local::from_usize(0)],
                return_place: Local::from_usize(1),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );
        verify_program(&program).expect("a moved local can be reinitialized before reuse");
    }

    #[test]
    fn copy_preserves_source_ownership() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(0)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(0)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "main".into(),
                params: vec![Local::from_usize(0)],
                return_place: Local::from_usize(1),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );
        verify_program(&program).expect("copy must not consume the source");
    }

    #[test]
    fn branch_join_does_not_assume_unmoved_value() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let boolean = tcx.intern(omni_mir::TyKind::Bool);
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("cond".into()), ty: Some(boolean) });

        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: Vec::new(),
            terminator: Some(Terminator::SwitchInt {
                discr: Operand::Copy(Place::local(Local::from_usize(2))),
                targets: vec![(1, BasicBlock::from_usize(1))],
                otherwise: BasicBlock::from_usize(2),
            }),
        });
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place::local(Local::from_usize(1)),
                Rvalue::Use(Operand::Move(Place::local(Local::from_usize(0)))),
            )],
            terminator: Some(Terminator::Goto(BasicBlock::from_usize(3))),
        });
        blocks.push(omni_mir::ir::BlockData {
            statements: Vec::new(),
            terminator: Some(Terminator::Goto(BasicBlock::from_usize(3))),
        });
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place::local(Local::from_usize(1)),
                Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(0)))),
            )],
            terminator: Some(Terminator::Return),
        });

        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "main".into(),
                params: vec![Local::from_usize(0), Local::from_usize(2)],
                return_place: Local::from_usize(1),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );

        let error = verify_program(&program).expect_err("join must reject a path that moved x");
        assert!(error.message.contains("moved"));
    }
}
