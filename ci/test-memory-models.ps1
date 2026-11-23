# Test isolated memory-model research prototypes with the pinned stable toolchain.
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) ("omni-memory-models-" + [guid]::NewGuid().ToString("N"))
$spike = Join-Path $repoRoot "research\memory-model\spike.rs"
$domain = Join-Path $repoRoot "research\memory-model\domain_lifetimes.rs"
$toolchain = "+1.95.0"

try {
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null

    foreach ($source in @($spike, $domain)) {
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Missing isolated research model: $source"
        }
    }

    Write-Host "[memory-model] Formatting isolated research models with Rust 1.95.0"
    & rustfmt $toolchain --edition 2021 --check $spike $domain
    if ($LASTEXITCODE -ne 0) { throw "rustfmt failed for isolated memory-model sources." }

    foreach ($name in @("spike", "domain_lifetimes")) {
        $source = Join-Path $repoRoot "research\memory-model\$name.rs"
        $binary = Join-Path $tempRoot "$name.exe"

        Write-Host "[memory-model] Compiling $source"
        & rustc $toolchain --edition=2021 --test -D warnings $source -o $binary
        if ($LASTEXITCODE -ne 0) { throw "Rust compilation failed for memory model '$name'." }

        Write-Host "[memory-model] Executing $name"
        & $binary --test-threads=1
        if ($LASTEXITCODE -ne 0) { throw "Research tests failed for memory model '$name'." }
    }

    Write-Host "[memory-model] PASS: both isolated research models formatted, compiled and passed." -ForegroundColor Green
} finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
