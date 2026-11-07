---
trigger: model_decision
description: Use when managing Git operations, preserving working-tree changes, planning multi-step development, formatting handoffs, or collaborating in Omni.
---

# Omni Repository Discipline & Collaboration Workflow

This rule defines repository conventions, working-tree safety, development autonomy, and structured collaboration for Omni.

## 1. Git Discipline & Safety Constraints

- **Main-Only Development**: Omni follows a single-branch (`main`) development workflow. Avoid creating ad-hoc feature branches unless explicitly directed.
- **Strict Tree Preservation**:
  - Never execute `git reset --hard`, `git clean -fd`, or destructive cleanup commands that discard uncommitted or staged work.
  - Never force-push (`git push --force`).
  - Never rewrite published Git history.
- **Pre- and Post-Modification Inspection**:
  - Always run `git status --short` before touching files to inspect existing local modifications.
  - Always verify `git diff` before and after modifications to ensure only intended changes are touched.
  - Do not commit or push unless explicitly requested by the developer.

## 2. Autonomous & Thorough Problem Solving

When assigned a task or bug fix in the Omni codebase:
1. **Investigate Root Causes**: Do not apply superficial patches or paper over broken invariants.
2. **Consult Normative Specs**: Verify whether the existing code aligns with `spec/`.
3. **Implement Cleanly**: Respect crate boundaries and `#![forbid(unsafe_code)]`.
4. **Validate Thoroughly**: Run targeted tests, workspace tests, and lints.
5. **Update Documentation**: Synchronize specifications, ADRs, and tests when design changes occur.
6. **Minimize Interruption**: Proceed through clear technical milestones autonomously. Ask questions only when a genuine architectural ambiguity or tradeoff requires the developer's explicit policy decision.

## 3. Structured Progress Reporting & Session Handoffs

Provide clear, durable milestone updates. Always delineate:
- **Research Conducted**: Prior art explored, theoretical findings, alternatives considered.
- **Decisions Recorded**: Architectural choices made with justification.
- **Modifications Made**: Files created or modified with specific functional summaries.
- **Tests Executed**: Exact commands run, exit codes, and test outcomes.
- **Identified Blockers / Known Failures**: Explicit error messages and diagnostic classifications.
- **Unverified Assumptions**: Properties that require future empirical or formal validation.
- **Concrete Next Milestone**: Clear, actionable next steps so work can resume without context loss.
