//! Polonius Fact Generation and Linear Borrow Checking Engine (OWN-0005).

use omni_mir::ir::{BasicBlock, Body, Terminator};
use polonius_engine::{Algorithm, AllFacts, FactTypes, Output};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PoloniusId(usize);

impl From<usize> for PoloniusId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PoloniusId> for usize {
    fn from(value: PoloniusId) -> Self {
        value.0
    }
}

impl polonius_engine::Atom for PoloniusId {
    fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OmniFactTypes;

impl FactTypes for OmniFactTypes {
    type Origin = PoloniusId;
    type Loan = PoloniusId;
    type Point = PoloniusId;
    type Variable = PoloniusId;
    type Path = PoloniusId;
}

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
        use omni_mir::ir::{Projection, Rvalue, Statement, Terminator};
        use std::collections::BTreeMap;

        let mut facts = Self::default();
        let mut active_by_local: BTreeMap<omni_mir::ir::Local, String> = BTreeMap::new();
        let mut active_loans = std::collections::BTreeSet::<String>::new();
        let mut region_by_loan: BTreeMap<String, String> = BTreeMap::new();

        for (block_idx, block) in body.blocks.iter().enumerate() {
            for (stmt_idx, statement) in block.statements.iter().enumerate() {
                let point = format!("bb{}_{}", block_idx, stmt_idx);

                match statement {
                    Statement::Assign(destination, rvalue) => {
                        let parent_loan = match rvalue {
                            Rvalue::Reference { place: borrowed, .. }
                                if borrowed
                                    .projections
                                    .iter()
                                    .any(|p| matches!(p, Projection::Deref)) =>
                            {
                                active_by_local.get(&borrowed.local).cloned()
                            }
                            _ => None,
                        };

                        // A reborrow keeps its parent loan logically alive while
                        // the child loan is active. Do not kill that parent merely
                        // because the child reference overwrites the same local.
                        let replaces_parent = destination.projections.is_empty()
                            && parent_loan.is_some()
                            && active_by_local
                                .get(&destination.local)
                                .is_some_and(|current| Some(current) == parent_loan.as_ref());

                        if destination.projections.is_empty() && !replaces_parent {
                            if let Some(previous_loan) = active_by_local.remove(&destination.local)
                            {
                                active_loans.remove(&previous_loan);
                                facts.killed.push((previous_loan, point.clone()));
                            }
                        }

                        if let Rvalue::Reference { place: _borrowed, .. } = rvalue {
                            let loan = format!("loan_bb{}_{}", block_idx, stmt_idx);
                            let region = format!("'r_bb{}_{}", block_idx, stmt_idx);

                            facts.loan_issued.push((loan.clone(), point.clone()));
                            facts.borrow_region.push((region.clone(), point.clone()));
                            facts.region_live_at.push((region.clone(), point.clone()));

                            if let Some(parent_loan) = parent_loan.as_ref() {
                                if let Some(parent_region) = region_by_loan.get(parent_loan) {
                                    facts.outlives.push((parent_region.clone(), region.clone()));
                                }
                            }

                            region_by_loan.insert(loan.clone(), region);
                            active_loans.insert(loan.clone());
                            if destination.projections.is_empty() {
                                active_by_local.insert(destination.local, loan);
                            }
                        }
                    }
                    Statement::Drop(place) if place.projections.is_empty() => {
                        if let Some(loan) = active_by_local.remove(&place.local) {
                            active_loans.remove(&loan);
                            facts.killed.push((loan, point.clone()));
                        }
                    }
                    _ => {}
                }
            }

            if matches!(block.terminator, Some(Terminator::Return)) {
                let point = format!("bb{}_term", block_idx);
                for loan in &active_loans {
                    facts.killed.push((loan.clone(), point.clone()));
                }
                active_by_local.clear();
                active_loans.clear();
            }
        }

        facts
    }

    /// Run the actual Polonius engine over the facts extracted from this body.
    ///
    /// The current fact vocabulary intentionally starts with explicit loans,
    /// kills, outlives relations, and CFG edges. The existing ownership checker
    /// remains authoritative until path invalidation facts are complete.
    pub fn run_engine(&self, body: &Body) -> Output<OmniFactTypes> {
        let mut point_ids = BTreeMap::<String, PoloniusId>::new();
        let mut origin_ids = BTreeMap::<String, PoloniusId>::new();
        let mut loan_ids = BTreeMap::<String, PoloniusId>::new();

        let intern = |map: &mut BTreeMap<String, PoloniusId>, key: &str| -> PoloniusId {
            let next = PoloniusId::from(map.len());
            *map.entry(key.to_string()).or_insert(next)
        };

        for (loan, point) in &self.loan_issued {
            let _ = intern(&mut loan_ids, loan);
            let _ = intern(&mut point_ids, point);
        }
        for (loan, point) in &self.killed {
            let _ = intern(&mut loan_ids, loan);
            let _ = intern(&mut point_ids, point);
        }
        for (region, point) in &self.borrow_region {
            let _ = intern(&mut origin_ids, region);
            let _ = intern(&mut point_ids, point);
        }
        for (parent, child) in &self.outlives {
            let _ = intern(&mut origin_ids, parent);
            let _ = intern(&mut origin_ids, child);
        }

        let mut facts = AllFacts::<OmniFactTypes>::default();

        for (region, point) in &self.borrow_region {
            if let Some(loan) = self
                .loan_issued
                .iter()
                .find(|(_, issued_point)| issued_point == point)
                .map(|(loan, _)| loan)
            {
                facts.loan_issued_at.push((origin_ids[region], loan_ids[loan], point_ids[point]));
            }
        }

        for (loan, point) in &self.killed {
            facts.loan_killed_at.push((loan_ids[loan], point_ids[point]));
        }

        let block_entry =
            |block: BasicBlock, body: &Body, points: &mut BTreeMap<String, PoloniusId>| {
                if body.blocks[block].statements.is_empty() {
                    let key = format!("bb{}_term", block.index());
                    Some(intern(points, &key))
                } else {
                    let key = format!("bb{}_0", block.index());
                    Some(intern(points, &key))
                }
            };

        for (block, data) in body.blocks.iter_enumerated() {
            for statement_index in 0..data.statements.len() {
                let _ = intern(&mut point_ids, &format!("bb{}_{}", block.index(), statement_index));
            }
            let _ = intern(&mut point_ids, &format!("bb{}_term", block.index()));
        }

        for (block, data) in body.blocks.iter_enumerated() {
            for statement_index in 0..data.statements.len() {
                let from = point_ids[&format!("bb{}_{}", block.index(), statement_index)];
                let to = if statement_index + 1 < data.statements.len() {
                    point_ids[&format!("bb{}_{}", block.index(), statement_index + 1)]
                } else {
                    point_ids[&format!("bb{}_term", block.index())]
                };
                facts.cfg_edge.push((from, to));
            }

            if data.statements.is_empty() {
                let _ = block_entry(block, body, &mut point_ids);
            }

            if let Some(term) = data.terminator.as_ref() {
                let from = point_ids[&format!("bb{}_term", block.index())];
                match term {
                    Terminator::Goto(target) => {
                        if let Some(to) = block_entry(*target, body, &mut point_ids) {
                            facts.cfg_edge.push((from, to));
                        }
                    }
                    Terminator::SwitchInt { targets, otherwise, .. } => {
                        for target in
                            targets.iter().map(|(_, block)| block).chain(std::iter::once(otherwise))
                        {
                            if let Some(to) = block_entry(*target, body, &mut point_ids) {
                                facts.cfg_edge.push((from, to));
                            }
                        }
                    }
                    Terminator::Call { target, cleanup, .. } => {
                        if let Some(to) = block_entry(*target, body, &mut point_ids) {
                            facts.cfg_edge.push((from, to));
                        }
                        if let Some(cleanup) = cleanup {
                            if let Some(to) = block_entry(*cleanup, body, &mut point_ids) {
                                facts.cfg_edge.push((from, to));
                            }
                        }
                    }
                    Terminator::Return | Terminator::Unreachable => {}
                }
            }
        }

        for (parent, child) in &self.outlives {
            let point = self
                .borrow_region
                .iter()
                .find(|(region, _)| region == child)
                .map(|(_, point)| point);
            if let Some(point) = point {
                facts.subset_base.push((origin_ids[child], origin_ids[parent], point_ids[point]));
            }
        }

        Output::compute(&facts, Algorithm::Naive, false)
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
    use omni_mir::ast::Lit;
    use omni_mir::ir::{
        BlockData, Constant, Local, LocalDecl, Place, Rvalue, Statement, Terminator,
    };
    use omni_mir::{TyCtxt, TyKind};

    fn body_with_statements(statements: Vec<Statement>) -> Body {
        let mut body = Body::default();
        let mut locals = IndexVec::new();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner: int_ty });
        locals.push(LocalDecl { name: Some("value".into()), ty: Some(int_ty) });
        locals.push(LocalDecl { name: Some("reference".into()), ty: Some(ref_ty) });
        let mut blocks = IndexVec::new();
        blocks.push(BlockData { statements, terminator: Some(Terminator::Return) });
        body.local_decls = locals;
        body.blocks = blocks;
        body
    }

    fn reference_statement(
        destination: Local,
        borrowed: Place,
        mutable: bool,
        ty: omni_mir::Ty,
    ) -> Statement {
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
        let ref_ty = tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner: int_ty });
        let facts =
            PoloniusFacts::extract_from_mir(&body_with_statements(vec![reference_statement(
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
    fn engine_accepts_extracted_simple_borrow_facts() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner: int_ty });
        let body = body_with_statements(vec![reference_statement(
            Local::from_usize(1),
            Place::local(Local::from_usize(0)),
            true,
            ref_ty,
        )]);
        let facts = PoloniusFacts::extract_from_mir(&body);
        let output = facts.run_engine(&body);
        assert!(
            output.errors.is_empty(),
            "simple MIR borrow facts must be accepted by Polonius: {:?}",
            output.errors
        );
    }

    #[test]
    fn replacing_reference_local_kills_previous_loan_at_replacement_point() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner: int_ty });
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
        assert_eq!(facts.killed, vec![(String::from("loan_bb0_0"), String::from("bb0_1"))]);
    }

    #[test]
    fn dereference_reborrow_records_parent_region_relationship() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner: int_ty });
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
        assert_eq!(facts.outlives, vec![(String::from("'r_bb0_0"), String::from("'r_bb0_1"))]);
    }
}

pub mod llvm_val;
pub mod mir_verifier;
mod ownership;

pub use mir_verifier::{MirVerificationError, MirVerifier};
