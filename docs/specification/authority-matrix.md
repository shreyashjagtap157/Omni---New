# Omni Specification Authority Matrix

**Repository:** C:\Users\siddh\Downloads\ABC\Omni---New  
**Branch:** main  
**HEAD:** 27c5565  
**Generated:** 2026-11-20  
**Source:** Live spec/ tree, docs/adr/, AGENTS.md governance rules

---

## Purpose

This document establishes the authoritative specification lifecycle state for
every specification artifact in the Edition 1 baseline. Per AGENTS.md §3.1
(Domain-Specific Specification Authority Model), authority is established by
specific non-interchangeable artifact domains:

- **Normative Specification Artifacts** (`spec/reference/`, `spec/grammar/`):
  Define authoritative language semantics, typing rules, and grammar.
- **Edition Manifest** (`spec/manifest/omni-edition1.manifest.json`): Binds the
  selected specification snapshot and cryptographically pins integrity metadata
  (`spec_tree_sha256`).
- **Release Gates** (`spec/release/foundation-gate.json`): Govern qualification,
  milestone criteria, and release eligibility. A release gate is an operational
  qualification gate, not a substitute for a language specification.
- **Compiler Code, Tests & Plans:** Downstream engineering evidence and
  verification instruments. They provide implementation evidence, not normative
  language authority.

This document records the **current state of each artifact** and its
**authority tier**, per the governance matrix in AGENTS.md §3.2. It does not
modify any artifact bytes.

---

## Tier 1: Edition 1 Active Baseline (Candidate 1: 1.0.0-candidate.1)

**Authority:** Authoritative as bound by the Edition Manifest and governed by
the Release Gate. This is the active normative baseline. Candidate release
status must be preserved and must NOT be described as definitively ratified or
production-released without completed qualification gates.

| Artifact | Path | Status | Authority | Notes |
|----------|------|--------|-----------|-------|
| Edition Manifest | `spec/manifest/omni-edition1.manifest.json` | `1.0.0-candidate.1` | Binding | `spec_tree_sha256 = 3986c2a7...`; binds the normative spec tree |
| Release Gate | `spec/release/foundation-gate.json` | `OMNI-IMP-0.0.0.13` declared `"qualified"` | Operational gate (not formal sign-off) | Technical CI passed; formal sign-off unresolved under ADR-0002 |
| Normative EBNF | `spec/grammar/omni-edition1.ebnf` | Candidate 1 baseline | Normative | Contains appended VIBE-GRAM block (unratified, D-001) |
| Core Semantics & Types | `spec/reference/core_semantics_and_types.md` | Candidate 1 | Normative | Edition 1 type system, ownership, references |
| Vibe Surface Syntax | `spec/reference/candidate2_vibe_surface_syntax.md` | Candidate 2 | Pre-release | Not in force per candidate2-erratum |
| Registry Rules | `spec/registry/rules.json` | 567 rules (557 Candidate + 3 Superseded + 7 STAGE0 Candidate) | All Candidate | Zero Ratified; Ratified obligations vacuous |
| Rule Texts | `spec/registry/rule-texts.json` | 567 texts | Evidence | Mechanically rederivable hashes |
| Evidence Schemas (8) | `spec/schemas/*.schema.json` | v1.0.0 | Contract | diagnostic, witness, conformance-outcome, verification-failure, provenance, regression, fuzz-promotion |
| Qualification Record Schema | `spec/schemas/qualification-record.schema.json` | Referenced by ADR-0002 | Future | For formal owner-attested qualification record |

---

## Tier 2: Candidate 2 Erratum

**Authority:** Formally recorded as `status: pre-release`, `signature_state:
pending`, `implementation_impact: "none"`. NOT in force normatively for the
Edition 1 baseline.

| Artifact | Path | Status | Authority | Notes |
|----------|------|--------|-----------|-------|
| Candidate 2 Erratum | `spec/grammar/candidate2-erratum.md` | `pre-release`, `signature_state: pending`, `implementation_impact: "none"` | Pre-release | Explicitly NOT applied; Candidate-1 baseline continues to govern |
| Candidate 2 Spec Surface | `docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.2_vibe.md` | Candidate 2 | Pre-release | Referenced by erratum as replacement source; not ratified |
| VIBE-GRAM Rules | `spec/registry/rules.json` (VIBE-GRAM-0001–0007) | `Candidate` | Not ratified | Zero implementation impact |

---

## Tier 3: Candidate 3 Proposals

**Authority:** Exploratory research prototypes, NOT ratified Edition 1
semantics.

| Artifact | Path | Status | Authority | Notes |
|----------|------|--------|-----------|-------|
| Candidate 3 Spec | `docs/specification/Omni_Complete_Specification_Edition1_1_0_0-candidate_3_vibe_reconciled_corrected.md` | Candidate 3 | Experimental | Not ratified |
| Candidate 3 Spec (final reviewed) | `docs/specification/Omni_Complete_Specification_Edition1_1_0_0-candidate_3_vibe_reconciled_final_reviewed_v2.md` | Candidate 3 | Experimental | Not ratified |
| Parser Experiments | `compiler/omni-parse/src/grammar_contract.rs` | `candidate3_pipeline_and_optional_navigation_parse` | Experimental | `ExperimentalVibeSyntax` profile only |

---

## Governance Decisions (ADRs)

| ADR | Path | Status | Authority | Summary |
|-----|------|--------|-----------|---------|
| ADR-0001 | `docs/adr/0001-erratum-corrected-lifecycle-state.md` | Accepted | Implementation | Fixes `ErratumCorrected` state spelling in registry |
| ADR-0002 | `docs/adr/0002-foundation-gate-qualification-evidence.md` / `ADR-0002-DECISION-PACKAGE.md` | Accepted (Alternative C) | Project Owner | Separates technical gate outcome from formal qualification; formal sign-off PENDING |
| ADR-0003 | `docs/adr/0003-unratified-vibe-syntax-authority-boundary.md` | Accepted | Implementation | Strict Edition 1 default; candidate syntax only via explicit `ExperimentalVibeSyntax` profile; does not modify normative grammar/manifest/registry/digest |

---

## Conflict Resolution Record (D-Entries)

| Entry | ID | Issue | Resolution Status | Authority Required |
|-------|----|-------|-------------------|-------------------|
| D-001 | VIBE-GRAM EBNF block | EBNF trailing block calls VIBE-GRAM productions "normatively required" but they are unratified Candidate 2/3 | Recorded; no bytes changed | Controlled specification process; would rebind `spec_tree_sha256` |
| D-002 | Foundation gate status | JSON declares `qualified` but ADR-0002 is proposed/not accepted; formal sign-off unresolved | Recorded; no bytes changed | Project owner acceptance of ADR-0002 |
| D-003 | rust-toolchain.toml targets | Omits Windows despite Windows-first mandate | Recorded; to be addressed during Windows platform qualification | Implementation decision (delegated) |
| D-004 | Inventory overclaims | 4 overclaims identified (20 crates, source-to-executable, EBNF Verified, foundation gate qualified) | Corrected in inventory | None (editorial) |
| D-005 | Registry rules | All 567 rules Candidate; zero Ratified | Recorded; evidence gap | Rule ratification is future milestone |

---

## Authority Conflict Resolution Protocol

When specification, manifest, grammar, and compiler implementation disagree:

1. Record the exact conflict across rule IDs, file paths, and AST constructs.
2. Apply the domain authority model: normative spec defines semantics, manifest
   binds tree integrity, release gate checks criteria. Implementation code or
   tests never override normative specifications.
3. Block unsupported Edition 1 conformance claims.
4. Never silently modify the grammar, manifest, release digest, or compiler
   tests to manufacture false consistency.

---

## Unresolved Normative Questions

| Question | Artifact(s) | Status |
|----------|-------------|--------|
| Shared-reference copyability | `spec/reference/core_semantics_and_types.md` | Normative question until established by complete in-force specification |
| Memory model (managed vs affine) | `docs/research/memory-architecture-managed-vs-affine.md` | Open research area; Edition 1 specifies affine by default |
| Foundation gate formal sign-off | `spec/release/foundation-gate.json` + ADR-0002 | PENDING owner-attested record |
| VIBE-GRAM EBNF alignment | `spec/grammar/omni-edition1.ebnf` lines 392-396 | Open defect; editorial correction requires digest rebinding |
| Rule ratification | `spec/registry/rules.json` | 567 rules Candidate; future milestone |

---

## Audit Evidence

- All spec artifacts read and verified against live filesystem
- `spec_tree_sha256 = 3986c2a71d9e5e4092331d025079055f34b98aa4179de1684b64e28bde74e6ed`
  matches manifest and foundation gate
- All 5 ADRs reviewed; ADR-0001 accepted, ADR-0002 accepted (Alternative C),
  ADR-0003 accepted (Alternative A)
- No normative specification bytes modified during this audit
