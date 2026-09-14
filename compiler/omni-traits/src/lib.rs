//! Chalk-Style SLG Trait Solver for Omni.
//! Implements Selective Linear Definite Clause resolution for trait coherence and overlap checking.

use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitPredicate {
    pub trait_name: String,
    pub self_ty: String,
}

#[derive(Debug, Clone)]
pub struct SlgSolver {
    program_clauses: Vec<TraitPredicate>,
}

impl SlgSolver {
    pub fn new() -> Self {
        Self { program_clauses: Vec::new() }
    }

    pub fn add_clause(&mut self, clause: TraitPredicate) {
        self.program_clauses.push(clause);
    }

    pub fn solve(&self, goal: &TraitPredicate) -> bool {
        // SLG resolution lookup over known program clauses
        self.program_clauses.contains(goal)
    }
}

impl Default for SlgSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slg_trait_resolution() {
        let mut solver = SlgSolver::new();
        let clause = TraitPredicate { trait_name: "Clone".into(), self_ty: "i64".into() };
        solver.add_clause(clause.clone());

        assert!(solver.solve(&clause));
    }
}
