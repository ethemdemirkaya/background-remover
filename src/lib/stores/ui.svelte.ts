import type { Background } from "../ipc";
import type { Mode } from "./document.svelte";

class UIStore {
  mode = $state<Mode>("auto");
  busy = $state(false);
  busyLabel = $state<string>("");
  progress = $state<number | null>(null);
  error = $state<string | null>(null);
  /** Non-fatal info text — surfaced briefly in the status bar. */
  notice = $state<string | null>(null);
  private noticeTimer: ReturnType<typeof setTimeout> | null = null;

  /** Currently selected export background. */
  background = $state<Background>({ kind: "transparent" });

  /** Canvas view transform. */
  zoom = $state(1);
  panX = $state(0);
  panY = $state(0);

  setMode(m: Mode) {
    this.mode = m;
  }

  setBusy(label: string, progress: number | null = null) {
    this.busy = true;
    this.busyLabel = label;
    this.progress = progress;
  }

  clearBusy() {
    this.busy = false;
    this.busyLabel = "";
    this.progress = null;
  }

  setError(msg: string | null) {
    this.error = msg;
  }

  /** Show an info message for a few seconds, then clear it. */
  flash(msg: string, ms = 4000) {
    this.notice = msg;
    if (this.noticeTimer) clearTimeout(this.noticeTimer);
    this.noticeTimer = setTimeout(() => { this.notice = null; }, ms);
  }
}

export const ui = new UIStore();
