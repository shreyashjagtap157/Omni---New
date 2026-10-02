//! Comprehensive tests for the Omni abstract machine.
//!
//! These tests verify that the abstract machine produces correct results for various MIR programs,
//! including edge cases and error conditions.

use omni_machine::{Interpreter, Value, ExecutionError};
use omni_mir::ir::*;
use omni_types::ast::Lit;

// Helper macro to create index types
define_index_type! { struct TestLocal = u32; }

fn create_test_program() -> MirProgram {
    MirProgram {
        tcx: Default::default(),
        functions: Vec::new(),
        struct_defs: HashMap::new(),
    }
}

fn create_test_function(name: &str, params: Vec<TestLocal>, return_place: TestLocal, body: Body) -> MirFunction {
    MirFunction {
        name: name.to_string(),
        params: params.into_iter().map(|p| p.into()).collect(),
        return_place: return_place.into(),
        return_type: omni_types::ast::TypeSpec::Int,
        body,
    }
}

fn create_test_block(statements: Vec<Statement>, terminator: Option<Terminator>) -> BlockData {
    BlockData {
        statements,
        terminator,
    }
}

fn create_place(local: TestLocal) -> Place {
    Place::local(local.into())
}

fn create_operand_copy(local: TestLocal) -> Operand {
    Operand::Copy(create_place(local))
}

fn create_operand_constant(value: Constant) -> Operand {
    Operand::Constant(value)
}

fn create_rvalue_use(operand: Operand) -> Rvalue {
    Rvalue::Use(operand)
}

fn create_rvalue_binary_op(op: BinOp, left: Operand, right: Operand) -> Rvalue {
    Rvalue::BinaryOp(op, left, right)
}

fn create_rvalue_unary_op(op: UnOp, operand: Operand) -> Rvalue {
    Rvalue::UnaryOp(op, operand)
}

fn create_rvalue_aggregate(kind: AggregateKind, operands: Vec<Operand>) -> Rvalue {
    Rvalue::Aggregate { kind, operands, ty: omni_types::intern::Ty::new() }
}

#[test]
fn test_simple_addition() {
    let mut interpreter = Interpreter::new();
    
    // Create a simple function that adds two numbers
    let local_a = TestLocal::from(0);
    let local_b = TestLocal::from(1);
    let local_result = TestLocal::from(2);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_result),
            create_rvalue_binary_op(
                BinOp::Add,
                create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(10))),
                create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(5))),
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("b".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("add", vec![local_a, local_b], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("add", vec![
        Value::Int(10),
        Value::Int(5),
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}

#[test]
fn test_subtraction_and_multiplication() {
    let mut interpreter = Interpreter::new();
    
    let local_x = TestLocal::from(0);
    let local_y = TestLocal::from(1);
    let local_temp = TestLocal::from(2);
    let local_result = TestLocal::from(3);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_temp),
            create_rvalue_binary_op(
                BinOp::Sub,
                create_operand_copy(local_x),
                create_operand_copy(local_y),
            ),
        ),
        Statement::Assign(
            create_place(local_result),
            create_rvalue_binary_op(
                BinOp::Mul,
                create_operand_copy(local_temp),
                create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(2))),
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("y".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("temp".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("compute", vec![local_x, local_y], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("compute", vec![
        Value::Int(20),
        Value::Int(8),
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(24)); // (20 - 8) * 2 = 24
}

#[test]
fn test_unary_operations() {
    let mut interpreter = Interpreter::new();
    
    let local_a = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_result),
            create_rvalue_unary_op(
                UnOp::Neg,
                create_operand_copy(local_a),
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("negate", vec![local_a], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("negate", vec![
        Value::Int(-42),
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_boolean_operations() {
    let mut interpreter = Interpreter::new();
    
    let local_a = TestLocal::from(0);
    let local_b = TestLocal::from(1);
    let local_result = TestLocal::from(2);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_result),
            create_rvalue_binary_op(
                BinOp::Eq,
                create_operand_copy(local_a),
                create_operand_copy(local_b),
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("b".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("logical_and", vec![local_a, local_b], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    
    // Test true && true = true
    let result1 = interpreter.execute_function("logical_and", vec![
        Value::Bool(true),
        Value::Bool(true),
    ]);
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), Value::Bool(true));
    
    // Test true && false = false
    let result2 = interpreter.execute_function("logical_and", vec![
        Value::Bool(true),
        Value::Bool(false),
    ]);
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Value::Bool(false));
}

#[test]
fn test_aggregate_construction() {
    let mut interpreter = Interpreter::new();
    
    let local_a = TestLocal::from(0);
    let local_b = TestLocal::from(1);
    let local_tuple = TestLocal::from(2);
    let local_array = TestLocal::from(3);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_tuple),
            create_rvalue_aggregate(
                AggregateKind::Tuple,
                vec![
                    create_operand_copy(local_a),
                    create_operand_copy(local_b),
                ],
            ),
        ),
        Statement::Assign(
            create_place(local_array),
            create_rvalue_aggregate(
                AggregateKind::Array,
                vec![
                    create_operand_copy(local_a),
                    create_operand_copy(local_b),
                ],
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("b".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("tuple".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("array".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("create_aggregates", vec![local_a, local_b], local_tuple, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("create_aggregates", vec![
        Value::Int(10),
        Value::Int(20),
    ]);
    
    assert!(result.is_ok());
    let result_value = result.unwrap();
    
    // Should be a tuple
    match result_value {
        Value::Tuple(elements) => {
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0], Value::Int(10));
            assert_eq!(elements[1], Value::Int(20));
        }
        _ => panic!("Expected tuple result"),
    }
}

#[test]
fn test_control_flow_goto() {
    let mut interpreter = Interpreter::new();
    
    let local_a = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    // Create two basic blocks
    let block0 = create_test_block(
        vec![
            Statement::Assign(
                create_place(local_result),
                create_rvalue_use(create_operand_copy(local_a)),
            ),
        ],
        Some(Terminator::Goto(BasicBlock::from(1))),
    );
    
    let block1 = create_test_block(
        vec![],
        Some(Terminator::Return),
    );
    
    let body = Body {
        blocks: vec![block0.into(), block1.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("goto_test", vec![local_a], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("goto_test", vec![
        Value::Int(42),
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_control_flow_switch() {
    let mut interpreter = Interpreter::new();
    
    let local_x = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let block0 = create_test_block(
        vec![
            Statement::Assign(
                create_place(local_result),
                create_rvalue_use(create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(1)))),
            ),
        ],
        Some(Terminator::SwitchInt {
            discr: create_operand_copy(local_x),
            targets: vec![
                (0, BasicBlock::from(1)), // 0 -> block1
                (1, BasicBlock::from(2)), // 1 -> block2
            ],
            otherwise: BasicBlock::from(3), // default -> block3
        }),
    );
    
    let block1 = create_test_block(
        vec![],
        Some(Terminator::Return),
    );
    
    let block2 = create_test_block(
        vec![],
        Some(Terminator::Return),
    );
    
    let block3 = create_test_block(
        vec![],
        Some(Terminator::Return),
    );
    
    let body = Body {
        blocks: vec![block0.into(), block1.into(), block2.into(), block3.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("switch_test", vec![local_x], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    
    // Test case 0
    let result0 = interpreter.execute_function("switch_test", vec![
        Value::Int(0),
    ]);
    assert!(result0.is_ok());
    assert_eq!(result0.unwrap(), Value::Int(1));
    
    // Test case 1
    let result1 = interpreter.execute_function("switch_test", vec![
        Value::Int(1),
    ]);
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), Value::Int(1));
    
    // Test default case
    let result2 = interpreter.execute_function("switch_test", vec![
        Value::Int(2),
    ]);
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Value::Int(1));
}

#[test]
fn test_bounds_check_pass() {
    let mut interpreter = Interpreter::new();
    
    let local_index = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::BoundsCheck { index: local_index, length: 10 },
        Statement::Assign(
            create_place(local_result),
            create_rvalue_use(create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(42)))),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("index".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("bounds_check_pass", vec![local_index], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("bounds_check_pass", vec![
        Value::Int(5), // Valid index (5 < 10)
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_bounds_check_fail() {
    let mut interpreter = Interpreter::new();
    
    let local_index = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::BoundsCheck { index: local_index, length: 10 },
        Statement::Assign(
            create_place(local_result),
            create_rvalue_use(create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(42)))),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("index".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("bounds_check_fail", vec![local_index], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("bounds_check_fail", vec![
        Value::Int(15), // Invalid index (15 >= 10)
    ]);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ExecutionError::BoundsCheckFailed { index, length } => {
            assert_eq!(index, 15);
            assert_eq!(length, 10);
        }
        _ => panic!("Expected BoundsCheckFailed error"),
    }
}

#[test]
fn test_division_by_zero() {
    let mut interpreter = Interpreter::new();
    
    let local_a = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_result),
            create_rvalue_binary_op(
                BinOp::Div,
                create_operand_copy(local_a),
                create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(0))),
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("div_by_zero", vec![local_a], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("div_by_zero", vec![
        Value::Int(10),
    ]);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ExecutionError::DivisionByZero => {
            // Expected error
        }
        _ => panic!("Expected DivisionByZero error"),
    }
}

#[test]
fn test_function_call() {
    let mut interpreter = Interpreter::new();
    
    // Create a nested function call scenario
    let local_x = TestLocal::from(0);
    let local_temp = TestLocal::from(1);
    let local_result = TestLocal::from(2);
    
    // Inner function: square
    let inner_local_x = TestLocal::from(0);
    let inner_local_result = TestLocal::from(1);
    
    let inner_statements = vec![
        Statement::Assign(
            create_place(inner_local_result),
            create_rvalue_binary_op(
                BinOp::Mul,
                create_operand_copy(inner_local_x),
                create_operand_copy(inner_local_x),
            ),
        ),
    ];
    
    let inner_terminator = Some(Terminator::Return);
    let inner_block = create_test_block(inner_statements, inner_terminator);
    let inner_body = Body {
        blocks: vec![inner_block.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let inner_function = create_test_function("square", vec![inner_local_x], inner_local_result, inner_body);
    
    // Outer function: calls square
    let outer_statements = vec![
        Statement::Assign(
            create_place(local_temp),
            create_rvalue_use(create_operand_copy(local_x)),
        ),
        Statement::Assign(
            create_place(local_result),
            create_rvalue_use(create_operand_copy(local_temp)),
        ),
    ];
    
    let outer_terminator = Some(Terminator::Return);
    let outer_block = create_test_block(outer_statements, outer_terminator);
    let outer_body = Body {
        blocks: vec![outer_block.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("temp".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let outer_function = create_test_function("main", vec![local_x], local_result, outer_body);
    
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![inner_function, outer_function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("main", vec![
        Value::Int(5),
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(5));
}

#[test]
fn test_unreachable_code() {
    let mut interpreter = Interpreter::new();
    
    let local_result = TestLocal::from(0);
    
    let block0 = create_test_block(
        vec![],
        Some(Terminator::Unreachable),
    );
    
    let body = Body {
        blocks: vec![block0.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("unreachable_test", vec![], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("unreachable_test", vec![]);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ExecutionError::UnreachableCode => {
            // Expected error
        }
        _ => panic!("Expected UnreachableCode error"),
    }
}

#[test]
fn test_multiple_basic_blocks_complex() {
    let mut interpreter = Interpreter::new();
    
    let local_x = TestLocal::from(0);
    let local_y = TestLocal::from(1);
    let local_result = TestLocal::from(2);
    
    // Block 0: entry point
    let block0 = create_test_block(
        vec![
            Statement::Assign(
                create_place(local_result),
                create_rvalue_use(create_operand_copy(local_x)),
            ),
        ],
        Some(Terminator::SwitchInt {
            discr: create_operand_copy(local_y),
            targets: vec![
                (0, BasicBlock::from(1)), // y == 0 -> block1
                (1, BasicBlock::from(2)), // y == 1 -> block2
            ],
            otherwise: BasicBlock::from(3), // default -> block3
        }),
    );
    
    // Block 1: y == 0, result = x + 1
    let block1 = create_test_block(
        vec![
            Statement::Assign(
                create_place(local_result),
                create_rvalue_binary_op(
                    BinOp::Add,
                    create_operand_copy(local_result),
                    create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(1))),
                ),
            ),
        ],
        Some(Terminator::Return),
    );
    
    // Block 2: y == 1, result = x * 2
    let block2 = create_test_block(
        vec![
            Statement::Assign(
                create_place(local_result),
                create_rvalue_binary_op(
                    BinOp::Mul,
                    create_operand_copy(local_result),
                    create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(2))),
                ),
            ),
        ],
        Some(Terminator::Return),
    );
    
    // Block 3: default, result = x - 1
    let block3 = create_test_block(
        vec![
            Statement::Assign(
                create_place(local_result),
                create_rvalue_binary_op(
                    BinOp::Sub,
                    create_operand_copy(local_result),
                    create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(1))),
                ),
            ),
        ],
        Some(Terminator::Return),
    );
    
    let body = Body {
        blocks: vec![block0.into(), block1.into(), block2.into(), block3.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("y".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("complex_control_flow", vec![local_x, local_y], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    
    // Test case: y == 0
    let result1 = interpreter.execute_function("complex_control_flow", vec![
        Value::Int(10),
        Value::Int(0),
    ]);
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), Value::Int(11)); // 10 + 1
    
    // Test case: y == 1
    let result2 = interpreter.execute_function("complex_control_flow", vec![
        Value::Int(10),
        Value::Int(1),
    ]);
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Value::Int(20)); // 10 * 2
    
    // Test case: default
    let result3 = interpreter.execute_function("complex_control_flow", vec![
        Value::Int(10),
        Value::Int(2),
    ]);
    assert!(result3.is_ok());
    assert_eq!(result3.unwrap(), Value::Int(9)); // 10 - 1
}

#[test]
fn test_edge_cases() {
    let mut interpreter = Interpreter::new();
    
    // Test with very large numbers
    let result1 = interpreter.run_snippet("large_number");
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), 42); // Current stub implementation
    
    // Test with negative numbers
    let local_a = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_result),
            create_rvalue_binary_op(
                BinOp::Add,
                create_operand_copy(local_a),
                create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(-5))),
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("negative_test", vec![local_a], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("negative_test", vec![
        Value::Int(10),
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(5)); // 10 + (-5) = 5
}

#[test]
fn test_error_messages() {
    let mut interpreter = Interpreter::new();
    
    // Test division by zero error message
    let local_a = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_result),
            create_rvalue_binary_op(
                BinOp::Div,
                create_operand_copy(local_a),
                create_operand_constant(Constant::Lit(omni_types::ast::Lit::Int(0))),
            ),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("div_error_test", vec![local_a], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("div_error_test", vec![
        Value::Int(10),
    ]);
    
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Division by zero"));
    
    // Test bounds check error message
    let local_index = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::BoundsCheck { index: local_index, length: 5 },
    ];
    
    let block = create_test_block(statements, Some(Terminator::Return));
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("index".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("bounds_error_test", vec![local_index], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("bounds_error_test", vec![
        Value::Int(10),
    ]);
    
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Bounds check failed"));
    assert!(error_message.contains("index 10 >= length 5"));
}

#[test]
fn test_function_not_found() {
    let mut interpreter = Interpreter::new();
    
    let program = MirProgram {
        tcx: Default::default(),
        functions: Vec::new(),
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("nonexistent_function", vec![]);
    
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Function not found"));
}

#[test]
fn test_argument_count_mismatch() {
    let mut interpreter = Interpreter::new();
    
    let local_a = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_result),
            create_rvalue_use(create_operand_copy(local_a)),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("single_param", vec![local_a], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("single_param", vec![
        Value::Int(1),
        Value::Int(2), // Extra argument
    ]);
    
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Argument count mismatch"));
    assert!(error_message.contains("expected 1, got 2"));
}

// Helper function needed for the tests
use std::collections::HashMap;

#[test]
fn test_field_projection() {
    let mut interpreter = Interpreter::new();
    
    let local_struct = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    // Create a struct value
    let struct_value = Value::Struct {
        name: "TestStruct".to_string(),
        fields: vec![
            ("field1".to_string(), Value::Int(10)),
            ("field2".to_string(), Value::String("hello".to_string())),
        ].into_iter().collect(),
    };
    
    // Assign struct to local
    interpreter.locals.insert(local_struct.into(), struct_value);
    
    // Test field projection
    let place = Place::local(local_struct.into()).project(Projection::Field("field1".to_string()));
    let result = interpreter.get_place_value(&place, Rc::new(create_test_function("test", vec![], local_result.into(), Body::default())));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(10));
}

#[test]
fn test_constant_index_projection() {
    let mut interpreter = Interpreter::new();
    
    let local_array = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    // Create an array value
    let array_value = Value::Array(vec![
        Value::Int(10),
        Value::Int(20),
        Value::Int(30),
    ]);
    
    // Assign array to local
    interpreter.locals.insert(local_array.into(), array_value);
    
    // Test constant index projection
    let place = Place::local(local_array.into()).project(Projection::ConstantIndex(1));
    let result = interpreter.get_place_value(&place, Rc::new(create_test_function("test", vec![], local_result.into(), Body::default())));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(20));
}

#[test]
fn test_index_projection_with_local() {
    let mut interpreter = Interpreter::new();
    
    let local_array = TestLocal::from(0);
    let local_index = TestLocal::from(1);
    let local_result = TestLocal::from(2);
    
    // Create an array value
    let array_value = Value::Array(vec![
        Value::Int(10),
        Value::Int(20),
        Value::Int(30),
    ]);
    
    // Assign array and index to locals
    interpreter.locals.insert(local_array.into(), array_value);
    interpreter.locals.insert(local_index.into(), Value::Int(1));
    
    // Test index projection with local
    let place = Place::local(local_array.into()).project(Projection::Index(local_index.into()));
    let result = interpreter.get_place_value(&place, Rc::new(create_test_function("test", vec![], local_result.into(), Body::default())));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(20));
}

#[test]
fn test_memory_allocation() {
    let mut interpreter = Interpreter::new();
    
    let local_size = TestLocal::from(0);
    let local_align = TestLocal::from(1);
    let local_result = TestLocal::from(2);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_size),
            create_rvalue_use(create_operand_constant(Constant::Lit(Lit::Int(100)))),
        ),
        Statement::Assign(
            create_place(local_align),
            create_rvalue_use(create_operand_constant(Constant::Lit(Lit::Int(8)))),
        ),
        Statement::Assign(
            create_place(local_result),
            create_rvalue_use(create_operand_constant(Constant::FnRef("alloc".to_string()))),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("size".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("align".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("memory_test", vec![], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("memory_test", vec![]);
    
    assert!(result.is_ok());
    match result.unwrap() {
        Value::Int(alloc_id) => {
            assert!(alloc_id > 0);
            assert!(interpreter.memory.allocation_exists(alloc_id as u64));
        }
        _ => panic!("Expected allocation ID as integer"),
    }
}

#[test]
fn test_memory_deallocation() {
    let mut interpreter = Interpreter::new();
    
    let local_size = TestLocal::from(0);
    let local_align = TestLocal::from(1);
    let local_alloc_id = TestLocal::from(2);
    let local_result = TestLocal::from(3);
    
    let statements = vec![
        Statement::Assign(
            create_place(local_size),
            create_rvalue_use(create_operand_constant(Constant::Lit(Lit::Int(100)))),
        ),
        Statement::Assign(
            create_place(local_align),
            create_rvalue_use(create_operand_constant(Constant::Lit(Lit::Int(8)))),
        ),
        Statement::Assign(
            create_place(local_alloc_id),
            create_rvalue_use(create_operand_constant(Constant::FnRef("alloc".to_string()))),
        ),
        Statement::Assign(
            create_place(local_result),
            create_rvalue_use(create_operand_constant(Constant::FnRef("dealloc".to_string()))),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("size".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("align".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("alloc_id".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("dealloc_test", vec![], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("dealloc_test", vec![]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Unit);
}

#[test]
fn test_nested_function_calls() {
    let mut interpreter = Interpreter::new();
    
    // Inner function: add one
    let inner_local_x = TestLocal::from(0);
    let inner_local_result = TestLocal::from(1);
    
    let inner_statements = vec![
        Statement::Assign(
            create_place(inner_local_result),
            create_rvalue_binary_op(
                BinOp::Add,
                create_operand_copy(inner_local_x),
                create_operand_constant(Constant::Lit(Lit::Int(1))),
            ),
        ),
    ];
    
    let inner_terminator = Some(Terminator::Return);
    let inner_block = create_test_block(inner_statements, inner_terminator);
    let inner_body = Body {
        blocks: vec![inner_block.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let inner_function = create_test_function("add_one", vec![inner_local_x], inner_local_result, inner_body);
    
    // Middle function: add two
    let middle_local_x = TestLocal::from(0);
    let middle_local_temp = TestLocal::from(1);
    let middle_local_result = TestLocal::from(2);
    
    let middle_statements = vec![
        Statement::Assign(
            create_place(middle_local_temp),
            create_rvalue_use(create_operand_copy(middle_local_x)),
        ),
        Statement::Assign(
            create_place(middle_local_result),
            create_rvalue_use(create_operand_constant(Constant::FnRef("add_one".to_string()))),
        ),
    ];
    
    let middle_terminator = Some(Terminator::Return);
    let middle_block = create_test_block(middle_statements, middle_terminator);
    let middle_body = Body {
        blocks: vec![middle_block.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("temp".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let middle_function = create_test_function("add_two", vec![middle_local_x], middle_local_result, middle_body);
    
    // Outer function: add three
    let outer_local_x = TestLocal::from(0);
    let outer_local_temp = TestLocal::from(1);
    let outer_local_result = TestLocal::from(2);
    
    let outer_statements = vec![
        Statement::Assign(
            create_place(outer_local_temp),
            create_rvalue_use(create_operand_copy(outer_local_x)),
        ),
        Statement::Assign(
            create_place(outer_local_result),
            create_rvalue_use(create_operand_constant(Constant::FnRef("add_two".to_string()))),
        ),
    ];
    
    let outer_terminator = Some(Terminator::Return);
    let outer_block = create_test_block(outer_statements, outer_terminator);
    let outer_body = Body {
        blocks: vec![outer_block.into()],
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("temp".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let outer_function = create_test_function("add_three", vec![outer_local_x], outer_local_result, outer_body);
    
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![inner_function, middle_function, outer_function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("add_three", vec![Value::Int(10)]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(13)); // 10 + 3 = 13
}

#[test]
fn test_diagnostic_reporting() {
    let mut interpreter = Interpreter::new();
    
    let local_index = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        Statement::BoundsCheck { index: local_index, length: 5 },
        Statement::Assign(
            create_place(local_result),
            create_rvalue_use(create_operand_constant(Constant::Lit(Lit::Int(42)))),
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("index".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("diagnostic_test", vec![local_index], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("diagnostic_test", vec![Value::Int(10)]);
    
    // Should fail due to bounds check
    assert!(result.is_err());
    
    // Check that diagnostics were collected
    let diagnostics = interpreter.diagnostics();
    assert!(!diagnostics.is_empty());
    
    // Check for error diagnostic
    let error_diagnostic = diagnostics.iter().find(|d| d.level == DiagnosticLevel::Error);
    assert!(error_diagnostic.is_some());
    assert!(error_diagnostic.unwrap().message.contains("out of bounds"));
    
    // Check for note diagnostic
    let note_diagnostic = diagnostics.iter().find(|d| d.level == DiagnosticLevel::Note);
    assert!(note_diagnostic.is_some());
}

#[test]
fn test_complex_aggregate_operations() {
    let mut interpreter = Interpreter::new();
    
    let local_a = TestLocal::from(0);
    let local_b = TestLocal::from(1);
    let local_c = TestLocal::from(2);
    let local_tuple = TestLocal::from(3);
    let local_nested = TestLocal::from(4);
    
    let statements = vec![
        // Create a tuple: (a, b, c)
        Statement::Assign(
            create_place(local_tuple),
            Rvalue::Aggregate {
                kind: AggregateKind::Tuple,
                operands: vec![
                    create_operand_copy(local_a),
                    create_operand_copy(local_b),
                    create_operand_copy(local_c),
                ],
                ty: omni_types::intern::Ty::new(),
            },
        ),
        // Create a nested tuple: (tuple, a)
        Statement::Assign(
            create_place(local_nested),
            Rvalue::Aggregate {
                kind: AggregateKind::Tuple,
                operands: vec![
                    create_operand_copy(local_tuple),
                    create_operand_copy(local_a),
                ],
                ty: omni_types::intern::Ty::new(),
            },
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("a".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("b".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("c".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("tuple".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("nested".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("aggregate_test", vec![local_a, local_b, local_c], local_nested, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("aggregate_test", vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ]);
    
    assert!(result.is_ok());
    let result_value = result.unwrap();
    
    // Should be a nested tuple
    match result_value {
        Value::Tuple(elements) => {
            assert_eq!(elements.len(), 2);
            
            // First element should be a tuple (1, 2, 3)
            match &elements[0] {
                Value::Tuple(inner) => {
                    assert_eq!(inner.len(), 3);
                    assert_eq!(inner[0], Value::Int(1));
                    assert_eq!(inner[1], Value::Int(2));
                    assert_eq!(inner[2], Value::Int(3));
                }
                _ => panic!("Expected inner tuple"),
            }
            
            // Second element should be 1
            assert_eq!(elements[1], Value::Int(1));
        }
        _ => panic!("Expected tuple result"),
    }
}

#[test]
fn test_struct_operations() {
    let mut interpreter = Interpreter::new();
    
    let local_x = TestLocal::from(0);
    let local_y = TestLocal::from(1);
    let local_point = TestLocal::from(2);
    
    let statements = vec![
        // Create a struct: Point { x: x, y: y }
        Statement::Assign(
            create_place(local_point),
            Rvalue::Struct {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), create_operand_copy(local_x)),
                    ("y".to_string(), create_operand_copy(local_y)),
                ],
                ty: omni_types::intern::Ty::new(),
            },
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("x".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("y".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("point".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("struct_test", vec![local_x, local_y], local_point, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("struct_test", vec![
        Value::Int(10),
        Value::Int(20),
    ]);
    
    assert!(result.is_ok());
    let result_value = result.unwrap();
    
    // Should be a struct
    match result_value {
        Value::Struct { name, fields } => {
            assert_eq!(name, "Point");
            assert_eq!(fields.get("x"), Some(&Value::Int(10)));
            assert_eq!(fields.get("y"), Some(&Value::Int(20)));
        }
        _ => panic!("Expected struct result"),
    }
}

#[test]
fn test_enum_operations() {
    let mut interpreter = Interpreter::new();
    
    let local_value = TestLocal::from(0);
    let local_result = TestLocal::from(1);
    
    let statements = vec![
        // Create an enum variant: Some(value)
        Statement::Assign(
            create_place(local_result),
            Rvalue::EnumVariant {
                enum_name: "Option".to_string(),
                variant: "Some".to_string(),
                operands: vec![create_operand_copy(local_value)],
                ty: omni_types::intern::Ty::new(),
            },
        ),
    ];
    
    let terminator = Some(Terminator::Return);
    let block = create_test_block(statements, terminator);
    let body = Body {
        blocks: vec![block.into()].into(),
        local_decls: vec![
            omni_mir::LocalDecl { name: Some("value".to_string()), ty: None },
            omni_mir::LocalDecl { name: Some("result".to_string()), ty: None },
        ].into(),
    };
    
    let function = create_test_function("enum_test", vec![local_value], local_result, body);
    let program = MirProgram {
        tcx: Default::default(),
        functions: vec![function],
        struct_defs: HashMap::new(),
    };
    
    let mut interpreter = Interpreter::new_with_program(program);
    let result = interpreter.execute_function("enum_test", vec![Value::Int(42)]);
    
    assert!(result.is_ok());
    let result_value = result.unwrap();
    
    // Should be an enum variant
    match result_value {
        Value::EnumVariant { enum_name, variant, fields } => {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant, "Some");
            assert_eq!(fields, vec![Value::Int(42)]);
        }
        _ => panic!("Expected enum variant result"),
    }
}