---
name: omni-research-to-spec
description: >-
  Use this skill to research, theoretically analyze, experimentally prototype,
  and formalize candidate language features into normative Omni specifications.
---

# Omni Research-to-Specification Methodology

This skill governs the systematic process of transforming an initial language concept or requirement into an experimentally validated, normative specification clause for Omni.

## 1. The Eighteen-Stage Research-to-Spec Pipeline

Whenever a new language feature, syntax proposal, memory model, or semantic capability is explored:

```
[Problem & Requirements]
         │
         ▼
[Prior Art & Survey]
         │
         ▼
[Theoretical Analysis] ──► [Alternative Candidates (≥2)]
         │                              │
         ▼                              ▼
[Feasibility & Cost]  ◄─────────────────┘
         │
         ▼
[Candidate Formalization]
         │
         ▼
[Experimental Prototype / Model]
         │
         ▼
[Counterexample & Falsification Testing]
         │
         ▼
[Evaluation & Selection]
         │
         ▼
[Architecture Decision Record (ADR)]
         │
         ▼
[Normative Spec Integration & Grammar]
         │
         ▼
[Executable Conformance Suite]
         │
         ▼
[Compiler Implementation & Qualification]
```

### Phase 1: Problem Formulation & Prior Art
1. **Define Problem**: What specific limitation, expressive challenge, or safety guarantee motivates this?
2. **Decompose Requirements**: Extract non-negotiable invariants, ergonomic goals, and performance bounds.
3. **Literature Survey**: Investigate established research (e.g. POPL, PLDI, ICFP, OOPSLA literature on type systems, borrow checking, effect handlers, linear logic).
4. **Language Comparisons**: Document how Rust, Haskell, OCaml, Koka, Cyclone, Austral, Vale, or Swift handle this problem.

### Phase 2: Theory, Alternatives & Feasibility
5. **Formulate Alternatives**: Design at least two competing models with distinct tradeoffs.
6. **Theoretical Grounding**: Specify formal properties (type safety, progress, preservation, termination, soundness).
7. **Feasibility Assessment**: Evaluate impact on compiler performance, parser lookahead, intermediate representation (HIR/MIR), and runtime overhead.
8. **Subsystem Interactions**: Identify how the proposal interacts with ownership, borrow checker, algebraic effects, and generics.

### Phase 3: Prototyping, Falsification & Validation
9. **Draft Candidate Spec**: Write precision operational semantics and typing rules.
10. **Build Experimental Prototype**: Implement a minimal prototype, formal calculus (e.g. Omni-λ model), or isolated test stage.
11. **Falsification Experiments**: Create adversarial tests and counterexamples designed to break the design.
12. **Empirical Evaluation**: Benchmark compile-time cost, generated code quality, and developer ergonomics.
13. **Design Revision**: Refine rules based on failure modes or reject unviable approaches.

### Phase 4: Formalization & Integration
14. **Record Architecture Decision**: Produce an ADR in `docs/adr/`.
15. **Normative Specification Integration**: Update `spec/reference/` and `spec/manifest/`.
16. **Grammar Update**: If syntax is altered, update `spec/grammar/omni-edition1.ebnf`.
17. **Executable Conformance Suite**: Author positive and negative test cases in `conformance/`.
18. **Publish Research Findings**: Record sources, experimental logs, and evidence classification in project documentation.

## 2. Special Research Protocols

### Vibe-First Syntax Research Protocol
- Evaluate ergonomics across high-level, intermediate, and low-level expression tiers.
- Formally test grammar for ambiguity, LR(1)/LL(1) properties, lookahead requirements, and error recovery.
- **Invariant**: Strict determinism must be preserved; NLP or heuristic interpretation is permanently barred.

### Memory Model Research Protocol
- Investigate the open architectural tension between managed memory and affine ownership.
- Model hybrid approaches (regions, generational arenas, deferred RC, profile-guided allocation).
- Benchmark throughput, pause times, predictable destruction, and real-time suitability.

## 3. Research Provenance & Evidence Standards

Every proposal document must include:
- **Evidence Level**: `HYPOTHESIS` → `EXPERIMENTALLY_VALIDATED` → `VERIFIED` / `PROVEN` (or `REJECTED`).
- **Source Traceability**: Citations to papers, RFCs, and code repositories.
- **Reproducible Commands**: Scripts or commands used to run experimental evaluations.
- **Explicit Limitations**: Stated boundary conditions where the design does not hold or requires future work.
