<script lang="ts">
  import { doc } from "../stores/document.svelte";
  import { ui } from "../stores/ui.svelte";

  interface Props {
    onShowShortcuts?: () => void;
    onShowAbout?: () => void;
  }
  let { onShowShortcuts, onShowAbout }: Props = $props();

  // Errors sometimes wrap a multi-line backtrace; the bar only has one line.
  // Take the first useful sentence, hover for the rest.
  function shortenError(s: string): string {
    const first = s.split(/\r?\n/)[0]?.trim() ?? s;
    return first.length > 140 ? first.slice(0, 137) + "..." : first;
  }
</script>

{#if ui.busy}
  <div class="progress" aria-hidden="true">
    <div class="progress-fill"></div>
  </div>
{/if}

<footer class="bar" role="status">
  <button
    type="button"
    class="lock mono"
    title="About — no telemetry, no uploads. Click for details."
    onclick={() => onShowAbout?.()}
  >
    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>
    on-device
  </button>

  <div class="middle">
    {#if ui.error}
      <button type="button" class="error" title={ui.error} onclick={() => ui.setError(null)}>
        <span class="error-text">{shortenError(ui.error)}</span>
        <span class="error-dismiss" aria-hidden="true">×</span>
      </button>
    {:else if ui.busy}
      <span class="busy">
        <span class="spinner" aria-hidden="true"></span>
        {ui.busyLabel}
        {#if ui.progress !== null}<span class="mono faint">{Math.round(ui.progress * 100)}%</span>{/if}
      </span>
    {:else if ui.notice}
      <span class="notice">{ui.notice}</span>
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
  .progress {
    position: fixed;
    bottom: 28px; /* sit on top of the status bar */
    left: 0; right: 0;
    height: 2px;
    background: transparent;
    overflow: hidden;
    z-index: 30;
  }
  .progress-fill {
    width: 35%;
    height: 100%;
    background: var(--accent);
    animation: slide 1.4s var(--ease) infinite;
  }
  @keyframes slide {
    0%   { transform: translateX(-100%); }
    100% { transform: translateX(285%); }
  }
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
  .lock {
    display: inline-flex; align-items: center; gap: var(--s1);
    color: var(--text-faint);
    padding: 0 var(--s2);
    height: 22px;
    border-radius: var(--r-sm);
    font-size: var(--fs-12);
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .lock:hover { background: var(--bg-hover); color: var(--text); }
  .middle { flex: 1; display: flex; justify-content: center; }
  .error {
    color: var(--danger);
    display: inline-flex;
    align-items: center;
    gap: var(--s2);
    padding: 0 var(--s2);
    border-radius: var(--r-sm);
    max-width: 60ch;
    border: 1px solid transparent;
    transition: background var(--dur) var(--ease), border-color var(--dur) var(--ease);
  }
  .error:hover { background: rgba(216, 73, 59, 0.08); border-color: var(--danger); }
  .error-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .error-dismiss { font-size: 14px; opacity: 0.7; }
  .notice { color: var(--text); }
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
