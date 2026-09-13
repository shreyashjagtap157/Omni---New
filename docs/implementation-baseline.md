# Omni implementation baseline

Implementation planning is frozen by the supplied sealed ledger:

- Plan SHA-256: `cdf7ff8cb0eae597b52e295fa43c11f701415f62f2a292f4bd4a2be5012cb506`
- Normative specification SHA-256: `e3ff0e1b6ef0f1f1d713647cc3c0c04dfd940513d292bac95f7b0d44e6c95f0c`
- Stage boundary sequence: `0.0.0.1` -> `1.0.0.0` -> `2.0.0.0` -> `3.0.0.0`

This repository currently contains only foundation mechanics. No lexer, parser, type checker,
ownership analysis, MIR evaluation, or code generation semantics are implemented.

The initial baseline commit is preserved in Git history. Foundation qualification is fail-closed when
the required Rust toolchain or authoritative specification inputs are absent.
