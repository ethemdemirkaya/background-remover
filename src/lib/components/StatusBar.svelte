<script lang="ts">
  import { doc } from "../stores/document.svelte";
  import { ui } from "../stores/ui.svelte";

  interface Props {
    onShowShortcuts?: () => void;
  }
  let { onShowShortcuts }: Props = $props();
</script>

<footer class="bar" role="status">
  <span class="lock mono" title="No telemetry, no uploads. The app cannot reach the network for image data.">
    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>
    on-device
  </span>

  <div class="middle">
    {#if ui.error}
      <span class="error">{ui.error}</span>
    {:else if ui.busy}
      <span class="busy">
        <span class="spinner" aria-hidden="true"></span>
        {ui.busyLabel}
        {#if ui.progress !== null}<span class="mono faint">{Math.round(ui.progress * 100)}%</span>{/if}
      </span>
    {:else if doc.hasImage}
      <span class="muted small mono">{doc.width} × {doc.height}</span>
    {/if}
  </div>

  <button
    type="button"
    class="shortcut-hint mono"
    title="Keyboard shortcuts (?)"
    onclick={() => onShowShortcuts?.()}
  >?</button>
  <span class="zoom mono small">{Math.round(ui.zoom * 100)}%</span>
</footer>

<style>
  .bar {
    height: 28px;
    background: var(--bg-surface);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    padding: 0 var(--s3);
    gap: var(--s3);
    font-size: var(--fs-12);
    color: var(--text-muted);
  }
  .lock { display: inline-flex; align-items: center; gap: var(--s1); color: var(--text-faint); }
  .middle { flex: 1; display: flex; justify-content: center; }
  .error { color: var(--danger); }
  .busy { display: inline-flex; align-items: center; gap: var(--s2); }
  .spinner {
    width: 10px; height: 10px;
    border-radius: 50%;
    border: 1.5px solid var(--border-strong);
    border-top-color: var(--accent);
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .zoom { color: var(--text-faint); }
  .shortcut-hint {
    width: 18px; height: 18px;
    border-radius: 4px;
    color: var(--text-faint);
    font-size: var(--fs-12);
    line-height: 1;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .shortcut-hint:hover { background: var(--bg-hover); color: var(--text); }
</style>
