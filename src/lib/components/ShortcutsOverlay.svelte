<script lang="ts">
  interface Props {
    open: boolean;
    onClose: () => void;
  }
  let { open, onClose }: Props = $props();

  const groups: { title: string; shortcuts: [string, string][] }[] = [
    {
      title: "Modes",
      shortcuts: [
        ["A", "Auto"],
        ["S", "Smart Select"],
        ["M", "Manual"],
      ],
    },
    {
      title: "Canvas",
      shortcuts: [
        ["Mouse wheel", "Zoom"],
        ["Shift + drag", "Pan"],
        ["F", "Fit to view"],
      ],
    },
    {
      title: "Editing",
      shortcuts: [
        ["Ctrl + Z", "Undo"],
        ["Ctrl + Shift + Z", "Redo"],
        ["Ctrl + S", "Save / export"],
        ["Ctrl + C", "Copy cutout to clipboard"],
      ],
    },
    {
      title: "App",
      shortcuts: [
        ["?", "Show this sheet"],
        ["Esc", "Dismiss"],
      ],
    },
  ];
</script>

{#if open}
  <div class="scrim" onclick={onClose} role="presentation"></div>
  <div class="sheet" role="dialog" aria-label="Keyboard shortcuts" aria-modal="true">
    <header>
      <h2>Shortcuts</h2>
      <button type="button" class="close" aria-label="Close" onclick={onClose}>×</button>
    </header>
    <div class="grid">
      {#each groups as g}
        <section>
          <h3>{g.title}</h3>
          <dl>
            {#each g.shortcuts as [key, desc]}
              <div class="row">
                <dt class="mono">{key}</dt>
                <dd class="muted">{desc}</dd>
              </div>
            {/each}
          </dl>
        </section>
      {/each}
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    z-index: 50;
    animation: fade var(--dur) var(--ease);
  }
  .sheet {
    position: fixed;
    z-index: 51;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(560px, calc(100vw - var(--s5) * 2));
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow);
    animation: pop var(--dur) var(--ease);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s4) var(--s5);
    border-bottom: 1px solid var(--border);
  }
  header h2 {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 500;
  }
  .close {
    width: 28px;
    height: 28px;
    color: var(--text-muted);
    border-radius: var(--r-sm);
    font-size: 20px;
    line-height: 1;
  }
  .close:hover { background: var(--bg-hover); color: var(--text); }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--s5);
    padding: var(--s5);
  }
  section h3 {
    margin: 0 0 var(--s2) 0;
    font-size: var(--fs-12);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }
  dl { margin: 0; display: flex; flex-direction: column; gap: var(--s1); }
  .row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s3);
  }
  dt {
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    padding: 2px 8px;
    font-size: var(--fs-12);
    color: var(--text);
    white-space: nowrap;
  }
  dd { margin: 0; font-size: var(--fs-13); }
  @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
  @keyframes pop {
    from { opacity: 0; transform: translate(-50%, -48%); }
    to   { opacity: 1; transform: translate(-50%, -50%); }
  }
</style>
