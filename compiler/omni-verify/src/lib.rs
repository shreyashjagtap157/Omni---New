//! Polonius Fact Generation and Linear Borrow Checking Engine (OWN-0005).

use omni_mir::ir::Body;

#[macro_export]
macro_rules! implements {
    ($tag:literal) => {};
}

implements!("OWN-0005");

/// Polonius-compatible fact structures for linear borrow checking
#[derive(Debug, Clone, Default)]
pub struct PoloniusFacts {
    pub loan_issued: Vec<(String, String)>,    // (loan, point)
    pub borrow_region: Vec<(String, String)>,  // (region, point)
    pub region_live_at: Vec<(String, String)>, // (region, point)
    pub killed: Vec<(String, String)>,         // (loan, point)
    pub outlives: Vec<(String, String)>,       // (region1, region2)
}

impl PoloniusFacts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract concrete Polonius facts from explicit MIR borrow operations.
    ///
    /// Facts are generated only for actual Rvalue::Reference operations and
    /// their concrete reference-local invalidations. The extractor deliberately
    /// does not invent loans for unrelated statements. Full CFG-aware region
    /// inference belongs to the ownership/lifetime pass; this layer provides a
    /// faithful fact projection of the MIR events that already exist.
    pub fn extract_from_mir(body: &Body) -> Self {
        use std::collections::BTreeMap;
        use omni_mir::ir::{Projection, Rvalue, Statement, Terminator};

        let mut facts = Self::default();
        let mut active_by_local: BTreeMap<omni_mir::ir::Local, String> = BTreeMap::new();
        let mut region_by_loan: BTreeMap<String, String> = BTreeMap::new();

        for (block_idx, block) in body.blocks.iter().enumerate() {
            for (stmt_idx, statement) in block.statements.iter().enumerate() {
                let point = format!("bb{}_{}", block_idx, stmt_idx);

                match statement {
                    Statement::Assign(destination, rvalue) => {
                        if destination.projections.is_empty() {
                            if let Some(previous_loan) = active_by_local.remove(&destination.local) {
                                facts.killed.push((previous_loan, point.clone()));
                            }
                        }

                        if let Rvalue::Reference { place: borrowed, .. } = rvalue {
                            let loan = format!("loan_bb{}_{}", block_idx, stmt_idx);
                            let region = format!("'r_bb{}_{}", block_idx, stmt_idx);

                            facts.loan_issued.push((loan.clone(), point.clone()));
                            facts.borrow_region.push((region.clone(), point.clone()));
                            facts.region_live_at.push((region.clone(), point.clone()));

                            if borrowed.projections.iter().any(|p| matches!(p, Projection::Deref)) {
                                if let Some(parent_loan) = active_by_local.get(&borrowed.local) {
                                    if let Some(parent_region) = region_by_loan.get(parent_loan) {
                                        facts.outlives.push((parent_region.clone(), region.clone()));
                                    }
                                }
                            }

                            region_by_loan.insert(loan.clone(), region);
                            if destination.projections.is_empty() {
                                active_by_local.insert(destination.local, loan);
                            }
                        }
                    }
                    Statement::Drop(place) if place.projections.is_empty() => {
                        if let Some(loan) = active_by_local.remove(&place.local) {
                            facts.killed.push((loan, point.clone()));
                        }
                    }
                    _ => {}
                }
            }

            if matches!(block.terminator, Some(Terminator::Return)) {
                let point = format!("bb{}_term", block_idx);
                for (_, loan) in active_by_local.iter() {
                    facts.killed.push((loan.clone(), point.clone()));
                }
                active_by_local.clear();
            }
        }

        facts
    }

    pub fn emit_fact(&mut self, category: &str, entity: &str, point: &str) {
        match category {
            "borrow_region" => self.borrow_region.push((entity.into(), point.into())),
            "killed" => self.killed.push((entity.into(), point.into())),
            "loan_issued" => self.loan_issued.push((entity.into(), point.into())),
            _ => {}
        }
    }
}

#[cfg(test)]
mod polonius_tests {
    use super::*;

    #[test]
    fn test_polonius_mir_extraction() {
        let body = Body::default();
        let facts = PoloniusFacts::extract_from_mir(&body);
        assert!(facts.borrow_region.is_empty());
    }
}

pub mod llvm_val;
pub mod mir_verifier;

pub use mir_verifier::{MirVerificationError, MirVerifier};
