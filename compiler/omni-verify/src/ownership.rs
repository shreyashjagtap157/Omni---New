//! CFG-aware affine ownership and initialization verification for MIR.
use std::collections::{BTreeMap, BTreeSet};

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
    /// Reference locals may denote different loans after a CFG join, so keep
    /// the complete may-live region set for each local.
    reference_loans: BTreeMap<Local, BTreeSet<String>>,
    /// For each loan, the parent loans it may depend on through a reborrow.
    loan_parents: BTreeMap<String, BTreeSet<String>>,
}

impl FlowState {
    fn new() -> Self {
        Self {
            ownership: OwnershipState::new(),
            reference_loans: BTreeMap::new(),
            loan_parents: BTreeMap::new(),
        }
    }

    fn join(states: &[Self]) -> Self {
        let inputs = states.iter().map(|state| &state.ownership).collect::<Vec<_>>();
        let mut reference_loans = BTreeMap::new();
        for state in states {
            for (&local, loans) in &state.reference_loans {
                reference_loans
                    .entry(local)
                    .or_insert_with(BTreeSet::new)
                    .extend(loans.iter().cloned());
            }
        }
        let mut loan_parents = BTreeMap::new();
        for state in states {
            for (loan, parents) in &state.loan_parents {
                loan_parents
                    .entry(loan.clone())
                    .or_insert_with(BTreeSet::new)
                    .extend(parents.iter().cloned());
            }
        }
        Self { ownership: OwnershipState::join_all(&inputs), reference_loans, loan_parents }
    }
}

pub(crate) fn verify_program(program: &MirProgram) -> Result<(), OwnershipVerificationError> {
    for function in &program.functions {
        verify_function(function, &program.tcx)?;
    }
    Ok(())
}

fn verify_function(
    function: &MirFunction,
    tcx: &omni_mir::TyCtxt,
) -> Result<(), OwnershipVerificationError> {
    if function.body.blocks.is_empty() {
        return Ok(());
    }

    let block_count = function.body.blocks.len();
    let live_after = compute_local_liveness(function);
    let mut entry_states = vec![None::<FlowState>; block_count];
    let mut edge_states = vec![Vec::<(BasicBlock, FlowState)>::new(); block_count];

    let mut initial = FlowState::new();
    for &param in &function.params {
        initial.ownership.initialize(OwnershipPlace::root(local_name(function, param)));
        if let Some(ty) = function.body.local_decls[param].ty {
            if tcx.contains(ty) {
                if let omni_mir::TyKind::Reference { mutable, .. } = tcx.get(ty) {
                    // Distinct abstract origins let this function's dereference
                    // accesses be checked. Cross-function alias relationships
                    // remain unqualified and are not inferred from this seeding.
                    let origin = OwnershipPlace::root(format!(
                        "\0external-reference-origin:{}:{}",
                        function.name,
                        param.index()
                    ));
                    initial.ownership.initialize(origin.clone());
                    let region = format!("{}:parameter:{}", function.name, param.index());
                    let result = if *mutable {
                        initial.ownership.borrow_mut(origin, region.clone())
                    } else {
                        initial.ownership.borrow_shared(origin, region.clone())
                    };
                    result.map_err(|error| {
                        violation(function, BasicBlock::from_usize(0), "reference parameter", error)
                    })?;
                    initial.reference_loans.entry(param).or_default().insert(region);
                }
            }
        }
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

            let new_edges = transfer_block(function, block, entry, &live_after, tcx)?;
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
            let _ = transfer_block(
                function,
                BasicBlock::from_usize(block_index),
                entry,
                &live_after,
                tcx,
            )?;
        }
    }
    Ok(())
}

fn transfer_block(
    function: &MirFunction,
    block: BasicBlock,
    mut state: FlowState,
    live_after: &[Vec<BTreeSet<Local>>],
    tcx: &omni_mir::TyCtxt,
) -> Result<Vec<(BasicBlock, FlowState)>, OwnershipVerificationError> {
    let data = &function.body.blocks[block];

    for (statement_index, statement) in data.statements.iter().enumerate() {
        let context = format!("statement {}", statement_index);
        match statement {
            Statement::Assign(destination, rvalue) => {
                // A moved reference transfers its loan associations to the
                // destination; a self-reborrow replaces the local's association
                // only after the child loan has captured its parent dependency.
                let moved_reference = match (destination.projections.is_empty(), rvalue) {
                    (true, Rvalue::Use(Operand::Move(source))) if source.projections.is_empty() => {
                        state
                            .reference_loans
                            .get(&source.local)
                            .cloned()
                            .map(|loans| (source.local, loans))
                    }
                    _ => None,
                };
                let self_move = moved_reference
                    .as_ref()
                    .is_some_and(|(source, _)| *source == destination.local);
                let self_reborrow = destination.projections.is_empty()
                    && matches!(
                        rvalue,
                        Rvalue::Reference { place, .. }
                            if place.local == destination.local
                                && matches!(place.projections.first(), Some(Projection::Deref))
                    );

                if destination.projections.is_empty() && !self_move && !self_reborrow {
                    if let Some(old_loans) = state.reference_loans.remove(&destination.local) {
                        for region in old_loans {
                            end_tracked_loan_if_unreferenced(&mut state, &region);
                        }
                    }
                }

                let new_reference_loans = if let Rvalue::Reference { place, mutable, .. } = rvalue {
                    if !destination.projections.is_empty() {
                        return Err(OwnershipVerificationError {
                            function: function.name.clone(),
                            block,
                            context: context.clone(),
                            message: "reference stored in a projected place has no tracked provenance representation".into(),
                        });
                    }
                    Some(create_reference_loans(
                        function, block, &context, place, *mutable, &mut state,
                    )?)
                } else {
                    if let Some((source, loans)) = moved_reference {
                        transfer_place_access(
                            function,
                            block,
                            &context,
                            &Place::local(source),
                            AccessKind::Move,
                            &mut state,
                        )?;
                        if source != destination.local {
                            state.reference_loans.remove(&source);
                            if !loans.is_empty() {
                                state.reference_loans.insert(destination.local, loans);
                            }
                        }
                    } else {
                        transfer_rvalue(function, block, &context, rvalue, &mut state)?;
                    }
                    None
                };

                if let Some(loans) = new_reference_loans {
                    state.reference_loans.insert(destination.local, loans);
                }
                assign_place(function, block, &context, destination, &mut state)?;
            }
            Statement::Drop(place) => {
                if place.projections.is_empty() {
                    if let Some(loans) = state.reference_loans.remove(&place.local) {
                        for region in loans {
                            end_tracked_loan_if_unreferenced(&mut state, &region);
                        }
                    }
                }
                transfer_place_access(
                    function,
                    block,
                    &context,
                    place,
                    AccessKind::Drop,
                    &mut state,
                )?;
            }
            Statement::BoundsCheck { index, .. } => {
                let place = Place::local(*index);
                transfer_place_access(
                    function,
                    block,
                    &context,
                    &place,
                    AccessKind::Read,
                    &mut state,
                )?;
            }
            Statement::Assume(_) => {}
        }

        shorten_dead_reference_loans(&mut state, &live_after[block.index()][statement_index]);
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
        Some(Terminator::SwitchEnum { place, targets, otherwise, .. }) => {
            transfer_place_access(
                function,
                block,
                "enum switch discriminant",
                place,
                AccessKind::Read,
                &mut state,
            )?;
            let mut edges =
                targets.iter().map(|(_, target)| (*target, state.clone())).collect::<Vec<_>>();
            edges.push((*otherwise, state));
            Ok(edges)
        }
        Some(Terminator::Call { func: callee, args, destination, target, cleanup }) => {
            verify_call_argument_aliases(function, block, args, &state, tcx)?;
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
            if !matches!(function.return_type, omni_mir::ast::TypeSpec::Unit) {
                let return_place = Place::local(function.return_place);
                transfer_place_access(
                    function,
                    block,
                    "function return",
                    &return_place,
                    AccessKind::Read,
                    &mut state,
                )?;
            }
            Ok(Vec::new())
        }
        Some(Terminator::Unreachable) | None => Ok(Vec::new()),
    }
}

fn compute_local_liveness(function: &MirFunction) -> Vec<Vec<BTreeSet<Local>>> {
    let block_count = function.body.blocks.len();
    let mut block_use = vec![BTreeSet::<Local>::new(); block_count];
    let mut block_def = vec![BTreeSet::<Local>::new(); block_count];
    let mut successors = vec![Vec::<BasicBlock>::new(); block_count];

    for (index, block) in function.body.blocks.iter().enumerate() {
        let mut seen_defs = BTreeSet::new();
        for statement in &block.statements {
            let (uses, defs) = statement_local_effects(statement);
            for local in uses {
                if !seen_defs.contains(&local) {
                    block_use[index].insert(local);
                }
            }
            for local in defs {
                seen_defs.insert(local);
                block_def[index].insert(local);
            }
        }

        let (term_uses, term_defs, term_successors) =
            terminator_local_effects(function, block.terminator.as_ref());
        for local in term_uses {
            if !seen_defs.contains(&local) {
                block_use[index].insert(local);
            }
        }
        block_def[index].extend(term_defs);
        successors[index] = term_successors;
    }

    let mut live_in = vec![BTreeSet::<Local>::new(); block_count];
    let mut live_out = vec![BTreeSet::<Local>::new(); block_count];

    loop {
        let mut changed = false;
        for index in (0..block_count).rev() {
            let mut new_out = BTreeSet::new();
            for successor in &successors[index] {
                new_out.extend(live_in[successor.index()].iter().copied());
            }

            let mut new_in = block_use[index].clone();
            for local in new_out.difference(&block_def[index]) {
                new_in.insert(*local);
            }

            if new_out != live_out[index] || new_in != live_in[index] {
                live_out[index] = new_out;
                live_in[index] = new_in;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut live_after = vec![Vec::<BTreeSet<Local>>::new(); block_count];
    for index in 0..block_count {
        let block = &function.body.blocks[BasicBlock::from_usize(index)];
        let (term_uses, term_defs, _) =
            terminator_local_effects(function, block.terminator.as_ref());
        let mut live = live_out[index].clone();
        for local in term_defs {
            live.remove(&local);
        }
        live.extend(term_uses);

        live_after[index].resize(block.statements.len(), BTreeSet::new());
        for statement_index in (0..block.statements.len()).rev() {
            live_after[index][statement_index] = live.clone();
            let (uses, defs) = statement_local_effects(&block.statements[statement_index]);
            for local in defs {
                live.remove(&local);
            }
            live.extend(uses);
        }
    }

    live_after
}

fn statement_local_effects(statement: &Statement) -> (BTreeSet<Local>, BTreeSet<Local>) {
    let mut uses = BTreeSet::new();
    let mut defs = BTreeSet::new();

    match statement {
        Statement::Assign(destination, rvalue) => {
            place_local_effects(destination, &mut uses, &mut defs, true);
            rvalue_local_uses(rvalue, &mut uses);
        }
        Statement::Drop(place) => {
            place_local_effects(place, &mut uses, &mut defs, false);
        }
        Statement::BoundsCheck { index, .. } => {
            uses.insert(*index);
        }
        Statement::Assume(_) => {}
    }

    (uses, defs)
}

fn terminator_local_effects(
    function: &MirFunction,
    terminator: Option<&Terminator>,
) -> (BTreeSet<Local>, BTreeSet<Local>, Vec<BasicBlock>) {
    let mut uses = BTreeSet::new();
    let mut defs = BTreeSet::new();

    let successors = match terminator {
        Some(Terminator::Goto(target)) => vec![*target],
        Some(Terminator::SwitchInt { discr, targets, otherwise }) => {
            operand_local_uses(discr, &mut uses);
            let mut result = targets.iter().map(|(_, target)| *target).collect::<Vec<_>>();
            result.push(*otherwise);
            result
        }
        Some(Terminator::SwitchEnum { place, targets, otherwise, .. }) => {
            place_local_effects(place, &mut uses, &mut defs, false);
            let mut result = targets.iter().map(|(_, target)| *target).collect::<Vec<_>>();
            result.push(*otherwise);
            result
        }
        Some(Terminator::Call { func, args, destination, target, cleanup }) => {
            operand_local_uses(func, &mut uses);
            for arg in args {
                operand_local_uses(arg, &mut uses);
            }
            if let Some(destination) = destination {
                place_local_effects(destination, &mut uses, &mut defs, true);
            }
            let mut result = vec![*target];
            if let Some(cleanup) = cleanup {
                result.push(*cleanup);
            }
            result
        }
        Some(Terminator::Return) => {
            if !matches!(function.return_type, omni_mir::ast::TypeSpec::Unit) {
                uses.insert(function.return_place);
            }
            Vec::new()
        }
        Some(Terminator::Unreachable) | None => Vec::new(),
    };

    (uses, defs, successors)
}

fn place_local_effects(
    place: &Place,
    uses: &mut BTreeSet<Local>,
    defs: &mut BTreeSet<Local>,
    writing: bool,
) {
    if writing && place.projections.is_empty() {
        defs.insert(place.local);
    } else {
        uses.insert(place.local);
    }

    for projection in &place.projections {
        if let Projection::Index(local) = projection {
            uses.insert(*local);
        }
    }
}

fn rvalue_local_uses(rvalue: &Rvalue, uses: &mut BTreeSet<Local>) {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) | Rvalue::Cast { operand, .. } => {
            operand_local_uses(operand, uses)
        }
        Rvalue::BinaryOp(_, lhs, rhs) => {
            operand_local_uses(lhs, uses);
            operand_local_uses(rhs, uses);
        }
        Rvalue::Aggregate { operands, .. } | Rvalue::EnumVariant { operands, .. } => {
            for operand in operands {
                operand_local_uses(operand, uses);
            }
        }
        Rvalue::Struct { fields, .. } => {
            for (_, operand) in fields {
                operand_local_uses(operand, uses);
            }
        }
        Rvalue::Reference { place, .. } => {
            place_local_effects(place, uses, &mut BTreeSet::new(), false);
        }
        Rvalue::Range { start, end, .. } => {
            operand_local_uses(start, uses);
            operand_local_uses(end, uses);
        }
        Rvalue::Field { base, .. } | Rvalue::EnumField { base, .. } => {
            operand_local_uses(base, uses)
        }
        Rvalue::Index { base, index, .. } => {
            operand_local_uses(base, uses);
            operand_local_uses(index, uses);
        }
    }
}

fn operand_local_uses(operand: &Operand, uses: &mut BTreeSet<Local>) {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            place_local_effects(place, uses, &mut BTreeSet::new(), false);
        }
        Operand::Constant(_) => {}
    }
}

fn end_tracked_loan(state: &mut FlowState, region: &str) {
    let _ = state.ownership.end_loan(region);
    state.loan_parents.remove(region);
    for parents in state.loan_parents.values_mut() {
        parents.remove(region);
    }
}

fn loan_has_active_children(state: &FlowState, region: &str) -> bool {
    state.loan_parents.iter().any(|(child, parents)| {
        child != region
            && parents.contains(region)
            && state.reference_loans.values().any(|loans| loans.contains(child))
    })
}

/// End a loan only after no live reference local or active child reborrow
/// depends on it. Releasing a child can make an otherwise orphaned parent
/// eligible for release, so walk parent links transitively and guard against
/// malformed cyclic metadata.
fn end_tracked_loan_if_unreferenced(state: &mut FlowState, region: &str) {
    let mut pending = vec![region.to_string()];
    let mut visited = BTreeSet::new();

    while let Some(current) = pending.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }

        let has_reference = state.reference_loans.values().any(|loans| loans.contains(&current));
        if has_reference || loan_has_active_children(state, &current) {
            continue;
        }

        let parents = state.loan_parents.get(&current).cloned().unwrap_or_default();
        end_tracked_loan(state, &current);
        pending.extend(parents);
    }
}

fn shorten_dead_reference_loans(state: &mut FlowState, live_after: &BTreeSet<Local>) {
    loop {
        let dead_locals = state
            .reference_loans
            .keys()
            .copied()
            .filter(|local| !live_after.contains(local))
            .collect::<Vec<_>>();
        let mut changed = false;

        for local in dead_locals {
            let loans = state.reference_loans.get(&local).cloned().unwrap_or_default();
            for region in loans {
                if let Some(active) = state.reference_loans.get_mut(&local) {
                    active.remove(&region);
                }
                changed = true;
                end_tracked_loan_if_unreferenced(state, &region);
            }

            if state.reference_loans.get(&local).is_some_and(BTreeSet::is_empty) {
                state.reference_loans.remove(&local);
            }
        }

        if !changed {
            break;
        }
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
        // Reference creation is handled by the assignment transfer so the
        // created loan and its reference-local provenance are updated atomically.
        Rvalue::Reference { .. } => Ok(()),
        Rvalue::Range { start, end, .. } => {
            transfer_operand(function, block, &format!("{context} range start"), start, state)?;
            transfer_operand(function, block, &format!("{context} range end"), end, state)
        }
        Rvalue::Field { base, .. } | Rvalue::EnumField { base, .. } => {
            transfer_operand(function, block, context, base, state)
        }
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
            transfer_place_access(function, block, context, place, AccessKind::Move, state)?;
            if place.projections.is_empty() {
                if let Some(loans) = state.reference_loans.remove(&place.local) {
                    for region in loans {
                        end_tracked_loan_if_unreferenced(state, &region);
                    }
                }
            }
            Ok(())
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
    if let Some(origins) = resolve_deref_origins(function, block, context, place, state)? {
        for origin in origins {
            if matches!(
                access,
                AccessKind::Write | AccessKind::Move | AccessKind::Drop | AccessKind::BorrowMut
            ) && !origin.mutable
            {
                return Err(violation(
                    function,
                    block,
                    context,
                    OwnershipError::WriteThroughSharedBorrow(origin.place),
                ));
            }
            apply_ownership_access_with_authority(
                function,
                block,
                context,
                origin.place,
                access,
                &origin.authority,
                &mut state.ownership,
            )?;
        }
        return Ok(());
    }

    let ownership_place = ownership_place(function, place);
    apply_ownership_access(function, block, context, ownership_place, access, &mut state.ownership)
}

fn assign_place(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    place: &Place,
    state: &mut FlowState,
) -> Result<(), OwnershipVerificationError> {
    if let Some(origins) = resolve_deref_origins(function, block, context, place, state)? {
        for origin in origins {
            if !origin.mutable {
                return Err(violation(
                    function,
                    block,
                    context,
                    OwnershipError::WriteThroughSharedBorrow(origin.place),
                ));
            }
            state
                .ownership
                .assign_with_authority(origin.place, &origin.authority)
                .map_err(|error| violation(function, block, context, error))?;
        }
        return Ok(());
    }

    let ownership_place = ownership_place(function, place);
    state
        .ownership
        .assign(ownership_place)
        .map_err(|error| violation(function, block, context, error))
}

#[derive(Debug, Clone)]
struct DerefOrigin {
    place: OwnershipPlace,
    authority: BTreeSet<String>,
    mutable: bool,
}

#[derive(Debug, Clone)]
struct PendingReborrow {
    parents: BTreeSet<String>,
    authority: BTreeSet<String>,
    mutable: bool,
}

/// Resolve the first dereference projection to the place(s) borrowed by the
/// tracked reference local. Unsupported projected-reference and nested-deref
/// shapes fail closed rather than treating a pointer projection as independent
/// storage.
fn resolve_deref_origins(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    place: &Place,
    state: &FlowState,
) -> Result<Option<Vec<DerefOrigin>>, OwnershipVerificationError> {
    let Some(deref_index) =
        place.projections.iter().position(|projection| matches!(projection, Projection::Deref))
    else {
        return Ok(None);
    };
    if deref_index != 0
        || place
            .projections
            .iter()
            .skip(1)
            .any(|projection| matches!(projection, Projection::Deref))
    {
        return Err(OwnershipVerificationError {
            function: function.name.clone(),
            block,
            context: context.to_string(),
            message: "nested or projected reference provenance is not yet representable".into(),
        });
    }

    let regions = state.reference_loans.get(&place.local).cloned().unwrap_or_default();
    if regions.is_empty() {
        return Err(OwnershipVerificationError {
            function: function.name.clone(),
            block,
            context: context.to_string(),
            message: format!(
                "cannot verify dereference of local '{}' without tracked loan provenance",
                local_name(function, place.local)
            ),
        });
    }

    let mut grouped = BTreeMap::<OwnershipPlace, DerefOrigin>::new();
    for region in regions {
        let loan = state.ownership.loans().find(|loan| loan.region == region).cloned().ok_or_else(
            || violation(function, block, context, OwnershipError::UnknownLoan(region.clone())),
        )?;
        let mut origin = loan.place.clone();
        for projection in place.projections.iter().skip(1) {
            origin.projections.push(match projection {
                Projection::Field(name) => OwnershipProjection::Field(name.clone()),
                Projection::ConstantIndex(index) => OwnershipProjection::Index(*index),
                Projection::Index(_) => OwnershipProjection::IndexAny,
                Projection::Deref => unreachable!("nested dereferences were rejected above"),
            });
        }

        let authority = collect_loan_authority(state, &region);
        match grouped.entry(origin.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(DerefOrigin { place: origin, authority, mutable: loan.mutable });
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                entry.get_mut().authority.extend(authority);
                entry.get_mut().mutable &= loan.mutable;
            }
        }
    }

    Ok(Some(grouped.into_values().collect()))
}

fn collect_loan_authority(state: &FlowState, region: &str) -> BTreeSet<String> {
    let mut authority = BTreeSet::new();
    let mut pending = vec![region.to_string()];
    while let Some(current) = pending.pop() {
        if !authority.insert(current.clone()) {
            continue;
        }
        if let Some(parents) = state.loan_parents.get(&current) {
            pending.extend(parents.iter().cloned());
        }
    }
    authority
}

fn create_reference_loans(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    place: &Place,
    mutable: bool,
    state: &mut FlowState,
) -> Result<BTreeSet<String>, OwnershipVerificationError> {
    let base_region = format!("{}:bb{}:{}", function.name, block.index(), context);
    let Some(deref_index) =
        place.projections.iter().position(|projection| matches!(projection, Projection::Deref))
    else {
        let target = ownership_place(function, place);
        let result = if mutable {
            state.ownership.borrow_mut(target, base_region.clone())
        } else {
            state.ownership.borrow_shared(target, base_region.clone())
        };
        result.map_err(|error| violation(function, block, context, error))?;
        state.loan_parents.insert(base_region.clone(), BTreeSet::new());
        return Ok(BTreeSet::from([base_region]));
    };

    if deref_index != 0
        || place
            .projections
            .iter()
            .skip(1)
            .any(|projection| matches!(projection, Projection::Deref))
    {
        return Err(OwnershipVerificationError {
            function: function.name.clone(),
            block,
            context: context.to_string(),
            message: "reborrow through a nested or projected reference is not yet representable"
                .into(),
        });
    }

    let parents = state.reference_loans.get(&place.local).cloned().unwrap_or_default();
    if parents.is_empty() {
        return Err(OwnershipVerificationError {
            function: function.name.clone(),
            block,
            context: context.to_string(),
            message: format!(
                "cannot create reborrow from local '{}' without tracked loan provenance",
                local_name(function, place.local)
            ),
        });
    }

    let mut grouped = BTreeMap::<OwnershipPlace, PendingReborrow>::new();
    for parent_region in parents {
        let parent_loan =
            state.ownership.loans().find(|loan| loan.region == parent_region).cloned().ok_or_else(
                || {
                    violation(
                        function,
                        block,
                        context,
                        OwnershipError::UnknownLoan(parent_region.clone()),
                    )
                },
            )?;
        if mutable && !parent_loan.mutable {
            return Err(violation(
                function,
                block,
                context,
                OwnershipError::WriteThroughSharedBorrow(parent_loan.place),
            ));
        }

        let mut origin = parent_loan.place.clone();
        for projection in place.projections.iter().skip(1) {
            origin.projections.push(match projection {
                Projection::Field(name) => OwnershipProjection::Field(name.clone()),
                Projection::ConstantIndex(index) => OwnershipProjection::Index(*index),
                Projection::Index(_) => OwnershipProjection::IndexAny,
                Projection::Deref => unreachable!("nested dereferences were rejected above"),
            });
        }
        let authority = collect_loan_authority(state, &parent_region);
        let entry = grouped.entry(origin).or_insert_with(|| PendingReborrow {
            parents: BTreeSet::new(),
            authority: BTreeSet::new(),
            mutable: parent_loan.mutable,
        });
        entry.parents.insert(parent_region);
        entry.authority.extend(authority);
        entry.mutable &= parent_loan.mutable;
    }

    let multiple_origins = grouped.len() > 1;
    let mut new_regions = BTreeSet::new();
    for (index, (origin, pending)) in grouped.into_iter().enumerate() {
        if mutable && !pending.mutable {
            return Err(violation(
                function,
                block,
                context,
                OwnershipError::WriteThroughSharedBorrow(origin),
            ));
        }
        let region = if multiple_origins {
            format!("{base_region}:origin:{index}")
        } else {
            base_region.clone()
        };
        if pending.authority.contains(&region) {
            return Err(OwnershipVerificationError {
                function: function.name.clone(),
                block,
                context: context.to_string(),
                message: "reborrow region collides with a live parent loan; loop-carried provenance is not yet representable".into(),
            });
        }
        let result = if mutable {
            state.ownership.borrow_mut_with_parents(origin, region.clone(), &pending.authority)
        } else {
            state.ownership.borrow_shared_with_parents(origin, region.clone(), &pending.authority)
        };
        result.map_err(|error| violation(function, block, context, error))?;
        state.loan_parents.insert(region.clone(), pending.parents);
        new_regions.insert(region);
    }
    Ok(new_regions)
}

#[derive(Debug, Clone)]
struct CallArgumentLoan {
    argument_index: usize,
    place: OwnershipPlace,
    mutable: bool,
}

fn verify_call_argument_aliases(
    function: &MirFunction,
    block: BasicBlock,
    args: &[Operand],
    state: &FlowState,
    tcx: &omni_mir::TyCtxt,
) -> Result<(), OwnershipVerificationError> {
    let mut loans = Vec::<CallArgumentLoan>::new();
    for (argument_index, argument) in args.iter().enumerate() {
        let place = match argument {
            Operand::Copy(place) | Operand::Move(place) if place.projections.is_empty() => place,
            _ => continue,
        };
        let is_reference = match function.body.local_decls[place.local].ty {
            Some(ty) if tcx.contains(ty) => {
                matches!(tcx.get(ty), omni_mir::TyKind::Reference { .. })
            }
            _ => false,
        };
        if !is_reference {
            continue;
        }
        let is_copy = matches!(argument, Operand::Copy(_));
        let regions = state.reference_loans.get(&place.local).cloned().unwrap_or_default();
        if regions.is_empty() {
            return Err(OwnershipVerificationError {
                function: function.name.clone(),
                block,
                context: format!("call argument {argument_index}"),
                message: "reference argument has no tracked loan provenance".into(),
            });
        }
        for region in regions {
            let loan =
                state.ownership.loans().find(|loan| loan.region == region).cloned().ok_or_else(
                    || {
                        violation(
                            function,
                            block,
                            "call arguments",
                            OwnershipError::UnknownLoan(region.clone()),
                        )
                    },
                )?;
            if is_copy && loan.mutable {
                return Err(OwnershipVerificationError {
                    function: function.name.clone(),
                    block,
                    context: format!("call argument {argument_index}"),
                    message: "copying a mutable reference is not permitted".into(),
                });
            }
            if loan.mutable && loan_has_active_children(state, &region) {
                return Err(OwnershipVerificationError {
                    function: function.name.clone(),
                    block,
                    context: format!("call argument {argument_index}"),
                    message: "mutable parent reference cannot be passed while a child reborrow remains live".into(),
                });
            }
            for previous in &loans {
                if previous.argument_index != argument_index
                    && ownership_places_overlap(&loan.place, &previous.place)
                    && (loan.mutable || previous.mutable)
                {
                    let error = if loan.mutable {
                        OwnershipError::MutableBorrowConflict {
                            place: loan.place.clone(),
                            existing: if previous.mutable { "mutable" } else { "shared" }.into(),
                        }
                    } else {
                        OwnershipError::BorrowConflict {
                            place: loan.place.clone(),
                            existing: "mutable".into(),
                        }
                    };
                    return Err(violation(function, block, "call arguments", error));
                }
            }
            loans.push(CallArgumentLoan {
                argument_index,
                place: loan.place,
                mutable: loan.mutable,
            });
        }
    }
    Ok(())
}

fn ownership_place_prefix(prefix: &OwnershipPlace, value: &OwnershipPlace) -> bool {
    prefix.root == value.root
        && prefix.projections.len() <= value.projections.len()
        && prefix.projections.iter().zip(&value.projections).all(|(left, right)| {
            matches!(
                (left, right),
                (OwnershipProjection::IndexAny, _) | (_, OwnershipProjection::IndexAny)
            ) || left == right
        })
}

fn ownership_places_overlap(left: &OwnershipPlace, right: &OwnershipPlace) -> bool {
    ownership_place_prefix(left, right) || ownership_place_prefix(right, left)
}

fn apply_ownership_access_with_authority(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    place: OwnershipPlace,
    access: AccessKind,
    authority: &BTreeSet<String>,
    ownership: &mut OwnershipState,
) -> Result<(), OwnershipVerificationError> {
    let result = match access {
        AccessKind::Read => ownership.read_with_authority(&place, authority),
        AccessKind::Move => ownership.move_with_authority(place, authority),
        AccessKind::Write => ownership.assign_with_authority(place, authority),
        AccessKind::Drop => ownership.drop_with_authority(&place, authority),
        AccessKind::BorrowShared | AccessKind::BorrowMut => {
            let region = format!("{}:bb{}:{}", function.name, block.index(), context);
            if matches!(access, AccessKind::BorrowShared) {
                ownership.borrow_shared_with_parents(place, region, authority)
            } else {
                ownership.borrow_mut_with_parents(place, region, authority)
            }
        }
    };
    result.map_err(|error| violation(function, block, context, error))
}

fn apply_ownership_access(
    function: &MirFunction,
    block: BasicBlock,
    context: &str,
    place: OwnershipPlace,
    access: AccessKind,
    ownership: &mut OwnershipState,
) -> Result<(), OwnershipVerificationError> {
    let result = match access {
        AccessKind::Read => ownership.read(&place),
        AccessKind::Move => ownership.move_place(place),
        AccessKind::Write => ownership.assign(place),
        AccessKind::Drop => ownership.drop_place(&place),
        AccessKind::BorrowShared | AccessKind::BorrowMut => {
            // Borrow creation requires a stable region identifier. Full MIR
            // lifetime elaboration owns region derivation; this verifier only
            // consumes explicit borrow accesses and therefore uses the current
            // CFG point as a deterministic local region identifier.
            let region = format!("{}:bb{}:{}", function.name, block.index(), context);
            if matches!(access, AccessKind::BorrowShared) {
                ownership.borrow_shared(place, region)
            } else {
                ownership.borrow_mut(place, region)
            }
        }
    };
    result.map_err(|error| violation(function, block, context, error))
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
    let display_name =
        function.body.local_decls[local].name.clone().unwrap_or_else(|| "__local".to_string());
    // Ownership identity is local-identity based, not source-name based:
    // MIR legitimately contains many compiler temporaries with repeated names.
    format!("{display_name}#{}", local.index())
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

    #[test]
    fn dropping_reference_ends_its_associated_loan() {
        let mut state = OwnershipState::new();
        let place = OwnershipPlace::root("x");
        state.initialize(place.clone());
        state.borrow_shared(place, "loan").expect("borrow");
        state.end_loan("loan").expect("loan remains explicitly endable");
    }

    #[test]
    fn borrow_ends_after_last_reference_use() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let reference =
            tcx.intern(omni_mir::TyKind::Reference { lifetime: None, mutable: true, inner: int });
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("r".into()), ty: Some(reference) });
        locals.push(LocalDecl { name: Some("y".into()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(1)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(2)),
                    Rvalue::Reference {
                        place: Place::local(Local::from_usize(1)),
                        mutable: true,
                        ty: reference,
                    },
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(2)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(2)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(0)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "nll_last_use".into(),
                params: vec![],
                return_place: Local::from_usize(0),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );

        verify_program(&program).expect("borrow should end after its last use");
    }

    #[test]
    fn reborrow_keeps_parent_loan_live_until_child_last_use() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let mutable_ref =
            tcx.intern(omni_mir::TyKind::Reference { lifetime: None, mutable: true, inner: int });
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("r".into()), ty: Some(mutable_ref) });
        locals.push(LocalDecl { name: Some("s".into()), ty: Some(mutable_ref) });
        locals.push(LocalDecl { name: Some("value".into()), ty: Some(int) });

        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(1)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(2)),
                    Rvalue::Reference {
                        place: Place::local(Local::from_usize(1)),
                        mutable: true,
                        ty: mutable_ref,
                    },
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(3)),
                    Rvalue::Reference {
                        place: Place {
                            local: Local::from_usize(2),
                            projections: vec![Projection::Deref],
                        },
                        mutable: true,
                        ty: mutable_ref,
                    },
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(2)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(4)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(3)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(3)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(0)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        });

        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "reborrow_liveness".into(),
                params: vec![],
                return_place: Local::from_usize(0),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );

        let error = verify_program(&program).expect_err(
            "parent mutable loan must still conflict while child reborrow remains live",
        );
        assert!(error.message.contains("borrow conflict"), "unexpected ownership error: {error:?}");
    }

    #[test]
    fn duplicate_mir_local_names_remain_distinct_ownership_places() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("_return".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("_tmp".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("_tmp".into()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(1)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(2)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(2)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local::from_usize(1)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local::from_usize(2)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "duplicate_names".into(),
                params: vec![],
                return_place: Local::from_usize(0),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );

        // The two compiler temporaries are different MIR places despite sharing
        // the same display name.
        verify_program(&program)
            .expect("duplicate local names must not alias ownership identities");
    }

    #[test]
    fn mutable_reference_conflicts_when_write_precedes_last_use() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let reference =
            tcx.intern(omni_mir::TyKind::Reference { lifetime: None, mutable: true, inner: int });
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("r".into()), ty: Some(reference) });
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(
                    Place::local(Local::from_usize(0)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(1)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(1)),
                    Rvalue::Reference {
                        place: Place::local(Local::from_usize(0)),
                        mutable: true,
                        ty: reference,
                    },
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(0)),
                    Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(2)))),
                ),
                Statement::Assign(
                    Place::local(Local::from_usize(2)),
                    Rvalue::Use(Operand::Copy(Place::local(Local::from_usize(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "main".into(),
                params: vec![],
                return_place: Local::from_usize(2),
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );

        let error =
            verify_program(&program).expect_err("write before the reference's last use must fail");
        assert!(error.message.contains("borrow conflict"), "unexpected ownership error: {error:?}");
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
    fn moved_mutable_reference_program(write_before_last_use: bool) -> MirProgram {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let mutable_ref =
            tcx.intern(omni_mir::TyKind::Reference { lifetime: None, mutable: true, inner: int });

        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("source".into()), ty: Some(mutable_ref) });
        locals.push(LocalDecl { name: Some("destination".into()), ty: Some(mutable_ref) });
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });

        let x = Local::from_usize(0);
        let source = Local::from_usize(1);
        let destination = Local::from_usize(2);
        let ret = Local::from_usize(3);
        let read_destination =
            || Place { local: destination, projections: vec![Projection::Deref] };
        let integer =
            |value| Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(value))));

        let mut statements = vec![
            Statement::Assign(Place::local(x), integer(1)),
            Statement::Assign(
                Place::local(source),
                Rvalue::Reference { place: Place::local(x), mutable: true, ty: mutable_ref },
            ),
            Statement::Assign(
                Place::local(destination),
                Rvalue::Use(Operand::Move(Place::local(source))),
            ),
        ];

        if write_before_last_use {
            statements.push(Statement::Assign(Place::local(x), integer(2)));
            statements.push(Statement::Assign(
                Place::local(ret),
                Rvalue::Use(Operand::Copy(read_destination())),
            ));
        } else {
            statements.push(Statement::Assign(
                Place::local(ret),
                Rvalue::Use(Operand::Copy(read_destination())),
            ));
            statements.push(Statement::Assign(Place::local(x), integer(2)));
            statements.push(Statement::Assign(
                Place::local(ret),
                Rvalue::Use(Operand::Copy(Place::local(x))),
            ));
        }

        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData { statements, terminator: Some(Terminator::Return) });

        MirProgram::new(
            tcx,
            vec![MirFunction {
                name: if write_before_last_use {
                    "moved_reference_conflict".into()
                } else {
                    "moved_reference_last_use".into()
                },
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        )
    }

    #[test]
    fn moving_mutable_reference_preserves_loan_until_destination_last_use() {
        let program = moved_mutable_reference_program(true);
        let error = verify_program(&program)
            .expect_err("moving a mutable reference must not release its live loan");
        assert!(error.message.contains("borrow conflict"), "unexpected ownership error: {error:?}");
    }

    #[test]
    fn moved_mutable_reference_loan_ends_after_destination_last_use() {
        let program = moved_mutable_reference_program(false);
        verify_program(&program).expect("loan should end after the moved reference's last use");
    }

    fn dereference_access_program(
        name: &str,
        reference_mutable: bool,
        self_reborrow: bool,
        write_through_reference: bool,
        write_origin_before_last_use: bool,
        write_origin_after_last_use: bool,
        mutable_reborrow_from_reference: bool,
    ) -> MirProgram {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let reference_ty = tcx.intern(omni_mir::TyKind::Reference {
            lifetime: None,
            mutable: reference_mutable,
            inner: int,
        });
        let child_ty =
            tcx.intern(omni_mir::TyKind::Reference { lifetime: None, mutable: true, inner: int });
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("r".into()), ty: Some(reference_ty) });
        locals.push(LocalDecl { name: Some("s".into()), ty: Some(child_ty) });
        let ret = Local::from_usize(0);
        let x = Local::from_usize(1);
        let r = Local::from_usize(2);
        let s = Local::from_usize(3);
        let integer =
            |value| Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(value))));
        let deref = |local| Place { local, projections: vec![Projection::Deref] };
        let mut statements = vec![
            Statement::Assign(Place::local(x), integer(1)),
            Statement::Assign(
                Place::local(r),
                Rvalue::Reference {
                    place: Place::local(x),
                    mutable: reference_mutable,
                    ty: reference_ty,
                },
            ),
        ];
        if self_reborrow {
            statements.push(Statement::Assign(
                Place::local(r),
                Rvalue::Reference { place: deref(r), mutable: true, ty: child_ty },
            ));
        }
        if mutable_reborrow_from_reference {
            statements.push(Statement::Assign(
                Place::local(s),
                Rvalue::Reference { place: deref(r), mutable: true, ty: child_ty },
            ));
        }
        if write_through_reference {
            statements.push(Statement::Assign(deref(r), integer(2)));
        }
        if write_origin_before_last_use {
            statements.push(Statement::Assign(Place::local(x), integer(3)));
        }
        statements.push(Statement::Assign(
            Place::local(ret),
            Rvalue::Use(Operand::Copy(deref(if mutable_reborrow_from_reference { s } else { r }))),
        ));
        if write_origin_after_last_use {
            statements.push(Statement::Assign(Place::local(x), integer(4)));
            statements.push(Statement::Assign(
                Place::local(ret),
                Rvalue::Use(Operand::Copy(Place::local(x))),
            ));
        }
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData { statements, terminator: Some(Terminator::Return) });
        MirProgram::new(
            tcx,
            vec![MirFunction {
                name: name.into(),
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        )
    }

    #[test]
    fn self_reborrow_preserves_origin_loan_until_destination_last_use() {
        let program =
            dereference_access_program("self_reborrow_live", true, true, false, true, false, false);
        let error = verify_program(&program)
            .expect_err("self-reborrow must keep the borrowed origin protected");
        assert!(error.message.contains("borrow conflict"), "unexpected ownership error: {error:?}");
    }

    #[test]
    fn self_reborrow_loan_ends_after_destination_last_use() {
        let program = dereference_access_program(
            "self_reborrow_last_use",
            true,
            true,
            false,
            false,
            true,
            false,
        );
        verify_program(&program).expect("loan should end after the reborrow's last use");
    }

    #[test]
    fn mutable_reference_can_access_its_own_dereferenced_loan() {
        let program = dereference_access_program(
            "mutable_deref_access",
            true,
            false,
            true,
            false,
            false,
            false,
        );
        verify_program(&program).expect("a mutable reference may access its own authorized loan");
    }

    #[test]
    fn shared_reference_cannot_be_written_through() {
        let program = dereference_access_program(
            "shared_deref_write",
            false,
            false,
            true,
            false,
            false,
            false,
        );
        let error = verify_program(&program)
            .expect_err("writes through a shared reference must be rejected");
        assert!(error.message.contains("shared borrow"), "unexpected ownership error: {error:?}");
    }

    #[test]
    fn mutable_reborrow_from_shared_reference_is_rejected() {
        let program = dereference_access_program(
            "mutable_reborrow_from_shared",
            false,
            false,
            false,
            false,
            false,
            true,
        );
        let error = verify_program(&program)
            .expect_err("a shared reference cannot authorize a mutable reborrow");
        assert!(error.message.contains("shared borrow"), "unexpected ownership error: {error:?}");
    }

    #[test]
    fn parent_reference_is_suspended_while_child_reborrow_is_live() {
        let program =
            dereference_access_program("parent_suspended", true, false, true, false, false, true);
        let error = verify_program(&program).expect_err(
            "the parent reference must not access the place during a live child reborrow",
        );
        assert!(error.message.contains("borrow conflict"), "unexpected ownership error: {error:?}");
    }

    #[test]
    fn reference_parameter_dereference_uses_an_abstract_origin() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let reference_ty =
            tcx.intern(omni_mir::TyKind::Reference { lifetime: None, mutable: true, inner: int });
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("value".into()), ty: Some(reference_ty) });
        let ret = Local::from_usize(0);
        let value = Local::from_usize(1);
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place::local(ret),
                Rvalue::Use(Operand::Copy(Place {
                    local: value,
                    projections: vec![Projection::Deref],
                })),
            )],
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "reference_parameter".into(),
                params: vec![value],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );
        verify_program(&program)
            .expect("a reference parameter must have tracked abstract provenance");
    }

    #[test]
    fn mutable_reference_arguments_cannot_alias_at_a_call_boundary() {
        let mut tcx = omni_mir::TyCtxt::new();
        let int = tcx.intern(omni_mir::TyKind::Int);
        let mutable_ref =
            tcx.intern(omni_mir::TyKind::Reference { lifetime: None, mutable: true, inner: int });
        let mut locals = IndexVec::new();
        locals.push(LocalDecl { name: Some("ret".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("x".into()), ty: Some(int) });
        locals.push(LocalDecl { name: Some("r".into()), ty: Some(mutable_ref) });
        locals.push(LocalDecl { name: Some("s".into()), ty: Some(mutable_ref) });
        let ret = Local::from_usize(0);
        let x = Local::from_usize(1);
        let r = Local::from_usize(2);
        let s = Local::from_usize(3);
        let integer =
            |value| Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(value))));
        let deref = |local| Place { local, projections: vec![Projection::Deref] };
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![
                Statement::Assign(Place::local(x), integer(1)),
                Statement::Assign(
                    Place::local(r),
                    Rvalue::Reference { place: Place::local(x), mutable: true, ty: mutable_ref },
                ),
                Statement::Assign(
                    Place::local(s),
                    Rvalue::Reference { place: deref(r), mutable: true, ty: mutable_ref },
                ),
                Statement::Assign(Place::local(ret), integer(1)),
            ],
            terminator: Some(Terminator::Call {
                func: Operand::Constant(Constant::FnRef("consume_two".into())),
                args: vec![Operand::Move(Place::local(r)), Operand::Move(Place::local(s))],
                destination: None,
                target: BasicBlock::from_usize(1),
                cleanup: None,
            }),
        });
        blocks.push(omni_mir::ir::BlockData {
            statements: Vec::new(),
            terminator: Some(Terminator::Return),
        });
        let program = MirProgram::new(
            tcx,
            vec![MirFunction {
                name: "mutable_call_alias".into(),
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls: locals, unsafe_blocks: Vec::new() },
            }],
        );
        let error = verify_program(&program)
            .expect_err("a suspended mutable parent and child cannot be passed together");
        assert!(error.message.contains("child reborrow"), "unexpected ownership error: {error:?}");
    }
}
