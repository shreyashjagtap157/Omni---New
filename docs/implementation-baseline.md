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
- a canonical HIR boundary consumed directly by MIR lowering;
- typed MIR with places, projections, control flow, calls, drops, bounds checks, and unsafe-region
  metadata;
- an independently invoked MIR verifier;
- a reference abstract-machine interpreter;
- deterministic Cranelift native object generation;
- Stage 4D bounds-verification work;
- Stage 4E aggregate ABI work;
- driver integration for reference-machine and native paths.

Important remaining qualification work includes richer canonical HIR typing/provenance and semantic
obligation preservation, complete ownership/lifetime integration into the production pipeline,
complete effects/capability enforcement, full MIR drop/initialization/assumption semantics, completion
of the reference-machine memory/execution model, minimal runtime/platform support, real differential
qualification across the supported observation set, and later package/tooling/self-hosting work.

The initial baseline commit and its claims must not be reused as evidence that the current tree is
still a scaffold. Current implementation status is determined by the actual source tree, tests,
specifications, CI, and qualification evidence.
