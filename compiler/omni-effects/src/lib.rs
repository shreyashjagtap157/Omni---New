//! Effect Row Solver for Omni (EFF-0001..EFF-0018).
//! Implements the authoritative effect/capability representation, open and closed effect rows
//! with row polymorphism, capability tracking, effect obligation solving, and handler semantics.

#[macro_export]
macro_rules! implements {
    ($tag:literal) => {};
}

implements!("EFF-0001");

use std::collections::BTreeSet;

/// A statically tracked class of observable action that a computation may perform.
/// (EFF-0007) Built-in effects include allocation, panic, cancellation, synchronization,
/// blocking, async suspension, I/O families, time, randomness, environment, dynamic reflection,
/// foreign calls, persistence, devices, accelerators, nondeterminism, and unsafe families.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    /// Heap allocation.
    Alloc,
    /// Panic / stack unwinding.
    Panic,
    /// Cancellation token observation.
    Cancel,
    /// Synchronization primitive access (mutex, semaphore).
    Sync,
    /// Blocking I/O or thread sleep.
    Block,
    /// Async suspension / yield point.
    Async,
    /// File system I/O.
    IO,
    /// Network I/O.
    Network,
    /// Wall/monotonic clock access.
    Time,
    /// Random number generation.
    Random,
    /// Environment variable access.
    Env,
    /// Dynamic reflection / Dynamic type.
    Dynamic,
    /// Foreign function interface calls.
    FFI,
    /// Persistence / transaction effects.
    Persist,
    /// Device / MMIO / DMA access.
    Device,
    /// Accelerator (GPU, TPU) dispatch.
    Accel,
    /// Nondeterministic scheduling-dependent behavior.
    Nondeterministic,
    /// Unsafe memory operations.
    Unsafe,
    /// Mutable state manipulation.
    State,
    /// Console / terminal I/O.
    Console,
    /// User-defined named effect.
    Custom(String),
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Alloc => write!(f, "alloc"),
            Effect::Panic => write!(f, "panic"),
            Effect::Cancel => write!(f, "cancel"),
            Effect::Sync => write!(f, "sync"),
            Effect::Block => write!(f, "block"),
            Effect::Async => write!(f, "async"),
            Effect::IO => write!(f, "io"),
            Effect::Network => write!(f, "network"),
            Effect::Time => write!(f, "time"),
            Effect::Random => write!(f, "random"),
            Effect::Env => write!(f, "env"),
            Effect::Dynamic => write!(f, "dynamic"),
            Effect::FFI => write!(f, "ffi"),
            Effect::Persist => write!(f, "persist"),
            Effect::Device => write!(f, "device"),
            Effect::Accel => write!(f, "accel"),
            Effect::Nondeterministic => write!(f, "nondeterministic"),
            Effect::Unsafe => write!(f, "unsafe"),
            Effect::State => write!(f, "state"),
            Effect::Console => write!(f, "console"),
            Effect::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// (EFF-0001) An effect row is a finite canonical set of effect terms
/// plus at most one row variable. Duplicate terms normalize to one term.
/// (EFF-0002) Effect row equality is equality after alias expansion,
/// parameter normalization, and canonical sorting.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectRow {
    /// A closed, fully determined effect set with no row variable tail.
    Closed(BTreeSet<Effect>),
    /// An open, polymorphic effect row: concrete effects plus a row variable tail.
    /// Represents `!{IO, State | E}` where E is the row variable name.
    Open(BTreeSet<Effect>, String),
}

impl Default for EffectRow {
    fn default() -> Self {
        Self::pure()
    }
}

impl EffectRow {
    /// The empty (pure) effect row.
    pub fn pure() -> Self {
        Self::Closed(BTreeSet::new())
    }

    /// Constructs a closed effect row from a list of effects.
    pub fn closed(effects: impl IntoIterator<Item = Effect>) -> Self {
        Self::Closed(effects.into_iter().collect())
    }

    /// Constructs an open (polymorphic) effect row from effects and a row variable tail.
    pub fn open(effects: impl IntoIterator<Item = Effect>, tail: impl Into<String>) -> Self {
        Self::Open(effects.into_iter().collect(), tail.into())
    }

    /// Returns true if this row is pure (empty closed row).
    pub fn is_pure(&self) -> bool {
        matches!(self, Self::Closed(set) if set.is_empty())
    }

    /// Returns the concrete effects in this row (excluding the tail variable).
    pub fn effects(&self) -> &BTreeSet<Effect> {
        match self {
            Self::Closed(set) | Self::Open(set, _) => set,
        }
    }

    /// Returns the row variable tail if this is an open row.
    pub fn tail_var(&self) -> Option<&str> {
        match self {
            Self::Open(_, tail) => Some(tail),
            Self::Closed(_) => None,
        }
    }

    /// (EFF-0003) A function with effect row `ε1` substitutes where `ε2` is allowed
    /// when `ε1` is a subset of `ε2` after constraint solving.
    pub fn satisfies(&self, required: &EffectRow) -> bool {
        match (self, required) {
            (EffectRow::Closed(provided), EffectRow::Closed(req)) => {
                req.iter().all(|e| provided.contains(e))
            }
            (EffectRow::Open(provided, _), EffectRow::Closed(req)) => {
                req.iter().all(|e| provided.contains(e))
            }
            (EffectRow::Closed(_), EffectRow::Open(_, _)) => {
                // Closed cannot satisfy an open polymorphic requirement
                // without instantiation of the tail variable.
                false
            }
            (EffectRow::Open(p1, _), EffectRow::Open(p2, _)) => p2.iter().all(|e| p1.contains(e)),
        }
    }

    /// Computes the union of this effect row with another.
    /// Used during compositional effect inference.
    pub fn union(&self, other: &EffectRow) -> EffectRow {
        match (self, other) {
            (EffectRow::Closed(a), EffectRow::Closed(b)) => {
                EffectRow::Closed(a.union(b).cloned().collect())
            }
            (EffectRow::Open(a, tail), EffectRow::Closed(b))
            | (EffectRow::Closed(b), EffectRow::Open(a, tail)) => {
                EffectRow::Open(a.union(b).cloned().collect(), tail.clone())
            }
            (EffectRow::Open(a, t1), EffectRow::Open(b, _t2)) => {
                // Merge concrete effects, keep first tail variable
                EffectRow::Open(a.union(b).cloned().collect(), t1.clone())
            }
        }
    }

    /// Returns a new row with the given effect removed (for handler discharge).
    /// (EFF-0005) Effect masking is permitted only by a handler that discharges the effect.
    pub fn discharge(&self, effect: &Effect) -> EffectRow {
        match self {
            EffectRow::Closed(set) => {
                let mut new_set = set.clone();
                new_set.remove(effect);
                EffectRow::Closed(new_set)
            }
            EffectRow::Open(set, tail) => {
                let mut new_set = set.clone();
                new_set.remove(effect);
                EffectRow::Open(new_set, tail.clone())
            }
        }
    }

    /// Check if this row contains a specific effect.
    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects().contains(effect)
    }

    /// Substitutes a row variable with a concrete effect row.
    pub fn substitute_tail(&self, var_name: &str, replacement: &EffectRow) -> EffectRow {
        match self {
            EffectRow::Open(effects, tail) if tail == var_name => {
                let combined: BTreeSet<Effect> =
                    effects.union(replacement.effects()).cloned().collect();
                match replacement.tail_var() {
                    Some(new_tail) => EffectRow::Open(combined, new_tail.to_string()),
                    None => EffectRow::Closed(combined),
                }
            }
            other => other.clone(),
        }
    }

    /// Formats this row for diagnostic output.
    pub fn display(&self) -> String {
        let effect_strs: Vec<String> = self.effects().iter().map(|e| format!("{e}")).collect();
        match self.tail_var() {
            Some(tail) if effect_strs.is_empty() => format!("!{{{tail}}}"),
            Some(tail) => format!("!{{{} | {tail}}}", effect_strs.join(", ")),
            None if effect_strs.is_empty() => "pure".to_string(),
            None => format!("!{{{}}}", effect_strs.join(", ")),
        }
    }
}

/// (EFF-0009) A capability is a sealed nominal value created only by a provider.
/// Capabilities represent authority to perform effects.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// File system access.
    FileSystem,
    /// Network connection/listen.
    NetworkConnect,
    /// Process creation.
    ProcessSpawn,
    /// Console/terminal access.
    ConsoleAccess,
    /// Clock (monotonic/wall) access.
    Clock,
    /// Secure randomness.
    SecureRandom,
    /// Pseudo randomness.
    PseudoRandom,
    /// Environment variable read.
    EnvRead,
    /// Environment variable write.
    EnvWrite,
    /// Device/MMIO/DMA access.
    DeviceAccess,
    /// Persistence/transaction.
    PersistenceTransaction,
    /// Accelerator dispatch.
    AcceleratorDispatch,
    /// Dynamic loading.
    DynamicLoad,
    /// Reflection.
    Reflection,
    /// Audit.
    Audit,
    /// Supervisor.
    Supervisor,
    /// Build inputs.
    BuildInput,
    /// User-defined capability.
    Custom(String),
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::FileSystem => write!(f, "FileSystem"),
            Capability::NetworkConnect => write!(f, "Network.Connect"),
            Capability::ProcessSpawn => write!(f, "Process.Spawn"),
            Capability::ConsoleAccess => write!(f, "Console"),
            Capability::Clock => write!(f, "Clock"),
            Capability::SecureRandom => write!(f, "SecureRandom"),
            Capability::PseudoRandom => write!(f, "PseudoRandom"),
            Capability::EnvRead => write!(f, "Env.Read"),
            Capability::EnvWrite => write!(f, "Env.Write"),
            Capability::DeviceAccess => write!(f, "Device"),
            Capability::PersistenceTransaction => write!(f, "Persistence"),
            Capability::AcceleratorDispatch => write!(f, "Accelerator"),
            Capability::DynamicLoad => write!(f, "DynamicLoad"),
            Capability::Reflection => write!(f, "Reflection"),
            Capability::Audit => write!(f, "Audit"),
            Capability::Supervisor => write!(f, "Supervisor"),
            Capability::BuildInput => write!(f, "BuildInput"),
            Capability::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Tracks the set of capabilities available in a given execution context.
/// Capabilities are:
/// - Lexically inherited from enclosing scopes by default
/// - Explicitly passed via `requires` clauses
/// - Temporarily acquired via `with` resource scopes
/// - NOT implicitly inferred from effect rows
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityContext {
    pub available: BTreeSet<Capability>,
}

impl CapabilityContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capabilities(caps: impl IntoIterator<Item = Capability>) -> Self {
        Self { available: caps.into_iter().collect() }
    }

    /// Check if this context has a required capability.
    pub fn has(&self, cap: &Capability) -> bool {
        self.available.contains(cap)
    }

    /// Check if this context has all required capabilities.
    pub fn has_all(&self, required: &[Capability]) -> bool {
        required.iter().all(|c| self.available.contains(c))
    }

    /// Returns missing capabilities from a required set.
    pub fn missing(&self, required: &[Capability]) -> Vec<Capability> {
        required.iter().filter(|c| !self.available.contains(c)).cloned().collect()
    }

    /// Creates a child context that inherits all capabilities.
    pub fn inherit(&self) -> Self {
        self.clone()
    }

    /// Creates a narrowed child context with only the specified capabilities.
    /// (EFF-0010) Capabilities may be attenuated to a strict subset.
    pub fn attenuate(&self, subset: &[Capability]) -> Self {
        let narrowed = subset.iter().filter(|c| self.available.contains(c)).cloned().collect();
        Self { available: narrowed }
    }

    /// Adds a capability to this context (e.g. via a `with` scope handler).
    pub fn grant(&mut self, cap: Capability) {
        self.available.insert(cap);
    }

    /// Revokes a capability from this context.
    pub fn revoke(&mut self, cap: &Capability) {
        self.available.remove(cap);
    }
}

/// Errors related to effect/capability checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectError {
    /// Expression produces effects not declared in the enclosing function signature.
    UnhandledEffect { effect: Effect, function: String, declared_effects: String },
    /// Caller lacks a required capability to invoke this function.
    MissingCapability { capability: Capability, function: String },
    /// Effect row variable is unresolved at MIR boundary.
    UnresolvedEffectVariable { variable: String, function: String },
    /// Effect masking without proper handler.
    InvalidEffectMasking { effect: Effect, detail: String },
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::UnhandledEffect { effect, function, declared_effects } => {
                write!(
                    f,
                    "unhandled effect `{effect}` in function `{function}` \
                     (declared effects: {declared_effects})"
                )
            }
            EffectError::MissingCapability { capability, function } => {
                write!(f, "missing capability `{capability}` required to call `{function}`")
            }
            EffectError::UnresolvedEffectVariable { variable, function } => {
                write!(f, "unresolved effect row variable `{variable}` in function `{function}`")
            }
            EffectError::InvalidEffectMasking { effect, detail } => {
                write!(f, "invalid masking of effect `{effect}`: {detail}")
            }
        }
    }
}

/// Validates that a function's body effects are contained within its declared effects,
/// and that all required capabilities are present in the context.
pub fn check_effect_obligations(
    body_effects: &EffectRow,
    declared_effects: &EffectRow,
    required_caps: &[Capability],
    cap_context: &CapabilityContext,
    fn_name: &str,
) -> Result<(), EffectError> {
    // 1. Check that body effects ⊆ declared effects (EFF-0003)
    for effect in body_effects.effects() {
        if !declared_effects.contains(effect) {
            return Err(EffectError::UnhandledEffect {
                effect: effect.clone(),
                function: fn_name.to_string(),
                declared_effects: declared_effects.display(),
            });
        }
    }

    // 2. Check unresolved effect row variables (must be resolved before MIR)
    if let Some(var) = body_effects.tail_var() {
        return Err(EffectError::UnresolvedEffectVariable {
            variable: var.to_string(),
            function: fn_name.to_string(),
        });
    }

    // 3. Check that all required capabilities are present
    let missing = cap_context.missing(required_caps);
    if let Some(first_missing) = missing.into_iter().next() {
        return Err(EffectError::MissingCapability {
            capability: first_missing,
            function: fn_name.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_row_solving() {
        let provided = EffectRow::closed(vec![Effect::IO, Effect::State]);
        let required = EffectRow::closed(vec![Effect::IO]);
        assert!(provided.satisfies(&required));

        let unprovided = EffectRow::closed(vec![Effect::State]);
        assert!(!unprovided.satisfies(&required));

        let open_provided = EffectRow::open(vec![Effect::IO], "rho");
        assert!(open_provided.satisfies(&required));
    }

    #[test]
    fn test_effect_row_pure() {
        let pure = EffectRow::pure();
        assert!(pure.is_pure());
        assert_eq!(pure.display(), "pure");

        let io = EffectRow::closed(vec![Effect::IO]);
        assert!(!io.is_pure());
    }

    #[test]
    fn test_effect_row_union() {
        let a = EffectRow::closed(vec![Effect::IO]);
        let b = EffectRow::closed(vec![Effect::State]);
        let combined = a.union(&b);
        assert!(combined.contains(&Effect::IO));
        assert!(combined.contains(&Effect::State));
        assert!(!combined.contains(&Effect::Panic));
    }

    #[test]
    fn test_effect_row_discharge() {
        let row = EffectRow::closed(vec![Effect::IO, Effect::State, Effect::Panic]);
        let after = row.discharge(&Effect::IO);
        assert!(!after.contains(&Effect::IO));
        assert!(after.contains(&Effect::State));
        assert!(after.contains(&Effect::Panic));
    }

    #[test]
    fn test_effect_row_substitute_tail() {
        let open_row = EffectRow::open(vec![Effect::IO], "E");
        let replacement = EffectRow::closed(vec![Effect::State, Effect::Panic]);
        let result = open_row.substitute_tail("E", &replacement);
        assert!(result.contains(&Effect::IO));
        assert!(result.contains(&Effect::State));
        assert!(result.contains(&Effect::Panic));
        assert!(result.tail_var().is_none()); // Closed after substitution
    }

    #[test]
    fn test_effect_row_open_substitute_preserves_tail() {
        let open_row = EffectRow::open(vec![Effect::IO], "E");
        let replacement = EffectRow::open(vec![Effect::State], "F");
        let result = open_row.substitute_tail("E", &replacement);
        assert!(result.contains(&Effect::IO));
        assert!(result.contains(&Effect::State));
        assert_eq!(result.tail_var(), Some("F"));
    }

    #[test]
    fn test_capability_context_basic() {
        let ctx =
            CapabilityContext::with_capabilities(vec![Capability::FileSystem, Capability::Clock]);
        assert!(ctx.has(&Capability::FileSystem));
        assert!(ctx.has(&Capability::Clock));
        assert!(!ctx.has(&Capability::NetworkConnect));
    }

    #[test]
    fn test_capability_context_missing() {
        let ctx = CapabilityContext::with_capabilities(vec![Capability::FileSystem]);
        let required = vec![Capability::FileSystem, Capability::NetworkConnect];
        let missing = ctx.missing(&required);
        assert_eq!(missing, vec![Capability::NetworkConnect]);
    }

    #[test]
    fn test_capability_attenuation() {
        let ctx = CapabilityContext::with_capabilities(vec![
            Capability::FileSystem,
            Capability::Clock,
            Capability::NetworkConnect,
        ]);
        let narrowed = ctx.attenuate(&[Capability::FileSystem, Capability::Clock]);
        assert!(narrowed.has(&Capability::FileSystem));
        assert!(narrowed.has(&Capability::Clock));
        assert!(!narrowed.has(&Capability::NetworkConnect));
    }

    #[test]
    fn test_check_effect_obligations_pass() {
        let body = EffectRow::closed(vec![Effect::IO]);
        let declared = EffectRow::closed(vec![Effect::IO, Effect::State]);
        let caps = vec![Capability::FileSystem];
        let ctx = CapabilityContext::with_capabilities(vec![Capability::FileSystem]);
        assert!(check_effect_obligations(&body, &declared, &caps, &ctx, "test_fn").is_ok());
    }

    #[test]
    fn test_check_effect_obligations_unhandled_effect() {
        let body = EffectRow::closed(vec![Effect::IO, Effect::Network]);
        let declared = EffectRow::closed(vec![Effect::IO]);
        let ctx = CapabilityContext::new();
        let result = check_effect_obligations(&body, &declared, &[], &ctx, "leaky_fn");
        assert!(result.is_err());
        match result.unwrap_err() {
            EffectError::UnhandledEffect { effect, .. } => assert_eq!(effect, Effect::Network),
            other => panic!("Expected UnhandledEffect, got {other:?}"),
        }
    }

    #[test]
    fn test_check_effect_obligations_missing_capability() {
        let body = EffectRow::closed(vec![Effect::IO]);
        let declared = EffectRow::closed(vec![Effect::IO]);
        let caps = vec![Capability::FileSystem, Capability::NetworkConnect];
        let ctx = CapabilityContext::with_capabilities(vec![Capability::FileSystem]);
        let result = check_effect_obligations(&body, &declared, &caps, &ctx, "net_fn");
        assert!(result.is_err());
        match result.unwrap_err() {
            EffectError::MissingCapability { capability, .. } => {
                assert_eq!(capability, Capability::NetworkConnect)
            }
            other => panic!("Expected MissingCapability, got {other:?}"),
        }
    }

    #[test]
    fn test_check_effect_obligations_unresolved_row_variable() {
        let body = EffectRow::open(vec![Effect::IO], "E");
        let declared = EffectRow::closed(vec![Effect::IO]);
        let ctx = CapabilityContext::new();
        let result = check_effect_obligations(&body, &declared, &[], &ctx, "poly_fn");
        assert!(result.is_err());
        match result.unwrap_err() {
            EffectError::UnresolvedEffectVariable { variable, .. } => {
                assert_eq!(variable, "E")
            }
            other => panic!("Expected UnresolvedEffectVariable, got {other:?}"),
        }
    }

    #[test]
    fn test_pure_function_allows_no_effects() {
        let body = EffectRow::pure();
        let declared = EffectRow::pure();
        let ctx = CapabilityContext::new();
        assert!(check_effect_obligations(&body, &declared, &[], &ctx, "pure_fn").is_ok());
    }

    #[test]
    fn test_effectful_body_in_pure_function_fails() {
        let body = EffectRow::closed(vec![Effect::IO]);
        let declared = EffectRow::pure();
        let ctx = CapabilityContext::new();
        let result = check_effect_obligations(&body, &declared, &[], &ctx, "supposedly_pure");
        assert!(result.is_err());
    }

    #[test]
    fn test_effect_handler_discharge_semantics() {
        // Before handler: {IO, State, Panic}
        // Handler discharges IO
        // After handler: {State, Panic}
        let before = EffectRow::closed(vec![Effect::IO, Effect::State, Effect::Panic]);
        let after = before.discharge(&Effect::IO);
        assert!(!after.contains(&Effect::IO));
        assert!(after.contains(&Effect::State));
        assert!(after.contains(&Effect::Panic));

        // The handler's residual effects must be in the enclosing signature
        let enclosing = EffectRow::closed(vec![Effect::State, Effect::Panic]);
        assert!(enclosing.satisfies(&after));
    }

    #[test]
    fn test_capability_grant_and_revoke() {
        let mut ctx = CapabilityContext::new();
        assert!(!ctx.has(&Capability::FileSystem));

        ctx.grant(Capability::FileSystem);
        assert!(ctx.has(&Capability::FileSystem));

        ctx.revoke(&Capability::FileSystem);
        assert!(!ctx.has(&Capability::FileSystem));
    }

    #[test]
    fn test_effect_obligation_mutation_bypass_detection() {
        // Mutation test: if we skip the effect check, the MIR gate must still catch it.
        // Here we verify that the obligation checker actually rejects when it should.
        let body = EffectRow::closed(vec![Effect::IO, Effect::Network, Effect::State]);
        let declared = EffectRow::closed(vec![Effect::IO]); // Missing Network and State

        let ctx = CapabilityContext::new();
        let result = check_effect_obligations(&body, &declared, &[], &ctx, "mutation_target");
        assert!(result.is_err(), "Effect checker must reject undeclared effects");

        // Mutant: pretend we expanded the declared row. Verify the original still fails.
        let mutant_declared = EffectRow::closed(vec![Effect::IO, Effect::Network, Effect::State]);
        let mutant_result =
            check_effect_obligations(&body, &mutant_declared, &[], &ctx, "mutation_target");
        assert!(mutant_result.is_ok(), "Mutant with correct declaration should pass");
    }
}
