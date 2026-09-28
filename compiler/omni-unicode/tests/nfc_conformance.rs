//! SRC-0004 conformance: the pinned `unicode-normalization` dependency must
//! agree with the pinned UCD 17.0.0 data.
//!
//! `unicode-normalization` publishes no Unicode version of its own, so "add a
//! Unicode crate" alone would leave the data version unpinned and the release
//! identity unverifiable. This test closes that gap mechanically: the corpus in
//! `nfc_cases.rs` is generated from UCD 17.0.0 `UnicodeData.txt` and
//! `CompositionExclusions.txt`, and every case asserts the UAX #15 requirement
//! that NFC recomposes a canonical decomposition back to its composed form.
//!
//! If a future dependency bump changes the crate's Unicode data, at least one
//! of these cases fails here rather than silently changing identifier equality
//! for some obscure code point.

use omni_unicode::canonical_ident;
use unicode_normalization::UnicodeNormalization;

#[path = "nfc_cases.rs"]
mod cases;

use cases::NFC_CASES;

/// The scalar value of the first character, for diagnostics.
fn first_cp(s: &str) -> u32 {
    s.chars().next().map_or(0, |c| c as u32)
}

#[test]
fn the_conformance_corpus_is_substantial() {
    assert!(
        NFC_CASES.len() > 900,
        "the conformance corpus must be substantial, got {}",
        NFC_CASES.len()
    );
}

#[test]
fn nfc_recomposes_every_ucd_canonical_decomposition() {
    for (decomposed, composed) in NFC_CASES {
        let got: String = decomposed.nfc().collect();
        assert_eq!(&got, composed, "NFC must recompose U+{:04X} per UAX #15", first_cp(composed));
    }
}

#[test]
fn identifier_canonicalization_agrees_with_the_ucd_data() {
    // The compiler's canonical key must be the same NFC the UCD requires, not a
    // different or absent normalization.
    for (decomposed, composed) in NFC_CASES {
        let key = canonical_ident(decomposed).expect("decomposed form is permitted");
        assert_eq!(
            key,
            *composed,
            "canonical_ident must match UCD 17.0.0 NFC for U+{:04X}",
            first_cp(composed)
        );
    }
}

#[test]
fn nfc_is_idempotent_over_the_conformance_corpus() {
    for (decomposed, composed) in NFC_CASES {
        let once: String = decomposed.nfc().collect();
        let twice: String = once.nfc().collect();
        assert_eq!(once, twice, "NFC must be idempotent (UAX #15)");
        assert_eq!(&once, composed);
    }
}

#[test]
fn nfd_of_a_composed_form_matches_the_ucd_decomposition() {
    // The UCD stores each character's *direct* canonical decomposition, which is
    // not necessarily fully recursive: U+01D5 is listed as `00DC 0304`, but
    // NFD is recursive, so it yields `0055 0308 0304`. Comparing the two fully
    // decomposed forms is therefore the correct assertion.
    for (decomposed, composed) in NFC_CASES {
        let nfd_composed: String = composed.nfd().collect();
        let nfd_decomposed: String = decomposed.nfd().collect();
        assert_eq!(
            nfd_composed,
            nfd_decomposed,
            "NFD of U+{:04X} must match the UCD decomposition",
            first_cp(composed)
        );
    }
}
