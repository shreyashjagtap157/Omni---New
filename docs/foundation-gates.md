# Foundation gates

The foundation sequence is ordered `OMNI-IMP-0.0.0.2` through `OMNI-IMP-0.0.0.13`.

The exact Rust pin is `1.95.0`, but qualification remains fail-closed until the host provides
`rustc`, `cargo`, `rustup`, and the required targets. The pinned workspace uses Cargo resolver 2
to preserve the declared Stage-1 toolchain contract; changing this would invalidate that contract.
This is a tooling-compatibility constraint, not a language-semantic change. Foundation targets are
`x86_64-unknown-linux-gnu` and `riscv64-unknown-none-elf`; under Rust 1.95.0 the latter is provided
as the canonical `riscv64gc-unknown-none-elf` target.

The repository carries the normative Edition 1 specification tree bound by `spec_tree_sha256`
(see below). `0.0.0.13` must not claim verification until the full foundation sequence qualifies
and the computed digest matches at release time.

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

## Canonicalization domain (0.0.0.6 engine, `omni-canon` lib)

JSON documents: recursive lexicographic key order (UTF-8 byte order), arrays keep document order
(order is semantic), exact UTF-8, LF normalization inside string values (RULES section 4),
integral finite floats fold to integers (MANIFEST-0002; fractional floats kept), insignificant
whitespace never participates, duplicate object keys rejected fail-closed, malformed input rejected.
Paths are structural, never substring substitution: a string virtualizes to `/omni-root/...` only on
a component-boundary prefix of the declared root (ASCII drive letter insignificant); absolute-shaped
strings outside the root are rejected as unresolved host identity (REPRO-0004, BUILD-0005);
all other strings pass through byte-identical, including backslash literals and relative fragments,
whose producers must pre-normalize. One digest representation: lowercase hex SHA-256.
Rule-text hashes re-derive the 0.0.0.4 registry vectors mechanically. Provenance artifact hashes
(raw byte SHA-256 in `omni-driver`) are an intentionally separate domain, not canonicalization.
NFC is a producer obligation (no normalization dependency in the workspace). The 0.0.0.5 tree
procedure and publication set are unchanged; tree digest still
`ecda85bb1f460266abf2b05f0a9f8058c9c6162f8c448ef1f46989ad3577698b`.
