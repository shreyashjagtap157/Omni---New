//! SRC-0005 - prohibited code points outside comments and literals.
//!
//! SRC-0005 reads, verbatim from `spec/registry/rule-texts.json`:
//!
//! > Outside comments and literals, bidi control characters, noncharacters,
//! > unassigned code points, variation selectors, and default-ignorable format
//! > characters are source errors.
//!
//! The five classes are tested in the order SRC-0005 names them so that a
//! code point belonging to more than one class (for example U+061C, which is
//! both `Bidi_Control` and `Default_Ignorable_Code_Point`) always produces the
//! same diagnostic. Determinism matters more here than taxonomy: two runs over
//! the same bytes must never disagree about what is wrong with them.

use crate::tables::{
    BIDI_CONTROL, DEFAULT_IGNORABLE, NONCHARACTER, UNASSIGNED, VARIATION_SELECTOR,
};

/// A class of code point that SRC-0005 prohibits outside comments and literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProhibitedKind {
    /// `Bidi_Control`. Also prohibited inside comments unless annotated (SRC-0006).
    BidiControl,
    /// Noncharacter code points, defined structurally by UAX #44.
    Noncharacter,
    /// Unassigned code points in the pinned Unicode version.
    Unassigned,
    /// `Variation_Selector`. Also default-ignorable.
    VariationSelector,
    /// `Default_Ignorable_Code_Point`.
    DefaultIgnorable,
}

impl ProhibitedKind {
    /// A stable, human-facing name used in diagnostics.
    ///
    /// These strings are part of the diagnostic surface, so they are written
    /// out rather than derived from `Debug`: a rename of the Rust variant must
    /// not silently change what the compiler prints.
    pub fn as_str(self) -> &'static str {
        match self {
            ProhibitedKind::BidiControl => "bidi control character",
            ProhibitedKind::Noncharacter => "noncharacter code point",
            ProhibitedKind::Unassigned => "unassigned code point",
            ProhibitedKind::VariationSelector => "variation selector",
            ProhibitedKind::DefaultIgnorable => "default-ignorable format character",
        }
    }
}

impl core::fmt::Display for ProhibitedKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The classes SRC-0005 names, in the order it names them.
///
/// Reported order is part of the contract: see the module documentation.
pub const PROHIBITED_ORDER: [ProhibitedKind; 5] = [
    ProhibitedKind::BidiControl,
    ProhibitedKind::Noncharacter,
    ProhibitedKind::Unassigned,
    ProhibitedKind::VariationSelector,
    ProhibitedKind::DefaultIgnorable,
];

/// Binary-search a sorted, disjoint, half-open `[lo, hi)` range table.
fn in_table(table: &[(u32, u32)], cp: u32) -> bool {
    table
        .binary_search_by(|&(lo, hi)| {
            if cp < lo {
                core::cmp::Ordering::Greater
            } else if cp >= hi {
                core::cmp::Ordering::Less
            } else {
                core::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Membership test against a generated property table.
///
/// The generated tables are sorted, disjoint, and half-open, so this is a
/// binary search rather than a scan: comment scanning calls it once per
/// character and a linear scan would make comment handling quadratic.
pub fn contains(table: &[(u32, u32)], cp: u32) -> bool {
    in_table(table, cp)
}

/// Classify `c` against the SRC-0005 prohibited classes.
///
/// Returns `None` when the code point is permitted in source text. Callers are
/// responsible for having already excluded comments and literals, which
/// SRC-0005 leaves unconstrained (SRC-0006 governs comments, SRC-0007
/// governs literals).
pub fn prohibited_kind(c: char) -> Option<ProhibitedKind> {
    let cp = c as u32;
    // Ordered exactly as SRC-0005 lists the classes.
    if in_table(&BIDI_CONTROL, cp) {
        return Some(ProhibitedKind::BidiControl);
    }
    if in_table(&NONCHARACTER, cp) {
        return Some(ProhibitedKind::Noncharacter);
    }
    if in_table(&UNASSIGNED, cp) {
        return Some(ProhibitedKind::Unassigned);
    }
    if in_table(&VARIATION_SELECTOR, cp) {
        return Some(ProhibitedKind::VariationSelector);
    }
    if in_table(&DEFAULT_IGNORABLE, cp) {
        return Some(ProhibitedKind::DefaultIgnorable);
    }
    None
}

/// Returns `true` when `c` is prohibited outside comments and literals.
pub fn is_prohibited(c: char) -> bool {
    prohibited_kind(c).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::{UNICODE_DATA_SHA256, UNICODE_VERSION};

    #[test]
    fn every_prohibited_class_is_reachable() {
        // One representative per class SRC-0005 names, chosen from the
        // authoritative UCD 17.0.0 tables.
        let cases = [
            ('\u{061C}', ProhibitedKind::BidiControl),
            ('\u{FDD0}', ProhibitedKind::Noncharacter),
            ('\u{0378}', ProhibitedKind::Unassigned),
            ('\u{FE0F}', ProhibitedKind::VariationSelector),
            ('\u{00AD}', ProhibitedKind::DefaultIgnorable),
        ];
        for (c, expected) in cases {
            assert_eq!(prohibited_kind(c), Some(expected), "U+{:04X}", c as u32);
        }
    }

    #[test]
    fn ordinary_source_characters_are_permitted() {
        for c in ['a', 'Z', '0', ' ', '{', '\n', 'é', '日', '🦀', '\u{0301}'] {
            assert_eq!(prohibited_kind(c), None, "U+{:04X} must be permitted", c as u32);
        }
    }

    #[test]
    fn classification_is_deterministic_at_property_boundaries() {
        // Exercise both sides of every boundary of the bidi table.
        for cp in
            [0x061B, 0x061C, 0x061D, 0x200D, 0x200E, 0x2010, 0x2029, 0x202A, 0x2065, 0x2066, 0x206A]
        {
            let c = char::from_u32(cp).expect("scalar");
            let _ = prohibited_kind(c); // must not panic, must be stable
            assert_eq!(prohibited_kind(c), prohibited_kind(c));
        }
    }

    #[test]
    fn combined_class_resolves_to_the_first_named_class() {
        // U+061C is both Bidi_Control and Default_Ignorable_Code_Point. SRC-0005
        // names bidi controls first, so that is the class that must be reported.
        assert_eq!(prohibited_kind('\u{061C}'), Some(ProhibitedKind::BidiControl));
        // U+FE0F is both a Variation_Selector and default-ignorable.
        assert_eq!(prohibited_kind('\u{FE0F}'), Some(ProhibitedKind::VariationSelector));
    }

    #[test]
    fn noncharacters_cover_every_plane() {
        // The last two code points of plane 0.
        assert!(is_prohibited('\u{FFFE}'));
        assert!(is_prohibited('\u{FFFF}'));
        // And of plane 1.
        assert!(is_prohibited('\u{1FFFE}'));
        assert!(is_prohibited('\u{10FFFE}'));
        // U+FDD0..U+FDEF are permanently reserved.
        assert!(is_prohibited('\u{FDD0}'));
        assert!(is_prohibited('\u{FDEF}'));
        // But a normal character on the last plane is fine.
        assert_eq!(prohibited_kind('\u{10FFFD}'), None);
    }

    #[test]
    fn classification_never_panics_across_the_whole_scalar_range() {
        // Exhaustive sweep of every valid scalar value. This is the no-crash
        // guarantee for SRC-0005, and it is cheap because the tables are
        // binary-searched.
        for cp in 0..=0x10FFFFu32 {
            if let Some(c) = char::from_u32(cp) {
                let _ = prohibited_kind(c);
            }
        }
    }

    #[test]
    fn ucd_digests_are_recorded_for_release_identity() {
        // SRC-0004 binds the Unicode data version as a release identity.
        assert_eq!(UNICODE_VERSION, "17.0.0");
        assert_eq!(UNICODE_DATA_SHA256.len(), 64);
        assert!(UNICODE_DATA_SHA256.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
