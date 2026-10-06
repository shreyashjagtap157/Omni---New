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
    use index_vec::IndexVec;
    use omni_mir::ir::{BlockData, Constant, Local, LocalDecl, Place, Rvalue, Statement, Terminator};
    use omni_mir::ast::Lit;
    use omni_mir::{TyCtxt, TyKind};

    fn body_with_statements(statements: Vec<Statement>) -> Body {
        let mut body = Body::default();
        let mut locals = IndexVec::new();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference {
            lifetime: None,
            mutable: true,
            inner: int_ty,
        });
        locals.push(LocalDecl { name: Some("value".into()), ty: Some(int_ty) });
        locals.push(LocalDecl { name: Some("reference".into()), ty: Some(ref_ty) });
        let mut blocks = IndexVec::new();
        blocks.push(BlockData { statements, terminator: Some(Terminator::Return) });
        body.local_decls = locals;
        body.blocks = blocks;
        body
    }

    fn reference_statement(destination: Local, borrowed: Place, mutable: bool, ty: omni_mir::Ty) -> Statement {
        Statement::Assign(
            Place::local(destination),
            Rvalue::Reference { place: borrowed, mutable, ty },
        )
    }

    #[test]
    fn extraction_has_no_fabricated_loans() {
        let statement = Statement::Assign(
            Place::local(Local::from_usize(0)),
            Rvalue::Use(omni_mir::ir::Operand::Constant(Constant::Lit(Lit::Int(1)))),
        );
        let facts = PoloniusFacts::extract_from_mir(&body_with_statements(vec![statement]));
        assert!(facts.loan_issued.is_empty());
        assert!(facts.borrow_region.is_empty());
        assert!(facts.killed.is_empty());
    }

    #[test]
    fn extraction_emits_one_loan_for_one_reference() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference {
            lifetime: None,
            mutable: true,
            inner: int_ty,
        });
        let facts = PoloniusFacts::extract_from_mir(&body_with_statements(vec![reference_statement(
            Local::from_usize(1),
            Place::local(Local::from_usize(0)),
            true,
            ref_ty,
        )]));
        assert_eq!(facts.loan_issued, vec![(String::from("loan_bb0_0"), String::from("bb0_0"))]);
        assert_eq!(facts.borrow_region, vec![(String::from("'r_bb0_0"), String::from("bb0_0"))]);
        assert_eq!(facts.region_live_at, vec![(String::from("'r_bb0_0"), String::from("bb0_0"))]);
        assert_eq!(facts.killed, vec![(String::from("loan_bb0_0"), String::from("bb0_term"))]);
    }

    #[test]
    fn replacing_reference_local_kills_previous_loan_at_replacement_point() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference {
            lifetime: None,
            mutable: true,
            inner: int_ty,
        });
        let statements = vec![
            reference_statement(
                Local::from_usize(1),
                Place::local(Local::from_usize(0)),
                true,
                ref_ty,
            ),
            Statement::Assign(
                Place::local(Local::from_usize(1)),
                Rvalue::Use(omni_mir::ir::Operand::Constant(Constant::Lit(Lit::Int(2)))),
            ),
        ];
        let facts = PoloniusFacts::extract_from_mir(&body_with_statements(statements));
        assert!(facts.loan_issued.iter().any(|(loan, _)| loan == "loan_bb0_0"));
        assert_eq!(
            facts.killed,
            vec![(String::from("loan_bb0_0"), String::from("bb0_1"))]
        );
    }

    #[test]
    fn dereference_reborrow_records_parent_region_relationship() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference {
            lifetime: None,
            mutable: true,
            inner: int_ty,
        });
        let child = reference_statement(
            Local::from_usize(1),
            Place::local(Local::from_usize(0)),
            true,
            ref_ty,
        );
        let reborrow = reference_statement(
            Local::from_usize(1),
            Place::local(Local::from_usize(1)).project(omni_mir::ir::Projection::Deref),
            true,
            ref_ty,
        );
        let facts = PoloniusFacts::extract_from_mir(&body_with_statements(vec![child, reborrow]));
        assert_eq!(
            facts.outlives,
            vec![(String::from("'r_bb0_0"), String::from("'r_bb0_1"))]
        );
    }
}

pub mod llvm_val;
pub mod mir_verifier;

pub use mir_verifier::{MirVerificationError, MirVerifier};
