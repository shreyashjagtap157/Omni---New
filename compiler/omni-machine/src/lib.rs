//! Abstract Machine Interpreter for Omni.
//!
//! This crate provides the reference abstract machine for Omni that executes MIR (Mid-Level IR)
//! with proper operand evaluation, statement interpretation, and control flow.

pub mod bridge;
pub mod interpreter;

pub use interpreter::{
    Interpreter, Value, ControlFlow, ExecutionError, Diagnostic, 
    DiagnosticLevel, SourceLocation, CallFrame, PlaceValue
};

#[cfg(test)]
mod tests;
