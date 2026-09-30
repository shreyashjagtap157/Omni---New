# Changelog

All notable project changes are documented here. Entries describe repository state and qualified implementation work; an entry does not imply a published release unless explicitly identified as released.

## [Unreleased]

### 0.0.2.2 — Lossless Concrete Syntax Tree

The CST substrate is implemented and CI-qualified on `main`. This milestone establishes the CST's contracts and invariants; it deliberately does **not** expand Edition 1 grammar coverage, which remains 0.0.2.3's work.

- **Fixed a losslessness defect in error recovery.** `bump_or_dummy` returned `tokens.len() - 1` as a "dummy" index, which is the EOF token. Every unclosed construct therefore re-emitted EOF, and because EOF carries the trailing trivia, that trivia was emitted a second time: `"fn f() {\n// c"` reconstructed as `"fn f() {\n// c\n// c"`. Recovery now yields a distinct zero-width `MissingToken` instead, so an absent token is represented without fabricating or duplicating source bytes.
- Added `SyntaxKind::MissingToken`, a zero-width placeholder distinguishable from both a real token and from `ErrorNode` recovery, keeping GRAM-0007's recovery tagging intact.
- Added `SyntaxKind::Unknown` and `SyntaxKind::ALL`. Rowan's reverse conversion is total, and it previously mapped *every* unrecognized raw kind to `ErrorNode` — which is the parser's recovery tag, so tooling could read a malformed tree as a recovered one. Unknown kinds now map to `Unknown`. Kinds were appended only; no existing discriminant was renumbered.
- Established a mechanically testable accounting invariant: the CST's leaves must exactly tile the source, with no gap and no overlap. String equality alone cannot distinguish a correct tree from one that drops a token and re-emits an identical-looking one elsewhere.
- Added a randomized differential suite (20,000 generated inputs, deterministic seed in-file) plus degenerate, adversarial-malformed, and trivia-only suites. Losslessness is asserted on both axes: malformed input must remain lossless **and** remain diagnosed, so the property cannot be satisfied by accepting everything.
- Added an independent adversarial suite (`compiler/omni-parse/tests/cst_adversarial.rs`, 11 tests) that re-derives every expectation from the public API and the lexer's own spans rather than reusing the implementation's own helpers, so a defect in the parser's internal accounting cannot mask itself. It additionally pins properties the in-crate suite did not cover: leaves must land on UTF-8 character boundaries; every zero-width leaf must be an absent-token marker; trivia count *and* trivia byte total must equal the lexer's own accounting exactly (the suite that motivated 0.0.2.2 checked neither); deterministic shape, text, and diagnostics across repeated and re-parsed runs; termination under 300–400-deep nesting; a 2,000-function long source; and a 3,000-case arbitrary-byte soup exercising the SRC-0001 malformed-UTF-8 path.
- Confirmed the suite is non-vacuous by deliberate mutation rather than by assertion: reintroducing the historical EOF-aliasing defect in `missing()` failed 4 in-crate tests and 4 independent ones; reintroducing the unknown-kind-as-`ErrorNode` mapping failed `an_unknown_raw_kind_is_not_an_error_node`; and dropping the first token skipped by `synchronize_top` — the original loss-of-source regression — failed 4 tests. All mutations were reverted; no committed implementation code was left modified.
- Recorded the CST contracts as established facts: exact source reconstruction for all input including malformed and recovery input; exact token, trivia, and interval tiling with no gap and no overlap; EOF emitted exactly once; `MissingToken` zero-width and unable to alias a source-bearing token; deterministic tree shape and diagnostics; byte-exact spans with no UTF-16 or scalar indexing.
- Left `grammar_contract.rs`, `docs/grammar-reconciliation.md`, the normative EBNF, the specification digest, and the Candidate 2 feature-gate state unchanged.

### 0.0.2.3 parser implementation

- Replaced the remaining top-level item stub path with real Edition 1 parsing for functions, structs, enums, type aliases, const/static declarations, use/module/extern-crate declarations, attributes, and nested module items.
- Implemented generic parameters/arguments, lifetime boundaries, paths, core type forms, where clauses, trait references, nested generic closer handling for `>`, `>>`, and `>>=`, and source-relative logical token pieces without losing source bytes or trivia.
- Implemented block expressions with final-expression semantics, statement/item parsing, control-flow expressions, patterns, postfix calls/fields/indexing/method calls/await, casts, ranges, all currently lexed Edition 1 binary and assignment operators, closures, async/unsafe/try blocks, and nested macro token trees.
- Preserved Candidate 2 as disabled.
- Left undefined normative productions and the trait/impl separator/function-signature inconsistencies explicitly blocked rather than inventing grammar.

### Current 0.0.2.3 implementation status

The parser/CST baseline is now materially implemented beyond the historical snapshot above: block expressions, control flow, patterns, generic syntax, ranges, postfix expressions, casts, compound assignments, and the Edition 1 binary/unary operator family are represented and lowered into the semantic AST.

#### Qualification gate clearance on `17c7f88`

The 0.0.2.3 baseline was the point from which this wave proceeded. Three independent CI-blocking defects were corrected, and the workspace test suite is now green apart from environmental failures. The result is **469 passed, 8 failed**, up from **464 passed, 13 failed** at the baseline.

- **Cleared the Rust 1.95 formatting gate.** `cargo fmt --all --check` exited non-zero on `compiler/omni-driver/src/main.rs` and `compiler/omni-mir/src/lower.rs`. Both were pure layout: a line-width and brace-shape reflow of the compound-assignment operator match, and one stray blank line. No behaviour changed. The check now exits 0.
- **Settled the lifetime/character lexical boundary.** The grammar admits two productions under one apostrophe introducer: `char_literal = "'" (escape_sequence | non_quote_char) "'"`, which requires a closing quote, and `lifetime = "'" identifier`, which does not. Bare `'a` is therefore genuinely ambiguous at the lexical layer. The repository had already recorded a resolution in `docs/grammar-reconciliation.md` under *Lifetime lexer/parser boundary correction*: a quote followed immediately by an identifier start lexes as `Punct::Apostrophe` plus the ordinary identifier, while a quote followed by trivia or by a quote stays on the character-literal path. That is the later of the two recorded positions, it is what the scanner implements, and it is the only reading that is both lossless and lets the parser build a `Lifetime` node. The failing test still encoded the earlier, superseded contract, and was in fact unsatisfiable: it asserted the same input with the same expected kind twice, once as a single `Error` token spanning bytes 0..2 and again as `tokens[0] == Error`. The test now pins the recorded contract and additionally covers two cases it never did: a quote followed by trivia still fails closed as a lexical error, and a multi-character run remains one `Error` token rather than being reclassified as a lifetime. `omni-lex` is now 88/88.
- **Fixed ownership projection initialization.** `OwnershipState::state` consulted moved ancestors and moved descendants but never an initialized ancestor, so a sub-place such as `x.a` had no entry of its own and fell through to `Uninitialized` after `declare_initialized("x")`, making it impossible to read or move a field. The new rule is ordered so a moved ancestor still wins and an aggregate with a moved field still reports `PartiallyMoved`. `omni-own` is now 7/7.
- **Fixed tuple-index typing.** The checker accepted `Expr::Field` only when the base was a struct, so the `t.0` projection form was rejected even though MIR lowering already resolves it against the tuple element types with an explicit bounds check. The checker now resolves numeric tuple fields with the same bounds check; a non-numeric field and an out-of-range index remain errors.
- **Fixed unary operand diagnostics.** `Neg` and `BitNot` reported `UnsupportedOperator` for a wrong operand while `Not` reported `MismatchedTypes`. `MismatchedTypes` is the correct variant, since the operator is well defined and only the operand type is unacceptable, and it is the variant that names the offending type. `Deref` keeps `UnsupportedOperator`, because dereferencing a non-reference is a missing structural precondition rather than a mismatch between two valid types.

**The 8 remaining failures are entirely environmental, not compiler defects.** Each one panics on `Command::new("cc")` returning `NotFound` before it reaches any assertion about compiler behaviour, because this environment has no system C linker. They are the native end-to-end tests (integer call semantics, integer `for` loop, inclusive range `for` loop, float arithmetic and comparison, boolean comparison and `not`, explicit generic call, unit call without a fabricated result) plus the Cranelift CFG-join SSA test. They are not evidence of broken language semantics, and no compiler code was changed to make them pass.

The semantic/MIR path now additionally covers typed `if`/labelled loops, `break`/`continue`, integer-range `for` lowering, explicit generic-call arguments, scalar casts, structured `let` patterns, match-arm-local bindings, definition-directed field/index typing, floating arithmetic/comparisons, struct literals, and enum constructors at the semantic layer.

The semantic/MIR path now additionally covers typed `if`/labelled loops, `break`/`continue`, integer-range `for` lowering, explicit generic-call arguments, scalar casts, structured `let` patterns, match-arm-local bindings, definition-directed field/index typing, floating arithmetic/comparisons, struct literals, and enum constructors at the semantic layer.

The current native MIR boundary remains intentionally fail-closed for aggregate storage/projection that the IR does not yet model: struct/enum construction, aggregate destructuring, and guarded or aggregate-pattern match execution are rejected rather than silently reinterpreted. Qualification is therefore not a release claim; the live repository is the source of truth and the current GitHub mainline has no associated pull-request workflow run for these commits.

#### Workspace repair and labelled-loop/control-flow closure

`main` did not compile: `omni-types` failed with 9 errors, and because the build stops at the first failing crate, roughly 450 further errors across `omni-mir`, `omni-verify`, `omni-parse`, `omni-own`, and `omni-driver` were masked behind it. The whole workspace now builds, and the test suite compiles and runs.

- **Repaired three corrupted merge artifacts that caused the cascade.** A stray `0` prefixed to `fn parse_closure_expr` in `omni-parse/src/parser.rs` broke the `impl Parser` item list and produced 399 downstream errors from that one character; an orphaned `&self,` fragment sat past EOF in `omni-mir/src/lower.rs` (brace counts balanced 512/512, so it was provably dead text); and a duplicated `#[derive(Debug, Clone)]` on `omni-mir`'s `AggregateKind` produced conflicting `Clone`/`Debug` impls.
- **Restored `bump_child` and implemented `consume_gt` in the parser.** Both were called ~118 times but neither was ever defined in any commit, so the crate could not compile. `bump_child` was recovered from its surviving doc comment and commit `3e0c167`; `consume_gt` consumes a logical `>`, splitting a lexed `>>`/`>>=` through the existing `split_generic_closer` authority rather than inventing a second splitting rule.
- **Closed the labelled-loop and control-flow gaps.** `loop`, `while` and `for` are statements in Edition 1 and take no trailing `;`; they previously fell through to the expression-statement path and demanded one. A `for` iterable, a `while` condition and a `match` scrutinee are each followed by their `{`, which was being read as a struct literal on the preceding path — so `for y in x { .. }` parsed `x { .. }` as a struct expression. A scoped `no_struct_literal` flag now covers exactly those three positions. `labeled_loop_forms_preserve_label_and_colon`, `labeled_loop_control_is_parsed_losslessly`, `edition1_control_flow_and_block_expressions_are_accepted` and `final_block_expressions_do_not_require_semicolons` all pass.
- **Fixed a losslessness defect in postfix field and method access.** The consumed `.` was pushed only in the `await` branch and silently dropped in the field, method-call and turbofish-field branches, so `x.a` reconstructed as `xa`. The dot is now retained in every branch.
- Threaded struct declarations into MIR lowering additively (`LoweringContext::set_struct_defs` plus `compile_monomorphized_program_with_structs`) so field projections can be typed, keeping the existing public signatures and their 23 call sites intact. Field types are now resolved from the instantiated type arguments rather than the function-level substitution environment, matching the checker's behaviour.
- **Previously known nested-generic closer defect is resolved.** `>>`/`>>=` are now surfaced as source-relative `Child::Piece` leaves; after generic closers are consumed, the remaining `=` from `>>=` is retired through the same logical-token cursor and can be consumed normally. Regression coverage verifies both lossless reconstruction and forward progress.

#### Split-token entry point, control-flow lowering, and resolver CST alignment

The nested-generic-closer work above did not compile: commit `4784095` replaced the body of `consume_gt` with the new `bump_split_piece` helper and left a bare `false` token where the entry point belonged, so `omni-parse` failed with 243 errors from a single structural break and no test in the workspace could run. The declared baseline was therefore not buildable, which is why the pre-existing failures below were never observed.

- **Wrote the missing `consume_gt` entry point.** `bump_split_piece` only serves a split that is *already* active; nothing initiated one, so no nested generic list could ever close. `consume_gt` now consumes a lone `>`, or *starts* a split and serves only the first logical `>`, leaving the remainder for the enclosing list. Only `>` pieces are consumed, so the trailing `=` of `>>=` stays an ordinary logical token.
- **Fixed the type-argument lookahead gate.** `looks_like_type_args` required `depth >= 2` for a `>>` closer, so the *innermost* list — which legitimately closes on the first `>` of the token — was never recognised. The scanner now tracks pending logical closers within the current token and, when a `>` piece is still owed, consults that piece rather than the next physical token. Fixing the continuation check is what removed the stack overflow that a naive fix reintroduced.
- **Restored a dropped `.` in field-expression recovery.** The error path taken when a field name is missing (`x. 5`) discarded the already-consumed dot, so `return1.51` reconstructed as `return151`. Every branch now retains the dot, preserving the 0.0.2.2 exact-reconstruction guarantee on malformed input.
- **Added `Parser::peek_token`.** The existing cursor-valued `peek()` cannot answer "the token at offset 1", which the trivia-boundary test needs; adding the offset-valued counterpart avoids distorting the test or the cursor contract.
- **Corrected a trivia-ownership expectation.** This scanner attaches every trivia run as *trailing* trivia of the token it follows, so `leading_trivia` is uniformly empty. The test asserted a leading/trailing split the lexer never produces; it now pins the invariant that actually matters for losslessness — trivia is carried exactly once in total — and a new test pins that a split `>>=` tiles its physical span exactly.
- **Reclassified `1 + * 2` as well-formed.** `unary_op` includes `"*"`, so the expression is derivable, and the `deref_expr` production that would restrict the operand to a place is recorded as an unresolved specification hole in `docs/grammar-reconciliation.md`. Rejecting it in the parser would resolve a specification question by implementation fiat, so the adversarial case was replaced with a test pinning the normative position, alongside a genuine consecutive-prefix-operator case.

- **Lowered `break`/`continue` in MIR.** Both were fully implemented (`lower_break_expression`, `lower_continue_expression`) but absent from the `lower_expr` dispatch, so every labelled and unlabelled loop failed with "Unsupported AST expression form". Dispatch arms now route them, and `omni-mir` is fully green.
- **Separated divergence from block termination in MIR lowering.** `Expr::Block` stopped at `current_block.is_none()`, which `break`/`continue` also set, so a `break` after a `continue` was silently dropped. A distinct `diverged` flag now marks a function-level `return`, and statements following a loop-scoped transfer are lowered into a fresh block. The flag is saved and restored around `if` branches, loop bodies and match arms, because a `return` inside one branch does not make the other branches — or the code after the construct — unreachable.
- **Fixed a `match` CFG hole.** When an arm supplied the wildcard, the implicit `otherwise` block was left with no terminator, and the verifier rejected the function with "BasicBlock N lacks a valid terminator". That block is now always terminated.
- **Admitted `Never` as a function return type in MIR.** A diverging body (an infinite `loop`) produced no value, which previously failed unless the declared return type was `Unit`.
- **Aligned the resolver with the current CST.** `let` and match-arm bindings are parsed as `IdentifierPattern`/`BindingPattern > Path > PathSegment`, and ordinary identifier references as `PathExpr > Path > PathSegment`, but the resolver looked only for `NameRef` — so *no* local was ever declared and *no* reference was ever recorded. Bindings now resolve through the pattern's path segment, single-segment expression paths resolve as references, `for` introduces its pattern binding in a body-scoped rib, and struct-constructor paths plus struct-literal field labels are excluded as non-runtime names. A turbofish is stripped from the segment text so `id<i64>(41)` resolves to `id`.
- **Lowered loop statements in the driver.** `loop`, `while` and `for` are emitted as bare loop nodes rather than wrapped in `ExprStmt`; the driver's block dispatcher rejected them as unsupported statements.
- **Scoped the struct-literal guard to `if` conditions too.** `if a { .. }` read the `{` as a struct literal on the condition, the same ambiguity already handled for `while`, `for` and `match`.
- **Fixed two stale test fixtures rather than weakening the parser.** `expr_stmt = expression ";"` means only a block's trailing `final_expression` may omit the semicolon, so `if c { .. }; return 0;` requires one; and the E2E harness links `main` with no arguments, so the one fixture declaring `main(x: i64)` was rewritten to use a helper like every other end-to-end test.
- **Replaced two obsolete `omni-mir` tests.** Commit `1f0eaee` deliberately removed the fail-closed arms for struct and enum construction and implemented real typed MIR for both, but left the older tests asserting the removed errors. They now assert the constructor and its declared type, and the aggregate layout boundary remains where it actually is: `omni-codegen`, which refuses to emit native code for `Rvalue::Struct`/`EnumVariant` without target layout metadata.

Workspace tests now run: 462 passing, 14 failing. All 14 pre-date this work and were unreachable while the workspace did not compile. Six require a system C linker (`cc`/`gcc`/`clang`), which is not installed in this environment; the remainder are an `omni-own` projection-initialization defect, a tuple-index monomorphization defect, a unary-operand type expectation, and one reference-storage fixture that still fails closed but at the type layer rather than the documented storage boundary. None are regressions from the changes above.

### 0.0.1.x — Source normalization, lexer, and source security

The `0.0.1.x` source/lexer boundary is implemented and CI-qualified on `main`. This is repository state, not a release: no version has been published, no tag has been cut, and the Foundation release gate remains **not declared and not published**.

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

### Qualification (lexer contracts)

- GitHub Actions qualification for the merged lexer work passed Rust formatting, strict Clippy, workspace checks/tests, locked metadata, specification binding, Reaper/linkage checks, compiler topology validation, the Foundation `0.0.0.13` gate, Security Audit, and CodeQL.
- The qualifying merge is `9e3d8ce1aa0174e6b08dd38736c7cded582cc864`.

### Remaining 0.0.1.x qualification work

None. `SRC-0003` through `SRC-0006` are implemented, covered, and CI-qualified; see the entries below.

### Source security and identifier canonicalization (SRC-0003 through SRC-0006)

A new leaf-ward crate, `compiler/omni-unicode`, is the single authoritative
implementation of the OMNI-SOURCE Unicode obligations. Nothing else in the
workspace re-derives an answer to "is this code point prohibited here?" or "what
is this identifier's canonical key?".

- **`SRC-0003` — NFC identifier equality.** `CanonicalName` carries both the
  original source spelling and the NFC canonical key, so name resolution,
  shadowing detection, and definition identity compare canonically while
  diagnostics, source fidelity, and CST reconstruction keep the bytes the source
  actually contained. Strings, character literals, raw strings, and comments are
  never normalized as a side effect.
- **`SRC-0004` — pinned Unicode data identity.** Edition 1 pins Unicode 17.0.0.
  Property tables and a 961-case NFC conformance corpus are generated by
  `scripts/generate-unicode-tables.py` from the authoritative UCD
  17.0.0 `PropList.txt`, `DerivedCoreProperties.txt`,
  `DerivedNormalizationProps.txt`, and `UnicodeData.txt`, and the SHA-256 of
  each input is compiled into `omni_unicode::tables` and asserted by tests. The
  compiler never reads the UCD at build or run time, performs no network access,
  and consults no host locale.
- **`SRC-0005` — source-security validation.** Outside comments and literals,
  bidi controls, noncharacters, unassigned code points, variation selectors, and
  default-ignorable format characters are source errors, each reported with its
  exact byte span and the class SRC-0005 names. Malformed UTF-8 remains a
  distinct `SRC-0001` error and is never conflated with a prohibited code point
  or smoothed into U+FFFD. Classes are tested in the order SRC-0005 names them,
  so a code point in two classes is reported identically on every run.
- **`SRC-0006` — comment security.** Bidi controls and invisible format
  characters inside a comment require a visible escaped annotation, and strict
  mode — the default — rejects unannotated occurrences. The annotation reuses the
  normative `escape_sequence` production from LEX-0007 rather than inventing a
  second notation. Comment bytes are never discarded, replaced, or dropped from
  their span: rejection never means losing the comment.
- Error tokens now carry an `ErrorReason` (`MalformedUtf8`, `ProhibitedSource`,
  `UnannotatedCommentSecurity`, `Lexical`) so a consumer can distinguish a
  security finding from malformed input and from an ordinary lexical error.

### Dependency reconciliation

The `d9f0f55` lockfile-only addition of `unic 0.9.0` and `matches 0.1.10` is
removed. Neither was reachable from any manifest, `cargo build` deleted all 303
lines on a clean checkout (which also broke the clean-worktree precondition of
`ci/gate-0-0-0-13.sh`), and `unic 0.9.0` encodes Unicode 7.0.0 — ten versions
behind the SRC-0004 pin. The replacement is `unicode-normalization 0.1.25`,
pinned exactly, and its Unicode data is bound to UCD 17.0.0 by the generated NFC
conformance corpus. `Cargo.toml` and `Cargo.lock` now describe the same
dependency graph.

### Qualification (source security)

- Formatting (`cargo fmt --all -- --check`), strict Clippy
  (`-D warnings`, all targets), `cargo check`, and the full workspace test suite
  pass locally on Rust 1.95.0; `omni-conform`, `omni-audit`, and
  `omni-topology` all pass, with `omni-unicode` registered at pipeline tier 0.
- GitHub Actions qualification on the merged `main` commit passed every
  repository workflow: Omni CI Pipeline, Rust, rust-clippy analyze, Security
  Audit, and CodeQL Advanced. The CI log shows the Foundation `0.0.0.13` gate
  passing and the spec-tree digest `7e63431f…` verified against the manifest.
- The four native end-to-end tests that shell out to a system C linker
  (`omni-codegen` `test_cfg_join_uses_cranelift_variable_ssa` and three
  `omni-driver` `source_pipeline_*` tests) require `cc` and are therefore not
  runnable on the Windows development host, where they fail. They pass on the
  Linux CI runners, and they fail identically on the pre-change commit, so the
  failures are environmental rather than a regression.
- The source-security merge is `743843e5abc44e3822962fd20eb7338091006d5c`.

### Unicode provenance precision

The generated provenance constants now cover **every** UCD file the generator
consumes, including `DerivedNormalizationProps.txt`, which supplies the derived
`Full_Composition_Exclusion` property used to build the NFC conformance corpus.
The four digests are also emitted as a single `UCD_DIGESTS` table, and a test
asserts that the table lists exactly the consumed inputs and agrees with each
individual constant, so the "every input is hashed" claim is mechanically
enforced rather than asserted in prose.


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

### 0.0.2.1 - Grammar reconciliation

The Candidate 2 Vibe-First amendment is reconciled against the bound Edition 1
EBNF in `docs/grammar-reconciliation.md`. The outcome is that the EBNF is
**not** edited: the amendment's erratum overlay is `status: pre-release` with
`signature_state: pending` and declares `implementation_impact: "none"`, so
`REL-0007` does not permit applying it; `GRAM-0003` admits the newline exception
only under a Candidate 2 feature gate that is not enabled; and `spec/grammar/`
is inside `omni_canon::INCLUDED_DIRS`, so any edit would rebind
`spec_tree_sha256` in both the manifest and the Foundation gate, while a new
file there fails the gate closed as an undeclared artifact. `ROOT-0002` forbids
resolving the conflict by implementation fiat.

- Eleven conflicts between `VIBE-GRAM-0001`..`0007` and the in-force Edition 1
  rules are inventoried, each mapped to the EBNF production it affects, with the
  five policy actions required before the amendment can take effect.
- The Edition 1 productions the parser does not yet implement are recorded, so
  the 0.0.2.3 scope is explicit: `block_expr`, most binary operators,
  `struct_def`/`enum_def` bodies, `if`, `match`, `loop`, `while`, `for`,
  closures, arrays, tuples, and indexing.
- The grammar that is actually in force is now pinned by tests in
  `compiler/omni-parse/src/grammar_contract.rs`, each assertion citing its rule,
  so the parser cannot drift toward Candidate 2 behaviour before the gate is
  enabled.

### Parser: fixed token loss during error recovery

`Parser::synchronize_top` and `Parser::recover_until` advanced the token
position without recording the tokens they skipped, so malformed input lost
source text: `"fn main( { return 1; }"` reconstructed as `"fn main( { "`. This
broke the CST's defining `parse(source).text() == source` invariant, and the
existing recovery test only asserted that the text was non-empty, so it went
unnoticed. Both helpers now return the skipped indices and callers attach them
to an `ErrorNode`, which also gives recovery the `GRAM-0007` recovery-only tag
it previously lacked. Nine tests added, including a sixteen-input recovery
round-trip matrix and a check that round-tripping is not achieved by accepting
everything.
### Dependency maintenance: Dependabot PRs closed deliberately

The five open Dependabot PRs were reviewed and closed with a recorded rationale
rather than merged or left to rot, per the smallest-change rule in
`docs/toolchain-qualification.md`.

- **#27 `cranelift-codegen` 0.110.3 -> 0.136.1** and **#28 `cranelift-object`
  0.110.3 -> 0.135.3**: the four cranelift crates are pinned at a single `0.110`
  and release in lockstep, so either bump alone would split the family across
  incompatible versions, and neither proposes the matching `cranelift-frontend`
  or `cranelift-module` bump. The `Rust` workflow also fails. This needs one
  coordinated four-crate migration with its own qualification.
- **#30 `target-lexicon` 0.12.16 -> 0.13.5**: a transitive dependency of the
  pinned cranelift 0.110 stack, which declares the 0.12 series. `Rust` fails.
  It should move only with the cranelift migration.
- **#29 `sha2` 0.10.9 -> 0.11.0**: CI is green, but `sha2` produces the
  `spec_tree_sha256` and `plan_sha256` release identity recorded in the manifest
  and the Foundation gate. A major bump of the digest primitive is a change to
  release identity and belongs in a dedicated pass that re-verifies every
  recorded digest, not in routine maintenance.
- **#31 `object` 0.36.7 -> 0.40.0**: a major API change in the native
  object-emission path used by `omni-codegen`. `Rust` fails. It belongs with
  native-backend hardening.

No dependency version is changed by this work; `Cargo.lock` still matches
`Cargo.toml` and the Foundation gate digest is unchanged. Dependabot will
re-propose against the current `main`, at which point the cranelift migration
can be scheduled as its own qualified task.
### 0.0.2.3-A — Parser infrastructure and lexer/parser contract corrections

This is an infrastructure wave for 0.0.2.3, not Edition 1 parser completion.

- Added bounded parser lookahead, safe missing-token consumption, and delimiter pair helpers while retaining zero-width `MissingToken` recovery.
- Added source-relative generic-closer infrastructure for `>`, `>>`, and `>>=`. Logical parts cover the original operator bytes exactly; source offsets and trivia remain owned by the original lexer token.
- Removed the parser-local numeric precedence table: the parser now consults the shared Edition 1 precedence authority for its currently implemented operator subset, with explicit constants for unary and postfix binding strength. This does not enable the remaining binary/cast productions.
- Added regression coverage for EOF, empty input, missing tokens, delimiter matching, generic-closer byte coverage, generic-closer trivia boundaries, and deterministic lexer lifetime behavior.
- Updated the 0.0.2.2 adversarial lexical-error probe from `'x` to `'ab'` because `'x` is now a valid lifetime start; the test still exercises a malformed character literal.
- Corrected the lexer/parser lifetime boundary: a quote followed immediately by an identifier-start is now `Punct::Apostrophe` plus the ordinary identifier token, while valid character/byte literals remain unchanged and malformed character literals such as `'ab'` remain lexical errors. The parser does not claim lifetime grammar completion in this wave.
- Recorded the unresolved trait/impl item-list separator ambiguity and the separate undefined `function_signature` production referenced by `trait_item`. No implementation-defined interpretation was introduced.
- Candidate 2 remains disabled and the normative Edition 1 EBNF remains unchanged.

The complete Edition 1 parser remains future 0.0.2.3-B+ work.
