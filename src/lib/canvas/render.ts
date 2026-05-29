/**
 * Canvas rendering helpers.
 *
 * The stage paints, in order:
 *   1. transparency checkerboard (signature motif — solid squares, never a gradient)
 *   2. the source image (fit-to-stage * user zoom, centered + panned)
 *   3. the mask as a tinted overlay
 *
 * The mask comes from Rust as a single-channel PNG. When the browser decodes it
 * to RGBA, all four channels equal the luminance (L) with alpha=255. We can't
 * just `source-in`-tint it — that would paint the *whole* canvas. So we
 * transpose L → A and write the accent RGB everywhere, producing a clean tinted
 * overlay only where the mask is non-zero.
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

export function drawImageFitted(
  ctx: CanvasRenderingContext2D,
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
  ctx.imageSmoothingEnabled = scale < 1;
  ctx.imageSmoothingQuality = "high";
  ctx.drawImage(img, x, y, drawW, drawH);
  return { x, y, drawW, drawH, scale };
}

export function drawMaskOverlay(
  ctx: CanvasRenderingContext2D,
  maskBitmap: HTMLImageElement | ImageBitmap,
  placement: Placement,
) {
  // 1. Decode mask into an offscreen at the *source-image* resolution so we don't
  //    smear the L→A transpose across upscaled pixels.
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
    // L8 PNGs decode as R=G=B=L, A=255. Use L as alpha, accent as RGB.
    const a = px[i];
    px[i] = accent.r;
    px[i + 1] = accent.g;
    px[i + 2] = accent.b;
    px[i + 3] = Math.round(a * accent.a);
  }
  oc.putImageData(data, 0, 0);

  // 2. Draw the tinted mask at the same scale/position as the image.
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
  // Use --select-fill if it's rgba; otherwise fall back to --accent at 0.55 opacity.
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
