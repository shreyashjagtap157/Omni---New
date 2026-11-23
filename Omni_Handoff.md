# Omni — Local Continuation Handoff

**Checkpoint:** 2026-11-23  
**Repository:** https://github.com/shreyashjagtap157/Omni---New  
**Branch policy:** \`main\` only; no feature branches  
**Remote HEAD when prepared:** \`d324a85d9fc6189a34eec5177908397018adcf35\`  
**Expected local path:** \`C:\Users\siddh\Downloads\ABC\Omni---New\`  
**Pinned Rust:** 1.95.0; workspace currently has 31 members (verify from manifest).

> This document supersedes older status notes in this file. The current native indirect-call implementation is committed but not validated; do not treat the current CI failure as a pass.

## Mission

Complete and qualify Omni Edition 1 as a coherent, integrated language/compiler/runtime/standard-library/tooling system, following the repository's normative specifications and the owner's previously recorded requirements. Research while implementing: inspect the actual specification and dependency path, identify what is required, implement a complete vertical slice, test it, integrate it, and update evidence. Do not stop at plans or placeholder crates.

Roadmap remains Rust-first production implementation, followed by an independently qualified Omni-written self-hosting compiler/toolchain. These are targets, not completion claims.

## Synchronize local repository first

Use PowerShell. Inspect before updating; preserve local work. Never reset or clean away uncommitted files.

\`\`\`powershell
$ErrorActionPreference = 'Stop'
$Repo = 'C:\Users\siddh\Downloads\ABC\Omni---New'
Set-Location $Repo

git status --short --branch
git remote -v
git branch --show-current
git log -1 --oneline --decorate

if ((git status --porcelain).Count -ne 0) {
    throw 'Working tree is dirty. Inspect and preserve local changes before syncing.'
}

git fetch origin --prune
if ((git branch --show-current) -ne 'main') {
    git switch main
}
git pull --ff-only origin main

git status --short --branch
git rev-parse HEAD
git rev-parse origin/main
git log -20 --oneline --decorate --graph
git diff --stat origin/main...HEAD
\`\`\`

The remote may have advanced since the handoff; fetched \`origin/main\` is authoritative. If the fast-forward fails due to divergence, stop and inspect. Do not reset, rebase, force-push or create another branch.

## Build-output cleanup requirement

Before compiling/testing a new implementation locally:
1. Inspect \`.cargo/config.toml\`, \`.cargo/config\`, workspace settings and \`CARGO_TARGET_DIR\` to identify the real target directory.
2. Remove stale Cargo test/build output first with \`cargo clean\`; verify the configured target directory was removed.
3. Inspect any root \`build/\` or other project output directory. Delete it only when confirmed to be generated output. Never delete source or user data just because a directory is named \`build\`.
4. Rebuild from current sources; do not reuse stale test binaries or object files as evidence.
5. Keep temporary objects, test sources, executables and logs under the OS temp directory and delete them. Do not add large generated files to the repo. Clean output again after a completed local milestone when practical.
6. Never use \`git clean -xfd\` as a shortcut.

## Work already committed on remote main

The linear history through \`d324a85\` includes:
- Memory-model regression correction for moved source places and alias retries.
- Fixed-denominator, fail-closed inventory validator; exact IDs, 4-digit ADR support, detail-table/count consistency; regression tests integrated with CI.
- Inventory corrected to 26 mandatory items: 23 Verified / COMP-019 Partial / ADR-0002 Accepted / INST-002 Blocked. COMP-017 is a duplicate of COMP-011; COMP-018 is unregistered. Readiness arithmetic includes Partial and does not grant qualification.
- Eleven Edition 1 conformance cases registered as an \`omni-driver\` Cargo test target. These passed on the earlier three-OS run before the current function-pointer work.
- Typed function-pointer work: exact FnPtr signature derivation in MIR verification, typed indirect-call argument/result checking, pointer-sized native ABI/storage, Cranelift indirect-call emission and module-managed function-address relocation.
- New native regression inspects a real object and exported entry symbol; \`object 0.36.7\` is a dev dependency with platform object readers enabled.

Key commits, all ancestors of the checkpoint: \`0cc9658\`, \`73737ab\`, \`82c7568\`, \`d45f96f\`, \`c5a146c\`, \`9c302c3\`, \`cafcfcc\`, \`8476001\`, \`1289f13\`, \`1681faf\`, \`0c931d7\`, \`1bd6b4c\`, \`275a183\`, \`a941654\`, \`78d7a5c\`, \`d324a85\`. Inspect the full history and patches; this list is only an index.

## Exact current CI state

At \`d324a85d9fc6189a34eec5177908397018adcf35\`:

- [Omni CI Pipeline #45138997819](https://github.com/shreyashjagtap157/Omni---New/actions/runs/45138997819): **failed** on Windows, Linux and macOS.
- [Rust #45138997923](https://github.com/shreyashjagtap157/Omni---New/actions/runs/45138997923): build passed; tests failed.
- [Strict Clippy #45138997860](https://github.com/shreyashjagtap157/Omni---New/actions/runs/45138997860): passed.
- [CodeQL #45138997855](https://github.com/shreyashjagtap157/Omni---New/actions/runs/45138997855): passed.
- [Security Audit #45138997796](https://github.com/shreyashjagtap157/Omni---New/actions/runs/45138997796): passed.
- Informational readiness step ran, but explicitly reported **NOT PASSED**.

Two concrete blockers:

**A. Formatting.** All three platform CI jobs stop at \`cargo fmt --check\` because of a rustfmt diff in \`compiler/omni-codegen/src/lib.rs\`, around the new \`object_prefix\` diagnostic (approximately line 3064). Apply the pinned Rust 1.95.0 formatter, inspect its diff, and commit the canonical formatting. Do not skip or weaken the formatting gate.

**B. Zero-byte native object.** The Rust workflow builds but reports 29 passing and 1 failing \`omni-codegen\` test. Failing test:
\`tests::test_indirect_function_pointer_call_verifies_emits_and_executes\`.
It panics when parsing the emitted object:
\`native object must be parseable (0 bytes, prefix []): Could not read file magic\`.

The function returns an empty byte vector before symbol inspection, linking or execution. The regression therefore does **not** yet prove a functioning linked indirect call. Trace the entire path:
- \`compile_verified_mir_program\` and verifier gates;
- Cranelift object module construction and function declaration/definition;
- local function linkage, \`FnRef\` lowering and module-managed relocation;
- indirect-call signature/ABI and result handling;
- module finalization and exact byte-return path;
- test fixture/target/module selection.

Acceptance is a non-empty valid native object, the expected exported entry symbol, a valid relocation to the indirect callee, successful link with the platform toolchain, and execution returning 42. Do not accept zero bytes, fake symbols, skip parsing/linking, or reduce this to object-size-only validation. Make platform differences explicit and preserve Windows/Linux/macOS coverage.

Memory-model suites passed in the latest failed CI (12/12 and 26/26); strict inventory tests passed (7/7). Earlier full three-OS baseline [#45134034709](https://github.com/shreyashjagtap157/Omni---New/actions/runs/45134034709) was green at \`cafcfcc\`, before function-pointer changes.

## Inventory and documentation caveats

Current \`IMPLEMENTATION_INVENTORY.md\` states:

| Status | Count | Records |
|---|---:|---|
| Verified | 23 | COMP-001–016, COMP-020–024, INST-003, PLAT-001 |
| Partial | 1 | COMP-019 |
| Accepted | 1 | ADR-0002 |
| Blocked | 1 | INST-002 |
| **Mandatory total** | **26** | |

Re-audit these claims before treating them as final:
- **COMP-019:** \`omni-pkg\` and \`omni-build\` entry points are TODO-only placeholders. Cargo compilation does not prove package management/build orchestration; keep Partial until behavior and acceptance tests exist.
- **COMP-011:** native indirect-call claim is under requalification while the zero-byte end-to-end object test fails. Presence of \`call_indirect\` alone is not end-to-end proof.
- **COMP-021:** inventory says Verified, but the older handoff says an actual RC cycle and RC binary distribution are required. Audit \`tools/omni-installer/rc-stabilize.ps1\`, clean-tree enforcement, actual workflow executions, retained artifacts and evidence. A script or hard-coded PASS report is not sufficient; correct the status if RC proof is absent.
- **COMP-022:** distinguish automated installer checks from clean non-admin install/upgrade/repair/uninstall evidence on an independent Windows environment.
- **INST-002:** blocked until a genuine trusted production Authenticode identity/signing service is owner-provisioned and real release artifacts pass publisher/chain/trust verification. Do not create self-signed substitutes or handle private keys.
- **ADR-0002:** Alternative C is accepted, but formal owner-attested qualification remains outstanding. Never fabricate or reuse a qualification record across commits.
- COMP-017 is a duplicate of COMP-011; COMP-018 is not registered. Both remain excluded from the 26-item denominator unless the authoritative registry formally changes.

README currently names old baseline \`82c7568\`; refresh its status paragraph and other stale docs only from verified current evidence. Keep historical snapshots clearly labelled.

## Local execution sequence

1. Synchronize local \`main\` as above; inspect local status, exact HEAD, remote alignment, recent commit history and every changed path since the local baseline.
2. Read the complete affected source and the authoritative spec material before semantic changes:
   - \`spec/grammar/omni-edition1.ebnf\`;
   - \`spec/registry/rules.json\`, \`spec/registry/rule-texts.json\`;
   - edition manifest, schemas, ADRs, conformance cases and architecture constraints;
   - inventory, discrepancy register, implementation plans and current CI workflows.
   Normative Edition 1 grammar is authoritative. Do not invent behavior from unratified candidates or fill specification gaps with local semantics.
3. Enumerate the tracked repository and audit complete affected dependency paths across compiler, runtime, stdlib, tooling, tests, manifests, lockfile, workflows and docs.
4. Run \`cargo clean\` before fresh local compilation/testing (after inspecting the target directory); do not reuse stale output.
5. Fix formatting. Then run targeted verifier/codegen tests, especially the failing FnPtr test.
6. Resolve zero-byte emission at root cause and prove object format/symbol/relocation/link/execution.
7. Run full gates:
   - \`cargo fmt --all -- --check\`
   - \`cargo check --workspace --locked\`
   - \`cargo build --workspace --locked\`
   - \`cargo clippy --workspace --all-targets --locked -- -D warnings\`
   - \`cargo test --workspace --locked\`
   - locked metadata/dependency and specification/architecture validators;
   - external C API consumer tests;
   - isolated memory-model tests;
   - strict inventory validator tests.
8. Commit coherent tested milestones to \`main\`, push normally (never force), then inspect GitHub Actions logs/artifacts for Windows, Linux and macOS plus Rust, Clippy, CodeQL and Security Audit.
9. Correct inventory, README, completion report and architecture docs using measured evidence. Report verified facts, decisions, implementation, validation, blockers and open questions separately.
10. Continue implementation and research as one integrated system; do not create a large collection of local reports or generated artifacts.

## Non-negotiable operating rules

- \`main\` only; no feature branches; preserve commit history; never force-push.
- No Sprites. Use GitHub Actions for hosted OS builds/tests.
- No yolo/no blind deletion; preserve user changes and local data.
- No speculative language semantics; adhere to normative Edition 1 and its registered requirements.
- Rust-first complete implementation, then qualified self-hosting.
- Do not declare success without actual tests/builds/link/run/CI evidence.
- Keep temporary outputs minimal and remove stale target/build outputs before local testing.

**Next milestone:** synchronize the user's local repository to the then-current \`origin/main\`, record its state, clean stale build output, fix formatting and the zero-byte native object failure, then get the complete multi-OS workflow green before moving to the next specification-backed language subsystem.
