# Foundation gates

The foundation sequence is ordered `OMNI-IMP-0.0.0.2` through `OMNI-IMP-0.0.0.13`.

The exact Rust pin is `1.95.0`, but qualification remains fail-closed until the host provides
`rustc`, `cargo`, `rustup`, and the required targets. The pinned workspace uses Cargo resolver 2
to preserve the declared Stage-1 toolchain contract; changing this would invalidate that contract.
This is a tooling-compatibility constraint, not a language-semantic change. Foundation targets are
`x86_64-unknown-linux-gnu` and `riscv64-unknown-none-elf`; under Rust 1.95.0 the latter is provided
as the canonical `riscv64gc-unknown-none-elf` target.

The repository does not yet contain the normative Edition 1 specification tree. The manifest records
the authoritative specification digest supplied for this implementation. `0.0.0.13` must not claim
verification until an authoritative `spec/` tree is installed and its computed digest matches.

## Specification digest domain

`spec_tree_sha256` is the digest of the normative Edition 1 specification content only. Generated
implementation manifests and release evidence under `spec/manifest/` and `spec/release/` are excluded
from the hashed tree to avoid a self-referential digest. The qualification tool must define and record
this exclusion set explicitly; it must never hash the manifest containing the digest it is checking.

Publication set (0.0.0.5): `grammar/omni-edition1.ebnf`, `registry/rules.json`,
`schemas/rule-registry.schema.json`, plus anything under `models/` or `data/` (currently only
`.gitkeep` scaffolding, which is excluded and contributes nothing). Procedure: walk included
directories in lexicographic order of slash-joined relative paths, normalize file bytes CRLF/CR to
LF, hash each file with SHA-256, then hash the concatenation of `path + LF + hex + LF` lines.
`omni-canon --spec-tree` computes it; `omni-conform` rejects any manifest/gate digest mismatch
fail-closed. Current digest: `ecda85bb1f460266abf2b05f0a9f8058c9c6162f8c448ef1f46989ad3577698b`.

Known specification inconsistency (open, not resolved by invention): the normative EBNF predates the
Candidate-2 amendment and contains no pipeline-operator, newline-termination, projection-shorthand,
or command-call productions, while `VIBE-GRAM-*` rules normatively require them. Grammar
modernization belongs to the grammar milestone; the tree binds the artifact as-is.
