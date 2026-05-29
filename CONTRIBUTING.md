# Contributing

Thanks for the interest. This is a small, focused desktop app — there's room
to help without stepping on anyone's toes.

## Quick start

```bash
npm install
pwsh scripts/fetch-models.ps1     # or bash scripts/fetch-models.sh
npm run tauri dev
```

If `tauri dev` is slow on first launch, that's `ort` linking the bundled ONNX
Runtime — second launch is much faster.

## Project map

- **`src/`** — Svelte 5 + TypeScript frontend.
  - `app.css` is the design-token source of truth. Never hard-code color/spacing
    values; pull from variables.
  - `lib/stores/` are runes-based state. Document = current image / mask /
    cutout / history; UI = mode, busy, background, view transform; Smart =
    captured prompts.
  - `lib/components/` keep one screen-region per file.
  - `lib/canvas/render.ts` owns all canvas drawing — checkerboard, image fit,
    cutout placement, mask overlay.
  - `lib/ipc.ts` is the only place `invoke` is called. Components never reach
    Tauri directly; they go through the typed wrapper.
- **`src-tauri/`** — Rust backend.
  - `commands.rs` defines the IPC surface (`load_image`, `auto_remove`,
    `smart_select`, `export_image`, `clear_image`). Each `#[tauri::command]`
    is async + `spawn_blocking` so the runtime never stalls on CPU work.
  - `compose.rs` builds the cutout, runs foreground color decontamination,
    composites over the chosen background, and encodes PNG/WebP.
  - `inference/matte.rs` runs RMBG-1.4 (or u2net family) via `ort` and post-
    processes the alpha with a sigmoid sharpening curve.
  - `inference/encoder.rs` + `decoder.rs` are Phase 2 stubs for MobileSAM.

## Conventions

- **Rust**: no `unwrap()` on user-facing paths. Errors flow through `AppError`
  and serialize to a string the frontend can display verbatim. Don't block
  the async runtime — wrap CPU work in `spawn_blocking`.
- **TypeScript**: `strict`. No `any`. Components reach `invoke` through
  `lib/ipc.ts` only.
- **Design**: gradient-free. If a surface needs to feel "above" another, use
  the surface step + the single shadow token. One accent color, used sparingly.
- **Coordinates**: anything that goes to Rust is in original-image pixels.
  Convert with `canvas/render.ts:screenToImage()`.

## Adding a model

1. Pick a model from the table in `src-tauri/resources/models/README.md` or
   add a new row with `name`, `purpose`, `license`, and `source`.
2. Drop the URL into `scripts/fetch-models.{ps1,sh}`.
3. If preprocessing differs from RMBG-1.4 (different input size, normalization,
   input name), branch in `inference/matte.rs`.
4. Update the `MATTE_MODEL` constant in `commands.rs` to point at the new
   filename if it becomes the new default.

## Running tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run check
```

CI runs both on every push.

## Out of scope (see CLAUDE.md §11)

- Cloud, server, login, account systems.
- Telemetry or analytics.
- Auto-updaters that require phoning home for models.
- Video segmentation.

When in doubt, defer to [CLAUDE.md](CLAUDE.md). It's the architectural source
of truth.
