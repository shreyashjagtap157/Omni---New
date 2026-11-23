# Windows Installer Qualification

## Scope

The qualification script is `tools/omni-installer/installer-qualify.ps1`. It builds the locked release workspace and runs the installer in child PowerShell processes on a clean GitHub-hosted Windows runner.

## Automated scenarios

Every assertion contributes to the final exit code:

- Locked release build and all mandatory product-artifact checks.
- Per-user install without elevation.
- Preservation of all prior PATH entries and exactly one owned `bin` entry.
- Install manifest coverage for every file in the normative specification.
- Actual runtime DLL, C ABI header and import library installation.
- Valid CycloneDX 1.5 SBOM with SHA-256 hashes for installed files.
- `omni-driver --help`, `--run` against the installed specification, formatter write plus idempotent `--check`, and native object emission with a provenance sidecar.
- External C programs compiled, linked and executed using the installed SDK header/import library and installed runtime DLL.
- Repair after deleting an installed executable and successful launch afterward.
- Upgrade, uninstall, reinstallation and a path containing spaces.
- Cleanup and restoration of runner PATH and `OMNI_SPEC_ROOT`, including on failure.

The separate `ci/test-c-api.ps1` workspace mode also validates the same C programs against the build-tree runtime. The installer qualification calls its installed-SDK mode so the actual distributed DLL and SDK are exercised.

## Qualification status

This document defines the acceptance procedure. The installer is not qualified until the Windows CI job `Qualify Windows installer payload` passes on the exact candidate commit. A local or historical result is not evidence for a newer commit.

Production Authenticode signing remains a separate owner-provisioned gate (INST-002). No self-signed test certificate satisfies production trust.
