# Downloads the ONNX models the app needs.
#
# Models are deliberately gitignored — they're large binaries and we want a clean repo.
# Run this once after cloning, and again whenever the model list in
# src-tauri/resources/models/README.md changes.

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "src-tauri/resources/models"
New-Item -ItemType Directory -Force -Path $dest | Out-Null

# Primary: BriaAI RMBG-1.4 — best-in-class open-weight matte model for general
# background removal. Cleanly distinguishes foreground from cluttered scenes
# (the failure mode that broke IS-Net on the beach photo). Non-commercial RAIL
# license — fine for personal/portfolio use; swap to isnet-general-use for
# fully permissive licensing.
$models = @(
  @{
    name = "rmbg-1.4.onnx"
    url  = "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx"
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
