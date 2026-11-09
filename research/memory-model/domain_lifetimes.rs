//! Isolated model for region capture, structured tasks, and explicit promotion.
//!
//! This is a research model, not an Omni runtime, static borrow checker, task
//! scheduler, or production garbage collector. It tests lifetime boundaries,
//! explicit deep-copy promotion, and a conservative shared/exclusive access
//! ledger. Passing tests validate only the encoded state transitions.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ObjectId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RegionId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CaptureId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TaskId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageDomain {
    Tracing,
    Region(RegionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Object {
    domain: StorageDomain,
    edges: BTreeSet<ObjectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureKind {
    Closure,
    StructuredTask,
    DetachedTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureLifetime {
    Scoped(RegionId),
    Escaping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureMode {
    SharedBorrow,
    ExclusiveBorrow,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CaptureRequest {
    object: ObjectId,
    mode: CaptureMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CaptureEnvironment {
    kind: CaptureKind,
    lifetime: CaptureLifetime,
    tracing_roots: BTreeSet<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromotionResult {
    promoted_root: ObjectId,
    copies: BTreeMap<ObjectId, ObjectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelError {
    UnknownObject(ObjectId),
    UnknownRegion(RegionId),
    UnknownCapture(CaptureId),
    RegionAlreadyEnded(RegionId),
    RegionHasLiveChildren(RegionId),
    RegionHasLiveCaptures(RegionId),
    TracingToRegionEscape {
        source: ObjectId,
        target: ObjectId,
    },
    CrossRegionReference {
        source: ObjectId,
        target: ObjectId,
    },
    RegionCaptureEscapes {
        object: ObjectId,
        region: RegionId,
    },
    RegionDoesNotOutliveCapture {
        object: ObjectId,
        storage_region: RegionId,
        capture_region: RegionId,
    },
    InvalidCaptureLifetime(CaptureKind),
    PromotionBudgetExceeded {
        required: usize,
        budget: usize,
    },
    IdentifierExhausted,
    BorrowConflict(ObjectId),
    UnknownSharedBorrow {
        object: ObjectId,
        task: TaskId,
    },
    UnknownExclusiveBorrow {
        object: ObjectId,
        task: TaskId,
    },
}

#[derive(Debug, Default)]
struct MemoryModel {
    next_object: u64,
    next_region: u64,
    next_capture: u64,
    regions: BTreeMap<RegionId, Option<RegionId>>,
    objects: BTreeMap<ObjectId, Object>,
    roots: BTreeSet<ObjectId>,
    captures: BTreeMap<CaptureId, CaptureEnvironment>,
}

impl MemoryModel {
    fn new_region(&mut self) -> Result<RegionId, ModelError> {
        self.insert_region(None)
    }

    fn new_child_region(&mut self, parent: RegionId) -> Result<RegionId, ModelError> {
        if !self.regions.contains_key(&parent) {
            return Err(ModelError::UnknownRegion(parent));
        }
        self.insert_region(Some(parent))
    }

    fn insert_region(&mut self, parent: Option<RegionId>) -> Result<RegionId, ModelError> {
        let next = self.next_region.checked_add(1).ok_or(ModelError::IdentifierExhausted)?;
        let region = RegionId(next);
        self.next_region = next;
        self.regions.insert(region, parent);
        Ok(region)
    }

    fn region_outlives(&self, outer: RegionId, inner: RegionId) -> bool {
        let mut current = Some(inner);
        while let Some(region) = current {
            if region == outer {
                return true;
            }
            current = self.regions.get(&region).copied().flatten();
        }
        false
    }

    fn allocate_tracing(&mut self) -> Result<ObjectId, ModelError> {
        self.allocate(StorageDomain::Tracing)
    }

    fn allocate_in_region(&mut self, region: RegionId) -> Result<ObjectId, ModelError> {
        if !self.regions.contains_key(&region) {
            return Err(ModelError::UnknownRegion(region));
        }
        self.allocate(StorageDomain::Region(region))
    }

    fn allocate(&mut self, domain: StorageDomain) -> Result<ObjectId, ModelError> {
        let next = self.next_object.checked_add(1).ok_or(ModelError::IdentifierExhausted)?;
        let object = ObjectId(next);
        self.next_object = next;
        self.objects.insert(object, Object { domain, edges: BTreeSet::new() });
        Ok(object)
    }

    fn add_edge(&mut self, source: ObjectId, target: ObjectId) -> Result<(), ModelError> {
        let source_domain =
            self.objects.get(&source).ok_or(ModelError::UnknownObject(source))?.domain;
        let target_domain =
            self.objects.get(&target).ok_or(ModelError::UnknownObject(target))?.domain;

        match (source_domain, target_domain) {
            (StorageDomain::Tracing, StorageDomain::Region(_)) => {
                return Err(ModelError::TracingToRegionEscape { source, target });
            }
            (StorageDomain::Region(source_region), StorageDomain::Region(target_region))
                if !self.region_outlives(target_region, source_region) =>
            {
                return Err(ModelError::CrossRegionReference { source, target });
            }
            _ => {}
        }

        self.objects.get_mut(&source).expect("source checked").edges.insert(target);
        Ok(())
    }

    fn add_tracing_root(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let node = self.objects.get(&object).ok_or(ModelError::UnknownObject(object))?;
        if node.domain != StorageDomain::Tracing {
            let StorageDomain::Region(region) = node.domain else {
                unreachable!("non-tracing domain is a region");
            };
            return Err(ModelError::RegionCaptureEscapes { object, region });
        }
        self.roots.insert(object);
        Ok(())
    }

    /// Validates all captures before publishing the environment. Move capture
    /// does not change the storage domain: a moved region value still cannot
    /// outlive the region. Promotion must be an explicit operation.
    fn create_capture_environment(
        &mut self,
        kind: CaptureKind,
        lifetime: CaptureLifetime,
        requests: &[CaptureRequest],
    ) -> Result<CaptureId, ModelError> {
        match (kind, lifetime) {
            (CaptureKind::DetachedTask, CaptureLifetime::Scoped(_))
            | (CaptureKind::StructuredTask, CaptureLifetime::Escaping) => {
                return Err(ModelError::InvalidCaptureLifetime(kind));
            }
            _ => {}
        }

        if let CaptureLifetime::Scoped(region) = lifetime {
            if !self.regions.contains_key(&region) {
                return Err(ModelError::UnknownRegion(region));
            }
        }

        let mut tracing_roots = BTreeSet::new();
        let mut permissions: BTreeMap<ObjectId, (usize, bool, bool)> = BTreeMap::new();
        for request in requests {
            let permission = permissions.entry(request.object).or_default();
            match request.mode {
                CaptureMode::SharedBorrow => {
                    if permission.1 || permission.2 {
                        return Err(ModelError::BorrowConflict(request.object));
                    }
                    permission.0 += 1;
                }
                CaptureMode::ExclusiveBorrow => {
                    if permission.0 > 0 || permission.1 || permission.2 {
                        return Err(ModelError::BorrowConflict(request.object));
                    }
                    permission.1 = true;
                }
                CaptureMode::Move => {
                    if permission.0 > 0 || permission.1 || permission.2 {
                        return Err(ModelError::BorrowConflict(request.object));
                    }
                    permission.2 = true;
                }
            }
            let node = self
                .objects
                .get(&request.object)
                .ok_or(ModelError::UnknownObject(request.object))?;

            match node.domain {
                StorageDomain::Tracing => {
                    tracing_roots.insert(request.object);
                }
                StorageDomain::Region(storage_region) => match lifetime {
                    CaptureLifetime::Escaping => {
                        return Err(ModelError::RegionCaptureEscapes {
                            object: request.object,
                            region: storage_region,
                        });
                    }
                    CaptureLifetime::Scoped(capture_region) => {
                        if !self.regions.contains_key(&storage_region) {
                            return Err(ModelError::UnknownRegion(storage_region));
                        }
                        if !self.region_outlives(storage_region, capture_region) {
                            return Err(ModelError::RegionDoesNotOutliveCapture {
                                object: request.object,
                                storage_region,
                                capture_region,
                            });
                        }
                    }
                },
            }
        }

        let next = self.next_capture.checked_add(1).ok_or(ModelError::IdentifierExhausted)?;
        let id = CaptureId(next);
        self.next_capture = next;
        self.captures.insert(id, CaptureEnvironment { kind, lifetime, tracing_roots });
        Ok(id)
    }

    fn capture_kind(&self, capture: CaptureId) -> Result<CaptureKind, ModelError> {
        self.captures
            .get(&capture)
            .map(|environment| environment.kind)
            .ok_or(ModelError::UnknownCapture(capture))
    }

    fn release_capture_environment(&mut self, capture: CaptureId) -> Result<(), ModelError> {
        self.captures.remove(&capture).ok_or(ModelError::UnknownCapture(capture))?;
        Ok(())
    }

    /// Copies every reachable region-domain node into tracing storage.
    ///
    /// The graph copy preserves cycles and shared subgraphs. Existing tracing
    /// targets are retained as edges rather than copied. A budget failure is
    /// detected before any object is inserted or any identifier is consumed.
    fn promote_region_graph(
        &mut self,
        root: ObjectId,
        allocation_budget: usize,
    ) -> Result<PromotionResult, ModelError> {
        let root_domain = self.objects.get(&root).ok_or(ModelError::UnknownObject(root))?.domain;
        if root_domain == StorageDomain::Tracing {
            return Ok(PromotionResult { promoted_root: root, copies: BTreeMap::new() });
        }

        let mut region_nodes = BTreeSet::new();
        let mut pending = VecDeque::from([root]);
        while let Some(object) = pending.pop_front() {
            if !region_nodes.insert(object) {
                continue;
            }
            let node = self.objects.get(&object).ok_or(ModelError::UnknownObject(object))?;
            if let StorageDomain::Region(region) = node.domain {
                if !self.regions.contains_key(&region) {
                    return Err(ModelError::UnknownRegion(region));
                }
                for edge in &node.edges {
                    let target = self.objects.get(edge).ok_or(ModelError::UnknownObject(*edge))?;
                    if matches!(target.domain, StorageDomain::Region(_)) {
                        pending.push_back(*edge);
                    }
                }
            } else {
                return Err(ModelError::UnknownObject(object));
            }
        }

        if region_nodes.len() > allocation_budget {
            return Err(ModelError::PromotionBudgetExceeded {
                required: region_nodes.len(),
                budget: allocation_budget,
            });
        }

        let required =
            u64::try_from(region_nodes.len()).map_err(|_| ModelError::IdentifierExhausted)?;
        let final_id =
            self.next_object.checked_add(required).ok_or(ModelError::IdentifierExhausted)?;

        let mut copies = BTreeMap::new();
        let mut next = self.next_object;
        for original in &region_nodes {
            next += 1;
            copies.insert(*original, ObjectId(next));
        }

        let mut staged = Vec::with_capacity(region_nodes.len());
        for original in &region_nodes {
            let source = self.objects.get(original).ok_or(ModelError::UnknownObject(*original))?;
            let edges = source
                .edges
                .iter()
                .map(|target| copies.get(target).copied().unwrap_or(*target))
                .collect::<BTreeSet<_>>();
            staged.push((copies[original], Object { domain: StorageDomain::Tracing, edges }));
        }

        for (id, object) in staged {
            self.objects.insert(id, object);
        }
        self.next_object = final_id;

        Ok(PromotionResult {
            promoted_root: *copies.get(&root).ok_or(ModelError::UnknownObject(root))?,
            copies,
        })
    }

    fn end_region(&mut self, region: RegionId) -> Result<Vec<ObjectId>, ModelError> {
        if !self.regions.contains_key(&region) {
            return Err(ModelError::RegionAlreadyEnded(region));
        }
        if self.regions.values().any(|parent| *parent == Some(region)) {
            return Err(ModelError::RegionHasLiveChildren(region));
        }
        if self.captures.values().any(|capture| capture.lifetime == CaptureLifetime::Scoped(region))
        {
            return Err(ModelError::RegionHasLiveCaptures(region));
        }

        self.regions.remove(&region);
        let members = self
            .objects
            .iter()
            .filter_map(|(object, node)| {
                (node.domain == StorageDomain::Region(region)).then_some(*object)
            })
            .collect::<Vec<_>>();
        for object in &members {
            self.objects.remove(object);
        }
        Ok(members)
    }

    /// Region objects root their outgoing tracing edges while the region lives.
    /// Capture environments root their tracing captures independently.
    fn collect_tracing(&mut self) -> Vec<ObjectId> {
        let mut pending = VecDeque::from_iter(self.roots.iter().copied());
        for capture in self.captures.values() {
            pending.extend(capture.tracing_roots.iter().copied());
        }
        for node in self.objects.values() {
            if matches!(node.domain, StorageDomain::Region(region) if self.regions.contains_key(&region))
            {
                pending.extend(
                    node.edges
                        .iter()
                        .filter(|target| {
                            matches!(
                                self.objects.get(*target).map(|node| node.domain),
                                Some(StorageDomain::Tracing)
                            )
                        })
                        .copied(),
                );
            }
        }

        let mut live = BTreeSet::new();
        while let Some(object) = pending.pop_front() {
            if !live.insert(object) {
                continue;
            }
            if let Some(node) = self.objects.get(&object) {
                if node.domain == StorageDomain::Tracing {
                    pending.extend(node.edges.iter().copied());
                }
            }
        }

        let reclaimed = self
            .objects
            .iter()
            .filter_map(|(object, node)| {
                (node.domain == StorageDomain::Tracing && !live.contains(object)).then_some(*object)
            })
            .collect::<Vec<_>>();
        for object in &reclaimed {
            self.objects.remove(object);
        }
        self.roots.retain(|root| self.objects.contains_key(root));
        reclaimed
    }

    fn contains(&self, object: ObjectId) -> bool {
        self.objects.contains_key(&object)
    }

    fn object_edges(&self, object: ObjectId) -> Result<&BTreeSet<ObjectId>, ModelError> {
        self.objects.get(&object).map(|node| &node.edges).ok_or(ModelError::UnknownObject(object))
    }
}

#[derive(Debug, Default)]
struct TaskBorrowState {
    shared: BTreeSet<TaskId>,
    exclusive: Option<TaskId>,
}

/// Logical task-level access ledger. It is not a thread scheduler or memory
/// model; it only checks that shared and exclusive capture permissions do not
/// overlap in a way that violates the encoded aliasing discipline.
#[derive(Debug, Default)]
struct TaskAccessLedger {
    states: BTreeMap<ObjectId, TaskBorrowState>,
}

impl TaskAccessLedger {
    fn begin_shared(&mut self, object: ObjectId, task: TaskId) -> Result<(), ModelError> {
        let state = self.states.entry(object).or_default();
        if state.exclusive.is_some() {
            return Err(ModelError::BorrowConflict(object));
        }
        state.shared.insert(task);
        Ok(())
    }

    fn end_shared(&mut self, object: ObjectId, task: TaskId) -> Result<(), ModelError> {
        let state =
            self.states.get_mut(&object).ok_or(ModelError::UnknownSharedBorrow { object, task })?;
        if !state.shared.remove(&task) {
            return Err(ModelError::UnknownSharedBorrow { object, task });
        }
        if state.shared.is_empty() && state.exclusive.is_none() {
            self.states.remove(&object);
        }
        Ok(())
    }

    fn begin_exclusive(&mut self, object: ObjectId, task: TaskId) -> Result<(), ModelError> {
        let state = self.states.entry(object).or_default();
        if state.exclusive.is_some() || !state.shared.is_empty() {
            return Err(ModelError::BorrowConflict(object));
        }
        state.exclusive = Some(task);
        Ok(())
    }

    fn end_exclusive(&mut self, object: ObjectId, task: TaskId) -> Result<(), ModelError> {
        let state = self
            .states
            .get_mut(&object)
            .ok_or(ModelError::UnknownExclusiveBorrow { object, task })?;
        if state.exclusive != Some(task) {
            return Err(ModelError::UnknownExclusiveBorrow { object, task });
        }
        state.exclusive = None;
        if state.shared.is_empty() {
            self.states.remove(&object);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(object: ObjectId, mode: CaptureMode) -> CaptureRequest {
        CaptureRequest { object, mode }
    }

    #[test]
    fn closures_and_structured_tasks_may_capture_outliving_region_values() {
        let mut model = MemoryModel::default();
        let outer = model.new_region().expect("outer");
        let inner = model.new_child_region(outer).expect("inner");
        let outer_value = model.allocate_in_region(outer).expect("outer value");
        let inner_value = model.allocate_in_region(inner).expect("inner value");

        let closure = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Scoped(inner),
                &[
                    request(outer_value, CaptureMode::SharedBorrow),
                    request(inner_value, CaptureMode::Move),
                ],
            )
            .expect("inner closure may capture same/longer-lived region values");

        assert_eq!(model.capture_kind(closure), Ok(CaptureKind::Closure));

        let task = model
            .create_capture_environment(
                CaptureKind::StructuredTask,
                CaptureLifetime::Scoped(inner),
                &[request(outer_value, CaptureMode::SharedBorrow)],
            )
            .expect("structured task is bounded by its parent scope");

        assert_eq!(model.end_region(inner), Err(ModelError::RegionHasLiveCaptures(inner)));
        model.release_capture_environment(closure).expect("release closure");
        model.release_capture_environment(task).expect("join/release structured task");
        assert_eq!(model.end_region(inner), Ok(vec![inner_value]));
        assert_eq!(model.end_region(outer), Ok(vec![outer_value]));
    }

    #[test]
    fn closure_cannot_capture_a_shorter_lived_region_into_an_outer_scope() {
        let mut model = MemoryModel::default();
        let outer = model.new_region().expect("outer");
        let inner = model.new_child_region(outer).expect("inner");
        let inner_value = model.allocate_in_region(inner).expect("inner value");

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Scoped(outer),
                &[request(inner_value, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::RegionDoesNotOutliveCapture {
                object: inner_value,
                storage_region: inner,
                capture_region: outer,
            })
        );
    }

    #[test]
    fn detached_task_cannot_capture_region_storage_even_by_move() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let value = model.allocate_in_region(region).expect("region value");

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(value, CaptureMode::Move)],
            ),
            Err(ModelError::RegionCaptureEscapes { object: value, region })
        );
        assert!(model.contains(value));
    }

    #[test]
    fn one_capture_environment_cannot_alias_a_move_or_exclusive_capture() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let value = model.allocate_in_region(region).expect("value");

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Scoped(region),
                &[
                    request(value, CaptureMode::SharedBorrow),
                    request(value, CaptureMode::ExclusiveBorrow),
                ],
            ),
            Err(ModelError::BorrowConflict(value))
        );
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Scoped(region),
                &[request(value, CaptureMode::Move), request(value, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::BorrowConflict(value))
        );
        assert!(model.captures.is_empty(), "rejected capture must not publish an environment");
    }

    #[test]
    fn detached_tasks_can_capture_tracing_objects_until_environment_release() {
        let mut model = MemoryModel::default();
        let value = model.allocate_tracing().expect("tracing object");
        let detached = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(value, CaptureMode::Move)],
            )
            .expect("tracing object can be rooted by detached task");

        assert!(model.collect_tracing().is_empty());
        assert!(model.contains(value));

        model.release_capture_environment(detached).expect("task completion");
        assert_eq!(model.collect_tracing(), vec![value]);
    }

    #[test]
    fn structured_and_detached_task_lifetime_kinds_cannot_be_mismatched() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Scoped(region),
                &[],
            ),
            Err(ModelError::InvalidCaptureLifetime(CaptureKind::DetachedTask))
        );
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::StructuredTask,
                CaptureLifetime::Escaping,
                &[],
            ),
            Err(ModelError::InvalidCaptureLifetime(CaptureKind::StructuredTask))
        );
    }

    #[test]
    fn promotion_deep_copies_cycles_and_shared_subgraphs_into_tracing_domain() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let first = model.allocate_in_region(region).expect("first");
        let second = model.allocate_in_region(region).expect("second");
        let shared = model.allocate_in_region(region).expect("shared node");
        let existing_tracing = model.allocate_tracing().expect("existing tracing node");

        model.add_edge(first, second).expect("first -> second");
        model.add_edge(second, first).expect("second -> first cycle");
        model.add_edge(first, shared).expect("first -> shared");
        model.add_edge(second, shared).expect("second -> same shared node");
        model.add_edge(second, existing_tracing).expect("keep existing tracing edge");

        let promoted = model.promote_region_graph(first, 3).expect("promotion");
        assert_eq!(promoted.copies.len(), 3);

        let promoted_first = promoted.promoted_root;
        let promoted_second = promoted.copies[&second];
        let promoted_shared = promoted.copies[&shared];

        assert_eq!(
            model.object_edges(promoted_first).expect("promoted first"),
            &BTreeSet::from([promoted_second, promoted_shared])
        );
        assert_eq!(
            model.object_edges(promoted_second).expect("promoted second"),
            &BTreeSet::from([promoted_first, promoted_shared, existing_tracing])
        );
        assert!(model.object_edges(promoted_shared).expect("promoted shared").is_empty());

        model.add_tracing_root(promoted_first).expect("root promoted graph");
        assert_eq!(model.end_region(region), Ok(vec![first, second, shared]));
        assert!(model.contains(promoted_first));
        assert!(model.contains(promoted_second));
        assert!(model.contains(promoted_shared));
        assert!(model.contains(existing_tracing));

        model.roots.remove(&promoted_first);
        let reclaimed = model.collect_tracing().into_iter().collect::<BTreeSet<_>>();
        assert_eq!(
            reclaimed,
            BTreeSet::from([promoted_first, promoted_second, promoted_shared, existing_tracing])
        );
    }

    #[test]
    fn promotion_budget_failure_does_not_publish_partial_objects_or_consume_ids() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let first = model.allocate_in_region(region).expect("first");
        let second = model.allocate_in_region(region).expect("second");
        model.add_edge(first, second).expect("edge");

        let before_objects = model.objects.clone();
        let before_next_id = model.next_object;
        assert_eq!(
            model.promote_region_graph(first, 1),
            Err(ModelError::PromotionBudgetExceeded { required: 2, budget: 1 })
        );
        assert_eq!(model.objects, before_objects);
        assert_eq!(model.next_object, before_next_id);
    }

    #[test]
    fn promotion_of_existing_tracing_object_is_identity_and_allocation_free() {
        let mut model = MemoryModel::default();
        let tracing = model.allocate_tracing().expect("tracing");
        let before_next_id = model.next_object;
        let result = model.promote_region_graph(tracing, 0).expect("already tracing");

        assert_eq!(result.promoted_root, tracing);
        assert!(result.copies.is_empty());
        assert_eq!(model.next_object, before_next_id);
    }

    #[test]
    fn two_tasks_can_hold_shared_access_but_exclusive_access_is_serialized() {
        let object = ObjectId(7);
        let first = TaskId(1);
        let second = TaskId(2);
        let mut ledger = TaskAccessLedger::default();

        ledger.begin_shared(object, first).expect("task one shared");
        ledger.begin_shared(object, second).expect("task two shared");
        assert_eq!(
            ledger.begin_exclusive(object, TaskId(3)),
            Err(ModelError::BorrowConflict(object))
        );

        ledger.end_shared(object, first).expect("release task one");
        ledger.end_shared(object, second).expect("release task two");
        ledger.begin_exclusive(object, first).expect("exclusive after shared borrows end");
        assert_eq!(ledger.begin_shared(object, second), Err(ModelError::BorrowConflict(object)));
        assert_eq!(
            ledger.end_exclusive(object, second),
            Err(ModelError::UnknownExclusiveBorrow { object, task: second })
        );
        ledger.end_exclusive(object, first).expect("release exclusive");
    }

    #[test]
    fn duplicate_capture_environment_release_fails_closed() {
        let mut model = MemoryModel::default();
        let tracing = model.allocate_tracing().expect("tracing object");
        let capture = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(tracing, CaptureMode::SharedBorrow)],
            )
            .expect("capture");

        model.release_capture_environment(capture).expect("first release");
        assert_eq!(
            model.release_capture_environment(capture),
            Err(ModelError::UnknownCapture(capture))
        );
    }
}
