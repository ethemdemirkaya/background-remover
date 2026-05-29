# Downloads the ONNX models the app needs.
#
# Models are deliberately gitignored — they're large binaries and we want a clean repo.
# Run this once after cloning, and again whenever the model list in
# src-tauri/resources/models/README.md changes.

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "src-tauri/resources/models"
New-Item -ItemType Directory -Force -Path $dest | Out-Null

$models = @(
  @{
    name = "u2netp.onnx"
    url  = "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2netp.onnx"
    sha256 = "8e83ca70e441ab06c318d82300c84d6936c9bf2dfc7ac4c4f9c1e7e1d4f3a8b8"
  }
)

foreach ($m in $models) {
  $out = Join-Path $dest $m.name
  if (Test-Path $out) {
    Write-Host "[skip] $($m.name) already present"
    continue
  }
  Write-Host "[fetch] $($m.name) ..."
  Invoke-WebRequest -Uri $m.url -OutFile $out -UseBasicParsing
  $size = (Get-Item $out).Length
  Write-Host "[ok]   $($m.name) ($([math]::Round($size/1MB, 1)) MB)"
}

Write-Host ""
Write-Host "Done. Models are in $dest"
