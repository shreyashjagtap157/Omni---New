#!/usr/bin/env pwsh
# Compile and execute external C API consumers against the real MSVC library.
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$buildDir = Join-Path $repoRoot "target\debug"
$includeDir = Join-Path $repoRoot "compiler\omni-runtime\include"
$sourceDir = Join-Path $repoRoot "compiler\omni-runtime\tests"
$dll = Join-Path $buildDir "omni_runtime_ffi.dll"
$header = Join-Path $includeDir "omni-rt.h"
$importLibs = @(Get-ChildItem -LiteralPath $buildDir -File -Filter "omni_runtime_ffi*.lib" -ErrorAction SilentlyContinue)
$importLib = $importLibs | Where-Object { $_.Name.EndsWith(".dll.lib", [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
if (-not $importLib) { $importLib = $importLibs | Select-Object -First 1 }

foreach ($required in @($dll, $header)) {
    if (-not (Test-Path $required -PathType Leaf)) { throw "Required C API artifact is missing: $required" }
}
if (-not $importLib) { throw "No omni-runtime-ffi .lib import/static library found in $buildDir." }

$programFilesX86 = [Environment]::GetFolderPath("ProgramFilesX86")
$vswhere = Join-Path $programFilesX86 "Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vswhere -PathType Leaf)) { throw "Visual Studio locator is missing: $vswhere" }
$vsInstall = & $vswhere -latest -products "*" -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($vsInstall)) {
    throw "A Visual Studio installation with the x64 C/C++ build tools is required."
}
$vcvars = Join-Path $vsInstall "VC\Auxiliary\Build\vcvars64.bat"
if (-not (Test-Path $vcvars -PathType Leaf)) { throw "MSVC environment setup script is missing: $vcvars" }

$outDir = Join-Path $repoRoot "target\c-api-smoke"
New-Item -ItemType Directory -Path $outDir -Force | Out-Null
$env:PATH = "$buildDir;$env:PATH"

foreach ($name in @("c_api_basic", "c_api_write")) {
    $source = Join-Path $sourceDir "$name.c"
    $exe = Join-Path $outDir "$name.exe"
    if (-not (Test-Path $source -PathType Leaf)) { throw "C API test source is missing: $source" }

    $command = 'call "' + $vcvars + '" >nul && cl.exe /nologo /W4 /I "' +
        $includeDir + '" "' + $source + '" "' + $importLib.FullName +
        '" /Fe:"' + $exe + '" && set "PATH=' + $buildDir + ';%PATH%" && "' + $exe + '"'
    & $env:ComSpec /d /s /c $command
    if ($LASTEXITCODE -ne 0) { throw "$name failed to compile, link or execute (exit $LASTEXITCODE)." }
    Write-Host "[PASS] $name compiled, linked and executed against omni-runtime-ffi." -ForegroundColor Green
}
