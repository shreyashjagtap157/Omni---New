# Omni Windows install layout

This document describes the payload currently produced by \`omni-installer.ps1\`.
It does not describe future language/runtime assets as though they already exist.

## Install roots

| Mode | Default root | Elevation |
|---|---|---|
| Per-user | \`%LOCALAPPDATA%\\Omni\\\` | Not required |
| Machine-wide | \`C:\\Program Files\\Omni\\\` | Required |

The install manifest is \`<root>\\install-manifest.json\`. It contains only
relative paths rooted beneath the installation directory. Uninstall validates
containment before deleting any manifest-listed file.

## Installed payload

\`\`\`
<root>\
├── bin\
│   ├── omni-driver.exe
│   ├── omni-fmt.exe
│   ├── omni-canon.exe
│   └── omni-registry.exe
├── lib\
│   └── omni_runtime_ffi.dll
├── sdk\
│   ├── include\\omni-rt.h
│   └── lib\\omni-rt.lib
├── share\
│   ├── rust-toolchain.toml
│   └── spec\\                 # complete manifest-bound Edition 1 spec tree
└── install-manifest.json
\`\`\`

The installer copies the actual compiled runtime DLL, the actual C ABI header,
and the actual import/static library from the locked build output. Installation
fails closed if any required artifact is missing. It does not create marker
files pretending to be runtime binaries, headers, import libraries or Omni
source standard-library modules.

The current \`omni-std\` is a Rust support crate; it does not yet provide a
distributable \`.oml\` standard-library source tree. The installer therefore
does not fabricate or install an \`omni-std.oml\` placeholder.

## Specification discovery

The driver requires the Edition 1 specification. The installer copies the
complete \`spec/\` tree to \`<root>\\share\\spec\`, copies the pinned
\`rust-toolchain.toml\` beside it, and sets the installation-scope
\`OMNI_SPEC_ROOT\` environment variable to that exact tree. Uninstall removes
this environment variable only when it still points to the installation being
removed. A conflicting existing \`OMNI_SPEC_ROOT\` prevents a second install
from silently redirecting another installation.

## PATH and uninstall

Per-user installs modify only the user's PATH; machine-wide installs modify
the machine PATH. Existing entries and their order are preserved. Uninstall
removes only the exact Omni \`bin\` entry and manifest-owned files. Manifest
paths are rejected if absolute or if they escape the installation root.

User data and projects are not stored under the install root and are not
removed by the installer.
