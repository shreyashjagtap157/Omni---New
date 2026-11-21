# Omni Discrepancy Register

**Repository:** C:\Users\siddh\Downloads\ABC\Omni---New  
**Branch:** main (up to date with origin/main)  
**HEAD:** 27c5565  
**Created:** 2026-11-19  
**Status:** Active — open items require governed resolution, not unilateral edits

---

## Purpose

This register records exact conflicts between repository artifacts (specification, grammar, manifest, registry, ADRs, compiler code, tests) so they can be resolved through the proper candidate/edition/ADR process. Per AGENTS.md §3.3 and `.agents/rules/omni-specification-governance.md`, no implementation change may silently override normative specifications, and no manifest digest, grammar text, registry lifecycle state, or parser profile may be altered to manufacture false consistency.

Entries below are **read-only observations**. None have been resolved by editing any artifact. Resolution requires explicit approval/authority.

---

## Entry D-001: VIBE-GGRAM productions in normative EBNF vs. unratified Candidate 2/3 status

**Contradiction:** `spec/grammar/omni-edition1.ebnf` ends with a trailing block labelled
`(* VIBE-GRAM extensions – normatively required productions that the baseline EBNF omitted *)`
(lines 392-396) defining `pipeline_operator`, `newline_termination`, `projection_shorthand`,
and `command_call`. The comment calls them "normatively required productions." Yet those same
constructs are unratified Candidate 2 / Candidate 3 syntax.

**Provenance / lifecycle state of each side:**

| Artifact | Location | Statement | Status |
|---|---|---|---|
| AGENTS.md §3.2.4 | Appended EBNF VIBE-GRAM productions are "unratified errata and exploratory candidates; not active in the baseline grammar contract" | Inactive | Unratified |
| spec/grammar/candidate2-erratum.md | `status: pre-release`, `signature_state: pending`, `implementation_impact: "none"`, "not signed or applied; Candidate-1 baseline continues to govern" | Pre-release | Not applied |
| docs/adr/0003 | ADR-0003 accepted: strict Edition 1 default; candidate syntax only via explicit `ExperimentalVibeSyntax` parser profile; "does not modify the normative grammar, manifest, registry lifecycle states, release digest" | Accepted implementation decision | Boundary enforced |
| docs/grammar-reconciliation.md | VIBE-GRAM-0001…0007 "not in force"; "This defect is real and it is not fixed by editing the EBNF"; `LEX-0002`/`GRAM-0002`/`GRAM-0003` are Edition 1 baseline in force | Open defect | Not in force |
| spec/grammar/omni-edition1.ebnf | Trailing block calls VIBE-GRAM productions "normatively required productions that the baseline EBNF omitted" | Bytes inside `spec_tree_sha256` digest | Contradicts the above |
| spec/registry/rules.json | `VIBE-GRAM-*` rules recorded with lifecycle status `Candidate`, not `Ratified` | Candidate | Not ratified |
| compiler/omni-parse | `Parser::from_source` selects `ParserProfile::Edition1` and rejects `|>`, `?.`, prefix-dot, `_` expression; `ExperimentalVibeSyntax` opts into experiments | Default profile = strict Edition 1 | Implemented per ADR-0003 |

**Exact bytes in conflict:** `spec/grammar/omni-edition1.ebnf` lines 392-396:
```
(* VIBE-GRAM extensions – normatively required productions that the baseline EBNF omitted *)
pipeline_operator = (expression) "|>" expression;
newline_termination = statement "\n";
projection_shorthand = struct_field "=" identifier;
command_call = identifier "(" [ expression { "," expression } [ "," ] ] ")";
```
These bytes participate in `spec_tree_sha256 = 3986c2a71d9e5e4092331d025079055f34b98aa4179de1684b64e28bde74e6ed`
bound by `spec/manifest/omni-edition1.manifest.json` and `spec/release/foundation-gate.json`.

**Risk:** A reader could interpret the EBNF comment as making VIBE-GRAM constructs normative Edition 1 authority, while ADR-0003, the candidate2 erratum, the registry, and grammar-reconciliation all state they are not in force. This is a genuine authority conflict.

**Classification:** Editorial/documentation defect in the EBNF comment (the word "normatively required" overstates the status of unratified productions). It is NOT a change to syntax, semantics, or compatibility.

**Required authority for resolution:** The grammar comment should be brought into alignment with ADR-0003 and the candidate2 erratum through the controlled specification process. This does NOT require changing the manifest digest, registry lifecycle, parser profile, or production parser — only rewording the EBNF comment (which would still rebind `spec_tree_sha256` and require manifest/gate digest updates through the established process). Any such change requires explicit approval and cannot be done unilaterally.

**Decision (this session):** Recorded only. No bytes changed. ADR-0003 remains the accepted boundary.

---

## Entry D-002: Foundation gate status field vs. ADR-0002 (proposed, not accepted)

**Contradiction:** `spec/release/foundation-gate.json` declares `OMNI-IMP-0.0.0.13` as `"status": "qualified"` with empty `blocked_reasons`. However, `docs/adr/0002-foundation-gate-qualification-evidence.md` is `Status: proposed (not accepted)` and explicitly states formal qualification remains unresolved, and `docs/foundation-gates.md` states "formal sign-off unresolved" and that the `qualified` field must not be treated as independent sign-off.

**Provenance:**

| Artifact | Location | Statement | Status |
|---|---|---|---|
| spec/release/foundation-gate.json | `"gate": "OMNI-IMP-0.0.0.13", "status": "qualified"` | Qualified | Declared in JSON |
| docs/adr/0002 | Status: proposed (not accepted); "Define what OMNI-IMP-0.0.0.13 means and what evidence is required before it can be called formally qualified." | Proposed | Not accepted |
| docs/foundation-gates.md | "0.0.0.13 gate closure (technical checks passed; formal sign-off unresolved)"; "master mandate §68 explicitly says not to infer qualification from that field alone" | Pre-sign-off | Documented |

**Risk:** Calling the foundation gate "formally qualified" based solely on the JSON status field overstates the evidence. Technical CI checks passed on specific commits, but formal sign-off is unresolved.

**Classification:** Clarificatory / documentation alignment issue. The JSON status field is a consistency declaration, not independent formal qualification.

**Required authority:** ADR-0002 must be accepted (or an alternative chosen) by the project owner/maintainer to define formal qualification scope, evidence threshold, and sign-off authority.

**Decision (this session):** Recorded only. No bytes changed.

---

## Entry D-003: rust-toolchain.toml targets omit Windows despite "Windows first" mandate

**Observation:** `rust-toolchain.toml` declares `targets = ["x86_64-unknown-linux-gnu", "riscv64gc-unknown-none-elf"]`. The host is Windows (x86_64-pc-windows-msvc) and the master goal mandates Windows as the first fully supported platform. No Windows target is declared in the toolchain, and no CI workflow targets Windows (`.github/workflows/` only runs on `ubuntu-latest`).

**Risk:** Windows support cannot be claimed or qualified without a declared Windows target and Windows CI. This is a platform-gap, not a contradiction within the specification.

**Classification:** Platform gap / roadmap item. Windows toolchain target and CI must be added to qualify Windows support.

**Required authority:** Implementation decision within delegated authority (adding a target and CI), but must be coordinated with the Windows installer/distribution work.

**Decision (this session):** Recorded. To be addressed during Windows platform qualification work.

---

## Entry D-004: IMPLEMENTATION_INVENTORY.md contains known overclaims (from prior session)

**Overclaims identified and to be corrected:**

| # | Overclaim in inventory | Corrected finding |
|---|---|---|
| 1 | "20 crates" test count (TST-001) | Workspace has 25 members (18 compiler + 7 tooling), per live `Cargo.toml` |
| 2 | Single `Verified` source-to-executable based on "interpreter exit code 30 and native object emission only" | Tests DO link and run native executables via `cc`/`cl.exe` and verify exit codes (driver main.rs lines 2340-2358, 2526-2540, 2558-2572, etc.), but the public `--native` CLI flag emits an object file without linking. Link/run verified at test level, not through a user-facing CLI command. |
| 3 | Normative EBNF (GOV-001) marked `Verified` | EBNF contains the unresolved VIBE-GRAM contradiction (D-001). Not cleanly Verified until governed resolution. |
| 4 | Foundation gate marked "qualified" (GOV-003) | ADR-0002 is proposed, not accepted; formal sign-off unresolved (D-002). |

**Classification:** Inventory correction. No product bytes changed.

**Decision (this session):** Correct IMPLEMENTATION_INVENTORY.md entries to reflect these findings.

---

## Entry D-005: All 567 registry rules are `Candidate`; zero `Ratified` rules

**Observation:** `spec/registry/rules.json` holds 567 rules (557 Candidate + 3 Superseded + 7 STAGE0 Candidate), all with `"status": "Candidate"`. `docs/foundation-gates.md` §Evidence schemas confirms "all 567 rules Candidate (Ratified obligations vacuous)," zero E-code diagnostics, zero witness records.

**Risk:** `omni-audit` linkage reaper and `omni-conform` can pass vacuously because witness obligations attach only to `Ratified` rules. No rule-level conformance evidence exists.

**Classification:** Evidence gap, not a contradiction. Rule ratification is a future milestone.

**Decision (this session):** Recorded. No bytes changed.

---

## Resolution Policy

- No entry in this register may be "resolved" by editing spec bytes, manifest digests, registry lifecycle states, parser profiles, or production parser behavior without explicit approval through the governed process.
- Editorial corrections to comments or documentation that do not change normative behavior may be made autonomously if they preserve approved behavior and are recorded here.
- Any change affecting syntax, semantics, public interfaces, compatibility, safety invariants, or major architecture requires escalation to the user/project owner.
