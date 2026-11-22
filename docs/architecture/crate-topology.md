# Omni Crate Architecture Map

**Repository:** C:\Users\siddh\Downloads\ABC\Omni---New  
**Branch:** main  
**Source baseline:** remote `main` at `d076b3d` (2026-11-22)  
**Updated:** 2026-11-22  
**Source:** Cargo workspace manifests; 31 members after isolating C ABI symbol exports from the safe runtime

---

## Purpose

This document maps the crate dependency graph and the source-to-execution pipeline
for the Omni Edition 1 compiler. It is an audit artifact produced during the
repository audit phase and must be kept consistent with the live `Cargo.toml`
manifests. It does not modify any normative specification, grammar, manifest,
or release gate.

---

## Workspace Membership (31 crates)

| Tier | Crate | Path | Role | Cargo.toml deps |
|------|-------|------|------|-----------------|
| **source** | omni-source | `compiler/omni-source/` | Source cursor, BOM handling, UTF-8 validation | (none) |
| **lex** | omni-unicode | `compiler/omni-unicode/` | Unicode 17.0.0 tables, normalization, classification | unicode-normalization, unicode-ident |
| **lex** | omni-lex | `compiler/omni-lex/` | Lexer, tokenization, security classification | omni-source, omni-unicode |
| **syntax** | omni-syntax | `compiler/omni-syntax/` | CST green tree, syntax node kinds | rowan, omni-lex |
| **parse** | omni-parse | `compiler/omni-parse/` | Pratt parser, recovery, experimental profiles | omni-syntax, omni-lex, omni-source, rowan |
| **names** | omni-names | `compiler/omni-names/` | Name resolution, NFC canonicalization | omni-syntax, omni-unicode |
| **types** | omni-types | `compiler/omni-types/` | TyCtxt, TyKind enum, TypeSpec lowering | ena, omni-effects |
| **own** | omni-own | `compiler/omni-own/` | Affine ownership state machine, loans, access kinds | omni-types |
| **effects** | omni-effects | `compiler/omni-effects/` | Effect rows, capabilities, obligation solving | (none) |
| **traits** | omni-traits | `compiler/omni-traits/` | Trait coherence, SLG resolution | index_vec, rustc-hash, omni-types, omni-effects |
| **hir** | omni-hir | `compiler/omni-hir/` | High-level IR, concrete program gate | omni-types, omni-own, omni-effects |
| **mir** | omni-mir | `compiler/omni-mir/` | MIR IR, Place/Projection model, lowering context | index_vec, omni-types, omni-hir, omni-effects |
| **verify** | omni-verify | `compiler/omni-verify/` | MIR structural verification, ownership/borrow checking | index_vec, polonius-engine, omni-mir, omni-own |
| **machine** | omni-machine | `compiler/omni-machine/` | Reference abstract machine interpreter | omni-mir |
| **codegen** | omni-codegen | `compiler/omni-codegen/` | Cranelift native codegen, indirect call emission | omni-mir, omni-verify, cranelift-* |
| **runtime** | omni-runtime | `compiler/omni-runtime/` | Safe Rust runtime services, allocation and process-local state | (none) |
| **runtime integration** | omni-runtime-ffi | `compiler/omni-runtime-ffi/` | Narrow C ABI symbol-export boundary; unsafe attributes isolated here | omni-runtime |
| **std** | omni-std | `compiler/omni-std/` | Standard library foundation on runtime services | omni-runtime |
| **driver** | omni-driver | `compiler/omni-driver/` | CLI orchestrator, source-to-execute pipeline | omni-parse, omni-syntax, omni-names, omni-types, omni-hir, omni-traits, omni-effects, omni-mir, omni-verify, omni-machine, omni-codegen, omni-evidence, omni-registry, omni-stage0 |
| **pkg** | omni-pkg | `compiler/omni-pkg/` | Package management | omni-types, omni-hir, omni-mir |
| **build** | omni-build | `compiler/omni-build/` | Build orchestration | omni-types, omni-hir, omni-mir, omni-codegen |
| **ls** | omni-ls | `compiler/omni-ls/` | Language server | cranelift-*, omni-types, omni-hir, omni-verify |
| **stage0** | omni-stage0 | `compiler/omni-stage0/` | Stage-0 predicate engine, bootstrap predicates | (dev: omni-registry) |
| **canon** | omni-canon | `tools/omni-canon/` | Canonicalization, spec-tree digest | serde, serde_json, sha2, hex, walkdir |
| **registry** | omni-registry | `tools/omni-registry/` | Specification loader, rule registry | omni-canon, serde, serde_json, walkdir |
| **evidence** | omni-evidence | `tools/omni-evidence/` | Evidence/provenance record generation | omni-canon, omni-registry, serde, serde_json |
| **conform** | omni-conform | `tools/omni-conform/` | Conformance validation, release bindings | omni-canon, omni-evidence, omni-registry, serde, serde_json, walkdir |
| **audit** | omni-audit | `tools/omni-audit/` | Linkage reaper, claim validation | omni-registry, serde, serde_json, syn, walkdir |
| **topology** | omni-topology | `tools/omni-topology/` | Compiler topology validation | serde, serde_json |
| **fmt** | omni-fmt | `tools/omni-fmt/` | CST-based formatter, --check and stdin modes | omni-parse, omni-syntax |
| **bindgen** | omni-bindgen | `tools/omni-bindgen/` | Bindings generator (stub) | (none) |

---

## Dependency Graph (crate-to-crate edges)

```
omni-source
  └── (leaf)

omni-unicode
  └── (leaf)

omni-lex ───────────▶ omni-source
omni-lex ───────────▶ omni-unicode

omni-syntax ───────▶ rowan (external)
omni-syntax ───────▶ omni-lex

omni-parse ────────▶ omni-syntax
omni-parse ────────▶ omni-lex
omni-parse ────────▶ omni-source
omni-parse ────────▶ rowan (external)

omni-names ────────▶ omni-syntax
omni-names ────────▶ omni-unicode

omni-types ────────▶ ena (external)
omni-types ────────▶ omni-effects

omni-own ──────────▶ omni-types

omni-effects ──────▶ (leaf)

omni-traits ───────▶ index_vec (external)
omni-traits ───────▶ rustc-hash (external)
omni-traits ───────▶ omni-types
omni-traits ───────▶ omni-effects

omni-hir ──────────▶ omni-types
omni-hir ──────────▶ omni-own
omni-hir ──────────▶ omni-effects

omni-mir ──────────▶ index_vec (external)
omni-mir ──────────▶ omni-types
omni-mir ──────────▶ omni-hir
omni-mir ──────────▶ omni-effects

omni-verify ───────▶ index_vec (external)
omni-verify ───────▶ polonius-engine (external)
omni-verify ───────▶ omni-mir
omni-verify ───────▶ omni-own

omni-machine ──────▶ omni-mir

omni-codegen ──────▶ omni-mir
omni-codegen ──────▶ omni-verify
omni-codegen ───────┬▶ cranelift-codegen (external)
omni-codegen ───────┬▶ cranelift-frontend (external)
omni-codegen ───────┬▶ cranelift-entity (external)
omni-codegen ───────┬▶ cranelift-module (external)
omni-codegen ───────┬▶ cranelift-object (external)
omni-codegen ───────┬▶ target-lexicon (external)
omni-codegen ──dev▶ omni-effects (dev-only)

omni-runtime ──────▶ (leaf)  [crate-type: rlib]
omni-runtime-ffi ──▶ omni-runtime  [crate-type: rlib, staticlib, cdylib]

omni-std ──────────▶ omni-runtime  [crate-type: rlib, staticlib]

omni-driver ───────▶ omni-parse
omni-driver ───────▶ omni-syntax
omni-driver ───────▶ omni-names
omni-driver ───────▶ omni-types
omni-driver ───────▶ omni-hir
omni-driver ───────▶ omni-traits
omni-driver ───────▶ omni-effects
omni-driver ───────▶ omni-mir
omni-driver ───────▶ omni-verify
omni-driver ───────▶ omni-machine
omni-driver ───────▶ omni-codegen
omni-driver ───────▶ omni-evidence
omni-driver ───────▶ omni-registry
omni-driver ───────▶ omni-stage0
omni-driver ──dev▶ omni-canon
omni-driver ──dev▶ serde_json

omni-pkg ──────────▶ omni-types
omni-pkg ──────────▶ omni-hir
omni-pkg ──────────▶ omni-mir

omni-build ────────▶ omni-types
omni-build ────────▶ omni-hir
omni-build ────────▶ omni-mir
omni-build ────────▶ omni-codegen

omni-ls ───────────▶ cranelift-codegen (external)
omni-ls ───────────▶ cranelift-module (external)
omni-ls ───────────▶ cranelift-frontend (external)
omni-ls ───────────▶ cranelift-object (external)
omni-ls ───────────▶ omni-types
omni-ls ───────────▶ omni-hir
omni-ls ───────────▶ omni-verify

omni-stage0 ──dev▶ omni-registry

omni-fmt ──────────▶ omni-parse
omni-fmt ──────────▶ omni-syntax

omni-canon ────────▶ serde, serde_json, sha2, hex, walkdir

omni-registry ─────▶ omni-canon
omni-registry ─────▶ serde, serde_json, walkdir

omni-evidence ─────▶ omni-canon
omni-evidence ─────▶ omni-registry
omni-evidence ─────▶ serde, serde_json

omni-conform ──────▶ omni-canon
omni-conform ──────▶ omni-evidence
omni-conform ──────▶ omni-registry
omni-conform ─────▶ serde, serde_json, walkdir

omni-audit ────────▶ omni-registry
omni-audit ────────▶ serde, serde_json, syn, walkdir

omni-topology ─────▶ serde, serde_json

omni-bindgen ──────▶ (leaf) [stub]
```

---

## Source-to-Execution Pipeline

Per `docs/foundation-gates.md` §Compiler topology (0.0.0.9), pipeline tiers are:

```
source(0) → lex(1) → syntax(2) → parse(3) → names(4) → types(5) →
own/effects/traits(6) → hir(7) → mir(8) → verify(9) →
machine/codegen(10) → runtime(11) → driver(12 orchestrates)
```

The driver (`omni-driver`) is tier 12 and orchestrates all tiers. It may invoke
frontend, semantic, monomorphization, MIR, verification, machine, and native
backend tiers directly. Lower compiler tiers never depend back on the driver.

Feed enforcement (exact, per topology gate):
- `machine: {mir}` — omni-machine consumes only omni-mir
- `codegen: {mir, verify}` — omni-codegen consumes omni-mir and omni-verify
- `verify: {mir}` — omni-verify consumes omni-mir

Dev/build edges are direction-exempt but cycle-checked. `stage0` is edgeless
bootstrap infrastructure. `tools/*` may use tools/externals but never
`compiler/*`, and vice versa.

---

## Crate Tier Classification

| Tier | Crates | Direction constraint |
|------|--------|---------------------|
| source (0) | omni-source | Leaf |
| lex (1) | omni-unicode, omni-lex | May depend on lower tiers only |
| syntax (2) | omni-syntax | May depend on lower tiers only |
| parse (3) | omni-parse | May depend on lower tiers only |
| names (4) | omni-names | May depend on lower tiers only |
| types (5) | omni-types | May depend on lower tiers only |
| own/effects/traits (6) | omni-own, omni-effects, omni-traits | May depend on lower tiers only |
| hir (7) | omni-hir | May depend on lower tiers only |
| mir (8) | omni-mir | May depend on lower tiers only |
| verify (9) | omni-verify | May depend on lower tiers only |
| machine/codegen (10) | omni-machine, omni-codegen | May depend on same or earlier tier |
| runtime (11) | omni-runtime | May depend on same or earlier tier |
| driver (12) | omni-driver | May invoke any tier |
| integration (12) | omni-pkg, omni-build, omni-ls, omni-fmt | Post-pipeline tools; consume lower compiler tiers only |
| compiler infrastructure | omni-stage0 | Bootstrap predicates; no production dependency edges |
| specification tooling | omni-canon, omni-registry, omni-evidence, omni-conform, omni-audit, omni-topology, omni-bindgen | Leaf-ward specification/repository tools |
| spec-infra | omni-canon, omni-registry, omni-evidence, omni-conform, omni-audit, omni-topology | Leaf-ward; may never depend outward on compiler/* |

---

## Boundaries

- **Specification tools → Compiler:** `omni-canon`, `omni-registry`, `omni-evidence`,
  `omni-conform`, `omni-audit`, `omni-topology` and `omni-bindgen` must not
  depend on compiler crates. The compiler-facing formatter `omni-fmt` is an
  explicit post-pipeline integration (tier 12) because it consumes `omni-parse`
  and `omni-syntax`; it must not be treated as specification infrastructure.
- **Driver/integrations:** `omni-driver`, `omni-pkg`, `omni-build`, `omni-ls`,
  `omni-fmt` and `omni-runtime-ffi` may consume lower compiler tiers but no lower
  tier may depend on them.
- **Runtime safety boundary:** `omni-runtime` inherits workspace `unsafe_code = "forbid"`.
  The dedicated `omni-runtime-ffi` crate keeps `unsafe_code = "deny"` and permits
  only item-scoped lint exceptions for the `#[unsafe(no_mangle)]` exports; it
  contains no unsafe blocks or unsafe functions.
- **Compiler → Tools:** `compiler/*` may NOT depend on `tools/*` except
  `omni-driver → omni-evidence` and `omni-driver → omni-registry` (provenance).
- **Driver:** Orchestrates all compiler tiers but lower tiers never depend back
  on it.
- **unsafe_code:** `forbid` at workspace level; no `unsafe` permitted.
- **Clippy:** `all = "deny"` at workspace level; zero warnings required.

---

## Validation Evidence

This document is an architecture map, not a qualification record. Build, lint, test,
conformance and native-execution evidence must be tied to the exact Git commit and
GitHub Actions run that produced it. The earlier 25-member workspace report is
superseded by the 30-member manifest after restoring `omni-std`; consult the current
workflow results rather than treating historical exit codes as evidence for later commits.
