---
trigger: model_decision
description: Use when authoring, modifying, or refactoring Rust code, adjusting workspace package boundaries, managing dependencies, or updating Cargo.toml in Omni.
---

# Omni Rust Engineering & Architecture Guidelines

The Omni compiler is engineered in Rust under uncompromising standards of memory safety, deterministic builds, and modular architecture.

## 1. Rust Safety & Workspace Lint Configuration

- **Workspace Lint Directives**: Root `Cargo.toml` mandates:
  ```toml
  [workspace.lints.rust]
  unsafe_code = "forbid"

  [workspace.lints.clippy]
  all = "deny"
  ```
  - All 25 workspace packages explicitly inherit these workspace lints via `[lints] workspace = true` in their manifests (verified by workspace audit).
  - While this is enforced via Cargo workspace lint inheritance rather than requiring manual source-level `#![forbid(unsafe_code)]` in every crate, the prohibition of unsafe code is absolute under current policy. No unsafe code is permitted without a formally approved policy and repository configuration change.
- **Strict Clippy Policy**:
  - All Clippy warnings are hard errors (`-D warnings`). Code must pass `cargo clippy --workspace --all-targets --locked -- -D warnings` with zero warnings.
- **Pinned Toolchain**:
  - Respect `rust-toolchain.toml` (pinned to Rust 1.95.0).
  - Do not silently substitute toolchain versions. Stable qualification gates must remain decoupled from nightly-only fuzzers/sanitizers.
- **Formatting**:
  - Code must strictly conform to `.rustfmt.toml`. Run `cargo fmt --all -- --check`.

## 2. Workspace Package Structure (25 Packages)

The Omni workspace is composed of 25 packages (18 compiler crates + 7 tooling crates) discovered via `cargo metadata`:

### Compiler Subsystem (18 Packages)
- `compiler/omni-source`: Source maps, span tracking, and diagnostic spans.
- `compiler/omni-unicode`: Unicode categorization and normalization.
- `compiler/omni-traits`: Shared compiler interfaces and fundamental traits.
- `compiler/omni-lex`: Lexical analysis and token stream generation.
- `compiler/omni-syntax`: Concrete syntax tree (CST) and token definitions.
- `compiler/omni-parse`: Recursive descent parser and AST construction.
- `compiler/omni-names`: Lexical scoping, name binding, and symbol tables.
- `compiler/omni-types`: Type system, unification, subtyping, and inference.
- `compiler/omni-own`: Linear/affine ownership, borrowing, and region tracking.
- `compiler/omni-effects`: Algebraic effect tracking and handlers.
- `compiler/omni-hir`: High-level intermediate representation and AST lowering.
- `compiler/omni-mir`: Mid-level control-flow graph (CFG) representation.
- `compiler/omni-verify`: Formal verification, SMT/predicate engines, and MIR checks.
- `compiler/omni-machine`: Target machine abstraction and instruction selection.
- `compiler/omni-codegen`: Native machine code emission (Cranelift integration).
- `compiler/omni-runtime`: Minimal runtime primitives, memory layout, and runtime support.
- `compiler/omni-driver`: Compiler driver, CLI orchestrator, and diagnostics engine.
- `compiler/omni-stage0`: Bootstrap compiler stage 0 harness.

### Tooling Subsystem (7 Packages)
- `tools/omni-canon`: Specification and grammar canonicalization.
- `tools/omni-audit`: Crate and dependency safety auditor.
- `tools/omni-registry`: Rule registry validator and schema checker.
- `tools/omni-conform`: Conformance test runner and harness.
- `tools/omni-bindgen`: Interoperability binding generator.
- `tools/omni-evidence`: Qualification evidence generator and tracker.
- `tools/omni-topology`: Workspace dependency topology analyzer.

> **Architectural Invariants**:
> - Preserve separation of concerns. Never introduce circular dependencies.
> - Downstream crates must never enforce invariants that belong to upstream representations.
> - Do not duplicate semantic checks across crate stages.

## 3. Dependency & Lockfile Hygiene

- Always run builds and tests with `--locked`:
  ```pwsh
  cargo test --workspace --locked
  ```
- **Never mutate `Cargo.lock` silently or implicitly**. Any dependency bump or addition must be deliberate, isolated, documented, and tested.
- Evaluate crate dependencies for security advisories, transitive bloat, and license compatibility.
