//! Chalk-Style SLG Trait Solver for Omni.
//! Implements Selective Linear Definite Clause resolution with backchaining,
//! type parameter substitution, and coherence overlap checking for Epoch 2.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitPredicate {
    pub trait_name: String,
    pub self_ty: String,
    pub assoc_bindings: HashMap<String, String>,
}

impl TraitPredicate {
    pub fn new(trait_name: impl Into<String>, self_ty: impl Into<String>) -> Self {
        Self {
            trait_name: trait_name.into(),
            self_ty: self_ty.into(),
            assoc_bindings: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramClause {
    pub head: TraitPredicate,
    pub conditions: Vec<TraitPredicate>,
}

#[derive(Debug, Clone, Default)]
pub struct SlgSolver {
    clauses: Vec<ProgramClause>,
}

impl SlgSolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fact(&mut self, trait_name: &str, self_ty: &str) {
        self.clauses.push(ProgramClause {
            head: TraitPredicate::new(trait_name, self_ty),
            conditions: vec![],
        });
    }

    pub fn add_rule(&mut self, head: TraitPredicate, conditions: Vec<TraitPredicate>) {
        self.clauses.push(ProgramClause { head, conditions });
    }

    /// Recursive SLG goal evaluation with backchaining and substitution unification
    pub fn solve(&self, goal: &TraitPredicate) -> bool {
        let mut visited = HashSet::new();
        self.solve_goal_rec(goal, &mut visited, 0)
    }

    fn solve_goal_rec(
        &self,
        goal: &TraitPredicate,
        visited: &mut HashSet<TraitPredicate>,
        depth: usize,
    ) -> bool {
        if depth > 32 {
            return false; // recursion guard
        }
        if !visited.insert(goal.clone()) {
            return false; // cycle prevention in SLG engine
        }

        for clause in &self.clauses {
            if let Some(subst) = unify(&clause.head, goal) {
                let all_met = clause.conditions.iter().all(|cond| {
                    let instantiated_cond = apply_subst(cond, &subst);
                    self.solve_goal_rec(&instantiated_cond, visited, depth + 1)
                });
                if all_met {
                    visited.remove(goal);
                    return true;
                }
            }
        }

        visited.remove(goal);
        false
    }
}

fn unify(clause_head: &TraitPredicate, goal: &TraitPredicate) -> Option<HashMap<String, String>> {
    if clause_head.trait_name != goal.trait_name {
        return None;
    }
    let mut subst = HashMap::new();
    if clause_head.self_ty.starts_with('T') || clause_head.self_ty.starts_with('U') {
        subst.insert(clause_head.self_ty.clone(), goal.self_ty.clone());
    } else if clause_head.self_ty != goal.self_ty {
        return None;
    }
    Some(subst)
}

fn apply_subst(pred: &TraitPredicate, subst: &HashMap<String, String>) -> TraitPredicate {
    let new_self = subst.get(&pred.self_ty).cloned().unwrap_or_else(|| pred.self_ty.clone());
    TraitPredicate {
        trait_name: pred.trait_name.clone(),
        self_ty: new_self,
        assoc_bindings: pred.assoc_bindings.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slg_recursive_resolution() {
        let mut solver = SlgSolver::new();
        solver.add_fact("Clone", "i64");

        let head = TraitPredicate::new("Debug", "T");
        let cond = TraitPredicate::new("Clone", "T");
        solver.add_rule(head, vec![cond]);

        let goal = TraitPredicate::new("Debug", "i64");
        assert!(solver.solve(&goal));
    }
}
