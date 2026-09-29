# Generic Closer Splitting Contract

## Context

In nested generic arguments such as `Vec<Option<i32>>` or shift-right assignment in generic const contexts (`>>=`), the lexer emits compound tokens like `Punct::Shr` (`>>`) or `Punct::ShrEq` (`>>=`).

## Contract Invariants (0.0.2.2 / 0.0.2.3-A)

When the parser needs to split compound closer tokens (`>>` or `>>=`) in generic contexts:

1. **Byte Exactness**: The original source bytes must be perfectly preserved.
2. **Offset Integrity**: Exact byte offsets (`start..end`) for each split character must be preserved without gap or overlap.
3. **Trivia Ownership**: Leading trivia attaches to the first split token (`>`); trailing trivia attaches to the final split token (`>` or `=`).
4. **No Mutation of Prior CST Nodes**: Previously emitted children and token indices remain valid and untouched.
5. **Determinism**: Token splitting behavior is completely deterministic.
6. **Zero Source Loss**: `parse(source).syntax().text() == source` must hold strictly.

## Tested Scenarios in 0.0.2.3-A

- Nested generic closers (`> >` vs `>>`).
- Generic closer followed by assignment (`>>=`).
- Comments and whitespace surrounding split generic closers.
- Multi-byte Unicode before and after generic closers.
