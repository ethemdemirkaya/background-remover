import { invoke } from "@tauri-apps/api/core";

export type ImageId = string;

export interface ImageMeta {
  image_id: ImageId;
  width: number;
  height: number;
}

export type Prompt =
  | { kind: "point"; x: number; y: number; label: "add" | "remove" }
  | { kind: "box"; x0: number; y0: number; x1: number; y1: number; label: "add" | "remove" };

export type Background =
  | { kind: "transparent" }
  | { kind: "color"; hex: string }
  | { kind: "image"; path: string }
  | { kind: "blur"; radius: number };

export type ExportFormat = "png" | "webp";

/**
 * Mask is a PNG-encoded single-channel image returned as raw bytes.
 * The frontend draws it as an overlay; we never marshal giant JSON arrays. (CLAUDE.md §3.3)
 */
export type Mask = Uint8Array;

export const ipc = {
  loadImage(path: string): Promise<ImageMeta> {
    return invoke<ImageMeta>("load_image", { path });
  },

  autoRemove(imageId: ImageId): Promise<Mask> {
    return invoke<number[]>("auto_remove", { imageId }).then((b) => new Uint8Array(b));
  },

  smartSelect(imageId: ImageId, prompts: Prompt[]): Promise<Mask> {
    return invoke<number[]>("smart_select", { imageId, prompts }).then((b) => new Uint8Array(b));
  },

  /** Returns encoded bytes when savePath is null; otherwise writes to disk and returns null. */
  exportImage(args: {
    imageId: ImageId;
    mask: Mask;
    background: Background;
    format: ExportFormat;
    savePath: string | null;
  }): Promise<Uint8Array | null> {
    return invoke<number[] | null>("export_image", {
      imageId: args.imageId,
      mask: Array.from(args.mask),
      background: args.background,
      format: args.format,
      savePath: args.savePath,
    }).then((b) => (b ? new Uint8Array(b) : null));
  },

  clearImage(imageId: ImageId): Promise<void> {
    return invoke("clear_image", { imageId });
  },
};
