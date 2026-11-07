---
trigger: always_on
description: Essential Omni project invariants, product vision, unbounded evolution, and core development philosophy.
---

# Omni Core Principles & Architecture Invariants

This rule establishes the permanent invariants governing development across the Omni compiler repository.

## 1. Project Identity & Unbounded Generational Roadmap

Omni is an advanced multi-paradigm programming language and computational platform designed for verified correctness, fine-grained ownership, algebraic effects, high-performance native execution, and unparalleled developer ergonomics ("Everything, but structured").

Omni recognizes **no predetermined final limit** on research, capability, optimization, or architectural enhancement. Its generational roadmap defines major milestones rather than a permanent stopping point:
- **Stage 1 (Current: `0.0.0.1` -> `1.0.0.0`)**: Minimal, functionally complete, verified working core implemented strictly in Rust.
- **Stage 2 (`1.0.0.1` -> `2.0.0.0`)**: Complete production-grade language platform in Rust with full optimization, standard libraries, and comprehensive tooling.
- **Stage 3 (`2.0.0.1` -> `3.0.0.0`)**: Self-hosting Omni written in verified Omni, with independent bootstrap qualification.
- **Continuous Evolution**: Ongoing adoption of superior type systems, runtime architectures, formal methods, and hardware targets.

> **Rule**: Never claim production readiness, full self-hosting, or formal completeness before the explicit qualification gate for that stage is verified with reproducible evidence.

## 2. Vibe-First Ergonomics with Strict Determinism

- Omni fundamentally seeks to eliminate unnecessary mechanical ceremony (repetitive type annotations, explicit ownership gymnastics, generic parameter duplication, error-handling boilerplate).
- Programming should feel natural, fluid, and intent-focused across high-level, intermediate, and low-level tiers.
- **Invariant**: Natural ergonomics must remain 100% deterministic and statically checkable. Natural language heuristics, fuzzy parsing, and probabilistic semantics are permanently forbidden.

## 3. Specification Ground Truth & Candidate Governance

- The normative files in `spec/` (specifically `spec/manifest/omni-edition1.manifest.json` and `spec/release/foundation-gate.json`) govern the Edition 1 Candidate 1 baseline.
- **The No-Override Principle**: Compiler source code, unit tests, code comments, and appended EBNF fragments cannot silently override normative specifications.
- Candidate 2 errata remain unratified (`status: pre-release`), and Candidate 3 tests represent experimental prototypes. Semantics must not be invented locally.
- The sibling `Omni-Research` initiative (in `../Ideas/Omni-Research`) explores idealized models; its hypotheses must not be treated as compiler ground truth until formally ratified in `spec/`.

## 4. Open Memory Architecture Invariant

- The tension between earlier managed-memory-by-default requirements and Edition 1 affine-ownership-by-default remains an active, open research domain.
- Do not make unilateral or silent architectural decisions; evaluate hybrid and profile-guided models through rigorous research.

## 5. Rust Safety & Determinism

- Workspace-level lints enforce `unsafe_code = "forbid"` and `clippy::all = "deny"`.
- Pinned toolchain: Rust 1.95.0 in `rust-toolchain.toml`.
- Builds, checks, and tests must use `--locked`.
