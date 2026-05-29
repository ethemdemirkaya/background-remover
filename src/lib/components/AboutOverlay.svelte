<script lang="ts">
  interface Props {
    open: boolean;
    onClose: () => void;
  }
  let { open, onClose }: Props = $props();

  const version = "0.1.0";

  const credits = [
    {
      name: "BriaAI RMBG-1.4",
      role: "Background matte model",
      license: "RAIL-M, non-commercial",
      url: "https://huggingface.co/briaai/RMBG-1.4",
    },
    {
      name: "ONNX Runtime",
      role: "On-device inference engine",
      license: "MIT",
      url: "https://onnxruntime.ai",
    },
    {
      name: "Tauri 2",
      role: "Cross-platform desktop shell",
      license: "MIT / Apache-2.0",
      url: "https://tauri.app",
    },
    {
      name: "Svelte 5",
      role: "UI runtime",
      license: "MIT",
      url: "https://svelte.dev",
    },
    {
      name: "image-rs",
      role: "Image I/O and pixel ops",
      license: "MIT / Apache-2.0",
      url: "https://github.com/image-rs/image",
    },
    {
      name: "Geist",
      role: "UI and monospace typefaces",
      license: "SIL Open Font License",
      url: "https://vercel.com/font",
    },
  ];
</script>

{#if open}
  <div class="scrim" onclick={onClose} role="presentation"></div>
  <div class="sheet" role="dialog" aria-label="About" aria-modal="true">
    <header>
      <div>
        <h2>Background Remover</h2>
        <p class="muted small mono">v{version}</p>
      </div>
      <button type="button" class="close" aria-label="Close" onclick={onClose}>×</button>
    </header>

    <section class="privacy">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>
      <div>
        <strong>100% on-device.</strong>
        <p class="muted small">No images are uploaded, no telemetry is collected. The app has no network calls for any image data — models are bundled with the install and inference runs locally on CPU.</p>
      </div>
    </section>

    <section>
      <h3>Built with</h3>
      <ul>
        {#each credits as c}
          <li>
            <div class="row">
              <span class="name">{c.name}</span>
              <span class="mono faint small">{c.license}</span>
            </div>
            <p class="muted small">{c.role} · <span class="mono faint">{new URL(c.url).host}</span></p>
          </li>
        {/each}
      </ul>
    </section>
  </div>
{/if}

<style>
  .scrim {
    position: fixed; inset: 0;
    background: rgba(0, 0, 0, 0.55);
    z-index: 50;
    animation: fade var(--dur) var(--ease);
  }
  .sheet {
    position: fixed; z-index: 51;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    width: min(520px, calc(100vw - var(--s5) * 2));
    max-height: calc(100vh - var(--s5) * 2);
    overflow-y: auto;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow);
    animation: pop var(--dur) var(--ease);
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: var(--s5) var(--s5) var(--s4);
    border-bottom: 1px solid var(--border);
  }
  header h2 { margin: 0; font-size: var(--fs-20); font-weight: 500; letter-spacing: -0.01em; }
  header p { margin: 2px 0 0 0; }
  .close {
    width: 28px; height: 28px;
    color: var(--text-muted);
    border-radius: var(--r-sm);
    font-size: 20px; line-height: 1;
  }
  .close:hover { background: var(--bg-hover); color: var(--text); }
  section { padding: var(--s4) var(--s5); }
  section + section { border-top: 1px solid var(--border); }
  section h3 {
    margin: 0 0 var(--s3) 0;
    font-size: var(--fs-12);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }
  .privacy {
    display: flex;
    gap: var(--s3);
    align-items: flex-start;
    color: var(--text);
  }
  .privacy svg { color: var(--accent); margin-top: 2px; flex-shrink: 0; }
  .privacy strong { font-weight: 500; }
  .privacy p { margin: var(--s1) 0 0 0; }
  ul { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: var(--s3); }
  .row { display: flex; align-items: baseline; justify-content: space-between; gap: var(--s3); }
  .name { font-weight: 500; }
  .small { font-size: var(--fs-12); }
  @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
  @keyframes pop {
    from { opacity: 0; transform: translate(-50%, -48%); }
    to   { opacity: 1; transform: translate(-50%, -50%); }
  }
</style>
