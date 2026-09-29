//! Structural MIR Verification Engine for Omni (`omni-verify`).
//! Validates structural and control-flow graph invariants before native code generation:
//! 1. All referenced Local places/operands exist in function `local_decls`.
//! 2. All target BasicBlock handles in terminators exist in function `blocks`.
//! 3. All basic blocks terminate in a valid Terminator (`Return`, `Goto`, `SwitchInt`, `Call`, `Unreachable`).
//! 4. Parameter and return local indices strictly match `MirFunction` signature parameters.

use std::collections::{HashSet, VecDeque};

use omni_mir::ir::{
    BasicBlock, BinOp, Constant, Local, MirFunction, MirProgram, Operand, Place, Rvalue, Statement,
    Terminator, UnOp,
};
use omni_mir::{Ty, TyCtxt, TyKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirVerificationError {
    UndefinedLocal {
        func: String,
        local: Local,
    },
    UndefinedBlock {
        func: String,
        block: BasicBlock,
    },
    UnterminatedBlock {
        func: String,
        block: BasicBlock,
    },
    InvalidReturnPlace {
        func: String,
        expected: Local,
        actual: Local,
    },
    InvalidParamLocal {
        func: String,
        param_index: usize,
        local: Local,
    },
    DuplicateParamLocal {
        func: String,
        param_index: usize,
        local: Local,
    },
    ParamAliasesReturnPlace {
        func: String,
        param_index: usize,
        local: Local,
    },
    EmptyFunctionBody {
        func: String,
    },
    DuplicateFunction {
        func: String,
    },
    UntypedLocal {
        func: String,
        local: Local,
    },
    InvalidTypeHandle {
        func: String,
        local: Local,
        ty: Ty,
    },
    TypeMismatch {
        func: String,
        context: String,
        expected: Ty,
        actual: Ty,
    },
    InvalidUnaryOperand {
        func: String,
        op: UnOp,
        expected: Ty,
        actual: Ty,
    },
    InvalidCallCallee {
        func: String,
    },
    UnknownFunction {
        func: String,
        callee: String,
    },
    CallArityMismatch {
        func: String,
        callee: String,
        expected: usize,
        actual: usize,
    },
    CallArgumentTypeMismatch {
        func: String,
        callee: String,
        arg_index: usize,
        expected: Ty,
        actual: Ty,
    },
    CallDestinationRequired {
        func: String,
        callee: String,
    },
    CallDestinationUnexpected {
        func: String,
        callee: String,
    },
    InvalidTypeSpec {
        func: String,
        context: String,
    },
    AggregateTypeMismatch {
        func: String,
        context: String,
    },
    UseBeforeAssignment {
        func: String,
        block: BasicBlock,
        local: Local,
    },
    UninitializedReturn {
        func: String,
        block: BasicBlock,
        local: Local,
    },
}

impl std::fmt::Display for MirVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndefinedLocal { func, local } => {
                write!(f, "MIR Verification Failure in '{}': Place/Operand references undefined local {:?}", func, local)
            }
            Self::UndefinedBlock { func, block } => {
                write!(f, "MIR Verification Failure in '{}': Terminator targets undefined BasicBlock {:?}", func, block)
            }
            Self::UnterminatedBlock { func, block } => {
                write!(
                    f,
                    "MIR Verification Failure in '{}': BasicBlock {:?} lacks a valid terminator",
                    func, block
                )
            }
            Self::InvalidReturnPlace { func, expected, actual } => {
                write!(f, "MIR Verification Failure in '{}': Return place mismatch (expected {:?}, got {:?})", func, expected, actual)
            }
            Self::InvalidParamLocal { func, param_index, local } => {
                write!(
                    f,
                    "MIR Verification Failure in '{}': Parameter {} maps to undefined local {:?}",
                    func, param_index, local
                )
            }
            Self::DuplicateParamLocal { func, param_index, local } => {
                write!(
                    f,
                    "MIR Verification Failure in '{}': Parameter {} duplicates parameter local {:?}",
                    func, param_index, local
                )
            }
            Self::ParamAliasesReturnPlace { func, param_index, local } => {
                write!(
                    f,
                    "MIR Verification Failure in '{}': Parameter {} aliases return place {:?}",
                    func, param_index, local
                )
            }
            Self::EmptyFunctionBody { func } => {
                write!(
                    f,
                    "MIR Verification Failure in '{}': Function contains zero basic blocks",
                    func
                )
            }
            Self::DuplicateFunction { func } => {
                write!(f, "MIR Verification Failure: duplicate function '{}'", func)
            }
            Self::UntypedLocal { func, local } => {
                write!(f, "MIR Verification Failure in '{}': local {:?} is untyped", func, local)
            }
            Self::InvalidTypeHandle { func, local, ty } => write!(
                f,
                "MIR Verification Failure in '{}': local {:?} has invalid type handle {:?}",
                func, local, ty
            ),
            Self::TypeMismatch { func, context, expected, actual } => write!(
                f,
                "MIR Verification Failure in '{}': {} (expected {:?}, found {:?})",
                func, context, expected, actual
            ),
            Self::InvalidUnaryOperand { func, op, expected, actual } => write!(
                f,
                "MIR Verification Failure in '{}': unary {:?} expects {:?}, found {:?}",
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
            Self::CallArgumentTypeMismatch {
                func,
                callee,
                arg_index,
                expected,
                actual,
            } => write!(
                f,
                "MIR Verification Failure in '{}': call '{}' argument {} has type {:?}, expected {:?}",
                func, callee, arg_index, actual, expected
            ),
            Self::CallDestinationRequired { func, callee } => write!(
                f,
                "MIR Verification Failure in '{}': value-returning call '{}' requires a destination",
                func, callee
            ),
            Self::CallDestinationUnexpected { func, callee } => write!(
                f,
                "MIR Verification Failure in '{}': Unit-returning call '{}' must not have a destination",
                func, callee
            ),
            Self::InvalidTypeSpec { func, context } => write!(
                f,
                "MIR Verification Failure in '{}': invalid or non-concrete type specification: {}",
                func, context
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
        let mut function_names = HashSet::new();
        for func in &prog.functions {
            if !function_names.insert(func.name.as_str()) {
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

        if func.body.blocks.is_empty() {
            return Err(MirVerificationError::EmptyFunctionBody { func: fn_name.clone() });
        }

        for (index, local_decl) in func.body.local_decls.iter().enumerate() {
            let local = Local::from_usize(index);
            let ty = local_decl.ty.ok_or_else(|| MirVerificationError::UntypedLocal {
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

        // Validate return place index exists and is typed
        if func.return_place.index() >= func.body.local_decls.len() {
            return Err(MirVerificationError::InvalidReturnPlace {
                func: fn_name.clone(),
                expected: func.return_place,
                actual: func.return_place,
            });
        }
        if func.body.local_decls[func.return_place].ty.is_none() {
            return Err(MirVerificationError::InvalidReturnPlace {
                func: fn_name.clone(),
                expected: func.return_place,
                actual: func.return_place,
            });
        }
        let actual_return = func.body.local_decls[func.return_place].ty.ok_or_else(|| {
            MirVerificationError::InvalidReturnPlace {
                func: fn_name.clone(),
                expected: func.return_place,
                actual: func.return_place,
            }
        })?;
        let expected_return = Self::spec_type(prog, &func.return_type, fn_name)?;
        if actual_return != expected_return {
            return Err(MirVerificationError::TypeMismatch {
                func: fn_name.clone(),
                context: "return place type differs from function return type".to_string(),
                expected: expected_return,
                actual: actual_return,
            });
        }

        // Validate parameter-local topology: every parameter must be a unique,
        // typed local distinct from the function's dedicated return place.
        let mut parameter_locals = HashSet::new();
        for (idx, &param_local) in func.params.iter().enumerate() {
            if param_local.index() >= func.body.local_decls.len()
                || func.body.local_decls[param_local].ty.is_none()
            {
                return Err(MirVerificationError::InvalidParamLocal {
                    func: fn_name.clone(),
                    param_index: idx,
                    local: param_local,
                });
            }
            if param_local == func.return_place {
                return Err(MirVerificationError::ParamAliasesReturnPlace {
                    func: fn_name.clone(),
                    param_index: idx,
                    local: param_local,
                });
            }
            if !parameter_locals.insert(param_local) {
                return Err(MirVerificationError::DuplicateParamLocal {
                    func: fn_name.clone(),
                    param_index: idx,
                    local: param_local,
                });
            }
        }

        let num_blocks = func.body.blocks.len();
        let num_locals = func.body.local_decls.len();

        for (b_idx, block) in func.body.blocks.iter().enumerate() {
            let bb_handle = BasicBlock::from_usize(b_idx);

            // Statements validation
            for stmt in &block.statements {
                match stmt {
                    Statement::Assign(place, rval) => {
                        Self::check_place(fn_name, place, num_locals)?;
                        match rval {
                            Rvalue::Use(op) => Self::check_operand(fn_name, op, num_locals)?,
                            Rvalue::BinaryOp(_, op1, op2) => {
                                Self::check_operand(fn_name, op1, num_locals)?;
                                Self::check_operand(fn_name, op2, num_locals)?;
                            }
                            Rvalue::UnaryOp(_, op) => Self::check_operand(fn_name, op, num_locals)?,
                            Rvalue::Cast { operand, .. } => Self::check_operand(fn_name, operand, num_locals)?,
                            Rvalue::Aggregate { operands, .. } => {
                                for operand in operands {
                                    Self::check_operand(fn_name, operand, num_locals)?;
                                }
                            }

                        }
                        Self::check_rvalue_type(prog, func, place, rval)?;
                    }
                    Statement::Assume(_) => {}
                    Statement::Drop(place) => Self::check_place(fn_name, place, num_locals)?,
                }
            }

            // Terminator validation
            let term = block.terminator.as_ref().ok_or_else(|| {
                MirVerificationError::UnterminatedBlock { func: fn_name.clone(), block: bb_handle }
            })?;

            match term {
                Terminator::Goto(target) => Self::check_block(fn_name, *target, num_blocks)?,
                Terminator::SwitchInt { discr, targets, otherwise } => {
                    Self::check_operand(fn_name, discr, num_locals)?;
                    for (_, t_block) in targets {
                        Self::check_block(fn_name, *t_block, num_blocks)?;
                    }
                    Self::check_block(fn_name, *otherwise, num_blocks)?;
                }
                Terminator::Call { func: f_op, args, destination, target, cleanup } => {
                    Self::check_operand(fn_name, f_op, num_locals)?;
                    for arg in args {
                        Self::check_operand(fn_name, arg, num_locals)?;
                    }
                    if let Some(destination) = destination {
                        Self::check_place(fn_name, destination, num_locals)?;
                    }
                    Self::check_block(fn_name, *target, num_blocks)?;
                    if let Some(c_block) = cleanup {
                        Self::check_block(fn_name, *c_block, num_blocks)?;
                    }
                }
                Terminator::Return => {}
                Terminator::Unreachable => {}
            }
        }

        Self::check_calls(prog, func)?;
        Self::check_definite_assignment(prog, func)?;
        Ok(())
    }

    fn check_definite_assignment(
        prog: &MirProgram,
        func: &MirFunction,
    ) -> Result<(), MirVerificationError> {
        let num_blocks = func.body.blocks.len();
        if num_blocks == 0 {
            return Ok(());
        }

        let reachable = Self::reachable_blocks(func, num_blocks);
        let mut predecessors = vec![Vec::<(BasicBlock, Option<Local>)>::new(); num_blocks];

        for (idx, block) in func.body.blocks.iter().enumerate() {
            if !reachable[idx] {
                continue;
            }
            let from = BasicBlock::from_usize(idx);
            match block.terminator.as_ref().expect("structural terminator already verified") {
                Terminator::Goto(target) => predecessors[target.index()].push((from, None)),
                Terminator::SwitchInt { targets, otherwise, .. } => {
                    for (_, target) in targets {
                        predecessors[target.index()].push((from, None));
                    }
                    predecessors[otherwise.index()].push((from, None));
                }
                Terminator::Call { target, cleanup, destination, .. } => {
                    predecessors[target.index()].push((from, destination.map(|place| place.local)));
                    if let Some(cleanup) = cleanup {
                        predecessors[cleanup.index()].push((from, None));
                    }
                }
                Terminator::Return | Terminator::Unreachable => {}
            }
        }

        let all_locals =
            (0..func.body.local_decls.len()).map(Local::from_usize).collect::<HashSet<Local>>();
        let mut in_sets = vec![all_locals.clone(); num_blocks];
        let mut out_sets = vec![all_locals.clone(); num_blocks];
        let entry = BasicBlock::from_usize(0);
        let mut entry_assigned = HashSet::new();
        entry_assigned.extend(func.params.iter().copied());
        in_sets[entry.index()] = entry_assigned.clone();
        out_sets[entry.index()] = Self::transfer_block(func, entry, &entry_assigned);

        loop {
            let mut changed = false;
            for idx in 0..num_blocks {
                let block = BasicBlock::from_usize(idx);
                if !reachable[idx] || block == entry {
                    continue;
                }
                let preds = &predecessors[idx];
                if preds.is_empty() {
                    continue;
                }

                let mut new_in = all_locals.clone();
                for (pred, edge_assignment) in preds {
                    let mut pred_out = out_sets[pred.index()].clone();
                    if let Some(local) = edge_assignment {
                        pred_out.insert(*local);
                    }
                    new_in.retain(|local| pred_out.contains(local));
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
        let unit = tcx.intern(TyKind::Unit);

        for (idx, block_data) in func.body.blocks.iter().enumerate() {
            if !reachable[idx] {
                continue;
            }
            let block = BasicBlock::from_usize(idx);
            let mut assigned = in_sets[idx].clone();

            for statement in &block_data.statements {
                match statement {
                    Statement::Assign(_, rvalue) => {
                        Self::check_rvalue_initialized(func, block, rvalue, &assigned)?;
                    }
                    Statement::Assume(_) => {}
                    Statement::Drop(place) => {
                        Self::require_assigned(func, block, place.local, &assigned)?;
                    }
                }
                if let Statement::Assign(place, _) = statement {
                    assigned.insert(place.local);
                } else if let Statement::Drop(place) = statement {
                    assigned.remove(&place.local);
                }
            }

            match block_data.terminator.as_ref().expect("structural terminator already verified") {
                Terminator::SwitchInt { discr, .. } => {
                    Self::check_operand_initialized(func, block, discr, &assigned)?;
                }
                Terminator::Call { args, .. } => {
                    for arg in args {
                        Self::check_operand_initialized(func, block, arg, &assigned)?;
                    }
                }
                Terminator::Return => {
                    let return_ty = Self::local_ty(func, func.return_place, &func.name)?;
                    if return_ty != unit && !assigned.contains(&func.return_place) {
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
        for statement in &func.body.blocks[block].statements {
            match statement {
                Statement::Assign(place, _) => {
                    assigned.insert(place.local);
                }
                Statement::Drop(place) => {
                    assigned.remove(&place.local);
                }
                Statement::Assume(_) => {}
            }
        }
        assigned
    }

    fn reachable_blocks(func: &MirFunction, num_blocks: usize) -> Vec<bool> {
        let mut reachable = vec![false; num_blocks];
        let mut queue = VecDeque::from([BasicBlock::from_usize(0)]);

        while let Some(block) = queue.pop_front() {
            if reachable[block.index()] {
                continue;
            }
            reachable[block.index()] = true;
            match func.body.blocks[block]
                .terminator
                .as_ref()
                .expect("structural terminator already verified")
            {
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
        reachable
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
            Rvalue::Cast { operand, .. } => {
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
            Err(MirVerificationError::UseBeforeAssignment { func: func.name.clone(), block, local })
        }
    }

    fn check_rvalue_type(
        prog: &MirProgram,
        func: &MirFunction,
        destination: &Place,
        rvalue: &Rvalue,
    ) -> Result<(), MirVerificationError> {
        let mut tcx = prog.tcx.clone();
        let actual = Self::rvalue_type(&mut tcx, func, rvalue)?;
        let expected = Self::local_ty(func, destination.local, &func.name)?;
        if actual != expected {
            return Err(MirVerificationError::TypeMismatch {
                func: func.name.clone(),
                context: format!("assignment to local {:?}", destination.local),
                expected,
                actual,
            });
        }
        Ok(())
    }

    fn rvalue_type(
        tcx: &mut TyCtxt,
        func: &MirFunction,
        rvalue: &Rvalue,
    ) -> Result<Ty, MirVerificationError> {
        match rvalue {
            Rvalue::Use(op) => Self::operand_type(tcx, func, op),
            Rvalue::BinaryOp(op, lhs, rhs) => {
                let lhs_ty = Self::operand_type(tcx, func, lhs)?;
                let rhs_ty = Self::operand_type(tcx, func, rhs)?;
                if lhs_ty != rhs_ty {
                    return Err(MirVerificationError::TypeMismatch {
                        func: func.name.clone(),
                        context: "binary operands have incompatible types".to_string(),
                        expected: lhs_ty,
                        actual: rhs_ty,
                    });
                }
                let is_int = matches!(tcx.get(lhs_ty), TyKind::Int);
                let is_float = matches!(tcx.get(lhs_ty), TyKind::Float);
                let is_ordered_scalar =
                    matches!(tcx.get(lhs_ty), TyKind::Int | TyKind::Byte | TyKind::Char | TyKind::Float);
                let is_equality_scalar =
                    matches!(tcx.get(lhs_ty), TyKind::Int | TyKind::Byte | TyKind::Char | TyKind::Bool | TyKind::Float);
                match op {
                    BinOp::Eq | BinOp::Ne => {
                        if !is_equality_scalar {
                            return Err(MirVerificationError::TypeMismatch {
                                func: func.name.clone(),
                                context: format!("operator {:?} requires comparable scalar operands", op),
                                expected: tcx.intern(TyKind::Int),
                                actual: lhs_ty,
                            });
                        }
                        Ok(tcx.intern(TyKind::Bool))
                    }
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        if !is_ordered_scalar {
                            return Err(MirVerificationError::TypeMismatch {
                                func: func.name.clone(),
                                context: format!("operator {:?} requires ordered scalar operands", op),
                                expected: tcx.intern(TyKind::Int),
                                actual: lhs_ty,
                            });
                        }
                        Ok(tcx.intern(TyKind::Bool))
                    }
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        if !is_int && !is_float {
                            return Err(MirVerificationError::TypeMismatch {
                                func: func.name.clone(),
                                context: format!("operator {:?} requires Int or Float operands", op),
                                expected: tcx.intern(TyKind::Int),
                                actual: lhs_ty,
                            });
                        }
                        Ok(lhs_ty)
                    }
                    BinOp::Rem | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                        if !is_int {
                            return Err(MirVerificationError::TypeMismatch {
                                func: func.name.clone(),
                                context: format!("operator {:?} requires Int operands", op),
                                expected: tcx.intern(TyKind::Int),
                                actual: lhs_ty,
                            });
                        }
                        Ok(lhs_ty)
                    }
                }
            }
            Rvalue::Aggregate { kind, operands, ty } => {
                let actual_types = operands
                    .iter()
                    .map(|operand| Self::operand_type(tcx, func, operand))
                    .collect::<Result<Vec<_>, _>>()?;
                match (kind, tcx.get(*ty)) {
                    (AggregateKind::Tuple, TyKind::Tuple(expected)) => {
                        if actual_types != *expected {
                            return Err(MirVerificationError::AggregateTypeMismatch {
                                func: func.name.clone(),
                                context: "tuple aggregate element types do not match declared type".to_string(),
                            });
                        }
                    }
                    (AggregateKind::Array, TyKind::Array(expected, len)) => {
                        if actual_types.len() != *len || actual_types.iter().any(|actual| *actual != *expected) {
                            return Err(MirVerificationError::AggregateTypeMismatch {
                                func: func.name.clone(),
                                context: "array aggregate length or element type does not match declared type".to_string(),
                            });
                        }
                    }
                    _ => {
                        return Err(MirVerificationError::AggregateTypeMismatch {
                            func: func.name.clone(),
                            context: "aggregate kind does not match declared MIR type".to_string(),
                        });
                    }
                }
                Ok(*ty)
            }
            Rvalue::Cast { operand, from, to } => {
                let actual = Self::operand_type(tcx, func, operand)?;
                if actual != *from {
                    return Err(MirVerificationError::TypeMismatch {
                        func: func.name.clone(),
                        context: "cast source type does not match its declared source".to_string(),
                        expected: *from,
                        actual,
                    });
                }
                if !tcx.contains(*to) {
                    return Err(MirVerificationError::InvalidTypeSpec {
                        func: func.name.clone(),
                        context: format!("cast target type handle {:?} is not present", to),
                    });
                }
                Ok(*to)
            }
            Rvalue::UnaryOp(op, operand) => {
                let actual = Self::operand_type(tcx, func, operand)?;
                let valid = match op {
                    UnOp::Neg => actual == tcx.intern(TyKind::Int) || actual == tcx.intern(TyKind::Float),
                    UnOp::BitNot => actual == tcx.intern(TyKind::Int),
                    UnOp::Not => actual == tcx.intern(TyKind::Bool),
                };
                let expected = match op {
                    UnOp::Neg => actual,
                    UnOp::BitNot => tcx.intern(TyKind::Int),
                    UnOp::Not => tcx.intern(TyKind::Bool),
                };
                if !valid {
                    return Err(MirVerificationError::InvalidUnaryOperand {
                        func: func.name.clone(),
                        op: *op,
                        expected,
                        actual,
                    });
                }
                Ok(expected)
            }
        }
    }

    fn operand_type(
        tcx: &mut TyCtxt,
        func: &MirFunction,
        operand: &Operand,
    ) -> Result<Ty, MirVerificationError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                Self::local_ty(func, place.local, &func.name)
            }
            Operand::Constant(Constant::Lit(lit)) => Ok(Self::literal_type(tcx, lit)),
            Operand::Constant(Constant::FnRef(_)) => {
                Err(MirVerificationError::InvalidCallCallee { func: func.name.clone() })
            }
        }
    }

    fn literal_type(tcx: &mut TyCtxt, lit: &omni_mir::ast::Lit) -> Ty {
        match lit {
            omni_mir::ast::Lit::Int(_) => tcx.intern(TyKind::Int),
            omni_mir::ast::Lit::Float(_) => tcx.intern(TyKind::Float),
            omni_mir::ast::Lit::Bool(_) => tcx.intern(TyKind::Bool),
            omni_mir::ast::Lit::Char(_) => tcx.intern(TyKind::Char),
            omni_mir::ast::Lit::Byte(_) => tcx.intern(TyKind::Byte),
            omni_mir::ast::Lit::String(_) => tcx.intern(TyKind::String),
        }
    }

    fn local_ty(
        func: &MirFunction,
        local: Local,
        fn_name: &str,
    ) -> Result<Ty, MirVerificationError> {
        if local.index() >= func.body.local_decls.len() {
            return Err(MirVerificationError::UndefinedLocal { func: fn_name.to_string(), local });
        }
        func.body.local_decls[local].ty.ok_or_else(|| MirVerificationError::InvalidParamLocal {
            func: fn_name.to_string(),
            param_index: usize::MAX,
            local,
        })
    }

    fn spec_type(
        prog: &MirProgram,
        spec: &omni_mir::ast::TypeSpec,
        func: &str,
    ) -> Result<Ty, MirVerificationError> {
        fn lower(tcx: &mut TyCtxt, spec: &omni_mir::ast::TypeSpec) -> Option<Ty> {
            use omni_mir::ast::TypeSpec;
            match spec {
                TypeSpec::Int => Some(tcx.intern(TyKind::Int)),
                TypeSpec::Float => Some(tcx.intern(TyKind::Float)),
                TypeSpec::Bool => Some(tcx.intern(TyKind::Bool)),
                TypeSpec::Char => Some(tcx.intern(TyKind::Char)),
                TypeSpec::Byte => Some(tcx.intern(TyKind::Byte)),
                TypeSpec::String => Some(tcx.intern(TyKind::String)),
                TypeSpec::Unit => Some(tcx.intern(TyKind::Unit)),
                TypeSpec::Never => Some(tcx.intern(TyKind::Never)),
                TypeSpec::Known(ty) if tcx.contains(*ty) => Some(*ty),
                TypeSpec::Known(_) | TypeSpec::GenericParam(_) => None,
                TypeSpec::Tuple(items) => {
                    let items =
                        items.iter().map(|item| lower(tcx, item)).collect::<Option<Vec<_>>>()?;
                    Some(tcx.intern(TyKind::Tuple(items)))
                }
                TypeSpec::Array(elem, len) => {
                    let elem = lower(tcx, elem)?;
                    Some(tcx.intern(TyKind::Array(elem, *len)))
                }
                TypeSpec::Range(elem) => {
                    let elem = lower(tcx, elem)?;
                    Some(tcx.intern(TyKind::Range(elem)))
                }
                TypeSpec::Reference { lifetime, mutable, inner } => {
                    let inner = lower(tcx, inner)?;
                    Some(tcx.intern(TyKind::Reference {
                        lifetime: lifetime.clone(),
                        mutable: *mutable,
                        inner,
                    }))
                }
                TypeSpec::Fn(params, ret) => {
                    let params =
                        params.iter().map(|param| lower(tcx, param)).collect::<Option<Vec<_>>>()?;
                    let ret = lower(tcx, ret)?;
                    Some(tcx.intern(TyKind::Fn(params, ret)))
                }
                TypeSpec::Struct(name, args) => {
                    let args =
                        args.iter().map(|arg| lower(tcx, arg)).collect::<Option<Vec<_>>>()?;
                    Some(tcx.intern(TyKind::Struct(name.clone(), args)))
                }
                TypeSpec::Enum(name, args) => {
                    let args =
                        args.iter().map(|arg| lower(tcx, arg)).collect::<Option<Vec<_>>>()?;
                    Some(tcx.intern(TyKind::Enum(name.clone(), args)))
                }
            }
        }

        let mut tcx = prog.tcx.clone();
        lower(&mut tcx, spec).ok_or_else(|| MirVerificationError::InvalidTypeSpec {
            func: func.to_string(),
            context: format!("{spec:?}"),
        })
    }

    fn check_calls(prog: &MirProgram, func: &MirFunction) -> Result<(), MirVerificationError> {
        for block in func.body.blocks.iter() {
            let Some(Terminator::Call { func: callee, args, destination, .. }) =
                block.terminator.as_ref()
            else {
                continue;
            };

            let callee_name = match callee {
                Operand::Constant(Constant::FnRef(name)) => name,
                _ => {
                    return Err(MirVerificationError::InvalidCallCallee { func: func.name.clone() })
                }
            };

            let target =
                prog.functions.iter().find(|candidate| candidate.name == *callee_name).ok_or_else(
                    || MirVerificationError::UnknownFunction {
                        func: func.name.clone(),
                        callee: callee_name.clone(),
                    },
                )?;

            if args.len() != target.params.len() {
                return Err(MirVerificationError::CallArityMismatch {
                    func: func.name.clone(),
                    callee: callee_name.clone(),
                    expected: target.params.len(),
                    actual: args.len(),
                });
            }

            let mut tcx = prog.tcx.clone();
            for (arg_index, (arg, param)) in args.iter().zip(&target.params).enumerate() {
                let actual = Self::operand_type(&mut tcx, func, arg)?;
                let expected = Self::local_ty(target, *param, &target.name)?;
                if actual != expected {
                    return Err(MirVerificationError::CallArgumentTypeMismatch {
                        func: func.name.clone(),
                        callee: callee_name.clone(),
                        arg_index,
                        expected,
                        actual,
                    });
                }
            }

            let return_ty = Self::local_ty(target, target.return_place, &target.name)?;
            let unit_ty = tcx.intern(TyKind::Unit);
            match (return_ty == unit_ty, destination) {
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
                    if destination_ty != return_ty {
                        return Err(MirVerificationError::TypeMismatch {
                            func: func.name.clone(),
                            context: format!("call '{}' destination type", callee_name),
                            expected: return_ty,
                            actual: destination_ty,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn check_place(
        func: &str,
        place: &Place,
        num_locals: usize,
    ) -> Result<(), MirVerificationError> {
        if place.local.index() >= num_locals {
            Err(MirVerificationError::UndefinedLocal { func: func.to_string(), local: place.local })
        } else {
            Ok(())
        }
    }

    fn check_operand(
        func: &str,
        op: &Operand,
        num_locals: usize,
    ) -> Result<(), MirVerificationError> {
        match op {
            Operand::Copy(p) | Operand::Move(p) => Self::check_place(func, p, num_locals),
            Operand::Constant(_) => Ok(()),
        }
    }

    fn check_block(
        func: &str,
        block: BasicBlock,
        num_blocks: usize,
    ) -> Result<(), MirVerificationError> {
        if block.index() >= num_blocks {
            Err(MirVerificationError::UndefinedBlock { func: func.to_string(), block })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use index_vec::IndexVec;
    use omni_mir::ir::{
        BlockData, Body, LocalDecl, MirFunction, MirProgram, Place, Rvalue, Statement, Terminator,
    };

    fn int_context() -> TyCtxt {
        let mut tcx = TyCtxt::new();
        tcx.intern(TyKind::Error);
        tcx.intern(TyKind::Int);
        tcx
    }

    #[test]
    fn test_verifier_passes_valid_mir() {
        let dummy_ty = omni_mir::ast::TypeSpec::Int;
        let mut local_decls = IndexVec::new();
        let ret_l = local_decls
            .push(LocalDecl { name: Some("_return".to_string()), ty: Some(omni_mir::Ty(1)) });
        let param_l =
            local_decls.push(LocalDecl { name: Some("x".to_string()), ty: Some(omni_mir::Ty(1)) });

        let mut blocks = IndexVec::new();
        blocks.push(BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret_l },
                Rvalue::Use(Operand::Copy(Place { local: param_l })),
            )],
            terminator: Some(Terminator::Return),
        });

        let prog = MirProgram {
            tcx: int_context(),
            functions: vec![MirFunction {
                name: "identity".to_string(),
                params: vec![param_l],
                return_place: ret_l,
                return_type: dummy_ty,
                body: Body { blocks, local_decls },
            }],
        };

        assert!(MirVerifier::verify_program(&prog).is_ok());
    }

    #[test]
    fn test_verifier_rejects_duplicate_parameter_local() {
        let mut local_decls = IndexVec::new();
        let ret_l = local_decls
            .push(LocalDecl { name: Some("_return".to_string()), ty: Some(omni_mir::Ty(1)) });
        let param_l =
            local_decls.push(LocalDecl { name: Some("x".to_string()), ty: Some(omni_mir::Ty(1)) });

        let mut blocks = IndexVec::new();
        blocks.push(BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret_l },
                Rvalue::Use(Operand::Copy(Place { local: param_l })),
            )],
            terminator: Some(Terminator::Return),
        });

        let prog = MirProgram {
            tcx: int_context(),
            functions: vec![MirFunction {
                name: "duplicate_params".to_string(),
                params: vec![param_l, param_l],
                return_place: ret_l,
                return_type: omni_mir::ast::TypeSpec::Int,
                body: Body { blocks, local_decls },
            }],
        };

        let res = MirVerifier::verify_program(&prog);
        assert!(matches!(res, Err(MirVerificationError::DuplicateParamLocal { .. })));
    }

    #[test]
    fn test_verifier_rejects_parameter_aliasing_return_place() {
        let mut unit_tcx = TyCtxt::new();
        let unit = unit_tcx.intern(TyKind::Unit);

        let mut local_decls = IndexVec::new();
        let shared =
            local_decls.push(LocalDecl { name: Some("shared".to_string()), ty: Some(unit) });

        let mut blocks = IndexVec::new();
        blocks.push(BlockData { statements: vec![], terminator: Some(Terminator::Return) });

        let prog = MirProgram {
            tcx: unit_tcx,
            functions: vec![MirFunction {
                name: "param_return_alias".to_string(),
                params: vec![shared],
                return_place: shared,
                return_type: omni_mir::ast::TypeSpec::Unit,
                body: Body { blocks, local_decls },
            }],
        };

        let res = MirVerifier::verify_program(&prog);
        assert!(matches!(res, Err(MirVerificationError::ParamAliasesReturnPlace { .. })));
    }

    #[test]
    fn test_verifier_fails_on_undefined_local() {
        let mut local_decls = IndexVec::new();
        let ret_l = local_decls
            .push(LocalDecl { name: Some("_return".to_string()), ty: Some(omni_mir::Ty(1)) });
        let invalid_l = Local::from_usize(99);

        let mut blocks = IndexVec::new();
        blocks.push(BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret_l },
                Rvalue::Use(Operand::Copy(Place { local: invalid_l })),
            )],
            terminator: Some(Terminator::Return),
        });

        let prog = MirProgram {
            tcx: int_context(),
            functions: vec![MirFunction {
                name: "bad_local".to_string(),
                params: vec![],
                return_place: ret_l,
                return_type: omni_mir::ast::TypeSpec::Int,
                body: Body { blocks, local_decls },
            }],
        };

        let res = MirVerifier::verify_program(&prog);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), MirVerificationError::UndefinedLocal { .. }));
    }

    #[test]
    fn test_verifier_rejects_non_bool_comparison_destination() {
        let (tcx, int, bool_ty) = {
            let mut tcx = TyCtxt::new();
            let int = tcx.intern(TyKind::Int);
            let bool_ty = tcx.intern(TyKind::Bool);
            (tcx, int, bool_ty)
        };
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(LocalDecl { name: Some("_return".to_string()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::BinaryOp(
                    omni_mir::ir::BinOp::Eq,
                    Operand::Constant(omni_mir::ir::Constant::Lit(omni_mir::ast::Lit::Int(1))),
                    Operand::Constant(omni_mir::ir::Constant::Lit(omni_mir::ast::Lit::Int(2))),
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
                return_type: omni_mir::ast::TypeSpec::Int,
                body: Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::TypeMismatch { expected, actual, .. })
                if expected == int && actual == bool_ty
        ));
    }

    #[test]
    fn test_verifier_rejects_invalid_unary_operand() {
        let mut tcx = TyCtxt::new();
        let int = tcx.intern(TyKind::Int);
        let ret = {
            let mut locals = IndexVec::new();
            locals.push(LocalDecl { name: Some("_return".to_string()), ty: Some(int) })
        };
        let mut blocks = IndexVec::new();
        blocks.push(BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::UnaryOp(
                    omni_mir::ir::UnOp::Not,
                    Operand::Constant(omni_mir::ir::Constant::Lit(omni_mir::ast::Lit::Int(1))),
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
                return_type: omni_mir::ast::TypeSpec::Int,
                body: Body {
                    blocks,
                    local_decls: {
                        let mut locals = IndexVec::new();
                        locals.push(LocalDecl { name: Some("_return".to_string()), ty: Some(int) });
                        locals
                    },
                },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::InvalidUnaryOperand { .. })
        ));
    }

    #[test]
    fn test_verifier_rejects_bad_call_signature() {
        let mut tcx = TyCtxt::new();
        let int = tcx.intern(TyKind::Int);

        let mut caller_locals = IndexVec::new();
        let caller_ret =
            caller_locals.push(LocalDecl { name: Some("_return".to_string()), ty: Some(int) });
        let mut callee_locals = IndexVec::new();
        let callee_ret =
            callee_locals.push(LocalDecl { name: Some("_return".to_string()), ty: Some(int) });
        let callee_param =
            callee_locals.push(LocalDecl { name: Some("x".to_string()), ty: Some(int) });

        let mut caller_blocks = IndexVec::new();
        caller_blocks.push(BlockData {
            statements: vec![],
            terminator: Some(Terminator::Call {
                func: Operand::Constant(omni_mir::ir::Constant::FnRef("inc".to_string())),
                args: vec![],
                destination: None,
                target: BasicBlock::from_usize(1),
                cleanup: None,
            }),
        });
        caller_blocks.push(BlockData {
            statements: vec![Statement::Assign(
                Place { local: caller_ret },
                Rvalue::Use(Operand::Constant(omni_mir::ir::Constant::Lit(
                    omni_mir::ast::Lit::Int(0),
                ))),
            )],
            terminator: Some(Terminator::Return),
        });

        let mut callee_blocks = IndexVec::new();
        callee_blocks.push(BlockData {
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
                    return_type: omni_mir::ast::TypeSpec::Int,
                    body: Body { blocks: caller_blocks, local_decls: caller_locals },
                },
                MirFunction {
                    name: "inc".to_string(),
                    params: vec![callee_param],
                    return_place: callee_ret,
                    return_type: omni_mir::ast::TypeSpec::Int,
                    body: Body { blocks: callee_blocks, local_decls: callee_locals },
                },
            ],
        };

        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::CallArityMismatch { .. })
        ));
    }

    #[test]
    fn test_verifier_rejects_uninitialized_return() {
        let mut tcx = TyCtxt::new();
        let int = tcx.intern(TyKind::Int);
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(LocalDecl { name: Some("_return".to_string()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(BlockData { statements: vec![], terminator: Some(Terminator::Return) });
        let prog = MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "bad_return".to_string(),
                params: vec![],
                return_place: ret,
                return_type: omni_mir::ast::TypeSpec::Int,
                body: Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::UninitializedReturn { .. })
        ));
    }

    #[test]
    fn test_verifier_rejects_use_before_assignment() {
        let mut tcx = TyCtxt::new();
        let int = tcx.intern(TyKind::Int);
        let mut local_decls = IndexVec::new();
        let ret = local_decls.push(LocalDecl { name: Some("_return".to_string()), ty: Some(int) });
        let tmp = local_decls.push(LocalDecl { name: Some("tmp".to_string()), ty: Some(int) });
        let mut blocks = IndexVec::new();
        blocks.push(BlockData {
            statements: vec![Statement::Assign(
                Place { local: ret },
                Rvalue::Use(Operand::Copy(Place { local: tmp })),
            )],
            terminator: Some(Terminator::Return),
        });
        let prog = MirProgram {
            tcx,
            functions: vec![MirFunction {
                name: "bad_use".to_string(),
                params: vec![],
                return_place: ret,
                return_type: omni_mir::ast::TypeSpec::Int,
                body: Body { blocks, local_decls },
            }],
        };
        assert!(matches!(
            MirVerifier::verify_program(&prog),
            Err(MirVerificationError::UseBeforeAssignment { .. })
        ));
    }

    #[test]
    fn test_verifier_fails_on_undefined_block() {
        let mut local_decls = IndexVec::new();
        let ret_l = local_decls
            .push(LocalDecl { name: Some("_return".to_string()), ty: Some(omni_mir::Ty(1)) });
        let invalid_bb = BasicBlock::from_usize(10);

        let mut blocks = IndexVec::new();
        blocks
            .push(BlockData { statements: vec![], terminator: Some(Terminator::Goto(invalid_bb)) });

        let prog = MirProgram {
            tcx: int_context(),
            functions: vec![MirFunction {
                name: "bad_goto".to_string(),
                params: vec![],
                return_place: ret_l,
                return_type: omni_mir::ast::TypeSpec::Int,
                body: Body { blocks, local_decls },
            }],
        };

        let res = MirVerifier::verify_program(&prog);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), MirVerificationError::UndefinedBlock { .. }));
    }
}
