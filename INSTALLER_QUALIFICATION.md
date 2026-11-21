# Windows Installer Qualification

## Overview
The Omni Windows installer (`tools/omni-installer/omni-installer.ps1`) provides per-user and machine-wide installation of the Omni compiler distribution. This document records the qualification status and test results.

## Installer Overview

**File:** `tools/omni-installer/omni-installer.ps1`

**Modes:**
- **Install:** Installs Omni to the specified directory (default: `%LOCALAPPDATA%\Omni\` for per-user, `C:\Program Files\Omni\` for machine-wide)
- **Uninstall:** Removes all installed files, cleans up PATH entries
- **Repair:** Verifies and restores missing files from the build output
- **Upgrade:** Re-installs over an existing installation with manifest-based stale file cleanup

**Install Layout:**
```
<root>\
├── bin\                    # Executables (added to PATH)
│   ├── omni-driver.exe     # Compiler driver
│   ├── omni-fmt.exe        # Source code formatter
│   ├── omni-canon.exe      # Specification canonicalization
│   ├── omni-conform.exe    # Conformance test runner
│   ├── omni-audit.exe      # Semantic linkage reaper
│   ├── omni-registry.exe   # Specification registry loader
│   ├── omni-topology.exe   # Compiler crate topology verifier
│   └── omni-bindgen.exe    # FFI binding generator
├── lib\                    # Runtime and standard library
│   ├── omni-rt.dll         # Runtime C ABI
│   └── std\                # Standard library modules
│       └── omni-std.oml
├── sdk\                    # SDK assets
│   ├── include\omni-rt.h   # Runtime C ABI header
│   └── lib\omni-rt.lib     # Runtime import library
└── share\                  # Documentation and examples
    ├── doc\README.md
    ├── examples\hello.omni
    └── templates\app\main.omni
```

## PATH Management

The installer adds `<install_root>\bin` to:
- **User PATH** (`HKCU\Environment\PATH`) for per-user installs
- **Machine PATH** (`HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment\PATH`) for machine-wide installs

**PATH Management Rules:**
1. Entry is added only if not already present (idempotent)
2. Existing entries and order are preserved
3. Entry is appended (not prepended)
4. Only Omni-owned entries are removed on uninstall

## Test Results

### Local Testing (Windows 11, x86_64-pc-windows-msvc)

| Test | Status | Notes |
|------|--------|-------|
| Per-user install | PASS | Installs to `%LOCALAPPDATA%\Omni\`, adds to User PATH |
| Machine-wide install | Not tested | Requires admin elevation |
| Uninstall | PASS | Removes all files, cleans PATH entries |
| Reinstall (idempotent) | PASS | No PATH duplication on re-install |
| Repair | Not tested | Requires missing files scenario |
| Native codegen + link + run | PASS | `omni-driver --run test.omni` produces exit code 42 |

## Limitations

1. **No Windows CI:** CI pipeline runs on `ubuntu-latest` only. Windows CI not yet configured.
2. **No code signing:** Installer is not code-signed.
3. **No checksums/SBOM:** No checksum or SBOM generation.
4. **Not tested on clean Windows:** All testing done on the development machine with the full Rust toolchain installed.

## Next Steps

1. Add Windows CI to validate installer on clean Windows environments
2. Add code signing for the installer
3. Add checksums and SBOM generation
4. Test installer on clean Windows without Rust toolchain
5. Add Linux/macOS installers
