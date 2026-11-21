# OMNI — UPDATED MASTER GOAL AND CONTINUOUS IMPLEMENTATION PROMPT

**Project:** Omni — Edition 1 Compiler and Toolchain  
**Canonical repository:** https://github.com/shreyashjagtap157/Omni---New  
**Primary branch:** `main` only  
**Prompt revision:** 2026-11-20  
**Next engineering milestone:** Edition 1 Semantic Integrity and Differential Qualification

---

## 1. Mission

Continue the design, research, implementation, verification, documentation and qualification of **Omni**, a general-purpose, multi-paradigm programming language and its production compiler/toolchain.

The immediate objective is to progress beyond the recently reported package/build tooling, CI expansion, conformance tests, language-server work, indirect-call code generation and Windows platform work. The next engineering milestone is:

> **Edition 1 Semantic Integrity and Differential Qualification** — establish that the normative Edition 1 semantics are preserved and enforced through source processing, semantic analysis, HIR, MIR, verification, reference execution and the supported native execution path.

This is an implementation mandate, not a planning-only exercise. Research and implementation must proceed together. Do not spend the whole task rewriting plans or reporting already-completed work.

The project is not complete merely because the current inventory is mostly verified, the workspace passes, native objects can be emitted, or the compiler can execute a small supported subset. Keep milestone qualification, language conformance, production readiness and self-hosting as distinct claims.

## 2. Repository, history and execution policy

The canonical repository is `https://github.com/shreyashjagtap157/Omni---New`. Do not use the older Omni repository.

### Mandatory repository policy

- Work on `main` only. Do not create feature branches.
- Inspect the actual local checkout before changing anything: repository identity, branch, HEAD, remote refs, `git status`, staged and unstaged changes, untracked files, recent history and relevant worktrees.
- Fetch/inspect remote state when appropriate, but do not overwrite local work.
- Preserve every existing user change, including uncommitted and untracked files. Inventory them before modifying files or staging anything.
- Never use destructive reset/clean/checkout operations to discard work.
- Never force-push, rewrite history, rebase shared history, or silently amend prior commits.
- Do not push, merge, open a pull request, change branch protection or otherwise write to the remote unless explicitly authorized.
- Do not stage unrelated files. Stage only files belonging to the reviewed implementation slice.
- Make small, coherent, reviewable local commits only when commit authorization is present and repository state is safe.
- Do not claim local changes are present on GitHub until the remote commit proves it.

### Known remote observation — must be refreshed

The most recently inspected public `origin/main` snapshot was `27c55651d8d20c4d46d1fb28a3c440770b9d4363` (short SHA `27c5565`, November 9, 2026). At that snapshot, some local changes described in later reports were not yet present remotely. This is an observation, not a permanent assumption: inspect the current local and remote states again at task start and report their exact relationship.

Do not mistake the public remote snapshot for the user's latest local worktree. Local uncommitted changes and commits not pushed to origin must be preserved and evaluated independently.

## 3. First task: reconstruct the real status and reconcile the inventory

Before implementing new functionality, reconcile the current implementation inventory against its authoritative source (requirements registry, milestone plan, release manifest or other designated source of truth).

A previous status report stated:

- Verified: 18 — COMP-001 through COMP-017, INST-003, COMP-021, COMP-022 and PLAT-001.
- Partial: COMP-020.
- Blocked: INST-002.
- Total: 20.

The listed verified identifiers do not arithmetically match the stated count: COMP-001 through COMP-017 are 17 identifiers, and the four additional identifiers make 21 listed verified identifiers. Together with the two remaining identifiers, that would be 23, not 20. Other reports have used different inventory totals. Treat all reported counts as provisional until the authoritative inventory is inspected.

Required reconciliation:

1. Locate the authoritative inventory and all documents that define the COMP, INST and PLAT identifiers.
2. Enumerate every identifier, including COMP-018/019 or any other reserved, omitted, retired, duplicated or untracked entries. Do not assume an identifier exists or is mandatory solely from a number sequence.
3. Reconcile every item against implementation source, tests, CI runs, release evidence and acceptance criteria.
4. Distinguish `verified`, `partial`, `accepted`, `blocked`, `deferred`, `not started`, `not applicable` and `unverified` states where the governance model supports them. Do not collapse decision acceptance into technical qualification.
5. Record evidence against exact commits and CI run IDs. A test suite passing on one commit does not establish the same result for a different commit.
6. Correct the counts only after the complete enumeration is proven.
7. Update all status documents consistently, preserving historical reports and their dates.

For each requirement, record at minimum:

- Exact identifier and authoritative requirement text.
- Scope and acceptance criteria.
- Implementation locations.
- Tests and test names.
- Exact commit SHA and workflow/run identity.
- Reproducible evidence artifact locations and checksums where applicable.
- Independent review evidence, if required.
- Remaining limitations and blockers.
- Final status and rationale.

Do not use a green workspace build as a substitute for feature-by-feature acceptance evidence.

## 4. Platform and distribution policy — Windows is the only currently supported OS

**Windows is the only officially supported operating system for the current development and initial production scope.** The policy does not prohibit future Linux/macOS support or constrain the long-term language architecture.

### Required Windows qualification

- Windows is the mandatory supported development and initial release platform.
- Validate the pinned Rust toolchain, build, tests, code generation, native linking, executable execution, installer, upgrade/repair/uninstall and relevant developer workflows on native Windows runners and the user's Windows environment where available.
- Windows CI must remain the required platform qualification path.
- Do not claim Windows support for a capability that has only been cross-compiled or object-emitted; distinguish compilation, linking, execution and installer qualification.

### Linux/macOS compatibility experiments

Use GitHub-hosted native runners to test compatibility without requiring the user to own or provide physical Linux/macOS machines:

- `ubuntu-latest` for experimental Linux compatibility.
- `macos-latest` for experimental macOS compatibility.
- `windows-latest` for mandatory Windows qualification.

Keep Linux/macOS compatibility jobs separate from the mandatory Windows qualification gate. Their failures must remain visible and actionable; do not suppress them with `continue-on-error`, ignore rules, fabricated passes or misleading labels. Do not make those jobs required checks for the Windows-only release gate unless the owner explicitly changes the policy.

A successful Linux/macOS CI run is evidence of the exact tested compatibility surface on that runner and commit only. It does **not** grant official platform support, qualify a distribution, or authorize publishing Linux/macOS installers. Do not claim those platforms are supported until a separate support decision and complete qualification are made.

COMP-020 must remain visible as future platform expansion / experimental compatibility work. It must not be silently removed, marked complete merely because a workflow file exists, or used to block the current Windows-only release scope when the approved scope does not require it.

## 5. Production code signing policy

INST-002 production-trusted publisher signing remains blocked until a legitimate, trusted code-signing certificate and required publisher identity are available.

- Keep test signing, ephemeral self-signed certificates and production-trusted signing as distinct states.
- Do not claim that a self-signed test certificate establishes trusted publisher identity.
- Continue Windows compiler, build, test, installer and internal qualification work without waiting for production signing.
- Do not purchase a certificate or other paid service without explicit authorization.
- Never place private keys, signing credentials, tokens or certificate secrets in source control, logs or this prompt.
- Public production-trusted installer distribution remains unqualified where trusted signing is mandatory.

## 6. Qualification governance and ADR-0002

Preserve the distinction between technical CI outcome and formal qualification.

The previously authorized direction for ADR-0002 was **Alternative C: separate technical-gate outcome from formal qualification**. Do not ask the owner to approve Alternative C again as though no decision existed. Inspect the latest local evidence and record the decision accurately.

However, distinguish:

- The ADR decision being accepted.
- A qualification-record schema being implemented.
- Technical gate CI passing on a specific commit.
- A formal qualification record being complete.
- An owner/reviewer attestation being bound to the exact qualified commit and evidence.

A qualification record must identify the exact source commit, normative specification digest, required CI run IDs, evidence references, scope, reviewer/owner identity and decision. Do not fabricate an attestation, signature or approval. If a formal attestation remains pending, report it as pending.

Never infer formal qualification from a JSON field, empty blocker list, successful build, or passing CI alone. Do not confuse qualification of a foundation/tooling milestone with qualification of the complete compiler, language, runtime or product release.

## 7. Primary engineering milestone: Edition 1 semantic integrity

Start by checking whether the latest local implementation already resolved the gaps below. The remote implementation baseline is historical relative to later local work; do not blindly repeat its claims. Verify the actual current code, tests and evidence first.

### 7.1 Place-aware ownership and reference provenance — first priority

Inspect at least:

- `compiler/omni-own`
- `compiler/omni-verify`
- `compiler/omni-mir`
- `compiler/omni-hir`
- `compiler/omni-types`
- Normative Edition 1 ownership, type, borrowing, reference, call and return rules.
- All existing ownership, MIR, parser, driver and conformance regressions.

The previously documented limitations include:

- Reference-to-loan association centered on MIR locals rather than arbitrary places.
- Incomplete references stored in projected places and aggregates.
- Incomplete dereference-origin resolution.
- Incomplete move/provenance transfer through projected and aggregate storage.
- Incomplete call-argument and call-return provenance.
- Limited control-flow join coverage.
- Nested reborrow and reference-lifetime interactions requiring further qualification.
- An unresolved normative question about shared-reference copyability.

Determine which are still real in the current checkout. For unresolved items, design and implement a sound, maintainable, place-aware provenance model in small vertical slices.

The model and its tests should cover, as required by the actual Edition 1 semantics:

1. Reference origins associated with places and projections.
2. References contained in aggregates and extracted from aggregate places.
3. Copy/move/assignment transfer rules for reference-valued places.
4. Reborrows, parent-child loan relationships and release conditions.
5. Dereference-origin resolution and access through references.
6. Aliasing and conflict detection for reads, writes, moves, drops and mutable reborrows.
7. Calls, argument passing, function boundaries, return values and result destinations.
8. Control-flow joins, loops and path-sensitive provenance states.
9. External, parameter, local, heap/global or other origins only where those origins are defined and modeled.
10. Fail-closed behavior for missing, ambiguous, stale or untracked provenance.
11. Tests proving both invalid operations are rejected and valid programs are not rejected solely because the analysis is unnecessarily conservative.

Do not create a broad rewrite before understanding the current abstractions. Prefer a minimal architectural change with strong invariants and adversarial tests.

#### Normative decision guard: reference copyability

Shared-reference copyability must not be decided by implementation convenience or by importing Rust behavior. Inspect the complete in-force Edition 1 specification and compatibility requirements. Comparative languages, including Rust, are research evidence only.

Until the normative rule is clear and appropriately approved:

- Do not silently implement general shared-reference copy propagation.
- Do not modify the normative grammar, semantics, registry, specification digest or rule lifecycle to force an implementation through.
- Keep unsupported cases fail-closed and document the exact limitation.
- Present a decision request only if the actual normative authority is insufficient and the decision is genuinely owner-level.

### 7.2 MIR initialization, moves, drops and unsafe assumptions

After the ownership/provenance foundation is sufficiently qualified, inspect and complete the relevant MIR semantics:

- Initialization-state tracking and uninitialized storage.
- Partial initialization and partial moves.
- Move paths and projected-place state.
- Assignment and overwrite semantics.
- Drop elaboration, drop flags and conditional drops.
- Destruction order and cleanup across all control-flow exits.
- Panic/unwind behavior only where the language/runtime specification defines it.
- Unsafe-region assumptions and their verifier obligations.
- Rejection of structurally valid but semantically invalid MIR.
- Avoidance of unjustified rejection of valid programs.

Every operation must preserve enough semantic information for later MIR verification and execution. Add targeted adversarial regressions before broad refactoring.

### 7.3 Effects and capability enforcement

Connect the effect/capability model to the actual production compilation pipeline.

Audit and implement, according to the normative rules:

- Effect declarations and effect-row preservation through lowering.
- Effect-row constraints and solving.
- Function signature and call-site compatibility.
- Generic/effect-polymorphic behavior where specified.
- Capability authority, provenance and availability.
- Authorization of effectful operations.
- Rejection of unauthorized or undeclared effects.
- Correct behavior at function, closure, task and control-flow boundaries where applicable.
- MIR verification of preserved effect/capability obligations.

Do not treat an enum, type definition, parser node or isolated model as complete enforcement. Demonstrate the full source-to-verifier path with negative and positive programs.

## 8. Reference execution machine and native execution parity

The reference machine must become a dependable semantic execution model, not merely a minimal demo interpreter.

Audit its actual completeness against Edition 1 and the supported MIR subset. Develop in dependency order:

- Value and type representation.
- Memory/object representation and place operations.
- Initialization and validity.
- Move, borrow, alias and resource lifetime behavior.
- Aggregate construction, access and destruction.
- Control flow, calls and returns.
- Defined operations, effects and capability checks.
- Defined traps, runtime errors and termination.
- Deterministic observable behavior.

Fail closed where semantics are not established. Keep unsupported language features clearly separated from implemented ones.

### Native backend

Preserve the completed indirect-call emission work. Do not reimplement it merely because this milestone is starting. Verify its integration with the current calling convention, function-pointer representation, return ABI, target ABI, relocations, object emission, linking and native execution.

The supported native execution path for the current product scope is Windows. Validate through the real Windows toolchain, linker and executable process. Object-file creation is not native execution; a successful link is not runtime correctness.

### Differential qualification

Build or extend an automated harness that runs the same supported programs through:

1. The reference abstract machine.
2. The Cranelift native backend on the officially supported Windows platform.

Compare the complete set of specified observable outcomes, including as applicable:

- Return values and process exit status.
- Standard output and error.
- Termination versus defined traps/errors.
- Defined memory and resource observations.
- Other externally observable effects specified by Edition 1.

Do not compare unspecified or undefined behavior as though it had a required identical result. Record the observation model and normalization rules explicitly.

Include indirect calls, direct calls, control flow, aggregates, supported ABI cases, ownership, borrowing, drops, effects and capabilities as each becomes qualified. Every divergence must produce a minimized reproducible test and a tracked root-cause resolution. Preserve test seeds, source files, build identity, runtime identity and result artifacts.

## 9. Specification authority and syntax policy

The normative Edition 1 specification under `spec/` remains authoritative. Implementation behavior must not be invented when normative text is silent or contradictory.

- Trace implemented behavior to normative rule identifiers.
- Verify the actual EBNF and specification digest authority.
- Keep Candidate 2 and Candidate 3 material non-normative unless formally ratified.
- Do not enable `|>`, optional chaining, projection shorthand, placeholder syntax, command-call syntax or other unratified forms in the Edition 1 production parser.
- Preserve strict Edition 1 behavior while allowing explicitly isolated experimental profiles only where already authorized.
- Do not change rule statuses, normative text, grammar, manifests or digest-bound files without a coordinated authority review and the required approval.
- Record specification defects and contradictions rather than inventing semantics to make code pass.

The long-term developer experience may include a more approachable, vibe-oriented surface. However, complete and qualify the conventional, explicit Edition 1 syntax and semantics first. Do not begin a new vibe-syntax phase prematurely or destabilize Edition 1.

## 10. Security, dependency and build discipline

- Use the pinned Rust toolchain and locked dependency graph.
- Inspect current advisories, licenses, provenance and supply-chain risks; do not assume historical audit results still apply.
- Investigate the previously documented `ruzstd` advisory history against the current lockfile and dependency graph. Do not claim the issue is present or resolved without current evidence.
- Preserve `unsafe_code = "forbid"` where established and do not weaken lint policy to make checks pass.
- Keep warnings and Clippy failures visible; do not add broad suppressions without a specific, documented justification.
- Keep test fixtures and generated artifacts out of production source unless intentionally checked in.
- Remove only temporary artifacts created by the current task, after verifying their paths and ownership. Never clean the user's existing worktree.
- Keep reproducible builds, checksums, provenance and release evidence tied to exact commits.

## 11. Validation contract

Run focused tests after each small implementation slice, followed by the complete relevant qualification set. Use repository scripts and the pinned toolchain rather than approximations.

At minimum, evaluate the applicable commands and gates:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --workspace --locked
cargo build --workspace --locked
cargo test --workspace --locked
cargo metadata --format-version 1 --locked
```

Also run the authoritative project validators for:

- Specification manifest and canonical digest binding.
- Rule linkage and witness obligations.
- Compiler crate topology and dependency direction.
- Parser and source conformance.
- MIR structural and semantic verification.
- Ownership/provenance negative and positive tests.
- Reference-machine execution.
- Differential qualification.
- Windows native linking and executable execution.
- Installer and release qualification where relevant.
- Security/dependency audits and CodeQL workflows.

Do not assume every command above is sufficient by itself. Inspect the scripts and current workflow to identify the complete mandatory gate. Do not weaken gates to get green checks.

For every reported validation result, record:

- Exact command or workflow/job.
- Exact source commit and dirty/clean state.
- Exit code and test counts.
- Toolchain and relevant host/target identity.
- Evidence artifact path and checksum where applicable.
- Known omissions, skipped tests and environmental constraints.

Tests, compilation, object emission, native linking, execution, conformance and formal qualification are different evidence levels. Label each accurately.

## 12. Three-stage generational roadmap

Preserve the approved generational strategy:

### Stage 1 — Rust-based minimal working core

Target: `1.0.0.0`.

Deliver a coherent, semantically sound minimal compiler and execution path with an explicit supported language subset, dependable validation, source-to-executable workflows and documented limitations.

### Stage 2 — Production-capable Rust platform

Target: `2.0.0.0`.

Complete the production compiler, semantic analyses, native backend, runtime, standard library, package/build tooling, diagnostics, language server, debugging/developer workflows, Windows distribution and qualification evidence. Validate representative real applications, not only synthetic unit tests.

### Stage 3 — Omni self-hosting

Target: `3.0.0.0`.

Implement and independently qualify the Omni-written compiler and bootstrap chain. Retain the Rust implementation as a trusted reference/bootstrap implementation during the transition. Demonstrate reproducible bootstrap stages and semantic equivalence; self-hosting alone is not proof of correctness.

Do not skip semantic completeness or production qualification to begin Stage 3 prematurely.

Keep the four-component product version `major.minor.patch.revision` distinct from implementation milestones, language editions, crate versions, ABI versions, runtime versions and CI run numbers.

## 13. Documentation and traceability

Update the relevant durable documentation alongside implementation changes. Inspect current paths and naming conventions before adding documents.

Maintain consistency among:

- Authoritative requirements/inventory.
- Implementation baseline.
- Implementation roadmap and milestone status.
- Normative specification and rule registry.
- ADRs and approved decisions.
- Conformance and differential test catalogues.
- CI workflow documentation.
- Security/dependency records.
- Changelog and session handoff.
- Release/qualification manifests and evidence records.

Do not overwrite historical snapshots. Clearly label historical reports and superseding evidence. Avoid maintaining multiple contradictory “current” status files; designate one authoritative current status source and link to supporting details.

## 14. Execution method and adversarial review

Use a continuous but bounded engineering loop:

1. Inspect and establish facts from the current repository.
2. Identify the highest-priority unresolved semantic obligation.
3. Read the governing normative rules and relevant design/code.
4. Form a minimal design with explicit invariants and failure behavior.
5. Implement one small vertical slice.
6. Add focused positive, negative, boundary and regression tests.
7. Obtain an independent adversarial review of the changed slice.
8. Address review findings and verify no unrelated regressions.
9. Run the broader validation gates.
10. Update documentation and exact evidence records.
11. Commit locally only when authorized and safe.
12. Continue to the next dependency-ordered slice without waiting for an unnecessary “continue” prompt.

Do not inflate completion by counting tests without mapping them to requirements and semantic cases. Do not call a design model a production implementation. Do not call a passing unit test formal proof. Do not claim full language support from a narrow execution subset.

If a true owner-level normative or product decision blocks the next implementation, present the exact alternatives, consequences and recommendation once. Do not ask again for an already approved decision. Continue all independent work that does not depend on that decision.

## 15. Immediate execution order

After repository and inventory reconciliation, follow this order, adjusting only when current evidence proves a dependency or limitation has changed:

1. Verify the current local `main`, remote `main`, worktree and existing evidence. Preserve all user work.
2. Correct the COMP/INST/PLAT inventory arithmetic and establish its authoritative total.
3. Confirm the current Windows-only support policy and align local platform CI/documentation with it.
4. Reconcile Alternative C for ADR-0002 and its formal qualification record without inventing missing attestations.
5. Audit the current place-aware ownership/reference-provenance implementation. If the documented gaps remain, select and implement the smallest high-value unresolved provenance slice.
6. Expand MIR initialization, move, drop and unsafe-assumption semantics in dependency order.
7. Integrate and qualify effect/capability enforcement end to end.
8. Complete the supported reference-machine memory/execution semantics.
9. Build and expand differential reference/native execution qualification on Windows.
10. Continue production Stage 1/Stage 2 capabilities according to actual dependency and requirement evidence.
11. Keep Linux/macOS compatibility experiments and production signing as separately tracked platform/release obligations.
12. Update the authoritative status, implementation baseline, roadmap, changelog and handoff.

Do not automatically start every item in parallel. Preserve semantic dependencies and produce a coherent, qualified milestone.

## 16. Required handoff/report format

At the end of each meaningful execution cycle, report:

### Verified facts
- Repository, branch, local HEAD, remote HEAD and worktree condition.
- Exact authoritative inventory counts and how they were derived.
- Current platform support status.
- Exact existing implementation and evidence inspected.

### Decisions
- Previously approved decisions preserved.
- New owner-level decisions required, with alternatives and recommendation.
- Explicitly deferred matters.

### Implementation
- Exact features/slices completed.
- Files and modules changed.
- Semantic invariants established.
- Adversarial review outcome.

### Validation
- Commands, workflow runs, exit codes, test counts and evidence references.
- Whether each result proves formatting, compilation, unit behavior, integration, native execution, conformance or formal qualification.

### Remaining work
- Partial, blocked, deferred and unverified items.
- Exact blockers and whether they actually block Windows development or only distribution/support claims.
- Next concrete implementation slice.

### Git and remote state
- Local commits created, if authorized.
- Uncommitted changes preserved.
- Remote writes performed only if explicitly authorized.
- No claim that work is pushed unless remote state proves it.

The report must be factual, concise enough to audit, and detailed enough for another engineer or agent to continue without repeating completed work.

---

## Final operating instruction

**Do not stop at the point where the previously listed implementation inventory is green.** Reconcile it, then continue improving the actual compiler and language implementation. Prioritize semantic soundness, specification fidelity, reproducible evidence, Windows-native execution and maintainable architecture. Keep future platform support and trusted signing visible without allowing them to halt independent compiler progress. Preserve user work and repository history. Never overstate what has been built, tested, executed, qualified or released.
