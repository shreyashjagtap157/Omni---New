# Omni Complete Specification — Edition 1
## Consolidated Candidate 1.0.0-candidate.1

**Status:** Complete normative candidate; not ratified or implementation-certified.
**Language edition:** 1
**Consolidation:** Integrates the latest Edition 1 normative candidate with the archived v3.4 converged rules and the earlier v2.0 complete design/rationale specification.
**Normative authority:** The Edition 1 candidate text below, together with the exact release artifacts named by its normative manifest, is the current semantic baseline.

> **Integration rule:** The latest Edition 1 normative candidate supersedes earlier wording wherever the two differ. The v3.4 and v2.0 sections are retained as integrated historical design rationale and reconciliation context; they do not override the Edition 1 normative suite.

---

## Consolidation Map

1. **Current normative specification:** the complete Edition 1 candidate suite is reproduced verbatim below.
2. **Converged v3.4 commitments:** retained after the normative suite as historical/reconciliation context; Edition 1 governs any conflict.
3. **Original v2.0 complete specification:** retained as design rationale/history so architectural intent is not lost during the migration to the Edition 1 normative suite.
4. **Repository implementation status:** remains separate from language definition; implementation/qualification claims are not inferred from specification prose.

---

# Part I — Current Normative Specification

# Omni Programming Language Standard

## Edition 1 — Complete Candidate 1.0.0-candidate.1

> This consolidated view is generated from the modular normative suite. Module files and machine-readable artifacts remain authoritative as described by OMNI-STD-ROOT.


---

# OMNI-STD-ROOT: Standard Suite Root

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | None |
| Date | `2026-08-04` |

## 1. Scope

Architecture, authority, terminology ownership, compatibility, governance, conformance claims, release classes.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Product definition

Omni Edition 1 is a statically typed, memory-safe-by-default, capability-secure, effect-aware, multi-paradigm language compiled normally to native machine code. Its default memory discipline is affine ownership with borrowing and regions. Explicit standard types provide reference counting and managed domains. Raw memory, foreign interfaces, device access, and inline assembly exist only behind audited unsafe obligations.

The suite defines four independently claimable products:

1. **Core Language**: source through dynamic semantics, memory, FFI, security, core library, and conformance.
2. **Platform**: target identity, ABI, runtime, object/link/startup/debug supplements.
3. **Distribution**: package, build, update, reproducibility, diagnostics, formatter, and documentation.
4. **Optional Profiles**: managed, accelerated, persistent, distributed, verified, deterministic, hardened, realtime, and constant-time facilities.

A product claim SHALL name the exact release manifest and every enabled profile.

## 4. Normative authority

Within a published release, domain-specific artifacts are co-normative and SHALL agree:

- source bytes and tokenization: `OMNI-SOURCE` and `OMNI-LEX`;
- parse trees: `OMNI-GRAMMAR` and `grammar/omni-edition1.ebnf`;
- static judgments: prose rules plus `models/static-semantics.yaml`;
- dynamic behavior: prose rules plus the abstract-machine transition definitions;
- memory behavior: prose axioms plus `models/memory-model.yaml`;
- ABI and wire formats: prose plus exact schemas and golden vectors.

Tests witness rules but do not invent semantics. When two normative artifacts disagree, no conforming interpretation exists until an erratum resolves the defect.

## 5. Required invariants

| Rule | Requirement |
|---|---|
| `ROOT-0001` | The normative Omni Edition 1 language definition consists of every module listed as `required` by the signed release manifest plus the exact data files and schemas named there. |
| `ROOT-0002` | No implementation, reference interpreter, compiler, test, example, or prior draft may override normative language text or formal rules; conflicts are specification defects that block publication. |
| `ROOT-0003` | Syntax is jointly governed by OMNI-SOURCE, OMNI-LEX, and OMNI-GRAMMAR. Static semantics are jointly governed by OMNI-NAMES, OMNI-TYPES, OMNI-NUM, OMNI-OWN, and OMNI-EFFECTS. Dynamic semantics are jointly governed by OMNI-MACHINE, OMNI-EVAL, OMNI-ERROR, OMNI-CONC, and OMNI-MEM. |
| `ROOT-0004` | A release SHALL contain no unresolved normative placeholder, no unclassified observable behavior, and no rule whose required conformance evidence is absent from the release manifest. |
| `ROOT-0005` | Safe well-typed Edition 1 programs SHALL NOT have undefined behavior. Every permitted implementation choice is classified as implementation-defined, unspecified, conditionally supported, or resource-dependent. |
| `ROOT-0006` | Native ahead-of-time compilation is the default execution route. A bytecode VM, interpreter, JIT, or REPL is an optional implementation technique and SHALL preserve Edition 1 observations. |
| `ROOT-0007` | Core language semantics are independent of optimization level, debug instrumentation, or build mode. |
| `ROOT-0008` | Extensions SHALL be namespaced, disabled by strict-conformance mode, fingerprinted into artifacts, and prohibited from changing the meaning of accepted Edition 1 programs. |
| `ROOT-0009` | A conforming implementation SHALL publish its supported targets, profiles, limits, external ABI references, implementation-defined choices, and known deviations in machine-readable form. |
| `ROOT-0010` | Edition, ABI, wire-schema, profile, Unicode-data, toolchain, and package versions are independent axes and SHALL NOT be conflated. |
| `ROOT-0011` | The specification text and machine-readable registries SHALL be published as immutable content-addressed artifacts with a signed manifest. |
| `ROOT-0012` | Security or soundness errata may narrow accepted programs but SHALL preserve already-specified safe observations whenever technically possible; compatibility impact SHALL be published. |

## 6. Edition compatibility

An Edition 1 source file is parsed and checked only under Edition 1 rules unless its package manifest selects a later edition. Later editions MAY accept an Edition 1 program with the same observations, or reject it only through an explicit migration boundary. They MUST NOT silently reinterpret the same token sequence under an Edition 1 manifest.

## 7. Completeness criterion

This candidate is definition-complete because every construct admitted by the grammar has a name-resolution rule, typing rule, ownership/effect rule, dynamic rule, fault behavior, and conformance classification. Optional facilities are complete within their named profiles. Ratification and implementation certification are separate processes and are not implied by definition completeness.

## 8. Intellectual property and contribution baseline

The specification text is intended for publication under a permissive specification license permitting implementation, quotation, translation, and derivative conformance material. Source examples and schemas are intended for a permissive software/documentation license. A production standards organization SHALL adopt an explicit royalty-free patent policy, contributor agreement, disclosure duty, appeal procedure, and security-embargo process before accepting external normative contributions.


---

# OMNI-TERMS: Vocabulary and Behavioral Taxonomy

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-STD-ROOT` |
| Date | `2026-08-04` |

## 1. Scope

Normative terms; required, implementation-defined, unspecified, invalid execution, erroneous, conditionally supported, observable behavior.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Normative vocabulary

| Term | Definition |
|---|---|
| **required behavior** | The single behavior or set of behaviors mandated by a normative rule. |
| **implementation-defined behavior** | A permitted choice selected by an implementation, documented before translation, queryable by tools, and stable for an artifact fingerprint. |
| **unspecified behavior** | One behavior from a finite set explicitly listed by the standard; the implementation need not document which member occurs on a particular execution. |
| **conditionally supported** | A feature or limit that an implementation may omit, but whose presence, absence, limits, and semantics must be declared. |
| **erroneous program** | A source program that violates a diagnosable rule. Translation SHALL fail before a release artifact is emitted. |
| **invalid unsafe execution** | A dynamic execution that reaches an unsafe operation while its stated precondition is false. Requirements after that event are withdrawn only for that execution as defined by OMNI-UNSAFE. |
| **resource failure** | Failure to obtain finite resources such as memory, stack, handles, storage, quota, or time. It produces the defined failure policy and never grants permission for memory unsafety. |
| **environmental failure** | A failure reported by the host, device, network, filesystem, process, or foreign service through a declared result/fault channel. |
| **unsupported program** | A well-formed program requiring a target, profile, limit, or feature not claimed by the implementation; rejection SHALL identify the unmet requirement. |
| **observable behavior** | An event included in the active observation set: returned values, emitted I/O, capability interactions, volatile/device operations, atomic synchronization, persistent commits, faults, termination, and explicitly exposed traces. |
| **undefined behavior** | A phrase prohibited for safe Edition 1 behavior. It may appear only when describing a foreign standard or as an informal synonym immediately corrected to `invalid unsafe execution`. |
| **artifact** | A content-addressed executable, library, object, package, schema, or image produced by a declared build action. |
| **execution** | One instantiation of an artifact with a target environment, capabilities, inputs, scheduler, and resource policy. |
| **place** | A typed storage location or projection that may be read, written, borrowed, moved, or addressed. |
| **value** | A valid inhabitant of a type, independent of any particular storage location. |
| **capability** | An unforgeable typed authority value issued by a trusted provider and attenuable but not widenable by ordinary code. |
| **effect** | A statically tracked class of observable action that a computation may perform. |
| **profile** | A named, versioned set of additional facilities and restrictions that does not redefine core syntax or core expression meaning. |

## 4. Behavioral classification rules

| Rule | Requirement |
|---|---|
| `TERM-0001` | Every normative clause that admits more than one observable result SHALL classify the choice using exactly one term from this document. |
| `TERM-0002` | Implementation-defined choices SHALL appear in the implementation manifest and artifact fingerprint when they can affect linking, layout, execution, persistence, or diagnostics. |
| `TERM-0003` | Unspecified choices SHALL be finite and explicitly enumerated; unconstrained behavior is prohibited. |
| `TERM-0004` | Quality-of-implementation latitude MAY affect performance, diagnostic wording beyond required fields, code layout, and other non-observable details. |
| `TERM-0005` | An erroneous program SHALL not produce a conforming release artifact, although tools MAY construct recovery trees for editing. |
| `TERM-0006` | A resource failure SHALL be reported through the allocator/API result, artifact fault policy, or target-mandated abort path named by the applicable rule. |
| `TERM-0007` | An external hardware failure does not make prior safe behavior invalid; target supplements SHALL classify its delivered signal, trap, abort, or environmental result. |
| `TERM-0008` | Timing, power, electromagnetic leakage, cache state, and speculative microarchitectural state are not ordinary core observations; constant-time and hardened profiles add explicit security observations. |

## 5. Prohibited ambiguous terms

Normative documents SHALL NOT use “undefined”, “platform dependent”, “compiler dependent”, “normally”, “usually”, “reasonable”, “as needed”, or “best effort” to describe semantics unless the clause immediately maps the phrase to a defined category and exact obligation.


---

# OMNI-RULES: Rule and Registry Infrastructure

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-STD-ROOT`, `OMNI-TERMS` |
| Date | `2026-08-04` |

## 1. Scope

Stable rule IDs, status lifecycle, hashes, dependency edges, test/proof links, reserved extension ranges.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Rule identifiers

A rule identifier has the form `DOMAIN-NNNN`, where `DOMAIN` is registered and `NNNN` is a four-digit monotonically allocated number. Published identifiers are never reused. Removed rules remain tombstones with their replacement or reason.

| Rule | Requirement |
|---|---|
| `RULE-0001` | Core domains reserve identifiers 0001 through 7999; profile standards use 8000 through 8999; vendor extensions use a reverse-domain namespace and cannot use unqualified Omni identifiers. |
| `RULE-0002` | Each rule registry record SHALL contain ID, title, normative text hash, module version, applicability, dependencies, diagnostic codes, tests, proofs/models, compatibility class, and lifecycle state. |
| `RULE-0003` | Lifecycle states are proposed, candidate, ratified, deprecated, superseded, withdrawn, and erratum-corrected. |
| `RULE-0004` | A semantic change to a ratified rule SHALL allocate a new revision record and preserve the prior text hash. |
| `RULE-0005` | Rule dependencies SHALL be acyclic after collapsing explicitly declared mutually recursive semantic clusters. |
| `RULE-0006` | Every required diagnostic and conformance test SHALL reference at least one rule ID; every ratified P0 rule SHALL reference at least one test or mechanically checked model obligation. |
| `RULE-0007` | Extension names use `x.<reverse-domain>.<name>` for attributes/effects/profiles and `X-<reverse-domain>-NNNN` for rule IDs. |
| `RULE-0008` | Collision resolution is by registered namespace ownership; display-name similarity never grants identity. |

## 4. Registry schema

The normative schema is `schemas/rule-registry.schema.json`. JSON objects are canonicalized using lexicographically sorted UTF-8 property names, no insignificant whitespace, and normalized LF before hashing.


---

# OMNI-SOURCE: Source Representation

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-TERMS` |
| Date | `2026-08-04` |

## 1. Scope

UTF-8, Unicode data versions, normalization, line endings, spans, bidi/invisible controls, source hashing.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Source unit

A source unit is a tuple `(edition, package-id, module-path, normalized-bytes, provenance)`. Source decoding occurs before lexical analysis. Ill-formed source never enters macro expansion or parsing.

| Rule | Requirement |
|---|---|
| `SRC-0001` | Source files SHALL be well-formed UTF-8. An optional UTF-8 BOM is accepted only at byte offset zero and is excluded from semantic hashing. |
| `SRC-0002` | Line endings CRLF and CR are semantically normalized to LF. Tools SHALL preserve an original-byte mapping for diagnostics and edits. |
| `SRC-0003` | Identifier equality uses Unicode NFC after tokenization. String and character data are never silently normalized. |
| `SRC-0004` | Edition 1 pins Unicode 17.0.0, UAX #15, UAX #31, UAX #9, UAX #24, UAX #44, and UTS #39 Revision 32 data through the release reference manifest and SHA-256 digests. |
| `SRC-0005` | Outside comments and literals, bidi control characters, noncharacters, unassigned code points, variation selectors, and default-ignorable format characters are source errors. |
| `SRC-0006` | Inside comments, bidi controls and invisible format characters require an escaped visible annotation generated by the formatter; strict mode rejects unannotated occurrences. |
| `SRC-0007` | Inside string and character literals, all Unicode scalar values except prohibited unescaped controls are data. Escape processing is defined by OMNI-LEX. |
| `SRC-0008` | Source spans are half-open byte ranges in normalized UTF-8 plus a source-file identity. Display columns are informative views. |
| `SRC-0009` | Semantic source identity is SHA-256 over edition tag, normalized UTF-8 bytes, package/module identity, generated-source recipe identity, and declared source attributes. |
| `SRC-0010` | Filesystem case, symlink spelling, timestamps, inode numbers, and enumeration order do not participate in module identity. |
| `SRC-0011` | Generated source SHALL record generator artifact digest, declared inputs, output path identity, and source map. |

## 4. Identifier security profile

Identifiers use `XID_Start` and `XID_Continue` from the pinned UCD, with `_` additionally permitted at the start and continuation. Join controls are excluded in Edition 1. Public identifiers SHALL satisfy the UTS #39 Highly Restrictive profile. Private identifiers that fail it are accepted only with an explicit `#[allow(mixed_script_identifier)]` attribute and remain subject to confusable collision checks.

Two identifiers in the same namespace whose NFKC casefolded skeletons are equal are a compile error unless both are ASCII and byte-distinct solely by case in a case-sensitive namespace, in which case strict mode still diagnoses them. Keywords are ASCII and compared byte-for-byte.

## 5. Hashing and archives

The candidate pins exact Unicode versions and revisions. A ratified mirrored publication SHALL additionally record SHA-256 digests for every normative Unicode data file; a compiler using different data SHALL reject an Edition 1 strict-conformance claim rather than silently retokenize source.


---

# OMNI-LEX: Lexical Grammar

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-SOURCE` |
| Date | `2026-08-04` |

## 1. Scope

Tokens, keywords, identifiers, literals, comments, punctuation, lexical errors, maximal munch.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Token classes

Edition 1 tokens are identifiers, keywords, literals, punctuation, delimiters, comments, and end-of-file. Comments and whitespace are retained in a lossless concrete syntax tree but are absent from the semantic token stream.

### 3.1 Keywords

`Never Self Sized addrspace as async await bare_metal bf16 bool break byte cap catch char const continue dec128 dec32 dec64 defer distributed dyn effect else enum ensure extern f128 f16 f32 f64 false fn for hosted i128 i16 i32 i64 i8 if impl in is isolate isize let loop macro managed match module move mut not opaque override package panic persistent pub pure ref relation require return script self static str struct super thread_local trait true try type typeof u128 u16 u32 u64 u8 unsafe use usize verified where while yield`

### 3.2 Punctuation

Single tokens: `(` `)` `[` `]` `{` `}` `,` `;` `:` `.` `@` `#` `?` `$` `_`.

Operators and compound tokens: `::` `->` `=>` `:-` `..` `..=` `...` `!` `!=` `=` `==` `<` `<=` `>` `>=` `+` `-` `*` `/` `%` `&` `|` `^` `~` `&&` `||` `<<` `>>` `+=` `-=` `*=` `/=` `%=` `&=` `|=` `^=` `<<=` `>>=` `?.` `??`.

Reserved for previous editions: `<-` `<=>` `**` and any maximal punctuation sequence not listed above.
`|>` is admitted by Candidate 2 Vibe Surface Syntax as the pipeline operator (`a |> b` desugars to `b(a)`). Its presence does not override Edition 1 strict-conformance rules for unaffiliated editions.

| Rule | Requirement |
|---|---|
| `LEX-0001` | Tokenization uses maximal munch except that `>>` in a type generic-argument context is two `>` tokens; the parser supplies this single context-sensitive split without changing source offsets. |
| `LEX-0002` | Whitespace separates tokens but is otherwise insignificant. Newlines never terminate statements. |
| `LEX-0003` | Line comments begin with `//` and end before LF. Block comments `/* ... */` nest. Unterminated comments are lexical errors. |
| `LEX-0004` | Documentation comments are `///`, `//!`, `/** ... */`, and `/*! ... */` and are converted to `doc` attributes before item parsing. |
| `LEX-0005` | Raw identifiers use `r#identifier`; the underlying identifier may equal a keyword but must otherwise satisfy identifier rules. |
| `LEX-0006` | Integer separators `_` are allowed only between digits of the same radix. Leading, trailing, adjacent-to-prefix, adjacent-to-suffix, or doubled separators are errors. |
| `LEX-0007` | Escape sequences are `\0`, `\t`, `\n`, `\r`, `\"`, `\'`, `\\`, `\xNN`, and `\u{H...}` with one to six hexadecimal digits naming a Unicode scalar. |
| `LEX-0008` | Character literals contain exactly one Unicode scalar after escape processing. Byte literals contain exactly one value 0 through 255. |
| `LEX-0009` | Raw strings use `r"..."` or `r#*"..."#*` with matching hash count from zero through 255. No escapes are processed. |
| `LEX-0010` | Interpolated strings use `f"text ${ expression } text"`; braces in text are escaped as `{{` and `}}`. Interpolation is tokenized recursively with balanced delimiters. |
| `LEX-0011` | Unknown punctuation is a lexical error unless introduced by an enabled namespaced extension. |

## 4. Literal grammar

Integer literals are binary (`0b`), octal (`0o`), decimal, or hexadecimal (`0x`) digit sequences with an optional fixed primitive suffix. Decimal floating literals require a decimal point or exponent; hexadecimal floating literals require a `p` exponent. Supported suffixes are `i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f16 bf16 f32 f64 f128 dec32 dec64 dec128` subject to profile support.

String prefixes are `b` for bytes, `r` for raw, `f` for interpolation, and `br`/`rb` for raw bytes. Interpolation and raw mode cannot be combined in Edition 1.

## 5. Lexical errors

A malformed token consumes the shortest prefix that proves malformed while permitting deterministic recovery. Recovery tokens are tooling-only and SHALL NOT appear in a release AST.


---

# OMNI-GRAMMAR: Syntactic Grammar

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-LEX` |
| Date | `2026-08-04` |

## 1. Scope

Complete grammar, precedence, associativity, disambiguation, parse errors, grammar versioning.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Grammar notation

`{ X }` means zero or more repetitions, `[ X ]` means optional, `( A | B )` means choice, and quoted strings are lexical terminals. `A - B` means the subset of `A` excluding forms classified as `B`; the distributed grammar replaces this convenience with generated productions.

| Rule | Requirement |
|---|---|
| `GRAM-0001` | The normative grammar is `grammar/omni-edition1.ebnf`; prose in this document resolves only notation and explicitly stated context restrictions. |
| `GRAM-0002` | A block final expression lacks a semicolon and yields the block value. A non-block expression used as a statement requires `;`. A block-form expression statement may omit `;`. |
| `GRAM-0003` | Newlines are not grammar terminals and cannot change a parse [in Edition 1 strict conformance]. Newlines MAY serve as statement boundaries at unambiguous statement boundaries per Candidate 2 Vibe Surface Syntax amendment; this exception does not apply to Edition 1 conformance claims without the Candidate 2 feature gate. |
| `GRAM-0004` | Assignment operators are right-associative. All other binary operators are left-associative except comparisons, which do not chain. |
| `GRAM-0005` | Postfix operators bind tighter than unary operators; unary operators bind tighter than multiplicative operators; the complete precedence is encoded by grammar nonterminals. |
| `GRAM-0006` | Expression generic arguments after a path or method use the explicit `::` introducer where needed to avoid comparison ambiguity. |
| `GRAM-0007` | Error productions MAY exist in an IDE parser but SHALL be tagged recovery-only and SHALL NOT be accepted in release translation. |
| `GRAM-0008` | Feature-gated grammar exists only in a named edition or profile namespace and is included in the source and artifact fingerprint. |

## 4. Semicolon and block rules

The grammar admits no automatic semicolon insertion. The parser never examines line breaks to decide whether a statement ended. `return`, `break`, and `continue` are expressions; when used as statements they follow the same semicolon rule.

## 5. Precedence, from lowest to highest

1. assignment;
2. ranges;
3. logical OR;
4. logical AND;
5. null coalescing;
6. one comparison;
7. bitwise OR, XOR, AND;
8. shifts;
9. additive;
10. multiplicative;
11. casts;
12. unary;
13. postfix;
14. primary.

## 6. Parse determinism

A conforming parser SHALL produce the canonical concrete tree defined by the grammar. Ambiguity is a specification defect. The release corpus includes every pair of adjacent token classes, nested generic/shift cases, macro token trees, and semicolon-sensitive token sequence.


---

# OMNI-ATTR: Attributes and Annotations

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-GRAMMAR`, `OMNI-NAMES` |
| Date | `2026-08-04` |

## 1. Scope

Attribute syntax, namespaces, retention, duplication, target gating, unknown attributes, semantic phases.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Built-in attributes

Edition 1 defines `#[repr(...)]`, `#[inline(...)]`, `#[cold]`, `#[must_use]`, `#[deprecated(...)]`, `#[target_feature(...)]`, `#[cfg(...)]`, `#[derive(...)]`, `#[no_mangle]`, `#[export_name(...)]`, `#[link_name(...)]`, `#[panic_policy(...)]`, `#[numeric(...)]`, `#[allow(...)]`, `#[warn(...)]`, `#[deny(...)]`, `#[forbid(...)]`, `#[test]`, `#[bench]`, and `#[doc(...)]`.

| Rule | Requirement |
|---|---|
| `ATTR-0001` | Attributes are resolved in the attribute namespace after tokenization and outer macro expansion but before the semantic phase named by the attribute definition. |
| `ATTR-0002` | Unknown unqualified attributes are errors. Unknown namespaced attributes are errors in strict mode and may be retained as inert metadata only when their namespace policy allows it. |
| `ATTR-0003` | An attribute definition declares targets, multiplicity, argument grammar, retention (`source`, `HIR`, `MIR`, `object`, or `runtime`), and whether it affects semantics or only diagnostics/tooling. |
| `ATTR-0004` | Duplicate non-repeatable attributes are errors. Repeatable attributes preserve source order unless their definition declares set semantics. |
| `ATTR-0005` | Attributes cannot widen capabilities, suppress type/ownership safety, or create unchecked behavior except through the standard `unsafe` mechanisms. |
| `ATTR-0006` | Target/profile-gating attributes remove an item before name resolution only when the condition depends solely on manifest-declared target/profile facts. |
| `ATTR-0007` | Built-in semantic attributes are reserved under `omni.*`; user and vendor attributes require a registered namespace. |

## 4. Attribute phase order

1. lexical conversion of documentation comments;
2. inner source/module attributes;
3. declarative token macros;
4. target/profile item filtering;
5. import and name resolution;
6. derive and typed macros;
7. static semantics;
8. lowering/optimization hints;
9. object/runtime metadata retention.


---

# OMNI-MACHINE: Abstract Machine and Translation Model

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-SOURCE`, `OMNI-GRAMMAR`, `OMNI-TERMS` |
| Date | `2026-08-04` |

## 1. Scope

Translation phases, program/environment model, observations, termination, resource failure, host/target distinction.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Abstract machine state

The machine state is `A = (P, Σ, T, C, X, R)`:

- `P`: immutable program and type metadata;
- `Σ`: allocations, object lifetimes, values, and persistent buffers;
- `T`: tasks/threads with control stacks and cleanup stacks;
- `C`: capabilities and provider state;
- `X`: ordered external input/event stream;
- `R`: finite resource budgets and artifact policies.

A transition `A --event--> A′` either emits no observation (`τ`) or one typed observation. A terminating machine produces `exit(status)`, `panic(payload)`, `isolate(task,payload)`, `abort(reason)`, or `target_trap(code)` as defined by the active artifact and target policy.

| Rule | Requirement |
|---|---|
| `MACH-0001` | A program is the closed package graph, selected target and profiles, linked artifacts, manifest capabilities, and entry point. |
| `MACH-0002` | Translation proceeds through source decoding, lexing, parsing, macro expansion, name resolution, static checking, semantic lowering, target lowering, object emission, linking, loading, runtime-component initialization, and entry invocation. |
| `MACH-0003` | Each translation phase consumes only declared inputs and emits content-addressed outputs plus diagnostics; no phase may observe undeclared time, randomness, environment, filesystem state, or network. |
| `MACH-0004` | An execution consists of an initial abstract store, task set, capability set, external-event stream, scheduler choices, and resource policy. |
| `MACH-0005` | Core observations are ordered I/O/capability events, volatile/device events, synchronization events explicitly exposed by APIs, persistent commit outcomes, returned exit status, faults, panic/isolation, and termination. |
| `MACH-0006` | Internal allocation addresses, stack layout, register contents, padding bytes, optimizer choices, and elapsed time are not core observations. |
| `MACH-0007` | Nontermination is a permitted outcome when the program has an infinite transition sequence and no stronger progress/realtime contract applies. |
| `MACH-0008` | Resource exhaustion follows OMNI-ERROR and cannot invalidate prior safe observations or permit memory/type violations. |
| `MACH-0009` | External asynchronous events enter only at target/profile-defined delivery points and are represented as typed events, cancellation, signals, interrupts, or device completions. |
| `MACH-0010` | A hosted entry point is `fn main(args: Args, caps: AppCaps) -> Exit ! ε`; a freestanding target supplement defines reset/entry signatures; script mode synthesizes a package and uses hosted semantics. |
| `MACH-0011` | Normal process termination flushes only resources whose APIs promise flush-on-close; power loss and forced termination do not run ordinary destructors. |

## 4. Host and target separation

Compile-time execution runs in a deterministic host-independent semantic machine. Target constants such as pointer width and endianness are explicit inputs. Host pointer size, locale, floating environment, path separator, and process environment cannot leak into target semantics.

## 5. Full expressions and sequence points

A full expression is an initializer, condition, return/break value, expression statement, match guard, call argument, aggregate element/field initializer, or interpolation expression at the grammar boundary named by OMNI-EVAL. Full-expression completion is a cleanup boundary for non-extended temporaries.


---

# OMNI-NAMES: Names, Modules, Linkage, and Initialization

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-GRAMMAR`, `OMNI-MACHINE` |
| Date | `2026-08-04` |

## 1. Scope

Scopes, imports, visibility, package/module identity, linkage, initialization, symbol identity, coherence anchors.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Scope model

Scopes are lexical and nested. Item declarations are visible throughout their module after target/profile filtering, except macro declarations whose visibility follows their declared expansion phase. Local bindings become visible after their initializer completes, preventing self-reference by accident.

| Rule | Requirement |
|---|---|
| `NAME-0001` | Module identity is `(package-source-id, package-name, package-version, feature-instance, module-path)` and never a raw filesystem path. |
| `NAME-0002` | A source file declares one module or one explicitly permitted fragment. Fragment merge order is the lexicographic order of normalized source identities and SHALL NOT affect semantic resolution. |
| `NAME-0003` | Separate namespaces exist for modules/types, values, traits, macros, lifetimes, labels, effects, capabilities, and attributes. |
| `NAME-0004` | Within a namespace, a declaration may not duplicate another declaration with the same normalized identifier unless it is an explicitly mergeable module fragment or trait implementation. |
| `NAME-0005` | Imports are explicit, non-transitive, and resolved independent of source/file enumeration order. Wildcard imports that create ambiguity are errors. |
| `NAME-0006` | Lexical shadowing is permitted for local values and lifetimes; items, generic parameters, labels, effects, and capabilities cannot be silently shadowed in the same declaration header. |
| `NAME-0007` | Name lookup order is local binding, generic parameter, current item members, explicit imports, module items, then prelude. Ambiguity at one level is an error and lower levels are not searched. |
| `NAME-0008` | Free functions do not overload by parameter type. Method and operator polymorphism are trait-based and coherence-checked. |
| `NAME-0009` | Import cycles are allowed only when all cycle edges are type/macro-signature-only and no value initialization depends cyclically. The initialization graph must be acyclic. |
| `NAME-0010` | `const` values are evaluated at translation. Immutable `static` values use constant initialization or an explicit lazy cell. Mutable static access is unsafe unless mediated by a safe synchronization type. |
| `NAME-0011` | Thread-local initialization occurs on first odr-use per thread, and failure is cached as the declared panic/error policy. |
| `NAME-0012` | Public symbol identity includes package identity, module path, item name, generic signature, ABI version, and representation/effect fingerprint. |
| `NAME-0013` | Trait coherence is global across the resolved package graph: an implementation is legal only if the trait or nominal self type is local to the defining package, except sealed delegated extension points. |
| `NAME-0014` | Dynamic loading cannot introduce a trait implementation that would overlap an implementation visible when the consuming artifact was linked. |

## 4. Initialization algorithm

1. evaluate all compile-time constants;
2. construct immutable constant-initialized statics;
3. register lazy and thread-local cells without executing user bodies;
4. initialize selected runtime components in dependency order;
5. invoke the entry point.

Top-level arbitrary executable initialization is prohibited. Cross-module initialization cycles therefore cannot hide in linker order.

## 5. Method resolution

For receiver type `R`, method resolution constructs a finite autoderef chain using built-in references and in-scope `Deref` implementations. At each level it considers inherent methods, then explicitly imported trait methods, applying at most one autoref. Exactly one candidate must remain after generic constraints. Return type alone cannot select a method.


---

# OMNI-TYPES: Static Type System

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-NAMES`, `OMNI-MACHINE` |
| Date | `2026-08-04` |

## 1. Scope

Types, inference, generics, traits, subtyping/coercions, variance, object validity, unsized and zero-sized types.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Static judgment


- `Γ` is the lexical typing and name environment.
- `Ω` is the ownership and initialization state.
- `Ε` is the available effect/capability environment.
- `Σ` is the abstract store, including allocation identities and object lifetimes.
- `Μ` is the concurrent memory event graph.
- `Γ; Ω; Ε ⊢ e : T ! ε ⇒ Ω′` means expression `e` has type `T`, may perform effect row `ε`, and transforms ownership state `Ω` to `Ω′`.
- `⟨e, Σ, κ⟩ → ⟨e′, Σ′, κ′⟩` is one dynamic evaluation step.
- `hb` denotes happens-before; `sw` denotes synchronizes-with; `mo` denotes per-atomic modification order.


A declaration is accepted only when name resolution is unique, type/effect constraints terminate with one solution, all ownership paths are valid, every potentially executed operation is authorized by its effect/capability context, and all required refinements are proved or checked.

| Rule | Requirement |
|---|---|
| `TYPE-0001` | Type identity is nominal for structs, enums, traits, opaque types, capabilities, effects, and aliases declared `distinct`; ordinary `type` aliases are transparent. |
| `TYPE-0002` | Tuples, function types, references, raw pointers, arrays, slices, and generic instantiations are structurally equal when all constituents and qualifiers are equal. |
| `TYPE-0003` | Recursive nominal types require an indirection, unsized tail, managed reference, or opaque boundary; infinitely sized value types are erroneous. |
| `TYPE-0004` | Every safe value satisfies its type validity invariant. Invalid scalar bit patterns cannot be constructed or observed in safe code. |
| `TYPE-0005` | `bool` has values `false` and `true`; `char` is any Unicode scalar; references are non-null, aligned, live, and provenance-valid; enums have a declared live variant. |
| `TYPE-0006` | Zero-sized types occupy no logical bytes but have distinct ownership/drop events. Arrays of zero-sized types retain length and iteration count. |
| `TYPE-0007` | `Never` is uninhabited and coerces to any type. Diverging expressions have type `Never`. |
| `TYPE-0008` | Unsized types are `[T]`, `str`, `dyn Trait`, and declared extern types. They may occur only behind a pointer/reference/owner or as the final field of a declared dynamically sized aggregate. |
| `TYPE-0009` | Public functions, public fields, FFI declarations, persisted schemas, capabilities, and exported constants require explicit types and effects. |
| `TYPE-0010` | Inference is local to a function/item body and must have a unique principal solution after defaults. Ambiguity is a diagnostic, not arbitrary selection. |
| `TYPE-0011` | Generic parameter kinds are type, lifetime, const value, effect row, and capability type. Defaults may reference earlier parameters only. |
| `TYPE-0012` | Constraint solving normalizes aliases and associated types, applies coherence-selected implementations, and terminates under the edition solver restrictions. |
| `TYPE-0013` | Trait objects contain only object-safe traits: no methods requiring `Self: Sized`, no generic methods, and associated types/consts required by calls must be fixed. |
| `TYPE-0014` | Downcast identity uses a 128-bit stable type fingerprint only for types opting into runtime identity; ordinary private type identity is not serialized. |
| `TYPE-0015` | Specialization exists only inside a sealed specialization family with a strict partial order where each pair of applicable implementations has one unique greatest element. |
| `TYPE-0016` | Implicit coercions are limited to lifetime shortening, `&mut T` to `&T`, reborrow, array reference to slice reference, concrete reference/owner to object-safe trait object, function item to function pointer, noncapturing closure to compatible function pointer, unsizing, and `Never` to any type. |
| `TYPE-0017` | No implicit numeric conversion, allocation, cloning, reference-count change, dynamic boxing, blocking, or capability acquisition occurs as a coercion. |
| `TYPE-0018` | Refinements are conjunctions of linear integer bounds, equalities, finite-set membership, length/alignment/unit predicates, and sealed protocol-state predicates. Undischarged obligations require explicit runtime checks or proof parameters. |
| `TYPE-0019` | Units and dimensions normalize to a rational scale and integer exponent vector. Unit metadata is erased from native ABI only after conversion is explicit and overflow-checked. |
| `TYPE-0020` | `Dynamic` is an explicit boxed tagged value with runtime type identity, checked extraction, `dynamic` effect for reflective operations, and no implicit conversion to static types. |

## 4. Variance

- `&'a T` is covariant in `'a` and in `T` when `T` contains no interior-mutability exposure through that reference.
- `&'a mut T`, `Own<T>`, `Pinned<T>`, and mutable capability types are invariant in `T`.
- raw pointers are invariant and have no safe subtyping.
- arrays and immutable containers inherit element covariance only for shared-borrow views, never for mutable/owning forms.
- function parameters are contravariant, returns covariant, and effect rows covariant by subset (a purer function substitutes for a more effectful allowance).

## 5. Traits and associated items

Trait laws are normative documentation/proof obligations but are not assumed by the optimizer unless attached as proved contracts. Marker traits affecting safety (`Copy`, `Send`, `Sync`, `Unpin`, `Capability`) are compiler-known, sealed, or unsafe to implement with explicit obligations.

## 6. Refinement fallback

The compiler uses the solver fragment version pinned in the release manifest. A failed proof is not proof of falsity. The programmer may add a checked contract, provide a proof term, or rewrite the program; the compiler may not guess.


---

# OMNI-NUM: Numeric Semantics

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-TYPES`, `OMNI-MACHINE` |
| Date | `2026-08-04` |

## 1. Scope

Integers, bit-precise types, floating modes, decimal, NaNs, rounding, exceptions, conversions, reproducibility.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Integer and bit-precise model

For `uint<N>`, values are integers in `[0, 2^N-1]`. For `int<N>`, values are integers in `[-2^(N-1), 2^(N-1)-1]`. The abstract value is mathematical; the native representation is fixed by this document and the target endian/layout supplement.

| Rule | Requirement |
|---|---|
| `NUM-0001` | Fixed-width signed integers use two’s-complement with no padding bits. Unsigned integers use pure binary representation. |
| `NUM-0002` | Edition 1 implementations SHALL support bit-precise `int<N>` and `uint<N>` for every `1 <= N <= 4096`; larger widths are conditionally supported and declared. |
| `NUM-0003` | `usize` and `isize` match the target default data-pointer address width but do not imply every address-space pointer has that representation. |
| `NUM-0004` | Integer literals are arbitrary-precision until constrained; unconstrained integer literals default to `i64`. Nonrepresentable literals are compile errors. |
| `NUM-0005` | Default integer arithmetic is checked in every build mode. Overflow, invalid shift counts, division by zero, and signed-minimum divided by minus one raise `ArithmeticFault`. |
| `NUM-0006` | Explicit arithmetic families are `checked`, `wrapping`, `saturating`, `widening`, `carrying/borrowing`, and proof-qualified `exact`. |
| `NUM-0007` | Shifts accept a nonnegative integer count strictly less than the left operand width. Wrapping shift APIs reduce the count modulo width explicitly. |
| `NUM-0008` | Integer division truncates toward zero; remainder has the dividend sign and satisfies `a = (a/b)*b + a%b` when defined. |
| `NUM-0009` | No implicit conversion occurs between distinct concrete numeric types. `as?` is checked and returns `Option`; `as!` traps on failure; named wrapping/truncating/bitcast APIs state alternate semantics. |
| `NUM-0010` | Bitcast requires equal bit width and a destination representation for which the produced bits are valid, or returns a raw byte/MaybeUninit form requiring validation. |
| `NUM-0011` | Floating functions inherit a lexical numeric policy: `strict` by default, `reproducible`, `contract`, or `fast`. |
| `NUM-0012` | `strict` uses IEEE roundTiesToEven for ordinary operations, preserves signed zero and infinities, forbids reassociation, and quiets signaling NaNs. NaN payload selection is implementation-defined and declared. |
| `NUM-0013` | `reproducible` canonicalizes NaNs, fixes subnormal handling to gradual underflow, fixes operation decomposition, and guarantees bit-identical results for the supported operation set across conforming targets. |
| `NUM-0014` | `contract` permits fused multiply-add only at source-marked contraction sites. `fast` permits only the exact relaxations listed in its annotation and never changes memory/type/authority safety. |
| `NUM-0015` | `f16`, `bf16`, `f32`, and `f64` have their named storage formats. Evaluation occurs in the declared type except explicit widening operations. `f128` is conditionally supported. |
| `NUM-0016` | Floating comparisons follow IEEE ordered/unordered semantics; total ordering is provided by a named `total_cmp` operation. |
| `NUM-0017` | Decimal types are available in the decimal profile with IEEE decimal32/64/128 interchange semantics, explicit decimal context, and no implicit binary-decimal conversion. |
| `NUM-0018` | Compile-time and runtime numeric behavior are identical under the same policy and target feature set. |

## 4. Floating environment

Edition 1 does not expose a mutable ambient processor rounding mode to ordinary arithmetic. Alternate rounding is an explicit operation or decimal/binary context capability. Implementations SHALL save/restore or avoid incompatible host floating state at foreign boundaries.

## 5. Constant-time interaction

Numeric operations whose target latency depends on secret operands are rejected in the constant-time profile unless the target supplement certifies the instruction sequence or a verified constant-time library implementation is selected.


---

# OMNI-OWN: Ownership, Lifetimes, Regions, and Destruction

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-TYPES`, `OMNI-MACHINE` |
| Date | `2026-08-04` |

## 1. Scope

Moves, borrows, reborrows, regions, pinning, partial initialization/moves, drops, interior mutability.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Ownership judgment


- `Γ` is the lexical typing and name environment.
- `Ω` is the ownership and initialization state.
- `Ε` is the available effect/capability environment.
- `Σ` is the abstract store, including allocation identities and object lifetimes.
- `Μ` is the concurrent memory event graph.
- `Γ; Ω; Ε ⊢ e : T ! ε ⇒ Ω′` means expression `e` has type `T`, may perform effect row `ε`, and transforms ownership state `Ω` to `Ω′`.
- `⟨e, Σ, κ⟩ → ⟨e′, Σ′, κ′⟩` is one dynamic evaluation step.
- `hb` denotes happens-before; `sw` denotes synchronizes-with; `mo` denotes per-atomic modification order.


The ownership state maps each place projection to initialization status and active loans. Control-flow joins are legal only when all incoming states admit a single conservative state in which no potentially moved value is treated as initialized and no expired loan is treated as live.

| Rule | Requirement |
|---|---|
| `OWN-0001` | Every non-`Copy` value is affine: it may be moved at most once and destroyed at most once. Unused owned values are destroyed at their cleanup boundary. |
| `OWN-0002` | A place is initialized, partially initialized, moved, or uninitialized. Reads and drops require the relevant portion to be initialized. |
| `OWN-0003` | A move transfers value ownership and marks the source projection uninitialized. Moving through a shared borrow is prohibited. |
| `OWN-0004` | `Copy` may be implemented only for types with no destructor and only `Copy` fields. Copying duplicates the value without invalidating the source. |
| `OWN-0005` | At any time a memory location has either any number of usable shared borrows or one usable mutable borrow, unless access is mediated by `UnsafeCell` and a safe synchronization contract. |
| `OWN-0006` | Reborrowing creates a child loan whose permissions and lifetime are no greater than the parent; conflicting use of the parent is suspended while the child is live. |
| `OWN-0007` | Borrow lifetimes are inferred from actual use and control flow. Lifetime elision applies only to the listed function-signature patterns and never guesses among multiple input lifetimes. |
| `OWN-0008` | Higher-ranked lifetime bounds quantify explicitly or through the `for<...>` form; escaping a locally bound lifetime is erroneous. |
| `OWN-0009` | Compiler-generated mutable receiver autoref uses a two-phase loan: reservation before later argument evaluation and activation immediately before call entry. No other borrow is two-phase unless explicitly specified. |
| `OWN-0010` | Partial moves are legal for aggregates without an unconditional whole-value destructor, or where compiler-generated drop flags identify each remaining field. User `Drop` types cannot be partially moved through safe syntax. |
| `OWN-0011` | Locals drop in reverse successful-initialization order. Fields and active variant payloads drop in reverse declaration order. Arrays drop from highest initialized index to lowest. |
| `OWN-0012` | Closure capture fields are ordered by first source occurrence of the captured root binding, then by projection order; they drop in reverse field order. |
| `OWN-0013` | Temporaries drop in reverse creation order at the end of the full expression unless a grammar-defined binding extends them. Borrow diagnostics and MIR expose any hidden binding. |
| `OWN-0014` | When returning, the return value is fully evaluated and moved into the return slot before local cleanup; cleanup then runs and the initialized return slot transfers to the caller. |
| `OWN-0015` | A failed constructor drops every successfully initialized field in reverse initialization order and never drops uninitialized fields. |
| `OWN-0016` | Pinning guarantees that the pinned value will not move or have its storage reused until its pinned destructor completes. Projection is safe only through a pin-projection contract. |
| `OWN-0017` | Self-referential initialization requires a pinned construction API that prevents observation before all internal references are established. |
| `OWN-0018` | `UnsafeCell<T>` is the sole primitive for legal interior mutation through shared references. Safe wrappers must establish synchronization or thread confinement. |
| `OWN-0019` | Regions own allocations collectively. Region references cannot outlive the region; unique regions may move between tasks; frozen regions may be shared immutably. |
| `OWN-0020` | Bulk region reclamation does not run element destructors unless the region was created as a finalizing region, which records and runs drops in reverse registration order. |
| `OWN-0021` | `Shared<T>` uses atomic reference counts with release on decrement and acquire fence on the transition to zero. `LocalShared<T>` is thread-confined and non-atomic. |
| `OWN-0022` | Weak upgrade races are resolved atomically: success obtains a strong count while the object is live; failure returns `None`. Cycles are not collected unless placed in a managed domain or explicit cycle collector. |

## 4. Lifetime elision

A single input reference lifetime is assigned to every elided output reference lifetime. For methods, an elided output lifetime is assigned the receiver lifetime. Any other elided output lifetime is an error. Body lifetime inference does not alter public signature identity.

## 5. Destructors

`Drop.drop(&mut self)` runs exactly once for a fully initialized owned value on ordinary scope exit and supported unwind. It cannot move fields through safe syntax. Destructor panic behavior is defined by OMNI-ERROR. Process abort, power loss, target reset, and invalid unsafe execution do not promise destructor execution.

## 6. Arena and managed interactions

A managed reference cannot point into a shorter native region unless the managed object holds an owning region handle. Native pointers into movable GC objects cannot cross a safepoint; pin, handle, or copy is required.


---

# OMNI-EFFECTS: Effects, Capabilities, and Authority

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-TYPES`, `OMNI-OWN` |
| Date | `2026-08-04` |

## 1. Scope

Effect rows, inference, subtyping, handlers, resumptions, capability issuance/delegation/revocation, determinism.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Effect typing


- `Γ` is the lexical typing and name environment.
- `Ω` is the ownership and initialization state.
- `Ε` is the available effect/capability environment.
- `Σ` is the abstract store, including allocation identities and object lifetimes.
- `Μ` is the concurrent memory event graph.
- `Γ; Ω; Ε ⊢ e : T ! ε ⇒ Ω′` means expression `e` has type `T`, may perform effect row `ε`, and transforms ownership state `Ω` to `Ω′`.
- `⟨e, Σ, κ⟩ → ⟨e′, Σ′, κ′⟩` is one dynamic evaluation step.
- `hb` denotes happens-before; `sw` denotes synchronizes-with; `mo` denotes per-atomic modification order.


Effect rows are written `!{fs.read, alloc | E}`. The empty row is pure. A caller must provide both static permission for the effect and any capability argument required by the callee.

| Rule | Requirement |
|---|---|
| `EFF-0001` | An effect row is a finite canonical set of effect terms plus at most one row variable. Duplicate terms normalize to one term. |
| `EFF-0002` | Effect row equality is equality after alias expansion, parameter normalization, and canonical sorting. |
| `EFF-0003` | A function with effect row `ε1` substitutes where `ε2` is allowed when `ε1` is a subset of `ε2` after constraint solving. |
| `EFF-0004` | Private functions may infer effects. Public functions and trait methods declare an explicit upper bound; adding an externally visible effect is a breaking API change unless already polymorphic. |
| `EFF-0005` | Effect masking is permitted only by a handler that discharges the effect and whose own residual effects are included in the result row. |
| `EFF-0006` | Effects are generalized only for syntactic value bindings whose captured values satisfy ownership constraints; effectful computations are not implicitly generalized. |
| `EFF-0007` | Built-in effects include allocation, panic, cancellation, synchronization, blocking, async suspension, I/O families, time, randomness, environment, dynamic reflection, foreign calls, persistence, devices, accelerators, nondeterminism, and unsafe families. |
| `EFF-0008` | An effect declaration states observations and handler protocol; it does not itself grant authority over a resource. |
| `EFF-0009` | A capability is a sealed nominal value created only by a provider trusted by the selected profile/host. Integer, byte, reflection, cloning, deserialization, and FFI operations cannot forge it. |
| `EFF-0010` | Capabilities may be attenuated to a strict subset of rights, scope, quota, duration, address/path range, protocol, or operation. Ordinary code cannot widen them. |
| `EFF-0011` | Delegation follows ownership: move transfers authority, borrow lends authority for the borrow lifetime, and explicit sub-capability creation delegates narrowed authority. |
| `EFF-0012` | Revocation is provider-defined and races are resolved at the authorized operation: the provider atomically accepts under the prior epoch or rejects as revoked. A check does not guarantee future use. |
| `EFF-0013` | Capability equality, hashing, serialization, and cross-process transfer are absent unless a provider-defined trait supplies an authenticated representation and import validation. |
| `EFF-0014` | Audit events disclose capability identity and operation metadata only to the extent declared by the audit capability; secret payloads are not automatically logged. |
| `EFF-0015` | Handlers are one-shot by default. A continuation can resume at most once and is affine. |
| `EFF-0016` | A multi-shot handler requires the effect declaration to be `multishot`, every captured continuation value to be clonable or persistent, and the `alloc` effect unless statically eliminated. |
| `EFF-0017` | Handler invocation preserves ownership, cancellation, and capability restrictions. Resumption cannot outlive captured borrows or detach child tasks. |
| `EFF-0018` | Deterministic code excludes unrecorded nondeterministic effects. Providers may satisfy such effects with explicit replay streams whose identity is recorded. |

## 4. Capability authenticity boundaries

Across a process, plugin, FFI, or distributed boundary, a capability is transferred only through a profile-defined authenticated handle exchange. Receiving bytes that resemble a handle does not create authority. Imports validate issuer, audience, rights, freshness, revocation epoch, and target resource binding.

## 5. Standard capability families

Edition 1 standard profiles define narrowed capabilities for files/directories, sockets/endpoints, process creation, console, monotonic and wall clocks, secure and pseudo randomness, environment variables, devices/MMIO/DMA, persistence transactions, accelerators, dynamic loading, reflection, audit, supervisors, and build inputs.


---

# OMNI-EVAL: Dynamic Evaluation Semantics

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-MACHINE`, `OMNI-TYPES`, `OMNI-OWN`, `OMNI-EFFECTS` |
| Date | `2026-08-04` |

## 1. Scope

Expression and statement operational semantics, places/values, calls, patterns, loops, closures, temporaries.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Dynamic judgment


- `Γ` is the lexical typing and name environment.
- `Ω` is the ownership and initialization state.
- `Ε` is the available effect/capability environment.
- `Σ` is the abstract store, including allocation identities and object lifetimes.
- `Μ` is the concurrent memory event graph.
- `Γ; Ω; Ε ⊢ e : T ! ε ⇒ Ω′` means expression `e` has type `T`, may perform effect row `ε`, and transforms ownership state `Ω` to `Ω′`.
- `⟨e, Σ, κ⟩ → ⟨e′, Σ′, κ′⟩` is one dynamic evaluation step.
- `hb` denotes happens-before; `sw` denotes synchronizes-with; `mo` denotes per-atomic modification order.


The evaluation relation is deterministic for a single task given the same store, provider responses, and explicit nondeterministic choices. Concurrent interleavings are constrained by OMNI-CONC and OMNI-MEM.

| Rule | Requirement |
|---|---|
| `EVAL-0001` | Operands, arguments, receiver expressions, aggregate fields, array elements, interpolation expressions, match guards, and chained postfix operations evaluate left-to-right. |
| `EVAL-0002` | `&&` and `||` evaluate the right operand only when required. `??` evaluates the right operand only when the left optional/result form is absent as defined by its trait. |
| `EVAL-0003` | A function call evaluates the callee, receiver if any, and arguments left-to-right; creates parameter bindings left-to-right; then activates any reserved receiver loan and enters the body. |
| `EVAL-0004` | Named arguments are reordered to parameter positions only after their source expressions have evaluated in source order. |
| `EVAL-0005` | An assignment first evaluates the left expression to a place without reading its old value, then evaluates the right expression, drops the old initialized destination value, and stores the new value. |
| `EVAL-0006` | A compound assignment evaluates the left place exactly once, reserves the required mutable access, evaluates the right operand, performs the trait operation, and writes/commits according to that trait contract. |
| `EVAL-0007` | A cast evaluates its operand once. Checked casts return `Option`; trapping casts panic with `ConversionFault`; bit casts follow OMNI-NUM and OMNI-UNSAFE validity rules. |
| `EVAL-0008` | Field and index access evaluate the base before the selector/index. Bounds are checked before producing a reference or reading/writing the element. |
| `EVAL-0009` | A block executes statements in source order. Its final un-terminated expression is the block value; otherwise its value is unit. |
| `EVAL-0010` | An `if` evaluates its condition once and then exactly one branch. Conditions require `bool`. |
| `EVAL-0011` | A `match` evaluates the scrutinee once into a temporary place, tests structural alternatives in source order semantics, evaluates a guard only after its pattern binds successfully, and commits moves only for the selected arm. |
| `EVAL-0012` | Or-pattern alternatives must bind the same names with the same types and binding modes. Failed alternatives leave the scrutinee unchanged. |
| `EVAL-0013` | A `loop` yields the value of a matching labeled `break`; all reachable breaks for a value-producing loop must coerce to one type. |
| `EVAL-0014` | `while` evaluates its condition before each iteration. `for` invokes `IntoIterator.into_iter` once and repeatedly calls `next` in source-defined sequence. |
| `EVAL-0015` | `continue` runs cleanup for scopes exited within the current iteration and begins the next iteration. `break` and `return` evaluate their value before cleanup. |
| `EVAL-0016` | `defer` registers a cleanup after its registration expression succeeds. Cleanups run in reverse registration order on normal transfer and supported unwind. |
| `EVAL-0017` | `async defer` may suspend only in an async scope and runs during asynchronous scope cleanup subject to cancellation masking bounds. |
| `EVAL-0018` | A closure capture mode is the least powerful mode required by its body: shared borrow, mutable borrow, or move. `move` forces capture by value. Capture expressions evaluate when the closure is created. |
| `EVAL-0019` | Closure parameters bind at invocation. A closure value is callable according to whether captures permit repeated shared calls, repeated mutable calls, or a single consuming call. |
| `EVAL-0020` | An async block constructs a lazy future and does not execute its body until polled. Captures occur at construction; body locals initialize on first poll as reached. |
| `EVAL-0021` | The postfix `?` performs the standard `Try` branch operation, returning the residual through the enclosing compatible function/try block after cleanup. |
| `EVAL-0022` | Panic, cancellation, and faults interrupt ordinary evaluation only at the exact operations defined by OMNI-ERROR, OMNI-CONC, or target/profile rules. |

## 4. Temporary scopes

A temporary normally lives until the end of the smallest enclosing full expression. A temporary borrowed by a grammar-defined `let` binding may be materialized as a hidden binding lasting to the end of the lexical scope when the binding pattern directly stores that borrow. No other lifetime extension occurs. The compiler SHALL display hidden bindings in expanded MIR and diagnostics.

## 5. Place evaluation

Place expressions include locals, statics, dereferences, field projections, indexing projections, and compiler-validated downcasts. Evaluating a place may fault for null/invalid raw pointers, bounds, alignment, capability, or device access only when the specific projection operation requires it.

## 6. Method calls

Method lookup follows OMNI-NAMES. Operator syntax maps to sealed or imported operator traits. Desugaring preserves evaluation order, borrow reservation/activation, effects, and diagnostics.


---

# OMNI-ERROR: Errors, Faults, Panic, Unwind, Cancellation

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-EVAL`, `OMNI-OWN`, `OMNI-EFFECTS` |
| Date | `2026-08-04` |

## 1. Scope

Result/Option, fault taxonomy, cleanup, double-fault behavior, OOM, stack exhaustion, cancellation.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Fault taxonomy

Defined fault categories are `ArithmeticFault`, `BoundsFault`, `ConversionFault`, `ContractFault`, `CapabilityFault`, `StackFault`, `AlignmentFault`, `InvalidValueFault`, `TargetFeatureFault`, `CancellationFault` (for adaptation only), and profile-specific device/persistence faults. Faulting operations either return a typed result by API contract or initiate panic/isolation/abort by artifact policy.

| Rule | Requirement |
|---|---|
| `ERR-0001` | Expected failure is represented by `Result<T,E>`; absence by `Option<T>`. Unchecked exceptions are not part of ordinary APIs. |
| `ERR-0002` | A panic payload is a typed `PanicInfo` containing stable fault category, optional static message ID, source location, cause chain, and profile-controlled backtrace token. |
| `ERR-0003` | Catchability is explicit: `catch panic` requires the `unwind` effect and catches only Omni panics within an unwind-enabled artifact, not aborts, stack faults, hardware traps, or foreign exceptions unless adapted. |
| `ERR-0004` | Artifact panic policies are `abort`, `unwind`, or `isolate`. The policy and unwind ABI are fingerprinted. |
| `ERR-0005` | On unwind, initialized locals and registered defers are cleaned in reverse order subject to destructor rules. A destructor that panics while another panic/unwind is active causes immediate artifact-policy abort or isolate termination; the second panic is recorded. |
| `ERR-0006` | Destructors are not permitted to return errors. Fallible cleanup is an explicit method called before destruction. |
| `ERR-0007` | Unwind-safe types are those whose invariants remain valid if a protected operation unwinds. Safe catch APIs require `UnwindSafe` or an explicit assertion wrapper. |
| `ERR-0008` | Poisoning of locks/cells is a library contract, not automatic language behavior. Standard mutexes mark poison when a panic exits a locked critical section and allow explicit recovery. |
| `ERR-0009` | Primitive allocation is fallible and returns `Result<..., AllocError>`. Infallible wrappers invoke the artifact OOM policy only after requested cleanup/retry hooks return failure. |
| `ERR-0010` | OOM policy is `abort`, `panic`, or `isolate`; panic/isolate OOM handling uses an emergency reserve and cannot attempt unbounded allocation. If reserve is unavailable, abort is permitted. |
| `ERR-0011` | Stack exhaustion produces `StackFault` only on targets with reliable guard/detection and a safe emergency stack; otherwise the target supplement mandates immediate abort. Recovery cannot resume the exhausted frame. |
| `ERR-0012` | Cancellation is a distinct control signal. It is observed only at declared cancellation points unless a target/profile explicitly defines asynchronous cancellation for unsafe compartments. |
| `ERR-0013` | When success/error/panic/cancellation race, the first event atomically committed by the operation protocol wins; later events become suppressed causes or are ignored according to that protocol. |
| `ERR-0014` | A completed successful result cannot be replaced by cancellation after its completion commit. Cancellation requested before commit may win at the next cancellation point. |
| `ERR-0015` | Cleanup runs under a bounded cancellation mask. A cleanup that exceeds its profile limit triggers the scope policy and may escalate to isolate/abort in realtime profiles. |
| `ERR-0016` | `isolate` terminates the current task/actor compartment, cancels children, runs only isolation-safe cleanups, revokes compartment capabilities where supported, and reports to the supervisor. |
| `ERR-0017` | Panic never crosses an ABI boundary lacking an explicit compatible panic protocol; the boundary converts, traps, isolates, or aborts as declared. |
| `ERR-0018` | Backtrace addresses are not stable semantic observations; stable frame identities are symbol/source IDs exposed only when the debug profile is enabled. |

## 4. Cleanup precedence

For any control transfer, the transfer value/cause is first committed to a hidden slot, then cleanups execute. Cleanup panic supersedes ordinary success/error/cancellation and chains the prior cause. A second panic during active unwind invokes the double-panic rule. Resource failure during cleanup is handled by the cleanup API’s declared result or escalates through the artifact policy.

## 5. Contracts

Failed runtime `require`, `ensure`, invariant, or resource contracts produce `ContractFault` unless the contract explicitly returns a typed validation error. Statically proved contracts emit no runtime check. Trusted assumptions are allowed only at unsafe/FFI boundaries and create named obligations.


---

# OMNI-CONST: Compile-Time Evaluation, Macros, and Reflection

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-GRAMMAR`, `OMNI-EVAL`, `OMNI-ERROR` |
| Date | `2026-08-04` |

## 1. Scope

Const evaluator, macro phases/hygiene, reflection, generated code, determinism, quotas, diagnostics.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Const eligibility

A function is const-callable when declared `const fn`, all reached operations are const-supported, its effect row is empty except permitted compile-time allocation/panic, and every called function is const-callable. Termination is enforced by a deterministic fuel and recursion-depth budget, not assumed.

| Rule | Requirement |
|---|---|
| `CONST-0001` | Const evaluation uses the same value, arithmetic, ownership, pattern, panic, and pure call semantics as runtime, with deterministic quotas and no undeclared external effects. |
| `CONST-0002` | Const evaluation may allocate immutable interpreter objects. Their addresses are abstract and cannot be converted to stable integers or compared across independent evaluations. |
| `CONST-0003` | A const pointer may refer only to a live const allocation or static object permitted by its type. Relocation records preserve provenance into the emitted artifact. |
| `CONST-0004` | Const panic rejects the containing constant/item with a required diagnostic; it does not emit a runtime panic unless the source explicitly requests deferred checking. |
| `CONST-0005` | Macro phases are: lexical token macros, item/declaration macros, target/profile filtering, import resolution, typed derive macros, and expression/type macros at their grammar positions. |
| `CONST-0006` | Macro input and output are token trees or typed semantic objects according to the declared phase. Textual preprocessing and arbitrary source concatenation are prohibited. |
| `CONST-0007` | Hygiene assigns syntax-context marks to introduced identifiers. Introduced names resolve at definition context; passed-through names retain call-site context. |
| `CONST-0008` | Deliberate capture uses explicit `capture(callsite, name)` or `capture(defsite, name)` APIs and is visible in expansion output. |
| `CONST-0009` | Generated identifiers carry source span, macro invocation, definition, expansion index, and stable generation key. |
| `CONST-0010` | Macro expansion is deterministic, cycle-checked, and limited by declared token, depth, time-instruction, and memory quotas. Exceeding a quota is a compile-time diagnostic. |
| `CONST-0011` | Compile-time file/resource access requires a declared build capability and content digest. Directory enumeration is sorted and captured as an input. |
| `CONST-0012` | Compile-time network, wall clock, ambient environment, process creation, and unseeded randomness are forbidden in reproducible mode. |
| `CONST-0013` | Reflection sees only declarations visible at the reflection site plus explicitly exported metadata. Private members cannot be reflected across package boundaries without a capability/attribute. |
| `CONST-0014` | Runtime reflection metadata is linked only for types/items reachable from an explicit `reflect` root or dynamic boundary. |
| `CONST-0015` | Compile-time and runtime type identities use the same canonical signature fingerprint where runtime identity is opted in. |

## 4. Macro declaration contract

A macro declares accepted fragment kinds, output fragment kind, phase, determinism class, required build capabilities, resource limits, and edition. Expansion output is parsed/validated under the same edition and cannot bypass static semantics.

## 5. Reflection contract

Compile-time reflection returns immutable descriptors, not mutable compiler internals. Descriptor schemas are versioned. Layout is visible only for an explicit `repr`, target-specific query, or after layout finalization; querying layout makes the target/layout input part of the build key.


---

# OMNI-UNSAFE: Unsafe Semantics and Obligations

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-EVAL`, `OMNI-OWN`, `OMNI-MEM` |
| Date | `2026-08-04` |

## 1. Scope

Unsafe operations, obligation language, invalid-execution scope, raw memory, MMIO, DMA, inline assembly.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Unsafe obligation model

An obligation is `O = (id, operation, precondition, scope, assumptions, containment)`. The compiler emits obligation IDs into MIR and optional object metadata. Safe wrapper documentation SHALL list every obligation it discharges and the invariant used.

| Rule | Requirement |
|---|---|
| `UNSAFE-0001` | `unsafe` permits performing operations with programmer-proved preconditions; it does not disable ordinary typing, ownership, effect, capability, initialization, or control-flow checks. |
| `UNSAFE-0002` | Unsafe operations are legal only inside an `unsafe` block/function or through an unsafe trait implementation, and their unsafe effect is visible in MIR and audit reports. |
| `UNSAFE-0003` | Calling an unsafe function requires an unsafe context even when the caller can prove its preconditions; safe wrappers discharge and document the proof. |
| `UNSAFE-0004` | Each unsafe operation has a named obligation record with precondition, protected invariant, permitted optimizer assumptions, and failure containment class. |
| `UNSAFE-0005` | If an unsafe precondition is false at event `e`, that execution becomes invalid from `e` onward. Observations before `e` remain required. Other executions and paths not reaching `e` retain full semantics. |
| `UNSAFE-0006` | An optimizer may assume an unsafe precondition only on paths that reach the operation and only while the facts/objects on which it depends remain live. Assumption provenance SHALL point to the obligation. |
| `UNSAFE-0007` | Invalid unsafe execution does not authorize compile-time compromise, cross-process effects, or modification of immutable artifacts; such outcomes are outside the language model and addressed by hardening. |
| `UNSAFE-0008` | Raw allocation requires `(size, alignment, address-space, allocator-family)` and returns a provenance-bearing allocation handle or failure. Zero-size allocations use a non-dereferenceable aligned sentinel or a declared unique object. |
| `UNSAFE-0009` | Deallocation requires the original live allocation identity, compatible allocator family, exact or accepted layout, no live derived references, and no outstanding device/foreign ownership. |
| `UNSAFE-0010` | Raw dereference requires live allocation provenance, in-bounds access, alignment unless using unaligned operations, valid address space, permissions, and a valid value for typed reads. |
| `UNSAFE-0011` | Inline assembly declares target, feature predicates, inputs, outputs, late outputs, clobbered registers, flags, memory regions, stack behavior, unwind behavior, control-flow successors, and volatility. |
| `UNSAFE-0012` | Assembly marked `pure`/`nomem`/`readonly` is verified syntactically where possible and remains an unsafe promise. Omitted memory clobbers cannot be inferred. |
| `UNSAFE-0013` | Assembly may branch only to declared labels, may unwind only with an explicit compatible unwind contract, and may not silently alter stack pointer, capability registers, TLS, or reserved runtime registers. |
| `UNSAFE-0014` | MMIO values use volatile device operations at declared widths and endianness. Ordinary references to MMIO storage are prohibited. |
| `UNSAFE-0015` | DMA registration pins or otherwise stabilizes buffers, establishes device-visible address mapping, records direction/coherence, and transfers a typed DMA lease. CPU access while leased follows the lease contract. |
| `UNSAFE-0016` | Device reset/revocation invalidates DMA leases atomically from the provider perspective; cleanup must tolerate completion/reset races without reusing storage early. |

## 4. Unchecked and checked unsafe modes

`unsafe.checked` retains runtime validation/sanitizers where available and converts violations to a defined trap before memory corruption. `unsafe.unchecked` permits omission of those checks using the obligation assumptions. Hardened and high-assurance profiles may prohibit unchecked unsafe or require compartment containment.

## 5. Transmutation

Typed transmutation is allowed only when source and destination have equal size, compatible alignment/storage, every destination validity invariant is proved, provenance/ownership is preserved, and no private layout is assumed without explicit `repr`. Otherwise conversion proceeds through bytes plus validation.


---

# OMNI-CONC: Concurrency and Async Semantics

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-EVAL`, `OMNI-ERROR`, `OMNI-OWN`, `OMNI-EFFECTS` |
| Date | `2026-08-04` |

## 1. Scope

Threads, tasks, scopes, actors, channels, blocking, progress, fairness, TLS, signals/interrupt interaction.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Structured task state

A child state is `created`, `runnable`, `waiting`, `completed(value)`, `failed(error)`, `panicked(info)`, or `cancelled`. Terminal transition is atomic. A scope maintains spawn order and completion order as separate sequences.

| Rule | Requirement |
|---|---|
| `CONC-0001` | Native threads, scoped tasks, async tasks, actors, and interrupts are distinct execution agents with profile-defined capabilities and memory participation. |
| `CONC-0002` | Thread/task creation publishes all moved arguments and initialized captured state to the child start; child completion synchronizes with successful join. |
| `CONC-0003` | A structured task scope cannot complete until every child is joined, cancelled and joined, or transferred to an authorized long-lived supervisor. |
| `CONC-0004` | Standard scope policies are `all`, `cancel_on_error`, `first_success`, `race`, `supervise`, and `quorum(k)` with the exact aggregation rules below. |
| `CONC-0005` | `all` waits for all children and returns ordered results by spawn order. `cancel_on_error` commits the first error/panic by completion order, requests cancellation of unfinished siblings, joins them, and returns the committed cause with suppressed causes. |
| `CONC-0006` | `first_success` returns the first committed success, cancels and joins others; if none succeeds, it aggregates terminal causes by completion order. `race` returns the first committed terminal outcome. |
| `CONC-0007` | `quorum(k)` commits once `k` successes exist, returns them in completion order, and cancels/joins remaining children; impossibility to reach `k` returns aggregated failure. |
| `CONC-0008` | Detach requires a supervisor/service capability and moves all captures. Detached work becomes a child of that supervisor, never an orphan. |
| `CONC-0009` | Bounded channels have capacity `N`; send commits when an element enters the buffer or is handed directly to a receiver. Receive commits when ownership of one element transfers to the receiver. |
| `CONC-0010` | Closing a channel prevents new successful sends; buffered elements remain receivable. After the buffer empties, receive returns `Closed`. Concurrent close/send is decided by the operation’s atomic commit order. |
| `CONC-0011` | Channel ordering is FIFO per sender and globally FIFO by send commit order for a single channel. Fairness is not guaranteed unless the channel type/profile states it. |
| `CONC-0012` | Cancellation before a channel operation commits leaves no element transferred. Cancellation after commit returns/records the committed result and cannot roll it back. |
| `CONC-0013` | Standard mutex lock acquisition may be unfair and may wake spuriously only through condition-variable waits, not lock success. Fair mutexes are separate types. |
| `CONC-0014` | Condition-variable wait atomically releases the mutex and blocks, may wake spuriously, then reacquires before returning. Callers must test predicates in a loop. |
| `CONC-0015` | Semaphores commit permit decrement/increment atomically; cancellation before acquire commit consumes no permit. |
| `CONC-0016` | Standard mutexes poison after panic exits a held critical section. Poison is advisory and recoverable through an explicit API; it is not memory unsafety. |
| `CONC-0017` | An executor SHALL not poll one task concurrently with itself, SHALL tolerate duplicate/coalesced wakes, and SHALL not poll after terminal completion. |
| `CONC-0018` | Polling a future is non-reentrant unless the future explicitly implements a reentrant protocol. A wake during poll schedules a later poll and does not recursively enter the task. |
| `CONC-0019` | Blocking operations in an async context require the `block` effect and a provider that routes them to a blocking resource or rejects them. |
| `CONC-0020` | Actor mailboxes preserve message commit order per sender; cross-sender order is by mailbox commit. A restart never rolls back external effects already committed. |
| `CONC-0021` | Region transfer in a message commits ownership atomically with enqueue. On enqueue failure ownership remains with the sender; on committed delivery failure the supervisor policy owns disposal/retry. |
| `CONC-0022` | Thread-local values initialize per native thread, are inaccessible from tasks migrated between threads unless task-local storage is used, and drop at orderly thread exit only. |
| `CONC-0023` | Signals/interrupts may access only async-signal/interrupt-safe operations declared by the target/profile. Ordinary locks, allocation, and destructors are prohibited unless explicitly supported. |

## 4. Progress classifications

APIs declare one of: blocking, obstruction-free, lock-free, wait-free, bounded-wait, or realtime-bounded. Absence of a declaration means only safety and eventual response under provider progress; fairness is not implied.

## 5. Async lowering

An async body lowers to a pinned state machine containing live-across-suspension locals and cleanup states. Every suspension point checks borrow/pin validity. Dropping the future begins cancellation cleanup and cannot silently detach child scopes.

## 6. Deterministic concurrency

The deterministic profile permits disjoint parallel mutation, deterministic reductions, explicitly ordered channels, recorded arbitration, and replayed external events. General scheduler-dependent shared-memory outcomes require the `nondeterministic` effect and are rejected otherwise.


---

# OMNI-MEM: Memory, Atomics, and Provenance Model

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-MACHINE`, `OMNI-OWN`, `OMNI-CONC` |
| Date | `2026-08-04` |

## 1. Scope

Allocation objects, validity, provenance, atomics, happens-before, races, tearing, volatile/device/persistent memory.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Event model

Memory events are reads, writes, read-modify-writes, fences, lock/unlock, spawn/start, completion/join, channel transfer, cancellation publication, volatile/device operations, and persistent flush/commit. Relations include sequenced-before (`sb`), synchronizes-with (`sw`), happens-before (`hb = (sb ∪ sw)+`), modification order (`mo`), reads-from (`rf`), and from-read (`fr`).

| Rule | Requirement |
|---|---|
| `MEM-0001` | An allocation has a unique allocation identity, address space, size, alignment, lifetime interval, permissions, and storage bytes. Reuse creates a new identity even at the same address. |
| `MEM-0002` | Object lifetime begins only after storage is allocated and initialization establishes a valid value; it ends before destruction releases/reuses storage. `MaybeUninit` storage has allocation lifetime but no `T` object lifetime until initialized. |
| `MEM-0003` | A pointer carries `(allocation-id or exposed-origin, address-space, offset/address, bounds, permissions, provenance-state)` abstractly; target representations may encode less but must preserve semantics. |
| `MEM-0004` | Derived pointers retain provenance and may narrow bounds/permissions. Pointer arithmetic is valid within the allocation and may form one-past; one-past cannot be dereferenced. |
| `MEM-0005` | Pointer subtraction/order is defined only for pointers into the same live allocation (including one-past) and compatible element type/address space. Equality has a named address-only form and a provenance-aware identity form. |
| `MEM-0006` | `expose_addr` emits an integer address and marks provenance as exposed. `from_exposed_addr` may recover dereference authority only if a live exposed allocation in the address space contains the address and policy permits recovery; ambiguity is an unsafe error unless an allocation handle is supplied. |
| `MEM-0007` | Integer casts that do not use exposure/recovery APIs do not create dereferenceable provenance. |
| `MEM-0008` | A data race occurs when two non-atomic conflicting memory events from different agents are not ordered by happens-before, at least one writes, and the accesses overlap in storage. Volatile does not make an access atomic. |
| `MEM-0009` | Unsafe, foreign, signal, interrupt, and device agents participate in race analysis according to their declared event contracts. A violated synchronization contract creates invalid unsafe execution. |
| `MEM-0010` | Concurrent mixed-size overlapping accesses are races unless every access is atomic and the target supplement defines the combination. Atomic tearing is forbidden for supported atomic widths/alignment. |
| `MEM-0011` | Each atomic object has one modification order. Reads select a write allowed by the ordering axioms; compare-exchange has separate success/failure ordering and failure cannot be release or acquire-release. |
| `MEM-0012` | Release stores and successful release operations synchronize with acquire reads/RMWs that read from their release sequence. Fences synchronize only through the specified atomic communication pattern. |
| `MEM-0013` | Sequentially consistent operations participate in one total order consistent with happens-before and modification order. |
| `MEM-0014` | A data-race-free safe program using only SC atomics has sequentially consistent abstract behavior. Relaxed atomics expose only outcomes permitted by this model. |
| `MEM-0015` | Atomic supported widths, alignments, and lock-free status are target-manifest facts. Unsupported widths use a conforming library lock or are rejected by a lock-free requirement. |
| `MEM-0016` | Ordinary shared immutable memory is safe after publication. Shared mutable memory requires atomics or synchronization abstractions. |
| `MEM-0017` | Volatile/device accesses occur exactly once at their abstract operation, are not merged, invented, or reordered across declared device barriers, and may produce target/device faults. |
| `MEM-0018` | Persistent durability order is separate from coherence. Stores become durable only after profile-defined flush and drain/commit operations; crash observations include only durability-committed records. |
| `MEM-0019` | Address spaces are nominal. Pointers from distinct address spaces do not convert implicitly and may have different width, provenance, and accessibility. |
| `MEM-0020` | Capability-pointer targets preserve bounds/permissions/tag validity; copying raw bytes does not necessarily copy a valid capability tag. Tagged-memory behavior is declared by target supplement. |
| `MEM-0021` | Safe code cannot observe uninitialized bytes, padding, invalid discriminants, dangling provenance, or IR poison. Raw byte views require validation/unsafe contracts. |

## 4. Atomic order semantics

- `relaxed`: atomicity and modification order only;
- `acquire`: prevents later ordinary/atomic events from moving before and imports writes from the release sequence read;
- `release`: publishes prior events to an acquiring reader of its release sequence;
- `acq_rel`: both for successful read-modify-write;
- `seq_cst`: acq_rel/acquire/release as appropriate plus membership in the SC total order.

Consume ordering is not part of Edition 1.

## 5. DRF-SC scope

The DRF-SC guarantee covers safe ordinary memory and synchronization primitives whose unsafe internals meet this model. It excludes explicit relaxed outcomes, foreign/device contracts that declare weaker ordering, invalid unsafe executions, and persistent durability observations.

## 6. Hardware mapping

Target supplements SHALL provide litmus-tested mappings to x86-TSO, AArch64, RVWMO, and other claimed models. A backend may strengthen ordering but may not weaken it.


---

# OMNI-FFI: Foreign Function and Memory Interfaces

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-TYPES`, `OMNI-OWN`, `OMNI-ERROR`, `OMNI-UNSAFE`, `OMNI-MEM` |
| Date | `2026-08-04` |

## 1. Scope

Foreign values, calls, callbacks, ownership, validation, varargs, exceptions, longjmp, TLS, foreign threads.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Boundary principle

The FFI boundary is a validation and ownership transition, not a magical exemption. Raw foreign representation may be received into raw/`MaybeUninit` forms; constructing a safe Omni value requires validation before any safe operation observes it.

| Rule | Requirement |
|---|---|
| `FFI-0001` | An FFI declaration names an ABI string and exact foreign type representation. Unknown ABI strings are errors in strict mode. |
| `FFI-0002` | Foreign scalar values entering safe Omni types are validated: `bool` is 0/1, `char` is a Unicode scalar, references satisfy non-null/alignment/liveness/provenance, and enum tags are valid. |
| `FFI-0003` | Nullable foreign pointers map to raw/optional pointer representations, never directly to non-null safe references without validation. |
| `FFI-0004` | Foreign aggregates use explicit `repr(C)`, `repr(omni_v1)`, or schema representations. Padding bytes are not read for equality/hash/serialization and may be uninitialized only behind raw storage types. |
| `FFI-0005` | Varargs are permitted only for target ABIs with a supplement and require each variadic argument to use the ABI-promoted foreign type explicitly. Safe generic values cannot be passed directly. |
| `FFI-0006` | Callbacks expose an ABI function pointer plus context handle and lifetime/ownership protocol. The foreign side may call only during the declared interval unless it owns a transferred callback object. |
| `FFI-0007` | A foreign thread calling Omni shall attach through the runtime ABI before touching Omni TLS, managed domains, panic, async, or stack-map facilities, and detach after all Omni frames/resources are gone. |
| `FFI-0008` | Foreign exceptions, SEH, Objective-C exceptions, and `longjmp` shall not cross Omni frames unless a target supplement defines a compatible adapter. Default behavior is boundary trap/abort or conversion by wrapper. |
| `FFI-0009` | `setjmp`/`longjmp` across live Omni owned values is prohibited because it skips destruction; wrappers may isolate the jump within a foreign frame and convert the result. |
| `FFI-0010` | POSIX signals and hardware exceptions use target-defined adapters and cannot call arbitrary Omni code. |
| `FFI-0011` | Panic cannot cross a foreign boundary without an explicit panic ABI. Callback wrappers catch/convert/isolate/abort according to the declaration. |
| `FFI-0012` | Ownership transfer annotations are `borrowed`, `borrowed_mut`, `consumed`, `returned_owned`, `shared_retain`, or custom handle protocol. Every boundary value has one annotation. |
| `FFI-0013` | Memory allocated by one allocator family may be freed by another only through an explicit compatibility contract. Stable buffers carry a deallocator callback/allocator handle when ownership crosses modules. |
| `FFI-0014` | Foreign memory imported as a safe slice/reference requires validated length, alignment, lifetime token, mutability/exclusivity, address-space accessibility, and provenance handle. |
| `FFI-0015` | Managed objects cross FFI only by pinned reference, stable handle, or copy. A raw pointer into movable storage cannot survive a safepoint. |
| `FFI-0016` | FFI calls carry the `foreign` effect plus every declared external effect. Foreign purity/noexcept/read-only attributes are unsafe promises subject to validation/audit. |

## 4. ABI declaration example

```omni
extern "C" {
    #[ffi(ownership="borrowed", null="forbidden")]
    fn write(fd: c_int, data: *const byte, len: usize) -> isize !{foreign, io};
}
```

## 5. Foreign callback lifetime

A callback token is affine. Revocation waits for or prevents new entries according to its synchronization protocol. Destroying the token before the foreign side relinquishes callback authority is an unsafe contract violation.


---

# OMNI-IR: Normative Semantic IRs

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `toolchain-required` |
| Dependencies | `OMNI-EVAL`, `OMNI-ERROR`, `OMNI-MEM` |
| Date | `2026-08-04` |

## 1. Scope

HIR/TIR/OIR/MIR/LIR schemas, verifiers, semantics, serialization, source/proof provenance.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Authority and levels

| Rule | Requirement |
|---|---|
| `IR-0001` | Source-language semantics are primary. Normative IRs are executable elaborations that SHALL be proven or differentially cross-checked against source rules and cannot redefine them. |
| `IR-0002` | Each IR has a versioned schema, verifier, type/effect/ownership invariants, source mapping, and feature fingerprint. |
| `IR-0003` | HIR contains resolved syntax and desugared constructs while preserving generics and source ownership notation. |
| `IR-0004` | TIR contains fully explicit types, coercions, effects, capabilities, contracts, numeric policy, trait selections, and dynamic boundaries. |
| `IR-0005` | OIR contains moves, copies, borrows, loans, regions, initialization state, drop flags, pinning, suspension liveness, and unsafe obligations. |
| `IR-0006` | MIR is typed SSA/CFG with explicit memory objects, checks/fault edges, cleanup/unwind/cancellation edges, calls, effects, and provenance operations. |
| `IR-0007` | Domain IRs preserve affine loops, vectors, tensors, sparse layouts, async machines, storage transactions, protocols, and cryptographic contracts until validated lowering. |
| `IR-0008` | LIR contains explicit layout, address spaces, atomics, fences, ABI calls, stack maps, exception tables, and runtime hooks but remains target-neutral. |
| `IR-0009` | Machine IR contains target instructions, registers, scheduling dependencies, stack frames, relocations, feature predicates, and debug/unwind mapping. |
| `IR-0010` | No IR value originating from safe source may become observable `undef` or poison. Internal invalid states are verifier errors or explicit assumptions with provenance and freeze/validation before observation. |
| `IR-0011` | Uninitialized storage is represented by a storage state, not an arbitrary typed value. Loads require initialized bytes/value validity or an unsafe raw operation. |
| `IR-0012` | IR serialization uses canonical CBOR-like schema rules defined in `schemas/ir-module.schema.json`, content hashes, and forward-compatible unknown-field rejection for normative fields. |
| `IR-0013` | Every source-to-IR elaboration rule records the source rule IDs it implements. The reference evaluator executes MIR only after verification. |
| `IR-0014` | Optimizer passes consume and emit verified IR; verifier failure is a compiler defect and no artifact is emitted. |

## 4. MIR core operations

MIR defines constants, aggregates, projections, checked arithmetic, comparisons, casts, allocation/deallocation, lifetime start/end, load/store, borrow/reborrow/end-loan, move/copy/drop, call/invoke, branch/switch, panic/fault, spawn/join/channel, atomic/fence, volatile/device, capability invoke, and return/abort. Every operation has operand types, effects, preconditions, normal successors, and exceptional successors.

## 5. MIR reference execution

The reference engine interprets verified MIR with the OMNI-MACHINE store and OMNI-MEM event graph. It is intentionally unoptimized and deterministic given explicit external/scheduler inputs. Source-vs-MIR differential cases are normative conformance evidence.

## 6. Preservation obligations

For each lowering `L`, accepted source state `s`, and generated IR `i=L(s)`, every permitted IR observation is a permitted source observation, and every required source termination/fault is represented. Aggressive lowerings may refine nondeterminism but cannot introduce disallowed observations.


---

# OMNI-OPT: Optimization Legality and Validation

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `toolchain-required` |
| Dependencies | `OMNI-IR`, `OMNI-MACHINE`, `OMNI-MEM` |
| Date | `2026-08-04` |

## 1. Scope

Permitted transformations, observable equivalence, assumptions, translation validation, proof certificates.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Observational refinement

For original program `P` and transformed program `P′`, legality requires `Obs(P′, I) ⊆ Obs(P, I)` for every conforming input/environment `I`, and equality where the source requires a unique observation. Refinement may choose among explicitly unspecified outcomes but cannot choose an outcome the source forbids.

| Rule | Requirement |
|---|---|
| `OPT-0001` | An optimization is legal only when the transformed artifact refines or equals the source permitted observation set under the selected profiles and target facts. |
| `OPT-0002` | Potential observations include panic/fault occurrence and ordering, allocation failure when allocation is semantically observable, destructor effects, I/O/capability operations, atomics/synchronization, volatile/device events, persistent commits, cancellation points, and numeric results under policy. |
| `OPT-0003` | Allocations may be removed when identity/address/OOM/destructor/trace effects are unobservable by contract. The optimizer cannot assume allocation always succeeds unless a proof or infallible policy permits it. |
| `OPT-0004` | Checks may be removed only when proved redundant from types, refinements, dominating checks, target guarantees, or explicit unsafe obligations. |
| `OPT-0005` | Reassociation, contraction, vectorization, and reduction reordering obey the lexical numeric policy and determinism profile. |
| `OPT-0006` | Concurrency transformations preserve happens-before, atomic modification order constraints, data-race freedom, cancellation/suspension points declared observable, and progress contracts. |
| `OPT-0007` | Assumptions carry origin rule/obligation, operands, scope, dominance, object lifetime, invalidation events, and proof status. Stale assumptions are verifier errors. |
| `OPT-0008` | Profile data affects profitability, layout, and dispatch but never semantic legality. Its source/IR hash, collection target, counters, merge algorithm, and age are recorded. |
| `OPT-0009` | Autotuning/ML-guided choices use a deterministic pinned model/seed in reproducible mode, have a deterministic fallback, and submit selected schedules to the same legality validator. |
| `OPT-0010` | Translation-validation classes are `proved`, `validated-complete`, `validated-bounded`, `differential-only`, and `unvalidated`. |
| `OPT-0011` | Verified/high-assurance builds reject `unvalidated` transformations and may restrict bounded validators to proved domains. Balanced builds report coverage and use conservative fallback when validation is unsupported or times out. |
| `OPT-0012` | Validation failure rejects the transformed result and reruns a conservative pipeline; repeated mismatch is a compiler defect and blocks release qualification. |
| `OPT-0013` | Post-link optimization preserves symbols, unwind/debug semantics required by profile, feature dispatch, control-flow integrity, and relocation/loader contracts. |

## 4. Optimization reports

The compiler emits machine-readable remarks for inlining, devirtualization, check elimination/retention, allocation/copy/retain/drop, vectorization, loop transforms, dispatch, code layout, profile use, and validation status. Remarks cite source and rule IDs.

## 5. No release-mode semantic changes

Optimization level cannot change overflow, bounds, aliasing, panic, race, cancellation, floating policy, or unsafe preconditions. Separate source/profile annotations choose alternate semantics.


---

# OMNI-TARGET: Target Identity and Feature Registry

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `platform-required` |
| Dependencies | `OMNI-MACHINE`, `OMNI-RULES` |
| Date | `2026-08-04` |

## 1. Scope

Target triples, data models, endianness, address spaces, ISA/OS features, deployment policy, multiversioning.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Target triple grammar

```text
arch        = x86_64 | aarch64 | riscv64 | registered-name
vendor      = pc | apple | unknown | registered-name
os          = linux | windows | macos | none | registered-name
environment = gnu | musl | msvc | elf | registered-name
abi         = omni_v1 | c | platform-name
data-model  = lp64 | llp64 | ilp32 | cap128 | registered-name
```

| Rule | Requirement |
|---|---|
| `TARGET-0001` | A canonical target identity is `arch-vendor-os-environment-abi[data-model]+feature-set`, with lowercase registered components and sorted feature names. |
| `TARGET-0002` | Aliases resolve through the signed target registry to exactly one canonical identity and cannot depend on the host machine. |
| `TARGET-0003` | Target manifests declare byte order, scalar widths/alignments, pointer widths per address space, maximum object size, stack alignment, atomic widths/lock-freedom, vector model, object format, ABI supplement, unwind/debug formats, and feature detection method. |
| `TARGET-0004` | Baseline feature sets are explicit. A distribution artifact cannot execute an instruction outside its baseline except within a correctly guarded multiversioned body. |
| `TARGET-0005` | Runtime feature detection is trusted only through the OS/firmware mechanism named by the target manifest; unprivileged self-reporting may further narrow but not widen authorized features. |
| `TARGET-0006` | Feature dispatch chooses a body whose required feature set is a subset of detected and deployment-policy-authorized features. Selection is cached with thread-safe initialization. |
| `TARGET-0007` | Target-specific semantics are limited to implementation-defined data layout, ABI/platform facilities, conditional instructions, and classified hardware faults. Core expression meaning remains unchanged. |
| `TARGET-0008` | Unsupported target features produce translation rejection or `TargetFeatureFault` before entering the body; illegal instruction traps are not an acceptable dispatch mechanism in safe hosted code. |
| `TARGET-0009` | Address spaces are registered with width, representation, accessibility, coherence, atomicity, provenance recovery, and conversion rules. |

## 4. Edition 1 reference targets

The suite includes target manifests for x86-64 Linux ELF SysV, x86-64 Windows PE/COFF, AArch64 Linux ELF, AArch64 macOS Mach-O, RV64GC Linux ELF, and freestanding AArch64/RV64 reference images. A target claim is conforming only with its exact supplement and test results.


---

# OMNI-ABI: Common Stable Omni ABI

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `platform-required` |
| Dependencies | `OMNI-TYPES`, `OMNI-OWN`, `OMNI-ERROR`, `OMNI-FFI`, `OMNI-TARGET` |
| Date | `2026-08-04` |

## 1. Scope

Stable data/call ABI, mangling, visibility, version negotiation, ownership, trait objects, async boundary.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Stable data and call ABI

| Rule | Requirement |
|---|---|
| `ABI-0001` | The stable ABI name is `omni_v1`. ABI use requires explicit `extern "omni_v1"` or `#[repr(omni_v1)]`; default native layout/calling is not stable. |
| `ABI-0002` | `omni_v1` supports LP64 little-endian, LLP64 little-endian, and registered capability variants. Endianness is part of ABI identity; cross-endian calls require serialization. |
| `ABI-0003` | Fixed integers use their exact width/alignment. `bool` is one byte 0 or 1. `char` is `u32` containing a Unicode scalar. `usize/isize` follow the data model. |
| `ABI-0004` | Aggregates use declaration order with deterministic padding and target-supplement alignment. Padding is not part of equality/hash/wire data and is zeroed only when the ABI contract requests hardened zero-padding. |
| `ABI-0005` | Enums use a stable explicit tag of the smallest declared tag type plus a union payload aligned to the maximum variant; niche optimization is not used unless `repr(nullable)` explicitly selects the standardized nullable representation. |
| `ABI-0006` | `Option<T>` and `Result<T,E>` use stable tagged layouts by default. Nullable pointer-like Option uses null only under `repr(nullable)` and when `T` has exactly one standardized invalid null representation. |
| `ABI-0007` | A slice/`str` ABI value is `{data: nonnull pointer, len: usize}`; zero length may use the registered aligned dangling sentinel. `str` bytes are valid UTF-8. |
| `ABI-0008` | An owned cross-boundary buffer is `{data, len, capacity, allocator_handle, drop_fn}`. Ordinary `String`/`Vec` private layouts are not ABI-stable unless wrapped by this descriptor. |
| `ABI-0009` | A trait object is `{data, vtable}`. The vtable begins with version, type fingerprint, size, alignment, drop function, and method count followed by methods in trait declaration order including inherited linearization. |
| `ABI-0010` | A closure object is `{env, invoke_fn, drop_fn, clone_fn?}` with call capability encoded by which functions are non-null. |
| `ABI-0011` | A dynamic value is `{type_fingerprint, data, descriptor}` where the descriptor supplies validation, drop, clone, equality/hash if supported, and reflection schema. |
| `ABI-0012` | An async ABI value is a pinned future object `{state, poll_fn, cancel_fn, drop_fn, descriptor}`. Poll uses an explicit context/waker ABI and returns `pending`, `ready`, or `panicked/failed` according to signature. |
| `ABI-0013` | Symbol mangling is `_O1` followed by length-prefixed UTF-8 NFC package/module/item components, kind/signature codes, ABI/profile fingerprint, and a 128-bit BLAKE3-derived collision suffix encoded base32. |
| `ABI-0014` | Demangling is deterministic. A linker detects full canonical-signature collisions even if the suffix collides and rejects the artifact. |
| `ABI-0015` | ABI evolution is additive within `omni_v1` only for reserved fields/vtable tails negotiated by size/version. Breaking layout/call changes require `omni_v2`. |
| `ABI-0016` | Symbol versioning and interface fingerprints allow loaders to reject incompatible libraries before calling code. |
| `ABI-0017` | Panic, ownership, effects, capabilities, target features, numeric policy, and profile requirements crossing an ABI boundary are part of the exported signature descriptor. |

## 4. Calling convention

The common ABI defines logical argument classification. Target supplements map classes `integer`, `float`, `vector`, `aggregate-register`, `indirect`, `capability`, and `sret` to platform registers/stack. Callee/caller-saved state, stack alignment, red zone, unwind records, TLS, and varargs are supplement-specific.

## 5. Vtable compatibility

A consumer requires a vtable major version and minimum byte size. New optional methods append to the tail and have feature bits/default adapters. Reordering or changing existing entries is breaking.

## 6. Async boundary

Cancellation is a request through `cancel_fn`; it does not free the state. The owner continues polling cleanup or invokes `drop_fn` only when the descriptor allows immediate cancellation drop. Wakers are ref-counted ABI objects with wake, wake-by-ref, clone, and drop functions.


---

# OMNI-ABI-*: Target and OS ABI Supplements

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `claim-dependent` |
| Dependencies | `OMNI-ABI`, `OMNI-TARGET`, `OMNI-IR` |
| Date | `2026-08-04` |

## 1. Scope

Calling convention, object format, relocation, TLS, unwind, debug mapping, startup, linker, platform C ABI.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Supplement contract

| Rule | Requirement |
|---|---|
| `ABISUP-0001` | Each supplement pins an external platform ABI version, object format, relocation set, unwind format, debug format, loader behavior, and required OS feature contracts. |
| `ABISUP-0002` | Startup defines process/reset entry, initial stack/register state, TLS setup, capability import, runtime construction, `main` invocation, exit, and abnormal shutdown. |
| `ABISUP-0003` | Orderly shutdown runs process-scope defers and runtime finalizers in reverse dependency order; it does not promise cleanup after `_exit`, abort, reset, power loss, or invalid execution. |
| `ABISUP-0004` | Object/link rules define section names/flags, COMDAT/grouping, weak/common symbols, visibility, interposition policy, LTO container identity, and deterministic resolution. |
| `ABISUP-0005` | Link resolution is independent of archive/member/filesystem enumeration order after canonical symbol ordering. Duplicate strong definitions are errors. |
| `ABISUP-0006` | Unwind tables describe every unwind-enabled frame, including assembly. Crossing a frame without compatible unwind metadata is prohibited. |

## 4. Included supplements

- `OMNI-ABI-X86_64-SYSV-ELF-1`
- `OMNI-ABI-X86_64-WINDOWS-PECOFF-1`
- `OMNI-ABI-AARCH64-SYSV-ELF-1`
- `OMNI-ABI-AARCH64-APPLE-MACHO-1`
- `OMNI-ABI-RV64-SYSV-ELF-1`
- `OMNI-ABI-AARCH64-FREESTANDING-1`
- `OMNI-ABI-RV64-FREESTANDING-1`

Exact machine-readable supplements are in `targets/` and `abi/`.


---

# OMNI-RUNTIME: Runtime Component Contracts

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `profile-required` |
| Dependencies | `OMNI-ERROR`, `OMNI-CONC`, `OMNI-MEM`, `OMNI-ABI` |
| Date | `2026-08-04` |

## 1. Scope

Startup, allocation, panic, unwinding, schedulers, GC domains, metadata, shutdown, observability.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Component ABI

| Rule | Requirement |
|---|---|
| `RT-0001` | Runtime support is a graph of explicitly selected components; unused allocator, unwinder, executor, GC, reflection, I/O, tracing, or accelerator components are not initialized or linked unless required. |
| `RT-0002` | Each component declares ABI version, dependencies, initialization/finalization, thread/task hooks, capabilities, effects, memory ownership, fault policy, and target/profile compatibility. |
| `RT-0003` | Component discovery occurs at link time or through an explicit signed plugin manifest. Ambient dynamic discovery and classpath-style scanning are prohibited. |
| `RT-0004` | Absence of an optional component is reported at translation/link time when statically required or as a typed `Unsupported` result when explicitly dynamically queried. |
| `RT-0005` | Runtime initialization is deterministic dependency-topological order with lexical tie-break by component identity; cycles are errors. |
| `RT-0006` | GC domains are explicit objects with collector policy, heap, roots, safepoints, barriers, weak/finalizer queues, and cross-domain rules. |
| `RT-0007` | Precise stack maps and typed roots are required. Conservative pointer guessing is not conforming for safe managed references. |
| `RT-0008` | A safepoint may occur only at operations marked in MIR. No raw pointer into movable managed storage may remain live across a safepoint. |
| `RT-0009` | Write/read barriers are explicit in IR and collector ABI. Missing a required barrier is a runtime/compiler defect. |
| `RT-0010` | Pinning removes an object from movement or uses a stable indirection according to collector policy; pin duration and fragmentation impact are inspectable. |
| `RT-0011` | Cross-domain managed references are prohibited unless represented by an owning bridge handle whose collector protocol traces both domains without cycles of uncoordinated collection. |
| `RT-0012` | Weak references do not keep objects alive. Upgrade atomically participates in liveness. Finalizers are unordered except dependency-safe constraints, may be delayed indefinitely, and cannot resurrect into ordinary safe ownership unless the profile explicitly permits one-shot resurrection. |
| `RT-0013` | Foreign calls pin/copy/handle managed references and publish roots through the runtime ABI. Foreign-attached threads register stack maps/handles. |

## 4. Core component set

`start`, `panic`, optional `unwind`, allocator providers, thread/TLS, parking/synchronization, async reactor/executor, managed domains, reflection, stack maps, sanitizers/hardening, I/O/persistence/device providers, and observability providers are separate components.

## 5. Finalization warning

Managed finalizers are not deterministic resource management. Files, locks, capabilities, transactions, and device leases require explicit lexical/async cleanup even inside a managed domain.


---

# OMNI-LIB-CORE: Freestanding Core Library

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-TYPES`, `OMNI-NUM`, `OMNI-OWN`, `OMNI-EFFECTS`, `OMNI-EVAL` |
| Date | `2026-08-04` |

## 1. Scope

Exact APIs and semantic contracts for core traits, values, slices, iterators, atomics, intrinsics.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Canonical API inventory

The machine-readable normative inventory is `library/core-api.yaml`. It is the exhaustive Edition 1 freestanding API set. Implementations MAY provide additional namespaced libraries but cannot add members to standard modules in strict mode.

| Rule | Requirement |
|---|---|
| `LIBCORE-0001` | The freestanding core library is available on every conforming Core Language target and performs no allocation, I/O, blocking, thread creation, reflection metadata loading, or hidden capability access unless its signature declares it. |
| `LIBCORE-0002` | Every public API record declares type/effect signature, ownership, panic/faults, allocation, blocking, determinism, thread safety, complexity, target/profile availability, and unsafe obligations. |
| `LIBCORE-0003` | Core modules are `core.bool`, `core.option`, `core.result`, `core.cmp`, `core.hash`, `core.clone`, `core.marker`, `core.mem`, `core.ptr`, `core.slice`, `core.str`, `core.array`, `core.iter`, `core.ops`, `core.num`, `core.atomic`, `core.task`, `core.contract`, and `core.intrinsics`. |
| `LIBCORE-0004` | `Option` and `Result` combinators evaluate callbacks at most once, left-to-right, and forward callback effects polymorphically. |
| `LIBCORE-0005` | Iterator `next` consumes one logical step. `size_hint` is a lower/optional upper bound, not a safety promise. `ExactSizeIterator` and `TrustedLen` carry stronger obligations, with `TrustedLen` unsafe to implement. |
| `LIBCORE-0006` | Slice indexing is bounds-checked; unchecked indexing is unsafe. Mutable iteration yields nonoverlapping elements according to the iterator contract. |
| `LIBCORE-0007` | UTF-8 string APIs distinguish bytes, Unicode scalars, and grapheme/text algorithms. Core supplies bytes/scalars only; grapheme/normalization/case/locale data belongs to text profile libraries. |
| `LIBCORE-0008` | Hashing is trait-based. No core hash algorithm is stable for persistence unless explicitly named. Default hashers in hosted collections use per-process secret seeding and are not deterministic. |
| `LIBCORE-0009` | Atomic APIs expose exact memory ordering and target lock-free queries; invalid compare-exchange failure ordering is a compile-time error when constant and runtime error otherwise. |
| `LIBCORE-0010` | Raw memory functions require validity, nonoverlap where specified, alignment, provenance, initialization, and allocator obligations stated by each function. |
| `LIBCORE-0011` | Intrinsic APIs are compiler-versioned and not stable public source contracts unless promoted into another core module. |

## 4. Complexity notation

Complexity is stated in abstract operations: comparisons, hashes, moves/copies, allocations, atomic operations, and element visits. Amortized bounds name the potential/resource assumptions. A target may have different constant factors but not asymptotically worse behavior for a conforming implementation.

## 5. Panic and allocation visibility

An API documented `no_panic` cannot panic for valid inputs except process-level target failure. An API documented `no_alloc` cannot invoke any allocator even if allocation would later be optimized away.


---

# OMNI-LIB-*: Profile Library Specifications

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `claim-dependent` |
| Dependencies | `OMNI-LIB-CORE`, `OMNI-PROFILES`, `OMNI-RUNTIME` |
| Date | `2026-08-04` |

## 1. Scope

Exact APIs, complexity, effects, allocation, blocking, panic, determinism, security and target availability.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Standard profile library families

- `alloc`: owned collections, allocators, arenas, RC;
- `system`: process, filesystem, networking, time, entropy, dynamic libraries;
- `async`: tasks, executors, reactors, async synchronization and I/O;
- `managed`: managed domains and collection policies;
- `accelerated`: vectors, tensors, kernels, devices;
- `persistent`: schemas, transactions, durable storage and direct I/O;
- `distributed`: RPC, protocols, discovery, resilience;
- `verified`: proofs, ghost state, contracts and verified collections;
- `text`: Unicode text algorithms and locale adapters.

| Rule | Requirement |
|---|---|
| `LIBP-0001` | Profile libraries are versioned separately but use the same type/effect/ownership semantics as core. |
| `LIBP-0002` | Every collection declares iterator/reference invalidation for insertion, removal, reserve, shrink, move, swap, and concurrent mutation. |
| `LIBP-0003` | `Vec` invalidates element references when capacity changes and preserves them for mutations that do not reallocate or remove/move the referenced element; all structural mutation still requires exclusive access. |
| `LIBP-0004` | Hash maps do not promise iteration order unless using an ordered-map type. Rehash invalidates internal iterators/references according to the API; keys/values remain owned by the map. |
| `LIBP-0005` | Default hosted hash maps use a cryptographically strong keyed hash or equivalent DoS-resistant strategy with explicit randomness capability at construction. Deterministic maps use a named stable algorithm and warn against hostile keys. |
| `LIBP-0006` | Text libraries pin Unicode data independently of source-edition Unicode data and expose the data version at runtime/compile time. |
| `LIBP-0007` | Filesystem APIs operate through directory/file capabilities, use platform-native raw path forms plus explicit Unicode views, and never assume paths are valid UTF-8. |
| `LIBP-0008` | Network APIs expose partial I/O, timeouts/cancellation, address-family capability, and protocol state; a write success means accepted by the local provider, not remote delivery. |
| `LIBP-0009` | Async I/O buffers remain owned/pinned by the operation until completion/cancellation commit and cleanup returns them. |
| `LIBP-0010` | Cryptographic APIs are provider/version explicit, avoid generic “encrypt/hash” defaults, expose constant-time claims by profile, and separate key capabilities from bytes. |
| `LIBP-0011` | Persistent and distributed libraries expose partial failure and do not promise transparent rollback of external effects. |

## 4. API completeness

Each profile release contains a canonical API inventory and conformance corpus. A distribution may omit an optional profile but cannot provide a partial standard module while claiming that profile.


---

# OMNI-PROFILES: Profile Definitions and Composition

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `claim-dependent` |
| Dependencies | `OMNI-STD-ROOT`, `OMNI-TARGET`, `OMNI-RUNTIME` |
| Date | `2026-08-04` |

## 1. Scope

Execution/runtime/assurance/numeric profiles, composition algebra, conflicts, fingerprints, required specs.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Composition table

| Profile | Requires | Conflicts/forbids | Meaning |
|---|---|---|---|
| `core` | none | none | Freestanding language and core library. |
| `alloc` | core | none | Allocator interfaces and owned collections. |
| `system` | core | none | Hosted OS providers through capabilities. |
| `async` | core | none | Structured tasks and async state machines. |
| `managed` | alloc | realtime.strict_gc_free | Explicit tracing-GC domains. |
| `accelerated` | core | none | SIMD/tensor/device kernels. |
| `persistent` | alloc | none | Canonical schemas and durable transactions. |
| `distributed` | system, async | none | Networked protocols with partial failure. |
| `verified` | core | none | Proof terms and verified contracts. |
| `deterministic` | core | ambient_nondeterminism | Recorded or excluded nondeterminism. |
| `hardened` | core | none | Defense-in-depth code generation/runtime. |
| `constant_time` | hardened | secret_dependent_observations | Secret-independent control/memory timing contract. |
| `realtime` | core | unbounded_blocking, unbounded_gc | Bounded allocation, blocking, and scheduling. |
| `high_assurance` | verified, hardened, deterministic | unchecked_unsafe, unvalidated_opt | Tight trusted base and validation. |

| Rule | Requirement |
|---|---|
| `PROF-0001` | Profile composition is set union followed by transitive requirement closure and conflict checking. A conflict makes the composition invalid rather than choosing one profile silently. |
| `PROF-0002` | A profile may add APIs, effects, capabilities, restrictions, runtime components, and conformance tests but may not redefine core syntax, type validity, evaluation order, or safe memory semantics. |
| `PROF-0003` | The artifact profile fingerprint includes profile IDs/versions, options, runtime policies, Unicode/text data, panic/OOM policy, numeric default, target features, and ABI supplements. |
| `PROF-0004` | Libraries/artifacts may link only when required profiles are present and every shared semantic/profile option is compatible or adapted through an explicit boundary. |
| `PROF-0005` | Core-required profiles for a Core Language claim are `core` and `safe`. All others are optional claim-dependent suites. |
| `PROF-0006` | `managed`, `accelerated`, `persistent`, `distributed`, and `verified` ratify independently and do not block Core Language 1.0. |
| `PROF-0007` | Restriction profiles compose monotonically: enabling deterministic, realtime, constant-time, hardened, or high-assurance can reject programs but cannot add authority or weaken safety. |

## 4. Compatibility

A function/library signature may be profile-polymorphic over facilities it does not inspect. Profile-specific behavior must be represented in types/effects/capabilities or artifact metadata; it cannot be selected by hidden global mode.


---

# OMNI-MANIFEST: Package and Artifact Manifest Schemas

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-RULES`, `OMNI-TARGET` |
| Date | `2026-08-04` |

## 1. Scope

Package manifest, lockfile, build manifest, artifact metadata, extension rules, canonical serialization.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Canonical schemas

Normative JSON schemas are `schemas/omni-manifest.schema.json` and `schemas/omni-lock.schema.json`. TOML is converted to the schema data model before validation; integer ranges and strings are checked without lossy conversion.

| Rule | Requirement |
|---|---|
| `MANIFEST-0001` | The source manifest is `omni.toml`, restricted to TOML 1.0 syntax with UTF-8, unique keys, no duplicate tables, and no datetime values in normative identity fields. |
| `MANIFEST-0002` | The lockfile is canonical JSON named `omni.lock`; object keys are sorted, numbers are integers, strings are NFC UTF-8, and no insignificant whitespace participates in canonical bytes. |
| `MANIFEST-0003` | A manifest declares package identity, edition, source roots, targets, profiles, features, dependencies, capabilities requested, build actions, exported artifacts, license/provenance, and security policy. |
| `MANIFEST-0004` | Unknown fields in the standard namespace are errors. Namespaced extension tables are retained and included in identity only when marked semantic. |
| `MANIFEST-0005` | Edition/profile/target selection is explicit and cannot be inferred from installed compiler defaults for release builds. |
| `MANIFEST-0006` | A lock record identifies every dependency by source identity, exact version/revision, content digest, feature instance, dependency edges, license/provenance metadata, and yanked/advisory state at resolution time. |
| `MANIFEST-0007` | Manifest and lock schemas are versioned independently. A tool that cannot interpret a semantic field SHALL reject rather than ignore it. |

## 4. Capability requests

A dependency may declare required capability interfaces but receives no authority during build or execution unless the root artifact/host explicitly delegates a scoped capability. Transitive requests are visible in the build and deployment report.


---

# OMNI-PKG: Package Identity and Resolution

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-MANIFEST`, `OMNI-NAMES` |
| Date | `2026-08-04` |

## 1. Scope

Package identity, version constraints, solver semantics, features, source identity, yanks, replacement, vendoring.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Resolution model

Resolution is a deterministic constraint problem over package identities and feature instances. The normative result is the unique maximal solution under the ordering defined by this document; if no solution exists, the resolver emits a deterministic incompatibility proof.

| Rule | Requirement |
|---|---|
| `PKG-0001` | A package source identity is one of registry, signed archive, Git commit, path workspace, or vendored content, each with the exact canonical fields below. |
| `PKG-0002` | Registry identity is `(registry-root-key-id, namespace, package-name, version, content-digest)`. Git identity includes normalized remote identity, full commit hash, subdirectory, and tree digest. Path identity is workspace-relative canonical path plus content digest. |
| `PKG-0003` | Versions use `MAJOR.MINOR.PATCH[-prerelease][+metadata]`. Compatibility operators are exact `=`, compatible `^`, patch-compatible `~`, comparisons, comma intersection, and `||` union. |
| `PKG-0004` | `^1.2.3` means `>=1.2.3,<2.0.0`; `^0.2.3` means `>=0.2.3,<0.3.0`; `^0.0.3` means exactly the 0.0 patch line. Prereleases are selected only by explicit prerelease constraints. |
| `PKG-0005` | Resolution chooses the highest non-yanked version satisfying all constraints for each package instance; ties use lexicographic source identity and then digest. Conflict explanations use deterministic lexicographic decision order. |
| `PKG-0006` | Different major versions and semantically incompatible feature instances may coexist as distinct package identities. The resolver never globally unifies features merely because names match. |
| `PKG-0007` | Features are namespaced and additive within one package instance. A feature that changes public semantics/ABI creates a distinct feature instance included in package identity and cannot be unified with a conflicting instance. |
| `PKG-0008` | Target-conditional dependencies are evaluated from manifest target facts only. Resolution records all target branches required by requested multi-target builds. |
| `PKG-0009` | Content digests cover normalized archive entries, permissions, symlink targets, and bytes; timestamps/owner IDs are excluded or canonicalized. |
| `PKG-0010` | Yanked versions remain usable only when already locked or explicitly allowed by policy. Security-revoked content is rejected even when locked unless an emergency waiver is recorded. |

## 4. Source forms

- registry: signed metadata plus content digest;
- archive: immutable URL/locator, digest, signature policy;
- Git: full commit and verified tree digest, never a floating branch in a release lock;
- path/workspace: local development only unless vendored into release inputs;
- vendored: content tree embedded under the root release source with original provenance.


---

# OMNI-BUILD: Hermetic Build Action Semantics

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-MANIFEST`, `OMNI-PKG`, `OMNI-CONST` |
| Date | `2026-08-04` |

## 1. Scope

Build graph, declared inputs/outputs, environment, sandbox, caching, remote execution, generated code.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Action record

```text
Action = {
  action_schema, tool_digest, argv, virtual_workdir,
  input_tree_digest, environment_map, target_manifest_digest,
  profile_fingerprint, capabilities, limits, expected_outputs
}
```

| Rule | Requirement |
|---|---|
| `BUILD-0001` | A build action is a pure declared function from content-addressed inputs and fixed parameters to content-addressed outputs plus structured diagnostics. |
| `BUILD-0002` | Inputs include source trees, dependency artifacts, tools, target/profile manifests, environment variables explicitly named, build capabilities, and resource limits. |
| `BUILD-0003` | Undeclared files, environment, clock, timezone, locale, randomness, network, process state, home directory, registry state, and filesystem enumeration cannot be observed. |
| `BUILD-0004` | Build network access is denied by default. A fetch action is separate, content-addressed, policy-authorized, and produces immutable inputs for later offline build actions. |
| `BUILD-0005` | Paths inside the sandbox are canonical virtual paths. Host paths do not enter object/debug/generated bytes except through normalized source remapping records. |
| `BUILD-0006` | Generated source identity includes generator artifact digest, command/schema version, exact inputs, semantic extension settings, and output path identity. |
| `BUILD-0007` | Incremental cache keys include every semantically relevant query input and compiler/spec data version. Missing an input is a correctness defect. |
| `BUILD-0008` | Remote cache/execution results are accepted only with authenticated action digest, worker/toolchain attestation, output digests, and policy-compatible isolation. |
| `BUILD-0009` | Build action outputs are written atomically and verified before publication. Partial output is never treated as cache hit. |
| `BUILD-0010` | Parallel action completion order cannot affect merged outputs; merge order is canonical by declared key. |
| `BUILD-0011` | Resource-limit failure is a structured build failure and not permission to emit a partial release artifact. |

## 4. Build scripts

Arbitrary host shell scripts are not a package primitive. Build extensions are Omni/Wasm-like sandboxed tools with declared schemas. Native tools may run only in a stronger isolated provider and are fingerprinted as trusted build inputs.


---

# OMNI-REGISTRY: Registry and Publication Protocol

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-PKG`, `OMNI-SECURITY` |
| Date | `2026-08-04` |

## 1. Scope

Namespaces, publisher identity, index protocol, immutability, mirrors, rate/size limits, moderation.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Publication protocol

| Rule | Requirement |
|---|---|
| `REG-0001` | Publisher identity is a cryptographic principal bound to one or more namespaces by signed registry metadata and transparency records. |
| `REG-0002` | Package publication requires namespace authority, content digest, manifest/lock/provenance/SBOM, signatures meeting policy threshold, and immutable version identity. |
| `REG-0003` | Namespace transfer uses old-and-new principal signatures, a public waiting period, and explicit package-by-package or namespace-wide scope. Emergency transfer requires registry root threshold and audit. |
| `REG-0004` | Typosquatting defenses include normalized/confusable skeleton checks, reserved names, similarity review, publisher history, and warnings; similarity never automatically transfers ownership. |
| `REG-0005` | Abandoned packages may be archived, adopted under a new namespace, or transferred through the published process. Existing immutable versions are never silently replaced. |
| `REG-0006` | Republication of the same version with different content is prohibited. A corrected release uses a new version. |
| `REG-0007` | Registry APIs are deterministic paginated/signed metadata protocols; clients do not trust transport security alone. |

## 4. Transparency

Every accepted publication, yank, transfer, revocation, and root-metadata change is entered into an append-only verifiable log or offline equivalent whose checkpoint is distributed independently.


---

# OMNI-UPDATE: Secure Update and Revocation

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-REGISTRY`, `OMNI-SECURITY` |
| Date | `2026-08-04` |

## 1. Scope

Rollback/freeze/mix-and-match resistance, metadata expiry, key rotation, revocation, compromise recovery.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Threat model

The update system is designed against rollback, freeze, mix-and-match, wrong-package, fast-forward, endless-data, key compromise below threshold, and repository compromise without sufficient delegated keys.

| Rule | Requirement |
|---|---|
| `UPDATE-0001` | Update metadata uses separately scoped root, timestamp, snapshot, targets, delegation, revocation, and advisory roles with threshold signatures and expiration. |
| `UPDATE-0002` | Root metadata is versioned and updated only through a chain validated from a trusted root with rollback protection. |
| `UPDATE-0003` | Timestamp metadata prevents freeze and detects stale snapshots. Snapshot metadata binds exact versions/hashes of delegated metadata and prevents mix-and-match. |
| `UPDATE-0004` | Target metadata binds package/artifact identity, version, length, digest, and custom compatibility fields, preventing wrong-package substitution. |
| `UPDATE-0005` | Clients enforce monotonically nondecreasing metadata versions, expiration using an authorized clock or offline freshness policy, and maximum metadata/target sizes. |
| `UPDATE-0006` | Endless-data defenses require declared length before download, streaming limits, decompression limits, and hash verification. |
| `UPDATE-0007` | Revocation identifies exact content digests/keys/versions and effective time/severity. A security-revoked artifact is not selected or executed absent an audited emergency waiver. |
| `UPDATE-0008` | Offline mirrors carry the complete signed metadata chain and checkpoints. Mirror transport cannot override signatures or freshness. |


---

# OMNI-REPRO: Reproducible Build and Release Rules

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-BUILD`, `OMNI-TARGET` |
| Date | `2026-08-04` |

## 1. Scope

Build perimeter, timestamps, paths, locales, randomness, parallel order, signatures, independent rebuilds.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Reproducibility classes

- `source-reproducible`: same normalized source/package graph;
- `semantic-reproducible`: same IR and public behavior fingerprints;
- `bit-reproducible`: identical unsigned output bytes;
- `diverse-reproducible`: independent toolchain path yields equivalent and authenticated output.

| Rule | Requirement |
|---|---|
| `REPRO-0001` | The reproducibility perimeter includes source/package trees, spec/data manifests, compiler and tools, target descriptions, libraries/runtime, build actions, external generators, linker/assembler, and signing boundary. |
| `REPRO-0002` | Fixed inputs may include target/profile, build mode, declared source date epoch, pinned seed/model, and authorized environment values; every fixed input appears in provenance. |
| `REPRO-0003` | Timestamps are zero or `SOURCE_DATE_EPOCH`-derived; timezones/locales are fixed; directory/map/set order is canonical; random decisions use recorded deterministic seeds. |
| `REPRO-0004` | Host paths are remapped to virtual source identities. User names, machine names, temporary paths, inode numbers, process IDs, and concurrency timing are excluded. |
| `REPRO-0005` | Parallel compilation merges symbols, diagnostics, archives, and metadata in canonical order independent of task completion. |
| `REPRO-0006` | Object/archive formats use deterministic headers and member order. Linkers use stable layout algorithms or record every layout seed/input. |
| `REPRO-0007` | Reproducible unsigned artifact bytes are computed before platform signing. Signatures are detached or inserted into excluded/normalized containers whose relationship to the unsigned digest is specified. |
| `REPRO-0008` | Two independent builders using the same release inputs SHALL produce byte-identical unsigned artifacts for a bit-reproducible claim. |
| `REPRO-0009` | When platform signing inherently changes bytes, the release records unsigned reproducible digest, signed digest, signer identity, timestamp authority, and deterministic verification mapping. |


---

# OMNI-SECURITY: Security Model and Lifecycle

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-STD-ROOT`, `OMNI-TERMS` |
| Date | `2026-08-04` |

## 1. Scope

Threat model, secure defaults, unsafe/capability requirements, supply chain, vulnerability response, crypto policy.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Security invariants

| Rule | Requirement |
|---|---|
| `SEC-0001` | The language security model combines memory/type/concurrency safety, explicit capabilities, least privilege, supply-chain integrity, hardening, and profile-specific side-channel contracts; no one layer substitutes for another. |
| `SEC-0002` | A threat model is mandatory for the compiler, package/build/update systems, runtime components, standard libraries, profiles, registry, bootstrap, and release infrastructure. |
| `SEC-0003` | Safe-code soundness vulnerabilities, capability escalation, compiler miscompilation, malicious dependencies/build tools, compromised signing keys, hostile inputs, side channels, and device/FFI faults are in scope. |
| `SEC-0004` | Speculative execution, cache/timing, power, and fault-injection leakage are not prevented by ordinary language safety. Constant-time/hardened profiles define additional observations and target-specific mitigations. |
| `SEC-0005` | Constant-time code forbids secret-dependent control flow, memory addresses, variable-time instructions, allocation, scheduling, and observable errors unless a certified primitive masks them. |
| `SEC-0006` | Cryptographic algorithms are selected by named provider/profile and version. Deprecation, disable dates, key sizes, parameter validation, and migration are published; generic unversioned crypto defaults are prohibited. |
| `SEC-0007` | A constant-time claim requires source/IR taint rules, target code audit/testing, compiler transformation restrictions, and provider attestation for the exact artifact. |
| `SEC-0008` | Official releases produce SLSA-compatible provenance, SPDX and/or CycloneDX SBOMs at pinned schema versions, dependency graph, signatures, and transparency checkpoints. |
| `SEC-0009` | Advisories identify affected package/artifact/spec ranges, severity, exploitability, fix/workaround, revocation/yank status, and machine-readable policy data. |
| `SEC-0010` | Policy enforcement may deny known-vulnerable, yanked, unmaintained, unsigned, unreviewed-unsafe, or license-incompatible dependencies according to project rules. |
| `SEC-0011` | Security reports use coordinated disclosure; temporary embargo access is least-privilege and audited. Emergency errata follow OMNI-RELEASE. |
| `SEC-0012` | Unsafe code, FFI, macros/build tools, compiler plugins/protocols, and runtime providers are separately auditable trust boundaries. |
| `SEC-0013` | Hardened targets enable available CFI, protected return flow, stack protection, ASLR/PIE compatibility, RELRO, memory tagging/capabilities, allocator hardening, and compartmentalization without changing source semantics. |

## 4. Capability security

Ambient authority is absent in capability-safe code. A dependency receives only explicit values delegated by its caller/host. Capability authenticity and revocation are provider responsibilities constrained by OMNI-EFFECTS.

## 5. Security lifecycle

Every candidate release requires threat-model update, fuzzing/property/differential results, dependency review, soundness audit, unsafe inventory, reproducibility/DDC evidence, incident-response drill, and unresolved-vulnerability disposition.


---

# OMNI-DIAG: Required Diagnostics and Result Schema

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-RULES`, `OMNI-SOURCE` |
| Date | `2026-08-04` |

## 1. Scope

Mandatory diagnostic classes/codes, spans, expansion traces, machine schema, severity, suppression, SARIF mapping.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Diagnostic classes

| Rule | Requirement |
|---|---|
| `DIAG-0001` | Required diagnostics are errors for malformed source/tokens/grammar, unresolved or ambiguous names, type/effect/ownership violations, invalid const evaluation, unsupported required features, ABI/profile incompatibility, and violated static contracts. |
| `DIAG-0002` | Quality-of-implementation diagnostics include performance remarks, style lints, suspicious-but-valid unsafe patterns, portability warnings, and additional security guidance; they cannot reject strict-conforming code unless promoted by user policy. |
| `DIAG-0003` | Every required diagnostic has a stable code, severity, primary rule ID, source spans, structured arguments, related locations, expansion trace, target/profile context, and optional machine-applicable fixes. |
| `DIAG-0004` | Diagnostic wording may improve within a tool version, but stable codes, schema fields, rule links, and fix semantics remain compatible. |
| `DIAG-0005` | Machine output uses `schemas/diagnostic.schema.json`; SARIF mapping preserves rule ID, locations, code flow, fixes, severity, and artifact identities. |
| `DIAG-0006` | A compiler exits 0 on successful requested action, 1 on source/conformance errors, 2 on invocation/configuration errors, 3 on internal compiler error, 4 on build/tool failure, 5 on policy/security denial, and 6 on target/runtime launch failure. |
| `DIAG-0007` | Machine-output mode writes only framed schema records to stdout; human progress/noise goes to stderr or is disabled. |
| `DIAG-0008` | Strict-conformance mode disables extensions, treats unknown attributes/profiles as errors, verifies external data versions, and emits a conformance claim record. |
| `DIAG-0009` | Error recovery may create placeholder nodes/types for IDE continuation but no executable/object/release IR is emitted from a compilation containing required errors. |

## 4. Stable code families

`E01xx` source/lex, `E02xx` grammar, `E03xx` names, `E04xx` types/numerics, `E05xx` ownership, `E06xx` effects/capabilities, `E07xx` const/macros, `E08xx` concurrency/memory, `E09xx` unsafe/FFI, `E10xx` target/ABI/profile, `E11xx` package/build/security, and `ICE0001` internal compiler failure.


---

# OMNI-FMT: Canonical Formatting

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-GRAMMAR`, `OMNI-SOURCE` |
| Date | `2026-08-04` |

## 1. Scope

Edition-pinned deterministic formatting, comments, generated code, idempotence, migration policy.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Canonical style

| Rule | Requirement |
|---|---|
| `FMT-0001` | The canonical formatter is edition-pinned, deterministic, idempotent, semantics-preserving, comment-preserving, and independent of terminal width unless a fixed width is part of the formatter edition. |
| `FMT-0002` | Edition 1 canonical width is 100 display columns using Unicode East Asian Width from the pinned source-data manifest; tabs are not emitted. |
| `FMT-0003` | Blocks use four-space indentation, opening braces on the declaration/control line, and one statement per line except compact empty bodies. |
| `FMT-0004` | Semicolons are emitted exactly where grammar requires or where optional presence prevents fragile adjacent-token parsing; line breaks never supply semantics. |
| `FMT-0005` | Imports are grouped by package root and sorted by normalized canonical path; the formatter does not remove or add imports except in organizer mode. |
| `FMT-0006` | Comments remain attached to their concrete syntax anchors. The formatter emits visible escapes/annotations for source-invisible or bidi characters according to OMNI-SOURCE. |
| `FMT-0007` | Raw strings remain raw when possible; the formatter chooses the minimum delimiter hash count that preserves content and does not rewrite string data. |
| `FMT-0008` | Formatting an already canonical file yields byte-identical output after LF/source normalization. |
| `FMT-0009` | A formatter edition never changes silently under the same language edition; migrations are explicit and produce reviewable diffs. |

## 4. Machine contract

`omni fmt --check --edition 1` exits 0 only when every file equals canonical bytes. `--emit-diff` emits deterministic unified diffs with normalized paths and LF.


---

# OMNI-DOC: Documentation Syntax and Semantics

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `distribution-required` |
| Dependencies | `OMNI-GRAMMAR`, `OMNI-NAMES`, `OMNI-DIAG` |
| Date | `2026-08-04` |

## 1. Scope

Doc comments, links, examples/doctests, API metadata, version/profile/target resolution.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Documentation grammar and attachment

| Rule | Requirement |
|---|---|
| `DOC-0001` | Outer documentation comments attach to the immediately following item after attributes; inner documentation comments attach to the containing module/item. |
| `DOC-0002` | Doc content is CommonMark-compatible Markdown with Omni extensions for symbol links, effects, capabilities, contracts, target/profile tables, and executable examples. |
| `DOC-0003` | An intra-doc link resolves using the same package/module/name rules as source, optionally with an explicit namespace prefix. Ambiguous or broken links are release errors for public APIs. |
| `DOC-0004` | Code examples declare edition, target/profile, capabilities, expected output/fault, and whether they compile, run, fail, or are illustrative. |
| `DOC-0005` | Doctests execute hermetically with only declared capabilities, deterministic inputs, fixed target emulator/provider, resource limits, and exact package graph. |
| `DOC-0006` | API documentation includes full signature, generics, ownership, effects, capabilities, errors, panics, allocation/blocking, cancellation safety, thread safety, complexity, determinism, target/profile availability, safety obligations, and compatibility history. |
| `DOC-0007` | Private items are omitted by default. Cross-package private documentation requires an explicit authenticated documentation capability and is not published accidentally. |
| `DOC-0008` | Generated documentation records compiler/spec/data versions and source hashes and is reproducible. |

## 4. Example fences

```text
```omni,edition=1,profile=core,compile-pass
...
```
```

Other modes are `compile-fail(code=...)`, `run-pass(output=...)`, `run-panic(category=...)`, `no-run`, and `ignore(reason=...)`.


---

# OMNI-DEBUG: Debug, Unwind, and Source Mapping

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `platform-required` |
| Dependencies | `OMNI-IR`, `OMNI-ABI-*`, `OMNI-DIAG` |
| Date | `2026-08-04` |

## 1. Scope

Optimized debug semantics, source maps, async/actor/region views, crash records, external debug-format mapping.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Mapping contract

| Rule | Requirement |
|---|---|
| `DEBUG-0001` | Debug/source mapping preserves normalized source identity, original-byte mapping, macro expansion stack, inline call stack, variable location ranges, ownership moves, optimized-away status, and async state identity. |
| `DEBUG-0002` | DWARF-based Edition 1 targets pin a stable target-supported DWARF revision in the target supplement; draft DWARF revisions are experimental and cannot be silently required. |
| `DEBUG-0003` | Windows targets use pinned CodeView/PDB specifications and PE/COFF unwind metadata; Apple targets use pinned Mach-O/DWARF/compact-unwind contracts. |
| `DEBUG-0004` | Every unwind-enabled frame has correct unwind information through prologue/epilogue and inline assembly, or the function is declared non-unwindable and boundaries prevent unwind crossing. |
| `DEBUG-0005` | Optimized debug info may report a variable as unavailable but cannot report a stale/moved value as live. Moves and destruction end location validity. |
| `DEBUG-0006` | Async stacks are reconstructed from future/task descriptors and suspension records, with stable logical frame IDs distinct from physical stacks. |
| `DEBUG-0007` | Crash records contain artifact/build IDs, target/profile fingerprint, fault/panic category, stable symbol/source IDs, and integrity metadata; raw addresses are optional and security-policy controlled. |
| `DEBUG-0008` | Debuggers cannot bypass capability/security policy merely because metadata exists; reading protected memory or process state requires debugger authority. |

## 4. Source maps

A source map entry maps machine address/IR operation ranges to normalized source span plus expansion and inline stacks. Generated source includes generator provenance. Mapping is many-to-many; tools SHALL distinguish exact, approximate, optimized-away, and unavailable locations.


---

# OMNI-WIRE: Canonical Wire and Persistence Schemas

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `profile-required` |
| Dependencies | `OMNI-TYPES`, `OMNI-NUM`, `OMNI-SECURITY` |
| Date | `2026-08-04` |

## 1. Scope

Canonical encodings, schema identity/evolution, limits, NaNs, unknown/duplicate fields, golden vectors.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Byte-level encoding

A message is:

```text
4 bytes  "OMW1"
16 bytes schema fingerprint
varuint  payload length
bytes    canonical payload
4 bytes  CRC32C of header+payload (integrity only, not authentication)
```

Authentication/encryption are profile envelopes and do not alter the canonical inner payload.

| Rule | Requirement |
|---|---|
| `WIRE-0001` | Wire/persistent schemas are independent of native layout and use stable schema, type, field, and variant identifiers. |
| `WIRE-0002` | Edition 1 canonical encoding begins with magic/version and encodes records as ascending numeric field ID, each with explicit wire type and length where applicable. |
| `WIRE-0003` | Unsigned integers use minimal unsigned LEB128; signed integers use minimal zigzag-LEB128 unless a fixed-width wire type is declared. Nonminimal encodings are rejected in canonical mode. |
| `WIRE-0004` | Fixed integers/floats are little-endian. Floating values use IEEE storage bits; canonical mode maps every NaN to one quiet canonical NaN per width and preserves signed zero. |
| `WIRE-0005` | Bytes and UTF-8 strings are length-prefixed. Strings are validated UTF-8. Collections have element count and each variable element has bounded length/structure. |
| `WIRE-0006` | Booleans encode as one byte 0 or 1; other values are rejected. |
| `WIRE-0007` | Record field IDs are unique. Duplicate singular fields are errors; repeated fields use an explicit repeated type. Unknown fields are skipped/preserved according to schema policy. |
| `WIRE-0008` | Variant IDs are stable integers. Unknown variants return a typed unknown-variant result or preserved opaque variant only when the schema declares extensibility. |
| `WIRE-0009` | Canonical ordering is ascending field ID, map entries sorted by canonical key bytes, and sets sorted by canonical element bytes. Duplicate canonical map/set keys are errors. |
| `WIRE-0010` | Compatibility classes are backward-readable, forward-readable, bidirectional, migration-required, and breaking. The schema checker computes class from declared changes. |
| `WIRE-0011` | Adding optional fields with defaults is backward-compatible; removing required fields, reusing IDs, changing wire type/meaning, narrowing ranges, or changing canonical defaults is breaking unless versioned migration is required. |
| `WIRE-0012` | Zero-copy views require alignment, endianness, lifetime ownership of the input buffer, validated bounds, and representation compatibility. Otherwise decoding copies/converts. |
| `WIRE-0013` | Decoders enforce schema-declared maximum nesting, total bytes, collection counts, string lengths, allocation budget, and processing fuel before allocating or recursing. |
| `WIRE-0014` | Hostile-input validation completes before constructing a safe value with invariants stronger than the wire representation. |
| `WIRE-0015` | Crash-safe persistent records use checksummed length-delimited frames and commit markers/journal protocol named by the persistence profile; partial frames are ignored/recovered, never interpreted as complete values. |

## 4. Field wire types

`0 varuint`, `1 varsint`, `2 fixed32`, `3 fixed64`, `4 fixed128`, `5 bytes`, `6 record`, `7 packed-sequence`, `8 capability-reference` (provider-authenticated profile only). Field key is `(field_id << 4) | wire_type` encoded as varuint.

## 5. Golden vectors

`conformance/wire-golden-vectors.json` contains canonical encodings for every primitive, boundary, NaN, unknown-field, duplicate, malformed-length, and compatibility case.


---

# OMNI-STAGE0: Stage-0 Language and Compiler Contract

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `bootstrap-required` |
| Dependencies | `OMNI-SOURCE`, `OMNI-GRAMMAR`, `OMNI-EVAL`, `OMNI-LIB-CORE` |
| Date | `2026-08-04` |

## 1. Scope

Exact bootstrap subset, forbidden features, seed interface, target, deterministic output, conformance corpus.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Exact subset

| Rule | Requirement |
|---|---|
| `STAGE0-0001` | Stage-0 is a strict syntactic and semantic subset of Edition 1; every Stage-0 source parses and has identical observations under Edition 1. |
| `STAGE0-0002` | Stage-0 includes UTF-8/ASCII identifiers, modules, structs/enums, fixed integers/bool/byte, arrays/slices, functions, generics without specialization, Result/Option, ownership/borrows, explicit effects, unsafe raw memory, and freestanding/native code generation. |
| `STAGE0-0003` | Stage-0 excludes macros, runtime reflection, async, managed domains, dynamic typing, relations, decimal, scalable vectors/tensors, profile libraries, and user-defined effect handlers. |
| `STAGE0-0004` | Stage-0 grammar is generated by disabling named Edition 1 productions, not maintained as an unrelated grammar. |
| `STAGE0-0005` | Stage-0 uses the same checked arithmetic, evaluation order, initialization/drop, panic-abort policy, provenance, and C/omni_v1 ABI subset. |
| `STAGE0-0006` | The seed compiler consumes canonical Stage-0 source and emits one reference object format/ISA plus deterministic diagnostic records. |
| `STAGE0-0007` | Every Stage-0 restriction is represented by a feature predicate and tested against the Edition 1 parser/model to prove subset inclusion. |

## 4. Reference target

The canonical seed target is `riscv64-unknown-none-elf-omni_v1[lp64]+rv64imac`, with an optional hosted x86-64 seed path for accessibility. A release may add seed targets but retains one immutable canonical path.

## 5. Seed simplifications

Stage-0 uses abort panic/OOM, no unwinder, one bump/system allocator interface, no dynamic linking, no trait objects across ABI, and a simple verified MIR-to-machine lowering. These are subset restrictions, not alternate semantics.


---

# OMNI-BOOT: Bootstrap, DDC, and Trusted Base

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `bootstrap-required` |
| Dependencies | `OMNI-STAGE0`, `OMNI-BUILD`, `OMNI-REPRO`, `OMNI-SECURITY` |
| Date | `2026-08-04` |

## 1. Scope

Seed artifacts, trust graph, diverse double compilation, reproducibility, audit procedure, TCB manifest.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Bootstrap chain

| Rule | Requirement |
|---|---|
| `BOOT-0001` | The bootstrap seed is an immutable content-addressed source/binary package with documented encoding, build platform, required external tools, input hashes, and step-by-step offline procedure. |
| `BOOT-0002` | The trusted bootstrap path consists of seed decoder/compiler, Stage-0 sources, assembler/object writer, linker/image builder, target emulator/hardware, specification data, and verification tools named in the TCB inventory. |
| `BOOT-0003` | Stage A builds compiler B from seed. B builds compiler C from canonical Omni sources. C rebuilds compiler D; C and D must converge byte-for-byte for deterministic stages or to declared semantic equivalence when signing/layout is external. |
| `BOOT-0004` | Diverse double compilation builds the same canonical compiler source with the trusted chain and an independently implemented diverse compiler/toolchain, then uses each result to rebuild and compares canonical outputs. |
| `BOOT-0005` | A diverse compiler qualifies only if its implementation lineage, language/tool dependencies, backend/link path, and build environment are sufficiently independent and documented. |
| `BOOT-0006` | DDC mismatch blocks release, preserves all artifacts/logs, and triggers differential localization; it is never waived merely because one binary passes tests. |
| `BOOT-0007` | The TCB inventory lists exact source/binary digests, role, reason trusted, audit status, replacement strategy, and transitive dependencies for proof checker, parser generator, solver, assembler, linker, Unicode data, crypto, OS/firmware, and signing tools. |
| `BOOT-0008` | Emergency bootstrap recovery uses archived source, specs, data, emulators, and at least two independent seed paths stored in reproducible offline media. |
| `BOOT-0009` | Official compiler artifacts include provenance, SBOM, conformance report, optimization-validation report, and DDC evidence bound to the artifact digest. |

## 4. Seed encoding

The canonical seed source is LF-normalized UTF-8 Stage-0. Seed binary packages use deterministic `tar`-like archives with sorted paths, fixed permissions/timestamps, SHA-256 manifest, and threshold signatures. No opaque installer is part of the canonical path.

## 5. Auditability

The canonical seed compiler prioritizes small code size, straightforward algorithms, and exhaustive tests over optimization performance. The production optimizer/backend are built later and are not required to trust their own output without DDC and validation.


---

# OMNI-CONFORM: Conformance and Certification

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-RULES`, `OMNI-LIB-CORE`, `OMNI-ABI-*` |
| Date | `2026-08-04` |

## 1. Scope

Test formats, oracle rules, coverage, waivers, extensions, certification, independent results, claim validation.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Case schema

Normative schema: `schemas/conformance-case.schema.json`. Result schema: `schemas/conformance-result.schema.json`. Manifest: `conformance/manifest.yaml`.

| Rule | Requirement |
|---|---|
| `CONF-0001` | Each conformance case has stable case ID, rule IDs, suite version, source/artifact inputs, expected classification/result, target/profile applicability, required limits, oracle/model version, and allowed implementation-defined alternatives. |
| `CONF-0002` | Test kinds are parse-pass/fail, static-pass/fail, run-value, run-output, run-fault, memory-litmus, ABI-cross, wire-golden, build-resolution, security-policy, differential, metamorphic, and performance-contract. |
| `CONF-0003` | Coverage includes positive, negative, boundary, interaction, differential, and metamorphic cases for every ratified core rule; high-risk rules additionally require an independent model or implementation. |
| `CONF-0004` | A test is corrected only through a versioned test erratum linked to the rule. Prior suite artifacts remain immutable; corrected conformance claims identify the overlay. |
| `CONF-0005` | Tests do not create semantics. When a test conflicts with normative rules, the test is invalid until corrected. |
| `CONF-0006` | Strict mode disables extension syntax/semantics and rejects artifacts whose unrecognized extension could affect behavior. Namespaced extensions may be tested only in separate claims. |
| `CONF-0007` | An implementation claim names edition, modules, profiles, targets, limits, external ABI/data versions, implementation-defined choices, deviations, and test-result digest. |
| `CONF-0008` | Parser-only, core-subset, profile, platform, distribution, and complete-release claims are distinct; partial implementations cannot claim complete conformity. |
| `CONF-0009` | Certification may be self-attested, independently audited, or authority-certified. The claim record states assurance level, auditor, evidence, waivers, expiration, and revocation status. |
| `CONF-0010` | Waivers cannot excuse a soundness violation or false accepted behavior; they may document unavailable optional hardware/profile tests and narrow the claim. |
| `CONF-0011` | A certification is revoked when evidence is falsified, a blocker soundness/security defect invalidates the claim, or required artifacts become untrusted; revocation is signed and published. |

## 4. Required implementation evidence

Core 1.0 release requires two independently implemented parsers, one executable semantic model, one native implementation, grammar fuzzing, source/model/native differential tests, memory litmus execution, cross-compiler ABI tests, package/build reproducibility tests, and rule coverage with no unresolved P0 gaps.

## 5. Limits

Implementations may declare finite nesting, type size, monomorphization, const fuel, macro token/depth, object size, and diagnostic limits only above release minima. Exceeding a declared limit yields `ImplementationLimit` and cannot miscompile.


---

# OMNI-BENCH: Benchmark and Performance Evidence

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `release-required` |
| Dependencies | `OMNI-TARGET`, `OMNI-PROFILES`, `OMNI-REPRO` |
| Date | `2026-08-04` |

## 1. Scope

Benchmark methodology, equivalence rules, datasets, statistics, energy/latency/size/compile-time reporting.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Evidence rules

| Rule | Requirement |
|---|---|
| `BENCH-0001` | Performance claims compare equivalent algorithms, data structures, numeric/safety policies, error handling, I/O durability, target features, and warm/cold state. |
| `BENCH-0002` | Benchmarks report source, compiler/artifact digests, flags/profiles, hardware/firmware/OS, topology/frequency/power settings, dataset, repetitions, warmup, and raw samples. |
| `BENCH-0003` | Metrics include wall/CPU time, throughput, median and tail latency, peak/live memory, allocations, code size, startup, energy where measurable, storage/network I/O, compile time, and cache artifact size. |
| `BENCH-0004` | Statistics report sample count, central tendency, dispersion/confidence interval, outliers with policy, and practical effect size. Cherry-picked best runs are prohibited. |
| `BENCH-0005` | Adaptive benchmarking chooses run count from a predeclared stopping rule and records it. Benchmark code cannot detect competitors or alter work by implementation identity. |
| `BENCH-0006` | Profile-guided/autotuned builds include profile-collection cost and identify whether the benchmark workload contaminated training data. |
| `BENCH-0007` | Correctness/conformance is checked before timing. A faster result with changed output, precision, fault, durability, or safety contract is not equivalent. |
| `BENCH-0008` | Release gates use representative suites and regression budgets, not a single geometric mean. Regressions may be accepted only with published tradeoff rationale and no hidden metric loss. |

## 4. Anti-gaming review

Benchmark harnesses inspect generated code for eliminated work, validate results, randomize/partition inputs deterministically, separate setup from measured regions, and publish scripts/raw data. Energy claims include measurement uncertainty and idle-baseline method.


---

# OMNI-RELEASE: Edition Release Manifest and Errata

| Field | Value |
|---|---|
| Suite version | `1.0.0-candidate.1` |
| Language edition | `1` |
| Status | Complete normative candidate; not yet ratified or implementation-certified |
| Classification | `core-required` |
| Dependencies | `OMNI-CONFORM`, `OMNI-REPRO`, `OMNI-BOOT` |
| Date | `2026-08-04` |

## 1. Scope

Pinned documents/data/digests, normative references, errata, known deviations, compatibility and migration report.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described by BCP 14 when, and only when, they appear in all capitals.

Every requirement in this document has a stable rule identifier. Informative notes and examples do not create requirements. An implementation claiming conformance to this module SHALL satisfy every applicable REQUIRED rule and SHALL report each implementation-defined choice named by this module.

## 3. Release manifest

| Rule | Requirement |
|---|---|
| `REL-0001` | A release manifest pins every normative module, grammar/data/model/schema/test artifact, target supplement, external normative reference, and cryptographic digest. |
| `REL-0002` | Version axes are: language edition, specification revision, corrigendum/erratum overlay, Unicode/source-data version, text-library data version, ABI version, profile version, wire-schema version, toolchain version, package version, and target-manifest version. |
| `REL-0003` | An edition changes source language compatibility. A revision clarifies/adds nonbreaking normative detail. A corrigendum fixes publication error without intended semantics change. A normative erratum corrects semantics and declares compatibility impact. |
| `REL-0004` | Normative references use exact version/date/revision and archived digest/location. Floating “latest” references are informative only. |
| `REL-0005` | Replacing a normative reference requires compatibility analysis, updated conformance evidence, and a new suite revision or edition as appropriate. |
| `REL-0006` | Errata severities are editorial, diagnostic, compatibility, soundness, and security. Soundness/security errata may invalidate prior claims and trigger revocation. |
| `REL-0007` | An erratum overlay is signed, immutable, rule-linked, and states effective releases, replacement text/data/tests, implementation impact, and migration. |
| `REL-0008` | A corrected-conformance claim names the base manifest plus every applied overlay; silent mutable web text is not a standard release. |
| `REL-0009` | Compatibility reports separately analyze lexical/grammar source, static acceptance, dynamic observations, ABI, wire/persistence, package resolution, build reproducibility, diagnostics/tooling, and profile behavior. |
| `REL-0010` | Release qualification requires no unresolved P0 gaps, no normative placeholders, complete rule-test/model mapping, independent review, security audit, reproducible artifacts, DDC, and signed archival publication. |
| `REL-0011` | Normative external artifacts are archived where license permits or referenced with verified immutable digests and retrieval instructions. |

## 4. Candidate status

This suite is `1.0.0-candidate.1`: the language definition has no intentionally unresolved semantic choices, but ratification, independent implementations, formal proof completion, and certification evidence remain future release gates. Those gates concern confidence and adoption, not missing definitions.

## 5. Publication set

The candidate ZIP, file manifest, SHA-256 list, rule registry, resolved gap register, normative references, and validation report form one immutable candidate publication.

---


# Part I-A — Omni Engineering, Compiler Construction, and Semantic Closure Addendum

## 1. Purpose and status

This section incorporates the latest architectural conclusions, compiler-engineering requirements, and semantic-closure audit criteria into the consolidated specification.

This section is **engineering guidance and implementation law derived from the current Edition 1 normative suite**. It does not silently override a normative module. Where this addendum identifies a requirement that must become normative, it SHALL be promoted into the appropriate `OMNI-*` module, rule registry, model, schema, or conformance corpus before ratification.

The central question for Edition 1 implementation is no longer whether Omni has a coherent language philosophy. The question is:

> **Can an independent implementation team translate every conforming Omni program without inventing a semantic decision that is not already determined by the standard suite?**

This property is called **semantic closure** throughout this addendum.

---

## 2. What Omni is

Omni is a statically typed, memory-safe-by-default, capability-secure, effect-aware, multi-paradigm programming language whose normal execution route is native ahead-of-time compilation.

Its default memory discipline is affine ownership with borrowing and regions. Reference-counted and managed domains are explicit facilities rather than hidden global execution semantics. Raw memory, foreign interfaces, device access, and inline assembly are restricted by unsafe obligations.

Omni is deliberately designed as a **language plus executable standards system**, not merely a parser, compiler frontend, or syntax specification. The complete product therefore includes:

1. a source language and lexical/syntactic definition;
2. static semantics for names, types, numbers, ownership, effects, and capabilities;
3. an abstract machine and dynamic semantics;
4. concurrency and memory-model rules;
5. unsafe and FFI boundaries;
6. canonical IR and optimization legality;
7. ABI, wire, persistence, package, and build semantics;
8. diagnostics and source mapping;
9. reference models and conformance tests;
10. deterministic release, reproducibility, bootstrap, DDC, and certification infrastructure.

The suite is structured so that the same semantic facts can be consumed by a human programmer, a compiler, a verifier, a test harness, a debugger, a build system, and an AI programming system.

---

## 3. Why Omni is designed this way

### 3.1 Safety is not sufficient by itself

Memory safety alone does not make a language straightforward to compile, verify, debug, or generate. A language can be memory-safe while leaving evaluation order, cancellation, resource failure, capability authority, ABI boundaries, or optimization legality underspecified.

Omni therefore treats safety as one layer of a larger requirement:

> **Every observable semantic decision must be explicit, classifiable, testable, and traceable to a normative rule.**

### 3.2 Determinism is a semantic tool, not merely a reproducible-build feature

Determinism reduces the number of possible interpretations that a compiler, debugger, optimizer, test oracle, or AI needs to consider.

Omni therefore fixes or explicitly classifies:

- evaluation order;
- initialization and destruction order;
- arithmetic failure;
- cancellation;
- capability transitions;
- package resolution;
- build inputs;
- artifact identity;
- semantic source identity;
- target-specific choices;
- observable event ordering.

A permitted implementation choice remains legal only when the standard explicitly places it in a defined classification.

### 3.3 AI co-reasoning is an architectural consequence

Omni is not defined as an "AI programming language" in the sense of requiring an AI-specific syntax.

Instead, its semantics are designed so that machine assistance can operate on the same artifacts used for ordinary compilation:

`source -> structured diagnostics -> typed/static model -> canonical IR -> validation -> executable artifact`

This supports a programming workflow in which an AI does not have to guess hidden runtime behavior. It can construct a candidate implementation, query the compiler/model, inspect structured failures, revise the program, and repeat until the implementation satisfies the declared semantic obligations.

The objective is therefore not to minimize all tokens emitted by an AI. The objective is to minimize **unverifiable assumptions**.

### 3.4 Strictness belongs primarily in the verifier

The existence of lifetimes, effects, capability requirements, refinements, and ownership constraints does not require every source program to spell out every inferred fact.

Omni SHALL prefer:

- bidirectional typing where inference is sound;
- local effect inference;
- implied bounds where uniquely determined;
- lifetime inference from actual use;
- structured capability attenuation;
- compiler-generated ownership facts exposed in diagnostics and MIR;
- concise syntax whose elaborated semantics remain explicit.

The design target is:

> **explicit semantic meaning, concise source expression.**

---

## 4. Core engineering principle: semantic closure

### 4.1 Definition

A suite is semantically closed for an observable construct when all of the following are determined:

1. how source bytes become tokens;
2. how tokens become a canonical syntax tree;
3. how names are resolved;
4. how types and refinements are inferred or checked;
5. how ownership and initialization state evolves;
6. which effects and capabilities are required;
7. how the construct lowers into the canonical semantic IR;
8. how the abstract machine evaluates it;
9. which faults or resource failures can occur;
10. which memory/concurrency events it produces;
11. which observations are permitted;
12. how unsafe/FFI assumptions are represented;
13. which optimizations preserve the same observations;
14. how target/ABI details are incorporated;
15. which conformance cases witness the rule.

If any of these questions has more than one answer that is not explicitly classified by the suite, the construct is not semantically closed.

### 4.2 Closure is stronger than “no undefined behavior”

“No undefined behavior” is a safety outcome. Semantic closure is an implementation-independence criterion.

A conforming implementation may still encounter:

- resource exhaustion;
- target limits;
- an external device failure;
- a declared implementation-defined representation;
- an explicitly enumerated unspecified choice;
- nontermination;
- invalid unsafe execution.

Those outcomes remain valid only because the standard defines their classification and the permitted consequences.

### 4.3 Closure evidence

Each high-risk semantic rule SHOULD have at least one of:

- an executable reference-model transition;
- a mechanically checked static judgment;
- a memory/concurrency litmus;
- a differential test against an independent implementation;
- a metamorphic property;
- a translation-validation obligation;
- a golden encoding vector.

For P0 soundness rules, the release process SHALL follow the existing `OMNI-RULES` and `OMNI-CONFORM` obligations and SHALL NOT rely exclusively on example programs.

---

## 5. Canonical compilation pipeline

The conceptual compiler pipeline is:

```text
Source bytes
  -> source normalization / identity
  -> lexing
  -> CST
  -> parsing / canonical AST
  -> macro/token transformation
  -> target/profile filtering
  -> name resolution
  -> type/refinement solving
  -> ownership / borrow analysis
  -> effect / capability checking
  -> typed HIR
  -> canonical semantic MIR
  -> MIR validation
  -> optimization with legality proofs/checks
  -> target-specific lowering
  -> object generation
  -> link / image construction
  -> runtime-component initialization
  -> executable artifact
```

The compiler SHALL preserve the semantic boundary between **meaning** and **optimization**.

### 5.1 Frontend responsibilities

The frontend is responsible for deriving a unique meaning from source:

- exact token stream;
- canonical tree;
- stable identifiers;
- deterministic name lookup;
- unique type/effect solutions;
- ownership state;
- capability authority;
- explicit refinement obligations.

Frontend recovery structures may exist for editors, but recovery syntax SHALL never leak into release semantics.

### 5.2 HIR responsibilities

HIR is the elaborated, typed representation in which:

- syntactic sugar is made explicit;
- inferred types and bounds are materialized;
- ownership transitions are explicit;
- effects and capability requirements are attached;
- macro-generated provenance remains traceable;
- source-level constructs map to stable semantic identities.

HIR MAY remain implementation-oriented, but its semantics SHALL be derivable entirely from the normative static rules.

### 5.3 MIR responsibilities

MIR is the canonical semantic execution representation.

The MIR contract SHALL make explicit at minimum:

- evaluation order;
- places and projections;
- initialization state;
- moves and copies;
- borrow creation/activation/end;
- cleanup/drop edges;
- effect operations;
- capability operations;
- task creation/join/cancellation;
- synchronization operations;
- unsafe preconditions;
- resource-failure points;
- calls and return-slot behavior;
- persistent/volatile operations;
- control-flow successors.

The MIR verifier is not an optimization checker. It first establishes that MIR is a legal representation of the source semantics.

### 5.4 Optimization responsibilities

Optimizers may:

- reorder computations;
- eliminate dead work;
- fuse or split operations;
- specialize generics;
- vectorize;
- inline;
- transform control flow;
- lower abstractions;

only when the transformation preserves all observations permitted by the active edition/profile/target and preserves all safety, authority, memory, and fault obligations.

An optimizer SHALL NOT obtain additional semantic freedom merely because an internal representation has discarded information that the normative semantics required.

---

## 6. Canonical IR and DDC requirements

### 6.1 Canonicality goal

For semantic lowering, two conforming implementations SHALL either:

1. emit canonically identical semantic IR for the same source and declared environment; or
2. emit IR instances that are proven equivalent by a specified canonicalization/equivalence relation.

The preferred Edition 1 architecture is canonical semantic MIR followed by implementation-specific optimization IR.

### 6.2 What must never be implicit in semantic MIR

The following are forbidden as hidden backend decisions:

- unspecified drop ordering;
- hidden partial-initialization state;
- untracked volatile/device access;
- hidden capability acquisition;
- hidden effect generation;
- hidden cancellation;
- hidden memory synchronization;
- untracked pointer provenance assumptions;
- validity assumptions absent from an unsafe precondition;
- target-dependent interpretation of core operations.

### 6.3 DDC role

DDC is a trust-amplification mechanism, not the language specification.

A DDC mismatch SHALL be treated as evidence that one or more of the following differ:

- source or specification inputs;
- semantic lowering;
- optimization legality;
- backend behavior;
- linker/object generation;
- trusted toolchain components;
- artifact canonicalization.

The release system SHALL preserve the mismatch artifacts and perform differential localization.

DDC SHALL NOT be used as a reason to declare two inconsistent semantic interpretations “both acceptable.”

---

## 7. Static-semantics to dynamic-semantics closure audit

The highest-priority semantic audit begins at:

`OMNI-TYPES + OMNI-NUM + OMNI-OWN + OMNI-EFFECTS`

and crosses into:

`OMNI-EVAL + OMNI-ERROR + OMNI-CONC + OMNI-MEM + OMNI-MACHINE`.

The following seam tests SHALL be part of the Edition 1 closure corpus.

### 7.1 Lexing / grammar / macro hygiene seam

**Threat:** generated identifiers, generic names, imports, attributes, or macro output change lookup unexpectedly.

**Required answer:** macro expansion phases, hygiene identity, introduced-name context, generated-source provenance, attribute phase order, shadowing, and name lookup SHALL yield one deterministic resolution.

**Acceptance criterion:** two conforming implementations given the same token-tree inputs select the same resolved declaration identities or both produce the same required diagnostic class.

### 7.2 Evaluation / error seam

**Threat:** left-to-right evaluation conflicts with a faulting operation and a later observable operation such as volatile/device I/O.

**Required answer:** the standard SHALL identify the exact evaluation point of every operand/subexpression and SHALL state whether the fault prevents subsequent evaluation.

**Acceptance criterion:** no optimizer or instruction scheduler may make the later observation occur when the abstract-machine evaluation never reached it.

### 7.3 Allocation / resource-failure seam

**Threat:** allocation or stack exhaustion occurs while a preceding computation has already performed externally visible work.

**Required answer:** resource failure is an explicit fault transition after a precisely defined point in execution.

**Acceptance criterion:** already-emitted observations remain preserved unless the applicable target/resource policy explicitly states otherwise; failure never grants memory-unsafety permission.

### 7.4 Ownership / error-unwind seam

**Threat:** a constructor, call, defer, panic, or cancellation path drops an incompletely initialized value.

**Required answer:** successful initialization state and cleanup edges are explicit.

**Acceptance criterion:** exactly the successfully initialized portions are dropped, exactly once, in the defined order.

### 7.5 Async / cancellation / memory seam

**Threat:** cancellation races with task execution, future cleanup, lock release, or shared-memory communication.

**Required answer:** cancellation is a semantic event with defined delivery points and explicit task/future cleanup behavior. Any synchronization required to observe cancellation or cleanup SHALL have a specified `hb`, `sw`, or equivalent memory-model relation.

**Acceptance criterion:** an independent implementation can construct the same permitted event graph without guessing whether cancellation happened before or after a specified cleanup boundary.

### 7.6 Unsafe / optimization seam

**Threat:** an optimizer treats an unsafe precondition as an everlasting fact and moves a transformed operation across an event that invalidates that assumption.

**Required answer:** unsafe assumptions have explicit scope, dominating validity conditions, and a lifetime within the semantic IR.

**Acceptance criterion:** translation validation rejects a transformed program whenever the optimized execution can rely on an assumption outside its valid semantic scope.

### 7.7 Capability / effect seam

**Threat:** a function is statically allowed to mention an effect but has insufficient authority to exercise the corresponding resource.

**Required answer:** effects classify possible observation; capabilities provide authority. Neither concept may implicitly substitute for the other.

**Acceptance criterion:** removing or attenuating a capability can only remove authorized operations; ordinary code cannot recreate the lost authority.

---

## 8. Effect-system engineering requirements

The effect system SHALL optimize for composability rather than maximum annotation verbosity.

The Edition 1 strategy is:

- finite canonical effect rows;
- at most one row variable where the rules allow it;
- local inference for private implementation details;
- explicit public upper bounds;
- effect masking only through explicit handlers;
- residual effects included in handler result rows;
- no implicit authority from merely naming an effect.

### 8.1 Avoiding effect-row explosion

The compiler SHOULD infer common local effects and normalize aliases automatically.

An API designer SHOULD annotate stable public boundaries rather than reproduce every inferred transitive detail in every internal signature.

A semantic effect is part of the public contract when callers can observe or depend upon it. Merely internal implementation structure need not become a permanent public API commitment.

### 8.2 Public API stability

A public function SHALL expose a stable effect boundary.

Adding a new externally observable effect to a non-polymorphic public API is a breaking change unless its previous contract already permitted it through the declared effect bound.

This protects incremental compilation and API compatibility while preserving local inference.

---

## 9. Capability composition and attenuation

Capabilities are authority values, not labels.

A useful composition model is:

```text
Provider-issued root capability
        |
        +-- attenuation: rights/scope/quota
        |
        +-- child capability
        |
        +-- borrowed capability
        |
        +-- operation-specific authority
```

### 9.1 Required properties

Capability operations SHALL provide:

- no forging from ordinary values;
- no silent widening;
- explicit ownership/borrowing semantics;
- provider-defined revocation behavior;
- explicit scope, quota, duration, or address constraints where applicable;
- auditability without automatic leakage of secret payloads.

A function requiring multiple authorities MAY receive a capability bundle when the bundle is itself a sealed, non-widenable value. It SHALL NOT require a universal authority object merely for convenience.

### 9.2 Effect/capability separation

An effect answers:

> “What kinds of observations may this computation cause?”

A capability answers:

> “Which specific authority may this code exercise?”

This distinction is fundamental to security, static analysis, replay, testing, and AI reasoning.

---

## 10. Generational references and cyclic data

`Gen<T>` and arena-based structures are intended to provide safe graph-oriented data structures without converting every graph into globally shared ownership.

The design SHALL define separately:

1. allocation identity;
2. generation/version;
3. live/dead state;
4. arena ownership;
5. mutation authorization;
6. reclamation;
7. stale-handle behavior.

A `Gen<T>` dereference SHALL fail in the defined way when the generation does not match a live slot.

Generational references do not automatically solve arbitrary memory reclamation. Cycles remain a reclamation problem unless the owning arena, reference-counted domain, managed domain, or explicit collector provides a complete policy.

Therefore the language SHALL NOT claim that `Gen<T>` is equivalent to a tracing collector.

### 10.1 Required tests

The conformance corpus SHALL include:

- stale generation access;
- slot reuse;
- nested arenas;
- cross-task transfer restrictions;
- region expiry;
- mutation during iteration;
- cycle formation and destruction;
- interaction with pinning;
- interaction with managed references;
- unsafe escape attempts.

---

## 11. Incremental compilation requirements

Incremental compilation SHALL preserve semantic correctness while minimizing invalidation.

A query/cache boundary SHOULD align with semantic boundaries such as:

- source normalization;
- module identity;
- public signature;
- public effect bound;
- public capability contract;
- type layout/ABI;
- macro expansion output;
- generic instance;
- MIR body;
- optimization result.

Private effect inference or local lifetime inference SHOULD NOT invalidate unrelated modules when the public semantic contract remains unchanged.

Every cache key SHALL include all semantically relevant inputs, as already required by `OMNI-BUILD`.

A cache hit obtained with incomplete semantic input is a correctness defect, not a performance bug.

---

## 12. Ownership, cleanup, and cancellation model

The compiler SHALL represent cleanup as explicit semantic control flow.

At minimum, every cleanup edge SHALL know:

- which places are initialized;
- which loans are live;
- which destructors apply;
- which locks/capabilities are held;
- which task context is active;
- which effects remain permitted.

Cancellation SHALL enter the same cleanup machinery rather than bypassing ordinary ownership reasoning.

A cancelled task SHALL NOT silently skip required safe-language cleanup unless the applicable task/runtime rule explicitly defines the resource as non-finalizing.

Process abort, power loss, target reset, and invalid unsafe execution remain separate from ordinary cancellation and unwind semantics.

---

## 13. Unsafe assumptions and optimization legality

Unsafe code does not create a universal optimization license.

An unsafe precondition SHALL be represented as a proof obligation or assumption attached to a specific operation and semantic region.

Optimizations may consume that assumption only while it remains valid under the transformed execution.

The translation validator SHOULD compare:

- source preconditions;
- transformed preconditions;
- assumption dominance;
- invalidation events;
- pointer provenance;
- aliasing/borrow facts;
- memory visibility;
- volatile/device observations.

A transformation that makes an unsafe assumption live longer than the source semantics permits SHALL fail validation.

---

## 14. How to build Omni

### 14.1 The build target

The first useful Omni build is a **verified end-to-end compiler vertical slice**:

```text
omni build hello.omni
        |
        v
validated package graph
        |
        v
lex + parse + resolve + typecheck
        |
        v
validated MIR
        |
        v
native object
        |
        v
linked executable
        |
        v
program execution
```

The milestone is not a parser demo. It is a source-to-running-binary path whose semantic stages are connected.

### 14.2 Required bootstrap prerequisites

The bootstrap plan requires:

- a supported Rust toolchain for the current bootstrap compiler;
- the pinned Edition 1 specification/data artifacts;
- the Stage-0 source and grammar subset;
- the canonical seed target;
- assembler/object writer;
- linker/image builder;
- target emulator or supported hardware;
- cryptographic/hash tooling required by the release process;
- the exact external tools recorded in the bootstrap TCB manifest.

The canonical seed target is `riscv64-unknown-none-elf-omni_v1[lp64]+rv64imac`; a hosted x86-64 seed path MAY exist for accessibility, but one immutable canonical path remains the bootstrap reference.

### 14.3 Development toolchain

The current engineering direction uses Rust for the bootstrap/compiler implementation, with:

- recursive-descent parsing plus Pratt-style expression parsing;
- Rowan-style lossless CST infrastructure where beneficial;
- `ena`-style unification for type inference;
- Polonius-compatible borrow analysis adapted to Omni MIR;
- Cranelift for fast development code generation;
- LLVM for release optimization;
- MLIR for future accelerator-oriented paths;
- structured asynchronous runtime infrastructure;
- PubGrub-style deterministic dependency resolution;
- Salsa-style incremental query architecture;
- libFuzzer/cargo-fuzz-class fuzzing;
- structured JSON/SARIF-compatible diagnostics;
- replay via effect interception.

These technologies are implementation choices. They do not override Omni semantics.

### 14.4 Recommended repository construction

The implementation repository SHOULD separate:

```text
spec/
    normative/
    models/
    schemas/
    grammar/
    references/
    tests/

compiler/
    lexer/
    parser/
    syntax/
    names/
    types/
    effects/
    ownership/
    hir/
    mir/
    optimizer/
    codegen/
    linker/

runtime/
stdlib/
tools/
lsp/
formatter/
conformance/
fuzz/
bootstrap/
targets/
docs/
```

The exact repository layout MAY evolve, but semantic specifications, executable models, conformance cases, compiler implementation, and target-specific code SHALL remain distinguishable.

### 14.5 Minimal vertical-slice implementation order

The implementation SHOULD progress in the following order:

1. source normalization and tokenization;
2. canonical parser/CST/AST;
3. deterministic name resolution;
4. a small but sound bidirectional type checker;
5. basic ownership and initialization tracking;
6. effect/capability plumbing;
7. minimal semantic MIR;
8. MIR verifier;
9. native code generation;
10. linking and executable startup;
11. end-to-end conformance cases;
12. optimization validation;
13. advanced generics, traits, handlers, async, regions, managed domains, and profiles.

Advanced subsystems MAY be prototyped early, but they SHALL NOT be presented as production-complete merely because their scaffolding exists.

---

## 15. What is required before the compiler is production-grade

A production-grade Edition 1 implementation requires more than successful compilation of examples.

### Language implementation

- complete lexer and grammar;
- deterministic name resolution;
- complete type and refinement solver;
- ownership/borrow checking;
- effect/capability enforcement;
- exact evaluation/error behavior;
- concurrency and memory semantics;
- unsafe and FFI enforcement;
- canonical MIR generation and validation;
- target lowering and ABI implementation.

### Tooling

- structured diagnostics;
- formatter;
- package/build tooling;
- language server;
- debugger/source mapping;
- documentation tooling;
- replay/debug trace support.

### Verification

- executable semantic model;
- rule-linked conformance suite;
- parser fuzzing;
- differential testing;
- metamorphic testing;
- memory litmus tests;
- ABI/wire golden vectors;
- optimization translation validation;
- reproducible build verification;
- DDC evidence.

### Security and release

- TCB inventory;
- provenance/SBOM records;
- signed manifests;
- artifact hashes;
- extension fingerprinting;
- dependency/advisory policy;
- security audit;
- independent review;
- release errata and revocation procedure.

---

## 16. AI-oriented compiler interaction contract

Omni's structured semantics make a useful machine-facing feedback protocol possible.

A tooling client, including an AI coding system, SHOULD be able to request:

```text
source
  -> diagnostics
  -> source ranges
  -> rule IDs
  -> inferred type/effect/capability facts
  -> ownership state
  -> MIR
  -> failed proof obligations
  -> suggested machine-applicable repairs
```

The compiler should expose the semantic reason for a failure rather than only a textual error message.

For example:

```text
TYPE-0018
proof obligation:
    index < len(buffer)

origin:
    source span X:Y..X:Z

required action:
    prove obligation
    or insert an explicit checked operation
    or change the algorithm
```

This turns compilation into a semantic feedback loop instead of an opaque parser gate.

The AI is then operating against the same language law as a human programmer.

---

## 17. Required semantic-closure test matrix

The Edition 1 closure corpus SHALL progressively cover the following cross-module matrix:

| Seam | Primary risk | Required witness |
|---|---|---|
| SOURCE -> LEX | Unicode, normalization, invisibles | source/token golden corpus |
| LEX -> GRAMMAR | token ambiguity | parser differential/fuzz corpus |
| GRAMMAR -> NAMES | scope and hygiene | resolution matrix |
| NAMES -> TYPES | generic/coherence ambiguity | solver cases |
| TYPES -> OWN | invalid moves/borrows | MIR ownership cases |
| NUM -> EVAL | overflow/conversion | arithmetic model tests |
| OWN -> ERROR | partial cleanup | unwind/fault cases |
| EFFECTS -> CAPABILITIES | authority escalation | negative security cases |
| EVAL -> ERROR | fault ordering | observable-event cases |
| EVAL -> CONC | cancellation | scheduler/model cases |
| CONC -> MEM | `hb`/`sw` correctness | memory litmus cases |
| OWN -> CONC | task-local ownership | transfer/lifetime cases |
| UNSAFE -> OPT | stale assumptions | translation-validation cases |
| MIR -> OPT | semantic preservation | equivalence/validation suite |
| IR -> ABI | layout/calling convention | ABI golden vectors |
| BUILD -> REPRO | hidden input | hermeticity tests |
| PKG -> BUILD | resolution identity | resolver reproducibility cases |
| SOURCE -> DEBUG | source mapping | round-trip mapping cases |
| CORE -> PROFILE | profile leakage | profile isolation cases |

A release candidate with an uncovered high-risk seam SHALL record the gap explicitly and SHALL NOT claim complete implementation certification until the applicable release gate is satisfied.

---

## 18. Metrics that matter

Omni implementation progress SHALL NOT be measured primarily by lines of compiler code or number of supported syntax constructs.

More meaningful metrics are:

- percentage of normative rules mapped to executable tests/models;
- percentage of semantic seams with differential coverage;
- number of unresolved P0/P1 specification defects;
- independent compiler agreement rate;
- MIR validator coverage;
- translation-validation coverage;
- reproducible-build rate;
- DDC convergence rate;
- time from diagnostic to corrected build;
- incremental invalidation precision;
- conformance corpus growth;
- number and age of unexplained implementation-defined choices.

A small compiler with complete semantic evidence is preferable to a large compiler with undocumented behavior.

---

## 19. Ratification gates arising from the closure audit

Before Edition 1 is ratified, the standards process SHOULD require:

1. every normative module present in the release manifest;
2. no unresolved normative placeholder;
3. every construct mapped across source, static, dynamic, memory, fault, and conformance layers;
4. executable semantic models for high-risk domains;
5. canonical/equivalent semantic MIR rules;
6. independent parser implementation;
7. independent semantic/model implementation;
8. independent native implementation;
9. differential and metamorphic testing;
10. memory-model litmus coverage;
11. optimization translation validation;
12. reproducible build evidence;
13. DDC evidence;
14. security/unsafe/FFI audit;
15. signed immutable release publication.

These gates extend the existing distinction between **definition completeness**, **implementation completeness**, **ratification**, and **certification**.

---

## 20. Interpretation of the current specification status

The phrase **“complete normative candidate”** means that the intended Edition 1 language definition is assembled and governed as a complete suite.

It does **not** mean:

- every compiler component is finished;
- every implementation target exists;
- every proof is mechanically discharged;
- independent compilers already agree;
- DDC has already succeeded;
- the language has already been ratified;
- a production distribution already exists.

This distinction is deliberate.

The standard is allowed to be definition-complete while the engineering implementation remains incomplete.

That is exactly the state in which a serious compiler project can now perform the semantic-closure audit without pretending the compiler is already finished.

---

## 21. Updated engineering priority

The next highest-value Omni work is therefore not another broad feature expansion.

It is:

> **Build the semantic closure evidence chain from static semantics into dynamic semantics, then make the compiler and executable models conform to it.**

The first focused slice SHALL be:

```text
OMNI-TYPES
   +
OMNI-NUM
   +
OMNI-OWN
   +
OMNI-EFFECTS
        |
        v
     HIR/MIR
        |
        v
OMNI-EVAL
   +
OMNI-ERROR
   +
OMNI-CONC
   +
OMNI-MEM
        |
        v
OMNI-MACHINE
```

The audit SHALL be adversarial. Every seam SHALL be treated as potentially defective until the specification, model, implementation, and conformance corpus agree.

This is the point at which Omni transitions from a very ambitious language design into an **independently implementable programming-language standard and compiler ecosystem**.

---

## 22. Historical integration note

Earlier Omni design documents and implementation plans remain valuable because they explain why ownership, algebraic effects, structured concurrency, Polonius-style analysis, generational references, linear resources, tensor/SIMD facilities, comptime build logic, safer FFI, replay debugging, and a multi-stage bootstrap were selected.

However, historical sections SHALL be interpreted according to the consolidation rule at the beginning of this document:

> **Edition 1 normative text governs; earlier versions explain intent, trade-offs, and evolution.**

Historical implementation snapshots SHALL not be read as current repository-status claims unless independently refreshed by a current repository audit.

---


---



## 18. Adversarial semantic-closure audit: async drop during cancellation

### 18.1 Audit status

This audit records the first high-volatility cross-module semantic-closure case raised against the Edition 1 candidate: a structured-concurrency scope using `cancel_on_error`, a sibling task undergoing cleanup, an asynchronous resource finalizer, and a fault during that finalizer.

The current suite already determines several important pieces of the scenario. `CONC-0005` requires `cancel_on_error` to commit the first error/panic, request cancellation of unfinished siblings, join them, and return the committed cause with suppressed causes. `ERR-0012` makes cancellation a distinct control signal observed at declared cancellation points. `ERR-0015` requires cleanup to execute under a bounded cancellation mask. `EVAL-0017` permits `async defer` to suspend during asynchronous cleanup subject to those bounds. The memory model includes cancellation publication as an event and defines `hb`, `sw`, and related relations.

However, the current candidate also contains an important boundary that SHALL be closed before ratification: the normative ownership/error rules define ordinary destruction as synchronous, while the historical/consolidated design material separately describes async destruction. An implementation therefore must not infer that historical `async drop` wording is already a complete Edition 1 semantic rule. The exact async-finalization protocol, its cancellation inheritance, and its fault aggregation must be promoted into normative rules.

### 18.2 Normative resolution target

The intended Edition 1 resolution for this audit is: **cancellation is not active panic/unwind**. Cancellation initiates a distinct control state. Cleanup performed because of cancellation remains cleanup, but it executes inside a bounded cancellation-masking context so that required finalization has a defined opportunity to complete.

Accordingly, `ERR-0005` SHALL apply only when a second panic occurs while an actual panic/unwind state is active. A cancellation-triggered cleanup fault SHALL NOT become a double panic merely because the cleanup was initiated in response to an earlier panic in a sibling task.

A capability, I/O, resource, or target fault arising during cancellation cleanup SHALL be associated with the cleanup operation and handled according to the cleanup operation's declared fault policy. When the enclosing operation already has a committed cause, the cleanup fault becomes a suppressed cause unless the applicable policy explicitly escalates it to isolate or abort. A cleanup panic remains a panic and is subject to the ordinary cleanup-precedence and double-panic rules.

### 18.3 Proposed normative rule amendments

The following rules are proposed for promotion into `OMNI-ERROR`, with corresponding references added to `OMNI-CONC`, `OMNI-EVAL`, `OMNI-OWN`, and the abstract-machine model. Rule identifiers shown with `AUDIT-` are audit placeholders only and SHALL receive final domain-local identifiers in the ratified registry.

| Rule | Proposed normative requirement |
|---|---|
| `AUDIT-ASYNC-0001` | Cancellation is a distinct control state from panic/unwind. Entering cancellation SHALL NOT by itself make `ERR-0005`'s "another panic/unwind is active" condition true. |
| `AUDIT-ASYNC-0002` | When `cancel_on_error` cancels an unfinished sibling, the sibling transitions into cancellation-aware cleanup. The committed sibling/scope cause remains the previously committed operation outcome; cancellation SHALL NOT replace that committed cause merely because cleanup is still executing. |
| `AUDIT-ASYNC-0003` | Required cleanup, including an asynchronous finalizer where the type/operation explicitly declares one, executes under a bounded cancellation mask. The mask suppresses delivery of the enclosing cancellation request to the currently executing cleanup operation, but does not revoke unrelated capability failures, resource failures, or panic semantics. |
| `AUDIT-ASYNC-0004` | An asynchronous finalizer MAY await only operations that are legal in the cleanup effect/capability context. An awaited child created by cleanup SHALL inherit the cleanup context and its remaining cancellation-mask budget. The child is not automatically forced to fail solely because the parent task is cancellation-marked. |
| `AUDIT-ASYNC-0005` | A cancellation request received while a cleanup operation is masked remains pending for the enclosing task. It becomes observable only after the cleanup returns to a cancellation-observation boundary or the mask is exhausted according to `ERR-0015`. |
| `AUDIT-ASYNC-0006` | A non-panic fault occurring during cancellation cleanup SHALL be recorded as a cleanup/suppressed cause and SHALL NOT constitute a second panic. Escalation to isolate or abort occurs only when required by the fault policy, cleanup policy, realtime/profile bound, or target supplement. |
| `AUDIT-ASYNC-0007` | A panic arising inside cancellation cleanup is an actual panic. If no panic/unwind state is otherwise active, ordinary panic policy applies. If an enclosing panic/unwind is active in the same control context, `ERR-0005` applies. A panic in a sibling task does not itself make another task's cancellation cleanup an active unwind context. |
| `AUDIT-ASYNC-0008` | Completion of an asynchronous cleanup operation is a semantic synchronization event with respect to the cleanup owner. For memory operations performed by the cleanup executor, the event order SHALL be represented in the `OMNI-MEM` event graph. The standard SHALL define the required `sb`, `sw`, and `hb` edges through the executor/cleanup protocol rather than leaving them to backend scheduling. |
| `AUDIT-ASYNC-0009` | Lock acquisition, channel operations, capability use, and device operations performed by cancellation cleanup obey their ordinary operation commit rules. Cancellation cannot retroactively roll back an operation that has already committed. |
| `AUDIT-ASYNC-0010` | If cleanup cannot complete within the applicable cancellation-mask budget, the cleanup outcome SHALL be classified by the profile and artifact cleanup policy. The implementation SHALL report the original committed cause together with the cleanup timeout/failure when the reporting channel permits; it SHALL NOT silently discard required cleanup failure. |
| `AUDIT-ASYNC-0011` | A cancellation cleanup SHALL execute at most once for each registered cleanup identity. Retry mechanisms must be explicit and may not cause duplicate destruction of the same owned value. |
| `AUDIT-ASYNC-0012` | A cleanup executor SHALL preserve ownership and capability authority while suspending an asynchronous cleanup. Suspending or resuming cleanup SHALL NOT create an implicit capability, extend a borrow beyond its proven lifetime, or bypass drop-order rules. |

### 18.4 Scenario derivation

For the audit scenario, the mechanically derived outcome is therefore:

1. Task A commits the panic under the `cancel_on_error` protocol.
2. The scope publishes cancellation to unfinished siblings.
3. Task B observes cancellation at its declared cancellation point and enters cancellation-aware cleanup.
4. Task B's asynchronous finalizer executes under the bounded cancellation mask.
5. The finalizer's network operation may fail with `CapabilityFault`; that fault is a cleanup fault/suppressed cause unless its declared policy explicitly escalates.
6. The `CapabilityFault` is **not** a double panic merely because Task A previously panicked.
7. If the finalizer itself panics, that is a genuine panic. It invokes `ERR-0005` only if an actual panic/unwind context is active in the same cleanup control chain.
8. If the finalizer awaits a child operation, that child remains subject to the cleanup context and remaining budget, rather than receiving an implicit immediate cancellation solely because Task B was cancelled.
9. A lock acquisition or device operation uses its normal atomic/commit semantics. Cancellation does not interrupt a committed lock acquisition retroactively. The exact synchronization edges are recorded in the memory-event graph.
10. If the cleanup budget expires, the standard applies the declared cleanup/profile escalation policy while retaining the original committed cause and recording the cleanup failure when representable.

### 18.5 Required model and conformance evidence

Before ratification, the suite SHALL add a model state containing at least:

`Normal | PanicUnwind | CancellationPending | CleanupMasked | AwaitingCleanupChild | CleanupFaulted | CleanupPanicked | CleanupCompleted | Abort | Isolate`

and explicit transitions for:

`panic_commit`, `cancel_publish`, `cancel_observe`, `cleanup_enter`, `mask_enter`, `await_child`, `capability_fault`, `cleanup_panic`, `cleanup_complete`, `mask_timeout`, and `scope_finalize`.

The conformance corpus SHALL include, at minimum:

- cancellation cleanup with successful async finalization;
- cancellation cleanup with `CapabilityFault`;
- cancellation cleanup with ordinary resource failure;
- cancellation cleanup with a panic;
- cancellation cleanup with nested await;
- nested await that observes the remaining cleanup budget;
- lock acquisition during cleanup;
- cancellation before and after lock-operation commit;
- device/volatile operation during cleanup;
- cleanup-budget exhaustion;
- simultaneous sibling failures with deterministic cause/suppression ordering;
- repeated or racing cancellation publication;
- destruction exactly-once verification.

These cases SHALL be represented both as executable abstract-machine traces and as rule-linked implementation tests.

### 18.6 Required cross-module synchronization

The closure patch is incomplete unless the following normative modules agree:

- `OMNI-ERROR`: cancellation-vs-unwind classification, cleanup fault precedence, suppression/escalation;
- `OMNI-EVAL`: async cleanup suspension and exact interruption points;
- `OMNI-OWN`: asynchronous finalization identity, exactly-once destruction, ownership across suspension;
- `OMNI-CONC`: cancellation publication, scope joining, cleanup executor semantics;
- `OMNI-MEM`: event-graph edges for cancellation, lock operations, cleanup completion, and awaited child completion;
- `OMNI-EFFECTS`: required effects/capabilities of asynchronous finalization and cancellation masking;
- `OMNI-IR` / `OMNI-OPT`: preservation of cleanup state, cancellation points, and fault observations during lowering and optimization.

No historical v2.0 async-drop description may substitute for these normative rules. Until the modules agree, the feature SHALL be considered an open semantic-closure item for ratification.

---


## 23. Adversarial semantic-closure audit: `OMNI-UNSAFE` → `OMNI-OPT`

### 23.1 Audit status and finding

This section records the second high-volatility semantic-closure audit of the Edition 1 candidate. It attacks the boundary between programmer-supplied unsafe obligations and optimizer transformations, with particular attention to assumption lifetime, instruction motion, dead-code elimination, FFI/device mutation, pointer provenance exposure, and translation validation.

The existing candidate already provides the essential ingredients: unsafe operations carry named obligations; invalid unsafe execution begins at the first event at which a required precondition is false; optimizer assumptions identify their origin, operands, scope, dominance, object lifetime, invalidation events, and proof status; and optimization legality is defined by observational refinement. However, the word **live** in `UNSAFE-0006` is not by itself a sufficient mechanical criterion. A compiler cannot implement conformance from an informal variable-liveness interpretation when the assumption actually depends on allocation identity, object validity, alias state, capability authority, synchronization, or external mutation.

This audit therefore replaces semantic vagueness with an explicit **assumption-token model**. An unsafe assumption is a first-class MIR fact whose validity interval is computed from its dependencies and invalidation events. Optimizer passes SHALL consume and transform these facts explicitly; they SHALL NOT infer assumption lifetime from ordinary SSA use-liveness alone.

### 23.2 Mechanical unsafe-assumption model

An optimizer assumption token SHALL have the canonical form:

```text
A = (
    id,
    obligation_id,
    predicate,
    subjects,
    provenance,
    activate_at,
    valid_until,
    dependency_set,
    invalidation_set,
    authority_scope,
    observation_class,
    proof_status
)
```

Where:

- `id` is a unique MIR assumption identity;
- `obligation_id` names the originating `OMNI-UNSAFE` obligation;
- `predicate` is the exact proposition being assumed;
- `subjects` are allocation identities, places, capabilities, refinements, or other semantic entities on which the proposition depends;
- `provenance` identifies the source operation, source span, rule IDs, and proof/assertion record;
- `activate_at` is the exact MIR event at which the assumption becomes usable;
- `valid_until` is an explicit set of MIR program points/events at which the assumption ceases to hold;
- `dependency_set` contains all semantic facts whose change can invalidate the predicate;
- `invalidation_set` contains all operations/events that can invalidate any dependency;
- `authority_scope` constrains which memory/capability domains the assumption may justify;
- `observation_class` records whether the guarded operation is pure, ordinary memory, volatile/device, capability, persistent, synchronization, or another observable class;
- `proof_status` is one of the specification-defined assumption/validation states.

A compiler SHALL be conforming only if every optimizer transformation that consumes, clones, merges, splits, hoists, sinks, or otherwise propagates an assumption token preserves its dependency and invalidation semantics.

### 23.3 Assumption validity is not ordinary variable liveness

`UNSAFE-0006` SHALL be interpreted as follows:

> An unsafe assumption is valid exactly on the control-flow/event region in which every member of its dependency set remains valid and every required authority remains available, subject to the operation's sequencing, synchronization, and external-effect contracts.

SSA use-liveness MAY be used as a profitability or representation optimization, but it SHALL NOT by itself extend an unsafe assumption's validity interval.

The compiler SHALL compute assumption validity over a semantic event graph, not merely over registers or local variable uses.

For an assumption concerning `index < array.len`, the dependency set SHALL include at minimum the identity and lifetime of the array object and the specific logical length fact used by the predicate. A call may preserve the assumption only when its declared effects and memory contract prove that those dependencies cannot change. An opaque FFI call with write/unknown-memory authority therefore invalidates any assumption whose dependency domain intersects the call's possible writes unless the FFI contract proves non-interference.

This is **not** a universal compiler memory fence. It is a semantic invalidation rule. An operation that cannot affect the relevant dependency need not block motion. An operation that can affect it creates an explicit invalidation edge.

### 23.4 MIR representation requirements

The normative MIR SHALL represent the following operations or equivalent canonical forms:

```text
assume.create      assumption_id, obligation_id, predicate, dependencies
assume.use         assumption_id, guarded_operation
assume.invalidate  assumption_id, invalidation_reason
assume.split       assumption_id -> child_ids
assume.merge       assumption_ids -> merged_id
assume.end         assumption_id
expose.provenance  allocation/domain/address-token
```

The exact textual spelling is implementation-defined only at the serialization layer; the semantic operation classes are normative.

Every guarded unsafe operation SHALL reference the assumption token(s) on which its legality depends. A transformation SHALL NOT manufacture an implicit unsafe assumption merely because a backend IR happens to encode an equivalent instruction pattern.

### 23.5 Invalidation classes

An assumption SHALL have an explicit invalidation class for each dependency. At minimum the standard SHALL distinguish:

| Invalidation class | Examples | Default consequence |
|---|---|---|
| lifetime | drop, deallocation, region end, storage reuse | assumption invalid immediately before reuse/dereference could observe the object as dead |
| initialization/value validity | move, destructive update, variant change, reinitialization | invalidate affected value predicates |
| shape/refinement | length/capacity mutation, container mutation | invalidate affected bounds/refinement predicates |
| alias/memory | unknown write, foreign call with write authority, raw memory operation | invalidate overlapping memory predicates unless contract proves disjointness |
| capability | revocation, lease expiry, scope exit | invalidate authority predicates |
| synchronization | required lock/epoch/atomic ordering no longer established | invalidate facts whose proof depends on that ordering |
| device/volatile | reset, MMIO side effect, device barrier | invalidate device-state predicates as declared by target contract |
| provenance/address | allocation exposure, address recovery ambiguity, storage reuse | invalidate provenance-sensitive assumptions as specified below |
| target state | feature disablement, address-space change, execution-mode change | invalidate target-dependent assumptions |

A compiler MAY refine these classes into a more precise dependency graph, but SHALL NOT coarsen them so far that a transformation can retain an assumption after a semantically relevant invalidation.

### 23.6 Instruction motion and hoisting/sinking

A transformation that moves an operation `O` across an event `E` is legal only if all of the following hold:

1. `O` remains in a control-flow region in which every assumption required by `O` is valid at the new location;
2. `E` does not invalidate any dependency of an assumption used by `O`, unless the transformed program re-establishes the assumption before `O`;
3. the move preserves all required `sb`, `hb`, `sw`, synchronization, volatile/device, persistent, capability, and fault observations;
4. the move does not cause an unsafe operation to execute on a path that could not reach the original guarded event;
5. translation validation can derive the same obligations and invalidation edges for the transformed operation.

Therefore, an opaque FFI call that may mutate an array's backing storage, length metadata, or associated state MUST be represented as an invalidation event for the corresponding dependency domain. An optimizer MAY hoist an unsafe read across that call only if the FFI contract proves that the relevant dependency remains valid or if the optimizer proves a semantically equivalent revalidation.

An ordinary register-preserving call with no relevant write/effect domain is not automatically a barrier.

### 23.7 Lock acquisition is not an automatic unsafe barrier

A lock acquisition SHALL NOT be treated as a universal compiler fence merely because it is a synchronization point. It constrains transformations according to the protected memory domain and the memory-ordering semantics of the lock.

An unsafe assumption dependent on data protected by the lock is valid after acquisition only if its proof is established under the lock's contract. Conversely, if an unsafe operation is already proven from facts independent of the lock, the optimizer need not retain an artificial barrier solely because the lock exists.

The MIR and validator SHALL record the protected domain, acquisition/release events, and synchronization edges used by the assumption proof.

### 23.8 Dead-code elimination of invalid unsafe executions

The distinction between `unsafe.unchecked`, checked unsafe execution, and observable device/volatile operations is normative.

For an `unsafe.unchecked` operation whose result and all effects are unobservable under the active observation set, an optimizer MAY remove the operation even though a hypothetical execution in which its precondition is false would become invalid at that event. `UNSAFE-0005` withdraws post-event semantic obligations for that invalid execution; it does not make an invalid execution a required semantic observation comparable to a defined panic or fault.

However:

- the optimizer SHALL preserve all observations that occur before the unsafe event;
- the optimizer SHALL NOT use the possibility of invalid unsafe execution to justify transformations that alter any valid execution in which the precondition holds;
- a `checked` unsafe operation whose failed-precondition behavior is a defined trap/fault is an ordinary semantic observation and MAY NOT be deleted when its trap is observable;
- volatile, MMIO, device, persistent, synchronization, or capability operations are retained according to their explicit observation contracts even if their returned value is unused.

This means the correct rule is **not** “unsafe operations can never be deleted,” and also **not** “invalid operations are freely deletable.” The correct rule is that invalidity is outside the required post-event observation set, while any independently specified operation effects remain observable and must be preserved.

### 23.9 Strict provenance and address exposure

`MEM-0006` exposes an address without automatically granting arbitrary provenance. `expose_addr` SHALL therefore create an explicit provenance-escape event in MIR.

The exposure event SHALL carry:

```text
exposure = (
    exposure_id,
    allocation_id/domain,
    address-space,
    exposed-address-range,
    exposure-epoch,
    recovery-policy,
    lifetime-dependencies
)
```

Exposure does **not** globally disable alias analysis for the entire program or entire allocation class. Instead, it creates a named possibility that an eligible integer address can later be recovered into a provenance-bearing pointer under the exact `from_exposed_addr` rules.

For optimization, an exposed allocation/domain SHALL be treated as belonging to an **exposed-alias set**. Any operation whose contract can read, write, deallocate, or otherwise change a storage location that can be recovered through that exposure must either:

1. prove disjointness from the exposed-alias set;
2. preserve the relevant ordering through explicit dependency edges; or
3. conservatively invalidate the affected assumptions.

A `from_exposed_addr` operation SHALL produce a recovery token containing the exposure identity, candidate address, resolution proof/status, and applicable provenance policy. Alias analysis and translation validation SHALL use this token rather than treating the resulting pointer as an unrelated SSA value.

### 23.10 Unknown FFI and foreign-memory barriers

An FFI declaration SHALL publish its memory/capability contract. At minimum it SHALL classify whether the operation may:

- read or write arbitrary memory;
- read or write named borrowed/owned regions;
- retain pointers beyond the call;
- mutate object headers or externally visible lengths;
- perform volatile/device accesses;
- synchronize with other agents;
- invoke callbacks;
- allocate/free foreign storage;
- alter target state;
- return aliases to previously passed addresses.

A foreign call with an unknown or maximally conservative memory contract SHALL invalidate every optimizer assumption whose dependency may overlap the declared foreign-access domain. It SHALL NOT be legal for an optimizer to move a dependent unsafe operation across such a call solely because the backend IR marks the call as opaque.

A `pure`, `readonly`, or `nomem` FFI claim is itself an unsafe/FFI obligation and SHALL be represented as an assumption with provenance. An incorrect claim is therefore subject to the unsafe invalid-execution model and translation-validation requirements.

### 23.11 Translation-validation obligations

For each transformation involving an unsafe assumption, the validator SHALL be able to recover:

```text
source operation
    -> originating obligation
    -> predicate
    -> dependency set
    -> invalidation events
    -> transformed placement
    -> preserved/re-established proof
    -> resulting observation set
```

A validation record SHALL identify every assumption token consumed by the transformation. If an assumption is cloned, the clones SHALL retain a derivation edge to the original obligation and MAY narrow but never silently widen the original authority scope.

A transformed program SHALL be rejected when:

- a required assumption has no provenance;
- an assumption is used after an unmodeled invalidation event;
- a guarded operation moves to a path outside its original domination/proof region;
- an unsafe assumption's dependency domain is widened without a corresponding proof;
- a provenance-exposed alias can conflict with a transformation that claims disjointness without proof;
- a required volatile/device/synchronization observation is removed or reordered;
- validation cannot classify the transformed observation behavior under the selected build assurance level.

### 23.12 Canonical adversarial outcomes

#### Scenario A — bounds assumption across opaque mutation

```text
assume A: index < array.len
unsafe.read array[index]
ffi_call()  // may mutate array metadata
use value
```

The unsafe read MAY be moved across `ffi_call()` only if the FFI contract proves that `array.len` and the relevant storage cannot be changed in a way that invalidates `A`. Otherwise `ffi_call()` emits `assume.invalidate(A, foreign-write)` and motion is forbidden without re-establishing the predicate.

#### Scenario B — dead unchecked read

```text
let x = unsafe.unchecked { array.read(index) }
if false:
    use(x)
```

If the read is ordinary, non-volatile, non-device, non-persistent, and has no independently observable allocation/capability/fault effect under the active contract, DCE MAY remove it. The hypothetical false-precondition execution does not create a required trap observation because it is invalid from the unsafe event onward. A checked variant with a defined bounds fault, or a volatile/device read, is not covered by that permission and remains observable.

#### Scenario C — exposed-address aliasing

```text
let raw = expose_addr(ptr_a)
let addr = raw + delta
let ptr_b = unsafe.from_exposed_addr(addr)
write(ptr_b, value)
write(ptr_a, other)
```

The optimizer SHALL NOT infer non-aliasing merely from distinct SSA provenance histories. The validator must inspect the exposure/recovery domain and either prove that `ptr_b` resolves outside `ptr_a`'s affected range or preserve an ordering compatible with the possible alias. If recovery is ambiguous and the unsafe contract is false, the execution becomes invalid at the recovery/dereference event as defined by `OMNI-MEM` and `OMNI-UNSAFE`.

### 23.13 Required normative amendments

The following rules SHALL be promoted from audit placeholders into the final rule registry, with final domain-local IDs assigned during ratification:

| Audit rule | Required amendment |
|---|---|
| `AUDIT-OPT-UNSAFE-0001` | Unsafe assumptions are explicit semantic facts in MIR and SHALL have obligation provenance, dependency sets, invalidation sets, activation points, validity regions, and proof status. |
| `AUDIT-OPT-UNSAFE-0002` | Unsafe-assumption lifetime SHALL be computed from semantic dependencies and invalidation events, not from ordinary SSA/register use-liveness. |
| `AUDIT-OPT-UNSAFE-0003` | Unknown/opaque effects that may mutate an assumption dependency SHALL invalidate the corresponding assumption unless the operation contract proves non-interference. |
| `AUDIT-OPT-UNSAFE-0004` | A transformation SHALL NOT move a guarded unsafe operation across an invalidation event unless it proves the dependency survives or re-establishes the predicate before the moved operation. |
| `AUDIT-OPT-UNSAFE-0005` | Invalid unsafe execution is not itself a required observation after the invalidating event; defined traps/faults and independently observable operations remain subject to normal optimization legality. |
| `AUDIT-OPT-UNSAFE-0006` | Volatile/device/persistent/capability/synchronization operations SHALL NOT be removed merely because their result is unused. Their observation contracts govern optimization legality. |
| `AUDIT-OPT-UNSAFE-0007` | Provenance exposure creates an explicit exposed-alias domain and recovery token; exposure SHALL NOT globally disable optimization, and optimization SHALL NOT assume non-aliasing inside the exposed domain without proof. |
| `AUDIT-OPT-UNSAFE-0008` | FFI memory contracts SHALL determine the invalidation domain for unsafe assumptions; unknown contracts use a conservative overlap rule. |
| `AUDIT-OPT-UNSAFE-0009` | Translation validation SHALL validate assumption provenance, dependency preservation, invalidation ordering, and observation refinement for every transformed unsafe assumption. |
| `AUDIT-OPT-UNSAFE-0010` | A stale, provenance-less, widened, or otherwise unvalidated unsafe assumption is a compiler-verifier error and SHALL prevent emission of the affected optimized artifact under a conforming build. |

### 23.14 Conformance corpus additions

The Edition 1 conformance corpus SHALL add at least the following cases:

1. unsafe bounds assumption across a pure call that cannot mutate the dependency;
2. unsafe bounds assumption across a call that mutates unrelated storage;
3. unsafe bounds assumption across an opaque write-capable FFI call;
4. unsafe assumption invalidated by object drop/reallocation;
5. assumption re-established after a runtime check;
6. assumption narrowed after an alias-splitting proof;
7. DCE of a dead ordinary `unsafe.unchecked` read;
8. retention of a dead-result volatile/device read;
9. retention of a checked unsafe fault whose fault is observable;
10. exposed-address recovery with proven disjointness;
11. exposed-address recovery with possible aliasing;
12. exposed-address recovery after storage reuse;
13. capability revocation invalidating an optimizer assumption;
14. synchronization-dependent assumption before and after lock release;
15. vectorization whose lanes rely on an unsafe non-overlap obligation;
16. speculative/hoisted unsafe access on a path that originally could not reach the operation;
17. assumption cloning and merge with preserved provenance;
18. translation-validator rejection of a deliberately stale assumption;
19. conservative fallback after validation timeout;
20. differential testing between at least two lowering paths for the same unsafe obligation.

### 23.15 Closure criterion

This seam SHALL be considered semantically closed only when an implementation-independent model can derive, for every optimizer transformation involving an unsafe operation:

```text
obligation
  -> assumption
  -> semantic dependencies
  -> invalidation events
  -> valid execution region
  -> transformed placement
  -> preserved/re-established proof
  -> permitted observations
```

and when the compiler's MIR verifier and translation validator reject every transformation that violates that derivation.

The resulting rule is therefore stronger than an optimizer barrier model. Omni does **not** require every unsafe obligation to become a global fence. It requires every unsafe obligation to become a **machine-checkable semantic dependency object** whose scope and expiration are explicit. This permits aggressive optimization where the proof survives while making unsound assumption motion mechanically rejectable.

# Part II — Integrated v3.4 Converged Reconciliation Record

The following is retained from the v3.4 complete specification as historical and architectural reconciliation context. It remains useful for understanding the source-order, observability, reproducibility, ABI/FFI, provenance, freeze-gating, and fail-closed commitments that preceded Edition 1. **Where this text conflicts with Edition 1, Edition 1 wins.**

# Omni Complete Specification v3.4

## Status

This document captures the converged specification outcome of the latest adversarial
review cycle discussed in the referenced ChatGPT conversation. It is the normative
archive for the v3.4 rule set and supersedes earlier draft language wherever it is
more specific.

This is a specification document, not an implementation claim. A rule may be defined
here before it is implemented in the current Rust bootstrap. When implementation
coverage is missing, the compiler and runtime must continue to fail closed.

## 1. Normative hierarchy

The project uses five progression states:

1. `Defined` - the rule exists in the specification.
2. `Formalized` - the rule has an explicit machine-checkable shape.
3. `Implemented` - the code path exists.
4. `Qualified` - the code path has been exercised by the project qualification gates.
5. `Frozen` - the behavior is locked for the current release line.

Specification prose can establish `Defined` and, where the document is precise enough,
`Formalized`. It cannot by itself establish `Implemented`, `Qualified`, or `Frozen`.

## 2. Specification authority

The repository's current living target may lag behind this v3.4 archive.
Implementation work must therefore distinguish:

* spec-defined behavior;
* currently qualified bootstrap behavior;
* explicitly fail-closed unsupported behavior;
* archived historical experiments that are not normative.

Where the codebase and this document differ, the current implementation must not invent
semantics to bridge the gap.

## 3. Converged rules

### 3.1 Source order and observability

Source semantics are evaluated in source order unless the compiler proves an
observational-equivalence transformation. Optimizers do not get to reorder or drop
operations merely because the compiler author believes the result is "pure enough".

In particular:

* observable reads and writes preserve program order unless a proof authorizes change;
* I/O is not assumed pure;
* "deterministic" does not mean "semantically reorderable";
* reordering requires explicit evidence of equivalence under the language model.

### 3.2 Reproducibility does not alter meaning

Reproducibility requirements apply to the emitted artifact, not to source semantics.
The compiler may canonicalize metadata, build envelopes, hashes, and packaging details.
It may not canonicalize argument evaluation, control flow, or value selection in a way
that changes program meaning.

### 3.3 FFI and ABI evaluation order

FFI lowering preserves the language evaluation order and then packs values according to
the declared ABI.

Required rule:

* evaluate argument 1;
* evaluate argument 2;
* evaluate the remaining arguments in source order;
* then place the resulting values into ABI-defined locations.

Forbidden rule:

* sorting arguments or otherwise reordering evaluation for reproducibility;
* changing source order to match a canonical encoding;
* treating ABI packing as permission to rewrite semantic order.

### 3.4 Provenance and identity

Semantic provenance is the identity required to preserve meaning across lowering,
optimization, storage, and reconstruction. Representation choice is implementation-
defined only when it preserves that meaning.

The implementation may choose any internal encoding that preserves:

* allocation identity where required;
* subobject identity where required;
* recovery rules for raw exposure;
* provenance-preserving lowering through IRs;
* failure-closed behavior where recovery is not sound.

### 3.5 Alignment, layout, and canonical lowerings

The compiler may canonicalize a representation only when canonicalization is semantics-
preserving. Layout canonicalization is permitted for envelopes, snapshots, and release
artifacts. It is not a license to invent a new source meaning.

### 3.6 Freeze gating

Freeze readiness is achieved only when the corresponding artifacts actually exist and
have been verified. A textual specification alone does not satisfy qualification.

Frozen means:

* the behavior has concrete test artifacts;
* the behavior has been exercised by the appropriate gate;
* the project has chosen to stop changing it in the current release line.

### 3.7 Fail-closed policy

Unsupported behavior must fail explicitly rather than silently degrade into a guessed
implementation. This applies to parser support, lowering, optimization, runtime
execution, FFI, and tooling.

## 4. Current converged architectural commitments

The following commitments remain in force:

* source semantics remain source-order-sensitive unless proven otherwise;
* optimizer transformations require observational-equivalence justification;
* provenance-preserving lowering is mandatory for any raw-pointer or allocation-
  identity-sensitive representation;
* the implementation must keep the current bootstrap target fail-closed for unqualified
  features;
* semantic freeze is distinct from documentation freeze;
* reproducibility envelopes cannot mutate program behavior;
* the current implementation may be narrower than the spec and still be correct if it
  refuses unsupported constructs.

## 5. Residual implementation expectations

The current repository still needs to reconcile its v0.x bootstrap line with the v3.4
convergence model. The main expectation is not "implement everything at once" but
"expand coverage in compatible batches without violating fail-closed boundaries".

That means future work should prefer:

* adjacent source/IR/runtime steps that share dependencies;
* conformance evidence that covers a batch, not a single isolated syntax point;
* plan documents that explicitly record what is implemented, qualified, and deferred;
* compatibility notes that distinguish superseded experiments from current rules.

## 6. Relationship to other docs

This archive should be read together with:

* `Omni_v3.4_Third_Audit_Adjudication.md`
* `Omni_v3.4_Implementation_Plan.md`
* `../../CURRENT_IMPLEMENTATION_MATRIX.md`
* `../../VERSIONING_AND_BOOTSTRAP_PLAN.md`
* `../../spec/README.md`

## 7. Practical summary

The v3.4 conclusion is simple:

* the spec is stricter than the current bootstrap in several places;
* the bootstrap must continue failing closed where it has not caught up;
* reproducibility, provenance, and ABI rules cannot rewrite source semantics;
* freeze claims require artifacts, not just prose;
* the next implementation step should be a batched dependency-aware reconciliation.


## 24. Adversarial semantic-closure audit: `OMNI-EFFECTS` → `CAPABILITIES`

### 24.1 Audit status and finding

This section records the third high-volatility semantic-closure audit of the Edition 1 candidate. It attacks the boundary between statically classified effects and dynamically held capability authority, with particular attention to multi-shot continuations, capability lifetime and revocation, closure capture, effect polymorphism, handlers, storage escape, and confused-deputy behavior.

The existing candidate correctly separates the concepts. `EFF-0008` defines effects as observations and explicitly states that an effect declaration does not itself grant resource authority. `EFF-0009` makes capabilities sealed nominal values, `EFF-0010` permits attenuation but not widening, `EFF-0011` ties delegation to ownership, `EFF-0012` defines provider-side revocation races, and `EFF-0015`/`EFF-0016` define one-shot and multi-shot continuation regimes. However, those rules do not yet provide one complete mechanical account of how capability authority is represented when continuations are captured, cloned, resumed, stored, or revoked across effect-polymorphic boundaries.

The closure requirement is therefore:

> **Effect polymorphism SHALL never erase, duplicate, widen, extend, or silently relocate capability authority.**

An effect row describes what a computation may observe; a capability environment describes what authority that computation currently possesses. Both are part of the static judgment, but they have different conservation laws. Effects may be masked by handlers; capability authority may only be transferred or narrowed under ownership and provider rules. Neither operation may implicitly strengthen the other.

### 24.2 Capability environment model

The static capability environment SHALL be represented conceptually as:

```text
K = {
    cap_id,
    capability_type,
    rights,
    scope,
    lifetime,
    ownership_mode,
    issuer,
    revocation_domain,
    attenuation_lineage
}
```

For a closure or continuation, the capture environment SHALL additionally record:

```text
C = (
    captures,
    capability_captures,
    borrow_captures,
    ownership_captures,
    escape_class,
    repetition_class,
    resumption_class
)
```

`effect row` and `capability environment` SHALL remain distinct dimensions. A function may have a small effect row while holding significant authority; the authority remains constrained by the capability environment and ownership rules.

### 24.3 Multi-shot continuations cannot duplicate affine authority

A multi-shot continuation represents potentially repeated execution of the continuation body. Repetition therefore requires that every value whose semantics would be duplicated across resumptions be safely repeatable.

A capability value SHALL be classified according to its repetition permission:

1. `affine`: at most one consuming transfer;
2. `borrow-shared`: repeatable shared observation only for the proven borrow lifetime;
3. `borrow-mut`: exclusive and non-repeatable while the mutable borrow is active;
4. `persistent`: explicitly repeatable according to provider-defined semantics;
5. `ephemeral`: valid only within a single dynamic activation or lease.

A multi-shot continuation MUST NOT capture an affine, move-only, linear, uniquely borrowed, or otherwise non-repeatable capability in a way that would require authority duplication on resumption.

A multi-shot continuation MAY capture a shared capability borrow only when all of the following are proved:

- the borrow remains valid for every permitted resumption;
- the continuation cannot create overlapping mutable access contrary to `OMNI-OWN`;
- the capability provider permits concurrent shared use;
- revocation is checked at each authorized operation according to `EFF-0012`;
- no resumption causes a capability to escape beyond the borrow's proven lifetime.

If these conditions cannot be established, the program is erroneous and translation SHALL fail rather than silently cloning or promoting the capability.

### 24.4 Capability use in handlers

A handler invocation and continuation capture SHALL preserve the capability environment of the captured computation.

A handler SHALL NOT:

- manufacture a capability from an effect term;
- convert a capability into a serializable ordinary value merely by capturing it;
- duplicate a non-repeatable capability through continuation cloning;
- widen the rights of a captured capability when installing a handler;
- extend a capability lifetime beyond its ownership/borrow rules;
- cause resumption to occur under a stronger authority environment than the captured continuation had at its suspension point.

On resumption, the effective authority is the intersection of:

```text
captured authority
∩ current provider-valid authority
∩ resumption scope
∩ handler-supplied authority
```

No component may widen the captured authority. Provider revocation may further reduce the effective authority at the authorized operation.

### 24.5 Explicit continuation capability record

MIR SHALL represent continuation capture/resumption with an explicit authority summary or semantically equivalent metadata:

```text
cont.capture continuation_id,
             effect_context,
             ownership_context,
             capability_set,
             repetition_class,
             lifetime_constraints

cont.resume continuation_id,
            authority_context,
            resumption_count

cont.clone continuation_id -> child_id

cap.use capability_id, operation
cap.attenuate capability_id -> child_id, rights_subset
cap.revoke_observe capability_id, provider_epoch
```

The concrete textual representation is implementation-defined; the semantic information is normative.

A continuation clone SHALL retain derivation provenance to the original continuation and SHALL be rejected when its captured environment contains non-repeatable authority that would be duplicated.

### 24.6 Revocation is a dynamic authority decision, not an untracked exception

A revocable capability operation SHALL have a statically visible failure contract appropriate to the API.

The fact that a capability was valid when a function began does not prove that it remains valid when the authorized operation commits. Every revocable operation is checked at its authorization point under the provider epoch/revocation rules.

A provider MAY expose revocation through:

- `Result<T, CapabilityFault>`;
- a declared `throw<CapabilityFault>`/fault effect;
- an explicit `panic` contract when the operation is specified as unrecoverable;
- an isolation/abort policy when the provider declares the operation non-recoverable.

The choice SHALL be part of the operation's static API contract. A runtime `CapabilityFault` SHALL NOT appear as an undeclared ordinary outcome of a function whose signature promises that no such dynamic fault mechanism exists.

An implementation SHALL NOT silently convert a revocation race into arbitrary `panic`, `isolate`, or `abort` merely because the signature lacks a failure path. The API must already specify the applicable fault channel or the translation must reject the call as type/effect-incompatible.

### 24.7 Async I/O, DMA, and revocation

A capability used to authorize an asynchronous operation SHALL remain valid according to the lease/operation contract for the entire period during which the provider may commit effects under that authority.

For DMA and similar device operations:

1. registration establishes the DMA lease and provider-visible ownership;
2. revocation/reset creates a provider-visible invalidation event;
3. in-flight completion is resolved at the provider's atomic commit point;
4. cleanup may revoke, fence, quarantine, or recover the buffer only according to the DMA lease contract;
5. the cleanup executor SHALL NOT be assigned responsibility for preventing device-side memory corruption that the provider contract already defines as possible after revocation;
6. any required cache/coherence/device ordering remains part of `OMNI-MEM` and the target/device profile.

The language model therefore distinguishes **authority failure** from **memory corruption**: a rejected capability operation yields the provider-declared fault or result; it does not authorize the executor to ignore outstanding device writes or to reuse storage before the lease has ended.

### 24.8 Effect polymorphism does not hide captured authority

For a generic function of the form:

```text
fn memoize<T, U, E>(input: T, f: (T) -> U !E) -> U !E
```

well-formedness of `E` does not by itself authorize the function to store, clone, move, or extend the closure `f`.

The closure type SHALL carry capability-capture information independently of the effect row. At minimum it SHALL expose the semantic properties required for storage and invocation:

```text
Closure<T,U,E,
        captures = ...,
        capabilities = ...,
        Send = ...,
        Sync = ...,
        escape = ...,
        repetition = ...>
```

A generic API that stores a closure beyond the lexical scope of its construction SHALL prove all captured values, including capabilities and borrows, satisfy the target storage lifetime and transfer requirements.

A function cannot become authority-neutral merely by being effect-polymorphic. A closure with `!{log}` may still capture `SystemAdminCap`; the capability remains part of the closure's semantic type and ownership state.

### 24.9 Confused-deputy prevention

An effect handler or generic higher-order function SHALL execute operations using only the capability environment explicitly available to the current dynamic authority scope.

Passing a closure to a less-privileged function SHALL NOT transfer more authority than the closure already owns or borrows. Conversely, a privileged outer scope SHALL NOT cause a callback to gain implicit access to capabilities that were not captured or explicitly passed.

A handler that intercepts an effect MAY replace the provider implementation, but the replacement SHALL satisfy the handler's declared capability requirements. Handling `io` does not grant arbitrary filesystem/network authority.

The compiler and MIR verifier SHALL reject a transformation or elaboration in which:

```text
available_effects ⊇ requested_effects
```

is used as evidence for:

```text
available_capabilities ⊇ requested_capabilities
```

because the two relations are intentionally independent.

### 24.10 Escape and storage rules

Capability-bearing closures, continuations, trait objects, futures, generators, actors, and heap objects SHALL carry sufficient ownership/authority metadata for escape analysis.

A capability captured by a lexical borrow cannot escape into:

- a `'static` or equivalent immortal storage class;
- a detached task;
- a global lazy cell;
- a persistent object;
- an externally serialized representation;
- a process or distributed boundary;

unless the capability type/provider explicitly defines an authenticated transferable representation and the required lifetime/authority transition is performed.

A capability moved into long-lived storage remains owned by that storage. The compiler SHALL NOT infer a fresh caller-owned copy when later code still refers to the original binding.

### 24.11 Capability attenuation through effect handlers

An effect handler MAY attenuate authority for its body or continuation, but it SHALL NOT widen it.

If a handler receives capability set `K_h` and invokes a continuation requiring `K_c`, the continuation SHALL execute with:

```text
K_effective = attenuate(K_c, K_h)
```

where `attenuate` can only remove rights, narrow scope, reduce quota/duration, or otherwise restrict authority. An attempted widening is a static error unless the provider explicitly issues a new sub-capability under its own authority protocol.

A handler SHALL NOT reify its private provider authority as a general-purpose capability available to the handled computation merely because the handled effect name matches.

### 24.12 Cancellation and capability cleanup

Cancellation does not revoke capabilities by language-level implication. Capability revocation is provider-controlled unless the enclosing scope explicitly owns the relevant capability and its cancellation contract includes revocation.

Cleanup code therefore SHALL distinguish:

- cancellation observation;
- capability revocation request;
- provider-confirmed revocation;
- resource cleanup;
- final ownership release.

A cancellation mask may permit cleanup to continue using a still-valid capability, but the cleanup must still obey provider revocation races and operation commit semantics. Masking cancellation does not mask revocation.

### 24.13 Capability serialization and continuation cloning

A capability SHALL be non-serializable by default, including when captured by:

- a multi-shot continuation;
- a closure;
- a future/generator state machine;
- an actor message;
- a persistent object;
- a package/plugin boundary.

Cloning a continuation therefore does not clone capability authority. A cloned continuation must either share an explicitly repeatable authority reference under provider rules or be rejected.

Provider-defined transfer may use an authenticated capability handle whose import performs issuer, audience, rights, freshness, revocation epoch, scope, and resource-binding validation. The imported handle is a new authority issuance, not an accidental copy of the original opaque token.

### 24.14 Required static/MIR invariants

The following invariants SHALL hold for every valid Edition 1 MIR graph:

1. **No authority amplification:** every capability in a derived environment has a derivation edge from an existing capability or trusted provider issuance.
2. **No affine duplication:** a move-only capability has at most one live consuming owner.
3. **No hidden capability capture:** every closure/continuation carrying authority identifies that capture explicitly.
4. **No repetition violation:** a multi-shot continuation cannot contain a non-repeatable capability obligation.
5. **No borrow escape:** capability borrows cannot escape their proven lifetime or exclusive-access region.
6. **No effect-authority substitution:** effect presence never constitutes proof of capability possession.
7. **No revocation blindness:** every revocable operation has a provider-defined authorization point and outcome contract.
8. **No stale authority:** an optimizer or lowering pass cannot reuse a capability assumption after a modeled revocation/lease invalidation event.
9. **No serialization widening:** byte-level copies cannot create capability authority.
10. **No confused-deputy widening:** handlers and higher-order functions cannot confer authority unavailable in their explicit capability environment.

### 24.15 Required normative amendments

The following audit rules SHALL be promoted into the final rule registry, with final domain-local IDs assigned during ratification:

| Audit rule | Required amendment |
|---|---|
| `AUDIT-EFF-CAP-0001` | Effect rows and capability environments are distinct semantic dimensions; effect inclusion SHALL never imply capability possession. |
| `AUDIT-EFF-CAP-0002` | Closure and continuation types SHALL carry capability-capture metadata sufficient for ownership, lifetime, escape, repetition, and transfer checking. |
| `AUDIT-EFF-CAP-0003` | Multi-shot continuation capture SHALL reject affine, linear, uniquely borrowed, ephemeral, or otherwise non-repeatable capabilities unless a provider-defined repeatability contract exists. |
| `AUDIT-EFF-CAP-0004` | Continuation cloning SHALL NOT duplicate capability authority; every clone SHALL retain derivation provenance and repeatability constraints. |
| `AUDIT-EFF-CAP-0005` | Handler resumption SHALL execute under an authority environment no stronger than the captured authority, current provider-valid authority, and handler restrictions. |
| `AUDIT-EFF-CAP-0006` | Revocable capability operations SHALL expose a declared result/fault/panic/isolation outcome; runtime capability rejection SHALL NOT become an undeclared ordinary failure. |
| `AUDIT-EFF-CAP-0007` | Async DMA/device capability revocation SHALL distinguish authority invalidation from memory-lease completion and storage reuse. |
| `AUDIT-EFF-CAP-0008` | Effect-polymorphic higher-order functions SHALL remain subject to closure capability capture and escape rules. |
| `AUDIT-EFF-CAP-0009` | Capability-bearing values SHALL NOT enter long-lived, detached, persistent, serialized, or distributed storage without explicit provider-authorized transfer semantics. |
| `AUDIT-EFF-CAP-0010` | Effect handlers may attenuate capability authority but SHALL NOT widen it or manufacture capability values from effect declarations. |
| `AUDIT-EFF-CAP-0011` | Cancellation masking SHALL NOT suppress capability revocation semantics; cleanup capability use remains subject to provider authorization at each operation. |
| `AUDIT-EFF-CAP-0012` | A stale capability assumption after revocation/lease expiry is a verifier error under the same semantic-assumption discipline used for unsafe optimization facts. |

### 24.16 Conformance corpus additions

The Edition 1 conformance corpus SHALL include at least:

1. multi-shot continuation capturing an affine capability — rejection;
2. multi-shot continuation capturing a shared capability borrow with valid lifetime — acceptance;
3. multi-shot continuation attempting concurrent `&mut` capability borrowing — rejection;
4. continuation clone attempting to duplicate a linear resource capability — rejection;
5. handler resumption after provider revocation — deterministic provider-defined rejection;
6. revocable file operation returning `CapabilityFault` through a declared `Result` channel;
7. revocable operation whose API explicitly maps rejection to panic;
8. revocable operation with no declared failure channel — rejection;
9. DMA lease reset racing completion and cleanup;
10. cleanup attempting buffer reuse before DMA lease completion — rejection;
11. effect-polymorphic `memoize` storing a closure containing a non-static capability borrow — rejection;
12. effect-polymorphic `memoize` storing a closure owning a persistent/repeatable capability under provider rules — acceptance;
13. effect row `io` with no filesystem capability — rejection;
14. filesystem capability held without an `io` effect where no operation occurs — acceptance;
15. handler handling `io` without filesystem/network authority — rejection of unauthorized provider use;
16. byte serialization/deserialization of an opaque capability token — no authority creation;
17. detached task attempting to capture a lexical capability borrow — rejection;
18. capability attenuation to a strict subset of rights — acceptance;
19. attempted capability widening through an effect handler — rejection;
20. optimizer/lowering pass attempting to reuse a revoked capability assumption — verifier rejection;
21. capability capture in a future across cancellation and revocation;
22. persistent object attempting to store a non-transferable capability;
23. distributed actor message attempting to transfer a non-serializable capability;
24. authenticated provider handle import producing a new, narrowed capability under explicit transfer rules.

### 24.17 Closure criterion

This seam SHALL be considered semantically closed only when an implementation-independent model can derive, for every effect handler, closure, continuation, or effect-polymorphic call:

```text
effect context
  -> capability environment
  -> capture set
  -> ownership/repetition classification
  -> escape/lifetime constraints
  -> provider/revocation state
  -> permitted resumption/storage operations
  -> resulting effective authority
```

and prove all of the following:

```text
no capability duplication
no capability widening
no borrow escape
no hidden authority capture
no undeclared revocation outcome
no effect-to-authority substitution
no stale authority use
```

A release SHALL NOT claim semantic closure for the effect/capability seam while any of these obligations is discharged only by backend convention, undocumented runtime behavior, or optimizer assumption.

### 24.18 Resolution of the three adversarial scenarios

**Scenario 1 — Multi-shot continuation:** the compiler rejects a continuation that would duplicate affine capability authority. Shared capability borrows are accepted only when repeated resumption remains valid under lifetime, concurrency, provider, and revocation constraints.

**Scenario 2 — Revocation:** capability validity is checked at the provider's authorization/commit point. The API's declared result/fault/panic/isolation contract determines the observable failure. The absence of such a contract is a static incompatibility, not permission for the runtime to invent one.

**Scenario 3 — Effect-polymorphic confused deputy:** the closure's capability captures are independent of `E`. Storing or escaping the closure therefore invokes ordinary ownership, lifetime, `Send`/`Sync`, and capability-transfer checks. `memoize` cannot smuggle `SystemAdminCap` merely because the closure's effect row is small.

---

---

# Part III — Integrated v2.0 Complete Design and Rationale

The following preserves the earlier complete specification and its extensive design rationale. It is included to avoid losing decisions, motivations, implementation guidance, and historical context during consolidation. **It is non-normative where it differs from the Edition 1 candidate.**

# OMNI PROGRAMMING LANGUAGE
## Exhaustive Specification, Design Rationale & Implementation Guide
**Version:** 2.0 | **Bootstrap:** Rust | **License:** Apache 2.0
**Last Revised:** 2026-04 — Incorporates improvements from research into Rust 2024 Edition roadmap and pain points, algebraic effect systems (Koka/Eff/Unison), Vale's generational references and linear types, Mojo/MLIR AI acceleration, Zig's comptime build model, Swift/Kotlin structured concurrency, and state-of-the-art compiler diagnostics (Elm/Rust standards).

> Superseded by `Omni_Complete_Specification_v3.4.md` and retained for historical context.

---

## TABLE OF CONTENTS

1. [What Omni Is](#1-what-omni-is)
2. [How Every Decision Was Made — The Full Rationale](#2-how-every-decision-was-made)
3. [Language Philosophy & Design Principles](#3-language-philosophy--design-principles)
4. [Type System](#4-type-system)
5. [Memory Model & Ownership](#5-memory-model--ownership)
6. [Effect System — A Unified Model for Side Effects](#6-effect-system)
7. [Concurrency & Execution Model](#7-concurrency--execution-model)
8. [Syntax & Surface Design](#8-syntax--surface-design)
9. [Module, Package & Visibility System](#9-module-package--visibility-system)
10. [Error Handling & Failure Model](#10-error-handling--failure-model)
11. [Standard Library Architecture](#11-standard-library-architecture)
12. [Compilation Model & IR Design](#12-compilation-model--ir-design)
13. [Runtime Architecture](#13-runtime-architecture)
14. [Tooling & Developer Experience](#14-tooling--developer-experience)
15. [Testing, Diagnostics & Validation](#15-testing-diagnostics--validation)
16. [Security, Safety & Capability System](#16-security-safety--capability-system)
17. [Interoperability & FFI](#17-interoperability--ffi)
18. [Bootstrap Strategy & Self-Hosting Roadmap](#18-bootstrap-strategy--self-hosting-roadmap)
19. [Phased Implementation Plan](#19-phased-implementation-plan)
20. [HELIOS Framework (Platform Layer)](#20-helios-framework-platform-layer)
21. [Current State & What Remains](#21-current-state--what-remains)
22. [Improvements Added in v2.0 — Research Basis](#22-improvements-added-in-v20)

---

## 1. WHAT OMNI IS

Omni is a **multi-level, hybrid programming language platform** designed for advanced developers, combining strong safety and performance guarantees with flexible abstraction layers. It enables controlled transitions between low-level and high-level programming through explicit execution modes, prioritizes deterministic correctness, and serves as the foundational language for building complex platform-level systems such as HELIOS.

This definition emerged through an exhaustive requirements interview covering over 160 design decisions, then refined in v2.0 through systematic research into the current state of language design — examining Rust's 2024 roadmap improvements and documented pain points, algebraic effect systems (Koka, Eff, Unison), Vale's linear types and generational references, Mojo's MLIR-based AI acceleration, Zig's comptime build model, structured concurrency from Swift and Kotlin, and best-in-class compiler diagnostic systems.

### 1.1 What Omni Is Not

Omni is explicitly not a beginner scripting language. It is not a loose general-purpose language with no enforced boundaries. It is not designed primarily for UI-heavy frontend development. It is not a toy compiler project. It is not a research-only language that will never ship. It is a structured platform language with a strict, deterministic core and carefully designed extensible outer layers.

### 1.2 The One-Sentence Definition

> **Omni is a layered, multi-paradigm, multi-runtime programming language platform with a structured ecosystem, deterministic core, algebraic effect system, and controlled extensibility, serving as the foundation for building advanced systems including the HELIOS cognitive platform.**

### 1.3 Primary Domains

Systems programming, backend services, AI and data infrastructure, and platform/framework construction. Secondary: CLI tooling and libraries. UI-heavy frontend is the acknowledged weak area — possible but not optimized.

---

## 2. HOW EVERY DECISION WAS MADE

### 2.1 Why "Everything, But Structured"

The initial design intent was to do everything. The resolution was organizing goals into layers. This pattern — layered architecture with strict defaults and controlled escalation — is the backbone of every design decision. The v2.0 research confirmed this approach is correct: every successful multi-domain language (Rust, Swift, Kotlin, Mojo) converged on layering as the answer to the "do everything" problem.

### 2.2 Why Not Choose One Memory Model

Omni's use cases span from low-level systems code where GC is unacceptable overhead, to high-level application code where manual memory management is unnecessary pain. The decision: ownership as the core model, with GC-compatible layer in higher-level modes. v2.0 adds generational references (from Vale) as an ergonomic bridge for graph-like and cyclic data structures, eliminating the `Rc<RefCell<T>>` ceremony that is one of Rust's most documented frustrations.

### 2.3 Why Rust as the Bootstrap Language

Omni's design philosophy maps almost exactly onto Rust's design space. Using Rust means the development team works in a mental model aligned with what Omni itself will become. v2.0 reconfirms this: the Rust 2024 ecosystem (Cranelift, Salsa, tower-lsp, rowan, polonius-engine) provides a mature, well-maintained foundation for every layer of the bootstrap project.

### 2.4 Why Determinism Is the Sacred Principle

No undefined behavior, deterministic correctness by default. v2.0 adds: determinism must extend to the effect system — all effects (IO, async, exceptions) must be trackable at the type level, so programs can reason about their side effects as precisely as they reason about their types.

### 2.5 Why an Effect System Was Added in v2.0

Research into Koka (Microsoft Research), Eff, and Unison reveals that an algebraic effect system cleanly unifies many language mechanisms that would otherwise require independent design: exceptions, async/await, generators, cancellation, probabilistic programming, and other control-flow abstractions. Koka has proven this approach is practically usable — not just theoretically interesting. Omni adopts a staged effect system: built-in effects first, user-defined effects later.

### 2.6 Why Structured Concurrency Was Strengthened

Research into Kotlin's coroutine model and Swift's structured concurrency manifesto shows that the most common concurrency bugs come from tasks that escape their intended scope. Structured concurrency (child tasks cannot outlive their parent scope) prevents this class of bugs by construction. Kotlin enforces parent-child relationships through its Job hierarchy, ensuring that child coroutines cannot outlive their parents, which prevents common resource leaks. Omni adopts structured concurrency as a hard constraint enforced at the type system level.

### 2.7 Why Borrow Checker Ergonomics Were Explicitly Addressed

The Rust language team's own 2024 roadmap explicitly acknowledges borrow checker ergonomics as the primary usability problem with ownership-based languages. Specific friction points documented by the community: where clause repetition, graph/cyclic data structures requiring `Rc<RefCell<T>>`, inability to partially borrow struct fields independently, and self-referential structure difficulty. Omni addresses all of these proactively: implied bounds, field projection support, Polonius from day one, generational references.

### 2.8 Why Diagnostic Quality Is a Language Feature

Research comparing compiler diagnostics across Rust, Elm, Go, Java, TypeScript, Kotlin, and Python confirms Elm and Rust lead the field. Crucially: good diagnostics require both UX design and architectural discipline — the compiler must preserve enough information at each pass to produce helpful messages. Omni treats diagnostic quality as a first-class design constraint, not an afterthought.

---

## 3. LANGUAGE PHILOSOPHY & DESIGN PRINCIPLES

### 3.1 Core Philosophy

Six foundational principles, ordered by priority:

**1. Deterministic Correctness First.** No undefined behavior. No silent failures. Every execution explainable by the language rules. All side effects trackable through the type-and-effect system.

**2. Layered Complexity, Not Flat Complexity.** A beginner in high-level mode is not exposed to ownership, effect annotations, or capability concepts unless they descend. An expert has full access to memory layout, unsafe operations, and raw hardware interfaces.

**3. Safe Defaults, Controlled Escape Hatches.** The default for everything is the safest available option. Unsafe, mutable, nondeterministic, or privileged behavior requires explicit opt-in. Always visible in code review, never implicit.

**4. Effects Are Visible in Types.** Every function that performs IO, launches a task, allocates heap memory, or has any side effect declares those effects in its type. Pure functions are pure by construction.

**5. Everything Extensible, But Not Everywhere Enabled.** The macro system, plugin system, metaprogramming, and runtime code generation all exist, but none are enabled everywhere by default.

**6. Phased Implementation From Structure to Platform.** The language is built in phases: correct structural core → functional core → enrichment → expansion → platform-level maturity.

### 3.2 Design Priority Ordering

When priorities conflict:
1. Safety and correctness (never sacrificed)
2. Performance (never sacrificed without an explicit, visible trade-off)
3. Developer productivity
4. Flexibility and extensibility (layered, never at the cost of 1 or 2)
5. Ecosystem breadth (important in later phases; irrelevant in early phases)

### 3.3 The "No Footguns at Default Reach" Principle (v2.0 Addition)

Features that are commonly misused or cause subtle bugs must be placed behind explicit opt-in at whatever abstraction level they exist. This principle was validated by examining Rust's documented pain points: shared mutable state, implicit drop-as-cancellation in async, implicit `Rc<RefCell<T>>` patterns. Every convenience in Omni must be explicitly invoked.

---

## 4. TYPE SYSTEM

### 4.1 Typing Style

Omni is statically typed by default. The typing model is a **type-and-effect system**: every expression has both a type and an effect set. A function with no side effects has the `pure` effect (empty set). A function reading from IO has the `io` effect. These appear in signatures and are checked by the compiler.

Layers:
- **Default**: static typing, effect inference
- **Strict**: effects must be explicitly annotated; for library APIs and critical systems
- **Dynamic zones**: explicitly marked modules or blocks where types resolve at runtime

### 4.2 Type Inference

Bidirectional type checking (not purely H-M top-down). More powerful than classic Hindley-Milner, produces better error messages, aligns with the research direction of Koka and recent functional languages. Effects are inferred in non-public code. Public API boundaries require explicit type and effect annotations.

Verbosity control: `@verbose_types` annotation forces explicit type expansion on a binding for debugging. `--types=minimal` flag suppresses inferred-type annotations in error messages.

### 4.3 Null Handling

Null does not exist in safe Omni code. Absence is `Option<T>`. Null pointers accessible in unsafe mode only. The `Option<T>` API includes a rich combinator set (`map`, `flat_map`, `or_else`, `filter`, `zip`, `unzip`, `transpose`) and participates in `?` propagation identically to `Result<T, E>`.

### 4.4 Error Handling in the Type System

`Result<T, E>` is the core error representation. **v2.0 additions**:

**Error set types** — inspired by Zig's error unions: a finite named set of possible error variants, exhaustively matchable, no heap allocation:
```omni
error set ParseErrors:
    InvalidSyntax(span: Span, message: String)
    UnexpectedEof(position: usize)
    InvalidEncoding

fn parse(input: &str) -> Result<Ast, ParseErrors>
```

**Typed error context chains** — the `|>` context operator wraps errors with context without erasing the underlying type:
```omni
let config = parse_config(path)? |> "while loading application config"
```

**Implicit error set widening** — when `?` propagates across a function boundary, error types are automatically widened when the relationship is a subset or conversion.

### 4.5 Generics

Monomorphization by default. **v2.0 additions**:

**Implied bounds** — when a struct is defined with a bound (`struct Cache<K: Hash>`) and a method is written on that struct, the bound is implied and does not need repeating in every method signature. This eliminates the where-clause copy-paste identified as one of Rust's ergonomic burdens.

**Variadic generics** — functions and types parameterized by arbitrary-length type tuples:
```omni
fn map_tuple<..Ts, ..Us>(t: (..Ts), f: (..Ts) -> (..Us)) -> (..Us)
```

**Limited specialization** — trait implementations can provide specialized versions for specific concrete types, with the general implementation as fallback.

### 4.6 Traits

Traits are the primary polymorphism mechanism. **v2.0 additions**:

**Trait upcasting** — `dyn SubTrait` coerces to `dyn SuperTrait` when `SubTrait: SuperTrait`. This was only recently stabilized in Rust 2024; Omni includes it from the start.

**Negative bounds** — `where T: !Copy` specifies that a type does not implement a trait. Enables API designs impossible with only positive bounds.

**Custom diagnostic attributes** — traits annotate with custom error messages when trait bounds are not satisfied:
```omni
@[diagnostic::on_unimplemented(
    message = "Type `{Self}` cannot be safely shared across threads",
    label = "add `Send + Sync` bounds or use `Mutex<T>`"
)]
trait ThreadSafe: Send + Sync
```

**Async traits (native)** — traits can contain `async fn` methods without boxing overhead. The compiler generates a concrete associated future type for each implementation.

### 4.7 Pattern Matching

Exhaustive. Expressive. Available in all expression positions. **v2.0 additions**:

**Or-patterns at all positions** — `(x | y) if cond` works at all nesting levels.

**Deconstructing function parameters**:
```omni
fn process((x, y): (i32, i32)) -> i32:
    x + y

fn handle(Point { lat, lon }: &Point) -> String:
    format("({lat}, {lon})")
```

**`let`-chains** — multiple pattern bindings chained with `and`:
```omni
if let Some(user) = get_user() and let Some(profile) = user.profile():
    display(profile)
```

### 4.8 Algebraic Data Types / Enums

Rich enums with payloads. Exhaustive matching mandatory. **v2.0 additions**:

**Sealed enums** — external crates cannot add new variants, enabling exhaustive matching without wildcards.

**Enum methods with field access** — direct method dispatch on enum variants without unwrapping.

### 4.9 Compile-Time Computation

`comptime` for compile-time evaluation of pure functions. **v2.0 additions**:

**Comptime string operations** — compile-time string manipulation for code generation and format-string validation.

**Comptime type reflection** — `comptime typeof(T)` returns structural type information as a comptime-evaluable value.

**Comptime budget annotations** — `@comptime_limit(ops: 1000000)` caps operations to prevent compilation-halting infinite loops.

### 4.10 Reflection

**Compile-time reflection (primary)**: Enumerate fields, methods, trait implementations, and type metadata at compile time with zero runtime cost.

**Limited runtime reflection (secondary, explicit import)**: Type names, debug formatting, dynamic dispatch only. `use std::reflect` required. Never Java-style full reflection.

---

## 5. MEMORY MODEL & OWNERSHIP

### 5.1 The Core Model: Ownership

Every value has exactly one owner. When the owner goes out of scope, the value is deterministically dropped. Ownership is transferred by moving. After a move, the original binding is inaccessible.

### 5.2 Borrowing

**Shared borrows (`&T`)**: Multiple can coexist; value cannot be mutated while any shared borrow is active.

**Exclusive borrows (`&mut T`)**: Exactly one can exist; no shared borrows coexist with it.

The borrow checker uses the **Polonius algorithm** from day one — more precise than NLL, eliminates false positives, adopted from the start to avoid shipping the weaker algorithm and then having users learn its workarounds. Implementation uses the `polonius-engine` crate (Apache 2.0) adapted for Omni's MIR.

### 5.3 Field Projections (v2.0)

The borrow checker tracks borrows at field granularity within a struct. Different fields can be independently borrowed simultaneously. This eliminates the pattern of splitting structs into sub-structs purely to enable independent field borrowing:

```omni
struct State:
    name: String
    config: Config
    cache: HashMap<String, Value>

fn update(state: &mut State):
    let name_ref = &state.name       -- shared borrow of `name` field
    let cache_ref = &mut state.cache -- exclusive borrow of `cache` field
    -- Both valid because they borrow different fields
```

### 5.4 Generational References (v2.0)

`Gen<T>` pairs a pointer with a generation counter. When an object is freed, its generation increments. Dereferencing a `Gen<T>` checks that the stored generation matches the current generation — O(1), detects use-after-free without lifetime tracking. This is memory-safe in safe code, no `unsafe` required:

```omni
struct Graph:
    nodes: Arena<Node>

struct Node:
    value: i32
    neighbors: Vec<Gen<Node>>  -- safe cyclic references

fn add_edge(graph: &mut Graph, from: Gen<Node>, to: Gen<Node>) -> Result<()>:
    let from_node = graph.nodes.get_mut(from)?  -- Returns Err if `from` was freed
    from_node.neighbors.push(to)
    Ok(())
```

Generational references are the recommended approach for graph algorithms, entity component systems, and any data structure where ownership cycles are structurally necessary. Runtime cost: a single integer comparison per dereference, eliminable by the region borrow checker in hot paths.

### 5.5 Linear Types (v2.0)

Rust uses **affine types** (used at most once). Omni additionally supports **linear types** (must be used exactly once — cannot be silently dropped):

```omni
linear struct DatabaseTransaction:
    connection_id: u64

impl DatabaseTransaction:
    fn commit(self) -> Result<(), DbError>
    fn rollback(self) -> Result<(), DbError>

-- Dropping a DatabaseTransaction without calling commit() or rollback() is a compile error.
```

Linear types model resources that must be explicitly released. They compose with ownership: a linear type is an owned type that additionally enforces usage. Use cases: file handles, network connections, transaction objects, cryptographic keys.

### 5.6 Inout Parameters (v2.0)

Syntactic sugar for the move-in/move-out pattern:
```omni
fn normalize(inout v: Vec<i32>):
    v = v.iter().map(|x| x * 2).collect()

let mut v = vec![1, 2, 3]
normalize(inout v)  -- v is updated; no rebinding needed
```

Compiles to move-in/move-out at the MIR level. Zero runtime overhead. Pure ergonomic improvement.

### 5.7 Arena Allocation

First-class arena allocators for bulk allocation and bulk deallocation:
```omni
let arena = Arena::new()
let node1 = arena.alloc(Node { value: 1 })
let node2 = arena.alloc(Node { value: 2 })
-- All freed when `arena` drops
```

The borrow checker enforces that arena-allocated object references don't outlive the arena.

### 5.8 Safe/Unsafe Boundary

`unsafe` blocks and `unsafe fn` enable raw pointer dereference, external unsafe function calls, accessing unsafe statics, and manual memory management. `@safe_wrapper` attribute signals that a function internally uses unsafe but guarantees safe-code contract to callers.

### 5.9 GC Compatibility Layer

Module-level `@gc_mode` annotation enables garbage collection for that module's allocations. GC-mode and ownership-mode objects have explicitly typed crossing points. GC uses a tracing collector with conservative stack scanning, tunable pause targets, and a write barrier integrated with the ownership system.

---

## 6. EFFECT SYSTEM

### 6.1 What an Effect System Is and Why Omni Has One

An effect system tracks what side effects a function may perform in addition to its types. A function's complete signature is `(Input) -> Output / Effects`. The compiler checks that callers handle every effect in a called function's signature.

Koka (Microsoft Research, actively developed 2025) demonstrates that algebraic effects and handlers let you define advanced control abstractions like async/await as a user library in a typed and composable way. This eliminates the need to design async, generators, exceptions, and cancellation separately. ICFP 2024 proceedings confirm algebraic effect handlers are both theoretically sound and practically implementable.

### 6.2 Built-In Effect Kinds

| Effect | Meaning |
|---|---|
| `io` | Reads/writes external state (filesystem, network, environment) |
| `async` | May suspend and resume (requires an executor) |
| `throw<E>` | May raise an exception of type `E` |
| `panic` | May panic (programming logic error) |
| `alloc` | May allocate heap memory |
| `rand` | May read from a random source |
| `time` | May read the current time |
| `log` | May produce log output |
| `pure` | No effects (empty effect set) |

A pure function has no effects and is safe to memoize, parallelize, or evaluate at compile time. Impure functions explicitly declare their effects.

### 6.3 Effect Inference

Effects are inferred by default in non-public code. The compiler propagates effects upward through the call graph automatically. Explicit effect annotations are required only when: defining a public API boundary, restricting a higher-order function to specific effects, or overriding an inferred effect for documentation:

```omni
-- Effect inferred (application code)
fn read_config(path: &str) -> Config:
    let text = std::fs::read_to_string(path)
    parse(text)

-- Effect explicit (public API)
pub fn fetch_user(id: u64) -> Result<User, ApiError> / io + throw<ApiError>:
    -- ...
```

### 6.4 Effect Handlers

Effects can be user-defined and handled with effect handlers:

```omni
-- Define a custom effect
effect Logging:
    fn log(msg: &str) -> ()

-- Use the effect
fn process_data(data: &[u8]) -> Result<Output> / Logging:
    Logging::log("Starting processing")
    let result = parse(data)
    Logging::log("Parsing complete")
    result

-- Install a handler
fn main():
    handle Logging:
        fn log(msg: &str):
            eprintln!("[INFO] {msg}")
    in:
        process_data(read_input())
```

Effect handlers compose: multiple effects can be handled at different call stack levels.

### 6.5 Async as an Effect (v2.0)

`async fn` is syntactic sugar for a function with the `async` effect. The executor is an effect handler, not a fixed language primitive. This means:
- Async and non-async code compose naturally through the effect system
- Different executors can be selected at different call sites
- Async cancellation is explicit via `CancelToken` (not implicit future-drop)

```omni
-- Explicit cancellation
async fn fetch_with_timeout(url: &str, timeout: Duration) -> Result<Response> / async + io:
    let (cancel_token, cancel_source) = CancelToken::new()
    let timer_task = spawn async:
        sleep(timeout).await
        cancel_source.cancel()
    let result = fetch(url).with_cancel(cancel_token).await?
    timer_task.abort()
    result
```

### 6.6 Generators as an Effect (v2.0)

```omni
fn fibonacci() -> Gen<i64>:
    let (a, b) = (0i64, 1i64)
    loop:
        yield a
        (a, b) = (b, a + b)

for n in fibonacci().take(10):
    println(n)
```

Generator functions compile to state machines with no heap allocation.

### 6.7 Effect Polymorphism

Functions can be polymorphic over effects, preserving the caller's effects automatically:
```omni
fn map<T, U, e>(items: &[T], f: (T) -> U / e) -> Vec<U> / e:
    items.iter().map(f).collect()

-- Caller's `io` effect propagates through map
fn process_files(paths: &[str]) -> Vec<Content> / io:
    map(paths, |p| std::fs::read_to_string(p))
```

---

## 7. CONCURRENCY & EXECUTION MODEL

### 7.1 The Hybrid Concurrency Model

Threads, structured async (via the effect system), message-passing via channels, and the actor model. Safe-code default prefers message-passing and ownership-based sharing over shared mutable state.

### 7.2 Structured Concurrency (v2.0, Hard Constraint)

No concurrent task can outlive the scope that created it. This is enforced by the type system:

```omni
async fn fetch_all(urls: &[str]) -> Vec<Result<Response>> / async:
    spawn_scope |scope|:
        urls.iter()
            .map(|url| scope.spawn(async: fetch(url).await))
            .collect::<JoinAll<_>>()
            .await
    -- scope dropped here; all tasks guaranteed complete
```

`spawn_scope` cancels all tasks and propagates errors if the scope exits early. It can never return before all tasks complete. Eliminates the most common class of async resource leaks.

**Unstructured concurrency** via `spawn_global` requires an explicit `GlobalSpawnCap` capability to prevent accidental use.

### 7.3 Explicit Async Cancellation (v2.0)

Async Rust's implicit cancellation (future-dropped = cancelled) is one of its most documented sources of subtle bugs. Omni makes cancellation explicit:

```omni
async fn download(url: &str, cancel: &CancelToken) -> Result<Bytes> / async + io:
    let response = http::get(url).await?
    cancel.check()?   -- explicit cancellation point
    let body = response.body().await?
    Ok(body)
```

### 7.4 Threads

OS threads via the standard library. `Send`/`Sync` marker traits enforced by the compiler. Compile-time data race prevention for all safe code.

### 7.5 Actors

Actor model with typed message channels and supervision trees. Actors are isolated: internal state never directly accessible from outside. Messages handled sequentially within an actor.

### 7.6 Determinism

Deterministic execution by default in development mode (fixed scheduler order, reproducible). Non-deterministic inputs (IO, time, randomness) are modeled as effects, so programs that depend on them declare their non-determinism explicitly in their types.

### 7.7 Execution Modes

- **Development**: Deterministic scheduler, full debug info, verbose diagnostics, replay debugging
- **Standard**: Balanced optimization, reasonable compile times
- **Release**: Maximum optimization, profile-guided, nondeterminism allowed where performance requires

---

## 8. SYNTAX & SURFACE DESIGN

### 8.1 Block Structure

Indentation-based blocks by default. Brace-delimited blocks in advanced modes. The layout engine handles indented continuations consistently, documented with a complete grammar rather than heuristics.

### 8.2 Semicolons

Newline-first syntax. A newline terminates a statement unless the next line's first token is a continuation token (binary operator, `.`, `?`, `)`, `]`, `}`). Semicolons used only where the parser requires disambiguation.

### 8.3 Expression Orientation

Expression-oriented by default. `if`, `match`, `loop`, `block`, and `try` all produce values. The `discard` keyword explicitly drops a non-`()` expression result.

### 8.4 Effect Annotations in Signatures (v2.0)

Effect annotations appear after the return type, separated by `/`:
```omni
fn read_line() -> Result<String, IoError> / io
async fn fetch(url: &str) -> Result<Response> / async + io
fn pure_compute(x: i32, y: i32) -> i32          -- inferred pure; no annotation needed
```

### 8.5 Naming Conventions

Enforced by formatter and linter, not compiler semantics. `PascalCase` for types, traits, effects. `snake_case` for values, functions, modules. `SCREAMING_SNAKE_CASE` for constants.

### 8.6 Comments and Documentation

```omni
-- line comment
--- multi-line comment ---
/// doc comment (attached to following item)
```

Doc comments: Markdown, executable `omni`-code examples, internationalization of doc text supported.

### 8.7 Annotations and Attributes

`@[attribute]` block form or `@attribute` inline. Custom diagnostic attributes (§4.6) for traits. `@[repr(c)]` for C ABI compatibility. `@safe_wrapper` for unsafe internals with safe external contract.

### 8.8 Operator Overloading

Via trait implementations, standard operator set. The `Try` trait is user-definable, enabling `?` propagation for custom result-like types beyond `Result` and `Option`.

### 8.9 Async Closures (v2.0)

First-class async closures implementing `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce`:
```omni
let fetch_and_process = async |url: &str| -> Result<Data>:
    let response = http::get(url).await?
    process(response.body().await?)
```

### 8.10 String Interpolation

`f"Hello {name}!"` — inline interpolation of any expression implementing `Display`. `d"..."` uses `Debug` formatting. The interpolated expression is evaluated eagerly and type-checked.

### 8.11 Macro System

**Declarative macros**: Pattern-based, hygienic, operating on token streams. No special permissions.

**Procedural macros**: Compiled separately, run as sandboxed processes. Defined API surface; cannot access internal compiler structures.

**Comptime code generation (v2.0)**: Comptime expressions that produce code directly, for cases where generated code is determined by type-level computation.

---

## 9. MODULE, PACKAGE & VISIBILITY SYSTEM

### 9.1 Module Hierarchy

Files → Modules → Packages → Workspaces. A workspace groups packages with a shared lockfile.

### 9.2 Visibility Levels

| Modifier | Meaning |
|---|---|
| (none) | Private to current module |
| `pub(mod)` | Current module and children |
| `pub(pkg)` | Within the package |
| `pub` | Publicly accessible |
| `pub(cap: X)` | Requires capability X |
| `pub(friend: other::module)` | Specific named module via capability grant |

### 9.3 Import System

Explicit imports. Glob imports warned in strict mode. Scoped imports that expire at block end:
```omni
fn process():
    use std::collections::{HashMap, HashSet} in:
        let map: HashMap<str, i32> = HashMap::new()
```

### 9.4 Package Manifest (`omni.toml`)

TOML format. Declares name, version, edition, dependencies, build targets, features, and **capability declarations** (v2.0 — visible to users before installation):
```toml
[capabilities]
network = ["read"]
filesystem = ["read", "write", "/tmp"]
subprocess = false
```

### 9.5 Dependency Resolution

PubGrub algorithm with lockfiles. v2.0 addition: **automatic API compatibility checking** — publishing a new version that breaks the public API without incrementing the major version is rejected by the registry.

### 9.6 Build System

Built-in extensible build system. v2.0 addition: **comptime build scripts** — build logic written in Omni using comptime functions, evaluated at build time (inspired by Zig's comptime build system):

```omni
-- build.omni (build script, evaluated at build time)
comptime fn configure_build(config: &BuildConfig):
    if config.target.is_wasm():
        config.add_feature("no_threads")
        config.set_opt_level(OptLevel::Size)
    config.add_link_lib("ssl")
```

---

## 10. ERROR HANDLING & FAILURE MODEL

### 10.1 Errors as Values

`Result<T, E>` is the fundamental error mechanism. Errors are values. The compiler warns when a `Result` is ignored. The compiler also warns when a `Result` is propagated through `?` across a function boundary that doesn't declare the `throw<E>` effect, making effect tracking consistent with error propagation.

### 10.2 Error Types

Structured, typed, rich. Each error type implements the `Error` trait. Error set types (§4.4) for finite, exhaustively matchable errors. Typed context chains via `|>`. Machine-readable error codes (E####) on all user-facing errors.

### 10.3 Propagation Operator

The `?` operator is polymorphic via the `Try` trait. Works on `Result<T,E>`, `Option<T>`, error sets, and any type implementing `Try`.

### 10.4 Panics

Panics represent logic invariant violations. In development mode: always unwind with full stack trace and debugger attachment option. Structured metadata (location, message, optional structured payload) capturable by a panic hook.

### 10.5 Resource Cleanup

Resources dropped in reverse initialization order at scope exit, in both normal and panic unwinds. Drops are synchronous. **Async drop (v2.0)**: types can implement async destruction for resources requiring async cleanup (e.g., network connection with protocol shutdown). Async drops run in a cleanup executor before the parent scope proceeds.

---

## 11. STANDARD LIBRARY ARCHITECTURE

### 11.1 Layered Design

`std::core` (no OS, no heap), `std::alloc` (heap, no OS), `std` (full OS). All IO functions in `std` declare the `io` effect.

### 11.2 Core Traits

`Copy`, `Clone`, `Drop`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Display`, `Debug`, `Default`, `Iterator`, `From`, `Into`, `TryFrom`, `TryInto`, `Error`, `Send`, `Sync`, `Try` (v2.0 — extensible `?` propagation).

### 11.3 Collections

`Vec<T>`, `HashMap<K,V>`, `HashSet<T>`, `BTreeMap<K,V>`, `BTreeSet<T>`, `VecDeque<T>`. v2.0 additions: `Arena<T>` — arena allocator. `Gen<T>` — generational reference. `SlotMap<T>` — dense mapping with stable handles (common in game engines and graph libraries).

### 11.4 IO Model

`Read`/`Write` traits. Async `AsyncRead`/`AsyncWrite`. All IO functions are capability-gated: accessing the filesystem requires a `FilesystemCap` token. IO is modeled as an effect.

### 11.5 Serialization

Pluggable system with backends for JSON, TOML, YAML, CBOR, MessagePack, and custom binary formats. Compile-time format validation ensures all fields of a `Serialize` type are serializable.

### 11.6 AI and Tensor Support (v2.0 — Required by HELIOS)

Informed by Mojo's MLIR-based approach for portable hardware acceleration:

```omni
use std::tensor::{Tensor, Shape, DType}
use std::simd::{f32x8, auto_vectorize}

-- Statically-typed tensor with compile-time shape verification
fn dot_product(a: Tensor<f32, Shape<N>>, b: Tensor<f32, Shape<N>>) -> f32:
    (a * b).sum()

-- Explicit SIMD for performance-critical paths
@[auto_vectorize]
fn scale(data: &mut [f32], factor: f32):
    for chunk in data.chunks_exact_mut(8):
        let v = f32x8::from_slice(chunk)
        (v * f32x8::splat(factor)).copy_to_slice(chunk)
```

The tensor module provides `Tensor<T, Shape>` with compile-time shape checking, SIMD dispatch for auto-vectorization, and a hardware abstraction layer for GPU dispatch (via MLIR, future phase). This is HELIOS's foundation for inference and embedding computation.

### 11.7 Time and Scheduling

Full time system: monotonic clock, wall clock, timers, scheduled tasks. Time-reading functions declare the `time` effect.

### 11.8 Cryptography

Safe high-level primitives (AEAD encryption, authenticated key exchange, password hashing) with correct defaults. Lower-level API for experts. Cryptographic key access is capability-gated.

---

## 12. COMPILATION MODEL & IR DESIGN

### 12.1 Compiler Pipeline

```
Source Text
    ↓ [Lexer + Indentation Layout Engine]
Token Stream (INDENT/DEDENT synthetic tokens)
    ↓ [Parallel Parser — Recursive Descent + Pratt, independent files parsed concurrently]
CST (Concrete Syntax Tree, lossless — all whitespace and comments preserved)
    ↓ [AST Lowering]
AST (Abstract Syntax Tree, structured)
    ↓ [Effect Resolution]
Effect-annotated AST (all effects inferred or verified)
    ↓ [Name Resolution — two-pass]
Resolved AST (all names bound to DefIds)
    ↓ [Type Inference — bidirectional]
Type+Effect-annotated AST
    ↓ [Type + Effect Checking]
Verified AST (type-correct, effect-correct, trait bounds satisfied)
    ↓ [MIR Lowering]
MIR (Mid-level IR — CFG, ownership explicit, drops inserted)
    ↓ [Borrow Checker — Polonius algorithm]
Verified MIR (memory safety proved)
    ↓ [MIR Optimization]
Optimized MIR
    ↓ [LIR Lowering]
LIR (Low-level IR — target-closer, no ownership concepts)
    ↓ [Codegen — Cranelift (dev) / LLVM (release) / MLIR (AI targets)]
Target-specific output
    ↓ [Linker]
Final binary or library
```

### 12.2 The IR Stack

**CST**: Lossless. Rowan-based. Used by formatter and incremental parser. Preserves all source information.

**AST**: Structural. Spans preserved. Arena-allocated.

**Effect-annotated AST (v2.0)**: A separate pass resolves and verifies all effect annotations before type inference. Type errors and effect errors are diagnosed independently for cleaner messages.

**MIR**: Control-flow-graph based. Ownership, borrows, drops explicit. **Polonius algorithm** for borrow checking — more precise than NLL, fewer false positives, adopted from day one.

**LIR**: Target-specific. Ownership resolved; all drops inserted as explicit calls.

### 12.3 Borrow Checker: Polonius Algorithm

Polonius treats the borrow checker as a Datalog program. It is complete (never rejects correct programs that NLL would reject due to precision limits) and maintains identical soundness guarantees. The `polonius-engine` crate (Apache 2.0) is adapted for Omni's MIR representation.

### 12.4 Backend Targets

**Development default**: Cranelift — fast compile times, correct native code. **Release default**: LLVM via `inkwell` — maximum optimization. **AI/hardware targets (v2.0, Phase 13)**: MLIR — GPU dispatch, TPU targeting, hardware-specific accelerator backends.

### 12.5 Incremental Compilation

Salsa-inspired query-based model. Every compiler pass is a query; queries cache results and invalidate only affected downstream queries when inputs change. Sub-second LSP response times for large projects.

### 12.6 Parallel Front End (v2.0)

The lexer and parser pipeline is parallelized: independent files are parsed on separate threads simultaneously. Safe because parsing is purely functional. Reduces build times proportionally to available CPU cores on large projects.

### 12.7 Automatic Applied Fixes (v2.0)

Every compiler diagnostic for which an unambiguous fix exists emits a machine-applicable fix operation. `omni fix` reads these and applies them automatically. This follows the `cargo fix` model from Rust's edition migration system.

### 12.8 Relink Without Rebuild (v2.0)

When only a library implementation changes without changing the ABI, the binary relinks without recompiling dependent crates. Requires a stable internal ABI for Omni library types (introduced Phase 6, strengthened Phase 12).

---

## 13. RUNTIME ARCHITECTURE

### 13.1 Default: AOT-First

Native AOT compilation produces standalone binaries. Correct, fast, deterministic. No runtime dependency except the system C library.

### 13.2 Modular Runtime

Modules: core, memory, async, threading, capability, plugin, tensor (v2.0). A minimal embedded program links only the core runtime.

### 13.3 Async Executor: Structured Concurrency Enforced

Work-stealing multi-threaded executor. The executor enforces the parent-task/child-task lifetime relationship at runtime. Attempting to spawn an unstructured task requires `spawn_global` and `GlobalSpawnCap`.

### 13.4 JIT Strategy

Profile-guided JIT (Phase 9), tiered adaptive JIT (Phase 10). JIT selectively replaces AOT-compiled hot paths with optimized JIT-compiled versions.

### 13.5 MLIR Integration (v2.0, Phase 13)

MLIR-compiled functions run through MLIR's LLVM backend or directly on GPU/TPU targets. Accessed through the tensor API (§11.6), not direct MLIR APIs. Provides the hardware portability that Mojo demonstrated with its MLIR foundation.

### 13.6 Binary Strategy

Static by default. Dynamic linking available as explicit option. Relink-without-rebuild for ABI-compatible library updates.

---

## 14. TOOLING & DEVELOPER EXPERIENCE

### 14.1 Official Toolchain

Unified `omni` CLI: `new`, `build`, `run`, `check`, `test`, `bench`, `fmt`, `lint`, `doc`, `clean`, `add`, `remove`, `update`, `publish`, `profile`, `debug`, `fix` (v2.0), `verify`, `semver-check` (v2.0), `migrate --edition` (v2.0).

### 14.2 Formatter (`omni fmt`)

Idempotent, deterministic. CST-based (preserves comments). Default style; minimal configuration. In strict mode: sorts imports, aligns doc comments, applies structural normalization. `--check` mode for CI.

### 14.3 Language Server (`omni-lsp`)

LSP-compliant. Query-based incremental compiler powers sub-second response times. v2.0 additions:

**Enhanced inlay hints**: Inferred types, effect annotations, and field types displayed inline (configurable verbosity).

**Effect explorer**: Hover over any function call to see its complete effect set including transitively inferred effects.

**Borrow checker visualization**: A dedicated view showing the borrow region graph for a function — which borrows are live at each point in the control flow.

**Semantic highlighting**: Effect-annotated expressions, unsafe blocks, linear types, and generational references highlighted distinctly.

### 14.4 Compiler Diagnostics Quality

Every diagnostic meets the **Elm-inspired standard** (Elm and Rust have the two best diagnostic systems as of current research):
- A stable error code (E####)
- A primary span at the exact source location
- A clear, non-jargon message
- Secondary spans with related context
- A help note suggesting how to fix the problem
- A machine-applicable fix when unambiguous

v2.0 additions:
- **Diagnostic translations**: Error messages translatable via external translation files; internationalization-ready from the start
- **Contextual "Did you mean?" suggestions**: Levenshtein distance for undefined identifiers, covering types, functions, traits, and module paths
- **Effect error messages**: When an effect is used without a handler, the error explains which effect is missing, where it originates in the call chain, and how to add the appropriate handler
- **JSON error output**: Machine-readable diagnostics for tooling integration

### 14.5 Debugger

DAP-compliant. DWARF debug info in development builds. v2.0 addition: **Replay debugging** — development builds record execution traces and replay them with perfect determinism. Since non-deterministic inputs are declared as effects (§6), the effect handler infrastructure intercepts and records them, enabling perfect replay.

### 14.6 Documentation Generator

Generates HTML from doc comments. Executable examples as tests (`omni doc --test`). v2.0 additions:
- **Effect documentation**: Functions' effect sets displayed prominently; searchable by effect
- **Versioned docs**: Registry hosts docs for all published versions with API diff-view

---

## 15. TESTING, DIAGNOSTICS & VALIDATION

### 15.1 Test Framework

`@test`, `@test_should_panic`, `@test_ignore`. Parallel execution. JUnit XML output.

v2.0 addition: `@effect_test` — tests run in a controlled effect environment where effects like `io` and `time` are replaced by mocks:
```omni
@effect_test
fn test_data_processing():
    with MockIo::file("/data.csv", "1,2,3"):
        let result = process_data()
        assert_eq(result.sum, 6)
```

### 15.2 Property-Based Testing

Built-in shrinking property-based framework. v2.0: Effect-aware property tests that can model stateful properties by controlling effect handlers.

### 15.3 Contract Annotations (v2.0)

Lightweight contracts, checked at compile time (when statically provable) or at runtime in debug builds:

```omni
@requires(n > 0, "n must be positive")
@ensures(result > 0, "result must be positive")
fn compute_positive(n: i32) -> i32:
    n * n

@invariant(self.len <= self.capacity, "length cannot exceed capacity")
struct Buffer:
    data: Vec<u8>
    capacity: usize
    len: usize
```

Contracts are: zero-cost when statically proved, checked at runtime in debug mode, stripped in release (configurable), and visible in generated documentation as formal preconditions/postconditions.

### 15.4 Fuzzing

Official fuzzing via `cargo-fuzz`/libfuzzer. Coverage-aware. v2.0: Fuzz corpus version-controlled alongside tests. CI runs fuzzing for a fixed time budget on every PR touching parsing, serialization, or security-sensitive paths.

### 15.5 Benchmarking

Statistical framework with regression tracking. v2.0 additions: `@assert_alloc_count(max: 0)` verifies no heap allocations; `@assert_compile_time_only` verifies full compile-time evaluation.

---

## 16. SECURITY, SAFETY & CAPABILITY SYSTEM

### 16.1 Security as a Layered System

Language (memory safety, no UB), types (capability types, effect annotations), runtime (capability tokens, sandbox enforcement), tooling (package signing, verified builds, audit logging), ecosystem (transparency log, revocable capabilities).

### 16.2 Capability-Based Access Control

Capabilities are unforgeable tokens granting access to specific resources. Created only by the runtime at program startup. Passed explicitly. Delegatable. Revocable.

v2.0: **Capability-effect alignment** — capabilities and effects are unified. Having the `io` capability token enables the `io` effect. The capability system and the effect system are two faces of the same mechanism.

```omni
fn write_log(cap: &IoCapability, msg: &str) / io:
    std::fs::write("/var/log/app.log", msg)?
```

### 16.3 Sandboxing

Sandboxed execution for plugins and untrusted code. Capability violations produce `CapabilityError`.

v2.0: **Fearless FFI sandboxing** — FFI calls run in an isolated execution context using stack switching. Memory corruption in an FFI call cannot propagate to Omni's managed memory because the two execution stacks are separate. This eliminates heap corruption from C code spreading into Omni code.

### 16.4 Package Security

All published packages signed. Transparency log. `omni verify` checks all packages. CLI permission flags for runtime capability grants.

v2.0: **Supply chain verification** — the package manager verifies that a package's published binary matches what building from source would produce (reproducible build verification, same model as the self-hosting bootstrap trust chain).

### 16.5 Memory Safety Layers

Safe code: complete memory safety guaranteed by the compiler. Generational references: memory-safe for cyclic data without `unsafe`. Linear types: resource safety enforced by type system. `unsafe` blocks: explicit risk declaration, auditable. GC mode: safety guaranteed by the collector.

---

## 17. INTEROPERABILITY & FFI

### 17.1 C FFI with Fearless FFI Sandboxing (v2.0)

All C FFI calls are `unsafe`. With `--fearless-ffi` build flag, FFI calls run in an isolated stack context preventing memory corruption spread:

```omni
@extern_c
fn strlen(s: *const u8) -> usize

pub fn str_len(s: &str) -> usize:
    unsafe: strlen(s.as_ptr())
```

### 17.2 Bindgen Tool

`omni bindgen` reads C headers and generates safe Omni wrappers. v2.0: Generates ownership annotations (which functions take vs. borrow pointers) based on naming convention heuristics and manual override annotations.

### 17.3 Phased Interoperability

1. C FFI with Fearless FFI (Phase 3)
2. WebAssembly export (Phase 4)
3. Python binding auto-generation (Phase 6)
4. JVM via JNI (Phase 8)
5. MLIR dialect for GPU/hardware (Phase 13)

### 17.4 ABI Stability (v2.0)

Omni defines a stable C-compatible ABI for exported types (`@[repr(c)]`) and a separate versioned "Omni ABI" for Omni-to-Omni interoperability. ABI compatibility checking is built into the package manager.

---

## 18. BOOTSTRAP STRATEGY & SELF-HOSTING ROADMAP

### 18.1 The Bootstrap Language: Rust

Rust is correct: philosophy alignment, mature tooling (`polonius-engine`, `cranelift`, `tower-lsp`, `rowan`), type safety catches bootstrap bugs at compile time. The Rust 2024 edition provides additional ergonomic improvements (async closures, precise capturing, improved lifetime elision) that make bootstrap development faster.

### 18.2 Multi-Stage Bootstrap Pipeline

Stage 0: Rust-written compiler (trusted baseline). Stage 1: Rust compiles partial Omni compiler (Omni-written). Stage 2: Stage 1 compiles full Omni compiler. Stage 3: Stage 1 == Stage 2 binary verification. Stage 4: Rust bootstrap retired to validation-only role.

### 18.3 Trust Model

Reproducible build verification. Any deviation between Stage 1 and Stage 2 outputs is a critical bug blocking release. CI verifies on every compiler commit.

### 18.4 Module-by-Module Migration

Migration order: Lexer → Parser → AST → Name Resolver → Type Inference → Type Checker → Effect Resolver → MIR Lowering → Borrow Checker (Polonius) → Optimizer → Codegen → Standard Library.

Both Rust and Omni implementations must produce identical outputs on all test inputs during transition. Output diff must be zero.

---

## 19. PHASED IMPLEMENTATION PLAN

The core principle: each phase produces a working, testable system. No phase adds complexity on top of an incomplete foundation.

### Phase 0: Project Foundation
Create the governance, repository, and CI scaffolding. All contributors can clone, build, and run tests in one command. No language features yet.

**Key deliverables**: Cargo workspace (all crates, even stubs), CI (fmt, clippy, test, docs, security), devcontainer, CONTRIBUTING, ROADMAP, ADR-0001 (Rust bootstrap), ADR-0002 (workspace structure).

**Acceptance criteria**: `cargo build --workspace` green. CI green on push. New contributor productive in 30 minutes.

---

### Phase 1: Language Core Skeleton
Programs can be parsed into a typed, richly annotated AST with useful diagnostics.

**Key deliverables**: Lexer (full token set, INDENT/DEDENT layout engine, string interpolation scaffolding, fuzz target), Parser (recursive descent + Pratt, panic-mode recovery, UI test harness, parallel multi-file), Diagnostics system (stable error codes, JSON output, machine-applicable fix encoding, "Did you mean?" foundation), CLI with `parse`, `fmt` stub, `fix` stub.

**Acceptance criteria**: `omni parse hello.omni` prints valid AST. Invalid syntax produces useful diagnostics. Formatter round-trips. Fuzz target runs 60 seconds without panics. JSON error output parseable. 20+ UI tests pass.

---

### Phase 2: Semantic Core and Type Checking
The compiler understands meaning. Name resolution, bidirectional type inference, effect inference, and basic type checking enforced. Programs execute via a minimal interpreter.

**Key deliverables**: Name resolver (two-pass, scope tree, DefId system, use declarations, implied bounds), Type system (bidirectional inference, effect set representation, basic effect inference), Type checker (unification, trait bound checking, basic effect handler checking), Basic effect kinds (io, async, panic, pure — built-in only), Minimal interpreter for testing, Integrated pipeline.

**Acceptance criteria**: Hello world, fizzbuzz, recursive fibonacci execute. Type errors produce diagnostics with spans. Basic effects inferred correctly. "Did you mean?" suggestions appear. 30+ UI tests pass.

---

### Phase 3: Ownership, Borrowing, and Safety Core
Core memory safety enforced using Polonius. Generational references and linear types available.

**Key deliverables**: MIR definition and AST→MIR lowering, CFG construction and liveness analysis, Polonius-based borrow checker (via `polonius-engine` adapted for Omni MIR), Field projection support, Generational references (`Gen<T>`) and arena allocator (`Arena<T>`), Linear type annotations and usage enforcement, `inout` parameter desugaring, Drop insertion, `unsafe` tracking, Fearless FFI sandbox skeleton.

**Acceptance criteria**: Use-after-move caught with diagnostics. Conflicting borrows caught. Field projections enable independent field borrows. Generational references catch use-after-free in debug mode. Linear types prevent dropped-without-use and double-use. 40+ UI tests pass. All Phase 2 programs still compile.

---

### Phase 4: Modules, Packages, and Build System
Multi-file, multi-package projects compile reproducibly. Comptime build scripts work.

**Key deliverables**: Hierarchical module system (file modules, inline modules, all visibility levels), `omni.toml` with capability declarations, `omni.lock` lockfile, PubGrub resolver, Build graph and incremental compilation, Monorepo workspace, Comptime build scripts (`build.omni`).

**Acceptance criteria**: 3-package project compiles. Lockfile deterministic. Comptime build script conditionally configures a build. Module privacy enforced. Capability declarations visible.

---

### Phase 5: Standard Library Core
Vec, HashMap, String, Result, Option, IO, tensor foundation — implemented, tested, documented.

**Key deliverables**: All core traits (including `Try`), Collections (Vec, HashMap, HashSet, String, Arena, Gen, SlotMap), IO traits with capability-gating and `io` effect annotation, Option and Result with `Try` integration, Error set types, Math primitives, Tensor module foundation (Tensor<T, Shape>, SIMD dispatch stubs).

**Acceptance criteria**: All stdlib types tested. `omni doc --test` passes. No `unwrap()` in library code. Tensor module compiles and runs basic operations. All IO functions correctly declare the `io` effect.

---

### Phase 6: Tooling and Developer Experience
Full development workflow from CLI alone.

**Key deliverables**: `omni-fmt` (CST-based, idempotent, effect annotation formatting), `omni-lsp` (diagnostics, go-to-def, hover with effect info, completion, inlay hints, borrow visualization, semantic highlighting), Test runner (parallel, JUnit, doc tests, `@effect_test`), Full CLI (`fix`, `verify`, `semver-check`), VS Code extension, `omni doc` with effect documentation and versioned docs.

**Acceptance criteria**: All CLI commands work. Formatter idempotent (property test in CI). LSP provides completions, go-to-def, and effect hover. `omni fix` applies at least 10 common automatically-fixable errors. Effect documentation visible in `omni doc` output.

---

### Phase 7: Advanced Type System
Generics, traits, pattern matching, macros, comptime, variadic generics, specialization.

**Key deliverables**: Generic functions/structs/enums with monomorphization and implied bounds, Trait definitions with async traits, trait upcasting, negative bounds, custom diagnostic attributes, Exhaustive pattern matching with usefulness algorithm, or-patterns, deconstructing parameters, let-chains, `comptime` with budget annotations and type reflection, Declarative and procedural macros (sandboxed), Variadic generics (basic form).

**Acceptance criteria**: Generic containers work with all element types. Async traits work without boxing. Trait upcasting works. Non-exhaustive matches rejected. Custom diagnostic attributes produce custom messages. Variadic tuples work in basic cases. 60+ UI tests pass.

---

### Phase 8: Effect System (Full Implementation)
Algebraic effects as a first-class language feature. User-defined effects. Effect polymorphism. Structured concurrency enforced.

**Key deliverables**: Full effect handler syntax and semantics, User-defined effect kinds, Effect polymorphism in generics, Structured concurrency (`spawn_scope`, enforced lifetime), Explicit cancellation tokens, Async closures (`AsyncFn` traits), Generator effects (`Gen<T>` as lazy sequence), Async drop.

**Acceptance criteria**: Custom effects can be defined and handled. Effect polymorphism works. Structured concurrency enforces task lifetime. Unstructured spawn requires `GlobalSpawnCap`. Async closures work in higher-order functions. Generators produce lazily. Async drop works. 40+ effect and concurrency tests pass.

---

### Phase 9: Concurrency Runtime and Tensor Acceleration
Production-grade concurrent/parallel execution with tensor acceleration.

**Key deliverables**: Work-stealing structured concurrency executor, Replay debugging infrastructure, Actor model with supervisor trees, Typed channels (MPSC, bounded, broadcast), Deterministic execution mode, SIMD dispatch in tensor module (auto-vectorization), SlotMap and Arena performance optimization.

**Acceptance criteria**: Concurrent programs execute correctly. Replay debugging works for simple concurrent programs. Actor ping-pong works. SIMD-accelerated tensor operations measurably faster than scalar equivalents. Deterministic mode produces identical output for same inputs.

---

### Phase 10: Security, Sandboxing, and Fearless FFI
Untrusted code cannot exceed granted capabilities. FFI is sandboxed. Package supply chain is verified.

**Key deliverables**: Full capability type system with effect-capability alignment, Fearless FFI sandboxing (isolated stack), Sandboxed plugin execution with revocable capabilities, Package signing, verification, supply chain verification, CLI permission flags, Audit logging.

**Acceptance criteria**: Plugin without `--allow-fs` cannot read files. FFI memory corruption does not spread to Omni memory (verified by test). Package verification catches tampered packages. Supply chain verification works.

---

### Phase 11: Interoperability Expansion
C FFI mature, WebAssembly working, Python bindings generating.

**Key deliverables**: C FFI with `omni bindgen` (ownership annotations), WebAssembly backend, Python binding auto-generation (`omni bindgen --python`), ABI stability documentation and versioning, ABI compatibility checks in package manager.

**Acceptance criteria**: C interop tests pass on Linux and macOS. WebAssembly output runs in Node.js and browser. Python bindings work for a simple type hierarchy. ABI compatibility checks catch breaking ABI changes.

---

### Phase 12: Self-Hosting Migration
Omni compiler progressively replaces Rust implementation.

**Key deliverables**: Module-by-module rewrite in Omni, Dual-compiler CI validation, Bootstrap trust verification (Stage 1 == Stage 2), Standard library migration.

**Acceptance criteria**: Omni compiler passes all test suites when compiled by itself. Stage 1 == Stage 2 binary comparison passes. Rust bootstrap retained as fallback only.

---

### Phase 13: Platform Maturity and MLIR Integration (v2.0 Addition)
Stabilize Omni as a production-grade platform. Full MLIR integration for AI acceleration.

**Key deliverables**: Edition system with `omni migrate --edition`, RFC process, Performance regression monitoring in CI, Long-term compatibility policy, MLIR backend (GPU dispatch through tensor API), Hardware abstraction layer for AI accelerators.

**Acceptance criteria**: Edition migration works on real code. CI catches >5% performance regressions. GPU tensor operations produce correct results. MLIR compilation pipeline executes on at least one GPU target.

---

## 20. HELIOS FRAMEWORK (PLATFORM LAYER)

### 20.1 What HELIOS Is

HELIOS is the first major platform built on Omni. An **advanced cognitive platform** for building AI-backed systems with structured knowledge management, capability-based security, multi-modal input processing, and autonomous reasoning support. HELIOS is not an AI model; it is a platform for orchestrating knowledge, reasoning, and execution in a controlled, auditable, and extensible way.

### 20.2 Relationship to Omni

HELIOS depends on and validates: the capability system (§16), async runtime with structured concurrency (§7), the effect system (§6), the tensor module (§11.6), and the plugin system (§16.3). HELIOS development begins in earnest after Phase 7 and scales as Phases 8-10 complete.

### 20.3 Seven Non-Negotiable Requirements

1. **Provenance-preserving knowledge storage** — every entry carries source, timestamp, confidence, author permanently.
2. **Immutable historical record** — updates create versions; deletions are soft; contradictions are explicit records.
3. **Structured confidence model** — every entry has a confidence score; decays for time-sensitive facts; influences retrieval ranking.
4. **Capability-gated access** — all HELIOS capabilities gated by the Omni capability system; no unrestricted access.
5. **Explainable reasoning** — all conclusions traceable to stored knowledge and reasoning steps; chains stored not just outputs.
6. **Layered plugin architecture** — HELIOS is extensible by signed, sandboxed, capability-declared plugins.
7. **Offline-first, local-primary operation** — operates without network access; cloud sync is optional.

### 20.4 HELIOS and the Omni Effect System

HELIOS defines its own effects aligned with its knowledge model:
```omni
effect KnowledgeStore:
    fn query(q: &Query) -> Vec<KnowledgeEntry> / KnowledgeStore
    fn insert(e: KnowledgeEntry) -> KnowledgeId / KnowledgeStore + io
    fn update(id: KnowledgeId, delta: Delta) -> Result<(), ConflictError> / KnowledgeStore + io

effect ReasoningEngine:
    fn infer(context: &Context) -> Vec<Hypothesis> / ReasoningEngine + KnowledgeStore
```

This makes HELIOS functions' dependencies on knowledge storage and reasoning explicit in their type signatures, enabling testing with mock effect handlers and deterministic replay of knowledge operations.

---

## 21. CURRENT STATE & WHAT REMAINS

### 21.1 What Has Been Built

Based on examination of the repository (`github.com/shreyashjagtap157/Helios`):

**Partial (structural work exists, needs completion)**: Project structure and Cargo workspace, bootstrap scaffolding and build scripts, partial stdlib fragments (`std/iter.omni`, logging), LSP and VS Code extension (architecturally correct, premature without a working compiler), mini-compiler demonstrating intent (not a complete pipeline), HELIOS runtime experiments and capability system scaffolding.

**Missing (critical path blockers)**: Complete, production-grade lexer; complete parser with error recovery and INDENT/DEDENT handling; semantic analysis (name resolution, type inference, type checking); MIR representation and borrow checker; IR-to-codegen pipeline producing executable output; package manager with working dependency resolution; verified end-to-end pipeline: source file → binary → execution.

### 21.2 The Honest Assessment

The repository has over-invested in future layers before the core compiler pipeline is stable. The LSP was built before the language server has a stable language to serve. The HELIOS capability system was designed before the Omni type system that will enforce it exists. This demonstrates clear vision of the end state, but requires deliberate refocusing on the foundation.

### 21.3 Recommended Immediate Focus (Priority Order)

**The first and only goal: a vertical slice that works end-to-end.**

1. Complete lexer — tokenize with INDENT/DEDENT. The literal foundation.
2. Complete parser — typed AST from token stream with error recovery and UI tests.
3. Name resolution — bind names to DefIds; report undefined names.
4. Type inference and checking — bidirectional; basic generics; not exhaustive, just correct for a small subset.
5. Minimal MIR and codegen — native code for: variable bindings, arithmetic, if/else, loops, function calls, basic structs. No generics, no traits, no effects.
6. Wire the pipeline — `omni build hello.omni` produces a binary. The binary runs. The binary prints "Hello, World!". Everything else is commentary until this works.

The existing LSP, HELIOS scaffolding, and advanced runtime experiments should be frozen (not deleted) until the core compiler pipeline is complete.

---

## 22. IMPROVEMENTS ADDED IN V2.0 — RESEARCH BASIS

Each improvement is documented with the specific research finding that motivated it.

### 22.1 Algebraic Effect System

**Added**: A first-class effect system where functions declare side effects in their type signatures. User-defined effects via effect handlers. Async, generators, exceptions as effects.

**Research**: Koka (Microsoft Research, actively developed 2025) demonstrates that algebraic effects and handlers let you define advanced control abstractions like async/await as a user library in a typed and composable way. ICFP 2024 proceedings confirm algebraic effect handlers are both theoretically sound and practically implementable.

**Feasibility**: Koka is a production language. The core effect system builds directly on the type inference engine. Effect inference eliminates most annotation burden.

### 22.2 Structured Concurrency as a Hard Constraint

**Added**: `spawn_scope` enforces child tasks cannot outlive creating scope. Unstructured `spawn_global` requires explicit capability.

**Research**: Kotlin enforces parent-child relationships through its Job hierarchy, ensuring that child coroutines cannot outlive their parents, which prevents common resource leaks. Swift's structured concurrency manifesto identified the same problem. Both adopted structured concurrency as the default model.

**Feasibility**: Implementable as a library layer on top of the async executor. `spawn_scope` is equivalent to Kotlin's `coroutineScope` and Swift's `withTaskGroup`. No exotic runtime support required.

### 22.3 Polonius Borrow Checker from Day One

**Added**: Using the Polonius algorithm instead of NLL as the borrow checker.

**Research**: The Rust language design team's 2024 roadmap states: "Non-lexical lifetimes were a big stride forward, but the Polonius project promises to improve the borrow check's precision even more." Polonius eliminates false positives that NLL produces for legitimate programs. Omni avoids shipping the weaker algorithm first.

**Feasibility**: The `polonius-engine` crate (Apache 2.0) is actively maintained. Integration requires adapting Omni's MIR to match Polonius's input format.

### 22.4 Generational References

**Added**: `Gen<T>` generational reference and `Arena<T>` arena allocator as first-class safe alternatives to `Rc<RefCell<T>>` for cyclic data.

**Research**: Vale demonstrates that generational references occupy a sweet spot because they allow objects to be linear yet allow shared mutability in a way that doesn't artificially extend the lifetime of the object. The Rust community identifies Rc<RefCell<T>> as a primary ergonomic pain point for graph-like and cyclic data structures.

**Feasibility**: The `generational-arena` crate demonstrates this is implementable today. Runtime cost is a single integer comparison per dereference.

### 22.5 Linear Types

**Added**: `linear` type modifier requiring types to be used exactly once.

**Research**: Linear types can take care of not just memory safety but resource safety (open files, network connections) generally. The Austral language demonstrates practical linear type usability. Affine types (Rust's ownership) allow discard; linear types do not.

**Feasibility**: Linear type tracking is a straightforward extension of the existing affine type system. The compiler tracks which linear bindings are consumed; scope exit without consuming a linear binding is a compile error.

### 22.6 Field Projections

**Added**: Borrow checker tracks borrows at field granularity within a struct.

**Research**: The Rust language team has been actively working on field projections since 2025, recognizing that the inability to independently borrow struct fields is a significant ergonomic limitation. The Polonius algorithm naturally extends to field-level tracking.

**Feasibility**: Part of the ongoing Polonius work in the Rust project. Implementable from the start since Omni's borrow checker is built fresh with Polonius.

### 22.7 Inout Parameters

**Added**: `inout` parameter syntax for the move-in/move-out pattern.

**Research**: The "move out and move back" pattern produces verbose, unintuitive code. Swift uses `inout` for exactly this purpose. Omni adopts the same solution.

**Feasibility**: Pure syntactic sugar that desugars to existing ownership semantics at the MIR level. Zero runtime cost. No new MIR operations required.

### 22.8 Implied Bounds

**Added**: Struct-level generic bounds are implied in method signatures.

**Research**: The Rust 2024 roadmap explicitly identifies implied bounds as something that "promises to remove a lot of copy-and-pasting of where clauses." This is a documented ergonomic burden in the Rust community.

**Feasibility**: Being implemented in Rust today. The design is well-understood; Omni adopts the final design directly.

### 22.9 Explicit Async Cancellation

**Added**: `CancelToken` and `with_cancel()` for explicit cancellation.

**Research**: Implicit cancellation (future-dropped = cancelled) is one of async Rust's most documented footguns. Swift's `Task.cancel()` and Kotlin's `Job.cancel()` demonstrate explicit cancellation with structured propagation.

**Feasibility**: Implementable as a library abstraction using the effect system. `CancelToken` is a capability-like token passed through the async call chain.

### 22.10 Variadic Generics

**Added**: Variadic generic parameters (`..Ts`) for arbitrary-length type tuples.

**Research**: The Rust 2024 roadmap identifies "variadic tuples and variadic generics" as addressing a common pain point of implementing traits for specific tuple arities.

**Feasibility**: Complex but can be introduced incrementally. Basic form (variadic tuples, variadic function arguments) covers most use cases. Full power is a Phase 7-8 deliverable.

### 22.11 Tensor and SIMD Standard Library

**Added**: `std::tensor` and `std::simd` modules.

**Research**: Mojo builds on MLIR to target CPUs, GPUs, TPUs, ASICs, and other accelerators directly — demonstrating that a systems language can provide portable hardware acceleration as a first-class feature. HELIOS requires native tensor support to avoid the Python/C++ split that has plagued AI tooling.

**Feasibility**: Initial CPU implementation with SIMD dispatch is feasible with existing Rust SIMD libraries as reference. GPU support via MLIR is Phase 13.

### 22.12 Comptime Build Scripts

**Added**: Build logic written in Omni using `comptime` functions, evaluated at build time.

**Research**: Zig's build system is written in Zig itself using comptime, enabling build logic to be expressed in the full language without a separate DSL. Zig is notable for using comptime which lets you run code at compile time instead of at runtime for metaprogramming without a separate macro system.

**Feasibility**: Executable once `comptime` evaluation is implemented (Phase 2-3). The build system calls the comptime evaluator on `build.omni` exactly like any other comptime function.

### 22.13 Fearless FFI Sandboxing

**Added**: Isolated execution context for FFI calls using stack switching.

**Research**: Vale's "Fearless FFI" design uses stack switching to isolate FFI calls, preventing memory corruption in C code from propagating to Vale's managed memory. The core insight: FFI memory corruption is only dangerous if C and Omni share a heap.

**Feasibility**: Stack switching via `sigaltstack` on Unix-like systems and fibers on Windows. Minimal per-FFI-call overhead (two context switches). Initial implementation can use process isolation; native stack switching added later.

### 22.14 Diagnostic Improvements

**Added**: JSON error output, machine-applicable fixes, custom diagnostic attributes for traits, "Did you mean?" suggestions, internationalization support.

**Research**: Research comparing compiler diagnostics across languages finds that Rust and Elm lead. Key characteristics: making it easy to get into the language, explaining errors clearly. Rust 1.78 introduced `#[diagnostic::on_unimplemented]`, allowing library authors to provide custom error messages.

**Feasibility**: All four improvements are straightforward. JSON output requires serializing the diagnostic structure. Machine-applicable fixes are encoded in the same diagnostic structure. Custom diagnostic attributes are simple trait metadata. "Did you mean?" uses Levenshtein edit distance on identifier names.

### 22.15 Replay Debugging

**Added**: Development builds record execution traces for perfect replay.

**Research**: Replay debugging (Mozilla's `rr`, Microsoft's Time Travel Debugging) eliminates Heisenbug problems where bugs disappear when debugging tools are attached. Omni's effect system makes this feasible: all non-deterministic inputs (IO, time, randomness) are declared as effects and can be intercepted and recorded.

**Feasibility**: Since non-deterministic inputs are declared as effects, the effect handler infrastructure intercepts and records them. Replay installs a handler returning recorded results instead of real system calls. Storage cost is O(size of IO operations).

---

## APPENDIX A: DESIGN DECISION REGISTRY (v2.0 — 40 Decisions)

| ID | Decision | Chosen | Rationale |
|---|---|---|---|
| D001 | Language category | Hybrid multi-level platform | Spans systems/application/AI domains |
| D002 | Primary priority | Safety + Performance | Cannot be sacrificed; productivity secondary |
| D003 | Abstraction levels | Multi-level with explicit transitions | All audiences without chaos |
| D004 | Sacred principle | Deterministic correctness + no UB + effects visible in types | Root cause of most expensive bugs |
| D005 | Primary audience | Advanced developers + Framework authors | Beginner access via modes, not primary design |
| D006 | Opinionation level | Moderately opinionated | Prevents chaos without killing flexibility |
| D007 | Typing model | Static + effect annotations; bidirectional inference | Safety + expressiveness |
| D008 | Null handling | Option types; null in restricted zones only | Eliminates NPEs in safe code |
| D009 | Error model | Result types + error sets + typed context chains | Explicit, traceable, composable |
| D010 | Memory core | Ownership-based + generational refs for cyclic data | Deterministic, GC-free, safe by construction |
| D011 | GC compatibility | Optional layer in higher-level modes | Some domains benefit without contaminating core |
| D012 | Unsafe code | Restricted to blocks/functions + Fearless FFI | Necessary for systems; must be auditable |
| D013 | Concurrency model | Hybrid: threads + structured async + actors + channels | Different workloads need different models |
| D014 | Shared mutable state | Only in unsafe mode | Prevents data races statically |
| D015 | Execution default | AOT native | Best performance; other modes explicit |
| D016 | Determinism | Core deterministic; nondeterminism is explicit opt-in | Reproducibility and debuggability |
| D017 | Block syntax | Indentation default, braces in advanced modes | Clean default; power user escape hatch |
| D018 | Semicolons | Newline-first; semicolons disambiguate only | Cleaner code |
| D019 | Expression orientation | Expression-oriented by default | More compositional and concise |
| D020 | Operator overloading | Via traits; standard set; `Try` trait extensible | Type-safe; no unreadable custom operators |
| D021 | Macro system | Two-tier: declarative + sandboxed procedural + comptime codegen | Power without chaos |
| D022 | Module visibility | Layered: private/mod/pkg/pub/capability/friend | Fine-grained encapsulation |
| D023 | Dependency resolution | PubGrub + lockfiles + API compatibility checks | Correct, good errors, reproducible |
| D024 | Package distribution | Multi-target + capability declarations in manifest | Maximum flexibility, security transparency |
| D025 | Language versioning | Edition-based | Controlled evolution |
| D026 | Standard library | Layered + tensor module for AI | Scales from embedded to AI platform |
| D027 | Bootstrap language | Rust | Philosophy alignment; mature tooling |
| D028 | Self-hosting strategy | Multi-stage bootstrap pipeline | Safe, verified, controlled migration |
| D029 | MVP scope | Systems-core first (vertical slice) | Ship something real |
| D030 | Security model | Layered: language + runtime + tooling + ecosystem | Cannot be bolted on later |
| D031 | Effect system | Algebraic effects (built-in + user-defined) | Unifies async, exceptions, generators |
| D032 | Structured concurrency | Hard constraint: children cannot outlive scope | Eliminates resource leak class of bugs |
| D033 | Borrow checker algorithm | Polonius from day one | More precise; no false positives from NLL |
| D034 | Generational references | First-class in stdlib | Safe, ergonomic cyclic data |
| D035 | Linear types | Compiler-enforced `linear` modifier | Resource safety beyond affine types |
| D036 | Field projections | Supported in borrow checker | Eliminates struct-splitting workarounds |
| D037 | Inout parameters | Syntactic sugar for move-in/move-out | Ergonomic ownership transfer |
| D038 | Implied bounds | Struct bounds implied in method signatures | Eliminates where-clause copy-paste |
| D039 | Async cancellation | Explicit CancelToken | Prevents implicit-drop cancellation bugs |
| D040 | Tensor/SIMD stdlib + MLIR | Built-in with MLIR for GPU (future) | HELIOS dependency; AI-first platform |

---

## APPENDIX B: TECHNOLOGY STACK (v2.0)

| Component | Technology | Rationale |
|---|---|---|
| Bootstrap language | Rust (2024 edition) | Philosophy alignment, safety, tooling |
| Parsing | Custom recursive descent + Pratt | Control, error recovery, parallel |
| CST (for formatter) | Rowan | Used by rust-analyzer, lossless |
| Union-find (type inference) | `ena` crate | Used by rustc, proven |
| Borrow checker | `polonius-engine` crate + Omni MIR adapter | More precise than NLL; Apache 2.0 |
| Codegen (development) | Cranelift | Fast, Rust-native, correct |
| Codegen (release) | LLVM via `inkwell` | Maximum optimization |
| Codegen (AI targets, Phase 13) | MLIR | GPU and hardware portability |
| SIMD dispatch | Std SIMD intrinsics via Cranelift/LLVM | Auto-vectorization foundation |
| LSP framework | `tower-lsp` | Mature, async |
| CLI argument parsing | `clap` (derive) | Ergonomic |
| Test runner | `cargo nextest` | Parallel, fast |
| Dependency resolution | PubGrub algorithm | Correct, complete, good errors |
| Manifest format | TOML + `serde` | Ecosystem standard |
| Async executor | Custom work-stealing (structured) | Structured concurrency enforcement |
| Fuzzing | `cargo-fuzz` / libfuzzer | Industry standard |
| Security audit | `cargo-audit` | RUSTSEC advisory database |
| Incremental compilation | Salsa-inspired query model | Fast LSP + incremental builds |
| Replay debugging | Trace recording via effect interceptors | Deterministic debugging |
| Generational references | `generational-arena` crate as reference | Proven approach |
| Fearless FFI | `sigaltstack` / Windows fibers | Stack isolation for FFI safety |
| API compatibility | `cargo-semver-checks` as reference | Supply chain and ABI safety |

---

# 29. Adversarial semantic-closure audit: `OMNI-EVAL` → `OMNI-WIRE` → `OMNI-DIST`

## 29.1 Audit status and finding

This section records the final distributed-state semantic-closure audit of the Edition 1 candidate. It attacks the intersection of suspended computation, algebraic-effect continuations, persistence, serialization, hydration, node migration, and capability re-authorization.

The central finding is that an Omni continuation MUST NOT be treated as a serializable copy of a machine stack. A live continuation may contain implementation-specific stack addresses, borrow lifetimes, provenance-bearing references, capability values, executor state, target-specific handles, and other entities whose meaning is local to one execution environment.

The closure requirement is therefore:

> **A distributed continuation is a canonical logical computation state, not a serialized native stack. Wire encoding SHALL contain only values and explicitly transferable handles whose contracts are valid in the destination environment. Hydration SHALL construct a fresh local execution state and SHALL re-authorize local authority before resumption.**

## 29.2 Continuation classes

Edition 1 distinguishes three continuation classes:

1. **ephemeral continuation** — valid only within its current task, stack, region, executor, and capability environment;
2. **persistent continuation** — checkpointable within a declared persistence profile, but still local to the node unless explicitly converted to a distributed continuation;
3. **distributed continuation** — a canonical, transferable representation admitted by `OMNI-WIRE` and `OMNI-DIST`.

A continuation SHALL never change class implicitly.

### Required normative amendments (`AUDIT-DIST-0001` through `AUDIT-DIST-0012`)

| Audit rule | Required amendment |
|---|---|
| `AUDIT-DIST-0001` | An ephemeral continuation MUST NOT be serialized, persisted, or transferred across a process, node, or trust domain. |
| `AUDIT-DIST-0002` | A persistent or distributed continuation SHALL be represented as an explicitly typed logical state machine checkpoint; native stack addresses, frame pointers, executor-internal pointers, borrow tokens, and raw capability values SHALL NOT occur in its wire representation. |
| `AUDIT-DIST-0003` | A type containing a non-`'static` borrow, raw pointer, region reference, task-local handle, non-transferable capability, open FFI resource, or target-local device reference SHALL NOT be directly serializable as continuation state. |
| `AUDIT-DIST-0004` | A continuation containing transferable state SHALL encode the canonical type identity, edition/profile identity, schema identity, program/package identity, and checkpoint version required to interpret that state. |
| `AUDIT-DIST-0005` | Hydration SHALL allocate fresh local storage and recreate references through local construction rules; it SHALL never restore a source-node pointer, allocation identity, provenance token, or stack address as a native executable reference. |
| `AUDIT-DIST-0006` | Local capabilities SHALL NOT cross a distributed serialization boundary as authority. A distributed continuation SHALL contain only explicitly transferable capability handles whose provider contract permits transfer, or capability requirements that MUST be re-authorized at hydration. |
| `AUDIT-DIST-0007` | Re-authorization SHALL occur before the continuation becomes runnable. Failure to obtain required authority produces a typed hydration failure or isolation outcome and SHALL NOT produce a partially runnable continuation. |
| `AUDIT-DIST-0008` | A continuation's effect row SHALL be interpreted as a requirement on the destination handler environment; it SHALL NOT by itself grant the corresponding authority. |
| `AUDIT-DIST-0009` | Hydration SHALL validate schema/version compatibility, capability requirements, profile compatibility, target-independent value invariants, and checkpoint integrity before publishing the resumed task. |
| `AUDIT-DIST-0010` | Distributed resumption SHALL be idempotence-aware. A checkpoint SHALL carry a unique logical execution identity and monotonic resume/commit token sufficient for the selected delivery semantics. |
| `AUDIT-DIST-0011` | If a distributed continuation resumes after a source-side capability revocation, expiration, or lease loss, the destination provider MUST re-evaluate authorization; source authorization SHALL never be treated as permanent. |
| `AUDIT-DIST-0012` | No distributed serialization operation may expose implementation-private memory representation, padding, pointer provenance, allocator identity, or target-native object layout unless the object explicitly opts into a standardized wire representation. |

## 29.3 Serialization barrier

The serialization checker computes a `WireClosure` for every root value:

`WireClosure(T) = { reachable fields, type identities, ownership modes, lifetimes, effects, capabilities, external handles, schema dependencies }`

A value is wire-safe only when every member of its closure is admitted by the active wire schema and distribution profile.

The closure algorithm MUST terminate and MUST be conservative. Failure to prove wire safety is a static rejection, not an implicit byte-copy fallback.

For continuation values, the wire closure SHALL additionally include every live variable, active handler, pending effect operation, cancellation state, cleanup obligation, and logical program-counter state required for correct resumption.

## 29.4 Hydration protocol

Hydration is an explicit state transition:

`SerializedCheckpoint → VerifiedCheckpoint → RehydratedState → AuthorizedState → Runnable`

No transition may skip verification or authority re-establishment.

The destination runtime MUST perform the following sequence:

1. verify wire integrity and schema identity;
2. verify package/program/edition compatibility;
3. validate all serialized values and refinement invariants;
4. reconstruct owned storage and local managed references;
5. resolve required transferable handles;
6. request fresh capability authorization for every local authority requirement;
7. reconstruct effect handlers and executor bindings;
8. install cancellation/supervision state;
9. publish the task as runnable only after all prior steps succeed.

Failure at any step produces a non-runnable checkpoint state with an explicit failure classification.

## 29.5 Borrow and lifetime rule

Distributed state has no direct access to source-node lexical lifetimes.

Therefore:

- a borrow crossing a distribution boundary MUST be converted into owned or explicitly durable data;
- region-backed values MUST be materialized into wire-safe representations;
- a `Gen<T>`, pointer, reference, or arena index MUST NOT be interpreted on another node as preserving source allocation identity;
- any local cache/reference reconstructed at the destination is a new object with new provenance.

A lifetime parameter is therefore a static construction constraint for a wire representation, not a runtime serializable token.

## 29.6 Distributed effects and authority

A distributed effect declaration may state that a computation requires, for example, `io.network`, `persistent`, or `remote.invoke` behavior. This declares an observation capability requirement, not possession of authority.

A destination handler MUST separately establish the capabilities required to implement that effect.

Consequently:

`effect requirement ≠ capability possession ≠ capability transfer`

No type inference, effect inference, or deserialization step may collapse these three concepts.

## 29.7 Exactly-once and at-least-once interaction

Distributed continuation resumption interacts with message delivery and persistent commit semantics.

The standard therefore distinguishes:

- **at-most-once resume** — duplicate resume attempts are rejected after the first accepted execution identity;
- **at-least-once resume** — duplicate delivery MAY cause duplicate execution, requiring application-level idempotence or deduplication;
- **effect-committed resume** — completion is committed atomically with the selected durable effect protocol.

A continuation protocol MUST name its selected delivery semantics. The default distributed profile SHALL NOT claim exactly-once execution unless the implementation proves an end-to-end commit protocol sufficient for the claimed scope.

## 29.8 Cross-target determinism

A distributed continuation MAY resume on a different target only when its schema and semantic profile guarantee target-independent meaning for every serialized value and operation.

Target-local data such as pointer width, endianness, native handles, device IDs, CPU feature state, floating environment, and host path syntax SHALL NOT be embedded as implicit semantic state.

Target-dependent state MUST be represented by an explicit portable schema or rejected for migration.

## 29.9 Conformance corpus additions

The Edition 1 corpus SHALL include at least:

1. serialize a continuation containing `&mut LocalData` — static rejection;
2. serialize a continuation containing a local filesystem capability — static rejection;
3. serialize a continuation containing a declared transferable cryptographic handle — accepted only when the provider permits transfer;
4. hydrate a continuation whose required capability is unavailable on Node B — deterministic hydration failure, never runnable;
5. hydrate after source-node capability revocation — destination re-authorization rejects the stale authority;
6. hydrate on a different endian target using a canonical wire schema — semantically identical state;
7. attempt to hydrate an incompatible checkpoint schema — deterministic compatibility failure;
8. duplicate delivery of an at-most-once continuation — second resume rejected;
9. duplicate delivery of an at-least-once continuation — allowed only under the declared idempotence contract;
10. attempt to deserialize raw pointer/provenance information as executable state — static/schema rejection.

## 29.10 Closure criterion

The distributed-state seam is semantically closed only when the implementation can mechanically prove:

1. no distributed continuation contains a source-node memory reference that can become a dangling or forged native reference;
2. no local capability becomes transferable authority merely because it was captured in a continuation;
3. hydration cannot produce a runnable state before value validation, handler reconstruction, and destination authorization complete;
4. cross-target execution preserves the semantic meaning defined by the selected wire schema and profile;
5. checkpoint identity and resume semantics are explicit and testable.

---

# 30. Semantic-closure boundary: core language versus certification

## 30.1 Closure result

With the completion of the distributed-state audit, the principal semantic seams identified during the adversarial review have explicit closure rules:

| Seam | Closure mechanism |
|---|---|
| `OMNI-OWN` → `OMNI-ERROR` → `OMNI-CONC` | Cancellation is distinct from unwind; cleanup masking and fault aggregation are explicit. |
| `OMNI-UNSAFE` → `OMNI-OPT` | Unsafe assumptions are explicit MIR dependency tokens with provenance and invalidation events. |
| `OMNI-EFFECTS` → capabilities | Effect requirements and dynamic authority are tracked separately; continuation capture cannot duplicate or widen authority. |
| `OMNI-TYPES` → `OMNI-OWN` → `OMNI-EFFECTS` | Continuation lifetimes, HRTB/effect scoping, and runtime type identity are statically constrained. |
| `OMNI-BUILD` → `OMNI-CONST` → `OMNI-REPRO` | Comptime dependency tracing and deterministic execution bind cache identity to observed inputs. |
| `OMNI-PKG` → `OMNI-UPDATE` | Resolution is snapshot-bound, namespace-bound, and revocation-aware. |
| `OMNI-FFI` → `OMNI-RUNTIME` | Foreign execution is an explicit runtime state with isolated unwind and safepoint semantics. |
| `OMNI-EVAL` → `OMNI-WIRE` → `OMNI-DIST` | Distributed continuations are logical checkpoints with validation and re-authorization, never serialized native stacks. |

## 30.2 What semantic closure means

Semantic closure means that an independently implemented conforming toolchain does not need to invent observable language behavior at any audited seam.

It does **not** mean that an implementation is automatically correct, optimized, certified, secure against all implementation defects, or ready for production.

Those are qualification properties defined separately below.

## 30.3 Prohibited declaration

No release document, compiler, marketing material, or conformance report SHALL claim that semantic closure alone proves compiler correctness, absence of implementation bugs, or security of foreign dependencies.

---

# 31. Final implementation qualification gates

Semantic closure is a prerequisite for implementation qualification, not a substitute for it.

A complete Edition 1 implementation qualification program SHALL include:

1. **Reference semantics conformance** — executable abstract-machine/reference-model agreement for the core language.
2. **Parser conformance** — canonical grammar corpus, ambiguity rejection, macro token-tree coverage, and exact source-span behavior.
3. **Static-semantics conformance** — type, ownership, lifetime, effect, capability, refinement, and inference suites including negative tests.
4. **Dynamic-semantics conformance** — evaluation, fault, unwind, cancellation, concurrency, and memory-model litmus suites.
5. **IR conformance** — verifier checks, canonicalization rules, legal-lowering proofs, and translation-validation coverage.
6. **Optimization qualification** — transformation-specific legality tests, stale-assumption tests, differential tests, and verified/high-assurance mode enforcement.
7. **Build/reproducibility qualification** — clean-room independent builds, cache invalidation tests, source/environment dependency tracing, and artifact hash comparison.
8. **Supply-chain qualification** — malicious registry/snapshot tests, revocation handling, namespace confusion tests, and lockfile downgrade tests.
9. **FFI qualification** — foreign exception, `longjmp`, signal, stack-switch, managed-memory, safepoint, ABI, and invalid-foreign-state tests.
10. **Distributed/persistence qualification** — schema migration, hydration, capability re-authorization, crash/restart, duplicate-delivery, and cross-target tests.
11. **Security qualification** — capability non-forgeability, compartment isolation, unsafe-boundary validation, hardened profile behavior, and fuzzing.
12. **Bootstrap qualification** — reproducible Stage-0 bootstrap, compiler self-build, independent compiler comparison, and DDC evidence.

A compiler claiming complete Edition 1 implementation conformance MUST publish machine-readable evidence for each applicable gate.

---

# 32. Omni AI-compilation contract: post-closure engineering model

## 32.1 Purpose

The language is explicitly designed so that AI systems can participate in programming without receiving privileged semantic exemptions.

The compiler remains the source of semantic authority. AI systems are clients of the same source, diagnostics, proof obligations, IR, and conformance interfaces available to human developers.

## 32.2 Canonical AI loop

The required production workflow is:

`intent/specification → source synthesis → parse/type/ownership/effect checking → structured diagnostics → proof obligations → repair → MIR validation → optimization validation → artifact verification`

The loop SHALL preserve source provenance and rule IDs so every repair can be traced to the violated semantic obligation.

## 32.3 Machine-readable feedback

AI-facing diagnostics SHALL expose, where applicable:

- stable rule ID;
- diagnostic code;
- source span;
- static environment summary;
- unsatisfied type/ownership/effect/capability obligation;
- relevant assumption/invalidator IDs;
- suggested repairs;
- proof or test evidence required for acceptance;
- whether the failure is source-invalid, profile-unsupported, resource-dependent, or implementation-defective.

The compiler SHALL NOT emit a weaker semantic contract merely because the caller is an AI system.

## 32.4 AI-generated unsafe code

AI-generated unsafe code is subject to exactly the same obligation records and validator requirements as human-written unsafe code.

The presence of a generated-code provenance tag MUST NOT suppress or weaken unsafe diagnostics.

A tool MAY require an explicit human approval policy before emitting or enabling unchecked unsafe operations, but this is an external governance policy and not a semantic relaxation.

## 32.5 Automated repair safety

An automated repair is semantically admissible only when the resulting artifact passes the same static and dynamic validation pipeline as manually authored source.

Textual similarity, model confidence, prior examples, or probabilistic ranking are not conformance evidence.

## 32.6 AI proof-carrying build artifacts

A high-assurance build MAY attach machine-readable proof/validation certificates containing:

- source and manifest digests;
- compiler and optimizer identities;
- IR fingerprints;
- rule IDs exercised;
- transformation-validation summaries;
- unsafe-obligation summaries;
- target/profile identities;
- reproducibility evidence;
- test/model evidence references.

Such certificates are evidence artifacts and SHALL NOT redefine language semantics.

---

# 33. Final status classification

The following statuses are distinct and SHALL NOT be conflated:

| Status | Meaning |
|---|---|
| **Architecture-complete** | Architectural intent and tradeoffs are documented. |
| **Definition-complete** | Every admitted construct has normative static/dynamic meaning and classification. |
| **Semantic-closure-complete** | The audited cross-module semantic seams have explicit mechanical closure rules. |
| **Implementation-conformant** | A compiler has produced sufficient evidence for the declared conformance claim. |
| **Implementation-certified** | An independent qualification process has accepted the implementation against the required evidence package. |
| **Ratified** | The standards governance process has approved the release as normative. |
| **Production-qualified** | Platform, distribution, security, performance, operational, and ecosystem gates have also passed for the claimed product profile. |

The Edition 1 candidate MAY claim semantic-closure-complete only after all required adversarial audit amendments are integrated into the normative suite and their corresponding conformance obligations are registered.

It SHALL NOT claim ratified, implementation-certified, or production-qualified status solely from this document.

---

# 34. Final engineering conclusion

Omni's design objective is not merely to eliminate undefined behavior or to provide a safer systems-language syntax.

Its stronger objective is:

> **Every observable program property that matters to correctness, authority, resource safety, reproducibility, concurrency, optimization legality, distribution, or AI-assisted transformation SHALL have an explicit semantic owner, a mechanically checkable rule, and conformance evidence.**

The final distributed-state audit closes the last identified semantic seam in the Edition 1 architecture: native execution state cannot silently become transferable distributed state.

The resulting standard therefore treats memory, authority, computation state, and build identity as separate semantic domains with explicit conversion barriers between them.

The remaining work after semantic closure is qualification and implementation: executable models, independent implementations, conformance corpora, proof obligations, bootstrap evidence, target certification, standard-library completion, operational hardening, and governance ratification.

Those activities are essential, but they are different engineering problems from discovering missing language semantics.

---

# PART III — Omni Implementation Master Plan

## 35. Implementation phase freeze and purpose

### 35.1 Status

**Architecture/Specification phase:** COMPLETE.

**Current language status:** Semantic-closure complete in architecture; implementation qualification pending.

**Implementation target:** Build the first complete, self-hosting, independently auditable Omni Edition 1 toolchain from an empty repository against the frozen normative suite.

This implementation plan is deliberately non-normative. The normative Edition 1 suite remains the semantic authority. Implementation code, compiler behavior, historical experiments, bootstrap shortcuts, optimizer behavior, and tests MUST NOT silently redefine language semantics.

### 35.2 What is being built

The implementation is a complete Omni computing toolchain, not merely a compiler executable. It consists of:

1. a Rust bootstrap compiler and Stage-0 toolchain;
2. a canonical lexer and parser;
3. lossless source/CST infrastructure and diagnostics;
4. name resolution and static semantic analysis;
5. type, lifetime, ownership, refinement, effect, and capability checking;
6. the canonical semantic IR and MIR verifier;
7. the abstract-machine/reference interpreter;
8. safe and unsafe execution semantics;
9. native code generation and target support;
10. runtime, allocation, concurrency, synchronization, and async infrastructure;
11. standard-library contracts and implementation;
12. package/build/reproducibility infrastructure;
13. optimization and translation validation;
14. FFI, ABI, plugin, and sandbox facilities;
15. persistent/distributed serialization and hydration support;
16. LSP, formatter, refactoring, debugger, trace, replay, and machine-readable tooling;
17. bootstrap/self-hosting infrastructure;
18. conformance, qualification, security, reproducibility, and certification evidence.

### 35.3 Implementation principle

Every completed milestone SHALL leave the repository as one coherent, buildable, tested, internally consistent system. Feature work MUST NOT be treated as complete merely because the feature itself compiles.

A milestone is complete only when its implementation, interfaces, diagnostics, tests, documentation, generated artifacts, dependency graph, and affected existing behavior are integrated and qualified together.

---

## 36. Implementation versioning model

### 36.1 Separate implementation version from language edition

The language remains **Omni Edition 1** while implementation development advances independently.

Implementation release identifiers use `OMNI-IMP-X.Y.Z` and MUST NOT be interpreted as language-edition numbers.

- `X` = implementation maturity stage.
- `Y` = coherent capability milestone within that stage.
- `Z` = corrective/integration release that does not intentionally expand the declared milestone scope.

The implementation MUST never infer semantic permission from the implementation version.

### 36.2 Recommended implementation ladder

| Implementation line | Scope | Completion meaning |
|---|---|---|
| `OMNI-IMP-0.0.x` | Workspace and bootstrap foundation | Empty-repository engineering system is reproducibly buildable and testable. |
| `OMNI-IMP-0.1.x` | Source frontend | Canonical source representation, lexer, parser, CST/AST, and diagnostics work together. |
| `OMNI-IMP-0.2.x` | Static core | Names, types, basic inference, literals, expressions, and initial ownership checking are integrated. |
| `OMNI-IMP-0.3.x` | Semantic IR | Canonical HIR/MIR, verifier, obligations, source mapping, and reference lowering exist. |
| `OMNI-IMP-0.4.x` | First executable vertical slice | Source can become a verified native executable through the complete pipeline for a deliberately small language subset. |
| `OMNI-IMP-0.5.x` | Core language completion | Edition 1 core static/dynamic semantics are implemented against the specification. |
| `OMNI-IMP-0.6.x` | Runtime/concurrency | Ownership runtime support, async, structured concurrency, cancellation, synchronization, and memory semantics are qualified. |
| `OMNI-IMP-0.7.x` | Unsafe/optimization | Unsafe obligations, provenance, canonical assumptions, optimization legality, and translation validation are operational. |
| `OMNI-IMP-0.8.x` | Distribution/toolchain | Build, package, cache, reproducibility, registry/update verification, tooling, and release artifacts are qualified. |
| `OMNI-IMP-0.9.x` | Platform/interoperability | FFI, ABI, target supplements, plugins, persistence, and distributed hydration are qualified. |
| `OMNI-IMP-1.0.0-rc.x` | Release candidate | Full Edition 1 implementation qualification and certification evidence is under final audit. |
| `OMNI-IMP-1.0.0` | Initial qualified implementation | All declared Edition 1 product claims are met and independently evidenced. |
| `OMNI-IMP-1.0.x` | Stable maintenance | Defect/security/compatibility fixes without semantic breaking changes. |
| `OMNI-IMP-1.1.x+` | Post-1.0 evolution | Additive implementation improvements and later-language/profile work under separate compatibility rules. |

### 36.3 Development commits

Commits SHOULD use the implementation milestone identifier and a semantic area, for example:

`OMNI-IMP-0.2.3: implement bidirectional expression typing`

A commit MUST represent a coherent, buildable state. Half-integrated experiments, knowingly broken intermediate states, and disconnected feature branches MUST NOT be presented as completed milestones.

---

## 37. Repository architecture from an empty workspace

### 37.1 Initial repository shape

The recommended repository MUST begin as a Cargo workspace with explicit ownership of each architectural layer. A starting structure is:

```text
omni/
├── Cargo.toml
├── rust-toolchain.toml
├── LICENSE*
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CHANGELOG.md
├── ROADMAP.md
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── implementation/
│   └── qualification/
├── spec/
│   ├── manifest/
│   ├── schemas/
│   ├── models/
│   ├── grammar/
│   └── corpus/
├── compiler/
│   ├── omni-source/
│   ├── omni-lex/
│   ├── omni-parse/
│   ├── omni-syntax/
│   ├── omni-names/
│   ├── omni-types/
│   ├── omni-own/
│   ├── omni-effects/
│   ├── omni-const/
│   ├── omni-hir/
│   ├── omni-mir/
│   ├── omni-verify/
│   ├── omni-machine/
│   ├── omni-opt/
│   ├── omni-codegen/
│   └── omni-driver/
├── runtime/
│   ├── omni-runtime/
│   ├── omni-alloc/
│   ├── omni-async/
│   ├── omni-conc/
│   └── omni-platform/
├── std/
│   ├── core/
│   ├── alloc/
│   └── std/
├── tools/
│   ├── omni-cli/
│   ├── omni-fmt/
│   ├── omni-fix/
│   ├── omni-lsp/
│   ├── omni-debug/
│   ├── omni-bindgen/
│   └── omni-test/
├── package/
│   ├── omni-pkg/
│   ├── omni-build/
│   ├── omni-registry/
│   └── omni-update/
├── bootstrap/
│   ├── stage0/
│   ├── stage1/
│   └── stage2/
├── tests/
│   ├── lexer/
│   ├── parser/
│   ├── ui/
│   ├── semantics/
│   ├── mir/
│   ├── machine/
│   ├── runtime/
│   ├── optimization/
│   ├── build/
│   ├── ffi/
│   ├── distributed/
│   ├── security/
│   ├── reproducibility/
│   └── bootstrap/
└── scripts/
```

The exact crate decomposition MAY change during implementation, but semantic ownership boundaries MUST remain explicit.

### 37.2 Source-of-truth separation

The repository SHALL distinguish:

- normative specification artifacts;
- machine-readable normative models;
- implementation code;
- generated code;
- conformance tests;
- implementation-specific tests;
- qualification evidence;
- historical experiments.

Historical experiments MUST NOT remain connected to active semantic paths merely because they compile.

---

## 38. Phase 0 — Workspace and bootstrap foundation

### Version target

**`OMNI-IMP-0.0.1` through `OMNI-IMP-0.0.9`**

### What it is

The minimum engineering substrate required to develop Omni reproducibly without yet claiming meaningful language-feature coverage.

### Why it exists

A compiler project without deterministic workspace rules, CI, artifact provenance, test infrastructure, and specification synchronization will accumulate accidental behavior before the first semantic feature is trustworthy.

### Required components

- pinned Rust toolchain;
- Cargo workspace;
- compiler/runtime/tooling crate boundaries;
- formatting and lint policy;
- unit, integration, UI, property, fuzz, differential, and golden-test harnesses;
- deterministic test runner;
- specification manifest ingestion;
- rule-ID registry ingestion;
- generated schema validation;
- CI for build/test/lint/docs/security/audit;
- reproducible developer bootstrap;
- release artifact naming and hashing;
- architectural decision records;
- contribution and security policy;
- fail-closed behavior for missing specification artifacts.

### Tests

`cargo build --workspace`, `cargo test --workspace`, lint/format checks, clean-machine bootstrap, offline reproducibility checks, schema validation, and specification-manifest validation MUST all pass.

### Qualification gate

No implementation feature work is admitted to the protected mainline until two clean builds from a fresh checkout produce equivalent declared bootstrap artifacts.

---

## 39. Phase 1 — Canonical source, lexer, CST, parser, and diagnostics

### Version target

**`OMNI-IMP-0.1.0` through `OMNI-IMP-0.1.9`**

### What it is

The complete source-front-end foundation implementing `OMNI-SOURCE`, `OMNI-LEX`, `OMNI-GRAMMAR`, and initial diagnostic infrastructure.

### Build requirements

The implementation MUST consume the pinned Unicode data/version, normative grammar, source rules, and diagnostic schemas. Parser recovery machinery MAY be richer than release parsing but recovery artifacts MUST remain distinguishable from release ASTs.

### Deliverables

- UTF-8 source loader and normalized-byte mapping;
- Unicode/security checks;
- maximal-munch lexer;
- lossless CST;
- canonical AST;
- macro/token-tree substrate;
- complete precedence/associativity implementation;
- parser ambiguity detection;
- stable source spans;
- diagnostic engine with stable IDs;
- JSON/SARIF diagnostic output;
- machine-applicable fix representation;
- `omni parse`, `omni check` command foundations;
- formatter round-trip engine.

### Lookouts

No newline-sensitive statement meaning; no filesystem ordering dependence; no Unicode-version drift; no recovery node leaking into release translation; no diagnostic code reuse; no AST ambiguity hidden by parser heuristics.

### Tests

Complete lexical corpus; Unicode adversarial corpus; parser golden tests; malformed-input UI tests; ambiguity tests; macro token-tree tests; formatter idempotence; fuzzing; differential parser comparison where available.

### Qualification

Every grammar production and lexical rule MUST map to implementation tests. Parser conformance blocks progression if any normative grammar construct has no executable coverage.

---

## 40. Phase 2 — Names, types, inference, literals, and static core

### Version target

**`OMNI-IMP-0.2.0` through `OMNI-IMP-0.2.9`**

### What it is

The static-semantic engine implementing name resolution, type identity, generics, inference, numeric typing, coercions, traits, object safety, and refinements.

### Why it exists

The parser proves only that a program is syntactically meaningful. Omni's semantic contract begins here: every accepted construct must have one deterministic static interpretation.

### Build requirements

- immutable interned identities;
- deterministic `DefId`/type/effect IDs;
- constraint solver with termination safeguards;
- canonical substitution representation;
- stable generic environment representation;
- diagnostic obligation graph;
- normalization/equality engine;
- specification-linked negative tests.

### Deliverables

- module/import/name resolution;
- bidirectional inference;
- generic constraints;
- trait/coherence solver;
- associated item resolution;
- object-safety checking;
- numeric literal typing and defaults;
- explicit coercions;
- refinement checking;
- stable canonical type representation;
- compile-time error recovery that never affects accepted semantics.

### Tests

Positive/negative type corpus; coherence conflicts; inference ambiguity; higher-ranked lifetime skeletons; numeric overflow/conversion tests; object-safety tests; deterministic repeated-solver tests.

### Qualification

The solver MUST have deterministic results across process order, hash seed, thread scheduling, and equivalent input enumeration. Any ICE or unconstrained solver fallback is a release blocker.

---

## 41. Phase 3 — Ownership, lifetimes, effects, capabilities, and semantic obligations

### Version target

**`OMNI-IMP-0.3.0` through `OMNI-IMP-0.3.9`**

### What it is

The static safety engine that combines type, ownership, lifetime, effect, capability, refinement, continuation, and unsafe obligations.

### Why it exists

This is the central semantic core. It must establish the invariants that later MIR and runtime components are allowed to assume.

### Deliverables

- initialization-state lattice;
- move/partial-move analysis;
- borrow and reborrow checking;
- lifetime region inference;
- pinning and self-reference checking;
- effect-row inference and normalization;
- effect handler checking;
- capability environment checking;
- closure capture environment checking;
- continuation lifetime representation;
- HRTB scope rules;
- runtime type-identity eligibility;
- unsafe obligation generation;
- proof-obligation graph and machine-readable diagnostics.

### Tests

Use the entire adversarial audit corpus already defined by the specification, including async-drop cancellation, stale unsafe assumptions, capability/multishot continuations, HRTB/effect leakage, closure capability escape, and dynamic type fingerprint tests.

### Qualification

No later compiler phase may receive a semantically accepted MIR candidate without a successful static obligation check. “Best effort” semantic acceptance is prohibited.

---

## 42. Phase 4 — Canonical HIR/MIR and verifier

### Version target

**`OMNI-IMP-0.4.0` through `OMNI-IMP-0.4.9`**

### What it is

The canonical intermediate representation layer and the mechanical verifier that becomes the central contract between front end, optimizer, runtime semantics, code generation, and validation.

### Why it exists

The MIR is where abstract language law becomes executable compiler state. It must make ownership, effects, capability authority, memory provenance, cleanup, assumptions, and control flow explicit rather than implicit in compiler implementation code.

### Required IR properties

Every semantically significant operation MUST have a stable representation. Assumptions, invalidators, ownership transitions, capability transitions, cleanup edges, cancellation points, synchronization events, provenance transitions, and fault paths MUST be represented explicitly enough for verification.

### Deliverables

- canonical HIR;
- canonical MIR;
- SSA/dataflow representation where applicable;
- explicit cleanup/unwind blocks;
- effect/capability tokens;
- unsafe `assume.create/use/invalidate` model;
- allocation/provenance representation;
- continuation/checkpoint representation;
- source/rule provenance;
- MIR verifier;
- canonical serialization/hash;
- verifier diagnostics.

### Tests

MIR golden tests; canonicalization idempotence; verifier mutation tests; invalid-MIR corpus; assumption lifetime tests; ownership-state transition tests; capability token tests; cleanup-path coverage.

### Qualification

Two separately constructed front-end inputs that are semantically equivalent under the specified canonicalization rules MUST converge to equivalent canonical MIR observations. A verifier bypass is a release blocker.

---

## 43. Phase 5 — Reference machine and first executable vertical slice

### Version target

**`OMNI-IMP-0.5.0`** as the first executable milestone, but the implementation versioning ladder MAY temporarily designate this as `0.4.x` if the team reaches it earlier.

### What it is

The first complete end-to-end execution path: valid source → semantic checking → MIR → reference machine/native lowering → executable → defined result.

### Initial supported subset

Only a deliberately small subset is required initially: bindings, literals, arithmetic, conditions, loops, functions, basic aggregates, deterministic return values, and basic I/O through a declared standard capability/effect path.

Unsupported Edition 1 constructs MUST fail explicitly rather than partially compile.

### Deliverables

- executable abstract-machine interpreter;
- MIR interpreter;
- first native backend;
- process entry/runtime bootstrap;
- basic allocator path;
- executable artifact writer;
- `omni build` vertical pipeline;
- Hello World and diagnostic examples.

### Qualification

Reference machine and native backend outputs MUST agree on all covered observations. This becomes the first mandatory differential-execution gate.

---

## 44. Phase 6 — Complete Edition 1 core semantics

### Version target

**`OMNI-IMP-0.5.x` to `OMNI-IMP-0.6.0`**

### Scope

Complete implementation of:

- expressions and control flow;
- all core types;
- generics/traits/specialization;
- closures;
- pattern matching;
- const evaluation;
- compile-time execution;
- ownership/destruction;
- panic/error/unwind;
- effects and handlers;
- capabilities;
- channels/tasks/async semantics needed by core;
- memory model;
- target-neutral machine semantics;
- unsafe core semantics.

### Rule

Every newly enabled language construct MUST be connected through the full pipeline. Front-end-only acceptance is forbidden.

---

## 45. Phase 7 — Runtime, memory, async, and structured concurrency

### Version target

**`OMNI-IMP-0.6.x`**

### What it is

The runtime support necessary to execute Omni's memory, ownership, cleanup, async, task, synchronization, cancellation, and structured-concurrency rules faithfully.

### Deliverables

- allocation/runtime ownership support;
- drop and cleanup runtime;
- async executor interface;
- task state machine;
- cancellation publication;
- cancellation masks;
- async drop executor;
- synchronization primitives;
- channel implementations;
- actor/supervisor primitives;
- deterministic replay hooks;
- runtime tracing.

### Lookouts

No runtime optimization may weaken language semantics. Runtime and reference machine behavior MUST be cross-validated.

### Tests

Concurrency stress tests, scheduler permutation tests, cancellation races, async-drop faults, lock poisoning, channel commit-order tests, memory-model litmus tests, deterministic replay tests.

### Qualification

The runtime MUST reproduce every required abstract-machine observation for the qualified subset. Scheduler choice MAY vary only where the specification permits it.

---

## 46. Phase 8 — Unsafe, provenance, assumptions, and optimization validation

### Version target

**`OMNI-IMP-0.7.x`**

### Deliverables

- unsafe obligation database;
- provenance engine;
- MIR assumption tokens;
- invalidation graph;
- unsafe checked/unchecked modes;
- memory alias model;
- optimizer legality framework;
- transformation certificates;
- translation validator;
- optimization diagnostic remarks;
- verified/high-assurance mode.

### Required optimizer workflow

`input MIR → legality precheck → transformation → transformed MIR → MIR verification → translation validation → fallback/reject`

The optimizer MUST never directly mutate a semantic invariant without producing the metadata needed by the verifier.

### Tests

Each optimization pass MUST have:

- positive legality cases;
- negative legality cases;
- stale-assumption cases;
- alias/provenance cases;
- concurrency cases where applicable;
- fault/cleanup preservation cases;
- differential reference-machine cases.

### Qualification

A validation failure MUST reject the transformed artifact and invoke the conservative fallback path. Repeated validation mismatch blocks the release branch.

---

## 47. Phase 9 — Native targets and backend maturity

### Version target

**`OMNI-IMP-0.7.x` to `0.8.x`**

### Recommended backend order

1. one stable development-native target through the simplest correct backend;
2. broader native target coverage;
3. LLVM backend where optimization breadth is needed;
4. Cranelift or equivalent fast backend where appropriate;
5. target-specific verified lowerings;
6. accelerator/MLIR work only after core native semantics are stable.

### Rule

Backend diversity MUST NOT precede canonical semantic validation. Multiple backends are useful only after one backend has passed the reference and conformance gates.

### Tests

Cross-backend semantic differential tests, ABI tests, object-format tests, relocation tests, unwind tests, target feature tests, numeric reproducibility tests, and target-specific memory/atomic tests.

---

## 48. Phase 10 — Build graph, hermetic comptime, incremental compilation, and reproducibility

### Version target

**`OMNI-IMP-0.8.0` through `OMNI-IMP-0.8.9`**

### Deliverables

- query-based incremental compiler;
- content-addressed intermediate artifacts;
- hermetic compile-time evaluator;
- capability-traced build dependencies;
- environment allowlists;
- `SOURCE_DATE_EPOCH` handling;
- deterministic seed/model policies;
- cache invalidation graph;
- reproducible artifact packaging;
- clean-room build verifier;
- build action records.

### Required cache model

A cache key MUST include every semantic input to the action, including dynamically traced compile-time dependencies and relevant target/profile/specification/toolchain identities.

### Tests

Modify every dependency class individually and prove the correct cache invalidation boundary. Run independent builds on clean machines and compare required artifact digests and semantic manifests.

### Qualification

No successful build may depend on undeclared ambient host state. Discovery of an undeclared build input is a hermeticity failure.

---

## 49. Phase 11 — Package manager, registry, update security, and release supply chain

### Version target

**`OMNI-IMP-0.8.5` through `0.9.0`**

### Deliverables

- package manifest and lockfile parser;
- deterministic resolver;
- snapshot-bound registry client;
- namespace principal verification;
- signature verification;
- revocation handling;
- update metadata chain;
- artifact digest verification;
- offline mirror support;
- secure cache handling;
- dependency provenance reports.

### Tests

Rollback, freeze, mix-and-match, namespace confusion, yanked-vs-revoked, compromised mirror, invalid signature, key rotation, lockfile downgrade, malformed metadata, endless download, and cache substitution tests.

### Qualification

A package artifact MUST NOT become build input until the resolver has accepted its complete security metadata and content identity.

---

## 50. Phase 12 — Standard library implementation and contracts

### Version target

**`OMNI-IMP-0.8.x` through `0.9.x`**, developed continuously but qualified in layers.

### What it is

The actual implementation of the library contracts already described by the specification. The standard library MUST be treated as part of the implementation qualification surface, not as examples.

### Build order

1. `std::core` with no OS dependencies;
2. allocation and low-level memory facilities;
3. collections and iterators;
4. strings/Unicode;
5. result/option/error facilities;
6. synchronization/channels/async;
7. filesystem/network/environment capabilities;
8. process/threading;
9. persistence/serialization;
10. cryptographic/security interfaces where specified;
11. diagnostics/logging;
12. higher-level platform libraries.

### Qualification

Every public library API MUST have type/effect/capability/ownership contracts, positive tests, negative tests, documentation, and compatibility metadata before being marked stable.

---

## 51. Phase 13 — FFI, ABI, runtime boundary, and plugins

### Version target

**`OMNI-IMP-0.9.0` through `0.9.4`**

### Deliverables

- C ABI support;
- foreign declaration validation;
- ABI layout contracts;
- object/link/startup integration;
- isolated FFI stack machinery where required;
- foreign exception containment;
- `longjmp`/SEH handling;
- managed-memory pin/handle bridges;
- safepoint coordination;
- plugin ABI;
- revocable plugin capabilities;
- bindgen.

### Tests

Foreign success/error/panic behavior, exception crossing, `longjmp`, asynchronous callback, signal, stack switching, GC/managed-memory interaction, pinning, FFI lifetime, ABI drift, plugin revocation, and malformed foreign metadata.

### Qualification

No foreign behavior may be inferred from the host compiler. Every crossing must be represented by an Omni-defined ABI/FFI contract.

---

## 52. Phase 14 — Persistence and distributed execution

### Version target

**`OMNI-IMP-0.9.4` through `0.9.7`**

### Deliverables

- schema identity and versioning;
- wire encoding/decoding;
- logical continuation checkpoints;
- hydration validator;
- capability re-authorization;
- duplicate-resume protection;
- crash/restart recovery;
- distributed effect/provider interfaces;
- persistence durability integration;
- cross-target checkpoint validation.

### Hard rule

A serialized continuation MUST never be treated as a remote memory image. Hydration constructs a new local execution state from validated logical data.

### Tests

Cross-architecture hydration, malicious checkpoint input, stale schema, missing capability, duplicate delivery, replay, crash before/after commit, non-static borrow rejection, capability substitution, partial hydration failure, and deterministic recovery tests.

---

## 53. Phase 15 — Developer tooling and AI compilation interface

### Version target

**`OMNI-IMP-0.8.x` onward; qualified by `0.9.x`**

### Deliverables

- LSP;
- formatter;
- refactoring engine;
- `omni fix`;
- debugger integration;
- MIR/obligation explorer;
- SARIF export;
- structured compiler API;
- AI-oriented diagnostic protocol;
- machine-applicable repairs;
- proof/validation certificate output;
- trace/replay APIs.

### AI contract

The tooling MUST make the same semantic facts available to AI systems that are available to human developers. Model confidence, generated-code provenance, or previous examples MUST never substitute for compiler evidence.

### Tests

Round-trip editing, concurrent IDE requests, stale document recovery, deterministic diagnostics, machine-repair correctness, proof-obligation completeness, and generated-code validation tests.

---

## 54. Phase 16 — Self-hosting and bootstrap trust

### Version target

**`OMNI-IMP-0.9.7` through `1.0.0-rc`**

### Stage model

**Stage-0:** Rust bootstrap implementing the smallest trusted compiler path.

**Stage-1:** Increasingly complete compiler built by Stage-0.

**Stage-2:** Self-hosted compiler built and executed using the complete Omni toolchain.

### Required migration order

Migrate in dependency order, not by convenience:

`source utilities → lexer/parser → diagnostics → data structures → names/types → HIR/MIR → verifier → compiler driver → optimizer → tooling → selected runtime/library components`

### Rules

Rust bootstrap MUST remain available until self-hosted qualification passes. Removing Stage-0 because Stage-1 “appears to work” is prohibited.

### Qualification

- bootstrap reproducibility;
- Stage-0 builds Stage-1;
- Stage-1 builds Stage-2;
- independent reconstruction of the bootstrap chain;
- compiler-output comparison;
- DDC evidence;
- complete conformance corpus;
- clean-room bootstrap.

---

## 55. Phase 17 — Full qualification and certification

### Version target

**`OMNI-IMP-1.0.0-rc.1` onward**

### What it is

The final stage where implementation claims are earned rather than inferred from feature completion.

### Qualification dimensions

Every applicable product claim MUST be qualified independently:

- Core Language;
- Platform;
- Distribution;
- each Optional Profile.

### Mandatory evidence

1. specification manifest consistency;
2. parser corpus coverage;
3. static semantics coverage;
4. dynamic/reference-machine agreement;
5. MIR verification coverage;
6. optimization validation;
7. target/ABI qualification;
8. runtime qualification;
9. standard-library qualification;
10. hermetic/reproducible builds;
11. supply-chain security;
12. FFI qualification;
13. persistent/distributed qualification;
14. security/fuzzing evidence;
15. bootstrap/DDC evidence;
16. performance and resource-bound evidence;
17. documentation and user-facing diagnostic quality.

### Final release gate

`OMNI-IMP-1.0.0` SHALL NOT be published as a complete implementation claim until every mandatory gate is either passed or explicitly classified under a published conditional-support/deviation statement allowed by the specification.

---

## 56. Cross-phase dependency discipline

### 56.1 No forward dependency leakage

A phase MUST NOT depend semantically on behavior that a later phase has not yet defined or qualified.

For example:

- LSP cannot invent language semantics to compensate for missing compiler rules.
- Optimizers cannot infer missing MIR semantics.
- Runtime code cannot invent cleanup behavior absent from the machine model.
- Standard-library APIs cannot silently widen effect/capability semantics.
- FFI cannot bypass ownership or provenance rules.
- Distributed execution cannot serialize local authority merely because a byte encoder exists.

### 56.2 Every feature must have an ownership map

Before implementation, each feature MUST identify:

- normative specification owner;
- implementation owner crate/module;
- IR representation;
- runtime representation if applicable;
- diagnostics;
- tests;
- qualification method;
- dependencies;
- invalidation/rebuild inputs;
- documentation;
- compatibility classification.

A feature without an ownership map is not ready for implementation.

---

## 57. The integrated-system rule

After every successful implementation milestone, the repository MUST be treated as a single system and re-audited as a whole.

### Required integration sequence

`implement → compile all → run focused tests → run affected conformance → run full regression → run lint/static analysis → inspect warnings → inspect generated artifacts → compare specification registry → remove obsolete code → verify dependency graph → verify documentation → verify release metadata → commit`

For larger milestones, this sequence MUST additionally include reproducibility and clean-build validation.

### Contradiction checks

Every milestone MUST check for:

- duplicate semantic implementations;
- stale code paths;
- conflicting feature flags;
- old rule IDs;
- obsolete parser branches;
- unused IR variants;
- unreachable runtime paths;
- incompatible diagnostics;
- stale generated files;
- stale documentation;
- conflicting serialization schemas;
- mismatched capability/effect rules;
- test cases that still encode superseded behavior.

### Removal rule

When a new implementation supersedes an old implementation, the old implementation MUST be deleted or explicitly isolated as historical/test-only code. Dead compatibility shims MUST NOT remain in production paths without a documented reason.

---

## 58. Testing architecture

### 58.1 Test layers

Every subsystem SHOULD have at least:

1. unit tests for local invariants;
2. integration tests for adjacent subsystem contracts;
3. conformance tests linked to normative rule IDs;
4. negative tests for rejection behavior;
5. property/fuzz tests where applicable;
6. differential tests against the reference machine or independent implementation;
7. deterministic replay tests where concurrency is involved;
8. security/adversarial tests for trust boundaries.

### 58.2 Test ownership

Each normative rule MUST have an evidence record stating whether it is covered by:

- executable test;
- formal model;
- proof certificate;
- differential comparison;
- target-specific test;
- documented conditional support.

### 58.3 Regression law

A previously passing conformance test MUST never be silently deleted because implementation changed. It may be superseded only by an explicit specification revision or test defect record.

---

## 59. Qualification metrics and dashboards

The implementation project SHOULD track at minimum:

- normative rules implemented / total;
- rules with executable tests / total;
- rules with independent model evidence / total;
- parser production coverage;
- semantic negative-test coverage;
- MIR verifier coverage;
- optimization validation coverage;
- reference/native differential agreement;
- reproducible-build rate;
- cache invalidation correctness;
- fuzzing duration and unique failures;
- compiler crash count;
- known semantic deviations;
- unsupported constructs;
- bootstrap depth;
- DDC agreement;
- security findings;
- performance regressions;
- standard-library contract coverage.

A single “test count” MUST NOT be used as the principal quality metric.

---

## 60. What to look out for throughout implementation

The following classes of defects are permanent red flags:

### Semantic drift

Implementation behavior differs from normative artifacts because an old compiler assumption survived a specification change.

### Semantic duplication

Two compiler paths independently implement the same rule and can disagree.

### Hidden ambient input

Host time, environment, filesystem enumeration, process state, randomness, locale, CPU feature state, or thread scheduling influences a semantic or cached result without appearing in the declared input set.

### Inference by accident

The compiler accepts ambiguous programs by choosing whichever result the current solver or traversal happens to produce.

### Runtime backdoors

A runtime helper silently bypasses type, ownership, effect, capability, provenance, or cancellation invariants because “the compiler already checked it.”

### Optimization laundering

An unsafe or target assumption is moved into a later compiler phase where its provenance and expiration information disappear.

### Dead compatibility code

An old implementation remains reachable and is accidentally exercised by one configuration.

### Test-shaped semantics

The implementation is made to satisfy the current tests rather than implementing the specification. Tests witness rules; they do not define them.

### Generated-artifact drift

Generated grammar tables, schemas, metadata, diagnostics, or bindings no longer correspond to their source-of-truth files.

### Tooling divergence

LSP/formatter/fix/debugger behavior parses or interprets source differently from the compiler.

### AI-specific overfitting

The compiler begins accepting constructs because an AI code generator frequently emits them rather than because the specification authorizes them.

---

## 61. Implementation change protocol

Every change MUST answer four questions before merge:

1. **What changed semantically or operationally?**
2. **Which normative rules and interfaces does it affect?**
3. **Which existing components could become inconsistent because of the change?**
4. **What evidence proves that the integrated system still satisfies the affected contract?**

The implementation process SHALL prefer one coherent change spanning all affected components over a sequence of narrowly “working” patches that leave the repository internally inconsistent.

---

## 62. Compiler trust model

The compiler itself is part of the Trusted Computing Base only to the extent required by the selected qualification claim.

Accordingly, Omni implementation work SHOULD progressively reduce unverified compiler behavior through:

- executable formal models;
- checked intermediate representations;
- mechanically linked diagnostics;
- translation validation;
- differential implementations;
- reproducible builds;
- bootstrap verification;
- DDC;
- independent rebuilds;
- hardened build modes.

The long-term objective is not merely “a compiler that seems correct,” but a compilation chain in which the remaining trust assumptions are explicit, finite, versioned, and auditable.

---

## 63. Reference implementation strategy

A reference implementation SHOULD prioritize semantic clarity over optimization.

The recommended order is:

`spec model → executable reference machine → reference checker → canonical MIR → conservative native compiler → optimized compiler`

The reference implementation MUST remain simple enough that it can serve as the baseline for differential tests even when the production compiler becomes substantially more sophisticated.

The reference implementation MUST NOT depend on optimizer-specific behavior to define language semantics.

---

## 64. First implementation milestone — exact starting point

Implementation starts with:

> **`OMNI-IMP-0.0.1 — Repository + Stage-0 Bootstrap Foundation`**

The first implementation commit MUST therefore create the trustworthy development substrate, not attempt to write the complete language parser.

The first milestone consists of:

1. empty-to-working Cargo workspace;
2. pinned Rust toolchain;
3. repository/module boundaries;
4. normative specification manifest loader;
5. stable rule-ID registry loader;
6. shared diagnostics schema;
7. test harness scaffolding;
8. deterministic build/test commands;
9. CI and security checks;
10. ADRs defining Stage-0 ownership and repository structure;
11. build provenance and artifact hashing;
12. a minimal Rust Stage-0 executable capable of loading a declared source file and producing a structured “not yet implemented” diagnostic without guessing semantics.

That final point is intentional. Before Omni can parse source correctly, the bootstrap tool must already demonstrate the project's central discipline: **unsupported behavior fails explicitly and never invents semantics.**

### `OMNI-IMP-0.0.2` acceptance gate

The repository MUST build from a clean checkout, the test harness MUST execute, specification manifests MUST validate, diagnostics MUST serialize deterministically, and the bootstrap executable MUST produce identical structured output for identical declared inputs.

Only after `OMNI-IMP-0.0.2` passes should lexer implementation begin.

---

## 65. Complete implementation progression at a glance

```text
0.0.x  Foundation / Stage-0
   ↓
0.1.x  Source / Lexer / Parser / Diagnostics
   ↓
0.2.x  Names / Types / Inference / Refinements
   ↓
0.3.x  Ownership / Lifetimes / Effects / Capabilities
   ↓
0.4.x  HIR / MIR / Verifier
   ↓
0.5.x  Reference Machine + First Executable Vertical Slice
   ↓
0.6.x  Complete Core + Runtime + Async + Concurrency
   ↓
0.7.x  Unsafe / Provenance / Optimization / Validation
   ↓
0.8.x  Backends / Build / Comptime / Reproducibility / Package System
   ↓
0.9.x  Stdlib / FFI / ABI / Plugins / Persistence / Distribution
   ↓
0.9.7+ Self-hosting / Bootstrap / DDC / Full Qualification
   ↓
1.0.0-rc  Certification candidate
   ↓
1.0.0  First qualified Omni Edition 1 implementation
```

Each downward transition is gated by integrated evidence rather than elapsed development time or feature count.

---

## 66. Completion definition for the entire project

The implementation project is complete only when all of the following are simultaneously true:

1. The compiler implements all required Edition 1 semantics within its declared product/profile claims.
2. The reference machine agrees with the implementation on the qualified semantic domain.
3. Canonical MIR is verified and used as the primary semantic checkpoint.
4. Optimizations are validated or rejected according to the selected qualification mode.
5. Safe code has no unspecified undefined behavior outside the categories explicitly permitted by the standard.
6. Ownership, effects, capabilities, concurrency, memory, unsafe, FFI, persistence, and distributed boundaries have executable evidence.
7. The build system is hermetic and reproducible for the declared environments.
8. Package and update verification prevents the specified supply-chain attacks.
9. Standard-library contracts are implemented and qualified.
10. The compiler and its tools agree on parsing and semantic interpretation.
11. Stage-0/Stage-1/Stage-2 bootstrap evidence exists.
12. DDC or equivalent independent-compilation evidence satisfies the applicable qualification claim.
13. Security, fuzzing, negative testing, and regression testing have passed their release gates.
14. No obsolete implementation path remains accidentally connected to production behavior.
15. Documentation, manifests, schemas, diagnostics, generated artifacts, and compatibility metadata describe the actual implementation state.
16. Any unsupported target/profile/feature is explicitly declared rather than silently approximated.
17. The release is independently reproducible and its qualification evidence is machine-readable.

Only then does `OMNI-IMP-1.0.0` become an appropriate implementation claim.

---

## 67. Architecture-to-implementation transition record

This section records the official transition:

**Frozen architecture:** complete.

**Semantic closure:** complete for the defined architectural audit surface.

**Normative specification:** Edition 1 `1.0.0-candidate.1` candidate suite.

**Implementation authority:** normative suite plus signed release artifacts; implementation code is subordinate evidence.

**Implementation starts at:** `OMNI-IMP-0.0.1`.

**First engineering objective:** establish the Rust Stage-0 workspace, deterministic build/test substrate, specification/rule registry ingestion, and fail-closed bootstrap executable.

**First language feature after foundation:** canonical lexer/source pipeline.

**First end-to-end proof objective:** a minimal accepted Omni program reaches canonical MIR, passes the MIR verifier, executes in the reference machine, and produces equivalent behavior through the first native backend.

**Final objective:** a self-hosting, independently qualified, reproducible Omni Edition 1 implementation whose behavior is derived from the normative suite rather than from historical compiler accidents.

---

# END OF IMPLEMENTATION MASTER PLAN


---

# Candidate 2 Amendment — Vibe-First Surface Syntax

**Status:** Candidate amendment; not ratified and not implementation-certified.
**Supersession scope:** This amendment supersedes earlier Candidate.1 wording only where this section explicitly states a surface-syntax change. All semantic rules, safety rules, type rules, ownership rules, effect/capability rules, dynamic rules, ABI rules, wire rules, build rules, and conformance rules remain unchanged unless a later rule explicitly amends them.

## A. Purpose

Edition 1 adopts a **vibe-first surface syntax** as a first-class language-design goal. The surface SHALL minimize ceremony, favor readable intent, and be suitable for both human authors and AI-generated source while preserving deterministic parsing and the exact semantic guarantees of the core language.

Vibe-first does **not** mean natural-language parsing, implicit authority, hidden blocking, hidden ownership changes, or implementation-specific interpretation. Every accepted source sequence MUST have one deterministic parse and one deterministic desugaring into the core semantic representation.

## B. Surface/Core separation

The implementation SHALL conceptually separate:

1. Lossless source/token representation.
2. Surface CST.
3. Surface AST.
4. Vibe desugaring.
5. Core semantic AST/HIR.
6. Static semantics and execution.

Surface conveniences MUST NOT require downstream semantic passes to duplicate language rules.

## C. Layout and statement termination

For Candidate 2, the earlier rules `LEX-0002`, `GRAM-0002`, and `GRAM-0003` are superseded for ordinary source statements by the following rules:

| Rule | Requirement |
|---|---|
| `VIBE-GRAM-0001` | Newline MAY terminate a statement when the parser is at a complete statement boundary and the next token cannot continue the current expression or declaration. |
| `VIBE-GRAM-0002` | A newline MUST NOT terminate a statement when the current construct is syntactically incomplete or the next token is a designated continuation token. |
| `VIBE-GRAM-0003` | Continuation tokens include binary operators, `.`, `?.`, `?`, `??`, `,`, `)`, `]`, `}`, `::`, and other grammar-declared continuation forms. The complete set is normative. |
| `VIBE-GRAM-0004` | Semicolons MAY be used explicitly and remain grammar separators; they are not required at ordinary statement boundaries when newline termination is unambiguous. |
| `VIBE-GRAM-0005` | Formatter output MUST be canonical and MUST NOT rely on indentation alone to alter the semantic interpretation of an already valid expression. |
| `VIBE-GRAM-0006` | Braced blocks remain valid in every context where the grammar admits blocks. Indentation-first formatting is preferred but does not create a second semantic language. |
| `VIBE-GRAM-0007` | Release translation MUST reject source for which newline interpretation remains ambiguous after the continuation rules are applied. IDE recovery MAY construct recovery nodes, but those nodes remain non-translatable. |

Canonical examples:

```omni
let total =
    price +
    tax +
    shipping

let user = load_user(id)?

print user.name
```

The following MUST remain one expression:

```omni
let total =
    price
    + tax
```

The following MUST remain two statements:

```omni
let x = 10
let y = 20
```

## D. Canonical vibe-oriented functions

Both explicit and compact function forms are valid where the grammar allows them:

```omni
fn add(a, b) -> a + b
```

```omni
fn greet(name) {
    print "Hello, {name}!"
}
```

The compact expression-body form desugars to the ordinary function-body representation.

## E. Command-style calls

A command-style call MAY omit parentheses when argument boundaries are deterministic:

```omni
print "hello"
save user
sleep 5.seconds
```

Parenthesized calls remain canonical escape syntax for ambiguous or nested calls:

```omni
save(user)
```

The parser MUST NOT interpret arbitrary English prose as code.

## F. Pipeline operator

Candidate 2 promotes `|>` from reserved punctuation to a standard pipeline operator.

```omni
users
    |> filter .active
    |> map .name
    |> sort
    |> take 20
```

`|>` SHALL have a precedence lower than ordinary function application but above statement termination and SHALL associate left-to-right.

The pipeline:

```omni
value |> f
```

desugars to the semantically equivalent application defined by the pipeline specification. The desugaring MUST preserve ownership, borrowing, lifetimes, effects, capabilities, and evaluation order.

## G. Projection shorthand

A leading field path in a pipeline stage MAY represent an implicit projection closure:

```omni
users |> map .name
```

which semantically corresponds to:

```omni
users |> map (|value| value.name)
```

The projection form MUST be limited to grammar-declared projection contexts and MUST NOT become arbitrary implicit variable resolution.

## H. Query/dataflow surface

The following are candidate standard-library/DSL forms built on ordinary calls and pipeline semantics:

```omni
users
    |> where .active
    |> where .age >= 18
    |> map .name
    |> sort
    |> take 20
```

Keywords such as `where`, `sort`, `take`, and `map` are NOT individually semantic compiler primitives unless the normative library or DSL specification says so. Their meaning normally comes from normal name resolution and typed APIs/macros.

## I. Resource scopes

Candidate 2 standardizes a concise resource-scope surface where the underlying semantic model supports deterministic cleanup:

```omni
with file = open(path)? {
    process file
}
```

The construct SHALL lower to the existing cleanup/ownership semantics and MUST preserve deterministic destruction and fault behavior.

## J. Structured concurrency surface

Candidate 2 standardizes concise structured concurrency syntax:

```omni
parallel {
    profile = fetch_profile(user)
    posts = fetch_posts(user)
}
```

and:

```omni
parallel for user in users {
    process user
}
```

These constructs SHALL represent structured task scopes, not detached/background tasks. Failure, sibling cancellation, cleanup, join, and happens-before behavior remain governed by `OMNI-CONC`, `OMNI-EFFECTS`, and `OMNI-MACHINE`.

## K. Explicit authority syntax

Capabilities remain distinct from effects. Candidate 2 provides concise declaration syntax:

```omni
fn send_report(report)
    requires Email.Send, Storage.Read
    -> Result<(), Error> / io
{
    ...
}
```

The `requires` clause declares authority requirements; it MUST NOT implicitly infer authority from the presence of an effect.

## L. Optional chaining and coalescing

Candidate 2 standardizes:

```omni
user?.address?.city ?? "unknown"
```

The forms MUST preserve the existing `Option`/nullable and evaluation semantics.

## M. Vibe-friendly closures

Where unambiguous, compact closures MAY be used:

```omni
users |> map user => user.name
```

and projection shorthand remains preferred for simple field projections:

```omni
users |> map .name
```

The grammar MUST distinguish closures from comparison/arrow syntax deterministically.

## N. Expression orientation

`if`, `match`, blocks, loops where specified, and `try` remain expression-oriented. A final expression may provide the block value without a trailing semicolon. Explicit `discard` remains available when a non-unit result is intentionally ignored.

## O. AI-generation invariants

The surface language SHALL be optimized for predictable generation and repair:

- repeated common idioms SHOULD have one obvious spelling;
- syntax sugar MUST have one deterministic desugaring;
- diagnostics SHOULD identify the smallest repairable region;
- generated code MUST remain valid after canonical formatting;
- syntax MUST avoid dependence on whitespace alignment for semantic identity;
- arbitrary natural-language interpretation is prohibited;
- omitted information may be inferred only through specified deterministic inference rules.

## P. Compatibility and implementation status

This Candidate 2 amendment does not by itself certify implementation support. Until the corresponding parser, lexer, formatter, AST, desugaring, diagnostics, semantic, and conformance gates pass, the implementation MUST report the Candidate 2 surface as planned/candidate rather than complete.

## Q. Required Candidate 2 conformance corpus

The release corpus SHALL contain positive, negative, recovery, ambiguity, CST round-trip, desugaring, and semantic-integration cases for every surface construct introduced or modified by this amendment.

At minimum the corpus SHALL cover:

- newline termination and continuation;
- semicolon compatibility;
- command-style calls;
- pipeline chains;
- pipeline precedence;
- pipeline ownership/effect preservation;
- projection shorthand;
- closures;
- optional chaining;
- coalescing;
- resource scopes;
- structured concurrency;
- capability requirements;
- generic/relational ambiguities;
- formatter canonicalization;
- malformed/incomplete AI-style source;
- deterministic recovery.

## R. Candidate 2 ratification gate

Candidate 2 SHALL NOT be promoted to a ratified Edition 1 normative baseline until:

1. all referenced normative artifacts agree;
2. the grammar is machine-readable and unambiguous;
3. the lexer/parser/CST implementation exists;
4. desugaring is deterministic;
5. semantic preservation tests pass;
6. diagnostics and formatter conformance pass;
7. the complete affected workspace is qualified;
8. the manifest and content hashes are regenerated;
9. the conformance evidence is linked to the corresponding rule IDs.
