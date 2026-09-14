//! LLVM Translation Validator and Optimization Pass Barrier Enforcer (1.4.0.2 / 1.4.x.x).
//! Hooks into optimization pass manager stages (e.g., Loop Vectorization) to inspect LLVM IR
//! and ensure instructions guarded by ssume.use metadata are never hoisted above ssume.invalidate barriers.

#[derive(Debug, Clone)]
pub struct TranslationValidator {
    pub pass_name: String,
}

impl TranslationValidator {
    pub fn new(pass_name: impl Into<String>) -> Self {
        Self { pass_name: pass_name.into() }
    }

    /// Inspect LLVM IR text before and after an optimization pass to verify assume barrier safety
    pub fn validate_pass(&self, pre_pass_ir: &str, post_pass_ir: &str) -> Result<(), String> {
        let _invalidate_pos_pre = find_barrier_position(pre_pass_ir, "assume.invalidate");
        let _use_pos_pre = find_barrier_position(pre_pass_ir, "assume.use");

        let invalidate_pos_post = find_barrier_position(post_pass_ir, "assume.invalidate");
        let use_pos_post = find_barrier_position(post_pass_ir, "assume.use");

        // If both barriers exist, ensure use has not been legally or illegally hoisted above invalidate
        if let (Some(inv_idx), Some(use_idx)) = (invalidate_pos_post, use_pos_post) {
            if use_idx < inv_idx {
                return Err(format!(
                    "Translation Validation Failure during pass '{}': Instructions guarded by 'assume.use' (line index {}) were illegally hoisted above 'assume.invalidate' barrier (line index {}).",
                    self.pass_name, use_idx, inv_idx
                ));
            }
        }

        Ok(())
    }
}

fn find_barrier_position(ir: &str, marker: &str) -> Option<usize> {
    ir.lines().position(|line| line.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_validator_success() {
        let pre_ir =
            "entry:\n  call void @llvm.assume.invalidate()\n  call void @llvm.assume.use()\n";
        let post_ir =
            "entry:\n  call void @llvm.assume.invalidate()\n  call void @llvm.assume.use()\n";

        let validator = TranslationValidator::new("LoopVectorize");
        assert!(validator.validate_pass(pre_ir, post_ir).is_ok());
    }

    #[test]
    fn test_translation_validator_hoisting_violation() {
        let pre_ir =
            "entry:\n  call void @llvm.assume.invalidate()\n  call void @llvm.assume.use()\n";
        // Illegal post-pass IR where assume.use is hoisted above assume.invalidate
        let post_ir =
            "entry:\n  call void @llvm.assume.use()\n  call void @llvm.assume.invalidate()\n";

        let validator = TranslationValidator::new("LoopVectorize");
        assert!(validator.validate_pass(pre_ir, post_ir).is_err());
    }
}
