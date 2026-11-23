# Omni project documentation

Classification: Informative

This directory is the curated entry point for owner requirements, work preferences, historical project sources and specification-authority boundaries.

## Read in this order

1. [User requirements baseline](USER_REQUIREMENTS_BASELINE.md) — the broad product, language, compiler, runtime, tooling, assurance and workflow requirements.
2. [Source reconciliation](SOURCE_RECONCILIATION.md) — source provenance, authority hierarchy, historical decisions, unresolved gaps and live evidence snapshot.
3. [Owner working preferences and repository expectations](OWNER_WORKING_PREFERENCES_AND_REPOSITORY_EXPECTATIONS.md) — preferred engineering execution and the expected end state of this repository.
4. [Installer and release qualification](INSTALLER_AND_RELEASE_QUALIFICATION.md) — Windows toolchain installer, lifecycle, signing and distribution expectations.
5. [Historical source archive index](ARCHIVE_INDEX.md) — preserved prompts and conversation synthesis.
6. [Current normative authority](../../spec/manifest/omni-edition1.manifest.json) — Edition 1 manifest.
7. [Normative grammar](../../spec/grammar/omni-edition1.ebnf), [rule registry](../../spec/registry/rules.json) and [core semantic reference](../../spec/reference/core_semantics_and_types.md) — check exact authority/status metadata for each domain.
8. [Implementation inventory](../../IMPLEMENTATION_INVENTORY.md), [implementation baseline](../implementation-baseline.md), [implementation plan](../../IMPLEMENTATION_PLAN.md), [foundation gates](../foundation-gates.md), [discrepancy register](../../DISCREPANCY_REGISTER.md) and [ADR index](../adr/) — implementation evidence and open work.
9. [Master project charter](../../OMNI%20-%20Master%20Project%20Vision,%20Historical%20Requirements,%20Research%20Charter,%20Rust-First%20Development%20Strategy,%20Self-Hosting%20Transition,%20Engineering%20Guidelines%20and%20Continuous%20Evolution%20Policy.md) and [20 November continuous engineering mandate](../../OMNI_LONG_HORIZON_CONTINUOUS_ENGINEERING_MANDATE_2026-11-20.md) — long-term goals and execution policy.

## Authority and organization

- **Language authority:** the in-force, hash-bound Edition 1 specification under `spec/`, interpreted through the manifest and governance records. Verify the authority for each domain; do not treat every file under `spec/` as automatically normative.
- **Project-goal authority:** the current owner master charter and continuous-engineering mandate. They define product intent and engineering obligations but do not amend language semantics by themselves.
- **Implementation truth:** current source, tests, inventory and CI evidence at an exact commit. A test result or plan is not a blanket feature qualification.
- **Historical/proposal material:** generated compendia, candidates, old prompts and exploratory research are informative unless formally adopted.
- **Research boundary:** independent `Omni-Research` is a separate effort. Findings enter this implementation only through review, evidence assessment, ADR and specification governance.

Root master-goal and mandate files are retained at their existing paths for link compatibility. This index is the navigation entry point; do not create more competing master documents.

## Live repository evidence snapshot — 23 November 2026

Observed public `main` HEAD: `8f4878e3f90cae7adda9ec95e4778aad2703e056`.

- Linux and Windows push-CI validation passed.
- macOS validation failed in `omni-codegen` test `test_indirect_function_pointer_call_verifies_emits_and_executes`; Apple `ld` rejected a text relocation from `_omni_main` to `_inc` (29 passed, 1 failed in that crate job). The complete three-platform CI matrix is not green.
- Separate strict Clippy, security audit and CodeQL workflows passed.
- First-stage readiness is informational and does not grant qualification.
- The implementation inventory records 26 mandatory items. The macOS regression must be reflected in status and retested on the next commit; package/build entry points, formal qualification and production signing remain separately tracked.
- A distinct dynamic Cargo/fuzz dependency-update run also failed; investigate it separately from the macOS linker failure.

Recheck Actions, inventory and source in every later session. This snapshot is not current test evidence after this commit.
