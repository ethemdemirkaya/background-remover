<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { ui } from "../stores/ui.svelte";
  import { doc } from "../stores/document.svelte";
  import type { Mode } from "../stores/document.svelte";
  import { ipc } from "../ipc";

  const modes: { id: Mode; label: string; key: string; icon: string }[] = [
    { id: "auto", label: "Auto", key: "A", icon: "M3 12h18M3 6h18M3 18h18" },
    { id: "smart", label: "Smart Select", key: "S", icon: "M4 4l6 6-3 3 7 7 3-3 6 6" },
    { id: "manual", label: "Manual", key: "M", icon: "M4 20l4-4 8-8 4-4-4 4-8 8z" },
  ];

  async function openNew() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff"] }],
    });
    if (typeof selected !== "string") return;
    ui.setError(null);
    ui.setBusy("Loading image...");
    try {
      const meta = await ipc.loadImage(selected);
      const url = convertFileSrc(selected);
      const fileName = selected.replace(/\\/g, "/").split("/").pop() ?? null;
      doc.setImage(meta.image_id, meta.width, meta.height, url, fileName);
      ui.zoom = 1; ui.panX = 0; ui.panY = 0;
    } catch (e) {
      ui.setError(String(e));
    } finally {
      ui.clearBusy();
    }
  }
</script>

<aside class="rail" aria-label="Modes">
  <button type="button" class="tool replace" title="Open another image" aria-label="Open image" onclick={openNew}>
    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <rect x="3" y="5" width="18" height="14" rx="2"/>
      <circle cx="9" cy="11" r="2"/>
      <path d="M5 17l5-5 4 4 3-3 3 3"/>
    </svg>
  </button>

  <div class="divider" aria-hidden="true"></div>

  <div class="group">
    {#each modes as m}
      <button
        type="button"
        class="tool"
        class:active={ui.mode === m.id}
        disabled={!doc.hasImage}
        title="{m.label} ({m.key})"
        aria-label={m.label}
        onclick={() => ui.setMode(m.id)}
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d={m.icon} />
        </svg>
      </button>
    {/each}
  </div>

  <div class="group bottom">
    <button type="button" class="tool" title="Zoom in" aria-label="Zoom in" onclick={() => ui.zoom = Math.min(ui.zoom * 1.2, 8)} disabled={!doc.hasImage}>
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="M11 8v6M8 11h6M20 20l-3.5-3.5"/></svg>
    </button>
    <button type="button" class="tool" title="Zoom out" aria-label="Zoom out" onclick={() => ui.zoom = Math.max(ui.zoom / 1.2, 0.1)} disabled={!doc.hasImage}>
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="M8 11h6M20 20l-3.5-3.5"/></svg>
    </button>
    <button type="button" class="tool" title="Fit (F)" aria-label="Fit to view" onclick={() => { ui.zoom = 1; ui.panX = 0; ui.panY = 0; }} disabled={!doc.hasImage}>
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"/></svg>
    </button>
  </div>
</aside>

<style>
  .rail {
    background: var(--bg-surface);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--s3) 0;
    gap: var(--s3);
  }
  .group { display: flex; flex-direction: column; gap: var(--s2); }
  .bottom { margin-top: auto; }
  .divider {
    width: 24px;
    height: 1px;
    background: var(--border);
    margin: var(--s1) 0;
  }
  .tool {
    width: 40px; height: 40px;
    border-radius: var(--r-sm);
    color: var(--text-muted);
    display: grid; place-items: center;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .tool:hover:not(:disabled) { background: var(--bg-hover); color: var(--text); }
  .tool:disabled { opacity: 0.35; cursor: not-allowed; }
  .tool.active {
    background: var(--accent);
    color: var(--accent-fg);
  }
  .tool.replace { color: var(--text); }
  .tool.replace:hover { background: var(--bg-hover); }
</style>
