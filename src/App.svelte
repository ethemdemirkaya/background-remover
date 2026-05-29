<script lang="ts">
  import Toolbar from "./lib/components/Toolbar.svelte";
  import Stage from "./lib/components/Stage.svelte";
  import Inspector from "./lib/components/Inspector.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import { doc } from "./lib/stores/document.svelte";
  import { ui } from "./lib/stores/ui.svelte";

  function onKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key.toLowerCase() === "z" && !e.shiftKey) { e.preventDefault(); doc.undo(); }
    else if (mod && (e.key.toLowerCase() === "y" || (e.shiftKey && e.key.toLowerCase() === "z"))) { e.preventDefault(); doc.redo(); }
    else if (!mod && e.key.toLowerCase() === "a") ui.setMode("auto");
    else if (!mod && e.key.toLowerCase() === "s") ui.setMode("smart");
    else if (!mod && e.key.toLowerCase() === "m") ui.setMode("manual");
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <Toolbar />
  <Stage />
  <Inspector />
</div>
<StatusBar />

<style>
  .app {
    display: grid;
    grid-template-columns: 56px 1fr 320px;
    height: calc(100% - 28px);
    background: var(--bg-base);
  }
</style>

