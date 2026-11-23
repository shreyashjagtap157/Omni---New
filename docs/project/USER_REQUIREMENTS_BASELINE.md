# Omni — reconstructed user requirements and product baseline

Classification: Informative

**Canonical implementation repository:** `https://github.com/shreyashjagtap157/Omni---New`  
**Canonical branch:** `main` only  
**Primary production compiler implementation:** Rust  
**Long-term product:** Omni language, compiler, runtime, standard library, package/build platform, developer toolchain and independently qualified self-hosted compiler  
**Requirements source:** prior project discussions, the current owner master charter, the 20 November engineering mandate, the 21 November local continuation mandate, repository rules/specification, and the October conversation-integrated compendium.

> This is a reconstructed project-management baseline, not a new language specification. Its `OMNI-UR-*` keys are traceability labels only; they are not normative language-rule IDs and do not replace IDs in `spec/registry/rules.json`, the Edition 1 manifest, ADRs or release gates. Where the user's product ambition and the normative language specification differ in maturity, the ambition remains the research/product objective while only the in-force specification governs Edition 1 implementation behavior.

## 1. Mission and enduring product intent

### OMNI-UR-MISSION — General-purpose computational language and platform

Omni is intended to become an exceptionally capable, general-purpose, multi-paradigm, multi-level, multi-runtime programming language and its supporting development platform. It is not merely a Rust compiler, a conventional systems language, a parser demonstration, an interpreter, or a single-purpose DSL.

The language should bring together advanced programming-language theory, type systems, memory management, compiler architecture, execution models, concurrency, security, formal reasoning, performance engineering, hardware abstraction and developer experience in one coherent system. It should serve high-level application programming as well as low-level operating-system, compiler, embedded, bare-metal, real-time, hardware-adjacent and platform work, subject to formally governed profiles and real implementation/qualification evidence.

The design aspiration includes aerospace/space-grade, defense/military-grade, enterprise-grade and other high-assurance applications. These are quality targets and use-case aspirations, not a claim that the language is certified or that a test suite constitutes certification.

### OMNI-UR-EVOLUTION — No artificial final ceiling

There is no predetermined final limit on research, capability, optimization, target support or architectural enhancement. Stage 1, Stage 2 and Stage 3 are development milestones, not an endpoint. The project must remain able to adopt useful future algorithms, type theories, architectures, runtimes, hardware targets, assurance methods and developer workflows through controlled, evidence-based evolution.

A stable release may be frozen and immutable; the project itself must continue to evolve. Do not require every future runtime, accelerator, hardware architecture or domain integration before Omni can become a useful language. Conversely, do not declare the long-term project complete merely because a first production release or self-hosting milestone is reached.

### OMNI-UR-COHERENCE — “Everything, but structured”

Omni should maximize expressive power without accumulating unstructured features. Every adopted feature needs:
- a precise semantic model and grammar;
- a sound and feasible implementation strategy;
- interaction rules with existing features;
- restrictions and profile behavior where needed;
- diagnostics and developer-facing behavior;
- conformance, regression and assurance evidence;
- migration and compatibility treatment;
- an explicit lifecycle state and owner-approved authority.

## 2. Programmer experience and language surface

### OMNI-UR-VIBE — Staged syntax strategy: conventional first, optional Vibe-first evolution later

The 20 November 2026 long-horizon mandate explicitly resolves the syntax sequence:

1. **Initial production language (Stage 1): conventional, explicit and developer-familiar.** Complete the conventional language surface first. It should be modern, professional, concise, readable, consistent, learnable and pleasant to write and review. Study established language conventions and evidence about familiarity, comprehension, ambiguity, expressiveness, tooling and error prevention, but do not clone another language. Do not make the initial implementation conversational, sentence-like, arbitrary-natural-language-based or dependent on an AI prompt.
2. **Later optional syntax evolution:** only after the first-stage language and its agreed ecosystem pass their required qualification gates, research and design an optional, distinctly Vibe-first surface syntax. This later surface may express programmer intent with lower ceremony and more natural forms, but it is a separate governed language evolution, not a reason to delay the conventional production compiler.
3. **One semantic core:** both surfaces, if the later one is approved, must lower into the same established semantic core and preserve type, ownership, lifetime, effect, capability, safety, diagnostics and verification guarantees. Preserve the conventional surface as a supported stable surface unless a separately approved compatibility/migration decision says otherwise.

The long-term Vibe-first ambition is preserved, but it does **not** redefine the initial production syntax or authorize Candidate 2/3. A desirable user experience does not ratify a specific grammar.

For both current and future syntax, parsing, name binding, typing, lowering, diagnostics and compile-time behavior must remain deterministic. Do not use probabilistic NLP, fuzzy interpretation, LLM-dependent semantics, hidden conversational context or guesswork. Every accepted form needs formal grammar, unambiguous binding, exact desugaring and conformance tests. Ambiguity must be rejected.

Reduce avoidable ceremony when it is justified and unambiguous; do not remove explicit syntax where ownership, authority, safety, resource use, ABI or semantic clarity requires it.

### OMNI-UR-LEVELS — Three compatible programming levels

1. **High-level / intent-oriented:** expressive abstractions, concise transformations, rapid application development and minimal ceremony.
2. **Intermediate / explicitly controlled:** direct control of types, effects, ownership, capabilities, constraints, concurrency and execution.
3. **Low-level / systems:** explicit representation, memory, regions, allocation, ABI, FFI, synchronization, hardware and performance control.

These are coherent levels of one language, not unrelated sublanguages with incompatible semantics.

### OMNI-UR-DETERMINISM — Safe and predictable inference

Inference, overload resolution, parsing, lowering and compile-time computation must be deterministic. Never guess when multiple materially different interpretations remain. Do not infer privileged capability, ownership transfer, unsafe conversion, ambiguous overload, hidden I/O or unbounded resource use. Natural-looking syntax must still be defined by formal grammar and statically analyzable semantics. No heuristic NLP/fuzzy parsing may decide what a program means.

Syntax candidates must be measured for readability, learnability, expressiveness, ambiguity, parser complexity, error rates, refactoring, formatting, diagnostics and actual developer productivity. A desirable UX does not ratify a syntax proposal.

## 3. Specification authority, language governance and compatibility

### OMNI-UR-SPEC — One explicit normative authority

The normative language definition is governed by the repository's actual `spec/` hierarchy, manifest, registry, ADRs and release gates. Consult the exact active files and their current digests. No implementation convenience, prompt, example, unit test, code comment, generated doc, research note or historical candidate may silently override normative authority.

When the specification is silent, ambiguous or contradictory:
- identify the exact grammar production, rule IDs, source locations and constructs;
- record the discrepancy and the competing interpretations;
- research relevant theory, standards and mature implementations;
- block unsupported conformance claims and fail closed where required;
- obtain an explicit governed decision before changing semantics;
- update the normative source, manifest/digests, tests, implementation and evidence together.

Never let multiple semantic authorities emerge between the specification, parser, compiler, interpreter, self-hosted implementation or test corpus.

### OMNI-UR-EDITION1 — Edition 1 invariant

The active Edition 1 grammar and language contract remain the baseline. The current in-force manifest and normative domain records determine precisely which rules are binding. Keep the accepted baseline immutable except through the formal amendment process. Do not infer ratification from a file name containing “final”, “complete”, “vibe” or “candidate”.

### OMNI-UR-CANDIDATES — Candidate boundaries are strict

- Candidate 1 is the active baseline only to the extent confirmed by the manifest and release governance.
- Candidate 2 (“vibe surface syntax”) is a pre-release/unratified proposal with pending signature/status metadata and no automatic Edition 1 implementation impact. It is not in force merely because its artifacts or appended productions exist.
- Candidate 3 syntax/parser experiments are research probes, not normative semantics.
- Pipeline operator `|>`, historical projection shorthand and other inactive legacy surface constructs must not be reintroduced as normative Edition 1 syntax without explicit ratification and a governed version/feature boundary.
- Parser acceptance of a test fixture does not establish semantic adoption.

### OMNI-UR-MEMORY — Preserve the open memory-model question

The recorded historical product vision includes managed-memory-by-default ambitions, while the active Edition 1 baseline uses affine ownership/borrow checking for the declared domain. Do not silently erase this tension or unilaterally choose a global memory model based on ergonomics or precedent.

Research hybrid/profile-aware designs—including affine ownership, deterministic destruction, managed facilities, region-based allocation, deferred or reference-counted management, capability-bounded heaps and profile-guided allocation—against safety, predictability, performance, resource bounds, interoperability and implementation complexity. The currently normative Edition 1 rules remain controlling until a governed amendment explicitly changes them.

## 4. Core language capabilities to design, specify and implement

The following are product-level capability areas recovered from the user's requirements. Their detailed semantics and mandatory/optional profile status must be checked against the active specification and requirement registry.

### Types, abstraction and compile-time computation

- Strong static typing with sound bidirectional inference and useful diagnostics.
- Generic and parametric programming with explicit constraints and predictable inference.
- Trait/type-class/interface-like abstraction, coherent implementation selection, overlap control and termination guarantees.
- Algebraic data types, product/sum types, records, enums, tuples, pattern matching and exhaustiveness where specified.
- Subtyping/coercion/conversion rules only where explicitly defined; conversions must not silently lose information or authority.
- Higher-kinded/type-level facilities where justified by a sound model.
- Compile-time computation and metaprogramming with resource limits, deterministic evaluation, provenance and clear compile/run boundaries.
- Precise generic monomorphization or alternative code-generation strategies with coherent ABI and diagnostics.
- Reflection/introspection only with explicit compile-time/runtime cost and security semantics.

### Memory, ownership and resource lifecycle

- Precise value/reference/place model, initialization, validity, move/borrow semantics and aliasing rules.
- Affine/linear resource handling where required; explicit treatment of destruction, partial initialization, drops, pinning, lifetime/region constraints and reborrows.
- Safe references, provenance tracking and access restrictions; no unsound aliasing or lifetime escape.
- Deterministic allocation/deallocation interfaces, allocator behavior and memory-resource accounting.
- Safe high-level memory facilities and any managed or tracing facility only through a formally governed design that preserves profile guarantees.
- Defined behavior for bounds, initialization, alignment, allocation failure, invalid access and resource exhaustion.
- Explicit unsafe/low-level boundaries if the normative language adopts them; do not use an implementation escape hatch to bypass the user's safety goals or repository's current Rust safety lint.

### Effects, capabilities, errors and security

- Algebraic or typed effect modeling for externally observable actions.
- Fine-grained capabilities for authority, access, resource and effects; no implicit privilege escalation.
- Explicit authority derivation, attenuation, transfer, revocation and validation as defined by normative rules.
- Deterministic and composable error/failure handling, cleanup and cancellation semantics.
- Structured resource management, quotas/limits and predictable failure handling.
- Capability requirements must be visible at appropriate boundaries; convenience must not obscure privileged behavior.

### Concurrency, parallelism and distributed execution

- Safe concurrency abstractions, ownership-aware data sharing, synchronization and task lifecycles.
- Predictable async/task execution and cancellation/failure semantics where adopted.
- Parallel execution and scheduling support without hidden data races or undefined memory interactions.
- Resource-aware concurrency and real-time execution profiles where their boundedness can be specified and tested.
- Distributed execution, communication and remote resource models as governed capabilities, not untrusted implicit behavior.
- Explicit memory ordering and atomics semantics tied to hardware mappings and the memory model.

### Modules, packages, numerics and scientific computing

- Coherent modules, visibility, imports, packages, versioning and dependency resolution.
- Build graph determinism, dependency provenance and conflict handling.
- Numeric types and representation, overflow/rounding, exactness, floating-point behavior and conversions.
- Scientific/numerical facilities and algorithms with accuracy and reproducibility requirements.
- Vectorization/SIMD and accelerator support only where semantics and portability are clear.
- Broad domain coverage without implying every domain is supported by every target or profile.

## 5. Compiler architecture and execution pipeline

### OMNI-UR-PIPELINE — One coherent verified compilation pipeline

Build the language as an integrated system, not a collection of disconnected crates or parser demonstrations. The pipeline should have explicit contracts and traceable data flow from source to execution:

- source input and lossless lexical/source-location representation;
- lexer/token stream and diagnostics;
- parser and governed grammar contract;
- AST/CST as needed for source fidelity;
- name/module/import resolution;
- type checking, generic/trait resolution and semantic obligations;
- ownership, lifetime, effects, capabilities and resource analyses;
- canonical HIR with semantic metadata/provenance;
- typed, place-based MIR/CFG and explicit control/resource-flow operations;
- independent fail-closed MIR/IR verifier;
- reference abstract machine/interpreter for deterministic semantic execution;
- lower-level representation and target-independent optimization boundaries;
- native backend(s), beginning with validated Rust/Cranelift pathways where appropriate;
- correct calling conventions, function pointers/indirect calls, data layout, relocation, object emission, linking, ABI and target triples;
- runtime/standard-library linkage, FFI and platform adapters;
- execution, debugging, diagnostics and reproducible artifacts.

Each stage must preserve all semantic obligations required by later stages. Source declarations, ownership/effect/capability metadata, error paths, provenance, cleanup and ABI facts must not be lost during lowering. Verifiers must reject invalid IR and avoid rejecting valid programs merely because an analysis is overly conservative; conservative failure boundaries must be explicit.

### OMNI-UR-REFERENCE — Reference semantics and independent execution route

Maintain a reference machine or executable semantic oracle to define and test behavior independently of the native backend. Differentially compare the reference path, native outputs and experimental self-hosted path. Do not confuse an interpreter demo with the full implementation and do not let a backend define language semantics.

### OMNI-UR-BACKEND — Real native output

Object generation must yield valid target objects with correct symbols, relocations, calling conventions, data layout, stack/alignment, return values, indirect calls, external symbols and ABI behavior. Validate both object-level properties and actual execution. Host linkers may impose target constraints; isolate and diagnose those correctly rather than hiding compiler defects behind environmental workarounds. Cross-target support requires exact toolchain and runner evidence.

## 6. Runtime, standard library and platform

The complete language platform includes:
- runtime initialization, allocation, memory, error handling, effects/capabilities, task/concurrency services and platform abstraction;
- core standard-library types, collections, text, numerics, iteration, I/O, filesystem/network adapters and other domain packages as specified;
- stable and documented ABI/FFI boundaries, headers and external-language interoperability;
- native executable generation, linking, runtime loading, package loading and execution;
- hosted, desktop, embedded, bare-metal, real-time and other governed execution profiles with explicit restrictions/resources/assumptions;
- cross-platform host/toolchain support, especially the current Windows official development/release target, plus Linux/macOS compatibility where declared;
- repeatable install, upgrade, repair, uninstall and distribution workflows for the toolchain;
- package manager, registry/repository interaction, dependency solving, build orchestration, artifact caching and integrity verification.

An empty crate or placeholder CLI that compiles is not a package manager, build system, runtime or standard-library implementation. Verify the actual public behavior and acceptance path for every claimed capability.

## 7. Developer toolchain and user-facing product

The product should offer an integrated developer experience:
- stable compiler driver and command-line interface;
- build/package/test workflows;
- formatter and linter;
- specification-linked diagnostics and source spans;
- debugger and inspection/observability facilities;
- language-server/IDE foundation and editor integration;
- shell completion and predictable configuration;
- documentation generation, examples and tutorials;
- test runner, conformance corpus, regression management and fuzzing;
- installation and upgrade/uninstall tooling;
- clear error messages, explainable compiler decisions, actionable diagnostics and reproducible bug reports.

Tooling quality is part of the language product, not an afterthought. Advanced users must retain precise low-level control while ordinary use avoids unnecessary ceremony.

## 8. HELIOS and broader domain use

Omni is intended to be capable of serving as a foundation language for complex platform/system projects such as HELIOS. Preserve a potential HELIOS integration path for system software, deterministic symbolic engines, security-sensitive code and low-level resource control. Do not couple the compiler to HELIOS-specific semantics, APIs or roadmap in a way that narrows Omni's general-purpose mission.

Support a broad set of domains through profiles, target runtimes and libraries: systems/OS development, embedded and bare-metal, desktop/application software, high-performance and scientific computing, safety-sensitive and high-assurance development, distributed systems, hardware/accelerator work and future domains. Claims must be bounded by actual conformance and target evidence.

## 9. Rust-first development and self-hosting transition (Approach C)

### OMNI-UR-APPROACH-C — Rust-first production path

Approach C is the approved primary strategy:
- **Track A:** develop the production compiler and toolchain in Rust first. Rust is the bootstrap and reference implementation strategy, not an irreversible restriction on the eventual platform.
- **Track B:** continue self-hosting experiments in parallel, isolated from normative authority. They may test whether the language is expressive enough to implement itself and may reveal design weaknesses.
- **Track C:** maintain language-theory, formalization, architecture, security and assurance research.

A practical, genuinely useful Rust implementation must reach the complete declared mandatory Edition 1 scope, executable native behavior, runtime/stdlib, package/build/toolchain and qualification requirements before an official migration or replacement is considered. Do not delay a useful Rust production release because every aspirational future capability has not been researched.

### OMNI-UR-BOOTSTRAP — Trustworthy, controlled self-hosting

Official self-hosting requires more than a compiler written in Omni compiling a trivial program. Require:
- sufficient language/runtime/stdlib capabilities to implement real compiler components;
- a documented bootstrap chain and trusted computing base;
- differential tests against the Rust implementation and reference machine;
- compiler self-compilation and reproducibility evidence;
- independent rebuild/verification and a defined bootstrap trust process;
- clear migration, rollback and ownership of the reference implementation;
- no second semantic authority and no reduction in conformance coverage.

Self-hosting is not proof of correctness, safety, production readiness or independence by itself. Maintain Rust/reference implementation as needed until the new toolchain is independently qualified.

### Recorded stage progression (not release claims)

- **Stage 1:** Rust minimal working core, leading toward project milestone `1.0.0.0`.
- **Stage 2:** production-capable Rust language/tooling platform, leading toward `2.0.0.0`.
- **Stage 3:** Omni-written compiler with independently qualified bootstrap trust, leading toward `3.0.0.0`.
- **Beyond:** continual evolution; no final completion ceiling.

Milestone boundaries and current gate definitions remain those in the repository. Do not infer they have been passed from version strings.

## 10. Research, experimentation and technology watch

Research is continuous and must inform both future design and re-evaluation of existing components. Cover:
- programming-language proposals and type-system theory;
- parser/front-end and compiler/IR architectures;
- ownership, borrow checking, memory management, regions and resource lifetimes;
- effect systems and capability security;
- concurrency, parallel/distributed models and real-time systems;
- formal semantics, theorem proving, model checking and verification methods;
- compiler optimization, incremental compilation and code-generation backends;
- runtime, libraries, package/build architecture and debugging;
- numerical/scientific methods and deterministic computation;
- processor, accelerator, GPU/NPU and evolving hardware;
- operating systems, firmware, bare-metal and hosted targets;
- cryptography, security vulnerabilities and software supply-chain defenses;
- developer ergonomics, syntax, diagnostics and human factors.

For every research item, distinguish publication/theory, formal result, available implementation, reproducibility, maturity, limitations, independent review and measured evidence. Compare alternatives, prototype to falsify, measure, record negative results, assess compatibility and adopt only through a controlled lifecycle. Reassess existing mechanisms periodically; “it works” does not prove “it is optimal”.

Research and engineering loop: research → hypotheses → alternatives → architecture → formalization/specification → executable prototype → falsification/measurement → decision/ADR → implementation → tests → independent review → qualification → documentation. A failed experiment is useful evidence and must not be concealed.

## 11. Verification, testing, assurance and release quality

### OMNI-UR-EVIDENCE — Evidence classes remain distinct

Keep separate and accurately report at least:
- Proposed / Hypothesis;
- Implemented but unverified;
- Experimentally validated;
- Verified by reproducible tests or execution;
- Proven within explicit machine-checked assumptions;
- Accepted as a decision only;
- Partially verified;
- Blocked with an exact external reason;
- Rejected / Superseded / Not applicable with rationale;
- Qualified for an exact declared scope/target/revision.

A test passing is not a proof. A model proof is not proof the model captures all user intent. Compiler correctness is not system certification. A commit, green job, parser test or successful self-hosted build is not blanket product qualification.

### OMNI-UR-TEST — Required verification depth

Use evidence-appropriate layers:
- unit and boundary tests for each subsystem;
- integration and end-to-end source-to-executable tests;
- executable conformance corpus tied to normative rule IDs;
- negative tests and ambiguity/rejection tests;
- property-based/model-based testing;
- fuzzing and sanitizer runs on appropriate toolchains;
- mutation testing and test-oracle quality checks;
- feature-interaction and cross-representation tests;
- differential execution across reference machine and native backend;
- stress, resource-exhaustion, failure-injection and recovery tests;
- compiler bootstrap/self-compilation tests when relevant;
- performance benchmarks with baselines and repeatable methodology;
- multi-platform CI for declared Windows/Linux/macOS scope;
- security/dependency review and independent adversarial review;
- reproducible builds, artifact integrity and provenance.

Do not weaken a test, remove a failing edge case, skip a target or hide a defect to obtain green CI. Fix root causes and preserve negative evidence.

### OMNI-UR-QUALIFICATION — Qualification is scoped and auditable

Every claim must point to the exact source revision, specification/manifest identity, test suite, environment, logs and reviewer. Qualification must identify target/profile/features included and excluded. Preserve blockers such as external production-trusted signing credentials honestly. Do not claim formal certification without the actual independent process and authority.

## 12. Security, dependencies and supply chain

- Treat security as a language/compiler/runtime concern, not just dependency scanning.
- Audit both direct and transitive dependencies, reachable code paths, build scripts, proc macros, target-specific crates and features.
- Track advisory ID, affected version, exploitability, reachability, mitigation, fixed version, verification and residual risk.
- Do not ignore an advisory because the code path seems unused; justify scope with evidence.
- Pin/lock reproducible dependencies and upgrade deliberately after compatibility tests.
- Keep compiler/runtime boundaries explicit; avoid unsafe Rust and preserve workspace-level safety/clippy policies currently enforced by the repo.
- Protect release keys, signing certificates, credentials and tokens. Never store private keys, passwords, recovery codes or tokens in Git, logs or artifacts.
- Distinguish ephemeral test signing from production-trusted signing. Production trust cannot be fabricated or self-attested without the actual authority.
- Produce checksums, SBOM, build provenance and reproducible artifacts for declared releases.
- Third-party licenses and normative-standard access must be lawful and tracked.

## 13. Versioning, compatibility and release governance

### OMNI-UR-VERSION — Four-component project release version

The current 20 November engineering mandate spells the product format as `major.minor.patch.revision` (also written `X.Y.Z.W`). This product release identity is separate from:
- language edition and specification candidate/version;
- engineering milestone and foundation-gate identifiers such as `OMNI-IMP-0.0.0.13`;
- Rust crate/package versions;
- compiler build/toolchain identity;
- runtime and standard-library version;
- ABI and FFI version;
- installer/package/artifact format;
- documentation revision and research-record revision.

The master charter records that the relationship between this product-release format and the older `R.M.m.P` project-generation/milestone scheme requires a dedicated versioning ADR. Preserve historical `OMNI-IMP-0.0.0.13` and other engineering references until that governed migration is resolved; do not silently reinterpret them as product releases. Technical gate passage is not formal owner sign-off, a product tag or a published release. Do not silently reuse/overwrite a released version. Released artifacts and their evidence are immutable; breaking changes require explicit specification/version/migration handling.

### OMNI-UR-RELEASE — No release by label alone

Use explicit release channels and gates. Stable/product releases require declared scope; approved specification identity; required features; end-to-end native behavior; runtime/stdlib and tooling; conformance/regression/security/performance/reproducibility; supported-target evidence; documentation and known limitations; version/changelog validation; artifact provenance; and required independent review.

Maintain a canonical changelog with an Unreleased section and categorized entries; validate it. Automate version checks, manifests, tags and release provenance without allowing automation to bypass qualification.

## 14. Requirements traceability and project records

Maintain a durable, exact-commit record set:
- project README and public status;
- master charter and current long-horizon mandate;
- requirement registry and acceptance criteria;
- implementation plan, inventory and exact baseline;
- normative spec and hash/digest manifest;
- grammar and rule registry;
- ADRs and decision/discrepancy registers;
- risk and blocker register;
- test/conformance and coverage maps;
- security/dependency status;
- release/qualification schema and records;
- changelog and known limitations;
- CI/workflow documentation and exact run IDs;
- session handoff and next-task queue.

Every requirement should trace:

`User intent → requirement key → normative rule/ADR → design contract → implementation path → test/conformance case → CI evidence → qualification status`

Do not declare a requirement verified because code compiles or a test exists. Do not let a new session reset or lose the previously established roadmap.

## 15. Repository, branch and agent workflow

### OMNI-UR-GIT — Preserve the canonical main-only workflow

- Canonical implementation repository is `shreyashjagtap157/Omni---New`; do not confuse it with the historical `Omni` repository.
- Work on `main` only. Do not create feature branches.
- Sync/inspect the actual remote baseline and local workspace before editing.
- Preserve all staged, unstaged, untracked, ignored and local-only user work; do not hard-reset, clean, overwrite, silently stash or replace a workspace.
- Never force-push or rewrite existing Git history.
- Integrate through ordinary commits and fast-forward updates. Check expected branch HEAD before publishing; reconcile a concurrent change rather than overriding it.
- Keep Cargo.lock deliberate and use locked verification. Do not mutate lockfiles implicitly.
- Keep implementation changes, spec changes and release records reviewable and commit-scoped.
- The user explicitly authorized pushing this documentation-reconciliation change to the canonical remote; that authorization does not extend to unrelated release publication, tags, signing or destructive operations.

### OMNI-UR-AGENTS — Parallel engineering with one integration authority

Use specialist workstreams when supported—spec/grammar, parser, type system, ownership, effects/capabilities, HIR/MIR/verifier, reference machine, backend/ABI, runtime/stdlib, package/build, Windows/platform, dev tooling, security, conformance/performance, documentation/release and adversarial verification.

Parallelize only work with clear ownership and non-conflicting interfaces. One integrator must reconcile assumptions, review the combined diff, run tests, resolve interface/semantic conflicts and record the exact integrated commit. Agent completion reports are leads, not evidence, until independently verified. Do not pretend that agents or specialist swarms were run if they were not.

### OMNI-UR-HANDOFF — Durable session continuity

At every meaningful milestone, preserve repository path, branch/HEAD, remote state, staged/unstaged/untracked user work, exact completed changes, commits, test commands/results, workflow IDs, requirement-status deltas, known defects, blockers, accepted/superseded/open decisions and a prioritized next-task queue. Continue dependency-ready engineering; do not repeatedly ask the owner to say “continue”.

## 16. Explicit boundaries and unresolved historical sources

1. An older charter says the original design interview involved more than 160 decisions and instructs recovery of the original questionnaire and answers. The complete raw questionnaire/transcript was **not recovered** from the available project files/conversation materials searched for this reconstruction. Do not invent its questions or answers. The current baseline records the recovered durable requirements; the original interview remains an explicit provenance-recovery task.
2. The October 2026 “Omni Complete Specification — Max” is preserved at [the archive](archive/Omni_Conversation_Integrated_Design_Compendium_2026-10-13_NON_NORMATIVE.md). It is a generated conversation-integrated reference, not the governing normative specification.
3. The separate `Omni-Research` workspace and its `Omni-MASTER-SPEC.md` are research artifacts with their own stage/evidence boundary. They are not imported into the implementation repo as Edition 1 semantics. Research results must pass the controlled governance path first.
4. The two master-vision versions and long-horizon prompts already at repository root are retained to avoid breaking historical file links. [Project index](README.md) organizes them; it does not create new independent authority.
5. Historical state claims inside old prompts, handoffs and compendia are not current facts. Verify them against current source, manifest, tests, live CI and exact commit.

## 17. Completion principle

Omni becomes a genuinely usable and independently validated language platform only when the declared release scope is implemented end-to-end with the required semantic, runtime, toolchain, conformance, security, reproducibility and qualification evidence. No single parser, compiler demo, successful unit suite, passing workflow, self-hosting milestone or product version ends the project. Continue to evolve the language and ecosystem responsibly after every milestone.
