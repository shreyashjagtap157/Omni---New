//! Mid-Level Intermediate Representation for Omni.

pub mod assume;
pub mod concurrency;
pub mod continuation;
pub mod drop;
pub mod ir;
pub mod lower;

pub use omni_types::ast;
pub use omni_types::checker::SubstEnv;
pub use omni_types::intern::{Ty, TyCtxt, TyKind};
pub use omni_types::monomorph::MonomorphizedProgram;

// Effect vocabulary belongs to the typed MIR boundary. Re-exporting it here lets
// below-MIR consumers reach effects through their declared feed instead of
// depending on `omni-effects` directly, which the topology gate forbids.
pub use omni_effects::{CapabilityContext, Effect, EffectRow};
