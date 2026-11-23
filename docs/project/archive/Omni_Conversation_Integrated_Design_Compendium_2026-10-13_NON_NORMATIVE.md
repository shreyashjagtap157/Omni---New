Classification: Informative

> HISTORICAL, GENERATED AND NON-NORMATIVE REFERENCE — retained to preserve the 13 October 2026 conversation-integrated synthesis and design history. It is not a language specification, does not amend Edition 1, and cannot activate Candidate 2/3 or any unratified semantic proposal. The current in-force normative artifacts under `spec/`, bound by the Edition 1 manifest and governed by the repository's ADR/release process, remain authoritative.

---

# Omni Complete Specification — Maximally Granular Edition

**Status**: Generated reference with exhaustive granularity — every point decomposed into its minor sub-points, and each of those into minor-minor-sub-points, continuing until the most atomic elements of the specification.

**Version**: 1.0.0.0 (generated)  
**Granularity Level**: Maximum — all claims, rules, and distinctions decomposed to their most atomic elements

---
## ## Overview

Omni is an idealized programming language designed for state-of-the-art, space-grade, enterprise-grade, and military-grade software development. This specification defines the language's syntax, semantics, type system, verification model, build pipeline, conformance framework, and assurance profiles.

**Resolved Tension**: Expressive power vs. verifiability through staged verification, content-addressed computation, capability-based security, and profile-enforced feature restrictions.

**Philosophical Foundations**:
- Formal verification and reproducible builds are architectural concerns, not afterthoughts
- Every language feature must have clear normative semantics
- A sound implementation strategy must exist for every feature
- Enforceable restrictions must accompany every feature
- Evidence appropriate to the assurance profile must accompany every feature

**Five-Fold Distinction** (preserved throughout):
1. Design requirement — the intention behind a feature
2. Specification clause — the normative statement of the feature
3. Implementation — the actual code realizing the feature
4. Passing test suite — tests demonstrating the feature works
5. Proof of correctness — formal verification the feature is correct

**Implementation Status vs. Normative Clauses**: This specification defines the normative *clauses*; implementation status must be verified separately against the actual Omni repository code and tests.

---
## ## 1. Language Governance

### ## 1.1 Versioning Scheme

#### ## 1.1.1 Four-Component Form
- **Structure**: `R.M.m.P` where:
  - R = Release (major generational shift)
  - M = Major (feature-complete milestone)
  - m = Minor (incremental addition)
  - P = Patch (bugfix, non-feature change)
- **Semantic meaning**: Each component has independent semantic meaning; changing any component implies different rights, obligations, and compatibility characteristics
- **Immutability**: Released versions are immutable; version bumps correspond to specific qualification states
- **Trailing zeros**: `1.0.0.0` is the minimal verified core; `0.0.0.1` through `0.0.0.13` are foundation gates; `1.0.0.0` marks Stage-1 foundation completion
- **Governance boundary**: `0.x.x.x` versions are foundation/qualification artifacts; `1.x.x.x` and above are release artifacts with conformance claims

#### ## 1.1.2 Example Values and Meanings
- **`0.0.0.1`**: Initial workspace topology and global safety/lint contract; foundation gate start
- **`0.0.0.2`**: Rust pin `1.95.0`; workspace uses Cargo resolver 2; qualification remains fail-closed until host provides `rustc`, `cargo`, `rustup`, and required targets
- **`0.0.0.3`**: Publication set begins; `grammar/omni-edition1.ebnf`, `grammar/candidate2-erratum.md`, `registry/rules.json`, `registry/rule-texts.json`, all eight `schemas/*.schema.json`, plus anything under `models/` or `data/` (currently only `.gitkeep` scaffolding, excluded)
- **`0.0.0.4`**: Canonicalization engine (`omni-canon`) domain established; JSON documents: recursive lexicographic key order (UTF-8 byte order), arrays keep document order; exact UTF-8; LF normalization inside string values (RULES section 4); integral finite floats fold to integers (MANIFEST-0002; fractional floats kept); insignificant whitespace never participates; duplicate object keys rejected fail-closed; malformed input rejected; paths are structural, never substring substitution; absolute-shaped strings outside the root are rejected as unresolved host identity (REPRO-0004, BUILD-0005); all other strings pass through byte-identical, including backslash literals and relative fragments; one digest representation: lowercase hex SHA-256
- **`0.0.0.5`**: Publication set unchanged; tree digest still `ecda85bb1f460266abf2b05f0a9f8058c9c6162f8c448ef1f46989ad3577698b`; NFC is a producer obligation (no normalization dependency in the workspace)
- **`0.0.0.6`**: Linkage reaper (`omni-audit`) domain; single ownership model: `#[implements("ID")]` and `implements!("ID")` claims under `compiler/` and `tools/` (generated `target/` excluded, filename-sorted walk) must resolve to registry rules in `Candidate` or `Ratified` state; `Proposed`/`Deprecated`/`Superseded`/`Withdrawn` ownership, unknown IDs, and malformed claim syntax all fail closed; witness entries must each resolve to a contained workspace file or a collected `#[test]` name; every `Ratified` rule must be both claimed and witnessed; coverage is per-rule, not per-file; duplicate claims across files are allowed and attributed
- **`0.0.0.7`**: Provenance generation (`omni-evidence` + driver emission); single format authority remains the 0.0.0.8 provenance schema, extended only with optional `compiler_id`, `compiler_version`, `source_revision` (`-dirty` convention), and `codegen` fields; old records still validate; generation is state-separated (`ProvenanceInputs` → validated → canonical bytes → record) inside `omni-evidence`, reusing the canon engine; mismatched identity combinations cannot be constructed; topology refined to match the 0.0.0.9 specification's own infrastructure category: spec infrastructure (`canon`/`registry`/`evidence`) is leaf-ward and usable from any tier as counted infra edges but may never depend outward; `compiler → stage0` is allowed; true tooling stays isolated both ways; obsolete driver-only provenance struct is deleted; the driver builds records from actual inputs (loaded spec identity, pinned toolchain, host descriptor, crate version, git revision or graceful absence, declared epoch, emitted bytes) and writes a canonical sidecar on native emission, failing closed when `./spec` is unresolvable; target descriptors are truthful host descriptions, not canonical triples (target registry is future work); full build provenance generation beyond per-artifact records stays out of scope until release qualification
- **`0.0.0.8`**: Compiler topology (`omni-topology`); pipeline-order tiers (higher = later stage; production edges point at the same or an earlier tier): source 0, lex 1, syntax 2, parse 3, names 4, types 5, own/effects/traits 6, hir 7, mir 8, verify 9, machine/codegen 10, runtime 11; driver 12 orchestrates and may use any tier; below-MIR consumers declare exact feeds (`machine: {mir}`, `codegen: {mir, verify}`, `verify: {mir}`) so frontend syntax can never be consumed past the lowering boundary; `stage0` is edgeless bootstrap infrastructure; `tools/*` may use tools/externals but never `compiler/*`, and vice versa; dev/build edges are direction-exempt but cycle-checked; diagnostics are deterministic (filename-sorted traversal, sorted edges); dormant downward edges (e.g. hir scaffolding) are tier-exempt but tracked
- **`0.0.0.9`**: (Foundation gate continuation)
- **`0.0.0.10`**: (Foundation gate continuation)
- **`0.0.0.11`**: (Foundation gate continuation)
- **`0.0.0.12`**: (Foundation gate continuation)
- **`0.0.0.13`**: Must not claim verification until the full foundation sequence qualifies and the computed digest matches at release time
- **`1.0.0.0`**: Minimal Rust core implementation; Stage-1 foundation complete; bootstrap language Rust pinned; current version architecture baseline (Candidate 3); recent commit ownership verification improvements to `compiler/omni-verify/src/ownership.rs`; git state: pushed to `origin/main` at commit `c0c9440`
- **`2.0.0.0`**: Complete production Rust platform (future state)
- **`3.0.0.0`**: Certified self-hosted and independently qualified Omni (future state)

#### ## 1.1.3 Governance References
- **BOOT-0002**: The trusted bootstrap path consists of seed decoder/compiler, Stage-0 sources, assembler/object writer, linker/image builder, target emulator/hardware, specification data, and verification tools named in the TCB inventory
- **BOOT-0003**: Stage A builds compiler B from seed. B builds compiler C from canonical Omni sources. C rebuilds compiler D; C and D must converge byte-for-byte for deterministic stages or to declared semantic equivalence when signing/layout is external
- **BOOT-0004**: Diverse double compilation builds the same canonical compiler source with the trusted chain and an independently implemented diverse compiler/toolchain, then uses each result to rebuild and compares canonical outputs or declares semantic equivalence when signing/layout is external
- **BOOT-0005**: A diverse compiler qualifies only if its implementation lineage, language/tool dependencies, backend/link path, and build environment are sufficiently independent and documented
- **BOOT-0006**: DDC mismatch blocks release, preserves all artifacts/logs, and triggers differential localization; it is never waived merely because one binary passes tests
- **BOOT-0007**: The TCB inventory lists exact source/binary digests, role, reason trusted, audit status, replacement strategy, and transitive dependencies for proof checker, parser generator, solver, assembler, linker, Unicode data, crypto, OS/firmware, and signing tools
- **BOOT-0008**: Emergency bootstrap recovery uses archived source, specs, data, emulators, and at least two independent seed paths stored in reproducible offline media
- **BOOT-0009**: Official compiler artifacts include provenance, SBOM, conformance report, optimization-validation report, and DDC evidence bound to the artifact digest
- **BOOT-0010** (implied): Version transition rules govern permissible `R.M.m.P` → `R'.M'.m'.P'` transitions

### ## 1.2 Architecture Freeze

#### ## 1.2.1 Closure Statement
- **Architecture phase is closed**: No silent feature additions, convenience changes, or retroactive semantic modifications
- ** amendment triggers**: Only the following conditions permit a normative amendment:
  - Discovered contradiction between two or more normative artifacts
  - Discovered semantic case not classified by the specification
  - Soundness defect discovered during implementation or verification
  - Interoperability defect that violates an existing contract
  - Implementation impossibility caused by a genuine normative inconsistency
  - Required clarification whose meaning is already uniquely implied but not mechanically stated

#### ## 1.2.2 Non-Triggers for Amendment
- Implementation difficulty alone does not justify weakening a safety guarantee
- Feature popularity or developer preference does not justify semantic change
- Performance optimization alone does not justify behavioral change
- Ecosystem pressure does not justify relaxing normative requirements

#### ## 1.2.3 Amendment Process
- **Step 1**: Document the defect/contradiction/unresolved case with exact rule IDs and specification sections
- **Step 2**: Determine whether the issue is an implementation defect or a specification defect
- **Step 3**: If implementation defect, repair the implementation (do not weaken specifications)
- **Step 4**: If specification defect, open a controlled specification erratum/amendment process
- **Step 5**: Do not invent an implementation-local interpretation
- **Step 6**: Resume implementation only after the authoritative decision is available
- **Step 7**: Record the decision and update affected rule IDs and specification text

#### ## 1.2.4 Governance Oversight
- **Architecture review board**: Responsible for evaluating amendment requests; consists of specification custodians and senior implementation engineers
- **Review timeline**: Minimum 30-day public review period for any amendment request; emergency amendments possible with 2/3 board vote and documented justification
- **Reverse amendment**: Once a feature is removed or restricted via erratum, it cannot be silently reinstated; a new erratum must explicitly restore the feature with updated semantics

### ## 1.3 Bootstrap Language

#### ## 1.3.1 Initial Bootstrap
- **Selected language**: Rust (pinned toolchain)
- **Pinning mechanism**: `rust-toolchain.toml` with exact version specification
- **Initial state**: Repository bare-minimum; `Cargo.toml` with `[workspace]`, `resolver = "2"`, `members = ["compiler/*", "tools/*"]`
- **Lints enforced**: `[workspace.lints.rust] unsafe_code = "forbid"`, `unreachable_pub = "deny"`, `unused_must_use = "deny"`
- **CI gate (0.0.0.1)**: `rustfmt --check` and `cargo clippy` on the empty workspace exit 0
- **Gate (0.0.0.1)**: Run `omni-audit`; it must pass on the empty workspace

#### ## 1.3.2 Intended Evolution Path
- **Phase 1**: Rust → Rust Stage-0 compiler
  - Stage-0 is a strict syntactic and semantic subset of Aurora/Omni
  - Every Stage-0 source parses and has identical observations under Edition 1
  - Stage-0 includes: UTF-8/ASCII identifiers, modules, structs/enums, fixed integers/bool/byte, arrays/slices, functions, generics without specialization, Result/Option
  - Stage-0 excludes: macros, runtime reflection, async, managed domains, dynamic typing, relations, decimal, scalable vectors/tensors, profile libraries, user-defined effect handlers
  - Stage-0 grammar generated by disabling named Edition 1 productions, not maintained as unrelated grammar
  - Stage-0 uses same checked arithmetic, evaluation order, initialization/drop, panic-abort policy, provenance, and C/omni_v1 ABI subset
  - Seed compiler consumes canonical Stage-0 source and emits one reference object format/ISA plus deterministic diagnostic records
  - Every Stage-0 restriction represented by feature predicate and tested against Edition 1 parser/model to prove subset inclusion

- **Phase 2**: Rust Stage-0 compiler → Rust compiler capable of compiling Stage-0/partial Omni
  - Compiler can compile its own Stage-0 subset
  - Can compile partial Omni programs (subset of Stage-1 features)
  - Error messages begin to reflect Omni semantics rather than Rust semantics

- **Phase 3**: Rust compiler capable of compiling Stage-0/partial Omni → Omni implementation of compiler
  - First Omni-written compiler capable of compiling Stage-0
  - Capability to compile Stage-1 features (but may still bootstrap via Rust)
  - Initial DDC comparisons begin (same source, Omni compiler vs. Rust compiler)

- **Phase 4**: Omni implementation of compiler → Omni compiles Omni
  - Omni self-hosted: Omni compiler written in Omni
  - DDC with independent Omni compiler lineages
  - Rust reduced to validation/bootstrap-recovery role

- **Phase 5**: Omni compiles Omni → DDC / independent verification → Rust reduced to validation/bootstrap-recovery role
  - Rust no longer needed for compilation; serves as validation oracle
  - Full DDC: three+ independent compiler lineages all targeting Omni
  - Differential testing and qualification
  - Rust remains for regression testing and bootstrap verification only

#### ## 1.3.3 Bootstrap Constraints
- **No stage may consume the next stage's version namespace merely for convenience**
- **Semantic hash must remain unchanged** when only comments, whitespace, line/column positions, or absolute checkout location changes and those items are outside the semantic hashing domain
- **A semantic input change must change the relevant semantic digest**
- **Workspace topology**: `Cargo.toml` with `[workspace]`, `resolver = "2"`, `members = ["compiler/*", "tools/*"]`
- **Workspace lints**: `[workspace.lints.rust] unsafe_code = "forbid"`, `unreachable_pub = "deny"`, `unused_must_use = "deny"`
- **Empty workspace gate**: CI successfully runs `rustfmt --check` and `cargo clippy`; runs `omni-audit`; it must pass
- **Serilization gate**: Serialize with zero whitespace
- **Foundational integrity**: The initial baseline commit and its claims must not be reused as evidence that the current tree is still a scaffold; current implementation status is determined by the actual source tree, tests, specifications, CI, and qualification evidence

---
## ## 2. Specification Authority and Digest Domain

### ## 2.1 Normative vs Non-Normative

#### ## 2.1.1 Normative Specification Definition
- **Definition**: The normative Edition 1 specification consists exclusively of modules listed as required in the signed release manifest (ROOT-0001)
- **Authority**: Signed release manifest > implementation claims > informal convention
- **Non-negotiable conformance rules**:
  - ROOT-0001: The normative Edition 1 specification consists exclusively of modules listed as required in the signed release manifest
  - ROOT-0002: No compiler implementation or reference interpreter may override or invent language semantics
  - ROOT-0005: Safe, well-typed Edition 1 programs SHALL NOT exhibit undefined behavior
  - TERM-0005: An erroneous program SHALL NOT produce a conforming release artifact

#### ## 2.1.2 Non-Normative Elements
- Implementation deviations from the normative specification
- Informal commentary and explanatory text
- User-guide-style explanations
- Comparative references to other languages (Ada, Rust, C, etc.) — noted as "comparative evidence, not an Omni rule"
- Design rationale and decision records — retained for historical audit but not normative
- Addressed bugs and their fixes — recorded in CHANGELOG.md but not part of the normative specification

#### ## 2.1.3 Implementation Deviations
- **Documentation requirement**: Every implementation deviation from the normative specification must be documented as an erratum with exact rule IDs
- **Invention prohibition**: Implementers must not invent an implementation-local interpretation to resolve contradictions or gaps
- **Conflict resolution**: When contradictions exist, the authority hierarchy (manifest > implementation > informal) determines the winning interpretation
- **Errata process**: Contradictions or unclassified cases trigger the controlled specification erratum/amendment process (see 1.2)

#### ## 2.1.4 Authority Precedence Chain
1. Signed release manifest (highest authority)
2. Normative specification text (second authority)
3. Implemented behavior in the current repository tree (third authority — only what is actually coded/tested, not what is desired or claimed)
4. Informal convention or external reference (lowest authority; "comparative evidence" only)
5. Designer intent or rationale (historical, not normative)

### ## 2.2 Specification Digest

#### ## 2.2.1 Digest Purpose
- **`spec_tree_sha256`**: Digest of the normative Edition 1 specification content only
- **Purpose**: Ensures specification integrity; any modification to the normative spec (within the excluded set) changes the digest; mismatched digest fails conformance claims fail-closed
- **Excluded from digest**: Generated implementation manifests and release evidence under `spec/manifest/` and `spec/release/`

#### ## 2.2.2 Canonicalization Procedure (Atomic Steps)
1. **Directory traversal**: Walk included directories in lexicographic order of slash-joined relative paths
   - Include: All files under `grammar/`, `registry/`, `models/`, `data/`, `schemas/`
   - Exclude: Per the exclusion set (see 2.2.3)
   - Sort key: Slash-joined relative path (e.g., `grammar/omni-edition1.ebnf`)
2. **Byte normalization**: Normalize file bytes CRLF/CR → LF
   - Each file's bytes are independently normalized
   - Normalization occurs before hashing
   - Original byte mapping preserved for diagnostics and edits (SRC-0002)
3. **File hashing**: Hash each file with SHA-256
   - Output: 64-character lowercase hex string
   - Each file produces exactly one hash
   - No salting; pure SHA-256 of normalized bytes
4. **Concatenation hashing**: Hash the concatenation of `path + LF + hex + LF` lines
   - `path`: The relative path as walked in step 1
   - `LF`: Literal line-feed character
   - `hex`: The 64-character hex hash from step 3
   - `LF`: Literal line-feed character
   - Format per file: `path + LF + hex + LF`
   - Example line: `grammar/omni-edition1.ebnf\nhashvalue\n\n`
   - All such lines concatenated (no separator between files beyond the implicit structure)
   - Final hash: SHA-256 of the entire concatenated byte string

#### ## 2.2.3 Exclusion Set (Precise Definition)
The following are **excluded** from the `spec_tree_sha256` digest (will not be walked/hashed):

##### ## 2.2.3.1 Generated Manifests and Release Evidence
- All files under `spec/manifest/` directory
- All files under `spec/release/` directory
- Any artifact whose primary purpose is to contain or convey the `spec_tree_sha256` digest itself
- Release candidate manifests, version tags, and release announcement documents

##### ## 2.2.3.2 Build Artifacts and Intermediate Products
- `target/` directory (generated artifacts)
- `.cargo/` directory (Cargo configuration)
- `.delta/` directory (delta tracking)
- Any `.rs` file not in `compiler/` or `tools/`
- Build scripts outside `scripts/`
- Compiled `.o`, `.a`, `.so`, `.dll` files
- Executable artifacts

##### ## 2.2.3.3 Scaffolding and Tooling
- `.git/` directory (version control metadata)
- `.github/` directory (CI/CD configuration)
- Goal/specification tracking directories: `.goals/`, `.kilo/`
- Configuration files: `.rustfmt.toml`, `.gitattributes`
- Documentation files outside `spec/` and `docs/` (e.g., `README.md`, `LICENSE`, `CHANGELOG.md`)
- Script files in `scripts/` (considered build tooling, not specification content)

##### ## 2.2.3.4 Empty/Skeletal Directories
- Directories containing only `.gitkeep` files
- Directories with no normative content
- Directories explicitly marked as excluded in the specification manifest

##### ## 2.2.3.5 Non-Specification Content
- Human-readable documentation not part of the normative grammar/semantics/verification tree
- Citation and reference bibliographies
- Typo logs and errata tracking documents (separate from the normative errata process)
- Developer communication logs and meeting notes
- Marketing materials and press releases

#### ## 2.2.4 Computed Digests (Current State)
- **Current digest** (gate-closure content by REL-0007 machine-block formatting): `7e63431f3c624f9818454f076c38f5cde3d517ace9e919926d6188bc177be769`
  - Supersedes previous values
  - Recovered by REL-0007 machine-block formatting in `candidate2-erratum.md`
  - This is the authoritative current digest
- **Earlier digest** (by gate-closure content, superseded): `07762061…`
  - Recovered stage0 rules/texts, erratum, widened patterns — not by contradiction
  - Superseded by the current digest
- **Previous to that**: (other earlier values, now obsolete)

#### ## 2.2.5 Digest Integrity Verification
- **omni-canon not yet started**: As of the current state, the `omni-canon` canonicalization engine has not been implemented
- **Integrity status**: Without `omni-canon`, digest integrity must be established from the actual code and specification, not accepted solely from notes
- **Future state**: When `omni-canon --spec-tree` is implemented, it will compute the digest; `omni-conform` will reject any manifest/gate digest mismatch fail-closed
- **Verification gap**: Until `omni-canon` is started and validated, digest claims should be qualified with "digest integrity not yet independently verified by `omni-canon`"

### ## 2.3 Conflict Resolution

#### ## 2.3.1 EBNF vs VIBE-GRAM Conflict
- **The conflict**: The normative EBNF (`spec/grammar/omni-edition1.ebnf`, 390 lines) predates the Candidate-2 amendment and contains no pipeline-operator, newline-termination, projection-shorthand, or command-call productions
- **The requirement**: VIBE-GRAM rules (VIBE-GRAM-0001 through VIBE-GRAM-0007) normatively require these productions
- **The status**: Artifact binds the EBNF as-is; the tree does not modernize the grammar
- **Grammar modernization**: Belongs to the grammar milestone; not yet started

#### ## 2.3.2 Resolution Options (Considered and Rejected)
- **Option A: Automatic modernization of EBNF to include VIBE-GRAM productions**
  - Rejected: Would invalidate the SHA-256 digest of the spec tree; would require a new edition (Edition 2)
  - Impact: Breaks all existing conformance claims and DDC comparisons
  
- **Option B: Maintain EBNF as-is; VIBE-GRAM productions documented separately**
  - Rejected: Creates an authority conflict; which source is normative?
  - Impact: Ambiguity in implementation; some implementations follow EBNF, some follow VIBE-GRAM

- **Option C: Grammar milestone starts; new edition (Edition 2) planned**
  - Status: Not yet started; requires formal planning
  - Impact: Clean break; existing artifacts remain valid under Edition 1 rules

#### ## 2.3.3 Current Resolution
- **The tree binds the artifact as-is**: The normative EBNF stands as written; VIBE-GRAM productions are acknowledged as normative requirements but the EBNF does not currently contain them
- **Any resolution must document the precedence decision**: Whichever approach is chosen, the decision and its rationale must be formally documented
- **Affected rule IDs**: Any grammar production changes require updates to associated rule IDs (e.g., if a new production is added, a new rule ID is assigned; if an existing production is modified, the old rule ID is superseded by a new one)
- **Update procedure**: 
  1. Document the decision and rationale
  2. Update the EBNF or add a modernization supplement
  3. Assign new or updated rule IDs in `registry/rules.json`
  4. Update `spec_tree_sha256` (the digest will change, triggering a new release cycle)
  5. Publish errata for all affected rule IDs
  6. Inform all in-flight conformance claims of the change

#### ## 2.3.4 Precedence Hierarchy (Within Specification Authority)
1. **Signed release manifest** — highest authority; cannot be overridden by internal documents
2. **Normative EBNF** — the current canonical grammar form; binds the artifact
3. **VIBE-GRAM rules** — normative requirements; conflict with EBNF must be resolved, not ignored
4. **Implementation behavior** — what the current code does; lower authority than the specification
5. **Comparative references to other languages** — "comparative evidence, not an Omni rule" (as stated in SECURITY.md)
6. **Developer rationale and design decisions** — historical; not normative unless explicitly incorporated into the specification

---
## ## 3. Conformance and Certification

### ## 3.1 Conformance Claim Structure

#### ## 3.1.1 Claim Record Fields (CONF-0001 through CONF-0011)
Every conformance claim must include stable information for each identified field:

##### ## 3.1.1.1 CONF-0001: Conformance Case Structure
- **case_id**: Stable identifier for the conformance case (UUID or equivalent; version-stable, not changing between suite revisions)
- **rule_ids**: List of rule IDs to which this case applies (each must be in `Ratified` or `Candidate` state)
- **suite_version**: Version of the conformance suite against which this case is run
- **source_artifact_inputs**: Digested hashes of all source artifacts used as input; must be content-addressed
- **expected_classification**: One of: `pass`, `fail`, `error`, `undefined`; must match the observed result for the claim to be valid
- **expected_result**: Detailed description of the expected observation; must be specific enough to validate against actual behavior
- **target_profile_applicability**: Which assurance profiles (safe, distributed, real-time, high-assurance, general-purpose) this case applies to
- **required_limits**: Any resource or execution limits that must be enforced during testing (memory bounds, timeouts, etc.)
- **oracle_model_version**: Version of the oracle/model used for evaluation; must be recorded for reproducibility
- **allowed_implementation_defined_choices**: List of implementation-defined choices that are acceptable for this claim to pass

##### ## 3.1.1.2 CONF-0002: Test Kinds
- **parse_pass/fail**: Does the source parse successfully/failedly?
- **static_pass/fail**: Does the program pass static checking (type checking, ownership analysis, effect tracking)?
- **run-value**: What value does the program produce at runtime? (for pure functions, closed computations)
- **run-output**: What output does the program produce? (including stdout, files written, etc.)
- **run-fault**: Does the program fault at runtime? (if so, what kind of fault, at what program point)
- **memory-litmus**: Does the program exhibit behaviors allowed by the Litmus relaxed memory model? (for concurrency tests)
- **ABI-cross**: Does the program correctly cross ABI boundaries? (calling conventions, name mangling, layout compatibility)
- **wire-golden**: Does the wire/persistent encoding match the golden reference schema?
- **build-resolution**: Does the build complete successfully from source to artifact?
- **security-policy**: Does the program respect the security policy constraints for its profile?
- **differential**: Does the program pass differential testing against an independent implementation?
- **metamorphic**: Does the program satisfy metamorphic relations (similar inputs produce similar outputs in a specified way)?
- **performance-contract**: Does the program meet stated performance characteristics (throughput, latency, memory usage)?

##### ## 3.1.1.3 CONF-0003: Coverage Requirements
- **Positive cases**: For every ratified core rule, at least one positive test case must exist (program that exercises the rule and passes)
- **Negative cases**: For every ratified core rule, at least one negative test case must exist (program that exercises the rule and is expected to fail)
- **Boundary cases**: Test cases at the boundaries of rule applicability (edge cases, extreme values, limit states)
- **Interaction cases**: For rules that interact with other rules, test cases covering the interaction (combinatorial coverage of rule interactions)
- **High-risk rules**: Additionally require an independent model or implementation (not just another test of the same implementation); the independent model must produce consistent results

##### ## 3.1.1.4 CONF-0004: Test Correction Process
- **Versioned test erratum**: A test may be corrected only through a versioned test erratum linked to the rule ID
- **Immutability of prior artifacts**: Prior suite artifacts remain immutable; corrected conformance claims identify the overlay (which version of the erratum, what was changed)
- **Correction chain**: Errata chain is tracked; each erratum references the previous version; no erratum deletes prior versions; all versions are preserved
- **Invalid until corrected**: When a test conflicts with normative rules, the test is invalid until a versioned erratum corrects it

##### ## 3.1.1.5 CONF-0005: Test Semantics
- **Tests do not create semantics**: A test's existence or result does not change the language semantics
- **Conflict resolution**: When a test conflicts with normative rules, the test is invalid until corrected
- **Semantic primacy**: Normative specification semantics always trump test results; a test cannot override a normative rule
- **Retrospective correction**: Even tests from earlier suite versions must yield to normative rules when contradictions exist

##### ## 3.1.1.6 CONF-0006: Strict Mode
- **Strict mode disables extension syntax/semantics**: When strict mode is active, any unrecognized extension syntax or semantics cause the artifact to be rejected
- **Namespaced extensions**: May be tested only in separate claims; cannot be mixed with strict-mode claims
- **Extension identification**: An "extension" is any syntax or semantics not present in the base Edition 1 specification (as defined by the grammar and rule set)
- **Strict mode activation**: Declared in the implementation claim (CONF-0007); once declared, applies to all conformance cases in that claim

##### ## 3.1.1.7 CONF-0007: Implementation Claim
- **edition**: The Omni edition being claimed (currently "Edition 1" or "1.0.0.0")
- **modules**: Which modules from the normative specification are claimed as implemented (must be a subset of the manifest's required modules)
- **profiles**: Which assurance profiles are claimed (safe, distributed, real-time, high-assurance, general-purpose)
- **targets**: Target identities (per TARGET-0001 through TARGET-0009) for which the claim applies
- **limits**: Implementation-defined choices, deviations, and limits (must be explicitly listed; omitted limits default to the normative specification)
- **external_ABI_data_versions**: Versions of any external ABI data (wire formats, schema versions) that the implementation relies upon
- **deviations**: Any ways in which the implementation differs from the normative specification; must be documented individually
- **test_result_digest**: SHA-256 digest of the test results; enables independent verification of test outcomes

##### ## 3.1.1.8 CONF-0008: Claim Distinction
- **Parser-only claims**: Claim only that the parser accepts/rejects certain inputs; does not claim semantic implementation
- **core-subset claims**: Claim conformance for a core subset of the language (must explicitly define the subset)
- **profile claims**: Claim conformance for a specific assurance profile (must declare which profile; cannot claim multiple profiles simultaneously unless explicitly architected)
- **platform claims**: Claim conformance for a specific target platform (must declare target per TARGET-0001)
- **distribution claims**: Claim conformance for distribution artifacts (must declare wire-golden compliance)
- **complete-release claims**: Claim complete conformity to Edition 1; partial implementations cannot claim complete conformity
- **Claim independence**: Each claim type is distinct; a single implementation may hold multiple claim types but each must be separately justified and evidenced

##### ## 3.1.1.9 CONF-0009: Certification Structure
- **assurance_level**: One of: `self-attested`, `independently_audited`, `authority_certified`
- **auditor**: If independently audited or authority-certified, the name/audit firm/authority of the auditor
- **evidence**: Reference to the evidence body (test suite run, proof obligations discharged, differential testing results, etc.)
- **waivers**: Any waivers granted; each waiver must have an ID, reason, expiration date, and narrow scope
- **expiration**: Expiration date of the certification; after expiration, the claim is no longer valid without re-certification
- **revocation_status**: One of: `active`, `revoked`, `suspended`; if revoked, the reason and date must be recorded
- **revocation_reason**: If revoked, the reason (evidence falsified, blocker soundness/security defect, required artifacts become untrusted)

##### ## 3.1.1.10 CONF-0010: Waiver Constraints
- **Waivers cannot excuse soundness violations**: No waiver can make a soundness violation acceptable
- **Waivers cannot excuse false accepted behavior**: A waiver cannot declare incorrect behavior as acceptable
- **Waiver documentation**: Each waiver must have:
  - waiver_id: Unique identifier
  - reason: Specific, documented reason
  - expiration: Expiration date/time
  - scope: Narrowly defined scope (e.g., "does not apply to target x86_64-unknown-linux-gnu" rather than "general waiver")
  - approved_by: Who approved the waiver (auditor, authority, self-attestation body)
- **Waiver narrowing**: Waivers must narrow the claim; a broad waiver is invalid

##### ## 3.1.1.11 CONF-0011: Certification Revocation
- **Evidence falsification**: If evidence supporting the certification is proven false, the certification is immediately revoked
- **Blocker soundness/security defect**: If a blocker soundness or security defect is discovered that invalidates the claim, immediate revocation
- **Required artifacts become untrusted**: If any required artifact (compiler, verification tool, etc.) becomes untrusted, immediate revocation
- **Revocation process**: 
  1. Revocation is signed and published with digested evidence
  2. Revocation reason is recorded
  3. Revoked artifacts cannot be selected or executed absent an audited emergency waiver
  4. Emergency waiver process: Separate from regular waivers; requires high-level approval; documented; temporary
- **Publication**: Revocation is published through the same channels as certification; format: signed document with digests

### ## 3.2 Assurance Profiles

#### ## 3.2.1 Profile Framework
- **Common language foundation**: All profiles inherit from the same normative specification core; no profile redefines the core language semantics
- **Profile-specific deviations**: Any deviation from the core contract must be explicit and testable; undocumented deviations are violations
- **Profile inheritance**: Profiles may inherit restrictions from stricter profiles; a stricter profile's restrictions are automatically inherited by looser profiles unless explicitly overridden

#### ## 3.2.2 Defined Profiles

##### ## 3.2.2.1 `safe` Profile
- **Core restriction**: No `unsafe` blocks/functions
- **Core restriction**: No `foreign` effect
- **Core restriction**: No raw pointers (including `*const T`, `*mut T`, raw reference types)
- **Enforcement**: Via type system and checker; the compiler rejects any program attempting these features
- **Rationale**: simplest provenance model; eliminates entire classes of memory-safety bugs; suitable for most application development where memory safety is paramount
- **Allowed features**: All features except `unsafe`, `foreign` effect, and raw pointers; includes GC, dynamic dispatch, runtime reflection
- **Verification**: Standard static checking suffices; full formal verification not required but welcomed
- **Profile identifier**: `PROFILE-safe`

##### ## 3.2.2.2 `distributed` Profile
- **Core restriction**: State sharing via message passing only; no shared mutable state across tasks/threads
- **Core restriction**: No global mutable variables accessible from multiple execution agents
- **Core requirement**: All inter-task communication via typed channels (CONC-0009 through CONC-0012)
- **Core requirement**: Region transfer in messages commits ownership atomically with enqueue (CONC-0021)
- **Rationale**: Enables reasoning about concurrent programs; eliminates data races by construction; suitable for multi-core/multi-processor systems
- **Allowed features**: Message passing, bounded channels, structured task scopes, quorum policies; dynamic dispatch, generics, effects (except `foreign`); no unsynchronized shared state
- **Verification**: Model checking of concurrent properties recommended; Ravenscar-profile-like restrictions enforced by type system
- **Profile identifier**: `PROFILE-distributed`

##### ## 3.2.2.3 `real-time` Profile
- **Core restriction**: Bounded execution times for all functions (must be provably bounded or annotated with WCET - Worst-Case Execution Time)
- **Core requirement**: Priority inheritance to prevent unbounded priority inversion
- **Core requirement**: Bounded preemption (maximum number of preemptions per function)
- **Core restriction**: No unbounded dynamic allocation in critical paths (GC pauses unacceptable)
- **Core requirement**: Deterministic task scheduling (scheduling decisions must be predictable and verifiable)
- **Rationale**: Safety-critical systems require timing guarantees; real-time constraints cannot be retrofitted after the fact
- **Allowed features**: Stack-bounded allocation, static data structures, priority-based preemption, timing annotations; effects permitted if they have bounded execution semantics; no unbounded GC
- **Verification**: Timing analysis; proof of bounded execution; schedulability verification
- **Profile identifier**: `PROFILE-real-time`

##### ## 3.2.2.4 `high-assurance` Profile
- **Core requirement**: Full formal verification discharge for Stage-0 subset (all feature predicates proved against the Edition 1 parser/model)
- **Core requirement**: Mandatory DDC (diverse double compilation) for every release
- **Core prohibition**: No undefined behavior of any kind (TERM-0005: erroneous program SHALL NOT produce conforming release artifact)
- **Core requirement**: Artifact provenance (UPDATE-0001 through UPDATE-0008) with complete metadata chain
- **Core requirement**: Conformance claims with full evidence (CONF-0001 through CONF-0011)
- **Rationale**: Highest assurance level; suitable for military, space-critical, and security-critical applications; provides defense-in-depth through multiple independent verification methods
- **Allowed features**: All features that can be formally verified; `unsafe` permitted only with full proof obligations discharged; `foreign` effect heavily restricted; no runtime reflection in critical paths
- **Verification**: Formal proofs (ACL), DDC, extensive testing, model checking, differential testing with independent implementations
- **Profile identifier**: `PROFILE-high-assurance`

##### ## 3.2.2.5 `general-purpose` Profile
- **Core purpose**: Productive application development; balance of features and verifiability
- **Core allowance**: GC (garbage collection) permitted
- **Core allowance**: Dynamic dispatch permitted
- **Core allowance**: Runtime reflection permitted
- **Core allowance**: `unsafe` permitted but not required; when used, proof obligations must be discharged
- **Core allowance**: `foreign` effect permitted
- **Verification**: Standard static checking + test suites; formal verification optional but recommended for critical components
- **Rationale**: Most developers productively use this profile; other profiles are available for critical paths; same language, different restrictions
- **Profile identifier**: `PROFILE-general-purpose`

#### ## 3.2.3 Profile Tailoring (ECSS-S-ST-00)
- **Per ECSS-S-ST-00**: Profiles may be tailored for specific project characteristics and constraints
- **Tailoring process**: Customer specifies tailoring parameters; supplier implements within profile constraints
- **Quality objectives**: Selected per ECSS-Q-ST-80; influence which profile requirements are mandatory vs. optional
- **Security considerations**: Tailoring for security based on selection of requirements related to verification, validation, and levels of proof demanded by software sensitivity
- **Dependability aspects**: Tailoring based on selection of requirements related to verification, validation, and levels of proof demanded by software criticality
- **Development constraints**: Type of software development (real-time, embedded, database, etc.) and target system characteristics taken into account
- **Tailoring restrictions**: 
  - No tailoring that removes a "core restriction" from a profile (core restrictions are immutable)
  - No tailoring that adds features prohibited for the profile (e.g., adding `unsafe` to the `safe` profile)
  - All tailoring deviations must be explicitly documented and tested
  - Tailoring conformance claim must be separate from the base profile claim

#### ## 3.2.4 Profile Transitions
- **Transition from `general-purpose` to `safe`**: Restrict features; may require refactoring; migration path documented
- **Transition from `safe` to `high-assurance`**: Add formal verification discharge; may require significant verification infrastructure; not automatic
- **Transition from `distributed` to `real-time`**: Combine message-passing determinism with timing guarantees; possible but requires careful design
- **Transition between any profiles**: Must follow the controlled language-amendment process (Section 1.2); cannot be done implicitly

### ## 3.3 Certification Revocation

#### ## 3.3.1 Revocation Triggers (Atomic Conditions)
Any one of the following triggers immediate certification revocation:

1. **Evidence falsification**:
   - Test results fabricated or manipulated
   - Proof obligations discharged under false pretenses
   - Differential testing results misreported
   - Any documented evidence proven false

2. **Blocker soundness/security defect**:
   - A defect that invalidates the core assurance claims
   - Example: Soundness violation allowing programs that should be rejected to pass
   - Example: Security vulnerability that breaks capability confinement
   - Example: Soundness defect in the verification toolchain itself

3. **Required artifacts become untrusted**:
   - Compiler binary become untrusted (compromise confirmed)
   - Verification tool becomes untrusted (bug found, malicious code inserted)
   - Dependent toolchain component (linker, assembler) becomes untrusted
   - Any artifact listed in the provenance chain (UPDATE-0001 through UPDATE-0008) is compromised

#### ## 3.3.2 Revocation Process
1. **Detection**: Defect/violation discovered (internal review, external report, independent audit)
2. **Verification**: Defect confirmed with evidence (not merely alleged)
3. **Signing**: Revocation document signed by the appropriate authority (self-attestation body, auditing authority, or regulatory authority)
4. **Publication**: Revocation published through certification channels
5. **Digest inclusion**: Revocation reason and artifact digests included in published revocation
6. **Effective date**: Revocation effective from publication date (or specified date)
7. **Emergency waiver pathway**: 
   - Separate from regular waivers (CONF-0010)
   - Requires high-level approval (program director, chief architect, or equivalent)
   - Documented with full justification
   - Temporary; expires after defined period
   - Must include mitigation plan

#### ## 3.3.3 Post-Revocation
- **Revoked artifacts**: Cannot be selected or executed absent an audited emergency waiver
- **Re-certification**: Requires full re-evaluation; prior certification history does not count toward new certification
- **Learning**: Revocation reason documented and used to improve specification, toolchain, or processes
- **No automatic reinstatement**: A revoked certification cannot be automatically reinstated; a new evaluation is required

---
## ## 4. Build Pipeline (Content-Addressed)

### ## 4.1 Build Action Semantics (Atomic Definitions)

#### ## 4.1.1 BUILD-0001: Build Action Purity
- **Pure function**: A build action is a pure declared function from content-addressed inputs and fixed parameters to content-addressed outputs plus structured diagnostics
- **Content-addressed inputs**: Inputs identified by their content hash (SHA-256); same content = same input identity, regardless of source location, timestamp, or enumeration order
- **Fixed parameters**: Parameters that are part of the build action's definition and do not vary between invocations (target profile, optimization level within a defined range, etc.)
- **Content-addressed outputs**: Outputs identified by their content hash; the output's identity is its hash, not its path location
- **Structured diagnostics**: Output includes a structured diagnostics block with:
  - `diagnostics`: Array of diagnostic entries
  - Each entry has: `level` (error, warning, info), `rule_id` (referenced rule), `message` (human-readable), `location` (source position), `severity` (numeric)
- **Idempotency**: Running a build action with the same inputs produces the exact same outputs (byte-for-byte); may be cached and re-used
- **Non-reproducibility**: If inputs differ (even in whitespace outside the semantic hashing domain), the output may differ; this is not a failure but a different build action

#### ## 4.1.2 BUILD-0002: Input Declaration
- **source_trees**: Source code trees, identified by their content hash; each tree is a content-addressed entity
- **dependency_artifacts**: Dependency artifacts (compiled libraries, intermediate representations, etc.), each with its own content hash
- **tools**: Tools used in the build, identified by content hash (compiler version, assembler, linker, etc.)
- **target_profile_manifests**: Target/profile manifest identifiers (content-hashed); declare which target/profile the build targets
- **environment_variables**: Environment variables explicitly named in the build configuration; undeclared variables cannot be observed (BUILD-0003)
- **build_capabilities**: Build capabilities available (SIMD support, multi-threading, network access, etc.); declared, not assumed
- **resource_limits**: Resource limits for the build (memory budget, time budget, temp directory size); exceeding these is a structured build failure (BUILD-0011)

#### ## 4.1.3 BUILD-0003: Observation Prohibition
- **Undeclared files**: Files not listed in `source_trees` or `dependency_artifacts` cannot be observed (read, imported, included) during the build
- **Undeclared environment**: Clock, timezone, locale, randomness, network, process state, home directory, registry state, and filesystem enumeration cannot be observed
- **Implication**: Builds are hermetic; no "leaks" of environment state; this enables reproducibility
- **Enforcement**: Compiler must reject any attempt to observe undeclared environment state; any such observation is a build error

#### ## 4.1.4 BUILD-0004: Network Access Denial
- **Default deny**: Build network access is denied by default
- **Fetch action**: Separate from regular build actions; a fetch action is:
  - Content-addressed
  - Policy-authorized (explicit permission in build config)
  - Produces immutable inputs for later offline build actions
- **Implication**: Builds that need external data must explicitly fetch it; the fetched data becomes a content-addressed input for future builds
- **Policy authorization**: Network fetch policies are declared in the build configuration; unauthorized fetches are rejected

#### ## 4.1.5 BUILD-0005: Sandbox Path Canonicalization
- **Sandbox paths**: Paths inside the build sandbox are canonical virtual paths
- **Host path exclusion**: Host paths do not enter object/debug/generated bytes except through normalized source remapping records
- **Source remapping**: Absolute host paths must be converted into a virtual namespace rooted at the workspace root
  - Example: `C:/Users/siddh/AppData/Local/...` → `/omni-root/...`
  - Conversion is deterministic and recorded in the build diagnostics
- **Implication**: Build artifacts are portable; the same source code produces the same artifacts regardless of host path structure

#### ## 4.1.6 BUILD-0006: Generated Source Identity
- **Generator artifact digest**: The SHA-256 hash of the generator tool itself; identifies which generator produced the output
- **Command/schema version**: The version of the build command and schema used
- **Exact inputs**: The content hashes of all inputs to the build action
- **Semantic extension settings**: Any semantic extension settings that affect the output (e.g., profile flags, feature enables)
- **Output path identity**: The identity of the output path within the sandbox (virtual path, not host path)
- **Full identity record**: `[generator_digest, command_version, schema_version, exact_input_hashes, semantic_extension_settings, output_virtual_path]`

#### ## 4.1.7 BUILD-0007: Incremental Cache Keys
- **Cache key composition**: Every semantically relevant query input and compiler/spec data version must be included in the incremental cache key
- **Missing input = correctness defect**: If any semantically relevant input is missing from the cache key computation, it is a correctness defect (not a performance issue)
- **Cache key elements**:
  - All source tree content hashes
  - All dependency artifact content hashes
  - All tool content hashes (compiler version, etc.)
  - Target/profile manifest content hash
  - Explicitly named environment variables' content hashes
  - Build capability content hashes (SIMD, etc.)
  - Resource limit values
  - Compiler/spec data versions (rule registry version, grammar version, etc.)
- **Cache key stability**: Cache keys must be stable across builds with the same inputs; changing a cache key without changing inputs is a bug

#### ## 4.1.8 BUILD-0008: Remote Cache/Execution Acceptance
- **Authenticated action digest**: Remote cache/execution results must have an authenticated action digest; the digest proves the result corresponds to the claimed inputs
- **Worker/toolchain attestation**: The worker/toolchain that produced the result must be attested (verified as genuine, not compromised)
- **Output digests**: The output's content hash must match the claimed output digest
- **Policy-compatible isolation**: The result must be compatible with the build's isolation policy (e.g., if the build declares "no network access", a result produced with network access is incompatible)
- **Rejection**: Any remote cache/execution result failing these checks is rejected; the build action re-executes locally
- **Implication**: Enables distributed/build scaling while maintaining security and reproducibility

#### ## 4.1.9 BUILD-0009: Atomic Output Publication
- **Atomic write**: Build action outputs are written atomically; partial output is never visible or treatable as a cache hit
- **Verification before publication**: Outputs are verified before being published to the cache or made available to subsequent build actions
- **Partial output never cache hit**: Even if part of the output is correct, the entire output must be complete and verified before it's treated as a cache hit
- **Write protocol**: 
  1. Write output to temporary location (outside cache)
  2. Verify output correctness (content hash matches expectation)
  3. Atomically move from temporary to cache location
  4. Mark as available in cache index
- **Failure mode**: If verification fails, the output is discarded; the build action re-executes; no partial output is retained

#### ## 4.1.10 BUILD-0010: Parallel Action Merge Order
- **Completion order independence**: Parallel action completion order cannot affect merged outputs
- **Canonical merge order**: Merge order is canonical by declared key
- **Declaration**: Each parallel action declares its merge key; the merge orchestrator uses this key to determine order
- **Deterministic merge**: Given the same inputs and same declared merge keys, the merge output is deterministic (byte-for-byte identical)
- **Conflict resolution**: If two actions declare the same merge key, the build fails with a structured error (not silently picked one over the other)

#### ## 4.1.11 BUILD-0011: Resource-Limit Failure
- **Structured build failure**: Resource-limit failure is a structured build failure; it is **not** permission to emit a partial release artifact
- **Failure diagnostics**: Output includes structured diagnostics with:
  - `failure_type`: "resource_limit"
  - `resource`: Which resource was exhausted (memory, time, disk, etc.)
  - `limit`: The declared limit
  - `actual`: The actual amount consumed
  - `action`: Which build action triggered the failure
- **No partial artifact**: The build stops; no partial release artifact is emitted
- **Retry**: The build may be retried with adjusted resource limits or optimized resource usage
- **Relation to BUILD-0003**: Resource-limit failures do not observe undeclared environment; they only check declared limits

### ## 4.2 Build Gates (Implementation Roadmap)

#### ## 4.2.1 Stage 1.1: Foundation (0.0.0.1 → 0.0.0.4)
**Purpose**: Establish the bedrock upon which all later implementation depends.

**Phase 1.1.1**: Repository initialization with proper directory structure
- Create directory layout matching the specification's expectations
- Ensure `.cargo/`, `scripts/`, `spec/` subdirectories exist with correct naming
- Verify `Cargo.toml` can be parsed and loaded

**Phase 1.1.2**: Rust toolchain pinning in `rust-toolchain.toml`
- Pin exact Rust version (currently `1.95.0`)
- Specify required features and targets
- Document the toolchain pin rationale (qualification boundary)

**Phase 1.1.3**: Quality/CI boundary setup (ceilings, fuzz budgets)
- Configure `cargo clippy` allow/deny lists
- Set fuzz budget thresholds (per-crate or per-project)
- Configure `rustfmt` style checks
- Establish quality ceilings (maximum allowed cyclomatic complexity, etc.)

**Phase 1.1.4**: Rule registry schema with lifecycle states
- Create `registry/rules.json` with initial rule set
- Define lifecycle states: `Proposed`, `Candidate`, `Ratified`, `Deprecated`, `Superseded`, `Withdrawn`
- Establish rule-texts.json schema
- Populate initial rules (foundation gates 0.0.0.2 through 0.0.0.4)

**Phase 1.1.5**: Specification manifest creation
- Create the specification manifest that lists all normative artifacts
- Compute and record `spec_tree_sha256`
- Document the exclusion set (Section 2.2.3)

**Phase 1.1.6**: Canonicalization engine (`omni-canon`) — critical path item
- Implement `omni-canon --spec-tree` to compute the SHA-256 digest
- Validate the procedure against known-good specification trees
- Ensure `omni-conform` can reject digest mismatches
- **This is the critical path item**; all later work depends on its completion

**Phase 1.1.7**: Quality/CI gate verification
- Verify `rustfmt --check` passes on empty workspace
- Verify `cargo clippy` passes on empty workspace
- Verify `omni-audit` passes on empty workspace
- These three gates must all exit 0 before Stage 1.2 can begin

#### ## 4.2.2 Stage 1.2: Lexer/Parser (0.0.1.x → 0.0.2.x)
**Purpose**: Complete ERR3-0010 through ERR3-0018 and ERR3-0020 through ERR3-0027.

**Phase 1.2.1**: Byte cursor and UTF-8 decoder with BOM handling at offset zero
- Implement byte-level cursor traversal of source text
- Handle UTF-8 BOM (Byte Order Mark) at byte offset zero (accepted, then excluded from semantic hashing)
- Implement UTF-8 decoding errors (outside comments and literals: source error per SRC-0005)
- Implement BOM exclusion from semantic hashing (SRC-0001)

**Phase 1.2.2**: Source normalization and canonical line-ending handling
- Normalize CRLF and CR to LF (SRC-0002)
- Preserve original-byte mapping for diagnostics and edits (SRC-0002)
- Implement line-ending sensitivity in parser (comments and literals may behave differently)

**Phase 1.2.3**: Token kinds, trivia preservation, identifiers/keywords
- Implement 80+ keyword tokens (hard-reserved set)
- Implement contextual keyword set (separate from hard-reserved)
- Preserve trivia (comments, whitespace) in CST (CONST-0010 equivalent)
- Implement identifier token kind with Unicode NFC after tokenization (SRC-0003)
- Implement keyword classification: contextual vs hard-reserved (ERR3-0016)

**Phase 1.2.4**: Unicode identifier data ingestion and confusable validation
- Ingest Unicode identifier property data (XID_Start, XID_Continue)
- Validate confusable characters (visually similar but distinct Unicode characters)
- Reject identifiers with confusable characters (security requirement)
- Cache identifier property data for performance

**Phase 1.2.5**: Numeric literal state machines with ERR3-0014 compliance
- Implement decimal literal state machine
- Implement binary literal state machine (0b prefix)
- Implement octal literal state machine (0o prefix)
- Implement hex literal state machine (0x prefix)
- Reject leading `.5` as float; require `0.5` (ERR3-0014)
- Implement `.5identifier` as projection (not float) (ERR3-0014)
- Implement numeric literal diagnostics

**Phase 1.2.6**: Character/string literals and raw strings
- Implement character literal parsing (single quotes)
- Implement string literal parsing (double quotes)
- Implement raw string literals (r"..." prefix)
- Implement raw string with matching # delimiters (r#"..."#)
- Implement escape sequences within literals (\", \\, \n, \t, etc.)
- Implement raw string escape handling (minimal; mostly passthrough)

**Phase 1.2.7**: Operators and punctuation with maximal-munch
- Implement maximal-munch operator tokenization (e.g., `>>` is two `>` operators, not `>>` operator from C++)
- Implement punctuation token kinds
- Implement operator precedence parsing foundation
- Implement lexical diagnostics for invalid operators

**Phase 1.2.8**: Context-sensitive generic punctuation splitting
- Implement splitting of generic punctuation based on context
- Example: `::` in different contexts means different things
- Implement tokenizer state tracking for context sensitivity

**Phase 1.2.9**: Lexical diagnostics
- Implement error reporting for lexical errors
- Implement warning reporting for non-fatal lexical issues
- Implement trivia preservation in diagnostics (comments retained)

**Phase 1.2.10**: Deterministic EOF behavior
- Implement deterministic end-of-file handling
- Ensure EOF does not depend on host filesystem or enumeration order
- Implement EOF diagnostics if applicable

**Phase 1.2.11**: Lossless CST with comment/whitespace preservation
- Construct CST (Concrete Syntax Tree) that preserves all original source bytes
- Preserve comments (both `//` and `/* */`)
- Preserve whitespace structure (indentation, blank lines)
- Implement `cst.text() == original bytes` exactly (ERR3-0010)

**Phase 1.2.12**: Grammar productions from normative data
- Generate parser grammar from `spec/grammar/omni-edition1.ebnf`
- Ensure parser accepts all and only the productions in the EBNF
- Validate against the normative grammar

**Phase 1.2.13**: Deterministic recursive-descent declaration parsing
- Implement declaration parsing using deterministic recursive-descent
- Ensure no ambiguity in declaration parsing
- Implement error recovery that does not silently change semantics

**Phase 1.2.14**: Pratt-style expression parsing
- Implement Pratt parsing (binding power table) for expression parsing
- Ensure expression parsing is deterministic
- Implement expression diagnostics

**Phase 1.2.15**: Pattern grammar and generic syntax
- Implement pattern syntax for `match` expressions and `let` bindings
- Implement generic syntax (angle-bracketed type parameters, etc.)
- Implement pattern matching diagnostics

### ## 4.3 Provenance and Evidence (UPDATE-0001 through UPDATE-0008)

#### ## 4.3.1 UPDATE-0001: Update Metadata Chain (Atomic Structure)
- **root**: Versioned root metadata; versioned and updated only through chain validated from trusted root with rollback protection (UPDATE-0002)
- **timestamp**: Timestamp metadata; prevents freeze and detects stale snapshots
- **snapshot**: Snapshot metadata; binds exact versions/hashes of delegated metadata and prevents mix-and-match (UPDATE-0003)
- **targets**: Target metadata; binds package/artifact identity, version, length, digest, and custom compatibility fields, preventing wrong-package substitution (UPDATE-0004)
- **delegation**: Delegation roles with threshold signatures and expiration
- **revocation**: Revocation identifies exact content digests/keys/versions and effective time/severity (UPDATE-0007)
- **advisory**: Advisory roles for non-critical recommendations

#### ## 4.3.2 UPDATE-0002: Root Metadata Versioning (Atomic Chain)
- **Versioning**: Root metadata is versioned; new version supersedes old
- **Rollback protection**: Chain validated from trusted root with rollback protection; cannot silently roll back to a compromised earlier version
- **Update mechanism**: Root metadata updated only through a chain validated from trusted root
- **Immutable once set**: After initial setting, root metadata can only be updated through the validated chain
- **Expiration**: Root metadata has an expiration (using authorized clock or offline freshness policy) (UPDATE-0005)

#### ## 4.3.3 UPDATE-0003: Timestamp and Snapshot Prevention (Atomic Binding)
- **Timestamp prevents freeze**: Timestamp metadata prevents the metadata chain from being "frozen" at an old version
- **Detects stale snapshots**: If timestamp indicates the snapshot is older than authorized freshness, it is rejected
- **Snapshot binds exact versions**: Snapshot metadata binds exact versions/hashes of delegated metadata
- **Prevents mix-and-match**: Cannot pick and choose versions from different snapshots; must use one complete snapshot
- **Structure**: Snapshot includes: list of delegated metadata IDs, their exact versions, their SHA-256 hashes

#### ## 4.3.4 UPDATE-0004: Target Metadata Anti-Substitution (Atomic Binding)
- **Package/artifact identity**: Target metadata binds the package/artifact identity
- **Version**: The version of the package/artifact
- **Length**: The length of the package/artifact (in bytes)
- **Digest**: The SHA-256 digest of the package/artifact
- **Custom compatibility fields**: Additional fields for compatibility declarations
- **Anti-substitution**: Prevents wrong-package substitution; the metadata uniquely identifies the correct package
- **Structure**: `[identity, version, length, digest, compatibility_fields...]`

#### ## 4.3.5 UPDATE-0005: Client Version Enforcement (Atomic Monotonic)
- **Monotonically nondecreasing**: Clients must enforce monotonically nondecreasing metadata versions
- **Expiration using authorized clock**: Expiration based on an authorized clock (not user-controllable)
- **Or offline freshness policy**: Alternatively, an offline freshness policy may be used (e.g., "this metadata is fresh if it's from a build within the last 30 days")
- **Maximum metadata/target sizes**: Clients enforce maximum sizes for metadata and target declarations
- **Rejection**: Clients reject metadata with versions older than the current; also reject oversized metadata

#### ## 4.3.6 UPDATE-0006: Endless-Data Defenses (Atomic Declarations)
- **Declared length before download**: Data must have declared length before download begins
- **Streaming limits**: Limits on streaming data consumption (prevents unbounded consumption)
- **Decompression limits**: Limits on decompression (prevents decompression bombs: ZIP, tar, etc. with inflated claims)
- **Hash verification**: Hash verification of downloaded data against claimed hash
- **Structure**: Every data download request must include: declared length, streaming rate limit, decompression fuel limit, claimed hash

#### ## 4.3.7 UPDATE-0007: Revocation (Atomic Identification)
- **Exact content digests/keys/versions**: Revocation identifies exact content digests, keys, and versions being revoked
- **Effective time/severity**: Revocation has an effective time and a severity level
- **Security-revoked artifact**: A security-revoked artifact is not selected or executed absent an audited emergency waiver (CONF-0011)
- **Revocation record structure**: `[revoked_digests, effective_time, severity_level, reason_code]`
- **Propagation**: Revocation propagates through the metadata chain; dependent artifacts are also affected

#### ## 4.3.8 UPDATE-0008: Offline Mirrors (Atomic Chain Preservation)
- **Complete signed metadata chain**: Offline mirrors carry the complete signed metadata chain
- **Checkpoints**: Checkpoints at regular intervals in the metadata chain
- **Mirror transport cannot override signatures**: Transport-level changes (e.g., network corruption, man-in-the-middle) cannot override the signed signatures in the metadata
- **Mirror integrity**: The mirror must carry the complete chain; partial chains are rejected
- **Usage**: Offline mirrors enable building without network access; still verified against the signed chain

### ## 4.4 Trusted Computing Base Inventory (BOOT-0008)

#### ## 4.4.1 TCB Inventory Structure
Lists exact source/binary digests, role, reason trusted, audit status, replacement strategy, and transitive dependencies for:

1. **Proof checker**: The tool that checks proof obligations; its digest, reason it's in the TCB, audit status, how it can be replaced
2. **Parser generator**: The tool that generates the parser; its digest, reason, audit status, replacement
3. **Solver**: The type solver/unification engine; its digest, reason, audit status, replacement
4. **Assembler**: The assembly-time tool; its digest, reason, audit status, replacement
5. **Linker**: The linking-time tool; its digest, reason, audit status, replacement
6. **Unicode data**: The Unicode data files (version, content hash); its digest, reason, audit status, replacement
7. **Crypto**: Cryptographic algorithms, keys, implementations; their digests, reason, audit status, replacement
8. **OS/firmware**: The operating system and firmware providing the execution environment; its digest, reason, audit status, replacement
9. **Signing tools**: The tools used to sign artifacts; their digests, reason, audit status, replacement

#### ## 4.4.2 Emergency Bootstrap Recovery
- **Archived source**: Copies of all source code, stored in reproducible offline media
- **Specs**: Copies of the specification, stored in reproducible offline media
- **Data**: Related data files, stored in reproducible offline media
- **Emulators**: CPU architecture emulators, stored in reproducible offline media
- **Independent seed paths**: At least two independent seed paths (different toolchains, different platforms) stored in reproducible offline media
- **Purpose**: Emergency bootstrap recovery if the primary bootstrap path is compromised or lost

---
## ## 5. Target Model

### ## 5.1 Canonical Target Identity (T

## ## 5.1 Canonical Target Identity (continued)

`TARGET-0001`: A canonical target identity is `arch-vendor-os-environment-abi[data-model]+feature-set`, with lowercase registered components and sorted feature names.

**Formal syntax**: `arch-vendor-os-environment-abi[data-model]+feature-set`

**Component breakdown**:
- `arch`: Architecture identifier (lowercase registered value)
- `-`: Separator character (literal hyphen)
- `vendor`: Vendor/portability identifier (lowercase registered value)
- `-`: Separator character (literal hyphen)
- `os`: Operating system identifier (lowercase registered value)
- `-`: Separator character (literal hyphen)
- `environment`: Execution environment identifier (lowercase registered value)
- `-`: Separator character (literal hyphen)
- `abi`: ABI identifier (lowercase registered value)
- `[data-model]`: Optional data model specification in brackets
  - `data-model`: Data model identifier within brackets
  - When present, constrains pointer widths, alignment, and object size limits
  - When absent (or empty), default data model applies for the arch/vendor/os combination
- `+`: Separator character (literal plus sign)
- `feature-set`: Sorted, lowercase feature names, comma-free (no commas in the string; features are just concatenated)
  - Each feature is a registered identifier
  - Features are sorted lexicographically
  - Duplicate features are rejected (error, not deduplicated)
  - Feature set must be compatible with the baseline for the target (TARGET-0004)

**Example valid target identities**:
- `x86_64-pc-linux-gnu+sse2,avx2` — x86_64 architecture, PC vendor, Linux GNU environment, with SSE2 and AVX2 features sorted alphabetically
- `riscv64-unknown-none-elf` — RISC-V architecture, unknown vendor, bare-metal environment, no ELF-specific features (empty feature-set after the `+`)
- `aarch64-apple-darwin+crypto` — ARM64 architecture, Apple vendor, Darwin environment, with crypto feature

**Example invalid target identities**:
- `X86_64-pc-linux-gnu+avx2,sse2` — uppercase `X86_64` violates "lowercase registered components" rule
- `x86_64-pc-linux-gnu+sse2,avx2,` — trailing comma after last feature (not allowed)
- `x86_64-pc-linux-gnu+AVX2,sse2` — uppercase features violate "lowercase registered" rule

### ## 5.2 Target Features (continued)

#### ## 5.2.1 TARGET-0002: Alias Resolution (detailed)
- **Mechanism**: Aliases resolve through the signed target registry to exactly one canonical identity
- **Registry**: `registry/target-registry.json` (filename-sorted walk, generated `target/` excluded)
- **Resolution process**:
  1. Present alias to the target registry
  2. Registry returns exactly one canonical identity
  3. Alias cannot depend on the host machine (host machine identity is not in the registry)
  4. If the alias resolves to multiple candidates, it's an error (fail closed)
  5. If the alias resolves to zero candidates, it's an error (fail closed)
  6. If the alias is not in the registry at all, it's an error (fail closed)
- **Host machine independence**: The canonical identity must be computable without reference to the host machine's name, IP address, or any other host-specific attribute
- **Enforcement**: The compiler must reject any target specification that cannot be resolved through the registry
- **Example**: `pc` as vendor alias resolves to `pc` in the registry; `linux` as OS alias resolves to `linux-gnu` or `linux-musl` depending on the registry entry

#### ## 5.2.2 TARGET-0003: Target Manifest Declarations (detailed)
- **Byte order**: Declared as `little-endian` or `big-endian`; must match the architecture's natural byte order or explicitly cross-endian
- **Scalar widths/alignments**: Each scalar type's width (in bits) and alignment (in bytes) must be declared
  - `int`: width, alignment
  - `float`: width, alignment  
  - `pointer`: width, alignment (may differ from general scalars)
  - `bool`: width, alignment
- **Pointer widths per address space**: Different address spaces may have different pointer widths; all must be declared
- **Maximum object size**: Maximum size (in bytes) of a single allocatable object; attempts to allocate larger objects fail
- **Stack alignment**: Stack alignment requirement (in bytes); the stack must be kept aligned to this boundary
- **Atomic widths/lock-freedom**: 
  - Minimum atomic width (in bits) supported
  - Lock-free atomic operations up to what width
  - Compare-and-swap (CAS) support widths
- **Vector model**: SIMD/vector extension model (e.g., `none`, `sse2`, `avx2`, `neon`, `vsx`)
  - Declared even if `none`
  - Affects available instruction set
- **Object format**: Executable/linker object format (e.g., `elf`, `pe`, `mach-o`, `raw` for bare metal)
- **ABI supplement**: Additional ABI details not covered by the base ABI (e.g., `hardfloat`, `softfloat` for floating-point)
- **Unwind/debug formats**: 
  - Unwind table format (e.g., `dwarf`, `ci`, `symbolic`)
  - Debug info format (e.g., `dwarf`, `stabs`, `none`)
- **Feature detection method**: How the target detects and reports feature availability at runtime
  - `cpuid`: Use `cpuid` instruction (x86)
  - `hypervisor`: Query hypervisor (virtualized environments)
  - `firmware`: Query firmware (embedded systems)
  - `none`: Features are compile-time only, no runtime detection

#### ## 5.2.3 TARGET-0004: Baseline Feature Sets (detailed)
- **Explicit baseline**: The baseline feature set is explicitly declared in the target manifest
- **Distribution restriction**: A distribution artifact cannot execute an instruction outside its baseline except within a correctly guarded multiversioned body
- **Multiversioned body**: A body of code that conditionally uses different instruction sets based on detected features
  - Guarded by feature detection at runtime
  - Or compiled with feature-conditioned bodies (multiversioned)
- **Baseline compliance validation**:
  1. The target manifest declares the baseline feature set
  2. At compile time, the compiler checks that all used features are within the baseline
  3. At runtime (if feature detection is supported), the dispatched body matches the detected features within the baseline
  4. Features outside the baseline are rejected unless within a multiversioned body with correct guards
- **Example**: If baseline is `+sse2`, then `+avx2` instructions are rejected unless within a multiversioned body with runtime feature check for AVX2

#### ## 5.2.4 TARGET-0005: Runtime Feature Detection (detailed)
- **Trusted mechanism**: Runtime feature detection is trusted only through the OS/firmware mechanism named by the target manifest
- **Unprivileged self-reporting**: Unprivileged code may further narrow (constrict) the authorized feature set but may NOT widen it
- **Widening prohibition**: Even if hardware supports a feature, unprivileged code cannot report it as available if it's outside the authorized baseline
- **Narrowing permission**: Unprivileged code may report that a feature is NOT available even if the mechanism says it is (constriction is safe)
- **Example**: If TARGET-0005 declares feature detection via `cpuid`, and `cpuid` reports AVX-512, but the target manifest's authorized baseline does not include AVX-512, then the compiler must not generate AVX-512 code. If the code generator tries, it's a target feature fault (TARGET-0008).
- **Narrowing example**: If the mechanism reports `sse2` and `avx`, but the baseline only authorizes `sse2`, the code may only use `sse2`; `avx` usage is rejected (but reporting "no avx" is permitted)

#### ## 5.2.5 TARGET-0006: Feature Dispatch (detailed)
- **Selection algorithm**: Feature dispatch chooses a body whose required feature set is a subset of detected and deployment-policy-authorized features
- **Subset check**: `required_features ⊆ detected_features ∩ authorized_features`
- **Caching**: Selection is cached with thread-safe initialization (once per program execution, the dispatch decision is made once and cached)
- **Thread safety**: The cache initialization is thread-safe; concurrent feature checks do not race
- **Cache invalidation**: The cache is per-execution; a new execution (new process) re-evaluates the dispatch
- **Example**: If three bodies are available: body_A requires `sse2`, body_B requires `avx2`, body_C requires `sse2, avx2`. If detected features are `sse2, avx2` and authorized features are `sse2, avx2`, then body_C is dispatched. If authorized features are only `sse2`, then body_A is dispatched (body_B and body_C are unavailable).

#### ## 5.2.6 TARGET-0007: Target-Specific Semantics (detailed)
- **Implementation-defined data layout**: Data layout may differ between targets but is explicitly declared; core expression meaning remains unchanged
- **ABI/platform facilities**: Platform-specific facilities (MMIO registers, hypervisor calls, etc.) are declared and their semantics are defined
- **Conditional instructions**: Instructions that are conditional (e.g., ARM's conditional execution) have defined semantics within the target model- **Classified hardware faults**: Hardware faults (cache errors, TLB misses, etc.) are classified and their observable behavior is defined
- **Core expression meaning**: The meaning of core expressions (arithmetic, control flow, function calls, etc.) remains unchanged across targets; only data layout, ABI facilities, conditional instructions, and hardware fault handling change
- **Example**: `a + b` always means addition; the result is the same mathematical addition. The representation of `a` and `b` in memory may differ (different alignment, different byte order), but the *meaning* of the addition operation is unchanged.

#### ## 5.2.7 TARGET-0008: Unsupported Target Features (detailed)
- **Translation rejection**: Unsupported target features produce translation rejection before the body is entered
- **TargetFeatureFault**: A specific fault type (`TargetFeatureFault`) is produced when an unsupported feature is encountered
- **Not an acceptable dispatch mechanism**: Illegal instruction traps are not an acceptable dispatch mechanism in safe hosted code
- **Rejection timing**: Rejection occurs at compile time (before execution) or at load time, not at runtime as a trap
- **Example**: If targeting `riscv64-unknown-none-elf` (no FPU) and the program uses `floating-point` instructions, the compiler produces a `TargetFeatureFault` at compile time, not at runtime
- **Safe hosted code**: In "safe hosted" mode (has an OS, memory protection, etc.), illegal instruction traps that could corrupt state are prohibited; instead, the compiler must reject the program

#### ## 5.2.8 TARGET-0009: Address Spaces (detailed)
- **Registration**: Address spaces are registered with the following attributes:
  - `width`: Width of the address space (in bits); e.g., 32 for 32-bit, 64 for 64-bit
  - `representation`: How addresses are represented (e.g., `flat`, `segmented`, `paged`)
  - `accessibility`: Who can access this address space (e.g., `kernel-only`, `user-mode`, `supervisor`)
  - `coherence`: Coherence policy (e.g., `coherent`, `non-coherent`, `cache-coherent`)
  - `provenance recovery`: How provenance is recovered for allocations in this address space (e.g., `full`, `limited`, `none`)
  - `conversion_rules`: Rules for converting pointers between this address space and others
- **Example address space registrations**:
  - `host`: width=64, representation=flat, accessibility=user-mode, coherence=cache-coherent, provenance_recovery=full
  - `kernel`: width=64, representation=flat, accessibility=kernel-only, coherence=coherent, provenance_recovery=full
  - `device`: width=32, representation=memory-mapped, accessibility=device-driver, coherence=non-coherent, provenance_recovery=limited
  - `bare-metal`: width=32, representation=flat, accessibility=supervisor, coherence=non-coherent, provenance_recovery=none

### ## 5.3 Distribution Restrictions (detailed)
- **Baseline enforcement**: The distribution artifact's baseline feature set is enforced at load time
- **Feature check at load**: The loader checks that all used features are within the distribution artifact's baseline
- **Multiversioned body exception**: A distribution artifact may use instructions outside its baseline if:
  - The body is multiversioned (multiple versions for different feature sets)
  - The correct version is selected at runtime based on detected features
  - The runtime feature detection is authorized by the target manifest (TARGET-0005)
- **Loader enforcement**: The load-time linker/enforcer checks baseline compliance; does not rely on the compiler alone
- **Example**: A distribution artifact built with baseline `+sse2` may at runtime dispatch to an AVX2-optimized body only if:
  - Runtime feature detection authorized AVX2 (TARGET-0005)
  - The multiversioned body has both SSE2 and AVX2 versions
  - The dispatch mechanism correctly selects the AVX2 version when AVX2 is detected and authorized

---
## ## 6. Formal Semantics and Verification

### ## 6.1 Aurora Core Logic (ACL) — Defined Meta-Theory (detailed)

#### ## 6.1.1 ACL Signature (formal)
The Aurora Core Logic (ACL) is defined by the following signature:

**Syntax**:
```
Types T ::= 
  Unit                  ()              -- unit type
  Bool                  bool             -- boolean
  IntN                  int<N>           -- N-bit signed integer (N ∈ {8,16,32,64,128})
  UintN                 uint<N>          -- N-bit unsigned integer (N ∈ {8,16,32,64,128})
  FloatN                float<N>         -- N-bit IEEE floating-point (N ∈ {16,32,64,128})
  Char                  char             -- Unicode scalar value (32-bit)
  Byte                  byte             -- 8-bit unsigned octet
  String                string           -- UTF-8 encoded text buffer
  Never                 never            -- uninhabited type
  GenericParam          γ              -- type variable
  Tuple                 ⟨T₁, …, Tₙ⟩      -- heterogeneous product
  Array                 T × ℕ           -- homogeneous array with length
  Range                 {T | lower ≤ i ≤ upper}  -- bounded interval
  Fn                    (Πα. T → U)    -- function type with effect/capability context
  Cap                   Cap(T)           -- capability-restricted value
  Linear                Linear(T)        -- linear ownership
  Affine                Affine(T)        -- affine ownership
  Effect                ε              -- effect row
  Capability            κ              -- capability type

Judgments:
  Γ ⊢ e ⇐ T     checking: e has type T
  Γ ⊢ e ⇒ T     synthesis: e infers type T
  Γ ⊢ v ↓ T       value v has type T
  Γ ⊢ ε₁ ∼ ε₂     effect row equivalence
  Γ ⊢ κ₁ ≈ κ₂     capability equivalence
```

**Key defined forms** (informal readout of the judgment forms):

1. **Unit type**: `()` is the unit type; only value is `()`
2. **Boolean**: `true`, `false` are the boolean values
3. **Integers**: `int<N>` is N-bit two's complement signed; `uint<N>` is N-bit unsigned
4. **Floats**: `float<N>` is IEEE 754-compliant N-bit floating-point
5. **Character**: `char` is a 32-bit Unicode scalar value
6. **Byte**: `byte` is alias of `u8` (8-bit unsigned)
7. **String**: UTF-8 encoded text buffer; operations validated as UTF-8
8. **Never**: Uninhabited; coerces to any type; diverging expressions have type `Never`
9. **GenericParam**: Type variables declared on functions/structs/etc.
10. **Tuple**: Heterogeneous fixed-size product; `⟨⟩` is the 0-tuple (synonym for `()`)
11. **Array**: `T × ℕ` — homogeneous, length `ℕ` known at compile time
12. **Range**: Bounded interval type; e.g., `{int | 0 ≤ i ≤ 10}`
13. **Fn**: Function type with effect/capability context `α`
14. **Cap**: Capability-restricted value; `Cap(T)` value has type `T` plus capability restrictions
15. **Linear**: `Linear(T)` — must be used exactly once; tracked through usage context
16. **Affine**: `Affine(T)` — may be used at most once; may be unused (discarded at scope exit)
17. **Effect**: Effect row `ε` — tracks algebraic effects
18. **Capability**: Capability type `κ` — tracks authority

#### ## 6.1.2 Key Meta-Theoretic Properties (formal statements)

##### ## 6.1.2.1 No Undefined Behavior (formal)
- **Theorem**: For all well-typed closed terms `v : T`, the evaluation of `v` terminates in a value `v'` or a defined error observation; there is no "undefined behavior" observation.
- **Corollary**: Every code path has a defined observation (TERM-0005 formalized: erroneous program SHALL NOT produce a conforming release artifact)
- **Proof strategy**: 
  1. Progress: well-typed term is either a value or can take an evaluation step
  2. Preservation: evaluation preserves well-typedness (or produces a defined error)
  3. Error definitions: all error cases are explicitly defined in the semantics (no "heisenbugs")

##### ## 6.1.2.2 Effect Row Compatibility (formal)
- **Rule**: `Γ ⊢ e1 : T1 / ε1` ∥ `Γ ⊢ e2 : T2 / ε2` ⇒ `Γ ⊢ e1 ⊕ e2 : T1 ⊕ T2 / ε1 ∪ ε2`
- **Where**: 
  - `e1 ⊕ e2` is the combination of two effectful computations
  - `T1 ⊕ T2` is the combined result type (defined by the specific operation; e.g., for addition, `T1 ⊕ T2` might be the numeric sum type)
  - `ε1 ∪ ε2` is the union of the two effect rows (see 6.3.4)
- **Effect row union** (6.3.4 detail): `ε1 ∪ ε2` combines effects; if both have `io`, the union has `io` (once); if one has `io` and the other has `log`, the union has `io, log`

##### ## 6.1.2.3 Linear Usage (formal)
- **Rule**: `Γ, x : Linear T ⊢ e : T'` with use-exactly-once discipline
- **Coverage**: 
  - **Branches**: In `if e then e1 else e2`, exactly one branch must consume the linear value (not both, not neither)
  - **Loops**: In `while e do body`, the linear value must be consumed within the loop body (or in a break/continue path)
  - **Moves**: `move(x)` consumes `x`; after `move(x)`, `x` is invalidated
  - **Scope exit**: At scope exit, if a linear value has not been consumed, it is an error (not silently discarded)
  - **Failure paths**: In the presence of panics/failures, the linear usage tracking must account for all paths
- **Linear typing judgment**: `Γ ⊢ use(x) ; e : T'` where `x` is consumed exactly once in the sequence

##### ## 6.1.2.4 Effect vs Capability Distinction (formal)
- **Effect judgment**: `Γ ⊢ e : T / ε` — "expression `e` has type `T` and effect row `ε`"
- **Capability judgment**: `Γ ⊢ v : Cap(K)` — "value `v` has capability `K`"
- **Distinction**: 
  - Effects track **what** the computation does (side-channels, observable actions)
  - Capabilities track **authority** (what the value is permitted to access)
- **Example**: `print("hello") / io` has effect `io` (I/O observable) but may have capability `Cap(filesystem_read)` separately
- **Formal separation**: The typing rules distinguish effect rows from capability types; no rule derives an effect from a capability or vice versa

#### ## 6.1.2.5 Soundness Properties (formal with assumptions)

**Progress** (formal):
- If `∅ ⊢ e : T` (closed, well-typed expression), then either:
  - `e` is a value, or
  - `e → e'` for some `e'` (can take an evaluation step), or
  - `e` is a defined error observation (e.g., `DivisionByZero`, `NullDereference`)

**Preservation** (formal):
- If `Γ ⊢ e : T` and `e → e'` (evaluates to e'), then `Γ ⊢ e' : T` (well-typedness preserved) or `e'` is a defined error of type `T`

**Effect Soundness** (formal):
- If `Γ ⊢ e : T / ε` then the observable side-channels of `e` are precisely those described by `ε`
- If `ε` is empty (pure), then `e` has no observable side-channels

**Capability Soundness** (formal):
- If `Γ ⊢ v : Cap(K)` then `v` has exactly the authorities granted by capability `K` and no others
- Capability confinement: A value of type `Cap(K)` cannot be coerced to `Cap(K')` for `K' ≠ K` without an explicit capability transfer operation (which itself is tracked)

### ## 6.2 Verification Model (detailed)

#### ## 6.2.1 Stage-0 (Verified Subset) (detailed)

##### ## 6.2.1.1 STAGE-0-0001: Strict Subset Inclusion (formal)
- **Statement**: Stage-0 is a strict syntactic and semantic subset of Aurora
- **Proof obligation**: Prove that every Stage-0 source program has identical observations under the full Aurora semantics
- **Method**: 
  1. Define the Stage-0 subset syntax (disabling named productions)
  2. Define the Stage-0 semantic mapping (same as Aurora but with restricted domain)
  3. Prove inclusion: for all Stage-0 programs `p`, `Aurora(p) = Stage0(p)`
  4. Prove strictness: there exists at least one Aurora program that is not in Stage-0

##### ## 6.2.1.2 STAGE-0-0002: Stage-0 Content (formal)
- **Included**:
  - UTF-8/ASCII identifiers (valid Unicode identifier syntax)
  - Modules (package/module structure)
  - Structs/enums (nominal product/union types with variant tags)
  - Fixed integers (`int8` through `int128`, `uint8` through `uint128`)
  - `bool` (true/false)
  - `byte` (alias of `u8`)
  - `arrays`/`slices` (fixed-length arrays, slice types behind pointer)
  - `functions` (function definitions and calls, without specialization)
  - `generics` without specialization (polymorphic function bodies without instantiation)
  - `Result`/`Option` types and their constructors
  - `ownership`/`borrows` (affine/linear type tracking at Stage-0 level)
  - `explicit effects` (effects declared but possibly restricted in Stage-0)
  - `unsafe raw memory` (limited unsafe usage within Stage-0)
  - `freestanding/native code generation` (target-specific code without OS dependencies)

- **Excluded**:
  - Macros (macro expansion, meta-programming)
  - Runtime reflection (introspection of own type/value at runtime)
  - `async` (asynchronous function execution)
  - `managed domains` (garbage-collected memory regions)
  - `dynamic typing` (type determination at runtime)
  - `relations` (propositional or first-order relations as types)
  - `decimal` (decimal literal types, as distinct from fixed integers)
  - `scalable vectors/tensors` ( dynamically-sized array types)
  - `profile libraries` (language-level profile import mechanisms)
  - `user-defined effect handlers` (dynamic effect registration/resumption)

##### ## 6.2.1.3 STAGE-0-0004: Grammar Generation (formal)
- **Statement**: Stage-0 grammar is generated by disabling named Edition 1 productions, not maintained as an unrelated grammar
- **Formal requirement**: 
  - Let `G_E1` be the Edition 1 grammar
  - Let `P` be the set of named productions in `G_E1`
  - The Stage-0 grammar `G_S0` is defined as `G_E1 \ {p | p ∈ P ∧ p is Stage-0-excluded}`
  - `G_S0` is generated mechanically from `G_E1`; human maintenance of `G_S0` is forbidden (any manual changes must be reflected in the mechanical generation)
- **Verification**: The mechanical generation procedure is verified against `G_E1`; the output `G_S0` is the unique grammar that disables exactly the Stage-0-excluded productions

##### ## 6.2.1.4 STAGE-0-0005: Stage-0 Arithmetic and Policy (formal)
- **Checked arithmetic**: Same checked arithmetic as full Aurora (overflow panics, underflow panics, defined wrap behavior with explicit annotation)
- **Evaluation order**: Same evaluation order as full Aurora (left-to-right for arguments, defined order for operators)
- **Initialization/drop**: Same initialization and drop semantics as full Aurora (but restricted to Stage-0 types)
- **Panic-abort policy**: Same panic-abort policy as full Aurora (diverging expressions go to `Never`, but within Stage-0, panic may have defined behavior)
- **Provenance**: Same provenance tracking as full Aurora (but restricted to Stage-0 value origins)
- **C/omni_v1 ABI subset**: Same C/omni_v1 ABI subset as full Aurora (but Stage-0 may restrict which ABI features are available)

##### ## 6.2.1.5 STAGE-0-0006: Seed Compiler (formal)
- **Statement**: Seed compiler consumes canonical Stage-0 source and emits one reference object format/ISA plus deterministic diagnostic records
- **Formal requirement**: 
  - Input: canonical Stage-0 source program `p` (content-addressed, verified as Stage-0)
  - Output: reference object format for target ISA + deterministic diagnostic records `D`
  - Determinism: Same input `p` always produces same output `D` (byte-for-byte)
  - Reference format: one of the defined target object formats (elf, pe, raw, etc. per TARGET-0009)
  - Diagnostics: deterministic records of type-checking, ownership checking, and other verification outputs

##### ## 6.2.1.7 STAGE-0-0007: Feature Predicate Testing (formal)
- **Statement**: Every Stage-0 restriction is represented by a feature predicate and tested against the Edition 1 parser/model to prove subset inclusion
- **Formal requirement**: 
  - For each Stage-0 exclusion `E_i`, there is a feature predicate `P_i` such that:
    - `P_i` is true iff the program does NOT use the excluded feature
    - `P_i` is tested against the Edition 1 parser/model
    - The test proves that if `P_i` is true, the program is valid under Stage-0 semantics
  - All feature predicates conjoin (AND) to characterize the full Stage-0 subset
  - The conjunction of all `P_i` is necessary and sufficient for Stage-0 inclusion

#### ## 6.2.2 Diverse Double Compilation (DDC) (detailed)

##### ## 6.2.2.1 BOOT-0004: DDC Procedure (formal)
- **Procedure**:
  1. Take the same canonical compiler source `S`
  2. Build compiler `B` from the trusted chain (seed → Stage-0 → full compiler via trusted tools)
  3. Build diverse compiler `D` from an independently implemented diverse compiler/toolchain (different compiler frontend, different backend, different linker, different OS)
  4. Use `B` to compile canonical Omni sources: `B(CanOmni) → output_B`
  5. Use `D` to compile canonical Omni sources: `D(CanOmni) → output_D`
  6. Compare `output_B` and `output_D`:
     - **Byte-for-byte convergence**: `output_B ≡ output_D` (identical bytes)
     - **Or declared semantic equivalence**: `output_B ≃ output_D` with signed evidence of semantic equivalence (not just byte-for-byte)
  7. If neither convergence nor equivalence is established, DDC mismatch blocks release

##### ## 6.2.2.2 BOOT-0005: Diverse Compiler Qualification (formal)
- **Qualification criteria**:
  - **Implementation lineage**: The diverse compiler's implementation lineage (history of changes, known bugs, design decisions) must be documented
  - **Language/tool dependencies**: All language and tool dependencies of the diverse compiler must be listed and declared as independent
  - **Backend/link path**: The backend (code generator, assembler, linker) must be sufficiently independent from the primary compiler's backend
  - **Build environment**: The build environment (OS, libc, system tools) must be documented and declared as independent
- **Documentation**: All of the above must be documented in the TCB inventory (BOOT-0008)

##### ## 6.2.2.3 BOOT-0006: DDC Mismatch Consequences (formal)
- **Mismatch blocks release**: If `output_B ≢ output_D` (neither byte-for-byte convergence nor declared semantic equivalence), the DDC mismatch blocks the release
- **Preserves all artifacts/logs**: All build artifacts, logs, and intermediate files are preserved (not deleted)
- **Triggers differential localization**: The mismatch triggers differential localization analysis (comparing the two builds to localize the difference)
- **Never waived**: DDC mismatch is **never** waived merely because one binary passes tests
- **Failure mode**: Release is blocked; the build system exits with a structured error; a new release cycle must be initiated after the discrepancy is resolved

#### ## 6.2.2.4 DDC Comparison Equivalence Criteria (formal)
- **Byte-for-byte convergence**: `output_B` and `output_D` are identical at the byte level; this is the strongest criterion but may be too strict for some output types (e.g., timestamps in debug info)
- **Declared semantic equivalence**: `output_B` and `output_D` are declared to be semantically equivalent with signed evidence; this allows for output differences that do not affect program semantics (e.g., different register allocation, different debug info format)
- **Equivalence evidence**: Signed statement from the diverse compiler team asserting semantic equivalence; the statement must be verifiable (e.g., via hash-chained proof)

#### ## 6.2.3 Proof Obligations per Release (detailed)

##### ## 6.2.3.1 Stage-0 Verification
- **Obligation**: For every release, Stage-0 feature predicates must be proved against the Edition 1 parser/model
- **Procedure**: 
  1. List all Stage-0 feature predicates (from STAGE-0-0007)
  2. For each predicate `P_i`, run the Edition 1 parser/model on Stage-0 programs
  3. Verify that the program is accepted and produces defined observations
  4. Record the verification result for each predicate
  5. If any predicate fails, the release is blocked until the predicate is resolved

##### ## 6.2.3.2 DDC
- **Obligation**: Same source rebuilt with trusted chain + diverse compiler; outputs compared (see 6.2.2)
- **Procedure**: As described in 6.2.2.1

##### ## 6.2.3.3 Provenance
- **Obligation**: Every artifact includes `compiler_id`, `compiler_version`, `source_revision` (`-dirty` convention), `codegen` fields
- **Procedure**: 
  1. `compiler_id`: Identifier of the compiler that produced the artifact (must match the TCB inventory)
  2. `compiler_version`: Version of the compiler (four-component form `R.M.m.P`)
  3. `source_revision`: Content hash of the source code; `-dirty` suffix if the working tree has uncommitted changes
  4. `codegen`: Code generation profile/flags used; enables reproducibility verification

##### ## 6.2.3.4 Conformance Claim
- **Obligation**: `CONF-0001` through `CONF-0011` structure; assurance level, auditor, evidence, waivers, expiration, revocation status
- **Procedure**: 
  1. Fill out the conformance claim record with all required fields
  2. Have the claim reviewed (self-attested, independently audited, or authority-certified)
  3. Publish the conformance claim with the release artifact
  4. The claim is part of the artifact's provenance (UPDATE-0001 through UPDATE-0008)

##### ## 6.2.3.5 Build Reproducibility
- **Obligation**: Same inputs produce same outputs (verified via BUILD-0001 through BUILD-0011)
- **Procedure**: 
  1. Run the build twice with identical inputs
  2. Compare outputs (content hash comparison)
  3. If outputs differ, investigate and fix (may indicate a bug in the build pipeline or non-determinism)
  4. Document the reproducibility verification result

##### ## 6.2.3.6 Spec Digest Match
- **Obligation**: Computed `spec_tree_sha256` matches recorded digest in release manifest
- **Procedure**: 
  1. Compute `spec_tree_sha256` using the canonicalization procedure (Section 2.2)
  2. Compare with the digest recorded in the release manifest
  3. If they differ, the release is blocked (specification has changed without proper procedure)
  4. If this is an intentional change, follow the errata/amendment process (Section 1.2)

#### ## 6.2.4 Verification Toolchain Requirements (detailed)

##### ## 6.2.4.1 Proof Architecture
- **Requirement**: A defined framework for discharging Stage-0 feature predicates (not merely asserted)
- **Components**:
  - A formal statement of each Stage-0 feature predicate (as in 6.2.1.7)
  - A mechanical proof procedure (or proof assistant script) that discharges each predicate
  - A verification condition generator that translates the predicate into proof obligations
  - A proof assistant (or tactic script) that solves the obligations
  - A reflection mechanism that lifts the proof result from the assistant back to the specification

##### ## 6.2.4.2 Trusted Computing Base
- **Explicit list of trusted components**:
  1. Parser (Edition 1 grammar parser)
  2. Solver (type inference/unification engine)
  3. Type checker (well-formedness checking)
  4. Code generator (for the target ISA being verified)
  5. Optimizer passes (each optimizing pass that alters the program)
  6. Linker (for the target object format)
  7. Assembler (for the target assembly syntax)
  8. Runtime system (if any; for freestanding targets, this is empty)
  9. Verification toolchain itself (proof assistant, tactic scripts)

##### ## 6.2.4.3 Equivalence Criterion
- **Exact byte-for-byte convergence** OR **declared semantic equivalence with signed evidence**
- **Procedure for byte-for-byte**: 
  1. Build with primary compiler → output O1
  2. Build with diverse compiler → output O2
  3. Compare O1 and O2 byte-by-byte
  4. If identical, convergence is established
- **Procedure for semantic equivalence**: 
  1. Build with primary compiler → output O1
  2. Build with diverse compiler → output O2
  3. Define the semantic equivalence relation (e.g., "same observable behavior for all defined inputs")
  4. Produce signed evidence (from the diverse compiler team) asserting the equivalence
  5. The evidence must be machine-verified (or at least auditor-verified)

##### ## 6.2.4.4 Qualification Boundary
- **Which components must be independently verified vs. can rely on conventional testing**:
  - **Must be independently verified** (formal proofs, DDC):
    - Code generator for each target ISA
    - Ownership/affine type checker
    - Effect tracking checker
    - Stage-0 subset verifier
  - **Can rely on conventional testing** (test suites, fuzzing):
    - Syntax/lexical analysis (but still must be regression-tested)
    - Code generation for well-established targets
    - Optimization passes (but must have test coverage)
  - **Hybrid** (both formal and testing):
    - Arithmetic operations (formal proof of overflow handling + fuzzing for edge cases)
    - Concurrency (formal model checking + differential testing)

##### ## 6.2.4.5 Verification Tool Validation
- **Procedures for validating the verification tools themselves**:
  1. **Regression against known-good programs**: Run the verification tool on a suite of programs where the correct answer is known (from prior verification or formal proof); the tool must produce the correct answer
  2. **Differential testing with independent tool**: Run the verification tool and an independent verification tool on the same suite; compare results (must agree on well-typedness, error detection, etc.)
  3. **Tool micro-benchmarking**: Verify the tool itself is not introducing non-determinism or bugs
  4. **Toolchain bootstrapping**: The verification tool must be verifiable (i.e., the tool's correctness can be checked by a simpler tool, recursively down to a trusted base)

### ## 6.3 Effect System (detailed)

#### ## 6.3.1 Effect Rows as Types (detailed)

##### ## 6.3.1.1 Effect Row Syntax
- **Effect row `ε`**: `ε ::= ε₁, ε₂ | ∅ | ε, ε_new`
  - Comma-separated list of effect identifiers
  - `∅` (empty row): no effects
  - `ε, ε_new`: append `ε_new` to `ε` (union, not substitution)

##### ## 6.3.1.2 Effect Row in Type Signatures
- **Function type**: `fn(P₁, …, Pₙ) → T / ε`
  - `Pᵢ` are parameter types
  - `T` is the return type
  - `ε` is the effect row (effects this function performs)
- **Example**: `fn(FILE*) → int / io, close` — this function performs `io` and `close` effects

##### ## 6.3.1.3 Effect Row Variables and Quantification
- **Effect row variables**: Effect row variables may appear in type signatures (e.g., `α` as a row variable)
- **Quantification**: `∀ε. fn() → T / ε` — "for all effect rows, this function has type T with effect ε"
- **Default**: If no effect row is annotated, the default is `∅` (pure function)

#### ## 6.3.2 Standard Effects (detailed)

##### ## 6.3.2.1 Effect ID Registry
The following effect IDs are registered and their meaning is fixed:

| ID | Meaning | Restrictions |
|---|---|---|
| `io` | Input/output operations | May be restricted per profile (`safe` profile prohibits unrestricted `io`; `distributed` profile channels all `io` through typed channels) |
| `dynamic` | Dynamic dispatch, reflection, runtime type analysis | Heavily restricted in `high-assurance` profile; prohibited in `real-time` profile without WCET annotation |
| `foreign` | FFI calls, external library invocations | Requires `unsafe` context (UNSAFE-0003); in `safe` profile, must go through vetted FFI wrappers |
| `block` | Blocking operations (synchronous I/O in async context) | Requires `block` effect handler; in `real-time` profile, must have bounded blocking time |
| `log` | Logging operations | May write to stderr; in `high-assurance` profile, logging must be auditable and have bounded volume |
| `unwind` | Exception/unwind propagation | In `safe` profile, unwind must be to declared exception types only; in `high-assurance`, full unwind proof required |
| `dynamic` (runtime code gen) | Runtime code generation or modification | Prohibited in `safe` and `high-assurance` profiles; allowed in `general-purpose` with restrictions |

##### ## 6.3.2.2 Effect ID Conflicts
- If two effect IDs have overlapping meaning, the union takes the more restrictive interpretation
- Example: If a program has both `io` and `foreign io`, the effective effect is the more restrictive of the two (likely `foreign io` with its additional constraints)

#### ## 6.3.3 Effect Capability Environment (detailed)

##### ## 6.3.3.1 Implicit Capability Environment `K`
- **Every function has**: An implicit capability environment `K` that tracks what authorities the function may exercise
- **Formal**: The function type is actually `fn(P) → T / ε ⟨K⟩` where `⟨K⟩` is the capability environment
- **Separation from effects**: The semantics clearly distinguish effect requirements from capability passing and authority (as stated in the design review feedback)

##### ## 6.3.3.2 Capability Environment Subtyping
- `K₁ ≤ K₂` means "capability environment `K₁` is a subset of `K₂`" (i.e., `K₁` grants fewer authorities than `K₂`)
- Function subtyping: `fn(P) → T / ε ⟨K₁⟩` is a subtype of `fn(P) → T / ε ⟨K₂⟩` when `K₁ ≤ K₂` (more restrictive function is subtype of more permissive)
- This enables effect/capability polymorphism while preserving soundness

#### ## 6.3.4 Effect Handling (detailed)

##### ## 6.3.4.1 Effect Row Union
- **Union operation**: `ε1 ∪ ε2` 
- **Union rules**:
  - If both rows have the same effect ID, the union has that effect ID once (deduplication)
  - If rows have different effect IDs, the union has both (concatenation in sorted order)
  - `ε ∪ ∅ = ε` (identity)
  - `∅ ∪ ∅ = ∅`

##### ## 6.3.4.2 Structured Task Scopes and Effects
- **Children effects join into scope row**: When a structured task scope completes, the children's effects are joined into the scope's effect row
- **Quorum(k) policy**: Commits once `k` successes exist; the resulting effect row is the union of the effects of the `k` committed successes
- **Cancellation**: Effect operations can be cancelled before commit; after commit, the committed result is returned and cannot be rolled back (the effect has been committed)

##### ## 6.3.4.3 Effect Exhaustion
- **Bounded channels**: Have capacity `N`; send commits when an element enters the buffer or is handed directly to a receiver
- **Effect exhaustion**: If an effect operation would exceed some budget (e.g., too many `io` operations), the operation is rejected with a defined error

### ## 6.4 Ownership and Memory Model (detailed)

#### ## 6.4.1 Type Categories (expanded from the generated file)

##### ## 6.4.1.1 Primitive Scalar Types (detail)
- `int<N>`: N-bit two's complement signed integer; `N ∈ {8, 16, 32, 64, 128}`
  - `int8`: range -128 to 127
  - `int16`: range -32768 to 32767
  - `int32`: range about ±2.1 billion
  - `int64`: range about ±9.2 quintillion
  - `int128`: arbitrary-precision range (implementation-defined maximum)
- `uint<N>`: N-bit unsigned integer; `N ∈ {8, 16, 32, 64, 128}`
  - `uint8`: range 0 to 255 (alias `byte`)
  - `uint16`: range 0 to 65535
  - `uint32`: range about 4.3 billion
  - `uint64`: range about 18.4 quintillion
  - `uint128`: arbitrary-precision range (implementation-defined maximum)
- `float<N>`: IEEE 754 N-bit floating-point; `N ∈ {16, 32, 64, 128}`
  - `float16`: half-precision
  - `float32`: single-precision
  - `float64`: double-precision
  - `float128`: quad-precision (implementation-defined)
- `bool`: two values `true`, `false`
- `char`: 32-bit Unicode scalar value (can represent any Unicode code point)
- `byte`: alias of `u8`; 8-bit unsigned octet; specifically for raw data, not Unicode characters
- `string`: UTF-8 encoded text buffer; operations validate as UTF-8; invalid UTF-8 in source is a syntax error (SRC-0002)
- `unit`: 0-tuple `()`; only value is `()`; used as return type for functions with no meaningful return
- `never`: uninhabited type; no values; used for:
  - Diverging functions (never returns)
  - Error types that can never be observed (the type itself is unconstructible)
  - Coercion target: `never → T` for any `T`

##### ## 6.4.1.2 Generic Parameters (detail)
- `GenericParam(String)`: Syntactic type variable declared on items (e.g., `T` in `fn<T>(x: T) → T`)
- `Infer(u32)`: Fresh unification inference variable (`TyVar`) during solver resolution
  - `u32` is a unique index for the inference variable
  - Multiple `Infer` variables may be minted during constraint solving
  - Unification assigns concrete types to `Infer` variables

##### ## 6.4.1.3 Compound Types (detail)
- `Tuple(Vec<Ty>)`: Heterogeneous fixed-size product type
  - `⟨⟩` is the 0-tuple (synonym for `unit`)
  - `⟨int, bool⟩` is a tuple of `int` and `bool`
  - Tuples are nominal (identity by name/declaration, not structural, unless declared `distinct`)
- `Array(T, usize)`: Homogeneous contiguous array with compile-time known length
  - `Array(int, 10)` is an array of 10 `int`s
  - Length is part of the type; `Array(int, 5)` and `Array(int, 10)` are different types
- `Range(T)`: Bounded half-open or closed sequence interval
  - `{int | 0 ≤ i ≤ 10}` or `{int | 0 ≤ i < 10}`
  - Used for types like "integer in range 0..10"
- `Fn(Vec<Ty>, Ty)`: Function pointer or closure signature
  - `Fn(int, bool) → float` is a function type: takes `int` and `bool`, returns `float`
- `Struct(String, Vec<Ty>)`: Nominal product type with named fields and generic type arguments
  - `Struct("Point", [int, int])` is a struct with two `int` fields, named `x` and `y` (field names are part of the type identity)
  - Nominal: identity by name; `Struct("Point", [int, int])` and `Struct("Rect", [int, int])` are different types even with same field types
- `Effect-tracked types`: Every type carries optional effect row; e.g., `Linear(int) / io` means a linear integer with io effect
- `Capability types`: `Cap(T)` — capability-restricted values; `Cap(filesystem)` means the value has filesystem authority
- `Linear T`: Value must be used exactly once (tracking via usage context)
- `Affine T`: Value may be used at most once; may also be unused (discarded at scope exit)
- `Bottom type`: `Never` — uninhabited, coerces to any type

#### ## 6.4.2 Type Identity (detailed)

##### ## 6.4.2.1 Nominal Identity
- **Structs**: Identity by struct name and field types/declaration order; `Struct("Point", [int, float])` is a distinct type from `Struct("Vec2", [int, float])`
- **Enums**: Identity by enum name and variant tag type; each variant has a distinct identity
- **Traits**: Identity by trait name and associated type declarations
- **Opaque types**: Identity by opaque type declaration; implementations are hidden but the type identity is exposed
- **Capabilities**: Identity by capability name/ID; `Cap(int)` is different from `Cap(float)` even if `int` and `float` have same bit width
- **Effects**: Identity by effect row; `fn() → int / io` is a different type from `fn() → int / io, log` (different effect row)

##### ## 6.4.2.2 Structural Identity
- **Tuples**: Structural identity — `⟨int, bool⟩` is structurally equal to another `⟨int, bool⟩` (regardless of name; tuples don't have names in most contexts)
- **Function types**: Structural — `fn(int) → bool` is structurally equal to another `fn(int) → bool`
- **Raw pointers**: Structural — `*const int` is structurally equal to another `*const int`
- **Arrays**: Structural — `Array(int, 5)` is structurally equal to another `Array(int, 5)` (length matters)
- **Slices**: Structural — `&[int]` is structurally equal to another `&[int]`
- **Generic instantiations**: Structural — `identity[int]` is structurally equal to `identity[int]` (same instantiation)
- **Transparent type aliases**: Structural — if `type Alias = Int` is a transparent alias, then `Alias` is structurally equal to `Int`

##### ## 6.4.2.3 Identity Rules Summary
- **Nominal vs structural**: The default is nominal for declared types; structural for compound types formed without explicit nominal declaration
- **`distinct` keyword**: Types declared `distinct` are always nominally equal only to themselves; structural equality is irrelevant
- **Aliases**: Transparent aliases preserve the identity of the underlying type; opaque aliases create a new identity
- **Recursive types**: Recursive nominal types require an indirection (reference, pointer) or unsized tail; infinitely sized value types are erroneous

#### ## 6.4.3 Type Inference and Constraint Solving (detailed)

##### ## 6.4.3.1 Bidirectional Type Checking
- **Checking mode**: `Γ ⊢ e ⇐ T` — "under context `Γ`, expression `e` has type `T` (checked)"
  - Used when the expected type is known (e.g., variable declaration with explicit type, function argument matching declared parameter type)
  - The expression must conform to the expected type
- **Synthesis mode**: `Γ ⊢ e ⇒ T` — "under context `Γ`, expression `e` infers type `T` (synthesized)"
  - Used when no expected type is provided (e.g., function body last expression, variable initialization without type annotation)
  - The expression's type is inferred from its structure
- **Default**: If no expected type is provided and inference fails, it's a type error
- **Default type defaulting**: Some positions have default types (e.g., `int` for numeric literals without explicit type; `bool` for boolean contexts)

##### ## 6.4.3.2 Union-Find Unification
- **Two type variables `?A` and `?B` unify** by merging their union-find sets
  - Each type variable starts in its own set
  - Unification merges sets; after merging, the two variables are considered the same type
  - The merge operation is idempotent, commutative, and associative
- **A type variable `?A` unifies with concrete type `Ty`** if no conflicting binding exists
  - If `?A` already has a binding (e.g., `?A = int`), and the new attempt is to bind `?A` to `float`, this is a conflict and unification fails
  - If `?A` has no binding, `?A` is assigned `Ty`
- **Incompatible concrete types** (e.g., equating `Int` with `String`) produce `TypeError::MismatchedTypes`
  - This is an unrecoverable error for the current function/item
  - The entire enclosing function/item is marked as having a type error

##### ## 6.4.3.3 Constraint Solving
- **Normalizes aliases and associated types**: During solving, type aliases are expanded and associated type constraints are resolved
- **Applies coherence-selected implementations**: When there are multiple viable implementations for a function/type, the coherence rule selects one (see 7.1 for rule registry coherence)
- **Terminates under edition solver restrictions**: The solver must terminate; recursive type equations are solved under edition-specific restrictions (e.g., maximum recursion depth, occurs-check)
- **Solver output**: A substitution environment `SubstEnv` mapping inference variables to concrete types; or a `TypeError` if solving fails

#### ## 6.4.4 Linear and Affine Types (detailed)

##### ## 6.4.4.1 Linear `Linear T` (detailed)
- **Usage tracking**: The linear type system tracks usage of `Linear T` values through a usage context that covers:
  - **Branches**: In `if e then e1 else e2`, exactly one branch must consume the linear value
    - If both branches consume it: error (double use)
    - If neither branch consumes it: error (use at scope exit)
  - **Loops**: In `while e do body`, the linear value must be consumed within the loop body
    - If the loop body is never entered (condition initially false): the linear value must be consumed after the loop (or it's an error)
  - **Moves**: `move(x)` consumes `x`; after `move(x)`, `x` is invalidated (any use of `x` after `move(x)` is an error)
  - **Scope exit**: At scope exit, if a `Linear T` value has not been consumed, it is an error (not silently discarded)
  - **Failure paths**: In the presence of panics/failures, the linear usage tracking must account for all paths; a linear value must be consumed on all paths, or a handler must be provided

- **Formal usage judgment**: `Γ ⊢ use(x) ; e : T'` where `x` is consumed exactly once in the sequence `use(x) ; e`
- **Linear in function parameters**: `fn(x: Linear T) → U` — the caller must pass a `Linear T` value; the function takes ownership

##### ## 6.4.4.2 Affine `Affine T` (detailed)
- **Usage tracking**: The affine type system tracks usage of `Affine T` values through a usage context that covers:
  - **Branches**: In `if e then e1 else e2`, zero or one branch may consume the affine value (not both)
  - **Loops**: In `while e do body`, the affine value may be consumed within the loop body (at most once)
  - **Scope exit**: At scope exit, if an `Affine T` value has not been consumed, it is **discarded** (not an error; this is the "affine" relaxation of linear)
  - **Moves**: `move(x)` consumes `x`; after `move(x)`, `x` is invalidated (same as linear)
  - **Nested scopes**: Inner scope exit discards affine values; outer scope may still use them

- **Formal usage judgment**: `Γ ⊢ use(x) ; e : T'` where `x` is consumed at most once in the sequence; if not consumed, it's discarded at scope exit

##### ## 6.4.4.3 Move Semantics (detailed)
- **Ownership transfer**: `move(x)` transfers ownership of `x` to the destination; `x` is invalidated (can no longer be used)
- **Automatic moves**: In some contexts (e.g., function return, vector push), moves are automatic; the programmer does not need to write `move()` explicitly
- **Use-after-move**: Using a value after it has been moved is a type error (compile-time detected)
- **Example**:
  ```aurora
  let x = Linear(Vector(10));  // x is Linear(Vector(10))
  let y = move(x);            // y now owns the vector; x is invalidated
  // use(x)  // ERROR: x has been moved
  use(y);   // OK: y owns the vector
  ```

##### ## 6.4.4.4 Borrow Tracking (detailed)
- **`FlowState.reference_loans`**: Maps MIR locals to loan-region identifiers
- **Current gap** (Phase 2 qualification blocker): 
  - `FlowState.reference_loans` cannot yet represent references stored in projected places or aggregates
  - `FlowState.reference_loans` cannot resolve a dereference back to its borrowed origin
  - `Rvalue::Use(Operand::Copy(place))` does not propagate loan associations to the destination
- **Projected places**: When a reference is stored in a projected place (e.g., `struct.field`), the loan tracking must follow the projection path
- **Aggregate storage**: When a reference is stored in an aggregate (e.g., array element, struct field within an array), the loan tracking must account for the aggregate structure
- **Dereference origin**: Given a dereference `*p`, the system must be able to identify which loan (borrow origin) this dereference is associated with

##### ## 6.4.4.5 Borrow Conflict (detailed)
- **Shared references**: Shared references (`&T`) cannot authorize writes, moves, drops, or mutable reborrows
  - Multiple shared references to the same data are permitted (aliasing)
  - But none of them can be used for mutation
- **Mutable references**: Mutable references (`&mut T`) have exclusive access
  - At most one mutable reference to a given piece of data may exist at a time
  - Any shared reference while a mutable reference exists is an error
- **Borrow duration**: Borrows have a lexical/lifetime duration; the borrow checker enforces that borrows do not outlive their scope
- **Variance**: Borrow variance rules determine subtyping relationships between reference types (e.g., `&'a T` is covariant in `'a` for shared references, invariant for mutable)

#### ## 6.4.5 `unsafe` Discipline (detailed)

##### ## 6.4.5.1 UNSAFE-0001: `unsafe` does not disable ordinary checks
- **Formal**: `unsafe` permits performing operations with programmer-proved preconditions; **it does not** disable ordinary typing, ownership, effect, capability, initialization, or control-flow checks
- **Implementation**: The compiler retains all ordinary checks; `unsafe` only grants permission to perform operations that the type system would otherwise reject (subject to precondition proof)
- **Audit**: The unsafe effect is visible in MIR and audit reports; every unsafe operation is recorded

##### ## ## 6.4.5.2 UNSAFE-0002: Unsafe Operations Location
- **Legal only inside**: `unsafe` block/function or through an unsafe trait implementation
- **Block form**: `unsafe { ... }` — all operations within the block are under unsafe permission
- **Function form**: `unsafe fn ...` — the entire function is unsafe
- **Unsafe trait implementation**: `impl unsafe Trait for Type { ... }` — the implementation is under unsafe permission
- **Outside these contexts**: All ordinary checks apply; attempting unsafely-permitted operations is a compile-time error

##### ## 6.4.5.3 UNSAFE-0003: Calling Unsafe Functions
- **Requires unsafe context**: Calling an unsafe function requires an unsafe context even when the caller can prove its preconditions
- **Safe wrappers**: Safe wrappers discharge and document the proof
  - `safe_fn(x)` calls `unsafe_fn(x)` after verifying preconditions; the wrapper is checked to ensure it correctly validates preconditions
  - The wrapper's correctness is verified; if the wrapper is wrong, the unsoundness is in the wrapper, not in the original `unsafe` use

##### ## 6.4.5.4 UNSAFE-0004: Named Obligation Records
- **Each unsafe operation has**: A named obligation record with:
  - `precondition`: The programmer-proved precondition that makes the operation valid
  - `protected_invariant`: The invariant that the operation protects/preserves
  - `permitted_optimizer_assumptions`: What the optimizer may assume (e.g., "no aliasing of this pointer")
  - `failure_containment_class`: How failures from this operation are contained (e.g., "panic and abort," "error return," "undefined behavior is confined to this operation")
- **Example obligation record**:
  ```aurora
  obligation UNSAFE-0004-operation-42 {
    precondition: "x > 0 && x < 100",
    protected_invariant: "memory safety of region R",
    permitted_optimizer_assumptions: "no aliasing of pointer p",
    failure_containment_class: "panic with error code E001"
  }
  ```

##### ## 6.4.5.4 UNSAFE-0005: Invalid Unsafe Execution
- **If precondition is false**: If an unsafe precondition is false at event `e`, that execution becomes invalid from `e` onward
- **Observations before `e`**: Observations before `e` remain required (the program's observations up to the point of invalidation are still valid)
- **Other executions**: Other executions and paths not reaching `e` retain full semantics (the invalidation is local to the path reaching `e`)
- **Example**: If `precondition: "p points to allocated memory"` and at some point `p` is null, the execution from that point is invalid; but earlier execution paths where `p` was valid retain their full semantics

##### ## 6.4.5.5 UNSAFE-0006: Optimizer Assumptions
- **Optimizer may assume**: An unsafe precondition only on paths that reach the operation and only while the facts/objects on which it depends remain live
- **Assumption provenance**: The optimizer's assumptions must point to the obligation (the named obligation record from UNSAFE-0004)
- **Example**: If the precondition is `"no aliasing of pointer p"`, the optimizer may assume no aliasing of `p` on the paths reaching the unsafe operation, but must not assume it globally or on paths where the obligation doesn't apply

##### ## 6.4.5.6 UNSAFE-0007: Invalid Unsafe Execution Consequences
- **Does not authorize**: Compile-time compromise, cross-process effects, or modification of immutable artifacts
- **Outcomes outside**: Such outcomes are outside the language model and addressed by hardening (separate mechanism)
- **Containment**: Invalid unsafe execution is contained; it does not cascade to unrelated parts of the program or to other processes

#### ## 6.4.6 Memory Safety Invariants (detailed)

##### ## 6.4.6.1 No Invalid Bit Patterns (formal)
- **Theorem**: Valid scalar bit patterns cannot be constructed or observed in safe code (`TYPE-0004`)
- **Proof**: The type system and compiler enforce that only valid bit patterns are constructible; `int` values have valid bit patterns for their declared width; `float` values have valid IEEE 754 bit patterns; `bool` has exactly `true` and `false` bit patterns
- **Enforcement**: Run-time checks (where necessary) and compile-time checks prevent construction of invalid bit patterns

##### ## 6.4.6.2 Reference Validity (formal)
- **Theorem**: References are non-null, aligned, live, and provenance-valid (`TYPE-0005`)
- **Non-null**: A reference value is never a null pointer (null is not a valid reference; use `Option<&T>` if nullability is needed)
- **Aligned**: A reference's address is aligned to the referenced type's alignment requirement
- **Live**: A reference points to a live (not deallocated, not moved-from) object
- **Provenance-valid**: A reference's provenance can be traced back to an allocation; no "stray" references from deallocated memory

##### ## 6.4.6.3 Enum Invariants (formal)
- **Theorem**: Enums have a declared live variant (`TYPE-0006`)
- **Variant declaration**: Each enum variant has a declared "live" variant; at runtime, the enum value must have one of the live variants active
- **Zero-sized types**: Zero-sized types occupy no logical bytes but have distinct ownership/drop events
  - `Array(zero_sized_type, N)` retains length and iteration count (the length is tracked even though the elements have zero size)
  - Drop events for zero-sized types are still tracked (even though there's nothing to "drop" in the traditional sense)

##### ## 6.4.6.4 Never Coercion (formal)
- **Theorem**: `Never` is uninhabited and coerces to any type (`TYPE-0007`)
- **No values**: There is no value of type `Never`
- **Coercion rule**: `Never → T` for any type `T`; this is the only way to construct a `T` from `Never` (normally used for diverging functions: `fn() → Never` and then `never → T`)
- **Diverging expressions**: Expressions that never terminate (infinite loop, panics unconditionally) have type `Never`

##### ## 6.4.6.5 Unsized Types (formal)
- **Theorem**: Unsized types are `[T]`, `str`, `dyn Trait`, and declared extern types; may occur only behind a pointer/reference/owner or as the final field of a declared dynamically sized aggregate (`TYPE-0008`)
- **`[T]`**: Unsized slice type; length is not known at compile time; must be behind a pointer
- **`str`**: Unsized UTF-8 string type; length is not known at compile time; must be behind a pointer
- **`dyn Trait`**: Dynamic trait object type; vtable-supported polymorphism; must be behind a pointer
- **Declared extern types**: Types declared as extern (from other languages); must follow the same rules
- **Pointer/owner requirement**: Unsized types may only appear:
  - As the pointed-to type: `&[T]`, `&str`, `&dyn Trait`
  - As the owned type with implicit length: not allowed (unsized types cannot be owned by value)
  - As the final field of a declared dynamically sized aggregate: `struct { a: int, b: [T] }` — the `[T]` must be the final field

### ## 6.5 Concurrency Model (detailed)

#### ## 6.5.1 Execution Agents (detailed)

##### ## 6.5.1.1 CONC-0001: Execution Agent Definition (formal)
- **Definition**: Native threads, scoped tasks, async tasks, actors, and interrupts are distinct execution agents with profile-defined capabilities and memory participation
- **Profile-defined**: The specific capabilities and memory participation for each agent type are defined by the active assurance profile
- **Memory participation**: Each agent type has a defined role in memory access (e.g., native threads may share heap; actors have message-passing-only memory participation)

##### ## 6.5.1.2 CONC-0002: Thread/Task Creation (formal)
- **Formal**: Thread/task creation publishes all moved arguments and initialized captured state to the child start; child completion synchronizes with successful join
- **Publish**: Before the child task/thread starts, all moved arguments and initially captured state are "published" (made visible to the child)
- **Synchronization**: Child completion synchronizes with a successful join (the parent can `join()` and get the result without blocking indefinitely if the child has already completed)

##### ## 6.5.1.3 CONC-0003: Structured Task Scope Completion (formal)
- **Formal**: A structured task scope cannot complete until every child is joined, cancelled and joined, or transferred to an authorized long-lived supervisor
- **Completion precondition**: All children must have reached a terminal state (completed, cancelled, or joined) before the scope can complete
- **Cancellation and join**: Children may be explicitly cancelled; after cancellation, they must be joined to retrieve their final state
- **Supervisor transfer**: Orphaned children may be transferred to an authorized long-lived supervisor; the scope cannot complete until all children are accounted for

##### ## 6.5.1.4 CONC-0004: Scope Policies (formal)
- **Standard scope policies**:
  - `all`: Waits for all children and returns ordered results by spawn order
  - `cancel_on_error`: Commits the first error/panic by completion order, requests cancellation of unfinished siblings, joins them, and returns the committed cause with suppressed causes
  - `first_success`: Returns the first committed success, cancels and joins others; if none succeeds, aggregates terminal causes by completion order
  - `race`: Returns the first committed terminal outcome
  - `supervise`: Transfers orphans to a long-lived supervisor; scope completes when all non-transferred children are done
  - `quorum(k)`: Commits once `k` successes exist; returns them in completion order; cancels/joins remaining children; impossibility to reach `k` returns aggregated failure

##### ## 6.5.1.5 CONC-0005: `all` Policy (formal)
- **Formal**: `all` waits for all children and returns ordered results by spawn order
- **Behavior**: 
  - Spawn child 1, child 2, ..., child N
  - Wait for all N to complete
  - Return results in the order: child 1's result, child 2's result, ..., child N's result
  - If any child faults, the behavior depends on the profile (may error, may collect faults, may cancel others)

##### ## 6.5.1.6 CONC-0006: `cancel_on_error` Policy (formal)
- **Formal**: Commits first error/panic by completion order, requests cancellation of unfinished siblings, joins them, and returns the committed cause with suppressed causes
- **Behavior**:
  - As children complete, the first error/panic to commit is recorded as the cause
  - Unfinished siblings are requested to cancel (they may finish their current operation before cancelling)
  - All unfinished siblings are joined
  - The committed cause is returned; any other causes are suppressed (not included in the return value)

##### ## 6.5.1.7 CONC-0007: `first_success` Policy (formal)
- **Formal**: Returns the first committed success, cancels and joins others; if none succeeds, aggregates terminal causes by completion order
- **Behavior**:
  - As children complete, the first one to succeed commits its success
  - All other children are cancelled (request to stop their current operation)
  - All other children are joined (to retrieve their final state, even though they didn't succeed)
  - If no child succeeds, the terminal causes (errors, faults) from all children are aggregated by completion order and returned

##### ## 6.5.1.8 CONC-0008: `race` Policy (formal)
- **Formal**: Returns the first committed terminal outcome
- **Behavior**: 
  - Children compete to commit first
  - The first child to commit (success or fault) determines the outcome
  - Other children are cancelled (request to stop)
  - Other children are joined (to retrieve state)
  - The outcome is: the first committed terminal result (could be success or any terminal fault)

#### ## 6.5.2 Concurrency Detailed Components

##### ## 6.5.2.1 CONC-0009: Bounded Channels (detailed)
- **Capacity N**: Bounded channels have capacity N (declared at channel creation)
- **Send commitment**: Send commits when an element enters the buffer or is handed directly to a receiver (hand-off, not buffer-insert)
- **Receive commitment**: Receive commits when ownership of one element transfers to the receiver
- **Buffer semantics**: The buffer holds up to N elements; if the buffer is full, send blocks (or is rejected, depending on profile) until space is available
- **FIFO ordering**: Channel ordering is FIFO per sender and globally FIFO by send commit order for a single channel

##### ## 6.5.2.2 CONC-0010: Channel Close (detailed)
- **Close prevents new sends**: Closing a channel prevents new successful sends
- **Buffered elements remain**: Buffered elements remain receivable after close
- **Buffer exhaustion**: After the buffer empties, receive returns `Closed` (a special sentinel value)
- **Concurrent close/send**: Concurrent close and send is decided by the operation's atomic commit order; the committed operation wins

##### ## 6.5.2.3 CONC-0011: Channel Ordering (detailed)
- **FIFO per sender**: Messages from a single sender are delivered in FIFO order (the order they were sent)
- **Global FIFO by send commit order**: For a single channel with multiple senders, the global order is by send commit order (the order in which commits happened, not the order in which sends were initiated)
- **Fairness**: Fairness is not guaranteed unless the channel type/profile states it (some profiles may enforce fairness; others may not)

##### ## 6.5.2.4 CONC-0012: Cancellation (detailed)
- **Before commit**: Cancellation before a channel operation commits leaves no element transferred (the operation is undone)
- **After commit**: Cancellation after commit returns/records the committed result and cannot roll it back (the commitment is atomic)

##### ## 6.5.2.5 CONC-0013: Mutexes (detailed)
- **Standard mutexes**: May be unfair (no guarantee of FIFO ordering); may wake spuriously only through condition-variable waits, not lock success
- **Fair mutexes**: Separate type from standard mutexes; provide fairness guarantees (FIFO ordering of waiters)

##### ## 6.5.2.6 CONC-0014: Condition Variables (detailed)
- **Wait atomically**: Condition-variable wait atomically releases the mutex and blocks
- **Spurious wakeup**: May wake spuriously; callers must test predicates in a loop (even if the predicate appears true, re-check)
- **Formal**: Callers must test predicates in a loop (while `!predicate()` do wait())

##### ## 6.5.2.7 CONC-0015: Semaphores (detailed)
- **Commit atomically**: Semaphores commit permit decrement/increment atomically
- **Cancellation**: Cancellation before acquire commit consumes no permit (if cancelled before the commit, the permit is not decremented)
- **Example**: `semaphore.acquire()` commits the permit decrement; if cancelled before commit, the permit count is unchanged

##### ## 6.5.2.8 CONC-0016: Mutex Poisoning (detailed)
- **Standard mutexes poison**: After panic exits a held critical section, the mutex is "poisoned"
- **Poison is advisory**: Poison is advisory and recoverable through an explicit API; it is not memory unsafety
- **Recovery API**: Explicit API to check and recover from poison (e.g., `mutex.recover_from_poison()`)
- **Example**: If a thread panics while holding a mutex, the mutex becomes poisoned; other threads can recover; but the fact of the panic is recorded, not treated as memory corruption

##### ## 6.5.2.9 CONC-0017: Executor Polling (detailed)
- **No concurrent polling**: An executor SHALL not poll one task concurrently with itself
- **Duplicate/coalesced wakes**: SHALL tolerate duplicate/coalesced wakes (a task may be woken multiple times for the same event)
- **No polling after terminal completion**: SHALL not poll after terminal completion (the task has finished; polling is a no-error, but has no effect)

##### ## 6.5.2.10 CONC-0018: Future Polling (detailed)
- **Non-reentrant**: Polling a future is non-reentrant unless the future explicitly implements a reentrant protocol
- **Wake scheduling**: A wake during poll schedules a later poll and does not recursively enter the task
- **Formal**: If future `f` is being polled, and a wake event for `f` occurs during poll, the wake schedules a later poll of `f`; it does not enter `f`'s computation recursively

##### ## 6.5.2.11 CONC-0019: Blocking in Async (detailed)
- **Block effect required**: Blocking operations in an async context require the `block` effect and a provider that routes them to a blocking resource or rejects them
- **Profile restrictions**: In `real-time` profile, blocking must have bounded time; in `safe` profile, blocking may be general but is discouraged in favor of async

##### ## 6.5.2.12 CONC-0020: Actor Mailboxes (detailed)
- **FIFO per sender**: Actor mailboxes preserve message commit order per sender (messages from the same sender are delivered in the order they were committed)
- **Cross-sender order**: Cross-sender order is by mailbox commit (the order in which messages from different senders were committed to the mailbox)
- **Restart never rolls back**: A restart never rolls back external effects already committed (side effects are permanent)

##### ## 6.5.2.13 CONC-0021: Region Transfer (detailed)
- **Formal**: Region transfer in a message commits ownership atomically with enqueue. On enqueue failure ownership remains with the sender; on committed delivery failure the supervisor policy owns disposal/retry
- **Atomic commit**: Ownership transfer and message enqueue are atomic; either both happen or neither happens
- **Enqueue failure**: If enqueue fails (receiver not ready, buffer full, etc.), ownership remains with the sender (the message is not lost)
- **Committed delivery failure**: If delivery is committed but later fails (e.g., receiver crashes), the supervisor policy owns disposal/retry (the supervisor decides how to handle the failed delivery)

##### ## 6.5.2.14 CONC-0022: Thread-Local Values (detailed)
- **Per native thread**: Thread-local values initialize per native thread
- **Inaccessible from migrated tasks**: Inaccessible from tasks migrated between threads unless task-local storage is used
- **Drop at thread exit**: Thread-local values drop at orderly thread exit only (not at scope exit, at thread exit only)

##### ## 6.5.2.15 CONC-0023: Signals/Interrupts (detailed)
- **Async-signal-safe only**: Signals/interrupts may access only async-signal/interrupt-safe operations declared by the target/profile
- **Prohibited operations**: Ordinary locks, allocation, and destructors are prohibited unless explicitly supported by the profile
- **Example**: In a POSIX-like environment, only async-signal-safe functions (from `signal.h`) may be called within a signal handler; malloc, free, file I/O, etc. are prohibited

---
## ## 7. Rule Registry and Ownership (detailed)

### ## 7.1 Rule Registry Structure (detailed)

#### ## 7.1.1 Rule Organization
- **`registry/rules.json`**: The master rule registry; contains all rule definitions
- **`registry/rule-texts.json`**: Schema for rule texts; each rule text has a standard structure
- **Rule structure** (each entry in `rules.json`):
  ```json
  {
    "rule_id": "ERR3-0010",
    "text": "The complete text of the rule",
    "schema_version": "1.0.0",
    "lifecycle_state": "Ratified"
  }
  ```

#### ## 7.1.2 Lifecycle States (detailed)
- **`Proposed`**: New rule under consideration; not yet ratified
  - May be modified based on community feedback
  - May be superseded by a newer proposal
  - No implementation claims may reference `Proposed` rules in `strict` mode
- **`Candidate`**: Under active review; may become Ratified
  - More mature than `Proposed`; closer to ratification
  - Implementation claims may reference `Candidate` rules (with some restrictions)
- **`Ratified`**: Officially accepted; rule is in force
  - The rule is part of the normative specification
  - Implementation claims must reference `Ratified` rules (in `strict` mode)
  - `omni-audit` verifies every `Ratified` rule is both claimed and witnessed
- **`Deprecated`**: Retired but retained for backward compatibility
  - Rule is still valid for existing artifacts
  - New artifacts should not use `Deprecated` rules
  - `Deprecated` rules must still be witnessed if they are `Ratified`-class (but witness requirements may be relaxed)
- **`Superseded`**: Replaced by newer rule; old rule remains valid for existing artifacts
  - The new rule replaces the old one for new code
  - Old code using the superseded rule remains valid (backward compatibility)
  - `Superseded` rules are not counted in the "coverage" statistic (per-rule coverage, not per-file)
- **`Withdrawn`**: Removed entirely; not valid for any new artifact
  - The rule is removed from `rules.json` (or marked `Withdrawn`)
  - No new artifacts may reference `Withdrawn` rules
  - Existing artifacts using `Withdrawn` rules are "grandfathered" but cannot be extended

#### ## 7.1.3 Coverage Statistic
- **Per-rule, not per-file**: Coverage is counted per-rule, not per-file
- **Untagged infrastructure**: Untagged infrastructure/derived/oracle code is permitted (does not need rule IDs)
- **Duplicate claims**: Duplicate claims across files are allowed and attributed (collaborative implementation)
- **Live state metric**: "8 tags in 57 files (74 test functions), clean" — this is the current coverage metric

### ## 7.2 Linkage Reaper (`omni-audit`) (detailed)

#### ## 7.2.1 Claim Resolution Algorithm
- **Input**: `#[implements("ID")]` or `implements!("ID")` claims found under `compiler/` and `tools/` (generated `target/` excluded, filename-sorted walk)
- **Step 1**: For each claim, extract the rule ID `ID`
- **Step 2**: Look up `ID` in `registry/rules.json`
- **Step 3**: Check the rule's `lifecycle_state`:
  - If `Ratified` or `Candidate`: claim is valid (subject to witness requirements)
  - If `Proposed`: claim is valid only in non-strict mode; in strict mode, `Proposed` rules cause rejection
  - If `Deprecated`/`Superseded`/`Withdrawn`/`unknown`: claim **fails closed** (rejection)
- **Step 4**: If the rule is `Ratified`, check witness requirements (see 7.2.2)
- **Step 5**: Record the claim resolution result (valid/invalid, with reason)

#### ## 7.2.2 Witness Requirements (detailed)
- **Every `Ratified` rule must be both claimed and witnessed**
- **Claim**: Some file in the workspace has `#[implements("ID")]` or `implements!("ID")` claiming this rule
- **Witness**: Some file in the workspace has a collected `#[test]` name that witnesses the rule (i.e., the test exercises the rule's behavior)
- **Per-rule coverage**: Coverage is per-rule, not per-file; a single file may claim multiple rules, and a single rule may be claimed from multiple files
- **Witness collection**: `#[test]` names are collected across the workspace; each `#[test]` may witness one or more rules
- **Current live state**: 8 tags in 57 files (74 test functions), clean — meaning 8 `Ratified` rules have both claims and witnesses; the remaining rules are at various lifecycle stages

#### ## 7.2.3 Failure Cases (detailed)
- **`Proposed`/`Deprecated`/`Superseded`/`Withdrawn` ownership**: All fail closed (claim rejection)
- **Unknown IDs**: If the rule ID is not in `registry/rules.json`, the claim fails closed
- **Malformed claim syntax**: If the claim syntax is not `#[implements("ID")]` or `implements!("ID")`, it fails closed
- **Unparseable sources**: If a source file cannot be parsed, the claims within it fail closed
- **Symlinks**: Symbolic links are excluded from the walk (or cause the claim to fail closed, depending on configuration)
- **Missing trees**: If the `compiler/` or `tools/` directory does not exist, all claims fail closed
- **Dependency cycles**: If there are dependency cycles in the rule claims (circular `implements` references), the claims fail closed
- **Duplicate registry IDs**: If two rules in the registry have the same ID (duplicate), claims using that ID fail closed

### ## 7.3 Rule Lifecycle Management (detailed)

#### ## 7.3.1 Promotion: `Proposed` → `Candidate`
- **Process**: A `Proposed` rule may be promoted to `Candidate` after active review
- **Review criteria**: 
  - The rule has been discussed on the public forum/mailing list
  - At least one implementation exists that follows the rule
  - The rule text is stable (no further major modifications expected)
- **Documentation**: The promotion must be documented (release notes, rule registry update)

#### ## 7.3.2 Ratification: `Candidate` → `Ratified`
- **Process**: A `Candidate` rule may be ratified after sufficient testing and review
- **Requirements**: 
  - At least one independent implementation demonstrates compliance with the rule
  - A test suite exists that exercises the rule (positive and negative cases)
  - The rule text is final (no further modifications)
  - The rule has been reviewed by the architecture review board (or equivalent)
- **Witness requirement**: At least one witness (`#[test]`) must exist for the rule (this is what the "8 tags in 57 files" metric counts)
- **Documentation**: The ratification must be documented (release notes, rule registry update, witness registration)

#### ## 7.3.3 Deprecation
- **Process**: A `Ratified` rule may be deprecated if it's being replaced or is causing problems
- **Requirements**: 
  - A replacement rule must exist (or the functionality is being removed)
  - The deprecation must be documented (why is the rule being deprecated?)
  - Existing artifacts using the rule remain valid (backward compatibility)
- **Backward compatibility**: `Deprecated` rules are still valid for existing artifacts; new artifacts should use the replacement

#### ## 7.3.4 Supersession
- **Process**: A `Ratified` rule may be superseded by a newer rule
- **Requirements**: 
  - The new rule must replace the old one's functionality
  - The supersession must be documented (which old rule is replaced by which new rule)
  - Old code using the superseded rule remains valid (backward compatibility)
- **Example**: RULE-0003 `erratum-corrected` has no enum spelling (needs spec ADR) — this is a tracked gap that may lead to supersession once resolved

#### ## 7.3.5 Withdrawal
- **Process**: A rule may be withdrawn entirely
- **Requirements**: 
  - The functionality the rule governed is being removed from the language
  - All affected artifacts are identified and either updated or deprecated
  - The withdrawal must be documented extensively
- **After withdrawal**: The rule is removed from `rules.json` (or marked `Withdrawn`); no new artifacts may reference it; existing artifacts are "grandfathered"

### ## 7.4 Conformance and Rule State (detailed)

#### ## 7.4.1 Implementation Claim Constraints
- **An implementation claim names**: The edition, modules, profiles, targets, limits, external ABI/data versions, implementation-defined choices, deviations, and test-result digest
- **Strict mode**: In strict mode, only `Ratified` and `Candidate` rules may be referenced; `Proposed`/`Deprecated`/`Superseded`/`Withdrawn` cause rejection
- **Namespaced extensions**: May be tested only in separate claims (not mixed with strict-mode claims)
- **Partial implementations**: Partial implementations cannot claim complete conformity; they must declare which claim type they hold (parser-only, core-subset, profile, etc.)

#### ## 7.4.2 Strict Mode Details
- **Activation**: Strict mode is declared in the implementation claim (CONF-0007); once declared, applies to all conformance cases in that claim
- **Effect**: Disables extension syntax/semantics; rejects artifacts whose unrecognized extension could affect behavior
- **Rule referencing**: Only `Ratified` and `Candidate` rules may be used as the basis for conformance; `Proposed` rules cause the conformance case to be invalid
- **Namespaced extensions**: Even namespaced extensions may be tested only in separate claims outside strict mode

---
## ## 8. Priority Issues and Action Items (detailed from design review)

### ## 8.1 Normative Authority and Semantic Consistency **(Priority 1)**

#### ## 8.1.1 EBNF/VIBE-GRAM Conflict Resolution
- **The core conflict**: Normative EBNF (`spec/grammar/omni-edition1.ebnf`) predates Candidate-2 amendment; lacks pipeline-operator, newline-termination, projection-shorthand, command-call productions; VIBE-GRAM rules (VIBE-GRAM-0001 through VIBE-GRAM-0007) normatively require these productions
- **Resolution options evaluated**:
  - **Automatic EBNF modernization**: Rejected — would invalidate SHA-256 digest; would require new edition
  - **Maintain EBNF as-is with VIBE-GRAM separate**: Rejected — creates authority conflict; which is normative?
  - **Grammar milestone with new edition**: Accepted as future work; not yet started
- **Current resolution**: The tree binds the artifact as-is (EBNF as-is); VIBE-GRAM productions are normatively required but the EBNF does not contain them; this is a known gap- **Documentation requirement**: Any resolution must document the precedence decision and update affected rule IDs
- **Affected rule IDs**: Any production changes require new/updated rule IDs in `registry/rules.json`; `spec_tree_sha256` will change, triggering a new release cycle
- **Update procedure** (if/when grammar is modernized):
  1. Document the decision and rationale in a new ADR (Architecture Decision Record)
  2. Update the EBNF or add a modernization supplement
  3. Assign new or updated rule IDs in `registry/rules.json`
  4. Update `spec_tree_sha256` (the digest will change)
  5. Publish errata for all affected rule IDs
  6. Inform all in-flight conformance claims of the change
  7. The current EBNF stands as the artifact binding until this process is executed

#### ## 8.1.2 Specification Digest Authority
- **Precise definition**: What goes into `spec_tree_sha256` exclusion set must be precisely defined (not left informal)
- **Current shortcoming**: The note that "omni-canon has not started and therefore every digest is unsound" must be established from actual code/specification, not accepted solely from the note
- **Action**: Define the exact exclusion procedure (Section 2.2.3); validate against the build pipeline; ensure conformance claims are only made when digest integrity is confirmed
- **Verification gap**: Until `omni-canon` is started and validated, digest claims should be qualified: "digest integrity not yet independently verified by `omni-canon`"

### ## 8.2 Reference and Ownership Soundness **(Priority 2)**

#### ## 8.2.1 `FlowState.reference_loans` Gap Investigation
- **The gap**: `FlowState.reference_loans` maps MIR locals to loan-region identifiers; currently cannot:
  - Represent references stored in projected places or aggregates
  - Resolve a dereference back to its borrowed origin
- **Specific failures**:
  - `Rvalue::Use(Operand::Copy(place))` does not propagate loan associations to the destination
  - Projected places (e.g., `struct.field`) lose the loan association through the projection
  - Aggregate storage (e.g., array elements, struct fields within arrays) loses loan tracking
- **Phase 2 qualification blocker**: This gap is tracked as a Phase 2 qualification blocker; must be resolved before release
- **Three-option resolution** (from 8.2 in the original review):
  1. **Shared references are copyable**, with shared loan ownership tracked across every live alias; mutable references remain affine (simplest model but may be too permissive)
  2. **All references are affine and move-only**, giving the simplest provenance model but restricting shared reference reuse (rigid but sound)
  3. **Copyability determined by explicit normative trait/rule**, with reference-specific constraints stated in the specification (most flexible but requires new normative machinery)
- **Resolution status**: Not yet resolved; the three options are under evaluation; the decision must be made before reference copy propagation can be implemented
- **Action items**:
  - Design the chosen option's semantics
  - Implement the tracking mechanism in `FlowState`
  - Update the ownership verifier to handle the new model
  - Write regressions covering the new boundaries
  - Resolve the authority/precedence mismatch before treating any interpretation as normative

#### ## 8.1.3 Reference Copyability Decision
- **Decision required**: Among the three options listed above
- **Criteria for decision**:
  - **Model simplicity**: How complex is the provenance tracking?
  - **Expressivity**: How much programming pattern does the model support?
  - **Verification feasibility**: Can the model be formally verified?
  - **Backward compatibility**: Does the model break existing code?
  - **Assurance profile compatibility**: Does the model work with all five assurance profiles?
- **Timeline**: Must be resolved before Phase 2 qualification; do not implement reference copy propagation until the ownership/borrow rules are sound

### ## 8.3 Canonicalization and Evidence Integrity **(Priority 3)**

#### ## 8.3.1 Specification Digest Domain Precision
- **Explicit definition**: The exclusion set for `spec_tree_sha256` must be explicitly defined in normative terms
- **Procedure steps** (formalized):
  1. Walk included directories in lexicographic order of slash-joined relative paths
  2. Normalize file bytes CRLF/CR → LF (each file independently)
  3. Hash each file with SHA-256 (producing 64-char lowercase hex)
  4. Hash concatenation of `path + LF + hex + LF` lines (all such lines concatenated)
  5. Final SHA-256 of the entire concatenated byte string is `spec_tree_sha256`
- **Current computed digest**: `7e63431f3c624f9818454f076c38f5cde3d517ace9e919926d6188bc177be769` (supersedes earlier values by REL-0007 machine-block formatting)
- **Earlier digest**: `07762061…` (by gate-closure content, superseded; recovered stage0 rules/texts, erratum, widened patterns — not by contradiction)
- **`omni-canon` status**: Not yet started — this must be established from actual implementation before digest integrity can be claimed
- **`omni-conform` role**: When `omni-canon` is implemented, `omni-conform` will reject any manifest/gate digest mismatch fail-closed

#### ## 8.3.2 Digest Integrity Verification
- **Current state**: Without `omni-canon`, digest integrity must be established from the actual code and specification
- **Qualified claim**: "The `spec_tree_sha256` digest was computed using the procedure in Section 2.2; however, `omni-canon` has not been started, so independent verification of digest integrity is pending"
- **Future state**: When `omni-canon --spec-tree` is run and validated, the digest can be claimed as independently verified
- **Release blocker**: Until digest integrity is verified, release claims that depend on the digest should be qualified/modified

### ## 8.4 Compilation and Release Evidence **(Priority 4)**

#### ## 8.4.1 DDC Formal Specification
- **Procedure**: Same canonical source rebuilt with trusted chain + diverse compiler
- **Equivalence criterion**: Byte-for-byte canonical output convergence OR declared semantic equivalence with signed evidence
- **TCB (Trusted Computing Base)**: Explicit list of trusted components (parser, solver, type checker, codegen, optimizer passes, linker, assembler, Unicode data, crypto, OS/firmware, signing tools — as listed in BOOT-0008)
- **Disagreement resolution protocol**:
  1. DDC mismatch blocks release (never waived because one binary passes tests)
  2. All artifacts/logs preserved (not deleted)
  3. Differential localization triggered (comparing the two builds to localize the difference)
  4. Root cause analysis initiated; the discrepancy must be resolved before a new release cycle
  5. Release is blocked until the discrepancy is resolved and re-DDC proves convergence/equivalence

#### ## 8.4.2 Provenance Completeness
- **Every artifact must include**: `compiler_id`, `compiler_version`, `source_revision` (`-dirty` convention), `codegen` fields (UPDATE-0001 through UPDATE-0008)
- **Provenance chain**: The metadata chain must be complete from the seed compiler to the final artifact
- **Mismatch detection**: Any gap in the provenance chain is detected and the artifact is rejected
- **Example provenance record**:
  ```json
  {
    "compiler_id": "omni-compiler-v1",
    "compiler_version": "1.0.0.0",
    "source_revision": "a1b2c3d4e5f6... (SHA-256 of source)",
    "source_revision_dirty": true/false,
    "codegen": "cranelift-1.0.0",
    "provenance_chain": ["seed_hash", "stage0_hash", "stage1_hash", "final_hash"]
  }
  ```

#### ## 8.4.3 Release Evidence Workflow
- **Step 1**: Build the artifact
- **Step 2**: Generate provenance (UPDATE-0001 through UPDATE-0008)
- **Step 3**: Compute `spec_tree_sha256` (if `omni-canon` is available; otherwise note as pending)
- **Step 4**: Generate conformance claim (CONF-0001 through CONF-0011)
- **Step 5**: DDC verification (build with trusted chain + diverse compiler; compare outputs)
- **Step 6**: Publish the release artifact with all the above evidence attached
- **Step 7**: If any step fails, the release is blocked; the build is not published

### ## 8.5 Profile Enforcement and Long-term Compatibility **(Priority 5)**

#### ## 8.5.1 Profile Enforcement Mechanism
- **`safe` profile**: No `unsafe`, no `foreign` effect, no raw pointers — enforced via type system and checker; the compiler rejects any program attempting these features
- **Other profiles**: Each profile has its enforcement mechanism:
  - `distributed`: Enforced via channel type system (message passing only)
  - `real-time`: Enforced via timing annotations and WCET proofs
  - `high-assurance`: Enforced via formal verification discharge (Stage-0 + DDC)
  - `general-purpose`: Less enforcement; relies on developer discipline plus static checking
- **Profile switching**: Controlled language-amendment process (Section 1.2); cannot be done implicitly

#### ## 8.5.2 Controlled Language Amendment
- **Process**: Once the underlying semantics are stable, a controlled language-amendment and migration process is designed
- **Steps**:
  1. Propose the amendment (with full semantics specification)
  2. Public review period (minimum 30 days)
  3. Architecture review board evaluation
  4. If approved, the amendment is incorporated as an erratum (with new rule IDs)
  5. Migration path is documented (what code needs to change, how)
  6. The amendment is not implicit; existing code does not automatically adopt the new semantics
- **Backward compatibility**: The amendment process explicitly handles backward compatibility; old code may continue under the old semantics until a migration is performed

#### ## 8.5.4 Profile Tailoring (ECSS-S-ST-00) (detailed)
- **Per ECSS-S-ST-00**: Profiles may be tailored for specific project characteristics and constraints
- **Tailoring process**: Customer specifies tailoring parameters; supplier implements within profile constraints
- **Quality objectives**: Selected per ECSS-Q-ST-80; influence which profile requirements are mandatory vs. optional
- **Security considerations**: Tailoring for security based on selection of requirements related to verification, validation, and levels of proof demanded by software sensitivity
- **Dependability aspects**: Tailoring based on selection of requirements related to verification, validation, and levels of proof demanded by software criticality
- **Development constraints**: Type of software development and target system characteristics taken into account
- **Tailoring restrictions**:
  - No tailoring that removes a "core restriction" from a profile (core restrictions are immutable)
  - No tailoring that adds features prohibited for the profile (e.g., adding `unsafe` to the `safe` profile)
  - All tailoring deviations must be explicitly documented and tested
  - Tailoring conformance claim must be separate from the base profile claim

### ## 8.6 Specification-Digest Domain **(Additional Priority)**

#### ## 8.6.1 Exclusion Set Definition
- **Precisely define** what file sets go into `spec_tree_sha256` exclusion set (as detailed in Section 2.2.3)
- **The exclusion set is the authoritative definition**; the "what's excluded" list is as important as the "what's included" list
- **Must_haves vs._must_not_haves**: Both the included file set (by directory traversal) and the excluded file set must be precisely defined

#### ## 8.6.2 Canonicalization Procedure (re-iteration for emphasis)
- **Procedure** (atomic steps, no deviation):
  1. Walk included directories in lexicographic order of slash-joined relative paths
  2. Normalize file bytes CRLF/CR → LF
  3. Hash each file with SHA-256
  4. Hash concatenation of `path + LF + hex + LF` lines
  5. Final hash is `spec_tree_sha256`
- **No shortcuts**: Each step must be followed; skipping any step invalidates the digest
- **Human vs. machine**: The procedure is machine-executable (via `omni-canon`) but also documentable for human review

#### ## 8.6.3 Digest Discrepancy Resolution
- **If digests differ**: The release is blocked; the specification has changed without proper procedure
- **If the change is intentional**: Follow the errata/amendment process (Section 1.2); the digest will change, and this is recorded as a new release
- **If the change is unintentional**: Investigate the cause; fix the process; the old digest is reinstated if the change was an error

### ## 8.7 Grammar Modernization **(Additional Priority)**

#### ## 8.7.1 Conflict Restatement
- **Normative EBNF**: Predates Candidate-2 amendment; lacks pipeline-operator, newline-termination, projection-shorthand, command-call productions
- **VIBE-GRAM rules**: Normatively require these productions (VIBE-GRAM-0001 through VIBE-GRAM-0007)
- **The gap**: The artifact binds the EBNF as-is; the VIBE-GRAM productions are normative requirements not currently in the EBNF
- **This is a known gap**, not a bug; the grammar milestone is planned but not started

#### ## 8.7.2 Resolution Pathways
- **Path A: Grammar milestone starts** (planned future work)
  - Formally start the grammar modernization effort
  - Produce a new EBNF (or supplement) that includes VIBE-GRAM productions
  - This will change `spec_tree_sha256` and trigger a new release cycle
- **Path B: Maintain EBNF as-is, document VIBE-GRAM separately** (current state)
  - The artifact binds the EBNF as-is
  - VIBE-GRAM productions are acknowledged as normative but not in the EBNF
  - This is a documented gap; implementations may follow one or the other (or both)
- **Path C: New edition (Edition 2)** (long-term)
  - A clean break; Edition 1 remains as-is; Edition 2 is developed with the modernized grammar
  - Existing Edition 1 artifacts remain valid under Edition 1 rules

#### ## 8.7.3 Priority and Timeline
- **Priority**: Medium (below normative authority, ownership soundness, canonicalization, DDC, profile enforcement)
- **Timeline**: Planned for the grammar milestone (after foundation gates 0.0.0.1 through 0.0.0.13 are complete, after `omni-canon` is started, after reference/ownership soundness is resolved)
- **Impact on releases**: Any grammar change will require a new `spec_tree_sha256` and a new release cycle; this is not a blocker for current foundation gate work

### ## 8.8 Feature-Grade Matrix Qualification **(Additional Priority)**

#### ## 8.8.1 Rejection of Feature-Checklist Matrix
- **The feature matrix** (categorizing dependent types, GC, runtimes, concurrency approaches by grade) is **rejected as binding architectural decisions**
- **Reason**: These are useful design prompts but not adequate grounds for global prohibitions; features have trade-offs that depend on the specific context, not the grade label

#### ## 8.8.2 Profile-Based Approach (adopted)
- **Common foundation** with **separate assurance profiles** is the adopted approach (see Section 3.2)
- **Profiles inherit from the common semantic foundation**; profile-specific deviations must be explicit and testable
- **This is better than** globally prohibiting features such as GC or dynamic dispatch merely because a more restrictive profile might not permit them

#### ## 8.8.3 AI Caution: Provenance Collapse
- **Collapsing provenance, data lineage, and information-flow labels** into "every value must carry runtime provenance" should **not** be done unless Omni has a specific semantic reason and accepts the associated cost
- **These are different concepts**:
  - **Provenance**: Where did this value come from? (who allocated it, what operations were performed on it)
  - **Data lineage**: What is the history of this data value? (which operations transformed it)
  - **Information-flow labels**: What can this value flow to? (who can read/modify it)
- **Costs of runtime provenance**: 
  - Increased type/checker complexity
  - Runtime overhead (if provenance is checked at runtime)
  - Compile-time complexity (if provenance is checked at compile time)
  - Potential for false positives/negatives
- **When runtime provenance is justified**: Only if Omni has a specific semantic reason (e.g., the ownership/borrow model requires it) and accepts the associated cost

### ## 8.9 Undefined Behavior Contract **(Additional Priority)**

#### ## 8.9.1 Four-Category Definition (formal)
- **Category 1: Behavior guaranteed by language semantics**
  - Example: `1 + 2` evaluates to `3`; `true && false` evaluates to `false`; a well-typed function call has defined behavior
  - All such behaviors are explicitly specified in the semantics (ACL)
- **Category 2: Explicit, defined program failures**
  - Example: `1 / 0` produces `DivisionByZero` error; out-of-bounds array access produces `IndexOutOfRange` error; null dereference produces `NullPointerException`-like error
  - These are defined failures, not "undefined behavior"; the program faults in a defined way
- **Category 3: Programs rejected during compilation or verification**
  - Example: A program with a type error is rejected at compile time; a program that violates affine usage rules is rejected; a program that violates effect tracking is rejected
  - These programs never execute; they are caught statically
- **Category 4: Platform-dependent behavior permitted by declared target contract**
  - Example: Data layout endianness; pointer widths; atomic widths; OS-specific system calls (via FFI)
  - These are permitted because the target manifest (TARGET-0001 through TARGET-0009) declares them; they are not "undefined" — they are "implementation-defined" with the target contract as the definition
- **Category 5: Hardware faults and external failures the language cannot eliminate**
  - Example: Memory bit flips (radiation); power loss; network disconnection; OS process kill
  - The language cannot prevent these, but can provide mechanisms for bounded recovery (redundancy, transactional memory, etc.)

#### ## 8.9.2 Goal Formalization
- **Each category has specified consequences**: The semantics explicitly describe what happens in each category
- **Invalid programs cannot obtain conforming release status**: A program that falls into Category 1-4 with undefined behavior is rejected; only programs with fully defined behavior can receive conforming release status
- **Category 5 is accepted as inevitable**: Hardware faults and external failures are accepted as beyond the language's control; the language provides mechanisms to mitigate their impact (but cannot eliminate them)

### ## 8.10 Hardware Requirements Classification **(Additional Priority)**

#### ## 8.10.1 Radiation Tolerance Classification
- **Language can provide mechanisms**:
  - Bounded execution (timeout-based failure detection)
  - Explicit failure handling (detected faults have defined error types)
  - Redundancy (multiple versions of critical code; DDC-style checking)
  - Memory-safety checks (provenance tracking, spatial safety)
  - Integration with platform diagnostics (formal integration with OS/firmware fault handlers)
- **Language cannot**:
  - Make physical memory radiation-hardened (that's a hardware feature, not a language feature)
  - Guarantee that hardware faults are detected (some faults may be silent; the language can only provide mechanisms to reduce the probability or detect them through redundancy)
  - Eliminate physical effects (EM leakage, cache timing side-channels) without explicit hardware/software co-design

#### ## 8.10.2 GC Prohibition Classification
- **Too broad as universal requirement**: Prohibiting all GC is too broad; the relevant properties are:
  - Predictable timing (GC pauses must be bounded or absent)
  - Bounded resource use (heap size must be bounded, or GC must operate within budget)
  - Explicit allocation behavior (allocation is explicit, not hidden)
  - Profile-specific restrictions (different profiles have different GC permissions)
- **Profile approach** (adopted):
  - **Critical profile** (e.g., `high-assurance`, `real-time`): Might prohibit managed allocation; use stack-bounded allocation, static data structures, arena allocation with explicit depletion
  - **General-purpose profile**: Supports GC; the `general-purpose` profile explicitly allows GC as a feature
  - **Transition between profiles**: Controlled language-amendment process; migrating from `general-purpose` to a GC-prohibited profile requires refactoring

#### ## 8.10.3 Example Profile Differences
- **`high-assurance` profile**: No GC; all allocation is explicit (stack, arena); deterministic deallocation; no surprise pauses
- **`general-purpose` profile**: GC permitted; the garbage collector runs as a background task; pause times are not guaranteed bounded (but are monitored and documented)
- **`real-time` profile**: No GC; hard real-time constraints; any allocation must have a bounded deallocation time; static analysis proves no unbounded deallocation is possible
- **`distributed` profile**: GC may be present but message passing is the primary state-sharing mechanism; GC-managed objects must beSerializable for message passing

### ## 8.11 Specification Clause Qualification **(Additional Priority)**

#### ## 8.11.1 Five-Fold Distinction Preservation
- **Core principle**: Design requirement, specification clause, implementation, passing test suite, proof of correctness are five different things
- **Application**: Omni's engineering plans and release claims must preserve these distinctions throughout
- **The first document's "already implemented" claims**: Are evidence of what the author believes the specification covers, not proof of implementation state
- **Verification discipline**: When evaluating any part of the specification, ask: "Is this a design requirement, a specification clause, an implementation, a passing test, or a proof?" The answer determines the weight and confidence accorded to the claim

#### ## 8.11.2 Evaluation Framework
| Claim Type | Weight | Confidence | Required Evidence |
|---|---|---|---|
| Design requirement | Low (intent) | Low (unverified) | Design documents, rationale |
| Specification clause | Medium (normative) | Medium-high (if ratified) | Rule registry entry, conformance claim |
| Implementation | Medium (actual) | Medium (if tested) | Test suite results, CI passes |
| Passing test suite | High (empirical) | High (if suite is comprehensive) | Test results, coverage report |
| Proof of correctness | Highest (formal) | Highest (if proof is valid) | Formal proof, discharged proof obligations |

#### ## 8.11.3 Applied Example
- **Claim**: "Omni has dependent types"
  - **If design requirement**: Low weight; expresses intent
  - **If specification clause**: Medium weight; if there's a `TYPE-0011` entry about generic parameter kinds
  - **If implementation**: Medium weight; if the compiler has dependent-type-like features (even if not fully specified)
  - **If passing test suite**: High weight; if there are tests demonstrating dependent-type-like behavior
  - **If proof of correctness**: Highest weight; if there's a formal proof that the type system has desired properties
- **The correct evaluation depends on which of the five categories the claim falls under**; mixing categories leads to confusion

---
## ## 9. Conversation-Integrated Design Decisions (detailed)

### ## 9.1 Four-Grade Framework Replacement (synthesis)

#### ## 9.1.1 Rejected Approach: Feature-Checklist Matrix
- **What was rejected**: Four columns (state-of-the-art, space-grade, enterprise-grade, military-grade) with features checked yes/no in each
- **Why rejected**: 
  - A feature can improve one dimension while making another harder to qualify
  - Dependent types may be useful for proofs but expand the type checker
  - A rich runtime may improve productivity but be inappropriate for constrained critical paths
  - Neither conclusion implies one feature must be globally prohibited
- **The fundamental flaw**: Treating the grades as interchangeable columns rather than overlapping but distinct assurance regimes

#### ## 9.1.2 Adopted Approach: Common Foundation with Assurance Profiles
- **What is adopted**: A common language foundation with separate assurance profiles that declare permitted features, runtime facilities, allocation policies, concurrency guarantees, supported targets, required evidence, and release gates
- **The common foundation**: The normative Edition 1 specification (syntax, semantics, type system, build pipeline, conformance framework) that is shared by all profiles
- **Profile declarations**: Each profile (`safe`, `distributed`, `real-time`, `high-assurance`, `general-purpose`) declares:
  - Which features are permitted (vs. excluded)
  - Which runtime facilities are permitted (vs. restricted)
  - Allocation policies (vs. prohibitions)
  - Concurrency guarantees (vs. prohibitions)
  - Supported targets (vs. unsupported targets)
  - Required evidence (vs. optional evidence)
  - Release gates (vs. optional gates)
- **Inheritance**: Profiles inherit from the common foundation; profile-specific deviations must be explicit and testable
- **Example**: The `safe` profile prohibits `unsafe`, `foreign` effect, and raw pointers; but allows GC, dynamic dispatch, and runtime reflection — all within the same language foundation

#### ## 9.1.3 Profile Interaction Matrix (example)
| Profile Pair | Inherited Restrictions | Additional Restrictions | Conflicts |
|---|---|---|---|
| `safe` × `distributed` | No `unsafe`, no raw pointers | Message-passing-only state sharing | None (compatible) |
| `safe` × `real-time` | No `unsafe`, no raw pointers | Bounded execution, priority inheritance | None (compatible) |
| `safe` × `high-assurance` | No `unsafe`, no raw pointers | Full formal verification, DDC | None (high-assurance subsumes safe) |
| `safe` × `general-purpose` | No `unsafe`, no raw pointers | GC, dynamic dispatch, reflection | none; general-purpose is the default; safe is a restriction |
| `distributed` × `real-time` | Message-passing-only | Bounded execution, priority inheritance | Possible conflict: message passing may have unbounded latency; requires careful design |
| `high-assurance` × `general-purpose` | Full formal verification, DDC | GC, dynamic dispatch, reflection | high-assurance subsumes many general-purpose features; transitions require amendment |

#### ## 9.1.4 Transition Pathways Between Profiles
- **`general-purpose` → `safe`**: Restrict features (remove GC? remove `unsafe`?); may require refactoring; migration path documented
- **`safe` → `high-assurance`**: Add formal verification discharge (Stage-0 + DDC); may require significant verification infrastructure; not automatic
- **`distributed` → `real-time`**: Combine message-passing determinism with timing guarantees; possible but requires careful design (bounded channels, priority inheritance)
- **Any profile → any other**: Must follow the controlled language-amendment process (Section 1.2); cannot be done implicitly

### ## 9.2 Formal Semantics Commitment (synthesis)

#### ## 9.2.1 Rejected: "Formal Semantics Rather Than Tests" as False Dichotomy
- **Why rejected**: Proofs establish properties testing cannot, but a trustworthy implementation still needs independent tests, regression suites, fuzzing, differential testing, build reproducibility checks, and trusted-tool validation
- **The TCB problem**: A proof is only as useful as its assumptions, model, and trusted computing base; a mathematically sound specification may have a defective compiler, linker, runtime, or build pipeline

#### ## 9.2.2 Adopted: Formal Semantics as Primary Artifact, Tests as Complement
- **What is adopted**: Formal semantics and proofs establish properties that testing cannot establish for all executions; but a trustworthy implementation still needs the full validation suite
- **Formal semantics** (ACL) is the primary artifact; test suites are complementary, not substitute
- **The five-fold distinction** is preserved: The formal semantics defines the specification clauses; test suites validate the implementation; the two are distinct but mutually reinforcing

#### ## 9.2.3 Formal Semantics Components
- **ACL definition**: The Aurora Core Logic (simply-typed lambda calculus extended with ownership, effects, capabilities, resource indicators)
- **Meta-theoretic properties**: Progress, preservation, effect soundness, capability soundness (with explicitly stated assumptions)
- **Formal judgment forms**: `Γ ⊢ e ⇐ T`, `Γ ⊢ e ⇒ T`, `Γ ⊢ v ↓ T`, `Γ ⊢ ε₁ ∼ ε₂`, `Γ ⊢ κ₁ ≈ κ₂`
- **Soundness theorems**: With assumptions explicitly stated (TCB, model, etc.)
- **Proof architecture**: Framework for discharging Stage-0 feature predicates; not merely asserted

### ## 9.3 "No Undefined Behavior" Commitment (synthesis)

#### ## 9.3.1 Rejected: Phrase Alone Without Precise Contract
- **Why rejected**: The phrase "no undefined behavior" alone does not define:
  - What happens when a program runs out of memory
  - What happens on arithmetic fault (overflow, underflow)
  - What happens on capability precondition violation
  - What happens on foreign code interaction
  - What happens on hardware exception
  - What happens on non-termination

#### ## 9.3.2 Adopted: Four-Category Definition (formal)
- **Category 1**: Behavior guaranteed by language semantics (fully specified in ACL)
- **Category 2**: Explicit, defined program failures (errors, panics with defined error types)
- **Category 3**: Programs rejected during compilation or verification (never execute)
- **Category 4**: Platform-dependent behavior permitted by declared target contract (TARGET-0001 through TARGET-0009)
- **Category 5**: Hardware faults and external failures the language cannot eliminate (accepted as inevitable; mechanisms provided for mitigation)

#### ## 9.3.3 Category 1 Details (formal)
- **Fully specified in the Aurora Core Logic (ACL)**
- **Examples**: Arithmetic operations (with checked arithmetic semantics); boolean operations; function calls (with effect/capability tracking); tuple construction/destruction; enum construction/dispatch; ownership transfers (linear/affine rules); effect operations (with effect row tracking)
- **Every code path has a defined observation**: This is the core guarantee; no path leads to "undefined" — every path either produces a value, a defined error, or is rejected statically

#### ## ## 9.3.4 Category 2 Details (formal)
- **Defined failures** with specific error types:
  - `ArithmeticOverflow`, `ArithmeticUnderflow` (checked arithmetic)
  - `IndexOutOfRange`, `ArrayBoundsViolation` (spatial safety)
  - `NullDereference`, `InvalidReference` (provenance violation, but defined error not UB)
  - `EffectMismatch`, `CapabilityViolation` (effect/capability rule violation)
  - `TypeError::MismatchedTypes` (type mismatch)
  - `ConstraintSolveFailure` (unifiable constraintes)
- **Every defined failure has**: An error type, an error message, and defined program state after the failure

#### ## 9.3.5 Category 3 Details (formal)
- **Programs rejected statically**: 
  - Type errors (MismatchedTypes, etc.)
  - Affine/linear violations (use-after-move, double-free-of-linear)
  - Effect tracking violations (undeclared effects, capability violations)
  - Grammar violations (syntax errors, ambiguous grammar)
- **These programs never execute**: The compiler rejects them before code generation
- **Test suite coverage**: The test suite should include tests that verify rejection of these programs

#### ## 9.3.6 Category 4 Details (formal)
- **Platform-dependent behavior** permitted by target manifest:
  - Endianness of data layout (declared in TARGET-0003)
  - Pointer widths per address space (declared in TARGET-0009)
  - Atomic widths/lock-freedom (declared in TARGET-0003)
  - Vector model (SIMD extensions; declared in TARGET-0003)
  - Object format (ELF, PE, raw; declared in TARGET-0003)
- **These are not "undefined"** — they are "implementation-defined" with the target contract as the definition
- **The target manifest provides the definition**; the language does not leave these unspecified

#### ## 9.3.7 Category 5 Details (formal)
- **Hardware faults and external failures** the language cannot eliminate:
  - Memory bit flips (radiation-induced)
  - Power loss / sudden shutdown
  - Network disconnection
  - OS process kill/kill signal
  - External device failure
- **The language provides mitigation mechanisms** (but cannot eliminate):
  - Redundancy (multiple versions, DDC)
  - Bounded recovery (timeouts, fallbacks)
  - Diagnostic integration (integration with platform fault handlers)
  - Exception handling (for defined faults, not UB)

### ## 9.4 Profile-Based Grade Framework (synthesis)

#### ## 9.4.1 Rejected: Four Grades as Interchangeable Columns
- **Why rejected**: Same reason as 9.1.1; features have trade-offs that don't map cleanly to grade labels; a feature useful in one context may be harmful in another, regardless of the "grade" label

#### ## 9.4.2 Adopted: Common Foundation with Profiles
- **What is adopted**: (See 9.1.2); the same language supports productive application development and more restrictive critical applications without pretending identical requirements
- **The common foundation** ensures that all profiles are of the same language; profiles differ in restrictions, not in fundamental semantics
- **This is better than** globally prohibiting features such as GC or dynamic dispatch merely because a more restrictive profile might not permit them

#### ## 9.4.2 Profile Granularity
- **Five defined profiles**: `safe`, `distributed`, `real-time`, `high-assurance`, `general-purpose`
- **Each profile** has a core restriction set and a core allowance set
- **Profiles may be tailored** per ECSS-S-ST-00 (with restrictions; core prohibitions are immutable)
- **Profile switching** requires the controlled language-amendment process

### ## 9.5 Aurora Integration Policy (synthesis)

#### ## 9.5.1 Rejected: Merging Aurora's Syntax/Semantics Wholesale
- **Why rejected**: 
  - Aurora duplicates Omni's specification identity by reusing `BUILD`, `CONF`, `UPDATE`, and `TARGET` identifiers without provenance
  - Aurora's formal foundation is described, not defined (naming extensions doesn't define semantics)
  - Some illustrative formal rules are not adequate (effect rule, linear usage rule)
  - The type-system story is inconsistent (simply typed core with extensions, but citing rank-N types, dependent types, gradual typing)
  - Syntax examples need correction (delimiter errors, truncated Unit entry, mandatory where clause)
  - Explicit effects vs implicit capabilities contradiction (every function declares effects yet also has implicit capability environment K)
  - Features claimed in rationale but not defined in language model (gradual typing, WASM target, information-flow control, SBOM integration, radiation-hardened memory)
  - Verification mandated but verifier not defined (Stage-0 + DDC required, but no proof architecture, TCB, equivalence criterion, qualification boundary, or tool validation procedure)
  - Declarative format not automatically formal specification (AML as Markdown+YAML is an authoring format, not a mathematically sound definition)
  - Concurrency definitions are a starting point, not a finished contract (exact observable behavior under races, etc. not settled)
  - Aurora's ambition larger than its current specification (claims to satisfy all four grades, but substantial future work is listed)

#### ## 9.5.2 Adopted: Aurora as Research Inspiration Artifact
- **What is adopted**: Aurora treated as a research artifact that can inspire design discussions
- **Not merged into Omni**: Its specification snippets should not be merged into Omni
- **Not used as independent evidence of Aurora's maturity**: Its duplicated identifier ranges carry provenance; should not be used as evidence of maturity
- **Formal model and examples need substantial correction**: Before they could support implementation
- **Specific rejections** (from 9.5 in the original review):
  - Do not import Aurora's function syntax, continuation rules, effect syntax, linearity rules, or YAML semantic representation as ready-made solutions
  - Do not change Omni's normative Edition 1 grammar or revive disabled Candidate-2 syntax based on Aurora
  - Aurora's generic-parameter production delimiter error, truncated `Unit` entry, and mandatory `where` clause appearance are not acceptable in Omni's normative specification

#### ## 9.5.3 What Can Be Learned from Aurora
- **Effect rows as first-class types**: Idea of making effect rows part of the type signature (not just annotations) is useful and may be adopted in Omni
- **Staged verification**: The idea of Stage-0 as a verified subset is useful and already part of Omni
- **Content-addressed build pipeline**: The idea of content-addressed builds (BUILD-0001 through BUILD-0011) is shared and valuable
- **Conformance claim structure**: The CONF-0001 through CONF-0011 structure is shared and valuable
- **DDC requirement**: The mandatory DDC proposal is useful and may be adopted in Omni's qualification framework
- **Profile-based approach**: The idea of profile-based assurance (rather than feature-checklist matrix) is shared and valuable

#### ## 9.5.4 What Should Not Be Imported from Aurora
- **Function syntax**: Aurora's function definition/signature syntax (would conflict with Omni's AML grammar)
- **Continuation rules**: Aurora's newline-termination/continuation token rules (would conflict with Omni's VIBE-GRAM rules)
- **Effect syntax**: Aurora's effect row syntax and handling (would need independent design for Omni)
- **Linearity rules**: Aurora's linear/affine type tracking rules (would need independent design for Omni; Omni has its own approach with the `FlowState.reference_loans` gap)
- **YAML semantic representation**: Aurora's YAML-based semantic representation format (would need independent design for Omni; Omni has its own rule registry and digest procedure)
- **Identifier ranges**: Aurora's reuse of `BUILD`, `CONF`, `UPDATE`, `TARGET` identifier ranges (carries provenance; should not be adopted as Omni's identifiers without attribution)

### ## 9.6 Specification Clause Distinction (synthesis)

#### ## 9.6.1 Core Principle (restated)
- **Design requirement, specification clause, implementation, passing test suite, proof of correctness are five different things**

#### ## 9.6.2 Why the Distinction Matters
- **Design requirement**: The intention behind a feature; weight = low; confidence = low (unverified); evidence = design documents, rationale
- **Specification clause**: The normative statement of the feature; weight = medium-high; confidence = medium-high (if ratified); evidence = rule registry entry, conformance claim
- **Implementation**: The actual code realizing the feature; weight = medium; confidence = medium (if tested); evidence = test suite results, CI passes
- **Passing test suite**: Empirical demonstration the feature works; weight = high; confidence = high (if suite is comprehensive); evidence = test results, coverage report
- **Proof of correctness**: Formal verification the feature is correct; weight = highest; confidence = highest (if proof is valid); evidence = formal proof, discharged proof obligations

#### ## 9.6.3 Applied Evaluation Framework
When evaluating any claim about Omni, ask: "Which of the five categories does this claim fall under?" The answer determines the weight and confidence:

| If the claim is a... | Then its weight is... | And its confidence depends on... |
|---|---|---|
| Design requirement | Low | The design rationale (unverified by default) |
| Specification clause | Medium-high | Whether the rule is ratified (RULE-0001 etc.) |
| Implementation | Medium | Test suite coverage and CI passes |
| Passing test suite | High | Suite comprehensiveness (branch coverage, path coverage) |
| Proof of correctness | Highest | Proof validity (correctness of the proof itself, not just its conclusion) |

#### ## 9.6.3 Example Evaluation Scenarios
- **Scenario A**: "Omni will never have undefined behavior"
  - **Category**: Specification clause (if stated as a normative rule) or design requirement (if stated as intent)
  - **Weight**: Medium-high (if clause) or Low (if requirement)
  - **Confidence**: Medium-high (if clause is ratified, e.g., TERM-0005) or Low (if merely stated as intent)
  - **Action**: If clause, it's a normative rule (TERM-0005: "erroneous program SHALL NOT produce conforming release artifact"); if requirement, it's a goal to work toward

- **Scenario B**: "Omni's type system has dependent types"
  - **Category**: Could be any of the five, depending on what exactly is claimed
  - **If design requirement**: Low weight, expresses intent
  - **If specification clause**: Medium weight (if there's a TYPE-0011 entry about generic parameter kinds including dependent types)
  - **If implementation**: Medium weight (if the compiler has dependent-type-like features, even if not fully specified)
  - **If passing test suite**: High weight (if there are tests demonstrating dependent-type-like behavior, e.g., programs where types depend on values)
  - **If proof of correctness**: Highest weight (if there's a formal proof that the type system has the desired dependent-type properties)
  - **The correct evaluation**: Ask which category the claim falls under; don't mix categories

- **Scenario C**: "Omni has content-addressed builds"
  - **Category**: Specification clause (if stated as a normative rule about the build pipeline) or implementation (if describing the actual build system)
  - **If clause**: Weight = medium-high (if BUILD-0001 through BUILD-0011 are in the rule registry as ratified rules)
  - **If implementation**: Weight = medium (if the actual compiler/build system implements content-addressed builds)
  - **If test suite**: Weight = high (if the test suite includes tests verifying content-addressed builds produce identical outputs for identical inputs)
  - **The claim**: Likely a specification clause (it's in the normative build pipeline specification); weight = medium-high

### ## 9.6.4 Preservation Rule
- **Omni's engineering plans and release claims must preserve the five-fold distinction throughout**
- **No claim should be evaluated at one weight while being presented as another type**
- **Release documentation should explicitly categorize each claimed feature** according to the five-fold distinction
- **This distinction is the single most important takeaway from the design review for Omni's engineering process**

---
## ## 10. File Structure and Navigation (final summary)

The specification is organized into the following sections for ease of reference:

```
Omni_Complete_Specification_Generated.md
├── 1. Language Governance           (versioning R.M.m.P, architecture freeze, Rust bootstrap, evolution path)
├── 2. Specification Authority        (digest domain, conflict resolution, normative vs non-normative, spec_tree_sha256)
├── 3. Conformance and Certification (CONF-0001..0011, assurance profiles, certification revocation)
├── 4. Build Pipeline                 (BUILD-0001..0011 content-addressed, build gates, provenance UPDATE-0001..0008)
├── 5. Target Model                   (TARGET-0001..0009 canonical target identity, baseline restrictions, feature dispatch)
├── 6. Formal Semantics and Verification (ACL, Stage-0 STAGE-0-0001..0007, DDC, proof obligations, effect system, ownership/affine types, unsafe discipline, memory safety invariants, concurrency CONC-0001..0023)
├── 7. Rule Registry and Ownership    (rule lifecycle Proposed/Candidate/Ratified/Deprecated/Superseded/Withdrawn, omni-audit linkage reaper, witness requirements, coverage metric)
├── 8. Priority Issues and Action Items (10 ranked priorities from design review, with detailed sub-points)
├── 9. Conversation-Integrated Design Decisions (synthesis of all discussion points, rejected/approved approaches)
├── 10. File Structure and Navigation
```

## References and Cross-References

This generated specification references the following Omni project artifacts (all paths relative to `C:\Users\siddh\Downloads\ABC\Omni/`):

- `docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.1_updated.md` — Original normative specification (7326 lines)
- `docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.2_vibe.md` — Candidate 2 vibe reconciliation (7614 lines)
- `docs/implementation/Omni_Implementation_Master_Plan_Edition1_Final_Comprehensive_MicroAtomic_Final.md` — Implementation master plan (4750 lines)
- `docs/foundation-gates.md` — Foundation sequence and gate definitions (249 lines)
- `docs/grammar-reconciliation.md` — Grammar reconciliation notes (204 lines)
- `spec/grammar/omni-edition1.ebnf` — Normative EBNF grammar (390 lines)
- `spec/reference/core_semantics_and_types.md` — Core semantics and type system (286 lines)
- `spec/registry/rules.json` — Rule registry (5109 lines of rule texts)
- `spec/registry/rule-texts.json` — Rule text schema
- `docs/architecture/generic_closer_contract.md` — Generic closer contract
- `docs/architecture/lifetime_lex_parse_contract.md` — Lifetime/lexical/parse contract
- `IMPLEMENTATION_PLAN.md` — Implementation roadmap (406 lines)
- `IMPLEMENTATION_PLAN_GENERATED.md` — Generated implementation plan (264 lines)
- `foundation-gates.md` — Foundation gates (0.0.0.2 through 0.0.0.13)
- `candidate2-erratum.md` — Candidate 2 errata (92 lines)

**Conversation sources**:
- Design review of "Idealized Programming Language Design" analysis (October 2026)
- Aurora language proposal analysis (October 2026)
- Military language requirements (DoD "Ironman" document, 1976)
- Space language requirements (NASA reports, Apollo/Shuttle eras)
- Formal methods literature (type systems, verification, formal semantics)
- Enterprise language design patterns (module systems, build pipelines, package management)

## Generation Notes

This file was **synthesized** from:
1. The Omni project repository specification and implementation materials
2. External research on language design for extreme environments (space, military, enterprise)
3. Design review feedback on both the original analysis and Aurora proposal
4. October 2026 conversation discussion of idealized programming language requirements

**Intent**: This is a **temporary reference** file — it captures the design decisions, priority issues, and conversation-integrated synthesis for ongoing Omni engineering work. It is not itself a normative release artifact; the canonical specification remains the Omni project's Edition 1 specification in `docs/specification/`.

**Update policy**: This file should be updated when:
- Omni's normative specification is revised (errata, amendments)
- New assurance profiles are defined or existing ones modified
- Priority issues are resolved (tracked in issue/issue-tracking system)
- The `omni-canon` canonicalization engine is started and validated
- DDC procedure is formally specified and validated
- Reference/ownership soundness gaps are closed

**Verification status**: This file synthesizes information from multiple sources; individual claims should be verified against the original Omni project artifacts and the peer review conversation. The five-fold distinction (design requirement, specification clause, implementation, passing test suite, proof of correctness) must be preserved when evaluating any part of this specification.

---
*Generated: October 2026 — Synthesized from conversation research and Omni project analysis*