# Downloads the ONNX models the app needs.
#
# Models are deliberately gitignored — they're large binaries and we want a clean repo.
# Run this once after cloning, and again whenever the model list in
# src-tauri/resources/models/README.md changes.

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "src-tauri/resources/models"
New-Item -ItemType Directory -Force -Path $dest | Out-Null

# Primary: IS-Net "general use" — much sharper masks than u2netp at the cost of
# size (~170 MB) and ~3× inference time. This is what makes "Remove background"
# look professional instead of "kind of OK".
$models = @(
  @{
    name = "isnet-general-use.onnx"
    url  = "https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-general-use.onnx"
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
