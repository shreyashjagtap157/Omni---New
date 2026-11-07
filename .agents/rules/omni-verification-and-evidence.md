---
trigger: model_decision
description: Use when running test suites, validating compiler correctness, classifying evidence, reviewing dependency advisories, or investigating environmental test blockers.
---

# Omni Verification, Testing & Evidence Standards

Omni rejects subjective claims of correctness. Passing tests do not imply language completeness, and an error-free compilation does not imply semantic soundness.

## 1. Test Suite Architecture & Coverage Matrix

Every language feature and compiler stage must be covered across multiple testing layers:
- **Positive Tests**: Valid language programs demonstrating correct evaluation, typing, and execution.
- **Negative Tests**: Programs intentionally violating syntax, types, borrow rules, or effect scopes. Verify exact diagnostic codes, spans, and error explanations.
- **Boundary & Pathological Cases**: Deep recursion, empty blocks, cyclic types, complex borrow splitting, edge arithmetic.
- **Diagnostic Spans**: Verify that error locations pinpoint the exact faulty token or expression.
- **Recovery & Robustness**: Verify that compiler parser and type checker recover gracefully and continue analysis without crashing.
- **Regression Tests**: Any fixed defect must include an isolated regression test referencing its issue or PR.

## 2. Test Layer Distinctions

Maintain explicit separation between:
1. **Unit Tests**: Targeted crate-internal logic (`cargo test -p omni-own --locked`).
2. **Integration Tests**: End-to-end pipeline execution from source string to MIR/machine output.
3. **Conformance Tests**: Executable tests located under `conformance/` validating normative spec clauses.
4. **Property-Based & Fuzz Tests**: Invariant generators testing parser resilience and verifier soundness.
5. **Formal Verification**: Proof-assistant models and SMT predicate engine validations.

## 3. Dissecting Production-Readiness and Formal Claims

Never conflate distinct technical milestones:
- A *syntactically functional parser* is not a complete compiler.
- A *passing unit test suite* is not a complete language implementation.
- A *working prototype* is not a production-ready toolchain.
- A *compiler using formal terms* (e.g. predicate engine) is not a formally verified compiler.
- A *bootstrapping prototype* is not an independently verified self-hosting language.

Do not assert formal claims without machine-checked proofs or verifiable evidence.

## 4. Handling Environmental Blockers & Dependency Advisories

### Environmental Blocker Protocol
When qualification checks cannot execute due to environment constraints:
- Missing toolchain binaries on PATH, missing target C linkers, unavailable MSVC/LLVM tools, or offline networks are **`BLOCKED: Environmental Dependency`** conditions, NOT passing tests.
- **Reporting Protocol**: Always state:
  1. Exact failing command.
  2. Exact shell output and error code.
  3. Environment context (OS, architecture, toolchain path).
  4. Root cause (code defect vs. missing system dependency).
  5. What was verified vs. what remains unverified.
  6. The exact command needed to resume when the dependency is satisfied.
- Never report a test suite as passed if tests were skipped, ignored, or blocked.
- Historical benchmark or test count numbers (e.g., 900+ tests in Stage0, 19/19 in omni-verify) must be independently re-executed and verified before being presented as current facts.

### Dependency Security Advisory Protocol
- When security vulnerabilities are reported (e.g. Issue #46 regarding `ruzstd 0.7.3`):
  1. Inspect the live lockfile (`Cargo.lock`) and verify reverse dependency paths via `cargo tree -i ruzstd --locked`.
     - Direct & Transitive chains: `omni-codegen` -> `cranelift-object 0.110.3` -> `object 0.36.7` -> `ruzstd 0.7.3`, and direct `omni-codegen` -> `object 0.36` (which does not disable default features).
  2. Document the advisory ID, affected package, and safe upstream versions (e.g. 0.9.1+).
  3. Evaluate the actual attack surface: assess whether untrusted inputs reach the affected decoder within Omni compiler stages. In Omni, `object` is utilized during machine code generation and Cranelift object emission of local artifacts; no untrusted network streams flow into this decoder during compilation.
  4. Never silently bump dependencies without compatibility testing. Remediations must be deliberate, isolated, tested, and documented.
