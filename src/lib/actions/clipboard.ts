import { doc } from "../stores/document.svelte";
import { ui } from "../stores/ui.svelte";

/**
 * Put the current decontaminated cutout on the system clipboard as a PNG so
 * the user can paste it straight into another app. We use the cutout
 * (transparent background) rather than the chosen export background — that's
 * almost always what "copy" should mean for a background remover.
 *
 * Returns true when the write succeeded so the caller can show feedback.
 */
export async function copyCutoutToClipboard(): Promise<boolean> {
  if (!doc.cutout) return false;
  try {
    const buf = new Uint8Array(doc.cutout).buffer;
    const blob = new Blob([buf], { type: "image/png" });
    await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
    return true;
  } catch (e) {
    ui.setError("Couldn't copy: " + String(e));
    return false;
  }
}
