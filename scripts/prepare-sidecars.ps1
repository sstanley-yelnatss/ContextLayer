# Build ContextLayer CLI binaries and stage them for Tauri externalBin bundling.
# Run from repo root: powershell -File scripts/prepare-sidecars.ps1

$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$BinDir = Join-Path $Root "apps\desktop\src-tauri\binaries"
$Triple = (rustc --print host-tuple).Trim()

if (-not $Triple) {
    throw "Failed to read rustc host target triple"
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

Write-Host "Building sidecars (target: $Triple)..."
Push-Location $Root
try {
    cargo build -p contextlayer-recorder -p contextlayer-trace-cli --release
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed for recorder/trace (exit $LASTEXITCODE)"
    }

    cargo build -p contextlayer-mcp --release
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "contextlayer-mcp rebuild failed (exit $LASTEXITCODE). Often caused by Cursor holding the exe open."
        Write-Warning "Disable ContextLayer MCP in Cursor and re-run, or continue if target/release/contextlayer-mcp.exe already exists."
    }
} finally {
    Pop-Location
}

$Release = Join-Path $Root "target\release"
$Pairs = @(
    @{ Src = "contextlayer-recorder.exe"; Base = "contextlayer-recorder" },
    @{ Src = "contextlayer-mcp.exe"; Base = "contextlayer-mcp" },
    @{ Src = "contextlayer-trace.exe"; Base = "contextlayer-trace" }
)

foreach ($pair in $Pairs) {
    $srcPath = Join-Path $Release $pair.Src
    if (-not (Test-Path $srcPath)) {
        throw "Missing build output: $srcPath - cannot bundle installer"
    }
    $dstPath = Join-Path $BinDir "$($pair.Base)-$Triple.exe"
    Copy-Item -Path $srcPath -Destination $dstPath -Force
    Write-Host "  $($pair.Base) -> $dstPath"
}

Write-Host "Sidecars ready for Tauri bundle."
