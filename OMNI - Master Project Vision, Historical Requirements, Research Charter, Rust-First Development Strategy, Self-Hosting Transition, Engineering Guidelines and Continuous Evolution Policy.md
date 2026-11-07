# OMNI
# Master Project Vision, Requirements, Architecture, Research and Engineering Charter, Development Strategy, Versioning and Release Governance, and Continuous Evolution Policy

**Document status:** Expanded and revised master project charter.

**Project:** Omni Programming Language and Compiler Platform.

**Canonical implementation repository:** https://github.com/shreyashjagtap157/Omni---New

**Canonical implementation branch:** `main`.

**Historical implementation repository:** https://github.com/shreyashjagtap157/Omni

**Independent research workspace:** `C:\Users\siddh\Downloads\ABC\Ideas\Omni-Research`

**Primary development strategy:** Approach C — Rust-first production implementation, experimental self-hosting in parallel, followed by a controlled transition to an Omni-written compiler.

**Versioning standard:** Four-component project release version `X.Y.Z.W`, with distinct engineering milestone, specification, crate, ABI, runtime and artifact identities.

**Project duration:** Continuous. There is no predetermined final development endpoint.

**Authority:** This is the master project-goal and development-governance document. It does not silently replace or amend the current normative Edition 1 specification. Changes to normative grammar, semantics, schemas, release manifests and qualification policy require the repository's established controlled process.

---

# PART I — THE APPROVED PROJECT GOAL

## 1. The Goal

The goal of Omni remains the one approved by the user:

**Omni is intended to become an exceptionally capable, general-purpose, multi-paradigm, multi-level, multi-runtime programming language and its supporting development platform, bringing together advanced programming-language theory, type systems, memory management, compiler architecture, execution models, concurrency, security, formal verification, performance engineering, hardware abstraction and developer experience.**

The intention is to develop a language that meaningfully advances programming-language design and supports an exceptionally broad range of programming tasks.

It should enable developers to express high-level intent naturally while retaining precise control over low-level execution, memory, resources, security, hardware and performance whenever necessary.

It should not be a narrow derivative of another programming language.

It should not become limited to the implementation requirements of its own compiler.

It should not become an AI-dependent programming language.

It should not stop evolving after a particular release or self-hosting milestone.

**All other requirements in this document are additive engineering, research, governance, validation and implementation provisions in support of this goal. They are not separate or competing project goals and must not narrow the approved ambition.**

## 2. Foundational Philosophy

The original design philosophy remains:

**Everything, but structured.**

The language should have a rigorous and comprehensible semantic foundation while offering a very broad range of abstractions and computational facilities.

This requires:

1. A coherent semantic core.
2. Composable type and effect systems.
3. Flexible memory and ownership mechanisms.
4. Appropriate execution abstractions.
5. Explicit authority and resource management.
6. Multiple abstraction levels.
7. Multiple target and runtime profiles.
8. A substantial standard library.
9. A modular compiler and toolchain.
10. Excellent programmer ergonomics.
11. Strong correctness and security objectives.
12. Permanent research and technological evolution.

---

# PART II — UNLIMITED EVOLUTION AND SCOPE

## 3. No Fixed Development Ceiling

Omni must never have an artificial final development endpoint.

Its development must continue as relevant advances become available through:

- Programming-language theory.
- Mathematics.
- Logic and formal methods.
- Algorithms.
- Compilers.
- Optimization.
- Type systems.
- Memory management.
- Operating systems.
- Runtime engineering.
- Concurrency.
- Distributed computing.
- Cybersecurity.
- Cryptography.
- Hardware.
- Processor architectures.
- GPUs and accelerators.
- Numerical computing.
- Software engineering.
- Developer-experience research.

The project must continuously reassess existing mechanisms and investigate better replacements.

### 3.1 Existing decisions are not permanently immune to improvement

A subsystem must not be considered optimal merely because it works.

Research whether better alternatives have become practical.

Examples include:

- Better type inference.
- More precise ownership analysis.
- Better borrow-checking algorithms.
- Improved effect systems.
- More ergonomic memory management.
- Faster intermediate representations.
- More effective optimization.
- Better native backends.
- Better incremental compilation.
- More reliable concurrency facilities.
- Better debugging and diagnostics.

Adoption must be supported by evidence and controlled compatibility procedures.

### 3.2 Stable releases and continuous development

Omni should have stable, immutable, reproducible releases while its overall architecture continues to evolve.

A release may be frozen.

The project must not be.

### 3.3 No artificial feature ceiling

The project must not reject useful future features simply because they were not contemplated in the original specification.

New features must be researched, evaluated and introduced through controlled edition, profile or extension governance.

### 3.4 No artificial completion requirement

Do not require every intended future technology, runtime, hardware architecture or application domain to exist before Omni becomes a usable programming language.

The project must be able to deliver progressively more capable, independently useful editions.

---

# PART III — PROGRAMMER EXPERIENCE

## 4. Vibe-First Programming Through Actual Omni Syntax

Vibe-first programming remains a core requirement.

This means that Omni source code should make expressing programmer intent as natural, concise, intuitive and readable as technically possible.

It concerns the actual language syntax and programming abstractions.

It does not mean AI-generated implementations, natural-language prompts as mandatory source code, or nondeterministic semantic interpretation.

### 4.1 Intent-oriented programming

Programmers should primarily express:

- What should happen.
- What data is involved.
- What transformation is intended.
- What result is required.
- What constraints must hold.
- How components interact.
- What resources or authority are necessary.

The compiler should infer implementation details wherever the answer is unique, well-defined and sound.

### 4.2 Reduce unnecessary ceremony

Research ways to reduce:

- Repeated type annotations.
- Boilerplate.
- Explicit generic arguments where inferable.
- Redundant conversions.
- Manual cleanup.
- Repetitive iteration.
- Repeated ownership annotations.
- Unnecessary intermediate values.
- Verbose error propagation.
- Repeated configuration.
- Unnecessary concurrency setup.

### 4.3 Safe inference

Inference must be deterministic.

The compiler must not guess when several materially different interpretations remain.

It must not silently infer privileged authority, ownership transfer, unsafe conversions or ambiguous overloads.

### 4.4 Multiple programming levels

Support:

**High-level programming:** concise, intent-oriented and expressive.

**Intermediate-level programming:** explicit control of types, effects, ownership, constraints and execution.

**Low-level programming:** precise representation, memory, ABI, allocation, hardware and synchronization control.

These levels must share a coherent language model.

### 4.5 Syntax experimentation

Research actual source-language ergonomics independently.

Do not assume that existing language syntax is optimal.

Measure:

- Readability.
- Learnability.
- Expressiveness.
- Error rates.
- Ambiguity.
- Parser complexity.
- Refactoring.
- Formatting.
- Diagnostic quality.
- Developer productivity.

Candidate 2 and later syntax proposals remain subject to normative adoption. A master vision does not activate a pending amendment.

---

# PART IV — HISTORICAL REQUIREMENTS

## 5. Original Requirements

The historical design records report an extensive requirements interview involving more than 160 design decisions.

Recover the original questionnaire and answers where possible.

Retain the historical records as evidence, including:

- Original requirement.
- Original reasoning.
- Subsequent changes.
- Conflicting answers.
- Later decisions.
- Current normative status.

Do not assume the latest specification contains every original intention.

## 6. Established Design Areas

### 6.1 Types

Research:

- Static typing.
- Bidirectional checking.
- Type inference.
- Algebraic data types.
- Pattern matching.
- Generics.
- Traits.
- Interfaces.
- Associated types.
- Variance.
- Refinements.
- Function types.
- Recursive types.
- Existential and universal types.
- Type-level computation.
- Explicit dynamic facilities where justified.

### 6.2 Memory and ownership

Research:

- Affine and linear ownership.
- Borrowing.
- Regions.
- Arenas.
- Reference counting.
- Garbage collection.
- Managed domains.
- Pinning.
- Generational references.
- Persistent memory.
- Device memory.
- Custom allocators.
- Memory provenance.
- Allocation identity.
- Resource cleanup.

**Unresolved historical conflict:** Earlier user intent described managed memory as the default with opt-in ownership, regions and linearity. The current Edition 1 candidate uses affine ownership as a core model.

This must be investigated and explicitly resolved. Neither intention may be silently discarded.

### 6.3 Effects

Research:

- I/O.
- Filesystem.
- Networking.
- Allocation.
- Time.
- Randomness.
- Blocking.
- Async suspension.
- Cancellation.
- Synchronization.
- Persistence.
- Foreign calls.
- Device access.
- Reflection.
- Nondeterminism.
- Unsafe operations.
- User-defined effects and handlers.

### 6.4 Capability security

Research:

- Unforgeable authority.
- Issuance.
- Attenuation.
- Delegation.
- Revocation.
- Quotas.
- Scope.
- Auditing.
- Resource access.
- Device access.
- Plugin authority.

Effects classify computational behaviour; they do not independently grant authority.

### 6.5 Errors and resource management

Define:

- Option and Result.
- Expected failures.
- Panic.
- Trap.
- Fault.
- Unwinding.
- Abort.
- Isolation.
- OOM.
- Stack exhaustion.
- Cancellation.
- Cleanup.
- Double-fault behaviour.

### 6.6 Concurrency

Research:

- Virtual threads and lightweight tasks.
- Structured concurrency.
- Async suspension.
- Native threads.
- Channels.
- Actors.
- Work stealing.
- Scheduling.
- Cancellation.
- Atomics.
- Memory ordering.
- Parallel iteration.
- Distributed execution.

**Unresolved historical conflict:** The original requirement preferred a coherent programming model rather than an unnecessary division between synchronous and asynchronous programming. Later candidate material includes async constructs. Reconcile these approaches through research.

### 6.7 Modules and packages

Research modules, packages, imports, visibility, workspaces, dependency resolution, lockfiles, signed packages, registries, reproducibility, offline operation and provenance.

### 6.8 Numerics and scientific computation

Research exact and approximate arithmetic, fixed-width integers, floating-point behaviour, decimal, fixed-point, units, SIMD, matrices, tensors, accelerators, numerical stability and reproducibility.

### 6.9 Runtime and compiler

Research AOT, JIT, interpreters, runtime services, target-neutral IR, optimization, native code generation, object files, linking, debugging and profiling.

### 6.10 Developer tooling

Develop toward a unified CLI, formatter, linter, language server, debugger, test runner, benchmark tooling, documentation generator, static analysis, refactoring and package manager.

---

# PART V — ARCHITECTURE AND TECHNICAL FOUNDATIONS

## 7. Semantic Foundations

The language must define what a program means.

This requires explicit treatment of:

- Static semantics.
- Operational semantics.
- Evaluation order.
- Expression sequencing.
- Observable behaviour.
- Integer semantics.
- Floating-point semantics.
- Memory semantics.
- Aliasing.
- Pointer provenance.
- Data representation.
- Traps.
- Panics.
- Faults.
- Concurrency.
- Atomicity.
- Memory ordering.
- Resource management.
- Termination assumptions.
- Undefined behaviour.
- Implementation-defined behaviour.

Where formalization is important, define:

- Typing judgments.
- Transition systems.
- State invariants.
- Evaluation rules.
- Memory models.
- Proof obligations.
- Resource bounds.

No backend or implementation convenience may create independent language semantics.

## 8. Type-System Research

The type-system research programme should consider:

- Type inference and decidability.
- Unification.
- Recursive types.
- Generics.
- Trait coherence.
- Overload resolution.
- Specialization.
- Associated types.
- Higher-ranked types.
- Effect polymorphism.
- Variance.
- Refinement types.
- Existential and universal quantification.
- Type-level computation.
- Compile-time constraints.

Do not assume every advanced type-system concept belongs in the initial production edition.

Evaluate each feature by utility, soundness, decidability, implementation complexity and developer experience.

## 9. Compile-Time Computation and Metaprogramming

Treat these as distinct research domains:

- Constant evaluation.
- Compile-time functions.
- Declarative macros.
- Procedural macros.
- Hygienic expansion.
- Reflection.
- Code generation.
- Type-level computation.
- Compiler plugins.
- Compile-time resource limits.
- Reproducibility.
- Sandboxing.

Prefer predictable and deterministic compile-time mechanisms.

Unrestricted compiler plugins or arbitrary build-time execution must not be introduced without a clear security model.

## 10. Memory Architecture

Compare the strengths, limitations and interactions of:

- Ownership and borrowing.
- Region-based memory.
- Reference counting.
- Tracing garbage collection.
- Hybrid collection.
- Managed domains.
- Arenas.
- Pools.
- Stack allocation.
- Heap allocation.
- Persistent memory.
- Shared memory.
- Device memory.
- Custom allocators.

Study interoperability between memory models.

The objective is a usable and flexible language that offers safe defaults without unnecessarily restricting low-level programming.

## 11. Concurrency and Parallel Execution

Establish a coherent programming model while allowing different internal execution mechanisms.

Potential execution agents include:

- Native threads.
- Lightweight tasks.
- Async tasks.
- Actors.
- Interrupt handlers.
- GPU execution.
- Distributed tasks.

Research:

- Structured lifetimes.
- Cancellation.
- Cleanup.
- Deadlock.
- Starvation.
- Fairness.
- Priority inversion.
- Isolation.
- Data races.
- Atomics.
- Memory ordering.
- Task failure propagation.
- Resource bounds.

A unified programming experience does not require identical scheduling semantics across all platforms.

## 12. Numeric and Computational Model

Define cross-target behaviour for:

- Integer overflow.
- Signedness.
- Floating-point rounding.
- NaNs.
- Subnormals.
- Fused operations.
- SIMD.
- Decimal.
- Fixed point.
- Arbitrary precision.
- Complex arithmetic.
- Units.
- Vectors.
- Matrices.
- Tensors.
- Accelerator execution.
- Reproducibility.

Numerical differences between targets must be explicitly permitted or prohibited by the relevant specification.

## 13. ABI, FFI and Interoperability

Research:

- C ABI.
- Calling conventions.
- Object formats.
- Symbol naming.
- Struct and enum layout.
- Parameter and return-value representation.
- Static and dynamic linking.
- Foreign ownership.
- Error boundaries.
- Unwinding.
- Plugin interfaces.
- System library integration.
- Cross-language memory safety.

Practical interoperability is essential to a useful programming ecosystem.

## 14. Runtime Architecture

Support research into:

- Hosted execution.
- Freestanding execution.
- Bare-metal execution.
- Kernel execution.
- Embedded runtime.
- Real-time runtime.
- Sandboxed execution.
- Interpreter.
- JIT.
- AOT.
- Runtime-free compilation.
- Accelerator runtime.

Do not require every programme to depend on a large hosted runtime.

## 15. Standard Library

Develop a layered standard library:

1. Fundamental language-provided operations.
2. Core types and traits.
3. Allocation-dependent facilities.
4. Operating-system interfaces.
5. Concurrency and async.
6. Networking and distributed computing.
7. Numerical and accelerator facilities.
8. Optional ecosystem packages.

The exact boundaries should be researched and formalized.

Freestanding applications must not be forced to depend on hosted-only services.

## 16. Compiler Architecture

The intended compiler pipeline should be developed with explicit stage contracts:

Source → Lexing → CST → Parsing → AST → Name Resolution → Type and Effect Analysis → Ownership Analysis → HIR → MIR → MIR Verification → Optimization → LIR → Code Generation → Object Emission → Linking → Runtime.

Every stage must define:

- Inputs.
- Outputs.
- Invariants.
- Failure conditions.
- Diagnostics.
- Source provenance.
- Resource ownership.
- Semantic responsibilities.

The actual pipeline may evolve, provided the language semantics and validation obligations remain intact.

### 16.1 Intermediate representations

Research and implement:

- Lossless syntax representation.
- Semantic AST.
- HIR.
- Place-based MIR.
- Target-neutral LIR.
- Machine-specific representation.
- Debug and source mapping.
- Optimization metadata.
- Assumption provenance.

### 16.2 Reference machine

The abstract machine must be more than a demo interpreter.

It should act as an executable semantic model for applicable features.

Its coverage must include memory identity, initialization, provenance, ownership, control flow, errors and relevant concurrency semantics.

### 16.3 Backends

Cranelift is an existing implementation direction, not an irrevocable permanent choice.

Evaluate backends using:

- Correctness.
- Target support.
- Code quality.
- Compile time.
- Binary size.
- Debugging.
- Incremental compilation.
- Reliability.
- Security.
- Maintainability.

LLVM, MLIR, custom code generation and other architectures remain research possibilities.

Do not prematurely maintain several production backends.

## 17. Debugging, Observability and Incremental Compilation

Add explicit support for research into:

- Source-level debugging.
- Stack traces.
- Async/task traces.
- Symbol information.
- Profiling.
- Memory tracing.
- Compiler telemetry where privacy permits.
- Incremental query systems.
- Dependency invalidation.
- Cache correctness.
- Reproducible incremental builds.

Caches must never change program semantics.

---

# PART VI — EXECUTION PROFILES AND DOMAIN COVERAGE

## 18. One Language, Multiple Execution Environments

Omni should support coherent language semantics across multiple environments without requiring every environment to provide identical facilities.

Research explicit profiles for:

- General hosted development.
- Server and enterprise applications.
- Systems programming.
- Freestanding and bare-metal.
- Embedded.
- Real-time.
- High-assurance.
- Accelerator and GPU.
- Scientific computing.
- Distributed applications.

Each approved profile should specify:

- Available language features.
- Required runtime support.
- Effects.
- Capabilities.
- Memory model.
- Target requirements.
- ABI.
- Resource limitations.
- Concurrency facilities.
- Unsafe-operation constraints.
- Qualification expectations.

These names and boundaries remain research proposals until approved.

## 19. Release Scope and Domain Coverage

A particular release must explicitly declare what it supports.

Do not require the first production release to support every processor, accelerator, real-time system, operating system or certification regime.

Each domain should have a development and qualification roadmap.

This is a delivery strategy, not a restriction on Omni's final ambition.

## 20. HELIOS

HELIOS remains an important intended application of Omni, including provenance, confidence, immutable history, capability security, explainable reasoning, extensible components and offline-first operation.

It must provide representative validation workloads.

It must not become the sole design target of Omni.

---

# PART VII — RESEARCH AND ENGINEERING METHODOLOGY

## 21. Research Must Inform Implementation

Omni must be built through a research-and-engineering process.

The cycle is:

**Requirements → Research → Competing designs → Formal models → Prototypes → Evaluation → Normative specification → Production implementation → Verification → Qualification → Feedback.**

### 21.1 Research requirements

Establish the actual problem, intended domains, guarantees, constraints, costs and interactions.

### 21.2 Literature and implementation research

Study primary academic sources, standards, language proposals, compiler source, runtime implementations and hardware documentation.

### 21.3 Competing designs

Compare approaches by:

- Correctness.
- Soundness.
- Decidability.
- Termination.
- Complexity.
- Compilation time.
- Runtime performance.
- Memory.
- Developer experience.
- Security.
- Maintainability.
- Interactions.

### 21.4 Executable prototypes

Where useful, create:

- Grammar experiments.
- Reference interpreters.
- Type solvers.
- Borrow-checking prototypes.
- Effect models.
- Concurrency simulations.
- Compiler IR experiments.
- Optimization prototypes.
- Backend experiments.
- Benchmarks.

### 21.5 Formalization

Define semantics and invariants before relying on implementation convention.

### 21.6 Evaluation

Test valid and invalid examples, boundary cases, interactions, termination, resource use, performance and usability.

### 21.7 Controlled adoption

Only approved designs become normative.

### 21.8 Implementation feedback

If implementation exposes a design problem, revisit the research and architecture.

Do not weaken tests or invent semantics merely to make implementation easier.

## 22. Feature Engineering Lifecycle

Every substantial feature follows this lifecycle:

1. Requirement established.
2. Research completed.
3. Alternatives compared.
4. Architecture reviewed.
5. Formal model established where applicable.
6. Prototype developed.
7. Evaluation completed.
8. Design approved.
9. Normative specification updated.
10. Production implementation integrated.
11. Conformance and regression evidence completed.
12. Independent review completed where required.
13. Qualification status recorded.
14. Long-term maintenance and evolution established.

A feature may return to earlier stages when new evidence invalidates an assumption.

## 23. Feature-Interaction Testing

Maintain an explicit interaction register.

Examples include:

- Ownership with async cancellation.
- Generic traits with specialization.
- Effects with capability delegation.
- FFI with borrowed references.
- Exception cleanup with resource management.
- GPU memory with async tasks.
- Macros with source maps.
- Optimization with floating-point semantics.

Individually passing feature tests do not prove valid interactions.

---

# PART VIII — DEVELOPMENT STRATEGY: APPROACH C

## 24. Rust-First Primary Implementation with Parallel Self-Hosting

Approach C remains the official development direction.

The primary Rust implementation must establish a reliable and complete enough programming language before the official self-hosting transition.

Experimental self-hosting can proceed in parallel without becoming a release prerequisite.

### 24.1 Track A — Primary Rust implementation

Responsible for:

- Language implementation.
- Compiler.
- Reference machine.
- Runtime.
- Standard library.
- Native backend.
- Tooling.
- Conformance.
- Production qualification.

### 24.2 Track B — Experimental self-hosting

Responsible for:

- Omni-written lexer.
- Omni-written parser.
- Compiler data structures.
- Semantic analysis experiments.
- IR experiments.
- Differential comparison.
- Language suitability for compiler development.

### 24.3 Track C — Research and assurance

Responsible for:

- Continuous research.
- Formal models.
- Independent review.
- Security.
- Performance.
- Architecture reassessment.
- Formal verification.

### 24.4 Interaction

If an Omni-written compiler experiment exposes a missing language capability:

1. Document the actual requirement.
2. Research alternatives.
3. Prototype.
4. Formalize.
5. Approve the design.
6. Implement in Rust.
7. Validate.
8. Update the Omni-written component.

The self-hosted implementation must not create a second semantic authority.

## 25. Rust Is the Bootstrap Strategy, Not a Permanent Restriction on Every Component

Rust is the primary initial compiler implementation language.

It is not a permanent rule that every future runtime service, native interface, hardware provider or ecosystem package must be written in Rust.

External components and alternative implementation languages may be used where technically justified, provided their boundaries, safety assumptions, ABI and trust implications are explicit.

The Rust reference implementation should remain available after self-hosting where practical.

---

# PART IX — THREE-STAGE DEVELOPMENT ROADMAP

## 26. Stage 1 — Rust Minimal Working Core

**Objective:** Establish a defined, executable, correctly constrained subset.

Implement:

- Source handling.
- Lexer.
- Parser.
- CST and AST.
- Name resolution.
- Core typing.
- Core ownership.
- HIR.
- MIR.
- MIR verification.
- Reference machine.
- Minimal native backend.
- Minimal runtime.
- Basic CLI.
- Conformance infrastructure.
- Reproducibility and provenance.

The Stage-1 subset must be explicitly defined.

Acceptance requires real programs to pass through the supported compilation path and execute with validated results.

A parser, AST or MIR model alone is not a completed language implementation.

## 27. Stage 2 — Production-Capable Rust Platform

**Objective:** Make Omni a genuinely usable programming language within a declared release scope.

Complete the applicable mandatory Edition 1 features, compiler, runtime, standard library and essential tooling.

This includes:

- Frontend.
- Types and generics.
- Ownership and borrowing.
- Effects and capabilities.
- Errors and cleanup.
- HIR and MIR.
- Verifier.
- Reference execution.
- Native code generation.
- Linking.
- ABI.
- Runtime.
- Essential standard library.
- CLI.
- Build system.
- Formatter.
- Linter.
- Test runner.
- Diagnostics.
- Documentation.
- Language-server foundation.

### 27.1 Stage-2 production acceptance

Demonstrate meaningful applications such as:

- A multi-module CLI application.
- A system-oriented application.
- A concurrent application.
- A numerical or data-processing application.
- A non-trivial compiler or developer tool.

Evaluate actual executable behaviour, conformance, resource safety, diagnostics, performance, usability, security and reproducibility.

The release must declare its supported targets, profiles, limitations and runtime dependencies.

Stage 2 does not require every future language feature or domain.

## 28. Stage 3 — Official Self-Hosting

Progressively implement the compiler and supporting toolchain in Omni.

Suggested migration dependency order:

1. Source representation.
2. Lexer.
3. Parser.
4. Name resolution.
5. Type checking.
6. Effects and capabilities.
7. Ownership.
8. HIR and MIR.
9. Verification.
10. Optimization.
11. Backend integration.
12. Driver and CLI.
13. Runtime and tools.

Keep the Rust reference implementation and compare the two implementations.

### 28.1 Distinguish self-hosting from total independence

Track four achievements independently:

1. Compiler implemented in Omni and self-compiling.
2. Trustworthy and reproducible bootstrap.
3. Toolchain independence from the original host-language compiler.
4. Broader ecosystem independence.

Using an external linker, assembler, SDK or operating-system service does not automatically mean the compiler is not self-hosted.

Total Rust dependency elimination must be separately justified and qualified.

---

# PART X — PARALLEL SELF-HOSTING EXPERIMENTS

## 29. Existing Self-Hosting Work

The repository contains:

- `compiler/omni-selfhost/lexer.omni`
- `compiler/omni-selfhost/parser.omni`
- `compiler/omni-selfhost/README.md`

Retain these as experimental components.

The current prototypes demonstrate limited lexer and parser functionality.

They are not a complete compiler or proof of self-hosting.

### 29.1 Lexer completion requirements

Investigate and implement the normative requirements for:

- Keywords.
- Unicode.
- Identifiers.
- Numeric literals.
- Strings.
- Characters.
- Raw strings.
- Escapes.
- Comments.
- Trivia.
- Source spans.
- Errors.
- Recovery.
- Malformed input.

### 29.2 Parser completion requirements

Investigate and implement:

- Complete grammar coverage.
- Precedence.
- Associativity.
- Patterns.
- Declarations.
- Modules.
- Types.
- Expressions.
- Error recovery.
- Diagnostics.
- Invalid syntax.
- CST/AST invariants.

### 29.3 Differential testing

Compare Rust and Omni-written components on identical inputs.

Compare tokens, spans, syntax trees, diagnostics, semantics and execution results.

Differences must be investigated.

### 29.4 No independent semantic authority

The normative grammar and semantics remain authoritative.

A prototype must not redefine the language.

---

# PART XI — FORMAL CORRECTNESS AND ASSURANCE

## 30. Evidence Categories

Keep these states distinct:

- Required.
- Proposed.
- Researched.
- Formally specified.
- Prototype implemented.
- Production implemented.
- Tested.
- Independently reviewed.
- Machine-checked.
- Qualified.
- Released.

A passing test does not prove correctness.

A formal model does not prove implementation equivalence.

Self-hosting does not prove the absence of compiler compromise.

A compiler's safety does not automatically guarantee the safety of its generated applications.

## 31. Layered Assurance

Establish:

- Formal semantic definitions for mandatory features.
- Soundness obligations for foundational type and ownership rules.
- Machine-checked proofs for selected critical properties.
- Translation validation for important compiler transformations.
- Property-based testing.
- Fuzzing.
- Differential testing.
- Independent review.
- Security analysis.
- Residual-risk records.

Do not impose an impossible requirement for complete formal proof of every subsystem before the initial production release.

Use assurance levels appropriate to the feature, target and application domain.

## 32. Safety and Certification Boundaries

Omni may provide mechanisms that support high-assurance software development.

However, using Omni does not automatically certify a program, device, aircraft, vehicle, medical system or other physical system.

Each deployment must still undergo its own appropriate hazard analysis, verification, validation and compliance assessment.

---

# PART XII — TESTING AND VALIDATION

## 33. Mandatory Validation Methods

Use:

- Unit testing.
- Integration testing.
- End-to-end testing.
- Conformance testing.
- Property-based testing.
- Fuzz testing.
- Differential testing.
- Regression testing.
- Performance testing.
- Cross-target testing.
- Reproducibility testing.
- Security testing.
- Usability evaluation.

### 33.1 Native execution evidence

Actually execute generated binaries.

Check meaningful observable behaviour, not just successful compilation or process exit codes.

### 33.2 Mutation and test-quality assessment

Where useful, use mutation testing and other test-quality measures to detect weak assertions and untested semantic obligations.

### 33.3 Resource-exhaustion testing

Test oversized, malformed and adversarial inputs.

Ensure compiler and runtime failures are controlled and diagnosable.

### 33.4 Performance evidence

Track compilation time, incremental compilation, memory use, runtime throughput, latency, executable size, code generation and concurrency overhead.

Use reproducible benchmarks.

---

# PART XIII — SECURITY AND SUPPLY-CHAIN GOVERNANCE

## 34. Security Is a Core Engineering Concern

Research and implement:

- Memory safety.
- Type safety.
- Concurrency safety.
- Capability security.
- FFI security.
- Compiler input hardening.
- Resource-exhaustion protection.
- Build isolation.
- Plugin isolation.
- Dependency security.
- Artifact integrity.
- Cryptographic provenance.
- Side-channel analysis.
- Runtime isolation.

### 34.1 Dependency requirements

Every dependency should have:

- Purpose.
- Version constraint.
- License.
- Security status.
- Compatibility evidence.
- Upgrade policy.
- Replacement strategy.

### 34.2 Dependency updates

Do not adopt major compiler-backend updates merely because Dependabot proposes them.

Evaluate:

- Compiler API changes.
- Generated-code changes.
- Semantic correctness.
- Target compatibility.
- Performance.
- Build reproducibility.
- Security.
- Bootstrap effects.

### 34.3 Security advisories

Track each advisory to a documented disposition:

- Affected or unaffected.
- Reachable or unreachable.
- Exploitability.
- Mitigation.
- Patch.
- Residual risk.
- Verification.

An ignored advisory must have a documented reason and an owner.

---

# PART XIV — VERSIONING STANDARD

## 35. Project Release Version: `X.Y.Z.W`

Omni shall use a four-component project release version:

```text
X.Y.Z.W
```

This is the canonical product and project release identifier.

The components have the following meanings:

| Component | Meaning | Purpose |
|---|---|---|
| X | Stable release generation | Identifies the stable release family or major generational platform transition |
| Y | Major release | Identifies a major release within the current release generation |
| Z | Minor release | Identifies a compatible feature release |
| W | Patch | Identifies a compatible bug-fix or maintenance release |

The components are independent and must not be casually incremented.

### 35.1 X — Stable release generation

The first component identifies the stable release generation.

It changes when Omni enters a new major generational release family or a deliberately qualified platform generation.

Examples:

- `1.x.x.x` — first stable release generation.
- `2.x.x.x` — second stable release generation.
- `3.x.x.x` — third stable release generation.

A new X generation requires an explicit generational transition decision, compatibility policy and qualification gate.

**Important:** `X = 0` is reserved for pre-stable engineering and foundation work. It must not be treated as a stable product release generation.

`X >= 1` identifies the release generation; it does not automatically prove that an individual artifact has passed its stable-release gate.

### 35.2 Y — Major release

The second component identifies a major release within the current X generation.

Increment Y when a release introduces a deliberately major compatibility or architecture change within that generation.

Examples of changes potentially requiring a Y increment:

- Breaking public standard-library APIs.
- Incompatible compiler interfaces.
- Incompatible runtime interfaces.
- Major ABI changes.
- A deliberate incompatible feature revision.
- Other explicitly approved major release changes.

A major release must include migration guidance and a compatibility-impact assessment.

It must not be used as an arbitrary milestone counter.

### 35.3 Z — Minor release

The third component identifies compatible feature releases.

Increment Z when adding compatible functionality or a meaningful integrated capability without breaking the established compatibility contract.

Examples:

- Additive standard-library APIs.
- New compatible compiler features.
- Additional supported targets.
- New optional tooling.
- Compatible runtime capabilities.
- Compatible package-format extensions.

A minor release must not silently change existing normative behaviour.

### 35.4 W — Patch

The fourth component identifies patches.

Increment W for compatible maintenance changes such as:

- Correctness fixes.
- Security fixes.
- Compiler defect corrections.
- Runtime defect corrections.
- Documentation corrections.
- Build-system corrections.
- Performance improvements that preserve semantics.
- Reliability improvements.

A patch must preserve the relevant compatibility contract.

If a reported bug fix changes previously specified observable behaviour, its specification and compatibility implications must be explicitly assessed. It must not be disguised as an ordinary patch.

### 35.5 Version examples

| Version | Meaning |
|---|---|
| `1.0.0.0` | First stable release generation, initial qualified baseline |
| `1.0.0.1` | First patch to that release |
| `1.0.0.2` | Second patch |
| `1.0.1.0` | First compatible minor release |
| `1.0.2.0` | Second compatible minor release |
| `1.1.0.0` | First major release within generation 1 |
| `1.1.0.1` | Patch to that major release |
| `2.0.0.0` | Second stable release generation |
| `2.0.1.0` | Compatible minor release within generation 2 |
| `3.0.0.0` | Third stable release generation |

These are examples of the numbering policy, not claims that these releases have been published or qualified.

## 36. Version Channels and Stability Status

Numerical version and release maturity must be separate.

Maintain explicit channels:

1. Development.
2. Alpha.
3. Beta.
4. Release candidate.
5. Stable.
6. Maintenance.
7. End of life.

Example pre-release identifiers:

```text
1.0.0.0-alpha.1
1.0.0.0-beta.1
1.0.0.0-rc.1
```

A pre-release must never be labelled stable.

A version becomes stable only after its release gate passes and the authorized release record declares it stable.

Do not infer stability from a numeric value alone.

## 37. Versioning Is Separate Across Different Project Artifacts

The product release version must not be confused with the following.

### 37.1 Language edition and specification version

The language edition is versioned independently.

For example:

- Edition number.
- Candidate number.
- Ratification status.
- Normative tree digest.
- Approved errata.

The Edition 1 manifest currently identifies:

`1.0.0-candidate.1`

This is a specification status, not the project's four-component product release version.

### 37.2 Engineering milestone IDs

Engineering milestones and qualification gates are not product releases.

Use a clearly namespaced identity, for example:

```text
OMNI-IMP-S1-PH00-M013
OMNI-GATE-STAGE1-EXIT
OMNI-GATE-STAGE2-EXIT
OMNI-GATE-STAGE3-EXIT
```

The existing `OMNI-IMP-0.0.0.13` and other four-component engineering references must be preserved historically until a governed migration is approved.

They must not be confused with a publicly released product version.

The current implementation plan uses `R.M.m.P` as a project-generation and milestone scheme. Its relationship to the new product-release versioning policy must be resolved through a dedicated versioning ADR before changing any existing normative allocation rule or tag.

### 37.3 Rust crate/package versions

Individual Rust workspace packages must continue to use the version format and constraints required by Cargo.

For example:

```text
0.2.7
```

This is a crate version, not the four-component Omni product version.

For internal `publish = false` compiler crates:

- Decide whether the workspace should use a centrally inherited version.
- Prefer a common workspace version when the crates are intentionally developed in lockstep.
- Allow separate versions only when independent versioning is justified.
- Do not imply public API stability merely from a Cargo version.
- Keep Cargo.lock synchronized through deliberate reviewed changes.

The migration to a common workspace version, if selected, must be tested and performed deliberately.

### 37.4 ABI version

Define an independent ABI compatibility identifier.

ABI evolution must explicitly define:

- Backward compatibility.
- Forward compatibility where supported.
- Data layout stability.
- Calling conventions.
- Symbol rules.
- Runtime interaction.
- FFI compatibility.
- Migration.

Product version changes must not automatically imply ABI changes.

### 37.5 Runtime and standard-library versions

Keep runtime and standard-library compatibility explicit.

They may inherit the product release version when they are released as one inseparable package.

If independently versioned, define their supported compiler and ABI compatibility ranges.

### 37.6 Toolchain and build identity

A compiler binary must carry a traceable identity including:

- Product version.
- Channel.
- Compiler build identity.
- Source commit.
- Dirty-state indicator.
- Specification digest.
- Rust toolchain or bootstrap identity.
- Target.
- Backend.
- Build profile.
- Relevant configuration.

A Git commit SHA or build hash is not a substitute for a release version.

### 37.7 Document revision

The master vision and engineering documents may use an independent document revision and update date.

A document revision is not a language release.

## 38. Version Increment Rules

Use the following decision table:

| Change type | Expected version action |
|---|---|
| New stable generational platform | Increment X |
| Major breaking release within a generation | Increment Y |
| Compatible functionality | Increment Z |
| Compatible bug or security fix | Increment W |
| Documentation-only change | Usually no product version bump |
| Internal refactoring | Usually no product version bump |
| Research proposal | No product version bump |
| Compiler experiment | No product version bump unless released |
| Specification candidate amendment | Update specification identity, not automatically product version |
| New target support | Usually Z if compatible; otherwise classify explicitly |
| Security release with compatible behaviour | W |
| ABI-breaking release | Y or X, according to the declared compatibility boundary |
| Normative language edition change | Must be explicitly classified through the edition and release process |

These are release-classification rules, not automatic authorization to publish.

## 39. Version Immutability

Once a release has been published:

- Never reuse its version.
- Never replace its release artifacts silently.
- Never move an existing release tag.
- Never overwrite signed provenance.
- Never silently change its specification identity.
- Never issue different binaries under the same release identity.

A corrected release requires a new version and an explicit changelog entry.

## 40. Version Compatibility Policy

Maintain a compatibility matrix for:

- Product version.
- Language edition.
- Compiler.
- Standard library.
- Runtime.
- ABI.
- Package format.
- Target.
- Toolchain.

For each compatibility relationship, declare:

- Supported.
- Supported with limitations.
- Deprecated.
- Unsupported.
- Unknown.

Unknown compatibility must not be silently treated as supported.

## 41. Support and Maintenance Policy

Every stable release generation must have a declared maintenance policy.

It should identify:

- Supported stable releases.
- Security maintenance status.
- Bug-fix eligibility.
- Backport rules.
- Deprecation policy.
- Migration requirements.
- End-of-life conditions.

Do not promise an indefinite support period without the resources to provide it.

A previous stable generation may receive security fixes even after a newer generation is available when formally supported.

---

# PART XV — CHANGELOG STANDARD

## 42. Canonical Changelog

`CHANGELOG.md` remains the canonical human-readable product and project release history.

It should follow a predictable structure.

The top-level section is:

```markdown
# Changelog

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

### Performance

### Tooling

### Build and CI

### Documentation

### Research and Architecture Decisions

### Compatibility and Migration

### Known Limitations
```

Only applicable categories need entries.

Empty categories may be omitted from a release section.

## 43. Unreleased Section

All accepted but unreleased changes should be recorded under one `Unreleased` section.

Every new meaningful change must be classified.

Never create multiple competing Unreleased sections.

Do not present pending work as completed.

Do not put failed or unverified changes in a release-qualified category.

## 44. Changelog Categories

### Added

New user-visible capabilities.

### Changed

Changes to existing behaviour or interfaces.

### Deprecated

Functionality that remains available but has a declared future removal or replacement path.

### Removed

Removed functionality, including breaking changes.

### Fixed

Bug fixes with an explanation of the affected behaviour.

### Security

Security defects, advisories, impact, mitigation and remediation.

### Performance

Measured performance improvements with relevant benchmark evidence.

### Tooling

CLI, formatter, debugger, LSP, test runner, profiler and developer-tool changes.

### Build and CI

Build configuration, CI checks, dependency and release-process changes.

### Documentation

User and developer documentation changes.

### Research and Architecture Decisions

Important accepted decisions that affect future development, without presenting proposals as implemented functionality.

### Compatibility and Migration

API, ABI, edition, runtime and package compatibility changes.

### Known Limitations

Important limitations that developers should be aware of.

## 45. Changelog Entry Requirements

A meaningful entry should identify:

- What changed.
- Why it changed.
- Which subsystem is affected.
- Whether it changes observable behaviour.
- Relevant issue or requirement.
- Relevant specification rule.
- Relevant ADR, if applicable.
- Tests or qualification evidence.
- Compatibility implications.
- Known limitations.

Where possible, link repository issues, PRs, commits, normative rules and verification records.

Avoid vague entries such as:

- Improved compiler.
- Fixed bugs.
- Better performance.
- Completed testing.

Explain the actual work and the evidence.

## 46. Changelog Release Sections

A released section must use this format:

```markdown
## [X.Y.Z.W] — YYYY-MM-DD
```

Example only:

```markdown
## [1.0.0.0] — YYYY-MM-DD

### Added
- First stable qualified language baseline for the declared release scope.

### Changed
- ...

### Fixed
- ...

### Security
- ...

### Compatibility and Migration
- ...
```

The actual date and contents must be grounded in the real release.

## 47. Changelog Accuracy Rules

- Do not claim a test passed unless it was actually run.
- Do not describe a feature as complete when it is experimental.
- Do not call an implementation production-ready without qualification.
- Do not imply a GitHub release exists merely because a changelog heading exists.
- Do not repeat a release entry in multiple versions.
- Preserve historical entries.
- Corrections to historical changelog entries must be traceable.
- Distinguish implementation progress from published release contents.

## 48. Changelog Cleanup and Validation

The retrieved current `CHANGELOG.md` contains historical ordering and duplication issues, including repeated `0.2.6` entries and out-of-order patch sections.

Perform a one-time historical normalization when authorized:

1. Preserve every valid historical change.
2. Identify duplicated records.
3. Verify the original commit and date.
4. Correct ordering.
5. Separate development snapshots from actual releases.
6. Retain the historical meaning.
7. Record corrections.
8. Run the changelog validator.

Never delete historical engineering information merely to make the document look cleaner.

---

# PART XVI — VERSION AND RELEASE AUTOMATION

## 49. Single Source of Truth

Introduce a machine-readable project release manifest.

A proposed location is:

```text
release/omni-release.json
```

Its schema should include:

- Schema version.
- Product name.
- Product release version.
- Release channel.
- Release generation.
- Major, minor and patch coordinates.
- Language edition.
- Normative specification manifest.
- Specification tree digest.
- Implementation stage.
- Release qualification profile.
- Supported targets.
- ABI compatibility identity.
- Runtime and standard-library compatibility.
- Source commit.
- Toolchain identity.
- Build provenance.
- Release date.
- Required qualification results.
- Known limitations.
- Previous release identity.

The project must not maintain contradictory manually edited copies of the canonical product release version.

Human-readable documentation, tags and release notes should be checked against this manifest.

The normative language specification manifest remains a separate authority.

## 50. Release Version Validator

Create a dedicated validation tool or script.

Suggested command:

```text
omni-release validate
```

It should verify:

1. Version format.
2. Correct component values.
3. Valid channel.
4. Version monotonicity.
5. No previously used version.
6. Changelog consistency.
7. Tag consistency.
8. Release manifest consistency.
9. Specification identity consistency.
10. Required qualification records.
11. Cargo metadata consistency where applicable.
12. ABI compatibility declarations.
13. Provenance completeness.
14. Artifact identity.
15. Release-date and channel consistency.

The tool must fail closed for malformed or contradictory release metadata.

## 51. Changelog Validator

Suggested command:

```text
omni-release changelog-check
```

It should detect:

- Missing Unreleased heading.
- Duplicate release headings.
- Invalid versions.
- Duplicate entries.
- Incorrect release ordering.
- Missing dates.
- Malformed categories.
- Empty release sections.
- Invalid issue/ADR/rule references.
- Unresolved changelog conflicts.
- Inconsistent version metadata.

## 52. Automated Version Recommendations

Use change classification to propose the appropriate version increment.

Potential commit prefixes include:

```text
feat:
fix:
security:
perf:
refactor:
docs:
test:
build:
ci:
spec:
research:
release:
```

Breaking changes should be explicitly declared, for example through reviewed breaking-change metadata.

Examples:

```text
feat: add compatible standard-library capability
fix: correct MIR lowering for loop conditions
security: remediate vulnerable dependency
perf: reduce allocation overhead in parser
docs: clarify ownership semantics
spec: propose a new grammar amendment
```

Commit prefixes alone must not determine the final release version.

A human-reviewed compatibility classification remains mandatory.

## 53. Release Automation Pipeline

A release candidate should pass:

1. Clean repository check.
2. Correct source revision.
3. Formatting.
4. Clippy and denied-warning policy.
5. Unit tests.
6. Integration tests.
7. Native end-to-end execution.
8. Conformance.
9. Property-based tests.
10. Fuzzing requirements.
11. Security audit.
12. Cross-target tests.
13. Reference-machine comparison.
14. Differential tests where applicable.
15. Reproducible artifact checks.
16. Version validator.
17. Changelog validator.
18. Provenance validation.
19. Documentation checks.
20. Final qualification review.

A failed security audit or semantic qualification check blocks a stable release unless a documented policy-approved disposition establishes that the finding is not applicable or is otherwise safely resolved.

## 54. Git Tags and Releases

Use a canonical stable tag:

```text
omni-vX.Y.Z.W
```

For example:

```text
omni-v1.0.0.0
```

Pre-release tags should preserve the channel:

```text
omni-v1.0.0.0-alpha.1
omni-v1.0.0.0-beta.1
omni-v1.0.0.0-rc.1
```

Release requirements:

- Tag points to the exact qualified commit.
- Tag is annotated.
- Tag is signed where signing infrastructure is available.
- Release manifest identifies the same commit.
- Artifacts have cryptographic digests.
- Provenance is preserved.
- Release notes match `CHANGELOG.md`.
- Published artifacts are immutable.
- No release is created from dirty or unqualified source.

Do not tag every engineering milestone as a product release.

## 55. Release Security and Provenance

Each stable release should preserve:

- Exact source commit.
- Normative specification digest.
- Compiler identity.
- Build toolchain.
- Dependency inventory.
- Target.
- Backend.
- Build configuration.
- Artifact hashes.
- Qualification reports.
- SBOM where applicable.
- Signature and provenance records where supported.

The objective is to make every published artifact traceable to its source, specification, toolchain and verification evidence.

---

# PART XVII — PROJECT TRACKING AND RECORDKEEPING

## 56. Project Records

Maintain clearly separated records for different types of information.

| Record | Purpose |
|---|---|
| Master vision | Approved goal and overall project direction |
| Requirements registry | Individual requirements and traceability |
| Architecture decision records | Approved design decisions and alternatives |
| Normative specification | Actual language semantics |
| Implementation plan | Engineering work breakdown |
| Implementation baseline | Verified current source-code state |
| Changelog | Historical changes and releases |
| Release manifest | Canonical product release identity |
| Qualification records | Milestone and release evidence |
| Test and conformance records | Executable verification evidence |
| Dependency inventory | Version, security, license and compatibility data |
| Security register | Advisories, threats, mitigations and risks |
| Known limitations register | Unsupported or incomplete functionality |
| Migration register | Compatibility and deprecation changes |
| Research register | Papers, findings, prototypes and alternatives |
| Decision log | Pending and completed project decisions |

Avoid duplicated sources of truth.

If the same fact appears in multiple places, establish which record owns it and how other documents are derived or validated.

## 57. Requirements Traceability

Each substantial requirement should have:

- Stable ID.
- Original source.
- Exact statement.
- Rationale.
- Priority.
- Acceptance criteria.
- Dependencies.
- Research evidence.
- Design alternatives.
- Approved decision.
- Normative rule.
- Implementation mapping.
- Tests.
- Proof obligations.
- Qualification status.
- Known limitations.
- Revision history.

## 58. Architecture Decision Records

Each significant architectural decision should document:

- Decision ID.
- Title.
- Date.
- Status.
- Context.
- Problem.
- Alternatives.
- Evidence.
- Trade-offs.
- Selected approach.
- Consequences.
- Affected components.
- Compatibility impact.
- Migration requirements.
- Revisit conditions.

Use states such as:

- Proposed.
- Under review.
- Accepted.
- Rejected.
- Superseded.
- Deprecated.

Do not silently promote proposals into approved design.

## 59. Implementation Status Tracking

Track every subsystem using explicit states:

- Not started.
- Research required.
- Design in progress.
- Prototype.
- Partially implemented.
- Implemented.
- Tests incomplete.
- Tested.
- Independently reviewed.
- Qualified.
- Released.
- Deprecated.

Do not reduce implementation status to an unqualified green checkmark.

## 60. Blockers and Risks

Maintain:

- Blocker ID.
- Description.
- Root cause.
- Severity.
- Affected milestones.
- Dependencies.
- Workaround.
- Permanent resolution.
- Owner.
- Evidence.
- Status.

Distinguish genuine blockers from improvements that can continue in parallel.

## 61. Project Metrics

Track useful engineering indicators:

- Requirements implementation coverage.
- Normative-rule coverage.
- Witness coverage.
- Conformance pass/fail.
- Regression count.
- Fuzz campaign coverage.
- Known crashers.
- Outstanding security findings.
- Build reliability.
- Compilation time.
- Test duration.
- Runtime performance.
- Memory usage.
- Cross-target coverage.
- Release reproducibility.
- Documentation completeness.
- Open architectural decisions.

Metrics must not become artificial targets that encourage weakening tests or adding misleading coverage.

---

# PART XVIII — GIT, CI AND REPOSITORY GOVERNANCE

## 62. Canonical Repository

The canonical implementation repository is:

`https://github.com/shreyashjagtap157/Omni---New`

The canonical branch remains:

`main`

The separate local `Omni` workspace must be reconciled with `Omni---New` before history migration or source transfer.

Do not assume the two directories are identical.

## 63. Git Rules

- Preserve commit history.
- Never force-push.
- Never rewrite published history.
- Inspect status and commit identity before modifications.
- Do not run destructive reset or clean operations without explicit review.
- Maintain the established main-only engineering workflow.
- Review automated dependency branches independently.
- Do not merge unqualified changes.
- Do not create release tags prematurely.
- Keep generated artifacts synchronized.
- Preserve auditable commit messages.

## 64. CI Architecture

Separate the gates into logical categories:

**Fast correctness checks**
- Formatting.
- Clippy.
- Unit tests.
- Static checks.
- Repository consistency.

**Compiler integration**
- Frontend-to-MIR.
- Verifier.
- Abstract-machine execution.
- Native execution.
- Runtime tests.

**Language conformance**
- Normative cases.
- Invalid-program rejection.
- Diagnostic checks.
- Feature interactions.

**Robustness**
- Fuzzing.
- Property testing.
- Adversarial inputs.
- Resource exhaustion.

**Security**
- Dependency audit.
- CodeQL.
- Unsafe-boundary review.
- Supply-chain checks.

**Release**
- Cross-target qualification.
- Reproducibility.
- Version validation.
- Changelog validation.
- Provenance.
- Signed artifacts where supported.

Stable and nightly-specific toolchain requirements must be declared explicitly.

## 65. CI Result Interpretation

Distinguish:

- Workflow success.
- Job success.
- Test-suite success.
- Security audit success.
- Conformance success.
- Qualification success.
- Product release readiness.

A workflow passing does not automatically establish all of these.

---

# PART XIX — CURRENT REPOSITORY BASELINE AND KNOWN DISCREPANCIES

## 66. Repository Snapshot Reviewed

The GitHub repository review observed:

- Repository: `shreyashjagtap157/Omni---New`.
- Default branch: `main`.
- Latest surfaced commit: `692f47973e4760d94b8329585baee8caf9b122fa`.
- Root workspace: 25 declared members.
- Rust workspace lint: `unsafe_code = "forbid"`.
- Clippy workspace lint: `all = "deny"`.
- Experimental Omni lexer and parser present.
- Master project document present.
- `IMPLEMENTATION_PLAN.md` present.
- `docs/implementation-baseline.md` present.
- `CHANGELOG.md` present.

These are repository observations, not claims that every subsystem is complete.

## 67. Edition 1 and Grammar

The live manifest identifies Edition 1 as:

`1.0.0-candidate.1`

The current normative EBNF remains the grammar artifact identified by the candidate baseline.

Candidate 2 remains a pending amendment and is not automatically active.

The repository documents a mismatch between the normative EBNF and later vibe-first syntax requirements.

Any grammar or semantic change must be reconciled through the controlled specification process.

The master vision must not activate Candidate 2.

## 68. Foundation Gate Reconciliation

The repository contains:

- `spec/release/foundation-gate.json`, recording `OMNI-IMP-0.0.0.13` as qualified.
- `docs/foundation-gates.md`, which documents the gate as pre-sign-off and identifies unresolved evidence limitations.
- Documentation stating zero emitted E-code diagnostics and zero rule-witness records.
- Different historical/current-looking specification digest values in associated documentation.

These records require reconciliation.

The authoritative digest must be computed by the canonicalization tool and matched against the manifest and release evidence.

Do not infer foundation qualification merely from a committed JSON status field.

Do not assume that a foundation gate qualifies the complete compiler or language.

## 69. Implementation Baseline

`docs/implementation-baseline.md` identifies outstanding technical work, including:

- Complete reference provenance.
- Complete ownership/lifetime integration.
- Effects and capability enforcement.
- MIR initialization, drops and unsafe-assumption semantics.
- Complete abstract-machine memory and execution model.
- Runtime and platform support.
- Real differential qualification.
- Package and tooling development.
- Self-hosting.

The root implementation plan contains an older October 12 snapshot and must be refreshed against the live repository.

## 70. Current Workflow Evidence

At the reviewed commit `692f479`, the November 7, 2026 GitHub Actions results included:

- Rust workflow — passed.
- Omni CI Pipeline — passed.
- Clippy analysis — passed.
- CodeQL Advanced — passed.
- Security Audit — failed.

The Security Audit failure was identified as `RUSTSEC-2026-0344`, involving `ruzstd 0.7.3`.

The advisory identifies version `0.9.1` or later as patched.

Repository issue:

https://github.com/shreyashjagtap157/Omni---New/issues/46

This must be remediated or explicitly dispositioned and verified.

## 71. Dependency Update Work

The repository also surfaced open automated dependency pull requests involving Cranelift and other components.

Treat these as compatibility-engineering tasks.

Do not automatically merge backend upgrades.

## 72. Changelog Issues

The current `CHANGELOG.md` contains duplicated and out-of-order historical version entries.

It requires an evidence-preserving cleanup and adoption of the canonical structure in this document.

## 73. Historical Test Claims

The supplied engineering report recorded 713+ workspace tests passing and successful audit, conformance, formatting and native execution checks.

Those claims should be retained as historical session evidence, but a fresh validation must be performed against the intended current commit.

The GitHub Rust workflow passing is current CI evidence for that commit; it is not a substitute for all release qualification obligations.

---

# PART XX — UNRESOLVED DECISION REGISTER

## 74. Decisions That Require Explicit Research and Approval

Maintain these as unresolved until sufficient evidence and approval exist.

| Decision | Alternatives or research area | Why it matters |
|---|---|---|
| Memory default | Managed default, affine core, hybrid profile | Safety and ergonomics |
| Reference copyability | Shared-reference copying, all references affine, explicit trait rules | Ownership correctness |
| Concurrency model | Unified surface model, explicit async, profile-specific agents | Usability and runtime |
| Future surface syntax | Candidate 1, Candidate 2, future alternatives | Vibe-first programming |
| Type-system scope | Core static typing, advanced refinements, higher-ranked features | Soundness and complexity |
| Compile-time execution | Const evaluation, macros, plugins, reflection | Power and security |
| Effects | Closed core, extensible effects, handlers | Modularity and analysis |
| Capability model | Static and dynamic authority boundaries | Security |
| Execution profiles | Hosted, freestanding, embedded, real-time, accelerator | Portability |
| Backend strategy | Cranelift, LLVM, MLIR, custom | Performance and maintainability |
| ABI evolution | Stability and compatibility policy | Interoperability |
| Production release scope | Required features, targets and profiles | Practical release |
| Formal assurance | Proof priorities and independent evidence | Trust |
| Self-hosting | Compiler and toolchain independence milestones | Bootstrap integrity |
| Version-policy migration | Existing four-part engineering labels versus product versions | Release correctness |
| Rust crate versioning | Shared workspace version versus independent versions | Package consistency |
| Support lifecycle | Supported stable generations and LTS policy | Long-term maintenance |

Every decision must have a status, alternatives, evidence, dependencies, impact and approval requirements.

---

# PART XXI — ROADMAP AND RELEASE QUALIFICATION

## 75. Research and Engineering Phases

### Phase 0 — Baseline

- Reconcile repositories.
- Verify live `main`.
- Refresh implementation status.
- Inspect CI.
- Resolve specification authority conflicts.
- Recover original requirements.
- Establish traceability.

### Phase 1 — Core semantics

- Research syntax.
- Formalize types.
- Establish evaluation semantics.
- Resolve core memory and ownership requirements.
- Define executable reference models.

### Phase 2 — Rust compiler core

- Lexer.
- Parser.
- Names.
- Types.
- Ownership.
- HIR.
- MIR.
- Verifier.

### Phase 3 — Native execution

- Reference machine.
- Low-level lowering.
- Code generation.
- Linking.
- Runtime.

### Phase 4 — Full mandatory language functionality

- Types and generics.
- Ownership.
- Effects.
- Capabilities.
- Errors.
- Concurrency.
- Required language interactions.

### Phase 5 — Runtime and standard library

- Required runtime services.
- Core facilities.
- Allocation.
- I/O.
- Platform abstraction.
- Required concurrency.

### Phase 6 — Toolchain

- CLI.
- Build.
- Formatter.
- Linter.
- Testing.
- Documentation.
- Diagnostics.
- Debugging and language-server foundation.

### Phase 7 — Production qualification

- Representative applications.
- Full declared-scope conformance.
- Security.
- Performance.
- Reproducibility.
- Target support.
- Developer experience.

### Phase 8 — Self-hosting migration

- Omni-written compiler components.
- Differential validation.
- Gradual replacement.
- Compiler self-compilation.

### Phase 9 — Bootstrap trust

- Reproducibility.
- DDC.
- Independent verification.
- Toolchain trust.
- Recovery procedures.

### Phase 10 onwards — Continuous evolution

No final completion date or fixed feature ceiling.

## 76. Production Release Gate

A stable product release requires:

- Declared scope.
- Approved specification identity.
- Complete mandatory features.
- Correct compiler pipeline.
- Validated native output.
- Required runtime.
- Required standard library.
- Conformance.
- Regression testing.
- Security assessment.
- Reproducibility.
- Documentation.
- Known limitations.
- Version validation.
- Changelog validation.
- Provenance.
- Required independent review.

A release cannot be called stable solely because its version is `1.0.0.0` or higher.

---

# PART XXII — ENGINEERING RULES

## 77. Non-Negotiable Rules

Never:

1. Replace the approved project goal with a narrower goal.
2. Treat additive engineering requirements as limits on the ultimate ambition.
3. Invent language semantics.
4. Activate pending grammar amendments implicitly.
5. Confuse specification maturity with compiler maturity.
6. Confuse compiler demonstrations with production implementation.
7. Confuse tests with proof.
8. Confuse self-hosting with correctness.
9. Confuse product version with milestone ID.
10. Confuse product version with Cargo crate version.
11. Confuse specification edition with product release.
12. Publish unqualified releases.
13. Reuse or overwrite release versions.
14. Ignore security advisories without an explicit disposition.
15. Weaken tests to make CI pass.
16. Hide compiler defects behind environmental workarounds.
17. Allow multiple semantic authorities.
18. Abandon the Rust reference implementation prematurely.
19. Overdesign the first production release.
20. Neglect developer ergonomics.
21. Claim unsupported safety or certification guarantees.
22. Leave the changelog and implementation baseline stale.
23. Treat a commit or a passing workflow as proof of all release gates.
24. Stop researching after a successful release or self-hosting milestone.

---

# PART XXIII — MASTER DEVELOPMENT MANDATE

## 78. Complete Project Instruction

Develop Omni according to the user's approved goal.

Approach C is the primary implementation strategy.

Use Rust for the initial production compiler and reference implementation.

Continue self-hosting experiments in parallel.

Complete the declared mandatory language functionality and a practical production platform before the official self-hosting transition.

Preserve the original broad ambition and vibe-first programmer experience.

Research and evaluate new theories, algorithms, technologies, hardware and engineering techniques continuously.

Do not allow implementation convenience to redefine the language.

Do not allow the first production release to become blocked by every future research objective.

Use the four-component project release version `X.Y.Z.W`, with separate identities for engineering gates, language editions, Rust crates, ABIs, runtimes, artifacts and documentation.

Maintain a rigorously validated changelog, release manifest, requirements registry, architecture decision record, implementation baseline, security register, known-limitations register and qualification evidence.

Preserve the Git repository's history and main-branch workflow.

Maintain truthful, reproducible engineering reports.

The Rust-first milestone must make Omni a genuinely usable programming language.

The self-hosting milestone must establish a trustworthy compiler bootstrap.

Neither milestone ends the project.

**The approved goal remains unchanged. All technical, research, versioning, release and governance requirements are additive mechanisms for achieving it. Omni must continue to evolve without an artificial final endpoint.**