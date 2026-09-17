# Omni Vibe Syntax — Implementation Handoff & Acceptance Contract

## Purpose

This is the implementation handoff for the Candidate 2 vibe-first surface language.

It is intentionally separate from the complete semantic specification: the language should be easy to write, while its internal semantics remain strict.

## Non-negotiable rule

Do not implement this as arbitrary natural-language parsing.

Every accepted form must have deterministic grammar, deterministic AST, deterministic desugaring, and ordinary downstream semantic checking.

## Required compiler architecture

```text
UTF-8 source
  -> Source/Cursor
  -> lossless lexer
  -> trivia-aware token stream
  -> parser events
  -> Rowan green CST
  -> surface AST views
  -> vibe desugaring
  -> semantic AST/HIR
  -> names
  -> types
  -> ownership/lifetimes
  -> effects/capabilities
  -> MIR
  -> verifier
  -> reference machine
  -> backend/runtime
```

## Candidate 2 feature checklist

### Layout

- newline statement termination at unambiguous boundaries;
- continuation token rules;
- explicit semicolon compatibility;
- brace blocks;
- canonical formatter.

### Calls

- parenthesized calls;
- deterministic command-style calls;
- named arguments;
- closure calls.

### Expressions

- complete precedence table;
- assignment;
- ranges;
- comparisons without chaining;
- logical/bitwise operators;
- casts;
- unary operators;
- postfix calls/indexing/field access;
- optional chaining;
- null coalescing.

### Vibe conveniences

- `|>` pipeline;
- `map .field` projection shorthand;
- `where .field == value` DSL/API style;
- expression-oriented blocks;
- `with resource = acquire()` resource scopes;
- `parallel { ... }` structured concurrency;
- `parallel for`.

### Static semantics integration

Every convenience must preserve:

- DefIds;
- scopes;
- type inference;
- ownership;
- borrowing;
- lifetimes;
- effects;
- capabilities;
- async/cancellation semantics;
- unsafe assumptions;
- source spans.

## Mandatory test matrix

For each surface construct provide at least:

```text
valid-minimal
valid-normal
valid-nested
valid-composed
invalid-syntax
recovery
ambiguity
round-trip
CST-golden
desugaring-golden
semantic-accept
semantic-reject
formatter
regression
```

## Pipeline requirements

Test:

```omni
users |> map .name
users |> filter .active |> map .name
value |> f |> g
```

and malformed/ambiguous cases involving:

```text
>
>>
|>
||
->
=>
::<
< >
```

The parser must never confuse pipeline syntax with logical OR, comparison, generic arguments, or lambda syntax.

## Newline requirements

Test:

```omni
let x =
    a +
    b
```

as one expression, while:

```omni
let x = 1
let y = 2
```

is two statements.

Test all designated continuation tokens.

## CST requirement

For every valid source fixture where losslessness is required:

```text
parse(source).cst.text() == source
```

must hold exactly.

## Desugaring requirement

For every sugar form, retain a test proving equivalence to explicit core syntax.

Example:

```omni
users |> map .name
```

must be equivalent to the corresponding explicit map call with a field projection closure.

## Safety requirement

Surface sugar must never bypass semantic checks.

Examples:

```omni
with file = open(path)? { ... }
```

must still receive full ownership and cleanup checking.

```omni
parallel { ... }
```

must still receive full structured-concurrency and cancellation checking.

```omni
fn send_report(report) requires Email.Send { ... }
```

must still use the capability environment independently from the effect row.

## Acceptance

Candidate 2 surface support can be marked implementation-qualified only when the complete affected workspace passes the project's required checks and the entire syntax/desugaring/semantic matrix passes.
