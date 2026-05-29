import type { Background } from "../ipc";
import type { Mode } from "./document.svelte";

class UIStore {
  mode = $state<Mode>("auto");
  busy = $state(false);
  busyLabel = $state<string>("");
  progress = $state<number | null>(null);
  error = $state<string | null>(null);

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
}

export const ui = new UIStore();
