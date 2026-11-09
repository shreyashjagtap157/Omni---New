# Omni — Edition 1 Compiler and Toolchain

This repository contains the Rust implementation of the Omni Edition 1 programming language and toolchain. The implementation is governed by the normative specifications under `spec/`; implementation behavior must not be invented locally when a specification is silent or contradictory.

> **Repository status — 9 November 2026:** this remains an active, early-stage compiler implementation, not a complete production compiler or self-hosted language. Exact head `3902f6d242abdb788f8e50e2f502ccfb395f3f47` passed the full Omni CI pipeline, both isolated memory-research models (12/12 and 22/22), focused ownership tests (16/16), Rust build/tests, strict Clippy, Cargo Audit, and CodeQL Rust/Actions analyses. The latest domain prototype additionally tests cross-environment capture access reservations, explicit resource ownership, pinning, promotion refusal, and region teardown refusal. Foundation 0.0.0.13 technical checks passed, but formal foundation sign-off, complete-language conformance, production readiness, and self-hosting remain unresolved. See [Actions](https://github.com/shreyashjagtap157/Omni---New/actions) for current run state.

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

These commands express the core Rust qualification checks. Commit `3902f6d` passed the complete main CI pipeline, including formatting and execution of both isolated memory-research models (12/12 and 22/22), focused ownership verification (16/16), locked workspace tests, specification linkage/topology checks, and the Foundation 0.0.0.13 technical gate. The exact workflows also passed Rust tests, strict Clippy, Cargo Audit, and both CodeQL analyses. A passing technical gate does not by itself establish formal sign-off, complete-language qualification, or product release readiness.

Do not update `Cargo.lock` implicitly in CI. If dependency manifests have intentionally changed, update and review the lockfile in a deliberate commit, then rerun locked verification.

Fuzz jobs that require `-Z` options or sanitizers must use an explicitly supported nightly toolchain. Keep the stable compiler correctness gate separate from nightly-only fuzzing.

## Security and dependency hygiene

The prior `ruzstd` decompression-bomb issue [#46](https://github.com/shreyashjagtap157/Omni---New/issues/46) is closed. The unused direct `object` dependency and its compression feature path were removed; the current `Cargo.lock` contains neither `ruzstd` nor `flate2`. Cargo Audit run `42589337774` on commit `3902f6d` passed. Re-run the audit after every dependency change; a green audit is commit-scoped and does not replace broader supply-chain review.

Open dependency updates must be evaluated against the language/compiler compatibility contract; a newer backend crate should not be adopted without compatibility testing.

## Current research and validation priorities

1. Continue falsification of the memory-domain prototype: explicit source-place invalidation for move captures; cross-domain strong/weak references and cycles; promotion failure atomicity; partial initialization.
2. Continue CFG/NLL and reborrow testing across loops, reference parameters, drops, moves, return provenance, aggregate field reinitialization, and the verifier's trusted reference-provenance boundary where normative rules are explicit.
3. Continue target/runtime research for allocation provenance, alignment, a fully specified native ABI, and host/target conformance. Opaque-handle reference memory is not a physical pointer ABI.
4. Investigate the highest-priority incomplete compiler dependencies from actual source/spec/CI evidence, then implement only behavior with established Edition 1 semantics; fail closed where the specification is silent.
5. Recover and reconcile the original requirements interview and connect the independent Omni-Research workspace to executable research/implementation evidence when that workspace is accessible.
6. Maintain the research-to-prototype-to-specification-to-implementation-to-validation cycle. Keep proposals, prototypes, tests, proofs, qualification, and release claims distinct.
## Releases

Do not infer a public release from development version strings or changelog entries. A release claim requires a qualified tag and matching release evidence under the project's versioning and governance policy.

## License

See [LICENSE](LICENSE).
