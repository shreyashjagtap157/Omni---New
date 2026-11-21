# Omni Install Layout — Windows

This document defines the installed file layout for Omni on Windows, covering
per-user (no elevation) and machine-wide (elevation) installation modes.

## Install roots

| Mode | Default root | Elevation | Notes |
|------|-------------|-----------|-------|
| Per-user | `%LOCALAPPDATA%\Omni\` | No | Recommended default. Installs for the current user only. |
| Machine-wide | `C:\Program Files\Omni\` | Yes (admin) | Installs for all users on the machine. |

The user may override either root during installation. The chosen root is
recorded in `<root>\install-manifest.json` for uninstall and upgrade.

## Directory layout under the install root

```
<root>\
├── bin\                    # Executables (added to PATH)
│   ├── omni-driver.exe     # Compiler driver (CLI entry point)
│   ├── omni-fmt.exe        # Source code formatter
│   ├── omni-canon.exe      # Specification canonicalization tool
│   ├── omni-conform.exe    # Conformance test runner
│   ├── omni-audit.exe      # Semantic linkage reaper
│   ├── omni-registry.exe   # Specification registry loader
│   ├── omni-topology.exe   # Compiler crate topology verifier
│   ├── omni-bindgen.exe    # FFI binding generator (optional)
│   └── omni-installer.exe  # Installer/updater/uninstaller (this tool)
├── lib\                    # Runtime and standard library
│   ├── omni-rt.dll         # Omni runtime (C ABI entry points)
│   └── std\                # Omni standard library modules
│       └── core\
│           └── omni-std.oml
├── sdk\                    # SDK assets (headers, import libraries)
│   ├── include\
│   │   └── omni-rt.h       # Runtime C ABI header
│   └── lib\
│       └── omni-rt.lib     # Runtime import library
├── share\                  # Read-only architecture-independent data
│   ├── doc\                # Documentation
│   │   ├── README.md
│   │   ├── language-spec.md
│   │   └── install-guide.md
│   ├── examples\           # Example programs
│   │   └── hello.omni
│   └── templates\          # Project templates
│       └── app\
│           └── main.omni
└── install-manifest.json   # Record of installed files (for uninstall/upgrade)
```

## User data (separate from install-owned files)

User data is never stored under the install root, so upgrades and uninstalls
do not touch it.

| Purpose | Location |
|---------|----------|
| User configuration | `%LOCALAPPDATA%\Omni\config\` |
| User cache | `%LOCALAPPDATA%\Omni\cache\` |
| User packages | `%LOCALAPPDATA%\Omni\packages\` |
| User projects | User's Documents or `%USERPROFILE%\omni-projects\` |

## PATH management

The installer adds `<root>\bin` to the appropriate PATH:

- **Per-user install**: User PATH (`HKCU\Environment\PATH`)
- **Machine-wide install**: Machine PATH (`HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment\PATH`)

Rules:
1. The entry is added only if not already present (idempotent).
2. Existing entries and their order are preserved.
3. The entry is appended (not prepended) to avoid shadowing system tools.
4. On uninstall, only Omni-owned PATH entries are removed.
5. The change takes effect for new processes; existing shells need to re-read PATH.

## Upgrade and repair

- **Upgrade**: Re-install over an existing root. The install-manifest.json from
  the previous version is used to remove stale files before copying new ones.
- **Repair**: Re-verify and restore all files from the install payload, rebuild
  the PATH entry, and re-emit the install-manifest.json.

## Uninstall

- Remove all files listed in install-manifest.json.
- Remove Omni-owned entries from PATH.
- Remove the install root directory if empty.
- User data under `%LOCALAPPDATA%\Omni\config\`, `cache\`, `packages\` is
  preserved (the user may choose to remove it separately).
