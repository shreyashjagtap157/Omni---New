#!/usr/bin/env pwsh
#===============================================================================
# omni-installer.ps1 — Omni Programming Language Installer for Windows
#
# Installs Omni per-user (no elevation) or machine-wide (elevation required).
# Manages PATH safely and idempotently, preserving all pre-existing entries.
#
# Usage:
#   .\omni-installer.ps1                          # Interactive install
#   .\omni-installer.ps1 -Mode PerUser -Root "C:\Users\Alice\AppData\Local\Omni"
#   .\omni-installer.ps1 -Mode MachineWide -Root "C:\Program Files\Omni"
#   .\omni-installer.ps1 -Mode Uninstall -Root "C:\...\Omni"
#   .\omni-installer.ps1 -Mode Repair -Root "C:\...\Omni"
#===============================================================================
[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [ValidateSet("Install", "Uninstall", "Repair", "Upgrade")]
    [string] $Mode = "Install",

    [Parameter(Mandatory = $false)]
    [ValidateSet("PerUser", "MachineWide")]
    [string] $Scope = "PerUser",

    [Parameter(Mandatory = $false)]
    [string] $Root,

    [Parameter(Mandatory = $false)]
    [switch] $Quiet,

    [Parameter(Mandatory = $false)]
    [switch] $AcceptLicense
)

$ErrorActionPreference = 'Stop'
$SETUP_VERSION = "0.2.7"

#-------------------------------------------------------------------------------
# Helpers
#-------------------------------------------------------------------------------
function Write-ProgressStep {
    param([string] $Message, [int] $Percent)
    if (-not $Quiet) {
        Write-Progress -Activity "Omni Installer" -Status $Message -PercentComplete $Percent
    }
    Write-Host "[${Percent}%] $Message" -ForegroundColor Cyan
}

function Write-ErrorStep {
    param([string] $Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Success {
    param([string] $Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Info {
    param([string] $Message)
    Write-Host "[INFO] $Message" -ForegroundColor Gray
}

function Test-IsAdmin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $pr = New-Object Security.Principal.WindowsPrincipal($id)
    return $pr.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

#-------------------------------------------------------------------------------
# Signing workflow test (INST-002)
# Does NOT claim production-signing qualification.
# Identifies secure certificate provisioning requirements.
#-------------------------------------------------------------------------------
function Test-SigningWorkflow {
    param([switch] $Verbose)

    Write-Info "INST-002: Signing workflow test (qualification NOT claimed)."
    Write-Info "This test verifies the signing workflow structure and identifies"
    Write-Info "certificate provisioning requirements. It does NOT qualify the installer"
    Write-Info "for production use without valid code signing certificates."

    $certResult = ""
    $certs = Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue
    if ($certs) {
        $certResult = "Certificates found in CurrentUser store: $($certs.Count)"
        if ($Verbose) {
            foreach ($cert in $certs) {
                Write-Info "  - $($cert.Thumbprint): $($cert.Subject)"
            }
        }
    }
    else {
        $certResult = "No certificates found in CurrentUser store (expected - certificates required for signing)"
    }

    $localCerts = Get-ChildItem Cert:\LocalMachine\My -ErrorAction SilentlyContinue
    if ($localCerts) {
        $certResult += "`nLocalMachine certificates: $($localCerts.Count)"
        if ($Verbose) {
            foreach ($cert in $localCerts) {
                Write-Info "  - $($cert.Thumbprint): $($cert.Subject)"
            }
        }
    }
    else {
        $certResult += "`nNo certificates found in LocalMachine store (expected - certificates required for signing)"
    }

    Write-Info "INST-002: Signing workflow assessment complete."
    Write-Info "Certificate provisioning requirement: Code signing certificates"
    Write-Info "  are required for production-signing qualification."
    Write-Info "  See: https://docs.microsoft.com/en-us/powershell/module/pkgetlabsigncode"
    Write-Info "  The installer currently has no signing step; add -SignCode flag"
    Write-Info "  when certificates are available."

    return $certResult
}

#-------------------------------------------------------------------------------
# Signing implementation (INST-002)
# Signs files with an available code signing certificate.
# Does NOT claim production-trusted signing without a valid certificate.
# Test with a self-signed cert: New-SelfSignedCertificate -CertStoreLocation Cert:\CurrentUser\My -DnsName "Omni Test" -KeyExportPolicy Exportable -KeySpec Signature
#-------------------------------------------------------------------------------
function Invoke-FileSigning {
    param(
        [string[]] $Files,
        [string] $CertThumbprint
    )

    $cert = Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue |
        Where-Object { $_.Thumbprint -eq $CertThumbprint -or $CertThumbprint -eq "" }
    if (-not $cert) {
        $cert = Get-ChildItem Cert:\LocalMachine\My -ErrorAction SilentlyContinue |
            Where-Object { $_.Thumbprint -eq $CertThumbprint -or $CertThumbprint -eq "" }
    }

    if (-not $cert) {
        Write-Info "INST-002: No code signing certificate found. Files will not be signed."
        Write-Info "  To sign: obtain a code signing certificate and install it to Cert:\CurrentUser\My or Cert:\LocalMachine\My"
        Write-Info "  Test with: New-SelfSignedCertificate -CertStoreLocation Cert:\CurrentUser\My -DnsName 'Omni Test' -KeyExportPolicy Exportable -KeySpec Signature"
        return $false
    }

    Write-Info "INST-002: Signing files with certificate $($cert.Thumbprint): $($cert.Subject)"
    $signed = 0
    $failed = 0
    foreach ($file in $Files) {
        if (-not (Test-Path $file)) {
            Write-Info "  SKIP (not found): $file"
            $failed++
            continue
        }
        try {
            Set-AuthenticodeSignature -FilePath $file -Certificate $cert -ErrorAction Stop
            Write-Info "  SIGNED: $file"
            $signed++
        } catch {
            Write-Info "  FAILED: $file - $_"
            $failed++
        }
    }
    Write-Info "INST-002: Signing complete. $signed signed, $failed failed."
    return ($failed -eq 0)
}

#-------------------------------------------------------------------------------
# Resolve install root
#-------------------------------------------------------------------------------
function Resolve-InstallRoot {
    param([string] $Scope, [string] $ExplicitRoot)

    if ($ExplicitRoot) {
        # Convert to absolute path without requiring existence
        $root = Resolve-Path -Path $ExplicitRoot -ErrorAction SilentlyContinue
        if (-not $root) {
            # Path doesn't exist yet; make it absolute relative to current dir
            if (-not [System.IO.Path]::IsPathRooted($ExplicitRoot)) {
                $root = Join-Path (Get-Location) $ExplicitRoot
            }
            else {
                $root = $ExplicitRoot
            }
        }
        else {
            $root = $root.Path
        }
        return $root
    }

    if ($Scope -eq "PerUser") {
        return "$env:LOCALAPPDATA\Omni"
    }
    else {
        return "C:\Program Files\Omni"
    }
}

#-------------------------------------------------------------------------------
# Entry point
#-------------------------------------------------------------------------------
try {
    $installRoot = Resolve-InstallRoot -Scope $Scope -ExplicitRoot $Root
    Write-Host "Install root: $installRoot" -ForegroundColor Cyan
}
catch {
    Write-ErrorStep "$_"
    exit 1
}

#-------------------------------------------------------------------------------
# PATH management (safe, idempotent, preserves order)
#-------------------------------------------------------------------------------
function Get-PathEntries {
    param([string] $Scope)  # "User" or "Machine"

    if ($Scope -eq "User") {
        $raw = [Environment]::GetEnvironmentVariable("PATH", "User")
    }
    else {
        $raw = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    }

    if (-not $raw) { return @() }
    return ($raw -split ';' | Where-Object { $_ -ne "" })
}

function Set-PathEntries {
    param([string] $Scope, [string[]] $Entries)

    $newPath = ($Entries -join ';')
    if ($Scope -eq "User") {
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    }
    else {
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "Machine")
    }
}

function Add-OmniPathEntry {
    param([string] $BinDir, [string] $Scope)

    $entries = Get-PathEntries -Scope $Scope
    $binDirNormalized = $BinDir.TrimEnd('\')

    # Idempotent: do not add if already present (case-insensitive)
    foreach ($e in $entries) {
        if ($e.TrimEnd('\') -eq $binDirNormalized) {
            Write-Info "PATH already contains '$binDirNormalized' ($Scope PATH)."
            return $false
        }
    }

    $entries += $binDirNormalized
    Set-PathEntries -Scope $Scope -Entries $entries
    Write-Success "Added '$binDirNormalized' to $Scope PATH."
    return $true
}

function Remove-OmniPathEntry {
    param([string] $BinDir, [string] $Scope)

    $entries = Get-PathEntries -Scope $Scope
    $binDirNormalized = $BinDir.TrimEnd('\')
    $removed = $false

    $newEntries = @()
    foreach ($e in $entries) {
        if ($e.TrimEnd('\') -eq $binDirNormalized) {
            $removed = $true
            Write-Info "Removing '$binDirNormalized' from $Scope PATH."
        }
        else {
            $newEntries += $e
        }
    }

    if ($removed) {
        Set-PathEntries -Scope $Scope -Entries $newEntries
        Write-Success "Removed Omni entry from $Scope PATH."
    }
    else {
        Write-Info "No Omni entry found in $Scope PATH."
    }
    return $removed
}

#-------------------------------------------------------------------------------
# Install manifest
#-------------------------------------------------------------------------------
$MANIFEST_NAME = "install-manifest.json"

function Resolve-InstallOwnedPath {
    param([string] $Root, [string] $RelativePath)

    if ([string]::IsNullOrWhiteSpace($RelativePath) -or [IO.Path]::IsPathRooted($RelativePath)) {
        throw "Install manifest contains an invalid relative path: '$RelativePath'."
    }

    $rootFull = [IO.Path]::GetFullPath($Root)
    $candidate = [IO.Path]::GetFullPath([IO.Path]::Combine($rootFull, $RelativePath))
    $rootPrefix = $rootFull
    if (-not $rootPrefix.EndsWith([IO.Path]::DirectorySeparatorChar)) {
        $rootPrefix += [IO.Path]::DirectorySeparatorChar
    }

    if (-not $candidate.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Install manifest path escapes the installation root: '$RelativePath'."
    }
    return $candidate
}

function Get-EnvironmentScope {
    param([string] $InstallScope)
    if ($InstallScope -eq "PerUser") { return "User" }
    return "Machine"
}

function Assert-OmniSpecRootAvailable {
    param([string] $Root, [string] $InstallScope)

    $scopeName = Get-EnvironmentScope -InstallScope $InstallScope
    $existing = [Environment]::GetEnvironmentVariable("OMNI_SPEC_ROOT", $scopeName)
    if (-not $existing) { return }

    $existingFull = [IO.Path]::GetFullPath($existing)
    $desiredFull = [IO.Path]::GetFullPath((Join-Path $Root "share\spec"))
    if (-not $existingFull.Equals($desiredFull, [StringComparison]::OrdinalIgnoreCase)) {
        throw "OMNI_SPEC_ROOT already points to '$existingFull'. Uninstall that Omni installation before installing to '$desiredFull'."
    }
}

function Set-OmniSpecRoot {
    param([string] $Root, [string] $InstallScope)

    $scopeName = Get-EnvironmentScope -InstallScope $InstallScope
    $specRoot = [IO.Path]::GetFullPath((Join-Path $Root "share\spec"))
    [Environment]::SetEnvironmentVariable("OMNI_SPEC_ROOT", $specRoot, $scopeName)
    Write-Info "Set OMNI_SPEC_ROOT for $scopeName to '$specRoot'. Open a new terminal to use it."
}

function Remove-OmniSpecRoot {
    param([string] $Root, [string] $InstallScope)

    $scopeName = Get-EnvironmentScope -InstallScope $InstallScope
    $existing = [Environment]::GetEnvironmentVariable("OMNI_SPEC_ROOT", $scopeName)
    if (-not $existing) { return }

    $expected = [IO.Path]::GetFullPath((Join-Path $Root "share\spec"))
    $actual = [IO.Path]::GetFullPath($existing)
    if ($actual.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) {
        [Environment]::SetEnvironmentVariable("OMNI_SPEC_ROOT", $null, $scopeName)
        Write-Info "Removed the OMNI_SPEC_ROOT value owned by this installation."
    }
}

function Get-ManifestPath {
    param([string] $Root)
    return Join-Path $Root $MANIFEST_NAME
}

function Read-Manifest {
    param([string] $Root)
    $path = Get-ManifestPath -Root $Root
    if (-not (Test-Path $path)) { return @{} }
    $json = Get-Content $path -Raw
    return $json | ConvertFrom-Json
}

function Write-Manifest {
    param([string] $Root, [pscustomobject] $Manifest)
    $path = Get-ManifestPath -Root $Root
    $json = $Manifest | ConvertTo-Json -Depth 10
    $json | Set-Content $path -Encoding UTF8
}

#-------------------------------------------------------------------------------
# File copy with progress
#-------------------------------------------------------------------------------
function Copy-InstallFiles {
    param([string] $SourceDir, [string] $DestDir, [ref] $Manifest)

    $files = Get-ChildItem $SourceDir -Recurse -File
    $total = $files.Count
    $index = 0

    foreach ($file in $files) {
        $index++
        $relativePath = $file.FullName.Substring($SourceDir.Length).TrimStart('\')
        $destFile = Join-Path $DestDir $relativePath
        $destDir = Split-Path -Parent $destFile

        if (-not (Test-Path $destDir)) {
            New-Item -ItemType Directory -Path $destDir -Force | Out-Null
        }

        Copy-Item -Path $file.FullName -Destination $destFile -Force

        # Record in manifest (relative to install root)
        $relativeToRoot = $relativePath
        $Manifest.Value.files += @($relativeToRoot)

        $pct = [int](($index / $total) * 100)
        Write-ProgressStep -Message "Installing $relativePath" -Percent $pct
    }
}

#-------------------------------------------------------------------------------
# License prompt
#-------------------------------------------------------------------------------
function Show-License {
    if ($AcceptLicense) { return $true }
    if ($Quiet) {
        Write-ErrorStep "License acceptance required in quiet mode. Use -AcceptLicense."
        return $false
    }

    Write-Host @"
================================================================================
                    Omni Programming Language v$SETUP_VERSION
                    End-User License Agreement

Omni is provided under the terms of the LICENSE file in the repository.
By installing Omni you agree to the license terms.

================================================================================
"@ -ForegroundColor Gray

    while ($true) {
        $choice = Read-Host "Do you accept the license? (y/n)"
        if ($choice -eq "y" -or $choice -eq "Y") { return $true }
        if ($choice -eq "n" -or $choice -eq "N") { return $false }
        Write-Host "Please answer y or n." -ForegroundColor Yellow
    }
}

#-------------------------------------------------------------------------------
# Main install flow
#-------------------------------------------------------------------------------
function Install-Omni {
    param([string] $Root, [string] $Scope)

    Write-ProgressStep -Message "Validating installation scope" -Percent 5

    # A MachineWide install must fail for non-administrators, not administrators.
    if ($Scope -eq "MachineWide" -and -not (Test-IsAdmin)) {
        Write-ErrorStep "Machine-wide installation requires administrator privileges."
        Write-Host "Re-run with elevated rights, or use -Scope PerUser (no elevation needed)." -ForegroundColor Yellow
        exit 1
    }

    Assert-OmniSpecRootAvailable -Root $Root -InstallScope $Scope
    $pathScope = if ($Scope -eq "PerUser") { "User" } else { "Machine" }

    # Check for existing install
    $manifest = Read-Manifest -Root $Root
    $existing = $manifest.PSObject.Properties.Name -contains "files"

    if ($existing) {
        Write-Info "Existing Omni installation detected at '$Root'."
        if (-not $Quiet) {
            $choice = Read-Host "Upgrade existing installation? (y/n)"
            if ($choice -ne "y" -and $choice -ne "Y") {
                Write-Host "Installation cancelled." -ForegroundColor Yellow
                exit 0
            }
        }
        # Remove old files
        foreach ($oldFile in $manifest.files) {
            $oldPath = Resolve-InstallOwnedPath -Root $Root -RelativePath $oldFile
            if (Test-Path $oldPath) {
                Remove-Item $oldPath -Force | Out-Null
            }
        }
    }

    # License
    if (-not (Show-License)) {
        Write-Host "Installation cancelled (license not accepted)." -ForegroundColor Yellow
        exit 2
    }

    # Create root directory
    if (-not (Test-Path $Root)) {
        New-Item -ItemType Directory -Path $Root -Force | Out-Null
    }

    # Determine source directory (build output adjacent to this script)
    $scriptDir = $PSScriptRoot
    if (-not $scriptDir) {
        $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
    }
    $repoRoot = Resolve-Path (Join-Path $scriptDir "..\..") -ErrorAction SilentlyContinue
    if (-not $repoRoot) {
        $repoRoot = Resolve-Path (Join-Path $scriptDir "..") -ErrorAction SilentlyContinue
    }
    # Check release build first, then debug
    $buildDir = Join-Path $repoRoot "target\release"
    if (-not (Test-Path $buildDir)) {
        $buildDir = Join-Path $repoRoot "target\debug"
    }

    if (-not (Test-Path $buildDir)) {
        Write-ErrorStep "Build output directory not found: $buildDir"
        Write-Host "Build Omni first (cargo build --workspace --locked)." -ForegroundColor Yellow
        exit 1
    }

    # Product-facing tools that can operate from the installed payload.
    # Repository-analysis tools are intentionally not shipped as standalone
    # product commands because their source workspace is not installed.
    $executables = @(
        "omni-driver.exe",
        "omni-fmt.exe",
        "omni-canon.exe",
        "omni-registry.exe"
    )

    $manifestObj = [pscustomobject]@{
        version    = $SETUP_VERSION
        scope      = $Scope
        root       = $Root
        installedOn = (Get-Date).ToString("o")
        files      = @()
    }

    $manifestRef = [ref] $manifestObj

    # Install binaries
    Write-ProgressStep -Message "Installing executables" -Percent 20
    $binDir = Join-Path $Root "bin"
    foreach ($exe in $executables) {
        $src = Join-Path $buildDir $exe
        if (-not (Test-Path $src -PathType Leaf)) {
            throw "Required product executable is missing from the build output: $src"
        }
        if (-not (Test-Path $binDir)) {
            New-Item -ItemType Directory -Path $binDir -Force | Out-Null
        }
        Copy-Item $src (Join-Path $binDir $exe) -Force
        $manifestObj.files += @("bin\$exe")
    }

    # Install the real Rust-backed C ABI DLL. A marker file is never an ABI.
    Write-ProgressStep -Message "Installing compiled runtime and SDK" -Percent 50
    $libDir = Join-Path $Root "lib"
    New-Item -ItemType Directory -Path $libDir -Force | Out-Null
    $runtimeDllSource = Join-Path $buildDir "omni_runtime_ffi.dll"
    if (-not (Test-Path $runtimeDllSource -PathType Leaf)) {
        throw "Compiled runtime DLL is missing: $runtimeDllSource"
    }
    Copy-Item -LiteralPath $runtimeDllSource -Destination (Join-Path $libDir "omni_runtime_ffi.dll") -Force
    $manifestObj.files += @("lib\omni_runtime_ffi.dll")

    # Prefer the cdylib import library; use the static library only when the
    # selected Windows linker does not emit a separate DLL import library.
    $importLibCandidates = @(Get-ChildItem -LiteralPath $buildDir -File -Filter "omni_runtime_ffi*.lib")
    $importLibSource = $importLibCandidates |
        Where-Object { $_.Name.EndsWith(".dll.lib", [StringComparison]::OrdinalIgnoreCase) } |
        Select-Object -First 1
    if (-not $importLibSource) {
        $importLibSource = $importLibCandidates | Select-Object -First 1
    }
    if (-not $importLibSource) {
        throw "Compiled runtime import/static library is missing from $buildDir."
    }

    Write-ProgressStep -Message "Installing actual C API header and library" -Percent 65
    $sdkDir = Join-Path $Root "sdk"
    $incDir = Join-Path $sdkDir "include"
    $libSdkDir = Join-Path $sdkDir "lib"
    New-Item -ItemType Directory -Path $incDir -Force | Out-Null
    New-Item -ItemType Directory -Path $libSdkDir -Force | Out-Null
    $headerSource = Join-Path $repoRoot "compiler\omni-runtime\include\omni-rt.h"
    if (-not (Test-Path $headerSource -PathType Leaf)) {
        throw "Runtime C API header is missing: $headerSource"
    }
    Copy-Item -LiteralPath $headerSource -Destination (Join-Path $incDir "omni-rt.h") -Force
    Copy-Item -LiteralPath $importLibSource.FullName -Destination (Join-Path $libSdkDir "omni-rt.lib") -Force
    $manifestObj.files += @("sdk\include\omni-rt.h", "sdk\lib\omni-rt.lib")

    # The Rust standard-library crate is currently a support crate, not a
    # distributable .oml source library. Do not fabricate one.

    # Copy the exact manifest-bound normative specification required at runtime.
    $specSource = Join-Path $repoRoot "spec"
    if (-not (Test-Path (Join-Path $specSource "manifest\omni-edition1.manifest.json") -PathType Leaf)) {
        throw "Normative specification payload is incomplete: $specSource"
    }
    $shareDir = Join-Path $Root "share"
    $specDest = Join-Path $shareDir "spec"
    New-Item -ItemType Directory -Path $specDest -Force | Out-Null
    foreach ($entry in (Get-ChildItem -LiteralPath $specSource -Force)) {
        Copy-Item -LiteralPath $entry.FullName -Destination $specDest -Recurse -Force
    }
    $toolchainSource = Join-Path $repoRoot "rust-toolchain.toml"
    if (-not (Test-Path $toolchainSource -PathType Leaf)) {
        throw "Pinned toolchain declaration is missing: $toolchainSource"
    }
    Copy-Item -LiteralPath $toolchainSource -Destination (Join-Path $shareDir "rust-toolchain.toml") -Force
    $manifestObj.files += @("share\rust-toolchain.toml")
    foreach ($specFile in (Get-ChildItem -LiteralPath $specDest -Recurse -File)) {
        $relative = $specFile.FullName.Substring($Root.TrimEnd('\').Length).TrimStart('\')
        $manifestObj.files += @($relative)
    }

    # Install docs and examples
    Write-ProgressStep -Message "Installing documentation and examples" -Percent 75
    $shareDir = Join-Path $Root "share"
    $docDir = Join-Path $shareDir "doc"
    $examplesDir = Join-Path $shareDir "examples"
    $templatesDir = Join-Path $shareDir "templates"
    New-Item -ItemType Directory -Path $docDir -Force | Out-Null
    New-Item -ItemType Directory -Path $examplesDir -Force | Out-Null
    New-Item -ItemType Directory -Path $templatesDir -Force | Out-Null

    $readme = Join-Path $docDir "README.md"
    Set-Content -Path $readme -Value "# Omni v$SETUP_VERSION`n`nInstalled at: $Root" -Encoding UTF8
    $manifestObj.files += @("share\doc\README.md")

    $hello = Join-Path $examplesDir "hello.omni"
    Set-Content -Path $hello -Value "fn main() -> i64 {`n    return 42;`n}" -Encoding UTF8
    $manifestObj.files += @("share\examples\hello.omni")

    $tmpl = Join-Path $templatesDir "app\main.omni"
    $tmplDir = Split-Path -Parent $tmpl
    New-Item -ItemType Directory -Path $tmplDir -Force | Out-Null
    Set-Content -Path $tmpl -Value "fn main() -> i64 {`n    return 0;`n}" -Encoding UTF8
    $manifestObj.files += @("share\templates\app\main.omni")

    # Write manifest
    Write-Manifest -Root $Root -Manifest $manifestObj

    # PATH management
    Write-ProgressStep -Message "Configuring PATH" -Percent 90
    $binDirForPath = Join-Path $Root "bin"
    Add-OmniPathEntry -BinDir $binDirForPath -Scope $pathScope
    Set-OmniSpecRoot -Root $Root -InstallScope $Scope

    Write-ProgressStep -Message "Installation complete" -Percent 100

    Write-Host @"
================================================================================
                    Omni v$SETUP_VERSION installed successfully

Install root: $Root
Scope:        $Scope
Bin directory: $binDirForPath

The Omni executable directory has been added to the $pathScope PATH.
Please open a new terminal to use the `omni` command.

Quick start:
  omni-driver --help
  omni-driver --run share\examples\hello.omni
  omni-fmt --check share\examples\hello.omni

================================================================================
"@ -ForegroundColor Green
}

#-------------------------------------------------------------------------------
# Uninstall flow
#-------------------------------------------------------------------------------
function Uninstall-Omni {
    param([string] $Root, [string] $Scope)

    $pathScope = if ($Scope -eq "PerUser") { "User" } else { "Machine" }

    if (-not (Test-Path $Root)) {
        Write-ErrorStep "Install root not found: $Root"
        exit 1
    }

    $manifest = Read-Manifest -Root $Root
    $files = if ($manifest.PSObject.Properties.Name -contains "files") { $manifest.files } else { @() }

    Write-ProgressStep -Message "Removing installed files" -Percent 30
    foreach ($relFile in $files) {
        $fullPath = Resolve-InstallOwnedPath -Root $Root -RelativePath $relFile
        if (Test-Path $fullPath) {
            Remove-Item $fullPath -Force | Out-Null
            Write-Info "Removed: $relFile"
        }
    }

    # PATH cleanup
    Write-ProgressStep -Message "Cleaning up PATH" -Percent 60
    $binDir = Join-Path $Root "bin"
    Remove-OmniPathEntry -BinDir $binDir -Scope $pathScope
    Remove-OmniSpecRoot -Root $Root -InstallScope $Scope

    # Remove install-manifest.json
    $manifestPath = Get-ManifestPath -Root $Root
    if (Test-Path $manifestPath) {
        Remove-Item $manifestPath -Force | Out-Null
    }

    # Remove empty directories (bottom-up)
    Write-ProgressStep -Message "Removing empty directories" -Percent 80
    $dirs = Get-ChildItem $Root -Directory -Recurse | Sort-Object -Property FullName -Descending
    foreach ($dir in $dirs) {
        $childItems = Get-ChildItem $dir.FullName -Force -ErrorAction SilentlyContinue
        if (-not $childItems) {
            Remove-Item $dir.FullName -Force | Out-Null
        }
    }
    # Try to remove root if empty
    $remaining = Get-ChildItem $Root -Force -ErrorAction SilentlyContinue
    if (-not $remaining) {
        Remove-Item $Root -Force | Out-Null
        Write-Success "Removed install root: $Root"
    }

    Write-ProgressStep -Message "Uninstall complete" -Percent 100
    Write-Success "Omni has been uninstalled."
    Write-Host "User data under %LOCALAPPDATA%\Omni\ (config, cache, packages) was preserved." -ForegroundColor Yellow
}

#-------------------------------------------------------------------------------
# Repair flow
#-------------------------------------------------------------------------------
function Repair-Omni {
    param([string] $Root, [string] $Scope)

    if (-not (Test-Path $Root)) {
        Write-ErrorStep "Install root not found: $Root"
        exit 1
    }

    $manifest = Read-Manifest -Root $Root
    if (-not ($manifest.PSObject.Properties.Name -contains "files")) {
        Write-ErrorStep "No install manifest found. Cannot repair."
        exit 1
    }

    # Reinstall the complete manifest-bound payload from the current locked
    # build, restoring binaries, native runtime, SDK assets and spec data.
    Install-Omni -Root $Root -Scope $Scope
}

#-------------------------------------------------------------------------------
# Checksum/SBOM helpers
#-------------------------------------------------------------------------------
function Compute-FileChecksum {
    param([string] $Path)

    if (-not (Test-Path $Path)) {
        return ""
    }
    return (Get-FileHash -Algorithm SHA256 $Path).Hash
}

function Write-SBOM {
    param([string] $Root, [pscustomobject] $Manifest)

    $components = @()
    foreach ($file in $Manifest.files) {
        $fullPath = Resolve-InstallOwnedPath -Root $Root -RelativePath $file
        if (Test-Path $fullPath -PathType Leaf) {
            $checksum = Compute-FileChecksum -Path $fullPath
            if ($checksum) {
                $components += @{
                    type = "file"
                    name = $file
                    version = $Manifest.version
                    hashes = @(@{ alg = "SHA-256"; content = $checksum })
                }
            }
        }
    }

    # CycloneDX 1.5 JSON with hashes of files that actually exist.
    $sbom = @{
        bomFormat = "CycloneDX"
        specVersion = "1.5"
        version = 1
        metadata = @{
            timestamp = (Get-Date).ToUniversalTime().ToString("o")
            component = @{
                type = "application"
                name = "Omni Windows distribution"
                version = $Manifest.version
            }
        }
        components = $components
    }
    $sbomPath = Join-Path $env:TEMP "omni-sbom-$($Manifest.scope)-$([guid]::NewGuid().ToString('N')).json"
    $sbom | ConvertTo-Json -Depth 20 | Set-Content -Path $sbomPath -Encoding UTF8
    Write-Info "CycloneDX SBOM written to $sbomPath"
}

try {
    $installRoot = Resolve-InstallRoot -Scope $Scope -ExplicitRoot $Root

    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host "  Omni Programming Language Installer v$SETUP_VERSION (Windows)" -ForegroundColor Cyan
    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host "Mode:     $Mode"
    Write-Host "Scope:    $Scope"
    Write-Host "Root:     $installRoot"
    Write-Host ""

    switch ($Mode) {
        "Install"   { Install-Omni -Root $installRoot -Scope $Scope }
        "Uninstall" { Uninstall-Omni -Root $installRoot -Scope $Scope }
        "Repair"    { Repair-Omni -Root $installRoot -Scope $Scope }
        "Upgrade"   {
            Write-Host "Upgrade is equivalent to Install with existing-manifest handling." -ForegroundColor Yellow
            Install-Omni -Root $installRoot -Scope $Scope
        }
    }
}
catch {
    Write-ErrorStep "$_"
    exit 1
}
