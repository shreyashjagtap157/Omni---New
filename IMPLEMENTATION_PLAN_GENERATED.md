# Omni Implementation Plan - Derived from Specification Analysis
## Generation Date: 2026-10-12
## Normative Specification: Edition 1 Candidate 1.0.0-candidate.1 (Singular Baseline)

---

## 1. Implementation Status Overview

### 1.1 Current State
- **Bootstrap language**: Rust (pinned toolchain via rust-toolchain.toml)
- **Current version**: 1.0.0.0 architecture baseline
- **Implementation phase**: Stage 1 Rust-only minimal compiler
- **Normative specification**: Omni Edition 1 Candidate 1.0.0-candidate.1 (2026-08-04)

### 1.2 Compiler Crates Status (from existing IMPLEMENTATION_PLAN.md)

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

---

## 2. Specification vs Implementation Gaps (Candidate 1)

### 2.1 Parser Gaps (CRITICAL)

#### Gap 2.1.1: `let_expr` and `deref_expr` Productions
- **Specification**: GRAM-0001 names `grammar/omni-edition1.ebnf` as the normative grammar; `let_expr` and `deref_expr` are referenced in prose but have no EBNF productions
- **Current status**: Parser references `let_expr` and `deref_expr` without corresponding grammar productions
- **Impact**: Parser cannot fully parse Edition 1 source containing `let` expressions and dereference expressions
- **Resolution**: 
  1. Add `let_expr` production to the Edition 1 EBNF grammar
  2. Add `deref_expr` production to the Edition 1 EBNF grammar
  3. Update parser to use the new productions
  4. Ensure round-trip source parsing and reconstruction

#### Gap 2.1.2: Effect/Capability Bounds Undefined
- **Specification**: Effect/capability bounds have no definitions in the normative prose for the parser layer
- **Current status**: `omni-effects` has the effect enum defined (EFF-0001 to EFF-0018), but parser/type checker lacks bounds definitions
- **Impact**: Type checking of effect-polymorphic functions is incomplete
- **Resolution**:
  1. Define effect/capability bound syntax in the parser
  2. Add type checker rules for effect/ capability constraints
  3. Implement effect row solving in the type checker (building on existing EFF-0001..EFF-0018 framework)

#### Gap 2.1.3: Trait Function Signatures Undefined
- **Specification**: Trait function signatures are undefined in the normative EBNF (noted in parser comments)
- **Current status**: Trait function signatures exist in the type system but lack definitive grammar productions
- **Impact**: Trait method parsing and checking may have ambiguities
- **Resolution**:
  1. Define trait function signature grammar in the EBNF
  2. Ensure consistent parsing with the type checker
  3. Add corresponding test cases

### 2.2 Source Representation Gaps

#### Gap 2.2.1: SRC-0004 Unicode Version Pinning
- **Specification**: Edition 1 pins Unicode 17.0.0 with specific UAX standards
- **Current status**: Need to verify Unicode version pinning in source module
- **Impact**: Source tokenization may use different Unicode data than specified
- **Resolution**: Verify and implement Unicode 17.0.0 pinning with UAX #15, #31, #9, #24, #44, #39 Rev 32

#### Gap 2.2.2: SRC-0005 Bidi/Control Characters
- **Specification**: Outside comments and literals, bidi controls, noncharacters, unassigned code points, variation selectors, and default-ignorable format characters are source errors
- **Current status**: Scanner handles some of these cases but needs verification
- **Impact**: Malformed source may not be properly rejected
- **Resolution**: Verify scanner error handling for all SRC-0005 prohibited kinds

#### Gap 2.2.3: SRC-0011 Generated Source
- **Specification**: Generated source SHALL record generator artifact digest, declared inputs, output path identity, and source map
- **Current status**: Need to verify generated source tracking
- **Impact**: Source identity and reproducibility may be compromised
- **Resolution**: Implement source map and artifact digest tracking for generated source

### 2.3 Lexical Grammar Gaps

#### Gap 2.3.1: LEX-0007 Escape Sequences
- **Specification**: Escape sequences: `\0`, `\t`, `\n`, `\r`, `\"`, `\'`, `\\`, `\xNN`, and `\u{H...}` with one to six hex digits
- **Current status**: Scanner implements escape processing but needs verification against all escape sequences
- **Impact**: Invalid escape sequences may not be properly rejected
- **Resolution**: Verify all escape sequence handling in scanner

#### Gap 2.3.2: LEX-0009 Raw Strings
- **Specification**: Raw strings use `r"..."` or `r#*"..."#*` with matching hash count from zero through 255. No escapes are processed.
- **Current status**: Raw string handling exists but needs verification
- **Impact**: Raw string parsing may have edge cases
- **Resolution**: Verify raw string span tracking and hash count handling

#### Gap 2.3.3: LEX-0010 Interpolated Strings
- **Specification**: Interpolated strings use `f"text ${ expression } text"`; braces in text are escaped as `{{` and `}}`. Interpolation is tokenized recursively with balanced delimiters.
- **Current status**: Interpolation handling exists but needs full verification
- **Impact**: Interpolated string parsing may have edge cases with nested braces
- **Resolution**: Verify interpolated string tokenization and bracket balancing

### 2.4 Syntactic Grammar Gaps

#### Gap 2.4.1: GRAM-0003 Newline Termination
- **Specification**: "Newlines are not grammar terminals and cannot change a parse [in Edition 1 strict conformance]."
- **Current status**: Parser handles newlines but needs to ensure strict conformance (no automatic semicolon insertion)
- **Impact**: Code relying on automatic semicolon insertion may fail or behave unexpectedly
- **Resolution**: Ensure parser never examines line breaks to decide statement termination

#### Gap 2.4.2: GRAM-0006 Generic Arguments Context
- **Specification**: Expression generic arguments after a path or method use explicit `::` introducer where needed to avoid comparison ambiguity
- **Current status**: Generic argument parsing needs verification for context-sensitive `>>` splitting
- **Impact**: Generic type parsing may have ambiguity errors
- **Resolution**: Implement context-sensitive `>>` splitting as documented

### 2.5 Ownership/Gaps (already partially implemented)

#### Gap 2.5.1: Ownership Error Messages
- **Specification**: OMNI-OWN rules for affine ownership with borrowing and regions
- **Current status**: `omni-verify/src/ownership.rs` exists with flow analysis, but error messages can be improved
- **Impact**: Development experience for ownership errors
- **Resolution**: Improve error messages in ownership verification (as noted in recent commits)

---

## 3. Resolution Plan

### 3.1 Phase 1: Critical Parser Fixes (Weeks 1-2)
- [ ] Add `let_expr` EBNF production
- [ ] Add `deref_expr` EBNF production
- [ ] Update parser to use new productions
- [ ] Add round-trip tests for `let` and deref expressions
- [ ] Fix: Ensure parser never uses newlines for statement termination (GRAM-0003)

### 3.2 Phase 2: Effect/Capability Bounds (Weeks 2-3)
- [ ] Define effect/capability bound syntax in parser
- [ ] Add type checker rules for effect constraints
- [ ] Implement effect row solving for function signatures
- [ ] Add tests for effect-polymorphic functions

### 3.3 Phase 3: Trait Signatures (Weeks 3-4)
- [ ] Define trait function signature grammar
- [ ] Ensure consistent parsing with type checker
- [ ] Add trait method round-trip tests

### 3.4 Phase 4: Source Representation Verification (Weeks 4-5)
- [ ] Verify Unicode 17.0.0 pinning with all UAX standards
- [ ] Verify bidi/control character rejection (SRC-0005)
- [ ] Implement generated source artifact tracking (SRC-0011)

### 3.5 Phase 5: Lexical Verification (Weeks 5-6)
- [ ] Verify all escape sequences (LEX-0007)
- [ ] Verify raw string handling (LEX-0009)
- [ ] Verify interpolated string tokenization (LEX-0010)

### 3.6 Phase 6: Syntactic Grammar (Weeks 6-7)
- [ ] Verify newline handling (GRAM-0003)
- [ ] Verify generic argument `>>` splitting (GRAM-0006)

### 3.7 Phase 7: Ownership Polish (Weeks 7-8)
- [ ] Improve ownership error messages
- [ ] Add drop logic improvements
- [ ] Enhance self-reborrow test expectations

---

## 4. Specification Conflict Resolution

### 4.1 No Active Contradictions Within Candidate 1
After thorough analysis of all specification files in the codebase:

- **Candidate 1.0.0-candidate.1** (the normative baseline) has no internal contradictions
- **All documented conflicts are with Candidate 2 Vibe Surface Syntax** amendments, which have:
  - `status: pre-release`
  - `signature_state: pending` (not signed)
  - `implementation_impact: "none: no implementation consumes this overlay"`
  - Not applied per `ROOT-0002`: "No implementation... may override normative language text or formal rules"

- **Candidate 3** amendments are also not in force ("not ratified or implementation-certified")
- The repository correctly handles conflicts by:
  - Recording them in `grammar-reconciliation.md`
  - Not applying unsigned overlays
  - Keeping the Candidate 1 baseline as the governing specification
  - Per `ROOT-0002`, no implementation may choose between conflicting authorities

### 4.2 Resolution Approach
Since the user instructed to "keep a singular specs file for the entire codebase, no need to differentiate into multiple files":

- **Use Edition 1 Candidate 1.0.0-candidate.1 as the singular baseline**
- Candidate 2 and Candidate 3 amendments are documented as future/optional work
- The implementation targets Candidate 1 features solidly
- Any Candidate 3 changes would require a separate, coordinated amendment process
- This avoids fragmenting the codebase into multiple specification versions

---

## 5. Implementation Priorities

### Priority 1: Critical (Blocks compiler completion)
1. Add `let_expr` and `deref_expr` EBNF productions
2. Update parser to use new productions
3. Ensure GRAM-0003 newline conformance (no automatic semicolon insertion)

### Priority 2: High (Enables effect-polymorphic types)
4. Define effect/capability bound syntax
5. Implement effect row solving in type checker
6. Add effect-polymorphic function tests

### Priority 3: Medium (Improves developer experience)
7. Improve ownership error messages
8. Verify all SRC rules against implementation
9. Verify all LEX rules against implementation

### Priority 4: Lower (Completeness)
10. Define trait function signature grammar
11. Verify GRAM-0006 generic argument splitting
12. Add comprehensive round-trip tests

---

## 6. Verification Checklist

After implementing the above, verify:

- [ ] All `SRC-0001` to `SRC-0011` rules satisfied
- [ ] All `LEX-0001` to `LEX-0011` rules satisfied
- [ ] All `GRAM-0001` to `GRAM-0008` rules satisfied
- [ ] All `OWN-*` ownership rules satisfied
- [ ] All `EFF-0001` to `EFF-0018` effect rules satisfied
- [ ] Parser round-trip: `parse_source().syntax().text() == source` for test corpus
- [ ] No ambiguous parse errors for well-formed Edition 1 source
- [ ] Candidate 2 `/ Candidate 3 features properly gated/absent in Edition 1 strict conformance
- [ ] All existing tests pass

---

## 7. Notes on Candidate 2/3 Amendments (Documentation Only)

### Candidate 2 Vibe Surface Syntax
- Status: Pre-release overlay, not signed, not in force
- Key changes NOT implemented (and not for Edition 1 strict conformance):
  - Pipeline operator `|>` as standard token
  - Newline termination changes (ERR3-0020)
  - Statement continuation rules (ERR3-0021-0027)
  - Projection shorthand (ERR3-0033)
  - Command call changes (ERR3-0035)
- These are documented for future consideration but do not affect the core implementation

### Candidate 3 Vibe-First
- Status: Not ratified, not implementation-certified
- Key differences from Candidate 1:
  - Different statement termination model (newlines as statement terminators vs. `;` optional terminators)
  - Enhanced interpolation rules
  - Different keyword classification (contextual vs. hard-reserved)
  - Different comment syntax
  - Different number parsing
- These are documented for future consideration but do not affect the core implementation

### Maintaining Singular Specs File
As instructed, this plan uses **Edition 1 Candidate 1.0.0-candidate.1** as the singular baseline. The Candidate 2 and Candidate 3 amendments are tracked in separate documentation (`grammar-reconciliation.md`) but are not integrated into the main implementation branch. This avoids codebase fragmentation while still allowing future amendment integration when properly signed and ratified.