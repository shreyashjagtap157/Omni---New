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

        for (block, data) in body.blocks.iter_enumerated_mut() {
            if !reachable[block.index()] || !matches!(data.terminator, Some(Terminator::Return)) {
                continue;
            }

            let mut to_drop = out_states[block.index()]
                .iter()
                .copied()
                .filter(|local| *local != return_place)
                .collect::<Vec<_>>();
            to_drop.sort_by_key(|local| std::cmp::Reverse(local.index()));

            let existing = data
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
                    data.statements.push(Statement::Drop(place));
                }
            }
        }
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
        Rvalue::Use(operand)
        | Rvalue::UnaryOp(_, operand)
        | Rvalue::Cast { operand, .. } => transfer_operand(operand, state),
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
        Body {
            blocks: block_vec,
            local_decls: local_vec,
            unsafe_blocks: Vec::new(),
        }
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
                BlockData {
                    statements: vec![],
                    terminator: Some(Terminator::Return),
                },
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
                BlockData {
                    statements: vec![],
                    terminator: Some(Terminator::Return),
                },
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
