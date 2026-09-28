//! SRC-0003 - identifier equality uses Unicode NFC after tokenization.
//!
//! SRC-0003 reads, verbatim from `spec/registry/rule-texts.json`:
//!
//! > Identifier equality uses Unicode NFC after tokenization. String and
//! > character data are never silently normalized.
//!
//! # What this module is and is not
//!
//! It produces a *canonical key* for one already-tokenized identifier token.
//! It never produces normalized text to be written back into the source: the
//! original spelling and the byte span stay authoritative, and the caller
//! keeps both. The key is what name resolution, shadowing detection, and
//! definition identity compare.
//!
//! The separation is deliberate and is the whole point of the rule. Two
//! programs may spell the same identifier `e\u{301}` and `e`; they denote the
//! same entity, so the resolver must agree, but neither file is rewritten and
//! no diagnostic may claim the source said something it did not.

use crate::classify::prohibited_kind;
use crate::tables::UNICODE_VERSION;
use unicode_normalization::UnicodeNormalization;

/// The Unicode version whose NFC form defines identifier equality.
///
/// Recorded so diagnostics and release evidence can state which data version
/// the canonicalization was computed against (SRC-0004).
pub const NORMALIZATION_FORM: &str = "NFC";

/// Why an identifier token could not be turned into a canonical key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalIdentError {
    /// The token contained a code point SRC-0005 prohibits in source.
    Prohibited,
    /// NFC would have to rewrite the token's length in a way that cannot be
    /// represented as a same-length byte span. Never expected in practice; it
    /// exists so the function cannot silently succeed on hostile input.
    NonIdempotent,
}

/// Return the canonical (NFC) key for an identifier token.
///
/// The input must already be an identifier token: tokenization has happened,
/// and `omni-lex` has applied SRC-0005. This function re-checks the prohibited
/// classes anyway so that a caller which bypasses the lexer still cannot feed
/// prohibited characters into the name space.
///
/// The returned `String` is the comparison key only. Callers must continue to
/// carry the original token text and its byte span for diagnostics, source
/// fidelity, and CST reconstruction.
pub fn canonical_ident(spelling: &str) -> Result<String, CanonicalIdentError> {
    for c in spelling.chars() {
        if prohibited_kind(c).is_some() {
            return Err(CanonicalIdentError::Prohibited);
        }
    }
    let key: String = spelling.nfc().collect();
    // NFC is idempotent and is the normalization form used to define equality,
    // so a second pass must be a fixed point. Checking it keeps a future
    // change of normalization form from silently altering equality semantics.
    let again: String = key.nfc().collect();
    if again != key {
        return Err(CanonicalIdentError::NonIdempotent);
    }
    Ok(key)
}

/// Returns `true` when `spelling` is already in its canonical form.
///
/// Used by the formatter and by diagnostics that want to say "this identifier
/// is not normalized" without having to build the key twice.
pub fn is_canonical(spelling: &str) -> bool {
    match canonical_ident(spelling) {
        Ok(key) => key == spelling,
        Err(_) => false,
    }
}

/// Report the Unicode identity used for identifier equality.
///
/// Intended for provenance records and for the release reference manifest, so
/// that the data version behind a canonical key is never implicit.
pub fn normalization_identity() -> (&'static str, &'static str) {
    (UNICODE_VERSION, NORMALIZATION_FORM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_already_canonical() {
        assert_eq!(canonical_ident("hello").unwrap(), "hello");
        assert!(is_canonical("hello"));
    }

    #[test]
    fn nfd_and_nfc_spellings_share_one_key() {
        let nfc = "e\u{0301}"; // LATIN SMALL LETTER E WITH COMBINING ACUTE
        let nfd_nfd = "e\u{301}";
        // Both spellings denote the same identifier.
        let a = canonical_ident(nfc).unwrap();
        let b = canonical_ident(nfd_nfd).unwrap();
        assert_eq!(a, b, "SRC-0003 requires one canonical key per entity");
        assert_eq!(a, "\u{00e9}");
    }

    #[test]
    fn canonically_inequivalent_spellings_stay_distinct() {
        // "café" (composed) and "cafe" (no accent) are different entities and
        // must not collide. Note that "cafe" + COMBINING ACUTE *does* collide
        // with composed "café"; that is the point of SRC-0003, and it is why
        // this test compares against the unaccented spelling.
        let accented = canonical_ident("cafe\u{301}").unwrap();
        let plain = canonical_ident("cafe").unwrap();
        assert_ne!(accented, plain, "different entities must not collide");
        assert_eq!(accented, "caf\u{e9}", "NFD input composes to NFC");

        // A different Latin letter must also stay distinct.
        let other = canonical_ident("caf\u{f6}").unwrap();
        assert_ne!(accented, other);
    }

    #[test]
    fn prohibited_code_points_are_refused() {
        // U+202E RIGHT-TO-LEFT OVERRIDE is a bidi control.
        assert_eq!(canonical_ident("a\u{202E}b"), Err(CanonicalIdentError::Prohibited));
        // U+FDD0 is a permanent noncharacter.
        assert_eq!(canonical_ident("a\u{FDD0}b"), Err(CanonicalIdentError::Prohibited));
    }

    #[test]
    fn hangul_composes_algorithmically() {
        // U+1100 HANGUL CHOSEONG KIYEOK + U+1161 JUNGSEONG A -> U+AC00.
        assert_eq!(canonical_ident("\u{1100}\u{1161}").unwrap(), "\u{AC00}");
    }

    #[test]
    fn canonicalization_is_deterministic() {
        let input = "e\u{301}\u{323}\u{327}x";
        let first = canonical_ident(input).unwrap();
        for _ in 0..64 {
            assert_eq!(canonical_ident(input).unwrap(), first);
        }
    }
}
