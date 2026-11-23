Classification: Informative

> HISTORICAL LOCAL-WORKSPACE CONTINUATION MANDATE — retained as source provenance. It predates the current explicit authorization to publish this documentation update. Its local-workspace preservation, no-force-push, installer-safety, signing-secret, test-evidence and handoff constraints remain useful; its remote baseline, working-tree status and authorization wording are historical and must not be treated as current state.

---

# Omni — Local Workspace Recovery, Reconciliation and Continuous Engineering Mandate

**Prepared:** 21 November 2026  
**Canonical repository:** `https://github.com/shreyashjagtap157/Omni---New`  
**Canonical branch:** `main` only  
**Current remote `main` baseline observed:** `b89af1a14a093e7f94b7ae097340c649731691a8`  
**Primary objective:** Recover and validate the real local implementation, especially the installer work, reconcile it with the canonical remote branch, and continue engineering without losing local work or overstating qualification.

---

## 0. Operating instruction — continue autonomously

You are operating inside the owner's actual local Windows development environment. The owner is actively working locally. Treat the current workspace as potentially containing valuable uncommitted, untracked, ignored, or locally committed implementation that is not present on GitHub.

This is an execution mandate, not a request for another plan. Work continuously through every unblocked task. After completing one coherent task, inspect its consumers, run relevant tests, update documentation/evidence, and immediately select the next ready task. Do not stop after an audit, a plan, one repaired workflow, or one green CI run. Do not repeatedly ask the owner to say “continue”.

Ask the owner only when a genuine owner-only decision or resource is required, such as a production signing identity, actual formal attestation, a paid purchase, a normative language decision, a destructive action, or a remote publication explicitly requiring authorization. Record the exact blocker once and continue all independent engineering.

### Authority and boundaries

- Local repository edits and local validation are authorized by this mandate.
- Do not push, publish a release, create a tag, or otherwise mutate the remote repository unless the owner explicitly authorizes that specific action.
- Do not force-push, rewrite history, reset hard, clean, discard, overwrite, or silently stash any owner work.
- Do not create feature branches. Omni work stays on `main`. If the current local branch is not `main`, inspect and report why before switching; do not switch away from a dirty or valuable worktree.
- Preserve all history. Integrate by normal fast-forward or ordinary commits only where appropriate and authorized by the repository workflow.
- Never delete local-only files just because they are absent from remote `main`.
- Never put private keys, PFX/P12 files, signing passwords, recovery codes, access tokens, or certificate secrets in Git, logs, test artifacts, chat, or reports.
- Never invent owner approval, signatures, qualification, test output, or implementation evidence.

---

## 1. Verified remote baseline and current CI evidence

Before starting, refresh the live remote state. The following is the latest verified baseline from the preceding audit; it is not permission to skip rechecking it.

### Remote `main`

- Repository: `shreyashjagtap157/Omni---New`
- Observed HEAD: `b89af1a14a093e7f94b7ae097340c649731691a8`
- Commit message: `docs: record remote-main first-stage gate reconciliation`
- Current `main` includes:
  - repaired `.github/workflows/ci.yml`;
  - portable timeout handling and explicit workspace build in `ci/pipeline.sh`;
  - a remote-main evidence warning in `IMPLEMENTATION_INVENTORY.md`;
  - `docs/implementation/remote-main-gate-reconciliation-2026-11-21.md`.

### Latest full CI matrix

Run `44873961209` on commit `b89af1a14a093e7f94b7ae097340c649731691a8` completed successfully:

- Windows mandatory validation: PASS.
- Linux experimental compatibility: PASS.
- macOS experimental compatibility: PASS.
- Informational first-stage readiness report: job succeeded, but its **reported gate result is NOT PASSED**.
- CodeQL Advanced: PASS (`44873961216`).
- Security Audit: PASS (`44873961224`).
- Standard Rust workflow: PASS (`44873961242`).
- Rust Clippy analysis workflow: PASS (`44873961246`).

The native validation matrix performs the pinned Rust 1.95.0 toolchain assertion, formatting, workspace check, workspace build, strict Clippy, workspace tests, locked metadata/dependency-tree checks and the repository's `omni-conform`, `omni-audit` and `omni-topology` checks.

**Important:** green technical CI does not prove the product inventory, installer, release signing, first-stage gate, or complete language are qualified. Do not report the first-stage gate as passed.

### Exact blockers reported by the latest readiness job

The job output on commit `b89af1a...` reports:

1. Absent remote paths:
   - `compiler/omni-std`
   - `tools/omni-fmt`
   - `compiler/omni-pkg`
   - `compiler/omni-build`
   - `tools/omni-installer`
   - `tools/omni-installer/INSTALLER_QUALIFICATION.md`
   - `tools/omni-installer/installer-qualify.ps1`
   - `tools/omni-driver/completion.ps1`
   - `tools/omni-driver/completion.cmd`
   - `include/omni-rt.h`
2. Inventory summary says 24 Verified / 1 Accepted / 1 Blocked / 26 Total, but its Verified ID list expands to 25 IDs.
3. The detailed Verified table has 24 rows and omits `COMP-020` from the list.
4. `25 + 1 + 1 != 26`.
5. `COMP-018` is shown as N/A although the inventory claims 26 mandatory items.
6. `COMP-015` claim is not substantiated: the current remote driver source does not contain a `--run` option.
7. `COMP-011`/`COMP-017` claim is not substantiated by the current codegen source: no `call_indirect` emission token was found.
8. `PLAT-001` claim is not substantiated: the runtime source has no `extern "C"` export and `include/omni-rt.h` is absent.
9. `INST-002` still requires a trusted production Authenticode publisher identity and clean-Windows trust verification.
10. `ADR-0002` formal qualification still requires a truthful exact-commit record backed by passing evidence and actual owner approval.

The readiness workflow was hardened to distinguish the required Windows platform from informational Linux/macOS compatibility, compare unique IDs against detailed inventory rows, and inspect several source claims. Inspect the current script and its latest run output before making changes.

---

## 2. Phase 0 — preserve and inventory the actual local workspace

**This phase is mandatory. Do not begin restoration or checkout operations until it is complete.**

### 2.1 Establish repository identity and state

Run these read-only commands first in PowerShell from the current working directory:

```powershell
git rev-parse --show-toplevel
git rev-parse --show-prefix
git status --short --branch
git status --porcelain=v2 --branch
git branch --show-current
git rev-parse HEAD
git log -1 --format="%H%n%ad%n%s" --date=iso-strict
git remote -v
git branch -vv
```

Confirm the root is the canonical Omni repository. Do not assume the shell is already in the right directory. If it is not, locate the repository without modifying any candidate directory.

### 2.2 Protect local state before syncing

- Record the full output of the commands above in a dated local audit log outside the repository or in a temporary audit directory.
- Enumerate modified, staged, untracked and ignored files. Use `git status --ignored --short` and targeted `git check-ignore -v` for candidate source files.
- Do not use `git clean`, `git reset`, `git checkout --`, `git restore`, `git stash`, or overwrite operations as shortcuts.
- Preserve any local-only source in place. If a safety copy is necessary, copy source files to a clearly named recovery directory outside the repository; do not copy build output, caches, certificate packages or secrets.
- Preserve locally created commits and inspect all local branches/tags without switching them.
- Do not stage all files with `git add -A`.

### 2.3 Fetch remote `main` without changing the checked-out worktree

After recording the pre-fetch state:

```powershell
git fetch --prune origin main
git rev-parse origin/main
git log --oneline --decorate -12 origin/main
git log --left-right --cherry-pick --oneline HEAD...origin/main
```

Fetching is allowed; checking out, resetting or merging is not automatic. Confirm the fetched `origin/main` commit is the current canonical commit. The expected starting point at mandate creation was `b89af1a14a093e7f94b7ae097340c649731691a8`, but remote may have advanced.

### 2.4 Build a local-versus-remote file map

For each required path, classify it as:

- tracked in local HEAD and remote `main`;
- tracked locally but not in remote `main`;
- committed locally in a descendant commit not yet on remote;
- modified in the local worktree;
- untracked local file;
- ignored local file;
- absent locally and remotely;
- moved/renamed to a different path;
- superseded by a documented replacement.

At minimum inspect:

```text
tools/omni-installer/
tools/omni-installer/installer-qualify.ps1
tools/omni-installer/INSTALLER_QUALIFICATION.md
ci/gate-0-0-0-14.sh
ci/gate-0-0-0-13.sh
ci/pipeline.sh
ci/assert-toolchain.ps1
ci/assert-toolchain.sh
compiler/omni-std/
tools/omni-fmt/
compiler/omni-pkg/
compiler/omni-build/
compiler/omni-driver/
tools/omni-driver/
include/omni-rt.h
```

Use `git ls-files`, `Get-ChildItem -Force -Recurse`, `git log --all -- <path>`, and `git diff` carefully. Record exact file paths, blob hashes where available, local modification status, remote existence and the source of each file.

**Do not infer that a missing remote file is missing locally. The owner has reported that installer artifacts may exist locally. The purpose of this mandate is to find and preserve them.**

---

## 3. Phase 1 — recover original installer and qualification implementation

### 3.1 If local installer files exist

Treat the existing local implementation as the recovery source. Inspect the complete directory, not only the named entry point. Include related modules, tests, fixtures, documentation, manifests and workflow references.

Before editing:

1. Record each file, size, hash and Git state.
2. Read the installer entry point and every imported/helper script.
3. Trace how install, uninstall, upgrade, repair, PATH changes, file ownership, checksum/SBOM generation and signing are implemented.
4. Inspect tests for what they actually exercise and whether they touched a real host system or a simulated test root.
5. Compare the original local implementation with remote `main` and identify dependencies it assumes.
6. Determine if the local code is a coherent implementation, a partial implementation or a broken experiment. Do not blindly copy broken or obsolete code.

Restore the original implementation without changing its semantics first. Then make small reviewed fixes with regression tests. Preserve evidence of the original state.

### 3.2 If the installer is absent locally too

Do not fabricate a historical restoration or pretend a new implementation already passed the reported tests. Continue by researching the existing release requirements and implement the installer as a new, explicitly documented implementation. Keep old evidence classified as unavailable or unsubstantiated.

A new installer design must assess the current repository's actual binary/artifact layout before choosing WiX/MSI, MSIX, Inno Setup, or another installer system. Do not select a technology solely by habit. Prefer per-user installation without elevation if compatible with the actual product layout and Windows policies.

### 3.3 Installer functional/security acceptance requirements

The implementation must address:

- deterministic package layout and version metadata;
- compiler/driver executable and required runtime/library assets;
- install root and user-data separation;
- per-user install by default where feasible;
- optional machine-wide install only with explicit elevation and well-defined scope;
- safe, idempotent PATH update that preserves all existing entries, order, casing and unrelated paths;
- duplicate-path detection that handles case-insensitive Windows paths and trailing separators;
- rollback if install fails part-way through;
- ownership manifest so uninstall removes only Omni-owned files/entries;
- upgrade without deleting user projects, configuration, packages or caches;
- repair that restores owned program files and verifies hashes;
- uninstall that preserves user-created data by default and reports remnants;
- correct quoting for spaces, Unicode and PowerShell/CMD/Windows Terminal use;
- actionable diagnostics and nonzero exit codes on failures;
- deterministic offline/portable installation option where justified;
- checksums and an SBOM generated from the actual final package contents;
- provenance linking the installer to exact source commit, toolchain and artifact hashes;
- signing adapter supporting an approved secure signing provider without embedding secrets.

All destructive file/path/registry changes need bounded ownership checks. Never overwrite or remove an unowned path entry. Use explicit install/uninstall manifests and idempotent behavior.

### 3.4 Installer qualification tests

Build tests covering at least:

- fresh install as a non-admin per-user account;
- paths containing spaces and Unicode;
- PATH append, no duplicate on second install, and preservation of unrelated entries;
- rollback after deliberate mid-install failure;
- executable discovery from a new shell session;
- compiler invocation and a real compile/run path once the driver supports it;
- upgrade from prior version without user-data loss;
- repair after one owned file is corrupted or removed;
- uninstall removes owned files and owned PATH changes only;
- uninstall preserves pre-existing PATH and user data;
- reinstall after uninstall;
- checksum mismatch and tampered artifact rejection;
- absent/expired/wrong-purpose/self-signed certificate negative cases;
- valid test certificate flow kept separate from production trust.

Prefer disposable Windows CI runners, a VM, Windows Sandbox or a controlled temporary test root. Do not modify the owner's real system PATH, install location, registry or existing Omni installation without explicit permission. Do not require administrator access for the default qualification suite.

### 3.5 Signing boundary

`INST-002` remains blocked until the owner has provisioned one of:

- a genuine CA-issued OV Authenticode certificate for the intended publisher identity; or
- an owner-approved managed signing service backed by an equivalent public-trust identity.

The investigation identified a traditional OV certificate route as matching the current requirement. SignPath Foundation was found ineligible under the repository's current licensing state; Microsoft Store MSIX signing is not equivalent to signing an EXE/MSI; self-signed certificates are test-only. Recheck eligibility only if repository licensing/terms or the service conditions materially change.

Do not search vendors, request quotes, purchase, enroll, or generate a fake production identity. Do not ask the owner to paste keys or passwords. Continue by implementing and testing the secure signing interface with clearly labelled test material, then leave production trust blocked.

For production completion, validate Code Signing EKU, subject/publisher, validity, chain, revocation, trusted timestamp, signature state, and post-signing hashes of the exact release artifacts. Keep signing secrets in the approved Windows certificate store/HSM or CI secret provider.

---

## 4. Phase 2 — CI and release gate reconciliation

The remote workflow has already been repaired. Inspect the current exact version; do not recreate old defects or add references to nonexistent scripts.

### 4.1 Required CI structure

- Windows (`windows-latest`) is the mandatory supported platform.
- Linux (`ubuntu-latest`) and macOS (`macos-latest`) are experimental compatibility jobs only under the currently approved product policy.
- All three should remain visible, with no fabricated green statuses or blanket `continue-on-error`.
- Do not make Linux/macOS required checks for the Windows-only release gate unless the owner changes that scope.
- Keep the pinned toolchain at Rust 1.95.0 until a deliberate toolchain update is justified and tested.
- Use the existing `ci/assert-toolchain.ps1` on Windows and `ci/assert-toolchain.sh` on Unix-like runners.
- Use the portable timeout strategy in `ci/pipeline.sh`; ensure a missing `timeout`/`gtimeout` is handled without silently disabling the GitHub job-level timeout.
- Ensure every command propagates its true exit code and no pipe masks failures.

### 4.2 Verify the corrected run

Inspect the latest Actions run and job outputs on the exact candidate commit. Verify:

```text
cargo fmt --all -- --check
cargo check -j 1 --workspace --all-targets --locked
cargo build -j 1 --workspace --locked
cargo clippy -j 1 --workspace --all-targets --locked -- -D warnings
cargo test -j 1 --workspace --locked -- --test-threads=1
cargo metadata --format-version 1 --locked
cargo tree --locked
cargo run -j 1 -p omni-conform --locked
cargo run -j 1 -p omni-audit --locked
cargo run -j 1 -p omni-topology --locked
```

Use PowerShell-specific exit-code handling where necessary. The workflow must continue to execute on the actual native Windows runner, not only cross-compile.

### 4.3 Legacy RC script

The previous workflow referenced `ci/gate-0-0-0-14.sh`, which was absent from remote `main`. The repaired workflow removed that broken dependency and uses the validated pipeline/architecture commands instead.

If a local `gate-0-0-0-14.sh` exists, inspect what unique requirements it enforces. Preserve it if still useful, but do not reintroduce an obsolete or duplicate gate merely to satisfy an old workflow reference. If no current consumer exists, document it as superseded rather than making the new workflow fail on an unnecessary path.

### 4.4 Release candidate and qualification record

A technical CI pass is separate from formal qualification. Release reports must be generated from real command results on the exact commit. Do not hardcode successful build/test/signing claims.

The existing schema is:

`spec/release/qualification-record.schema.json`

Do not create a duplicate schema or qualification record. Inspect its fields and cross-field constraints; strengthen validator/tests if needed, but do not invent an attester, date, signature, approval, or formal status. The owner must supply actual attestation after real evidence exists.

---

## 5. Phase 3 — canonical implementation inventory: rebuild from requirements, not prose

The current `IMPLEMENTATION_INVENTORY.md` cannot be treated as reliable current remote-main evidence.

### 5.1 Known arithmetic errors to resolve

- Summary says 24 Verified, but its ID list expands to 25.
- Detailed Verified table has 24 entries and omits `COMP-020`.
- 25 IDs + 1 Accepted + 1 Blocked = 27, not the declared total of 26.
- `COMP-018` is declared N/A, while the inventory still says 26 mandatory items.
- `COMP-011` and `COMP-017` both describe indirect-call code generation; determine whether they are truly separate requirements or duplicate labels.

Do not arbitrarily remove a requirement to make arithmetic work. Trace the authoritative requirement IDs and acceptance criteria through their original sources/specs/ADRs. Determine whether the total really is 26, or whether the inventory is stale and the canonical scope is different. Record the reasoning and approval provenance for any scope change.

### 5.2 Required per-item evidence

For every canonical ID record:

- exact ID and original requirement text;
- requirement authority and release scope classification;
- acceptance criteria (atomic and testable);
- implementation path(s) on the current local/remote commit;
- relevant symbols/interfaces;
- tests (exact names and paths);
- test command and observed result;
- actual native platform/environment;
- exact source commit SHA;
- workflow run/job identity where applicable;
- artifact hashes or evidence refs;
- limitations, missing integration and external dependencies;
- state justified by evidence.

Distinguish Proposed, Designed, Implemented, Integrated, Tested, Verified, Accepted (decision only), Blocked and Approved for Deferral. A passing workspace build, a file existing, or an inventory claim is not sufficient to mark a feature Verified.

### 5.3 Specific source claims to re-audit

- **COMP-007 / standard library:** `compiler/omni-std` is absent remotely. If local source exists, inspect actual integration into the driver/import/module system; do not accept a standalone crate with no usable language interface.
- **COMP-008 / formatter:** `tools/omni-fmt` is absent remotely. Verify parse/format round-trip, comments/trivia, idempotence, malformed input behavior and Edition 1 syntax compliance.
- **COMP-011 and COMP-017 / indirect calls:** no `call_indirect` occurrence was found in remote codegen sources. Inspect the actual Cranelift APIs and generated IR; token search alone is not the final semantic test. Verify function pointer typing, ABI, argument/result lowering, null/invalid pointer rules as specified, and native execution.
- **COMP-012/013 / shell completions:** inventory paths `tools/omni-driver/...` do not match the actual driver crate location `compiler/omni-driver`. Find any local scripts, integrate them with the actual CLI and test in PowerShell/CMD.
- **COMP-014 / conformance:** remote `conformance/` contains only `.gitkeep`. Inspect actual driver/compiler tests and the normative registry. Build a real rule-linked conformance corpus; do not equate unit-test count with conformance coverage.
- **COMP-015 / `--run`:** current remote parser does not expose `--run`. Verify parser, linker/tool invocation, process launch, exit-code semantics, stdout/stderr and errors. A compiler process exit code is not the same as the compiled program's expected exit code.
- **COMP-016 / installer documentation:** restore the actual docs and ensure they describe implemented behavior, not planned functionality.
- **COMP-019 / package/build tooling:** `compiler/omni-pkg` and `compiler/omni-build` absent remotely. If local code exists, test dependency resolution, lockfiles, reproducibility, offline behavior, integrity, build graphs and actual CLI integration.
- **COMP-020 / platform compatibility:** keep the previously approved Windows-only official support policy; Linux/macOS jobs remain experimental. Do not ask for physical non-Windows machines or treat them as a release blocker under the present scope.
- **COMP-021 / RC stabilization:** ensure it is an actual release-candidate process with real build/test/package/installer/checksum/SBOM/provenance evidence, not a successful workflow name.
- **COMP-022 / clean Windows installer qualification:** no current remote installer qualification script exists. Run clean Windows end-to-end install/upgrade/repair/uninstall tests after recovering/implementing the installer.
- **PLAT-001 / C API:** remote runtime functions currently are Rust functions and `omni-runtime/src/lib.rs` explicitly indicates the native ABI is not exported from that safe-Rust crate. `include/omni-rt.h` is absent. If local implementation exists, inspect correct `extern "C"` export boundary, panic/abort semantics, allocation handles, ownership, error codes, header ABI consistency and actual C compiler integration. Never expose unsafe FFI by bypassing the workspace safety architecture; isolate/justify the boundary precisely.
- **COMP-023/024 / reference provenance:** current code contains extensive loan and projected-place tests, but the exact claimed symbols `reference_slot_loans` and `track_aggregate_reference_provenance()` were not found. Identify semantically equivalent current code and tests or implement missing provenance transfer. Test references stored in structs, tuples, arrays, projected places, moves, assignments, calls, return values and control-flow joins.
- **ADR-0002:** Alternative C is already accepted and recorded. Do not ask the owner to decide it again. The formal qualification record remains a separate owner-attested exact-commit artifact.
- **INST-002:** production trusted signing remains owner-provisioned and externally blocked; no self-signed substitute.

---

## 6. Phase 4 — recover and integrate local-only changes safely

After the local/remote map is complete, select a safe integration method for each item.

### If local commits are ahead of `origin/main`

- Inspect commit messages and full diffs.
- Verify those commits are genuinely Omni source commits and not unrelated local experiments.
- Preserve original commit history.
- If local `main` is a strict descendant of `origin/main`, keep it locally and validate the full range.
- Do not push automatically.

### If source files are untracked or modified

- Preserve them and inspect fully.
- Compare with existing remote counterparts and specifications.
- Add only intentional source/docs/tests to the local change set.
- Do not stage secrets, private keys, test certificate packages, `target/`, caches, or generated throwaway artifacts.
- Do not overwrite a newer remote file with an older local copy. Merge the actual changes explicitly.

### If local changes are in a separate local-only branch

Do not create a new branch and do not force-switch. Inspect commit ancestry and worktree first. Integrate changes into `main` with a normal, reviewable merge/cherry-pick or file-level transfer only if safe, preserving the source branch and commit hashes. If uncertain, leave it intact and report the exact reason.

### When integrating to local `main`

- First verify `main` is clean or that every dirty path has been safely preserved and understood.
- Bring local `main` up to date without discarding local commits.
- Make one coherent change at a time.
- Review the entire diff, including binary/renamed/deleted files.
- Re-run affected tests and the full relevant qualification suite.
- Do not push.

---

## 7. Phase 5 — implement the next missing capabilities in dependency order

After original local work has been recovered and the inventory rebuilt, continue with the highest-value in-scope unblocked work. Do not attempt to implement every missing subsystem in one oversized patch.

Recommended order, subject to actual source dependencies:

1. Installer restoration and native Windows packaging qualification, if the local implementation exists and is coherent.
2. Driver capability gaps required by installer/end-user workflow (`--run`, correct linking/execution and useful diagnostics), with tests proving actual compiled-program behavior.
3. C API/FFI boundary and header, if this is a mandatory release requirement; isolate unsafe/platform boundary and provide C integration tests.
4. Standard library integration (imports/modules, source packaging, stable runtime surface) before marking the library complete.
5. Formatter with parser-compatible round-tripping and idempotence.
6. Package/build tools integrated into real user projects, lockfile/integrity/offline workflows.
7. Indirect-call backend implementation and native execution tests if semantically required by normative specs.
8. Reference provenance and ownership verifier completion against normative requirements.
9. A real rule-linked conformance corpus and differential/negative testing.
10. End-to-end release candidate pipeline and provenance/SBOM/checksum/signing separation.

Reorder when evidence proves a dependency. Keep the product versioning policy unchanged; do not invent a new version just to label a milestone.

---

## 8. Mandatory engineering and verification policy

### Rust quality gates

- Preserve `unsafe_code = "forbid"` and warnings-as-errors.
- Keep strict Clippy: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Use `--locked` for all release/qualification Cargo operations.
- Do not weaken lints or suppress failures to obtain green CI.
- Inspect dependency security advisories and licensing; investigate advisories at root cause.
- Do not assume a previous green run validates a new local tree.

### Test depth

Use the relevant subset of:

- unit, integration and end-to-end;
- positive, negative, boundary and malformed-input;
- regression and conformance;
- property-based, fuzz, differential and metamorphic;
- cross-platform compatibility;
- native link-and-run;
- installer lifecycle and recovery;
- security/signature/tamper;
- reproducibility and artifact integrity;
- performance only where a requirement or regression risk justifies it.

### Evidence capture

For every test command, record:

- exact command and working directory;
- source commit or local working-tree diff identity;
- toolchain and OS/runner;
- process exit status;
- relevant output/log location;
- test names/counts only when directly observed;
- whether the test actually exercises the claimed acceptance criterion.

Never rely on text generated by a script if the command itself did not run. Inspect log files before claiming zero warnings, test counts, timings or successful package behavior.

### Temporary artifact cleanup

- Keep useful reproducible test fixtures and release evidence.
- Delete only generated throwaway files owned by your current task after verification.
- Do not delete owner data, local artifacts, installer outputs, original test material or source snapshots without authorization.
- Never run `cargo clean` merely as routine cleanup; check disk usage and ask only if a destructive cleanup is genuinely necessary.

---

## 9. Documentation and durable handoff

Update authoritative docs whenever implementation or evidence changes. Preserve historical snapshots and their dates. At minimum keep synchronized:

- `IMPLEMENTATION_INVENTORY.md`;
- `docs/implementation/remote-main-gate-reconciliation-2026-11-21.md` (append/update only with new dated evidence; preserve its original baseline scope);
- `docs/implementation-baseline.md`;
- relevant architecture docs and ADRs;
- installer/Windows installation and troubleshooting docs;
- release/CI qualification procedure;
- known limitations and risk register;
- test/conformance map;
- session handoff.

For the local-versus-remote issue, add a specific dated recovery report containing:

1. local HEAD and branch;
2. origin/main SHA;
3. local modified/untracked/ignored files before edits;
4. exact files recovered from the local workspace;
5. old and new path mapping;
6. tests and commands executed;
7. local diff summary;
8. any unresolved blockers;
9. whether a commit was made locally;
10. explicit statement that nothing was pushed.

Do not replace the old `24/26` status with a different invented number. Recompute only after the canonical unique requirement set and all statuses are justified.

---

## 10. Immediate execution checklist

Start now and continue in order, without requesting another “continue” after each step:

- [ ] Identify and record the current local repository root, branch, HEAD, dirty state, remotes and all local-only files.
- [ ] Fetch `origin/main` and record its actual current SHA without resetting or checking out over local changes.
- [ ] Inventory the entire local installer tree and qualification scripts, including untracked and ignored candidates.
- [ ] Inspect local commits and history for installer and associated files.
- [ ] Build local-versus-remote path/hash map for all paths listed in Section 2.4.
- [ ] Restore original installer files to the local canonical workspace safely if valid; do not overwrite better current work.
- [ ] Inspect/repair installer lifecycle, PATH ownership, rollback, upgrade, repair, uninstall, offline packaging and test coverage.
- [ ] Inspect local `ci/gate-0-0-0-14.sh`; determine whether still required or superseded by current pipeline. Do not restore merely to satisfy a stale reference.
- [ ] Inspect and repair all inventory claims listed in Section 5.3.
- [ ] Reconstruct the canonical requirement ID set from authoritative sources and resolve the 24/25/26 conflict without arbitrary reclassification.
- [ ] Run focused tests after every coherent fix.
- [ ] Run the full Windows qualification commands on the local native Windows toolchain.
- [ ] Re-run or inspect the latest GitHub validation on the exact candidate commit; do not push.
- [ ] Update docs and produce a dated local recovery/implementation report.
- [ ] Continue to the next ready missing feature after each task, leaving only genuine owner-only blockers open.

---

## 11. Definition of a successful local handoff

The session is not complete merely because the initial audit is done. A good handoff will state:

- what the local repository actually contained before work;
- which missing remote components were recovered from original local source;
- which files were modified/created and why;
- which requirements are now genuinely implemented and independently tested;
- exact commands and results from the local Windows environment;
- exact current remote `main` SHA and the CI run used as baseline;
- whether local work is uncommitted or locally committed;
- what was deliberately not pushed;
- unresolved requirements and owner-provisioned dependencies;
- the next highest-priority task already selected.

Do not state that the first-stage gate is passed until the canonical requirement set is reconciled, every mandatory item has current-commit evidence in Verified status, production signing has been completed if required by scope, and the actual owner-approved exact-commit qualification record exists.

**Begin with Phase 0 now. Preserve the local workspace first; then recover, implement, test, document and continue.**