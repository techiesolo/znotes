<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    onclose: () => void;
    children: Snippet;
  }

  let { title, onclose, children }: Props = $props();

  function onKey(event: KeyboardEvent) {
    if (event.key === "Escape") onclose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="modal" role="dialog" aria-modal="true" aria-label={title}>
    <header>
      <h2>{title}</h2>
      <button type="button" class="close" onclick={onclose} aria-label="Close">×</button>
    </header>
    <div class="body">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: grid;
    place-items: center;
    z-index: 100;
  }
  .modal {
    width: min(440px, 92vw);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.25);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    border-bottom: 1px solid var(--border);
  }
  h2 {
    margin: 0;
    font-size: 15px;
  }
  .close {
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    color: var(--fg-muted);
    line-height: 1;
  }
  .body {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .body :global(label) {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    color: var(--fg-muted);
  }
  .body :global(input) {
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--fg);
    font-size: 14px;
  }
  .body :global(.actions) {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 6px;
  }
  .body :global(.actions button) {
    padding: 7px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-muted);
    cursor: pointer;
  }
  .body :global(.actions .primary) {
    background: var(--accent);
    color: var(--accent-fg);
    border-color: transparent;
  }
  .body :global(.actions .danger) {
    color: var(--danger);
    border-color: var(--danger);
  }
  .body :global(button:disabled) {
    opacity: 0.6;
    cursor: default;
  }
  .body :global(.error) {
    margin: 0;
    color: var(--danger);
  }
  .body :global(.note) {
    margin: 0;
    color: var(--fg-muted);
    font-size: 12px;
  }
</style>
