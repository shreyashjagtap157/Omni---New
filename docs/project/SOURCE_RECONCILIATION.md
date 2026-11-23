# Omni — source reconciliation and requirement provenance

Classification: Informative

## 1. Canonical project identity and current checkout

| Item | Reconciled value |
|---|---|
| Canonical repository | `https://github.com/shreyashjagtap157/Omni---New` |
| Branch | `main` only |
| Live HEAD inspected | `8f4878e3f90cae7adda9ec95e4778aad2703e056` |
| Recursive tree | 618 entries; not truncated |
| Production compiler strategy | Rust-first (Approach C) |
| Experimental self-hosting | Parallel, isolated and non-authoritative until independently qualified |
| Normative source | Hash-bound Edition 1 specification under `spec/` |
| Long-term duration | Continuous evolution; no artificial completion ceiling |

The older `Omni` repository is historical and is not the canonical implementation source. The independent `Omni-Research` workspace is a related research effort, not a competing production compiler or semantic authority.

## 2. Source set and purpose

- Root master charter — durable product ambition, Rust-first development, four-component versioning, research, release and engineering policy.
- Root 20 November continuous-engineering mandate — current long-horizon implementation, qualification, security, handoff and continuity obligations.
- The 18-Nov first-iteration OpenCode master-goal prompt, 18-Nov v2 prompt, 20-Nov continuous-engineering mandate, 21-Nov local continuation mandate, gate-closure notes and handoffs — dated operational snapshots. The early prompt's Stage-1-conventional/Stage-2-optional-Vibe sequence is explicitly reaffirmed by the 20-Nov mandate; historical machine/commit/worktree claims must be revalidated.
- `.agents/rules/` and `.agents/skills/` — durable agent and repository workflows.
- `spec/manifest/omni-edition1.manifest.json`, `spec/grammar/omni-edition1.ebnf`, `spec/registry/rules.json`, `spec/reference/` — domain-specific language specification authority; inspect each artifact's actual status and hash binding.
- `docs/adr/`, `docs/foundation-gates.md`, `spec/release/` — decisions and qualification gates.
- `IMPLEMENTATION_INVENTORY.md`, `docs/implementation-baseline.md`, plans, tests, discrepancy/risk registers and CI — implementation evidence at exact revisions.
- [October conversation-integrated design compendium](archive/Omni_Conversation_Integrated_Design_Compendium_2026-10-13_NON_NORMATIVE.md) — preserved generated, non-normative synthesis.
- [Owner working preferences](OWNER_WORKING_PREFERENCES_AND_REPOSITORY_EXPECTATIONS.md) — operating style and expected repository end state.
- Independent `Omni-Research` workspace — exploratory evidence only; its findings enter implementation through source review, ADR and formal specification governance.

## 3. Authority and precedence

1. The active, hash-bound language specification and domain-specific authority model govern Edition 1 semantics. Check the manifest, registry, grammar and accepted ADRs; not every file under `spec/` is automatically normative.
2. The owner master charter and active continuous-engineering mandate govern intended product scope and execution obligations, but do not silently amend language semantics.
3. Explicit accepted ADRs and formal amendments govern approved changes within their scope. Pending Candidate 2 and Candidate 3 experiments are not active merely because their files/tests exist.
4. Current source, tests and live CI at an exact revision determine implementation state. Old plans/handoffs do not override live evidence.
5. Historical compendia and independent research workspaces are informative. Conflicts must be recorded, not silently harmonized.

## 4. Recovered durable owner decisions

- One general-purpose, multi-paradigm, multi-level language and complete development platform; not a narrow compiler demo or AI-dependent language.
- The 19-Nov master charter preserves the long-term Vibe-first ambition; the newer 20-Nov long-horizon mandate stages it after the initial conventional, explicit, developer-familiar production syntax passes first-stage qualification. The later Vibe-first surface is optional and separately governed. Both remain deterministic, formally specified and statically verifiable.
- Coherent high-level, intermediate and low-level programming tiers.
- Rich type inference, abstraction, memory/ownership, effect/capability, concurrency, numerics, modules, runtime, libraries, compiler, package/build and developer tooling ambitions.
- Preserve the managed-memory versus Edition 1 affine-ownership tension as an explicit research/open decision; the active Edition 1 rules remain in force.
- Rust-first production compiler, parallel experimental self-hosting and a controlled, independently qualified bootstrap transition.
- Continuous research and reassessment without a fixed project endpoint.
- Separate specification, formalization, implementation, test, proof, review, qualification and release evidence.
- Four-component `X.Y.Z.W` project release identity, distinct from editions, specification candidates, engineering gates, crate versions, ABIs, runtimes and artifact versions.
- The 20-Nov mandate spells the product format as `major.minor.patch.revision`; the master charter also records `X.Y.Z.W` and leaves its relationship to the older `R.M.m.P` milestone scheme for a dedicated versioning ADR. Preserve historical `OMNI-IMP-0.0.0.13` engineering references until that governance decision; do not confuse technical gate passage with owner sign-off or a published product release.
- Immutable releases, compatibility/migration discipline, provenance and complete product-release gates.
- Omni uses canonical `main` only; no feature branches; no destructive Git operations or history rewriting.
- Durable handoffs, autonomous dependency-ready execution, clear specialist ownership and independent verification.

## 5. Candidate syntax and memory-model boundaries

Candidate 2 remains pending/unratified in the recorded baseline; Candidate 3 parser tests remain research probes. The later optional Vibe-first phase is not permission to activate either candidate or appended EBNF. Pipeline `|>`, historical projection shorthand, `?.`, prefix-dot projection shorthand, placeholder expressions, command-call syntax and other inactive constructs are not normative Edition 1 merely because older prompts, compendia or grammar sections mention them. Stage 1 remains conventional and explicit; any later Vibe-first surface requires its own formal grammar, lowering, diagnostics, compatibility and approval.

Historical managed-memory-by-default statements and the current Edition 1 affine-ownership model form a real design tension. Keep it visible for research and governed decision; this project-requirements document introduces no new language semantics.

## 6. Historical questionnaire recovery gap

The master charter refers to an original language-design interview containing more than 160 decisions and asks that the original questionnaire and answers be recovered. The full raw questionnaire/answer transcript was not present in the available materials searched for this reconstruction. The master charter and October compendium are syntheses, not proof of every original answer. Do not invent missing questions or answers; retain this as an explicit provenance-recovery task.

## 7. Live verification snapshot — 23 November 2026

Observed public `main` HEAD: `8f4878e3f90cae7adda9ec95e4778aad2703e056`.

- Current Rust push validation passed on Linux and Windows.
- macOS failed in `omni-codegen` test `test_indirect_function_pointer_call_verifies_emits_and_executes`: Apple `ld` rejected a text relocation from `_omni_main` to `_inc`; 29 passed and 1 failed in that crate job.
- Separate strict Clippy, Security Audit and CodeQL workflows passed.
- First-stage readiness was informational and does not grant qualification.
- Inventory status has been reconciled to **22 Verified / 2 Partial / 1 Accepted / 1 Blocked** (26 mandatory items). `COMP-019` remains Partial for package/build placeholders; `COMP-020` is Partial because the current declared macOS end-to-end matrix is failing; `ADR-0002` is Accepted but the formal owner-attested qualification record remains pending; `INST-002` remains Blocked pending genuine production-trusted signing identity.
- A distinct dynamic Cargo/fuzz dependency-update run also failed and should be investigated separately.

No language grammar, normative registry, Edition 1 manifest, product version or compiler implementation was changed by this requirements-documentation work.

## 8. Maintenance

Update this document when source provenance, authority or durable owner decisions materially change. Update the user requirements baseline when a new durable requirement is recovered or explicitly approved/superseded. Amend normative sources only through existing specification governance.
