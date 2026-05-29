<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { doc } from "../stores/document.svelte";
  import { ui } from "../stores/ui.svelte";
  import { ipc } from "../ipc";
  import {
    drawCheckerboard,
    drawImage,
    drawCutout,
    drawMaskOverlay,
    computePlacement,
  } from "../canvas/render";

  let canvas: HTMLCanvasElement;
  let wrap: HTMLElement;
  let img: HTMLImageElement | null = null;
  let maskImg: HTMLImageElement | null = null;
  let cutoutImg: HTMLImageElement | null = null;
  let dragOver = $state(false);

  function paint() {
    if (!canvas || !wrap) return;
    const dpr = window.devicePixelRatio || 1;
    const rect = wrap.getBoundingClientRect();
    const w = Math.max(1, Math.floor(rect.width));
    const h = Math.max(1, Math.floor(rect.height));
    if (canvas.width !== w * dpr) canvas.width = w * dpr;
    if (canvas.height !== h * dpr) canvas.height = h * dpr;
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    drawCheckerboard(ctx, w, h);

    if (!img) return;
    const placement = computePlacement(img, { zoom: ui.zoom, panX: ui.panX, panY: ui.panY }, w, h);

    if (cutoutImg && ui.mode !== "smart") {
      // Server-built cutout: foreground decontaminated by Rust, alpha already
      // in the PNG. Just draw it — no compositing here, no CORS risk.
      drawImage(ctx, cutoutImg, placement);
    } else if (maskImg && ui.mode === "smart") {
      // Smart Select is iterative — show source + tinted mask proposal.
      drawImage(ctx, img, placement);
      drawMaskOverlay(ctx, maskImg, placement);
    } else if (maskImg) {
      // Fallback: mask exists but no Rust cutout (older results, edge cases).
      drawCutout(ctx, img, maskImg, placement);
    } else {
      drawImage(ctx, img, placement);
    }
  }

  // Re-paint on any state change the canvas cares about.
  $effect(() => {
    void doc.sourceUrl;
    void doc.mask;
    void doc.cutout;
    void ui.zoom; void ui.panX; void ui.panY;
    paint();
  });

  // Decode source whenever its URL changes. The source is only used for the
  // "no mask yet" preview path now — cutout rendering goes through doc.cutout
  // which is decontaminated server-side, so we no longer need crossOrigin
  // tricks to read source pixels back into JS.
  $effect(() => {
    const url = doc.sourceUrl;
    if (!url) { img = null; paint(); return; }
    const el = new Image();
    el.decoding = "async";
    el.onload = () => { img = el; paint(); };
    el.onerror = () => {
      ui.setError("Couldn't load the image preview. The asset protocol may be misconfigured.");
      img = null;
      paint();
    };
    el.src = url;
  });

  // Decode mask PNG bytes when they change.
  $effect(() => {
    const m = doc.mask;
    if (!m) { maskImg = null; paint(); return; }
    const buf = new Uint8Array(m).buffer;
    const blob = new Blob([buf], { type: "image/png" });
    const url = URL.createObjectURL(blob);
    const el = new Image();
    el.onload = () => { maskImg = el; URL.revokeObjectURL(url); paint(); };
    el.onerror = () => { URL.revokeObjectURL(url); };
    el.src = url;
  });

  // Decode RGBA cutout PNG bytes when they change.
  $effect(() => {
    const c = doc.cutout;
    if (!c) { cutoutImg = null; paint(); return; }
    const buf = new Uint8Array(c).buffer;
    const blob = new Blob([buf], { type: "image/png" });
    const url = URL.createObjectURL(blob);
    const el = new Image();
    el.onload = () => { cutoutImg = el; URL.revokeObjectURL(url); paint(); };
    el.onerror = () => { URL.revokeObjectURL(url); };
    el.src = url;
  });

  onMount(() => {
    const ro = new ResizeObserver(() => paint());
    ro.observe(wrap);

    // Tauri webviews don't fill `dataTransfer.files[i].path`. We have to listen to
    // the webview's own drag-drop event, which gives us absolute paths.
    let unlisten: (() => void) | null = null;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") dragOver = true;
        else if (event.payload.type === "leave") dragOver = false;
        else if (event.payload.type === "drop") {
          dragOver = false;
          const first = event.payload.paths?.[0];
          if (first) loadFromPath(first);
        }
      })
      .then((fn) => { unlisten = fn; })
      .catch(() => { /* outside Tauri — ignore */ });

    paint();

    return () => {
      ro.disconnect();
      unlisten?.();
    };
  });

  async function loadFromPath(path: string) {
    ui.setError(null);
    ui.setBusy("Encoding image...");
    try {
      const meta = await ipc.loadImage(path);
      const fileUrl = convertFileSrc(path);
      doc.setImage(meta.image_id, meta.width, meta.height, fileUrl);
      ui.zoom = 1; ui.panX = 0; ui.panY = 0;
    } catch (e) {
      ui.setError(String(e));
    } finally {
      ui.clearBusy();
    }
  }

  async function pickFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff"] }],
    });
    if (typeof selected === "string") loadFromPath(selected);
  }

  function onWheel(e: WheelEvent) {
    if (!img) return;
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1;
    ui.zoom = Math.max(0.1, Math.min(8, ui.zoom * factor));
  }

  let panning = false;
  let panStart = { x: 0, y: 0, panX: 0, panY: 0 };
  function onPointerDown(e: PointerEvent) {
    // Space-drag pan: shift OR middle button. Plain click is reserved for Smart Select (Phase 2).
    if (!e.shiftKey && e.button !== 1) return;
    panning = true;
    canvas.setPointerCapture(e.pointerId);
    panStart = { x: e.clientX, y: e.clientY, panX: ui.panX, panY: ui.panY };
  }
  function onPointerMove(e: PointerEvent) {
    if (!panning) return;
    ui.panX = panStart.panX + (e.clientX - panStart.x);
    ui.panY = panStart.panY + (e.clientY - panStart.y);
  }
  function onPointerUp(e: PointerEvent) {
    if (!panning) return;
    panning = false;
    canvas.releasePointerCapture(e.pointerId);
  }
</script>

<section
  class="stage"
  bind:this={wrap}
  aria-label="Canvas"
>
  <canvas
    bind:this={canvas}
    onwheel={onWheel}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
  ></canvas>

  {#if !doc.hasImage}
    <div class="empty" class:over={dragOver}>
      <h2>Drop an image</h2>
      <p class="muted">or <button type="button" class="link" onclick={pickFile}>browse files</button></p>
      <p class="privacy faint">Images are processed entirely on your device.</p>
    </div>
  {/if}

  {#if dragOver && doc.hasImage}
    <div class="drop-hint" aria-hidden="true">Drop to replace</div>
  {/if}
</section>

<style>
  .stage {
    position: relative;
    background: var(--bg-base);
    overflow: hidden;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
  .empty {
    position: absolute;
    inset: 0;
    display: grid;
    place-content: center;
    justify-items: center;
    text-align: center;
    pointer-events: none;
    gap: var(--s3);
  }
  .empty > * { pointer-events: auto; }
  .empty h2 {
    font-weight: 500;
    font-size: var(--fs-20);
    letter-spacing: -0.01em;
    margin: 0;
  }
  .empty .privacy {
    margin-top: var(--s5);
    font-size: var(--fs-12);
  }
  .link {
    color: var(--accent);
    text-decoration: none;
    border-bottom: 1px solid transparent;
  }
  .link:hover { border-bottom-color: var(--accent); }
  .empty.over::before,
  .stage:has(.drop-hint)::before {
    content: "";
    position: absolute;
    inset: var(--s5);
    border: 1px dashed var(--accent);
    border-radius: var(--r-lg);
    pointer-events: none;
  }
  .drop-hint {
    position: absolute;
    top: var(--s5);
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-raised);
    border: 1px solid var(--border);
    color: var(--text);
    padding: var(--s2) var(--s4);
    border-radius: var(--r-sm);
    font-size: var(--fs-12);
    pointer-events: none;
  }
</style>
