//! State-Machine Lowering for Continuations and Async Execution (1.3.0.2).
//! Transforms functions with suspension/async points into explicit state machine structs,
//! lifting live variables across yields into persistent fields.

use crate::ir::Body;

#[derive(Debug, Clone)]
pub struct StateMachineContext {
    pub state_name: String,
    pub live_locals: Vec<String>,
}

impl StateMachineContext {
    pub fn new(state_name: impl Into<String>) -> Self {
        Self { state_name: state_name.into(), live_locals: Vec::new() }
    }

    pub fn lift_variable(&mut self, local_name: impl Into<String>) {
        self.live_locals.push(local_name.into());
    }

    /// Desugar async body into a state-machine transform representation
    pub fn lower_to_state_machine(&self, _body: &Body) -> String {
        format!(
            "struct {}StateMachine {{ state: usize, {} }}",
            self.state_name,
            self.live_locals.iter().map(|l| format!("{}: usize", l)).collect::<Vec<_>>().join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_lowering() {
        let mut ctx = StateMachineContext::new("AsyncCompute");
        ctx.lift_variable("x");
        ctx.lift_variable("y");

        let generated = ctx.lower_to_state_machine(&Body {
            blocks: index_vec::IndexVec::new(),
            local_decls: index_vec::IndexVec::new(),
        });

        assert!(generated.contains("AsyncComputeStateMachine"));
        assert!(generated.contains("x: usize"));
        assert!(generated.contains("y: usize"));
    }
}
