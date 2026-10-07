use crate::ir::{BasicBlock, Body, Local, Operand, Place, Rvalue, Statement, Terminator};
use std::collections::{HashSet, VecDeque};

/// Elaborates semantic drops from definite-initialization information.
pub struct DropElaborator;

impl DropElaborator {
    /// Computes definite initialization across the reachable MIR CFG and injects
    /// whole-local drops for values that are definitely initialized on every path
    /// to each return. Parameters enter initialized; the result local is excluded.
    ///
    /// The current MIR initialization lattice is whole-local. A projected move
    /// therefore conservatively invalidates the root local instead of modeling
    /// partial aggregate initialization.
    pub fn elaborate(body: &mut Body, params: &[Local], return_place: Local) {
        if body.blocks.is_empty() {
            return;
        }

        let count = body.blocks.len();
        let entry = BasicBlock::from(0);

        let mut successors_cache: Vec<Vec<BasicBlock>> = vec![Vec::new(); count];
        for (block, data) in body.blocks.iter_enumerated() {
            if let Some(term) = data.terminator.as_ref() {
                successors_cache[block.index()] = successors(term);
            }
        }

        // Restrict the dataflow to reachable blocks. Every reachable non-entry
        // block starts at the lattice bottom (no definitely-initialized locals).
        let mut reachable = vec![false; count];
        let mut reach_worklist = VecDeque::from([entry]);
        while let Some(block) = reach_worklist.pop_front() {
            if reachable[block.index()] {
                continue;
            }
            reachable[block.index()] = true;
            for successor in &successors_cache[block.index()] {
                reach_worklist.push_back(*successor);
            }
        }

        let mut predecessors: Vec<Vec<BasicBlock>> = vec![Vec::new(); count];
        for block in reachable
            .iter()
            .enumerate()
            .filter_map(|(idx, is_reachable)| is_reachable.then_some(BasicBlock::from(idx)))
        {
            for successor in &successors_cache[block.index()] {
                predecessors[successor.index()].push(block);
            }
        }

        let mut in_states: Vec<HashSet<Local>> = vec![HashSet::new(); count];
        let mut out_states: Vec<HashSet<Local>> = vec![HashSet::new(); count];

        in_states[entry.index()].extend(params.iter().copied());

        // All reachable blocks are processed at least once. Re-processing happens
        // only when their input state changes, which reaches a fixed point because
        // the transfer/meet functions are monotone over the finite local lattice.
        let initial_blocks = reachable
            .iter()
            .enumerate()
            .filter_map(|(idx, is_reachable)| is_reachable.then_some(BasicBlock::from(idx)))
            .collect::<Vec<_>>();
        let mut worklist = VecDeque::from(initial_blocks);

        while let Some(block) = worklist.pop_front() {
            let mut state = in_states[block.index()].clone();
            for statement in &body.blocks[block].statements {
                transfer_statement(statement, &mut state);
            }
            if let Some(term) = body.blocks[block].terminator.as_ref() {
                transfer_terminator(term, &mut state);
            }

            if state == out_states[block.index()] {
                continue;
            }

            out_states[block.index()] = state;

            for successor in &successors_cache[block.index()] {
                if !reachable[successor.index()] {
                    continue;
                }

                let preds = &predecessors[successor.index()];
                let mut new_in = if *successor == entry {
                    in_states[successor.index()].clone()
                } else {
                    // Missing predecessor information is conservatively treated
                    // as bottom during fixed-point convergence.
                    let mut meet = HashSet::new();
                    if let Some(first) = preds.first() {
                        meet = out_states[first.index()].clone();
                        for pred in &preds[1..] {
                            meet.retain(|local| out_states[pred.index()].contains(local));
                        }
                    }
                    meet
                };

                if *successor == entry {
                    let mut entry_values = HashSet::new();
                    entry_values.extend(params.iter().copied());
                    new_in = entry_values;
                }

                if new_in != in_states[successor.index()] {
                    in_states[successor.index()] = new_in;
                    worklist.push_back(*successor);
                }
            }
        }

        for idx in 0..count {
            let block = BasicBlock::from_usize(idx);
            if !reachable[idx] || !matches!(body.blocks[block].terminator, Some(Terminator::Return))
            {
                continue;
            }

            let mut to_drop = linear_initialization_order(body, block, &predecessors, params)
                .unwrap_or_else(|| {
                    out_states[idx]
                        .iter()
                        .copied()
                        .filter(|local| *local != return_place)
                        .collect::<Vec<_>>()
                })
                .into_iter()
                .filter(|local| out_states[idx].contains(local) && *local != return_place)
                .rev()
                .collect::<Vec<_>>();

            let existing = body.blocks[block]
                .statements
                .iter()
                .filter_map(|statement| match statement {
                    Statement::Drop(place) => Some(place.clone()),
                    _ => None,
                })
                .collect::<HashSet<_>>();

            for local in to_drop {
                let place = Place::local(local);
                if !existing.contains(&place) {
                    body.blocks[block].statements.push(Statement::Drop(place));
                }
            }
        }
    }
}

fn linear_initialization_order(
    body: &Body,
    return_block: BasicBlock,
    predecessors: &[Vec<BasicBlock>],
    params: &[Local],
) -> Option<Vec<Local>> {
    let entry = BasicBlock::from(0);
    let mut chain = Vec::new();
    let mut current = return_block;
    let mut seen = HashSet::new();

    loop {
        if !seen.insert(current) {
            return None;
        }
        chain.push(current);
        if current == entry {
            break;
        }
        let preds = &predecessors[current.index()];
        if preds.len() != 1 {
            return None;
        }
        current = preds[0];
    }

    chain.reverse();
    let mut order = params.to_vec();
    for block in chain {
        for statement in &body.blocks[block].statements {
            transfer_order_statement(statement, &mut order);
        }
        if let Some(term) = body.blocks[block].terminator.as_ref() {
            transfer_order_terminator(term, &mut order);
        }
    }
    Some(order)
}

fn transfer_order_statement(statement: &Statement, order: &mut Vec<Local>) {
    match statement {
        Statement::Assign(place, rvalue) => {
            transfer_order_rvalue(rvalue, order);
            if place.is_local() {
                order.retain(|local| *local != place.local);
                order.push(place.local);
            }
        }
        Statement::Drop(place) => {
            order.retain(|local| *local != place.local);
        }
        Statement::Assume(_) | Statement::BoundsCheck { .. } => {}
    }
}

fn transfer_order_terminator(term: &Terminator, order: &mut Vec<Local>) {
    if let Terminator::Call { func, args, destination, .. } = term {
        transfer_order_operand(func, order);
        for arg in args {
            transfer_order_operand(arg, order);
        }
        if let Some(destination) = destination.filter(|place| place.is_local()) {
            order.retain(|local| *local != destination.local);
            order.push(destination.local);
        }
    }
}

fn transfer_order_rvalue(rvalue: &Rvalue, order: &mut Vec<Local>) {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) | Rvalue::Cast { operand, .. } => {
            transfer_order_operand(operand, order);
        }
        Rvalue::BinaryOp(_, lhs, rhs) => {
            transfer_order_operand(lhs, order);
            transfer_order_operand(rhs, order);
        }
        Rvalue::Aggregate { operands, .. } | Rvalue::EnumVariant { operands, .. } => {
            for operand in operands {
                transfer_order_operand(operand, order);
            }
        }
        Rvalue::Struct { fields, .. } => {
            for (_, operand) in fields {
                transfer_order_operand(operand, order);
            }
        }
        Rvalue::Reference { .. } => {}
        Rvalue::Range { start, end, .. } => {
            transfer_order_operand(start, order);
            transfer_order_operand(end, order);
        }
        Rvalue::Field { base, .. } => transfer_order_operand(base, order),
        Rvalue::Index { base, index, .. } => {
            transfer_order_operand(base, order);
            transfer_order_operand(index, order);
        }
    }
}

fn transfer_order_operand(operand: &Operand, order: &mut Vec<Local>) {
    if let Operand::Move(place) = operand {
        order.retain(|local| *local != place.local);
    }
}

fn successors(term: &Terminator) -> Vec<BasicBlock> {
    match term {
        Terminator::Goto(target) => vec![*target],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut result = targets.iter().map(|(_, block)| *block).collect::<Vec<_>>();
            result.push(*otherwise);
            result.sort_unstable();
            result.dedup();
            result
        }
        Terminator::Call { target, cleanup, .. } => {
            let mut result = vec![*target];
            if let Some(cleanup) = cleanup {
                result.push(*cleanup);
            }
            result.sort_unstable();
            result.dedup();
            result
        }
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn transfer_statement(statement: &Statement, state: &mut HashSet<Local>) {
    match statement {
        Statement::Assign(place, rvalue) => {
            transfer_rvalue(rvalue, state);
            if place.is_local() {
                state.insert(place.local);
            }
        }
        Statement::Drop(place) => {
            state.remove(&place.local);
        }
        Statement::Assume(_) | Statement::BoundsCheck { .. } => {}
    }
}

fn transfer_terminator(term: &Terminator, state: &mut HashSet<Local>) {
    if let Terminator::Call { func, args, destination, .. } = term {
        transfer_operand(func, state);
        for arg in args {
            transfer_operand(arg, state);
        }
        if let Some(destination) = destination {
            if destination.is_local() {
                state.insert(destination.local);
            }
        }
    }
}

fn transfer_rvalue(rvalue: &Rvalue, state: &mut HashSet<Local>) {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) | Rvalue::Cast { operand, .. } => {
            transfer_operand(operand, state)
        }
        Rvalue::BinaryOp(_, lhs, rhs) => {
            transfer_operand(lhs, state);
            transfer_operand(rhs, state);
        }
        Rvalue::Aggregate { operands, .. } | Rvalue::EnumVariant { operands, .. } => {
            for operand in operands {
                transfer_operand(operand, state);
            }
        }
        Rvalue::Struct { fields, .. } => {
            for (_, operand) in fields {
                transfer_operand(operand, state);
            }
        }
        Rvalue::Reference { .. } => {}
        Rvalue::Range { start, end, .. } => {
            transfer_operand(start, state);
            transfer_operand(end, state);
        }
        Rvalue::Field { base, .. } => transfer_operand(base, state),
        Rvalue::Index { base, index, .. } => {
            transfer_operand(base, state);
            transfer_operand(index, state);
        }
    }
}

fn transfer_operand(operand: &Operand, state: &mut HashSet<Local>) {
    if let Operand::Move(place) = operand {
        state.remove(&place.local);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BlockData, Constant, LocalDecl};
    use index_vec::IndexVec;
    use omni_types::ast::Lit;
    use omni_types::intern::{TyCtxt, TyKind};

    fn int_ty() -> omni_types::Ty {
        let mut tcx = TyCtxt::new();
        tcx.intern(TyKind::Int)
    }

    fn body(blocks: Vec<BlockData>, locals: Vec<LocalDecl>) -> Body {
        let mut block_vec = IndexVec::new();
        for block in blocks {
            block_vec.push(block);
        }
        let mut local_vec = IndexVec::new();
        for local in locals {
            local_vec.push(local);
        }
        Body { blocks: block_vec, local_decls: local_vec, unsafe_blocks: Vec::new() }
    }

    #[test]
    fn drops_only_definitely_initialized_locals() {
        let ret = Local::from(0);
        let temp = Local::from(1);
        let mut mir = body(
            vec![BlockData {
                statements: vec![Statement::Assign(
                    Place::local(temp),
                    Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(1)))),
                )],
                terminator: Some(Terminator::Return),
            }],
            vec![
                LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
                LocalDecl { name: Some("temp".into()), ty: Some(int_ty()) },
            ],
        );
        DropElaborator::elaborate(&mut mir, &[], ret);
        assert!(mir.blocks[BasicBlock::from(0)]
            .statements
            .iter()
            .any(|s| matches!(s, Statement::Drop(p) if p.local == temp)));
        assert!(!mir.blocks[BasicBlock::from(0)]
            .statements
            .iter()
            .any(|s| matches!(s, Statement::Drop(p) if p.local == ret)));
    }

    #[test]
    fn drops_follow_successful_initialization_order_on_linear_path() {
        let ret = Local::from(0);
        let first = Local::from(1);
        let second = Local::from(2);
        let mut mir = body(
            vec![BlockData {
                statements: vec![
                    Statement::Assign(
                        Place::local(second),
                        Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(2)))),
                    ),
                    Statement::Assign(
                        Place::local(first),
                        Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(1)))),
                    ),
                ],
                terminator: Some(Terminator::Return),
            }],
            vec![
                LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
                LocalDecl { name: Some("first".into()), ty: Some(int_ty()) },
                LocalDecl { name: Some("second".into()), ty: Some(int_ty()) },
            ],
        );

        DropElaborator::elaborate(&mut mir, &[], ret);
        let drops = mir.blocks[BasicBlock::from(0)]
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::Drop(place) => Some(place.local),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(drops, vec![first, second]);
    }

    #[test]
    fn branch_join_drops_only_values_initialized_on_all_paths() {
        let ret = Local::from(0);
        let temp = Local::from(1);
        let mut mir = body(
            vec![
                BlockData {
                    statements: vec![],
                    terminator: Some(Terminator::SwitchInt {
                        discr: Operand::Constant(Constant::Lit(Lit::Bool(true))),
                        targets: vec![(1, BasicBlock::from(1))],
                        otherwise: BasicBlock::from(2),
                    }),
                },
                BlockData {
                    statements: vec![Statement::Assign(
                        Place::local(temp),
                        Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(1)))),
                    )],
                    terminator: Some(Terminator::Goto(BasicBlock::from(3))),
                },
                BlockData {
                    statements: vec![],
                    terminator: Some(Terminator::Goto(BasicBlock::from(3))),
                },
                BlockData { statements: vec![], terminator: Some(Terminator::Return) },
            ],
            vec![
                LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
                LocalDecl { name: Some("temp".into()), ty: Some(int_ty()) },
            ],
        );

        DropElaborator::elaborate(&mut mir, &[], ret);
        assert!(!mir.blocks[BasicBlock::from(3)]
            .statements
            .iter()
            .any(|s| matches!(s, Statement::Drop(p) if p.local == temp)));
    }

    #[test]
    fn move_invalidates_local_before_return() {
        let ret = Local::from(0);
        let temp = Local::from(1);
        let mut mir = body(
            vec![BlockData {
                statements: vec![
                    Statement::Assign(
                        Place::local(temp),
                        Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(7)))),
                    ),
                    Statement::Assign(
                        Place::local(ret),
                        Rvalue::Use(Operand::Move(Place::local(temp))),
                    ),
                ],
                terminator: Some(Terminator::Return),
            }],
            vec![
                LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
                LocalDecl { name: Some("temp".into()), ty: Some(int_ty()) },
            ],
        );

        DropElaborator::elaborate(&mut mir, &[], ret);
        assert!(!mir.blocks[BasicBlock::from(0)]
            .statements
            .iter()
            .any(|s| matches!(s, Statement::Drop(p) if p.local == temp)));
    }

    #[test]
    fn loop_header_converges_and_preserves_definite_initialization() {
        let ret = Local::from(0);
        let param = Local::from(1);
        let mut mir = body(
            vec![
                BlockData {
                    statements: vec![],
                    terminator: Some(Terminator::Goto(BasicBlock::from(1))),
                },
                BlockData {
                    statements: vec![Statement::Assign(
                        Place::local(param),
                        Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(2)))),
                    )],
                    terminator: Some(Terminator::SwitchInt {
                        discr: Operand::Constant(Constant::Lit(Lit::Bool(false))),
                        targets: vec![(1, BasicBlock::from(1))],
                        otherwise: BasicBlock::from(2),
                    }),
                },
                BlockData { statements: vec![], terminator: Some(Terminator::Return) },
            ],
            vec![
                LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
                LocalDecl { name: Some("x".into()), ty: Some(int_ty()) },
            ],
        );
        DropElaborator::elaborate(&mut mir, &[param], ret);
        assert!(mir.blocks[BasicBlock::from(2)]
            .statements
            .iter()
            .any(|s| matches!(s, Statement::Drop(p) if p.local == param)));
    }
}
