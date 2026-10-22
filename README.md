# Omni — Edition 1 Compiler and Toolchain

This repository contains the Rust implementation of the Omni Edition 1 programming language and toolchain. The implementation is governed by the normative specifications under `spec/`; implementation behavior must not be invented locally when a specification is silent or contradictory.

> **Repository status — 22 October 2026:** this is an active, early-stage compiler implementation, not a complete production compiler or self-hosted language. The latest inspected main-branch qualification run failed at focused ownership-verifier tests because Cargo.lock needed an update under `--locked`. A recent fuzz workflow also encountered a stable/nightly toolchain mismatch. See [Actions](https://github.com/shreyashjagtap157/Omni---New/actions) for live state.

## Generational roadmap

- **Stage 1:** Rust-only minimal working core, from 0.0.0.1 to 1.0.0.0.
- **Stage 2:** Rust-only complete production platform, from 1.0.0.1 to 2.0.0.0.
- **Stage 3:** Omni-written, independently qualified self-hosting Omni, from 2.0.0.1 to 3.0.0.0.

These are roadmap targets, not claims that all stages have been completed.

## Repository map

- `compiler/` — compiler implementation.
- `spec/` — normative specification and related material.
- `conformance/` — language conformance cases.
- `tests/` — tests.
- `ci/` — CI qualification helpers.
- `tools/` — repository and specification tooling.
- `Cargo.toml` and `Cargo.lock` — Rust workspace and dependency lock.
- `rust-toolchain.toml` — pinned toolchain contract.

The implementation plan identifies foundations for lexing/parsing, type/name work, ownership state and MIR definitions, while effects, HIR lowering, complete MIR verification and machine/code-generation/runtime components remain partial or incomplete. Consult the implementation plan and current source for the status of each feature.

## Build and qualification

The repository pins Rust 1.95.0 in `rust-toolchain.toml`.

Use the declared toolchain and run locked checks:

```bash
rustc --version
cargo --version
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
```

These commands express the intended qualification checks; the current baseline should be considered unqualified until they pass on the candidate commit.

Do not update `Cargo.lock` implicitly in CI. If dependency manifests have intentionally changed, update and review the lockfile in a deliberate commit, then rerun locked verification.

Fuzz jobs that require `-Z` options or sanitizers must use an explicitly supported nightly toolchain. Keep the stable compiler correctness gate separate from nightly-only fuzzing.

## Security and dependency hygiene

The repository has an open issue concerning a ruzstd decompression-bomb advisory: [issue #46](https://github.com/shreyashjagtap157/Omni---New/issues/46). Review the dependency graph and attack surface, make a documented remediation decision, and verify the selected dependency version.

Open dependency updates must be evaluated against the language/compiler compatibility contract; a newer backend crate should not be adopted without compatibility testing.

## Current validation priorities

1. Reconcile Cargo manifests and lockfile for repeatable `--locked` builds.
2. Fix the focused ownership-verifier failures and rerun the complete workspace gate.
3. Align fuzz/sanitizer jobs with a compatible nightly toolchain.
4. Resolve and document the dependency security advisory.
5. Keep each semantic feature linked to its normative rule and executable conformance test.
6. Do not claim self-hosting or production readiness until the corresponding end-to-end stage passes.

## Releases

No GitHub release was found in the repository audit dated 22 October 2026. Treat any version or changelog entry as development metadata unless accompanied by a qualified tag and release evidence.

## License

See [LICENSE](LICENSE).
