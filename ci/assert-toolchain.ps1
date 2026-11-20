# ci/assert-toolchain.ps1 — Omni Toolchain Qualification Gate (Windows PowerShell)
# Mirrors ci/assert-toolchain.sh for native Windows runners.
# Fail-closed: any failure exits non-zero.

$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$workspaceRoot = Resolve-Path (Join-Path $scriptDir "..")
$toolchainFile = Join-Path $workspaceRoot "rust-toolchain.toml"

# Foundation contract targets per spec/release/foundation-gate.json:
# x86_64-unknown-linux-gnu + riscv64-unknown-none-elf (plus x86_64-pc-windows-msvc
# for Windows host/cross-compilation support).
# Rust 1.95.0 renamed riscv64-unknown-none-elf -> riscv64gc-unknown-none-elf.
$expectedComponents = @("rustfmt", "clippy", "rust-src", "llvm-tools-preview")
$requiredTargets = @("x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "riscv64gc-unknown-none-elf")

function Fail([string] $message) {
    Write-Host "[-] FAIL-CLOSED: $message" -ForegroundColor Red
    exit 101
}

Write-Host "=== [Omni CI] Toolchain Qualification Gate (Windows) ==="

if (!(Test-Path $toolchainFile)) {
    Fail "Toolchain definition file missing at $toolchainFile"
}

# Parse pinned channel from rust-toolchain.toml.
$toolchainContent = Get-Content $toolchainFile -Raw
if ($toolchainContent -match 'channel\s*=\s*"([^"]+)"') {
    $expectedChannel = $matches[1]
} else {
    Fail "Failed to parse 'channel' from $toolchainFile"
}
Write-Host "[+] Pinned toolchain channel: $expectedChannel"

# Verify required binaries are on PATH.
foreach ($bin in @("rustup", "rustc", "cargo", "rustfmt", "cargo-clippy")) {
    if (!(Get-Command $bin -ErrorAction SilentlyContinue)) {
        Fail "Required binary '$bin' not found in PATH."
    }
}

# Verify active toolchain matches pinned channel.
$activeToolchain = & rustup show active-toolchain
$activeToolchainName = ($activeToolchain -split " ")[0]
if (-not $activeToolchainName.StartsWith($expectedChannel)) {
    Fail "Active rustup toolchain ($activeToolchainName) does not match pinned channel ($expectedChannel)."
}

# Verify rustc version matches pinned channel.
$activeRustcVersion = & rustc --version
Write-Host "[+] Detected rustc version: $activeRustcVersion"
if (-not $activeRustcVersion.Contains(" $expectedChannel ")) {
    Fail "Active rustc ($activeRustcVersion) does not match pinned channel ($expectedChannel)."
}

# Verify required components are installed.
$installedComponents = & rustup component list --installed --toolchain $expectedChannel 2>$null
foreach ($component in $expectedComponents) {
    $baseComponent = $component -replace '-preview$', ''
    $found = $installedComponents | Where-Object { $_ -match "^($component|$baseComponent)(-|$)" }
    if (-not $found) {
        Fail "Required component '$component' is not installed for $expectedChannel."
    }
    Write-Host "[+] Verified component: $component"
}

# Verify required targets are installed.
$installedTargets = & rustup target list --installed --toolchain $expectedChannel 2>$null
foreach ($target in $requiredTargets) {
    $found = $installedTargets | Where-Object { $_ -eq $target }
    if (-not $found) {
        Fail "Required target '$target' is not installed for $expectedChannel."
    }
    Write-Host "[+] Verified target: $target"
}

& rustfmt --version > $null 2>&1 || Fail "rustfmt execution failed."
& cargo clippy --version > $null 2>&1 || Fail "cargo-clippy execution failed."

$rustupRustcVersion = & rustup run $expectedChannel rustc --version
if ($rustupRustcVersion -ne $activeRustcVersion) {
    Fail "Active rustc differs from rustup toolchain $expectedChannel rustc."
}

Write-Host "[+] PASS: Toolchain qualification verified successfully against $expectedChannel."
exit 0
