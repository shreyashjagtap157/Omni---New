use omni_lex::TokenKind;
use crate::expr::ExprParser;

/// Recovers the parser state after a syntax error by advancing the token stream
/// until it encounters a safe synchronization boundary.
pub fn panic_recover<P: ExprParser>(p: &mut P, sync_tokens: &[TokenKind]) {
    while let Some(kind) = p.peek() {
        if sync_tokens.contains(&kind) {
            // We found a synchronization point. Stop consuming.
            break;
        }
        
        // Consume the malformed token. The event machine will automatically
        // attach these to the currently open ErrorNode in the CST.
        p.advance();
    }
}
