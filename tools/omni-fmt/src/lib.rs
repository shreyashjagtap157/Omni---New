//! Omni source code formatter.
//!
//! Parses Omni source into a lossless CST and re-emits it in canonical form:
//! 4-space indentation, one declaration/statement per line, consistent
//! spacing around operators and punctuation, and comment preservation.
//!
//! The formatter is idempotent: formatting already-formatted source produces
//! byte-identical output. It operates on the concrete syntax tree, so every
//! token (including comments and trivia) is preserved verbatim; only the
//! whitespace *between* tokens is re-emitted.

use omni_parse::Parser;
use omni_syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// Canonical indentation: 4 spaces.
const INDENT: &str = "    ";

/// A diagnostic produced by the formatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    pub message: String,
}

/// Result of formatting: either the formatted source or a list of parse errors.
pub type FormatResult = Result<String, Vec<FormatError>>;

/// The formatter state.
pub struct Formatter {
    output: String,
    /// Current indentation level (in units of INDENT).
    indent: usize,
    /// Whether a space should be emitted before the next token.
    pending_space: bool,
    /// Whether a newline should be emitted before the next token.
    pending_newline: bool,
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter {
    pub fn new() -> Self {
        Self { output: String::new(), indent: 0, pending_space: false, pending_newline: false }
    }

    /// Format a source string. Returns the formatted source on success, or
    /// parse diagnostics if the source cannot be parsed.
    pub fn format(&mut self, source: &str) -> FormatResult {
        let mut parser = Parser::from_source(source);
        let result = parser.parse_source();
        if !result.is_ok() {
            return Err(result
                .diagnostics
                .iter()
                .map(|d| FormatError { message: d.message.clone() })
                .collect());
        }
        let syntax = result.syntax();
        self.output.clear();
        self.indent = 0;
        self.pending_space = false;
        self.pending_newline = false;
        self.emit_node(&syntax);
        Ok(self.output.clone())
    }

    /// Convenience: format a string, returning the input unchanged on parse
    /// error (callers can decide how to handle unparseable input).
    pub fn format_or_keep(&mut self, source: &str) -> String {
        match self.format(source) {
            Ok(formatted) => formatted,
            Err(_) => source.to_string(),
        }
    }

    // ── emission helpers ──────────────────────────────────────────────

    fn flush_pending(&mut self) {
        if self.pending_newline {
            self.output.push('\n');
            for _ in 0..self.indent {
                self.output.push_str(INDENT);
            }
            self.pending_newline = false;
            self.pending_space = false;
        } else if self.pending_space && !self.output.ends_with(' ') && !self.output.ends_with('\n')
        {
            self.output.push(' ');
            self.pending_space = false;
        }
    }

    fn space(&mut self) {
        // A pending newline takes precedence over a pending space.
        if !self.pending_newline {
            self.pending_space = true;
        }
    }

    fn newline(&mut self) {
        self.pending_newline = true;
        self.pending_space = false;
    }

    fn indent_in(&mut self) {
        self.indent += 1;
    }

    fn indent_out(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    /// Emit a token's raw text. Indent/Dedent tokens are structural (zero
    /// width) and are not emitted; the formatter manages indentation itself.
    fn emit_token(&mut self, token: &SyntaxToken) {
        match token.kind() {
            SyntaxKind::Indent | SyntaxKind::Dedent => {
                // Structural tokens: do not emit text. Indentation is managed
                // by the formatter based on block structure.
                return;
            }
            SyntaxKind::Whitespace => {
                // Original whitespace is discarded; we emit our own.
                return;
            }
            _ => {}
        }

        let text = token.text();

        // Suppress a preceding space before closing punctuation and statement
        // terminators (e.g. `42;` not `42 ;`, `fn()` not `fn ()`). Also
        // suppress before `.`/`::` for field/method access (e.g. `foo.bar`
        // not `foo . bar`). Do NOT suppress before `->` (space is wanted:
        // `i64 ->` not `i64->`).
        let no_space_before = matches!(text, ")" | "]" | "}" | ";" | "," | "." | "::" | "?");
        // Also suppress space before `(`/`[`/`{` when the preceding output ends
        // in an identifier, literal, or closing punctuation (e.g. `fn(` not
        // `fn (`).
        let suppress_before_open = matches!(text, "(" | "[" | "{")
            && self.output.ends_with(|c: char| {
                c.is_alphanumeric() || c == '_' || c == ')' || c == ']' || c == '}'
            });
        if no_space_before || suppress_before_open {
            self.pending_space = false;
        }

        self.flush_pending();
        self.output.push_str(text);

        // Suppress a following space after opening punctuation and `.`/`::`
        // (e.g. `fn()` not `fn ()`, `foo.bar` not `foo . bar`).
        let no_space_after = matches!(text, "(" | "[" | "{" | "." | "::");
        if !no_space_after {
            self.pending_space = true;
        }
    }

    /// Emit all children of a node, inserting separators between them.
    /// This is the default handler for nodes without special formatting.
    fn emit_children(&mut self, node: &SyntaxNode) {
        let mut first = true;
        for child in node.children_with_tokens() {
            if !first {
                self.space();
            }
            first = false;
            match child {
                SyntaxElement::Node(child_node) => {
                    self.emit_node(&child_node);
                }
                SyntaxElement::Token(token) => {
                    self.emit_token(&token);
                }
            }
        }
    }

    /// Dispatch to the specific formatter for a node's kind.
    fn emit_node(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::SourceFile => self.emit_source_file(node),
            SyntaxKind::FnDef => self.emit_fn_def(node),
            SyntaxKind::FunctionSignature => self.emit_children(node),
            SyntaxKind::Block => self.emit_block(node),
            SyntaxKind::LetStmt => self.emit_let_stmt(node),
            SyntaxKind::ExprStmt => self.emit_expr_stmt(node),
            SyntaxKind::ReturnExpr => self.emit_return_expr(node),
            SyntaxKind::IfExpr => self.emit_if_expr(node),
            SyntaxKind::MatchExpr => self.emit_match_expr(node),
            SyntaxKind::MatchArm => self.emit_match_arm(node),
            SyntaxKind::LoopExpr => self.emit_loop_expr(node),
            SyntaxKind::WhileExpr => self.emit_while_expr(node),
            SyntaxKind::ForExpr => self.emit_for_expr(node),
            SyntaxKind::TryExpr => self.emit_try_expr(node),
            SyntaxKind::CatchClause => self.emit_catch_clause(node),
            SyntaxKind::BreakExpr | SyntaxKind::ContinueExpr | SyntaxKind::YieldExpr => {
                self.emit_children(node)
            }
            SyntaxKind::AwaitExpr => self.emit_await_expr(node),
            SyntaxKind::AssignExpr => self.emit_children(node),
            SyntaxKind::BinaryExpr => self.emit_binary_expr(node),
            SyntaxKind::UnaryExpr => self.emit_unary_expr(node),
            SyntaxKind::CallExpr => self.emit_call_expr(node),
            SyntaxKind::MethodCallExpr => self.emit_method_call_expr(node),
            SyntaxKind::FieldExpr => self.emit_field_expr(node),
            SyntaxKind::IndexExpr => self.emit_index_expr(node),
            SyntaxKind::RangeExpr => self.emit_range_expr(node),
            SyntaxKind::CastExpr => self.emit_cast_expr(node),
            SyntaxKind::PathExpr => self.emit_children(node),
            SyntaxKind::MacroInvocation => self.emit_children(node),
            SyntaxKind::ParenthesizedExpr => self.emit_parenthesized_expr(node),
            SyntaxKind::StructExpr => self.emit_struct_expr(node),
            SyntaxKind::StructExprField => self.emit_children(node),
            SyntaxKind::ArrayExpr => self.emit_array_expr(node),
            SyntaxKind::TupleExpr => self.emit_tuple_expr(node),
            SyntaxKind::ClosureExpr => self.emit_closure_expr(node),
            SyntaxKind::AsyncBlock => self.emit_async_block(node),
            SyntaxKind::UnsafeBlock => self.emit_unsafe_block(node),
            SyntaxKind::TryBlock => self.emit_try_block(node),
            SyntaxKind::LiteralExpr => self.emit_children(node),
            SyntaxKind::StructDef => self.emit_struct_def(node),
            SyntaxKind::StructField => self.emit_children(node),
            SyntaxKind::EnumDef => self.emit_enum_def(node),
            SyntaxKind::EnumVariant => self.emit_children(node),
            SyntaxKind::TraitDef => self.emit_trait_def(node),
            SyntaxKind::TraitItem => self.emit_children(node),
            SyntaxKind::ImplDef => self.emit_impl_def(node),
            SyntaxKind::ImplItem => self.emit_children(node),
            SyntaxKind::TypeAlias => self.emit_type_alias(node),
            SyntaxKind::ConstDef => self.emit_const_def(node),
            SyntaxKind::StaticDef => self.emit_static_def(node),
            SyntaxKind::UseDecl => self.emit_use_decl(node),
            SyntaxKind::UseTree => self.emit_children(node),
            SyntaxKind::ModuleDecl => self.emit_module_decl(node),
            SyntaxKind::ExternCrateDecl => self.emit_children(node),
            SyntaxKind::AttributeItem => self.emit_children(node),
            SyntaxKind::Attribute => self.emit_children(node),
            SyntaxKind::GenericParams => self.emit_generic_params(node),
            SyntaxKind::GenericParam => self.emit_children(node),
            SyntaxKind::TypeParam => self.emit_children(node),
            SyntaxKind::WhereClause => self.emit_where_clause(node),
            SyntaxKind::WherePredicate => self.emit_children(node),
            SyntaxKind::TypeBound => self.emit_children(node),
            SyntaxKind::TraitRef => self.emit_children(node),
            SyntaxKind::TypeArgs => self.emit_type_args(node),
            SyntaxKind::TypeArg => self.emit_children(node),
            SyntaxKind::Path => self.emit_path(node),
            SyntaxKind::PathSegment => self.emit_children(node),
            SyntaxKind::Type => self.emit_children(node),
            SyntaxKind::TupleType => self.emit_children(node),
            SyntaxKind::ArrayType => self.emit_children(node),
            SyntaxKind::SliceType => self.emit_children(node),
            SyntaxKind::ReferenceType => self.emit_children(node),
            SyntaxKind::RawPointerType => self.emit_children(node),
            SyntaxKind::PathType => self.emit_children(node),
            SyntaxKind::NeverType => self.emit_children(node),
            SyntaxKind::ParenthesizedType => self.emit_children(node),
            SyntaxKind::FunctionType => self.emit_children(node),
            SyntaxKind::Visibility => self.emit_children(node),
            SyntaxKind::Lifetime => self.emit_children(node),
            SyntaxKind::ParamList => self.emit_param_list(node),
            SyntaxKind::Param => self.emit_param(node),
            // Patterns.
            SyntaxKind::LiteralPattern
            | SyntaxKind::IdentifierPattern
            | SyntaxKind::WildcardPattern => self.emit_children(node),
            SyntaxKind::TuplePattern => self.emit_children(node),
            SyntaxKind::SlicePattern => self.emit_children(node),
            SyntaxKind::StructPattern => self.emit_children(node),
            SyntaxKind::EnumPattern => self.emit_children(node),
            SyntaxKind::ReferencePattern => self.emit_children(node),
            SyntaxKind::OrPattern => self.emit_children(node),
            SyntaxKind::BindingPattern => self.emit_children(node),
            SyntaxKind::RangePattern => self.emit_children(node),
            SyntaxKind::GuardPattern => self.emit_children(node),
            SyntaxKind::PatternField => self.emit_children(node),
            // Experimental / unratified constructs: emit as-is.
            SyntaxKind::PipelineExpr
            | SyntaxKind::QuestionDotExpr
            | SyntaxKind::PlaceholderExpr
            | SyntaxKind::CommandCallExpr
            | SyntaxKind::Label
            | SyntaxKind::FinalExpr
            | SyntaxKind::ItemDeclStmt
            | SyntaxKind::MacroStmt
            | SyntaxKind::TokenTree
            | SyntaxKind::Ident
            | SyntaxKind::Int
            | SyntaxKind::Float
            | SyntaxKind::Keyword
            | SyntaxKind::Punct
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::DocComment
            | SyntaxKind::ErrorToken
            | SyntaxKind::ErrorNode
            | SyntaxKind::MissingToken
            | SyntaxKind::Unknown => {
                self.emit_children(node);
            }
            _ => {
                self.emit_children(node);
            }
        }
    }

    // ── construct-specific emitters ───────────────────────────────────

    fn emit_source_file(&mut self, node: &SyntaxNode) {
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(child_node) => {
                    if !self.output.is_empty() {
                        self.newline();
                    }
                    self.emit_node(&child_node);
                }
                SyntaxElement::Token(token) => {
                    self.emit_token(&token);
                }
            }
        }
        // Ensure trailing newline.
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn emit_fn_def(&mut self, node: &SyntaxNode) {
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(child_node) => match child_node.kind() {
                    SyntaxKind::Block => {
                        self.space();
                        self.emit_block_with_brace(&child_node);
                    }
                    _ => {
                        self.space();
                        self.emit_node(&child_node);
                    }
                },
                SyntaxElement::Token(token) => {
                    self.emit_token(&token);
                }
            }
        }
        self.newline();
    }

    fn emit_block(&mut self, node: &SyntaxNode) {
        self.emit_block_with_brace(node);
    }

    /// Emit a block: `{` on the same line, indented body, `}` on its own line.
    fn emit_block_with_brace(&mut self, node: &SyntaxNode) {
        let mut first = true;
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Token(token) => {
                    match token.kind() {
                        SyntaxKind::Punct if token.text() == "{" => {
                            self.flush_pending();
                            self.output.push('{');
                            self.pending_space = false;
                            self.newline();
                            self.indent_in();
                        }
                        SyntaxKind::Punct if token.text() == "}" => {
                            // Suppress any pending space before closing brace
                            // (e.g. `return 42;\n}` not `return 42; \n}`).
                            self.pending_space = false;
                            self.flush_pending();
                            self.indent_out();
                            self.newline();
                            self.flush_pending();
                            self.output.push('}');
                            self.pending_space = true;
                        }
                        _ => {
                            self.emit_token(&token);
                        }
                    }
                }
                SyntaxElement::Node(child_node) => {
                    if !first {
                        self.newline();
                    }
                    first = false;
                    self.emit_node(&child_node);
                }
            }
        }
    }

    fn emit_let_stmt(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_expr_stmt(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_return_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_if_expr(&mut self, node: &SyntaxNode) {
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(child_node) => match child_node.kind() {
                    SyntaxKind::Block => {
                        self.space();
                        self.emit_block_with_brace(&child_node);
                    }
                    SyntaxKind::IfExpr => {
                        self.space();
                        self.emit_node(&child_node);
                    }
                    _ => {
                        self.space();
                        self.emit_node(&child_node);
                    }
                },
                SyntaxElement::Token(token) => {
                    let text = token.text();
                    if text == "else" {
                        self.flush_pending();
                        self.output.pop(); // remove trailing space from pending_space
                        self.output.push_str(" else");
                        self.pending_space = true;
                    } else {
                        self.emit_token(&token);
                    }
                }
            }
        }
    }

    fn emit_match_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_match_arm(&mut self, node: &SyntaxNode) {
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(child_node) => {
                    if child_node.kind() == SyntaxKind::Block {
                        self.space();
                        self.emit_block_with_brace(&child_node);
                    } else {
                        self.space();
                        self.emit_node(&child_node);
                    }
                }
                SyntaxElement::Token(token) => {
                    self.emit_token(&token);
                }
            }
        }
        self.newline();
    }

    fn emit_loop_expr(&mut self, node: &SyntaxNode) {
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(child_node) => match child_node.kind() {
                    SyntaxKind::Block => {
                        self.space();
                        self.emit_block_with_brace(&child_node);
                    }
                    _ => {
                        self.space();
                        self.emit_node(&child_node);
                    }
                },
                SyntaxElement::Token(token) => self.emit_token(&token),
            }
        }
    }

    fn emit_while_expr(&mut self, node: &SyntaxNode) {
        self.emit_loop_expr(node);
    }

    fn emit_for_expr(&mut self, node: &SyntaxNode) {
        self.emit_loop_expr(node);
    }

    fn emit_try_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_catch_clause(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_await_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_binary_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_unary_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_call_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_method_call_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_field_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_index_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_range_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_cast_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_parenthesized_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_struct_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_array_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_tuple_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_closure_expr(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_async_block(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_unsafe_block(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_try_block(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_struct_def(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_enum_def(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_trait_def(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_impl_def(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_type_alias(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_const_def(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_static_def(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_use_decl(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_module_decl(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
        self.newline();
    }

    fn emit_generic_params(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_where_clause(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_type_args(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_path(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_param_list(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }

    fn emit_param(&mut self, node: &SyntaxNode) {
        self.emit_children(node);
    }
}

/// Format a source string, returning the formatted source or parse errors.
pub fn format_source(source: &str) -> FormatResult {
    let mut formatter = Formatter::new();
    formatter.format(source)
}

/// Format a source string, returning the input unchanged on parse error.
pub fn format_source_or_keep(source: &str) -> String {
    let mut formatter = Formatter::new();
    formatter.format_or_keep(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_simple_function() {
        let source = "fn main() -> i64 {\n  return 42;\n}\n";
        let mut fmt = Formatter::new();
        let result = fmt.format(source).expect("format should succeed");
        assert_eq!(result, "fn main() -> i64 {\n    return 42;\n}\n");
    }

    #[test]
    fn is_idempotent() {
        let source = "fn main() -> i64 {\n    return 42;\n}\n";
        let mut fmt = Formatter::new();
        let first = fmt.format(source).expect("first format");
        let second = fmt.format(&first).expect("second format");
        assert_eq!(first, second);
    }

    #[test]
    fn formats_let_statement() {
        let source = "fn main() -> i64 {\nlet x = 1 + 2;\nreturn x;\n}\n";
        let mut fmt = Formatter::new();
        let result = fmt.format(source).expect("format should succeed");
        assert_eq!(result, "fn main() -> i64 {\n    let x = 1 + 2;\n    return x;\n}\n");
    }

    #[test]
    fn formats_if_expression() {
        let source = "fn main() -> i64 {\nif x > 0 {\nreturn 1;\n} else {\nreturn 0;\n}\n}\n";
        let mut fmt = Formatter::new();
        let result = fmt.format(source).expect("format should succeed");
        assert_eq!(
            result,
            "fn main() -> i64 {\n    if x > 0 {\n        return 1;\n    } else {\n        return 0;\n    }\n}\n"
        );
    }

    #[test]
    fn rejects_unparseable_source() {
        let mut fmt = Formatter::new();
        let err = fmt.format("fn main( -> i64 {").expect_err("should fail to parse");
        assert!(!err.is_empty());
    }

    #[test]
    fn preserves_comments() {
        let source = "fn main() -> i64 {\n    // a comment\n    return 42;\n}\n";
        let mut fmt = Formatter::new();
        let result = fmt.format(source).expect("format should succeed");
        assert!(result.contains("// a comment"));
    }

    #[test]
    fn preserves_strings() {
        let source = "fn main() -> i64 {\n    let s = \"hello\";\n    return 0;\n}\n";
        let mut fmt = Formatter::new();
        let result = fmt.format(source).expect("format should succeed");
        assert!(result.contains("\"hello\""));
    }
}
