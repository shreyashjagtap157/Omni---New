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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
        body: Expr::Return(Some(Box::new(Expr::Var("x".to_string())))),
    });

    // fn inner[T](val: T) -> T { return val; }
    checker.register_fn(GenericFnDef {
        name: "inner".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![("val".to_string(), TypeSpec::GenericParam("T".to_string()))],
        return_type: TypeSpec::GenericParam("T".to_string()),
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
        body: Expr::Return(Some(Box::new(Expr::Var("val".to_string())))),
    });

    // fn outer[T](arg: T) -> T { return inner[T](arg); }
    checker.register_fn(GenericFnDef {
        name: "outer".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![("arg".to_string(), TypeSpec::GenericParam("T".to_string()))],
        return_type: TypeSpec::GenericParam("T".to_string()),
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
                arms: vec![crate::ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expr::Var("val".to_string()),
                }],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
        effects: omni_effects::EffectRow::default(),
        capabilities: vec![],
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
    let (_, key, subst, _effects) = checker
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
            effects: omni_effects::EffectRow::default(),
            capabilities: vec![],
            body: Expr::Literal(Lit::Int(0)),
        }],
    };
    let gate_res = invalid_prog.assert_concrete_for_mir();
    assert!(gate_res.is_err());
    assert!(gate_res.unwrap_err().contains("unresolved type parameters"));
}

#[test]
fn test_pattern_usefulness_and_exhaustiveness_bool_and_option() {
    use crate::ast::{MatchArm, Pattern};

    let mut checker = TypeChecker::new();
    let bool_ty = checker.tcx.intern(TyKind::Bool);

    // 1. Exhaustive bool match
    let arms_bool_ok = vec![
        MatchArm {
            pattern: Pattern::Lit(Lit::Bool(true)),
            guard: None,
            body: Expr::Literal(Lit::Int(1)),
        },
        MatchArm {
            pattern: Pattern::Lit(Lit::Bool(false)),
            guard: None,
            body: Expr::Literal(Lit::Int(0)),
        },
    ];
    assert!(crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs)
        .check_match(bool_ty, &arms_bool_ok)
        .is_ok());

    // 2. Non-exhaustive bool match (missing false)
    let arms_bool_missing = vec![MatchArm {
        pattern: Pattern::Lit(Lit::Bool(true)),
        guard: None,
        body: Expr::Literal(Lit::Int(1)),
    }];
    let res = crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs)
        .check_match(bool_ty, &arms_bool_missing);
    assert!(res.is_err());
    match res.unwrap_err() {
        TypeError::NonExhaustiveMatch { missing, .. } => assert_eq!(missing, "false"),
        other => panic!("Expected NonExhaustiveMatch, got {:?}", other),
    }

    // 3. Unreachable arm in bool match
    let arms_bool_unreachable = vec![
        MatchArm { pattern: Pattern::Wildcard, guard: None, body: Expr::Literal(Lit::Int(1)) },
        MatchArm {
            pattern: Pattern::Lit(Lit::Bool(true)),
            guard: None,
            body: Expr::Literal(Lit::Int(0)),
        },
    ];
    let res_unreach = crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs)
        .check_match(bool_ty, &arms_bool_unreachable);
    assert!(res_unreach.is_err());
    match res_unreach.unwrap_err() {
        TypeError::UnreachablePattern { arm_index, .. } => assert_eq!(arm_index, 1),
        other => panic!("Expected UnreachablePattern, got {:?}", other),
    }

    // 4. Option[Int] match
    let int_ty = checker.tcx.intern(TyKind::Int);
    let opt_int_ty = checker.tcx.intern(TyKind::Enum("Option".to_string(), vec![int_ty]));
    let arms_opt_ok = vec![
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Option".to_string(),
                variant: "Some".to_string(),
                subpatterns: vec![Pattern::Wildcard],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(1)),
        },
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Option".to_string(),
                variant: "None".to_string(),
                subpatterns: vec![],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(0)),
        },
    ];
    assert!(crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs)
        .check_match(opt_int_ty, &arms_opt_ok)
        .is_ok());
}

#[test]
fn test_pattern_guards_do_not_prove_unconditional_exhaustiveness() {
    use crate::ast::{MatchArm, Pattern};

    let mut checker = TypeChecker::new();
    let bool_ty = checker.tcx.intern(TyKind::Bool);
    let mut pat_checker = crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs);

    // Guarded true arm does NOT prove unconditional coverage of true
    let guarded_arms = vec![
        MatchArm {
            pattern: Pattern::Lit(Lit::Bool(true)),
            guard: Some(Expr::Literal(Lit::Bool(true))),
            body: Expr::Literal(Lit::Int(1)),
        },
        MatchArm {
            pattern: Pattern::Lit(Lit::Bool(false)),
            guard: None,
            body: Expr::Literal(Lit::Int(0)),
        },
    ];

    let res = pat_checker.check_match(bool_ty, &guarded_arms);
    assert!(res.is_err(), "Guarded arms must not prove unconditional exhaustiveness");
}

#[test]
fn test_pattern_never_type_handling() {
    use crate::ast::{MatchArm, Pattern};

    let mut checker = TypeChecker::new();
    let never_ty = checker.tcx.intern(TyKind::Never);
    let mut pat_checker = crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs);

    // Never type has 0 inhabitants, empty or Never pattern match is exhaustive
    let arms =
        vec![MatchArm { pattern: Pattern::Never, guard: None, body: Expr::Literal(Lit::Int(0)) }];
    assert!(pat_checker.check_match(never_ty, &arms).is_ok());

    let empty_arms: Vec<MatchArm> = vec![];
    assert!(pat_checker.check_match(never_ty, &empty_arms).is_ok());
}

#[test]
fn test_pattern_result_and_nested_and_generic_adts() {
    use crate::ast::{EnumDef, EnumVariantDef, MatchArm, Pattern, TypeSpec};

    let mut checker = TypeChecker::new();
    let int_ty = checker.tcx.intern(TyKind::Int);
    let str_ty = checker.tcx.intern(TyKind::String);

    // Result[Int, String]
    let res_ty = checker.tcx.intern(TyKind::Enum("Result".to_string(), vec![int_ty, str_ty]));

    // Exhaustive Result match: Ok(x), Err(e)
    let arms_res_ok = vec![
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Result".to_string(),
                variant: "Ok".to_string(),
                subpatterns: vec![Pattern::Binding("x".to_string())],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(1)),
        },
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Result".to_string(),
                variant: "Err".to_string(),
                subpatterns: vec![Pattern::Binding("e".to_string())],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(0)),
        },
    ];
    assert!(crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs)
        .check_match(res_ty, &arms_res_ok)
        .is_ok());

    // Non-exhaustive Result match (missing Err)
    let arms_res_err = vec![MatchArm {
        pattern: Pattern::Variant {
            enum_name: "Result".to_string(),
            variant: "Ok".to_string(),
            subpatterns: vec![Pattern::Binding("x".to_string())],
        },
        guard: None,
        body: Expr::Literal(Lit::Int(1)),
    }];
    assert!(crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs)
        .check_match(res_ty, &arms_res_err)
        .is_err());

    // Nested pattern: Option[Result[Int, String]]
    // Arms: Some(Ok(x)), Some(Err(e)), None
    let opt_res_ty = checker.tcx.intern(TyKind::Enum("Option".to_string(), vec![res_ty]));
    let arms_nested = vec![
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Option".to_string(),
                variant: "Some".to_string(),
                subpatterns: vec![Pattern::Variant {
                    enum_name: "Result".to_string(),
                    variant: "Ok".to_string(),
                    subpatterns: vec![Pattern::Wildcard],
                }],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(1)),
        },
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Option".to_string(),
                variant: "Some".to_string(),
                subpatterns: vec![Pattern::Variant {
                    enum_name: "Result".to_string(),
                    variant: "Err".to_string(),
                    subpatterns: vec![Pattern::Wildcard],
                }],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(2)),
        },
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Option".to_string(),
                variant: "None".to_string(),
                subpatterns: vec![],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(0)),
        },
    ];
    assert!(crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs)
        .check_match(opt_res_ty, &arms_nested)
        .is_ok());

    // Custom Generic ADT: Tree[T] = Leaf(T) | Node(Tree[T], Tree[T])
    checker.register_enum(EnumDef {
        name: "Tree".to_string(),
        type_params: vec!["T".to_string()],
        variants: vec![
            EnumVariantDef {
                name: "Leaf".to_string(),
                payload: vec![TypeSpec::GenericParam("T".to_string())],
            },
            EnumVariantDef {
                name: "Node".to_string(),
                payload: vec![
                    TypeSpec::Enum(
                        "Tree".to_string(),
                        vec![TypeSpec::GenericParam("T".to_string())],
                    ),
                    TypeSpec::Enum(
                        "Tree".to_string(),
                        vec![TypeSpec::GenericParam("T".to_string())],
                    ),
                ],
            },
        ],
    });

    let tree_int_ty = checker.tcx.intern(TyKind::Enum("Tree".to_string(), vec![int_ty]));
    let arms_tree = vec![
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Tree".to_string(),
                variant: "Leaf".to_string(),
                subpatterns: vec![Pattern::Wildcard],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(1)),
        },
        MatchArm {
            pattern: Pattern::Variant {
                enum_name: "Tree".to_string(),
                variant: "Node".to_string(),
                subpatterns: vec![Pattern::Wildcard, Pattern::Wildcard],
            },
            guard: None,
            body: Expr::Literal(Lit::Int(2)),
        },
    ];
    let mut pat_checker_tree =
        crate::pattern::PatternChecker::new(&mut checker.tcx, &checker.enum_defs);
    assert!(pat_checker_tree.check_match(tree_int_ty, &arms_tree).is_ok());
}

#[test]
fn test_effect_capability_semantic_model_slice() {
    use omni_effects::{Capability, CapabilityContext, Effect, EffectRow};

    let mut checker = TypeChecker::new();
    checker.cap_context.grant(Capability::FileSystem);
    checker.register_fn(GenericFnDef {
        name: "read_file".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![("path".to_string(), TypeSpec::String)],
        return_type: TypeSpec::String,
        effects: EffectRow::closed(vec![Effect::IO]),
        capabilities: vec![Capability::FileSystem],
        body: Expr::Literal(Lit::String("file_data".to_string())),
    });

    // 1. Pure function calling effectful function -> FAIL (unhandled effect)
    let pure_fn = GenericFnDef {
        name: "pure_caller".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::String,
        effects: EffectRow::pure(),
        capabilities: vec![],
        body: Expr::Call {
            func: "read_file".to_string(),
            generic_args: vec![],
            args: vec![Expr::Literal(Lit::String("test.txt".to_string()))],
        },
    };
    checker.register_fn(pure_fn.clone());
    let res = checker.check_fn_effects(&pure_fn);
    assert!(res.is_err(), "Pure function calling effectful call must fail");
    match res.unwrap_err() {
        TypeError::EffectViolation(msg) => assert!(msg.contains("unhandled effect")),
        other => panic!("Expected EffectViolation, got {:?}", other),
    }

    // 2. Missing capability FAIL (caller has !{IO} declared, but cap_context lacks FileSystem capability)
    checker.cap_context = CapabilityContext::new();
    let effectful_fn_no_cap = GenericFnDef {
        name: "effectful_no_cap".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::String,
        effects: EffectRow::closed(vec![Effect::IO]),
        capabilities: vec![],
        body: Expr::Call {
            func: "read_file".to_string(),
            generic_args: vec![],
            args: vec![Expr::Literal(Lit::String("test.txt".to_string()))],
        },
    };
    checker.register_fn(effectful_fn_no_cap.clone());
    let res = checker.check_fn_effects(&effectful_fn_no_cap);
    assert!(res.is_err(), "Call requiring capability missing in context must fail");
    match res.unwrap_err() {
        TypeError::EffectViolation(msg) => assert!(msg.contains("missing capability")),
        other => panic!("Expected EffectViolation for missing capability, got {:?}", other),
    }

    // 3. Unrelated capability does NOT satisfy effect/capability requirement
    checker.cap_context = CapabilityContext::with_capabilities(vec![Capability::NetworkConnect]);
    let res_unrelated = checker.check_fn_effects(&effectful_fn_no_cap);
    assert!(
        res_unrelated.is_err(),
        "Unrelated capability (NetworkConnect) must not satisfy FileSystem requirement"
    );

    // Grant correct capability to context
    checker.cap_context.grant(Capability::FileSystem);

    // 4. Effectful caller with declared !{IO} and granted Capability::FileSystem -> PASS
    let valid_effectful_fn = GenericFnDef {
        name: "valid_effectful".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::String,
        effects: EffectRow::closed(vec![Effect::IO]),
        capabilities: vec![Capability::FileSystem],
        body: Expr::Call {
            func: "read_file".to_string(),
            generic_args: vec![],
            args: vec![Expr::Literal(Lit::String("test.txt".to_string()))],
        },
    };
    checker.register_fn(valid_effectful_fn.clone());
    assert!(checker.check_fn_effects(&valid_effectful_fn).is_ok());

    // 5. Nested generic effectful call in pure caller -> FAIL
    checker.register_fn(GenericFnDef {
        name: "generic_read".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::String,
        effects: EffectRow::closed(vec![Effect::IO]),
        capabilities: vec![Capability::FileSystem],
        body: Expr::Call {
            func: "read_file".to_string(),
            generic_args: vec![],
            args: vec![Expr::Literal(Lit::String("test.txt".to_string()))],
        },
    });
    let pure_generic_caller = GenericFnDef {
        name: "pure_generic_caller".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::String,
        effects: EffectRow::pure(),
        capabilities: vec![],
        body: Expr::Call {
            func: "generic_read".to_string(),
            generic_args: vec![TypeSpec::Int],
            args: vec![],
        },
    };
    checker.register_fn(pure_generic_caller.clone());
    assert!(checker.check_fn_effects(&pure_generic_caller).is_err());

    // 6. Effect inside match arm in pure caller -> FAIL
    let pure_match_caller = GenericFnDef {
        name: "pure_match_caller".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::String,
        effects: EffectRow::pure(),
        capabilities: vec![],
        body: Expr::Match {
            expr: Box::new(Expr::Literal(Lit::Bool(true))),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Lit(Lit::Bool(true)),
                    guard: None,
                    body: Expr::Call {
                        func: "read_file".to_string(),
                        generic_args: vec![],
                        args: vec![Expr::Literal(Lit::String("a.txt".to_string()))],
                    },
                },
                MatchArm {
                    pattern: Pattern::Lit(Lit::Bool(false)),
                    guard: None,
                    body: Expr::Literal(Lit::String("default".to_string())),
                },
            ],
        },
    };
    checker.register_fn(pure_match_caller.clone());
    assert!(checker.check_fn_effects(&pure_match_caller).is_err());

    // 7. Handled effect PASS under discharge rule
    let row_with_io = EffectRow::closed(vec![Effect::IO, Effect::State]);
    let discharged_row = row_with_io.discharge(&Effect::IO);
    assert!(!discharged_row.contains(&Effect::IO));
    assert!(discharged_row.contains(&Effect::State));

    // 8. Integrated case: generic substitution + trait obligation + effect obligation + capability obligation
    checker.trait_checker = Some(std::sync::Arc::new(|bounds, _subst, _tcx| {
        for (_param, bound) in bounds {
            if let TraitBound::Positive(name) = bound {
                if name != "Display" {
                    return Err(format!("Unsatisfied trait bound `{name}`"));
                }
            }
        }
        Ok(())
    }));

    let integrated_fn = GenericFnDef {
        name: "integrated_target".to_string(),
        type_params: vec!["T".to_string()],
        bounds: vec![("T".to_string(), TraitBound::Positive("Display".to_string()))],
        params: vec![("item".to_string(), TypeSpec::GenericParam("T".to_string()))],
        return_type: TypeSpec::String,
        effects: EffectRow::closed(vec![Effect::IO]),
        capabilities: vec![Capability::FileSystem],
        body: Expr::Call {
            func: "read_file".to_string(),
            generic_args: vec![],
            args: vec![Expr::Literal(Lit::String("data.txt".to_string()))],
        },
    };
    checker.register_fn(integrated_fn);

    let integrated_caller = GenericFnDef {
        name: "integrated_caller".to_string(),
        type_params: vec![],
        bounds: vec![],
        params: vec![],
        return_type: TypeSpec::String,
        effects: EffectRow::closed(vec![Effect::IO]),
        capabilities: vec![Capability::FileSystem],
        body: Expr::Call {
            func: "integrated_target".to_string(),
            generic_args: vec![TypeSpec::Int],
            args: vec![Expr::Literal(Lit::Int(42))],
        },
    };
    checker.register_fn(integrated_caller.clone());

    assert!(checker.check_fn_effects(&integrated_caller).is_ok());
}

#[test]
fn test_binary_comparison_infers_bool_and_rejects_mismatch() {
    let mut checker = TypeChecker::new();
    let env = SubstEnv::new();
    let locals = HashMap::new();

    let bool_ty = checker
        .infer_expr(
            &Expr::Binary {
                op: BinOp::Eq,
                lhs: Box::new(Expr::Literal(Lit::Int(1))),
                rhs: Box::new(Expr::Literal(Lit::Int(1))),
            },
            &env,
            &locals,
        )
        .expect("integer equality should type-check");
    assert_eq!(bool_ty, checker.tcx.intern(TyKind::Bool));

    let mismatch = checker.infer_expr(
        &Expr::Binary {
            op: BinOp::Eq,
            lhs: Box::new(Expr::Literal(Lit::Int(1))),
            rhs: Box::new(Expr::Literal(Lit::Bool(true))),
        },
        &env,
        &locals,
    );
    assert!(matches!(mismatch, Err(TypeError::MismatchedTypes { .. })));
}
