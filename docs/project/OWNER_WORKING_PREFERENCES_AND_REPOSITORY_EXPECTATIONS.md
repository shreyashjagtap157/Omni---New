# Owner working preferences and repository expectations — Omni

Classification: Informative

This document records the owner's durable expectations for how engineering work is conducted and what the canonical Omni repository must contain. It complements, but never overrides, normative language specifications or explicit repository governance.

## 1. Preferred working style

- **Inspect the real repository first.** Refresh the remote, inspect current `main`, commit history, complete relevant file contents, workspace metadata, all local changes and current CI before relying on a prior handoff or remembered state.
- **Work on canonical `main` only.** Do not create feature branches for Omni. Preserve existing history and integrate only through ordinary, non-rewriting commits.
- **Preserve work absolutely.** Never force-push, rewrite history, hard-reset, clean, overwrite, silently stash or discard user files, even if they appear generated or untracked. Inspect provenance first.
- **Implement, do not stop at planning.** When authorized, perform research, design, code changes, integration, testing, debugging, documentation and qualification. An audit or roadmap is an input to execution, not a completion point.
- **Continue autonomously through dependency-ready tasks.** Do not repeatedly ask the owner to say “continue”. Escalate only a real owner-only decision, an actual ambiguity in normative semantics, production signing identity, external purchase/resource, destructive action or authorization boundary.
- **Research while implementing.** Use current authoritative standards, peer-reviewed research, source repositories, maintained implementations, and tool/vendor documentation for claims that may change. Compare alternatives; assess maturity and limitations; prototype and measure rather than assuming novelty means superiority.
- **Fix root causes.** Do not weaken tests, relax lints, hide failures, hard-code false success, introduce compatibility hacks, or redefine semantics merely to obtain green CI.
- **Keep status exact.** Distinguish design, prototype, implementation, integration, tests, proof, independent review, qualification and release. Tie every result to exact commit, spec identity, target, command, logs and workflow ID.
- **Maintain all project records.** Update requirements, architecture/ADRs, plans, inventory, changelog, known limitations, tests and session handoffs whenever implementation or approved decisions change.
- **Use specialists with clear scope.** Decompose work into non-overlapping specialist tasks where supported; establish file/interface ownership and one integration authority. Use an independent verification gate for acceptance. Agent reports are claims until their diffs and evidence are checked.
- **Prefer durable milestones over repeated status chatter.** Close coherent work units, validate affected consumers, record evidence, then select the next dependency-ready task.

## 2. What the canonical Omni repository is expected to become

The repository must evolve into the complete, production-grade Omni programming-language platform—not remain a collection of disconnected compiler crates or specifications.

It must contain and maintain traceable, coherent implementations and evidence for:
- the governed language editions, grammar, semantic rules, manifests, schemas and conformance corpus;
- the Rust-first production compiler pipeline and independent reference execution model;
- name/type/generic/trait resolution, ownership/lifetime, memory, effects, capabilities, error handling and concurrency semantics;
- HIR/MIR or their governed successor representations, semantic-obligation preservation and fail-closed independent verification;
- correct target-independent lowering, native object emission, ABI/FFI, relocations, linker integration and real program execution;
- runtime, standard library, allocation/resource services and platform abstractions;
- modules, packages, deterministic dependency management, build orchestration and artifact provenance;
- CLI, formatter, linter, diagnostics, tests, fuzzing, debugger/observability, language-server/IDE foundation, documentation and shell integration;
- Windows-first official toolchain/install qualification alongside declared Linux/macOS and cross-target support;
- security, dependency and supply-chain controls, reproducible build, SBOM, signing and release qualification;
- controlled, parallel self-hosting experiments followed by a genuinely independent and reproducible bootstrap qualification.

All capabilities must trace from owner intent through approved specification and architecture to implementation, tests and qualification evidence. Placeholders and compile-only crates must remain marked incomplete. No self-hosting, completeness, safety or production claim without its actual gate evidence.

## 3. Long-term product expectations

- Preserve the multi-paradigm, multi-level, general-purpose and vibe-first programmer experience.
- Keep the high-level, intermediate and low-level programming tiers coherent.
- Maximize expressiveness and reduce unnecessary ceremony without sacrificing deterministic grammar, static verifiability, security or predictable resource behavior.
- Preserve the broad intended application domain: systems, operating systems, compilers, embedded/bare-metal, real-time, desktop, distributed, scientific/high-performance and high-assurance development as the language and target evidence mature.
- Keep the memory-model tension (historical managed-memory ambition versus current Edition 1 affine ownership) visible for research and governance; never silently change the active Edition 1 rules.
- Treat Approach C as the implementation strategy: Rust production compiler first, self-hosting experiments in parallel, then controlled migration only after independent qualification.
- Continue research and evolution beyond Stage 3. No finite release or self-hosting milestone ends the project.

## 4. Authority and safety boundaries

This file and the [requirements baseline](USER_REQUIREMENTS_BASELINE.md) express project intent. The active hash-bound Edition 1 specification and accepted governance decisions control language semantics. Historical prompts, generated compendia, experiments and the independent `Omni-Research` workspace cannot override that authority.

Do not push tags/releases, sign production artifacts, disclose secrets, or perform destructive host operations without the required separate authorization. In the current task, the owner explicitly authorized the requirements/documentation changes and remote publication only.
