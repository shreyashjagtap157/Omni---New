# OMNI — Long-Horizon Continuous Engineering, Qualification, Production Platform and Self-Hosting Mandate

**Revision:** 2026-11-20  
**Canonical repository:** https://github.com/shreyashjagtap157/Omni---New  
**Canonical branch:** `main`  
**Primary implementation language:** Rust  
**Current official product OS:** Windows  
**Long-term objective:** A complete, dependable, multi-paradigm programming language, production compiler/toolchain, runtime and standard library, followed by independently qualified Omni self-hosting.

---

# 0. EXECUTION MANDATE — DO NOT STOP AFTER EACH SMALL TASK

You are the continuing engineering agent for Omni. This document is a long-horizon execution mandate, not a one-issue task ticket and not a planning-only request.

Your responsibility is to continue research, architecture, implementation, testing, adversarial review, qualification, documentation and release engineering across the roadmap below.

## 0.1 Continuous execution rule

- Do not stop merely because one bug, requirement, test failure, workflow, milestone or document has been completed.
- Do not ask the project owner whether to continue after each task, slice, phase or milestone.
- After completing a task, immediately select the next highest-priority unblocked task in the dependency graph and continue.
- At a milestone boundary, verify the milestone and automatically proceed to the next planned milestone.
- If one item is blocked by an external resource, isolate that blocker, record the exact required resource and continue all independent work.
- Do not wait idle for a person, runner, certificate, approval or remote workflow. Continue with other useful work while that dependency is pending.
- Do not repeatedly ask for information already present in repository records, prior owner decisions, this mandate or the current working context.
- Ask the project owner only when a genuinely new decision changes normative language semantics, compatibility, public commitments, spending, legal obligations, security posture, irreversible operations or remote repository state.
- When a decision is needed, research the alternatives, state the impact, recommend a default and ask one consolidated question. Do not turn routine engineering choices into approval gates.
- When a task is complete and independently verified, update the queue and move forward. Do not endlessly revisit completed work unless new evidence invalidates it.

Continue for as much of the available execution session as possible. Never claim that work continues in the background after the session ends. At the end of an execution session, leave a durable handoff with the exact next task so the next session can resume without rediscovery.

## 0.2 Autonomous engineering authority

Proceed without asking for routine approval on:

- Inspecting source, specifications, tests, CI, build scripts, logs, artifacts and dependencies.
- Researching current technical approaches and authoritative documentation.
- Making reversible implementation decisions consistent with the normative specification and existing architecture.
- Fixing root causes and adding regression tests.
- Improving deterministic validation, tooling, diagnostics, documentation and test coverage.
- Removing demonstrably unused code or dependencies only after proving they are unused and validating the impact.
- Creating local implementation commits on the existing `main` branch when the repository is in a safe, understood state and the commit contains only the intended work.
- Updating implementation status, handoffs, changelogs, traceability and evidence after changes.

Do not do the following without the project owner's explicit authorization:

- Push commits, merge pull requests, create remote branches, publish releases, publish packages or installers, create release tags, or change remote repository settings.
- Force-push, rewrite Git history, reset, clean, discard or overwrite user work.
- Purchase certificates, services, software, hardware or subscriptions.
- Obtain, store, expose or use production credentials, private keys, PFX files or signing passwords.
- Change a normative language rule, Edition 1 compatibility contract, memory model, concurrency model, effect semantics, reference-copyability rule or ABI contract when the specification is silent or conflicting.
- Make a legal, licensing, distribution or security commitment on behalf of the project owner.

Do not stage or commit unrelated user changes. Never treat the existence of a remote write permission as authorization to use it.

## 0.3 Blocker-isolation policy

A blocked item is not a reason to stop the project.

For each blocker:
1. Identify the exact missing resource or decision.
2. Verify that it is genuinely unavailable through already-connected tooling, GitHub-hosted infrastructure, existing credentials or existing repository data.
3. Record why it cannot be self-provisioned safely.
4. Identify work that does not depend on it.
5. Continue that independent work.
6. Revisit the blocker only when new evidence or access becomes available.

Never fabricate credentials, certificates, signatures, approvals, runner results, test results or evidence.

---

# 1. REPOSITORY IDENTITY, SAFETY AND SOURCE OF TRUTH

Canonical repository:

`https://github.com/shreyashjagtap157/Omni---New`

Use `main` only. Do not switch to or revive the older Omni repository.

## 1.1 Initial reconciliation on every continuation

Before editing:

1. Inspect the actual local repository path, branch, HEAD, remotes, status, staged diff, unstaged diff and untracked files.
2. Inspect recent commits and their complete diffs.
3. Fetch or otherwise inspect the current `origin/main` without changing the local worktree.
4. Compare local `main` and `origin/main`, including ahead/behind counts and changed paths.
5. Inspect the current authoritative requirements inventory, specification digests, release records, CI workflows and session handoffs.
6. Identify local changes that predate the current task and preserve them.
7. Establish which claims are verified on the current commit and which are only historical evidence from earlier commits.

Never infer local state from the public GitHub snapshot or from an old handoff when a live checkout is available.

## 1.2 Current externally observed snapshot — recheck before relying on it

The following snapshot was independently inspected on 2026-11-20:

- Remote `main`: `8581706f13a1fcd50996b06b63a6e6a069f1ed88`.
- Commit subject: `Complete COMP-020: publish GitHub-hosted validate-linux and validate-macos jobs; add ADR-0002 qualification record schema`.
- Workflow run: [44746310922](https://github.com/shreyashjagtap157/Omni---New/actions/runs/44746310922).
- The run is **completed with overall conclusion `failure`**, not still running.
- `validate-linux`: passed.
- `validate-macos`: passed.
- `validate-windows`: failed at the toolchain assertion step.
- The older `validate` job: failed in the memory-model research prototype.
- `stabilize-rc`: reported success, but its inventory gate and report generation contain correctness defects described below.

Treat this as a snapshot to verify, not permanent state. Always query the latest commit and Actions run before making a current claim.

## 1.3 Preserve established project constraints

- Edition 1 normative specification remains the implementation authority.
- Rust is the production compiler implementation language for Stages 1 and 2.
- Keep the Rust implementation as a dependable reference implementation during and after the self-hosting transition.
- Work on `main` only.
- Preserve the four-component product version format `major.minor.patch.revision`.
- Do not confuse product version, language edition, crate version, ABI version, runtime version, implementation milestone or roadmap stage.
- Keep Candidate 2 and Candidate 3 syntax/semantics gated unless formally approved and ratified.
- Do not enable `|>`, `?.`, prefix-dot projection shorthand, placeholder expressions, command-call syntax or other unratified syntax in the Edition 1 production driver.
- Complete the conventional, explicit, developer-familiar language surface before considering the later optional Vibe-first syntax phase.
- Keep existing source history, test corpus and qualification evidence intact.

---

# 2. CURRENT PLATFORM AND WORKFLOW POLICY

## 2.1 Supported operating system

Windows is the only officially supported product operating system for the current development and initial production scope.

The Windows qualification path is mandatory.

Linux and macOS are experimental compatibility targets for now. Successful experiments are useful evidence, but they do not grant official platform support, create release obligations or authorize Linux/macOS installer publication.

Use GitHub-hosted native runners:

- Windows: `windows-latest`.
- Linux: `ubuntu-latest`.
- macOS: `macos-latest`.

For additional optional architecture coverage, investigate Intel macOS runners after the existing ARM64 macOS experiment is stable. Do not make that expansion a blocker for the currently defined COMP-020 acceptance criteria unless the authoritative requirement says it is mandatory.

The repository is public. Standard GitHub-hosted runners for public repositories are available without a separate paid runner subscription. Do not ask the owner to provision physical Linux/macOS computers, WSL, Docker or private runner infrastructure when GitHub-hosted runners satisfy the test requirements.

## 2.2 COMP-020 current evidence

Run `44746310922` produced the following job results on exact commit `8581706f13a1fcd50996b06b63a6e6a069f1ed88`:

| Job | Result | Interpretation |
|---|---|---|
| `validate-linux` | Success | Native Ubuntu build/check/test and specification-manifest binding passed |
| `validate-macos` | Success | Native macOS build/check/test and specification-manifest binding passed |
| `validate-windows` | Failure | Failure occurred before build/tests because the workflow invoked a missing PowerShell assertion script |
| `validate` | Failure | Legacy Ubuntu job failed compiling a research-only memory-model test |
| `stabilize-rc` | Success | Do not treat as valid gate passage until its false-positive inventory logic and stale report are corrected |

Linux and macOS platform-experiment jobs have therefore both produced successful results for this exact commit. Reconcile the authoritative requirement acceptance criteria and, if they are exactly these native jobs, update COMP-020 to Verified with the run ID, commit SHA, runner labels, job conclusions and evidence links.

Do not call the entire CI run successful. Do not treat a successful Linux/macOS job as a successful Windows qualification or a green overall run.

## 2.3 Root causes visible in the current CI run

### A. Windows toolchain assertion path

The Windows job invokes:

`./ci/assert-toolchain.ps1`

The workflow log shows that PowerShell cannot find that script. The commit tree's `ci/` directory contains `assert-toolchain.sh`, `gate-0-0-0-13.sh`, `pipeline.sh` and `.gitkeep`; it does not contain `assert-toolchain.ps1`.

This is a concrete repository/workflow defect, not an unexplained runner outage.

Resolve it properly:

1. Inspect the pinned toolchain contract in `rust-toolchain.toml`.
2. Inspect `ci/assert-toolchain.sh` and `ci/assert-toolchain.ps1` references across the whole repository.
3. Implement the missing PowerShell equivalent or redesign the shared assertion interface so it works correctly on Windows.
4. Ensure the Windows job installs exactly the targets that the Windows assertion expects.
5. Verify pinned Rust version, host triple, required components and `x86_64-pc-windows-msvc` target.
6. Verify any additional target required by the relevant foundation gate separately; do not make a per-job assertion demand targets that the job never installs.
7. Ensure paths, quoting, PowerShell exit handling and line endings work on a clean GitHub-hosted Windows runner.
8. Add tests for parser/validation behavior where practical.
9. Re-run the complete Windows job, not just the assertion step.

Do not weaken the toolchain assertions to obtain a green check.

### B. Legacy `validate` job

The older Ubuntu `validate` job fails compiling:

`research/memory-model/domain_lifetimes.rs:471`

The compiler reports `E0425`: `cannot find value 'object' in this scope`, from:

`let _ = object;`

The same research test previously passed on another commit, but the current commit fails. Treat the exact current code as authoritative and diagnose the scope/lifetime/unused-variable mistake. Do not assume it is harmless or suppress the test.

Fix the root cause and add or preserve regressions. Then determine whether this job should be:
- retained as a foundation/research qualification job;
- consolidated into `validate-linux`; or
- split into a separate research-prototype workflow.

Any consolidation must preserve every unique validation currently performed by the legacy job, including focused ownership tests, semantic linkage, compiler topology and the Foundation 0.0.0.13 gate. Do not silently delete coverage.

Avoid wasteful duplicate workspace builds where the same validation is already performed elsewhere, but do not drop checks solely to reduce runtime.

### C. Release-Candidate Stabilization inventory logic

The current `stabilize-rc` job has an invalid acceptance rule:

- It counts matching text rather than reading a canonical machine-readable inventory.
- It passes if the count equals the total **or simply exceeds 15**.
- The inspected commit tree does not contain `IMPLEMENTATION_INVENTORY.md`, so the step can warn and still exit successfully.
- The generated stabilization report contains hardcoded stale counts/statuses such as “17 of ~25+” and stale statements that COMP-020 is untested, INST-003 is not implemented and PLAT-001 is under evaluation.
- It writes a hardcoded `cargo build: PASSED` line without an actual standalone `cargo build` command in that job.

Replace this with a fail-closed, machine-readable validator driven by the canonical requirements inventory. It must:
1. Read the real authoritative inventory.
2. Validate exact unique requirement IDs and states.
3. Reject missing IDs, duplicate IDs, unknown IDs, inconsistent totals and invalid transitions.
4. Calculate counts from records, not text matching.
5. Compare against the exact gate acceptance criteria.
6. Fail when any mandatory item is not Verified.
7. Keep Accepted, Partial, Blocked, Experimental and Not Applicable distinct from Verified.
8. Generate the report from actual job outputs, commit identity and evidence references.
9. Never hardcode a test result or a completion count.
10. Include negative tests proving that partial inventories, missing evidence, incorrect counts and blocked items cannot pass.

Do not make a “greater than 15” fallback or other heuristic completion threshold.

### D. Workflow structure

Keep Windows qualification mandatory and experimental platform qualification clearly separated.

A suitable structure is:
- Mandatory Windows CI and installer qualification workflow.
- Linux/macOS compatibility experiment workflow.
- Foundation/research qualification workflow, if it has distinct checks.
- Security/dependency analysis workflow.
- Release-candidate workflow that consumes real evidence from the exact commit.

It is acceptable to retain a single workflow file if separation is explicit and semantically correct. Prefer a dedicated `.github/workflows/platform-compatibility-experiment.yml` for Linux/macOS so those jobs do not become mandatory Windows branch-protection checks.

Do not suppress experimental failures with `continue-on-error`, `|| true`, skipped checks or fake success. Their failures must remain visible as experimental compatibility failures without falsely blocking the official Windows support claim.

For macOS, record the actual host triple and architecture. The current runner was reported as Apple Silicon/ARM64 while the job explicitly installed `x86_64-apple-darwin`; distinguish native-host execution from cross-target compilation. If the acceptance criterion is native macOS execution, prove the host architecture from the job rather than inferring it from the runner label.

Use immutable action references or the repository's established action-pinning policy, least-privilege workflow permissions, explicit timeouts and artifact retention appropriate to qualification. Review action/runtime deprecation warnings instead of ignoring them.

---

# 3. REQUIREMENTS INVENTORY AND FIRST-STAGE GATE

The current owner-provided inventory is described as 26 mandatory items:

- 23 Verified.
- COMP-020 Partial/Unblocked pending macOS completion in the earlier snapshot; both Linux and macOS have since passed on run `44746310922`.
- ADR-0002 Accepted, with commit-bound record work pending.
- INST-002 Blocked pending a production-trusted code-signing certificate.

Recompute the exact current count from the authoritative inventory after inspecting the local checkout. Do not hardcode 23/26 or assume the status remains unchanged.

## 3.1 COMP-020

If the authoritative acceptance criteria require successful native Linux and macOS jobs and no additional criteria remain, promote COMP-020 to Verified using run `44746310922` as evidence.

Record:
- Full commit SHA.
- Run URL and run ID.
- `validate-linux` success.
- `validate-macos` success.
- Runner labels and host architecture.
- Exact commands and validations completed.
- Any limitations on platform scope.

Keep the platform policy unchanged: Windows remains the sole officially supported product OS.

## 3.2 INST-002

Production-trusted Windows Authenticode signing remains blocked until an appropriate genuine code-signing certificate or approved signing service is provisioned.

The following are separate:
- Local self-signed test certificate.
- Test-signing workflow validation.
- Production-trusted publisher identity.
- Production release signing and timestamp verification.

Continue all compiler, runtime, tests, packaging, installer engineering and non-production distribution work while INST-002 is blocked.

Do not purchase a certificate or service. Do not ask for or expose a private key, PFX file or password. Research trusted certificate options and requirements, document the minimum owner action, and leave the blocker visible.

Do not falsely mark INST-002 Verified through a self-signed certificate or by disabling signature checks.

## 3.3 ADR-0002 — preserve the already-approved decision

The project owner has previously approved **Alternative C**: technical CI outcome and formal qualification are distinct states, with any formal qualification bound to a specific commit and its evidence.

Do not ask the owner to choose Alternative C again.

Reconcile the actual ADR document, which may still say `proposed`, with the previously approved owner decision. Correct the status and record the decision provenance accurately.

Alternative C does not automatically mean:
- the entire Omni language is complete;
- the first-stage 26-item gate has passed;
- the foundation technical gate has passed on the latest commit;
- a cryptographic signature exists; or
- all normative rules are ratified.

Keep those qualifications separate.

## 3.4 Qualification record schema — audit and strengthen

The published schema is:

`spec/release/qualification-record.schema.json`

Audit it and its consumers. The currently published schema requires only:
- `commit_sha`
- `technical_status`
- `verified_item_count`
- `total_mandatory_items`
- `verified_gaps`
- `formal_status`

The following properties are currently optional even though they are essential to a meaningful owner-attested record:
- `attester`
- `attestation_date`
- `spec_digest`

The current schema also lacks a strict full-SHA pattern for `commit_sha` and has no direct field for exact CI evidence/run references. Its `verified_gaps` enum is narrowly tied to the present three known gaps.

Strengthen it based on the real qualification model. Include, where appropriate:
- exact 40-character lowercase commit SHA;
- explicit qualification scope and gate identifier;
- technical result separate from formal approval;
- total and verified counts validated against the actual inventory;
- exact gap IDs and states;
- exact specification digest;
- relevant CI run IDs, URLs, job names, conclusions and artifact references;
- attester identity and UTC attestation timestamp;
- approval/decision identity and the accepted ADR revision;
- explicit limitations and blocked reasons;
- schema version and record version.

Require essential fields for a formally approved record. Add a validator and positive/negative schema tests. Do not rely solely on JSON Schema for cross-field arithmetic or inventory completeness: validate those invariants in code.

A committed JSON record is not automatically a cryptographic signature. If cryptographic attestation is required, use a real supported signing mechanism and owner-controlled key. Never invent or copy a signature.

Bind the record to the exact commit it qualifies. If the commit changes, re-run relevant checks and create a new record or an explicitly versioned superseding record. Do not keep an old SHA after modifying the source.

## 3.5 The 26/26 claim

The user-provided first-stage gate requires all 26 mandatory items to be Verified.

If COMP-020 becomes Verified and ADR-0002's required owner attestation is completed, INST-002 remains blocked. Therefore the result would be 25/26, not 26/26, unless the authoritative gate policy explicitly excludes production-trusted signing from this particular gate.

Do not claim “one item away” while a mandatory signing item remains unverified.

Do not waive or redefine INST-002 unilaterally. Research whether it is truly a first-stage compiler qualification requirement or specifically a public signed-installer/release requirement. If the approved requirements state it blocks the gate, keep the gate blocked until the certificate exists. If the existing specification clearly separates internal qualification from signed public distribution, represent those as separate gates without changing the underlying requirement.

Use precise gate names:
- Foundation technical gate.
- First-stage implementation inventory gate.
- Windows platform qualification.
- Public trusted-installer distribution gate.
- Complete Rust production-platform gate.
- Self-hosting bootstrap qualification.

Do not merge their outcomes.

---

# 4. IMMEDIATE QUEUE — EXECUTE IN THIS ORDER, THEN CONTINUE

Do not stop after completing these items. They are the immediate repair queue before the long-term compiler roadmap.

## Q1. Reconcile exact current state

Inspect local and remote `main`, current run state, all CI workflows, all relevant scripts, the schema, ADR, requirement inventory and exact status evidence.

Identify every uncommitted user change. Preserve it.

Produce an internal dependency graph and a short task queue, then begin implementation immediately.

## Q2. Fix the Windows toolchain assertion

Resolve the missing `ci/assert-toolchain.ps1` failure, add robust platform-aware validation and test it. Then execute the full Windows validation job.

## Q3. Fix the research-prototype compile failure

Resolve `E0425` at `research/memory-model/domain_lifetimes.rs:471`. Preserve semantics and add the relevant regression. Keep research prototypes explicitly isolated from production compiler components.

## Q4. Consolidate CI without losing validation coverage

Remove or consolidate only genuinely redundant workflow execution. Preserve every unique semantic, topology, ownership, foundation, security and platform check. Keep Linux/macOS experiments independent of mandatory Windows support.

## Q5. Replace false-positive release qualification

Implement the strict machine-readable inventory validator, accurate artifact generation and negative tests. The RC job must fail when the inventory is incomplete or evidence is missing.

## Q6. Finish the inventory evidence

Promote COMP-020 only when the authoritative acceptance criteria are satisfied by the exact run. Reconcile ADR-0002 with Alternative C and produce the correctly scoped, commit-bound record. Keep INST-002 blocked unless a genuine production credential is available.

## Q7. Run the complete qualification set on the corrected exact commit

Run Windows mandatory qualification, Linux/macOS experiments, foundation checks, all workspace validations, security analysis, release-evidence generation and schema/inventory tests. Preserve logs and artifacts.

## Q8. Establish a trustworthy first-stage baseline

Update:
- authoritative requirements inventory;
- implementation status;
- roadmap;
- README;
- implementation baseline;
- ADR and qualification records;
- changelog;
- CI documentation;
- session handoff.

Record exact source commit and evidence. Then proceed immediately to the next unblocked engineering milestone.

---

# 5. LONG-HORIZON ROADMAP — DO NOT END AT THE FIRST-STAGE GATE

The project has three generational stages. Completing one stage is a transition, not a reason to stop.

| Stage | Objective | Target release |
|---|---|---|
| Stage 1 | Rust-based minimal working Omni compiler and core toolchain | `1.0.0.0` |
| Stage 2 | Complete production-capable Rust compiler, runtime, standard library and developer platform | `2.0.0.0` |
| Stage 3 | Omni-written, independently qualified self-hosting compiler and bootstrap system | `3.0.0.0` |

These are acceptance targets, not claims that the stages are currently complete.

## 5.1 Stage 1 — complete and qualify the language implementation

Continue through each semantic and implementation layer. Determine current completion from source and evidence; do not blindly redo completed features.

### A. Source model and lexical analysis

Audit and complete all applicable normative source and lexer rules, including:
- UTF-8 handling and source identity.
- Unicode version pinning and identifier properties.
- Normalization and security-sensitive Unicode behavior.
- Bidi/control/noncharacter handling.
- Comments, whitespace and trivia preservation.
- Numeric literals and overflow boundaries.
- String, raw-string and interpolated-string escape handling.
- Nested interpolation and balanced delimiters.
- Source spans, offsets and diagnostic stability.
- Deterministic lexical errors.

Build adversarial tests, fuzz tests and minimized regression cases.

### B. Syntax and parsing

Verify the normative Edition 1 EBNF against the production parser.

Cover:
- every normative production;
- expression precedence and associativity;
- declarations, patterns, types, generics and attributes;
- block and statement boundaries;
- parser recovery;
- ambiguous and malformed input;
- exact source preservation and lossless CST round-tripping;
- diagnostics with stable source locations.

Treat the grammar as authority. A parser implementation detail cannot establish a new language rule.

Keep experimental syntax profiles explicitly isolated from the Edition 1 production driver.

### C. Names, modules and packages

Complete and qualify:
- module and package identity;
- visibility and privacy;
- imports and re-exports;
- cyclic dependency behavior;
- namespace separation;
- shadowing and resolution order;
- deterministic diagnostics;
- package boundaries and public API identity;
- dependency resolution and lockfile semantics.

Ensure module resolution is deterministic and reproducible across clean environments.

### D. Type system and compile-time semantics

Audit:
- type representation and canonicalization;
- inference and unification;
- generic constraints;
- trait resolution and coherence;
- associated types and method lookup;
- monomorphization;
- const evaluation and compile-time execution;
- function and closure types;
- aggregate layouts and enum semantics;
- casts, conversions and numeric behavior;
- error localization and diagnostic quality.

Implement only behavior supported by normative Edition 1 semantics.

### E. Ownership, references and memory-model semantics

Inspect the current production implementation before changing it. Do not assume historical baseline gaps still exist.

Qualify:
- affine/move semantics;
- place-sensitive initialization;
- loans and borrow regions;
- reborrows and parent-child relationships;
- control-flow joins;
- reference origin and provenance;
- references in projections and aggregates;
- dereference origin;
- call argument and return provenance;
- lifetime and outlives constraints;
- alias/conflict detection;
- drops and destruction;
- resource lifetime and deterministic cleanup.

The previous provenance and ownership work must be treated as completed only to the extent established by current source and evidence. Preserve all completed regressions.

Normative questions that must not be invented include:
- shared-reference copyability;
- the relation between managed memory and affine ownership;
- the final memory-domain model;
- concurrency and memory-ordering semantics;
- resource transfer across asynchronous boundaries.

Research these questions against the full in-force specification, historic owner requirements and credible technical literature. Record alternatives and compatibility implications. Continue unaffected implementation work while genuinely normative decisions remain open.

Do not silently adopt Rust semantics as Omni semantics.

### F. Effects and capabilities

Complete actual semantic enforcement, not just type definitions:
- effect rows and constraints;
- effect propagation through calls and control flow;
- function effect contracts;
- capability authority and derivation;
- operation authorization;
- effect/capability interaction with ownership;
- MIR preservation and independent verification;
- deterministic diagnostics for unauthorized operations.

### G. HIR and MIR

Ensure:
- canonical HIR preserves all semantic obligations and source provenance;
- lowering is deterministic;
- types and declarations retain identity;
- MIR represents all supported control flow and operations;
- place projections are sound;
- calls, indirect calls and returns preserve ABI contracts;
- initialization, moves, drops and unsafe assumptions are explicit;
- verification is independently invocable and fail-closed.

Do not allow lower layers to reconstruct or invent semantic information that should have been preserved from higher layers.

### H. Independent MIR verifier

Treat the verifier as a security and correctness boundary.

Add positive and negative tests for:
- invalid CFG edges and terminators;
- type/operand mismatches;
- invalid projections;
- invalid initialization and move paths;
- ownership and loan conflicts;
- reference provenance;
- call and return boundaries;
- invalid drop states and order;
- unsafe-region assumption violations;
- malformed or adversarial MIR.

Avoid both unsound acceptance and accidental rejection of valid programs. Every conservative rejection must have an explicit reason and a tracked path toward qualification.

### I. Reference abstract machine

Complete the reference machine as an executable semantic oracle for the supported language subset.

Qualify:
- value representation;
- memory and address model;
- allocation and object lifetime;
- initialization and destruction;
- functions and indirect calls;
- aggregates and layout;
- control flow;
- defined runtime errors and traps;
- effects and capability checks;
- deterministic output and observable behavior.

Keep unsupported behavior explicit and fail-closed. Do not treat a unit-test model as a complete production execution engine.

### J. Native backend and executable pipeline

The Cranelift backend's indirect-call full emission has already been reported as implemented and verified. Preserve it and its regression tests; do not reimplement completed code without new evidence of a defect.

Continue to qualify:
- supported scalar and aggregate ABI;
- direct and indirect calls;
- returns and parameter passing;
- stack and data layout;
- control-flow lowering;
- bounds and safety checks;
- runtime helpers;
- relocation and object generation;
- native linking;
- executable loading and execution;
- deterministic native behavior.

Keep object generation, linking, execution and runtime correctness as distinct evidence claims.

Windows MSVC x64 is the mandatory official native host target. Continue GitHub-hosted Linux/macOS experiments independently.

### K. Runtime, standard library and FFI

Build and qualify the actual production runtime and standard library.

Include:
- memory and allocation contracts;
- strings, collections and numeric primitives;
- I/O and filesystem APIs;
- process and environment access;
- errors and panic/trap behavior;
- concurrency/task APIs only after their semantics are specified;
- resource management;
- C ABI interoperability;
- stable header and symbol contracts;
- ABI/version compatibility tests.

C API support must be validated with real C compilation, linking, invocation, return-value checks and error handling. A header-only or symbol-only test is insufficient.

### L. Compiler driver, build and package system

Complete the user-facing toolchain:
- source compilation;
- project/workspace manifests;
- dependency graph resolution;
- locked reproducible builds;
- incremental compilation;
- build caching with correct invalidation;
- output directory and artifact identity;
- diagnostics and exit codes;
- package creation and verification;
- installer integration;
- checksums, SBOM and provenance;
- offline and clean-machine installation.

Do not silently update dependencies or lockfiles.

### M. Developer tooling

Continue beyond a minimal language server:
- complete LSP initialization and lifecycle;
- diagnostics, publish and clear;
- hover and signature help;
- completion and resolve;
- go-to-definition and references;
- rename and code actions;
- document/workspace symbols;
- formatting;
- semantic tokens;
- incremental document updates;
- cancellation and concurrency correctness;
- stable Windows path/URI handling.

Add a formatter, debugger integration, useful CLI help, project templates, documentation generation and REPL only after their architecture and scope are justified.

Tooling must use the canonical compiler front-end and must not maintain contradictory parsing or semantic logic.

## 5.2 Stage 2 — production Rust platform

Once Stage 1's approved acceptance criteria pass, continue automatically into Stage 2.

The target is a dependable production toolchain, not just a compiler that passes a workspace test suite.

### A. Runtime and standard-library production readiness

Complete the runtime, standard library, memory/resource behavior, panic/trap contracts, I/O and FFI. Establish API stability, documentation, compatibility policy and migration mechanisms.

### B. Production compiler reliability

Qualify:
- deterministic clean builds;
- incremental builds and cache correctness;
- large workspaces;
- parallel compilation;
- reproducible diagnostics;
- error recovery;
- compiler crash resistance;
- memory/time/resource limits;
- stable ABI and object format behavior;
- safe failure on unsupported features.

### C. Windows product experience

Windows is the only officially supported OS for this initial production scope.

Complete:
- MSVC native compilation and linking;
- stable installer and uninstaller;
- user/system installation behavior as specified;
- safe PATH handling;
- upgrades, repair, rollback and uninstall;
- clean-machine validation;
- signed release pipeline once a trusted certificate is provisioned;
- clear diagnostics and recovery;
- update/rollback mechanism only under approved governance;
- release channels and version compatibility.

Keep the production signing blocker separate from internal build/test qualification.

### D. Security and supply chain

Maintain:
- dependency audit and advisory response;
- least-privilege CI;
- action pinning policy;
- artifact provenance;
- SBOM generation and verification;
- reproducible builds;
- source and binary checksums;
- secure installer/update design;
- vulnerability disclosure and response;
- threat models for compiler, package manager, runtime, FFI and build scripts.

Do not claim supply-chain security from a single passing audit.

### E. Quality, performance and scale

Create repeatable benchmark suites for:
- front-end throughput;
- type checking and trait solving;
- MIR construction and verification;
- code generation;
- link time;
- incremental build performance;
- compiler memory use;
- generated executable size;
- runtime performance for defined workloads.

Compare against established languages only with fair, reproducible methodology. Optimize measured bottlenecks without weakening semantic correctness.

Add fuzzing, property-based testing, mutation tests, stress tests, long-running regression suites and clean-machine qualification.

### F. Ecosystem and documentation

Prepare:
- language reference;
- standard-library reference;
- tutorial and practical guides;
- compiler architecture documentation;
- package authoring documentation;
- FFI guides;
- migration/versioning policy;
- security and support policy;
- release notes and known limitations;
- developer onboarding and contribution workflows.

Do not claim a broad ecosystem exists until real tooling, libraries and users support that claim.

## 5.3 Stage 3 — self-hosting and bootstrap trust

Do not start self-hosting as a substitute for finishing the Rust implementation.

After the production Rust platform is qualified, begin the self-hosting transition.

### A. Define the bootstrap contract

Specify:
- bootstrap compiler versions;
- source-to-binary trust chain;
- compiler stage identities;
- compiler artifact provenance;
- reproducibility requirements;
- stage comparison methodology;
- allowed bootstrap dependencies;
- long-term independence from non-Rust implementation components.

### B. Implement Omni components in Omni

Port compiler components incrementally, in dependency order:
- source and lexer;
- parser and CST;
- names and module resolution;
- type system;
- ownership/effects/capabilities;
- HIR and MIR;
- MIR verifier;
- reference machine;
- native backend;
- runtime and standard tooling.

Keep the Rust implementation as an independent semantic and behavioral reference during this work.

### C. Differential bootstrap and trust

Require:
- Rust compiler and Omni compiler to agree on the supported conformance corpus;
- reproducible binaries;
- independent rebuilds;
- stage-to-stage comparison;
- stable bootstrap artifacts;
- clear trust boundaries;
- documented exceptions and limitations.

Self-hosting alone does not prove correctness. A self-hosted compiler must pass the same semantic, conformance, security and reproducibility standards as the Rust implementation.

### D. Final Stage 3 qualification

Do not declare `3.0.0.0` merely because the compiler can compile itself.

Require independent qualification, reproducible bootstrap, stable runtime/tooling, conformance evidence, security review and an auditable source-to-binary trust chain.

---

# 6. LANGUAGE DESIGN, RESEARCH AND FUTURE EVOLUTION

## 6.1 Preserve the singular Edition 1 authority

The current in-force Edition 1 specification is authoritative. Maintain one coherent normative contract and its canonical digest.

Do not implement a rule because:
- a comment suggests it;
- a third-party language does it;
- the parser already accepts it;
- an example uses it;
- a research prototype models it;
- an AI agent proposed it.

Every normative behavior change requires source-backed specification analysis and the proper approval process.

## 6.2 Vibe-first evolution

Preserve the long-term ambition for a highly expressive, approachable and productive multi-paradigm language.

Do not sacrifice semantic predictability or conventional developer familiarity in the initial production language.

Complete the conventional explicit syntax and its compiler behavior first. Treat any later Vibe-first syntax as a separate optional language evolution phase with explicit grammar, lowering, diagnostics, compatibility and approval. Do not enable unratified surface syntax in Edition 1.

## 6.3 Memory model and concurrency research

Continue research and prototype work in isolated locations. Compare plausible memory and resource models, including affine/ownership-first and managed-domain approaches, without prematurely ratifying a design.

Separate:
- storage reclamation;
- aliasing and access control;
- object lifetime;
- deterministic resource cleanup;
- concurrency and memory ordering;
- task/closure capture;
- region promotion and escape behavior.

A prototype proves only the transitions it actually models. Do not copy research-only code into the production compiler without a separate specification, architecture and safety review.

## 6.4 Ongoing technical research

Continue monitoring authoritative developments in:
- compiler architecture and intermediate representations;
- type systems and proof techniques;
- borrow/ownership and memory safety;
- capability security;
- effect systems;
- native code generation and ABIs;
- package/build systems;
- language-server protocols;
- supply-chain security;
- reproducible builds;
- self-hosting and bootstrap verification.

For each proposed adoption, record:
- source and publication date;
- applicability to Omni;
- maturity and maintenance status;
- security and licensing implications;
- compatibility risks;
- implementation cost;
- measurable benefit;
- decision and reason for adopting or rejecting.

Research must inform implementation, but must never silently supersede Omni's normative contract.

---

# 7. TESTING, REVIEW AND QUALIFICATION STANDARD

## 7.1 Required verification pattern

For every meaningful change:

1. State the requirement and semantic obligation.
2. Identify the authoritative specification rule(s).
3. Inspect the current implementation.
4. Identify root cause or missing behavior.
5. Implement a small coherent slice.
6. Add positive tests.
7. Add negative/adversarial tests.
8. Run focused tests.
9. Run relevant integration and conformance tests.
10. Run the full workspace checks when appropriate.
11. Conduct an independent adversarial review.
12. Fix discovered defects.
13. Re-run affected validation.
14. Update traceability, baseline, changelog and handoff.
15. Commit only the intended work when safe.
16. Select the next task and continue.

## 7.2 Minimum Rust workspace validation

Use the pinned toolchain and locked dependency graph. Run the actual repository validation commands, including as applicable:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --workspace --locked
cargo build --workspace --locked
cargo test --workspace --locked
cargo metadata --format-version 1 --locked
```

Run repository-specific:
- toolchain assertions;
- conformance runners;
- specification manifest/digest checks;
- registry and rule linkage;
- crate topology checks;
- foundation gate;
- installer qualification;
- native C API integration;
- security/dependency audits;
- provenance, SBOM and reproducibility checks.

Do not report a command as passed unless it was actually executed and its exit status was observed.

## 7.3 Adversarial review

For every safety-critical or semantic change, have an independent reviewer examine:
- unsound acceptance;
- incorrect rejection;
- missing control-flow paths;
- stale or forged provenance;
- aliasing and lifetime violations;
- overflow and representation errors;
- platform-dependent behavior;
- error handling and recovery;
- test gaps;
- bypasses in CI or qualification logic.

The reviewer must inspect the actual diff and tests, not only the implementation summary.

Resolve critical and high-severity findings before claiming completion. Record lower-severity findings with owner, rationale and a tracked plan.

## 7.4 No test or evidence manipulation

Never:
- delete a failing test to make CI green;
- weaken assertions without a semantic justification;
- hide failures with `continue-on-error`, `|| true` or skipped jobs;
- count warnings as passes;
- hardcode test outcomes in generated reports;
- generate fake conformance witnesses;
- claim object emission is executable success;
- claim cross-compilation is native platform qualification;
- claim a reference model is a production runtime;
- claim passing tests prove the entire language;
- claim a self-hosting compiler is correct merely because it compiles itself.

Use minimized regressions for every real defect.

## 7.5 Evidence identity

Every qualification claim must be bound to:
- exact full source commit;
- exact toolchain and host architecture;
- exact command/workflow and run ID;
- exact job result;
- relevant artifact or log;
- relevant specification digest;
- limitations and excluded scope.

Evidence from an earlier commit is historical evidence, not current-commit proof.

---

# 8. VERSIONING, RELEASE ENGINEERING AND GOVERNANCE

## 8.1 Version discipline

Keep the product version at four components:

`major.minor.patch.revision`

Do not use milestone names, language edition names, crate versions, ABI versions or runtime versions as substitutes for the product version.

Every version change must be deliberate, documented, justified and consistent with the established release rules.

## 8.2 Release qualification

A release-candidate workflow must perform real work and produce real evidence.

At minimum, verify:
- clean source identity;
- pinned toolchain;
- formatting and linting;
- standalone build;
- complete test suite;
- conformance and specification binding;
- semantic linkage and compiler topology;
- security and dependency audit;
- platform qualification appropriate to release scope;
- artifact checksums and SBOM;
- provenance and reproducibility;
- installer install/upgrade/repair/uninstall behavior;
- signing status and trust limitations;
- generated release report based on observed results.

A release report must never be a static template claiming success. It must derive every result from an actual command, workflow output or validated evidence record.

## 8.3 Formal decisions and attestation

Keep three things distinct:
1. A project decision is accepted.
2. A technical qualification run passes.
3. A specific commit is formally attested as qualified for a stated scope.

Alternative C of ADR-0002 was previously approved by the project owner. Preserve that approval and record it accurately.

Do not turn that decision into an unsupported claim that all 26 items are Verified or that the whole language is production-qualified.

Any formal record must use the strengthened schema and be bound to the exact source commit and evidence set. Do not manufacture cryptographic authenticity.

## 8.4 Installer signing

Keep test signing, trusted publisher signing and release signing as distinct states.

Production-trusted signing remains externally blocked until the owner provisions a genuine trusted certificate or approved service. Continue all independent engineering.

---

# 9. DOCUMENTATION AND TRACEABILITY — ALWAYS KEEP IN SYNC

Update the authoritative documents when implementation or decisions change. Inspect actual repository paths; do not assume historical filenames still exist.

Maintain at least:
- project README and public project status;
- canonical master mandate;
- implementation roadmap;
- exact requirements inventory;
- implementation baseline;
- normative specification and digest manifest;
- ADRs and decision register;
- release qualification schema and records;
- CI/workflow documentation;
- changelog;
- known limitations and deferred decisions;
- security and dependency status;
- test/conformance coverage;
- session handoff.

Each requirement should trace:

`Requirement ID → Normative rule → Design/ADR → Implementation → Tests → CI evidence → Qualification status`

Do not mark a requirement Verified merely because:
- its code compiles;
- a test exists;
- a job succeeded on another commit;
- a related implementation was completed;
- a report says it is complete.

Use explicit statuses such as:
- Proposed;
- In Progress;
- Implemented;
- Partially Verified;
- Verified;
- Accepted (decision only);
- Blocked (external dependency);
- Deferred;
- Superseded;
- Not Applicable (with approved rationale).

Do not collapse Accepted into Verified.

---

# 10. ENGINEERING ORGANIZATION AND PARALLELISM

Use multiple focused workstreams when available, but maintain one integration authority.

Suggested specialist responsibilities:
- specification and language-rule auditor;
- front-end and parser specialist;
- type-system specialist;
- ownership/provenance specialist;
- effects/capability specialist;
- HIR/MIR and verifier specialist;
- reference-machine specialist;
- Cranelift/backend/ABI specialist;
- runtime/stdlib/FFI specialist;
- Windows platform and installer specialist;
- Linux/macOS compatibility CI specialist;
- developer tooling/LSP specialist;
- security/supply-chain specialist;
- conformance/fuzz/performance specialist;
- documentation and release-evidence specialist;
- independent adversarial verifier.

Parallelize only work with clear file ownership and non-conflicting dependencies. Do not have multiple agents rewrite the same central compiler component or normative specification simultaneously.

The integrator must:
- reconcile assumptions;
- review all diffs;
- run combined tests;
- reject incompatible interfaces;
- resolve ownership conflicts;
- preserve semantic consistency;
- record the exact integrated commit.

Do not treat an agent's completion report as evidence until the integrator independently verifies it.

---

# 11. SESSION CONTINUITY — NEVER RESET THE PROJECT PLAN

At the beginning of each session, resume from the latest durable handoff and verify its commit identity.

At the end of each meaningful work cycle, update a durable handoff containing:
- repository path and branch;
- exact HEAD and remote state;
- clean/dirty status and preserved user changes;
- exact completed changes and local commits;
- test commands, results and evidence references;
- CI run IDs and job conclusions;
- verified requirement changes;
- known defects and limitations;
- external blockers;
- accepted, deferred and unresolved decisions;
- files and areas not to redo;
- the next ordered implementation queue.

Do not reset the roadmap after each session. Do not re-audit all completed functionality without a concrete reason. Maintain a completion ledger and a next-task queue.

When a task is blocked, work on the next independent item. When a milestone is complete, automatically begin the next milestone. When the available session ends, leave the repository in a safe state and provide a precise handoff.

---

# 12. DEFINITION OF SUCCESS

The project is successful only through evidence-backed progression across all three stages.

## Stage 1 success

A functioning Rust-based Omni compiler and core toolchain that:
- implements the approved Edition 1 language subset;
- preserves specification authority and source provenance;
- performs sound type, ownership, effect and capability checks for the supported subset;
- has an independently tested MIR verifier;
- has a deterministic reference machine;
- emits, links and executes supported native programs on the official Windows platform;
- has meaningful conformance, adversarial, regression and differential testing;
- has reproducible builds, diagnostics and usable developer tools;
- has documented scope and qualification evidence.

## Stage 2 success

A complete production-capable Rust platform with:
- mature compiler and runtime;
- standard library;
- stable ABI and FFI;
- package/build system;
- production developer tooling;
- supported Windows distribution;
- secure release/update and installer practices;
- documented compatibility and migration policy;
- reproducible, audited and qualified releases.

## Stage 3 success

A self-hosted Omni implementation that:
- is written in Omni;
- has a trustworthy bootstrap chain;
- reproduces independently;
- agrees with the Rust reference implementation on the qualified corpus;
- passes language conformance, safety, security and release qualification;
- can be maintained without relying on Rust as the permanent implementation language, while retaining the Rust implementation as an independent reference where useful.

Never collapse these milestones into one claim.

---

# 13. REQUIRED FIRST RESPONSE AND IMMEDIATE ACTION

Begin by reporting the actual local/remote state and verifying run `44746310922` against the live GitHub record.

Then immediately execute this queue:

1. Confirm COMP-020's two successful native compatibility jobs and update its status only if the authoritative criteria are fully satisfied.
2. Fix the Windows `assert-toolchain.ps1` defect and prove the complete native Windows CI path.
3. Fix `E0425` in `research/memory-model/domain_lifetimes.rs:471`; do not suppress the research test.
4. Reorganize the old `validate`, platform-experiment and stabilization workflows without losing unique validations.
5. Replace the release-candidate inventory false-positive logic and static report with strict, evidence-derived qualification.
6. Reconcile the 26-item inventory using its real authoritative source. Do not assume `IMPLEMENTATION_INVENTORY.md` exists.
7. Reconcile ADR-0002 with the already-approved Alternative C decision and strengthen schema/validator/tests.
8. Keep INST-002 blocked until a legitimate production-trusted certificate exists; continue every independent task.
9. Run full current-commit Windows, foundation, research, security, conformance, release and experimental platform validation.
10. Update all records and evidence.
11. **Do not stop here.** Continue into the highest-priority incomplete Stage 1 semantic/compiler work, then Stage 2 production readiness, and finally Stage 3 self-hosting, in dependency order.

Do not answer with another long plan and no implementation. Research and implementation must proceed together. Do not ask the owner whether to continue after an individual task or milestone. Stop only at a genuine owner decision, external authorization/provisioning boundary, safety/legal boundary, or the end of the available execution session. When blocked, document and continue elsewhere.

**The governing instruction is continuous, evidence-driven implementation across the complete Omni roadmap—not repeated one-task handoffs.**
