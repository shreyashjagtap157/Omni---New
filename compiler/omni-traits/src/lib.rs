//! Chalk-Style SLG Trait Solver and Coherence Engine for Omni.
//! Implements Selective Linear Definite Clause resolution with backchaining,
//! type parameter substitution, positive/negative bounds, orphan coherence validation,
//! and trait registration error diagnostics.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use omni_types::ast::{ImplDef, TraitBound, TraitDef, TypeSpec};
use omni_types::checker::{SubstEnv, TraitObligationChecker, TypeChecker};
use omni_types::intern::{Ty, TyCtxt};

/// Explicit errors during trait and impl registration and coherence checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitRegistrationError {
    DuplicateTrait(String),
    UnknownTrait(String),
    DuplicateImpl { trait_name: String, target_ty: String },
    MissingRequiredMethod { trait_name: String, method_name: String },
    InvalidMethodSignature { trait_name: String, method_name: String, detail: String },
    InvalidBound { param: String, bound: String, detail: String },
    InvalidSupertrait { trait_name: String, supertrait: String },
    ConflictingImplementation { trait_name: String, target_ty: String, reason: String },
    CoherenceOrphanViolation { trait_name: String, target_ty: String },
    NegativeBoundConflict { trait_name: String, target_ty: String },
}

impl std::fmt::Display for TraitRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateTrait(name) => write!(f, "duplicate trait declaration: {name}"),
            Self::UnknownTrait(name) => write!(f, "unknown trait: {name}"),
            Self::DuplicateImpl { trait_name, target_ty } => {
                write!(f, "duplicate impl: {trait_name} for {target_ty}")
            }
            Self::MissingRequiredMethod { trait_name, method_name } => {
                write!(f, "missing required method `{method_name}` for trait `{trait_name}`")
            }
            Self::InvalidMethodSignature { trait_name, method_name, detail } => {
                write!(
                    f,
                    "invalid signature for method `{method_name}` in trait `{trait_name}`: {detail}"
                )
            }
            Self::InvalidBound { param, bound, detail } => {
                write!(f, "invalid bound `{bound}` on parameter `{param}`: {detail}")
            }
            Self::InvalidSupertrait { trait_name, supertrait } => {
                write!(f, "invalid supertrait `{supertrait}` for trait `{trait_name}`")
            }
            Self::ConflictingImplementation { trait_name, target_ty, reason } => {
                write!(f, "conflicting impl for `{trait_name}` on `{target_ty}`: {reason}")
            }
            Self::CoherenceOrphanViolation { trait_name, target_ty } => {
                write!(
                    f,
                    "coherence violation: cannot implement foreign trait `{trait_name}` for foreign type `{target_ty}` (orphan rule)"
                )
            }
            Self::NegativeBoundConflict { trait_name, target_ty } => {
                write!(
                    f,
                    "negative bound conflict: type `{target_ty}` is explicitly negative for `{trait_name}`"
                )
            }
        }
    }
}

impl std::error::Error for TraitRegistrationError {}

/// A trait predicate goal or clause head, e.g. `Debug: i64` or `Clone: T`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitPredicate {
    pub trait_name: String,
    pub self_ty: String,
    pub is_positive: bool,
    pub assoc_bindings: Vec<(String, String)>,
}

impl TraitPredicate {
    pub fn new(trait_name: impl Into<String>, self_ty: impl Into<String>) -> Self {
        Self {
            trait_name: trait_name.into(),
            self_ty: self_ty.into(),
            is_positive: true,
            assoc_bindings: Vec::new(),
        }
    }

    pub fn new_negative(trait_name: impl Into<String>, self_ty: impl Into<String>) -> Self {
        Self {
            trait_name: trait_name.into(),
            self_ty: self_ty.into(),
            is_positive: false,
            assoc_bindings: Vec::new(),
        }
    }
}

/// A definite program clause for SLG resolution: `head :- conditions`.
#[derive(Debug, Clone)]
pub struct ProgramClause {
    pub head: TraitPredicate,
    pub conditions: Vec<TraitPredicate>,
}

/// The Chalk-style SLG solver.
#[derive(Debug, Clone, Default)]
pub struct SlgSolver {
    pub clauses: Vec<ProgramClause>,
    pub negative_facts: HashSet<(String, String)>, // (trait_name, self_ty) explicitly negative
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

    pub fn add_negative_fact(&mut self, trait_name: &str, self_ty: &str) {
        self.negative_facts.insert((trait_name.to_string(), self_ty.to_string()));
    }

    pub fn add_rule(&mut self, head: TraitPredicate, conditions: Vec<TraitPredicate>) {
        self.clauses.push(ProgramClause { head, conditions });
    }

    /// Evaluates whether a trait predicate goal is satisfied.
    pub fn solve(&self, goal: &TraitPredicate) -> bool {
        // Check negative bound violation
        if goal.is_positive {
            if self.negative_facts.contains(&(goal.trait_name.clone(), goal.self_ty.clone())) {
                return false;
            }
        } else {
            // Negative goal: true if explicitly negative fact exists and positive cannot be proved
            if self.negative_facts.contains(&(goal.trait_name.clone(), goal.self_ty.clone())) {
                return true;
            }
            let mut pos_visited = HashSet::new();
            let mut pos_goal = goal.clone();
            pos_goal.is_positive = true;
            return !self.solve_goal_rec(&pos_goal, &mut pos_visited, 0);
        }

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
    if clause_head.trait_name != goal.trait_name || clause_head.is_positive != goal.is_positive {
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
        is_positive: pred.is_positive,
        assoc_bindings: pred.assoc_bindings.clone(),
    }
}

/// The unified Trait System serving as the single semantic authority for trait declarations,
/// impls, coherence checking, and obligation resolution.
#[derive(Debug, Clone, Default)]
pub struct TraitSystem {
    pub traits: HashMap<String, TraitDef>,
    pub impls: Vec<ImplDef>,
    pub solver: SlgSolver,
}

impl TraitSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a trait definition with fatal validation diagnostics.
    pub fn register_trait(&mut self, trait_def: TraitDef) -> Result<(), TraitRegistrationError> {
        if self.traits.contains_key(&trait_def.name) {
            return Err(TraitRegistrationError::DuplicateTrait(trait_def.name));
        }

        // Verify supertraits exist
        for supertrait in &trait_def.supertraits {
            if !self.traits.contains_key(supertrait) {
                return Err(TraitRegistrationError::InvalidSupertrait {
                    trait_name: trait_def.name.clone(),
                    supertrait: supertrait.clone(),
                });
            }
        }

        self.traits.insert(trait_def.name.clone(), trait_def);
        Ok(())
    }

    /// Registers an impl block with fatal coherence and completeness checks.
    pub fn register_impl(
        &mut self,
        impl_def: ImplDef,
        tcx: &TyCtxt,
    ) -> Result<(), TraitRegistrationError> {
        let trait_def = self
            .traits
            .get(&impl_def.trait_name)
            .cloned()
            .ok_or_else(|| TraitRegistrationError::UnknownTrait(impl_def.trait_name.clone()))?;

        let target_ty_mangled = self.type_spec_to_mangled(&impl_def.target_ty, tcx);

        // 1. Coherence / Orphan check:
        // A trait implementation is permitted if and only if either:
        // - The trait is local, OR
        // - The target type is local.
        // If BOTH the trait is foreign and the target type is foreign, it is rejected fail-closed.
        let target_ty_is_local = impl_def.is_local;
        if !trait_def.is_local && !target_ty_is_local {
            return Err(TraitRegistrationError::CoherenceOrphanViolation {
                trait_name: impl_def.trait_name.clone(),
                target_ty: target_ty_mangled.clone(),
            });
        }

        // 2. Overlap / Duplicate impl check:
        for existing in &self.impls {
            if existing.trait_name == impl_def.trait_name {
                let existing_ty_mangled = self.type_spec_to_mangled(&existing.target_ty, tcx);
                if existing_ty_mangled == target_ty_mangled {
                    return Err(TraitRegistrationError::DuplicateImpl {
                        trait_name: impl_def.trait_name.clone(),
                        target_ty: target_ty_mangled.clone(),
                    });
                }
            }
        }

        // 3. Completeness check: all required methods without default implementations must be supplied.
        let provided_methods: HashSet<&str> =
            impl_def.methods.iter().map(|(name, _)| name.as_str()).collect();

        for method in &trait_def.methods {
            if !method.has_default && !provided_methods.contains(method.name.as_str()) {
                return Err(TraitRegistrationError::MissingRequiredMethod {
                    trait_name: impl_def.trait_name.clone(),
                    method_name: method.name.clone(),
                });
            }
        }

        // 4. Register SLG clause rules and supertrait obligations
        let mut conditions = Vec::new();
        for (param, bound) in &impl_def.conditions {
            match bound {
                TraitBound::Positive(tr) => {
                    conditions.push(TraitPredicate::new(tr.clone(), param.clone()));
                }
                TraitBound::Negative(tr) => {
                    conditions.push(TraitPredicate::new_negative(tr.clone(), param.clone()));
                }
            }
        }

        // Impl head clause
        let head = TraitPredicate::new(impl_def.trait_name.clone(), target_ty_mangled.clone());
        self.solver.add_rule(head, conditions.clone());

        // Supertrait implied bounds: if Trait requires Supertrait, target must satisfy it
        for supertrait in &trait_def.supertraits {
            let super_head = TraitPredicate::new(supertrait.clone(), target_ty_mangled.clone());
            self.solver.add_rule(super_head, conditions.clone());
        }

        self.impls.push(impl_def);
        Ok(())
    }

    /// Resolves whether a concrete type satisfies a trait bound.
    pub fn satisfies_bound(&self, trait_name: &str, concrete_ty: Ty, tcx: &TyCtxt) -> bool {
        let ty_name = tcx.mangle(concrete_ty);
        let goal = TraitPredicate::new(trait_name, ty_name);
        self.solver.solve(&goal)
    }

    /// Resolves whether a concrete type satisfies a negative trait bound.
    pub fn satisfies_negative_bound(
        &self,
        trait_name: &str,
        concrete_ty: Ty,
        tcx: &TyCtxt,
    ) -> bool {
        let ty_name = tcx.mangle(concrete_ty);
        let goal = TraitPredicate::new_negative(trait_name, ty_name);
        self.solver.solve(&goal)
    }

    /// Evaluates trait obligations on a generic function instantiation under its concrete `SubstEnv`.
    pub fn verify_obligations(
        &self,
        bounds: &[(String, TraitBound)],
        subst_env: &SubstEnv,
        tcx: &TyCtxt,
    ) -> Result<(), String> {
        for (param, bound) in bounds {
            if let Some(concrete_ty) = subst_env.get(param) {
                match bound {
                    TraitBound::Positive(trait_name) => {
                        if !self.satisfies_bound(trait_name, concrete_ty, tcx) {
                            return Err(format!(
                                "type `{}` does not satisfy required trait bound `{trait_name}` for parameter `{param}`",
                                tcx.mangle(concrete_ty)
                            ));
                        }
                    }
                    TraitBound::Negative(trait_name) => {
                        if !self.satisfies_negative_bound(trait_name, concrete_ty, tcx) {
                            return Err(format!(
                                "type `{}` conflicts with negative trait bound `!{trait_name}` for parameter `{param}`",
                                tcx.mangle(concrete_ty)
                            ));
                        }
                    }
                }
            } else {
                return Err(format!(
                    "unresolved type parameter `{param}` in trait obligation check"
                ));
            }
        }
        Ok(())
    }

    /// Creates an `Arc` closure that hooks into `TypeChecker` for trait obligation solving.
    pub fn to_checker_hook(self: &Arc<Self>) -> TraitObligationChecker {
        let ts = Arc::clone(self);
        Arc::new(move |bounds, subst_env, tcx| ts.verify_obligations(bounds, subst_env, tcx))
    }

    /// Attaches this TraitSystem's obligation verification to a `TypeChecker`.
    pub fn attach_to_checker(self: &Arc<Self>, checker: &mut TypeChecker) {
        checker.set_trait_checker(self.to_checker_hook());
    }

    fn type_spec_to_mangled(&self, spec: &TypeSpec, tcx: &TyCtxt) -> String {
        match spec {
            TypeSpec::Int => "i64".to_string(),
            TypeSpec::Float => "f64".to_string(),
            TypeSpec::Bool => "bool".to_string(),
            TypeSpec::Char => "char".to_string(),
            TypeSpec::Byte => "u8".to_string(),
            TypeSpec::String => "String".to_string(),
            TypeSpec::Unit => "unit".to_string(),
            TypeSpec::GenericParam(name) => name.clone(),
            TypeSpec::Known(ty) => tcx.mangle(*ty),
            TypeSpec::Tuple(tys) => {
                let parts: Vec<String> =
                    tys.iter().map(|t| self.type_spec_to_mangled(t, tcx)).collect();
                format!("tuple_{}_end", parts.join("_"))
            }
            TypeSpec::Array(elem, len) => {
                format!("arr_{len}_{}_end", self.type_spec_to_mangled(elem, tcx))
            }
            TypeSpec::Range(elem) => {
                format!("range_{}_end", self.type_spec_to_mangled(elem, tcx))
            }
            TypeSpec::Fn(params, ret) => {
                let p: Vec<String> =
                    params.iter().map(|t| self.type_spec_to_mangled(t, tcx)).collect();
                format!("fn_{}_ret_{}_end", p.join("_"), self.type_spec_to_mangled(ret, tcx))
            }
            TypeSpec::Struct(name, args) => {
                let a: Vec<String> =
                    args.iter().map(|t| self.type_spec_to_mangled(t, tcx)).collect();
                format!("struct_{}_{}_end", name, a.join("_"))
            }
            TypeSpec::Enum(name, args) => {
                let a: Vec<String> =
                    args.iter().map(|t| self.type_spec_to_mangled(t, tcx)).collect();
                format!("enum_{}_{}_end", name, a.join("_"))
            }
            TypeSpec::Never => "never".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_types::ast::MethodSig;
    use omni_types::intern::TyKind;

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

    #[test]
    fn test_trait_registration_fatal_errors() {
        let mut ts = TraitSystem::new();
        let tcx = TyCtxt::new();

        let trait_def = TraitDef {
            name: "Display".to_string(),
            supertraits: vec![],
            methods: vec![MethodSig {
                name: "to_string".to_string(),
                params: vec![],
                return_type: TypeSpec::String,
                has_default: false,
            }],
            is_local: true,
        };
        assert!(ts.register_trait(trait_def.clone()).is_ok());

        // Duplicate trait must fail fatal
        let dup = ts.register_trait(trait_def);
        assert_eq!(dup, Err(TraitRegistrationError::DuplicateTrait("Display".to_string())));

        // Impl missing required method must fail fatal
        let bad_impl = ImplDef {
            trait_name: "Display".to_string(),
            target_ty: TypeSpec::Int,
            conditions: vec![],
            methods: vec![],
            is_local: true,
        };
        let res = ts.register_impl(bad_impl, &tcx);
        assert_eq!(
            res,
            Err(TraitRegistrationError::MissingRequiredMethod {
                trait_name: "Display".to_string(),
                method_name: "to_string".to_string()
            })
        );
    }

    #[test]
    fn test_coherence_orphan_rule() {
        let mut ts = TraitSystem::new();
        let tcx = TyCtxt::new();

        // Foreign trait
        ts.register_trait(TraitDef {
            name: "ForeignTrait".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: false,
        })
        .unwrap();

        // Impl foreign trait for foreign type -> MUST FAIL ORPHAN
        let orphan_impl = ImplDef {
            trait_name: "ForeignTrait".to_string(),
            target_ty: TypeSpec::Int,
            conditions: vec![],
            methods: vec![],
            is_local: false, // foreign type
        };
        let res = ts.register_impl(orphan_impl, &tcx);
        assert_eq!(
            res,
            Err(TraitRegistrationError::CoherenceOrphanViolation {
                trait_name: "ForeignTrait".to_string(),
                target_ty: "i64".to_string()
            })
        );

        // Impl foreign trait for local type -> MUST SUCCEED
        let valid_impl = ImplDef {
            trait_name: "ForeignTrait".to_string(),
            target_ty: TypeSpec::Struct("LocalStruct".to_string(), vec![]),
            conditions: vec![],
            methods: vec![],
            is_local: true, // local type
        };
        assert!(ts.register_impl(valid_impl, &tcx).is_ok());
    }

    #[test]
    fn test_negative_bounds_and_obligations() {
        let mut ts = TraitSystem::new();
        let mut tcx = TyCtxt::new();

        let i64_ty = tcx.intern(TyKind::Int);
        let str_ty = tcx.intern(TyKind::String);

        ts.register_trait(TraitDef {
            name: "Copy".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();

        // Add positive fact: i64 is Copy
        ts.solver.add_fact("Copy", "i64");
        // Add negative fact: String is !Copy
        ts.solver.add_negative_fact("Copy", "String");

        assert!(ts.satisfies_bound("Copy", i64_ty, &tcx));
        assert!(!ts.satisfies_bound("Copy", str_ty, &tcx));
        assert!(ts.satisfies_negative_bound("Copy", str_ty, &tcx));

        let mut subst_env = SubstEnv::new();
        subst_env.insert("T".to_string(), str_ty);

        let bounds = vec![("T".to_string(), TraitBound::Negative("Copy".to_string()))];
        assert!(ts.verify_obligations(&bounds, &subst_env, &tcx).is_ok());

        let pos_bounds = vec![("T".to_string(), TraitBound::Positive("Copy".to_string()))];
        assert!(ts.verify_obligations(&pos_bounds, &subst_env, &tcx).is_err());
    }

    #[test]
    fn test_typechecker_trait_obligation_integration() {
        use omni_types::ast::{Expr, GenericFnDef, Lit};

        let mut ts = TraitSystem::new();
        let mut checker = TypeChecker::new();

        ts.register_trait(TraitDef {
            name: "Printable".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();

        // Implement Printable for Int (i64), but NOT for Float (f64)
        ts.solver.add_fact("Printable", "i64");

        let ts_arc = Arc::new(ts);
        ts_arc.attach_to_checker(&mut checker);

        // Register generic function with trait bound T: Printable
        checker.register_fn(GenericFnDef {
            name: "print_val".to_string(),
            type_params: vec!["T".to_string()],
            bounds: vec![("T".to_string(), TraitBound::Positive("Printable".to_string()))],
            params: vec![("x".to_string(), TypeSpec::GenericParam("T".to_string()))],
            return_type: TypeSpec::GenericParam("T".to_string()),
            body: Expr::Var("x".to_string()),
        });

        let env = SubstEnv::new();
        let local_vars = HashMap::new();

        // 1. Call with Lit::Int(42) -> T resolves to i64 -> satisfies Printable -> SUCCESS
        let call_res =
            checker.infer_call("print_val", &[], &[Expr::Literal(Lit::Int(42))], &env, &local_vars);
        assert!(call_res.is_ok(), "Expected call with i64 to satisfy Printable bound");

        // 2. Call with Lit::Float(4.5f64.to_bits()) -> T resolves to f64 -> does NOT satisfy Printable -> FAILS CLOSED
        let call_fail = checker.infer_call(
            "print_val",
            &[],
            &[Expr::Literal(Lit::Float(4.5f64.to_bits()))],
            &env,
            &local_vars,
        );
        assert!(
            call_fail.is_err(),
            "Expected call with f64 to fail closed on unsatisfied trait obligation"
        );
        match call_fail.unwrap_err() {
            omni_types::checker::TypeError::TraitObligationUnsatisfied(msg) => {
                assert!(msg.contains("Printable"));
            }
            other => panic!("Expected TraitObligationUnsatisfied, got {:?}", other),
        }
    }

    #[test]
    fn test_coherence_matrix() {
        let tcx = TyCtxt::new();

        // Matrix case 1: Local Trait + Local Type -> OK
        let mut ts1 = TraitSystem::new();
        ts1.register_trait(TraitDef {
            name: "TLocal".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();
        assert!(ts1
            .register_impl(
                ImplDef {
                    trait_name: "TLocal".to_string(),
                    target_ty: TypeSpec::Struct("SLocal".to_string(), vec![]),
                    conditions: vec![],
                    methods: vec![],
                    is_local: true,
                },
                &tcx
            )
            .is_ok());

        // Matrix case 2: Local Trait + Foreign Type -> OK
        let mut ts2 = TraitSystem::new();
        ts2.register_trait(TraitDef {
            name: "TLocal".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();
        assert!(ts2
            .register_impl(
                ImplDef {
                    trait_name: "TLocal".to_string(),
                    target_ty: TypeSpec::Int,
                    conditions: vec![],
                    methods: vec![],
                    is_local: false,
                },
                &tcx
            )
            .is_ok());

        // Matrix case 3: Foreign Trait + Local Type -> OK
        let mut ts3 = TraitSystem::new();
        ts3.register_trait(TraitDef {
            name: "TForeign".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: false,
        })
        .unwrap();
        assert!(ts3
            .register_impl(
                ImplDef {
                    trait_name: "TForeign".to_string(),
                    target_ty: TypeSpec::Struct("SLocal".to_string(), vec![]),
                    conditions: vec![],
                    methods: vec![],
                    is_local: true,
                },
                &tcx
            )
            .is_ok());

        // Matrix case 4: Foreign Trait + Foreign Type -> REJECTED (CoherenceOrphanViolation)
        let mut ts4 = TraitSystem::new();
        ts4.register_trait(TraitDef {
            name: "TForeign".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: false,
        })
        .unwrap();
        let orphan_res = ts4.register_impl(
            ImplDef {
                trait_name: "TForeign".to_string(),
                target_ty: TypeSpec::Int,
                conditions: vec![],
                methods: vec![],
                is_local: false,
            },
            &tcx,
        );
        assert!(orphan_res.is_err());
        assert_eq!(
            orphan_res.unwrap_err(),
            TraitRegistrationError::CoherenceOrphanViolation {
                trait_name: "TForeign".to_string(),
                target_ty: "i64".to_string(),
            }
        );
    }

    #[test]
    fn test_obligation_mutation_fails() {
        // Mutation check: if trait obligations are ignored/bypassed, invalid calls would succeed.
        // We verify that an unbound type will strictly trigger an obligation error.
        let mut ts = TraitSystem::new();
        let mut checker = TypeChecker::new();

        ts.register_trait(TraitDef {
            name: "Serializable".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();

        let ts_arc = Arc::new(ts);
        ts_arc.attach_to_checker(&mut checker);

        checker.register_fn(omni_types::ast::GenericFnDef {
            name: "serialize".to_string(),
            type_params: vec!["T".to_string()],
            bounds: vec![("T".to_string(), TraitBound::Positive("Serializable".to_string()))],
            params: vec![("item".to_string(), TypeSpec::GenericParam("T".to_string()))],
            return_type: TypeSpec::GenericParam("T".to_string()),
            body: omni_types::ast::Expr::Var("item".to_string()),
        });

        let env = SubstEnv::new();
        let local_vars = HashMap::new();

        let res = checker.infer_call(
            "serialize",
            &[],
            &[omni_types::ast::Expr::Literal(omni_types::ast::Lit::Int(10))],
            &env,
            &local_vars,
        );

        assert!(
            res.is_err(),
            "Mutation test: un-implemented trait obligation MUST fail closed, never succeed"
        );
    }

    #[test]
    fn test_conformance_nested_generic_trait_obligations() {
        use omni_types::ast::{Expr, GenericFnDef, Lit};

        let mut ts = TraitSystem::new();
        let mut checker = TypeChecker::new();

        ts.register_trait(TraitDef {
            name: "Display".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();

        // Implement Display for String (only)
        ts.solver.add_fact("Display", "String");

        let ts_arc = Arc::new(ts);
        ts_arc.attach_to_checker(&mut checker);

        // inner[T: Display](x: T) -> T
        checker.register_fn(GenericFnDef {
            name: "inner".to_string(),
            type_params: vec!["T".to_string()],
            bounds: vec![("T".to_string(), TraitBound::Positive("Display".to_string()))],
            params: vec![("x".to_string(), TypeSpec::GenericParam("T".to_string()))],
            return_type: TypeSpec::GenericParam("T".to_string()),
            body: Expr::Var("x".to_string()),
        });

        // outer[T: Display](x: T) -> T calls inner[T](x)
        checker.register_fn(GenericFnDef {
            name: "outer".to_string(),
            type_params: vec!["T".to_string()],
            bounds: vec![("T".to_string(), TraitBound::Positive("Display".to_string()))],
            params: vec![("x".to_string(), TypeSpec::GenericParam("T".to_string()))],
            return_type: TypeSpec::GenericParam("T".to_string()),
            body: Expr::Call {
                func: "inner".to_string(),
                generic_args: vec![TypeSpec::GenericParam("T".to_string())],
                args: vec![Expr::Var("x".to_string())],
            },
        });

        let mut mono = omni_types::Monomorphizer::new(&mut checker);

        // 1. Specialize outer with concrete String: String implements Display -> SUCCESS
        let prog_res = mono.monomorphize_entry(
            "outer",
            &[],
            &[Expr::Literal(Lit::String("hello".to_string()))],
        );
        assert!(prog_res.is_ok(), "Expected outer[String] to succeed and satisfy inner[String]");
        let prog = prog_res.unwrap();
        assert!(prog.assert_concrete_for_mir().is_ok());

        // 2. Try calling outer with Int (42): Int does NOT implement Display -> FAILS CLOSED
        let fail_res = mono.monomorphize_entry("outer", &[], &[Expr::Literal(Lit::Int(42))]);
        assert!(fail_res.is_err(), "Expected outer[Int] to fail closed on Display obligation");
    }

    #[test]
    fn test_conformance_supertrait_transitive_satisfaction() {
        let mut ts = TraitSystem::new();
        let mut checker = TypeChecker::new();

        // Trait Base
        ts.register_trait(TraitDef {
            name: "Base".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();

        // Trait Derived : Base
        ts.register_trait(TraitDef {
            name: "Derived".to_string(),
            supertraits: vec!["Base".to_string()],
            methods: vec![],
            is_local: true,
        })
        .unwrap();

        // Implement Derived for i64
        let tcx = TyCtxt::new();
        ts.register_impl(
            ImplDef {
                trait_name: "Derived".to_string(),
                target_ty: TypeSpec::Int,
                conditions: vec![],
                methods: vec![],
                is_local: true,
            },
            &tcx,
        )
        .unwrap();

        let ts_arc = Arc::new(ts);
        ts_arc.attach_to_checker(&mut checker);

        // Function requiring Base bound
        checker.register_fn(omni_types::ast::GenericFnDef {
            name: "require_base".to_string(),
            type_params: vec!["T".to_string()],
            bounds: vec![("T".to_string(), TraitBound::Positive("Base".to_string()))],
            params: vec![("x".to_string(), TypeSpec::GenericParam("T".to_string()))],
            return_type: TypeSpec::GenericParam("T".to_string()),
            body: omni_types::ast::Expr::Var("x".to_string()),
        });

        let env = SubstEnv::new();
        let local_vars = HashMap::new();

        // Int satisfies Base transitively through supertrait Derived
        let res = checker.infer_call(
            "require_base",
            &[],
            &[omni_types::ast::Expr::Literal(omni_types::ast::Lit::Int(10))],
            &env,
            &local_vars,
        );
        assert!(
            res.is_ok(),
            "Transitive supertrait requirement Base must be satisfied by Derived impl"
        );
    }

    #[test]
    fn test_conformance_negative_bound_rejection() {
        let mut ts = TraitSystem::new();
        let mut checker = TypeChecker::new();

        ts.register_trait(TraitDef {
            name: "ThreadSafe".to_string(),
            supertraits: vec![],
            methods: vec![],
            is_local: true,
        })
        .unwrap();

        // i64 is marked ThreadSafe
        ts.solver.add_fact("ThreadSafe", "i64");

        let ts_arc = Arc::new(ts);
        ts_arc.attach_to_checker(&mut checker);

        // Function requiring !ThreadSafe
        checker.register_fn(omni_types::ast::GenericFnDef {
            name: "require_thread_local".to_string(),
            type_params: vec!["T".to_string()],
            bounds: vec![("T".to_string(), TraitBound::Negative("ThreadSafe".to_string()))],
            params: vec![("x".to_string(), TypeSpec::GenericParam("T".to_string()))],
            return_type: TypeSpec::GenericParam("T".to_string()),
            body: omni_types::ast::Expr::Var("x".to_string()),
        });

        let env = SubstEnv::new();
        let local_vars = HashMap::new();

        // Calling with i64 must fail because i64 is positive ThreadSafe
        let res = checker.infer_call(
            "require_thread_local",
            &[],
            &[omni_types::ast::Expr::Literal(omni_types::ast::Lit::Int(100))],
            &env,
            &local_vars,
        );
        assert!(
            res.is_err(),
            "Type satisfying positive trait must conflict with negative trait bound"
        );
    }
}
