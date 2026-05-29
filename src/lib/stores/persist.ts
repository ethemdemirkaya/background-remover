import type { Background } from "../ipc";
import type { ExportFormat } from "../ipc";

const KEY = "bg-remover/prefs/v1";

interface Persisted {
  background?: Background;
  exportFormat?: ExportFormat;
  colorHex?: string;
  blurRadius?: number;
}

/**
 * Load persisted UI preferences. Returns an empty object if storage is empty
 * or the schema doesn't match — we never throw on parse errors because that
 * would block the whole app from starting.
 */
export function loadPrefs(): Persisted {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null) return {};
    return parsed as Persisted;
  } catch {
    return {};
  }
}

export function savePrefs(p: Persisted) {
  try {
    localStorage.setItem(KEY, JSON.stringify(p));
  } catch {
    // Storage full or disabled — silently ignore, prefs aren't critical.
  }
}
