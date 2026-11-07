# Omni Antigravity Customizations

This directory contains the workspace customization configuration for the **Omni** programming language, toolchain, and computational platform.

---

## Directory Layout

```
.agents/
├── README.md                          # Directory overview and customization documentation
├── CUSTOMIZATION_BUDGET.md            # Token budgets, file size inventory, and headroom
├── rules/                             # Workspace rules (always_on and model_decision)
│   ├── omni-core-principles.md        # Permanent invariants, mission & unbounded roadmap (always_on)
│   ├── omni-research-driven-development.md # Research protocol, vibe-first research & evidence taxonomy (model_decision)
│   ├── omni-specification-governance.md    # Spec authority, Candidate 1/2/3 matrix & traceability (model_decision)
│   ├── omni-rust-engineering.md       # Rust 25-package architecture, lints & safety (model_decision)
│   ├── omni-verification-and-evidence.md   # Test layers, formal assurance & security advisories (model_decision)
│   └── omni-repository-workflow.md    # Git discipline & structured handoffs (model_decision)
└── skills/                            # On-demand modular procedures & runbooks
    ├── omni-qualification/            # Authoritative qualification gate runbook (25 packages)
    │   └── SKILL.md
    ├── omni-spec-conformance/         # Spec-to-compiler conformance & candidate governance
    │   └── SKILL.md
    ├── omni-research-to-spec/         # 18-stage research-to-specification pipeline
    │   └── SKILL.md
    ├── omni-architecture-review/      # Whole-platform architectural review protocol
    │   └── SKILL.md
    └── omni-session-handoff/          # Cross-session durable handoff documentation
        └── SKILL.md
```

---

## Customization Philosophy

Antigravity operates on Omni with an integrated scientific methodology:
1. **Unbounded Generational Vision**: Omni is a multi-paradigm computational platform with no predetermined final development ceiling ("Everything, but structured").
2. **Vibe-First Ergonomics with Strict Determinism**: Omni minimizes boilerplate ceremony across expression tiers, while enforcing 100% deterministic, formal grammar. Natural language heuristics and NLP interpretation are forbidden.
3. **Candidate Governance & The No-Override Principle**: Edition 1 Candidate 1 is the ratified normative baseline (`spec/manifest/omni-edition1.manifest.json`). Candidate 2 is pre-release/pending, and Candidate 3 tests are experimental probes. Compiler code, tests, or appended EBNF fragments cannot silently override normative specifications.
4. **Preserved Memory Architecture Inquiry**: The open requirement between managed memory and affine ownership remains under active research.
5. **Research Separation**: Idealized future calculi (such as the sibling `Omni-Research` initiative in `../Ideas/Omni-Research`) are kept strictly separated from active Edition 1 compiler development until formally adopted.
6. **Rust Rigor**: Workspace lints enforce `unsafe_code = "forbid"` and `clippy::all = "deny"` across 25 packages (18 compiler + 7 tooling), pinned to Rust 1.95.0, with deterministic `--locked` builds.
7. **Evidence & Truthful Reporting**: Claims must use explicit evidence classifications (`VERIFIED`, `PROVEN`, `EXPERIMENTALLY_VALIDATED`, `BLOCKED`, etc.). Passing tests are never claimed unless executed and inspected.

---

## Maintenance Conventions

- All rules in `.agents/rules/` must include YAML frontmatter with `trigger: always_on` or `trigger: model_decision`.
- Rule files must remain under the 24,000-byte per-file limit.
- All skills in `.agents/skills/<name>/` must contain a `SKILL.md` with `name` and descriptive `description` frontmatter.
- Token budgets must be reviewed and maintained in [`CUSTOMIZATION_BUDGET.md`](CUSTOMIZATION_BUDGET.md).
