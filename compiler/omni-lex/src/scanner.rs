use crate::token::{Kw, Punct, Span, Token, TokenKind, Trivia, TriviaKind};
use omni_source::Cursor;
use unicode_ident::{is_xid_continue, is_xid_start};

pub struct Scanner<'a> {
    source: &'a [u8],
    cursor: Cursor<'a>,
    file_id: u16,
    eof_trivia: Vec<Trivia>,
}

impl<'a> Scanner<'a> {
    /// Creates a scanner over raw source bytes.
    ///
    /// Accepting `&[u8]` is what makes malformed UTF-8 reachable: the cursor
    /// decodes what it can and emits `U+FFFD` for the rest without ever
    /// panicking, and every token span is an exact byte range of this buffer
    /// taken before line-ending normalization, so `source[span]` round-trips.
    pub fn new(source: &'a [u8], file_id: u16) -> Self {
        Self { source, cursor: Cursor::new(source), file_id, eof_trivia: Vec::new() }
    }

    /// Convenience constructor for UTF-8 text sources.
    pub fn from_str(source: &'a str, file_id: u16) -> Self {
        Self::new(source.as_bytes(), file_id)
    }

    /// Trailing trivia that follows the last token, captured when
    /// [`Scanner::next_token`] first reported end of input.
    ///
    /// Taking it is required for lossless reconstruction: without a following
    /// token there is nothing else to carry the bytes past the final newline,
    /// the byte-order mark of a mark-only file, or an otherwise trivia-only
    /// source.
    pub fn take_eof_trivia(&mut self) -> Vec<Trivia> {
        std::mem::take(&mut self.eof_trivia)
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let leading_trivia = self.scan_trivia(false);
        let start = self.cursor.pos() as u32;
        let c = match self.cursor.peek() {
            Some(c) => c,
            None => {
                if self.eof_trivia.is_empty() {
                    self.eof_trivia = leading_trivia;
                }
                return None;
            }
        };

        // Peek ahead to handle multi-char tokens
        let c2 = self.cursor.peek_nth(1);

        let kind = match c {
            '0'..='9' => self.scan_number(),
            'r' if c2 == Some('"') || (c2 == Some('#') && self.cursor.peek_nth(2) == Some('"')) => {
                self.scan_raw_string()
            }
            // `r#"` is a raw string (checked above); any other `r#` is a raw
            // identifier, which is the only spelling that makes a keyword
            // usable as a name (LEX-0005).
            'r' if c2 == Some('#') => self.scan_raw_identifier(),
            'b' if c2 == Some('\'') || c2 == Some('"') => {
                self.cursor.advance();
                if self.cursor.peek() == Some('\'') {
                    self.scan_char_or_byte()
                } else {
                    self.scan_string()
                }
            }
            '\'' => self.scan_char_or_byte(),
            '"' => self.scan_string(),
            // identifier = (XID_Start | "_") (XID_Continue | "_")*
            c if is_xid_start(c) || c == '_' => self.scan_ident_or_keyword(),
            _ => self.scan_punctuation(),
        };
        let end = self.cursor.pos() as u32;
        let trailing_trivia = self.scan_trivia(true);
        Some(Token {
            kind,
            span: Span { start, end, file_id: self.file_id },
            leading_trivia,
            trailing_trivia,
        })
    }

    fn scan_trivia(&mut self, stop_at_newline: bool) -> Vec<Trivia> {
        let mut trivias = Vec::new();
        // SRC-0001: an optional UTF-8 BOM is honoured only at byte offset zero.
        // Carrying it as whitespace trivia keeps the three bytes span-covered,
        // so reconstruction stays lossless while semantic hashing (which skips
        // trivia) never sees it.
        if self.cursor.pos() == 0 && self.cursor.skip_bom() {
            trivias.push(Trivia {
                kind: TriviaKind::Whitespace,
                span: Span { start: 0, end: 3, file_id: self.file_id },
            });
        }
        while let Some(c) = self.cursor.peek() {
            if stop_at_newline && c == '\n' {
                break;
            }
            let start = self.cursor.pos() as u32;
            if c.is_whitespace() {
                self.cursor.advance();
                while let Some(w) = self.cursor.peek() {
                    if stop_at_newline && w == '\n' {
                        break;
                    }
                    if w.is_whitespace() {
                        self.cursor.advance();
                    } else {
                        break;
                    }
                }
                trivias.push(Trivia {
                    kind: TriviaKind::Whitespace,
                    span: Span { start, end: self.cursor.pos() as u32, file_id: self.file_id },
                });
            } else if c == '/' {
                let mut lookahead = self.cursor;
                lookahead.advance();
                let c2 = lookahead.peek();
                match c2 {
                    Some('/') => {
                        let c3 = lookahead.peek_nth(1);
                        let kind = if c3 == Some('/') || c3 == Some('!') {
                            TriviaKind::DocComment
                        } else {
                            TriviaKind::LineComment
                        };
                        self.cursor.advance();
                        self.cursor.advance();
                        while let Some(ch) = self.cursor.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.cursor.advance();
                        }
                        trivias.push(Trivia {
                            kind,
                            span: Span {
                                start,
                                end: self.cursor.pos() as u32,
                                file_id: self.file_id,
                            },
                        });
                    }
                    Some('*') => {
                        let c3 = lookahead.peek_nth(1);
                        let kind = if c3 == Some('*') || c3 == Some('!') {
                            TriviaKind::DocComment
                        } else {
                            TriviaKind::BlockComment
                        };
                        self.cursor.advance();
                        self.cursor.advance();
                        let mut depth = 1;
                        while depth > 0 {
                            match self.cursor.advance() {
                                Some('/') if self.cursor.peek() == Some('*') => {
                                    self.cursor.advance();
                                    depth += 1;
                                }
                                Some('*') if self.cursor.peek() == Some('/') => {
                                    self.cursor.advance();
                                    depth -= 1;
                                }
                                Some(_) => {}
                                None => break,
                            }
                        }
                        trivias.push(Trivia {
                            kind,
                            span: Span {
                                start,
                                end: self.cursor.pos() as u32,
                                file_id: self.file_id,
                            },
                        });
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        trivias
    }

    fn scan_ident_tail(&mut self) {
        while let Some(c) = self.cursor.peek() {
            if is_xid_continue(c) || c == '_' {
                self.cursor.advance();
            } else {
                break;
            }
        }
    }

    /// `raw_identifier = "r#" identifier` (LEX-0005).
    ///
    /// The `r#` prefix is validated by the caller's guard. The tail is checked
    /// against the identifier rule but is deliberately never looked up in the
    /// keyword table — that omission is the entire point of the form, and it
    /// also means a malformed tail reports `Error` instead of silently
    /// producing a keyword.
    fn scan_raw_identifier(&mut self) -> TokenKind {
        self.cursor.advance();
        self.cursor.advance();
        match self.cursor.peek() {
            Some(c) if is_xid_start(c) || c == '_' => {}
            _ => return TokenKind::Error,
        }
        self.scan_ident_tail();
        TokenKind::Ident
    }

    fn scan_ident_or_keyword(&mut self) -> TokenKind {
        let start = self.cursor.pos() as u32;
        self.scan_ident_tail();
        let end = self.cursor.pos() as u32;
        // Keyword set = the normative `keyword` production (91 spellings)
        // plus the two identifier-shaped terminals the grammar body requires
        // but the production omits: `mod` (`module_decl`, spec line 166) and
        // `crate` (`extern_crate_decl`, line 169). Honouring the production
        // alone would make both productions unsatisfiable. Conversely
        // `parallel` and `with` are in neither place, so they stay
        // identifiers. See `tests::keyword_table_matches_the_normative_grammar`.
        let text =
            std::str::from_utf8(&self.source[start as usize..end as usize]).unwrap_or_default();
        match text {
            "Never" => TokenKind::Keyword(Kw::Never),
            "Self" => TokenKind::Keyword(Kw::SelfKw),
            "Sized" => TokenKind::Keyword(Kw::Sized),
            "addrspace" => TokenKind::Keyword(Kw::Addrspace),
            "as" => TokenKind::Keyword(Kw::As),
            "async" => TokenKind::Keyword(Kw::Async),
            "await" => TokenKind::Keyword(Kw::Await),
            "bare_metal" => TokenKind::Keyword(Kw::BareMetal),
            "bf16" => TokenKind::Keyword(Kw::Bf16),
            "bool" => TokenKind::Keyword(Kw::Bool),
            "break" => TokenKind::Keyword(Kw::Break),
            "byte" => TokenKind::Keyword(Kw::Byte),
            "cap" => TokenKind::Keyword(Kw::Cap),
            "catch" => TokenKind::Keyword(Kw::Catch),
            "char" => TokenKind::Keyword(Kw::Char),
            "const" => TokenKind::Keyword(Kw::Const),
            "continue" => TokenKind::Keyword(Kw::Continue),
            "crate" => TokenKind::Keyword(Kw::Crate),
            "dec128" => TokenKind::Keyword(Kw::Dec128),
            "dec32" => TokenKind::Keyword(Kw::Dec32),
            "dec64" => TokenKind::Keyword(Kw::Dec64),
            "defer" => TokenKind::Keyword(Kw::Defer),
            "distributed" => TokenKind::Keyword(Kw::Distributed),
            "dyn" => TokenKind::Keyword(Kw::Dyn),
            "effect" => TokenKind::Keyword(Kw::Effect),
            "else" => TokenKind::Keyword(Kw::Else),
            "enum" => TokenKind::Keyword(Kw::Enum),
            "ensure" => TokenKind::Keyword(Kw::Ensure),
            "extern" => TokenKind::Keyword(Kw::Extern),
            "f128" => TokenKind::Keyword(Kw::F128),
            "f16" => TokenKind::Keyword(Kw::F16),
            "f32" => TokenKind::Keyword(Kw::F32),
            "f64" => TokenKind::Keyword(Kw::F64),
            "false" => TokenKind::Keyword(Kw::False),
            "fn" => TokenKind::Keyword(Kw::Fn),
            "for" => TokenKind::Keyword(Kw::For),
            "hosted" => TokenKind::Keyword(Kw::Hosted),
            "i128" => TokenKind::Keyword(Kw::I128),
            "i16" => TokenKind::Keyword(Kw::I16),
            "i32" => TokenKind::Keyword(Kw::I32),
            "i64" => TokenKind::Keyword(Kw::I64),
            "i8" => TokenKind::Keyword(Kw::I8),
            "if" => TokenKind::Keyword(Kw::If),
            "impl" => TokenKind::Keyword(Kw::Impl),
            "in" => TokenKind::Keyword(Kw::In),
            "is" => TokenKind::Keyword(Kw::Is),
            "isolate" => TokenKind::Keyword(Kw::Isolate),
            "isize" => TokenKind::Keyword(Kw::Isize),
            "let" => TokenKind::Keyword(Kw::Let),
            "loop" => TokenKind::Keyword(Kw::Loop),
            "macro" => TokenKind::Keyword(Kw::Macro),
            "managed" => TokenKind::Keyword(Kw::Managed),
            "match" => TokenKind::Keyword(Kw::Match),
            "mod" => TokenKind::Keyword(Kw::Mod),
            "module" => TokenKind::Keyword(Kw::Module),
            "move" => TokenKind::Keyword(Kw::Move),
            "mut" => TokenKind::Keyword(Kw::Mut),
            "not" => TokenKind::Keyword(Kw::Not),
            "opaque" => TokenKind::Keyword(Kw::Opaque),
            "override" => TokenKind::Keyword(Kw::Override),
            "package" => TokenKind::Keyword(Kw::Package),
            "panic" => TokenKind::Keyword(Kw::Panic),
            "persistent" => TokenKind::Keyword(Kw::Persistent),
            "pub" => TokenKind::Keyword(Kw::Pub),
            "pure" => TokenKind::Keyword(Kw::Pure),
            "ref" => TokenKind::Keyword(Kw::Ref),
            "relation" => TokenKind::Keyword(Kw::Relation),
            "require" => TokenKind::Keyword(Kw::Require),
            "return" => TokenKind::Keyword(Kw::Return),
            "script" => TokenKind::Keyword(Kw::Script),
            "self" => TokenKind::Keyword(Kw::SelfRef),
            "static" => TokenKind::Keyword(Kw::Static),
            "str" => TokenKind::Keyword(Kw::Str),
            "struct" => TokenKind::Keyword(Kw::Struct),
            "super" => TokenKind::Keyword(Kw::Super),
            "thread_local" => TokenKind::Keyword(Kw::ThreadLocal),
            "trait" => TokenKind::Keyword(Kw::Trait),
            "true" => TokenKind::Keyword(Kw::True),
            "try" => TokenKind::Keyword(Kw::Try),
            "type" => TokenKind::Keyword(Kw::Type),
            "typeof" => TokenKind::Keyword(Kw::Typeof),
            "u128" => TokenKind::Keyword(Kw::U128),
            "u16" => TokenKind::Keyword(Kw::U16),
            "u32" => TokenKind::Keyword(Kw::U32),
            "u64" => TokenKind::Keyword(Kw::U64),
            "u8" => TokenKind::Keyword(Kw::U8),
            "unsafe" => TokenKind::Keyword(Kw::Unsafe),
            "use" => TokenKind::Keyword(Kw::Use),
            "usize" => TokenKind::Keyword(Kw::Usize),
            "verified" => TokenKind::Keyword(Kw::Verified),
            "where" => TokenKind::Keyword(Kw::Where),
            "while" => TokenKind::Keyword(Kw::While),
            "yield" => TokenKind::Keyword(Kw::Yield),
            _ => TokenKind::Ident,
        }
    }

    fn scan_punctuation(&mut self) -> TokenKind {
        let first = self.cursor.advance().expect("punctuation requires a character");
        TokenKind::Punct(match first {
            '+' => Punct::Plus,
            '-' if self.cursor.peek() == Some('>') => {
                self.cursor.advance();
                Punct::Arrow
            }
            '-' => Punct::Minus,
            '*' => Punct::Star,
            '/' => Punct::Slash,
            '%' => Punct::Percent,
            '^' => Punct::Caret,
            '=' if self.cursor.peek() == Some('=') => {
                self.cursor.advance();
                Punct::EqEq
            }
            '=' if self.cursor.peek() == Some('>') => {
                self.cursor.advance();
                Punct::FatArrow
            }
            '=' => Punct::Eq,
            '!' if self.cursor.peek() == Some('=') => {
                self.cursor.advance();
                Punct::NotEq
            }
            '!' => Punct::Bang,
            '<' if self.cursor.peek() == Some('=') => {
                self.cursor.advance();
                Punct::Le
            }
            '<' => Punct::Lt,
            '>' if self.cursor.peek() == Some('=') => {
                self.cursor.advance();
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
            ':' if self.cursor.peek() == Some(':') => {
                self.cursor.advance();
                Punct::ColonColon
            }
            ':' => Punct::Colon,
            ';' => Punct::Semicolon,
            '&' if self.cursor.peek() == Some('&') => {
                self.cursor.advance();
                Punct::AmpAmp
            }
            '&' => Punct::Amp,
            '|' if self.cursor.peek() == Some('>') => {
                self.cursor.advance();
                Punct::PipeArrow
            }
            '|' if self.cursor.peek() == Some('|') => {
                self.cursor.advance();
                Punct::PipePipe
            }
            '|' => Punct::Pipe,
            '.' if self.cursor.peek() == Some('.') => {
                self.cursor.advance();
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    Punct::DotDotEq
                } else {
                    Punct::DotDot
                }
            }
            '.' if self.cursor.peek() == Some('?') => {
                self.cursor.advance();
                Punct::QuestionDot
            }
            '.' => Punct::Dot,
            '?' if self.cursor.peek() == Some('?') => {
                self.cursor.advance();
                Punct::QuestionQuestion
            }
            '?' => Punct::Question,
            '@' => Punct::At,
            '#' => Punct::Hash,
            '$' => Punct::Dollar,
            '_' => Punct::Underscore,
            _ => return TokenKind::Error,
        })
    }

    fn scan_number(&mut self) -> TokenKind {
        while let Some(c) = self.cursor.peek() {
            if c.is_ascii_digit() || c == '_' {
                self.cursor.advance();
            } else {
                break;
            }
        }
        if self.cursor.peek() == Some('.') {
            self.cursor.advance();
            while let Some(c) = self.cursor.peek() {
                if c.is_ascii_digit() || c == '_' {
                    self.cursor.advance();
                } else {
                    break;
                }
            }
            TokenKind::Float
        } else {
            TokenKind::Int
        }
    }

    fn scan_string(&mut self) -> TokenKind {
        self.cursor.advance();
        while let Some(c) = self.cursor.peek() {
            if c == '"' {
                self.cursor.advance();
                return TokenKind::String;
            }
            if c == '\\' {
                self.cursor.advance();
            }
            self.cursor.advance();
        }
        TokenKind::Error
    }

    fn scan_char_or_byte(&mut self) -> TokenKind {
        self.cursor.advance();
        while let Some(c) = self.cursor.peek() {
            if c == '\'' {
                self.cursor.advance();
                return TokenKind::Char;
            }
            if c == '\\' {
                self.cursor.advance();
            }
            self.cursor.advance();
        }
        TokenKind::Error
    }

    fn scan_raw_string(&mut self) -> TokenKind {
        self.cursor.advance();
        let mut hashes = 0;
        while self.cursor.peek() == Some('#') {
            self.cursor.advance();
            hashes += 1;
        }
        if self.cursor.advance() != Some('"') {
            return TokenKind::Error;
        }
        while let Some(c) = self.cursor.advance() {
            if c == '"' {
                let mut h = 0;
                while self.cursor.peek() == Some('#') && h < hashes {
                    self.cursor.advance();
                    h += 1;
                }
                if h == hashes {
                    return TokenKind::RawString;
                }
            }
        }
        TokenKind::Error
    }
}
