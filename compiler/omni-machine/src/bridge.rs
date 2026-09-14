//! Integration bridge connecting the parser AST/HIR to the abstract machine interpreter.

use omni_parse::Parser;
use crate::interpreter::Interpreter;

pub fn execute_source(source_code: &str) -> Result<i64, String> {
    let mut parser = Parser::new(vec![]);
    parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    let mut interpreter = Interpreter::new();
    interpreter.run_snippet(source_code)
}
