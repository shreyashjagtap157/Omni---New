//! Stage 4C native aggregate tests: stack-slot locals with layout-driven
//! field/element addressing.
//!
//! Scalar locals keep SSA `Variable` storage; aggregate locals take
//! `StackSlot` storage sized and aligned by `TargetLayout`, with struct
//! fields, constant tuple positions and constant array positions addressed by
//! layout offsets. Executable tests link and run, proving the projection
//! chain is really executable rather than merely accepted.

use omni_mir::ast::{Expr, GenericFnDef, Lit, Pattern, StructDef, StructFieldDef, TypeSpec};
use std::collections::HashMap;

fn pair_defs() -> HashMap<String, StructDef> {
    HashMap::from([(
        "Pair".to_string(),
        StructDef {
            name: "Pair".to_string(),
            type_params: vec![],
            fields: vec![
                StructFieldDef { name: "a".to_string(), ty: TypeSpec::Int },
                StructFieldDef { name: "b".to_string(), ty: TypeSpec::Int },
            ],
        },
    )])
}

fn int_main(body: Expr) -> omni_mir::MonomorphizedProgram {
    omni_mir::MonomorphizedProgram {
        functions: vec![GenericFnDef {
            // Named `omni_main` rather than `main`: the test shim below
            // supplies the real entry point and calls this, so the object
            // under test never collides with the host CRT entry.
            name: "omni_main".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: TypeSpec::Int,
            effects: omni_effects::EffectRow::pure(),
            capabilities: vec![],
            body,
        }],
    }
}

fn pair_value(first: i64, second: i64) -> Expr {
    Expr::Struct {
        name: "Pair".to_string(),
        generic_args: vec![],
        fields: vec![
            ("a".to_string(), Expr::Literal(Lit::Int(first))),
            ("b".to_string(), Expr::Literal(Lit::Int(second))),
        ],
    }
}

fn let_in(name: &str, init: Expr, body: Expr) -> Expr {
    Expr::Let {
        pattern: Pattern::Binding(name.to_string()),
        ty: None,
        init: Box::new(init),
        body: Box::new(body),
    }
}

fn field_of(base: Expr, field: &str) -> Expr {
    Expr::Field { expr: Box::new(base), field: field.to_string() }
}

fn return_of(value: Expr) -> Expr {
    Expr::Return(Some(Box::new(value)))
}

/// Links an object and runs it, returning the process exit code.
///
/// `cc` is not guaranteed on every dev machine, so linking goes through the
/// toolchain that is always present when these tests run: `rustc` forwards
/// the object to its self-contained linker, and a two-line shim supplies the
/// entry point that calls the compiled `omni_main`.
fn run_object(object: &[u8]) -> i32 {
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);
    let stem =
        format!("omni-codegen-4c-{}-{}", std::process::id(), SEQ.fetch_add(1, Ordering::SeqCst));
    let dir = std::env::temp_dir();
    let object_path = dir.join(format!("{stem}.o"));
    let shim_path = dir.join(format!("{stem}_shim.rs"));
    let exe_path = dir.join(format!("{stem}.exe"));
    std::fs::write(&object_path, object).expect("object write");
    std::fs::write(
        &shim_path,
        "unsafe extern \"C\" { fn omni_main() -> i64; }\nfn main() { std::process::exit(unsafe { omni_main() } as i32); }\n",
    )
    .expect("shim write");

    let link = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&shim_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-C")
        .arg(format!("link-arg={}", object_path.display()))
        .output()
        .expect("rustc must be available");
    assert!(
        link.status.success(),
        "rustc link failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&link.stdout),
        String::from_utf8_lossy(&link.stderr)
    );

    let run = std::process::Command::new(&exe_path).status().expect("executable must run");
    let code = run.code().expect("process must report an exit code");

    std::fs::remove_file(object_path).ok();
    std::fs::remove_file(shim_path).ok();
    std::fs::remove_file(exe_path).ok();
    code
}

#[test]
fn struct_field_load_executes() {
    // let p = Pair { a: 10, b: 20 }; return p.a;  =>  10
    let body =
        let_in("p", pair_value(10, 20), return_of(field_of(Expr::Var("p".to_string()), "a")));
    let object =
        omni_codegen::compile_monomorphized_program_with_structs(&int_main(body), pair_defs())
            .expect("struct field load must compile");
    assert_eq!(run_object(&object), 10);
}

#[test]
fn struct_field_store_executes() {
    // let p = Pair { a: 10, b: 20 }; p.a = 30; return p.a;  =>  30
    let body = let_in(
        "p",
        pair_value(10, 20),
        Expr::Block(vec![
            Expr::Assign {
                target: Box::new(field_of(Expr::Var("p".to_string()), "a")),
                value: Box::new(Expr::Literal(Lit::Int(30))),
            },
            return_of(field_of(Expr::Var("p".to_string()), "a")),
        ]),
    );
    let object =
        omni_codegen::compile_monomorphized_program_with_structs(&int_main(body), pair_defs())
            .expect("struct field store must compile");
    assert_eq!(run_object(&object), 30);
}

#[test]
fn struct_second_field_load_executes() {
    // The second field must land at its layout offset, not at zero.
    let body =
        let_in("p", pair_value(10, 20), return_of(field_of(Expr::Var("p".to_string()), "b")));
    let object =
        omni_codegen::compile_monomorphized_program_with_structs(&int_main(body), pair_defs())
            .expect("second field load must compile");
    assert_eq!(run_object(&object), 20);
}

#[test]
fn tuple_constant_index_load_executes() {
    // let t = (7, 8); return t.1;  =>  8
    let body = let_in(
        "t",
        Expr::Tuple(vec![Expr::Literal(Lit::Int(7)), Expr::Literal(Lit::Int(8))]),
        return_of(field_of(Expr::Var("t".to_string()), "1")),
    );
    let object = omni_codegen::compile_monomorphized_program(&int_main(body))
        .expect("tuple load must compile");
    assert_eq!(run_object(&object), 8);
}

#[test]
fn array_constant_index_load_executes() {
    // let a = [5, 6, 7]; return a[1];  =>  6
    let body = let_in(
        "a",
        Expr::Array(vec![
            Expr::Literal(Lit::Int(5)),
            Expr::Literal(Lit::Int(6)),
            Expr::Literal(Lit::Int(7)),
        ]),
        return_of(Expr::Index {
            expr: Box::new(Expr::Var("a".to_string())),
            index: Box::new(Expr::Literal(Lit::Int(1))),
        }),
    );
    let object = omni_codegen::compile_monomorphized_program(&int_main(body))
        .expect("array load must compile");
    assert_eq!(run_object(&object), 6);
}

#[test]
fn array_constant_index_store_executes() {
    // let a = [5, 6, 7]; a[0] = 9; return a[0] + a[2];  =>  16
    let body = let_in(
        "a",
        Expr::Array(vec![
            Expr::Literal(Lit::Int(5)),
            Expr::Literal(Lit::Int(6)),
            Expr::Literal(Lit::Int(7)),
        ]),
        Expr::Block(vec![
            Expr::Assign {
                target: Box::new(Expr::Index {
                    expr: Box::new(Expr::Var("a".to_string())),
                    index: Box::new(Expr::Literal(Lit::Int(0))),
                }),
                value: Box::new(Expr::Literal(Lit::Int(9))),
            },
            return_of(Expr::Binary {
                op: omni_mir::ast::BinOp::Add,
                lhs: Box::new(Expr::Index {
                    expr: Box::new(Expr::Var("a".to_string())),
                    index: Box::new(Expr::Literal(Lit::Int(0))),
                }),
                rhs: Box::new(Expr::Index {
                    expr: Box::new(Expr::Var("a".to_string())),
                    index: Box::new(Expr::Literal(Lit::Int(2))),
                }),
            }),
        ]),
    );
    let object = omni_codegen::compile_monomorphized_program(&int_main(body))
        .expect("array store must compile");
    assert_eq!(run_object(&object), 16);
}

#[test]
fn aggregate_let_copy_executes() {
    // let p = Pair { a: 10, b: 20 }; let q = p; return q.b;  =>  20
    let body = let_in(
        "p",
        pair_value(10, 20),
        let_in(
            "q",
            Expr::Var("p".to_string()),
            return_of(field_of(Expr::Var("q".to_string()), "b")),
        ),
    );
    let object =
        omni_codegen::compile_monomorphized_program_with_structs(&int_main(body), pair_defs())
            .expect("aggregate copy must compile");
    assert_eq!(run_object(&object), 20);
}

#[test]
fn dynamic_array_index_is_rejected_without_bounds_checks() {
    // a[i] with a runtime i must not become an unchecked native access.
    let body = let_in(
        "a",
        Expr::Array(vec![Expr::Literal(Lit::Int(5)), Expr::Literal(Lit::Int(6))]),
        let_in(
            "i",
            Expr::Literal(Lit::Int(1)),
            return_of(Expr::Index {
                expr: Box::new(Expr::Var("a".to_string())),
                index: Box::new(Expr::Var("i".to_string())),
            }),
        ),
    );
    let err = omni_codegen::compile_monomorphized_program(&int_main(body))
        .expect_err("dynamic indexing must fail closed until Stage 4D");
    assert!(err.contains("bounds"), "unexpected error: {err}");
}

#[test]
fn aggregate_parameter_is_rejected_at_the_abi_boundary() {
    let prog = omni_mir::MonomorphizedProgram {
        functions: vec![GenericFnDef {
            name: "first".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![("p".to_string(), TypeSpec::Struct("Pair".to_string(), vec![]))],
            return_type: TypeSpec::Int,
            effects: omni_effects::EffectRow::pure(),
            capabilities: vec![],
            body: return_of(field_of(Expr::Var("p".to_string()), "a")),
        }],
    };
    let err = omni_codegen::compile_monomorphized_program_with_structs(&prog, pair_defs())
        .expect_err("aggregate parameters require an explicit ABI contract");
    assert!(err.contains("ABI"), "unexpected error: {err}");
}

#[test]
fn aggregate_return_is_rejected_at_the_abi_boundary() {
    let prog = omni_mir::MonomorphizedProgram {
        functions: vec![GenericFnDef {
            name: "make".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: TypeSpec::Struct("Pair".to_string(), vec![]),
            effects: omni_effects::EffectRow::pure(),
            capabilities: vec![],
            body: pair_value(1, 2),
        }],
    };
    let err = omni_codegen::compile_monomorphized_program_with_structs(&prog, pair_defs())
        .expect_err("aggregate returns require an explicit ABI contract");
    assert!(err.contains("ABI"), "unexpected error: {err}");
}
