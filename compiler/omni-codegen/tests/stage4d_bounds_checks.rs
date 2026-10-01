//! Stage 4D tests: Dynamic array indexing with explicit bounds checks.

use omni_mir::ast::{Expr, GenericFnDef, Lit, TypeSpec};
use std::collections::HashMap;

fn int_main_with_array_param(body: Expr) -> omni_mir::MonomorphizedProgram {
omni_mir::MonomorphizedProgram {
functions: vec![GenericFnDef {
// Named `omni_main` rather than `main`: the test shim below
// supplies the real entry point and calls this, so the object
// under test never collides with the host CRT entry.
name: "omni_main".to_string(),
type_params: vec![],
bounds: vec![],
params: vec![
("array".to_string(), TypeSpec::Array(Box::new(TypeSpec::Int), 5)),
("index".to_string(), TypeSpec::Int),
],
return_type: TypeSpec::Int,
effects: omni_effects::EffectRow::pure(),
capabilities: vec![],
body,
}],
}
}

fn int_main_with_locals(body: Expr) -> omni_mir::MonomorphizedProgram {
omni_mir::MonomorphizedProgram {
functions: vec![GenericFnDef {
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

fn int_main_with_index_param(body: Expr) -> omni_mir::MonomorphizedProgram {
omni_mir::MonomorphizedProgram {
functions: vec![GenericFnDef {
name: "omni_main".to_string(),
type_params: vec![],
bounds: vec![],
params: vec![("index".to_string(), TypeSpec::Int)],
return_type: TypeSpec::Int,
effects: omni_effects::EffectRow::pure(),
capabilities: vec![],
body,
}],
}
}

#[test]
fn constant_index_zero_is_valid() {
// Test that constant indexing generates MIR without bounds checks
let array = Expr::Array(vec![
Expr::Literal(Lit::Int(1)),
Expr::Literal(Lit::Int(2)),
Expr::Literal(Lit::Int(3)),
]);
let index = Expr::Literal(Lit::Int(0));
let body = Expr::Index {
expr: Box::new(array),
index: Box::new(index),
};

let program = int_main_with_locals(body);
let mir_function = omni_mir::lower::LoweringContext::new()
.lower_monomorphized_program(&program)
.expect("lowering should succeed");
let function = &mir_function.functions[0];

// Check that the MIR does NOT contain a BoundsCheck statement for constant indexing
let has_bounds_check = function.body.blocks.iter().any(|block| {
block.statements.iter().any(|stmt| {
matches!(stmt, omni_mir::ir::Statement::BoundsCheck { .. })
})
});

assert!(!has_bounds_check, "Constant indexing should not generate runtime bounds checks");
}

#[test]
fn constant_index_out_of_bounds_high_rejected() {
// Test that constant out-of-bounds indexing is rejected at compile time
let array = Expr::Array(vec![
Expr::Literal(Lit::Int(1)),
Expr::Literal(Lit::Int(2)),
Expr::Literal(Lit::Int(3)),
]);
let index = Expr::Literal(Lit::Int(5));
let body = Expr::Index {
expr: Box::new(array),
index: Box::new(index),
};

let program = int_main_with_locals(body);
let result = omni_mir::lower::LoweringContext::new()
.lower_monomorphized_program(&program);

// Should fail with out-of-bounds error
assert!(result.is_err(), "Constant out-of-bounds indexing should be rejected at compile time");
let error_msg = result.unwrap_err();
assert!(error_msg.contains("array index 5 out of bounds for length 3"), 
"Error message should indicate out-of-bounds access");
}

#[test]
fn dynamic_index_in_bounds() {
// Test that dynamic indexing generates MIR with bounds checks
let array = Expr::Array(vec![
Expr::Literal(Lit::Int(1)),
Expr::Literal(Lit::Int(2)),
Expr::Literal(Lit::Int(3)),
]);
let index = Expr::Var("index".to_string());
let body = Expr::Index {
expr: Box::new(array),
index: Box::new(index),
};

let program = int_main_with_index_param(body);
let mir_function = omni_mir::lower::LoweringContext::new()
.lower_monomorphized_program(&program)
.expect("lowering should succeed");
let function = &mir_function.functions[0];

// Check that the MIR contains a BoundsCheck statement for dynamic indexing
let has_bounds_check = function.body.blocks.iter().any(|block| {
block.statements.iter().any(|stmt| {
matches!(stmt, omni_mir::ir::Statement::BoundsCheck { .. })
})
});

assert!(has_bounds_check, "Dynamic indexing should generate runtime bounds checks");
}

#[test]
fn dynamic_index_out_of_bounds() {
// Test that dynamic out-of-bounds indexing generates MIR with bounds checks (will trap at runtime)
let array = Expr::Array(vec![
Expr::Literal(Lit::Int(1)),
Expr::Literal(Lit::Int(2)),
Expr::Literal(Lit::Int(3)),
]);
let index = Expr::Var("index".to_string());
let body = Expr::Index {
expr: Box::new(array),
index: Box::new(index),
};

let program = int_main_with_index_param(body);
let mir_function = omni_mir::lower::LoweringContext::new()
.lower_monomorphized_program(&program)
.expect("lowering should succeed");
let function = &mir_function.functions[0];

// Check that the MIR contains a BoundsCheck statement for dynamic indexing
let has_bounds_check = function.body.blocks.iter().any(|block| {
block.statements.iter().any(|stmt| {
matches!(stmt, omni_mir::ir::Statement::BoundsCheck { .. })
})
});

assert!(has_bounds_check, "Dynamic out-of-bounds indexing should generate runtime bounds checks");
}

#[test]
fn tuple_indexing_remains_compile_time() {
// Test that tuple indexing still uses constant bounds checking and doesn't generate runtime bounds checks
let tuple = Expr::Tuple(vec![
Expr::Literal(Lit::Int(1)),
Expr::Literal(Lit::Int(2)),
Expr::Literal(Lit::Int(3)),
]);
let index = Expr::Literal(Lit::Int(1)); // Constant index

let body = Expr::Index {
expr: Box::new(tuple),
index: Box::new(index),
};

let program = int_main_with_locals(body);
let mir_function = omni_mir::lower::LoweringContext::new()
.lower_monomorphized_program(&program)
.expect("lowering should succeed");
let function = &mir_function.functions[0];

// Check that the MIR does NOT contain a BoundsCheck statement for tuple indexing
let has_bounds_check = function.body.blocks.iter().any(|block| {
block.statements.iter().any(|stmt| {
matches!(stmt, omni_mir::ir::Statement::BoundsCheck { .. })
})
});

assert!(!has_bounds_check, "Tuple indexing should not generate runtime bounds checks");
}

#[test]
fn bounds_check_inserted_for_dynamic_index() {
// Test that bounds check statements are properly inserted in the MIR
// This is more of a structural test than an execution test
let array = Expr::Array(vec![
Expr::Literal(Lit::Int(1)),
Expr::Literal(Lit::Int(2)),
Expr::Literal(Lit::Int(3)),
]);
let index = Expr::Var("index".to_string()); // Dynamic index

let body = Expr::Index {
expr: Box::new(array),
index: Box::new(index),
};

let program = int_main_with_index_param(body);

// Check that the MIR contains a BoundsCheck statement
let mir_function = omni_mir::lower::LoweringContext::new()
.lower_monomorphized_program(&program)
.expect("lowering should succeed");
let function = &mir_function.functions[0];
let has_bounds_check = function.body.blocks.iter().any(|block| {
block.statements.iter().any(|stmt| {
matches!(stmt, omni_mir::ir::Statement::BoundsCheck { .. })
})
});

assert!(has_bounds_check, "MIR should contain BoundsCheck statement for dynamic indexing");
}

#[test]
fn no_bounds_check_for_constant_index() {
// Test that constant indices don't generate bounds checks
let array = Expr::Array(vec![
Expr::Literal(Lit::Int(1)),
Expr::Literal(Lit::Int(2)),
Expr::Literal(Lit::Int(3)),
]);
let index = Expr::Literal(Lit::Int(1)); // Constant index

let body = Expr::Index {
expr: Box::new(array),
index: Box::new(index),
};

let program = int_main_with_locals(body);

// Check that the MIR does NOT contain a BoundsCheck statement
let mir_function = omni_mir::lower::LoweringContext::new()
.lower_monomorphized_program(&program)
.expect("lowering should succeed");
let function = &mir_function.functions[0];
let has_bounds_check = function.body.blocks.iter().any(|block| {
block.statements.iter().any(|stmt| {
matches!(stmt, omni_mir::ir::Statement::BoundsCheck { .. })
})
});

assert!(!has_bounds_check, "MIR should not contain BoundsCheck statement for constant indexing");
}