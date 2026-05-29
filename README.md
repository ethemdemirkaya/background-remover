# Background Remover

Offline, privacy-first background remover for the desktop. Images never leave your
machine — models are bundled and run locally.

Built with **Tauri 2**, **Rust**, **ONNX Runtime** (via `ort`), and **Svelte 5**.

## Features (current)

- **Auto** — one-click removal via the bundled U-2-Net matte model
- Background replace: transparent, solid color, blur original, or image
- PNG / WebP export with a real alpha matte
- Zoom + pan canvas with the signature transparency checkerboard
- Undo / redo for every mask-changing action
- Dark, gradient-free, instrument-like UI (Geist + Geist Mono)

Coming soon: **Smart Select** (MobileSAM, encoder-once / decoder-per-click),
**Manual** lasso + brush refinement, and batch folder processing. See
[CLAUDE.md](CLAUDE.md) for the full design.

## Getting started

```bash
# 1. install deps
npm install

# 2. download the matte model (~4.7 MB, gitignored)
pwsh scripts/fetch-models.ps1   # Windows
bash scripts/fetch-models.sh    # macOS / Linux

# 3. run
npm run tauri dev
```

Release bundle:

```bash
npm run tauri build
```

## Repo layout

```
src/                  # Svelte 5 + TypeScript frontend
src-tauri/            # Rust backend, Tauri shell, ONNX inference
src-tauri/resources/  # Bundled models (gitignored binaries)
scripts/              # Helper scripts (model fetch)
CLAUDE.md             # Project guide / architecture source-of-truth
```

## License

MIT — see [LICENSE](LICENSE). Bundled model weights carry their own licenses,
listed in [`src-tauri/resources/models/README.md`](src-tauri/resources/models/README.md).
