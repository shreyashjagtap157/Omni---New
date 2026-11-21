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

    # Check elevation for machine-wide
    if ($Scope -eq "MachineWide" -and -(Test-IsAdmin)) {
        Write-ErrorStep "Machine-wide installation requires administrator privileges."
        Write-Host "Re-run with elevated rights, or use -Scope PerUser (no elevation needed)." -ForegroundColor Yellow
        exit 1
    }

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
            $oldPath = Join-Path $Root $oldFile
            if (Test-Path $oldPath) {
                Remove-Item $oldPath -Force | Out-Null
            }
        }
    }

    # License
    if (-not (Show-License)) {
        Write-Host "Installation cancelled (license not accepted)." -ForegroundColor Yellow
        exit 0
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

    # Executables to install
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
        if (Test-Path $src) {
            if (-not (Test-Path $binDir)) {
                New-Item -ItemType Directory -Path $binDir -Force | Out-Null
            }
            Copy-Item $src (Join-Path $binDir $exe) -Force
            $manifestObj.files += @("bin\$exe")
        }
    }

    # Install runtime and stdlib (placeholders for now; actual content from build)
    Write-ProgressStep -Message "Installing runtime and standard library" -Percent 50
    $libDir = Join-Path $Root "lib"
    $stdDir = Join-Path $libDir "std"
    New-Item -ItemType Directory -Path $stdDir -Force | Out-Null

    # Create a minimal runtime marker
    $rtMarker = Join-Path $libDir "omni-rt.dll"
    if (-not (Test-Path $rtMarker)) {
        Set-Content -Path $rtMarker -Value "Omni Runtime v$SETUP_VERSION" -Encoding UTF8
    }
    $manifestObj.files += @("lib\omni-rt.dll")

    $stdMarker = Join-Path $stdDir "omni-std.oml"
    Set-Content -Path $stdMarker -Value "Omni Standard Library v$SETUP_VERSION" -Encoding UTF8
    $manifestObj.files += @("lib\std\omni-std.oml")

    # Install SDK
    Write-ProgressStep -Message "Installing SDK assets" -Percent 65
    $sdkDir = Join-Path $Root "sdk"
    $incDir = Join-Path $sdkDir "include"
    $libSdkDir = Join-Path $sdkDir "lib"
    New-Item -ItemType Directory -Path $incDir -Force | Out-Null
    New-Item -ItemType Directory -Path $libSdkDir -Force | Out-Null

    $rtHeader = Join-Path $incDir "omni-rt.h"
    Set-Content -Path $rtHeader -Value "// Omni Runtime C ABI header v$SETUP_VERSION" -Encoding UTF8
    $manifestObj.files += @("sdk\include\omni-rt.h")

    $rtLib = Join-Path $libSdkDir "omni-rt.lib"
    Set-Content -Path $rtLib -Value "Omni Runtime import library v$SETUP_VERSION" -Encoding UTF8
    $manifestObj.files += @("sdk\lib\omni-rt.lib")

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
        $fullPath = Join-Path $Root $relFile
        if (Test-Path $fullPath) {
            Remove-Item $fullPath -Force | Out-Null
            Write-Info "Removed: $relFile"
        }
    }

    # PATH cleanup
    Write-ProgressStep -Message "Cleaning up PATH" -Percent 60
    $binDir = Join-Path $Root "bin"
    Remove-OmniPathEntry -BinDir $binDir -Scope $pathScope

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

    Write-ProgressStep -Message "Verifying installation" -Percent 50
    $missing = @()
    foreach ($relFile in $manifest.files) {
        $fullPath = Join-Path $Root $relFile
        if (-not (Test-Path $fullPath)) {
            $missing += $relFile
        }
    }

    if ($missing.Count -gt 0) {
        Write-Host "Missing files detected. Re-installing missing files..." -ForegroundColor Yellow
        # Re-install missing files (simplified: copy from build output)
        $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
        $repoRoot = Resolve-Path (Join-Path $scriptDir "..\..") -ErrorAction SilentlyContinue
        $buildDir = Join-Path $repoRoot "target\debug"
        foreach ($relFile in $missing) {
            $dest = Join-Path $Root $relFile
            $src = Join-Path $buildDir (Split-Path -Leaf $relFile)
            if (Test-Path $src) {
                $destDir = Split-Path -Parent $dest
                New-Item -ItemType Directory -Path $destDir -Force | Out-Null
                Copy-Item $src $dest -Force
                Write-Info "Restored: $relFile"
            }
        }
    }

    # Re-ensure PATH
    $pathScope = if ($Scope -eq "PerUser") { "User" } else { "Machine" }
    $binDir = Join-Path $Root "bin"
    Add-OmniPathEntry -BinDir $binDir -Scope $pathScope

    Write-ProgressStep -Message "Repair complete" -Percent 100
    Write-Success "Omni installation repaired."
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

    $sboomPath = Join-Path $env:TEMP "omni-sboom-$($Manifest.scope).json"
    $sboom = @{
        version       = $Manifest.version
        scope         = $Manifest.scope
        root          = $Manifest.root
        installedOn   = $Manifest.installedOn
        files         = @()
        checksums     = @{}
    }

    foreach ($file in $Manifest.files) {
        $fullPath = Join-Path $Root $file
        if (Test-Path $fullPath) {
            $checksum = Compute-FileChecksum -Path $fullPath
            $sboom.files += @($file)
            if ($checksum) {
                $sboom.checksums[$file] = $checksum
            }
        }
    }

    $sboomJSON = $sboom | ConvertTo-Json -Depth 10
    $sboomPath | Set-Content -Encoding UTF8
    Write-Info "SBOM written to $sboomPath"
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
