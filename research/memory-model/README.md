# Memory-model research spike

**Status:** Isolated executable model; not production implementation.

This directory tests a narrow hypothesis from docs/research/memory-architecture-managed-vs-affine.md: storage reclamation, alias/access permissions, and deterministic resource cleanup can be represented as separate interacting mechanisms.

The prototype contains:
- a dynamic shared/exclusive access ledger;
- a region store with bulk region termination;
- a strong-reference-counted graph with no cycle collector;
- a tracing graph collector that can reclaim unreachable cycles;
- a mixed-domain graph that rejects tracing-to-region lifetime escape and treats live-region edges as roots into the tracing heap;
- a deterministic resource token whose lifetime is independent of a separately modeled tracing object.

It intentionally does not implement a language type checker, sound lifetime inference, concurrent collector, weak references, nested-region subtyping, automatic promotion, arbitrary domain crossings, a native allocator, or ABI. The mixed-domain tests encode one conservative lifetime policy; they do not prove it is the best or complete Omni policy. The tests therefore validate only the model's encoded transitions. They do not select a language architecture or establish production memory safety.

## Reproduce

From the repository root with the pinned Rust 1.95.0 toolchain:

    rustfmt +1.95.0 --edition 2021 --check research/memory-model/spike.rs
    rustc +1.95.0 --edition=2021 --test -D warnings research/memory-model/spike.rs -o /tmp/omni-memory-model-spike
    /tmp/omni-memory-model-spike --test-threads=1

The main CI workflow runs the same formatting and test commands. The spike is outside the production Cargo workspace and introduces no dependencies.
