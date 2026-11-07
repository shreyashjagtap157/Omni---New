---
trigger: model_decision
description: Use when investigating new language features, evaluating theoretical foundations, designing experiments, assessing memory architectures, or researching vibe-first ergonomics.
---

# Omni Research-Driven Development Methodology

Omni is developed through continuous scientific inquiry rather than ad-hoc feature accumulation. Every non-trivial capability or semantic modification must undergo systematic evaluation.

## 1. The Twelve-Step Research Protocol

For every significant language feature, type-system rule, or architectural proposal:
1. **Problem Formulation**: Precisely define the developer problem or language gap being addressed.
2. **Fundamental Requirements**: Identify invariants, safety boundaries, and ergonomic goals across expression tiers.
3. **Prior Art & Literature**: Survey existing research, academic papers (POPL, PLDI, ICFP, OOPSLA), and prior language solutions (Rust, Haskell, OCaml, Koka, Eff, Cyclone, Austral, Vale, ATS).
4. **Comparative Analysis**: Compare how mainstream and research compilers handle similar challenges.
5. **Alternative Candidate Designs**: Formulate at least two distinct design strategies; never settle on an initial idea without exploring alternatives.
6. **Theoretical Foundations**: Analyze mathematical properties (type soundness, progress/preservation, termination, operational semantics).
7. **Implementation Complexity**: Evaluate compiler overhead, lowering complexity, and intermediate representation impact.
8. **Subsystem Interactions**: Identify side-effects on ownership, borrow-checker, effect handling, type inference, monomorphization, and code generation.
9. **Performance, Safety & Portability**: Quantify runtime overhead, memory footprint, cache implications, and target compatibility.
10. **Testable Acceptance Criteria**: Formulate falsifiable criteria (success and failure conditions).
11. **Document Assumptions & Open Questions**: Explicitly list unknown variables and potential failure points.
12. **Documented Selection Justification**: Record the architectural decision with explicit rationale in an ADR.

## 2. Experimental Implementation as an Instrument

Prototypes and executable models are instruments to challenge hypotheses:
- **Design Falsification**: Design experiments capable of disproving the feature's viability.
- **Negative Results**: Failed experiments and rejected hypotheses must be documented with root-cause analysis—never hidden or silently discarded.
- **Testing Instruments**:
  - Property-based testing (fuzzing invariants across generated ASTs).
  - Metamorphic and differential testing (cross-checking against reference evaluators).
  - Counterexample generation for type rules and borrow invariants.
  - Resource and throughput benchmarking.

## 3. Investigating Vibe-First Ergonomics & Memory Models

- **Vibe-First Syntax Research**:
  - Evaluate how to reduce ceremony while guaranteeing 100% formal determinism.
  - Rigorously test ambiguity, parsing complexity, error recovery, and formatting.
  - Ensure the compiler never uses heuristic natural language interpretation to determine program semantics.
- **Memory Architecture Research**:
  - Preserve the open architectural tension between managed memory and affine ownership models.
  - Research hybrid models (regions, deferred reference counting, capability-bounded heaps, profile-guided memory).
  - Evaluate ergonomics, safety, predictability, and real-time execution across hardware targets.

## 4. Evidence Classification Taxonomy

All claims, reports, and documentation must classify status using the standard taxonomy:
- `VERIFIED`: Directly established by reproducible automated tests or qualification runs.
- `PROVEN`: Machine-checked or rigorously established within formal mathematical assumptions.
- `EXPERIMENTALLY_VALIDATED`: Confirmed by dedicated empirical prototypes under specified conditions.
- `IMPLEMENTED_UNVERIFIED`: Code is committed but lacks sufficient qualification evidence.
- `PROPOSED`: Architectural or language proposal not yet experimentally vetted.
- `HYPOTHESIS`: Working assumption under active investigation.
- `PARTIALLY_VALIDATED`: Some properties proven/tested, but edge cases or integrations remain open.
- `REJECTED`: Formally analyzed and dismissed, with recorded justification.
- `BLOCKED`: Work halted due to missing dependencies, tooling, or external constraints.
- `OPEN`: Unresolved question awaiting study or authoritative developer decision.

## 5. Bridging Omni-Research & the Compiler Implementation

The workspace acknowledges the independent `Omni-Research` repository located at sibling directory `../Ideas/Omni-Research`:
- `Omni-Research` houses the 17-module `MASTER-SPEC.md` and `STATE.md`, exploring long-term calculi, proof assistant comparisons, and theoretical constitutions.
- The active compiler in this repository implements Edition 1.
- Insights from `Omni-Research` flow into the compiler only through the formal specification governance process.
