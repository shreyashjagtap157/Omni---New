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

A source review of `compiler/omni-verify/src/ownership.rs` identified an unqualified reference-provenance
transfer path. `FlowState.reference_loans` maps MIR locals to loan-region identifiers. Assignments using
`Rvalue::Use(Operand::Copy(place))` perform an ordinary read but do not transfer the source local's
loan associations to the destination. The move path removes the source associations and ends those loans
immediately. Loan shortening also reasons about dead locals individually rather than explicitly
preserving a region while another live local still carries the same region. The state does not expose a
canonical loan-to-place provenance map for resolving dereference accesses against the borrowed storage.

The regression coverage now constructs typed MIR in which a mutable reference is moved to a new local.
A write to the borrowed place before the destination's last use must be rejected, while a write after
that last use must be accepted. The verifier's root-local move path transfers the tracked loan association
to the destination, and loan release checks for remaining reference associations and active child
reborrows before ending a region. This qualifies the direct root-local move case; projected reference
storage, reference arguments/returns, and dereference-to-origin resolution remain outstanding.

The broader verifier must continue to reject invalid MIR and must not make valid programs fail solely
because the analysis is conservative. CFG joins, nested reborrows, aggregate-contained references, and
all call/return provenance paths still require dedicated qualification.

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
