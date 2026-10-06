//! Abstract Machine Interpreter for Omni.
//!
//! This crate provides the reference abstract machine for Omni that executes MIR (Mid-Level IR)
//! with proper operand evaluation, statement interpretation, and control flow.

pub mod interpreter;

pub use interpreter::{
    CallFrame, ControlFlow, Diagnostic, DiagnosticLevel, ExecutionError, Interpreter, PlaceValue,
    ReferenceValue, SourceLocation, Value,
};

#[cfg(test)]
mod tests;
