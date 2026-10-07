# Goal: align-omni-parse-driver

## User Request

The user requests that the Omni compiler's parser and driver layers be fully aligned with the current authoritative Omni Candidate 3 specification while preserving all existing MIR, ownership, and code‑generation logic.

## Refined Goal

Resolve the 13 failing parser/driver integration tests by implementing or correcting any missing or incorrect parsing behavior for language constructs defined in Candidate 3. Update the grammar artifact and manifest as necessary to reflect the authoritative specification. Ensure all existing MIR, ownership, and code‑generation tests continue to pass.

## Acceptance Criteria

- [ ] All 13 omni‑driver parser failures are removed; `cargo test -p omni‑parse --quiet` and `cargo test -p omni‑driver --quiet` complete with zero failures.
- [ ] The emitted CST for the failing examples matches the expected source exactly, confirming loss‑less parsing.
- [ ] The manifest file `omni-edition1.manifest.json` (or equivalent) shows `candidate: 3`, `status: released`, and all rule references correspond to Candidate 3.
- [ ] `cargo test --workspace --all-targets --quiet` passes with no regressions.
- [ ] No prior MIR or ownership behaviours are altered; MIR and ownership test suites continue to succeed.

## Scope Boundaries

**In scope:**
- Parser and driver source files under `compiler/omni-parse` and `compiler/omni-driver`.
- Grammar artifact (`spec/grammar/omni-edition1.ebnf`), manifest, and related metadata.
- Test harnesses for parser and driver only.

**Out of scope:**
- Any changes to MIR, ownership, or code‑generation other than preserving existing behavior.
- Adding new language features beyond the fixes required to satisfy the current Candidate 3 spec.
- Refactoring unrelated modules.

## Applicable Project Conventions

**Quality gate command:**
- `cargo test --workspace --all-targets`

**Commit convention:**
- Conventional commits (default). Each agent commit must include a `[B]` or `[I]` marker and an `Assisted‑by` trailer.

**Guidelines:**
- None specific beyond the standard quality gates.

**Rules:**
- All changes must be made in the `main` branch, no separate feature branch.
