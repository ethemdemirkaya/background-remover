/**
 * Canvas rendering helpers.
 *
 * The stage paints, in order:
 *   1. transparency checkerboard (signature motif — solid squares, never a gradient)
 *   2. either:
 *        - the source image (no mask yet), OR
 *        - the *cutout*: source image with the mask used as alpha, so the
 *          checkerboard shows through where the background was removed.
 *      In Smart Select mode the cutout is suppressed and we draw the source +
 *      a tinted mask overlay instead, since the user is iterating on a
 *      proposal.
 *
 * Mask transport: Rust returns a single-channel PNG. When the browser decodes
 * it to RGBA, all four channels equal the luminance (L) with alpha=255. We
 * have to transpose L → A before either compositing or tinting.
 */

export const CHECKER_SIZE = 12;

export function drawCheckerboard(ctx: CanvasRenderingContext2D, w: number, h: number) {
  const a = getCssVar("--checker-a") || "#1B1D20";
  const b = getCssVar("--checker-b") || "#232629";
  ctx.fillStyle = a;
  ctx.fillRect(0, 0, w, h);
  ctx.fillStyle = b;
  for (let y = 0; y < h; y += CHECKER_SIZE) {
    const offset = ((y / CHECKER_SIZE) | 0) % 2 === 0 ? 0 : CHECKER_SIZE;
    for (let x = offset; x < w; x += CHECKER_SIZE * 2) {
      ctx.fillRect(x, y, CHECKER_SIZE, CHECKER_SIZE);
    }
  }
}

export interface Placement {
  x: number;
  y: number;
  drawW: number;
  drawH: number;
  scale: number;
}

export function computePlacement(
  img: HTMLImageElement,
  view: { zoom: number; panX: number; panY: number },
  canvasW: number,
  canvasH: number,
): Placement {
  const fit = fitScale(img.naturalWidth, img.naturalHeight, canvasW, canvasH);
  const scale = fit * view.zoom;
  const drawW = img.naturalWidth * scale;
  const drawH = img.naturalHeight * scale;
  const x = (canvasW - drawW) / 2 + view.panX;
  const y = (canvasH - drawH) / 2 + view.panY;
  return { x, y, drawW, drawH, scale };
}

export function drawImage(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  placement: Placement,
) {
  ctx.save();
  ctx.imageSmoothingEnabled = placement.scale < 1;
  ctx.imageSmoothingQuality = "high";
  ctx.drawImage(img, placement.x, placement.y, placement.drawW, placement.drawH);
  ctx.restore();
}

/**
 * Render the *cutout*: source image with the mask used as alpha. Anywhere the
 * mask is dark, the source becomes transparent so the checkerboard underneath
 * shows through. This is what makes "Remove background" feel like it actually
 * removed the background.
 */
export function drawCutout(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  maskBitmap: HTMLImageElement | ImageBitmap,
  placement: Placement,
) {
  const w = img.naturalWidth;
  const h = img.naturalHeight;
  if (w === 0 || h === 0) return;

  const off = document.createElement("canvas");
  off.width = w;
  off.height = h;
  const oc = off.getContext("2d");
  if (!oc) return;

  // 1. Source image at full resolution.
  oc.drawImage(img, 0, 0, w, h);

  // 2. Build an alpha-only stencil from the mask's luminance channel.
  const stencil = document.createElement("canvas");
  stencil.width = w;
  stencil.height = h;
  const sc = stencil.getContext("2d");
  if (!sc) return;
  sc.drawImage(maskBitmap as CanvasImageSource, 0, 0, w, h);
  const data = sc.getImageData(0, 0, w, h);
  const px = data.data;
  for (let i = 0; i < px.length; i += 4) {
    const l = px[i]; // R == G == B == L for an L8 PNG decoded to RGBA
    px[i] = 255;
    px[i + 1] = 255;
    px[i + 2] = 255;
    px[i + 3] = l;
  }
  sc.putImageData(data, 0, 0);

  // 3. destination-in keeps source pixels only where stencil alpha is set.
  oc.globalCompositeOperation = "destination-in";
  oc.drawImage(stencil, 0, 0);

  // 4. Draw the cutout at placement on the main canvas.
  ctx.save();
  ctx.imageSmoothingEnabled = placement.scale < 1;
  ctx.imageSmoothingQuality = "high";
  ctx.drawImage(off, placement.x, placement.y, placement.drawW, placement.drawH);
  ctx.restore();
}

/**
 * Tinted selection overlay for Smart Select. Uses the accent fill where the
 * mask is active; transparent elsewhere. Kept for Phase 2 — Auto mode uses
 * drawCutout so the user sees the actual result.
 */
export function drawMaskOverlay(
  ctx: CanvasRenderingContext2D,
  maskBitmap: HTMLImageElement | ImageBitmap,
  placement: Placement,
) {
  const srcW = "naturalWidth" in maskBitmap ? maskBitmap.naturalWidth : maskBitmap.width;
  const srcH = "naturalHeight" in maskBitmap ? maskBitmap.naturalHeight : maskBitmap.height;
  if (srcW === 0 || srcH === 0) return;

  const off = document.createElement("canvas");
  off.width = srcW;
  off.height = srcH;
  const oc = off.getContext("2d");
  if (!oc) return;
  oc.drawImage(maskBitmap as CanvasImageSource, 0, 0);

  const accent = parseAccent();
  const data = oc.getImageData(0, 0, srcW, srcH);
  const px = data.data;
  for (let i = 0; i < px.length; i += 4) {
    const a = px[i];
    px[i] = accent.r;
    px[i + 1] = accent.g;
    px[i + 2] = accent.b;
    px[i + 3] = Math.round(a * accent.a);
  }
  oc.putImageData(data, 0, 0);

  ctx.save();
  ctx.imageSmoothingEnabled = placement.scale < 1;
  ctx.imageSmoothingQuality = "high";
  ctx.drawImage(off, placement.x, placement.y, placement.drawW, placement.drawH);
  ctx.restore();
}

export function fitScale(imgW: number, imgH: number, canvasW: number, canvasH: number): number {
  const pad = 32;
  return Math.min((canvasW - pad * 2) / imgW, (canvasH - pad * 2) / imgH, 1);
}

export function screenToImage(
  sx: number,
  sy: number,
  placement: Placement,
  img: { width: number; height: number },
) {
  return {
    x: ((sx - placement.x) / placement.drawW) * img.width,
    y: ((sy - placement.y) / placement.drawH) * img.height,
  };
}

function getCssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

interface Rgba { r: number; g: number; b: number; a: number; }
function parseAccent(): Rgba {
  const fill = getCssVar("--select-fill");
  const m = fill.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)/i);
  if (m) {
    return { r: +m[1], g: +m[2], b: +m[3], a: m[4] !== undefined ? +m[4] : 1 };
  }
  const hex = (getCssVar("--accent") || "#e8533f").replace("#", "");
  const r = parseInt(hex.slice(0, 2), 16);
  const g = parseInt(hex.slice(2, 4), 16);
  const b = parseInt(hex.slice(4, 6), 16);
  return { r, g, b, a: 0.55 };
}
