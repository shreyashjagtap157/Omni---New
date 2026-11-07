---
name: omni-architecture-review
description: >-
  Use this skill to evaluate architectural consistency, crate boundaries, compiler
  lowering stages, language invariants, and prevent local shortcut designs in Omni.
---

# Omni Compiler Architecture Review Protocol

This skill enforces systemic architectural integrity across the entire Omni platform and compiler workspace, preventing local implementation shortcuts from degrading the overarching design.

## 1. Whole-Platform Architectural Integrity Checklist

When reviewing any proposed structural change, refactoring, or new subsystem, evaluate:

### A. Separation of Responsibilities & Crate Boundaries
- Does this change violate single-responsibility principles across the 25 workspace packages?
- Are upstream representations (e.g. `omni-syntax` or `omni-parse`) being burdened with downstream semantic checks (e.g. `omni-types` or `omni-own`)?
- Does this introduce circular dependencies or tighten coupling between independent crates?

### B. Intermediate Representation & Lowering Purity
- Does AST lowering to `omni-hir` preserve all necessary source spans and provenance?
- Does `omni-mir` maintain a clean control-flow graph (CFG) representation suitable for verification?
- Are formal verification obligations cleanly separated in `omni-verify` rather than tangled in code generation (`omni-codegen`)?

### C. Language Semantic & Type-Theoretic Invariants
- Are linear/affine ownership guarantees strictly maintained?
- Does borrow-checking allow context splitting without compromising soundness?
- Are effect scopes, handlers, and capability bounds explicitly accounted for in the evaluation semantics?
- Are error diagnostics deterministic, pinpointing exact spans without cascaded garbage errors?

### D. Memory Models, Concurrency & Execution Tiers
- Does the design align with the unresolved memory architecture inquiry (managed vs. affine ownership vs. hybrid regions)?
- How does the change affect asynchronous concurrency, actor isolation, and memory ordering?
- Does it maintain compatibility across the high-level, intermediate, and low-level expression tiers?

### E. Safety, ABI, FFI & Platform Portability
- Is `#![forbid(unsafe_code)]` / workspace `unsafe_code = "forbid"` strictly preserved without exceptions?
- Does the design avoid unbounded allocations or deep recursive stack overflows in compiler passes?
- Are ABI boundaries, foreign function interfaces (FFI), and machine representations platform-agnostic?

## 2. Review Deliverables & Documentation

An architecture review must produce:
1. **Tradeoff Analysis Matrix**: Comparing at least two structural alternatives.
2. **Failure Modes & Stress Scenarios**: Analysis of pathological inputs, cyclic references, or resource exhaustion.
3. **Dependency Impact Assessment**: Effect on compile times, transitive dependencies, and build graph determinism.
4. **Actionable Recommendations**: Clear approve/reject/revise verdict with specific required adjustments.
