import type { Background } from "../ipc";
import type { ExportFormat } from "../ipc";

const KEY = "bg-remover/prefs/v1";
const RECENTS_KEY = "bg-remover/recents/v1";
const RECENTS_MAX = 8;

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

export function loadRecents(): string[] {
  try {
    const raw = localStorage.getItem(RECENTS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((x): x is string => typeof x === "string").slice(0, RECENTS_MAX);
  } catch {
    return [];
  }
}

export function pushRecent(path: string) {
  const existing = loadRecents().filter((p) => p !== path);
  const next = [path, ...existing].slice(0, RECENTS_MAX);
  try { localStorage.setItem(RECENTS_KEY, JSON.stringify(next)); } catch { /* ignore */ }
}
