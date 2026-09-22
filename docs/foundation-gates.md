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
`2af51aa99272b82db8785d234b971e859d0503d24017eb09f5d25c919cd8ba95` (supersedes the 0.0.0.8
`ec1f8b1e…` value by content repair — seven recovered STAGE0 rules plus widened schema patterns —
not by contradiction).

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

## Compiler topology (0.0.0.9, `omni-topology`)

Pipeline-order tiers (higher = later stage; production edges point at the same or an earlier tier):
source 0, lex 1, syntax 2, parse 3, names 4, types 5, own/effects/traits 6, hir 7, mir 8, verify 9,
machine/codegen 10, runtime 11; driver 12 orchestrates and may use any tier. Below-MIR consumers
declare exact feeds (`machine: {mir}`, `codegen: {mir, verify}`, `verify: {mir}`) so frontend
syntax can never be consumed past the lowering boundary. `stage0` is edgeless bootstrap
infrastructure; `tools/*` may use tools/externals but never `compiler/*`, and vice versa.
Dev/build edges are direction-exempt but cycle-checked; diagnostics are deterministic
(filename-sorted traversal, sorted edges). Dormant downward edges (e.g. hir scaffolding) are tier
intent, not violations. 0.0.0.9 corrections: removed the layer-skipping `machine → parse` scaffold
(the constant-input parse never influenced execution; behavior unchanged) and the twelve unused
driver path dependencies, leaving the proven `machine`/`codegen` orchestration interface. Live
state: 24 members, 19 normal + 1 dev + 0 build internal edges, clean.

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
Live gaps, honestly held: zero E-code diagnostics emitted, zero witness records, all 567 rules
Candidate (Ratified obligations vacuous), `models/`/`data` still empty.

## Specification loader (0.0.0.10, `omni-registry`)

Typed load boundary owned by `omni-registry` (TOOL tier; no compiler edges): `RawSpecification`
→ `validate()` → `ValidatedSpecification` → `load()` → immutable `LoadedSpecification`, with
phase-ordered checks (containment, duplicate-rejecting parse, version, schema shape, canonical
form via `omni-canon`, digest verification, cross-artifact consistency, construction, provenance).
Manifest authority is enforced: the tree is never inferred by scanning; required artifacts come
from the canon publication set; `models/`/`data` emptiness is an explicit verified state, while
any present-but-unregistered model file fails as undeclared (no model schema exists yet, and none
is invented). Only `1.0.0`/edition 1 loads; future versions fail closed. Load provenance
(loader version, tree digest, per-artifact digests, ordering) is captured; full cryptographic
build provenance stays at 0.0.0.12. Authority seams: canon owns bytes/digests, the loader owns
typed validation, `omni-conform` adjudicates release bindings and evidence on loaded values
(no parallel parsing), `omni-audit` keeps linkage claims, cycles, and witness resolution.
Rule-text to hash binding remains procedural (texts live outside the tree).

## Stage-0 predicates (0.0.0.11, `omni-stage0` + loader assist)

Postmortem honestly recorded: the 0.0.0.4 extractor's `[A-Z]+` identifier class silently dropped
all seven `STAGE0-*` rules (the sole digit-bearing prefix in the normative suite). The registry now
holds 567 rules (557 Candidate + 3 Superseded + 7 STAGE0 Candidate); the same class bug was repaired
in all eight schema patterns and the evidence ID validator, with a regression test proving
`STAGE0-0007` validates end to end. Tree digest rebound by publication growth (`ec1f8b1e…` →
`2af51aa9…`) through the established process, never by hand edit.

Predicate mechanism (no second grammar, no new semantics): `Stage0PredicateEngine` built only from
validated lists (raw-JSON constructor removed), enforcing duplicate/conflict/empty/version
failures; exactly one profile (`stage0`); unknown features fail as unknown, never as disabled
(distinct from forbidden). Restriction records map Stage-0 rules to registry or grammar-production
authorities with resolvability checks; no mappings are curated yet, and none are fabricated.
Manifest predicate-set integrity (presence, string elements, disjointness) is enforced by the
loader helper and re-checked on every `omni-conform` run. A dev-dependency integration test proves
the chain canon → loader → predicates against the live manifest (14 allowed + 8 forbidden);
production compiler wiring awaits 0.0.1.x consumers, with `compiler → stage0` documented as the
future query edge (infra stays edge-free today, so topology is unchanged).
