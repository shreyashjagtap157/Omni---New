---
trigger: model_decision
description: Use when modifying language specifications, updating the EBNF grammar, interpreting language semantics, or resolving Candidate 1/2/3 governance ambiguities.
---

# Omni Specification Governance & Semantic Traceability

The normative specifications under `spec/` constitute the absolute authority for Omni Edition 1 compiler behavior.

## 1. Specification Authority & The No-Override Principle

- **Domain-Specific Authority Model**:
  1. **Normative Specification Artifacts (`spec/reference/`, `spec/grammar/`)**: Define authoritative language semantics, typing rules, and grammar.
  2. **Edition Manifest (`spec/manifest/omni-edition1.manifest.json`)**: Identifies and binds the selected specification tree snapshot and integrity digest (`spec_tree_sha256`).
  3. **Release Gates (`spec/release/foundation-gate.json`, `docs/foundation-gates.md`)**: Establish operational qualification criteria and release readiness. A release gate is an operational gating check, not a language specification.
  4. **Compiler Implementation & Test Suites**: Downstream verification instruments, not sources of normative authority.
- **The No-Override Principle**: Compiler source code, unit tests, code comments, example code, and unratified grammar fragments cannot silently override normative language authority.

## 2. Candidate 1, Candidate 2, and Candidate 3 Governance

The workspace contains multi-candidate artifacts that must be handled with strict precision:
- **Candidate 1 (Edition 1 Active Baseline: `1.0.0-candidate.1`)**: The active normative foundation declared by `spec/manifest/omni-edition1.manifest.json` and `spec/release/foundation-gate.json`. Maintain its candidate lifecycle status; do not describe it as definitively ratified or production-released without completed qualification gates.
- **Candidate 2 ("Vibe-First" Syntax Erratum)**:
  - Formally recorded in `spec/grammar/candidate2-erratum.md` as `status: pre-release`, `signature_state: pending`, and `implementation_impact: "none"`.
  - Documented in `docs/grammar-reconciliation.md` and `compiler/omni-parse/src/grammar_contract.rs` as **not in force**.
  - Appended productions labeled `(* VIBE-GRAM extensions *)` in `omni-edition1.ebnf` (pipeline `|>`, newline termination, projection shorthand) reflect this pre-release erratum and are not active in the baseline grammar contract without an explicit feature gate.
- **Candidate 3 (Exploratory Pipelines, Precedence & Navigation)**:
  - Artifacts include parser tests (`candidate3_pipeline_and_optional_navigation_parse` in `omni-parse`) and parser precedence definitions (`compiler/omni-parse/src/precedence.rs`).
  - These represent **experimental parser prototypes**, never proof of ratified Edition 1 semantics.

### Conflict Resolution Protocol
When specification, manifest, grammar, and compiler code conflict:
1. Document the exact conflict across rule IDs, file paths, and AST constructs.
2. Apply the domain authority model: normative spec defines semantics, manifest binds tree integrity, release gate checks criteria. Implementation code or tests never override normative specifications.
3. Block unsupported conformance claims; never alter manifests, hashes, or release digests to force artificial alignment.

## 3. Bidirectional Traceability Standard

Every semantic implementation in Omni must maintain an unbroken bidirectional chain:
$$\text{Requirement} \longleftrightarrow \text{Research Evidence} \longleftrightarrow \text{Hypothesis} \longleftrightarrow \text{ADR} \longleftrightarrow \text{Spec Clause} \longleftrightarrow \text{Grammar} \longleftrightarrow \text{Compiler AST/HIR/MIR} \longleftrightarrow \text{Conformance Test} \longleftrightarrow \text{Qualification Evidence}$$

When modifying or introducing a language construct:
1. Cite the exact rule ID from `spec/registry/rules.json`.
2. Map to corresponding grammar rules in `omni-edition1.ebnf`.
3. Locate or author the associated positive and negative conformance tests in `conformance/`.
4. Verify compiler parsing, type checking, and lowering across crate boundaries.
