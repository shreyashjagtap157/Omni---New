//! Name resolution and canonical identifier management for Omni.

pub mod def_id;
pub mod resolve;

pub use def_id::DefId;
pub use resolve::{ResolveError, Resolver, Rib};
