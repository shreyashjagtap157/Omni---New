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
    pub loan_issued: Vec<(String, String)>,       // (loan, point)
    pub borrow_region: Vec<(String, String)>,     // (region, point)
    pub region_live_at: Vec<(String, String)>,    // (region, point)
    pub killed: Vec<(String, String)>,            // (loan, point)
    pub outlives: Vec<(String, String)>,          // (region1, region2)
    pub var_used_at: Vec<(String, String)>,       // (variable, point)
    pub var_defined_at: Vec<(String, String)>,    // (variable, point)
    pub var_dropped_at: Vec<(String, String)>,    // (variable, point)
    pub deref_origin: Vec<(String, String)>,      // (variable, region)
    pub drop_deref_origin: Vec<(String, String)>, // (variable, region)
    pub invalidated: Vec<(String, String)>,       // (loan, point)
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
        let mut active_loans: BTreeMap<String, omni_mir::ir::Place> = BTreeMap::new();
        let mut origin_by_local: BTreeMap<omni_mir::ir::Local, String> = BTreeMap::new();
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
                        let replaces_parent = destination.projections.is_empty()
                            && parent_loan.is_some()
                            && active_by_local
                                .get(&destination.local)
                                .is_some_and(|current| Some(current) == parent_loan.as_ref());

                        record_rvalue_uses(
                            rvalue,
                            &point,
                            &origin_by_local,
                            &active_loans,
                            &mut facts,
                        );
                        record_place_write(destination, &point, &active_loans, &mut facts);

                        if destination.projections.is_empty() {
                            if !replaces_parent {
                                if let Some(previous_loan) =
                                    active_by_local.remove(&destination.local)
                                {
                                    active_loans.remove(&previous_loan);
                                    region_by_loan.remove(&previous_loan);
                                    origin_by_local.remove(&destination.local);
                                    push_unique(&mut facts.killed, (previous_loan, point.clone()));
                                }
                            }
                            push_unique(
                                &mut facts.var_defined_at,
                                (local_key(destination.local), point.clone()),
                            );
                        }

                        if let Rvalue::Reference { place: borrowed, .. } = rvalue {
                            let loan = format!("loan_bb{}_{}", block_idx, stmt_idx);
                            let region = format!("'r_bb{}_{}", block_idx, stmt_idx);

                            push_unique(&mut facts.loan_issued, (loan.clone(), point.clone()));
                            push_unique(&mut facts.borrow_region, (region.clone(), point.clone()));
                            push_unique(&mut facts.region_live_at, (region.clone(), point.clone()));

                            if let Some(parent_loan) = parent_loan.as_ref() {
                                if let Some(parent_region) = region_by_loan.get(parent_loan) {
                                    push_unique(
                                        &mut facts.outlives,
                                        (parent_region.clone(), region.clone()),
                                    );
                                }
                            }

                            region_by_loan.insert(loan.clone(), region.clone());
                            if destination.projections.is_empty() {
                                active_by_local.insert(destination.local, loan.clone());
                                active_loans.insert(loan, borrowed.clone());
                                origin_by_local.insert(destination.local, region);
                            }
                        }
                    }
                    Statement::Drop(place) => {
                        record_place_use(place, &point, &origin_by_local, &mut facts, true);
                        push_unique(
                            &mut facts.var_dropped_at,
                            (local_key(place.local), point.clone()),
                        );

                        if place.projections.is_empty() {
                            if let Some(loan) = active_by_local.remove(&place.local) {
                                active_loans.remove(&loan);
                                region_by_loan.remove(&loan);
                                origin_by_local.remove(&place.local);
                                push_unique(&mut facts.killed, (loan, point.clone()));
                            }
                        }
                    }
                    Statement::BoundsCheck { index, .. } => {
                        let place = omni_mir::ir::Place::local(*index);
                        record_place_use(&place, &point, &origin_by_local, &mut facts, false);
                    }
                    Statement::Assume(_) => {}
                }
            }

            if let Some(terminator) = block.terminator.as_ref() {
                let point = format!("bb{}_term", block_idx);
                match terminator {
                    Terminator::SwitchInt { discr, .. } => {
                        record_operand_use(
                            discr,
                            &point,
                            &origin_by_local,
                            &active_loans,
                            &mut facts,
                        );
                    }
                    Terminator::SwitchEnum { place, .. } => {
                        record_place_use(place, &point, &origin_by_local, &mut facts, false);
                    }
                    Terminator::Call { func, args, destination, .. } => {
                        record_operand_use(
                            func,
                            &point,
                            &origin_by_local,
                            &active_loans,
                            &mut facts,
                        );
                        for argument in args {
                            record_operand_use(
                                argument,
                                &point,
                                &origin_by_local,
                                &active_loans,
                                &mut facts,
                            );
                        }
                        if let Some(destination) = destination {
                            record_place_write(destination, &point, &active_loans, &mut facts);
                            if destination.projections.is_empty() {
                                push_unique(
                                    &mut facts.var_defined_at,
                                    (local_key(destination.local), point.clone()),
                                );
                            }
                        }
                    }
                    Terminator::Return => {
                        for loan in active_loans.keys().cloned().collect::<Vec<_>>() {
                            push_unique(&mut facts.killed, (loan, point.clone()));
                        }
                        active_by_local.clear();
                        active_loans.clear();
                        origin_by_local.clear();
                    }
                    Terminator::Goto(_) | Terminator::Unreachable => {}
                }
            }
        }

        // The explicit region-live vector remains useful as an extraction-side
        // witness; the Polonius engine derives origin liveness from variable
        // liveness plus dereference-origin facts.
        for (local, region) in &origin_by_local {
            let _ = (local, region);
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
        let mut variable_ids = BTreeMap::<String, PoloniusId>::new();

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
        for (variable, point) in &self.var_used_at {
            let _ = intern(&mut variable_ids, variable);
            let _ = intern(&mut point_ids, point);
        }
        for (variable, point) in &self.var_defined_at {
            let _ = intern(&mut variable_ids, variable);
            let _ = intern(&mut point_ids, point);
        }
        for (variable, point) in &self.var_dropped_at {
            let _ = intern(&mut variable_ids, variable);
            let _ = intern(&mut point_ids, point);
        }
        for (variable, origin) in &self.deref_origin {
            let _ = intern(&mut variable_ids, variable);
            let _ = intern(&mut origin_ids, origin);
        }
        for (variable, origin) in &self.drop_deref_origin {
            let _ = intern(&mut variable_ids, variable);
            let _ = intern(&mut origin_ids, origin);
        }
        for (loan, point) in &self.invalidated {
            let _ = intern(&mut loan_ids, loan);
            let _ = intern(&mut point_ids, point);
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
        for (loan, point) in &self.invalidated {
            facts.loan_invalidated_at.push((point_ids[point], loan_ids[loan]));
        }
        for (variable, point) in &self.var_used_at {
            facts.var_used_at.push((variable_ids[variable], point_ids[point]));
        }
        for (variable, point) in &self.var_defined_at {
            facts.var_defined_at.push((variable_ids[variable], point_ids[point]));
        }
        for (variable, point) in &self.var_dropped_at {
            facts.var_dropped_at.push((variable_ids[variable], point_ids[point]));
        }
        for (variable, origin) in &self.deref_origin {
            facts.use_of_var_derefs_origin.push((variable_ids[variable], origin_ids[origin]));
        }
        for (variable, origin) in &self.drop_deref_origin {
            facts.drop_of_var_derefs_origin.push((variable_ids[variable], origin_ids[origin]));
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
                    Terminator::SwitchEnum { targets, otherwise, .. } => {
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

fn local_key(local: omni_mir::ir::Local) -> String {
    format!("local{}", local.index())
}

fn push_unique<T: PartialEq>(facts: &mut Vec<T>, value: T) {
    if !facts.contains(&value) {
        facts.push(value);
    }
}

fn places_conflict(a: &omni_mir::ir::Place, b: &omni_mir::ir::Place) -> bool {
    if a.local != b.local {
        return false;
    }
    let shared = a.projections.len().min(b.projections.len());
    a.projections[..shared] == b.projections[..shared]
}

fn record_place_use(
    place: &omni_mir::ir::Place,
    point: &str,
    origin_by_local: &BTreeMap<omni_mir::ir::Local, String>,
    facts: &mut PoloniusFacts,
    is_drop: bool,
) {
    push_unique(&mut facts.var_used_at, (local_key(place.local), point.to_string()));
    if place
        .projections
        .iter()
        .any(|projection| matches!(projection, omni_mir::ir::Projection::Deref))
    {
        if let Some(origin) = origin_by_local.get(&place.local) {
            push_unique(&mut facts.deref_origin, (local_key(place.local), origin.clone()));
            push_unique(&mut facts.region_live_at, (origin.clone(), point.to_string()));
        }
    } else if is_drop {
        if let Some(origin) = origin_by_local.get(&place.local) {
            push_unique(&mut facts.drop_deref_origin, (local_key(place.local), origin.clone()));
            push_unique(&mut facts.region_live_at, (origin.clone(), point.to_string()));
        }
    }
}

fn record_place_write(
    destination: &omni_mir::ir::Place,
    point: &str,
    active_loans: &BTreeMap<String, omni_mir::ir::Place>,
    facts: &mut PoloniusFacts,
) {
    for (loan, borrowed_place) in active_loans {
        if places_conflict(destination, borrowed_place) {
            push_unique(&mut facts.invalidated, (loan.clone(), point.to_string()));
        }
    }
}

fn record_operand_use(
    operand: &omni_mir::ir::Operand,
    point: &str,
    origin_by_local: &BTreeMap<omni_mir::ir::Local, String>,
    active_loans: &BTreeMap<String, omni_mir::ir::Place>,
    facts: &mut PoloniusFacts,
) {
    match operand {
        omni_mir::ir::Operand::Copy(place) => {
            record_place_use(place, point, origin_by_local, facts, false);
        }
        omni_mir::ir::Operand::Move(place) => {
            record_place_use(place, point, origin_by_local, facts, false);
            for (loan, borrowed_place) in active_loans {
                if places_conflict(place, borrowed_place) {
                    push_unique(&mut facts.invalidated, (loan.clone(), point.to_string()));
                }
            }
        }
        omni_mir::ir::Operand::Constant(_) => {}
    }
}

fn record_rvalue_uses(
    rvalue: &omni_mir::ir::Rvalue,
    point: &str,
    origin_by_local: &BTreeMap<omni_mir::ir::Local, String>,
    active_loans: &BTreeMap<String, omni_mir::ir::Place>,
    facts: &mut PoloniusFacts,
) {
    use omni_mir::ir::Rvalue;
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) | Rvalue::Cast { operand, .. } => {
            record_operand_use(operand, point, origin_by_local, active_loans, facts);
        }
        Rvalue::BinaryOp(_, lhs, rhs) => {
            record_operand_use(lhs, point, origin_by_local, active_loans, facts);
            record_operand_use(rhs, point, origin_by_local, active_loans, facts);
        }
        Rvalue::Aggregate { operands, .. } | Rvalue::EnumVariant { operands, .. } => {
            for operand in operands {
                record_operand_use(operand, point, origin_by_local, &BTreeMap::new(), facts);
            }
        }
        Rvalue::Struct { fields, .. } => {
            for (_, operand) in fields {
                record_operand_use(operand, point, origin_by_local, &BTreeMap::new(), facts);
            }
        }
        Rvalue::Reference { place, .. } => {
            record_place_use(place, point, origin_by_local, facts, false);
        }
        Rvalue::Range { start, end, .. } => {
            record_operand_use(start, point, origin_by_local, active_loans, facts);
            record_operand_use(end, point, origin_by_local, active_loans, facts);
        }
        Rvalue::Field { base, .. } | Rvalue::EnumField { base, .. } => {
            record_operand_use(base, point, origin_by_local, active_loans, facts);
        }
        Rvalue::Index { base, index, .. } => {
            record_operand_use(base, point, origin_by_local, &BTreeMap::new(), facts);
            record_operand_use(index, point, origin_by_local, active_loans, facts);
        }
    }
}

#[cfg(test)]
mod polonius_tests {
    use super::*;
    use index_vec::IndexVec;
    use omni_mir::ast::Lit;
    use omni_mir::ir::{
        BlockData, Constant, Local, LocalDecl, Operand, Place, Rvalue, Statement, Terminator,
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
    fn engine_rejects_write_while_borrow_will_be_dereferenced() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner: int_ty });
        let deref_place =
            Place::local(Local::from_usize(1)).project(omni_mir::ir::Projection::Deref);
        let body = body_with_statements(vec![
            reference_statement(
                Local::from_usize(1),
                Place::local(Local::from_usize(0)),
                true,
                ref_ty,
            ),
            Statement::Assign(
                Place::local(Local::from_usize(0)),
                Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(2)))),
            ),
            Statement::Assign(
                Place::local(Local::from_usize(0)),
                Rvalue::Use(Operand::Copy(deref_place)),
            ),
        ]);

        let facts = PoloniusFacts::extract_from_mir(&body);
        assert!(facts
            .invalidated
            .iter()
            .any(|(loan, point)| { loan == "loan_bb0_0" && point == "bb0_1" }));
        let output = facts.run_engine(&body);
        assert!(
            !output.errors.is_empty(),
            "a write that invalidates a live borrow must be rejected by Polonius"
        );
    }

    #[test]
    fn engine_accepts_write_after_reference_drop() {
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let ref_ty = tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner: int_ty });
        let body = body_with_statements(vec![
            reference_statement(
                Local::from_usize(1),
                Place::local(Local::from_usize(0)),
                true,
                ref_ty,
            ),
            Statement::Drop(Place::local(Local::from_usize(1))),
            Statement::Assign(
                Place::local(Local::from_usize(0)),
                Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(2)))),
            ),
        ]);

        let facts = PoloniusFacts::extract_from_mir(&body);
        let output = facts.run_engine(&body);
        assert!(
            output.errors.is_empty(),
            "dropping the reference must end the borrow before the later write"
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
