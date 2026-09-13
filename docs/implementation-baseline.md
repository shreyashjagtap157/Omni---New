# OMNI-IMP-0.0.0.1 — Repository Baseline

## Authority

Implementation planning is frozen against the sealed master plan:

- File: `Omni_Implementation_Master_Plan_Edition1_Final_Comprehensive_MicroAtomic_Final.md`
- SHA-256: `cdf7ff8cb0eae597b52e295fa43c11f701415f62f2a292f4bd4a2be5012cb506`
- Lines: 4736
- Bytes: 135563

Normative language authority is the Omni Edition 1 specification candidate:

- File: `Omni_Complete_Specification_Edition1_1.0.0-candidate.1_updated.md`
- SHA-256: `e3ff0e1b6ef0f1f1d713647cc3c0c04dfd940513d292bac95f7b0d44e6c95f0c`

## Pre-flight findings

- Git: available (`2.47.3`).
- Host: Linux x86_64.
- `umask`: `0022`.
- Ambient `RUSTFLAGS`, `RUSTC_WRAPPER`, `CARGO_TARGET_DIR`, and `CARGO_BUILD_TARGET`: empty in the audited shell environment.
- Rustup/Cargo/rustc: not installed on this host at initialization time.
- Installed Rust targets: unavailable because Rustup is absent.
- No pre-existing `/mnt/data/omni` repository was present.
- No symlinks were introduced into the new Omni repository tree.
- Shell locale/timezone for initialization: `LC_ALL=C.UTF-8`, `TZ=UTC`.

## Blocking condition

Rust toolchain qualification cannot yet be executed. This repository initialization therefore establishes topology and safety policy only; it does not claim a passing Rust/toolchain gate.

The exact implementation toolchain must be selected and pinned at the next toolchain milestone rather than inferred from an example version.
