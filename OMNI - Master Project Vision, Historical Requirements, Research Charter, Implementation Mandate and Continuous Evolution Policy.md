# OMNI
## Master Project Vision, Historical Requirements, Research Charter, Implementation Mandate and Continuous Evolution Policy

**Document purpose:** Establish the complete project vision and development mandate for Omni, bringing together its original conception, historical requirements, subsequent architectural decisions, research initiative, implementation repository, intended language experience, technical requirements, verification strategy, and future evolution.

**Project:** Omni Programming Language and Compiler Platform

**Primary implementation repository:** https://github.com/shreyashjagtap157/Omni---New

**Other historical repository:** https://github.com/shreyashjagtap157/Omni

**Independent research workspace:** `C:\Users\siddh\Downloads\ABC\Ideas\Omni-Research`

**Status:** Continuing research and engineering project. Completion of a specification, compiler milestone, language edition, or self-hosting milestone must never be mistaken for the end of Omni's development.

---

# PART I — THE FUNDAMENTAL VISION

## 1. What Omni Is Supposed to Become

Omni is intended to become an exceptionally capable, general-purpose, multi-paradigm, multi-level, multi-runtime programming language and its supporting development platform.

It should combine the strongest aspects of programming-language theory, type systems, memory management, compiler architecture, execution models, concurrency, security, formal verification, performance engineering, hardware abstraction, and developer experience.

The intention is not to reproduce an existing programming language with different syntax.

The intention is to research and develop a language that meaningfully advances the state of programming-language design by bringing together the capabilities that developers need across different levels of abstraction and application domains.

Omni must be capable of supporting everything from low-level machine-oriented programming to highly abstract application development, without forcing developers to sacrifice safety, performance, expressiveness, or control unnecessarily.

It should be suitable for:

- Systems programming.
- Operating-system and kernel development.
- Embedded programming.
- Bare-metal programming.
- Firmware.
- Real-time systems.
- Safety-critical software.
- Security-critical software.
- Aerospace and space systems.
- Defense-related applications.
- Automotive systems.
- Industrial control systems.
- Enterprise software.
- Backend services.
- Distributed systems.
- Database engines.
- Programming-language and compiler development.
- AI infrastructure.
- Machine-learning infrastructure.
- Scientific computing.
- Numerical computing.
- High-performance computing.
- Parallel computing.
- GPU and accelerator programming.
- Application development.
- Framework and platform development.
- Developer tooling.
- CLI applications.
- Desktop applications.
- Networking.
- Cryptographic systems.
- Storage systems.
- Simulation.
- Game-engine development.
- Hardware abstraction and interfacing.

These are intended application domains, not a requirement that every target must support every feature.

Omni must establish mechanisms that allow different computational and assurance profiles to support these domains without compromising the integrity of the shared language foundation.

### 1.1 The central design philosophy

The original ambition can be summarized as:

**Everything, but structured.**

The language should provide extensive capabilities, but those capabilities must be organized into coherent layers with explicit semantics and well-understood boundaries.

The language should not become an unstructured accumulation of features.

Its complexity should be managed through:

1. A small, mathematically understandable semantic foundation.
2. Composable type, effect, ownership, and capability systems.
3. Multiple abstraction levels.
4. Explicit computational and execution profiles.
5. A carefully defined standard library.
6. Modular compiler and runtime architecture.
7. Controlled language extensions.
8. Deterministic compilation and execution guarantees where applicable.
9. Continuous compatibility and conformance management.

The original complete specification describes deterministic correctness, layered complexity, safe defaults, visible effects, controlled extensibility, and phased implementation as foundational principles. Those principles remain important historical design commitments, subject to formal examination and refinement. 

## 2. Omni Must Never Have a Fixed Development Ceiling

This is a fundamental, permanent requirement.

**There must be no predetermined final limit on the development, enhancement, evolution, or improvement of Omni.**

Omni must continue to be researched, upgraded, expanded, corrected, optimized, and improved as the fields relevant to programming languages evolve.

This includes future developments in:

- Programming-language theory.
- Type theory.
- Formal semantics.
- Compiler construction.
- Memory management.
- Garbage collection.
- Ownership systems.
- Borrow checking.
- Static analysis.
- Formal verification.
- Mathematical logic.
- Algorithms and data structures.
- Operating systems.
- Computer architecture.
- Processor design.
- Parallel and distributed computing.
- Concurrency.
- Cryptography.
- Security engineering.
- Artificial intelligence.
- Hardware acceleration.
- GPU and accelerator architectures.
- Runtime design.
- Optimization.
- Incremental computation.
- Developer tooling.
- Program synthesis.
- Software engineering.
- Software verification.
- Software assurance.
- Build systems.
- Package management.
- Reproducible computing.
- New computational models.
- New mathematical theories.
- New hardware capabilities.
- New execution environments.

Whenever a relevant technology, technique, algorithm, logical system, mathematical theory, compiler architecture, or programming-language concept becomes available, Omni's architecture must be eligible for reassessment.

The project must investigate whether the development is:

- Directly applicable.
- Applicable after adaptation.
- Useful only in an optional profile.
- Useful for a particular target architecture.
- Useful for improving an existing implementation.
- Better than an existing solution.
- Incompatible with existing guarantees.
- Not sufficiently mature to adopt.
- Not beneficial after empirical evaluation.

The objective is not to adopt every new development indiscriminately.

The objective is to ensure that Omni does not become technologically stagnant or preserve an inferior design simply because it was selected earlier.

### 2.1 Continuous improvement is not optional

The following must become permanent project principles:

- No artificial feature-completion ceiling.
- No assumption that a language is permanently finished.
- No prohibition against researching a better replacement for an existing subsystem.
- No preservation of inferior algorithms merely because they are already implemented.
- No permanent dependence on an outdated compiler backend when a better suitable alternative emerges.
- No refusal to improve a sound subsystem simply because it already passes its current tests.
- No restriction against adding new computational models when justified.
- No restriction against replacing internal representations when a superior design is demonstrated.
- No assumption that future hardware can be handled adequately by today's architecture.
- No permanent commitment to one compiler implementation strategy.

Omni should have a continuous research, development, validation, and evolution programme.

### 2.2 Important distinction: unlimited evolution versus stable releases

Although Omni's overall development must have no fixed endpoint, individual language editions and production releases must have controlled, reproducible definitions.

This means:

- A released language edition must have immutable normative semantics.
- A compiler release must identify the exact language edition and feature support.
- Future language editions may introduce improvements and new capabilities.
- Breaking changes must have explicit migration procedures.
- Security and correctness fixes must be tracked.
- Old programs must not silently acquire different meanings after an upgrade.
- Implementations must publish their conformance and compatibility claims.
- Experimental features must remain distinguishable from established language functionality.

**Omni is never finished as a developing technology, but individual releases must be rigorously defined and reproducible.**

This distinction is essential.

---

# PART II — THE ORIGINAL PROGRAMMING EXPERIENCE

## 3. The Most Important User-Facing Requirement: Vibe-Coded Programming

This requirement needs particular attention because it concerns the actual experience of using Omni.

Omni should make programming as close to vibe coding as possible through the language's own source syntax and programming constructs.

This is a requirement about the programming language presented to the developer.

It is NOT a requirement that Omni's compiler be implemented through natural-language prompts.

It is NOT a requirement that Omni itself be written by an AI.

It is NOT a requirement that AI translate arbitrary English into code.

It means that the syntax a programmer writes when creating Omni programs should be exceptionally natural, expressive, readable, intuitive, and close to communicating the intended computational operation.

The programmer should spend as little effort as reasonably possible expressing mechanical implementation details that can be inferred safely and deterministically.

### 3.1 Express intent rather than unnecessary machinery

Omni should encourage developers to express:

- What they want a computation to accomplish.
- Which data they are working with.
- Which transformations should occur.
- Which constraints must hold.
- What outcomes are acceptable.
- Which operations should happen.
- How components should interact.

The language should determine the remaining implementation details where those details can be derived unambiguously.

For example, the language should investigate ways to minimize unnecessary:

- Type annotations.
- Generic parameter repetition.
- Explicit conversions.
- Repetitive error-handling code.
- Boilerplate control structures.
- Redundant ownership annotations.
- Repeated effect declarations in private implementation details.
- Excessive trait-bound declarations.
- Manual iteration machinery.
- Manual resource-cleanup code.
- Explicit intermediate-variable construction.
- Repetitive data-structure transformations.
- Verbose concurrency setup.
- Unnecessary configuration duplication.

However, this must never come at the expense of semantic clarity or safety.

A programmer should not need to understand every compiler subsystem to write ordinary Omni programs.

At the same time, an advanced programmer must be able to inspect, constrain, and control the details whenever necessary.

### 3.2 Vibe-first syntax must still be deterministic

An important rule:

**Natural-feeling syntax must not mean ambiguous semantics.**

The language must not attempt to interpret arbitrary prose as program instructions.

Every accepted program must have a precisely defined interpretation.

Convenient surface constructs must be deterministic transformations of the established semantic foundation.

The compiler must never silently guess:

- What a name refers to.
- Which overload should be selected.
- Which numeric conversion should occur.
- Which capability should be granted.
- Whether a computation may mutate external state.
- Whether a task may outlive its parent.
- Whether memory should be copied or moved.
- Which algorithm is intended when multiple materially different interpretations exist.

When inference has one valid solution, Omni should infer.

When more than one materially different solution exists, Omni should require the necessary information and provide a useful diagnostic.

### 3.3 Multiple levels of expression

The language should research and establish a coherent progression of programming styles.

**High-level expressive programming**

The developer expresses intent, and the compiler handles ordinary typing, data transformations, iteration, resource handling, and safe implementation details automatically wherever possible.

**Intermediate explicit programming**

The developer explicitly controls types, constraints, ownership, effects, concurrency policies, memory domains, and execution behaviour.

**Low-level systems programming**

The developer can directly manage memory layout, representation, allocation, lifetime, synchronization, hardware access, ABI requirements, and carefully controlled unsafe operations.

These levels should be compatible rather than separate languages with unrelated semantic models.

### 3.4 The programmer should have excellent diagnostics

Vibe-first programming does not mean hiding problems.

When something is incorrect, Omni should explain:

- What is incorrect.
- Where the problem originated.
- Which language rule applies.
- Why the operation is invalid.
- Which values or types are involved.
- Whether the problem is syntactic, semantic, ownership-related, effect-related, or target-related.
- How the developer can fix it.
- Whether an automated fix is safe.
- What consequences a proposed correction could have.

Diagnostics should be sufficiently informative that developers do not have to search through compiler implementation code to understand normal programming errors.

The compiler should provide actionable suggestions without making unrequested semantic changes.

### 3.5 Syntax research is a first-class activity

Do not simply adopt the syntax from Rust, Python, Kotlin, Swift, Zig, Haskell, or another language.

Research syntax and grammar design independently.

Study:

- Readability.
- Learnability.
- Ambiguity.
- Parsing complexity.
- Error recovery.
- Expressiveness.
- Refactorability.
- Type-inference compatibility.
- Formatting.
- Tooling support.
- Incremental parsing.
- Semantic transparency.
- Mathematical precision.
- Low-level control.
- Long-term language evolution.

Test proposed syntax with real programs, increasingly complicated examples, parser prototypes, diagnostics, and developer usability evaluations.

The goal is to discover a surface language that combines exceptional usability with uncompromising semantic precision.

### 3.6 Preserve the separation between programming and compiler implementation

The language a programmer writes and the language used to implement the compiler are separate matters.

The current bootstrap uses Rust.

The programmer-facing language should receive its own independently researched syntax and semantics.

The eventual self-hosting objective is a separate bootstrap and compiler-trust milestone.

The vibe-first requirement applies to the experience of writing Omni programs, not to the implementation language selected for Omni's compiler.

---

# PART III — HISTORICAL DEVELOPMENT AND REQUIREMENTS

## 4. Original Conception

Omni began with the intention of creating an idealized programming language capable of supporting a very broad range of programming tasks.

The initial design process was not simply a decision to make another systems language.

It involved a substantial requirements interview, with the historical complete specification reporting more than 160 design decisions.

The original direction was to combine high-level development convenience with low-level control, strong correctness, flexibility, performance, and safety.

The resulting design recognized that trying to expose every mechanism in every programming context would make the language difficult to use.

The adopted architectural answer was layering.

This produced the historical direction of a multi-level, hybrid, multi-paradigm, multi-runtime language.

### 4.1 Original foundational requirements

The following intentions are important:

1. A general-purpose language rather than a language limited to one application domain.
2. Strong static typing.
3. Local type inference.
4. Algebraic data types.
5. Pattern matching.
6. Safe handling of absent values.
7. First-class error values.
8. Resource management.
9. Multiple memory-management strategies.
10. High-performance native compilation.
11. Low-level system access.
12. Managed and higher-level programming facilities.
13. Concurrency and parallelism.
14. Structured task lifetimes.
15. Strong security boundaries.
16. Rich compiler diagnostics.
17. Advanced development tooling.
18. A substantial standard library.
19. Multiple target architectures and execution environments.
20. A long-term self-hosting implementation.

These were subsequently developed into much larger architectural and specification documents.

## 5. Earlier language-design decisions

The historical design material establishes the following major subsystem directions.

### 5.1 Type system

The intended type system is statically typed by default.

Important capabilities include:

- ML-inspired type-system characteristics.
- Bidirectional type checking.
- Local inference.
- Algebraic data types.
- Product and sum types.
- Pattern matching.
- Generics.
- Traits and interfaces.
- Associated types.
- Type constraints.
- Variance.
- Lifetimes and regions.
- Function types.
- Type aliases and distinct types.
- Compile-time type computation.
- Refinement-like constraints where feasible.
- Explicit dynamic types where appropriate.
- Strong type validity guarantees.

Null should not exist as an ordinary safe value.

Absence should be represented through types such as `Option<T>`.

Expected failures should be represented through `Result<T,E>`.

Inference should be powerful enough to minimize boilerplate, but must produce a unique and sound result under the adopted language rules.

### 5.2 Memory model

The historical design explored multiple memory-management models because Omni is intended for applications ranging from low-level systems software to high-level applications.

The architecture should investigate:

- Ownership.
- Borrowing.
- Shared borrowing.
- Mutable borrowing.
- Affine resource management.
- Linear resource management where justified.
- Regions.
- Arenas.
- Stack allocation.
- Heap allocation.
- Reference counting.
- Garbage collection.
- Managed memory domains.
- Generational references.
- Stable handles.
- Pinning.
- Persistent memory.
- Memory-mapped regions.
- Device memory.
- DMA.
- Custom allocators.
- Memory layout control.
- Memory provenance.

A particularly important historical conflict requires explicit resolution.

Earlier recorded user requirements described managed memory as the default, with ownership and regions as opt-in facilities. Later Edition 1 material made affine ownership and borrowing the fundamental model while introducing explicit managed-domain facilities.

Neither statement should be silently discarded.

Research must determine the best overall memory architecture, and the original user requirement must remain visible in the requirements register until the decision is explicitly adjudicated.

The goal is to provide memory management that is safe, ergonomic, efficient, predictable, and suitable for all supported computational profiles.

### 5.3 Borrow checker and ownership ergonomics

The historical architecture places strong emphasis on eliminating unnecessary friction in ownership-based programming.

Research directions include:

- Non-lexical lifetimes.
- Polonius-style loan analysis.
- Field-sensitive borrowing.
- Independent borrowing of separate structure fields.
- Reborrowing.
- Nested references.
- Higher-ranked lifetimes.
- Generational references.
- Self-referential structures.
- Cyclic data structures.
- Resource cleanup.
- Borrow splitting.
- Precise initialization tracking.
- Drop elaboration.
- Ownership-aware compiler transformations.

Omni must not simply reproduce the existing frustrations of Rust's borrow checker.

It should research the strongest available ownership and lifetime mechanisms and investigate whether a more ergonomic and equally sound model is possible.

### 5.4 Effect system

The effect system is intended to provide a unified treatment of computational effects.

It should investigate and support, where justified:

- Input/output.
- Filesystem access.
- Networking.
- Allocation.
- Time.
- Randomness.
- Exceptions or typed failures.
- Panics.
- Async suspension.
- Blocking.
- Cancellation.
- Synchronization.
- Persistence.
- Foreign calls.
- Device operations.
- Accelerator access.
- Reflection.
- Dynamic execution.
- Nondeterminism.
- Unsafe operations.
- User-defined effects and handlers.

Effect tracking must support static reasoning about program behaviour.

A function's public contract should communicate its allowed effects.

Private implementation details should benefit from inference when the result is unambiguous.

Effect handling must preserve ownership, authority, cancellation, and task-lifetime guarantees.

### 5.5 Capability system

Capabilities should be treated as actual authority, not merely permission annotations.

Investigate:

- Unforgeable capability values.
- Capability issuance.
- Capability delegation.
- Capability attenuation.
- Capability revocation.
- Scope restrictions.
- Resource quotas.
- Filesystem permissions.
- Network permissions.
- Device access.
- Process creation.
- Plugin authority.
- Cross-process capability transfer.
- Authentication and provenance.
- Capability auditing.

Effects and capabilities should work together.

The existence of a function or effect must not automatically grant access to a resource.

Authority must be provided through an explicit, validated mechanism.

### 5.6 Error handling and failure semantics

Expected failures should be represented as values.

Programmer errors and violated invariants should have clearly defined panic or fault semantics.

Investigate:

- `Option`.
- `Result`.
- Typed errors.
- Error propagation.
- `?`-style mechanisms.
- Panic handling.
- Unwinding.
- Abort policies.
- Isolation.
- Resource exhaustion.
- Out-of-memory behaviour.
- Stack exhaustion.
- Cancellation.
- Cleanup guarantees.
- Double-fault handling.
- Error provenance.
- Structured diagnostic identifiers.

All failure paths must be specified.

No silent error handling or accidental loss of important failures should be permitted.

### 5.7 Concurrency and parallelism

Concurrency is one of Omni's major intended capabilities.

Historical requirements included:

- Virtual threads.
- Structured concurrency.
- Explicit cancellation.
- A unified concurrency model.
- Task scopes.
- Message passing.
- Channels.
- Actor-like programming.
- Parallel iteration.
- Task scheduling.
- Work stealing.
- Resource-aware execution.
- Synchronization primitives.
- Atomic operations.
- Memory-ordering semantics.
- Distributed execution.

One important unresolved historical decision is the relationship between a unified concurrency model and explicit `async`/`await` surface syntax.

The earlier user intent favored a unified model rather than forcing programmers into separate async and synchronous programming worlds.

Later candidate specifications contain explicit async constructs and effects.

Research must establish whether these can be reconciled through a unified execution abstraction, whether a superior surface design exists, and how blocking and suspension should interact.

Do not treat an older syntax choice as permanent merely because it appeared in a specification draft.

### 5.8 Syntax and language ergonomics

Earlier designs explored:

- Expression-oriented programming.
- Newline-first syntax.
- Indentation-based syntax.
- Brace-based syntax.
- Command-style calls.
- Pipeline constructs.
- Closures.
- Implicit parameters.
- Effect annotations.
- Type inference.
- Concise declarations.
- Pattern-oriented programming.

The surface syntax subsequently underwent considerable reconciliation.

Candidate 2 was explicitly disabled in the recorded project instructions.

Historical shorthand and projection syntax must not be reintroduced simply because they appeared in older documents.

The live normative grammar and approved amendments determine what is currently valid.

However, the long-term vibe-first programming requirement remains broader than any one syntax candidate and requires continuing research.

### 5.9 Modules, packages and visibility

Omni should have a complete module and package architecture.

Capabilities to research include:

- Packages.
- Workspaces.
- Modules.
- Explicit imports.
- Visibility levels.
- Package identity.
- Version constraints.
- Dependency resolution.
- Feature selection.
- Package lockfiles.
- Signed packages.
- Source provenance.
- Reproducible dependency resolution.
- Private registries.
- Offline dependency management.
- Dependency revocation.
- Compatibility checks.

The package system must be deterministic and auditable.

### 5.10 Standard library

Omni should include a comprehensive standard library divided into appropriate layers.

Areas include:

- Core language primitives.
- Collections.
- Iterators.
- Strings.
- Text processing.
- Filesystem operations.
- Networking.
- Serialization.
- Deserialization.
- Error handling.
- Logging.
- Diagnostics.
- Time.
- Scheduling.
- Cryptography.
- Synchronization.
- Concurrency.
- Processes.
- Memory allocation.
- Numerical computation.
- SIMD.
- Tensor operations.
- Hardware interfaces.
- Persistence.
- Testing.
- Reflection.
- Mathematical functions.

The standard library should be layered so bare-metal or minimal environments are not forced to link an operating-system-dependent runtime.

### 5.11 Numeric and scientific computing

Omni's numerical model should be researched extensively.

Relevant areas include:

- Fixed-width integers.
- Arbitrary-precision integer literals.
- Floating-point types.
- Decimal floating-point.
- Fixed-point arithmetic.
- Bit-precise integers.
- Numeric overflow policies.
- Checked arithmetic.
- Wrapping arithmetic.
- Saturating arithmetic.
- Exact arithmetic where possible.
- Units and dimensions.
- SIMD.
- Vectors.
- Matrices.
- Tensors.
- Floating-point reproducibility.
- Accelerator computation.
- Numerical stability.
- Compiler transformations affecting floating-point results.

### 5.12 Runtime architecture

A modular runtime is required.

The earlier design explored:

- Ahead-of-time compilation.
- Native binaries.
- Minimal core runtime.
- Memory runtime.
- Concurrency runtime.
- Capability runtime.
- Plugin runtime.
- Dynamic execution.
- JIT compilation.
- Profile-guided optimization.
- Accelerator support.

The default execution path should prioritize efficient native execution where appropriate.

Optional interpreters, virtual machines, JITs, or other execution models may exist, but they must preserve the defined semantics of Omni programs.

### 5.13 Compilation architecture

The general architecture historically included:

Source processing → parsing → semantic analysis → type/effect analysis → ownership analysis → intermediate representations → verification → optimization → backend lowering → code generation → linking → execution.

The implementation must use explicit intermediate representations with well-defined invariants.

It should investigate:

- Lossless concrete syntax trees.
- Abstract syntax trees.
- Typed intermediate representations.
- High-level IR.
- Ownership-aware MIR.
- Low-level IR.
- SSA representations.
- Control-flow graphs.
- Incremental compilation.
- Query-based compilation.
- Parallel compilation.
- Translation validation.
- Optimization correctness.
- Backend-independent semantics.

### 5.14 Compiler backends

The project has historically considered:

- Cranelift.
- LLVM.
- MLIR.
- Native target backends.
- Cross-compilation.
- Accelerator compilation.
- Future specialized architectures.

These must not be permanent commitments without further evaluation.

The architecture should support replacing or adding backends without changing the language's semantics.

### 5.15 Tooling and developer experience

The complete Omni ecosystem should eventually offer:

- Unified CLI.
- Compiler.
- Build system.
- Package manager.
- Formatter.
- Linter.
- Language server.
- IDE integration.
- Debugger.
- Test runner.
- Benchmarking.
- Profiling.
- Documentation generation.
- Static analysis.
- Security analysis.
- Refactoring.
- Automated fixes.
- Conformance tools.
- Dependency auditing.
- Release tooling.
- Compiler exploration and diagnostics.

The tooling is not secondary to the language's success.

A sophisticated compiler with a frustrating development experience would fail an important part of the original objective.

---

# PART IV — HELIOS AND THE ORIGINAL PLATFORM VISION

## 6. Relationship Between Omni and HELIOS

HELIOS was envisioned as a major platform built using Omni.

HELIOS is an advanced cognitive platform for structured knowledge management, multimodal processing, auditable reasoning, and controlled execution.

It is not simply an AI model.

The historical HELIOS architecture includes:

1. Provenance-preserving knowledge storage.
2. Immutable historical records.
3. Structured confidence models.
4. Capability-controlled access.
5. Explainable reasoning.
6. Extensible signed and sandboxed plugins.
7. Offline-first and local-primary operation.

The original Omni specification identifies several features that HELIOS depends upon:

- Capability-based security.
- Structured concurrency.
- Effect handling.
- Tensor functionality.
- Plugin architecture.
- Runtime execution.
- Controlled memory management.

HELIOS should remain a major validation target for Omni.

However, Omni should not be designed exclusively around HELIOS.

The language must be independently capable of supporting the entire range of intended application domains.

HELIOS is a flagship workload and an integration test, not the authority that defines every language feature.

---

# PART V — THE RESEARCH REQUIREMENT

## 7. Omni Must Be Researched Before Its Architecture Is Considered Established

This requirement is fundamental.

It is not scientifically or technically sufficient to specify a complicated programming language in advance and assume that implementing the resulting specification will produce a working, coherent, efficient, safe, and useful programming language.

A specification can be internally inconsistent.

A proposed type system may have undecidable or impractical inference behaviour.

A memory model may be difficult or impossible to implement soundly.

A concurrency mechanism may produce contradictions when combined with resource ownership.

A compiler representation may be incapable of expressing the semantics that the language requires.

A proposed feature may be theoretically plausible but operationally infeasible on realistic hardware.

Two individually valid features may interact incorrectly.

A compiler may accept programs that violate the intended semantics.

A compiler may reject programs that should be valid.

A theoretically sound model may still produce a frustrating programming experience.

A successful parser does not establish that the language is usable.

A passing unit test does not establish that the entire compiler implements the language correctly.

Therefore:

**Omni must be researched as a programming-language science and engineering project, with implementation serving as an integral means of testing, validating, challenging, and improving the research and design.**

## 8. The Research–Specification–Implementation–Validation Cycle

The project must operate through a continuous feedback cycle.

The correct process is not:

Specification → Implementation → Completion.

It is:

**Stakeholder requirements → Research → Candidate architectures → Formal models → Small executable experiments → Evaluation → Revised architecture → Normative specification → Production implementation → Independent verification → Integrated validation → Research feedback → Further improvement.**

Every feature should follow this process in proportion to its risk and complexity.

### 8.1 Stage A — Establish the actual requirement

For every proposed feature, determine:

- What problem is being solved?
- Why is it important?
- Which users and application domains need it?
- Which properties must hold?
- What is the expected developer experience?
- What performance characteristics are required?
- What security and safety guarantees are expected?
- What hardware or execution environments are relevant?
- What related language mechanisms could be affected?

Requirements must be linked to the original user intention.

### 8.2 Stage B — Conduct research

Investigate existing and emerging work relevant to the requirement.

Study:

- Academic research.
- Programming-language literature.
- Formal-methods literature.
- Compiler implementations.
- Operating-system implementations.
- Hardware architecture.
- Algorithmic research.
- Mathematical results.
- Production engineering experience.
- Known limitations and failure cases.

Do not rely solely on popular languages or introductory documentation.

Primary research, original technical papers, official specifications, and real implementations should be preferred.

### 8.3 Stage C — Identify competing approaches

For every significant feature, examine multiple approaches.

Compare:

- Correctness.
- Soundness.
- Completeness where applicable.
- Complexity.
- Implementation feasibility.
- Compiler performance.
- Runtime performance.
- Memory overhead.
- Scalability.
- Developer ergonomics.
- Diagnostics.
- Interactions with other language features.
- Formal verification opportunities.
- Maintenance burden.

Do not adopt an approach simply because it is used by Rust or another established language.

Likewise, do not reject a mature approach simply because Omni is intended to be different.

### 8.4 Stage D — Formalize the proposed design

Before considering a design established, define its behaviour sufficiently precisely.

Depending on the feature, this may require:

- Mathematical definitions.
- Typing judgments.
- Operational semantics.
- Abstract-machine rules.
- Inference algorithms.
- State-transition models.
- Memory-model definitions.
- Resource bounds.
- Invariants.
- Proof obligations.
- Error semantics.
- Interaction rules.

The formalization should make assumptions explicit and expose impossibility results or undecidable cases.

### 8.5 Stage E — Prototype the design

Create small, focused prototypes when necessary.

These may include:

- Grammar prototypes.
- Lexer and parser experiments.
- Type-inference solvers.
- Ownership-analysis prototypes.
- Effect-system models.
- Concurrency simulators.
- Reference interpreters.
- Intermediate-representation experiments.
- Small compiler pipelines.
- Backend experiments.
- Performance benchmarks.
- Property-based test generators.

The purpose is to challenge the design.

A prototype is not automatically a production implementation.

### 8.6 Stage F — Evaluate the prototype

Evaluate whether the prototype actually exhibits the intended properties.

Check:

- Valid program acceptance.
- Invalid program rejection.
- Soundness conditions.
- Type-inference behaviour.
- Termination.
- Algorithmic complexity.
- Memory usage.
- Compiler execution time.
- Runtime behaviour.
- Feature interactions.
- Error handling.
- Failure conditions.
- Security boundaries.
- Developer experience.

Where a proposed feature cannot satisfy its requirements, investigate alternatives or revise the requirement transparently.

### 8.7 Stage G — Establish the normative design

Only after adequate research and evaluation should the chosen behaviour enter the normative specification.

The specification must include enough detail to permit independent implementations.

Every requirement needs a defined acceptance method.

### 8.8 Stage H — Implement the production mechanism

Implement the established behaviour in the actual compiler, runtime, standard library, or tooling subsystem.

Do not substitute a demonstration implementation for the final production mechanism.

### 8.9 Stage I — Validate the actual implementation

The production implementation must be checked against the research model and normative specification.

Test both individual components and their integration.

### 8.10 Stage J — Feed implementation findings back into research

If implementation reveals:

- Unexpected complexity.
- Impractical performance.
- Semantic ambiguity.
- Unsound assumptions.
- Inconsistent feature interactions.
- Difficult diagnostics.
- Hardware limitations.
- Excessive runtime overhead.
- Poor developer ergonomics.

then the issue must be returned to research and architecture.

Do not force implementation to conform to an unworkable design simply because the design has already been written down.

Equally, do not silently change normative semantics to make the code easier to implement.

A controlled research and specification revision is required.

---

# PART VI — FORMAL VERIFICATION AND CORRECTNESS

## 9. Correctness Must Be an Engineering Objective, Not a Marketing Claim

The intended quality level requires a substantial assurance programme.

No single testing method can establish every property of a complex compiler or runtime.

Use complementary evidence.

### 9.1 Formal semantics

Define the behaviour of the language independently of a particular compiler implementation.

### 9.2 Type-system soundness

Establish the desired preservation, progress, and related properties under precisely stated assumptions for the chosen core calculus.

### 9.3 Memory safety

Investigate and prove the relevant properties of:

- Ownership.
- Borrowing.
- Aliasing.
- Initialization.
- Lifetimes.
- Deallocation.
- Pointer validity.
- Allocation provenance.
- Memory representation.
- Unsafe operations.

### 9.4 Effect and capability correctness

Verify that effect declarations and capability requirements are enforced.

Investigate whether an untrusted program can obtain authority it was never granted.

### 9.5 Concurrency correctness

Investigate:

- Data races.
- Deadlocks.
- Task lifetime.
- Cancellation.
- Synchronization.
- Memory ordering.
- Resource exhaustion.
- Message ownership transfer.
- Scheduler correctness.
- Structured task completion.

### 9.6 Compiler correctness

The compiler must preserve the semantics of accepted programs across:

- Parsing.
- Semantic analysis.
- Lowering.
- Optimization.
- Code generation.
- Linking.
- Runtime execution.

The correctness of each transformation must be established using appropriate methods.

### 9.7 Optimization correctness

Optimization must not change permitted observable behaviour.

Where transformations rely on assumptions, those assumptions must remain valid across the transformation.

The historical v3.4 reconciliation material provides a particularly valuable direction: unsafe-operation assumptions should become explicit semantic dependencies with scoped validity, allowing optimizations where their proofs remain valid rather than blindly disabling optimization.

This should remain an explicit research topic.

### 9.8 Reproducibility

Investigate reproducible compilation, hermetic builds, dependency integrity, deterministic diagnostics, and artifact provenance.

### 9.9 Independent verification

Independent implementations and independent verification tools should eventually provide evidence beyond the primary compiler's self-reported results.

### 9.10 Hardware and physical-system boundaries

For critical targets, distinguish software correctness from physical validation.

Hardware faults, timing, power consumption, electromagnetic effects, device failures, and environmental assumptions cannot be proven away merely through programming-language semantics.

The language should provide suitable abstractions and assurance support, but system-level validation remains necessary.

---

# PART VII — IMPLEMENTATION STRATEGY

## 10. Implementation Must Progress Alongside Research

Omni must be developed through a research-informed engineering programme.

Implementation is not postponed until every possible language feature has been researched.

Instead, implementation begins with a sufficiently justified, deliberately limited core and expands as the underlying designs become ready.

The compiler should evolve through working, testable increments.

### 10.1 Bootstrap language

The current direction uses Rust for bootstrap implementation.

This is a practical engineering choice because Rust provides:

- Memory safety in the implementation language.
- A mature systems-programming ecosystem.
- Suitable compiler and parser libraries.
- Strong build tooling.
- Established testing support.
- Suitable backend integrations.
- A credible path towards eventual self-hosting.

The choice must remain open to future reassessment where relevant, without destabilizing the current implementation unnecessarily.

### 10.2 Compiler architecture

The intended architectural path should be examined and implemented as a coherent pipeline:

1. Source loading and validation.
2. Lexical analysis.
3. Lossless concrete syntax construction.
4. Parsing.
5. Syntax-tree lowering.
6. Desugaring.
7. Name resolution.
8. Type inference.
9. Type checking.
10. Effect analysis.
11. Capability analysis.
12. Ownership and borrow analysis.
13. High-level IR construction.
14. MIR construction.
15. MIR verification.
16. Drop and cleanup elaboration.
17. Optimization.
18. Low-level IR lowering.
19. Target selection.
20. Machine lowering.
21. Code generation.
22. Object emission.
23. Linking.
24. Runtime initialization.
25. Program execution.

The actual pipeline may be refined by research.

Its important property is that every stage has well-defined responsibilities, interfaces, invariants, and tests.

### 10.3 Real implementation integration

A compiler milestone is not complete merely because:

- A crate compiles.
- A parser accepts sample programs.
- An isolated semantic unit test passes.
- An IR data structure exists.
- A backend dependency is integrated.
- A CLI command prints expected-looking output.

The implementation must work through the integrated pipeline.

Where a feature is intended to produce native executable output, demonstrate an actual source-to-execution path.

### 10.4 Fail closed

If a language feature is not implemented, or a required semantic artifact is missing, the compiler must not fabricate a substitute interpretation.

It must report the limitation and reject unsupported uses appropriately.

Infrastructure that does not depend on the unavailable feature may continue developing.

### 10.5 Rust safety and code quality

Maintain the established requirements:

- `unsafe_code = "forbid"` at workspace level, with narrowly justified and audited exceptions if the normative design requires them.
- Strict Clippy enforcement.
- Clean formatting.
- No unexplained compiler warnings.
- No accidental unused implementation.
- No ignored errors.
- No test weakening to manufacture green results.
- No dead fallback code concealing incomplete functionality.
- Explicit error propagation.
- Proper resource management.
- Consistent interfaces.
- Reviewable code changes.

---

# PART VIII — CONTINUOUS IMPLEMENTATION VALIDATION

## 11. Implementation Must Demonstrate That Omni Actually Works

This is an explicit part of the original request and must become a permanent project requirement.

The purpose of implementation is not just to create software that resembles a compiler.

It is to demonstrate that the language, as researched and specified, can be realized and used successfully.

The validation programme must operate at several levels.

### 11.1 Unit testing

Test individual algorithms and components.

Examples:

- Lexer.
- Parser.
- Type inference.
- Name resolution.
- Ownership analysis.
- Effect inference.
- Capability checking.
- MIR construction.
- MIR verification.
- Machine semantics.
- Code generation.
- Runtime services.

### 11.2 Integration testing

Test connections between subsystems.

Examples:

- Lexer to parser.
- Parser to syntax-tree lowering.
- Type checker to HIR.
- Ownership checker to MIR.
- MIR verifier to code generation.
- Backend to linker.
- Runtime to capabilities.
- Compiler to standard library.
- Package manager to build system.

### 11.3 Conformance testing

Every implemented language rule should be linked to executable evidence.

Test:

- Valid programs.
- Invalid programs.
- Boundary conditions.
- Feature interactions.
- Error cases.
- Unsupported features.
- Compiler diagnostics.
- Expected execution outcomes.

### 11.4 Property-based testing

Generate many valid and invalid programs and verify required properties.

### 11.5 Fuzz testing

Fuzz:

- Source decoding.
- Lexing.
- Parsing.
- Semantic analysis.
- Intermediate representations.
- Serialization.
- Package processing.
- Unsafe boundaries.
- Runtime inputs.

### 11.6 Differential testing

Compare different execution or compiler implementations wherever appropriate.

Examples:

- Reference interpreter versus native execution.
- Bootstrap compiler versus Omni-written compiler.
- Independent parser versus primary parser.
- Different backends.
- Different supported targets.

### 11.7 Regression testing

Every discovered defect should result in a regression test when feasible.

Fix the underlying defect rather than suppressing the symptom.

### 11.8 End-to-end testing

Demonstrate that ordinary developers can create, compile, and execute real Omni programs.

Test complete projects, not only isolated code snippets.

### 11.9 Performance testing

Benchmark:

- Compilation time.
- Incremental compilation.
- Memory consumption.
- Executable size.
- Runtime performance.
- Allocation overhead.
- Concurrency overhead.
- Numerical performance.
- Code-generation quality.
- Optimization effectiveness.

Compare against relevant established alternatives under controlled, fair conditions.

### 11.10 Developer-experience validation

This is particularly important for the vibe-first requirement.

Evaluate whether ordinary tasks can be expressed naturally.

Investigate:

- Lines of code required.
- Boilerplate.
- Cognitive burden.
- Number of annotations.
- Diagnostic understandability.
- Common programming patterns.
- Refactoring effort.
- Ease of learning.
- Readability.
- Consistency.
- Error correction.
- Editor assistance.

A language can be formally sound and still be unpleasant to program in.

Both dimensions must be considered.

---

# PART IX — UNLIMITED EVOLUTION AND MAINTENANCE

## 12. Omni Must Have a Permanent Technology-Watching and Research Process

Omni must not depend on a one-time research exercise.

The research process must continue throughout its lifetime.

### 12.1 Continuous research topics

Maintain research coverage of:

- New programming languages.
- New language proposals.
- New type-system research.
- New compiler architectures.
- New compiler optimizations.
- New formal-verification methods.
- New proof assistants.
- New memory-management algorithms.
- New concurrency models.
- New distributed execution models.
- New cryptographic standards.
- New processor architectures.
- New GPU and accelerator architectures.
- New hardware execution models.
- New runtime systems.
- New build architectures.
- New software-assurance practices.
- New security vulnerabilities and defensive techniques.
- New numerical methods.
- New mathematical results.
- New developer-experience research.

### 12.2 Research every existing subsystem periodically

Do not only research proposed new features.

Reassess existing subsystems.

For example:

- Is the current borrow checker still the best feasible approach?
- Is the current type solver still suitable?
- Can compilation become more incremental?
- Is a better intermediate representation available?
- Can compiler diagnostics become substantially more informative?
- Can runtime overhead be reduced?
- Can resource management become more ergonomic?
- Can security enforcement become stronger?
- Can a better concurrency abstraction replace the current one?
- Are new hardware features worth supporting?
- Can the language express common algorithms more naturally?

### 12.3 Research maturity

Do not confuse the publication of a new algorithm or theory with demonstrated applicability.

Track:

- Research publication.
- Formal results.
- Implementation availability.
- Reproducibility.
- Maturity.
- Known limitations.
- Independent validation.
- Performance evidence.
- Compatibility implications.

Adoption must be evidence-based.

### 12.4 Architecture evolution

The architecture must allow improvements without requiring complete reinvention.

Use modular subsystem boundaries, versioned interfaces, stable semantic contracts, dependency tracking, and migration processes.

The compiler architecture, runtime, library, tooling, and language standard must evolve together where necessary.

---

# PART X — CURRENT PROJECT HISTORY AND IMPLEMENTATION STATE

## 13. The Two Related Omni Workspaces

There are two major efforts that must remain connected.

### 13.1 Implementation repository

The designated implementation repository is:

`https://github.com/shreyashjagtap157/Omni---New`

It contains:

- Rust workspace.
- Compiler crates.
- Normative specification.
- Grammar.
- Type and semantic references.
- Conformance material.
- Tests.
- CI configuration.
- Qualification tools.
- Implementation roadmap.

This is the actual compiler and toolchain engineering project.

Do not accidentally use the older `Omni` repository as the implementation source of truth.

### 13.2 Independent research workspace

The research workspace is:

`C:\Users\siddh\Downloads\ABC\Ideas\Omni-Research`

It was created to investigate an idealized programming language from first principles.

The recorded workspace contained 52 documents of approximately 338 KB before subsequent research development.

Its latest recorded Master Specification snapshot, from November 1, 2026, was:

- Version: `0.1.6-s0`.
- Content SHA-256: `83bddfc7c3f41434d662a65cfd33682f714d1ce65192e4a189677658862df76e`.
- 17 specialist roles documented.
- Independent review pending.
- The S0 gate not passed.
- No language-level normative semantics adopted.

This workspace developed an evidence-based research charter, requirement policy, formalization policy, standards mapping, claims and sources, provenance records, research logs, risk and resource planning, and staged research direction.

These are valuable foundations.

However, its original charter deliberately defined documentation as the deliverable and explicitly excluded producing a compiler or runtime.

That limitation must be changed for the complete Omni objective.

The research workspace must become an integrated research and engineering programme, rather than a documentation-only project.

### 13.3 Connecting the two efforts

The relationship must be:

**Original user intent → Historical requirements archive → Independent research → Formal models → Candidate specifications → Implementation experiments → Production compiler → Independent conformance and validation → Updated research and specifications.**

Both workspaces must maintain a consistent requirements and decision history.

Research must not silently discard the original project vision.

Equally, historical documents must not automatically become normative merely because they were written earlier.

They are historical requirements evidence and design material to be examined, reconciled, and either adopted, refined, rejected, or superseded through explicit decisions.

---

## 14. Important Existing Repository Facts

The following facts were retrieved from the designated GitHub repository and its tracked documents.

The repository uses `main` as its default branch.

Its README, updated in the retrieved repository history on October 22, 2026, describes it as an early-stage compiler implementation rather than a completed production compiler or self-hosted language.

The repository pins Rust 1.95.0 in its recorded toolchain configuration.

Its documented generational roadmap is:

**Stage 1 — Rust-only minimal working core**

Version progression: `0.0.0.1` through `1.0.0.0`.

**Stage 2 — Rust-only complete production platform**

Version progression: `1.0.0.1` through `2.0.0.0`.

**Stage 3 — Omni-written, self-hosted, independently qualified implementation**

Version progression: `2.0.0.1` through `3.0.0.0`.

These are roadmap goals rather than evidence of completed stages.

### 14.1 Historical implementation architecture

The implementation plan documents the following crates:

| Crate | Historically recorded status |
|---|---|
| `omni-lex` | Core functionality present |
| `omni-parse` | Core functionality present |
| `omni-types` | Core functionality present |
| `omni-names` | Core functionality present |
| `omni-own` | Ownership framework present |
| `omni-effects` | Framework/skeleton |
| `omni-hir` | Partial |
| `omni-mir` | Core representation defined |
| `omni-verify` | Partial |
| `omni-machine` | Not started in the recorded implementation plan |
| `omni-codegen` | Not started in the recorded implementation plan |
| `omni-runtime` | Not started in the recorded implementation plan |

The implementation plan's status is dated October 12 and must not be treated as the exact present-day status.

The README's October 22 snapshot also recorded a failed qualification attempt involving Cargo.lock and `--locked` verification, a stable/nightly fuzz-toolchain mismatch, and an outstanding dependency security issue concerning ruzstd.

These require fresh investigation against the live repository and current GitHub Actions.

### 14.2 Existing milestones and evidence

Previously recorded implementation work includes:

- Normative grammar and specification milestones.
- Lexer and parser work.
- Type-system work.
- Name-resolution work.
- Place models.
- Assignment handling.
- Ownership verification.
- MIR development.
- Cranelift integration.
- Target-layout work.
- Bounds checking.
- Aggregate ABI work.
- Stage-0 predicate-engine development.
- Effect-scope improvements.
- Compiler audit tooling.
- Canonicalization work.

Historical test reports include:

- 19/19 `omni-verify` tests.
- More than 900 Stage-0 predicate-engine tests.
- 18 Cranelift-related layout tests.
- Additional focused verifier and integration tests.

These are historical results, not current qualification claims.

Every relevant result must be revalidated against the current code and actual CI output.

### 14.3 Existing implementation constraints

The following historical requirements remain important unless superseded through explicit decisions:

- `spec/grammar/omni-edition1.ebnf` is the normative Edition 1 grammar.
- Candidate 2 was disabled.
- Do not revive historical syntax merely because it appears in older specification documents.
- Do not invent missing semantics.
- Do not create an unauthorized version progression.
- `0.0.0.13` must remain undeclared until its qualification conditions are met.
- Do not use an unauthorized `0.0.1.x` progression.
- Milestone versions use four numeric components.
- Preserve Git history.
- Do not force-push.
- Maintain `main` as the prescribed implementation branch.
- Verify the exact repository before synchronization or destructive Git operations.
- Do not declare milestones based on isolated successful tests.

Current source, manifest, grammar, rules, and CI must always be inspected before work.

---

# PART XI — MISSING AND CONFLICTING REQUIREMENTS

## 15. Important Items That Must Not Be Lost

Reviewing the historical requirements, later specification documents, research workspace, and implementation history reveals several issues that require explicit attention.

### 15.1 Original requirements must be recovered

The historical specification says that more than 160 design decisions were made during the original requirements interview.

The currently available consolidated records are not the same thing as a complete transcript of those original questions and answers.

The original questionnaire, user responses, follow-up clarifications, rejected alternatives, and final decisions should be recovered wherever possible.

The project should establish a permanent historical requirements archive.

This archive should include:

- Original requirement.
- Original user statement, where recoverable.
- Date and context.
- Subsequent interpretation.
- Related decisions.
- Current status.
- Replacement or supersession record.
- Remaining uncertainty.

Do not claim complete historical coverage until that archive has been examined.

### 15.2 Resolve the memory model conflict

Earlier user intent:

- Managed memory as a default.
- Opt-in ownership, linearity, and regions.

Later Candidate 3 architecture:

- Affine ownership as the fundamental model.
- Borrowing and regions.
- Explicit managed-domain facilities.
- `linear` withdrawn.

This requires a deliberate research decision.

### 15.3 Resolve the concurrency conflict

Earlier user intent:

- Virtual threads.
- Unified concurrency model.
- Explicit cancellation.
- Structured concurrency.
- Avoiding a disruptive async/synchronous programming split.

Later specifications include explicit async constructs and effects.

Research a unified developer-facing concurrency architecture before ratifying that decision permanently.

### 15.4 Reconcile syntax history

Different historical documents contain:

- Indentation-based blocks.
- Brace-based blocks.
- Different pipeline formulations.
- Different projection forms.
- Different effect syntax.
- Different closure syntax.
- Different command-call rules.

The latest authorized normative grammar must govern present implementation.

Historical experiments remain archived and must not be confused with the current language.

Future improvements must undergo the established syntax and compatibility process.

### 15.5 Reconcile specification and implementation version authority

The historical Candidate 3 consolidated documents are useful, but the retrieved repository manifest reports `1.0.0-candidate.1`.

The exact repository manifest, module files, co-normative artifacts, errata, and grammar must establish the actual current normative baseline.

Do not assume an archived specification is the live normative specification merely because its title identifies Candidate 3.

### 15.6 Upgrade the research programme

The new research workspace must not remain documentation-only.

It must include:

- Research.
- Mathematical modelling.
- Formalization.
- Prototype implementation.
- Compiler experiments.
- Executable reference models.
- Production implementation.
- Conformance testing.
- Independent validation.
- Feedback into architectural design.

Documentation must remain rigorous, but it must serve an actual engineering programme.

### 15.7 Future target architectures

Maintain research coverage for future architectures rather than only currently available CPUs.

Investigate:

- Different processor architectures.
- Heterogeneous systems.
- GPUs.
- TPUs.
- Specialized accelerators.
- FPGA-based computation.
- Distributed hardware.
- Emerging memory architectures.
- Embedded architectures.
- Real-time hardware.
- Hardware-assisted security.

### 15.8 Real-time and resource guarantees

A language intended for demanding systems needs more than ordinary performance benchmarks.

Research:

- Worst-case execution time.
- Bounded allocation.
- Stack limits.
- Scheduling determinism.
- Priority inversion.
- Resource budgets.
- Bounded communication.
- Latency guarantees.
- Interrupt handling.
- Fault containment.

Avoid claiming hard real-time guarantees unless the required execution environment, compiler properties, and hardware assumptions support them.

### 15.9 Side-channel and security analysis

Memory safety and capability checks are not sufficient for all security requirements.

Research:

- Constant-time operations.
- Speculative execution.
- Cache side channels.
- Information-flow analysis.
- Secret-dependent control flow.
- Cryptographic implementation safety.
- Compiler transformations affecting secrecy.
- Secure isolation.

### 15.10 Distributed and fault-tolerant semantics

The language's intended domains justify serious investigation into:

- Message delivery.
- Distributed ownership.
- Failure detection.
- Consensus assumptions.
- Crash recovery.
- Persistent state.
- Transactions.
- Replication.
- Network partitions.
- Byzantine failures.
- Deterministic replay.

No impossible distributed guarantees should be promised.

### 15.11 Language evolution and compatibility

An indefinitely evolving language requires explicit rules for:

- Versioning.
- Editions.
- Deprecation.
- Migration.
- ABI evolution.
- Package compatibility.
- Security updates.
- Compiler bootstrap compatibility.
- Standard-library evolution.
- Profile compatibility.
- Long-term support.

This must be designed early enough to avoid making future evolution impractical.

### 15.12 Developer experience needs independent acceptance criteria

The original vibe-first objective should not be reduced to an informal statement.

Create measurable criteria for common programming tasks and test whether Omni achieves its intended expressiveness and usability.

### 15.13 Environmental independence

The design and implementation must address:

- Operating-system differences.
- Filesystem differences.
- Host and target separation.
- Cross-compilation.
- Toolchain reproducibility.
- Hostile build inputs.
- Offline builds.
- Package provenance.
- Deterministic artifact creation.

### 15.14 Independent implementation

Eventually, at least one independently developed implementation or sufficiently independent semantic model should be capable of validating the same specification and conformance suite.

This reduces dependence on one compiler's interpretation of its own rules.

---

# PART XII — FORMAL PROJECT GOVERNANCE

## 16. Every Requirement Must Have Traceability

Maintain an immutable requirement identification system.

Each important requirement should contain:

1. Requirement ID.
2. Exact requirement statement.
3. Original source of the requirement.
4. Rationale.
5. Priority.
6. Risk classification.
7. Design status.
8. Research evidence.
9. Alternative designs.
10. Formal definition, when necessary.
11. Dependencies.
12. Feature interactions.
13. Implementation mapping.
14. Test mapping.
15. Proof mapping.
16. Validation mapping.
17. Current limitations.
18. Version history.
19. Review status.

Maintain explicit distinctions between:

- User requirements.
- Research findings.
- Proposals.
- Architectural decisions.
- Normative language rules.
- Implementation requirements.
- Compiler implementation facts.
- Tests.
- Proofs.
- Qualification results.
- Unresolved matters.

No agent should silently turn a proposal into an approved requirement.

## 17. Evidence Classification

Every meaningful claim must be classified.

Useful categories include:

- Unverified assertion.
- Sourced claim.
- Corroborated claim.
- Formally defined property.
- Machine-checked proof.
- Implemented behaviour.
- Tested behaviour.
- Independently reviewed result.
- Qualified release claim.
- System-level validation.

These categories are separate.

A program passing tests does not mean its behaviour has been formally proven.

A formally verified model does not establish that the implementation matches it.

An implementation conforming to a language edition does not establish that a complete physical system is certified.

Do not overstate any of these claims.

## 18. Independent Review

Use specialists for:

- Programming-language research.
- Type theory.
- Formal semantics.
- Compiler architecture.
- Memory management.
- Concurrency.
- Distributed systems.
- Security.
- Cryptography.
- Hardware.
- Runtime systems.
- Optimization.
- Testing.
- Developer experience.
- Standards and assurance.

Independent reviewers should actively challenge the designs.

The reviewer responsible for accepting an implementation should not simply repeat the implementer's claims.

A verifier must inspect actual evidence.

The orchestrator must track dependencies and handoffs.

A qualifier must enforce explicit acceptance criteria.

Multi-agent execution should increase research coverage and independent scrutiny, not create duplicated, conflicting work.

---

# PART XIII — THE COMPLETE DEVELOPMENT ROADMAP

## 19. Research and Engineering Stages

The historical three-stage generation roadmap should remain recognizable, but the actual project should use finer dependency-driven milestones.

The following is an architectural roadmap, not a declaration that every feature is ready for immediate implementation.

### Phase 0 — Historical requirements and research foundation

- Recover the original requirements interview.
- Consolidate historical documents.
- Reconcile the two Omni workspaces.
- Identify normative authority.
- Establish a complete requirement registry.
- Identify design conflicts.
- Establish research and evidence policies.
- Create the long-term evolution policy.

**Acceptance:** No major original intention is silently discarded, and every known unresolved issue is identified.

### Phase 1 — Research the minimal semantic core

- Source model.
- Lexical model.
- Grammar.
- Core expressions.
- Primitive types.
- Algebraic data types.
- Name resolution.
- Core typing rules.
- Evaluation semantics.
- Error semantics.
- Initial memory semantics.

**Acceptance:** The core is sufficiently formalized to support a reference implementation and independent semantic examination.

### Phase 2 — Prototype the semantic architecture

- Lexer.
- Parser.
- Reference model.
- Core interpreter or executable semantics.
- Type checker.
- Name resolver.
- Basic IR.
- Initial compiler path.

**Acceptance:** Core semantic claims can be tested through actual programs.

### Phase 3 — Ownership, effects and authority

- Ownership.
- Borrowing.
- Region analysis.
- Effect system.
- Capability model.
- Cleanup semantics.
- Resource tracking.

**Acceptance:** The core safety and authority claims have implemented validation paths and explicit remaining proof obligations.

### Phase 4 — HIR and MIR

- Typed HIR.
- Normative MIR.
- Control-flow analysis.
- Definite initialization.
- Ownership state.
- Borrow analysis.
- Drop elaboration.
- Verification.

**Acceptance:** Real programs can be lowered into verified intermediate representations.

### Phase 5 — Machine and execution semantics

- Abstract machine.
- Numeric semantics.
- Memory semantics.
- Fault behaviour.
- Resource exhaustion.
- Execution tracing.

**Acceptance:** An executable reference model or equivalent independent semantic oracle exists for the selected core.

### Phase 6 — Native backend

- Backend research.
- Machine lowering.
- Object emission.
- Linker integration.
- ABI.
- Platform contracts.
- Native executable production.

**Acceptance:** Actual Omni programs compile and execute natively while satisfying the selected core conformance suite.

### Phase 7 — Runtime and core library

- Runtime initialization.
- Memory services.
- Resource management.
- Core standard library.
- I/O providers.
- Error handling.
- Execution services.

**Acceptance:** The language can support real, integrated applications rather than demonstrations alone.

### Phase 8 — Concurrency

- Task scopes.
- Structured concurrency.
- Cancellation.
- Channels.
- Synchronization.
- Scheduling.
- Parallel operations.

**Acceptance:** Concurrency rules are executable, tested, and supported by the intended runtime and static analysis.

### Phase 9 — Development tooling

- CLI.
- Formatter.
- LSP.
- IDE integration.
- Diagnostics.
- Debugger.
- Profiler.
- Test runner.
- Documentation.

**Acceptance:** A programmer can use Omni for practical projects with a coherent development workflow.

### Phase 10 — Package and build platform

- Package manager.
- Dependency resolver.
- Hermetic builds.
- Reproducibility.
- Artifact signing.
- Release management.
- Supply-chain controls.

**Acceptance:** Omni projects and dependencies can be managed reproducibly.

### Phase 11 — Advanced language capabilities

Subject to the research and dependency gates:

- Advanced generics.
- User-defined effects.
- Macros.
- Compile-time computation.
- Reflection.
- Dynamic dispatch.
- Managed memory domains.
- Advanced numerical functionality.
- Tensor operations.
- Plugins.
- Persistent data.
- Specialized execution profiles.

**Acceptance:** Each feature has a researched, defined, implemented, tested, and appropriately qualified status.

### Phase 12 — Hardware and computational expansion

- Additional processor targets.
- SIMD.
- GPUs.
- Accelerators.
- Embedded targets.
- Bare-metal targets.
- Real-time profiles.
- Heterogeneous execution.

**Acceptance:** Target-specific behaviour is supported by explicit contracts and appropriate hardware validation.

### Phase 13 — Production platform maturity

- Complete conformance.
- Security assessment.
- Performance evaluation.
- Cross-platform validation.
- Reproducible distribution.
- Documentation.
- Toolchain stability.
- Release qualification.

**Acceptance:** Production readiness is demonstrated with evidence for the stated release claims.

### Phase 14 — Self-hosting

- Omni-written compiler components.
- Bootstrap compiler.
- Independent compiler validation.
- Diverse double compilation.
- Reproducibility.
- Trust-chain analysis.

**Acceptance:** Omni can reliably compile its own implementation under the established bootstrap and independent-verification conditions.

### Phase 15 onwards — Continuous evolution

There is no final phase that closes development permanently.

Future work should continuously assess new research, hardware, algorithms, programming-language techniques, compiler architectures, optimization methods, security practices, and mathematical results.

Each future addition should follow the same evidence-driven development process.

---

# PART XIV — SELF-HOSTING AND TRUST

## 20. Self-Hosting Is an Important Milestone, Not the Ultimate Definition of Success

Omni should ultimately be capable of implementing its own compiler and tooling in Omni.

However, self-hosting is not itself proof of correctness.

A compiler that successfully compiles its own source code can still consistently generate incorrect machine code.

Therefore, the bootstrap roadmap must include:

- A trusted initial compiler.
- Reproducible compiler builds.
- Compiler-stage comparison.
- Diverse double compilation.
- Independent compiler lineages.
- Differential tests.
- Conformance validation.
- Bootstrap recovery procedures.
- Toolchain provenance.
- Independent verification.

Rust can remain useful as a bootstrap and validation implementation even after the production compiler is self-hosted.

The long-term objective is to reduce unnecessary dependencies on the bootstrap implementation without sacrificing the ability to validate the compiler itself.

---

# PART XV — NON-NEGOTIABLE DEVELOPMENT RULES

## 21. What Must Never Happen

1. Do not assume that writing a comprehensive specification guarantees a workable language.
2. Do not skip research merely to accelerate implementation.
3. Do not delay all implementation until every future feature is researched.
4. Do not invent semantics when a specification is incomplete.
5. Do not adopt proposals simply because they look elegant.
6. Do not assume theoretical feasibility proves practical usability.
7. Do not declare features complete based solely on isolated tests.
8. Do not claim formal proof without a reproducible proof artifact.
9. Do not claim safety based solely on testing.
10. Do not conceal implementation limitations.
11. Do not weaken tests to achieve green CI.
12. Do not silently change established language semantics.
13. Do not revive disabled syntax.
14. Do not allow historical drafts to override current normative documents.
15. Do not let the research workspace become disconnected from implementation.
16. Do not allow implementation convenience to decide language design.
17. Do not restrict future development to an arbitrary final set of features.
18. Do not assume existing architectures must remain permanent.
19. Do not allow the vibe-first surface language to introduce ambiguous or unsafe hidden behaviour.
20. Do not confuse self-hosting with correctness.
21. Do not claim certification without an applicable external assessment.
22. Do not sacrifice the original user-facing intent while refining the technical design.

---

# PART XVI — FINAL DEFINITION OF SUCCESS

## 22. What Omni Should Ultimately Be

Omni will have achieved its central vision when it has become a practically usable, independently understandable, thoroughly researched, formally specified, implemented, tested, and continuously evolving programming-language platform.

Its developers should be able to write programs using an exceptionally ergonomic, intent-oriented, vibe-first surface language.

Its experts should be able to access sophisticated type-system, memory, concurrency, security, hardware, and execution mechanisms.

Its compiler should translate programs through a coherent, verifiable pipeline into efficient execution artifacts.

Its runtime should provide the services required by the selected execution profile.

Its standard library and ecosystem should make it practical to develop serious real-world systems.

Its safety and security claims should be supported by appropriate formal definitions, implementation evidence, tests, independent review, and system-level validation.

Its computational models should allow it to address a very broad range of programming domains.

Its architecture should be sufficiently modular and extensible that advances in programming-language research, compiler engineering, hardware, mathematics, algorithms, and security can be adopted without arbitrary restrictions.

Its development must continue indefinitely as long as further improvements are possible and beneficial.

### Ultimate principle

**Omni is not being developed merely to become another programming language. It is being developed to become an exceptionally capable, rigorously engineered, developer-friendly, high-performance and continuously evolving programming-language platform whose design and implementation are repeatedly challenged by research and validated through actual engineering evidence.**

---

# PART XVII — MASTER REQUEST TO THE OMNI RESEARCH AND ENGINEERING TEAM

## 23. Complete Project Instruction

I want you to take responsibility for researching, designing, implementing, testing, validating, and continuously improving Omni according to this complete vision.

Do not interpret this request as an instruction to take an existing large specification, translate it directly into code, and consider the result complete.

That is specifically not what I want.

First understand the original purpose of Omni from its conception until the present.

Recover and inspect the original requirements, historical conversations where available, research documents, previous specifications, architectural proposals, rejected designs, implementation history, existing repositories, tests, and documented user decisions.

Do not discard requirements just because they are missing from the latest consolidated specification.

Do not adopt historical requirements without checking whether they were superseded.

Preserve all relevant historical decisions and record how each is resolved.

### Research mandate

Research Omni as an independent programming-language research and development project.

Investigate the mathematics, theories, algorithms, formal methods, type systems, memory models, compiler techniques, execution architectures, programming-language designs, and hardware technologies relevant to achieving the intended language.

Do not assume the current architecture is necessarily optimal.

Do not copy another language's design without examining its underlying reasoning and limitations.

Compare different approaches and document why a particular solution is superior for Omni's requirements.

Where necessary, develop executable models and prototypes to test whether a proposed design is actually feasible.

### Implementation mandate

Implement the language progressively, beginning with a research-supported core.

Use implementation to challenge the research and specifications.

Test whether proposed semantics can actually be represented, compiled, optimized, and executed.

Investigate real compiler behaviour, not merely whether code compiles.

Build an actual end-to-end compiler and runtime.

Make the compiler reject invalid programs and produce the correct behaviour for valid programs.

Do not substitute mock implementations, placeholder functionality, fake successes, or fallback logic for actual features.

### Validation mandate

Every important implementation must have an explicit validation strategy.

Run unit, integration, conformance, property-based, fuzz, differential, regression, and end-to-end tests where applicable.

Use formal verification when justified.

Use independent implementations and verification tools where appropriate.

Measure actual compiler performance, generated-code performance, runtime overhead, memory consumption, resource behaviour, and developer experience.

Record the precise environment and tool versions used for validation.

Distinguish successful tests from proof, independent review, and system-level qualification.

Whenever an implementation fails to validate an intended property, return to the research and design process.

Determine whether the implementation is wrong, the design is incomplete, the specification is ambiguous, or the requirement itself needs reconsideration.

Correct the root cause.

### Programming experience mandate

Make the actual programming syntax of Omni as vibe-first as reasonably achievable.

Programmers should be able to express their intentions naturally and concisely using the language itself.

Minimize unnecessary boilerplate and repetitive annotations.

Maximize safe inference, high-level abstractions, discoverability, expressiveness, readability, and ease of modification.

Ensure that advanced developers can still access and control low-level implementation details.

Do not confuse this requirement with implementing Omni through AI or writing compiler code in natural-language prompts.

### Continuous development mandate

There is no fixed final scope or development endpoint.

Continue researching and improving Omni as relevant technologies, techniques, algorithms, theories, logical systems, and hardware architectures evolve.

Reassess old decisions periodically.

Research better alternatives for existing systems.

Add capabilities when they offer justified benefits.

Retire or replace inferior mechanisms through controlled evolution.

Keep the language and its ecosystem technologically current without compromising reproducibility or backwards-compatibility commitments.

### Project management mandate

Maintain clear, traceable requirements and decisions.

Preserve the relationship between the original conception, research workspace, normative specification, implementation repository, tests, and qualification evidence.

Work in dependency order.

Use independent review.

Never report unverified results as successful.

Never confuse proposed, implemented, tested, proved, and qualified functionality.

Update authoritative documents whenever an approved design decision or implementation status materially changes.

Maintain the required Git workflow and preserve the repository's history.

### Final objective

Deliver the complete Omni programming-language platform, including its language definition, compiler, runtime, standard library, package and build systems, development tooling, target support, and required verification infrastructure.

The exact contents and order must be determined by continuous research and dependency analysis rather than an assumption that every feature can be specified correctly in advance.

Build Omni in a way that proves its designs are implementable and that the implemented language behaves according to its intended semantics.

Do not stop at an attractive specification, a working parser, a partial compiler, or a self-hosting demonstration.

The goal is a real, usable, independently validated programming language platform.

And after reaching that level, continue improving it.

**There is no final development ceiling for Omni.**