# CLAUDE.md

Project guide for an **offline, privacy-first background remover** desktop app.
This file is the source of truth for architecture, conventions, and design.
Read it fully before generating or modifying code.

---

## 1. What we are building

A cross-platform desktop application that removes and replaces image backgrounds
**100% on-device**. No image ever leaves the machine; the AI models are bundled
with the app and run locally. This privacy guarantee is the product's main
differentiator versus the web tools that upload to a server.

Three ways to cut out a subject:

1. **Auto** — one click, the app removes the background automatically.
2. **Smart Select** — the user clicks on (or draws a box around) an object and the
   model segments exactly that object. Multiple clicks refine the selection
   ("add this region" / "remove that region"). This is the headline feature.
3. **Manual** — a lasso/brush on a plain canvas for hand-refining tricky edges
   (hair, fur). No model involved here, pure raster masking.

After a cutout exists, the user can keep transparency or replace the background
with a solid color, another image, or a blur of the original.

### Non-negotiable principles
- **Offline always.** Models ship inside the bundle. First launch must work with
  no network. Never add a runtime model download as the only path.
- **Privacy.** No telemetry, no uploads, no analytics calls. State this in the UI.
- **Responsive UI.** Heavy work runs off the UI thread; the interface never freezes.
- **Gradient-free design.** See §8. Depth comes from solid layered surfaces,
  hairline borders, and soft shadows — never from gradients.

---

## 2. Tech stack

| Layer        | Choice                                   | Notes |
|--------------|------------------------------------------|-------|
| Shell        | **Tauri 2**                              | System webview, small bundle, native APIs |
| Backend      | **Rust**                                 | Model inference, image I/O, heavy compute |
| Inference    | **`ort`** (ONNX Runtime bindings)        | Runs the bundled `.onnx` models |
| Image ops    | **`image`** crate (+ `fast_image_resize`)| Decode, resize, normalize, composite |
| Frontend     | **Svelte 5 + TypeScript + Vite**         | Matches existing project conventions |
| Styling      | Plain CSS with design tokens (see §8)    | No CSS framework; tokens in one file |
| Canvas       | HTML `<canvas>` 2D                        | Image preview, overlay, manual lasso/brush |

Do **not** add: Electron, a Python sidecar, Tailwind, a component library, or any
cloud/AI API. Keep dependencies minimal and justified.

---

## 3. Architecture

### 3.1 The key performance idea — read this first

Interactive selection (Smart Select) uses **SAM (Segment Anything)**, which splits
into a heavy **encoder** and a light **decoder**:

- When an image is loaded, the **encoder runs ONCE** and produces an embedding.
  This takes ~1–2 s. Cache the embedding in Rust state keyed by image id.
- Every user click then runs **only the decoder** against the cached embedding,
  which is near-instant (milliseconds). This is what makes selection feel live.

If you run the full model on every click, the tool becomes unusably slow. The
encode-once / decode-per-click split is mandatory.

### 3.2 Models

| Mode          | Model                         | Format | Input | Role |
|---------------|-------------------------------|--------|-------|------|
| Auto          | RMBG-1.4 (or BiRefNet)        | ONNX   | 1024² | Full auto matte |
| Smart Select  | MobileSAM (encoder + decoder) | ONNX   | 1024² | Prompt-driven segmentation |
| Quality (opt) | SAM2-tiny                     | ONNX   | 1024² | Optional higher-quality mode |

- Default Smart Select model is **MobileSAM** — lightweight, fast on CPU/laptop GPU.
- SAM2-tiny is an **optional** "quality mode", loaded lazily, never on startup.
- Box prompts generally give better masks than single points; support both.
- Bundle models under `src-tauri/resources/models/` and reference them via Tauri's
  resource resolver, never an absolute path.

### 3.3 Data flow

```
load image
  └─ Rust: decode + run SAM encoder once → cache embedding (image_id → embedding)

Auto mode
  └─ Rust: run RMBG → alpha matte → return mask

Smart Select
  └─ frontend sends click/box coords (+ add/remove labels)
       └─ Rust: run SAM decoder with cached embedding + prompts → return mask
            └─ frontend draws mask as a live overlay on the canvas

Manual mode
  └─ frontend only: lasso/brush edits the mask on canvas, no Rust call

Confirm / export
  └─ Rust: apply final mask to source as alpha → composite chosen background
       → encode PNG/WebP → return bytes or save to chosen path
```

Masks are passed between Rust and the frontend as compact binary (e.g. PNG-encoded
single-channel) — not giant JSON arrays.

### 3.4 Rust command surface (Tauri `#[command]`)

Keep the IPC surface small and explicit:

- `load_image(path) -> ImageMeta` — decode, run encoder, cache embedding, return
  `{ image_id, width, height }`.
- `auto_remove(image_id) -> Mask` — RMBG matte.
- `smart_select(image_id, prompts: Prompt[]) -> Mask` — decoder per prompt set.
  `Prompt` = `{ kind: "point" | "box", coords, label: "add" | "remove" }`.
- `export(image_id, mask, background: Background, format) -> bytes | saved_path` —
  `Background` = `{ transparent } | { color } | { image(path) } | { blur(radius) }`.
- `clear_image(image_id)` — drop cached embedding to free memory.

Long-running commands must be `async` and report progress via Tauri events
(`encode:progress`, etc.) so the UI can show a determinate bar.

---

## 4. Repository structure

```
.
├── CLAUDE.md
├── package.json
├── vite.config.ts
├── src/                         # Svelte 5 + TS frontend
│   ├── app.css                  # design tokens ONLY (see §8)
│   ├── lib/
│   │   ├── ipc.ts               # typed wrappers around invoke()
│   │   ├── canvas/              # render, overlay, lasso, brush
│   │   ├── stores/              # app state (Svelte runes)
│   │   └── components/          # Toolbar, Canvas, ModePanel, BackgroundPanel...
│   └── routes/ (or App.svelte)
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── resources/models/        # bundled .onnx files
    └── src/
        ├── main.rs
        ├── commands.rs          # #[command] functions
        ├── inference/
        │   ├── encoder.rs        # SAM encoder + embedding cache
        │   ├── decoder.rs        # SAM decoder (per-prompt)
        │   └── matte.rs          # RMBG auto matte
        └── compose.rs            # apply mask + background, encode output
```

---

## 5. Frontend behavior

- **State** lives in Svelte 5 runes (`$state`, `$derived`). One store for the
  active document (image_id, dims, current mask, history), one for UI (mode,
  selected background, busy flags).
- **Undo/redo** for every mask-changing action (auto, each smart click, each
  manual stroke). Keep a bounded mask-history stack.
- **Live overlay**: the current mask renders as a tinted overlay on the canvas with
  a crisp selection outline. Cutout edges must look clean at 100% zoom.
- **Zoom & pan** the canvas (wheel = zoom, space-drag = pan). Coordinates sent to
  Rust are always in original image pixels, not screen pixels.
- **Batch mode**: drop a folder → run Auto on each → review grid → bulk export.
- **Empty/loading/error states** are designed, not afterthoughts (see §8.7).

---

## 6. Build & run

```bash
npm install
npm run tauri dev      # development
npm run tauri build    # production bundles (.msi/.exe, .dmg, .deb/.AppImage)
```

ONNX Runtime: prefer the `ort` "download-binaries" feature for dev; for release,
verify the runtime is bundled so the installed app needs no system ONNX install.
Enable hardware execution providers where available (DirectML on Windows, CoreML
on macOS), always with a CPU fallback. Never hard-require a GPU.

---

## 7. Models — sourcing & bundling

- Place exported `.onnx` files in `src-tauri/resources/models/` and declare them in
  `tauri.conf.json > bundle.resources`.
- Document each model's source, license, and SHA-256 in `resources/models/README.md`.
  RMBG and SAM variants carry their own licenses — record them and surface
  attribution in an in-app "About / Licenses" view.
- SAM models export as **two** files (encoder + decoder). Keep them paired and
  versioned together.
- Quality-mode (SAM2-tiny) loads lazily on first use, not at startup, to keep cold
  start fast.

---

## 8. Design system

**Aesthetic direction: precise, calm, professional tool.** Think a focused pro
imaging utility — quiet chrome that gets out of the way so the image is the hero.
Dark theme, one confident accent, lots of breathing room. **Absolutely no
gradients anywhere** — not in backgrounds, buttons, borders, overlays, or icons.
Depth is built from: (1) stepped solid surface colors, (2) 1px hairline borders,
(3) one soft shadow token. That's it.

### 8.1 Color tokens (solid only)

Define in `src/app.css` as CSS variables. These are starting values — keep the
*structure* even if exact hex is tuned.

```css
:root {
  /* Surfaces — flat steps create elevation, no gradients */
  --bg-base:      #0E0F11;  /* app background */
  --bg-surface:   #16181B;  /* panels, toolbar */
  --bg-raised:    #1D2024;  /* cards, menus, inputs */
  --bg-hover:     #24272C;  /* hover state of raised */

  /* Hairlines & dividers */
  --border:       #2A2D33;
  --border-strong:#3A3E45;

  /* Text */
  --text:         #ECEDEE;  /* primary */
  --text-muted:   #9BA1A6;  /* secondary */
  --text-faint:   #6B7177;  /* tertiary, hints */

  /* Single accent — solid, used sparingly for primary action & selection */
  --accent:       #E8533F;  /* warm signal red-orange; pick ONE, keep it solid */
  --accent-press: #C9442F;
  --accent-fg:    #FFFFFF;

  /* Feedback */
  --ok:           #3FB07A;
  --warn:         #D8A23B;
  --danger:       #D8493B;

  /* Selection overlay (semi-transparent solid, not a gradient) */
  --select-fill:  rgba(232, 83, 63, 0.22);
  --select-line:  #E8533F;

  /* The signature: transparency checkerboard (two solid greys) */
  --checker-a:    #1B1D20;
  --checker-b:    #232629;

  /* One shadow token — soft, low, never stacked into a glow */
  --shadow: 0 1px 2px rgba(0,0,0,0.4), 0 8px 24px rgba(0,0,0,0.28);

  /* Radii */
  --r-sm: 6px; --r-md: 10px; --r-lg: 14px;

  /* Spacing scale (use these, not arbitrary px) */
  --s1:4px; --s2:8px; --s3:12px; --s4:16px; --s5:24px; --s6:32px; --s7:48px;
}
```

The accent is the only saturated color in the chrome. Keep it for: the primary
button, the active mode, and the selection outline. Everything else is neutral so
the user's image is never visually overpowered.

### 8.2 The signature motif — transparency checkerboard

Render the classic transparency checkerboard as **crisp solid squares** behind any
cutout/preview (canvas background and the export thumbnail). This is the app's
visual identity and it's inherently gradient-free. Implement with a tiled
two-color pattern (CSS `background` with hard color stops or a tiny canvas
pattern), ~12px cells. Hard edges only — no anti-aliased fade.

### 8.3 Typography

- **UI / display:** `Geist` (distinctive, modern, not generic). Weights 400/500/600.
- **Numeric & technical readouts** (dimensions, zoom %, coordinates, file sizes):
  `Geist Mono`. Monospaced numerals give the tool a precise, instrument-like feel.
- Do **not** use Inter, Roboto, Arial, or system-ui as the primary face.
- Bundle the font files locally (offline app) under `src/lib/fonts/`.
- Type scale: 12 / 13 / 14 (base UI) / 16 / 20 / 28. Tight line-height (1.3–1.4)
  for labels, 1.5 for any paragraph text. Letter-spacing slightly negative on the
  larger sizes only.

### 8.4 Layout

- Three zones: a left **tool rail** (icon buttons for the three modes + zoom),
  a center **canvas stage** (the image on the checkerboard, dominant), a right
  **inspector** (mode options, background chooser, export). Collapsible side panels.
- Generous padding (`--s4`/`--s5`), clear separation by hairline borders rather
  than boxes-in-boxes. Let the canvas breathe; the chrome is thin.
- Panels are `--bg-surface`; cards/inputs inside are `--bg-raised`; the stage is
  `--bg-base`. The step in lightness *is* the elevation.

### 8.5 Components

- **Buttons.** Solid fills only. Primary = `--accent` fill, `--accent-fg` text.
  Secondary = `--bg-raised` fill, `--border` hairline, `--text` label. Ghost =
  transparent with hairline on hover. Radius `--r-sm`. No gradient, no inner glow.
- **Mode toggle (Auto / Smart / Manual).** A segmented control: a `--bg-raised`
  track, the active segment filled with `--accent`, others plain. Active state must
  be obvious at a glance.
- **Inputs / sliders** (blur radius, brush size): flat track `--bg-raised`, solid
  `--accent` thumb/fill portion, hairline border. Show the numeric value in
  `Geist Mono`.
- **Color picker** for background-replace: a flat swatch grid + a hex field
  (mono). Include a clear "transparent" swatch shown as the checkerboard.
- **Icons:** single-weight line icons (e.g. Lucide), 1.5px stroke, `currentColor`.

### 8.6 Motion (subtle, fast)

- Durations 120–180ms, `ease-out`. Nothing bouncy in the chrome.
- **The one delightful moment:** when a Smart Select mask lands, animate the
  selection outline drawing in (~200ms) and the overlay fading to its fill. This is
  the "magic" beat — make it feel responsive and precise, not flashy.
- Hover/press: 1-step surface color change + the existing shadow; no scale jumps.
- Respect `prefers-reduced-motion`: drop the outline animation, keep instant states.

### 8.7 State design

Design these explicitly, in the same calm style:
- **Empty stage:** centered hint to drop an image, with the checkerboard faintly
  visible. Privacy line: "Images are processed entirely on your device."
- **Encoding:** determinate progress bar (driven by Rust events), accent fill.
- **Busy (per-click decode):** a small inline spinner near the cursor/inspector,
  never a full-screen block — the canvas stays interactive where possible.
- **Error:** quiet inline message in `--danger`, with a retry affordance. No modals
  for recoverable errors.

### 8.8 Hard design rules
- No gradients of any kind. If a surface needs to feel "above" another, change its
  solid color one step and/or add the shadow token.
- No glassmorphism, no neon glows, no purple-on-white, no decorative blur.
- One accent color in the chrome. The image is the only place rich color lives.
- Pixel-crisp: hairlines are exactly 1px, checkerboard edges are hard, icons align
  to the grid.

---

## 9. Implementation phases

Ship a working core first, then layer features. Don't build everything at once.

- **Phase 1 — MVP:** load image → `auto_remove` (RMBG) → show cutout on
  checkerboard → export transparent PNG. Full layout shell + design tokens.
- **Phase 2 — Smart Select:** SAM encoder-on-load + decoder-per-click, point & box
  prompts, add/remove refinement, live overlay, undo/redo.
- **Phase 3 — Manual + backgrounds:** lasso/brush mask editing; background replace
  with color / image / blur.
- **Phase 4 — Batch:** folder drop, grid review, bulk export; WebP output; quality
  mode (SAM2-tiny) toggle; About/Licenses view.

Each phase must build and run before starting the next.

---

## 10. Conventions & quality bar

- **Rust:** keep commands thin; put inference logic in `inference/` modules. Return
  typed errors (`thiserror`), never `unwrap()` on user-facing paths. Don't block the
  async runtime with CPU work — use `spawn_blocking` for inference.
- **TS/Svelte:** strict mode on. All `invoke` calls go through typed wrappers in
  `lib/ipc.ts`; components never call `invoke` directly. No `any`.
- **No magic numbers** in styles — use the spacing/radii/color tokens only.
- **Coordinates:** always convert to original-image pixels before sending to Rust;
  document the transform in `canvas/`.
- **Memory:** call `clear_image` when a document closes; don't leak embeddings.
- **Performance targets:** encode ≤ 2s on a typical laptop; per-click decode
  feels instant (< ~100ms perceived); UI thread never blocks > 1 frame.
- **Accessibility:** keyboard shortcuts for modes (A/S/M), undo/redo, export;
  focus-visible rings (solid, accent); contrast meets WCAG AA on all text tokens.
- **Comments:** explain *why*, especially around the encoder/decoder split and
  coordinate transforms.

---

## 11. Out of scope (do not add unless asked)
- Any cloud, server, login, or account system.
- Telemetry/analytics of any kind.
- Auto-updaters that require phoning home for models.
- Video segmentation (note: SAM2 supports it; keep it out of v1).