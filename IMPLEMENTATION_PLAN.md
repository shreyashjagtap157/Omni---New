# Omni Implementation Plan - Edition 1

## Current Status (as of 2026-10-12)

### Implementation State
- **Bootstrap language**: Rust (pinned toolchain)
- **Current version**: `1.0.0.0` architecture baseline (Candidate 3)
- **Recent commit**: Ownership verification improvements to `compiler/omni-verify/src/ownership.rs`
  - Improved error message for projected reference storage
  - Added drop logic for reference places with projections
  - Enhanced self-reborrow test expectation message
- **Git state**: Pushed to `origin/main` at commit `c0c9440`

### Compiler Crates Status
| Crate | Status | Key Features |
|-------|--------|-------------|
| `omni-lex` | ✅ Core complete | 80+ keywords, punctuation, error handling, trivia preservation, UTF-8 validation |
| `omni-parse` | ✅ Core complete | Pratt parser, binding power table, layout handling, CST construction |
| `omni-types` | ✅ Core complete | Type checker, monomorphization, unification, solver |
| `omni-names` | ✅ Core complete | Package/module identities, imports, shadowing, visibility |
| `omni-own` | ✅ Framework complete | Ownership state, loan tracking, reborrow dependencies |
| `omni-effects` | ⚠️ Framework only | Effect/capability system skeleton |
| `omni-hir` | ⚠️ Partial | HIR lowering with some Stage-0 support |
| `omni-mir` | ✅ Definition complete | MIR model: basic blocks, places, projections, operands, rvalues, statements, terminators |
| `omni-verify` | ⚠️ Partial | MIR verifier with CFG integrity, initialization, ownership, drops, assumptions |
| `omni-machine` | ⚠️ Not started | Reference abstract machine, native lowering |
| `omni-codegen` | ⚠️ Not started | Code generation |
| `omni-runtime` | ⚠️ Not started | Runtime support |

### Specification Alignment
- **Normative spec**: `Omni Complete Specification Edition 1 Candidate 3`
- **Implementation plan**: `Omni Implementation Master Plan Edition 1 Vibe First Revised`
- **Current target**: Stage 1 foundation (0.0.0.1 → 1.0.0.0)

---

## Gaps vs Specification

### Critical Gaps (Stage 1 Foundation)

| Specification Section | Current Status | Implementation Priority |
|----------------------|----------------|------------------------|
| **ERR3-0010** (CST preservation) | ✅ Partially implemented | Lossless CST with byte-accurate spans |
| **ERR3-0011** (Ill-formed UTF-8) | ✅ Implemented | Rejected at source layer with E0101 |
| **ERR3-0012** (Raw-string hash) | ⚠️ Partial | Minimum 255 limit declared |
| **ERR3-0013** (64-bit offsets) | ⚠️ Partial | Span model uses 64-bit offsets |
| **ERR3-0014** (Number parsing) | ✅ Implemented | Leading `.5` not float, require `0.5` |
| **ERR3-0015** (Interpolation) | ⚠️ Partial | `{expr}` in strings, `{{`/`}}` for literal braces |
| **ERR3-0016** (Keywords) | ✅ Implemented | Contextual vs hard-reserved sets |
| **ERR3-0017** (Pipeline `\|>`) | ⚠️ Partial | Standard token, never splits |
| **ERR3-0018** (Comments) | ✅ Implemented | `//` and `/* */` (nesting) |
| **ERR3-0020** (Statement termination) | ⚠️ Partial | Newline semantics, `;` as optional terminator |
| **ERR3-0021** (Trailing continuation) | ⚠️ Partial | Binary operator/`,`/`,` etc. continue |
| **ERR3-0022** (Leading continuation) | ⚠️ Partial | Next line begins with continuation chars |
| **ERR3-0023** (Never leading) | ⚠️ Partial | Lines starting with `- + * & ! ~ ( [ < \|` are new stmts |
| **ERR3-0024** (Delimiter nesting) | ⚠️ Partial | `( )` and `[ ]` suspend termination |
| **ERR3-0025** (`.name` continuation) | ⚠️ Partial | Postfix chain continuation |
| **ERR3-0026** (Machine-readable tables) | ❌ Not started | `grammar/layout.yaml` not generated |
| **ERR3-0030** (Pipeline `[ADOPTED]`) | ⚠️ Partial | `x \|> f` means `f(x)` |
| **ERR3-0031** (`\|>` precedence) | ⚠️ Partial | Above assignment, below `??`, `or`, `and` |
| **ERR3-0032** (`?` binding) | ⚠️ Partial | `?` binds tighter than `\|>` |
| **ERR3-0033** (Projection) | ⚠️ Partial | Leading `.path` in stage args desugars to `\|it\| it.path` |
| **ERR3-0034** (Bare `it`) | ⚠️ Partial | Available only through leading-dot desugaring |
| **ERR3-0035** (Command calls) | ⚠️ Partial | At statement start, `let`, `=`, `return`, `\|>` |
| **ERR3-0036** (Closures) | ⚠️ Partial | `\|x\| e`, `\|x, y\| e`, `\|\| e`, `x => e` |
| **ERR3-0037** (Optional chaining) | ⚠️ Partial | `a?.b` yields `Option`, `a ?? b` coalesce |
| **ERR3-0040** (Expression-bodied) | ⚠️ Partial | `fn add(a, b) = a + b` |
| **ERR3-0041** (Struct disambiguation) | ⚠️ Partial | `{` in conditions = block, not struct literal |
| **ERR3-0042** (Parameter types) | ✅ Implemented | Public items require explicit types |
| **ERR3-0043** (Effect syntax) | ⚠️ Partial | `fn f(x) -> T / io, fs.read` |
| **ERR3-0044** (`requires`) | ⚠️ Partial | Implicit context parameters from capability env `K` |
| **ERR3-0045** (Ambient calls) | ⚠️ Partial | `print`, `open`, `sleep` with implicit context |
| **ERR3-0046** (main signature) | ⚠️ Partial | Script mode: `fn main(args: Args, caps: AppCaps) -> Exit` |
| **ERR3-0047** (Handlers vs resources) | ⚠️ Partial | `with` for resources, `handle E { ... } in { ... }` for handlers |
| **ERR3-0050** (parallel) | ⚠️ Partial | Structured task scope, policy algorithms |
| **ERR3-0051** (parallel with bindings) | ⚠️ Partial | Returns `Vec<T>` in input order |
| **ERR3-0052** (parallel effects) | ⚠️ Partial | Children effects join into scope row |
| **ERR3-0053** (parallel for) | ⚠️ Partial | Returns `Vec<T>` in spawn order |
| **ERR3-0060** (Ownership affine) | ✅ Implemented | `linear` withdrawn, `#[must_consume]` verifier check |
| **ERR3-0061** (Generator/lazy) | ❌ Not started | `Seq<T>` generator type |
| **ERR3-0062** (Generics) | ⚠️ Partial | Full monomorphization, dictionary passing |
| **ERR3-0063** (Variance) | ⚠️ Partial | Declared on public types, inferred on private |
| **ERR3-0064** (Inference) | ⚠️ Partial | Bidirectional checking, equality by union-find |
| **ERR3-0065** (Borrow analysis) | ⚠️ Partial | Polonius-derived, higher-ranked lifetimes |
| **ERR3-0066** (MIR normative) | ⚠️ Partial | Place-based CFG, SSA property of LIR only |
| **ERR3-0067** (Precise int types) | ✅ Implemented | `int<N>`/`uint<N>` generic forms, `i8..i128` aliases |
| **ERR3-0068** (Byte alias) | ✅ Implemented | `byte` is alias of `u8` |
| **ERR3-0069** (Stage-0 definition) | ⚠️ Partial | UTF-8 idents, modules, structs, enums, Option/Result, fixed ints/bool/byte |
| **ERR3-0070** (Target manifest) | ⚠️ Partial | Hosted x86-64 and RV64 bare-metal |
| **ERR3-0071** (unsafe_code allowlist) | ✅ Implemented | Workspace-wide `forbid` with audited allow-list |
| **ERR3-0072** (`[implements]`/`[witnesses]`) | ✅ Recognized | Inert attributes by `omni-audit` |
| **ERR3-0080-0089** (Package/build/governance) | ⚠️ Partial | CI ceilings, canonical JSON, profiles |

### Moderate Gaps

| Area | Status |
|------|--------|
| Canonicalization engine (`omni-canon`) | ❌ Not started |
| Rule registry schema (Proposed/Candidate/Ratified/Deprecated) | ⚠️ Partial framework |
| CI Reaper and semantic linkage | ⚠️ Partial |
| Conformance and diagnostic schemas | ⚠️ Partial |
| Artifact identity and provenance | ⚠️ Partial |
| Stage-0 feature predicate system | ⚠️ Partial |
| Semantic linkage and anti-rot system | ⚠️ Partial |
| Repository topology organization | ⚠️ Partial |
| Dependency graph rules | ⚠️ Partial |
| Three-stage roadmap gates | ⚠️ Partially structured |
| Version transition rules | ⚠️ Partial |
| Anti-rot system (rule ownership, test witnesses) | ⚠️ Partial |

---

## Contradictions & Conflict Resolution

### 1. ERR3-0010 vs SRC-0002 Conflict
**Issue**: CST preserves original bytes (including BOM/CR-CRLF) vs round-trip normalization
**Resolution**: CST is lossless byte preservation; normalization to UTF-8 only for semantic hashing. `cst.text() == original bytes` exactly. Normalization map yields normalized UTF-8 for semantics.

### 2. Number Parsing ERR3-0014
**Issue**: Leading `.5` not a float; require `0.5`; leading `.` before identifier is projection
**Resolution**: Implemented in scanner state machines - `.5` rejected as float, `.5identifier` treated as projection

### 3. Keyword Classification ERR3-0016
**Issue**: `and`/`or` as contextual keywords vs `&&`/`||` word forms with identical precedence
**Resolution**: Separate keyword sets implemented - hard-reserved set minus contextual `where with parallel requires handle and or when in`

### 4. Statement Continuation ERR3-0021/ERR3-0022
**Issue**: Trailing/leading continuation rules with multiple operator classes
**Resolution**: Parser implements Pratt binding power for continuity; ERR3-0024 delimiter nesting suspends termination inside `( )` and `[ ]`

### 5. Pipeline Precedence ERR3-0031
**Issue**: `\|>` above assignment, below `??`, `or`, `and`
**Resolution**: Binding power table correctly places `\|>` at precedence 7 (above assignment at 1, below `??` at 6, `or` at 4, `and` at 5)

### 6. Effect Syntax ERR3-0043
**Issue**: `/` never division in signature position after `->`
**Resolution**: Parser handles `/` as effect separator, not division, in function signatures

### 7. Stage-0 Definition ERR3-0069
**Issue**: Explicit Stage-0 subset definition vs implementation convenience
**Resolution**: Implementation must fail-closed on Stage-1 features not in defined subset: macros, async, effect handlers, traits beyond marker/core, dynamic, `parallel`, refinements

### 8. MIR Normative Model ERR3-0066
**Issue**: Normative MIR is place-based CFG vs SSA property of LIR only
**Resolution**: MIR verifier implements place-based CFG with explicit memory objects, projections, switch, fault, cleanup, cancellation, capability edges

### 9. Versioning Namespace ERR3-0002
**Issue**: `OMNI-IMP-X.Y.Z` namespace withdrawn vs `R.M.m.P` four-component form
**Resolution**: Adopted `R.M.m.P` form; `OMNI-IMP` namespace deprecated

### 10. DDC Self-Compilation ERR3-0084
**Issue**: `v2 == v3` self-compilation check vs independent compiler lineage
**Resolution**: DDC requires independent compiler lineage (`BOOT-0004/0005`); LLVM, libc, linker classified in Stage-3 dependency ledger

### 11. Independent Parser CONF-0004
**Issue**: Rust and Omni compilers alone don't satisfy "independent parser" requirement
**Resolution**: Third-party or clean-room parser scheduled for ratification; Omni compilers alone do not satisfy

### 12. Effect Sets ERR3-0088
**Issue**: `EFF-0007` lists `unwind`, `dynamic`, `foreign`, `block`, `log`; `OMNI-DIST` deferred
**Resolution**: Effect sets registered in profile table; `OMNI-DIST` tokens remain reserved for future

### 13. Profile `safe` ERR3-0086
**Issue**: Defined as: no `unsafe`, no `foreign` effect, no raw pointers
**Resolution**: Registered in profile table; enforcement via type system and checker

---

## Prioritized Implementation Roadmap

### Stage 1: Rust Minimal Core (0.0.0.1 → 1.0.0.0) - **PRIORITY PHASE**

#### Phase 1.1: Foundation (0.0.0.1 → 0.0.0.4) - **IMMEDIATE**
- [ ] Repository initialization with proper directory structure
- [ ] Rust toolchain pinning in `rust-toolchain.toml`
- [ ] Quality/CI boundary setup (ceilings, fuzz budgets)
- [ ] Rule registry schema with lifecycle states
- [ ] Specification manifest creation
- [ ] Canonicalization engine (`omni-canon`)

**Rationale**: These are the bedrock upon which all later implementation depends. Without proper foundation, later work risks semantic divergence.

#### Phase 1.2: Lexer/Parser (0.0.1.x → 0.0.2.x) - **HIGH PRIORITY**
Complete ERR3-0010 through ERR3-0018 and ERR3-0020 through ERR3-0027:

- [ ] Byte cursor and UTF-8 decoder with BOM handling at offset zero
- [ ] Source normalization and canonical line-ending handling
- [ ] Token kinds, trivia preservation, identifiers/keywords
- [ ] Unicode identifier data ingestion and confusable validation
- [ ] Numeric literal state machines with ERR3-0014 compliance
- [ ] Character/string literals and raw strings
- [ ] Operators and punctuation with maximal-munch
- [ ] Context-sensitive generic punctuation splitting
- [ ] Lexical diagnostics
- [ ] Deterministic EOF behavior
- [ ] Lossless CST with comment/whitespace preservation
- [ ] Grammar productions from normative data
- [ ] Deterministic recursive-descent declaration parsing
- [ ] Pratt-style expression parsing
- [ ] Pattern grammar and generic syntax
- [ ] Error recovery and syntax diagnostics
- [ ] AST lowering to semantic layer

**Rationale**: Lexer and parser are the entry point to the compiler. Full spec compliance here ensures all subsequent processing receives well-formed input.

#### Phase 1.3: Type System (0.0.3.x) - **HIGH PRIORITY**
- [ ] Package and module identities
- [ ] Lexical scopes and DefId allocation
- [ ] Imports, shadowing rules, visibility checks
- [ ] Primitive types, tuples/arrays/slices
- [ ] Structs/enums, function types
- [ ] Type variables, interning, unification
- [ ] Expected-type checking, inference diagnostics
- [ ] Explicit coercions only where specified

**Rationale**: Type system is the foundation for semantic analysis. Stage-0 type checking must be sound and fail-closed.

#### Phase 1.4: Ownership/Effects (0.0.4.x) - **HIGH PRIORITY**
- [ ] Minimal Stage-0 ownership/lifetime for sound MIR
- [ ] Interface architecture for complete Edition 1 model
- [ ] Type-owned ownership relations
- [ ] Valid lifetime relationships from ownership
- [ ] Effects classification without capability authority grant
- [ ] Capability environment governance
- [ ] Continuation type lifetime constraints
- [ ] Higher-ranked effect binders

**Note**: Recent commit improved ownership verification for projected reference storage and drop logic - this work continues here.

#### Phase 1.5: HIR/MIR (0.0.5.x) - **HIGH PRIORITY**
- [ ] HIR lowering from syntax-oriented structure
- [ ] Elimination of syntactic distinctions no longer semantically useful
- [ ] MIR definition: basic blocks, places, projections, operands
- [ ] Rvalues, statements, terminators
- [ ] Storage state, initialization state, drops
- [ ] Calls, control flow, effect/capability transitions
- [ ] Unsafe assumptions: assume.create, assume.use, assume.invalidate
- [ ] MIR verifier: CFG integrity, block reachability, initialization, ownership/move legality, drop legality, assumption lifetime/dominance, effect constraints, capability constraints, representation invariants, impossible state rejection

**Rationale**: HIR/MIR is the intermediate representation bridge between front-end and back-end. Full verifier ensures semantic integrity.

#### Phase 1.6: Reference Execution (0.0.6.x → 0.0.8.x) - **MEDIUM PRIORITY**
- [ ] Reference abstract machine with allocation identity, byte-level memory, pointer provenance
- [ ] Initialization/lifetime state, panic/abort/cancellation states
- [ ] Low-level lowering to target-neutral representation
- [ ] Minimal runtime: process startup/exit, I/O, memory allocator, panic/abort path
- [ ] Standard core subset for Stage-0

#### Phase 1.7: Backend (0.0.7.x) - **MEDIUM PRIORITY**
- [ ] Cranelift as initial backend (deterministic path)
- [ ] Target-neutral low-level representation
- [ ] Minimal Stage-0 executable slice

#### Phase 1.8: Qualification (0.0.9.x → 0.0.13.x) - **GATE PRIORITY**
- [ ] All CI boundaries enforced (PR: 20min, main: 30min, nightly: 60min)
- [ ] Canonicalization deterministic
- [ ] Host-path virtualization deterministic
- [ ] Provenance generated correctly
- [ ] No invented compiler semantics
- [ ] Full conformance suite passage
- [ ] Stage 1 exit criteria met: `1.0.0.0`

---

### Stage 2: Complete Production Rust Platform (1.0.0.1 → 2.0.0.0)

#### Phase 2.1: Language Features Beyond Stage-0
- [ ] Async support and effect handlers
- [ ] Trait system with coherence
- [ ] Full generic monomorphization (beyond Stage-0 core types)
- [ ] Cross-package generics with `.omniface` interface artifacts
- [ ] Refinements and specialized type constraints

#### Phase 2.2: Full Toolchain
- [ ] Complete code generation pipeline
- [ ] Optimizing backend (LLVM integration or Cranelift improvements)
- [ ] Full runtime support
- [ ] Package manager and build system
- [ ] Testing infrastructure

#### Phase 2.3: Qualification
- [ ] Full conformance test suite passage
- [ ] DDC (self-compilation) qualification
- [ ] Stage 2 exit criteria met: `2.0.0.0`

---

### Stage 3: Self-Hosted Omni (2.0.0.1 → 3.0.0.0)

#### Phase 3.1: Omni Compiler in Omni
- [ ] Rewrite compiler front-end/middle in Omni
- [ ] Sever Rust dependencies incrementally
- [ ] Complete DDC with independent compiler lineages
- [ ] Certified self-hosting toolchain

#### Phase 3.2: Final Certification
- [ ] Full qualification and certification
- [ ] Dependency severance from Rust
- [ ] Independent Omni toolchain
- [ ] Stage 3 exit criteria met: `3.0.0.0`

---

### Version Allocation Law Enforcement

- **Patch versions**: Only for atomic coherent engineering changes
- **Minor versions**: Only after associated subsystem integrated and qualified
- **Major versions**: Only at deliberate implementation-generation boundary
- **Release gates**: Only after stage exit criteria satisfied
- **No stage skipping**: Earlier stage must be qualified before later stage claimed

---

## What's Already Conforming vs What Needs Work

### ✅ Already Conforming (Partially Complete Stage 1 Foundation)

- Basic lexer tokenization with keyword/punctuation sets
- Pratt expression parsing with binding power table
- Core type system with unification and solver
- Basic ownership/lifetime framework (enhanced in recent commit)
- MIR definition and partial verifier
- Test corpus and fuzz targets
- Stage-0 subset definition (ERR3-0069)
- `unsafe_code = "forbid"` with allowlist (ERR3-0071)
- `and`/`or` as contextual keywords (ERR3-0016)
- Number parsing with leading `.` rejection (ERR3-0014)
- Affine ownership without `linear` (ERR3-0060)
- `byte` as `u8` alias (ERR3-0068)
- Precise integer types `int<N>`/`uint<N>` (ERR3-0067)
- MIR place-based CFG model (ERR3-0066)
- Minimal runtime identification (ERR3-0080 series)
- CI boundary enforcement (ERR3-0083)

### ⚠️ Needs Significant Work

- Full ERR3 specification compliance across all rule IDs (77+ rules)
- Pipeline/projection desugaring semantics (ERR3-0030 to ERR3-0037)
- Effect/capability system full implementation (ERR3-0043 to ERR3-0048, ERR3-0086 to ERR3-0089)
- MIR verifier completeness (CFG, initialization, ownership, drops, assumptions)
- Reference abstract machine implementation (ERR3-006.x)
- Native code generation with backend (Cranelift integration)
- Conformance test suite development
- DDC and Stage 2/3 qualification paths
- Canonicalization engine (`omni-canon`)
- Rule registry schema with lifecycle states
- CI Reaper and semantic linkage
- Repository topology organization
- Dependency graph rules enforcement
- Anti-rot system (rule ownership, test witnesses)
- Package/build/governance infrastructure

### 🚫 Not Started (Critical for Stage 1 Completion)

- Canonicalization engine (`omni-canon`)
- Rule registry schema (Proposed/Candidate/Ratified/Deprecated lifecycle)
- Specification manifest creation
- CI Reaper and semantic linkage
- Repository topology organization
- Anti-rot system components
- Feature predicate system for Stage-0
- Artifact identity and provenance
- Full conformance test suite
- Reference abstract machine
- Native code generation beyond minimal slice
- DDC qualification infrastructure
- Stage 2/3 transition planning

---

## Success Metrics for Stage 1 Completion (`1.0.0.0`)

| Metric | Target |
|--------|--------|
| CI ceiling (PR) | ≤ 20 minutes |
| CI ceiling (main) | ≤ 30 minutes |
| CI ceiling (nightly) | ≤ 60 minutes |
| Conformance test passage | 100% Stage-0 subset |
| Fuzz budget (PR) | ≤ 60 seconds per target |
| Fuzz budget (main) | ≤ 5 minutes per target |
| Fuzz budget (nightly) | ≤ 15 minutes per target |
| Deterministic builds | All artifacts reproducible |
| Host-path virtualization | Deterministic, CWD-independent |
| Provenance generation | Correct SHA-256 digests |
| Rule ownership traceability | 100% semantic decisions linked to rule IDs |
| No semantic duplication | No facts hard-coded in multiple locations |
| Fail-closed behavior | All unsupported semantics explicit errors |
| Previous functionality qualified | Regression suite passage |

---

## Immediate Next Steps

1. **Continue Phase 1.2**: Complete lexer/parser ERR3-0010 through ERR3-0027 compliance
2. **Continue Phase 1.4**: Enhance ownership verification (building on recent commit)
3. **Start Phase 1.3**: Implement Type System (0.0.3.x) core infrastructure
4. **Initialize Phase 1.1**: Set up foundation (toolchain, CI, rule registry, manifest)
5. **Create conformance test suite** for Stage-0 subset
6. **Develop reference abstract machine** skeleton
7. **Implement canonicalization engine** (`omni-canon`)

---

**Plan generated**: 2026-10-12

*This plan follows the three-stage generational approach specified in the Omni Implementation Master Plan Edition 1 (Vibe First Revised), with Stage 1 targeting `1.0.0.0` as the minimal Rust core qualification gate.*