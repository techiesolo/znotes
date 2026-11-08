<script lang="ts">
  import { workspace } from "../state/workspace.svelte";
  import { focusOnMount } from "../utils/focus";
  import FolderNode from "./FolderNode.svelte";

  let addingRoot = $state(false);
  let draft = $state("");

  const roots = $derived(workspace.folders.filter((f) => f.parent_id === null));

  async function commitRoot() {
    if (draft.trim()) await workspace.createFolder(null, draft);
    addingRoot = false;
  }
</script>

<nav class="tree" aria-label="Folders">
  <div class="header">
    <span>Folders</span>
    <button type="button" title="New folder" onclick={() => { draft = ""; addingRoot = true; }}>+</button>
  </div>

  <button class="all" class:selected={workspace.selectedFolderId === null} type="button" onclick={() => workspace.selectFolder(null)}>
    All notes
  </button>

  {#if addingRoot}
    <input
      class="inline"
      bind:value={draft}
      placeholder="Folder name"
      onblur={commitRoot}
      onkeydown={(e) => {
        if (e.key === "Enter") { e.preventDefault(); commitRoot(); }
        if (e.key === "Escape") addingRoot = false;
      }}
      aria-label="New folder name"
      use:focusOnMount
    />
  {/if}

  <ul>
    {#each roots as folder (folder.id)}
      <FolderNode {folder} depth={0} />
    {/each}
  </ul>
</nav>

<style>
  .tree {
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted);
    padding: 4px 8px;
  }
  .header button {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--fg-muted);
    font-size: 14px;
  }
  .all {
    text-align: left;
    background: none;
    border: none;
    padding: 6px 8px;
    border-radius: 6px;
    cursor: pointer;
  }
  .all:hover,
  .all.selected {
    background: var(--bg-muted);
  }
  .all.selected {
    color: var(--accent);
    font-weight: 600;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .inline {
    margin: 0 8px;
    padding: 3px 6px;
    border: 1px solid var(--accent);
    border-radius: 4px;
    background: var(--bg);
  }
</style>
