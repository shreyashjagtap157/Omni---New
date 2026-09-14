use crate::token::TokenKind;

/// Tracks indentation levels and emits structural layout tokens.
#[derive(Debug, Clone)]
pub struct LayoutEngine {
    /// The stack of indentation levels. Always contains at least one element (0).
    stack: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// Emitted when an un-indent does not align with any previously established indentation level.
    MismatchedUnindent(u32),
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            // Base indentation level is always 0
            stack: vec![0],
        }
    }

    /// Processes the number of spaces after a newline and returns the resulting layout tokens.
    /// Traps and returns a `LayoutError` if the un-indent is mismatched.
    pub fn process_indent(&mut self, spaces: u32) -> Result<Vec<TokenKind>, LayoutError> {
        let mut tokens = Vec::new();
        let current = *self.stack.last().expect("Indent stack must never be empty");

        if spaces > current {
            // Deeper indentation: push onto stack, emit Indent
            self.stack.push(spaces);
            tokens.push(TokenKind::Indent);
        } else if spaces < current {
            // Shallower indentation: pop stack and emit Dedent until aligned
            while let Some(&top) = self.stack.last() {
                if spaces == top {
                    break; // Successfully aligned
                }
                
                if spaces > top {
                    // We popped past the target without finding an exact match
                    return Err(LayoutError::MismatchedUnindent(spaces));
                }
                
                self.stack.pop();
                tokens.push(TokenKind::Dedent);
            }
        }

        Ok(tokens)
    }

    /// Called at EOF to drain any remaining indentation levels.
    pub fn finish(&mut self) -> Vec<TokenKind> {
        let mut tokens = Vec::new();
        // Pop until only the base 0 remains
        while self.stack.len() > 1 {
            self.stack.pop();
            tokens.push(TokenKind::Dedent);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn test_indent_dedent_sequence() {
        let mut layout = LayoutEngine::new();

        // Step in
        assert_eq!(layout.process_indent(4).unwrap(), vec![TokenKind::Indent]);
        assert_eq!(layout.process_indent(8).unwrap(), vec![TokenKind::Indent]);
        
        // Step out to 4
        assert_eq!(layout.process_indent(4).unwrap(), vec![TokenKind::Dedent]);
        
        // Step out to base (0)
        assert_eq!(layout.process_indent(0).unwrap(), vec![TokenKind::Dedent]);
    }

    #[test]
    fn test_multiple_dedent_in_one_step() {
        let mut layout = LayoutEngine::new();
        layout.process_indent(4).unwrap();
        layout.process_indent(8).unwrap();
        layout.process_indent(12).unwrap();

        // Jump all the way back to base (0), should emit 3 Dedents
        let tokens = layout.process_indent(0).unwrap();
        assert_eq!(tokens, vec![TokenKind::Dedent, TokenKind::Dedent, TokenKind::Dedent]);
    }

    #[test]
    fn test_mismatched_unindent_trap() {
        let mut layout = LayoutEngine::new();
        layout.process_indent(4).unwrap();
        layout.process_indent(8).unwrap();

        // Try to unindent to 6, which isn't in the stack [0, 4, 8]
        let result = layout.process_indent(6);
        assert_eq!(result, Err(LayoutError::MismatchedUnindent(6)));
    }

    #[test]
    fn test_finish_drain() {
        let mut layout = LayoutEngine::new();
        layout.process_indent(2).unwrap();
        layout.process_indent(4).unwrap();

        // Reaching EOF should drain the remaining 2 levels
        assert_eq!(layout.finish(), vec![TokenKind::Dedent, TokenKind::Dedent]);
        
        // Calling finish again should be empty
        assert_eq!(layout.finish(), vec![]);
    }
}