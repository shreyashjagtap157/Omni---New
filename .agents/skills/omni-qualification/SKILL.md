---
name: omni-qualification
description: >-
  Authoritative qualification runbook to execute, diagnose, and report the full
  Omni compiler qualification gate across all 25 workspace packages.
---

# Omni Authoritative Qualification Runbook

This runbook defines the exact sequence of commands, environment checks, and diagnostic reporting required to qualify changes across the Omni compiler repository.

## 1. Toolchain & Environment Verification

Verify that the local environment matches the pinned specification in `rust-toolchain.toml` (Rust 1.95.0):

```pwsh
# When cargo is on PATH:
rustc --version
cargo --version

# If cargo is not on user system PATH, use standard profile path:
& "$HOME\.cargo\bin\rustc.exe" --version
& "$HOME\.cargo\bin\cargo.exe" --version
```
- Pinned toolchain: `1.95.0`.
- If toolchain binaries are missing or inaccessible, record as `BLOCKED: Environmental Dependency`.

### Dynamic Workspace Package Discovery
Do not assume a permanently fixed inventory of crates. Discover active workspace packages dynamically using:
```pwsh
cargo metadata --no-deps --format-version 1
```
*(Reference audit, Nov 2026: 25 packages discovered — 18 compiler crates + 7 tooling crates).*

## 2. Official Qualification Sequence

Execute the standard qualification sequence in order:

### Step A: Formatting Gate
```pwsh
cargo fmt --all -- --check
```
- If discrepancies occur, run `cargo fmt --all` to resolve them, then re-check.

### Step B: Strict Workspace Clippy Lint Gate
```pwsh
cargo clippy --workspace --all-targets --locked -- -D warnings
```
- All Clippy warnings are treated as hard errors.
- Confirm workspace lint inheritance (`unsafe_code = "forbid"` and `clippy::all = "deny"`).

### Step C: Locked Workspace Unit & Integration Tests
```pwsh
cargo test --workspace --locked
```
- **Mandatory**: Use `--locked` to ensure `Cargo.lock` is not modified implicitly.
- Tests all 25 workspace packages (18 compiler + 7 tools).

## 3. Targeted Subsystem Testing (Across 25 Packages)

Execute targeted crate tests during focused development:

```pwsh
# Source, Tokens, Parsing
cargo test -p omni-unicode --locked
cargo test -p omni-traits --locked
cargo test -p omni-source --locked
cargo test -p omni-lex --locked
cargo test -p omni-syntax --locked
cargo test -p omni-parse --locked

# Semantic Analysis & Ownership
cargo test -p omni-names --locked
cargo test -p omni-types --locked
cargo test -p omni-own --locked
cargo test -p omni-effects --locked

# Lowering & Verification
cargo test -p omni-hir --locked
cargo test -p omni-mir --locked
cargo test -p omni-verify --locked

# Backend, Runtime & CLI
cargo test -p omni-machine --locked
cargo test -p omni-codegen --locked
cargo test -p omni-runtime --locked
cargo test -p omni-driver --locked
cargo test -p omni-stage0 --locked

# Tools & Conformance
cargo test -p omni-canon --locked
cargo test -p omni-audit --locked
cargo test -p omni-registry --locked
cargo test -p omni-conform --locked
cargo test -p omni-bindgen --locked
cargo test -p omni-evidence --locked
cargo test -p omni-topology --locked
```

## 4. Security Advisory Review (Issue #46: ruzstd)

When evaluating dependency hygiene:
1. Check lockfile and dependency tree for known advisories:
   - Example: `ruzstd 0.7.3` decompression-bomb advisory (patched in 0.9.1+).
2. Trace verified reverse dependency path:
   ```pwsh
   cargo tree -i ruzstd --locked
   ```
   Verified chain: `omni-codegen` -> `cranelift-object 0.110.3` -> `object 0.36.7` -> `ruzstd 0.7.3`.
3. Assess exploitability in compiler context:
   - Examine whether untrusted input can reach the affected decompression routines (Omni only reads trusted local object files during code generation).
4. Remediate with deliberate testing:
   - Never bump dependencies without compatibility testing and deliberate review. Never modify `Cargo.lock` silently.

## 5. Failure Diagnosis & Evidence Classification

If any check fails:
1. **Differentiate Error Types**:
   - **Compiler Defect**: Logic bug, panic, failing test, or clippy warning.
   - **Spec Discrepancy**: Implementation contradicts ratified `spec/`.
   - **Environmental Blocker**: Missing C linker, missing MSVC tools, or toolchain not on PATH (`BLOCKED: Environmental Dependency`).
2. **Never Conceal Incomplete Checks**: Never report a test suite as passed if tests were skipped or blocked.
3. **Qualification Evidence Report Schema**:
```markdown
### Omni Qualification Summary
- **Execution Status**: PASSED | FAILED | BLOCKED | PARTIALLY_EXECUTED
- **Evidence Classification**: VERIFIED | EXPERIMENTALLY_VALIDATED | BLOCKED
- **Toolchain**: rustc 1.95.0 / cargo 1.95.0
- **Format Check (`cargo fmt`)**: PASSED / FAILED
- **Clippy Check (`cargo clippy -D warnings`)**: PASSED (0 warnings) / FAILED
- **Workspace Tests (`cargo test --workspace --locked`)**: PASSED (X packages) / FAILED
- **Targeted Subsystem Tests**: [List crates and counts]
- **Environmental Limitations**: [None / List details]
```
