//! Maranget/Matrix usefulness and exhaustiveness checking engine for Omni patterns.
//! Implements Maranget's matrix usefulness algorithm extended with:
//! - Wildcard & Variable bindings
//! - Literal patterns (Int, Float, Bool, Char, Byte, String)
//! - Tuples & Struct patterns
//! - Enum variants (Option, Result, custom user ADTs)
//! - Bounded Range patterns
//! - Or-patterns (`P1 | P2`)
//! - Never type (`!`) uninhabited pattern handling
//! - Guarded arms (which do not contribute to unconditional exhaustiveness)

use std::collections::HashMap;

use crate::ast::{EnumDef, Lit, MatchArm, Pattern, PatternRangeBoundary};
use crate::checker::TypeError;
use crate::intern::{Ty, TyCtxt, TyKind};

/// Constructor representing the top-level head of a pattern matrix column.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constructor {
    Single,                                 // Wildcard, Binding, Tuple, Struct, Unit
    Bool(bool),                             // true or false
    Lit(Lit),                               // Int, Char, Byte, String literal
    Variant { name: String, arity: usize }, // Enum variant (e.g. Some, None, Ok, Err)
    Range { start: u64, end: u64, inclusive: bool },
    Never, // Uninhabited Never type
}

/// A simplified pattern matrix row used during Maranget reduction.
#[derive(Debug, Clone)]
pub struct PatternRow {
    pub pats: Vec<Pattern>,
    pub guard: bool, // true if arm has a guard expression
}

/// Result of evaluating a match matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchAnalysisResult {
    pub is_exhaustive: bool,
    pub missing_witness: Option<String>,
    pub unreachable_arms: Vec<usize>,
}

pub struct PatternChecker<'a> {
    pub tcx: &'a TyCtxt,
    pub enum_defs: &'a HashMap<String, EnumDef>,
}

impl<'a> PatternChecker<'a> {
    pub fn new(tcx: &'a TyCtxt, enum_defs: &'a HashMap<String, EnumDef>) -> Self {
        Self { tcx, enum_defs }
    }

    /// Primary entry point: validates usefulness of all arms and exhaustiveness of the pattern matrix.
    pub fn check_match(
        &self,
        scrutinee_ty: Ty,
        arms: &[MatchArm],
    ) -> Result<MatchAnalysisResult, TypeError> {
        let mut matrix: Vec<PatternRow> = Vec::new();
        let mut unreachable_arms = Vec::new();

        for (idx, arm) in arms.iter().enumerate() {
            // Expand Or-patterns in arm into distinct rows
            let expanded_pats = self.expand_or_pattern(&arm.pattern);
            let mut useful = false;

            for pat in expanded_pats {
                let row = PatternRow { pats: vec![pat], guard: arm.guard.is_some() };
                if self.is_useful(&matrix, &row, vec![scrutinee_ty]) {
                    useful = true;
                    // Only unguarded arms contribute to matrix reduction for subsequent arms
                    if arm.guard.is_none() {
                        matrix.push(row);
                    }
                }
            }

            if !useful {
                unreachable_arms.push(idx);
            }
        }

        // Exhaustiveness test: check if wildcard row `[_]` is useful against unguarded matrix
        let is_exhaustive = if matches!(self.tcx.get(scrutinee_ty), TyKind::Never) {
            true
        } else {
            let wildcard_row = PatternRow { pats: vec![Pattern::Wildcard], guard: false };
            !self.is_useful(&matrix, &wildcard_row, vec![scrutinee_ty])
        };

        let missing_witness = if !is_exhaustive {
            Some(self.synthesize_witness(&matrix, scrutinee_ty))
        } else {
            None
        };

        if !is_exhaustive {
            return Err(TypeError::NonExhaustiveMatch {
                scrutinee_ty: self.tcx.mangle(scrutinee_ty),
                missing: missing_witness.unwrap_or_else(|| "_".to_string()),
            });
        }

        if let Some(&first_unreachable) = unreachable_arms.first() {
            return Err(TypeError::UnreachablePattern {
                arm_index: first_unreachable,
                detail: format!("arm {first_unreachable} is never reached"),
            });
        }

        Ok(MatchAnalysisResult {
            is_exhaustive: true,
            missing_witness: None,
            unreachable_arms: Vec::new(),
        })
    }

    /// Evaluates whether `vector` is useful given `matrix` under types `tys`.
    fn is_useful(&self, matrix: &[PatternRow], vector: &PatternRow, tys: Vec<Ty>) -> bool {
        if vector.pats.is_empty() {
            return matrix.is_empty();
        }

        let first_ty = tys[0];
        let first_pat = &vector.pats[0];

        if matches!(self.tcx.get(first_ty), TyKind::Never) {
            return matrix.is_empty();
        }

        match first_pat {
            Pattern::Never => matrix.is_empty(),
            Pattern::Or(subpats) => {
                for sub in subpats {
                    let mut new_vec = vector.clone();
                    new_vec.pats[0] = sub.clone();
                    if self.is_useful(matrix, &new_vec, tys.clone()) {
                        return true;
                    }
                }
                false
            }
            _ => {
                let constructors = self.constructors_of_type(first_ty);

                if let Some(c) = self.head_constructor(first_pat) {
                    let spec_matrix = self.specialize_matrix(&c, matrix, first_ty);
                    let spec_vector = self.specialize_vector(&c, vector, first_ty);
                    let spec_tys = self.specialize_types(&c, &tys, first_ty);
                    self.is_useful(&spec_matrix, &spec_vector, spec_tys)
                } else {
                    // Wildcard / Binding
                    if !constructors.is_empty()
                        && self.is_complete_set(&constructors, matrix, first_ty)
                    {
                        // Complete set: specialize against all constructors
                        constructors.iter().any(|c| {
                            let spec_matrix = self.specialize_matrix(c, matrix, first_ty);
                            let spec_vector = self.specialize_vector(c, vector, first_ty);
                            let spec_tys = self.specialize_types(c, &tys, first_ty);
                            self.is_useful(&spec_matrix, &spec_vector, spec_tys)
                        })
                    } else {
                        // Incomplete set: default matrix reduction
                        let def_matrix = self.default_matrix(matrix);
                        let def_vector =
                            PatternRow { pats: vector.pats[1..].to_vec(), guard: vector.guard };
                        self.is_useful(&def_matrix, &def_vector, tys[1..].to_vec())
                    }
                }
            }
        }
    }

    fn expand_or_pattern(&self, pat: &Pattern) -> Vec<Pattern> {
        match pat {
            Pattern::Or(subs) => subs.iter().flat_map(|s| self.expand_or_pattern(s)).collect(),
            _ => vec![pat.clone()],
        }
    }

    fn head_constructor(&self, pat: &Pattern) -> Option<Constructor> {
        match pat {
            Pattern::Lit(lit) => match lit {
                Lit::Bool(b) => Some(Constructor::Bool(*b)),
                _ => Some(Constructor::Lit(lit.clone())),
            },
            Pattern::Variant { variant, subpatterns, .. } => {
                Some(Constructor::Variant { name: variant.clone(), arity: subpatterns.len() })
            }
            Pattern::Range { start, end } => {
                let s_val = self.boundary_to_u64(start);
                let e_val = self.boundary_to_u64(end);
                let is_inc = matches!(end, PatternRangeBoundary::Inclusive(_));
                Some(Constructor::Range { start: s_val, end: e_val, inclusive: is_inc })
            }
            Pattern::Tuple(_) | Pattern::Struct { .. } => Some(Constructor::Single),
            Pattern::Never => Some(Constructor::Never),
            Pattern::Wildcard | Pattern::Binding(_) => None,
            Pattern::Or(_) => None,
        }
    }

    fn boundary_to_u64(&self, b: &PatternRangeBoundary) -> u64 {
        match b {
            PatternRangeBoundary::Inclusive(Lit::Int(n))
            | PatternRangeBoundary::Exclusive(Lit::Int(n)) => *n as u64,
            PatternRangeBoundary::Inclusive(Lit::Byte(b))
            | PatternRangeBoundary::Exclusive(Lit::Byte(b)) => *b as u64,
            PatternRangeBoundary::Inclusive(Lit::Char(c))
            | PatternRangeBoundary::Exclusive(Lit::Char(c)) => *c as u64,
            _ => 0,
        }
    }

    fn constructors_of_type(&self, ty: Ty) -> Vec<Constructor> {
        match self.tcx.get(ty) {
            TyKind::Bool => vec![Constructor::Bool(true), Constructor::Bool(false)],
            TyKind::Unit => vec![Constructor::Single],
            TyKind::Never => vec![Constructor::Never],
            TyKind::Tuple(_) | TyKind::Struct(_, _) => vec![Constructor::Single],
            TyKind::Enum(name, _) => {
                if let Some(def) = self.enum_defs.get(name) {
                    def.variants
                        .iter()
                        .map(|v| Constructor::Variant {
                            name: v.name.clone(),
                            arity: v.payload.len(),
                        })
                        .collect()
                } else if name == "Option" {
                    vec![
                        Constructor::Variant { name: "Some".to_string(), arity: 1 },
                        Constructor::Variant { name: "None".to_string(), arity: 0 },
                    ]
                } else if name == "Result" {
                    vec![
                        Constructor::Variant { name: "Ok".to_string(), arity: 1 },
                        Constructor::Variant { name: "Err".to_string(), arity: 1 },
                    ]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    fn is_complete_set(
        &self,
        constructors: &[Constructor],
        matrix: &[PatternRow],
        _ty: Ty,
    ) -> bool {
        if constructors.is_empty() {
            return false;
        }
        let seen: std::collections::HashSet<Constructor> = matrix
            .iter()
            .filter_map(|r| r.pats.first().and_then(|p| self.head_constructor(p)))
            .collect();

        constructors.iter().all(|c| seen.contains(c))
    }

    fn specialize_matrix(&self, c: &Constructor, matrix: &[PatternRow], ty: Ty) -> Vec<PatternRow> {
        let mut result = Vec::new();
        for row in matrix {
            if row.pats.is_empty() {
                continue;
            }
            let first = &row.pats[0];
            let rest = &row.pats[1..];

            match first {
                Pattern::Wildcard | Pattern::Binding(_) => {
                    let arity = self.constructor_arity(c, ty);
                    let mut new_pats = vec![Pattern::Wildcard; arity];
                    new_pats.extend_from_slice(rest);
                    result.push(PatternRow { pats: new_pats, guard: row.guard });
                }
                pat => {
                    if let Some(pat_c) = self.head_constructor(pat) {
                        if &pat_c == c {
                            let mut sub_pats = self.extract_subpatterns(pat);
                            sub_pats.extend_from_slice(rest);
                            result.push(PatternRow { pats: sub_pats, guard: row.guard });
                        }
                    }
                }
            }
        }
        result
    }

    fn specialize_vector(&self, c: &Constructor, vector: &PatternRow, ty: Ty) -> PatternRow {
        if vector.pats.is_empty() {
            return vector.clone();
        }
        let first = &vector.pats[0];
        let rest = &vector.pats[1..];

        let mut sub_pats = match first {
            Pattern::Wildcard | Pattern::Binding(_) => {
                vec![Pattern::Wildcard; self.constructor_arity(c, ty)]
            }
            pat => self.extract_subpatterns(pat),
        };
        sub_pats.extend_from_slice(rest);
        PatternRow { pats: sub_pats, guard: vector.guard }
    }

    fn specialize_types(&self, c: &Constructor, tys: &[Ty], first_ty: Ty) -> Vec<Ty> {
        let mut result = Vec::new();
        let payload_tys = self.constructor_payload_types(c, first_ty);
        result.extend(payload_tys);
        result.extend_from_slice(&tys[1..]);
        result
    }

    fn default_matrix(&self, matrix: &[PatternRow]) -> Vec<PatternRow> {
        matrix
            .iter()
            .filter(|r| {
                if r.pats.is_empty() {
                    false
                } else {
                    matches!(r.pats[0], Pattern::Wildcard | Pattern::Binding(_))
                }
            })
            .map(|r| PatternRow { pats: r.pats[1..].to_vec(), guard: r.guard })
            .collect()
    }

    fn constructor_arity(&self, c: &Constructor, ty: Ty) -> usize {
        match c {
            Constructor::Single => match self.tcx.get(ty) {
                TyKind::Tuple(tys) => tys.len(),
                TyKind::Struct(_, fields) => fields.len(),
                _ => 0,
            },
            Constructor::Variant { arity, .. } => *arity,
            _ => 0,
        }
    }

    fn constructor_payload_types(&self, c: &Constructor, ty: Ty) -> Vec<Ty> {
        match c {
            Constructor::Single => match self.tcx.get(ty) {
                TyKind::Tuple(tys) | TyKind::Struct(_, tys) => tys.clone(),
                _ => vec![],
            },
            Constructor::Variant { name, .. } => match self.tcx.get(ty) {
                TyKind::Enum(enum_name, type_args) => {
                    if let Some(def) = self.enum_defs.get(enum_name) {
                        if let Some(var) = def.variants.iter().find(|v| &v.name == name) {
                            let mut subst = crate::checker::SubstEnv::new();
                            for (param, &arg) in def.type_params.iter().zip(type_args) {
                                subst.insert(param.clone(), arg);
                            }
                            let mut dummy_checker = crate::checker::TypeChecker::new();
                            return var
                                .payload
                                .iter()
                                .map(|spec| dummy_checker.lower_type_spec(spec, &subst))
                                .collect();
                        }
                    }
                    if enum_name == "Option" && name == "Some" {
                        return type_args.first().cloned().into_iter().collect();
                    }
                    if enum_name == "Result" {
                        if name == "Ok" {
                            return type_args.first().cloned().into_iter().collect();
                        }
                        if name == "Err" {
                            return type_args.get(1).cloned().into_iter().collect();
                        }
                    }
                    vec![]
                }
                _ => vec![],
            },
            _ => vec![],
        }
    }

    fn extract_subpatterns(&self, pat: &Pattern) -> Vec<Pattern> {
        match pat {
            Pattern::Tuple(pats) => pats.clone(),
            Pattern::Struct { fields, .. } => fields.iter().map(|(_, p)| p.clone()).collect(),
            Pattern::Variant { subpatterns, .. } => subpatterns.clone(),
            _ => vec![],
        }
    }

    fn synthesize_witness(&self, _matrix: &[PatternRow], scrutinee_ty: Ty) -> String {
        match self.tcx.get(scrutinee_ty) {
            TyKind::Bool => "false".to_string(),
            TyKind::Enum(name, _) if name == "Option" => "None".to_string(),
            TyKind::Enum(name, _) if name == "Result" => "Err(_) ".to_string(),
            TyKind::Enum(name, _) => format!("{name}::_"),
            TyKind::Int => "0".to_string(),
            _ => "_".to_string(),
        }
    }
}
