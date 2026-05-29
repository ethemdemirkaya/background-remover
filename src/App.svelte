<script lang="ts">
  import Toolbar from "./lib/components/Toolbar.svelte";
  import Stage from "./lib/components/Stage.svelte";
  import Inspector from "./lib/components/Inspector.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import ShortcutsOverlay from "./lib/components/ShortcutsOverlay.svelte";
  import { doc } from "./lib/stores/document.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let shortcutsOpen = $state(false);

  // Keep the OS window title in sync with the active file. Useful when several
  // copies of the app are open or when alt-tabbing — the title bar tells you
  // which image you'll act on.
  $effect(() => {
    const name = doc.fileName;
    const title = name ? `${name} — Background Remover` : "Background Remover";
    getCurrentWindow().setTitle(title).catch(() => { /* fine outside Tauri */ });
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const mod = e.ctrlKey || e.metaKey;

    if (e.key === "Escape" && shortcutsOpen) { shortcutsOpen = false; return; }
    if (!mod && e.key === "?") { e.preventDefault(); shortcutsOpen = !shortcutsOpen; return; }

    if (mod && e.key.toLowerCase() === "z" && !e.shiftKey) { e.preventDefault(); doc.undo(); }
    else if (mod && (e.key.toLowerCase() === "y" || (e.shiftKey && e.key.toLowerCase() === "z"))) { e.preventDefault(); doc.redo(); }
    else if (!mod && e.key.toLowerCase() === "a") ui.setMode("auto");
    else if (!mod && e.key.toLowerCase() === "s") ui.setMode("smart");
    else if (!mod && e.key.toLowerCase() === "m") ui.setMode("manual");
    else if (!mod && e.key.toLowerCase() === "f") { ui.zoom = 1; ui.panX = 0; ui.panY = 0; }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <Toolbar />
  <Stage />
  <Inspector />
</div>
<StatusBar onShowShortcuts={() => (shortcutsOpen = true)} />

<ShortcutsOverlay open={shortcutsOpen} onClose={() => (shortcutsOpen = false)} />

<style>
  .app {
    display: grid;
    grid-template-columns: 56px 1fr 320px;
    height: calc(100% - 28px);
    background: var(--bg-base);
  }
</style>
