//! Integration bridge connecting the parser AST/HIR to the abstract machine interpreter.

use omni_parse::parser::Parser;
use omni_machine::interpreter::Interpreter;

pub fn execute_source(source_code: &str) -> Result<i64, String> {
    // Stage-0 parser integration stub
    let mut interpreter = Interpreter::new();
    interpreter.run_snippet(source_code)
}
