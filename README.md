# Omni — Edition 1 Compiler and Toolchain

This repository contains the Rust implementation of the Omni Edition 1 programming language and toolchain. The implementation is governed by the normative specifications under `spec/`; implementation behavior must not be invented locally when a specification is silent or contradictory.

> **Repository status — 9 November 2026:** this remains an active, early-stage compiler implementation, not a complete production compiler or self-hosted language. Commit `3529484d992e46ded452d96afbc42254f47ea882` passed the complete Omni CI pipeline (including 11 isolated memory-model research tests), Rust build/tests, strict Clippy, Cargo Audit, and both Rust and Actions CodeQL analyses. Foundation 0.0.0.13 technical checks passed on that commit, but formal qualification scope/sign-off, complete-language conformance, and production readiness remain unresolved. Subsequent research changes require fresh validation. See [Actions](https://github.com/shreyashjagtap157/Omni---New/actions) for current run state.

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
- `docs/research/` — evidence-based architecture studies and research handoffs.
- `research/` — isolated executable research prototypes; not production compiler components.
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

These commands express the core Rust qualification checks. Commit `3529484d` also passed the complete main CI pipeline, including the additional isolated research-model check and Foundation 0.0.0.13 technical gate. A passing technical gate does not by itself establish formal sign-off, complete-language qualification, or product release readiness.

Do not update `Cargo.lock` implicitly in CI. If dependency manifests have intentionally changed, update and review the lockfile in a deliberate commit, then rerun locked verification.

Fuzz jobs that require `-Z` options or sanitizers must use an explicitly supported nightly toolchain. Keep the stable compiler correctness gate separate from nightly-only fuzzing.

## Security and dependency hygiene

The prior `ruzstd` decompression-bomb issue [#46](https://github.com/shreyashjagtap157/Omni---New/issues/46) is closed. The unused direct `object` dependency and its compression feature path were removed; the current `Cargo.lock` contains neither `ruzstd` nor `flate2`. Cargo Audit run `42570448695` on commit `3529484d` passed. Re-run the audit after every dependency change; a green audit is commit-scoped and does not replace broader supply-chain review.

Open dependency updates must be evaluated against the language/compiler compatibility contract; a newer backend crate should not be adopted without compatibility testing.

## Current research and validation priorities

1. Continue the memory-architecture research in [the managed-memory versus affine-ownership study](docs/research/memory-architecture-managed-vs-affine.md). The historical managed-memory-default requirement and the Edition 1 affine-ownership reference remain an open design conflict; no architecture has been selected.
2. Extend CFG/NLL and reborrow testing across loops, reference parameters, drops, moves, return provenance, and aggregate field reinitialization where normative rules are explicit.
3. Continue target/runtime research for allocation provenance, alignment, a fully specified native ABI, and host/target conformance. Opaque-handle reference memory is not a physical pointer ABI.
4. Maintain the research-to-prototype-to-specification-to-implementation-to-validation cycle; do not let a passing prototype become an unapproved language rule.
5. Keep every semantic feature linked to its normative rule and executable conformance evidence.
6. Keep formal foundation sign-off, full-language qualification, self-hosting, and production readiness distinct from successful technical CI.
## Releases

Do not infer a public release from development version strings or changelog entries. A release claim requires a qualified tag and matching release evidence under the project's versioning and governance policy.

## License

See [LICENSE](LICENSE).
