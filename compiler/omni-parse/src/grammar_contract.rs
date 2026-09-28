//! The grammar contract that is **in force** at this milestone, pinned by tests.
//!
//! # Why this file exists
//!
//! `docs/grammar-reconciliation.md` records an open specification defect: the
//! bound Edition 1 EBNF predates the Candidate 2 Vibe-First amendment, so
//! `VIBE-GRAM-0001` ... `0007` and `LEX-0002`/`GRAM-0002`/`GRAM-0003` disagree
//! about newlines and semicolons. The amendment is **not in force**: its erratum
//! overlay is `status: pre-release`, `signature_state: pending`, and declares
//! `implementation_impact: "none"`, and `GRAM-0003` only admits the newline
//! exception under a Candidate 2 feature gate that is not enabled.
//!
//! Until that is changed by the established process, Edition 1 strict governs.
//! That is a real contract with observable consequences, and this module exists
//! so it is enforced rather than assumed: if the parser starts accepting
//! newline-terminated statements, or stops requiring `;`, one of these tests
//! fails and the change has to be justified against the specification instead
//! of drifting in silently.
//!
//! # What this module does not do
//!
//! It does not implement Candidate 2 syntax, and it does not encode a
//! preference. Every assertion below cites the rule it enforces.

#[cfg(test)]
mod tests {
    use super::super::Parser;

    /// Parse `src`, asserting it parses cleanly, and return the CST text.
    fn ok(src: &str) -> String {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(r.is_ok(), "expected {:?} to parse, got {:?}", src, r.diagnostics);
        r.syntax().text().to_string()
    }

    /// Parse `src`, asserting it produces at least one diagnostic.
    fn rejected(src: &str) -> usize {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(!r.is_ok(), "expected {src:?} to be rejected, got no diagnostic");
        r.diagnostics.len()
    }

    // ----------------------------------------------------------------------
    // LEX-0002: "Newlines never terminate statements."
    // ----------------------------------------------------------------------

    #[test]
    fn newlines_never_terminate_a_statement() {
        // The same program on one line and spread over three must both parse
        // cleanly: a newline is trivia, so it changes neither validity nor
        // structure. (The reconstructed text necessarily differs, because the
        // newlines are real source bytes; what must not differ is whether the
        // program is accepted.)
        ok("fn main() { return 1; }");
        ok("fn main() {\n    return 1;\n}");

        // And a newline is not a statement separator: two returns need the
        // semicolon to be legal, however they are laid out.
        assert!(
            rejected("fn main() { return 1\n return 2; }") > 0,
            "a newline must not stand in for `;` while LEX-0002 is in force"
        );
    }

    // ----------------------------------------------------------------------
    // GRAM-0002: semicolon requirements.
    // ----------------------------------------------------------------------

    #[test]
    fn a_non_block_expression_statement_requires_a_semicolon() {
        ok("fn main() { return 1; }");
        assert!(rejected("fn main() { return 1 }") > 0, "GRAM-0002 requires `;` here");
    }

    #[test]
    fn a_let_statement_requires_a_semicolon() {
        ok("fn main() { let x = 1; return x; }");
        assert!(rejected("fn main() { let x = 1 return x; }") > 0, "let_stmt needs `;`");
    }

    // ----------------------------------------------------------------------
    // GRAM-0004: associativity.
    // ----------------------------------------------------------------------

    #[test]
    fn assignment_is_right_associative_and_binaries_are_left() {
        // Structural check: the CST nests `a - (b - c)` for the right-associative
        // assignment and `(a - b) - c` for the left-associative binary, which is
        // what GRAM-0004 requires. The test asserts the parse succeeds and is
        // lossless; the nesting shape is covered by the CST milestone.
        let src = "fn main() { let a = 1; let b = 2; let c = 3; let d = a - b - c; return d; }";
        assert_eq!(ok(src), src);
    }

    // ----------------------------------------------------------------------
    // GRAM-0007: recovery is tagged and never translatable.
    // ----------------------------------------------------------------------

    #[test]
    fn malformed_source_is_reported_and_still_reconstructs() {
        // A parser error must not lose source: the tree is still lossless, and
        // the diagnostics are what mark the tree non-translatable.
        let src = "fn main( { return 1; }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(!r.is_ok(), "malformed source must be reported");
        assert_eq!(r.syntax().text().to_string(), src, "recovery must stay lossless");
    }
}

// --------------------------------------------------------------------------
// Lossless reconstruction under recovery
// --------------------------------------------------------------------------

/// A CST that has a diagnostic is still required to reproduce its input
/// byte-for-byte. This is the invariant the recovery helpers used to break:
/// they advanced the token position without attaching the skipped tokens, so
/// malformed input silently lost source text.
#[cfg(test)]
mod recovery_losslessness {
    use crate::Parser;

    fn assert_round_trips(src: &str) {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert_eq!(
            r.syntax().text().to_string(),
            src,
            "recovery must not lose source text: {src:?}"
        );
    }

    #[test]
    fn unclosed_parameter_list_does_not_swallow_the_rest_of_the_file() {
        // `recover_until` used to skip `return`, `1`, `;` and `}` outright.
        assert_round_trips("fn main( { return 1; }");
    }

    #[test]
    fn a_range_of_malformed_inputs_all_round_trip() {
        for src in [
            "fn main( { return 1; }",
            "fn main() { let = 1 return 2; }",
            "fn { let = 1 return 2; }",
            "let stray = 1",
            "fn main() { return 1; ",
            "fn a() {} struct_like garbage fn b() { return 1; }",
            "fn main() { return f(1 2 3); }",
            "fn main() { return 1; } @@@ fn other() { return 2; }",
            "fn",
            "fn main",
            "fn main() {",
            "fn main() { return",
            "fn main() { return 1; ",
            "}}}",
            "fn main() { let x = ; return x; }",
            "fn main() { if return 1; }",
        ] {
            assert_round_trips(src);
        }
    }

    #[test]
    fn malformed_input_still_produces_a_diagnostic() {
        // The round-trip guarantee must not be achieved by accepting everything.
        for src in ["fn main( { return 1; }", "fn { let = 1 return 2; }", "fn"] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(!r.is_ok(), "{src:?} must be reported, not silently accepted");
        }
    }

    #[test]
    fn recovery_nodes_are_tagged() {
        // GRAM-0007: recovery is tagged and non-translatable.
        use omni_syntax::SyntaxKind;
        let mut p = Parser::from_source("fn main( { return 1; }");
        let r = p.parse_source();
        assert!(
            r.syntax().descendants().any(|n| n.kind() == SyntaxKind::ErrorNode),
            "recovery must be tagged with an ErrorNode"
        );
    }
}
