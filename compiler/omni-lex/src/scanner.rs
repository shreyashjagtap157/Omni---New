use crate::token::{Kw, Punct, Span, Token, TokenKind, Trivia, TriviaKind};
use omni_source::Cursor;

/// The Maximal Munch lexical scanner.
pub struct Scanner<'a> {
    source: &'a str,
    cursor: Cursor<'a>,
    file_id: u16,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, cursor: Cursor<'a>, file_id: u16) -> Self {
        Self { source, cursor, file_id }
    }

    /// Pulls the next token using maximal munch DFA logic, attaching leading and trailing trivia.
    pub fn next_token(&mut self) -> Option<Token> {
        // 1. Consume all leading trivia (crosses newlines)
        let leading_trivia = self.scan_trivia(false);

        let start = self.pos();
        let Some(c) = self.peek() else {
            // EOF reached
            return None;
        };

        // 2. Scan the actual token
        let kind = match c {
            'a'..='z' | 'A'..='Z' | '_' => self.scan_ident_or_keyword(),
            '0'..='9' => self.scan_number(),
            _ => self.scan_punctuation(),
        };

        let end = self.pos();

        // 3. Consume trailing trivia (stops at the first newline)
        let trailing_trivia = self.scan_trivia(true);

        Some(Token {
            kind,
            span: Span { start, end, file_id: self.file_id },
            leading_trivia,
            trailing_trivia,
        })
    }

    /// Scans consecutive trivia (whitespace and comments).
    /// If `stop_at_newline` is true, it aborts upon hitting a `\n`.
    fn scan_trivia(&mut self, stop_at_newline: bool) -> Vec<Trivia> {
        let mut trivias = Vec::new();

        while let Some(c) = self.peek() {
            if stop_at_newline && c == '\n' {
                break;
            }

            let start = self.pos();

            // 1. Whitespace
            if c.is_whitespace() {
                self.advance();
                while let Some(w) = self.peek() {
                    if stop_at_newline && w == '\n' {
                        break;
                    }
                    if w.is_whitespace() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                trivias.push(Trivia {
                    kind: TriviaKind::Whitespace,
                    span: Span { start, end: self.pos(), file_id: self.file_id },
                });
                continue;
            }

            // 2. Comments (Using Cursor lookahead)
            let mut lookahead = self.cursor;
            if lookahead.advance() == Some('/') {
                let next = lookahead.advance();

                // Line Comment
                if next == Some('/') {
                    self.cursor = lookahead; // Commit lookahead
                    let mut kind = TriviaKind::LineComment;

                    // Check for Doc Comment `///` or `//!`
                    if let Some(c3) = self.peek() {
                        if c3 == '/' || c3 == '!' {
                            kind = TriviaKind::DocComment;
                        }
                    }

                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        } // Do not consume the newline
                        self.advance();
                    }

                    trivias.push(Trivia {
                        kind,
                        span: Span { start, end: self.pos(), file_id: self.file_id },
                    });
                    continue;
                }
                // Block Comment
                else if next == Some('*') {
                    self.cursor = lookahead; // Commit lookahead
                    let mut kind = TriviaKind::BlockComment;

                    // Check for Doc Comment `/**` or `/*!`
                    if let Some(c3) = self.peek() {
                        if (c3 == '*' || c3 == '!') && c3 != '/' {
                            kind = TriviaKind::DocComment;
                        }
                    }

                    let mut depth = 1;
                    while depth > 0 {
                        match self.advance() {
                            Some('/') if self.peek() == Some('*') => {
                                self.advance();
                                depth += 1;
                            }
                            Some('*') if self.peek() == Some('/') => {
                                self.advance();
                                depth -= 1;
                            }
                            Some(_) => {}
                            None => break, // Lexical error: Unterminated block comment
                        }
                    }

                    trivias.push(Trivia {
                        kind,
                        span: Span { start, end: self.pos(), file_id: self.file_id },
                    });
                    continue;
                }
            }

            // If it's not whitespace and not a comment, trivia collection is done.
            break;
        }

        trivias
    }

    #[cfg(any())]
    #[implements("LEX-0003")]
    #[implements("LEX-0004")]
    fn _audit_trivia_comments() {}

    // DFA Logic (Unchanged from previous step)
    fn scan_ident_or_keyword(&mut self) -> TokenKind {
        let start = self.pos();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.pos();
        let text = &self.source[start as usize..end as usize];

        let keyword = match text {
            "Never" => Some(Kw::Never),
            "Self" => Some(Kw::SelfKw),
            "Sized" => Some(Kw::Sized),
            "addrspace" => Some(Kw::Addrspace),
            "as" => Some(Kw::As),
            "async" => Some(Kw::Async),
            "await" => Some(Kw::Await),
            "bare_metal" => Some(Kw::BareMetal),
            "bf16" => Some(Kw::Bf16),
            "bool" => Some(Kw::Bool),
            "break" => Some(Kw::Break),
            "byte" => Some(Kw::Byte),
            "cap" => Some(Kw::Cap),
            "catch" => Some(Kw::Catch),
            "char" => Some(Kw::Char),
            "const" => Some(Kw::Const),
            "continue" => Some(Kw::Continue),
            "dec128" => Some(Kw::Dec128),
            "dec32" => Some(Kw::Dec32),
            "dec64" => Some(Kw::Dec64),
            "defer" => Some(Kw::Defer),
            "distributed" => Some(Kw::Distributed),
            "dyn" => Some(Kw::Dyn),
            "effect" => Some(Kw::Effect),
            "else" => Some(Kw::Else),
            "enum" => Some(Kw::Enum),
            "ensure" => Some(Kw::Ensure),
            "extern" => Some(Kw::Extern),
            "f128" => Some(Kw::F128),
            "f16" => Some(Kw::F16),
            "f32" => Some(Kw::F32),
            "f64" => Some(Kw::F64),
            "false" => Some(Kw::False),
            "fn" => Some(Kw::Fn),
            "for" => Some(Kw::For),
            "hosted" => Some(Kw::Hosted),
            "i128" => Some(Kw::I128),
            "i16" => Some(Kw::I16),
            "i32" => Some(Kw::I32),
            "i64" => Some(Kw::I64),
            "i8" => Some(Kw::I8),
            "if" => Some(Kw::If),
            "impl" => Some(Kw::Impl),
            "in" => Some(Kw::In),
            "is" => Some(Kw::Is),
            "isolate" => Some(Kw::Isolate),
            "isize" => Some(Kw::Isize),
            "let" => Some(Kw::Let),
            "loop" => Some(Kw::Loop),
            "macro" => Some(Kw::Macro),
            "managed" => Some(Kw::Managed),
            "match" => Some(Kw::Match),
            "mod" => Some(Kw::Mod),
            "module" => Some(Kw::Module),
            "move" => Some(Kw::Move),
            "mut" => Some(Kw::Mut),
            "not" => Some(Kw::Not),
            "opaque" => Some(Kw::Opaque),
            "override" => Some(Kw::Override),
            "package" => Some(Kw::Package),
            "panic" => Some(Kw::Panic),
            "parallel" => Some(Kw::Parallel),
            "persistent" => Some(Kw::Persistent),
            "pub" => Some(Kw::Pub),
            "pure" => Some(Kw::Pure),
            "ref" => Some(Kw::Ref),
            "relation" => Some(Kw::Relation),
            "require" => Some(Kw::Require),
            "return" => Some(Kw::Return),
            "script" => Some(Kw::Script),
            "self" => Some(Kw::SelfRef),
            "static" => Some(Kw::Static),
            "str" => Some(Kw::Str),
            "struct" => Some(Kw::Struct),
            "super" => Some(Kw::Super),
            "thread_local" => Some(Kw::ThreadLocal),
            "trait" => Some(Kw::Trait),
            "true" => Some(Kw::True),
            "try" => Some(Kw::Try),
            "type" => Some(Kw::Type),
            "typeof" => Some(Kw::Typeof),
            "u128" => Some(Kw::U128),
            "u16" => Some(Kw::U16),
            "u32" => Some(Kw::U32),
            "u64" => Some(Kw::U64),
            "u8" => Some(Kw::U8),
            "unsafe" => Some(Kw::Unsafe),
            "use" => Some(Kw::Use),
            "usize" => Some(Kw::Usize),
            "verified" => Some(Kw::Verified),
            "where" => Some(Kw::Where),
            "while" => Some(Kw::While),
            "with" => Some(Kw::With),
            "yield" => Some(Kw::Yield),
            _ => None,
        };
        keyword.map(TokenKind::Keyword).unwrap_or(TokenKind::Ident)
    }

    fn scan_punctuation(&mut self) -> TokenKind {
        let first = self.advance().expect("punctuation requires a character");
        let kind = match first {
            '+' => Punct::Plus,
            '-' if self.peek() == Some('>') => {
                self.advance();
                Punct::Arrow
            }
            '-' => Punct::Minus,
            '*' => Punct::Star,
            '/' => Punct::Slash,
            '%' => Punct::Percent,
            '^' => Punct::Caret,
            '=' if self.peek() == Some('=') => {
                self.advance();
                Punct::EqEq
            }
            '=' if self.peek() == Some('>') => {
                self.advance();
                Punct::FatArrow
            }
            '=' => Punct::Eq,
            '!' if self.peek() == Some('=') => {
                self.advance();
                Punct::NotEq
            }
            '!' => Punct::Bang,
            '<' if self.peek() == Some('=') => {
                self.advance();
                Punct::Le
            }
            '<' => Punct::Lt,
            '>' if self.peek() == Some('=') => {
                self.advance();
                Punct::Ge
            }
            '>' => Punct::Gt,
            '(' => Punct::LParen,
            ')' => Punct::RParen,
            '{' => Punct::LBrace,
            '}' => Punct::RBrace,
            '[' => Punct::LBracket,
            ']' => Punct::RBracket,
            ',' => Punct::Comma,
            ':' if self.peek() == Some(':') => {
                self.advance();
                Punct::ColonColon
            }
            ':' => Punct::Colon,
            ';' => Punct::Semicolon,
            '&' if self.peek() == Some('&') => {
                self.advance();
                Punct::AmpAmp
            }
            '&' => Punct::Amp,
            '|' if self.peek() == Some('>') => {
                self.advance();
                Punct::PipeArrow
            }
            '|' if self.peek() == Some('|') => {
                self.advance();
                Punct::PipePipe
            }
            '|' => Punct::Pipe,
            '.' if self.peek() == Some('.') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Punct::DotDotEq
                } else {
                    Punct::DotDot
                }
            }
            '.' => Punct::Dot,
            '?' if self.peek() == Some('.') => {
                self.advance();
                Punct::QuestionDot
            }
            '?' if self.peek() == Some('?') => {
                self.advance();
                Punct::QuestionQuestion
            }
            '?' => Punct::Question,
            '@' => Punct::At,
            '#' => Punct::Hash,
            '$' => Punct::Dollar,
            '_' => Punct::Underscore,
            _ => return TokenKind::Error,
        };
        TokenKind::Punct(kind)
    }

    fn scan_number(&mut self) -> TokenKind {
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        if let Some('.') = self.peek() {
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
            return TokenKind::Float;
        }
        TokenKind::Int
    }

    #[inline(always)]
    fn peek(&self) -> Option<char> {
        self.cursor.peek()
    }

    #[inline(always)]
    fn advance(&mut self) -> Option<char> {
        self.cursor.advance()
    }

    #[inline(always)]
    fn pos(&self) -> u32 {
        self.cursor.pos() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Kw, Punct, TokenKind, TriviaKind};
    use omni_source::Cursor;

    // Helper macro to keep the test assertions crisp
    macro_rules! assert_trivia {
        ($trivias:expr, [$( $kind:pat ),*]) => {
            let kinds: Vec<_> = $trivias.iter().map(|t| &t.kind).collect();
            assert!(
                matches!(kinds.as_slice(), [$( $kind ),*]),
                "Trivia mismatch. Got: {:?}", kinds
            );
        };
    }

    #[test]
    fn test_basic_token_scanning() {
        let source = "let val = 42.5;";
        let mut scanner = Scanner::new(source, Cursor::new(source.as_bytes()), 0);

        assert_eq!(scanner.next_token().unwrap().kind, TokenKind::Keyword(Kw::Let)); // "let"
        assert_eq!(scanner.next_token().unwrap().kind, TokenKind::Ident); // "val"
        assert_eq!(scanner.next_token().unwrap().kind, TokenKind::Punct(Punct::Eq)); // "="
        assert_eq!(scanner.next_token().unwrap().kind, TokenKind::Float); // "42.5"
        assert_eq!(scanner.next_token().unwrap().kind, TokenKind::Punct(Punct::Semicolon)); // ";"
        assert!(scanner.next_token().is_none());
    }

    #[test]
    fn test_trivia_separation() {
        // Leading: spaces, newline, spaces, line comment, newline, spaces
        // Token: "foo"
        // Trailing: spaces, line comment (stops at newline)
        let source = "  \n  // lead \n  foo  // trail";
        let mut scanner = Scanner::new(source, Cursor::new(source.as_bytes()), 0);

        let token = scanner.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Ident);

        // Verify leading trivia bridged the newlines correctly
        assert_trivia!(
            token.leading_trivia,
            [&TriviaKind::Whitespace, &TriviaKind::LineComment, &TriviaKind::Whitespace]
        );

        // Verify trailing trivia stopped exactly at the end of the line
        assert_trivia!(token.trailing_trivia, [&TriviaKind::Whitespace, &TriviaKind::LineComment]);
    }

    #[test]
    fn test_nested_block_comments() {
        // A block comment containing another block comment, followed by a token.
        let source = "/* outer /* inner */ still outer */ target";
        let mut scanner = Scanner::new(source, Cursor::new(source.as_bytes()), 0);

        let token = scanner.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Ident);

        assert_trivia!(token.leading_trivia, [&TriviaKind::BlockComment, &TriviaKind::Whitespace]);
    }

    #[test]
    fn test_keyword_and_punctuation_vocabulary() {
        let source = "fn f(a: i32) -> i32 { let x = a + 1; x == 2 && true }";
        let mut scanner = Scanner::new(source, Cursor::new(source.as_bytes()), 0);
        let kinds: Vec<_> = std::iter::from_fn(|| scanner.next_token().map(|t| t.kind)).collect();
        assert!(kinds.contains(&TokenKind::Keyword(Kw::Fn)));
        assert!(kinds.contains(&TokenKind::Punct(Punct::Arrow)));
        assert!(kinds.contains(&TokenKind::Punct(Punct::EqEq)));
        assert!(kinds.contains(&TokenKind::Keyword(Kw::True)));
    }

    #[test]
    fn test_doc_comments() {
        let source = "/// Line doc\n/** Block doc */ fn";
        let mut scanner = Scanner::new(source, Cursor::new(source.as_bytes()), 0);

        let token = scanner.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Keyword(Kw::Fn));

        assert_trivia!(
            token.leading_trivia,
            [
                &TriviaKind::DocComment,
                &TriviaKind::Whitespace,
                &TriviaKind::DocComment,
                &TriviaKind::Whitespace
            ]
        );
    }
}
