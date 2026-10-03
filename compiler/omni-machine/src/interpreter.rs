//! Abstract Machine Interpreter implementation.
//!
//! This is the reference abstract machine for Omni that executes MIR (Mid-Level IR)
//! with proper operand evaluation, statement interpretation, and control flow.

use omni_mir::ast::Lit;
use omni_mir::ir::*;
use omni_mir::Ty;
use omni_mir::{CapabilityContext, Effect, EffectRow};
use std::collections::HashMap;
use std::rc::Rc;

/// The main abstract machine interpreter that executes MIR programs.
pub struct Interpreter {
    /// The MIR program being executed
    program: MirProgram,
    /// Current function being executed
    current_function: Option<Rc<MirFunction>>,
    /// Current basic block being executed
    current_block: Option<BasicBlock>,
    /// Local variable storage (indexed by Local)
    locals: HashMap<Local, Value>,
    /// Memory allocation tracking
    memory: Memory,
    /// Effect tracking for the current execution context
    effects: EffectRow,
    /// Capability context for the current execution
    capabilities: CapabilityContext,
    /// Call stack for tracking function calls
    call_stack: Vec<CallFrame>,
    /// Error diagnostics
    diagnostics: Vec<Diagnostic>,
}

/// A value in the abstract machine
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Character value
    Char(char),
    /// Byte value
    Byte(u8),
    /// String value
    String(String),
    /// Unit value
    Unit,
    /// Tuple value
    Tuple(Vec<Value>),
    /// Array value
    Array(Vec<Value>),
    /// Reference to a place in memory
    Reference(PlaceValue),
    /// Function reference
    FunctionRef(String),
    /// Enum variant
    EnumVariant { enum_name: String, variant: String, fields: Vec<Value> },
    /// Struct value
    Struct { name: String, fields: HashMap<String, Value> },
    /// Uninitialized value (for tracking definite initialization)
    Uninit,
}

/// A place value that includes allocation information
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceValue {
    pub alloc_id: u64,
    pub offset: usize,
    pub size: usize,
}

/// A call frame tracks function execution state
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub return_place: Option<Place>,
    pub return_block: BasicBlock,
    pub caller_locals: HashMap<Local, Value>,
}

/// Diagnostic information for errors and warnings
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub block: BasicBlock,
    pub statement_index: Option<usize>,
}

/// Execution errors
#[derive(Debug, Clone)]
pub enum ExecutionError {
    /// Invalid operand evaluation
    InvalidOperand { operand: Operand, message: String },
    /// Invalid rvalue evaluation
    InvalidRvalue { rvalue: Rvalue, message: String },
    /// Control flow error
    ControlFlowError { message: String },
    /// Bounds check failure
    BoundsCheckFailed { index: i64, length: usize },
    /// Type mismatch
    TypeMismatch { expected: String, actual: String },
    /// Division by zero
    DivisionByZero,
    /// Invalid function call
    InvalidFunctionCall { function: String, message: String },
    /// Memory access error
    MemoryAccessError { message: String },
    /// Effect violation
    EffectViolation { effect: Effect, function: String },
    /// Capability violation
    CapabilityViolation { capability: String, function: String },
    /// Unreachable code
    UnreachableCode,
    /// Invalid projection
    InvalidProjection { place: Place, message: String },
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::InvalidOperand { operand, message } => {
                write!(f, "Invalid operand {:?}: {}", operand, message)
            }
            ExecutionError::InvalidRvalue { rvalue, message } => {
                write!(f, "Invalid rvalue {:?}: {}", rvalue, message)
            }
            ExecutionError::ControlFlowError { message } => {
                write!(f, "Control flow error: {}", message)
            }
            ExecutionError::BoundsCheckFailed { index, length } => {
                write!(f, "Bounds check failed: index {} >= length {}", index, length)
            }
            ExecutionError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            ExecutionError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            ExecutionError::InvalidFunctionCall { function, message } => {
                write!(f, "Invalid function call to {}: {}", function, message)
            }
            ExecutionError::MemoryAccessError { message } => {
                write!(f, "Memory access error: {}", message)
            }
            ExecutionError::EffectViolation { effect, function } => {
                write!(f, "Effect violation: {} not allowed in function {}", effect, function)
            }
            ExecutionError::CapabilityViolation { capability, function } => {
                write!(
                    f,
                    "Capability violation: {} not available in function {}",
                    capability, function
                )
            }
            ExecutionError::UnreachableCode => {
                write!(f, "Reached unreachable code")
            }
            ExecutionError::InvalidProjection { place, message } => {
                write!(f, "Invalid projection for place {}: {}", place, message)
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new interpreter with the given MIR program
    pub fn new_with_program(program: MirProgram) -> Self {
        Self {
            program,
            current_function: None,
            current_block: None,
            locals: HashMap::new(),
            memory: Memory::new(),
            effects: EffectRow::pure(),
            capabilities: CapabilityContext::new(),
            call_stack: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Create a new empty interpreter
    pub fn new() -> Self {
        Self::new_with_program(MirProgram {
            tcx: Default::default(),
            functions: Vec::new(),
            struct_defs: HashMap::new(),
        })
    }

    /// Execute a MIR function by name
    pub fn execute_function(
        &mut self,
        function_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, ExecutionError> {
        // Find the function in the program
        let function =
            self.program.functions.iter().find(|f| f.name == function_name).ok_or_else(|| {
                ExecutionError::InvalidFunctionCall {
                    function: function_name.to_string(),
                    message: "Function not found".to_string(),
                }
            })?;

        // Create a copy of the function for execution
        let function_rc = Rc::new(function.clone());
        self.current_function = Some(function_rc.clone());

        // Initialize locals with arguments
        self.locals.clear();
        for (i, arg) in args.into_iter().enumerate() {
            let local =
                function.params.get(i).ok_or_else(|| ExecutionError::InvalidFunctionCall {
                    function: function_name.to_string(),
                    message: format!(
                        "Argument count mismatch: expected {}, got {}",
                        function.params.len(),
                        i + 1
                    ),
                })?;
            self.locals.insert(*local, arg);
        }

        // Initialize return place
        self.locals.insert(function.return_place, Value::Uninit);

        // Start execution from the first basic block
        if function.body.blocks.is_empty() {
            return Err(ExecutionError::ControlFlowError {
                message: "Function has no basic blocks".to_string(),
            });
        }

        self.current_block = Some(BasicBlock::from(0));

        // Execute the function
        let result = self.execute_function_body(function_rc);

        // Clean up
        self.current_function = None;
        self.current_block = None;
        self.locals.clear();

        result
    }

    /// Execute the body of a function
    fn execute_function_body(
        &mut self,
        function: Rc<MirFunction>,
    ) -> Result<Value, ExecutionError> {
        let mut current_block = self.current_block.ok_or_else(|| {
            ExecutionError::ControlFlowError { message: "No current block".to_string() }
        })?;

        loop {
            let block_data = &function.body.blocks[current_block];

            // Execute statements in the block
            for (stmt_index, statement) in block_data.statements.iter().enumerate() {
                self.execute_statement(statement, function.clone(), current_block, stmt_index)?;
            }

            // Execute terminator
            let terminator = block_data.terminator.as_ref().ok_or_else(|| {
                ExecutionError::ControlFlowError { message: "Block missing terminator".to_string() }
            })?;

            match self.execute_terminator(terminator, function.clone(), current_block)? {
                ControlFlow::Next(next_block) => {
                    current_block = next_block;
                    self.current_block = Some(next_block);
                }
                ControlFlow::Return(value) => {
                    return Ok(value);
                }
                ControlFlow::Unreachable => {
                    return Err(ExecutionError::UnreachableCode);
                }
            }
        }
    }

    /// Execute a statement
    fn execute_statement(
        &mut self,
        statement: &Statement,
        function: Rc<MirFunction>,
        _block: BasicBlock,
        _stmt_index: usize,
    ) -> Result<(), ExecutionError> {
        match statement {
            Statement::Assign(place, rvalue) => {
                let value = self.evaluate_rvalue(rvalue, function.clone())?;
                self.assign_place(place, value)?;
            }
            Statement::Drop(place) => {
                self.execute_drop(place, function.clone())?;
            }
            Statement::BoundsCheck { index, length } => {
                self.execute_bounds_check(*index, *length, function.clone())?;
            }
            Statement::Assume(assumption) => {
                self.execute_assumption(assumption, function.clone())?;
            }
        }
        Ok(())
    }

    /// Execute a terminator
    fn execute_terminator(
        &mut self,
        terminator: &Terminator,
        function: Rc<MirFunction>,
        _block: BasicBlock,
    ) -> Result<ControlFlow, ExecutionError> {
        match terminator {
            Terminator::Goto(target) => Ok(ControlFlow::Next(*target)),
            Terminator::SwitchInt { discr, targets, otherwise } => {
                let value = self.evaluate_operand(discr, function.clone())?;
                let discriminant = match value {
                    Value::Int(i) => i as u64,
                    Value::Bool(b) => b as u64,
                    _ => {
                        return Err(ExecutionError::InvalidOperand {
                            operand: discr.clone(),
                            message: "Switch discriminant must be integer or boolean".to_string(),
                        })
                    }
                };

                // Find the matching target
                for (val, target) in targets {
                    if *val == discriminant {
                        return Ok(ControlFlow::Next(*target));
                    }
                }

                // Otherwise go to the default target
                Ok(ControlFlow::Next(*otherwise))
            }
            Terminator::Call { func, args, destination, target, cleanup: _ } => {
                let func_value = self.evaluate_operand(func, function.clone())?;
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| self.evaluate_operand(arg, function.clone()))
                    .collect::<Result<_, _>>()?;

                let result = self.execute_call(&func_value, &arg_values, function.clone())?;

                // Store result in destination if provided
                if let Some(place) = destination {
                    self.assign_place(place, result)?;
                }

                // Go to the target block
                Ok(ControlFlow::Next(*target))
            }
            Terminator::Return => {
                // Read the return value through the same place path as any
                // other read, so returning without producing a value is
                // reported instead of yielding `Uninit`.
                let return_place = function.return_place;
                let return_value =
                    self.get_place_value(&Place::local(return_place), function.clone())?;

                Ok(ControlFlow::Return(return_value))
            }
            Terminator::Unreachable => Ok(ControlFlow::Unreachable),
        }
    }

    /// Evaluate an operand to a value
    fn evaluate_operand(
        &mut self,
        operand: &Operand,
        function: Rc<MirFunction>,
    ) -> Result<Value, ExecutionError> {
        match operand {
            Operand::Copy(place) => self.get_place_value(place, function),
            Operand::Move(place) => {
                let value = self.get_place_value(place, function)?;
                // Mark the place as moved (invalidated)
                self.invalidate_place(place);
                Ok(value)
            }
            Operand::Constant(constant) => self.evaluate_constant(constant),
        }
    }

    /// Evaluate a constant to a value
    fn evaluate_constant(&self, constant: &Constant) -> Result<Value, ExecutionError> {
        match constant {
            Constant::Lit(lit) => match lit {
                Lit::Int(i) => Ok(Value::Int(*i)),
                Lit::Float(f) => Ok(Value::Float(*f as f64)),
                Lit::Bool(b) => Ok(Value::Bool(*b)),
                Lit::Char(c) => Ok(Value::Char(*c)),
                Lit::Byte(b) => Ok(Value::Byte(*b)),
                Lit::String(s) => Ok(Value::String(s.clone())),
            },
            Constant::FnRef(name) => Ok(Value::FunctionRef(name.clone())),
        }
    }

    /// Evaluate an rvalue to a value
    fn evaluate_rvalue(
        &mut self,
        rvalue: &Rvalue,
        function: Rc<MirFunction>,
    ) -> Result<Value, ExecutionError> {
        match rvalue {
            Rvalue::Use(operand) => self.evaluate_operand(operand, function),
            Rvalue::BinaryOp(op, left, right) => {
                let left_val = self.evaluate_operand(left, function.clone())?;
                let right_val = self.evaluate_operand(right, function)?;
                self.execute_binary_op(*op, left_val, right_val)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.evaluate_operand(operand, function)?;
                self.execute_unary_op(*op, val)
            }
            Rvalue::Cast { operand, from, to } => {
                let val = self.evaluate_operand(operand, function)?;
                self.execute_cast(val, from, to)
            }
            Rvalue::Aggregate { kind, operands, ty } => {
                let values: Vec<Value> = operands
                    .iter()
                    .map(|op| self.evaluate_operand(op, function.clone()))
                    .collect::<Result<_, _>>()?;
                self.execute_aggregate(*kind, values, ty)
            }
            Rvalue::Struct { name, fields, ty: _ } => {
                let mut field_map = HashMap::new();
                for (field_name, operand) in fields {
                    let value = self.evaluate_operand(operand, function.clone())?;
                    field_map.insert(field_name.clone(), value);
                }
                Ok(Value::Struct { name: name.clone(), fields: field_map })
            }
            Rvalue::EnumVariant { enum_name, variant, operands, ty: _ } => {
                let values: Vec<Value> = operands
                    .iter()
                    .map(|op| self.evaluate_operand(op, function.clone()))
                    .collect::<Result<_, _>>()?;
                Ok(Value::EnumVariant {
                    enum_name: enum_name.clone(),
                    variant: variant.clone(),
                    fields: values,
                })
            }
            Rvalue::Range { start, end, inclusive, ty } => {
                let start_val = self.evaluate_operand(start, function.clone())?;
                let end_val = self.evaluate_operand(end, function)?;
                self.execute_range(start_val, end_val, *inclusive, ty)
            }
            Rvalue::Field { base, field, ty: _ } => {
                let base_val = self.evaluate_operand(base, function.clone())?;
                self.execute_field_projection(base_val, field)
            }
            Rvalue::Index { base, index, ty: _ } => {
                let base_val = self.evaluate_operand(base, function.clone())?;
                let index_val = self.evaluate_operand(index, function)?;
                self.execute_index_projection(base_val, index_val)
            }
        }
    }

    /// Execute a binary operation
    fn execute_binary_op(
        &self,
        op: BinOp,
        left: Value,
        right: Value,
    ) -> Result<Value, ExecutionError> {
        match (op, left, right) {
            (BinOp::Add, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
            (BinOp::Sub, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
            (BinOp::Mul, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
            (BinOp::Div, Value::Int(l), Value::Int(r)) => {
                if r == 0 {
                    Err(ExecutionError::DivisionByZero)
                } else {
                    Ok(Value::Int(l / r))
                }
            }
            (BinOp::Rem, Value::Int(l), Value::Int(r)) => {
                if r == 0 {
                    Err(ExecutionError::DivisionByZero)
                } else {
                    Ok(Value::Int(l % r))
                }
            }
            (BinOp::BitAnd, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l & r)),
            (BinOp::BitOr, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l | r)),
            (BinOp::BitXor, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l ^ r)),
            (BinOp::Shl, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l << r)),
            (BinOp::Shr, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l >> r)),
            (BinOp::Eq, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l == r)),
            (BinOp::Ne, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l != r)),
            (BinOp::Lt, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l < r)),
            (BinOp::Le, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l <= r)),
            (BinOp::Gt, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l > r)),
            (BinOp::Ge, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l >= r)),
            (BinOp::Eq, Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l == r)),
            (BinOp::Ne, Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l != r)),
            (BinOp::Eq, Value::String(l), Value::String(r)) => Ok(Value::Bool(l == r)),
            _ => Err(ExecutionError::InvalidRvalue {
                rvalue: Rvalue::BinaryOp(
                    op,
                    Operand::Constant(Constant::Lit(Lit::Int(0))),
                    Operand::Constant(Constant::Lit(Lit::Int(0))),
                ),
                message: "Invalid binary operation operands".to_string(),
            }),
        }
    }

    /// Execute a unary operation
    fn execute_unary_op(&self, op: UnOp, operand: Value) -> Result<Value, ExecutionError> {
        match (op, operand) {
            (UnOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
            (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (UnOp::BitNot, Value::Int(i)) => Ok(Value::Int(!i)),
            _ => Err(ExecutionError::InvalidRvalue {
                rvalue: Rvalue::UnaryOp(op, Operand::Constant(Constant::Lit(Lit::Int(0)))),
                message: "Invalid unary operation operand".to_string(),
            }),
        }
    }

    /// Execute a type cast
    fn execute_cast(&self, value: Value, _from: &Ty, _to: &Ty) -> Result<Value, ExecutionError> {
        // For now, just return the value as-is
        // In a full implementation, this would handle actual type conversions
        Ok(value)
    }

    /// Execute an aggregate construction
    fn execute_aggregate(
        &self,
        kind: AggregateKind,
        values: Vec<Value>,
        _ty: &Ty,
    ) -> Result<Value, ExecutionError> {
        match kind {
            AggregateKind::Tuple => Ok(Value::Tuple(values)),
            AggregateKind::Array => Ok(Value::Array(values)),
        }
    }

    /// Execute a range construction
    fn execute_range(
        &self,
        start: Value,
        end: Value,
        inclusive: bool,
        _ty: &Ty,
    ) -> Result<Value, ExecutionError> {
        // For now, just return a tuple representing the range
        let values = [start, end];
        let range_value = if inclusive {
            Value::String(format!("[{:?}..{:?}]", values[0], values[1]))
        } else {
            Value::String(format!("[{:?}..{:?})", values[0], values[1]))
        };
        Ok(range_value)
    }

    /// Execute a field projection
    ///
    /// The MIR `Rvalue::Field` carries a `Ty` for the projected result, but projection
    /// here is resolved structurally against the runtime `Value` shape, so no type
    /// parameter is required.
    fn execute_field_projection(&self, base: Value, field: &str) -> Result<Value, ExecutionError> {
        match base {
            Value::Struct { name, fields } => {
                fields.get(field).cloned().ok_or_else(|| ExecutionError::InvalidProjection {
                    place: Place::local(Local::from(0)), // Placeholder
                    message: format!("Field '{}' not found in struct '{}'", field, name),
                })
            }
            Value::EnumVariant { enum_name: _, variant, fields } => {
                // For enum variants, we assume the field index corresponds to the operand position
                // This is a simplification; a full implementation would need proper enum layout
                let field_index =
                    field.parse::<usize>().map_err(|_| ExecutionError::InvalidProjection {
                        place: Place::local(Local::from(0)), // Placeholder
                        message: format!(
                            "Invalid field index '{}' in enum variant '{}'",
                            field, variant
                        ),
                    })?;
                fields.get(field_index).cloned().ok_or_else(|| ExecutionError::InvalidProjection {
                    place: Place::local(Local::from(0)), // Placeholder
                    message: format!(
                        "Field index {} out of range in enum variant '{}'",
                        field_index, variant
                    ),
                })
            }
            _ => Err(ExecutionError::InvalidProjection {
                place: Place::local(Local::from(0)), // Placeholder
                message: format!("Cannot project field '{}' from non-struct value", field),
            }),
        }
    }

    /// Execute an index projection
    ///
    /// Index and constant-index projections are bounds-checked against the runtime
    /// container, not against a static type, so no type parameter is required.
    ///
    /// The MIR index is a signed integer. A negative index must be rejected as
    /// out of range in its own right rather than being cast to `usize` first,
    /// which would wrap to a huge offset and report a nonsensical bound.
    fn execute_index_projection(&self, base: Value, index: Value) -> Result<Value, ExecutionError> {
        match (base, index) {
            (Value::Tuple(elements), Value::Int(idx)) => {
                let len = elements.len();
                let slot = usize::try_from(idx).ok().and_then(|i| elements.get(i));
                slot.cloned().ok_or_else(|| ExecutionError::InvalidProjection {
                    place: Place::local(Local::from(0)), // Placeholder
                    message: format!("Tuple index {} out of range for length {}", idx, len),
                })
            }
            (Value::Array(elements), Value::Int(idx)) => {
                let len = elements.len();
                let slot = usize::try_from(idx).ok().and_then(|i| elements.get(i));
                slot.cloned().ok_or_else(|| ExecutionError::InvalidProjection {
                    place: Place::local(Local::from(0)), // Placeholder
                    message: format!("Array index {} out of range for length {}", idx, len),
                })
            }
            (Value::String(s), Value::Int(idx)) => {
                let len = s.chars().count();
                let ch = usize::try_from(idx).ok().and_then(|i| s.chars().nth(i));
                ch.map(Value::Char).ok_or_else(|| ExecutionError::InvalidProjection {
                    place: Place::local(Local::from(0)), // Placeholder
                    message: format!("String index {} out of range for length {}", idx, len),
                })
            }
            _ => Err(ExecutionError::InvalidProjection {
                place: Place::local(Local::from(0)), // Placeholder
                message: "Cannot index non-array/tuple/string value".to_string(),
            }),
        }
    }

    /// Get the value at a place
    fn get_place_value(
        &self,
        place: &Place,
        function: Rc<MirFunction>,
    ) -> Result<Value, ExecutionError> {
        // Start with the root local
        let mut current_value = self
            .locals
            .get(&place.local)
            .ok_or_else(|| ExecutionError::MemoryAccessError {
                message: format!("Local {:?} not found", place.local),
            })?
            .clone();

        // Applying each projection in order
        for projection in &place.projections {
            current_value = self.apply_projection(current_value, projection, function.clone())?;
        }

        // A place that was dropped, moved out of, or never initialized still
        // holds `Uninit`. Reading it is a use of a dead place, so it must be
        // reported rather than propagating a sentinel value into the result.
        if current_value == Value::Uninit {
            return Err(ExecutionError::MemoryAccessError {
                message: format!("read of uninitialized place {}", place),
            });
        }

        Ok(current_value)
    }

    /// Assign a value to a place
    fn assign_place(&mut self, place: &Place, value: Value) -> Result<(), ExecutionError> {
        // For now, just assign to the local directly
        // In a full implementation, this would handle projections and memory references
        self.locals.insert(place.local, value);
        Ok(())
    }

    /// Apply a projection to a value
    fn apply_projection(
        &self,
        base: Value,
        projection: &Projection,
        function: Rc<MirFunction>,
    ) -> Result<Value, ExecutionError> {
        match projection {
            Projection::Field(field_name) => {
                let _ = &function;
                self.execute_field_projection(base, field_name)
            }
            Projection::ConstantIndex(index) => {
                self.execute_constant_index_projection(base, *index)
            }
            Projection::Index(index_local) => {
                let index_value = self
                    .locals
                    .get(index_local)
                    .ok_or_else(|| ExecutionError::MemoryAccessError {
                        message: format!("Index local {:?} not found", index_local),
                    })?
                    .clone();
                self.execute_index_projection(base, index_value)
            }
            Projection::Deref => {
                match base {
                    Value::Reference(_place_value) => {
                        // Dereference the place value
                        // For now, return a placeholder value
                        // In a full implementation, this would read from memory
                        Ok(Value::Int(0)) // Placeholder
                    }
                    _ => Err(ExecutionError::InvalidProjection {
                        place: Place::local(Local::from(0)), // Placeholder
                        message: "Cannot dereference non-reference value".to_string(),
                    }),
                }
            }
        }
    }

    /// Execute a constant index projection
    fn execute_constant_index_projection(
        &self,
        base: Value,
        index: usize,
    ) -> Result<Value, ExecutionError> {
        match base {
            Value::Tuple(elements) => {
                elements.get(index).cloned().ok_or_else(|| ExecutionError::InvalidProjection {
                    place: Place::local(Local::from(0)), // Placeholder
                    message: format!("Tuple index {} out of range", index),
                })
            }
            Value::Array(elements) => {
                elements.get(index).cloned().ok_or_else(|| ExecutionError::InvalidProjection {
                    place: Place::local(Local::from(0)), // Placeholder
                    message: format!("Array index {} out of range", index),
                })
            }
            Value::String(s) => {
                s.chars().nth(index).map(Value::Char).ok_or_else(|| {
                    ExecutionError::InvalidProjection {
                        place: Place::local(Local::from(0)), // Placeholder
                        message: format!("String index {} out of range", index),
                    }
                })
            }
            _ => Err(ExecutionError::InvalidProjection {
                place: Place::local(Local::from(0)), // Placeholder
                message: format!(
                    "Cannot apply constant index to non-array/tuple/string value: {:?}",
                    base
                ),
            }),
        }
    }

    /// Invalidate a place (e.g., after a move)
    fn invalidate_place(&mut self, place: &Place) {
        self.locals.insert(place.local, Value::Uninit);
    }

    /// Execute an assumption
    fn execute_assumption(
        &mut self,
        _assumption: &Assumption,
        _function: Rc<MirFunction>,
    ) -> Result<(), ExecutionError> {
        // `Assumption` currently carries no payload. Recording that one was
        // reached keeps the statement observable in the diagnostic log instead
        // of being absorbed by a silent catch-all.
        self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Note,
            message: "Verifier assumption encountered".to_string(),
            location: Some(SourceLocation {
                block: self.current_block.unwrap_or(BasicBlock::from(0)),
                statement_index: None,
            }),
        });
        Ok(())
    }

    /// Execute a drop operation
    fn execute_drop(
        &mut self,
        place: &Place,
        function: Rc<MirFunction>,
    ) -> Result<(), ExecutionError> {
        // Get the value to determine if it needs special cleanup
        let value = self.get_place_value(place, function.clone())?;

        match value {
            Value::Array(elements) => {
                // For arrays, we might need to drop each element
                for _ in elements {
                    // In a full implementation, this would recursively drop elements
                }
            }
            Value::Tuple(elements) => {
                // For tuples, we might need to drop each element
                for _ in elements {
                    // In a full implementation, this would recursively drop elements
                }
            }
            Value::Struct { fields, .. } => {
                // For structs, we might need to drop each field
                for _ in fields.values() {
                    // In a full implementation, this would recursively drop fields
                }
            }
            Value::EnumVariant { fields, .. } => {
                // For enum variants, we might need to drop each field
                for _ in fields {
                    // In a full implementation, this would recursively drop fields
                }
            }
            Value::Reference(_) => {
                // For references, we might need to deallocate memory
                // In a full implementation, this would handle reference counting or unique ownership
            }
            _ => {
                // For simple types, just invalidate
            }
        }

        // Invalidate the place
        self.invalidate_place(place);
        Ok(())
    }

    /// Execute a bounds check
    fn execute_bounds_check(
        &mut self,
        index_local: Local,
        length: usize,
        _function: Rc<MirFunction>,
    ) -> Result<(), ExecutionError> {
        // Copy the index out before any fallible handling so that reporting a
        // diagnostic never borrows `self` while a `self.locals` borrow is live.
        let index = match self.locals.get(&index_local).cloned() {
            Some(index) => index,
            None => {
                self.add_diagnostic(
                    DiagnosticLevel::Error,
                    format!("Index local {:?} not found for bounds check", index_local),
                    Some(SourceLocation {
                        block: self.current_block.unwrap_or(BasicBlock::from(0)),
                        statement_index: None,
                    }),
                );
                return Err(ExecutionError::BoundsCheckFailed { index: -1, length });
            }
        };

        let index_val = match index {
            Value::Int(i) => i,
            _ => {
                self.add_diagnostic(
                    DiagnosticLevel::Error,
                    format!("Index must be integer, got {:?}", index),
                    Some(SourceLocation {
                        block: self.current_block.unwrap_or(BasicBlock::from(0)),
                        statement_index: None,
                    }),
                );
                return Err(ExecutionError::BoundsCheckFailed { index: -1, length });
            }
        };

        if index_val < 0 {
            self.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Negative index {} in bounds check", index_val),
                Some(SourceLocation {
                    block: self.current_block.unwrap_or(BasicBlock::from(0)),
                    statement_index: None,
                }),
            );
            return Err(ExecutionError::BoundsCheckFailed { index: index_val, length });
        }

        if index_val as usize >= length {
            self.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Index {} out of bounds for length {}", index_val, length),
                Some(SourceLocation {
                    block: self.current_block.unwrap_or(BasicBlock::from(0)),
                    statement_index: None,
                }),
            );
            return Err(ExecutionError::BoundsCheckFailed { index: index_val, length });
        }

        // Add diagnostic for successful bounds check
        self.add_diagnostic(
            DiagnosticLevel::Note,
            format!("Bounds check passed: index {} < length {}", index_val, length),
            Some(SourceLocation {
                block: self.current_block.unwrap_or(BasicBlock::from(0)),
                statement_index: None,
            }),
        );

        Ok(())
    }

    /// Execute a function call
    fn execute_call(
        &mut self,
        func: &Value,
        args: &[Value],
        caller_function: Rc<MirFunction>,
    ) -> Result<Value, ExecutionError> {
        match func {
            Value::FunctionRef(name) => {
                // Handle built-in functions
                match name.as_str() {
                    "print" => {
                        // Print function - just return unit for now
                        for arg in args {
                            println!("{}", self.value_to_string(arg));
                        }
                        Ok(Value::Unit)
                    }
                    "assert" => {
                        // Assert function
                        if let Some(Value::Bool(condition)) = args.first() {
                            if !condition {
                                self.add_diagnostic(
                                    DiagnosticLevel::Error,
                                    "Assertion failed".to_string(),
                                    Some(SourceLocation {
                                        block: self.current_block.unwrap_or(BasicBlock::from(0)),
                                        statement_index: None,
                                    }),
                                );
                                return Err(ExecutionError::ControlFlowError {
                                    message: "Assertion failed".to_string(),
                                });
                            }
                            Ok(Value::Unit)
                        } else {
                            Err(ExecutionError::InvalidFunctionCall {
                                function: name.clone(),
                                message: "Assert requires a boolean condition".to_string(),
                            })
                        }
                    }
                    "alloc" => {
                        // Memory allocation function
                        if args.len() == 2 {
                            if let (Value::Int(size), Value::Int(align)) = (&args[0], &args[1]) {
                                let alloc_id =
                                    self.memory.allocate(*size as usize, *align as u32, true);
                                Ok(Value::Int(alloc_id as i64))
                            } else {
                                Err(ExecutionError::InvalidFunctionCall {
                                    function: name.clone(),
                                    message: "alloc requires size and align as integers"
                                        .to_string(),
                                })
                            }
                        } else {
                            Err(ExecutionError::InvalidFunctionCall {
                                function: name.clone(),
                                message: "alloc requires exactly 2 arguments (size, align)"
                                    .to_string(),
                            })
                        }
                    }
                    "dealloc" => {
                        // Memory deallocation function
                        if args.len() == 1 {
                            if let Value::Int(alloc_id) = &args[0] {
                                match self.memory.deallocate(*alloc_id as u64) {
                                    Ok(_) => Ok(Value::Unit),
                                    Err(e) => Err(ExecutionError::MemoryAccessError {
                                        message: format!("dealloc failed: {}", e),
                                    }),
                                }
                            } else {
                                Err(ExecutionError::InvalidFunctionCall {
                                    function: name.clone(),
                                    message: "dealloc requires allocation ID as integer"
                                        .to_string(),
                                })
                            }
                        } else {
                            Err(ExecutionError::InvalidFunctionCall {
                                function: name.clone(),
                                message: "dealloc requires exactly 1 argument (alloc_id)"
                                    .to_string(),
                            })
                        }
                    }
                    _ => {
                        // User-defined function
                        self.execute_function_with_stack(name, args, caller_function)
                    }
                }
            }
            _ => Err(ExecutionError::InvalidFunctionCall {
                function: "unknown".to_string(),
                message: "Cannot call non-function value".to_string(),
            }),
        }
    }

    /// Execute a function with proper call stack management
    fn execute_function_with_stack(
        &mut self,
        name: &str,
        args: &[Value],
        caller_function: Rc<MirFunction>,
    ) -> Result<Value, ExecutionError> {
        // Find the function in the program
        let function = self.program.functions.iter().find(|f| f.name == name).ok_or_else(|| {
            ExecutionError::InvalidFunctionCall {
                function: name.to_string(),
                message: "Function not found".to_string(),
            }
        })?;

        // Create a copy of the function for execution
        let function_rc = Rc::new(function.clone());

        // Push current state to call stack
        let caller_locals = self.locals.clone();
        let return_place = Place::local(function.return_place);

        self.call_stack.push(CallFrame {
            function_name: caller_function.name.clone(),
            return_place: Some(return_place),
            return_block: self.current_block.unwrap_or(BasicBlock::from(0)),
            caller_locals,
        });

        // Initialize locals with arguments
        self.locals.clear();
        for (i, arg) in args.iter().enumerate() {
            let local =
                function.params.get(i).ok_or_else(|| ExecutionError::InvalidFunctionCall {
                    function: name.to_string(),
                    message: format!(
                        "Argument count mismatch: expected {}, got {}",
                        function.params.len(),
                        i + 1
                    ),
                })?;
            self.locals.insert(*local, arg.clone());
        }

        // Initialize return place
        self.locals.insert(function.return_place, Value::Uninit);

        // Start execution from the first basic block
        if function.body.blocks.is_empty() {
            self.call_stack.pop();
            return Err(ExecutionError::ControlFlowError {
                message: "Function has no basic blocks".to_string(),
            });
        }

        self.current_function = Some(function_rc.clone());
        self.current_block = Some(BasicBlock::from(0));

        // Execute the function
        let result = self.execute_function_body(function_rc);

        // Restore caller state
        if let Some(call_frame) = self.call_stack.pop() {
            self.locals = call_frame.caller_locals;
            self.current_block = Some(call_frame.return_block);
        } else {
            self.locals.clear();
            self.current_block = None;
        }

        self.current_function = None;

        result
    }

    /// Convert a value to a string for display
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Char(c) => c.to_string(),
            Value::Byte(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Unit => "()".to_string(),
            Value::Tuple(elements) => {
                format!(
                    "({})",
                    elements.iter().map(|e| self.value_to_string(e)).collect::<Vec<_>>().join(", ")
                )
            }
            Value::Array(elements) => {
                format!(
                    "[{}]",
                    elements.iter().map(|e| self.value_to_string(e)).collect::<Vec<_>>().join(", ")
                )
            }
            Value::Reference(_) => "&ref".to_string(),
            Value::FunctionRef(name) => format!("fn {}", name),
            Value::EnumVariant { enum_name, variant, fields } => {
                format!(
                    "{}::{}({})",
                    enum_name,
                    variant,
                    fields.iter().map(|e| self.value_to_string(e)).collect::<Vec<_>>().join(", ")
                )
            }
            Value::Struct { name, fields } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{} {{ {} }}", name, field_strs.join(", "))
            }
            Value::Uninit => "uninit".to_string(),
        }
    }

    /// Get the current diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Get the effect row accumulated by the current execution context.
    ///
    /// The interpreter starts from `EffectRow::pure()` and grows it as
    /// effectful operations are discharged, so this reports what execution
    /// actually performed rather than what was merely expected.
    pub fn effects(&self) -> &EffectRow {
        &self.effects
    }

    /// Get the capabilities available to the current execution.
    pub fn capabilities(&self) -> &CapabilityContext {
        &self.capabilities
    }

    /// Whether an allocation handle still refers to live memory.
    ///
    /// The `Memory` state itself stays private to the interpreter; this exposes
    /// only the query needed to observe that a builtin `alloc` really produced
    /// storage and that a builtin `dealloc` really released it.
    pub fn allocation_is_live(&self, id: u64) -> bool {
        self.memory.allocation_exists(id)
    }

    /// Clear diagnostics
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    /// Add a diagnostic to the diagnostic list
    fn add_diagnostic(
        &mut self,
        level: DiagnosticLevel,
        message: String,
        location: Option<SourceLocation>,
    ) {
        self.diagnostics.push(Diagnostic { level, message, location });
    }

    /// Get the current diagnostics and clear them
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

/// Control flow from executing a terminator
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    /// Continue execution to the specified basic block
    Next(BasicBlock),
    /// Return from the function with the specified value
    Return(Value),
    /// Unreachable code was encountered
    Unreachable,
}

/// Memory management for the abstract machine
#[derive(Debug, Clone, Default)]
pub struct Memory {
    allocations: HashMap<u64, Allocation>,
    next_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Allocation {
    bytes: Vec<u8>,
    initialized: Vec<bool>,
    align: u32,
    mutable: bool,
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&mut self, size: usize, align: u32, mutable: bool) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let alloc =
            Allocation { bytes: vec![0; size], initialized: vec![false; size], align, mutable };

        self.allocations.insert(id, alloc);
        id
    }

    pub fn read(&self, id: u64, offset: usize, size: usize) -> Result<&[u8], String> {
        let alloc = self
            .allocations
            .get(&id)
            .ok_or("Invalid pointer provenance: dangling allocation ID".to_string())?;
        if offset + size > alloc.bytes.len() {
            return Err("Out of bounds read trap".to_string());
        }
        for i in offset..(offset + size) {
            if !alloc.initialized[i] {
                return Err("Read from uninitialized memory trap".to_string());
            }
        }
        Ok(&alloc.bytes[offset..(offset + size)])
    }

    pub fn write(&mut self, id: u64, offset: usize, data: &[u8]) -> Result<(), String> {
        let alloc = self
            .allocations
            .get_mut(&id)
            .ok_or("Invalid pointer provenance: dangling allocation ID".to_string())?;
        if !alloc.mutable {
            return Err("Write to immutable memory trap".to_string());
        }
        if offset + data.len() > alloc.bytes.len() {
            return Err("Out of bounds write trap".to_string());
        }
        for (i, &byte) in data.iter().enumerate() {
            alloc.bytes[offset + i] = byte;
            alloc.initialized[offset + i] = true;
        }
        Ok(())
    }

    /// Check if an allocation exists
    pub fn allocation_exists(&self, id: u64) -> bool {
        self.allocations.contains_key(&id)
    }

    /// Deallocate an allocation
    pub fn deallocate(&mut self, id: u64) -> Result<(), String> {
        if !self.allocations.contains_key(&id) {
            return Err("Cannot deallocate non-existent allocation".to_string());
        }
        self.allocations.remove(&id);
        Ok(())
    }

    /// Get allocation info
    pub fn get_allocation_info(&self, id: u64) -> Option<(usize, u32, bool)> {
        self.allocations.get(&id).map(|alloc| (alloc.bytes.len(), alloc.align, alloc.mutable))
    }

    /// Check if memory is initialized at a specific location
    pub fn is_initialized(&self, id: u64, offset: usize) -> Result<bool, String> {
        let alloc = self.allocations.get(&id).ok_or("Invalid pointer provenance".to_string())?;
        if offset >= alloc.bytes.len() {
            return Err("Offset out of bounds".to_string());
        }
        Ok(alloc.initialized[offset])
    }
}
