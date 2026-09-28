# Changelog

All notable project changes are documented here. Entries describe repository state and qualified implementation work; an entry does not imply a published release unless explicitly identified as released.

## [Unreleased]

### 0.0.1.x — Source normalization and lexer

The lexer work currently on `main` is qualified by the repository's GitHub Actions suite, but `0.0.1.x` is not yet declared complete.

- Hardened lossless byte-oriented lexing while preserving exact source spans.
- Added malformed UTF-8 preflight and deterministic lexical error spans, including malformed input encountered inside comments, raw strings, and interpolation paths.
- Preserved an optional UTF-8 BOM only at byte offset zero as recoverable trivia and retained byte-accurate source coverage.
- Kept CRLF/CR normalization in the source cursor while preserving original byte offsets.
- Completed Edition 1 punctuation/maximal-munch coverage for shift, shift-assignment, compound-assignment, unary `~`, optional chaining `?.`, and related operators.
- Added parser-side lossless splitting of `>>` into two `>` tokens for generic-argument contexts without changing source offsets.
- Removed the obsolete fixed raw-string delimiter limit and hardened nested raw/interpolated-string scanning.
- Corrected nested block-comment ownership, malformed-comment reconstruction, interpolation comment error propagation, and related regression fixtures.
- Expanded lexer acceptance/rejection coverage for malformed UTF-8, BOM placement, line endings, identifiers, raw identifiers, literals, comments, operators, large raw delimiters, and adversarial numeric forms.
- Updated the implementation master plan to record the completed lexer lexical contracts and their qualification boundary.

### Qualification

- GitHub Actions qualification for the merged lexer work passed Rust formatting, strict Clippy, workspace checks/tests, locked metadata, specification binding, Reaper/linkage checks, compiler topology validation, the Foundation `0.0.0.13` gate, Security Audit, and CodeQL.
- The qualifying merge is `9e3d8ce1aa0174e6b08dd38736c7cded582cc864`.

### Remaining 0.0.1.x qualification work

- `SRC-0003` requires NFC-based identifier equality; the current name-resolution implementation still compares raw `String` keys and does not yet demonstrate this normalization contract.
- `SRC-0004`/`SRC-0005`/`SRC-0006` require the specified pinned Unicode/security handling for bidi controls, noncharacters, unassigned code points, variation selectors, and default-ignorable format characters; the current repository does not yet demonstrate the required enforcement/data path.
- These items remain qualification work and are intentionally not recorded as completed.

## Foundation 0.0.0.x — qualified, unreleased

The foundation sequence `OMNI-IMP-0.0.0.1` through `OMNI-IMP-0.0.0.13` established the deterministic repository and specification containment boundary.

### Added

- Workspace topology and global safety/lint contracts.
- Deterministic Rust 1.95.0 toolchain pinning and required target definitions.
- CI Reaper, quality controls, hard-timeout/fuzz budgets, and fail-closed qualification plumbing.
- Normative rule-registry schema, lifecycle locks, specification manifest binding, and deterministic specification-tree canonicalization.
- Semantic linkage Reaper and compiler/tooling topology validation.
- Evidence schemas for diagnostics, witnesses, conformance outcomes, verification failures, provenance, regressions, and fuzz promotion.
- Typed specification loading and integrity verification through `omni-registry`.
- Stage-0 feature predicate infrastructure and manifest predicate integrity checks.
- Canonical provenance construction/emission with specification, toolchain, source, target, and artifact identity.

### Corrected and hardened

- Repaired extraction/validation of digit-bearing `STAGE0-*` rule identifiers.
- Corrected lifecycle handling for `ErratumCorrected`.
- Bound `registry/rule-texts.json` mechanically to the normative rule corpus.
- Recorded the Candidate-2 grammar erratum and precedence relationship without inventing productions.
- Hardened specification discovery, CWD independence, layout checks, and outside-tree rejection.
- Hardened target/host provenance labeling and manifest/model/data containment rules.
- Removed an obsolete layer-skipping machine dependency and unused driver path dependencies.
- Completed Foundation `0.0.0.13` release-gate qualification in the repository evidence.

## MIR, verifier, native backend, and driver hardening

### Added and hardened

- Typed MIR lowering and native-call ABI identity checks.
- MIR semantic verification, including parameter/local aliasing and cleanup-edge validation.
- Production source-to-native driver integration through frontend, semantic checking, monomorphization, typed MIR, verification, and native emission.
- Cranelift CFG/SSA and ABI lowering hardening driven from verified MIR.
- Regression fixtures for verifier-invalid cleanup edges and native type identity mismatches.

### Qualification

- The integrated MIR/backend/driver work was merged through the repository's reviewed pull requests and subsequently included in the green qualification state of `main`.

## Repository process and specification discipline

- The normative specification under `spec/` and the final micro-atomic implementation master plan remain the authoritative sources for language semantics and milestone ordering.
- Changelog entries distinguish qualified repository work from work that is still pending qualification or has not been released.
- No published release or `0.0.0.13` tag is implied by these entries.
