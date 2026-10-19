# Omni Complete Specification — Generated Reference with Conversation Integration

**Status**: Generated reference synthesis — incorporates design analysis, review feedback, and idealized language design discussion from October 2026 session.

**Source**: Synthesized from Omni project repository (C:\Users\siddh\Downloads\ABC\Omni/), conversation research on state-of-the-art/space/enterprise/military language design, and design review of Aurora language proposal.

**Version**: 1.0.0.0 (generated)

---

## Overview

Omni is an idealized programming language designed for state-of-the-art, space-grade, enterprise-grade, and military-grade software development. This specification defines the language's syntax, semantics, type system, verification model, build pipeline, conformance framework, and assurance profiles.

The language resolves the tension between expressive power and verifiability through staged verification, content-addressed computation, capability-based security, and profile-enforced feature restrictions.

**Philosophy**: Formal verification and reproducible builds are architectural concerns, not afterthoughts. Every language feature must have clear normative semantics, a sound implementation strategy, enforceable restrictions, and evidence appropriate to the assurance profile.

**Distinction preserved**: Design requirement, specification clause, implementation, passing test suite, and proof of correctness are five different things — this specification defines the normative *clauses*; implementation status must be verified separately.

---

## 1. Language Governance

### 1.1 Versioning Scheme

- **Four-component form**: `R.M.m.P` (Release.Major.Minor.Patch)
- **Example**: `1.0.0.0` = Minimal verified core (Stage-0); `2.0.0.0` = Complete production language; `3.0.0.0` = Certified self-hosted independent compiler
- **Governance**: `BOOT-0002` through `BOOT-0009` define transition rules, DDC requirements, and trusted compiler chain

### 1.2 Architecture Freeze

- Architecture phase is closed; no silent feature additions or convenience changes
- amendments require: discovered contradiction, unclassified semantic case, soundness defect, interoperability violation, implementation impossibility due to normative inconsistency, or required clarification uniquely implied but not mechanically stated
- Implementation team must not use difficulty alone as justification for weakening safety guarantees

### 1.3 Bootstrap Language

- Initial implementation bootstrapped from Rust (pinned toolchain)
- Intended evolution: Rust → Rust Stage-0 compiler → Rust compiler capable of compiling Stage-0/partial Omni → Omni implementation of compiler → Omni compiles Omni → DDC / independent verification → Rust reduced to validation/bootstrap-recovery role

---

## 2. Specification Authority and Digest Domain

### 2.1 Normative vs Non-Normative

- **Normative specification**: Modules listed as required in the signed release manifest (ROOT-0001)
- **Implementation deviations**: Must be documented as errata with rule IDs; cannot invent interpretation
- **Authority precedence**: Signed release manifest > implementation claims > informal convention

### 2.2 Specification Digest

- `spec_tree_sha256` = digest of normative specification content only
- **Excluded from digest**: Generated implementation manifests, release evidence under `spec/manifest/`, `spec/release/`
- **Canonicalization procedure**: 
  1. Walk included directories in lexicographic order of slash-joined relative paths
  2. Normalize file bytes: CRLF/CR → LF
  3. Hash each file with SHA-256
  4. Hash concatenation of `path + LF + hex + LF` lines
- **Computed digest** (current): `7e63431f3c624f9818454f076c38f5cde3d517ace9e919926d6188bc177be769` (supersedes previous values by REL-0007 machine-block formatting in `candidate2-erratum.md`)
- **Earlier digest**: `07762061…` value by gate-closure content
- **omni-canon** not yet started — digest integrity must be established from actual code/specification before release claims can be trusted

### 2.3 Conflict Resolution

- **EBNF vs VIBE-GRAM conflict**: The normative EBNF predates Candidate-2 amendment and contains no pipeline-operator, newline-termination, projection-shorthand, or command-call productions, while VIBE-GRAM rules normatively require them. Grammar modernization belongs to the grammar milestone; the tree binds the artifact as-is. Any resolution must document precedence decision and update affected rule IDs.

---

## 3. Conformance and Certification

### 3.1 Conformance Claim Structure

Every conformance claim follows `CONF-0001` through `CONF-0011`:

| ID | Description |
|---|---|
| `CONF-0001` | Each conformance case has stable case ID, rule IDs, suite version, source/artifact inputs, expected classification/result, target/profile applicability, required limits, oracle/model version, and allowed implementation-defined alternatives |
| `CONF-0002` | Test kinds: parse-pass/fail, static-pass/fail, run-value, run-output, run-fault, memory-litmus, ABI-cross, wire-golden, build-resolution, security-policy, differential, metamorphic, performance-contract |
| `CONF-0003` | Coverage includes positive, negative, boundary, interaction, differential, and metamorphic cases for every ratified core rule; high-risk rules additionally require an independent model or implementation |
| `CONF-0004` | A test is corrected only through a versioned test erratum linked to the rule. Prior suite artifacts remain immutable; corrected conformance claims identify the overlay |
| `CONF-0005` | Tests do not create semantics. When a test conflicts with normative rules, the test is invalid until corrected |
| `CONF-0006` | Strict mode disables extension syntax/semantics and rejects artifacts whose unrecognized extension could affect behavior. Namespaced extensions may be tested only in separate claims |
| `CONF-0007` | An implementation claim names edition, modules, profiles, targets, limits, external ABI/data versions, implementation-defined choices, deviations, and test-result digest |
| `CONF-0008` | Parser-only, core-subset, profile, platform, distribution, and complete-release claims are distinct; partial implementations cannot claim complete conformity |
| `CONF-0009` | Certification may be self-attested, independently audited, or authority-certified. The claim record states assurance level, auditor, evidence, waivers, expiration, and revocation status |
| `CONF-0010` | Waivers cannot excuse a soundness violation or false accepted behavior; they may document unavailable optional hardware/profile tests and narrow the claim |
| `CONF-0011` | A certification is revoked when evidence is falsified, a blocker soundness/security defect invalidates the claim, or required artifacts become untrusted; revocation is signed and published |

### 3.2 Assurance Profiles

Instead of treating four grades as a feature checklist, Omni defines a **common language foundation** with **separate assurance profiles**. Each profile declares:

| Contract | What Must Be Specified |
|---|---|
| Language features | Which features are permitted and which are excluded |
| Memory and allocation | Permitted allocation strategies, ownership guarantees, resource bounds |
| Concurrency | Scheduling, synchronization, cancellation, failure guarantees |
| Runtime and platform | Required services, trusted components, target assumptions, unsupported behavior |
| Verification | Required proofs, static analyses, tests, differential checks, coverage of assumptions |
| Release evidence | Artifact provenance, dependency integrity, reproducibility, independent review, waivers |

Profiles inherit from the common semantic foundation; profile-specific deviations must be explicit and testable.

**Profiles defined**:
- `safe`: No `unsafe`, no `foreign` effect, no raw pointers — enforced via type system and checker
- `distributed`: Designed for multi-core/multi-processor with message-passing-only state sharing
- `real-time`: Bounded execution, priority inheritance, bounded preemption
- `high-assurance`: Full formal verification discharge, DDC mandatory, no undefined behavior
- `general-purpose`: Productive application development with GC, dynamic dispatch, runtime reflection

Any profile may be tailored per `ECSS-S-ST-00` tailoring rules, but deviations from the profile's core contract must be explicitly documented and cannot excuse soundness violations.

### 3.3 Certification Revocation

- Evidence falsification → immediate revocation
- Blocker soundness/security defect → immediate revocation
- Required artifacts become untrusted → immediate revocation
- Revocation signed and published with digested evidence
- Revoked artifacts cannot be selected or executed absent audited emergency waiver

---

## 4. Build Pipeline (Content-Addressed)

### 4.1 Build Action Semantics

- `BUILD-0001`: A build action is a pure declared function from content-addressed inputs and fixed parameters to content-addressed outputs plus structured diagnostics
- `BUILD-0002`: Inputs include source trees, dependency artifacts, tools, target/profile manifests, environment variables explicitly named, build capabilities, and resource limits
- `BUILD-0003`: Undeclared files, environment, clock, timezone, locale, randomness, network, process state, home directory, registry state, and filesystem enumeration **cannot be observed**
- `BUILD-0004`: Build network access is denied by default. A fetch action is separate, content-addressed, policy-authorized, and produces immutable inputs for later offline build actions
- `BUILD-0005`: Paths inside the sandbox are canonical virtual paths. Host paths do **not** enter object/debug/generated bytes except through normalized source remapping records
- `BUILD-0006`: Generated source identity includes generator artifact digest, command/schema version, exact inputs, semantic extension settings, and output path identity
- `BUILD-0007`: Incremental cache keys include every semantically relevant query input and compiler/spec data version. Missing an input is a **correctness defect**
- `BUILD-0008`: Remote cache/execution results accepted only with authenticated action digest, worker/toolchain attestation, output digests, and policy-compatible isolation
- `BUILD-0009`: Build action outputs written atomically and verified before publication. Partial output is **never** treated as cache hit
- `BUILD-0010`: Parallel action completion order cannot affect merged outputs; merge order is canonical by declared key
- `BUILD-0011`: Resource-limit failure is a structured build failure and **not** permission to emit a partial release artifact

### 4.2 Build Gates (from Implementation Plan)

**Stage 1.1: Foundation (0.0.0.1 → 0.0.0.4) — Immediate**
- Repository initialization with proper directory structure
- Rust toolchain pinning in `rust-toolchain.toml`
- Quality/CI boundary setup (ceilings, fuzz budgets)
- Rule registry schema with lifecycle states
- Specification manifest creation
- Canonicalization engine (`omni-canon`) — **critical path item**

**Stage 1.2: Lexer/Parser (0.0.1.x → 0.0.2.x) — High Priority**
Complete ERR3-0010 through ERR3-0018 and ERR3-0020 through ERR3-0027:
- Byte cursor and UTF-8 decoder with BOM handling at offset zero
- Source normalization and canonical line-ending handling
- Token kinds, trivia preservation, identifiers/keywords
- Unicode identifier data ingestion and confusable validation
- Numeric literal state machines with ERR3-0014 compliance
- Character/string literals and raw strings
- Operators and punctuation with maximal-munch
- Context-sensitive generic punctuation splitting
- Lexical diagnostics
- Deterministic EOF behavior
- Lossless CST with comment/whitespace preservation
- Grammar productions from normative data
- Deterministic recursive-descent declaration parsing
- Pratt-style expression parsing
- Pattern grammar and generic syntax

### 4.3 Provenance and Evidence

- `UPDATE-0001` through `UPDATE-0008` define metadata chain
- `UPDATE-0001`: Update metadata uses separately scoped root, timestamp, snapshot, targets, delegation, revocation, and advisory roles with threshold signatures and expiration
- `UPDATE-0002`: Root metadata versioned and updated only through chain validated from trusted root with rollback protection
- `UPDATE-0003`: Timestamp metadata prevents freeze and detects stale snapshots; snapshot metadata binds exact versions/hashes of delegated metadata and prevents mix-and-match
- `UPDATE-0004`: Target metadata binds package/artifact identity, version, length, digest, and custom compatibility fields, preventing wrong-package substitution
- `UPDATE-0005`: Clients enforce monotonically nondecreasing metadata versions, expiration using authorized clock or offline freshness policy, and maximum metadata/target sizes
- `UPDATE-0006`: Endless-data defenses require declared length before download, streaming limits, decompression limits, and hash verification
- `UPDATE-0007`: Revocation identifies exact content digests/keys/versions and effective time/severity; security-revoked artifact not selected/executed absent audited emergency waiver
- `UPDATE-0008`: Offline mirrors carry complete signed metadata chain and checkpoints; mirror transport cannot override signatures or freshness

### 4.4 Trusted Computing Base Inventory (BOOT-0008)

Lists exact source/binary digests, role, reason trusted, audit status, replacement strategy, and transitive dependencies for:
- Proof checker
- Parser generator
- Solver
- Assembler
- Linker
- Unicode data
- Crypto
- OS/firmware
- Signing tools

Emergency bootstrap recovery uses archived source, specs, data, emulators, and at least two independent seed paths stored in reproducible offline media.

---

## 5. Target Model

### 5.1 Canonical Target Identity

`TARGET-0001`: A canonical target identity is `arch-vendor-os-environment-abi[data-model]+feature-set`, with lowercase registered components and sorted feature names.

### 5.2 Target Features

- `TARGET-0002`: Aliases resolve through the signed target registry to exactly one canonical identity and cannot depend on the host machine
- `TARGET-0003`: Target manifests declare byte order, scalar widths/alignments, pointer widths per address space, maximum object size, stack alignment, atomic widths/lock-freedom, vector model, object format, ABI supplement, unwind/debug formats, and feature detection method
- `TARGET-0004`: Baseline feature sets are explicit. A distribution artifact cannot execute an instruction outside its baseline except within a correctly guarded multiversioned body
- `TARGET-0005`: Runtime feature detection is trusted only through the OS/firmware mechanism named by the target manifest; unprivileged self-reporting may further narrow but not widen authorized features
- `TARGET-0006`: Feature dispatch chooses a body whose required feature set is a subset of detected and deployment-policy-authorized features. Selection is cached with thread-safe initialization
- `TARGET-0007`: Target-specific semantics are limited to implementation-defined data layout, ABI/platform facilities, conditional instructions, and classified hardware faults. Core expression meaning remains unchanged
- `TARGET-0008`: Unsupported target features produce translation rejection or `TargetFeatureFault` before entering the body; illegal instruction traps are not an acceptable dispatch mechanism in safe hosted code
- `TARGET-0009`: Address spaces are registered with width, representation, accessibility, coherence, atomicity, provenance recovery, and conversion rules

### 5.3 Distribution Restrictions

- Distribution artifact cannot execute an instruction outside its baseline feature set
- Feature dispatch is cached; unauthorized feature combinations rejected at load time
- Multiversioned bodies allow guarded use of out-of-baseline instructions when correctly conditionally compiled

---

## 6. Formal Semantics and Verification

### 6.1 Aurora Core Logic (ACL) — Defined Meta-Theory

Omni's formal semantics are defined in the **Aurora Core Logic (ACL)**, a simply-typed lambda calculus extended with:

1. **Ownership/affine types** with linear usage tracking
2. **Effect rows** as part of type signatures: `fn(T) -> T / ε`
3. **Capability types** for security confinement: `Cap(T)`
4. **Resource indicators** for lifecycle management

**Key meta-theoretic properties**:
- **No undefined behavior**: Every code path has a defined observation (TERM-0005: erroneous program SHALL NOT produce conforming release artifact)
- **Effect row compatibility**: `Γ ⊢ e1 : T1 / ε1` ∥ `Γ ⊢ e2 : T2 / ε2` ⇒ `Γ ⊢ e1 ⊕ e2 : T1 ⊕ T2 / ε1 ∪ ε2`
- **Linear usage**: `Γ, x : Linear T ⊢ e : T'` with use-exactly-once discipline covering branches, loops, moves, scope exit, and failure paths
- **Effect vs capability distinction**: Every function declares its effects (effect row), but also has an implicit capability environment `K`. These are distinguished: effects track algebraic effects (IO, state, side-channels); capabilities track authority (file system access, network, etc.). The semantics clearly separates effect requirements from capability passing and authority.

**Soundness properties** (with assumptions explicitly stated):
- Progress: A well-typed program either is a value or can take a evaluation step
- Preservation: Well-typedness is preserved by evaluation
- Effect soundness: Declared effects precisely characterize observable side-channels
- Capability soundness: A value of type `Cap(T)` holds exactly the authorities granted by its capability token

### 6.2 Verification Model

#### 6.2.1 Stage-0 (Verified Subset)

Stage-0 is a strict syntactic and semantic subset of Aurora, proven included via feature predicates:

| Predicate | Content |
|---|---|
| `STAGE0-0001` | Stage-0 is a strict syntactic and semantic subset of Aurora |
| `STAGE0-0002` | Stage-0 includes: UTF-8/ASCII identifiers, modules, structs/enums, fixed integers/bool/byte, arrays/slices, functions, generics without specialization, Result/Option |
| `STAGE0-0003` | Stage-0 excludes: macros, runtime reflection, async, managed domains, dynamic typing, relations, decimal, scalable vectors/tensors, profile libraries, user-defined effect handlers |
| `STAGE0-0004` | Stage-0 grammar generated by disabling named Aurora productions, not maintained as unrelated grammar |
| `STAGE0-0005` | Stage-0 uses same checked arithmetic, evaluation order, initialization/drop, panic-abort policy, provenance, and C/omni_v1 ABI subset |
| `STAGE0-0006` | Seed compiler consumes canonical Stage-0 source and emits one reference object format/ISA plus deterministic diagnostic records |
| `STAGE0-0007` | Every Stage-0 restriction represented by feature predicate and tested against Aurora parser/model to prove subset inclusion |

**Every Aurora release requires Stage-0 verification discharged before publication.**

#### 6.2.2 Diverse Double Compilation (DDC)

- `BOOT-0004`: Diverse double compilation builds the same canonical compiler source with the trusted chain and an independently implemented diverse compiler/toolchain, then uses each result to rebuild and compares canonical outputs or declares semantic equivalence when signing/layout is external
- `BOOT-0005`: A diverse compiler qualifies only if its implementation lineage, language/tool dependencies, backend/link path, and build environment are sufficiently independent and documented
- `BOOT-0006`: DDC mismatch blocks release, preserves all artifacts/logs, and triggers differential localization; it is **never** waived merely because one binary passes tests
- DDC comparison: byte-for-byte canonical output convergence, or declared semantic equivalence with signed evidence

#### 6.2.3 Proof Obligations per Release

For every release, the following must be discharged:

1. **Stage-0 verification**: Feature predicates proved against Edition 1 parser/model
2. **DDC**: Same source rebuilt with trusted chain + diverse compiler; outputs compared
3. **Provenance**: Every artifact includes `compiler_id`, `compiler_version`, `source_revision` (`-dirty` convention), `codegen` fields
4. **Conformance claim**: `CONF-0001` through `CONF-0011` structure; assurance level, auditor, evidence, waivers, expiration, revocation status
5. **Build reproducibility**: Same inputs produce same outputs (verified via `BUILD-0001` through `BUILD-0011`)
6. **Spec digest match**: Computed `spec_tree_sha256` matches recorded digest in release manifest

#### 6.2.4 Verification Toolchain Requirements

- **Proof architecture**: Defined framework for discharging Stage-0 feature predicates (not merely asserted)
- **Trusted computing base**: Explicit list of trusted components (parser, solver, type checker, codegen, optimizer passes)
- **Equivalence criterion**: Exact byte-for-byte convergence OR declared semantic equivalence with signed evidence
- **Qualification boundary**: Which components must be independently verified vs. can rely on conventional testing
- **Verification tool validation**: Procedures for validating the verification tools themselves (regression against known-good programs, differential testing with independent tool)

**These are credible assurance goals, but the complete proof architecture, TCB, equivalence criterion, qualification boundary, and tool validation procedure must be specified before they can support implementation.**

### 6.3 Effect System

#### 6.3.1 Effect Rows as Types

Every function declares its effects in its type signature:

```
fn(f64, f64) -> f64 / ε       -- pure function, no effects
fn(FILE*) -> int / io, close  -- impure: io and close effects
fn(T) -> T / ε, dynamic       -- pure but may dynamic-dispatch
```

Effect row syntax: `ε ::= ε1, ε2` (comma-separated effect identifiers)

#### 6.3.2 Standard Effects

| Effect ID | Meaning |
|---|---|
| `io` | Input/output operations (file, network, device) |
| `dynamic` | Dynamic dispatch, reflection, runtime type analysis |
| `foreign` | FFI calls, external library invocations |
| `block` | Blocking operations (synchronous I/O in async context) |
| `log` | Logging operations |
| `unwind` | Exception/unwind propagation |
| `dynamic` | Runtime code generation or modification |

#### 6.3.3 Effect Capability Environment

Every function has an implicit capability environment `K` that tracks what authorities the function may exercise. Effects and capabilities are distinguished:

- **Effects**: What the function does (side-channels, observable actions)
- **Capabilities**: What the function is authorized to access

The semantics clearly distinguish effect requirements from capability passing and authority. A function may have effect `io` without having capability to open `/etc/passwd` — capability checks are separate from effect tracking.

#### 6.3.4 Effect Handling

- Effect rows are **union-compatible**: `ε1 ∪ ε2` combines effects from multiple subcomputations
- Structured task scopes: Children effects join into scope row; quorum(k) policy commits once k successes exist
- Cancellation: Effect operations can be cancelled before commit; after commit, the committed result is returned and cannot be rolled back
- Effect exhaustion: Bounded channels have capacity N; send commits when an element enters the buffer or is handed directly to a receiver

### 6.4 Ownership and Memory Model

#### 6.4.1 Type Categories

Omni's type system has these categories (from `core_semantics_and_types.md`):

1. **Primitive scalar types**: `Int`/`i64`, `Float`/`f64`, `Bool`, `Char`/`u32` (Unicode scalar), `Byte`/`u8`, `String` (UTF-8), `Unit`/`()`, `Never` (uninhabited)
2. **Generic parameters**: `GenericParam(String)`, `Infer(u32)` — fresh unification variables
3. **Compound types**: `Tuple(Vec<Ty>)`, `Array(Ty, usize)`, `Range(Ty)`, `Fn(Vec<Ty>, Ty)`, `Struct(String, Vec<Ty>)`
4. **Effect-tracked types**: Every type carries optional effect row
5. **Capability types**: `Cap(T)` — capability-restricted values
6. **Ownership types**: `Linear T`, `Affine T` — usage tracking
7. **Bottom type**: `Never` — uninhabited, coerces to any type

#### 6.4.2 Type Identity

- **Nominal**: For structs, enums, traits, opaque types, capabilities, effects, and aliases declared `distinct`
- **Structural**: For tuples, function types, references, raw pointers, arrays, slices, and generic instantiations — equal when all constituents and qualifiers are equal
- **Recursive nominal types**: Require indirection, unsized tail, managed reference, or opaque boundary; infinitely sized value types are erroneous

#### 6.4.3 Type Inference and Constraint Solving

- **Bidirectional type checking**: 
  - Checking mode: `Γ ⊢ e ⇐ T` (e has type T)
  - Synthesis mode: `Γ ⊢ e ⇒ T` (e infers type T)
- **Union-Find unification**: 
  - Two type variables `?A` and `?B` unify by merging their union-find sets
  - A type variable `?A` unifies with concrete type `Ty` if no conflicting binding exists
  - Incompatible concrete types (e.g., equating `Int` with `String`) produce `TypeError::MismatchedTypes`
- **Constraint solving**: Normalizes aliases and associated types, applies coherence-selected implementations, terminates under edition solver restrictions

#### 6.4.4 Linear and Affine Types

- **`Linear T`**: Value must be used exactly once — tracked via usage context covering branches, loops, moves, scope exit, and failure paths
- **`Affine T`**: Value may be used at most once; may also be unused (discarded at scope exit)
- **Move semantics**: Ownership transfer is tracked; moved-from values are invalided
- **Borrow tracking**: `FlowState.reference_loans` maps MIR locals to loan-region identifiers; must represent references stored in projected places/aggregates
- **Borrow conflict**: Shared references cannot authorize writes, moves, drops, or mutable reborrows. Mutable references have exclusive access.

**Critical gap (Phase 2 qualification blocker)**: `FlowState.reference_loans` cannot yet represent references stored in projected places or aggregates, or resolve a dereference back to its borrowed origin. `Rvalue::Use(Operand::Copy(place))` does not propagate loan associations to the destination. This is tracked as a qualification blocker and must be resolved before release.

#### 6.4.5 `unsafe` Discipline

- `UNSAFE-0001`: `unsafe` permits performing operations with programmer-proved preconditions; **does not** disable ordinary typing, ownership, effect, capability, initialization, or control-flow checks
- `UNSAFE-0002`: Unsafe operations legal only inside `unsafe` block/function or through unsafe trait implementation; unsafe effect visible in MIR and audit reports
- `UNSAFE-0003`: Calling an unsafe function requires unsafe context even when caller can prove preconditions; safe wrappers discharge and document the proof
- `UNSAFE-0004`: Each unsafe operation has named obligation record with precondition, protected invariant, permitted optimizer assumptions, and failure containment class
- `UNSAFE-0005`: If unsafe precondition is false at event `e`, execution becomes invalid from `e` onward; observations before `e` remain required
- `UNSAFE-0006`: Optimizer may assume unsafe precondition only on paths reaching the operation and only while facts/objects it depends on remain live; assumption provenance points to the obligation
- `UNSAFE-0007`: Invalid unsafe execution does not authorize compile-time compromise, cross-process effects, or modification of immutable artifacts; outcomes outside language model addressed by hardening

#### 6.4.6 Memory Safety Invariants

- **No invalid bit patterns**: Valid scalar bit patterns cannot be constructed or observed in safe code (`TYPE-0004`)
- **Reference validity**: References are non-null, aligned, live, and provenance-valid (`TYPE-0005`)
- **Enum invariants**: Enums have a declared live variant; zero-sized types occupy no logical bytes but have distinct ownership/drop events (`TYPE-0006`)
- **Never coercion**: `Never` is uninhabited and coerces to any type (`TYPE-0007`)
- **Unsized types**: `[T]`, `str`, `dyn Trait`, and declared extern types may occur only behind a pointer/reference/owner or as the final field of a declared dynamically sized aggregate (`TYPE-0008`)

### 6.5 Concurrency Model

#### 6.5.1 Execution Agents

Every Aurora program runs as distinct execution agents with profile-defined capabilities (`CONC-0001` through `CONC-0023`):

| Concern | Rule |
|---|---|
| `CONC-0001` | Native threads, scoped tasks, async tasks, actors, and interrupts are distinct execution agents with profile-defined capabilities and memory participation |
| `CONC-0002` | Thread/task creation publishes all moved arguments and initialized captured state to the child start; child completion synchronizes with successful join |
| `CONC-0003` | A structured task scope cannot complete until every child is joined, cancelled and joined, or transferred to an authorized long-lived supervisor |
| `CONC-0004` | Standard scope policies: `all`, `cancel_on_error`, `first_success`, `race`, `supervise`, `quorum(k)` with exact aggregation rules |
| `CONC-0005` | `all` waits for all children and returns ordered results by spawn order |
| `CONC-0006` | `cancel_on_error` commits first error/panic by completion order, requests cancellation of unfinished siblings, joins them, and returns committed cause with suppressed causes |
| `CONC-0007` | `first_success` returns first committed success, cancels and joins others; if none succeeds, aggregates terminal causes by completion order |
| `CONC-0008` | `race` returns first committed terminal outcome |
| `CONC-0009` | Bounded channels have capacity N; send commits when an element enters the buffer or is handed directly to a receiver. Receive commits when ownership of one element transfers to the receiver |
| `CONC-0010` | Closing a channel prevents new successful sends; buffered elements remain receivable. After buffer empties, receive returns `Closed`. Concurrent close/send decided by operation's atomic commit order |
| `CONC-0011` | Channel ordering is FIFO per sender and globally FIFO by send commit order for a single channel. Fairness not guaranteed unless channel type/profile states it |
| `CONC-0012` | Cancellation before a channel operation commits leaves no element transferred. Cancellation after commit returns/records the committed result and cannot roll it back |
| `CONC-0013` | Standard mutex lock acquisition may be unfair and may wake spuriously only through condition-variable waits, not lock success. Fair mutexes are separate types |
| `CONC-0014` | Condition-variable wait atomically releases the mutex and blocks, may wake spuriously, then reacquires before returning. Callers must test predicates in a loop |
| `CONC-0015` | Semaphores commit permit decrement/increment atomically; cancellation before acquire commit consumes no permit |
| `CONC-0016` | Standard mutexes poison after panic exits a held critical section. Poison is advisory and recoverable through explicit API; not memory unsafety |
| `CONC-0017` | An executor SHALL not poll one task concurrently with itself, SHALL tolerate duplicate/coalesced wakes, and SHALL not poll after terminal completion |
| `CONC-0018` | Polling a future is non-reentrant unless the future explicitly implements a reentrant protocol. A wake during poll schedules a later poll and does not recursively enter the task |
| `CONC-0019` | Blocking operations in an async context require the `block` effect and a provider that routes them to a blocking resource or rejects them |
| `CONC-0020` | Actor mailboxes preserve message commit order per sender; cross-sender order is by mailbox commit. A restart never rolls back external effects already committed |
| `CONC-0021` | Region transfer in a message commits ownership atomically with enqueue. On enqueue failure ownership remains with the sender; on committed delivery failure, supervisor policy owns disposal/retry |
| `CONC-0022` | Thread-local values initialize per native thread, are inaccessible from tasks migrated between threads unless task-local storage is used, and drop at orderly thread exit only |
| `CONC-0023` | Signals/interrupts may access only async-signal/interrupt-safe operations declared by the target/profile. Ordinary locks, allocation, and destructors are prohibited unless explicitly supported |

#### 6.5.2 Structured Task Scopes

- Scopes enforce completion ordering policies
- `quorum(k)`: Commits once k successes exist; returns them in completion order; cancels/joins remaining children; impossibility to reach k returns aggregated failure
- Nested scopes: Inner scope completion synchronizes with outer scope; cancellation propagates inward per profile
- Long-lived supervisors: Orphaned work may be transferred to authorized long-lived supervisor; never an orphan

#### 6.5.3 Channel Semantics

- **Send**: Commits when element enters buffer or is handed directly to receiver; after buffer empties, receive returns `Closed`
- **Receive**: Commits when ownership of one element transfers to receiver; FIFO per sender, globally FIFO by send commit order for single channel
- **Close**: Prevents new successful sends; buffered elements remain receivable
- **Concurrent close/send**: Atomic commit order decides; if both occur, the committed operation wins

#### 6.5.4 Cancellation and Failure Propagation

- Cancellation before operation commit: No element transferred
- Cancellation after commit: Committed result returned/recorded; cannot roll back
- Panic propagation: First panic in `cancel_on_error` scope commits as cause; unfinished siblings joined and cancelled
- Resource cleanup: On scope exit, all children joined/cancelled; resources released in reverse dependency order
- Region transfer: Ownership atomically transferred with enqueue on message; failure modes defined per `CONC-0021`

### 6.6 Syntax and Grammar

#### 6.6.1 Language Syntax (AML — Aurora Mark Language)

The surface syntax is defined in **AML** (Aurora Mark Language), a Markdown+YAML format that is both human-readable and machine-checkable.

**Key syntax categories**:

```yaml
language: Aurora
version: "1.0.0.0"

syntax:
  # Statement termination
  stmt_terminator: newline
  newline_terminates: true
  continuation_tokens: [binary_operators, ".", "?.", "?", "??", ",", ")", "]", "}", "::"]
  
  # Function definition
  function_def:
    modifiers: ["pub", "const", "unsafe", "async", "effect"]
    signature: "fn" identifier generic_params "(" param_list ")" "->" type "where" where_clause
    body: block_expr
  
  # Effect row in types
  effect_row: "effect" identifier ":" effect_bound
  
  # Generic parameters
  generic_params: "<" generic_param { "," generic_param ">"
  generic_param:
    lifetime: "'" identifier
    type: identifier ":" type_bound { "+" type_bound }
    const: "const" identifier ":" type
    effect: "effect" identifier
    cap: "cap" identifier
  
  # Parameter list
  param_list: param { "," param } [ "," ]
  param: [ "mut" ] identifier ":" type
  
  # Where clause
  where_clause: "where" generic_bound { "and" generic_bound }
  
  # Block expression
  block_expr: "{" module_item { } "}"
  
  # If expression
  if_expr: "if" expression "{" block_expr "}" ("else" ("{" block_expr "}" | expression))?
  
  # Match expression
  match_expr: "match" expression "{" arm { ";" arm } "}"
  arm: pattern "=>" expression
  
  # Literals
  integer_literal: (decimal | binary "0b" | octal "0o" | hex "0x") integer_suffix?
  float_literal: (decimal_float | hex_float) float_suffix
  char_literal: "'" (escape_sequence | non_quote_char) "'"
  string_literal: "\"" string_content "\""
  raw_string_literal: "r" "#"* "\"" raw_string_content "\""# (matching # count)
  
  # Comments
  line_comment: "//" [^"\n"]*
  block_comment: "/*" (block_comment_content | block_comment)* "*/"
  doc_line_comment: "///" [^"\n"]* | "//!" [^"\n"]*
  doc_block_comment: "/**" block_comment_content "**/" | "/*!" block_comment_content "*/"
```

#### 6.6.2 Grammar Productions (Normative)

The normative grammar is generated by disabling named Edition 1 productions (not maintained as unrelated grammar). Key productions:

- **Keywords**: `Never`, `Self`, `Sized`, `addrspace`, `as`, `async`, `await`, `bare_metal`, `bf16`, `bool`, `break`, `byte`, `cap`, `catch`, `char`, `const`, `continue`, `dec128`, `dec32`, `dec64`, `defer`, `distributed`, `dyn`, `effect`, `else`, `enum`, `ensure`, `extern`, `f128`, `f16`, `f32`, `f64`, `false`, `fn`, `for`, `hosted`, `i128`, `i16`, `i32`, `i64`, `i8`, `if`, `impl`, `in`, `is`, `isolate`, `isize`, `let`, `loop`, `macro`, `managed`, `match`, `module`, `move`, `mut`, `not`, `opaque`, `override`, `package`, `panic`, `persistent`, `pub`, `pure`, `ref`, `relation`, `require`, `return`, `script`, `self`, `static`, `str`, `struct`, `super`, `thread_local`, `trait`, `true`, `try`, `type`, `typeof`, `u128`, `u16`, `u32`, `u64`, `u8`, `usize`, `use`, `verified`, `where`, `while`, `yield`

- **Identifier**: `(XID_Start | "_") (XID_Continue | "_")*`
- **Raw identifier**: `r#` identifier
- **Integer literal**: `(decimal | binary | octal | hex) integer_suffix?`
- **Floating-point literal**: `(decimal_float | hex_float) float_suffix?`
- **Character literal**: `'` (escape_sequence | non_quote_char) `'`
- **String literal**: `(` `b` | `r` | `br` | `rb` `)` `"` string_content `"`
- **Raw string literal**: `r` `#*` `"` raw_string_content `"#` (matching `#` count)
- **Escape sequence**: `\` (`0` | `t` | `n` | `r` | `"` | `'` | `\` | `x` hex_digit hex_digit | `u` `{` hex_digit+ `}`)
- **Line comment**: `//` `[^"\n`]* 
- **Block comment**: `/*` (block_comment_content | block_comment)* `*/`
- **Documentation comments**: `///` `[^"\n`]* | `//!` `[^"\n`]* ; `/**` block_comment_content `**/` | `/*!` block_comment_content `*/`

#### 6.6.3 Statement Continuation Rules (VIBE-GRAM)

Newline termination is unambiguous after continuation rules are applied:

- `VIBE-GRAM-0001`: Newline MAY terminate a statement when the parser is at a complete statement boundary and the next token cannot continue the current expression or declaration
- `VIBE-GRAM-0002`: A newline MUST NOT terminate a statement when the current construct is syntactically incomplete or the next token is a designated continuation token
- `VIBE-GRAM-0003`: Continuation tokens include binary operators, `.`, `?.`, `?`, `??`, `,`, `)`, `]`, `}`, `::`, and other grammar-declared continuation forms. The complete set is normative.
- `VIBE-GRAM-0004`: Semicolons MAY be used explicitly and remain grammar separators; they are not required at ordinary statement boundaries when newline termination is unambiguous
- `VIBE-GRAM-0005`: Formatter output MUST be canonical and MUST NOT rely on indentation alone to alter the semantic interpretation of an already valid expression
- `VIBE-GRAM-0006`: Braced blocks remain valid in every context where the grammar admits blocks. Indentation-first formatting is preferred but does not create a second semantic language
- `VIBE-GRAM-0007`: Release translation MUST reject source for which newline interpretation remains ambiguous after the continuation rules are applied. IDE recovery MAY construct recovery nodes, but those nodes remain non-translatable

**Important**: The normative EBNF predates Candidate-2 amendment and contains no pipeline-operator, newline-termination, projection-shorthand, or command-call productions, while VIBE-GRAM rules normatively require them. Grammar modernization belongs to the grammar milestone.

#### 6.6.4 Effect Syntax

- Function signature effects: `fn(T) -> T / ε` where `ε` is the effect row
- Effect separator `/` never division in signature position after `->`
- Effect rows in function bodies: effect operations are first-class expressions

#### 6.6.5 Statement Modifiers

- `async`: Declares function as async (returns pinned future); poll uses explicit context/waker ABI
- `effect`: Declares effect row parameter; `fn(T) -> T / effect CapabilityName`
- `const`: Function is const-evaluable (same value arithmetic, ownership, pattern, panic, and pure call semantics as runtime, with deterministic quotas and no undeclared external effects)
- `unsafe`: Permits operations with programmer-proved preconditions; does not disable ordinary checks

---

## 7. Rule Registry and Ownership

### 7.1 Rule Registry Structure

Rules are organized in `registry/rules.json` with associated `registry/rule-texts.json`. Each rule has:

- `rule_id`: Unique identifier (e.g., `ERR3-0010`, `CONF-0001`, `BUILD-0001`)
- `text`: Human-readable description of the rule
- `schema_version`: Version of the rule schema
- `lifecycle_state`: `Proposed` | `Candidate` | `Ratified` | `Deprecated` | `Superseded` | `Withdrawn`

### 7.2 Linkage Reaper (`omni-audit`)

- `#[implements("ID")]` and `implements!("ID")` claims under `compiler/` and `tools/` (generated `target/` excluded, filename-sorted walk) must resolve to registry rules in `Candidate` or `Ratified` state
- `Proposed`/`Deprecated`/`Superseded`/`Withdrawn` ownership, unknown IDs, and malformed claim syntax all **fail closed**
- Unparseable sources, symlinks, missing trees, duplicate registry IDs, dependency cycles, and unknown statuses all fail closed
- Witness entries must each resolve to a contained workspace file or a collected `#[test]` name
- Every `Ratified` rule must be both claimed and witnessed
- **Coverage is per-rule, not per-file**: Untagged infrastructure/derived/oracle code is permitted
- **Duplicate claims across files**: Allowed and attributed (collaborative implementation)

**Current live state**: 8 tags in 57 files (74 test functions), clean.

**Tracked gaps** (not invented):
- RULE-0003 `erratum-corrected` has no enum spelling (needs spec ADR)
- Rule-text to hash binding is procedural (deterministic extraction, tree-pinned registry bytes)
- `models/`/`data` emptiness belongs to later milestones

### 7.3 Rule Lifecycle

- `Proposed`: New rule under consideration; not yet ratified
- `Candidate`: Under active review; may become Ratified
- `Ratified`: Officially accepted; rule is in force
- `Deprecated`: Retired but retained for backward compatibility
- `Superseded`: Replaced by newer rule; old rule remains valid for existing artifacts
- `Withdrawn`: Removed entirely; not valid for any new artifact

### 7.4 Conformance and Rule State

- An implementation claim names the edition, modules, profiles, targets, limits, external ABI/data versions, implementation-defined choices, deviations, and test-result digest
- Strict mode disables extension syntax/semantics and rejects artifacts whose unrecognized extension could affect behavior; namespaced extensions may be tested only in separate claims
- Parser-only, core-subset, profile, platform, distribution, and complete-release claims are distinct; partial implementations cannot claim complete conformity

---

## 8. Priority Issues and Action Items (from Design Review)

Based on the October 2026 design review conversation, these are the highest-priority issues for Omni:

### 8.1 Normative Authority and Semantic Consistency **(Priority 1)**

- **Resolve EBNF/VIBE-GRAM conflict**: The grammar-modernization item needs careful treatment with explicit precedence and consistency review, not automatic rewrite
- **Action**: Document the precedence decision; update affected rule IDs and grammar productions; ensure all subsequent references are consistent

### 8.2 Reference and Ownership Soundness **(Priority 2)**

- **Investigate `FlowState.reference_loans` gap**: Cannot represent references held in projected places or aggregates; cannot resolve dereference back to borrowed origin
- **Resolve reference copyability**: Before implementing propagation, decide among three options:
  1. Shared references are copyable, with shared loan ownership tracked across every live alias; mutable references remain affine
  2. All references are affine and move-only (simplest provenance model but restricts shared reference reuse)
  3. Copyability determined by explicit normative trait/rule, with reference-specific constraints stated in specification
- **Action**: Resolve the authority/precedence mismatch before treating the compendium as the source of a new normative rule; record the decision and update normative text

### 8.3 Canonicalization and Evidence Integrity **(Priority 3)**

- **Precisely define specification digest domain**: What goes into `spec_tree_sha256`, how canonicalization works, how rule references are validated
- **Do NOT accept "omni-canon not started → every digest is unsound" solely from notes**: Establish from actual code/specification
- **Action**: Define the exact exclusion procedure; validate against the build pipeline; ensure conformance claims are only made when digest integrity is confirmed

### 8.4 Compilation and Release Evidence **(Priority 4)**

- **Define DDC in practice**: What independent compiler qualification means, what is compared, which components are trusted, what happens on disagreement
- **DDC should be clearly specified assurance requirement** with evidence and applicability conditions, not just a checkbox
- **Action**: Document the DDC procedure, equivalence criterion, TCB, and disagreement resolution protocol

### 8.5 Profile Enforcement and Long-term Compatibility **(Priority 5)**

- **Ensure profile enforcement**: `safe` profile and any other profiles defined in normative terms and enforced by compiler and relevant tooling
- **Design controlled language-amendment and migration process**: Once the underlying semantics are stable
- **Action**: Profile definitions must be normative; enforcement mechanisms must exist; migration path must be documented

### 8.6 Specification-Digest Domain **(Additional)**

- **Explicitly define** what file sets go into `spec_tree_sha256` exclusion set
- **Procedure**: Walk included directories in lexicographic order, normalize CRLF/CR→LF, hash each file with SHA-256, hash concatenation of `path + LF + hex + LF` lines
- **Current computed digest**: `7e63431f3c624f9818454f076c38f5cde3d517ace9e919926d6188bc177be769` (supersedes earlier values by REL-0007 machine-block formatting)
- **Earlier digest**: `07762061…` (by gate-closure content, superseded)
- **omni-canon status**: Not yet started — this must be established from actual implementation before digest integrity can be claimed

### 8.7 Grammar Modernization **(Additional)**

- **The conflict**: Normative EBNF predates Candidate-2 amendment; lacks pipeline-operator, newline-termination, projection-shorthand, command-call productions
- **VIBE-GRAM rules**: Normatively require these productions
- **Resolution**: Grammar modernization belongs to the grammar milestone; the tree binds the artifact as-is. Any resolution must document the precedence decision and update affected rule IDs

### 8.8 Feature-Grade Matrix Qualification **(Additional)**

- **Do not treat the feature matrix as binding architectural decisions**: Dependent types, GC, runtimes, and concurrency approaches are useful design prompts but not adequate grounds for global prohibitions
- **Profile-based approach**: Common foundation with separate assurance profiles is preferred over feature prohibition
- **Artificial intelligence caution**: Collapsing provenance, data lineage, and information-flow labels into "every value must carry runtime provenance" should not be done unless Omni has a specific semantic reason and accepts the associated cost

### 8.9 Undefined Behavior Contract **(Additional)**

- **Define precisely** what happens when: out of memory, arithmetic fault, capability precondition violation, foreign code interaction, hardware exception, non-termination
- **Four categories**:
  1. Behavior guaranteed by language semantics
  2. Explicit, defined program failures (errors, panics)
  3. Programs rejected during compilation/verification
  4. Platform-dependent behavior permitted by declared target contract
  5. Hardware faults and external failures language cannot eliminate
- **Goal**: Each category has specified consequences; invalid programs cannot obtain conforming release status

### 8.10 Hardware Requirements Classification **(Additional)**

- **Radiation tolerance**: Language can provide mechanisms for bounded execution, explicit failure handling, redundancy, memory-safety checks, and platform diagnostics; **cannot** make physical memory radiation-hardened or guarantee hardware-fault detection
- **GC prohibition**: Too broad as universal requirement; relevant properties are predictable timing, bounded resource use, explicit allocation behavior, and profile-specific restrictions
- **Profile approach**: Critical profile might prohibit managed allocation; general-purpose profile supports it

### 8.11 Specification Clause Qualification **(Additional)**

- **Design requirement, specification clause, implementation, passing test suite, proof of correctness are five different things**
- **Omni's engineering plans and release claims should preserve these distinctions throughout**
- **The first document's "already implemented" claims are evidence of what the author believes the specification covers, not proof of implementation state**

---

## 9. Conversation-Integrated Design Decisions

### 9.1 Four-Grade Framework Replacement

**Rejected**: Feature-checklist matrix assigning dependent types, GC, runtimes, and concurrency approaches to individual grades.

**Adopted**: Common language foundation with assurance profiles. Each profile declares permitted features, runtime facilities, allocation policies, concurrency guarantees, supported targets, required evidence, and release gates. The same language supports productive application development and more restrictive critical applications without pretending identical requirements.

**Profiles** (defined in Section 3.2):
- `safe`: No `unsafe`, no `foreign` effect, no raw pointers
- `distributed`: Message-passing-only state sharing
- `real-time`: Bounded execution, priority inheritance, bounded preemption
- `high-assurance`: Full formal verification discharge, DDC mandatory, no undefined behavior
- `general-purpose`: Productive application development with GC, dynamic dispatch, runtime reflection

### 9.2 Formal Semantics Commitment

**Rejected**: "Formal semantics rather than tests" as false dichotomy.

**Adopted**: Formal semantics and proofs establish properties testing cannot, but trustworthy implementation still needs independent tests, regression suites, fuzzing, differential testing, build reproducibility checks, and trusted-tool validation. The TCB, assumptions, and model of the proof must be explicitly documented.

**Formal semantics** (ACL) is the primary artifact; test suites are complementary, not substitute.

### 9.3 "No Undefined Behavior" Commitment

**Rejected**: Phrase alone without precise contract.

**Adopted**: Four-category definition:
1. Behavior guaranteed by language semantics
2. Explicit, defined program failures (errors, panics)
3. Programs rejected during compilation or verification
4. Platform-dependent behavior permitted by declared target contract (with hardware/fault exceptions as separate category)

**Goal**: Each category has specified consequences; invalid programs cannot obtain conforming release status.

### 9.4 Profile-Based Grade Framework

**Rejected**: Four grades as interchangeable columns in feature matrix.

**Adopted**: Common foundation with profiles that inherit from it, with explicit deviations. This is better than globally prohibiting features such as GC or dynamic dispatch merely because a more restrictive profile might not permit them.

### 9.5 Aurora Integration Policy

**Rejected**: Merging Aurora's syntax/semantics wholesale into Omni.

**Adopted**: Aurora treated as research inspiration artifact. Its formal model and examples need substantial correction before supporting implementation. Its duplicated `BUILD`, `CONF`, `UPDATE`, and `TARGET` identifier ranges carry provenance — should not be used as independent evidence of Aurora's maturity.

**Specific rejections**:
- Do not import Aurora's function syntax, continuation rules, effect syntax, linearity rules, or YAML semantic representation as ready-made solutions
- Do not change Omni's normative Edition 1 grammar or revive disabled Candidate-2 syntax based on Aurora
- Aurora's generic-parameter production delimiter error, truncated `Unit` entry, and mandatory `where` clause appearance are not acceptable in Omni's normative specification

### 9.6 Specification Clause Distinction

**Core principle**: Design requirement, specification clause, implementation, passing test suite, and proof of correctness are five different things.

**Application**: Omni's engineering plans and release claims must preserve these distinctions throughout. The first document's "already implemented" claims are beliefs about specification coverage, not implementation state.

### 9.6 Canonicalization Engine

**Critical path item**: `omni-canon` not yet started per implementation plan. Without it, all digests and conformance claims are unsound.

**Action**: This is a foundation item — repository initialization, rule registry schema, specification manifest, and canonicalization engine must be established before later work can be qualified.

### 9.8 Reference Provenance Blockers

**Qualification blocker**: `FlowState.reference_loans` cannot represent references stored in projected places or aggregates, nor resolve dereference back to borrowed origin.

**Action**: Track as Phase 2 qualification item; resolve before release; do not implement reference copy propagation until ownership/borrow rules are sound.

### 9.10 Specification Digest Domain

**Action**: Precisely define exclusion set for `spec_tree_sha256`. The current computed digest `7e63431f3c624f9818454f076c38f5cde3d517ace9e919926d6188bc177be769` supersedes earlier values by REL-0007 machine-block formatting. `omni-canon` must be started and validated before digest integrity can be claimed for release.

### 9.12 DDC Definition

**Action**: Define DDC as mandatory release requirement with:
- Procedure: Same canonical source rebuilt with trusted chain + diverse compiler
- Equivalence criterion: Byte-for-byte canonical output convergence OR declared semantic equivalence with signed evidence
- TCB: Explicit list of trusted components
- Disagreement resolution: Block release, preserve all artifacts/logs, trigger differential localization — never waived because one binary passes tests

---

## 10. File Structure and Navigation

The specification is organized into the following sections for ease of reference:

```
Omni_Complete_Specification_Generated.md
├── 1. Language Governance          (versioning, architecture freeze, bootstrap)
├── 2. Specification Authority       (digest domain, conflict resolution, normative vs non-normative)
├── 3. Conformance and Certification (conformance claims, assurance profiles, certification revocation)
├── 4. Build Pipeline                (content-addressed builds, build gates, provenance)
├── 5. Target Model                  (canonical target identity, baseline restrictions, feature dispatch)
├── 6. Formal Semantics and Verification (ACL, Stage-0, DDC, proof obligations, effect system, ownership)
├── 7. Rule Registry and Ownership   (rule lifecycle, linkage reaper, conformance and rule state)
├── 8. Priority Issues and Action Items (review feedback, ranked priorities)
├── 9. Conversation-Integrated Design Decisions (synthesis of discussion)
└── 10. File Structure and Navigation
```

---

## References and Cross-References

This generated specification references the following Omni project artifacts (all paths relative to `C:\Users\siddh\Downloads\ABC\Omni/`):

- `docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.1_updated.md` — Original normative specification (7326 lines)
- `docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.2_vibe.md` — Candidate 2 vibe reconciliation (7614 lines)
- `docs/implementation/Omni_Implementation_Master_Plan_Edition1_Final_Comprehensive_MicroAtomic_Final.md` — Implementation master plan (4750 lines)
- `docs/foundation-gates.md` — Foundation sequence and gate definitions (249 lines)
- `docs/grammar-reconciliation.md` — Grammar reconciliation notes (204 lines)
- `spec/grammar/omni-edition1.ebnf` — Normative EBNF grammar (390 lines)
- `spec/reference/core_semantics_and_types.md` — Core semantics and type system (286 lines)
- `spec/registry/rules.json` — Rule registry (5109 lines of rule texts)
- `spec/registry/rule-texts.json` — Rule text schema
- `docs/architecture/generic_closer_contract.md` — Generic closer contract
- `docs/architecture/lifetime_lex_parse_contract.md` — Lifetime/lexical/parse contract
- `IMPLEMENTATION_PLAN.md` — Implementation roadmap (406 lines)
- `IMPLEMENTATION_PLAN_GENERATED.md` — Generated implementation plan (264 lines)
- `foundation-gates.md` — Foundation gates (0.0.0.2 through 0.0.0.13)
- `candidate2-erratum.md` — Candidate 2 errata (92 lines)

**Conversation sources**:
- Design review of "Idealized Programming Language Design" analysis (October 2026)
- Aurora language proposal analysis (October 2026)
- Military language requirements (DoD "Ironman" document, 1976)
- Space language requirements (NASA reports, Apollo/Shuttle eras)
- Formal methods literature (type systems, verification, formal semantics)
- Enterprise language design patterns (module systems, build pipelines, package management)

---

## Generation Notes

This file was **synthesized** from:
1. The Omni project repository specification and implementation materials
2. External research on language design for extreme environments (space, military, enterprise)
3. Design review feedback on both the original analysis and Aurora proposal
4. October 2026 conversation discussion of idealized programming language requirements

**Intent**: This is a **temporary reference** file — it captures the design decisions, priority issues, and conversation-integrated synthesis for ongoing Omni engineering work. It is not itself a normative release artifact; the canonical specification remains the Omni project's Edition 1 specification in `docs/specification/`.

**Update policy**: This file should be updated when:
- Omni's normative specification is revised (errata, amendments)
- New assurance profiles are defined or existing ones modified
- Priority issues are resolved (tracked in issue/issue-tracking system)
- The `omni-canon` canonicalization engine is started and validated
- DDC procedure is formally specified and validated
- Reference/ownership soundness gaps are closed

**Verification status**: This file synthesizes information from multiple sources; individual claims should be verified against the original Omni project artifacts and the peer review conversation. The five-fold distinction (design requirement, specification clause, implementation, passing test suite, proof of correctness) must be preserved when evaluating any part of this specification.

---
*Generated: October 2026 — Synthesized from conversation research and Omni project analysis*