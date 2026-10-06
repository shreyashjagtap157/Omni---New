//! Affine ownership and borrow analysis primitives for Omni Edition 1.
//!
//! This crate contains the ownership state machine itself, independent of a
//! particular frontend or backend representation. MIR/HIR adapters can feed
//! concrete place identities into these primitives without duplicating the
//! affine/loan rules.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A projection component identifying a field/index/subplace within a value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Projection {
    /// Named field projection.
    Field(String),
    /// Constant tuple/array index projection.
    Index(usize),
    /// Dynamic index whose concrete value is not known to the ownership analysis.
    ///
    /// It overlaps every concrete index at the same projection position, which
    /// keeps the analysis conservative when runtime indices may alias.
    IndexAny,
    /// Dereference through a reference place.
    Deref,
}

/// A canonical memory place used by the ownership engine.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    /// Local/root binding name.
    pub root: String,
    /// Ordered projections from the root.
    pub projections: Vec<Projection>,
}

impl Place {
    /// Creates a root place with no projections.
    pub fn root(root: impl Into<String>) -> Self {
        Self { root: root.into(), projections: Vec::new() }
    }

    /// Creates a projected child place.
    pub fn project(&self, projection: Projection) -> Self {
        let mut next = self.clone();
        next.projections.push(projection);
        next
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root)?;
        for projection in &self.projections {
            match projection {
                Projection::Field(name) => write!(f, ".{name}")?,
                Projection::Index(index) => write!(f, "[{index}]")?,
                Projection::IndexAny => write!(f, "[*]")?,
                Projection::Deref => write!(f, ".*")?,
            }
        }
        Ok(())
    }
}

/// Affine initialization state of one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceState {
    /// The place has not been initialized.
    Uninitialized,
    /// The place is fully initialized.
    Initialized,
    /// The place has been partially moved.
    PartiallyMoved,
    /// The whole place has been moved.
    Moved,
}

/// Kind of access requested against a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind {
    /// Immutable read/copy access.
    Read,
    /// Ownership-consuming move.
    Move,
    /// Mutation/write access.
    Write,
    /// Shared borrow.
    BorrowShared,
    /// Exclusive mutable borrow.
    BorrowMut,
    /// Drop/destruction.
    Drop,
}

/// A live borrow loan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loan {
    /// Place through which the loan was created.
    pub place: Place,
    /// Whether the loan is mutable.
    pub mutable: bool,
    /// Stable region name/identifier.
    pub region: String,
}

/// Deterministic ownership diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipError {
    /// Read/write/move/drop of an uninitialized place.
    UseBeforeInitialization(Place),
    /// Move from a place that has already been moved.
    UseAfterMove(Place),
    /// A shared borrow conflicts with a mutable access.
    BorrowConflict { place: Place, existing: String },
    /// A mutable borrow conflicts with any existing usable loan.
    MutableBorrowConflict { place: Place, existing: String },
    /// A write occurs through a shared borrow.
    WriteThroughSharedBorrow(Place),
    /// A second mutable borrow is attempted.
    MultipleMutableBorrows(Place),
    /// A loan is referenced after it has ended.
    UnknownLoan(String),
}

impl fmt::Display for OwnershipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UseBeforeInitialization(place) => {
                write!(f, "use of uninitialized place '{place}'")
            }
            Self::UseAfterMove(place) => write!(f, "use of moved place '{place}'"),
            Self::BorrowConflict { place, existing } => {
                write!(f, "borrow conflict at '{place}': active {existing} loan")
            }
            Self::MutableBorrowConflict { place, existing } => {
                write!(f, "mutable borrow conflict at '{place}': active {existing} loan")
            }
            Self::WriteThroughSharedBorrow(place) => {
                write!(f, "cannot write through shared borrow of '{place}'")
            }
            Self::MultipleMutableBorrows(place) => {
                write!(f, "multiple mutable borrows of '{place}'")
            }
            Self::UnknownLoan(region) => write!(f, "unknown ownership loan region '{region}'"),
        }
    }
}

/// Deterministic affine ownership state for a collection of places and loans.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OwnershipState {
    places: BTreeMap<Place, PlaceState>,
    loans: BTreeMap<String, Loan>,
}

impl OwnershipState {
    /// Creates an empty ownership state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current state of a place, defaulting to uninitialized.
    pub fn state(&self, place: &Place) -> PlaceState {
        if let Some(state) = self.places.get(place).copied() {
            if state != PlaceState::Initialized {
                return state;
            }
        }

        let ancestor_moved = self.places.iter().any(|(candidate, state)| {
            matches!(state, PlaceState::Moved | PlaceState::PartiallyMoved)
                && is_prefix(candidate, place)
                && candidate != place
        });
        if ancestor_moved {
            return PlaceState::Moved;
        }

        // A projection of an initialized aggregate is itself initialized, unless
        // a descendant move has already been recorded above. Without this the
        // sub-place has no entry of its own and would fall through to
        // `Uninitialized`, so `x.a` could never be read or moved after
        // `declare_initialized("x")`.
        let ancestor_initialized = self.places.iter().any(|(candidate, state)| {
            *state == PlaceState::Initialized && is_prefix(candidate, place) && candidate != place
        });
        if ancestor_initialized {
            return PlaceState::Initialized;
        }

        let descendant_moved = self.places.iter().any(|(candidate, state)| {
            matches!(state, PlaceState::Moved | PlaceState::PartiallyMoved)
                && is_prefix(place, candidate)
                && candidate != place
        });
        if descendant_moved {
            return PlaceState::PartiallyMoved;
        }

        self.places.get(place).copied().unwrap_or(PlaceState::Uninitialized)
    }

    /// Marks a place as initialized.
    pub fn initialize(&mut self, place: Place) {
        self.places.insert(place, PlaceState::Initialized);
    }

    /// Marks a place as explicitly dropped.
    pub fn drop_place(&mut self, place: &Place) -> Result<(), OwnershipError> {
        self.require_initialized(place)?;
        self.ensure_access_allowed(place, AccessKind::Drop)?;
        self.places.insert(place.clone(), PlaceState::Moved);
        Ok(())
    }

    /// Reads a place without consuming it.
    pub fn read(&self, place: &Place) -> Result<(), OwnershipError> {
        self.require_initialized(place)?;
        self.ensure_access_allowed(place, AccessKind::Read)
    }

    /// Moves a place, consuming the whole place.
    pub fn move_place(&mut self, place: Place) -> Result<(), OwnershipError> {
        self.require_initialized(&place)?;
        self.ensure_access_allowed(&place, AccessKind::Move)?;
        if place.projections.is_empty() {
            self.places.insert(place, PlaceState::Moved);
        } else {
            self.move_projection(place)?;
        }
        Ok(())
    }

    /// Moves a projected field/index while retaining other aggregate fields.
    pub fn move_projection(&mut self, place: Place) -> Result<(), OwnershipError> {
        self.require_initialized(&place)?;
        self.ensure_access_allowed(&place, AccessKind::Move)?;
        // Moving only a projection leaves the parent aggregate partly live:
        // sibling fields stay readable, while the parent itself may no longer be
        // consumed whole. `PlaceState::PartiallyMoved` is what `state` reports
        // for that parent, so it is recorded directly rather than relying on the
        // ancestor query to infer it from a plain `Moved` entry.
        let state = if place.projections.is_empty() {
            PlaceState::Moved
        } else {
            PlaceState::PartiallyMoved
        };
        self.places.insert(place, state);
        Ok(())
    }

    /// Creates a shared loan for a place.
    pub fn borrow_shared(
        &mut self,
        place: Place,
        region: impl Into<String>,
    ) -> Result<(), OwnershipError> {
        self.require_initialized(&place)?;
        if let Some(existing) = self.conflicting_loan(&place, false) {
            return Err(OwnershipError::BorrowConflict {
                place,
                existing: if existing.mutable { "mutable".into() } else { "shared".into() },
            });
        }
        let region = region.into();
        self.loans.insert(region.clone(), Loan { place, mutable: false, region });
        Ok(())
    }

    /// Creates a mutable loan for a place.
    pub fn borrow_mut(
        &mut self,
        place: Place,
        region: impl Into<String>,
    ) -> Result<(), OwnershipError> {
        self.require_initialized(&place)?;
        if self.conflicting_loan(&place, true).is_some() {
            return Err(OwnershipError::MutableBorrowConflict { place, existing: "active".into() });
        }
        let region = region.into();
        self.loans.insert(region.clone(), Loan { place, mutable: true, region });
        Ok(())
    }

    /// Ends a live loan by region identifier.
    pub fn end_loan(&mut self, region: &str) -> Result<Loan, OwnershipError> {
        self.loans.remove(region).ok_or_else(|| OwnershipError::UnknownLoan(region.into()))
    }

    /// Returns all live loans in deterministic region order.
    pub fn loans(&self) -> impl Iterator<Item = &Loan> {
        self.loans.values()
    }

    /// Assigns a value into a place, initializing it when necessary.
    ///
    /// Assignment is distinct from AccessKind::Write: a first assignment to
    /// an uninitialized local is valid and establishes initialization.
    pub fn assign(&mut self, place: Place) -> Result<(), OwnershipError> {
        self.ensure_access_allowed(&place, AccessKind::Write)?;
        self.places.insert(place, PlaceState::Initialized);
        Ok(())
    }

    /// Joins ownership states at a control-flow merge conservatively.
    pub fn join_all(states: &[&Self]) -> Self {
        if states.is_empty() {
            return Self::new();
        }

        let mut places = BTreeSet::new();
        for state in states {
            places.extend(state.places.keys().cloned());
        }

        let mut joined = Self::new();
        for place in places {
            let states_for_place =
                states.iter().map(|state| state.state(&place)).collect::<Vec<_>>();
            let combined = if states_for_place.iter().all(|state| *state == PlaceState::Initialized)
            {
                PlaceState::Initialized
            } else if states_for_place.iter().all(|state| *state == PlaceState::Uninitialized) {
                PlaceState::Uninitialized
            } else if states_for_place.iter().all(|state| *state == PlaceState::Moved) {
                PlaceState::Moved
            } else if states_for_place.iter().all(|state| *state == PlaceState::PartiallyMoved) {
                PlaceState::PartiallyMoved
            } else {
                PlaceState::PartiallyMoved
            };
            if combined != PlaceState::Uninitialized {
                joined.places.insert(place, combined);
            }
        }

        for (region, loan) in &states[0].loans {
            if states.iter().skip(1).all(|state| state.loans.get(region) == Some(loan)) {
                joined.loans.insert(region.clone(), loan.clone());
            }
        }

        joined
    }

    fn require_initialized(&self, place: &Place) -> Result<(), OwnershipError> {
        match self.state(place) {
            PlaceState::Initialized => Ok(()),
            PlaceState::PartiallyMoved => Err(OwnershipError::UseAfterMove(place.clone())),
            PlaceState::Moved => Err(OwnershipError::UseAfterMove(place.clone())),
            PlaceState::Uninitialized => {
                Err(OwnershipError::UseBeforeInitialization(place.clone()))
            }
        }
    }

    fn ensure_access_allowed(
        &self,
        place: &Place,
        access: AccessKind,
    ) -> Result<(), OwnershipError> {
        let related = self
            .loans
            .values()
            .filter(|loan| places_overlap(&loan.place, place))
            .collect::<Vec<_>>();

        for loan in related {
            match access {
                AccessKind::BorrowShared | AccessKind::Read => {
                    if loan.mutable {
                        return Err(OwnershipError::BorrowConflict {
                            place: place.clone(),
                            existing: "mutable".into(),
                        });
                    }
                }
                AccessKind::BorrowMut | AccessKind::Write | AccessKind::Move | AccessKind::Drop => {
                    if loan.mutable {
                        return Err(OwnershipError::MutableBorrowConflict {
                            place: place.clone(),
                            existing: "mutable".into(),
                        });
                    }
                    return Err(OwnershipError::BorrowConflict {
                        place: place.clone(),
                        existing: "shared".into(),
                    });
                }
            }
        }
        Ok(())
    }

    fn conflicting_loan(&self, place: &Place, mutable: bool) -> Option<&Loan> {
        self.loans
            .values()
            .find(|loan| places_overlap(&loan.place, place) && (mutable || loan.mutable))
    }
}

/// Returns whether two places may alias the same storage.
fn is_prefix(prefix: &Place, value: &Place) -> bool {
    prefix.root == value.root
        && prefix.projections.len() <= value.projections.len()
        && prefix.projections.iter().zip(&value.projections).all(|(a, b)| {
            matches!((a, b), (Projection::IndexAny, _) | (_, Projection::IndexAny)) || a == b
        })
}

fn places_overlap(a: &Place, b: &Place) -> bool {
    is_prefix(a, b) || is_prefix(b, a)
}

/// Convenience ownership checker for a single function/control-flow region.
#[derive(Debug, Clone, Default)]
pub struct OwnershipChecker {
    state: OwnershipState,
}

impl OwnershipChecker {
    /// Creates a checker with empty place/loan state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the underlying state.
    pub fn state(&self) -> &OwnershipState {
        &self.state
    }

    /// Initializes a local root.
    pub fn declare_initialized(&mut self, name: impl Into<String>) {
        self.state.initialize(Place::root(name));
    }

    /// Applies an access operation.
    pub fn access(&mut self, place: Place, kind: AccessKind) -> Result<(), OwnershipError> {
        match kind {
            AccessKind::Read => self.state.read(&place),
            AccessKind::Move => self.state.move_place(place),
            AccessKind::Write => {
                self.state.require_initialized(&place)?;
                self.state.ensure_access_allowed(&place, AccessKind::Write)?;
                self.state.places.insert(place, PlaceState::Initialized);
                Ok(())
            }
            AccessKind::BorrowShared => self.state.borrow_shared(place, "anonymous"),
            AccessKind::BorrowMut => self.state.borrow_mut(place, "anonymous"),
            AccessKind::Drop => self.state.drop_place(&place),
        }
    }

    /// Issues a named shared loan.
    pub fn issue_shared(
        &mut self,
        place: Place,
        region: impl Into<String>,
    ) -> Result<(), OwnershipError> {
        self.state.borrow_shared(place, region)
    }

    /// Issues a named mutable loan.
    pub fn issue_mut(
        &mut self,
        place: Place,
        region: impl Into<String>,
    ) -> Result<(), OwnershipError> {
        self.state.borrow_mut(place, region)
    }

    /// Ends a named loan.
    pub fn end_loan(&mut self, region: &str) -> Result<Loan, OwnershipError> {
        self.state.end_loan(region)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affine_move_is_single_use() {
        let mut checker = OwnershipChecker::new();
        checker.declare_initialized("x");
        checker.access(Place::root("x"), AccessKind::Move).expect("first move");
        let error = checker.access(Place::root("x"), AccessKind::Read).expect_err("second use");
        assert_eq!(error, OwnershipError::UseAfterMove(Place::root("x")));
    }

    #[test]
    fn shared_borrows_can_coexist() {
        let mut checker = OwnershipChecker::new();
        let x = Place::root("x");
        checker.declare_initialized("x");
        checker.issue_shared(x.clone(), "r1").expect("first shared loan");
        checker.issue_shared(x.clone(), "r2").expect("second shared loan");
        assert_eq!(checker.state().loans().count(), 2);
    }

    #[test]
    fn mutable_borrow_conflicts_with_shared_borrow() {
        let mut checker = OwnershipChecker::new();
        let x = Place::root("x");
        checker.declare_initialized("x");
        checker.issue_shared(x.clone(), "r1").expect("shared loan");
        let error = checker.issue_mut(x, "r2").expect_err("mutable conflict");
        assert!(matches!(error, OwnershipError::MutableBorrowConflict { .. }));
    }

    #[test]
    fn shared_borrow_blocks_write_and_move() {
        let mut checker = OwnershipChecker::new();
        let x = Place::root("x");
        checker.declare_initialized("x");
        checker.issue_shared(x.clone(), "r1").expect("shared loan");
        assert!(matches!(
            checker.access(x.clone(), AccessKind::Write),
            Err(OwnershipError::BorrowConflict { .. })
        ));
        assert!(matches!(
            checker.access(x, AccessKind::Move),
            Err(OwnershipError::BorrowConflict { .. })
        ));
    }

    #[test]
    fn disjoint_fields_do_not_conflict() {
        let mut checker = OwnershipChecker::new();
        let x = Place::root("x");
        let a = x.project(Projection::Field("a".into()));
        let b = x.project(Projection::Field("b".into()));
        checker.declare_initialized("x");
        checker.issue_mut(a.clone(), "ra").expect("field a loan");
        checker.issue_shared(b.clone(), "rb").expect("disjoint field b loan");
        checker.state().loans().count();
    }

    #[test]
    fn moving_a_field_partially_moves_the_parent() {
        let mut checker = OwnershipChecker::new();
        let x = Place::root("x");
        let a = x.project(Projection::Field("a".into()));
        checker.declare_initialized("x");
        checker.access(a, AccessKind::Move).expect("move field");
        assert_eq!(checker.state().state(&x), PlaceState::PartiallyMoved);
    }

    #[test]
    fn disjoint_roots_do_not_conflict() {
        let mut checker = OwnershipChecker::new();
        let x = Place::root("x");
        let y = Place::root("y");
        checker.declare_initialized("x");
        checker.declare_initialized("y");
        checker.issue_mut(x, "rx").expect("x mutable loan");
        checker.access(y, AccessKind::Write).expect("disjoint write");
    }
}
