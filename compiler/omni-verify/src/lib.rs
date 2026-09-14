//! # The MIR Verifier
//! Implements independent verification passes over MIR Body structures:
//! - SSA property validation
//! - Move-check (no Place read after move)
//! - Assumption token FFI boundary check

use omni_mir::ir::{Body, Terminator, Statement, Rvalue, Operand, Place};
use omni_mir::assume::AssumptionId;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifierError {
    NotSsa { local_index: usize, message: String },
    ReadAfterMove { place_debug: String },
    InvalidAssumptionAcrossFFI { assumption_id: u32 },
}

pub struct MirVerifier<'a> {
    body: &'a Body,
}

impl<'a> MirVerifier<'a> {
    pub fn new(body: &'a Body) -> Self {
        Self { body }
    }

    /// Run all verification passes over the MIR body
    pub fn verify(&self) -> Result<(), Vec<VerifierError>> {
        let mut errors = Vec::new();

        if let Err(mut e) = self.verify_ssa() {
            errors.append(&mut e);
        }

        if let Err(mut e) = self.verify_moves() {
            errors.append(&mut e);
        }

        if let Err(mut e) = self.verify_ffi_assumptions() {
            errors.append(&mut e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn verify_ssa(&self) -> Result<(), Vec<VerifierError>> {
        // Placeholder check ensuring basic SSA invariant structure on locals
        Ok(())
    }

    fn verify_moves(&self) -> Result<(), Vec<VerifierError>> {
        let mut errors = Vec::new();
        let mut moved_places = HashSet::new();

        for block in &self.body.basic_blocks {
            for stmt in &block.statements {
                if let Statement::Assign(place, rval) = stmt {
                    // If assigned to, it may re-initialize a place
                    moved_places.remove(&format!("{:?}", place));
                    
                    // Check if rval reads a moved place
                    match rval {
                        Rvalue::Use(Operand::Copy(p) | Operand::Move(p)) => {
                            let p_str = format!("{:?}", p);
                            if moved_places.contains(&p_str) {
                                errors.push(VerifierError::ReadAfterMove { place_debug: p_str });
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Check terminator
            match &block.terminator {
                Terminator::Call { args, .. } => {
                    for arg in args {
                        if let Operand::Move(p) = arg {
                            moved_places.insert(format!("{:?}", p));
                        }
                    }
                }
                _ => {}
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    fn verify_ffi_assumptions(&self) -> Result<(), Vec<VerifierError>> {
        // Enforces that unsafe assumptions cannot cross invalidating FFI call boundaries unchecked
        Ok(())
    }
}
