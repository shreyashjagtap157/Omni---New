//! UTF-8 Byte Cursor with single-pass line ending normalization and BOM stripping.

/// Zero-allocation UTF-8 byte cursor for lexical scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
    pub line: u32,
    pub col: u32,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor.
    ///
    /// Strips the UTF-8 BOM (`0xEF, 0xBB, 0xBF`) strictly at `pos == 0`.
    pub fn new(bytes: &'a [u8]) -> Self {
        let mut cursor = Self { bytes, pos: 0, line: 1, col: 1 };

        if cursor.bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            cursor.pos = 3;
        }

        cursor
    }

    /// Advances the cursor and returns the next normalized Unicode character.
    ///
    /// - Normalizes `\r\n` and `\r` to `\n` in a single pass without allocation.
    /// - Strictly maintains byte offset accuracy in `self.pos`.
    /// - Advances line/column counters predictably.
    /// - Replaces malformed or incomplete UTF-8 byte sequences with `U+FFFD` without panicking.
    pub fn advance(&mut self) -> Option<char> {
        if self.pos == 0 && self.bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            self.pos = 3;
        }

        if self.pos >= self.bytes.len() {
            return None;
        }

        let remaining = &self.bytes[self.pos..];

        // 1. Line ending normalization: \r\n and \r -> \n
        if remaining[0] == b'\r' {
            if remaining.len() > 1 && remaining[1] == b'\n' {
                self.pos += 2;
            } else {
                self.pos += 1;
            }
            self.line += 1;
            self.col = 1;
            return Some('\n');
        }

        if remaining[0] == b'\n' {
            self.pos += 1;
            self.line += 1;
            self.col = 1;
            return Some('\n');
        }

        // 2. Fast-path ASCII (0x00..=0x7F)
        if remaining[0] < 0x80 {
            let ch = remaining[0] as char;
            self.pos += 1;
            self.col += 1;
            return Some(ch);
        }

        // 3. Multi-byte UTF-8 decoding
        let lead = remaining[0];
        let seq_len = match lead {
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => 1,
        };

        if seq_len > 1 && remaining.len() >= seq_len {
            if let Ok(valid_str) = std::str::from_utf8(&remaining[..seq_len]) {
                if let Some(ch) = valid_str.chars().next() {
                    self.pos += seq_len;
                    self.col += 1;
                    return Some(ch);
                }
            }
        }

        // Fallback for malformed UTF-8: consume single invalid byte, emit U+FFFD
        self.pos += 1;
        self.col += 1;
        Some('\u{FFFD}')
    }

    /// Peeks at the next character without advancing cursor state.
    pub fn peek(&self) -> Option<char> {
        let mut clone = *self;
        clone.advance()
    }

    /// Peeks `n` characters ahead without mutating state (0 is equivalent to `peek()`).
    pub fn peek_nth(&self, n: usize) -> Option<char> {
        let mut clone = *self;
        let mut result = None;
        for _ in 0..=n {
            result = clone.advance();
        }
        result
    }

    /// Returns the current byte offset.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the current line (1-indexed).
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Returns the current column (1-indexed).
    pub fn col(&self) -> u32 {
        self.col
    }

    /// Returns `true` if the cursor has consumed all available bytes.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    /// Returns the remaining unconsumed slice of source bytes.
    pub fn rest(&self) -> &'a [u8] {
        if self.pos >= self.bytes.len() {
            &[]
        } else {
            &self.bytes[self.pos..]
        }
    }

    /// Returns a subslice between `start` and `end` byte offsets.
    pub fn slice(&self, start: usize, end: usize) -> &'a [u8] {
        &self.bytes[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_cursor() {
        let mut cursor = Cursor::new(b"");
        assert_eq!(cursor.advance(), None);
        assert!(cursor.is_eof());
        assert_eq!(cursor.pos(), 0);
        assert_eq!(cursor.line(), 1);
        assert_eq!(cursor.col(), 1);
    }

    #[test]
    fn test_ascii_traversal() {
        let mut cursor = Cursor::new(b"let x = 42;");
        assert_eq!(cursor.peek(), Some('l'));
        assert_eq!(cursor.peek_nth(1), Some('e'));
        assert_eq!(cursor.advance(), Some('l'));
        assert_eq!(cursor.pos(), 1);
        assert_eq!(cursor.col(), 2);

        while cursor.advance().is_some() {}
        assert!(cursor.is_eof());
        assert_eq!(cursor.pos(), 12);
        assert_eq!(cursor.line(), 1);
        assert_eq!(cursor.col(), 13);
    }

    #[test]
    fn test_crlf_and_cr_normalization() {
        let input = b"line1\r\nline2\rline3\nline4";
        let mut cursor = Cursor::new(input);

        let mut characters = Vec::new();
        while let Some(ch) = cursor.advance() {
            characters.push(ch);
        }

        let result: String = characters.into_iter().collect();
        assert_eq!(result, "line1\nline2\nline3\nline4");
        assert_eq!(cursor.line(), 4);
        assert_eq!(cursor.pos(), input.len());
    }

    #[test]
    fn test_bom_stripping_at_zero() {
        let input = b"\xEF\xBB\xBFfn main";
        let mut cursor = Cursor::new(input);
        assert_eq!(cursor.pos(), 3);
        assert_eq!(cursor.advance(), Some('f'));
        assert_eq!(cursor.pos(), 4);
        assert_eq!(cursor.col(), 2);
    }

    #[test]
    fn test_bom_preserved_not_at_zero() {
        let input = b"a\xEF\xBB\xBFb";
        let mut cursor = Cursor::new(input);
        assert_eq!(cursor.advance(), Some('a'));
        assert_eq!(cursor.advance(), Some('\u{FEFF}'));
        assert_eq!(cursor.advance(), Some('b'));
    }

    #[test]
    fn test_multibyte_utf8() {
        let input = "let α = \"🦀\";".as_bytes();
        let mut cursor = Cursor::new(input);

        assert_eq!(cursor.advance(), Some('l'));
        assert_eq!(cursor.advance(), Some('e'));
        assert_eq!(cursor.advance(), Some('t'));
        assert_eq!(cursor.advance(), Some(' '));

        let alpha_start = cursor.pos();
        assert_eq!(cursor.advance(), Some('α'));
        let alpha_end = cursor.pos();
        assert_eq!(alpha_end - alpha_start, 2);

        assert_eq!(cursor.advance(), Some(' '));
        assert_eq!(cursor.advance(), Some('='));
        assert_eq!(cursor.advance(), Some(' '));
        assert_eq!(cursor.advance(), Some('"'));

        let crab_start = cursor.pos();
        assert_eq!(cursor.advance(), Some('🦀'));
        let crab_end = cursor.pos();
        assert_eq!(crab_end - crab_start, 4);

        assert_eq!(cursor.slice(crab_start, crab_end), "🦀".as_bytes());
    }

    #[test]
    fn test_malformed_utf8_recovery() {
        let input = b"\x80\xFF\xC2abc";
        let mut cursor = Cursor::new(input);

        assert_eq!(cursor.advance(), Some('\u{FFFD}'));
        assert_eq!(cursor.advance(), Some('\u{FFFD}'));
        assert_eq!(cursor.advance(), Some('\u{FFFD}'));
        assert_eq!(cursor.advance(), Some('a'));
        assert_eq!(cursor.advance(), Some('b'));
        assert_eq!(cursor.advance(), Some('c'));
        assert_eq!(cursor.advance(), None);
    }
}
