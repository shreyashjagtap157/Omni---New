# Omni Implementation Inventory


> **Gate inventory reconciliation — 2026-11-23:** The fixed first-stage denominator is 26 unique requirements. The current evidence-backed inventory is **23 Verified / 1 Partial / 1 Accepted / 1 Blocked**. COMP-017 is excluded as a duplicate of COMP-011's full indirect-call emission scope; COMP-018 is excluded because no such requirement is registered. Neither exclusion is counted toward the 26 mandatory requirements.
>
> **COMP-019 is Partial:** `compiler/omni-pkg/src/main.rs` and `compiler/omni-build/src/main.rs` remain placeholder entry points. Their successful compilation only proves that the placeholder crates compile; it does not prove package management, dependency resolution, build orchestration or artifact management. The inventory therefore does not claim COMP-019 as Verified.

## Status Tracking for Mandatory Inventory Items

## Summary

| Category   | Count | Items |
|------------|-------|-------|
| Verified   | 23    | COMP-001–016, COMP-020–024, INST-003, PLAT-001 |
| Partial    | 1     | COMP-019 (package/build tools remain placeholder entry points) |
| Accepted   | 1     | ADR-0002 (Alternative C approved; formal qualification record pending) |
| Blocked    | 1     | INST-002 (production code signing certificates required) |
| **Total**  | **26**| Mandatory items |

## Verified Items (23)

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
| COMP-012 | Shell completions (PowerShell) | tools/omni-driver/completion.ps1 | --help, --native, --output, --opt-level | Verified |
| COMP-013 | Shell completions (CMD) | tools/omni-driver/completion.cmd | All PowerShell options + value completion | Verified |
| COMP-014 | Conformance test suite | compiler/omni-driver/tests/ | 70 tests passing (65 original + 5 new) | Verified |
| COMP-015 | Driver CLI --run flag | compiler/omni-driver/src/main.rs | --run flag supported, links and runs | Verified |
| COMP-016 | Installer qualification docs | tools/omni-installer/ | INSTALLER_QUALIFICATION.md created | Verified |
| COMP-020 | Linux/macOS compatibility | GitHub Actions run 44746310922 | validate-linux pass, validate-macos pass | Verified |
| INST-003 | Checksums/SBOM generation | tools/omni-installer/ | Compute-FileChecksum() (SHA256) and Write-SBOM() functions implemented; SBOM structure defined | Verified |
| COMP-021 | Release-candidate stabilization | BLD-006 | RC cycle completed: clean build, full test matrix (fmt, clippy, test), checksums/SBOM generated, installer qualification passed | Verified |
| COMP-022 | Windows installer qualification (clean Windows) | INST-001/003 | Windows CI job (windows-latest) runs installer-qualify.ps1. Local qualification passed: per-user install, paths with spaces, upgrade, repair, uninstall, reinstall, driver --help, driver --run | Verified |
| PLAT-001 | C API compatibility | compiler/omni-runtime/ | C ABI surface documented and tested with actual C programs on Windows. omni_rt_alloc/dealloc/write_stdout_byte/flush_stdout/exit/abort all callable from C. C header (omni-rt.h) created. RUNTIME_ABI_VERSION=1, INVALID_ALLOCATION_ID=0. Two C test programs (c_api_basic.c, c_api_write.c) compiled and executed successfully | Verified |
| COMP-023 | Reference provenance for projected places | compiler/omni-verify/src/ownership.rs | reference_slot_loans map; dereference through projected place resolves slot provenance; 74 ownership tests pass | Verified |
| COMP-024 | Aggregate-contained reference provenance | compiler/omni-verify/src/ownership.rs | track_aggregate_reference_provenance() records provenance for references stored in struct fields, tuple elements, array elements; move ends source slot loan; 74 ownership tests pass | Verified |

## Partial Items (1)

| ID | Item | Crate/Module | Tests/Evidence | Status |
|----|------|-------------|----------------|--------|
| COMP-019 | Package/build tools | compiler/omni-pkg/, compiler/omni-build/ | Both entry points remain TODO-only placeholders; workspace compilation is not behavioral evidence | Partial |

## Accepted Items (1)

| ID | Item | Crate/Module | Status | Notes |
|----|------|-------------|--------|-------|
| ADR-0002 | Foundation gate sign-off | spec/manifest/ | Accepted | Alternative C (separate technical outcome from formal qualification) approved by project owner. Qualification record schema created (spec/schemas/qualification-record.schema.json). Technical/formal states separated. Formal owner-attested qualification record pending. |

## Blocked Items (1)

| ID | Item | Crate/Module | Block Reason |
|----|------|-------------|------------|
| INST-002 | Installer code signing | tools/omni-installer/ | Invoke-FileSigning() implemented; test-certificate signing VERIFIED (ephemeral self-signed cert: cert discovery, file signing, tampered-file rejection, missing/invalid cert handling all verified). Production-trusted publisher identity UNQUALIFIED - requires genuine trusted production certificate per owner authorization G. | Production code signing certificates required |

## Excluded Identifiers (2)

| ID | Item | Reason |
|----|------|--------|
| COMP-017 | Codegen full indirect calls | Duplicate acceptance scope already covered by COMP-011 full indirect-call emission; COMP-011 remains the single counted requirement. |
| COMP-018 | Reserved/unknown identifier | No COMP-018 requirement is registered; it is not a mandatory inventory item. |

## Live Evidence Discrepancies

| ID | Evidence conflict | Gate treatment |
|----|-------------------|----------------|
| COMP-019 | `compiler/omni-pkg/src/main.rs` and `compiler/omni-build/src/main.rs` contain TODO-only placeholder entry points. Cargo build/test success with no meaningful subsystem tests does not substantiate package management or build orchestration. | Classified Partial. Implement the specified behavior and acceptance evidence before claiming Verified. |

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
| COMP-017 | N/A (duplicate of COMP-011; not a separate mandatory requirement) |
| COMP-018 | N/A (identifier does not exist) |
| COMP-019 | Partial |
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

2026-11-23: Reconciled all 26 mandatory IDs and corrected source-backed status: 23 Verified, COMP-019 Partial, ADR-0002 Accepted, and INST-002 Blocked. COMP-017 is a duplicate of COMP-011 and COMP-018 is unregistered; both are excluded from the denominator.