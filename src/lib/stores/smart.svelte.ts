import type { Prompt } from "../ipc";

/**
 * Smart Select prompt buffer — points and boxes the user has dropped on the
 * canvas to tell the model what to keep / what to drop. We capture them now so
 * the UI is real and discoverable; the actual decoder call is wired in Phase 2
 * once MobileSAM is bundled.
 */
class SmartStore {
  prompts = $state<Prompt[]>([]);

  add(prompt: Prompt) {
    this.prompts.push(prompt);
  }

  removeLast() {
    this.prompts.pop();
  }

  clear() {
    this.prompts = [];
  }

  get count() { return this.prompts.length; }
}

export const smart = new SmartStore();
