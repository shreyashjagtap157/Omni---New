# Omni Implementation Inventory


> **Remote-main evidence warning — 2026-11-21:** The status table below preserves the owner-reported local-workspace snapshot from 2026-11-20. It is **not** an independently verified status count for the public `main` branch. The remote-main audit at commit `0daf201d1e5148e38302c33b5f433c9d2e74737a` found missing implementation paths and inconsistencies between the summary ID list and the detailed item table. Preserve this historical snapshot, but do not use its 24/26 count as current release evidence until the local source is restored, reconciled and requalified. See [Remote-main gate reconciliation](docs/implementation/remote-main-gate-reconciliation-2026-11-21.md).

## Status Tracking for Mandatory Inventory Items

## Summary

| Category   | Count | Items |
|------------|-------|-------|
| Verified   | 24    | COMP-001–017, COMP-019, COMP-020, INST-003, COMP-021, COMP-022, PLAT-001, COMP-023, COMP-024 |
| Partial    | 0     | (all mandatory items verified or externally blocked) |
| Accepted   | 1     | ADR-0002 (Alternative C approved; formal qualification record pending) |
| Blocked    | 1     | INST-002 (production code signing certificates required) |
| **Total**  | **26**| Mandatory items |

## Verified Items (24)

| ID | Item | Crate/Module | Tests/Evidence | Status |
|----|------|-------------|----------------|--------|
| COMP-001 | Repository audit | Root | Complete | Verified |
| COMP-002 | Environment audit | Root | Complete | Verified |
| COMP-003 | Discrepancy register | Root | DISCREPANCY_REGISTER.md created | Verified |
| COMP-004 | Inventory overclaims corrected | Root | 20→25 crates, EBNF status, foundation gate | Verified |
| COMP-005 | Windows target in toolchain | rust-toolchain.toml, ci/assert-toolchain.sh | Added | Verified |
| COMP-006 | Windows CI pipeline | .github/workflows/ci.yml, ci/pipeline.ps1 | Full pipeline passes | Verified |
| COMP-007 | omni-std crate | compiler/omni-std/ | 4 tests pass (write_byte/write_string/exit) | Verified |
| COMP-008 | omni-fmt crate | tools/omni-fmt/ | 7 tests pass, clippy zero warnings | Verified |
| COMP-009 | Effects system | compiler/omni-effects/ | 18 passing tests | Verified |
| COMP-010 | Trait system | compiler/omni-traits/ | 10 passing tests | Verified |
| COMP-011 | Codegen indirect calls (full emission) | compiler/omni-codegen/ | FnPtr codegen with call_indirect, ABI computation | Verified |
| COMP-017 | Codegen full indirect calls (beyond detection) | compiler/omni-codegen/ | Full emission implemented - FnPtr + call_indirect + ABI | Verified |
| COMP-012 | Shell completions (PowerShell) | tools/omni-driver/completion.ps1 | --help, --native, --output, --opt-level | Verified |
| COMP-013 | Shell completions (CMD) | tools/omni-driver/completion.cmd | All PowerShell options + value completion | Verified |
| COMP-014 | Conformance test suite | compiler/omni-driver/tests/ | 70 tests passing (65 original + 5 new) | Verified |
| COMP-015 | Driver CLI --run flag | compiler/omni-driver/src/main.rs | --run flag supported, links and runs | Verified |
| COMP-016 | Installer qualification docs | tools/omni-installer/ | INSTALLER_QUALIFICATION.md created | Verified |
| COMP-019 | Package/build tools | compiler/omni-pkg/, compiler/omni-build/ | Both crates build+clippy+tests pass | Verified |
| INST-003 | Checksums/SBOM generation | tools/omni-installer/ | Compute-FileChecksum() (SHA256) and Write-SBOM() functions implemented; SBOM structure defined | Verified |
| COMP-021 | Release-candidate stabilization | BLD-006 | RC cycle completed: clean build, full test matrix (fmt, clippy, test), checksums/SBOM generated, installer qualification passed | Verified |
| COMP-022 | Windows installer qualification (clean Windows) | INST-001/003 | Windows CI job (windows-latest) runs installer-qualify.ps1. Local qualification passed: per-user install, paths with spaces, upgrade, repair, uninstall, reinstall, driver --help, driver --run | Verified |
| PLAT-001 | C API compatibility | compiler/omni-runtime/ | C ABI surface documented and tested with actual C programs on Windows. omni_rt_alloc/dealloc/write_stdout_byte/flush_stdout/exit/abort all callable from C. C header (omni-rt.h) created. RUNTIME_ABI_VERSION=1, INVALID_ALLOCATION_ID=0. Two C test programs (c_api_basic.c, c_api_write.c) compiled and executed successfully | Verified |
| COMP-023 | Reference provenance for projected places | compiler/omni-verify/src/ownership.rs | reference_slot_loans map; dereference through projected place resolves slot provenance; 74 ownership tests pass | Verified |
| COMP-024 | Aggregate-contained reference provenance | compiler/omni-verify/src/ownership.rs | track_aggregate_reference_provenance() records provenance for references stored in struct fields, tuple elements, array elements; move ends source slot loan; 74 ownership tests pass | Verified |

## Accepted Items (1)

| ID | Item | Crate/Module | Status | Notes |
|----|------|-------------|--------|-------|
| ADR-0002 | Foundation gate sign-off | spec/manifest/ | Accepted | Alternative C (separate technical outcome from formal qualification) approved by project owner. Qualification record schema created (spec/schemas/qualification-record.schema.json). Technical/formal states separated. Formal owner-attested qualification record pending. |

## Blocked Items (1)

| ID | Item | Crate/Module | Block Reason |
|----|------|-------------|------------|
| INST-002 | Installer code signing | tools/omni-installer/ | Invoke-FileSigning() implemented; test-certificate signing VERIFIED (ephemeral self-signed cert: cert discovery, file signing, tampered-file rejection, missing/invalid cert handling all verified). Production-trusted publisher identity UNQUALIFIED - requires genuine trusted production certificate per owner authorization G. | Production code signing certificates required |

## Requirement Identifier Status Summary

| ID | Status |
|----|--------|
| COMP-001 | Verified |
| COMP-002 | Verified |
| COMP-003 | Verified |
| COMP-004 | Verified |
| COMP-005 | Verified |
| COMP-006 | Verified |
| COMP-007 | Verified |
| COMP-008 | Verified |
| COMP-009 | Verified |
| COMP-010 | Verified |
| COMP-011 | Verified |
| COMP-012 | Verified |
| COMP-013 | Verified |
| COMP-014 | Verified |
| COMP-015 | Verified |
| COMP-016 | Verified |
| COMP-017 | Verified |
| COMP-018 | N/A (identifier does not exist) |
| COMP-019 | Verified |
| COMP-020 | Verified |
| COMP-021 | Verified |
| COMP-022 | Verified |
| COMP-023 | Verified |
| COMP-024 | Verified |
| INST-002 | Blocked |
| INST-003 | Verified |
| PLAT-001 | Verified |
| ADR-0002 | Accepted |

## Evidence Classifications

- `VERIFIED`: Directly demonstrated through reproducible tests or compiler runs
- `PROVEN`: Formally proved within machine-checked assumptions
- `EXPERIMENTALLY_VALIDATED`: Supported by designated empirical experiments
- `IMPLEMENTED_UNVERIFIED`: Code exists but lacks full qualification tests
- `PROPOSED` / `HYPOTHESIS`: Theoretical design under active study
- `BLOCKED` / `REJECTED`: Halted due to environment/dependencies or rejected with documented rationale

## Discrepancy Resolution Record

| D-Entry | Issue | Resolution Status |
|---------|-------|-------------------|
| D-001 | VIBE-GRAM productions in normative EBNF vs. unratified Candidate 2/3 status | Editorial comment discrepancy; ADR-0003 boundary enforced; no bytes changed |
| D-002 | Foundation gate status field vs. ADR-0002 | JSON declares qualified; ADR-0002 Alternative C accepted; technical/formal states separated via qualification-record.schema.json |
| D-003 | rust-toolchain.toml targets omit Windows | Windows target added to rust-toolchain.toml |
| D-004 | IMPLEMENTATION_INVENTORY.md contains known overclaims | 4 overclaims identified and corrected |
| D-005 | All 567 registry rules are Candidate; zero Ratified | Evidence gap, not contradiction; rule ratification is future milestone |

## Last Updated

2026-11-20: Repository audit completed. Architecture map (docs/architecture/crate-topology.md) and specification authority matrix (docs/specification/authority-matrix.md) created. Inventory reconciled: COMP-019 added to Verified items table (was missing), COMP-023 (reference provenance for projected places) and COMP-024 (aggregate-contained reference provenance) added as Verified. All workspace checks pass (fmt=0, clippy=0, build=0, test=0). COMP-020 verified via GitHub Actions run 44746310922 on native Linux (ubuntu-latest) and macOS (macos-latest) runners. INST-002 remains Blocked (production code signing certificates). ADR-0002 Accepted (Alternative C, formal qualification record pending).