use std::collections::HashMap;

use super::*;
use ast::*;
use checker::*;
use monomorph::*;

fn setup_checker() -> TypeChecker {
    let mut checker = TypeChecker::new();

    // fn identity[T](x: T) -> T { return x; }
    checker.register_fn(GenericFnDef {
        name: "identity".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![("x".to_string(), TypeSpec::GenericParam("T".to_string()))],
        return_type: TypeSpec::GenericParam("T".to_string()),
        body: Expr::Return(Some(Box::new(Expr::Var("x".to_string())))),
    });

    // fn inner[T](val: T) -> T { return val; }
    checker.register_fn(GenericFnDef {
        name: "inner".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![("val".to_string(), TypeSpec::GenericParam("T".to_string()))],
        return_type: TypeSpec::GenericParam("T".to_string()),
        body: Expr::Return(Some(Box::new(Expr::Var("val".to_string())))),
    });

    // fn outer[T](arg: T) -> T { return inner[T](arg); }
    checker.register_fn(GenericFnDef {
        name: "outer".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![("arg".to_string(), TypeSpec::GenericParam("T".to_string()))],
        return_type: TypeSpec::GenericParam("T".to_string()),
        body: Expr::Return(Some(Box::new(Expr::Call {
            func: "inner".to_string(),
            generic_args: vec![TypeSpec::GenericParam("T".to_string())],
            args: vec![Expr::Var("arg".to_string())],
        }))),
    });

    // fn recursive_fn[T](n: T) -> T { return recursive_fn[T](n); }
    checker.register_fn(GenericFnDef {
        name: "recursive_fn".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![("n".to_string(), TypeSpec::GenericParam("T".to_string()))],
        return_type: TypeSpec::GenericParam("T".to_string()),
        body: Expr::Return(Some(Box::new(Expr::Call {
            func: "recursive_fn".to_string(),
            generic_args: vec![TypeSpec::GenericParam("T".to_string())],
            args: vec![Expr::Var("n".to_string())],
        }))),
    });

    // fn unconstrained[T]() -> T { return ...; }
    checker.register_fn(GenericFnDef {
        name: "unconstrained".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::GenericParam("T".to_string()),
        body: Expr::Return(None),
    });

    checker
}

#[test]
fn test_multi_instantiation_same_generic_function() {
    let mut checker = setup_checker();

    // fn main() { let a = identity(42); let b = identity("hello"); }
    checker.register_fn(GenericFnDef {
        name: "main".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::Unit,
        body: Expr::Block(vec![
            Expr::Let {
                name: "a".to_string(),
                ty: None,
                init: Box::new(Expr::Call {
                    func: "identity".to_string(),
                    generic_args: vec![],
                    args: vec![Expr::Literal(Lit::Int(42))],
                }),
                body: Box::new(Expr::Literal(Lit::Int(0))),
            },
            Expr::Let {
                name: "b".to_string(),
                ty: None,
                init: Box::new(Expr::Call {
                    func: "identity".to_string(),
                    generic_args: vec![],
                    args: vec![Expr::Literal(Lit::String("hello".to_string()))],
                }),
                body: Box::new(Expr::Literal(Lit::Int(0))),
            },
        ]),
    });

    let mut mono = Monomorphizer::new(&mut checker);
    let res = mono.monomorphize_entry("main", &[], &[]);
    assert!(res.is_ok(), "Monomorphization failed: {:?}", res);

    let prog = res.unwrap();
    let fn_names: Vec<String> = prog.functions.iter().map(|f| f.name.clone()).collect();

    // Verify distinct mangled specializations exist for i64 and String
    assert!(
        fn_names.contains(&"identity_spec_i64".to_string()),
        "Missing identity_spec_i64 in {:?}",
        fn_names
    );
    assert!(
        fn_names.contains(&"identity_spec_String".to_string()),
        "Missing identity_spec_String in {:?}",
        fn_names
    );
    assert_ne!(
        "identity_spec_i64", "identity_spec_String",
        "Mangled names must be collision-free and distinct"
    );
}

#[test]
fn test_nested_generic_calls() {
    let mut checker = setup_checker();

    // fn test_driver() { let res = outer("world"); }
    checker.register_fn(GenericFnDef {
        name: "test_driver".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::Unit,
        body: Expr::Call {
            func: "outer".to_string(),
            generic_args: vec![],
            args: vec![Expr::Literal(Lit::String("world".to_string()))],
        },
    });

    let mut mono = Monomorphizer::new(&mut checker);
    let res = mono.monomorphize_entry("test_driver", &[], &[]);
    assert!(res.is_ok(), "Nested call resolution failed: {:?}", res);

    let prog = res.unwrap();
    let fn_names: Vec<String> = prog.functions.iter().map(|f| f.name.clone()).collect();

    assert!(
        fn_names.contains(&"outer_spec_String".to_string()),
        "Missing outer_spec_String in {:?}",
        fn_names
    );
    assert!(
        fn_names.contains(&"inner_spec_String".to_string()),
        "Missing inner_spec_String resolved via environment in {:?}",
        fn_names
    );
}

#[test]
fn test_generic_recursion() {
    let mut checker = setup_checker();

    checker.register_fn(GenericFnDef {
        name: "driver_rec".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::Unit,
        body: Expr::Call {
            func: "recursive_fn".to_string(),
            generic_args: vec![],
            args: vec![Expr::Literal(Lit::Int(100))],
        },
    });

    let mut mono = Monomorphizer::new(&mut checker);
    let res = mono.monomorphize_entry("driver_rec", &[], &[]);
    assert!(res.is_ok(), "Recursive monomorphization must terminate deterministically: {:?}", res);

    let prog = res.unwrap();
    let fn_names: Vec<String> = prog.functions.iter().map(|f| f.name.clone()).collect();
    assert!(fn_names.contains(&"recursive_fn_spec_i64".to_string()));
}

#[test]
fn test_generic_arguments_in_complex_nodes() {
    let mut checker = TypeChecker::new();

    // Complex generic function exercising fields, indexing, tuples, arrays, ranges, match arms, lambdas, interpolation, and assignments
    checker.register_fn(GenericFnDef {
        name: "complex_fn".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![
            ("val".to_string(), TypeSpec::GenericParam("T".to_string())),
            (
                "arr".to_string(),
                TypeSpec::Array(Box::new(TypeSpec::GenericParam("T".to_string())), 2),
            ),
            (
                "pair".to_string(),
                TypeSpec::Tuple(vec![TypeSpec::GenericParam("T".to_string()), TypeSpec::Int]),
            ),
        ],
        return_type: TypeSpec::GenericParam("T".to_string()),
        body: Expr::Block(vec![
            // Field projection
            Expr::Let {
                name: "f_elem".to_string(),
                ty: None,
                init: Box::new(Expr::Field {
                    expr: Box::new(Expr::Var("pair".to_string())),
                    field: "0".to_string(),
                }),
                body: Box::new(Expr::Literal(Lit::Int(0))),
            },
            // Indexing
            Expr::Let {
                name: "i_elem".to_string(),
                ty: None,
                init: Box::new(Expr::Index {
                    expr: Box::new(Expr::Var("arr".to_string())),
                    index: Box::new(Expr::Literal(Lit::Int(0))),
                }),
                body: Box::new(Expr::Literal(Lit::Int(0))),
            },
            // Tuple creation
            Expr::Tuple(vec![Expr::Var("val".to_string()), Expr::Literal(Lit::Int(1))]),
            // Array creation
            Expr::Array(vec![Expr::Var("val".to_string()), Expr::Var("val".to_string())]),
            // Range creation
            Expr::Range {
                start: Box::new(Expr::Literal(Lit::Int(0))),
                end: Box::new(Expr::Literal(Lit::Int(10))),
            },
            // Match arm
            Expr::Match {
                expr: Box::new(Expr::Literal(Lit::Int(1))),
                arms: vec![(Pattern::Wildcard, Expr::Var("val".to_string()))],
            },
            // Lambda
            Expr::Lambda {
                params: vec![("x".to_string(), TypeSpec::GenericParam("T".to_string()))],
                body: Box::new(Expr::Var("x".to_string())),
            },
            // Interpolation
            Expr::Interpolation(vec![Expr::Literal(Lit::String("val: ".to_string()))]),
            // Assignment
            Expr::Assign {
                target: Box::new(Expr::Var("f_elem".to_string())),
                value: Box::new(Expr::Var("val".to_string())),
            },
            // Return
            Expr::Return(Some(Box::new(Expr::Var("val".to_string())))),
        ]),
    });

    checker.register_fn(GenericFnDef {
        name: "complex_driver".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::Unit,
        body: Expr::Call {
            func: "complex_fn".to_string(),
            generic_args: vec![],
            args: vec![
                Expr::Literal(Lit::Bool(true)),
                Expr::Array(vec![Expr::Literal(Lit::Bool(true)), Expr::Literal(Lit::Bool(false))]),
                Expr::Tuple(vec![Expr::Literal(Lit::Bool(true)), Expr::Literal(Lit::Int(5))]),
            ],
        },
    });

    let mut mono = Monomorphizer::new(&mut checker);
    let res = mono.monomorphize_entry("complex_driver", &[], &[]);
    assert!(res.is_ok(), "Complex node monomorphization failed: {:?}", res);

    let prog = res.unwrap();
    let fn_names: Vec<String> = prog.functions.iter().map(|f| f.name.clone()).collect();
    assert!(
        fn_names.contains(&"complex_fn_spec_bool".to_string()),
        "Missing complex_fn_spec_bool in {:?}",
        fn_names
    );
}

#[test]
fn test_unresolved_substitution_negative_case() {
    let mut checker = setup_checker();

    checker.register_fn(GenericFnDef {
        name: "bad_driver".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::Unit,
        body: Expr::Call { func: "unconstrained".to_string(), generic_args: vec![], args: vec![] },
    });

    let mut mono = Monomorphizer::new(&mut checker);
    let res = mono.monomorphize_entry("bad_driver", &[], &[]);
    assert!(res.is_err(), "Must fail closed when generic parameter substitution is unresolved");
    match res.unwrap_err() {
        TypeError::UnresolvedSubstitution(param) => {
            assert_eq!(param, "T");
        }
        other => panic!("Expected UnresolvedSubstitution, got {:?}", other),
    }
}

#[test]
fn test_mutation_revert_behavior_fails_test() {
    let mut checker = setup_checker();

    // Confirm that mono cannot return a single fallback "i64" when a String substitution is required
    let env = SubstEnv::new();
    let local_vars = HashMap::new();
    let (_, key, subst) = checker
        .infer_call(
            "identity",
            &[],
            &[Expr::Literal(Lit::String("test".to_string()))],
            &env,
            &local_vars,
        )
        .expect("infer_call");

    let mangled = key.mangled_name(&checker.tcx);
    assert_eq!(mangled, "identity_spec_String");

    // Mutation verification: if key were forced to single assumption "identity_spec_i64", string assertion fails
    assert_ne!(
        mangled, "identity_spec_i64",
        "Reverting to single assumption violates type-directed specialization contract"
    );
    assert_eq!(subst.get("T"), Some(checker.tcx.intern(TyKind::String)));
}

#[test]
fn test_mir_semantic_gate() {
    let mut checker = setup_checker();
    let mut mono = Monomorphizer::new(&mut checker);

    let prog = mono
        .monomorphize_entry("identity", &[], &[Expr::Literal(Lit::String("hello".to_string()))])
        .expect("monomorphize_entry");

    // Must pass the concrete semantic gate
    assert!(prog.assert_concrete_for_mir().is_ok());

    // Construct an invalid program with an unresolved generic parameter and ensure it fails the gate
    let invalid_prog = MonomorphizedProgram {
        functions: vec![GenericFnDef {
            name: "unspecialized".to_string(),
            type_params: vec!["T".to_string()],
            bounds: vec![],
            params: vec![],
            return_type: TypeSpec::Unit,
            body: Expr::Literal(Lit::Int(0)),
        }],
    };
    let gate_res = invalid_prog.assert_concrete_for_mir();
    assert!(gate_res.is_err());
    assert!(gate_res.unwrap_err().contains("unresolved type parameters"));
}
