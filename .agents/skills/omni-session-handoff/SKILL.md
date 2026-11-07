---
name: omni-session-handoff
description: >-
  Use this skill at session end or major milestones to produce a durable, self-contained
  handoff report capturing Git status, decisions, qualification evidence, and next steps.
---

# Omni Session & Milestone Handoff Protocol

This skill standardizes durable handoff reports so that subsequent development sessions or collaborators can immediately resume work with zero context loss.

## 1. Durable Handoff Template

Generate a structured handoff document following this schema:

```markdown
# Omni Development Session Handoff

## 1. Repository & Environment Identity
- **Repository**: https://github.com/shreyashjagtap157/Omni---New.git
- **Workspace Path**: [Workspace directory]
- **Current Branch**: `main`
- **HEAD Commit**: [SHA and commit subject]
- **Toolchain Status**: `rustc --version` / `cargo --version` (pinned: Rust 1.95.0)

## 2. Working Tree & Git Status
- **Modified Files**: [List from git status]
- **Untracked Files**: [List from git status]
- **Staged Changes**: [List if any]
- **Integrity Check**: Confirm no destructive git commands were run and user edits are preserved.

## 3. Work Completed in Current Session
- **Research Conducted**: [Literature reviewed, alternatives analyzed, design tradeoffs]
- **Architectural Decisions**: [ADRs created or updated, invariant changes]
- **Specifications Updated**: [Clauses in spec/ or grammar updated]
- **Implementation Changes**: [Crates, modules, and code modified]

## 4. Verification & Qualification Results
- **Execution Status**: [PASSED / FAILED / BLOCKED / PARTIALLY_EXECUTED]
- **Evidence Classification**: [VERIFIED / EXPERIMENTALLY_VALIDATED / BLOCKED]
- **Formatting (`cargo fmt --check`)**: [PASSED / FAILED / NOT RUN]
- **Clippy (`cargo clippy -D warnings`)**: [PASSED / FAILED / NOT RUN]
- **Locked Tests (`cargo test --workspace --locked`)**: [PASSED (counts) / FAILED]
- **Targeted Subsystem Tests**: [Crates tested and results]

## 5. Known Issues, Blockers & Incomplete Areas
- **Environmental Blockers**: [Missing tools, unavailable linkers, target mismatches]
- **Open Architectural Questions**: [Unresolved decisions, memory models, candidate status]
- **Failing Tests / Panics**: [Exact error messages and repro commands]
- **Dependency Advisories**: [Status of open security advisories, e.g. Issue #46 ruzstd]

## 6. Concrete Next Milestone
- [ ] Next immediate task 1
- [ ] Next immediate task 2
- [ ] Qualification goal
```

## 2. Handoff Verification Checklist

Before completing the handoff:
- [ ] Run `git status --short` and confirm no unintended files were modified.
- [ ] Verify that test assertions match actual executed commands.
- [ ] State explicitly what was verified versus what remains unverified or blocked.
- [ ] Never claim complete qualification if environmental dependencies caused tests to be skipped.
