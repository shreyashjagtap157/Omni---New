//! Structural MIR Verification Engine for Omni (`omni-verify`).
//! Validates structural and control-flow graph invariants before native code generation:
//! 1. All referenced Local places/operands exist in function `local_decls`.
//! 2. All target BasicBlock handles in terminators exist in function `blocks`.
//! 3. All basic blocks terminate in a valid Terminator (`Return`, `Goto`, `SwitchInt`, `Call`, `Unreachable`).
//! 4. Parameter and return local indices strictly match `MirFunction` signature parameters.

use omni_mir::ir::{
    BasicBlock, Local, MirFunction, MirProgram, Operand, Place, Statement, Terminator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirVerificationError {
    UndefinedLocal { func: String, local: Local },
    UndefinedBlock { func: String, block: BasicBlock },
    UnterminatedBlock { func: String, block: BasicBlock },
    InvalidReturnPlace { func: String, expected: Local, actual: Local },
    InvalidParamLocal { func: String, param_index: usize, local: Local },
    EmptyFunctionBody { func: String },
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
            Self::EmptyFunctionBody { func } => {
                write!(
                    f,
                    "MIR Verification Failure in '{}': Function contains zero basic blocks",
                    func
                )
            }
        }
    }
}

pub struct MirVerifier;

impl MirVerifier {
    pub fn verify_program(prog: &MirProgram) -> Result<(), MirVerificationError> {
        for func in &prog.functions {
            Self::verify_function(func)?;
        }
        Ok(())
    }

    pub fn verify_function(func: &MirFunction) -> Result<(), MirVerificationError> {
        let fn_name = &func.name;

        if func.body.blocks.is_empty() {
            return Err(MirVerificationError::EmptyFunctionBody { func: fn_name.clone() });
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

        // Validate parameter locals exist and are typed
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
                            omni_mir::ir::Rvalue::Use(op) => {
                                Self::check_operand(fn_name, op, num_locals)?
                            }
                            omni_mir::ir::Rvalue::BinaryOp(_, op1, op2) => {
                                Self::check_operand(fn_name, op1, num_locals)?;
                                Self::check_operand(fn_name, op2, num_locals)?;
                            }
                            omni_mir::ir::Rvalue::UnaryOp(_, op) => {
                                Self::check_operand(fn_name, op, num_locals)?
                            }
                        }
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
    fn test_verifier_fails_on_undefined_block() {
        let mut local_decls = IndexVec::new();
        let ret_l = local_decls
            .push(LocalDecl { name: Some("_return".to_string()), ty: Some(omni_mir::Ty(1)) });
        let invalid_bb = BasicBlock::from_usize(10);

        let mut blocks = IndexVec::new();
        blocks
            .push(BlockData { statements: vec![], terminator: Some(Terminator::Goto(invalid_bb)) });

        let prog = MirProgram {
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
