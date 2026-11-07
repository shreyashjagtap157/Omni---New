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

## 3. Current Implemented Self-Hosting Modules

1. **`lexer.omni`**:
   - Tokenizes Omni keywords (`fn`, `let`, `mut`, `struct`, `return`, `if`, `else`, `while`).
   - Identifiers, integer literals, ASCII operators, and delimiters.
   - Verified on both the abstract reference machine (`omni-driver`) and native Cranelift / MSVC compilation.
2. **`parser.omni`**:
   - Encodes AST nodes, parameter lists, function signatures, binary expression trees, and return statements.
   - Verified on both the abstract machine and native Cranelift / MSVC compilation.

## 4. Verification & Conformance

- The Stage-0 Rust compiler remains available as a permanent validation oracle and differential test harness.
- All self-hosted stages execute against the authoritative normative Edition-1 test corpus.
