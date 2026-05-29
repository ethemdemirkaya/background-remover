<script lang="ts">
  import { onMount } from "svelte";
  import { ipc, type SetupStatus } from "../ipc";

  let status = $state<SetupStatus | null>(null);
  let checking = $state(false);

  async function recheck() {
    checking = true;
    try {
      status = await ipc.checkSetup();
    } catch {
      status = null;
    } finally {
      checking = false;
    }
  }

  onMount(recheck);
</script>

{#if status && !status.matte_ready}
  <div class="banner" role="alert">
    <div class="content">
      <strong>Setup needed.</strong>
      <span class="muted">The matte model isn't bundled yet — run the fetch script once to download it.</span>
      <code class="mono small">pwsh scripts/fetch-models.ps1</code>
      <span class="path mono faint small">expects: {status.matte_path}</span>
    </div>
    <button type="button" class="btn ghost" disabled={checking} onclick={recheck}>
      {checking ? "Checking..." : "Try again"}
    </button>
  </div>
{/if}

<style>
  .banner {
    background: var(--bg-raised);
    border-bottom: 1px solid var(--border);
    padding: var(--s3) var(--s4);
    display: flex;
    align-items: center;
    gap: var(--s4);
  }
  .content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  strong { font-weight: 500; }
  code {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    padding: 2px var(--s2);
    font-size: var(--fs-12);
    width: fit-content;
    margin-top: 2px;
  }
  .path { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .small { font-size: var(--fs-12); }
  .btn {
    border-radius: var(--r-sm);
    padding: var(--s2) var(--s4);
    font-size: var(--fs-13);
    font-weight: 500;
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--border);
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .btn:hover:not(:disabled) { color: var(--text); border-color: var(--border-strong); }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
