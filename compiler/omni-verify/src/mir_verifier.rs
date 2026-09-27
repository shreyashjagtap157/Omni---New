//! Semantic and structural MIR verification for Omni.
//!
//! MIR carries the exact type arena that produced every Ty handle. Verification
//! checks structural invariants, type consistency, call signatures, and
//! definite assignment without reconstructing another type universe.

use std::collections::{HashSet, VecDeque};

use omni_mir::ast::TypeSpec;
use omni_mir::ir::{
    BasicBlock, BinOp, Constant, Local, MirFunction, MirProgram, Operand, Place, Rvalue, Statement,
    Terminator, UnOp,
};
use omni_types::checker::SubstEnv;
use omni_types::intern::{Ty, TyCtxt, TyKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirVerificationError {
    UndefinedLocal { func: String, local: Local },
    UndefinedBlock { func: String, block: BasicBlock },
    UnterminatedBlock { func: String, block: BasicBlock },
    InvalidReturnPlace { func: String, local: Local },
    InvalidParamLocal { func: String, param_index: usize, local: Local },
    EmptyFunctionBody { func: String },
    DuplicateFunction { func: String },
    UntypedLocal { func: String, local: Local },
    InvalidTypeHandle { func: String, local: Local, ty: Ty },
    TypeMismatch { func: String, context: String, expected: Ty, actual: Ty },
    InvalidUnaryOperand { func: String, op: UnOp, expected: Ty, actual: Ty },
    InvalidCallCallee { func: String },
    UnknownFunction { func: String, callee: String },
    CallArityMismatch { func: String, callee: String, expected: usize, actual: usize },
    CallArgumentTypeMismatch {
        func: String,
        callee: String,
        arg_index: usize,
        expected: Ty,
        actual: Ty,
    },
    CallDestinationRequired { func: String, callee: String },
    CallDestinationUnexpected { func: String, callee: String },
    InvalidSwitchDiscriminant { func: String, ty: Ty },
    UseBeforeAssignment { func: String, block: BasicBlock, local: Local },
    UninitializedReturn { func: String, block: BasicBlock, local: Local },
}

impl std::fmt::Display for MirVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndefinedLocal { func, local } => write!(
                f,
                "MIR Verification Failure in '{}': reference to undefined local {:?}",
                func, local
            ),
            Self::UndefinedBlock { func, block } => write!(
                f,
                "MIR Verification Failure in '{}': reference to undefined block {:?}",
                func, block
            ),
            Self::UnterminatedBlock { func, block } => write!(
                f,
                "MIR Verification Failure in '{}': block {:?} lacks a terminator",
                func, block
            ),
            Self::InvalidReturnPlace { func, local } => write!(
                f,
                "MIR Verification Failure in '{}': invalid or untyped return place {:?}",
                func, local
            ),
            Self::InvalidParamLocal { func, param_index, local } => write!(
                f,
                "MIR Verification Failure in '{}': parameter {} maps to invalid or untyped local {:?}",
                func, param_index, local
            ),
            Self::EmptyFunctionBody { func } => write!(
                f,
                "MIR Verification Failure in '{}': function contains zero basic blocks",
                func
            ),
            Self::DuplicateFunction { func } => {
                write!(f, "MIR Verification Failure: duplicate function '{}'", func)
            }
            Self::UntypedLocal { func, local } => write!(
                f,
                "MIR Verification Failure in '{}': local {:?} has no type",
                func, local
            ),
            Self::InvalidTypeHandle { func, local, ty } => write!(
                f,
                "MIR Verification Failure in '{}': local {:?} contains invalid type handle {:?}",
                func, local, ty
            ),
            Self::TypeMismatch { func, context, expected, actual } => write!(
                f,
                "MIR Verification Failure in '{}': {} (expected {:?}, found {:?})",
                func, context, expected, actual
            ),
            Self::InvalidUnaryOperand { func, op, expected, actual } => write!(
                f,
                "MIR Verification Failure in '{}': unary {:?} requires type {:?}, found {:?}",
                func, op, expected, actual
            ),
            Self::InvalidCallCallee { func } => write!(
                f,
                "MIR Verification Failure in '{}': call callee is not a direct function reference",
                func
            ),
            Self::UnknownFunction { func, callee } => write!(
                f,
                "MIR Verification Failure in '{}': call targets unknown function '{}'",
                func, callee
            ),
            Self::CallArityMismatch { func, callee, expected, actual } => write!(
                f,
                "MIR Verification Failure in '{}': call '{}' expects {} arguments, found {}",
                func, callee, expected, actual
            ),
            Self::CallArgumentTypeMismatch { func, callee, arg_index, expected, actual } => write!(
                f,
                "MIR Verification Failure in '{}': call '{}' argument {} has type {:?}, expected {:?}",
                func, callee, arg_index, actual, expected
            ),
            Self::CallDestinationRequired { func, callee } => write!(
                f,
                "MIR Verification Failure in '{}': value-returning call '{}' has no destination",
                func, callee
            ),
            Self::CallDestinationUnexpected { func, callee } => write!(
                f,
                "MIR Verification Failure in '{}': Unit-returning call '{}' has a destination",
                func, callee
            ),
            Self::InvalidSwitchDiscriminant { func, ty } => write!(
                f,
                "MIR Verification Failure in '{}': switch discriminant has unsupported type {:?}",
                func, ty
            ),
            Self::UseBeforeAssignment { func, block, local } => write!(
                f,
                "MIR Verification Failure in '{}': local {:?} is used before assignment in block {:?}",
                func, local, block
            ),
            Self::UninitializedReturn { func, block, local } => write!(
                f,
                "MIR Verification Failure in '{}': return local {:?} is not definitely assigned in block {:?}",
                func, local, block
            ),
        }
    }
}

pub struct MirVerifier;

impl MirVerifier {
    pub fn verify_program(prog: &MirProgram) -> Result<(), MirVerificationError> {
        let mut names = HashSet::new();
        for func in &prog.functions {
            if !names.insert(func.name.as_str()) {
                return Err(MirVerificationError::DuplicateFunction { func: func.name.clone() });
            }
        }
        for func in &prog.functions {
            Self::verify_function(prog, func)?;
        }
        Ok(())
    }

    pub fn verify_function(
        prog: &MirProgram,
        func: &MirFunction,
    ) -> Result<(), MirVerificationError> {
        let fn_name = &func.name;
        let num_blocks = func.body.blocks.len();
        let num_locals = func.body.local_decls.len();

        if num_blocks == 0 {
            return Err(MirVerificationError::EmptyFunctionBody { func: fn_name.clone() });
        }

        for (idx, decl) in func.body.local_decls.iter().enumerate() {
            let local = Local::from_usize(idx);
            let ty = decl.ty.ok_or_else(|| MirVerificationError::UntypedLocal {
                func: fn_name.clone(),
                local,
            })?;
            if !prog.tcx.contains(ty) {
                return Err(MirVerificationError::InvalidTypeHandle {
                    func: fn_name.clone(),
                    local,
                    ty,
                });
            }
        }

        if func.return_place.index() >= num_locals {
            return Err(MirVerificationError::InvalidReturnPlace {
                func: fn_name.clone(),
                local: func.return_place,
            });
        }

        let actual_return = Self::local_ty(func, func.return_place, fn_name)?;
        let expected_return = Self::spec_ty(&prog.tcx, &func.return_type);
        if actual_return != expected_return {
            return Err(MirVerificationError::TypeMismatch {
                func: fn_name.clone(),
                context: "return place type differs from function return type".to_string(),
                expected: expected_return,
                actual: actual_return,
            });
        }

        for (index, &param_local) in func.params.iter().enumerate() {
            if param_local.index() >= num_locals {
                return Err(MirVerificationError::InvalidParamLocal {
                    func: fn_name.clone(),
                    param_index: index,
                    local: param_local,
                });
            }
            Self::local_ty(func, param_local, fn_name)?;
        }

        let mut predecessors = vec![Vec::<BasicBlock>::new(); num_blocks];

        for (b_idx, block) in func.body.blocks.iter().enumerate() {
            let bb = BasicBlock::from_usize(b_idx);

            for stmt in &block.statements {
                match stmt {
                    Statement::Assign(place, rvalue) => {
                        Self::check_place(fn_name, place, num_locals)?;
                        Self::check_rvalue_types(prog, func, rvalue, *place, fn_name)?;
                    }
                    Statement::Assume(_) => {}
                    Statement::Drop(place) => Self::check_place(fn_name, place, num_locals)?,
                }
            }

            let term = block.terminator.as_ref().ok_or_else(|| {
                MirVerificationError::UnterminatedBlock { func: fn_name.clone(), block: bb }
            })?;

            match term {
                Terminator::Goto(target) => {
                    Self::check_block(fn_name, *target, num_blocks)?;
                    predecessors[target.index()].push(bb);
                }
                Terminator::SwitchInt { discr, targets, otherwise } => {
                    Self::check_operand_structure(prog, func, discr)?;
                    for (_, target) in targets {
                        Self::check_block(fn_name, *target, num_blocks)?;
                        predecessors[target.index()].push(bb);
                    }
                    Self::check_block(fn_name, *otherwise, num_blocks)?;
                    predecessors[otherwise.index()].push(bb);
                }
                Terminator::Call {
                    func: callee_op,
                    args,
                    destination,
                    target,
                    cleanup,
                } => {
                    Self::check_operand_structure(prog, func, callee_op)?;
                    for arg in args {
                        Self::check_operand_structure(prog, func, arg)?;
                    }
                    if let Some(place) = destination {
                        Self::check_place(fn_name, place, num_locals)?;
                    }
                    Self::check_block(fn_name, *target, num_blocks)?;
                    predecessors[target.index()].push(bb);
                    if let Some(cleanup_block) = cleanup {
                        Self::check_block(fn_name, *cleanup_block, num_blocks)?;
                        predecessors[cleanup_block.index()].push(bb);
                    }
                }
                Terminator::Return | Terminator::Unreachable => {}
            }
        }

        Self::verify_call_signatures(prog, func)?;
        Self::verify_definite_assignment(prog, func, &predecessors, actual_return)?;

        Ok(())
    }

    fn verify_call_signatures(
        prog: &MirProgram,
        func: &MirFunction,
    ) -> Result<(), MirVerificationError> {
        for block in func.body.blocks.iter() {
            let Some(Terminator::Call {
                func: callee_op,
                args,
                destination,
                ..
            }) = block.terminator.as_ref()
            else {
                continue;
            };

            let callee_name = match callee_op {
                Operand::Constant(Constant::FnRef(name)) => name,
                _ => {
                    return Err(MirVerificationError::InvalidCallCallee {
                        func: func.name.clone(),
                    })
                }
            };

            let callee = prog
                .functions
                .iter()
                .find(|candidate| candidate.name == *callee_name)
                .ok_or_else(|| MirVerificationError::UnknownFunction {
                    func: func.name.clone(),
                    callee: callee_name.clone(),
                })?;

            if args.len() != callee.params.len() {
                return Err(MirVerificationError::CallArityMismatch {
                    func: func.name.clone(),
                    callee: callee_name.clone(),
                    expected: callee.params.len(),
                    actual: args.len(),
                });
            }

            let mut tcx = prog.tcx.clone();
            for (index, (arg, &param_local)) in args.iter().zip(callee.params.iter()).enumerate() {
                let arg_ty = Self::operand_ty(&mut tcx, func, arg)?;
                let param_ty = Self::local_ty(callee, param_local, callee_name)?;
                if arg_ty != param_ty {
                    return Err(MirVerificationError::CallArgumentTypeMismatch {
                        func: func.name.clone(),
                        callee: callee_name.clone(),
                        arg_index: index,
                        expected: param_ty,
                        actual: arg_ty,
                    });
                }
            }

            let callee_return_ty = Self::local_ty(callee, callee.return_place, callee_name)?;
            let unit_ty = Self::kind_ty(&mut tcx, TyKind::Unit);

            match (callee_return_ty == unit_ty, destination) {
                (true, None) => {}
                (true, Some(_)) => {
                    return Err(MirVerificationError::CallDestinationUnexpected {
                        func: func.name.clone(),
                        callee: callee_name.clone(),
                    })
                }
                (false, None) => {
                    return Err(MirVerificationError::CallDestinationRequired {
                        func: func.name.clone(),
                        callee: callee_name.clone(),
                    })
                }
                (false, Some(place)) => {
                    let destination_ty = Self::local_ty(func, place.local, &func.name)?;
                    if destination_ty != callee_return_ty {
                        return Err(MirVerificationError::TypeMismatch {
                            func: func.name.clone(),
                            context: format!("call '{}' destination type", callee_name),
                            expected: callee_return_ty,
                            actual: destination_ty,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn verify_definite_assignment(
        prog: &MirProgram,
        func: &MirFunction,
        predecessors: &[Vec<BasicBlock>],
        return_ty: Ty,
    ) -> Result<(), MirVerificationError> {
        let num_blocks = func.body.blocks.len();
        let num_locals = func.body.local_decls.len();
        let entry = BasicBlock::from_usize(0);
        let reachable = Self::reachable_blocks(func, num_blocks)?;

        let all_locals: HashSet<Local> = (0..num_locals).map(Local::from_usize).collect();
        let mut in_sets = vec![all_locals.clone(); num_blocks];
        let mut out_sets = vec![all_locals.clone(); num_blocks];

        let mut entry_assigned = HashSet::new();
        for &param in &func.params {
            entry_assigned.insert(param);
        }
        in_sets[entry.index()] = entry_assigned.clone();
        out_sets[entry.index()] = Self::transfer_block(func, entry, &entry_assigned);

        loop {
            let mut changed = false;
            for idx in 0..num_blocks {
                let block = BasicBlock::from_usize(idx);
                if !reachable[idx] || block == entry {
                    continue;
                }

                let reachable_preds: Vec<_> = predecessors[idx]
                    .iter()
                    .copied()
                    .filter(|pred| reachable[pred.index()])
                    .collect();

                if reachable_preds.is_empty() {
                    continue;
                }

                let mut new_in = all_locals.clone();
                for pred in reachable_preds {
                    new_in.retain(|local| out_sets[pred.index()].contains(local));
                }
                let new_out = Self::transfer_block(func, block, &new_in);

                if new_in != in_sets[idx] {
                    in_sets[idx] = new_in;
                    changed = true;
                }
                if new_out != out_sets[idx] {
                    out_sets[idx] = new_out;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut tcx = prog.tcx.clone();
        let unit_ty = Self::kind_ty(&mut tcx, TyKind::Unit);

        for (idx, block_data) in func.body.blocks.iter().enumerate() {
            if !reachable[idx] {
                continue;
            }

            let block = BasicBlock::from_usize(idx);
            let mut assigned = in_sets[idx].clone();

            for stmt in &block_data.statements {
                match stmt {
                    Statement::Assign(place, rvalue) => {
                        Self::check_rvalue_initialized(func, block, rvalue, &assigned)?;
                        assigned.insert(place.local);
                    }
                    Statement::Assume(_) => {}
                    Statement::Drop(place) => {
                        Self::require_assigned(func, block, place.local, &assigned)?;
                        assigned.remove(&place.local);
                    }
                }
            }

            match block_data.terminator.as_ref().expect("structural terminator validation already completed") {
                Terminator::SwitchInt { discr, .. } => {
                    Self::check_operand_initialized(func, block, discr, &assigned)?;
                }
                Terminator::Call { args, destination, .. } => {
                    for arg in args {
                        Self::check_operand_initialized(func, block, arg, &assigned)?;
                    }
                    if let Some(place) = destination {
                        let _ = place;
                    }
                }
                Terminator::Return => {
                    if return_ty != unit_ty && !assigned.contains(&func.return_place) {
                        return Err(MirVerificationError::UninitializedReturn {
                            func: func.name.clone(),
                            block,
                            local: func.return_place,
                        });
                    }
                }
                Terminator::Goto(_) | Terminator::Unreachable => {}
            }
        }

        Ok(())
    }

    fn transfer_block(
        func: &MirFunction,
        block: BasicBlock,
        initial: &HashSet<Local>,
    ) -> HashSet<Local> {
        let mut assigned = initial.clone();
        for stmt in &func.body.blocks[block].statements {
            match stmt {
                Statement::Assign(place, _) => {
                    assigned.insert(place.local);
                }
                Statement::Drop(place) => {
                    assigned.remove(&place.local);
                }
                Statement::Assume(_) => {}
            }
        }
        if let Some(Terminator::Call { destination: Some(place), .. }) =
            func.body.blocks[block].terminator.as_ref()
        {
            assigned.insert(place.local);
        }
        assigned
    }

    fn reachable_blocks(
        func: &MirFunction,
        num_blocks: usize,
    ) -> Result<Vec<bool>, MirVerificationError> {
        let mut reachable = vec![false; num_blocks];
        let mut queue = VecDeque::from([BasicBlock::from_usize(0)]);

        while let Some(block) = queue.pop_front() {
            if reachable[block.index()] {
                continue;
            }
            reachable[block.index()] = true;

            let term = func.body.blocks[block]
                .terminator
                .as_ref()
                .expect("structural terminator validation already completed");

            match term {
                Terminator::Goto(target) => queue.push_back(*target),
                Terminator::SwitchInt { targets, otherwise, .. } => {
                    for (_, target) in targets {
                        queue.push_back(*target);
                    }
                    queue.push_back(*otherwise);
                }
                Terminator::Call { target, cleanup, .. } => {
                    queue.push_back(*target);
                    if let Some(cleanup) = cleanup {
                        queue.push_back(*cleanup);
                    }
                }
                Terminator::Return | Terminator::Unreachable => {}
            }
        }

        Ok(reachable)
    }

    fn check_rvalue_types(
        prog: &MirProgram,
        func: &MirFunction,
        rvalue: &Rvalue,
        destination: Place,
        fn_name: &str,
    ) -> Result<(), MirVerificationError> {
        let mut tcx = prog.tcx.clone();
        let rvalue_ty = Self::rvalue_ty(&mut tcx, func, rvalue, fn_name)?;
        let destination_ty = Self::local_ty(func, destination.local, fn_name)?;

        if rvalue_ty != destination_ty {
            return Err(MirVerificationError::TypeMismatch {
                func: fn_name.to_string(),
                context: format!("assignment to local {:?}", destination.local),
                expected: destination_ty,
                actual: rvalue_ty,
            });
        }
        Ok(())
    }

    fn rvalue_ty(
        tcx: &mut TyCtxt,
        func: &MirFunction,
        rvalue: &Rvalue,
        fn_name: &str,
    ) -> Result<Ty, MirVerificationError> {
        match rvalue {
            Rvalue::Use(op) => Self::operand_ty(tcx, func, op),
            Rvalue::BinaryOp(op, lhs, rhs) => {
                let lhs_ty = Self::operand_ty(tcx, func, lhs)?;
                let rhs_ty = Self::operand_ty(tcx, func, rhs)?;
                if lhs_ty != rhs_ty {
                    return Err(MirVerificationError::TypeMismatch {
                        func: fn_name.to_string(),
                        context: "binary operands have different types".to_string(),
                        expected: lhs_ty,
                        actual: rhs_ty,
                    });
                }

                match op {
                    BinOp::Eq
                    | BinOp::Ne
                    | BinOp::Lt
                    | BinOp::Gt
                    | BinOp::Le
                    | BinOp::Ge => Ok(Self::kind_ty(tcx, TyKind::Bool)),
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => Ok(lhs_ty),
                }
            }
            Rvalue::UnaryOp(op, operand) => {
                let actual = Self::operand_ty(tcx, func, operand)?;
                let expected = match op {
                    UnOp::Neg => Self::kind_ty(tcx, TyKind::Int),
                    UnOp::Not => Self::kind_ty(tcx, TyKind::Bool),
                };
                if actual != expected {
                    return Err(MirVerificationError::InvalidUnaryOperand {
                        func: fn_name.to_string(),
                        op: *op,
                        expected,
                        actual,
                    });
                }
                Ok(expected)
            }
        }
    }

    fn operand_ty(
        tcx: &mut TyCtxt,
        func: &MirFunction,
        operand: &Operand,
    ) -> Result<Ty, MirVerificationError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                Self::local_ty(func, place.local, &func.name)
            }
            Operand::Constant(Constant::Lit(lit)) => Ok(Self::literal_ty(tcx, lit)),
            Operand::Constant(Constant::FnRef(_)) => Err(MirVerificationError::InvalidCallCallee {
                func: func.name.clone(),
            }),
        }
    }

    fn literal_ty(tcx: &mut TyCtxt, lit: &omni_mir::ast::Lit) -> Ty {
        match lit {
            omni_mir::ast::Lit::Int(_) => Self::kind_ty(tcx, TyKind::Int),
            omni_mir::ast::Lit::Float(_) => Self::kind_ty(tcx, TyKind::Float),
            omni_mir::ast::Lit::Bool(_) => Self::kind_ty(tcx, TyKind::Bool),
            omni_mir::ast::Lit::Char(_) => Self::kind_ty(tcx, TyKind::Char),
            omni_mir::ast::Lit::Byte(_) => Self::kind_ty(tcx, TyKind::Byte),
            omni_mir::ast::Lit::String(_) => Self::kind_ty(tcx, TyKind::String),
        }
    }

    fn kind_ty(tcx: &mut TyCtxt, kind: TyKind) -> Ty {
        tcx.intern(kind)
    }

    fn spec_ty(tcx: &TyCtxt, spec: &TypeSpec) -> Ty {
        let mut clone = tcx.clone();
        clone.lower_type_spec(spec, &SubstEnv::new())
    }

    fn local_ty(
        func: &MirFunction,
        local: Local,
        fn_name: &str,
    ) -> Result<Ty, MirVerificationError> {
        if local.index() >= func.body.local_decls.len() {
            return Err(MirVerificationError::UndefinedLocal {
                func: fn_name.to_string(),
                local,
            });
        }
        func.body.local_decls[local]
            .ty
            .ok_or_else(|| MirVerificationError::UntypedLocal {
                func: fn_name.to_string(),
                local,
            })
    }

    fn check_place(
        func: &str,
        place: &Place,
        num_locals: usize,
    ) -> Result<(), MirVerificationError> {
        if place.local.index() >= num_locals {
            Err(MirVerificationError::UndefinedLocal {
                func: func.to_string(),
                local: place.local,
            })
        } else {
            Ok(())
        }
    }

    fn check_operand_structure(
        _prog: &MirProgram,
        func: &MirFunction,
        op: &Operand,
    ) -> Result<(), MirVerificationError> {
        match op {
            Operand::Copy(place) | Operand::Move(place) => {
                Self::check_place(&func.name, place, func.body.local_decls.len())
            }
            Operand::Constant(Constant::Lit(_)) | Operand::Constant(Constant::FnRef(_)) => Ok(()),
        }
    }

    fn check_block(
        func: &str,
        block: BasicBlock,
        num_blocks: usize,
    ) -> Result<(), MirVerificationError> {
        if block.index() >= num_blocks {
            Err(MirVerificationError::UndefinedBlock {
                func: func.to_string(),
                block,
            })
        } else {
            Ok(())
        }
    }

    fn check_rvalue_initialized(
        func: &MirFunction,
        block: BasicBlock,
        rvalue: &Rvalue,
        assigned: &HashSet<Local>,
    ) -> Result<(), MirVerificationError> {
        match rvalue {
            Rvalue::Use(op) => Self::check_operand_initialized(func, block, op, assigned),
            Rvalue::BinaryOp(_, lhs, rhs) => {
                Self::check_operand_initialized(func, block, lhs, assigned)?;
                Self::check_operand_initialized(func, block, rhs, assigned)
            }
            Rvalue::UnaryOp(_, operand) => {
                Self::check_operand_initialized(func, block, operand, assigned)
            }
        }
    }

    fn check_operand_initialized(
        func: &MirFunction,
        block: BasicBlock,
        operand: &Operand,
        assigned: &HashSet<Local>,
    ) -> Result<(), MirVerificationError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                Self::require_assigned(func, block, place.local, assigned)
            }
            Operand::Constant(_) => Ok(()),
        }
    }

    fn require_assigned(
        func: &MirFunction,
        block: BasicBlock,
        local: Local,
        assigned: &HashSet<Local>,
    ) -> Result<(), MirVerificationError> {
        if assigned.contains(&local) {
            Ok(())
        } else {
            Err(MirVerificationError::UseBeforeAssignment {
                func: func.name.clone(),
                block,
                local,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use index_vec::IndexVec;

    fn context() -> (TyCtxt, Ty, Ty) {
        let mut tcx = TyCtxt::new();
        let int = tcx.intern(TyKind::Int);
        let bool_ty = tcx.intern(TyKind::Bool);
        (tcx, int, bool_ty)
    }

    fn identity_program() -> MirProgram {
        let (tcx, int, _) = context();
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let param = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("x".to_string()),
            ty: Some(int),
        });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::Use(Operand::Copy(Place { local: param })),
            )],
            terminator: Some(Terminator::Return),
        });
        MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "identity".to_string(),
                params: vec![param],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls },
            }],
        }
    }

    #[test]
    fn verifier_accepts_valid_identity() {
        assert!(MirVerifier::verify_program(&identity_program()).is_ok());
    }

    #[test]
    fn verifier_rejects_undefined_local() {
        let mut prog = identity_program();
        let invalid = Local::from_usize(99);
        prog.functions[0].body.blocks[0].statements[0] = Statement::Assign(
            Place { local: prog.functions[0].return_place },
            Rvalue::Use(Operand::Copy(Place { local: invalid })),
        );
        let result = MirVerifier::verify_program(&prog);
        assert!(matches!(result, Err(MirVerificationError::UndefinedLocal { .. })));
    }

    #[test]
    fn verifier_rejects_undefined_block() {
        let mut prog = identity_program();
        prog.functions[0].body.blocks[0].terminator =
            Some(Terminator::Goto(BasicBlock::from_usize(99)));
        let result = MirVerifier::verify_program(&prog);
        assert!(matches!(result, Err(MirVerificationError::UndefinedBlock { .. })));
    }

    #[test]
    fn verifier_rejects_assignment_type_mismatch() {
        let (tcx, int, _) = context();
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Bool(true)))),
            )],
            terminator: Some(Terminator::Return),
        });
        let prog = MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "bad_assign".to_string(),
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn verifier_rejects_use_before_assignment() {
        let (tcx, int, _) = context();
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let tmp = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("tmp".to_string()),
            ty: Some(int),
        });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::Use(Operand::Copy(Place { local: tmp })),
            )],
            terminator: Some(Terminator::Return),
        });
        let prog = MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "use_before_init".to_string(),
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::UseBeforeAssignment { .. })
        ));
    }

    #[test]
    fn verifier_rejects_invalid_unary_type() {
        let (tcx, int, _) = context();
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::UnaryOp(
                    UnOp::Not,
                    Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(1))),
                ),
            )],
            terminator: Some(Terminator::Return),
        });
        let prog = MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "bad_not".to_string(),
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::InvalidUnaryOperand { .. })
        ));
    }

    #[test]
    fn verifier_rejects_uninitialized_return() {
        let (tcx, int, _) = context();
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(Terminator::Return),
        });
        let prog = MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "bad_return".to_string(),
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::UninitializedReturn { .. })
        ));
    }

    #[test]
    fn verifier_rejects_bad_call_arity_and_destination() {
        let (tcx, int, _) = context();
        let mut caller_locals = IndexVec::new();
        let caller_ret = caller_locals.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let mut callee_locals = IndexVec::new();
        let callee_ret = callee_locals.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let callee_param = callee_locals.push(omni_mir::ir::LocalDecl {
            name: Some("x".to_string()),
            ty: Some(int),
        });

        let mut caller_blocks = IndexVec::new();
        caller_blocks.push(omni_mir::ir::BlockData {
            statements: vec![],
            terminator: Some(Terminator::Call {
                func: Operand::Constant(Constant::FnRef("inc".to_string())),
                args: vec![],
                destination: None,
                target: BasicBlock::from_usize(1),
                cleanup: None,
            }),
        });
        caller_blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place { local: caller_ret },
                Rvalue::Use(Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(0)))),
            )],
            terminator: Some(Terminator::Return),
        });

        let mut callee_blocks = IndexVec::new();
        callee_blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place { local: callee_ret },
                Rvalue::Use(Operand::Copy(Place { local: callee_param })),
            )],
            terminator: Some(Terminator::Return),
        });

        let prog = MirProgram {
            tcx,
            functions: vec![
                MirFunction {
                    name: "main".to_string(),
                    params: vec![],
                    return_place: caller_ret,
                    return_type: TypeSpec::Int,
                    body: omni_mir::ir::Body {
                        blocks: caller_blocks,
                        local_decls: caller_locals,
                    },
                },
                MirFunction {
                    name: "inc".to_string(),
                    params: vec![callee_param],
                    return_place: callee_ret,
                    return_type: TypeSpec::Int,
                    body: omni_mir::ir::Body {
                        blocks: callee_blocks,
                        local_decls: callee_locals,
                    },
                },
            ],
        };

        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::CallArityMismatch { .. })
        ));
    }

    #[test]
    fn verifier_rejects_comparison_result_with_non_bool_destination() {
        let (tcx, int, _) = context();
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(omni_mir::ir::LocalDecl {
            name: Some("_return".to_string()),
            ty: Some(int),
        });
        let mut blocks = IndexVec::new();
        blocks.push(omni_mir::ir::BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::BinaryOp(
                    BinOp::Eq,
                    Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(1))),
                    Operand::Constant(Constant::Lit(omni_mir::ast::Lit::Int(2))),
                ),
            )],
            terminator: Some(Terminator::Return),
        });
        let prog = MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "bad_cmp".to_string(),
                params: vec![],
                return_place: ret,
                return_type: TypeSpec::Int,
                body: omni_mir::ir::Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::TypeMismatch { .. })
        ));
    }
}
