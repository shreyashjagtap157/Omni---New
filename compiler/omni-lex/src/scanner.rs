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
            "fn" => Some(Kw::Fn),
            "let" => Some(Kw::Let),
            "mut" => Some(Kw::Mut),
            "if" => Some(Kw::If),
            "else" => Some(Kw::Else),
            "return" => Some(Kw::Return),
            "match" => Some(Kw::Match),
            "struct" => Some(Kw::Struct),
            "enum" => Some(Kw::Enum),
            "true" => Some(Kw::True),
            "false" => Some(Kw::False),
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
            '=' if self.peek() == Some('=') => {
                self.advance();
                Punct::EqEq
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
            ':' => Punct::Colon,
            ';' => Punct::Semicolon,
            '&' => Punct::Amp,
            '|' if self.peek() == Some('>') => {
                self.advance();
                Punct::PipeArrow
            }
            '|' => Punct::Pipe,
            '.' => Punct::Dot,
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
