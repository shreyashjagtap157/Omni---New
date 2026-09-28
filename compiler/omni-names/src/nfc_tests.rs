//! SRC-0003 identifier-equality tests for name resolution.
//!
//! SRC-0003 reads, verbatim from `spec/registry/rule-texts.json`:
//!
//! > Identifier equality uses Unicode NFC after tokenization. String and
//! > character data are never silently normalized.
//!
//! The tests below pin both halves of that sentence. The first is equality:
//! declarations, references, shadowing, and forward references must all agree
//! across canonically equivalent spellings, and must keep canonically
//! *inequivalent* spellings apart. The second is the prohibition on silent
//! normalization: the original spelling and the byte span must survive, and a
//! string or character literal must never be rewritten by the fact that some
//! identifier elsewhere in the file was canonicalized.

use super::*;
use omni_parse::Parser;

/// Parse `src` and resolve it, returning the resolver's view.
fn resolve(src: &str) -> Result<ResolvedNames, Vec<ResolveError>> {
    let mut p = Parser::from_source(src);
    let parsed = p.parse_source();
    assert!(parsed.is_ok(), "fixture must parse: {:?} for {src:?}", parsed.diagnostics);
    let mut r = Resolver::new(1, 2);
    r.resolve_source(&parsed.syntax())
}

fn errors(src: &str) -> Vec<ResolveError> {
    resolve(src).err().unwrap_or_default()
}

// U+00E9 LATIN SMALL LETTER E WITH ACUTE (precomposed)
// U+0065 'e' + U+0301 COMBINING ACUTE ACCENT (decomposed)

#[test]
fn a_canonical_identifier_declares_and_resolves_normally() {
    let r = resolve("fn main() { let x = 1; return x; }").expect("resolve");
    assert!(r.definitions.len() >= 2);
}

#[test]
fn canonically_equivalent_spellings_resolve_to_the_same_identifier() {
    // Declared decomposed, referenced composed: one definition, one reference,
    // the same DefId. This is the core of SRC-0003.
    let r = resolve("fn main() { let e\u{301} = 1; return \u{e9}; }").expect("resolve");
    let defs: Vec<DefId> = r.definitions.values().copied().collect();
    let refs: Vec<DefId> = r.references.values().copied().collect();
    assert!(!defs.is_empty() && !refs.is_empty());
    for d in &defs {
        for rf in &refs {
            // `main` is a definition with no reference here, so require only
            // that the two sets intersect: the local is referenced by the
            // return statement.
            assert!(
                defs.contains(rf) || refs.contains(d),
                "the local binding must be referenced by its canonical form"
            );
        }
    }
    // The reference resolves to a definition, not left dangling.
    assert_eq!(refs.len(), 1, "exactly one reference in this fixture");
    assert!(defs.contains(&refs[0]), "reference must bind to the declared local");
}

#[test]
fn canonically_equivalent_function_names_collide() {
    // Two distinct functions whose names differ only by normalization are the
    // same name, so the second declaration is a duplicate, not a new symbol.
    let errs = errors("fn caf\u{e9}() { return 1; } fn cafe\u{301}() { return 2; }");
    assert!(
        errs.iter().any(|e| matches!(e, ResolveError::ShadowingViolation { .. })),
        "NFC-equal function names must collide, got {errs:?}"
    );
}

#[test]
fn canonically_inequivalent_identifiers_stay_distinct() {
    // "cafe" and "caf�" are different entities and must both exist.
    let r =
        resolve("fn main() { let cafe = 1; let caf\u{e9} = 2; return cafe; }").expect("resolve");
    assert!(!r.definitions.is_empty());
    let errs = errors("fn main() { let cafe = 1; let caf\u{e9} = 2; return cafe; }");
    assert!(errs.is_empty(), "NFC-inequivalent names must not collide, got {errs:?}");
}

#[test]
fn shadowing_detection_obeys_canonical_equality() {
    // The second binding collides with the first because the two spellings are
    // canonically equal.
    let errs = errors("fn main() { let \u{e9} = 1; let e\u{301} = 2; return \u{e9}; }");
    assert!(
        errs.iter().any(|e| matches!(e, ResolveError::ShadowingViolation { .. })),
        "shadowing must use the canonical key, got {errs:?}"
    );
}

#[test]
fn forward_references_obey_canonical_equality() {
    // The call target is declared later and spelled canonically differently.
    let r = resolve("fn main() { return caf\u{e9}(); } fn cafe\u{301}() { return 1; }")
        .expect("a forward reference across spellings must resolve");
    assert!(!r.references.is_empty());
}

#[test]
fn a_reference_may_use_a_different_canonically_equivalent_spelling() {
    let r = resolve("fn main() { let caf\u{e9} = 1; return cafe\u{301}; }")
        .expect("declaration and reference may differ by normalization");
    let refs: Vec<DefId> = r.references.values().copied().collect();
    let defs: Vec<DefId> = r.definitions.values().copied().collect();
    assert_eq!(refs.len(), 1);
    assert!(defs.contains(&refs[0]), "the reference must bind to the declared local");
}

#[test]
fn an_unresolved_reference_is_still_reported() {
    // Canonicalization must not make unknown names resolvable.
    let errs = errors("fn main() { let x = 1; return caf\u{e9}; }");
    assert!(
        errs.iter().any(|e| matches!(e, ResolveError::UnresolvedName { .. })),
        "an unknown name must not resolve, got {errs:?}"
    );
}
// --------------------------------------------------------------------------
// The prohibition on silent normalization
// --------------------------------------------------------------------------

#[test]
fn the_original_spelling_survives_into_the_resolved_names() {
    // The diagnostic-facing name must be what the source said, not the NFC key.
    let r = resolve("fn main() { let cafe\u{301} = 1; return cafe\u{301}; }").expect("resolve");
    let spellings: Vec<&String> = r.names.values().collect();
    assert!(
        spellings.iter().any(|s| s.as_str() == "cafe\u{301}"),
        "the decomposed spelling must be preserved for diagnostics, got {spellings:?}"
    );
    assert!(
        !spellings.iter().any(|s| s.as_str() == "caf\u{e9}"),
        "the spelling must not be rewritten to NFC, got {spellings:?}"
    );
}

#[test]
fn canonicalization_does_not_alter_string_literals() {
    // A string literal containing the same characters must be untouched: SRC-0003
    // says string data is never silently normalized.
    let src = "fn main() { let cafe\u{301} = \"cafe\u{301}\"; return cafe\u{301}; }";
    let mut p = Parser::from_source(src);
    let parsed = p.parse_source();
    assert!(parsed.is_ok(), "{:?}", parsed.diagnostics);
    assert!(
        parsed.syntax().text().to_string().contains("\"cafe\u{301}\""),
        "the string literal must keep its original bytes"
    );
    assert_eq!(parsed.syntax().text().to_string(), src, "source round-trip must be exact");
}

#[test]
fn canonicalization_does_not_alter_character_literals() {
    let src = "fn main() { let e\u{301} = '\u{e9}'; return \u{e9}; }";
    let mut p = Parser::from_source(src);
    let parsed = p.parse_source();
    assert!(parsed.is_ok(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.syntax().text().to_string(), src, "source round-trip must be exact");
    assert!(parsed.syntax().text().to_string().contains("'\u{e9}'"));
}

#[test]
fn canonicalization_does_not_alter_raw_strings() {
    let src = "fn main() { let s = r\"cafe\u{301}\"; return s; }";
    let mut p = Parser::from_source(src);
    let parsed = p.parse_source();
    assert!(parsed.is_ok(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.syntax().text().to_string(), src);
    assert!(parsed.syntax().text().to_string().contains("r\"cafe\u{301}\""));
}

#[test]
fn canonicalization_does_not_alter_comments() {
    let src = "// cafe\u{301} in a comment\nfn main() { return 1; }\n";
    let mut p = Parser::from_source(src);
    let parsed = p.parse_source();
    assert!(parsed.is_ok(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.syntax().text().to_string(), src, "comments must be byte-exact");
}

#[test]
fn canonicalization_does_not_alter_arbitrary_source_bytes() {
    // The lexer/parser path is byte-preserving for every fixture containing an
    // identifier that requires canonicalization.
    for src in [
        "fn main() { let e\u{301} = 1; return e\u{301}; }",
        "fn main() { let caf\u{e9} = \"x\"; return caf\u{e9}; }",
        "// c\u{301}\nfn main() { let a\u{301} = 1; return a\u{301}; }",
    ] {
        let mut p = Parser::from_source(src);
        let parsed = p.parse_source();
        assert!(parsed.is_ok(), "{:?} for {src:?}", parsed.diagnostics);
        assert_eq!(parsed.syntax().text().to_string(), src, "byte-exact round-trip for {src:?}");
    }
}

#[test]
fn prohibited_code_points_never_reach_the_name_space() {
    // A bidi control in a name must be refused even if a caller reaches name
    // resolution without going through the lexer.
    let mut r = Resolver::new(1, 2);
    assert!(matches!(r.declare("a\u{202E}b"), Err(ResolveError::ProhibitedIdentifier { .. })));
    assert!(matches!(r.resolve("a\u{202E}b"), Err(ResolveError::ProhibitedIdentifier { .. })));
}

// --------------------------------------------------------------------------
// Determinism
// --------------------------------------------------------------------------

#[test]
fn canonical_resolution_is_deterministic() {
    let src = "fn main() { let caf\u{e9} = 1; return cafe\u{301}; }";
    // `HashMap` iteration order is randomised per instance, so compare the
    // contents in a stable order. What the test pins is that the same source
    // yields the same definition/reference mapping every time.
    let shape = |r: &ResolvedNames| {
        let mut defs: Vec<(u32, u32)> = r.definitions.iter().map(|(k, v)| (*k, v.index)).collect();
        let mut refs: Vec<(u32, u32)> = r.references.iter().map(|(k, v)| (*k, v.index)).collect();
        defs.sort_unstable();
        refs.sort_unstable();
        (defs, refs)
    };
    let first = shape(&resolve(src).expect("resolve"));
    for _ in 0..16 {
        assert_eq!(
            shape(&resolve(src).expect("resolve")),
            first,
            "resolution must be deterministic"
        );
    }
}

#[test]
fn the_canonical_name_type_keeps_both_representations() {
    let c = CanonicalName::new("cafe\u{301}").expect("canonical");
    assert_eq!(c.spelling(), "cafe\u{301}", "spelling is preserved");
    assert_eq!(c.key(), "caf\u{e9}", "key is NFC");
    assert!(!c.is_canonical(), "the decomposed spelling is not canonical");

    let c2 = CanonicalName::new("caf\u{e9}").expect("canonical");
    assert_eq!(c2.key(), c.key(), "equivalent spellings share one key");
    assert!(c2.is_canonical(), "the composed spelling is already canonical");
}

#[test]
fn canonical_name_rejects_prohibited_code_points() {
    assert!(CanonicalName::new("a\u{FDD0}").is_err());
    assert!(CanonicalName::new("a\u{202E}").is_err());
}
