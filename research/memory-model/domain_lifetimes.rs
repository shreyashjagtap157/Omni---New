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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ResourceId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SourcePlaceId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageDomain {
    Tracing,
    Region(RegionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Object {
    domain: StorageDomain,
    edges: BTreeSet<ObjectId>,
    pinned: bool,
    external_resources: BTreeSet<ResourceId>,
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
    source_place: SourcePlaceId,
    object: ObjectId,
    mode: CaptureMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourcePlaceState {
    CallerOwned,
    Captured(CaptureId),
    Moved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CaptureEnvironment {
    kind: CaptureKind,
    lifetime: CaptureLifetime,
    tracing_roots: BTreeSet<ObjectId>,
    moved_source_places: BTreeSet<SourcePlaceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromotionResult {
    promoted_root: ObjectId,
    copies: BTreeMap<ObjectId, ObjectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelError {
    UnknownObject(ObjectId),
    UnknownResource(ResourceId),
    UnknownRegion(RegionId),
    UnknownCapture(CaptureId),
    SourcePlaceOwnedByCapture {
        place: SourcePlaceId,
        capture: CaptureId,
    },
    SourcePlaceMoved(SourcePlaceId),
    SourcePlaceBoundToDifferentObject {
        place: SourcePlaceId,
        existing: ObjectId,
        requested: ObjectId,
    },
    SourcePlaceOwnershipInvariant(SourcePlaceId),
    DuplicateResourceInTransfer(ResourceId),
    ResourceOwnedByCapture(ResourceId),
    ResourceOwnedByObject(ResourceId),
    ResourceNotAttachedToObject {
        object: ObjectId,
        resource: ResourceId,
    },
    ResourceAlreadyReleased(ResourceId),
    ObjectAlreadyPinned(ObjectId),
    ObjectNotPinned(ObjectId),
    CannotUnpinObjectWithExternalResources(ObjectId),
    PinnedObjectCannotBePromoted(ObjectId),
    ExternalResourceCannotBePromoted(ObjectId),
    RegionContainsPinnedObject {
        region: RegionId,
        object: ObjectId,
    },
    RegionContainsExternalResource {
        region: RegionId,
        object: ObjectId,
    },
    ResourceOwnershipInvariant(ResourceId),
    ResourceReleaseCountOverflow(ResourceId),
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
    capture_access: BTreeMap<ObjectId, BTreeMap<CaptureId, CaptureMode>>,
    source_places: BTreeMap<SourcePlaceId, (ObjectId, SourcePlaceState)>,
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
        self.objects.insert(
            object,
            Object {
                domain,
                edges: BTreeSet::new(),
                pinned: false,
                external_resources: BTreeSet::new(),
            },
        );
        Ok(object)
    }

    fn pin_object(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let node = self.objects.get_mut(&object).ok_or(ModelError::UnknownObject(object))?;
        if node.pinned {
            return Err(ModelError::ObjectAlreadyPinned(object));
        }
        node.pinned = true;
        Ok(())
    }

    fn unpin_object(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let node = self.objects.get_mut(&object).ok_or(ModelError::UnknownObject(object))?;
        if !node.pinned {
            return Err(ModelError::ObjectNotPinned(object));
        }
        if !node.external_resources.is_empty() {
            return Err(ModelError::CannotUnpinObjectWithExternalResources(object));
        }
        node.pinned = false;
        Ok(())
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
        let mut moved_source_places = BTreeSet::new();
        let mut requested_places = BTreeMap::new();
        let mut permissions: BTreeMap<ObjectId, (usize, bool, bool)> = BTreeMap::new();
        for request in requests {
            if let Some(existing) = requested_places.get(&request.source_place).copied() {
                if existing != request.object {
                    return Err(ModelError::SourcePlaceBoundToDifferentObject {
                        place: request.source_place,
                        existing,
                        requested: request.object,
                    });
                }
            } else {
                requested_places.insert(request.source_place, request.object);
            }

            match self.source_places.get(&request.source_place) {
                None => {}
                Some((existing, _)) if *existing != request.object => {
                    return Err(ModelError::SourcePlaceBoundToDifferentObject {
                        place: request.source_place,
                        existing: *existing,
                        requested: request.object,
                    });
                }
                Some((_, SourcePlaceState::CallerOwned)) => {}
                Some((_, SourcePlaceState::Captured(capture))) => {
                    return Err(ModelError::SourcePlaceOwnedByCapture {
                        place: request.source_place,
                        capture: *capture,
                    });
                }
                Some((_, SourcePlaceState::Moved)) => {
                    return Err(ModelError::SourcePlaceMoved(request.source_place));
                }
            }

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
                    moved_source_places.insert(request.source_place);
                }
            }
            let node = self
                .objects
                .get(&request.object)
                .ok_or(ModelError::UnknownObject(request.object))?;

            // A capture environment holds an access reservation for its entire
            // lifetime. Shared reservations may coexist; exclusive and move
            // reservations exclude every other live capture of this object.
            if let Some(active) = self.capture_access.get(&request.object) {
                let compatible = request.mode == CaptureMode::SharedBorrow
                    && active.values().all(|mode| *mode == CaptureMode::SharedBorrow);
                if !compatible {
                    return Err(ModelError::BorrowConflict(request.object));
                }
            }

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
        self.captures.insert(
            id,
            CaptureEnvironment {
                kind,
                lifetime,
                tracing_roots,
                moved_source_places: moved_source_places.clone(),
            },
        );
        for request in requests {
            self.source_places
                .entry(request.source_place)
                .or_insert((request.object, SourcePlaceState::CallerOwned));
            if request.mode == CaptureMode::Move {
                self.source_places
                    .insert(request.source_place, (request.object, SourcePlaceState::Captured(id)));
            }
            self.capture_access.entry(request.object).or_default().insert(id, request.mode);
        }
        Ok(id)
    }

    fn capture_kind(&self, capture: CaptureId) -> Result<CaptureKind, ModelError> {
        self.captures
            .get(&capture)
            .map(|environment| environment.kind)
            .ok_or(ModelError::UnknownCapture(capture))
    }

    fn release_capture_environment(&mut self, capture: CaptureId) -> Result<(), ModelError> {
        let environment = self.captures.get(&capture).ok_or(ModelError::UnknownCapture(capture))?;
        for place in &environment.moved_source_places {
            match self.source_places.get(place) {
                Some((_, SourcePlaceState::Captured(owner))) if *owner == capture => {}
                _ => return Err(ModelError::SourcePlaceOwnershipInvariant(*place)),
            }
        }

        let moved_places = environment.moved_source_places.clone();
        self.captures.remove(&capture).ok_or(ModelError::UnknownCapture(capture))?;
        for place in moved_places {
            let (_, state) = self
                .source_places
                .get_mut(&place)
                .ok_or(ModelError::SourcePlaceOwnershipInvariant(place))?;
            if *state != SourcePlaceState::Captured(capture) {
                return Err(ModelError::SourcePlaceOwnershipInvariant(place));
            }
            *state = SourcePlaceState::Moved;
            let _ = object;
        }
        for active in self.capture_access.values_mut() {
            active.remove(&capture);
        }
        self.capture_access.retain(|_, active| !active.is_empty());
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
                if !node.external_resources.is_empty() {
                    return Err(ModelError::ExternalResourceCannotBePromoted(object));
                }
                if node.pinned {
                    return Err(ModelError::PinnedObjectCannotBePromoted(object));
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
            staged.push((
                copies[original],
                Object {
                    domain: StorageDomain::Tracing,
                    edges,
                    pinned: false,
                    external_resources: BTreeSet::new(),
                },
            ));
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

        let members = self
            .objects
            .iter()
            .filter_map(|(object, node)| {
                (node.domain == StorageDomain::Region(region)).then_some(*object)
            })
            .collect::<Vec<_>>();
        for object in &members {
            let node = self.objects.get(object).expect("region member exists");
            if !node.external_resources.is_empty() {
                return Err(ModelError::RegionContainsExternalResource { region, object: *object });
            }
            if node.pinned {
                return Err(ModelError::RegionContainsPinnedObject { region, object: *object });
            }
        }

        self.regions.remove(&region);
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
        pending.extend(self.objects.iter().filter_map(|(object, node)| {
            (node.domain == StorageDomain::Tracing && node.pinned).then_some(*object)
        }));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureFinishReason {
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceOwner {
    Caller,
    Capture(CaptureId),
    Object(ObjectId),
    Released,
}

/// Explicit owner ledger for external resources. Ordinary data-object storage
/// may remain live after a resource is closed, and a task environment that owns
/// a moved resource is responsible for releasing it on every finish path.
#[derive(Debug, Default)]
struct ResourceLedger {
    next_resource: u64,
    owners: BTreeMap<ResourceId, ResourceOwner>,
    capture_resources: BTreeMap<CaptureId, BTreeSet<ResourceId>>,
    release_counts: BTreeMap<ResourceId, u32>,
    finish_reasons: BTreeMap<CaptureId, CaptureFinishReason>,
}

impl ResourceLedger {
    fn allocate(&mut self) -> Result<ResourceId, ModelError> {
        let next = self.next_resource.checked_add(1).ok_or(ModelError::IdentifierExhausted)?;
        let resource = ResourceId(next);
        self.next_resource = next;
        self.owners.insert(resource, ResourceOwner::Caller);
        self.release_counts.insert(resource, 0);
        Ok(resource)
    }

    /// Transfers a complete batch atomically. The caller retains ownership if
    /// any requested resource is unknown, already moved, or already released.
    fn transfer_to_capture(
        &mut self,
        model: &MemoryModel,
        capture: CaptureId,
        resources: &[ResourceId],
    ) -> Result<(), ModelError> {
        if !model.captures.contains_key(&capture) {
            return Err(ModelError::UnknownCapture(capture));
        }

        let requested = resources.iter().copied().collect::<BTreeSet<_>>();
        if requested.len() != resources.len() {
            let duplicate = resources
                .iter()
                .find(|resource| {
                    resources.iter().filter(|candidate| *candidate == *resource).count() > 1
                })
                .copied()
                .ok_or(ModelError::IdentifierExhausted)?;
            return Err(ModelError::DuplicateResourceInTransfer(duplicate));
        }

        for resource in &requested {
            match self.owners.get(resource).copied() {
                None => return Err(ModelError::UnknownResource(*resource)),
                Some(ResourceOwner::Caller) => {}
                Some(ResourceOwner::Capture(_)) => {
                    return Err(ModelError::ResourceOwnedByCapture(*resource));
                }
                Some(ResourceOwner::Object(_)) => {
                    return Err(ModelError::ResourceOwnedByObject(*resource));
                }
                Some(ResourceOwner::Released) => {
                    return Err(ModelError::ResourceAlreadyReleased(*resource));
                }
            }
        }

        for resource in &requested {
            self.owners.insert(*resource, ResourceOwner::Capture(capture));
        }
        self.capture_resources.entry(capture).or_default().extend(requested);
        Ok(())
    }

    /// Attaches an external resource to a stable object. Attachment transfers
    /// resource ownership to the object and pins the object's storage.
    fn attach_to_object(
        &mut self,
        model: &mut MemoryModel,
        object: ObjectId,
        resource: ResourceId,
    ) -> Result<(), ModelError> {
        let node = model.objects.get(&object).ok_or(ModelError::UnknownObject(object))?;
        if node.external_resources.contains(&resource) {
            return Err(ModelError::ResourceOwnershipInvariant(resource));
        }
        match self.owners.get(&resource).copied() {
            None => return Err(ModelError::UnknownResource(resource)),
            Some(ResourceOwner::Caller) => {}
            Some(ResourceOwner::Capture(_)) => {
                return Err(ModelError::ResourceOwnedByCapture(resource));
            }
            Some(ResourceOwner::Object(_)) => {
                return Err(ModelError::ResourceOwnedByObject(resource));
            }
            Some(ResourceOwner::Released) => {
                return Err(ModelError::ResourceAlreadyReleased(resource));
            }
        }

        let node = model.objects.get_mut(&object).expect("object validated");
        node.external_resources.insert(resource);
        node.pinned = true;
        self.owners.insert(resource, ResourceOwner::Object(object));
        Ok(())
    }

    /// Deterministically releases a resource attached to an object. The object
    /// remains pinned until a separate unpin transition succeeds.
    fn release_object_resource(
        &mut self,
        model: &mut MemoryModel,
        object: ObjectId,
        resource: ResourceId,
    ) -> Result<(), ModelError> {
        let node = model.objects.get(&object).ok_or(ModelError::UnknownObject(object))?;
        if !node.external_resources.contains(&resource) {
            return Err(ModelError::ResourceNotAttachedToObject { object, resource });
        }
        if self.owners.get(&resource) != Some(&ResourceOwner::Object(object)) {
            return Err(ModelError::ResourceOwnershipInvariant(resource));
        }
        let count = self
            .release_counts
            .get(&resource)
            .copied()
            .ok_or(ModelError::ResourceOwnershipInvariant(resource))?;
        count.checked_add(1).ok_or(ModelError::ResourceReleaseCountOverflow(resource))?;

        model
            .objects
            .get_mut(&object)
            .expect("object validated")
            .external_resources
            .remove(&resource);
        self.release_once(resource)?;
        Ok(())
    }

    /// Caller-side release is permitted only before ownership is transferred.
    fn release_by_caller(&mut self, resource: ResourceId) -> Result<(), ModelError> {
        match self.owners.get(&resource).copied() {
            None => return Err(ModelError::UnknownResource(resource)),
            Some(ResourceOwner::Capture(_)) => {
                return Err(ModelError::ResourceOwnedByCapture(resource));
            }
            Some(ResourceOwner::Object(_)) => {
                return Err(ModelError::ResourceOwnedByObject(resource));
            }
            Some(ResourceOwner::Released) => {
                return Err(ModelError::ResourceAlreadyReleased(resource));
            }
            Some(ResourceOwner::Caller) => {}
        }
        self.release_once(resource)?;
        Ok(())
    }

    /// A task/closure environment releases every resource it owns when it
    /// completes, is cancelled, or fails. State is validated before mutation.
    fn finish_capture(
        &mut self,
        model: &mut MemoryModel,
        capture: CaptureId,
        reason: CaptureFinishReason,
    ) -> Result<Vec<ResourceId>, ModelError> {
        if !model.captures.contains_key(&capture) {
            return Err(ModelError::UnknownCapture(capture));
        }

        let resources = self.capture_resources.get(&capture).cloned().unwrap_or_default();
        for resource in &resources {
            if self.owners.get(resource) != Some(&ResourceOwner::Capture(capture)) {
                return Err(ModelError::ResourceOwnershipInvariant(*resource));
            }
            let count = self
                .release_counts
                .get(resource)
                .copied()
                .ok_or(ModelError::ResourceOwnershipInvariant(*resource))?;
            if count.checked_add(1).is_none() {
                return Err(ModelError::ResourceReleaseCountOverflow(*resource));
            }
        }

        for resource in &resources {
            self.release_once(*resource)?;
        }
        self.capture_resources.remove(&capture);
        self.finish_reasons.insert(capture, reason);
        model.release_capture_environment(capture)?;
        Ok(resources.into_iter().collect())
    }

    fn release_once(&mut self, resource: ResourceId) -> Result<(), ModelError> {
        let owner =
            self.owners.get(&resource).copied().ok_or(ModelError::UnknownResource(resource))?;
        if owner == ResourceOwner::Released {
            return Err(ModelError::ResourceAlreadyReleased(resource));
        }
        let count = self
            .release_counts
            .get(&resource)
            .copied()
            .ok_or(ModelError::ResourceOwnershipInvariant(resource))?;
        let next =
            count.checked_add(1).ok_or(ModelError::ResourceReleaseCountOverflow(resource))?;
        self.owners.insert(resource, ResourceOwner::Released);
        self.release_counts.insert(resource, next);
        Ok(())
    }

    fn owner(&self, resource: ResourceId) -> Result<ResourceOwner, ModelError> {
        self.owners.get(&resource).copied().ok_or(ModelError::UnknownResource(resource))
    }

    fn release_count(&self, resource: ResourceId) -> Result<u32, ModelError> {
        self.release_counts.get(&resource).copied().ok_or(ModelError::UnknownResource(resource))
    }

    fn finish_reason(&self, capture: CaptureId) -> Result<CaptureFinishReason, ModelError> {
        self.finish_reasons.get(&capture).copied().ok_or(ModelError::UnknownCapture(capture))
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
        CaptureRequest { source_place: SourcePlaceId(object.0), object, mode }
    }

    fn request_at(
        source_place: SourcePlaceId,
        object: ObjectId,
        mode: CaptureMode,
    ) -> CaptureRequest {
        CaptureRequest { source_place, object, mode }
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
    fn capture_access_reservations_conflict_across_live_environments() {
        let mut model = MemoryModel::default();
        let object = model.allocate_tracing().expect("tracing object");

        let first_shared = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::SharedBorrow)],
            )
            .expect("first shared capture");
        let second_shared = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::SharedBorrow)],
            )
            .expect("second shared capture may coexist");

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::ExclusiveBorrow)],
            ),
            Err(ModelError::BorrowConflict(object))
        );
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::Move)],
            ),
            Err(ModelError::BorrowConflict(object))
        );

        model.release_capture_environment(first_shared).expect("release first shared capture");
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::ExclusiveBorrow)],
            ),
            Err(ModelError::BorrowConflict(object)),
            "the second shared capture must continue reserving shared access"
        );

        model.release_capture_environment(second_shared).expect("release second shared capture");
        let exclusive = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::ExclusiveBorrow)],
            )
            .expect("exclusive capture after all shared reservations end");
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::BorrowConflict(object))
        );
        model.release_capture_environment(exclusive).expect("release exclusive capture");

        let moved = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::Move)],
            )
            .expect("move capture after previous reservations end");
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::BorrowConflict(object))
        );
        model.release_capture_environment(moved).expect("release move reservation");
    }

    #[test]
    fn rejected_cross_environment_capture_does_not_publish_or_consume_an_id() {
        let mut model = MemoryModel::default();
        let object = model.allocate_tracing().expect("tracing object");
        let active = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::Move)],
            )
            .expect("move capture");
        let next_before = model.next_capture;
        let access_before = model.capture_access.clone();

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::BorrowConflict(object))
        );
        assert_eq!(model.next_capture, next_before);
        assert_eq!(model.capture_access, access_before);
        assert_eq!(model.captures.len(), 1);

        model.release_capture_environment(active).expect("release active capture");
        let next = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::SharedBorrow)],
            )
            .expect("create after release");
        assert_eq!(next.0, next_before + 1);
    }

    #[test]
    fn repeated_shared_aliases_in_one_capture_release_one_environment_reservation() {
        let mut model = MemoryModel::default();
        let object = model.allocate_tracing().expect("tracing object");
        let capture = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[
                    request(object, CaptureMode::SharedBorrow),
                    request(object, CaptureMode::SharedBorrow),
                ],
            )
            .expect("two shared aliases in one environment");
        assert_eq!(model.capture_access[&object].len(), 1);
        model.release_capture_environment(capture).expect("release environment");
        assert!(!model.capture_access.contains_key(&object));
    }

    #[test]
    fn move_capture_invalidates_the_source_place_and_it_stays_moved_after_completion() {
        let mut model = MemoryModel::default();
        let object = model.allocate_tracing().expect("tracing object");
        let source = SourcePlaceId(700);
        let capture = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request_at(source, object, CaptureMode::Move)],
            )
            .expect("move source place into task");
        assert_eq!(
            model.source_places.get(&source),
            Some(&(object, SourcePlaceState::Captured(capture)))
        );

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request_at(source, object, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::SourcePlaceOwnedByCapture { place: source, capture })
        );

        model.release_capture_environment(capture).expect("task completion drops moved value");
        assert_eq!(model.source_places.get(&source), Some(&(object, SourcePlaceState::Moved)));
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request_at(source, object, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::SourcePlaceMoved(source)),
            "completion does not resurrect the moved source place"
        );
        assert_eq!(model.collect_tracing(), vec![object]);
    }

    #[test]
    fn one_capture_batch_cannot_bind_one_source_place_to_two_objects() {
        let mut model = MemoryModel::default();
        let first = model.allocate_tracing().expect("first object");
        let second = model.allocate_tracing().expect("second object");
        let source = SourcePlaceId(699);
        let next_before = model.next_capture;

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[
                    request_at(source, first, CaptureMode::SharedBorrow),
                    request_at(source, second, CaptureMode::SharedBorrow),
                ],
            ),
            Err(ModelError::SourcePlaceBoundToDifferentObject {
                place: source,
                existing: first,
                requested: second,
            })
        );
        assert_eq!(model.next_capture, next_before);
        assert!(model.source_places.is_empty());
        assert!(model.capture_access.is_empty());
        assert!(model.captures.is_empty());
    }

    #[test]
    fn failed_move_capture_does_not_invalidate_any_source_place_or_consume_capture_id() {
        let mut model = MemoryModel::default();
        let first = model.allocate_tracing().expect("first object");
        let second = model.allocate_tracing().expect("second object");
        let first_place = SourcePlaceId(701);
        let second_place = SourcePlaceId(702);
        let blocking = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request_at(second_place, second, CaptureMode::SharedBorrow)],
            )
            .expect("existing shared capture");
        let next_before = model.next_capture;

        assert_eq!(
            model.create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[
                    request_at(first_place, first, CaptureMode::Move),
                    request_at(second_place, second, CaptureMode::Move),
                ],
            ),
            Err(ModelError::BorrowConflict(second))
        );
        assert_eq!(model.next_capture, next_before);
        assert_eq!(model.source_places.get(&first_place), None);
        assert_eq!(
            model.source_places.get(&second_place),
            Some(&(second, SourcePlaceState::CallerOwned))
        );

        model.release_capture_environment(blocking).expect("release blocker");
        let moved = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[
                    request_at(first_place, first, CaptureMode::Move),
                    request_at(second_place, second, CaptureMode::Move),
                ],
            )
            .expect("retry move after conflict ends");
        assert_eq!(
            model.source_places.get(&first_place),
            Some(&(first, SourcePlaceState::Captured(moved)))
        );
        assert_eq!(
            model.source_places.get(&second_place),
            Some(&(second, SourcePlaceState::Captured(moved)))
        );
    }

    #[test]
    fn move_of_one_handle_does_not_invalidate_a_distinct_source_place_alias() {
        let mut model = MemoryModel::default();
        let object = model.allocate_tracing().expect("tracing object");
        let moved_place = SourcePlaceId(703);
        let alias_place = SourcePlaceId(704);

        let capture = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request_at(moved_place, object, CaptureMode::Move)],
            )
            .expect("move one source handle");
        assert_eq!(
            model.create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request_at(alias_place, object, CaptureMode::SharedBorrow)],
            ),
            Err(ModelError::BorrowConflict(object)),
            "the object access reservation prevents a concurrent conflicting alias"
        );

        model.release_capture_environment(capture).expect("finish moved handle");
        let shared = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Escaping,
                &[request_at(alias_place, object, CaptureMode::SharedBorrow)],
            )
            .expect("distinct source place remains caller-owned after moved handle ends");
        assert_eq!(model.source_places.get(&moved_place), Some(&(object, SourcePlaceState::Moved)));
        assert_eq!(
            model.source_places.get(&alias_place),
            Some(&(object, SourcePlaceState::CallerOwned))
        );
        model.release_capture_environment(shared).expect("release alias borrow");
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
    fn moved_resource_is_released_once_when_capture_completes() {
        let mut model = MemoryModel::default();
        let capture = model
            .create_capture_environment(CaptureKind::Closure, CaptureLifetime::Escaping, &[])
            .expect("escaping closure environment");
        let mut resources = ResourceLedger::default();
        let resource = resources.allocate().expect("resource");

        resources
            .transfer_to_capture(&model, capture, &[resource])
            .expect("move resource into closure");
        assert_eq!(resources.owner(resource), Ok(ResourceOwner::Capture(capture)));
        assert_eq!(
            resources.release_by_caller(resource),
            Err(ModelError::ResourceOwnedByCapture(resource))
        );

        assert_eq!(
            resources.finish_capture(&mut model, capture, CaptureFinishReason::Completed),
            Ok(vec![resource])
        );
        assert_eq!(resources.owner(resource), Ok(ResourceOwner::Released));
        assert_eq!(resources.release_count(resource), Ok(1));
        assert_eq!(resources.finish_reason(capture), Ok(CaptureFinishReason::Completed));
        assert_eq!(
            resources.finish_capture(&mut model, capture, CaptureFinishReason::Completed),
            Err(ModelError::UnknownCapture(capture))
        );
        assert_eq!(resources.release_count(resource), Ok(1));
    }

    #[test]
    fn cancellation_and_failure_release_captured_resources_exactly_once() {
        for reason in [CaptureFinishReason::Cancelled, CaptureFinishReason::Failed] {
            let mut model = MemoryModel::default();
            let capture = model
                .create_capture_environment(
                    CaptureKind::DetachedTask,
                    CaptureLifetime::Escaping,
                    &[],
                )
                .expect("detached task environment");
            let mut resources = ResourceLedger::default();
            let first = resources.allocate().expect("first resource");
            let second = resources.allocate().expect("second resource");
            resources
                .transfer_to_capture(&model, capture, &[first, second])
                .expect("transfer resources");

            let released =
                resources.finish_capture(&mut model, capture, reason).expect("finish task");
            assert_eq!(released, vec![first, second]);
            assert_eq!(resources.release_count(first), Ok(1));
            assert_eq!(resources.release_count(second), Ok(1));
            assert_eq!(resources.finish_reason(capture), Ok(reason));
            assert_eq!(resources.owner(first), Ok(ResourceOwner::Released));
            assert_eq!(resources.owner(second), Ok(ResourceOwner::Released));
        }
    }

    #[test]
    fn failed_batch_resource_transfer_is_atomic() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let capture = model
            .create_capture_environment(
                CaptureKind::StructuredTask,
                CaptureLifetime::Scoped(region),
                &[],
            )
            .expect("structured task environment");
        let mut resources = ResourceLedger::default();
        let caller_owned = resources.allocate().expect("caller-owned resource");
        let already_released = resources.allocate().expect("second resource");
        resources.release_by_caller(already_released).expect("release second resource");

        assert_eq!(
            resources.transfer_to_capture(&model, capture, &[caller_owned, already_released]),
            Err(ModelError::ResourceAlreadyReleased(already_released))
        );
        assert_eq!(resources.owner(caller_owned), Ok(ResourceOwner::Caller));
        assert_eq!(resources.release_count(caller_owned), Ok(0));
        assert_eq!(
            resources.transfer_to_capture(&model, capture, &[caller_owned, caller_owned]),
            Err(ModelError::DuplicateResourceInTransfer(caller_owned))
        );
        assert_eq!(resources.owner(caller_owned), Ok(ResourceOwner::Caller));
    }

    #[test]
    fn resource_cleanup_and_traced_storage_reclamation_are_independent() {
        let mut model = MemoryModel::default();
        let object = model.allocate_tracing().expect("tracing object");
        model.add_tracing_root(object).expect("external root");
        let capture = model
            .create_capture_environment(
                CaptureKind::DetachedTask,
                CaptureLifetime::Escaping,
                &[request(object, CaptureMode::SharedBorrow)],
            )
            .expect("capture tracing object");
        let mut resources = ResourceLedger::default();
        let resource = resources.allocate().expect("external resource");
        resources.transfer_to_capture(&model, capture, &[resource]).expect("resource transfer");

        resources
            .finish_capture(&mut model, capture, CaptureFinishReason::Cancelled)
            .expect("cancel task and release external resource");
        assert_eq!(resources.release_count(resource), Ok(1));
        assert!(model.contains(object), "object storage stays live through independent root");

        model.roots.remove(&object);
        assert_eq!(model.collect_tracing(), vec![object]);
        assert_eq!(
            resources.release_count(resource),
            Ok(1),
            "storage collection must not release the resource twice"
        );
    }

    #[test]
    fn pinned_objects_cannot_be_promoted_or_ended_until_explicitly_unpinned() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let object = model.allocate_in_region(region).expect("region object");
        model.pin_object(object).expect("pin object");

        assert_eq!(
            model.promote_region_graph(object, 1),
            Err(ModelError::PinnedObjectCannotBePromoted(object))
        );
        assert_eq!(
            model.end_region(region),
            Err(ModelError::RegionContainsPinnedObject { region, object })
        );
        assert!(model.contains(object));

        model.unpin_object(object).expect("unpin after external use ends");
        let promoted = model.promote_region_graph(object, 1).expect("promotion after unpin");
        assert!(model.contains(promoted.promoted_root));
        assert_eq!(model.end_region(region), Ok(vec![object]));
    }

    #[test]
    fn attached_external_resource_blocks_copy_and_region_teardown_until_closed_and_unpinned() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let object = model.allocate_in_region(region).expect("region object");
        let mut resources = ResourceLedger::default();
        let resource = resources.allocate().expect("FFI/DMA resource");
        resources.attach_to_object(&mut model, object, resource).expect("attach external resource");

        assert_eq!(resources.owner(resource), Ok(ResourceOwner::Object(object)));
        assert_eq!(
            resources.release_by_caller(resource),
            Err(ModelError::ResourceOwnedByObject(resource))
        );
        let capture = model
            .create_capture_environment(
                CaptureKind::Closure,
                CaptureLifetime::Scoped(region),
                &[request(object, CaptureMode::SharedBorrow)],
            )
            .expect("scoped capture");
        assert_eq!(
            resources.transfer_to_capture(&model, capture, &[resource]),
            Err(ModelError::ResourceOwnedByObject(resource))
        );
        model.release_capture_environment(capture).expect("release scoped capture");
        assert_eq!(
            model.promote_region_graph(object, 1),
            Err(ModelError::ExternalResourceCannotBePromoted(object))
        );
        assert_eq!(
            model.end_region(region),
            Err(ModelError::RegionContainsExternalResource { region, object })
        );
        assert_eq!(resources.release_count(resource), Ok(0));

        resources
            .release_object_resource(&mut model, object, resource)
            .expect("close external resource");
        assert_eq!(resources.owner(resource), Ok(ResourceOwner::Released));
        assert_eq!(resources.release_count(resource), Ok(1));

        assert_eq!(
            model.promote_region_graph(object, 1),
            Err(ModelError::PinnedObjectCannotBePromoted(object)),
            "closing the resource does not implicitly unpin its storage"
        );
        assert_eq!(
            model.end_region(region),
            Err(ModelError::RegionContainsPinnedObject { region, object })
        );

        model.unpin_object(object).expect("explicit unpin");
        let promoted =
            model.promote_region_graph(object, 1).expect("promote after close and unpin");
        assert!(model.contains(promoted.promoted_root));
        assert_eq!(model.end_region(region), Ok(vec![object]));
        assert_eq!(resources.release_count(resource), Ok(1));
    }

    #[test]
    fn pinned_tracing_objects_are_gc_roots_until_resource_close_and_unpin() {
        let mut model = MemoryModel::default();
        let object = model.allocate_tracing().expect("tracing object");
        let mut resources = ResourceLedger::default();
        let resource = resources.allocate().expect("device resource");
        resources.attach_to_object(&mut model, object, resource).expect("attach resource");

        assert!(model.collect_tracing().is_empty());
        assert!(model.contains(object));

        resources.release_object_resource(&mut model, object, resource).expect("close resource");
        assert!(model.collect_tracing().is_empty());
        assert!(model.contains(object), "closed resource remains pinned until explicit unpin");

        model.unpin_object(object).expect("unpin");
        assert_eq!(model.collect_tracing(), vec![object]);
        assert_eq!(resources.release_count(resource), Ok(1));
    }

    #[test]
    fn object_resource_attachment_is_atomic_for_unknown_or_non_caller_owned_resources() {
        let mut model = MemoryModel::default();
        let region = model.new_region().expect("region");
        let object = model.allocate_in_region(region).expect("object");
        let capture = model
            .create_capture_environment(CaptureKind::Closure, CaptureLifetime::Scoped(region), &[])
            .expect("capture");
        let mut resources = ResourceLedger::default();
        let resource = resources.allocate().expect("resource");
        resources.transfer_to_capture(&model, capture, &[resource]).expect("transfer");

        assert_eq!(
            resources.attach_to_object(&mut model, object, resource),
            Err(ModelError::ResourceOwnedByCapture(resource))
        );
        assert_eq!(resources.owner(resource), Ok(ResourceOwner::Capture(capture)));
        assert!(!model.objects[&object].pinned);
        assert!(model.objects[&object].external_resources.is_empty());
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
