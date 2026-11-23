# Omni Windows install layout

This document describes the payload produced by `omni-installer.ps1`; it does not describe future language/runtime assets as though they already exist.

## Install roots

| Mode | Default root | Elevation |
|---|---|---|
| Per-user | `%LOCALAPPDATA%\Omni\` | Not required |
| Machine-wide | `C:\Program Files\Omni\` | Required |

The install manifest is `<root>\install-manifest.json`. It contains relative paths for every installed file, including every file in the copied specification tree and the generated CycloneDX SBOM. Uninstall validates path containment and rejects reparse-point ancestors before deleting manifest-owned files.

## Installed payload

```text
<root>\
├── bin\
│   ├── omni-driver.exe
│   ├── omni-fmt.exe
│   ├── omni-canon.exe
│   ├── omni-registry.exe
│   └── omni_runtime_ffi.dll
├── sdk\
│   ├── include\omni-rt.h
│   └── lib\omni-rt.lib
└── share\
    ├── rust-toolchain.toml
    ├── spec\                 # complete manifest-bound Edition 1 spec tree
    └── doc\
        └── sbom.cdx.json     # CycloneDX 1.5, SHA-256 hashes of installed files
```

The installer copies the actual compiled runtime DLL, C ABI header and import library. The DLL is placed beside the executables so Windows can locate it for external C clients when the installed `bin` directory is on PATH. The SBOM lists each installed file except itself; hashes are calculated from the copied payload.

The current `omni-std` is a Rust support crate; it does not yet provide a distributable `.oml` source library. The installer therefore does not fabricate an `omni-std.oml` placeholder.

## Specification discovery

The installer copies the complete `spec/` tree to `<root>\share\spec`, copies the pinned `rust-toolchain.toml` beside it, and sets the installation-scope `OMNI_SPEC_ROOT` environment variable to that tree. Uninstall removes that environment variable only when it still points to the installation being removed. A conflicting existing value prevents one install from redirecting another installation.

## PATH, repair and uninstall

Per-user installs modify only the user's PATH; machine-wide installs modify machine PATH and require an administrator. Existing PATH entries and order are preserved. Repair and upgrade preflight all mandatory source artifacts before removing the previous manifest-owned files. Uninstall removes only exact Omni PATH/spec-root entries and manifest-owned files. Untracked user data inside the root is preserved rather than recursively deleted.
