//! Isolated research model for memory reclamation/access-policy separation.
//!
//! Not a production allocator, collector, type checker, or borrow checker.
//! These tests exercise only the transitions encoded by this small model.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ObjectId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RegionId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelError {
    UnknownObject(ObjectId),
    UnknownRegion(RegionId),
    RegionAlreadyEnded(RegionId),
    RootNotHeld(ObjectId),
    BorrowConflict(ObjectId),
    NoActiveSharedBorrow(ObjectId),
    NoActiveMutableBorrow(ObjectId),
    ReferenceCountUnderflow(ObjectId),
    TracingRootCannotReferenceRegion(ObjectId),
    TracingToRegionEscape { source: ObjectId, target: ObjectId },
    CrossRegionReference { source: ObjectId, target: ObjectId },
}

#[derive(Debug, Default)]
struct BorrowState {
    shared: usize,
    mutable: bool,
}

#[derive(Debug, Default)]
struct AccessLedger {
    states: BTreeMap<ObjectId, BorrowState>,
}

impl AccessLedger {
    fn begin_shared(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let state = self.states.entry(object).or_default();
        if state.mutable {
            return Err(ModelError::BorrowConflict(object));
        }
        state.shared += 1;
        Ok(())
    }

    fn end_shared(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let state = self.states.get_mut(&object).ok_or(ModelError::NoActiveSharedBorrow(object))?;
        if state.shared == 0 {
            return Err(ModelError::NoActiveSharedBorrow(object));
        }
        state.shared -= 1;
        if state.shared == 0 && !state.mutable {
            self.states.remove(&object);
        }
        Ok(())
    }

    fn begin_mutable(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let state = self.states.entry(object).or_default();
        if state.mutable || state.shared != 0 {
            return Err(ModelError::BorrowConflict(object));
        }
        state.mutable = true;
        Ok(())
    }

    fn end_mutable(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let state =
            self.states.get_mut(&object).ok_or(ModelError::NoActiveMutableBorrow(object))?;
        if !state.mutable {
            return Err(ModelError::NoActiveMutableBorrow(object));
        }
        state.mutable = false;
        if state.shared == 0 {
            self.states.remove(&object);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RegionHeap {
    next_object: u64,
    next_region: u64,
    live_regions: BTreeSet<RegionId>,
    objects: BTreeMap<ObjectId, RegionId>,
}

impl Default for RegionHeap {
    fn default() -> Self {
        Self {
            next_object: 1,
            next_region: 1,
            live_regions: BTreeSet::new(),
            objects: BTreeMap::new(),
        }
    }
}

impl RegionHeap {
    fn new_region(&mut self) -> RegionId {
        let region = RegionId(self.next_region);
        self.next_region += 1;
        self.live_regions.insert(region);
        region
    }

    fn allocate(&mut self, region: RegionId) -> Result<ObjectId, ModelError> {
        if !self.live_regions.contains(&region) {
            return Err(ModelError::UnknownRegion(region));
        }
        let object = ObjectId(self.next_object);
        self.next_object += 1;
        self.objects.insert(object, region);
        Ok(object)
    }

    fn end_region(&mut self, region: RegionId) -> Result<Vec<ObjectId>, ModelError> {
        if !self.live_regions.remove(&region) {
            return Err(ModelError::RegionAlreadyEnded(region));
        }
        let reclaimed = self
            .objects
            .iter()
            .filter_map(|(object, owner)| (*owner == region).then_some(*object))
            .collect::<Vec<_>>();
        for object in &reclaimed {
            self.objects.remove(object);
        }
        Ok(reclaimed)
    }

    fn contains(&self, object: ObjectId) -> bool {
        self.objects.contains_key(&object)
    }
}

#[derive(Debug, Default)]
struct TraceNode {
    edges: BTreeSet<ObjectId>,
}

#[derive(Debug, Default)]
struct TraceHeap {
    next_object: u64,
    objects: BTreeMap<ObjectId, TraceNode>,
    roots: BTreeSet<ObjectId>,
}

impl TraceHeap {
    fn allocate(&mut self) -> ObjectId {
        let object = ObjectId(self.next_object + 1);
        self.next_object += 1;
        self.objects.insert(object, TraceNode::default());
        object
    }

    fn add_root(&mut self, object: ObjectId) -> Result<(), ModelError> {
        if !self.objects.contains_key(&object) {
            return Err(ModelError::UnknownObject(object));
        }
        self.roots.insert(object);
        Ok(())
    }

    fn remove_root(&mut self, object: ObjectId) {
        self.roots.remove(&object);
    }

    fn add_edge(&mut self, from: ObjectId, to: ObjectId) -> Result<(), ModelError> {
        if !self.objects.contains_key(&from) {
            return Err(ModelError::UnknownObject(from));
        }
        if !self.objects.contains_key(&to) {
            return Err(ModelError::UnknownObject(to));
        }
        self.objects.get_mut(&from).expect("validated source").edges.insert(to);
        Ok(())
    }

    fn collect(&mut self) -> Vec<ObjectId> {
        let mut live = BTreeSet::new();
        let mut pending = VecDeque::from_iter(self.roots.iter().copied());

        while let Some(object) = pending.pop_front() {
            if !live.insert(object) {
                continue;
            }
            if let Some(node) = self.objects.get(&object) {
                pending.extend(node.edges.iter().copied());
            }
        }

        let reclaimed = self
            .objects
            .keys()
            .filter(|object| !live.contains(object))
            .copied()
            .collect::<Vec<_>>();
        for object in &reclaimed {
            self.objects.remove(object);
        }
        reclaimed
    }

    fn contains(&self, object: ObjectId) -> bool {
        self.objects.contains_key(&object)
    }

    fn len(&self) -> usize {
        self.objects.len()
    }
}

#[derive(Debug, Default)]
struct RcNode {
    strong_count: usize,
    edges: BTreeSet<ObjectId>,
}

#[derive(Debug, Default)]
struct RcHeap {
    next_object: u64,
    objects: BTreeMap<ObjectId, RcNode>,
    roots: BTreeMap<ObjectId, usize>,
}

impl RcHeap {
    fn allocate_root(&mut self) -> ObjectId {
        let object = ObjectId(self.next_object + 1);
        self.next_object += 1;
        self.objects.insert(object, RcNode { strong_count: 1, edges: BTreeSet::new() });
        self.roots.insert(object, 1);
        object
    }

    fn clone_root(&mut self, object: ObjectId) -> Result<(), ModelError> {
        if !self.objects.contains_key(&object) {
            return Err(ModelError::UnknownObject(object));
        }
        *self.roots.entry(object).or_default() += 1;
        self.objects.get_mut(&object).expect("validated object").strong_count += 1;
        Ok(())
    }

    fn add_edge(&mut self, from: ObjectId, to: ObjectId) -> Result<(), ModelError> {
        if !self.objects.contains_key(&from) {
            return Err(ModelError::UnknownObject(from));
        }
        if !self.objects.contains_key(&to) {
            return Err(ModelError::UnknownObject(to));
        }
        let inserted = self.objects.get_mut(&from).expect("validated source").edges.insert(to);
        if inserted {
            self.objects.get_mut(&to).expect("validated target").strong_count += 1;
        }
        Ok(())
    }

    fn release_root(&mut self, object: ObjectId) -> Result<Vec<ObjectId>, ModelError> {
        let root_count = self.roots.get_mut(&object).ok_or(ModelError::RootNotHeld(object))?;
        if *root_count == 0 {
            return Err(ModelError::RootNotHeld(object));
        }
        *root_count -= 1;
        if *root_count == 0 {
            self.roots.remove(&object);
        }
        self.decrement_and_collect(object)
    }

    fn decrement_and_collect(&mut self, object: ObjectId) -> Result<Vec<ObjectId>, ModelError> {
        let node = self.objects.get_mut(&object).ok_or(ModelError::UnknownObject(object))?;
        if node.strong_count == 0 {
            return Err(ModelError::ReferenceCountUnderflow(object));
        }
        node.strong_count -= 1;

        let mut pending = VecDeque::from([object]);
        let mut reclaimed = Vec::new();
        while let Some(candidate) = pending.pop_front() {
            let dead = self.objects.get(&candidate).is_some_and(|node| node.strong_count == 0);
            if !dead {
                continue;
            }
            let removed = self.objects.remove(&candidate).expect("dead object exists");
            reclaimed.push(candidate);
            for child in removed.edges {
                let child_node =
                    self.objects.get_mut(&child).ok_or(ModelError::UnknownObject(child))?;
                if child_node.strong_count == 0 {
                    return Err(ModelError::ReferenceCountUnderflow(child));
                }
                child_node.strong_count -= 1;
                if child_node.strong_count == 0 {
                    pending.push_back(child);
                }
            }
        }
        Ok(reclaimed)
    }

    fn contains(&self, object: ObjectId) -> bool {
        self.objects.contains_key(&object)
    }

    fn len(&self) -> usize {
        self.objects.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageDomain {
    Tracing,
    Region(RegionId),
}

#[derive(Debug, Default)]
struct MixedDomainNode {
    domain: Option<StorageDomain>,
    edges: BTreeSet<ObjectId>,
}

/// A conservative mixed-region/tracing model. Active region objects are roots
/// for tracing objects they reference. Longer-lived tracing objects may not
/// retain shorter-lived region objects. Nested-region outlives relations are
/// intentionally not modeled: references between distinct regions are rejected.
#[derive(Debug, Default)]
struct MixedDomainHeap {
    next_object: u64,
    next_region: u64,
    live_regions: BTreeSet<RegionId>,
    objects: BTreeMap<ObjectId, MixedDomainNode>,
    tracing_roots: BTreeSet<ObjectId>,
}

impl MixedDomainHeap {
    fn new_region(&mut self) -> RegionId {
        let region = RegionId(self.next_region + 1);
        self.next_region += 1;
        self.live_regions.insert(region);
        region
    }

    fn allocate_tracing(&mut self) -> ObjectId {
        let object = ObjectId(self.next_object + 1);
        self.next_object += 1;
        self.objects.insert(
            object,
            MixedDomainNode { domain: Some(StorageDomain::Tracing), edges: BTreeSet::new() },
        );
        object
    }

    fn allocate_in_region(&mut self, region: RegionId) -> Result<ObjectId, ModelError> {
        if !self.live_regions.contains(&region) {
            return Err(ModelError::UnknownRegion(region));
        }
        let object = ObjectId(self.next_object + 1);
        self.next_object += 1;
        self.objects.insert(
            object,
            MixedDomainNode { domain: Some(StorageDomain::Region(region)), edges: BTreeSet::new() },
        );
        Ok(object)
    }

    fn add_tracing_root(&mut self, object: ObjectId) -> Result<(), ModelError> {
        let node = self.objects.get(&object).ok_or(ModelError::UnknownObject(object))?;
        if node.domain != Some(StorageDomain::Tracing) {
            return Err(ModelError::TracingRootCannotReferenceRegion(object));
        }
        self.tracing_roots.insert(object);
        Ok(())
    }

    fn add_edge(&mut self, source: ObjectId, target: ObjectId) -> Result<(), ModelError> {
        let source_domain = self
            .objects
            .get(&source)
            .ok_or(ModelError::UnknownObject(source))?
            .domain
            .ok_or(ModelError::UnknownObject(source))?;
        let target_domain = self
            .objects
            .get(&target)
            .ok_or(ModelError::UnknownObject(target))?
            .domain
            .ok_or(ModelError::UnknownObject(target))?;

        match (source_domain, target_domain) {
            (StorageDomain::Tracing, StorageDomain::Region(_)) => {
                return Err(ModelError::TracingToRegionEscape { source, target });
            }
            (StorageDomain::Region(a), StorageDomain::Region(b)) if a != b => {
                return Err(ModelError::CrossRegionReference { source, target });
            }
            _ => {}
        }

        self.objects
            .get_mut(&source)
            .expect("source checked above")
            .edges
            .insert(target);
        Ok(())
    }

    fn end_region(&mut self, region: RegionId) -> Result<Vec<ObjectId>, ModelError> {
        if !self.live_regions.remove(&region) {
            return Err(ModelError::RegionAlreadyEnded(region));
        }
        let members = self
            .objects
            .iter()
            .filter_map(|(object, node)| {
                (node.domain == Some(StorageDomain::Region(region))).then_some(*object)
            })
            .collect::<Vec<_>>();
        for object in &members {
            self.objects.remove(object);
        }
        Ok(members)
    }

    /// Region objects in live regions act as roots for outgoing tracing-domain
    /// edges. Once a region ends, those roots disappear and a later collection
    /// can reclaim otherwise-unreachable tracing objects.
    fn collect_tracing(&mut self) -> Vec<ObjectId> {
        let mut pending = VecDeque::from_iter(self.tracing_roots.iter().copied());
        for node in self.objects.values() {
            if matches!(node.domain, Some(StorageDomain::Region(region)) if self.live_regions.contains(&region)) {
                pending.extend(node.edges.iter().copied());
            }
        }

        let mut live = BTreeSet::new();
        while let Some(object) = pending.pop_front() {
            if !live.insert(object) {
                continue;
            }
            if let Some(node) = self.objects.get(&object) {
                if node.domain == Some(StorageDomain::Tracing) {
                    pending.extend(node.edges.iter().copied());
                }
            }
        }

        let reclaimed = self
            .objects
            .iter()
            .filter_map(|(object, node)| {
                (node.domain == Some(StorageDomain::Tracing) && !live.contains(object))
                    .then_some(*object)
            })
            .collect::<Vec<_>>();
        for object in &reclaimed {
            self.objects.remove(object);
        }
        reclaimed
    }

    fn contains(&self, object: ObjectId) -> bool {
        self.objects.contains_key(&object)
    }
}

struct DeterministicResourceToken(Rc<Cell<usize>>);

impl DeterministicResourceToken {
    fn new(drop_count: Rc<Cell<usize>>) -> Self {
        Self(drop_count)
    }
}

impl Drop for DeterministicResourceToken {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_accesses_coexist_but_exclude_mutable_access() {
        let object = ObjectId(1);
        let mut ledger = AccessLedger::default();

        ledger.begin_shared(object).expect("first shared borrow");
        ledger.begin_shared(object).expect("second shared borrow");
        assert_eq!(ledger.begin_mutable(object), Err(ModelError::BorrowConflict(object)));

        ledger.end_shared(object).expect("release one shared borrow");
        assert_eq!(ledger.begin_mutable(object), Err(ModelError::BorrowConflict(object)));

        ledger.end_shared(object).expect("release last shared borrow");
        ledger.begin_mutable(object).expect("exclusive access after shared borrows end");
        assert_eq!(ledger.begin_shared(object), Err(ModelError::BorrowConflict(object)));
        ledger.end_mutable(object).expect("release exclusive borrow");
    }

    #[test]
    fn region_end_bulk_reclaims_only_objects_in_that_region() {
        let mut heap = RegionHeap::default();
        let first = heap.new_region();
        let second = heap.new_region();
        let a = heap.allocate(first).expect("allocate a");
        let b = heap.allocate(first).expect("allocate b");
        let c = heap.allocate(second).expect("allocate c");

        let reclaimed = heap.end_region(first).expect("end first region");
        assert_eq!(reclaimed, vec![a, b]);
        assert!(!heap.contains(a));
        assert!(!heap.contains(b));
        assert!(heap.contains(c));
        assert_eq!(heap.allocate(first), Err(ModelError::UnknownRegion(first)));
        assert_eq!(heap.end_region(first), Err(ModelError::RegionAlreadyEnded(first)));
    }

    #[test]
    fn tracing_reclaims_unreachable_cycles_but_preserves_rooted_graphs() {
        let mut heap = TraceHeap::default();
        let a = heap.allocate();
        let b = heap.allocate();
        let c = heap.allocate();
        heap.add_edge(a, b).expect("a -> b");
        heap.add_edge(b, a).expect("b -> a");
        heap.add_edge(b, c).expect("b -> c");
        heap.add_root(a).expect("root a");

        assert!(heap.collect().is_empty());
        assert_eq!(heap.len(), 3);

        heap.remove_root(a);
        let reclaimed = heap.collect();
        assert_eq!(reclaimed, vec![a, b, c]);
        assert_eq!(heap.len(), 0);
    }

    #[test]
    fn reference_counting_reclaims_an_acyclic_graph_when_last_roots_end() {
        let mut heap = RcHeap::default();
        let parent = heap.allocate_root();
        let child = heap.allocate_root();
        heap.add_edge(parent, child).expect("parent -> child");

        heap.release_root(child).expect("release child root");
        let reclaimed = heap.release_root(parent).expect("release parent root");
        assert_eq!(reclaimed, vec![parent, child]);
        assert_eq!(heap.len(), 0);
    }

    #[test]
    fn plain_reference_counting_retains_an_unreachable_strong_cycle() {
        let mut heap = RcHeap::default();
        let a = heap.allocate_root();
        let b = heap.allocate_root();
        heap.add_edge(a, b).expect("a -> b");
        heap.add_edge(b, a).expect("b -> a");

        heap.release_root(a).expect("release a root");
        heap.release_root(b).expect("release b root");
        assert!(heap.contains(a));
        assert!(heap.contains(b));
        assert_eq!(heap.len(), 2);
    }

    #[test]
    fn cloned_roots_keep_the_counted_object_live_until_the_last_root_ends() {
        let mut heap = RcHeap::default();
        let object = heap.allocate_root();
        heap.clone_root(object).expect("clone root");

        assert!(heap.release_root(object).expect("release first root").is_empty());
        assert!(heap.contains(object));
        assert_eq!(heap.release_root(object).expect("release final root"), vec![object]);
        assert!(!heap.contains(object));
    }

    #[test]
    fn tracing_domain_cannot_retain_a_shorter_lived_region_object() {
        let mut heap = MixedDomainHeap::default();
        let region = heap.new_region();
        let region_object = heap.allocate_in_region(region).expect("region allocation");
        let tracing_object = heap.allocate_tracing();
        heap.add_tracing_root(tracing_object).expect("root tracing object");

        assert_eq!(
            heap.add_edge(tracing_object, region_object),
            Err(ModelError::TracingToRegionEscape {
                source: tracing_object,
                target: region_object,
            })
        );
        assert!(heap.collect_tracing().is_empty());
        assert!(heap.contains(tracing_object));
        assert!(heap.contains(region_object));
    }

    #[test]
    fn live_region_edges_keep_tracing_objects_alive_until_region_end() {
        let mut heap = MixedDomainHeap::default();
        let region = heap.new_region();
        let region_object = heap.allocate_in_region(region).expect("region allocation");
        let managed_object = heap.allocate_tracing();

        heap.add_edge(region_object, managed_object).expect("shorter-lived to longer-lived edge");
        assert!(heap.collect_tracing().is_empty());
        assert!(heap.contains(managed_object));

        assert_eq!(heap.end_region(region), Ok(vec![region_object]));
        assert_eq!(heap.collect_tracing(), vec![managed_object]);
        assert!(!heap.contains(managed_object));
    }

    #[test]
    fn explicit_tracing_root_survives_the_region_that_also_referenced_it() {
        let mut heap = MixedDomainHeap::default();
        let region = heap.new_region();
        let region_object = heap.allocate_in_region(region).expect("region allocation");
        let managed_object = heap.allocate_tracing();

        heap.add_edge(region_object, managed_object).expect("region edge");
        heap.add_tracing_root(managed_object).expect("explicit long-lived root");
        heap.end_region(region).expect("end short region");
        assert!(heap.collect_tracing().is_empty());
        assert!(heap.contains(managed_object));

        heap.tracing_roots.remove(&managed_object);
        assert_eq!(heap.collect_tracing(), vec![managed_object]);
    }

    #[test]
    fn tracing_root_api_rejects_a_region_object() {
        let mut heap = MixedDomainHeap::default();
        let region = heap.new_region();
        let region_object = heap.allocate_in_region(region).expect("region allocation");

        assert_eq!(
            heap.add_tracing_root(region_object),
            Err(ModelError::TracingRootCannotReferenceRegion(region_object))
        );
    }

    #[test]
    fn deterministic_resource_drop_is_independent_of_traced_storage_reclamation() {
        let mut heap = TraceHeap::default();
        let object = heap.allocate();
        heap.add_root(object).expect("root managed object");

        let close_count = Rc::new(Cell::new(0));
        let resource = DeterministicResourceToken::new(Rc::clone(&close_count));
        drop(resource);

        assert_eq!(close_count.get(), 1);
        assert!(
            heap.contains(object),
            "resource cleanup must not imply object storage reclamation"
        );

        heap.remove_root(object);
        assert_eq!(heap.collect(), vec![object]);
        assert_eq!(
            close_count.get(),
            1,
            "later storage reclamation must not close the resource twice"
        );
    }
}
