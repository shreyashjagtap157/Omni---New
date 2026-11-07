# Omni Customization Token Budget & Asset Inventory

This document tracks the live IDE-reported customization meter counts, full on-disk asset inventories, estimated token footprints, trigger modes, and capacity allocations for Antigravity workspace customizations in the Omni repository.

---

## 1. Actual Antigravity Token Meter Snapshot

The following data reflects the **actual IDE-reported active customization usage** supplied from the developer snapshot on **2026-11-07**:

### Budget A: Rules (Ceiling: 20,000 tokens)
- **Active Usage**: **5,547 tokens**
- **Utilization**: **27.7%**
- **Available Capacity**: **14,453 tokens (72.3% headroom)**

| Active Component | Type / Location | Active Tokens | Purpose |
|---|---|---:|---|
| `AGENTS.md` | Root / Always Active | 3,693 | Canonical charter, product vision, domain authority model, Rust safety. |
| `.agents/rules/omni-core-principles.md` | Modular Rule (`always_on`) | 1,094 | Invariants, unbounded roadmap, determinism, and open memory architecture. |
| `GEMINI.md` | Root / Always Active | 760 | Concise agent directive linking directly to `AGENTS.md` and modular rules. |
| **Total Active Rules** | — | **5,547** | **Ceiling: 20,000 tokens** |

*(Note: Model-decision rules are loaded dynamically on demand when relevant to the task and are not continuously active in context.)*

### Budget B: Other Customizations (Ceiling: 20,000 tokens)
- **Active Usage**: **437 tokens**
- **Utilization**: **2.2%**
- **Available Capacity**: **19,563 tokens (97.8% headroom)**

| Active Component | Location | Active Metadata Tokens | Primary Capability |
|---|---|---:|---|
| `omni-session-handoff` | `.agents/skills/omni-session-handoff/` | 91 | Durable cross-session and milestone handoff generator. |
| `omni-spec-conformance` | `.agents/skills/omni-spec-conformance/` | 91 | Spec-to-compiler conformance & Candidate 1/2/3 governance. |
| `omni-architecture-review` | `.agents/skills/omni-architecture-review/` | 90 | Whole-platform architectural review: types, memory, IRs, FFI. |
| `omni-research-to-spec` | `.agents/skills/omni-research-to-spec/` | 85 | 18-stage pipeline transforming research into normative specs. |
| `omni-qualification` | `.agents/skills/omni-qualification/` | 80 | Authoritative qualification runbook across all workspace packages. |
| **Total Active Metadata** | — | **437** | **Ceiling: 20,000 tokens** |

> **Important Progressive Loading Principle**:
> Antigravity injects only skill **name and description metadata** (~80–91 tokens per skill) into the primary context budget. The complete `SKILL.md` file contents are loaded **on-demand** only when the skill is explicitly activated by the model or user. Therefore, the 437-token count represents live metadata overhead, preserving over 97% capacity for progressive execution.

---

## 2. Complete On-Disk Customization Inventory (Storage & Size Metrics)

Below is the complete physical on-disk inventory of all customization assets, including file byte sizes, 24 KB per-file local guardrail checks, and independent estimated full-text token footprints (~3.8 bytes/token conversion estimate):

### A. Rules & Root Instructions (Physical Files)

| File Path | Trigger Mode | File Size (Bytes) | 24 KB Guardrail | Est. Full Text Tokens | Scope |
|---|---|---|---|---|---|
| [`AGENTS.md`](../AGENTS.md) | Root / Always Active | 12,362 B | PASS (51.5%) | ~3,254 | Foundational workspace charter. |
| [`GEMINI.md`](../GEMINI.md) | Root / Pointer | 2,370 B | PASS (9.9%) | ~624 | Directives and pointers. |
| [`omni-core-principles.md`](rules/omni-core-principles.md) | `always_on` | 3,425 B | PASS (14.3%) | ~902 | Core invariants & roadmap. |
| [`omni-research-driven-development.md`](rules/omni-research-driven-development.md) | `model_decision` | 5,026 B | PASS (20.9%) | ~1,323 | Research methodology & evidence. |
| [`omni-specification-governance.md`](rules/omni-specification-governance.md) | `model_decision` | 4,115 B | PASS (17.1%) | ~1,083 | Domain authority, Candidate 1 (`1.0.0-candidate.1`), No-Override. |
| [`omni-rust-engineering.md`](rules/omni-rust-engineering.md) | `model_decision` | 4,290 B | PASS (17.9%) | ~1,129 | Rust workspace lints (`unsafe_code = "forbid"`), lockfile. |
| [`omni-verification-and-evidence.md`](rules/omni-verification-and-evidence.md) | `model_decision` | 4,398 B | PASS (18.3%) | ~1,158 | Testing layers, verified `ruzstd` paths, blockers. |
| [`omni-repository-workflow.md`](rules/omni-repository-workflow.md) | `model_decision` | 2,625 B | PASS (10.9%) | ~691 | Git discipline & handoffs. |
| **Total Rules Disk Footprint** | — | **38,611 B** | **All < 24 KB** | **~10,164 (Est.)** | — |

### B. Modular Skills (Physical Files on Disk)

| Skill Identifier | Location | File Size (Bytes) | Est. Full Content Tokens | Progressive Metadata Tokens |
|---|---|---|---|---|
| [`omni-qualification`](skills/omni-qualification/SKILL.md) | `.agents/skills/omni-qualification/` | 4,978 B | ~1,310 | 80 (Active) |
| [`omni-spec-conformance`](skills/omni-spec-conformance/SKILL.md) | `.agents/skills/omni-spec-conformance/` | 4,961 B | ~1,306 | 91 (Active) |
| [`omni-research-to-spec`](skills/omni-research-to-spec/SKILL.md) | `.agents/skills/omni-research-to-spec/` | 4,764 B | ~1,254 | 85 (Active) |
| [`omni-architecture-review`](skills/omni-architecture-review/SKILL.md) | `.agents/skills/omni-architecture-review/` | 3,000 B | ~790 | 90 (Active) |
| [`omni-session-handoff`](skills/omni-session-handoff/SKILL.md) | `.agents/skills/omni-session-handoff/` | 2,866 B | ~755 | 91 (Active) |
| **Total Skills Disk Footprint** | — | **20,569 B** | **~5,415 (Est.)** | **437 (Active)** |

---

## 3. Headroom & Loading Behavior Policy

1. **Independent Budget Ceilings**:
   - Budget A (Rules) and Budget B (Skills) are strictly independent 20,000-token pools.
   - Rules active utilization is **27.7%** (14,453 tokens reserved headroom).
   - Skills metadata active utilization is **2.2%** (19,563 tokens reserved headroom).
2. **Protection Against Budget Overflow**:
   - When a budget ceiling is exceeded, Antigravity demotes large rules to path pointers and excludes excessive customizations.
   - To prevent critical loss of invariants, all essential Omni requirements (specification authority, No-Override Principle, Rust safety, memory models, Candidate 1/2/3 governance) are preserved in the compact always-active files (`AGENTS.md`, `GEMINI.md`, `omni-core-principles.md`).
   - Detailed multi-step procedures remain in modular skills to be progressively loaded only when activated.
3. **Headroom Reservation**:
   - Unused capacity is intentionally preserved as headroom for ongoing compiler evolution, rather than consumed unnecessarily.
