//! Maranget/Matrix usefulness and exhaustiveness checking engine for Omni patterns.
//! Implements Maranget's matrix usefulness algorithm extended with:
//! - Wildcard & Variable bindings
//! - Literal patterns (Int, Float, Bool, Char, Byte, String)
//! - Tuples & Struct patterns
//! - Enum variants (Option, Result, custom user ADTs)
//! - Numeric interval range patterns, including trailing-unbounded ranges
//! - Or-patterns (`P1 | P2`)
//! - Never type (`!`) uninhabited pattern handling
//! - Guarded arms (which do not contribute to unconditional exhaustiveness)

use std::collections::HashMap;

use crate::ast::{EnumDef, Lit, MatchArm, Pattern, PatternRangeBoundary};
use crate::checker::TypeError;
use crate::intern::{Ty, TyCtxt, TyKind};

type Numeric = i128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NumericRange {
    lo: Numeric,
    hi: Numeric,
}

impl NumericRange {
    fn new(lo: Numeric, hi: Numeric) -> Option<Self> {
        (lo <= hi).then_some(Self { lo, hi })
    }

    fn overlaps_or_touches(self, other: Self) -> bool {
        self.lo <= other.hi.saturating_add(1) && other.lo <= self.hi.saturating_add(1)
    }
}

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
    pub tcx: &'a mut TyCtxt,
    pub enum_defs: &'a HashMap<String, EnumDef>,
}

impl<'a> PatternChecker<'a> {
    pub fn new(tcx: &'a mut TyCtxt, enum_defs: &'a HashMap<String, EnumDef>) -> Self {
        Self { tcx, enum_defs }
    }

    /// Primary entry point: validates usefulness of all arms and exhaustiveness of the pattern matrix.
    pub fn check_match(
        &mut self,
        scrutinee_ty: Ty,
        arms: &[MatchArm],
    ) -> Result<MatchAnalysisResult, TypeError> {
        if numeric_scrutinee(self.tcx.get(scrutinee_ty))
            && arms
                .iter()
                .all(|arm| numeric_pattern_for_type(&arm.pattern, self.tcx.get(scrutinee_ty)))
        {
            return self.check_numeric_match(scrutinee_ty, arms);
        }
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

    fn check_numeric_match(
        &self,
        scrutinee_ty: Ty,
        arms: &[MatchArm],
    ) -> Result<MatchAnalysisResult, TypeError> {
        let domain = numeric_domain(self.tcx.get(scrutinee_ty));
        let mut covered = Vec::<NumericRange>::new();
        let mut unreachable_arms = Vec::new();

        for (idx, arm) in arms.iter().enumerate() {
            let mut useful = false;
            for pattern in self.expand_or_pattern(&arm.pattern) {
                let ranges = numeric_pattern_ranges(&pattern, domain);
                if ranges
                    .iter()
                    .any(|range| range_has_uncovered(*range, &covered, self.tcx.get(scrutinee_ty)))
                {
                    useful = true;
                }
            }

            if !useful && arm.guard.is_none() {
                unreachable_arms.push(idx);
            }

            if arm.guard.is_none() {
                for pattern in self.expand_or_pattern(&arm.pattern) {
                    covered.extend(numeric_pattern_ranges(&pattern, domain));
                }
                normalize_ranges(&mut covered);
            }
        }

        let missing = first_uncovered(domain, &covered, self.tcx.get(scrutinee_ty));
        if let Some(value) = missing {
            return Err(TypeError::NonExhaustiveMatch {
                scrutinee_ty: self.tcx.mangle(scrutinee_ty),
                missing: numeric_witness(self.tcx.get(scrutinee_ty), value),
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
    fn is_useful(&mut self, matrix: &[PatternRow], vector: &PatternRow, tys: Vec<Ty>) -> bool {
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
            Pattern::Tuple(_) | Pattern::Struct { .. } | Pattern::Reference { .. } => {
                Some(Constructor::Single)
            }
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

    fn specialize_matrix(
        &mut self,
        c: &Constructor,
        matrix: &[PatternRow],
        ty: Ty,
    ) -> Vec<PatternRow> {
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

    fn specialize_vector(&mut self, c: &Constructor, vector: &PatternRow, ty: Ty) -> PatternRow {
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

    fn specialize_types(&mut self, c: &Constructor, tys: &[Ty], first_ty: Ty) -> Vec<Ty> {
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

    fn constructor_payload_types(&mut self, c: &Constructor, ty: Ty) -> Vec<Ty> {
        match c {
            Constructor::Single => match self.tcx.get(ty) {
                TyKind::Tuple(tys) | TyKind::Struct(_, tys) => tys.clone(),
                _ => vec![],
            },
            Constructor::Variant { name, .. } => match self.tcx.get(ty).clone() {
                TyKind::Enum(enum_name, type_args) => {
                    if let Some(def) = self.enum_defs.get(&enum_name).cloned() {
                        if let Some(var) = def.variants.iter().find(|v| &v.name == name) {
                            let mut subst = crate::checker::SubstEnv::new();
                            for (param, &arg) in def.type_params.iter().zip(&type_args) {
                                subst.insert(param.clone(), arg);
                            }
                            return var
                                .payload
                                .iter()
                                .map(|spec| self.tcx.lower_type_spec(spec, &subst))
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

fn numeric_scrutinee(kind: &TyKind) -> bool {
    matches!(kind, TyKind::Int | TyKind::Byte | TyKind::Char)
}

fn numeric_literal_for_type(lit: &Lit, kind: &TyKind) -> bool {
    matches!(
        (lit, kind),
        (Lit::Int(_), TyKind::Int) | (Lit::Byte(_), TyKind::Byte) | (Lit::Char(_), TyKind::Char)
    )
}

fn numeric_boundary_for_type(boundary: &PatternRangeBoundary, kind: &TyKind) -> bool {
    match boundary {
        PatternRangeBoundary::Unbounded => true,
        PatternRangeBoundary::Inclusive(lit) | PatternRangeBoundary::Exclusive(lit) => {
            numeric_literal_for_type(lit, kind)
        }
    }
}

fn numeric_pattern_for_type(pattern: &Pattern, kind: &TyKind) -> bool {
    match pattern {
        Pattern::Wildcard | Pattern::Binding(_) => true,
        Pattern::Lit(lit) => numeric_literal_for_type(lit, kind),
        Pattern::Range { start, end } => {
            numeric_boundary_for_type(start, kind) && numeric_boundary_for_type(end, kind)
        }
        Pattern::Or(patterns) => {
            patterns.iter().all(|pattern| numeric_pattern_for_type(pattern, kind))
        }
        _ => false,
    }
}

fn numeric_domain(kind: &TyKind) -> (Numeric, Numeric) {
    match kind {
        TyKind::Int => (i64::MIN as Numeric, i64::MAX as Numeric),
        TyKind::Byte => (0, u8::MAX as Numeric),
        TyKind::Char => (0, char::MAX as Numeric),
        _ => unreachable!("numeric pattern checker called for non-numeric type"),
    }
}

fn lit_numeric(lit: &Lit) -> Option<Numeric> {
    match lit {
        Lit::Int(value) => Some(*value as Numeric),
        Lit::Byte(value) => Some(*value as Numeric),
        Lit::Char(value) => Some(*value as Numeric),
        _ => None,
    }
}

fn numeric_pattern_ranges(pattern: &Pattern, domain: (Numeric, Numeric)) -> Vec<NumericRange> {
    match pattern {
        Pattern::Wildcard | Pattern::Binding(_) => {
            vec![NumericRange { lo: domain.0, hi: domain.1 }]
        }
        Pattern::Lit(lit) => lit_numeric(lit)
            .and_then(|value| NumericRange::new(value, value))
            .into_iter()
            .filter(|r| r.lo >= domain.0 && r.hi <= domain.1)
            .collect(),
        Pattern::Range { start, end } => {
            let lo = match start {
                PatternRangeBoundary::Unbounded => domain.0,
                PatternRangeBoundary::Inclusive(lit) => lit_numeric(lit).unwrap_or(domain.0),
                PatternRangeBoundary::Exclusive(lit) => {
                    lit_numeric(lit).map_or(domain.0, |value| value.saturating_add(1))
                }
            };
            let hi = match end {
                PatternRangeBoundary::Unbounded => domain.1,
                PatternRangeBoundary::Inclusive(lit) => lit_numeric(lit).unwrap_or(domain.1),
                PatternRangeBoundary::Exclusive(lit) => {
                    lit_numeric(lit).map_or(domain.1, |value| value.saturating_sub(1))
                }
            };
            NumericRange::new(lo.max(domain.0), hi.min(domain.1))
                .map(|range| {
                    char_valid_ranges(range, domain.0 == 0 && domain.1 == char::MAX as Numeric)
                })
                .unwrap_or_default()
        }
        Pattern::Or(patterns) => {
            patterns.iter().flat_map(|pattern| numeric_pattern_ranges(pattern, domain)).collect()
        }
        _ => Vec::new(),
    }
}

fn char_valid_ranges(range: NumericRange, is_char_domain: bool) -> Vec<NumericRange> {
    if !is_char_domain || range.hi < 0xD800 || range.lo > 0xDFFF {
        return vec![range];
    }
    let mut result = Vec::new();
    if range.lo < 0xD800 {
        if let Some(prefix) = NumericRange::new(range.lo, 0xD7FF.min(range.hi)) {
            result.push(prefix);
        }
    }
    if range.hi > 0xDFFF {
        if let Some(suffix) = NumericRange::new(0xE000.max(range.lo), range.hi) {
            result.push(suffix);
        }
    }
    result
}

fn range_has_uncovered(range: NumericRange, covered: &[NumericRange], kind: &TyKind) -> bool {
    let mut cursor = range.lo;
    for existing in covered {
        if existing.hi < cursor {
            continue;
        }
        if existing.lo > cursor {
            return true;
        }
        cursor = cursor.max(existing.hi.saturating_add(1));
        if matches!(kind, TyKind::Char) && (0xD800..=0xDFFF).contains(&(cursor as u32)) {
            cursor = 0xE000;
        }
        if cursor > range.hi {
            return false;
        }
    }
    cursor <= range.hi
}

fn normalize_ranges(ranges: &mut Vec<NumericRange>) {
    ranges.sort_by_key(|range| (range.lo, range.hi));
    let mut normalized: Vec<NumericRange> = Vec::with_capacity(ranges.len());
    for range in ranges.drain(..) {
        if let Some(last) = normalized.last_mut() {
            if last.overlaps_or_touches(range) {
                last.hi = last.hi.max(range.hi);
                continue;
            }
        }
        normalized.push(range);
    }
    *ranges = normalized;
}

fn first_uncovered(
    domain: (Numeric, Numeric),
    covered: &[NumericRange],
    kind: &TyKind,
) -> Option<Numeric> {
    let mut cursor = domain.0;
    if matches!(kind, TyKind::Char) && (0xD800..=0xDFFF).contains(&(cursor as u32)) {
        cursor = 0xE000;
    }
    for range in covered {
        if range.hi < cursor {
            continue;
        }
        if range.lo > cursor {
            return Some(cursor);
        }
        cursor = cursor.max(range.hi.saturating_add(1));
        if matches!(kind, TyKind::Char) && (0xD800..=0xDFFF).contains(&(cursor as u32)) {
            cursor = 0xE000;
        }
        if cursor > domain.1 {
            return None;
        }
    }
    (cursor <= domain.1).then_some(cursor)
}

fn numeric_witness(kind: &TyKind, value: Numeric) -> String {
    match kind {
        TyKind::Char => char::from_u32(value as u32)
            .map(|c| format!("{:?}", c))
            .unwrap_or_else(|| "_".to_string()),
        _ => value.to_string(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn int_tcx() -> TyCtxt {
        TyCtxt::new()
    }

    fn int_ty(tcx: &mut TyCtxt) -> Ty {
        tcx.intern(TyKind::Int)
    }

    fn arm(pattern: Pattern) -> MatchArm {
        MatchArm { pattern, guard: None, body: crate::ast::Expr::Literal(Lit::Int(0)) }
    }

    #[test]
    fn unbounded_integer_ranges_can_be_exhaustive() {
        let mut tcx = int_tcx();
        let ty = int_ty(&mut tcx);
        let enum_defs = HashMap::new();
        let mut checker = PatternChecker::new(&mut tcx, &enum_defs);
        let arms = vec![
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Int(i64::MIN)),
                end: PatternRangeBoundary::Exclusive(Lit::Int(0)),
            }),
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Int(0)),
                end: PatternRangeBoundary::Unbounded,
            }),
        ];

        checker.check_match(ty, &arms).expect("ranges should cover all integers");
    }

    #[test]
    fn overlapping_numeric_range_makes_later_literal_unreachable() {
        let mut tcx = int_tcx();
        let ty = int_ty(&mut tcx);
        let enum_defs = HashMap::new();
        let mut checker = PatternChecker::new(&mut tcx, &enum_defs);
        let arms = vec![
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Int(0)),
                end: PatternRangeBoundary::Inclusive(Lit::Int(10)),
            }),
            arm(Pattern::Lit(Lit::Int(5))),
            arm(Pattern::Wildcard),
        ];

        let error =
            checker.check_match(ty, &arms).expect_err("overlapping literal must be unreachable");
        assert!(matches!(error, TypeError::UnreachablePattern { arm_index: 1, .. }));
    }

    #[test]
    fn integer_range_reports_lowest_missing_witness() {
        let mut tcx = int_tcx();
        let ty = int_ty(&mut tcx);
        let enum_defs = HashMap::new();
        let mut checker = PatternChecker::new(&mut tcx, &enum_defs);
        let arms = vec![
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Int(-10)),
                end: PatternRangeBoundary::Inclusive(Lit::Int(-1)),
            }),
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Int(1)),
                end: PatternRangeBoundary::Inclusive(Lit::Int(10)),
            }),
        ];

        let error = checker
            .check_match(ty, &arms)
            .expect_err("zero and exterior integers remain uncovered");
        assert!(matches!(
            error,
            TypeError::NonExhaustiveMatch { missing, .. } if missing == i64::MIN.to_string()
        ));
    }

    #[test]
    fn guarded_overlapping_range_remains_potentially_reachable() {
        let mut tcx = int_tcx();
        let ty = int_ty(&mut tcx);
        let enum_defs = HashMap::new();
        let mut checker = PatternChecker::new(&mut tcx, &enum_defs);
        let arms = vec![
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Int(0)),
                end: PatternRangeBoundary::Inclusive(Lit::Int(10)),
            }),
            MatchArm {
                pattern: Pattern::Range {
                    start: PatternRangeBoundary::Inclusive(Lit::Int(5)),
                    end: PatternRangeBoundary::Inclusive(Lit::Int(15)),
                },
                guard: Some(crate::ast::Expr::Literal(Lit::Bool(true))),
                body: crate::ast::Expr::Literal(Lit::Int(0)),
            },
            arm(Pattern::Range {
                start: PatternRangeBoundary::Exclusive(Lit::Int(10)),
                end: PatternRangeBoundary::Unbounded,
            }),
        ];

        checker.check_match(ty, &arms).expect("guarded overlap must not make the arm unreachable");
    }

    #[test]
    fn split_character_scalar_ranges_are_exhaustive() {
        let mut tcx = int_tcx();
        let ty = tcx.intern(TyKind::Char);
        let enum_defs = HashMap::new();
        let mut checker = PatternChecker::new(&mut tcx, &enum_defs);
        let arms = vec![
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Char('\0')),
                end: PatternRangeBoundary::Inclusive(Lit::Char('\u{D7FF}')),
            }),
            arm(Pattern::Range {
                start: PatternRangeBoundary::Inclusive(Lit::Char('\u{E000}')),
                end: PatternRangeBoundary::Inclusive(Lit::Char(char::MAX)),
            }),
        ];

        checker.check_match(ty, &arms).expect("surrogate gap is not inhabited by Char");
    }

    #[test]
    fn character_ranges_skip_surrogate_code_points() {
        let mut tcx = int_tcx();
        let ty = tcx.intern(TyKind::Char);
        let enum_defs = HashMap::new();
        let mut checker = PatternChecker::new(&mut tcx, &enum_defs);
        let arms = vec![arm(Pattern::Range {
            start: PatternRangeBoundary::Inclusive(Lit::Char('\0')),
            end: PatternRangeBoundary::Inclusive(Lit::Char(char::MAX)),
        })];

        checker
            .check_match(ty, &arms)
            .expect("full scalar range should cover Unicode scalar values");
    }
}

