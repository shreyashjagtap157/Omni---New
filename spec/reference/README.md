# Omni Specification — Exhaustive Reference and Architecture Compendium

This directory serves as the exhaustive reference collection within the `spec/` root, translating and consolidating the normative Edition 1 baseline and Candidate 2 Vibe-First specification sections into structured architectural documentation.

## Normative Specification Root & Cryptographic Boundary

The Omni language repository strictly isolates normative specification artifacts that participate in the canonical tree digest (`spec_tree_sha256`) from auxiliary explanatory material:

- **Normative Hashed Domain (`INCLUDED_DIRS`)**:
  - `grammar/` (`omni-edition1.ebnf`, `candidate2-erratum.md`)
  - `registry/` (`rules.json`, `rule-texts.json`)
  - `schemas/` (`*.schema.json`)
  - `models/` (reserved; currently empty)
  - `data/` (reserved; currently empty)
- **Manifest Domain (Self-Reference Excluded)**:
  - `manifest/` (`omni-edition1.manifest.json`)
  - `release/` (`foundation-gate.json`)
- **Exhaustive Reference Domain**:
  - `reference/` (Detailed architectural syntheses, formal rules, type systems, and surface-to-core lowering specifications).

## Document Structure

1. [Candidate 2 Vibe-First Surface Syntax Specification](file:///c:/Users/siddh/Downloads/ABC/Omni/spec/reference/candidate2_vibe_surface_syntax.md)
   - Detailed treatment of Sections A through R of the Candidate 2 Amendment.
   - Deterministic newline continuation and statement termination rules (`VIBE-GRAM-0001` through `VIBE-GRAM-0007`).
   - Pipeline operator semantics (`|>`), left-associativity, and evaluation contracts.
   - Projection shorthand (`.field`) and desugaring closures.
   - Command-style call disambiguation.
   - Resource scopes (`with`) and structured concurrency (`parallel`).
   - Explicit capability declarations (`requires`).
   - Expression-orientation, compact closures, and AI-generation invariants.

2. [Core Language Semantics and Type System Specification](file:///c:/Users/siddh/Downloads/ABC/Omni/spec/reference/core_semantics_and_types.md)
   - Module breakdown across `OMNI-STD-ROOT`, `OMNI-TERMS`, `OMNI-NAMES`, `OMNI-TYPES`, and `OMNI-OWN`.
   - Affine ownership model, borrowing, region parameters, and drop mechanics.
   - Static type system: primitives, tuples, arrays, ranges, function signatures, user structs, and generic parameterization.
   - Type inference, unification solver, and constraint satisfaction.
   - Type-directed multi-instantiation monomorphization contracts.
   - Unified TraitSystem: declarations, required methods, fatal registration diagnostics, and supertraits.
   - Strict coherence four-quadrant matrix and orphan rule validation (`trait.is_local || target_ty.is_local`).
   - SLG resolution engine for positive (`T: Trait`) and negative (`T: !Trait`) obligations.
   - Generic substitution (`SubstEnv`) and concrete trait obligation solving pipeline.
   - Strict concrete semantic gate before MIR lowering (`assert_concrete_for_mir`).

