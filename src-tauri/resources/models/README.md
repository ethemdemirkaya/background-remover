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
| `rmbg-1.4.onnx` | 1 (Auto) | Best-in-class matte (~170 MB, 1024² input) — production default | RAIL-M non-commercial (BriaAI) | [huggingface.co/briaai/RMBG-1.4](https://huggingface.co/briaai/RMBG-1.4) |
| `isnet-general-use.onnx` (alt) | 1 (Auto) | High-quality permissive alternative | Apache-2.0 (IS-Net) | [rembg releases](https://github.com/danielgatis/rembg/releases/tag/v0.0.0) |
| `u2netp.onnx` (alt) | 1 (Auto, fast) | Lightweight matte (~4.7 MB, 320² input) | Apache-2.0 (U-2-Net) | [rembg releases](https://github.com/danielgatis/rembg/releases/tag/v0.0.0) |
| `mobile-sam-encoder.onnx` | 2 (Smart) | Image embedding, runs once on load | Apache-2.0 | _planned_ |
| `mobile-sam-decoder.onnx` | 2 (Smart) | Prompt-driven mask | Apache-2.0 | _planned_ |
| `sam2-tiny.onnx` | 4 (Quality) | Optional higher-quality mode | _tbd_ | _planned_ |

`rmbg-1.4` is the production default — BriaAI's RMBG-1.4 is widely regarded as
the strongest open-weight model for general background removal, with much
better discrimination on cluttered scenes than IS-Net (which kept beach pixels
classed as foreground on portrait test shots). Output is paired with a
foreground color decontamination pass in the renderer to strip background
color bleed from soft edges.

Switch the `MATTE_MODEL` constant in `commands.rs` to fall back to
`isnet-general-use.onnx` (fully permissive Apache-2.0) or `u2netp.onnx`
(fast/small) if RMBG's non-commercial license is a blocker.

## Why SAM is two files

The encoder is heavy (~1–2 s) and runs **once** on image load — its output (the
image embedding) is cached in `Document::embedding`. Every user click then runs
only the **decoder**, which takes milliseconds. Keep the pair versioned together.
(CLAUDE.md §3.1.)
