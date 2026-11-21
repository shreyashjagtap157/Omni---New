#!/usr/bin/env pwsh
#===============================================================================
# installer-qualify.ps1 — Windows Installer Qualification
#
# Qualifies the Omni Windows installer on clean Windows environments.
# Tests: install, uninstall, repair, upgrade, reinstall, PATH management,
#        non-admin scope, paths with spaces, launch/build/run/test.
#
# Usage: .\installer-qualify.ps1
#===============================================================================
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$SETUP_VERSION = "0.2.7"
$TEST_ROOT = Join-Path $env:TEMP "omni-install-test-$([guid]::NewGuid().ToString('N'))"
$PASS_COUNT = 0
$FAIL_COUNT = 0

function Write-TestResult {
    param([string] $Test, [string] $Status, [string] $Notes = "")
    if ($Status -eq "PASS") {
        $PASS_COUNT++
        Write-Host "[PASS] $Test" -ForegroundColor Green
    } else {
        $FAIL_COUNT++
        Write-Host "[FAIL] $Test" -ForegroundColor Red
    }
    if ($Notes) { Write-Host "      $Notes" -ForegroundColor Gray }
}

function Test-Installer {
    param([string] $Mode, [string] $Scope, [string] $Root, [switch] $Quiet)

    # Dot-source the installer so its functions are available in this scope
    . "$PSScriptRoot/omni-installer.ps1" -Mode $Mode -Scope $Scope -Root $Root -Quiet:$Quiet -AcceptLicense:$Quiet 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-TestResult "$Mode ($Scope)" "FAIL" "Exit code $LASTEXITCODE"
        return $false
    }
    Write-TestResult "$Mode ($Scope)" "PASS"
    return $true
}

function Test-PathPreservation {
    param([string] $Root)

    $binDir = Join-Path $Root "bin"
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $machinePath = [Environment]::GetEnvironmentVariable("PATH", "Machine")

    $userContains = $userPath -split ';' | Where-Object { $_ -eq $binDir }
    $machineContains = $machinePath -split ';' | Where-Object { $_ -eq $binDir }

    if ($userContains -or $machineContains) {
        Write-TestResult "PATH preservation" "PASS" "Omni bin dir found in PATH"
        return $true
    } else {
        Write-TestResult "PATH preservation" "FAIL" "Omni bin dir not found in PATH"
        return $false
    }
}

function Test-LaunchAndRun {
    param([string] $Root)

    Write-Host "DEBUG: Root=$Root" -ForegroundColor Gray
    $driver = Join-Path $Root "bin" "omni-driver.exe"
    Write-Host "DEBUG: Driver path=$driver" -ForegroundColor Gray
    Write-Host "DEBUG: Test-Path=$(Test-Path $driver)" -ForegroundColor Gray

    if (-not (Test-Path $driver)) {
        Write-TestResult "Driver launch" "FAIL" "omni-driver.exe not found at $driver"
        return $false
    }

    $help = & $driver --help 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-TestResult "Driver --help" "FAIL" "Exit code $LASTEXITCODE"
        return $false
    }
    Write-TestResult "Driver --help" "PASS"

    # Test --run with a simple Omni program
    $testScript = Join-Path $env:TEMP "omni-qual-test.omni"
    "fn main() -> i64 { return 42; }" | Set-Content -Path $testScript -Encoding UTF8

    $env:OMNI_SPEC_ROOT = Resolve-Path (Join-Path $PSScriptRoot "..\..\spec") -ErrorAction SilentlyContinue
    $runResult = & $driver $testScript 2>&1
    $runCode = $LASTEXITCODE
    if ($runCode -ne 0) {
        Write-TestResult "Driver --run" "FAIL" "Expected exit 0, got $runCode"
        return $false
    }
    Write-TestResult "Driver --run (exit 0)" "PASS"
    return $true
}

#-------------------------------------------------------------------------------
# Main qualification
#-------------------------------------------------------------------------------
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  Omni Windows Installer Qualification v$SETUP_VERSION" -ForegroundColor Cyan
Write-Host "  Test root: $TEST_ROOT" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

# Build release binaries first
Write-Host ""
Write-Host "Step 1: Building release binaries..." -ForegroundColor Cyan
$buildResult = & cargo build --workspace --locked --release 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-TestResult "Release build" "FAIL" "cargo build failed"
    exit 1
}
Write-TestResult "Release build" "PASS"

# Collect executables
$buildDir = Join-Path (Resolve-Path "target/release") ""
$executables = @(
    "omni-driver.exe",
    "omni-fmt.exe",
    "omni-canon.exe",
    "omni-conform.exe",
    "omni-audit.exe",
    "omni-registry.exe",
    "omni-topology.exe",
    "omni-bindgen.exe"
)

$missing = @()
foreach ($exe in $executables) {
    $exePath = Join-Path $buildDir $exe
    if (-not (Test-Path $exePath)) {
        $missing += $exe
    }
}
if ($missing.Count -gt 0) {
    Write-TestResult "Executables present" "FAIL" "Missing: $($missing -join ', ')"
    exit 1
}
Write-TestResult "Executables present" "PASS" "$($executables.Count) executables"

#-------------------------------------------------------------------------------
# Test 1: Per-user install (no elevation)
#-------------------------------------------------------------------------------
Write-Host ""
Write-Host "Step 2: Per-user installation (no elevation)..." -ForegroundColor Cyan
$testRoot1 = Join-Path $TEST_ROOT "install1"
if (Test-Installer -Mode "Install" -Scope "PerUser" -Root $testRoot1 -Quiet) {
    Test-PathPreservation -Root $testRoot1
    Test-LaunchAndRun -Root $testRoot1
}

#-------------------------------------------------------------------------------
# Test 2: Install with spaces in path
#-------------------------------------------------------------------------------
Write-Host ""
Write-Host "Step 3: Installation with spaces in path..." -ForegroundColor Cyan
$testRoot2 = Join-Path $env:TEMP "Omni Test Install" "Omni"
if (Test-Installer -Mode "Install" -Scope "PerUser" -Root $testRoot2 -Quiet) {
    Test-LaunchAndRun -Root $testRoot2
}

#-------------------------------------------------------------------------------
# Test 3: Upgrade (re-install over existing)
#-------------------------------------------------------------------------------
Write-Host ""
Write-Host "Step 4: Upgrade (re-install over existing)..." -ForegroundColor Cyan
if (Test-Installer -Mode "Install" -Scope "PerUser" -Root $testRoot1 -Quiet) {
    Write-TestResult "Upgrade" "PASS" "Re-install over existing succeeded"
}

#-------------------------------------------------------------------------------
# Test 4: Repair
#-------------------------------------------------------------------------------
Write-Host ""
Write-Host "Step 5: Repair..." -ForegroundColor Cyan
if (Test-Installer -Mode "Repair" -Scope "PerUser" -Root $testRoot1 -Quiet) {
    Write-TestResult "Repair" "PASS"
}

#-------------------------------------------------------------------------------
# Test 5: Uninstall
#-------------------------------------------------------------------------------
Write-Host ""
Write-Host "Step 6: Uninstall..." -ForegroundColor Cyan
if (Test-Installer -Mode "Uninstall" -Scope "PerUser" -Root $testRoot1 -Quiet) {
    Write-TestResult "Uninstall" "PASS"
}

#-------------------------------------------------------------------------------
# Test 6: Reinstall after uninstall
#-------------------------------------------------------------------------------
Write-Host ""
Write-Host "Step 7: Reinstall after uninstall..." -ForegroundColor Cyan
if (Test-Installer -Mode "Install" -Scope "PerUser" -Root $testRoot1 -Quiet) {
    Test-LaunchAndRun -Root $testRoot1
    Write-TestResult "Reinstall after uninstall" "PASS"
}

#-------------------------------------------------------------------------------
# Summary
#-------------------------------------------------------------------------------
Write-Host ""
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  Qualification Summary" -ForegroundColor Cyan
Write-Host "  PASS: $PASS_COUNT" -ForegroundColor Green
Write-Host "  FAIL: $FAIL_COUNT" -ForegroundColor Red
Write-Host "================================================================================" -ForegroundColor Cyan

if ($FAIL_COUNT -gt 0) {
    exit 1
}
exit 0
