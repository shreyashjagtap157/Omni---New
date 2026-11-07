# Omni Self-Hosting Compiler Architecture and Bootstrap Strategy

This document specifies the self-hosting architecture for the Omni programming language compiler and runtime in accordance with Part XIV of `OMNI - Master Project Vision, Historical Requirements, Research Charter, Implementation Mandate and Continuous Evolution Policy.md` and the Stage-0/Stage-1/Stage-2 bootstrap charter.

## 1. Principles of Self-Hosting in Omni

Per §20 of the Master Charter:
- Self-hosting is an important developmental milestone, not the ultimate definition of correctness.
- A trusted initial compiler (the Rust-based `omni-driver` and compiler suite) acts as the trusted seed.
- Diverse Double Compilation (DDC) and compiler-stage comparison ensure bootstrap integrity and protect against compiler trust compromise (Thomson-style Trojan insertions).

## 2. Bootstrap Pipeline Stages

```
   +--------------------+
   |   Stage 0 (Seed)   |  Written in Rust: omni-lex, omni-parse, omni-types, omni-mir,
   |     (Trusted)      |  omni-verify, omni-machine, omni-codegen, omni-driver.
   +---------+----------+
             |
             | Compiles Stage 1 Omni source
             v
   +--------------------+
   |  Stage 1 (Omni)    |  Written in Omni: self-hosting lexer (lexer.omni),
   | (Initial Bootstrap)|  parser (parser.omni), type analyzer, and MIR emitter.
   +---------+----------+
             |
             | Compiles Stage 2 Omni source
             v
   +--------------------+
   |  Stage 2 (Fixed)   |  Self-built Omni compiler: compiles itself into Stage 3.
   |  (Self-Hosting)    |  Stage 2 and Stage 3 outputs must converge identically.
   +--------------------+
```

## 3. Experimental Omni-written micro-prototypes (not operational compiler stages)

The files below are early experiments written in Omni. Their current source is deliberately small and uses fixed, hand-authored examples. They are useful for exercising the existing compiler on Omni syntax and for exploring representations, but they are **not yet input-driven compiler components** and must not be described as a working self-hosted frontend or compiler.

1. **`lexer.omni` — fixed-input scanner experiment**
   - `main` embeds one fixed ASCII source byte array and scans a narrow subset of identifiers, decimal integers, selected keywords, operators and delimiters.
   - It does not expose a token stream to another module; it returns a token-count/hash value. The declared `Token` structure is not used to produce tokens.
   - It does not yet implement the normative lexical contract: arbitrary source input, full Unicode identifiers, complete literal/comment/trivia handling, source-span diagnostics, malformed-input behavior or recovery.

2. **`parser.omni` — AST-encoding experiment**
   - It manually calls helper functions with preselected integer values to encode a sample signature, expressions and statements.
   - It does not consume lexer output, inspect source text, recognize productions, construct a general AST, or report parse diagnostics. Despite the historical filename/comment, this is not yet a recursive-descent parser.

3. **`names.omni` — bounded scope-lookup experiment**
   - It initializes fixed arrays of integer name IDs, scope levels and symbol-kind values, then demonstrates reverse lookup and a simple shadowing case.
   - It is not connected to parsed identifiers or declarations and does not yet implement general module, namespace, declaration, visibility or error-resolution behavior.

4. **`types.omni` — operator-rule experiment**
   - It evaluates a small set of hard-coded integer type tags and binary/conditional typing cases.
   - It is not connected to parsed expressions or resolved symbols and does not implement a general type representation, unification, inference, generics, traits, effects or diagnostics.

5. **`mir.omni` — hand-constructed CFG encoding experiment**
   - It constructs three sample blocks and terminators directly, then returns a composite integer.
   - It does not lower parsed or typed source, define the production MIR data model, run MIR verification, or establish semantic preservation.

The historical execution reports for these fixtures describe reference-machine and native execution. Such execution demonstrates that the particular fixture computations can run through those execution paths; it does **not** establish that the files implement the corresponding compiler subsystems, conform to the complete Edition 1 rules, or are semantically equivalent to the Rust implementation.

## 4. Verification & Conformance

- The Stage-0 Rust compiler remains available as a permanent validation oracle and differential test harness.
- All self-hosted stages execute against the authoritative normative Edition-1 test corpus.
