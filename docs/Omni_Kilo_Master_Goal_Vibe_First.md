# Omni — Kilo Master Goal (Vibe-First Edition)

Complete Omni entirely according to the repository's authoritative specifications, Candidate 2 Vibe Surface Syntax amendment, and master implementation plan.

Work autonomously from the actual repository state and continue through every unfinished milestone until the language/compiler/toolchain/runtime is genuinely complete.

- Read the authoritative specs, implementation plan, tests, manifests, CI, and existing code before changing anything; verify claims against the actual repository rather than commit messages or version labels.
- Treat Candidate 2 Vibe Surface Syntax as the surface-language target: minimal ceremony, readable intent, AI-friendly, deterministic, and never arbitrary natural-language parsing.
- Keep a strict separation between lossless CST, surface AST, deterministic vibe desugaring, and the semantic core.
- Implement real functionality; never replace missing semantics with stubs, hard-coded results, fake validation, disconnected scaffolding, or tests that merely prove compilation.
- For every change, inspect all affected downstream consumers, APIs, dependencies, semantics, tests, tooling, documentation, and release artifacts; fix regressions and warnings rather than hiding them.
- For every feature, test positive, negative, boundary, recovery, interaction, determinism, and regression behavior. Parser features must test CST structure and source round-trip; surface sugar must test deterministic desugaring and semantic equivalence.
- Preserve semantic rigor: types, ownership, lifetimes, effects, capabilities, async/cancellation, unsafe assumptions, FFI, comptime, distributed execution, reproducibility, and security must never be weakened by vibe syntax.
- Keep the specification, grammar, master implementation plan, status ledger, architecture docs, test corpus, changelog, compatibility matrix, and verification evidence synchronized after every successful coherent milestone.
- When a specification conflict is found, identify the exact conflicting rules and resolve it through the authoritative specification/erratum process; never invent an implementation-local semantic rule.
- Verify with the complete applicable suite: formatting, workspace build/check, all tests, Clippy with `-D warnings`, metadata/dependency checks, syntax/conformance fixtures, fuzzing, formal verification, target/backend tests, and reproducibility checks as applicable. Do not hide authoritative failures with output filtering.
- Before committing, audit `git status`, `git diff`, `git diff --cached`, `git diff --check`, changed files, generated artifacts, and dependency changes.
- Commit each coherent successfully verified milestone with a meaningful commit message. Preserve all historical commits; never rewrite or squash history.
- Push every successfully verified milestone to the authoritative remote branch unless an explicitly documented external blocker prevents it. After pushing, verify local HEAD and the remote branch resolve to the exact same commit.
- Continue automatically to the next lowest unfinished milestone. Do not stop after fixing one compiler error, reaching a version number, or obtaining a single green test.
- Only interrupt the user for a genuine external requirement that cannot be obtained or performed by the agent itself; otherwise diagnose, implement, integrate, test, document, commit, push, and continue.

Completion means implementation + integration + semantic correctness + tests + verification + documentation + reproducibility + compatibility + security evidence, not merely compilation.
