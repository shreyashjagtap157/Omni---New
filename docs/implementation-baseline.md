# Omni implementation baseline

The repository's initial implementation baseline is preserved in Git history. It is historical
context, not a description of the current implementation state.

The generational release gates remain:

- `1.0.0.0` — minimal Rust-core implementation;
- `2.0.0.0` — complete production Rust platform;
- `3.0.0.0` — certified self-hosted and independently qualified Omni.

The current repository has progressed substantially beyond the initial foundation baseline. The
active implementation path now includes:

- lossless source/lexical and CST/parser infrastructure;
- Edition 1 semantic name/type infrastructure;
- ownership, effects, capability, trait, and monomorphization infrastructure;
- a canonical HIR boundary consumed directly by MIR lowering, preserving source enum declarations;
- typed MIR with places, projections, control flow, calls, drops, bounds checks, enum declaration
  metadata, and unsafe-region metadata;
- an independently invoked MIR verifier;
- a reference abstract-machine interpreter;
- deterministic Cranelift native object generation;
- Stage 4D bounds-verification work;
- Stage 4E aggregate ABI work;
- driver integration for reference-machine and native paths.

The MIR verifier checks enum constructor declaration identity, generic-argument arity, variant
existence, and payload arity against declarations carried from HIR. MIR has an explicit, non-consuming
enum-variant branch terminator and a typed `Rvalue::EnumField` carrying enum/variant identity. Lowering
and driver tests cover wildcard payload matching, payload bindings, ordered payload-literal matching,
and nested enum payload binding through verified MIR and the reference machine. Native enum construction
and dispatch remain fail-closed because no target tagged-layout contract is in force.

Important remaining qualification work includes richer canonical HIR typing/provenance and semantic
obligation preservation, complete ownership/lifetime integration into the production pipeline,
complete effects/capability enforcement, full MIR drop/initialization/assumption semantics, completion
of the reference-machine memory/execution model, minimal runtime/platform support, real differential
qualification across the supported observation set, and later package/tooling/self-hosting work.

### Next ownership/lifetime qualification blocker: reference provenance

The reference-provenance model remains incomplete. `FlowState.reference_loans` maps MIR locals to
loan-region identifiers, so it cannot yet represent references stored in projected places or aggregates,
or resolve a dereference back to its borrowed origin. `Rvalue::Use(Operand::Copy(place))` does not
propagate loan associations to the destination; copy propagation remains blocked until the normative
reference-copyability rule is resolved. Moves into projected/aggregate storage and through call
arguments/returns are also not fully represented.

The root-local assignment move path is now qualified: it transfers tracked loan associations to the
destination, and loan release checks remaining reference-local associations and active child reborrows,
including cascading release of now-unreferenced parents. Typed-MIR regressions reject a write to the
borrowed place before the moved destination's last use and accept a write after that last use. This does
not qualify the remaining projected-place, aggregate, dereference-origin, or call/return paths.

The broader verifier must continue to reject invalid MIR and must not make valid programs fail solely
because the analysis is conservative. CFG joins, nested reborrows, aggregate-contained references, and
all call/return provenance paths still require dedicated qualification.

### Specification authority caveat

`spec/reference/core_semantics_and_types.md` labels itself “Normative Candidate Reference for Edition 1”,
but `spec/reference/README.md` classifies `reference/` as an exhaustive reference domain outside the
normative hashed domain. The normative registry's `OWN-0001` and `OWN-0004` define affine behavior for
non-`Copy` values and eligibility constraints for `Copy`, but do not explicitly define reference-specific
`Copy` implementations. Resolve the authority/precedence mismatch before treating the compendium as the
source of a new normative rule.

The official Rust Reference documents one feasible precedent: shared references are copyable independently
of their referent type, while mutable references are not copyable to prevent aliased mutable references
(https://doc.rust-lang.org/reference/types/pointer.html). This is comparative evidence, not an Omni rule.
No option is selected here, and no normative spec hash or grammar was changed.

Before implementing copy propagation, resolve this normative question: **is a shared reference `&T`
copyable, or are all reference values affine unless an explicit Copy-like rule says otherwise?** The
current Edition 1 core-semantics section 5.1 defines by-value ownership transfer, and section 5.2 defines
shared/exclusive access restrictions, but neither explicitly settles reference copyability or whether it
is trait-governed. The decision options are:

1. Shared references are copyable, with shared loan ownership tracked across every live alias; mutable
   references remain affine unless the normative rules explicitly provide otherwise.
2. All references are affine and move-only, giving the simplest provenance model but restricting shared
   reference reuse.
3. Copyability is determined by an explicit normative trait/rule, with reference-specific constraints
   stated in the specification.

Do not select an option from convention alone. Check the complete in-force Edition 1 specification and
compatibility requirements, record the decision, and update normative text only after that review.

The initial baseline commit and its claims must not be reused as evidence that the current tree is
still a scaffold. Current implementation status is determined by the actual source tree, tests,
specifications, CI, and qualification evidence.
