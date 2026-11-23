# Windows Installer Qualification

## Scope

The qualification script is \`tools/omni-installer/installer-qualify.ps1\`.
It builds the locked release workspace and exercises the installer in child
PowerShell processes on a clean GitHub-hosted Windows runner.

## Automated scenarios

The qualification must pass all of the following, with every assertion included
in the final process exit code:

- Locked release build and required product-artifact presence.
- Per-user installation without elevation.
- Preservation and single insertion of the existing user PATH entries.
- Installation of the actual runtime DLL, C ABI header, import library,
  Edition 1 specification tree and pinned toolchain declaration.
- \`omni-driver --help\`, explicit \`omni-driver --run\` against the installed
  specification, formatter \`--check\`, and native object emission with a
  valid provenance sidecar.
- Repair after deleting an installed executable.
- Upgrade over an existing installation.
- Uninstall, including removal of only the owned PATH and \`OMNI_SPEC_ROOT\`
  entries.
- Reinstallation and installation under a path containing spaces.
- Cleanup and restoration of the GitHub runner's original per-user
  environment, including on test failure.

The companion \`ci/test-c-api.ps1\` compiles and executes both C programs under
\`compiler/omni-runtime/tests/\` using MSVC and the actual \`omni-runtime-ffi\`
library. Rust FFI unit tests alone do not qualify external C linkage.

## Qualification status

This document defines the acceptance procedure. The installer is not qualified
until the Windows CI job \`Qualify Windows installer payload\` and the external C
API step complete successfully on the exact candidate commit. A local or
historical test result is not evidence for a newer commit.

Production Authenticode signing remains a separate owner-provisioned gate
(INST-002). No self-signed test certificate can satisfy production trust.
