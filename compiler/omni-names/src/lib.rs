//! Name resolution and canonical identifier management for Omni.

pub mod def_id;
#[cfg(test)]
mod nfc_tests;
pub mod resolve;

pub use def_id::DefId;
pub use resolve::{CanonicalName, ResolveError, ResolvedNames, Resolver, Rib};
