# Omni Implementation Master Plan
## Edition 1 — Frozen Architecture → Fully Qualified Implementation

**Document type:** Implementation plan only
**Language target:** Omni Edition 1
**Normative specification status:** `1.0.0-candidate.1` architecture baseline
**Implementation status:** Not started by this plan
**Starting implementation language:** Rust
**Primary implementation strategy:** Stage 1 Rust-only minimal compiler → Stage 2 Rust-only complete production compiler → Stage 3 Omni-written compiler, self-compilation, DDC, and dependency severance
**Versioning scheme:** `Release.Major.Minor.Patch` (`R.M.m.P`)
**Generational gates:** `1.0.0.0` = Minimal Rust Core; `2.0.0.0` = Complete Production Rust Platform; `3.0.0.0` = Certified Self-Hosted and Independent Omni

---

# 0. Purpose and Boundary

This document is the **implementation master plan** for Omni Edition 1.

It is intentionally separate from the complete Omni specification.

The complete specification defines **what Omni means**.

This document defines **how the implementation is to be constructed, integrated, tested, qualified, maintained, and eventually certified against that specification**.

No item in this plan is allowed to silently create language semantics that are absent from the normative specification.

When an implementation discovers a genuine semantic gap, contradiction, ambiguity, or impossible requirement, the process is:

1. stop the affected implementation path;
2. record the defect against the exact specification/rule IDs;
3. determine whether the issue is an implementation defect or a specification defect;
4. if it is an implementation defect, repair the implementation;
5. if it is a specification defect, open a controlled specification erratum/amendment process;
6. do not invent an implementation-local interpretation;
7. resume implementation only after the authoritative decision is available.

The implementation plan therefore treats the specification suite as the semantic authority and the repository as an executable proof/qualification effort against that authority.

---

# 1. Architecture Freeze

## 1.1 Frozen milestone

**Omni Edition 1 — Architecture/Specification Phase: COMPLETE**

**Status: Semantic-closure complete in architecture; implementation qualification pending.**

The architecture phase is not to be reopened merely to add convenience features, implementation preferences, or speculative optimizations.

A normative amendment during implementation requires one of:

- a discovered contradiction between normative artifacts;
- a discovered semantic case not classified by the specification;
- a soundness defect;
- an interoperability defect that violates an existing contract;
- an implementation impossibility caused by a genuine normative inconsistency;
- a required clarification whose meaning is already uniquely implied but not mechanically stated.

An implementation team must not use implementation difficulty alone as justification for weakening a safety guarantee.

---

# 2. Starting Programming Language

## 2.1 Selected bootstrap language: Rust

The initial implementation language is **Rust**.

Rust is the bootstrap implementation language because the existing Omni architecture explicitly identifies Rust as the Stage-0 bootstrap environment and aligns it with the intended implementation tooling, including Polonius-oriented ownership analysis, Cranelift integration, structured compiler infrastructure, and strong compile-time safety properties.

This choice applies **only to the initial implementation phase**.

Omni is the language being created; Rust is not the language being created.

The intended evolution is:

```text
Rust
  ↓
Rust Stage-0 compiler
  ↓
Rust compiler capable of compiling Stage-0/partial Omni
  ↓
Omni implementation of the compiler
  ↓
Omni compiles Omni
  ↓
DDC / independent verification
  ↓
Rust reduced to validation/bootstrap-recovery role
```

The bootstrap model already specifies this transition and the eventual Stage-1/Stage-2 comparison.

## 2.2 Rust version policy

The exact Rust compiler version SHALL NOT be treated as an informal preference.

At bootstrap initialization:

- choose one exact stable Rust toolchain;
- record the exact toolchain version in `rust-toolchain.toml`;
- pin required components;
- pin required target triples;
- record the toolchain checksum or equivalent release identity;
- record all external tool versions that participate in deterministic artifacts;
- record the toolchain as part of the build provenance.

Rust upgrades are implementation changes and must go through the implementation qualification process.

The plan deliberately does **not** hard-code a future Rust patch release today. The exact pinned version is selected at actual implementation initialization and then frozen for the applicable implementation generation.

## 2.3 Rust is not the semantic authority

Rust compiler behavior, Rust MIR behavior, Cranelift behavior, LLVM behavior, or any other host compiler behavior must never be treated as an Omni semantic rule.

They are implementation mechanisms subject to Omni's semantic contracts.

---

# 3. Implementation Versioning

Omni implementation versions use exactly:

```text
Release . Major . Minor . Patch
R       . M     . m     . P
```

The version is an engineering identity, not merely a marketing label. Every component has a defined meaning and every stage boundary is encoded into the release-generation component.

## 3.1 Release (`R`)

The release component identifies the generational implementation family:

- `0` — construction and qualification before the first minimal release gate;
- `1` — the minimal Rust release generation and the complete Rust production generation leading to `2.0.0.0`;
- `2` — the self-hosting construction/severance generation leading to `3.0.0.0`;
- `3` — the first certified independently self-hosted Omni generation.

The stage boundary releases are fixed: `1.0.0.0`, `2.0.0.0`, and `3.0.0.0`.

## 3.2 Major (`M`)

A major component identifies a large internal implementation-generation transition inside a release family. It can denote a compiler pipeline generation, trust-base transition, backend generation, or similarly significant implementation boundary.

A major increment never authorizes a change to Omni semantics by itself. The normative specification remains authoritative.

## 3.3 Minor (`m`)

A minor component identifies a completed and integrated subsystem or milestone.

Examples:

```text
1.0.1.0  advanced types
1.0.2.0  memory model
1.0.3.0  effects/capabilities
2.0.1.0  Omni lexer
2.0.2.0  Omni parser
2.0.3.0  Omni semantic analyzer
```

A minor milestone is not complete until implementation, integration, tests, regression impact, linkage audit, and qualification all pass.

## 3.4 Patch (`P`)

The patch component identifies an atomic engineering change inside a minor milestone. A patch must be independently understandable and qualified.

Example:

```text
2.0.1.1
2.0.1.2
2.0.1.3
```

The patch component must not be used as an undeclared feature counter.

## 3.5 Stage mapping

```text
Stage 1: `0.0.0.1` → `1.0.0.0`
Stage 2: `1.0.0.1` → `2.0.0.0`
Stage 3: `2.0.0.1` → `3.0.0.0`
```

Within each interval, the same `R.M.m.P` rules apply.

---

# 4. Version Transition Rules

## 4.1 Patch transition

A patch may advance only when its declared dependencies are satisfied, affected tests pass, semantic linkage passes, no stale implementation remains, and the repository remains coherent.

## 4.2 Minor transition

A minor transition requires the subsystem-level integration gate to pass. All previous qualified behavior remains regression-qualified.

## 4.3 Stage transition

A stage transition is a release gate and requires an explicit evidence package.

`1.0.0.0` seals the Minimal Rust Core.

`2.0.0.0` seals the Complete Production Rust Platform and freezes it as the Stage-3 bootstrap baseline.

`3.0.0.0` seals the Self-Hosted and Independently Qualified Omni toolchain.

## 4.4 No stage skipping

No implementation may claim a later-stage version while an earlier stage remains unqualified.

## 4.5 Semantic compatibility

A version transition never authorizes silent semantic divergence from the normative specification.

## 4.6 Historical artifacts

Superseded plans may be retained as history, but active implementation and planning references must point only to the current authoritative model.
# 5. Fundamental Implementation Laws

These rules apply to the entire project.

## 5.1 Specification authority

The implementation SHALL consume the normative specification suite and its machine-readable manifests.

Compiler code MUST NOT become a second semantic authority.

## 5.2 One-way semantic dependency

The intended dependency direction is:

```text
specification data
       ↑
       │
compiler/tools/conformance
```

In repository terms:

```text
compiler/*  ─────→  spec/*
tools/*     ─────→  spec/*
conformance/* ────→ spec/*
```

and never:

```text
spec/* ─────→ compiler/*
```

The specification must remain independently inspectable.

## 5.3 No semantic duplication

No semantic fact should be independently hard-coded in multiple locations.

Examples of prohibited duplication:

- grammar precedence repeated manually in parser branches;
- fault categories independently recreated in runtime and diagnostics;
- capability rights represented differently by compiler and runtime;
- numeric semantics separately maintained by interpreter and code generator.

Where a representation is needed for performance, it must be derived from the authoritative definition or validated against it.

## 5.4 Fail closed

Unsupported semantics must fail explicitly.

The compiler must not:

- guess;
- silently downgrade;
- silently broaden permissions;
- silently accept unsupported features;
- silently choose among ambiguous type constraints;
- treat unknown registry entries as permissive;
- continue after a semantic verification failure as though the program were valid.

## 5.5 Previous functionality remains qualified

Every milestone must preserve all already-qualified behavior unless an explicit compatibility decision says otherwise.

Every later change must run the accumulated regression suite.

## 5.6 No abandoned implementation paths

A superseded implementation must be:

- removed;
- disabled and quarantined with an explicit migration record; or
- retained only as a test oracle/reference implementation.

Dead code must not remain indefinitely as an accidental second implementation.

## 5.7 No placeholder production behavior

Stubs are permitted only where the milestone explicitly defines the component as unavailable and fail-closed.

A production claim cannot be made while required runtime behavior still consists of placeholders.

## 5.8 No test-shaped semantics

Tests validate the specification.

Tests must not become the reason a behavior exists.

If a test and normative specification disagree, the specification wins and the test must be corrected or the specification defect processed.

---

# 6. Semantic Linkage and Anti-Rot System

The earlier implementation draft proposed aggressive linkage using annotations. The finalized plan keeps that concept but makes it scalable.

## 6.1 Rule ownership

Compiler implementation units that make actual semantic decisions SHOULD declare the relevant rule ownership using machine-readable linkage metadata, for example:

```text
#[implements("TYPE-0010")]
```

The annotation is required on semantic decision points, not mechanically on every Rust function or every data structure.

The goal is traceability, not annotation noise.

## 6.2 Test witnesses

Conformance and semantic tests SHALL identify the rule(s) they witness:

```text
#[witnesses("TYPE-0010")]
```

or an equivalent machine-readable test manifest.

## 6.3 Linkage audit

A repository audit tool SHALL verify:

- every referenced rule exists;
- no removed rule remains referenced;
- no implementation-only semantic rule has no registered authority;
- required rules have implementation coverage status;
- required rules have witness coverage;
- tests reference valid rule IDs;
- implementation metadata is not stale;
- superseded implementations do not claim active rule ownership;
- rule dependencies are consistent with compiler dependencies.

## 6.4 Coverage categories

Rule coverage should distinguish:

```text
specified
implemented
unit-tested
conformance-tested
model-checked
differentially-tested
qualification-passed
certified
```

A rule being “implemented” must never be interpreted as “certified”.

---

# 7. Repository Topology

The repository must remain organized around stable responsibilities.

```text
omni/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── .gitignore
├── LICENSE
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CHANGELOG.md
│
├── spec/
│   ├── manifest/
│   ├── schemas/
│   ├── registry/
│   ├── grammar/
│   ├── models/
│   ├── data/
│   └── release/
│
├── compiler/
│   ├── omni-source/
│   ├── omni-lex/
│   ├── omni-syntax/
│   ├── omni-parse/
│   ├── omni-names/
│   ├── omni-types/
│   ├── omni-own/
│   ├── omni-effects/
│   ├── omni-hir/
│   ├── omni-mir/
│   ├── omni-verify/
│   ├── omni-machine/
│   ├── omni-codegen/
│   ├── omni-runtime/
│   ├── omni-driver/
│   └── omni-stage0/
│
├── tools/
│   ├── omni-canon/
│   ├── omni-audit/
│   ├── omni-registry/
│   ├── omni-conform/
│   ├── omni-bindgen/
│   └── ...
│
├── conformance/
├── tests/
├── docs/
├── scripts/
└── ci/
```

The exact crate topology may evolve, but dependency direction must remain explicit and acyclic.

---

# 8. Dependency Graph Rule

A compiler layer may depend only on layers below it unless a deliberate interface abstraction creates a permitted cycle-free dependency.

Initial conceptual direction:

```text
driver
  ↓
frontend
  ↓
syntax
  ↓
names
  ↓
types
  ↓
ownership/effects
  ↓
HIR
  ↓
MIR
  ↓
verification
  ↓
machine/codegen
  ↓
runtime/platform
```

Infrastructure flows into these layers:

```text
spec data
diagnostics
source/span infrastructure
interning
serialization
rule registry
```

must be factored so they do not cause semantic cycles.

Any dependency cycle is a design defect unless it is a purely data-model recursion represented without a build dependency cycle.

---

# 9. Global Build and Health Gate

Every milestone must pass all applicable gates.

## 9.1 Formatting

- canonical `rustfmt`;
- no formatting drift;
- deterministic generated source;
- canonical newline handling.

## 9.2 Static analysis

- Clippy at the pinned toolchain;
- denied warnings for release qualification;
- explicit unsafe audit where applicable;
- dependency/license/security checks.

## 9.3 Unit tests

All changed units must have or update focused tests.

## 9.4 Integration tests

Changed components must be exercised through their actual integration paths.

## 9.5 Conformance tests

Affected normative rule IDs must have conformance coverage.

## 9.6 Differential tests

Where another implementation exists, compare outputs/observations.

## 9.7 Reference-machine checks

Semantic changes must be checked against the reference abstract machine whenever applicable.

## 9.8 Artifact reproducibility

Affected deterministic artifacts must reproduce.

## 9.9 Dead-code audit

No newly superseded or unreachable production implementation may remain without a documented reason.

## 9.10 Repository cleanliness

Before every milestone gate:

```text
working tree clean
generated files synchronized
no stale build metadata
no accidental debug artifacts
no temporary test fixtures
no untracked semantic inputs
```

---

# 10. Three-Stage Generational Implementation Roadmap

This section supersedes the earlier pre-three-stage roadmap and establishes the final generational version boundaries.

The implementation lifecycle is now divided into exactly three engineering stages. The version ranges are part of the planning contract.

## 10.1 Stage boundaries

| Stage | Version interval | Implementation language | Primary purpose | Exit condition |
|---|---|---|---|---|
| Stage 1 | `0.0.0.1` → `1.0.0.0` | Rust only | Build the first minimal, correct, executable Omni implementation | Minimal Stage-0 language works end-to-end and is independently qualified |
| Stage 2 | `1.0.0.1` → `2.0.0.0` | Rust only | Complete the production Omni compiler/toolchain in Rust | Edition 1 is fully implemented and production-qualified in Rust |
| Stage 3 | `2.0.0.1` → `3.0.0.0` | Omni for the compiler, with Stage-2 Rust compiler as bootstrap host at first | Rebuild the compiler and language toolchain in Omni, then sever external language/compiler dependencies and complete DDC | Certified self-hosting Omni compiler and independent toolchain |

The boundaries are cumulative:

```text
Stage 1: prove that Omni can exist as a minimal executable language implementation.
        ↓
Stage 2: prove that the complete Edition 1 language can be produced in Rust.
        ↓
Stage 3: prove that Omni can reproduce its own compiler and eventually no longer
         depends on Rust or another programming language for ordinary operation.
```

A later stage cannot be declared complete merely because its headline implementation exists. Its complete qualification gates must pass.

## 10.2 Version range interpretation

The four components remain exactly:

```text
Release . Major . Minor . Patch
R       . M     . m     . P
```

Within each generational stage:

- the first component identifies the current release generation;
- the second component identifies a large implementation generation within that release family;
- the third component identifies a completed subsystem or milestone;
- the fourth component identifies the atomic engineering patch.

The three stage gates intentionally align with the first component:

```text
Stage 1 begins in Release 0 and ends at 1.0.0.0.
Stage 2 begins immediately after 1.0.0.0 and ends at 2.0.0.0.
Stage 3 begins immediately after 2.0.0.0 and ends at 3.0.0.0.
```

`1.0.0.0`, `2.0.0.0`, and `3.0.0.0` are therefore **generational qualification gates**, not arbitrary marketing releases.

## 10.3 Version allocation law

A new patch version may be issued only for one atomic coherent engineering change.

A minor version advances only after the associated subsystem is integrated and qualified.

A major version advances only at a deliberate implementation-generation boundary.

A release-generation gate advances only after the stage exit criteria are satisfied.

No stage may consume the next stage's version namespace merely for convenience.

---

# 11. Stage 1 — Rust Minimal Core (`0.0.0.1` → `1.0.0.0`)

## 11.1 Objective

Stage 1 is the first complete engineering construction of Omni. Everything required for the minimal language compiler is implemented in Rust.

The purpose is **not** to implement every Edition 1 feature immediately. The purpose is to establish a mathematically constrained vertical slice that can lex, parse, analyze, lower, verify, interpret, and execute a defined Stage-0 subset.

Stage 1 must establish all infrastructure later stages depend upon:

- specification ingestion;
- canonicalization;
- diagnostics;
- source management;
- lexer/token model;
- CST/AST;
- name resolution;
- type checking sufficient for Stage-0;
- ownership/lifetime checking sufficient for Stage-0;
- HIR/MIR;
- MIR verification;
- reference abstract machine;
- deterministic native lowering;
- minimal runtime;
- reproducible artifact/provenance generation;
- conformance and fuzzing infrastructure.

## 11.2 Stage-1 trust rule

Rust is an implementation mechanism only.

The Rust implementation is never allowed to define Omni semantics by convention, compiler accident, backend behavior, or host-platform behavior.

Every observable semantic choice must be traceable to an authoritative specification rule, machine-readable model, derived implementation contract, or formally admitted qualification oracle.

## 11.3 Stage-1 version ladder

### `0.0.0.1` — Repository blank page

Establish the repository as an intentionally empty engineering system.

Acceptance:

- expected top-level directories exist;
- no compiler implementation exists;
- no generated artifacts are committed;
- Git metadata is initialized cleanly;
- repository policy documents are present where required.

### `0.0.0.2` — Toolchain identity

Define the Rust bootstrap environment.

Required:

- exact stable Rust release selected at implementation kickoff;
- `rust-toolchain.toml`;
- required components;
- required target triples;
- cryptographic/toolchain release identity;
- build provenance capture.

The implementation plan intentionally does not freeze a future Rust patch release before implementation kickoff. Once selected, the toolchain becomes immutable for the applicable qualification generation.

### `0.0.0.3` — Quality and CI boundary

Establish mandatory CI enforcement:

| Job class | Fatal ceiling |
|---|---:|
| Pull request | 20 minutes |
| Main branch | 30 minutes |
| Nightly | 60 minutes |
| Release qualification | 120 minutes |

Fuzz budgets:

| Execution context | Per-target budget |
|---|---:|
| Pull request | 60 seconds |
| Main branch | 5 minutes |
| Nightly | 15 minutes |

Timeouts, hangs, crashes, sanitizer failures, nondeterministic outputs, and semantic mismatches are failures.

### `0.0.0.4` — Rule registry schema

Create the machine-readable semantic rule registry with strict lifecycle states:

```text
Proposed
Candidate
Ratified
Deprecated
```

Rules:

- `Proposed` rules may exist but cannot be implementation-owned by release-qualified code;
- `Candidate` rules may participate only in explicitly provisional paths;
- `Ratified` rules are eligible for ordinary release-qualified implementation ownership;
- `Deprecated` rules cannot receive new implementation ownership;
- a `Ratified` rule requires at least one resolvable witness test;
- stale linkage to removed, renamed, or deprecated rules fails the CI Reaper.

### `0.0.0.5` — Specification manifest

Create the root manifest that identifies the normative specification tree and its machine-readable inputs.

The implementation must read the manifest rather than infer normative input by arbitrary directory scanning.

The manifest must identify:

- edition;
- implementation profile;
- rule registry;
- schemas;
- grammar data;
- semantic models;
- feature predicates;
- canonical tree digest;
- provenance metadata.

### `0.0.0.6` — Canonicalization engine

Build `omni-canon`.

The canonicalization contract must include:

- canonical encoding;
- deterministic object-member ordering;
- deterministic number representation where applicable;
- explicit treatment of null/absence;
- removal of all semantic-irrelevant spans and source coordinates from semantic hashes;
- canonical line endings;
- host-path virtualization to `/omni-root/`;
- rejection of unresolved host-specific path forms;
- no dependence on current working directory;
- no dependence on locale;
- no dependence on process environment unless explicitly supplied as an input.

The semantic hash must remain unchanged when only comments, whitespace, line/column positions, or absolute checkout location changes and those items are outside the hashed semantic domain.

### `0.0.0.7` — CI Reaper and semantic linkage

Build `omni-audit`.

It must validate:

- rule identifiers exist;
- dependency graphs are acyclic where required;
- implementation tags reference valid lifecycle states;
- tests reference valid rules;
- witness obligations resolve;
- required rules have implementation coverage status;
- required rules have witness coverage;
- deprecated/removed linkages are absent;
- semantic code does not silently escape rule ownership;
- the registry and implementation metadata agree.

### `0.0.0.8` — Conformance and diagnostic schemas

Define machine-readable formats for:

- compiler diagnostics;
- rule witnesses;
- conformance outcomes;
- verification failures;
- provenance records;
- regression-case metadata;
- fuzz failure promotion records.

### `0.0.0.9` — Compiler crate topology

Establish the Rust crate DAG without implementing language behavior.

Conceptual dependency direction:

```text
omni-driver
  ↓
omni-source / omni-lex / omni-syntax / omni-parse
  ↓
omni-names
  ↓
omni-types
  ↓
omni-own / omni-effects
  ↓
omni-hir
  ↓
omni-mir
  ↓
omni-verify
  ↓
omni-machine / omni-codegen
  ↓
omni-runtime
```

Infrastructure crates must be factored to avoid semantic cycles.

### `0.0.0.10` — Specification model loading

Define typed loaders for the machine-readable specification artifacts.

Required properties:

- schema validation before semantic use;
- canonicalized digest verification;
- deterministic load ordering;
- duplicate detection;
- unknown-field rejection where schemas require strictness;
- provenance capture.

### `0.0.0.11` — Stage-0 feature predicate system

Create the feature/profile mechanism consumed by the parser and semantic pipeline.

Stage-0 must be a strict syntactic/semantic subset of the Edition 1 language, not an independent grammar with duplicated semantics.

### `0.0.0.12` — Artifact identity and provenance

Define the artifact header and build metadata binding:

- compiler identity;
- implementation version;
- source revision;
- canonical `spec/` tree SHA-256;
- toolchain identity;
- target triple;
- code-generation configuration;
- reproducibility inputs.

### `0.0.0.13` — Foundation qualification gate

Stage 1 foundation may advance only when:

- schemas validate;
- rule registry resolves;
- linkage auditing passes;
- dependency graph is acyclic;
- canonicalization is deterministic;
- host-path virtualization is deterministic;
- provenance is generated correctly;
- CI boundaries are enforced;
- no compiler semantics have been invented merely for convenience.

---

# 12. Stage 1 Language Construction

## 12.1 Source and lexer generation — `0.0.1.x`

Implement:

1. byte cursor and UTF-8 decoder;
2. source normalization;
3. BOM handling only at offset zero;
4. canonical line-ending normalization;
5. byte-accurate source spans;
6. token kinds;
7. trivia preservation;
8. identifiers and keyword recognition;
9. Unicode identifier data ingestion;
10. identifier security/confusable validation where required by the specification;
11. numeric literal state machines;
12. character and string literals;
13. raw strings and any specified interpolated forms;
14. operators and punctuation;
15. maximal-munch behavior;
16. context-sensitive generic punctuation splitting without destroying source identity;
17. lexical diagnostics;
18. deterministic EOF behavior.

The lexer qualification suite must include:

- specification examples;
- malformed UTF-8;
- arbitrary byte sequences;
- large tokens;
- long trivia runs;
- nested raw-string delimiters where applicable;
- confusable identifiers;
- adversarial numeric literals;
- differential replay against a reference lexer where available.

The target remains **zero compiler crashes** for bounded fuzz campaigns.

## 12.2 CST and parser generation — `0.0.2.x`

Implement:

- lossless CST;
- grammar productions driven from normative grammar data;
- deterministic recursive-descent declaration parsing;
- Pratt-style expression parsing where appropriate;
- pattern grammar;
- generic syntax;
- error recovery;
- syntax diagnostics;
- AST lowering.

Parser invariants:

- identical input produces identical CST structure;
- comments and whitespace remain recoverable from the CST;
- error recovery never mutates the authoritative source;
- ambiguity is resolved deterministically;
- Stage-0 unsupported features are rejected as profile violations rather than being silently reinterpreted as unrelated syntax errors when the specification distinguishes the cases.

## 12.3 Names and minimal type core — `0.0.3.x`

Implement:

- package and module identities;
- lexical scopes;
- `DefId` allocation;
- imports;
- shadowing rules;
- visibility checks;
- primitive types;
- tuples/arrays/slices where applicable;
- structs/enums;
- function types;
- type variables;
- interning;
- unification;
- expected-type checking;
- inference diagnostics;
- explicit coercions only where specified.

Qualification must include termination tests for recursive constraints and rejection tests for unconstrained or ambiguous type variables.

## 12.4 Ownership, lifetime, effects, and capabilities — `0.0.4.x`

Implement the minimal Stage-0 subset necessary for sound MIR construction, while architecting the interfaces for the complete Edition 1 model.

The implementation must preserve the established semantic closure:

- types determine what ownership relations are possible;
- ownership determines valid lifetime relationships;
- effects classify computational behavior;
- effects do not grant capability authority;
- capabilities remain independently governed by the capability environment;
- capability-bearing values cannot escape through disallowed static, detached, persistent, or distributed channels;
- continuation types inherit the relevant lifetime constraints;
- higher-ranked effect binders are required where local lifetimes would otherwise escape their scope.

## 12.5 HIR — `0.0.5.x` early portion

Lower syntax-oriented structure into a canonical semantic representation.

The HIR must eliminate merely syntactic distinctions that are no longer semantically useful while preserving all information needed for diagnostics and provenance.

## 12.6 MIR — `0.0.5.x` later portion

Define the canonical MIR model:

- basic blocks;
- places;
- projections;
- operands;
- rvalues;
- statements;
- terminators;
- storage state;
- initialization state;
- drops;
- calls;
- control flow;
- effect/capability transitions where required;
- assumption dependency tokens.

Unsafe-assumption operations must include:

```text
assume.create
assume.use
assume.invalidate
```

Each assumption records sufficient dependency information to reject stale uses after invalidation.

## 12.7 MIR verifier — `0.0.5.x`

The verifier must validate the IR independently of the producer.

At minimum:

- CFG integrity;
- block reachability invariants;
- initialization state;
- ownership/move legality;
- drop legality;
- assumption lifetime/dominance;
- effect constraints;
- capability constraints;
- representation invariants;
- impossible state rejection;
- verifier determinism.

No later backend is allowed to make an invalid MIR program executable.

---

# 13. Stage 1 Reference Execution and Native Vertical Slice

## 13.1 Reference abstract machine — `0.0.6.x`

Implement the semantic oracle independently of native code generation.

Track explicitly:

- allocation identity;
- byte-level memory;
- pointer provenance;
- exposed addresses where specified;
- initialization state;
- lifetime state;
- synchronization/happens-before information;
- panic/abort/cancellation states;
- foreign execution boundaries where applicable.

The reference machine is an executable semantic model, not merely a debugging interpreter.

## 13.2 Low-level lowering — `0.0.7.x`

Define a minimal target-neutral low-level representation and lower verified MIR into it.

The representation must preserve enough information for:

- calling convention assignment;
- stack layout;
- register allocation strategy;
- memory accesses;
- branch structure;
- symbol references;
- relocations;
- runtime entry points.

## 13.3 Stage-0 backend — `0.0.7.x`

Use the selected backend technology for the minimal executable path.

Cranelift may serve as the first implementation backend because it provides an explicit deterministic compiler path suitable for the early executable slice. Its behavior remains an implementation detail; it is never the semantic oracle.

## 13.4 Minimal runtime — `0.0.8.x`

Provide only the runtime facilities required by Stage-0:

- process startup;
- exit status;
- minimal I/O;
- required memory allocator interface;
- panic/abort path as defined for Stage-0;
- required platform glue.

No unnecessary runtime services should enter the Stage-0 trusted computing base.

## 13.5 Standard core subset — `0.0.9.x`

Implement only the library surface needed to exercise the Stage-0 language and compiler infrastructure.

Library APIs must be developed from normative contracts, not from whatever is most convenient for the Rust implementation.

## 13.6 Stage-0 differential gate — `0.0.10.x`

For every supported Stage-0 semantic case where comparison is meaningful:

```text
Omni source
   ↓
parser / semantic analysis
   ↓
verified MIR
   ├──────────────→ reference abstract machine
   └──────────────→ native backend
```

Observable outcomes must agree within the explicitly declared equivalence relation.

No native binary is considered correct merely because it executes successfully.

## 13.7 Stage-0 qualification — `0.0.11.x` → `0.0.12.x`

Expand the Stage-0 corpus to cover:

- valid programs;
- invalid programs;
- ownership failures;
- type failures;
- boundary values;
- arithmetic checks;
- initialization/drop paths;
- panic and abort behavior;
- provenance behavior;
- deterministic diagnostics;
- cross-host canonicalization.

## 13.8 Stage-1 release gate — `1.0.0.0`

`1.0.0.0` is achieved only when the minimal Rust implementation is:

- executable;
- deterministic;
- reproducible;
- specification-linked;
- reference-machine checked;
- fuzz-qualified;
- provenance-bound;
- release-audited;
- free of unresolved critical correctness defects.

`1.0.0.0` does **not** claim the full Edition 1 feature set. It claims the first qualified minimal implementation generation.

---

# 14. Stage 2 — Complete Production Platform in Rust (`1.0.0.1` → `2.0.0.0`)

## 14.1 Objective

Stage 2 keeps the entire production implementation in Rust. Every compiler component, compiler tool, reference implementation, runtime component required for production, package/build component, and qualification harness that belongs to the production toolchain is implemented in Rust unless the frozen specification explicitly requires a non-Rust external system boundary.

The objective is to transform the minimal Stage-1 compiler into a complete Edition 1 implementation suitable for production use, including the full compiler pipeline, runtime, package/build system, interoperation facilities, developer tooling, optimization, and distributed semantics defined by the specification.

Rust remains the implementation language throughout this stage.

No part of Stage 2 may be implemented in Omni merely because the self-hosted rewrite is planned later. Stage 2 is the deliberate **all-Rust production engineering generation**.

## 14.2 Stage-2 entry gate

`1.0.0.1` can begin only after `1.0.0.0` is sealed.

Entry evidence must include:

- Stage-0 release artifacts;
- complete provenance;
- passing foundation CI;
- working reference machine;
- reproducible native vertical slice;
- archived test corpus;
- specification digest.

## 14.3 Complete type and generic system — `1.0.1.x`

Implement the complete Edition 1 type system:

- generic parameters;
- inference;
- trait constraints;
- associated types/items;
- specialization rules if specified;
- refinement constraints;
- algebraic data types;
- exhaustive matching;
- higher-ranked relationships;
- dynamic/runtime type identity where specified;
- runtime fingerprints including the mandated generic/capability components;
- lifetime-bearing types excluded from runtime identities where the specification requires non-static exclusion.

The solver must be terminating under its declared limits and deterministic in output and diagnostics.

## 14.4 Ownership and complete memory model — `1.0.2.x`

Implement:

- full ownership analysis;
- complete lifetime reasoning;
- field-sensitive projections;
- linear/affine values;
- generational references;
- allocation identity;
- provenance;
- address exposure/reconstruction rules;
- atomic memory ordering;
- happens-before;
- modification order;
- sequential consistency rules;
- memory-model diagnostics and proof witnesses.

Any optimization or backend transformation must preserve these properties.

## 14.5 Effects and capability authority — `1.0.3.x`

Implement the full algebraic effect system.

Requirements:

- effect rows;
- effect inference;
- public effect bounds;
- effect handlers;
- one-shot continuation semantics;
- continuation lifetime constraints;
- multi-shot rules where permitted;
- rejection of affine/linear/mutable-borrow capability capture by multi-shot continuations;
- capability authority environment `K` kept distinct from effect rows;
- closure capture environment `C` kept distinct from effect rows;
- provider-side authorization at operation commit;
- revocation semantics;
- capability escape restrictions.

## 14.6 Structured concurrency and async cleanup — `1.0.4.x`

Implement:

- task trees;
- structured spawning;
- join semantics;
- cancellation;
- cancellation masking during cleanup;
- async drop;
- panic/cancellation separation;
- sibling-failure cancellation;
- cleanup fault aggregation/suppression as specified;
- happens-before edges for task and channel operations.

Qualification must include adversarial cases for:

- parent cancellation;
- sibling failure;
- cleanup failure during cancellation;
- active panic plus cleanup fault;
- nested cancellation;
- cancellation masking violations;
- double-drop/double-cleanup hazards.

## 14.7 Unsafe assumptions and optimization legality — `1.0.5.x`

Implement the complete `OMNI-UNSAFE` → `OMNI-OPT` dependency system.

Assumptions must record:

- origin;
- dependency tokens;
- affected operands;
- scope;
- dominance;
- lifetime;
- invalidation events;
- verification/proof state.

The optimizer must reject stale assumptions rather than treating unsafe annotations as global compiler permission.

Dead-code elimination may remove non-observable computations, but must preserve:

- volatile/device observations;
- mandated checks;
- faults;
- externally visible side effects;
- provenance-sensitive behavior.

## 14.8 Backend and translation validation — `1.0.6.x`

Introduce the high-performance production backend.

LLVM may be introduced here and, once introduced, its exact source/release/toolchain identity becomes part of the build provenance and relevant TCB.

The implementation must define:

- exact LLVM version/source identity;
- backend build method;
- target configuration;
- optimization pipeline;
- pass configuration;
- deterministic controls;
- translation-validation contract;
- backend artifact provenance.

LLVM is not permitted in the earlier Stage-1 foundation merely because it is useful later.

## 14.9 Hermetic build and comptime — `1.0.7.x`

Implement:

- hermetic compile-time execution;
- explicit environment allowlists;
- dynamic dependency tracing;
- cache keys containing discovered dependencies;
- canonical AST outputs;
- `SOURCE_DATE_EPOCH` or the normative reproducibility mechanism;
- no ambient host state;
- deterministic resource/fuel limits;
- reproducible package/build metadata.

## 14.10 Package, update, and supply-chain system — `1.0.8.x`

Implement:

- snapshot-bound resolution;
- package identity;
- namespace rules;
- lockfiles;
- secure repository metadata;
- revocation;
- yanked vs security-revoked semantics;
- deterministic dependency selection;
- provenance and SBOM generation;
- offline/recovery workflows.

A security-revoked artifact must not become reachable merely because an old lockfile still names it.

## 14.11 Standard library completion — `1.0.9.x`

Complete the production standard library by clearly separating:

- freestanding core;
- allocation layer;
- operating-system capability layer;
- networking/filesystem/time/entropy interfaces;
- concurrency primitives;
- data structures;
- compiler support libraries.

The standard library itself must obey the capability model.

## 14.12 ABI and Fearless FFI — `1.0.10.x`

Implement the common ABI and foreign-function boundaries.

The FFI transition model must explicitly represent:

```text
Omni execution
      ↓
Executing_Foreign
      ↓
Foreign execution
      ↓
Synchronization barrier
      ↓
Omni execution
```

Foreign exceptions, `longjmp`, SEH, and equivalent foreign control-flow escapes must stop at the defined isolation boundary.

Managed memory crossing an FFI boundary must remain pinned according to the authoritative rule.

## 14.13 Tooling — `1.0.11.x`

Complete:

- formatter;
- language server;
- source navigation;
- semantic highlighting;
- inlay information;
- borrow/effect/capability diagnostics;
- machine-readable compiler diagnostics;
- AI compiler API exposing rule violations, proof obligations, and classified legal repair surfaces without bypassing verification.

## 14.14 Canonical wire and distributed execution — `1.0.12.x`

Implement the complete distributed-state contracts.

The wire format must be canonical and deterministic.

Distributed continuation state is a logical checkpoint, not a raw serialized native stack.

The serialization barrier must reject:

- raw pointers;
- local borrows;
- local OS capabilities;
- non-portable host state;
- other values forbidden by the specification.

Hydration must reconstruct fresh local state and re-authorize local capabilities.

Duplicate resume/state replay protections must be explicit.

## 14.15 Production performance and validation — `1.0.13.x`

Establish:

- benchmark corpus;
- compile-time budgets;
- runtime performance budgets;
- memory-use ceilings where specified;
- backend optimization baselines;
- regression thresholds;
- clean-room reproducibility;
- multi-host builds;
- differential testing against the reference machine;
- backend translation validation.

Performance improvements may not bypass semantic proof or qualification.

## 14.16 Stage-2 hardening — `1.0.14.x`

Run the complete adversarial corpus across:

- type system;
- ownership;
- effects;
- capabilities;
- concurrency;
- async cleanup;
- unsafe assumptions;
- optimization legality;
- build reproducibility;
- packages/security;
- FFI;
- distributed state.

Every discovered issue becomes either:

- a fixed implementation defect with a regression witness; or
- a controlled specification defect with authoritative disposition.

## 14.17 Stage-2 release gate — `2.0.0.0`

`2.0.0.0` means:

> **Complete Edition 1 implementation, fully production-qualified, still implemented in Rust.**

Required evidence includes:

- all normative rules accounted for;
- implementation/witness linkage complete;
- complete conformance suite passing;
- reference-machine agreement passing where applicable;
- native backend validation passing;
- reproducibility passing;
- package/supply-chain checks passing;
- FFI containment passing;
- distributed execution qualification passing;
- security review passing;
- release artifacts reproducible;
- Stage-3 bootstrap inputs archived.

`2.0.0.0` is the authoritative Rust production baseline from which self-hosting is allowed to begin.

---

# 15. Stage 3 — Omni-Owned Compiler Construction (`2.0.0.1` onward)

## 15.1 Objective

Stage 3 is divided into two inseparable sub-generations:

1. **Omni compiler construction:** build Omni's own compiler, tokenizer, parser, interpreter/reference machine, semantic analyzer, MIR verifier, standard compiler libraries, backend/tooling, and build system in Omni source.
2. **Independence/severance:** prove that the Omni implementation can sustain itself without Rust or another programming language as an ordinary compilation dependency.

The first part is **self-reimplementation**.

The second part is **self-hosting independence**.

The distinction is mandatory.

## 15.2 Stage-3 bootstrap rule

At the beginning of Stage 3, the Rust production compiler is still the bootstrap host.

This is intentional.

The first Omni-written compiler may be compiled by the already-qualified Rust compiler. This does not constitute final independence.

The transition is:

```text
Rust production compiler
       ↓
compiles Omni-written compiler source
       ↓
Omni compiler binary A
       ↓
Omni compiler binary A compiles Omni compiler source
       ↓
Omni compiler binary B
       ↓
DDC / independent verification
       ↓
Rust removed from ordinary self-hosted toolchain operation
```

## 15.3 `2.0.0.1` — Omni compiler foundation

Implement in Omni:

- source/span model;
- token model;
- syntax tree model;
- symbol interning;
- compiler arenas;
- diagnostics;
- canonical serialization interfaces;
- rule-linkage metadata;
- compiler configuration model.

These modules must not depend semantically on Rust-specific concepts.

## 15.4 `2.0.1.x` — Omni tokenizer and source subsystem

Rewrite the complete source and lexer pipeline in Omni.

It must reproduce the Rust implementation's qualified behavior for:

- UTF-8;
- Unicode data;
- source normalization;
- tokenization;
- trivia;
- spans;
- diagnostics;
- malformed-input handling.

The Omni tokenizer is not permitted to inherit hidden semantics from the Rust implementation merely because the Rust compiler is used as the bootstrap host.

## 15.5 `2.0.2.x` — Omni parser, CST, and AST

Implement:

- lossless CST;
- declaration parser;
- Pratt expression parser where required;
- pattern parser;
- error recovery;
- AST lowering;
- deterministic parse diagnostics.

Cross-compiler equivalence must compare normalized semantic artifacts rather than requiring implementation-identical internal data structures.

## 15.6 `2.0.3.x` — Omni names and semantic analyzer

Implement in Omni:

- package/module resolver;
- scope graph;
- definition IDs;
- visibility;
- imports;
- type interning;
- unification;
- trait solver;
- inference;
- refinement constraints;
- exhaustiveness checking;
- semantic diagnostics.

## 15.7 `2.0.4.x` — Omni ownership, lifetime, effects, and capabilities

Reimplement the complete semantic engines in Omni.

The implementation must preserve all previously qualified closure rules, especially:

- continuation/lifetime intersection;
- no escaping higher-ranked local lifetime dependencies;
- independent effect and capability authority;
- multi-shot continuation restrictions;
- capability re-authorization;
- revocation semantics.

## 15.8 `2.0.5.x` — Omni HIR/MIR/verifier

Implement in Omni:

- HIR;
- MIR;
- drop elaboration;
- initialization dataflow;
- assumption tokens;
- assumption invalidation;
- MIR verification;
- semantic hash generation;
- verifier diagnostics.

The verifier must be independently executable from the optimizer and backend.

## 15.9 `2.0.6.x` — Omni reference abstract machine/interpreter

Implement the reference machine in Omni.

It must reproduce the authoritative semantic observations for:

- arithmetic;
- memory;
- ownership;
- initialization/drop;
- provenance;
- effects;
- capabilities;
- synchronization;
- panic/abort;
- cancellation;
- foreign boundaries;
- distributed checkpoint constraints where represented by the machine model.

This interpreter becomes an important independent oracle for the self-hosted compiler.

## 15.10 `2.0.7.x` — Omni backend and executable pipeline

Implement the compiler backend in Omni.

The long-term target is a native Omni toolchain that does not require another programming language to express the compiler's ordinary logic.

Backend integration may temporarily bind to externally implemented libraries when expressly permitted by the Stage-3 migration plan, but every such dependency must be classified explicitly and must have a defined final-severance disposition:

- bootstrap-only;
- build-time temporary;
- runtime permanent and permitted;
- or forbidden at final severance.

The final classification must be explicit.

## 15.11 `2.0.8.x` — Omni standard library and build system

Implement in Omni the core services needed by the compiler itself:

- strings and collections;
- file system abstractions;
- process execution where permitted;
- compiler data structures;
- allocator interfaces;
- package metadata;
- canonical serialization;
- deterministic filesystem discovery;
- build graph resolution;
- artifact packaging.

The compiler must no longer rely on Rust helper programs for ordinary compilation behavior.

## 15.12 `2.0.9.x` — Omni development tooling

Rebuild in Omni the components that are essential to the language's own development loop:

- formatter core;
- diagnostics renderer;
- conformance runner;
- semantic query layer;
- package/build driver;
- compiler test harness;
- machine-readable diagnostics producer.

Non-essential GUI/editor integrations may remain external if the independence definition explicitly permits them, but the compiler itself must not depend on them.

## 15.13 `2.0.10.x` — Complete Omni compiler integration

Connect the Omni-written frontend, semantic engine, MIR, verifier, reference machine, backend, runtime interfaces, build driver, and qualification harness into one coherent toolchain.

Required properties:

- no duplicate semantic implementation paths;
- one authoritative Omni-written pipeline;
- complete rule linkage;
- complete test-witness linkage;
- deterministic compiler-driver behavior;
- deterministic artifact identity;
- clean rebuild from a fresh checkout using the Stage-2 bootstrap compiler.

## 15.14 `2.0.11.x` — Omni compiler conformance parity

Run the full Edition 1 conformance corpus through both the Rust production compiler and the Omni-written compiler.

Parity must be established at each applicable layer:

- token stream;
- syntax/CST semantics;
- name resolution;
- type/effect/capability conclusions;
- ownership/lifetime conclusions;
- MIR validity;
- reference-machine observations;
- native executable observations;
- diagnostics and rule IDs.

Any intentional representation difference must be covered by a declared equivalence relation.

## 15.15 `2.0.12.x` — Bootstrap dependency elimination

Remove temporary compiler-language dependencies one class at a time.

Track every external component in the Stage-3 dependency ledger with:

- identity;
- purpose;
- transitive dependencies;
- bootstrap-only/temporary/permanent classification;
- removal criterion;
- proof that its removal does not change semantics.

The target is an Omni-written ordinary compiler path that no longer invokes Rust helper binaries, Rust-generated source, or Rust-specific compiler services.

## 15.16 `2.0.13.x` — Self-compilation

The Omni-written compiler must compile its own complete compiler source tree.

The self-compilation procedure must be scripted as an explicit, reproducible build graph and must record all inputs.

Success requires at least two successive self-builds to remain within the declared artifact-equivalence relation.

## 15.17 `2.0.14.x` — Independent rebuild

Perform independent clean builds using separately provisioned environments.

The environments must vary the declared trust-base dimensions while holding the normative inputs fixed.

Compare:

- compiler artifacts;
- canonical metadata;
- semantic outputs;
- executable behavior;
- provenance records.

## 15.18 `2.0.15.x` — Final self-hosting hardening

Run the complete adversarial qualification battery against the Omni-written compiler:

- parser fuzzing;
- semantic-failure corpus;
- ownership adversarial corpus;
- effects/capabilities corpus;
- concurrency/cancellation corpus;
- unsafe-assumption corpus;
- optimization validation;
- package/update security;
- FFI containment;
- distributed checkpoint/hydration;
- reproducibility;
- bootstrap contamination;
- dependency-closure audit.

No critical or high-severity unresolved issue may remain at the final gate.

---

# 15.19 Stage-3 release gate — `3.0.0.0`

`3.0.0.0` is issued only after all of the following are simultaneously true:

- the compiler source is written in Omni;
- the compiler compiles itself;
- the Omni-written compiler passes the complete Edition 1 conformance suite;
- the reference-machine and native execution paths agree where applicable;
- DDC passes;
- all forbidden Rust/compiler dependencies are removed from the ordinary toolchain;
- the final transitive TCB is inventoried and hashed;
- reproducible artifacts are produced by independent clean environments;
- the bootstrap recovery kit is archived separately;
- no unresolved critical trust, correctness, security, or specification-compliance defect remains.

`3.0.0.0` is therefore the first **self-hosted and independently qualified Omni implementation generation**, not merely the first compiler written in Omni.

---

# 16. Stage 3 Dual-Compiler and Bootstrap Transition

## 16.1 Rust compiles Omni compiler

The qualified `2.0.0.0` Rust compiler compiles the complete Omni-written compiler.

Produce:

- compiler A;
- its complete provenance;
- its exact source/spec digest;
- its target information;
- its runtime dependencies;
- its reproducibility evidence.

## 16.2 Omni compiler self-compilation

Compiler A must compile the same Omni compiler source and produce compiler B.

The self-build must succeed without using Rust code as an input to the Omni compiler's ordinary semantic pipeline.

## 16.3 Semantic convergence gate

Compare A and B using the strongest applicable equivalence:

1. byte-for-byte identity where fully deterministic binary identity is required;
2. canonical object identity where link layout is intentionally excluded;
3. semantic MIR/object equivalence where the specification explicitly permits layout differences.

The equivalence relation must be declared before qualification and may not be weakened after a mismatch is observed.

## 16.4 Bootstrap contamination checks

Search the Stage-3 compiler and generated artifacts for forbidden dependencies on:

- Rust source crates;
- Rust standard library APIs;
- Rust compiler plugins;
- host-language compiler services;
- undeclared build scripts;
- hidden generator binaries;
- unrecorded code-generation utilities.

Every remaining external component must have a written classification and exit/removal plan.

---

# 17. Stage 3 Independence and Severance

## 17.1 Independence definition

For `3.0.0.0`, “self-hosted” means all of the following:

- Omni compiler source is written in Omni;
- the Omni compiler can compile itself;
- ordinary compiler operation does not require Rust source or the Rust compiler;
- no other programming language is required to express or execute the compiler's core semantics;
- required runtime support is supplied by Omni's own specified runtime/core facilities or explicitly permitted low-level ABI/system interfaces;
- bootstrap/recovery artifacts are separately archived and are not ordinary daily-build dependencies.

## 17.2 Bootstrap recovery vs ordinary independence

Rust may remain in a documented **bootstrap recovery kit** after severance.

That kit is not part of the ordinary Omni development dependency graph.

This distinction prevents the false claim that a compiler is independent simply because a Rust seed exists somewhere in archival infrastructure.

## 17.3 Dependency closure audit

The severance audit must traverse the complete transitive dependency graph.

Inspect:

- compiler executables;
- runtime libraries;
- build scripts;
- generated sources;
- code generators;
- package manager plugins;
- test harnesses used for ordinary qualification;
- host tools invoked implicitly.

Undeclared dependencies are release-blocking defects.

## 17.4 Freestanding capability

The final compiler environment must use Omni's own capability-aware platform interfaces rather than assuming unrestricted host authority.

Build operations, package access, filesystem access, and process execution must be explicitly authorized.

---

# 18. Diverse Double Compilation and Trust Qualification

## 18.1 DDC objective

DDC is used to establish confidence that the self-hosted compiler output is not dependent on hidden behavior injected by a compromised bootstrap compiler/toolchain.

## 18.2 Required diversity

The independent compilation path must differ in at least one meaningful trust-base dimension, such as:

- distinct compiler implementation/toolchain;
- distinct build host;
- distinct binary distribution provenance;
- independently produced bootstrap artifact.

The exact diversity set must be recorded in the TCB manifest.

## 18.3 DDC stages

At minimum:

```text
Independent environment I compiles Omni compiler → C1
Independent environment II compiles Omni compiler → C2
C1 compiles Omni compiler → C3
C2 compiles Omni compiler → C4
```

The release candidate must pass the declared convergence/equivalence checks among the appropriate outputs.

## 18.4 DDC mismatch rule

Any unexplained DDC mismatch blocks `3.0.0.0`.

A mismatch is never classified as harmless merely because the binaries still execute.

It must be reduced to an approved source of variation or fixed.

## 18.5 TCB inventory

The final release record must contain exact digests, roles, and trust classifications for:

- bootstrap compiler(s);
- assembler/object writer;
- linker/image builder;
- target/emulator/hardware;
- standard library/runtime;
- specification data;
- verification tools;
- package metadata used for the release;
- DDC comparison tools.

Transitive dependencies must be included.

---

# 19. Paradigm Coverage Strategy

Omni is intended to be unusually comprehensive in feature coverage while retaining one semantic core.

The implementation plan therefore treats major programming paradigms as **usage methodologies over a common language**, not as separate dialects.

## 19.1 Functional programming

Support should cover, where defined by the specification:

- immutable-by-default programming;
- pure functions/effect-free functions;
- algebraic data types;
- pattern matching;
- higher-order functions;
- closures;
- recursion;
- tail-call facilities where normatively required.

The MIR should make closure captures and ownership explicit.

## 19.2 Object-oriented methodology

Support should be expressed through:

- encapsulation;
- modules/visibility;
- traits/interfaces;
- composition;
- dynamic dispatch where specified;
- explicit object/runtime identity where specified.

Fragile inheritance hierarchies should not be introduced merely to advertise OOP support when the language's trait/composition system already expresses the intended methodology.

## 19.3 Data-oriented and systems programming

Support should include the facilities needed to control:

- representation/layout;
- alignment;
- allocation;
- data locality;
- SIMD/vector operations;
- arenas;
- generational references;
- direct memory interaction under the capability/provenance model.

## 19.4 Concurrent, actor, and reactive programming

Support should include:

- structured concurrency;
- asynchronous operations;
- channels/message passing;
- task trees;
- cancellation;
- state-machine lowering;
- synchronization verification.

## 19.5 Declarative and metaprogramming

Support should include where specified:

- `comptime`;
- macros;
- generated state machines;
- compile-time checking;
- source transformation under hygiene/provenance constraints.

## 19.6 Generic, logic, and constraint-oriented methodologies

The type/effect system should be capable of expressing:

- generic programming;
- trait/constraint programming;
- refinement-style constraints;
- compile-time proofs available within the language's formal system.

## 19.7 Paradigm neutrality rule

No paradigm-specific implementation may bypass:

- ownership;
- effects;
- capabilities;
- memory model;
- MIR verification;
- deterministic evaluation;
- provenance;
- build hermeticity.

This is how broad feature coverage remains one coherent language instead of becoming a collection of special-case subsystems.

---

# 20. Compiler Pipeline Contract

Every production compiler generation must expose the following conceptual pipeline even if implementation modules are rearranged:

```text
source bytes
  ↓
source normalization
  ↓
lexer/token stream
  ↓
lossless CST
  ↓
typed AST
  ↓
name resolution
  ↓
type/effect/capability analysis
  ↓
ownership/lifetime analysis
  ↓
HIR
  ↓
MIR
  ↓
MIR verification
  ↓
reference-machine validation
  ├──────────────→ interpreter/reference execution
  ↓
optimization legality
  ↓
low-level/backend lowering
  ↓
object/link/image generation
  ↓
runtime execution
```

Any implementation that bypasses a semantic gate must document an equivalent proof-preserving mechanism and obtain qualification approval.

---

# 21. Testing and Fuzzing Master Contract

## 21.1 Test layers

Every stage must maintain:

- schema tests;
- unit tests;
- property tests;
- parser/lexer golden tests;
- semantic conformance tests;
- adversarial ownership/effect/capability tests;
- MIR verifier tests;
- reference-machine tests;
- backend differential tests;
- reproducibility tests;
- integration tests;
- cross-host tests;
- regression tests;
- security tests;
- fuzz targets.

## 21.2 Fuzzing progression

The fixed CI budgets from Stage 1 remain the baseline contract:

```text
PR      60 seconds/target
Main    5 minutes/target
Nightly 15 minutes/target
```

Longer campaigns may exist outside required CI only when separately budgeted and recorded.

## 21.3 Fuzz failure promotion

Every minimized failure must become:

- a deterministic regression case;
- a preserved seed/input;
- a rule linkage where applicable;
- a documented expected outcome;
- a provenance record.

## 21.4 Differential testing

Whenever two independently implemented paths exist, compare them.

Examples:

- Rust lexer vs Omni lexer;
- Rust parser vs Omni parser;
- Rust semantic engine vs Omni semantic engine;
- Rust reference machine vs Omni reference machine;
- Rust backend vs Omni backend;
- Stage-1 compiler vs Stage-2 compiler;
- independent DDC toolchains.

---

# 22. Reproducibility and Canonicalization Contract

The reproducibility contract is independent of the programming language used to implement the compiler.

## 22.1 Semantic hash invariants

A semantic hash must not change solely because of:

- host checkout path;
- path separator style;
- source line endings where normalized;
- comments;
- whitespace outside semantic structure;
- source line/column metadata;
- non-semantic diagnostic spans.

A semantic hash must change whenever a semantically hashed input changes.

## 22.2 Host-path virtualization

Absolute host paths must be converted into a virtual namespace rooted at:

```text
/omni-root/
```

Normalization must operate structurally on path-bearing data rather than by relying only on raw string replacement.

## 22.3 Spec tree binding

Every deterministic compiler artifact must identify the canonical hash of the normative `spec/` tree used to create it.

The implementation must detect and reject mismatched specification data.

---

# 23. Repository Health and Integration Law

Every implementation patch must satisfy all applicable checks.

Before integration:

1. dependencies are satisfied;
2. implementation is coherent;
3. affected tests pass;
4. regression suite passes;
5. semantic linkage audit passes;
6. generated artifacts are synchronized;
7. obsolete implementations are removed or explicitly retained as oracles;
8. no forbidden dependency was introduced;
9. reproducibility remains valid;
10. repository is clean.

A change that passes its local tests but breaks another compiler layer is not qualified.

---

# 24. Documentation and Engineering Evidence

Every completed milestone must leave enough evidence for an independent engineer to reconstruct:

- what changed;
- why it changed;
- which specification rules were affected;
- which tests witness the behavior;
- which models/oracles were used;
- what external tools were used;
- what toolchain generated the artifacts;
- which assumptions were made;
- what remains intentionally incomplete.

No milestone may describe a provisional mechanism as final production behavior.

---

# 25. Stage Exit Criteria Summary

## 25.1 `1.0.0.0` — Minimal Rust Core

Must provide:

- minimal Stage-0 language subset;
- Rust compiler implementation;
- lexer/parser/name/type/ownership pipeline for that subset;
- HIR/MIR;
- independent MIR verifier;
- reference abstract machine;
- native executable path;
- deterministic diagnostics;
- build provenance;
- conformance/fuzz infrastructure.

## 25.2 `2.0.0.0` — Complete Rust Production

Must provide all applicable Edition 1 facilities:

- full type/generic system;
- ownership/lifetime/memory model;
- effects/capabilities;
- structured concurrency/async cleanup;
- unsafe assumption graph;
- production optimization and translation validation;
- hermetic builds/comptime;
- package/update/supply-chain system;
- complete standard library;
- ABI/FFI;
- wire/distributed execution;
- formatter/LSP/AI compiler API;
- complete qualification and security evidence.

## 25.3 `3.0.0.0` — Self-Hosted and Independent Omni

Must provide:

- Omni-written tokenizer;
- Omni-written parser/CST/AST;
- Omni-written semantic analyzer;
- Omni-written ownership/effect/capability engine;
- Omni-written HIR/MIR/verifier;
- Omni-written reference interpreter;
- Omni-written backend/build tooling to the final permitted independence boundary;
- self-compilation;
- bootstrap transition;
- DDC qualification;
- zero ordinary Rust/compiler dependency;
- complete transitive dependency audit;
- reproducible release artifacts;
- archived bootstrap recovery kit.

---

# 26. Release and Certification Rules

## 26.1 No certification by feature count

A version cannot be declared complete merely because a checklist of features appears implemented.

Certification requires implementation evidence, conformance, verification, reproducibility, and dependency audits.

## 26.2 No silent semantic divergence

A later compiler may not reinterpret an already-ratified rule merely because a new backend, optimizer, or host language makes implementation easier.

## 26.3 No release on unresolved critical mismatch

The following are release blockers:

- DDC mismatch;
- reproducibility mismatch without approved equivalence;
- stale semantic linkage;
- unknown registry references;
- rule lifecycle violation;
- unsound MIR acceptance;
- unsafe assumption use after invalidation;
- capability escalation;
- memory-model violation;
- unexplained FFI escape;
- distributed capability bypass;
- unresolved critical security issue.

---

# 27. Final Implementation Philosophy

The complete development strategy is therefore:

```text
SPECIFICATION
     ↓
MACHINE-READABLE SEMANTIC CONTRACTS
     ↓
RUST MINIMAL COMPILER
     ↓
1.0.0.0  — MINIMAL QUALIFIED CORE
     ↓
RUST FULL PRODUCTION COMPILER
     ↓
2.0.0.0  — COMPLETE QUALIFIED RUST PLATFORM
     ↓
OMNI-WRITTEN COMPILER + OWN TOOLCHAIN
     ↓
SELF-COMPILATION
     ↓
DDC / INDEPENDENT QUALIFICATION
     ↓
RUST SEVERANCE
     ↓
3.0.0.0  — CERTIFIED SELF-HOSTED OMNI
```

This makes the goals distinct:

- Stage 1 proves **minimal correctness**;
- Stage 2 proves **complete production capability in Rust**;
- Stage 3 proves **language ownership, self-compilation, and eventual independence**.

The sequence is intentionally conservative about trust while remaining aggressive about feature coverage.

---

# 28. Final Starting Point

The implementation plan begins at:

```text
OMNI-IMP-0.0.0.1
Repository Blank Page + Specification Topology
```

The first implementation language is Rust.

The first semantic infrastructure is the specification registry, manifest, canonicalization, conformance, and provenance boundary.

The first compiler capability is canonical source handling and lexical analysis.

The first complete executable proof target is the Stage-0 vertical slice.

The first major qualification boundary is:

```text
1.0.0.0 — Minimal Qualified Rust Core
```

The second major qualification boundary is:

```text
2.0.0.0 — Complete Production Rust Platform
```

The final target is:

```text
3.0.0.0 — Certified Self-Hosted, Independently Qualified Omni
```

---

# 29. Planning-Phase Closure

This document is an implementation plan only.

No compiler source, repository bootstrap, CI workflow, package, generated artifact, or implementation claim is created merely by this plan.

The complete Omni specification remains the semantic authority.

The implementation plan remains the authoritative sequencing, engineering, integration, testing, qualification, and self-hosting roadmap.

The following are frozen at planning level:

- exact `R.M.m.P` versioning;
- three-stage generation boundaries;
- Rust-only implementation throughout Stage 1;
- Rust-only complete production implementation throughout Stage 2;
- Omni compiler/toolchain construction during Stage 3;
- final self-hosting and independence as a separate qualification boundary;
- CI Reaper lifecycle enforcement;
- witness-test obligations;
- deterministic canonicalization;
- `/omni-root/` path virtualization;
- specification-tree cryptographic provenance binding;
- hard CI/fuzzing budgets;
- staged backend/toolchain entry, including delayed LLVM introduction;
- exhaustive compatibility/regression testing;
- DDC and transitive TCB auditing;
- no implementation-local invention of semantics;
- no stale or abandoned production paths;
- no release certification based solely on feature count.

Any change to these planning laws requires a deliberate revision of this Master Plan rather than an implementation-local exception.


# 50. Final Planning Amendment — Three-Stage Execution Contract

This section is the definitive planning interpretation of the three implementation stages. It supersedes any earlier roadmap wording that places complete production, self-hosting, or final independence in a different generation.

## 50.1 Stage 1 — Rust-Only Minimal Build

**Version range:** `0.0.0.1` → `1.0.0.0`

Stage 1 produces the smallest trustworthy working implementation of the Omni language using Rust as the sole implementation language. It is a minimal language implementation, not a claim of complete Edition-1 feature coverage.

The minimum executable pipeline is:

```text
source bytes
→ canonical source representation
→ lexer/token stream
→ lossless CST
→ typed AST
→ name resolution
→ Stage-0 type checking
→ HIR
→ canonical MIR
→ MIR verification
→ reference abstract-machine interpretation
→ minimal native lowering
→ object/image generation
→ Stage-0 runtime startup
```

Stage 1 prioritizes semantic correctness, deterministic diagnostics, specification traceability, reference-machine agreement, reproducible artifacts, fail-closed behavior, and a minimal trusted computing base.

Stage 1 SHALL NOT silently emulate deferred Stage-2 semantics.

### `1.0.0.0` release gate

`1.0.0.0` requires:

1. every advertised Stage-0 feature implemented;
2. rule-level conformance coverage;
3. reference-machine/native differential agreement over the qualified corpus;
4. zero release-path compiler panics;
5. semantic-linkage audit success;
6. specification-tree provenance binding;
7. reproducible bootstrap artifacts;
8. complete release evidence.

Therefore `1.0.0.0` means **Minimal Working Omni Implementation in Rust**.

## 50.2 Stage 2 — Rust-Only Complete Production Platform

**Version range:** `1.0.0.1` → `2.0.0.0`

Stage 2 expands the Stage-1 implementation into the complete production Omni platform while keeping Rust as the implementation language for the compiler/toolchain. No Stage-2 production milestone may switch the primary implementation language to Omni.

The production completeness envelope includes, where defined by the frozen specification:

```text
frontend
  source / Unicode / lexer / CST / AST
  names / modules / macros / diagnostics

semantic engine
  types / generics / inference / traits
  ownership / borrowing / lifetimes / drop
  effects / capabilities / continuation rules

IR and correctness
  HIR / MIR
  memory and provenance
  concurrency and memory order
  unsafe-assumption dependency tracking
  MIR verification
  translation validation

execution
  reference machine / interpreter
  native backend(s)
  runtime / ABI / object / linker integration

ecosystem
  standard libraries
  hermetic builds
  package/update/supply-chain system
  FFI
  persistence / wire
  distributed execution
  formatter / LSP / compiler APIs

qualification
  unit / property / fuzz
  model / differential
  reproducibility
  security / dependency / TCB audits
```

Every subsystem advances through:

```text
specified
→ designed
→ implemented
→ unit-qualified
→ integration-qualified
→ conformance-qualified
→ differential/model-qualified where applicable
→ reproducibility-qualified where applicable
→ release-qualified
```

A feature is never complete merely because its parser accepts its syntax.

### `2.0.0.0` release gate

`2.0.0.0` means **Complete Production Omni Platform implemented and qualified in Rust against the applicable Edition-1 specification**.

Rust remains the implementation language through this gate.

## 50.3 Stage 3 — Omni-Owned Compiler, Self-Hosting, and Independence

**Version range:** `2.0.0.1` → `3.0.0.0`

Stage 3 has three distinct proof transitions:

```text
A. Omni implementation
B. Omni self-compilation
C. Independent severance
```

A does not imply B, and B does not imply C.

### A. Omni implementation

The compiler/toolchain is rewritten as a first-class Omni implementation. The Omni implementation must cover the compiler responsibilities themselves:

```text
source loader
lexer/tokenizer
CST
parser
AST/HIR
name resolution
type/inference engine
ownership/lifetime checking
effects/capability checking
HIR→MIR
MIR verifier
reference interpreter
code generation
runtime support
diagnostics
build/package tooling
formatter
conformance runner
bootstrap/reproduction tooling
```

External tools may be used as explicitly classified bootstrap mechanisms, but the Omni compiler cannot remain merely a wrapper around a foreign compiler and still claim to be rewritten in Omni.

### B. Omni self-compilation

The Epoch-2 Rust compiler initially compiles the Omni-written compiler. Then the Omni-written compiler compiles its own source.

```text
Rust production compiler
→ Omni compiler source
→ compiler binary A
→ compiler source
→ compiler binary B
→ compiler source
→ …
→ convergence
```

Every self-build records source-tree digest, normative spec-tree digest, compiler identity, target identity, dependency inventory, configuration, artifact digest, conformance status, and reproducibility evidence.

### C. Independent severance

Final independence is not established by language-of-implementation alone. The severance audit must inspect source dependencies, transitive libraries, generated code, runtime linkage, startup paths, build scripts, host tools, environment assumptions, dynamic libraries, embedded toolchain artifacts, and hidden generators.

Every dependency is classified as one of:

```text
retained platform primitive
retained interoperability boundary
temporary bootstrap dependency
verification-only dependency
forbidden final dependency
```

A dependency cannot be declared removed merely because it disappeared from Cargo manifests.

The final artifact SHALL NOT require the Rust compiler, Rust source/runtime libraries, another programming-language compiler as an implicit build stage, an undeclared scripting runtime, an undisclosed remote build service, or an undeclared generator.

Explicit hardware/OS primitives may remain where required by the platform contract. Any remaining foreign backend library must be explicitly classified, version-pinned, reproducibly acquired, and audited rather than being called "zero dependency" informally.

### `3.0.0.0` release gate

`3.0.0.0` requires:

1. the compiler/toolchain is written in Omni;
2. the compiler can compile Omni compiler source;
3. self-compilation has qualified convergence;
4. independent rebuilds reproduce the declared artifact;
5. DDC evidence is complete;
6. the final dependency inventory contains no forbidden bootstrap dependency;
7. TCB inventory and audit are complete;
8. final conformance and security qualification pass;
9. Rust is no longer required for normal production compilation.

Therefore `3.0.0.0` means **Certified Self-Hosted and Independent Omni**.

# 51. Final Multi-Paradigm Engineering Objective

Omni is explicitly intended to accommodate as many programming methodologies as the frozen semantic architecture can support without dialect fragmentation. This is an engineering objective, not permission to invent unspecified semantics.

Where specified, the implementation should support:

- **Functional:** pure computation, immutability, algebraic data types, exhaustive pattern matching, higher-order functions, closures, recursion, and tail-recursive forms.
- **Object-oriented:** encapsulation through visibility/module boundaries, composition, trait-based polymorphism, dynamic dispatch where specified, and explicit representation/layout semantics.
- **Systems/data-oriented:** predictable layouts, explicit representation, ownership-driven memory management, arenas, low-level operations, alignment, vectorization where legal, and data-oriented transformations.
- **Concurrent/asynchronous/actor/reactive:** structured concurrency, task trees, channels/message passing, asynchronous state machines, cancellation, synchronization, and explicit memory-order semantics.
- **Declarative/metaprogramming/compile-time:** macros, comptime execution, generated state machines, compile-time validation, and deterministic generated artifacts.

Future paradigms may be accommodated only by mapping them into the existing semantic model or by changing the normative specification through its formal amendment process.

The target architecture is therefore:

```text
many programming methodologies
        ↓
one semantic type/effect/ownership/capability model
        ↓
common HIR
        ↓
common canonical MIR
        ↓
common verification
        ↓
reference execution semantics
        ↓
multiple legal backend targets
```

# 52. Feature-Completeness Ledger

For every substantial feature, the implementation plan must track:

```text
syntax
→ tokenization
→ CST/AST representation
→ name/visibility semantics
→ type semantics
→ ownership/lifetime implications
→ effect/capability implications
→ HIR lowering
→ MIR representation
→ verifier rules
→ reference-machine behavior
→ native lowering
→ runtime support
→ diagnostics
→ conformance tests
→ fuzz/property tests where applicable
→ reproducibility impact
→ documentation
→ compatibility impact
```

This prevents implementation from stopping at syntax or parser support.

# 53. Compiler Component Parity for Stage 3

The Omni-written implementation is expected to provide first-class equivalents for all essential compiler responsibilities:

| Responsibility | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|
| source handling | Rust | Rust | Omni |
| lexer/tokenizer | Rust | Rust | Omni |
| CST/parser | Rust | Rust | Omni |
| names/modules | Rust | Rust | Omni |
| type/inference | minimal | complete | Omni |
| ownership/lifetimes | minimal | complete | Omni |
| effects/capabilities | deferred/minimal | complete | Omni |
| HIR/MIR | minimal | complete | Omni |
| MIR verifier | baseline | complete | Omni |
| reference machine | baseline | complete | Omni |
| native codegen | minimal | production | Omni |
| runtime support | minimal | production | Omni |
| diagnostics | baseline | complete | Omni |
| package/build tooling | deferred | production | Omni |
| formatter/LSP/tooling | deferred | production | Omni |
| conformance runner | baseline | complete | Omni |
| bootstrap/reproduction tools | baseline | complete | Omni |

Parity means equivalent observable semantic and qualification behavior; it does not require identical internal algorithms.

# 54. Bootstrap Trust Progression

```text
Frozen normative specification
        ↓
machine-readable spec registry/manifest
        ↓
Rust Stage 1 minimal compiler
        ↓
Rust Stage 1 qualified artifact
        ↓
Rust Stage 2 complete production compiler
        ↓
Rust compiler bootstraps Omni-written compiler
        ↓
Omni compiler compiles Omni compiler
        ↓
independent rebuilds
        ↓
DDC
        ↓
TCB audit
        ↓
Rust removed from normal production compilation
        ↓
3.0.0.0
```

An implementation must never claim independence merely because its source language is Omni.

# 55. Final Qualification Evidence

Every generational gate must leave an auditable evidence package containing, as applicable:

```text
source-tree digest
spec-tree digest
toolchain identity
target identity
dependency locks
configuration
test manifests
conformance report
fuzz report
reference-machine differential report
reproducibility report
artifact digests
semantic-linkage report
TCB inventory
SBOM
security/dependency audit
known limitations
release decision
```

Stage 3 additionally requires bootstrap dependency inventory, self-compilation evidence, independent rebuild evidence, dependency-severance evidence, DDC evidence, final TCB report, and final artifact convergence evidence.

# 56. Final Prohibition on Premature Completion

The following states are distinct and must never be conflated:

```text
plan finalized
≠ repository initialized
≠ Stage 1 implemented
≠ 1.0.0.0 qualified
≠ Stage 2 implemented
≠ 2.0.0.0 qualified
≠ Omni compiler rewritten
≠ self-hosted
≠ independently rebuilt
≠ DDC qualified
≠ 3.0.0.0 certified
```

Only implementation artifacts plus their qualification evidence may advance a planned milestone to achieved status.

# 57. Comprehensive Planning Coverage Contract

The phrase **most comprehensive implementation plan** is a binding planning requirement. The plan is not considered complete merely because the compiler pipeline is listed. It must cover the complete engineering lifecycle needed to take Omni from an empty repository to a certified self-hosted implementation.

The plan SHALL account for all of the following dimensions:

1. normative specification traceability;
2. machine-readable semantic contracts;
3. repository and workspace topology;
4. bootstrap toolchain selection and pinning;
5. source acquisition and provenance;
6. lexical, syntactic, semantic, and intermediate representations;
7. type, ownership, lifetime, effect, capability, and memory systems;
8. compile-time execution and metaprogramming;
9. runtime, ABI, object format, linking, and startup;
10. optimization and translation validation;
11. concurrency, asynchronous execution, cancellation, and cleanup;
12. FFI and foreign-runtime containment;
13. package management, dependency resolution, updates, and supply-chain security;
14. hermetic and reproducible builds;
15. persistence, serialization, wire protocols, and distributed execution where specified;
16. standard library architecture and platform abstraction;
17. compiler tooling and developer experience;
18. AI-assisted compilation interfaces and safety boundaries;
19. testing, fuzzing, property testing, model checking, differential testing, and formal evidence;
20. performance engineering;
21. portability and target support;
22. security engineering and threat modeling;
23. repository health and anti-rot controls;
24. release engineering and artifact publication;
25. bootstrap recovery;
26. self-hosting migration;
27. DDC and independent qualification;
28. dependency severance;
29. long-term maintenance and governance;
30. final certification and operational readiness.

Any component that is necessary to make a listed feature practically usable must be scheduled or explicitly classified as out of scope for the relevant generation. It must not remain an accidental omission.

# 58. Specification-to-Implementation Traceability Matrix

The implementation plan must maintain a machine-readable and human-readable traceability matrix connecting:

```text
Normative specification section
        ↓
Rule identifier(s)
        ↓
Machine-readable schema/model
        ↓
Compiler subsystem
        ↓
Implementation module(s)
        ↓
IR representation
        ↓
Verifier obligation(s)
        ↓
Runtime/reference-machine behavior
        ↓
Conformance witness(es)
        ↓
Fuzz/property/model test(s)
        ↓
Differential test(s)
        ↓
Release qualification gate
```

A feature or rule is considered fully accounted for only when every applicable node in this chain is either implemented or explicitly marked as not applicable with a reason.

Traceability status SHALL distinguish at least:

```text
not mapped
mapped
specified
designed
implemented
unit-tested
integration-tested
conformance-tested
model-checked
differentially-tested
reproducibility-tested
security-reviewed
release-qualified
certified
```

The status must never automatically advance because another status was reached.

# 59. Semantic Inventory and Single-Authority Registry

Every semantic quantity used by the implementation must have a single authoritative definition or derivation path.

The semantic inventory should catalog, where applicable:

- primitive types;
- compound types;
- operators;
- precedence/associativity;
- literal forms;
- coercions;
- subtyping or compatibility rules;
- lifetime relations;
- ownership transitions;
- initialization states;
- drop states;
- effect constructors;
- capability rights;
- memory orderings;
- pointer/provenance states;
- panic/abort states;
- cancellation states;
- wire types;
- runtime identities;
- diagnostic codes;
- target capabilities;
- profile predicates.

Each implementation representation must identify whether it is:

- authoritative;
- derived;
- cached;
- diagnostic-only;
- performance-only;
- test-oracle-only.

Two independent authoritative representations of the same semantic fact are prohibited unless a formal equivalence checker is provided.

# 60. Intermediate Representation Design Contract

The IR architecture must be planned as a sequence of intentionally different abstractions rather than one overloaded representation.

At minimum, the plan must distinguish:

```text
CST
  ↓
AST
  ↓
HIR
  ↓
MIR
  ↓
Verifier-normalized MIR
  ↓
Target-neutral LIR where necessary
  ↓
Backend IR
  ↓
Object/image representation
```

For each representation the plan must define:

- purpose;
- invariants;
- ownership of source information;
- canonicalization rules;
- serialization format if applicable;
- mutability policy;
- allowed transformations;
- prohibited transformations;
- verification boundary;
- debugging/diagnostic mapping;
- lifetime of instances;
- caching behavior.

No optimization pass may mutate an IR beyond its declared contract.

# 61. Compiler Query and Incremental Computation Architecture

The implementation must plan for deterministic query-based compilation even if incremental compilation is introduced only after the minimal core.

Queries may cover:

- source loading;
- parsing;
- module discovery;
- name resolution;
- type computation;
- effect inference;
- capability analysis;
- ownership/lifetime analysis;
- HIR construction;
- MIR construction;
- verification;
- code generation;
- diagnostics;
- package resolution.

Each query must define:

- input identity;
- dependency set;
- cache key;
- invalidation causes;
- output identity;
- deterministic serialization if persisted;
- concurrency behavior.

The cache must never accidentally encode host paths, clock time, process IDs, random values, or uncontrolled environment state.

# 62. Compiler Memory and Resource Management Plan

The implementation plan must account for compiler resource behavior itself.

It must define, as applicable:

- arena strategies;
- interning;
- string storage;
- incremental memory reclamation;
- cache limits;
- peak memory targets;
- recursion limits;
- parser nesting limits;
- type-solver fuel;
- comptime fuel;
- macro-expansion limits;
- diagnostic limits;
- output-size limits.

Resource exhaustion must result in deterministic, classified diagnostics rather than host-language crashes or unbounded resource consumption wherever the specification permits controlled failure.

# 63. Deterministic Diagnostics Contract

Diagnostics are part of the implementation contract.

The plan must define:

- stable diagnostic identifiers;
- severity classes;
- deterministic ordering;
- primary/secondary spans;
- related-rule identifiers;
- machine-readable representation;
- human-readable rendering;
- fix-it representation where supported;
- canonical serialization;
- localization policy if localization is ever added;
- regression requirements.

Two compilations of identical canonical inputs must not produce different diagnostic identities or order merely because of hash-map iteration order, thread scheduling, filesystem order, or host paths.

# 64. Error Recovery and Fault Containment

Every pipeline layer must specify whether an error is:

- recoverable;
- recoverable only for diagnostics;
- semantically fatal;
- compilation-fatal;
- process-fatal;
- sandbox-fatal;
- security-fatal.

Error recovery must not accidentally permit invalid nodes into release translation.

The implementation plan must explicitly distinguish parser recovery from semantic acceptance. Recovering from syntax errors for editor usability is not permission to compile the recovered malformed program.

# 65. Concurrency Inside the Compiler

The plan must cover both **program concurrency semantics** and **compiler implementation concurrency**.

Compiler concurrency must preserve deterministic results.
Parallelism may reorder internal work, but it must not alter:

- semantic conclusions;
- diagnostic ordering after canonical sorting;
- rule linkage;
- artifact hashes;
- cache identities;
- optimization decisions except where an explicitly deterministic tie-break policy exists.

Shared compiler state must have an explicit ownership/synchronization design. Race freedom of the Rust implementation is necessary but not sufficient; deterministic compiler output remains a separate requirement.

# 66. Target and Platform Strategy

The implementation plan must maintain a target matrix.

Each target record should contain:

- architecture;
- operating environment;
- object format;
- ABI;
- calling convention;
- data layout;
- atomic capabilities;
- vector capabilities;
- linker requirements;
- startup model;
- runtime requirements;
- unsupported features;
- conformance status;
- reproducibility status.

The canonical Stage-0 target remains the authoritative bootstrap target identified by the specification. Optional host targets must be treated as additional implementation targets rather than silently redefining the bootstrap contract.

# 67. Backend Abstraction and Multiple Code Generators

The compiler must separate semantic lowering from backend-specific code generation.

A backend interface should define:

- target discovery;
- data layout;
- instruction-selection capabilities;
- register/stack requirements;
- relocation model;
- object emission;
- debug/provenance metadata;
- linker integration;
- runtime ABI.

The reference machine is always independent of the backend.

Multiple backends are allowed only when each obeys the same verified MIR contract.

A backend may not introduce semantics that the reference machine does not permit.

# 68. Optimization Pipeline Governance

Optimization must be treated as a separately qualified subsystem.

For each optimization, record:

- source IR invariants;
- preconditions;
- legality proof;
- affected observable behaviors;
- unsafe-assumption dependencies;
- memory/provenance implications;
- target dependencies;
- deterministic controls;
- validation tests;
- rollback path.

Optimization passes must be classified as:

```text
semantics-preserving
semantics-refining under declared assumptions
profile-dependent
backend-only
non-deterministic and therefore forbidden
```

The final category is not allowed in release-qualified deterministic builds.

# 69. Security Architecture and Threat Model

The plan must include explicit threat models for:

- malicious source input;
- malformed source;
- malicious packages;
- dependency confusion;
- signature compromise;
- compromised build hosts;
- malicious build scripts;
- compromised bootstrap compiler;
- poisoned compiler cache;
- compiler supply-chain attacks;
- hostile FFI code;
- unsafe code;
- distributed execution state;
- capability escalation;
- downgrade/replay attacks;
- artifact substitution;
- timestamp/path/environment injection.

For each threat, the plan must identify:

```text
asset
attacker capability
attack surface
preventive control
detection control
containment
recovery
qualification evidence
```

# 70. Bootstrap Recovery Architecture

The final ecosystem must retain a documented recovery path in case the primary self-hosted compiler becomes unavailable or suspect.

The recovery kit must contain, where applicable:

- trusted seed source;
- trusted seed compiler binaries;
- target emulator/hardware definition;
- assembler/object writer;
- linker/image builder;
- specification data;
- toolchain identities;
- cryptographic digests;
- offline build instructions;
- reproducibility instructions;
- verification procedures;
- independent DDC inputs.

Recovery artifacts must be separately classified from the ordinary production dependency graph.

# 71. Build Graph and Hermeticity Model

The build system must model every artifact as a function of explicit inputs.

Conceptually:

```text
Artifact = F(
  source,
  normative_spec,
  toolchain,
  target,
  configuration,
  declared_environment,
  dependency_snapshot
)
```

Any input not included in the declared function is a hermeticity defect.

The plan must specify mechanisms for detecting hidden inputs, including filesystem accesses, environment reads, network accesses, time reads, locale, randomness, CPU-feature discovery, and host-path leakage where relevant.

# 72. Package and Supply-Chain Lifecycle

Package management must be planned beyond dependency resolution.

The lifecycle includes:

```text
registry metadata
→ snapshot verification
→ package discovery
→ version selection
→ dependency solving
→ artifact verification
→ local cache
→ build
→ provenance
→ installation
→ update
→ yank/revoke
→ rollback/recovery
```

Security revocation must be stronger than ordinary version selection. A revoked package must not be reintroduced solely through stale metadata or a preserved lockfile.

# 73. Runtime and Platform Abstraction

The runtime plan must distinguish:

- language runtime semantics;
- platform ABI;
- operating-system bindings;
- capability providers;
- allocator implementation;
- scheduling implementation;
- I/O implementation;
- time/entropy providers;
- process lifecycle;
- signal/exception handling.

Only the appropriate layer may depend on platform-specific mechanisms.

# 74. ABI Stability and Compatibility Matrix

The implementation plan must maintain explicit compatibility records for:

- source compatibility;
- AST/CST compatibility;
- semantic compatibility;
- ABI compatibility;
- object compatibility;
- package metadata compatibility;
- wire compatibility;
- runtime compatibility;
- tooling protocol compatibility.

An implementation version change may preserve some dimensions while changing others, but the matrix must declare that explicitly.

# 75. Standard Library Governance

The standard library must be treated as an extension of the language contract, not as arbitrary application code.

Each standard-library module must specify:

- semantic purpose;
- capability requirements;
- effect requirements;
- memory/ownership behavior;
- platform dependencies;
- error model;
- determinism properties;
- test coverage;
- portability;
- stability status.

Core/freestanding components must remain separable from platform-dependent services.

# 76. Tooling and IDE Architecture

The development environment must be planned as a family of interfaces over the same semantic engine.

Components may include:

- formatter;
- language server;
- code navigation;
- completion;
- refactoring;
- semantic highlighting;
- diagnostics visualization;
- ownership/effect/capability visualization;
- package tools;
- debugger integration;
- test explorer;
- build graph inspection.

Tooling must never become a second semantic implementation.

# 77. AI Compiler Interface and Safety

The AI-facing compiler interface must expose structured information such as:

- diagnostics;
- violated rules;
- relevant source spans;
- expected/observed semantic state;
- proof obligations;
- available legal repairs;
- qualification status.

It must not expose an escape hatch that allows an AI tool to bypass:

- type checking;
- ownership checks;
- capability checks;
- MIR verification;
- package security;
- release qualification.

AI-generated changes remain subject to the exact same repository and semantic gates as human-generated changes.

# 78. Performance Engineering Contract

Performance work must be measurable and reproducible.

The plan must maintain benchmark suites for:

- lexer throughput;
- parser throughput;
- name resolution;
- type checking;
- ownership checking;
- effect/capability checking;
- MIR construction;
- verification;
- code generation;
- incremental rebuilds;
- clean builds;
- memory consumption;
- generated-binary quality;
- runtime execution.

Benchmark regressions must have defined thresholds and environment metadata.

Performance optimization may be postponed, but hidden performance debt may not be mistaken for completed production quality.

# 79. Portability and Environmental Qualification

Qualification must eventually cover the supported host/build environments declared by the project.

The plan should maintain a matrix covering:

- host operating system;
- host architecture;
- filesystem behavior;
- path representation;
- line endings;
- available compiler toolchains;
- linker implementations;
- CPU feature sets;
- locale/encoding environments.

Where platform differences are intentionally unsupported, the reason and boundary must be documented.

# 80. Repository Operations and Anti-Rot Process

The project must continuously prevent repository drift.

At every coherent milestone:

```text
inspect current tree
→ compare against plan
→ identify obsolete artifacts
→ remove or quarantine superseded paths
→ refresh generated metadata
→ run full affected tests
→ run global health gate
→ verify clean tree
→ create coherent commit
→ record release evidence
```

No old implementation is retained merely because removing it is inconvenient.

# 81. Change Impact Analysis

Before accepting a change, identify its impact across:

```text
specification
→ semantic registry
→ compiler layers
→ runtime
→ tools
→ conformance
→ fuzzing
→ generated artifacts
→ packages
→ documentation
→ release metadata
```

Changes that cross subsystem boundaries require cross-subsystem qualification.

A local test pass is not sufficient evidence for a system-wide change.

# 82. Failure Classification and Escalation

All discovered problems must receive a classification:

```text
build failure
lint failure
unit-test failure
integration failure
semantic mismatch
conformance failure
verifier failure
runtime failure
reproducibility failure
security failure
bootstrap failure
DDC failure
specification defect
infrastructure defect
```

Each class must have an escalation path and release-blocking status.

# 83. Documentation Synchronization

The plan must keep documentation synchronized across:

- normative specification references;
- implementation architecture documents;
- API documentation;
- compiler internals documentation;
- bootstrap instructions;
- build instructions;
- contributor documentation;
- release notes;
- migration notes;
- security advisories.

Documentation drift that causes a contributor to follow a semantically invalid procedure is a project defect.

# 84. Release Packaging

Every release gate must define the exact publication set.

A release package may contain:

- compiler binaries;
- runtime/core libraries;
- package manager tools;
- target support files;
- standard library artifacts;
- documentation;
- source archives;
- SBOM;
- signatures;
- provenance;
- conformance report;
- reproducibility evidence;
- DDC evidence where required;
- bootstrap recovery materials.

The release manifest must enumerate every published artifact and its digest.

# 85. Final Acceptance Review Procedure

Before any generational gate is declared achieved, an acceptance review must walk the implementation from top to bottom:

```text
specification
→ registry
→ build environment
→ source/lexer
→ parser/CST/AST
→ names
→ types
→ ownership/lifetimes
→ effects/capabilities
→ HIR
→ MIR
→ verifier
→ reference machine
→ optimization
→ backend
→ runtime
→ packages
→ tooling
→ distributed/interop features
→ tests
→ reproducibility
→ security
→ TCB
→ release artifacts
```

For each layer, the reviewer records:

- implemented status;
- test status;
- known limitations;
- dependencies;
- semantic ownership;
- qualification evidence;
- rollback/recovery path.

The review is complete only when every required layer has an affirmative disposition.

# 86. Final Definition of “Complete”

The word **complete** has the following formal meaning in this implementation plan:

A subsystem is complete only when its implementation, integration, tests, semantic linkage, applicable verification, documentation, resource behavior, compatibility impact, reproducibility impact, and release status have all been addressed.

A stage is complete only when all required subsystems are complete and the stage exit gate passes.

The project is complete for Edition 1 only when:

```text
Rust Minimal Core
      ↓
1.0.0.0
      ↓
Rust Complete Production Platform
      ↓
2.0.0.0
      ↓
Omni Compiler Reimplementation
      ↓
Omni Self-Compilation
      ↓
Independent Rebuilds
      ↓
DDC
      ↓
Dependency Severance
      ↓
3.0.0.0
```

# 87. Final Planning Principle

The implementation plan is intentionally more conservative than a conventional compiler roadmap because Omni is intended to become its own implementation foundation.

The project therefore optimizes for the following hierarchy:

```text
semantic correctness
    > soundness
    > deterministic behavior
    > reproducibility
    > security
    > traceability
    > maintainability
    > portability
    > performance
    > convenience
```

Performance and convenience remain important, but no optimization is allowed to weaken a higher-ranked property.

The breadth objective is equally explicit:

```text
one semantic core
   → many programming methodologies
   → many execution environments
   → many tooling workflows
   → one qualification framework
```

That is the standard against which the implementation roadmap is to be judged.

# 88. Planning Closure Addendum

With this coverage contract, the plan is considered comprehensive at the planning level. Future implementation work may reveal additional engineering tasks that cannot reasonably be known before the repository and specification-derived models are instantiated. Such discoveries are to be incorporated as controlled plan amendments rather than silently omitted or improvised.

This does not reopen the frozen semantic architecture. It expands implementation detail only when evidence demonstrates that the existing plan lacks an engineering task required to realize an already-defined contract.

---

# Final Micro-Atomic Execution Ledger Integration

This section is the execution-level refinement of the comprehensive Master Plan. It specifies implementation artifacts, data structures, algorithms, linkage requirements, invariants, tests, and qualification gates without changing the normative language semantics. The three generational release gates remain authoritative: `1.0.0.0` (minimal Rust core), `2.0.0.0` (complete production Rust platform), and `3.0.0.0` (certified self-hosted and independently qualified Omni).

## 10. Hyper-Granular Implementation Ledger: Epoch 1 (Rust Minimal Core)

**Version Range:** **`0.0.0.1`** **→** **`1.0.0.0`**

### 10.0: The Foundation & Topography (`0.0.0.x`)

**`0.0.0.1`** **- The Void State (Repository Blank Page)**

- **Action:** `mkdir omni && cd omni && git init`



- **Files Created:** `.gitignore` (ignoring `/target`, `*.swp`, OS artifacts).



- **Directories Created:** `spec/`, `compiler/`, `tools/`, `conformance/`, `tests/`, `docs/`, `scripts/`, `ci/`.



- **Workspace Root:** Create `Cargo.toml`.



  - *Contents:* `[workspace]`, `resolver = "2"`, `members = ["compiler/*", "tools/*"]`.



  - *Lints:* `[workspace.lints.rust] unsafe_code = "forbid"`, `unreachable_pub = "deny"`, `unused_must_use = "deny"`.



- **Gate:** `cargo tree` executes instantly. Zero Rust dependencies downloaded. Zero compilation targets.




**`0.0.0.2`** **- The Hermetic Pin**

- **Files Created:** `rust-toolchain.toml`



  - *Contents:* `channel = "1.80.0"` (or exact active stable version), `components = ["rustfmt", "clippy"]`, `targets = ["riscv64-unknown-none-elf"]`, `profile = "minimal"`.



- **Gate:** Any attempt to use `cargo +nightly` or `cargo +1.79.0` MUST be rejected by the environment.




**`0.0.0.3`** **- The Quality Contract**

- **Files Created:** `.rustfmt.toml`, `clippy.toml`, `.github/workflows/ci.yml` (or equivalent).



- **Configuration:** `rustfmt` set to `max_width = 100`, `newline_style = "Unix"`.



- **CI Configuration:** Implement the hard timeout variables: `PR_TIMEOUT: 20m`, `MAIN_TIMEOUT: 30m`.



- **Gate:** CI successfully runs `rustfmt --check` and `cargo clippy` on the empty workspace and exits `0`.




**`0.0.0.4`** **- The Law (Rule Registry Schema)**

- **Files Created:** `spec/schemas/rule-registry.schema.json`.



- **Data Structure:** JSON Schema strictly enforcing `{ rule_id: String, domain: String, status: Enum["Proposed", "Candidate", "Ratified", "Deprecated"], normative: Bool, text_hash: String, dependencies: Array<String>, witness_tests: Array<String> }`.



- **Validation Rule:** `if status == Ratified { minItems(witness_tests) == 1 }`. Additional fields `additionalProperties: false`.




**`0.0.0.5`** **- The Root of Trust (Specification Manifest)**

- **Files Created:** `spec/manifest/omni-edition1.manifest.json`.



- **Data Structure:** Contains relative paths to all schemas, models, and grammar files.



- **Feature Predicates:** Defines `stage0_allowed: ["core_types", "fixed_integers", ...]`.




**`0.0.0.6`** **- Canonicalization Engine (****`omni-canon`****)**

- **Files Created:** `tools/omni-canon/Cargo.toml`, `tools/omni-canon/src/main.rs`.



- **Dependencies:** `serde_json`, `sha2`, `hex`.



- **Algorithm:**



  1. Ingest JSON/YAML.



  2. Recursively sort object keys lexicographically.



  3. Recursively match string values for the absolute host path root (e.g., `env::current_dir()`) and replace with `/omni-root/`.



  4. Serialize with zero whitespace.



  5. Pass through SHA-256.



  6. Print hex digest.



- **Gate:** Process two identical JSON objects with scrambled keys and Windows vs Unix paths. Outputs MUST match.




**`0.0.0.7`** **- The CI Reaper (****`omni-audit`****)**

- **Files Created:** `tools/omni-audit/Cargo.toml`, `tools/omni-audit/src/main.rs`.



- **Dependencies:** `syn` (for parsing Rust AST), `serde_json`.



- **Algorithm:**



  1. Parse `spec/registry/rules.json`. Build in-memory Directed Acyclic Graph (DAG) of dependencies. Detect cycles.



  2. Walk the `compiler/` directory. Use `syn` to parse all `.rs` files.



  3. Extract all instances of `#[implements("XXX-0000")]`.



  4. Walk the `conformance/` directory. Extract all `#[witnesses("XXX-0000")]`.



  5. Perform set-difference operations: Cross-check `Ratified` rules vs implementation. Trap `Deprecated` rules vs implementation. Trap `Proposed` rules vs implementation.



- **Gate:** Run `omni-audit`. It must pass on the empty workspace.




**`0.0.0.8`** **-** **`0.0.0.13`****: Pipeline Topography & Finalization**

- **Action:** Generate all `compiler/omni-*` crates.



- **Constraint:** `omni-parse/Cargo.toml` specifies `omni-lex = { path = "../omni-lex" }`. Cycles are explicitly forbidden by Cargo, fulfilling the acyclic DAG requirement.



- **Release:** Tag commit as `OMNI-IMP-0.0.0.13`.




### 10.1: Canonical Source & Lexer (`0.1.x.x`)

**`0.1.0.1`** **- UTF-8 Byte Cursor**

- **Location:** `compiler/omni-source/src/cursor.rs`.



- **Data Structure:** `pub struct Cursor<'a> { bytes: &'a [u8], pos: usize, line: u32, col: u32 }`.



- **Logic:** `advance()` method that tracks bytes, mapping `\r\n` and `\r` to `\n` in a single pass without allocating a new string. Strips `0xEF,0xBB,0xBF` (BOM) strictly at `pos == 0`.




**`0.1.0.2`** **- Lossless Span & Token Model**

- **Location:** `compiler/omni-lex/src/token.rs`.



- **Data Structure:**



  - `pub struct Span { start: u32, end: u32, file_id: u16 }`



  - `pub enum TokenKind { Ident, Int, Float, Keyword(Kw), Punct(Punct), Indent, Dedent, Error }`



  - `pub struct Token { kind: TokenKind, span: Span, leading_trivia: Vec<Trivia>, trailing_trivia: Vec<Trivia> }`



- **Linkage:** `#[implements("LEX-0002")]` (Trivia preservation).




**`0.1.0.3`** **- Indentation Layout Engine**

- **Location:** `compiler/omni-lex/src/layout.rs`.



- **Algorithm:** Maintain `Vec<u32>` (indent stack). On `\n`, count following spaces. If `spaces > stack.last()`, emit `TokenKind::Indent`, push to stack. If `spaces < stack.last()`, pop stack and emit `TokenKind::Dedent` until `spaces == stack.last()`. Trap on mismatched un-indent.




**`0.1.0.4`** **- Maximal Munch Scanner**

- **Location:** `compiler/omni-lex/src/scanner.rs`.



- **Logic:** Implement DFA for keywords, identifiers, and numbers.



- **Linkage:** `#[implements("LEX-0001")]` (Contextual `>>` split).




**`0.1.0.5`** **- Lexer Fuzzing & Differential Gate**

- **Location:** `tests/fuzz/lexer_fuzz.rs`.



- **Logic:** `cargo fuzz` harness injecting random `&[u8]`. Verify `TokenKind::Error` is emitted instead of panics. Verify concatenating all token slices + trivia exactly reproduces the input bytes.




### 10.2: Concrete Syntax Tree & Parser (`0.2.x.x`)

**`0.2.0.1`** **- Rowan CST Foundation**

- **Location:** `compiler/omni-syntax/src/cst.rs`.



- **Dependencies:** `rowan`.



- **Logic:** Define the `OmniLanguage` struct mapping to `rowan::Language`. Define all `SyntaxKind` variants matching `OMNI-GRAMMAR`.




**`0.2.0.2`** **- Parser Event Machine**

- **Location:** `compiler/omni-parse/src/event.rs`.



- **Logic:** Instead of building the tree directly, the parser emits `StartNode`, `AddToken`, `FinishNode`, `Error`. This isolates grammatical analysis from tree construction.




**`0.2.0.3`** **- Pratt Expression Parser**

- **Location:** `compiler/omni-parse/src/expr.rs`.



- **Logic:** Define `fn binding_power(kind: TokenKind) -> (u8, u8)`. Implement `parse_expr_bp(min_bp: u8)`.



- **Linkage:** `#[implements("GRAM-0005")]`.




**`0.2.0.4`** **- Panic-Mode Error Recovery**

- **Location:** `compiler/omni-parse/src/recovery.rs`.



- **Logic:** On unexpected token, emit `Error` event, then consume tokens until `TokenKind::Dedent`, `TokenKind::Newline`, or `TokenKind::Semi` is encountered. Resume parsing.



- **Linkage:** `#[implements("GRAM-0007")]` (Error nodes never enter release translation).




**`0.2.0.5`** **- AST Lowering (Typed Wrappers)**

- **Location:** `compiler/omni-syntax/src/ast.rs`.



- **Logic:** Generate typed wrappers over Rowan nodes. E.g., `impl ast::FnDef { pub fn name(&self) -> Option<ast::Name> }`.




### 10.3: Names, DefId, & Type Solver (`0.3.x.x`)

**`0.3.0.1`** **- Canonical DefId**

- **Location:** `compiler/omni-names/src/def_id.rs`.



- **Data Structure:** `pub struct DefId { package: u32, module: u32, index: u32 }`.



- **Linkage:** `#[implements("NAME-0001")]`.




**`0.3.0.2`** **- Multi-Pass Resolver**

- **Location:** `compiler/omni-names/src/resolve.rs`.



- **Logic:** Traverse AST. Build `Rib` hierarchy (lexical scopes). Map AST Node IDs to `DefId`s. Detect and trap shadowing violations (`NAME-0006`).




**`0.3.0.3`** **- Type Interner Arena**

- **Location:** `compiler/omni-types/src/intern.rs`.



- **Data Structure:** `pub struct TyCtxt { types: HashSet<TyKind> }`. Returns `Ty` (a pointer/index). Allows `ty1 == ty2` to be a $O(1)$ integer comparison.




**`0.3.0.4`** **- Union-Find Constraint Solver**

- **Location:** `compiler/omni-types/src/solver.rs`.



- **Dependencies:** `ena` (Union-Find).



- **Logic:** Assign `TyVid` (type variables) to expressions. Generate equations (`T1 = T2`). Unify.



- **Linkage:** `#[implements("TYPE-0010")]` (Ambiguity is a hard error, no guessing).




**`0.3.0.5`** **- Stage-0 Type Enforcement**

- **Location:** `compiler/omni-types/src/check.rs`.



- **Logic:** Strictly limit types to fixed integers, booleans, structs, and pointers. Reject traits and generics for the Stage-0 bootstrap requirement.




### 10.4: Canonical MIR & Mechanical Verifier (`0.4.x.x`)

**`0.4.0.1`** **- MIR Data Structures**

- **Location:** `compiler/omni-mir/src/ir.rs`.



- **Data Structure:**



  - `pub struct Body { blocks: IndexVec<BasicBlock, BlockData>, local_decls: IndexVec<Local, LocalDecl> }`



  - `pub enum Statement { Assign(Place, Rvalue), Assume(Assumption), Drop(Place) }`



  - `pub enum Terminator { Goto(BasicBlock), Call { func: Operand, args: Vec<Operand>, target: BasicBlock, cleanup: Option<BasicBlock> }, Return }`




**`0.4.0.2`** **- Ownership & Drop Elaboration**

- **Location:** `compiler/omni-mir/src/drop.rs`.



- **Logic:** Compute initialization states using dataflow analysis. Inject `Statement::Drop` into the MIR CFG ensuring variables are dropped in reverse-initialization order.



- **Linkage:** `#[implements("OWN-0011")]`.




**`0.4.0.3`** **- Unsafe Assumption Tokens**

- **Location:** `compiler/omni-mir/src/assume.rs`.



- **Data Structure:** `pub struct Assumption { id: AssumptionId, obligation: String, deps: Vec<Place> }`.



- **Linkage:** `#[implements("AUDIT-OPT-UNSAFE-0001")]`.




**`0.4.0.4`** **- The MIR Verifier**

- **Location:** `compiler/omni-verify/src/main.rs`.



- **Logic:** A completely independent pass that reads `Body`. Validates that SSA properties hold. Validates that no `Place` is read after being moved. Validates that no `Assumption` is used across an invalidating FFI boundary.




### 10.5: Reference Machine & Stage-0 Native Codegen (`0.5.x.x`)

**`0.5.0.1`** **- Abstract Store (The Interpreter)**

- **Location:** `compiler/omni-machine/src/eval.rs`.



- **Data Structure:** `Memory { allocations: HashMap<AllocId, Allocation> }`. Tracks byte-level initialization and pointer provenance.



- **Logic:** Steps through MIR terminators. Directly executes mathematical logic. Traps immediately on out-of-bounds or invalid provenance.




**`0.5.0.2`** **- Cranelift Lowering**

- **Location:** `compiler/omni-codegen/src/cranelift.rs`.



- **Dependencies:** `cranelift-codegen`, `cranelift-module`, `cranelift-object`.



- **Logic:** Map MIR `Local`s to Cranelift `Variable`s. Translate MIR `Statement::Assign` to Cranelift `inst::store`. Translate `Terminator::Call` to Cranelift ABI calls.




**`0.5.0.3`** **- ELF/PE Object Writer**

- **Location:** `compiler/omni-codegen/src/object.rs`.



- **Logic:** Bundle compiled Cranelift functions into `.o` files. Generate the `main` entry point wrapper that initializes the minimal Stage-0 runtime.




**`0.5.0.4`** **- The Differential Gate (****`1.0.0.0`** **Release Gate)**

- **Location:** `tests/differential/`.



- **Logic:** Test script compiles `tests/corpus/stage0/math.omni`.



  1. Executes it in `omni-machine`. Captures stdout and exit code.



  2. Compiles it via `omni-codegen`. Executes native binary. Captures stdout and exit code.



  3. Asserts `Machine == Native`.



- **Result:** Achievement of `1.0.0.0` — Minimal Rust Core.




## 11. Hyper-Granular Implementation Ledger: Epoch 2 (Full Rust Production)

**Version Range:** **`1.0.0.1`** **→** **`2.0.0.0`**

### 11.1: Polonius, Traits, and Effects (`1.1.x.x` -> `1.3.x.x`)

**`1.1.0.1`** **- Chalk-Style Trait Solver**

- **Logic:** Implement SLG (Selective Linear Definite Clause Resolution) solver for traits. Map `impl T for U` to logic facts. Query `IsImplemented(T: Trait)`.




**`1.2.0.1`** **- Polonius Fact Generation**

- **Logic:** Extract CFG points from MIR. Emit `.facts` files (or in-memory equivalents): `borrow_region`, `killed`, `outlives`. Pass to `polonius-engine`. Parse result to determine lifetime errors.



- **Linkage:** `#[implements("OWN-0005")]`.




**`1.3.0.1`** **- Effect Row Solver**

- **Logic:** Extend Type Solver. `EffectRow = Open(Vec<Effect>) | Closed(Vec<Effect>)`.



- **Linkage:** `#[implements("EFF-0001")]`.




**`1.3.0.2`** **- State-Machine Lowering for Continuations**

- **Logic:** If a function has `async` or an unhandled custom effect, lower it into a struct representing a state machine (similar to Rust futures). Variables live across suspension points are transformed into fields of this struct.




**`1.3.0.3`** **- Structured Concurrency Enforcement**

- **Logic:** The `spawn_scope` block desugars into a closure. The AST-to-MIR lowering injects a mandatory `JoinAll` terminator at the end of the block, preventing the scope from exiting until all spawned state machines complete.



- **Linkage:** `#[implements("CONC-0003")]`.




### 11.2: LLVM Backend & Translation Validation (`1.4.x.x`)

**`1.4.0.1`** **- LLVM IR Emission**

- **Dependencies:** `inkwell`.



- **Logic:** Translate verified MIR into LLVM IR. Pass Target Data Layouts.




**`1.4.0.2`** **- The Translation Validator**

- **Location:** `compiler/omni-verify/src/llvm_val.rs`.



- **Logic:** Hook into LLVM's Optimization Pass Manager. Before and after passes (e.g., Loop Vectorization), inspect the LLVM IR. Ensure that instructions guarded by `assume.use` metadata have not been hoisted above `assume.invalidate` barriers.




### 11.3: Hermeticity, Packages, and Wire (`1.5.x.x` -> `1.7.x.x`)

**`1.5.0.1`** **- PubGrub Resolver & TUF Verification**

- **Location:** `tools/omni-pkg/`.



- **Dependencies:** `pubgrub`, `ed25519-dalek` (for TUF signatures).



- **Logic:** Read snapshot metadata. Verify ED25519 signatures against pinned root keys. Execute PubGrub SAT solver over the verified dependency graph.




**`1.5.0.2`** **- Comptime Capability Sandbox**

- **Logic:** Inside `omni-machine`, intercept `fs::read` operations. Check against `omni.toml` capability allowlist. If allowed, append file hash to `CacheKey::comptime_deps`.




**`1.6.0.1`** **- Fearless FFI Stack Switching**

- **Location:** `runtime/omni-platform/src/ffi.rs`.



- **Logic:** Implement ASM stubs utilizing `sigaltstack` (POSIX) or Fibers (Windows). Before calling `extern "C"`, swap the stack pointer to an isolated 2MB block. Place a trap handler at the base. If C++ throws an exception, the trap handler transitions the state to `Isolate` and safely unwinds the *Omni* stack cleanly.




**`1.7.0.1`** **- Distributed State Hydration**

- **Location:** `std::distributed::hydrate`.



- **Logic:** Byte parser for `OMW1` format. Instantiate fresh memory regions. Parse capability requirements. Call the local node's `AuthorityManager::request()`. If denied, return `Err(HydrationFault)`.



- **Linkage:** `#[implements("AUDIT-DIST-0005")]`.




**`Release Gate: 2.0.0.0`**

Complete Rust Production Platform. The compiler is feature-complete, highly optimized, and mathematically verified.

## 12. Hyper-Granular Implementation Ledger: Epoch 3 (Omni-in-Omni Severance)

**Version Range:** **`2.0.0.1`** **→** **`3.0.0.0`**

### 12.1: Rebuilding the Foundation in Omni (`2.1.x.x` -> `2.3.x.x`)

**`2.1.0.1`** **- The Omni Arena**

- **Language:** Omni.



- **Logic:** Write the bump allocator for the AST in pure Omni, using `Arena<T>` and `Gen<T>`.




**`2.2.0.1`** **- The Omni Lexer & Parser**

- **Language:** Omni.



- **Logic:** Rewrite `0.1.0.x` and `0.2.0.x` in Omni. Use Omni's explicit error sets (`error set ParseErr { ... }`) to handle malformed grammar without panics. Use `comptime` to generate the Pratt parsing tables at compile time.




**`2.3.0.1`** **- The Omni Type & Effect Solver**

- **Language:** Omni.



- **Logic:** Implement the Union-Find algorithm using Omni's linear types to guarantee that inference scopes are rigorously opened and closed, shifting logic bugs into compile-time errors.




### 12.2: Rebuilding the Backend & Validating DDC (`2.4.x.x` -> `2.7.x.x`)

**`2.4.0.1`** **- Omni MIR & Codegen**

- **Language:** Omni.



- **Logic:** Implement MIR generation. Write Fearless FFI wrappers over LLVM's C API to allow the Omni-written compiler to generate optimized machine code.




**`2.5.0.1`** **- The Dual Compiler Checkpoint**

- **Action:** Run `omni-rust build omni-compiler-source`. This produces `omni-compiler-v1.elf`.




**`2.6.0.1`** **- Zero-Dependency Severance**

- **Action:** Audit `omni-compiler-v1.elf`. Verify via `ldd` (or equivalent) that it does not link against `librustc`, `libstd` (Rust), or any hidden host runtimes. It must rely strictly on `OMNI-LIB-CORE` and `libc`/syscalls.




**`2.7.0.1`** **- Diverse Double Compilation (DDC)**

- **Action:**



  1. Run `omni-compiler-v1.elf build omni-compiler-source`. This produces `omni-compiler-v2.elf`.



  2. Run `omni-compiler-v2.elf build omni-compiler-source`. This produces `omni-compiler-v3.elf`.



- **Gate:** Execute `sha256sum omni-compiler-v2.elf omni-compiler-v3.elf`. The hashes MUST be bit-for-bit identical.




### Release Gate: `3.0.0.0` — Certified Self-Hosted Independent Omni

- **Deliverable:** An airtight, multi-paradigm, distributed-capable, AI-verifiable systems language platform. The Rust source code is formally archived. Omni is entirely self-sustaining.




*End of Master Plan.*

# Final Planning Amendment — Micro-Atomic Execution Contract

This amendment is the final planning-level interpretation of the three generational implementation stages. It does not authorize implementation by itself. It exists to make the execution ledger explicit enough that implementation can proceed later without inventing missing sequencing, ownership, qualification, or trust rules.

## A. Authoritative Generational Boundaries

The implementation lifecycle is fixed as:

```text
Stage 1: 0.0.0.1 -> 1.0.0.0
Rust-only Minimal Working Omni Core

Stage 2: 1.0.0.1 -> 2.0.0.0
Rust-only Complete Production Omni Platform

Stage 3: 2.0.0.1 -> 3.0.0.0
Omni-written Compiler -> Self-compilation -> DDC -> Dependency Severance
```

The first release gate does not claim full Edition-1 implementation. The second release gate does not claim self-hosting. The third release gate does not become valid until self-hosting, independent rebuild, DDC, and final dependency qualification all pass.

## B. Stage 1 Micro-Atomic Contract

Stage 1 proceeds through the following mandatory minor sequence:

```text
0.0.0.x  Foundation, schemas, repository infrastructure
0.0.1.x  Source normalization and lexer
0.0.2.x  Lossless CST, parser, AST
0.0.3.x  Names, scopes, DefId, minimal type system
0.0.4.x  Ownership, move lattice, drops
0.0.5.x  HIR, MIR, verifier
0.0.6.x  Reference abstract machine
0.0.7.x  Native code generation and vertical slice
```

The foundation patch sequence is fixed as the implementation-contract starting ledger:

```text
0.0.0.1  Workspace topology and global safety/lint contract
0.0.0.2  Deterministic Rust toolchain pinning
0.0.0.3  CI Reaper, quality controls, hard timeout/fuzz budgets
0.0.0.4  Normative rule registry schema and lifecycle locks
0.0.0.5  Specification manifest and specification-tree identity
0.0.0.6  Deterministic canonicalization engine
0.0.0.7  Semantic linkage Reaper / omni-audit
0.0.0.8  Conformance and diagnostic schemas
0.0.0.9  Downward-only compiler crate topology
0.0.0.10 Specification model loader and integrity checking
0.0.0.11 Stage-0 feature predicate infrastructure
0.0.0.12 Cryptographic provenance record generation
0.0.0.13 Foundation integration/release gate
```

No language implementation is permitted to appear in the foundation merely to make later milestones easier. The foundation establishes the deterministic containment boundary first.

## C. Foundation Quality Contract

The mandatory CI ceilings are:

```text
Pull request      1200 s
Main branch       1800 s
Nightly           3600 s
Qualification     7200 s
```

The fuzzing budgets are:

```text
Pull request       60 s / target
Main branch       300 s / target
Nightly           900 s / target
```

A timeout, hang, crash, sanitizer failure, nondeterministic outcome, or semantic discrepancy is a failure. A minimized fuzz failure becomes a deterministic regression case.

The exact toolchain used for an implementation generation must be pinned during implementation kickoff and recorded with its complete release identity. A future toolchain version is not to be hard-coded merely because it is currently available during planning.

## D. Rule Registry and Lifecycle Lock

The registry lifecycle is exactly:

```text
Proposed -> Candidate -> Ratified -> Deprecated
```

Required rules:

- `Proposed` rules may exist as planning/specification material but must not be release-qualified implementation-owned rules.
- `Candidate` rules may be used only in explicitly provisional implementation/conformance paths.
- `Ratified` rules are the ordinary release-qualified implementation authority.
- `Deprecated` rules reject new implementation ownership.
- `Ratified` rules require at least one resolvable witness test.
- References to unknown, removed, renamed, or deprecated rules are hard failures.
- The schema, the rule registry, the implementation linkage, and the witness suite must agree.

The CI Reaper must therefore reject at minimum:

```text
unknown rule linkage
Proposed implementation linkage
Deprecated implementation linkage
missing implementation coverage for required rules
missing witness coverage for Ratified rules
invalid dependency graph
stale semantic ownership metadata
```

## E. Canonicalization Contract

`omni-canon` must operate on a precisely defined canonical semantic representation rather than relying on incidental serializer behavior.

For semantic hashing it must:

- remove non-semantic source spans and source-position metadata;
- normalize permitted line-ending differences;
- canonicalize path representation through `/omni-root/`;
- remove host-specific absolute checkout locations from semantic identities;
- define deterministic object/member ordering;
- define deterministic numeric representation where the encoded domain permits multiple representations;
- define deterministic text encoding;
- reject unresolved host-specific identity inputs rather than silently accepting them;
- emit a single deterministic digest representation.

A comment-only, whitespace-only, source-location-only, or checkout-path-only change must not change a semantic hash when the affected information lies outside the semantic hashing domain. A semantic input change must change the relevant semantic digest.

## F. Stage 1 Language Pipeline

The first complete semantic pipeline is:

```text
raw source bytes
 -> source normalization
 -> UTF-8 validation
 -> tokens/trivia/spans
 -> lossless CST
 -> typed AST
 -> names/DefId
 -> Stage-0 type system
 -> ownership/lifetime checks
 -> HIR
 -> canonical MIR
 -> MIR verification
 -> reference abstract machine
 -> target-neutral lowering
 -> native backend
 -> minimal runtime
 -> executable artifact
```

The reference abstract machine is the primary semantic execution oracle. Native execution must agree with it over the qualified observation set.

## G. Stage 2 Production Contract

Stage 2 is entirely Rust-engineered. No subsystem is to be rewritten in Omni during the Stage-2 production generation.

The production sequence must cover, at minimum:

```text
advanced generics and trait system
refinement/constraint solving
complete ownership and memory model
Polonius integration
algebraic effects
capability authority
continuations and handler machinery
structured concurrency and async cleanup
unsafe-assumption dependency tracking
optimization legality and translation validation
production backend(s)
hermetic comptime/build system
package/update/supply-chain security
standard library
ABI/object/link support
Fearless FFI
canonical wire format
persistence/distributed execution
formatter/LSP/compiler API
complete conformance and qualification
```

Every Stage-2 subsystem must pass the full change-impact process: implementation, integration, regression testing, semantic linkage, applicable model/reference validation, reproducibility, security review, documentation synchronization, and release qualification.

## H. Backend and LLVM Governance

LLVM is not part of the Stage-1 foundation trust base simply because it may be useful for production optimization.

When introduced in Stage 2, the plan must lock:

```text
exact LLVM source/release identity
exact wrapper/binding identity
build configuration
target configuration
optimization pipeline
feature flags
provenance
SBOM/TCB status
translation-validation coverage
```

Backend behavior remains an implementation mechanism, never an alternative semantic authority.

## I. Stage 3 Compiler Parity Contract

Stage 3 is not satisfied by writing a new compiler driver in Omni.

The Omni implementation must reproduce the essential compiler responsibilities:

```text
source management
lexer/tokenizer
CST
parser
AST/HIR
name resolution
semantic/type analysis
ownership/lifetime analysis
effect/capability analysis
MIR construction
MIR verifier
reference abstract machine/interpreter
backend/code generation
runtime support required by the compiler
build/package orchestration
diagnostics
formatter/compiler tooling required for ordinary development
conformance infrastructure
bootstrap/reproduction tooling
```

Parity is judged by observable semantic and qualification equivalence. Internal algorithms and data structures may differ where that difference is explicitly permitted and independently validated.

## J. Self-Hosting Is Not Yet Independence

The initial Stage-3 compiler may be compiled by the qualified Stage-2 Rust compiler. This creates an Omni-written compiler but does not yet prove independence.

The required progression is:

```text
Stage-2 Rust compiler
      |
      v
Omni-written compiler A
      |
      v
compiler A builds compiler B
      |
      v
compiler B builds compiler C
      |
      v
independent rebuilds / DDC / dependency audit
      |
      v
ordinary build path no longer requires Rust
```

Every external dependency must be classified as:

```text
final permitted platform primitive
final permitted interoperability dependency
temporary bootstrap dependency
verification-only dependency
forbidden final dependency
```

A dependency does not count as removed merely because it disappears from a manifest. Final qualification must inspect transitive dynamic linkage, generated code, build helpers, embedded runtimes, host invocations, and hidden generators.

## K. DDC Contract

DDC is a qualification process rather than a single binary hash comparison.

At minimum it must establish:

1. independent bootstrap inputs;
2. independently tracked compiler/source/spec identities;
3. repeated self-compilation;
4. output convergence under the declared equivalence relation;
5. independent rebuild evidence;
6. complete TCB inventory;
7. unexplained divergence as a release blocker.

Byte-for-byte identity is required wherever the final artifact contract explicitly requires it. Where platform metadata makes binary identity intentionally impossible, the plan must define the stronger semantic/canonical equivalence relation in advance rather than weakening it after observing a mismatch.

## L. Multi-Paradigm Objective

The language is intended to be broadly useful across programming methodologies without splitting into dialects. The implementation should support, where covered by the normative specification:

```text
Functional
Object-oriented / trait-based
Systems / low-level
Data-oriented
Concurrent / asynchronous
Actor / reactive
Declarative
Metaprogramming / comptime
Generic / constraint-oriented
```

All paradigms must ultimately respect the same:

```text
type model
ownership/lifetime model
effect model
capability authority
memory model
MIR verification
provenance rules
determinism rules
```

Adding another methodology must not create a bypass around those foundations.

## M. Feature-Completeness Rule

A feature is not complete merely because its syntax is accepted.

The implementation ledger for every substantial feature must account for:

```text
lexical form
syntax/CST
AST/HIR representation
name resolution
static semantics
ownership/lifetime effects
effect/capability implications
MIR representation
verification
reference execution
backend lowering
runtime support
diagnostics
conformance
fuzz/property testing where applicable
differential testing where applicable
reproducibility impact
documentation
compatibility/migration impact
```

This becomes the mandatory completion template for future implementation patches.

## N. Final Planning State

The blueprint is considered planning-complete only when:

- the complete Omni specification remains the sole semantic authority;
- the three implementation stages are version-locked;
- the Rust-only boundaries are explicit;
- the Stage-3 Omni rewrite is a complete compiler/toolchain construction rather than a driver rewrite;
- self-hosting and final independence are separate qualification states;
- DDC and transitive TCB auditing are explicit;
- semantic linkage and witness obligations are mechanically enforceable;
- canonicalization and provenance are deterministic;
- CI/fuzz budgets are mandatory;
- obsolete implementation paths have an explicit disposition rule;
- every milestone has implementation, integration, regression, and qualification requirements;
- no implementation-local semantic invention is allowed.

The execution authorization remains separate from the existence of this plan.

```text
PLAN COMPLETE
      !=
IMPLEMENTATION STARTED
```

The first implementation action, when explicitly authorized, is:

```text
OMNI-IMP-0.0.0.1
```

No earlier or alternative implementation starting point supersedes this contract.

---

# Amendment V — Vibe-First Surface Language Integration

**Status:** Revised implementation directive. This amendment supersedes earlier surface-syntax implementation wording wherever it conflicts with Candidate 2 Vibe Surface Syntax. It does not weaken the semantic core or release gates.

## V.1 New implementation principle

Omni's surface language is now treated as a first-class implementation concern. The parser/compiler SHALL support a low-ceremony, AI-friendly surface while retaining the same typed, ownership-safe, effect-aware, capability-secure semantic core.

The implementation architecture SHALL therefore contain an explicit:

```text
Vibe Surface
    -> Surface CST/AST
    -> Deterministic Desugaring
    -> Core Semantic AST/HIR
```

boundary.

## V.2 Existing versions affected

The earlier `0.0.2.x` parser/CST milestone is expanded. It is not sufficient to implement only a Rust-like brace/semicolon grammar. The milestone must include the Candidate 2 surface contract before it can be closed.

Affected areas:

- `0.0.1.x` lexer: complete Candidate 2 token vocabulary needed by the parser;
- `0.0.2.x` parser/CST: newline continuation, command calls, pipelines, interpolation, projection shorthand, recovery;
- `0.0.2.x` formatter/layout: canonical vibe formatting;
- `0.0.2.x` surface AST: node forms for all vibe constructs;
- `0.0.2.x` desugaring boundary: deterministic surface-to-core lowering;
- `0.0.3.x` names/types: semantic checking of desugared forms;
- later ownership/effects/concurrency stages: preservation tests through desugaring.

## V.3 New micro-gate sequence

Before closing the parser family:

```text
V0 — token vocabulary
V1 — lossless layout/newline model
V2 — core declarations and expressions
V3 — vibe calls and pipelines
V4 — projection shorthand
V5 — optional chaining/coalescing
V6 — resource/concurrency surface
V7 — surface AST
V8 — deterministic desugaring
V9 — CST/AST/desugaring golden corpus
V10 — semantic integration
V11 — formatter canonicalization
V12 — parser/conformance qualification
```

Each gate requires implementation + tests + integration evidence.

## V.4 Parser architecture requirement

Use parser events or an equivalent immutable tree-building architecture. Do not mutate already-emitted Rowan subtrees to retrofit nesting.

Pratt parsing must produce actual nested expression structure.

Newline handling must be a parser/token-stream concern with explicit continuation-token rules, not a collection of ad hoc string tests.

## V.5 Surface-to-core contract

Every vibe construct must have:

- one parse;
- one canonical CST form;
- one deterministic surface AST representation;
- one deterministic desugaring;
- semantic preservation tests;
- diagnostics;
- formatter behavior;
- documentation;
- compatibility coverage.

## V.6 Version advancement rule

No `0.0.2.x` completion claim may be made until the Candidate 2 surface corpus passes.

No `0.0.3.x` names/types completion claim may be made until name/type behavior has been verified on both explicit core syntax and vibe syntax after desugaring.

No higher milestone may claim completion if it breaks a valid Candidate 2 surface program or changes its desugaring semantics.

## V.7 Documentation update rule

For every completed V-gate update:

1. the complete specification companion;
2. the master implementation plan status;
3. the syntax conformance corpus;
4. parser/CST design documentation;
5. semantic/desugaring documentation;
6. changelog;
7. milestone evidence;
8. compatibility matrix.

## V.8 Candidate 2 qualification gate

Candidate 2 surface syntax becomes implementation-qualified only after:

- full workspace build/check passes;
- complete workspace tests pass;
- Clippy with `-D warnings` passes;
- formatting passes;
- syntax corpus passes;
- negative/recovery corpus passes;
- CST round-trip passes;
- desugaring goldens pass;
- affected semantic tests pass;
- deterministic repeated runs agree;
- documentation is synchronized;
- the exact candidate/manifest hashes are recorded;
- the milestone is committed;
- the authoritative Git remote is synchronized.
