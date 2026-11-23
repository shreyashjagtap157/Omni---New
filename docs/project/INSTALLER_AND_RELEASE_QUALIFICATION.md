# Omni — installer, Windows toolchain and release qualification requirements

Classification: Informative

This document preserves durable distribution and installer requirements from the owner master charter, implementation mandate, inventory and local continuation prompt. It supplements the [user requirements baseline](USER_REQUIREMENTS_BASELINE.md) and does not change any normative language rule.

## Windows platform and toolchain requirements

- Keep the pinned Rust toolchain and explicit Windows target support aligned with `rust-toolchain.toml` and CI.
- Maintain a reliable native Windows build, test, installer qualification and developer workflow.
- Provide working PowerShell and CMD driver/CLI completion where the published CLI supports those options.
- Avoid relying on WSL, an undeclared external linker, shell-specific behavior or implicit environment state for a claimed native Windows workflow.
- Record the exact shell and host, tool versions, target triple, command, exit status and output for qualification.
- Keep stable compiler correctness checks separate from nightly-only fuzz/sanitizer tasks.

## Installer lifecycle and acceptance

The toolchain installer must be a real product workflow, not a packaging stub. Its qualification scope includes:
- clean per-user installation and a clearly defined machine-wide mode only where separately supported/authorized;
- paths containing spaces and non-ASCII characters where supported;
- explicit ownership and preservation of PATH and other environment changes;
- correct CLI/toolchain discovery and documented shell integration;
- upgrade and version transition behavior;
- repair of missing/corrupted files;
- uninstall that removes only installer-owned artifacts and reverses only installer-owned environment changes;
- reinstall after uninstall and repeatable behavior on clean Windows environments;
- offline packaging/install behavior where declared;
- failure atomicity, rollback and preservation of user project data;
- installer integrity, checksum verification and software bill of materials;
- truthful diagnostics and logs without leaking secrets.

Every lifecycle scenario must run against the actual installable artifact, not merely a unit-tested helper or a mocked file tree.

## Code signing and trust

- Test certificates may validate the signing and signature-verification mechanism but do not establish public publisher trust.
- Production-trusted Authenticode signing requires the genuine owner-authorized publisher identity and certificate, protected signing process and verified release pipeline.
- Never invent, self-assign or commit a production signing identity, private key or certificate secret.
- Keep the production-trust requirement Blocked until the real external credential/authorization exists and is used in a verifiable release.
- Signed artifact verification must detect tampering, missing/invalid certificates and signature-policy failures.

## Release and artifact qualification

A release candidate must tie together exact source commit, declared product version `X.Y.Z.W`, compiler/runtime/toolchain identity, installer version, target OS/architecture, artifacts, checksums, SBOM, build provenance, test matrix, known limitations and signing state.

A development version or successful installer test does not automatically qualify the language platform or authorize a public release. Preserve immutable release artifacts and evidence. Changes to language Edition 1 semantics require the normal specification governance; installer/release documents cannot authorize them.
