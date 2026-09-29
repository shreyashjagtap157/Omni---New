# 0.0.2.1 — Grammar reconciliation status

This record is the mechanical reconciliation of the bound Edition 1 EBNF with
the Candidate 2 Vibe-First amendment (`VIBE-GRAM-0001` … `VIBE-GRAM-0007`). It
exists because the repository's own `docs/foundation-gates.md` records an open
defect:

> Known specification inconsistency (open, not resolved by invention): the
> normative EBNF predates the Candidate-2 amendment and contains no
> pipeline-operator, newline-termination, projection-shorthand, or command-call
> productions, while `VIBE-GRAM-*` rules normatively require them.

That defect is real and it is **not** fixed by editing the EBNF. This document
records why, exactly what conflicts, what is in force today, and what has to
happen before any of it changes.

## Why the EBNF is not being edited

Three independent constraints each forbid rewriting `spec/grammar/omni-edition1.ebnf`
as part of the parser milestone:

1. **The amendment is not in force.** `spec/grammar/candidate2-erratum.md`
   declares `status: pre-release` and `signature_state: pending`, and states
   `implementation_impact: "none: no implementation consumes this overlay; the
   Candidate-1 baseline and grammar/omni-edition1.ebnf continue to govern every
   observable behavior"`. `REL-0007` requires an overlay to be *signed* before it
   is applied, and the loader rejects any attempt to mark it `applied`. Applying
   an unsigned overlay to the normative grammar would make the tree claim an
   authority it does not have.

2. **`GRAM-0003` gates the exception explicitly.** Its own text reads: "Newlines
   MAY serve as statement boundaries at unambiguous statement boundaries per
   Candidate 2 Vibe Surface Syntax amendment; **this exception does not apply to
   Edition 1 conformance claims without the Candidate 2 feature gate**." The
   feature gate is not enabled, so Edition 1 strict conformance still governs.

3. **Editing the file rebinds the release identity.**
   `spec/grammar/` is in `omni_canon::INCLUDED_DIRS`, so any change to it changes
   `spec_tree_sha256`, which is bound in **both**
   `spec/manifest/omni-edition1.manifest.json` and
   `spec/release/foundation-gate.json`. Worse, `omni-canon` fails closed on
   "any other file under an included directory [that] is an undeclared
   artifact", so simply *adding* an amended grammar file under `spec/grammar/`
   breaks the gate until it is declared and the digest is rebound.

`ROOT-0002` closes the last escape route: "No implementation, reference
interpreter, compiler, test, example, or prior draft may override normative
language text or formal rules; conflicts are specification defects that block
publication." The repository's own response to a conflict is to record it, not
to resolve it by fiat.

## The conflict inventory

`LEX-0002` and `GRAM-0002`/`GRAM-0003` are the Edition 1 baseline.
`VIBE-GRAM-0001` ... `0007` are the Candidate 2 amendment. Where they disagree,
the erratum precedence clause governs *within its scope* -- but its scope is not
yet in force.

| # | Candidate 2 rule | Edition 1 rule in force | EBNF production | State |
|---|---|---|---|---|
| 1 | `VIBE-GRAM-0001` newline MAY terminate at an unambiguous statement boundary | `LEX-0002` "Newlines never terminate statements" | `expr_stmt` | not in force |
| 2 | `VIBE-GRAM-0002` newline MUST NOT terminate when incomplete or next token is a continuation token | no Edition 1 counterpart | `expr_stmt`, `let_stmt` | not in force |
| 3 | `VIBE-GRAM-0003` normative continuation-token set: binary operators, `.`, `?.`, `?`, `??`, `,`, `)`, `]`, `}`, `::` | no Edition 1 counterpart | none | not in force |
| 4 | `VIBE-GRAM-0004` semicolons optional at ordinary boundaries | `GRAM-0002` non-block expression statement requires `;` | `expr_stmt`, `let_stmt`, `return_expr` | not in force |
| 5 | `VIBE-GRAM-0005` canonical formatter output; indentation must not alter semantics | no Edition 1 formatter rule | formatter | not in force |
| 6 | `VIBE-GRAM-0006` braced blocks valid everywhere blocks are admitted | already consistent in the EBNF | `block_expr` | already satisfied |
| 7 | `VIBE-GRAM-0007` release translation MUST reject ambiguous newline interpretation | no Edition 1 counterpart | whole grammar | not in force |
| 8 | pipeline `\|>` promoted from reserved punctuation by the amendment | `binary_op` has no `\|>` alternative | `binary_op`, `binary_expr` | not in force |
| 9 | projection shorthand `.field` as an implicit closure | no production | none | not in force |
| 10 | command-style call disambiguation | `call_expr` is parenthesized only | `call_expr` | not in force |
| 11 | optional chaining `?.` and null coalescing `??` | `??` is in `binary_op`; `?.` has no production | `binary_expr`, `field_expr` | partial |

`GRAM-0001` names `grammar/omni-edition1.ebnf` as the normative grammar, and
`GRAM-0008` requires feature-gated grammar to live in "a named edition or
profile namespace" and to be included in the source and artifact fingerprint.
Rows 1-5 and 8-10 therefore need **two** coordinated changes when the gate opens:
an amended grammar in a named namespace, and a fingerprint that covers it.

## What is in force today, and is implemented

The current parser implements the Edition 1 baseline. That is correct, and it is
pinned by `compiler/omni-parse/src/grammar_contract.rs` so that it cannot drift
silently while reconciliation is pending:

- newlines never terminate a statement, so a program written on one line and the
  same program split across lines parse identically;
- a non-block expression statement requires `;`;
- a block final expression yields the block value without `;` (`GRAM-0002`);
- assignment is right-associative, other binaries left-associative, and
  comparisons do not chain (`GRAM-0004`);
- `>>` is split into two `>` tokens in generic-argument context by the parser
  without changing source offsets (`LEX-0001`);
- error recovery is tagged and does not produce a translatable tree
  (`GRAM-0007`).

## What must happen before any of it changes

These are policy actions, not implementation tasks, and this milestone does not
take them unilaterally:

1. Sign the Candidate 2 erratum, or record a decision to ratify it, so that
   `signature_state` leaves `pending` and `status` leaves `pre-release`.
2. Enable the Candidate 2 feature gate named by `GRAM-0003`.
3. Produce the amended grammar as a **named namespace** artifact per `GRAM-0008`,
   declare it in `omni_canon::DECLARED_FILES`, and deliberately rebind
   `spec_tree_sha256` in both the manifest and the Foundation gate through the
   established process.
4. Fold the `SRC-0006` comment-security annotation syntax (recorded in
   `docs/foundation-gates.md` as an implementation choice reusing the `LEX-0007`
   `escape_sequence`) into that same amendment, so the surface language stops
   depending on a code-level decision.
5. Re-run the Foundation `0.0.0.13` gate and rebind its evidence.

Until then, `0.0.2.2` (lossless CST) and `0.0.2.3` (parser) proceed against the
Edition 1 baseline, which is exactly what the current qualified lexer emits.
## Defect found and fixed during this reconciliation

Writing the in-force contract above immediately produced a failure, and the
cause was a real bug rather than a bad expectation.

`Parser::synchronize_top` and `Parser::recover_until` advanced the token
position with `self.pos += 1` and returned nothing, so every token they skipped
was **dropped from the green tree**. `"fn main( { return 1; }"` reconstructed as
`"fn main( { "`: the `return`, `1`, `;` and `}` simply disappeared.

The lexer is lossless, and the parser's own `reports_and_recovers` test only
asserted `!r.syntax().text().is_empty()`, so the hole was invisible. The CST
contract is that `parse(source).text() == source`, so this was a defect in an
already-claimed property rather than missing milestone work.

Both helpers now return the skipped token indices, and the callers attach them
to an `ErrorNode` -- which also gives the recovery the `GRAM-0007` recovery-only
tag it previously lacked. `compiler/omni-parse/src/grammar_contract.rs` now
pins the invariant across sixteen malformed inputs, and separately asserts that
round-tripping is not achieved by accepting everything.

## Productions the current parser does not yet implement

The Edition 1 baseline is in force, but the parser is not yet a complete
Edition 1 parser. Recorded here so the 0.0.2.3 scope is explicit rather than
discovered later:

- `block_expr` is not implemented: a braced expression cannot appear in
  expression position, so `GRAM-0002`'s block-final-expression rule cannot be
  exercised yet. The semicolon rules that *are* implemented are tested.
- `binary_op` covers `|`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `+`, `-`, `*`, `/`,
  `=`, but omits `||`, `&&`, `>>`, `<<`, `^`, `&`, `%`, `..`, `..=`, and the
  shift-assignment and compound-assignment forms the lexer already emits.
- `struct_def` and `enum_def` are parsed as a bare stub that consumes the
  keyword and one identifier; their bodies are not built.
- `if`, `match`, `loop`, `while`, `for`, closures, arrays, tuples, and indexing
  have no production in the parser.

These are ordinary 0.0.2.3 work against the in-force baseline. None of them is
Candidate 2 surface syntax, and none of them changes the reconciliation.
## 0.0.2.3-A parser infrastructure boundary

The first 0.0.2.3 wave establishes reusable parser infrastructure only. It does not claim Edition 1 parser completion and does not enable Candidate 2 syntax.

The parser now has bounded lookahead, a state-safe missing-token consumption primitive, delimiter-pair helpers, and source-span access through the existing lexer spans. The generic-closer utility represents `>`, `>>`, and `>>=` as logical parts of the original token with contiguous byte ranges. It does not rewrite source text, duplicate trivia, or create generic parsing semantics.

### Lifetime lexer/parser contract blocker

The normative grammar contains `lifetime = "'" identifier` and also contains character literals introduced by the same apostrophe. The current lexer has no Lifetime token. Direct inspection of the scanner shows that `'a` is consumed by `scan_char_or_byte(false)` and produces one `TokenKind::Error` over bytes `0..2` with `ErrorReason::Lexical`; valid `'a'` remains `TokenKind::Char`, and `b'a'` remains `TokenKind::Byte`.

0.0.2.3-A deliberately does **not** reinterpret that lexical Error token inside the parser. Doing so without an established contextual-token contract would hide a lexer/parser boundary defect and could misclassify malformed character literals. The exact behavior is pinned by lexer regression tests. The lifetime production therefore remains a parser-blocking contract issue for the next specification/lexer decision; no grammar text is changed here.

### Trait/impl item-list separator ambiguity

The in-force EBNF defines both `trait_def` and `impl_def` item lists using comma separators, while `function_def` has no trailing semicolon. Consequently an ordinary example such as `impl Foo { fn a() {} fn b() {} }` does not have an authoritative separator interpretation from those productions alone. There is a second defect in the same area: `trait_item = function_signature | type_alias | const_def`, but the EBNF contains no `function_signature` production.

No implementation rule resolves these defects in 0.0.2.3-A. They remain explicit blockers for the affected trait/impl item-list productions. The normative EBNF is unchanged and Candidate 2 is not used to resolve them.

### 0.0.2.3 parser scope

The Edition 1 parser now produces structured CST nodes across the declaration, type, statement, expression, control-flow, pattern, attribute, generic, path, and macro-token-tree families present in the normative grammar. The semantic frontend additionally carries structured `let` patterns, generic calls, ranges with endpoint inclusivity, scalar casts, field/index reads, struct literals, enum constructors, and labelled loop forms.

The current MIR boundary includes scalar expressions, control-flow CFGs, tuple/array aggregate values, and typed field/index projection rvalues. Aggregate storage layout, enum representation, projected lvalue assignment, and guarded/aggregate-pattern match dispatch remain explicit downstream work where the repository does not yet define the required target representation.

The following normative specification holes remain unchanged and are not resolved by implementation fiat: `trait_item` references an undefined `function_signature` production; `let_expr` and `deref_expr` are referenced without standalone productions in the current EBNF; and `effect_bound`/`capability_bound` are referenced without definitions. The trait/impl comma-versus-function-terminator ambiguity is also preserved as a specification issue.

### Lifetime lexer/parser boundary correction

The normative grammar contains `lifetime = "'" identifier` and character literals use the same apostrophe introducer. The repository previously had no dedicated apostrophe punctuation token, so an input such as `'a` entered the character-literal error path.

0.0.2.3-A now makes the smallest lexer-side correction: a quote followed immediately by an identifier-start is emitted as `Punct::Apostrophe` plus the ordinary identifier token, while a valid single-character literal remains `TokenKind::Char`. A malformed character literal such as `'ab'` remains one lexical `ErrorToken`, and `b'a'` remains `TokenKind::Byte`. A quote followed by trivia rather than an identifier remains on the existing character-literal path, so the lexer does not invent a lifetime across whitespace or comments.

The parser does not claim the lifetime production itself is implemented in 0.0.2.3-A. It now receives the correct lexical boundary and preserves it losslessly; actual lifetime parsing remains later grammar work. No grammar text is changed.

### 0.0.2.3 parser implementation status

The parser now implements substantial Edition 1 coverage: module items (functions, structs, enums, type aliases, const/static declarations, use/module/extern-crate declarations, attributes), generic parameters and arguments, paths and core type forms, blocks and statement/final-expression handling, control-flow expressions, pattern families, postfix calls/fields/indexing/method calls/await, casts, ranges, all Edition 1 binary and assignment operator tokens already emitted by the lexer, closures, async/unsafe/try blocks, and nested macro token trees.

The implementation does not silently resolve specification holes. Trait function signatures remain blocked because `trait_item` references `function_signature` without defining that production; `let_expr` and `deref_expr` are referenced without productions; and `effect_bound`/`capability_bound` are referenced without definitions. The trait/impl comma-vs-function terminator ambiguity also remains recorded. These are specification blockers, not implementation-defined semantics.

Candidate 2 syntax remains disabled and the normative Edition 1 EBNF remains unchanged.
