#!/usr/bin/env bash
# Downloads the ONNX models the app needs (macOS / Linux).
# Mirror of fetch-models.ps1. Run once after cloning.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$root/src-tauri/resources/models"
mkdir -p "$dest"

declare -a models=(
  "rmbg-1.4.onnx|https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx"
)

for entry in "${models[@]}"; do
  name="${entry%%|*}"
  url="${entry##*|}"
  out="$dest/$name"
  if [[ -f "$out" ]]; then
    echo "[skip] $name already present"
    continue
  fi
  echo "[fetch] $name ..."
  curl -L --fail --progress-bar -o "$out" "$url"
  size=$(stat -c%s "$out" 2>/dev/null || stat -f%z "$out")
  printf "[ok]   %s (%.1f MB)\n" "$name" "$(echo "$size / 1048576" | bc -l)"
done

echo
echo "Done. Models are in $dest"
