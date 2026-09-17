# Omni Vibe Surface Syntax — Edition 1 Candidate 1

**Status:** Candidate design/amendment for Edition 1; not ratified.

This document is the focused surface-syntax companion to the complete Candidate 2 specification. It exists so parser, formatter, AST, desugaring, IDE, and conformance work can proceed against one explicit surface-language target without weakening Omni's semantic guarantees.

## 1. Design principle

Omni should feel like the programmer is expressing intent directly:

```omni
users
    |> where .active
    |> map .name
    |> sort
    |> take 20
```

while the compiler still produces a fully explicit, statically checked semantic representation.

**Vibe-oriented means low ceremony and predictable intent, not ambiguous English parsing.**

## 2. Surface architecture

```text
source bytes
  -> lossless lexer
  -> parser events
  -> lossless CST
  -> surface AST
  -> deterministic vibe desugaring
  -> core semantic AST/HIR
  -> names / types / ownership / effects / capabilities
  -> MIR
  -> verifier
  -> reference machine / backend
```

## 3. Canonical examples

### Function

```omni
fn greet(name) {
    print "Hello, {name}!"
}
```

### Expression function

```omni
fn add(a, b) -> a + b
```

### Data transformation

```omni
let names =
    users
        |> where .active
        |> map .name
        |> sort
```

### Structured concurrency

```omni
parallel {
    profile = fetch_profile(user)
    posts = fetch_posts(user)
}
```

### Resource scope

```omni
with file = open(path)? {
    process file
}
```

### Explicit authority

```omni
fn send_report(report)
    requires Email.Send, Storage.Read
    -> Result<(), Error> / io
{
    ...
}
```

## 4. Core rules

### Newlines

Newline termination is deterministic. It applies only at a complete statement boundary when the next token cannot continue the current construct. Continuation forms remain part of the same expression.

### Semicolons

Semicolons remain legal explicit separators. Ordinary unambiguous statement boundaries need not use them.

### Pipelines

`|>` is a standard left-associative pipeline operator in Candidate 2.

### Projection shorthand

`.field` in a projection context becomes an implicit field-access closure.

### Command calls

`print value` may be a call when the grammar can determine the call boundary uniquely. Ambiguous cases use parentheses.

### Braces

Braced blocks remain fully supported. Indentation is formatter/layout syntax, not a second semantic type of block.

## 5. What must remain explicit

The vibe surface MUST NOT hide:

- ownership transfer;
- lifetime violations;
- capability acquisition or escalation;
- effect authority;
- unsafe operations;
- foreign execution boundaries;
- cancellation semantics;
- distributed state transfer;
- nondeterministic host access;
- security-relevant operations.

## 6. Desugaring examples

```omni
users |> map .name
```

becomes conceptually:

```omni
map(users, |user| user.name)
```

while:

```omni
with file = open(path)? {
    process file
}
```

becomes the corresponding typed resource-scope/cleanup representation.

Surface sugar MUST preserve semantic identity and must never bypass static checking.

## 7. Invalid vibe syntax

The language MUST reject attempts to turn arbitrary prose into source:

```omni
give me all users that are active and older than 18
```

The supported equivalent is explicit, deterministic syntax:

```omni
users
    |> where .active
    |> where .age >= 18
```

## 8. Required parser test families

1. Every keyword/token.
2. Every declaration form.
3. Every expression form.
4. Every precedence boundary.
5. Every newline continuation case.
6. Every pipeline ambiguity.
7. Every generic/shift ambiguity.
8. Every closure ambiguity.
9. Every malformed form.
10. Every recovery form.
11. CST exact-source round trip.
12. Surface-to-core desugaring equivalence.

## 9. Required semantic test families

Every surface convenience must additionally be tested for:

- name resolution;
- type inference/checking;
- ownership and borrowing;
- lifetime constraints;
- effects;
- capabilities;
- async/concurrency;
- unsafe assumptions;
- deterministic execution;
- backend equivalence where applicable.
