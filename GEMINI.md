# Omni Development Directives

Antigravity operates under the authoritative charter established in [AGENTS.md](AGENTS.md) and modular workspace rules in [`.agents/rules/`](.agents/rules/).

## Non-Negotiable Core Directives

1. **Foundational Product Vision**: Omni is a multi-paradigm computational platform with no predetermined development ceiling ("Everything, but structured"). Vibe-first ergonomics must be pursued as a permanent goal while preserving strict mathematical determinism.
2. **Research-Driven Foundation**: Omni is developed through continuous scientific inquiry (`Research → Analysis → Design → Spec → Experiment → Validation`). An implementation is a scientific instrument for testing hypotheses, not merely output code.
3. **Specification Authority & Candidate Governance**: Normative authority is established by `spec/manifest/omni-edition1.manifest.json` and `spec/release/foundation-gate.json` (Candidate 1 baseline: `1.0.0-candidate.1`). Preserve candidate status without claiming completed release sign-off. Candidate 2 errata remain unratified/pre-release, and Candidate 3 tests represent experimental prototypes. The No-Override Principle strictly forbids compiler code or test cases from silently overriding specifications.
4. **Preserved Memory Architecture Tension**: The balance between managed-memory-by-default and affine-ownership-by-default remains an open architectural inquiry; do not force a silent choice without formal research and trade-off evaluation.
5. **Research vs. Implementation Boundary**: The separate `Omni-Research` initiative (at sibling path `../Ideas/Omni-Research`) explores idealized long-term calculi. Its hypotheses inform the compiler, but never become compiler ground truth until formally ratified into `spec/`.
6. **Strict Rust Engineering**: Workspace lints enforce `unsafe_code = "forbid"` and Clippy `all = "deny"`. Adhere to pinned Rust 1.95.0, zero warnings, and deterministic `--locked` Cargo operations.
7. **Evidence-Based Qualification**: Classify assertions accurately (`VERIFIED`, `PROVEN`, `EXPERIMENTALLY_VALIDATED`, etc.). Execute the official qualification gate and report exact results honestly without concealing environmental blockers.
8. **Git & Workspace Integrity**: Follow main-only conventions. Never force-push, rewrite history, or discard uncommitted user modifications.
