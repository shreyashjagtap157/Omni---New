# Memory-model research spike

**Status:** Isolated executable model; not production implementation.

This directory tests a narrow hypothesis from docs/research/memory-architecture-managed-vs-affine.md: storage reclamation, alias/access permissions, and deterministic resource cleanup can be represented as separate interacting mechanisms.

The prototype contains:
- a dynamic shared/exclusive access ledger;
- a region store with bulk region termination;
- a strong-reference-counted graph with no cycle collector;
- a tracing graph collector that can reclaim unreachable cycles;
- a mixed-domain graph that rejects tracing-to-region lifetime escape, treats live-region edges as roots into the tracing heap, and allows only references to the same or an outer region;
- a deterministic resource token whose lifetime is independent of a separately modeled tracing object.

It intentionally does not implement a language type checker, sound lifetime inference, concurrent collector, weak references, general region subtyping, automatic promotion, closure/task escape analysis, arbitrary domain crossings, a native allocator, or ABI. The mixed-domain tests encode one conservative lifetime policy; they do not prove it is the best or complete Omni policy. The tests therefore validate only the model's encoded transitions. They do not select a language architecture or establish production memory safety.

## Reproduce

From the repository root with the pinned Rust 1.95.0 toolchain:

    rustfmt +1.95.0 --edition 2021 --check research/memory-model/spike.rs
    rustc +1.95.0 --edition=2021 --test -D warnings research/memory-model/spike.rs -o /tmp/omni-memory-model-spike
    /tmp/omni-memory-model-spike --test-threads=1

The main CI workflow runs the same formatting and test commands. The spike is outside the production Cargo workspace and introduces no dependencies.

## Domain lifetime and promotion experiment

The second prototype isolates closure/task capture lifetime checking, capture-held tracing roots, shared/exclusive access among logical tasks, per-object access reservations across independent capture environments, and explicit deep-copy promotion of region graphs into tracing storage.

Reproduce with the pinned toolchain:

    rustfmt +1.95.0 --edition 2021 --check research/memory-model/domain_lifetimes.rs
    rustc +1.95.0 --edition=2021 --test -D warnings research/memory-model/domain_lifetimes.rs -o /tmp/omni-domain-lifetimes
    /tmp/omni-domain-lifetimes --test-threads=1

This model is also not production memory management, a task scheduler, or a complete concurrency/memory model. In particular, its promotion routine is a deterministic graph-copy model with an explicit allocation budget, not an OOM-safe allocator.


The second model also rejects exclusive/move capture reservations that overlap any existing live capture of the same object, permits multiple shared capture leases, and removes leases when an environment is released. Its move mode remains a reservation model: caller-side source-place invalidation is not yet represented.


## Pinned/external-resource lifetime experiment

The domain model now has an explicit object-pinning bit and object-owned external-resource set. Attaching an external resource transfers its logical ownership to the object and pins it. Closing the resource is exact-once, but storage remains pinned until an explicit unpin. Region teardown and region-to-tracing promotion reject pinned or resource-bearing nodes. Pinned tracing nodes are roots in the prototype collector.

On code commit \`3902f6d\`, the complete domain prototype passed 22 tests. The model does not emulate physical address pinning, an OS/device resource, DMA coherence, or a native FFI boundary. Its pin/resource state is a semantic experiment only.
