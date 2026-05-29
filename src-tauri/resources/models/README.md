# Bundled models

This folder is the **only** place model weights live. They ship inside the app bundle so
first launch works with no network. (CLAUDE.md §1.)

## Quick start

```powershell
# Windows
pwsh scripts/fetch-models.ps1
```
```bash
# macOS / Linux
bash scripts/fetch-models.sh
```

The script downloads the files below and drops them in this folder. They are
**gitignored** — never commit them, just fetch them.

## Models

| File | Phase | Purpose | License | Source |
|------|-------|---------|---------|--------|
| `u2netp.onnx` | 1 (Auto) | Lightweight matte (~4.7 MB) — fast on CPU | Apache-2.0 (U-2-Net) | [rembg releases](https://github.com/danielgatis/rembg/releases/tag/v0.0.0) |
| `mobile-sam-encoder.onnx` | 2 (Smart) | Image embedding, runs once on load | Apache-2.0 | _planned_ |
| `mobile-sam-decoder.onnx` | 2 (Smart) | Prompt-driven mask | Apache-2.0 | _planned_ |
| `sam2-tiny.onnx` | 4 (Quality) | Optional higher-quality mode | _tbd_ | _planned_ |

`u2netp` is the default matte model for v1. Swap it for the heavier `u2net.onnx`
(176 MB) or `isnet-general.onnx` for cleaner edges; just change the filename in
`commands.rs` (`MATTE_MODEL` constant).

## Why SAM is two files

The encoder is heavy (~1–2 s) and runs **once** on image load — its output (the
image embedding) is cached in `Document::embedding`. Every user click then runs
only the **decoder**, which takes milliseconds. Keep the pair versioned together.
(CLAUDE.md §3.1.)
