//! Unicode data identity, source-security classification, and identifier
//! canonicalization for the Omni compiler.
//!
//! This crate is the single authoritative implementation of the OMNI-SOURCE
//! Unicode obligations. It exists so that exactly one subsystem answers
//! "is this code point prohibited here?" and "what is this identifier's
//! canonical key?", rather than each downstream tier re-deriving the answer.
//!
//! # Rule ownership
//!
//! * `SRC-0003` - identifier equality uses Unicode NFC after tokenization,
//!   implemented in [`canonical_ident`]. String and character data are never
//!   silently normalized; nothing in this crate rewrites literal contents.
//! * `SRC-0004` - Edition 1 pins Unicode 17.0.0 together with UAX #15, #31,
//!   #9, #24, #44 and UTS #39 Revision 32. The identity is recorded in
//!   [`tables`] (generated from the UCD) and in the release reference manifest.
//! * `SRC-0005` - prohibited code points outside comments and literals are
//!   source errors, implemented in [`classify`].
//! * `SRC-0006` - bidi controls and invisible format characters inside comments
//!   require an escaped visible annotation, and strict mode rejects
//!   unannotated occurrences, implemented in [`annotation`].
//!
//! # Determinism
//!
//! Every function here is a pure function of its input. Classification never
//! consults the host locale, never performs network access, and never reads
//! platform Unicode data; the committed tables in [`tables`] are the only data
//! source.

pub mod annotation;
pub mod classify;
pub mod ident;
pub mod tables;

pub use annotation::{
    annotate_comment, scan_comment_security, visible_annotation, CommentSecurity, SecurityMode,
};
pub use classify::{is_prohibited, prohibited_kind, ProhibitedKind, PROHIBITED_ORDER};
pub use ident::{canonical_ident, is_canonical, normalization_identity, CanonicalIdentError};
pub use tables::{
    DERIVED_CORE_PROPERTIES_SHA256, PROPLIST_SHA256, UNASSIGNED, UNICODE_DATA_SHA256,
    UNICODE_VERSION,
};

/// UAX #31 identifier classes are supplied by `unicode-ident`.
///
/// The lexer already depends on it, and reusing one source of truth for
/// `XID_Start`/`XID_Continue` keeps UAX #31 (pinned by SRC-0004) consistent
/// between tokenization and canonicalization.
pub mod xid {
    pub use unicode_ident::{is_xid_continue, is_xid_start};
}
