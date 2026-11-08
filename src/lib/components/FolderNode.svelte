<script lang="ts">
  import type { Folder } from "../api";
  import { workspace } from "../state/workspace.svelte";
  import { focusOnMount } from "../utils/focus";
  import FolderNode from "./FolderNode.svelte";

  interface Props {
    folder: Folder;
    depth: number;
  }

  let { folder, depth }: Props = $props();

  let expanded = $state(true);
  let renaming = $state(false);
  let confirmingDelete = $state(false);
  let addingChild = $state(false);
  let draft = $state("");

  const children = $derived(workspace.folders.filter((f) => f.parent_id === folder.id));
  const selected = $derived(workspace.selectedFolderId === folder.id);

  async function commitRename() {
    if (draft.trim() && draft.trim() !== folder.name) {
      await workspace.renameFolder(folder.id, draft);
    }
    renaming = false;
  }

  async function commitChild() {
    if (draft.trim()) {
      const ok = await workspace.createFolder(folder.id, draft);
      if (ok) expanded = true;
    }
    addingChild = false;
  }

  function onKey(event: KeyboardEvent, commit: () => void, cancel: () => void) {
    if (event.key === "Enter") {
      event.preventDefault();
      commit();
    } else if (event.key === "Escape") {
      cancel();
    }
  }
</script>

<li>
  <div class="node" class:selected style:padding-left="{8 + depth * 14}px">
    <button class="twisty" type="button" onclick={() => (expanded = !expanded)} aria-label={expanded ? "Collapse" : "Expand"}>
      {children.length ? (expanded ? "▾" : "▸") : "·"}
    </button>

    {#if renaming}
      <input
        class="inline"
        bind:value={draft}
        onblur={commitRename}
        onkeydown={(e) => onKey(e, commitRename, () => (renaming = false))}
        aria-label="Folder name"
        use:focusOnMount
      />
    {:else}
      <button class="name" type="button" onclick={() => workspace.selectFolder(folder.id)} title={folder.name}>
        {folder.name}
      </button>
      <span class="actions">
        <button type="button" title="New subfolder" onclick={() => { draft = ""; addingChild = true; }}>+</button>
        <button type="button" title="Rename" onclick={() => { draft = folder.name; renaming = true; }}>✎</button>
        <button type="button" title="Delete" onclick={() => (confirmingDelete = true)}>🗑</button>
      </span>
    {/if}
  </div>

  {#if confirmingDelete}
    <div class="confirm" style:padding-left="{30 + depth * 14}px">
      Delete "{folder.name}" and everything inside?
      <button type="button" class="danger" onclick={() => { confirmingDelete = false; workspace.deleteFolder(folder.id); }}>Delete</button>
      <button type="button" onclick={() => (confirmingDelete = false)}>Cancel</button>
    </div>
  {/if}

  {#if addingChild}
    <div class="node" style:padding-left="{8 + (depth + 1) * 14}px">
      <span class="twisty">·</span>
      <input
        class="inline"
        bind:value={draft}
        placeholder="Folder name"
        onblur={commitChild}
        onkeydown={(e) => onKey(e, commitChild, () => (addingChild = false))}
        aria-label="New folder name"
        use:focusOnMount
      />
    </div>
  {/if}

  {#if expanded && children.length}
    <ul>
      {#each children as child (child.id)}
        <FolderNode folder={child} depth={depth + 1} />
      {/each}
    </ul>
  {/if}
</li>

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    list-style: none;
  }
  .node {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 28px;
    padding-right: 6px;
    border-radius: 6px;
  }
  .node:hover,
  .node.selected {
    background: var(--bg-muted);
  }
  .node.selected .name {
    color: var(--accent);
    font-weight: 600;
  }
  .twisty {
    width: 18px;
    text-align: center;
    color: var(--fg-muted);
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }
  .name {
    flex: 1;
    text-align: left;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .actions {
    display: none;
    gap: 2px;
  }
  .node:hover .actions {
    display: inline-flex;
  }
  .actions button {
    background: none;
    border: none;
    padding: 0 3px;
    cursor: pointer;
    color: var(--fg-muted);
    font-size: 12px;
  }
  .actions button:hover {
    color: var(--fg);
  }
  .inline {
    flex: 1;
    min-width: 0;
    padding: 2px 6px;
    border: 1px solid var(--accent);
    border-radius: 4px;
    background: var(--bg);
  }
  .confirm {
    font-size: 12px;
    color: var(--fg-muted);
    padding: 4px 8px 6px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .confirm button {
    font-size: 12px;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg);
    cursor: pointer;
  }
  .confirm .danger {
    color: var(--danger);
    border-color: var(--danger);
  }
</style>
