use crate::ir::{BasicBlock, Body, Local, Operand, Place, Rvalue, Statement, Terminator};
use std::collections::{HashSet, VecDeque};

pub struct DropElaborator;

impl DropElaborator {
    pub fn elaborate(body: &mut Body, params: &[Local], return_place: Local) {
        if body.blocks.is_empty() { return; }

        let count = body.blocks.len();
        let mut predecessors = vec![Vec::<BasicBlock>::new(); count];
        for (block, data) in body.blocks.iter_enumerated() {
            if let Some(term) = data.terminator.as_ref() {
                for successor in successors(term) {
                    predecessors[successor.index()].push(block);
                }
            }
        }

        let entry = BasicBlock::from(0);
        let mut in_states: Vec<Option<HashSet<Local>>> = vec![None; count];
        let mut out_states: Vec<Option<HashSet<Local>>> = vec![None; count];
        let mut entry_state = HashSet::new();
        entry_state.extend(params.iter().copied());
        in_states[entry.index()] = Some(entry_state);
        let mut worklist = VecDeque::from([entry]);

        while let Some(block) = worklist.pop_front() {
            let Some(in_state) = in_states[block.index()].clone() else { continue; };
            let mut state = in_state;
            for statement in &body.blocks[block].statements {
                transfer_statement(statement, &mut state);
            }
            if let Some(term) = body.blocks[block].terminator.as_ref() {
                transfer_terminator(term, &mut state);
            }

            let changed = out_states[block.index()].as_ref() != Some(&state);
            if !changed { continue; }
            out_states[block.index()] = Some(state);

            let successors_to_update = body.blocks[block]
                .terminator.as_ref().map(successors).unwrap_or_default();
            for successor in successors_to_update {
                if successor == entry { continue; }
                let preds = &predecessors[successor.index()];
                if preds.is_empty() || preds.iter().any(|pred| out_states[pred.index()].is_none()) {
                    continue;
                }
                let mut intersection = out_states[preds[0].index()].as_ref().cloned().unwrap_or_default();
                for pred in &preds[1..] {
                    if let Some(pred_state) = out_states[pred.index()].as_ref() {
                        intersection.retain(|local| pred_state.contains(local));
                    }
                }
                if in_states[successor.index()].as_ref() != Some(&intersection) {
                    in_states[successor.index()] = Some(intersection);
                    worklist.push_back(successor);
                }
            }
        }

        for (block, data) in body.blocks.iter_enumerated_mut() {
            if !matches!(data.terminator, Some(Terminator::Return)) { continue; }
            let Some(state) = out_states[block.index()].as_ref() else { continue; };
            let mut locals: Vec<Local> = state.iter().copied().filter(|l| *l != return_place).collect();
            locals.sort_by_key(|l| std::cmp::Reverse(l.index()));
            let existing: HashSet<Place> = data.statements.iter().filter_map(|s| match s {
                Statement::Drop(place) => Some(place.clone()),
                _ => None,
            }).collect();
            for local in locals {
                let place = Place::local(local);
                if !existing.contains(&place) { data.statements.push(Statement::Drop(place)); }
            }
        }
    }
}

fn successors(term: &Terminator) -> Vec<BasicBlock> {
    match term {
        Terminator::Goto(target) => vec![*target],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut result = targets.iter().map(|(_, block)| *block).collect::<Vec<_>>();
            result.push(*otherwise); result.sort_unstable(); result.dedup(); result
        }
        Terminator::Call { target, cleanup, .. } => {
            let mut result = vec![*target];
            if let Some(cleanup) = cleanup { result.push(*cleanup); }
            result.sort_unstable(); result.dedup(); result
        }
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn transfer_statement(statement: &Statement, state: &mut HashSet<Local>) {
    match statement {
        Statement::Assign(place, rvalue) => {
            transfer_rvalue(rvalue, state);
            if place.is_local() { state.insert(place.local); }
        }
        Statement::Drop(place) => { state.remove(&place.local); }
        Statement::Assume(_) | Statement::BoundsCheck { .. } => {}
    }
}

fn transfer_terminator(term: &Terminator, state: &mut HashSet<Local>) {
    if let Terminator::Call { func, args, destination, .. } = term {
        transfer_operand(func, state);
        for arg in args { transfer_operand(arg, state); }
        if let Some(destination) = destination {
            if destination.is_local() { state.insert(destination.local); }
        }
    }
}

fn transfer_rvalue(rvalue: &Rvalue, state: &mut HashSet<Local>) {
    match rvalue {
        Rvalue::Use(op) | Rvalue::UnaryOp(_, op) | Rvalue::Cast { operand: op, .. } => transfer_operand(op, state),
        Rvalue::BinaryOp(_, lhs, rhs) => { transfer_operand(lhs, state); transfer_operand(rhs, state); }
        Rvalue::Aggregate { operands, .. } | Rvalue::EnumVariant { operands, .. } => { for op in operands { transfer_operand(op, state); } }
        Rvalue::Struct { fields, .. } => { for (_, op) in fields { transfer_operand(op, state); } }
        Rvalue::Reference { .. } => {}
        Rvalue::Range { start, end, .. } => { transfer_operand(start, state); transfer_operand(end, state); }
        Rvalue::Field { base, .. } | Rvalue::Index { base, .. } => transfer_operand(base, state),
    }
}

fn transfer_operand(operand: &Operand, state: &mut HashSet<Local>) {
    if let Operand::Move(place) = operand { state.remove(&place.local); }
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
        for block in blocks { block_vec.push(block); }
        let mut local_vec = IndexVec::new();
        for local in locals { local_vec.push(local); }
        Body { blocks: block_vec, local_decls: local_vec, unsafe_blocks: Vec::new() }
    }

    #[test]
    fn drops_only_definitely_initialized_locals() {
        let ret = Local::from(0);
        let temp = Local::from(1);
        let mut mir = body(vec![BlockData {
            statements: vec![Statement::Assign(Place::local(temp), Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(1)))) )],
            terminator: Some(Terminator::Return),
        }], vec![
            LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
            LocalDecl { name: Some("temp".into()), ty: Some(int_ty()) },
        ]);
        DropElaborator::elaborate(&mut mir, &[], ret);
        assert!(mir.blocks[BasicBlock::from(0)].statements.iter().any(|s| matches!(s, Statement::Drop(p) if p.local == temp)));
        assert!(!mir.blocks[BasicBlock::from(0)].statements.iter().any(|s| matches!(s, Statement::Drop(p) if p.local == ret)));
    }

    #[test]
    fn branch_join_drops_only_values_initialized_on_all_paths() {
        let ret = Local::from(0);
        let temp = Local::from(1);
        let mut mir = body(vec![
            BlockData { statements: vec![], terminator: Some(Terminator::SwitchInt {
                discr: Operand::Constant(Constant::Lit(Lit::Bool(true))), targets: vec![(1, BasicBlock::from(1))], otherwise: BasicBlock::from(2) }), },
            BlockData { statements: vec![Statement::Assign(Place::local(temp), Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(1)))))], terminator: Some(Terminator::Goto(BasicBlock::from(3))) },
            BlockData { statements: vec![], terminator: Some(Terminator::Goto(BasicBlock::from(3))) },
            BlockData { statements: vec![], terminator: Some(Terminator::Return) },
        ], vec![
            LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
            LocalDecl { name: Some("temp".into()), ty: Some(int_ty()) },
        ]);
        DropElaborator::elaborate(&mut mir, &[], ret);
        assert!(!mir.blocks[BasicBlock::from(3)].statements.iter().any(|s| matches!(s, Statement::Drop(p) if p.local == temp)));
    }

    #[test]
    fn move_invalidates_local_before_return() {
        let ret = Local::from(0);
        let temp = Local::from(1);
        let mut mir = body(vec![BlockData {
            statements: vec![
                Statement::Assign(Place::local(temp), Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(7))))),
                Statement::Assign(Place::local(ret), Rvalue::Use(Operand::Move(Place::local(temp)))),
            ],
            terminator: Some(Terminator::Return),
        }], vec![
            LocalDecl { name: Some("_return".into()), ty: Some(int_ty()) },
            LocalDecl { name: Some("temp".into()), ty: Some(int_ty()) },
        ]);
        DropElaborator::elaborate(&mut mir, &[], ret);
        assert!(!mir.blocks[BasicBlock::from(0)].statements.iter().any(|s| matches!(s, Statement::Drop(p) if p.local == temp)));
    }
}