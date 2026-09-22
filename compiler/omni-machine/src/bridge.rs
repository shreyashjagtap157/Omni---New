//! Integration bridge routing execution requests to the machine interpreter.
//!
//! The machine tier consumes lowered representations through its declared MIR
//! boundary (see the crate manifest). It must not couple to frontend syntax:
//! the former parser hook parsed a constant empty input and never influenced
//! execution, so it was removed as layer-skipping scaffolding in 0.0.0.9.

use crate::interpreter::Interpreter;

pub fn execute_source(source_code: &str) -> Result<i64, String> {
    let mut interpreter = Interpreter::new();
    interpreter.run_snippet(source_code)
}
