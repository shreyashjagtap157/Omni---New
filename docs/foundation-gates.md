# Foundation gates

The foundation sequence is ordered `OMNI-IMP-0.0.0.2` through `OMNI-IMP-0.0.0.13`.

The exact Rust pin is `1.80.0`, but qualification remains fail-closed until the host provides
`rustc`, `cargo`, `rustup`, and the required targets. The pinned workspace uses Cargo resolver 2
because resolver 3 was not available in Rust 1.80; changing this would invalidate the declared
Stage-1 toolchain contract. This is a tooling-compatibility constraint, not a language-semantic
change.

The repository does not yet contain the normative Edition 1 specification tree. The manifest records
the authoritative specification digest supplied for this implementation. `0.0.0.13` must not claim
verification until an authoritative `spec/` tree is installed and its computed digest matches.

## Specification digest domain

`spec_tree_sha256` is the digest of the normative Edition 1 specification content only. Generated
implementation manifests and release evidence under `spec/manifest/` and `spec/release/` are excluded
from the hashed tree to avoid a self-referential digest. The qualification tool must define and record
this exclusion set explicitly; it must never hash the manifest containing the digest it is checking.
