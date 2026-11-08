<script lang="ts">
  import { workspace } from "../state/workspace.svelte";
  import Menu from "./Menu.svelte";

  let confirmingDelete = $state(false);

  const statusLabel = $derived(
    {
      idle: "",
      dirty: "Unsaved",
      saving: "Saving…",
      saved: "Saved",
      error: "Save failed",
    }[workspace.saveState],
  );

  function onTitleInput(event: Event) {
    const title = (event.currentTarget as HTMLInputElement).value;
    if (workspace.current) workspace.updateDraft(title, workspace.current.body);
  }
</script>

<header class="bar">
  {#if workspace.current}
    <input
      class="title"
      value={workspace.current.title}
      oninput={onTitleInput}
      onblur={() => workspace.flush()}
      placeholder="Untitled"
      aria-label="Note title"
      spellcheck="false"
    />
    <span class="status" class:error={workspace.saveState === "error"}>{statusLabel}</span>
    {#if confirmingDelete}
      <span class="confirm">
        Delete this note?
        <button type="button" class="danger" onclick={() => { confirmingDelete = false; workspace.deleteNote(workspace.current!.id); }}>Delete</button>
        <button type="button" onclick={() => (confirmingDelete = false)}>Cancel</button>
      </span>
    {:else}
      <button type="button" title="Delete note" onclick={() => (confirmingDelete = true)}>🗑</button>
    {/if}
  {:else}
    <span class="placeholder">Select or create a note</span>
  {/if}
  <Menu />
</header>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    min-height: 44px;
  }
  .title {
    flex: 1;
    font-size: 16px;
    font-weight: 600;
    border: none;
    background: transparent;
    padding: 4px 6px;
    border-radius: 4px;
    min-width: 0;
  }
  .title:focus {
    outline: 1px solid var(--accent);
  }
  .placeholder {
    flex: 1;
    color: var(--fg-muted);
  }
  .status {
    font-size: 12px;
    color: var(--fg-muted);
    min-width: 64px;
    text-align: right;
  }
  .status.error {
    color: var(--danger);
  }
  button {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-muted);
    padding: 4px 8px;
    cursor: pointer;
    font-size: 12px;
  }
  .confirm {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    font-size: 12px;
    color: var(--fg-muted);
  }
  .danger {
    color: var(--danger);
    border-color: var(--danger);
  }
</style>
