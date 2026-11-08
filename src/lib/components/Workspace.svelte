<script lang="ts">
  import { onMount } from "svelte";
  import { workspace } from "../state/workspace.svelte";
  import { vault } from "../state/vault.svelte";
  import FolderTree from "./FolderTree.svelte";
  import NoteList from "./NoteList.svelte";
  import TitleBar from "./TitleBar.svelte";
  import Editor from "./Editor.svelte";
  import Search from "./Search.svelte";

  let search: Search | undefined = $state();

  function onKeydown(event: KeyboardEvent) {
    const mod = event.metaKey || event.ctrlKey;
    if (!mod) return;
    const key = event.key.toLowerCase();
    if (key === "k") {
      event.preventDefault();
      search?.focus();
    } else if (key === "n") {
      event.preventDefault();
      workspace.createNote();
    } else if (key === "l") {
      event.preventDefault();
      vault.lock();
    }
  }

  function flushOnBlur() {
    void workspace.flush();
  }

  onMount(() => {
    window.addEventListener("blur", flushOnBlur);
    window.addEventListener("beforeunload", flushOnBlur);
    return () => {
      window.removeEventListener("blur", flushOnBlur);
      window.removeEventListener("beforeunload", flushOnBlur);
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="workspace">
  <aside class="sidebar">
    <Search bind:this={search} />
    <FolderTree />
  </aside>
  <NoteList />
  <main class="main">
    <TitleBar />
    {#if workspace.error}
      <p class="error" role="alert">{workspace.error}</p>
    {/if}
    {#if workspace.current}
      {#key workspace.current.id}
        <Editor
          value={workspace.current.body}
          onchange={(body) => workspace.updateDraft(workspace.current?.title ?? "", body)}
        />
      {/key}
    {:else}
      <div class="empty">
        <p>Select a note, or press <kbd>⌘/Ctrl</kbd>+<kbd>N</kbd> to create one.</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .workspace {
    height: 100%;
    display: grid;
    grid-template-columns: 240px 280px 1fr;
    overflow: hidden;
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-muted);
  }
  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }
  .error {
    margin: 0;
    padding: 6px 12px;
    color: var(--danger);
    font-size: 12px;
    border-bottom: 1px solid var(--border);
  }
  .empty {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--fg-muted);
  }
  kbd {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0 4px;
    font-size: 12px;
  }
</style>
