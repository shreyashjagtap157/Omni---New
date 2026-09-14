use omni_lex::TokenKind;
use omni_syntax::SyntaxKind;


/// Minimal parser interface required to drive the Pratt expression parser.
pub trait ExprParser {
    fn peek(&self) -> Option<TokenKind>;
    fn advance(&mut self);
    fn start_node(&mut self, kind: SyntaxKind) -> usize;
    fn finish_node(&mut self);
    fn error(&mut self, msg: String);
}

/// Returns the (left, right) binding power for infix operators.
pub fn binding_power(kind: TokenKind) -> (u8, u8) {
    match kind {
        // TODO: Expand when omni-lex::Punct is fully populated (Plus, Star, etc.)
        // For now, any punctuation gets a default binding power.
        TokenKind::Punct(_) => (1, 2),
        _ => (0, 0),
    }
}

/// Parses an expression using Top-Down Operator Precedence (Pratt Parsing).
pub fn parse_expr_bp<P: ExprParser>(p: &mut P, min_bp: u8) {
    let Some(kind) = p.peek() else {
        p.error("Unexpected EOF while parsing expression".into());
        return;
    };

    // 1. Parse prefix / atomic expression (LHS)
    // In a full implementation, we'd emit SyntaxKind::LiteralExpr or similar.
    let _lhs_mark = p.start_node(SyntaxKind::ErrorNode); 
    match kind {
        TokenKind::Ident | TokenKind::Int | TokenKind::Float => {
            p.advance(); // Consume the atomic token
        }
        _ => {
            p.error("Expected expression".into());
            // The panic-mode error recovery (0.2.0.4) will handle un-borking this state later.
        }
    }
    p.finish_node();

    // 2. Parse infix operators (RHS)
    loop {
        let Some(op_kind) = p.peek() else { break };
        
        let (left_bp, right_bp) = binding_power(op_kind);
        if left_bp < min_bp {
            break; // The operator binds weaker than our current context, back out.
        }

        // Consume the operator
        p.advance();

        // Recursively parse the RHS of the infix expression.
        // We'll use the event machine's orward_parent feature in the future 
        // to retroactively wrap the LHS and RHS into an InfixExpr node.
        parse_expr_bp(p, right_bp);
    }
}

#[cfg(any())]
#[implements("GRAM-0005")]
fn _audit_pratt_parser() {}
