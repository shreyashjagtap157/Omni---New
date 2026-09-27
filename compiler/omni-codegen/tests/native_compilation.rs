//! MIR-driven native backend integration test.

#[test]
fn test_native_object_emission_from_typed_mir_program() {
    let prog = omni_mir::MonomorphizedProgram {
        functions: vec![omni_mir::ast::GenericFnDef {
            name: "main".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: omni_mir::ast::TypeSpec::Int,
            effects: omni_effects::EffectRow::pure(),
            capabilities: vec![],
            body: omni_mir::ast::Expr::Return(Some(Box::new(
                omni_mir::ast::Expr::Literal(omni_mir::ast::Lit::Int(42)),
            ))),
        }],
    };

    let bytes =
        omni_codegen::compile_monomorphized_program(&prog).expect("typed MIR native compilation");
    assert!(!bytes.is_empty(), "Emitted object file is empty");
}
