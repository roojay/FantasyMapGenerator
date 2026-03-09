# build-wasm.ps1 — Build the renderer-wasm crate with wasm-pack and copy the
# output into the web app's public directory (Windows PowerShell version).
#
# Usage:
#   .\scripts\build-wasm.ps1            # release build (default)
#   .\scripts\build-wasm.ps1 -Dev       # debug / unoptimised build
#
# Prerequisites:
#   * Rust toolchain  (https://rustup.rs)
#   * wasm-pack       (cargo install wasm-pack)

param(
    [switch]$Dev
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Paths ──────────────────────────────────────────────────────────────────
$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot   = Split-Path -Parent $ScriptDir
$WasmCrate  = Join-Path $RepoRoot "rust\crates\renderer-wasm"
$OutDir     = Join-Path $RepoRoot "rust\apps\web\public\wasm"

# ── Mode ───────────────────────────────────────────────────────────────────
$BuildMode  = if ($Dev) { "dev" } else { "release" }
$ExtraFlags = if ($Dev) { "--dev" } else { @() }

Write-Host "==> Building WASM ($BuildMode) …"

# ── Ensure wasm-pack is available ──────────────────────────────────────────
if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "    wasm-pack not found – installing via cargo …"
    cargo install wasm-pack
}

# ── Build ──────────────────────────────────────────────────────────────────
$args = @(
    "build", $WasmCrate,
    "--target", "web",
    "--features", "wasm",
    "--out-dir", $OutDir
) + $ExtraFlags

& wasm-pack @args
if ($LASTEXITCODE -ne 0) { throw "wasm-pack failed with exit code $LASTEXITCODE" }

# ── Clean up unneeded files ────────────────────────────────────────────────
$gitignore = Join-Path $OutDir ".gitignore"
if (Test-Path $gitignore) { Remove-Item $gitignore }

Write-Host ""
Write-Host "==> WASM build complete!"
Write-Host "    Output: $OutDir"
Write-Host ""
Write-Host "    Next steps:"
Write-Host "      cd rust\apps\web && npm run dev    # start dev server with WASM"
Write-Host "      cd rust\apps\web && npm run build  # production build with WASM"
