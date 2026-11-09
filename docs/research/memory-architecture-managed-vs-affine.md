# Memory Architecture Research: Managed Memory, Affine Ownership, and Composable Domains

**Date:** 2026-11-09  
**Status:** Research in progress; no language-design decision or normative amendment  
**Evidence:** Repository facts directly inspected; literature claims sourced; candidates PROPOSED; unifying hypothesis H1.

## 1. Executive finding

Omni has an unresolved requirement conflict that must not be erased:

- Historical user intent records managed memory as the default, with ownership, linearity, and regions as opt-in facilities.
- The current Edition 1 core reference describes affine ownership as the fundamental model, with shared/exclusive borrowing, non-lexical lifetimes, and deterministic destruction.

The current repository implements an affine place/loan analysis. Its runtime allocation services are opaque-handle byte stores, not a tracing collector, general reference-counting heap, or region-inference system.

A useful research lead is to separate three related questions:

1. **Storage reclamation:** when backing storage can be reused (stack/region end, explicit owner destruction, reference counting, tracing reachability, or an external allocator).
2. **Access and alias discipline:** who may read or mutate an object at a program point (unique/affine access, shared immutable access, exclusive mutable access, or synchronized shared mutation).
3. **Resource lifetime:** when external resources must be closed or released (files, locks, device resources, mappings, handles, transactions).

These dimensions interact, but are not identical. A tracing-managed object can still have statically constrained mutation. A move-only resource handle can be destroyed deterministically even if ordinary data storage is reclaimed later. Region allocation can coexist with stack allocation and tracing collection.

**H1 (hypothesis):** a profile- and domain-aware architecture may reconcile the historical high-level ergonomic requirement with the current low-level ownership model better than imposing one global reclamation policy on every Omni program.

H1 is not a selected architecture. This report changes no Edition 1 grammar, rule, manifest, registry lifecycle, or ownership semantics.

## 2. Authority and current repository facts

### 2.1 Historical user requirement — OPEN

The supplied master project vision records the earlier user requirement as managed memory by default, with ownership, linearity, and regions as opt-in facilities. It also records the later conflict with Candidate 3/Edition 1 affine ownership and explicitly requires deliberate research rather than silently discarding either statement.

The original requirements interview and all underlying responses have not been recovered in this audit. Complete historical provenance must not be claimed.

### 2.2 Current core reference — SOURCE-RECORDED

The repository's spec/reference/core_semantics_and_types.md, §5, is labelled a “Normative Candidate Reference for Edition 1”. It states:

- Each heap resource or linear value has exactly one owner place at a point in execution.
- Passing a place by value transfers ownership and invalidates the source.
- Shared borrows permit concurrent reads while forbidding writes or moves.
- Exclusive borrows permit reading and writing while guaranteeing no alias.
- Non-lexical lifetimes are tracked over the control-flow graph using Polonius-style borrow checking.
- Lifetime termination synthesizes a MIR Drop terminator, with deterministic release in reverse declaration order.

This section is the current core reference direction; this research does not silently reinterpret its status or amend it.

### 2.3 Current implementation — IMPLEMENTED, NOT A MEMORY-ARCHITECTURE PROOF

The compiler/omni-own crate implements place states, moves, projections, loans, shared/mutable conflict checks, parent-authorized reborrowing, and conservative ownership-state joins. It is an analysis primitive, not a heap collector or allocator.

The compiler/omni-runtime service stores allocations in a process-local map keyed by opaque allocation IDs. Each entry contains byte storage and an initialization bitmap. Its FFI allocation interface returns an integer handle, not a host pointer.

The compiler/omni-machine abstract memory has a separate handle-based allocation store. Both paths now reserve allocation-table capacity and backing vectors fallibly before publishing an entry. That work is allocation-failure hardening; it does not implement tracing GC, RC, or inferred regions.

## 3. Why one global policy is questionable

The master vision covers both high-level application development and kernels, embedded/bare-metal, real-time and safety-critical systems, HPC, accelerators, storage, networking, and device interfaces. It also says every target need not support every feature.

Examples of different needs:

- High-level applications may benefit from automatic reclamation and convenient cyclic object graphs.
- Bare-metal or hard-real-time profiles may need bounded allocation, explicit pools, no collector, and analyzable destruction.
- Systems code often needs deterministic resource cleanup and precise layout/ABI control.
- Concurrent services need safe sharing and predictable latency.
- Graph-heavy data may naturally contain cycles or shared subgraphs.
- Region-shaped workloads may benefit from bulk reclamation.

“Managed memory by default” is not yet precise enough to select an implementation. It could mean tracing GC, compiler-inserted RC, inferred regions, owned handles into a managed heap, or a profile-selected strategy. These options have materially different semantics and costs.

## 4. Do not conflate reclamation, access, and resources

| Dimension | Question | Example mechanisms |
|---|---|---|
| Storage reclamation | When can storage be reused? | Stack, region end, explicit deallocation, RC, tracing |
| Access/alias discipline | Which aliases and mutations are legal? | Unique ownership, shared read, exclusive borrow, synchronized mutation |
| Resource lifetime | When must an external resource be released? | Deterministic close/drop, scoped lock, transaction completion, device unmap |

A garbage collector answers a storage-reachability question; it does not by itself define legal concurrent mutation. A borrow checker answers an access/alias question; it does not by itself select an allocator or define a target ABI. Deterministic resource cleanup should not depend solely on the timing of tracing collection.

This separation is a hypothesis to investigate, not an established Omni semantic rule.

## 5. Prior art and limitations

### 5.1 Region-based memory management

Tofte and Talpin describe a region discipline in which all values reside in regions, allocation/deallocation points are inferred using type/effect analysis, and the store is a stack of regions. Their paper defines a dynamic semantics and proves the inference system sound with respect to a standard semantics. This is a formal basis for investigating inferred regions, not an endorsement of a particular Omni design.

Cyclone integrates region subtyping with stack allocation and a garbage collector. The paper reports some explicit annotations for separate compilation, while defaults, local inference, and region effects reduce annotation burden. This demonstrates a hybrid design point; it does not show that cross-domain complexity is free.

### 5.2 Precise reference counting

Perceus introduces precise compiler-inserted reference counting, reuse analysis, and specialization. Its formalization uses a linear resource calculus and proves soundness/garbage-freedom for cycle-free programs in its stated setting. Koka documentation explains that its inductive data types cannot form cycles, enabling this style of reclamation.

Therefore ordinary strong RC must not be treated as a drop-in tracing replacement for arbitrary cyclic object graphs. A general Omni design would need explicit cycle policy: tracing/cycle collection, constrained graph formation, weak edges, or another mechanism.

### 5.3 Tracing collectors

The Go GC guide describes an explicit CPU-time versus memory trade-off. Collection frequency, live heap size, allocation rate, roots, and metadata affect the cost. Go performs core tracing concurrently to limit global pauses, but notes that concurrency has throughput and implementation costs.

The OCaml documentation describes a generational collector: copying minor collections stop the world; major-heap marking/sweeping proceeds in slices to reduce pause duration; compaction can touch the heap more broadly. These are different design points, not one generic “GC cost”.

### 5.4 Ownership/borrow checking

Oxide provides a formal account of Rust-like ownership and borrowing, including NLL, and establishes type-safety properties for its formal calculus. It is a useful starting point for an ownership model, not proof that Omni's current implementation is sound.

A preprint submitted 2026-11-04, “Verified Implementation of the Next-gen Borrow Checking Algorithm for Rust via Abstract Interpretation”, formalizes Polonius over a core Rust MIR subset and reports a CompCert-based verified implementation. Its scope is a core subset and it is an arXiv preprint; it must not be represented as proof of Omni or proof of the complete production Rust compiler.

## 6. Three candidate architectures

At least three materially different candidates remain open.

### Candidate A — Affine/ownership-first core; managed domains are explicit

Retain the Edition 1 affine model as the default semantic foundation and add tracing-managed or RC-managed domains as explicit facilities.

**Potential advantages:** closest to current reference and implementation; supports no-collector and low-level profiles; access discipline remains explicit; managed graphs remain possible.

**Potential costs:** may not satisfy historical managed-by-default ergonomics for ordinary application code; managed handle types may be visible; cycles, borrows into movable managed objects, and pinning need rules.

### Candidate B — Managed heap by default; affine/unique control is opt-in

Ordinary heap values are automatically reclaimed by tracing GC or a formally restricted RC system. Unique borrows/ownership remain for exclusive mutation, external resources, and low-level control.

**Potential advantages:** closest to historical high-level managed-memory intent; natural cyclic data if tracing is used; less explicit reclamation ceremony for ordinary applications.

**Potential costs:** runtime/root metadata required; CPU/memory/latency and OOM policy must be specified; deterministic resource cleanup is separate; bare-metal and hard-real-time profiles need alternatives; RC alone does not collect arbitrary cycles.

### Candidate C — Composable domains with independent access and reclamation policies

A shared foundation separates storage domains from access/alias permissions and deterministic resource obligations. Profiles could select managed defaults for high-level applications and affine/region/arena policies for systems targets.

Illustrative research vocabulary only (not proposed source syntax):

- Reclamation: Stack, Region, ReferenceCounted, Tracing, External.
- Access: Unique, SharedRead, ExclusiveBorrow, synchronized-mutation capability.
- Resource lifetime: deterministic owner token independent of ordinary data reclamation.

**Potential advantages:** could preserve both historical ergonomics and low-level control; profiles can exclude unavailable mechanisms; data domains may specialize.

**Potential costs:** highest type-system/compiler/runtime complexity; cross-domain pointers, cycles, moves, escapes, pinning, FFI, concurrency, inference, diagnostics, MIR, cleanup, and ABI need explicit semantics; risks becoming unstructured feature accumulation.

### Preliminary comparison

| Criterion | A: Affine-first | B: Managed-first | C: Composable domains |
|---|---|---|---|
| Compatibility with current reference | Highest | Requires redesign or profile boundary | Possible if additive and scoped |
| Historical managed-default intent | Partial/uncertain | Closest | Could support in high-level profile |
| Deterministic resource cleanup | Natural for owned resources | Separate from heap reclamation | Separate axis |
| Cyclic object graphs | Explicit managed domain needed | Natural with tracing; RC needs cycle policy | Select tracing or constrained domain |
| Bare metal/no runtime | Strong fit | Needs alternative profile | Strong if domains are profile-gated |
| Complexity | Lower core | Collector and root metadata | Highest |
| Real-time predictability | Potentially strong; still needs bounds | Collector profile needs latency analysis | Per-profile assurance required |
| High-level ergonomics | Depends on wrappers | Strong potential | Strong potential, but complexity risk |
| Status | Existing source direction | Research required | Hypothesis/prototype required |

No candidate is selected.

## 7. Preliminary findings

1. Managed storage need not imply unrestricted aliasing; access permissions can remain constrained.
2. Deterministic resource destruction should be specified separately from ordinary heap reclamation.
3. Precise RC can be compelling for acyclic data but does not by itself collect strong cycles.
4. Tracing collector designs expose different CPU, memory, write-barrier, and pause trade-offs.
5. Region inference is more than manual arenas: escape/subtyping/type-effect rules determine safety.
6. Hybrid designs have precedent but still require Omni-specific models and feature-interaction evaluation.
7. Borrow-checking research can improve the current affine subsystem independently from the storage-reclamation decision.

## 8. Hypothesis and formal obligations

### H1 — Reclamation and access are separate interacting dimensions

A value may need (a) a storage domain, (b) legal access/alias permissions, and (c) deterministic resource obligations. Keeping these distinct may allow high-level managed values and low-level affine values to coexist without semantic ambiguity.

A minimal formal model should include:

- A store mapping object identities to values and outgoing references.
- Roots or owners appropriate to each reclamation policy.
- Region identities, nesting, and escape constraints.
- Borrow permissions and loan invalidation.
- Deterministic external-resource obligations.
- Cross-domain reference and transfer rules.
- Failure and resource-exhaustion behaviour.

Required obligations, not yet proved:

- Temporal safety: no access through a reference to reclaimed storage.
- Spatial safety: no access outside the represented object/region.
- Alias discipline: shared/exclusive permissions cannot be used inconsistently.
- Region containment: references cannot outlive the region.
- RC accounting: counts equal specified root/edge contributions; cycle behaviour is explicit.
- Tracing soundness: reachable objects survive; unreachable objects are reclaimable per contract.
- Resource cleanup: deterministic resources release exactly once on all specified paths.
- Domain crossing: copies, moves, pins, promotions, transfers preserve provenance.
- No hidden authority: memory-domain choice cannot silently grant capabilities or mutation authority.

## 9. Falsification experiment

An isolated executable model is added under research/memory-model/spike.rs, outside the production Cargo workspace. It models:

- Dynamic shared/exclusive access permission checks.
- Region bulk reclamation.
- Acyclic strong-reference-count reclamation.
- Retention of an unreachable strong-reference cycle by plain RC.
- Tracing reachability that preserves rooted graphs and collects unreachable cycles.
- Deterministic resource-token destruction independent of separate traced-object reclamation.

The tests validate only the state transitions encoded in this small model. They do not show that Omni's type system can express the rules, that a compiler can enforce them, or that a production collector has suitable performance or memory safety.

### Next falsification stage

After reviewing this model, build a typed calculus/prototype combining domains and attempt to break it with:

- Cycles crossing region/tracing boundaries.
- RC↔tracing cross-domain object edges.
- Region escape through closures or tasks.
- Unique borrow into movable/compactable managed objects.
- Pin/FFI/DMA handles crossing domains.
- Cancellation while deterministic resources remain live.
- Shared references crossing tasks/threads.
- Partial initialization and allocation failure during transfer.
- Weak references/resurrection/finalizer-like behaviour.
- Cyclic resources requiring deterministic close.

Do not add normative syntax until these interaction rules are explicit.

## 10. Measurement plan

Measure rather than assume:

- Allocation throughput and metadata per allocation.
- Peak RSS and retained-but-unreachable memory.
- Collector CPU and write-barrier cost.
- RC increment/decrement and contention under sharing.
- Region fragmentation and retained-space when lifetimes do not nest.
- Tail latency and worst observed pause under recorded conditions.
- Resource cleanup latency and exact-once behaviour.
- Compiler analysis time/memory.
- Code size and target-runtime requirements.
- Cross-domain conversion cost.

Workloads should include acyclic trees, shared DAGs, cyclic graphs, short-lived AST/HIR objects, long-lived app graphs, high-allocation request workloads, byte buffers/arenas, pinned device/FFI resources, concurrent shared access, and region-shaped batch processing.

Benchmark at least high-level managed, affine/explicit, and collector-free/region-oriented profiles. Record hardware, OS, compiler commit, toolchain, collector configuration, and workload generator. Do not declare a winner without reproducible multi-run results.

## 11. Current decision status

- Historical managed-memory-default requirement: OPEN.
- Edition 1 affine ownership reference: current source-recorded direction; do not silently supersede.
- Current omni-own implementation: implemented/tested affine place/loan analysis; not a formal proof.
- Runtime allocation tables: opaque-handle byte stores; not tracing/RC/region memory architecture.
- Candidates A/B/C: PROPOSED.
- H1: HYPOTHESIS.
- Isolated model: EXPERIMENTALLY_VALIDATED only after the exact CI workflow passes.
- Normative language change: none.
- Architecture decision: none.
- User approval: not assumed.

## 12. Immediate next steps

1. Review exact CI results for the isolated model and preserve negative outcomes.
2. Extend the model with mixed-domain references and region escape.
3. Recover the original requirements interview and map the managed-memory requirement to exact statements.
4. Compare default policies across embedded, bare-metal, real-time, and security-critical profiles.
5. Formalize deterministic resource destruction separately from object reclamation.
6. Only then decide whether a second prototype should focus on A, B, or C.
7. Do not edit Edition 1 normative text until evidence and controlled governance authorize it.

## 13. Sources

1. Mads Tofte and Jean-Pierre Talpin, “Region-Based Memory Management”, Information and Computation 132(2), 1997, pp. 109–176. DOI: https://doi.org/10.1006/inco.1996.2613
2. Dan Grossman, Greg Morrisett, Trevor Jim, Michael Hicks, Yanling Wang, and James Cheney, “Region-based memory management in Cyclone”, PLDI 2002, pp. 282–293. DOI: https://doi.org/10.1145/512529.512563
3. Alex Reinking, Ningning Xie, Leonardo de Moura, and Daan Leijen, “Perceus: Garbage Free Reference Counting with Reuse”, PLDI 2021. DOI: https://doi.org/10.1145/3453483.3454032
4. Daan Leijen, The Koka Programming Language: https://koka-lang.github.io/koka/doc/manual.html
5. The Go Authors, “A Guide to the Go Garbage Collector”: https://go.dev/doc/gc-guide
6. OCaml Documentation, “Understanding the Garbage Collector”: https://ocaml.org/docs/garbage-collector
7. Aaron Weiss, Olek Gierczak, Daniel Patterson, and Amal Ahmed, “Oxide: The Essence of Rust”, PACMPL/POPL 2019 preprint: https://arxiv.org/abs/1903.00982
8. Jinhua Wu, Yuting Wang, and Zhemin Chen, “Verified Implementation of the Next-gen Borrow Checking Algorithm for Rust via Abstract Interpretation”, arXiv preprint submitted 2026-11-04: https://arxiv.org/abs/2611.06319

Research access date: 2026-11-09.
