<script lang="ts">
  import { save, open } from "@tauri-apps/plugin-dialog";
  import { doc } from "../stores/document.svelte";
  import { ui } from "../stores/ui.svelte";
  import { ipc, type Background, type ExportFormat } from "../ipc";
  import { copyCutoutToClipboard } from "../actions/clipboard";

  let format = $state<ExportFormat>("png");
  let colorHex = $state("#ffffff");
  let blurRadius = $state(20);
  let bgImagePath = $state<string | null>(null);
  let copied = $state(false);

  // Push local control changes into the store *only when the kind is already that kind*
  // and the actual value differs — avoids an infinite reactive loop (each assignment
  // is a fresh object, which would otherwise retrigger the effect).
  $effect(() => {
    if (ui.background.kind === "color" && ui.background.hex !== colorHex) {
      ui.background = { kind: "color", hex: colorHex };
    }
  });
  $effect(() => {
    if (ui.background.kind === "blur" && ui.background.radius !== blurRadius) {
      ui.background = { kind: "blur", radius: blurRadius };
    }
  });

  async function runAuto() {
    if (!doc.imageId) return;
    ui.setError(null);
    ui.setBusy("Removing background...");
    try {
      const result = await ipc.autoRemove(doc.imageId);
      doc.pushAutoResult(result.mask, result.cutout, "auto");
    } catch (e) {
      ui.setError(String(e));
    } finally {
      ui.clearBusy();
    }
  }

  function setBackground(kind: Background["kind"]) {
    if (kind === "transparent") ui.background = { kind: "transparent" };
    else if (kind === "color") ui.background = { kind: "color", hex: colorHex };
    else if (kind === "blur") ui.background = { kind: "blur", radius: blurRadius };
    else if (kind === "image") {
      if (bgImagePath) ui.background = { kind: "image", path: bgImagePath };
      else pickBgImage();
    }
  }

  async function pickBgImage() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "bmp"] }],
    });
    if (typeof selected === "string") {
      bgImagePath = selected;
      ui.background = { kind: "image", path: selected };
    }
  }

  async function copyCutout() {
    const ok = await copyCutoutToClipboard();
    if (ok) {
      copied = true;
      setTimeout(() => { copied = false; }, 1200);
    }
  }

  async function doExport() {
    if (!doc.imageId || !doc.mask) return;
    const ext = format === "webp" ? "webp" : "png";
    const path = await save({
      defaultPath: `cutout.${ext}`,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    });
    if (!path) return;
    ui.setError(null);
    ui.setBusy("Exporting...");
    try {
      await ipc.exportImage({
        imageId: doc.imageId,
        mask: doc.mask,
        background: ui.background,
        format,
        savePath: path,
      });
    } catch (e) {
      ui.setError(String(e));
    } finally {
      ui.clearBusy();
    }
  }
</script>

<aside class="inspector" aria-label="Inspector">
  <section class="panel">
    <header><h3>{ui.mode === "auto" ? "Auto" : ui.mode === "smart" ? "Smart Select" : "Manual"}</h3></header>

    {#if ui.mode === "auto"}
      <p class="muted small">One click removes the background using a bundled model.</p>
      <button type="button" class="btn primary" disabled={!doc.hasImage || ui.busy} onclick={runAuto}>
        Remove background
      </button>
    {:else if ui.mode === "smart"}
      <p class="muted small">Click on the subject to select. Shift-click adds, alt-click removes.</p>
      <p class="faint small">Coming in Phase 2.</p>
    {:else}
      <p class="muted small">Refine the mask with a brush or lasso.</p>
      <p class="faint small">Coming in Phase 3.</p>
    {/if}
  </section>

  <section class="panel">
    <header><h3>Background</h3></header>
    <div class="swatches">
      <button
        type="button"
        class="swatch checker"
        class:active={ui.background.kind === "transparent"}
        title="Transparent"
        aria-label="Transparent background"
        onclick={() => setBackground("transparent")}
      ></button>
      <button
        type="button"
        class="swatch"
        class:active={ui.background.kind === "color"}
        style="background: {colorHex}"
        title="Solid color"
        aria-label="Solid color background"
        onclick={() => setBackground("color")}
      ></button>
      <button
        type="button"
        class="swatch"
        class:active={ui.background.kind === "blur"}
        title="Blur original"
        aria-label="Blur original background"
        onclick={() => setBackground("blur")}
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="6" opacity="0.45"/><circle cx="12" cy="12" r="3"/></svg>
      </button>
      <button
        type="button"
        class="swatch"
        class:active={ui.background.kind === "image"}
        title="Image"
        aria-label="Image background"
        onclick={() => setBackground("image")}
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="4" y="5" width="16" height="14" rx="1"/><circle cx="9" cy="10" r="1.5"/><path d="M5 17l5-5 4 4 3-3 3 3"/></svg>
      </button>
    </div>

    {#if ui.background.kind === "color"}
      <label class="field">
        <span class="muted small">Hex</span>
        <input type="text" class="mono" bind:value={colorHex} maxlength="7" />
      </label>
    {:else if ui.background.kind === "blur"}
      <label class="field">
        <span class="muted small">Radius <span class="mono faint">{blurRadius}px</span></span>
        <input type="range" min="2" max="80" step="1" bind:value={blurRadius} />
      </label>
    {:else if ui.background.kind === "image"}
      <div class="field">
        <span class="muted small">Image</span>
        <div class="row">
          <input type="text" class="mono grow" readonly value={bgImagePath ?? ""} placeholder="No file chosen" />
          <button type="button" class="btn ghost" onclick={pickBgImage}>Browse...</button>
        </div>
      </div>
    {/if}
  </section>

  <section class="panel">
    <header><h3>Export</h3></header>
    <div class="row">
      <label class="field grow">
        <span class="muted small">Format</span>
        <select bind:value={format}>
          <option value="png">PNG</option>
          <option value="webp">WebP</option>
        </select>
      </label>
      <button type="button" class="btn ghost" disabled={!doc.cutout || ui.busy} onclick={copyCutout} title="Copy cutout to clipboard (Ctrl+C)">
        {copied ? "Copied" : "Copy"}
      </button>
      <button type="button" class="btn primary" disabled={!doc.mask || ui.busy} onclick={doExport}>
        Save...
      </button>
    </div>
    {#if doc.imageId}
      <p class="faint small mono">{doc.width} x {doc.height}</p>
    {/if}
  </section>

  <div class="spacer"></div>

  <footer class="undo">
    <button type="button" class="btn ghost" disabled={!doc.canUndo} onclick={() => doc.undo()}>Undo</button>
    <button type="button" class="btn ghost" disabled={!doc.canRedo} onclick={() => doc.redo()}>Redo</button>
  </footer>
</aside>

<style>
  .inspector {
    background: var(--bg-surface);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: var(--s4);
    padding: var(--s4);
    overflow-y: auto;
  }
  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--s3);
    padding-bottom: var(--s4);
    border-bottom: 1px solid var(--border);
  }
  .panel:last-of-type { border-bottom: 0; }
  header h3 {
    margin: 0;
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .small { font-size: var(--fs-12); }
  .row { display: flex; gap: var(--s2); align-items: end; }
  .row .grow { flex: 1; }
  .field { display: flex; flex-direction: column; gap: var(--s1); }

  .btn {
    border-radius: var(--r-sm);
    padding: var(--s2) var(--s4);
    font-size: var(--fs-13);
    font-weight: 500;
    transition: background var(--dur) var(--ease);
  }
  .btn.primary { background: var(--accent); color: var(--accent-fg); }
  .btn.primary:hover:not(:disabled) { background: var(--accent-press); }
  .btn.ghost { background: transparent; color: var(--text-muted); border: 1px solid var(--border); }
  .btn.ghost:hover:not(:disabled) { color: var(--text); border-color: var(--border-strong); }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }

  input[type="text"], select {
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    padding: var(--s2) var(--s3);
  }
  input[type="text"]:focus, select:focus { border-color: var(--accent); outline: 0; }
  input[type="range"] { accent-color: var(--accent); }

  .swatches { display: flex; gap: var(--s2); }
  .swatch {
    width: 32px; height: 32px;
    border-radius: var(--r-sm);
    border: 1px solid var(--border);
    display: grid; place-items: center;
    color: var(--text-muted);
  }
  .swatch.active { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent) inset; }
  .swatch.checker {
    background-color: var(--checker-a);
    background-image:
      linear-gradient(45deg, var(--checker-b) 25%, transparent 25%),
      linear-gradient(-45deg, var(--checker-b) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, var(--checker-b) 75%),
      linear-gradient(-45deg, transparent 75%, var(--checker-b) 75%);
    background-size: 12px 12px;
    background-position: 0 0, 0 6px, 6px -6px, -6px 0;
  }

  .spacer { flex: 1; }
  .undo { display: flex; gap: var(--s2); }
  .undo .btn { flex: 1; }
</style>

