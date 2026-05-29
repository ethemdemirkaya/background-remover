import type { CutoutPng, ImageId, Mask } from "../ipc";

export type Mode = "auto" | "smart" | "manual";

interface HistoryEntry {
  mask: Mask;
  cutout: CutoutPng | null;
  label: string;
}

class DocumentStore {
  imageId = $state<ImageId | null>(null);
  width = $state(0);
  height = $state(0);

  /** URL for the loaded source image (Tauri asset:// URL or blob:). */
  sourceUrl = $state<string | null>(null);

  /** Current L8 mask PNG bytes. Carried through to the export pipeline. */
  mask = $state<Mask | null>(null);

  /**
   * Current RGBA cutout PNG bytes — already decontaminated server-side, so this
   * is what we draw on the canvas. Only present for Auto/Manual results; Smart
   * Select keeps `cutout` null and renders the source + mask overlay instead.
   */
  cutout = $state<CutoutPng | null>(null);

  history = $state<HistoryEntry[]>([]);
  redoStack = $state<HistoryEntry[]>([]);

  get canUndo() { return this.history.length > 0; }
  get canRedo() { return this.redoStack.length > 0; }
  get hasImage() { return this.imageId !== null; }

  setImage(id: ImageId, width: number, height: number, sourceUrl: string) {
    this.imageId = id;
    this.width = width;
    this.height = height;
    this.sourceUrl = sourceUrl;
    this.mask = null;
    this.cutout = null;
    this.history = [];
    this.redoStack = [];
  }

  pushAutoResult(mask: Mask, cutout: CutoutPng, label: string) {
    this.snapshotCurrent(label);
    this.mask = mask;
    this.cutout = cutout;
    this.redoStack = [];
  }

  /** Smart Select pushes only a mask; no decontaminated cutout yet. */
  pushMask(mask: Mask, label: string) {
    this.snapshotCurrent(label);
    this.mask = mask;
    this.cutout = null;
    this.redoStack = [];
  }

  private snapshotCurrent(label: string) {
    if (this.mask) {
      this.history.push({ mask: this.mask, cutout: this.cutout, label });
    }
  }

  undo() {
    if (!this.canUndo) return;
    const prev = this.history.pop()!;
    if (this.mask) this.redoStack.push({ mask: this.mask, cutout: this.cutout, label: prev.label });
    this.mask = prev.mask;
    this.cutout = prev.cutout;
  }

  redo() {
    if (!this.canRedo) return;
    const next = this.redoStack.pop()!;
    if (this.mask) this.history.push({ mask: this.mask, cutout: this.cutout, label: next.label });
    this.mask = next.mask;
    this.cutout = next.cutout;
  }

  reset() {
    this.imageId = null;
    this.width = 0;
    this.height = 0;
    this.sourceUrl = null;
    this.mask = null;
    this.cutout = null;
    this.history = [];
    this.redoStack = [];
  }
}

export const doc = new DocumentStore();
