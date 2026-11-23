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
    $rootItem = Get-Item -LiteralPath $rootFull -Force -ErrorAction SilentlyContinue
    if ($rootItem -and (($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) {
        throw "Installation root must not be a reparse point: '$rootFull'."
    }

    $candidate = [IO.Path]::GetFullPath([IO.Path]::Combine($rootFull, $RelativePath))
    $rootPrefix = $rootFull
    if (-not $rootPrefix.EndsWith([IO.Path]::DirectorySeparatorChar)) {
        $rootPrefix += [IO.Path]::DirectorySeparatorChar
    }

    if (-not $candidate.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Install manifest path escapes the installation root: '$RelativePath'."
    }

    $current = $candidate
    while ($current -and $current.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Install manifest path traverses a reparse point: '$RelativePath'."
            }
        }
        $current = [IO.Path]::GetDirectoryName($current)
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

    if ($Scope -eq "MachineWide" -and -not (Test-IsAdmin)) {
        Write-ErrorStep "Machine-wide installation requires administrator privileges."
        Write-Host "Re-run with elevated rights, or use -Scope PerUser (no elevation needed)." -ForegroundColor Yellow
        exit 1
    }

    # Do not disturb a previous installation before license acceptance and
    # preflight validation of every mandatory source artifact.
    if (-not (Show-License)) {
        Write-Host "Installation cancelled (license not accepted)." -ForegroundColor Yellow
        exit 2
    }

    Assert-OmniSpecRootAvailable -Root $Root -InstallScope $Scope
    $pathScope = if ($Scope -eq "PerUser") { "User" } else { "Machine" }

    $scriptDir = $PSScriptRoot
    if (-not $scriptDir) {
        $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
    }
    $repoRoot = (Resolve-Path -LiteralPath (Join-Path $scriptDir "..\..") -ErrorAction Stop).Path

    $buildDir = Join-Path $repoRoot "target\release"
    if (-not (Test-Path -LiteralPath $buildDir -PathType Container)) {
        $buildDir = Join-Path $repoRoot "target\debug"
    }
    if (-not (Test-Path -LiteralPath $buildDir -PathType Container)) {
        throw "Build output directory not found: $buildDir. Build Omni first with cargo build --workspace --locked."
    }

    $executables = @("omni-driver.exe", "omni-fmt.exe", "omni-canon.exe", "omni-registry.exe")
    $missingExecutables = @($executables | Where-Object { -not (Test-Path -LiteralPath (Join-Path $buildDir $_) -PathType Leaf) })
    if ($missingExecutables.Count -gt 0) {
        throw "Required product executables are missing from '$buildDir': $($missingExecutables -join ', ')"
    }

    $runtimeDllSource = Join-Path $buildDir "omni_runtime_ffi.dll"
    if (-not (Test-Path -LiteralPath $runtimeDllSource -PathType Leaf)) {
        throw "Compiled runtime DLL is missing: $runtimeDllSource"
    }

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

    $headerSource = Join-Path $repoRoot "compiler\omni-runtime\include\omni-rt.h"
    if (-not (Test-Path -LiteralPath $headerSource -PathType Leaf)) {
        throw "Runtime C API header is missing: $headerSource"
    }

    $specSource = Join-Path $repoRoot "spec"
    if (-not (Test-Path -LiteralPath (Join-Path $specSource "manifest\omni-edition1.manifest.json") -PathType Leaf)) {
        throw "Normative specification payload is incomplete: $specSource"
    }
    $toolchainSource = Join-Path $repoRoot "rust-toolchain.toml"
    if (-not (Test-Path -LiteralPath $toolchainSource -PathType Leaf)) {
        throw "Pinned toolchain declaration is missing: $toolchainSource"
    }

    # All mandatory source artifacts are available. Now an old installation
    # can be replaced safely with respect to missing build inputs.
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
        foreach ($oldFile in $manifest.files) {
            $oldPath = Resolve-InstallOwnedPath -Root $Root -RelativePath $oldFile
            if (Test-Path -LiteralPath $oldPath -PathType Leaf) {
                Remove-Item -LiteralPath $oldPath -Force
            }
        }
    }

    if (-not (Test-Path -LiteralPath $Root -PathType Container)) {
        New-Item -ItemType Directory -Path $Root -Force | Out-Null
    }

    $manifestObj = [pscustomobject]@{
        version     = $SETUP_VERSION
        scope       = $Scope
        root        = $Root
        installedOn = (Get-Date).ToString("o")
        files       = @()
    }

    Write-ProgressStep -Message "Installing executables" -Percent 20
    $binDir = Join-Path $Root "bin"
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null
    foreach ($exe in $executables) {
        Copy-Item -LiteralPath (Join-Path $buildDir $exe) -Destination (Join-Path $binDir $exe) -Force
        $manifestObj.files += @("bin\$exe")
    }

    # Put the runtime DLL beside the tools so Windows can locate it for external
    # C clients when the installed bin directory is on PATH.
    Copy-Item -LiteralPath $runtimeDllSource -Destination (Join-Path $binDir "omni_runtime_ffi.dll") -Force
    $manifestObj.files += @("bin\omni_runtime_ffi.dll")

    Write-ProgressStep -Message "Installing C API SDK" -Percent 50
    $sdkDir = Join-Path $Root "sdk"
    $incDir = Join-Path $sdkDir "include"
    $libSdkDir = Join-Path $sdkDir "lib"
    New-Item -ItemType Directory -Path $incDir -Force | Out-Null
    New-Item -ItemType Directory -Path $libSdkDir -Force | Out-Null
    Copy-Item -LiteralPath $headerSource -Destination (Join-Path $incDir "omni-rt.h") -Force
    Copy-Item -LiteralPath $importLibSource.FullName -Destination (Join-Path $libSdkDir "omni-rt.lib") -Force
    $manifestObj.files += @("sdk\include\omni-rt.h", "sdk\lib\omni-rt.lib")

    Write-ProgressStep -Message "Installing Edition 1 specification" -Percent 65
    $shareDir = Join-Path $Root "share"
    $specDest = Join-Path $shareDir "spec"
    New-Item -ItemType Directory -Path $specDest -Force | Out-Null
    foreach ($entry in (Get-ChildItem -LiteralPath $specSource -Force)) {
        Copy-Item -LiteralPath $entry.FullName -Destination $specDest -Recurse -Force
    }
    Copy-Item -LiteralPath $toolchainSource -Destination (Join-Path $shareDir "rust-toolchain.toml") -Force
    $manifestObj.files += @("share\rust-toolchain.toml")

    # Derive each path relative to the same enumeration root. A substring of a
    # differently-spelled install root breaks when Windows uses 8.3 aliases.
    foreach ($specFile in (Get-ChildItem -LiteralPath $specDest -Recurse -File -Force)) {
        $relativeSpecFile = [IO.Path]::GetRelativePath($specDest, $specFile.FullName)
        $manifestObj.files += @((Join-Path "share\spec" $relativeSpecFile))
    }

    # omni-std is a Rust support crate, not a distributable .oml source tree.
    Write-ProgressStep -Message "Installing documentation and examples" -Percent 75
    $docDir = Join-Path $shareDir "doc"
    $examplesDir = Join-Path $shareDir "examples"
    $templatesDir = Join-Path $shareDir "templates"
    New-Item -ItemType Directory -Path $docDir -Force | Out-Null
    New-Item -ItemType Directory -Path $examplesDir -Force | Out-Null
    New-Item -ItemType Directory -Path $templatesDir -Force | Out-Null

    $readme = Join-Path $docDir "README.md"
    $readmeContent = "# Omni v$SETUP_VERSION" + [Environment]::NewLine + [Environment]::NewLine + "Installed at: $Root"
    Set-Content -LiteralPath $readme -Value $readmeContent -Encoding UTF8
    $manifestObj.files += @("share\doc\README.md")

    $hello = Join-Path $examplesDir "hello.omni"
    $helloContent = "fn main() -> i64 {" + [Environment]::NewLine + "    return 42;" + [Environment]::NewLine + "}"
    Set-Content -LiteralPath $hello -Value $helloContent -Encoding UTF8
    $manifestObj.files += @("share\examples\hello.omni")

    $tmpl = Join-Path $templatesDir "app\main.omni"
    $tmplDir = Split-Path -Parent $tmpl
    New-Item -ItemType Directory -Path $tmplDir -Force | Out-Null
    $templateContent = "fn main() -> i64 {" + [Environment]::NewLine + "    return 0;" + [Environment]::NewLine + "}"
    Set-Content -LiteralPath $tmpl -Value $templateContent -Encoding UTF8
    $manifestObj.files += @("share\templates\app\main.omni")

    $sbomPath = Join-Path $docDir "sbom.cdx.json"
    $null = Write-SBOM -Root $Root -Manifest $manifestObj -OutputPath $sbomPath
    $manifestObj.files += @("share\doc\sbom.cdx.json")

    Write-Manifest -Root $Root -Manifest $manifestObj

    Write-ProgressStep -Message "Configuring PATH" -Percent 90
    $binDirForPath = Join-Path $Root "bin"
    $null = Add-OmniPathEntry -BinDir $binDirForPath -Scope $pathScope
    Set-OmniSpecRoot -Root $Root -InstallScope $Scope

    Write-ProgressStep -Message "Installation complete" -Percent 100
    Write-Host @"
================================================================================
                    Omni v$SETUP_VERSION installed successfully

Install root: $Root
Scope:        $Scope
Bin directory: $binDirForPath

The Omni executable directory has been added to the $pathScope PATH.
Open a new terminal to use omni-driver and omni-fmt.

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
    $dirs = Get-ChildItem -LiteralPath $Root -Directory -Recurse -Force | Sort-Object -Property FullName -Descending
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
    param(
        [string] $Root,
        [pscustomobject] $Manifest,
        [string] $OutputPath
    )

    $components = @()
    foreach ($file in $Manifest.files) {
        $fullPath = Resolve-InstallOwnedPath -Root $Root -RelativePath $file
        if (Test-Path -LiteralPath $fullPath -PathType Leaf) {
            $checksum = Compute-FileChecksum -Path $fullPath
            if (-not $checksum) {
                throw "Unable to calculate SHA-256 for installed file '$file'."
            }
            $components += @{
                type = "file"
                name = $file.Replace("\", "/")
                version = $Manifest.version
                hashes = @(@{ alg = "SHA-256"; content = $checksum.ToLowerInvariant() })
            }
        }
    }

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

    $parent = Split-Path -Parent $OutputPath
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    $sbom | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
    Write-Info "CycloneDX SBOM written to $OutputPath ($($components.Count) file components)."
    return $OutputPath
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
