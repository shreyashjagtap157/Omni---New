# Omni Development & Engineering Charter

This repository contains the Rust implementation of the **Omni Edition 1 programming language and toolchain** (`https://github.com/shreyashjagtap157/Omni---New.git`). All work in this repository is governed by the principles of scientific inquiry, rigorous verification, strict Rust engineering, normative specification authority, and a permanent commitment to unbounded evolution.

---

## 1. Project Mission & Foundational Product Vision

Omni is not merely a conventional systems programming language with a Rust-implemented compiler. It is an expressive, general-purpose, multi-paradigm programming language and computational platform designed for:
- **Exceptional Developer Ergonomics & Vibe-First Programming**: Natural, intuitive, low-ceremony programming allowing developers to express computational intent without mechanical boilerplate, while maintaining 100% deterministic, statically verifiable semantics (never heuristics or NLP interpretation).
- **Strong Static Typing & Deep Inference**: Rigorous type safety with advanced bidirectional inference to eliminate redundant type and generic parameter annotations.
- **Reliable Memory & Ownership**: Fine-grained ownership, affine/linear typing, and memory models spanning systems-level deterministic destruction to high-level managed facilities.
- **Algebraic Effects & Capabilities**: Principled effect tracking, fine-grained capability security, and structured resource handling.
- **High-Performance Native Execution & Portability**: Native compilation across architectures and execution profiles (embedded, bare-metal, desktop, distributed).
- **Formal Verification & Correctness**: Machine-checked properties, predicate engine validation, and provable safety invariants.
- **Continuous Evolution ("Everything, but structured")**: Omni accumulates expressive power, multi-paradigm facilities, and tooling without compromising mathematical cohesion, explicit interfaces, or phase boundaries.

### 1.1 No Predetermined Development Ceiling
Omni has **no predetermined final limit** on research, capability, optimization, or architectural enhancement. The three-stage roadmap represents evolutionary milestones, not a stopping point:
- **Stage 1 (Current: `0.0.0.1` -> `1.0.0.0`)**: Minimal, functionally complete, verified working core implemented strictly in Rust (lexing, parsing, name resolution, type inference, ownership/borrow checking, effects, HIR/MIR, verification, driver).
- **Stage 2 (`1.0.0.1` -> `2.0.0.0`)**: Complete production-grade language platform in Rust with full optimization, standard libraries, package tooling, IDE integrations, and multi-platform targets.
- **Stage 3 (`2.0.0.1` -> `3.0.0.0`)**: Independently qualified, self-hosting Omni implementation written in verified Omni, with reproducible bootstrap trust.
- **Beyond Stage 3**: Continuous adoption of superior type theories, runtime models, hardware architectures, optimization techniques, and verification methodologies.

> **Crucial Rule**: Never claim self-hosting, full production readiness, or formal completeness until the explicit qualification gates for that stage are independently proven and verified.

---

## 2. Vibe-First Programming Philosophy

Omni is fundamentally designed to minimize unnecessary boilerplate:
- Minimize repetitive type annotations, generic parameters, ownership boilerplate, explicit casts, iteration mechanics, intermediate scratch variables, and error-handling ceremony.
- Provide three compatible expression tiers:
  1. *High-level intent-oriented tier* (rapid prototyping, low ceremony).
  2. *Intermediate explicitly controlled tier* (application logic, effect handling).
  3. *Low-level systems tier* (fine-grained ownership, memory regions, hardware control).
- **Inviolable Determinism**: Natural syntax must be fully deterministic and parseable via standard formal grammar. Natural language heuristics, fuzzy parsing, or non-deterministic recovery must never dictate language semantics.
- **Distinction**: The permanent vibe-first product mission must never be confused with the ratification status of specific syntax proposals (e.g. Candidate 2 or Candidate 3).

---

## 3. Normative Specification Authority & Governance Matrix

The repository contains historical and active artifacts spanning Candidate 1, Candidate 2, and Candidate 3. All agents must navigate this governance hierarchy strictly:

### 3.1 Domain-Specific Specification Authority Model
Authority in Omni is established by specific, non-interchangeable artifact domains:
- **Normative Specification Artifacts (`spec/reference/`, `spec/grammar/`)**: Define the authoritative language semantics, typing rules, and grammar.
- **Edition Manifest (`spec/manifest/omni-edition1.manifest.json`)**: Binds the selected specification snapshot and cryptographically pins integrity metadata (`spec_tree_sha256`).
- **Release Gates (`spec/release/foundation-gate.json`, `docs/foundation-gates.md`)**: Govern qualification, milestone criteria, and release eligibility. A release gate is an operational qualification gate, not a substitute for a language specification.
- **Compiler Code, Tests & Plans**: Downstream engineering evidence and verification instruments. They provide implementation evidence, not normative language authority.

### 3.2 Authority Tiers & Status Matrix
1. **Edition 1 Active Baseline (Candidate 1: `1.0.0-candidate.1`)**: Authoritative as bound by `spec/manifest/omni-edition1.manifest.json` and governed by `spec/release/foundation-gate.json`. It is the active normative baseline; its candidate release status must be preserved and must not be described as definitively ratified or production-released without completed qualification gates.
2. **Candidate 2 Erratum (`spec/grammar/candidate2-erratum.md`)**: Formally recorded as `status: pre-release`, `signature_state: pending`, `implementation_impact: "none"`. It is **NOT in force** normatively for the Edition 1 baseline.
3. **Candidate 3 Proposals**: Exploratory syntax, precedence definitions, and parser experiments (e.g. `precedence.rs` Candidate 3 table, `candidate3_pipeline_and_optional_navigation_parse` in `omni-parse`). These are **experimental research prototypes**, not ratified Edition 1 semantics.
4. **Appended EBNF Productions**: The `(* VIBE-GRAM extensions *)` appended to `spec/grammar/omni-edition1.ebnf` (pipeline `|>`, newline termination, projection shorthand) reflect unratified errata and exploratory candidates; they are not active in the baseline grammar contract and must not be treated as ratified Edition 1 authority unless formally enabled under explicit feature gates.

### 3.3 Conflict Resolution Protocol
When specification, manifest, grammar, and compiler implementation disagree:
- Record the exact conflict across rule IDs, file paths, and AST constructs.
- Apply the domain authority model: normative spec defines semantics, manifest binds tree integrity, release gate checks criteria. Implementation code or tests never override normative specifications.
- Block unsupported Edition 1 conformance claims.
- **Never silently modify** the grammar, manifest, release digest, or compiler tests to manufacture false consistency.

---

## 4. Open Architectural Decisions: Memory Models

Omni explicitly preserves an open architectural requirement regarding its memory model:
- Earlier historical vision documents specify managed memory by default.
- Edition 1 compiler specifications specify affine ownership and borrow checking by default.
- **Policy**: This tension remains an active, open research area. Omni must research, benchmark, and evaluate hybrid approaches (e.g. region-based memory, deferred reference counting, capability-bounded heaps, profile-guided allocation) rather than silently hardcoding an unexamined choice.

---

## 5. Rust Engineering & Code Quality Standards

The Omni compiler is engineered under strict Rust safety standards:
- **Workspace Lint Directives**: Root `Cargo.toml` mandates:
  ```toml
  [workspace.lints.rust]
  unsafe_code = "forbid"
  [workspace.lints.clippy]
  all = "deny"
  ```
  All 25 workspace packages inherit these lints via `[lints] workspace = true` in their respective manifests. No `unsafe` code is permitted.
- **Zero-Warning Policy**: Code must pass `cargo clippy --workspace --all-targets --locked -- -D warnings` with zero warnings.
- **Pinned Toolchain**: Adhere strictly to Rust 1.95.0 in [`rust-toolchain.toml`](rust-toolchain.toml). Stable compiler gates must remain decoupled from nightly-only fuzzers/sanitizers.
- **Formatting**: Strict enforcement of [`..rustfmt.toml`](.rustfmt.toml) via `cargo fmt --all -- --check`.
- **Locked Dependency Hygiene**: Builds, tests, and checks must use `--locked` (e.g. `cargo test --workspace --locked`). Never modify `Cargo.lock` silently.
- **Security Advisories**: Known advisories (such as `ruzstd 0.7.3` in the dependency tree) must be audited across both direct (`omni-codegen -> object`) and transitive (`omni-codegen -> cranelift-object -> object`) paths. Distinguish dependency resolution from actual decoder call-site exploitability; never modify lockfiles without deliberate verification.

---

## 6. Verification, Evidence Classification & Qualification Gate

Assertions and progress reports must use explicit evidence classifications:
- `VERIFIED`: Directly demonstrated through reproducible tests or compiler runs.
- `PROVEN`: Formally proved within machine-checked assumptions.
- `EXPERIMENTALLY_VALIDATED`: Supported by designated empirical experiments.
- `IMPLEMENTED_UNVERIFIED`: Code exists but lacks full qualification tests.
- `PROPOSED` / `HYPOTHESIS`: Theoretical design under active study.
- `BLOCKED` / `REJECTED`: Halted due to environment/dependencies or rejected with documented rationale.

### Official Qualification Sequence
1. Toolchain: `rustc --version` and `cargo --version` (Rust 1.95.0)
2. Format: `cargo fmt --all -- --check`
3. Clippy: `cargo clippy --workspace --all-targets -- -D warnings`
4. Locked Test: `cargo test --workspace --locked`
5. Package Subsystems: Verify affected crates across the 25 workspace packages.

---

## 7. Repository Discipline & Collaboration Habits

- **Git Integrity**: Main-only development workflow. Never force-push, rewrite history, or hard-reset away local modifications. Always preserve user work and inspect status before and after edits.
- **Autonomy with Accountability**: Conduct deep research, diagnose root causes, validate fixes, and update docs independently. Escalate only genuine architectural conflicts requiring developer authority.
- **Durable Handoffs**: At milestones or session ends, provide structured status reports covering exact commands executed, test results, known blockers, and concrete next steps.

---

## 8. Customization System Navigation

- **Rules (`.agents/rules/`)**:
  - [`omni-core-principles.md`](.agents/rules/omni-core-principles.md) — Core invariants and mission (always on).
  - [`omni-research-driven-development.md`](.agents/rules/omni-research-driven-development.md) — Research protocols and evidence classification.
  - [`omni-specification-governance.md`](.agents/rules/omni-specification-governance.md) — Spec traceability, Candidate 1/2/3 matrix, and EBNF authority.
  - [`omni-rust-engineering.md`](.agents/rules/omni-rust-engineering.md) — Rust crates, lints, safety, and lockfile hygiene.
  - [`omni-verification-and-evidence.md`](.agents/rules/omni-verification-and-evidence.md) — Testing, diagnostics, security reviews, and qualification.
  - [`omni-repository-workflow.md`](.agents/rules/omni-repository-workflow.md) — Git preservation and session handoffs.
- **Skills (`.agents/skills/`)**:
  - [`omni-qualification`](.agents/skills/omni-qualification/SKILL.md) — Authoritative qualification gate runbook.
  - [`omni-spec-conformance`](.agents/skills/omni-spec-conformance/SKILL.md) — Spec-to-compiler conformance & governance matrix.
  - [`omni-research-to-spec`](.agents/skills/omni-research-to-spec/SKILL.md) — Transforming research into validated normative specs.
  - [`omni-architecture-review`](.agents/skills/omni-architecture-review/SKILL.md) — Architectural invariants and phase review.
  - [`omni-session-handoff`](.agents/skills/omni-session-handoff/SKILL.md) — Cross-session handoff generation.
