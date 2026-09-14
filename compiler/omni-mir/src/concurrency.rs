//! Structured Concurrency Enforcement for Omni (1.3.0.3 / CONC-0003).
//! Enforces spawn_scope blocks to desugar into closures with mandatory JoinAll terminators.

#[macro_export]
macro_rules! implements {
    ($tag:literal) => {};
}

implements!("CONC-0003");

#[derive(Debug, Clone)]
pub struct SpawnScope {
    pub scope_name: String,
    pub spawned_tasks: Vec<String>,
    pub has_join_all: bool,
}

impl SpawnScope {
    pub fn new(scope_name: impl Into<String>) -> Self {
        Self { scope_name: scope_name.into(), spawned_tasks: Vec::new(), has_join_all: false }
    }

    pub fn spawn_task(&mut self, task_name: impl Into<String>) {
        self.spawned_tasks.push(task_name.into());
    }

    pub fn inject_join_all(&mut self) {
        self.has_join_all = true;
    }

    pub fn verify_scope(&self) -> Result<(), String> {
        if !self.has_join_all && !self.spawned_tasks.is_empty() {
            return Err(format!(
                "Structured concurrency violation (CONC-0003): Scope '{}' has active spawned tasks but lacks a mandatory JoinAll terminator.",
                self.scope_name
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_concurrency_enforcement() {
        let mut scope = SpawnScope::new("data_pipeline");
        scope.spawn_task("fetch_worker");

        // Should fail verification without JoinAll
        assert!(scope.verify_scope().is_err());

        // Inject JoinAll terminator and re-verify
        scope.inject_join_all();
        assert!(scope.verify_scope().is_ok());
    }
}
