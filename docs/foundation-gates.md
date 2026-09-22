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

Publication set (0.0.0.5, extended 0.0.0.8 with the seven evidence schemas):
`grammar/omni-edition1.ebnf`, `registry/rules.json`, all ten `schemas/*.schema.json`, plus anything
under `models/` or `data/` (currently only `.gitkeep` scaffolding, which is excluded and contributes
nothing). Procedure: walk included directories in lexicographic order of slash-joined relative
paths, normalize file bytes CRLF/CR to LF, hash each file with SHA-256, then hash the concatenation
of `path + LF + hex + LF` lines. `omni-canon --spec-tree` computes it; `omni-conform` rejects any
manifest/gate digest mismatch fail-closed. Current digest:
`ec1f8b1e680b189493be45a7cb52d0c74cc0d99b70e9a6193dcda2dc50e6a130` (supersedes the 0.0.0.5
`ecda85bb…` value by publication-set growth, not by contradiction).

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

## Linkage reaper (0.0.0.7, `omni-audit`)

Single ownership model: `#[implements("ID")]` and `implements!("ID")` claims under `compiler/` and
`tools/` (generated `target/` excluded, filename-sorted walk) must resolve to registry rules in
`Candidate` or `Ratified` state; `Proposed`/`Deprecated`/`Superseded`/`Withdrawn` ownership, unknown
IDs, and malformed claim syntax all fail closed, as do unparseable sources, symlinks, missing trees,
duplicate registry IDs, unknown dependencies, dependency cycles, and unknown statuses. Witness
entries must each resolve to a contained workspace file or a collected `#[test]` name; every
`Ratified` rule must be both claimed and witnessed. Coverage is per-rule, not per-file: untagged
infrastructure/derived/oracle code is permitted. Duplicate claims across files are allowed and
attributed (collaborative implementation). Live state: 8 tags in 57 files (74 test fns), clean.
Tracked gaps (not invented): RULE-0003 `erratum-corrected` has no enum spelling (needs spec ADR);
rule-text to hash binding is procedural (deterministic extraction, tree-pinned registry bytes);
`models/`/`data` emptiness belongs to later milestones.

## Evidence schemas (0.0.0.8, `omni-evidence` + `spec/schemas/`)

Seven strict (`additionalProperties: false`, version `"1.0.0"`) contracts: `diagnostic` (E-code ID
per the registry diagnostics vocabulary, severity error/warning/note/help, primary rule, spans,
params, fixits), `witness` (`WIT-NNNN`, live-rule only, optional revision pinned to the registry
hash with staleness rejection, file artifact resolution), `conformance-outcome` (`CONF-NNNN`,
pass/fail, tree/toolchain-bound), `verification-failure` (`VF-NNNN`, seven failure classes,
rule-or-model obligation, release impact), `provenance` (spec/tree/target/artifact identity;
historical bindings format-checked, never equality-forced), `regression` (`REG-NNNN`,
open/accepted/closed, contained input), `fuzz-promotion` (`FUZZ-NNNN`, candidate/minimized/
promoted/rejected, seed digest, reproducer). Cross-references (diagnostic/witness/regression/
promotion links) resolve within the validated corpus; `omni-conform` additionally validates any
`spec/evidence/*.json` corpus against registry, tree, and toolchain. Pre-existing compiler error
enums and the parse `Diagnostic` remain implementation-internal and must map to E-code records at
emission boundaries (future work); `omni-driver` provenance adopts this format at 0.0.0.12.
Live gaps, honestly held: zero E-code diagnostics emitted, zero witness records, all 560 rules
Candidate (Ratified obligations vacuous), `models/`/`data` still empty.
