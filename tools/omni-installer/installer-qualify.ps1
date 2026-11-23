#!/usr/bin/env pwsh
# Windows installer qualification. Every assertion affects the process result.
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$script:PASS_COUNT = 0
$script:FAIL_COUNT = 0
$script:TEST_ROOT = Join-Path $env:TEMP "omni-install-test-$([guid]::NewGuid().ToString('N'))"
$script:INSTALLER = Join-Path $PSScriptRoot "omni-installer.ps1"
$script:ORIGINAL_USER_PATH = [Environment]::GetEnvironmentVariable("PATH", "User")
$script:ORIGINAL_USER_SPEC_ROOT = [Environment]::GetEnvironmentVariable("OMNI_SPEC_ROOT", "User")
$script:ORIGINAL_PROCESS_SPEC_ROOT = $env:OMNI_SPEC_ROOT

function Write-TestResult {
    param([string] $Name, [bool] $Passed, [string] $Notes = "")
    if ($Passed) {
        $script:PASS_COUNT++
        Write-Host "[PASS] $Name" -ForegroundColor Green
    } else {
        $script:FAIL_COUNT++
        Write-Host "[FAIL] $Name" -ForegroundColor Red
    }
    if ($Notes) { Write-Host "      $Notes" -ForegroundColor Gray }
}

function Invoke-Installer {
    param(
        [ValidateSet("Install", "Uninstall", "Repair", "Upgrade")]
        [string] $Mode,
        [string] $Root
    )

    $pwsh = (Get-Command pwsh -ErrorAction Stop).Source
    $output = & $pwsh -NoProfile -File $script:INSTALLER -Mode $Mode -Scope PerUser -Root $Root -Quiet -AcceptLicense 2>&1
    $exitCode = $LASTEXITCODE
    foreach ($line in $output) { Write-Host $line }
    if ($exitCode -ne 0) {
        Write-TestResult "$Mode ($Root)" $false "Installer exit code $exitCode"
        return $false
    }
    Write-TestResult "$Mode ($Root)" $true
    return $true
}

function Get-PathEntries {
    param([AllowNull()][string] $Value)
    if ([string]::IsNullOrWhiteSpace($Value)) { return @() }
    return @($Value -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

function Test-PathPreservation {
    param([string] $Root, [string] $BaselineUserPath)

    $binDir = [IO.Path]::GetFullPath((Join-Path $Root "bin"))
    $before = @(Get-PathEntries $BaselineUserPath)
    $after = @(Get-PathEntries ([Environment]::GetEnvironmentVariable("PATH", "User")))
    $owned = @($after | Where-Object { $_.TrimEnd('\') -ieq $binDir.TrimEnd('\') })
    $withoutOwned = @($after | Where-Object { $_.TrimEnd('\') -ine $binDir.TrimEnd('\') })
    $unchanged = ($before.Count -eq $withoutOwned.Count)
    if ($unchanged) {
        for ($i = 0; $i -lt $before.Count; $i++) {
            if ($before[$i] -ine $withoutOwned[$i]) { $unchanged = $false; break }
        }
    }
    $passed = ($owned.Count -eq 1 -and $unchanged)
    Write-TestResult "PATH preservation and single owned entry" $passed
    return $passed
}

function Test-InstalledPayload {
    param([string] $Root)

    $required = @(
        "bin\omni-driver.exe",
        "bin\omni-fmt.exe",
        "bin\omni-canon.exe",
        "bin\omni-registry.exe",
        "lib\omni_runtime_ffi.dll",
        "sdk\include\omni-rt.h",
        "sdk\lib\omni-rt.lib",
        "share\rust-toolchain.toml",
        "share\spec\manifest\omni-edition1.manifest.json",
        "share\spec\grammar\omni-edition1.ebnf",
        "share\spec\registry\rules.json",
        "install-manifest.json"
    )
    $missing = @($required | Where-Object { -not (Test-Path (Join-Path $Root $_) -PathType Leaf) })
    Write-TestResult "Installed runtime, SDK and normative specification payload" ($missing.Count -eq 0) ($missing -join ", ")
    return ($missing.Count -eq 0)
}

function Test-LaunchAndRun {
    param([string] $Root)

    $driver = Join-Path $Root "bin\omni-driver.exe"
    $fmt = Join-Path $Root "bin\omni-fmt.exe"
    $specRoot = Join-Path $Root "share\spec"
    $oldProcessSpec = $env:OMNI_SPEC_ROOT
    try {
        $env:OMNI_SPEC_ROOT = $specRoot
        $help = & $driver --help 2>&1
        $helpCode = $LASTEXITCODE
        Write-TestResult "Installed driver --help" ($helpCode -eq 0)

        $source = Join-Path $script:TEST_ROOT "hello.omni"
        "fn main() -> i64 { return 0; }" | Set-Content -LiteralPath $source -Encoding utf8
        $runOutput = & $driver --run $source 2>&1
        $runCode = $LASTEXITCODE
        $runText = ($runOutput | Out-String)
        Write-TestResult "Installed driver --run using installed spec" ($runCode -eq 0 -and $runText.Contains("Exit code: 0")) $runText.Trim()

        $formatted = & $fmt --check $source 2>&1
        $fmtCode = $LASTEXITCODE
        Write-TestResult "Installed formatter --check" ($fmtCode -eq 0)

        $objectPath = Join-Path $script:TEST_ROOT "hello.o"
        $nativeOutput = & $driver --native -o $objectPath $source 2>&1
        $nativeCode = $LASTEXITCODE
        $sidecar = [IO.Path]::ChangeExtension($objectPath, "provenance.json")
        $nativePassed = ($nativeCode -eq 0 -and (Test-Path $objectPath -PathType Leaf) -and (Test-Path $sidecar -PathType Leaf))
        Write-TestResult "Installed native emission and provenance sidecar" $nativePassed (($nativeOutput | Out-String).Trim())
        return ($helpCode -eq 0 -and $runCode -eq 0 -and $fmtCode -eq 0 -and $nativePassed)
    } finally {
        $env:OMNI_SPEC_ROOT = $oldProcessSpec
    }
}

$rootOne = Join-Path $script:TEST_ROOT "install-one"
$rootTwo = Join-Path $script:TEST_ROOT "Omni Test Install\Omni"
$baselinePath = $script:ORIGINAL_USER_PATH
$overall = $true

try {
    New-Item -ItemType Directory -Path $script:TEST_ROOT -Force | Out-Null

    Write-Host "Building locked release binaries..." -ForegroundColor Cyan
    $build = & cargo build --workspace --locked --release 2>&1
    $buildCode = $LASTEXITCODE
    foreach ($line in $build) { Write-Host $line }
    Write-TestResult "Locked release build" ($buildCode -eq 0)
    if ($buildCode -ne 0) { throw "Release build failed." }

    $requiredBinaries = @("omni-driver.exe", "omni-fmt.exe", "omni-canon.exe", "omni-registry.exe", "omni_runtime_ffi.dll")
    $missingBinaries = @($requiredBinaries | Where-Object { -not (Test-Path (Join-Path "target\release" $_) -PathType Leaf) })
    Write-TestResult "Required product binaries are present" ($missingBinaries.Count -eq 0) ($missingBinaries -join ", ")
    if ($missingBinaries.Count -gt 0) { throw "Required release payload is incomplete." }

    if (Invoke-Installer -Mode Install -Root $rootOne) {
        $overall = (Test-PathPreservation -Root $rootOne -BaselineUserPath $baselinePath) -and $overall
        $overall = (Test-InstalledPayload -Root $rootOne) -and $overall
        $overall = (Test-LaunchAndRun -Root $rootOne) -and $overall

        $driverPath = Join-Path $rootOne "bin\omni-driver.exe"
        if (Test-Path $driverPath) { Remove-Item -LiteralPath $driverPath -Force }
        if (Invoke-Installer -Mode Repair -Root $rootOne) {
            $overall = (Test-Path $driverPath -PathType Leaf) -and $overall
            $overall = (Test-LaunchAndRun -Root $rootOne) -and $overall
        } else { $overall = $false }

        if (Invoke-Installer -Mode Upgrade -Root $rootOne) {
            $overall = (Test-InstalledPayload -Root $rootOne) -and $overall
        } else { $overall = $false }

        if (Invoke-Installer -Mode Uninstall -Root $rootOne) {
            $removed = -not (Test-Path $rootOne)
            Write-TestResult "Uninstall removes installation root" $removed
            $overall = $removed -and $overall
        } else { $overall = $false }
    } else { $overall = $false }

    if (Invoke-Installer -Mode Install -Root $rootTwo) {
        $overall = (Test-InstalledPayload -Root $rootTwo) -and $overall
        $overall = (Test-LaunchAndRun -Root $rootTwo) -and $overall
        if (Invoke-Installer -Mode Uninstall -Root $rootTwo) {
            $removed = -not (Test-Path $rootTwo)
            Write-TestResult "Uninstall from a path containing spaces" $removed
            $overall = $removed -and $overall
        } else { $overall = $false }
    } else { $overall = $false }
} catch {
    Write-TestResult "Qualification harness" $false $_.Exception.Message
    $overall = $false
} finally {
    foreach ($root in @($rootOne, $rootTwo)) {
        if (Test-Path $root) {
            try {
                & (Get-Command pwsh).Source -NoProfile -File $script:INSTALLER -Mode Uninstall -Scope PerUser -Root $root -Quiet -AcceptLicense *> $null
            } catch {}
        }
    }
    [Environment]::SetEnvironmentVariable("PATH", $script:ORIGINAL_USER_PATH, "User")
    [Environment]::SetEnvironmentVariable("OMNI_SPEC_ROOT", $script:ORIGINAL_USER_SPEC_ROOT, "User")
    $env:OMNI_SPEC_ROOT = $script:ORIGINAL_PROCESS_SPEC_ROOT
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Omni Windows Installer Qualification"
Write-Host "  PASS: $script:PASS_COUNT" -ForegroundColor Green
Write-Host "  FAIL: $script:FAIL_COUNT" -ForegroundColor Red
Write-Host "============================================================" -ForegroundColor Cyan

if (-not $overall -or $script:FAIL_COUNT -gt 0) { exit 1 }
exit 0
