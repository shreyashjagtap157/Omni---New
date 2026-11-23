# ci/pipeline.ps1 — Omni CI Pipeline (Windows PowerShell edition)
# Mirrors ci/pipeline.sh for native Windows runners.
# Runs under pwsh with $ErrorActionPreference = 'Stop' (fail-closed).

$ErrorActionPreference = 'Stop'
$MAX_TIME = 1800
$FUZZ_BUDGET = 300

Write-Host "CI mode=main ceiling=${MAX_TIME}s fuzz_budget=${FUZZ_BUDGET}s/target"

# Low-memory guard: cap any Rust operation under ~500MB via single-job builds.
$env:CARGO_BUILD_JOBS = 1
$env:CARGO_BUILD_CODEGEN_UNITS = 1

# Assert pinned toolchain.
& ./ci/assert-toolchain.ps1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Isolated research models are compiled and tested outside the production workspace.
& ./ci/test-memory-models.ps1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[Step] Test strict inventory validator"
& python ./ci/tests/test_validate_inventory_strict.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Umask equivalent (Windows ACLs are not POSIX; skip the 0022 check).

# Formatting check.
Write-Host "[Step] cargo fmt --all -- --check"
& cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Workspace check.
Write-Host "[Step] cargo check --workspace --locked"
& cargo check -j 1 --workspace --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Strict Clippy.
Write-Host "[Step] cargo clippy --workspace --all-targets --locked -- -D warnings"
& cargo clippy -j 1 --workspace --all-targets --locked -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Workspace tests.
Write-Host "[Step] cargo test --workspace --locked"
& cargo test -j 1 --workspace --locked -- --test-threads=1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Locked metadata.
Write-Host "[Step] cargo metadata"
& cargo metadata --format-version 1 --locked > $null
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Conformance, linkage, topology gates.
Write-Host "[Step] omni-conform"
& cargo run -j 1 -p omni-conform --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[Step] omni-audit"
& cargo run -j 1 -p omni-audit --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[Step] omni-topology"
& cargo run -j 1 -p omni-topology --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Canonical specification-tree identity.
Write-Host "[Step] Verify spec-tree digest"
$manifest = Get-Content ./spec/manifest/omni-edition1.manifest.json | ConvertFrom-Json
$expected = $manifest.spec_tree_sha256
$actual = & cargo run -j 1 -p omni-canon --locked -- --spec-tree ./spec
if ($actual.Trim() -ne $expected) {
    Write-Host "FAIL: Spec-tree digest mismatch: expected $expected, got $actual" -ForegroundColor Red
    exit 1
}
Write-Host "PASS: Spec-tree digest $actual verified against the manifest."

Write-Host "[+] PASS: Omni CI Pipeline (Windows) completed."
exit 0
