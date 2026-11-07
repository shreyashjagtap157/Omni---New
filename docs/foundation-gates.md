# Foundation gates

The foundation sequence is ordered `OMNI-IMP-0.0.0.2` through `OMNI-IMP-0.0.0.13`.

The exact Rust pin is `1.95.0`, but qualification remains fail-closed until the host provides
`rustc`, `cargo`, `rustup`, and the required targets. The pinned workspace uses Cargo resolver 2
to preserve the declared Stage-1 toolchain contract; changing this would invalidate that contract.
This is a tooling-compatibility constraint, not a language-semantic change. Foundation targets are
`x86_64-unknown-linux-gnu` and `riscv64-unknown-none-elf`; under Rust 1.95.0 the latter is provided
as the canonical `riscv64gc-unknown-none-elf` target.

The repository carries the normative Edition 1 specification tree bound by `spec_tree_sha256`
(see below). `0.0.0.13` technical checks have passed on commits `d270e938` and `64f734b9`.
This is evidence for those exact commits, not a declaration that the complete compiler or
language is qualified. The committed gate status is not sufficient evidence of formal
sign-off; see the proposed `docs/adr/0002-foundation-gate-qualification-evidence.md`.
The canonical specification digest must still match the manifest and gate record at the
exact commit being evaluated.

## Specification digest domain

`spec_tree_sha256` is the digest of the normative Edition 1 specification content only. Generated
implementation manifests and release evidence under `spec/manifest/` and `spec/release/` are excluded
from the hashed tree to avoid a self-referential digest. The qualification tool must define and record
this exclusion set explicitly; it must never hash the manifest containing the digest it is checking.

Publication set (0.0.0.5, extended 0.0.0.8 with the seven evidence schemas, extended by gate
closure with `registry/rule-texts.json` and `grammar/candidate2-erratum.md`):
`grammar/omni-edition1.ebnf`, `grammar/candidate2-erratum.md`, `registry/rules.json`,
`registry/rule-texts.json`, all eight `schemas/*.schema.json`, plus anything under `models/` or
`data/` (currently only `.gitkeep` scaffolding, which is excluded and contributes nothing).
Procedure: walk included directories in lexicographic order of slash-joined relative paths,
normalize file bytes CRLF/CR to LF, hash each file with SHA-256, then hash the concatenation
of `path + LF + hex + LF` lines. `omni-canon --spec-tree` computes it; `omni-conform` rejects any manifest/gate digest mismatch fail-closed. Current canonical
digest, independently printed by `omni-canon` and matched by the manifest and gate in CI:
`3986c2a71d9e5e4092331d025079055f34b98aa4179de1684b64e28bde74e6ed`.
The earlier `07762061…` and `7e63431f…` values are historical intermediate snapshots,
not the current tree identity.

Known specification/implementation inconsistency (open, not resolved by invention): the core
Edition 1 `expression`/`statement` productions do not reference the VIBE-GRAM constructs, but
the EBNF file contains a trailing block of standalone VIBE-GRAM extension definitions. The
default parser also accepts some overlapping unratified forms (`|>`, `?.`, and prefix-dot
projection shorthand). The driver lowers pipeline and projection-shorthand nodes; optional chaining
parses but currently fails at native AST lowering. Command-call lowering exists but command-call
syntax is disabled by default and explicitly rejected by parser tests. Candidate 2 remains pre-release
and Candidate 3 is unratified. ADR-0003 records the authority-boundary decision; no grammar or parser
behavior is changed here.

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
NFC is a producer obligation (no normalization dependency in the workspace). The 0.0.0.5 tree procedure and publication set are unchanged. The digest
`ecda85bb1f460266abf2b05f0a9f8058c9c6162f8c448ef1f46989ad3577698b` is the
historical 0.0.0.6 checkpoint and is superseded by the current digest recorded above.

## Linkage reaper (0.0.0.7, `omni-audit`)

Single ownership model: `#[implements("ID")]` and `implements!("ID")` claims under `compiler/` and
`tools/` (generated `target/` excluded, filename-sorted walk) must resolve to registry rules in
`Candidate` or `Ratified` state; `Proposed`/`Deprecated`/`Superseded`/`Withdrawn` ownership, unknown
IDs, and malformed claim syntax all fail closed, as do unparseable sources, symlinks, missing trees,
duplicate registry IDs, unknown dependencies, dependency cycles, and unknown statuses. Witness
entries must each resolve to a contained workspace file or a collected `#[test]` name; every
`Ratified` rule must be both claimed and witnessed. Coverage is per-rule, not per-file: untagged
infrastructure/derived/oracle code is permitted. Duplicate claims across files are allowed and
attributed (collaborative implementation). Live state from the current CI evidence: 6 implementation tags in 82 files (713 test
functions), clean. RULE-0003's `ErratumCorrected` spelling is now fixed by accepted ADR-0001
and is present in the canonical lifecycle set; zero registry records currently use that
state. Remaining gaps: rule-text to hash binding is procedural (deterministic extraction,
tree-pinned registry bytes); `models/`/`data` emptiness belongs to later milestones.

## Provenance generation (0.0.0.12, `omni-evidence` + driver emission)

Single format authority remains the 0.0.0.8 provenance schema, extended only with optional
`compiler_id`, `compiler_version`, `source_revision` (`-dirty` convention), and `codegen`
fields — old records still validate. Generation is state-separated
(`ProvenanceInputs` → validated → canonical bytes → record) inside `omni-evidence`, reusing the
canon engine; mismatched identity combinations cannot be constructed. Topology refined to match
the 0.0.0.9 specification's own infrastructure category: spec infrastructure
(`canon`/`registry`/`evidence`) is leaf-ward and usable from any tier as counted infra edges but
may never depend outward; `compiler → stage0` is allowed; true tooling stays isolated both ways.
The obsolete driver-only provenance struct is deleted; the driver builds records from actual
inputs (loaded spec identity, pinned toolchain, host descriptor, crate version, git revision or
graceful absence, declared epoch, emitted bytes) and writes a canonical sidecar on native
emission, failing closed when `./spec` is unresolvable. Target descriptors are truthful host
descriptions, not canonical triples (target registry is future work). Full build provenance
generation beyond per-artifact records stays out of scope until release qualification.

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
driver path dependencies, leaving the proven `machine`/`codegen` orchestration interface.

Feed enforcement was previously satisfied in practice only because `omni-machine` happened not to
use its undeclared edges; the gate still reported `omni-machine bypasses its declared feeds (uses
omni-effects)`. The machine legitimately needs the effect vocabulary (`EffectRow`,
`CapabilityContext`) and the type handles (`Ty`, `Lit`, `TypeSpec`) that its MIR program carries, so
both are now re-exported through `omni-mir` — the machine's single declared feed — and the direct
`omni-machine → omni-effects` and `omni-machine → omni-types` edges are removed. This keeps the
`machine: {mir}` feed exact rather than widening it, and matches how `omni-codegen` already depends
on `omni-effects` only from its test targets. Live state from the current topology gate: 25 members, 34 normal + 4 dev + 0 build +
10 infra internal edges, clean.

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
the chain canon → loader → predicates against the live manifest (14 allowed + 8 forbidden).
The native driver integration now reaches the frontend, semantic checking, monomorphization, typed
MIR, MIR verification, and Cranelift object emission through explicit compiler-tier dependencies.
This remains implementation progress only. The technical foundation sequence passed on
`d270e938` and `64f734b9`; formal qualification scope and sign-off remain unresolved under
proposed ADR-0002. The foundation milestone does not qualify the complete compiler or language.

## 0.0.0.13 gate closure (technical checks passed; formal sign-off unresolved)

Lifecycle: ADR-0001 fixes the seventh state spelling as `ErratumCorrected` (normative prose keeps
the hyphenated form); the canonical set lives once in `omni-registry` and is consumed by the
loader, audit, conformance, and evidence validators, with a test asserting the schema file's enum
equals it. Ownership of non-live states stays forbidden; zero records carry the new state.
Rule-text binding: `registry/rule-texts.json` (567 texts) makes every stored hash mechanically
rederivable — the loader rejects stale/mutated/orphan/textless entries, and a test asserts the
texts equal a fresh table-grammar extraction of the normative document.
Grammar: `grammar/candidate2-erratum.md` records the appendix precedence over the Candidate-1
EBNF (citing rule IDs and hashes, inventing no productions), pointed to by a new manifest
`errata` array that the loader resolves and reference-scans; promotion stays blocked per the
amendment's own ratification gate.
Discovery: `omni-registry` discovery (explicit `OMNI_SPEC_ROOT`, else manifest-anchored ancestor
walk) is used by all four binaries and the driver; CWD-independence and layout strictness are
tested, including nested-directory and outside-tree cases.
Target/host: provenance records label `target_source` (`host` vs `explicit`); the driver always
emits `host`, and a purity test proves the descriptor equals the two build consts exactly.
Models/data: the manifest declares module names, never model/data files, so the loader's explicit
empty states are contract-consistent (tested).

The current canonical digest is `3986c2a71d9e5e4092331d025079055f34b98aa4179de1684b64e28bde74e6ed`.
The foundation-gate JSON still declares `qualified`, but master mandate §68 explicitly says
not to infer qualification from that field alone. `omni-conform` now requires the declared
status to be `qualified` and `blocked_reasons` to be empty; this is a consistency check, not
independent sign-off. The CI technical checks passed on `d270e938` and `64f734b9` (including
the canonical digest and gate step). The evidence corpus remains empty, all 567 rules remain
Candidate, and there are zero E-code diagnostics and zero rule-witness records. Formal
qualification remains unresolved pending the scope/evidence/sign-off decision in ADR-0002.
No product release or complete-language qualification is implied.

## Unicode data binding (SRC-0004) and the SRC-0006 annotation syntax

`SRC-0004` pins Unicode 17.0.0 together with UAX #15, #31, #9, #24, #44 and
UTS #39 Revision 32, bound "through the release reference manifest and SHA-256
digests". That binding is implemented as committed data, not as a dependency on
whatever version a crate happens to ship:

- `compiler/omni-unicode/src/tables.rs` is generated by
  `scripts/generate-unicode-tables.py` from the UCD 17.0.0 `PropList.txt`,
  `DerivedCoreProperties.txt`, `DerivedNormalizationProps.txt`, and
  `UnicodeData.txt`. All four digests are compiled into the table module as
  individual constants and as a single `UCD_DIGESTS` table, and a test asserts
  that the table lists exactly the consumed inputs and agrees with each
  individual constant. The tables therefore cannot drift from the UCD they
  claim to encode, and a newly consumed input file cannot be added without also
  being digested.
- `compiler/omni-unicode/tests/nfc_cases.rs` is generated from the same UCD and
  holds 961 canonical-decomposition cases that must recompose under UAX #15 NFC.
  This is what pins the `unicode-normalization` dependency, which publishes no
  Unicode version of its own, to the Edition 1 data.
  `Full_Composition_Exclusion` is read from the derived
  `DerivedNormalizationProps.txt` rather than the primary
  `CompositionExclusions.txt`: the primary list deliberately omits non-starter
  decompositions, so using it would wrongly admit characters such as U+0344 into
  the corpus, where they do not recompose.
- The compiler performs no network access and reads no host locale or platform
  Unicode data. Classification is a binary search over committed tables, so it
  is identical on every host and in every process.
- Regeneration is idempotent: running the generator twice reproduces both
  outputs byte-for-byte.

### Recorded specification gap: the SRC-0006 annotation syntax

`SRC-0006` requires that a bidi control or invisible format character inside a
comment "require an escaped visible annotation generated by the formatter", but
does not fix the syntax of that annotation. An implementation must therefore
choose one, and the choice is recorded here rather than being made silently in
code:

- The annotation reuses the existing normative `escape_sequence` production from
  LEX-0007, i.e. `\u{202E}`. No new production is invented, the surface language
  stays closed, the annotation is pure ASCII and therefore visible in any
  terminal or log, and it is unambiguous against an adjacent literal backslash.
- `omni_unicode::annotation::annotate_comment` is the transformation the rule
  calls "generated by the formatter". It is opt-in: the lexer never applies it,
  and it is idempotent, so a formatter cannot double-escape its own output. The
  repository has no formatter crate yet (`OMNI-FMT` is a declared module with no
  implementation), so the generator lives here and a future formatter consumes
  it.
- The scope boundary is also recorded: `SRC-0006` names exactly two classes,
  bidi controls and invisible format characters. Noncharacters and unassigned
  code points remain `SRC-0005` classes and are not reported by the comment
  path.

Binding the annotation syntax into the normative grammar, so that it is fixed by
the specification rather than by this record, is deferred to the grammar
milestone alongside the still-open `VIBE-GRAM-*` reconciliation noted above.