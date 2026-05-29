<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { doc } from "../stores/document.svelte";
  import { ui } from "../stores/ui.svelte";
  import { smart } from "../stores/smart.svelte";
  import { ipc } from "../ipc";
  import { screenToImage } from "../canvas/render";
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

  // Cross-fade fraction during the "mask just landed" beat (CLAUDE.md §8.6).
  // 0 = show source only, 1 = show cutout only. We sit at 1 between transitions.
  let cutoutFade = 1;

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

      // Preview the chosen background under the cutout so the canvas reflects
      // what export will actually produce. Transparent bg → checkerboard stays,
      // solid color → fill rect, blur/image → keep checkerboard preview (those
      // need source pixels and would either taint the canvas or require a
      // round-trip to Rust; we save those for the export pipeline itself).
      if (ui.background.kind === "color") {
        ctx.fillStyle = ui.background.hex;
        ctx.fillRect(placement.x, placement.y, placement.drawW, placement.drawH);
      }

      if (cutoutFade < 1) {
        // The delightful moment — source underneath, cutout fading in on top.
        drawImage(ctx, img, placement);
        ctx.save();
        ctx.globalAlpha = cutoutFade;
        drawImage(ctx, cutoutImg, placement);
        ctx.restore();
      } else {
        drawImage(ctx, cutoutImg, placement);
      }
    } else if (maskImg && ui.mode === "smart") {
      // Smart Select is iterative — show source + tinted mask proposal.
      drawImage(ctx, img, placement);
      drawMaskOverlay(ctx, maskImg, placement);
    } else if (ui.mode === "smart") {
      drawImage(ctx, img, placement);
    } else if (maskImg) {
      // Fallback: mask exists but no Rust cutout (older results, edge cases).
      drawCutout(ctx, img, maskImg, placement);
    } else {
      drawImage(ctx, img, placement);
    }

    // Smart Select prompt markers float above whatever's drawn below.
    if (ui.mode === "smart" && smart.count > 0) {
      drawPromptMarkers(ctx, placement);
    }
  }

  function drawPromptMarkers(ctx: CanvasRenderingContext2D, placement: { x: number; y: number; drawW: number; drawH: number }) {
    if (!img) return;
    const accent = getComputedStyle(document.documentElement).getPropertyValue("--accent").trim() || "#E8533F";
    const danger = getComputedStyle(document.documentElement).getPropertyValue("--danger").trim() || "#D8493B";
    for (const p of smart.prompts) {
      if (p.kind !== "point") continue;
      const sx = placement.x + (p.x / img.naturalWidth) * placement.drawW;
      const sy = placement.y + (p.y / img.naturalHeight) * placement.drawH;
      ctx.save();
      ctx.lineWidth = 2;
      ctx.strokeStyle = "#FFFFFF";
      ctx.fillStyle = p.label === "remove" ? danger : accent;
      ctx.beginPath();
      ctx.arc(sx, sy, 6, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      ctx.restore();
    }
  }

  // Re-paint on any state change the canvas cares about.
  $effect(() => {
    void doc.sourceUrl;
    void doc.mask;
    void doc.cutout;
    void ui.zoom; void ui.panX; void ui.panY;
    void ui.background;
    void ui.mode;
    void smart.prompts;
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

  // Decode RGBA cutout PNG bytes when they change. Once the bitmap is ready,
  // kick off the 220ms fade-in that lifts the subject off the background.
  $effect(() => {
    const c = doc.cutout;
    if (!c) { cutoutImg = null; cutoutFade = 1; paint(); return; }
    const buf = new Uint8Array(c).buffer;
    const blob = new Blob([buf], { type: "image/png" });
    const url = URL.createObjectURL(blob);
    const el = new Image();
    el.onload = () => {
      cutoutImg = el;
      URL.revokeObjectURL(url);
      animateCutoutIn();
    };
    el.onerror = () => { URL.revokeObjectURL(url); };
    el.src = url;
  });

  function animateCutoutIn() {
    if (typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches) {
      cutoutFade = 1;
      paint();
      return;
    }
    const duration = 220;
    const start = performance.now();
    cutoutFade = 0;
    const tick = (now: number) => {
      const p = Math.min((now - start) / duration, 1);
      cutoutFade = easeOutCubic(p);
      paint();
      if (p < 1) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  function easeOutCubic(t: number) { const u = 1 - t; return 1 - u * u * u; }

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
          const paths = event.payload.paths ?? [];
          if (paths.length === 0) return;
          handleDrop(paths);
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
    ui.setBusy("Loading image...");
    try {
      const meta = await ipc.loadImage(path);
      const fileUrl = convertFileSrc(path);
      const fileName = basename(path);
      doc.setImage(meta.image_id, meta.width, meta.height, fileUrl, fileName);
      ui.zoom = 1; ui.panX = 0; ui.panY = 0;
    } catch (e) {
      ui.setError(String(e));
    } finally {
      ui.clearBusy();
    }
  }

  function basename(p: string): string {
    const m = p.replace(/\\/g, "/").match(/([^/]+)$/);
    return m ? m[1] : p;
  }

  // A path is "probably a folder" if it has no recognized image extension.
  // We can't statSync from the webview, so we go by extension; the worst
  // outcome is that an oddly-named image triggers the batch notice.
  const IMG_EXT = /\.(png|jpe?g|webp|bmp|tiff?|gif)$/i;

  function handleDrop(paths: string[]) {
    const images = paths.filter((p) => IMG_EXT.test(p));
    const folders = paths.filter((p) => !IMG_EXT.test(p));

    if (images.length === 0 && folders.length > 0) {
      ui.flash(`Folder detected — batch mode is coming in v0.2. Drop a single image to use Auto for now.`);
      return;
    }

    if (images.length > 1) {
      ui.flash(`${images.length} images detected — batch mode is coming in v0.2. Loading the first one.`);
    }
    loadFromPath(images[0]);
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
    // Middle button always pans.
    if (e.button === 1) { startPan(e); return; }
    // Shift + left = pan everywhere.
    if (e.shiftKey && e.button === 0) { startPan(e); return; }
    // In Smart Select mode, a plain left-click drops a point prompt.
    if (ui.mode === "smart" && e.button === 0 && img) {
      dropPoint(e);
      return;
    }
  }

  function startPan(e: PointerEvent) {
    panning = true;
    canvas.setPointerCapture(e.pointerId);
    panStart = { x: e.clientX, y: e.clientY, panX: ui.panX, panY: ui.panY };
  }

  function dropPoint(e: PointerEvent) {
    if (!img) return;
    const rect = canvas.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;
    const placement = computePlacement(img, { zoom: ui.zoom, panX: ui.panX, panY: ui.panY }, rect.width, rect.height);
    // Clicks outside the image bounds are ignored — no prompt placed.
    if (sx < placement.x || sy < placement.y || sx > placement.x + placement.drawW || sy > placement.y + placement.drawH) return;
    const { x, y } = screenToImage(sx, sy, placement, { width: img.naturalWidth, height: img.naturalHeight });
    smart.add({ kind: "point", x, y, label: e.altKey ? "remove" : "add" });
    paint();
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
      <div class="icon" aria-hidden="true">
        <svg viewBox="0 0 48 48" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="6" y="10" width="36" height="28" rx="3"/>
          <circle cx="16" cy="20" r="3"/>
          <path d="M8 34l10-10 7 7 6-6 9 9"/>
        </svg>
      </div>
      <h2>Drop an image to begin</h2>
      <p class="muted">or <button type="button" class="link" onclick={pickFile}>browse files</button></p>
      <p class="formats mono faint">png · jpg · webp · bmp · tiff</p>
      <p class="privacy faint">
        <svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>
        Processed entirely on your device. No uploads.
      </p>
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
  .empty .icon {
    color: var(--text-faint);
    margin-bottom: var(--s2);
  }
  .empty h2 {
    font-weight: 500;
    font-size: var(--fs-20);
    letter-spacing: -0.01em;
    margin: 0;
  }
  .empty .formats {
    margin: var(--s1) 0 0 0;
    font-size: var(--fs-12);
    letter-spacing: 0.04em;
  }
  .empty .privacy {
    margin-top: var(--s5);
    font-size: var(--fs-12);
    display: inline-flex;
    align-items: center;
    gap: var(--s1);
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
