<script lang="ts">
  import { workspace } from "../state/workspace.svelte";
  import { formatRelative } from "../utils/time";

  const folderName = $derived(
    workspace.selectedFolderId === null
      ? "All notes"
      : (workspace.folders.find((f) => f.id === workspace.selectedFolderId)?.name ?? "Folder"),
  );
</script>

<section class="list" aria-label="Notes">
  <div class="header">
    <span class="title" title={folderName}>{folderName}</span>
    <button type="button" class="new" onclick={() => workspace.createNote()} title="New note (⌘/Ctrl+N)">
      + New
    </button>
  </div>

  {#if workspace.notes.length === 0}
    <p class="empty">No notes here yet.</p>
  {:else}
    <ul>
      {#each workspace.notes as note (note.id)}
        <li>
          <button
            type="button"
            class:selected={workspace.selectedNoteId === note.id}
            onclick={() => workspace.selectNote(note.id)}
          >
            <span class="note-title">{note.title}</span>
            <span class="meta">{formatRelative(note.updated_at)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .list {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }
  .title {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .new {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-muted);
    padding: 4px 8px;
    cursor: pointer;
    font-size: 12px;
  }
  .empty {
    color: var(--fg-muted);
    padding: 16px 12px;
    margin: 0;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 6px;
    overflow-y: auto;
  }
  li button {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: left;
    background: none;
    border: none;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: pointer;
  }
  li button:hover {
    background: var(--bg-muted);
  }
  li button.selected {
    background: var(--bg-muted);
    outline: 1px solid var(--accent);
  }
  .note-title {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
  }
  .meta {
    font-size: 11px;
    color: var(--fg-muted);
  }
</style>
