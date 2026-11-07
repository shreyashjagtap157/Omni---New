---
name: omni-spec-conformance
description: >-
  Use this skill when implementing, refactoring, or verifying compiler features against
  the normative Omni language specification (spec/) and conformance tests (conformance/).
---

# Omni Specification-to-Implementation Conformance Workflow

This skill guides the end-to-end process of taking an Omni language requirement, navigating Candidate 1/2/3 governance, implementing it across compiler crates, and establishing executable conformance evidence.

## 1. Specification Sourcing & Candidate Governance Hierarchy

Before writing code, establish normative authority:

1. **Check Ratified Foundation**:
   - Consult `spec/release/foundation-gate.json` and `spec/manifest/omni-edition1.manifest.json`.
   - **Candidate 1 (`1.0.0-candidate.1`) is the active normative baseline**. Maintain candidate lifecycle status without premature claims of release sign-off.
2. **Evaluate Candidate 2 & 3 Status**:
   - `spec/grammar/candidate2-erratum.md` is `status: pre-release`, `signature_state: pending`, with `implementation_impact: "none"`. It is **NOT in force** for Edition 1 baseline.
   - Appended `(* VIBE-GRAM extensions *)` in `spec/grammar/omni-edition1.ebnf` must be guarded under feature gates and cannot be assumed active in the baseline grammar contract.
   - Candidate 3 tests in `omni-parse` are experimental research probes.
3. **The No-Override Principle**:
   - If an existing compiler test, demo file, or parser method supports unratified syntax (such as pipeline `|>` or optional navigation `?.`), that implementation **does not grant normative authority**.
   - Do NOT mark conformance tests as passing for Edition 1 baseline based on unratified extensions.

## 2. Ambiguity & Conflict Handling Protocol

If a specification clause is ambiguous, silent, or contradictory:
1. **STOP**: Do not invent semantics or choose an arbitrary parser interpretation.
2. **Document Conflict**: Record the exact divergence between manifest, grammar, rule registry (`spec/registry/rules.json`), and compiler code.
3. **Draft Specification Erratum or ADR**: Prepare a formal delta under `docs/adr/`.
4. **Halt Dependent Conformance Claims**: Mark the feature as `OPEN` or `BLOCKED` until ratified.

## 3. Conformance Test Suite Authoring

Conformance tests in `conformance/` and `tests/` must precede or accompany implementation:
1. **Positive Tests**: Valid code examples asserting correct parsing, typing, and evaluation.
2. **Negative Tests**: Malformed syntax, invalid type unifications, borrow conflicts, or unhandled effects.
3. **Diagnostic Checks**: Verify exact error codes, primary spans, and help messages produced by `omni-driver`.

## 4. Compiler Stage Implementation Order (25 Packages)

Implement features sequentially through the compiler phase pipeline:
1. **Source & Tokens** (`omni-source`, `omni-unicode`, `omni-traits`, `omni-lex`):
   - Add new token kinds, keywords, or delimiters mandated by the ratified grammar.
2. **Syntax & AST** (`omni-syntax`, `omni-parse`):
   - Extend AST representations; implement recursive-descent parsing with robust error recovery.
3. **Names & Scoping** (`omni-names`):
   - Resolve identifier bindings, lexical scopes, and module exports.
4. **Types & Constraints** (`omni-types`):
   - Formulate typing judgements; implement bidirectional type checking and inference.
5. **Ownership & Borrowing** (`omni-own`):
   - Enforce affine/linear semantics, borrow scopes, and lifetime region splitting.
6. **Effects** (`omni-effects`):
   - Verify effect labels, handlers, and effect inference constraints.
7. **Lowering & Verification** (`omni-hir`, `omni-mir`, `omni-verify`):
   - Lower AST to HIR and MIR; run formal verification and predicate engine checks.
8. **Codegen & Runtime** (`omni-machine`, `omni-codegen`, `omni-runtime`):
   - Lower MIR to target machine representations (Cranelift integration).
9. **Driver & CLI** (`omni-driver`, `omni-stage0`):
   - Orchestrate pipeline and report diagnostics.

## 5. End-to-End Traceability & Formal Governance Audit

Before concluding work, verify that any semantic or syntax change has fulfilled the full repository governance cycle:
- [ ] Requirement mapped to spec clause in `spec/reference/`
- [ ] Governance Approval: Editing `omni-edition1.ebnf` alone does NOT make a syntax change normative; it requires corresponding updates to `spec/manifest/omni-edition1.manifest.json`, integrity hash (`spec_tree_sha256`), registry schemas in `spec/registry/rules.json`, formal errata if applicable, and release gate bindings in `spec/release/foundation-gate.json`.
- [ ] Conformance tests authored in `conformance/` covering positive and negative cases
- [ ] Compiler stages implemented under inherited workspace lints (`unsafe_code = "forbid"`)
- [ ] Zero Clippy warnings (`cargo clippy --workspace --all-targets --locked -- -D warnings`)
- [ ] Full locked workspace tests pass (`cargo test --workspace --locked`)
