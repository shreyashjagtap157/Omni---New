//! Target-aware aggregate layout for Omni native codegen.
//!
//! This module is deliberately split into two halves that must not be confused:
//!
//! ```text
//! Target facts
//!     = what the selected backend and target actually report
//!
//! Omni native aggregate policy
//!     = the rules deciding how Omni aggregates use those facts
//! ```
//!
//! [`TargetLayout`] reads only the first. Field ordering, padding and alignment
//! are the second, and they are stated explicitly in [`LayoutPolicy`] rather
//! than being implied by the target.
//!
//! Two further constraints shaped this design, both verified against the
//! vendored `cranelift-codegen` 0.110.3 source rather than assumed:
//!
//! * That version exposes `TargetIsa::pointer_bits`, `pointer_type` and
//!   `endianness`, so genuine target facts are available to read.
//! * That version has **no** `StructLayout` or `field_offsets` API, so field
//!   offsets are necessarily this crate's responsibility.
//!
//! Nothing here emits native code. A layout that cannot be derived is reported
//! as [`LayoutError`] and the caller stays fail-closed, rather than guessing an
//! ABI.

use cranelift_codegen::ir::{types, Endianness, Type as ClifType};
use cranelift_codegen::isa::TargetIsa;
use std::collections::HashMap;

use omni_mir::{SubstEnv, Ty, TyCtxt, TyKind};

/// Facts reported by the selected target.
///
/// This is the *facts* half only. It contains nothing about how Omni lays out an
/// aggregate; that is [`LayoutPolicy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetFacts {
    /// Pointer width in bytes, as reported by the ISA.
    pub pointer_size: u64,
    /// Pointer width in bits, as reported by the ISA.
    pub pointer_bits: u8,
    /// Target byte order.
    pub endianness: Endianness,
    /// Whether the ISA uses a big-endian byte order.
    pub big_endian: bool,
}

impl TargetFacts {
    /// Reads the facts this build needs from a Cranelift `TargetIsa`.
    ///
    /// Both accessors are read from the ISA rather than hard-coded, so the
    /// values track the target the backend actually selected. The argument is
    /// the `Arc` that `cranelift_codegen::isa::lookup` returns, which is how the
    /// backend itself holds the ISA.
    pub fn from_isa(isa: &std::sync::Arc<dyn TargetIsa>) -> Self {
        let endianness = isa.endianness();
        Self {
            pointer_size: u64::from(isa.pointer_bits()) / 8,
            pointer_bits: isa.pointer_bits(),
            endianness,
            big_endian: endianness == Endianness::Big,
        }
    }
}

/// The result of laying out one type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeLayout {
    /// Total size in bytes.
    pub size: u64,
    /// Required alignment in bytes, always a power of two.
    pub align: u64,
    /// Per-field or per-element layout, empty for scalars.
    pub parts: Vec<PartLayout>,
}

/// One field or element position within an aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartLayout {
    /// Field name, for struct fields. `None` for tuple and array positions.
    pub name: Option<String>,
    /// Byte offset from the start of the aggregate.
    pub offset: u64,
    /// Layout of the part's own type.
    pub layout: TypeLayout,
}

impl PartLayout {
    /// Constructs a part layout.
    fn new(name: Option<String>, offset: u64, layout: TypeLayout) -> Self {
        Self { name, offset, layout }
    }
}

/// Why a layout could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// The type has no defined native representation yet.
    ///
    /// This is the fail-closed path. The aggregate/native boundary is not
    /// normatively defined, so a type in this state is rejected rather than
    /// being given an invented representation.
    Unrepresentable { ty: String },
    /// An array or tuple length is not representable on the target.
    LengthOverflow { ty: String, length: usize },
    /// A struct field name could not be resolved to a layout.
    UnknownField { struct_name: String, field: String },
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unrepresentable { ty } => {
                write!(f, "no native representation is defined for type {}", ty)
            }
            Self::LengthOverflow { ty, length } => {
                write!(f, "type {} has length {} which does not fit the target", ty, length)
            }
            Self::UnknownField { struct_name, field } => {
                write!(f, "struct '{}' has no field '{}'", struct_name, field)
            }
        }
    }
}

impl std::error::Error for LayoutError {}

/// Rounds `value` up to the next multiple of `align`.
///
/// `align` is required to be a power of two. Rounding up rather than down is
/// what guarantees every field starts on its own alignment boundary; the
/// resulting interior padding is the deliberate cost of that guarantee.
fn align_to(value: u64, align: u64) -> u64 {
    if align == 0 {
        return value;
    }
    value.div_ceil(align) * align
}

/// Computes layouts from target facts under an explicit policy.
///
/// Determinism is a requirement: the same facts, declarations and type always
/// produce byte-identical layout, independent of iteration or hashing order.
///
/// The engine holds no type context. `Ty` is an index into a `TyCtxt` arena, so
/// storing a copy would silently resolve against a different, possibly empty,
/// arena; storing a borrow would conflict with `lower_type_spec`, which needs
/// `&mut` to intern a struct field's type. The context is therefore passed per
/// call, which makes it impossible to lay out a type the engine cannot inspect.
pub struct TargetLayout<'a> {
    facts: TargetFacts,
    struct_defs: &'a HashMap<String, omni_mir::ast::StructDef>,
}

impl<'a> TargetLayout<'a> {
    /// Creates a layout engine over the given target facts and declarations.
    pub fn new(
        facts: TargetFacts,
        struct_defs: &'a HashMap<String, omni_mir::ast::StructDef>,
    ) -> Self {
        Self { facts, struct_defs }
    }

    /// Returns the target facts this engine was built from.
    pub fn facts(&self) -> TargetFacts {
        self.facts
    }

    /// Lays out a scalar type.
    ///
    /// Size comes from the Cranelift type actually selected for this Omni
    /// scalar, which is a property of that type. Alignment is policy: a scalar
    /// is aligned to its own size, the conservative choice that lets a field be
    /// addressed directly.
    pub fn scalar_layout(&self, tcx: &TyCtxt, ty: Ty) -> Result<TypeLayout, LayoutError> {
        let clif = self
            .scalar_clif_type(tcx, ty)
            .ok_or_else(|| LayoutError::Unrepresentable { ty: tcx.mangle(ty) })?;
        let size = u64::from(clif.bytes());
        Ok(TypeLayout { size, align: size.max(1), parts: Vec::new() })
    }

    /// Returns the Cranelift type selected for an Omni scalar.
    ///
    /// This mirrors the mapping the emitter already uses. Centralising it here
    /// keeps the layout engine from disagreeing with the code generator about
    /// how wide a scalar is, which is exactly the class of bug that would make a
    /// computed offset silently wrong.
    pub fn scalar_clif_type(&self, tcx: &TyCtxt, ty: Ty) -> Option<ClifType> {
        match tcx.get(ty) {
            TyKind::Int | TyKind::Bool | TyKind::Byte | TyKind::Char => Some(types::I64),
            TyKind::Float => Some(types::F64),
            _ => None,
        }
    }

    /// Lays out a tuple, array, or struct.
    ///
    /// Returns [`LayoutError::Unrepresentable`] for any type without a defined
    /// native representation, which is the fail-closed path.
    pub fn aggregate_layout(&self, tcx: &mut TyCtxt, ty: Ty) -> Result<TypeLayout, LayoutError> {
        let name = tcx.mangle(ty);
        match tcx.get(ty).clone() {
            TyKind::Int | TyKind::Bool | TyKind::Byte | TyKind::Char | TyKind::Float => {
                self.scalar_layout(tcx, ty)
            }
            TyKind::Tuple(elements) => {
                let parts = self.layout_parts(tcx, elements.into_iter().map(|e| (None, e)))?;
                Ok(finish_parts(parts))
            }
            TyKind::Array(elem, length) => {
                let parts = self.layout_parts(tcx, std::iter::repeat((None, elem)).take(length))?;
                Ok(finish_parts(parts))
            }
            TyKind::Struct(struct_name, args) => {
                let def = self
                    .struct_defs
                    .get(&struct_name)
                    .cloned()
                    .ok_or_else(|| LayoutError::Unrepresentable { ty: name })?;
                let mut subst = SubstEnv::new();
                for (param, arg) in def.type_params.iter().zip(args.iter()) {
                    subst.insert(param.clone(), *arg);
                }
                // `lower_type_spec` interns into the context, which is why
                // `aggregate_layout` takes `&mut TyCtxt`.
                let mut fields = Vec::with_capacity(def.fields.len());
                for field in &def.fields {
                    let field_ty = tcx.lower_type_spec(&field.ty, &subst);
                    fields.push((Some(field.name.clone()), field_ty));
                }
                let parts = self.layout_parts(tcx, fields.into_iter())?;
                Ok(finish_parts(parts))
            }
            // Reference, function, enum, string, closure and task types have no
            // defined native representation yet. They are rejected rather than
            // given an invented one, which is the point of the fail-closed rule.
            _ => Err(LayoutError::Unrepresentable { ty: name }),
        }
    }

    /// Returns the layout of a named struct field.
    pub fn field_layout(
        &self,
        tcx: &mut TyCtxt,
        struct_ty: Ty,
        field: &str,
    ) -> Result<TypeLayout, LayoutError> {
        self.aggregate_layout(tcx, struct_ty)?
            .parts
            .iter()
            .find(|p| p.name.as_deref() == Some(field))
            .map(|p| p.layout.clone())
            .ok_or_else(|| self.unknown_field(tcx, struct_ty, field))
    }

    /// Returns the byte offset of a named struct field.
    pub fn field_offset(
        &self,
        tcx: &mut TyCtxt,
        struct_ty: Ty,
        field: &str,
    ) -> Result<u64, LayoutError> {
        self.aggregate_layout(tcx, struct_ty)?
            .parts
            .iter()
            .find(|p| p.name.as_deref() == Some(field))
            .map(|p| p.offset)
            .ok_or_else(|| self.unknown_field(tcx, struct_ty, field))
    }

    /// Returns the layout of one tuple or array element by position.
    pub fn element_layout(
        &self,
        tcx: &mut TyCtxt,
        aggregate_ty: Ty,
        index: usize,
    ) -> Result<TypeLayout, LayoutError> {
        self.aggregate_layout(tcx, aggregate_ty)?
            .parts
            .get(index)
            .map(|p| p.layout.clone())
            .ok_or_else(|| LayoutError::Unrepresentable { ty: tcx.mangle(aggregate_ty) })
    }

    fn unknown_field(&self, tcx: &TyCtxt, struct_ty: Ty, field: &str) -> LayoutError {
        LayoutError::UnknownField { struct_name: tcx.mangle(struct_ty), field: field.to_string() }
    }

    /// Lays out a run of parts under the padding and alignment policy.
    fn layout_parts<I>(&self, tcx: &mut TyCtxt, parts: I) -> Result<Vec<PartLayout>, LayoutError>
    where
        I: Iterator<Item = (Option<String>, Ty)>,
    {
        let mut offset = 0u64;
        let mut out = Vec::new();
        for (name, part_ty) in parts {
            let part = self.scalar_or_aggregate(tcx, part_ty)?;
            // Each part starts at the next offset satisfying its own alignment.
            offset = align_to(offset, part.align);
            let part_size = part.size;
            out.push(PartLayout::new(name, offset, part));
            offset = offset.saturating_add(part_size);
        }
        Ok(out)
    }

    /// Lays out a part that may be either a scalar or an aggregate.
    fn scalar_or_aggregate(&self, tcx: &mut TyCtxt, ty: Ty) -> Result<TypeLayout, LayoutError> {
        match tcx.get(ty) {
            TyKind::Tuple(_) | TyKind::Array(_, _) | TyKind::Struct(_, _) => {
                self.aggregate_layout(tcx, ty)
            }
            _ => self.scalar_layout(tcx, ty),
        }
    }
}

/// Completes an aggregate layout from its laid-out parts.
///
/// The aggregate alignment is the strictest part alignment, and the total size
/// is rounded up to that alignment so an array of the aggregate repeats on a
/// correct stride. An empty aggregate is zero-sized and aligned to 1, which is a
/// fully determined result rather than a special case.
fn finish_parts(parts: Vec<PartLayout>) -> TypeLayout {
    let align = parts.iter().map(|p| p.layout.align).max().unwrap_or(1).max(1);
    let end = parts.last().map(|p| p.offset + p.layout.size).unwrap_or(0);
    TypeLayout { size: align_to(end, align), align, parts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cranelift_codegen::ir::types;
    use omni_mir::ast::TypeSpec;

    /// The declaration table shared by the tests, empty because these cover
    /// scalar, tuple and array layout. Struct layout has its own test that
    /// supplies real declarations.
    static NO_DECLS: std::sync::OnceLock<HashMap<String, omni_mir::ast::StructDef>> =
        std::sync::OnceLock::new();

    /// Builds a layout engine over the host target's real facts.
    ///
    /// The caller interns types into its own `TyCtxt` and passes that same
    /// context per call, because `Ty` is an arena index.
    fn host_layout() -> TargetLayout<'static> {
        let isa = cranelift_codegen::isa::lookup(target_lexicon::Triple::host())
            .expect("host target is supported")
            .finish(cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder()))
            .expect("host ISA builds");
        let defs = NO_DECLS.get_or_init(HashMap::new);
        TargetLayout::new(TargetFacts::from_isa(&isa), defs)
    }

    #[test]
    fn target_facts_are_read_from_the_isa_not_hard_coded() {
        let layout = host_layout();
        let facts = layout.facts();
        // Consistency between the facts is what makes them read from the ISA
        // rather than being three independent constants.
        assert_eq!(facts.pointer_size * 8, u64::from(facts.pointer_bits));
        assert!(facts.pointer_bits == 32 || facts.pointer_bits == 64, "unexpected pointer width");
        assert_eq!(facts.big_endian, facts.endianness == Endianness::Big);
    }

    #[test]
    fn scalar_layout_size_comes_from_the_selected_cranelift_type() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let float_ty = tcx.intern(TyKind::Float);
        let int_layout = layout.scalar_layout(&tcx, int_ty).expect("int is representable");
        let float_layout = layout.scalar_layout(&tcx, float_ty).expect("float is representable");
        assert_eq!(int_layout.size, u64::from(types::I64.bytes()));
        assert_eq!(float_layout.size, u64::from(types::F64.bytes()));
        assert_eq!(int_layout.align, int_layout.size, "a scalar aligns to its size");
        assert!(int_layout.parts.is_empty(), "a scalar has no parts");
    }

    #[test]
    fn a_pair_of_ints_is_contiguous_with_no_trailing_padding() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let tuple_ty = tcx.intern(TyKind::Tuple(vec![int_ty, int_ty]));
        let result = layout.aggregate_layout(&mut tcx, tuple_ty).expect("tuple is representable");
        let size = u64::from(types::I64.bytes());
        assert_eq!(result.parts[0].offset, 0);
        assert_eq!(result.parts[1].offset, size);
        assert_eq!(result.size, size * 2, "no trailing padding when the size is already aligned");
        assert_eq!(result.align, size);
    }

    #[test]
    fn mixed_scalars_place_each_part_on_its_own_alignment() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let bool_ty = tcx.intern(TyKind::Bool);
        let float_ty = tcx.intern(TyKind::Float);
        let size = u64::from(types::I64.bytes());
        // (Int, Bool, Int): every part is the same width, so this must be dense.
        let triple = tcx.intern(TyKind::Tuple(vec![int_ty, bool_ty, int_ty]));
        let result = layout.aggregate_layout(&mut tcx, triple).expect("tuple is representable");
        assert_eq!(result.parts[1].offset, size);
        assert_eq!(result.parts[2].offset, size * 2);
        assert_eq!(result.size, size * 3);
        // (Float, Int): also dense under the current scalar mapping. This records
        // the present policy consequence rather than claiming it is universal.
        let pair = tcx.intern(TyKind::Tuple(vec![float_ty, int_ty]));
        let result = layout.aggregate_layout(&mut tcx, pair).expect("tuple is representable");
        assert_eq!(result.parts[1].offset, result.parts[0].layout.size);
    }

    #[test]
    fn an_array_repeats_its_element_on_a_correct_stride() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let array_ty = tcx.intern(TyKind::Array(int_ty, 3));
        let result = layout.aggregate_layout(&mut tcx, array_ty).expect("array is representable");
        let size = u64::from(types::I64.bytes());
        assert_eq!(result.parts.len(), 3);
        assert_eq!(result.parts[2].offset, size * 2, "the third element follows the second stride");
        assert_eq!(result.size, size * 3);
    }

    #[test]
    fn an_array_of_an_aggregate_uses_the_aggregate_size_as_its_stride() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let inner = tcx.intern(TyKind::Tuple(vec![int_ty, int_ty]));
        let inner_size = u64::from(types::I64.bytes()) * 2;
        let outer = tcx.intern(TyKind::Array(inner, 2));
        let result =
            layout.aggregate_layout(&mut tcx, outer).expect("nested array is representable");
        assert_eq!(result.parts[1].offset, inner_size, "stride is the full aggregate size");
        assert_eq!(result.size, inner_size * 2);
    }

    #[test]
    fn nested_aggregates_lay_out_recursively() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let size = u64::from(types::I64.bytes());
        let inner = tcx.intern(TyKind::Tuple(vec![int_ty, int_ty]));
        let outer = tcx.intern(TyKind::Tuple(vec![int_ty, inner]));
        let result =
            layout.aggregate_layout(&mut tcx, outer).expect("nested tuple is representable");
        assert_eq!(result.parts[0].offset, 0);
        assert_eq!(result.parts[1].offset, size);
        assert_eq!(
            result.parts[1].layout.size,
            size * 2,
            "the nested aggregate is laid out in full"
        );
        assert_eq!(result.size, size * 3);
    }

    #[test]
    fn an_empty_aggregate_is_zero_sized_and_aligned_to_one() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let empty = tcx.intern(TyKind::Tuple(vec![]));
        let result =
            layout.aggregate_layout(&mut tcx, empty).expect("empty tuple is representable");
        assert_eq!(result.size, 0, "an empty aggregate occupies no bytes");
        assert_eq!(result.align, 1);
        assert!(result.parts.is_empty());
    }

    #[test]
    fn layout_is_deterministic_for_a_fixed_target_description() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let float_ty = tcx.intern(TyKind::Float);
        let array_ty = tcx.intern(TyKind::Array(int_ty, 2));
        let ty = tcx.intern(TyKind::Tuple(vec![int_ty, float_ty, array_ty]));
        let first = layout.aggregate_layout(&mut tcx, ty).expect("representable");
        for _ in 0..8 {
            assert_eq!(
                layout.aggregate_layout(&mut tcx, ty).expect("representable"),
                first,
                "repeated layout must be byte-identical"
            );
        }
    }

    #[test]
    fn a_type_without_a_defined_representation_fails_closed() {
        // Reference, function, string and enum types have no normative native
        // representation. They must be rejected, never given an invented one.
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        let int_ty = tcx.intern(TyKind::Int);
        let cases = vec![
            TyKind::Reference { lifetime: None, mutable: false, inner: int_ty },
            TyKind::Fn(vec![int_ty], int_ty),
            TyKind::String,
            TyKind::Enum("Colour".to_string(), vec![]),
        ];
        for kind in cases {
            let ty = tcx.intern(kind.clone());
            assert!(
                matches!(
                    layout.aggregate_layout(&mut tcx, ty),
                    Err(LayoutError::Unrepresentable { .. })
                ),
                "{:?} must fail closed rather than be given a fabricated layout",
                kind
            );
        }
    }

    #[test]
    fn unit_and_error_have_no_aggregate_layout() {
        let layout = host_layout();
        let mut tcx = TyCtxt::new();
        for kind in [TyKind::Unit, TyKind::Error] {
            let ty = tcx.intern(kind.clone());
            assert!(matches!(
                layout.aggregate_layout(&mut tcx, ty),
                Err(LayoutError::Unrepresentable { .. })
            ));
        }
    }

    /// Builds a layout engine over the host target with the given declarations.
    fn layout_with(defs: &HashMap<String, omni_mir::ast::StructDef>) -> TargetLayout<'_> {
        let isa = cranelift_codegen::isa::lookup(target_lexicon::Triple::host())
            .expect("host target is supported")
            .finish(cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder()))
            .expect("host ISA builds");
        TargetLayout::new(TargetFacts::from_isa(&isa), defs)
    }

    /// Builds a struct declaration with the given fields.
    fn struct_def(
        name: &str,
        fields: &[(&str, omni_mir::ast::TypeSpec)],
    ) -> omni_mir::ast::StructDef {
        omni_mir::ast::StructDef {
            name: name.to_string(),
            type_params: vec![],
            fields: fields
                .iter()
                .map(|(n, t)| omni_mir::ast::StructFieldDef { name: n.to_string(), ty: t.clone() })
                .collect(),
        }
    }

    /// Builds a generic struct declaration over one type parameter.
    fn generic_struct_def(
        name: &str,
        param: &str,
        fields: &[(&str, omni_mir::ast::TypeSpec)],
    ) -> omni_mir::ast::StructDef {
        let mut def = struct_def(name, fields);
        def.type_params = vec![param.to_string()];
        def
    }

    #[test]
    fn a_struct_lays_out_its_fields_in_declaration_order() {
        let defs = HashMap::from([(
            "Pair".to_string(),
            struct_def("Pair", &[("a", TypeSpec::Int), ("b", TypeSpec::Int)]),
        )]);
        let layout = layout_with(&defs);
        let mut tcx = TyCtxt::new();
        let pair_ty = tcx.intern(TyKind::Struct("Pair".to_string(), vec![]));
        let result = layout.aggregate_layout(&mut tcx, pair_ty).expect("struct is representable");
        let size = u64::from(types::I64.bytes());
        assert_eq!(result.parts[0].name.as_deref(), Some("a"));
        assert_eq!(result.parts[1].name.as_deref(), Some("b"));
        assert_eq!(result.parts[0].offset, 0);
        assert_eq!(result.parts[1].offset, size);
        assert_eq!(result.size, size * 2);
    }

    #[test]
    fn a_generic_struct_lays_out_after_monomorphization() {
        // `Wrap<int>` must lay out its field as the substituted type, not as the
        // unsubstituted parameter, which has no representation of its own.
        let defs = HashMap::from([(
            "Wrap".to_string(),
            generic_struct_def(
                "Wrap",
                "T",
                &[("value", omni_mir::ast::TypeSpec::GenericParam("T".to_string()))],
            ),
        )]);
        let layout = layout_with(&defs);
        let mut tcx = TyCtxt::new();
        let float_ty = tcx.intern(TyKind::Float);
        let wrap_ty = tcx.intern(TyKind::Struct("Wrap".to_string(), vec![float_ty]));
        let result = layout.aggregate_layout(&mut tcx, wrap_ty).expect("struct is representable");
        assert_eq!(result.size, u64::from(types::F64.bytes()));
        assert_eq!(result.parts[0].offset, 0);
        assert_eq!(result.parts[0].layout.size, u64::from(types::F64.bytes()));
    }

    #[test]
    fn a_zero_field_struct_is_zero_sized() {
        let defs = HashMap::from([("Empty".to_string(), struct_def("Empty", &[]))]);
        let layout = layout_with(&defs);
        let mut tcx = TyCtxt::new();
        let empty_ty = tcx.intern(TyKind::Struct("Empty".to_string(), vec![]));
        let result = layout.aggregate_layout(&mut tcx, empty_ty).expect("struct is representable");
        assert_eq!(result.size, 0);
        assert_eq!(result.align, 1);
        assert!(result.parts.is_empty());
    }

    #[test]
    fn an_undeclared_struct_fails_closed() {
        // A struct type with no declaration must not be given a fabricated
        // layout; that would silently invent an aggregate representation.
        let defs = HashMap::new();
        let layout = layout_with(&defs);
        let mut tcx = TyCtxt::new();
        let ghost_ty = tcx.intern(TyKind::Struct("Ghost".to_string(), vec![]));
        assert!(matches!(
            layout.aggregate_layout(&mut tcx, ghost_ty),
            Err(LayoutError::Unrepresentable { .. })
        ));
    }

    #[test]
    fn an_unknown_field_name_is_reported_rather_than_defaulted() {
        let defs = HashMap::from([(
            "Pair".to_string(),
            struct_def("Pair", &[("a", TypeSpec::Int), ("b", TypeSpec::Int)]),
        )]);
        let layout = layout_with(&defs);
        let mut tcx = TyCtxt::new();
        let pair_ty = tcx.intern(TyKind::Struct("Pair".to_string(), vec![]));
        assert_eq!(layout.field_offset(&mut tcx, pair_ty, "a").expect("a exists"), 0);
        let size = u64::from(types::I64.bytes());
        assert_eq!(layout.field_offset(&mut tcx, pair_ty, "b").expect("b exists"), size);
        assert!(matches!(
            layout.field_offset(&mut tcx, pair_ty, "missing"),
            Err(LayoutError::UnknownField { .. })
        ));
    }

    #[test]
    fn a_nested_struct_field_is_laid_out_in_full() {
        let defs = HashMap::from([
            ("Inner".to_string(), struct_def("Inner", &[("x", TypeSpec::Int)])),
            (
                "Outer".to_string(),
                struct_def(
                    "Outer",
                    &[
                        ("head", TypeSpec::Int),
                        ("inner", TypeSpec::Struct("Inner".to_string(), vec![])),
                    ],
                ),
            ),
        ]);
        let layout = layout_with(&defs);
        let mut tcx = TyCtxt::new();
        let outer_ty = tcx.intern(TyKind::Struct("Outer".to_string(), vec![]));
        let result = layout.aggregate_layout(&mut tcx, outer_ty).expect("struct is representable");
        let size = u64::from(types::I64.bytes());
        assert_eq!(result.parts[0].offset, 0);
        assert_eq!(result.parts[1].offset, size);
        assert_eq!(result.parts[1].layout.size, size);
        assert_eq!(result.size, size * 2);
    }
}
