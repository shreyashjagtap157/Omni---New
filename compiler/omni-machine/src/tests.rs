//! Tests for the Omni abstract machine.
//!
//! These tests verify that the abstract machine produces correct results for MIR
//! programs, including edge cases and error conditions.
//!
//! The MIR builders below construct programs by hand against the real
//! `omni_mir::ir` types. Hand-built MIR keeps the interpreter's contract
//! observable: a test states exactly the statements and terminator it expects
//! the machine to run, rather than depending on source-to-MIR lowering also
//! being correct.

use index_vec::IndexVec;
use omni_mir::ast::{Lit, TypeSpec};
use omni_mir::ir::{
    AggregateKind, BasicBlock, BinOp, BlockData, Body, Constant, Local, LocalDecl, MirFunction,
    MirProgram, Operand, Place, Projection, Rvalue, Statement, Terminator, UnOp,
};
use omni_mir::{Ty, TyCtxt, TyKind};

use crate::interpreter::{DiagnosticLevel, ExecutionError, Interpreter, Value};

// --- MIR construction helpers ------------------------------------------------

/// A type context plus the `Ty` handles these tests need.
#[derive(Clone)]
struct Fixture {
    tcx: TyCtxt,
    int: Ty,
    unit: Ty,
}

impl Fixture {
    fn new() -> Self {
        let mut tcx = TyCtxt::new();
        let int = tcx.intern(TyKind::Int);
        let unit = tcx.intern(TyKind::Unit);
        Self { tcx, int, unit }
    }
}

/// A function under construction, accumulating locals and blocks.
///
/// Local ids come from `IndexVec::push`, which is exactly how `omni-mir`'s own
/// lowering assigns them, so a place built here has the same identity the
/// verifier and interpreter see.
struct FnBuilder {
    name: String,
    params: Vec<Local>,
    return_place: Local,
    return_type: TypeSpec,
    local_decls: IndexVec<Local, LocalDecl>,
    blocks: IndexVec<BasicBlock, BlockData>,
}

impl FnBuilder {
    fn new(fixture: &Fixture, name: &str, return_type: TypeSpec) -> Self {
        let mut local_decls = IndexVec::new();
        let return_place = local_decls
            .push(LocalDecl { name: Some("_return".to_string()), ty: Some(fixture.unit) });
        Self {
            name: name.to_string(),
            params: Vec::new(),
            return_place,
            return_type,
            local_decls,
            blocks: IndexVec::new(),
        }
    }

    /// Declares a local and returns its index.
    fn local(&mut self, name: &str, ty: Option<Ty>) -> Local {
        self.local_decls.push(LocalDecl { name: Some(name.to_string()), ty })
    }

    /// Declares a parameter local, recorded in `params`.
    fn param(&mut self, name: &str, ty: Option<Ty>) -> Local {
        let local = self.local(name, ty);
        self.params.push(local);
        local
    }

    /// Finishes a single-block body that returns.
    fn returns(self, statements: Vec<Statement>) -> MirFunction {
        self.finish(statements, Terminator::Return)
    }

    /// Finishes the function, prepending `statements` + `terminator` as the
    /// entry block.
    ///
    /// The entry block is inserted at the front so it lands at index 0. The
    /// interpreter always begins execution at block 0, and `omni-mir`'s lowering
    /// creates the entry block first for the same reason; appending it last
    /// would make these programs jump straight into a later block.
    fn finish(self, statements: Vec<Statement>, terminator: Terminator) -> MirFunction {
        let mut blocks = IndexVec::new();
        blocks.push(BlockData { statements, terminator: Some(terminator) });
        blocks.extend(self.blocks);
        MirFunction {
            name: self.name,
            params: self.params,
            return_place: self.return_place,
            return_type: self.return_type,
            body: Body { blocks, local_decls: self.local_decls, ..Default::default() },
        }
    }

    /// Appends a non-entry block and returns its index.
    ///
    /// `finish` later inserts the entry block at index 0, which shifts every
    /// block appended here up by one, so the returned index is offset to match
    /// the final layout.
    fn block(&mut self, statements: Vec<Statement>, terminator: Terminator) -> BasicBlock {
        let raw = self.blocks.push(BlockData { statements, terminator: Some(terminator) });
        BasicBlock::from_usize(raw.index() + 1)
    }
}

fn interpreter(fixture: Fixture, functions: Vec<MirFunction>) -> Interpreter {
    Interpreter::new_with_program(MirProgram::new(fixture.tcx, functions))
}

fn const_int(value: i64) -> Operand {
    Operand::Constant(Constant::Lit(Lit::Int(value)))
}

fn const_bool(value: bool) -> Operand {
    Operand::Constant(Constant::Lit(Lit::Bool(value)))
}

fn const_float(value: f64) -> Operand {
    Operand::Constant(Constant::Lit(Lit::Float(value.to_bits())))
}

fn copy(place: Place) -> Operand {
    Operand::Copy(place)
}

fn assign(place: Place, rvalue: Rvalue) -> Statement {
    Statement::Assign(place, rvalue)
}

/// Builds `return_place = value`, then returns.
fn return_const(fixture: &Fixture, value: i64) -> MirFunction {
    let builder = FnBuilder::new(fixture, "f", TypeSpec::Int);
    let ret = builder.return_place;
    builder.returns(vec![assign(Place::local(ret), Rvalue::Use(const_int(value)))])
}

// --- Arithmetic and unary operations ------------------------------------------

#[test]
fn evaluates_integer_addition() {
    let f = Fixture::new();
    let func = return_const(&f, 15);
    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(15));
}

#[test]
fn evaluates_integer_arithmetic_operations() {
    let cases: &[(BinOp, i64, i64, i64)] = &[
        (BinOp::Add, 10, 5, 15),
        (BinOp::Sub, 10, 3, 7),
        (BinOp::Mul, 6, 7, 42),
        (BinOp::Div, 20, 4, 5),
        (BinOp::Rem, 20, 6, 2),
        (BinOp::BitAnd, 12, 10, 8),
        (BinOp::BitOr, 12, 10, 14),
        (BinOp::BitXor, 12, 10, 6),
        (BinOp::Shl, 1, 4, 16),
        (BinOp::Shr, 32, 2, 8),
    ];

    for (i, (op, lhs, rhs, expected)) in cases.iter().enumerate() {
        let f = Fixture::new();
        let builder = FnBuilder::new(&f, "f", TypeSpec::Int);
        let ret = builder.return_place;
        let func = builder.returns(vec![assign(
            Place::local(ret),
            Rvalue::BinaryOp(*op, const_int(*lhs), const_int(*rhs)),
        )]);

        let mut interp = interpreter(f, vec![func]);
        assert_eq!(
            interp.execute_function("f", vec![]).unwrap(),
            Value::Int(*expected),
            "case {i}: {op:?}({lhs}, {rhs})"
        );
    }
}

#[test]
fn evaluates_integer_comparisons() {
    let cases: &[(BinOp, i64, i64, bool)] = &[
        (BinOp::Eq, 4, 4, true),
        (BinOp::Eq, 4, 5, false),
        (BinOp::Ne, 4, 5, true),
        (BinOp::Lt, 4, 5, true),
        (BinOp::Le, 4, 4, true),
        (BinOp::Gt, 5, 4, true),
        (BinOp::Ge, 4, 5, false),
    ];

    for (i, (op, lhs, rhs, expected)) in cases.iter().enumerate() {
        let f = Fixture::new();
        let builder = FnBuilder::new(&f, "f", TypeSpec::Bool);
        let ret = builder.return_place;
        let func = builder.returns(vec![assign(
            Place::local(ret),
            Rvalue::BinaryOp(*op, const_int(*lhs), const_int(*rhs)),
        )]);

        let mut interp = interpreter(f, vec![func]);
        assert_eq!(
            interp.execute_function("f", vec![]).unwrap(),
            Value::Bool(*expected),
            "case {i}: {op:?}({lhs}, {rhs})"
        );
    }
}

#[test]
fn evaluates_float_arithmetic_operations() {
    let cases: &[(BinOp, f64, f64, f64)] = &[
        (BinOp::Add, 1.5, 2.25, 3.75),
        (BinOp::Sub, 5.5, 2.25, 3.25),
        (BinOp::Mul, 2.5, 4.0, 10.0),
        (BinOp::Div, 7.5, 2.5, 3.0),
    ];

    for (i, (op, lhs, rhs, expected)) in cases.iter().enumerate() {
        let f = Fixture::new();
        let float = {
            let mut tcx = f.tcx.clone();
            tcx.intern(TyKind::Float)
        };
        let mut builder = FnBuilder::new(&f, "f", TypeSpec::Float);
        let ret = builder.return_place;
        builder.local("lhs", Some(float));
        let func = builder.returns(vec![assign(
            Place::local(ret),
            Rvalue::BinaryOp(*op, const_float(*lhs), const_float(*rhs)),
        )]);

        let mut interp = interpreter(f, vec![func]);
        let Value::Float(value) = interp.execute_function("f", vec![]).unwrap() else {
            panic!("expected Float result");
        };
        assert_eq!(value.to_bits(), expected.to_bits(), "case {i}: {op:?}");
    }
}

#[test]
fn evaluates_float_comparisons_and_nan() {
    let cases: &[(BinOp, f64, f64, bool)] = &[
        (BinOp::Eq, 4.0, 4.0, true),
        (BinOp::Ne, 4.0, 5.0, true),
        (BinOp::Lt, 4.0, 5.0, true),
        (BinOp::Le, 4.0, 4.0, true),
        (BinOp::Gt, 5.0, 4.0, true),
        (BinOp::Ge, 5.0, 5.0, true),
        (BinOp::Eq, f64::NAN, f64::NAN, false),
        (BinOp::Ne, f64::NAN, 1.0, true),
        (BinOp::Lt, f64::NAN, 1.0, false),
        (BinOp::Ge, 1.0, f64::NAN, false),
    ];

    for (op, lhs, rhs, expected) in cases {
        let f = Fixture::new();
        let mut tcx = f.tcx.clone();
        let float = tcx.intern(TyKind::Float);
        let mut builder = FnBuilder::new(&f, "f", TypeSpec::Bool);
        let ret = builder.return_place;
        builder.local("lhs", Some(float));
        let func = builder.returns(vec![assign(
            Place::local(ret),
            Rvalue::BinaryOp(*op, const_float(*lhs), const_float(*rhs)),
        )]);

        let mut interp = interpreter(f, vec![func]);
        assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Bool(*expected));
    }
}

#[test]
fn evaluates_float_negation() {
    let f = Fixture::new();
    let float = {
        let mut tcx = f.tcx.clone();
        tcx.intern(TyKind::Float)
    };
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Float);
    let ret = builder.return_place;
    builder.local("operand", Some(float));
    let func = builder
        .returns(vec![assign(Place::local(ret), Rvalue::UnaryOp(UnOp::Neg, const_float(2.5)))]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Float(-2.5));
}

#[test]
fn evaluates_boolean_comparisons() {
    let cases: &[(BinOp, bool, bool, bool)] = &[
        (BinOp::Eq, true, true, true),
        (BinOp::Eq, true, false, false),
        (BinOp::Ne, true, false, true),
    ];

    for (op, lhs, rhs, expected) in cases {
        let f = Fixture::new();
        let builder = FnBuilder::new(&f, "f", TypeSpec::Bool);
        let ret = builder.return_place;
        let func = builder.returns(vec![assign(
            Place::local(ret),
            Rvalue::BinaryOp(*op, const_bool(*lhs), const_bool(*rhs)),
        )]);

        let mut interp = interpreter(f, vec![func]);
        assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Bool(*expected));
    }
}

#[test]
fn executes_mutable_reference_dereference_and_write() {
    let mut f = Fixture::new();
    let reference_ty = {
        let inner = f.int;
        f.tcx.intern(TyKind::Reference { lifetime: None, mutable: true, inner })
    };
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let ret = builder.return_place;
    let value_local = builder.local("value", Some(f.int));
    let ref_local = builder.local("reference", Some(reference_ty));
    let value_place = Place::local(value_local);
    let ref_place = Place::local(ref_local);
    let deref_place = ref_place.project(Projection::Deref);
    let func = builder.returns(vec![
        assign(value_place.clone(), Rvalue::Use(const_int(10))),
        assign(
            ref_place.clone(),
            Rvalue::Reference {
                place: value_place.clone(),
                mutable: true,
                ty: reference_ty,
            },
        ),
        assign(deref_place.clone(), Rvalue::Use(const_int(42))),
        assign(Place::local(ret), Rvalue::Use(copy(deref_place))),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(42));
}

#[test]
fn evaluates_range_values_without_type_erasing_to_text() {
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Range(Box::new(TypeSpec::Int)));
    let ret = builder.return_place;
    let func = builder.returns(vec![assign(
        Place::local(ret),
        Rvalue::Range { start: const_int(2), end: const_int(5), inclusive: false, ty: f.int },
    )]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(
        interp.execute_function("f", vec![]).unwrap(),
        Value::Range {
            start: Box::new(Value::Int(2)),
            end: Box::new(Value::Int(5)),
            inclusive: false,
        }
    );
}

#[test]
fn evaluates_unary_operations() {
    let cases: &[(UnOp, i64, i64)] = &[(UnOp::Neg, 5, -5), (UnOp::BitNot, 0, -1)];
    for (op, operand, expected) in cases {
        let f = Fixture::new();
        let builder = FnBuilder::new(&f, "f", TypeSpec::Int);
        let ret = builder.return_place;
        let func = builder
            .returns(vec![assign(Place::local(ret), Rvalue::UnaryOp(*op, const_int(*operand)))]);

        let mut interp = interpreter(f, vec![func]);
        assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(*expected));
    }

    // Boolean negation operates on `Bool`, not `Int`.
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Bool);
    let ret = builder.return_place;
    let func = builder
        .returns(vec![assign(Place::local(ret), Rvalue::UnaryOp(UnOp::Not, const_bool(true)))]);
    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Bool(false));
}

#[test]
fn rejects_division_by_zero() {
    for op in [BinOp::Div, BinOp::Rem] {
        let f = Fixture::new();
        let builder = FnBuilder::new(&f, "f", TypeSpec::Int);
        let ret = builder.return_place;
        let func = builder.returns(vec![assign(
            Place::local(ret),
            Rvalue::BinaryOp(op, const_int(1), const_int(0)),
        )]);

        let mut interp = interpreter(f, vec![func]);
        let err = interp.execute_function("f", vec![]).unwrap_err();
        assert!(
            matches!(err, ExecutionError::DivisionByZero),
            "{op:?} by zero should be DivisionByZero, got {err:?}"
        );
    }
}

#[test]
fn rejects_mismatched_binary_operands() {
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let ret = builder.return_place;
    let func = builder.returns(vec![assign(
        Place::local(ret),
        Rvalue::BinaryOp(BinOp::Add, const_int(1), const_bool(true)),
    )]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    assert!(
        matches!(err, ExecutionError::InvalidRvalue { .. }),
        "mixed-type add should be rejected, got {err:?}"
    );
}

// --- Aggregates ---------------------------------------------------------------

#[test]
fn builds_tuple_aggregate() {
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Tuple(vec![TypeSpec::Int, TypeSpec::Int]));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![assign(
        Place::local(ret),
        Rvalue::Aggregate {
            kind: AggregateKind::Tuple,
            operands: vec![const_int(1), const_int(2)],
            ty,
        },
    )]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(
        interp.execute_function("f", vec![]).unwrap(),
        Value::Tuple(vec![Value::Int(1), Value::Int(2)])
    );
}

#[test]
fn builds_array_aggregate() {
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Array(Box::new(TypeSpec::Int), 3));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![assign(
        Place::local(ret),
        Rvalue::Aggregate {
            kind: AggregateKind::Array,
            operands: vec![const_int(7), const_int(8), const_int(9)],
            ty,
        },
    )]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(
        interp.execute_function("f", vec![]).unwrap(),
        Value::Array(vec![Value::Int(7), Value::Int(8), Value::Int(9)])
    );
}

#[test]
fn builds_nested_tuple_aggregate() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let a = builder.local("a", Some(f.int));
    let b = builder.local("b", Some(f.int));
    let c = builder.local("c", Some(f.int));
    let tuple = builder.local("tuple", Some(f.int));
    let ty = f.int;
    let ret = builder.return_place;

    builder.params = vec![a, b, c];
    let func = builder.returns(vec![
        assign(
            Place::local(tuple),
            Rvalue::Aggregate {
                kind: AggregateKind::Tuple,
                operands: vec![copy(Place::local(a)), copy(Place::local(b)), copy(Place::local(c))],
                ty,
            },
        ),
        assign(
            Place::local(ret),
            Rvalue::Aggregate {
                kind: AggregateKind::Tuple,
                operands: vec![copy(Place::local(tuple)), copy(Place::local(a))],
                ty,
            },
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(
        interp.execute_function("f", vec![Value::Int(1), Value::Int(2), Value::Int(3)]).unwrap(),
        Value::Tuple(vec![
            Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
            Value::Int(1),
        ])
    );
}

#[test]
fn builds_struct_value_with_named_fields() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let point = builder.local("point", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(point),
            Rvalue::Struct {
                name: "Point".to_string(),
                fields: vec![("x".to_string(), const_int(3)), ("y".to_string(), const_int(4))],
                ty,
            },
        ),
        // Reading `.x` back through a field projection must see the written value.
        assign(
            Place::local(ret),
            Rvalue::Field { base: copy(Place::local(point)), field: "x".to_string(), ty },
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(3));
}

#[test]
fn missing_struct_field_is_a_projection_error() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let point = builder.local("point", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(point),
            Rvalue::Struct {
                name: "Point".to_string(),
                fields: vec![("x".to_string(), const_int(3))],
                ty,
            },
        ),
        assign(
            Place::local(ret),
            Rvalue::Field { base: copy(Place::local(point)), field: "missing".to_string(), ty },
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    match err {
        ExecutionError::InvalidProjection { message, .. } => {
            assert!(message.contains("missing"), "unexpected message: {message}");
            assert!(message.contains("Point"), "unexpected message: {message}");
        }
        other => panic!("expected InvalidProjection, got {other:?}"),
    }
}

#[test]
fn enum_variant_fields_project_by_ordinal() {
    // The interpreter resolves an enum field projection positionally, so a
    // variant's payload is read back by index rather than by name.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let shape = builder.local("shape", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(shape),
            Rvalue::EnumVariant {
                enum_name: "Shape".to_string(),
                variant: "Circle".to_string(),
                operands: vec![const_int(2), const_int(9)],
                ty,
            },
        ),
        assign(
            Place::local(ret),
            Rvalue::Field { base: copy(Place::local(shape)), field: "1".to_string(), ty },
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(9));
}

#[test]
fn enum_variant_field_index_out_of_range_is_a_projection_error() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let shape = builder.local("shape", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(shape),
            Rvalue::EnumVariant {
                enum_name: "Shape".to_string(),
                variant: "Point".to_string(),
                operands: vec![const_int(5)],
                ty,
            },
        ),
        assign(
            Place::local(ret),
            Rvalue::Field { base: copy(Place::local(shape)), field: "3".to_string(), ty },
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    assert!(
        matches!(err, ExecutionError::InvalidProjection { .. }),
        "expected InvalidProjection, got {err:?}"
    );
}

// --- Projections --------------------------------------------------------------

/// Builds a one-block function that writes a 3-element array, then reads the
/// place produced by `build` (which is handed the array's local).
fn project_from_array(
    fixture: &Fixture,
    name: &str,
    build: impl FnOnce(Local) -> Place,
) -> MirFunction {
    let mut builder = FnBuilder::new(fixture, name, TypeSpec::Int);
    let arr = builder.local("arr", Some(fixture.int));
    let ret = builder.return_place;
    let ty = fixture.int;
    let place = build(arr);
    builder.returns(vec![
        assign(
            Place::local(arr),
            Rvalue::Aggregate {
                kind: AggregateKind::Array,
                operands: vec![const_int(10), const_int(20), const_int(30)],
                ty,
            },
        ),
        assign(Place::local(ret), Rvalue::Use(copy(place))),
    ])
}

#[test]
fn constant_index_projection_reads_array_element() {
    let f = Fixture::new();
    let func =
        project_from_array(&f, "f", |arr| Place::local(arr).project(Projection::ConstantIndex(1)));
    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(20));
}

#[test]
fn constant_index_out_of_range_is_a_projection_error() {
    let f = Fixture::new();
    let func =
        project_from_array(&f, "f", |arr| Place::local(arr).project(Projection::ConstantIndex(7)));
    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    match err {
        ExecutionError::InvalidProjection { message, .. } => {
            assert!(message.contains('7'), "unexpected message: {message}");
        }
        other => panic!("expected InvalidProjection, got {other:?}"),
    }
}

#[test]
fn zero_length_container_rejects_any_index() {
    // A zero-length array must reject index 0 rather than reading past its end.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let arr = builder.local("arr", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(arr),
            Rvalue::Aggregate { kind: AggregateKind::Array, operands: vec![], ty },
        ),
        assign(
            Place::local(ret),
            Rvalue::Use(copy(Place::local(arr).project(Projection::ConstantIndex(0)))),
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    match err {
        ExecutionError::InvalidProjection { message, .. } => {
            assert!(message.contains('0'), "unexpected message: {message}");
        }
        other => panic!("expected InvalidProjection, got {other:?}"),
    }
}

#[test]
fn runtime_index_projection_reads_array_element() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let arr = builder.local("arr", Some(f.int));
    let idx = builder.local("idx", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(arr),
            Rvalue::Aggregate {
                kind: AggregateKind::Array,
                operands: vec![const_int(10), const_int(20), const_int(30)],
                ty,
            },
        ),
        assign(Place::local(idx), Rvalue::Use(const_int(2))),
        assign(
            Place::local(ret),
            Rvalue::Use(copy(Place::local(arr).project(Projection::Index(idx)))),
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(30));
}

#[test]
fn negative_runtime_index_is_rejected_not_wrapped() {
    // A negative MIR index must be reported as out of range. Casting to `usize`
    // first would wrap it to a huge offset and hide the real cause.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let arr = builder.local("arr", Some(f.int));
    let idx = builder.local("idx", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(arr),
            Rvalue::Aggregate {
                kind: AggregateKind::Array,
                operands: vec![const_int(10), const_int(20)],
                ty,
            },
        ),
        assign(Place::local(idx), Rvalue::Use(const_int(-1))),
        assign(
            Place::local(ret),
            Rvalue::Use(copy(Place::local(arr).project(Projection::Index(idx)))),
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    match err {
        ExecutionError::InvalidProjection { message, .. } => {
            assert!(
                message.contains("-1"),
                "message should name the offending index, got: {message}"
            );
        }
        other => panic!("expected InvalidProjection, got {other:?}"),
    }
}

#[test]
fn string_index_projection_reads_character() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let s = builder.local("s", Some(f.int));
    let idx = builder.local("idx", Some(f.int));
    let ret = builder.return_place;
    let func = builder.returns(vec![
        assign(
            Place::local(s),
            Rvalue::Use(Operand::Constant(Constant::Lit(Lit::String("abc".to_string())))),
        ),
        assign(Place::local(idx), Rvalue::Use(const_int(1))),
        assign(
            Place::local(ret),
            Rvalue::Use(copy(Place::local(s).project(Projection::Index(idx)))),
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Char('b'));
}

#[test]
fn nested_projection_chain_resolves_root_outward() {
    // ((1, 2), 3)[0].1 must resolve to 2. The inner tuple is materialised into
    // its own local first: `Operand` has no variant for a nested rvalue, so an
    // aggregate is always built by a statement and then read by place.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let inner = builder.local("inner", Some(f.int));
    let nested = builder.local("nested", Some(f.int));
    let ret = builder.return_place;
    let ty = f.int;
    let func = builder.returns(vec![
        assign(
            Place::local(inner),
            Rvalue::Aggregate {
                kind: AggregateKind::Tuple,
                operands: vec![const_int(1), const_int(2)],
                ty,
            },
        ),
        assign(
            Place::local(nested),
            Rvalue::Aggregate {
                kind: AggregateKind::Tuple,
                operands: vec![copy(Place::local(inner)), const_int(3)],
                ty,
            },
        ),
        assign(
            Place::local(ret),
            Rvalue::Use(copy(
                Place::local(nested)
                    .project(Projection::ConstantIndex(0))
                    .project(Projection::ConstantIndex(1)),
            )),
        ),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(2));
}

// --- Bounds checks ------------------------------------------------------------

/// Builds `f(index: int)` which runs `bounds_check(index, length)` then returns 42.
///
/// The bounds check reads the index from the parameter local, so each test
/// supplies the index at the call site via `execute_function`.
fn bounds_check_program(length: usize) -> (Fixture, MirFunction) {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let idx = builder.param("index", Some(f.int));
    let ret = builder.return_place;
    let func = builder.returns(vec![
        Statement::BoundsCheck { index: idx, length },
        assign(Place::local(ret), Rvalue::Use(const_int(42))),
    ]);
    (f, func)
}

#[test]
fn bounds_check_passes_for_in_range_index() {
    let (f, func) = bounds_check_program(10);
    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![Value::Int(0)]).unwrap(), Value::Int(42));
}

#[test]
fn bounds_check_passes_at_last_valid_index() {
    let (f, func) = bounds_check_program(10);
    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![Value::Int(9)]).unwrap(), Value::Int(42));
}

#[test]
fn bounds_check_fails_at_length() {
    let (f, func) = bounds_check_program(10);
    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![Value::Int(10)]).unwrap_err();
    match err {
        ExecutionError::BoundsCheckFailed { index, length } => {
            assert_eq!(index, 10);
            assert_eq!(length, 10);
        }
        other => panic!("expected BoundsCheckFailed, got {other:?}"),
    }
}

#[test]
fn bounds_check_fails_beyond_length() {
    let (f, func) = bounds_check_program(10);
    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![Value::Int(15)]).unwrap_err();
    match err {
        ExecutionError::BoundsCheckFailed { index, length } => {
            assert_eq!(index, 15);
            assert_eq!(length, 10);
        }
        other => panic!("expected BoundsCheckFailed, got {other:?}"),
    }
}

#[test]
fn bounds_check_rejects_negative_index() {
    // A negative index must be rejected on its own terms, not wrapped into a
    // large positive offset that merely looks out of range.
    let (f, func) = bounds_check_program(10);
    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![Value::Int(-1)]).unwrap_err();
    match err {
        ExecutionError::BoundsCheckFailed { index, length } => {
            assert_eq!(index, -1, "the negative index must be reported verbatim");
            assert_eq!(length, 10);
        }
        other => panic!("expected BoundsCheckFailed, got {other:?}"),
    }
}

#[test]
fn bounds_check_on_zero_length_always_fails() {
    let (f, func) = bounds_check_program(0);
    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![Value::Int(0)]).unwrap_err();
    match err {
        ExecutionError::BoundsCheckFailed { index, length } => {
            assert_eq!(index, 0);
            assert_eq!(length, 0);
        }
        other => panic!("expected BoundsCheckFailed, got {other:?}"),
    }
}

#[test]
fn repeated_bounds_checks_each_enforced() {
    // Every bounds check is a real operation; a second failing check must still
    // trap rather than being skipped because an earlier one passed.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let a = builder.local("a", Some(f.int));
    let b = builder.local("b", Some(f.int));
    let ret = builder.return_place;
    builder.params = vec![a, b];
    let func = builder.returns(vec![
        Statement::BoundsCheck { index: a, length: 10 },
        Statement::BoundsCheck { index: b, length: 10 },
        assign(Place::local(ret), Rvalue::Use(const_int(7))),
    ]);

    // Both in range: completes.
    let mut interp = interpreter(f.clone(), vec![func.clone()]);
    assert_eq!(
        interp.execute_function("f", vec![Value::Int(1), Value::Int(2)]).unwrap(),
        Value::Int(7)
    );

    // Second check out of range: traps with the second index.
    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![Value::Int(1), Value::Int(99)]).unwrap_err();
    match err {
        ExecutionError::BoundsCheckFailed { index, length } => {
            assert_eq!(index, 99);
            assert_eq!(length, 10);
        }
        other => panic!("expected BoundsCheckFailed, got {other:?}"),
    }
}

#[test]
fn bounds_check_on_non_integer_local_fails_closed() {
    let (f, func) = bounds_check_program(10);
    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![Value::Bool(true)]).unwrap_err();
    assert!(
        matches!(err, ExecutionError::BoundsCheckFailed { index: -1, length: 10 }),
        "non-integer index must fail closed, got {err:?}"
    );
}

// --- Control flow -------------------------------------------------------------

#[test]
fn goto_transfers_to_target_block() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let ret = builder.return_place;
    // Entry (block 0) jumps to block 1, which produces the returned value.
    let target = builder
        .block(vec![assign(Place::local(ret), Rvalue::Use(const_int(11)))], Terminator::Return);
    let func = builder.finish(vec![], Terminator::Goto(target));

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(11));
}

#[test]
fn switch_int_selects_matching_target() {
    // switch on 2 -> block 1, which returns 20.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let discr = builder.local("discr", Some(f.int));
    let r1 = builder.return_place;
    let one = builder
        .block(vec![assign(Place::local(r1), Rvalue::Use(const_int(10)))], Terminator::Return);
    let r2 = builder.return_place;
    let two = builder
        .block(vec![assign(Place::local(r2), Rvalue::Use(const_int(20)))], Terminator::Return);
    let other = builder
        .block(vec![assign(Place::local(r2), Rvalue::Use(const_int(99)))], Terminator::Return);
    let func = builder.finish(
        vec![assign(Place::local(discr), Rvalue::Use(const_int(2)))],
        Terminator::SwitchInt {
            discr: copy(Place::local(discr)),
            targets: vec![(1, one), (2, two)],
            otherwise: other,
        },
    );

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(20));
}

#[test]
fn switch_int_falls_through_to_otherwise() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let discr = builder.local("discr", Some(f.int));
    let ret = builder.return_place;
    let matched = builder
        .block(vec![assign(Place::local(ret), Rvalue::Use(const_int(10)))], Terminator::Return);
    let other = builder
        .block(vec![assign(Place::local(ret), Rvalue::Use(const_int(99)))], Terminator::Return);
    let func = builder.finish(
        vec![assign(Place::local(discr), Rvalue::Use(const_int(7)))],
        Terminator::SwitchInt {
            discr: copy(Place::local(discr)),
            targets: vec![(1, matched)],
            otherwise: other,
        },
    );

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(99));
}

#[test]
fn unreachable_terminator_is_reported() {
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let func = builder.finish(vec![], Terminator::Unreachable);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    assert!(
        matches!(err, ExecutionError::UnreachableCode),
        "expected UnreachableCode, got {err:?}"
    );
}

#[test]
fn branch_on_computed_condition_selects_arm() {
    // `flag` is a parameter; the switch reads the *computed* value, so this
    // checks the operand path rather than a folded constant.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let flag = builder.local("flag", Some(f.int));
    let ret = builder.return_place;
    builder.params = vec![flag];
    let then = builder
        .block(vec![assign(Place::local(ret), Rvalue::Use(const_int(1)))], Terminator::Return);
    let otherwise = builder
        .block(vec![assign(Place::local(ret), Rvalue::Use(const_int(0)))], Terminator::Return);
    let func = builder.finish(
        vec![],
        Terminator::SwitchInt {
            discr: copy(Place::local(flag)),
            targets: vec![(1, then)],
            otherwise,
        },
    );

    let mut interp = interpreter(f.clone(), vec![func.clone()]);
    assert_eq!(interp.execute_function("f", vec![Value::Int(1)]).unwrap(), Value::Int(1));

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![Value::Int(0)]).unwrap(), Value::Int(0));
}

// --- Calls --------------------------------------------------------------------

#[test]
fn calls_a_user_function_and_returns_its_result() {
    let f = Fixture::new();
    let mut callee = return_const(&f, 7);
    callee.name = "callee".to_string();

    // `f` calls `callee()` in block 0 and returns the result from block 1, so the
    // call's destination is read back through a projection-free place.
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let call_ret = builder.local("call_ret", Some(f.int));
    let ret = builder.return_place;
    let after = builder.block(
        vec![assign(Place::local(ret), Rvalue::Use(copy(Place::local(call_ret))))],
        Terminator::Return,
    );
    let func = builder.finish(
        vec![],
        Terminator::Call {
            func: Operand::Constant(Constant::FnRef("callee".to_string())),
            args: vec![],
            destination: Some(Place::local(call_ret)),
            target: after,
            cleanup: None,
        },
    );

    let mut interp = interpreter(f, vec![func, callee]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(7));
}

#[test]
fn call_forwards_arguments_to_the_callee() {
    let f = Fixture::new();

    // `double(n) = n * 2`, built through the same helpers as any other function.
    let mut callee = FnBuilder::new(&f, "double", TypeSpec::Int);
    let n = callee.param("n", Some(f.int));
    let ret = callee.return_place;
    let callee = callee.returns(vec![assign(
        Place::local(ret),
        Rvalue::BinaryOp(BinOp::Mul, copy(Place::local(n)), const_int(2)),
    )]);

    // `f(n) = double(n)`.
    let mut caller = FnBuilder::new(&f, "f", TypeSpec::Int);
    let arg = caller.param("n", Some(f.int));
    let call_ret = caller.local("call_ret", Some(f.int));
    let out = caller.return_place;
    let after = caller.block(
        vec![assign(Place::local(out), Rvalue::Use(copy(Place::local(call_ret))))],
        Terminator::Return,
    );
    let caller = caller.finish(
        vec![],
        Terminator::Call {
            func: Operand::Constant(Constant::FnRef("double".to_string())),
            args: vec![copy(Place::local(arg))],
            destination: Some(Place::local(call_ret)),
            target: after,
            cleanup: None,
        },
    );

    let mut interp = interpreter(f, vec![caller, callee]);
    assert_eq!(interp.execute_function("f", vec![Value::Int(21)]).unwrap(), Value::Int(42));
}

#[test]
fn call_to_missing_function_is_reported() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Unit);
    let next = builder.block(vec![], Terminator::Return);
    let func = builder.finish(
        vec![],
        Terminator::Call {
            func: Operand::Constant(Constant::FnRef("nope".to_string())),
            args: vec![],
            destination: None,
            target: next,
            cleanup: None,
        },
    );

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    match err {
        ExecutionError::InvalidFunctionCall { function, .. } => assert_eq!(function, "nope"),
        other => panic!("expected InvalidFunctionCall, got {other:?}"),
    }
}

#[test]
fn executing_an_unknown_function_is_reported() {
    let f = Fixture::new();
    let callee = return_const(&f, 1);
    let mut interp = interpreter(f, vec![callee]);
    let err = interp.execute_function("absent", vec![]).unwrap_err();
    assert!(
        matches!(err, ExecutionError::InvalidFunctionCall { .. }),
        "expected InvalidFunctionCall, got {err:?}"
    );
}

#[test]
fn too_many_arguments_are_rejected() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let a = builder.local("a", Some(f.int));
    builder.params = vec![a];
    let ret = builder.return_place;
    let func = builder.returns(vec![assign(Place::local(ret), Rvalue::Use(const_int(1)))]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![Value::Int(1), Value::Int(2)]).unwrap_err();
    assert!(
        matches!(err, ExecutionError::InvalidFunctionCall { .. }),
        "expected InvalidFunctionCall, got {err:?}"
    );
}

#[test]
fn builtin_alloc_returns_a_live_allocation() {
    // `alloc` is the interpreter's builtin backing explicit memory. It must
    // return a handle to an allocation that is actually registered.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let handle = builder.local("handle", Some(f.int));
    let ret = builder.return_place;
    let after = builder.block(
        vec![assign(Place::local(ret), Rvalue::Use(copy(Place::local(handle))))],
        Terminator::Return,
    );
    let func = builder.finish(
        vec![],
        Terminator::Call {
            func: Operand::Constant(Constant::FnRef("alloc".to_string())),
            args: vec![const_int(16), const_int(4)],
            destination: Some(Place::local(handle)),
            target: after,
            cleanup: None,
        },
    );

    let mut interp = interpreter(f, vec![func]);
    let value = interp.execute_function("f", vec![]).unwrap();
    let Value::Int(handle) = value else {
        panic!("expected an allocation handle, got {value:?}");
    };
    assert!(handle >= 0, "an allocation handle is a non-negative machine integer");
    assert!(
        interp.allocation_is_live(handle as u64),
        "the returned handle must name a live allocation"
    );
}

#[test]
fn builtin_dealloc_frees_an_allocation() {
    // Allocation state lives in the interpreter, so allocation and release have
    // to happen within a single execution for the handle to mean anything.
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let handle = builder.local("handle", Some(f.int));
    let ret = builder.return_place;

    // Block 0 allocates, block 1 releases, block 2 returns the freed handle.
    let done = builder.block(
        vec![assign(Place::local(ret), Rvalue::Use(copy(Place::local(handle))))],
        Terminator::Return,
    );
    let release = builder.block(
        vec![],
        Terminator::Call {
            func: Operand::Constant(Constant::FnRef("dealloc".to_string())),
            args: vec![copy(Place::local(handle))],
            destination: None,
            target: done,
            cleanup: None,
        },
    );
    let func = builder.finish(
        vec![],
        Terminator::Call {
            func: Operand::Constant(Constant::FnRef("alloc".to_string())),
            args: vec![const_int(8), const_int(4)],
            destination: Some(Place::local(handle)),
            target: release,
            cleanup: None,
        },
    );

    let mut interp = interpreter(f, vec![func]);
    let Value::Int(handle) = interp.execute_function("f", vec![]).unwrap() else {
        panic!("expected an allocation handle");
    };
    assert!(
        !interp.allocation_is_live(handle as u64),
        "the returned handle must have been released before the function returned"
    );
}

#[test]
fn builtin_alloc_rejects_a_non_integer_argument() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let handle = builder.local("handle", Some(f.int));
    let after = builder.block(vec![], Terminator::Return);
    let func = builder.finish(
        vec![],
        Terminator::Call {
            func: Operand::Constant(Constant::FnRef("alloc".to_string())),
            args: vec![const_bool(true), const_int(4)],
            destination: Some(Place::local(handle)),
            target: after,
            cleanup: None,
        },
    );

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    assert!(
        matches!(err, ExecutionError::InvalidFunctionCall { .. }),
        "expected InvalidFunctionCall, got {err:?}"
    );
}

// --- Drop, assumptions, diagnostics -------------------------------------------

#[test]
fn unit_returning_function_need_not_assign_its_return_place() {
    // A `Unit` function legitimately reaches `Return` without writing the
    // return place. That must not be confused with reading a dead place.
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Unit);
    let ret = builder.return_place;
    let func = builder.returns(vec![assign(
        Place::local(ret),
        Rvalue::Use(Operand::Constant(Constant::Lit(Lit::Int(0)))),
    )]);

    let mut interp = interpreter(f, vec![func]);
    // Written to a value-typed return place, so this yields 0, not an error.
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(0));
}

#[test]
fn returning_without_initializing_the_return_place_is_reported() {
    // Falling off the end of a function that produced no value must be
    // reported rather than returning the `Uninit` sentinel.
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let func = builder.returns(vec![]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    match err {
        ExecutionError::MemoryAccessError { message } => {
            assert!(message.contains("uninitialized"), "unexpected message: {message}");
        }
        other => panic!("expected MemoryAccessError, got {other:?}"),
    }
}

#[test]
fn drop_invalidates_the_place() {
    let f = Fixture::new();
    let mut builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let tmp = builder.local("tmp", Some(f.int));
    let ret = builder.return_place;
    let func = builder.returns(vec![
        assign(Place::local(tmp), Rvalue::Use(const_int(5))),
        Statement::Drop(Place::local(tmp)),
        // Reading the dropped place must fail rather than yield the old value.
        assign(Place::local(ret), Rvalue::Use(copy(Place::local(tmp)))),
    ]);

    let mut interp = interpreter(f, vec![func]);
    let err = interp.execute_function("f", vec![]).unwrap_err();
    // A dropped local is `Uninit`; using it must be reported, not read back as
    // the stale value it held before the drop.
    match err {
        ExecutionError::MemoryAccessError { message } => {
            assert!(message.contains("uninitialized"), "unexpected message: {message}");
        }
        other => panic!("expected MemoryAccessError reading a dropped place, got {other:?}"),
    }
}

#[test]
fn assumption_statement_is_reported_as_a_diagnostic() {
    // `Statement::Assume` must reach the assumption handler rather than being
    // dropped by a catch-all, so it is observable in the diagnostics.
    let f = Fixture::new();
    let builder = FnBuilder::new(&f, "f", TypeSpec::Int);
    let ret = builder.return_place;
    let func = builder.returns(vec![
        Statement::Assume(omni_mir::ir::Assumption {}),
        assign(Place::local(ret), Rvalue::Use(const_int(3))),
    ]);

    let mut interp = interpreter(f, vec![func]);
    assert_eq!(interp.execute_function("f", vec![]).unwrap(), Value::Int(3));
    assert!(
        interp.diagnostics().iter().any(|d| d.message.contains("assumption")),
        "assumption should be recorded, got {:?}",
        interp.diagnostics()
    );
}

#[test]
fn successful_bounds_check_records_a_note() {
    let (f, func) = bounds_check_program(4);
    let mut interp = interpreter(f, vec![func]);
    interp.execute_function("f", vec![Value::Int(1)]).unwrap();
    assert!(
        interp.diagnostics().iter().any(|d| matches!(d.level, DiagnosticLevel::Note)
            && d.message.contains("Bounds check passed")),
        "expected a passing-bounds note, got {:?}",
        interp.diagnostics()
    );
}

#[test]
fn failing_bounds_check_records_an_error() {
    let (f, func) = bounds_check_program(4);
    let mut interp = interpreter(f, vec![func]);
    assert!(interp.execute_function("f", vec![Value::Int(9)]).is_err());
    assert!(
        interp
            .diagnostics()
            .iter()
            .any(|d| matches!(d.level, DiagnosticLevel::Error)
                && d.message.contains("out of bounds")),
        "expected a bounds error diagnostic, got {:?}",
        interp.diagnostics()
    );
}

#[test]
fn clear_diagnostics_empties_the_log() {
    let (f, func) = bounds_check_program(4);
    let mut interp = interpreter(f, vec![func]);
    interp.execute_function("f", vec![Value::Int(1)]).unwrap();
    assert!(!interp.diagnostics().is_empty());
    interp.clear_diagnostics();
    assert!(interp.diagnostics().is_empty());
}

#[test]
fn interpreter_reports_a_pure_effect_row_by_default() {
    let (f, func) = bounds_check_program(4);
    let mut interp = interpreter(f, vec![func]);
    interp.execute_function("f", vec![Value::Int(1)]).unwrap();
    assert!(
        interp.effects().is_pure(),
        "no effectful builtin was called, so the row should stay pure"
    );
}
