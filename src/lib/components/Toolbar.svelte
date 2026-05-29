<script lang="ts">
  import { ui } from "../stores/ui.svelte";
  import { doc } from "../stores/document.svelte";
  import type { Mode } from "../stores/document.svelte";

  const modes: { id: Mode; label: string; key: string; icon: string }[] = [
    { id: "auto", label: "Auto", key: "A", icon: "M3 12h18M3 6h18M3 18h18" },
    { id: "smart", label: "Smart Select", key: "S", icon: "M4 4l6 6-3 3 7 7 3-3 6 6" },
    { id: "manual", label: "Manual", key: "M", icon: "M4 20l4-4 8-8 4-4-4 4-8 8z" },
  ];
</script>

<aside class="rail" aria-label="Modes">
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
    <button type="button" class="tool" title="Fit" aria-label="Fit to view" onclick={() => { ui.zoom = 1; ui.panX = 0; ui.panY = 0; }} disabled={!doc.hasImage}>
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
    gap: var(--s5);
  }
  .group { display: flex; flex-direction: column; gap: var(--s2); }
  .bottom { margin-top: auto; }
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
</style>

