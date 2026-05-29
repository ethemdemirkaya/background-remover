import type { ImageId, Mask } from "../ipc";

export type Mode = "auto" | "smart" | "manual";

interface HistoryEntry {
  mask: Mask;
  label: string;
}

class DocumentStore {
  imageId = $state<ImageId | null>(null);
  width = $state(0);
  height = $state(0);

  /** Object URL for the loaded source image, used by <canvas> render. */
  sourceUrl = $state<string | null>(null);

  /** Current mask (PNG bytes). The canvas decodes & draws it as overlay. */
  mask = $state<Mask | null>(null);

  history = $state<HistoryEntry[]>([]);
  redoStack = $state<HistoryEntry[]>([]);

  get canUndo() { return this.history.length > 1; }
  get canRedo() { return this.redoStack.length > 0; }
  get hasImage() { return this.imageId !== null; }

  setImage(id: ImageId, width: number, height: number, sourceUrl: string) {
    this.imageId = id;
    this.width = width;
    this.height = height;
    this.sourceUrl = sourceUrl;
    this.mask = null;
    this.history = [];
    this.redoStack = [];
  }

  pushMask(mask: Mask, label: string) {
    if (this.mask) this.history.push({ mask: this.mask, label });
    this.mask = mask;
    this.redoStack = [];
  }

  undo() {
    if (!this.canUndo) return;
    const prev = this.history.pop()!;
    if (this.mask) this.redoStack.push({ mask: this.mask, label: prev.label });
    this.mask = prev.mask;
  }

  redo() {
    if (!this.canRedo) return;
    const next = this.redoStack.pop()!;
    if (this.mask) this.history.push({ mask: this.mask, label: next.label });
    this.mask = next.mask;
  }

  reset() {
    if (this.sourceUrl) URL.revokeObjectURL(this.sourceUrl);
    this.imageId = null;
    this.width = 0;
    this.height = 0;
    this.sourceUrl = null;
    this.mask = null;
    this.history = [];
    this.redoStack = [];
  }
}

export const doc = new DocumentStore();
