# Remote-main first-stage gate reconciliation — 2026-11-21

## Scope and conclusion

This audit is against the public `main` tree at commit `0daf201d1e5148e38302c33b5f433c9d2e74737a`:

- Repository: [Omni---New](https://github.com/shreyashjagtap157/Omni---New)
- Audited commit: [`0daf201d1e5148e38302c33b5f433c9d2e74737a`](https://github.com/shreyashjagtap157/Omni---New/commit/0daf201d1e5148e38302c33b5f433c9d2e74737a)
- Qualification run: [GitHub Actions run 44873145748](https://github.com/shreyashjagtap157/Omni---New/actions/runs/44873145748)

The native validation matrix on that commit completed successfully:

| Job | Result | Release policy |
|---|---|---|
| Linux | Passed | Experimental compatibility |
| macOS | Passed | Experimental compatibility |
| Windows | Passed | Mandatory supported platform |

The workflow executed pinned Rust 1.95.0 qualification, formatting, workspace checking, workspace build, strict Clippy, workspace tests, locked Cargo metadata and dependency-tree checks, and the `omni-conform`, `omni-audit` and `omni-topology` gates. These results validate the Rust workspace and specified checks on this commit. They do **not** establish that the installer, formatter, standard library, package/build tools, C API, or the full implementation inventory are complete.

**First-stage gate: NOT PASSED.** The informational gate-readiness job records the exact missing remote artifacts, inventory discrepancies and owner-provisioned signing/attestation requirements. Successful compiler CI is not a qualification decision.

## Remote-tree discrepancies

The following paths are absent from the audited `main` tree:

| Inventory claim | Remote-main evidence |
|---|---|
| `compiler/omni-std` (COMP-007) | Directory absent. |
| `tools/omni-fmt` (COMP-008) | Directory absent. |
| `tools/omni-driver/completion.ps1` (COMP-012) | Path absent. The driver crate is under `compiler/omni-driver`. |
| `tools/omni-driver/completion.cmd` (COMP-013) | Path absent. |
| Installer qualification documentation (COMP-016) | `tools/omni-installer/INSTALLER_QUALIFICATION.md` absent. |
| `compiler/omni-pkg` and `compiler/omni-build` (COMP-019) | Both directories absent from the workspace tree. |
| `tools/omni-installer` and `installer-qualify.ps1` (INST-002, COMP-022) | Directory and qualification script absent. Path-specific GitHub commit history returned no restoration commit. |
| Checksum/SBOM installer implementation (INST-003) | No installer source path exists in the remote tree to substantiate the inventory claim. |
| Release-candidate installer/packaging evidence (COMP-021) | The previous `ci/gate-0-0-0-14.sh` path is absent. The repaired CI workflow no longer calls this nonexistent script, but a full release-candidate cycle is still unverified. |
| `include/omni-rt.h` (PLAT-001) | Header is absent. |
| Driver `--run` option (COMP-015) | The current `compiler/omni-driver/src/main.rs` argument parser does not define `--run`. |
| Indirect-call emission (COMP-011 / COMP-017) | No `call_indirect` occurrence was found in `compiler/omni-codegen/src`; the matching text occurs in the inventory claim, not the implementation sources. |

The `compiler/omni-runtime/src/lib.rs` functions named `omni_rt_alloc`, `omni_rt_dealloc`, `omni_rt_write_stdout_byte`, `omni_rt_flush_stdout`, `omni_rt_exit` and `omni_rt_abort` are Rust functions. The file explicitly says the native ABI is not exported from the safe-Rust crate; no `extern "C"` export was found in the runtime source. The current remote source therefore does not substantiate the inventory statement that these functions are callable from C.

The conformance directory contains only `.gitkeep`; the driver contains unit tests, but this is not evidence of the separately claimed 70-case conformance corpus. The ownership verifier has reference/loan tests, but the exact implementation names `reference_slot_loans` and `track_aggregate_reference_provenance()` cited by COMP-023/024 do not occur in current source. Those items need requirement-to-current-implementation revalidation rather than name-based acceptance.

## Inventory arithmetic is inconsistent

The current `IMPLEMENTATION_INVENTORY.md` requires reconciliation before a formal record can be produced:

1. The Verified summary says **24**, but the identifiers written in its own list expand to **25**: COMP-001–017 (17 IDs), COMP-019, COMP-020, INST-003, COMP-021, COMP-022, PLAT-001, COMP-023 and COMP-024.
2. The detailed `Verified Items` table has 24 rows but omits COMP-020, so its ID set differs from the summary.
3. COMP-011 and COMP-017 both describe indirect-call code generation, yet current source contains no corresponding `call_indirect` emission.
4. The status table includes COMP-018 as N/A while the inventory total is 26. The canonical set of 26 unique mandatory IDs is not established by the current table.
5. If the 25 IDs enumerated in the Verified summary are combined with one Accepted and one Blocked item, the result is 27, not the declared 26.

The audit does not silently choose which ID to remove, merge or reclassify. The authoritative requirement set must be reconciled against source requirements and acceptance criteria. The historical 24/26 local snapshot is preserved, but is not current remote-main verification.

## ADR-0002

Alternative C is accepted and recorded in [ADR-0002](https://github.com/shreyashjagtap157/Omni---New/blob/main/docs/adr/0002-foundation-gate-qualification-evidence.md) by commit [`838179a`](https://github.com/shreyashjagtap157/Omni---New/commit/838179ac6165639afb8f6f816bef7681fe54ae7b).

That decision separates technical CI results from formal qualification. It does not approve any particular current commit. No `formal_status: "approved"` qualification record is present, and none is generated by this audit. A formal record can only describe a reconciled requirement set and a technically qualified exact commit, with the actual owner attestation.

## INST-002 production signing

No production certificate or signing identity is in the repository, nor should a private key be committed. The current source has no installer artifact to sign.

When the genuine publisher identity has been provisioned into the approved signing environment, the production release pipeline must:

1. Verify the certificate's Code Signing EKU, validity period, private-key availability through the approved provider, publisher subject and trusted certificate chain.
2. Sign the actual release binaries and installer with SHA-256.
3. Apply a trusted RFC 3161 timestamp using SHA-256.
4. Verify the signature under the Windows Authenticode policy, including publisher identity, trust chain, revocation status and timestamp.
5. Record final post-signing artifact hashes and matching release evidence.

Microsoft's [SignTool documentation](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool) documents signing and verification. Microsoft's [Authenticode timestamp guidance](https://learn.microsoft.com/en-us/windows/win32/seccrypto/time-stamping-authenticode-signatures) recommends SHA-256 digests and RFC 3161 timestamps. SignPath Foundation's [eligibility conditions](https://signpath.org/terms) require an OSI-approved open-source licence and other conditions; the current repository's [LICENSE](https://github.com/shreyashjagtap157/Omni---New/blob/main/LICENSE) says licensing terms are still to be established, so Foundation signing is not currently an eligible shortcut.

No certificate is purchased, created or substituted by this audit. Self-signed test certificates do not satisfy production publisher trust.

## Required restoration and closure sequence

1. Recover the original local installer source and associated tests/docs from the local working tree. Restore the intended files to `main` without replacing them with speculative rewrites.
2. Reconcile every inventory row against the canonical requirement-ID set, exact source paths, test names and current-commit evidence. Resolve the 24/25/26 arithmetic discrepancy.
3. Restore or implement absent required components: installer, shell completions, formatter, standard library, package/build tools, C API exports/header and CLI features as required by the approved release scope. Do not claim any item verified until its acceptance tests pass.
4. Build and test the installer on clean native Windows, including per-user install, PATH preservation, paths with spaces/Unicode, launch, upgrade, repair, uninstall and reinstall.
5. Provision the genuine production signing identity in the approved secure environment, then sign and independently verify the release artifacts. Keep private keys and passwords out of chat and source control.
6. Re-run the full qualification set on the final exact `main` commit.
7. Only after the requirement set is unambiguous, all mandatory entries are Verified, technical evidence is green, and the project owner supplies the real attestation, validate the existing qualification-record schema and record formal approval.

## Source recovery boundary

The local installer source may exist in the owner's local workspace, but it is not present in public `main`, path-specific Git history, or the accessible project-file library. This audit does not claim access to the owner's local disk. The recovery input is a ZIP of the local `tools/omni-installer/` tree and its associated installer tests and qualification documentation, plus any local completion scripts that belong to the existing implementation. Exclude `target/`, caches, certificate files, PFX files, private keys, passwords and tokens.
